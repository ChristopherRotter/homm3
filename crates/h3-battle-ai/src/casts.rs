//! The computer's creature casts (0x421280, `docs/formats/installation.md` §7.42, §7.43;
//! BAT-AI-012, BAT-AI-013): the entry a walker asks before its action and again after a choice of
//! nothing, with the value its choice found (the best candidate's value or plan value per turn,
//! 0 without one). Nothing while the creatures may not cast; then by the caster's kind (by
//! creature in the original's table 0x42136C, by ability here): the raise of the Archangel and
//! the Pit Lord ([`raise`]), which must beat that value, the Faerie Dragon's drawn spell
//! ([`dragon`]), cast at the first hex where it is worth anything whatever that value, and the
//! Ogre Mage's Bloodlust and the Master Genie's drawn spell ([`scan`], [`bloodlust`], [`genie`]),
//! which must beat that value. Every other creature casts nothing: the computer's elementals never protect (§7.42, BAT-Q57). The engine
//! gives the Ogre Mage and the elementals one ability, a creature's cast of a fixed spell: the
//! policy tells the Ogre Mage by its spell, the only one that raises the melee attack (the user,
//! E23 T4b1).
//!
//! The policy casts only at the targets the engine accepts (ADR-0027 point 3; the user, E23
//! T4a): the original's dragon may also aim an area spell at a free hex, its own or an immune
//! stack's, which the rules' cursor refuses (BAT-MAG-049; BAT-STUB-8, BAT-Q32).
//!

use h3_battle::computer::{self, Context};
use h3_battle::{
    BattleSide, BattleState, CastSpell, CastTarget, CastTargets, Command, Hex, STACK_CAPACITY,
    StackId, StackTraits, Status, creature_cast_spell, creature_cast_targets,
};
use h3_content::{CreatureTypeId, ORIGINAL_AI_SLIPS, SpellId};
use h3_content_battle::{
    Ability, BattleRuleset, BattleSpell, MAX_MASTERY, SpellEffect, SpellTarget, TimedChange,
};
use h3_kernel::Rng;
use h3_kernel::arith::{DOUBLE_BITS, SINGLE_BITS, X87};

use crate::predict;
use crate::spells::{Candidate, SpellContext, has_spell};
use crate::walker::Walker;

/// The scan's draw (0x420D98: `random(1, 100)`) and the share that casts nothing (30 or less).
const SKIP: (u32, u32, u32) = (1, 100, 30);

/// The balance class's thresholds (0x63B798, doubles): the larger able strength over the smaller,
/// in thousandths.
const THRESHOLDS: [u32; 6] = [2600, 1900, 1500, 1310, 1200, 1130];

/// The class below every threshold (0x435EC0's +0x1C: an even battle).
const EVEN: u8 = 7;

/// The creature cast entry (0x421280; BAT-AI-012): nothing while the creatures may not cast
/// (Cursed Ground, an anti-magic garrison, the Orb of Inhibition; [`computer::creature_casts_blocked`],
/// whatever a creature's `ignores_magic_block`) or without a cast left; a raiser's raise
/// ([`raise`]); a random caster's drawn spell ([`dragon`]); the Ogre Mage's scan ([`scan`], its
/// draw from `rng`); any other stack nothing. `value` is the value so far, which a cast's
/// candidate replaces.
// rule: BAT-AI-012
pub(crate) fn entry(w: &Walker<'_>, value: &mut i32, rng: &mut Rng) -> Option<Command> {
    if computer::creature_casts_blocked(w.state) || w.state.casts_left(w.id) == 0 {
        return None;
    }
    w.ruleset
        .abilities(w.me.creature)
        .iter()
        .find_map(|a| match *a {
            Ability::StackRaiser { .. } => Some(raise(w, None, value)),
            Ability::CorpseRaiser { creature, .. } => Some(raise(w, Some(creature), value)),
            Ability::RandomSpellCaster {
                mastery,
                power_per_creature,
            } => Some(dragon(w, (mastery, power_per_creature), value, rng)),
            Ability::SpellCaster {
                spell,
                mastery,
                power,
            } => raises_melee_attack(w.ruleset, spell)
                .then(|| scan(w, Caster::Ogre(spell, mastery, power), value, rng)),
            Ability::HelpfulSpellCaster { mastery, power } => {
                Some(scan(w, Caster::Genie(mastery, power), value, rng))
            }
            _ => None,
        })
        .flatten()
}

/// Whether `spell` raises the melee attack (Bloodlust's timed change): the Ogre Mage's spell
/// among the creatures' fixed casts, which the computer's table sends to the scan (0x42136C's
/// case 1, creature 91); the elementals' Protections do not (BAT-Q57).
fn raises_melee_attack(ruleset: &BattleRuleset, spell: SpellId) -> bool {
    ruleset.spell(spell).is_some_and(|s| {
        s.magic
            .effects
            .contains(&SpellEffect::Timed(TimedChange::MeleeAttack))
    })
}

/// A caster the scan serves (0x421280's case 1): the Ogre Mage with its spell, mastery and power,
/// or the Master Genie with its mastery and power.
#[derive(Debug, Clone, Copy)]
enum Caster {
    Ogre(SpellId, u8, u8),
    Genie(u8, u8),
}

/// The Ogre Mage's and the Master Genie's scan (0x420D20; BAT-AI-014, BAT-AI-016), for a caster
/// of `caster`: with a value so far that is not 0 (a negative one too) a draw of 1 to 100
/// ([`SKIP`]) casts nothing on 30 or less. Then over the walker's side's stacks from the last to
/// the first, but the caster and the turrets, each that the engine accepts as the cast's target
/// (where the rules cast the Ogre Mage's mass version, which takes no target, each the spell can
/// affect, [`computer::spell_affects`]: the original aims it at the stack's hex all the same),
/// and once the value so far is above 0 only one at least as strong as the caster (0x442E60 in
/// the walker's context): its value, the Ogre Mage's [`bloodlust`], the Genie's [`genie`]; one
/// above 0 and above `value` replaces it and takes the stack (the mass version: no target), so the
/// cast must beat the walker's value, and the later stack stays at a tie.
// rule: BAT-AI-014, BAT-AI-016
fn scan(w: &Walker<'_>, caster: Caster, value: &mut i32, rng: &mut Rng) -> Option<Command> {
    let (state, ruleset, side) = (w.state, w.ruleset, w.side);
    let strength = computer::strength(state, ruleset, &w.ctx, w.id);
    let sc = SpellContext::new(state, ruleset, (&w.ctx, side), rng);
    if *value != 0 && rng.range_inclusive(SKIP.0, SKIP.1) <= SKIP.2 {
        return None;
    }
    // The rules cast the Ogre Mage's mass version without a target (Magic Plains raises a
    // creature's cast to Expert): the stacks the scan values are then those the spell can affect.
    // The Genie's cast always takes a stack.
    let mass = match caster {
        Caster::Ogre(spell, mastery, _) => {
            (!computer::takes_stack(ruleset, spell, computer::creature_mastery(state, mastery)))
                .then_some(spell)
        }
        Caster::Genie(..) => None,
    };
    let mut targets = CastTargets::new();
    if mass.is_none() {
        creature_cast_targets(state, ruleset, side, &mut targets);
    }
    let mut best = None;
    for (i, s) in state.stacks().iter().enumerate().rev() {
        let id = StackId(u8::try_from(i).unwrap_or(u8::MAX));
        let target = CastTarget::Stack(id);
        let accepted = match mass {
            Some(spell) => computer::spell_affects(state, ruleset, side, spell, id),
            None => targets.as_slice().contains(&target),
        };
        if s.side != side
            || id == w.id
            || s.is_turret()
            || !accepted
            || (*value > 0 && strength > computer::strength(state, ruleset, &w.ctx, id))
        {
            continue;
        }
        let v = match caster {
            Caster::Ogre(spell, mastery, power) => {
                bloodlust(w, &sc, (spell, mastery, u32::from(power)), id)
            }
            Caster::Genie(mastery, power) => genie(w, &sc, (mastery, u32::from(power)), id),
        };
        if v > 0 && v > *value {
            *value = v;
            best = Some(if mass.is_some() {
                CastTarget::None
            } else {
                target
            });
        }
    }
    best.map(|target| Command::CreatureCast { target })
}

/// The mastery the computer's estimate of a creature's cast of `spell` at the ability's `mastery`
/// takes (0x43C330's and 0x43C4A0's switch): the creature's (Expert on Magic Plains,
/// [`computer::creature_mastery`]); with BAT-QUIRK-25 Expert also on the spell's school's field
/// (survey slip 8), where the rules keep the creature's.
// rule: BAT-AI-014, BAT-AI-016, BAT-TER-005
fn estimate_mastery(w: &Walker<'_>, spell: SpellId, mastery: u8) -> u8 {
    let slips = w.ruleset.quirks().is_enabled(ORIGINAL_AI_SLIPS);
    if slips && computer::field_raises(w.state, w.ruleset, spell) {
        MAX_MASTERY
    } else {
        computer::creature_mastery(w.state, mastery)
    }
}

/// The computer's value of an Ogre Mage's Bloodlust on stack `id` (0x43C330; BAT-AI-014), the
/// caster's `spell` at its `mastery` lasting `rounds`: 0 while Bloodlust lasts on the stack; at
/// the estimate's mastery ([`estimate_mastery`]) aimed at a stack, Bloodlust's evaluator on the
/// stack ([`SpellContext::bloodlust`]); its mass version, the value on the side's whole army
/// ([`SpellContext::on_side`]).
// rule: BAT-AI-014
fn bloodlust(
    w: &Walker<'_>,
    sc: &SpellContext<'_>,
    (spell, mastery, rounds): (SpellId, u8, u32),
    id: StackId,
) -> i32 {
    if has_spell(w.state, id, spell) {
        return 0;
    }
    let mastery = estimate_mastery(w, spell, mastery);
    let c = Candidate::cast(spell, mastery, rounds);
    if computer::takes_stack(w.ruleset, spell, mastery) {
        sc.bloodlust(id, c)
    } else {
        sc.on_side(c, |sc, s, c| sc.bloodlust(s, c))
    }
}

/// The computer's value of a Master Genie's cast on stack `id` (0x43C4A0; BAT-AI-016), at the
/// ability's `mastery` with `power` (its power and rounds): the mean, in 32-bit integers, over the
/// spells its cast could draw for the stack ([`computer::helpful_spells`]: the rules' usefulness
/// test, as 0x447EB0 asks 0x447A80) of each one's value at its estimate's mastery
/// ([`estimate_mastery`]): its evaluator on the stack when aimed at a stack at that mastery
/// ([`SpellContext::value`]), else on the side's whole army ([`SpellContext::on_side`]). The
/// rules draw among the same spells (BAT-MAG-050): the value of the spell the cast draws, each
/// counted once whatever its weight (the original has none). 0 without such a spell.
// rule: BAT-AI-016
pub(crate) fn genie(
    w: &Walker<'_>,
    sc: &SpellContext<'_>,
    (mastery, power): (u8, u32),
    id: StackId,
) -> i32 {
    let (mut sum, mut n) = (0i32, 0i32);
    for spell in computer::helpful_spells(w.state, w.ruleset, w.side, (w.id, id)) {
        n += 1;
        let m = estimate_mastery(w, spell, mastery);
        let c = Candidate::cast(spell, m, power);
        let v = if computer::takes_stack(w.ruleset, spell, m) {
            sc.value(id, c)
        } else {
            sc.on_side(c, |sc, s, c| sc.value(s, c))
        };
        sum = sum.wrapping_add(v);
    }
    if n == 0 { 0 } else { sum / n }
}

/// The two sides' able strengths and their balance class (0x435EC0's +0x14, +0x18 and +0x1C).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Balance {
    /// The side's able strength (0x41EAC0 with flag 0).
    pub(crate) own: i32,
    /// The other side's.
    pub(crate) other: i32,
    /// 1 when the smaller is at most a fifth of the larger (in 32-bit arithmetic) or the larger is
    /// 0; else 1 + the first threshold the larger over the smaller reaches (a double), 7 below
    /// them all.
    pub(crate) class: u8,
}

/// The balance of `side` in the context `ctx` (0x435EC0; E23 T4a): each side's able strength
/// sums the strength (0x442E60) of its living stacks but the turrets, those that lose their turns,
/// the First Aid Tent and the Ammo Cart, as the original's 32-bit integers.
pub(crate) fn balance(
    state: &BattleState,
    ruleset: &BattleRuleset,
    ctx: &Context,
    side: BattleSide,
) -> Balance {
    let able = |of: BattleSide| {
        (0u8..)
            .zip(state.stacks())
            .filter(|(_, s)| {
                s.side == of
                    && s.is_alive()
                    && !s.is_turret()
                    && !s.spells.status.contains(Status::NO_TURNS)
                    && !s.has(StackTraits::FIRST_AID)
                    && !s.has(StackTraits::AMMO_SUPPLY)
            })
            .fold(0i32, |sum, (i, _)| {
                sum.wrapping_add(computer::strength(state, ruleset, ctx, StackId(i)))
            })
    };
    let (own, other) = (able(side), able(side.opponent()));
    let (larger, smaller) = if own >= other {
        (own, other)
    } else {
        (other, own)
    };
    let class = if smaller.wrapping_mul(5) <= larger || larger == 0 {
        1
    } else {
        // Both are above 0 here: the smaller's fifth is below the larger, which is not 0.
        let int = |n: i32| X87::int(u64::try_from(n).unwrap_or(0));
        let ratio = int(larger)
            .over(int(smaller))
            .unwrap_or(X87::MAX)
            .round(DOUBLE_BITS);
        THRESHOLDS
            .iter()
            .position(|&t| ratio >= X87::ratio(t, 1000, DOUBLE_BITS).unwrap_or(X87::ZERO))
            .map_or(EVEN, |i| u8::try_from(i + 1).unwrap_or(EVEN))
    };
    Balance { own, other, class }
}

/// The computer's raise (0x421000; BAT-AI-012), for a corpse raiser of `created` (the Pit Lord's
/// Demons) or a stack raiser (the Archangel, `created` none): over the walker's side's stacks
/// from the last to the first, each that the engine accepts as the raise's target (another stack
/// of the side: the Pit Lord's corpse on top, the Archangel's stack Resurrection's test takes,
/// living or dead; BAT-MAG-051, BAT-MAG-052), its value `trunc(n × w)`: `n` the creatures raised
/// ([`computer::raise_count`]; at least 1 for such a target), `w` one creature's worth
/// (0x442A50 in the context's least bonuses: a raised Demon's without the shot flag,
/// [`computer::created_worth`]; the stack's with its own); doubled when the side's able strength
/// exceeds the other's and the balance class is 1 ([`balance`]). A value above `value` replaces
/// it and takes the stack, so the raise must beat the walker's value, and the later stack stays
/// at a tie.
// rule: BAT-AI-012
fn raise(w: &Walker<'_>, created: Option<CreatureTypeId>, value: &mut i32) -> Option<Command> {
    let (state, ruleset, side) = (w.state, w.ruleset, w.side);
    let mut targets = CastTargets::new();
    // stub: BAT-STUB-8
    // Only the targets the engine accepts (ADR-0027 point 3; the user, E23 T4a).
    creature_cast_targets(state, ruleset, side, &mut targets);
    if targets.is_empty() {
        return None;
    }
    let created = created.map(|c| computer::created_worth(state, ruleset, &w.ctx, side, c));
    let b = balance(state, ruleset, &w.ctx, side);
    let doubles = b.own > b.other && b.class <= 1;
    let mut best = None;
    for i in (0..state.stacks().len()).rev() {
        let id = StackId(u8::try_from(i).unwrap_or(u8::MAX));
        let target = CastTarget::Stack(id);
        if !targets.as_slice().contains(&target) {
            continue;
        }
        let n = computer::raise_count(state, ruleset, w.id, id);
        let worth = created.unwrap_or_else(|| {
            let shot = computer::shoots(state, ruleset, id);
            computer::worth(state, ruleset, &w.ctx, id, shot)
        });
        let v = computer::ftol(worth.times(X87::int(u64::from(n))));
        let v = if doubles { v.wrapping_add(v) } else { v };
        if v > *value {
            *value = v;
            best = Some(target);
        }
    }
    best.map(|target| Command::CreatureCast { target })
}

/// How the computer values a random caster's drawn spell (0x43C620's cases): a bolt (Magic Arrow,
/// Ice Bolt, Lightning Bolt, Implosion: a damage spell aimed at a stack), Chain Lightning (a
/// damage spell with a chain) or an area spell (Frost Ring, Fireball, Inferno, Meteor Shower: a
/// damage spell aimed at a hex with an area); every other spell is worth nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Bolt,
    Chain,
    Area,
}

impl Kind {
    /// The kind of `spell`, by its data (the original's spells 15 to 23 by number; Titan's
    /// Lightning Bolt, 57, which no random caster draws, would be a bolt here and nothing there).
    fn of(spell: &BattleSpell) -> Option<Self> {
        let m = &spell.magic;
        let damages = m.effects.contains(&SpellEffect::Damage);
        let area = m.target == SpellTarget::Hex && m.area.is_some();
        let bolt = m.target == SpellTarget::Stack && m.chain.is_none();
        [
            (m.chain.is_some(), Self::Chain),
            (area, Self::Area),
            (bolt, Self::Bolt),
        ]
        .into_iter()
        .find_map(|(is, kind)| (is && damages).then_some(kind))
    }
}

/// The computer's Faerie Dragon (0x420F00 with 0x43C620; BAT-AI-013), a random caster at the
/// ability's `mastery` with `per_creature` power per creature: its drawn spell's damage, factor ×
/// power + its value at the estimate's mastery (the creature's, Expert on Magic Plains; with
/// BAT-QUIRK-25 also on its school's field, survey slip 7, where the rules keep a creature's), at
/// each hex but the side columns that the engine accepts as its target: a bolt's on the stack
/// there ([`computer::spell_damage_value`]; the side's hero's bonuses with the slip, none without,
/// as the rules), a chain's from the stack there ([`computer::chain`]) and an area's around the
/// hex ([`computer::area_stacks`]), each stack's value with the bonuses into the side's sum or
/// the other's ([`pays`]). The first hex worth above 0 is taken whatever `value`, a later one only
/// when worth more (the lowest hex at a tie), and `value` becomes its worth: a dragon with a cast
/// left casts whenever its spell is worth anything, before any attack. The original builds the
/// spell context first (0x4369C0) and reads none of it but its battle context: the policy runs
/// only its predictions, for their draws from `rng` ([`predict::plan`]; BAT-AI-015).
// rule: BAT-AI-013
fn dragon(
    w: &Walker<'_>,
    (mastery, per_creature): (u8, u8),
    value: &mut i32,
    rng: &mut Rng,
) -> Option<Command> {
    let (state, ruleset, side) = (w.state, w.ruleset, w.side);
    // rule: BAT-AI-015
    predict::plan(state, ruleset, side, rng);
    let mut targets = CastTargets::new();
    // stub: BAT-STUB-8
    // Only the targets the engine accepts (ADR-0027 point 3; the user, E23 T4a).
    creature_cast_targets(state, ruleset, side, &mut targets);
    let drawn = creature_cast_spell(state, ruleset, side)
        .and_then(CastSpell::named)
        .and_then(|id| ruleset.spell(id).map(|s| (id, s)))
        .and_then(|(id, s)| Kind::of(s).map(|kind| (id, s, kind)));
    drawn.and_then(|(id, spell, kind)| {
        let slips = ruleset.quirks().is_enabled(ORIGINAL_AI_SLIPS);
        // rule: BAT-TER-005
        let m = if slips && computer::field_raises(state, ruleset, id) {
            MAX_MASTERY
        } else {
            computer::creature_mastery(state, mastery)
        };
        let power = u64::from(per_creature) * u64::from(w.me.count);
        let at = usize::from(m.min(MAX_MASTERY));
        let damage = u64::from(spell.power) * power + u64::from(spell.effect[at]);
        let spell = Spell {
            id,
            mastery: m,
            damage,
        };
        let mut cache = [None; STACK_CAPACITY];
        let mut best = None;
        for h in (0u8..).take(w.field.hex_count()).map(Hex) {
            let target = match kind {
                Kind::Area => Some(CastTarget::Hex(h)),
                Kind::Bolt | Kind::Chain => w.on[h.index()].map(CastTarget::Stack),
            };
            let Some(target) =
                target.filter(|t| w.field.is_standable(h) && targets.as_slice().contains(t))
            else {
                continue;
            };
            let v = match (kind, target) {
                (Kind::Bolt, CastTarget::Stack(s)) => {
                    computer::spell_damage_value(state, ruleset, &w.ctx, (id, damage), slips, s)
                }
                (Kind::Chain, CastTarget::Stack(s)) => chain(w, spell, s),
                _ => area(w, spell, h, &mut cache),
            };
            if v > 0 && (best.is_none() || v > *value) {
                *value = v;
                best = Some(target);
            }
        }
        best.map(|target| Command::CreatureCast { target })
    })
}

/// A random caster's drawn spell as the computer values it: the spell, the estimate's mastery and
/// its damage.
#[derive(Debug, Clone, Copy)]
struct Spell {
    id: SpellId,
    mastery: u8,
    damage: u64,
}

/// Chain Lightning's value from stack `first` (0x437190): each stack of the estimated chain
/// ([`computer::chain`]: the damage halved after each strike) by its spell value with the side's
/// hero's bonuses into the side's sum (a stack of the side, whoever controls it) or the other's;
/// then [`pays`].
fn chain(w: &Walker<'_>, spell: Spell, first: StackId) -> i32 {
    let (state, ruleset) = (w.state, w.ruleset);
    let mut sums = (0i32, 0i32);
    for (s, damage) in computer::chain(
        state,
        ruleset,
        w.side,
        (spell.id, spell.mastery),
        (first, spell.damage),
    ) {
        let v = computer::spell_damage_value(state, ruleset, &w.ctx, (spell.id, damage), true, s);
        add(&mut sums, state.stacks()[s.index()].side == w.side, v);
    }
    pays(&w.ctx, sums)
}

/// An area spell's value at `hex` (0x437040): each stack of its area ([`computer::area_stacks`])
/// by its spell value with the side's hero's bonuses (each stack's computed once a decision, in
/// `cache`) into the side's sum or the other's; then [`pays`].
fn area(w: &Walker<'_>, spell: Spell, hex: Hex, cache: &mut [Option<i32>; STACK_CAPACITY]) -> i32 {
    let (state, ruleset) = (w.state, w.ruleset);
    let mut sums = (0i32, 0i32);
    for s in computer::area_stacks(state, ruleset, w.side, (spell.id, spell.mastery), hex) {
        let v = *cache[s.index()].get_or_insert_with(|| {
            computer::spell_damage_value(state, ruleset, &w.ctx, (spell.id, spell.damage), true, s)
        });
        add(&mut sums, state.stacks()[s.index()].side == w.side, v);
    }
    pays(&w.ctx, sums)
}

/// Adds `v` to the side's sum when `own`, else to the other's, as the original's 32-bit integers.
fn add(sums: &mut (i32, i32), own: bool, v: i32) {
    if own {
        sums.0 = sums.0.wrapping_add(v);
    } else {
        sums.1 = sums.1.wrapping_add(v);
    }
}

/// What a spell's values `own` on the side's stacks and `other` on the other side's are worth to
/// the side (0x437190's and 0x437040's end): `other − own` when `other` is above `own` (taken as at
/// least 0), the side's share of its strength is below the other's (`own / S_own < other /
/// S_other`, single-precision values divided at 53 bits; the strengths of every stack, the
/// context's) and `own` is below the side's strength; else 0. So friendly damage counts against
/// the cast, and a cast that costs the side a larger part of its strength than the enemy's is never
/// taken.
fn pays(ctx: &Context, (own, other): (i32, i32)) -> i32 {
    let (s_own, s_other) = ctx.strengths();
    let own = own.max(0);
    let single = |n: i32| X87::int(u64::try_from(n).unwrap_or(0)).round(SINGLE_BITS);
    let share = |n: i32, d: i32| single(n).over(single(d)).unwrap_or(X87::MAX);
    if other > own && share(own, s_own) < share(other, s_other) && own < s_own {
        other.wrapping_sub(own)
    } else {
        0
    }
}
