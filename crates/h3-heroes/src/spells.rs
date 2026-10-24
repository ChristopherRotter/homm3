//! The spells a hero has and its mastery of a spell (E04b T8a, BAT-Q19): one derivation from the
//! hero's details for every context that shows or uses them, the battle's spell book
//! (HER-BAT-011) as the adventure map's (E08–E10). The original keeps the available spells in the
//! hero record (`+0x430`, written by 0x4D95D0 whenever the artifacts change) and computes a
//! spell's mastery in a function of the hero (0x4E5080); `docs/formats/installation.md` §7.22.

use alloc::collections::BTreeSet;
use alloc::vec::Vec;

use h3_content::{
    HeroEffect, MAX_SKILL_LEVEL, MAX_SPELL_LEVEL, MAX_SPELL_SCHOOLS, Ruleset, SpellId,
};

use crate::change::{HeroChange, check_change};
use crate::derive::{Count, effects};
use crate::details::{HeroDetails, check};
use crate::error::DetailsError;

/// The highest spell level the hero may learn (HER-SPL-005, E08 T19a2b): the hero parameters'
/// `learn_level` (0 without them, which validation refuses) plus the `levels` of its
/// `learn_level` effects (Wisdom: 1, 2, 3; each artifact type once), at most
/// [`MAX_SPELL_LEVEL`]. The original's Wisdom level + 2 (`eagle_eye_learn` 0x469FE0; the mage
/// guilds, E09, ask the same).
pub fn learn_level(details: &HeroDetails, ruleset: &Ruleset) -> u8 {
    // rule: HER-SPL-005
    let base = ruleset.hero_params().map_or(0, |params| params.learn_level);
    let added: u32 = effects(details, ruleset, Count::OncePerType)
        .filter_map(|effect| match *effect {
            HeroEffect::LearnLevel { levels } => Some(u32::from(levels)),
            _ => None,
        })
        .sum();
    let level = u32::from(base)
        .saturating_add(added)
        .min(MAX_SPELL_LEVEL.into());
    u8::try_from(level).unwrap_or(MAX_SPELL_LEVEL)
}

/// Whether the hero learns `spell`, an Eagle Eye pick, after a battle its side won (HER-SPL-006,
/// BAT-Q24; E08 T19a2b): while it wears a spell book (an effect gives one), a hero spell of the
/// ruleset it has not learned ([`HeroChange::LearnSpell`]'s check), of a level at most its
/// [`learn_level`]. The original's `eagle_eye_learn` (0x469FE0): the Spell Book worn and the level
/// at most Wisdom + 2, which batch 2f observed (a winner with its book in the backpack learned
/// nothing; one without Wisdom learned a level 1 spell, not a level 3 one).
pub fn learns_spell(details: &HeroDetails, ruleset: &Ruleset, spell: SpellId) -> bool {
    // rule: HER-SPL-006
    let level = ruleset.spell(spell).map(|s| s.level);
    wears_book(details, ruleset)
        && level.is_some_and(|level| level <= learn_level(details, ruleset))
        && check_change(details, ruleset, HeroChange::LearnSpell(spell)).is_ok()
}

/// The spells two meeting heroes teach each other (HER-SPL-007, E08 T49b; Scholar): `s` is the
/// greater `meeting_spells` level of the two (each hero's highest source; none: nothing); either
/// without a worn spell book (an effect gives one, HER-BAT-011): nothing; else `a` learns each
/// spell `b` has learned (its learned spells, not an artifact's or a scroll's) that it has not,
/// of a level at most the lesser of its learnable level ([`learn_level`], HER-SPL-005) and `s`,
/// in spell order; `b` alike from `a`. The game applies them as
/// [`HeroChange::LearnSpell`] changes. The original's 0x4A2940, at every meeting of two heroes of
/// one player before the exchange screen: each limit the lesser of Wisdom + 2 and the Scholar
/// level + 1, both Spell Books worn, the 70 spells walked by id (the same sets).
pub fn meeting_spells(
    a: &HeroDetails,
    b: &HeroDetails,
    ruleset: &Ruleset,
) -> (Vec<SpellId>, Vec<SpellId>) {
    // rule: HER-SPL-007
    let level = |details: &HeroDetails| {
        effects(details, ruleset, Count::OncePerType)
            .filter_map(|effect| match *effect {
                HeroEffect::MeetingSpells { level } => Some(level),
                _ => None,
            })
            .max()
            .unwrap_or(0)
    };
    let s = level(a).max(level(b));
    let teach = s > 0 && wears_book(a, ruleset) && wears_book(b, ruleset);
    let learns = |learner: &HeroDetails, teacher: &HeroDetails| -> Vec<SpellId> {
        let limit = learn_level(learner, ruleset).min(s);
        (0_u16..)
            .zip(ruleset.spells())
            .filter(|_| teach)
            .filter(|(_, spell)| spell.level <= limit)
            .map(|(raw, _)| SpellId::from_raw(raw))
            .filter(|id| teacher.spells.contains(id) && !learner.spells.contains(id))
            .collect()
    };
    (learns(a, b), learns(b, a))
}

/// Whether the hero wears a spell book: one of its effects gives one (HER-BAT-011; in the
/// original the Spell Book worn, not in the backpack).
fn wears_book(details: &HeroDetails, ruleset: &Ruleset) -> bool {
    effects(details, ruleset, Count::OncePerType)
        .any(|effect| matches!(effect, HeroEffect::SpellBook))
}

/// The hero's available spells, in spell order (HER-SPL-002): the learned spells, the spell of
/// every worn scroll and the spells its `spells` effects give (in the original only artifacts
/// have them; a combination's parts included). A spell book is needed to cast them
/// (HER-BAT-011), not to have them. The details are validated first.
///
/// # Errors
/// The [`DetailsError`] of the first rule the details break ([`crate::validate`]).
pub fn available_spells(
    details: &HeroDetails,
    ruleset: &Ruleset,
) -> Result<Vec<SpellId>, DetailsError> {
    check(details, ruleset)?;
    Ok(available(details, ruleset))
}

/// The hero's mastery of every spell it would cast above none, in spell order (HER-SPL-003):
/// the best of its school masteries among the spell's schools, raised by a `spell_mastery`
/// effect for the spell (Armageddon's Blade), at most Expert; on ordinary ground (a battlefield's
/// raise is the battle's, E04c). Spells it does not have are listed too: the original asks the
/// hero for any spell's mastery (Sacrifice's test asks for Resurrection's, BAT-QUIRK-5). The
/// details are validated first.
///
/// # Errors
/// The [`DetailsError`] of the first rule the details break ([`crate::validate`]).
pub fn spell_masteries(
    details: &HeroDetails,
    ruleset: &Ruleset,
) -> Result<Vec<(SpellId, u8)>, DetailsError> {
    check(details, ruleset)?;
    Ok(masteries(details, ruleset))
}

/// [`available_spells`] of validated details.
pub(crate) fn available(details: &HeroDetails, ruleset: &Ruleset) -> Vec<SpellId> {
    // rule: HER-SPL-002
    let mut set: BTreeSet<SpellId> = details.spells.iter().copied().collect();
    set.extend(details.worn.iter().flatten().filter_map(|item| item.spell));
    for effect in effects(details, ruleset, Count::OncePerType) {
        if let HeroEffect::Spells { grant } = *effect {
            set.extend(all_spells(ruleset).filter(|&s| grant.gives(ruleset, s)));
        }
    }
    set.into_iter().collect()
}

/// [`spell_masteries`] of validated details.
pub(crate) fn masteries(details: &HeroDetails, ruleset: &Ruleset) -> Vec<(SpellId, u8)> {
    // rule: HER-SPL-003
    let mut schools = [0u8; MAX_SPELL_SCHOOLS];
    let mut raised: Vec<(SpellId, u8)> = Vec::new();
    for effect in effects(details, ruleset, Count::OncePerType) {
        match *effect {
            HeroEffect::SchoolMastery { school, level } => {
                // Content has at most `MAX_SPELL_SCHOOLS` schools.
                let mastery = &mut schools[school.index().min(MAX_SPELL_SCHOOLS - 1)];
                *mastery = (*mastery).max(level);
            }
            HeroEffect::SpellMastery { spell, level } => raised.push((spell, level)),
            _ => {}
        }
    }
    all_spells(ruleset)
        .zip(ruleset.spells())
        .filter_map(|(id, spell)| {
            let by_school = spell
                .schools
                .iter()
                .map(|l| schools[l.index().min(MAX_SPELL_SCHOOLS - 1)]);
            let by_artifact = raised
                .iter()
                .filter(|&&(s, _)| s == id)
                .map(|&(_, level)| level);
            let mastery = by_school
                .chain(by_artifact)
                .max()
                .unwrap_or(0)
                .min(MAX_SKILL_LEVEL);
            (mastery > 0).then_some((id, mastery))
        })
        .collect()
}

/// Every spell of the ruleset, by handle.
fn all_spells(ruleset: &Ruleset) -> impl Iterator<Item = SpellId> {
    (0..ruleset.spells().len()).filter_map(SpellId::from_index)
}
