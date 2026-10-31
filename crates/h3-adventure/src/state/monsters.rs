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
    AdventureEvent, AdventureEventKind, AdventureOutcome, Defender, NoRoom, Offer, OfferKind, Sink,
    Viewers,
};
use crate::ruleset::{AdventureRuleset, ObjectKind, ObjectTypeId};
use crate::setup::Pos;

/// The room `army` has for a group of `count` `creature`s, as the original's `army_add_stack`
/// (0x44ACE0) finds it, with a stack's most creatures (ADV-ARM-007, E08 decision 115): its first
/// stack of the creature that takes the group whole within `most` (the original's first stack,
/// which takes any), else its first free slot of `slots` if the group fits a stack, else none.
// rule: ADV-ARM-007
fn room(
    army: &[ArmyStack],
    (slots, most): (u8, u32),
    creature: CreatureTypeId,
    count: u32,
) -> Option<Room> {
    let fits = |s: &ArmyStack| s.creature == creature && count <= most.saturating_sub(s.count);
    if let Some(index) = army.iter().position(fits) {
        return Some(Room::Stack(index));
    }
    (0..slots)
        .find(|&slot| count <= most && army.iter().all(|s| s.slot != slot))
        .map(Room::Slot)
}

/// Where a group goes in an army ([`room`]): onto the stack at this index, or into this free slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Room {
    Stack(usize),
    Slot(u8),
}

/// Adds `count` of `creature` to `army` as the original's `army_add_stack` (0x44ACE0), where
/// [`room`] finds room (`rules`: the army's slots and a stack's most creatures, ADV-ARM-007):
/// to its first stack of the creature that takes them within the most, else as a stack in its
/// first free slot. Returns whether they were added.
// rule: ADV-ARM-007
pub(super) fn add_stack(
    army: &mut Vec<ArmyStack>,
    rules: (u8, u32),
    creature: CreatureTypeId,
    count: u32,
) -> bool {
    match room(army, rules, creature, count) {
        None => false,
        Some(Room::Stack(index)) => {
            army[index].count += count;
            true
        }
        Some(Room::Slot(slot)) => {
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
        }
    }
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
    /// strength and aggression, on the game's difficulty). An offer to join is made whether or
    /// not the hero's army has room for the group: accepted without room, the game opens the
    /// army dialog (GAM-DEC-004, E08 T51b; [`AdventureOutcome::no_room`]).
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
    /// or the one after a declined join), with the monster as the answer read it; none when the
    /// hero or the monster is not on the map. An offer to join is made whatever room the army has
    /// (E08 T51b: the original's army dialog, GAM-DEC-004, follows an accepted join without room).
    fn answer_to(
        &self,
        (core, _, heroes): (&Ruleset, &AdventureRuleset, &[Arc<HeroDetails>]),
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
        Some((step(temper), monster))
    }

    /// The object at `index`, one still on the map (the callers find it among those), as its
    /// answer reads it if it is a monster: its creature, its count as it stands
    /// ([`Self::monster_count`]), its aggression and whether it never flees.
    fn wanderer(&self, index: usize) -> Option<Wanderer> {
        let monster = self.object_at(index).monster.as_ref()?;
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
        let anchor = self.object_at(index).anchor;
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
        let tile = self.object_at(index).anchor;
        let battle = self.hero(hero).map(|attacker| {
            self.battle_at(
                part,
                attacker,
                Defender::Monster {
                    creature: monster.creature,
                    count: monster.count,
                },
                tile,
            )
        });
        AdventureOutcome {
            battle,
            ..AdventureOutcome::default()
        }
    }

    /// Applies the end of a battle of the hero `hero` against the monster visited at `tile`
    /// (ADV-MON-010, E08 T20b2; the original's `monster_fight`, 0x4A6C60, after
    /// `monster_battle`'s count of the survivors, 0x4AC905): the monster's `survivors`, every
    /// stack it has left, count as its own creature (A-6 fixed, E08 decision 23: the original
    /// counts its own type only, so an upgraded stack's survivors vanished). None left: the
    /// monster leaves the map, told to the players that see it there, and when the visitor's side
    /// `won` its treasure is the outcome's (a draw gives none): its artifact to the hero first
    /// when it can receive it (ADV-MON-011, E08 T62a: [`Self::treasure_artifact`]), then its
    /// resources, the gains; else it stays with that count, which its next
    /// answer, battle and the word a viewer sees read ([`Self::monster_count`]), the word's change
    /// told to the players that see it (E08 T20b3b1). Infallible: nothing when no monster is
    /// visited at `tile`.
    // rule: ADV-MON-010, ADV-MON-011
    pub fn end_monster_battle<S: Sink>(
        &mut self,
        (core, part, heroes): (&Ruleset, &AdventureRuleset, &[Arc<HeroDetails>]),
        (hero, tile): (HeroId, Pos),
        (survivors, won): (&[ArmyStack], bool),
        sink: &mut S,
    ) -> AdventureOutcome {
        let found = self
            .tile_object(part, tile)
            .filter(|(_, _, ty)| matches!(ty.kind, ObjectKind::Monster { .. }))
            .and_then(|(index, id, _)| {
                let monster = self.object_at(index).monster.as_ref();
                monster.map(|m| (index, id, m.creature, (m.treasure.clone(), m.artifact)))
            });
        let Some((index, object_type, creature, (treasure, artifact))) = found else {
            return AdventureOutcome::default();
        };
        let left = survivors
            .iter()
            .fold(0u32, |sum, stack| sum.saturating_add(stack.count));
        let had = self.monster_count(index).unwrap_or_default();
        self.army_left.set(
            index,
            (left > 0).then(|| {
                vec![ArmyStack {
                    slot: 0,
                    creature,
                    count: left,
                }]
            }),
        );
        if left > 0 {
            let word = |count| part.params().monsters().map_or(0, |w| w.count_word(count));
            if word(left) != word(had) {
                let object = self.object_at(index);
                sink.emit(AdventureEvent {
                    kind: AdventureEventKind::MonsterChanged {
                        object_type,
                        anchor: object.anchor,
                        count_word: word(left),
                    },
                    seen_by: self.object_viewers(core, object),
                });
            }
            return AdventureOutcome::default();
        }
        self.take_object(core, (index, object_type), sink);
        if !won {
            return AdventureOutcome::default();
        }
        AdventureOutcome {
            gains: treasure,
            hero: Self::treasure_artifact(core, heroes, hero, artifact),
            ..AdventureOutcome::default()
        }
    }

    /// Declines `offer`, made by a visit (ADR-0017 points 6 and 10; E08 T20b1): a
    /// wandering monster's answer goes on. A declined offer to join, for free or for its price
    /// (or one the game did not open because the player could not pay), is followed by the
    /// monster's answer after a declined join (ADV-MON-007: a fight when its aggression equals
    /// the charisma or it never flees, else its offer to flee); a declined offer to flee lets the
    /// monster leave the map without its treasure (ADV-MON-008), told to the players that see it
    /// there. A Cartographer's, a border guard's or a guarded pile's (ADV-PIK-004, E08 T21a: the
    /// hero stays) decline changes nothing, nor that of an offer whose monster is no longer on the
    /// map. Infallible.
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
    /// treasure's artifact goes to the hero when it can receive it (ADV-MON-011, E08 T62a:
    /// [`Self::treasure_artifact`]) and its resources are the player's gains; the monster leaves
    /// the map, told to the
    /// players that see it there. A group that finds no room ([`room`]) joins nothing and is
    /// reported as [`AdventureOutcome::no_room`] (E08 T50), the rest as for a join: the game opens
    /// the army dialog with it (GAM-DEC-004, E08 T51b), as the original's after its yes. Nothing
    /// happens when the monster or the hero is gone.
    // rule: ADV-MON-008, ADV-MON-011, ADV-ARM-007
    pub(super) fn join<S: Sink>(
        &mut self,
        (core, part, heroes): (&Ruleset, &AdventureRuleset, &[Arc<HeroDetails>]),
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
        let rules = (part.params().army_slots(), part.params().max_stack_count());
        let Some(visitor) = self.heroes.iter_mut().find(|h| h.id == offer.hero) else {
            return AdventureOutcome::default();
        };
        let joined = add_stack(&mut visitor.army, rules, monster.creature, monster.count);
        if joined {
            sink.emit(AdventureEvent {
                kind: AdventureEventKind::CreaturesJoined {
                    hero: offer.hero,
                    creature: monster.creature,
                    count: monster.count,
                },
                seen_by: Viewers::NONE.with(player),
            });
        }
        let (gains, artifact) = self
            .object_at(index)
            .monster
            .as_ref()
            .map(|m| (m.treasure.clone(), m.artifact))
            .unwrap_or_default();
        let artifact = Self::treasure_artifact(core, heroes, offer.hero, artifact);
        self.take_object(core, (index, offer.object_type), sink);
        AdventureOutcome {
            gains,
            hero: artifact,
            no_room: (!joined).then(|| NoRoom {
                hero: offer.hero,
                group: vec![ArmyStack {
                    slot: 0,
                    creature: monster.creature,
                    count: monster.count,
                }],
            }),
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
