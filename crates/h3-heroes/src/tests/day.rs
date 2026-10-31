//! What a hero brings to the day's start (GAM-RES-004, HER-MAN-002; E08 T82): its income and
//! its day's spell points.

use h3_content::{HeroEffectDef, MAX_DAILY_AMOUNT};

use super::*;

/// The test pack with the day's kinds (E08 T82): crystal and gems beside gold; `t:estates` 125,
/// 250, 500 gold a day and `t:mysticism` a regain of 2, 3, 4; `t:helm` the Purse's 500 gold and
/// the Wizard's Well's full mana, `t:band` a gem (the Ring of Infinite Gems') and an extra regain
/// of 2 (the Talisman's), `t:eye` the Charm's extra regain of 1, `t:loop` (needs Luck) a crystal;
/// the combination `t:circle` (band, loop, helm) 4 gems of its own (the Cornucopia's shape: its
/// parts add theirs); `t:hook` a regain of 3 (no pack's: an artifact source); `t:kit` (misc in
/// this pack) the treasury's bound of gold. `specialty` is Orrin's.
fn day_pack(specialty: Vec<HeroEffectDef>) -> ContentPack {
    let mut p = pack();
    for k in ["t:crystal", "t:gems"] {
        p.resources.push(ResourceDef {
            key: key(k),
            name: "Resource".into(),
        });
    }
    let daily = |k: &str, amount: u32| HeroEffectDef::DailyResource {
        resource: key(k),
        amount,
    };
    p.skills.push(SecondarySkillDef {
        key: key("t:estates"),
        name: "Skill".into(),
        levels: [125, 250, 500].map(|gold| vec![daily("t:gold", gold)]),
    });
    p.skills.push(SecondarySkillDef {
        key: key("t:mysticism"),
        name: "Skill".into(),
        levels: [2, 3, 4].map(|points| vec![HeroEffectDef::ManaRegain { points }]),
    });
    for a in &mut p.artifacts {
        a.effects = match a.key.to_string().as_str() {
            "t:helm" => vec![daily("t:gold", 500), HeroEffectDef::FullMana],
            "t:band" => vec![
                daily("t:gems", 1),
                HeroEffectDef::ExtraManaRegain { points: 2 },
            ],
            "t:eye" => vec![HeroEffectDef::ExtraManaRegain { points: 1 }],
            "t:loop" => {
                a.needs_skill = Some(key("t:luck"));
                vec![daily("t:crystal", 1)]
            }
            "t:circle" => vec![daily("t:gems", 4)],
            "t:hook" => vec![HeroEffectDef::ManaRegain { points: 3 }],
            "t:kit" => {
                a.slot = Some(key("t:misc"));
                a.war_machine = None;
                vec![daily("t:gold", MAX_DAILY_AMOUNT)]
            }
            _ => continue,
        };
    }
    p.heroes[0].specialty = specialty;
    p
}

/// A skill specialist's specialty: the skill's values grow by 5 % of themselves per level.
fn specialist(skill: &str) -> Vec<HeroEffectDef> {
    vec![HeroEffectDef::SkillSpecialty {
        skill: key(skill),
        bp_per_level: 500,
    }]
}

/// Orrin at `level` with Knowledge `knowledge`, `mana` spell points, `skills` (by key and level),
/// wearing `worn`; a Purse in the backpack (never counted).
fn hero(
    r: &Ruleset,
    level: u8,
    knowledge: u8,
    mana: u16,
    skills: &[(&str, u8)],
    worn: &[&str],
) -> HeroDetails {
    HeroDetails {
        level,
        experience: 0,
        primary: [0, 0, 1, knowledge],
        skills: skills.iter().map(|&(k, l)| (skill(r, k), l)).collect(),
        worn: wearing(r, &worn.iter().map(|k| art(r, k)).collect::<Vec<_>>()),
        backpack: vec![art(r, "t:helm")],
        spells: vec![],
        mana,
        ..details(r)
    }
}

/// The faithful and the original (quirk Q-0005) rulesets of a pack: both give exact integers.
fn both(p: ContentPack) -> [Ruleset; 2] {
    let original = Ruleset::compile(core::slice::from_ref(&p), "t:original").unwrap();
    [compile(p).unwrap(), original]
}

/// An income by the test pack's resources: gold, crystal, gems.
fn income(gold: u32, crystal: u32, gems: u32) -> Vec<u32> {
    vec![gold, crystal, gems]
}

// covers: GAM-RES-004
#[test]
fn the_income_counts_every_worn_copy_and_a_combinations_parts() {
    for r in both(day_pack(vec![])) {
        let of = |skills: &[(&str, u8)], worn: &[&str]| {
            daily_income(&hero(&r, 10, 1, 0, skills, worn), &r)
        };
        // Nothing worn, no skill: nothing, one entry per resource of the core.
        assert_eq!(of(&[], &[]), income(0, 0, 0));
        // The Purse's 500; the backpack's copy not.
        assert_eq!(of(&[], &["t:helm"]), income(500, 0, 0));
        // Two Rings of Infinite Gems: 2 gems (each copy, unlike the movement's types).
        assert_eq!(of(&[], &["t:band"]), income(0, 0, 1));
        assert_eq!(of(&[], &["t:band", "t:band"]), income(0, 0, 2));
        // The combination's own 4 and its parts' (the Cornucopia's 5, batch E9's E9-7); a part
        // that needs a skill only with it.
        assert_eq!(of(&[], &["t:circle"]), income(500, 0, 5));
        assert_eq!(of(&[("t:luck", 1)], &["t:circle"]), income(500, 1, 5));
        assert_eq!(of(&[], &["t:loop"]), income(0, 0, 0));
        // Estates by level, not grown without a specialty, beside the artifacts.
        for (level, gold) in [(1, 125), (2, 250), (3, 500)] {
            assert_eq!(of(&[("t:estates", level)], &[]), income(gold, 0, 0));
        }
        assert_eq!(
            of(&[("t:estates", 1)], &["t:helm", "t:band"]),
            income(625, 0, 1)
        );
        // Saturating per resource: three copies of the treasury's bound.
        assert_eq!(
            of(&[], &["t:kit", "t:kit", "t:kit"]),
            income(u32::MAX, 0, 0)
        );
    }
}

// covers: GAM-RES-004
#[test]
fn an_estates_specialist_grows_the_skills_gold_in_exact_integers() {
    for r in both(day_pack(specialist("t:estates"))) {
        let of = |level: u8, skills: &[(&str, u8)]| {
            daily_income(&hero(&r, level, 1, 0, skills, &["t:helm"]), &r)[0]
        };
        // ⌊125 × (1 + 0.05 × level)⌋ beside the Purse's 500 (never grown): 131 at level 1,
        // 187 at level 10 (187.5), 250 at level 20 (exact: no single-precision term).
        assert_eq!(of(1, &[("t:estates", 1)]), 631);
        assert_eq!(of(10, &[("t:estates", 1)]), 687);
        assert_eq!(of(20, &[("t:estates", 1)]), 750);
        assert_eq!(of(20, &[("t:estates", 3)]), 1500);
        // Without the skill the specialty grows nothing.
        assert_eq!(of(20, &[]), 500);
    }
    // A specialty on another skill grows nothing.
    let r = compile(day_pack(specialist("t:logistics"))).unwrap();
    let other = hero(&r, 20, 1, 0, &[("t:estates", 1), ("t:logistics", 1)], &[]);
    assert_eq!(daily_income(&other, &r), income(125, 0, 0));
}

// covers: GAM-RES-004
#[test]
fn a_resource_or_gold_specialty_adds_its_amount_ungrown() {
    let crystal = vec![HeroEffectDef::DailyResource {
        resource: key("t:crystal"),
        amount: 1,
    }];
    let gold = vec![HeroEffectDef::DailyResource {
        resource: key("t:gold"),
        amount: 350,
    }];
    // Batch E9's E9-7: the crystal specialist's 1 a day.
    for (specialty, expected) in [(crystal, income(0, 1, 0)), (gold, income(350, 0, 0))] {
        for r in both(day_pack(specialty.clone())) {
            for level in [1, 30] {
                assert_eq!(daily_income(&hero(&r, level, 1, 0, &[], &[]), &r), expected);
            }
        }
    }
}

// covers: HER-MAN-002
#[test]
fn the_day_regains_the_base_or_the_highest_regain_up_to_the_maximum() {
    for r in both(day_pack(vec![])) {
        // Knowledge 1: a maximum of 10.
        let points = |mana: u16, skills: &[(&str, u8)], worn: &[&str]| {
            day_mana(&hero(&r, 10, 1, mana, skills, worn), &r)
        };
        // The base 1 without a source; at most the maximum; points above it stay.
        assert_eq!(points(0, &[], &[]), 1);
        assert_eq!(points(9, &[], &[]), 10);
        assert_eq!(points(10, &[], &[]), 10);
        assert_eq!(points(15, &[], &[]), 15);
        // Another skill's effects regain nothing.
        assert_eq!(points(0, &[("t:estates", 3)], &[]), 1);
        // Mysticism by level instead of the base.
        for (level, regain) in [(1, 2), (2, 3), (3, 4)] {
            assert_eq!(points(0, &[("t:mysticism", level)], &[]), regain);
        }
        assert_eq!(points(9, &[("t:mysticism", 3)], &[]), 10);
        // Several sources: the highest, not their sum.
        assert_eq!(points(0, &[("t:mysticism", 1)], &["t:hook"]), 3);
        assert_eq!(points(0, &[("t:mysticism", 3)], &["t:hook"]), 4);
        assert_eq!(points(0, &[], &["t:hook"]), 3);
    }
}

// covers: HER-MAN-002
#[test]
fn the_extra_regains_add_once_per_type_and_the_well_fills() {
    for r in both(day_pack(vec![])) {
        let points = |mana: u16, skills: &[(&str, u8)], worn: &[&str]| {
            day_mana(&hero(&r, 10, 5, mana, skills, worn), &r)
        };
        // The Charm 1 and the Talisman 2 on top of the regain; two Talismans count once.
        assert_eq!(points(0, &[], &["t:eye"]), 2);
        assert_eq!(points(0, &[], &["t:band", "t:band"]), 3);
        assert_eq!(points(0, &[("t:mysticism", 3)], &["t:eye", "t:band"]), 7);
        // At most the maximum (50), never lowered.
        assert_eq!(points(48, &[], &["t:eye", "t:band"]), 50);
        assert_eq!(points(60, &[], &["t:eye", "t:band"]), 60);
        // The Well fills to the maximum, worn or inside a worn combination; above it, kept.
        assert_eq!(points(0, &[], &["t:helm"]), 50);
        assert_eq!(points(0, &[], &["t:circle"]), 50);
        assert_eq!(points(60, &[], &["t:helm"]), 60);
        // The Well in the backpack (every test hero carries one) fills nothing.
        assert_eq!(points(0, &[], &[]), 1);
    }
}

// covers: HER-MAN-002
#[test]
fn a_mysticism_specialist_grows_the_skills_regain_plus_the_bonus_only_with_it() {
    for r in both(day_pack(specialist("t:mysticism"))) {
        let points = |level: u8, skills: &[(&str, u8)], worn: &[&str]| {
            day_mana(&hero(&r, level, 50, 0, skills, worn), &r)
        };
        // ⌊2 × (1 + 0.05 × level)⌋ + 1: 3 at level 1, 4 at level 10 (3 + 1), 5 at level 20.
        assert_eq!(points(1, &[("t:mysticism", 1)], &[]), 3);
        assert_eq!(points(10, &[("t:mysticism", 1)], &[]), 4);
        assert_eq!(points(20, &[("t:mysticism", 1)], &[]), 5);
        // Expert at level 15: ⌊4 × 1.75⌋ + 1 = 8; the extras after it.
        assert_eq!(points(15, &[("t:mysticism", 3)], &[]), 8);
        assert_eq!(points(15, &[("t:mysticism", 3)], &["t:eye"]), 9);
        // Without the skill: the base, no growth and no bonus.
        assert_eq!(points(30, &[], &[]), 1);
        // A skill wins a tie with an artifact's regain; a higher artifact's is not grown.
        assert_eq!(points(1, &[("t:mysticism", 2)], &["t:hook"]), 4);
        assert_eq!(points(30, &[("t:mysticism", 1)], &["t:hook"]), 3);
    }
    // A specialty on another skill grows nothing.
    let r = compile(day_pack(specialist("t:logistics"))).unwrap();
    let other = hero(
        &r,
        30,
        50,
        0,
        &[("t:mysticism", 1), ("t:logistics", 1)],
        &[],
    );
    assert_eq!(day_mana(&other, &r), 2);
}

// covers: HER-MAN-002
#[test]
fn batch_e9s_heroes_regain_as_observed() {
    // E9-12: three heroes of Knowledge 2 at 0 points: Basic Mysticism 0, 2, 4; with the Charm 0,
    // 3, 6; the Wizard's Well 0, 20, 20.
    for r in both(day_pack(vec![])) {
        let days = |skills: &[(&str, u8)], worn: &[&str]| {
            let mut d = hero(&r, 1, 2, 0, skills, worn);
            let mut seen = vec![d.mana];
            for _ in 0..2 {
                d.mana = day_mana(&d, &r);
                seen.push(d.mana);
            }
            seen
        };
        assert_eq!(days(&[("t:mysticism", 1)], &[]), [0, 2, 4]);
        assert_eq!(days(&[("t:mysticism", 1)], &["t:eye"]), [0, 3, 6]);
        assert_eq!(days(&[], &["t:helm"]), [0, 20, 20]);
    }
}

// covers: HER-MAN-002
#[test]
fn the_regain_saturates_and_a_ruleset_without_hero_parameters_regains_nothing() {
    let r = compile(day_pack(vec![])).unwrap();
    let full = hero(
        &r,
        1,
        99,
        u16::MAX - 1,
        &[("t:mysticism", 3)],
        &["t:eye", "t:band"],
    );
    assert_eq!(day_mana(&full, &r), u16::MAX - 1);
    // Without hero parameters (which a hero's validation refuses) neither base nor bonus.
    let mut p = day_pack(specialist("t:mysticism"));
    p.hero_params = None;
    let bare = compile(p).unwrap();
    assert_eq!(day_mana(&hero(&r, 1, 1, 0, &[], &[]), &bare), 0);
    assert_eq!(
        day_mana(&hero(&r, 20, 1, 0, &[("t:mysticism", 1)], &[]), &bare),
        4
    );
}
