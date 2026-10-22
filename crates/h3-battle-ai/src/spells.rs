//! The computer's spell context and its spell values (0x4369C0, 0x43A910, the evaluators of
//! 0x43B690; `docs/formats/installation.md` §7.42, §7.44 to §7.46, §7.51 to §7.53; BAT-AI-014 to
//! BAT-AI-016): what a cast on a stack is worth to the side. The creature casts that weigh a spell
//! (the Ogre Mage's and the Master Genie's scan, [`casts`](crate::casts)) build the context and
//! ask its evaluator for each stack.
//!
//! The context holds the damage the predictions plan ([`predict`]: what the side's stacks that
//! act before the other side's next one are expected to deal, with their draws), each stack's
//! threat record (0x422B20 with flag 0, [`computer::threat`]) and a half per side: its battle
//! context, balance class, "the enemy beaten" and threat tables ([`tables`]); the other side's
//! half is the original's second context (+0x48), which the removals value spells in. Each
//! spell's evaluator is told by the spell's data ([`Kind`]), as the original's table 0x43B690
//! tells it by the spell's number: the attack bonuses, luck, morale, speed and the second strikes
//! ([`boosts`]), the defenses and the spell protections ([`defenses`]), the harmful spells
//! ([`harms`]) and the removals ([`removals`]). The bolts', Dispel's and Clone's evaluators serve
//! only the hero's choice (E23 T5): they value 0 here.

pub(crate) mod boosts;
mod defenses;
mod harms;
mod removals;
pub(crate) mod tables;

use core::cell::{Cell, OnceCell};

use h3_battle::computer::{self, Context, Map, Planned, Threat};
use h3_battle::{
    BattleSide, BattleState, STACK_CAPACITY, StackFlags, StackId, StackTraits, Status,
};
use h3_content::{ORIGINAL_AI_SLIPS, SpellCategory, SpellId};
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
pub(crate) type Evaluator = fn(&View<'_>, StackId, Candidate) -> i32;

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
    /// Cure (37; 0x439C30).
    Cure,
    /// Anti-Magic (34; 0x439D40).
    AntiMagic,
    /// Curse (42; 0x43B370).
    Curse,
    /// Weakness (45; 0x438ED0).
    Weakness,
    /// Disrupting Ray (47; 0x438DC0).
    DisruptingRay,
    /// Sorrow (50; 0x4382C0).
    Sorrow,
    /// Misfortune (52; 0x438F60).
    Misfortune,
    /// Slow (54; 0x439270).
    Slow,
    /// Berserk (59; 0x43A400).
    Berserk,
    /// Hypnotize (60; 0x43A500).
    Hypnotize,
    /// Forgetfulness (61; 0x43B500).
    Forgetfulness,
    /// Blind and Paralyze (62, 74; 0x439100).
    Disable,
    /// Poison (71; 0x439500).
    Poison,
    /// Disease (73; 0x438A10).
    Disease,
    /// Age (75; 0x4373F0).
    Age,
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
        // Disease lowers the attack and the defense, Weakness the attack alone; Disrupting Ray
        // lowers the defense alone and a hero casts it (the Rust Dragons' Acid Breath is an
        // ability the table values at 0).
        if timed(TimedChange::AttackLoss) {
            return Some(if timed(TimedChange::DefenseLoss) {
                Self::Disease
            } else {
                Self::Weakness
            });
        }
        if timed(TimedChange::DefenseLoss) {
            return (spell.category != SpellCategory::Ability).then_some(Self::DisruptingRay);
        }
        // Cure ends harmful spells and heals (Dispel only ends: the hero's, E23 T5).
        if effects.contains(&SpellEffect::Dispel) && effects.contains(&SpellEffect::Heal) {
            return Some(Self::Cure);
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
            (TimedChange::SpellImmunity, Self::AntiMagic),
            (TimedChange::MinimumDamage, Self::Curse),
            (TimedChange::MoraleLoss, Self::Sorrow),
            (TimedChange::LuckLoss, Self::Misfortune),
            (TimedChange::SpeedPercent, Self::Slow),
            (TimedChange::Berserk, Self::Berserk),
            (TimedChange::Hypnotize, Self::Hypnotize),
            (TimedChange::Forgetfulness, Self::Forgetfulness),
            (TimedChange::Blind, Self::Disable),
            (TimedChange::Paralyze, Self::Disable),
            (TimedChange::Poison, Self::Poison),
            (TimedChange::Age, Self::Age),
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
            Self::Cure => |sc, s, c| sc.cure(s, c),
            Self::AntiMagic => |sc, s, c| sc.anti_magic(s, c),
            Self::Curse => |sc, s, c| sc.curse(s, c),
            Self::Weakness => |sc, s, c| sc.weakness(s, c),
            Self::DisruptingRay => |sc, s, c| sc.disrupting_ray(s, c),
            Self::Sorrow => |sc, s, c| sc.sorrow(s, c),
            Self::Misfortune => |sc, s, c| sc.misfortune(s, c),
            Self::Slow => |sc, s, c| sc.slow(s, c),
            Self::Berserk => |sc, s, c| sc.berserk(s, c),
            Self::Hypnotize => |sc, s, c| sc.hypnotize(s, c),
            Self::Forgetfulness => |sc, s, c| sc.forgetfulness(s, c),
            Self::Disable => |sc, s, c| sc.disable(s, c),
            Self::Poison => |sc, s, c| sc.poison(s, c),
            Self::Disease => |sc, s, c| sc.disease(s, c),
            Self::Age => |sc, s, c| sc.age(s, c),
        }
    }
}

/// The computer's spell context (0x4369C0 with the creature casts' flag 1; E23 T4b1, T4c): the
/// damage the predictions plan, the threat records of both sides (0x422B20 with flag 0, in the
/// side's battle context), the side's half and the other side's (the second context, +0x48),
/// built when first asked. The evaluators read it through a [`View`].
pub(crate) struct SpellContext<'a> {
    state: &'a BattleState,
    ruleset: &'a BattleRuleset,
    /// The damage the predictions plan (each stack's +0x534).
    planned: Planned,
    /// Each stack's threat record (0x422B20 with flag 0), computed the first time it is asked
    /// (ADR-0027 point 11: the records read nothing a value changes).
    threats: [OnceCell<Option<Threat>>; STACK_CAPACITY],
    /// The side's half.
    first: Half,
    /// The other side's (the second context, +0x48).
    second: OnceCell<Half>,
    /// With BAT-QUIRK-25 (survey slip 22), the map of the last search an evaluator ran in the
    /// computer's pathfinder (Berserk's and Hypnotize's, 0x445582, 0x43A5DB), which the walker's
    /// fallbacks then read.
    searched: Cell<Option<Map>>,
}

/// One side's half of the spell context (the context's own fields, 0x4369C0, or the second
/// context's).
pub(crate) struct Half {
    /// The side (+0xC).
    side: BattleSide,
    /// The side's battle context (+0x20, 0x435EC0).
    ctx: Context,
    /// Its balance class (+0x3C, the battle context's +0x1C).
    class: u8,
    /// The enemy beaten (+0x1C): no stack of the other side but the war machines has hit points
    /// left after the planned damage.
    beaten: bool,
    /// The other side's able strength (+0x38, the battle context's +0x18).
    other_able: i32,
    /// The threat tables (0x43C040), built when first asked.
    tables: OnceCell<Tables>,
}

impl Half {
    /// The half of `side` with its battle context `ctx`: its balance class and the other side's
    /// able strength ([`balance`]) and "the enemy beaten" after the damage `planned`
    /// ([`beaten`]).
    fn new(
        state: &BattleState,
        ruleset: &BattleRuleset,
        (ctx, side): (Context, BattleSide),
        planned: &Planned,
    ) -> Self {
        let balance = balance(state, ruleset, &ctx, side);
        Self {
            side,
            ctx,
            class: balance.class,
            beaten: beaten(state, planned, side),
            other_able: balance.other,
            tables: OnceCell::new(),
        }
    }
}

impl<'a> SpellContext<'a> {
    /// The spell context of `side` with its battle context `ctx` (0x4369C0; BAT-AI-015): the
    /// predictions ([`predict::plan`], their walkers' draws from `rng`), then the side's half.
    // rule: BAT-AI-015
    pub(crate) fn new(
        state: &'a BattleState,
        ruleset: &'a BattleRuleset,
        (ctx, side): (&Context, BattleSide),
        rng: &mut Rng,
    ) -> Self {
        let planned = predict::plan(state, ruleset, side, rng);
        let first = Half::new(state, ruleset, (*ctx, side), &planned);
        Self {
            state,
            ruleset,
            planned,
            threats: [const { OnceCell::new() }; STACK_CAPACITY],
            first,
            second: OnceCell::new(),
            searched: Cell::new(None),
        }
    }

    /// Whether the original's slips are kept (BAT-QUIRK-25).
    fn slips(&self) -> bool {
        self.ruleset.quirks().is_enabled(ORIGINAL_AI_SLIPS)
    }

    /// An evaluator's search left `map` in the computer's pathfinder: with BAT-QUIRK-25 (survey
    /// slip 22) the context keeps it.
    fn searched(&self, map: Map) {
        if self.slips() {
            self.searched.set(Some(map));
        }
    }

    /// The map the computer's pathfinder holds after the context and its values (survey slip
    /// 22): the last evaluator's search, else the threat records' last
    /// ([`computer::last_searched`]); none when nothing searched (only a hypnotized walker's
    /// context can meet that: every other walker's own record searches).
    pub(crate) fn cells(&self) -> Option<Map> {
        self.searched
            .take()
            .or_else(|| last_searched(self.state, self.ruleset))
    }

    /// The side's half on the battle (the context the casts ask).
    pub(crate) fn view(&self) -> View<'_> {
        View::of(self, &self.first, (self.state, None))
    }

    /// The value of the candidate `c` on stack `s` ([`View::value`]).
    pub(crate) fn value(&self, s: StackId, c: Candidate) -> i32 {
        self.view().value(s, c)
    }

    /// The value of the candidate `c` cast on the side's whole army ([`View::on_side`]).
    pub(crate) fn on_side(&self, c: Candidate, value: Evaluator) -> i32 {
        self.view().on_side(c, value)
    }

    /// The other side's half (0x4369C0's step 6: its own battle context, its "enemy beaten" for
    /// the side's own stacks after the planned damage), built when first asked.
    fn second(&self) -> &Half {
        self.second.get_or_init(|| {
            let side = self.first.side.opponent();
            let ctx = computer::context(self.state, self.ruleset, side);
            Half::new(self.state, self.ruleset, (ctx, side), &self.planned)
        })
    }

    /// The threat record of stack `id` ([`computer::threat`] in the side's battle context, on
    /// the battle), computed once.
    fn threat(&self, id: StackId) -> Option<Threat> {
        *self.threats[id.index()]
            .get_or_init(|| computer::threat(self.state, self.ruleset, &self.first.ctx, id))
    }
}

/// The map the computer's pathfinder holds after the threat records of both sides (0x436ABA;
/// survey slip 22): the last searched stack's ([`computer::last_searched`]) for its side, none
/// when no record searched.
pub(crate) fn last_searched(state: &BattleState, ruleset: &BattleRuleset) -> Option<Map> {
    computer::last_searched(state, ruleset)
        .map(|id| computer::map(state, ruleset, id, state.stacks()[id.index()].side))
}

/// A half of the spell context on a state: the battle's, or a copy with spells ended (a
/// removal's, 0x439A80, which values each spell on the stack's copy). Its records (the threat
/// records, the tables) are the context's, as the original's copy keeps them. With BAT-QUIRK-25
/// (survey slip 24) a removal's copy is a lone record beside the battle ([`Self::lone`]).
#[derive(Clone, Copy)]
pub(crate) struct View<'s> {
    state: &'s BattleState,
    ruleset: &'s BattleRuleset,
    root: &'s SpellContext<'s>,
    half: &'s Half,
    side: BattleSide,
    ctx: &'s Context,
    class: u8,
    beaten: bool,
    planned: &'s Planned,
    /// With BAT-QUIRK-25 (survey slip 24), on a removal's copy: the battle, whose records the
    /// original walks beside the copy (0x437A00 copies the stack's record alone, the battle
    /// untouched). Its record of the stack valued still holds every spell, and no record is the
    /// copy: the original tells them apart by pointer.
    lone: Option<&'s BattleState>,
}

impl<'s> View<'s> {
    /// The view of `half` of `root` on `state`, `lone` the battle beside a lone copy.
    fn of(
        root: &'s SpellContext<'s>,
        half: &'s Half,
        (state, lone): (&'s BattleState, Option<&'s BattleState>),
    ) -> Self {
        Self {
            state,
            ruleset: root.ruleset,
            root,
            half,
            side: half.side,
            ctx: &half.ctx,
            class: half.class,
            beaten: half.beaten,
            planned: &root.planned,
            lone,
        }
    }

    /// The same half on `state` (the stack valued is the state's own record).
    pub(crate) fn on<'t>(&self, state: &'t BattleState) -> View<'t>
    where
        's: 't,
    {
        View::of(self.root, self.half, (state, None))
    }

    /// The same half on `copy`, a removal's lone copy of the valued stack's record beside the
    /// battle (survey slip 24): [`Self::lone`].
    pub(crate) fn on_copy<'t>(&self, copy: &'t BattleState) -> View<'t>
    where
        's: 't,
    {
        View::of(self.root, self.half, (copy, Some(self.root.state)))
    }

    /// The other half on the same state (the second context, or from it the first).
    pub(crate) fn other(&self) -> Self {
        let half = if core::ptr::eq(self.half, &self.root.first) {
            self.root.second()
        } else {
            &self.root.first
        };
        View::of(self.root, half, (self.state, self.lone))
    }

    /// Whether the predictions beat the enemy (+0x1C).
    #[cfg(test)]
    pub(crate) fn beaten(&self) -> bool {
        self.beaten
    }

    /// The value of the candidate `c` on stack `s` by its spell's evaluator ([`Kind`]; 0x43B690):
    /// 0 for a spell of no kind or one the ruleset lacks.
    pub(crate) fn value(&self, s: StackId, c: Candidate) -> i32 {
        self.ruleset
            .spell(c.spell)
            .and_then(Kind::of)
            .map_or(0, |k| k.evaluator()(self, s, c))
    }

    /// The threat record of stack `id` (the context's, [`SpellContext::threat`]).
    fn threat(&self, id: StackId) -> Option<Threat> {
        self.root.threat(id)
    }

    /// The enemy of stack `s`'s threat record when `s` attacks it within a turn (0x448BD0 at its
    /// speed now, at most 1), else none. The original counts the turns as it values, on the record
    /// it is given: a removal's copy counts them at its own speed and shot (E23 T4d: an ended Slow,
    /// Haste, Prayer or Forgetfulness changes them); on the battle the record's turns are those.
    fn within_a_turn(&self, s: StackId) -> Option<StackId> {
        let threat = self.threat(s)?;
        let turns = if core::ptr::eq(self.state, self.root.state) {
            threat.turns
        } else {
            self.turns_at(s, u32::from(self.stack(s).speed_now()))
        };
        (turns <= 1).then_some(threat.enemy)
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

    /// The strength of stack `s` in the half's battle context (0x442E60 with its least bonuses),
    /// a double.
    fn strength(&self, s: StackId) -> X87 {
        X87::int(u64::try_from(self.strength_of(s)).unwrap_or(0))
    }

    /// The strength of stack `s` in the half's battle context (0x442E60), as the original's
    /// integer.
    fn strength_of(&self, s: StackId) -> i32 {
        computer::strength(self.state, self.ruleset, self.ctx, s)
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
    fn stack(&self, id: StackId) -> &'s h3_battle::Stack {
        &self.state.stacks()[id.index()]
    }

    /// Whether the ruleset ports the original's slips (BAT-QUIRK-25).
    fn slips(&self) -> bool {
        self.ruleset.quirks().is_enabled(ORIGINAL_AI_SLIPS)
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
