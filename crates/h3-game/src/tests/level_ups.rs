//! Tests of the level-ups in play (E08 T30): the battle's winner's pending levels (GAM-BAT-008),
//! given one at a time, a level with two offers a pending decision of the hero's owner answered
//! `First` or `Second`, one with one offer or none given at once (GAM-DEC-002); the draws' source
//! in play (GAM-HER-002, `ADV-QUIRK-9`); the events and their projection, the observation, the
//! legal commands, the system actor's default; the decision in the hash.

use h3_contracts::{Army, BattleOutcome, BattleResult, BattleSide, StackOrigin, StackRecord};
use h3_heroes::{
    ArmyWorth, Choice, HeroChange, HeroDetails, LevelUp, Offers, OriginalDraws, apply_change,
    draw_level_up, gained_experience, original_choice,
};

use super::decisions::{self, play};
use super::*;

const RED: PlayerId = PlayerId::new(0);
const BLUE: PlayerId = PlayerId::new(1);
const RED_HERO: HeroId = HeroId::new(0);
const BLUE_HERO: HeroId = HeroId::new(1);
const GOG: CreatureTypeId = CreatureTypeId::from_raw(1);
const FIRST: GameCommand = GameCommand::Answer(Answer::First);
const SECOND: GameCommand = GameCommand::Answer(Answer::Second);

/// The sides of a battle red's hero (the attacker) and blue's fought, and its tile.
const SIDES: ([Option<PlayerId>; 2], [Option<HeroId>; 2], Pos) = (
    [Some(RED), Some(BLUE)],
    [Some(RED_HERO), Some(BLUE_HERO)],
    Pos::new(0, 0, 0),
);

/// [`rules`] under `profile` (`t:seeded`: a level's draws from the hero's seed, `ADV-QUIRK-9`;
/// the test pack's `t:faithful` has the flag off, as `strict-fixed`: from the owner's stream).
fn rules_in(profile: &str) -> GameRuleset {
    let core = Arc::new(core_with(false, profile));
    let part = part_with(&core, vec![88, 25], 2);
    let battle = battle_part(&core, 7);
    GameRuleset::new(core, Arc::new(part), Arc::new(battle)).unwrap()
}

/// The game of [`setup`] with seed 5, both heroes at the custom experience `experience` (so
/// level 1 below 1 000) and `banned` banned.
fn game(rules: &GameRuleset, experience: u32, banned: &[&str]) -> GameState {
    let mut setup = setup();
    for hero in &mut setup.heroes {
        hero.details.experience = Some(experience);
    }
    setup.banned_skills = banned
        .iter()
        .map(|k| rules.core().skill_by_key(k).unwrap())
        .collect();
    GameState::new(rules, &setup, 5).unwrap()
}

/// A victory of `side` over the other side's hero: its army keeps an imp, the other lost `gogs`
/// gogs (13 hit points each).
fn won_by(side: BattleSide, gogs: u32) -> BattleOutcome {
    let mut survivors = [Army { stacks: vec![] }, Army { stacks: vec![] }];
    survivors[side.index()].stacks = vec![ArmyStack {
        slot: 1,
        creature: IMP,
        count: 1,
    }];
    let mut stacks = [vec![], vec![]];
    stacks[side.opponent().index()] = vec![StackRecord {
        origin: StackOrigin::Army(0),
        creature: GOG,
        start: gogs,
        end: 0,
    }];
    BattleOutcome {
        result: BattleResult::Victory(side),
        survivors,
        war_machines: [vec![], vec![]],
        stacks,
        surrender_cost: None,
        heroes: [None, None],
        rounds: 1,
    }
}

/// `game` after `outcome`, with the events.
fn end(
    game: &GameState,
    rules: &GameRuleset,
    outcome: &BattleOutcome,
) -> (GameState, Vec<GameEvent>) {
    let mut ended = game.clone();
    let mut events = Vec::new();
    ended.end_battle(rules, SIDES, outcome, &mut events);
    (ended, events)
}

/// `hero`'s details in `game` after it gains the experience of a victory over a hero that lost
/// `gogs` gogs (GAM-BAT-002), before any level.
fn experienced(rules: &GameRuleset, game: &GameState, hero: HeroId, gogs: u32) -> HeroDetails {
    let mut details = game.hero(hero).unwrap().clone();
    let gained = gained_experience(&details, rules.core(), 13 * gogs + 500);
    apply_change(
        &mut details,
        rules.core(),
        HeroChange::GainExperience(gained),
    );
    details
}

/// The level events of `events`: the opened level-ups and the levels gained.
fn levels(events: &[GameEvent]) -> Vec<GameEvent> {
    events
        .iter()
        .filter(|e| {
            matches!(
                e,
                GameEvent::LevelUpOpened { .. } | GameEvent::LevelGained { .. }
            )
        })
        .copied()
        .collect()
}

/// The event that opens `level_up` of `hero` for `player`.
fn opened(player: PlayerId, hero: HeroId, level_up: &LevelUp) -> GameEvent {
    let Offers::Two(first, second) = level_up.offers else {
        panic!("{level_up:?} has no two offers");
    };
    GameEvent::LevelUpOpened {
        player,
        hero,
        level: level_up.level,
        primary: level_up.primary,
        offers: [first, second],
    }
}

// covers: GAM-BAT-008, GAM-DEC-002, GAM-HER-002
#[test]
fn a_won_battle_opens_the_winners_level_up_which_its_answer_gives() {
    let rules = rules();
    let game = game(&rules, 950, &[]);
    let (ended, events) = end(&game, &rules, &won_by(BattleSide::Attacker, 2));
    // In strict-fixed (the test pack's `t:faithful`) the level is drawn from the owner's stream.
    let details = experienced(&rules, &game, RED_HERO, 2);
    let mut stream = game.stream(RED);
    let drawn = draw_level_up(&details, rules.core(), &[], &mut stream).unwrap();
    assert_eq!(ended.stream(RED), stream);
    assert_eq!(ended.stream(BLUE), game.stream(BLUE));
    let event = opened(RED, RED_HERO, &drawn);
    // After the battle's end and its experience, the last of the outcome.
    assert_eq!(levels(&events), [event]);
    assert_eq!(events.last(), Some(&event));
    assert_eq!(project(&event, Viewer::Player(RED)), Some(event));
    assert_eq!(project(&event, Viewer::Player(BLUE)), None);
    assert_eq!(ended.hero(RED_HERO), Some(&details));
    let Offers::Two(first, second) = drawn.offers else {
        panic!("{drawn:?} has no two offers");
    };
    let view = LevelUpView {
        hero: RED_HERO,
        level: 2,
        primary: drawn.primary,
        offers: [first, second],
    };
    assert_eq!(ended.level_up(RED), Some(view));
    assert_eq!((ended.level_up(BLUE), ended.decision(RED)), (None, None));
    assert!(ended.has_decision(RED));
    let seen = observe(&ended, &rules, Viewer::Player(RED))
        .decision
        .unwrap();
    assert_eq!((seen.offer(), seen.level_up()), (None, Some(&view)));
    assert_eq!(observe(&ended, &rules, Viewer::Player(BLUE)).decision, None);
    // Its answers only, in red's turn; blue has its turn's `EndTurn`, nothing more.
    let (red, blue) = (Actor::Player(RED), Actor::Player(BLUE));
    assert_eq!(legal_commands(&ended, &rules, red), [FIRST, SECOND]);
    assert_eq!(legal_moves(&ended, &rules, red, RED_HERO), None);
    assert_eq!(legal_commands(&ended, &rules, blue), []);
    // The system actor's default: the original's choice with the setup's inputs (ADV-STUB-8).
    let shares = ended.adventure().map().terrain_shares();
    let choice = original_choice(&details, rules.core(), &drawn, ArmyWorth::SETUP, shares);
    assert_eq!(
        default_answer(&ended, &rules, RED),
        Some(Answer::of(choice))
    );
    assert_eq!(default_answer(&ended, &rules, BLUE), None);
    // Wrong answers, other commands and other players change nothing.
    let mut state = ended.clone();
    for answer in [Answer::Accept, Answer::Decline] {
        let wrong = play(&mut state, &rules, RED, GameCommand::Answer(answer));
        assert_eq!(wrong, Err(GameCommandError::WrongAnswer));
    }
    let open = Err(GameCommandError::DecisionOpen);
    assert_eq!(play(&mut state, &rules, RED, GameCommand::EndTurn), open);
    assert_eq!(state.waiting_for(), Some(RED));
    let held = Err(GameCommandError::WaitingForLevelUp(RED));
    assert_eq!(play(&mut state, &rules, BLUE, FIRST), held);
    // Each answer gives the level with its offer, told to the owner alone; the decision closes.
    let mut hashes = vec![ended.state_hash()];
    for (command, choice) in [(FIRST, Choice::First), (SECOND, Choice::Second)] {
        let mut answered = ended.clone();
        let events = play(&mut answered, &rules, RED, command).unwrap();
        let gained = GameEvent::LevelGained {
            player: RED,
            hero: RED_HERO,
            level: 2,
            primary: drawn.primary,
            skill: drawn.learned(choice),
        };
        assert_eq!(events, [gained]);
        assert_eq!(project(&gained, Viewer::Player(BLUE)), None);
        let mut expected = details.clone();
        apply_change(&mut expected, rules.core(), drawn.change(choice));
        assert_eq!(answered.hero(RED_HERO), Some(&expected));
        assert_eq!(expected.level, 2);
        assert!(!answered.has_decision(RED));
        assert_eq!(
            legal_commands(&answered, &rules, red),
            [GameCommand::EndTurn]
        );
        assert_eq!(answered.stream(RED), stream);
        hashes.push(answered.state_hash());
    }
    // The open level-up and each answer hash apart.
    hashes.sort_unstable();
    hashes.dedup();
    assert_eq!(hashes.len(), 3);
}

// covers: GAM-DEC-002, GAM-HER-002
#[test]
fn levels_come_one_at_a_time_each_drawn_after_the_previous_answer() {
    let rules = rules();
    let game = game(&rules, 0, &[]);
    // 500 + 120 × 13 = 2 060 experience: levels 2 and 3.
    let (ended, events) = end(&game, &rules, &won_by(BattleSide::Attacker, 120));
    let mut details = experienced(&rules, &game, RED_HERO, 120);
    let mut stream = game.stream(RED);
    let second = draw_level_up(&details, rules.core(), &[], &mut stream).unwrap();
    assert_eq!(levels(&events), [opened(RED, RED_HERO, &second)]);
    // Only the first level is drawn so far.
    assert_eq!(ended.stream(RED), stream);
    let mut state = ended;
    let events = play(&mut state, &rules, RED, SECOND).unwrap();
    apply_change(&mut details, rules.core(), second.change(Choice::Second));
    let third = draw_level_up(&details, rules.core(), &[], &mut stream).unwrap();
    assert_eq!(third.level, 3);
    let gained = GameEvent::LevelGained {
        player: RED,
        hero: RED_HERO,
        level: 2,
        primary: second.primary,
        skill: second.learned(Choice::Second),
    };
    assert_eq!(events, [gained, opened(RED, RED_HERO, &third)]);
    assert_eq!(state.stream(RED), stream);
    play(&mut state, &rules, RED, FIRST).unwrap();
    apply_change(&mut details, rules.core(), third.change(Choice::First));
    assert_eq!(state.hero(RED_HERO), Some(&details));
    assert!(!state.has_decision(RED));
}

// covers: GAM-DEC-002, HER-LVL-003
#[test]
fn a_level_with_one_offer_or_none_is_given_at_once() {
    let rules = rules();
    for (banned, one) in [
        (&["t:wits", "t:lore", "t:dark"][..], true),
        (&["t:wits", "t:lore", "t:dark", "t:sense"][..], false),
    ] {
        let game = game(&rules, 950, banned);
        let (ended, events) = end(&game, &rules, &won_by(BattleSide::Attacker, 2));
        let mut details = experienced(&rules, &game, RED_HERO, 2);
        let mut stream = game.stream(RED);
        let drawn =
            draw_level_up(&details, rules.core(), game.banned_skills(), &mut stream).unwrap();
        assert_eq!(matches!(drawn.offers, Offers::One(_)), one, "{drawn:?}");
        assert_eq!(matches!(drawn.offers, Offers::None), !one, "{drawn:?}");
        let gained = GameEvent::LevelGained {
            player: RED,
            hero: RED_HERO,
            level: 2,
            primary: drawn.primary,
            skill: drawn.learned(Choice::First),
        };
        assert_eq!(levels(&events), [gained]);
        assert_eq!(drawn.learned(Choice::First).is_some(), one);
        apply_change(&mut details, rules.core(), drawn.change(Choice::First));
        assert_eq!(ended.hero(RED_HERO), Some(&details));
        assert!(!ended.has_decision(RED));
        assert_eq!(ended.stream(RED), stream);
    }
}

// covers: GAM-HER-002, GAM-BAT-008
#[test]
fn in_faithful_a_level_in_play_follows_the_heros_seed() {
    let rules = rules_in("t:seeded");
    let game = game(&rules, 950, &[]);
    let (ended, events) = end(&game, &rules, &won_by(BattleSide::Attacker, 2));
    let details = experienced(&rules, &game, RED_HERO, 2);
    let mut draws = OriginalDraws::new(game.level_seed(RED_HERO), 2);
    let drawn = draw_level_up(&details, rules.core(), &[], &mut draws).unwrap();
    assert_eq!(levels(&events), [opened(RED, RED_HERO, &drawn)]);
    // The owner's stream is not drawn from.
    assert_eq!(ended.stream(RED), game.stream(RED));
}

// covers: GAM-BAT-008, GAM-DEC-002
#[test]
fn a_defender_that_wins_holds_the_game_until_it_answers_outside_its_turn() {
    let rules = rules();
    // Red's second hero (id 2) stays on the map while its first loses to blue's.
    let game = with_second_red(&rules, 950);
    let (ended, events) = end(&game, &rules, &won_by(BattleSide::Defender, 2));
    let details = experienced(&rules, &game, BLUE_HERO, 2);
    let mut stream = game.stream(BLUE);
    let drawn = draw_level_up(&details, rules.core(), &[], &mut stream).unwrap();
    assert_eq!(levels(&events), [opened(BLUE, BLUE_HERO, &drawn)]);
    assert_eq!(
        (ended.stream(RED), ended.stream(BLUE)),
        (game.stream(RED), stream)
    );
    // Red's turn runs, but the game waits for blue's answer: red has no command, no move.
    let (red, blue) = (Actor::Player(RED), Actor::Player(BLUE));
    assert!(ended.is_acting(RED) && !ended.is_acting(BLUE));
    assert_eq!(ended.waiting_for(), Some(BLUE));
    assert_eq!(legal_commands(&ended, &rules, red), []);
    assert_eq!(legal_commands(&ended, &rules, Actor::System), []);
    assert_eq!(legal_moves(&ended, &rules, red, HeroId::new(2)), None);
    assert_eq!(legal_commands(&ended, &rules, blue), [FIRST, SECOND]);
    let mut state = ended;
    let held = Err(GameCommandError::WaitingForLevelUp(BLUE));
    let step = decisions::move_hero(2, &[Pos::new(3, 1, 0)]);
    for command in [GameCommand::EndTurn, step.clone(), FIRST] {
        assert_eq!(play(&mut state, &rules, RED, command), held);
    }
    let wait = GameCommand::Battle(h3_battle::Command::Wait);
    let mut events = Vec::new();
    let system = apply(&mut state, &rules, Actor::System, wait, &mut events);
    let refused = Err(GameCommandError::WaitingForLevelUp(BLUE));
    assert_eq!((system, events.len()), (refused, 0));
    // Blue answers outside its turn; then red's turn goes on.
    let events = play(&mut state, &rules, BLUE, FIRST).unwrap();
    assert_eq!(levels(&events).len(), 1);
    assert_eq!(state.hero(BLUE_HERO).unwrap().level, 2);
    assert_eq!(state.waiting_for(), None);
    assert_eq!(legal_commands(&state, &rules, blue), []);
    assert_eq!(legal_commands(&state, &rules, red), [GameCommand::EndTurn]);
    assert!(legal_moves(&state, &rules, red, HeroId::new(2)).is_some());
    play(&mut state, &rules, RED, step).unwrap();
}

// covers: GAM-BAT-008, HER-EXP-003
#[test]
fn the_scenarios_level_limit_bounds_the_levels_in_play() {
    let rules = rules();
    let mut limited = setup();
    limited.heroes[0].details.experience = Some(0);
    limited.level_limit = 2;
    let game = GameState::new(&rules, &limited, 5).unwrap();
    let (ended, events) = end(&game, &rules, &won_by(BattleSide::Attacker, 120));
    // The experience stops at level 2's threshold: one level, not two.
    assert_eq!(ended.hero(RED_HERO).unwrap().experience, 1000);
    assert_eq!(levels(&events).len(), 1);
    let mut state = ended;
    play(&mut state, &rules, RED, FIRST).unwrap();
    assert_eq!(state.hero(RED_HERO).unwrap().level, 2);
    assert!(!state.has_decision(RED));
}

// covers: GAM-DEC-002
#[test]
fn a_fork_shares_an_open_level_up_and_answers_apart() {
    let rules = rules();
    let game = game(&rules, 950, &[]);
    let (ended, _) = end(&game, &rules, &won_by(BattleSide::Attacker, 2));
    let mut fork = ended.clone();
    play(&mut fork, &rules, RED, FIRST).unwrap();
    assert!(ended.level_up(RED).is_some());
    assert!(fork.level_up(RED).is_none());
}

/// [`game`] with both heroes at `experience` and red's second hero (id 2) at (3, 0).
fn with_second_red(rules: &GameRuleset, experience: u32) -> GameState {
    let mut setup = setup();
    for hero in &mut setup.heroes {
        hero.details.experience = Some(experience);
    }
    let mut second = setup.heroes[0].clone();
    second.pos = Pos::new(3, 0, 0);
    setup.heroes.push(second);
    GameState::new(rules, &setup, 5).unwrap()
}

// covers: GAM-DEC-002, HER-LVL-006
#[test]
fn the_system_actors_default_takes_the_offer_the_original_chooses() {
    let rules = rules();
    let mut answers = Vec::new();
    for seed in 0..40 {
        let mut setup = setup();
        setup.heroes[0].details.experience = Some(950);
        let game = GameState::new(&rules, &setup, seed).unwrap();
        let (ended, _) = end(&game, &rules, &won_by(BattleSide::Attacker, 2));
        let (_, level_up) = ended.open_level_up(RED).unwrap();
        let details = ended.hero(RED_HERO).unwrap();
        let shares = ended.adventure().map().terrain_shares();
        let choice = original_choice(details, rules.core(), &level_up, ArmyWorth::SETUP, shares);
        let answer = default_answer(&ended, &rules, RED).unwrap();
        assert_eq!(answer, Answer::of(choice));
        answers.push(answer);
    }
    // Over the seeds' draws the original takes either offer.
    assert!(answers.contains(&Answer::First) && answers.contains(&Answer::Second));
}
