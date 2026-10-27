//! The visits of increment 7's pickups, teachers and finds (E08 T62a; design §11 and §13 of
//! `docs/epics/E08-design.md`, survey `docs/rules/surveys/E08-rewards.md` §2 to §6, §8): what
//! each kind gives its visitor, asks, refuses or turns into something else, in the original's
//! order, as the outcome's hero changes, gains, offers and choices the game applies (ADR-0017
//! point 10). What a visit leaves on the objects goes into the state's visits' block ([`Visits`]):
//! a hero's once-per-hero marks, set only when the gift is given; the visitor's owner's knowledge
//! (ADV-VIS-006), set where the original sets the team's mark; the finds emptied. The hero's
//! side (what it may receive, learn or be taught) is `h3_heroes`'s, read here against the details
//! of the game's store.

use alloc::sync::Arc;
use alloc::vec::Vec;

use h3_content::{ArtifactTypeId, Ruleset};
use h3_contracts::{HeroId, PlayerId};
use h3_heroes::{
    ArtifactInstance, HeroChange, HeroDetails, can_learn_spell, can_receive_artifact, hut_teaches,
    learning_experience, library_admits, next_level_experience, scholar_reward,
};

use super::{AdventureState, MapObject, Resolved, Visits};
use crate::command::{
    AdventureOutcome, Choice, ChoiceKind, HeroGains, Offer, OfferKind, Pick, Sink, UniversityVisit,
};
use crate::movement::details_of;
use crate::ruleset::{
    AdventureRuleset, Chest, Find, Gift, Gives, ObjectKind, ObjectTypeId, Resources, Turn,
};
use crate::setup::{Holds, Pos};

/// The rules a visit reads: the core, the ruleset part and the game's store of hero details.
type Rules<'a> = (&'a Ruleset, &'a AdventureRuleset, &'a [Arc<HeroDetails>]);

/// The artifact an artifact or a Spell Scroll on the map carries (ADV-PIK-006, ADV-PIK-008): the
/// map's placed one or scroll, else a random artifact's draw (ADV-OBJ-006); none for a random
/// artifact whose draw found none, and for every object that is neither (a resource pile).
pub(super) fn carried(object: &MapObject) -> Option<ArtifactInstance> {
    match (&object.holds, &object.resolved) {
        (Holds::Artifact(item), _) => Some(*item),
        (_, Resolved::Artifact(artifact)) => Some(ArtifactInstance::from(*artifact)),
        _ => None,
    }
}

/// Whether the hero of these details receives `item` (`h3_heroes::can_receive_artifact`, under
/// `ADV-QUIRK-22`'s flag); a hero without details receives nothing.
fn receives(core: &Ruleset, details: Option<&HeroDetails>, item: ArtifactInstance) -> bool {
    details.is_some_and(|details| can_receive_artifact(details, core, item))
}

/// An outcome giving `hero` these changes, none when there are none.
pub(super) fn changes(hero: HeroId, changes: Vec<HeroChange>) -> AdventureOutcome {
    AdventureOutcome {
        hero: (!changes.is_empty()).then(|| HeroGains::changes(hero, changes)),
        ..AdventureOutcome::default()
    }
}

impl AdventureState {
    /// The visits' block to write: built on its first write (none before, so a fork costs nothing
    /// more until then), copied when a fork shares it.
    pub(super) fn visits_mut(&mut self) -> &mut Visits {
        let (objects, players) = (self.map.objects().len(), self.players);
        let block = self
            .visits
            .get_or_insert_with(|| Arc::new(Visits::new(objects, players)));
        Arc::make_mut(block)
    }

    /// Whether `hero` holds the once-per-hero mark of the object at `index` (ADV-TCH-001 to -003).
    pub fn is_marked(&self, hero: HeroId, index: usize) -> bool {
        self.visits
            .as_ref()
            .is_some_and(|visits| visits.is_marked(hero, index))
    }

    /// Whether `player` knows what a visit left on the object at `index` (ADV-VIS-006).
    pub fn knows(&self, player: PlayerId, index: usize) -> bool {
        self.visits
            .as_ref()
            .is_some_and(|visits| visits.knows(player, index))
    }

    /// Whether a visit emptied the object at `index` (a find searched, ADV-PIK-009 to -012).
    pub fn is_emptied(&self, index: usize) -> bool {
        self.visits
            .as_ref()
            .is_some_and(|visits| visits.is_emptied(index))
    }

    /// `player` learns what a visit left on the object at `index` (ADV-VIS-006): the block is
    /// written only when the player did not know it yet.
    pub(super) fn learn(&mut self, player: PlayerId, index: usize) {
        if !self.knows(player, index) {
            self.visits_mut().learn(player, index);
        }
    }

    /// The object of the type and anchor an offer or a choice names, if it is still on the map:
    /// the last in map order with them ([`Self::tile_object`]'s order).
    pub(super) fn object_named(&self, object_type: ObjectTypeId, anchor: Pos) -> Option<usize> {
        self.map
            .objects()
            .iter()
            .enumerate()
            .rev()
            .find(|&(index, object)| {
                object.object_type == Some(object_type)
                    && object.anchor == anchor
                    && !self.is_removed(index)
            })
            .map(|(index, _)| index)
    }

    /// A treasure chest's visit (ADV-PIK-005; the original's `visit_treasure_chest`, 0x4A6520,
    /// survey §2.2): an artifact the hero receives (`ADV-QUIRK-22`: under its flag a full
    /// backpack refuses even with a body position free) is given; else the choice between the
    /// gold and its experience (the gold less the type's offset, times the hero's Learning,
    /// `h3_heroes::learning_experience`): the chest's gold, or for an artifact the hero cannot
    /// receive the type's full-backpack gold (1 000), as for a chest whose artifact draw found
    /// none (every artifact banned). Never refused; the chest leaves the map with the visit,
    /// whatever the answer.
    // rule: ADV-PIK-005
    pub(super) fn visit_chest<S: Sink>(
        &mut self,
        (core, _, heroes): Rules<'_>,
        hero: HeroId,
        (index, object_type): (usize, ObjectTypeId),
        chest: &Chest,
        sink: &mut S,
    ) -> AdventureOutcome {
        let object = &self.map.objects()[index];
        let details = details_of(heroes, hero);
        let artifact = object.resolved.artifact().map(ArtifactInstance::from);
        let outcome = match (artifact, object.contents.first()) {
            (Some(item), _) if receives(core, details, item) => {
                changes(hero, Vec::from([HeroChange::AddArtifact(item)]))
            }
            (artifact, gold) => {
                let gold = gold
                    .filter(|_| artifact.is_none())
                    .copied()
                    .unwrap_or((chest.resource, chest.full_backpack));
                let amount = gold.1.saturating_sub(chest.experience_offset);
                let experience =
                    details.map_or(0, |details| learning_experience(details, core, amount));
                AdventureOutcome {
                    choice: Some(Choice {
                        hero,
                        object_type,
                        anchor: object.anchor,
                        kind: ChoiceKind::Chest { gold, experience },
                    }),
                    ..AdventureOutcome::default()
                }
            }
        };
        self.take_object(core, (index, object_type), sink);
        outcome
    }

    /// An artifact's or a Spell Scroll's visit (ADV-PIK-006 to -008; the original's
    /// `visit_artifact` 0x49F7E0 and `visit_spell_scroll` 0x4A5EA0, survey §3.2, §4): a hero who
    /// cannot receive the artifact (a full backpack under `ADV-QUIRK-22`'s flag, or no room) is
    /// refused, nothing changes and the object stays; else its guards ask or fight as a guarded
    /// pile's ([`Self::visit_pile`]: an artifact's always ask, D-29; a scroll's only with a
    /// text), and without guards standing it is picked up ([`Self::pick_up`]). A random artifact
    /// whose draw found none (every artifact banned) holds nothing: it leaves the map, giving
    /// nothing.
    // rule: ADV-PIK-006, ADV-PIK-007, ADV-PIK-008
    pub(super) fn visit_artifact<S: Sink>(
        &mut self,
        (core, part, heroes): Rules<'_>,
        hero: HeroId,
        object: (usize, ObjectTypeId),
        sink: &mut S,
    ) -> AdventureOutcome {
        match carried(&self.map.objects()[object.0]) {
            None => {
                self.take_object(core, object, sink);
                AdventureOutcome::default()
            }
            Some(item) if !receives(core, details_of(heroes, hero), item) => {
                AdventureOutcome::default()
            }
            Some(_) => self.visit_pile(core, part, hero, object, sink),
        }
    }

    /// The pickup of the guarded object or pile at `index` once no guards stand (ADV-PIK-001,
    /// ADV-PIK-003, ADV-PIK-006, ADV-PIK-008): an artifact or a scroll goes to the hero `hero`
    /// (`HeroChange::AddArtifact`; the visit checked that it can receive it, and neither the
    /// question's wait nor a battle against guards adds to its artifacts), a pile's contents to
    /// the player; it leaves the map ([`Self::take_object`], ADV-OBJ-005).
    // rule: ADV-OBJ-005
    pub(super) fn pick_up<S: Sink>(
        &mut self,
        core: &Ruleset,
        hero: HeroId,
        (index, object_type): (usize, ObjectTypeId),
        sink: &mut S,
    ) -> AdventureOutcome {
        let object = &self.map.objects()[index];
        let outcome = match carried(object) {
            Some(item) => changes(hero, Vec::from([HeroChange::AddArtifact(item)])),
            None => AdventureOutcome {
                gains: object.contents.clone(),
                ..AdventureOutcome::default()
            },
        };
        self.take_object(core, (index, object_type), sink);
        outcome
    }

    /// A wandering monster's treasure artifact `artifact` for the hero that beat it or that it
    /// joined (ADV-MON-011; the original's `monster_treasure` 0x4A6B30, survey §5): given before
    /// the treasure's resources when the hero can receive it, else lost (a full backpack under
    /// `ADV-QUIRK-22`'s flag, or no room).
    // rule: ADV-MON-011
    pub(super) fn treasure_artifact(
        core: &Ruleset,
        heroes: &[Arc<HeroDetails>],
        hero: HeroId,
        artifact: Option<ArtifactTypeId>,
    ) -> Option<HeroGains> {
        artifact
            .map(ArtifactInstance::from)
            .filter(|&item| receives(core, details_of(heroes, hero), item))
            .map(|item| HeroGains::changes(hero, Vec::from([HeroChange::AddArtifact(item)])))
    }

    /// A once-per-hero gift's visit (ADV-TCH-001 to -003; survey §6.2 to §6.4): a hero holding the
    /// gift's mark is refused (the original's "already visited"), as one the Library does not
    /// admit (`h3_heroes::library_admits`, no mark). A gift whose type draws its price (the Tree of
    /// Knowledge) tells the visitor's owner the drawn price (ADV-VIS-006, before its price's
    /// test). Then a choice of two primary skills opens as the player's choice, for the gift's
    /// price (the Arena, the Schools: [`Self::choose`] gives it); another gift with a price opens
    /// its offer to pay ([`OfferKind::Gift`]: the game opens it when the treasury can pay, else
    /// tells the refusal); a free one is given at once ([`Self::give_gift`]), which marks the
    /// hero.
    // rule: ADV-TCH-001, ADV-TCH-002, ADV-TCH-003, ADV-VIS-006
    pub(super) fn visit_gift(
        &mut self,
        (core, _, heroes): Rules<'_>,
        (player, hero): (PlayerId, HeroId),
        (index, object_type): (usize, ObjectTypeId),
        gift: &Gift,
    ) -> AdventureOutcome {
        let details = details_of(heroes, hero);
        let admitted = gift
            .admits
            .is_none_or(|level| details.is_some_and(|d| library_admits(d, core, level)));
        if self.is_marked(hero, index) || !admitted {
            return AdventureOutcome::default();
        }
        if gift.prices.len() > 1 {
            self.learn(player, index);
        }
        let price = self.gift_price(index, gift).to_vec();
        let anchor = self.map.objects()[index].anchor;
        match gift.gives {
            Gives::Choice { skills, amount } => AdventureOutcome {
                choice: Some(Choice {
                    hero,
                    object_type,
                    anchor,
                    kind: ChoiceKind::Primary {
                        skills,
                        amount,
                        price,
                        leave: gift.leave,
                    },
                }),
                ..AdventureOutcome::default()
            },
            _ if !price.is_empty() => AdventureOutcome {
                offer: Some(Offer {
                    hero,
                    object_type,
                    anchor,
                    kind: OfferKind::Gift { price },
                }),
                ..AdventureOutcome::default()
            },
            gives => self.give_gift(core, details, hero, index, gives),
        }
    }

    /// The price of the gift at `index`: none for a free type, the type's one price, or the one
    /// the build drew among its prices (ADV-TCH-003: the Tree of Knowledge's, `Resolved::Price`).
    pub(crate) fn gift_price<'g>(&self, index: usize, gift: &'g Gift) -> &'g Resources {
        const FREE: &Resources = &Vec::new();
        let drawn = match self.map.objects()[index].resolved {
            Resolved::Price(drawn) => usize::from(drawn),
            _ => 0,
        };
        gift.prices.get(drawn).unwrap_or(FREE)
    }

    /// Gives the gift `gives` of the object at `index` to `hero` and marks it (ADV-TCH-001,
    /// ADV-TCH-003): primary skills (`HeroChange::GainPrimary`, at most `primary_max`: A-9's
    /// class), an experience amount times the hero's Learning (the Learning Stone's,
    /// `h3_heroes::learning_experience`) or the experience to the hero's next level without
    /// Learning (the Tree's, `h3_heroes::next_level_experience`), which the game gives last with
    /// its level-up. A hero without details gets the mark and no experience.
    // rule: ADV-TCH-001, ADV-TCH-003
    fn give_gift(
        &mut self,
        core: &Ruleset,
        details: Option<&HeroDetails>,
        hero: HeroId,
        index: usize,
        gives: Gives,
    ) -> AdventureOutcome {
        self.visits_mut().mark(hero, index);
        let gains = match gives {
            Gives::Primary(gains) => {
                HeroGains::changes(hero, Vec::from([HeroChange::GainPrimary(gains)]))
            }
            Gives::Experience(amount) => HeroGains::experience(
                hero,
                details.map_or(0, |details| learning_experience(details, core, amount)),
            ),
            // A choice never comes here: [`Self::choose`] gives it.
            Gives::Level | Gives::Choice { .. } => HeroGains::experience(
                hero,
                details.map_or(0, |details| next_level_experience(details, core)),
            ),
        };
        AdventureOutcome {
            hero: Some(gains),
            ..AdventureOutcome::default()
        }
    }

    /// An accepted offer to pay a gift's price (ADV-TCH-003, [`OfferKind::Gift`]; the game paid
    /// it): the gift given and the hero marked ([`Self::give_gift`]); nothing when the object is
    /// gone or is no gift.
    pub(super) fn accept_gift(
        &mut self,
        (core, part, heroes): Rules<'_>,
        offer: &Offer,
    ) -> AdventureOutcome {
        let gift = part
            .object_type(offer.object_type)
            .and_then(|ty| match &ty.kind {
                ObjectKind::Gift(gift) => Some(gift.gives),
                _ => None,
            });
        match (self.object_named(offer.object_type, offer.anchor), gift) {
            (Some(index), Some(gives)) => self.give_gift(
                core,
                details_of(heroes, offer.hero),
                offer.hero,
                index,
                gives,
            ),
            _ => AdventureOutcome::default(),
        }
    }

    /// The answer `pick` to `choice`, made by a visit of `player`'s (E08 T62a; the game checked
    /// that the choice is its first open decision and paid a School's price for a pick that is
    /// not leaving): a chest's first pick is its gold, the outcome's gain, its second the
    /// experience to the hero (ADV-PIK-005); a gift's pick adds its amount to that primary skill
    /// (`HeroChange::GainPrimary`) and marks the hero, while the object is on the map
    /// (ADV-TCH-001, ADV-TCH-002). Leaving, or a pick the choice does not allow
    /// ([`Choice::allows`]), gives nothing. Infallible.
    // rule: ADV-PIK-005, ADV-TCH-001, ADV-TCH-002
    pub fn choose(&mut self, choice: &Choice, pick: Pick) -> AdventureOutcome {
        let hero = choice.hero;
        match (&choice.kind, pick) {
            (_, Pick::Leave) => AdventureOutcome::default(),
            (ChoiceKind::Chest { gold, .. }, Pick::First) => AdventureOutcome {
                gains: Vec::from([*gold]),
                ..AdventureOutcome::default()
            },
            (ChoiceKind::Chest { experience, .. }, Pick::Second) => AdventureOutcome {
                hero: Some(HeroGains::experience(hero, *experience)),
                ..AdventureOutcome::default()
            },
            (ChoiceKind::Primary { skills, amount, .. }, pick) => {
                let Some(index) = self.object_named(choice.object_type, choice.anchor) else {
                    return AdventureOutcome::default();
                };
                self.visits_mut().mark(hero, index);
                let mut gains = [0; 4];
                gains[skills[usize::from(pick == Pick::Second)].index()] = *amount;
                changes(hero, Vec::from([HeroChange::GainPrimary(gains)]))
            }
        }
    }

    /// A Shrine's visit (ADV-TCH-004; the original's `visit_shrine` 0x4A5610, survey §6.5): its
    /// owner learns the spell first (ADV-VIS-006), then a hero who cannot learn it (no Spell
    /// Book, a level above its Wisdom's, a known spell: `h3_heroes::can_learn_spell`) is refused,
    /// as by a Shrine without a spell to give (A-18 fixed); else it learns it. No mark: every
    /// visit.
    // rule: ADV-TCH-004, ADV-VIS-006
    pub(super) fn visit_shrine(
        &mut self,
        (core, _, heroes): Rules<'_>,
        (player, hero): (PlayerId, HeroId),
        index: usize,
    ) -> AdventureOutcome {
        self.learn(player, index);
        let details = details_of(heroes, hero);
        match self.map.objects()[index].resolved {
            Resolved::Spell(spell)
                if details.is_some_and(|d| can_learn_spell(d, core, spell, true)) =>
            {
                changes(hero, Vec::from([HeroChange::LearnSpell(spell)]))
            }
            _ => AdventureOutcome::default(),
        }
    }

    /// A Witch Hut's visit (ADV-TCH-005; the original's `visit_witch_hut` 0x4A8080, survey
    /// §6.6): its owner learns the skill first (ADV-VIS-006), then a hero it does not teach (a
    /// known skill, eight skills: `h3_heroes::hut_teaches`; no class test) is refused, as by a hut
    /// without a skill (every skill of its mask banned); else the skill at Basic. No mark: every
    /// visit.
    // rule: ADV-TCH-005, ADV-VIS-006
    pub(super) fn visit_witch_hut(
        &mut self,
        (core, _, heroes): Rules<'_>,
        (player, hero): (PlayerId, HeroId),
        index: usize,
    ) -> AdventureOutcome {
        self.learn(player, index);
        let details = details_of(heroes, hero);
        match self.map.objects()[index].resolved {
            Resolved::Skill(skill) if details.is_some_and(|d| hut_teaches(d, core, skill)) => {
                changes(
                    hero,
                    Vec::from([HeroChange::RaiseSkill { skill, level: 1 }]),
                )
            }
            _ => AdventureOutcome::default(),
        }
    }

    /// A Scholar's visit (ADV-TCH-006; the original's `visit_scholar` 0x4A4DC0, survey §6.7): its
    /// gift, or +1 to the drawn primary skill instead of a skill or spell the hero cannot take
    /// (`h3_heroes::scholar_reward`); then it leaves the map. A hero without details gets
    /// nothing.
    // rule: ADV-TCH-006
    pub(super) fn visit_scholar<S: Sink>(
        &mut self,
        (core, _, heroes): Rules<'_>,
        hero: HeroId,
        object: (usize, ObjectTypeId),
        sink: &mut S,
    ) -> AdventureOutcome {
        let taught = match (
            &self.map.objects()[object.0].resolved,
            details_of(heroes, hero),
        ) {
            (&Resolved::Scholar(gift, fallback), Some(details)) => {
                Vec::from([scholar_reward(details, core, gift, fallback)])
            }
            _ => Vec::new(),
        };
        self.take_object(core, object, sink);
        changes(hero, taught)
    }

    /// A University's visit (ADV-TCH-007, survey §6.8): its owner learns its skills
    /// (ADV-VIS-006: the dialog shows them), and it opens over them at the type's price
    /// ([`AdventureOutcome::university`]); every visit.
    // rule: ADV-TCH-007, ADV-VIS-006
    pub(super) fn visit_university(
        &mut self,
        (player, hero): (PlayerId, HeroId),
        (index, object_type): (usize, ObjectTypeId),
        price: &Resources,
    ) -> AdventureOutcome {
        self.learn(player, index);
        let object = &self.map.objects()[index];
        AdventureOutcome {
            university: Some(UniversityVisit {
                hero,
                object_type,
                anchor: object.anchor,
                skills: object.resolved.skills().to_vec(),
                price: price.clone(),
            }),
            ..AdventureOutcome::default()
        }
    }

    /// A find's visit (ADV-PIK-009 to -012; survey §6.9): one that asks (the Warrior's Tomb)
    /// opens its question ([`OfferKind::Search`], every visit); else it is searched at once
    /// ([`Self::search`]).
    // rule: ADV-PIK-011
    pub(super) fn visit_find(
        &mut self,
        rules: Rules<'_>,
        (player, hero): (PlayerId, HeroId),
        (index, object_type): (usize, ObjectTypeId),
        find: &Find,
    ) -> AdventureOutcome {
        if !find.question {
            return self.search(rules, (player, hero), index, find);
        }
        AdventureOutcome {
            offer: Some(Offer {
                hero,
                object_type,
                anchor: self.map.objects()[index].anchor,
                kind: OfferKind::Search,
            }),
            ..AdventureOutcome::default()
        }
    }

    /// An accepted find's question ([`OfferKind::Search`]): the find searched ([`Self::search`]);
    /// nothing when the object is gone or is no find.
    pub(super) fn accept_search(
        &mut self,
        rules: Rules<'_>,
        player: PlayerId,
        offer: &Offer,
    ) -> AdventureOutcome {
        let find = rules
            .1
            .object_type(offer.object_type)
            .and_then(|ty| match &ty.kind {
                ObjectKind::Find(find) => Some(find),
                _ => None,
            });
        match (self.object_named(offer.object_type, offer.anchor), find) {
            (Some(index), Some(find)) => self.search(rules, (player, offer.hero), index, find),
            _ => AdventureOutcome::default(),
        }
    }

    /// A find searched by `player`'s hero `hero` (ADV-PIK-009 to -012; the original's
    /// `visit_corpse` 0x4A5480, `visit_wagon` 0x4A69B0, `visit_warriors_tomb` 0x4A7C30 after its
    /// question, `visit_lean_to` 0x4A31A0): its owner learns whether it is emptied (ADV-VIS-006).
    /// Not emptied yet: an artifact the hero receives (`ADV-QUIRK-22`) is given; one it cannot
    /// receive turns as the type says (the Corpse's 1 000 gold, the Wagon's drawn resources, or
    /// it stays in the Tomb, which is then not emptied); without an artifact the find's
    /// resources, or nothing. Then the type's bonus (the Tomb's −3 morale, once until the next
    /// battle by its mark: HER-ADV-008), whether or not the find held anything or was emptied.
    // rule: ADV-PIK-009, ADV-PIK-010, ADV-PIK-011, ADV-PIK-012, ADV-VIS-006
    fn search(
        &mut self,
        (core, _, heroes): Rules<'_>,
        (player, hero): (PlayerId, HeroId),
        index: usize,
        find: &Find,
    ) -> AdventureOutcome {
        self.learn(player, index);
        let mut taken = Vec::new();
        let mut outcome = AdventureOutcome::default();
        if !self.is_emptied(index) {
            let object = &self.map.objects()[index];
            let artifact = object.resolved.artifact().map(ArtifactInstance::from);
            let empties = match (artifact, &find.full_backpack) {
                (Some(item), _) if receives(core, details_of(heroes, hero), item) => {
                    taken.push(HeroChange::AddArtifact(item));
                    true
                }
                (Some(_), Some(Turn::Give(resources))) => {
                    outcome.gains.clone_from(resources);
                    true
                }
                (Some(_), Some(Turn::Stays) | None) => false,
                (Some(_), Some(Turn::Drawn)) | (None, _) => {
                    outcome.gains.clone_from(&object.contents);
                    true
                }
            };
            if empties {
                self.visits_mut().empty(index);
            }
        }
        taken.extend(find.bonus.map(|bonus| HeroChange::Bonus {
            morale: bonus.morale,
            luck: bonus.luck,
            mark: Some(bonus.mark),
        }));
        AdventureOutcome {
            hero: changes(hero, taken).hero,
            ..outcome
        }
    }
}
