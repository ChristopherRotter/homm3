//! The evaluators of the attack bonuses, Bless, morale and luck (§7.45, §7.46; BAT-AI-014,
//! BAT-AI-016): each values the rise of the stack's worth when its strike on its threat record's
//! enemy grows, for the share of the battle the spell lasts.

use h3_battle::computer;
use h3_battle::{StackFlags, StackId, StackTraits};
use h3_content::{CreatureFlag, ORIGINAL_AI_SLIPS};
use h3_content_battle::Ability;
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
        if stack.has(StackTraits::NO_MORALE) {
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

    /// Counterstrike's evaluator (0x439E80; BAT-AI-016): the stack's own retaliations `r` (1, more
    /// for a creature with extra ones; 0 for one that always retaliates, the Royal Griffins); 0
    /// when the predictions beat the enemy. The extra retaliations used, `min(v, n − r)` with `v`
    /// the spell's value and `n` the melee threats ([`tables`](super::tables)), 0 at none.
    /// Against the strongest melee threat `e`: 0 when its strike (capped) kills the stack; `d2`
    /// the survivors' strike back on it (capped), `base = d1 + r × d2` (`d1` the stack's strike,
    /// one `d2` more for a walker that strikes twice), `g = (base + extra × d2) / base`; the gain
    /// on the stack's strike with its shot flag ([`Self::gain`]).
    // rule: BAT-AI-016
    pub(crate) fn counterstrike(&self, s: StackId, c: Candidate) -> i32 {
        let (state, ruleset) = (self.state, self.ruleset);
        let stack = self.stack(s);
        let abilities = ruleset.abilities(stack.creature);
        if abilities.contains(&Ability::UnlimitedRetaliations) || self.beaten {
            return 0;
        }
        let r = 1 + abilities
            .iter()
            .find_map(|a| match *a {
                Ability::ExtraRetaliations { count } => Some(i64::from(count)),
                _ => None,
            })
            .unwrap_or(0);
        let entry = self.tables().melee[s.index()];
        let extra = i64::from(self.effect(c)).min(i64::from(entry.count) - r);
        let (Some(e), Ok(extra)) = (entry.stack, u64::try_from(extra)) else {
            return 0;
        };
        if extra == 0 {
            return 0;
        }
        let count = u64::from(self.stack(e).count);
        let struck = computer::damage_estimate(state, ruleset, (e, s), count, false, 0);
        let left = computer::hit_points(state, s).saturating_sub(struck);
        if left == 0 {
            return 0;
        }
        let own = u64::from(stack.count);
        let d1 = computer::damage_estimate(state, ruleset, (s, e), own, false, 0);
        let survivors = left.div_ceil(u64::from(stack.max_hp.max(1)));
        let d2 = computer::damage_estimate(state, ruleset, (s, e), survivors, false, 0);
        let mut base = d1 + u64::try_from(r).unwrap_or(0) * d2;
        let shooter = ruleset
            .creature(stack.creature)
            .is_some_and(|cr| cr.flags.contains(CreatureFlag::Shooter));
        if stack.has(StackTraits::STRIKES_TWICE) && !shooter {
            base += d2;
        }
        let g = X87::int(d2 * extra + base)
            .over(X87::int(base))
            .unwrap_or(X87::MAX)
            .round(DOUBLE_BITS);
        self.gain((s, e), (self.strike((s, e)), g), c.rounds)
    }

    /// Frenzy's evaluator (0x4375D0; BAT-AI-016): against the threat record's enemy `e` within a
    /// turn. With BAT-QUIRK-25 valued as a second strike, the spell's value unread (survey slip
    /// 15): `d1` the stack's strike by its count from its hit points (its shot flag, capped);
    /// the exchange of the two ([`computer::exchange`]), 0 when the stack dies in it; `d2` its
    /// strike by the survivors (halved for a shooter that shoots twice, which the estimate
    /// doubled); `g = (d1 + d2) / d1`, the gain on its strike with its shot flag
    /// ([`Self::gain`]). Without the slip, as the rule: the attack bonus's value of its defense
    /// times the spell's value in percent ([`Self::attack_bonus`]) less the defense bonus's value
    /// of the defense it loses ([`Self::defense_bonus`]).
    // rule: BAT-AI-016
    pub(crate) fn frenzy(&self, s: StackId, c: Candidate) -> i32 {
        let Some(e) = self.within_a_turn(s) else {
            return 0;
        };
        let (state, ruleset) = (self.state, self.ruleset);
        if !self.slips() {
            let defense = u64::try_from(computer::defense(state, ruleset, s)).unwrap_or(0);
            let gain = defense * u64::from(self.effect(c)) / 100;
            let gain = u16::try_from(gain).unwrap_or(u16::MAX);
            let lost = u16::try_from(defense).unwrap_or(u16::MAX);
            return self
                .attack_bonus((s, e), c.rounds, gain)
                .wrapping_sub(self.defense_bonus(s, c.rounds, lost));
        }
        let stack = self.stack(s);
        let shot = computer::shoots(state, ruleset, s);
        let per = u64::from(stack.max_hp.max(1));
        let hp = (
            computer::hit_points(state, s),
            computer::hit_points(state, e),
        );
        let d1 = computer::damage_estimate(state, ruleset, (s, e), hp.0.div_ceil(per), shot, 0);
        let (left, _) =
            computer::exchange(state, ruleset, self.side, ((s, hp.0), (e, hp.1)), (shot, 0));
        if left == 0 {
            return 0;
        }
        let mut d2 = computer::damage_estimate(state, ruleset, (s, e), left.div_ceil(per), shot, 0);
        if shot && stack.has(StackTraits::STRIKES_TWICE) {
            d2 /= 2;
        }
        let g = X87::int(d1 + d2)
            .over(X87::int(d1))
            .unwrap_or(X87::MAX)
            .round(DOUBLE_BITS);
        self.gain((s, e), (self.strike((s, e)), g), c.rounds)
    }

    /// The turns stack `s` needs to attack its threat record's enemy at `speed` (0x448BD0): 1 for
    /// a stack that can shoot now or at speed 0, else from the record's cost.
    fn turns_at(&self, s: StackId, speed: u32) -> u32 {
        if computer::shoots(self.state, self.ruleset, s) {
            return 1;
        }
        self.threat(s).map_or(1, |t| t.turns_at(speed))
    }

    /// The value of stack `a` attacking `b` first (0x435DC0; BAT-AI-016): `a`'s exchange with `b`
    /// ([`computer::exchange`], `a`'s shot flag), then `b`'s with `a` if `b` lives; the kill value
    /// of `b`'s loss less that of `a`'s ([`computer::damage_value`], desperate in the side's
    /// desperate mode). With BAT-QUIRK-25 `b`'s exchange and kill value take `a`'s shot flag
    /// (survey slip 16), without its own.
    // rule: BAT-AI-016
    pub(crate) fn exchange_value(&self, (a, b): (StackId, StackId)) -> i32 {
        let (state, ruleset, side) = (self.state, self.ruleset, self.side);
        let shot = computer::shoots(state, ruleset, a);
        let other = if self.slips() {
            shot
        } else {
            computer::shoots(state, ruleset, b)
        };
        let hp = (
            computer::hit_points(state, a),
            computer::hit_points(state, b),
        );
        let (mut ha, mut hb) =
            computer::exchange(state, ruleset, side, ((a, hp.0), (b, hp.1)), (shot, 0));
        if hb > 0 {
            (hb, ha) = computer::exchange(state, ruleset, side, ((b, hb), (a, ha)), (other, 0));
        }
        let desperate = self.ctx.desperate();
        let value = |id, lost, shot| {
            computer::damage_value(state, ruleset, &self.ctx, (id, lost), (shot, desperate))
        };
        let mut v = value(b, hp.1 - hb, other);
        if hp.0 > ha {
            v = v.wrapping_sub(value(a, hp.0 - ha, shot));
        }
        v
    }

    /// A speed bonus's value (0x439550; BAT-AI-016) of `bonus` more speed for stack `s` over
    /// `rounds` rounds: 0 without a threat record's enemy `e`, when the predictions beat the
    /// enemy, or with no round left (a round less for a stack that has acted). `s0` its speed
    /// now, `s1` its speed before its effects plus the bonus, `t0` and `t1` its turns to attack
    /// at each; 0 when `t1` exceeds the balance class `k`. When `t1 = 1` and the stack overtakes
    /// `e` (`s0 ≤ e's speed < s1`), the values of striking first both ways
    /// ([`Self::exchange_value`]), at least 0. Plus, when `t1 < t0`, the rounds of fighting won:
    /// `(k − t1 + 1) × w / k − (k − min(t0, k + 1) + 1) × w / k`, `w` its strength, as the
    /// original's 32-bit integers.
    // rule: BAT-AI-016
    pub(crate) fn speed_bonus(&self, s: StackId, bonus: u16, rounds: u32) -> i32 {
        let Some(threat) = self.threat(s) else {
            return 0;
        };
        let stack = self.stack(s);
        let left = if stack.flags.contains(StackFlags::ACTED) {
            i64::from(rounds) - 1
        } else {
            i64::from(rounds)
        };
        if self.beaten || left == 0 {
            return 0;
        }
        let s0 = u32::from(stack.speed_now());
        let s1 = u32::from(stack.speed) + u32::from(bonus);
        let (t0, t1) = (self.turns_at(s, s0), self.turns_at(s, s1));
        let k = u32::from(self.class);
        if t1 > k {
            return 0;
        }
        let mut v = 0i32;
        let e = threat.enemy;
        let se = u32::from(self.stack(e).speed_now());
        if t1 == 1 && s0 <= se && se < s1 {
            v = self
                .exchange_value((s, e))
                .wrapping_add(self.exchange_value((e, s)))
                .max(0);
        }
        if t1 < t0 {
            let t0 = t0.min(k + 1);
            let w = computer::strength(self.state, self.ruleset, &self.ctx, s);
            let k = i32::from(self.class);
            let share = |t: u32| (k - i32::try_from(t).unwrap_or(0) + 1).wrapping_mul(w) / k;
            v = v.wrapping_add(share(t1).wrapping_sub(share(t0)));
        }
        v
    }

    /// Haste's evaluator (0x4396B0; BAT-AI-016): the speed bonus's value of the spell's value
    /// ([`Self::speed_bonus`]).
    // rule: BAT-AI-016
    pub(crate) fn haste(&self, s: StackId, c: Candidate) -> i32 {
        self.speed_bonus(s, self.effect(c), c.rounds)
    }

    /// Prayer's evaluator (0x438AC0; BAT-AI-016): the spell's value as a defense bonus
    /// ([`Self::defense_bonus`]), a speed bonus ([`Self::speed_bonus`]) and, against the threat
    /// record's enemy within a turn, an attack bonus ([`Self::attack_bonus`]), summed.
    // rule: BAT-AI-016
    pub(crate) fn prayer(&self, s: StackId, c: Candidate) -> i32 {
        let v = self.effect(c);
        let attack = self
            .within_a_turn(s)
            .map_or(0, |e| self.attack_bonus((s, e), c.rounds, v));
        self.defense_bonus(s, c.rounds, v)
            .wrapping_add(self.speed_bonus(s, v, c.rounds))
            .wrapping_add(attack)
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
