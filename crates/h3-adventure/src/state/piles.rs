//! The guarded piles' visits (ADV-PIK-004, E08 T21a; survey `E08-battles.md` §5.1, the original's
//! `visit_guarded_resource`, 0x4A4780): a pile whose guards still stand asks its question (a
//! [`OfferKind::Fight`] offer) or fights at once; once they are beaten it is picked up as any pile.

use alloc::sync::Arc;
use alloc::vec::Vec;

use h3_content::Ruleset;
use h3_contracts::{ArmyStack, HeroId};
use h3_heroes::HeroDetails;

use super::AdventureState;
use crate::command::{AdventureOutcome, Defender, Offer, OfferKind, Sink};
use crate::ruleset::{AdventureRuleset, ObjectKind, ObjectTypeId};
use crate::setup::Pos;

impl AdventureState {
    /// The guards of the pile at object index `index` as they stand (ADV-PIK-004, E08 T21a): the
    /// stacks a battle they survived left ([`Self::army_left`]: none after a draw), else the map's;
    /// empty for a pile without guards (and every other object).
    pub fn guards_left(&self, index: usize) -> &[ArmyStack] {
        let guards = self.map.objects()[index].guards.as_ref();
        match (guards, self.army_left(index)) {
            (Some(_), Some(left)) => left,
            (Some(guards), None) => &guards.army,
            (None, _) => &[],
        }
    }

    /// The visit of the pile at object index `index`, of type `object_type`, by the hero `hero`
    /// (ADV-PIK-001, ADV-PIK-003, ADV-PIK-004; the original's `visit_resource_pile`, 0x4A4BE0,
    /// and `visit_guarded_resource`, 0x4A4780): while its guards stand, the question
    /// ([`OfferKind::Fight`], named by type and anchor) when its message asks one, else the
    /// battle at once ([`Self::fight_or_pick`]); without guards standing, the pickup.
    // rule: ADV-PIK-004
    pub(super) fn visit_pile<S: Sink>(
        &mut self,
        core: &Ruleset,
        part: &AdventureRuleset,
        hero: HeroId,
        (index, object_type): (usize, ObjectTypeId),
        sink: &mut S,
    ) -> AdventureOutcome {
        let object = &self.map.objects()[index];
        let asks = object.guards.as_ref().is_some_and(|guards| guards.question);
        if asks && !self.guards_left(index).is_empty() {
            return AdventureOutcome {
                offer: Some(Offer {
                    hero,
                    object_type,
                    anchor: object.anchor,
                    kind: OfferKind::Fight,
                }),
                ..AdventureOutcome::default()
            };
        }
        self.fight_or_pick(core, part, hero, (index, object_type), sink)
    }

    /// The pile at object index `index` fought for or picked up by the hero `hero` (ADV-PIK-004):
    /// while its guards stand, the battle the game opens at once ([`Self::guards_battle`]); else
    /// the pickup ([`Self::pick_up`]: a pile's contents are the gains, an artifact or a scroll the
    /// hero's) and it leaves the map (ADV-OBJ-005). An accepted question comes here too (the
    /// guards a battle of another player's left, or none after a draw, are what it meets).
    // rule: ADV-PIK-001, ADV-PIK-003, ADV-PIK-004, ADV-OBJ-005
    pub(super) fn fight_or_pick<S: Sink>(
        &mut self,
        core: &Ruleset,
        part: &AdventureRuleset,
        hero: HeroId,
        (index, object_type): (usize, ObjectTypeId),
        sink: &mut S,
    ) -> AdventureOutcome {
        if self.guards_left(index).is_empty() {
            return self.pick_up(core, hero, (index, object_type), sink);
        }
        self.guards_battle(part, hero, index)
    }

    /// The battle against the guards of the object at index `index` as they stand (ADV-PIK-004;
    /// since E08 T62b1 Pandora's Box's too, ADV-PIK-013): the hero `hero`'s side attacking as it
    /// stands, the guards' stacks defending unsplit and unowned, fought at the object's tile
    /// ([`Self::pile_tile`], [`Self::battle_ground`]; the original passes the visited tile and
    /// the map's army to the battle, 0x4A4886); none when the hero is not on the map.
    // rule: ADV-PIK-004
    pub(super) fn guards_battle(
        &self,
        part: &AdventureRuleset,
        hero: HeroId,
        index: usize,
    ) -> AdventureOutcome {
        let army = self.guards_left(index).to_vec();
        let battle = self
            .hero(hero)
            .zip(self.pile_tile(index))
            .map(|(attacker, tile)| {
                self.battle_at(part, attacker, Defender::Guards { army }, tile)
            });
        AdventureOutcome {
            battle,
            ..AdventureOutcome::default()
        }
    }

    /// The tile the pile at object index `index` is visited from: its first visitable tile on the
    /// map, in its mask's order (a pile's template has one; Pandora's Box's and the Pyramid's
    /// too).
    pub(super) fn pile_tile(&self, index: usize) -> Option<Pos> {
        let object = &self.map.objects()[index];
        object
            .visitable
            .offsets()
            .find_map(|(column, row)| self.map.offset_pos(object.anchor, column, row))
    }

    /// Applies the end of a battle no player's hero defended, fought by the visitor's hero `hero`
    /// at `tile` (ADR-0017 point 11), by the object visited there: a wandering monster's
    /// ([`Self::end_monster_battle`], ADV-MON-010), a mine's guards' (ADV-MIN-002, E08 T84a: a
    /// neutral one's or its owner's side, [`Self::end_mine_battle`]), a guarded object's
    /// (ADV-PIK-004, E08 T21a; an
    /// artifact's or a scroll's since T62a, ADV-PIK-007, ADV-PIK-008; Pandora's Box's since
    /// T62b1, ADV-PIK-013) or the Pyramid's (ADV-TCH-008, E08 T62b1). After the visitor's side
    /// `won` against an object's guards they are beaten for good, and what the object gives comes
    /// after the winner's levels ([`AdventureOutcome::reward_after`], GAM-DEC-007, E08 T63b:
    /// [`Self::give_won`]); else the guards' `survivors`, by slot, stay as its guards (none after
    /// a draw: the next visit picks it up, or opens the box), unseen by every player (the guards
    /// are hidden). A won Pyramid's loot and spell come after the levels too; after any other end
    /// nothing changes (its guards' losses are not kept). Else, since E08 T62b2, an event's
    /// guards' battle at its tile, where its visitor stands ([`Self::end_event_battle`],
    /// ADV-OBJ-007). Infallible: nothing when none of them is at `tile`.
    // rule: ADV-PIK-004, ADV-MON-010, ADV-PIK-013, ADV-TCH-008, ADV-OBJ-007, GAM-DEC-007
    // rule: ADV-MIN-002
    pub fn end_unowned_battle<S: Sink>(
        &mut self,
        rules: (&Ruleset, &AdventureRuleset, &[Arc<HeroDetails>]),
        (hero, tile): (HeroId, Pos),
        (survivors, won): (&[ArmyStack], bool),
        sink: &mut S,
    ) -> AdventureOutcome {
        let part = rules.1;
        let Some((index, object_type, ty)) = self.tile_object(part, tile) else {
            return self.end_event_battle((hero, tile), (survivors, won));
        };
        let reward = AdventureOutcome {
            reward_after: won.then_some((hero, tile)),
            ..AdventureOutcome::default()
        };
        match ty.kind {
            ObjectKind::Monster { .. } => {
                self.end_monster_battle(rules, (hero, tile), (survivors, won), sink)
            }
            ObjectKind::Mine { .. } | ObjectKind::AbandonedMine { .. } => self.end_mine_battle(
                (rules.0, part),
                (index, object_type),
                (hero, tile),
                (survivors, won),
                sink,
            ),
            ObjectKind::Pyramid(_) => reward,
            _ if self.map.objects()[index].guards.is_some() => {
                let left = if won { Vec::new() } else { survivors.to_vec() };
                self.army_left.set(index, Some(left));
                reward
            }
            _ => self.end_event_battle((hero, tile), (survivors, won)),
        }
    }

    /// What a won battle against the guards of the object at `tile` gives the hero `hero` after
    /// the winner's levels (GAM-DEC-007, E08 T63b; [`AdventureOutcome::reward_after`]): a mine is
    /// flagged for the hero's owner (ADV-MIN-002, E08 T84a: [`Self::flag_won`]); a guarded
    /// pile, artifact or scroll is picked up ([`Self::pick_up`]: a pile's contents are the
    /// outcome's gains, an artifact or a scroll goes to the hero) or a box gives its record
    /// ([`Self::give_box`]), and it leaves the map, told to the players that see it there; a
    /// Pyramid not looted is looted and teaches its spell ([`Self::loot_pyramid`]); an event whose
    /// guards are beaten gives its record ([`Self::give_event`]). Read when it is given: the
    /// levels' skills decide what the hero can receive or learn (the Pyramid's spell, E7-22).
    /// Nothing when none of them is at `tile` any more.
    // rule: GAM-DEC-007, ADV-PIK-004, ADV-PIK-013, ADV-TCH-008, ADV-OBJ-007, ADV-MIN-002
    pub fn give_won<S: Sink>(
        &mut self,
        rules: (&Ruleset, &AdventureRuleset, &[Arc<HeroDetails>]),
        (hero, tile): (HeroId, Pos),
        sink: &mut S,
    ) -> AdventureOutcome {
        let (core, part, _) = rules;
        let beaten = |s: &Self, index: usize| {
            s.map.objects()[index].guards.is_some() && s.guards_left(index).is_empty()
        };
        if let Some(mine) = self.mine_at(part, tile) {
            self.flag_won((core, part), hero, mine, sink);
            return AdventureOutcome::default();
        }
        match self.tile_object(part, tile) {
            Some((index, _, ty)) if matches!(ty.kind, ObjectKind::Pyramid(_)) => {
                if self.is_emptied(index) {
                    AdventureOutcome::default()
                } else {
                    self.loot_pyramid(rules, hero, index)
                }
            }
            Some((index, object_type, ty)) if beaten(self, index) => match ty.kind {
                ObjectKind::PandoraBox => self.give_box(rules, hero, (index, object_type), sink),
                _ => self.pick_up(core, hero, (index, object_type), sink),
            },
            _ => self
                .event_at(tile)
                .filter(|&index| beaten(self, index))
                .map(|index| self.give_event(rules, hero, index, sink))
                .unwrap_or_default(),
        }
    }
}
