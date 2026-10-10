//! The spells a hero has and its mastery of a spell (E04b T8a, BAT-Q19): one derivation from the
//! hero's details for every context that shows or uses them, the battle's spell book
//! (HER-BAT-011) as the adventure map's (E08–E10). The original keeps the available spells in the
//! hero record (`+0x430`, written by 0x4D95D0 whenever the artifacts change) and computes a
//! spell's mastery in a function of the hero (0x4E5080); `docs/formats/installation.md` §7.22.

use alloc::collections::BTreeSet;
use alloc::vec::Vec;

use h3_content::{HeroEffect, MAX_SKILL_LEVEL, MAX_SPELL_SCHOOLS, Ruleset, SpellId};

use crate::derive::{Count, artifact_effects, effects};
use crate::details::{HeroDetails, check};
use crate::error::DetailsError;

/// The hero's available spells, in spell order (HER-SPL-002): the learned spells, the spell of
/// every worn scroll and the spells its worn artifacts give (a combination's parts included). A
/// spell book is needed to cast them (HER-BAT-011), not to have them. The details are validated
/// first.
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
    set.extend(details.worn.iter().filter_map(|item| item.spell));
    for effect in artifact_effects(details, ruleset, Count::OncePerType) {
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
