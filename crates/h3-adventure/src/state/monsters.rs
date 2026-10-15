//! The wandering monsters' visits (E08 T20b1): the monster's answer to a hero (ADV-MON-007, its
//! rule in `answer.rs`) turned into an offer or a fight, the offers' answers (ADV-MON-008: the
//! group joining, the monster fleeing) and the guard's visit after a step onto a tile it guards
//! (ADV-MON-006).

use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;

use h3_content::{CreatureTypeId, ResourceId, Ruleset};
use h3_contracts::{ArmyStack, HeroId, PlayerId};
use h3_heroes::HeroDetails;

use super::AdventureState;
use crate::answer::{Answer, Temper, Wanderer};
use crate::command::{
    AdventureEvent, AdventureEventKind, AdventureOutcome, BattleStart, Combatant, Defender, Offer,
    OfferKind, Sink, Viewers,
};
use crate::ruleset::{AdventureRuleset, ObjectKind, ObjectTypeId};
use crate::setup::Pos;

/// Whether `army` has room for a group of `creature`: a stack of it, or one of `slots` slots
/// free (the original's `army_add_stack`, 0x44ACE0).
fn has_room(army: &[ArmyStack], slots: u8, creature: CreatureTypeId) -> bool {
    army.iter().any(|stack| stack.creature == creature) || army.len() < usize::from(slots)
}

/// Adds `count` of `creature` to `army` as the original's `army_add_stack` (0x44ACE0): to its
/// first stack of the creature, else as a stack in its first free slot of `slots`. Returns
/// whether they were added; counts saturate.
pub(super) fn add_stack(
    army: &mut Vec<ArmyStack>,
    slots: u8,
    creature: CreatureTypeId,
    count: u32,
) -> bool {
    if let Some(stack) = army.iter_mut().find(|s| s.creature == creature) {
        stack.count = stack.count.saturating_add(count);
        return true;
    }
    let free = (0..slots).find(|&slot| army.iter().all(|s| s.slot != slot));
    free.is_some_and(|slot| {
        let at = army
            .iter()
            .position(|s| s.slot > slot)
            .unwrap_or(army.len());
        army.insert(
            at,
            ArmyStack {
                slot,
                creature,
                count,
            },
        );
        true
    })
}

/// A group's price to join for gold (ADV-MON-008; the original's `monster_join_for_gold`,
/// 0x4A7250): its creature's cost in the monsters' price resource times its count, saturating;
/// empty when the creature costs none of it.
fn join_price(
    core: &Ruleset,
    part: &AdventureRuleset,
    monster: Wanderer,
) -> Vec<(ResourceId, u32)> {
    part.params()
        .monsters()
        .map(|params| params.price_resource())
        .and_then(|resource| {
            let cost = core.creature(monster.creature).and_then(|info| {
                info.cost
                    .iter()
                    .find(|&&(r, _)| r == resource)
                    .map(|&(_, amount)| amount)
            })?;
            let price = u64::from(cost) * u64::from(monster.count);
            Some((resource, u32::try_from(price).unwrap_or(u32::MAX)))
        })
        .filter(|&(_, amount)| amount > 0)
        .into_iter()
        .collect()
}

impl AdventureState {
    /// The visit of the monster at object index `index`, of type `object_type`, by the hero
    /// `hero` (ADV-MON-007, E08 T20b1): the monster's answer to the hero as the store has
    /// its details ([`Temper`]: the hero's power, sympathy and Diplomacy against the monster's
    /// strength and aggression, on the game's difficulty). An offer to join is made only while
    /// the hero's army has room for the group (a stack of its creature or a free slot); without
    /// room it counts as declined (E08 decision 22: the original's army dialog is `ADV-STUB-2`).
    /// An offer names the monster by type and anchor; the game opens it as the player's decision
    /// (a price to join only when the player can pay); a fight is a battle it opens at once
    /// ([`Self::fight`], E08 T20b2).
    // rule: ADV-MON-007
    pub(super) fn visit_monster(
        &self,
        (core, part, heroes): (&Ruleset, &AdventureRuleset, &[Arc<HeroDetails>]),
        hero: HeroId,
        (index, object_type): (usize, ObjectTypeId),
    ) -> AdventureOutcome {
        self.answer_to((core, part, heroes), hero, index, Temper::first)
            .map(|(answer, monster)| {
                self.carry_out((core, part), hero, (index, object_type), (answer, monster))
            })
            .unwrap_or_default()
    }

    /// The answer of the monster at object index `index` to `hero` by `step` (its first answer
    /// or the one after a declined join), an offer to join counting as declined when the army has
    /// no room for the group (`ADV-STUB-2`), with the monster as the answer read it; none when
    /// the hero or the monster is not on the map.
    // stub: ADV-STUB-2
    fn answer_to(
        &self,
        (core, part, heroes): (&Ruleset, &AdventureRuleset, &[Arc<HeroDetails>]),
        hero: HeroId,
        index: usize,
        step: fn(Temper) -> Answer,
    ) -> Option<(Answer, Wanderer)> {
        let visitor = self.hero(hero)?;
        let monster = self.wanderer(index)?;
        let details = heroes
            .get(usize::from(hero.raw()))
            .map(|details| &**details);
        let temper = Temper::new(core, (&visitor.army, details), monster, self.difficulty);
        let slots = part.params().army_slots();
        let answer = match step(temper) {
            Answer::Join | Answer::JoinForGold
                if !has_room(&visitor.army, slots, monster.creature) =>
            {
                temper.after_join()
            }
            answer => answer,
        };
        Some((answer, monster))
    }

    /// The object at `index`, one still on the map (the callers find it among those), as its
    /// answer reads it if it is a monster: its creature, its count as it stands
    /// ([`Self::monster_count`]), its aggression and whether it never flees.
    fn wanderer(&self, index: usize) -> Option<Wanderer> {
        let monster = self.map.objects()[index].monster.as_ref()?;
        Some(Wanderer {
            creature: monster.creature,
            count: self.count_of(index, monster),
            aggression: monster.aggression,
            never_flees: monster.never_flees,
        })
    }

    /// What an answer does (ADV-MON-007, ADV-MON-008): a fight ([`Self::fight`]), or the offer to
    /// join, to join for its price, or to flee, named by the monster's type and anchor.
    fn carry_out(
        &self,
        (core, part): (&Ruleset, &AdventureRuleset),
        hero: HeroId,
        (index, object_type): (usize, ObjectTypeId),
        (answer, monster): (Answer, Wanderer),
    ) -> AdventureOutcome {
        let anchor = self.map.objects()[index].anchor;
        let offer = |kind| AdventureOutcome {
            offer: Some(Offer {
                hero,
                object_type,
                anchor,
                kind,
            }),
            ..AdventureOutcome::default()
        };
        match answer {
            Answer::Fight => self.fight(part, hero, index, monster),
            Answer::Join => offer(OfferKind::Join),
            Answer::JoinForGold => offer(OfferKind::JoinForGold {
                price: join_price(core, part, monster),
            }),
            Answer::Pursue => offer(OfferKind::Pursue),
        }
    }

    /// `monster`, the object at index `index`, fights the hero `hero` (ADV-MON-009, E08 T20b2;
    /// the original's `monster_fight`, 0x4A6C60): a battle the game opens at once, the hero's side
    /// attacking as it stands, the monster's defending unsplit and unowned (the game splits it,
    /// [`crate::split`]), fought at the monster's tile ([`Self::battle_ground`]; a monster is
    /// blocked and visited on its anchor). None when the hero is not on the map.
    // rule: ADV-MON-009
    fn fight(
        &self,
        part: &AdventureRuleset,
        hero: HeroId,
        index: usize,
        monster: Wanderer,
    ) -> AdventureOutcome {
        let tile = self.map.objects()[index].anchor;
        let (terrain, battlefield) = self.battle_ground(part, tile);
        let battle = self.hero(hero).map(|attacker| BattleStart {
            attacker: Combatant {
                hero: attacker.id,
                owner: attacker.owner,
                army: attacker.army.clone(),
                formation: attacker.formation,
            },
            defender: Defender::Monster {
                creature: monster.creature,
                count: monster.count,
            },
            tile,
            terrain,
            battlefield,
        });
        AdventureOutcome {
            battle,
            ..AdventureOutcome::default()
        }
    }

    /// Applies the end of a battle against the monster visited at `tile` (ADV-MON-010, E08
    /// T20b2; the original's `monster_fight`, 0x4A6C60, after `monster_battle`'s count of the
    /// survivors, 0x4AC905): the monster's `survivors`, every stack it has left, count as its own
    /// creature (A-6 fixed, E08 decision 23: the original counts its own type only, so an
    /// upgraded stack's survivors vanished). None left: the monster leaves the map, told to the
    /// players that see it there, and when the visitor's side `won` its treasure's resources are
    /// the returned gains (a draw gives none); else it stays with that count, which its next
    /// answer, battle and the word a viewer sees read ([`Self::monster_count`]), the word's change
    /// told to the players that see it (E08 T20b3b1). Infallible: nothing when no monster is
    /// visited at `tile`.
    // rule: ADV-MON-010
    pub fn end_monster_battle<S: Sink>(
        &mut self,
        (core, part): (&Ruleset, &AdventureRuleset),
        tile: Pos,
        (survivors, won): (&[ArmyStack], bool),
        sink: &mut S,
    ) -> Vec<(ResourceId, u32)> {
        let found = self
            .tile_object(part, tile)
            .filter(|(_, _, ty)| matches!(ty.kind, ObjectKind::Monster { .. }))
            .and_then(|(index, id, _)| {
                let monster = self.map.objects()[index].monster.as_ref();
                monster.map(|m| (index, id, m.creature, m.treasure.clone()))
            });
        let Some((index, object_type, creature, treasure)) = found else {
            return Vec::new();
        };
        let left = survivors
            .iter()
            .fold(0u32, |sum, stack| sum.saturating_add(stack.count));
        let had = self.monster_count(index).unwrap_or_default();
        let mut armies = self.army_left.as_deref().cloned().unwrap_or_default();
        if left == 0 {
            armies.remove(&index);
        } else {
            armies.insert(
                index,
                vec![ArmyStack {
                    slot: 0,
                    creature,
                    count: left,
                }],
            );
        }
        self.army_left = (!armies.is_empty()).then(|| Arc::new(armies));
        if left > 0 {
            let word = |count| part.params().monsters().map_or(0, |w| w.count_word(count));
            if word(left) != word(had) {
                let object = &self.map.objects()[index];
                sink.emit(AdventureEvent {
                    kind: AdventureEventKind::MonsterChanged {
                        object_type,
                        anchor: object.anchor,
                        count_word: word(left),
                    },
                    seen_by: self.object_viewers(core, object),
                });
            }
            return Vec::new();
        }
        self.take_object(core, (index, object_type), sink);
        if won { treasure } else { Vec::new() }
    }

    /// Declines `offer`, made by a visit (ADR-0017 points 6 and 10; E08 T20b1): a
    /// wandering monster's answer goes on. A declined offer to join, for free or for its price
    /// (or one the game did not open because the player could not pay), is followed by the
    /// monster's answer after a declined join (ADV-MON-007: a fight when its aggression equals
    /// the charisma or it never flees, else its offer to flee); a declined offer to flee lets the
    /// monster leave the map without its treasure (ADV-MON-008), told to the players that see it
    /// there. A Cartographer's or a border guard's decline changes nothing, nor that of an offer
    /// whose monster is no longer on the map. Infallible.
    // rule: ADV-MON-007, ADV-MON-008
    pub fn decline_offer<S: Sink>(
        &mut self,
        (core, part, heroes): (&Ruleset, &AdventureRuleset, &[Arc<HeroDetails>]),
        offer: &Offer,
        sink: &mut S,
    ) -> AdventureOutcome {
        let object = self.offer_object(offer);
        match (&offer.kind, object) {
            (OfferKind::Join | OfferKind::JoinForGold { .. }, Some(index)) => self
                .answer_to((core, part, heroes), offer.hero, index, Temper::after_join)
                .map(|found| {
                    let monster = (index, offer.object_type);
                    self.carry_out((core, part), offer.hero, monster, found)
                })
                .unwrap_or_default(),
            (OfferKind::Pursue, Some(index)) => {
                self.take_object(core, (index, offer.object_type), sink);
                AdventureOutcome::default()
            }
            _ => AdventureOutcome::default(),
        }
    }

    /// An accepted offer to join (ADV-MON-008, E08 T20b1; the original's `monster_join_free`
    /// 0x4A7000 and `monster_join_for_gold` 0x4A7250, the price paid by the game): the whole
    /// group joins the hero's army as one stack ([`add_stack`]), told to the player; its
    /// treasure's resources are the player's gains; the monster leaves the map, told to the
    /// players that see it there. Nothing happens when the group finds no room after all (the
    /// offer is made only while it does) or the monster or the hero is gone.
    // rule: ADV-MON-008
    pub(super) fn join<S: Sink>(
        &mut self,
        (core, part): (&Ruleset, &AdventureRuleset),
        player: PlayerId,
        offer: &Offer,
        sink: &mut S,
    ) -> AdventureOutcome {
        let Some((index, monster)) = self
            .offer_object(offer)
            .and_then(|index| self.wanderer(index).map(|monster| (index, monster)))
        else {
            return AdventureOutcome::default();
        };
        let slots = part.params().army_slots();
        let joined = self
            .heroes
            .iter_mut()
            .find(|h| h.id == offer.hero)
            .is_some_and(|h| add_stack(&mut h.army, slots, monster.creature, monster.count));
        if !joined {
            return AdventureOutcome::default();
        }
        sink.emit(AdventureEvent {
            kind: AdventureEventKind::CreaturesJoined {
                hero: offer.hero,
                creature: monster.creature,
                count: monster.count,
            },
            seen_by: Viewers::NONE.with(player),
        });
        let gains = self.map.objects()[index]
            .monster
            .as_ref()
            .map(|m| m.treasure.clone())
            .unwrap_or_default();
        self.take_object(core, (index, offer.object_type), sink);
        AdventureOutcome {
            gains,
            ..AdventureOutcome::default()
        }
    }

    /// An accepted offer to flee (ADV-MON-008): the hero pursues the monster and fights it
    /// ([`Self::fight`]; nothing when it is gone).
    // rule: ADV-MON-008
    pub(super) fn pursue(&self, part: &AdventureRuleset, offer: &Offer) -> AdventureOutcome {
        self.offer_object(offer)
            .and_then(|index| self.wanderer(index).map(|monster| (index, monster)))
            .map(|(index, monster)| self.fight(part, offer.hero, index, monster))
            .unwrap_or_default()
    }

    /// The guard's visit that waited for an offer's answer (ADV-MON-006, E08 T20b1;
    /// [`AdventureOutcome::guard_after`]): the monster that guards the hero `hero`'s tile now
    /// ([`Self::guard_of`]), if any, visits as after the step ([`Self::visit_monster`]).
    /// Infallible: nothing for a hero not on the map or a tile no monster guards any more.
    // rule: ADV-MON-006
    pub fn visit_guard(
        &self,
        rules: (&Ruleset, &AdventureRuleset, &[Arc<HeroDetails>]),
        hero: HeroId,
    ) -> AdventureOutcome {
        self.hero(hero)
            .and_then(|h| self.guard_at(rules.1, h.pos))
            .map(|guard| self.visit_monster(rules, hero, guard))
            .unwrap_or_default()
    }
}
