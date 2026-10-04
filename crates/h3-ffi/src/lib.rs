//! Adapter: the stable C ABI over the simulation environment (ADR-0011).
//!
//! Conventions: opaque handles created by `h3_env_create_*` and released by `h3_env_destroy`;
//! functions return `H3_OK` (0) or a negative error code and never unwind across the boundary
//! (panics are caught and reported as `H3_ERR_PANIC`); sides are `0` (attacker) and `1`
//! (defender); action and observation encodings are those of `h3-sim`, versioned by
//! `h3_abi_version`. The C declarations live in `include/h3.h`.
//!
//! Server-mode note (ADR-0008): this ABI is for trusted local consumers (training, tools). It
//! exposes the full-state hash; a server must not forward it to untrusted clients.
#![allow(unsafe_code)]

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;

use h3_battle::LegalCommands;
use h3_contracts::{BattleResult, BattleSide};
use h3_sim::{ACTION_COUNT, ActionMask, BattleEnv, OBSERVATION_LEN, StepError};

/// Version of this ABI (functions, structs, error codes, and the action/observation encodings).
///
/// v2 (E03 T9): `H3StepResult.result` replaces `winner` (draws, `H3_RESULT_*`); action encoding
/// v4 and observation encoding v3 (v1 had the E01 encodings v1). v3 (E04a T7b): action encoding
/// v5 (heals appended; `h3_action_count` grows by the stack slots, earlier indices unchanged).
/// v4 (E04a T8): action encoding v6 (`EndTactics` appended; `h3_action_count` grows by one).
/// v5 (E04a T9a): action encoding v7 (Retreat appended) and the `H3_RESULT_*` codes of a retreat
/// or a surrender. v6 (E04a T9b): action encoding v8 (Surrender appended). v7 (E04a T11b):
/// observation encoding v4 (the heroes, the tactics phase, retreat and surrender;
/// `h3_observation_len` grows).
pub const H3_ABI_VERSION: u32 = 7;

/// `H3StepResult.result`: the battle is not decided (still running, or truncated).
pub const H3_RESULT_NONE: i32 = -1;
/// `H3StepResult.result`: the attacker won.
pub const H3_RESULT_ATTACKER: i32 = 0;
/// `H3StepResult.result`: the defender won.
pub const H3_RESULT_DEFENDER: i32 = 1;
/// `H3StepResult.result`: a draw (both armies wiped out at once).
pub const H3_RESULT_DRAW: i32 = 2;
/// `H3StepResult.result`: the attacker retreated (the defender wins).
pub const H3_RESULT_ATTACKER_RETREATED: i32 = 3;
/// `H3StepResult.result`: the defender retreated (the attacker wins).
pub const H3_RESULT_DEFENDER_RETREATED: i32 = 4;
/// `H3StepResult.result`: the attacker surrendered (the defender wins).
pub const H3_RESULT_ATTACKER_SURRENDERED: i32 = 5;
/// `H3StepResult.result`: the defender surrendered (the attacker wins).
pub const H3_RESULT_DEFENDER_SURRENDERED: i32 = 6;

/// Success.
pub const H3_OK: i32 = 0;
/// A required pointer was null.
pub const H3_ERR_NULL: i32 = -1;
/// An argument is out of range (side, scenario, buffer length).
pub const H3_ERR_ARGUMENT: i32 = -2;
/// The action index is not a valid action.
pub const H3_ERR_INVALID_ACTION: i32 = -3;
/// The action was rejected by the rules (not legal now, or not this side's turn).
pub const H3_ERR_REJECTED: i32 = -4;
/// The episode is over; reset first.
pub const H3_ERR_EPISODE_OVER: i32 = -5;
/// A panic was caught (a bug; the handle should be destroyed).
pub const H3_ERR_PANIC: i32 = -99;

/// Result of `h3_env_step`.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct H3StepResult {
    /// 1 if the episode is over.
    pub done: u8,
    /// 1 if the episode was truncated by the step limit.
    pub truncated: u8,
    /// How the battle ended: `H3_RESULT_ATTACKER`, `H3_RESULT_DEFENDER`, `H3_RESULT_DRAW`, a
    /// side's retreat or surrender (`H3_RESULT_*_RETREATED`, `H3_RESULT_*_SURRENDERED`), or
    /// `H3_RESULT_NONE` while it is not decided.
    pub result: i32,
    /// Reward of the attacker.
    pub reward_attacker: f32,
    /// Reward of the defender.
    pub reward_defender: f32,
}

/// Opaque environment handle.
#[derive(Debug)]
pub struct H3Env {
    env: BattleEnv,
    legal: LegalCommands,
    mask: ActionMask,
}

fn side(value: i32) -> Option<BattleSide> {
    match value {
        0 => Some(BattleSide::Attacker),
        1 => Some(BattleSide::Defender),
        _ => None,
    }
}

fn side_code(value: Option<BattleSide>) -> i32 {
    match value {
        Some(BattleSide::Attacker) => 0,
        Some(BattleSide::Defender) => 1,
        None => -1,
    }
}

fn result_code(value: Option<BattleResult>) -> i32 {
    match value {
        None => H3_RESULT_NONE,
        Some(BattleResult::Victory(BattleSide::Attacker)) => H3_RESULT_ATTACKER,
        Some(BattleResult::Victory(BattleSide::Defender)) => H3_RESULT_DEFENDER,
        Some(BattleResult::Draw) => H3_RESULT_DRAW,
        Some(BattleResult::Retreat(BattleSide::Attacker)) => H3_RESULT_ATTACKER_RETREATED,
        Some(BattleResult::Retreat(BattleSide::Defender)) => H3_RESULT_DEFENDER_RETREATED,
        Some(BattleResult::Surrender(BattleSide::Attacker)) => H3_RESULT_ATTACKER_SURRENDERED,
        Some(BattleResult::Surrender(BattleSide::Defender)) => H3_RESULT_DEFENDER_SURRENDERED,
    }
}

fn guard(f: impl FnOnce() -> i32) -> i32 {
    catch_unwind(AssertUnwindSafe(f)).unwrap_or(H3_ERR_PANIC)
}

/// The ABI version ([`H3_ABI_VERSION`]).
#[unsafe(no_mangle)]
pub extern "C" fn h3_abi_version() -> u32 {
    H3_ABI_VERSION
}

/// Number of actions in the action encoding.
#[unsafe(no_mangle)]
pub extern "C" fn h3_action_count() -> u32 {
    ACTION_COUNT as u32
}

/// Length of the observation encoding (number of `f32`).
#[unsafe(no_mangle)]
pub extern "C" fn h3_observation_len() -> u32 {
    OBSERVATION_LEN as u32
}

/// Creates an environment on the built-in synthetic content pack. `scenario`: 0 = duel (obstacles
/// generated for each episode from its seed, BAT-GEO-004), 1 = skirmish, 2 = full armies (both
/// led by heroes). `max_steps` = 0 means no step limit. Returns null on error.
#[unsafe(no_mangle)]
pub extern "C" fn h3_env_create_synthetic(scenario: u32, seed: u64, max_steps: u32) -> *mut H3Env {
    let created = catch_unwind(|| {
        let text = match scenario {
            0 => h3_pack::SYNTHETIC_DUEL,
            1 => h3_pack::SYNTHETIC_SKIRMISH,
            2 => h3_pack::SYNTHETIC_FULL,
            _ => return None,
        };
        let random_obstacles = h3_pack::parse_scenario(text).ok()?.random_obstacles;
        let (ruleset, setup) = h3_pack::synthetic(text).ok()?;
        let (ruleset, setup) = (Arc::new(ruleset), Arc::new(setup));
        let env = if random_obstacles {
            BattleEnv::with_random_obstacles(ruleset, setup, seed, max_steps)
        } else {
            BattleEnv::new(ruleset, setup, seed, max_steps)
        }
        .ok()?;
        Some(Box::new(H3Env {
            env,
            legal: LegalCommands::new(),
            mask: [false; ACTION_COUNT],
        }))
    });
    match created {
        Ok(Some(b)) => Box::into_raw(b),
        _ => std::ptr::null_mut(),
    }
}

/// Destroys an environment. Null is ignored.
///
/// # Safety
/// `env` must be null or a handle returned by this library that has not been destroyed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn h3_env_destroy(env: *mut H3Env) {
    if !env.is_null() {
        // SAFETY: per the contract, `env` came from `Box::into_raw` and is destroyed only once.
        drop(unsafe { Box::from_raw(env) });
    }
}

/// Forks (clones) an environment. Returns null if `env` is null.
///
/// # Safety
/// `env` must be null or a live handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn h3_env_fork(env: *const H3Env) -> *mut H3Env {
    // SAFETY: per the contract, a non-null `env` is a live handle.
    let Some(e) = (unsafe { env.as_ref() }) else {
        return std::ptr::null_mut();
    };
    let forked = catch_unwind(AssertUnwindSafe(|| {
        Box::new(H3Env {
            env: e.env.fork(),
            legal: LegalCommands::new(),
            mask: [false; ACTION_COUNT],
        })
    }));
    forked.map_or(std::ptr::null_mut(), Box::into_raw)
}

/// Restarts the battle with a new seed.
///
/// # Safety
/// `env` must be null or a live handle not used concurrently.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn h3_env_reset(env: *mut H3Env, seed: u64) -> i32 {
    // SAFETY: per the contract.
    let Some(e) = (unsafe { env.as_mut() }) else {
        return H3_ERR_NULL;
    };
    guard(|| match e.env.reset(seed) {
        Ok(()) => H3_OK,
        Err(_) => H3_ERR_ARGUMENT,
    })
}

/// The side to act (0/1), or -1 if the episode is over or `env` is null.
///
/// # Safety
/// `env` must be null or a live handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn h3_env_to_move(env: *const H3Env) -> i32 {
    // SAFETY: per the contract.
    let Some(e) = (unsafe { env.as_ref() }) else {
        return -1;
    };
    side_code(e.env.to_move())
}

/// Writes the legal-action mask of `actor` (0/1 bytes) into `out[0..len]`; `len` must be at
/// least `h3_action_count()`.
///
/// # Safety
/// `env` must be null or a live handle not used concurrently; `out` must be null or valid for
/// `len` bytes of writes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn h3_env_legal_mask(
    env: *mut H3Env,
    actor: i32,
    out: *mut u8,
    len: usize,
) -> i32 {
    // SAFETY: per the contract.
    let Some(e) = (unsafe { env.as_mut() }) else {
        return H3_ERR_NULL;
    };
    if out.is_null() {
        return H3_ERR_NULL;
    }
    let (Some(actor), true) = (side(actor), len >= ACTION_COUNT) else {
        return H3_ERR_ARGUMENT;
    };
    // SAFETY: `out` is valid for `len >= ACTION_COUNT` bytes per the contract.
    let out = unsafe { std::slice::from_raw_parts_mut(out, ACTION_COUNT) };
    guard(|| {
        let H3Env { env, legal, mask } = e;
        env.legal_mask(actor, legal, mask);
        for (o, m) in out.iter_mut().zip(mask.iter()) {
            *o = u8::from(*m);
        }
        H3_OK
    })
}

/// Writes the observation for `viewer` into `out[0..len]`; `len` must be at least
/// `h3_observation_len()`.
///
/// # Safety
/// `env` must be null or a live handle; `out` must be null or valid for `len` `f32` writes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn h3_env_observe(
    env: *const H3Env,
    viewer: i32,
    out: *mut f32,
    len: usize,
) -> i32 {
    // SAFETY: per the contract.
    let Some(e) = (unsafe { env.as_ref() }) else {
        return H3_ERR_NULL;
    };
    if out.is_null() {
        return H3_ERR_NULL;
    }
    let (Some(viewer), true) = (side(viewer), len >= OBSERVATION_LEN) else {
        return H3_ERR_ARGUMENT;
    };
    // SAFETY: `out` is valid for `len >= OBSERVATION_LEN` floats per the contract.
    let out = unsafe { std::slice::from_raw_parts_mut(out, OBSERVATION_LEN) };
    guard(|| match e.env.encode_observation(viewer, out) {
        Ok(()) => H3_OK,
        Err(_) => H3_ERR_ARGUMENT,
    })
}

/// Plays `action` for `actor` and writes the result to `out` (may be null).
///
/// # Safety
/// `env` must be null or a live handle not used concurrently; `out` must be null or valid for
/// one `H3StepResult` write.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn h3_env_step(
    env: *mut H3Env,
    actor: i32,
    action: u32,
    out: *mut H3StepResult,
) -> i32 {
    // SAFETY: per the contract.
    let Some(e) = (unsafe { env.as_mut() }) else {
        return H3_ERR_NULL;
    };
    let Some(actor) = side(actor) else {
        return H3_ERR_ARGUMENT;
    };
    guard(|| match e.env.step(actor, action as usize) {
        Ok(r) => {
            let result = H3StepResult {
                done: u8::from(r.done),
                truncated: u8::from(r.truncated),
                result: result_code(r.result),
                reward_attacker: r.reward[0],
                reward_defender: r.reward[1],
            };
            // SAFETY: a non-null `out` is valid for one write per the contract.
            if let Some(slot) = unsafe { out.as_mut() } {
                *slot = result;
            }
            H3_OK
        }
        Err(StepError::InvalidAction) => H3_ERR_INVALID_ACTION,
        Err(StepError::Rejected(_)) => H3_ERR_REJECTED,
        Err(StepError::EpisodeOver) => H3_ERR_EPISODE_OVER,
    })
}

/// Canonical hash of the battle state (0 if `env` is null).
///
/// # Safety
/// `env` must be null or a live handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn h3_env_state_hash(env: *const H3Env) -> u64 {
    // SAFETY: per the contract.
    unsafe { env.as_ref() }.map_or(0, |e| e.env.state_hash())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn abi_smoke_create_step_destroy() {
        assert_eq!(h3_abi_version(), 7);
        assert_eq!(h3_action_count() as usize, ACTION_COUNT);
        assert_eq!(h3_observation_len() as usize, OBSERVATION_LEN);
        assert!(h3_env_create_synthetic(9, 1, 0).is_null());
        let env = h3_env_create_synthetic(1, 42, 500);
        assert!(!env.is_null());
        unsafe {
            let mut mask = vec![0u8; ACTION_COUNT];
            let mut obs = vec![0f32; OBSERVATION_LEN];
            let mut result = H3StepResult::default();
            let mut steps = 0;
            while h3_env_to_move(env) >= 0 {
                let side = h3_env_to_move(env);
                assert_eq!(
                    h3_env_legal_mask(env, side, mask.as_mut_ptr(), mask.len()),
                    H3_OK
                );
                assert_eq!(
                    h3_env_observe(env, side, obs.as_mut_ptr(), obs.len()),
                    H3_OK
                );
                // Prefer attacks (indices ≥ 257; the skirmish has no heroes, so no Retreat), else
                // the first legal action.
                let action = (257..ACTION_COUNT)
                    .find(|&i| mask[i] == 1)
                    .or_else(|| mask.iter().position(|m| *m == 1));
                assert_eq!(
                    h3_env_step(env, side, action.unwrap() as u32, &mut result),
                    H3_OK
                );
                steps += 1;
            }
            assert!(steps > 0);
            assert_eq!(result.done, 1);
            assert!(result.result >= 0 || result.truncated == 1);
            assert_eq!(h3_env_step(env, 0, 1, &mut result), H3_ERR_EPISODE_OVER);
            let hash = h3_env_state_hash(env);
            let fork = h3_env_fork(env);
            assert_eq!(h3_env_state_hash(fork), hash);
            assert_eq!(h3_env_reset(env, 42), H3_OK);
            assert_ne!(h3_env_state_hash(env), hash);
            h3_env_destroy(fork);
            h3_env_destroy(env);
        }
    }

    #[test]
    fn errors_are_codes_not_panics() {
        unsafe {
            let null = std::ptr::null_mut();
            let mut byte = 0u8;
            let mut float = 0f32;
            assert_eq!(h3_env_reset(null, 1), H3_ERR_NULL);
            assert_eq!(h3_env_to_move(null), -1);
            assert_eq!(h3_env_legal_mask(null, 0, &mut byte, 1), H3_ERR_NULL);
            assert_eq!(h3_env_observe(null, 0, &mut float, 1), H3_ERR_NULL);
            assert_eq!(h3_env_step(null, 0, 0, std::ptr::null_mut()), H3_ERR_NULL);
            assert_eq!(h3_env_state_hash(null), 0);
            assert!(h3_env_fork(null).is_null());
            h3_env_destroy(null);

            let env = h3_env_create_synthetic(0, 1, 0);
            assert_eq!(
                h3_env_legal_mask(env, 0, std::ptr::null_mut(), 0),
                H3_ERR_NULL
            );
            assert_eq!(
                h3_env_legal_mask(env, 2, &mut byte, ACTION_COUNT),
                H3_ERR_ARGUMENT
            );
            assert_eq!(h3_env_legal_mask(env, 0, &mut byte, 1), H3_ERR_ARGUMENT);
            assert_eq!(h3_env_observe(env, 0, std::ptr::null_mut(), 0), H3_ERR_NULL);
            assert_eq!(h3_env_observe(env, 0, &mut float, 1), H3_ERR_ARGUMENT);
            assert_eq!(
                h3_env_step(env, 5, 0, std::ptr::null_mut()),
                H3_ERR_ARGUMENT
            );
            assert_eq!(
                h3_env_step(env, 0, u32::MAX, std::ptr::null_mut()),
                H3_ERR_INVALID_ACTION
            );
            let other = 1 - h3_env_to_move(env);
            assert_eq!(
                h3_env_step(env, other, 1, std::ptr::null_mut()),
                H3_ERR_REJECTED
            );
            assert_eq!(h3_env_step(env, 1 - other, 1, std::ptr::null_mut()), H3_OK);
            h3_env_destroy(env);
        }
        assert_eq!(guard(|| panic!("boom")), H3_ERR_PANIC);
    }

    #[test]
    fn result_codes() {
        assert_eq!(result_code(None), H3_RESULT_NONE);
        assert_eq!(
            result_code(Some(BattleResult::Victory(BattleSide::Attacker))),
            H3_RESULT_ATTACKER
        );
        assert_eq!(
            result_code(Some(BattleResult::Victory(BattleSide::Defender))),
            H3_RESULT_DEFENDER
        );
        assert_eq!(result_code(Some(BattleResult::Draw)), H3_RESULT_DRAW);
        let codes = BattleSide::BOTH.map(|side| {
            [
                result_code(Some(BattleResult::Retreat(side))),
                result_code(Some(BattleResult::Surrender(side))),
            ]
        });
        assert_eq!(codes, [[3, 5], [4, 6]]);
    }
}
