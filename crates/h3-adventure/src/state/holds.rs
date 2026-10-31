//! What the map puts in an object of increment 7's kinds (E08 T61a; increment 7's design note §7 in
//! `E08-log.md`): checked against the object's kind when the game is built, and
//! written into the static map's hash. The visits that read it come with T62.

use alloc::collections::BTreeSet;

use h3_content::{ArtifactTypeId, CreatureFlag, Ruleset, SpellCategory, SpellId};
use h3_heroes::{ArtifactInstance, ScholarGift, validate_artifact};
use h3_kernel::{CanonicalHash, StateHasher};

use super::check_army;
use crate::ruleset::{AdventureRuleset, ObjectKind};
use crate::setup::{EventSetup, Holds, RewardRecord};

/// Whether `holds` fits `kind`, an object's (none for a blocker), and is valid (ADV-OBJ-002):
/// a placed artifact holds an artifact without a spell, a scroll the Spell Scroll with one, each
/// one a hero may hold; a Shrine a hero spell or none; a Witch Hut distinct skills or none; a
/// Scholar a gift of the ruleset or none; Pandora's Box and an event a valid record
/// ([`check_record`]); since E08 T83a an abandoned mine at least one resource of the ruleset, in
/// the core's order without repeats; every other kind nothing.
pub(super) fn check_holds(
    core: &Ruleset,
    part: &AdventureRuleset,
    kind: Option<&ObjectKind>,
    holds: &Holds,
) -> bool {
    match (kind, holds) {
        (Some(ObjectKind::Artifact { classes: None }), Holds::Artifact(item)) => {
            item.spell.is_none() && validate_artifact(*item, core).is_ok()
        }
        (Some(ObjectKind::Scroll), Holds::Artifact(item)) => {
            item.spell.is_some() && validate_artifact(*item, core).is_ok()
        }
        (Some(ObjectKind::Shrine { .. }), Holds::Spell(spell)) => {
            spell.is_none_or(|spell| hero_spell(core, spell))
        }
        (Some(ObjectKind::WitchHut { .. }), Holds::Skills(skills)) => {
            skills.as_ref().is_none_or(|skills| {
                skills.iter().all(|&skill| core.skill(skill).is_some())
                    && skills.iter().collect::<BTreeSet<_>>().len() == skills.len()
            })
        }
        (Some(ObjectKind::Scholar), Holds::Gift(gift)) => gift.is_none_or(|gift| match gift {
            ScholarGift::Primary(_) => true,
            ScholarGift::Skill(skill) => core.skill(skill).is_some(),
            ScholarGift::Spell(spell) => hero_spell(core, spell),
        }),
        (Some(ObjectKind::PandoraBox), Holds::Record(record)) => check_record(core, part, record),
        (Some(ObjectKind::Event), Holds::Event(event)) => check_record(core, part, &event.record),
        // rule: ADV-MIN-003
        (Some(ObjectKind::AbandonedMine { .. }), Holds::Resources(resources)) => {
            !resources.is_empty()
                && resources.windows(2).all(|pair| pair[0] < pair[1])
                && resources
                    .iter()
                    .all(|&r| usize::from(r.raw()) < core.resources().len())
        }
        (
            Some(
                ObjectKind::Artifact { classes: None }
                | ObjectKind::Scroll
                | ObjectKind::Shrine { .. }
                | ObjectKind::WitchHut { .. }
                | ObjectKind::Scholar
                | ObjectKind::PandoraBox
                | ObjectKind::Event
                | ObjectKind::AbandonedMine { .. },
            ),
            _,
        ) => false,
        (_, holds) => *holds == Holds::Nothing,
    }
}

/// A monster's treasure artifact (ADV-MON-011): one a hero may hold, without a spell.
pub(super) fn check_treasure_artifact(core: &Ruleset, artifact: Option<ArtifactTypeId>) -> bool {
    artifact
        .is_none_or(|artifact| validate_artifact(ArtifactInstance::from(artifact), core).is_ok())
}

/// A spell of the ruleset that is no creature ability.
fn hero_spell(core: &Ruleset, spell: SpellId) -> bool {
    core.spell(spell)
        .is_some_and(|s| s.category != SpellCategory::Ability)
}

/// A reward record's parts (ADV-RWD-001): its skills the ruleset's at Basic to Expert, its
/// artifacts the ruleset's, its spells hero spells; its resources distinct and the ruleset's,
/// none 0; its creatures, if any, valid as an army with no war machine.
fn check_record(core: &Ruleset, part: &AdventureRuleset, record: &RewardRecord) -> bool {
    let hero = &record.hero;
    let resources = record
        .resources
        .iter()
        .map(|(resource, _)| resource)
        .collect::<BTreeSet<_>>();
    let creatures = record.creatures.is_empty()
        || check_army(core, part, &record.creatures).is_ok_and(|army| {
            army.iter().all(|stack| {
                core.creature(stack.creature)
                    .is_some_and(|c| !c.flags.contains(CreatureFlag::WarMachine))
            })
        });
    hero.skills
        .iter()
        .all(|&(skill, level)| core.skill(skill).is_some() && (1..=3).contains(&level))
        && hero
            .artifacts
            .iter()
            .all(|&artifact| core.artifact(artifact).is_some())
        && hero.spells.iter().all(|&spell| hero_spell(core, spell))
        && resources.len() == record.resources.len()
        && record.resources.iter().all(|&(resource, amount)| {
            usize::from(resource.raw()) < core.resources().len() && amount != 0
        })
        && creatures
}

impl CanonicalHash for Holds {
    /// Its tag (0 nothing, 1 an artifact: the artifact and its spell; 2 a spell, 3 skills, 4 a
    /// Scholar's gift, each 0 for none or 1 and it; 5 a record; 6 an event: the record, the
    /// colours, the computer and removal flags; since adventure state hash v17, E08 T83a, 7 an
    /// abandoned mine's resources as length and handles).
    fn hash_into(&self, h: &mut StateHasher) {
        match self {
            Self::Nothing => h.write_u8(0),
            Self::Artifact(item) => {
                h.write_u8(1);
                item.artifact.hash_into(h);
                item.spell.hash_into(h);
            }
            Self::Spell(spell) => {
                h.write_u8(2);
                spell.hash_into(h);
            }
            Self::Skills(skills) => {
                h.write_u8(3);
                match skills {
                    None => h.write_u8(0),
                    Some(skills) => {
                        h.write_u8(1);
                        skills.hash_into(h);
                    }
                }
            }
            Self::Gift(gift) => {
                h.write_u8(4);
                match gift {
                    None => h.write_u8(0),
                    Some(ScholarGift::Primary(skill)) => {
                        h.write_u8(1);
                        h.write_u8(*skill as u8);
                    }
                    Some(ScholarGift::Skill(skill)) => {
                        h.write_u8(2);
                        skill.hash_into(h);
                    }
                    Some(ScholarGift::Spell(spell)) => {
                        h.write_u8(3);
                        spell.hash_into(h);
                    }
                }
            }
            Self::Record(record) => {
                h.write_u8(5);
                record.hash_into(h);
            }
            Self::Event(event) => {
                h.write_u8(6);
                event.hash_into(h);
            }
            Self::Resources(resources) => {
                h.write_u8(7);
                resources.hash_into(h);
            }
        }
    }
}

impl CanonicalHash for RewardRecord {
    /// The experience, the primary skills, the skills as length and pairs, the mana, the morale
    /// and luck, the artifacts, the spells, the resources as length and pairs, the creatures as
    /// length and slot, creature and count each.
    fn hash_into(&self, h: &mut StateHasher) {
        let hero = &self.hero;
        h.write_u32(hero.experience);
        for value in hero.primary {
            h.write_i32(i32::from(value));
        }
        h.write_len(hero.skills.len());
        for &(skill, level) in &hero.skills {
            skill.hash_into(h);
            h.write_u8(level);
        }
        h.write_i32(hero.mana);
        h.write_i32(i32::from(hero.morale));
        h.write_i32(i32::from(hero.luck));
        hero.artifacts.hash_into(h);
        hero.spells.hash_into(h);
        h.write_len(self.resources.len());
        for &(resource, amount) in &self.resources {
            resource.hash_into(h);
            h.write_i32(amount);
        }
        h.write_len(self.creatures.len());
        for stack in &self.creatures {
            h.write_u8(stack.slot);
            stack.creature.hash_into(h);
            h.write_u32(stack.count);
        }
    }
}

impl CanonicalHash for EventSetup {
    fn hash_into(&self, h: &mut StateHasher) {
        self.record.hash_into(h);
        h.write_u8(self.colours);
        h.write_bool(self.computer);
        h.write_bool(self.remove);
    }
}
