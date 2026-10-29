//! What the adventure map reads to move and scout (HER-ADV-003 to HER-ADV-007, E08 T39).

use std::format;
use std::vec;

use h3_content::{ContentPack, CreatureDef, CreatureInfoDef, HeroEffectDef, Ruleset};

use super::*;

/// The test pack with movement and scouting (E08 T39): Logistics 10/20/30 %, Pathfinding 1/2/3
/// and Scouting 1/2/3 tiles; `t:helm` +600 land points (Boots of Speed's), reached as a part of
/// the combination `t:circle` too, `t:eye` +300 (Equestrian's Gloves') and a radius tile (worn
/// twice in some tests: counted once), `t:band` a radius tile, `t:loop` (needs Luck) flying 3
/// and Pathfinding 3, `t:hook` water walking 3 and `t:kit` water walking 0; `t:archer` (speed 4)
/// upgrades into `t:marksman` (6), whose own upgrade `t:sharpshooter` (7) is not a direct one of
/// the archer's, `t:quick` has speed 255. `specialty` is Orrin's.
fn movement_pack(specialty: Vec<HeroEffectDef>) -> ContentPack {
    let mut p = pack();
    for s in &mut p.skills {
        if s.key.to_string() == "t:logistics" {
            s.levels = [1000, 2000, 3000].map(|bp| vec![HeroEffectDef::LandMovement { bp }]);
        }
    }
    let skill = |k: &str, levels: [Vec<HeroEffectDef>; 3]| SecondarySkillDef {
        key: key(k),
        name: "Skill".into(),
        levels,
    };
    p.skills.push(skill(
        "t:pathfinding",
        [1, 2, 3].map(|level| vec![HeroEffectDef::Pathfinding { level }]),
    ));
    p.skills.push(skill(
        "t:scouting",
        [1, 2, 3].map(|tiles| vec![HeroEffectDef::ScoutingRadius { tiles }]),
    ));
    for a in &mut p.artifacts {
        a.effects = match a.key.to_string().as_str() {
            "t:helm" => vec![HeroEffectDef::LandMovementPoints { points: 600 }],
            "t:eye" => vec![
                HeroEffectDef::LandMovementPoints { points: 300 },
                HeroEffectDef::ScoutingRadius { tiles: 1 },
            ],
            "t:band" => vec![HeroEffectDef::ScoutingRadius { tiles: 1 }],
            "t:loop" => {
                a.needs_skill = Some(key("t:luck"));
                vec![
                    HeroEffectDef::Flying { level: 3 },
                    HeroEffectDef::Pathfinding { level: 3 },
                ]
            }
            "t:hook" => vec![HeroEffectDef::WaterWalking { level: 3 }],
            "t:kit" => vec![HeroEffectDef::WaterWalking { level: 0 }],
            _ => continue,
        };
    }
    let creature = |k: &str, speed: u8, upgrades: &[&str]| CreatureDef {
        key: key(k),
        speed,
        info: CreatureInfoDef {
            level: 1,
            upgrades: upgrades.iter().map(|u| key(u)).collect(),
            ..CreatureInfoDef::default()
        },
    };
    p.creatures.extend([
        creature("t:archer", 4, &["t:marksman"]),
        creature("t:marksman", 6, &["t:sharpshooter"]),
        creature("t:sharpshooter", 7, &[]),
        creature("t:quick", 255, &[]),
    ]);
    p.heroes[0].specialty = specialty;
    p
}

/// A Logistics specialist's specialty: the skill's rates grow by 5 % of themselves per level.
fn logistics_specialist() -> Vec<HeroEffectDef> {
    vec![HeroEffectDef::SkillSpecialty {
        skill: key("t:logistics"),
        bp_per_level: 500,
    }]
}

/// Orrin at `level` with `skills` (by key and level), wearing `worn`.
fn hero(r: &Ruleset, level: u8, skills: &[(&str, u8)], worn: &[&str]) -> HeroDetails {
    HeroDetails {
        level,
        experience: 0,
        primary: [0, 0, 1, 1],
        skills: skills.iter().map(|&(k, l)| (skill(r, k), l)).collect(),
        worn: wearing(r, &worn.iter().map(|k| art(r, k)).collect::<Vec<_>>()),
        backpack: vec![art(r, "t:helm")],
        spells: vec![],
        ..details(r)
    }
}

/// The faithful and the original (quirk Q-0005) rulesets of a pack.
fn both(p: ContentPack) -> [Ruleset; 2] {
    [compile(p.clone()).unwrap(), original(p)]
}

fn original(p: ContentPack) -> Ruleset {
    Ruleset::compile(&[p], "t:original").unwrap()
}

/// The original's land table (`movement.txt`, ADV-MOV-001), the bases of the day's points.
const LAND_TABLE: [u32; 12] = [
    1300, 1360, 1430, 1500, 1560, 1630, 1700, 1760, 1830, 1900, 1960, 2000,
];

// covers: HER-ADV-003
#[test]
fn the_land_points_grow_by_logistics_without_a_specialty_alike_in_both_arithmetics() {
    // Survey §1.3's last sentence: without a specialty the f32 rates give exactly
    // ⌊points × (10 + level) / 10⌋, here for every base up to 10 000, the table's included.
    for r in both(movement_pack(vec![])) {
        let none = hero(&r, 1, &[], &[]);
        let heroes = [1, 2, 3].map(|level| (level, hero(&r, 1, &[("t:logistics", level)], &[])));
        for base in (1..=10_000).chain(LAND_TABLE) {
            assert_eq!(land_points(&none, &r, base), base);
            for (level, d) in &heroes {
                let exact = base * (10 + u32::from(*level)) / 10;
                assert_eq!(land_points(d, &r, base), exact, "{base} at {level}");
            }
        }
    }
}

// covers: HER-ADV-003
#[test]
fn a_logistics_specialists_points_follow_quirk_q0005() {
    let [exact, single] = both(movement_pack(logistics_specialist()));
    // Survey §1.3 (the table's 1500, Basic) and batch E5's E5-2 (Advanced at levels 2 and 6 in
    // single precision: 1829 and 1889, observed; exact 1830 and 1890), with `e4_score`'s cases.
    let cases: &[(u32, u8, u8, u32, u32)] = &[
        (1500, 1, 1, 1657, 1657),
        (1500, 2, 1, 1664, 1665),
        (1500, 3, 1, 1672, 1672),
        (1500, 6, 1, 1694, 1695),
        (1500, 12, 1, 1739, 1740),
        (1500, 20, 1, 1800, 1800),
        (1500, 2, 2, 1829, 1830),
        (1500, 3, 2, 1845, 1845),
        (1500, 6, 2, 1889, 1890),
        (1500, 9, 2, 1934, 1935),
        (1500, 12, 3, 2220, 2220),
        // The table's 2000 (speed 11 and up), Basic: the survey's second table.
        (2000, 1, 1, 2210, 2210),
        (2000, 2, 1, 2219, 2220),
        (2000, 3, 1, 2230, 2230),
        (2000, 6, 1, 2259, 2260),
        (2000, 9, 1, 2289, 2290),
        (2000, 12, 1, 2319, 2320),
    ];
    for &(base, level, logistics, original, strict) in cases {
        let d = hero(&exact, level, &[("t:logistics", logistics)], &[]);
        let at = || format!("{base} at level {level}, Logistics {logistics}");
        assert_eq!(land_points(&d, &single, base), original, "{}", at());
        assert_eq!(land_points(&d, &exact, base), strict, "{}", at());
    }
    // Without the skill the specialty grows nothing.
    let d = hero(&exact, 12, &[], &[]);
    assert_eq!(land_points(&d, &single, 1500), 1500);
    assert_eq!(land_points(&d, &exact, 1500), 1500);
}

// covers: HER-ADV-003
#[test]
fn the_flat_bonuses_come_after_the_rate_once_per_artifact_type() {
    for r in both(movement_pack(vec![])) {
        // Batch E5's E5-1 (the table's 1560): Boots of Speed 2160, Equestrian's Gloves 1860,
        // both with Expert Logistics 2928 (⌊1560 × 1.3⌋ + 900).
        assert_eq!(land_points(&hero(&r, 1, &[], &["t:helm"]), &r, 1560), 2160);
        assert_eq!(land_points(&hero(&r, 1, &[], &["t:eye"]), &r, 1560), 1860);
        let both = hero(&r, 1, &[("t:logistics", 3)], &["t:helm", "t:eye"]);
        assert_eq!(land_points(&both, &r, 1560), 2928);
        // Two copies count once; a part of a worn combination counts as worn; the backpack not.
        let twice = hero(&r, 1, &[], &["t:eye", "t:eye"]);
        assert_eq!(land_points(&twice, &r, 1560), 1860);
        let combined = hero(&r, 1, &[], &["t:circle"]);
        assert_eq!(land_points(&combined, &r, 1560), 2160);
        assert_eq!(land_points(&hero(&r, 1, &[], &[]), &r, 1560), 1560);
        // The sum saturates.
        let expert = hero(&r, 1, &[("t:logistics", 3)], &["t:helm"]);
        assert_eq!(land_points(&expert, &r, u32::MAX), u32::MAX);
        assert_eq!(land_points(&combined, &r, u32::MAX - 10), u32::MAX);
    }
}

// covers: HER-ADV-004
#[test]
fn a_creature_specialty_makes_its_creature_and_direct_upgrades_one_faster() {
    let specialty = vec![HeroEffectDef::CreatureSpecialty {
        creature: key("t:archer"),
        bp_per_step: 500,
        speed: 1,
    }];
    let r = compile(movement_pack(specialty)).unwrap();
    let plain = compile(movement_pack(vec![])).unwrap();
    let creature = |r: &Ruleset, k: &str| r.creature_by_key(k).unwrap();
    let d = hero(&r, 1, &[], &[]);
    // Batch E5's E5-1h: an Archer specialist's Archers count as speed 5 (the table's 1630).
    for (k, speed, specialist) in [
        ("t:archer", 4, 5),
        ("t:marksman", 6, 7),
        ("t:sharpshooter", 7, 7),
        ("t:peasant", 5, 5),
        ("t:quick", 255, 255),
    ] {
        assert_eq!(
            movement_speed(&d, &plain, creature(&plain, k)),
            speed,
            "{k}"
        );
        assert_eq!(movement_speed(&d, &r, creature(&r, k)), specialist, "{k}");
    }
    // A creature the core lacks counts as the fastest.
    let unknown = creature(&r, "t:quick");
    let small = compile(pack()).unwrap();
    assert_eq!(movement_speed(&details(&small), &small, unknown), u8::MAX);
    // Speed artifacts are not read: a creature_stat effect on the archers changes nothing.
    let mut p = movement_pack(vec![]);
    for a in &mut p.artifacts {
        if a.key.to_string() == "t:helm" {
            a.effects = vec![HeroEffectDef::CreatureStat {
                creatures: h3_content::CreatureSelector::All,
                stat: h3_content::CreatureStat::Speed,
                amount: 1,
            }];
        }
    }
    let r = compile(p).unwrap();
    let fast = hero(&r, 1, &[], &["t:helm"]);
    assert_eq!(movement_speed(&fast, &r, creature(&r, "t:archer")), 4);
}

// covers: HER-ADV-005, HER-ADV-007
#[test]
fn the_movement_levels_are_the_highest_of_their_sources() {
    let r = compile(movement_pack(vec![])).unwrap();
    let levels = |skills: &[(&str, u8)], worn: &[&str]| movement(&hero(&r, 1, skills, worn), &r);
    let none = Movement {
        pathfinding: 0,
        flying: None,
        water_walking: None,
    };
    assert_eq!(levels(&[], &[]), none);
    assert_eq!(Movement::default(), none);
    for level in 1..=3 {
        assert_eq!(levels(&[("t:pathfinding", level)], &[]).pathfinding, level);
    }
    // The loop needs Luck: without it neither its flying nor its Pathfinding counts.
    assert_eq!(
        levels(&[("t:pathfinding", 1)], &["t:circle"]).pathfinding,
        1
    );
    assert_eq!(levels(&[("t:pathfinding", 1)], &["t:circle"]).flying, None);
    let flyer = levels(&[("t:pathfinding", 1), ("t:luck", 1)], &["t:circle"]);
    assert_eq!(flyer.pathfinding, 3);
    assert_eq!(flyer.flying, Some(3));
    assert_eq!(flyer.water_walking, None);
    // Water walking: the kit's 0 alone, the hook's 3 above it, whichever comes first.
    assert_eq!(levels(&[], &["t:kit"]).water_walking, Some(0));
    assert_eq!(levels(&[], &["t:kit", "t:hook"]).water_walking, Some(3));
    assert_eq!(levels(&[], &["t:hook", "t:kit"]).water_walking, Some(3));
    let walker = levels(&[], &["t:hook"]);
    assert_eq!((walker.pathfinding, walker.flying), (0, None));
}

// covers: HER-ADV-006
#[test]
fn the_scouting_radius_adds_the_skills_and_artifacts_tiles() {
    // A Scouting specialty grows nothing (ADR-0013 point 13.4).
    let specialty = vec![HeroEffectDef::SkillSpecialty {
        skill: key("t:scouting"),
        bp_per_level: 500,
    }];
    for r in [
        compile(movement_pack(vec![])).unwrap(),
        compile(movement_pack(specialty)).unwrap(),
    ] {
        let radius = |level: u8, skills: &[(&str, u8)], worn: &[&str], base: u8| {
            scouting_radius(&hero(&r, level, skills, worn), &r, base)
        };
        assert_eq!(radius(1, &[], &[], 5), 5);
        for level in 1..=3 {
            assert_eq!(radius(20, &[("t:scouting", level)], &[], 5), 5 + level);
        }
        // The Speculum's and the Spyglass's stand-ins: one tile each, two copies once; Expert
        // Scouting with both reaches the original's 10.
        assert_eq!(radius(1, &[], &["t:eye", "t:eye"], 5), 6);
        assert_eq!(
            radius(1, &[("t:scouting", 3)], &["t:eye", "t:circle"], 5),
            10
        );
        assert_eq!(radius(1, &[("t:scouting", 3)], &["t:eye"], 254), u8::MAX);
    }
}

/// The original's sea table (`movement.txt`'s "Sea Movement", HER-ADV-009): by Navigation level.
const SEA_TABLE: [u32; 4] = [1500, 2250, 3000, 3750];

/// The movement pack with the sea (E08 T71): Navigation 1/2/3 as `t:navigation`; `t:band` +1 000
/// sea points (the Necklace of Ocean Guidance's), `t:helm` +500 (the Sea Captain's Hat's), the
/// combination `t:circle` (band, loop, helm) free boarding (the Admiral's Hat's), `t:hook` a
/// Navigation level 2 of its own; with `loop_boards`, `t:loop` (needs Luck) free boarding too.
fn sea_pack(specialty: Vec<HeroEffectDef>, loop_boards: bool) -> ContentPack {
    let mut p = movement_pack(specialty);
    p.skills.push(SecondarySkillDef {
        key: key("t:navigation"),
        name: "Skill".into(),
        levels: [1, 2, 3].map(|level| vec![HeroEffectDef::Navigation { level }]),
    });
    for a in &mut p.artifacts {
        let effect = match a.key.to_string().as_str() {
            "t:band" => HeroEffectDef::SeaMovementPoints { points: 1000 },
            "t:helm" => HeroEffectDef::SeaMovementPoints { points: 500 },
            "t:circle" => HeroEffectDef::FreeBoarding,
            "t:hook" => HeroEffectDef::Navigation { level: 2 },
            "t:loop" if loop_boards => HeroEffectDef::FreeBoarding,
            _ => continue,
        };
        a.effects.push(effect);
    }
    p
}

/// A Navigation specialist's specialty (Sylvia's, Voy's): 5 % of the table's first entry per
/// level with the skill.
fn navigation_specialist() -> Vec<HeroEffectDef> {
    vec![HeroEffectDef::SkillSpecialty {
        skill: key("t:navigation"),
        bp_per_level: 500,
    }]
}

// covers: HER-ADV-009
#[test]
fn the_sea_points_are_the_tables_entry_at_the_navigation_level() {
    // Exact integers in both profiles (ADR-0013 point 13.6).
    for r in both(sea_pack(vec![], false)) {
        let points = |skills: &[(&str, u8)], worn: &[&str]| {
            sea_points(&hero(&r, 10, skills, worn), &r, &SEA_TABLE)
        };
        // Without Navigation 1 500, whatever the army or the land's Logistics.
        assert_eq!(points(&[], &[]), 1500);
        assert_eq!(points(&[("t:logistics", 3)], &[]), 1500);
        for (level, expected) in [(1, 2250), (2, 3000), (3, 3750)] {
            assert_eq!(points(&[("t:navigation", level)], &[]), expected);
        }
        // An artifact's Navigation level counts as the skill's; several sources: the highest.
        assert_eq!(points(&[], &["t:hook"]), 3000);
        assert_eq!(points(&[("t:navigation", 1)], &["t:hook"]), 3000);
        assert_eq!(points(&[("t:navigation", 3)], &["t:hook"]), 3750);
        // A table the adventure gives otherwise is read as given.
        let advanced = hero(&r, 1, &[("t:navigation", 2)], &[]);
        assert_eq!(sea_points(&advanced, &r, &[1, 2, 3, 4]), 3);
    }
}

// covers: HER-ADV-009
#[test]
fn a_navigation_specialist_adds_its_share_of_the_first_entry_with_the_skill() {
    for r in both(sea_pack(navigation_specialist(), false)) {
        let points = |level: u8, skills: &[(&str, u8)], worn: &[&str]| {
            sea_points(&hero(&r, level, skills, worn), &r, &SEA_TABLE)
        };
        // Sylvia at level 10: Basic Navigation 3 000 (2 250 + 750), none without the skill.
        assert_eq!(points(10, &[("t:navigation", 1)], &[]), 3000);
        assert_eq!(points(10, &[], &[]), 1500);
        // 75 a level (⌊level × 1 500 ÷ 20⌋), whatever the skill's level.
        assert_eq!(points(1, &[("t:navigation", 1)], &[]), 2325);
        assert_eq!(points(20, &[("t:navigation", 3)], &[]), 5250);
        // The level from an artifact alone gets no share; with the skill the share comes once.
        assert_eq!(points(10, &[], &["t:hook"]), 3000);
        assert_eq!(points(10, &[("t:navigation", 1)], &["t:hook"]), 3750);
        // A specialty on another skill adds nothing at sea.
        let logistics = compile(sea_pack(logistics_specialist(), false)).unwrap();
        let other = hero(
            &logistics,
            10,
            &[("t:navigation", 1), ("t:logistics", 1)],
            &[],
        );
        assert_eq!(sea_points(&other, &logistics, &SEA_TABLE), 2250);
        // The share's ⌊⌋ is taken once, of the exact product.
        let third = hero(&r, 3, &[("t:navigation", 1)], &[]);
        assert_eq!(sea_points(&third, &r, &[7, 0, 0, 0]), 1);
    }
}

// covers: HER-ADV-009
#[test]
fn the_sea_artifacts_add_once_per_type_and_the_admirals_hat_boards_free() {
    for r in both(sea_pack(vec![], false)) {
        let d = |skills: &[(&str, u8)], worn: &[&str]| hero(&r, 1, skills, worn);
        // Batch E8's E8-10: Advanced Navigation and the Necklace 4 000; the Sea Captain's Hat
        // alone 2 000.
        assert_eq!(
            sea_points(&d(&[("t:navigation", 2)], &["t:band"]), &r, &SEA_TABLE),
            4000
        );
        assert_eq!(sea_points(&d(&[], &["t:helm"]), &r, &SEA_TABLE), 2000);
        // Two copies count once; the backpack not (the test hero carries a helm there).
        assert_eq!(
            sea_points(&d(&[], &["t:band", "t:band"]), &r, &SEA_TABLE),
            2500
        );
        assert_eq!(sea_points(&d(&[], &[]), &r, &SEA_TABLE), 1500);
        // The Admiral's Hat: 3 000 through its parts, and free boarding; its parts worn apart
        // give the same points and no free boarding (the original tests the combination).
        let admiral = d(&[], &["t:circle"]);
        assert_eq!(sea_points(&admiral, &r, &SEA_TABLE), 3000);
        assert!(free_boarding(&admiral, &r));
        let apart = d(&[], &["t:band", "t:helm"]);
        assert_eq!(sea_points(&apart, &r, &SEA_TABLE), 3000);
        assert!(!free_boarding(&apart, &r));
        assert!(!free_boarding(&d(&[("t:navigation", 3)], &[]), &r));
        // The sum saturates.
        let table = [u32::MAX - 100, 0, 0, 0];
        assert_eq!(sea_points(&admiral, &r, &table), u32::MAX);
    }
    // A specialist's share saturates too.
    let r = compile(sea_pack(navigation_specialist(), false)).unwrap();
    let veteran = hero(&r, u8::MAX, &[("t:navigation", 1)], &[]);
    assert_eq!(sea_points(&veteran, &r, &[u32::MAX, 0, 0, 0]), u32::MAX);
}

// covers: HER-ADV-009
#[test]
fn free_boarding_counts_a_part_of_a_worn_combination_that_needs_its_skill() {
    // `t:loop` boards free but needs Luck; the circle (here without its own) holds it.
    let mut p = sea_pack(vec![], true);
    for a in &mut p.artifacts {
        if a.key.to_string() == "t:circle" {
            a.effects.clear();
        }
    }
    let r = compile(p).unwrap();
    assert!(!free_boarding(&hero(&r, 1, &[], &["t:circle"]), &r));
    assert!(free_boarding(
        &hero(&r, 1, &[("t:luck", 1)], &["t:circle"]),
        &r
    ));
    assert!(free_boarding(
        &hero(&r, 1, &[("t:luck", 1)], &["t:loop"]),
        &r
    ));
}
