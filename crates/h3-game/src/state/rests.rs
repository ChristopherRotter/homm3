//! The rests (GAM-DEC-007, E08 T63b; increment 7's design note §14 in `E08-log.md`, survey
//! `docs/rules/surveys/E08-rewards.md` §8.1): what a visit still owes its hero once a decision of
//! the visit closes or a battle's levels are given. The original's visit runs its dialogs, its
//! battle and the levels the battle gave inside the visit and goes on after them; a player of the
//! engine has at most one open decision (ADR-0017 point 6), so the game holds what follows beside
//! the decisions and runs it once the owner has none open.

use alloc::vec::Vec;

use h3_adventure::{HeroGains, Pos};
use h3_contracts::{HeroId, PlayerId};
use h3_kernel::{CanonicalHash, StateHasher};

use super::GameState;
use crate::GameRuleset;
use crate::command::{AdventureSink, Sink};

/// What a visit still owes its hero (GAM-DEC-007).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Rest {
    /// The visit of the entered object at the hero's tile, after the visit of the monster guarding
    /// the tile with everything it opened (its offer, its battle and the levels it gave), for a
    /// hero still on the map (ADV-MON-006, E08 T103d, decision 279: the original's step tests the
    /// guard and its walk visits the object after it; `AdventureState::visit_held`). The hero's
    /// tile is the one it entered: its owner has an open decision until the rest runs. From E08
    /// T63b to T103d the rest was the guard's visit after the object's (`Guard`), hashed the same.
    Visit,
    /// A reward record's experience (ADV-RWD-001, HER-CHG-003: computed with the record), given
    /// with its levels once the record's army dialog closes (the original's order, D-39).
    Experience(u32),
    /// What a won battle against the guards of the object at this tile gives, after the winner's
    /// levels (the original's visit goes on after 0x4DA720): a pickup, a box's or an event's
    /// record, the Pyramid's loot and spell, a mine's flag, a dwelling's visit; since E09 T8b1 the
    /// capture of a town whose siege was won at its entrance, against its garrison or its visitor
    /// (TWN-CAP-003; `AdventureState::give_won`).
    Won(Pos),
    /// The event at this tile, after the visit of the monster guarding it, for a hero still on
    /// the tile (ADV-OBJ-007, `ADV-QUIRK-29`: `AdventureState::fire_held_event`).
    Event(Pos),
}

/// A rest held for `owner`'s `hero`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Held {
    /// The player whose decisions it waits for.
    pub(super) owner: PlayerId,
    /// The hero the visit is of.
    pub(super) hero: HeroId,
    /// What is owed.
    pub(super) rest: Rest,
}

impl CanonicalHash for Held {
    /// The owner, the hero, then the rest's kind: 0 an entered object's visit after its guard's
    /// (before E08 T103d the guard's visit after the object's), 1 an experience (its amount), 2 a
    /// won battle's reward (its tile), 3 an event (its tile).
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_u8(self.owner.raw());
        h.write_u16(self.hero.raw());
        match self.rest {
            Rest::Visit => h.write_u8(0),
            Rest::Experience(experience) => {
                h.write_u8(1);
                h.write_u32(experience);
            }
            Rest::Won(tile) => {
                h.write_u8(2);
                tile.hash_into(h);
            }
            Rest::Event(tile) => {
                h.write_u8(3);
                tile.hash_into(h);
            }
        }
    }
}

impl GameState {
    /// Holds `rest` of `owner`'s `hero` until the owner has no open decision.
    pub(super) fn hold(&mut self, owner: PlayerId, hero: HeroId, rest: Rest) {
        self.rests.push(Held { owner, hero, rest });
    }

    /// Runs the held rests whose owners have no open decision (GAM-DEC-007; after every command,
    /// [`crate::apply`]): the last held first, each taken from the list before it runs, until
    /// every rest left waits for an open decision. A rest that opens a decision holds the ones
    /// below it.
    // rule: GAM-DEC-007
    pub(crate) fn run_rests<S: Sink>(&mut self, rules: &GameRuleset, sink: &mut S) {
        while let Some(at) = self
            .rests
            .iter()
            .rposition(|held| !self.has_decision(held.owner))
        {
            let held = self.rests.remove(at);
            self.run_rest(rules, held, sink);
        }
    }

    /// Runs one rest: an entered object's visit after its guard's (`AdventureState::visit_held`:
    /// nothing for a hero that left the map or a tile no object is visited from any more),
    /// the experience with its levels
    /// ([`Self::give_hero`]), a won battle's reward (`AdventureState::give_won`; nothing when
    /// the object left the map meanwhile; a won siege's capture, TWN-CAP-003), or a held event
    /// (`AdventureState::fire_held_event`: nothing for a hero no longer on its tile), each applied
    /// as a command's outcome.
    // rule: ADV-MON-006, ADV-RWD-001, ADV-PIK-004, ADV-PIK-013, ADV-TCH-008, ADV-OBJ-007
    // rule: TWN-CAP-003
    fn run_rest<S: Sink>(&mut self, rules: &GameRuleset, held: Held, sink: &mut S) {
        let Held { owner, hero, rest } = held;
        match rest {
            Rest::Visit => {
                // The visitor's owner's stream: a jump after its entrance's guard draws its exit
                // from it (ADV-TEL-003, E08 T105a1).
                let (adventure, store, draws) = self.adventure_for(owner);
                let context = (rules.core(), rules.adventure(), store);
                let events = &mut AdventureSink(&mut *sink);
                let outcome = adventure.visit_held(context, (owner, hero), draws, events);
                self.settle_outcome(rules, owner, outcome, sink);
            }
            Rest::Experience(experience) => {
                let gains = HeroGains {
                    hero,
                    changes: Vec::new(),
                    experience,
                };
                self.give_hero(rules, owner, gains, sink);
            }
            Rest::Won(tile) => {
                let (adventure, store) = self.adventure_with_heroes();
                let context = (rules.core(), rules.adventure(), store);
                let outcome = adventure.give_won(context, (hero, tile), &mut AdventureSink(sink));
                self.settle_outcome(rules, owner, outcome, sink);
            }
            Rest::Event(tile) => {
                let (adventure, store) = self.adventure_with_heroes();
                let context = (rules.core(), rules.adventure(), store);
                let events = &mut AdventureSink(&mut *sink);
                let outcome = adventure.fire_held_event(context, (owner, hero), tile, events);
                self.settle_outcome(rules, owner, outcome, sink);
            }
        }
    }
}
