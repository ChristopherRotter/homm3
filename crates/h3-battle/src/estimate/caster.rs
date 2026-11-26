//! The computer's values of a hero's spells (E31 T2e, AI-Q36; survey
//! `docs/rules/surveys/E31-computer-player.md` §2.4, `installation.md` §7.39): the adventure
//! computer's caster record of a hero (`ai_caster_record` 0x526D40, CPU-SPL-001), a spell's value
//! (`ai_spell_value` 0x5273D0 with its kinds `ai_damage_spell_value` 0x5270E0,
//! `ai_mass_damage_spell_value` 0x5271C0 and `ai_effect_spell_value` 0x527290, CPU-SPL-002 to
//! -005) and the best value among a hero's spells (`ai_best_spell_value` 0x5275B0, CPU-SPL-006).
//! The adventure's computer player calls them (`h3_game::computer`: a spell's gain, the hero's
//! values per unit, the schools' values); the battle AI's odds take the best damage spell from
//! them too (BAT-AI-008, `crate::aim`): one implementation (ADR-0030 point 6). They live here
//! because they read the battle's spell rules (the mastery on a field, the cost, the caster's
//! bonuses, a creature's part of the effect test, whether a spell takes a stack), as the
//! simulated combat does.
//!
//! Every value is the original's arithmetic: 32-bit integers, wrapping where its `imul` and
//! `add` wrap, and its x87 doubles (`SignedX87`, each operation rounded to 53 bits, `_ftol`'s
//! truncation and low 32 bits). No draws.

use alloc::vec::Vec;

use h3_content::{CreatureTypeId, SINGLE_PRECISION_RATES, SpellId};
use h3_content_battle::{
    BattleRuleset, BattlefieldId, MAX_MASTERY, SpellAiKind, SpellAiKinds, SpellPolarity,
};
use h3_contracts::ArmyStack;
use h3_kernel::arith::{DOUBLE_BITS, SignedX87, X87};
use h3_kernel::order::sort_by_unique_key;

use super::{EstimateHero, MAX_POWER, SLOTS};
use crate::computer::takes_stack;
use crate::magic::{self, Guard, HeroMagic, SpellRef, SpellSet};
use crate::mana;

#[cfg(test)]
mod tests;

/// The value curve of a damage spell (`ai_damage_spell_curve` 0x640398, `installation.md`
/// §7.39), 14 rows in thousandths, each the nearest double to its decimal: the casts' factor φ,
/// the threshold t (row 13's unread), the slope α and the intercept β.
const SPELL_CURVE: [[u32; 4]; 14] = [
    [0, 4800, 0, 4000],
    [830, 1557, 500, 1600],
    [1530, 752, 987, 842],
    [2110, 433, 1452, 492],
    [2590, 275, 1888, 303],
    [2990, 186, 2291, 192],
    [3330, 131, 2657, 124],
    [3600, 95, 2986, 81],
    [3840, 71, 3277, 53],
    [4030, 54, 3353, 35],
    [4190, 41, 3755, 23],
    [4330, 32, 3947, 16],
    [4440, 25, 4112, 10],
    [4530, 0, 4530, 0],
];

/// The casts the curve's factor stops growing at (0x5270E0, 0x52756A: `min(n, 13)`).
const CURVE_CASTS: i32 = 13;

/// The casts a free spell allows (0x5273D0: 1000 at cost 0).
const FREE_CASTS: i32 = 1000;

/// The rounds an effect spell is valued for at most, and a single-target one's rounds per stack
/// (0x527290's 7).
const EFFECT_ROUNDS: i32 = 7;

/// The effect value's divisor (0x5273A7: 700).
const EFFECT_DIVISOR: i32 = 700;

/// A raise's ratio before the casts' factor, at most (0x527545: 0.9), and after it (0x52757E:
/// 3.9), in tenths.
const RAISE_CAPS: (u32, u32) = (9, 39);

/// The map spells' factor on the casts' root and its offset (0x5274EB: 0.001, 0x5274F9:
/// 0.009), in thousandths.
const SPECIAL_FACTORS: (u32, u32) = (1, 9);

/// A curve entry as the double the table holds.
fn curve(row: usize, column: usize) -> SignedX87 {
    let n = SPELL_CURVE[row.min(SPELL_CURVE.len() - 1)][column];
    SignedX87::from(X87::ratio(n, 1000, DOUBLE_BITS).unwrap_or(X87::ZERO))
}

/// A constant of the code: `n / d` as its nearest double.
fn constant(n: u32, d: u32) -> SignedX87 {
    SignedX87::from(X87::ratio(n, d, DOUBLE_BITS).unwrap_or(X87::ZERO))
}

/// An `int` as `fild` loads it.
fn int(n: i32) -> SignedX87 {
    SignedX87::int(n.into())
}

/// A primary skill byte as the record holds it (0x526DB5, 0x526DE4): a signed byte above 99 as
/// 99, at most 0 as 1.
fn held(byte: u8) -> i32 {
    match i8::from_ne_bytes([byte]) {
        v if v > 99 => i32::from(MAX_POWER),
        v if v > 0 => i32::from(v),
        _ => 1,
    }
}

/// One entry of a caster record's army list (+0x18, 12 bytes): a stack's creature and its value,
/// the creature's AI value times the count's low 16 bits as a signed word (0x526E2F, 32 bits).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Entry {
    /// The creature (+0).
    creature: CreatureTypeId,
    /// Its value (+4).
    value: i32,
}

/// What the spell values read of a caster: the record's numbers and its hero's magic, borrowed
/// (the computer's [`CasterRecord`], and the battle AI's odds on a battle's hero).
#[derive(Debug, Clone, Copy)]
pub(crate) struct SpellCaster<'a> {
    /// The hero's magic (its masteries, its spell bonuses, its level limit, its defences).
    pub(crate) magic: &'a HeroMagic,
    /// Its available spells (+0x430).
    pub(crate) spells: &'a SpellSet,
    /// The special battlefield of the hero's map tile its masteries are raised on
    /// (`hero_tile_spell_field` 0x4E4FA0).
    pub(crate) ground: Option<BattlefieldId>,
    /// The strength the damage values count against (+4).
    pub(crate) strength: i32,
    /// The power (+8): the hero's Spell Power held to 1..99, 0 without power.
    pub(crate) power: i32,
    /// The rounds (+0xC): the power plus the duration bonus.
    pub(crate) rounds: i32,
    /// The mana (+0x10): the hero's maximum spell points.
    pub(crate) mana: i32,
    /// The hero's army, by slot (+0x91: the mass damage reads it).
    pub(crate) army: &'a [ArmyStack],
    /// The army's list, by value, highest first (+0x18: the effect value reads it).
    pub(crate) entries: &'a [Entry],
}

/// The computer's caster record of a hero (`ai_caster_record` 0x526D40; CPU-SPL-001) and the
/// values of its spells ([`CasterRecord::value`], [`CasterRecord::best`]). The record's numbers
/// are fields: the computer's callers change them to value a hero with more power, rounds or
/// mana (`ai_hero_effectiveness` 0x527760) and back.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CasterRecord {
    /// The army's value (+4: `army_value` 0x44AC80, 32 bits), the strength its damage spells are
    /// valued against; 0 without power.
    pub strength: i32,
    /// The power (+8): its Spell Power held to 1..99; 0 without a spell book or with a
    /// `NoCasting` effect (the Orb of Inhibition), and then no spell has a value.
    pub power: i32,
    /// The rounds (+0xC): the power plus its duration bonus (`hero_duration_bonus` 0x4E4DB0);
    /// 0 without power.
    pub rounds: i32,
    /// The mana (+0x10): its maximum spell points, not those it has left (the adventure map's
    /// formula, HER-MAN-001: Knowledge held to 1..99 times ten, grown by Intelligence); 0
    /// without power.
    pub mana: i32,
    magic: HeroMagic,
    spells: SpellSet,
    ground: Option<BattlefieldId>,
    army: Vec<ArmyStack>,
    entries: Vec<Entry>,
}

impl CasterRecord {
    /// The record of `hero`, its army `army`, on its map tile's special battlefield `ground`
    /// (`ai_caster_record` 0x526D40; CPU-SPL-001): without a spell book or with a `NoCasting`
    /// effect every number 0 and no list; else the army's value, the power, the rounds and the
    /// mana, and per army slot with a creature, in slot order, the entry of its creature and its
    /// value (the AI value times the count's low 16 bits as a signed word, 32 bits), sorted by
    /// value, highest first (the library's insertion sort of seven entries: equal values keep
    /// their slot order).
    // rule: CPU-SPL-001
    pub fn new(
        ruleset: &BattleRuleset,
        hero: EstimateHero<'_>,
        army: &[ArmyStack],
        ground: Option<BattlefieldId>,
    ) -> Self {
        let single = ruleset.quirks().is_enabled(SINGLE_PRECISION_RATES);
        let magic = HeroMagic::of(hero.hero, single).unwrap_or_default();
        let mut spells = SpellSet::default();
        for &spell in hero.spells {
            spells.insert(spell);
        }
        let mut record = Self {
            strength: 0,
            power: 0,
            rounds: 0,
            mana: 0,
            magic,
            spells,
            ground,
            army: army.to_vec(),
            entries: Vec::new(),
        };
        if hero.hero.magic.spell_book.is_none() || record.magic.no_casting {
            return record;
        }
        let ai_value = |creature| {
            ruleset
                .creature_info(creature)
                .map_or(0, |c| i32::from_ne_bytes(c.ai_value.to_ne_bytes()))
        };
        let mut keyed = Vec::new();
        for slot in 0..SLOTS {
            let Some(stack) = army.iter().find(|s| s.slot == slot) else {
                continue;
            };
            let count = i32::from_ne_bytes(stack.count.to_ne_bytes());
            record.strength = record
                .strength
                .wrapping_add(ai_value(stack.creature).wrapping_mul(count));
            let [low, high, _, _] = stack.count.to_le_bytes();
            let word = i32::from(i16::from_le_bytes([low, high]));
            let entry = Entry {
                creature: stack.creature,
                value: ai_value(stack.creature).wrapping_mul(word),
            };
            keyed.push((entry, slot));
        }
        // The slots are unique: the key is.
        let _ = sort_by_unique_key(&mut keyed, |&(e, slot)| (core::cmp::Reverse(e.value), slot));
        record.entries = keyed.into_iter().map(|(e, _)| e).collect();
        record.power = held(hero.hero.power);
        record.rounds = record
            .power
            .wrapping_add(i32::from(record.magic.duration_bonus));
        record.mana = i32::from(hero.hero.magic.max_mana);
        record
    }

    /// The record as the values read it.
    fn caster(&self) -> SpellCaster<'_> {
        SpellCaster {
            magic: &self.magic,
            spells: &self.spells,
            ground: self.ground,
            strength: self.strength,
            power: self.power,
            rounds: self.rounds,
            mana: self.mana,
            army: &self.army,
            entries: &self.entries,
        }
    }

    /// The value of spell `spell` (`ai_spell_value` 0x5273D0; CPU-SPL-002): [`value`].
    pub fn value(&self, ruleset: &BattleRuleset, spell: SpellId) -> i32 {
        ruleset
            .spell(spell)
            .map_or(0, |s| value(ruleset, &self.caster(), (spell, s)))
    }

    /// The best value among its spells of the AI kinds `kinds` (`ai_best_spell_value`
    /// 0x5275B0; CPU-SPL-006): [`best`].
    pub fn best(&self, ruleset: &BattleRuleset, kinds: SpellAiKinds) -> i32 {
        best(ruleset, &self.caster(), kinds)
    }
}

/// The value of `spell` to caster `c` (`ai_spell_value` 0x5273D0; CPU-SPL-002): its mastery `m`
/// on the hero's map tile (`hero_spell_mastery` 0x4E5080 on `hero_tile_spell_field` 0x4E4FA0);
/// its cost at the hero's own mastery with no army (`hero_spell_cost` 0x4E5240 with none and no
/// field: neither the enemy's Pegasi nor the own Mages); 0 when it costs more than the mana; else
/// the casts `n`, the mana over the cost (1000 for a free spell), and by its AI kinds as one
/// value: `Damage` [`damage_value`] with `n` casts against the strength, `DamageOnce` the same
/// with one, `DamageAll` [`mass_value`], `Effect` [`effect_value`], `Stacks` (a raise or a
/// summon) `trunc(S × min(min(10 v_m (P + m) / S, 0.9) × φ_min(n, 13), 3.9))`, `Special` (the
/// map's spells, Earthquake) `trunc((0.001 √n + 0.009) × (v_m × S))`, with `v_m` its AI value at
/// `m`, `P` the power and `S` the strength (each product of integers in 32 bits); no kind or
/// several (Titan's Lightning Bolt): 1.
// rule: CPU-SPL-002
pub(crate) fn value(
    ruleset: &BattleRuleset,
    c: &SpellCaster<'_>,
    (id, spell): SpellRef<'_>,
) -> i32 {
    let m = magic::mastery_at(ruleset, c.ground, c.magic, id);
    let cost = i32::from(mana::cost(spell, c.magic.masteries.get(id), 0));
    if cost > c.mana {
        return 0;
    }
    let casts = if cost > 0 { c.mana / cost } else { FREE_CASTS };
    let v = i32::from(spell.ai.values[usize::from(m.min(MAX_MASTERY))]);
    match spell.ai.kinds.only() {
        Some(SpellAiKind::Damage) => damage_value(ruleset, c, (id, spell), m, (casts, c.strength)),
        Some(SpellAiKind::DamageOnce) => damage_value(ruleset, c, (id, spell), m, (1, c.strength)),
        Some(SpellAiKind::DamageAll) => mass_value(ruleset, c, (id, spell), m, casts),
        Some(SpellAiKind::Effect) => effect_value(ruleset, c, (id, spell), m, casts),
        Some(SpellAiKind::Stacks) => {
            let s = int(c.strength);
            let ten = v
                .wrapping_mul(c.power.wrapping_add(i32::from(m)))
                .wrapping_mul(10);
            let Some(x) = int(ten).over(s) else {
                // 0 / 0 or ±∞ / 0, times a strength of 0: the indefinite's low half.
                return 0;
            };
            let x = at_most(x, constant(RAISE_CAPS.0, 10));
            let x = at_most(x.times(curve(capped(casts), 0)), constant(RAISE_CAPS.1, 10));
            s.times(x).ftol()
        }
        Some(SpellAiKind::Special) => {
            let root = SignedX87::from(X87::int(u64::try_from(casts).unwrap_or(0)).sqrt());
            let a = root
                .times(constant(SPECIAL_FACTORS.0, 1000))
                .plus(constant(SPECIAL_FACTORS.1, 1000));
            a.times(int(v.wrapping_mul(c.strength))).ftol()
        }
        None => 1,
    }
}

/// `x` held to `cap` as the code holds it (`fcomp`, then the constant stored when `x` is
/// above).
fn at_most(x: SignedX87, cap: SignedX87) -> SignedX87 {
    if greater(x, cap) { cap } else { x }
}

/// `a > b` for two values of the FPU.
fn greater(a: SignedX87, b: SignedX87) -> bool {
    let d = a.minus(b);
    !d.is_negative() && d != SignedX87::ZERO
}

/// The curve's row of `n` casts (`min(n, 13)`; at least 0).
fn capped(n: i32) -> usize {
    usize::try_from(n.clamp(0, CURVE_CASTS)).unwrap_or(0)
}

/// The damage value of `spell` at mastery `m` with `n` casts against the strength `s`
/// (`ai_damage_spell_value` 0x5270E0; CPU-SPL-003): the base `b = v_m × (P + m)` (32 bits), `d`
/// the hero's bonuses on it without a target stack (`hero_spell_damage_bonus` 0x4E5760 given
/// none: the Orbs and Sorcery), `X = 10 d / s` in doubles, `Y = α_k X + β_k` for the first row
/// `k` of 0 to 12 whose threshold `X` reaches (13 without one), `Z = φ_min(n, 13) X`, and
/// `trunc(s × Z)` when `Y > Z`, else `trunc(s × Y)`; 0 for `s` = 0 (the original's NaN). The
/// code's case of Titan's Lightning Bolt (its effect as the base) is never reached: its two AI
/// kinds value it at 1.
// rule: CPU-SPL-003
fn damage_value(
    ruleset: &BattleRuleset,
    c: &SpellCaster<'_>,
    (_, spell): SpellRef<'_>,
    m: u8,
    (n, s): (i32, i32),
) -> i32 {
    let v = i32::from(spell.ai.values[usize::from(m.min(MAX_MASTERY))]);
    let b = v.wrapping_mul(c.power.wrapping_add(i32::from(m)));
    let d = magic::untargeted_bonus(
        ruleset,
        spell,
        &c.magic.bonuses,
        u64::try_from(b).unwrap_or(0),
    );
    let d = i32::try_from(d).unwrap_or(i32::MAX);
    let s = int(s);
    let Some(x) = int(d.wrapping_mul(10)).over(s) else {
        return 0;
    };
    let k = (0..SPELL_CURVE.len() - 1)
        .find(|&k| !greater(curve(k, 1), x))
        .unwrap_or(SPELL_CURVE.len() - 1);
    let y = x.times(curve(k, 2)).plus(curve(k, 3));
    let z = x.times(curve(capped(n), 0));
    s.times(if greater(y, z) { z } else { y }).ftol()
}

/// The value of a damage spell on every stack (`ai_mass_damage_spell_value` 0x5271C0;
/// CPU-SPL-004): over the hero's army slots in order, `A = trunc(w × count × (1 − f) + A)` in
/// doubles, `w` the creature's AI value and `f` its part of the effect test under the hero with no
/// controlling hero (`creature_spell_factor` 0x44A4D0): the army's value the spell spares; 0 when
/// `A` is 0, else the [`damage_value`] against `A` less the part it hurts (the strength less
/// `A`), at least 0 (32 bits).
// rule: CPU-SPL-004
fn mass_value(
    ruleset: &BattleRuleset,
    c: &SpellCaster<'_>,
    (id, spell): SpellRef<'_>,
    m: u8,
    n: i32,
) -> i32 {
    let guard = Guard::of_heroes(Some(c.magic), None);
    let mut spared = 0_i32;
    for slot in 0..SLOTS {
        let Some(stack) = c.army.iter().find(|s| s.slot == slot) else {
            continue;
        };
        // The factor is returned as an `f32` (0x44A8C5) and stored as a double.
        let f = magic::creature_factor(ruleset, (id, spell), stack.creature, guard, 0)
            .round(h3_kernel::arith::SINGLE_BITS);
        let w = ruleset
            .creature_info(stack.creature)
            .map_or(0, |i| i32::from_ne_bytes(i.ai_value.to_ne_bytes()));
        let count = i32::from_ne_bytes(stack.count.to_ne_bytes());
        spared = int(w)
            .times(int(count))
            .times(SignedX87::ONE.minus(SignedX87::from(f)))
            .plus(int(spared))
            .ftol();
    }
    if spared == 0 {
        return 0;
    }
    let hurt = spared.wrapping_sub(c.strength);
    hurt.wrapping_add(damage_value(ruleset, c, (id, spell), m, (n, spared)))
        .max(0)
}

/// The value of any other battle spell (`ai_effect_spell_value` 0x527290; CPU-SPL-005): its
/// rounds `e` the record's rounds times the casts `n` (32 bits); a spell not aimed at a stack at
/// `m` (0x59E060) counts the whole army list with `e` held to 7; one aimed at a stack one entry
/// while `e` is below 7, else `min(e / 7, rounds)` entries for 7 rounds each; over those first
/// entries of the list (for a positive spell only those its effect test does not spare:
/// `creature_spell_factor` 0x44A4D0 under the hero, no controlling hero, above 0), the sum of
/// `v_m × the entry's value × e` (32 bits), over 700 (toward 0).
// rule: CPU-SPL-005
fn effect_value(
    ruleset: &BattleRuleset,
    c: &SpellCaster<'_>,
    (id, spell): SpellRef<'_>,
    m: u8,
    n: i32,
) -> i32 {
    let mut e = c.rounds.wrapping_mul(n);
    let entries = if !takes_stack(ruleset, id, m) {
        e = e.min(EFFECT_ROUNDS);
        c.entries.len()
    } else if e < EFFECT_ROUNDS {
        1
    } else {
        let take = (e / EFFECT_ROUNDS).min(c.rounds);
        e = EFFECT_ROUNDS;
        usize::try_from(take).unwrap_or(0)
    };
    let v = i32::from(spell.ai.values[usize::from(m.min(MAX_MASTERY))]);
    let guard = Guard::of_heroes(Some(c.magic), None);
    let sum = c
        .entries
        .iter()
        .take(entries)
        .filter(|entry| {
            spell.polarity != SpellPolarity::Positive
                || magic::creature_factor(ruleset, (id, spell), entry.creature, guard, 0)
                    != X87::ZERO
        })
        .fold(0_i32, |sum, entry| {
            sum.wrapping_add(v.wrapping_mul(entry.value).wrapping_mul(e))
        });
    sum / EFFECT_DIVISOR
}

/// The best value among `c`'s spells whose AI kinds share one with `kinds` (`ai_best_spell_value`
/// 0x5275B0; CPU-SPL-006): 0 when its power is 0; else the largest [`value`], from 0, among its
/// available spells of those kinds, none above its own spell level limit (Recanter's Cloak on
/// the hero: level 2).
// rule: CPU-SPL-006
pub(crate) fn best(ruleset: &BattleRuleset, c: &SpellCaster<'_>, kinds: SpellAiKinds) -> i32 {
    if c.power == 0 {
        return 0;
    }
    c.spells
        .iter()
        .filter_map(|id| ruleset.spell(id).map(|spell| (id, spell)))
        .filter(|(_, spell)| {
            spell.ai.kinds.bits() & kinds.bits() != 0 && spell.level <= c.magic.level_limit
        })
        .map(|spell| value(ruleset, c, spell))
        .fold(0, i32::max)
}
