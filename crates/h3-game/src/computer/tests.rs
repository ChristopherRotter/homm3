//! Tests of the computer player's frame (E31 T2a): [`choose`] on the town tests' game (red's T0 on
//! (1, 1), its entrance; blue's T1 on (3, 0); red's hero on (0, 0), blue's on (2, 2); the
//! difficulty Normal) and on the crate's plain game (no town): the turn's passes and its end,
//! the order of heroes, the garrison steps (an entry, a refused one, a swap, a garrison hero sent
//! out), the open decisions and the battles, the turn's flag, a new day's turn; the memory's
//! encoding.

use alloc::vec;
use alloc::vec::Vec;

use h3_adventure::{AdventureCommand, Pos};
use h3_battle::BattleSide;
use h3_content::CreatureTypeId;
use h3_contracts::{
    Army, ArmyStack, BattleOutcome, BattleResult, HeroId, PlayerId, StackOrigin, StackRecord,
    TownId,
};
use h3_kernel::hash::{StateHasher, hash_of};
use h3_kernel::{Rng, StreamId};
use h3_towns::{TownCommand, TownHeroChange};
use proptest::prelude::*;

use super::memory::{Pass, Turn};
use super::start::{self, Record};
use super::turn::{Pick, counted, next_hero};
use super::*;
use crate::town::tests::{towns_rules, towns_setup};
use crate::{Actor, Answer, GameCommandError, GameEvent, GameSetup, HeroArmy, apply};

const RED: PlayerId = PlayerId::new(0);
const BLUE: PlayerId = PlayerId::new(1);
const T0: TownId = TownId::new(0);
const ENTRANCE: Pos = Pos::new(1, 1, 0);

fn heroes(change: TownHeroChange) -> GameCommand {
    GameCommand::Town(TownCommand::Heroes { town: T0, change })
}

fn stream() -> Rng {
    Rng::for_stream(9, StreamId::AGENT)
}

/// The town tests' game with `edit` made to its setup.
fn town_game(edit: impl FnOnce(&mut GameSetup)) -> (GameRuleset, GameState) {
    let rules = towns_rules();
    let mut setup = towns_setup(&rules);
    edit(&mut setup);
    let game = GameState::new(&rules, &setup, 3).unwrap_or_else(|e| panic!("{e:?}"));
    (rules, game)
}

/// A second hero of red's on `pos`, of the first one's type and army, with `imps` imps.
fn red_hero(setup: &mut GameSetup, pos: Pos, imps: u32) {
    let mut hero = setup.heroes[0].clone();
    hero.pos = pos;
    hero.army = imps_army(imps);
    setup.heroes.push(hero);
}

/// `count` imps in slot 0.
fn imps_army(count: u32) -> HeroArmy {
    HeroArmy::Custom(vec![ArmyStack {
        slot: 0,
        creature: CreatureTypeId::from_raw(0),
        count,
    }])
}

/// `command` of `player`, which must apply.
fn run(state: &mut GameState, rules: &GameRuleset, player: PlayerId, command: GameCommand) {
    let mut events: Vec<GameEvent> = Vec::new();
    apply(
        state,
        rules,
        Actor::Player(player),
        command.clone(),
        &mut events,
    )
    .unwrap_or_else(|e| panic!("{command:?}: {e:?}"));
}

/// The commands of `player`'s turn by [`choose`], each applied, to its `EndTurn`; each call made
/// twice on copies of the memory and the stream, which must agree.
fn turn(state: &mut GameState, rules: &GameRuleset, memory: &mut Memory) -> Vec<GameCommand> {
    let player = state.turn_group()[0];
    let mut rng = stream();
    let mut commands = Vec::new();
    while let Some(command) = {
        let (mut again, mut other) = (memory.clone(), rng.clone());
        let command = choose(state, rules, player, memory, &mut rng);
        assert_eq!(
            choose(state, rules, player, &mut again, &mut other),
            command
        );
        assert_eq!((&again, &other), (&*memory, &rng));
        command
    } {
        run(state, rules, player, command.clone());
        commands.push(command.clone());
        assert!(commands.len() < 32, "{commands:?}");
        if command == GameCommand::EndTurn {
            break;
        }
    }
    commands
}

/// The memory's turn, which [`Memory::turn_of`] gives for its own day.
fn frame(memory: &mut Memory, day: u32) -> Turn {
    memory.turn_of(day, || panic!("no turn")).clone()
}

// covers: CPU-TRN-001, CPU-TRN-002, CPU-TRN-004
#[test]
fn a_turn_without_a_step_ends_at_once_and_a_new_day_starts_a_new_turn() {
    let (rules, mut game) = town_game(|_| {});
    let (mut red, mut blue) = (Memory::new(), Memory::new());
    let record = start::record(&game, &rules, RED);
    // Only the acting player has a command; none for a player not in the game.
    assert_eq!(choose(&game, &rules, BLUE, &mut blue, &mut stream()), None);
    let nobody = PlayerId::new(7);
    assert_eq!(
        choose(&game, &rules, nobody, &mut blue, &mut stream()),
        None
    );
    assert_eq!(blue, Memory::new());
    // Red's hero is done at once (AI-STUB-2) and stands on no town: the turn ends.
    assert_eq!(turn(&mut game, &rules, &mut red), [GameCommand::EndTurn]);
    let played = frame(&mut red, 1);
    assert_eq!(
        played,
        Turn {
            day: 1,
            pass: Pass::Over,
            flag: true,
            next: None,
            asleep: vec![HeroId::new(0)],
            record,
        }
    );
    assert_eq!(choose(&game, &rules, RED, &mut red, &mut stream()), None);
    assert_eq!(turn(&mut game, &rules, &mut blue), [GameCommand::EndTurn]);
    // Day 2: red's turn starts anew, its heroes awake again.
    assert_eq!(game.day(), 2);
    let mut fresh = red.clone();
    let mut rng = stream();
    assert_eq!(
        choose(&game, &rules, RED, &mut fresh, &mut rng),
        Some(GameCommand::EndTurn)
    );
    // Its records anew too: the treasury grew by the day's income.
    let record = start::record(&game, &rules, RED);
    assert_ne!(record.means, played.record.means);
    assert_eq!(
        frame(&mut fresh, 2),
        Turn {
            day: 2,
            record,
            ..played
        }
    );
    assert_eq!(rng, stream(), "the frame draws nothing");
}

// covers: CPU-TRN-002
#[test]
fn the_turns_flag_is_a_town_owned_above_easy() {
    let flag = |difficulty: u8, towns: bool| {
        let (rules, game) = town_game(|setup| {
            setup.difficulty = difficulty;
            if !towns {
                setup.towns.clear();
                setup.map.objects.clear();
                setup.players[0].start = Default::default();
                setup.players[1].start = Default::default();
            }
        });
        let mut memory = Memory::new();
        choose(&game, &rules, RED, &mut memory, &mut stream()).unwrap();
        frame(&mut memory, 1).flag
    };
    assert!(flag(1, true));
    assert!(!flag(0, true));
    assert!(!flag(1, false));
}

// covers: CPU-TRN-004
#[test]
fn a_hero_done_on_its_town_goes_into_the_empty_garrison() {
    let (rules, mut game) = town_game(|setup| setup.heroes[0].pos = ENTRANCE);
    let mut memory = Memory::new();
    let commands = turn(&mut game, &rules, &mut memory);
    assert_eq!(
        commands,
        [heroes(TownHeroChange::Enter), GameCommand::EndTurn]
    );
    assert_eq!(
        game.adventure().town_hero(T0).map(|hero| hero.id),
        Some(HeroId::new(0))
    );
    // Blue's hero stands on no town of blue's: nothing.
    assert_eq!(
        turn(&mut game, &rules, &mut Memory::new()),
        [GameCommand::EndTurn]
    );
}

// covers: CPU-TRN-004
#[test]
fn an_entry_the_engine_refuses_is_no_step() {
    // The visitor's full stack of imps does not take the garrison's imp (TWN-ROW-002): the
    // original's town_visitor_to_garrison does nothing then.
    let rules = towns_rules();
    let most = rules.adventure().params().max_stack_count();
    let (rules, mut game) = town_game(|setup| {
        setup.heroes[0].pos = ENTRANCE;
        setup.heroes[0].army = imps_army(most);
    });
    let (adventure, _) = game.adventure_with_heroes();
    adventure.set_town_garrison(
        T0,
        vec![ArmyStack {
            slot: 1,
            creature: CreatureTypeId::from_raw(0),
            count: 1,
        }],
    );
    let mut refused = game.clone();
    let mut events = Vec::new();
    let enter = heroes(TownHeroChange::Enter);
    assert!(apply(&mut refused, &rules, Actor::Player(RED), enter, &mut events).is_err());
    assert_eq!(
        turn(&mut game, &rules, &mut Memory::new()),
        [GameCommand::EndTurn]
    );
    assert_eq!(game.adventure().town_hero(T0), None);
}

/// Red's hero 2 (20 imps) in T0's garrison, red's hero 0 (9 imps) moved onto its entrance.
fn swap_game() -> (GameRuleset, GameState) {
    let (rules, mut game) = town_game(|setup| red_hero(setup, ENTRANCE, 20));
    run(&mut game, &rules, RED, heroes(TownHeroChange::Enter));
    let step = GameCommand::Adventure(AdventureCommand::MoveHero {
        hero: HeroId::new(0),
        path: vec![ENTRANCE],
    });
    run(&mut game, &rules, RED, step);
    (rules, game)
}

// covers: CPU-TRN-003, CPU-TRN-004
#[test]
fn a_visitor_done_swaps_with_an_awake_garrison_hero_which_then_stays_out() {
    // The garrison hero has its points and has not played: the two change places. Out and done,
    // it keeps the visitor's place: the hero now in the garrison is done, and its army is worth
    // no less (the test core's creatures are worth 0; `h3-sim`'s tests swap back on the
    // original packs' values).
    let (rules, mut game) = swap_game();
    let mut memory = Memory::new();
    assert_eq!(
        turn(&mut game, &rules, &mut memory),
        [heroes(TownHeroChange::Swap), GameCommand::EndTurn]
    );
    assert_eq!(
        game.adventure().town_hero(T0).map(|hero| hero.id),
        Some(HeroId::new(0))
    );
    assert_eq!(game.adventure().town_visitor(T0), Some(HeroId::new(2)));
    assert_eq!(
        frame(&mut memory, 1).asleep,
        [HeroId::new(0), HeroId::new(2)]
    );
}

// covers: CPU-TRN-003, CPU-TRN-004
#[test]
fn a_garrison_hero_is_sent_out_once_every_hero_is_done_and_plays_next() {
    let (rules, mut game) = town_game(|setup| red_hero(setup, ENTRANCE, 4));
    run(&mut game, &rules, RED, heroes(TownHeroChange::Enter));
    let mut memory = Memory::new();
    let mut rng = stream();
    // Red's hero 0 plays first and is done; then hero 2 is sent out, to play next.
    assert_eq!(
        choose(&game, &rules, RED, &mut memory, &mut rng),
        Some(heroes(TownHeroChange::Leave))
    );
    assert_eq!(frame(&mut memory, 1).next, Some(HeroId::new(2)));
    assert_eq!(frame(&mut memory, 1).asleep, [HeroId::new(0)]);
    run(&mut game, &rules, RED, heroes(TownHeroChange::Leave));
    // Done at once, on its town's entrance with the garrison empty: it goes back in.
    assert_eq!(
        turn(&mut game, &rules, &mut memory),
        [heroes(TownHeroChange::Enter), GameCommand::EndTurn]
    );
    assert_eq!(frame(&mut memory, 1).next, None);
}

// covers: CPU-TRN-003
#[test]
fn no_garrison_hero_is_sent_out_with_eight_heroes_on_the_map() {
    let (rules, mut game) = town_game(|setup| {
        red_hero(setup, ENTRANCE, 4);
        for (x, y) in [(0, 1), (0, 2), (0, 3), (1, 0), (1, 2), (1, 3), (2, 0)] {
            red_hero(setup, Pos::new(x, y, 0), 1);
        }
    });
    run(&mut game, &rules, RED, heroes(TownHeroChange::Enter));
    assert_eq!(game.adventure().hero_list(RED).len(), 8);
    assert_eq!(
        turn(&mut game, &rules, &mut Memory::new()),
        [GameCommand::EndTurn]
    );
}

// covers: CPU-TRN-003
#[test]
fn the_weakest_hero_by_its_primary_skills_plays_first_a_tie_to_the_earlier_in_the_list() {
    let (rules, game) = town_game(|setup| {
        for (x, primary) in [(0, [0, 0, 0, 0]), (1, [3, 3, 3, 3]), (2, [0, 0, 0, 0])] {
            red_hero(setup, Pos::new(x, 3, 0), 1);
            setup.heroes.last_mut().unwrap().details.primary = Some(primary);
        }
        setup.heroes[0].details.primary = Some([5, 5, 5, 5]);
    });
    let mut memory = Memory::new();
    let turn = memory.turn_of(1, || Turn {
        day: 1,
        pass: Pass::First,
        flag: false,
        next: None,
        asleep: Vec::new(),
        record: Record::default(),
    });
    let mut order = Vec::new();
    while let Pick::Hero(hero, alone) = next_hero(&game, &rules, RED, turn) {
        order.push((hero.id.raw(), alone));
        turn.sleep(hero.id);
    }
    assert!(matches!(next_hero(&game, &rules, RED, turn), Pick::None));
    assert_eq!(order, [(2, false), (4, false), (3, false), (0, true)]);
}

#[test]
fn a_primary_skill_counts_as_its_signed_byte_from_its_floor_to_99() {
    for (value, floor, counts) in [
        (0, 0, 0),
        (0, 1, 1),
        (1, 0, 1),
        (57, 1, 57),
        (99, 0, 99),
        (100, 1, 99),
        (127, 0, 99),
        (128, 1, 1),
        (255, 0, 0),
    ] {
        assert_eq!(counted(value, floor), counts, "{value} from {floor}");
    }
}

/// The plain game's red hero (0) attacking blue's on (2, 2): a battle open in red's turn.
fn battle() -> (GameRuleset, GameState) {
    let rules = crate::tests::rules();
    let mut game = GameState::new(&rules, &crate::tests::setup(), 5).unwrap();
    let attack = GameCommand::Adventure(AdventureCommand::MoveHero {
        hero: HeroId::new(0),
        path: vec![Pos::new(1, 1, 0), Pos::new(2, 2, 0)],
    });
    run(&mut game, &rules, RED, attack);
    (rules, game)
}

// covers: CPU-TRN-001
#[test]
fn a_battle_is_the_battle_seats_and_a_level_up_is_answered_while_it_holds_the_game() {
    let (rules, game) = battle();
    assert!(game.battle(RED).is_some());
    for player in [RED, BLUE] {
        let mut memory = Memory::new();
        assert_eq!(
            choose(&game, &rules, player, &mut memory, &mut stream()),
            None
        );
        assert_eq!(memory, Memory::new());
    }
    // Blue's hero wins as the defender and levels up: the game waits for its answer, which is
    // the default (AI-STUB-6); red has no command meanwhile.
    let mut setup = crate::tests::setup();
    for hero in &mut setup.heroes {
        hero.details.experience = Some(950);
    }
    let mut second = setup.heroes[0].clone();
    second.pos = Pos::new(3, 0, 0);
    setup.heroes.push(second);
    let mut held = GameState::new(&rules, &setup, 5).unwrap();
    let mut survivors = [Army { stacks: vec![] }, Army { stacks: vec![] }];
    survivors[1].stacks = vec![ArmyStack {
        slot: 1,
        creature: CreatureTypeId::from_raw(0),
        count: 1,
    }];
    let mut stacks = [vec![], vec![]];
    stacks[0] = vec![StackRecord {
        origin: StackOrigin::Army(0),
        creature: CreatureTypeId::from_raw(1),
        start: 2,
        end: 0,
    }];
    let outcome = BattleOutcome {
        result: BattleResult::Victory(BattleSide::Defender),
        survivors,
        war_machines: [vec![], vec![]],
        stacks,
        surrender_cost: None,
        heroes: [None, None],
        rounds: 1,
    };
    let sides = (
        [Some(RED), Some(BLUE)],
        [Some(HeroId::new(0)), Some(HeroId::new(1))],
        Pos::new(0, 0, 0),
    );
    held.end_battle(&rules, sides, None, &outcome, &mut Vec::new());
    assert_eq!(held.waiting_for(), Some(BLUE));
    let mut memory = Memory::new();
    assert_eq!(choose(&held, &rules, RED, &mut memory, &mut stream()), None);
    let answer = choose(&held, &rules, BLUE, &mut memory, &mut stream()).unwrap();
    assert_eq!(
        answer,
        GameCommand::Answer(default_answer(&held, &rules, BLUE).unwrap())
    );
    assert!(matches!(answer, GameCommand::Answer(Answer::LevelUp(_))));
    run(&mut held, &rules, BLUE, answer);
    assert_eq!(held.waiting_for(), None);
    // Then red's turn goes on: its hero 2 is done, and the turn ends.
    assert_eq!(turn(&mut held, &rules, &mut memory), [GameCommand::EndTurn]);
    let mut refused = held.clone();
    let mut events = Vec::new();
    assert_eq!(
        apply(
            &mut refused,
            &rules,
            Actor::Player(RED),
            GameCommand::EndTurn,
            &mut events
        ),
        Err(GameCommandError::NotYourTurn(RED))
    );
}

/// A memory holding `turn`.
fn holding(turn: Turn) -> Memory {
    let mut memory = Memory::new();
    let day = turn.day;
    memory.turn_of(day, || turn);
    memory
}

fn record_strategy() -> impl Strategy<Value = Record> {
    let lists = (
        proptest::collection::btree_map(any::<u16>(), any::<u32>(), 0..6),
        proptest::collection::btree_map(
            any::<u16>(),
            any::<i32>().prop_filter("not 0", |t| *t != 0),
            0..6,
        ),
        proptest::collection::btree_map(any::<u16>(), 1_u8.., 0..6),
    );
    let arrays = (0_usize..8).prop_flat_map(|n| {
        proptest::collection::vec(any::<i32>(), 3 * n)
            .prop_map(move |v| (v[..n].to_vec(), v[n..2 * n].to_vec(), v[2 * n..].to_vec()))
    });
    (any::<i32>(), arrays, 1_u8..=10, any::<i32>(), lists).prop_map(
        |(eyes, (reserve, means, demand), markets, mean, (experience, threats, towns))| Record {
            eyes,
            reserve,
            means,
            demand,
            markets,
            mean,
            experience: experience
                .into_iter()
                .map(|(h, b)| (HeroId::new(h), b))
                .collect(),
            threats: threats
                .into_iter()
                .map(|(h, t)| (HeroId::new(h), t))
                .collect(),
            towns: towns
                .into_iter()
                .map(|(t, c)| (TownId::new(t), c))
                .collect(),
        },
    )
}

fn turn_strategy() -> impl Strategy<Value = Turn> {
    (
        any::<u32>(),
        0_u8..3,
        any::<bool>(),
        proptest::option::of(any::<u16>()),
        proptest::collection::btree_set(any::<u16>(), 0..12),
        record_strategy(),
    )
        .prop_map(|(day, pass, flag, next, asleep, record)| Turn {
            day,
            pass: [Pass::First, Pass::Second, Pass::Over][usize::from(pass)],
            flag,
            next: next.map(HeroId::new),
            asleep: asleep.into_iter().map(HeroId::new).collect(),
            record,
        })
}

/// A start's record with every field set (two resources, one entry in each list).
fn sample_record() -> Record {
    Record {
        eyes: -2,
        reserve: vec![1, 2],
        means: vec![3, -1],
        demand: vec![0x0100, 4],
        markets: 3,
        mean: 7,
        experience: vec![(HeroId::new(1), 0x3F80_0000)],
        threats: vec![(HeroId::new(9), 2_500_000)],
        towns: vec![(TownId::new(0), 2)],
    }
}

// covers: CPU-TRN-005
#[test]
fn the_memory_has_one_encoding_and_refuses_any_other_bytes() {
    let empty = Memory::new();
    assert_eq!(empty, Memory::default());
    assert_eq!(empty.encode(), [2, 0, 0, 0, 0]);
    let turn = Turn {
        day: 0x0102_0304,
        pass: Pass::Second,
        flag: true,
        next: Some(HeroId::new(0x0506)),
        asleep: vec![HeroId::new(1), HeroId::new(0x0203)],
        record: sample_record(),
    };
    let full = holding(turn.clone());
    let bytes = full.encode();
    let frame: [u8; 22] = [
        2, 0, 0, 0, 1, 4, 3, 2, 1, 1, 1, 1, 6, 5, 2, 0, 0, 0, 1, 0, 3, 2,
    ];
    let record: [u8; 64] = [
        0xFE, 0xFF, 0xFF, 0xFF, 2, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 0xFF, 0xFF, 0xFF,
        0xFF, 0, 1, 0, 0, 4, 0, 0, 0, 3, 7, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0x80, 0x3F, 1, 0, 0,
        0, 9, 0, 0xA0, 0x25, 0x26, 0, 1, 0, 0, 0, 0, 0, 2,
    ];
    assert_eq!(bytes, [&frame[..], &record[..]].concat());
    assert_eq!(Memory::decode(&bytes), Ok(full.clone()));
    assert_eq!(Memory::decode(&empty.encode()), Ok(empty.clone()));
    // The hash is FNV-1a over the encoding.
    let mut h = StateHasher::new();
    h.write_bytes(&bytes);
    assert_eq!((full.hash(), hash_of(&full)), (h.finish(), h.finish()));
    assert_ne!(full.hash(), empty.hash());
    // Every shorter prefix, a longer one and each value out of its range are refused: the tags,
    // the pass, the flag, the done heroes' order, the rate's row (0 and 11), a threat of 0 and a
    // town's count of 0.
    let malformed = Err(MemoryError::Malformed);
    for end in 0..bytes.len() {
        assert_eq!(Memory::decode(&bytes[..end]), malformed, "{end}");
    }
    let mut longer = bytes.clone();
    longer.push(0);
    assert_eq!(Memory::decode(&longer), malformed);
    let edits: [(usize, &[u8]); 9] = [
        (4, &[2]),
        (9, &[3]),
        (10, &[2]),
        (11, &[2]),
        (19, &[3]),
        (54, &[0]),
        (54, &[11]),
        (75, &[0, 0, 0, 0]),
        (85, &[0]),
    ];
    for (at, values) in edits {
        let mut edited = bytes.clone();
        edited[at..at + values.len()].copy_from_slice(values);
        assert_eq!(Memory::decode(&edited), malformed, "{at}: {values:?}");
    }
    let equal = holding(Turn {
        asleep: vec![HeroId::new(3), HeroId::new(3)],
        ..turn.clone()
    });
    assert_eq!(Memory::decode(&equal.encode()), malformed);
    let mut twice = sample_record();
    twice.experience.push((HeroId::new(1), 0));
    let mut threats = sample_record();
    threats.threats.insert(0, (HeroId::new(10), 1));
    let mut towns = sample_record();
    towns.towns.push((TownId::new(0), 1));
    for record in [twice, threats, towns] {
        let unordered = holding(Turn {
            record,
            ..turn.clone()
        });
        assert_eq!(Memory::decode(&unordered.encode()), malformed);
    }
    let none_sent = holding(Turn { next: None, ..turn });
    assert_eq!(Memory::decode(&none_sent.encode()), Ok(none_sent));
    let mut other = bytes;
    other[0] = 1;
    assert_eq!(Memory::decode(&other), Err(MemoryError::Version(1)));
    assert_eq!(MEMORY_ENCODING_VERSION, 2);
}

proptest! {
    // covers: CPU-TRN-005
    #[test]
    fn a_memory_decodes_from_its_encoding_and_bytes_decode_only_to_theirs(
        turn in proptest::option::of(turn_strategy()),
        bytes in proptest::collection::vec(any::<u8>(), 0..120),
    ) {
        let memory = turn.map_or_else(Memory::new, holding);
        prop_assert_eq!(Memory::decode(&memory.encode()), Ok(memory.clone()));
        let mut prefixed = MEMORY_ENCODING_VERSION.to_le_bytes().to_vec();
        prefixed.extend_from_slice(&bytes);
        if let Ok(decoded) = Memory::decode(&prefixed) {
            prop_assert_eq!(decoded.encode(), prefixed);
        }
    }
}
