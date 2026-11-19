//! The reach block of observation encoding v23 (E25 T38a, AI-Q19; the design: `docs/epics/
//! E25-log.md` T37): what each stack can strike in melee on its next turn, from the battle's state
//! through the viewer-safe query [`h3_battle::reach_seen_by`]. Unlike the rest of the encoding it
//! is not read off the [`Observation`](h3_battle::Observation): it is computed from the state for
//! `viewer`, reading only what `viewer` may know (the positions, speeds, flight, shapes, binds,
//! the gate, the Force Fields, the pits both a stack's controller and `viewer` see, the shooters'
//! state), so two battles that differ only in what `viewer` may not know (a world,
//! `BattleState::determinized`) give the same block.
//!
//! A *striker* is a living stack, not a war machine or a town's turret, whose turns are not lost
//! (not blind, petrified or paralysed: BAT-MAG-024, BAT-MAG-043); a shooter too (whether it can
//! shoot now is its slot's first feature). Its *strike hexes* are the hexes next to its footprint
//! at a position it attacks from this turn, as its legal attacks: where it stands, or a position
//! of its reach where its walk does not stop (BAT-MAG-032), its own footprint there left out,
//! standable hexes only. A stack standing on a strike hex can be struck in melee (BAT-ATK-001:
//! the footprints touch); the counts take the stacks as they stand (a stack on the hex itself
//! would block the walk there).

use h3_battle::{
    BattleState, Field, Footprint, Hex, HexSet, STACK_CAPACITY, Stack, StackId, StackTraits,
    Status, computer, reach_seen_by,
};
use h3_content_battle::{BattleRuleset, MAX_FIELD_HEXES};
use h3_contracts::BattleSide;

/// Features per stack slot of the reach block.
pub const REACH_FEATURES: usize = 3;

/// The reach block of one viewer's observation (the module's documentation): per hex, for the
/// viewer's side and then the opponent's (by the stacks' controllers, BAT-MAG-026), the number of
/// the side's strikers that can strike a stack standing on it; per stack slot `[can shoot now
/// (BAT-RNG-001), the stacks it may attack (BAT-MAG-025, BAT-MAG-026: of the side its controller
/// fights, no turret) it can strike, the strikers that may attack it and can strike it]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReachBlock {
    planes: [[u8; MAX_FIELD_HEXES]; 2],
    slots: [[u8; REACH_FEATURES]; STACK_CAPACITY],
}

impl ReachBlock {
    /// No reach: the block where no battle state is at hand (zeros).
    pub const EMPTY: Self = Self {
        planes: [[0; MAX_FIELD_HEXES]; 2],
        slots: [[0; REACH_FEATURES]; STACK_CAPACITY],
    };

    /// The block of `state` (whose ruleset is `ruleset`) for `viewer`.
    pub fn of(state: &BattleState, ruleset: &BattleRuleset, viewer: BattleSide) -> Self {
        Self::with(&Neighbours::new(ruleset), state, ruleset, viewer)
    }

    /// [`Self::of`] with the field's neighbour table `around` (`ruleset`'s, made once).
    // rule: BAT-MOV-001, BAT-MOV-002, BAT-ATK-001, BAT-RNG-001, BAT-MAG-031, BAT-MAG-032
    pub fn with(
        around: &Neighbours,
        state: &BattleState,
        ruleset: &BattleRuleset,
        viewer: BattleSide,
    ) -> Self {
        let field = Field::new(ruleset.battle().field);
        let stacks = state.stacks();
        let mut block = Self::EMPTY;
        let mut strikes = [HexSet::new(); STACK_CAPACITY];
        let mut own = [HexSet::new(); STACK_CAPACITY];
        for ((s, id), (slot, (strike, own))) in stacks.iter().zip((0u8..).map(StackId)).zip(
            block
                .slots
                .iter_mut()
                .zip(strikes.iter_mut().zip(own.iter_mut())),
        ) {
            if !s.is_alive() {
                continue;
            }
            *own = hexes(s.footprint(field));
            slot[0] = u8::from(computer::shoots(state, ruleset, id));
            if !strikes_in_melee(s) {
                continue;
            }
            let reach = reach_seen_by(state, ruleset, id, viewer);
            *strike = around.strike_hexes(field, s, reach.positions.without(reach.stops));
            let plane = &mut block.planes[usize::from(s.controller() != viewer)];
            for h in strike.iter() {
                plane[h.index()] += 1;
            }
        }
        // Each striker `i` and each stack `j` it may attack and strikes: once in either's slot.
        for (i, (s, strike)) in stacks.iter().zip(strikes.iter()).enumerate() {
            if strike.is_empty() {
                continue;
            }
            for (j, (t, hexes)) in stacks.iter().zip(own.iter()).enumerate() {
                if i != j && t.is_alive() && s.may_attack(t) && strike.intersects(*hexes) {
                    block.slots[i][1] += 1;
                    block.slots[j][2] += 1;
                }
            }
        }
        block
    }

    /// Writes the block into `out` (its length, the end of the encoding).
    pub(super) fn write(&self, out: &mut [f32]) {
        let (planes, slots) = out.split_at_mut(2 * MAX_FIELD_HEXES);
        for (o, &v) in planes.iter_mut().zip(self.planes.iter().flatten()) {
            *o = f32::from(v);
        }
        for (o, &v) in slots.iter_mut().zip(self.slots.iter().flatten()) {
            *o = f32::from(v);
        }
    }
}

/// Whether the living stack `s` is a striker (the module's documentation).
fn strikes_in_melee(s: &Stack) -> bool {
    !s.has(StackTraits::WAR_MACHINE)
        && !s.is_turret()
        && !s.spells.status.contains(Status::NO_TURNS)
}

/// Each hex's standable neighbours as a set, on one ruleset's field: the strike hexes' table (a
/// union per position), made once per environment ([`ReachBlock::with`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Neighbours([HexSet; MAX_FIELD_HEXES]);

impl Neighbours {
    /// The table of `ruleset`'s field.
    pub fn new(ruleset: &BattleRuleset) -> Self {
        let field = Field::new(ruleset.battle().field);
        let mut table = [HexSet::new(); MAX_FIELD_HEXES];
        for (set, hex) in table
            .iter_mut()
            .zip((0u8..).map(Hex))
            .take(field.hex_count())
        {
            for n in field.neighbors(hex).into_iter().flatten() {
                if field.is_standable(n) {
                    set.insert(n);
                }
            }
        }
        Self(table)
    }

    /// The standable neighbours of `hex`.
    fn of(&self, hex: Hex) -> HexSet {
        self.0.get(hex.index()).copied().unwrap_or_default()
    }

    /// The strike hexes of `s` from the positions `from`: next to its footprint at one of them,
    /// that footprint left out (a one-hex stack's position is no neighbour of itself).
    fn strike_hexes(&self, field: Field, s: &Stack, from: HexSet) -> HexSet {
        if !s.is_two_hex() {
            return from
                .iter()
                .fold(HexSet::new(), |set, p| set.union(self.of(p)));
        }
        // A position of the reach has its footprint on the field.
        from.iter()
            .filter_map(|p| field.footprint(p, true, s.side))
            .fold(HexSet::new(), |set, fp| {
                let near = fp.hexes().fold(HexSet::new(), |n, h| n.union(self.of(h)));
                set.union(near.without(hexes(fp)))
            })
    }
}

/// The hexes of `fp` as a set.
fn hexes(fp: Footprint) -> HexSet {
    let mut set = HexSet::new();
    fp.hexes().for_each(|h| set.insert(h));
    set
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use h3_battle::{CastTarget, Command, LegalCommands, legal_commands};
    use h3_content_battle::ObstacleKind;
    use h3_kernel::{Rng, StreamId};

    use super::*;
    use crate::BattleEnv;
    use crate::observation::{OBSERVATION_LEN, REACH};
    use crate::rollout::{pick, step_command};

    /// Plays random battles of `scenario` (seeds `1..=battles`) and calls `each` at every decision.
    /// A side's hero casts the hidden obstacles (the synthetic pack's Mire, a Quicksand, and
    /// Snare, a Land Mine) whenever it can, the rest is random.
    fn decisions(scenario: &str, battles: u64, mut each: impl FnMut(&BattleEnv)) {
        let (r, setup) = h3_pack_battle::synthetic(scenario).unwrap();
        let hidden = ["synthetic:mire", "synthetic:snare"].map(|k| Command::Cast {
            spell: r.spell_by_key(k).unwrap(),
            target: CastTarget::None,
            second: CastTarget::None,
        });
        let (r, setup) = (Arc::new(r), Arc::new(setup));
        let mut legal = LegalCommands::new();
        for seed in 1..=battles {
            let mut env = BattleEnv::new(r.clone(), setup.clone(), seed, 400).unwrap();
            let mut rng = Rng::for_stream(seed, StreamId::AGENT);
            while let Some(side) = env.to_move() {
                each(&env);
                if hidden
                    .iter()
                    .any(|&c| step_command(&mut env, side, c).is_ok())
                {
                    continue;
                }
                let command = pick(&env, side, &mut rng, &mut legal);
                let _ = step_command(&mut env, side, command);
            }
        }
    }

    /// E25 T38a: the block agrees with the rules' commands where the viewer is the active stack's
    /// controller (its reach then is the rules'): the shot flag with the shots, a striker's
    /// strikes with its attacks' targets, whose hexes its side's plane counts; each pair counts
    /// once from either side.
    // covers: BAT-ATK-001, BAT-RNG-001, BAT-MOV-001
    #[test]
    fn the_block_agrees_with_the_active_stacks_commands() {
        let mut strikes = 0;
        for scenario in [
            h3_pack_battle::SYNTHETIC_FULL,
            h3_pack_battle::SYNTHETIC_SIEGE,
        ] {
            decisions(scenario, 2, |env| {
                let (s, r) = (env.state(), env.ruleset());
                let side = env.to_move().unwrap();
                let block = ReachBlock::of(s, r, side);
                let total = |k: usize| block.slots.iter().map(|f| u32::from(f[k])).sum::<u32>();
                assert_eq!(total(1), total(2));
                let obs = env.observation(side);
                let (Some(active), None) = (obs.active, obs.tactics) else {
                    return;
                };
                let mut legal = LegalCommands::new();
                legal_commands(s, r, side, &mut legal);
                let slot = block.slots[active.index()];
                let shots = legal.iter().any(|c| matches!(c, Command::Shoot { .. }));
                assert_eq!(slot[0] == 1, shots);
                let mut targets = HexSet::new();
                let field = Field::new(r.battle().field);
                for c in legal.iter() {
                    if let Command::Attack { target, .. } = *c {
                        targets.insert(Hex(target.0));
                        let fp = s.stack(target).unwrap().footprint(field);
                        assert!(fp.hexes().any(|h| block.planes[0][h.index()] > 0));
                    }
                }
                let me = s.stack(active).unwrap();
                if strikes_in_melee(me) && !shots {
                    assert_eq!(u32::from(slot[1]), targets.len());
                    strikes += u32::from(slot[1]);
                }
            });
        }
        assert!(strikes > 200, "{strikes}");
        // At the duel's start the two stacks stand apart: each has strike hexes, none a target.
        let (r, setup) = h3_pack_battle::synthetic(h3_pack_battle::SYNTHETIC_DUEL).unwrap();
        let s = BattleState::new(&r, &setup, 1).unwrap();
        let block = ReachBlock::of(&s, &r, BattleSide::Attacker);
        assert!(block.slots.iter().all(|f| f[1] == 0 && f[2] == 0));
        for plane in &block.planes {
            assert!(plane.contains(&1) && plane.iter().all(|&n| n <= 1));
        }
        let mut out = [1.0; OBSERVATION_LEN];
        block.write(&mut out[REACH..]);
        let planes = &out[REACH..REACH + 2 * MAX_FIELD_HEXES];
        assert_eq!(
            planes.iter().sum::<f32>(),
            block.planes.iter().flatten().map(|&n| f32::from(n)).sum()
        );
        assert!(out[REACH + 2 * MAX_FIELD_HEXES..].iter().all(|&v| v == 0.0));
    }

    /// E25 T38a (AI-Q19): a viewer's worlds of a battle (`BattleEnv::world`, the other side's
    /// hidden pits and mines placed again) encode as the battle does for it, the reach block too.
    // covers: BAT-MAG-031
    #[test]
    fn a_viewers_worlds_encode_as_its_battle() {
        let mut hidden = 0;
        let (mut a, mut b) = ([0.0; OBSERVATION_LEN], [0.0; OBSERVATION_LEN]);
        decisions(h3_pack_battle::SYNTHETIC_FULL, 2, |env| {
            for viewer in [BattleSide::Attacker, BattleSide::Defender] {
                env.encode_observation(viewer, &mut a).unwrap();
                for seed in [1, 2] {
                    let world = env.world(viewer, None, seed).unwrap();
                    world.encode_observation(viewer, &mut b).unwrap();
                    assert!(a == b, "{viewer:?} {seed}");
                }
                hidden += usize::from(
                    env.state()
                        .spell_obstacles()
                        .iter()
                        .any(|o| o.kind == ObstacleKind::Pit && o.side != viewer && !o.revealed),
                );
            }
        });
        assert!(hidden > 500, "{hidden}");
    }
}
