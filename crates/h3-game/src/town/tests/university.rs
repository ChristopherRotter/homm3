//! Tests of a town's Magic University (E09 T59b1, design note `E09-design.md` §112;
//! TWN-UNI-001, GAM-DEC-006 amended): `TownCommand::University` opens E08's University decision
//! for the town's hero it names (the visitor or the garrison hero, the actor's) with the town as
//! its place, the part's skills (a school the map bans left out with `TWN-QUIRK-16`'s flag off)
//! at the part's price; its purchases are E08's; each refusal changes nothing; the legal commands
//! list each opening whose University sells a slot the treasury pays. On the town tests' game
//! (red's T0 on (1, 1) and blue's T1 on (3, 0), each built by its setup's slots only) with a
//! University (map slot 23) teaching Wits and Lore at 2 000 gold.

use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use h3_adventure::{AdventureCommand, Pos};
use h3_content::{ProfileDef, ResourceId, Ruleset, SkillId};
use h3_contracts::{HeroId, PlayerId, TownId};
use h3_heroes::HeroChange;
use h3_towns::{BuildingDef, BuildingRoles, TownCommand, TownCommandError, TownHeroChange};

use super::command::{RED, T0, T1, run};
use super::learn::{BLUE_HERO, RED_HERO, game, setup, town_command};
use super::{rules_edited, town_pack, towns_core_pack};
use crate::tests::key;
use crate::{
    Actor, Answer, GameCommand, GameCommandError, GameEvent, GameRuleset, GameState,
    UniversityAnswer, UniversityPlace, UniversityVisit,
};

const GOLD: ResourceId = ResourceId::from_raw(0);
/// The University's map slot.
const UNIVERSITY_SLOT: u8 = 23;

/// The tests' rules under `profile`: the test pack's `t:faithful` (no quirk flag on: a banned
/// school is left out), `t:bans` (`TWN-QUIRK-16`'s flag on: offered) or `t:visitor_only`
/// (`TWN-QUIRK-15`'s on).
fn rules_in(profile: &str) -> GameRuleset {
    let mut core = towns_core_pack(false);
    for (profile, quirk) in [
        ("t:bans", "quirk.town.university_ignores_map_bans"),
        ("t:visitor_only", "quirk.town.visitor_only_trades"),
    ] {
        core.profiles.push(ProfileDef {
            key: key(profile),
            quirks: vec![(quirk.to_string(), true)],
        });
    }
    let core = Ruleset::compile(&[core], profile).unwrap();
    let mut towns = town_pack(false);
    towns.towns[0].buildings.push(BuildingDef {
        key: "university".into(),
        name: "University".into(),
        slot: Some(UNIVERSITY_SLOT),
        supersedes: vec![],
        forbidden_with: None,
        auto_with: vec![],
        requires: vec![],
        cost: vec![],
        roles: BuildingRoles {
            university: true,
            ..BuildingRoles::default()
        },
        war_machine: None,
    });
    let params = towns.params.as_mut().unwrap();
    params.university_skills = vec![key("t:wits"), key("t:lore")];
    rules_edited(core, &towns, false, 4, |_| {})
}

fn rules() -> GameRuleset {
    rules_in("t:faithful")
}

fn skill(rules: &GameRuleset, k: &str) -> SkillId {
    rules.core().skill_by_key(k).unwrap()
}

fn university(town: TownId, hero: HeroId) -> GameCommand {
    town_command(TownCommand::University { town, hero })
}

/// The game with both towns built by `slots`, red's hero on T0's entrance, the scenario banning
/// `banned`; red holds 10 000 gold.
fn built(rules: &GameRuleset, slots: &[u8], banned: &[&str]) -> GameState {
    let mut setup = setup(rules, slots);
    setup.banned_skills = banned.iter().map(|k| skill(rules, k)).collect();
    game(rules, &setup)
}

/// Red's listed openings, in the listing's order.
fn listed(rules: &GameRuleset, game: &GameState) -> Vec<GameCommand> {
    crate::legal_commands(game, rules, Actor::Player(RED))
        .into_iter()
        .filter(|c| matches!(c, GameCommand::Town(TownCommand::University { .. })))
        .collect()
}

/// `player`'s opening at `town` for `hero` refused with `error` (`run` checks the state
/// unchanged).
fn refused(
    rules: &GameRuleset,
    game: &mut GameState,
    (player, town, hero): (PlayerId, TownId, HeroId),
    error: TownCommandError,
) {
    assert_eq!(
        run(rules, game, player, university(town, hero)),
        Err(GameCommandError::Town(error))
    );
}

fn answer(answer: UniversityAnswer) -> GameCommand {
    GameCommand::Answer(Answer::University(answer))
}

// covers: TWN-UNI-001, GAM-DEC-006
#[test]
fn the_visitor_opens_the_towns_university_and_buys_at_its_price() {
    let rules = rules();
    let (wits, lore) = (skill(&rules, "t:wits"), skill(&rules, "t:lore"));
    let mut game = built(&rules, &[UNIVERSITY_SLOT], &[]);
    assert_eq!(listed(&rules, &game), [university(T0, RED_HERO)]);
    let events = run(&rules, &mut game, RED, university(T0, RED_HERO)).unwrap();
    assert_eq!(
        events,
        [GameEvent::UniversityOpened {
            player: RED,
            hero: RED_HERO,
            place: UniversityPlace::Town(T0),
        }]
    );
    let visit = UniversityVisit {
        hero: RED_HERO,
        place: UniversityPlace::Town(T0),
        skills: vec![wits, lore],
        price: vec![(GOLD, 2000)],
    };
    assert_eq!(game.university(RED), Some(&visit));
    // E08's purchase: the price, then the skill at Basic; the University stays open.
    let events = run(&rules, &mut game, RED, answer(UniversityAnswer::Buy(1))).unwrap();
    assert_eq!(
        events,
        [
            GameEvent::ResourcePaid {
                player: RED,
                resource: GOLD,
                amount: 2000,
            },
            GameEvent::HeroChanged {
                player: RED,
                hero: RED_HERO,
                change: HeroChange::RaiseSkill {
                    skill: lore,
                    level: 1,
                },
            },
        ]
    );
    assert_eq!(game.university(RED), Some(&visit));
    assert_eq!(game.treasury(RED).unwrap()[GOLD.index()], 8_000);
    run(&rules, &mut game, RED, answer(UniversityAnswer::Leave)).unwrap();
    assert_eq!(game.university(RED), None);
    // Any number a turn: listed again, Wits still open.
    assert_eq!(listed(&rules, &game), [university(T0, RED_HERO)]);
    let before = game.state_hash();
    run(&rules, &mut game, RED, university(T0, RED_HERO)).unwrap();
    assert!(game.university(RED).is_some());
    assert_ne!(game.state_hash(), before);
}

// covers: TWN-UNI-001
#[test]
fn each_refusal_changes_nothing_in_its_order() {
    let rules = rules();
    let mut game = built(&rules, &[UNIVERSITY_SLOT], &[]);
    refused(
        &rules,
        &mut game,
        (RED, TownId::new(9), RED_HERO),
        TownCommandError::UnknownTown,
    );
    refused(
        &rules,
        &mut game,
        (RED, T1, RED_HERO),
        TownCommandError::NotOwner,
    );
    let mut bare = built(&rules, &[], &[]);
    assert!(listed(&rules, &bare).is_empty());
    refused(
        &rules,
        &mut bare,
        (RED, T0, RED_HERO),
        TownCommandError::NoUniversity,
    );
    // Another player's hero, a hero the game lacks.
    for hero in [BLUE_HERO, HeroId::new(40)] {
        refused(
            &rules,
            &mut game,
            (RED, T0, hero),
            TownCommandError::NotTownHero,
        );
    }
    // A decision open: the turn's check first.
    let mut open = game.clone();
    run(&rules, &mut open, RED, university(T0, RED_HERO)).unwrap();
    assert_eq!(
        run(&rules, &mut open, RED, university(T0, RED_HERO)),
        Err(GameCommandError::DecisionOpen)
    );
    // No hero in the town: the hero leaves the entrance.
    let step = GameCommand::Adventure(AdventureCommand::MoveHero {
        hero: RED_HERO,
        path: vec![Pos::new(1, 2, 0)],
    });
    run(&rules, &mut game, RED, step).unwrap();
    assert!(listed(&rules, &game).is_empty());
    refused(
        &rules,
        &mut game,
        (RED, T0, RED_HERO),
        TownCommandError::NotTownHero,
    );
}

// covers: TWN-UNI-001
#[test]
fn the_garrison_hero_studies_whatever_the_visitor_only_flag() {
    // The University takes the garrison hero (survey §6.2), not `TWN-QUIRK-15`'s visitor alone.
    for profile in ["t:faithful", "t:visitor_only"] {
        let rules = rules_in(profile);
        let mut game = built(&rules, &[UNIVERSITY_SLOT], &[]);
        let enter = town_command(TownCommand::Heroes {
            town: T0,
            change: TownHeroChange::Enter,
        });
        run(&rules, &mut game, RED, enter).unwrap();
        assert!(game.adventure().town_visitor(T0).is_none());
        assert_eq!(listed(&rules, &game), [university(T0, RED_HERO)]);
        run(&rules, &mut game, RED, university(T0, RED_HERO)).unwrap();
        assert_eq!(game.university(RED).unwrap().hero, RED_HERO);
    }
}

// covers: TWN-UNI-001
#[test]
fn a_banned_school_is_offered_only_with_the_flag_on() {
    // `TWN-QUIRK-16` (T-23): off (`strict-fixed`, the test pack's `t:faithful`) the scenario's
    // banned Wits is left out; on (`faithful`, `t:bans`) offered and taught.
    let (wits, lore) = {
        let rules = rules();
        (skill(&rules, "t:wits"), skill(&rules, "t:lore"))
    };
    for (profile, skills) in [("t:faithful", vec![lore]), ("t:bans", vec![wits, lore])] {
        let rules = rules_in(profile);
        let mut game = built(&rules, &[UNIVERSITY_SLOT], &["t:wits"]);
        run(&rules, &mut game, RED, university(T0, RED_HERO)).unwrap();
        assert_eq!(game.university(RED).unwrap().skills, skills, "{profile}");
    }
}

// covers: TWN-UNI-001
#[test]
fn an_opening_is_listed_only_where_a_slot_is_open_at_a_price_paid() {
    let rules = rules();
    let mut game = built(&rules, &[UNIVERSITY_SLOT], &["t:wits"]);
    // The treasury below the price: not listed, yet opened (the original's dialog opens and
    // disables the purchase).
    let mut poor = game.clone();
    poor.set_resource(RED, GOLD, 1999);
    assert!(listed(&rules, &poor).is_empty());
    run(&rules, &mut poor, RED, university(T0, RED_HERO)).unwrap();
    // Lore learned, Wits banned (the flag off): no slot open, not listed; still opened.
    run(&rules, &mut game, RED, university(T0, RED_HERO)).unwrap();
    run(&rules, &mut game, RED, answer(UniversityAnswer::Buy(0))).unwrap();
    run(&rules, &mut game, RED, answer(UniversityAnswer::Leave)).unwrap();
    assert!(listed(&rules, &game).is_empty());
    run(&rules, &mut game, RED, university(T0, RED_HERO)).unwrap();
    assert_eq!(game.university(RED).unwrap().skills.len(), 1);
}
