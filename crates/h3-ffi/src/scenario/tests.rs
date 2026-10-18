//! E05 T2: every place a battle is fought, created from a scenario's text through the C ABI on
//! the shipped packs and played to the end; the errors as messages.

use std::ffi::{CStr, c_char};

use h3_sim::action::{HERO_CAST, RETREAT, SURRENDER, WALL_SHOT_BASE};
use h3_sim::{ACTION_COUNT, OBSERVATION_LEN, SPELL_COUNT, TARGET_COUNT};

use super::write_error;
use crate::*;

const FIELD: &str = include_str!("field.toml");
const GROUND: &str = include_str!("ground.toml");
const GARRISON: &str = include_str!("garrison.toml");
const TOWN: &str = include_str!("town.toml");

/// Every special battlefield of the shipped packs with a terrain it lies on.
const GROUNDS: [(&str, &str); 12] = [
    ("base:desert", "base:shore"),
    ("base:grass", "base:magic_plains"),
    ("base:dirt", "base:cursed_ground"),
    ("base:water", "base:deck"),
    ("base:water", "base:boats"),
    ("base:grass", "sod:holy_ground"),
    ("base:swamp", "sod:evil_fog"),
    ("base:grass", "sod:clover_field"),
    ("base:dirt", "sod:lucid_pools"),
    ("base:lava", "sod:fiery_fields"),
    ("base:rough", "sod:rocklands"),
    ("base:snow", "sod:magic_clouds"),
];

/// The step limit of the tests' environments.
const MAX_STEPS: u32 = 3000;

/// Creates an environment from `text` with `seed` through the ABI: the handle, or the error's
/// message.
fn create(text: &str, seed: u64) -> Result<*mut H3Env, String> {
    let mut error = [0 as c_char; 512];
    // SAFETY: `text` is valid for its length, `error` for its length.
    let env = unsafe {
        h3_env_create_scenario(
            text.as_ptr().cast(),
            text.len(),
            seed,
            MAX_STEPS,
            error.as_mut_ptr(),
            error.len(),
        )
    };
    if env.is_null() {
        // SAFETY: on error the message is NUL-terminated within the buffer.
        Err(unsafe { CStr::from_ptr(error.as_ptr()) }
            .to_string_lossy()
            .into_owned())
    } else {
        Ok(env)
    }
}

/// What a played battle showed.
#[derive(Debug, Default)]
struct Played {
    steps: u32,
    result: H3StepResult,
    /// Decisions at which the hero's cast was legal.
    hero_cast_legal: u32,
    /// Decisions at which a wall shot was legal (a hero aims the Catapult).
    wall_shot_legal: u32,
    /// Casts chosen (a hero's or a stack's).
    casts: u32,
}

/// A uniformly random index among the set bytes of `mask` (from a linear congruential
/// generator, as `c/smoke.c`); `mask.len()` if none is set.
fn pick(mask: &[u8], lcg: &mut u64) -> usize {
    let legal = mask.iter().filter(|m| **m == 1).count();
    if legal == 0 {
        return mask.len();
    }
    *lcg = lcg
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);
    let n = (*lcg >> 33) as usize % legal;
    mask.iter()
        .enumerate()
        .filter(|(_, m)| **m == 1)
        .nth(n)
        .map_or(mask.len(), |(i, _)| i)
}

/// Plays `env` to its end through the ABI with random legal actions, each head drawn from its
/// mask, retreat and surrender masked out; observes both sides at every decision. Destroys
/// `env`.
fn play(env: *mut H3Env, seed: u64) -> Played {
    let mut mask = vec![0u8; ACTION_COUNT];
    let mut spells = vec![0u8; SPELL_COUNT];
    let mut targets = vec![0u8; TARGET_COUNT];
    let mut obs = vec![0f32; OBSERVATION_LEN];
    let mut lcg = seed;
    let mut played = Played::default();
    // SAFETY: `env` is a live handle, the buffers have the lengths passed.
    unsafe {
        while h3_env_to_move(env) >= 0 {
            let side = h3_env_to_move(env);
            for viewer in 0..2 {
                assert_eq!(
                    h3_env_observe(env, viewer, obs.as_mut_ptr(), obs.len()),
                    H3_OK
                );
            }
            assert_eq!(
                h3_env_legal_mask(env, side, mask.as_mut_ptr(), mask.len()),
                H3_OK
            );
            mask[RETREAT] = 0;
            mask[SURRENDER] = 0;
            played.hero_cast_legal += u32::from(mask[HERO_CAST]);
            played.wall_shot_legal += u32::from(mask[WALL_SHOT_BASE..].contains(&1));
            let entry = pick(&mask, &mut lcg) as u32;
            assert_eq!(
                h3_env_spell_mask(env, side, entry, spells.as_mut_ptr(), spells.len()),
                H3_OK
            );
            let spell = pick(&spells, &mut lcg) as u16;
            assert_eq!(
                h3_env_target_mask(env, side, entry, spell, targets.as_mut_ptr(), targets.len()),
                H3_OK
            );
            let target = pick(&targets, &mut lcg) as u16;
            assert_eq!(
                h3_env_second_mask(
                    env,
                    side,
                    entry,
                    spell,
                    target,
                    targets.as_mut_ptr(),
                    targets.len()
                ),
                H3_OK
            );
            let second = pick(&targets, &mut lcg) as u16;
            played.casts +=
                u32::from(entry as usize >= HERO_CAST && (entry as usize) < WALL_SHOT_BASE);
            let action = H3Action {
                entry,
                spell,
                target,
                second,
            };
            assert_eq!(h3_env_step(env, side, action, &mut played.result), H3_OK);
            played.steps += 1;
        }
        h3_env_destroy(env);
    }
    assert_eq!(played.result.done, 1);
    assert!(
        played.result.result >= H3_RESULT_ATTACKER || played.result.truncated == 1,
        "{played:?}"
    );
    played
}

#[test]
fn a_field_battle_on_the_shipped_packs() {
    let played = play(create(FIELD, 1).unwrap(), 1);
    assert!(played.steps > 10, "{played:?}");
    assert!(played.hero_cast_legal > 0, "{played:?}");
}

#[test]
fn every_special_battlefield() {
    for (seed, (terrain, ground)) in (1..).zip(GROUNDS) {
        let text = GROUND
            .replace("{terrain}", terrain)
            .replace("{battlefield}", ground);
        let env = create(&text, seed).unwrap_or_else(|e| panic!("{ground}: {e}"));
        let played = play(env, seed);
        assert!(played.steps > 0, "{ground}: {played:?}");
    }
}

#[test]
fn a_garrison_forbids_casting() {
    // BAT-MAG-061: at an anti-magic garrison no hero casts; the same battle elsewhere lets them.
    let garrison = play(create(GARRISON, 2).unwrap(), 2);
    assert_eq!(garrison.hero_cast_legal, 0, "{garrison:?}");
    let field = GARRISON.replace("anti_magic = true", "anti_magic = false");
    let field = play(create(&field, 2).unwrap(), 2);
    assert!(field.hero_cast_legal > 0, "{field:?}");
}

#[test]
fn a_town_under_siege() {
    // The attacker's hero aims the Catapult (BAT-SIE-011): the wall shots become legal, which
    // they never are in the field.
    let town = play(create(TOWN, 3).unwrap(), 3);
    assert!(town.wall_shot_legal > 0, "{town:?}");
    let start = TOWN.find("[scenario.siege]").unwrap();
    let end = TOWN.find("[scenario.attacker_hero]").unwrap();
    let field = format!("{}{}", &TOWN[..start], &TOWN[end..]);
    let field = play(create(&field, 3).unwrap(), 3);
    assert_eq!(field.wall_shot_legal, 0, "{field:?}");
}

#[test]
fn an_episode_follows_its_seed() {
    // The obstacles are generated from each episode's seed (BAT-GEO-004): the same seed gives the
    // same battle, a reset with it the first state again.
    let [a, b, c] = [5, 5, 6].map(|seed| create(FIELD, seed).unwrap());
    // SAFETY: live handles, each destroyed once.
    unsafe {
        let first = h3_env_state_hash(a);
        assert_eq!(h3_env_state_hash(b), first);
        assert_ne!(h3_env_state_hash(c), first);
        let mut result = H3StepResult::default();
        let mut mask = vec![0u8; ACTION_COUNT];
        let side = h3_env_to_move(a);
        assert_eq!(
            h3_env_legal_mask(a, side, mask.as_mut_ptr(), mask.len()),
            H3_OK
        );
        let entry = mask.iter().position(|m| *m == 1).unwrap() as u32;
        let action = H3Action {
            entry,
            ..H3Action::default()
        };
        assert_eq!(h3_env_step(a, side, action, &mut result), H3_OK);
        assert_ne!(h3_env_state_hash(a), first);
        assert_eq!(h3_env_reset(a, 5), H3_OK);
        assert_eq!(h3_env_state_hash(a), first);
        for env in [a, b, c] {
            h3_env_destroy(env);
        }
    }
}

#[test]
fn the_synthetic_scenarios_by_text() {
    // The synthetic siege, which `h3_env_create_synthetic` does not offer (F-robust-2), and the
    // skirmish by text equal to its number.
    play(create(h3_pack_battle::SYNTHETIC_SIEGE, 7).unwrap(), 7);
    let by_text = create(h3_pack_battle::SYNTHETIC_SKIRMISH, 4).unwrap();
    let by_number = h3_env_create_synthetic(1, 4, MAX_STEPS);
    // SAFETY: live handles, each destroyed once.
    unsafe {
        assert_eq!(h3_env_state_hash(by_text), h3_env_state_hash(by_number));
        h3_env_destroy(by_text);
        h3_env_destroy(by_number);
    }
}

#[test]
fn errors_are_messages() {
    let error = |text: &str| create(text, 1).unwrap_err();
    assert!(error("[scenario").starts_with("scenario: "));
    let hota = FIELD.replace(
        r#"packs = ["base", "ab", "sod"]"#,
        r#"packs = ["base", "hota"]"#,
    );
    assert_eq!(
        error(&hota),
        "unknown pack `hota` (built-in packs: synthetic, synthetic-ext, base, ab, sod)"
    );
    let twice = FIELD.replace(
        r#"packs = ["base", "ab", "sod"]"#,
        r#"packs = ["base", "base"]"#,
    );
    assert!(error(&twice).starts_with("packs: "));
    let dragonfly = FIELD.replace("base:goblin", "base:dragonfly");
    assert!(error(&dragonfly).starts_with("scenario: "));
    let empty = FIELD.replace("count = 25", "count = 0");
    assert!(error(&empty).starts_with("setup: "));

    let mut buffer = [b'x' as c_char; 8];
    // SAFETY: the pointers are null or valid for the lengths passed.
    unsafe {
        let null =
            h3_env_create_scenario(std::ptr::null(), 0, 1, 0, buffer.as_mut_ptr(), buffer.len());
        assert!(null.is_null());
        assert_eq!(CStr::from_ptr(buffer.as_ptr()).to_bytes(), b"the sce");
        let bytes = [0xffu8, 0xfe];
        let mut message = [0 as c_char; 64];
        let not_utf8 = h3_env_create_scenario(
            bytes.as_ptr().cast(),
            bytes.len(),
            1,
            0,
            message.as_mut_ptr(),
            message.len(),
        );
        assert!(not_utf8.is_null());
        assert!(
            CStr::from_ptr(message.as_ptr())
                .to_bytes()
                .starts_with(b"the scenario text is not UTF-8")
        );
        // Without a buffer nothing is written.
        assert!(
            h3_env_create_scenario(std::ptr::null(), 0, 1, 0, std::ptr::null_mut(), 8).is_null()
        );
        let mut untouched = [b'x' as c_char; 4];
        assert!(
            h3_env_create_scenario(std::ptr::null(), 0, 1, 0, untouched.as_mut_ptr(), 0).is_null()
        );
        assert_eq!(untouched, [b'x' as c_char; 4]);
        // On success the buffer is left as it was.
        let env = h3_env_create_scenario(
            FIELD.as_ptr().cast(),
            FIELD.len(),
            1,
            0,
            untouched.as_mut_ptr(),
            untouched.len(),
        );
        assert!(!env.is_null());
        assert_eq!(untouched, [b'x' as c_char; 4]);
        h3_env_destroy(env);
    }
}

#[test]
fn a_message_is_cut_at_a_character_boundary() {
    let mut buffer = [b'x' as c_char; 3];
    // SAFETY: `buffer` is valid for its length.
    unsafe {
        // "é" takes two bytes: with room for two, only the "a" fits before the NUL.
        write_error("aé", buffer.as_mut_ptr(), buffer.len());
        assert_eq!(CStr::from_ptr(buffer.as_ptr()).to_bytes(), b"a");
        write_error("é", buffer.as_mut_ptr(), 1);
        assert_eq!(buffer[0], 0);
    }
}
