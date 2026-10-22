//! The evaluators of the harmful spells (§7.46, §7.53; BAT-AI-016): what a spell on a stack of
//! the other side is worth to the half's side. On the neutral side only the removals reach them
//! ([`removals`](super::removals): the value of a harmful spell lasting on the side's own stack, in
//! the other side's half, without the resistance); a cast's candidate (the hero's, E23 T5) also
//! weighs the stack's resistance.

use h3_battle::computer;
use h3_battle::{StackFlags, StackId, StackTraits, Status};
use h3_content::SpellCategory;
use h3_content::SpellId;
use h3_content_battle::{SpellEffect, TimedChange};
use h3_kernel::arith::{DOUBLE_BITS, SINGLE_BITS, X87};

use super::boosts::{LIMIT, Signed, change_value, values};
use super::{Candidate, View};

/// The values of a luck point above and below 0 (0x63B790, floats: 0.0173 and −0.0122, the
/// second's magnitude; Misfortune's).
const LUCK: [(u32, u32, u32); 2] = [(173, 10_000, SINGLE_BITS), (122, 10_000, SINGLE_BITS)];

/// The values of a morale point (0x63B780, doubles: 0.0173 and −0.0833; Sorrow's).
const MORALE: [(u32, u32, u32); 2] = [(173, 10_000, DOUBLE_BITS), (833, 10_000, DOUBLE_BITS)];

/// The Hypnotize value's turns by mastery (0x660858: 1, 1, 2, 3).
const HYPNOTIZE_TURNS: [u32; 4] = [1, 1, 2, 3];

impl View<'_> {
    /// Whether stack `s` can strike one of the half's side's stacks this turn (the tables' +0x14).
    fn threatens(&self, s: StackId) -> bool {
        self.tables().threatens & (1 << s.index()) != 0
    }

    /// Whether the half's side is desperate (+0x28, its battle context's +8).
    fn desperate(&self) -> bool {
        self.ctx.desperate()
    }

    /// The first spell of the ruleset with the timed change `change` alone among its timed
    /// changes and of `category` (the original's spell by number: Blind 62, Weakness 45), if any.
    fn spell_of(&self, change: TimedChange, category: SpellCategory) -> Option<SpellId> {
        self.ruleset
            .spells()
            .iter()
            .position(|sp| {
                sp.category == category
                    && sp
                        .magic
                        .effects
                        .iter()
                        .filter_map(|e| e.timed())
                        .eq([change])
            })
            .and_then(SpellId::from_index)
    }

    /// `value` resisted as a cast's candidate weighs it (0x5A8090 of `spell` from the half's side
    /// at `s`): with the candidate's flag `trunc(f32(value) × f)`, else `value` as it is.
    fn resisted(&self, c: Candidate, spell: SpellId, s: StackId, value: i32) -> i32 {
        if !c.cast {
            return value;
        }
        let f = computer::effect_factor(self.state, self.ruleset, self.side, spell, s);
        let magnitude = X87::int(u64::from(value.unsigned_abs())).round(SINGLE_BITS);
        Signed {
            negative: value < 0,
            magnitude,
        }
        .ftol_times(f)
    }

    /// `w − √g × w` for a strength `w` and a factor `g` as the original's x87 computes it, then
    /// truncated: below 0 when `g` exceeds 1.
    fn loss(w: X87, g: X87) -> i32 {
        let kept = g.sqrt().times(w);
        if kept > w {
            computer::ftol(kept.minus(w)).wrapping_neg()
        } else {
            computer::ftol(w.minus(kept))
        }
    }

    /// Curse's evaluator (0x43B370; BAT-AI-016): `s` threatens the side, not desperate, not
    /// beaten. `g = max(1, min − v) / p`, `min` the least damage of a creature of `s`, `v` the
    /// spell's value and `p` its damage per creature now (0x4426F0); `trunc(trunc(w − √g × w) ×
    /// t)`, `w` its strength and `t` the rounds' share; with the candidate's flag resisted.
    // rule: BAT-AI-016
    pub(crate) fn curse(&self, s: StackId, c: Candidate) -> i32 {
        if !self.threatens(s) || self.desperate() || self.beaten {
            return 0;
        }
        let (state, ruleset) = (self.state, self.ruleset);
        let w = self.strength(s);
        let p = computer::damage_per_creature(state, ruleset, s);
        let min = i64::try_from(computer::minimum_damage(state, ruleset, s)).unwrap_or(0)
            - i64::from(self.effect(c));
        let num = X87::int(u64::try_from(min.max(1)).unwrap_or(1));
        let g = num.over(p).unwrap_or(X87::MAX).round(DOUBLE_BITS);
        let a = Self::loss(w, g);
        let t = self.share(s, c.rounds);
        let b = Signed {
            negative: a < 0,
            magnitude: X87::int(u64::from(a.unsigned_abs())),
        }
        .ftol_times(t);
        self.resisted(c, c.spell, s, b)
    }

    /// Weakness's evaluator (0x438ED0; BAT-AI-016): `s` threatens the side, not desperate, its
    /// threat record's enemy within a turn: the attack bonus's value of `s` against it of the
    /// smaller of its attack and the spell's value ([`Self::attack_bonus`]: what it would lose
    /// taken as what it would gain). No resistance.
    // rule: BAT-AI-016
    pub(crate) fn weakness(&self, s: StackId, c: Candidate) -> i32 {
        if !self.threatens(s) || self.desperate() {
            return 0;
        }
        let Some(e) = self.within_a_turn(s) else {
            return 0;
        };
        let lost = self.stack(s).attack.min(self.effect(c));
        self.attack_bonus((s, e), c.rounds, lost)
    }

    /// Disrupting Ray's evaluator (0x438DC0; BAT-AI-016): not desperate; some stack of the side
    /// has `s` as its threat record's enemy; `trunc(w − √(1 − 0.05 v) × w)`, `w` the strength of
    /// `s` and `v` the spell's value; no rounds' share, no "beaten" test; with the candidate's
    /// flag resisted.
    // rule: BAT-AI-016
    pub(crate) fn disrupting_ray(&self, s: StackId, c: Candidate) -> i32 {
        let targeted = (0u8..)
            .zip(self.state.stacks())
            .filter(|(_, r)| r.side == self.side)
            .any(|(i, _)| self.threat(StackId(i)).is_some_and(|t| t.enemy == s));
        if self.desperate() || !targeted {
            return 0;
        }
        let step = X87::ratio(1, 20, DOUBLE_BITS).unwrap_or(X87::ZERO);
        let g = X87::ONE
            .minus(X87::int(u64::from(self.effect(c))).times(step))
            .round(DOUBLE_BITS);
        let v = Self::loss(self.strength(s), g);
        self.resisted(c, c.spell, s, v)
    }

    /// Sorrow's and Misfortune's value (0x4382C0, 0x438F60; BAT-AI-016): `x` minus the value of
    /// the fall by the spell's value from `m` ([`change_value`] with `points`), 0 when it is 0;
    /// with the candidate's flag times the resistance as a double; `trunc(w × t × x)`.
    fn fall(&self, s: StackId, c: Candidate, (m, points): (i32, (X87, X87))) -> i32 {
        let v = -i32::from(self.effect(c));
        let mut x = change_value(m.clamp(-LIMIT, LIMIT), v, points, self.slips()).negated();
        if x.is_zero() {
            return 0;
        }
        if c.cast {
            let f = computer::effect_factor(self.state, self.ruleset, self.side, c.spell, s);
            x.magnitude = x.magnitude.times(f).round(DOUBLE_BITS);
        }
        x.ftol_times(self.strength(s).times(self.share(s, c.rounds)))
    }

    /// Sorrow's evaluator (0x4382C0; BAT-AI-016): 0 for a creature without morale, when `s` does
    /// not threaten the side, in the desperate mode or when beaten; else the fall of its morale
    /// ([`Self::fall`], morale's values).
    // rule: BAT-AI-016
    pub(crate) fn sorrow(&self, s: StackId, c: Candidate) -> i32 {
        let stack = self.stack(s);
        if stack.has(StackTraits::NO_MORALE)
            || !self.threatens(s)
            || self.desperate()
            || self.beaten
        {
            return 0;
        }
        self.fall(s, c, (i32::from(stack.morale), values(MORALE)))
    }

    /// Misfortune's evaluator (0x438F60; BAT-AI-016): `s` threatens the side, not desperate (no
    /// "beaten" test); the fall of its luck as its record keeps it ([`computer::luck`]; luck's
    /// values, floats).
    // rule: BAT-AI-016
    pub(crate) fn misfortune(&self, s: StackId, c: Candidate) -> i32 {
        if !self.threatens(s) || self.desperate() {
            return 0;
        }
        let luck = computer::luck(self.state, self.ruleset, s);
        self.fall(s, c, (luck, values(LUCK)))
    }

    /// Slow's evaluator (0x439270; BAT-AI-016): 0 without a threat record's enemy, when beaten,
    /// when `s`'s turns to attack `t0` exceed the class `k` or with no round left (a round less
    /// when it acted). `s1 = max(1, trunc(v × s0 / 100))`, `s0` its speed now. When `t0 = 1`: the
    /// best of striking first both ways ([`Self::exchange_value`]) over the side's stacks that
    /// can act whose threat record's enemy is `s` and that Slow lets act first (`s1 < speed ≤
    /// s0`), at least 0. Plus, when `s` cannot shoot now, the rounds of fighting it loses, `n =
    /// min(its turns at s1 − t0, the rounds)`: `(k − t0 + 1) × w / k − (k − min(t0 + n, k + 1) +
    /// 1) × w / k`. With the candidate's flag resisted.
    // rule: BAT-AI-016
    pub(crate) fn slow(&self, s: StackId, c: Candidate) -> i32 {
        if self.threat(s).is_none() || self.beaten {
            return 0;
        }
        let stack = self.stack(s);
        let s0 = u32::from(stack.speed_now());
        let t0 = self.turns_at(s, s0);
        let k = u32::from(self.class);
        let left = if stack.flags.contains(StackFlags::ACTED) {
            i64::from(c.rounds) - 1
        } else {
            i64::from(c.rounds)
        };
        if t0 > k || left == 0 {
            return 0;
        }
        let s1 = (u32::from(self.effect(c)) * s0 / 100).max(1);
        let mut best = 0i32;
        if t0 == 1 {
            for (i, r) in (0u8..).zip(self.state.stacks()) {
                let id = StackId(i);
                let speed = u32::from(r.speed_now());
                let able = r.side == self.side
                    && r.is_alive()
                    && !r.spells.status.contains(Status::NO_TURNS)
                    && !r.has(StackTraits::FIRST_AID)
                    && !r.has(StackTraits::AMMO_SUPPLY);
                if able
                    && self.threat(id).is_some_and(|t| t.enemy == s)
                    && s1 < speed
                    && speed <= s0
                {
                    let x = self
                        .exchange_value((id, s))
                        .wrapping_add(self.exchange_value((s, id)));
                    best = best.max(x);
                }
            }
        }
        if !computer::shoots(self.state, self.ruleset, s) {
            let lost = i64::from(self.turns_at(s, s1)) - i64::from(t0);
            let n = lost.min(left);
            if n > 0 {
                let t1 = (i64::from(t0) + n).min(i64::from(k) + 1);
                let w = self.strength_of(s);
                let k = i32::from(self.class);
                let share = |t: i64| (k - i32::try_from(t).unwrap_or(0) + 1).wrapping_mul(w) / k;
                best = best.wrapping_add(share(i64::from(t0)).wrapping_sub(share(t1)));
            }
        }
        self.resisted(c, c.spell, s, best)
    }

    /// Blind's and Paralyze's evaluator (0x439100; BAT-AI-016): `s` threatens the side, not
    /// beaten. When its strength `w` is at least its side's able strength (it is all that
    /// fights), `trunc((0.5 − √(e / 400)) × w)`, `e` the retaliation share at the mastery; else
    /// `trunc(w × t)`. With the candidate's flag resisted. With BAT-QUIRK-25 Paralyze takes
    /// Blind's share and resistance (survey slip 13), without its own.
    // rule: BAT-AI-016
    pub(crate) fn disable(&self, s: StackId, c: Candidate) -> i32 {
        if !self.threatens(s) || self.beaten {
            return 0;
        }
        let blind = self
            .slips()
            .then(|| self.spell_of(TimedChange::Blind, SpellCategory::Combat))
            .flatten();
        let record = blind.map_or(c, |spell| Candidate { spell, ..c });
        let w = self.strength_of(s);
        let wx = X87::int(u64::try_from(w).unwrap_or(0));
        let v = if w >= self.half.other_able {
            let e = X87::int(u64::from(self.effect(record)));
            let root = e
                .over(X87::int(400))
                .unwrap_or(X87::ZERO)
                .round(DOUBLE_BITS)
                .sqrt();
            let half = X87::ratio(1, 2, DOUBLE_BITS).unwrap_or(X87::ZERO);
            computer::ftol(half.minus(root).times(wx))
        } else {
            computer::ftol(wx.times(self.share(s, c.rounds)))
        };
        self.resisted(c, record.spell, s, v)
    }

    /// Forgetfulness's evaluator (0x43B500; BAT-AI-016): `s` can shoot now, no clone, not
    /// desperate, not beaten: its worth with the shot flag less without (0x442A50), times its
    /// hit points over a creature's, then times the rounds' share; with the candidate's flag
    /// resisted.
    // rule: BAT-AI-016
    pub(crate) fn forgetfulness(&self, s: StackId, c: Candidate) -> i32 {
        let (state, ruleset) = (self.state, self.ruleset);
        let stack = self.stack(s);
        if !computer::shoots(state, ruleset, s)
            || stack.has(StackTraits::CLONE)
            || self.desperate()
            || self.beaten
        {
            return 0;
        }
        let (with, without) = (
            computer::worth(state, ruleset, self.ctx, s, true),
            computer::worth(state, ruleset, self.ctx, s, false),
        );
        let dw = if with >= without {
            Signed {
                negative: false,
                magnitude: with.minus(without),
            }
        } else {
            Signed {
                negative: true,
                magnitude: without.minus(with),
            }
        };
        let hp = X87::int(computer::hit_points(state, s));
        let per = X87::int(u64::from(stack.max_hp.max(1)));
        let a = Signed {
            negative: dw.negative,
            magnitude: hp.times(dw.magnitude).over(per).unwrap_or(X87::ZERO),
        }
        .ftol_times(X87::ONE);
        let b = Signed {
            negative: a < 0,
            magnitude: X87::int(u64::from(a.unsigned_abs())),
        }
        .ftol_times(self.share(s, c.rounds));
        self.resisted(c, c.spell, s, b)
    }

    /// A berserk attack's value (0x43A340; BAT-AI-016): 0 for a target `t` of the half's side;
    /// else the exchange of `s` on it ([`computer::exchange`], `s`'s shot flag), both stacks'
    /// losses counted as gains ([`computer::damage_value`], the side's desperate mode).
    // rule: BAT-AI-016
    pub(crate) fn berserk_attack(&self, (s, t): (StackId, StackId)) -> i32 {
        if self.stack(t).side == self.side {
            return 0;
        }
        let (state, ruleset) = (self.state, self.ruleset);
        let shot = computer::shoots(state, ruleset, s);
        let hp = (
            computer::hit_points(state, s),
            computer::hit_points(state, t),
        );
        let (hs, ht) =
            computer::exchange(state, ruleset, self.side, ((s, hp.0), (t, hp.1)), (shot, 0));
        let desperate = self.desperate();
        let value = |id, lost| {
            computer::damage_value(state, ruleset, self.ctx, (id, lost), (shot, desperate))
        };
        value(s, hp.0 - hs).wrapping_add(value(t, hp.1 - ht))
    }

    /// Berserk's evaluator (0x43A400; BAT-AI-016): 0 when beaten or without a target; the mean of
    /// the berserk attack's values ([`Self::berserk_attack`]) over the stacks a berserk `s`
    /// would take ([`computer::berserk_search`]), the sum divided as an unsigned 32-bit integer.
    /// No rounds' share. With BAT-QUIRK-25 the context keeps the candidates' search (survey slip
    /// 22: the walker's fallbacks read it).
    // rule: BAT-AI-016
    pub(crate) fn berserk(&self, s: StackId, _c: Candidate) -> i32 {
        if self.beaten {
            return 0;
        }
        let (mut sum, mut n) = (0i32, 0u32);
        let (candidates, map) = computer::berserk_search(self.state, self.ruleset, s);
        for t in candidates {
            sum = sum.wrapping_add(self.berserk_attack((s, t)));
            n += 1;
        }
        if let Some(map) = map {
            self.root.searched(map);
        }
        if n == 0 {
            return 0;
        }
        i32::from_ne_bytes((u32::from_ne_bytes(sum.to_ne_bytes()) / n).to_ne_bytes())
    }

    /// Hypnotize's evaluator (0x43A500; BAT-AI-016): 0 when beaten or when `s` cannot act (dead,
    /// blind, turned to stone or paralysed, the First Aid Tent or the Ammo Cart). `n = min(k,
    /// (1, 1, 2, 3) by the mastery)`, less 1 when it acted, 0 at none; the pathfinder's map from
    /// `s` for the side up to `n` times its speed before Slow ([`computer::map_within`]); the
    /// best berserk attack's value ([`Self::berserk_attack`]) over the other stacks of its side
    /// it reaches, at least 0. With BAT-QUIRK-25 the context keeps that map (survey slip 22: the
    /// walker's fallbacks read it).
    // rule: BAT-AI-016
    pub(crate) fn hypnotize(&self, s: StackId, c: Candidate) -> i32 {
        let stack = self.stack(s);
        let able = stack.is_alive()
            && !stack.spells.status.contains(Status::NO_TURNS)
            && !stack.has(StackTraits::FIRST_AID)
            && !stack.has(StackTraits::AMMO_SUPPLY);
        if self.beaten || !able {
            return 0;
        }
        let turns = HYPNOTIZE_TURNS[usize::from(c.mastery.min(3))].min(u32::from(self.class));
        let turns = if stack.flags.contains(StackFlags::ACTED) {
            turns.saturating_sub(1)
        } else {
            turns
        };
        if turns == 0 {
            return 0;
        }
        let speed = u32::from(computer::unslowed_speed(self.state, s));
        let range = u16::try_from(speed * turns).unwrap_or(u16::MAX);
        let map = computer::map_within(self.state, self.ruleset, (s, self.side), range);
        let best = (0u8..)
            .zip(self.state.stacks())
            .filter(|&(i, r)| {
                StackId(i) != s && r.side == stack.side && r.is_alive() && map.mark(r.hex).is_some()
            })
            .map(|(i, _)| self.berserk_attack((s, StackId(i))))
            .fold(0, i32::max);
        self.root.searched(map);
        best
    }

    /// Poison's evaluator (0x439500; BAT-AI-016): 0 when beaten; else the kill value of half the
    /// hit points of `s`'s top creature ([`computer::damage_value`], its shot flag, the side's
    /// desperate mode).
    // rule: BAT-AI-016
    pub(crate) fn poison(&self, s: StackId, _c: Candidate) -> i32 {
        if self.beaten {
            return 0;
        }
        let (state, ruleset) = (self.state, self.ruleset);
        let shot = computer::shoots(state, ruleset, s);
        let half = u64::from(self.stack(s).top_hp / 2);
        computer::damage_value(
            state,
            ruleset,
            self.ctx,
            (s, half),
            (shot, self.desperate()),
        )
    }

    /// Disease's evaluator (0x438A10; BAT-AI-016): `s` threatens the side, not desperate:
    /// `trunc((w − 0.9 w) × rounds)` (no class), `w` its strength; with the candidate's flag
    /// resisted, with BAT-QUIRK-25 as Weakness is (survey slip 14), without as Disease.
    // rule: BAT-AI-016
    pub(crate) fn disease(&self, s: StackId, c: Candidate) -> i32 {
        if !self.threatens(s) || self.desperate() {
            return 0;
        }
        let w = self.strength(s);
        let tenth = X87::ratio(9, 10, DOUBLE_BITS).unwrap_or(X87::ZERO);
        let v = computer::ftol(w.minus(w.times(tenth)).times(X87::int(u64::from(c.rounds))));
        let weakness = self
            .slips()
            .then(|| self.spell_of(TimedChange::AttackLoss, SpellCategory::Combat))
            .flatten();
        self.resisted(c, weakness.unwrap_or(c.spell), s, v)
    }

    /// Age's evaluator (0x4373F0; BAT-AI-016): 0 when beaten; else a third of `s`'s strength.
    // rule: BAT-AI-016
    pub(crate) fn age(&self, s: StackId, _c: Candidate) -> i32 {
        if self.beaten {
            return 0;
        }
        self.strength_of(s) / 3
    }
}

/// Whether `effects` are those of a spell that ends Poison (the removals' exception, 71): its
/// timed change.
pub(crate) fn poisons(effects: &[SpellEffect]) -> bool {
    effects.contains(&SpellEffect::Timed(TimedChange::Poison))
}
