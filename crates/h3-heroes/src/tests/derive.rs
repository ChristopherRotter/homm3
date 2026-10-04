//! `derive_battle_hero` (HER-BAT-001, HER-BAT-002, HER-BAT-003).

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
    assert_eq!(
        derive_battle_hero(&d, &r),
        Ok(expected(&r, [99, 5, 1, 1], 3, u16::MAX))
    );

    // With Luck (any level) the loop counts: +10 each.
    let mut lucky = d.clone();
    lucky.skills.push((skill(&r, "t:luck"), 1));
    assert_eq!(
        derive_battle_hero(&lucky, &r),
        Ok(expected(&r, [99, 15, 10, 10], 3, u16::MAX))
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
    assert_eq!(hero.damage, DamageBonuses::default());
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

proptest! {
    // covers: HER-BAT-001, HER-BAT-002, HER-BAT-003
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
            // At most one effect per morale/luck kind and affected sides (HER-BAT-003).
            prop_assert!(hero.effects.len() <= 12);
        }
    }
}
