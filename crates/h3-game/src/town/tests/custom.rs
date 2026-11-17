//! Tests of the map's customised hero types (E09 T37b, decisions 137, 138; design note
//! `E09-design.md` §70): every hero the build makes of such a type takes its entry (GAM-HER-001),
//! a placed or random hero's own customisations over it, and each such type still free gets its
//! record at the build, which the taverns offer as the map sets it (TWN-SET-005, TWN-TAV-002),
//! its levels at load in the build's order (GAM-SET-002, GAM-HER-002); the setup's checks.

use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use h3_content::{HeroTypeId, Ruleset};
use h3_contracts::{HeroId, HeroSex, PlayerId};
use h3_heroes::{DetailsError, max_mana};

use super::{faction, hero, rules_of, start, town_pack, towns_core_pack, towns_rules, towns_setup};
use crate::tests::key;
use crate::tests::levels::oracle;
use crate::{GameRuleset, GameSetupError, GameState, HeroDetailsSetup};

const RED: PlayerId = PlayerId::new(0);

/// The town test rules under `profile` (`t:seeded`: a level's draws from the hero's level seed,
/// `ADV-QUIRK-9`; `t:faithful` has the flag off), `t:town`'s tavern weights the knight 3 and the
/// cleric 2 (the tavern tests').
fn rules_in(profile: &str) -> GameRuleset {
    let mut towns = town_pack(false);
    for record in &mut towns.towns {
        record.tavern = vec![(key("t:knight"), 3), (key("t:cleric"), 2)];
    }
    let core = Ruleset::compile(&[towns_core_pack(false)], profile).unwrap();
    rules_of(core, &towns, false, 4)
}

/// An entry customising only the experience.
fn experienced(experience: u32) -> HeroDetailsSetup {
    HeroDetailsSetup {
        experience: Some(experience),
        ..HeroDetailsSetup::default()
    }
}

/// The sum of a hero's base primary skills.
fn primary_sum(primary: [u8; 4]) -> u32 {
    primary.iter().map(|&p| u32::from(p)).sum()
}

// covers: TWN-SET-005, GAM-HER-001, GAM-SET-002, TWN-TAV-002
#[test]
fn a_customised_type_still_free_has_its_record_at_the_build_and_is_offered_as_set() {
    let rules = rules_in("t:faithful");
    let core = rules.core();
    let [orrin, loner, crowd, sage, c0] =
        ["t:orrin", "t:loner", "t:crowd", "t:sage", "t:c0"].map(|k| hero(&rules, k));
    let mut setup = towns_setup(&rules);
    // The map does not allow the loner, and a prison holds the crowd: no record for either.
    setup.allowed_heroes.retain(|&t| t != loner);
    setup.prisoners = vec![crowd];
    setup.custom_types = vec![
        (
            c0,
            HeroDetailsSetup {
                sex: Some(HeroSex::Female),
                ..experienced(2000)
            },
        ),
        (orrin, experienced(1000)),
        (
            sage,
            HeroDetailsSetup {
                primary: Some([4, 3, 2, 1]),
                ..HeroDetailsSetup::default()
            },
        ),
        (loner, experienced(2000)),
        (crowd, experienced(2000)),
    ];
    let game = GameState::new(&rules, &setup, 3).unwrap();
    let pool = game.pool();
    // Orrin's two placed heroes take his entry (level 2 at 1 000); his record is the first's.
    for id in [0, 1] {
        let placed = game.hero(HeroId::new(id)).unwrap();
        assert_eq!((placed.experience, placed.level), (1000, 2));
    }
    assert_eq!(pool.record(orrin), Some(HeroId::new(0)));
    // The free customised types' records follow the two placed heroes, in type order.
    assert_eq!(pool.record(sage), Some(HeroId::new(2)));
    assert_eq!(pool.record(c0), Some(HeroId::new(3)));
    assert_eq!((pool.record(loner), pool.record(crowd)), (None, None));
    let sage_record = game.hero(HeroId::new(2)).unwrap();
    assert_eq!(
        (
            sage_record.hero_type,
            sage_record.primary,
            sage_record.level
        ),
        (sage, [4, 3, 2, 1], 1)
    );
    assert!((40..=90).contains(&sage_record.experience));
    // c0's at 2 000 (thresholds 1 000, 2 000: level 3), her spell points full, her sex the
    // entry's (GAM-BAT-012).
    let offered = game.hero(HeroId::new(3)).unwrap();
    assert_eq!(
        (offered.hero_type, offered.experience, offered.level),
        (c0, 2000, 3)
    );
    assert_eq!(primary_sum(offered.primary), 4 + 2);
    assert_eq!(offered.mana, max_mana(offered, core));
    assert_eq!(game.hero_sex(HeroId::new(3)), Some(HeroSex::Female));
    // The start's fill offers c0 to red's slot 1 at that record, as the map sets it: the taverns'
    // stream and the pool's statuses are the plain setup's, so are the offers.
    let mut plain = towns_setup(&rules);
    plain.allowed_heroes.retain(|&t| t != loner);
    plain.prisoners = vec![crowd];
    let without = GameState::new(&rules, &plain, 3).unwrap();
    let slots = |g: &GameState| g.pool().tavern(RED).unwrap().slots;
    assert_eq!(slots(&game), slots(&without));
    assert_eq!(slots(&game)[1], Some(c0));
    assert_eq!(game.pool().record(c0), Some(HeroId::new(3)));
    // One hero more than the plain game's store: the sage's record, made at the build and not
    // offered (c0's took the id the plain game's fill gave it).
    assert_eq!(game.heroes().len(), without.heroes().len() + 1);
}

// covers: TWN-SET-009, TWN-SET-008, GAM-HER-001, GAM-SET-002
#[test]
fn a_main_towns_hero_and_a_random_hero_take_their_drawn_types_entries() {
    let rules = towns_rules();
    let [gunner, sage] = ["t:gunner", "t:sage"].map(|k| hero(&rules, k));
    let mut setup = towns_setup(&rules);
    setup.players[0].start = start(faction(&rules, "t:town"), true);
    setup.allowed_heroes = vec![gunner, sage];
    // Blue's placed hero is random, with primary skills of its own.
    setup.heroes[1].hero_type = None;
    setup.heroes[1].details.primary = Some([3, 3, 3, 3]);
    let entry = HeroDetailsSetup {
        primary: Some([9, 9, 9, 9]),
        ..experienced(2000)
    };
    setup.custom_types = vec![(gunner, entry.clone()), (sage, entry)];
    let mut types = BTreeSet::new();
    for seed in 0..8 {
        let game = GameState::new(&rules, &setup, seed).unwrap();
        // Blue's random hero (hero 1): its own primary skills over its drawn type's experience
        // (its two levels at load add one point each).
        let random = game.hero(HeroId::new(1)).unwrap();
        assert_eq!((random.experience, random.level), (2000, 3));
        assert_eq!(primary_sum(random.primary), 12 + 2);
        // Red's main town's hero (hero 2): the other type, its entry whole.
        let main = game.hero(HeroId::new(2)).unwrap();
        assert_eq!((main.experience, main.level), (2000, 3));
        assert_eq!(primary_sum(main.primary), 36 + 2);
        assert_ne!(main.hero_type, random.hero_type);
        types.insert(random.hero_type);
        // Both types taken: no record, no tavern hero.
        assert_eq!(game.heroes().len(), 3);
        assert_eq!(game.pool().tavern(RED).unwrap().slots, [None; 2]);
    }
    assert_eq!(types.len(), 2, "the random hero draws either type");
}

// covers: GAM-SET-002, GAM-HER-002
#[test]
fn in_faithful_a_records_levels_at_load_follow_its_level_seed() {
    let rules = rules_in("t:seeded");
    let c0 = hero(&rules, "t:c0");
    let game = |seed, experience| {
        let mut setup = towns_setup(&rules);
        setup.custom_types = vec![(c0, experienced(experience))];
        GameState::new(&rules, &setup, seed).unwrap()
    };
    let mut levelled_alike = BTreeSet::new();
    for seed in 0..24 {
        // The same record at experience 0: a custom experience draws nothing, so the level
        // seeds match; the seed drawn before the levels, which read it.
        let start = game(seed, 0);
        let levelled = game(seed, 5000);
        let id = levelled.pool().record(c0).unwrap();
        assert_eq!(id, HeroId::new(2));
        let byte = levelled.level_seed(id);
        assert_eq!(start.level_seed(id), byte);
        let shares = levelled.adventure().map().terrain_shares();
        let expected = oracle(&rules, start.hero(id).unwrap(), 5000, byte, shares);
        let record = levelled.hero(id).unwrap();
        assert_eq!(record, &expected, "seed {seed}");
        assert_eq!((record.level, record.experience), (5, 5000));
        levelled_alike.insert((record.primary, record.skills.clone()));
    }
    assert!(levelled_alike.len() > 1, "the seed decides");
}

// covers: TWN-TAV-002, HER-LVL-003
#[test]
fn a_tavern_hero_keeps_a_starting_skill_its_map_bans() {
    let rules = rules_in("t:faithful");
    let (sage, wits) = (
        hero(&rules, "t:sage"),
        rules.core().skill_by_key("t:wits").unwrap(),
    );
    let mut setup = towns_setup(&rules);
    setup.allowed_heroes = vec![sage];
    setup.banned_skills = vec![wits];
    let game = GameState::new(&rules, &setup, 0).unwrap();
    // Red's slot 0, its own faction's knight: the sage with its Expert Wits (no ban touches a
    // type's starting skills: the reading, VCMI agreeing; survey `E09-tavern.md` §5).
    let slots = game.pool().tavern(RED).unwrap().slots;
    assert_eq!(slots, [Some(sage), None]);
    let record = game.hero(game.pool().record(sage).unwrap()).unwrap();
    assert_eq!(record.skills, vec![(wits, 3)]);
}

// covers: GAM-HER-001
#[test]
fn a_customised_type_is_the_rulesets_named_once_and_valid_on_its_own() {
    let rules = towns_rules();
    let sage = hero(&rules, "t:sage");
    let unknown = HeroTypeId::from_index(rules.core().hero_types().len()).unwrap();
    let wits = rules.core().skill_by_key("t:wits").unwrap();
    let refused = |custom_types: Vec<(HeroTypeId, HeroDetailsSetup)>| {
        let mut setup = towns_setup(&rules);
        setup.custom_types = custom_types;
        setup.validate(&rules).unwrap_err()
    };
    let error = refused(vec![(unknown, HeroDetailsSetup::default())]);
    assert_eq!(error, GameSetupError::CustomType(unknown));
    assert_eq!(
        error.to_string(),
        format!(
            "customised hero type {} is unknown or named twice",
            unknown.raw()
        )
    );
    let twice = vec![(sage, experienced(10)), (sage, experienced(20))];
    assert_eq!(refused(twice), GameSetupError::CustomType(sage));
    // Checked on its own, whatever takes it: an Expert level past Expert (skill levels 1 to 3).
    let bad = HeroDetailsSetup {
        skills: Some(vec![(wits, 4)]),
        ..HeroDetailsSetup::default()
    };
    let error = refused(vec![(sage, bad.clone())]);
    let level = GameSetupError::CustomDetails(sage, DetailsError::SkillLevel(wits));
    assert_eq!(error, level);
    assert!(
        error
            .to_string()
            .starts_with(&format!("customised hero type {}: ", sage.raw()))
    );
    // A type the map does not allow is checked too (no hero takes it).
    let mut setup = towns_setup(&rules);
    setup.allowed_heroes.retain(|&t| t != sage);
    setup.custom_types = vec![(sage, bad)];
    assert_eq!(setup.validate(&rules), Err(level));
}
