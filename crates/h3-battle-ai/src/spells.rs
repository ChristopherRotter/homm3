//! The computer's spell context and its spell values (0x4369C0, 0x43A910, the evaluators of
//! 0x43B690; `docs/formats/installation.md` §7.42, §7.44 to §7.46, §7.51; BAT-AI-014): what a cast
//! on a stack is worth to the side. The creature casts that weigh a spell (the Ogre Mage's scan,
//! [`casts`](crate::casts)) build the context and ask its evaluator for each stack.
//!
//! So far (E23 T4b1, T4b2) the context holds the side's battle context, its balance class, "the
//! enemy beaten" after the predictions ([`predict`]: the damage the side's stacks that act before
//! the other side's next one are expected to deal, with their draws) and each stack's threat
//! record (0x422B20 with flag 0, [`computer::threat`]), and one evaluator, Bloodlust's: the attack
//! bonus's value (0x437800). Not built yet: the threat tables and the second context (0x43C040,
//! E23 T4c) and the other evaluators (E23 T4c).

use core::cell::OnceCell;

use h3_battle::computer::{self, Context, Planned, Threat};
use h3_battle::{
    BattleSide, BattleState, STACK_CAPACITY, StackFlags, StackId, StackTraits, Status,
};
use h3_content::SpellId;
use h3_content_battle::BattleRuleset;
use h3_kernel::Rng;
use h3_kernel::arith::{DOUBLE_BITS, X87};

use crate::casts::balance;
use crate::predict;

/// The base damage the attack bonus's value compares (0x437800's 100).
const BASE: u64 = 100;

/// A cast the computer weighs (the cast candidate, §7.44): the spell, its mastery and the rounds
/// its timed effects last.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Candidate {
    pub(crate) spell: SpellId,
    pub(crate) mastery: u8,
    pub(crate) rounds: u32,
}

/// An evaluator (0x43B690's table): the value of a candidate on a stack.
pub(crate) type Evaluator = fn(&SpellContext<'_>, StackId, Candidate) -> i32;

/// The computer's spell context of a side (0x4369C0 with the creature casts' flag 1; E23 T4b1).
pub(crate) struct SpellContext<'a> {
    state: &'a BattleState,
    ruleset: &'a BattleRuleset,
    /// The side (+0xC).
    side: BattleSide,
    /// The side's battle context (+0x20, 0x435EC0).
    ctx: &'a Context,
    /// Its balance class (+0x3C, the battle context's +0x1C).
    class: u8,
    /// The enemy beaten (+0x1C): no stack of the other side but the war machines has hit points
    /// left after the planned damage.
    beaten: bool,
    /// Each stack's threat record (0x422B20 with flag 0), computed the first time it is asked
    /// (ADR-0027 point 11: the records read nothing a value changes).
    threats: [OnceCell<Option<Threat>>; STACK_CAPACITY],
}

impl<'a> SpellContext<'a> {
    /// The spell context of `side` with its battle context `ctx` (0x4369C0; BAT-AI-015): its
    /// balance class ([`balance`]), then the predictions ([`predict::plan`], their walkers' draws
    /// from `rng`) and "the enemy beaten" after them ([`beaten`]).
    // rule: BAT-AI-015
    pub(crate) fn new(
        state: &'a BattleState,
        ruleset: &'a BattleRuleset,
        (ctx, side): (&'a Context, BattleSide),
        rng: &mut Rng,
    ) -> Self {
        let planned = predict::plan(state, ruleset, side, rng);
        Self {
            state,
            ruleset,
            side,
            ctx,
            class: balance(state, ruleset, ctx, side).class,
            beaten: beaten(state, &planned, side),
            threats: [const { OnceCell::new() }; STACK_CAPACITY],
        }
    }

    /// Whether the predictions beat the enemy (+0x1C).
    #[cfg(test)]
    pub(crate) fn beaten(&self) -> bool {
        self.beaten
    }

    /// The threat record of stack `id` ([`computer::threat`]), computed once.
    fn threat(&self, id: StackId) -> Option<Threat> {
        *self.threats[id.index()]
            .get_or_init(|| computer::threat(self.state, self.ruleset, self.ctx, id))
    }

    /// The enemy of stack `s`'s threat record when `s` attacks it within a turn (0x448BD0 at its
    /// speed: at most 1), else none.
    fn within_a_turn(&self, s: StackId) -> Option<StackId> {
        self.threat(s).filter(|t| t.turns <= 1).map(|t| t.enemy)
    }

    /// The value of `bonus` more attack for stack `s` against `e` over `rounds` rounds (0x437800;
    /// BAT-AI-014): 0 when the predictions beat the enemy (BAT-AI-015); else the gain `g = D' / D`, `D` and `D'` the damage of a base of 100 on `e` without
    /// and with the bonus ([`computer::base_damage`], `s`'s shot flag), doubles; `s`'s strike `d`
    /// by its count on `e` (capped at `e`'s hit points `h`, no run-up) rises to `d' = trunc(d ×
    /// g)`, at most `h` (then `g = h / d`); 0 unless `d' > d`. The rounds' share `t = min(1,
    /// rounds / k)`, `k` the balance class, less `1 / k` (at least 0) for a stack that acted this
    /// round. The value is `trunc((√g − 1) × w × t)`, `w` the strength of `s` (0x442E60): the rise
    /// of its worth, whose offense enters under a square root, for the share of the battle's
    /// expected length the bonus lasts. As the original's x87 doubles.
    // rule: BAT-AI-014
    pub(crate) fn attack_bonus(&self, (s, e): (StackId, StackId), rounds: u32, bonus: u16) -> i32 {
        if self.beaten {
            return 0;
        }
        let (state, ruleset) = (self.state, self.ruleset);
        let shot = computer::shoots(state, ruleset, s);
        let bonus = i16::try_from(bonus).unwrap_or(i16::MAX);
        let base = |b| {
            X87::int(computer::base_damage(
                state,
                ruleset,
                (s, e),
                (BASE, shot),
                b,
            ))
        };
        let mut g = base(bonus)
            .over(base(0))
            .unwrap_or(X87::MAX)
            .round(DOUBLE_BITS);
        let count = u64::from(state.stacks()[s.index()].count);
        let d = computer::damage_estimate(state, ruleset, (s, e), count, shot, 0);
        let mut boosted = computer::ftol(X87::int(d).times(g));
        let h = computer::hit_points(state, e);
        let ih = i32::try_from(h).unwrap_or(i32::MAX);
        if boosted > ih {
            boosted = ih;
            g = X87::int(h).over(X87::int(d)).unwrap_or(X87::MAX);
        }
        if i64::from(boosted) <= i64::try_from(d).unwrap_or(i64::MAX) {
            return 0;
        }
        let k = X87::int(u64::from(self.class));
        let mut t = if rounds >= u32::from(self.class) {
            X87::ONE
        } else {
            X87::int(u64::from(rounds)).over(k).unwrap_or(X87::ONE)
        };
        if state.stacks()[s.index()].flags.contains(StackFlags::ACTED) {
            // `minus` stops at 0, as the original's floor.
            t = t.minus(X87::ONE.over(k).unwrap_or(X87::ZERO));
        }
        let w = computer::strength(state, ruleset, self.ctx, s);
        let w = X87::int(u64::try_from(w).unwrap_or(0));
        computer::ftol(g.sqrt().minus(X87::ONE).times(w).times(t))
    }

    /// Bloodlust's evaluator (0x438100; BAT-AI-014): 0 for a stack `s` that can shoot now or that
    /// does not attack its threat record's enemy within a turn; else the attack bonus's value
    /// ([`Self::attack_bonus`]) against that enemy over the candidate's rounds with the spell's
    /// value at its mastery.
    // rule: BAT-AI-014
    pub(crate) fn bloodlust(&self, s: StackId, c: Candidate) -> i32 {
        if computer::shoots(self.state, self.ruleset, s) {
            return 0;
        }
        let bonus = self
            .ruleset
            .spell(c.spell)
            .map_or(0, |sp| sp.effect[usize::from(c.mastery.min(3))]);
        self.within_a_turn(s)
            .map_or(0, |e| self.attack_bonus((s, e), c.rounds, bonus))
    }

    /// The value of the candidate `c` cast on the side's whole army (0x43A910's mass path; BAT-AI-014):
    /// over the side's stacks in record order that can act (alive, not blind, turned to stone or
    /// paralysed, no First Aid Tent or Ammo Cart), lack the spell and that the spell can affect
    /// ([`computer::spell_affects`]), the sum of `value`'s values, as the original's 32-bit
    /// integers.
    // rule: BAT-AI-014
    pub(crate) fn on_side(&self, c: Candidate, value: Evaluator) -> i32 {
        let (state, ruleset) = (self.state, self.ruleset);
        (0u8..)
            .zip(state.stacks())
            .filter(|(i, s)| {
                s.side == self.side
                    && s.is_alive()
                    && !s.spells.status.contains(Status::NO_TURNS)
                    && !s.has(StackTraits::FIRST_AID)
                    && !s.has(StackTraits::AMMO_SUPPLY)
                    && !has_spell(state, StackId(*i), c.spell)
                    && computer::spell_affects(state, ruleset, self.side, c.spell, StackId(*i))
            })
            .fold(0i32, |sum, (i, _)| {
                sum.wrapping_add(value(self, StackId(i), c))
            })
    }
}

/// Whether the damage `planned` beats `side`'s enemy (0x4369C0's +0x1C; BAT-AI-015): no stack of
/// the other side (its own, whoever controls it) is alive with hit points left after the damage
/// planned on it, but the war machines and the turrets (the original's arrow towers are war
/// machines).
// rule: BAT-AI-015
pub(crate) fn beaten(state: &BattleState, planned: &Planned, side: BattleSide) -> bool {
    let other = side.opponent();
    !(0u8..).zip(state.stacks()).any(|(i, s)| {
        s.side == other
            && s.is_alive()
            && planned.left(state, StackId(i)) > 0
            && !s.has(StackTraits::WAR_MACHINE)
            && !s.is_turret()
    })
}

/// Whether `spell` lasts on stack `id` (the record's +0x198 + 4 × spell: its rounds left).
pub(crate) fn has_spell(state: &BattleState, id: StackId, spell: SpellId) -> bool {
    state
        .effects()
        .iter()
        .any(|e| e.stack == id && e.spell == spell)
}
