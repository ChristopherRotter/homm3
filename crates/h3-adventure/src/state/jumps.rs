//! The teleporters' jumps (E08 T105a1; ADV-TEL-003, ADV-TEL-004, ADV-TEL-005, ADV-TEL-007;
//! increment 11's design note §7 and §10, survey `E08-teleporters.md` §4 and §5): the visit of a
//! monolith, a Subterranean Gate or (E08 T105a2, ADV-TEL-006, design note §8) a Whirlpool the
//! hero stands on, the step onto it having been priced, paid and revealed as any step's (its walk
//! ends there, D-58). The exit comes from the static map's channels and pairs (ADV-TEL-002, E08
//! T104), read here alone; a monolith's and a Whirlpool's is drawn among its candidates from the
//! visitor's stream (ADR-0016 point 4, ADR-0017 point 4), after the whole command was checked.
//! Since E08 T105b1 (design note §12, §13) a hero on the exit is met or fought from the entrance,
//! the jump after a won battle held as the visit's rest ([`AdventureState::arrive`]).

use alloc::sync::Arc;

use h3_content::Ruleset;
use h3_contracts::{ArmyStack, HeroId, PlayerId};
use h3_heroes::{HeroDetails, whirlpool_protection};
use h3_kernel::Rng;

use super::{AdventureState, StepVisit};
use crate::command::{
    AdventureEvent, AdventureEventKind, AdventureOutcome, Meeting, Sink, Viewers,
};
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
    ) -> AdventureOutcome {
        let channels = self.map.channels();
        let tiles = match way {
            MonolithWay::Entrance => channels.exits(channel),
            MonolithWay::TwoWay => channels.two_way(channel),
            MonolithWay::Exit => {
                self.leads_nowhere((player, hero), visit, sink);
                return AdventureOutcome::default();
            }
        };
        let listed = tiles.iter().map(|&tile| (tile, tile == visit.tile));
        match self.draw_exit(player, listed, draws) {
            Some(exit) => self.jump(rules, (player, hero), (visit.tile, exit), sink),
            None => AdventureOutcome::default(),
        }
    }

    /// A Whirlpool's visit by `player`'s hero `hero`, which stands in its boat on one of its
    /// tiles (ADV-TEL-006; the original's case 111 at 0x4A95A5, survey §4.4): first **the loss**
    /// ([`Self::whirlpool_loss`]) unless the hero is protected (`h3_heroes::whirlpool_protection`:
    /// the Sea Captain's Hat, alone or in the Admiral's Hat; a hero without details is not), then
    /// the jump to a tile drawn among the other Whirlpools' ([`Self::draw_exit`] over the
    /// Whirlpools' list, the entrance's own Whirlpool's tiles excluded by its object: the
    /// original's shared word, so one draw gives the object and the tile). The hero arrives with
    /// its boat, whose record keeps its boarded tile (ADR-0016 point 1); no candidate: it stays,
    /// the loss taken (the wiki: a lone Whirlpool only takes troops; batch E11's E11-10 to E11-13,
    /// E11-17).
    // rule: ADV-TEL-006
    pub(super) fn visit_whirlpool<S: Sink>(
        &mut self,
        rules: Rules<'_>,
        (player, hero): (PlayerId, HeroId),
        (visit, divisor): (StepVisit<'_>, u32),
        draws: &mut Rng,
        sink: &mut S,
    ) -> AdventureOutcome {
        let (core, _, heroes) = rules;
        if !details_of(heroes, hero).is_some_and(|details| whirlpool_protection(details, core)) {
            self.whirlpool_loss(core, (player, hero), divisor, sink);
        }
        let listed = (self.map.channels().whirlpools().iter())
            .map(|whirlpool| (whirlpool.tile, whirlpool.object == visit.object));
        match self.draw_exit(player, listed, draws) {
            Some(exit) => self.jump(rules, (player, hero), (visit.tile, exit), sink),
            None => AdventureOutcome::default(),
        }
    }

    /// **The Whirlpool's loss** of `player`'s hero `hero` with `divisor` (ADV-TEL-006;
    /// `whirlpool_losses` 0x4ACAA0, survey §4.4; D-54, D-55 as the original): its **weakest
    /// stack**, the least creature fight value × count as a 64-bit product, the first slot on a
    /// tie (the army is in slot order, the slot breaks the tie), keeps `count / divisor` rounded
    /// down, the slot emptied at 0, when the army holds more than one stack; a lone stack keeps
    /// `max(1, count / divisor)`. Told to the owner alone with the count lost, 0 for a lone
    /// creature kept (the original's ADVEVENT 168 shows then too). **A-38 not reproduced**
    /// (`ADV-QUIRK-40`, decision 266): the original ranks from 99 999 999 with an `i32` product,
    /// so a stack at that bound or past it is never the weakest (a lone stack of 1 776 Azure
    /// Dragons keeps them all, the wiki's "bug", batch E11's E11-12) and several such stacks
    /// corrupt the army. **A-40 not reproduced** (`ADV-QUIRK-42`, decision 267): every hero
    /// loses, where the original spares a computer player's (decision 16: no human or computer
    /// player here). The day's points stay (the army's speed counts at the next day's start).
    // rule: ADV-TEL-006
    fn whirlpool_loss<S: Sink>(
        &mut self,
        core: &Ruleset,
        (player, hero): (PlayerId, HeroId),
        divisor: u32,
        sink: &mut S,
    ) {
        let index = self.heroes.partition_point(|h| h.id < hero);
        let army = &mut self.heroes[index].army;
        let lone = army.len() == 1;
        let value = |stack: &ArmyStack| {
            let fight = core.creature(stack.creature).map_or(0, |c| c.fight_value);
            (u64::from(fight) * u64::from(stack.count), stack.slot)
        };
        let lost = (army.iter_mut().min_by_key(|stack| value(stack))).map(|stack| {
            let kept = stack.count / divisor;
            let kept = if lone { kept.max(1) } else { kept };
            let lost = (stack.slot, stack.creature, stack.count - kept);
            stack.count = kept;
            lost
        });
        army.retain(|stack| stack.count > 0);
        lost.into_iter().for_each(|(slot, creature, count)| {
            sink.emit(AdventureEvent {
                kind: AdventureEventKind::WhirlpoolLoss {
                    hero,
                    slot,
                    creature,
                    count,
                },
                seen_by: Viewers::NONE.with(player),
            });
        });
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
    ) -> AdventureOutcome {
        match self.map.channels().gate_pair(visit.tile) {
            Some(exit) => self.jump(rules, (player, hero), (visit.tile, exit), sink),
            None => {
                self.leads_nowhere((player, hero), visit, sink);
                AdventureOutcome::default()
            }
        }
    }

    /// The exit of `player`'s jump among `listed`, a channel's tiles in the walk's order, each
    /// with whether the visit excludes it (ADV-TEL-003; `teleport_exit_draw` 0x4CDB80, survey
    /// §4.1, read whole; D-53, D-57 as the original): over the list, a tile is a **candidate** when
    /// a hero of another team stands on it, whatever lies there (a team is its player until E10's
    /// allies), or when no hero and no empty boat stand on it and the visit does not exclude it (a
    /// two-way monolith excludes its own tile, a Whirlpool its own Whirlpool's; the mover stands
    /// on the entrance anyway, a hero of its team). *Agent's call* (design note §7): a tile with
    /// an empty boat is no candidate. None: no exit. **One: no draw** (the original's
    /// `random_between` returns equal bounds at once). Else one uniform draw of the candidate's
    /// place among them from the visitor's stream, which no profile can reproduce (the original
    /// draws from the C runtime's one stream; the spread uniform as read, batch E11's E11-15 and
    /// E11-17).
    // rule: ADV-TEL-003
    fn draw_exit(
        &self,
        player: PlayerId,
        listed: impl Iterator<Item = (Pos, bool)> + Clone,
        draws: &mut Rng,
    ) -> Option<Pos> {
        let candidate =
            |&(tile, excluded): &(Pos, bool)| match self.heroes.iter().find(|h| h.pos == tile) {
                Some(other) => other.owner != player,
                None => !excluded && !self.is_empty_boat(tile),
            };
        let count = u32::try_from(listed.clone().filter(candidate).count()).unwrap_or(u32::MAX);
        let place = match count {
            0 => return None,
            1 => 0,
            count => usize::try_from(draws.below(count)).unwrap_or(usize::MAX),
        };
        listed.filter(candidate).nth(place).map(|(tile, _)| tile)
    }

    /// The jump of `player`'s hero `hero` from `entrance` to `exit`: **the arrival** when no hero
    /// stands on the exit ([`Self::arrival`]). Else the tiles within 1 of the exit are explored
    /// by the mover's owner, with no event (`reveal_disk` with radius 1 before `visit_hero`,
    /// survey §4.2, §4.3; batch E11's E11-7), the mover stays on the entrance and visits the hero
    /// on the exit from there (E08 T105b1, increment 11's design note §12): one of its team (only
    /// a gate's pair can hold one: ADV-TEL-003's candidates leave them out) is **met**
    /// (GAM-MET-001 amended: the outcome's meeting, the two heroes no neighbours; batch E11's
    /// E11-6, the mover staying); an enemy is **fought** (GAM-BAT-004 amended: the outcome's
    /// battle at the exit, the defender's tile, with the attacker's ground under the entrance),
    /// the jump held after it (the outcome's `jump_after`, given by [`Self::arrive`] after the
    /// winner's levels). A Whirlpool's exit under an enemy hero is fought the same way, on the two
    /// boats (A-37 fixed by the battle, decision 265, `ADV-QUIRK-39`: the original moves the
    /// mover onto that hero's tile, two heroes on one tile, batch E11's E11-16).
    // rule: GAM-BAT-004, GAM-MET-001
    fn jump<S: Sink>(
        &mut self,
        rules: Rules<'_>,
        (player, hero): (PlayerId, HeroId),
        (entrance, exit): (Pos, Pos),
        sink: &mut S,
    ) -> AdventureOutcome {
        let Some(visited) = self.heroes.iter().position(|h| h.pos == exit) else {
            self.arrival(rules, (player, hero), (entrance, exit), sink);
            return AdventureOutcome::default();
        };
        reveal(&mut self.explored[usize::from(player.raw())], exit, 1);
        let other = &self.heroes[visited];
        if other.owner == player {
            let meeting = Meeting {
                visitor: hero,
                visited: other.id,
            };
            return AdventureOutcome {
                meeting: Some(meeting),
                ..AdventureOutcome::default()
            };
        }
        let mover = self.heroes.partition_point(|h| h.id < hero);
        AdventureOutcome {
            battle: Some(self.battle_start(rules.1, mover, visited)),
            jump_after: Some((hero, exit)),
            ..AdventureOutcome::default()
        }
    }

    /// **The held jump** of `player`'s hero `hero` to `exit` (GAM-DEC-007 amended; E08 T105b1,
    /// increment 11's design note §13; [`AdventureOutcome::jump_after`]): once the battle at the
    /// exit and the winner's levels are over, the hero arrives on `exit` from the entrance it
    /// still stands on ([`Self::arrival`], with its event). Nothing for a hero the battle took
    /// off the map or another player's, nor when the exit holds a hero or an empty boat
    /// (unreachable after a won battle, whose defender left the map with its boat, ADV-SEA-006,
    /// as the original's test that the exit is a gate again, 0x4AA4A6): a rest never puts two
    /// heroes, or a hero and an empty boat, on one tile.
    // rule: GAM-DEC-007, ADV-TEL-007
    pub fn arrive<S: Sink>(
        &mut self,
        rules: (&Ruleset, &AdventureRuleset, &[Arc<HeroDetails>]),
        (player, hero): (PlayerId, HeroId),
        exit: Pos,
        sink: &mut S,
    ) {
        let entrance = self.hero(hero).filter(|h| h.owner == player).map(|h| h.pos);
        let taken = self.heroes.iter().any(|h| h.pos == exit) || self.is_empty_boat(exit);
        if let Some(entrance) = entrance.filter(|_| !taken) {
            self.arrival(rules, (player, hero), (entrance, exit), sink);
        }
    }

    /// **The arrival** of `player`'s hero `hero` from `entrance` on the free `exit` (ADV-TEL-007;
    /// `hero_relocate` 0x41D930, survey §5; batch E11's E11-2, E11-3): its tile becomes the exit,
    /// its points and the day's maximum kept; the tiles within its scouting radius of the exit
    /// explored by its owner (ADV-FOG-002's disk, as a step's); **no guard test and no visit**
    /// there (a two-way exit is not entered again); the move ends. A hero in a boat (a
    /// Whirlpool's) arrives in it: the boat's record keeps the tile it was boarded on (ADR-0016
    /// point 1), the hero's tile is the boat's. Told as [`AdventureEventKind::HeroTeleported`]
    /// (ADV-FOG-004).
    // rule: ADV-TEL-007, ADV-FOG-004
    fn arrival<S: Sink>(
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
