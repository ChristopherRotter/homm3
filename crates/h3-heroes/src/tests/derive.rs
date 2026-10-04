//! `derive_battle_hero` (HER-BAT-001 to HER-BAT-008).

use std::vec;

use h3_content::{Affects, HeroEffectDef, Ruleset};
use h3_contracts::{BattleEffect, BattleHero, BattleMagic, DamageBonuses, MachineControl};
use proptest::prelude::*;

use super::*;

fn primary(attack: i8, defense: i8, power: i8, knowledge: i8) -> HeroEffectDef {
    HeroEffectDef::PrimarySkills {
        attack,
        defense,
        power,
        knowledge,
    }
}

/// The test pack with effects: Tactics at its three levels and Archery with a damage bonus;
/// primary skills on the combination `t:circle` and its parts (`t:loop` needs Luck), on `t:eye`
/// and in the specialty; `t:hook` gives Tactics 1.
fn effect_pack() -> ContentPack {
    let mut p = pack();
    for s in &mut p.skills {
        match s.key.to_string().as_str() {
            "t:tactics" => {
                s.levels = [1, 2, 3].map(|level| vec![HeroEffectDef::Tactics { level }]);
            }
            "t:archery" => {
                s.levels = [(); 3].map(|()| vec![HeroEffectDef::RangedDamage { bp: 1000 }])
            }
            _ => {}
        }
    }
    for a in &mut p.artifacts {
        match a.key.to_string().as_str() {
            "t:circle" => a.effects = vec![primary(1, 1, 1, 1)],
            "t:helm" => a.effects = vec![primary(1, 2, 3, 4)],
            "t:band" => a.effects = vec![primary(-5, -5, -5, -5)],
            "t:loop" => {
                a.needs_skill = Some(key("t:luck"));
                a.effects = vec![primary(10, 10, 10, 10)];
            }
            "t:eye" => a.effects = vec![primary(2, 3, 0, 0)],
            "t:hook" => a.effects = vec![HeroEffectDef::Tactics { level: 1 }],
            _ => {}
        }
    }
    p.heroes[0].specialty = vec![primary(0, 1, 0, 0)];
    p
}

fn effect_ruleset() -> Ruleset {
    compile(effect_pack()).unwrap()
}

/// The effect pack plus morale and luck: Leadership and Luck +1/+2/+3 for the own side; the
/// combination `t:circle` morale +1, its parts `t:band` morale +1 and enemy luck at most 0 and
/// `t:helm` morale at most 1 for both sides; `t:eye` luck +1; `t:hook` enemy morale −1 and
/// morale at most 2 for both sides; the specialty enemy luck −1 and enemy morale +1.
fn morale_ruleset() -> Ruleset {
    use Affects::{Both, Enemy, Own};
    let morale = |amount, affects| HeroEffectDef::Morale { amount, affects };
    let luck = |amount, affects| HeroEffectDef::Luck { amount, affects };
    let mut p = effect_pack();
    for s in &mut p.skills {
        match s.key.to_string().as_str() {
            "t:leadership" => s.levels = [1, 2, 3].map(|n| vec![morale(n, Own)]),
            "t:luck" => s.levels = [1, 2, 3].map(|n| vec![luck(n, Own)]),
            _ => {}
        }
    }
    for a in &mut p.artifacts {
        let more = match a.key.to_string().as_str() {
            "t:circle" => vec![morale(1, Own)],
            "t:band" => vec![
                morale(1, Own),
                HeroEffectDef::LuckAtMost {
                    value: 0,
                    affects: Enemy,
                },
            ],
            "t:helm" => vec![HeroEffectDef::MoraleAtMost {
                value: 1,
                affects: Both,
            }],
            "t:eye" => vec![luck(1, Own)],
            "t:hook" => vec![
                morale(-1, Enemy),
                HeroEffectDef::MoraleAtMost {
                    value: 2,
                    affects: Both,
                },
            ],
            _ => vec![],
        };
        a.effects.extend(more);
    }
    p.heroes[0]
        .specialty
        .extend([luck(-1, Enemy), morale(1, Enemy)]);
    compile(p).unwrap()
}

/// The effect pack plus damage bonuses: Offense +10/20/30 % melee and Armorer 5/10/15 %
/// reduction; `t:eye` +5 % ranged, `t:loop` (part of `t:circle`, needs Luck) +20 % ranged,
/// `t:helm` (another part) 90 % reduction; the specialty grows `skill` by `bp_per_level` of
/// itself per hero level and adds +1 % melee of its own.
fn damage_ruleset(skill: &str, bp_per_level: u16) -> Ruleset {
    let mut p = effect_pack();
    for s in &mut p.skills {
        match s.key.to_string().as_str() {
            "t:offense" => {
                s.levels = [1000, 2000, 3000].map(|bp| vec![HeroEffectDef::MeleeDamage { bp }]);
            }
            "t:armorer" => {
                s.levels = [500, 1000, 1500].map(|bp| vec![HeroEffectDef::DamageReduction { bp }]);
            }
            _ => {}
        }
    }
    for a in &mut p.artifacts {
        let more = match a.key.to_string().as_str() {
            "t:eye" => HeroEffectDef::RangedDamage { bp: 500 },
            "t:loop" => HeroEffectDef::RangedDamage { bp: 2000 },
            "t:helm" => HeroEffectDef::DamageReduction { bp: 9000 },
            _ => continue,
        };
        a.effects.push(more);
    }
    p.heroes[0].specialty.extend([
        HeroEffectDef::SkillSpecialty {
            skill: key(skill),
            bp_per_level,
        },
        HeroEffectDef::MeleeDamage { bp: 100 },
    ]);
    compile(p).unwrap()
}

/// The battle hero with the given derived values and everything else neutral.
fn expected(r: &Ruleset, primary: [u8; 4], tactics: u8, mana: u16) -> BattleHero {
    let mut hero = BattleHero::new(r.hero_type_by_key("t:orrin").unwrap());
    [hero.attack, hero.defense, hero.power, hero.knowledge] = primary;
    hero.tactics = tactics;
    hero.magic.mana = mana;
    hero
}

// covers: HER-BAT-001, HER-BAT-002
#[test]
fn derived_primary_skills_and_tactics() {
    let r = effect_ruleset();
    let d = details(&r);
    // Base [99, 0, 1, 0]; the circle +1 each and its parts band -5 each and helm +1/+2/+3/+4
    // (the loop needs Luck); two eyes +4/+6; the specialty +1 defense. Totals 100, 5, 0, 0:
    // attack capped at 99, power and knowledge raised to their minimum 1. Tactics: the skill's
    // Expert level beats the hook's 1.
    // Basic Archery's +10 % is the only damage bonus (HER-BAT-004).
    let archer = |hero: BattleHero| BattleHero {
        damage: DamageBonuses {
            ranged_bp: 1000,
            ..DamageBonuses::default()
        },
        ..hero
    };
    assert_eq!(
        derive_battle_hero(&d, &r),
        Ok(archer(expected(&r, [99, 5, 1, 1], 3, u16::MAX)))
    );

    // With Luck (any level) the loop counts: +10 each.
    let mut lucky = d.clone();
    lucky.skills.push((skill(&r, "t:luck"), 1));
    assert_eq!(
        derive_battle_hero(&lucky, &r),
        Ok(archer(expected(&r, [99, 15, 10, 10], 3, u16::MAX)))
    );

    // Without the Tactics skill the hook's level remains; without any source it is 0.
    let mut untrained = d.clone();
    untrained
        .skills
        .retain(|&(s, _)| s != skill(&r, "t:tactics"));
    assert_eq!(derive_battle_hero(&untrained, &r).unwrap().tactics, 1);
    untrained.skills.push((skill(&r, "t:tactics"), 1));
    assert_eq!(derive_battle_hero(&untrained, &r).unwrap().tactics, 1);
    untrained.skills.pop();
    untrained.skills.push((skill(&r, "t:tactics"), 2));
    assert_eq!(derive_battle_hero(&untrained, &r).unwrap().tactics, 2);

    // Bare details: the base values (below-minimum knowledge raised) and the specialty only.
    let bare = HeroDetails {
        skills: vec![],
        worn: vec![],
        backpack: vec![],
        spells: vec![],
        mana: 7,
        ..d
    };
    assert_eq!(
        derive_battle_hero(&bare, &r),
        Ok(expected(&r, [99, 1, 1, 1], 0, 7))
    );
}

// covers: HER-BAT-001
#[test]
fn derivation_leaves_later_fields_neutral_and_validates_first() {
    let r = effect_ruleset();
    let hero = derive_battle_hero(&details(&r), &r).unwrap();
    assert_eq!(hero.war_machines, MachineControl::default());
    assert_eq!(hero.surrender_discount_bp, 0);
    assert!(hero.effects.is_empty());
    assert_eq!(
        hero.magic,
        BattleMagic {
            mana: u16::MAX,
            ..BattleMagic::default()
        }
    );

    let mut d = details(&r);
    d.level = 0;
    assert_eq!(derive_battle_hero(&d, &r), Err(DetailsError::LevelZero));
    let mut p = effect_pack();
    p.hero_params = None;
    let without = compile(p).unwrap();
    assert_eq!(
        derive_battle_hero(&details(&without), &without),
        Err(DetailsError::NoHeroParams)
    );
}

// covers: HER-BAT-003
#[test]
fn morale_and_luck_effects_are_combined_and_count_each_artifact_once() {
    use Affects::{Both, Enemy, Own};
    let r = morale_ruleset();
    let d = details(&r);
    // Own morale: the circle and its part band, +1 each; enemy morale: the hook's −1 and the
    // specialty's +1 cancel and are left out; own luck: the eye once although two are worn;
    // enemy luck: the specialty's −1; the caps: the smaller of the hook's 2 and the helm's 1
    // for both sides, the band's 0 for the enemy.
    let caps = [
        BattleEffect::MoraleAtMost {
            value: 1,
            affects: Both,
        },
        BattleEffect::LuckAtMost {
            value: 0,
            affects: Enemy,
        },
    ];
    let hero = derive_battle_hero(&d, &r).unwrap();
    assert_eq!(
        hero.effects,
        [
            &[
                BattleEffect::Morale {
                    amount: 2,
                    affects: Own,
                },
                BattleEffect::Luck {
                    amount: 1,
                    affects: Own,
                },
                BattleEffect::Luck {
                    amount: -1,
                    affects: Enemy,
                },
            ][..],
            &caps,
        ]
        .concat()
    );
    // Advanced Leadership and Basic Luck add to the own morale and luck; without the hook the
    // specialty's enemy morale +1 remains, and the helm's cap.
    let mut d = d;
    d.skills
        .extend([(skill(&r, "t:leadership"), 2), (skill(&r, "t:luck"), 1)]);
    d.worn.retain(|&a| a != art(&r, "t:hook"));
    let hero = derive_battle_hero(&d, &r).unwrap();
    assert_eq!(
        hero.effects,
        [
            &[
                BattleEffect::Morale {
                    amount: 4,
                    affects: Own,
                },
                BattleEffect::Morale {
                    amount: 1,
                    affects: Enemy,
                },
                BattleEffect::Luck {
                    amount: 2,
                    affects: Own,
                },
                BattleEffect::Luck {
                    amount: -1,
                    affects: Enemy,
                },
            ][..],
            &caps,
        ]
        .concat()
    );
}

// covers: HER-BAT-006
#[test]
fn a_worn_no_retreat_artifact_adds_one_no_retreat_effect() {
    let mut p = effect_pack();
    for a in &mut p.artifacts {
        if a.key.to_string() == "t:eye" {
            a.effects.push(HeroEffectDef::NoRetreat);
        }
    }
    let r = compile(p).unwrap();
    // Two eyes are worn (and one is in the backpack): one effect.
    let mut d = details(&r);
    let hero = derive_battle_hero(&d, &r).unwrap();
    let shackles = |h: &BattleHero| {
        h.effects
            .iter()
            .filter(|e| **e == BattleEffect::NoRetreat)
            .count()
    };
    assert_eq!(shackles(&hero), 1);
    d.worn.retain(|&a| a != art(&r, "t:eye"));
    assert_eq!(shackles(&derive_battle_hero(&d, &r).unwrap()), 0);
}

// covers: HER-BAT-007
#[test]
fn surrender_discounts_add_up_to_at_most_all() {
    let discount = |eye: u16, helm: u16| {
        let mut p = effect_pack();
        for a in &mut p.artifacts {
            let bp = match a.key.to_string().as_str() {
                "t:eye" => eye,
                "t:helm" => helm,
                _ => continue,
            };
            a.effects.push(HeroEffectDef::SurrenderDiscount { bp });
        }
        let r = compile(p).unwrap();
        derive_battle_hero(&details(&r), &r)
            .unwrap()
            .surrender_discount_bp
    };
    // The eye once although two are worn, plus the helm (a part of the worn circle): they add
    // (batch 1b R4: 20 % + 10 % + 10 % = 40 %).
    assert_eq!(discount(1000, 2000), 3000);
    assert_eq!(discount(9000, 2000), 10_000);
}

// covers: HER-BAT-004
#[test]
fn damage_bonuses_sum_skills_artifacts_and_specialty() {
    let r = damage_ruleset("t:archery", 500);
    let d = HeroDetails {
        level: 10,
        ..details(&r)
    };
    let bonuses = |ranged_bp, melee_bp, reduction_bp| DamageBonuses {
        ranged_bp,
        melee_bp,
        reduction_bp,
    };
    // Basic Archery 10 % grown by 5 % of itself per level (×1.5 at level 10) and the eye once
    // although two are worn; the specialty's own melee 1 %; the helm's 90 % (a part of the
    // worn circle). The loop needs Luck.
    let derive = |d: &HeroDetails| derive_battle_hero(d, &r).unwrap().damage;
    assert_eq!(derive(&d), bonuses(1500 + 500, 100, 9000));
    // Advanced Offense 20 % (no specialty for it); Expert Armorer's 15 % takes the reduction
    // past 100 %, so it stops there; with Luck the loop's 20 % counts.
    let mut more = d.clone();
    more.skills.extend([
        (skill(&r, "t:offense"), 2),
        (skill(&r, "t:armorer"), 3),
        (skill(&r, "t:luck"), 1),
    ]);
    assert_eq!(
        derive(&more),
        bonuses(1500 + 500 + 2000, 2000 + 100, 10_000)
    );
    // An Offense specialist: Archery does not grow now, the artifacts never do; a growth of
    // 655.35 % per level at level 3 gives 2000 + floor(2000 · 196 605 / 10 000) = 41 321; at
    // level 10 the sum saturates.
    let r = damage_ruleset("t:offense", 65_535);
    let fast = HeroDetails {
        level: 3,
        ..more.clone()
    };
    let grown = derive_battle_hero(&fast, &r).unwrap().damage;
    assert_eq!(grown, bonuses(1000 + 500 + 2000, 41_321 + 100, 10_000));
    let saturated = HeroDetails { level: 10, ..more };
    assert_eq!(
        derive_battle_hero(&saturated, &r).unwrap().damage.melee_bp,
        u16::MAX
    );
}

// covers: HER-BAT-005
#[test]
fn artillery_and_first_aid_make_the_hero_command_its_war_machines() {
    // Artillery: one shot and 50 %, two and 75 %, two and 100 % (the base pack's values);
    // First Aid: +100/200/300 % of the tent's largest heal. `t:kit` (worn) brings one shot at
    // 90 % and +50 % heal as an artifact could. The specialty grows First Aid by 5 % of itself
    // per level (an Artillery specialty would grow nothing; one specialty per skill only).
    let mut p = effect_pack();
    for s in &mut p.skills {
        match s.key.to_string().as_str() {
            "t:artillery" => {
                s.levels = [(1, 5000), (2, 7500), (2, 10_000)].map(|(shots, chance)| {
                    vec![HeroEffectDef::Artillery {
                        shots,
                        double_damage_chance_bp: chance,
                    }]
                });
            }
            "t:first_aid" => {
                s.levels = [10_000, 20_000, 30_000]
                    .map(|heal_bp| vec![HeroEffectDef::FirstAid { heal_bp }]);
            }
            _ => {}
        }
    }
    p.heroes[0].specialty.push(HeroEffectDef::SkillSpecialty {
        skill: key("t:first_aid"),
        bp_per_level: 500,
    });
    let without_kit = compile(p.clone()).unwrap();
    for a in &mut p.artifacts {
        if a.key.to_string() == "t:kit" {
            a.effects = vec![
                HeroEffectDef::Artillery {
                    shots: 1,
                    double_damage_chance_bp: 9000,
                },
                HeroEffectDef::FirstAid { heal_bp: 5000 },
            ];
        }
    }
    let r = compile(p).unwrap();
    let control = |r: &Ruleset, level: u8, artillery: Option<u8>, first_aid: Option<u8>| {
        let mut d = HeroDetails {
            level,
            ..details(r)
        };
        d.skills
            .extend(artillery.map(|l| (skill(r, "t:artillery"), l)));
        d.skills
            .extend(first_aid.map(|l| (skill(r, "t:first_aid"), l)));
        let machines = derive_battle_hero(&d, r).unwrap().war_machines;
        (
            machines
                .ballista
                .map(|b| (b.shots, b.double_damage_chance_bp)),
            machines.tent.map(|t| t.heal_bonus_bp),
        )
    };
    assert_eq!(control(&without_kit, 20, None, None), (None, None));
    assert_eq!(
        control(&without_kit, 20, Some(1), None),
        (Some((1, 5000)), None)
    );
    assert_eq!(control(&without_kit, 20, Some(2), None).0, Some((2, 7500)));
    assert_eq!(
        control(&without_kit, 20, Some(3), None).0,
        Some((2, 10_000))
    );
    // First Aid at level 1: +5 % of itself; Expert at level 10: 300 % × 1.5 = 450 % (the
    // executable's 1 + 3 × (1 + 0.05 × 10) = 5.5 times the tent's 25, 137).
    assert_eq!(
        control(&without_kit, 1, None, Some(1)),
        (None, Some(10_500))
    );
    assert_eq!(control(&without_kit, 10, None, Some(3)).1, Some(45_000));
    // With the kit: the most shots and the highest chance of all sources; heal bonuses add up,
    // the artifact's not grown.
    assert_eq!(control(&r, 20, None, None), (Some((1, 9000)), Some(5000)));
    assert_eq!(control(&r, 20, Some(2), None).0, Some((2, 9000)));
    assert_eq!(control(&r, 20, Some(3), None).0, Some((2, 10_000)));
    assert_eq!(control(&r, 10, None, Some(3)).1, Some(45_000 + 5000));
    // Far beyond `u16`: level 255.
    assert_eq!(
        control(&r, u8::MAX, None, Some(3)).1,
        Some(30_000 + 30_000 * 500 * 255 / 10_000 + 5000)
    );
}

/// The test pack plus stack statistics: `t:squire` (level 2, alive) upgrades into `t:knight`;
/// the combination `t:circle` +25 % hit points of living creatures, its parts `t:band` +1 hit
/// point and +1 speed for all and `t:helm` +5 attack for squires; `t:eye` +1 hit point for all;
/// `t:hook` −1 speed for all; the specialty a squire specialty (+5 % attack and defense per two
/// levels, +1 speed).
fn stat_ruleset() -> Ruleset {
    use h3_content::{CreatureFlag, CreatureSelector, CreatureStat};
    let all = CreatureSelector::All;
    let squires = CreatureSelector::Creature(key("t:squire"));
    let stat = |creatures, stat, amount| HeroEffectDef::CreatureStat {
        creatures,
        stat,
        amount,
    };
    let mut p = pack();
    let mut squire = creature("t:squire", &["alive"]);
    squire.info.level = 2;
    squire.info.upgrades = vec![key("t:knight")];
    let mut knight = creature("t:knight", &["alive"]);
    squire.speed = 5;
    knight.speed = 5;
    p.creatures.extend([squire, knight]);
    for a in &mut p.artifacts {
        a.effects = match a.key.to_string().as_str() {
            "t:circle" => vec![HeroEffectDef::CreatureStatPercent {
                creatures: CreatureSelector::Flag(CreatureFlag::Alive),
                stat: CreatureStat::HitPoints,
                bp: 2500,
            }],
            "t:band" => vec![
                stat(all.clone(), CreatureStat::HitPoints, 1),
                stat(all.clone(), CreatureStat::Speed, 1),
            ],
            "t:helm" => vec![stat(squires.clone(), CreatureStat::Attack, 5)],
            "t:eye" => vec![stat(all.clone(), CreatureStat::HitPoints, 1)],
            "t:hook" => vec![stat(all.clone(), CreatureStat::Speed, -1)],
            _ => continue,
        };
    }
    p.heroes[0].specialty = vec![HeroEffectDef::CreatureSpecialty {
        creature: key("t:squire"),
        bp_per_step: 500,
        speed: 1,
    }];
    compile(p).unwrap()
}

// covers: HER-BAT-008
#[test]
fn stack_statistics_are_summed_per_creatures_statistic_and_operation() {
    use h3_content::{CreatureFlag, CreatureSelector, CreatureStat};
    use h3_contracts::StatOp;
    let r = stat_ruleset();
    let squires = CreatureSelector::Creature(r.creature_by_key("t:squire").unwrap());
    let effect = |creatures, stat, op, amount| BattleEffect::StackStat {
        creatures,
        stat,
        op,
        amount,
    };
    let specialty = |bp| {
        [
            effect(squires, CreatureStat::Attack, StatOp::PercentBp, bp),
            effect(squires, CreatureStat::Defense, StatOp::PercentBp, bp),
            effect(squires, CreatureStat::Speed, StatOp::Add, 1),
        ]
    };
    // The circle's share; band and one eye +2 hit points (the second eye counts not); band and
    // hook cancel in speed; the helm's +5 attack. Level 5: two full steps of the squire's level
    // 2, +10 % attack and defense; the speed in any case.
    let mut d = details(&r);
    d.level = 5;
    let mut want = vec![
        effect(
            CreatureSelector::Flag(CreatureFlag::Alive),
            CreatureStat::HitPoints,
            StatOp::PercentBpDown,
            2500,
        ),
        effect(
            CreatureSelector::All,
            CreatureStat::HitPoints,
            StatOp::Add,
            2,
        ),
        effect(squires, CreatureStat::Attack, StatOp::Add, 5),
    ];
    want.extend(specialty(1000));
    assert_eq!(derive_battle_hero(&d, &r).unwrap().effects, want);
    // Level 1 is no full step: only the speed.
    d.level = 1;
    want.truncate(3);
    want.push(specialty(0)[2]);
    assert_eq!(derive_battle_hero(&d, &r).unwrap().effects, want);
    // At level 255, 127 steps of 5 % saturate at i16::MAX basis points; so does the largest
    // growth per step at level 1 of a level-1 creature, whose specialty without speed gives none.
    d.level = u8::MAX;
    let hero = derive_battle_hero(&d, &r).unwrap();
    assert_eq!(hero.effects.get(3..), Some(&specialty(i16::MAX)[..]));
    let mut p = pack();
    p.heroes[0].specialty = vec![HeroEffectDef::CreatureSpecialty {
        creature: key("t:peasant"),
        bp_per_step: u16::MAX,
        speed: 0,
    }];
    let r = compile(p).unwrap();
    let peasants = CreatureSelector::Creature(r.creature_by_key("t:peasant").unwrap());
    assert_eq!(
        derive_battle_hero(&details(&r), &r).unwrap().effects,
        [CreatureStat::Attack, CreatureStat::Defense].map(|stat| effect(
            peasants,
            stat,
            StatOp::PercentBp,
            i16::MAX
        ))
    );
}

// covers: HER-BAT-008
#[test]
fn more_battle_effects_than_the_seam_holds_are_an_error() {
    use h3_content::{CreatureFlag, CreatureSelector, CreatureStat};
    // 16 distinct statistic effects on each of the circle, its three parts and the eye: 80.
    let mut keys = CreatureFlag::ALL
        .into_iter()
        .flat_map(|flag| CreatureStat::ALL.map(|stat| (flag, stat)));
    let mut p = pack();
    for a in &mut p.artifacts {
        if ["t:circle", "t:band", "t:loop", "t:helm", "t:eye"].contains(&a.key.to_string().as_str())
        {
            a.effects = keys
                .by_ref()
                .take(16)
                .map(|(flag, stat)| HeroEffectDef::CreatureStat {
                    creatures: CreatureSelector::Flag(flag),
                    stat,
                    amount: 1,
                })
                .collect();
        }
    }
    let r = compile(p).unwrap();
    let error = derive_battle_hero(&details(&r), &r).unwrap_err();
    assert_eq!(error, DetailsError::TooManyBattleEffects);
    assert_eq!(error.to_string(), "too many battle effects");
    // Without the eye's 16: 64, the limit.
    let mut d = details(&r);
    d.worn.retain(|&a| a != art(&r, "t:eye"));
    assert_eq!(derive_battle_hero(&d, &r).unwrap().effects.len(), 64);
}

proptest! {
    // covers: HER-BAT-001, HER-BAT-002, HER-BAT-003, HER-BAT-004
    #[test]
    fn derivation_succeeds_exactly_for_valid_details_and_stays_in_limits(
        worn in proptest::collection::vec(0u16..10, 0..8),
        skills in proptest::collection::vec((0u16..10, 1u8..=3), 0..3),
        primary in proptest::array::uniform4(0u8..=100),
    ) {
        let r = morale_ruleset();
        let d = HeroDetails {
            primary,
            skills: skills.iter().map(|&(s, l)| (SkillId::from_raw(s), l)).collect(),
            worn: worn.iter().map(|&a| ArtifactTypeId::from_raw(a)).collect(),
            ..details(&r)
        };
        let derived = derive_battle_hero(&d, &r);
        prop_assert_eq!(derived.is_ok(), validate(&d, &r).is_ok());
        if let Ok(hero) = derived {
            let values = [hero.attack, hero.defense, hero.power, hero.knowledge];
            for (value, min) in values.into_iter().zip([0, 0, 1, 1]) {
                prop_assert!((min..=99).contains(&value));
            }
            prop_assert!(hero.tactics <= 3);
            prop_assert!(hero.damage.reduction_bp <= 10_000);
            // At most one effect per morale/luck kind and affected sides (HER-BAT-003).
            prop_assert!(hero.effects.len() <= 12);
        }
    }
}
