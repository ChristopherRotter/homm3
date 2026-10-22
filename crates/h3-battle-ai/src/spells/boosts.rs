//! The evaluators of the attack bonuses, Bless, morale and luck (§7.45, §7.46; BAT-AI-014,
//! BAT-AI-016): each values the rise of the stack's worth when its strike on its threat record's
//! enemy grows, for the share of the battle the spell lasts.

use h3_battle::StackId;
use h3_battle::computer;
use h3_content::{CreatureFlag, ORIGINAL_AI_SLIPS};
use h3_kernel::arith::{DOUBLE_BITS, X87};

use super::{Candidate, SpellContext};

/// The base damage the attack bonus's value compares (0x437800's 100).
const BASE: u64 = 100;

/// The values of a morale point above and below 0 (0x63B780, doubles: 0.0173 and −0.0833; the
/// second's magnitude).
const MORALE: [(u32, u32, u32); 2] = [(173, 10_000, DOUBLE_BITS), (833, 10_000, DOUBLE_BITS)];

/// The highest and lowest morale or luck the values count (the callers' clamp to ±3).
const LIMIT: i32 = 3;

/// The dice of a luck roll (0x438490's divisor: one chance in 24 per point).
const LUCK_DICE: i32 = 24;

/// A signed value of the original's x87 arithmetic: its sign and its magnitude ([`X87`] holds
/// none below 0).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Signed {
    pub(crate) negative: bool,
    pub(crate) magnitude: X87,
}

impl Signed {
    /// `n × f` for an integer `n` and a positive factor `f`, rounded as the FPU does.
    fn product(n: i32, f: X87) -> Self {
        Self {
            negative: n < 0,
            magnitude: X87::int(u64::from(n.unsigned_abs())).times(f),
        }
    }

    /// `self + rhs`, rounded as the FPU does.
    fn plus(self, rhs: Self) -> Self {
        if self.negative == rhs.negative {
            return Self {
                negative: self.negative,
                magnitude: self.magnitude.plus(rhs.magnitude),
            };
        }
        let (big, small) = if self.magnitude >= rhs.magnitude {
            (self, rhs)
        } else {
            (rhs, self)
        };
        Self {
            negative: big.negative,
            magnitude: big.magnitude.minus(small.magnitude),
        }
    }

    /// `−self`.
    fn negated(self) -> Self {
        Self {
            negative: !self.negative,
            magnitude: self.magnitude,
        }
    }

    /// Whether it is 0 (either sign).
    pub(crate) fn is_zero(self) -> bool {
        self.magnitude == X87::ZERO
    }

    /// `trunc(self × f)` for a positive factor `f`, as `_ftol` gives it (0x6179D4).
    pub(crate) fn ftol_times(self, f: X87) -> i32 {
        let v = computer::ftol(self.magnitude.times(f));
        if self.negative { v.wrapping_neg() } else { v }
    }
}

/// The value of a change of morale or luck by `c` from `m` (0x435850; BAT-AI-016), with the
/// values `(p, n)` of a point above 0 and the magnitude of one below (each point below 0 costs
/// `n`): the change of `V`, `V(m) = p × m` above 0 and `−n × m` below, the result clamped to ±3;
/// 0 for a rise at 3 or a fall at −3. With BAT-QUIRK-25 a rise from below 0 is always valued as
/// one that crosses 0 (survey slip 12: the test compares the change with `m`, not with `−m`):
/// from −3 by 1 it gives `−2p + 3n`, not `n`.
// rule: BAT-AI-016
pub(crate) fn change_value(m: i32, c: i32, (p, n): (X87, X87), slip: bool) -> Signed {
    let zero = Signed {
        negative: false,
        magnitude: X87::ZERO,
    };
    if c > 0 {
        if m >= LIMIT {
            return zero;
        }
        let c = c.min(LIMIT - m);
        if m >= 0 {
            return Signed::product(c, p);
        }
        let stays_below = if slip { c <= m } else { c <= -m };
        if stays_below {
            return Signed::product(c, n);
        }
        return Signed::product(m + c, p).plus(Signed::product(-m, n));
    }
    if m <= -LIMIT {
        return zero;
    }
    let c = c.max(-LIMIT - m);
    if m <= 0 {
        return Signed::product(c, n);
    }
    if -c <= m {
        return Signed::product(c, p);
    }
    Signed::product(m, p)
        .negated()
        .plus(Signed::product(m + c, n))
}

impl SpellContext<'_> {
    /// The value of `bonus` more attack for stack `s` against `e` over `rounds` rounds (0x437800;
    /// BAT-AI-014): 0 when the predictions beat the enemy (BAT-AI-015); else the gain `g = D' /
    /// D`, `D` and `D'` the damage of a base of 100 on `e` without and with the bonus
    /// ([`computer::base_damage`], `s`'s shot flag), doubles, on `s`'s strike by its count on `e`
    /// ([`Self::gain`]).
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
        let g = base(bonus)
            .over(base(0))
            .unwrap_or(X87::MAX)
            .round(DOUBLE_BITS);
        self.gain((s, e), (self.strike((s, e)), g), rounds)
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
        self.within_a_turn(s)
            .map_or(0, |e| self.attack_bonus((s, e), c.rounds, self.effect(c)))
    }

    /// Precision's evaluator (0x438B50; BAT-AI-016): Bloodlust's for a stack `s` that can shoot
    /// now, 0 for any other.
    // rule: BAT-AI-016
    pub(crate) fn precision(&self, s: StackId, c: Candidate) -> i32 {
        if !computer::shoots(self.state, self.ruleset, s) {
            return 0;
        }
        self.within_a_turn(s)
            .map_or(0, |e| self.attack_bonus((s, e), c.rounds, self.effect(c)))
    }

    /// Slayer's evaluator (0x438D00; BAT-AI-016): the attack bonus's value of the spell's value
    /// against the threat record's enemy within a turn when it is a king the mastery reaches (king
    /// 1 always, king 2 from Advanced, king 3 at Expert, as the rule, BAT-MAG-014), else 0.
    // rule: BAT-AI-016
    pub(crate) fn slayer(&self, s: StackId, c: Candidate) -> i32 {
        let Some(e) = self.within_a_turn(s) else {
            return 0;
        };
        let flags = self
            .ruleset
            .creature(self.stack(e).creature)
            .map(|cr| cr.flags)
            .unwrap_or_default();
        let king = flags.contains(CreatureFlag::King1)
            || (flags.contains(CreatureFlag::King2) && c.mastery >= 2)
            || (flags.contains(CreatureFlag::King3) && c.mastery >= 3);
        if !king {
            return 0;
        }
        self.attack_bonus((s, e), c.rounds, self.effect(c))
    }

    /// Bless's evaluator (0x437430; BAT-AI-016): against the threat record's enemy within a turn,
    /// the gain `g = (max + v) / p`, `max` the most damage a creature of `s` deals, `v` the
    /// spell's value and `p` its damage per creature now (0x4426F0), on its strike by its count
    /// ([`Self::gain`]). No "beaten" test.
    // rule: BAT-AI-016
    pub(crate) fn bless(&self, s: StackId, c: Candidate) -> i32 {
        let Some(e) = self.within_a_turn(s) else {
            return 0;
        };
        let (state, ruleset) = (self.state, self.ruleset);
        let max = computer::maximum_damage(state, ruleset, s) + u64::from(self.effect(c));
        let p = computer::damage_per_creature(state, ruleset, s);
        let g = X87::int(max).over(p).unwrap_or(X87::MAX).round(DOUBLE_BITS);
        self.gain((s, e), (self.strike((s, e)), g), c.rounds)
    }

    /// Mirth's evaluator (0x438170; BAT-AI-016): 0 for a creature without morale; `x` the value
    /// of the morale's change by the spell's value from the stack's morale clamped to ±3
    /// ([`change_value`] with morale's values), 0 when `x` is 0; else `trunc(w × t × x)`, `w`
    /// the stack's strength and `t` the rounds' share. No "beaten" test.
    // rule: BAT-AI-016
    pub(crate) fn mirth(&self, s: StackId, c: Candidate) -> i32 {
        let stack = self.stack(s);
        if stack.has(h3_battle::StackTraits::NO_MORALE) {
            return 0;
        }
        let m = i32::from(stack.morale).clamp(-LIMIT, LIMIT);
        let x = change_value(m, i32::from(self.effect(c)), values(MORALE), self.slips());
        if x.is_zero() {
            return 0;
        }
        x.ftol_times(self.strength(s).times(self.share(s, c.rounds)))
    }

    /// Fortune's evaluator (0x438490; BAT-AI-016): against the threat record's enemy within a
    /// turn, from the stack's luck `L` as its record keeps it (not clamped, [`computer::luck`])
    /// and the spell's value `v`, cut to `3 − L`: nothing at `L ≥ 3` or `L + v ≤ −3`. With `d` the
    /// stack's strike on the enemy by its count, uncapped: each point of bad luck removed (`−L`
    /// when the luck crosses 0, else `v`) is worth `u / 24`, `u` the gain of a halved strike
    /// doubled (`d / 2` raised by 2, [`Self::gain`]); each point of good luck (`L + v` from below
    /// 0, else `v`) `u' / 24`, `u'` the gain of `d` doubled: one chance in 24 per point, as the
    /// original's 32-bit integers. No "beaten" test.
    // rule: BAT-AI-016
    pub(crate) fn fortune(&self, s: StackId, c: Candidate) -> i32 {
        let Some(e) = self.within_a_turn(s) else {
            return 0;
        };
        let (state, ruleset) = (self.state, self.ruleset);
        let l = computer::luck(state, ruleset, s);
        let mut v = i32::from(self.effect(c));
        if l >= LIMIT || l.saturating_add(v) <= -LIMIT {
            return 0;
        }
        if l + v > LIMIT {
            v = LIMIT - l;
        }
        let shot = computer::shoots(state, ruleset, s);
        let count = u64::from(self.stack(s).count);
        let d = computer::uncapped_damage(state, ruleset, (s, e), count, shot, 0);
        let d = i32::try_from(d).unwrap_or(i32::MAX);
        let two = X87::int(2);
        let mut sum = 0i32;
        if l < 0 {
            let points = if l + v > 0 { -l } else { v };
            let half = u64::try_from(d / 2).unwrap_or(0);
            let u = self.gain((s, e), (half, two), c.rounds);
            sum = u.wrapping_mul(points) / LUCK_DICE;
        }
        if l + v > 0 {
            let points = if l < 0 { l + v } else { v };
            let u = self.gain((s, e), (u64::try_from(d).unwrap_or(0), two), c.rounds);
            sum = sum.wrapping_add(u.wrapping_mul(points) / LUCK_DICE);
        }
        sum
    }

    /// Whether the ruleset ports the original's slips (BAT-QUIRK-25).
    fn slips(&self) -> bool {
        self.ruleset.quirks().is_enabled(ORIGINAL_AI_SLIPS)
    }
}

/// A point's values as [`change_value`] takes them.
fn values(table: [(u32, u32, u32); 2]) -> (X87, X87) {
    let [(pn, pd, pb), (nn, nd, nb)] = table;
    (
        X87::ratio(pn, pd, pb).unwrap_or(X87::ZERO),
        X87::ratio(nn, nd, nb).unwrap_or(X87::ZERO),
    )
}
