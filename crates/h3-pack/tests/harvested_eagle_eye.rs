//! Eagle Eye's chance on the harvested SoD packs (E04b T11b review, r6 F-5): the chance the
//! Heroes context derives from the skill, an Eagle Eye specialty and the three artifacts
//! (HER-SPL-004), in whole percent as the battle rolls it (BAT-MAG-069), with quirk Q-0005
//! (`base:faithful`: the original's single-precision sums, `docs/formats/installation.md`
//! §7.24) and without it (`base:strict-fixed`: exact).

use h3_battle::{BattleState, observe};
use h3_content::{Formation, Ruleset};
use h3_contracts::{Army, ArmyStack, BattleSetup, BattleSide};
use h3_heroes::{HeroDetails, derive_battle_hero};

fn sod(profile: &str) -> Ruleset {
    h3_pack::compile(
        &[h3_pack::SOD_PACK, h3_pack::AB_PACK, h3_pack::BASE_PACK],
        profile,
    )
    .unwrap()
}

/// The Eagle Eye chance, percent, of `hero` at `level` with the skill at `skill` wearing
/// `worn`, as a battle it leads shows it to its own side.
fn chance(r: &Ruleset, (hero, level): (&str, u8), skill: u8, worn: &[&str]) -> u8 {
    let details = HeroDetails {
        hero_type: r.hero_type_by_key(hero).unwrap(),
        level,
        experience: 0,
        primary: [0, 0, 1, 1],
        skills: vec![(r.skill_by_key("base:eagle_eye").unwrap(), skill)],
        worn: worn
            .iter()
            .map(|k| r.artifact_by_key(k).unwrap().into())
            .collect(),
        backpack: vec![],
        spells: vec![],
        mana: 0,
    };
    let one = |key: &str| Army {
        stacks: vec![ArmyStack {
            slot: 0,
            creature: r.creature_by_key(key).unwrap(),
            count: 1,
        }],
    };
    let setup = BattleSetup {
        attacker: one("base:pikeman"),
        defender: one("base:gnoll"),
        obstacles: vec![],
        heroes: [Some(derive_battle_hero(&details, r).unwrap()), None],
        terrain: None,
        war_machines: [vec![], vec![]],
        formations: [Formation::Loose; 2],
        funds: [0; 2],
    };
    let s = BattleState::new(r, &setup, 1).unwrap();
    observe(&s, r, BattleSide::Attacker)
        .own_hero
        .unwrap()
        .eagle_chance
}

/// A case: the hero and its level, the skill's level, the worn artifacts, the percent with
/// quirk Q-0005 and the exact percent.
type Case<'a> = ((&'a str, u8), u8, &'a [&'a str], u8, u8);

// covers: HER-SPL-004, BAT-MAG-069
/// The original keeps the chance as an `f32`: the skill's 0.40, 0.50 or 0.60, for a specialist
/// times `level × 0.05f + 1`, then `+ 0.05f`, `+ 0.10f`, `+ 0.15f` for the Bird of Perception,
/// the Stoic Watchman and the Emblem of Cognizance, each result stored as an `f32`, and rolls
/// against `trunc(chance × 100)`. Where a stored sum falls just below the decimal value the
/// hero has one percent less than the exact sum gives. The expected values are computed from
/// that arithmetic with exact fractions (the reviewer's and, again, for this test).
#[test]
fn eagle_eyes_chance_is_the_originals_single_precision_sum_with_quirk_q_0005() {
    let (faithful, fixed) = (sod("base:faithful"), sod("base:strict-fixed"));
    let (orrin, bird, watchman, emblem) = (
        ("base:orrin", 1),
        "base:bird_of_perception",
        "base:stoic_watchman",
        "base:emblem_of_cognizance",
    );
    // A specialist (5 % of the skill's value per level) at levels 2 and 6.
    let serena = |level| ("base:serena", level);
    let cases: [Case<'_>; 11] = [
        // Advanced with the Emblem: 0.5 + 0.15f lies exactly between two `f32` values and goes
        // to the even one, below 0.65.
        (orrin, 2, &[emblem], 64, 65),
        // Basic with the Watchman and the Emblem: 0.4f + 0.1f is stored as 0.5; the same tie.
        (orrin, 1, &[watchman, emblem], 64, 65),
        // Basic with the Bird: the sum's `f32` lies above 0.45 (the `f32` of 0.45 lies below).
        (orrin, 1, &[bird], 45, 45),
        // The specialist: 0.4f × 1.1 and 0.4f × 1.3 are stored below 0.44 and 0.52, 0.5 × 1.3
        // below 0.65.
        (serena(2), 1, &[], 43, 44),
        (serena(6), 1, &[], 51, 52),
        (serena(6), 2, &[], 64, 65),
        // No difference: the skill alone, Expert with all three, and the limit of 100 %.
        (orrin, 1, &[], 40, 40),
        (orrin, 2, &[], 50, 50),
        (orrin, 3, &[], 60, 60),
        (orrin, 3, &[bird, watchman, emblem], 90, 90),
        (serena(40), 3, &[bird, watchman, emblem], 100, 100),
    ];
    for (hero, skill, worn, single, exact) in cases {
        assert_eq!(
            (
                chance(&faithful, hero, skill, worn),
                chance(&fixed, hero, skill, worn)
            ),
            (single, exact),
            "{hero:?} {skill} {worn:?}"
        );
    }
}
