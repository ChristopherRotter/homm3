//! The computer's spell context and its spell values (0x4369C0, 0x43A910, the evaluators of
//! 0x43B690; `docs/formats/installation.md` §7.42, §7.44 to §7.46, §7.51 to §7.53; BAT-AI-014 to
//! BAT-AI-016): what a cast on a stack is worth to the side. The creature casts that weigh a spell
//! (the Ogre Mage's and the Master Genie's scan, [`casts`](crate::casts)) build the context and
//! ask its evaluator for each stack.
//!
//! The context holds the side's battle context, its balance class, "the enemy beaten" after the
//! predictions ([`predict`]: the damage the side's stacks that act before the other side's next
//! one are expected to deal, with their draws) and each stack's threat record (0x422B20 with flag
//! 0, [`computer::threat`]). Each spell's evaluator is told by the spell's data ([`Kind`]), as the
//! original's table 0x43B690 tells it by the spell's number: the attack bonuses, luck and morale
//! ([`boosts`]). Not built yet (BAT-STUB-4, E23 T4c): the threat tables and the second context
//! (0x43C040) and the other evaluators.

pub(crate) mod boosts;
mod defenses;
pub(crate) mod tables;

use core::cell::OnceCell;

use h3_battle::computer::{self, Context, Planned, Threat};
use h3_battle::{
    BattleSide, BattleState, STACK_CAPACITY, StackFlags, StackId, StackTraits, Status,
};
use h3_content::SpellId;
use h3_content_battle::{BattleRuleset, BattleSpell, SpellEffect, TimedChange};
use h3_kernel::Rng;
use h3_kernel::arith::X87;

use crate::casts::balance;
use crate::predict;
use tables::Tables;

/// A cast the computer weighs (the cast candidate, §7.44): the spell, its mastery, its power, the
/// rounds its timed effects last and whether it is a cast's candidate (+0x10: 1) or a spell a
/// removal values (0; [`Self::cast`]).
#[derive(Debug, Clone, Copy)]
pub(crate) struct Candidate {
    pub(crate) spell: SpellId,
    pub(crate) mastery: u8,
    pub(crate) power: u32,
    pub(crate) rounds: u32,
    pub(crate) cast: bool,
}

impl Candidate {
    /// A cast's candidate of `spell` at `mastery` with the creature's `power` as its power and
    /// rounds (6 and 6 for the Ogre Mage and the Master Genie, §7.44).
    pub(crate) const fn cast(spell: SpellId, mastery: u8, power: u32) -> Self {
        Self {
            spell,
            mastery,
            power,
            rounds: power,
            cast: true,
        }
    }
}

/// An evaluator (0x43B690's table): the value of a candidate on a stack.
pub(crate) type Evaluator = fn(&SpellContext<'_>, StackId, Candidate) -> i32;

/// Which evaluator of 0x43B690's table values a spell (§7.44, §7.46), told by the spell's data as
/// the original tells it by the spell's number: its timed changes, its other effects and, for a
/// defense loss alone, whether a hero may cast it (Disrupting Ray, 47; the Rust Dragons' Acid
/// Breath, 80, is a creature's ability the table values at 0). A spell of no kind is worth 0
/// (0x43B680: the damage, raise, summon and obstacle spells, Stone Gaze, Bind).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Kind {
    /// Bloodlust (43; 0x438100).
    Bloodlust,
    /// Precision (44; 0x438B50).
    Precision,
    /// Slayer (55; 0x438D00).
    Slayer,
    /// Bless (41; 0x437430).
    Bless,
    /// Mirth (49; 0x438170).
    Mirth,
    /// Fortune (51; 0x438490).
    Fortune,
    /// Shield (27; 0x438C60).
    Shield,
    /// Air Shield (28; 0x438BC0).
    AirShield,
    /// Fire Shield (29; 0x43A110).
    FireShield,
    /// Stone Skin (46; 0x438D90).
    StoneSkin,
    /// Counterstrike (58; 0x439E80).
    Counterstrike,
    /// Frenzy (56; 0x4375D0).
    Frenzy,
    /// Haste (53; 0x4396B0).
    Haste,
    /// Prayer (48; 0x438AC0).
    Prayer,
    /// The Protections from Air, Fire, Water and Earth (30 to 33; 0x4399A0, 0x4399D0, 0x439A40,
    /// 0x439A00).
    Protection,
    /// Magic Mirror (36; 0x439DE0).
    MagicMirror,
}

impl Kind {
    /// The kind of `spell` by its data.
    pub(crate) fn of(spell: &BattleSpell) -> Option<Self> {
        let effects = &spell.magic.effects;
        let timed = |c: TimedChange| effects.contains(&SpellEffect::Timed(c));
        // Stone Skin raises the defense alone, Haste the speed alone; Prayer all three.
        if effects.as_slice() == [SpellEffect::Timed(TimedChange::Defense)] {
            return Some(Self::StoneSkin);
        }
        if effects.as_slice() == [SpellEffect::Timed(TimedChange::Speed)] {
            return Some(Self::Haste);
        }
        if timed(TimedChange::Attack) && timed(TimedChange::Defense) && timed(TimedChange::Speed) {
            return Some(Self::Prayer);
        }
        [
            (TimedChange::MeleeAttack, Self::Bloodlust),
            (TimedChange::RangedAttack, Self::Precision),
            (TimedChange::Slayer, Self::Slayer),
            (TimedChange::MaximumDamage, Self::Bless),
            (TimedChange::Morale, Self::Mirth),
            (TimedChange::Luck, Self::Fortune),
            (TimedChange::MeleeProtection, Self::Shield),
            (TimedChange::RangedProtection, Self::AirShield),
            (TimedChange::FireShield, Self::FireShield),
            (TimedChange::Retaliations, Self::Counterstrike),
            (TimedChange::Frenzy, Self::Frenzy),
            (TimedChange::SchoolProtection, Self::Protection),
            (TimedChange::MagicMirror, Self::MagicMirror),
        ]
        .into_iter()
        .find_map(|(c, kind)| timed(c).then_some(kind))
    }

    /// Its evaluator.
    fn evaluator(self) -> Evaluator {
        match self {
            Self::Bloodlust => |sc, s, c| sc.bloodlust(s, c),
            Self::Precision => |sc, s, c| sc.precision(s, c),
            Self::Slayer => |sc, s, c| sc.slayer(s, c),
            Self::Bless => |sc, s, c| sc.bless(s, c),
            Self::Mirth => |sc, s, c| sc.mirth(s, c),
            Self::Fortune => |sc, s, c| sc.fortune(s, c),
            Self::Shield => |sc, s, c| sc.shield(s, c),
            Self::AirShield => |sc, s, c| sc.air_shield(s, c),
            Self::FireShield => |sc, s, c| sc.fire_shield(s, c),
            Self::StoneSkin => |sc, s, c| sc.stone_skin(s, c),
            Self::Counterstrike => |sc, s, c| sc.counterstrike(s, c),
            Self::Frenzy => |sc, s, c| sc.frenzy(s, c),
            Self::Haste => |sc, s, c| sc.haste(s, c),
            Self::Prayer => |sc, s, c| sc.prayer(s, c),
            Self::Protection => |sc, s, c| sc.protection_from(s, c),
            Self::MagicMirror => |sc, s, c| sc.magic_mirror(s, c),
        }
    }
}

/// The computer's spell context of a side (0x4369C0 with the creature casts' flag 1; E23 T4b1).
pub(crate) struct SpellContext<'a> {
    state: &'a BattleState,
    ruleset: &'a BattleRuleset,
    /// The side (+0xC).
    side: BattleSide,
    /// The side's battle context (+0x20, 0x435EC0).
    ctx: Context,
    /// Its balance class (+0x3C, the battle context's +0x1C).
    class: u8,
    /// The enemy beaten (+0x1C): no stack of the other side but the war machines has hit points
    /// left after the planned damage.
    beaten: bool,
    /// The damage the predictions plan (each stack's +0x534).
    planned: Planned,
    /// The threat tables (0x43C040), built when first asked.
    tables: OnceCell<Tables>,
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
        (ctx, side): (&Context, BattleSide),
        rng: &mut Rng,
    ) -> Self {
        let planned = predict::plan(state, ruleset, side, rng);
        Self {
            state,
            ruleset,
            side,
            ctx: *ctx,
            class: balance(state, ruleset, ctx, side).class,
            beaten: beaten(state, &planned, side),
            planned,
            tables: OnceCell::new(),
            threats: [const { OnceCell::new() }; STACK_CAPACITY],
        }
    }

    /// Whether the predictions beat the enemy (+0x1C).
    #[cfg(test)]
    pub(crate) fn beaten(&self) -> bool {
        self.beaten
    }

    /// The value of the candidate `c` on stack `s` by its spell's evaluator ([`Kind`]; 0x43B690):
    /// 0 for a spell of no kind or one the ruleset lacks.
    pub(crate) fn value(&self, s: StackId, c: Candidate) -> i32 {
        // stub: BAT-STUB-4
        // The other evaluators and the threat tables (E23 T4c, in progress).
        self.ruleset
            .spell(c.spell)
            .and_then(Kind::of)
            .map_or(0, |k| k.evaluator()(self, s, c))
    }

    /// The threat record of stack `id` ([`computer::threat`]), computed once.
    fn threat(&self, id: StackId) -> Option<Threat> {
        *self.threats[id.index()]
            .get_or_init(|| computer::threat(self.state, self.ruleset, &self.ctx, id))
    }

    /// The enemy of stack `s`'s threat record when `s` attacks it within a turn (0x448BD0 at its
    /// speed: at most 1), else none.
    fn within_a_turn(&self, s: StackId) -> Option<StackId> {
        self.threat(s).filter(|t| t.turns <= 1).map(|t| t.enemy)
    }

    /// The candidate's value at its mastery (the spell's value, +0x34 of its record), 0 for a
    /// spell the ruleset lacks.
    fn effect(&self, c: Candidate) -> u16 {
        self.ruleset
            .spell(c.spell)
            .map_or(0, |sp| sp.effect[usize::from(c.mastery.min(3))])
    }

    /// The rounds' share of the battle's expected length a change of `rounds` rounds on stack `s`
    /// lasts (§7.45): `t = min(1, rounds / k)`, `k` the balance class, less `1 / k` (at least 0)
    /// for a stack that has acted this round. As the original's x87 doubles.
    fn share(&self, s: StackId, rounds: u32) -> X87 {
        let k = X87::int(u64::from(self.class));
        let t = self.share_of(rounds);
        if self.stack(s).flags.contains(StackFlags::ACTED) {
            // `minus` stops at 0, as the original's floor.
            t.minus(X87::ONE.over(k).unwrap_or(X87::ZERO))
        } else {
            t
        }
    }

    /// The rounds' share without the step for a stack that has acted: `min(1, rounds / k)`.
    fn share_of(&self, rounds: u32) -> X87 {
        if rounds >= u32::from(self.class) {
            X87::ONE
        } else {
            X87::int(u64::from(rounds))
                .over(X87::int(u64::from(self.class)))
                .unwrap_or(X87::ONE)
        }
    }

    /// The strength of stack `s` in the side's battle context (0x442E60 with its least bonuses),
    /// a double.
    fn strength(&self, s: StackId) -> X87 {
        let w = computer::strength(self.state, self.ruleset, &self.ctx, s);
        X87::int(u64::try_from(w).unwrap_or(0))
    }

    /// The gain of a strike of `d` by stack `s` on `e` raised by the factor `g` over `rounds`
    /// rounds (0x437800's end, inline in the evaluators of Bless, Fortune, Frenzy, Fire Shield and
    /// Counterstrike): the raised strike `d' = trunc(d × g)`, when it exceeds `e`'s hit points `h`
    /// (0x443080) `h` with `g = h / d`; 0 unless `d' > d`; else `trunc((√g − 1) × w × t)`, `w`
    /// the strength of `s` and `t` the rounds' share ([`Self::share`]): the rise of its worth,
    /// whose offense enters under a square root, for the share of the battle it lasts.
    fn gain(&self, (s, e): (StackId, StackId), (d, mut g): (u64, X87), rounds: u32) -> i32 {
        let mut raised = computer::ftol(X87::int(d).times(g));
        let h = computer::hit_points(self.state, e);
        let ih = i32::try_from(h).unwrap_or(i32::MAX);
        if raised > ih {
            raised = ih;
            g = X87::int(h).over(X87::int(d)).unwrap_or(X87::MAX);
        }
        if i64::from(raised) <= i64::try_from(d).unwrap_or(i64::MAX) {
            return 0;
        }
        let t = self.share(s, rounds);
        computer::ftol(g.sqrt().minus(X87::ONE).times(self.strength(s)).times(t))
    }

    /// The strike of stack `s` by its whole count on `e` (0x442780 with its shot flag, capped at
    /// `e`'s hit points, no run-up).
    fn strike(&self, (s, e): (StackId, StackId)) -> u64 {
        let (state, ruleset) = (self.state, self.ruleset);
        let shot = computer::shoots(state, ruleset, s);
        let count = u64::from(self.stack(s).count);
        computer::damage_estimate(state, ruleset, (s, e), count, shot, 0)
    }

    /// Stack `id`.
    fn stack(&self, id: StackId) -> &h3_battle::Stack {
        &self.state.stacks()[id.index()]
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
