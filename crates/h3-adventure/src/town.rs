//! The town on the adventure map (E09 T8a2; the town epic's design note §9 and §11; survey
//! `E09-towns.md` §7, §8): the step onto a town's entrance by its owner's team and the town's
//! garrison (TWN-VIS-001), the visitor, the hero standing on the entrance (TWN-VIS-002), and the
//! capture of an undefended town with its reveal (TWN-CAP-001). The siege of a defended town comes
//! with E09 T8b (`TWN-STUB-7`); until then such a step is refused as a visit not built.

use alloc::vec::Vec;

use h3_content::Ruleset;
use h3_contracts::{HeroId, PlayerId, TownId};

use crate::command::{AdventureEvent, AdventureEventKind, AdventureOutcome, Sink, Viewers};
use crate::fog;
use crate::persistent::BitGrid;
use crate::setup::Pos;
use crate::state::{AdventureState, HeroOnMap, StaticMap, allied};

/// What a town reveals to its owner (TWN-CAP-001; the town epic's design note §11): at the start
/// of the game for each owned town ([`AdventureState::start_game`]) and at its capture
/// ([`AdventureState::capture_town`]); the town context's `h3_towns::reveal`, which the game
/// passes on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TownReveal {
    /// The tiles within this radius of its entrance on its level (ADV-FOG-002's disc).
    Radius(u8),
    /// Every tile of every level of the map (a Tower with its Grail, the original's
    /// `town_change_owner` 0x4C61E0; survey `E09-towns.md` §8.3 step 8).
    All,
}

impl TownReveal {
    /// Explores in `grid` what this reveal shows around `entrance`, a tile of `map`.
    pub(crate) fn reveal(self, grid: &mut BitGrid, map: &StaticMap, entrance: Pos) {
        match self {
            Self::Radius(radius) => fog::reveal(grid, entrance, radius),
            Self::All => {
                for level in 0..map.levels() {
                    for y in 0..map.height() {
                        grid.set_run(Pos::new(0, y, level), map.width(), true);
                    }
                }
            }
        }
    }
}

impl StaticMap {
    /// The town whose entrance is `pos` (E09 T8a2, the original's `town_at_entrance` 0x4BB870: the
    /// town whose gate is the tile), if one's is: the first tile it is visited from
    /// ([`crate::MapObject::entrance`]).
    pub fn town_at(&self, pos: Pos) -> Option<TownId> {
        self.visited_from(pos).iter().find_map(|&index| {
            self.town_id(index)
                .filter(|_| self.objects()[index].entrance(self) == Some(pos))
        })
    }
}

impl AdventureState {
    /// The town's visitor (TWN-VIS-002; the town epic's design note §9, survey `E09-towns.md` §7.3:
    /// the original's town +0x10, which `hero_show` and `hero_hide` keep): the hero standing on
    /// its entrance, if one does; never stored (ADR-0016's amendment point 2). None for a
    /// `TownId` the map does not have.
    // rule: TWN-VIS-002
    pub fn town_visitor(&self, town: TownId) -> Option<HeroId> {
        let map = self.map();
        map.town_object(town)
            .and_then(|index| map.objects()[index].entrance(map))
            .and_then(|entrance| self.heroes().iter().find(|hero| hero.pos == entrance))
            .map(|hero| hero.id)
    }

    /// Whether a step of `player`'s hero onto `other`, another hero, is the siege of the town it
    /// visits (TWN-VIS-002; survey `E09-towns.md` §7.1, the original's `visit_hero` 0x4A2800
    /// sending such a hero to the town's visit): `other` is of another team and stands on a
    /// town's entrance. The step and the reachable tiles refuse it until the siege (E09 T8b,
    /// `TWN-STUB-7`), never a field battle.
    // rule: TWN-VIS-002
    #[inline(never)]
    pub(crate) fn besieges(&self, player: PlayerId, other: &HeroOnMap) -> bool {
        !allied(other.owner, player) && self.map().town_at(other.pos).is_some()
    }

    /// Whether the town at object index `index` is defended against `player`'s hero
    /// (TWN-VIS-001's second case; survey `E09-towns.md` §7.1, the original's `town_defended`
    /// 0x5BDFE0): its owner of another team or none and a creature in its garrison, or, whoever
    /// owns it, a visitor of another team (TWN-VIS-002). The step meets a visitor first and
    /// refuses it ([`Self::besieges`]); the walking search, which leaves such a hero's tile
    /// unmarked, meets it here. Until the siege (E09 T8b, `TWN-STUB-7`) the step onto a defended
    /// town is refused as a visit not built, and the reachable tiles leave it out.
    // rule: TWN-VIS-001, TWN-VIS-002
    #[inline(never)]
    pub(crate) fn town_defended(&self, index: usize, player: PlayerId) -> bool {
        let map = self.map();
        let own = self.owner(index).is_some_and(|owner| allied(owner, player));
        (!own
            && self
                .town_garrison(index)
                .is_some_and(|army| !army.is_empty()))
            || map.objects()[index].entrance(map).is_some_and(|entrance| {
                self.heroes()
                    .iter()
                    .any(|hero| hero.pos == entrance && !allied(hero.owner, player))
            })
    }

    /// The visit of the town at object index `index` by `player`'s hero, which entered it
    /// (TWN-VIS-001; the town epic's design note §9; survey `E09-towns.md` §7.2, the original's
    /// `town_visit` 0x4AAFD0): a town of the hero's team gives nothing (no service, no decision,
    /// no event beyond the step's, *agent's call*); another player's or a neutral one, which the
    /// step entered undefended, the capture ([`AdventureOutcome::capture`]), which the game
    /// applies at once ([`Self::capture_town`]).
    // rule: TWN-VIS-001
    pub(crate) fn visit_town(&self, player: PlayerId, index: usize) -> AdventureOutcome {
        let own = self.owner(index).is_some_and(|owner| allied(owner, player));
        AdventureOutcome {
            capture: self.map().town_id(index).filter(|_| !own),
            ..AdventureOutcome::default()
        }
    }

    /// Captures `town` for `player`, a player of the game (TWN-CAP-001; the town epic's design
    /// note §11; survey `E09-towns.md` §8.1, §8.3, the original's `town_change_owner` 0x4C61E0):
    /// its owner written (as a mine's flag writes it, ADV-OWN-001), its garrison written empty,
    /// told to the players that see the town and to its new owner
    /// ([`AdventureEventKind::TownCaptured`]), then what `reveal` shows around its entrance (its
    /// anchor for a town visited from no tile of the map) explored for the new owner, told to it
    /// alone with the number it had not explored ([`AdventureEventKind::TilesRevealed`]).
    /// Infallible: the step checked it; a `TownId` the map does not have changes nothing. The
    /// game composes it: `h3_towns::reveal`, then this, then the Capitol's limit (TWN-CAP-002).
    // rule: TWN-CAP-001
    pub fn capture_town<S: Sink>(
        &mut self,
        core: &Ruleset,
        town: TownId,
        player: PlayerId,
        reveal: TownReveal,
        sink: &mut S,
    ) {
        let map = self.map();
        let Some((index, object_type)) = map.town_object(town).and_then(|index| {
            map.objects()[index]
                .object_type
                .map(|object_type| (index, object_type))
        }) else {
            return;
        };
        self.set_owner(index, player);
        self.set_army_left(index, Vec::new());
        let object = self.object_at(index);
        let anchor = object.anchor;
        let entrance = object.entrance(self.map()).unwrap_or(anchor);
        sink.emit(AdventureEvent {
            kind: AdventureEventKind::TownCaptured {
                town,
                anchor,
                owner: player,
            },
            seen_by: self.object_viewers(core, object).with(player),
        });
        let (grid, map) = self.explored_mut(player);
        let before = grid.count();
        reveal.reveal(grid, map, entrance);
        sink.emit(AdventureEvent {
            kind: AdventureEventKind::TilesRevealed {
                object_type,
                count: grid.count() - before,
            },
            seen_by: Viewers::NONE.with(player),
        });
    }
}
