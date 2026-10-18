//! Environments from scenario texts on the built-in packs (E05 T2; ADR-0025 point 9, design
//! §4.1): the battle lane's scenario format (`h3_pack_battle::parse_scenario`, the files of
//! `packs/synthetic/scenarios/`) describes every battle the engine plays, sieges included.

use std::ffi::c_char;
use std::sync::Arc;

use h3_sim::BattleEnv;

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
    let packs = def
        .packs
        .iter()
        .map(|id| {
            builtin_pack(id).ok_or_else(|| {
                format!(
                    "unknown pack `{id}` (built-in packs: {})",
                    BUILTIN_PACKS.join(", ")
                )
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
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
