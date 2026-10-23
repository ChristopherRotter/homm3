//! The computer's hero cast (`docs/formats/installation.md` §7.55; BAT-AI-017, BAT-AI-018): at
//! the activation of each stack of its side but a war machine, before the stack acts, a hero with
//! a spell book that has not cast this round and may cast builds the spell context for its side
//! (0x422DA0, [`SpellContext`]) and weighs every spell it can cast now (0x43C800): the spell's
//! value (0x43BB20, [`value`]), 2.5 times when its spell points cover seven casts, else times the
//! square root of the casts they cover, drawn at 75 to 100 %; the best is cast when it is due now
//! (+0x20) or when the side would retreat. The raises, Sacrifice and Earthquake have cases of
//! their own ([`specials`], E23 T5b1), Teleport too ([`teleport`], E23 T5b2a: its stacks' walker
//! turns, a caster's cast entry in them but the active stack's E23 T5b2b's, BAT-STUB-14); the
//! retreat test is T6's (BAT-STUB-13, BAT-Q61).

mod specials;
mod teleport;

use core::cell::OnceCell;

use h3_battle::computer::{self, HeroCast};
use h3_battle::{
    BattleSide, BattleState, CastTarget, CastTargets, Command, HexSet, STACK_CAPACITY, StackFlags,
    StackId, StackTraits, Status, cast_targets, castable_spells,
};
use h3_content::{ORIGINAL_AI_SLIPS, SpellId};
use h3_content_battle::{BattleRuleset, BattleSpell, SpellEffect, SpellPolarity, SpellTarget};
use h3_kernel::Rng;
use h3_kernel::arith::X87;

use crate::casts::{self, Scanner, Spell};
use crate::order;
use crate::spells::{Candidate, Kind, SpellContext, has_spell};

/// The choice's draw (0x43CA60: `random(75, 100)`), per spell with a value.
const DRAW: (u32, u32) = (75, 100);

/// The casts the spell points must cover for the value's 2.5 times (0x43CA1D).
const CASTS: i32 = 7;

/// The command of `side`'s hero at the activation of its stack `id` (0x422DA0; BAT-AI-017), its
/// draws from `rng`: none without a hero with a spell book, once it has cast this round, while a
/// hero forbids casting (BAT-MAG-061) or when `id` is a war machine; else the spell context for
/// the side (its predictions' draws) and the choice ([`choose`]). The policy plays any side as
/// the computer's: the original's tests for a human side are not asked (ADR-0027 point 8).
// rule: BAT-AI-017
pub(crate) fn cast(
    state: &BattleState,
    ruleset: &BattleRuleset,
    (id, side): (StackId, BattleSide),
    rng: &mut Rng,
) -> Option<Command> {
    let hero = state.hero(side)?;
    if !hero.spell_book
        || hero.casts > 0
        || computer::casting_forbidden(state)
        || state.stacks()[id.index()].has(StackTraits::WAR_MACHINE)
    {
        return None;
    }
    let ctx = computer::context(state, ruleset, side);
    let mut sc = SpellContext::new(state, ruleset, (&ctx, side), rng);
    // stub: BAT-STUB-13
    // The retreat test (0x41E570) reads the adventure's state no seam carries (BAT-Q61): no
    // retreat until T6, so the choice is never desperate.
    choose(&mut sc, (state, ruleset), (id, side, ctx), false, rng)
}

/// A spell's value for the hero's choice (0x43BB20's candidate after it): the value, the target
/// and the second target the cast takes (+0x14, +0x18: Sacrifice's victim) and whether the cast
/// is due now (+0x20).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Valued {
    pub(crate) value: i32,
    pub(crate) target: CastTarget,
    pub(crate) second: CastTarget,
    pub(crate) now: bool,
}

impl Valued {
    /// Nothing: a value of 0.
    const ZERO: Self = Self {
        value: 0,
        target: CastTarget::None,
        second: CastTarget::None,
        now: false,
    };

    /// A value due now without a target (0x43BB20's cases of the spells without one).
    const fn untargeted(value: i32) -> Self {
        Self {
            value,
            now: true,
            ..Self::ZERO
        }
    }
}

/// The hero's choice (0x43C800; BAT-AI-017) in the spell context `sc` of `side`, `id` the active
/// stack, `retreat` the retreat test's answer: with it the context is desperate and only damage
/// spells count. When the predictions beat the enemy and no living stack of the side would lose
/// a creature to them, only the raises count. Over the spells the engine lets the hero cast now
/// in their order: each spell's value ([`value`]) above 0 is `trunc(5v / 2)` with spell points
/// for at least seven casts, else `trunc(v × √⌊mana / cost⌋)` (doubles), then `trunc(r × v /
/// 100)` with `r` drawn from 75 to 100; a strictly larger one is the best (the earlier spell at a
/// tie). The best is cast at its target when it is due now or the side would retreat.
// rule: BAT-AI-017
pub(crate) fn choose(
    sc: &mut SpellContext<'_>,
    (state, ruleset): (&BattleState, &BattleRuleset),
    (id, side, ctx): (StackId, BattleSide, computer::Context),
    retreat: bool,
    rng: &mut Rng,
) -> Option<Command> {
    if retreat {
        sc.desperately();
    }
    let raises_only = sc.beaten() && !sc.at_risk();
    let hero = Hero::new(sc, (state, ruleset), (id, side, ctx));
    let mut best: Option<(SpellId, Valued)> = None;
    let mut top = 0i32;
    let spells = castable_spells(state, ruleset, side);
    for (spell, s) in spells
        .iter()
        .filter_map(|id| ruleset.spell(id).map(|s| (id, s)))
    {
        let effects = &s.magic.effects;
        if (retreat && !effects.contains(&SpellEffect::Damage))
            || (raises_only && !effects.contains(&SpellEffect::Raise))
        {
            continue;
        }
        let hc = computer::hero_cast(state, ruleset, side, (spell, s));
        let c = Candidate {
            spell,
            mastery: hc.mastery,
            power: u32::from(hc.power),
            rounds: u32::from(hc.rounds),
            cast: true,
        };
        let v = value(&hero, (spell, s), c, rng);
        if v.value <= 0 {
            continue;
        }
        let r = i32::try_from(rng.range_inclusive(DRAW.0, DRAW.1)).unwrap_or(100);
        let drawn = r.wrapping_mul(scaled(v.value, hc)) / 100;
        if drawn > top {
            top = drawn;
            best = Some((spell, v));
        }
    }
    best.filter(|(_, v)| v.now || retreat)
        .map(|(spell, v)| Command::Cast {
            spell,
            target: v.target,
            second: v.second,
        })
}

/// A value `v` above 0 by the casts the hero's spell points cover (0x43CA0B to 0x43CA4E): `trunc(5v
/// / 2)` (32 bits) when they cover at least seven (a free spell always), else `trunc(v ×
/// √⌊mana / cost⌋)` as doubles.
pub(crate) fn scaled(v: i32, hc: HeroCast) -> i32 {
    let (mana, cost) = (i32::from(hc.mana), i32::from(hc.cost));
    if mana >= cost.wrapping_mul(CASTS) {
        v.wrapping_mul(5) / 2
    } else {
        // The cost is above 0 here: the spell points cover fewer than seven casts.
        let casts = u64::try_from(mana / cost.max(1)).unwrap_or(0);
        let v = u64::try_from(v).unwrap_or(0);
        computer::ftol(X87::int(casts).sqrt().times(X87::int(v)))
    }
}

/// The hero's side in its spell context, as the values read it.
pub(crate) struct Hero<'h, 'a> {
    state: &'a BattleState,
    ruleset: &'a BattleRuleset,
    side: BattleSide,
    /// The active stack (+0x132B8, +0x132BC).
    active: StackId,
    sc: &'h SpellContext<'a>,
    /// The side's stack for the damage spells' estimates (the context's battle context).
    scanner: Scanner<'a>,
    /// The order of action's ranks (+0x190: the context's 0x41F140), computed when "now" on an
    /// enemy first asks them.
    ranks: OnceCell<[u8; STACK_CAPACITY]>,
}

/// The engine's stack and hex targets of a hero's cast now (a stack mask and a hex set for quick
/// tests; a mass version's target is the engine's whenever the spell is castable).
pub(crate) struct Targets {
    stacks: u64,
    hexes: HexSet,
}

impl Targets {
    /// Whether stack `s` is one of them.
    const fn takes_stack(&self, s: StackId) -> bool {
        self.stacks & (1u64 << s.index()) != 0
    }

    /// Whether hex `h` is one of them.
    const fn takes_hex(&self, h: h3_battle::Hex) -> bool {
        self.hexes.contains(h)
    }
}

impl<'h, 'a> Hero<'h, 'a> {
    /// `side`'s hero in the spell context `sc`, `id` the active stack, `ctx` the side's battle
    /// context.
    pub(crate) fn new(
        sc: &'h SpellContext<'a>,
        (state, ruleset): (&'a BattleState, &'a BattleRuleset),
        (id, side, ctx): (StackId, BattleSide, computer::Context),
    ) -> Self {
        Self {
            state,
            ruleset,
            side,
            active: id,
            sc,
            scanner: Scanner::new(state, ruleset, (id, side), ctx),
            ranks: OnceCell::new(),
        }
    }

    /// The engine's targets of the hero's cast of `spell` now.
    pub(crate) fn targets(&self, spell: SpellId) -> Targets {
        let mut list = CastTargets::new();
        cast_targets(self.state, self.ruleset, self.side, spell, &mut list);
        let list = list.as_slice();
        Targets {
            stacks: list.iter().fold(0, |mask, t| match *t {
                CastTarget::Stack(s) => mask | 1u64 << s.index(),
                _ => mask,
            }),
            hexes: list.iter().fold(HexSet::new(), |mut set, t| {
                if let CastTarget::Hex(h) = *t {
                    set.insert(h);
                }
                set
            }),
        }
    }

    /// Whether stack `s` of `of` can act this round apart from the active stack (0x43A879,
    /// 0x436CE0): alive, no war machine, not blind, turned to stone or paralysed, not acted.
    fn can_act(&self, s: StackId) -> bool {
        let st = &self.state.stacks()[s.index()];
        s != self.active
            && st.is_alive()
            && !st.has(StackTraits::WAR_MACHINE)
            && !st.spells.status.contains(Status::NO_TURNS)
            && !st.flags.contains(StackFlags::ACTED)
    }

    /// Whether no stack of the side but the active one can still act this round.
    fn last_to_act(&self) -> bool {
        !(0u8..)
            .zip(self.state.stacks())
            .any(|(i, s)| s.side == self.side && self.can_act(StackId(i)))
    }
}

/// A spell's value for the hero's choice (0x43BB20; BAT-AI-017 to BAT-AI-020), its candidate `c`:
/// Chain Lightning's ([`chain`]), an area spell's ([`area`]) and a mass damage spell's
/// ([`mass_damage`]), the summons' ([`summon`]), Dispel's ([`dispel`]), the raises', Sacrifice's
/// and Earthquake's ([`specials`]) and Teleport's ([`teleport`], its walker turns' draws from
/// `rng`) by their own cases; every other spell aimed at stacks by its evaluator ([`Kind`]) on the
/// side's stacks for a positive spell, the other side's for a negative one ([`evaluated`]; the
/// original weighs a neutral one on both, the only such spell Dispel, whose case falls into this
/// one, [`dispel`]); any other nothing: the computer's hero never casts Quicksand, Land Mine,
/// Force Field, Fire Wall, Berserk (aimed at a hex) or Remove Obstacle.
// rule: BAT-AI-017, BAT-AI-018, BAT-AI-019, BAT-AI-020
pub(crate) fn value(
    h: &Hero<'_, '_>,
    (id, s): (SpellId, &BattleSpell),
    c: Candidate,
    rng: &mut Rng,
) -> Valued {
    let m = &s.magic;
    let has = |e: SpellEffect| m.effects.contains(&e);
    if has(SpellEffect::Teleport) {
        return teleport::teleport(h, id, rng);
    }
    if has(SpellEffect::Earthquake) {
        return specials::earthquake(h, s, c);
    }
    if has(SpellEffect::Sacrifice) {
        return specials::sacrifice(h, (id, s), c);
    }
    if has(SpellEffect::Raise) {
        return specials::raise(h, (id, s), c);
    }
    if has(SpellEffect::Damage) && m.target != SpellTarget::Stack {
        let factor = u64::from(s.power);
        let at = usize::from(c.mastery.min(3));
        let spell = Spell {
            id,
            mastery: c.mastery,
            damage: factor * u64::from(c.power) + u64::from(s.effect[at]),
        };
        return if m.target == SpellTarget::Hex {
            area(h, spell)
        } else {
            mass_damage(h, spell)
        };
    }
    if has(SpellEffect::Damage) && m.chain.is_some() {
        let factor = u64::from(s.power);
        let at = usize::from(c.mastery.min(3));
        let spell = Spell {
            id,
            mastery: c.mastery,
            damage: factor * u64::from(c.power) + u64::from(s.effect[at]),
        };
        return chain(h, spell);
    }
    if has(SpellEffect::Summon) {
        return summon(h, s, c);
    }
    if Kind::of(s) == Some(Kind::Dispel) {
        return dispel(h, c);
    }
    let of = if s.polarity == SpellPolarity::Negative {
        h.side.opponent()
    } else {
        h.side
    };
    if m.target == SpellTarget::Stack {
        evaluated(h, of, c)
    } else {
        Valued::ZERO
    }
}

/// A spell aimed at stacks valued by its evaluator on `of`'s stacks (0x43A910; BAT-AI-017):
/// aimed at one stack at the candidate's mastery, the best target ([`one_target`]); its mass
/// version (no target, which the engine takes: the spell is castable), the sum over `of`'s stacks
/// ([`on_stacks`]), due now.
fn evaluated(h: &Hero<'_, '_>, of: BattleSide, c: Candidate) -> Valued {
    if computer::takes_stack(h.ruleset, c.spell, c.mastery) {
        one_target(h, of, c, &h.targets(c.spell))
    } else {
        Valued::untargeted(on_stacks(h, of, c))
    }
}

/// Whether the hero's candidate `c` is weighed on stack `s` (0x43A6C5 to 0x43A74E, as 0x43A910's
/// mass path): a living stack not blind, turned to stone or paralysed, no First Aid Tent or Ammo
/// Cart, without the spell, that the side's cast can affect ([`computer::spell_affects`]).
fn weighed(h: &Hero<'_, '_>, s: StackId, c: Candidate) -> bool {
    let st = &h.state.stacks()[s.index()];
    st.is_alive()
        && !st.spells.status.contains(Status::NO_TURNS)
        && !st.has(StackTraits::FIRST_AID)
        && !st.has(StackTraits::AMMO_SUPPLY)
        && !has_spell(h.state, s, c.spell)
        && computer::spell_affects(h.state, h.ruleset, h.side, c.spell, s)
}

/// The sum of the candidate's values over `of`'s stacks it weighs ([`weighed`]), in record order,
/// as 32-bit integers (0x43A910's mass path).
pub(crate) fn on_stacks(h: &Hero<'_, '_>, of: BattleSide, c: Candidate) -> i32 {
    let view = h.sc.view();
    (0u8..)
        .zip(h.state.stacks())
        .filter(|&(i, s)| s.side == of && weighed(h, StackId(i), c))
        .fold(0i32, |sum, (i, _)| {
            sum.wrapping_add(view.value(StackId(i), c))
        })
}

/// The best target of the candidate `c` among `of`'s stacks (0x43A670; BAT-AI-017): over those it
/// weighs ([`weighed`]) and the engine takes, in record order, each one's value by the spell's
/// evaluator; on a stack of the other side under a cast Magic Mirror a positive value of any spell
/// but Dispel becomes `trunc(2v(50 − c) / 100)`, `c` the mirror's chance (32 bits); the first
/// strictly above the best so far (from 0) is the target. Then whether it is due now
/// ([`now_on_own`], [`now_on_enemy`]); Haste always.
pub(crate) fn one_target(
    h: &Hero<'_, '_>,
    of: BattleSide,
    c: Candidate,
    targets: &Targets,
) -> Valued {
    let view = h.sc.view();
    let dispel = h.ruleset.spell(c.spell).and_then(Kind::of) == Some(Kind::Dispel);
    let mut best = Valued::ZERO;
    let mut target = None;
    for (i, st) in (0u8..).zip(h.state.stacks()) {
        let s = StackId(i);
        if st.side != of || !weighed(h, s, c) || !targets.takes_stack(s) {
            continue;
        }
        let mut v = view.value(s, c);
        let chance = i32::from(computer::cast_mirror_chance(h.state, h.ruleset, s));
        if of != h.side && chance > 0 && v > 0 && !dispel {
            v = (50 - chance).wrapping_mul(v).wrapping_mul(2) / 100;
        }
        if v > best.value {
            best.value = v;
            target = Some(s);
        }
    }
    let Some(t) = target else {
        return Valued::ZERO;
    };
    let haste = h
        .ruleset
        .spell(c.spell)
        .and_then(Kind::of)
        .is_some_and(|k| k == Kind::Haste);
    let now = haste
        || if of == h.side {
            now_on_own(h, t, c)
        } else {
            now_on_enemy(h, t)
        };
    Valued {
        value: best.value,
        target: CastTarget::Stack(t),
        second: CastTarget::None,
        now,
    }
}

/// Whether a cast on stack `t` of the side is due now (0x43A80A): `t` is the active stack, no
/// other stack of the side can still act this round, or the spell protects (0x4000: Shield, Air
/// Shield, Fire Shield, the Protections, Anti-Magic, Magic Mirror, Stone Skin, Counterstrike).
fn now_on_own(h: &Hero<'_, '_>, t: StackId, c: Candidate) -> bool {
    let protects = h
        .ruleset
        .spell(c.spell)
        .and_then(Kind::of)
        .is_some_and(|k| {
            matches!(
                k,
                Kind::Shield
                    | Kind::AirShield
                    | Kind::FireShield
                    | Kind::Protection
                    | Kind::AntiMagic
                    | Kind::MagicMirror
                    | Kind::StoneSkin
                    | Kind::Counterstrike
            )
        });
    t == h.active || h.last_to_act() || protects
}

/// Whether a cast on enemy `e` is due now (0x436C60): in the desperate mode or when no other stack
/// of the side can still act this round; when the active stack is the side's, attacks `e` (its
/// threat record's enemy) within a turn, cannot shoot now and meets a retaliation; else only when
/// `e` threatens the side this turn (the threat tables) and no other stack of the side, alive,
/// not blind, turned to stone or paralysed and no turret, acts before `e` by the order's ranks.
pub(crate) fn now_on_enemy(h: &Hero<'_, '_>, e: StackId) -> bool {
    let view = h.sc.view();
    if view.desperate() || h.last_to_act() {
        return true;
    }
    let (state, ruleset) = (h.state, h.ruleset);
    let active = &state.stacks()[h.active.index()];
    let strikes = active.side == h.side
        && view.within_a_turn(h.active) == Some(e)
        && !computer::shoots(state, ruleset, h.active)
        && !active.has(StackTraits::NO_ENEMY_RETALIATION);
    if strikes {
        return true;
    }
    if view.tables().threatens & (1u64 << e.index()) == 0 {
        return false;
    }
    let ranks = h
        .ranks
        .get_or_init(|| order::ranks(state, ruleset, active.side));
    !(0u8..).zip(state.stacks()).any(|(i, s)| {
        let s_id = StackId(i);
        s.side == h.side
            && s_id != h.active
            && s.is_alive()
            && !s.spells.status.contains(Status::NO_TURNS)
            && !s.is_turret()
            && ranks[s_id.index()] > ranks[e.index()]
    })
}

/// Chain Lightning's value (0x437310; BAT-AI-018): over the other side's living stacks in record
/// order that the cast can affect and the engine takes as the first, the chain's value from each
/// ([`casts::chain`]: the friendly part counts against it); a strictly larger one takes the stack;
/// due now.
fn chain(h: &Hero<'_, '_>, spell: Spell) -> Valued {
    let targets = h.targets(spell.id);
    let mut best = Valued {
        now: true,
        ..Valued::ZERO
    };
    for (i, st) in (0u8..).zip(h.state.stacks()) {
        let s = StackId(i);
        if st.side == h.side
            || !st.is_alive()
            || !computer::spell_affects(h.state, h.ruleset, h.side, spell.id, s)
            || !targets.takes_stack(s)
        {
            continue;
        }
        let v = casts::chain(&h.scanner, spell, s);
        if v > best.value {
            best.value = v;
            best.target = CastTarget::Stack(s);
        }
    }
    best
}

/// An area spell's value (0x43BB20's case for Frost Ring, Fireball, Inferno and Meteor Shower;
/// BAT-AI-018): at each hex but the side columns that the engine takes, the area's value
/// ([`casts::area`]: each stack's value with the hero's bonuses, the friendly part against it);
/// a strictly larger one takes the hex (the first of equals); due now.
fn area(h: &Hero<'_, '_>, spell: Spell) -> Valued {
    let targets = h.targets(spell.id);
    let field = h_battle_field(h);
    let mut cache = [None; STACK_CAPACITY];
    let mut best = Valued {
        now: true,
        ..Valued::ZERO
    };
    for hex in (0u8..).take(field).map(h3_battle::Hex) {
        let column = hex.0 % 17;
        if column == 0 || column == 16 || !targets.takes_hex(hex) {
            continue;
        }
        let v = casts::area(&h.scanner, spell, hex, &mut cache);
        if v > best.value {
            best.value = v;
            best.target = CastTarget::Hex(hex);
        }
    }
    best
}

/// The battlefield's hex count.
fn h_battle_field(h: &Hero<'_, '_>) -> usize {
    h3_battle::Field::new(h.ruleset.battle().field).hex_count()
}

/// A mass damage spell's value (Death Ripple, Destroy Undead, Armageddon: 0x43BB20 with 0x436FB0;
/// BAT-AI-018): `E` the sum of its values with the hero's bonuses on the other side's stacks
/// ([`computer::spell_damage_value`], 0x436E30), `O` on the side's own, each from the last stack;
/// worth `E − O` when it pays ([`casts::pays`]); no target (the engine's: the spell is castable);
/// due now.
fn mass_damage(h: &Hero<'_, '_>, spell: Spell) -> Valued {
    let (state, ruleset) = (h.state, h.ruleset);
    let ctx = h.sc.view().context();
    let mut sums = (0i32, 0i32);
    for (i, st) in state.stacks().iter().enumerate().rev() {
        if !st.is_alive() || st.is_turret() {
            continue;
        }
        let id = StackId(u8::try_from(i).unwrap_or(u8::MAX));
        let v =
            computer::spell_damage_value(state, ruleset, ctx, (spell.id, spell.damage), true, id);
        casts::add(&mut sums, st.side == h.side, v);
    }
    Valued::untargeted(casts::pays(ctx, sums))
}

/// A summon's value (0x43BB20's case for the four elementals; BAT-AI-018): 0 when the predictions
/// beat the enemy (the rules' refusal, 0x5A93A0, keeps the spell from being castable); else `n` =
/// the spell's value at the mastery × the power creatures, worth `1000 n` in the desperate mode,
/// else `n` × the summoned creature's AI value (32 bits); no target; due now.
pub(crate) fn summon(h: &Hero<'_, '_>, s: &BattleSpell, c: Candidate) -> Valued {
    if h.sc.beaten() {
        return Valued::ZERO;
    }
    let at = usize::from(c.mastery.min(3));
    let n = i32::from(s.effect[at]).wrapping_mul(i32::try_from(c.power).unwrap_or(i32::MAX));
    let worth = if h.sc.view().desperate() {
        1000
    } else {
        s.summons
            .and_then(|cr| h.ruleset.creature_info(cr))
            .map_or(0, |cr| i32::try_from(cr.ai_value).unwrap_or(i32::MAX))
    };
    Valued::untargeted(n.wrapping_mul(worth))
}

/// Dispel's value (0x43BB20's case and, without a return, the stacks' case after it, Dispel being
/// neutral; BAT-AI-017): by its evaluator on the side's stacks (one target or the mass version,
/// [`evaluated`]). Aimed at one stack, the other side's best target too, which replaces the side's
/// only when strictly larger (at Advanced, and again in the stacks' case; below Advanced the engine
/// takes none of them). At Expert the case sums the side's and the other side's, which the stacks'
/// case then overwrites with the side's sum and then the other side's: with BAT-QUIRK-25 (survey
/// slip 27) the other side's sum alone, without the two sums.
// rule: BAT-AI-017
fn dispel(h: &Hero<'_, '_>, c: Candidate) -> Valued {
    let other = h.side.opponent();
    let own = evaluated(h, h.side, c);
    if c.mastery >= 3 {
        let theirs = on_stacks(h, other, c);
        let slip = h.ruleset.quirks().is_enabled(ORIGINAL_AI_SLIPS);
        return Valued {
            value: if slip {
                theirs
            } else {
                own.value.wrapping_add(theirs)
            },
            ..own
        };
    }
    let theirs = evaluated(h, other, c);
    if theirs.value > own.value {
        theirs
    } else {
        own
    }
}
