//! Flat observation encoding (version 3) of a per-viewer [`Observation`].
//!
//! Layout: [`GLOBALS`] global features, then [`STACK_FEATURES`] features for each of the
//! [`STACK_CAPACITY`] stack slots (zeros for unused slots). Values are raw game quantities as
//! `f32` (normalisation is the trainer's business). Stacks and sides are described relative to
//! the viewer ("own" flag, "viewer"/"opponent"), so one policy can play both sides.
//!
//! Globals: `[round, viewer is attacker, viewer to act, battle over, viewer led by a hero,
//! opponent led by a hero]` (without a hero a side gets no lucky strikes in profiles with quirk
//! Q-0003).
//! Per stack: `[present, alive, own, creature handle, count, top_hp, max_hp, attack, defense,
//! damage_min, damage_max, speed, x, y, acted, waited, retaliated, defending, active, morale,
//! luck, shots, retaliations left (255 = unlimited), had good morale this round, two-hex,
//! flies, shooter, war machine]`.
//!
//! v3 (E03 T9): the two hero globals and the per-stack features from `morale` on.

use h3_battle::{Observation, STACK_CAPACITY, StackFlags, StackTraits};
use h3_contracts::BattleSide;

/// Version of this encoding; bump on any change of the layout above (and bump the C ABI
/// version with it, ADR-0011).
pub const OBSERVATION_ENCODING_VERSION: u32 = 3;

/// Global features.
pub const GLOBALS: usize = 6;
/// Features per stack slot.
pub const STACK_FEATURES: usize = 28;
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
    out[3] = flag(obs.result.is_some());
    out[4] = flag(obs.led_by_hero[obs.viewer.index()]);
    out[5] = flag(obs.led_by_hero[obs.viewer.opponent().index()]);
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
            f32::from(s.morale),
            f32::from(s.luck),
            f32::from(s.shots),
            f32::from(s.retaliations_left),
            flag(s.flags.contains(StackFlags::HAD_MORALE)),
            flag(s.has(StackTraits::TWO_HEX)),
            flag(s.has(StackTraits::FLIES)),
            flag(s.has(StackTraits::SHOOTER)),
            flag(s.has(StackTraits::WAR_MACHINE)),
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
        assert_eq!(&out[..GLOBALS], &[1.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
        // Stack 0: the attacker's two-hex brute ×8, head at (2, 5) (tail in column 1), active.
        let s0 = &out[GLOBALS..GLOBALS + STACK_FEATURES];
        assert_eq!(s0[0..3], [1.0, 1.0, 0.0]);
        assert_eq!(s0[4], 8.0);
        assert_eq!((s0[12], s0[13]), (2.0, 5.0));
        assert_eq!(s0[18], 1.0);
        // Morale +1 (one-faction army, BAT-MOR-002), luck 0, no shots, one retaliation, two-hex
        // walker.
        assert_eq!(s0[19..], [1.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0]);
        // Unused slots are zero; the tail beyond OBSERVATION_LEN is untouched.
        assert!(
            out[GLOBALS + 2 * STACK_FEATURES..OBSERVATION_LEN]
                .iter()
                .all(|v| *v == 0.0)
        );
        assert_eq!(out[OBSERVATION_LEN], 9.0);
        let mut short = vec![0.0; 3];
        assert_eq!(encode(&obs, 17, &mut short), Err(OBSERVATION_LEN));
        assert_eq!(OBSERVATION_LEN, 6 + 28 * 38);
        assert_eq!(coords(20, 0), (0.0, 20.0));
    }

    /// The features of stack `i` in an encoded observation.
    fn stack(out: &[f32], i: usize) -> &[f32] {
        &out[GLOBALS + i * STACK_FEATURES..GLOBALS + (i + 1) * STACK_FEATURES]
    }

    #[test]
    fn encodes_heroes_morale_luck_shots_and_war_machines() {
        let (r, setup) = h3_pack::synthetic(h3_pack::SYNTHETIC_FULL).unwrap();
        let s = BattleState::new(&r, &setup, 1).unwrap();
        let mut out = vec![0.0; OBSERVATION_LEN];
        // Only the attacker is led by a hero; the globals are relative to the viewer.
        for (viewer, heroes) in [
            (BattleSide::Attacker, [1.0, 0.0]),
            (BattleSide::Defender, [0.0, 1.0]),
        ] {
            encode(&observe(&s, viewer), 17, &mut out).unwrap();
            assert_eq!(out[4..6], heroes);
        }
        // Ids: attacker army 0..=6, its bolt thrower 7 and aid tent 8; defender army 9..=15,
        // its bolt thrower 16 and supply wagon 17.
        let kinds: Vec<[f32; 4]> = (0..18)
            .map(|i| {
                let f = stack(&out, i);
                [f[24], f[25], f[26], f[27]]
            })
            .collect();
        assert_eq!(kinds[2], [0.0, 1.0, 1.0, 0.0], "wisp: flies and shoots");
        assert_eq!(kinds[7], [1.0, 0.0, 1.0, 1.0], "bolt thrower");
        assert_eq!(kinds[8], [1.0, 0.0, 0.0, 1.0], "aid tent");
        assert_eq!(kinds[17], [0.0, 0.0, 0.0, 1.0], "supply wagon");
        assert_eq!(stack(&out, 7)[21], 12.0, "the bolt thrower's shots");
        // Morale, luck and the good-morale flag come from the stacks.
        let mut obs = observe(&s, BattleSide::Attacker);
        let st = &mut obs.stacks.as_mut_slice()[0];
        st.morale = 2;
        st.luck = -1;
        st.flags.set(StackFlags::HAD_MORALE, true);
        encode(&obs, 17, &mut out).unwrap();
        assert_eq!(stack(&out, 0)[19..24], [2.0, -1.0, 0.0, 1.0, 1.0]);
    }
}
