//! The translation from hero details to the battle hero (ADR-0013 point 2; HER-BAT-001 to
//! HER-BAT-017).

use alloc::collections::BTreeSet;
use alloc::vec::Vec;
use core::num::NonZeroU32;

use h3_content::{
    Affects, ArtifactType, ArtifactTypeId, CompiledHeroEffect, CreatureSelector, CreatureStat,
    CreatureTypeId, FULL_BP, HeroEffect, HeroParams, RAISED_LEVELS, RaisedCreature, Ruleset,
    SINGLE_PRECISION_RATES, SchoolId, SkillId, SpellCategory, SpellId, SpellScope,
};
use h3_contracts::{
    BallistaControl, BattleEffect, BattleHero, BattleMagic, CatapultControl, DamageBonuses,
    EagleEye, FactionGroup, MAX_BATTLE_EFFECTS, MachineControl, Rate, StatOp, TentControl,
};
use h3_kernel::arith::{SINGLE_BITS, X87, mul_div_floor};

use crate::details::{HeroDetails, PrimarySkill, check};
use crate::error::DetailsError;
use crate::spells;

/// Derives what a battle needs to know about the hero (`BattleSetup.heroes`): the hero type, the
/// effective primary skills (HER-BAT-001), the Tactics level (HER-BAT-002), the morale and luck
/// effects (HER-BAT-003), the damage bonuses (HER-BAT-004), the war machines it commands
/// (HER-BAT-005), the `NoRetreat` effect (HER-BAT-006), the surrender discount (HER-BAT-007),
/// the stack statistic effects (HER-BAT-008), the stack trait effects (HER-BAT-009), the
/// alignment groups (HER-BAT-010), the magic (HER-BAT-011: the spell book, HER-SPL-002, the
/// spell masteries, HER-SPL-003, and Eagle Eye, HER-SPL-004; the mana passed through), the spell
/// bonuses (HER-BAT-012), the magic resistance (HER-BAT-013), the spell duration bonus
/// (HER-BAT-014), the casting limits (HER-BAT-015), the spell defences (HER-BAT-016) and the
/// opening spells (HER-BAT-017). The details are validated first.
///
/// The maximum mana stays 0 (informative; it is derived with hero progression, E08–E10).
///
/// # Errors
/// The [`DetailsError`] of the first rule the details break ([`crate::validate`]);
/// [`DetailsError::TooManyBattleEffects`] if the effects exceed [`MAX_BATTLE_EFFECTS`].
pub fn derive_battle_hero(
    details: &HeroDetails,
    ruleset: &Ruleset,
) -> Result<BattleHero, DetailsError> {
    let params = check(details, ruleset)?;
    let mut hero = BattleHero::new(details.hero_type);

    // rule: HER-BAT-001
    let effective = |skill: PrimarySkill| effective_primary(details, ruleset, params, skill);
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

    // rule: HER-BAT-006
    if effects(details, ruleset, Count::OncePerType).any(|e| matches!(e, HeroEffect::NoRetreat)) {
        hero.effects.push(BattleEffect::NoRetreat);
    }

    // rule: HER-BAT-008
    hero.effects.extend(stack_stats(details, ruleset));

    // rule: HER-BAT-009
    hero.effects.extend(stack_traits(details, ruleset));

    // rule: HER-BAT-010
    hero.effects.extend(alignment_groups(details, ruleset));

    // rule: HER-BAT-012
    hero.effects.extend(spell_bonuses(details, ruleset));

    // rule: HER-BAT-013
    hero.effects.extend(magic_resistance(details, ruleset));

    // rule: HER-BAT-014
    hero.effects.extend(spell_duration(details, ruleset));

    // rule: HER-BAT-015
    hero.effects.extend(casting_limits(details, ruleset));

    // rule: HER-BAT-016
    hero.effects.extend(spell_defences(details, ruleset));

    // rule: HER-BAT-017
    hero.effects.extend(opening_spells(details, ruleset));
    if hero.effects.len() > MAX_BATTLE_EFFECTS {
        return Err(DetailsError::TooManyBattleEffects);
    }

    // rule: HER-BAT-004
    hero.damage = damage_bonuses(details, ruleset);

    // rule: HER-BAT-005
    hero.war_machines = machine_control(details, ruleset);

    // rule: HER-BAT-007
    hero.surrender_discount_bp = surrender_discount(details, ruleset);

    // rule: HER-BAT-011
    hero.magic = magic(details, ruleset);
    Ok(hero)
}

/// The hero's magic (HER-BAT-011): its available spells ([`crate::available_spells`]) as its
/// spell book if one of its effects gives a spell book (each artifact type once), its spell
/// masteries ([`crate::spell_masteries`]), its Eagle Eye ([`eagle_eye`]), the mana as given.
fn magic(details: &HeroDetails, ruleset: &Ruleset) -> BattleMagic {
    let book = effects(details, ruleset, Count::OncePerType)
        .any(|effect| matches!(effect, HeroEffect::SpellBook));
    BattleMagic {
        spell_book: book.then(|| spells::available(details, ruleset)),
        masteries: spells::masteries(details, ruleset),
        mana: details.mana,
        max_mana: 0,
        eagle_eye: eagle_eye(details, ruleset),
    }
}

/// The hero's Eagle Eye (HER-SPL-004; the original's 0x4E4420 and the cast routine's test,
/// `docs/formats/installation.md` §7.24): the [`RateSum`] of the `eagle_eye` values of
/// [`grown_effects`] (the skill's grown by an Eagle Eye specialty, then each artifact type once,
/// which need the skill), at most 10 000 basis points; the spells it may learn are the combat
/// spells the hero has not learned (an artifact's spell is not learned) of a level at most the
/// largest `max_level`. None without a `max_level` (no skill).
fn eagle_eye(details: &HeroDetails, ruleset: &Ruleset) -> Option<EagleEye> {
    // rule: HER-SPL-004
    let single = single_precision(ruleset);
    let mut sum = RateSum::default();
    let mut max_level = 0;
    for (effect, growth) in grown_effects(details, ruleset) {
        if let HeroEffect::EagleEye { bp, max_level: l } = *effect {
            sum.add(bp.into(), growth, single);
            max_level = max_level.max(l);
        }
    }
    (max_level > 0).then(|| EagleEye {
        chance: sum.rate(FULL_BP.into()),
        spells: (0..ruleset.spells().len())
            .filter_map(SpellId::from_index)
            .zip(ruleset.spells())
            .filter(|&(id, spell)| {
                spell.category == SpellCategory::Combat
                    && spell.level <= max_level
                    && !details.spells.contains(&id)
            })
            .map(|(id, _)| id)
            .collect(),
    })
}

/// The opening spells (HER-BAT-017): one `OpeningSpell` per `opening_spell` effect of
/// [`grown_effects`] (each artifact type once, in the order of the types, whatever the order
/// worn), within one source in the order of its effects: the original casts Angelic Alliance's
/// (129) Prayer, then Armor of the Damned's (132) four (0x465330). The battle casts them at the
/// side's first activation (BAT-MAG-068).
fn opening_spells<'a>(
    details: &'a HeroDetails,
    ruleset: &'a Ruleset,
) -> impl Iterator<Item = BattleEffect> + 'a {
    grown_effects(details, ruleset).filter_map(|(effect, _)| match *effect {
        HeroEffect::OpeningSpell {
            spell,
            mastery,
            power,
        } => Some(BattleEffect::OpeningSpell {
            spell,
            mastery,
            power,
        }),
        _ => None,
    })
}

/// The spell bonuses (HER-BAT-012): per scope (every spell, a school, a spell) the [`RateSum`] of
/// the `spell_damage` values of [`grown_effects`] (Sorcery grows with a Sorcery specialty),
/// saturated at `u16::MAX` basis points, in the order of their first appearance; then the
/// specialty's `spell_specialty` with the hero's level (the battle divides it by the creature's
/// level).
fn spell_bonuses(details: &HeroDetails, ruleset: &Ruleset) -> Vec<BattleEffect> {
    let single = single_precision(ruleset);
    let mut totals: Vec<(SpellScope<SchoolId, SpellId>, RateSum)> = Vec::new();
    for (effect, growth) in grown_effects(details, ruleset) {
        let HeroEffect::SpellDamage { scope, bp } = *effect else {
            continue;
        };
        let index = match totals.iter().position(|(s, _)| *s == scope) {
            Some(i) => i,
            None => {
                totals.push((scope, RateSum::default()));
                totals.len() - 1
            }
        };
        totals[index].1.add(bp.into(), growth, single);
    }
    let mut out: Vec<BattleEffect> = totals
        .into_iter()
        .map(|(scope, sum)| BattleEffect::SpellDamage {
            scope,
            rate: sum.rate(u16::MAX.into()),
        })
        .collect();
    out.extend(
        specialty(details, ruleset)
            .iter()
            .filter_map(|effect| match *effect {
                HeroEffect::SpellSpecialty {
                    spell,
                    bp_per_level,
                } => Some(BattleEffect::SpellSpecialty {
                    spell,
                    bp_per_level,
                    level: details.level,
                }),
                _ => None,
            }),
    );
    out
}

/// The magic resistance (HER-BAT-013): one `MagicResistance` effect per source in the order of
/// [`grown_effects`] (the Resistance skill grown by a Resistance specialty, then each artifact
/// type once, in the order of the types, as the original's getter adds them), saturated at
/// `u16::MAX` basis points, with quirk Q-0005 each with the single-precision value the original
/// computes ([`Growth::single`]); the battle adds them in that order (BAT-MAG-008).
fn magic_resistance(details: &HeroDetails, ruleset: &Ruleset) -> Vec<BattleEffect> {
    let single = single_precision(ruleset);
    grown_effects(details, ruleset)
        .filter_map(|(effect, growth)| match *effect {
            HeroEffect::MagicResistance { bp } => {
                let mut sum = RateSum::default();
                sum.add(bp.into(), growth, single);
                Some(BattleEffect::MagicResistance {
                    rate: sum.rate(u16::MAX.into()),
                })
            }
            _ => None,
        })
        .collect()
}

/// The spell duration bonus (HER-BAT-014): one `SpellDuration` effect with the sum of the
/// `spell_duration` effects, each artifact type once (a combination's parts count too: the Ring of
/// the Magi gives 50 + 1 + 2 + 3), saturated at `u8::MAX`; none without any.
fn spell_duration(details: &HeroDetails, ruleset: &Ruleset) -> Option<BattleEffect> {
    let sum: u32 = effects(details, ruleset, Count::OncePerType)
        .filter_map(|effect| match *effect {
            HeroEffect::SpellDuration { rounds } => Some(u32::from(rounds)),
            _ => None,
        })
        .sum();
    (sum > 0).then(|| BattleEffect::SpellDuration {
        rounds: u8::try_from(sum).unwrap_or(u8::MAX),
    })
}

/// The casting limits (HER-BAT-015): `NoCasting` if one of the effects forbids casting, then one
/// `SpellLevelLimit` with the smallest `spell_level_limit` level, each artifact type once; none
/// without any. The battle applies them to both sides (BAT-MAG-061, BAT-MAG-062).
fn casting_limits(details: &HeroDetails, ruleset: &Ruleset) -> Vec<BattleEffect> {
    let no_casting = effects(details, ruleset, Count::OncePerType)
        .any(|effect| matches!(effect, HeroEffect::NoCasting))
        .then_some(BattleEffect::NoCasting);
    let limit = effects(details, ruleset, Count::OncePerType)
        .filter_map(|effect| match *effect {
            HeroEffect::SpellLevelLimit { level } => Some(level),
            _ => None,
        })
        .min()
        .map(|max_level| BattleEffect::SpellLevelLimit { max_level });
    no_casting.into_iter().chain(limit).collect()
}

/// The spell defences (HER-BAT-016): one `SpellImmunity` per spell its effects name, in the order
/// of their first appearance; one `SpellLevelImmunity` with the largest `spell_level_immunity`
/// level; `MindImmunity` and `NegateImmunities` once if an effect gives them; each artifact type
/// once, none without any. The battle applies them to the stacks the hero controls and, for the
/// negation, to its side's spells (BAT-MAG-063 to BAT-MAG-066).
fn spell_defences(details: &HeroDetails, ruleset: &Ruleset) -> Vec<BattleEffect> {
    let mut out: Vec<BattleEffect> = Vec::new();
    let mut level: Option<u8> = None;
    let (mut mind, mut negates) = (false, false);
    for effect in effects(details, ruleset, Count::OncePerType) {
        match *effect {
            HeroEffect::SpellImmunity { spell } => {
                let immunity = BattleEffect::SpellImmunity { spell };
                if !out.contains(&immunity) {
                    out.push(immunity);
                }
            }
            HeroEffect::SpellLevelImmunity { level: l } => level = level.max(Some(l)),
            HeroEffect::MindImmunity => mind = true,
            HeroEffect::NegateImmunities => negates = true,
            _ => {}
        }
    }
    out.extend(level.map(|max_level| BattleEffect::SpellLevelImmunity { max_level }));
    out.extend(mind.then_some(BattleEffect::MindImmunity));
    out.extend(negates.then_some(BattleEffect::NegateImmunities));
    out
}

/// An effective primary skill (HER-BAT-001): the base value plus the `primary_skills` effects of
/// every worn copy, between the ruleset's minimum and maximum.
fn effective_primary(
    details: &HeroDetails,
    ruleset: &Ruleset,
    params: &HeroParams,
    skill: PrimarySkill,
) -> u8 {
    let mut total = i32::from(details.primary[skill.index()]);
    for effect in effects(details, ruleset, Count::PerCopy) {
        if let HeroEffect::PrimarySkills {
            attack,
            defense,
            power,
            knowledge,
        } = *effect
        {
            let change = [attack, defense, power, knowledge][skill.index()];
            total = total.saturating_add(i32::from(change));
        }
    }
    let min = params.primary_min[skill.index()];
    let value = total.clamp(i32::from(min), i32::from(params.primary_max));
    u8::try_from(value).unwrap_or(params.primary_max)
}

/// Spell points per point of effective Knowledge (HER-MAN-001; the original's `10 × Knowledge`,
/// 0x486C80).
pub const MANA_PER_KNOWLEDGE: u32 = 10;

/// The hero's maximum spell points (HER-MAN-001, E08 T18b): [`MANA_PER_KNOWLEDGE`] times its
/// effective Knowledge (HER-BAT-001's: base and artifacts, between the ruleset's bounds), grown
/// by the sum of the `max_mana` values of [`grown_effects`] (Intelligence, grown by an
/// Intelligence specialty: 5 % of itself per level), rounded down; with quirk Q-0005 the
/// original's single-precision rate: `trunc((1 + f32(r × (1 + 0.05 × level))) × 10 K)` in its
/// x87 arithmetic (the getter 0x4E48B0, its caller 0x486C7B). Artifacts and skills the ruleset
/// does not know count for nothing, and a ruleset without hero parameters (which validation
/// refuses) takes Knowledge as given: validate first.
pub fn max_mana(details: &HeroDetails, ruleset: &Ruleset) -> u16 {
    // rule: HER-MAN-001
    let knowledge = ruleset
        .hero_params()
        .map_or(details.primary[PrimarySkill::Knowledge.index()], |params| {
            effective_primary(details, ruleset, params, PrimarySkill::Knowledge)
        });
    let base = MANA_PER_KNOWLEDGE * u32::from(knowledge);
    let points = grown(details, ruleset, base, |effect| match *effect {
        HeroEffect::MaxMana { bp } => Some(bp),
        _ => None,
    });
    u16::try_from(points).unwrap_or(u16::MAX)
}

/// The hero's Attack and Defense as the adventure map reads them (HER-ADV-001, E08 T20a1): its
/// effective values (HER-BAT-001: the base values plus the `primary_skills` effects, the worn
/// artifacts' per copy, between the ruleset's bounds, 0 to 99), as the original's hero record
/// holds them: equipping an artifact adds its primary skills and its parts' to +0x476 to +0x479,
/// taking it off subtracts them (`hero_equip_artifact` 0x4E2A00, `hero_remove_artifact`
/// 0x4E2BD0), and the reader clamps them to 0–99 (`hero_attack_defense_factor` 0x4E5400, the
/// power in the wandering monsters' answer, ADV-MON-007). A ruleset without hero parameters
/// (which validation refuses) takes the base values: validate first.
pub fn attack_defense(details: &HeroDetails, ruleset: &Ruleset) -> (u8, u8) {
    // rule: HER-ADV-001
    let value = |skill: PrimarySkill| {
        ruleset
            .hero_params()
            .map_or(details.primary[skill.index()], |params| {
                effective_primary(details, ruleset, params, skill)
            })
    };
    (value(PrimarySkill::Attack), value(PrimarySkill::Defense))
}

/// The most a hero's join bonus counts ([`join_bonus`]): Expert Diplomacy's.
pub const MAX_JOIN_BONUS: u8 = 3;

/// How much readier wandering creatures are to join the hero (HER-ADV-002, E08 T20a1): the sum of
/// the levels of its `join_bonus` effects (Diplomacy: 1, 2, 3 by its level; each artifact type
/// once), at most [`MAX_JOIN_BONUS`]. The original's Diplomacy level (+0xCD) in the wandering
/// monsters' answer (`monster_visit_decision` 0x4A7740; its Easy rule is the adventure's,
/// ADV-MON-007); no skill is read by name (ADR-0016 point 4).
pub fn join_bonus(details: &HeroDetails, ruleset: &Ruleset) -> u8 {
    // rule: HER-ADV-002
    effects(details, ruleset, Count::OncePerType)
        .filter_map(|effect| match *effect {
            HeroEffect::JoinBonus { level } => Some(level),
            _ => None,
        })
        .fold(0, u8::saturating_add)
        .min(MAX_JOIN_BONUS)
}

/// The experience a hero gains from `base` points (HER-EXP-001, E08 T19a2a): `base` grown by the
/// sum of the `experience_gain` values of [`grown_effects`] (Learning, grown by a Learning
/// specialty: 5 % of itself per level), rounded down, at most `u32::MAX`; with quirk Q-0005 the
/// original's single-precision rate and product: `trunc((1 + f32(r × (1 + 0.05 × level))) ×
/// f32(base))` in its x87 arithmetic (`battle_experience` 0x46A42D with the Learning getter
/// 0x4E4840, a twin of Intelligence's), so a specialist can gain one point less and a `base` above
/// 2^24 is rounded to single precision even without Learning.
pub fn gained_experience(details: &HeroDetails, ruleset: &Ruleset, base: u32) -> u32 {
    // rule: HER-EXP-001
    let points = grown(details, ruleset, base, |effect| match *effect {
        HeroEffect::ExperienceGain { bp } => Some(bp),
        _ => None,
    });
    u32::try_from(points).unwrap_or(u32::MAX)
}

/// `base` grown by the sum of the rates `rate` picks among [`grown_effects`] (each grown by its
/// skill's specialty), rounded down; with quirk Q-0005 as the original's skill getters and their
/// callers compute it: `trunc((1 + the single-precision sum) × f32(base))` in x87 arithmetic.
fn grown(
    details: &HeroDetails,
    ruleset: &Ruleset,
    base: u32,
    rate: impl Fn(&CompiledHeroEffect) -> Option<u16>,
) -> u64 {
    let single = single_precision(ruleset);
    let mut sum = RateSum::default();
    for (effect, growth) in grown_effects(details, ruleset) {
        if let Some(bp) = rate(effect) {
            sum.add(bp.into(), growth, single);
        }
    }
    if single {
        let factor = X87::ONE.plus(sum.single.unwrap_or(X87::ZERO));
        return factor
            .times(X87::int(base.into()).round(SINGLE_BITS))
            .trunc();
    }
    let gain = mul_div_floor(base.into(), sum.bp, FULL_BP.into()).unwrap_or(u64::MAX);
    u64::from(base).saturating_add(gain)
}

/// The surrender discount (HER-BAT-007): the sum of the grown values of the surrender discounts
/// of [`grown_effects`] (they add, batch 1b R4), at most 10 000.
fn surrender_discount(details: &HeroDetails, ruleset: &Ruleset) -> u16 {
    let sum: u64 = grown_effects(details, ruleset)
        .filter_map(|(effect, growth)| match *effect {
            HeroEffect::SurrenderDiscount { bp } => Some(growth.grow(u64::from(bp))),
            _ => None,
        })
        .sum();
    u16::try_from(sum.min(u64::from(FULL_BP))).unwrap_or(FULL_BP)
}

/// The stack statistic effects (HER-BAT-008): the `creature_stat` effects as additions, the
/// `creature_stat_percent` effects as shares rounded down, and a creature specialty as attack and
/// defense shares rounded up of `bp_per_step` per full step of (the creature's level) hero
/// levels plus its speed as an addition, for the named creature and its upgrades. Each artifact
/// type counts once; skill specialties do not grow them. Effects with the same creatures,
/// statistic and operation are summed (saturated to `i16`; a sum of 0 is left out), in the order
/// of their first appearance.
fn stack_stats(details: &HeroDetails, ruleset: &Ruleset) -> Vec<BattleEffect> {
    type Key = (CreatureSelector<CreatureTypeId>, CreatureStat, StatOp);
    let mut totals: Vec<(Key, i32)> = Vec::new();
    let mut add = |key: Key, amount: i32| match totals.iter_mut().find(|(k, _)| *k == key) {
        Some((_, total)) => *total = total.saturating_add(amount),
        None => totals.push((key, amount)),
    };
    for effect in effects(details, ruleset, Count::OncePerType) {
        match *effect {
            HeroEffect::CreatureStat {
                creatures,
                stat,
                amount,
            } => add((creatures, stat, StatOp::Add), i32::from(amount)),
            HeroEffect::CreatureStatPercent {
                creatures,
                stat,
                bp,
            } => add((creatures, stat, StatOp::PercentBpDown), i32::from(bp)),
            HeroEffect::CreatureSpecialty {
                creature,
                bp_per_step,
                speed,
            } => {
                // Creature levels are at least 1 (content validation).
                let level = ruleset
                    .creature_info(creature)
                    .map_or(1, |i| i.level)
                    .max(1);
                let bp = i32::from(bp_per_step).saturating_mul(i32::from(details.level / level));
                let creatures = CreatureSelector::Creature(creature);
                add((creatures, CreatureStat::Attack, StatOp::PercentBp), bp);
                add((creatures, CreatureStat::Defense, StatOp::PercentBp), bp);
                add(
                    (creatures, CreatureStat::Speed, StatOp::Add),
                    i32::from(speed),
                );
            }
            _ => {}
        }
    }
    totals
        .into_iter()
        .filter(|&(_, total)| total != 0)
        .map(|((creatures, stat, op), total)| BattleEffect::StackStat {
            creatures,
            stat,
            op,
            // Within `i16` after the clamp.
            amount: i16::try_from(total.clamp(i16::MIN.into(), i16::MAX.into()))
                .unwrap_or_default(),
        })
        .collect()
}

/// The stack trait effects (HER-BAT-009): `Regeneration`, `NoRangePenalty` and
/// `ShootWhenAdjacent`, each artifact type once, in the order of their first appearance; one
/// effect per kind and creatures, a regeneration with the most hit points.
fn stack_traits(details: &HeroDetails, ruleset: &Ruleset) -> Vec<BattleEffect> {
    let mut out: Vec<BattleEffect> = Vec::new();
    for effect in effects(details, ruleset, Count::OncePerType) {
        let new = match *effect {
            HeroEffect::Regeneration {
                creatures,
                hit_points,
            } => {
                let same = out.iter_mut().find_map(|e| match e {
                    BattleEffect::Regeneration {
                        creatures: c,
                        hit_points: h,
                    } if *c == creatures => Some(h),
                    _ => None,
                });
                if let Some(h) = same {
                    *h = (*h).max(hit_points);
                    continue;
                }
                BattleEffect::Regeneration {
                    creatures,
                    hit_points,
                }
            }
            HeroEffect::NoRangePenalty { creatures } => BattleEffect::NoRangePenalty { creatures },
            HeroEffect::ShootWhenAdjacent { creatures } => {
                BattleEffect::ShootWhenAdjacent { creatures }
            }
            _ => continue,
        };
        if !out.contains(&new) {
            out.push(new);
        }
    }
    out
}

/// The alignment groups (HER-BAT-010): each `alignment_group` effect, each artifact type once,
/// equal groups once, in the order of their first appearance.
fn alignment_groups(details: &HeroDetails, ruleset: &Ruleset) -> Vec<BattleEffect> {
    let mut out: Vec<BattleEffect> = Vec::new();
    let groups = effects(details, ruleset, Count::OncePerType).filter_map(|e| match e {
        // Content holds at most `MAX_ALIGNMENT_FACTIONS` factions per group.
        HeroEffect::AlignmentGroup { factions } => FactionGroup::new(factions),
        _ => None,
    });
    for factions in groups {
        let group = BattleEffect::AlignmentGroup { factions };
        if !out.contains(&group) {
            out.push(group);
        }
    }
    out
}

/// How often a worn artifact's effects count.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Count {
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
/// once, in the order of the types, as the original's skill getters add them) and the
/// specialty's own, not grown.
fn grown_effects<'a>(
    details: &'a HeroDetails,
    ruleset: &'a Ruleset,
) -> impl Iterator<Item = (&'a CompiledHeroEffect, Growth)> {
    let specialty = specialty(details, ruleset);
    let skills = details.skills.iter().flat_map(move |&(id, level)| {
        let per_level_bp: u64 = specialty
            .iter()
            .filter_map(|effect| match *effect {
                HeroEffect::SkillSpecialty {
                    skill,
                    bp_per_level,
                } if skill == id => Some(u64::from(bp_per_level)),
                _ => None,
            })
            .sum();
        let growth = Growth {
            per_level_bp,
            level: details.level,
        };
        ruleset
            .skill(id)
            .map(|skill| skill.effects(level))
            .unwrap_or_default()
            .iter()
            .map(move |effect| (effect, growth))
    });
    let artifacts = worn_types(details, ruleset).flat_map(|(_, artifact)| artifact.effects.iter());
    let others = artifacts.chain(specialty);
    skills.chain(others.map(|effect| (effect, Growth::NONE)))
}

/// The worn artifact types whose effects count, each once, in type order: every worn artifact
/// and every part of a worn combination, an artifact that needs a skill only while the hero has
/// it. Artifacts the ruleset does not know are skipped.
fn worn_types<'a>(
    details: &'a HeroDetails,
    ruleset: &'a Ruleset,
) -> impl Iterator<Item = (ArtifactTypeId, &'a ArtifactType)> {
    let mut types = BTreeSet::new();
    for item in &details.worn {
        let id = item.artifact;
        types.insert(id);
        let parts = ruleset.artifact(id).map(|a| a.parts.as_slice());
        types.extend(parts.unwrap_or_default().iter().copied());
    }
    types
        .into_iter()
        .filter_map(|id: ArtifactTypeId| ruleset.artifact(id).map(|a| (id, a)))
        .filter(|(_, artifact)| {
            artifact
                .needs_skill
                .is_none_or(|needed| has_skill(details, needed))
        })
}

/// Whether the hero has `skill` at any level.
fn has_skill(details: &HeroDetails, skill: SkillId) -> bool {
    details.skills.iter().any(|&(id, _)| id == skill)
}

/// What a hero's Necromancy raises after a won battle (E08 T19a2b; GAM-BAT-006 applies it).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Necromancy {
    /// The share of the killed enemy creatures raised (HER-NEC-001): basis points (1..=10 000)
    /// and, under quirk Q-0005, the original's single-precision value where it differs.
    pub share: Rate,
    /// The creature raised (HER-NEC-002).
    pub creature: CreatureTypeId,
}

/// The hero's Necromancy (HER-NEC-001, HER-NEC-002; E08 T19a2b), `None` without a share or a
/// creature to raise. The share is the [`RateSum`] of the `necromancy` values of
/// [`grown_effects`] — the skill's grown by a Necromancy specialty (5 % of itself per level),
/// then each artifact type once, those that need the skill only with it, a share with
/// `unless_skill` only without that skill — at most 10 000 basis points. The original's
/// `hero_necromancy_power` (0x4E3CD0): with the skill its table value (times 1 + 0.05 × level
/// for a specialist), +0.05, +0.10, +0.15 for the Amulet of the Undertaker, the Vampire's Cowl
/// and Dead Man's Boots (each worn or through the Cloak of the Undead King, its combination), in
/// single precision (quirk Q-0005), capped at 1.0; without the skill only +0.30 for the Cloak.
/// Its town buildings' +0.10 and +0.20 come with towns (E09). The creature: [`raised_creature`].
pub fn necromancy(details: &HeroDetails, ruleset: &Ruleset) -> Option<Necromancy> {
    // rule: HER-NEC-001
    let single = single_precision(ruleset);
    let mut sum = RateSum::default();
    for (effect, growth) in grown_effects(details, ruleset) {
        if let HeroEffect::Necromancy { bp, unless_skill } = *effect
            && unless_skill.is_none_or(|skill| !has_skill(details, skill))
        {
            sum.add(bp.into(), growth, single);
        }
    }
    let share = sum.rate(FULL_BP.into());
    let creature = raised_creature(details, ruleset)?;
    (share.bp > 0).then_some(Necromancy { share, creature })
}

/// The creature the hero's Necromancy raises (HER-NEC-002, E08 decision 30): of its `raises`
/// effects (the secondary skills', the worn artifact types' as [`worn_types`] counts them, the
/// specialty's) the highest `priority` decides; a `by_level` effect names the creature of the
/// hero's level in its skill (none to Expert). Among equal priorities, if every one comes from
/// an artifact, the lowest artifact id (that of the artifact or combination part whose list holds
/// the effect), then the lowest creature id; otherwise the lowest creature id. `None` without a
/// `raises` effect. The original's `hero_necromancy_creature` (0x4E3C60): the Skeleton, with the
/// Cloak of the Undead King by the Necromancy level Skeleton, Walking Dead, Wight, Lich.
fn raised_creature(details: &HeroDetails, ruleset: &Ruleset) -> Option<CreatureTypeId> {
    // rule: HER-NEC-002
    let level_in = |skill: SkillId| {
        let level = details.skills.iter().find(|&&(id, _)| id == skill);
        level.map_or(0, |&(_, level)| usize::from(level))
    };
    let skills = details.skills.iter().flat_map(|&(id, level)| {
        let effects = ruleset.skill(id).map(|skill| skill.effects(level));
        effects
            .unwrap_or_default()
            .iter()
            .map(|effect| (effect, None))
    });
    let artifacts = worn_types(details, ruleset).flat_map(|(id, artifact)| {
        artifact
            .effects
            .iter()
            .map(move |effect| (effect, Some(id)))
    });
    let specialty = specialty(details, ruleset)
        .iter()
        .map(|effect| (effect, None));
    let candidates: Vec<(u8, Option<ArtifactTypeId>, CreatureTypeId)> = skills
        .chain(artifacts)
        .chain(specialty)
        .filter_map(|(effect, source)| match *effect {
            HeroEffect::Raises { priority, creature } => {
                let creature = match creature {
                    RaisedCreature::Creature(c) => c,
                    RaisedCreature::ByLevel { skill, by_level } => {
                        by_level[level_in(skill).min(RAISED_LEVELS - 1)]
                    }
                };
                Some((priority, source, creature))
            }
            _ => None,
        })
        .collect();
    let top = candidates.iter().map(|&(p, _, _)| p).max()?;
    let tied = candidates.iter().filter(|&&(p, _, _)| p == top);
    let from_artifacts = tied.clone().all(|&(_, source, _)| source.is_some());
    tied.min_by_key(|&&(_, source, creature)| {
        let artifact = source.filter(|_| from_artifacts).map(|a| a.raw());
        (artifact, creature.raw())
    })
    .map(|&(_, _, creature)| creature)
}

/// Whether the derivation follows the original's single-precision rates (quirk Q-0005,
/// OPEN-QUESTIONS Q45).
fn single_precision(ruleset: &Ruleset) -> bool {
    ruleset.quirks().is_enabled(SINGLE_PRECISION_RATES)
}

/// A skill specialty's growth of a value (HER-BAT-004): basis points of itself per hero level
/// (0: none) and the hero's level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Growth {
    /// Growth per hero level, basis points.
    per_level_bp: u64,
    /// The hero's level.
    level: u8,
}

impl Growth {
    /// No growth.
    const NONE: Self = Self {
        per_level_bp: 0,
        level: 0,
    };

    /// `bp` grown, rounded down.
    fn grow(self, bp: u64) -> u64 {
        let growth = self.per_level_bp * u64::from(self.level);
        bp.saturating_add(bp.saturating_mul(growth) / u64::from(FULL_BP))
    }

    /// The single-precision value of `bp` basis points grown as the original's skill getters
    /// grow it (`installation.md` §7.4, OPEN-QUESTIONS Q45): `f32(r × (f32(level) × g + 1))` in
    /// its x87 arithmetic, with `r` and `g` (the growth per level) single precision; `f32(r)`
    /// without growth.
    fn single(self, bp: u32) -> X87 {
        let ratio = |n: u64| {
            let n = u32::try_from(n).unwrap_or(u32::MAX);
            X87::ratio(n, FULL_BP.into(), SINGLE_BITS).unwrap_or(X87::ZERO)
        };
        let rate = ratio(bp.into());
        if self.per_level_bp == 0 {
            return rate;
        }
        X87::int(self.level.into())
            .round(SINGLE_BITS)
            .times(ratio(self.per_level_bp))
            .plus(X87::ONE)
            .times(rate)
            .round(SINGLE_BITS)
    }
}

/// A rate summed from several sources (OPEN-QUESTIONS Q45): the basis points, each grown value
/// rounded down (saturating), and, with quirk Q-0005, the single-precision sum the original
/// keeps, each source's [`Growth::single`] added and the sum rounded to single precision in the
/// order of the sources.
#[derive(Debug, Clone, Copy, Default)]
struct RateSum {
    /// Basis points.
    bp: u64,
    /// The single-precision sum.
    single: Option<X87>,
}

impl RateSum {
    /// Adds `bp` basis points grown by `growth`; with `single` also its single-precision value.
    fn add(&mut self, bp: u32, growth: Growth, single: bool) {
        self.bp = self.bp.saturating_add(growth.grow(bp.into()));
        if single {
            let sum = self.single.unwrap_or(X87::ZERO).plus(growth.single(bp));
            self.single = Some(sum.round(SINGLE_BITS));
        }
    }

    /// The rate, its basis points at most `max` and its single-precision value at most that of
    /// `max`; the single-precision value only where it differs from that of the basis points.
    fn rate(self, max: u32) -> Rate {
        let bp = u32::try_from(self.bp).unwrap_or(u32::MAX).min(max);
        let single = self
            .single
            .map(|v| v.min(Growth::NONE.single(max)))
            .filter(|&v| v != Growth::NONE.single(bp))
            .and_then(|v| NonZeroU32::new(v.to_f32_bits()));
        Rate { bp, single }
    }
}

/// The damage bonuses (HER-BAT-004): per kind (ranged, melee, reduction) the sum of the grown
/// values of [`grown_effects`]. Sums saturate at `u16::MAX`; the reduction is at most 10 000.
fn damage_bonuses(details: &HeroDetails, ruleset: &Ruleset) -> DamageBonuses {
    let single = single_precision(ruleset);
    // [ranged, melee, reduction]
    let mut sums = [RateSum::default(); 3];
    for (effect, growth) in grown_effects(details, ruleset) {
        let (kind, bp) = match *effect {
            HeroEffect::RangedDamage { bp } => (0, bp),
            HeroEffect::MeleeDamage { bp } => (1, bp),
            HeroEffect::DamageReduction { bp } => (2, bp),
            _ => continue,
        };
        sums[kind].add(bp.into(), growth, single);
    }
    DamageBonuses {
        ranged: sums[0].rate(u16::MAX.into()),
        melee: sums[1].rate(u16::MAX.into()),
        reduction: sums[2].rate(FULL_BP.into()),
    }
}

/// The war machines the hero commands (HER-BAT-005): the Ballista if an effect gives Artillery,
/// with the highest level, the most shots and the highest double-damage chance among such
/// effects (skill
/// specialties do not grow them: the original has no Artillery specialist and keeps no
/// Artillery value its specialty getter could scale, `docs/formats/installation.md` §7.4); the
/// First Aid Tent if an effect gives First Aid, its heal bonus the sum of the grown `heal_bp`
/// values of [`grown_effects`] (a First Aid specialist's grows by 5 % per level), saturated; the
/// Catapult if an effect gives Ballistics, at the highest such level (HER-BAT-018, BAT-Q43).
fn machine_control(details: &HeroDetails, ruleset: &Ruleset) -> MachineControl {
    let single = single_precision(ruleset);
    let mut control = MachineControl::default();
    let mut heal = RateSum::default();
    for (effect, growth) in grown_effects(details, ruleset) {
        match *effect {
            HeroEffect::Artillery {
                level,
                shots,
                double_damage_chance_bp,
            } => {
                let ballista = control.ballista.get_or_insert(BallistaControl {
                    level,
                    shots,
                    double_damage_chance_bp,
                });
                ballista.level = ballista.level.max(level);
                ballista.shots = ballista.shots.max(shots);
                ballista.double_damage_chance_bp = ballista
                    .double_damage_chance_bp
                    .max(double_damage_chance_bp);
            }
            HeroEffect::FirstAid { heal_bp } => {
                heal.add(heal_bp.into(), growth, single);
                control.tent = Some(TentControl {
                    heal_bonus: heal.rate(u32::MAX),
                });
            }
            // rule: HER-BAT-018
            HeroEffect::Ballistics { level } => {
                let catapult = control.catapult.get_or_insert(CatapultControl { level });
                catapult.level = catapult.level.max(level);
            }
            _ => {}
        }
    }
    control
}

/// Every effect the hero's details bring into a battle: its secondary skills at their levels,
/// its worn artifacts ([`artifact_effects`]) and its specialty. References the ruleset does not
/// know are skipped (the details are validated before).
pub(crate) fn effects<'a>(
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
        .flat_map(|item| {
            let id = item.artifact;
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
