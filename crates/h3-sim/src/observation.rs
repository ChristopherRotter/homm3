//! Flat observation encoding (version 4) of a per-viewer [`Observation`].
//!
//! Layout: [`GLOBALS`] global features, then two hero blocks of [`HERO_FEATURES`] features (the
//! viewer's, then the opponent's), then [`STACK_FEATURES`] features for each of the
//! [`STACK_CAPACITY`] stack slots (zeros for unused slots). Values are raw game quantities as
//! `f32` (normalisation is the trainer's business). Stacks and sides are described relative to
//! the viewer ("own" flag, "viewer"/"opponent"), so one policy can play both sides.
//!
//! Globals: `[round (0 in the tactics phase), viewer is attacker, viewer to act, battle over,
//! viewer places in a tactics phase, opponent places in a tactics phase, the phase's zone in
//! columns, viewer may retreat, opponent may retreat, viewer may surrender at all, opponent may
//! surrender at all]` (BAT-TAC-001, BAT-FLE-001, BAT-FLE-002).
//! Hero block, zeros for a side without a hero (which gets no lucky strikes in profiles with
//! quirk Q-0003): the [`HERO_PUBLIC_FEATURES`] public features `[led by a hero, hero type handle,
//! attack, defense, power, knowledge, mana, max mana, commands its Ballista, commands its First
//! Aid Tent]`, then the private ones, zeros in the opponent's block (ADR-0013 point 9):
//! `[Tactics level, ranged bonus bp, melee bonus bp, damage reduction bp, the commanded
//! Ballista's shots per turn, its double-damage chance bp, the commanded tent's heal bonus bp,
//! surrender discount bp, funds, surrender price (0 if the side may not surrender)]`.
//! Per stack: `[present, alive, own, creature handle, count, top_hp, max_hp, attack, defense,
//! damage_min, damage_max, speed, x, y, acted, waited, retaliated, defending, active, morale,
//! luck, shots, retaliations left (255 = unlimited), had good morale this round, two-hex,
//! flies, shooter, war machine, no distance penalty, shoots when adjacent, the hero's
//! regeneration in hit points]` (the last three can come from the side's hero, BAT-HER-006).
//!
//! v3 (E03 T9): the two hero globals and the per-stack features from `morale` on. v4 (E04a
//! T11b): the tactics, retreat and surrender globals, the hero blocks (instead of the two hero
//! globals) and the last three per-stack features.

use h3_battle::{HeroView, Observation, OwnHeroView, STACK_CAPACITY, StackFlags, StackTraits};
use h3_contracts::BattleSide;

/// Version of this encoding; bump on any change of the layout above (and bump the C ABI
/// version with it, ADR-0011).
pub const OBSERVATION_ENCODING_VERSION: u32 = 4;

/// Global features.
pub const GLOBALS: usize = 11;
/// Features per hero block.
pub const HERO_FEATURES: usize = 20;
/// The public features at the start of a hero block; the rest are private to the hero's side.
pub const HERO_PUBLIC_FEATURES: usize = 10;
/// Features per stack slot.
pub const STACK_FEATURES: usize = 31;
/// Offset of the viewer's hero block; the opponent's follows it.
pub const HEROES: usize = GLOBALS;
/// Offset of the first stack slot.
pub const STACKS: usize = HEROES + 2 * HERO_FEATURES;
/// Length of the encoded observation.
pub const OBSERVATION_LEN: usize = STACKS + STACK_FEATURES * STACK_CAPACITY;

/// Hex columns of the encoded field, used to derive `x`/`y` from the hex index.
fn coords(hex: u8, width: u8) -> (f32, f32) {
    let w = width.max(1);
    (f32::from(hex % w), f32::from(hex / w))
}

fn flag(on: bool) -> f32 {
    if on { 1.0 } else { 0.0 }
}

/// Writes a hero block into `out`: the public part `view` and, for the viewer's own hero only,
/// the private part `own`.
fn encode_hero(view: Option<HeroView>, own: Option<OwnHeroView>, out: &mut [f32]) {
    if let Some(h) = view {
        out[..HERO_PUBLIC_FEATURES].copy_from_slice(&[
            1.0,
            f32::from(h.hero_type.raw()),
            f32::from(h.attack),
            f32::from(h.defense),
            f32::from(h.power),
            f32::from(h.knowledge),
            f32::from(h.mana),
            f32::from(h.max_mana),
            flag(h.commands_ballista),
            flag(h.commands_tent),
        ]);
    }
    if let Some(h) = own {
        out[HERO_PUBLIC_FEATURES..].copy_from_slice(&[
            f32::from(h.tactics),
            f32::from(h.damage.ranged_bp),
            f32::from(h.damage.melee_bp),
            f32::from(h.damage.reduction_bp),
            h.ballista.map_or(0.0, |b| f32::from(b.shots)),
            h.ballista
                .map_or(0.0, |b| f32::from(b.double_damage_chance_bp)),
            h.tent.map_or(0.0, |t| t.heal_bonus_bp as f32),
            f32::from(h.surrender_discount_bp),
            h.funds as f32,
            h.surrender_price.map_or(0.0, |p| p as f32),
        ]);
    }
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
    let (own, other) = (obs.viewer.index(), obs.viewer.opponent().index());
    let phase = obs.tactics;
    out[..GLOBALS].copy_from_slice(&[
        f32::from(obs.round),
        flag(obs.viewer == BattleSide::Attacker),
        flag(to_act == Some(obs.viewer)),
        flag(obs.result.is_some()),
        flag(phase.is_some_and(|p| p.side == obs.viewer)),
        flag(phase.is_some_and(|p| p.side != obs.viewer)),
        phase.map_or(0.0, |p| f32::from(p.columns)),
        flag(obs.may_retreat[own]),
        flag(obs.may_retreat[other]),
        flag(obs.surrender_possible[own]),
        flag(obs.surrender_possible[other]),
    ]);
    let (viewer, opponent) = out[HEROES..STACKS].split_at_mut(HERO_FEATURES);
    encode_hero(obs.heroes[own], obs.own_hero, viewer);
    encode_hero(obs.heroes[other], None, opponent);
    for (i, s) in obs.stacks.iter().enumerate() {
        let f = &mut out[STACKS + i * STACK_FEATURES..STACKS + (i + 1) * STACK_FEATURES];
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
            flag(s.has(StackTraits::NO_DISTANCE_PENALTY)),
            flag(s.has(StackTraits::SHOOTS_WHEN_ADJACENT)),
            f32::from(s.regeneration),
        ];
        f.copy_from_slice(&values);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use h3_battle::{BattleState, observe};

    /// The features of stack `i` in an encoded observation.
    fn stack(out: &[f32], i: usize) -> &[f32] {
        &out[STACKS + i * STACK_FEATURES..STACKS + (i + 1) * STACK_FEATURES]
    }

    /// The viewer's (0) or the opponent's (1) hero block.
    fn hero(out: &[f32], block: usize) -> &[f32] {
        &out[HEROES + block * HERO_FEATURES..HEROES + (block + 1) * HERO_FEATURES]
    }

    #[test]
    fn encodes_globals_and_stacks() {
        let (r, setup) = h3_pack::synthetic(h3_pack::SYNTHETIC_DUEL).unwrap();
        let s = BattleState::new(&r, &setup, 1).unwrap();
        let obs = observe(&s, &r, BattleSide::Defender);
        let mut out = vec![9.0; OBSERVATION_LEN + 3];
        encode(&obs, r.battle().field.width, &mut out).unwrap();
        // Round 1, the defender's view, the attacker to act; no tactics phase, no heroes, so no
        // retreat or surrender.
        assert_eq!(
            out[..GLOBALS],
            [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]
        );
        assert!(out[HEROES..STACKS].iter().all(|v| *v == 0.0));
        // Stack 0: the attacker's two-hex brute ×8, head at (2, 5) (tail in column 1), active.
        let s0 = stack(&out, 0);
        assert_eq!(s0[0..3], [1.0, 1.0, 0.0]);
        assert_eq!(s0[4], 8.0);
        assert_eq!((s0[12], s0[13]), (2.0, 5.0));
        assert_eq!(s0[18], 1.0);
        // Morale +1 (one-faction army, BAT-MOR-002), luck 0, no shots, one retaliation, two-hex
        // walker, nothing from a hero.
        assert_eq!(
            s0[19..],
            [1.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]
        );
        // Unused slots are zero; the tail beyond OBSERVATION_LEN is untouched.
        assert!(
            out[STACKS + 2 * STACK_FEATURES..OBSERVATION_LEN]
                .iter()
                .all(|v| *v == 0.0)
        );
        assert_eq!(out[OBSERVATION_LEN], 9.0);
        let mut short = vec![0.0; 3];
        assert_eq!(encode(&obs, 17, &mut short), Err(OBSERVATION_LEN));
        assert_eq!(OBSERVATION_LEN, 11 + 2 * 20 + 31 * 38);
        assert_eq!(coords(20, 0), (0.0, 20.0));
    }

    #[test]
    fn encodes_the_heroes_and_shows_the_private_part_only_to_its_side() {
        let (r, setup) = h3_pack::synthetic(h3_pack::SYNTHETIC_FULL).unwrap();
        let s = BattleState::new(&r, &setup, 1).unwrap();
        let mut out = vec![0.0; OBSERVATION_LEN];
        // Only the attacker is led by a hero (Brea, `full.toml`); her Tactics give her a tactics
        // phase with a five-column zone, she may retreat, and nobody may surrender.
        let brea = f32::from(r.hero_type_by_key("synthetic:brea").unwrap().raw());
        let public = [1.0, brea, 2.0, 2.0, 1.0, 1.0, 10.0, 10.0, 1.0, 1.0];
        let private = [
            2.0, 1000.0, 1000.0, 500.0, 1.0, 5000.0, 10_000.0, 0.0, 0.0, 0.0,
        ];
        encode(&observe(&s, &r, BattleSide::Attacker), 17, &mut out).unwrap();
        assert_eq!(
            out[..GLOBALS],
            [0.0, 1.0, 1.0, 0.0, 1.0, 0.0, 5.0, 1.0, 0.0, 0.0, 0.0]
        );
        assert_eq!(hero(&out, 0)[..HERO_PUBLIC_FEATURES], public);
        assert_eq!(hero(&out, 0)[HERO_PUBLIC_FEATURES..], private);
        assert!(hero(&out, 1).iter().all(|v| *v == 0.0));
        // The defender sees her public part only.
        encode(&observe(&s, &r, BattleSide::Defender), 17, &mut out).unwrap();
        assert_eq!(
            out[..GLOBALS],
            [0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 5.0, 0.0, 1.0, 0.0, 0.0]
        );
        assert!(hero(&out, 0).iter().all(|v| *v == 0.0));
        assert_eq!(hero(&out, 1)[..HERO_PUBLIC_FEATURES], public);
        assert!(
            hero(&out, 1)[HERO_PUBLIC_FEATURES..]
                .iter()
                .all(|v| *v == 0.0)
        );
        // The surrender possibilities and the own price, when there is one.
        let mut obs = observe(&s, &r, BattleSide::Attacker);
        obs.surrender_possible = [true, false];
        obs.own_hero.as_mut().unwrap().surrender_price = Some(1472);
        obs.own_hero.as_mut().unwrap().funds = 5000;
        encode(&obs, 17, &mut out).unwrap();
        assert_eq!(out[9..GLOBALS], [1.0, 0.0]);
        assert_eq!(hero(&out, 0)[HERO_FEATURES - 2..], [5000.0, 1472.0]);
    }

    #[test]
    fn encodes_morale_luck_shots_war_machines_and_traits_from_heroes() {
        let (r, setup) = h3_pack::synthetic(h3_pack::SYNTHETIC_FULL).unwrap();
        let s = BattleState::new(&r, &setup, 1).unwrap();
        let mut out = vec![0.0; OBSERVATION_LEN];
        encode(&observe(&s, &r, BattleSide::Attacker), 17, &mut out).unwrap();
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
        // Morale, luck, the good-morale flag and the hero's traits come from the stacks.
        let mut obs = observe(&s, &r, BattleSide::Attacker);
        let st = &mut obs.stacks.as_mut_slice()[0];
        st.morale = 2;
        st.luck = -1;
        st.flags.set(StackFlags::HAD_MORALE, true);
        st.traits = st
            .traits
            .with(StackTraits::NO_DISTANCE_PENALTY, true)
            .with(StackTraits::SHOOTS_WHEN_ADJACENT, true);
        st.regeneration = 50;
        encode(&obs, 17, &mut out).unwrap();
        assert_eq!(stack(&out, 0)[19..24], [2.0, -1.0, 0.0, 1.0, 1.0]);
        assert_eq!(stack(&out, 0)[28..], [1.0, 1.0, 50.0]);
    }
}
