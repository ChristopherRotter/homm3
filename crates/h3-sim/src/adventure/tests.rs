//! Tests of the adventure environment on the committed generated map `fixtures/maps/e08-duel-36.h3m`
//! (`h3-map`'s `tests/fixtures.rs` lays it out): the masks against the game's legal commands and
//! moves, steps against `apply`, refused actions, forks, determinism, the day limit (since E08
//! T16a on the duel's typed objects too); with a land Cartographer beside each hero (built here on
//! the nearest tile, also for the underground heroes: the duel's own two are farther), the open decisions ([`decisions`]);
//! with blue's hero two steps from red's, the battles (E08 T19b1, [`battles`]); with a monster
//! beside red's hero, its battle and the neutral seat (E08 T20b3a, [`monsters`]).

use std::sync::{Arc, LazyLock};

use h3_adventure::AdventureCommand;
use h3_adventure::{ObjectSetup, TileMask};
use h3_battle::LegalCommands;
use h3_contracts::PlayerId;
use h3_game::{
    Actor, GameCommand, GameCommandError, GameRuleset, GameSetup, GameState, apply, legal_commands,
    legal_moves, observe,
};
use h3_kernel::{Rng, StreamId};
use proptest::prelude::*;

use super::action::{self, hero_in_slot, tile_index, tile_pos};
use super::rollout::choose;
use super::*;
use crate::action as battle;

mod battles;
mod decisions;
mod monsters;

const DUEL: &[u8] = include_bytes!("../../../../fixtures/maps/e08-duel-36.h3m");

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
    GameRuleset::new(Arc::new(core), Arc::new(part), Arc::new(battle)).unwrap()
}

static FAITHFUL: LazyLock<GameRuleset> = LazyLock::new(|| rules("base:faithful"));
static STRICT: LazyLock<GameRuleset> = LazyLock::new(|| rules("base:strict-fixed"));

fn setup(rules: &GameRuleset) -> Arc<GameSetup> {
    let m = h3_map::read(DUEL, &h3_map::Limits::default()).unwrap();
    Arc::new(h3_map::import(&m, rules).unwrap())
}

static DUEL_SETUP: LazyLock<Arc<GameSetup>> = LazyLock::new(|| setup(&FAITHFUL));

/// The duel at `difficulty` with a land Cartographer near each hero: on the nearest tile it
/// reaches on day 1 by a last step not from above (a Cartographer is not entered from above,
/// ADV-OBJ-004).
fn chart_setup(rules: &GameRuleset, difficulty: u8) -> Arc<GameSetup> {
    let mut setup = GameSetup::clone(&setup(rules));
    setup.difficulty = difficulty;
    let chart = rules
        .adventure()
        .object_type_by_key("base:cartographer_land");
    assert!(chart.is_some(), "the original packs' land Cartographer");
    let state = GameState::new(rules, &setup, 0).unwrap();
    for hero in state.adventure().heroes() {
        let reach = state
            .adventure()
            .reach(rules.core(), rules.adventure(), state.heroes(), hero.id)
            .unwrap();
        let (_, anchor) = reach
            .tiles()
            .filter_map(|(pos, _)| {
                let path = reach.path(pos)?;
                let before = path.len().checked_sub(2).map_or(hero.pos, |i| path[i]);
                (before.y >= pos.y).then_some((path.len(), pos))
            })
            .min_by_key(|&(steps, _)| steps)
            .unwrap();
        setup.map.objects.push(ObjectSetup {
            anchor,
            blocked: TileMask::from_bits(1),
            visitable: TileMask::from_bits(1),
            object_type: chart,
            amount: None,
            drawn: TileMask::default(),
            monster: None,
            guards: None,
        });
    }
    Arc::new(setup)
}

/// The tiles of [`chart_setup`]'s Cartographers.
fn charts(setup: &GameSetup) -> Vec<h3_adventure::Pos> {
    let n = setup.map.objects.len() - setup.heroes.len();
    setup.map.objects[n..].iter().map(|o| o.anchor).collect()
}

fn env(seed: u64, max_days: u32) -> AdventureEnv {
    AdventureEnv::new(FAITHFUL.clone(), Arc::clone(&DUEL_SETUP), seed, max_days).unwrap()
}

fn acting(env: &AdventureEnv) -> PlayerId {
    env.to_move().next().and_then(Seat::player).unwrap()
}

#[test]
fn the_duel_starts_with_red_to_move_and_both_its_heroes_free() {
    let env = env(1, 28);
    assert_eq!(env.tile_count(), 36 * 36 * 2);
    assert_eq!(
        env.to_move().collect::<Vec<_>>(),
        [Seat::Player(PlayerId::new(0))]
    );
    assert_eq!((env.day(), env.steps(), env.is_done()), (1, 0, false));
    assert_eq!(env.phase(PlayerId::new(0)), Phase::Adventure);
    let mut mask = [true; ENTRY_COUNT];
    env.legal_mask(PlayerId::new(0), &mut mask);
    assert_eq!(
        mask,
        [
            true, true, true, false, false, false, false, false, false, false, false, false, false
        ]
    );
    env.legal_mask(PlayerId::new(1), &mut mask);
    assert_eq!(mask, [false; ENTRY_COUNT]);
    // After a legal `EndTurn` the tile head allows 0 alone; after an illegal one nothing.
    let mut tiles = Vec::new();
    env.tile_mask(PlayerId::new(0), END_TURN, &mut tiles);
    assert_eq!(tiles.len(), env.tile_count());
    assert_eq!(tiles.iter().filter(|&&t| t).count(), 1);
    assert!(tiles[0]);
    env.tile_mask(PlayerId::new(1), END_TURN, &mut tiles);
    assert!(!tiles.iter().any(|&t| t));
    assert_eq!(
        env.observation(PlayerId::new(1)),
        observe(
            env.state(),
            env.rules(),
            h3_game::Viewer::Player(PlayerId::new(1))
        )
    );
}

#[test]
fn tiles_and_actions_encode_as_documented() {
    let env = env(1, 0);
    let state = env.state();
    let pos = h3_adventure::Pos::new(5, 7, 1);
    assert_eq!(tile_index(state, pos), Some((36 + 7) * 36 + 5));
    assert_eq!(tile_pos(state, (36 + 7) * 36 + 5), Some(pos));
    assert_eq!(tile_index(state, h3_adventure::Pos::new(36, 0, 0)), None);
    assert_eq!(tile_index(state, h3_adventure::Pos::new(0, 36, 0)), None);
    assert_eq!(tile_index(state, h3_adventure::Pos::new(0, 0, 2)), None);
    assert_eq!(tile_pos(state, 36 * 36 * 2), None);
    assert_eq!(ACTION_ENCODING_VERSION, 4);
    assert_eq!((ACCEPT, DECLINE, ENTRY_COUNT), (9, 10, 13));
    assert_eq!((FIRST, SECOND), (11, 12));
    assert_eq!(Action::end_turn(), Action::Adventure(MapAction::default()));
    let map = |entry, tile| Action::Adventure(MapAction { entry, tile });
    assert_eq!(Action::move_to(2, 9), map(3, 9));
    assert_eq!(Action::accept(), map(9, 0));
    assert_eq!(Action::decline(), map(10, 0));
    assert_eq!(
        (Action::first(), Action::second()),
        (map(11, 0), map(12, 0))
    );
    assert_eq!(hero_in_slot(state, PlayerId::new(0), HERO_SLOTS), None);
    assert_eq!(hero_in_slot(state, PlayerId::new(1), 2), None);
    assert!(hero_in_slot(state, PlayerId::new(1), 1).is_some());
    for (error, text) in [
        (StepError::InvalidAction, "invalid action"),
        (StepError::EpisodeOver, "the episode is over"),
        (
            StepError::Rejected(GameCommandError::NotYourTurn(PlayerId::new(1))),
            "rejected: it is not player 1's turn",
        ),
    ] {
        assert_eq!(error.to_string(), text);
    }
}

/// One step of the property test's play.
#[derive(Debug, Clone)]
enum Pick {
    /// The acting seat's legal action, as the random agent picks it from this stream.
    Legal(u64),
    /// The acting player's first hero that can enter a Cartographer of [`chart_setup`] (or step
    /// toward the monster of [`monsters::monster_setup`]) does so; else as `Legal(0)`.
    Chart,
    /// The acting player's first hero that can reach another player's hero moves onto it (a
    /// battle, E08 T19b1); else as `Legal(0)`.
    Attack,
    /// An arbitrary map action of the acting seat (`None`) or of a player of the game, the one
    /// after the last or the neutral seat: the entry and tile modulo one more than their counts.
    Free {
        player: Option<u8>,
        entry: u8,
        tile: u32,
    },
    /// An arbitrary battle action of a seat as `Free`'s, the entry and heads modulo one more than
    /// their counts.
    FreeBattle {
        player: Option<u8>,
        entry: u16,
        heads: (u16, u16, u16),
    },
}

fn pick() -> impl Strategy<Value = Pick> {
    prop_oneof![
        6 => any::<u64>().prop_map(Pick::Legal),
        1 => Just(Pick::Chart),
        1 => Just(Pick::Attack),
        1 => (proptest::option::of(any::<u8>()), any::<u8>(), any::<u32>())
            .prop_map(|(player, entry, tile)| Pick::Free { player, entry, tile }),
        1 => (proptest::option::of(any::<u8>()), any::<u16>(), any::<(u16, u16, u16)>())
            .prop_map(|(player, entry, heads)| Pick::FreeBattle { player, entry, heads }),
    ]
}

/// Whether `entry` is a hero's move, the only entry that reads the tile head.
fn is_move(entry: usize) -> bool {
    (MOVE_BASE..ACCEPT).contains(&entry)
}

/// The masks of `seat` agree with the game. In the map phase: an entry is legal exactly when its
/// tile mask has a tile; the entries without a tile are the commands [`legal_commands`] lists
/// (with an open decision its answers alone: `Decline`, and `Accept` while the offer stands; a
/// level-up's `First` and `Second`, E08 T30), tile
/// 0 alone their tile mask and the only tile they decode with; a move's tiles are the hero's legal
/// moves but its own tile (since encoding v3 other players' heroes' too, E08 T19b1), each decoding
/// to the query's path. In the battle phase (the neutral seat's only phase) every map mask is
/// empty. The battle masks: [`check_battle_masks`].
fn check_masks(env: &AdventureEnv, seat: impl Into<Seat>) -> Result<(), TestCaseError> {
    let seat = seat.into();
    let (state, rules) = (env.state(), env.rules());
    check_battle_masks(env, seat)?;
    let mut entries = [true; ENTRY_COUNT];
    env.legal_mask(seat, &mut entries);
    let mut tiles = Vec::new();
    if env.phase(seat) == Phase::Battle {
        prop_assert_eq!(entries, [false; ENTRY_COUNT]);
        for entry in 0..=ENTRY_COUNT {
            env.tile_mask(seat, entry, &mut tiles);
            prop_assert!(!tiles.contains(&true), "entry {}", entry);
        }
        return Ok(());
    }
    let player = seat.player().unwrap();
    let commands = legal_commands(state, rules, Actor::Player(player));
    if state.decision(player).is_some() {
        let legal: Vec<usize> = (0..ENTRY_COUNT).filter(|&e| entries[e]).collect();
        prop_assert!(
            legal.contains(&DECLINE) && legal.iter().all(|&e| e == ACCEPT || e == DECLINE),
            "{:?}",
            legal
        );
    }
    // E08 T30: a level-up's answers alone.
    if state.level_up(player).is_some() {
        let legal: Vec<usize> = (0..ENTRY_COUNT).filter(|&e| entries[e]).collect();
        prop_assert_eq!(legal, [FIRST, SECOND]);
    }
    for (entry, &legal) in entries.iter().enumerate() {
        env.tile_mask(player, entry, &mut tiles);
        prop_assert_eq!(tiles.len(), env.tile_count());
        prop_assert_eq!(legal, tiles.iter().any(|&t| t), "entry {}", entry);
        if !is_move(entry) {
            prop_assert_eq!(tiles.iter().filter(|&&t| t).count(), usize::from(legal));
            let command = action::decode(state, rules, player, Action::map(entry, 0));
            prop_assert_eq!(
                legal,
                commands.contains(&command.unwrap()),
                "entry {}",
                entry
            );
            let off = Action::map(entry, 1);
            prop_assert_eq!(action::decode(state, rules, player, off), None);
            continue;
        }
        let Some(hero) = hero_in_slot(state, player, entry - MOVE_BASE) else {
            continue;
        };
        let reach = legal_moves(state, rules, Actor::Player(player), hero);
        let listed: Vec<u32> = reach
            .iter()
            .flat_map(|r| r.tiles())
            .map(|(pos, _)| tile_index(state, pos).unwrap())
            .collect();
        let masked: Vec<u32> = (0u32..)
            .zip(&tiles)
            .filter(|&(_, &t)| t)
            .map(|(i, _)| i)
            .collect();
        prop_assert_eq!(&masked, &listed);
        for &tile in masked.iter().step_by(17) {
            let action = Action::move_to(entry - MOVE_BASE, tile);
            let command = action::decode(state, rules, player, action);
            let path = reach
                .as_ref()
                .and_then(|r| r.path(tile_pos(state, tile).unwrap()));
            prop_assert_eq!(
                &command,
                &path.map(|path| GameCommand::Adventure(AdventureCommand::MoveHero { hero, path }))
            );
            prop_assert_eq!(
                action::encode(state, rules, player, &command.unwrap()),
                Some(action)
            );
        }
    }
    Ok(())
}

/// The battle masks of `seat` agree with its battle (E08 T19b1): outside a battle they are
/// empty and a battle action decodes to nothing, a battle command encodes to nothing; in one they
/// are the battle encoding's for the side it owns (empty while the other side is to act; the
/// neutral seat's battle is one whose unowned side is to act, E08 T20b3a), and each legal entry
/// with its first legal heads decodes to the battle's command, which encodes back to an action
/// that decodes to it again.
fn check_battle_masks(env: &AdventureEnv, seat: impl Into<Seat>) -> Result<(), TestCaseError> {
    let seat = seat.into();
    let (state, rules) = (env.state(), env.rules());
    let mut legal = LegalCommands::new();
    let (mut entries, mut spells, mut targets, mut seconds) = (
        [true; battle::ACTION_COUNT],
        [true; battle::SPELL_COUNT],
        [true; battle::TARGET_COUNT],
        [true; battle::TARGET_COUNT],
    );
    env.battle_mask(seat, &mut legal, &mut entries);
    let Some((fight, side)) = action::battle_of(state, seat) else {
        env.spell_mask(seat, battle::HERO_CAST, &mut legal, &mut spells);
        env.target_mask(seat, (battle::HERO_CAST, 0), &mut legal, &mut targets);
        let prefix = (battle::HERO_CAST, 0, 0);
        env.second_mask(seat, prefix, &mut legal, &mut seconds);
        prop_assert!(!entries.contains(&true) && !spells.contains(&true));
        prop_assert!(!targets.contains(&true) && !seconds.contains(&true));
        let wait = GameCommand::Battle(h3_battle::Command::Wait);
        prop_assert_eq!(action::encode(state, rules, seat, &wait), None);
        let wait = Action::Battle(battle::Action::entry(0));
        prop_assert_eq!(action::decode(state, rules, seat, wait), None);
        return Ok(());
    };
    let to_act = match seat {
        Seat::Player(player) => state.battle(player).unwrap().owner_to_act() == Some(player),
        Seat::Neutral => true,
    };
    let mut expected = [false; battle::ACTION_COUNT];
    battle::legal_mask(fight, rules.battle(), side, &mut legal, &mut expected);
    prop_assert_eq!(entries, expected);
    prop_assert_eq!(entries.contains(&true), to_act);
    let first = |mask: &[bool]| mask.iter().position(|&m| m);
    // Every ninth legal entry and the casts: each head's mask builds the battle's legal commands.
    let listed: Vec<usize> = (0..battle::ACTION_COUNT).filter(|&e| entries[e]).collect();
    let casts = [battle::HERO_CAST, battle::CREATURE_CAST];
    let some = listed
        .iter()
        .step_by(9)
        .chain(listed.iter().filter(|e| casts.contains(e)));
    for &entry in some {
        env.spell_mask(seat, entry, &mut legal, &mut spells);
        let spell = u16::try_from(first(&spells).unwrap()).unwrap();
        env.target_mask(seat, (entry, spell), &mut legal, &mut targets);
        let target = u16::try_from(first(&targets).unwrap()).unwrap();
        env.second_mask(seat, (entry, spell, target), &mut legal, &mut seconds);
        let second = u16::try_from(first(&seconds).unwrap()).unwrap();
        let chosen = battle::Action {
            entry: u32::try_from(entry).unwrap(),
            spell,
            target,
            second,
        };
        let command = action::decode(state, rules, seat, Action::Battle(chosen));
        let own = battle::decode(fight, rules.battle(), chosen).map(GameCommand::Battle);
        prop_assert!(command.is_some(), "entry {}", entry);
        prop_assert_eq!(&command, &own);
        let back = action::encode(state, rules, seat, &command.clone().unwrap());
        prop_assert!(matches!(back, Some(Action::Battle(_))));
        prop_assert_eq!(action::decode(state, rules, seat, back.unwrap()), command);
    }
    Ok(())
}

/// Whether `action` is legal for `seat` by its masks: a map action's entry and tile, a battle
/// action's entry and three heads.
fn legal_by_masks(env: &AdventureEnv, seat: Seat, action: Action) -> bool {
    match action {
        Action::Adventure(action) => {
            let mut entries = [false; ENTRY_COUNT];
            env.legal_mask(seat, &mut entries);
            let mut tiles = Vec::new();
            env.tile_mask(seat, action.entry, &mut tiles);
            entries.get(action.entry) == Some(&true)
                && tiles.get(action.tile as usize) == Some(&true)
        }
        Action::Battle(action) => {
            let mut legal = LegalCommands::new();
            let (mut entries, mut spells, mut targets) = (
                [false; battle::ACTION_COUNT],
                [false; battle::SPELL_COUNT],
                [false; battle::TARGET_COUNT],
            );
            let entry = action.entry as usize;
            env.battle_mask(seat, &mut legal, &mut entries);
            env.spell_mask(seat, entry, &mut legal, &mut spells);
            let heads = entries.get(entry) == Some(&true)
                && spells.get(usize::from(action.spell)) == Some(&true);
            env.target_mask(seat, (entry, action.spell), &mut legal, &mut targets);
            let heads = heads && targets.get(usize::from(action.target)) == Some(&true);
            let prefix = (entry, action.spell, action.target);
            env.second_mask(seat, prefix, &mut legal, &mut targets);
            heads && targets.get(usize::from(action.second)) == Some(&true)
        }
    }
}

/// The acting seat's move onto another player's hero, if one of its heroes can make it.
fn attack(env: &AdventureEnv, seat: Seat) -> Option<Action> {
    let heroes = env.state().adventure().heroes();
    let others: Vec<_> = heroes
        .iter()
        .filter(|h| Some(h.owner) != seat.player())
        .collect();
    enter_chart(env, seat, &others.iter().map(|h| h.pos).collect::<Vec<_>>())
}

/// The acting seat's move onto one of `charts`, if one of its heroes can make it.
fn enter_chart(env: &AdventureEnv, seat: Seat, charts: &[h3_adventure::Pos]) -> Option<Action> {
    let mut tiles = Vec::new();
    (0..HERO_SLOTS).find_map(|slot| {
        env.tile_mask(seat, MOVE_BASE + slot, &mut tiles);
        charts
            .iter()
            .filter_map(|&pos| tile_index(env.state(), pos))
            .find(|&tile| tiles[tile as usize])
            .map(|tile| Action::move_to(slot, tile))
    })
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 16, ..ProptestConfig::default() })]

    /// Random play of legal and arbitrary actions on the duel with a Cartographer beside each
    /// hero, or (E08 T19b1) with blue's hero two steps from red's ([`battles::battle_setup`]), or
    /// (E08 T20b3a) with a monster beside red's hero ([`monsters::monster_setup`], aimed at by
    /// `Chart`), at any difficulty (4, impossible, cannot pay an offer): the masks of both phases
    /// agree with the game for every seat, the neutral one included; a legal action is accepted
    /// and does what `apply` does with its command under the seat's actor; an action outside the
    /// masks is refused and leaves the environment as it was.
    #[test]
    fn steps_agree_with_the_masks_and_the_game(
        seed in any::<u64>(),
        strict in any::<bool>(),
        kind in 0u8..3,
        difficulty in 0u8..5,
        picks in proptest::collection::vec(pick(), 1..60),
    ) {
        let rules = if strict { &*STRICT } else { &*FAITHFUL };
        let (setup, charts) = match kind {
            0 => {
                let setup = chart_setup(rules, difficulty);
                let charts = charts(&setup);
                (setup, charts)
            }
            1 => (battles::battle_setup(rules, difficulty).0, Vec::new()),
            _ => {
                let (setup, monster) = monsters::monster_setup(rules, difficulty);
                (setup, vec![monster])
            }
        };
        let mut env = AdventureEnv::new(rules.clone(), setup, seed, 0).unwrap();
        let players = u8::try_from(env.state().players()).unwrap();
        let seats: Vec<Seat> = (0..players)
            .map(|p| Seat::Player(PlayerId::new(p)))
            .chain([Seat::Neutral])
            .collect();
        // The seat of a free pick: a player of the game, the one after the last, or the neutral
        // seat.
        let seat_of = |p: u8| match p % (players + 2) {
            n if n == players + 1 => Seat::Neutral,
            n => Seat::Player(PlayerId::new(n)),
        };
        for pick in &picks {
            let movers: Vec<Seat> = env.to_move().collect();
            for &seat in &seats {
                check_masks(&env, seat)?;
                // A seat to move has a legal action in its phase, any other none (E08 T19b1:
                // in a battle the owner of the side to act moves; T20b3a: the neutral seat for
                // an unowned side).
                let mut entries = [false; ENTRY_COUNT];
                env.legal_mask(seat, &mut entries);
                let mut fights = [false; battle::ACTION_COUNT];
                env.battle_mask(seat, &mut LegalCommands::new(), &mut fights);
                let any = entries.contains(&true) || fights.contains(&true);
                prop_assert_eq!(any, movers.contains(&seat), "{:?}", seat);
                if seat == Seat::Neutral {
                    prop_assert_eq!(env.neutral_observation().is_some(), any);
                }
            }
            let actor = env.to_move().next().unwrap();
            let (seat, action) = match *pick {
                Pick::Legal(stream) => {
                    let mut rng = Rng::for_stream(stream, StreamId::AGENT);
                    (actor, choose(&env, actor, &mut rng))
                }
                Pick::Chart | Pick::Attack => {
                    let mut rng = Rng::for_stream(0, StreamId::AGENT);
                    let aimed = if matches!(pick, Pick::Chart) {
                        enter_chart(&env, actor, &charts)
                    } else {
                        attack(&env, actor)
                    };
                    (actor, aimed.unwrap_or_else(|| choose(&env, actor, &mut rng)))
                }
                Pick::Free { player, entry, tile } => {
                    let seat = player.map_or(actor, seat_of);
                    let entry = usize::from(entry) % (ENTRY_COUNT + 1);
                    let tile = tile % (u32::try_from(env.tile_count()).unwrap() + 1);
                    (seat, Action::map(entry, tile))
                }
                Pick::FreeBattle { player, entry, heads: (spell, target, second) } => {
                    let seat = player.map_or(actor, seat_of);
                    let heads = |head: u16, count: usize| head % (u16::try_from(count).unwrap() + 1);
                    (seat, Action::Battle(battle::Action {
                        entry: u32::from(entry) % (u32::try_from(battle::ACTION_COUNT).unwrap() + 1),
                        spell: heads(spell, battle::SPELL_COUNT),
                        target: heads(target, battle::TARGET_COUNT),
                        second: heads(second, battle::TARGET_COUNT),
                    }))
                }
            };
            let legal = legal_by_masks(&env, seat, action);
            let command = action::decode(env.state(), env.rules(), seat, action);
            let mut expected = env.state().clone();
            let applied = command.clone().map(|command| {
                apply(&mut expected, env.rules(), seat.actor(), command, &mut Vec::new())
            });
            let (hash, steps) = (env.state_hash(), env.steps());
            let mut events = Vec::new();
            let result = env.step_with(seat, action, &mut events);
            if legal {
                prop_assert_eq!(result, Ok(StepResult { done: false, truncated: false }));
            }
            match result {
                Ok(_) => {
                    prop_assert!(legal, "{:?} {:?} accepted outside the masks", seat, action);
                    prop_assert_eq!(applied, Some(Ok(())));
                    prop_assert_eq!(env.state(), &expected);
                    prop_assert_eq!(env.steps(), steps + 1);
                    // A command tells something, but a one-step visit that does nothing (a
                    // border guard's or gate's without the key, E08 T15a, T15b).
                    let silent = matches!(
                        &command,
                        Some(GameCommand::Adventure(AdventureCommand::MoveHero { path, .. }))
                            if path.len() == 1
                                && expected.adventure().map().tile(path[0]).is_some_and(|t| t.visitable)
                    );
                    prop_assert!(!events.is_empty() || silent);
                }
                Err(error) => {
                    prop_assert!(!legal);
                    prop_assert_eq!((env.state_hash(), env.steps()), (hash, steps));
                    prop_assert!(events.is_empty());
                    match error {
                        StepError::InvalidAction => prop_assert_eq!(command, None),
                        StepError::Rejected(e) => prop_assert_eq!(applied, Some(Err(e))),
                        StepError::EpisodeOver => prop_assert!(false, "no day limit"),
                    }
                }
            }
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 12, ..ProptestConfig::default() })]

    /// Clone equivalence: a fork playing on leaves its parent as it was, and a fork and its
    /// parent evolve identically under the same agent.
    #[test]
    fn forks_evolve_as_their_parent(seed in any::<u64>(), agent in any::<u64>(), prefix in 1u32..80) {
        let mut parent = env(seed, 0);
        random_rollout(&mut parent, agent, prefix);
        let hash = parent.state_hash();
        let mut wild = parent.fork();
        random_rollout(&mut wild, agent ^ 1, 40);
        prop_assert_eq!(parent.state_hash(), hash);
        let mut fork = parent.fork();
        let a = random_rollout(&mut parent, agent, 40);
        let b = random_rollout(&mut fork, agent, 40);
        prop_assert_eq!(a, b);
        prop_assert_eq!(parent.state(), fork.state());
        prop_assert_eq!(parent.state_hash(), fork.state_hash());
    }
}

#[test]
fn rollouts_are_deterministic_and_end_at_the_day_limit() {
    let mut a = env(7, 4);
    let mut b = env(7, 4);
    let ra = random_rollout(&mut a, 11, 0);
    assert_eq!(ra, random_rollout(&mut b, 11, 0));
    assert_eq!(a.state_hash(), b.state_hash());
    assert_eq!(ra.day, 5);
    assert!(ra.steps > 8, "{ra:?}");
    assert!(a.is_done());
    assert_eq!(a.to_move().count(), 0);
    let mut mask = [true; ENTRY_COUNT];
    a.legal_mask(PlayerId::new(0), &mut mask);
    assert_eq!(mask, [false; ENTRY_COUNT]);
    let mut tiles = vec![true];
    a.tile_mask(PlayerId::new(0), END_TURN, &mut tiles);
    assert!(!tiles.iter().any(|&t| t));
    battles::assert_no_battle_masks(&a, PlayerId::new(0));
    assert_eq!(
        a.step(PlayerId::new(0), Action::end_turn()),
        Err(StepError::EpisodeOver)
    );
    // The heroes moved: random play is not only ending turns.
    let start = env(7, 4);
    let moved = a
        .state()
        .adventure()
        .heroes()
        .iter()
        .zip(start.state().adventure().heroes())
        .filter(|(now, then)| now.pos != then.pos)
        .count();
    assert!(moved >= 2, "{moved} heroes moved");
    a.reset(7).unwrap();
    assert_eq!((a.day(), a.steps(), a.is_done()), (1, 0, false));
    assert_eq!(a.state(), start.state());
    // A step limit stops a rollout early.
    let mut c = env(7, 4);
    assert_eq!(random_rollout(&mut c, 11, 5).steps, 5);
}

/// Counts the moves `player`'s flyers (`passed[0]`) and water walkers (`passed[1]`) may make that
/// pass over water or, for a flyer, over a tile an object blocks, and checks that each tile their
/// move heads allow is one a hero may stop on: land no live object blocks but where one is visited
/// (an entered object, a neighbour's visit; another player's hero stands on land).
fn check_lifted(env: &AdventureEnv, player: PlayerId, passed: &mut [usize; 2]) {
    let (state, rules) = (env.state(), env.rules());
    let adventure = state.adventure();
    let live = |pos: h3_adventure::Pos| {
        let objects = adventure.map().objects().iter().enumerate();
        objects
            .filter(|&(i, _)| !adventure.is_removed(i))
            .map(|(_, o)| o.covers(pos))
            .fold((false, false), |(b, v), (ob, ov)| (b || ob, v || ov))
    };
    let water = |pos| {
        let tile = adventure.map().tile(pos).unwrap();
        matches!(
            rules.adventure().passage(tile.terrain),
            Some(h3_adventure::Passage::Water(_))
        )
    };
    let mut tiles = Vec::new();
    for slot in 0..HERO_SLOTS {
        let Some(hero) = hero_in_slot(state, player, slot) else {
            continue;
        };
        let movement = h3_heroes::movement(state.hero(hero).unwrap(), rules.core());
        let kind = match (movement.flying, movement.water_walking) {
            (Some(_), _) => 0,
            (None, Some(_)) => 1,
            (None, None) => continue,
        };
        env.tile_mask(player, MOVE_BASE + slot, &mut tiles);
        let reach = legal_moves(state, rules, Actor::Player(player), hero);
        for tile in (0u32..).zip(&tiles).filter(|&(_, &t)| t).map(|(i, _)| i) {
            let pos = tile_pos(state, tile).unwrap();
            let (blocked, visited) = live(pos);
            assert!(
                !water(pos) && (!blocked || visited),
                "{hero:?} may stop on {pos:?}"
            );
            let path = reach.as_ref().and_then(|r| r.path(pos)).unwrap();
            let over = &path[..path.len() - 1];
            let lifted = over.iter().any(|&p| {
                let (blocked, visited) = live(p);
                water(p) || (kind == 0 && blocked && !visited)
            });
            passed[kind] += usize::from(lifted);
        }
    }
}

/// The env plays flyers and water walkers (E08 T42, design note §8: no encoding change): on the
/// duel (red's surface hero flies, its underground hero walks on water, `fixtures.rs`' kits), over
/// random games of two weeks, their move heads allow only tiles a hero may stop on, the reach's
/// stop tiles (`steps_agree_with_the_masks_and_the_game` checks them against the reach), and some
/// of those tiles are reached over water or (the flyer) over tiles objects block.
// covers: ADV-MOV-007, ADV-MOV-009, ADV-MOV-010
#[test]
fn a_flyer_and_a_water_walker_are_masked_by_their_stop_tiles() {
    let mut passed = [0; 2];
    for seed in 0..4 {
        let mut env = env(seed, 14);
        let mut rng = Rng::for_stream(seed, StreamId::AGENT);
        loop {
            let Some(seat) = env.to_move().next() else {
                break;
            };
            if let (Seat::Player(player), Phase::Adventure) = (seat, env.phase(seat)) {
                check_lifted(&env, player, &mut passed);
            }
            let action = choose(&env, seat, &mut rng);
            env.step(seat, action).unwrap();
        }
    }
    assert!(passed.iter().all(|&n| n > 0), "{passed:?}");
}

fn duel_map() -> replay::MapRef {
    replay::MapRef {
        path: "fixtures/maps/e08-duel-36.h3m".to_owned(),
        content_hash: replay::map_content_hash(DUEL),
        import_version: h3_map::MAP_IMPORT_VERSION,
    }
}

fn record(seed: u64, agent_seed: u64, max_days: u32) -> replay::Replay {
    let options = replay::RecordOptions {
        seed,
        agent_seed,
        max_days,
        max_steps: 0,
        checkpoint_every: 10,
    };
    replay::Replay::record_random(&FAITHFUL, Arc::clone(&DUEL_SETUP), duel_map(), options).unwrap()
}

/// A replay records the commands the random rollout plays and verifies against the game.
#[test]
fn a_recorded_game_is_the_rollout_and_verifies() {
    let r = record(3, 5, 3);
    let mut e = env(3, 3);
    let played = random_rollout(&mut e, 5, 0);
    assert_eq!(r.commands.len(), played.steps as usize);
    assert_eq!((r.final_hash, r.final_day), (e.state_hash(), 4));
    assert_eq!(r.checkpoints.len(), r.commands.len() / 10);
    assert!(
        r.commands
            .iter()
            .any(|(_, c)| matches!(c, GameCommand::Adventure(_)))
    );
    assert_eq!(r.verify(&FAITHFUL, &duel_map(), &DUEL_SETUP), Ok(()));
    // A step limit stops the record early; no checkpoints without an interval.
    let short = replay::Replay::record_random(
        &FAITHFUL,
        Arc::clone(&DUEL_SETUP),
        duel_map(),
        replay::RecordOptions {
            seed: 3,
            agent_seed: 5,
            max_days: 0,
            max_steps: 4,
            checkpoint_every: 0,
        },
    )
    .unwrap();
    assert_eq!((short.commands.len(), short.checkpoints.len()), (4, 0));
    assert_eq!(short.verify(&FAITHFUL, &duel_map(), &DUEL_SETUP), Ok(()));
}

/// Every way a replay can go wrong is reported, the first one found.
#[test]
fn a_tampered_replay_is_refused_with_its_reason() {
    use replay::ReplayError as E;
    let good = record(2, 9, 2);
    let verify = |r: &replay::Replay| r.verify(&FAITHFUL, &duel_map(), &DUEL_SETUP);
    let mut r = good.clone();
    r.header.profile = "base:strict-fixed".to_owned();
    assert_eq!(verify(&r), Err(E::Incompatible("profile")));
    // The strict rules are other rules: the header says so before any command runs.
    assert_eq!(
        good.verify(&STRICT, &duel_map(), &DUEL_SETUP),
        Err(E::Incompatible("profile"))
    );
    for (field, edit) in [
        (
            "format_version",
            (|r| r.header.format_version += 1) as fn(&mut replay::Replay),
        ),
        ("packs", |r| r.header.packs.pop().map_or((), drop)),
        ("ruleset_hash", |r| r.header.ruleset_hash ^= 1),
        ("adventure_ruleset_hash", |r| {
            r.header.adventure_ruleset_hash ^= 1
        }),
        ("rng", |r| r.header.rng_version += 1),
        ("hash", |r| r.header.hash_algorithm.push('x')),
        ("state_hash_versions", |r| {
            r.header.adventure_state_hash_version += 1
        }),
        ("map.hash", |r| r.map.content_hash ^= 1),
        ("map.import_version", |r| r.map.import_version += 1),
    ] {
        let mut r = good.clone();
        edit(&mut r);
        assert_eq!(verify(&r), Err(E::Incompatible(field)), "{field}");
    }
    // The engine version is informational, the path too (the hash names the file).
    let mut r = good.clone();
    r.header.engine_version.push('x');
    r.map.path.push('x');
    assert_eq!(verify(&r), Ok(()));
    let mut r = good.clone();
    r.checkpoints.pop();
    assert_eq!(verify(&r), Err(E::CheckpointCount));
    let mut r = good.clone();
    r.checkpoints[0] ^= 1;
    assert!(matches!(verify(&r), Err(E::HashMismatch { step: 10, .. })));
    let mut r = good.clone();
    r.final_hash ^= 1;
    let n = r.commands.len();
    assert!(matches!(verify(&r), Err(E::HashMismatch { step, .. }) if step == n));
    let mut r = good.clone();
    r.final_day += 1;
    assert_eq!(
        verify(&r),
        Err(E::DayMismatch {
            expected: good.final_day + 1,
            actual: good.final_day
        })
    );
    // A command of the player not acting, then the game's own seed changed.
    let mut r = good.clone();
    let first = match r.commands[0].0 {
        Actor::Player(player) => player.raw(),
        Actor::System => 1,
    };
    r.commands[0].0 = Actor::Player(PlayerId::new(1 - first));
    assert!(matches!(
        verify(&r),
        Err(E::Rejected {
            step: 0,
            error: GameCommandError::NotYourTurn(_)
        })
    ));
    let mut r = good.clone();
    r.seed ^= 1;
    let changed = verify(&r);
    assert!(
        matches!(changed, Err(E::HashMismatch { .. })),
        "{changed:?}"
    );
    // Each error explains itself.
    for e in [
        E::Incompatible("profile"),
        E::Setup(h3_game::GameSetupError::Players(9)),
        E::Rejected {
            step: 0,
            error: GameCommandError::NotYourTurn(PlayerId::new(1)),
        },
        E::HashMismatch {
            step: 1,
            expected: 1,
            actual: 2,
        },
        E::CheckpointCount,
        E::DayMismatch {
            expected: 2,
            actual: 3,
        },
    ] {
        assert!(!e.to_string().is_empty());
    }
}

/// A command whose path the env's action could not name (a client's own path) steps the env and
/// replays: the replay holds commands, not actions (the user, T9b).
#[test]
fn a_path_of_the_players_own_choice_steps_and_replays() {
    let mut e = env(1, 0);
    let player = acting(&e);
    let hero = hero_in_slot(e.state(), player, 0).unwrap();
    let reach = legal_moves(e.state(), e.rules(), Actor::Player(player), hero).unwrap();
    // A reachable tile two steps away, reached through a neighbour other than the query's.
    let (target, detour) = reach
        .tiles()
        .filter_map(|(pos, _)| {
            let path = reach.path(pos)?;
            let [first, second] = path.as_slice() else {
                return None;
            };
            let other = reach.tiles().map(|(p, _)| p).find(|&p| {
                p != *first
                    && neighbours(p, *second)
                    && neighbours(p, e.state().adventure().hero(hero).unwrap().pos)
            })?;
            Some((*second, vec![other, *second]))
        })
        .next()
        .expect("the duel has a tile with two two-step paths");
    let canonical = action::decode(
        e.state(),
        e.rules(),
        player,
        Action::move_to(0, tile_index(e.state(), target).unwrap()),
    )
    .unwrap();
    let own = GameCommand::Adventure(AdventureCommand::MoveHero { hero, path: detour });
    assert_ne!(own, canonical);
    // The action names the destination: the own path encodes to the query's.
    assert_eq!(
        action::encode(e.state(), e.rules(), player, &own),
        action::encode(e.state(), e.rules(), player, &canonical)
    );
    e.step_command(player, own.clone(), &mut h3_game::NoopSink)
        .unwrap();
    let replay = replay::Replay {
        header: replay::ReplayHeader::current(&FAITHFUL),
        map: duel_map(),
        seed: 1,
        difficulty: h3_game::DEFAULT_DIFFICULTY,
        commands: vec![(Actor::Player(player), own)],
        checkpoint_every: 0,
        checkpoints: Vec::new(),
        final_day: 1,
        final_hash: e.state_hash(),
    };
    assert_eq!(replay.verify(&FAITHFUL, &duel_map(), &DUEL_SETUP), Ok(()));
}

fn neighbours(a: h3_adventure::Pos, b: h3_adventure::Pos) -> bool {
    a.level == b.level && a != b && a.x.abs_diff(b.x) <= 1 && a.y.abs_diff(b.y) <= 1
}

/// A refused command of the player's own leaves the env as it was and emits nothing; after the
/// episode no command steps.
#[test]
fn a_refused_command_changes_nothing() {
    #[derive(Default)]
    struct Count(usize);
    impl h3_game::Sink for Count {
        fn emit(&mut self, _: h3_game::GameEvent) {
            self.0 += 1;
        }
    }
    let mut e = env(1, 1);
    let player = acting(&e);
    let hero = hero_in_slot(e.state(), player, 0).unwrap();
    let at = e.state().adventure().hero(hero).unwrap().pos;
    let far = h3_adventure::Pos::new(at.x + 3, at.y, at.level);
    let (hash, steps) = (e.state_hash(), e.steps());
    for path in [Vec::new(), vec![far], vec![at]] {
        let mut sink = Count::default();
        let move_hero = GameCommand::Adventure(AdventureCommand::MoveHero { hero, path });
        let refused = e.step_command(player, move_hero, &mut sink);
        assert!(
            matches!(refused, Err(StepError::Rejected(_))),
            "{refused:?}"
        );
        assert_eq!((e.state_hash(), e.steps(), sink.0), (hash, steps, 0));
    }
    let other = PlayerId::new(1 - player.raw());
    let mut sink = Count::default();
    assert_eq!(
        e.step_command(other, GameCommand::EndTurn, &mut sink),
        Err(StepError::Rejected(GameCommandError::NotYourTurn(other)))
    );
    assert_eq!((e.state_hash(), sink.0), (hash, 0));
    random_rollout(&mut e, 3, 0);
    assert!(e.is_done());
    let hash = e.state_hash();
    assert_eq!(
        e.step_command(PlayerId::new(0), GameCommand::EndTurn, &mut sink),
        Err(StepError::EpisodeOver)
    );
    assert_eq!((e.state_hash(), sink.0), (hash, 0));
}

#[test]
fn a_record_needs_a_limit() {
    let options = replay::RecordOptions {
        seed: 1,
        agent_seed: 1,
        max_days: 0,
        max_steps: 0,
        checkpoint_every: 0,
    };
    let none =
        replay::Replay::record_random(&FAITHFUL, Arc::clone(&DUEL_SETUP), duel_map(), options);
    assert_eq!(none, Err(replay::RecordError::NoLimit));
    for e in [
        replay::RecordError::NoLimit,
        replay::RecordError::Setup(h3_game::GameSetupError::Players(9)),
        replay::RecordError::Refused {
            step: 3,
            error: StepError::InvalidAction,
        },
    ] {
        assert!(!e.to_string().is_empty());
    }
}
