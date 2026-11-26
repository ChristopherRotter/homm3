//! The env's casts on the map (E08 T144a2, action encoding v21; GAM-SPL-001, E08 decisions 421,
//! 422): on the duel with a Shipyard of each player's and every hero holding a spell book, the
//! five adventure spells and spell points to spare ([`caster_setup`]), each hero slot's five
//! entries: Summon Boat's, Fly's and Water Walk's legal with tile 0 alone where `legal_commands`
//! lists the hero's cast, Scuttle Boat's and Dimension Door's where the hero's targets query is
//! not empty, their tile masks its targets ([`check_cast`]); each legal action decodes to its cast
//! and encodes back; an empty slot, a tile for a spell without a target, a target off the map, a
//! building and the neutral seat name nothing; the casts step through the env; the random agent
//! draws both kinds.

use h3_adventure::AdventureSpell;
use h3_content::SpellId;
use h3_game::{GameEvent, HeroArtifactsSetup, legal_cast_targets};
use h3_heroes::ArtifactInstance;

use super::decisions::legal_entries;
use super::shipyards::shipyard_setup;
use super::*;

const RED: PlayerId = PlayerId::new(0);
const BLUE: PlayerId = PlayerId::new(1);

/// The five adventure spells' keys in the original packs, in `AdventureSpell::ALL`'s order (the
/// entries' order within a slot).
const SPELLS: [&str; CAST_SPELLS] = [
    "base:summon_boat",
    "base:scuttle_boat",
    "base:fly",
    "base:water_walk",
    "base:dimension_door",
];

/// [`shipyard_setup`] at `difficulty` with every hero holding a spell book, the five adventure
/// spells ([`SPELLS`]) and Knowledge 20 (200 spell points at the start).
pub(super) fn caster_setup(rules: &GameRuleset, difficulty: u8) -> Arc<GameSetup> {
    let mut setup = GameSetup::clone(&shipyard_setup(rules, difficulty));
    let core = rules.core();
    let book = ArtifactInstance::from(core.artifact_by_key("base:spell_book").unwrap());
    let spells: Vec<SpellId> = SPELLS
        .iter()
        .map(|k| core.spell_by_key(k).unwrap())
        .collect();
    for hero in &mut setup.heroes {
        hero.details.spells = Some(spells.clone());
        hero.details.artifacts = Some(HeroArtifactsSetup {
            worn: h3_heroes::wearing(core, &[book]),
            backpack: Vec::new(),
        });
        hero.details.primary = Some([1, 1, 1, 20]);
    }
    Arc::new(setup)
}

/// The core spell the original packs bind to `spell` ([`SPELLS`]).
fn spell_id(rules: &GameRuleset, spell: AdventureSpell) -> SpellId {
    let key = SPELLS[usize::from(spell.code())];
    rules.core().spell_by_key(key).unwrap()
}

/// The tiles `tiles` allows, in order.
fn masked(tiles: &[bool]) -> Vec<u32> {
    (0u32..)
        .zip(tiles)
        .filter(|&(_, &t)| t)
        .map(|(i, _)| i)
        .collect()
}

/// The cast entry `entry`'s `tiles` for `player` agree with the game (E08 T144a2): Summon Boat's,
/// Fly's and Water Walk's 0 alone when `commands`, its legal commands, list the cast of the
/// slot's hero, Scuttle Boat's and Dimension Door's the hero's queried targets
/// ([`legal_cast_targets`]) in row order; each such action decodes to its cast, which encodes
/// back to the action; a spell without a target names nothing at another tile.
pub(super) fn check_cast(
    env: &AdventureEnv,
    (player, entry): (PlayerId, usize),
    commands: &[GameCommand],
    tiles: &[bool],
) -> Result<(), TestCaseError> {
    let (state, rules) = (env.state(), env.rules());
    let at = entry - CAST_BASE;
    let (slot, spell) = (at / CAST_SPELLS, AdventureSpell::ALL[at % CAST_SPELLS]);
    let id = spell_id(rules, spell);
    let hero = hero_in_slot(state, player, slot);
    let cast = |hero, target| GameCommand::Cast {
        hero,
        spell: id,
        target,
    };
    let expected: Vec<(u32, GameCommand)> = match hero {
        Some(hero) if spell.targets() => {
            legal_cast_targets(state, rules, Actor::Player(player), (hero, id))
                .into_iter()
                .map(|pos| (tile_index(state, pos).unwrap(), cast(hero, Some(pos))))
                .collect()
        }
        Some(hero) => Some(cast(hero, None))
            .filter(|command| commands.contains(command))
            .map(|command| (0, command))
            .into_iter()
            .collect(),
        None => Vec::new(),
    };
    let listed: Vec<u32> = expected.iter().map(|&(tile, _)| tile).collect();
    prop_assert_eq!(masked(tiles), listed, "entry {}", entry);
    for (tile, command) in expected.iter().step_by(17) {
        let action = Action::map(entry, *tile);
        let decoded = action::decode(state, rules, player, action);
        prop_assert_eq!(decoded.as_ref(), Some(command), "entry {}", entry);
        prop_assert_eq!(action::encode(state, rules, player, command), Some(action));
    }
    if !spell.targets() {
        let off = Action::map(entry, 1);
        prop_assert_eq!(action::decode(state, rules, player, off), None);
    }
    Ok(())
}

/// The duel's casters under `rules` at seed 1, difficulty 1.
fn caster_env(rules: &GameRuleset) -> AdventureEnv {
    AdventureEnv::new(rules.clone(), caster_setup(rules, 1), 1, 0).unwrap()
}

// covers: GAM-SPL-001
#[test]
fn each_hero_slot_has_five_cast_entries_masked_by_the_game() {
    for rules in [&*FAITHFUL, &*STRICT] {
        let env = caster_env(rules);
        let state = env.state();
        let commands = legal_commands(state, rules, Actor::Player(RED));
        let mut tiles = Vec::new();
        for slot in 0..HERO_SLOTS {
            for spell in AdventureSpell::ALL {
                let entry = CAST_BASE + slot * CAST_SPELLS + usize::from(spell.code());
                assert_eq!(Action::cast(slot, spell, 3), Action::map(entry, 3));
                env.tile_mask(RED, entry, &mut tiles);
                check_cast(&env, (RED, entry), &commands, &tiles).unwrap();
            }
        }
        // Red's two heroes on land with 200 points: Fly, Water Walk and the door each; blue's
        // turn has not come.
        let legal = legal_entries(&env, RED);
        for slot in 0..2 {
            for spell in [
                AdventureSpell::Fly,
                AdventureSpell::WaterWalk,
                AdventureSpell::DimensionDoor,
            ] {
                let entry = CAST_BASE + slot * CAST_SPELLS + usize::from(spell.code());
                assert!(
                    legal.contains(&entry),
                    "{spell:?} of slot {slot}: {legal:?}"
                );
            }
        }
        assert!(!legal.iter().any(|&e| e >= CAST_BASE + 2 * CAST_SPELLS));
        assert!(!legal_entries(&env, BLUE).iter().any(|&e| e >= CAST_BASE));
        let mut entries = [true; ENTRY_COUNT];
        env.legal_mask(Seat::Neutral, &mut entries);
        assert_eq!(entries, [false; ENTRY_COUNT]);
    }
}

// covers: GAM-SPL-001
#[test]
fn a_cast_names_its_hero_spell_and_target_alone() {
    let env = caster_env(&FAITHFUL);
    let (state, rules) = (env.state(), env.rules());
    let hero = hero_in_slot(state, RED, 0).unwrap();
    let blues = hero_in_slot(state, BLUE, 0).unwrap();
    let (fly, door) = (
        spell_id(rules, AdventureSpell::Fly),
        spell_id(rules, AdventureSpell::DimensionDoor),
    );
    let pos = state.adventure().hero(hero).unwrap().pos;
    let tile = tile_index(state, pos).unwrap();
    let flight = GameCommand::Cast {
        hero,
        spell: fly,
        target: None,
    };
    let jump = GameCommand::Cast {
        hero,
        spell: door,
        target: Some(pos),
    };
    for (action, command) in [
        (Action::cast(0, AdventureSpell::Fly, 0), &flight),
        (Action::cast(0, AdventureSpell::DimensionDoor, tile), &jump),
    ] {
        assert_eq!(
            action::decode(state, rules, RED, action).as_ref(),
            Some(command)
        );
        assert_eq!(action::encode(state, rules, RED, command), Some(action));
    }
    // An empty slot, a tile for Fly, a target off the map, a building, the neutral seat and
    // another player name nothing.
    let off = u32::try_from(env.tile_count()).unwrap();
    let built = Action::Adventure(MapAction {
        entry: CAST_BASE + usize::from(AdventureSpell::Fly.code()),
        tile: 0,
        building: 1,
    });
    for (seat, action) in [
        (
            Seat::Player(RED),
            Action::cast(HERO_SLOTS - 1, AdventureSpell::Fly, 0),
        ),
        (
            Seat::Player(RED),
            Action::cast(0, AdventureSpell::Fly, tile),
        ),
        (
            Seat::Player(RED),
            Action::cast(0, AdventureSpell::DimensionDoor, off),
        ),
        (Seat::Player(RED), built),
        (Seat::Neutral, Action::cast(0, AdventureSpell::Fly, 0)),
    ] {
        assert_eq!(
            action::decode(state, rules, seat, action),
            None,
            "{action:?}"
        );
    }
    // A target with Fly, none with the door, a target off the map, a spell the part does not
    // bind, another player's hero and the neutral seat encode to nothing.
    let magic_arrow = rules.core().spell_by_key("base:magic_arrow").unwrap();
    let beyond = h3_adventure::Pos::new(36, 0, 0);
    for (seat, command) in [
        (
            RED,
            GameCommand::Cast {
                hero,
                spell: fly,
                target: Some(pos),
            },
        ),
        (
            RED,
            GameCommand::Cast {
                hero,
                spell: door,
                target: None,
            },
        ),
        (
            RED,
            GameCommand::Cast {
                hero,
                spell: door,
                target: Some(beyond),
            },
        ),
        (
            RED,
            GameCommand::Cast {
                hero,
                spell: magic_arrow,
                target: None,
            },
        ),
        (
            RED,
            GameCommand::Cast {
                hero: blues,
                spell: fly,
                target: None,
            },
        ),
    ] {
        assert_eq!(
            action::encode(state, rules, seat, &command),
            None,
            "{command:?}"
        );
    }
    assert_eq!(action::encode(state, rules, Seat::Neutral, &flight), None);
    assert!(
        action::cast_targets(
            state,
            rules,
            Seat::Neutral,
            (0, AdventureSpell::DimensionDoor)
        )
        .is_empty()
    );
}

// covers: GAM-SPL-001
#[test]
fn the_casts_step_through_the_env() {
    let mut env = caster_env(&FAITHFUL);
    let hero = hero_in_slot(env.state(), RED, 0).unwrap();
    let mut events = Vec::new();
    env.step_with(RED, Action::cast(0, AdventureSpell::Fly, 0), &mut events)
        .unwrap();
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::SpellCast { .. })),
        "{events:?}"
    );
    // The door's first target the hero's query lists.
    let mut tiles = Vec::new();
    let door = CAST_BASE + usize::from(AdventureSpell::DimensionDoor.code());
    env.tile_mask(RED, door, &mut tiles);
    let tile = masked(&tiles)[0];
    let before = env.state().adventure().hero(hero).unwrap().pos;
    events.clear();
    env.step_with(RED, Action::map(door, tile), &mut events)
        .unwrap();
    assert!(!events.is_empty());
    let after = env.state().adventure().hero(hero).unwrap().pos;
    assert!(
        after == tile_pos(env.state(), tile).unwrap() || after == before,
        "{after:?}"
    );
    // Out of its turn blue's cast is refused, and changes nothing.
    let hash = env.state_hash();
    assert_eq!(
        env.step(BLUE, Action::cast(0, AdventureSpell::Fly, 0)),
        Err(StepError::Rejected(GameCommandError::NotYourTurn(BLUE)))
    );
    assert_eq!(env.state_hash(), hash);
}

// covers: GAM-SPL-001
#[test]
fn the_random_agent_draws_both_kinds_of_cast() {
    let env = caster_env(&FAITHFUL);
    let mut drawn = [false; 2];
    for seed in 0..200 {
        let mut rng = Rng::for_stream(seed, StreamId::AGENT);
        let Action::Adventure(chosen) = choose(&env, RED, &mut rng) else {
            panic!("a map action");
        };
        let Some(at) = chosen.entry.checked_sub(CAST_BASE) else {
            continue;
        };
        let spell = AdventureSpell::ALL[at % CAST_SPELLS];
        drawn[usize::from(spell.targets())] = true;
        let mut fork = env.fork();
        assert!(
            fork.step(RED, Action::Adventure(chosen)).is_ok(),
            "{chosen:?}"
        );
    }
    assert_eq!(drawn, [true, true]);
}
