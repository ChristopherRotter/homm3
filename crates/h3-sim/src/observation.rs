//! Flat observation encoding (version 1) of a per-viewer [`Observation`].
//!
//! Layout: 4 global features, then [`STACK_FEATURES`] features for each of the
//! [`STACK_CAPACITY`] stack slots (zeros for unused slots). Values are raw game quantities as
//! `f32` (normalisation is the trainer's business). Stacks are described relative to the viewer
//! ("own" flag), so one policy can play both sides.
//!
//! Globals: `[round, viewer is attacker, viewer to act, battle over]`.
//! Per stack: `[present, alive, own, creature handle, count, top_hp, max_hp, attack, defense,
//! damage_min, damage_max, speed, x, y, acted, waited, retaliated, defending, active]`.

use h3_battle::{Observation, STACK_CAPACITY, StackFlags};
use h3_contracts::BattleSide;

/// Version of this encoding; bump on any change of the layout above.
pub const OBSERVATION_ENCODING_VERSION: u32 = 1;

const GLOBALS: usize = 4;
/// Features per stack slot.
pub const STACK_FEATURES: usize = 19;
/// Length of the encoded observation.
pub const OBSERVATION_LEN: usize = GLOBALS + STACK_FEATURES * STACK_CAPACITY;

/// Hex columns of the encoded field, used to derive `x`/`y` from the hex index.
fn coords(hex: u8, width: u8) -> (f32, f32) {
    let w = width.max(1);
    (f32::from(hex % w), f32::from(hex / w))
}

fn flag(on: bool) -> f32 {
    if on { 1.0 } else { 0.0 }
}

/// Encodes `obs` into `out`. Returns `Err(required_len)` if `out` is too short. `field_width`
/// (from the ruleset) turns hex indices into coordinates.
pub fn encode(obs: &Observation, field_width: u8, out: &mut [f32]) -> Result<(), usize> {
    let out = out.get_mut(..OBSERVATION_LEN).ok_or(OBSERVATION_LEN)?;
    out.fill(0.0);
    let to_act = obs
        .active
        .and_then(|id| obs.stacks.get(id.index()))
        .map(|s| s.side);
    out[0] = f32::from(obs.round);
    out[1] = flag(obs.viewer == BattleSide::Attacker);
    out[2] = flag(to_act == Some(obs.viewer));
    out[3] = flag(obs.winner.is_some());
    for (i, s) in obs.stacks.iter().enumerate() {
        let f = &mut out[GLOBALS + i * STACK_FEATURES..GLOBALS + (i + 1) * STACK_FEATURES];
        let (x, y) = coords(s.hex.0, field_width);
        let values = [
            1.0,
            flag(s.is_alive()),
            flag(s.side == obs.viewer),
            f32::from(s.creature.raw()),
            s.count as f32,
            s.top_hp as f32,
            s.max_hp as f32,
            f32::from(s.attack),
            f32::from(s.defense),
            f32::from(s.damage_min),
            f32::from(s.damage_max),
            f32::from(s.speed),
            x,
            y,
            flag(s.flags.contains(StackFlags::ACTED)),
            flag(s.flags.contains(StackFlags::WAITED)),
            flag(s.flags.contains(StackFlags::RETALIATED)),
            flag(s.flags.contains(StackFlags::DEFENDING)),
            flag(obs.active.is_some_and(|a| a.index() == i)),
        ];
        f.copy_from_slice(&values);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use h3_battle::{BattleState, observe};

    #[test]
    fn encodes_globals_and_stacks() {
        let (r, setup) = h3_pack::synthetic(h3_pack::SYNTHETIC_DUEL).unwrap();
        let s = BattleState::new(&r, &setup, 1).unwrap();
        let obs = observe(&s, BattleSide::Defender);
        let mut out = vec![9.0; OBSERVATION_LEN + 3];
        encode(&obs, r.battle().field.width, &mut out).unwrap();
        assert_eq!(&out[..4], &[1.0, 0.0, 0.0, 0.0]);
        // Stack 0: the attacker's brute ×8 at (1, 5), active.
        let s0 = &out[GLOBALS..GLOBALS + STACK_FEATURES];
        assert_eq!(s0[0..3], [1.0, 1.0, 0.0]);
        assert_eq!(s0[4], 8.0);
        assert_eq!((s0[12], s0[13]), (1.0, 5.0));
        assert_eq!(s0[18], 1.0);
        // Unused slots are zero; the tail beyond OBSERVATION_LEN is untouched.
        assert!(
            out[GLOBALS + 2 * STACK_FEATURES..OBSERVATION_LEN]
                .iter()
                .all(|v| *v == 0.0)
        );
        assert_eq!(out[OBSERVATION_LEN], 9.0);
        let mut short = vec![0.0; 3];
        assert_eq!(encode(&obs, 17, &mut short), Err(OBSERVATION_LEN));
        assert_eq!(OBSERVATION_LEN, 4 + 19 * 32);
        assert_eq!(coords(20, 0), (0.0, 20.0));
    }
}
