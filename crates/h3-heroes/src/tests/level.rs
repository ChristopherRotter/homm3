//! Tests of a hero's progression (E08 T27): experience to levels, the bound and the scenario's
//! level limit (HER-EXP-002, HER-EXP-003), a level-up's draws, primary skill and offers
//! (HER-LVL-001 to HER-LVL-004) and the level as a change (HER-LVL-005), with the survey's test
//! vectors (`docs/rules/surveys/E08-progression.md` §1.1, §2.3, §2.4, §3.3).

use std::string::ToString;
use std::vec;
use std::vec::Vec;

use h3_kernel::Rng;

use super::*;

/// The original's skills in its order (the base pack's), as test keys.
const BASE_SKILLS: [&str; 28] = [
    "pathfinding",
    "archery",
    "logistics",
    "scouting",
    "diplomacy",
    "navigation",
    "leadership",
    "wisdom",
    "mysticism",
    "luck",
    "ballistics",
    "eagle_eye",
    "necromancy",
    "estates",
    "fire_magic",
    "air_magic",
    "water_magic",
    "earth_magic",
    "scholar",
    "tactics",
    "artillery",
    "learning",
    "offense",
    "armorer",
    "intelligence",
    "sorcery",
    "resistance",
    "first_aid",
];

/// The Knight's and the Death Knight's weights in [`BASE_SKILLS`] order (`packs/base/core.toml`,
/// harvested from the original's `HCTRAITS.TXT`; 0 for a skill the class lacks).
const KNIGHT: [u8; 28] = [
    4, 5, 5, 4, 4, 8, 10, 3, 2, 3, 8, 2, 0, 6, 1, 3, 4, 2, 1, 7, 5, 4, 7, 5, 1, 1, 5, 2,
];
const DEATH_KNIGHT: [u8; 28] = [
    4, 5, 5, 4, 2, 8, 0, 6, 4, 1, 7, 4, 10, 0, 1, 2, 3, 4, 2, 5, 5, 4, 7, 5, 5, 4, 5, 0,
];

fn class(k: &str, primary: [u8; 4], gain: [[u8; 4]; 2], weights: [u8; 28]) -> HeroClassDef {
    HeroClassDef {
        key: key(k),
        name: "Class".into(),
        faction: key("t:town"),
        primary,
        primary_gain: gain,
        skill_weights: BASE_SKILLS
            .iter()
            .zip(weights)
            .filter(|&(_, w)| w > 0)
            .map(|(s, w)| (key(&std::format!("t:{s}")), w))
            .collect(),
        offer_intervals: [6, 4],
    }
}

/// The test pack with the original's 28 skills, Wisdom and the four schools, a Knight (Orrin) and
/// a Death Knight (`t:dk`) with the base pack's chances and weights.
pub(super) fn base_pack() -> ContentPack {
    let mut p = pack();
    p.skills = BASE_SKILLS
        .iter()
        .map(|s| SecondarySkillDef {
            key: key(&std::format!("t:{s}")),
            name: "Skill".into(),
            levels: [vec![], vec![], vec![]],
        })
        .collect();
    p.hero_classes = vec![
        class(
            "t:knight",
            [2, 2, 1, 1],
            [[35, 45, 10, 10], [30, 30, 20, 20]],
            KNIGHT,
        ),
        class(
            "t:death_knight",
            [1, 2, 2, 1],
            [[30, 25, 20, 25], [25, 25, 25, 25]],
            DEATH_KNIGHT,
        ),
    ];
    let mut dk = p.heroes[0].clone();
    dk.key = key("t:dk");
    dk.class = key("t:death_knight");
    p.heroes.push(dk);
    let params = p.hero_params.as_mut().unwrap();
    params.wisdom = key("t:wisdom");
    params.schools = ["fire_magic", "air_magic", "water_magic", "earth_magic"]
        .iter()
        .map(|s| key(&std::format!("t:{s}")))
        .collect();
    p
}

pub(super) fn base() -> Ruleset {
    compile(base_pack()).unwrap()
}

pub(super) fn s(r: &Ruleset, name: &str) -> SkillId {
    skill(r, &std::format!("t:{name}"))
}

/// A bare hero of `hero` at level 1 with `experience`, the skills given at their levels.
pub(super) fn hero(r: &Ruleset, hero: &str, experience: u32, skills: &[(&str, u8)]) -> HeroDetails {
    HeroDetails {
        hero_type: r.hero_type_by_key(hero).unwrap(),
        level: 1,
        experience,
        offered: OfferLevels::default(),
        primary: [2, 2, 1, 1],
        skills: skills.iter().map(|&(k, l)| (s(r, k), l)).collect(),
        worn: wearing(r, &[]),
        backpack: vec![],
        spells: vec![],
        mana: 0,
    }
}

/// Draws that return the numbers given, in order, and record the ranges asked.
#[derive(Debug, Default)]
struct Fixed {
    numbers: Vec<u32>,
    asked: Vec<(u32, u32)>,
}

impl Fixed {
    fn new(numbers: &[u32]) -> Self {
        Self {
            numbers: numbers.to_vec(),
            asked: vec![],
        }
    }
}

impl Draws for Fixed {
    fn between(&mut self, low: u32, high: u32) -> u32 {
        self.asked.push((low, high));
        self.numbers.remove(0)
    }
}

/// The level-up drawn with the original's draws for the seed byte at the next level.
pub(super) fn original(r: &Ruleset, d: &HeroDetails, seed: u8, banned: &[SkillId]) -> LevelUp {
    let mut draws = OriginalDraws::new(seed, d.level + 1);
    draw_level_up(d, r, banned, &mut draws).unwrap()
}

/// `d` with the level-up's answer applied, checked first and valid after.
pub(super) fn answered(
    r: &Ruleset,
    mut d: HeroDetails,
    up: &LevelUp,
    choice: Choice,
) -> HeroDetails {
    let change = up.change(choice);
    assert_eq!(check_change(&d, r, change), Ok(()), "{change:?}");
    apply_change(&mut d, r, change);
    assert_eq!(validate(&d, r), Ok(()), "{change:?}");
    d
}

// covers: HER-EXP-002
#[test]
fn experience_gives_the_originals_levels_up_to_74() {
    let r = ruleset();
    let params = r.hero_params().unwrap();
    // Survey §1.1's thresholds, each step from 13 on the previous + a fifth.
    let vectors = [
        (1, 0),
        (2, 1000),
        (12, 20_600),
        (13, 24_320),
        (14, 28_784),
        (15, 34_140),
        (16, 40_567),
        (20, 81_961),
        (30, 496_963),
        (40, 3_066_455),
        (50, 18_975_933),
        (60, 117_483_099),
        (70, 727_413_425),
        (74, 1_508_362_195),
        (75, 1_810_034_207),
    ];
    for (level, threshold) in vectors {
        assert_eq!(experience_for(params, level), threshold, "level {level}");
    }
    // Level 0 counts as 1; past the table, its last entry.
    assert_eq!(experience_for(params, 0), 0);
    assert_eq!(experience_for(params, u8::MAX), 1_810_034_207);
    // The bound (A-7): 74 is the highest level.
    assert_eq!(experience_bound(params), 1_810_034_206);
    let levels = [
        (0, 1),
        (999, 1),
        (1000, 2),
        (24_319, 12),
        (24_320, 13),
        (1_508_362_194, 73),
        (1_508_362_195, 74),
        (1_810_034_206, 74),
        // Beyond the bound (invalid details): still 74, never the original's wrapped 88.
        (u32::MAX, 74),
    ];
    for (experience, level) in levels {
        assert_eq!(level_of(params, experience), level, "{experience}");
    }
    // The details' checks: the experience at most the bound, the level at most the
    // experience's, the offers' levels at most the level.
    let mut d = details(&r);
    d.experience = 1_810_034_207;
    assert_eq!(validate(&d, &r), Err(DetailsError::ExperienceAboveBound));
    d.experience = 999;
    d.level = 2;
    assert_eq!(validate(&d, &r), Err(DetailsError::LevelAboveExperience));
    d.experience = 1000;
    assert_eq!(validate(&d, &r), Ok(()));
    assert_eq!(pending_levels(&d, params), 0);
    d.offered.school = 3;
    assert_eq!(validate(&d, &r), Err(DetailsError::OfferedAboveLevel));
    d.offered = OfferLevels {
        wisdom: 3,
        school: 2,
    };
    assert_eq!(validate(&d, &r), Err(DetailsError::OfferedAboveLevel));
    // Pending levels: the experience's level beyond the level given.
    d.offered = OfferLevels::default();
    d.experience = 4600;
    assert_eq!(pending_levels(&d, params), 3);
    let texts = [
        (
            DetailsError::ExperienceAboveBound,
            "the experience exceeds the highest level's",
        ),
        (
            DetailsError::LevelAboveExperience,
            "the level exceeds the experience's",
        ),
        (
            DetailsError::OfferedAboveLevel,
            "an offer's level exceeds the level",
        ),
    ];
    for (error, text) in texts {
        assert_eq!(error.to_string(), text);
    }
}

// covers: HER-EXP-003
#[test]
fn a_gain_stops_at_the_bound_and_the_scenarios_level_limit() {
    let r = ruleset();
    let params = r.hero_params().unwrap();
    // The cap: a limit's threshold; 0 or above 74 is none (A-8), the bound then.
    let caps = [
        (0, 1_810_034_206),
        (1, 0),
        (10, 14_700),
        (74, 1_508_362_195),
        (75, 1_810_034_206),
        (u8::MAX, 1_810_034_206),
    ];
    for (limit, cap) in caps {
        assert_eq!(experience_cap(params, limit), cap, "limit {limit}");
    }
    let mut d = details(&r);
    d.experience = 70;
    assert_eq!(gained_under_limit(&d, params, 100, 0), 170);
    assert_eq!(gained_under_limit(&d, params, u32::MAX, 0), 1_810_034_206);
    assert_eq!(gained_under_limit(&d, params, 100, 1), 0);
    assert_eq!(gained_under_limit(&d, params, 100_000, 10), 14_700);
    // Above the cap, any gain lowers the experience to it (D-19), not below the level's own
    // threshold (a hero above the limit, which the game's setup does not make).
    d.level = 2;
    d.experience = 1500;
    assert_eq!(gained_under_limit(&d, params, 0, 2), 1000);
    d.level = 3;
    d.experience = 2500;
    assert_eq!(gained_under_limit(&d, params, 0, 2), 2000);
    // As a change: without a limit the bound, with one its cap.
    let mut d = details(&r);
    apply_change(&mut d, &r, HeroChange::GainExperience(u32::MAX));
    assert_eq!(d.experience, 1_810_034_206);
    assert_eq!(validate(&d, &r), Ok(()));
    let mut d = details(&r);
    apply_change_limited(&mut d, &r, 10, HeroChange::GainExperience(u32::MAX));
    assert_eq!(d.experience, 14_700);
    apply_change_limited(&mut d, &r, 2, HeroChange::GainExperience(5));
    assert_eq!(d.experience, 1000);
}

// covers: HER-LVL-002
#[test]
fn the_originals_draws_follow_the_seed_byte_and_the_level() {
    // Survey §2.3: the Knight's level 2 with S = 1 and S = 200.
    let mut one = OriginalDraws::new(1, 2);
    assert_eq!(one.state(), 738_928);
    assert_eq!(one.between(1, 100), 3);
    let mut two = OriginalDraws::new(200, 2);
    assert_eq!(two.state(), 31_946_705);
    assert_eq!(two.between(1, 100), 91);
    // The highest seed: 214 013 × 74 + 156 823 × 255 + 154 079.
    assert_eq!(OriginalDraws::new(255, 74).state(), 55_980_906);
    // `rand` keeps bits 16 to 30 of the next state.
    let mut raw = OriginalDraws::new(0, 0);
    let next = 154_079_u32.wrapping_mul(214_013).wrapping_add(2_531_011);
    assert_eq!(raw.rand(), (next >> 16) & 0x7FFF);
    assert_eq!(raw.state(), next);
    // A single number takes no draw's range beyond itself; the owner's stream stays in range.
    assert_eq!(OriginalDraws::new(9, 9).between(7, 7), 7);
    let mut stream = Rng::from_seed(7);
    for _ in 0..50 {
        assert!((3..=9).contains(&stream.between(3, 9)));
    }
}

// covers: HER-LVL-001
#[test]
fn the_primary_skill_walks_the_class_chances() {
    let r = base();
    // The Knight at level 2 (survey §2.3): S = 1 draws 3, Attack; S = 200 draws 91, Knowledge.
    let d = hero(&r, "t:orrin", 1000, &[("leadership", 1)]);
    assert_eq!(original(&r, &d, 1, &[]).primary, PrimarySkill::Attack);
    assert_eq!(original(&r, &d, 200, &[]).primary, PrimarySkill::Knowledge);
    // 65 lies in Defense's share of the first row (35 + 45) and Spell Power's of the second
    // (30 + 30 + 20), which counts from the level-up to level 10 (D-15).
    let primary = |level: u8, number: u32| {
        let mut d = hero(
            &r,
            "t:orrin",
            experience_for(r.hero_params().unwrap(), level),
            &[],
        );
        d.level = level - 1;
        let mut draws = Fixed::new(&[number, 1, 1]);
        draw_level_up(&d, &r, &[], &mut draws).unwrap().primary
    };
    assert_eq!(primary(9, 65), PrimarySkill::Defense);
    assert_eq!(primary(10, 65), PrimarySkill::Power);
    assert_eq!(primary(10, 1), PrimarySkill::Attack);
    assert_eq!(primary(10, 100), PrimarySkill::Knowledge);
    // A number past every chance (no source of 1 to 100 gives one) stops at the last.
    assert_eq!(primary(10, 101), PrimarySkill::Knowledge);
}

// covers: HER-LVL-003
#[test]
fn the_offers_are_an_advance_and_a_new_skill() {
    let r = base();
    // Survey §2.4: the Knight knowing basic Leadership only.
    let d = hero(&r, "t:orrin", 1000, &[("leadership", 1)]);
    let up = original(&r, &d, 1, &[]);
    assert_eq!(
        up.offers,
        Offers::Two(s(&r, "leadership"), s(&r, "tactics"))
    );
    assert_eq!(up.level, 2);
    let up = original(&r, &d, 200, &[]);
    assert_eq!(
        up.offers,
        Offers::Two(s(&r, "leadership"), s(&r, "sorcery"))
    );
    // The draws: the primary, the advance among Leadership (10), the new skill among the 26
    // others (102 in all; Necromancy weighs 0 for the Knight).
    let mut draws = Fixed::new(&[1, 1, 102]);
    let up = draw_level_up(&d, &r, &[], &mut draws).unwrap();
    assert_eq!(draws.asked, [(1, 100), (1, 10), (1, 102)]);
    assert_eq!(
        up.offers,
        Offers::Two(s(&r, "leadership"), s(&r, "first_aid"))
    );
    // Without a skill to advance, the first is any skill below Expert (here a new one), the
    // second another new one.
    let bare = hero(&r, "t:orrin", 1000, &[]);
    let mut draws = Fixed::new(&[1, 1, 1]);
    let up = draw_level_up(&bare, &r, &[], &mut draws).unwrap();
    assert_eq!(draws.asked, [(1, 100), (1, 112), (1, 108)]);
    assert_eq!(
        up.offers,
        Offers::Two(s(&r, "pathfinding"), s(&r, "archery"))
    );
    // With eight skills the second is another advance; with seven at Expert, one offer; with
    // all eight at Expert, none.
    let eight: Vec<(&str, u8)> = BASE_SKILLS[..8].iter().map(|&k| (k, 1)).collect();
    let mut draws = Fixed::new(&[1, 1, 1]);
    let full = hero(&r, "t:orrin", 1000, &eight);
    let up = draw_level_up(&full, &r, &[], &mut draws).unwrap();
    assert_eq!(
        up.offers,
        Offers::Two(s(&r, "pathfinding"), s(&r, "archery"))
    );
    let mut expert = full.clone();
    expert.skills.iter_mut().skip(1).for_each(|(_, l)| *l = 3);
    let up = draw_level_up(&expert, &r, &[], &mut Fixed::new(&[1, 1])).unwrap();
    assert_eq!(up.offers, Offers::One(s(&r, "pathfinding")));
    expert.skills[0].1 = 3;
    let mut draws = Fixed::new(&[1]);
    let up = draw_level_up(&expert, &r, &[], &mut draws).unwrap();
    assert_eq!((up.offers, draws.asked.len()), (Offers::None, 1));
    // A banned skill weighs 0, or 1 when the hero knows it (D-18); banning every unknown skill
    // leaves the advance alone.
    let leadership = s(&r, "leadership");
    let others: Vec<SkillId> = (0..28)
        .map(SkillId::from_raw)
        .filter(|&k| k != leadership)
        .collect();
    let mut draws = Fixed::new(&[1, 1]);
    let up = draw_level_up(&d, &r, &[&others[..], &[leadership]].concat(), &mut draws).unwrap();
    assert_eq!(
        (up.offers, draws.asked),
        (Offers::One(leadership), vec![(1, 100), (1, 1)])
    );
    let up = draw_level_up(&d, &r, &others, &mut Fixed::new(&[1, 1])).unwrap();
    assert_eq!(up.offers, Offers::One(leadership));
    // A known skill weighs at least 1: the Knight's Necromancy (weight 0) known.
    let necro = hero(&r, "t:orrin", 1000, &[("necromancy", 1)]);
    let mut draws = Fixed::new(&[1, 1, 1]);
    let up = draw_level_up(&necro, &r, &[], &mut draws).unwrap();
    assert_eq!(draws.asked[1], (1, 1));
    assert_eq!(
        up.offers,
        Offers::Two(s(&r, "necromancy"), s(&r, "pathfinding"))
    );
}

// covers: HER-LVL-004
#[test]
fn wisdom_and_a_school_are_offered_after_the_class_intervals() {
    let r = base();
    let params = r.hero_params().unwrap();
    let at = |level: u8, skills: &[(&str, u8)]| {
        let mut d = hero(&r, "t:orrin", experience_for(params, level), skills);
        d.level = level - 1;
        d
    };
    // Level 4: a school is due (4 = 0 + the Knight's 4), Wisdom is not (6). Unknown, the
    // schools weigh their class weights (1, 3, 4, 2): the first offer (no advance) is drawn
    // among them, and its level is the school's before the second is drawn.
    let mut draws = Fixed::new(&[1, 2, 1]);
    let up = draw_level_up(&at(4, &[]), &r, &[], &mut draws).unwrap();
    assert_eq!(draws.asked, [(1, 100), (1, 10), (1, 109)]);
    assert_eq!(
        up.offers,
        Offers::Two(s(&r, "air_magic"), s(&r, "pathfinding"))
    );
    assert_eq!(
        up.offered,
        OfferLevels {
            wisdom: 0,
            school: 4
        }
    );
    // A known school weighs 1 (D-17): the advance of Earth Magic is a draw of 1 to 1.
    let mut draws = Fixed::new(&[1, 1, 1]);
    let up = draw_level_up(&at(4, &[("earth_magic", 1)]), &r, &[], &mut draws).unwrap();
    assert_eq!(draws.asked[1], (1, 1));
    assert_eq!(
        up.offers,
        Offers::Two(s(&r, "earth_magic"), s(&r, "pathfinding"))
    );
    // Level 6: Wisdom is due and offered outright, without a draw; the second is a school.
    let mut draws = Fixed::new(&[1, 10]);
    let up = draw_level_up(&at(6, &[]), &r, &[], &mut draws).unwrap();
    assert_eq!(draws.asked, [(1, 100), (1, 10)]);
    assert_eq!(
        up.offers,
        Offers::Two(s(&r, "wisdom"), s(&r, "earth_magic"))
    );
    assert_eq!(
        up.offered,
        OfferLevels {
            wisdom: 6,
            school: 6
        }
    );
    // Not again before the next interval; banned, not at all (the schools neither).
    let mut d = at(6, &[]);
    d.offered = OfferLevels {
        wisdom: 1,
        school: 3,
    };
    let up = draw_level_up(&d, &r, &[], &mut Fixed::new(&[1, 1, 1])).unwrap();
    assert_eq!(
        up.offers,
        Offers::Two(s(&r, "pathfinding"), s(&r, "archery"))
    );
    assert_eq!(up.offered, d.offered);
    let schools = &params.schools;
    let banned = [&schools[..], &[params.wisdom]].concat();
    let up = draw_level_up(&at(6, &[]), &r, &banned, &mut Fixed::new(&[1, 1, 1])).unwrap();
    assert_eq!(
        up.offers,
        Offers::Two(s(&r, "pathfinding"), s(&r, "archery"))
    );
    // Level 8: both due; the advance of Fire Magic first, then Wisdom outright as the new skill.
    let mut draws = Fixed::new(&[1, 1]);
    let up = draw_level_up(&at(8, &[("fire_magic", 1)]), &r, &[], &mut draws).unwrap();
    assert_eq!(draws.asked, [(1, 100), (1, 1)]);
    assert_eq!(up.offers, Offers::Two(s(&r, "fire_magic"), s(&r, "wisdom")));
    assert_eq!(
        up.offered,
        OfferLevels {
            wisdom: 8,
            school: 8
        }
    );
    // A school offered when none is due (by its weight) sets its level all the same.
    let mut draws = Fixed::new(&[1, 1, 1]);
    let up = draw_level_up(&at(2, &[("fire_magic", 1)]), &r, &[], &mut draws).unwrap();
    assert_eq!(draws.asked, [(1, 100), (1, 1), (1, 111)]);
    assert_eq!(
        up.offers,
        Offers::Two(s(&r, "fire_magic"), s(&r, "pathfinding"))
    );
    assert_eq!(
        up.offered,
        OfferLevels {
            wisdom: 0,
            school: 2
        }
    );
}

// covers: HER-LVL-001, HER-LVL-002, HER-LVL-003, HER-LVL-004, HER-LVL-005, HER-LVL-006
#[test]
fn batch_e3c_e3_6_levels_a_death_knight_from_its_seed() {
    // Survey §3.3: a Death Knight with 5 000 experience, 2/3/1/1 and eight basic skills, S = 75,
    // levels 2 to 5 chosen left, right, right, right; §10.6: the original's choice at load takes
    // them on the batch's map, all grass without water (Pathfinding and Navigation worth 0; values 0
    // against 0, 35 against 100, 0 against 50, 1 against 100).
    let r = base();
    let skills = [
        "pathfinding",
        "archery",
        "logistics",
        "scouting",
        "navigation",
        "mysticism",
        "offense",
        "armorer",
    ]
    .map(|k| (k, 1));
    let mut d = hero(&r, "t:dk", 5000, &skills);
    d.primary = [2, 3, 1, 1];
    let expected = [
        ("pathfinding", "navigation", Choice::First),
        ("archery", "logistics", Choice::Second),
        ("navigation", "scouting", Choice::Second),
        ("mysticism", "logistics", Choice::Second),
    ];
    for (level, (first, second, choice)) in (2..).zip(expected) {
        let up = original(&r, &d, 75, &[]);
        assert_eq!(up.level, level);
        assert_eq!(
            up.offers,
            Offers::Two(s(&r, first), s(&r, second)),
            "level {level}"
        );
        let chosen = original_choice(&d, &r, &up, ArmyWorth::SETUP, TerrainShares::default());
        assert_eq!(chosen, choice, "level {level}");
        // With the file's shares (1.0, as before any map is counted) levels 2 and 4 would take
        // Navigation (500 against Pathfinding's 250, then against Scouting's 50).
        let file = TerrainShares {
            tiles: 1,
            water: 1,
            rough: [1; 3],
        };
        let at_file = original_choice(&d, &r, &up, ArmyWorth::SETUP, file);
        let flips = level == 2 || level == 4;
        assert_eq!(at_file != choice, flips, "level {level}");
        d = answered(&r, d, &up, chosen);
    }
    assert_eq!(d.level, 5);
    assert_eq!(d.primary, [3, 6, 1, 1]);
    let advanced: Vec<(SkillId, u8)> = d.skills.iter().copied().filter(|&(_, l)| l > 1).collect();
    assert_eq!(
        advanced,
        [
            (s(&r, "pathfinding"), 2),
            (s(&r, "logistics"), 3),
            (s(&r, "scouting"), 2)
        ]
    );
    // Every level is given: none is drawn now.
    assert_eq!(
        draw_level_up(&d, &r, &[], &mut OriginalDraws::new(75, 6)),
        None
    );
}

// covers: HER-LVL-005, HER-CHG-001
#[test]
fn a_level_is_checked_and_applied() {
    let r = base();
    let d = hero(&r, "t:orrin", 2000, &[("leadership", 3), ("tactics", 1)]);
    let wisdom = s(&r, "wisdom");
    let gain = |learned, offered| HeroChange::GainLevel {
        primary: PrimarySkill::Power,
        learned,
        offered,
    };
    let none = OfferLevels::default();
    // A new skill at Basic after the others; a known one a level up; the offers' levels set.
    let after = {
        let mut d = d.clone();
        apply_change(
            &mut d,
            &r,
            gain(
                Some(wisdom),
                OfferLevels {
                    wisdom: 2,
                    school: 0,
                },
            ),
        );
        d
    };
    assert_eq!(after.level, 2);
    assert_eq!(after.primary, [2, 2, 2, 1]);
    assert_eq!(after.skills.last(), Some(&(wisdom, 1)));
    assert_eq!(
        after.offered,
        OfferLevels {
            wisdom: 2,
            school: 0
        }
    );
    let tactics = s(&r, "tactics");
    let mut again = after.clone();
    apply_change(&mut again, &r, gain(Some(tactics), none));
    assert_eq!(again.skills[1], (tactics, 2));
    assert_eq!(again.level, 3);
    assert_eq!(validate(&again, &r), Ok(()));
    // The primary skill saturates at `primary_max` (A-9).
    let mut top = d.clone();
    top.primary[2] = 99;
    apply_change(&mut top, &r, gain(None, none));
    assert_eq!((top.primary[2], top.level), (99, 2));
    // Refused: no level pending; a skill at Expert, unknown, or new to a hero with eight; an
    // offer's level above the new one.
    let refused = |d: &HeroDetails, change: HeroChange, error: ChangeError| {
        assert_eq!(check_change(d, &r, change), Err(error), "{change:?}");
        let mut after = d.clone();
        apply_change(&mut after, &r, change);
        assert_eq!(&after, d);
    };
    refused(&again, gain(None, none), ChangeError::NoLevelPending);
    let leadership = s(&r, "leadership");
    refused(
        &d,
        gain(Some(leadership), none),
        ChangeError::CannotLearn(leadership),
    );
    let unknown = SkillId::from_raw(99);
    refused(
        &d,
        gain(Some(unknown), none),
        ChangeError::CannotLearn(unknown),
    );
    let eight: Vec<(&str, u8)> = BASE_SKILLS[..8].iter().map(|&k| (k, 1)).collect();
    let full = hero(&r, "t:orrin", 1000, &eight);
    let luck = s(&r, "luck");
    refused(
        &full,
        gain(Some(luck), none),
        ChangeError::CannotLearn(luck),
    );
    let high = OfferLevels {
        wisdom: 0,
        school: 3,
    };
    refused(&d, gain(None, high), ChangeError::OfferedAboveLevel);
    // Without hero parameters no level is pending.
    let mut p = base_pack();
    p.hero_params = None;
    let without = compile(p).unwrap();
    assert_eq!(
        check_change(&d, &without, gain(None, none)),
        Err(ChangeError::NoLevelPending)
    );
    let texts = [
        (ChangeError::NoLevelPending, "no level is pending"),
        (
            ChangeError::CannotLearn(SkillId::from_raw(3)),
            "secondary skill 3 cannot be learned",
        ),
        (
            ChangeError::OfferedAboveLevel,
            "an offer's level exceeds the new level",
        ),
    ];
    for (error, text) in texts {
        assert_eq!(error.to_string(), text);
    }
}

// covers: HER-LVL-005
#[test]
fn the_answer_learns_the_chosen_offer() {
    let r = base();
    let (a, b) = (s(&r, "luck"), s(&r, "tactics"));
    let up = |offers| LevelUp {
        level: 2,
        primary: PrimarySkill::Attack,
        offers,
        offered: OfferLevels::default(),
    };
    let cases = [
        (Offers::None, Choice::First, None),
        (Offers::None, Choice::Second, None),
        (Offers::One(a), Choice::First, Some(a)),
        (Offers::One(a), Choice::Second, Some(a)),
        (Offers::Two(a, b), Choice::First, Some(a)),
        (Offers::Two(a, b), Choice::Second, Some(b)),
    ];
    for (offers, choice, learned) in cases {
        assert_eq!(up(offers).learned(choice), learned);
        assert_eq!(
            up(offers).change(choice),
            HeroChange::GainLevel {
                primary: PrimarySkill::Attack,
                learned,
                offered: OfferLevels::default(),
            }
        );
    }
    // Nothing to draw: no level pending, no hero parameters, an unknown hero type.
    let d = hero(&r, "t:orrin", 999, &[]);
    assert_eq!(draw_level_up(&d, &r, &[], &mut Fixed::new(&[])), None);
    let mut p = base_pack();
    p.hero_params = None;
    let without = compile(p).unwrap();
    let ready = hero(&r, "t:orrin", 1000, &[]);
    assert_eq!(
        draw_level_up(&ready, &without, &[], &mut Fixed::new(&[])),
        None
    );
    let stranger = HeroDetails {
        hero_type: HeroTypeId::from_raw(99),
        ..ready
    };
    assert_eq!(
        draw_level_up(&stranger, &r, &[], &mut Fixed::new(&[])),
        None
    );
}
