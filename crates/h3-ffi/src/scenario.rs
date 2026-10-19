//! Environments from scenario texts on the built-in packs (E05 T2; ADR-0025 point 9, design
//! §4.1): the battle lane's scenario format (`h3_pack_battle::parse_scenario`, the files of
//! `packs/synthetic/scenarios/`) describes every battle the engine plays, sieges included.

use std::ffi::c_char;
use std::sync::Arc;

use h3_pack_battle::ScenarioDef;
use h3_sim::{BattleBatch, BattleEnv, EncounterTable, Sampler, Scenario, Seat};

/// The ids of the built-in packs, compiled into the library ([`builtin_pack`]).
pub const BUILTIN_PACKS: [&str; 5] = ["synthetic", "synthetic-ext", "base", "ab", "sod"];

/// The built-in pack `id` as its `(core.toml, battle.toml)` texts; `None` for any other id.
/// Packs from files or texts stay out until a mod needs them (design §4.1).
pub fn builtin_pack(id: &str) -> Option<(&'static str, &'static str)> {
    match id {
        "synthetic" => Some(h3_pack_battle::SYNTHETIC_PACK),
        "synthetic-ext" => Some(h3_pack_battle::SYNTHETIC_EXT_PACK),
        "base" => Some(h3_pack_battle::BASE_PACK),
        "ab" => Some(h3_pack_battle::AB_PACK),
        "sod" => Some(h3_pack_battle::SOD_PACK),
        _ => None,
    }
}

/// An environment for the scenario `text` on the built-in packs it names, its battle started
/// with `seed`: obstacles generated for each episode from its seed where the scenario asks for
/// random ones (BAT-GEO-004), episodes truncated after `max_steps` steps (0 = no limit). The
/// scenario's `difficulty` and `human` apply as written (ADR-0023 point 7).
///
/// # Errors
/// A message naming the stage that failed: the scenario's syntax or keys, an unknown pack, the
/// packs' compilation, or the battle's setup.
pub fn scenario_env(text: &str, seed: u64, max_steps: u32) -> Result<BattleEnv, String> {
    let def = h3_pack_battle::parse_scenario(text).map_err(|e| format!("scenario: {e}"))?;
    let packs = packs(&def)?;
    let ruleset =
        h3_pack_battle::compile(&packs, &def.profile).map_err(|e| format!("packs: {e}"))?;
    let setup = def
        .resolve(&ruleset)
        .map_err(|e| format!("scenario: {e}"))?;
    let create = if def.random_obstacles {
        BattleEnv::with_random_obstacles
    } else {
        BattleEnv::new
    };
    create(Arc::new(ruleset), Arc::new(setup), seed, max_steps).map_err(|e| format!("setup: {e}"))
}

/// The texts of the built-in packs `def` names.
fn packs(def: &ScenarioDef) -> Result<Vec<(&'static str, &'static str)>, String> {
    def.packs
        .iter()
        .map(|id| {
            builtin_pack(id).ok_or_else(|| {
                format!(
                    "unknown pack `{id}` (built-in packs: {})",
                    BUILTIN_PACKS.join(", ")
                )
            })
        })
        .collect()
}

/// Adds the scenario `text` on the built-in packs it names to `batch` with `weight` and `seats`
/// (E05 T3d). Scenarios on the same packs and profile share one ruleset (ADR-0025 point 9):
/// `rulesets` holds, per packs and profile, the index of the first scenario that compiled it.
///
/// # Errors
/// A message naming the stage that failed, as [`scenario_env`]'s, or the batch's refusal (seats,
/// weight); the batch is unchanged.
pub fn add_to_batch(
    batch: &mut BattleBatch,
    rulesets: &mut Vec<(Vec<String>, String, usize)>,
    text: &str,
    weight: u32,
    seats: [Seat; 2],
) -> Result<(), String> {
    let def = h3_pack_battle::parse_scenario(text).map_err(|e| format!("scenario: {e}"))?;
    let shared = rulesets
        .iter()
        .find(|(packs, profile, _)| *packs == def.packs && *profile == def.profile)
        .and_then(|&(_, _, index)| batch.scenarios().get(index))
        .map(|s| s.ruleset().clone());
    let compiled = shared.is_none();
    let ruleset = match shared {
        Some(ruleset) => ruleset,
        None => Arc::new(
            h3_pack_battle::compile(&packs(&def)?, &def.profile)
                .map_err(|e| format!("packs: {e}"))?,
        ),
    };
    let setup = def
        .resolve(&ruleset)
        .map_err(|e| format!("scenario: {e}"))?;
    let scenario = Scenario::new(ruleset, &setup, def.random_obstacles, weight, seats)
        .map_err(|e| e.to_string())?;
    let index = batch.scenarios().len();
    batch.add_scenario(scenario).map_err(|e| e.to_string())?;
    if compiled {
        rulesets.push((def.packs, def.profile, index));
    }
    Ok(())
}

/// The packs and profile of the encounter distribution v1: the census's (`census-v1.toml`).
const ENCOUNTER_PACKS: [&str; 3] = ["base", "ab", "sod"];
/// See [`ENCOUNTER_PACKS`].
const ENCOUNTER_PROFILE: &str = "base:faithful";

/// Adds the encounter distribution v1 (E05 T5b2; `h3_sim::encounter`) to `batch` with `weight`,
/// its kinds drawn by `kinds` (in `h3_sim::Kind::ALL`'s order) or, without, by the
/// distribution's default (the neutral kinds 0 until E23), each episode with the encounter's
/// seats or `seats` (E05 T8b, `Scenario::encounters`). It runs on the original packs
/// (`base`, `ab`, `sod`, `base:faithful`), sharing their ruleset with the batch's scenarios on
/// them (ADR-0025 point 9); the adventure part (its random monsters) is compiled here.
///
/// # Errors
/// A message naming the stage that failed (`packs:`, `adventure:`, `encounters:`), or the batch's
/// refusal (weight, seats); the batch is unchanged.
pub fn add_encounters(
    batch: &mut BattleBatch,
    rulesets: &mut Vec<(Vec<String>, String, usize)>,
    weight: u32,
    kinds: Option<[u32; 6]>,
    seats: Option<[Seat; 2]>,
) -> Result<(), String> {
    let packs: Vec<String> = ENCOUNTER_PACKS.map(String::from).to_vec();
    let shared = rulesets
        .iter()
        .find(|(p, profile, _)| *p == packs && profile == ENCOUNTER_PROFILE)
        .and_then(|&(_, _, index)| batch.scenarios().get(index))
        .map(|s| s.ruleset().clone());
    let compiled = shared.is_none();
    let battle = match shared {
        Some(ruleset) => ruleset,
        None => Arc::new(
            h3_pack_battle::compile(
                &[
                    h3_pack_battle::BASE_PACK,
                    h3_pack_battle::AB_PACK,
                    h3_pack_battle::SOD_PACK,
                ],
                ENCOUNTER_PROFILE,
            )
            .map_err(|e| format!("packs: {e}"))?,
        ),
    };
    let (_, adventure) =
        h3_pack_adventure::compile(&h3_pack_adventure::ORIGINAL_PACKS, ENCOUNTER_PROFILE)
            .map_err(|e| format!("adventure: {e}"))?;
    let mut sampler = Sampler::new(battle, Arc::new(adventure), &EncounterTable::v1())
        .map_err(|e| format!("encounters: {e}"))?;
    if let Some(kinds) = kinds {
        sampler = sampler
            .with_weights(kinds)
            .map_err(|e| format!("encounters: {e}"))?;
    }
    let scenario =
        Scenario::encounters(Arc::new(sampler), weight, seats).map_err(|e| e.to_string())?;
    let index = batch.scenarios().len();
    batch.add_scenario(scenario).map_err(|e| e.to_string())?;
    if compiled {
        rulesets.push((packs, ENCOUNTER_PROFILE.to_owned(), index));
    }
    Ok(())
}

/// Writes `message` into `error[0..len]` as a NUL-terminated string, cut at a character boundary
/// to fit; writes nothing if `error` is null or `len` is 0.
///
/// # Safety
/// `error` must be null or valid for `len` bytes of writes.
pub unsafe fn write_error(message: &str, error: *mut c_char, len: usize) {
    if error.is_null() || len == 0 {
        return;
    }
    let n = message.floor_char_boundary(len - 1);
    // SAFETY: `error` is valid for `len` bytes of writes per the contract.
    let out = unsafe { std::slice::from_raw_parts_mut(error.cast::<u8>(), len) };
    out[..n].copy_from_slice(&message.as_bytes()[..n]);
    out[n] = 0;
}

#[cfg(test)]
mod tests;
