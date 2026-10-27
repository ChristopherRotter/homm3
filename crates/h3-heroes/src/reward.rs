//! A reward record's hero half (E08 T60b; ADR-0013 points 15.2 and 15.4, HER-CHG-003): what a
//! Pandora's Box or an event gives the hero, in the original's order, as the changes the game
//! applies, and whether a hero can receive an artifact at all (A-25, `ADV-QUIRK-22`). Survey
//! `docs/rules/surveys/E08-rewards.md` §7.4, §8.2, §15; design §3 (`docs/epics/E08-design.md`,
//! since T60b in `E08-log.md`). The record's resources and creatures are the adventure's, its
//! experience the game's to give last (ADR-0016 point 4, ADR-0017 point 6).

use alloc::vec::Vec;

use h3_content::{ArtifactTypeId, FULL_BACKPACK_REFUSES, Ruleset, SkillId, SpellId};

use crate::change::{HeroChange, apply_change, check_change, has_space};
use crate::details::{ArtifactInstance, HeroDetails};
use crate::spells::can_learn_spell;
use crate::teach::learning_experience;

/// The hero's half of a reward record (HER-CHG-003; ADR-0013 point 15.2): the map's values as it
/// stores them, every part optional (0 or empty gives nothing).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HeroReward {
    /// The experience amount before Learning: the map's u32, which the original reads as signed,
    /// so 2³¹ and above give none (survey §7.4).
    pub experience: u32,
    /// The primary skills in [`crate::PrimarySkill`] order: only a value above 0 is added.
    pub primary: [i8; 4],
    /// Secondary skills and their levels, in the record's order: each raised to its level.
    pub skills: Vec<(SkillId, u8)>,
    /// The spell points added: negative a loss.
    pub mana: i32,
    /// The morale added to the battle bonuses (HER-ADV-008).
    pub morale: i8,
    /// The luck added to the battle bonuses.
    pub luck: i8,
    /// The artifacts, in the record's order.
    pub artifacts: Vec<ArtifactTypeId>,
    /// The spells, in the record's order.
    pub spells: Vec<SpellId>,
}

/// What a reward record gives the hero ([`grant`]).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Granted {
    /// The changes, in order: each passes [`check_change`] once the earlier ones are applied.
    pub changes: Vec<HeroChange>,
    /// The experience the game gives last, after the creatures and their army dialog
    /// ([`HeroChange::GainExperience`] under the scenario's level limit): the amount after the
    /// hero's Learning as it was before the record's skills (D-39).
    pub experience: u32,
    /// The record's artifacts the hero could not receive ([`can_receive_artifact`]), lost.
    pub lost: Vec<ArtifactTypeId>,
    /// The record's spells the hero did not learn ([`can_learn_spell`]), skipped without a word.
    pub skipped: Vec<SpellId>,
}

/// The hero's half of a reward record (HER-CHG-003; ADR-0013 point 15.2), each part tested
/// against the details as the earlier parts leave them, in the original's order
/// (`grant_rewards` 0x49FA90, survey §7.4):
///
/// 1. the experience amount, if it is above 0 as a signed value, times the hero's Learning factor
///    ([`learning_experience`]) with its Learning **before** the record's skills (D-39); returned
///    in [`Granted::experience`] for the game to give last, not as a change;
/// 2. [`HeroChange::GainPrimary`] of the values above 0 (at most `primary_max`: A-9's class);
/// 3. [`HeroChange::RaiseSkill`] per pair that raises a skill: a known one below its level rises
///    to it, never falls (D-37); a new one only below the most skills (eight); no class or ban
///    test; a pair the ruleset refuses (no such skill, a level not Basic to Expert) gives nothing;
/// 4. [`HeroChange::SetSpellPoints`] of the points plus the record's (when not 0), held between 0
///    and the ruleset's mana ceiling (999): above the hero's maximum, and a hero above the
///    ceiling falls to it (D-38);
/// 5. [`HeroChange::Bonus`] of the morale and luck, without a once-mark, when either is not 0;
/// 6. [`HeroChange::AddArtifact`] per artifact the hero can receive ([`can_receive_artifact`]),
///    else it is lost ([`Granted::lost`]); an artifact the ruleset refuses as received (one it
///    lacks, a Spell Scroll without a spell: HER-ART-003) is lost too;
/// 7. [`HeroChange::LearnSpell`] per spell the hero can learn with a spell book step 6 may have
///    given, up to the learnable level step 3's Wisdom may have raised, not known
///    ([`can_learn_spell`]; the map's bans not tested), else it is skipped ([`Granted::skipped`]).
///
/// A combination a received part completes asks nothing here: the player assembles it with a
/// command (HER-ART-006). Pure and total; an empty record grants nothing.
pub fn grant(details: &HeroDetails, ruleset: &Ruleset, reward: &HeroReward) -> Granted {
    // rule: HER-CHG-003
    // D-39: the amount with the Learning the hero has before this record's skills.
    let signed = i32::try_from(reward.experience).is_ok_and(|amount| amount > 0);
    let experience = if signed {
        learning_experience(details, ruleset, reward.experience)
    } else {
        0
    };
    let mut scratch = Scratch {
        details: details.clone(),
        ruleset,
        changes: Vec::new(),
    };
    let gains = reward.primary.map(|value| u8::try_from(value).unwrap_or(0));
    if gains.iter().any(|&gain| gain > 0) {
        scratch.give(HeroChange::GainPrimary(gains));
    }
    for &(skill, level) in &reward.skills {
        // D-37: raised, never lowered; one at or above the level stays as it is.
        let known = scratch.details.skills.iter().find(|&&(s, _)| s == skill);
        if known.is_none_or(|&(_, known)| known < level) {
            scratch.give(HeroChange::RaiseSkill { skill, level });
        }
    }
    if reward.mana != 0 {
        // D-38: held in 0 to the ceiling, whatever the hero's maximum.
        let ceiling = ruleset
            .hero_params()
            .map_or(0, |params| params.mana_ceiling);
        let points =
            (i64::from(scratch.details.mana) + i64::from(reward.mana)).clamp(0, i64::from(ceiling));
        scratch.give(HeroChange::SetSpellPoints(
            u16::try_from(points).unwrap_or(ceiling),
        ));
    }
    if reward.morale != 0 || reward.luck != 0 {
        scratch.give(HeroChange::Bonus {
            morale: reward.morale,
            luck: reward.luck,
            mark: None,
        });
    }
    let mut lost = Vec::new();
    for &artifact in &reward.artifacts {
        let item = ArtifactInstance::from(artifact);
        if can_receive_artifact(&scratch.details, ruleset, item) {
            scratch.give(HeroChange::AddArtifact(item));
        } else {
            lost.push(artifact);
        }
    }
    let mut skipped = Vec::new();
    for &spell in &reward.spells {
        if can_learn_spell(&scratch.details, ruleset, spell, true) {
            scratch.give(HeroChange::LearnSpell(spell));
        } else {
            skipped.push(spell);
        }
    }
    Granted {
        changes: scratch.changes,
        experience,
        lost,
        skipped,
    }
}

/// Whether the hero can receive `item` (HER-CHG-003; ADR-0013 point 15.4): the received artifact
/// finds room ([`HeroChange::AddArtifact`]'s check, HER-CHG-002: the first position that takes
/// it, else the backpack below its most), and with `FULL_BACKPACK_REFUSES` (A-25,
/// `ADV-QUIRK-22`, on in `faithful`) only while the backpack holds fewer than its most (64),
/// whatever the free positions: the original's visits test the count before
/// `hero_give_artifact` places anything (survey §2.2, §5, §6.9, §7.4; batch E7's E7-5, E7-6,
/// E7-10, E7-19). The caller loses or converts a refused artifact (a chest's 1 000 gold, a
/// Corpse's gold, a Wagon's resources).
pub fn can_receive_artifact(
    details: &HeroDetails,
    ruleset: &Ruleset,
    item: ArtifactInstance,
) -> bool {
    // rule: HER-CHG-003
    // ADV-QUIRK-22 (A-25): the backpack's count first, whatever the free positions.
    let counted =
        !ruleset.quirks().is_enabled(FULL_BACKPACK_REFUSES) || has_space(details, ruleset);
    counted && check_change(details, ruleset, HeroChange::AddArtifact(item)).is_ok()
}

/// The details as a grant's earlier parts leave them, and the changes given so far.
struct Scratch<'a> {
    details: HeroDetails,
    ruleset: &'a Ruleset,
    changes: Vec<HeroChange>,
}

impl Scratch<'_> {
    /// Gives `change` if it passes its check: applied to the scratch details and recorded.
    fn give(&mut self, change: HeroChange) {
        if check_change(&self.details, self.ruleset, change).is_ok() {
            apply_change(&mut self.details, self.ruleset, change);
            self.changes.push(change);
        }
    }
}
