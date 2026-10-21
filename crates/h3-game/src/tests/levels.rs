//! Tests of the heroes' levels in the game (E08 T29): the level seeds and the draws' source by the
//! profile (GAM-HER-002, `ADV-QUIRK-9`), the levels at load (GAM-SET-002) under the scenario's
//! level limit and banned skills, the progression in the hash and in forks.

use std::collections::BTreeSet;

use h3_heroes::{
    ArmyWorth, HeroChange, HeroDetails, OriginalDraws, TerrainShares, apply_change_limited,
    draw_level_up, experience_bound, max_mana, original_choice,
};

use super::*;

const ONE: HeroId = HeroId::new(1);

/// [`rules`] under `profile` (`t:seeded`: a level's draws from the hero's seed, `ADV-QUIRK-9`;
/// the test pack's `t:faithful` has the flag off, as `strict-fixed`).
fn rules_in(profile: &str) -> GameRuleset {
    let core = Arc::new(core_with(false, profile));
    let part = part_with(&core, vec![88, 25], 2);
    let battle = battle_part(&core, 7);
    GameRuleset::new(core, Arc::new(part), Arc::new(battle)).unwrap()
}

/// The game of [`setup`] with `seed`, blue's hero edited by `edit`.
fn game(rules: &GameRuleset, seed: u64, edit: impl FnOnce(&mut GameSetup)) -> GameState {
    let mut setup = setup();
    edit(&mut setup);
    GameState::new(rules, &setup, seed).unwrap()
}

/// Blue's hero with the custom experience `experience`.
fn experienced(experience: u32) -> impl FnOnce(&mut GameSetup) {
    move |setup| setup.heroes[1].details.experience = Some(experience)
}

/// The levels at load as the hero model gives them: `start` (at level 1 and experience 0) gains
/// `experience` under `limit`, then each level drawn from the seed byte `seed` and chosen by the
/// original's choice with the setup's army worth and `shares`; the spell points full.
fn oracle(
    rules: &GameRuleset,
    start: &HeroDetails,
    experience: u32,
    seed: u8,
    shares: TerrainShares,
) -> HeroDetails {
    let core = rules.core();
    let mut details = start.clone();
    apply_change_limited(
        &mut details,
        core,
        0,
        HeroChange::GainExperience(experience),
    );
    loop {
        let mut draws = OriginalDraws::new(seed, details.level + 1);
        let Some(up) = draw_level_up(&details, core, &[], &mut draws) else {
            break;
        };
        let choice = original_choice(&details, core, &up, ArmyWorth::SETUP, shares);
        apply_change_limited(&mut details, core, 0, up.change(choice));
    }
    details.mana = max_mana(&details, core);
    details
}

// covers: GAM-HER-002, GAM-SET-002, HER-LVL-006
#[test]
fn in_faithful_the_levels_at_load_follow_the_heros_seed() {
    let rules = rules_in("t:seeded");
    let mut heroes = BTreeSet::new();
    for seed in 0..40 {
        // The same hero at experience 0: a custom experience draws nothing, so the seeds match.
        let start = game(&rules, seed, experienced(0));
        let levelled = game(&rules, seed, experienced(5000));
        let byte = levelled.level_seed(ONE);
        assert_eq!(start.level_seed(ONE), byte);
        let shares = levelled.adventure().map().terrain_shares();
        let expected = oracle(&rules, start.hero(ONE).unwrap(), 5000, byte, shares);
        let hero = levelled.hero(ONE).unwrap();
        assert_eq!(hero, &expected, "seed {seed}");
        // Levels 2 to 5 (thresholds 1 000, 2 000, 3 200, 4 600): four primary skills, offers.
        assert_eq!((hero.level, hero.experience), (5, 5000));
        assert_eq!(hero.primary.iter().map(|&p| u32::from(p)).sum::<u32>(), 8);
        assert!(!hero.skills.is_empty());
        // Red's hero has no custom experience: its default, at level 1.
        assert_eq!(levelled.hero(HeroId::new(0)).unwrap().level, 1);
        heroes.insert((hero.primary, hero.skills.clone()));
    }
    // The seed decides: not every game levels alike.
    assert!(heroes.len() > 1);
}

// covers: GAM-HER-002, GAM-SET-002
#[test]
fn in_strict_fixed_the_levels_at_load_draw_from_the_setup_stream() {
    let rules = rules();
    let games: Vec<GameState> = (0..60)
        .map(|seed| game(&rules, seed, experienced(5000)))
        .collect();
    for (seed, levelled) in (0..).zip(&games) {
        let hero = levelled.hero(ONE).unwrap();
        assert_eq!((hero.level, hero.experience), (5, 5000));
        assert_eq!(
            Some(hero),
            game(&rules, seed, experienced(5000)).hero(ONE),
            "the same seed, the same levels"
        );
    }
    // Not a function of the seed byte: two games whose hero has one byte level differently.
    let differ = games.iter().enumerate().any(|(i, a)| {
        games[i + 1..]
            .iter()
            .any(|b| a.level_seed(ONE) == b.level_seed(ONE) && a.hero(ONE) != b.hero(ONE))
    });
    assert!(differ);
}

// covers: GAM-SET-002, HER-EXP-003
#[test]
fn the_levels_at_load_stop_at_the_scenarios_limit() {
    let rules = rules_in("t:seeded");
    let core = rules.core();
    let at = |limit, experience| {
        let game = game(&rules, 3, |setup| {
            setup.level_limit = limit;
            experienced(experience)(setup);
        });
        assert_eq!(game.level_limit(), limit);
        game.hero(ONE).unwrap().clone()
    };
    // Capped at level 3's threshold (D-21: no hero starts above the limit).
    let capped = at(3, 5000);
    assert_eq!((capped.level, capped.experience), (3, 2000));
    // No limit, or one above the highest level (`ADV-QUIRK-12`).
    for limit in [0, 75, u8::MAX] {
        let free = at(limit, 5000);
        assert_eq!((free.level, free.experience), (5, 5000), "limit {limit}");
    }
    // Below level 2: no level, the experience as given.
    let low = at(0, 999);
    assert_eq!((low.level, low.experience), (1, 999));
    // Any experience: at most the bound and level 74, the details valid, the spell points full.
    let top = at(0, u32::MAX);
    let params = core.hero_params().unwrap();
    assert_eq!((top.level, top.experience), (74, experience_bound(params)));
    assert_eq!(h3_heroes::validate(&top, core), Ok(()));
    assert_eq!(top.mana, max_mana(&top, core));
    assert!(top.mana > at(0, 0).mana, "Knowledge rose with the levels");
}

// covers: GAM-SET-002, HER-LVL-003
#[test]
fn a_banned_skill_is_never_learned_at_load() {
    let rules = rules_in("t:seeded");
    let core = rules.core();
    let [wits, lore, dark, sense] =
        ["t:wits", "t:lore", "t:dark", "t:sense"].map(|k| core.skill_by_key(k).unwrap());
    let learned = |banned: &[SkillId], seed| {
        let game = game(&rules, seed, |setup| {
            setup.banned_skills = banned.to_vec();
            experienced(20_600)(setup);
        });
        game.hero(ONE).unwrap().skills.clone()
    };
    // Banned in any order, a skill twice: only Sense remains to learn.
    for seed in 0..20 {
        let skills = learned(&[dark, wits, lore, wits], seed);
        assert!(!skills.is_empty());
        assert!(
            skills.iter().all(|&(skill, _)| skill == sense),
            "seed {seed}"
        );
    }
    assert!((0..20).any(|seed| learned(&[], seed).iter().any(|&(skill, _)| skill == wits)));
    let banned = game(&rules, 0, |setup| {
        setup.banned_skills = vec![dark, wits, lore, wits]
    });
    assert_eq!(banned.banned_skills(), [wits, lore, dark]);
}

// covers: GAM-HER-002
#[test]
fn the_progression_is_hashed_and_shared_by_forks() {
    let rules = rules();
    let base = game(&rules, 3, |_| {});
    assert_eq!((base.level_limit(), base.banned_skills()), (0, &[][..]));
    let [first, second] = [0, 1].map(SkillId::from_raw);
    let limited = game(&rules, 3, |setup| setup.level_limit = 3);
    assert_ne!(limited.state_hash(), base.state_hash());
    let banned = game(&rules, 3, |setup| setup.banned_skills = vec![second, first]);
    assert_ne!(banned.state_hash(), base.state_hash());
    let again = game(&rules, 3, |setup| {
        setup.banned_skills = vec![first, second, first];
    });
    assert_eq!(
        again.state_hash(),
        banned.state_hash(),
        "a set: order and repeats"
    );
    // A fork shares the progression; it is never copied.
    let fork = base.clone();
    assert!(fork.shares_progression(&base));
    assert!(!limited.shares_progression(&base));
    // The seeds: 1 to 255, a draw per hero.
    let seeds: BTreeSet<u8> = (0..300)
        .flat_map(|seed| {
            let game = game(&rules, seed, |_| {});
            [game.level_seed(HeroId::new(0)), game.level_seed(ONE)]
        })
        .collect();
    assert!(seeds.len() > 200 && !seeds.contains(&0));
    assert_eq!(base.level_seed(HeroId::new(2)), 1, "no such hero");
}
