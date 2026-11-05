//! The town on the adventure map (E09 T8a2; the town epic's design note §9 and §11; survey
//! `E09-towns.md` §7, §8): the step onto a town's entrance by its owner's team and the town's
//! garrison (TWN-VIS-001), the visitor, the hero standing on the entrance (TWN-VIS-002), and the
//! capture of an undefended town with its reveal (TWN-CAP-001). The siege's end (E09 T8b1): its
//! garrison's survivors and the capture after an attacker's win (TWN-CAP-003). The siege's start
//! (E09 T8b2a): a town its garrison alone defends is visited from the tile before, the siege
//! against the garrison (TWN-SIE-001). Since E09 T8b2b a town a visitor of another team defends,
//! the step onto the town or onto that visitor the same siege, against the visitor (TWN-SIE-001,
//! TWN-VIS-002), with the garrison merged into its army first (TWN-SIE-002, [`merge`]).

use alloc::vec::Vec;

use h3_content::{Formation, Ruleset};
use h3_contracts::{ArmyStack, HeroId, PlayerId, TownId};

use crate::army::{ArmyRef, ArmySlot};
use crate::command::{
    AdventureEvent, AdventureEventKind, AdventureOutcome, BattleStart, Defender, Sink, Viewers,
};
use crate::fog;
use crate::persistent::BitGrid;
use crate::ruleset::{AdventureRuleset, Visit};
use crate::setup::{Holds, Pos};
use crate::state::{AdventureState, HeroOnMap, Room, StaticMap, allied, join_at, room};

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

    /// The formation of `town`'s garrison when it defends without a hero (E09 T8b2a; the town
    /// epic's design note §8, *agent's call*: the map's byte, [`Holds::Town`]); none for a
    /// `TownId` the map does not have.
    pub fn town_formation(&self, town: TownId) -> Option<Formation> {
        self.town_object(town)
            .and_then(|index| holds_formation(&self.objects()[index].holds))
    }
}

/// The formation `holds` names: a town's ([`Holds::Town`], which every town holds: the setup
/// refuses another); none for every other holding.
pub(crate) const fn holds_formation(holds: &Holds) -> Option<Formation> {
    match holds {
        Holds::Town { formation } => Some(*formation),
        _ => None,
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
    /// town's entrance, the town the tile's object ([`Self::tile_index`], the object
    /// [`Self::visit_at`] reads, so that the step and the reachable tiles meet the same one). The
    /// step then visits the town instead of the hero (E09 T8b2b: from the tile before, the siege
    /// against that visitor, [`Self::town_visit`]), never a field battle.
    // rule: TWN-VIS-002
    #[inline(never)]
    pub(crate) fn besieges(&self, player: PlayerId, other: &HeroOnMap) -> bool {
        let map = self.map();
        !allied(other.owner, player)
            && self.tile_index(other.pos).is_some_and(|index| {
                map.town_id(index).is_some()
                    && map.objects()[index].entrance(map) == Some(other.pos)
            })
    }

    /// How `player`'s hero visits the town at object index `index` (TWN-VIS-001; survey
    /// `E09-towns.md` §7.1, the original's step 0x480E3B with `town_defended` 0x5BDFE0): entered
    /// ([`Visit::Enter`], the type's) when the town is of its team or undefended; from the tile
    /// before ([`Visit::Neighbour`], ADV-OBJ-003's neighbour mode: the step paid, ADV-MOV-006's
    /// drop, the hero stays; E09 T8b2a) when its owner is of another team or none and a creature
    /// stands in its garrison, the siege against the garrison ([`Self::visit_town`],
    /// TWN-SIE-001); from the tile before too, whoever owns it, when a visitor of another team
    /// stands on its entrance (TWN-VIS-002, E09 T8b2b), the siege against that visitor (the
    /// survey's §7.2 step 3: a visiting hero defends; an own town's included, which only a map
    /// placing another player's hero on its entrance gives, TWN-SET-010). The step meets such a
    /// visitor first ([`Self::besieges`]) and visits the town here; the walking search, which
    /// leaves its tile unmarked, meets it here too. Always a visit (an `Option` for
    /// [`Self::visit_at`], whose other kinds may have none).
    // rule: TWN-VIS-001, TWN-VIS-002
    #[inline(never)]
    pub(crate) fn town_visit(&self, index: usize, player: PlayerId) -> Option<Visit> {
        let map = self.map();
        let visitor = map.objects()[index].entrance(map).is_some_and(|entrance| {
            self.heroes()
                .iter()
                .any(|hero| hero.pos == entrance && !allied(hero.owner, player))
        });
        let own = self.owner(index).is_some_and(|owner| allied(owner, player));
        let defended = visitor
            || !own
                && self
                    .town_garrison(index)
                    .is_some_and(|army| !army.is_empty());
        Some(if defended {
            Visit::Neighbour
        } else {
            Visit::Enter
        })
    }

    /// The visit of the town at object index `index` by `player`'s hero `hero` (TWN-VIS-001; the
    /// town epic's design note §9; survey `E09-towns.md` §7.2, the original's `town_visit`
    /// 0x4AAFD0), as [`Self::town_visit`] said the step makes it (`mode`): a town of the hero's
    /// team gives nothing (no service, no decision, no event beyond the step's, *agent's call*);
    /// another player's or a neutral one, which the step entered undefended, the capture
    /// ([`AdventureOutcome::capture`]), which the game applies at once ([`Self::capture_town`]);
    /// one its garrison or a visitor of another team defends, visited from the tile before, the
    /// siege (E09 T8b2a, T8b2b; the town epic's design note §10, survey §9.1's second and last
    /// rows: [`Self::siege_start`]).
    // rule: TWN-VIS-001, TWN-SIE-001
    pub(crate) fn visit_town(
        &self,
        part: &AdventureRuleset,
        (player, hero): (PlayerId, HeroId),
        (index, mode): (usize, Visit),
    ) -> AdventureOutcome {
        if mode == Visit::Neighbour {
            return AdventureOutcome {
                battle: self.siege_start(part, hero, index),
                ..AdventureOutcome::default()
            };
        }
        let own = self.owner(index).is_some_and(|owner| allied(owner, player));
        AdventureOutcome {
            capture: self.map().town_id(index).filter(|_| !own),
            ..AdventureOutcome::default()
        }
    }

    /// The siege the hero `hero` starts against the town at object index `index` (TWN-SIE-001;
    /// E09 T8b2a, T8b2b; the town epic's design note §10; survey `E09-towns.md` §9.1, `town_visit`
    /// 0x4AB25A to 0x4AB338). Its visitor, the hero on its entrance (of another team: the step
    /// meets one of its own team as a hero), defends as itself ([`Defender::Hero`], its owner's
    /// side, its army as it stands: the game merges the garrison into it before the battle,
    /// [`Self::merge_garrison`], TWN-SIE-002). Without a visitor, the garrison as an army without
    /// a hero, its stacks by slot as they stand, unsplit: its owner's side for an owned town
    /// ([`Defender::Garrison`], played by its owner outside its turn, as a mine's guards,
    /// ADV-MIN-002), the neutral seat's for a neutral one ([`Defender::Guards`], *agent's call*: no
    /// new variant). Fought at the town's entrance ([`Self::battle_at`]; with a visitor in a boat
    /// never: an entrance is on land) as the battle of its town ([`BattleStart::town`]), which the
    /// game builds from the town (TWN-SIE-003 to -005). None when `hero` is not on the map or
    /// `index` is not a town's: never, the step checked both (a none would leave the step paid and
    /// no battle).
    // rule: TWN-SIE-001
    fn siege_start(
        &self,
        part: &AdventureRuleset,
        hero: HeroId,
        index: usize,
    ) -> Option<BattleStart> {
        let map = self.map();
        let entrance = map.objects()[index].entrance(map);
        let visitor =
            entrance.and_then(|entrance| self.heroes().iter().find(|h| h.pos == entrance));
        let army = self.town_garrison(index).unwrap_or_default().to_vec();
        let defender = match (visitor, self.owner(index)) {
            (Some(visitor), _) => Defender::Hero(visitor.combatant()),
            (None, Some(owner)) => Defender::Garrison { owner, army },
            (None, None) => Defender::Guards { army },
        };
        map.town_id(index)
            .zip(entrance)
            .zip(self.hero(hero))
            .map(|((town, entrance), attacker)| BattleStart {
                town: Some(town),
                ..self.battle_at(part, attacker, defender, entrance)
            })
    }

    /// Merges the garrison of `town` into its visitor's army before the siege the visitor defends
    /// (TWN-SIE-002; E09 T8b2b; the town epic's design note §10; survey `E09-towns.md` §9.2, the
    /// original's `garrison_merge` 0x44B820, TD-7 confirmed by batch TWN-1's row 3): [`merge`]
    /// with the part's army slots and a stack's most creatures (ADV-ARM-007), both armies
    /// written. Returns the slots it changed, the visitor's ([`ArmyRef::Hero`]) then the
    /// garrison's ([`ArmyRef::Garrison`]), each in slot order, which the game tells the visitor's
    /// owner; none without a visitor or a change. The game calls it as it starts the siege the
    /// step gave ([`Self::siege_start`]'s [`Defender::Hero`]); what stays in the garrison does not
    /// fight, and is lost if the town falls (its capture empties it, TWN-CAP-001).
    // rule: TWN-SIE-002
    pub fn merge_garrison(
        &mut self,
        core: &Ruleset,
        part: &AdventureRuleset,
        town: TownId,
    ) -> Vec<ArmySlot> {
        let params = part.params();
        let rules = (params.army_slots(), params.max_stack_count());
        let index = self.map().town_object(town);
        let garrison = index.and_then(|index| self.town_garrison(index));
        let mut garrison = garrison.unwrap_or_default().to_vec();
        let visitor = self.town_visitor(town);
        let Some((id, hero)) = visitor.and_then(|id| Some(id).zip(self.hero_mut(id))) else {
            return Vec::new();
        };
        let before = (hero.army.clone(), garrison.clone());
        merge(core, rules, &mut hero.army, &mut garrison);
        let mut slots = Vec::new();
        changed_slots(
            &mut slots,
            ArmyRef::Hero(id),
            rules.0,
            (&before.0, &hero.army),
        );
        let start = slots.len();
        changed_slots(
            &mut slots,
            ArmyRef::Garrison(town),
            rules.0,
            (&before.1, &garrison),
        );
        if slots.len() > start {
            // A change in the garrison is written; an unchanged one keeps its block as it was.
            self.set_army_left(index.unwrap_or_default(), garrison);
        }
        slots
    }

    /// Captures `town` for `player`, a player of the game (TWN-CAP-001; the town epic's design
    /// note §11; survey `E09-towns.md` §8.1, §8.3, the original's `town_change_owner` 0x4C61E0):
    /// its owner written (as a mine's flag writes it, ADV-OWN-001), its garrison written empty,
    /// told to the players that see the town and to its new owner
    /// ([`AdventureEventKind::TownCaptured`]), then what `reveal` shows around its entrance (its
    /// anchor for a town visited from no tile of the map) explored for the new owner, told to it
    /// alone with the number it had not explored ([`AdventureEventKind::TilesRevealed`]).
    /// Returns the players it told of the capture, `TownCaptured`'s viewers, whom the game tells
    /// of the town's change too (TWN-CAP-002). Infallible: the step checked it; a `TownId` the map
    /// does not have changes nothing (`None`). The game composes it: `h3_towns::reveal`, then
    /// this, then the Capitol's limit (TWN-CAP-002).
    // rule: TWN-CAP-001
    pub fn capture_town<S: Sink>(
        &mut self,
        core: &Ruleset,
        town: TownId,
        player: PlayerId,
        reveal: TownReveal,
        sink: &mut S,
    ) -> Option<Viewers> {
        let map = self.map();
        let (index, object_type) = map.town_object(town).and_then(|index| {
            map.objects()[index]
                .object_type
                .map(|object_type| (index, object_type))
        })?;
        self.set_owner(index, player);
        self.set_army_left(index, Vec::new());
        let object = self.object_at(index);
        let anchor = object.anchor;
        let entrance = object.entrance(self.map()).unwrap_or(anchor);
        let seen_by = self.object_viewers(core, object).with(player);
        sink.emit(AdventureEvent {
            kind: AdventureEventKind::TownCaptured {
                town,
                anchor,
                owner: player,
            },
            seen_by,
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
        Some(seen_by)
    }

    /// The end of a siege of the town at object index `index` that its garrison fought without a
    /// hero, against the visitor's hero `hero` at `tile`, the town's entrance (E09 T8b1; the town
    /// epic's design note §11; survey `E09-towns.md` §8.2, §9.3): after the visitor's side `won`
    /// the garrison is beaten, written empty, and the capture comes after the winner's levels
    /// ([`AdventureOutcome::reward_after`], then [`Self::give_won`]: [`Self::town_won`]); else its
    /// `survivors`, by slot, stay as its garrison, as a mine's guards' (none after a battle without
    /// a winner, and the town keeps its owner).
    // rule: TWN-CAP-003
    pub(crate) fn end_town_battle(
        &mut self,
        index: usize,
        (hero, tile): (HeroId, Pos),
        (survivors, won): (&[ArmyStack], bool),
    ) -> AdventureOutcome {
        let left = if won { Vec::new() } else { survivors.to_vec() };
        self.set_army_left(index, left);
        AdventureOutcome {
            reward_after: won.then_some((hero, tile)),
            ..AdventureOutcome::default()
        }
    }

    /// What a siege of `town` won by the hero `hero` gives after the winner's levels
    /// (TWN-CAP-003; the town epic's design note §11; survey `E09-towns.md` §8.2, the original's
    /// `town_visit` 0x4AB33D to 0x4AB3BA): the town's capture for the hero's owner
    /// ([`AdventureOutcome::capture`], which the game applies with its reveal and the Capitol's
    /// limit, as an undefended town's), unless the town is of the owner's team by then. The hero
    /// stays on the tile it attacked from (no town screen): a later step enters the town, which it
    /// finds its own. Nothing when the hero left the map (a win with only creatures the battle
    /// made, BAT-END-001), as a mine's flag (ADV-MIN-002).
    // rule: TWN-CAP-003
    pub(crate) fn town_won(&self, hero: HeroId, town: TownId) -> AdventureOutcome {
        let capture = self
            .hero(hero)
            .zip(self.map().town_object(town))
            .filter(|&(hero, index)| !self.owner(index).is_some_and(|o| allied(o, hero.owner)))
            .map(|_| town);
        AdventureOutcome {
            capture,
            ..AdventureOutcome::default()
        }
    }
}

/// Adds to `slots` each slot of `army`, below `count`, whose stack differs between `before` and
/// `after`, in slot order.
fn changed_slots(
    slots: &mut Vec<ArmySlot>,
    army: ArmyRef,
    count: u8,
    (before, after): (&[ArmyStack], &[ArmyStack]),
) {
    let at = |stacks: &[ArmyStack], slot| stacks.iter().copied().find(|s| s.slot == slot);
    slots.extend(
        (0..count)
            .filter(|&slot| at(before, slot) != at(after, slot))
            .map(|slot| ArmySlot { army, slot }),
    );
}

/// The garrison's merge into its visitor's army (TWN-SIE-002; the town epic's design note §10;
/// survey `E09-towns.md` §9.2, the original's `garrison_merge` 0x44B820, read whole; TD-7 by the
/// reading, confirmed by batch TWN-1's row 3), each stack's value its creature's AI value
/// (`CRTRAITS.TXT`'s, the record's +0x40) times its count, round after round:
///
/// 1. the visitor's weakest stack: the least value, of equal ones the last slot;
/// 2. each garrison stack's gain: its value when `army` has room for it (a stack of its creature
///    or a free slot, as the original's `army_add_stack` 0x44ACE0 finds it with a stack's most
///    creatures, ADV-ARM-007: [`room`]), else its value less the weakest's; the stack of the
///    greatest positive gain, of equal ones the first slot;
/// 3. it joins `army` where it has room ([`join_at`]) and leaves the garrison; without room it
///    changes places with the weakest stack (their creatures and counts swapped, each keeping its
///    slot), which stays in the garrison.
///
/// Until no stack gains: every round empties a garrison slot or raises the army's value by a
/// swap of the same stacks, so it ends. `rules`: the army slots and a stack's most creatures. A
/// creature the ruleset does not have is worth 0 (it gains nothing). VCMI's merge (the stack of
/// most power first, one round per garrison stack) is refuted by the batch (survey §15).
// rule: TWN-SIE-002
pub(crate) fn merge(
    core: &Ruleset,
    rules: (u8, u32),
    army: &mut Vec<ArmyStack>,
    garrison: &mut Vec<ArmyStack>,
) {
    let value = |stack: &ArmyStack| {
        let ai = core.creature(stack.creature).map_or(0, |c| c.ai_value);
        i128::from(ai) * i128::from(stack.count)
    };
    loop {
        // The visitor's weakest stack: the least value, of equal ones the last slot. A hero's
        // army is never empty (the setup and the army commands keep a stack).
        let Some(weakest) = army
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| value(a).cmp(&value(b)).then(b.slot.cmp(&a.slot)))
            .map(|(index, stack)| (index, *stack))
        else {
            return;
        };
        let worth = value(&weakest.1);
        let mut best: Option<(i128, u8, usize, Option<Room>)> = None;
        for (index, stack) in garrison.iter().enumerate() {
            let found = room(army, rules, stack.creature, stack.count);
            let gain = value(stack) - if found.is_some() { 0 } else { worth };
            let better = best
                .is_none_or(|(most, slot, ..)| gain > most || gain == most && stack.slot < slot);
            if gain > 0 && better {
                best = Some((gain, stack.slot, index, found));
            }
        }
        let Some((_, _, index, found)) = best else {
            return;
        };
        let joining = garrison[index];
        match found {
            Some(found) => {
                join_at(army, found, joining.creature, joining.count);
                garrison.remove(index);
            }
            None => {
                let (at, out) = weakest;
                army[at] = ArmyStack {
                    slot: out.slot,
                    ..joining
                };
                garrison[index] = ArmyStack {
                    slot: joining.slot,
                    ..out
                };
            }
        }
    }
}
