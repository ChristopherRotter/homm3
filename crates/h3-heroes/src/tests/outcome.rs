//! Tests of what a won battle gives a hero beyond its experience (E08 T19a2b): its Necromancy
//! (the share and the creature raised) and the spells its Eagle Eye picks that it learns.

use std::vec;

use h3_content::{CreatureTypeId, HeroEffectDef, RaisedCreature, SpellCategory, SpellDef};
use h3_contracts::Rate;
use h3_kernel::arith::{SINGLE_BITS, X87};

use super::*;

/// The test pack with Necromancy and Wisdom: First Aid stands for Necromancy (10/20/30 %, and the
/// bones raised at priority 0), Wisdom adds 1/2/3 spell levels; the combination `t:circle`'s
/// parts `t:band`, `t:loop` and `t:helm` add 5/10/15 % with the skill, the combination itself
/// 30 % without it and raises by the skill's level (peasants, ghouls, wraiths, liches) at
/// priority 1; `t:book` gives a spell book; spells of levels 3 and 5.
fn outcome_pack() -> ContentPack {
    let mut p = pack();
    let necromancy = Some(key("t:first_aid"));
    for s in &mut p.skills {
        match s.key.to_string().as_str() {
            "t:first_aid" => {
                s.levels = [1000, 2000, 3000].map(|bp| {
                    vec![
                        HeroEffectDef::Necromancy {
                            bp,
                            unless_skill: None,
                        },
                        HeroEffectDef::Raises {
                            priority: 0,
                            creature: RaisedCreature::Creature(key("t:bones")),
                        },
                    ]
                });
            }
            "t:wisdom" => {
                s.levels = [1, 2, 3].map(|levels| vec![HeroEffectDef::LearnLevel { levels }]);
            }
            _ => {}
        }
    }
    for a in &mut p.artifacts {
        let share = |bp| HeroEffectDef::Necromancy {
            bp,
            unless_skill: None,
        };
        match a.key.to_string().as_str() {
            "t:band" | "t:loop" | "t:helm" => {
                let bp = match a.key.to_string().as_str() {
                    "t:band" => 500,
                    "t:loop" => 1000,
                    _ => 1500,
                };
                a.needs_skill.clone_from(&necromancy);
                a.effects = vec![share(bp)];
            }
            "t:circle" => {
                a.effects = vec![
                    HeroEffectDef::Necromancy {
                        bp: 3000,
                        unless_skill: necromancy.clone(),
                    },
                    HeroEffectDef::Raises {
                        priority: 1,
                        creature: RaisedCreature::ByLevel {
                            skill: key("t:first_aid"),
                            by_level: ["t:peasant", "t:ghoul", "t:wraith", "t:lich"].map(key),
                        },
                    },
                ];
            }
            "t:book" => a.effects = vec![HeroEffect::SpellBook],
            _ => {}
        }
    }
    for k in ["t:bones", "t:ghoul", "t:wraith", "t:lich"] {
        p.creatures.push(creature(k, &[]));
    }
    p.spells.push(SpellDef {
        level: 3,
        ..spell("t:shield", SpellCategory::Combat)
    });
    p.spells.push(SpellDef {
        level: 5,
        ..spell("t:storm", SpellCategory::Combat)
    });
    p
}

fn outcome_ruleset() -> Ruleset {
    compile(outcome_pack()).unwrap()
}

/// Details without skills, artifacts or spells; the hero at level 1.
fn bare(r: &Ruleset) -> HeroDetails {
    HeroDetails {
        skills: vec![],
        worn: vec![],
        backpack: vec![],
        spells: vec![],
        ..details(r)
    }
}

fn creature_id(r: &Ruleset, k: &str) -> CreatureTypeId {
    r.creature_by_key(k).unwrap()
}

// covers: HER-NEC-001
#[test]
fn the_necromancy_share_adds_the_skill_and_the_artifacts() {
    let r = outcome_ruleset();
    let mut d = bare(&r);
    let share = |d: &HeroDetails, r: &Ruleset| necromancy(d, r).map(|n| n.share.bp);
    // Nothing without a source.
    assert_eq!(necromancy(&d, &r), None);
    // The skill's share by level.
    let skill_id = skill(&r, "t:first_aid");
    for (level, bp) in [(1, 1000), (2, 2000), (3, 3000)] {
        d.skills = vec![(skill_id, level)];
        assert_eq!(share(&d, &r), Some(bp));
    }
    // The parts add with the skill, worn alone or through the combination; the combination's own
    // 30 % does not count with the skill.
    d.worn = vec![art(&r, "t:band"), art(&r, "t:loop")];
    assert_eq!(share(&d, &r), Some(4500));
    d.worn = vec![art(&r, "t:circle")];
    assert_eq!(share(&d, &r), Some(6000));
    // Without the skill the parts give nothing and the combination its own 30 %.
    d.skills.clear();
    assert_eq!(share(&d, &r), Some(3000));
    d.worn = vec![art(&r, "t:band"), art(&r, "t:helm")];
    assert_eq!(necromancy(&d, &r), None);
    // A specialist's skill share grows by 5 % of itself per level; the share is at most 100 %.
    let mut p = outcome_pack();
    p.heroes[0].specialty = vec![HeroEffectDef::SkillSpecialty {
        skill: key("t:first_aid"),
        bp_per_level: 500,
    }];
    let r = compile(p).unwrap();
    d.skills = vec![(skill_id, 3)];
    d.worn = vec![art(&r, "t:circle")];
    d.level = 10;
    assert_eq!(share(&d, &r), Some(3000 + 1500 + 3000));
    d.level = 40;
    assert_eq!(share(&d, &r), Some(10_000));
}

// covers: HER-NEC-001
#[test]
fn with_quirk_q0005_the_share_is_the_originals_single_precision_sum() {
    let r = Ruleset::compile(&[outcome_pack()], "t:original").unwrap();
    let mut d = bare(&r);
    d.skills = vec![(skill(&r, "t:first_aid"), 3)];
    d.worn = vec![art(&r, "t:circle")];
    // 0.3 + 0.05 + 0.10 + 0.15 in single precision, each sum stored: f32(0.6) again.
    let n = necromancy(&d, &r).unwrap();
    assert_eq!(n.share.bp, 6000);
    let bits = n.share.single.map_or_else(
        || X87::ratio(6000, 10_000, SINGLE_BITS).unwrap().to_f32_bits(),
        core::num::NonZeroU32::get,
    );
    assert_eq!(bits, 0x3f19_999a);
    // Without the quirk the share carries no single-precision value of its own.
    let r = outcome_ruleset();
    assert_eq!(necromancy(&d, &r).unwrap().share, Rate::bp(6000));
}

// covers: HER-NEC-002
#[test]
fn the_creature_raised_is_the_highest_priority_by_level() {
    let r = outcome_ruleset();
    let mut d = bare(&r);
    let raised = |d: &HeroDetails, r: &Ruleset| necromancy(d, r).map(|n| n.creature);
    let necro = skill(&r, "t:first_aid");
    d.skills = vec![(necro, 1)];
    assert_eq!(raised(&d, &r), Some(creature_id(&r, "t:bones")));
    // The combination's priority 1 wins: by the skill's level, none to Expert.
    d.worn = vec![art(&r, "t:circle")];
    for (level, k) in [(1, "t:ghoul"), (2, "t:wraith"), (3, "t:lich")] {
        d.skills = vec![(necro, level)];
        assert_eq!(raised(&d, &r), Some(creature_id(&r, k)), "{level}");
    }
    d.skills.clear();
    assert_eq!(raised(&d, &r), Some(creature_id(&r, "t:peasant")));
    // A share without a creature raises nothing.
    let mut p = outcome_pack();
    for s in &mut p.skills {
        if s.key.to_string() == "t:first_aid" {
            for level in &mut s.levels {
                level.truncate(1);
            }
        }
    }
    let r = compile(p).unwrap();
    d.worn.clear();
    d.skills = vec![(necro, 1)];
    assert_eq!(necromancy(&d, &r), None);
}

// covers: HER-NEC-002
#[test]
fn equal_priorities_pick_the_lowest_artifact_then_the_lowest_creature() {
    // `t:eye` and `t:hook` (in artifact order: hook after eye) raise at priority 2; the skill at 2
    // too when `skill_ties`.
    let pack_with = |skill_ties: bool| {
        let mut p = outcome_pack();
        let raises = |k: &str| HeroEffectDef::Raises {
            priority: 2,
            creature: RaisedCreature::Creature(key(k)),
        };
        for a in &mut p.artifacts {
            match a.key.to_string().as_str() {
                "t:eye" => a.effects = vec![raises("t:wraith")],
                "t:hook" => a.effects = vec![raises("t:ghoul")],
                _ => {}
            }
        }
        if skill_ties {
            for s in &mut p.skills {
                if s.key.to_string() == "t:first_aid" {
                    s.levels[0][1] = raises("t:lich");
                }
            }
        }
        compile(p).unwrap()
    };
    let r = pack_with(false);
    let mut d = bare(&r);
    d.skills = vec![(skill(&r, "t:first_aid"), 1)];
    d.worn = vec![art(&r, "t:hook"), art(&r, "t:eye")];
    // Only artifacts at the top: the lowest artifact id, whatever the creatures' ids.
    assert_eq!(
        necromancy(&d, &r).map(|n| n.creature),
        Some(creature_id(&r, "t:wraith"))
    );
    // With the skill among them: the lowest creature id.
    let r = pack_with(true);
    assert_eq!(
        necromancy(&d, &r).map(|n| n.creature),
        Some(creature_id(&r, "t:ghoul"))
    );
}

// covers: HER-SPL-005
#[test]
fn wisdom_raises_the_highest_spell_level_learnable() {
    let r = outcome_ruleset();
    let mut d = bare(&r);
    assert_eq!(learn_level(&d, &r), 2);
    let wisdom = skill(&r, "t:wisdom");
    for (level, highest) in [(1, 3), (2, 4), (3, 5)] {
        d.skills = vec![(wisdom, level)];
        assert_eq!(learn_level(&d, &r), highest);
    }
    // At most level 5, and 0 plus the effects without hero parameters.
    let mut p = outcome_pack();
    p.hero_params.as_mut().unwrap().learn_level = 4;
    assert_eq!(learn_level(&d, &compile(p).unwrap()), 5);
    let mut p = outcome_pack();
    p.hero_params = None;
    assert_eq!(learn_level(&d, &compile(p).unwrap()), 3);
}

// covers: HER-SPL-006
#[test]
fn a_pick_is_learned_with_a_spell_book_up_to_the_learnable_level() {
    let r = outcome_ruleset();
    let mut d = bare(&r);
    let (bolt, shield, storm) = (
        spell_id(&r, "t:bolt"),
        spell_id(&r, "t:shield"),
        spell_id(&r, "t:storm"),
    );
    // No spell book worn: nothing (one in the backpack does not count).
    d.backpack = vec![art(&r, "t:book")];
    assert!(!learns_spell(&d, &r, bolt));
    d.backpack.clear();
    d.worn = vec![art(&r, "t:book")];
    assert!(learns_spell(&d, &r, bolt));
    // Level 3 needs Basic Wisdom, level 5 Expert.
    assert!(!learns_spell(&d, &r, shield));
    let wisdom = skill(&r, "t:wisdom");
    d.skills = vec![(wisdom, 1)];
    assert!(learns_spell(&d, &r, shield));
    assert!(!learns_spell(&d, &r, storm));
    d.skills = vec![(wisdom, 3)];
    assert!(learns_spell(&d, &r, storm));
    // A spell it has learned, a creature ability and one the ruleset lacks are not.
    d.spells = vec![bolt];
    assert!(!learns_spell(&d, &r, bolt));
    assert!(!learns_spell(&d, &r, spell_id(&r, "t:gaze")));
    assert!(!learns_spell(&d, &r, SpellId::from_raw(99)));
}
