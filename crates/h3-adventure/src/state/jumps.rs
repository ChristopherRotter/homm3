//! The teleporters' jumps (E08 T105a1; ADV-TEL-003, ADV-TEL-004, ADV-TEL-005, ADV-TEL-007;
//! increment 11's design note §7 and §10, survey `E08-teleporters.md` §4 and §5): the visit of a
//! monolith or a Subterranean Gate the hero stands on, the step onto it having been priced, paid
//! and revealed as any step's (its walk ends there, D-58). The exit comes from the static map's
//! channels and pairs (ADV-TEL-002, E08 T104), read here alone; a monolith's is drawn among its
//! candidates from the visitor's stream (ADR-0016 point 4, ADR-0017 point 4), after the whole
//! command was checked. The Whirlpool's jump, its loss first, is T105a2's; the battle at an
//! occupied exit and a gate's meeting are T105b's (`ADV-STUB-16`).

use alloc::sync::Arc;

use h3_content::Ruleset;
use h3_contracts::{HeroId, PlayerId};
use h3_heroes::HeroDetails;
use h3_kernel::Rng;

use super::{AdventureState, StepVisit};
use crate::command::{AdventureEvent, AdventureEventKind, Sink, Viewers};
use crate::fog::reveal;
use crate::movement::{details_of, hero_radius};
use crate::ruleset::{AdventureRuleset, MonolithWay};
use crate::setup::Pos;

type Rules<'a> = (&'a Ruleset, &'a AdventureRuleset, &'a [Arc<HeroDetails>]);

impl AdventureState {
    /// A monolith's visit by `player`'s hero `hero`, which stands on it (ADV-TEL-004; the
    /// original's cases 43 to 45 of `visit_object` 0x4A84F0, survey §4.2): a one-way entrance
    /// jumps to an exit drawn among its channel's one-way exits, a two-way monolith to one drawn
    /// among its channel's other two-way monoliths ([`Self::draw_exit`]); none to draw from,
    /// nothing at all (no event: the original's silence, batch E11's E11-4). A one-way exit leads
    /// nowhere, told to the visitor's owner alone (the original's ADVEVENT 70 for a human, every
    /// owner's here, decision 16; E11-5).
    // rule: ADV-TEL-004
    pub(super) fn visit_monolith<S: Sink>(
        &mut self,
        rules: Rules<'_>,
        (player, hero): (PlayerId, HeroId),
        (visit, (channel, way)): (StepVisit<'_>, (u8, MonolithWay)),
        draws: &mut Rng,
        sink: &mut S,
    ) {
        let channels = self.map.channels();
        let tiles = match way {
            MonolithWay::Entrance => channels.exits(channel),
            MonolithWay::TwoWay => channels.two_way(channel),
            MonolithWay::Exit => return self.leads_nowhere((player, hero), visit, sink),
        };
        if let Some(exit) = self.draw_exit(player, tiles, visit.tile, draws) {
            self.jump(rules, (player, hero), (visit.tile, exit), sink);
        }
    }

    /// A Subterranean Gate's visit by `player`'s hero `hero`, which stands on it (ADV-TEL-005;
    /// the original's case 103, `gate_exit` 0x4CDE40, survey §4.3): the jump to its pair, with no
    /// draw (batch E11's E11-9); a gate without a pair leads nowhere, told to the visitor's owner
    /// alone (ADVEVENT 153; E11-8). A hero on the pair, of any team, is the meeting's or the
    /// battle's ([`Self::jump`]).
    // rule: ADV-TEL-005
    pub(super) fn visit_gate<S: Sink>(
        &mut self,
        rules: Rules<'_>,
        (player, hero): (PlayerId, HeroId),
        visit: StepVisit<'_>,
        sink: &mut S,
    ) {
        match self.map.channels().gate_pair(visit.tile) {
            Some(exit) => self.jump(rules, (player, hero), (visit.tile, exit), sink),
            None => self.leads_nowhere((player, hero), visit, sink),
        }
    }

    /// The exit of `player`'s jump from `entrance` among `tiles`, a channel's list in the walk's
    /// order (ADV-TEL-003; `teleport_exit_draw` 0x4CDB80, survey §4.1, read whole; D-53, D-57 as
    /// the original): over the list, a tile is a **candidate** when a hero of another team stands
    /// on it, whatever lies there (a team is its player until E10's allies), or when no hero and
    /// no empty boat stand on it and it is not `entrance` (a two-way monolith excludes its own
    /// tile; the mover stands there anyway, a hero of its team). *Agent's call* (design note §7):
    /// a tile with an empty boat is no candidate. None: no exit. **One: no draw** (the original's
    /// `random_between` returns equal bounds at once). Else one uniform draw of the candidate's
    /// place among them from the visitor's stream, which no profile can reproduce (the original
    /// draws from the C runtime's one stream; the spread uniform as read, batch E11's E11-15).
    // rule: ADV-TEL-003
    fn draw_exit(
        &self,
        player: PlayerId,
        tiles: &[Pos],
        entrance: Pos,
        draws: &mut Rng,
    ) -> Option<Pos> {
        let candidate = |tile: &&Pos| match self.heroes.iter().find(|h| h.pos == **tile) {
            Some(other) => other.owner != player,
            None => **tile != entrance && !self.is_empty_boat(**tile),
        };
        let count = u32::try_from(tiles.iter().filter(candidate).count()).unwrap_or(u32::MAX);
        let place = match count {
            0 => return None,
            1 => 0,
            count => usize::try_from(draws.below(count)).unwrap_or(usize::MAX),
        };
        tiles.iter().filter(candidate).nth(place).copied()
    }

    /// The jump of `player`'s hero `hero` from `entrance` to `exit`: **the arrival** when no hero
    /// stands on the exit ([`Self::arrive`]); else the tiles within 1 of the exit are explored by
    /// the mover's owner, with no event (`reveal_disk` with radius 1 before `visit_hero`, survey
    /// §4.2, §4.3; batch E11's E11-7), and the hero stays on the entrance.
    fn jump<S: Sink>(
        &mut self,
        rules: Rules<'_>,
        (player, hero): (PlayerId, HeroId),
        (entrance, exit): (Pos, Pos),
        sink: &mut S,
    ) {
        if self.heroes.iter().any(|h| h.pos == exit) {
            // stub: ADV-STUB-16 (the battle with an enemy hero on the exit, GAM-BAT-004, and a
            // gate's meeting with one of the team on its pair, GAM-MET-001: E08 T105b, design
            // note §12)
            reveal(&mut self.explored[usize::from(player.raw())], exit, 1);
            return;
        }
        self.arrive(rules, (player, hero), (entrance, exit), sink);
    }

    /// **The arrival** of `player`'s hero `hero` from `entrance` on the free `exit` (ADV-TEL-007;
    /// `hero_relocate` 0x41D930, survey §5; batch E11's E11-2, E11-3): its tile becomes the exit,
    /// its points and the day's maximum kept; the tiles within its scouting radius of the exit
    /// explored by its owner (ADV-FOG-002's disk, as a step's); **no guard test and no visit**
    /// there (a two-way exit is not entered again); the move ends. Told as
    /// [`AdventureEventKind::HeroTeleported`] (ADV-FOG-004).
    // rule: ADV-TEL-007, ADV-FOG-004
    fn arrive<S: Sink>(
        &mut self,
        (core, part, heroes): Rules<'_>,
        (player, hero): (PlayerId, HeroId),
        (entrance, exit): (Pos, Pos),
        sink: &mut S,
    ) {
        let index = self.heroes.partition_point(|h| h.id < hero);
        self.heroes[index].pos = exit;
        let radius = hero_radius(core, part, details_of(heroes, hero));
        reveal(&mut self.explored[usize::from(player.raw())], exit, radius);
        let (seen_from, seen_to) = (
            self.viewers(core, player, entrance),
            self.viewers(core, player, exit),
        );
        sink.emit(AdventureEvent {
            kind: AdventureEventKind::HeroTeleported {
                hero,
                from: Some(entrance),
                to: Some(exit),
                seen_from,
                seen_to,
            },
            seen_by: seen_from.union(seen_to),
        });
    }

    /// That `player`'s hero `hero`'s teleporter of `visit` leads nowhere (ADV-TEL-004,
    /// ADV-TEL-005): told to the owner alone, by the object's type and anchor.
    fn leads_nowhere<S: Sink>(
        &self,
        (player, hero): (PlayerId, HeroId),
        visit: StepVisit<'_>,
        sink: &mut S,
    ) {
        sink.emit(AdventureEvent {
            kind: AdventureEventKind::LeadsNowhere {
                hero,
                object_type: visit.object_type,
                anchor: self.object_at(visit.object).anchor,
            },
            seen_by: Viewers::NONE.with(player),
        });
    }
}
