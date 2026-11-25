//! Tests of the computer player hosted (E31 T2a): whole games of the committed generated maps
//! (`fixtures/maps/`) with every player the original's computer, at every difficulty: every
//! command legal, each turn within its bound, each command the same on copies, the same game for
//! the same seed; the garrison steps on the original packs' values.

use std::sync::{Arc, LazyLock};

use h3_adventure::{AdventureCommand, Pos};
use h3_contracts::{HeroId, PlayerId, TownId};
use h3_game::computer::{Memory, choose};
use h3_game::{Actor, GameCommand, GameRuleset, GameSetup, GameState, NoopSink, apply};
use h3_kernel::{Rng, StreamId};
use h3_towns::{TownCommand, TownHeroChange};
use proptest::prelude::*;

use super::*;

const MAPS: [&[u8]; 5] = [
    include_bytes!("../../../../fixtures/maps/e08-duel-36.h3m"),
    include_bytes!("../../../../fixtures/maps/e09-towns-36.h3m"),
    include_bytes!("../../../../fixtures/maps/e09-builds-36.h3m"),
    include_bytes!("../../../../fixtures/maps/e09-taverns-36.h3m"),
    include_bytes!("../../../../fixtures/maps/e09-guild-36.h3m"),
];

fn rules(profile: &str) -> GameRuleset {
    let (core, part) =
        h3_pack_adventure::compile(&h3_pack_adventure::ORIGINAL_PACKS, profile).unwrap();
    let battle = h3_pack_battle::compile(
        &[
            h3_pack_battle::BASE_PACK,
            h3_pack_battle::AB_PACK,
            h3_pack_battle::SOD_PACK,
        ],
        profile,
    )
    .unwrap();
    let towns = h3_pack_towns::compile(&h3_pack_towns::ORIGINAL_PACKS, profile).unwrap();
    GameRuleset::new(
        Arc::new(core),
        Arc::new(part),
        Arc::new(battle),
        Arc::new(towns),
    )
    .unwrap()
}

static FAITHFUL: LazyLock<GameRuleset> = LazyLock::new(|| rules("base:faithful"));

/// The setup of map `map` at `difficulty`.
fn setup(map: usize, difficulty: u8) -> GameSetup {
    let raw = h3_map::read(MAPS[map], &h3_map::Limits::default()).unwrap();
    let mut setup = h3_map::import(&raw, &FAITHFUL).unwrap();
    setup.difficulty = difficulty;
    setup
}

fn game(setup: &GameSetup, seed: u64) -> GameState {
    GameState::new(&FAITHFUL, setup, seed).unwrap()
}

// covers: CPU-TRN-001, CPU-TRN-002, CPU-TRN-004
#[test]
fn the_town_maps_computers_play_a_week_and_go_into_their_garrisons() {
    let setup = setup(1, 1);
    let played = play(&FAITHFUL, game(&setup, 1), 1, 7, true).unwrap();
    assert_eq!(played.day, 8);
    // Blue's Yog stands on the Tower's entrance: done at once, it goes into the garrison.
    assert!(played.garrison_moves[0] > 0, "{played:?}");
    assert_eq!(played.turns, 14);
    // The same seed plays the same game; another seed's draws differ.
    assert_eq!(
        play(&FAITHFUL, game(&setup, 1), 1, 7, false),
        Ok(played.clone())
    );
    assert_ne!(
        play(&FAITHFUL, game(&setup, 2), 2, 7, false).unwrap().hash,
        played.hash
    );
}

/// The town map's blue Tower (TownId 6, entered on (29, 10)).
const TOWER: TownId = TownId::new(6);

// covers: CPU-TRN-003, CPU-TRN-004
#[test]
fn a_garrison_hero_done_swaps_back_with_a_weaker_visitor() {
    // Blue's Gurnisson placed below the Tower's entrance; on day 1 Yog goes into the garrison.
    let mut setup = setup(1, 1);
    let gurnisson = 5;
    setup.heroes[gurnisson].pos = Pos::new(29, 11, 0);
    let mut state = game(&setup, 3);
    let blue = PlayerId::new(1);
    let mut memories = [Memory::new(), Memory::new()];
    let mut rng = Rng::for_stream(3, StreamId::AGENT);
    let mut commands = Vec::new();
    let mut entered = false;
    while state.day() <= 2 {
        let acting = state.turn_group()[0];
        if state.day() == 2 && acting == blue && !entered {
            // Day 2: Gurnisson steps onto the entrance, its garrison held by Yog.
            assert_eq!(
                state.adventure().town_hero(TOWER).map(|h| h.id),
                Some(HeroId::new(4))
            );
            let step = GameCommand::Adventure(AdventureCommand::MoveHero {
                hero: HeroId::new(5),
                path: vec![Pos::new(29, 10, 0)],
            });
            apply(
                &mut state,
                &FAITHFUL,
                Actor::Player(blue),
                step,
                &mut NoopSink,
            )
            .unwrap();
            entered = true;
        }
        let memory = &mut memories[usize::from(acting.raw())];
        let command = choose(&state, &FAITHFUL, acting, memory, &mut rng).unwrap();
        if state.day() == 2 && acting == blue {
            commands.push(command.clone());
        }
        apply(
            &mut state,
            &FAITHFUL,
            Actor::Player(acting),
            command,
            &mut NoopSink,
        )
        .unwrap();
    }
    // Yog, awake in the garrison, comes out; done at once, it finds Gurnisson's weaker army in
    // the garrison and the two change places again.
    let swap = GameCommand::Town(TownCommand::Heroes {
        town: TOWER,
        change: TownHeroChange::Swap,
    });
    assert_eq!(commands, [swap.clone(), swap, GameCommand::EndTurn]);
    assert_eq!(
        state.adventure().town_hero(TOWER).map(|h| h.id),
        Some(HeroId::new(4))
    );
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(48))]

    // covers: CPU-TRN-001, CPU-TRN-002, CPU-TRN-003, CPU-TRN-004
    #[test]
    fn the_computer_plays_the_generated_maps_legally_at_every_difficulty(
        map in 0..MAPS.len(),
        difficulty in 0_u8..5,
        seed in any::<u64>(),
        days in 1_u32..8,
    ) {
        let played = play(&FAITHFUL, game(&setup(map, difficulty), seed), seed, days, true);
        prop_assert_eq!(played.map(|played| played.day), Ok(days + 1));
    }
}
