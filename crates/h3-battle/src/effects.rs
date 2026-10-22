//! Timed effects on stacks (E04b T3a; ADR-0014 point 5): the battle's effect list, what a timed
//! spell does to a stack (BAT-MAG-009), how long it lasts and how it ends (BAT-MAG-010), and the
//! values the stack's rules read while it lasts (BAT-MAG-011).
//!
//! The list holds one entry per stack and spell, sorted by (stack, spell), so a stack's entries
//! are one contiguous run. It is reserved at the battle's start (and on every fork) for the usual
//! case, so steady play does not allocate. Each entry keeps what the cast changed on the stack's
//! attack, defense and speed; the stack's [`SpellValues`] are recomputed from its entries whenever
//! one of them changes, so the hot paths read inline values and never the list. The changes and
//! their order follow the original's code (`docs/formats/installation.md` §7.9): a change takes
//! effect at the cast and is given back when the effect ends, so a value can pass below 0 (a
//! defense that Disrupting Ray pushed to 0 when Stone Skin ends).

use alloc::vec::Vec;

use h3_content::{
    COUNTERSTRIKE_EXTRA_ROUND, MAX_SPELL_SCHOOLS, SINGLE_PRECISION_RATES, SpellCategory, SpellId,
};
use h3_content_battle::BattleSpell;
use h3_content_battle::{
    BattleRuleset, EffectDuration, EffectEnd, SpellEffect, SpellPolarity, TimedChange,
};
use h3_kernel::arith::{SINGLE_FRAC_BITS, single_precision};
use h3_kernel::{CanonicalHash, StateHasher};

use crate::event::{Event, EventKind, Sink};
use crate::magic::{SpellBonus, specialist_bonus};
use crate::state::{BattleState, Stack, StackFlags, StackId, StackTraits, UNLIMITED_RETALIATIONS};
use crate::{binds, vitality};

/// Entries reserved at the battle's start and on every fork (ADR-0014 point 5; measured in T3a).
pub const EFFECT_CAPACITY: usize = 32;

/// One timed effect on a stack (ADR-0014 point 5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActiveEffect {
    /// The stack.
    pub stack: StackId,
    /// The spell.
    pub spell: SpellId,
    /// Rounds left, the current one included; `None` for the rest of the battle.
    pub rounds: Option<u16>,
    /// `v`: the spell's effect at the first cast's mastery.
    pub value: u16,
    /// The first cast's mastery (Slayer reads it, BAT-MAG-014).
    pub mastery: u8,
    /// What the effect added to the stack's attack (negative: took from it).
    pub attack: i16,
    /// What it added to the stack's defense.
    pub defense: i16,
    /// What it added to the stack's speed.
    pub speed: i16,
}

impl CanonicalHash for ActiveEffect {
    /// Stack, spell, rounds (0: permanent), value, mastery (v21), attack, defense, speed.
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_u8(self.stack.0);
        self.spell.hash_into(h);
        h.write_u16(self.rounds.unwrap_or(0));
        h.write_u16(self.value);
        h.write_u8(self.mastery);
        for v in [self.attack, self.defense, self.speed] {
            h.write_bytes(&v.to_le_bytes());
        }
    }
}

/// The battle's timed effects, sorted by (stack, spell).
#[derive(Debug, PartialEq, Eq)]
pub struct Effects(Vec<ActiveEffect>);

impl Effects {
    /// An empty list with [`EFFECT_CAPACITY`] reserved.
    pub(crate) fn new() -> Self {
        Self(Vec::with_capacity(EFFECT_CAPACITY))
    }

    /// Every active effect, by stack and spell.
    pub fn as_slice(&self) -> &[ActiveEffect] {
        &self.0
    }

    /// The effects on `stack`, by spell.
    pub fn of(&self, stack: StackId) -> &[ActiveEffect] {
        let start = self.0.partition_point(|e| e.stack < stack);
        let end = self.0.partition_point(|e| e.stack <= stack);
        self.0.get(start..end).unwrap_or_default()
    }

    /// The position of `spell`'s entry on `stack`, or where it would go.
    fn find(&self, stack: StackId, spell: SpellId) -> Result<usize, usize> {
        self.0
            .binary_search_by(|e| (e.stack, e.spell).cmp(&(stack, spell)))
    }
}

impl Clone for Effects {
    /// A fork keeps the reserve (one allocation, ADR-0014 point 5).
    fn clone(&self) -> Self {
        let mut v = Vec::with_capacity(self.0.len().max(EFFECT_CAPACITY));
        v.extend_from_slice(&self.0);
        Self(v)
    }
}

impl CanonicalHash for Effects {
    /// Count, then the entries in order.
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_len(self.0.len());
        for e in &self.0 {
            e.hash_into(h);
        }
    }
}

/// What a stack's timed effects change, materialised from its entries (BAT-MAG-011). The
/// default is a stack of speed 0 without effects.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpellValues {
    /// Added to the attack (Prayer; Weakness takes).
    pub attack: i16,
    /// Added to the defense (Stone Skin, Prayer; Disrupting Ray takes).
    pub defense: i16,
    /// The speed now: the stack's speed with what its effects added, slowed (BAT-MAG-011).
    pub speed: u8,
    /// Added to the attack of melee attacks (Bloodlust; at most 255, which keeps the stack
    /// record at 72 bytes since v24).
    pub melee_attack: u8,
    /// Added to the attack of ranged attacks (Precision; at most 255).
    pub ranged_attack: u8,
    /// How each creature's base damage is rolled.
    pub damage: DamageRoll,
    /// The share of the speed left while slowed, percent (100: not slowed).
    pub speed_percent: u8,
    /// Added to the morale before its minimums and caps (Mirth, Sorrow).
    pub morale: i8,
    /// Added to the luck before its minimums and caps (Fortune, Misfortune).
    pub luck: i8,
    /// The share of melee damage let through, percent (100: none; Shield, BAT-MAG-012).
    pub melee_protection: u8,
    /// The share of ranged damage let through, percent (100: none; Air Shield, BAT-MAG-012).
    pub ranged_protection: u8,
    /// The share of spell damage let through per school handle, percent (100: none; the
    /// Protections, BAT-MAG-013).
    pub school_protection: [u8; MAX_SPELL_SCHOOLS],
    /// Slayer's attack bonus (0: none) and the mastery that decides the creatures it counts
    /// against (BAT-MAG-014).
    pub slayer: (u8, u8),
    /// The share of a melee strike's first-stage damage reflected onto the striker, percent
    /// (0: none; Fire Shield, BAT-MAG-018).
    pub fire_shield: u8,
    /// The share of its defense added to its attack, percent (0: not frenzied; Frenzy,
    /// BAT-MAG-019).
    pub frenzy: u16,
    /// Spells below this level are kept off it unless they are creature abilities (0: none;
    /// Anti-Magic, BAT-MAG-021).
    pub spell_immunity: u8,
    /// The chance that a single-target spell aimed at it from the other side goes to another
    /// stack, percent (0: none; Magic Mirror, BAT-MAG-022).
    pub mirror: u8,
    /// What its effects make of the stack (Forgetfulness, Blind; since v25, in the byte the
    /// record had spare).
    pub status: Status,
}

/// What a stack's timed effects make of it (BAT-MAG-023 to BAT-MAG-026), materialised with its
/// [`SpellValues`] so that the hot paths never read the effect list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Status(u8);

impl Status {
    /// A shooter that counts half its creatures for damage (Forgetfulness, BAT-MAG-023).
    pub const FORGETFUL: Self = Self(1);
    /// It cannot shoot (Forgetfulness cast at Advanced or Expert, BAT-MAG-023).
    pub const NO_SHOOTING: Self = Self(1 << 1);
    /// It loses its turns (Blind, BAT-MAG-024; Stone Gaze, Paralyze, BAT-MAG-043).
    pub const NO_TURNS: Self = Self(1 << 2);
    /// One of its effects ends when it takes damage (Blind, BAT-MAG-024; Stone Gaze, Paralyze,
    /// BAT-MAG-043).
    pub const DAMAGE_ENDS: Self = Self(1 << 3);
    /// Its turns are automatic and take any other stack; next to a stack it counts as an enemy
    /// (Berserk, BAT-MAG-025).
    pub const BERSERK: Self = Self(1 << 4);
    /// The other side controls it (Hypnotize, BAT-MAG-026).
    pub const HYPNOTIZED: Self = Self(1 << 5);
    /// One of its effects ends after its own attack (Berserk, BAT-MAG-025).
    pub const ATTACK_ENDS: Self = Self(1 << 6);
    /// It takes half the damage of strikes and does not retaliate (Stone Gaze, BAT-MAG-043).
    pub const PETRIFIED: Self = Self(1 << 7);

    /// Whether all bits of `other` are set.
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    /// The set with the bits of `other` added if `on`.
    #[must_use]
    pub const fn with(self, other: Self, on: bool) -> Self {
        if on { Self(self.0 | other.0) } else { self }
    }

    /// The raw bits.
    pub const fn bits(self) -> u8 {
        self.0
    }
}

impl SpellValues {
    /// The values of a stack with `speed` and no effects.
    pub(crate) const fn new(speed: u8) -> Self {
        Self {
            attack: 0,
            defense: 0,
            speed,
            melee_attack: 0,
            ranged_attack: 0,
            damage: DamageRoll::Random,
            speed_percent: 100,
            morale: 0,
            luck: 0,
            melee_protection: 100,
            ranged_protection: 100,
            school_protection: [100; MAX_SPELL_SCHOOLS],
            slayer: (0, 0),
            fire_shield: 0,
            frenzy: 0,
            spell_immunity: 0,
            mirror: 0,
            status: Status(0),
        }
    }
}

impl Default for SpellValues {
    fn default() -> Self {
        Self::new(0)
    }
}

impl CanonicalHash for SpellValues {
    /// Attack, defense, speed, melee and ranged attack (u8 since v24), the damage roll (code,
    /// value), the speed share, morale, luck; since v21 the melee, ranged and school protections
    /// and Slayer; since v22 Fire Shield and Frenzy; since v24 Anti-Magic's level and Magic
    /// Mirror's chance; since v25 the status bits.
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_bytes(&self.attack.to_le_bytes());
        h.write_bytes(&self.defense.to_le_bytes());
        h.write_u8(self.speed);
        h.write_u8(self.melee_attack);
        h.write_u8(self.ranged_attack);
        let (code, v) = match self.damage {
            DamageRoll::Random => (0, 0),
            DamageRoll::Maximum(v) => (1, v),
            DamageRoll::Minimum(v) => (2, v),
        };
        h.write_u8(code);
        h.write_u16(v);
        h.write_u8(self.speed_percent);
        h.write_bytes(&self.morale.to_le_bytes());
        h.write_bytes(&self.luck.to_le_bytes());
        h.write_u8(self.melee_protection);
        h.write_u8(self.ranged_protection);
        h.write_bytes(&self.school_protection);
        h.write_u8(self.slayer.0);
        h.write_u8(self.slayer.1);
        h.write_u8(self.fire_shield);
        h.write_u16(self.frenzy);
        h.write_u8(self.spell_immunity);
        h.write_u8(self.mirror);
        h.write_u8(self.status.bits());
    }
}

/// How a stack's creatures roll their base damage (BAT-MAG-011).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DamageRoll {
    /// A roll in the damage range (BAT-DMG-001).
    #[default]
    Random,
    /// The maximum plus the value for every creature (Bless).
    Maximum(u16),
    /// The minimum minus the value, at least 1, for every creature (Curse).
    Minimum(u16),
}

impl DamageRoll {
    /// The fixed damage per creature with range `[min, max]`, or `None` to roll.
    pub(crate) fn fixed(self, min: u16, max: u16) -> Option<u64> {
        match self {
            Self::Random => None,
            Self::Maximum(v) => Some(u64::from(max) + u64::from(v)),
            Self::Minimum(v) => Some(u64::from(min.saturating_sub(v).max(1))),
        }
    }
}

impl Stack {
    /// The attack of a melee or (`ranged`) a ranged attack now: the attack with what its effects
    /// added and Bloodlust's or Precision's bonus (BAT-MAG-011).
    pub fn attack_now(&self, ranged: bool) -> i32 {
        let bonus = if ranged {
            self.spells.ranged_attack
        } else {
            self.spells.melee_attack
        };
        i32::from(self.attack) + i32::from(self.spells.attack) + i32::from(bonus)
    }

    /// The defense now, with what its effects added (BAT-MAG-011; below 0 after Disrupting Ray
    /// and the end of Stone Skin).
    pub fn defense_now(&self) -> i32 {
        i32::from(self.defense) + i32::from(self.spells.defense)
    }

    /// The speed now, with its effects (BAT-MAG-011).
    pub const fn speed_now(&self) -> u8 {
        self.spells.speed
    }
}

/// A timed cast on one stack: the spell, its mastery and its duration.
#[derive(Debug, Clone, Copy)]
pub(crate) struct TimedCast<'r> {
    /// The spell.
    pub(crate) id: SpellId,
    /// Its data.
    pub(crate) spell: &'r BattleSpell,
    /// The caster's mastery.
    pub(crate) mastery: u8,
    /// The caster's power plus its duration bonus, at least 1 (BAT-MAG-010).
    pub(crate) rounds: u16,
    /// The casting hero's bonuses for the spell (none for a creature's cast; BAT-MAG-020).
    pub(crate) own: SpellBonus,
}

/// Applies `cast` to the living stack `s` (BAT-MAG-009): its counters end first; the value is the
/// spell's effect at the mastery plus, for a spell with a specialist's rule of its own, the
/// caster's specialist bonus for the stack (BAT-MAG-020; Slayer's counts at the strike); an
/// active effect of the same spell keeps its value and takes the longer duration, unless the
/// spell is cumulative, which applies its changes again; otherwise a new entry takes the
/// changes. The stack's values are recomputed and an `EffectApplied` event reports the rounds
/// left.
// rule: BAT-MAG-009
pub(crate) fn apply<S: Sink>(
    state: &mut BattleState,
    ruleset: &BattleRuleset,
    cast: TimedCast<'_>,
    s: StackId,
    sink: &mut S,
) {
    for &c in &cast.spell.counters {
        end(state, s, c, sink);
    }
    let rounds = match cast.spell.magic.duration {
        EffectDuration::Rounds => Some(cast.rounds),
        EffectDuration::Permanent => None,
        // rule: BAT-MAG-019
        EffectDuration::OwnTurn => Some(if state.active == Some(s) { 1 } else { 2 }),
        // rule: BAT-MAG-025
        EffectDuration::Fixed(n) => Some(n),
    };
    let stack = *state.at(s);
    let mut value = cast.spell.effect[usize::from(cast.mastery.min(3))];
    let changes = cast.spell.magic.effects.iter().filter_map(|e| e.timed());
    // rule: BAT-MAG-020
    if cast.spell.magic.specialist.is_some() && !changes.clone().any(|c| c == TimedChange::Slayer) {
        let bonus = specialist_bonus(ruleset, cast.spell, cast.own, stack.creature, value.into());
        value = u16::try_from(
            i64::from(value)
                .saturating_add(bonus)
                .clamp(0, u16::MAX.into()),
        )
        .unwrap_or(u16::MAX);
    }
    let now = (
        i32::from(stack.attack) + i32::from(stack.spells.attack),
        stack.defense_now(),
    );
    let rounds = match state.effects.find(s, cast.id) {
        Ok(i) => {
            let e = &mut state.effects.0[i];
            if cast.spell.magic.cumulative {
                let (a, d, sp) = deltas(changes, value, &stack, now);
                e.attack = e.attack.saturating_add(a);
                e.defense = e.defense.saturating_add(d);
                e.speed = e.speed.saturating_add(sp);
            } else {
                e.rounds = e.rounds.zip(rounds).map(|(a, b)| a.max(b));
            }
            e.rounds
        }
        Err(i) => {
            let (a, d, sp) = deltas(changes.clone(), value, &stack, now);
            state.effects.0.insert(
                i,
                ActiveEffect {
                    stack: s,
                    spell: cast.id,
                    rounds,
                    value,
                    mastery: cast.mastery,
                    attack: a,
                    defense: d,
                    speed: sp,
                },
            );
            // rule: BAT-MAG-015
            if changes.clone().any(|c| c == TimedChange::Retaliations) {
                add_retaliations(state.at_mut(s), value);
            }
            // rule: BAT-MAG-044
            if changes.clone().any(|c| c == TimedChange::Poison) {
                vitality::poison(state, s);
            }
            // rule: BAT-MAG-021
            if changes.into_iter().any(|c| c == TimedChange::SpellImmunity) {
                end_where(state, s, sink, |e| {
                    e.rounds.is_some()
                        && ruleset.spell(e.spell).is_some_and(|sp| {
                            sp.polarity == SpellPolarity::Negative
                                && sp.category != SpellCategory::Ability
                                && u16::from(sp.level) < value
                        })
                });
            }
            rounds
        }
    };
    recompute(state, ruleset, s);
    sink.emit(Event::public(EventKind::EffectApplied {
        target: s,
        spell: cast.id,
        rounds,
    }));
}

/// Adds `v` retaliations to those `stack` has left this round, unless it has unlimited ones
/// (Counterstrike, BAT-MAG-015).
fn add_retaliations(stack: &mut Stack, v: u16) {
    if stack.retaliations != UNLIMITED_RETALIATIONS {
        let extra = u8::try_from(v).unwrap_or(u8::MAX);
        stack.retaliations_left = stack.retaliations_left.saturating_add(extra);
        stack
            .flags
            .set(StackFlags::RETALIATED, stack.retaliations_left == 0);
    }
}

/// Gives every stack with Counterstrike its extra retaliations at a round start, after the reset
/// and before the effects count down (BAT-MAG-015). With quirk Q-0008, as the original does,
/// also an effect in its last round, which ends at this round start; without it, not.
// rule: BAT-MAG-015
pub(crate) fn round_retaliations(state: &mut BattleState, ruleset: &BattleRuleset) {
    let retaliations = SpellEffect::Timed(TimedChange::Retaliations);
    let last_round = ruleset.quirks().is_enabled(COUNTERSTRIKE_EXTRA_ROUND);
    let stacks = state.stacks.as_mut_slice();
    for e in &state.effects.0 {
        let extra = (last_round || e.rounds != Some(1))
            && ruleset
                .spell(e.spell)
                .is_some_and(|sp| sp.magic.effects.contains(&retaliations));
        if extra {
            add_retaliations(&mut stacks[e.stack.index()], e.value);
        }
    }
}

/// What `changes` with value `v` add to a stack's attack, defense and speed now (BAT-MAG-009), the
/// stack's current attack (without Bloodlust and Precision) and defense given: `speed` adds `v`
/// unless the stack is a war machine, `attack` and `defense` add `v`, `attack_loss` takes `v`
/// but never more than the attack (the original's comparison, also when the attack is below
/// 0), `defense_loss` lowers the defense by `v` to at least 0; the other changes are read while
/// the effect lasts.
fn deltas(
    changes: impl Iterator<Item = TimedChange>,
    v: u16,
    stack: &Stack,
    (mut attack, mut defense): (i32, i32),
) -> (i16, i16, i16) {
    let v = i32::from(v);
    let (mut a, mut d, mut s) = (0i32, 0i32, 0i32);
    for change in changes {
        match change {
            TimedChange::Speed if !stack.has(StackTraits::WAR_MACHINE) => s += v,
            TimedChange::Attack => {
                a += v;
                attack += v;
            }
            TimedChange::Defense => {
                d += v;
                defense += v;
            }
            TimedChange::AttackLoss => {
                let taken = v.min(attack);
                a -= taken;
                attack -= taken;
            }
            TimedChange::DefenseLoss => {
                let lowered = (defense - v).max(0);
                d += lowered - defense;
                defense = lowered;
            }
            _ => {}
        }
    }
    let narrow = |x: i32| i16::try_from(x.clamp(i16::MIN.into(), i16::MAX.into())).unwrap_or(0);
    (narrow(a), narrow(d), narrow(s))
}

/// Ends `spell`'s effect on stack `s`, if active, giving back what it changed, and reports it
/// (BAT-MAG-010). The caller recomputes the stack's values.
fn end<S: Sink>(state: &mut BattleState, s: StackId, spell: SpellId, sink: &mut S) {
    if let Ok(i) = state.effects.find(s, spell) {
        state.effects.0.remove(i);
        sink.emit(Event::public(EventKind::EffectEnded { target: s, spell }));
    }
}

/// Ends the timed effects on the living stack `s` that `spell`'s dispel effect ends
/// (BAT-MAG-016): those of the spell's `dispels`, or, without that list, every one but those of
/// its `spares`; a permanent effect never ends (BAT-MAG-010, the original's end routine). Each
/// ending is reported; the stack's values are recomputed if any ended.
// rule: BAT-MAG-016
pub(crate) fn dispel<S: Sink>(
    state: &mut BattleState,
    ruleset: &BattleRuleset,
    spell: &BattleSpell,
    s: StackId,
    sink: &mut S,
) {
    if end_where(state, s, sink, |e| dispel_ends(ruleset, spell, e)) {
        recompute(state, ruleset, s);
    }
}

/// Whether `spell`'s dispel effect ends the active effect `e` (BAT-MAG-016): a timed one of its
/// `dispels`, or, without that list, any timed one but those of its `spares`; for a dispel of
/// positive spells only one of a positive spell (Dispel Helpful Spells, BAT-MAG-042).
pub(crate) fn dispel_ends(ruleset: &BattleRuleset, spell: &BattleSpell, e: &ActiveEffect) -> bool {
    e.rounds.is_some()
        && (!spell.magic.dispels_positive
            || ruleset
                .spell(e.spell)
                .is_some_and(|sp| sp.polarity == SpellPolarity::Positive))
        && if spell.dispels.is_empty() {
            !spell.spares.contains(&e.spell)
        } else {
            spell.dispels.contains(&e.spell)
        }
}

/// Ends the effects on stack `s` of the spells `ends` selects, reporting each; the stack's values
/// are recomputed if any ended.
pub(crate) fn end_spells<S: Sink>(
    state: &mut BattleState,
    ruleset: &BattleRuleset,
    s: StackId,
    sink: &mut S,
    ends: impl Fn(&BattleSpell) -> bool,
) {
    if end_where(state, s, sink, |e| {
        ruleset.spell(e.spell).is_some_and(&ends)
    }) {
        recompute(state, ruleset, s);
    }
}

/// Ends the effects on stack `s` that `ends` selects, reporting each, and says whether any
/// ended. The caller recomputes the stack's values.
pub(crate) fn end_matching<S: Sink>(
    state: &mut BattleState,
    s: StackId,
    sink: &mut S,
    ends: impl Fn(&ActiveEffect) -> bool,
) -> bool {
    end_where(state, s, sink, ends)
}

/// Ends the effects on stack `s` that `ends` selects, reporting each, and says whether any
/// ended. The caller recomputes the stack's values.
fn end_where<S: Sink>(
    state: &mut BattleState,
    s: StackId,
    sink: &mut S,
    ends: impl Fn(&ActiveEffect) -> bool,
) -> bool {
    let mut i = state.effects.0.partition_point(|e| e.stack < s);
    let mut ended = false;
    while let Some(&e) = state.effects.0.get(i).filter(|e| e.stack == s) {
        if ends(&e) {
            state.effects.0.remove(i);
            ended = true;
            sink.emit(Event::public(EventKind::EffectEnded {
                target: s,
                spell: e.spell,
            }));
        } else {
            i += 1;
        }
    }
    ended
}

/// After stack `s` took damage: if it died, its clone vanishes or, a clone, it frees its original
/// (BAT-MAG-038); the effects on it that any damage ends end (BAT-MAG-024: Blind; BAT-MAG-043:
/// Stone Gaze, Paralyze; the original's 0x444090, `installation.md` §7.14, §7.20), reported.
// rule: BAT-MAG-024, BAT-MAG-043
pub(crate) fn damaged<S: Sink>(
    state: &mut BattleState,
    ruleset: &BattleRuleset,
    s: StackId,
    sink: &mut S,
) {
    // rule: BAT-MAG-038
    crate::created::died(state, ruleset, s, sink);
    if !state.at(s).spells.status.contains(Status::DAMAGE_ENDS) {
        return;
    }
    let ends = |e: &ActiveEffect| {
        ruleset
            .spell(e.spell)
            .is_some_and(|sp| sp.magic.ends == EffectEnd::Damaged)
    };
    // The status says an effect that damage ends is active, so one ends.
    end_where(state, s, sink, ends);
    recompute(state, ruleset, s);
}

/// Ends the effects on the living stack `s` that its own attack ends (BAT-MAG-025: Berserk; the
/// original's 0x441F55 and 0x4402F7, `installation.md` §7.14), after its melee attack or shot,
/// reporting each.
// rule: BAT-MAG-025
pub(crate) fn attacked<S: Sink>(
    state: &mut BattleState,
    ruleset: &BattleRuleset,
    s: StackId,
    sink: &mut S,
) {
    let stack = state.at(s);
    if !stack.is_alive() || !stack.spells.status.contains(Status::ATTACK_ENDS) {
        return;
    }
    let ends = |e: &ActiveEffect| {
        ruleset
            .spell(e.spell)
            .is_some_and(|sp| sp.magic.ends == EffectEnd::OwnAttack)
    };
    // The status says an effect that the attack ends is active, so one ends.
    end_where(state, s, sink, ends);
    recompute(state, ruleset, s);
}

/// The share of its retaliation's damage, percent, that stack `s` deals while it loses its turns:
/// Blind's value (BAT-MAG-024) or Paralyze's (BAT-MAG-043), the smaller if both; `None` if it
/// is neither.
pub(crate) fn retaliation_share(
    state: &BattleState,
    ruleset: &BattleRuleset,
    s: StackId,
) -> Option<u16> {
    let shares = [TimedChange::Blind, TimedChange::Paralyze].map(SpellEffect::Timed);
    if !state.at(s).spells.status.contains(Status::NO_TURNS) {
        return None;
    }
    state
        .effects
        .of(s)
        .iter()
        .filter(|e| {
            ruleset
                .spell(e.spell)
                .is_some_and(|sp| shares.iter().any(|c| sp.magic.effects.contains(c)))
        })
        .map(|e| e.value.min(100))
        .min()
}

/// Whether a timed `change` makes the stack lose its turns (BAT-MAG-024, BAT-MAG-043): a spell
/// after a strike with one stops the struck stack's retaliation (BAT-MAG-040). The content's
/// list, which the compiler keeps from lasting the whole battle.
pub(crate) const fn stops_turns(change: TimedChange) -> bool {
    change.takes_turns()
}

/// Recomputes the [`SpellValues`] of stack `s` from its entries (BAT-MAG-011): the sums of what
/// they added to attack, defense and speed; Bloodlust's and Precision's bonuses and the morale
/// and luck changes summed by kind; the base damage from Bless (which wins over Curse, as the
/// original checks it first) or Curse; the strongest slowing; then the speed: the stack's speed
/// with what was added, at least 0, and while slowed `max(1, ⌊speed × share⌋)` with the share
/// exact or, under quirk Q-0005, single precision as the original stores it; its hit points per
/// creature under Poison and Age (BAT-MAG-044); whether it is bound (BAT-MAG-045). Then every
/// stack's morale and luck are materialised again.
pub(crate) fn recompute(state: &mut BattleState, ruleset: &BattleRuleset, s: StackId) {
    let single = ruleset.quirks().is_enabled(SINGLE_PRECISION_RATES);
    let mut values = SpellValues::default();
    let (mut attack, mut defense, mut speed) = (0i32, 0i32, 0i32);
    let (mut maximum, mut minimum) = (None::<u16>, None::<u16>);
    let (mut morale, mut luck) = (0i32, 0i32);
    let (mut aged, mut bound) = (false, false);
    for e in state.effects.of(s) {
        attack += i32::from(e.attack);
        defense += i32::from(e.defense);
        speed += i32::from(e.speed);
        let v = e.value;
        let spell = ruleset.spell(e.spell);
        let changes = spell
            .map(|sp| sp.magic.effects.as_slice())
            .unwrap_or_default();
        let schools = spell.map(|sp| sp.schools.as_slice()).unwrap_or_default();
        let percent = u8::try_from(v.min(100)).unwrap_or(100);
        let small = u8::try_from(v).unwrap_or(u8::MAX);
        let ends = spell.map_or(EffectEnd::Never, |sp| sp.magic.ends);
        values.status = values
            .status
            .with(Status::DAMAGE_ENDS, ends == EffectEnd::Damaged)
            .with(Status::ATTACK_ENDS, ends == EffectEnd::OwnAttack);
        for change in changes.iter().filter_map(|c| c.timed()) {
            match change {
                TimedChange::MeleeAttack => {
                    values.melee_attack = values.melee_attack.saturating_add(small)
                }
                TimedChange::RangedAttack => {
                    values.ranged_attack = values.ranged_attack.saturating_add(small);
                }
                TimedChange::MaximumDamage => {
                    maximum = Some(maximum.unwrap_or(0).saturating_add(v))
                }
                TimedChange::MinimumDamage => {
                    minimum = Some(minimum.unwrap_or(0).saturating_add(v))
                }
                TimedChange::SpeedPercent => {
                    let p = u8::try_from(v.min(100)).unwrap_or(100);
                    values.speed_percent = values.speed_percent.min(p);
                }
                TimedChange::Morale => morale += i32::from(v),
                TimedChange::MoraleLoss => morale -= i32::from(v),
                TimedChange::Luck => luck += i32::from(v),
                TimedChange::LuckLoss => luck -= i32::from(v),
                TimedChange::MeleeProtection => {
                    values.melee_protection = values.melee_protection.min(percent);
                }
                TimedChange::RangedProtection => {
                    values.ranged_protection = values.ranged_protection.min(percent);
                }
                TimedChange::SchoolProtection => {
                    for school in schools {
                        let p = &mut values.school_protection
                            [school.index().min(MAX_SPELL_SCHOOLS - 1)];
                        *p = (*p).min(percent);
                    }
                }
                TimedChange::Slayer => {
                    values.slayer = (u8::try_from(v).unwrap_or(u8::MAX), e.mastery);
                }
                TimedChange::FireShield => values.fire_shield = values.fire_shield.max(percent),
                TimedChange::Frenzy => values.frenzy = values.frenzy.max(v),
                TimedChange::SpellImmunity => {
                    values.spell_immunity = values.spell_immunity.max(small);
                }
                TimedChange::MagicMirror => values.mirror = values.mirror.max(percent),
                // rule: BAT-MAG-023
                TimedChange::Forgetfulness => {
                    values.status = values
                        .status
                        .with(Status::FORGETFUL, true)
                        .with(Status::NO_SHOOTING, e.mastery >= 2);
                }
                // rule: BAT-MAG-024, BAT-MAG-043
                TimedChange::Blind | TimedChange::Paralyze => {
                    values.status = values.status.with(Status::NO_TURNS, true);
                }
                // rule: BAT-MAG-043
                TimedChange::Petrify => {
                    values.status = values
                        .status
                        .with(Status::NO_TURNS, true)
                        .with(Status::PETRIFIED, true);
                }
                // rule: BAT-MAG-044
                TimedChange::Age => aged = true,
                // rule: BAT-MAG-045
                TimedChange::Bind => bound = true,
                // rule: BAT-MAG-025
                TimedChange::Berserk => values.status = values.status.with(Status::BERSERK, true),
                // rule: BAT-MAG-026
                TimedChange::Hypnotize => {
                    values.status = values.status.with(Status::HYPNOTIZED, true);
                }
                _ => {}
            }
        }
    }
    let narrow16 = |x: i32| i16::try_from(x.clamp(i16::MIN.into(), i16::MAX.into())).unwrap_or(0);
    let narrow8 = |x: i32| i8::try_from(x.clamp(i8::MIN.into(), i8::MAX.into())).unwrap_or(0);
    values.attack = narrow16(attack);
    values.defense = narrow16(defense);
    values.morale = narrow8(morale);
    values.luck = narrow8(luck);
    values.damage = match (maximum, minimum) {
        (Some(v), _) => DamageRoll::Maximum(v),
        (None, Some(v)) => DamageRoll::Minimum(v),
        (None, None) => DamageRoll::Random,
    };
    let stack = state.at_mut(s);
    let base =
        u8::try_from((i32::from(stack.speed) + speed).clamp(0, u8::MAX.into())).unwrap_or(u8::MAX);
    values.speed = slowed(base, values.speed_percent, single);
    stack.spells = values;
    // rule: BAT-MAG-044
    vitality::materialize(state, ruleset, s, aged);
    binds::materialize(state, s, bound);
    crate::morale::materialize(state, ruleset);
}

/// `speed` slowed to `percent` % (100: not slowed): `max(1, ⌊speed × share⌋)`, the share exact or,
/// with `single` (quirk Q-0005), its single-precision value as the original stores it
/// (BAT-MAG-011).
pub(crate) fn slowed(speed: u8, percent: u8, single: bool) -> u8 {
    if percent >= 100 {
        return speed;
    }
    let speed = u64::from(speed);
    let value = if single {
        let rate = single_precision(u16::from(percent), 100).unwrap_or(0);
        (speed * rate) >> SINGLE_FRAC_BITS
    } else {
        speed * u64::from(percent) / 100
    };
    u8::try_from(value.max(1)).unwrap_or(u8::MAX)
}

/// Whether the effect of `spell` counts down at its stack's own turn start (Frenzy,
/// BAT-MAG-019) rather than at the round start.
fn on_own_turn(ruleset: &BattleRuleset, spell: SpellId) -> bool {
    ruleset
        .spell(spell)
        .is_some_and(|sp| sp.magic.duration == EffectDuration::OwnTurn)
}

/// Starts a round for the effects (BAT-MAG-010): on every living stack, each effect with rounds
/// either ends (its last round has passed) or loses a round; permanent effects stay, and those
/// counted at the stack's own turn wait for it ([`own_turn`]). Then Poison, where it still
/// lasts, takes another tenth (BAT-MAG-044; the original's 0x447120), reported when the hit
/// points fell below what they were once the ended effects were given back (an Age that ends
/// at the same round start ends first, as in the original's countdown).
// rule: BAT-MAG-010
pub(crate) fn count_down<S: Sink>(state: &mut BattleState, ruleset: &BattleRuleset, sink: &mut S) {
    if state.effects.0.is_empty() {
        return;
    }
    let mut changed = [false; crate::state::STACK_CAPACITY];
    let mut i = 0;
    while let Some(e) = state.effects.0.get_mut(i) {
        let (stack, spell) = (e.stack, e.spell);
        if !state.stacks.as_slice()[stack.index()].is_alive() || on_own_turn(ruleset, spell) {
            i += 1;
            continue;
        }
        match e.rounds {
            Some(1) => {
                state.effects.0.remove(i);
                changed[stack.index()] = true;
                sink.emit(Event::public(EventKind::EffectEnded {
                    target: stack,
                    spell,
                }));
            }
            Some(n) => {
                e.rounds = Some(n - 1);
                i += 1;
            }
            None => i += 1,
        }
    }
    // rule: BAT-MAG-044
    let poison = SpellEffect::Timed(TimedChange::Poison);
    let mut poisoned = [false; crate::state::STACK_CAPACITY];
    for e in &state.effects.0 {
        let alive = state.stacks.as_slice()[e.stack.index()].is_alive();
        if alive
            && ruleset
                .spell(e.spell)
                .is_some_and(|sp| sp.magic.effects.contains(&poison))
        {
            poisoned[e.stack.index()] = true;
        }
    }
    for (index, (&c, &p)) in changed.iter().zip(&poisoned).enumerate() {
        let id = StackId(u8::try_from(index).unwrap_or(u8::MAX));
        // What ended is given back before Poison's tenth is compared (the original recomputes
        // the hit points at Age's end, in the countdown, and compares with that value).
        if c {
            recompute(state, ruleset, id);
        }
        if p {
            let before = state.at(id).max_hp;
            vitality::poison(state, id);
            recompute(state, ruleset, id);
            let hit_points = state.at(id).max_hp;
            if hit_points < before {
                sink.emit(Event::public(EventKind::Poisoned {
                    stack: id,
                    hit_points,
                }));
            }
        }
    }
}

/// Starts stack `s`'s turn for its effects counted at its own turns (BAT-MAG-019): each either
/// ends (its last turn has come) or loses one. The caller calls this once per round, at the
/// stack's first turn (not after waiting).
// rule: BAT-MAG-019
pub(crate) fn own_turn<S: Sink>(
    state: &mut BattleState,
    ruleset: &BattleRuleset,
    s: StackId,
    sink: &mut S,
) {
    let mut i = state.effects.0.partition_point(|e| e.stack < s);
    let mut ended = false;
    while let Some(e) = state.effects.0.get_mut(i).filter(|e| e.stack == s) {
        match e.rounds {
            Some(n) if on_own_turn(ruleset, e.spell) => {
                if n > 1 {
                    e.rounds = Some(n - 1);
                    i += 1;
                } else {
                    let spell = e.spell;
                    state.effects.0.remove(i);
                    ended = true;
                    sink.emit(Event::public(EventKind::EffectEnded { target: s, spell }));
                }
            }
            _ => i += 1,
        }
    }
    if ended {
        recompute(state, ruleset, s);
    }
}

/// Removes the timed effects of dead stacks (BAT-MAG-010: a stack that dies loses its effects
/// with a duration) and recomputes what they changed, as the original's death routine ends every
/// spell with a duration (0x444120, `installation.md` §7.9): Age's hit points come back before a
/// raise counts them, Poison's lost tenths stay (BAT-MAG-044). A permanent effect stays on the
/// dead stack: the original's end routine refuses Disrupting Ray (0x444510 returns at once for
/// it), Acid Breath only lowers the defense, and neither the death nor a raise gives the defense
/// back, so a stack that is raised or reborn comes back with it (BAT-MAG-035, BAT-MAG-059).
pub(crate) fn remove_dead(state: &mut BattleState, ruleset: &BattleRuleset) {
    if state.effects.0.is_empty() {
        return;
    }
    let stacks = state.stacks.as_slice();
    let mut ended = 0u64;
    state.effects.0.retain(|e| {
        let stays = e.rounds.is_none() || stacks.get(e.stack.index()).is_some_and(Stack::is_alive);
        if !stays {
            ended |= 1 << e.stack.0;
        }
        stays
    });
    while ended != 0 {
        let s = StackId(u8::try_from(ended.trailing_zeros()).unwrap_or(u8::MAX));
        ended &= ended - 1;
        recompute(state, ruleset, s);
    }
}

/// Removes every effect of stack `s`, the permanent ones too, unreported: its record is set up
/// anew for a stack the battle creates (BAT-MAG-037; the original's 0x47A100 sets a reused record
/// up from scratch). The caller replaces the stack with its values.
pub(crate) fn forget(state: &mut BattleState, s: StackId) {
    state.effects.0.retain(|e| e.stack != s);
}
