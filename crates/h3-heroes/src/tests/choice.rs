//! Tests of the original's choice between two offers (E08 T28, HER-LVL-006): a skill's value
//! (0x524690, the flag clear), the terrain shares' arithmetic, the choice between two alike
//! offers and the ranking of a new skill against the free slots (0x524DD0), with the survey's
//! worked example (`docs/rules/surveys/E08-progression.md` §10.5). The expected rankings come
//! from a model of the listing written apart from the code (tie order included). Batch E3c's E3-6
//! is in `level.rs`.

use h3_content::HeroEffectDef;
use h3_contracts::ArmyStack;

use super::level::{base, base_pack, hero, s};
use super::*;
use crate::choice::Evaluation;

/// A level-up to level 2 raising `primary`, with `offers`.
fn level_up(primary: PrimarySkill, offers: Offers) -> LevelUp {
    LevelUp {
        level: 2,
        primary,
        offers,
        offered: OfferLevels::default(),
    }
}

/// The shares of a map with 200 tiles counted (ADV-MAP-002), 20 of them water, 50, 30 and 10 rough by
/// Pathfinding level.
const SHARES: TerrainShares = TerrainShares {
    tiles: 200,
    water: 20,
    rough: [50, 30, 10],
};

/// Every skill's value for `d` at a level raising `primary`, in the original's order.
fn values(r: &Ruleset, d: &HeroDetails, primary: PrimarySkill) -> Vec<u64> {
    let up = level_up(primary, Offers::None);
    let hero = Evaluation::new(d, r, &up, ArmyWorth::SETUP, SHARES);
    let count = u16::try_from(r.skills().len()).unwrap();
    (0..count)
        .map(|raw| hero.value(SkillId::from_raw(raw)))
        .collect()
}

/// The choice for `d` between `first` and `second` at a level raising Attack, on a map without a
/// tile counted (both shares 0 / 0).
fn choose(r: &Ruleset, d: &HeroDetails, first: &str, second: &str) -> Choice {
    let up = level_up(PrimarySkill::Attack, Offers::Two(s(r, first), s(r, second)));
    original_choice(d, r, &up, ArmyWorth::SETUP, TerrainShares::default())
}

// covers: HER-LVL-006
#[test]
fn a_skills_value_follows_the_originals_table() {
    // Survey §10.2, the flag clear: A = 1 000, B = 500; Attack 3, Spell Power 5, Knowledge 7
    // (Defense raised, which no value reads).
    let r = base();
    let mut d = hero(&r, "t:orrin", 0, &[]);
    d.primary = [3, 0, 5, 7];
    let unknown = [
        62, 80, 100, 50, 10, 50, 20, 125, 1, 20, 125, 0, 50, 725, 0, 25, 3, 30, 0, 20, 30, 50, 70,
        70, 70, 10, 25, 250,
    ];
    assert_eq!(values(&r, &d, PrimarySkill::Defense), unknown);
    // With Wisdom: Eagle Eye 2, each school Spell Power × 10, Scholar × 5; Wisdom itself at Basic
    // still Spell Power × 25.
    d.skills = vec![(s(&r, "wisdom"), 1)];
    let wise = values(&r, &d, PrimarySkill::Defense);
    let at = |name: &str| wise[usize::from(s(&r, name).raw())];
    assert_eq!(at("wisdom"), 125);
    assert_eq!(at("eagle_eye"), 2);
    for school in ["fire_magic", "air_magic", "water_magic", "earth_magic"] {
        assert_eq!(at(school), 50, "{school}");
    }
    assert_eq!(at("scholar"), 25);
    // By the skill's level: Pathfinding's rough share, Archery's and Estates' tables; 0 at Expert.
    for (level, pathfinding, archery, estates) in
        [(1, 37, 35, 725), (2, 12, 55, 1550), (3, 0, 0, 0)]
    {
        d.skills = ["pathfinding", "archery", "estates"]
            .map(|k| (s(&r, k), level))
            .to_vec();
        let v = values(&r, &d, PrimarySkill::Defense);
        let at = |name: &str| v[usize::from(s(&r, name).raw())];
        assert_eq!(
            [at("pathfinding"), at("archery"), at("estates")],
            [pathfinding, archery, estates],
            "level {level}"
        );
    }
    // A hero knowing eight skills: an unknown one is worth 0, a known one its value.
    let eight = [
        "logistics",
        "scouting",
        "diplomacy",
        "leadership",
        "luck",
        "tactics",
        "offense",
        "armorer",
    ];
    d.skills = eight.map(|k| (s(&r, k), 1)).to_vec();
    let full = values(&r, &d, PrimarySkill::Defense);
    assert_eq!(full[usize::from(s(&r, "estates").raw())], 0);
    assert_eq!(full[usize::from(s(&r, "logistics").raw())], 100);
    // A skill past the original's 28 is worth 0.
    let mut p = base_pack();
    let mut extra = p.skills[0].clone();
    extra.key = key("t:extra");
    p.skills.push(extra);
    let r = compile(p).unwrap();
    let d = hero(&r, "t:orrin", 0, &[]);
    assert_eq!(values(&r, &d, PrimarySkill::Defense)[28], 0);
}

// covers: HER-LVL-006
#[test]
fn the_primary_skills_are_seen_raised_with_the_worn_artifacts() {
    // The level's primary skill counts raised (the original chooses after 0x4DA8F7), with the
    // worn artifacts' bonuses, Spell Power and Knowledge held to 1–99, Attack to 0–99.
    let mut p = base_pack();
    let helm = p.artifacts.iter_mut().find(|a| a.key == key("t:helm"));
    helm.unwrap().effects = vec![HeroEffectDef::PrimarySkills {
        attack: 0,
        defense: 0,
        power: 1,
        knowledge: 0,
    }];
    let r = compile(p).unwrap();
    let mut d = hero(&r, "t:orrin", 0, &[]);
    d.primary = [0, 0, 3, 99];
    let wisdom = usize::from(s(&r, "wisdom").raw());
    let artillery = usize::from(s(&r, "artillery").raw());
    let intelligence = usize::from(s(&r, "intelligence").raw());
    let v = values(&r, &d, PrimarySkill::Attack);
    assert_eq!([v[wisdom], v[artillery], v[intelligence]], [75, 10, 990]);
    let v = values(&r, &d, PrimarySkill::Power);
    assert_eq!([v[wisdom], v[artillery]], [100, 0]);
    // Knowledge at 99 stays there (the gain saturates at `primary_max`).
    assert_eq!(values(&r, &d, PrimarySkill::Knowledge)[intelligence], 990);
    put_on(&r, &mut d.worn, art(&r, "t:helm"));
    assert_eq!(values(&r, &d, PrimarySkill::Power)[wisdom], 125);
    d.primary[2] = 99;
    assert_eq!(values(&r, &d, PrimarySkill::Power)[wisdom], 2475);
    // Wisdom against Logistics, both new: 75 against 100 takes Logistics; with Spell Power raised
    // to 4, 100 against 100 takes the first.
    let mut d = hero(&r, "t:orrin", 0, &[]);
    d.primary = [2, 2, 3, 1];
    let offers = Offers::Two(s(&r, "wisdom"), s(&r, "logistics"));
    let choice = |d: &HeroDetails, primary| {
        let up = level_up(primary, offers);
        original_choice(d, &r, &up, ArmyWorth::SETUP, SHARES)
    };
    assert_eq!(choice(&d, PrimarySkill::Attack), Choice::Second);
    assert_eq!(choice(&d, PrimarySkill::Power), Choice::First);
}

// covers: HER-LVL-006
#[test]
fn the_shares_are_the_originals_doubles() {
    // trunc(1 000 × count / tiles / 4) for Pathfinding, / 2 for Navigation, through the stored
    // f64 share (a scan of every count to 144 × 144 × 2 tiles found it equal to IEEE doubles,
    // E08 T28); none without tiles (the original's NaN gives 0).
    let r = base();
    let d = hero(&r, "t:orrin", 0, &[]);
    let up = level_up(PrimarySkill::Attack, Offers::None);
    let pathfinding = s(&r, "pathfinding");
    let navigation = s(&r, "navigation");
    let worth = |shares: TerrainShares| {
        let hero = Evaluation::new(&d, &r, &up, ArmyWorth::SETUP, shares);
        [hero.value(pathfinding), hero.value(navigation)]
    };
    let shares = |tiles, water, rough| TerrainShares {
        tiles,
        water,
        rough: [rough, 0, 0],
    };
    assert_eq!(worth(TerrainShares::default()), [0, 0]);
    assert_eq!(worth(shares(0, 7, 7)), [0, 0]);
    assert_eq!(worth(shares(3, 3, 1)), [83, 500]);
    assert_eq!(worth(shares(7, 3, 7)), [250, 214]);
    assert_eq!(worth(shares(41_472, 1, 41_471)), [249, 0]);
    assert_eq!(worth(shares(41_472, 41_471, 1)), [0, 499]);
    // The army's worth scales them.
    let hero = Evaluation::new(
        &d,
        &r,
        &up,
        ArmyWorth {
            army: 4_000,
            shooters: 0,
        },
        shares(3, 1, 1),
    );
    assert_eq!(
        [hero.value(pathfinding), hero.value(navigation)],
        [333, 666]
    );
    assert_eq!(hero.value(s(&r, "archery")), 0);
}

// covers: HER-LVL-006
#[test]
fn two_alike_offers_take_the_higher_value_the_first_on_a_tie() {
    let r = base();
    let d = hero(
        &r,
        "t:orrin",
        0,
        &[
            ("logistics", 1),
            ("archery", 1),
            ("leadership", 1),
            ("luck", 1),
        ],
    );
    // Both known: Archery 35 against Logistics 100; Leadership and Luck 20 each.
    assert_eq!(choose(&r, &d, "archery", "logistics"), Choice::Second);
    assert_eq!(choose(&r, &d, "logistics", "archery"), Choice::First);
    assert_eq!(choose(&r, &d, "leadership", "luck"), Choice::First);
    assert_eq!(choose(&r, &d, "luck", "leadership"), Choice::First);
    // Both new: Estates 725 against Ballistics 125.
    assert_eq!(choose(&r, &d, "ballistics", "estates"), Choice::Second);
    assert_eq!(choose(&r, &d, "estates", "ballistics"), Choice::First);
    // One offer or none: the first, which learns the one offer.
    for offers in [Offers::One(s(&r, "luck")), Offers::None] {
        let up = level_up(PrimarySkill::Attack, offers);
        let choice = original_choice(&d, &r, &up, ArmyWorth::SETUP, SHARES);
        assert_eq!(choice, Choice::First);
    }
}

// covers: HER-LVL-006
#[test]
fn a_new_skill_is_taken_when_it_ranks_within_the_free_slots() {
    // Survey §10.5's worked example: a Knight knowing Basic Leadership, offered its advance and
    // Tactics, on a map without a coast: Estates, First Aid, Ballistics, Logistics, Archery,
    // Offense and Armorer use up the seven free slots before Tactics: the advance.
    let r = base();
    let d = hero(&r, "t:orrin", 0, &[("leadership", 1)]);
    assert_eq!(choose(&r, &d, "leadership", "tactics"), Choice::First);
    assert_eq!(choose(&r, &d, "tactics", "leadership"), Choice::Second);
    // Estates ranks first; Armorer seventh, with seven slots free.
    assert_eq!(choose(&r, &d, "leadership", "estates"), Choice::Second);
    assert_eq!(choose(&r, &d, "estates", "leadership"), Choice::First);
    assert_eq!(choose(&r, &d, "leadership", "armorer"), Choice::Second);
    assert_eq!(choose(&r, &d, "leadership", "scouting"), Choice::First);
    // The exchange sort's ties: with Leadership and Tactics known (six slots), Offense walks
    // before Armorer (both 70) and takes the last slot; with Estates, First Aid, Ballistics,
    // Logistics and Archery known (three), Armorer walks first, and Learning before Scouting (both
    // 50). A skill the class never offers counts once it is offered (Necromancy: 50).
    let d = hero(&r, "t:orrin", 0, &[("leadership", 1), ("tactics", 1)]);
    assert_eq!(choose(&r, &d, "leadership", "offense"), Choice::Second);
    assert_eq!(choose(&r, &d, "leadership", "armorer"), Choice::First);
    let known = ["estates", "first_aid", "ballistics", "logistics", "archery"];
    let d = hero(&r, "t:orrin", 0, &known.map(|k| (k, 1)));
    for (new, taken) in [
        ("armorer", true),
        ("offense", true),
        ("learning", true),
        ("scouting", false),
        ("necromancy", false),
    ] {
        let expected = if taken { Choice::Second } else { Choice::First };
        assert_eq!(choose(&r, &d, "estates", new), expected, "{new}");
    }
}

/// The flag-set path's reads (E31 T2e): A 2 000, B 900, the power gain 30, the knowledge gain
/// 45, a gold weight of 0.75, the schools 11 to 14, a Ballista and a Tent worn, [`SHARES`].
fn reads() -> ComputerReads {
    ComputerReads {
        worth: (2000, 900),
        power_gain: 30,
        knowledge_gain: 45,
        gold_weight: h3_kernel::arith::SignedX87::int(3)
            .over(h3_kernel::arith::SignedX87::int(4))
            .unwrap(),
        schools: [11, 12, 13, 14],
        ballista: true,
        tent: true,
        shares: SHARES,
    }
}

/// Every skill's value to a computer player's hero `d` with `reads`, in the original's order.
fn computer_values(r: &Ruleset, d: &HeroDetails, reads: &ComputerReads) -> Vec<i32> {
    let count = u16::try_from(r.skills().len()).unwrap();
    (0..count)
        .map(|raw| computer_skill_value(d, r, reads, SkillId::from_raw(raw)))
        .collect()
}

// covers: CPU-SKL-001
#[test]
fn a_computer_players_hero_values_its_skills_by_its_values_per_unit() {
    // Survey §10.2's flag-set column (E31 T2e's re-read of 0x524690): Attack 3, Spell Power 5,
    // Knowledge 7, nothing raised. Wisdom 5 × 30 / 2, Mysticism 45 / 10, Estates trunc(725 ×
    // 0.75), the schools theirs without Wisdom, Intelligence 7 × 45 / 4, Sorcery 5 × 30 / 20;
    // the others by A and B as with the flag clear.
    let r = base();
    let mut d = hero(&r, "t:orrin", 0, &[]);
    d.primary = [3, 0, 5, 7];
    let unknown = [
        125, 144, 200, 100, 20, 100, 40, 75, 4, 40, 250, 0, 100, 543, 11, 12, 13, 14, 0, 40, 30,
        100, 140, 140, 78, 7, 50, 250,
    ];
    assert_eq!(computer_values(&r, &d, &reads()), unknown);
    // With Wisdom: Eagle Eye 30 / 5, Scholar 5 × 30 / 10; the schools stay theirs.
    d.skills = vec![(s(&r, "wisdom"), 1)];
    let wise = computer_values(&r, &d, &reads());
    let at = |v: &[i32], name: &str| v[usize::from(s(&r, name).raw())];
    assert_eq!(
        ["eagle_eye", "scholar", "fire_magic"].map(|k| at(&wise, k)),
        [6, 15, 11]
    );
    // Artillery and First Aid count only with their machine worn; a negative gain divides toward
    // 0 (5 × -3 / 2).
    let bare = ComputerReads {
        ballista: false,
        tent: false,
        power_gain: -3,
        ..reads()
    };
    let v = computer_values(&r, &d, &bare);
    assert_eq!(
        ["artillery", "first_aid", "wisdom"].map(|k| at(&v, k)),
        [0, 0, -7]
    );
    // The flag-clear path keeps its values (A 1 000).
    assert_eq!(values(&r, &d, PrimarySkill::Defense)[2], 100);
}

// covers: CPU-SKL-001
#[test]
fn the_counted_army_adds_every_stack_and_the_shooters_again_in_32_bits() {
    let mut p = base_pack();
    for def in &mut p.creatures {
        def.info.ai_value = match def.key.name() {
            "peasant" => 3,
            "ballista" => 1000,
            _ => 0,
        };
    }
    let r = compile(p).unwrap();
    let c = |k: &str| r.creature_by_key(&std::format!("t:{k}")).unwrap();
    let army = [
        ArmyStack {
            slot: 0,
            creature: c("peasant"),
            count: 40_000,
        },
        ArmyStack {
            slot: 3,
            creature: c("ballista"),
            count: 2,
        },
    ];
    assert_eq!(counted_worth(&r, &army), (123_000, 2500));
    // 3 × 1 000 000 000 wraps: 3 000 001 000 - 2^32.
    let huge = [ArmyStack {
        slot: 0,
        creature: c("peasant"),
        count: 1_000_000_000,
    }];
    assert_eq!(counted_worth(&r, &huge), (-1_294_966_296, 500));
    assert_eq!(counted_worth(&r, &[]), (1000, 500));
}

// covers: CPU-SKL-001
#[test]
fn a_computer_players_hero_ranks_a_new_skill_by_its_own_signed_values() {
    // Knowing Leadership (seven slots free), offered Tactics (40): Estates 543, Ballistics and
    // First Aid 250, Logistics 200, Archery 144, Offense and Armorer 140 use up the slots first.
    let r = base();
    let d = hero(&r, "t:orrin", 0, &[("leadership", 1)]);
    assert!(!computer_takes_new_skill(
        &d,
        &r,
        &reads(),
        s(&r, "tactics")
    ));
    // With nothing else worth more than 0, Fire Magic at 1 ranks first; at -1 below every skill
    // worth 0 (a signed ranking), which use up the slots.
    let fire = s(&r, "fire_magic");
    let flat = ComputerReads {
        worth: (0, 0),
        power_gain: 0,
        knowledge_gain: 0,
        gold_weight: h3_kernel::arith::SignedX87::ZERO,
        schools: [1, -1, -1, -1],
        ballista: false,
        tent: false,
        ..reads()
    };
    assert!(computer_takes_new_skill(&d, &r, &flat, fire));
    let below = ComputerReads {
        schools: [-1; 4],
        ..flat
    };
    assert!(!computer_takes_new_skill(&d, &r, &below, fire));
}
