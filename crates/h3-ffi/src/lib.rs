//! Adapter: the stable C ABI over the simulation environment (ADR-0011).
//!
//! Conventions: opaque handles created by `h3_env_create_*` and released by `h3_env_destroy`
//! (a training batch's by `h3_batch_create` and `h3_batch_destroy`, module `batch`);
//! functions return `H3_OK` (0) or a negative error code and never unwind across the boundary
//! (panics are caught and reported as `H3_ERR_PANIC`); sides are `0` (attacker) and `1`
//! (defender); action and observation encodings are those of `h3-sim`, versioned by
//! `h3_abi_version`. The C declarations live in `include/h3.h`.
//!
//! Server-mode note (ADR-0008): this ABI is for trusted local consumers (training, tools). It
//! exposes the full state (the fork, its hash) and the episode seeds, which decide a battle's
//! draws: trusted tooling (`h3.h`), never an agent's input; a server must not forward them to
//! untrusted clients.
#![allow(unsafe_code)]

use std::ffi::c_char;
use std::panic::{AssertUnwindSafe, catch_unwind};

use h3_battle::LegalCommands;
use h3_contracts::{BattleResult, BattleSide};
use h3_sim::{
    ACTION_COUNT, Action, ActionMask, BattleEnv, OBSERVATION_LEN, SPELL_COUNT, SpellMask,
    StepError, TARGET_COUNT, TargetMask,
};

mod batch;
mod scenario;

pub use batch::{
    H3_CURRENCY_AI_VALUE, H3_CURRENCY_GOLD, H3_ENCOUNTER_GUARDS, H3_ENCOUNTER_HERO,
    H3_ENCOUNTER_NONE, H3_ENCOUNTER_SIEGE_GARRISON, H3_ENCOUNTER_SIEGE_HERO,
    H3_ENCOUNTER_SIEGE_NEUTRAL, H3_ENCOUNTER_WANDERING, H3_ERR_NOT_READY, H3_SEAT_AGGRESSOR,
    H3_SEAT_EXTERNAL, H3_SEAT_GREEDY, H3_SEAT_RANDOM, H3Batch, H3BatchBuffers, H3BatchConfig,
    H3RewardConfig, H3Seats, h3_batch_add_encounters, h3_batch_add_scenario, h3_batch_create,
    h3_batch_destroy, h3_batch_episode_seed, h3_batch_reset, h3_batch_second_mask,
    h3_batch_spell_mask, h3_batch_step, h3_batch_target_mask,
};

/// Version of this ABI (functions, structs, error codes, and the action/observation encodings).
///
/// v2 (E03 T9): `H3StepResult.result` replaces `winner` (draws, `H3_RESULT_*`); action encoding
/// v4 and observation encoding v3 (v1 had the E01 encodings v1). v3 (E04a T7b): action encoding
/// v5 (heals appended; `h3_action_count` grows by the stack slots, earlier indices unchanged).
/// v4 (E04a T8): action encoding v6 (`EndTactics` appended; `h3_action_count` grows by one).
/// v5 (E04a T9a): action encoding v7 (Retreat appended) and the `H3_RESULT_*` codes of a retreat
/// or a surrender. v6 (E04a T9b): action encoding v8 (Surrender appended). v7 (E04a T11b):
/// observation encoding v4 (the heroes, the tactics phase, retreat and surrender;
/// `h3_observation_len` grows). v8 (E04b T2c): action encoding v9 and observation encoding v5:
/// an action is an [`H3Action`] (an entry and the cast's spell and target heads), served head by
/// head by `h3_env_legal_mask`, `h3_env_spell_mask`, `h3_env_target_mask` and
/// `h3_env_second_mask` (ADR-0014 point 9); `h3_env_step` takes an `H3Action`; the stack slots
/// grow to 42. v9 (E04b T3a): observation encoding v6 (the timed effects of stacks;
/// `h3_observation_len` grows). v10 (E04b T5a): action encoding v10 (the second target head
/// carries Teleport's destination hex); no function changes. v11 (E04b T6b): observation
/// encoding v7 (created stacks and clones; `h3_observation_len` grows). v12 (E04b T7b1): action
/// encoding v11 (the stack's cast, entry 1395, becomes legal; its spell head allows the stack's
/// spell, its second target head 0) and observation encoding v8 (casts left;
/// `h3_observation_len` grows). v13 (E04b T7b2a): observation encoding v9 (a global: the spell
/// the active stack drew, for its controller; `h3_observation_len` grows); the stack's cast's
/// target head may carry hexes (a Faerie Dragon's area spells); no function changes. v14 (E04b
/// T7b2b): observation encoding v10 (the hero's spell book flag, each stack's count of the spells a
/// helpful caster's draw could pick; `h3_observation_len` grows); the stack's cast's spell head
/// holds 0 for the Master Genie, whose spell is drawn at the cast; no function changes. v15 (E04b
/// T7b4): observation encoding v11 (two globals: the round starts since each side's periodic
/// casters cast; `h3_observation_len` grows); no function changes. v16 (E04b T8a): observation
/// encoding v12 (the spell book's features carry the masteries, the school masteries are gone;
/// `h3_observation_len` shrinks); no function changes. v17 (E04b T9a): observation encoding v13
/// (complete: the battlefield planes, the hero's private part once with its hidden magic, the
/// new globals and per-stack features; `h3_observation_len` grows); no function changes. v18:
/// action encoding v12 and observation encoding v13 → v14 (a two-hex stack's position is its rear
/// hex, BAT-GEO-002); no function changes. v19 (E04c T2b): observation encoding v15 (two
/// globals: the special battlefield, the anti-magic garrison; `h3_observation_len` grows); no
/// function changes. v20 (E04c T5): observation encoding v16 (the town's four globals and two
/// battlefield planes; `h3_observation_len` grows); no function changes. v21 (E04c T6): action
/// encoding v13 (eight wall-shot entries appended; `h3_action_count` grows) and observation
/// encoding v17 (the wall-shot targets' globals, the heroes' Catapult features;
/// `h3_observation_len` grows); no function changes. v22 (E04c T7): observation encoding v18 (the
/// turret flag per stack; `h3_observation_len` grows); no function changes. v23 (E04c T7d):
/// observation encoding v19 (the game's difficulty and the human players as three globals;
/// `h3_observation_len` grows); no function changes. v24 (E04c T7e): observation encoding v20 (the
/// own hero's Artillery level; `h3_observation_len` grows); no function changes. v25 (E04c T10b):
/// observation encoding v21 (at a town the defender's option to retreat shows to the defender
/// only, BAT-Q50 (4); the layout unchanged); no function changes. v26 (E05 T2):
/// `h3_env_create_scenario` (an environment from a scenario's text on the built-in packs: any
/// battle the engine plays, sieges included; ADR-0025 point 9); the encodings unchanged. v27 (E05
/// T3d): the training batch, `h3_batch_*` (N envs per handle on T threads, seats per scenario,
/// same-step automatic reset, the head calls per batch, reward v1, the caller's arrays in
/// `H3BatchBuffers`; ADR-0025 points 1 to 8) and `H3_ERR_NOT_READY`; the encodings and the single
/// env's functions unchanged. v28 (E05 T5b2): `h3_batch_add_encounters` (the encounter
/// distribution v1 as an episode source) and `H3BatchBuffers.encounter` (each env's encounter
/// kind, `H3_ENCOUNTER_*`; the struct grows by one pointer at its end); the encodings unchanged.
/// v29 (E05 T8b): `h3_batch_add_encounters` takes `const H3Seats *seats` after `kinds` (null:
/// each encounter's own seats; else these for every episode of the entry); the encodings
/// unchanged. v30 (E05 T7): the seats `H3_SEAT_AGGRESSOR` and `H3_SEAT_GREEDY`, the baseline
/// bots inside the call (`h3_sim::bots`); the encodings unchanged.
pub const H3_ABI_VERSION: u32 = 30;

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
/// The action of the side to move is not a valid action.
pub const H3_ERR_INVALID_ACTION: i32 = -3;
/// The action was rejected by the rules: not legal now, or not this side's turn (then whatever
/// the action is: it is not decoded).
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

/// An action (`h3_sim::Action`): an entry of the action encoding and, for a cast, the spell and
/// target heads (0 for every other entry).
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct H3Action {
    /// The entry (`h3_action_count()` entries).
    pub entry: u32,
    /// A cast's spell (`h3_spell_count()` values).
    pub spell: u16,
    /// A cast's target (`h3_target_count()` values: 0 none, `1 + h` hex, `256 + s` stack slot).
    pub target: u16,
    /// A cast's second target (same layout: Teleport's destination hex; 0 for every other
    /// spell).
    pub second: u16,
}

impl From<H3Action> for Action {
    fn from(a: H3Action) -> Self {
        Self {
            entry: a.entry,
            spell: a.spell,
            target: a.target,
            second: a.second,
        }
    }
}

/// Opaque environment handle.
#[derive(Debug)]
pub struct H3Env {
    env: BattleEnv,
    legal: LegalCommands,
    mask: ActionMask,
    spells: SpellMask,
    targets: TargetMask,
}

impl H3Env {
    fn new(env: BattleEnv) -> Self {
        Self {
            env,
            legal: LegalCommands::new(),
            mask: [false; ACTION_COUNT],
            spells: [false; SPELL_COUNT],
            targets: [false; TARGET_COUNT],
        }
    }
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

/// Number of values of the spell head (`H3Action.spell`).
#[unsafe(no_mangle)]
pub extern "C" fn h3_spell_count() -> u32 {
    SPELL_COUNT as u32
}

/// Number of values of each target head (`H3Action.target`, `H3Action.second`).
#[unsafe(no_mangle)]
pub extern "C" fn h3_target_count() -> u32 {
    TARGET_COUNT as u32
}

/// Length of the observation encoding (number of `f32`).
#[unsafe(no_mangle)]
pub extern "C" fn h3_observation_len() -> u32 {
    OBSERVATION_LEN as u32
}

/// Creates an environment on the built-in synthetic content pack. `scenario`: 0 = duel (obstacles
/// generated for each episode from its seed, BAT-GEO-004), 1 = skirmish, 2 = full armies (both
/// led by heroes). `max_steps` = 0 means no step limit. Returns null on error. Any other battle:
/// [`h3_env_create_scenario`].
#[unsafe(no_mangle)]
pub extern "C" fn h3_env_create_synthetic(scenario: u32, seed: u64, max_steps: u32) -> *mut H3Env {
    let text = match scenario {
        0 => h3_pack_battle::SYNTHETIC_DUEL,
        1 => h3_pack_battle::SYNTHETIC_SKIRMISH,
        2 => h3_pack_battle::SYNTHETIC_FULL,
        _ => return std::ptr::null_mut(),
    };
    catch_unwind(|| scenario::scenario_env(text, seed, max_steps))
        .ok()
        .and_then(Result::ok)
        .map_or(std::ptr::null_mut(), handle)
}

/// Creates an environment from a scenario: `toml[0..len]` is the text of a scenario file (UTF-8,
/// not NUL-terminated; the battle lane's format, `packs/synthetic/scenarios/`), on the built-in
/// packs it names by id (`synthetic`, `synthetic-ext`, `base`, `ab`, `sod`). Any battle the
/// engine plays: a field, a special battlefield, a garrison, a town under siege. Obstacles are
/// generated for each episode from its seed where the scenario asks for random ones
/// (BAT-GEO-004); the scenario's `difficulty` and `human` apply as written. `max_steps` = 0 means
/// no step limit. Returns null on error and writes its message into `error[0..error_len]` as a
/// NUL-terminated string cut to fit (nothing if `error` is null or `error_len` is 0); on success
/// `error` is left untouched. Each call compiles the scenario's packs.
///
/// # Safety
/// `toml` must be null or valid for `len` bytes of reads; `error` must be null or valid for
/// `error_len` bytes of writes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn h3_env_create_scenario(
    toml: *const c_char,
    len: usize,
    seed: u64,
    max_steps: u32,
    error: *mut c_char,
    error_len: usize,
) -> *mut H3Env {
    let created = catch_unwind(|| {
        if toml.is_null() {
            return Err("the scenario text is null".to_owned());
        }
        // SAFETY: a non-null `toml` is valid for `len` bytes of reads per the contract.
        let bytes = unsafe { std::slice::from_raw_parts(toml.cast::<u8>(), len) };
        let text = std::str::from_utf8(bytes)
            .map_err(|e| format!("the scenario text is not UTF-8: {e}"))?;
        scenario::scenario_env(text, seed, max_steps)
    })
    .unwrap_or(Err(PANIC_MESSAGE.to_owned()));
    match created {
        Ok(env) => handle(env),
        Err(message) => {
            // SAFETY: per the contract.
            unsafe { scenario::write_error(&message, error, error_len) };
            std::ptr::null_mut()
        }
    }
}

/// The message of a caught panic.
const PANIC_MESSAGE: &str = "a panic was caught (a bug)";

/// A new handle owning `env`.
fn handle(env: BattleEnv) -> *mut H3Env {
    Box::into_raw(Box::new(H3Env::new(env)))
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
    let forked = catch_unwind(AssertUnwindSafe(|| Box::new(H3Env::new(e.env.fork()))));
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

/// Converts `bools` into 0/1 bytes in `out`.
fn write_mask(bools: &[bool], out: &mut [u8]) {
    for (o, m) in out.iter_mut().zip(bools) {
        *o = u8::from(*m);
    }
}

/// Checks the arguments shared by the mask functions and returns the actor and the first `n`
/// bytes of `out`.
///
/// # Safety
/// `out` must be null or valid for `len` bytes of writes.
unsafe fn mask_args<'a>(
    actor: i32,
    out: *mut u8,
    len: usize,
    n: usize,
) -> Result<(BattleSide, &'a mut [u8]), i32> {
    if out.is_null() {
        return Err(H3_ERR_NULL);
    }
    let (Some(actor), true) = (side(actor), len >= n) else {
        return Err(H3_ERR_ARGUMENT);
    };
    // SAFETY: `out` is valid for `len >= n` bytes per the contract.
    Ok((actor, unsafe { std::slice::from_raw_parts_mut(out, n) }))
}

/// Writes the mask of `actor`'s legal entries (0/1 bytes) into `out[0..len]`; `len` must be at
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
    // SAFETY: per the contract.
    let (actor, out) = match unsafe { mask_args(actor, out, len, ACTION_COUNT) } {
        Ok(args) => args,
        Err(code) => return code,
    };
    guard(|| {
        e.env.legal_mask(actor, &mut e.legal, &mut e.mask);
        write_mask(&e.mask, out);
        H3_OK
    })
}

/// Writes the mask of `actor`'s legal spells after choosing `entry` into `out[0..len]`; `len`
/// must be at least `h3_spell_count()`.
///
/// # Safety
/// As [`h3_env_legal_mask`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn h3_env_spell_mask(
    env: *mut H3Env,
    actor: i32,
    entry: u32,
    out: *mut u8,
    len: usize,
) -> i32 {
    // SAFETY: per the contract.
    let Some(e) = (unsafe { env.as_mut() }) else {
        return H3_ERR_NULL;
    };
    // SAFETY: per the contract.
    let (actor, out) = match unsafe { mask_args(actor, out, len, SPELL_COUNT) } {
        Ok(args) => args,
        Err(code) => return code,
    };
    guard(|| {
        e.env
            .spell_mask(actor, entry as usize, &mut e.legal, &mut e.spells);
        write_mask(&e.spells, out);
        H3_OK
    })
}

/// Writes the mask of `actor`'s legal targets after choosing `entry` and `spell` into
/// `out[0..len]`; `len` must be at least `h3_target_count()`.
///
/// # Safety
/// As [`h3_env_legal_mask`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn h3_env_target_mask(
    env: *mut H3Env,
    actor: i32,
    entry: u32,
    spell: u16,
    out: *mut u8,
    len: usize,
) -> i32 {
    // SAFETY: per the contract.
    let Some(e) = (unsafe { env.as_mut() }) else {
        return H3_ERR_NULL;
    };
    // SAFETY: per the contract.
    let (actor, out) = match unsafe { mask_args(actor, out, len, TARGET_COUNT) } {
        Ok(args) => args,
        Err(code) => return code,
    };
    guard(|| {
        e.env
            .target_mask(actor, (entry as usize, spell), &mut e.legal, &mut e.targets);
        write_mask(&e.targets, out);
        H3_OK
    })
}

/// Writes the mask of `actor`'s legal second targets after choosing `entry`, `spell` and
/// `target` into `out[0..len]`; `len` must be at least `h3_target_count()`.
///
/// # Safety
/// As [`h3_env_legal_mask`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn h3_env_second_mask(
    env: *mut H3Env,
    actor: i32,
    entry: u32,
    spell: u16,
    target: u16,
    out: *mut u8,
    len: usize,
) -> i32 {
    // SAFETY: per the contract.
    let Some(e) = (unsafe { env.as_mut() }) else {
        return H3_ERR_NULL;
    };
    // SAFETY: per the contract.
    let (actor, out) = match unsafe { mask_args(actor, out, len, TARGET_COUNT) } {
        Ok(args) => args,
        Err(code) => return code,
    };
    guard(|| {
        e.env.second_mask(
            actor,
            (entry as usize, spell, target),
            &mut e.legal,
            &mut e.targets,
        );
        write_mask(&e.targets, out);
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
    action: H3Action,
    out: *mut H3StepResult,
) -> i32 {
    // SAFETY: per the contract.
    let Some(e) = (unsafe { env.as_mut() }) else {
        return H3_ERR_NULL;
    };
    let Some(actor) = side(actor) else {
        return H3_ERR_ARGUMENT;
    };
    guard(|| match e.env.step(actor, action.into()) {
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

    /// The action of a non-cast entry.
    fn entry(entry: usize) -> H3Action {
        H3Action {
            entry: entry as u32,
            ..H3Action::default()
        }
    }

    #[test]
    fn abi_smoke_create_step_destroy() {
        assert_eq!(h3_abi_version(), 30);
        assert_eq!(h3_action_count() as usize, ACTION_COUNT);
        assert_eq!(h3_spell_count() as usize, SPELL_COUNT);
        assert_eq!(h3_target_count() as usize, TARGET_COUNT);
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
                // Prefer attacks (entries from 257 up to the casts; the skirmish has no heroes, so
                // no Retreat and no hero's cast), else the first legal entry.
                let action = (257..h3_sim::action::HERO_CAST)
                    .find(|&i| mask[i] == 1)
                    .or_else(|| mask.iter().position(|m| *m == 1));
                assert_eq!(
                    h3_env_step(env, side, entry(action.unwrap()), &mut result),
                    H3_OK
                );
                steps += 1;
            }
            assert!(steps > 0);
            assert_eq!(result.done, 1);
            assert!(result.result >= 0 || result.truncated == 1);
            assert_eq!(
                h3_env_step(env, 0, entry(1), &mut result),
                H3_ERR_EPISODE_OVER
            );
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
    fn a_hero_casts_head_by_head() {
        // The full scenario's heroes have spell books (E04b T2c): play until a hero may cast,
        // then choose the cast's heads from their masks.
        let env = h3_env_create_synthetic(2, 3, 0);
        let mut mask = vec![0u8; ACTION_COUNT];
        let mut spells = vec![0u8; SPELL_COUNT];
        let mut targets = vec![0u8; TARGET_COUNT];
        let mut seconds = vec![0u8; TARGET_COUNT];
        let mut casts = 0;
        unsafe {
            while casts == 0 && h3_env_to_move(env) >= 0 {
                let side = h3_env_to_move(env);
                assert_eq!(
                    h3_env_legal_mask(env, side, mask.as_mut_ptr(), mask.len()),
                    H3_OK
                );
                let cast = h3_sim::action::HERO_CAST;
                // Defend, or end the tactics phase.
                let other = if mask[1] == 1 {
                    1
                } else {
                    h3_sim::action::END_TACTICS
                };
                assert_eq!(mask[other], 1);
                let mut action = entry(other);
                if mask[cast] == 1 {
                    let e = cast as u32;
                    assert_eq!(
                        h3_env_spell_mask(env, side, e, spells.as_mut_ptr(), spells.len()),
                        H3_OK
                    );
                    let spell = spells.iter().position(|m| *m == 1).unwrap() as u16;
                    assert_eq!(
                        h3_env_target_mask(
                            env,
                            side,
                            e,
                            spell,
                            targets.as_mut_ptr(),
                            targets.len()
                        ),
                        H3_OK
                    );
                    let target = targets.iter().position(|m| *m == 1).unwrap() as u16;
                    assert_eq!(
                        h3_env_second_mask(
                            env,
                            side,
                            e,
                            spell,
                            target,
                            seconds.as_mut_ptr(),
                            seconds.len()
                        ),
                        H3_OK
                    );
                    assert_eq!(seconds.iter().position(|m| *m == 1), Some(0));
                    action = H3Action {
                        entry: e,
                        spell,
                        target,
                        second: 0,
                    };
                    casts += 1;
                }
                assert_eq!(h3_env_step(env, side, action, std::ptr::null_mut()), H3_OK);
            }
            h3_env_destroy(env);
        }
        assert_eq!(casts, 1);
    }

    #[test]
    fn a_stack_casts_head_by_head() {
        // The skirmish's Wisps cast Hasten (E04b T7b1): play until one may, then choose its
        // heads from their masks: its spell only, a target, no second target.
        let env = h3_env_create_synthetic(1, 7, 0);
        let mut mask = vec![0u8; ACTION_COUNT];
        let mut spells = vec![0u8; SPELL_COUNT];
        let mut targets = vec![0u8; TARGET_COUNT];
        let mut seconds = vec![0u8; TARGET_COUNT];
        let mut casts = 0;
        let cast = h3_sim::action::CREATURE_CAST;
        let e = cast as u32;
        unsafe {
            while casts == 0 && h3_env_to_move(env) >= 0 {
                let side = h3_env_to_move(env);
                assert_eq!(
                    h3_env_legal_mask(env, side, mask.as_mut_ptr(), mask.len()),
                    H3_OK
                );
                let mut action = entry(1);
                if mask[cast] == 1 {
                    assert_eq!(
                        h3_env_spell_mask(env, side, e, spells.as_mut_ptr(), spells.len()),
                        H3_OK
                    );
                    assert_eq!(spells.iter().filter(|m| **m == 1).count(), 1);
                    let spell = spells.iter().position(|m| *m == 1).unwrap() as u16;
                    assert_eq!(
                        h3_env_target_mask(
                            env,
                            side,
                            e,
                            spell,
                            targets.as_mut_ptr(),
                            targets.len()
                        ),
                        H3_OK
                    );
                    let target = targets.iter().position(|m| *m == 1).unwrap() as u16;
                    assert_eq!(
                        h3_env_second_mask(
                            env,
                            side,
                            e,
                            spell,
                            target,
                            seconds.as_mut_ptr(),
                            seconds.len()
                        ),
                        H3_OK
                    );
                    assert_eq!(seconds.iter().filter(|m| **m == 1).count(), 1);
                    assert_eq!(seconds[0], 1);
                    action = H3Action {
                        entry: e,
                        spell,
                        target,
                        second: 0,
                    };
                    casts += 1;
                }
                assert_eq!(h3_env_step(env, side, action, std::ptr::null_mut()), H3_OK);
            }
            h3_env_destroy(env);
        }
        assert_eq!(casts, 1);
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
            assert_eq!(h3_env_spell_mask(null, 0, 0, &mut byte, 1), H3_ERR_NULL);
            assert_eq!(h3_env_target_mask(null, 0, 0, 0, &mut byte, 1), H3_ERR_NULL);
            assert_eq!(
                h3_env_second_mask(null, 0, 0, 0, 0, &mut byte, 1),
                H3_ERR_NULL
            );
            assert_eq!(h3_env_observe(null, 0, &mut float, 1), H3_ERR_NULL);
            assert_eq!(
                h3_env_step(null, 0, entry(0), std::ptr::null_mut()),
                H3_ERR_NULL
            );
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
            assert_eq!(h3_env_spell_mask(env, 0, 0, &mut byte, 1), H3_ERR_ARGUMENT);
            assert_eq!(
                h3_env_target_mask(env, 0, 0, 0, std::ptr::null_mut(), 0),
                H3_ERR_NULL
            );
            assert_eq!(
                h3_env_second_mask(env, 3, 0, 0, 0, &mut byte, TARGET_COUNT),
                H3_ERR_ARGUMENT
            );
            assert_eq!(h3_env_observe(env, 0, std::ptr::null_mut(), 0), H3_ERR_NULL);
            assert_eq!(h3_env_observe(env, 0, &mut float, 1), H3_ERR_ARGUMENT);
            assert_eq!(
                h3_env_step(env, 5, entry(0), std::ptr::null_mut()),
                H3_ERR_ARGUMENT
            );
            let mover = h3_env_to_move(env);
            assert_eq!(
                h3_env_step(env, mover, entry(u32::MAX as usize), std::ptr::null_mut()),
                H3_ERR_INVALID_ACTION
            );
            let other = 1 - mover;
            assert_eq!(
                h3_env_step(env, other, entry(1), std::ptr::null_mut()),
                H3_ERR_REJECTED
            );
            // Out of turn an action is not even decoded (E04b T11b review, F-sec-2: the kind of
            // refusal told the other side a Faerie Dragon's drawn spell).
            assert_eq!(
                h3_env_step(env, other, entry(u32::MAX as usize), std::ptr::null_mut()),
                H3_ERR_REJECTED
            );
            assert_eq!(
                h3_env_step(env, 1 - other, entry(1), std::ptr::null_mut()),
                H3_OK
            );
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
