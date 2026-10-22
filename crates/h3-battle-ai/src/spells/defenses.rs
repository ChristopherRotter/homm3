//! The evaluators of the defenses (§7.46; BAT-AI-016): the protections' common value 0x4387C0,
//! the defense bonus 0x438910, Shield, Air Shield, Stone Skin and Fire Shield. Each values how
//! much less the stack's strongest threat takes off it, against what the threats can do this
//! turn ([`tables`](super::tables)).

use h3_battle::{StackId, computer};
use h3_content::{CreatureFlag, ORIGINAL_AI_SLIPS, SchoolId};
use h3_content_battle::Ability;
use h3_kernel::arith::{DOUBLE_BITS, X87};

use super::{Candidate, View};

/// The base damage a defense bonus's value compares (0x438910's 100).
const BASE: u64 = 100;

/// A percentage's divisor.
const PERCENT: u64 = 100;

/// The highest spell level (the Protections' and Magic Mirror's limit, 5).
const MAX_LEVEL: u8 = 5;

impl View<'_> {
    /// A protection's value (0x4387C0; BAT-AI-016) for stack `s` against the threat `e` over
    /// `rounds` rounds, the factor `f` dividing `e`'s strike on it: `d`, the strike by `e`'s
    /// whole count with its shot flag, uncapped, becomes `d' = trunc(d / f)`; when `d` exceeds
    /// `s`'s hit points `h`, `d = h` and `f = h / d'` (0 when `d'` is 0: the original's infinite
    /// factor comes to 0). 0 unless `d' < d`; 0 when the predictions beat the enemy and the
    /// damage planned on `s` with its top creature's lost hit points is short of a creature's;
    /// 0 when the melee and shot sums ([`tables`](super::tables)) times the balance class `k`,
    /// with the lost hit points, are short of it too. Else `trunc((√f − 1) × w × t)`, `w` its
    /// strength and `t` the rounds' share without the step for a stack that has acted.
    // rule: BAT-AI-016
    pub(crate) fn protection(&self, (s, e): (StackId, StackId), rounds: u32, mut f: X87) -> i32 {
        let (state, ruleset) = (self.state, self.ruleset);
        let shot = computer::shoots(state, ruleset, e);
        let count = u64::from(self.stack(e).count);
        let mut d = computer::uncapped_damage(state, ruleset, (e, s), count, shot, 0);
        let reduced =
            u64::try_from(computer::ftol(X87::int(d).over(f).unwrap_or(X87::ZERO))).unwrap_or(0);
        let h = computer::hit_points(state, s);
        if d > h {
            d = h;
            let Some(g) = X87::int(h).over(X87::int(reduced)) else {
                return 0;
            };
            f = g.round(DOUBLE_BITS);
        }
        if reduced >= d {
            return 0;
        }
        let stack = self.stack(s);
        let creature = u64::from(stack.max_hp);
        let lost = u64::from(stack.max_hp.saturating_sub(stack.top_hp));
        if self.beaten && self.planned.on(s).saturating_add(lost) < creature {
            return 0;
        }
        let tables = self.tables();
        let sums = tables.melee[s.index()]
            .sum
            .saturating_add(tables.shots[s.index()].sum);
        if sums
            .saturating_mul(u64::from(self.class))
            .saturating_add(lost)
            < creature
        {
            return 0;
        }
        let t = self.share_of(rounds);
        computer::ftol(f.sqrt().minus(X87::ONE).times(self.strength(s)).times(t))
    }

    /// A defense bonus's value (0x438910; BAT-AI-016) of `bonus` more defense for stack `s` over
    /// `rounds` rounds: against its worse threat `e` (none: 0), the factor `f = D / D'`, `D` and
    /// `D'` the damage of a base of 100 by `e` (its shot flag) on `s` and on a copy of `s` with
    /// the bonus, doubles; then the protection's value ([`Self::protection`]).
    // rule: BAT-AI-016
    pub(crate) fn defense_bonus(&self, s: StackId, rounds: u32, bonus: u16) -> i32 {
        let Some(e) = self.tables().worse[s.index()].stack else {
            return 0;
        };
        let (state, ruleset) = (self.state, self.ruleset);
        let shot = computer::shoots(state, ruleset, e);
        let bonus = i16::try_from(bonus).unwrap_or(i16::MAX);
        let d = computer::base_damage(state, ruleset, (e, s), (BASE, shot), 0);
        let raised = computer::base_damage_on_defense(state, ruleset, (e, s), (BASE, shot), bonus);
        let f = X87::int(d)
            .over(X87::int(raised))
            .unwrap_or(X87::MAX)
            .round(DOUBLE_BITS);
        self.protection((s, e), rounds, f)
    }

    /// Shield's and Air Shield's value (0x438C60, 0x438BC0; BAT-AI-016): against the strongest
    /// melee threat (Air Shield: the strongest shot; none: 0), with `M` and `T` the melee and
    /// shot sums and `e` the share the spell keeps, the factor `f = (M + T) / K`, `K =
    /// trunc(M × e / 100) + T` (Air Shield: `T` kept instead), then the protection's value. 0
    /// when nothing is kept (the original's infinite factor).
    fn shields(&self, s: StackId, c: Candidate, melee: bool) -> i32 {
        let tables = self.tables();
        let (m, t) = (&tables.melee[s.index()], &tables.shots[s.index()]);
        let Some(e) = (if melee { m.stack } else { t.stack }) else {
            return 0;
        };
        let share = u64::from(self.effect(c));
        let kept = if melee {
            m.sum * share / PERCENT + t.sum
        } else {
            m.sum + t.sum * share / PERCENT
        };
        let Some(f) = X87::int(m.sum + t.sum).over(X87::int(kept)) else {
            return 0;
        };
        self.protection((s, e), c.rounds, f.round(DOUBLE_BITS))
    }

    /// Shield's evaluator (0x438C60; BAT-AI-016): [`Self::shields`] against the melee threats.
    // rule: BAT-AI-016
    pub(crate) fn shield(&self, s: StackId, c: Candidate) -> i32 {
        self.shields(s, c, true)
    }

    /// Air Shield's evaluator (0x438BC0; BAT-AI-016): [`Self::shields`] against the shots.
    // rule: BAT-AI-016
    pub(crate) fn air_shield(&self, s: StackId, c: Candidate) -> i32 {
        self.shields(s, c, false)
    }

    /// Stone Skin's evaluator (0x438D90; BAT-AI-016): the defense bonus's value of the spell's
    /// value ([`Self::defense_bonus`]).
    // rule: BAT-AI-016
    pub(crate) fn stone_skin(&self, s: StackId, c: Candidate) -> i32 {
        self.defense_bonus(s, c.rounds, self.effect(c))
    }

    /// A spell protection's value (0x4396E0; BAT-AI-016) for stack `s` over `rounds` rounds: 0
    /// when the predictions beat the enemy; else the best kill value the protection keeping
    /// `kept` percent takes off the other side's hero's damage spells of `schools` (`None`: every
    /// school) up to `level` ([`computer::spell_protection`]: with BAT-QUIRK-25 survey slip 9)
    /// times the rounds' share.
    // rule: BAT-AI-016
    pub(crate) fn spell_protection(
        &self,
        s: StackId,
        (schools, level, kept): (Option<&[SchoolId]>, u8, u16),
        rounds: u32,
    ) -> i32 {
        if self.beaten {
            return 0;
        }
        let best = computer::spell_protection(
            self.state,
            self.ruleset,
            self.ctx,
            s,
            (schools, level, kept),
            self.ruleset.quirks().is_enabled(ORIGINAL_AI_SLIPS),
        );
        let best = X87::int(u64::try_from(best).unwrap_or(0));
        computer::ftol(best.times(self.share(s, rounds)))
    }

    /// The Protections' evaluator (0x4399A0, 0x4399D0, 0x439A40, 0x439A00; BAT-AI-016): the spell
    /// protection's value against the damage spells of the spell's schools at every level,
    /// keeping the spell's value ([`Self::spell_protection`]).
    // rule: BAT-AI-016
    pub(crate) fn protection_from(&self, s: StackId, c: Candidate) -> i32 {
        let schools = self
            .ruleset
            .spell(c.spell)
            .map_or(&[][..], |sp| sp.schools.as_slice());
        self.spell_protection(s, (Some(schools), MAX_LEVEL, self.effect(c)), c.rounds)
    }

    /// Magic Mirror's evaluator (0x439DE0; BAT-AI-016): the spell protection's value against
    /// every damage spell, keeping `100 − c` percent, `c` the spell's chance; with BAT-QUIRK-25
    /// `2 × (50 − c)` (survey slip 10: twice the chance).
    // rule: BAT-AI-016
    pub(crate) fn magic_mirror(&self, s: StackId, c: Candidate) -> i32 {
        let chance = self.effect(c);
        let kept = if self.ruleset.quirks().is_enabled(ORIGINAL_AI_SLIPS) {
            2 * 50u16.saturating_sub(chance)
        } else {
            100u16.saturating_sub(chance)
        };
        self.spell_protection(s, (None, MAX_LEVEL, kept), c.rounds)
    }

    /// Fire Shield's evaluator (0x43A110; BAT-AI-016): 0 when the predictions beat the enemy. Its
    /// share `p` at the mastery, less the stack's own Fire Shield (the Efreet Sultans' 20); 0
    /// when nothing is left, without a melee threat or against a fire-immune one. Each of the `n`
    /// melee threats is reflected as the strongest one's strike would be, `min(trunc(d_E × p /
    /// 100), h)` (`d_E` its strike capped at the stack's hit points `h`): `g = (n × that + d) /
    /// d`, `d` the stack's strike on it, then the gain on the stack's strike with its shot flag
    /// ([`Self::gain`]).
    // rule: BAT-AI-016
    pub(crate) fn fire_shield(&self, s: StackId, c: Candidate) -> i32 {
        if self.beaten {
            return 0;
        }
        let (state, ruleset) = (self.state, self.ruleset);
        let stack = self.stack(s);
        let own = ruleset
            .abilities(stack.creature)
            .iter()
            .find_map(|a| match *a {
                Ability::InnateSpell { spell, value } if spell == c.spell => Some(value),
                _ => None,
            })
            .unwrap_or(0);
        let p = i32::from(self.effect(c)) - i32::from(own);
        let entry = self.tables().melee[s.index()];
        let (Some(e), Ok(p)) = (entry.stack, u64::try_from(p)) else {
            return 0;
        };
        let immune = ruleset
            .creature(self.stack(e).creature)
            .is_some_and(|cr| cr.flags.contains(CreatureFlag::FireImmune));
        if p == 0 || immune {
            return 0;
        }
        let count = u64::from(self.stack(e).count);
        let struck = computer::damage_estimate(state, ruleset, (e, s), count, false, 0);
        let reflected = (struck * p / PERCENT).min(computer::hit_points(state, s));
        let own_count = u64::from(stack.count);
        let d = computer::damage_estimate(state, ruleset, (s, e), own_count, false, 0);
        let g = X87::int(u64::from(entry.count) * reflected + d)
            .over(X87::int(d))
            .unwrap_or(X87::MAX)
            .round(DOUBLE_BITS);
        self.gain((s, e), (self.strike((s, e)), g), c.rounds)
    }
}
