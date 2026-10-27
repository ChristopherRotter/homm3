//! Tests of the wandering monsters' answer (ADV-MON-007, E08 T20b1): the hero's power in the
//! executable's arithmetic, the strength step's thresholds, the sympathy with its related type,
//! Diplomacy and Easy, and the order of the tests.

extern crate std;

use std::string::ToString;
use std::vec;
use std::vec::Vec;

use h3_content::{
    ContentKey, ContentPack, CreatureDef, CreatureInfoDef, FactionDef, HeroEffectDef, HeroTypeId,
    PackManifest, ProfileDef, SecondarySkillDef, SkillId,
};

use super::*;

fn key(text: &str) -> ContentKey {
    ContentKey::parse(text).unwrap()
}

/// The creatures of [`core`], in core order.
const PIKEMAN: CreatureTypeId = CreatureTypeId::from_raw(0);
const HALBERDIER: CreatureTypeId = CreatureTypeId::from_raw(1);
const PEASANT: CreatureTypeId = CreatureTypeId::from_raw(2);
const WATER: CreatureTypeId = CreatureTypeId::from_raw(3);
const ICE: CreatureTypeId = CreatureTypeId::from_raw(4);
const STONE: CreatureTypeId = CreatureTypeId::from_raw(5);
const DIPLOMACY: SkillId = SkillId::from_raw(0);

/// A core with a pikeman (AI value 100) upgrading to a halberdier (115) of the castle, level 1; a
/// neutral peasant (10) that names an upgrade; a water elemental (100) of level 3 upgrading to an
/// ice elemental (100) of level 4, as `BAT-QUIRK-19` has them; a stone (AI value 0) of the castle;
/// Diplomacy's join bonus 1, 2, 3.
fn core() -> Ruleset {
    let creature =
        |k: &str, faction: Option<&str>, level, ai_value, upgrades: &[&str]| CreatureDef {
            key: key(k),
            speed: 5,
            info: CreatureInfoDef {
                faction: faction.map(key),
                level,
                ai_value,
                upgrades: upgrades.iter().map(|u| key(u)).collect(),
                ..CreatureInfoDef::default()
            },
        };
    let pack = ContentPack {
        factions: ["t:castle", "t:conflux"]
            .iter()
            .map(|k| FactionDef {
                key: key(k),
                name: "Faction".into(),
                native_terrain: None,
            })
            .collect(),
        creatures: vec![
            creature("t:pikeman", Some("t:castle"), 1, 100, &["t:halberdier"]),
            creature("t:halberdier", Some("t:castle"), 1, 115, &[]),
            creature("t:peasant", None, 1, 10, &["t:pikeman"]),
            creature("t:water", Some("t:conflux"), 3, 100, &["t:ice"]),
            creature("t:ice", Some("t:conflux"), 4, 100, &[]),
            creature("t:stone", Some("t:castle"), 2, 0, &[]),
        ],
        skills: vec![SecondarySkillDef {
            key: key("t:diplomacy"),
            name: "Diplomacy".into(),
            levels: [1, 2, 3].map(|level| vec![HeroEffectDef::JoinBonus { level }]),
        }],
        profiles: vec![ProfileDef {
            key: key("t:faithful"),
            quirks: vec![],
        }],
        ..ContentPack::new(PackManifest {
            id: "t".into(),
            version: "1".into(),
            depends: vec![],
        })
    };
    Ruleset::compile(&[pack], "t:faithful").unwrap()
}

/// Details with this Attack and Defense and these Diplomacy levels (none, or 1 to 3).
fn details(attack: u8, defense: u8, diplomacy: Option<u8>) -> HeroDetails {
    HeroDetails {
        hero_type: HeroTypeId::from_raw(0),
        level: 1,
        experience: 0,
        offered: Default::default(),
        primary: [attack, defense, 1, 1],
        skills: diplomacy
            .map(|level| (DIPLOMACY, level))
            .into_iter()
            .collect(),
        worn: vec![],
        backpack: vec![],
        spells: vec![],
        mana: 0,
        bonuses: Default::default(),
    }
}

/// An army of these stacks, from slot 0.
fn army(stacks: &[(CreatureTypeId, u32)]) -> Vec<ArmyStack> {
    stacks
        .iter()
        .zip(0..)
        .map(|(&(creature, count), slot)| ArmyStack {
            slot,
            creature,
            count,
        })
        .collect()
}

fn wanderer(creature: CreatureTypeId, count: u32, aggression: i8) -> Wanderer {
    Wanderer {
        creature,
        count,
        aggression,
        never_flees: false,
    }
}

// covers: ADV-MON-007
#[test]
fn the_power_is_the_army_value_as_a_float_times_the_float_factor_truncated() {
    let core = core();
    let power = |stacks: &[(CreatureTypeId, u32)], skills: Option<(u8, u8)>| {
        let details = skills.map(|(attack, defense)| details(attack, defense, None));
        power(&core, &army(stacks), details.as_ref())
    };
    // 10 pikemen and 2 halberdiers are worth 1 230; without details the factor is 1.
    assert_eq!(power(&[(PIKEMAN, 10), (HALBERDIER, 2)], None), 1230);
    assert_eq!(power(&[(PIKEMAN, 10)], Some((10, 10))), 1500);
    // √(1.15 · 1.25) stored as a float, 1.19895792; the same both ways round.
    assert_eq!(power(&[(PIKEMAN, 10)], Some((5, 3))), 1198);
    assert_eq!(power(&[(PIKEMAN, 10)], Some((3, 5))), 1198);
    // 5.95 stored as a float is 5.94999981: one point short of the exact product.
    assert_eq!(power(&[(PIKEMAN, 10)], Some((99, 99))), 5949);
    // The army's value is stored as a float first: 33 554 435 becomes 33 554 436.
    assert_eq!(
        power(&[(PIKEMAN, 335_534), (HALBERDIER, 9)], None),
        33_554_436
    );
    // A creature the core does not know is worth nothing.
    assert_eq!(power(&[(CreatureTypeId::from_raw(99), 50)], None), 0);
}

/// The ratio `num / den` as the answer stores it: both floats, divided, stored as a float.
fn ratio(num: u64, den: u64) -> X87 {
    X87::int(num)
        .round(SINGLE_BITS)
        .over(X87::int(den).round(SINGLE_BITS))
        .unwrap()
        .round(SINGLE_BITS)
}

// covers: ADV-MON-007
#[test]
fn the_strength_step_follows_the_executables_thresholds() {
    let step = |num, den| strength_step(ratio(num, den));
    assert_eq!(step(7000, 1000), 11);
    assert_eq!(step(70_000, 1000), 11);
    assert_eq!(step(6999, 1000), 11);
    assert_eq!(step(6499, 1000), 10);
    assert_eq!(step(1500, 1000), 1);
    assert_eq!(step(1000, 1000), 0);
    assert_eq!(step(999, 1000), -1);
    assert_eq!(step(501, 1000), -1);
    // D-8: exactly 0.5 is −2 (the original tests r > 0.5).
    assert_eq!(step(500, 1000), -2);
    // 0.333 as a float lies above the double 0.333: −2.
    assert_eq!(step(333, 1000), -2);
    assert_eq!(step(1, 3), -2);
    assert_eq!(step(332, 1000), -3);
    assert_eq!(step(0, 1000), -3);
}

// covers: ADV-MON-007
#[test]
fn the_sympathy_counts_the_monsters_type_and_its_related_one() {
    let core = core();
    let sympathy =
        |stacks: &[(CreatureTypeId, u32)], creature| sympathy(&core, &army(stacks), creature);
    // None of the type: 0; some: 1; more than half of the creatures by count: 2.
    assert_eq!(sympathy(&[(PEASANT, 10)], PIKEMAN), 0);
    assert_eq!(sympathy(&[(PIKEMAN, 5), (PEASANT, 6)], PIKEMAN), 1);
    assert_eq!(sympathy(&[(PIKEMAN, 5), (PEASANT, 5)], PIKEMAN), 1);
    assert_eq!(sympathy(&[(PIKEMAN, 6), (PEASANT, 5)], PIKEMAN), 2);
    // D-9: the monster's own type and its upgrade count, and an upgrade's base.
    assert_eq!(sympathy(&[(HALBERDIER, 6), (PEASANT, 5)], PIKEMAN), 2);
    assert_eq!(
        sympathy(&[(PIKEMAN, 1), (HALBERDIER, 5), (PEASANT, 5)], HALBERDIER),
        2
    );
    // A neutral creature has no related type, whatever it names.
    assert_eq!(sympathy(&[(PIKEMAN, 6)], PEASANT), 0);
    // The water elemental's upgrade is the ice elemental; the ice elemental, of another level
    // than its base (BAT-QUIRK-19), has no related type.
    assert_eq!(sympathy(&[(ICE, 3)], WATER), 2);
    assert_eq!(sympathy(&[(WATER, 3)], ICE), 0);
    // A creature of a faction without an upgrade or a base has none.
    assert_eq!(sympathy(&[(PIKEMAN, 3)], STONE), 0);
    assert_eq!(sympathy(&[], PIKEMAN), 0);
}

// covers: ADV-MON-007
#[test]
fn the_temper_adds_the_step_the_sympathy_and_diplomacy_one_higher_on_easy() {
    let core = core();
    let temper = |stacks: &[(CreatureTypeId, u32)], details: Option<&HeroDetails>, difficulty| {
        Temper::new(
            &core,
            (&army(stacks), details),
            wanderer(PIKEMAN, 10, 5),
            difficulty,
        )
    };
    // Power 1 230 against a strength of 1 000: St 0; Sy 2; no Diplomacy (Normal).
    let normal = temper(&[(PIKEMAN, 10), (HALBERDIER, 2)], None, 1);
    assert_eq!(
        normal,
        Temper {
            charisma: 2,
            sympathy: 2,
            diplomacy: 0,
            aggression: 5,
            never_flees: false,
        }
    );
    // On Easy no Diplomacy counts as Basic.
    assert_eq!(
        temper(&[(PIKEMAN, 10), (HALBERDIER, 2)], None, 0).diplomacy,
        1
    );
    let expert = details(0, 0, Some(3));
    let advanced = details(0, 0, Some(2));
    assert_eq!(temper(&[(PEASANT, 1)], Some(&expert), 1).diplomacy, 3);
    assert_eq!(temper(&[(PEASANT, 1)], Some(&expert), 0).diplomacy, 3);
    assert_eq!(temper(&[(PEASANT, 1)], Some(&advanced), 0).diplomacy, 3);
    assert_eq!(temper(&[(PEASANT, 1)], Some(&advanced), 4).diplomacy, 2);
    // A peasant against 1 000: St −3, Sy 0, L 2.
    assert_eq!(temper(&[(PEASANT, 1)], Some(&advanced), 4).charisma, -1);
    // A monster of strength 0 counts as r ≥ 7.
    let stones = Temper::new(&core, (&army(&[]), None), wanderer(STONE, 4, 1), 1);
    assert_eq!(stones.charisma, 11);
}

// covers: ADV-MON-007
#[test]
fn the_answer_tests_the_aggression_against_the_charisma_then_the_offers() {
    // C = 4 with Sy 1 and L 1: join for free up to M = 3, for gold up to 4 (Sy + 2L + 1).
    let temper = |aggression, never_flees| Temper {
        charisma: 4,
        sympathy: 1,
        diplomacy: 1,
        aggression,
        never_flees,
    };
    assert_eq!(temper(5, false).first(), Answer::Fight);
    assert_eq!(temper(-4, false).first(), Answer::Join);
    assert_eq!(temper(3, false).first(), Answer::Join);
    assert_eq!(temper(4, false).first(), Answer::JoinForGold);
    // M = C after a declined offer: the monster is insulted and fights.
    assert_eq!(temper(4, false).after_join(), Answer::Fight);
    assert_eq!(temper(3, false).after_join(), Answer::Pursue);
    assert_eq!(temper(3, true).after_join(), Answer::Fight);
    // No offer to join (M above Sy + 2L + 1, at most C): it flees, or fights if it never does.
    let proud = |never_flees| Temper {
        charisma: 9,
        sympathy: 0,
        diplomacy: 0,
        aggression: 5,
        never_flees,
    };
    assert_eq!(proud(false).first(), Answer::Pursue);
    assert_eq!(proud(true).first(), Answer::Fight);
    let insulted = Temper {
        aggression: 9,
        ..proud(false)
    };
    assert_eq!(insulted.first(), Answer::Fight);
}

#[test]
fn the_test_core_holds_the_creatures_in_order() {
    let core = core();
    let names: Vec<_> = (0..core.creature_count())
        .filter_map(CreatureTypeId::from_index)
        .filter_map(|id| core.creature_key(id).map(ToString::to_string))
        .collect();
    assert_eq!(
        names,
        [
            "t:pikeman",
            "t:halberdier",
            "t:peasant",
            "t:water",
            "t:ice",
            "t:stone"
        ]
    );
}
