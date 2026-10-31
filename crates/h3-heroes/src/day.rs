//! What a hero brings to the day's start (E08 T82; ADR-0013 point 16; survey
//! `docs/rules/surveys/E08-economy.md` §8, §12): its income (GAM-RES-004) and its day's spell
//! points (HER-MAN-002). Pure functions of the details and the core ruleset; when the game calls
//! them and what it does with the result is the game's (ADR-0017 point 3: the income paid to the
//! owner's treasury, the points applied as a hero change). No artifact or skill is read by name
//! (ADR-0016 point 4): the effect kinds `daily_resource`, `mana_regain`, `extra_mana_regain` and
//! `full_mana`, a skill specialty's growth, the hero parameters' `base_mana_regain` and
//! `regain_specialist_bonus`.

use alloc::vec;
use alloc::vec::Vec;

use h3_content::{CompiledHeroEffect, HeroEffect, Ruleset};

use crate::derive::{
    Count, Growth, artifact_effects, effects, grown_skill_effects, max_mana, specialty,
};
use crate::details::HeroDetails;

/// A hero's income a day (GAM-RES-004, E08 T82; ADR-0013 point 16.1): by resource, dense in the
/// core's order (a treasury's layout; one entry per resource of `core`), the `daily_resource`
/// amounts of
///
/// - every artifact in a worn position and every part of a worn combination, **each copy
///   counting** (the original counts worn slots, `count_worn_artifact` 0x4BABD0: two Rings of
///   Infinite Gems give 2 gems; the Cornucopia's 4 and its parts' 1 make the original's 5), one
///   that needs a skill only with it;
/// - each secondary skill's level, grown when the hero has a skill specialty on that skill by
///   `⌊amount × (10 000 + bp_per_level × level) ÷ 10 000⌋` (Estates 125, 250, 500 gold; its
///   specialists' 5 % of itself a level, `hero_gold_income` 0x4E43DA);
/// - its specialty (the resource specialists' 1, the gold specialists' 350).
///
/// Exact integers in every profile, no Q-0005 term: the original keeps the factor
/// `1 + 0.05 × level` in the x87 register (only the 0.05 is a single, 0x63EAE4), and its excess
/// never crosses an integer the exact product does not reach (the products are multiples of
/// 0.05). Saturating per resource; a resource the core lacks counts for nothing. The mines', the
/// towns' and the creatures' parts are the contexts' (ADR-0017 point 10).
pub fn daily_income(details: &HeroDetails, core: &Ruleset) -> Vec<u32> {
    // rule: GAM-RES-004
    let mut income = vec![0_u32; core.resources().len()];
    let mut add = |effect: &CompiledHeroEffect, growth: Growth| {
        if let HeroEffect::DailyResource { resource, amount } = *effect
            && let Some(entry) = income.get_mut(usize::from(resource.raw()))
        {
            let amount = u32::try_from(growth.grow(amount.into())).unwrap_or(u32::MAX);
            *entry = entry.saturating_add(amount);
        }
    };
    for (effect, growth) in grown_skill_effects(details, core) {
        add(effect, growth);
    }
    let others = artifact_effects(details, core, Count::PerCopy).chain(specialty(details, core));
    for effect in others {
        add(effect, Growth::NONE);
    }
    income
}

/// A hero's spell points after the day's regain (HER-MAN-002, E08 T82; ADR-0013 point 16.2),
/// from its points and its maximum (HER-MAN-001, [`max_mana`]):
///
/// - with a `full_mana` effect (the Wizard's Well, worn or inside a worn combination) the
///   maximum when its points are below it;
/// - else its points plus the day's regain, at most the maximum;
/// - never lowered: points above the maximum stay.
///
/// The regain: the highest `mana_regain` of the hero's sources (Mysticism 2, 3, 4), else the hero
/// parameters' `base_mana_regain` (1); a skill's grown, when the hero has a skill specialty on
/// it, as [`daily_income`]'s and rounded down, plus the parameters' `regain_specialist_bonus`
/// (`hero_mana_regained` 0x4E3F5F: `trunc(regain × (1 + 0.05 × level)) + 1`; nothing for a
/// specialist without the skill, whose regain is the base); a skill wins a tie with another
/// source. Then each worn artifact type's `extra_mana_regain` once (the Charm, the Talisman and
/// the Mystic Orb of Mana 1, 2, 3; worn or a part of a worn combination, the original's
/// `hero_wears_artifact` 0x4D91F0). Exact integers in every profile (point 16.2: grown as
/// 16.1's). Every source counts once per type for `full_mana` and `extra_mana_regain`, as the
/// movement's kinds (the packs give them to artifacts alone). Saturating.
pub fn day_mana(details: &HeroDetails, core: &Ruleset) -> u16 {
    // rule: HER-MAN-002
    let max = max_mana(details, core);
    let points = details.mana;
    let mut full = false;
    let mut extra = 0_u16;
    let mut regain: Option<(u16, Growth)> = None;
    let mut highest = |points: u16, growth: Growth| {
        if regain.is_none_or(|(best, _)| points > best) {
            regain = Some((points, growth));
        }
    };
    for (effect, growth) in grown_skill_effects(details, core) {
        if let HeroEffect::ManaRegain { points } = *effect {
            highest(points, growth);
        }
    }
    let others =
        artifact_effects(details, core, Count::OncePerType).chain(specialty(details, core));
    for effect in others {
        if let HeroEffect::ManaRegain { points } = *effect {
            highest(points, Growth::NONE);
        }
    }
    for effect in effects(details, core, Count::OncePerType) {
        match *effect {
            HeroEffect::FullMana => full = true,
            HeroEffect::ExtraManaRegain { points } => extra = extra.saturating_add(points),
            _ => {}
        }
    }
    if full {
        return points.max(max);
    }
    let params = core.hero_params();
    let regain = match regain {
        Some((points, growth)) if growth.grows() => {
            let bonus = params.map_or(0, |p| p.regain_specialist_bonus);
            u16::try_from(growth.grow(points.into()))
                .unwrap_or(u16::MAX)
                .saturating_add(bonus)
        }
        Some((points, _)) => points,
        None => params.map_or(0, |p| p.base_mana_regain),
    };
    let regained = points.saturating_add(regain).saturating_add(extra);
    points.max(regained.min(max))
}
