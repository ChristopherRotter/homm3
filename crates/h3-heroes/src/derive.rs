//! The translation from hero details to the battle hero (ADR-0013 point 2; HER-BAT-001 to
//! HER-BAT-005).

use alloc::collections::BTreeSet;
use alloc::vec::Vec;

use h3_content::{Affects, CompiledHeroEffect, FULL_BP, HeroEffect, Ruleset};
use h3_contracts::{
    BallistaControl, BattleEffect, BattleHero, DamageBonuses, MachineControl, TentControl,
};

use crate::details::{HeroDetails, PrimarySkill, check};
use crate::error::DetailsError;

/// Derives what a battle needs to know about the hero (`BattleSetup.heroes`): the hero type, the
/// effective primary skills (HER-BAT-001), the Tactics level (HER-BAT-002), the morale and luck
/// effects (HER-BAT-003), the damage bonuses (HER-BAT-004) and the war machines it commands
/// (HER-BAT-005); the mana is passed through. The details are validated first.
///
/// Later tasks fill the rest; until then it stays neutral: surrender discount (E04a T9), the
/// other effects (T10), and spell book, maximum mana and schools (E04b).
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
    for effect in effects(details, ruleset, Count::PerCopy) {
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
    hero.tactics = effects(details, ruleset, Count::PerCopy)
        .filter_map(|effect| match *effect {
            HeroEffect::Tactics { level } => Some(level),
            _ => None,
        })
        .max()
        .unwrap_or(0);

    // rule: HER-BAT-003
    hero.effects = morale_and_luck(effects(details, ruleset, Count::OncePerType));

    // rule: HER-BAT-004
    hero.damage = damage_bonuses(details, ruleset);

    // rule: HER-BAT-005
    hero.war_machines = machine_control(details, ruleset);

    hero.magic.mana = details.mana;
    Ok(hero)
}

/// How often a worn artifact's effects count.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Count {
    /// Once per worn copy (and per combination containing it).
    PerCopy,
    /// Once per artifact type, however many copies are worn.
    OncePerType,
}

/// The morale and luck effects among `effects`, combined per kind and affected stacks: the
/// changes summed (saturated to `i8`; a sum of 0 is left out), the caps the smallest. In the
/// order morale, luck, morale cap, luck cap, each for the own, the enemy and both sides
/// (HER-BAT-003).
fn morale_and_luck<'a>(effects: impl Iterator<Item = &'a CompiledHeroEffect>) -> Vec<BattleEffect> {
    // [morale, luck, morale cap, luck cap][own, enemy, both]
    let mut totals: [[Option<i32>; 3]; 4] = [[None; 3]; 4];
    for effect in effects {
        let (kind, value, affects, cap) = match *effect {
            HeroEffect::Morale { amount, affects } => (0, amount, affects, false),
            HeroEffect::Luck { amount, affects } => (1, amount, affects, false),
            HeroEffect::MoraleAtMost { value, affects } => (2, value, affects, true),
            HeroEffect::LuckAtMost { value, affects } => (3, value, affects, true),
            _ => continue,
        };
        let value = i32::from(value);
        let total = &mut totals[kind][affects_index(affects)];
        *total = Some(match *total {
            None => value,
            Some(t) if cap => t.min(value),
            Some(t) => t + value,
        });
    }
    let mut out = Vec::new();
    for (kind, by_affects) in totals.iter().enumerate() {
        for (&total, affects) in by_affects.iter().zip(Affects::ALL) {
            let Some(total) = total.filter(|&t| kind >= 2 || t != 0) else {
                continue;
            };
            // Within `i8` after the clamp.
            let value = i8::try_from(total.clamp(-128, 127)).unwrap_or_default();
            out.push(match kind {
                0 => BattleEffect::Morale {
                    amount: value,
                    affects,
                },
                1 => BattleEffect::Luck {
                    amount: value,
                    affects,
                },
                2 => BattleEffect::MoraleAtMost { value, affects },
                _ => BattleEffect::LuckAtMost { value, affects },
            });
        }
    }
    out
}

/// Index of `affects` in [`Affects::ALL`].
const fn affects_index(affects: Affects) -> usize {
    match affects {
        Affects::Own => 0,
        Affects::Enemy => 1,
        Affects::Both => 2,
    }
}

/// Every effect with the growth its value gets (HER-BAT-004, HER-BAT-005): the secondary skills'
/// effects at their levels, each grown by the hero's specialties for that skill by
/// `bp_per_level` basis points of itself per hero level; the worn artifacts' (each artifact type
/// once) and the specialty's own, not grown.
fn grown_effects<'a>(
    details: &'a HeroDetails,
    ruleset: &'a Ruleset,
) -> impl Iterator<Item = (&'a CompiledHeroEffect, u64)> {
    let specialty = specialty(details, ruleset);
    let skills = details.skills.iter().flat_map(move |&(id, level)| {
        let growth_bp: u64 = specialty
            .iter()
            .filter_map(|effect| match *effect {
                HeroEffect::SkillSpecialty {
                    skill,
                    bp_per_level,
                } if skill == id => Some(u64::from(bp_per_level) * u64::from(details.level)),
                _ => None,
            })
            .sum();
        ruleset
            .skill(id)
            .map(|skill| skill.effects(level))
            .unwrap_or_default()
            .iter()
            .map(move |effect| (effect, growth_bp))
    });
    let others = artifact_effects(details, ruleset, Count::OncePerType).chain(specialty);
    skills.chain(others.map(|effect| (effect, 0)))
}

/// `bp` grown by `growth_bp` basis points of itself, rounded down.
fn grow(bp: u64, growth_bp: u64) -> u64 {
    bp.saturating_add(bp.saturating_mul(growth_bp) / u64::from(FULL_BP))
}

/// The damage bonuses (HER-BAT-004): per kind (ranged, melee, reduction) the sum of the grown
/// values of [`grown_effects`]. Sums saturate at `u16::MAX`; the reduction is at most 10 000.
fn damage_bonuses(details: &HeroDetails, ruleset: &Ruleset) -> DamageBonuses {
    // [ranged, melee, reduction]
    let mut sums = [0u64; 3];
    for (effect, growth_bp) in grown_effects(details, ruleset) {
        let (kind, bp) = match *effect {
            HeroEffect::RangedDamage { bp } => (0, bp),
            HeroEffect::MeleeDamage { bp } => (1, bp),
            HeroEffect::DamageReduction { bp } => (2, bp),
            _ => continue,
        };
        sums[kind] = sums[kind].saturating_add(grow(u64::from(bp), growth_bp));
    }
    let bp = |sum: u64| u16::try_from(sum).unwrap_or(u16::MAX);
    DamageBonuses {
        ranged_bp: bp(sums[0]),
        melee_bp: bp(sums[1]),
        reduction_bp: bp(sums[2]).min(FULL_BP),
    }
}

/// The war machines the hero commands (HER-BAT-005): the Ballista if an effect gives Artillery,
/// with the most shots and the highest double-damage chance among such effects (skill
/// specialties do not grow them: the original has no Artillery specialist and keeps no
/// Artillery value its specialty getter could scale, `docs/formats/installation.md` §7.4); the
/// First Aid Tent if an effect gives First Aid, its heal bonus the sum of the grown `heal_bp`
/// values of [`grown_effects`] (a First Aid specialist's grows by 5 % per level), saturated.
fn machine_control(details: &HeroDetails, ruleset: &Ruleset) -> MachineControl {
    let mut control = MachineControl::default();
    for (effect, growth_bp) in grown_effects(details, ruleset) {
        match *effect {
            HeroEffect::Artillery {
                shots,
                double_damage_chance_bp,
            } => {
                let ballista = control.ballista.get_or_insert(BallistaControl {
                    shots,
                    double_damage_chance_bp,
                });
                ballista.shots = ballista.shots.max(shots);
                ballista.double_damage_chance_bp = ballista
                    .double_damage_chance_bp
                    .max(double_damage_chance_bp);
            }
            HeroEffect::FirstAid { heal_bp } => {
                let tent = control.tent.get_or_insert(TentControl { heal_bonus_bp: 0 });
                let grown = u32::try_from(grow(u64::from(heal_bp), growth_bp)).unwrap_or(u32::MAX);
                tent.heal_bonus_bp = tent.heal_bonus_bp.saturating_add(grown);
            }
            _ => {}
        }
    }
    control
}

/// Every effect the hero's details bring into a battle: its secondary skills at their levels,
/// its worn artifacts ([`artifact_effects`]) and its specialty. References the ruleset does not
/// know are skipped (the details are validated before).
fn effects<'a>(
    details: &'a HeroDetails,
    ruleset: &'a Ruleset,
    count: Count,
) -> impl Iterator<Item = &'a CompiledHeroEffect> {
    let skills = details.skills.iter().flat_map(|&(id, level)| {
        ruleset
            .skill(id)
            .map(|skill| skill.effects(level))
            .unwrap_or_default()
    });
    skills
        .chain(artifact_effects(details, ruleset, count))
        .chain(specialty(details, ruleset))
}

/// The effects of the worn artifacts: a combination together with its parts, an artifact that
/// needs a skill only while the hero has that skill at any level, each artifact once per worn
/// copy or once per type as `count` says.
fn artifact_effects<'a>(
    details: &'a HeroDetails,
    ruleset: &'a Ruleset,
    count: Count,
) -> impl Iterator<Item = &'a CompiledHeroEffect> {
    let mut seen = BTreeSet::new();
    details
        .worn
        .iter()
        .flat_map(|&id| {
            let parts = ruleset.artifact(id).map(|a| a.parts.as_slice());
            core::iter::once(id).chain(parts.unwrap_or_default().iter().copied())
        })
        .filter(move |&id| count == Count::PerCopy || seen.insert(id.raw()))
        .filter_map(|id| ruleset.artifact(id))
        .filter(|artifact| {
            artifact
                .needs_skill
                .is_none_or(|needed| details.skills.iter().any(|&(id, _)| id == needed))
        })
        .flat_map(|artifact| artifact.effects.iter())
}

/// The specialty of the hero's type.
fn specialty<'a>(details: &HeroDetails, ruleset: &'a Ruleset) -> &'a [CompiledHeroEffect] {
    ruleset
        .hero_type(details.hero_type)
        .map(|hero_type| hero_type.specialty.as_slice())
        .unwrap_or_default()
}
