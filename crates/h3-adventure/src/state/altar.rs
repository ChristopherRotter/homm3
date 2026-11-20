//! The Altars of Sacrifice on the map (E08 T134a; increment 14's design note §3, §10, §11, §14;
//! survey `docs/rules/surveys/E08-trades.md` §12): the screens a visitor's class's faction allows
//! (ADV-ALT-001), the adventure's checks and changes of a sacrifice of artifacts (ADV-ALT-002) and
//! of creatures (ADV-ALT-003), which the game makes (ADR-0017 point 10, as the market's: it removes
//! the artifacts, gives the experience and the levels), and what the window offers its visitor.
//! The visit itself is `AdventureState::visit`'s (`AdventureOutcome::service`). A sacrifice names
//! everything it takes in one answer (decision 366, A-48 fixed as `ADV-QUIRK-50`): nothing lies on
//! an altar between commands.

use alloc::sync::Arc;
use alloc::vec::Vec;

use h3_content::{ArtifactTypeId, CreatureTypeId, Ruleset};
use h3_contracts::HeroId;
use h3_heroes::{ArtifactPlace, HeroDetails, Position, experience_for, gained_experience};

use super::{AdventureState, HeroOnMap};
use crate::army::{ArmyRef, ArmySlot};
use crate::command::{ALTAR_SLOTS, AltarError, AltarPlaces, AltarVisit};
use crate::movement::details_of;
use crate::ruleset::{AdventureRuleset, Altar, AltarScreens, ObjectKind};

/// The rules an Altar's checks read: the core, the ruleset part and the game's store of hero
/// details.
type Rules<'a> = (&'a Ruleset, &'a AdventureRuleset, &'a [Arc<HeroDetails>]);

/// What `hero` sacrifices at an Altar (ADV-ALT-001; survey §12.1, the original's
/// `altar_dialog_create` 0x55FDD0): its class's faction's screens, none for a hero the store
/// lacks or a faction without an entry (none in the packs: with an Altar every faction has one).
pub(super) fn screens((core, part, heroes): Rules<'_>, hero: HeroId) -> Option<AltarScreens> {
    let details = details_of(heroes, hero)?;
    let class = core.hero_type(details.hero_type)?.class;
    part.altar_screens(core.hero_class(class)?.faction)
}

/// A sacrifice [`AdventureState::check_artifact_sacrifice`] or
/// [`AdventureState::check_creature_sacrifice`] accepted (ADV-ALT-002, -003; E08 T134a,
/// increment 14's design note §11): what it takes and its experience, which the game gives
/// (`HeroChange::GainExperience`, then the levels) once [`AdventureState::sacrifice`] took it. It
/// holds for the state that checked it; not `Clone`, so one check sacrifices once (the market's
/// form).
#[derive(Debug, PartialEq, Eq)]
pub struct Sacrifice {
    /// The sacrificing hero.
    hero: HeroId,
    /// What it takes.
    given: Given,
    /// The experience, each item's with Learning summed (saturating).
    experience: u32,
}

/// What a sacrifice takes.
#[derive(Debug, PartialEq, Eq)]
enum Given {
    /// The artifacts at these places.
    Artifacts(AltarPlaces),
    /// So many creatures of each slot.
    Creatures([u32; ALTAR_SLOTS]),
}

impl Sacrifice {
    /// The hero.
    pub const fn hero(&self) -> HeroId {
        self.hero
    }

    /// The experience it gives (before the scenario's level limit, HER-EXP-003).
    pub const fn experience(&self) -> u32 {
        self.experience
    }
}

/// What [`AdventureState::sacrifice`] took (E08 T134a): the artifacts' places the game removes
/// from the hero, in this order (`HeroChange::RemoveArtifact`, each told as `HeroChanged`), or
/// the army's slots it lowered (each told as `StackChanged`), and the experience the game gives.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sacrificed {
    /// The hero.
    pub hero: HeroId,
    /// The artifacts' places, in the change's order ([`AltarPlaces::places`]: the worn ones by
    /// position, then the backpack's from the last index down, so no place moves before it is
    /// removed); none for creatures.
    pub removed: Vec<ArtifactPlace>,
    /// The slots lowered (a slot at 0 emptied), in slot order; none for artifacts.
    pub lowered: Vec<ArmySlot>,
    /// The experience.
    pub experience: u32,
}

/// What an Altar's window offers its visitor (ADV-ALT-001 to -003, GAM-DEC-012; E08 T134a,
/// increment 14's design note §14): the screens, and for each screen it shows the experience the
/// state's answer would give, so a client and the env need no copy of the computation (ADR-0004:
/// the answers are checked against the state again, never against this). Read at each view.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AltarOffers {
    /// The screens the hero's class's faction allows.
    pub screens: AltarScreens,
    /// The most artifacts a sacrifice takes (the base pack's 23).
    pub places: u8,
    /// With the artifacts' screen: each artifact of the hero, the worn ones by position, then the
    /// backpack's in its order. Empty without the screen.
    pub artifacts: Vec<AltarArtifact>,
    /// With the creatures' screen: each stack of the hero's army in its order. Empty without the
    /// screen.
    pub stacks: Vec<AltarStack>,
    /// The experience the hero lacks for its next level (the original's `altar_update` 0x562500
    /// shows it beside the total; 0 without the hero parameters).
    pub to_next_level: u32,
}

/// An artifact in an Altar's window (ADV-ALT-002; E08 T134a): its place, its type and the
/// experience it gives the hero with Learning, none for one the Altar never takes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AltarArtifact {
    /// Where the hero holds it.
    pub place: ArtifactPlace,
    /// The artifact.
    pub artifact: ArtifactTypeId,
    /// Its experience with Learning (0 for a special one, A-47), none for a kept one.
    pub experience: Option<u32>,
}

/// A stack in an Altar's window (ADV-ALT-003; E08 T134a): its slot, a creature's experience before
/// Learning, the most a sacrifice takes of it and that most's experience with Learning.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AltarStack {
    /// The hero's army's slot.
    pub slot: u8,
    /// A creature's experience before Learning: `factor × ⌊AI value / divisor⌋`.
    pub each: u32,
    /// The most a sacrifice takes of it: its count, less one when the army's other slots are
    /// empty (the army keeps a creature; survey §12.3's slider).
    pub most: u32,
    /// The experience of `most` creatures with Learning.
    pub experience: u32,
}

/// The artifact at `place` of `details`, if one.
fn artifact_at(details: &HeroDetails, place: ArtifactPlace) -> Option<ArtifactTypeId> {
    let item = match place {
        ArtifactPlace::Worn(at) => details.worn.get(at.0).and_then(Option::as_ref),
        ArtifactPlace::Backpack(i) => details.backpack.get(i),
    };
    item.map(|item| item.artifact)
}

/// The experience `artifact` gives at `altar` before Learning, by its class (survey §12.2: the
/// original's `altar_artifact_entry` 0x55FC30); 0 for an artifact the core lacks.
fn artifact_base(core: &Ruleset, altar: &Altar, artifact: ArtifactTypeId) -> u32 {
    core.artifacts()
        .get(artifact.index())
        .map_or(0, |ty| altar.artifact_experience(ty.class))
}

/// A creature's experience at `altar` before Learning, `factor × ⌊AI value / divisor⌋` (survey
/// §12.3: the original's `altar_set_creature_amount` 0x564C00, 5 × ⌊AI value / 40⌋), saturating;
/// 0 for a creature the core lacks.
fn creature_base(core: &Ruleset, altar: &Altar, creature: CreatureTypeId) -> u32 {
    core.creature(creature).map_or(0, |info| {
        let parts = info.ai_value / altar.creature_divisor.get();
        altar.creature_factor.saturating_mul(parts)
    })
}

/// `amount` creatures of `each` experience, in 32 bits saturated (the original's product).
fn creatures_base(amount: u32, each: u32) -> u32 {
    u32::try_from(u64::from(amount) * u64::from(each)).unwrap_or(u32::MAX)
}

/// The Altar a visit opened, as its checks read it.
#[derive(Debug)]
struct Window<'a> {
    /// The Altar's kind.
    altar: &'a Altar,
    /// The hero on the map (its army).
    hero: &'a HeroOnMap,
    /// The hero's details (its artifacts, its Learning).
    details: &'a HeroDetails,
    /// Its class's faction's screens.
    screens: AltarScreens,
}

impl AdventureState {
    /// The Altar `visit` opened: its hero on the map and in the store with its class's faction's
    /// screens, its object an Altar's ([`AltarError::Visit`] else).
    fn altar<'a>(&'a self, rules: Rules<'a>, visit: &AltarVisit) -> Result<Window<'a>, AltarError> {
        let part = rules.1;
        let kind = self
            .object(visit.index)
            .and_then(|object| object.object_type)
            .and_then(|ty| part.object_type(ty))
            .map(|ty| &ty.kind);
        let hero = self.hero(visit.hero);
        let details = details_of(rules.2, visit.hero);
        let found = (kind, hero, details, screens(rules, visit.hero));
        let (Some(ObjectKind::Altar(altar)), Some(hero), Some(details), Some(screens)) = found
        else {
            return Err(AltarError::Visit);
        };
        Ok(Window {
            altar,
            hero,
            details,
            screens,
        })
    }

    /// Checks a sacrifice of the artifacts at `places` (ADV-ALT-002; E08 T134a, increment 14's
    /// design note §11; survey `E08-trades.md` §12.2, the original's `altar_worn_click` 0x5632A0,
    /// `altar_backpack_click` 0x5636C0 and `altar_sacrifice` 0x5646A0): each artifact's experience
    /// by its class (the Altar's), grown by the hero's Learning **per artifact**
    /// (`h3_heroes::gained_experience`, HER-EXP-001's single computation with quirk Q-0005: D-73,
    /// the original's `trunc(L × f32(base))` for each), summed saturating. A special artifact gives
    /// 0 and goes with the others (A-47, harmless and accepted as `ADV-QUIRK-49`). Refused, in this
    /// order: the visit ([`AltarError::Visit`]), a faction that does not sacrifice artifacts
    /// ([`AltarError::Screen`]), more places than the Altar takes ([`AltarError::Places`]), then
    /// for each place in the change's order one without an artifact ([`AltarError::Places`]) or
    /// with one the Altar never takes ([`AltarError::Kept`]: the Spell Book, the Catapult), and a
    /// total of 0 ([`AltarError::Nothing`]: no place, or special artifacts alone).
    // rule: ADV-ALT-002, HER-EXP-001
    pub fn check_artifact_sacrifice(
        &self,
        rules: Rules<'_>,
        visit: &AltarVisit,
        places: AltarPlaces,
    ) -> Result<Sacrifice, AltarError> {
        let core = rules.0;
        let window = self.altar(rules, visit)?;
        if !window.screens.artifacts {
            return Err(AltarError::Screen);
        }
        if places.len() > u32::from(window.altar.places) {
            return Err(AltarError::Places);
        }
        let mut experience = 0_u32;
        for place in places.places() {
            let artifact = artifact_at(window.details, place).ok_or(AltarError::Places)?;
            if window.altar.keeps.contains(&artifact) {
                return Err(AltarError::Kept(artifact));
            }
            let base = artifact_base(core, window.altar, artifact);
            experience = experience.saturating_add(gained_experience(window.details, core, base));
        }
        if experience == 0 {
            return Err(AltarError::Nothing);
        }
        Ok(Sacrifice {
            hero: visit.hero,
            given: Given::Artifacts(places),
            experience,
        })
    }

    /// Checks a sacrifice of `amounts` creatures by slot (ADV-ALT-003; E08 T134a, increment 14's
    /// design note §11; survey `E08-trades.md` §12.3, the original's `altar_set_creature_amount`
    /// 0x564C00 and `altar_sacrifice` 0x5646A0): each stack's experience `amount × e`, e = `factor
    /// × ⌊AI value / divisor⌋`, grown by the hero's Learning **per stack** (D-73), summed
    /// saturating. Refused, in this order: the visit ([`AltarError::Visit`]), a faction that does
    /// not sacrifice creatures ([`AltarError::Screen`]), a slot's amount above its stack's count
    /// or on an empty slot, in slot order ([`AltarError::Amount`]), the army's last creature
    /// ([`AltarError::LastCreature`]: the army keeps one), a total of 0 ([`AltarError::Nothing`]:
    /// no amount, or creatures of an AI value below the divisor alone).
    // rule: ADV-ALT-003, HER-EXP-001
    pub fn check_creature_sacrifice(
        &self,
        rules: Rules<'_>,
        visit: &AltarVisit,
        amounts: [u32; ALTAR_SLOTS],
    ) -> Result<Sacrifice, AltarError> {
        let core = rules.0;
        let window = self.altar(rules, visit)?;
        if !window.screens.creatures {
            return Err(AltarError::Screen);
        }
        let mut experience = 0_u32;
        let mut taken = 0_u64;
        for (slot, amount) in (0_u8..).zip(amounts).filter(|&(_, amount)| amount > 0) {
            let stack = window.hero.army.iter().find(|stack| stack.slot == slot);
            let Some(stack) = stack.filter(|stack| amount <= stack.count) else {
                return Err(AltarError::Amount(slot));
            };
            taken += u64::from(amount);
            let base = creatures_base(amount, creature_base(core, window.altar, stack.creature));
            experience = experience.saturating_add(gained_experience(window.details, core, base));
        }
        let army: u64 = window.hero.army.iter().map(|s| u64::from(s.count)).sum();
        if taken > 0 && taken >= army {
            return Err(AltarError::LastCreature);
        }
        if experience == 0 {
            return Err(AltarError::Nothing);
        }
        Ok(Sacrifice {
            hero: visit.hero,
            given: Given::Creatures(amounts),
            experience,
        })
    }

    /// Makes `sacrifice` (ADV-ALT-002, -003; E08 T134a), which a check gave for this state as it
    /// stands: the artifacts' places returned for the game to remove from the hero's details
    /// (the store is the game's), or the hero's stacks lowered by their amounts, a slot emptied at
    /// 0; with the experience the game gives. No adventure event (the game tells each change).
    // rule: ADV-ALT-002, ADV-ALT-003
    pub fn sacrifice(&mut self, sacrifice: Sacrifice) -> Sacrificed {
        let Sacrifice {
            hero,
            given,
            experience,
        } = sacrifice;
        match given {
            Given::Artifacts(places) => Sacrificed {
                hero,
                removed: places.places().collect(),
                lowered: Vec::new(),
                experience,
            },
            Given::Creatures(amounts) => {
                // The checked hero: no branch for a hero the check did not find.
                self.hero_mut(hero).into_iter().for_each(|on_map| {
                    on_map.army.iter_mut().for_each(|stack| {
                        let amount = amounts.get(usize::from(stack.slot)).copied();
                        stack.count = stack.count.saturating_sub(amount.unwrap_or_default());
                    });
                    on_map.army.retain(|stack| stack.count > 0);
                });
                let lowered = (0_u8..)
                    .zip(amounts)
                    .filter(|&(_, amount)| amount > 0)
                    .map(|(slot, _)| ArmySlot {
                        army: ArmyRef::Hero(hero),
                        slot,
                    })
                    .collect();
                Sacrificed {
                    hero,
                    removed: Vec::new(),
                    lowered,
                    experience,
                }
            }
        }
    }

    /// What `visit`'s window offers (ADV-ALT-001 to -003; E08 T134a, increment 14's design note
    /// §14): the screens, the most places; with the artifacts' screen each artifact of the hero
    /// with its experience (with Learning; none for a kept one), with the creatures' each stack's
    /// creature experience before Learning, its most and that most's experience with Learning;
    /// the experience to the hero's next level. The visit refused as the checks'
    /// ([`AltarError::Visit`]).
    // rule: ADV-ALT-001, ADV-ALT-002, ADV-ALT-003
    pub fn altar_offers(
        &self,
        rules: Rules<'_>,
        visit: &AltarVisit,
    ) -> Result<AltarOffers, AltarError> {
        let core = rules.0;
        let Window {
            altar,
            hero,
            details,
            screens,
        } = self.altar(rules, visit)?;
        let offer = |place, artifact| AltarArtifact {
            place,
            artifact,
            experience: (!altar.keeps.contains(&artifact))
                .then(|| gained_experience(details, core, artifact_base(core, altar, artifact))),
        };
        let worn = (0..details.worn.len()).map(|p| ArtifactPlace::Worn(Position(p)));
        let backpack = (0..details.backpack.len()).map(ArtifactPlace::Backpack);
        let artifacts = worn
            .chain(backpack)
            .filter(|_| screens.artifacts)
            .filter_map(|place| artifact_at(details, place).map(|artifact| offer(place, artifact)))
            .collect();
        let lone = u32::from(hero.army.len() == 1);
        let stacks = hero
            .army
            .iter()
            .filter(|_| screens.creatures)
            .map(|stack| {
                let each = creature_base(core, altar, stack.creature);
                let most = stack.count.saturating_sub(lone);
                let experience = gained_experience(details, core, creatures_base(most, each));
                AltarStack {
                    slot: stack.slot,
                    each,
                    most,
                    experience,
                }
            })
            .collect();
        let to_next_level = core.hero_params().map_or(0, |params| {
            let next = experience_for(params, details.level.saturating_add(1));
            next.saturating_sub(details.experience)
        });
        Ok(AltarOffers {
            screens,
            places: altar.places,
            artifacts,
            stacks,
            to_next_level,
        })
    }
}
