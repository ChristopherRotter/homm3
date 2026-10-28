//! The training batch through the C ABI (E05 T3d; ADR-0025, `docs/epics/E05-log.md` §1):
//! `h3_batch_*`. A handle owns an [`h3_sim::BattleBatch`]; every call writes into the caller's
//! arrays (numpy arrays in the Python binding, ADR-0025 point 11): the observations, entry masks
//! and final observations directly from the batch's workers, the small per-env values converted
//! from the batch's reports after the call. Since v28 (E05 T5b2) an episode source entry may be
//! the encounter distribution (`h3_batch_add_encounters`); since v31 (E06 T2c) each entry's
//! engine identity reads back (`h3_batch_scenario_info`) for a training run's record.

use std::ffi::c_char;
use std::panic::{AssertUnwindSafe, catch_unwind};

use h3_sim::batch::episode_seed;
use h3_sim::{
    ACTION_COUNT, Action, BatchConfig, BatchError, BattleBatch, Currency, ENCOUNTERS_VERSION,
    EnvReport, MAX_ENVS, OBSERVATION_LEN, Outputs, PREVIEW_LEN, Policy, RewardConfig, SPELL_COUNT,
    SYSTEM_POLICY_VERSION, Seat, StepError, TARGET_COUNT,
};

use crate::{
    H3_ERR_ARGUMENT, H3_ERR_EPISODE_OVER, H3_ERR_INVALID_ACTION, H3_ERR_NULL, H3_ERR_PANIC,
    H3_ERR_REJECTED, H3_OK, H3Action, H3Env, PANIC_MESSAGE, result_code, scenario, side, side_code,
};

/// The batch is not ready for the call: it has no scenario yet (`h3_batch_reset`), or it was not
/// reset (`h3_batch_step` and the head calls).
pub const H3_ERR_NOT_READY: i32 = -6;

/// `H3Seats`: the caller plays the side (the batch reports it to move and takes its actions).
pub const H3_SEAT_EXTERNAL: i32 = 0;
/// `H3Seats`: uniformly random legal commands played inside the call (tests and benchmarks).
pub const H3_SEAT_RANDOM: i32 = 1;
/// `H3Seats`: the baseline bot Aggressor inside the call (v30, E05 T7; `h3_sim::bots`).
pub const H3_SEAT_AGGRESSOR: i32 = 2;
/// `H3Seats`: the baseline bot Greedy inside the call (v30, E05 T7; `h3_sim::bots`).
pub const H3_SEAT_GREEDY: i32 = 3;
/// `H3Seats`: the system actor inside the call, the original's computer (v32, E06 T7;
/// `h3_battle_ai::choose`); an encounter of a neutral kind has it as its defender.
pub const H3_SEAT_SYSTEM: i32 = 4;

/// `H3BatchBuffers.encounter`: the env's scenario is no encounter distribution.
pub const H3_ENCOUNTER_NONE: i32 = -1;
/// `H3BatchBuffers.encounter` and `h3_batch_add_encounters`' weights, in this order
/// (`h3_sim::Kind::ALL`): a hero attacks a wandering monster.
pub const H3_ENCOUNTER_WANDERING: i32 = 0;
/// A hero attacks the guards of a pile or a garrison.
pub const H3_ENCOUNTER_GUARDS: i32 = 1;
/// A hero attacks a hero in the field.
pub const H3_ENCOUNTER_HERO: i32 = 2;
/// A hero attacks a town a hero defends.
pub const H3_ENCOUNTER_SIEGE_HERO: i32 = 3;
/// A hero attacks a player's town without a hero (its garrison).
pub const H3_ENCOUNTER_SIEGE_GARRISON: i32 = 4;
/// A hero attacks a neutral town (its garrison).
pub const H3_ENCOUNTER_SIEGE_NEUTRAL: i32 = 5;

/// The most envs `h3_batch_create` accepts (`h3_sim::MAX_ENVS`, v31): a larger N is refused with
/// a message instead of aborting on its allocation.
pub const H3_BATCH_MAX_ENVS: u32 = MAX_ENVS;

/// `H3RewardConfig.currency`: the creatures' `ai_value`.
pub const H3_CURRENCY_AI_VALUE: u32 = 0;
/// `H3RewardConfig.currency`: the creatures' cost in the ruleset's money (gold).
pub const H3_CURRENCY_GOLD: u32 = 1;

/// The reward's settings (`h3_sim::RewardConfig`).
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct H3RewardConfig {
    /// `H3_CURRENCY_*`.
    pub currency: u32,
    /// The price of a spell point, in the currency (the default is 10).
    pub mana_price: u32,
}

/// A batch's settings (`h3_sim::BatchConfig`).
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct H3BatchConfig {
    /// The number of envs, N (1 to `H3_BATCH_MAX_ENVS`).
    pub envs: u32,
    /// Worker threads, T (at most N are started); 0 runs every call in the caller's thread.
    pub threads: u32,
    /// The batch's seed: every env's episode seeds derive from it (`h3_batch_episode_seed`).
    pub seed: u64,
    /// Accepted steps of both seats per episode before it is truncated; 0 = no limit.
    pub max_steps: u32,
    /// The reward's currency and mana price.
    pub reward: H3RewardConfig,
}

/// Who plays each side of a scenario: `H3_SEAT_*`, at least one external.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct H3Seats {
    /// The attacker's seat.
    pub attacker: i32,
    /// The defender's seat.
    pub defender: i32,
}

/// The caller's arrays a call writes, one row per env (N rows); a null pointer skips that output.
/// The arrays must not overlap.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct H3BatchBuffers {
    /// [N, `h3_observation_len()`]: the acting side's projected observation (0 without one).
    pub obs: *mut f32,
    /// [N, `h3_action_count()`]: its legal entries (0/1; never Retreat or Surrender).
    pub entry_mask: *mut u8,
    /// [N]: the acting side (0 attacker, 1 defender), -1 for an env that gave up.
    pub to_move: *mut i8,
    /// [N, 2]: this call's reward per side (attacker, defender).
    pub reward: *mut f32,
    /// [N]: 1 if an episode ended in this call (decided or truncated); the env shows the next.
    pub done: *mut u8,
    /// [N]: 1 if that episode was truncated.
    pub truncated: *mut u8,
    /// [N]: `H3_RESULT_*` of the episode that ended, else `H3_RESULT_NONE`.
    pub result: *mut i8,
    /// [N, 2, `h3_observation_len()`]: both sides' last observations, written only for the envs
    /// truncated in this call.
    pub final_obs: *mut f32,
    /// [N]: the env's current episode (0 the first; it grows at each automatic reset).
    pub episode: *mut u32,
    /// [N]: the index of its scenario, in the order they were added.
    pub scenario: *mut u32,
    /// [N]: its seed, trusted tooling (it decides the battle's draws: never an agent's input). A
    /// single env of the episode's scenario with both sides human and this seed plays the same
    /// battle; an encounter's episode is `h3_sim::batch::Scenario::episode`'s.
    pub episode_seed: *mut u64,
    /// [N]: `H3_OK`, or why the env refused this call's action (it is unchanged).
    pub status: *mut i32,
    /// [N]: the current episode's encounter kind (`H3_ENCOUNTER_*`), `H3_ENCOUNTER_NONE` for a
    /// scenario's episode (C ABI v28).
    pub encounter: *mut i8,
    /// [N, `h3_preview_len()`]: the acting side's previewed kills of each attack and shot entry
    /// from `h3_preview_base()`, the least and the greatest (the engine's damage preview of the
    /// entry's first strike, what the original shows a player; 0 for an entry not legal or without
    /// a side to move; C ABI v33, AI-Q7 (b)).
    pub preview: *mut f32,
}

/// An episode source entry's engine identity (v31, E06 T2c): what a training run's record names,
/// so that its results tie to the content they were trained on.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct H3ScenarioInfo {
    /// The content hash of the core ruleset its battles run on (a replay header's
    /// `ruleset_hash`).
    pub ruleset_hash: u64,
    /// The content hash of the battle ruleset (a replay header's `battle_ruleset_hash`).
    pub battle_ruleset_hash: u64,
    /// The content hash of the encounter distribution's adventure ruleset (its random monsters,
    /// the heroes' derivation); 0 for a scenario.
    pub adventure_ruleset_hash: u64,
    /// Its weight among the batch's entries.
    pub weight: u32,
    /// 1 for the encounter distribution (`h3_batch_add_encounters`), 0 for a scenario.
    pub encounters: u32,
}

/// Opaque batch handle.
#[derive(Debug)]
pub struct H3Batch {
    batch: BattleBatch,
    reports: Vec<EnvReport>,
    actions: Vec<Action>,
    /// One ruleset per distinct packs and profile (ADR-0025 point 9): the index of the first
    /// scenario that compiled it.
    rulesets: Vec<(Vec<String>, String, usize)>,
}

fn currency(code: u32) -> Option<Currency> {
    match code {
        H3_CURRENCY_AI_VALUE => Some(Currency::AiValue),
        H3_CURRENCY_GOLD => Some(Currency::Gold),
        _ => None,
    }
}

fn seat(code: i32) -> Option<Seat> {
    match code {
        H3_SEAT_EXTERNAL => Some(Seat::External),
        H3_SEAT_RANDOM => Some(Seat::Internal(Policy::Random)),
        H3_SEAT_AGGRESSOR => Some(Seat::Internal(Policy::Aggressor)),
        H3_SEAT_GREEDY => Some(Seat::Internal(Policy::Greedy)),
        H3_SEAT_SYSTEM => Some(Seat::Internal(Policy::System)),
        _ => None,
    }
}

/// The code of a refused call.
fn error_code(e: &BatchError) -> i32 {
    match e {
        BatchError::NoScenario | BatchError::NotReset => H3_ERR_NOT_READY,
        _ => H3_ERR_ARGUMENT,
    }
}

/// The status of an env's step.
fn status_code(error: Option<StepError>) -> i32 {
    match error {
        None => H3_OK,
        Some(StepError::InvalidAction) => H3_ERR_INVALID_ACTION,
        Some(StepError::Rejected(_)) => H3_ERR_REJECTED,
        Some(StepError::EpisodeOver) => H3_ERR_EPISODE_OVER,
    }
}

/// `ptr[0..len]`, or `None` for a null pointer.
///
/// # Safety
/// `ptr` must be null or valid for `len` elements of writes, used by nothing else meanwhile.
unsafe fn rows<'a, T>(ptr: *mut T, len: usize) -> Option<&'a mut [T]> {
    // SAFETY: per the contract.
    (!ptr.is_null()).then(|| unsafe { std::slice::from_raw_parts_mut(ptr, len) })
}

/// `ptr[0..len]`.
///
/// # Safety
/// `ptr` must be non-null and valid for `len` elements of reads.
unsafe fn read<'a, T>(ptr: *const T, len: usize) -> &'a [T] {
    // SAFETY: per the contract.
    unsafe { std::slice::from_raw_parts(ptr, len) }
}

/// Plays `call` on the batch with the outputs of `out` (null: none) and converts its reports into
/// `out`'s per-env arrays; `H3_ERR_REJECTED` if an env refused its action.
///
/// # Safety
/// `out` must be null or point to buffers valid for their shapes (N rows).
unsafe fn with_outputs(
    h: &mut H3Batch,
    out: *const H3BatchBuffers,
    call: impl FnOnce(&mut BattleBatch, &[Action], &mut Outputs<'_>) -> Result<(), BatchError>,
) -> i32 {
    let n = h.batch.len();
    // SAFETY: a non-null `out` is valid for one read per the contract.
    let buffers = unsafe { out.as_ref() }.copied();
    // SAFETY: each non-null array is valid for its shape per the contract.
    let mut outputs = buffers.map_or_else(Outputs::default, |b| unsafe {
        Outputs {
            obs: rows(b.obs, n * OBSERVATION_LEN),
            entry_mask: rows(b.entry_mask, n * ACTION_COUNT),
            final_obs: rows(b.final_obs, n * 2 * OBSERVATION_LEN),
            preview: rows(b.preview, n * PREVIEW_LEN),
            reports: None,
        }
    });
    outputs.reports = Some(&mut h.reports);
    if let Err(e) = call(&mut h.batch, &h.actions, &mut outputs) {
        return error_code(&e);
    }
    if let Some(b) = buffers {
        // SAFETY: as above.
        unsafe { write_reports(&h.reports, &b) };
    }
    if h.reports.iter().any(|r| r.error.is_some()) {
        H3_ERR_REJECTED
    } else {
        H3_OK
    }
}

/// Converts `reports` into the per-env arrays of `b`.
///
/// # Safety
/// Each non-null array of `b` must be valid for its shape (`reports.len()` rows).
unsafe fn write_reports(reports: &[EnvReport], b: &H3BatchBuffers) {
    let n = reports.len();
    // SAFETY: per the contract (one row per report, two values per row for the rewards).
    let (to_move, reward, done, truncated, result, episode, scenario, seed, status, encounter) = unsafe {
        (
            rows(b.to_move, n),
            rows(b.reward, 2 * n),
            rows(b.done, n),
            rows(b.truncated, n),
            rows(b.result, n),
            rows(b.episode, n),
            rows(b.scenario, n),
            rows(b.episode_seed, n),
            rows(b.status, n),
            rows(b.encounter, n),
        )
    };
    put(to_move, reports, |r| side_code(r.to_move) as i8);
    put(done, reports, |r| u8::from(r.done));
    put(truncated, reports, |r| u8::from(r.truncated));
    put(result, reports, |r| result_code(r.result) as i8);
    put(episode, reports, |r| r.episode);
    put(scenario, reports, |r| r.scenario);
    put(seed, reports, |r| r.episode_seed);
    put(status, reports, |r| status_code(r.error));
    put(encounter, reports, |r| {
        r.encounter.map_or(H3_ENCOUNTER_NONE, |k| k.index() as i32) as i8
    });
    if let Some(reward) = reward {
        for (row, r) in reward.chunks_exact_mut(2).zip(reports) {
            row.copy_from_slice(&r.reward);
        }
    }
}

/// Writes `value(report)` into each row of `array`, if it is present.
fn put<T>(array: Option<&mut [T]>, reports: &[EnvReport], value: impl Fn(&EnvReport) -> T) {
    if let Some(array) = array {
        for (a, r) in array.iter_mut().zip(reports) {
            *a = value(r);
        }
    }
}

/// Creates a batch of `config->envs` envs without scenarios (add them with
/// `h3_batch_add_scenario`, then `h3_batch_reset`). Returns null on error and writes its message
/// into `error[0..error_len]` as for `h3_env_create_scenario`.
///
/// # Safety
/// `config` must be null or valid for one read; `error` must be null or valid for `error_len`
/// bytes of writes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn h3_batch_create(
    config: *const H3BatchConfig,
    error: *mut c_char,
    error_len: usize,
) -> *mut H3Batch {
    // SAFETY: a non-null `config` is valid for one read per the contract.
    let config = unsafe { config.as_ref() }.copied();
    let created = catch_unwind(|| {
        let config = config.ok_or("the config is null")?;
        let currency = currency(config.reward.currency).ok_or("unknown currency")?;
        let batch = BattleBatch::new(BatchConfig {
            envs: config.envs,
            threads: config.threads,
            seed: config.seed,
            max_steps: config.max_steps,
            reward: RewardConfig {
                currency,
                mana_price: config.reward.mana_price,
            },
        })
        .map_err(|e| e.to_string())?;
        let n = batch.len();
        Ok::<_, String>(H3Batch {
            batch,
            reports: vec![EnvReport::default(); n],
            actions: vec![Action::default(); n],
            rulesets: Vec::new(),
        })
    })
    .unwrap_or(Err(PANIC_MESSAGE.to_owned()));
    match created {
        Ok(h) => Box::into_raw(Box::new(h)),
        Err(message) => {
            // SAFETY: per the contract.
            unsafe { scenario::write_error(&message, error, error_len) };
            std::ptr::null_mut()
        }
    }
}

/// Destroys a batch (its threads end). Null is ignored.
///
/// # Safety
/// `batch` must be null or a handle returned by `h3_batch_create` that has not been destroyed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn h3_batch_destroy(batch: *mut H3Batch) {
    if !batch.is_null() {
        // SAFETY: per the contract, `batch` came from `Box::into_raw` and is destroyed only once.
        drop(unsafe { Box::from_raw(batch) });
    }
}

/// Adds an episode source entry: the scenario `toml[0..len]` (as for `h3_env_create_scenario`)
/// with `weight` (at least 1) among the batch's scenarios and its `seats`. Episodes started from
/// now on draw it by weight; scenarios on the same packs and profile share one ruleset. Every
/// side plays as a human player's whatever the scenario says (ADR-0025 point 6). On error the
/// batch is unchanged and the message is written into `error[0..error_len]`.
///
/// # Safety
/// `batch` must be null or a live handle not used concurrently; `toml` must be null or valid for
/// `len` bytes of reads; `error` must be null or valid for `error_len` bytes of writes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn h3_batch_add_scenario(
    batch: *mut H3Batch,
    toml: *const c_char,
    len: usize,
    weight: u32,
    seats: H3Seats,
    error: *mut c_char,
    error_len: usize,
) -> i32 {
    // SAFETY: per the contract.
    let Some(h) = (unsafe { batch.as_mut() }) else {
        return H3_ERR_NULL;
    };
    if toml.is_null() {
        // SAFETY: per the contract.
        unsafe { scenario::write_error("the scenario text is null", error, error_len) };
        return H3_ERR_NULL;
    }
    // SAFETY: a non-null `toml` is valid for `len` bytes of reads per the contract.
    let bytes = unsafe { read(toml.cast::<u8>(), len) };
    let added = catch_unwind(AssertUnwindSafe(|| {
        let text = std::str::from_utf8(bytes)
            .map_err(|e| format!("the scenario text is not UTF-8: {e}"))?;
        let (Some(attacker), Some(defender)) = (seat(seats.attacker), seat(seats.defender)) else {
            return Err("unknown seat".to_owned());
        };
        scenario::add_to_batch(
            &mut h.batch,
            &mut h.rulesets,
            text,
            weight,
            [attacker, defender],
        )
    }));
    match added {
        Ok(Ok(())) => H3_OK,
        Ok(Err(message)) => {
            // SAFETY: per the contract.
            unsafe { scenario::write_error(&message, error, error_len) };
            H3_ERR_ARGUMENT
        }
        Err(_) => {
            // SAFETY: per the contract.
            unsafe { scenario::write_error(PANIC_MESSAGE, error, error_len) };
            H3_ERR_PANIC
        }
    }
}

/// Adds the encounter distribution v1 (E05 T5b2, `docs/epics/E05-log.md` §4.2) as an episode
/// source entry with `weight` (at least 1): each episode's battle and seats are drawn from its
/// seed (a neutral defender is played inside the call). `kinds` is null for the distribution's
/// weights (the neutral kinds 0, E06 T7) or points to six weights in `H3_ENCOUNTER_*` order (a
/// curriculum; not all 0). `seats` is null for each encounter's own seats, or replaces them for
/// every episode of the entry (v29, E05 T8b: an evaluation against an internal seat; at least one
/// external). It runs on the original packs (`base`, `ab`, `sod`, `base:faithful`), sharing their
/// ruleset with the batch's scenarios on them. On error the batch is unchanged and the message
/// is written into `error[0..error_len]`.
///
/// # Safety
/// `batch` must be null or a live handle not used concurrently; `kinds` must be null or valid for
/// six reads; `seats` must be null or point to an `H3Seats`; `error` must be null or valid for
/// `error_len` bytes of writes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn h3_batch_add_encounters(
    batch: *mut H3Batch,
    weight: u32,
    kinds: *const u32,
    seats: *const H3Seats,
    error: *mut c_char,
    error_len: usize,
) -> i32 {
    // SAFETY: per the contract.
    let Some(h) = (unsafe { batch.as_mut() }) else {
        return H3_ERR_NULL;
    };
    // SAFETY: a non-null `kinds` is valid for six reads per the contract.
    let kinds = (!kinds.is_null()).then(|| unsafe { read(kinds, 6) });
    // SAFETY: a non-null `seats` points to an `H3Seats` per the contract.
    let seats = unsafe { seats.as_ref() }.copied();
    let added = catch_unwind(AssertUnwindSafe(|| {
        let kinds = kinds.map(|k| std::array::from_fn(|i| k[i]));
        let seats = match seats.map(|s| (seat(s.attacker), seat(s.defender))) {
            None => None,
            Some((Some(attacker), Some(defender))) => Some([attacker, defender]),
            Some(_) => return Err("unknown seat".to_owned()),
        };
        scenario::add_encounters(&mut h.batch, &mut h.rulesets, weight, kinds, seats)
    }));
    match added {
        Ok(Ok(())) => H3_OK,
        Ok(Err(message)) => {
            // SAFETY: per the contract.
            unsafe { scenario::write_error(&message, error, error_len) };
            H3_ERR_ARGUMENT
        }
        Err(_) => {
            // SAFETY: per the contract.
            unsafe { scenario::write_error(PANIC_MESSAGE, error, error_len) };
            H3_ERR_PANIC
        }
    }
}

/// Starts every env's first episode (again from the start on a later call: the batch then replays
/// the same episodes) and writes them into `out` (may be null).
///
/// # Safety
/// `batch` must be null or a live handle not used concurrently; `out` must be null or point to
/// buffers valid for their shapes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn h3_batch_reset(batch: *mut H3Batch, out: *const H3BatchBuffers) -> i32 {
    // SAFETY: per the contract.
    let Some(h) = (unsafe { batch.as_mut() }) else {
        return H3_ERR_NULL;
    };
    let reset = catch_unwind(AssertUnwindSafe(|| {
        // SAFETY: per the contract.
        unsafe { with_outputs(h, out, |b, _, out| b.reset(out)) }
    }));
    reset.unwrap_or(H3_ERR_PANIC)
}

/// Plays `actions[i]` (N of them) for env i's acting side, then its internal seats until an
/// external side must act or the episode ends (then its next episode starts), and writes the
/// result into `out` (may be null). An env that refuses its action is unchanged (its `status`);
/// the others step, and the call returns `H3_ERR_REJECTED`.
///
/// # Safety
/// `batch` must be null or a live handle not used concurrently; `actions` must be null or valid
/// for N reads; `out` must be null or point to buffers valid for their shapes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn h3_batch_step(
    batch: *mut H3Batch,
    actions: *const H3Action,
    out: *const H3BatchBuffers,
) -> i32 {
    // SAFETY: per the contract.
    let Some(h) = (unsafe { batch.as_mut() }) else {
        return H3_ERR_NULL;
    };
    if actions.is_null() {
        return H3_ERR_NULL;
    }
    // SAFETY: `actions` is valid for N reads per the contract.
    let actions = unsafe { read(actions, h.batch.len()) };
    let stepped = catch_unwind(AssertUnwindSafe(|| {
        for (a, action) in h.actions.iter_mut().zip(actions) {
            *a = Action::from(*action);
        }
        // SAFETY: per the contract.
        unsafe { with_outputs(h, out, |b, actions, out| b.step(actions, out)) }
    }));
    stepped.unwrap_or(H3_ERR_PANIC)
}

/// The arguments of a head call: the batch and its `out[0..len]`, `len` at least N × `width`.
///
/// # Safety
/// As the head calls.
unsafe fn head_args<'a>(
    batch: *mut H3Batch,
    inputs: &[*const u16],
    entries: *const u32,
    out: *mut u8,
    len: usize,
    width: usize,
) -> Result<(&'a mut H3Batch, &'a [u32], &'a mut [u8]), i32> {
    // SAFETY: per the contract.
    let h = unsafe { batch.as_mut() }.ok_or(H3_ERR_NULL)?;
    if entries.is_null() || out.is_null() || inputs.iter().any(|p| p.is_null()) {
        return Err(H3_ERR_NULL);
    }
    let n = h.batch.len();
    if len < n * width {
        return Err(H3_ERR_ARGUMENT);
    }
    // SAFETY: `entries` is valid for N reads and `out` for `len >= N * width` writes.
    unsafe {
        Ok((
            h,
            read(entries, n),
            std::slice::from_raw_parts_mut(out, n * width),
        ))
    }
}

/// Writes each env's spell-head mask after choosing `entries[i]` (N of them) into
/// `out[i * h3_spell_count()..]` (0/1; "0 only" for an entry that is no cast); `len` must be at
/// least N × `h3_spell_count()`.
///
/// # Safety
/// `batch` must be null or a live handle not used concurrently; `entries` must be null or valid
/// for N reads; `out` must be null or valid for `len` bytes of writes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn h3_batch_spell_mask(
    batch: *mut H3Batch,
    entries: *const u32,
    out: *mut u8,
    len: usize,
) -> i32 {
    // SAFETY: per the contract.
    let (h, entries, out) = match unsafe { head_args(batch, &[], entries, out, len, SPELL_COUNT) } {
        Ok(args) => args,
        Err(code) => return code,
    };
    let masked = catch_unwind(AssertUnwindSafe(|| h.batch.spell_mask(entries, out)));
    masked.map_or(H3_ERR_PANIC, |r| {
        r.map_or_else(|e| error_code(&e), |()| H3_OK)
    })
}

/// Writes each env's target-head mask after choosing `entries[i]` and `spells[i]` into
/// `out[i * h3_target_count()..]`; `len` must be at least N × `h3_target_count()`.
///
/// # Safety
/// As [`h3_batch_spell_mask`], and `spells` must be null or valid for N reads.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn h3_batch_target_mask(
    batch: *mut H3Batch,
    entries: *const u32,
    spells: *const u16,
    out: *mut u8,
    len: usize,
) -> i32 {
    // SAFETY: per the contract.
    let (h, entries, out) =
        match unsafe { head_args(batch, &[spells], entries, out, len, TARGET_COUNT) } {
            Ok(args) => args,
            Err(code) => return code,
        };
    // SAFETY: `spells` is non-null (checked) and valid for N reads per the contract.
    let spells = unsafe { read(spells, h.batch.len()) };
    let masked = catch_unwind(AssertUnwindSafe(|| {
        h.batch.target_mask(entries, spells, out)
    }));
    masked.map_or(H3_ERR_PANIC, |r| {
        r.map_or_else(|e| error_code(&e), |()| H3_OK)
    })
}

/// Writes each env's second-target mask after choosing `entries[i]`, `spells[i]` and
/// `targets[i]` into `out[i * h3_target_count()..]`; `len` must be at least
/// N × `h3_target_count()`.
///
/// # Safety
/// As [`h3_batch_target_mask`], and `targets` must be null or valid for N reads.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn h3_batch_second_mask(
    batch: *mut H3Batch,
    entries: *const u32,
    spells: *const u16,
    targets: *const u16,
    out: *mut u8,
    len: usize,
) -> i32 {
    // SAFETY: per the contract.
    let (h, entries, out) =
        match unsafe { head_args(batch, &[spells, targets], entries, out, len, TARGET_COUNT) } {
            Ok(args) => args,
            Err(code) => return code,
        };
    let n = h.batch.len();
    // SAFETY: both are non-null (checked) and valid for N reads per the contract.
    let (spells, targets) = unsafe { (read(spells, n), read(targets, n)) };
    let masked = catch_unwind(AssertUnwindSafe(|| {
        h.batch.second_mask(entries, spells, targets, out)
    }));
    masked.map_or(H3_ERR_PANIC, |r| {
        r.map_or_else(|e| error_code(&e), |()| H3_OK)
    })
}

/// The seed of episode `episode` (0 the first) of env `env` in a batch with seed `batch_seed`, as
/// [`H3BatchBuffers::episode_seed`] (trusted tooling).
#[unsafe(no_mangle)]
pub extern "C" fn h3_batch_episode_seed(batch_seed: u64, env: u32, episode: u32) -> u64 {
    episode_seed(batch_seed, env, episode)
}

/// The number of episode source entries the batch holds, in the order added (0 for null; v31).
///
/// # Safety
/// `batch` must be null or a live handle not used concurrently.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn h3_batch_scenario_count(batch: *const H3Batch) -> u32 {
    // SAFETY: per the contract.
    unsafe { batch.as_ref() }.map_or(0, |h| h.batch.scenarios().len() as u32)
}

/// Writes entry `index`'s engine identity (in the order added) into `out` (v31, E06 T2c).
/// `H3_ERR_NULL` for a null pointer, `H3_ERR_ARGUMENT` for an index past the entries.
///
/// # Safety
/// `batch` must be null or a live handle not used concurrently; `out` must be null or valid for
/// one write.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn h3_batch_scenario_info(
    batch: *const H3Batch,
    index: u32,
    out: *mut H3ScenarioInfo,
) -> i32 {
    // SAFETY: per the contract.
    let (Some(h), Some(out)) = (unsafe { batch.as_ref() }, unsafe { out.as_mut() }) else {
        return H3_ERR_NULL;
    };
    let Some(scenario) = h.batch.scenarios().get(index as usize) else {
        return H3_ERR_ARGUMENT;
    };
    let ruleset = scenario.ruleset();
    let sampler = scenario.sampler();
    *out = H3ScenarioInfo {
        ruleset_hash: ruleset.core_hash(),
        battle_ruleset_hash: ruleset.content_hash(),
        adventure_ruleset_hash: sampler.map_or(0, |s| s.adventure().content_hash()),
        weight: scenario.weight(),
        encounters: u32::from(sampler.is_some()),
    };
    H3_OK
}

/// A world of env `env`'s battle for side `viewer` (0 attacker, 1 defender) from `seed` (v34, E06
/// T8; `h3_sim::BattleBatch::determinize`, ADR-0025 point 12a): a new single env (destroy it with
/// `h3_env_destroy`) whose battle is the env's with everything `viewer` may not know replaced
/// (the other side's hero guessed by the env's encounter distribution, its hidden obstacles and a
/// drawn spell drawn again, the stream re-seeded), the agent's to fork, step and observe; its
/// steps (its forks' too) score a decided battle by the batch's reward v1 (v35, E25 T1a), the
/// batch's reward for the same battle, where any other single env scores ±1. Null
/// for a null or not reset batch, an env past it or not playing, or a `viewer` other than the
/// env's side to move (E06 T9b: a world of the other side would hold its own hero as it is, which
/// its opponent, the agent, may not know).
///
/// # Safety
/// `batch` must be null or a live handle not used concurrently.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn h3_batch_determinize(
    batch: *const H3Batch,
    env: u32,
    viewer: i32,
    seed: u64,
) -> *mut H3Env {
    // SAFETY: per the contract.
    let (Some(h), Some(viewer)) = (unsafe { batch.as_ref() }, side(viewer)) else {
        return std::ptr::null_mut();
    };
    let world = catch_unwind(AssertUnwindSafe(|| {
        let acting = h.batch.env(env as usize).and_then(|e| e.to_move());
        h.batch
            .determinize(env as usize, viewer, seed)
            .ok()
            .filter(|_| acting == Some(viewer))
            .map(|w| Box::new(H3Env::new(w)))
    }));
    world
        .ok()
        .flatten()
        .map_or(std::ptr::null_mut(), Box::into_raw)
}

/// A true world of env `env`'s battle from `seed` (v35, E25 T1b;
/// `h3_sim::BattleBatch::true_world`): a new single env (destroy it with `h3_env_destroy`) whose
/// battle is the env's with nothing guessed, its stream re-seeded, scored as a world. Trusted
/// tooling, a diagnostic: it reads the whole state, never an agent's input (ADR-0025 point 12a).
/// Null for a null or not reset batch, or an env past it or not playing.
///
/// # Safety
/// `batch` must be null or a live handle not used concurrently.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn h3_batch_true_world(
    batch: *const H3Batch,
    env: u32,
    seed: u64,
) -> *mut H3Env {
    // SAFETY: per the contract.
    let Some(h) = (unsafe { batch.as_ref() }) else {
        return std::ptr::null_mut();
    };
    let world = catch_unwind(AssertUnwindSafe(|| {
        let playing = h.batch.env(env as usize).is_some_and(|e| e.to_move().is_some());
        h.batch
            .true_world(env as usize, seed)
            .ok()
            .filter(|_| playing)
            .map(|w| Box::new(H3Env::new(w)))
    }));
    world
        .ok()
        .flatten()
        .map_or(std::ptr::null_mut(), Box::into_raw)
}

/// The version of the encounter distribution `h3_batch_add_encounters` plays
/// (`h3_sim::ENCOUNTERS_VERSION`; v31).
#[unsafe(no_mangle)]
pub extern "C" fn h3_encounters_version() -> u32 {
    ENCOUNTERS_VERSION
}

/// The behaviour version of the system actor's policy, `H3_SEAT_SYSTEM`
/// (`h3_sim::SYSTEM_POLICY_VERSION`, ADR-0027 point 12; v32): a training run's record names it.
#[unsafe(no_mangle)]
pub extern "C" fn h3_system_policy_version() -> u32 {
    SYSTEM_POLICY_VERSION
}

#[cfg(test)]
mod tests;
