//! Pandora's Box, the Pyramid and the reward record (E08 T62b1; design §11, §11.1 and §11.2 of
//! `docs/epics/E08-design.md`, survey `docs/rules/surveys/E08-rewards.md` §7.1, §7.2, §7.4 and
//! §7.5): the box's question, its guards' battle as a guarded pile's and its record; the
//! Pyramid's question at every visit, a looted one's luck, its battle with a second creature and
//! the spell a win gives; the record's adventure half (ADV-RWD-001), which an event gives too
//! (T62b2). The hero's half of the record is `h3_heroes::grant`'s, read against the details of the
//! game's store; the game applies the outcome.

use alloc::sync::Arc;
use alloc::vec::Vec;

use h3_content::Ruleset;
use h3_contracts::{HeroId, PlayerId};
use h3_heroes::{HeroChange, HeroDetails, can_learn_spell, grant};

use super::monsters::add_stack;
use super::rewards::changes;
use super::{AdventureState, Resolved};
use crate::command::{
    AdventureEvent, AdventureEventKind, AdventureOutcome, Defender, HeroGains, NoRoom, Offer,
    OfferKind, Sink, Viewers,
};
use crate::movement::details_of;
use crate::ruleset::{AdventureRuleset, ObjectKind, ObjectTypeId};
use crate::setup::{Holds, RewardRecord};

/// The rules a visit reads: the core, the ruleset part and the game's store of hero details.
type Rules<'a> = (&'a Ruleset, &'a AdventureRuleset, &'a [Arc<HeroDetails>]);

impl AdventureState {
    /// The question an object asks its visitor `hero` first, at every visit (Pandora's Box's,
    /// ADV-PIK-013; the Pyramid's, ADV-TCH-008): an offer of `kind` naming it by type and anchor.
    /// The original asks a human (ADVEVENT 14, 105) and the computer `ai_fight_answer` (E10);
    /// declined, nothing changes.
    // rule: ADV-PIK-013, ADV-TCH-008
    pub(super) fn ask(
        &self,
        hero: HeroId,
        (index, object_type): (usize, ObjectTypeId),
        kind: OfferKind,
    ) -> AdventureOutcome {
        AdventureOutcome {
            offer: Some(Offer {
                hero,
                object_type,
                anchor: self.map.objects()[index].anchor,
                kind,
            }),
            ..AdventureOutcome::default()
        }
    }

    /// An accepted Pandora's Box's question ([`OfferKind::OpenBox`], ADV-PIK-013; the original's
    /// `visit_pandora_box` 0x4A0C50, survey §7.2): while its guards stand, the battle against them
    /// at once, as a guarded pile's ([`Self::guards_battle`]: unsplit, unowned, at the box's tile;
    /// the original announces it, ADVEVENT 16, and asks nothing more); else the box's reward record
    /// ([`Self::give_box`]). Nothing when the box is gone.
    // rule: ADV-PIK-013
    pub(super) fn open_box<S: Sink>(
        &mut self,
        rules: Rules<'_>,
        offer: &Offer,
        sink: &mut S,
    ) -> AdventureOutcome {
        let Some(index) = self.offer_object(offer) else {
            return AdventureOutcome::default();
        };
        if self.guards_left(index).is_empty() {
            self.give_box(rules, offer.hero, (index, offer.object_type), sink)
        } else {
            self.guards_battle(rules.1, offer.hero, index)
        }
    }

    /// Pandora's Box at `index` gives its reward record to `hero` ([`Self::give_record`]) and
    /// leaves the map (ADV-PIK-013, ADV-OBJ-005), told to the players that see it there; after an
    /// accepted question without guards standing, or a won battle against them. The original's
    /// "the box is empty" for a record that gave nothing is its message only: no event.
    // rule: ADV-PIK-013, ADV-OBJ-005
    pub(super) fn give_box<S: Sink>(
        &mut self,
        rules: Rules<'_>,
        hero: HeroId,
        (index, object_type): (usize, ObjectTypeId),
        sink: &mut S,
    ) -> AdventureOutcome {
        let map = Arc::clone(&self.map);
        let mut outcome = AdventureOutcome::default();
        // The setup gives every box its record (`SetupError::Holds`).
        if let Holds::Record(record) = &map.objects()[index].holds {
            outcome = self.give_record(rules, hero, record, sink);
        }
        self.take_object(rules.0, (index, object_type), sink);
        outcome
    }

    /// The adventure's half of a reward record given to the hero `hero` (ADV-RWD-001; the
    /// original's `grant_rewards` 0x49FA90, survey §7.4): its hero half by `h3_heroes::grant`
    /// (HER-CHG-003: the primary skills, the skills, the spell points, the battle bonuses, the
    /// artifacts while the hero can receive them, else lost, the spells by book, Wisdom and
    /// knowledge, each against the details as the earlier parts leave them; the experience
    /// computed with the Learning before the record's skills, D-39) as the outcome's hero gains,
    /// the experience among them for the game to give last; the resources, a positive amount a
    /// gain and a negative one a loss the game takes never below 0; the creatures slot by slot onto
    /// the hero's army as the original's `army_add_stack` (ADV-ARM-007: onto its first stack of
    /// the creature that takes them within a stack's most, else into its first free slot), each
    /// told to the hero's owner (`CreaturesJoined`), the stacks that find no room kept at their
    /// slots as **one** group for the army dialog ([`AdventureOutcome::no_room`], GAM-DEC-004).
    /// The creatures join here, before the game applies the hero's changes; nothing of either
    /// reads the other. A hero without details gets no hero half; one not on the map no creature.
    // rule: ADV-RWD-001, ADV-ARM-007
    pub(super) fn give_record<S: Sink>(
        &mut self,
        (core, part, heroes): Rules<'_>,
        hero: HeroId,
        record: &RewardRecord,
        sink: &mut S,
    ) -> AdventureOutcome {
        let granted = details_of(heroes, hero)
            .map(|details| grant(details, core, &record.hero))
            .unwrap_or_default();
        let mut outcome = AdventureOutcome::default();
        // The setup gives no amount of 0 (`SetupError::Holds`).
        for &(resource, amount) in &record.resources {
            let into = if amount < 0 {
                &mut outcome.losses
            } else {
                &mut outcome.gains
            };
            into.push((resource, amount.unsigned_abs()));
        }
        let room = (part.params().army_slots(), part.params().max_stack_count());
        let mut group = Vec::new();
        if let Some(visitor) = self.heroes.iter_mut().find(|h| h.id == hero) {
            let owner = visitor.owner;
            for stack in &record.creatures {
                if add_stack(&mut visitor.army, room, stack.creature, stack.count) {
                    sink.emit(AdventureEvent {
                        kind: AdventureEventKind::CreaturesJoined {
                            hero,
                            creature: stack.creature,
                            count: stack.count,
                        },
                        seen_by: Viewers::NONE.with(owner),
                    });
                } else {
                    group.push(*stack);
                }
            }
        }
        if !granted.changes.is_empty() || granted.experience > 0 {
            outcome.hero = Some(HeroGains {
                hero,
                changes: granted.changes,
                experience: granted.experience,
            });
        }
        outcome.no_room = (!group.is_empty()).then_some(NoRoom { hero, group });
        outcome
    }

    /// An accepted Pyramid's question ([`OfferKind::SearchPyramid`], ADV-TCH-008; the original's
    /// `visit_pyramid` 0x4A4230, survey §7.5) of `player`'s hero: the player learns whether it is
    /// looted (ADV-VIS-006: the team's mark after the question); a looted one gives the hero its
    /// type's bonus (−2 luck, once until the next battle's end by its mark: HER-ADV-008); else the
    /// battle against its guards at once, its first creature split as a monster's less its
    /// second's stacks (ADV-MON-012, [`Defender::Monsters`]), fought at its tile. Nothing when the
    /// Pyramid is gone or the hero is not on the map.
    // rule: ADV-TCH-008, ADV-VIS-006, ADV-MON-012
    pub(super) fn search_pyramid(
        &mut self,
        part: &AdventureRuleset,
        player: PlayerId,
        offer: &Offer,
    ) -> AdventureOutcome {
        let pyramid = part
            .object_type(offer.object_type)
            .and_then(|ty| match &ty.kind {
                ObjectKind::Pyramid(pyramid) => Some(pyramid),
                _ => None,
            });
        let (Some(index), Some(pyramid)) = (self.offer_object(offer), pyramid) else {
            return AdventureOutcome::default();
        };
        self.learn(player, index);
        if self.is_emptied(index) {
            let looted = pyramid.looted;
            let bonus = HeroChange::Bonus {
                morale: looted.morale,
                luck: looted.luck,
                mark: Some(looted.mark),
            };
            return changes(offer.hero, Vec::from([bonus]));
        }
        let defender = Defender::Monsters {
            split: pyramid.split,
            fixed: pyramid.fixed.clone(),
        };
        let battle = self
            .hero(offer.hero)
            .zip(self.pile_tile(index))
            .map(|(attacker, tile)| self.battle_at(part, attacker, defender, tile));
        AdventureOutcome {
            battle,
            ..AdventureOutcome::default()
        }
    }

    /// A won battle against the Pyramid at `index` by `hero` (ADV-TCH-008; the original's
    /// `visit_pyramid` after its battle, 0x4A435E): it is looted (emptied), and its spell is
    /// learned when the hero can learn it without the known test (`h3_heroes::can_learn_spell`:
    /// a Spell Book, Wisdom for the level; the change only for a spell not learned yet, since
    /// relearning changes nothing), else lost (ADVEVENT 108, 109). A Pyramid without a spell (A-20
    /// fixed) gives nothing. Until T63b before the winner's levels (`ADV-STUB-10`).
    // rule: ADV-TCH-008
    pub(super) fn loot_pyramid(
        &mut self,
        (core, _, heroes): Rules<'_>,
        hero: HeroId,
        index: usize,
    ) -> AdventureOutcome {
        self.visits_mut().empty(index);
        let spell = match self.map.objects()[index].resolved {
            Resolved::Spell(spell) => Some(spell),
            _ => None,
        };
        let details = details_of(heroes, hero);
        let taught = spell.filter(|&spell| {
            details.is_some_and(|d| {
                can_learn_spell(d, core, spell, false) && !d.spells.contains(&spell)
            })
        });
        changes(
            hero,
            taught.map(HeroChange::LearnSpell).into_iter().collect(),
        )
    }
}
