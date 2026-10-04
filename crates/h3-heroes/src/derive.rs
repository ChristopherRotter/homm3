//! The translation from hero details to the battle hero (ADR-0013 point 2; HER-BAT-001,
//! HER-BAT-002).

use h3_content::{CompiledHeroEffect, HeroEffect, Ruleset};
use h3_contracts::BattleHero;

use crate::details::{HeroDetails, PrimarySkill, check};
use crate::error::DetailsError;

/// Derives what a battle needs to know about the hero (`BattleSetup.heroes`): the hero type, the
/// effective primary skills (HER-BAT-001) and the Tactics level (HER-BAT-002); the mana is passed
/// through. The details are validated first.
///
/// Later tasks fill the rest; until then it stays neutral: damage bonuses (E04a T6), war-machine
/// control (T7), surrender discount (T9), the effects list (T5, T10), and spell book, maximum
/// mana and schools (E04b).
///
/// # Errors
/// The [`DetailsError`] of the first rule the details break ([`crate::validate`]).
pub fn derive_battle_hero(
    details: &HeroDetails,
    ruleset: &Ruleset,
) -> Result<BattleHero, DetailsError> {
    let params = check(details, ruleset)?;
    let mut hero = BattleHero::new(details.hero_type);

    // rule: HER-BAT-001
    let mut total = details.primary.map(i32::from);
    for effect in effects(details, ruleset) {
        if let HeroEffect::PrimarySkills {
            attack,
            defense,
            power,
            knowledge,
        } = *effect
        {
            for (t, change) in total.iter_mut().zip([attack, defense, power, knowledge]) {
                *t = t.saturating_add(i32::from(change));
            }
        }
    }
    let effective = |skill: PrimarySkill| {
        let min = params.primary_min[skill.index()];
        let value = total[skill.index()].clamp(i32::from(min), i32::from(params.primary_max));
        u8::try_from(value).unwrap_or(params.primary_max)
    };
    hero.attack = effective(PrimarySkill::Attack);
    hero.defense = effective(PrimarySkill::Defense);
    hero.power = effective(PrimarySkill::Power);
    hero.knowledge = effective(PrimarySkill::Knowledge);

    // rule: HER-BAT-002
    hero.tactics = effects(details, ruleset)
        .filter_map(|effect| match *effect {
            HeroEffect::Tactics { level } => Some(level),
            _ => None,
        })
        .max()
        .unwrap_or(0);

    hero.magic.mana = details.mana;
    Ok(hero)
}

/// Every effect the hero's details bring into a battle: its secondary skills at their levels,
/// its worn artifacts — one entry per worn copy, a combination together with its parts, an
/// artifact that needs a skill only while the hero has that skill at any level — and its
/// specialty. Effects of kinds that count once per artifact type appear once per copy here;
/// their consumers count them (E04a T10). References the ruleset does not know are skipped
/// (the details are validated before).
fn effects<'a>(
    details: &'a HeroDetails,
    ruleset: &'a Ruleset,
) -> impl Iterator<Item = &'a CompiledHeroEffect> {
    let skills = details.skills.iter().flat_map(|&(id, level)| {
        ruleset
            .skill(id)
            .map(|skill| skill.effects(level))
            .unwrap_or_default()
    });
    let artifacts = details
        .worn
        .iter()
        .filter_map(|&id| ruleset.artifact(id))
        .flat_map(|artifact| {
            core::iter::once(artifact).chain(
                artifact
                    .parts
                    .iter()
                    .filter_map(|&part| ruleset.artifact(part)),
            )
        })
        .filter(|artifact| {
            artifact
                .needs_skill
                .is_none_or(|needed| details.skills.iter().any(|&(id, _)| id == needed))
        })
        .flat_map(|artifact| artifact.effects.iter());
    let specialty = ruleset
        .hero_type(details.hero_type)
        .map(|hero_type| hero_type.specialty.as_slice())
        .unwrap_or_default();
    skills.chain(artifacts).chain(specialty)
}
