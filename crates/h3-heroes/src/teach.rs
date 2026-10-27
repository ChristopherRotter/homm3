//! What a teacher may teach (E08 T60b; ADR-0013 point 15.3, HER-CHG-004): the hero's side of the
//! objects that teach (a Witch Hut's skill, a University's slots, a Scholar's gift, the Library's
//! condition, the Learning Stone's, a chest's and the Tree of Knowledge's experience; a Shrine's
//! and the Pyramid's spell is [`crate::can_learn_spell`]). The adventure's visits read them and
//! return the changes (ADR-0016 point 4); no object is named here. Survey
//! `docs/rules/surveys/E08-rewards.md` §2.2, §6.2 to §6.8; design §4 (`docs/epics/E08-design.md`,
//! since T60b in `E08-log.md`).

use h3_content::{MAX_HERO_SKILLS, MAX_SKILL_LEVEL, Ruleset, SkillId, SpellId};

use crate::change::HeroChange;
use crate::derive::{gained_experience, join_bonus};
use crate::details::{HeroDetails, PrimarySkill};
use crate::level::experience_for;
use crate::spells::can_learn_spell;

/// Whether a Witch Hut teaches `skill` (HER-CHG-004; survey §6.6): a skill of the ruleset the hero
/// does not know, while it knows fewer than the most skills (eight); no class test. The hut gives
/// it at Basic ([`HeroChange::RaiseSkill`] at level 1). The original's `visit_witch_hut`
/// (0x4A8080).
pub fn hut_teaches(details: &HeroDetails, ruleset: &Ruleset, skill: SkillId) -> bool {
    // rule: HER-CHG-004
    ruleset.skill(skill).is_some() && !knows(details, skill) && has_room(details)
}

/// The state of a University's slot for a hero ([`university_slot`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlotState {
    /// The hero knows the skill.
    Known,
    /// The hero knows the most skills (eight).
    NoRoom,
    /// The hero's class weighs the skill 0 at a level-up (Necromancy for most classes).
    Barred,
    /// The hero may buy the skill: it is learned at Basic.
    Open,
}

/// The state of a University's slot of `skill` for the hero (HER-CHG-004; survey §6.8, D-36):
/// known, no room (eight skills), barred (its class's level-up weight of the skill is 0; a skill
/// the ruleset lacks has none), else open, bought at Basic ([`HeroChange::RaiseSkill`] at level
/// 1; the price is the adventure's). The original's `university_slot_state` (0x5F0790) disables
/// a slot for any of the three; the slots are evaluated again after each purchase.
pub fn university_slot(details: &HeroDetails, ruleset: &Ruleset, skill: SkillId) -> SlotState {
    // rule: HER-CHG-004
    let weight = ruleset
        .hero_type(details.hero_type)
        .and_then(|hero| ruleset.hero_class(hero.class))
        .and_then(|class| class.skill_weights.iter().find(|&&(s, _)| s == skill))
        .map_or(0, |&(_, weight)| weight);
    if knows(details, skill) {
        SlotState::Known
    } else if !has_room(details) {
        SlotState::NoRoom
    } else if weight == 0 {
        SlotState::Barred
    } else {
        SlotState::Open
    }
}

/// What a Scholar holds (survey §6.7): drawn when the game is built for a random one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScholarGift {
    /// +1 to a primary skill.
    Primary(PrimarySkill),
    /// A secondary skill: a level up.
    Skill(SkillId),
    /// A spell.
    Spell(SpellId),
}

/// What a Scholar gives the hero (HER-CHG-004; survey §6.7): a spell it can learn
/// ([`can_learn_spell`] with the known spells counting) as [`HeroChange::LearnSpell`]; a skill a
/// level up, a known one below Expert or a new one at Basic below the most skills, as
/// [`HeroChange::RaiseSkill`] (no class test); a primary skill +1 as [`HeroChange::GainPrimary`]
/// (at most `primary_max`: A-9's class). A spell or skill the hero cannot take gives +1 to
/// `fallback`, the primary skill drawn with it. The original's `visit_scholar` (0x4A4DC0); the
/// Scholar is removed whatever it gave (the adventure's).
pub fn scholar_reward(
    details: &HeroDetails,
    ruleset: &Ruleset,
    gift: ScholarGift,
    fallback: PrimarySkill,
) -> HeroChange {
    // rule: HER-CHG-004
    let taught = match gift {
        ScholarGift::Primary(primary) => return plus_one(primary),
        ScholarGift::Skill(skill) => match details.skills.iter().find(|&&(s, _)| s == skill) {
            Some(&(_, level)) => (level < MAX_SKILL_LEVEL).then_some(level + 1),
            None => hut_teaches(details, ruleset, skill).then_some(1),
        }
        .map(|level| HeroChange::RaiseSkill { skill, level }),
        ScholarGift::Spell(spell) => {
            can_learn_spell(details, ruleset, spell, true).then_some(HeroChange::LearnSpell(spell))
        }
    };
    taught.unwrap_or(plus_one(fallback))
}

/// Whether the Library of Enlightenment admits the hero (HER-CHG-004; survey §6.2): its level plus
/// twice its Diplomacy level (the [`join_bonus`]: Diplomacy's effects are the only ones that give
/// it) at least `level`, the type's (10: level 10, or 8, 6, 4 with Basic, Advanced, Expert
/// Diplomacy). The original's 0x4A3280 (hero +0x55 and +0xCD); a refused hero gets no mark and
/// may come back.
pub fn library_admits(details: &HeroDetails, ruleset: &Ruleset, level: u8) -> bool {
    // rule: HER-CHG-004
    u16::from(details.level) + 2 * u16::from(join_bonus(details, ruleset)) >= u16::from(level)
}

/// The experience a teacher's `amount` gives the hero (HER-CHG-004; survey §2.2, §6.4, §7.4): the
/// amount times its Learning factor, truncated ([`gained_experience`], HER-EXP-001's single
/// computation: `hero_learning` 0x4E4840): the Learning Stone's 1 000, a chest's gold − 500, a
/// reward record's amount. The game gives it as [`HeroChange::GainExperience`].
pub fn learning_experience(details: &HeroDetails, ruleset: &Ruleset, amount: u32) -> u32 {
    // rule: HER-CHG-004, HER-EXP-001
    gained_experience(details, ruleset, amount)
}

/// The Tree of Knowledge's experience (HER-CHG-004; survey §6.4): the gap between the thresholds of
/// the hero's level and the next (HER-EXP-002), without Learning, so the hero gains exactly one
/// level; 0 without the hero parameters. The original's `experience_to_next_level` (0x4DA420).
pub fn next_level_experience(details: &HeroDetails, ruleset: &Ruleset) -> u32 {
    // rule: HER-CHG-004
    ruleset.hero_params().map_or(0, |params| {
        let next = experience_for(params, details.level.saturating_add(1));
        next.saturating_sub(experience_for(params, details.level))
    })
}

/// Whether the hero knows `skill`.
fn knows(details: &HeroDetails, skill: SkillId) -> bool {
    details.skills.iter().any(|&(s, _)| s == skill)
}

/// Whether the hero knows fewer than the most skills (HER-SEC-001).
fn has_room(details: &HeroDetails) -> bool {
    details.skills.len() < MAX_HERO_SKILLS
}

/// +1 to `primary`.
fn plus_one(primary: PrimarySkill) -> HeroChange {
    let mut gains = [0; 4];
    gains[primary.index()] = 1;
    HeroChange::GainPrimary(gains)
}
