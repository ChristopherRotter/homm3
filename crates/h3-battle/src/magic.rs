//! A hero's magic in battle (E04b T2b–T3a; ADR-0014 points 2, 4 and 8): spell books, casting by
//! a hero (BAT-MAG-001), mastery and cost (BAT-MAG-002), targets and mass versions (BAT-MAG-003),
//! the damage of damage spells (BAT-MAG-004), immunities (BAT-MAG-005), Chain Lightning
//! (BAT-MAG-006), area and battle-wide spells (BAT-MAG-007), magic resistance (BAT-MAG-008) and
//! timed spells (BAT-MAG-009 to 011, [`crate::effects`]).
//!
//! A hero casts during the turn of one of its side's stacks that awaits a command, at most
//! `hero_casts_per_round` times per round and never by obligation; the stack still acts
//! afterwards. Casting is not part of [`crate::legal_commands`]: [`castable_spells`] and
//! [`cast_targets`] list what [`crate::apply`] accepts, spell first, then the target (the
//! factorized casting action of ADR-0014 point 9).
//!
//! The order of the checks and factors follows the original's code (`docs/formats/
//! installation.md` §7.8): whether the spell can affect a stack at all, then one resistance roll
//! per stack, then the damage: the caster's bonuses before the target's reductions.

use core::num::NonZeroU32;

use h3_content::{
    CreatureFlag, CreatureTypeId, EARLY_MASS_VERSION, FIRST_PROTECTION, FULL_BP,
    MASS_CURE_SKIPS_HYPNOTIZED, MAX_SPELL_SCHOOLS, MAX_SPELLS, SINGLE_PRECISION_RATES,
    SpellCategory, SpellId, SpellScope,
};
use h3_content_battle::{
    Ability, BattleRuleset, MAX_FIELD_HEXES, SpecialistBonus, SpellEffect, SpellPolarity,
    SpellTarget, TargetSide, TimedChange,
};
use h3_content_battle::{BattleSpell, Battlefield, BattlefieldId, MAX_MASTERY};
use h3_contracts::{BattleEffect, BattleHero, BattleSide, Rate};
use h3_kernel::arith::{
    DOUBLE_BITS, SINGLE_BITS, SINGLE_FRAC_BITS, X87, mul_div_floor, round_single, single_precision,
};
use h3_kernel::{CanonicalHash, FixedVec, StateHasher};

use crate::casters::{self, CastSpell, CreatureSpell};
use crate::combat::check_end;
use crate::created;
use crate::damage::apply_damage;
use crate::effects::{self, Status, TimedCast};
use crate::error::{CommandError, HeroError};
use crate::event::{Event, EventKind, Sink};
use crate::geometry::{Field, Footprint, Hex, HexSet};
use crate::hero::{HeroRecord, hash_rate};
use crate::movement;
use crate::obstacles::{self, SpellObstacle};
use crate::raise;
use crate::state::{BattleState, STACK_CAPACITY, Stack, StackId, StackTraits};

/// A set of spells by handle (the content allows at most [`MAX_SPELLS`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SpellSet([u64; 2]);

impl SpellSet {
    /// The word and bit of `spell`, if it fits.
    fn position(spell: SpellId) -> Option<(usize, u64)> {
        let i = spell.index();
        (i < MAX_SPELLS).then(|| (i / 64, 1 << (i % 64)))
    }

    /// Whether `spell` is in the set.
    pub fn contains(self, spell: SpellId) -> bool {
        Self::position(spell).is_some_and(|(w, bit)| self.0[w] & bit != 0)
    }

    /// Adds `spell` (spells beyond [`MAX_SPELLS`] cannot exist in a ruleset and are ignored).
    pub fn insert(&mut self, spell: SpellId) {
        if let Some((w, bit)) = Self::position(spell) {
            self.0[w] |= bit;
        }
    }

    /// Whether the set is empty.
    pub const fn is_empty(self) -> bool {
        self.0[0] == 0 && self.0[1] == 0
    }

    /// The spells, by handle.
    pub fn iter(self) -> impl Iterator<Item = SpellId> {
        (0..MAX_SPELLS)
            .filter(move |&i| self.0[i / 64] & (1 << (i % 64)) != 0)
            .filter_map(|i| u16::try_from(i).ok().map(SpellId::from_raw))
    }

    /// The raw bits (bit `i` of word `i / 64` is the spell with handle `i`).
    pub const fn bits(self) -> [u64; 2] {
        self.0
    }
}

/// A hero's mastery of each spell (E04b T8a, BAT-MAG-002): 0 (none) to 3 (Expert) as two bit
/// planes over [`SpellSet`]'s positions, so that the hero record grows by 28 bytes, not by one
/// byte per spell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Masteries {
    /// Bit 0 of each mastery.
    low: SpellSet,
    /// Bit 1 of each mastery.
    high: SpellSet,
}

impl Masteries {
    /// The mastery of `spell` (0 for a spell not set).
    pub fn get(self, spell: SpellId) -> u8 {
        u8::from(self.low.contains(spell)) | (u8::from(self.high.contains(spell)) << 1)
    }

    /// The masteries of a `(spell, mastery)` list (checked: each spell once, at most 3).
    pub(crate) fn of(list: &[(SpellId, u8)]) -> Self {
        let mut out = Self::default();
        for &(spell, mastery) in list {
            out.set(spell, mastery);
        }
        out
    }

    /// Sets the mastery of `spell` from 0 (checked: at most 3).
    fn set(&mut self, spell: SpellId, mastery: u8) {
        if mastery & 1 != 0 {
            self.low.insert(spell);
        }
        if mastery & 2 != 0 {
            self.high.insert(spell);
        }
    }

    /// The raw bits of both planes (low, then high).
    pub const fn bits(self) -> [u64; 4] {
        let [a, b] = self.low.bits();
        let [c, d] = self.high.bits();
        [a, b, c, d]
    }
}

/// Most spells with a bonus of their own a battle hero may have (BAT-HER-001).
pub const MAX_SPELL_BONUSES: usize = 4;

/// A battle hero's bonuses for one spell (HER-BAT-012).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SpellBonus {
    /// The spell.
    pub spell: SpellId,
    /// More damage from the spell, basis points (Ciele's and Luna's specialties).
    pub damage_bp: u32,
    /// A specialty's growth per hero level, basis points (3 % in SoD).
    pub per_level_bp: u32,
    /// The hero's level for the specialty.
    pub level: u8,
}

/// A battle hero's spell-damage bonuses as the battle keeps them (private to its side):
/// its `SpellDamage` and `SpellSpecialty` effects summed per scope (HER-BAT-012).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SpellBonuses {
    /// More damage from every spell (Sorcery).
    pub all: Rate,
    /// More damage from the spells of each school, by school handle (the Orbs).
    pub school: [Rate; MAX_SPELL_SCHOOLS],
    /// Bonuses of single spells, in the order of their first effect.
    pub spells: FixedVec<SpellBonus, MAX_SPELL_BONUSES>,
}

impl SpellBonuses {
    /// The bonuses of `hero`'s effects, or `None` if more than [`MAX_SPELL_BONUSES`] spells have
    /// bonuses of their own.
    fn of(hero: &BattleHero) -> Option<Self> {
        let mut out = Self::default();
        for effect in &hero.effects {
            let (spell, damage_bp, per_level_bp, level) = match *effect {
                BattleEffect::SpellDamage { scope, rate } => match scope {
                    SpellScope::All => {
                        out.all = add_rates(out.all, rate);
                        continue;
                    }
                    SpellScope::School(l) => {
                        // Validated: the school exists, and there are at most four.
                        let i = l.index().min(MAX_SPELL_SCHOOLS - 1);
                        out.school[i] = add_rates(out.school[i], rate);
                        continue;
                    }
                    // A single spell's bonus is whole (Ciele's `t >> 1`, Luna's `t`): its
                    // basis points.
                    SpellScope::Spell(s) => (s, rate.bp, 0, 0),
                },
                BattleEffect::SpellSpecialty {
                    spell,
                    bp_per_level,
                    level,
                } => (spell, 0, u32::from(bp_per_level), level),
                _ => continue,
            };
            let index = match out.spells.iter().position(|b| b.spell == spell) {
                Some(i) => i,
                None => {
                    out.spells
                        .push(SpellBonus {
                            spell,
                            ..SpellBonus::default()
                        })
                        .ok()?;
                    out.spells.len() - 1
                }
            };
            let bonus = &mut out.spells.as_mut_slice()[index];
            bonus.damage_bp = bonus.damage_bp.saturating_add(damage_bp);
            bonus.per_level_bp = bonus.per_level_bp.saturating_add(per_level_bp);
            bonus.level = bonus.level.max(level);
        }
        Some(out)
    }

    /// Whether a spell specialty is among them (BAT-MAG-020).
    pub(crate) fn has_specialty(&self) -> bool {
        self.spells.iter().any(|b| b.per_level_bp > 0)
    }

    /// The bonuses for `spell` (zero without any).
    pub(crate) fn spell(&self, spell: SpellId) -> SpellBonus {
        self.spells
            .iter()
            .find(|b| b.spell == spell)
            .copied()
            .unwrap_or_default()
    }
}

/// The single-precision value of `bp` basis points, as a numerator over `2^SINGLE_FRAC_BITS`.
fn single_bp(bp: u32) -> u64 {
    X87::ratio(bp, u32::from(FULL_BP), SINGLE_BITS).map_or(0, X87::to_fixed)
}

/// The hero's `rate` at the [`scale`] (OPEN-QUESTIONS Q45): its basis points, or with `single`
/// (quirk Q-0005) its single-precision value, the original's where the derivation gave one.
pub(crate) fn hero_rate(single: bool, r: Rate) -> u64 {
    match (single, r.single) {
        (false, _) => u64::from(r.bp),
        (true, Some(bits)) => X87::from_f32_bits(bits.get()).to_fixed(),
        (true, None) => single_bp(r.bp),
    }
}

/// The sum of two of a hero's rates: the basis points added (saturating) and, if either has a
/// single-precision value of its own, the two single-precision values added and rounded to
/// single precision, as the original adds its stored rates.
fn add_rates(a: Rate, b: Rate) -> Rate {
    Rate {
        bp: a.bp.saturating_add(b.bp),
        single: (a.single.is_some() || b.single.is_some())
            .then(|| {
                let sum = hero_rate(true, a).saturating_add(hero_rate(true, b));
                NonZeroU32::new(X87::fixed(sum).to_f32_bits())
            })
            .flatten(),
    }
}

/// The numerator of 1 for rates: basis points, or `2^SINGLE_FRAC_BITS` for the single-precision
/// values of quirk Q-0005.
const fn scale(single: bool) -> u64 {
    if single {
        1 << SINGLE_FRAC_BITS
    } else {
        FULL_BP as u64
    }
}

/// A hero's [`HeroMagic::resistance`] in basis points, truncated (observations, BAT-Q26): at
/// most [`FULL_BP`], as the resistance is.
pub(crate) fn resistance_bp(ruleset: &BattleRuleset, resistance: u64) -> u32 {
    let one = scale(ruleset.quirks().is_enabled(SINGLE_PRECISION_RATES));
    let bp = mul_div_floor(resistance.min(one), u64::from(FULL_BP), one).unwrap_or(0);
    u32::try_from(bp).unwrap_or(0)
}

/// The rate `bp` (basis points) at the [`scale`]: exact, or its single-precision value.
fn rate(single: bool, bp: u16) -> u64 {
    if single {
        // `FULL_BP` is not 0.
        single_precision(bp, FULL_BP).unwrap_or(0)
    } else {
        u64::from(bp)
    }
}

/// What the battle keeps of a hero's magic besides its mana and casts this round (which are
/// public, `HeroView`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct HeroMagic {
    /// The spells in its spell book (none without a book; private).
    pub(crate) book: SpellSet,
    /// Its mastery of each spell, as the seam gives it (BAT-MAG-002; private).
    pub(crate) masteries: Masteries,
    /// Spell-damage bonuses (private).
    pub(crate) bonuses: SpellBonuses,
    /// The resistance of the side's stacks to hostile spells (BAT-MAG-008; private): its
    /// `MagicResistance` effects added in their order, at most 1, as a numerator over the
    /// [`scale`]; with quirk Q-0005 each sum is rounded to single precision, as the original
    /// stores it.
    pub(crate) resistance: u64,
    /// Rounds added to the duration of its timed spells: its `SpellDuration` effects summed
    /// (HER-BAT-014, BAT-MAG-010; private).
    pub(crate) duration_bonus: u16,
    /// Its opening spells, in their order (BAT-MAG-068; private until cast).
    pub(crate) opening: FixedVec<Opening, MAX_OPENING_SPELLS>,
    /// Whether its side's first activation outside the tactics phase, when it casts them, is
    /// still to come (the original's +0x54B0; BAT-MAG-068): set at the start if it has some.
    pub(crate) opening_due: bool,
    /// Its Eagle Eye's chance to pick a spell at a cast of the enemy hero, in percent: `trunc(p ×
    /// 100)` of the seam's rate at the [`scale`] (BAT-MAG-069; 0 without the skill; private).
    pub(crate) eagle_chance: u8,
    /// The spells its Eagle Eye may pick (BAT-MAG-069; private).
    pub(crate) eagle_spells: SpellSet,
    /// The spells its Eagle Eye picked so far (BAT-MAG-069; private; the outcome's `learned`).
    pub(crate) learned: SpellSet,
    /// The change of its spell costs from the creatures in the armies (BAT-MAG-054; set at the
    /// battle's start, public through the armies).
    pub(crate) cost_change: i8,
    /// Whether it forbids every cast in the battle: a `NoCasting` effect (BAT-MAG-061; the Orb of
    /// Inhibition, private until a cast is refused).
    pub(crate) no_casting: bool,
    /// The highest spell level it allows in the battle: the smallest of its `SpellLevelLimit`
    /// effects, `u8::MAX` without one (BAT-MAG-062; Recanter's Cloak).
    pub(crate) level_limit: u8,
    /// What it adds to the immunity of the stacks it controls (BAT-MAG-063 to BAT-MAG-065;
    /// private until a spell fails).
    pub(crate) defences: Defences,
    /// Whether it negates immunities and resistance: a `NegateImmunities` effect (BAT-MAG-066;
    /// the Orb of Vulnerability, private).
    pub(crate) negates: bool,
}

/// Most opening spells of one hero (BAT-MAG-068): Armor of the Damned has four.
pub const MAX_OPENING_SPELLS: usize = 8;

/// One opening spell of a hero (BAT-MAG-068; private to its side until cast, BAT-Q26).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Opening {
    /// The spell.
    pub spell: SpellId,
    /// Its mastery.
    pub mastery: u8,
    /// Its power, the duration of its effects.
    pub power: u16,
}

impl CanonicalHash for Opening {
    /// Spell, mastery, power.
    fn hash_into(&self, h: &mut StateHasher) {
        self.spell.hash_into(h);
        h.write_u8(self.mastery);
        h.write_u16(self.power);
    }
}

/// What the heroes and the battlefield of a battle do to all of its magic (E04b T8b, E04c T2b),
/// derived from them at its start for the hot paths (not hashed: the heroes, the battlefield and
/// the anti-magic flag are).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct MagicLimits {
    /// A hero or an anti-magic garrison forbids every cast (BAT-MAG-061).
    pub(crate) no_casting: bool,
    /// The battlefield forbids creature casts, raises and periodic casts (BAT-TER-004).
    pub(crate) no_creature_casts: bool,
    /// The highest spell level allowed: the smallest limit of the heroes and the battlefield,
    /// `u8::MAX` without one (BAT-MAG-062, BAT-TER-004).
    pub(crate) max_level: u8,
    /// The battlefield raises the casts that are not a hero's to Expert (BAT-TER-005).
    pub(crate) others_expert: bool,
    /// A hero has spell defences or negates immunities (BAT-MAG-063 to BAT-MAG-066): without one
    /// no stack's [`Guard`] is looked at.
    pub(crate) guards: bool,
}

impl MagicLimits {
    /// The limits of a battle with `heroes` on `battlefield`, at an anti-magic garrison if
    /// `anti_magic`.
    pub(crate) fn of(
        heroes: &[Option<HeroRecord>; 2],
        battlefield: Option<&Battlefield>,
        anti_magic: bool,
    ) -> Self {
        let magic = || heroes.iter().flatten().map(|h| &h.magic);
        let field_level = battlefield.map_or(u8::MAX, |b| b.max_spell_level);
        Self {
            no_casting: anti_magic || magic().any(|m| m.no_casting),
            no_creature_casts: battlefield.is_some_and(|b| b.no_creature_casts),
            max_level: magic()
                .map(|m| m.level_limit)
                .min()
                .unwrap_or(u8::MAX)
                .min(field_level),
            others_expert: battlefield.is_some_and(|b| b.expert_other_casts),
            guards: magic().any(|m| m.negates || m.defences != Defences::default()),
        }
    }
}

/// What a hero adds to the immunity of the stacks it controls (E04b T8b2; private to its side,
/// BAT-Q26).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Defences {
    /// The spells they are never affected by: its `SpellImmunity` effects (BAT-MAG-063).
    pub spells: SpellSet,
    /// The highest level of the spells they are never affected by, Dispel excepted: the largest
    /// of its `SpellLevelImmunity` effects (BAT-MAG-064).
    pub up_to_level: Option<u8>,
    /// Whether they are immune to mind spells: a `MindImmunity` effect (BAT-MAG-065).
    pub mind: bool,
}

impl HeroMagic {
    /// The magic of `hero` (checked by `hero::check`); `single` selects the single-precision
    /// rates of quirk Q-0005.
    pub(crate) fn of(hero: &BattleHero, single: bool) -> Result<Self, HeroError> {
        let mut book = SpellSet::default();
        for &spell in hero.magic.spell_book.as_deref().unwrap_or_default() {
            book.insert(spell);
        }
        let masteries = Masteries::of(&hero.magic.masteries);
        let mut resistance = 0u64;
        let mut duration_bonus = 0u16;
        let mut no_casting = false;
        let mut level_limit = u8::MAX;
        let mut defences = Defences::default();
        let mut negates = false;
        let mut opening = FixedVec::default();
        for effect in &hero.effects {
            match *effect {
                BattleEffect::MagicResistance { rate } => {
                    let sum = resistance.saturating_add(hero_rate(single, rate));
                    resistance = if single { round_single(sum) } else { sum };
                }
                BattleEffect::SpellDuration { rounds } => {
                    duration_bonus = duration_bonus.saturating_add(rounds.into());
                }
                BattleEffect::NoCasting => no_casting = true,
                BattleEffect::SpellLevelLimit { max_level } => {
                    level_limit = level_limit.min(max_level);
                }
                BattleEffect::SpellImmunity { spell } => defences.spells.insert(spell),
                BattleEffect::SpellLevelImmunity { max_level } => {
                    defences.up_to_level = defences.up_to_level.max(Some(max_level));
                }
                BattleEffect::MindImmunity => defences.mind = true,
                BattleEffect::NegateImmunities => negates = true,
                BattleEffect::OpeningSpell {
                    spell,
                    mastery,
                    power,
                } => opening
                    .push(Opening {
                        spell,
                        mastery,
                        power,
                    })
                    .map_err(|_| HeroError::OpeningSpell)?,
                _ => {}
            }
        }
        let mut eagle_spells = SpellSet::default();
        let mut eagle_chance = 0;
        if let Some(eagle) = &hero.magic.eagle_eye {
            for &spell in &eagle.spells {
                eagle_spells.insert(spell);
            }
            // rule: BAT-MAG-069
            // The seam's rate is at most 1 (checked by `hero::check`).
            let percent = hero_rate(single, eagle.chance) * 100 / scale(single);
            eagle_chance = u8::try_from(percent.min(100)).unwrap_or(100);
        }
        Ok(Self {
            book,
            masteries,
            bonuses: SpellBonuses::of(hero).ok_or(HeroError::TooManySpellBonuses)?,
            resistance: resistance.min(scale(single)),
            duration_bonus,
            opening_due: !opening.is_empty(),
            opening,
            eagle_chance,
            eagle_spells,
            learned: SpellSet::default(),
            cost_change: 0,
            no_casting,
            level_limit,
            defences,
            negates,
        })
    }
}

impl CanonicalHash for HeroMagic {
    /// Book bits, Eagle Eye's picked spells' bits (v41, before them the cast spells'), the
    /// masteries' bit planes (v39, before them the school masteries), the bonuses (every spell,
    /// by school (rates since v23), the single spells as length and spell, damage, growth per
    /// level, level), the resistance, the duration bonus (v20), the cost change (v38, as its
    /// two's-complement byte), whether it forbids casting, its spell level limit, its defences
    /// (the spells' bits, the level, mind) and whether it negates immunities (v40), the opening
    /// spells, whether they are due, Eagle Eye's chance and spells' bits (v41).
    fn hash_into(&self, h: &mut StateHasher) {
        for word in self.book.bits().into_iter().chain(self.learned.bits()) {
            h.write_u64(word);
        }
        for word in self.masteries.bits() {
            h.write_u64(word);
        }
        let b = &self.bonuses;
        for r in core::iter::once(b.all).chain(b.school) {
            hash_rate(h, r);
        }
        h.write_len(b.spells.len());
        for s in &b.spells {
            s.spell.hash_into(h);
            h.write_u32(s.damage_bp);
            h.write_u32(s.per_level_bp);
            h.write_u8(s.level);
        }
        h.write_u64(self.resistance);
        h.write_u16(self.duration_bonus);
        h.write_u8(self.cost_change.to_le_bytes()[0]);
        h.write_bool(self.no_casting);
        h.write_u8(self.level_limit);
        for word in self.defences.spells.bits() {
            h.write_u64(word);
        }
        self.defences.up_to_level.hash_into(h);
        h.write_bool(self.defences.mind);
        h.write_bool(self.negates);
        self.opening.hash_into(h);
        h.write_bool(self.opening_due);
        h.write_u8(self.eagle_chance);
        for word in self.eagle_spells.bits() {
            h.write_u64(word);
        }
    }
}

/// What a cast is aimed at (ADR-0014 point 3): its target, and its second target where the spell
/// takes one (Teleport's destination hex, BAT-MAG-029; Sacrifice's victim stack, BAT-MAG-036).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum CastTarget {
    /// Nothing (Armageddon).
    #[default]
    None,
    /// A stack.
    Stack(StackId),
    /// A hex.
    Hex(Hex),
}

impl CastTarget {
    /// The hex, if this is one.
    pub const fn hex(self) -> Option<Hex> {
        match self {
            Self::Hex(h) => Some(h),
            Self::None | Self::Stack(_) => None,
        }
    }

    /// The stack, if this is one.
    pub const fn stack(self) -> Option<StackId> {
        match self {
            Self::Stack(s) => Some(s),
            Self::None | Self::Hex(_) => None,
        }
    }
}

/// Most targets of one spell: none, every stack, every hex.
pub const MAX_CAST_TARGETS: usize = 1 + STACK_CAPACITY + MAX_FIELD_HEXES;

/// Fixed-capacity buffer for [`cast_targets`].
pub type CastTargets = FixedVec<CastTarget, MAX_CAST_TARGETS>;

/// A cast that passed validation, with what its resolution needs.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Cast<'r> {
    side: BattleSide,
    id: SpellId,
    spell: &'r BattleSpell,
    cost: u16,
    mastery: u8,
    target: CastTarget,
    /// The second target (Teleport's destination, Sacrifice's victim).
    second: CastTarget,
    /// `power × power factor + effect[mastery]` (BAT-MAG-004).
    value: u64,
    /// The caster's bonuses (BAT-MAG-004).
    bonuses: SpellBonuses,
    /// The duration of its timed effects: power plus the duration bonus, at least 1
    /// (BAT-MAG-010); a clone's rounds (BAT-MAG-038).
    rounds: u16,
    /// The caster's power (a summon's count, BAT-MAG-039).
    power: u16,
    /// The stack that casts as its action (BAT-MAG-048); none for the side's hero.
    by: Option<StackId>,
    /// Whether the casting stack draws the spell at the cast among those useful for the target
    /// (BAT-MAG-050): `id` is then the first of them until the draw.
    draws: bool,
    /// Whether the stack casts on its own at its activation (a periodic caster, BAT-MAG-053): no
    /// cast is used.
    automatic: bool,
}

/// Whether the battle implements the effects of `spell`: every effect kind is implemented
/// (damage since E04b T2, timed changes since T3a, dispel and heal since T3b1b), so a spell is
/// castable once it has effects.
fn implemented(spell: &BattleSpell) -> bool {
    !spell.magic.effects.is_empty()
}

/// The mastery at which `hero` casts `spell`: the one the seam gives it (derived in the Heroes
/// context, HER-SPL-003; BAT-MAG-002), Expert when the battlefield raises one of the spell's
/// schools (BAT-TER-005; the original's 0x4E5080 with the field, `installation.md` §7.26).
// rule: BAT-MAG-002, BAT-TER-005
pub(crate) fn mastery(
    state: &BattleState,
    ruleset: &BattleRuleset,
    hero: &HeroRecord,
    spell: SpellId,
) -> u8 {
    mastery_at(ruleset, state.battlefield, hero, spell)
}

/// [`mastery`] on `battlefield` (also before the battle's state exists: a town's land mines,
/// BAT-SIE-007).
pub(crate) fn mastery_at(
    ruleset: &BattleRuleset,
    battlefield: Option<BattlefieldId>,
    hero: &HeroRecord,
    spell: SpellId,
) -> u8 {
    let raised = battlefield
        .and_then(|b| ruleset.battlefield(b))
        .zip(ruleset.spell(spell))
        .is_some_and(|(b, s)| b.raises(&s.schools));
    if raised {
        MAX_MASTERY
    } else {
        hero.magic.masteries.get(spell)
    }
}

/// The mastery of a cast that is not a hero's own (a creature's, a periodic caster's, a strike's
/// spell, an opening spell) at `mastery`: Expert where the battlefield raises those
/// (BAT-TER-005; the original's 0x59FEBC: Magic Plains).
// rule: BAT-TER-005
pub(crate) fn other_mastery(state: &BattleState, mastery: u8) -> u8 {
    if state.limits.others_expert {
        MAX_MASTERY
    } else {
        mastery
    }
}

/// Checks that `side`'s hero may cast `id` now apart from its target (BAT-MAG-001, BAT-MAG-002)
/// and returns the spell, the hero, the mastery and the cost. The caller has checked that a
/// stack of `side` awaits a command outside the tactics phase.
// rule: BAT-MAG-001
fn check_spell<'s, 'r>(
    state: &'s BattleState,
    ruleset: &'r BattleRuleset,
    side: BattleSide,
    id: SpellId,
) -> Result<(&'r BattleSpell, &'s HeroRecord, u8, u16), CommandError> {
    let hero = state.heroes[side.index()]
        .as_ref()
        .filter(|h| h.magic.book.contains(id))
        .ok_or(CommandError::NotInSpellBook)?;
    // A spell in a book is a spell of the ruleset (BAT-HER-001).
    let spell = ruleset.spell(id).ok_or(CommandError::NotInSpellBook)?;
    if spell.category != SpellCategory::Combat {
        return Err(CommandError::NotACombatSpell);
    }
    if hero.view.casts >= ruleset.battle().hero_casts_per_round {
        return Err(CommandError::AlreadyCast);
    }
    // rule: BAT-MAG-061
    if casting_forbidden(state) {
        return Err(CommandError::CastingForbidden);
    }
    // rule: BAT-MAG-062
    if barred(state, spell) {
        return Err(CommandError::SpellLevelBarred);
    }
    if !implemented(spell) {
        return Err(CommandError::NotCastable);
    }
    let mastery = mastery(state, ruleset, hero, id);
    // rule: BAT-MAG-054
    let cost = crate::mana::cost(spell, mastery, hero.magic.cost_change);
    if hero.view.mana < cost {
        return Err(CommandError::NotEnoughMana);
    }
    Ok((spell, hero, mastery, cost))
}

/// A spell together with its handle.
pub(crate) type SpellRef<'r> = (SpellId, &'r BattleSpell);

/// Whether a hero in the battle forbids every cast (BAT-MAG-061; the original's 0x41F890 and the
/// hero's book button 0x59E900 refuse while either hero wears the Orb of Inhibition (126),
/// `installation.md` §7.23): no hero's spell, no creature's cast or raise, no periodic cast. The
/// strike's creature magic is no cast and stays.
// rule: BAT-MAG-061
pub(crate) fn casting_forbidden(state: &BattleState) -> bool {
    state.limits.no_casting
}

/// Whether the creatures may not cast, raise or cast periodically (BAT-MAG-061 and BAT-TER-004:
/// the original's 0x41F890 with its flag 0 answers 0 also on Cursed Ground, `installation.md`
/// §7.26). The strike's creature magic is no cast and stays (its spells' levels are barred).
// rule: BAT-TER-004
pub(crate) fn creature_casting_forbidden(state: &BattleState) -> bool {
    state.limits.no_casting || state.limits.no_creature_casts
}

/// Whether the battle's spell level limit bars `spell` (BAT-MAG-062; the original's book button
/// 0x59E900 refuses a hero's spell above level 2 and the effect test 0x5A8090 gives 0 for every
/// spell above level 2, creature abilities included, while either hero wears Recanter's Cloak
/// (83), `installation.md` §7.23): its level is above a hero's `SpellLevelLimit`.
// rule: BAT-MAG-062
pub(crate) fn barred(state: &BattleState, spell: &BattleSpell) -> bool {
    spell.level > state.limits.max_level
}

/// The heroes' part of whether a spell of one side affects a stack (E04b T8b2): the defences of
/// the hero controlling the stack and whether the caster's hero or that one negates immunities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct Guard {
    defences: Defences,
    negated: bool,
}

impl Guard {
    /// Whether the controlling hero's defences name `spell` (BAT-MAG-063).
    pub(crate) fn immune_to(self, spell: SpellId) -> bool {
        self.defences.spells.contains(spell)
    }

    /// For a spell of `caster`'s side at stack `s`: the controller's hero's defences (a hypnotized
    /// stack's are its controller's, as the original's 0x442690), negated if the caster's or the
    /// controller's hero negates immunities (BAT-MAG-066; the original asks both, 0x44A746).
    pub(crate) fn of(state: &BattleState, caster: BattleSide, s: StackId) -> Self {
        // Most battles have no such hero: then no stack is looked at.
        if !state.limits.guards {
            return Self::default();
        }
        let controller = state.at(s).controller();
        let hero = |side: BattleSide| state.heroes[side.index()].as_ref();
        Self {
            defences: hero(controller)
                .map(|h| h.magic.defences)
                .unwrap_or_default(),
            negated: [caster, controller]
                .into_iter()
                .any(|side| hero(side).is_some_and(|h| h.magic.negates)),
        }
    }
}

/// Whether a cast of `spell` can affect a stack of `creature` under `guard` at all (BAT-MAG-005,
/// in the original's order, `installation.md` §7.8, §7.23): not if the controlling hero's
/// defences name the spell (BAT-MAG-063); a spell that ignores immunity (Dispel) needs only the
/// spell's conditions; otherwise not up to the defences' level (BAT-MAG-064), the conditions (by
/// flags and the creature's damage, BAT-MAG-067) and the creature's absolute immunities hold, and
/// unless the immunities are negated (BAT-MAG-066)
/// the creature is not immune to it as a mind spell (flag `mind_immune`, or the controlling
/// hero's mind immunity, BAT-MAG-065), by the spell's level, one of its schools or the spell
/// itself.
// rule: BAT-MAG-005, BAT-MAG-063, BAT-MAG-064, BAT-MAG-065, BAT-MAG-066, BAT-MAG-067
pub(crate) fn affects(
    ruleset: &BattleRuleset,
    (id, spell): SpellRef<'_>,
    creature: CreatureTypeId,
    guard: Guard,
) -> bool {
    if guard.immune_to(id) {
        return false;
    }
    let (flags, damage_max) = ruleset
        .creature(creature)
        .map(|c| (c.flags, c.damage_max))
        .unwrap_or_default();
    if spell.magic.ignores_immunity {
        return spell.magic.conditions.admits(flags, damage_max);
    }
    if guard.defences.up_to_level.is_some_and(|l| spell.level <= l) {
        return false;
    }
    let abilities = ruleset.abilities(creature);
    let immune = (spell.magic.mind
        && (flags.contains(CreatureFlag::MindImmune) || guard.defences.mind))
        || abilities.iter().any(|a| match *a {
            Ability::SpellLevelImmunity { max_level } => spell.level <= max_level,
            Ability::SchoolImmunity { school } => spell.schools.contains(&school),
            Ability::SpellImmunity { spell: s, .. } => s == id,
            _ => false,
        });
    // An immunity yields only to a negation, and an absolute one not even to that.
    spell.magic.conditions.admits(flags, damage_max)
        && (!immune
            || (guard.negated
                && !abilities.iter().any(
                    |a| matches!(*a, Ability::SpellImmunity { spell: s, absolute: true } if s == id),
                )))
}

/// Whether Anti-Magic on `stack` keeps `spell` off it (BAT-MAG-021): a spell below the level it
/// bounds, whatever its polarity, unless the spell is a creature ability or ignores immunity
/// (Dispel; the original tests both before Anti-Magic, `installation.md` §7.13).
// rule: BAT-MAG-021
pub(crate) fn kept_off(stack: &Stack, spell: &BattleSpell) -> bool {
    spell.level < stack.spells.spell_immunity
        && spell.category != SpellCategory::Ability
        && !spell.magic.ignores_immunity
}

/// Whether a spell of `caster`'s side can affect the living stack `s` at all, wherever the caster
/// stands (BAT-MAG-005, BAT-MAG-021, BAT-MAG-062): the battle's spell level limit does not bar
/// the spell, its creature meets the spell's conditions and immunities with the heroes' part
/// ([`Guard`]), Anti-Magic does not keep the spell off it, and it is no clone if the spell
/// makes its stacks immune to spells (Anti-Magic itself: whoever casts, a hero or a stack that
/// draws its spell; the original's effect test 0x5A8090 has the clone test in the spell's own
/// case, `installation.md` §7.13), and no turret (BAT-SIE-019).
pub(crate) fn affects_stack(
    state: &BattleState,
    ruleset: &BattleRuleset,
    caster: BattleSide,
    spell: SpellRef<'_>,
    s: StackId,
) -> bool {
    let stack = state.at(s);
    stack.is_alive()
        // rule: BAT-SIE-019
        // A turret is immune to every spell (the original's 0x44A577).
        && !stack.is_turret()
        && !barred(state, spell.1)
        && affects(ruleset, spell, stack.creature, Guard::of(state, caster, s))
        && !kept_off(stack, spell.1)
        // rule: BAT-MAG-021
        && !(stack.has(StackTraits::CLONE) && grants_immunity(spell.1))
}

/// Whether `side`'s cast of `spell` at `mastery` may reach stack `s` (BAT-MAG-003,
/// BAT-MAG-005): a living stack on the side the spell may reach at that mastery (its own side,
/// whoever controls it: the original's side tests, `installation.md` §7.14) that the spell can
/// affect and [`admits`], unless a mass version passes over it hypnotized
/// ([`skips_hypnotized`]).
#[inline(always)]
fn reachable(
    state: &BattleState,
    ruleset: &BattleRuleset,
    side: BattleSide,
    (spell, mastery): (SpellRef<'_>, u8),
    s: StackId,
) -> bool {
    // Hypnotized stacks are rare: test them only when they apply.
    in_reach(state, ruleset, side, (spell, mastery), s)
        && !(state.at(s).spells.status.contains(Status::HYPNOTIZED)
            && skips_hypnotized(state, ruleset, (spell, mastery), s))
}

/// [`reachable`], counting the hypnotized stacks a mass version passes over.
#[inline(always)]
fn in_reach(
    state: &BattleState,
    ruleset: &BattleRuleset,
    side: BattleSide,
    (spell, mastery): (SpellRef<'_>, u8),
    s: StackId,
) -> bool {
    let stack = state.at(s);
    let side_ok = match spell.1.magic.side_at(mastery) {
        TargetSide::Any => true,
        TargetSide::Enemy => stack.side != side,
        TargetSide::Own => stack.side == side,
    };
    // The spell's own tests are rare: test them only when they apply.
    let own_tests = spell.1.magic.health_limit || !spell.1.magic.refuses.is_empty();
    side_ok
        && affects_stack(state, ruleset, side, spell, s)
        && (!own_tests || admits(state, ruleset, side, (spell, mastery), s))
}

/// Whether `side`'s cast of `spell` at `mastery` admits stack `s` by the spell's own tests
/// (BAT-MAG-026, `installation.md` §7.14): no active effect of a change the spell refuses
/// (Hypnotize: Magic Mirror), and with a health limit, full hit points (creatures × hit points
/// per creature, the top creature's wounds not counted) at most the cast's value with the
/// casting hero's specialist bonus for the stack (BAT-MAG-020).
// rule: BAT-MAG-026
fn admits(
    state: &BattleState,
    ruleset: &BattleRuleset,
    side: BattleSide,
    ((id, spell), mastery): (SpellRef<'_>, u8),
    s: StackId,
) -> bool {
    let stack = state.at(s);
    let refused = !spell.magic.refuses.is_empty()
        && state.effects.of(s).iter().any(|e| {
            ruleset.spell(e.spell).is_some_and(|sp| {
                sp.magic
                    .effects
                    .iter()
                    .filter_map(|c| c.timed())
                    .any(|c| spell.magic.refuses.contains(&c))
            })
        });
    let within = !spell.magic.health_limit
        || state.heroes[side.index()].as_ref().is_some_and(|hero| {
            let value = base_value(hero, spell, mastery);
            let own = hero.magic.bonuses.spell(id);
            let limit = value.saturating_add_signed(specialist_bonus(
                ruleset,
                spell,
                own,
                stack.creature,
                value,
            ));
            u64::from(stack.count) * u64::from(stack.max_hp) <= limit
        });
    !refused && within
}

/// Whether the mass version of `spell` at `mastery` passes over stack `s` because it is
/// hypnotized (BAT-MAG-026, `installation.md` §7.14). The original's mass handler of the timed
/// spells (0x5A66D0) and Expert Cure's loop skip hypnotized stacks; Dispel's own loop over both
/// sides' stacks (from 0x5A160F) reads no Hypnotize. So the mass version of a spell that only
/// dispels (Dispel) never does (BAT-MAG-016; E04b T11b review); of the others, with quirk Q-0012
/// every one does, without it those whose dispel does not end the stack's Hypnotize (Cure's
/// does).
fn skips_hypnotized(
    state: &BattleState,
    ruleset: &BattleRuleset,
    ((_, spell), mastery): (SpellRef<'_>, u8),
    s: StackId,
) -> bool {
    let mass = spell.magic.target == SpellTarget::Stack
        && target_at(ruleset, spell, mastery) == SpellTarget::None;
    // rule: BAT-MAG-016
    if !mass || spell.magic.effects == [SpellEffect::Dispel] {
        return false;
    }
    let hypnosis = SpellEffect::Timed(TimedChange::Hypnotize);
    let ends_it = spell.magic.dispels()
        && state.effects.of(s).iter().any(|e| {
            ruleset
                .spell(e.spell)
                .is_some_and(|sp| sp.magic.effects.contains(&hypnosis))
                && effects::dispel_ends(ruleset, spell, e)
        });
    ruleset.quirks().is_enabled(MASS_CURE_SKIPS_HYPNOTIZED) || !ends_it
}

/// Whether `side`'s cast of a stack spell at `mastery` may be aimed at stack `s` (BAT-MAG-003):
/// a stack it may reach, with an active timed effect if the spell needs one (Dispel), whose
/// share left after resistance is above zero (the original's target test, `installation.md`
/// §7.8: a stack that always resists is no target).
fn aimable(
    state: &BattleState,
    ruleset: &BattleRuleset,
    side: BattleSide,
    (spell, mastery): (SpellRef<'_>, u8),
    s: StackId,
) -> bool {
    reachable(state, ruleset, side, (spell, mastery), s) && accepts(state, ruleset, side, spell, s)
}

/// The part of the original's target test beyond a stack's reach (BAT-MAG-003): an active timed
/// effect if `spell` needs one (Dispel), and a chance above zero to be affected
/// ([`has_chance`]).
fn accepts(
    state: &BattleState,
    ruleset: &BattleRuleset,
    side: BattleSide,
    spell: SpellRef<'_>,
    s: StackId,
) -> bool {
    (!spell.1.magic.needs_effect || !state.effects.of(s).is_empty())
        && has_chance(state, ruleset, side, spell.1, s)
}

/// Whether `side`'s cast of `spell` has a chance above zero to affect stack `s`, which it can
/// affect (BAT-MAG-003, BAT-MAG-008; the original's factor above 0, `installation.md` §7.8):
/// always for a positive spell and for one that ignores immunity (Dispel), else if the share
/// left after resistance and auras is above zero ([`may_affect`]). A stack that can only resist
/// by a roll has one; a stack that always resists has none.
fn has_chance(
    state: &BattleState,
    ruleset: &BattleRuleset,
    side: BattleSide,
    spell: &BattleSpell,
    s: StackId,
) -> bool {
    spell.polarity == SpellPolarity::Positive
        || spell.magic.ignores_immunity
        || may_affect(state, ruleset, side, s)
}

/// Whether the battle can cast `spell` at `mastery` as an opening spell (BAT-MAG-068, checked
/// with the hero, BAT-HER-001): a hero's spell without a target at that mastery (a mass version
/// or a battle-wide spell) whose effects only change, dispel or heal stacks.
pub(crate) fn opens_with(ruleset: &BattleRuleset, spell: &BattleSpell, mastery: u8) -> bool {
    spell.category != SpellCategory::Ability
        && implemented(spell)
        && target_at(ruleset, spell, mastery) == SpellTarget::None
        && spell.magic.effects.iter().all(|e| {
            matches!(
                e,
                SpellEffect::Timed(_) | SpellEffect::Dispel | SpellEffect::Heal
            )
        })
}

/// The opening spells of the side controlling stack `id` at its activation (BAT-MAG-068; the
/// original's 0x4653A8, `installation.md` §7.24): at the side's first activation outside the
/// tactics phase (the battle's activations; the tactics phase has its own), its hero casts each
/// of its opening spells in their order if it affects some stack (0x5A40D0: some living stack
/// the target test accepts, hypnotized ones counted), as a mass cast of the hero at the spell's
/// mastery (Expert where the battlefield raises the casts that are not the hero's own,
/// BAT-TER-005) and power (caster kind 2): no mana, not the round's cast, no Eagle Eye, the power
/// its effects' duration without the duration bonus, the hero's specialty applied; each stack is
/// tested and rolls as for any of the hero's mass casts.
// rule: BAT-MAG-068
pub(crate) fn opening<S: Sink>(
    state: &mut BattleState,
    ruleset: &BattleRuleset,
    id: StackId,
    sink: &mut S,
) {
    let side = state.at(id).controller();
    let Some(hero) = state.heroes[side.index()]
        .as_mut()
        .filter(|h| h.magic.opening_due)
    else {
        return;
    };
    hero.magic.opening_due = false;
    let (openings, bonuses) = (hero.magic.opening, hero.magic.bonuses);
    // Validated with the hero (BAT-HER-001): every spell is the ruleset's.
    for (o, spell) in openings
        .iter()
        .filter_map(|o| ruleset.spell(o.spell).map(|s| (o, s)))
    {
        let pair = (o.spell, spell);
        // rule: BAT-TER-005
        let mastery = other_mastery(state, o.mastery);
        let affects_some = stack_ids(state).any(|s| {
            in_reach(state, ruleset, side, (pair, mastery), s)
                && accepts(state, ruleset, side, pair, s)
        });
        if !affects_some {
            continue;
        }
        sink.emit(Event::public(EventKind::OpeningSpell {
            side,
            spell: o.spell,
        }));
        let cast = Cast {
            side,
            id: o.spell,
            spell,
            cost: 0,
            mastery,
            target: CastTarget::None,
            second: CastTarget::None,
            // rule: BAT-MAG-004
            value: u64::from(o.power) * u64::from(spell.power)
                + u64::from(spell.effect[usize::from(mastery)]),
            bonuses,
            rounds: o.power,
            power: o.power,
            by: None,
            draws: false,
            automatic: false,
        };
        take_effect(state, ruleset, &cast, sink);
    }
}

/// Eagle Eye at a cast of `spell` by the hero of `caster` (BAT-MAG-069; the original's 0x59FF52,
/// `installation.md` §7.24): the other side's hero picks the spell if its Eagle Eye may (the
/// spell not learned, of a level its Eagle Eye allows: [`HeroMagic::eagle_spells`]) and a draw
/// from 1 to 100 is at most its chance, before the cast's own draws; a spell is kept once, while
/// each cast draws again. Nothing shows in the battle (the outcome reports the picks).
// rule: BAT-MAG-069
fn eagle_eye(state: &mut BattleState, caster: BattleSide, spell: SpellId) {
    let learner = caster.opponent().index();
    let Some(chance) = state.heroes[learner]
        .as_ref()
        .filter(|h| h.magic.eagle_spells.contains(spell))
        .map(|h| u32::from(h.magic.eagle_chance))
    else {
        return;
    };
    let picked = match chance {
        0 => false,
        100.. => true,
        _ => state.rng.below(100) < chance,
    };
    if picked && let Some(hero) = state.heroes[learner].as_mut() {
        hero.magic.learned.insert(spell);
    }
}

/// The living stacks of `side` but its turrets, by id: those a Magic Mirror may send a spell to
/// (BAT-MAG-022; the original leaves out arrow towers, `installation.md` §7.13; BAT-SIE-019).
fn living(state: &BattleState, side: BattleSide) -> impl Iterator<Item = StackId> + '_ {
    stack_ids(state).filter(move |&s| {
        let stack = state.at(s);
        stack.side == side && stack.is_alive() && !stack.is_turret()
    })
}

/// Every stack id of the battle.
fn stack_ids(state: &BattleState) -> impl Iterator<Item = StackId> + use<> {
    (0u8..).take(state.stacks.len()).map(StackId)
}

/// The distances from an area's centre that the area of `spell` at `mastery` covers: from
/// `inner` to the radius (BAT-MAG-007; a hex spell without an area covers its hex).
fn area(spell: &BattleSpell, mastery: u8) -> (u32, u32) {
    spell.magic.area.map_or((0, 0), |a| {
        (
            u32::from(a.inner),
            u32::from(a.radius[usize::from(mastery)]),
        )
    })
}

/// Whether a footprint has a hex at a distance in `(inner, radius)` from `centre`.
fn in_area(field: Field, centre: Hex, (inner, radius): (u32, u32), footprint: Footprint) -> bool {
    footprint.hexes().any(|h| {
        let d = field.distance(centre, h);
        inner <= d && d <= radius
    })
}

/// The stacks `side`'s cast of `spell` at `mastery` aimed at `hex` reaches, by id (BAT-MAG-007).
fn area_stacks<'a>(
    state: &'a BattleState,
    ruleset: &'a BattleRuleset,
    side: BattleSide,
    (spell, mastery): (SpellRef<'a>, u8),
    hex: Hex,
) -> impl Iterator<Item = StackId> + 'a {
    let field = Field::new(ruleset.battle().field);
    let range = area(spell.1, mastery);
    // The area first: it is the cheaper test.
    stack_ids(state).filter(move |&s| {
        in_area(field, hex, range, state.at(s).footprint(field))
            && reachable(state, ruleset, side, (spell, mastery), s)
    })
}

/// How `side`'s cast of the stack spell `spell` at `mastery` picks its stacks: `None` for a spell
/// that raises no creatures ([`aimable`]), else its raise test ([`raise::test_for`]; `None` inside
/// without a hero).
fn raise_test<'r>(
    state: &BattleState,
    ruleset: &'r BattleRuleset,
    side: BattleSide,
    (spell, mastery): (SpellRef<'r>, u8),
) -> Option<Option<raise::RaiseTest<'r>>> {
    spell.1.magic.effects.iter().any(|e| e.is_raise()).then(|| {
        state.heroes[side.index()]
            .as_ref()
            .map(|hero| raise::test_for(state, hero, ruleset, (spell, mastery)))
    })
}

/// Whether `spell` makes its stacks immune to spells (Anti-Magic): no clone takes it
/// ([`affects_stack`], BAT-MAG-021; the original's 0x5A8090, `installation.md` §7.13).
fn grants_immunity(spell: &BattleSpell) -> bool {
    spell
        .magic
        .effects
        .contains(&SpellEffect::Timed(TimedChange::SpellImmunity))
}

/// Whether `side`'s cast of the stack spell `spell` at `mastery`, whose [`raise_test`] is `raise`,
/// may be aimed at stack `s`: one of [`aimable`] (for Anti-Magic no clone, BAT-MAG-021), for
/// Clone also [`created::clonable`] (BAT-MAG-038), or for a spell that raises creatures one of
/// [`raise::raisable`] (BAT-MAG-035, BAT-MAG-036), dead stacks included.
fn stack_target(
    state: &BattleState,
    ruleset: &BattleRuleset,
    side: BattleSide,
    ((spell, mastery), raise): ((SpellRef<'_>, u8), Option<Option<raise::RaiseTest<'_>>>),
    s: StackId,
) -> bool {
    match raise {
        // rule: BAT-MAG-035
        Some(test) => test.is_some_and(|test| raise::raisable(state, ruleset, side, test, s)),
        None => {
            aimable(state, ruleset, side, (spell, mastery), s)
                // rule: BAT-MAG-038
                && (!spell.1.magic.clones()
                    || created::clonable(state, ruleset, side, (spell.1, mastery), s))
        }
    }
}

/// Whether `target` is a first target of `side`'s cast of `spell` at `mastery` (BAT-MAG-003): a
/// stack of [`stack_target`] for a stack spell; for a hex spell any hex outside the side columns,
/// whatever stands on it (the original's cursor test, `installation.md` §7.8; batch 2a R3),
/// but for an obstacle spell a hex where its shape fits hexes free as the side sees them
/// (BAT-MAG-027, BAT-MAG-031); nothing for the spells of [`no_target`]: a battle-wide spell or a
/// mass version, an obstacle spell of a random shape, a summon. Whether a second target exists
/// is not checked here.
// rule: BAT-MAG-003
fn first_target(
    state: &BattleState,
    ruleset: &BattleRuleset,
    side: BattleSide,
    (spell, mastery): (SpellRef<'_>, u8),
    target: CastTarget,
) -> bool {
    match (target_at(ruleset, spell.1, mastery), target) {
        (SpellTarget::Stack, CastTarget::Stack(s)) => {
            let raise = raise_test(state, ruleset, side, (spell, mastery));
            state.stack(s).is_some()
                && stack_target(state, ruleset, side, ((spell, mastery), raise), s)
        }
        (SpellTarget::Hex, CastTarget::Hex(h)) if spell.1.magic.removes_obstacles() => {
            // rule: BAT-MAG-034
            obstacles::removable(state, side, (&spell.1.magic, mastery), h)
        }
        (SpellTarget::Hex, CastTarget::Hex(h)) => {
            let field = Field::new(ruleset.battle().field);
            match &spell.1.magic.obstacle {
                Some(shape) => {
                    // rule: BAT-MAG-027
                    obstacles::shape_hexes(field, shape, (side, mastery), h)
                        .is_some_and(|hexes| obstacles::free(state, field, side, hexes.as_slice()))
                }
                None => field.is_standable(h),
            }
        }
        (SpellTarget::None, CastTarget::None) => no_target(state, ruleset, side, (spell, mastery)),
        _ => false,
    }
}

/// Whether `second` is the second target of `side`'s cast of `spell` at `mastery` aimed at
/// `target` (BAT-MAG-029): for Teleport a hex, other than the stack's position, where the stack
/// fits (inside the field, outside the side columns, not blocked by an obstacle or another
/// living stack; the original's 0x5A3700, `installation.md` §7.15), at a town the closed gate
/// passing as for the caster's side (0x469A10 reads the acting side) and only across the moat
/// and the walls as the mastery allows (BAT-SIE-026); for Sacrifice a victim other than
/// the target ([`raise::victim`], BAT-MAG-036); none for every other spell.
// rule: BAT-MAG-029
fn second_target(
    state: &BattleState,
    ruleset: &BattleRuleset,
    side: BattleSide,
    (spell, mastery): (SpellRef<'_>, u8),
    (target, second): (CastTarget, CastTarget),
) -> bool {
    let magic = &spell.1.magic;
    match (magic.teleports(), magic.sacrifices(), target, second) {
        (true, _, CastTarget::Stack(s), CastTarget::Hex(h)) => {
            let field = Field::new(ruleset.battle().field);
            let stack = state.at(s);
            h != stack.hex
                && movement::fits(field, stack, h, state.blocked_as(field, s, Some(side)))
                && crate::siege::teleport_allowed(state, ruleset, (stack.hex, h), mastery)
        }
        // rule: BAT-MAG-036
        (_, true, CastTarget::Stack(s), CastTarget::Stack(v)) => {
            v != s && raise::victim(state, ruleset, side, spell, v)
        }
        (false, false, _, CastTarget::None) => true,
        _ => false,
    }
}

/// Checks that `(target, second)` are the targets of `side`'s cast of `spell` at `mastery`
/// ([`first_target`], [`second_target`]).
fn check_target(
    state: &BattleState,
    ruleset: &BattleRuleset,
    side: BattleSide,
    (spell, mastery): (SpellRef<'_>, u8),
    (target, second): (CastTarget, CastTarget),
) -> Result<(), CommandError> {
    if first_target(state, ruleset, side, (spell, mastery), target)
        && second_target(state, ruleset, side, (spell, mastery), (target, second))
    {
        Ok(())
    } else {
        Err(CommandError::InvalidSpellTarget)
    }
}

/// Whether `side`'s cast of `spell` at `mastery` aimed at `target` has a second target
/// ([`second_target`]): every target of a spell without one does.
fn has_second(
    state: &BattleState,
    ruleset: &BattleRuleset,
    side: BattleSide,
    (spell, mastery): (SpellRef<'_>, u8),
    target: CastTarget,
) -> bool {
    let magic = &spell.1.magic;
    let spell = (spell, mastery);
    if magic.teleports() {
        hex_targets(ruleset).any(|h| second_target(state, ruleset, side, spell, (target, h)))
    } else if magic.sacrifices() {
        stack_ids(state)
            .any(|v| second_target(state, ruleset, side, spell, (target, CastTarget::Stack(v))))
    } else {
        true
    }
}

/// Every hex of the field as a target.
fn hex_targets(ruleset: &BattleRuleset) -> impl Iterator<Item = CastTarget> + use<> {
    (0u8..)
        .take(Field::new(ruleset.battle().field).hex_count())
        .map(|h| CastTarget::Hex(Hex(h)))
}

/// What a cast of `spell` at `mastery` is aimed at (BAT-MAG-003): nothing for its mass version,
/// which starts at the original's earlier mastery with quirk Q-0011 (BAT-MAG-023: Forgetfulness
/// from Advanced), else at the fixed one.
pub(crate) fn target_at(ruleset: &BattleRuleset, spell: &BattleSpell, mastery: u8) -> SpellTarget {
    let early = ruleset.quirks().is_enabled(EARLY_MASS_VERSION);
    spell.magic.target_at(mastery, early)
}

/// `power × power factor + effect[mastery]` of `hero`'s cast (BAT-MAG-004).
pub(crate) fn base_value(hero: &HeroRecord, spell: &BattleSpell, mastery: u8) -> u64 {
    u64::from(hero.view.power) * u64::from(spell.power)
        + u64::from(spell.effect[usize::from(mastery)])
}

/// Validates `side`'s cast of `spell` at `target`; the caller has checked that a stack of
/// `side` awaits a command outside the tactics phase.
pub(crate) fn plan_cast<'r>(
    state: &BattleState,
    ruleset: &'r BattleRuleset,
    side: BattleSide,
    id: SpellId,
    (target, second): (CastTarget, CastTarget),
) -> Result<Cast<'r>, CommandError> {
    let (spell, hero, mastery, cost) = check_spell(state, ruleset, side, id)?;
    check_target(
        state,
        ruleset,
        side,
        ((id, spell), mastery),
        (target, second),
    )?;
    Ok(Cast {
        side,
        id,
        spell,
        cost,
        mastery,
        target,
        second,
        value: base_value(hero, spell, mastery),
        bonuses: hero.magic.bonuses,
        // rule: BAT-MAG-010
        rounds: u16::from(hero.view.power)
            .saturating_add(hero.magic.duration_bonus)
            .max(1),
        power: u16::from(hero.view.power),
        by: None,
        automatic: false,
        draws: false,
    })
}

/// Whether `target` is a target of stack `caster`'s cast of `spell` for `side`, its controller
/// (BAT-MAG-048): a stack other than the caster that the spell may be aimed at ([`first_target`]
/// at the caster's mastery: a living stack the spell may reach and affect); for a drawn spell
/// one of [`drawn_target`] (BAT-MAG-049); for a draw at the cast one of [`helpful_target`]
/// (BAT-MAG-050). The original's test 0x4476C0 and its cursor (0x4766EA, which never offers the
/// caster itself), `installation.md` §7.21.
// rule: BAT-MAG-048
pub(crate) fn creature_target(
    state: &BattleState,
    ruleset: &BattleRuleset,
    side: BattleSide,
    (caster, spell): (StackId, CreatureSpell),
    target: CastTarget,
) -> bool {
    match spell.spell {
        // A caster's spell is one of the ruleset's (the compiler checks its abilities).
        CastSpell::Named(id) => ruleset.spell(id).is_some_and(|s| {
            let cast = ((id, s), spell.mastery);
            if spell.drawn {
                drawn_target(state, ruleset, side, (caster, cast), target)
            } else {
                target != CastTarget::Stack(caster)
                    && first_target(state, ruleset, side, cast, target)
            }
        }),
        CastSpell::AtResolution => helpful_target(state, ruleset, side, caster, target),
        // rule: BAT-MAG-051, BAT-MAG-052
        CastSpell::Raise => casters::raise_target(state, ruleset, side, caster, target),
    }
}

/// Whether `target` is a target of the helpful caster `caster`'s cast for `side`, its controller
/// (BAT-MAG-050; the original's 0x4476C0 case 1 and its cursor 0x4766EA, `installation.md`
/// §7.21): another stack of `side`'s own for which one of its drawn spells is useful
/// ([`casters::useful`]).
// rule: BAT-MAG-050
fn helpful_target(
    state: &BattleState,
    ruleset: &BattleRuleset,
    side: BattleSide,
    caster: StackId,
    target: CastTarget,
) -> bool {
    target.stack().is_some_and(|t| {
        t != caster
            && state.stack(t).is_some_and(|stack| stack.side == side)
            && casters::useful_spells(state, ruleset, side, (caster, t))
                .next()
                .is_some()
    })
}

/// Whether a cast of `spell` for `side` can affect stack `s` with a chance above zero, by the
/// spell's polarity (BAT-MAG-049; the original's 0x5A3C80 → 0x5A8090 above 0,
/// `installation.md` §7.8, §7.21): a living stack the spell can affect (BAT-MAG-005,
/// BAT-MAG-021), an enemy for a negative spell, an own stack for a positive one, either for a
/// neutral one (Chain Lightning and the area spells), whose share left after resistance and
/// auras is above zero unless the spell is positive (BAT-MAG-008).
pub(crate) fn affectable(
    state: &BattleState,
    ruleset: &BattleRuleset,
    side: BattleSide,
    spell: SpellRef<'_>,
    s: StackId,
) -> bool {
    let stack = state.at(s);
    let side_ok = match spell.1.polarity {
        SpellPolarity::Positive => stack.side == side,
        SpellPolarity::Negative => stack.side != side,
        SpellPolarity::Neutral => true,
    };
    side_ok
        && affects_stack(state, ruleset, side, spell, s)
        && (spell.1.polarity == SpellPolarity::Positive || may_affect(state, ruleset, side, s))
}

/// The stacks other than `caster` that a drawn spell for `side` may be aimed at
/// ([`affectable`]), by id (BAT-MAG-049).
fn drawn_stacks<'a>(
    state: &'a BattleState,
    ruleset: &'a BattleRuleset,
    side: BattleSide,
    (caster, spell): (StackId, SpellRef<'a>),
) -> impl Iterator<Item = StackId> + 'a {
    stack_ids(state).filter(move |&s| s != caster && affectable(state, ruleset, side, spell, s))
}

/// Whether `target` is a target of stack `caster`'s cast of the spell it drew, at `mastery`, for
/// `side`, its controller (BAT-MAG-049; the original's 0x4476C0 for the Faerie Dragon and its
/// cursor 0x4766FC, `installation.md` §7.21): a stack other than the caster, of either side, that
/// the spell can affect ([`affectable`]); a spell that takes a hex is aimed at a hex of that
/// stack (never at an empty hex).
// rule: BAT-MAG-049
fn drawn_target(
    state: &BattleState,
    ruleset: &BattleRuleset,
    side: BattleSide,
    (caster, (spell, mastery)): (StackId, (SpellRef<'_>, u8)),
    target: CastTarget,
) -> bool {
    let field = Field::new(ruleset.battle().field);
    match (target_at(ruleset, spell.1, mastery), target) {
        (SpellTarget::Stack, CastTarget::Stack(t)) => {
            state.stack(t).is_some()
                && drawn_stacks(state, ruleset, side, (caster, spell)).any(|s| s == t)
        }
        (SpellTarget::Hex, CastTarget::Hex(h)) => {
            drawn_stacks(state, ruleset, side, (caster, spell))
                .any(|s| state.at(s).footprint(field).hexes().any(|x| x == h))
        }
        _ => false,
    }
}

/// Fills `out` with the targets of stack `caster`'s cast of `spell` for `side` (BAT-MAG-048,
/// BAT-MAG-049): none first when the cast takes no target (a fixed spell its mastery raises to
/// the mass version, BAT-TER-005: the Ogre Mage's Bloodlust on Magic Plains), then stacks by id,
/// or for a drawn spell that takes a hex the hexes of its stacks by index.
pub(crate) fn creature_targets(
    state: &BattleState,
    ruleset: &BattleRuleset,
    side: BattleSide,
    (caster, spell): (StackId, CreatureSpell),
    out: &mut CastTargets,
) {
    // A caster's spell is one of the ruleset's (the compiler checks its abilities).
    let area = match spell.spell {
        CastSpell::Named(id) => ruleset
            .spell(id)
            .filter(|s| spell.drawn && target_at(ruleset, s, spell.mastery) == SpellTarget::Hex)
            .map(|s| (id, s)),
        CastSpell::AtResolution | CastSpell::Raise => None,
    };
    let field = Field::new(ruleset.battle().field);
    if let Some(drawn) = area {
        let mut hexes = HexSet::new();
        for t in drawn_stacks(state, ruleset, side, (caster, drawn)) {
            for h in state.at(t).footprint(field).hexes() {
                hexes.insert(h);
            }
        }
        for h in hexes.iter() {
            // At most `MAX_FIELD_HEXES` targets.
            let _ = out.push(CastTarget::Hex(h));
        }
        return;
    }
    // A cast without a target takes no stack (`first_target`): at most `STACK_CAPACITY` targets.
    let stacks = stack_ids(state).map(CastTarget::Stack);
    for target in core::iter::once(CastTarget::None).chain(stacks) {
        if creature_target(state, ruleset, side, (caster, spell), target) {
            let _ = out.push(target);
        }
    }
}

/// Validates stack `caster`'s cast of its `spell` at `target` for `side`, its controller
/// (BAT-MAG-048, BAT-MAG-049; the caller has checked that it may cast now): a creature's cast at
/// its ability's mastery and power (a random caster's: per creature, BAT-MAG-049), without a
/// hero's bonuses or cost; its timed effects last `power` rounds.
pub(crate) fn plan_creature_cast<'r>(
    state: &BattleState,
    ruleset: &'r BattleRuleset,
    side: BattleSide,
    (caster, spell): (StackId, CreatureSpell),
    target: CastTarget,
) -> Result<Cast<'r>, CommandError> {
    // A draw at the cast first takes the first spell useful for the stack (BAT-MAG-050).
    let named = match spell.spell {
        CastSpell::Named(id) => Some(id),
        // A raise is planned by `casters::plan`.
        CastSpell::AtResolution | CastSpell::Raise => target
            .stack()
            .filter(|&t| state.stack(t).is_some())
            .and_then(|t| casters::useful_spells(state, ruleset, side, (caster, t)).next())
            .map(|(id, _)| id),
    };
    let (id, s) = named
        .filter(|_| creature_target(state, ruleset, side, (caster, spell), target))
        .and_then(|id| ruleset.spell(id).map(|s| (id, s)))
        .ok_or(CommandError::InvalidSpellTarget)?;
    let power = u16::try_from(spell.power).unwrap_or(u16::MAX);
    Ok(Cast {
        side,
        id,
        spell: s,
        cost: 0,
        mastery: spell.mastery,
        target,
        second: CastTarget::None,
        // rule: BAT-MAG-004
        value: u64::from(spell.power) * u64::from(s.power)
            + u64::from(s.effect[usize::from(spell.mastery)]),
        bonuses: SpellBonuses::default(),
        rounds: power,
        power,
        by: Some(caster),
        draws: spell.spell == CastSpell::AtResolution,
        automatic: false,
    })
}

/// The cast of the periodic caster `caster` for `side`, its controller (BAT-MAG-053; the
/// original's 0x59FE30 as caster kind 1 with no hex, `installation.md` §7.21): `spell` at
/// `mastery` with `power` and no target (at Expert the mass version), without hero bonuses, its
/// timed effects lasting `power` rounds; no cast is used.
pub(crate) fn periodic_cast<'r>(
    side: BattleSide,
    (caster, (id, spell)): (StackId, SpellRef<'r>),
    (mastery, power): (u8, u8),
) -> Cast<'r> {
    Cast {
        side,
        id,
        spell,
        cost: 0,
        mastery,
        target: CastTarget::None,
        second: CastTarget::None,
        // rule: BAT-MAG-004
        value: u64::from(power) * u64::from(spell.power)
            + u64::from(spell.effect[usize::from(mastery)]),
        bonuses: SpellBonuses::default(),
        rounds: u16::from(power),
        power: u16::from(power),
        by: Some(caster),
        draws: false,
        automatic: true,
    }
}

/// The cast of a helpful caster (BAT-MAG-050; the original's 0x447EE0, `installation.md` §7.21):
/// the spell drawn among its creature's drawn spells useful for the target stack
/// ([`casters::useful`]), each with the chance of its weight (the Master Genie's are all 1: the
/// original's `random(1, n)` over them in spell order), at the caster's mastery and power.
// rule: BAT-MAG-050
fn draw_useful<'r>(
    state: &mut BattleState,
    ruleset: &'r BattleRuleset,
    cast: Cast<'r>,
) -> Cast<'r> {
    // Validated: a stack's cast at a stack with a useful spell.
    let pair = (
        cast.by.unwrap_or_default(),
        cast.target.stack().unwrap_or_default(),
    );
    let total: u32 = casters::useful_spells(state, ruleset, cast.side, pair)
        .map(|(_, w)| w)
        .sum();
    let mut r = state.rng.below(total);
    let id = casters::useful_spells(state, ruleset, cast.side, pair)
        .find(|&(_, w)| {
            let hit = r < w;
            r = r.saturating_sub(w);
            hit
        })
        .map_or(cast.id, |(id, _)| id);
    let spell = ruleset.spell(id).unwrap_or(cast.spell);
    Cast {
        id,
        spell,
        // rule: BAT-MAG-004
        value: u64::from(cast.power) * u64::from(spell.power)
            + u64::from(spell.effect[usize::from(cast.mastery)]),
        ..cast
    }
}

/// Whether a stack of `actor` awaits a command outside the tactics phase (when a hero may cast).
fn may_cast_now(state: &BattleState, actor: BattleSide) -> bool {
    state.tactics.is_none() && state.side_to_act() == Some(actor)
}

/// Fills `out` with every first target [`crate::apply`] accepts for `actor`'s hero casting
/// `spell` now (BAT-MAG-001 to BAT-MAG-003) with some second target ([`second_targets`]):
/// stacks by id, hexes by index, or none. Empty if the hero may not cast the spell now.
pub fn cast_targets(
    state: &BattleState,
    ruleset: &BattleRuleset,
    actor: BattleSide,
    spell: SpellId,
    out: &mut CastTargets,
) {
    out.clear();
    if !may_cast_now(state, actor) {
        return;
    }
    let Ok((s, _, mastery, _)) = check_spell(state, ruleset, actor, spell) else {
        return;
    };
    let stacks = stack_ids(state).map(CastTarget::Stack);
    for target in core::iter::once(CastTarget::None)
        .chain(stacks)
        .chain(hex_targets(ruleset))
    {
        if first_target(state, ruleset, actor, ((spell, s), mastery), target)
            && has_second(state, ruleset, actor, ((spell, s), mastery), target)
        {
            // At most `MAX_CAST_TARGETS` targets by construction.
            let _ = out.push(target);
        }
    }
}

/// Fills `out` with every second target [`crate::apply`] accepts for `actor`'s hero casting
/// `spell` at `target` now (BAT-MAG-029): Teleport's destination hexes, Sacrifice's victims
/// (BAT-MAG-036), or only none for a spell without a second target. Empty if `target` is no
/// target of the cast now.
pub fn second_targets(
    state: &BattleState,
    ruleset: &BattleRuleset,
    actor: BattleSide,
    (spell, target): (SpellId, CastTarget),
    out: &mut CastTargets,
) {
    out.clear();
    if !may_cast_now(state, actor) {
        return;
    }
    let Ok((s, _, mastery, _)) = check_spell(state, ruleset, actor, spell) else {
        return;
    };
    if !first_target(state, ruleset, actor, ((spell, s), mastery), target) {
        return;
    }
    // Only Teleport has hexes and only Sacrifice stacks as second targets (`second_target`).
    let hexes = hex_targets(ruleset).take(if s.magic.teleports() { usize::MAX } else { 0 });
    let stacks = stack_ids(state)
        .map(CastTarget::Stack)
        .take(if s.magic.sacrifices() { usize::MAX } else { 0 });
    for second in core::iter::once(CastTarget::None)
        .chain(hexes)
        .chain(stacks)
    {
        if second_target(
            state,
            ruleset,
            actor,
            ((spell, s), mastery),
            (target, second),
        ) {
            // At most `MAX_CAST_TARGETS` targets by construction.
            let _ = out.push(second);
        }
    }
}

/// Whether `actor`'s cast of `spell` has a target ([`cast_targets`] not empty, computed without
/// listing every hex where it can be avoided): a stack spell needs a stack of [`stack_target`] with a
/// second target, a spell without a target what [`no_target`] asks; a hex spell always has one, but
/// an obstacle spell needs a hex where its shape fits and a remove spell an obstacle it removes.
fn has_target(
    state: &BattleState,
    ruleset: &BattleRuleset,
    actor: BattleSide,
    (spell, mastery): (SpellRef<'_>, u8),
) -> bool {
    match target_at(ruleset, spell.1, mastery) {
        SpellTarget::Hex if spell.1.magic.removes_obstacles() => {
            obstacles::any_removable(state, actor, (&spell.1.magic, mastery))
        }
        SpellTarget::Hex if spell.1.magic.obstacle.is_some() => {
            hex_targets(ruleset).any(|h| first_target(state, ruleset, actor, (spell, mastery), h))
        }
        SpellTarget::Hex => true,
        SpellTarget::Stack => {
            let raise = raise_test(state, ruleset, actor, (spell, mastery));
            stack_ids(state).any(|s| {
                stack_target(state, ruleset, actor, ((spell, mastery), raise), s)
                    && has_second(
                        state,
                        ruleset,
                        actor,
                        (spell, mastery),
                        CastTarget::Stack(s),
                    )
            })
        }
        SpellTarget::None => no_target(state, ruleset, actor, (spell, mastery)),
    }
}

/// Whether `side` may cast `spell` at `mastery` without a target now (BAT-MAG-003): a summon
/// while the side may summon its creature (BAT-MAG-039), an obstacle spell of a random shape
/// whether or not a hex is free (BAT-MAG-032) but mines not while the other side's natives would
/// see them (BAT-MAG-033), any other (a battle-wide spell, a mass version from the spell's mass
/// mastery) if it would affect some stack: a stack in its reach with a chance above zero to be
/// affected ([`has_chance`]; one that always resists does not count). The original's book asks
/// its target test of every hex before anything is paid and refuses the spell when no stack
/// passes (0x59ED5A to 0x59EE03 with the factor of 0x5A39C0; E04b T11b review). That test reads
/// a stack's Hypnotize only for the hero that controls it, so the hypnotized stacks a mass
/// version passes over count here ([`in_reach`]): with no other stack the cast is paid for and
/// reaches nobody. A stack with no active effect counts for the mass version of a spell that
/// needs one when it is aimed (Expert Dispel: the original asks for an active spell below
/// Expert only, BAT-MAG-016).
fn no_target(
    state: &BattleState,
    ruleset: &BattleRuleset,
    side: BattleSide,
    (spell, mastery): (SpellRef<'_>, u8),
) -> bool {
    if spell.1.magic.summons() {
        // rule: BAT-MAG-039
        return created::may_summon(state, ruleset, side, spell.1);
    }
    if spell.1.magic.quakes() {
        // rule: BAT-SIE-015
        return crate::siege::may_quake(state);
    }
    // rule: BAT-MAG-032
    spell.1.magic.obstacle_kind().map_or_else(
        || {
            // rule: BAT-MAG-003
            stack_ids(state).any(|s| {
                in_reach(state, ruleset, side, (spell, mastery), s)
                    && has_chance(state, ruleset, side, spell.1, s)
            })
        },
        |kind| obstacles::may_place(state, side, kind),
    )
}

/// The spells `actor`'s hero may cast now: those with at least one target in
/// [`cast_targets`] (empty if it is not `actor`'s turn, in the tactics phase or without a hero).
pub fn castable_spells(
    state: &BattleState,
    ruleset: &BattleRuleset,
    actor: BattleSide,
) -> SpellSet {
    let mut out = SpellSet::default();
    let Some(hero) = state.heroes[actor.index()].as_ref() else {
        return out;
    };
    if !may_cast_now(state, actor) {
        return out;
    }
    for spell in hero.magic.book.iter() {
        if let Ok((s, _, mastery, _)) = check_spell(state, ruleset, actor, spell)
            && has_target(state, ruleset, actor, ((spell, s), mastery))
        {
            out.insert(spell);
        }
    }
    out
}

/// `value` grown by `bp` basis points of itself, rounded down (saturating).
fn grow(value: u64, bp: u64) -> u64 {
    mul_div_floor(value, 10_000 + bp, 10_000).unwrap_or(u64::MAX)
}

/// The damage of `spell` with the value `value` on a stack of `creature` with the school
/// protections `protection` (BAT-MAG-004) after the caster's bonuses and after the target's
/// extra damage or reduction and protection, in the original's order (`installation.md` §7.8):
/// the bonuses of the caster's hero `hero` ([`caster_stage`]; none without a hero: the original
/// runs that stage on the hero only, 0x5A78E0), then the target's (one spell's extra damage,
/// rounded down; the spell damage reduction, rounded down; one protection, [`protected`]). The
/// rates are exact, or single precision with quirk Q-0005. Saturates.
// rule: BAT-MAG-004
pub(crate) fn spell_damage(
    ruleset: &BattleRuleset,
    (spell, hero): (SpellRef<'_>, Option<&SpellBonuses>),
    value: u64,
    (creature, protection): (CreatureTypeId, &[u8; MAX_SPELL_SCHOOLS]),
) -> (u64, u64) {
    let caster = hero.map_or(value, |b| {
        caster_stage(ruleset, (spell, b), value, creature)
    });
    let mut damage = caster;
    for ability in ruleset.abilities(creature) {
        match *ability {
            Ability::SpellVulnerability { spell: s, percent } if s == spell.0 => {
                damage = grow(damage, u64::from(percent) * 100);
            }
            Ability::SpellDamageReduction { percent } => {
                damage = damage.saturating_mul(100 - u64::from(percent.min(100))) / 100;
            }
            _ => {}
        }
    }
    (caster, protected(ruleset, spell.1, damage, protection))
}

/// The value `value` of a damage spell after the bonuses `b` of its caster's hero, on a stack of
/// `creature` (BAT-MAG-004; the original's 0x4E5760, `installation.md` §7.8 step 1): × (1 + the
/// best Orb among the spell's schools) × (1 + Sorcery), then the spell's own bonus (a share,
/// rounded down) and a specialist's ([`specialist_bonus`]), both of the product's whole part.
///
/// With exact rates the product is kept fractional and rounded down once, and the bonuses are
/// added to it. With quirk Q-0005 the executable's arithmetic is followed: it keeps the value
/// as a single-precision number, so the value, its product with the Orb's factor and that one's
/// with Sorcery's are each stored as an `f32` (rounded to 24 bits, to nearest: 57.99999982
/// becomes 58), the bonuses of the last product's whole part are added to the stored product
/// itself, and only that sum, stored as an `f32` once more, is rounded down. Saturates for any
/// value and any rates.
// rule: BAT-MAG-004
fn caster_stage(
    ruleset: &BattleRuleset,
    ((id, spell), b): (SpellRef<'_>, &SpellBonuses),
    value: u64,
    creature: CreatureTypeId,
) -> u64 {
    let (single, school, all) = caster_rates(ruleset, spell, b);
    let own = b.spell(id);
    // The specialist's bonus (it may lower a value) and the spell's own, below 2^96, of `t`.
    let bonuses = |t: u64| {
        let fixed = u128::from(t) * u128::from(own.damage_bp) / u128::from(FULL_BP);
        (specialist_bonus(ruleset, spell, own, creature, t), fixed)
    };
    if single {
        let stored = |v: X87| v.round(SINGLE_BITS);
        let v = stored(X87::int(value));
        let v = stored(v.times(X87::ONE.plus(X87::fixed(school))));
        let v = stored(X87::ONE.plus(X87::fixed(all)).times(v));
        let (specialty, fixed) = bonuses(v.trunc());
        // One whole number, as the original's routine returns it, loaded and stored as an
        // `f32`: what it adds and what it takes (one of the two is 0; never more than `t`).
        let bonus = i128::from(specialty).saturating_add_unsigned(fixed);
        let part = |n: i128| stored(X87::int(u64::try_from(n.max(0)).unwrap_or(u64::MAX)));
        stored(v.plus(part(bonus)).minus(part(-bonus))).trunc()
    } else {
        let one = u128::from(FULL_BP);
        // Below 2^65: a rate is at most `u32::MAX` basis points.
        let factor = (one + u128::from(school)) * (one + u128::from(all));
        // A product beyond `u128` is a quotient beyond `u64`.
        let t = u128::from(value).checked_mul(factor).map_or(u64::MAX, |p| {
            u64::try_from(p / (one * one)).unwrap_or(u64::MAX)
        });
        let (specialty, fixed) = bonuses(t);
        u64::try_from(u128::from(t.saturating_add_signed(specialty)) + fixed).unwrap_or(u64::MAX)
    }
}

/// Whether the rates are single precision (quirk Q-0005), the best Orb's rate among `spell`'s
/// schools and Sorcery's, of the bonuses `b` ([`caster_stage`], [`untargeted_bonus`]).
fn caster_rates(
    ruleset: &BattleRuleset,
    spell: &BattleSpell,
    b: &SpellBonuses,
) -> (bool, u64, u64) {
    let single = ruleset.quirks().is_enabled(SINGLE_PRECISION_RATES);
    let school = spell
        .schools
        .iter()
        .map(|l| hero_rate(single, b.school[l.index().min(MAX_SPELL_SCHOOLS - 1)]))
        .max()
        .unwrap_or(0);
    (single, school, hero_rate(single, b.all))
}

/// The value `value` of `spell` after the bonuses `b` of its caster's hero without a target stack
/// (the original's 0x4E5760 given no stack, `installation.md` §7.8 step 1; the computer's odds,
/// §7.39): × (1 + the best Orb among the spell's schools) × (1 + Sorcery), rounded down once; no
/// single spell's or specialist's bonus, which need a stack. The rates exact, or with quirk
/// Q-0005 each product stored as an `f32`, as [`caster_stage`]. Saturates.
// rule: BAT-AI-008
pub(crate) fn untargeted_bonus(
    ruleset: &BattleRuleset,
    spell: &BattleSpell,
    b: &SpellBonuses,
    value: u64,
) -> u64 {
    let (single, school, all) = caster_rates(ruleset, spell, b);
    if single {
        let stored = |v: X87| v.round(SINGLE_BITS);
        let v = stored(X87::int(value));
        let v = stored(v.times(X87::ONE.plus(X87::fixed(school))));
        stored(X87::ONE.plus(X87::fixed(all)).times(v)).trunc()
    } else {
        let one = u128::from(FULL_BP);
        let factor = (one + u128::from(school)) * (one + u128::from(all));
        u128::from(value).checked_mul(factor).map_or(u64::MAX, |p| {
            u64::try_from(p / (one * one)).unwrap_or(u64::MAX)
        })
    }
}

/// A hero's specialty in a spell, for its casts or effects on one stack (BAT-MAG-020; the
/// original's 0x4E5FF0, `installation.md` §7.12).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Specialist {
    /// The spell's own rule, if it has one.
    rule: Option<SpecialistBonus>,
    /// The hero's growth per level, basis points (the scaled rule).
    per_level_bp: u32,
    /// The hero's level.
    hero_level: u8,
    /// The stack's creature level, at least 1.
    creature_level: u8,
}

impl Specialist {
    /// The specialty of a hero whose bonuses for `spell` are `own`, on a stack of `creature`;
    /// `None` if it has no specialty in the spell.
    pub(crate) fn of(
        ruleset: &BattleRuleset,
        spell: &BattleSpell,
        own: SpellBonus,
        creature: CreatureTypeId,
    ) -> Option<Self> {
        (own.per_level_bp > 0).then(|| Self {
            rule: spell.magic.specialist,
            per_level_bp: own.per_level_bp,
            hero_level: own.level,
            creature_level: ruleset
                .creature_info(creature)
                .map_or(1, |c| c.level)
                .max(1),
        })
    }

    /// The bonus to the value `t`: the spell's own rule (by the creature's level, a fixed
    /// amount, or what makes the value a total), or else the scaled one, `⌈per level × (hero
    /// level div creature level) × t⌉`, the levels divided in whole numbers.
    // rule: BAT-MAG-020
    pub(crate) fn bonus(self, t: u64) -> i64 {
        match self.rule {
            Some(rule) => rule.bonus(self.creature_level, t),
            None => {
                let steps = u128::from(self.hero_level / self.creature_level);
                let bonus = (u128::from(t) * u128::from(self.per_level_bp) * steps)
                    .div_ceil(u128::from(FULL_BP));
                i64::try_from(bonus).unwrap_or(i64::MAX)
            }
        }
    }
}

/// A spell specialist's bonus to the value `t` of `spell` on a stack of `creature`, `own` being
/// the hero's bonuses for the spell: [`Specialist::bonus`], 0 without a specialty in the spell
/// (BAT-MAG-020).
pub(crate) fn specialist_bonus(
    ruleset: &BattleRuleset,
    spell: &BattleSpell,
    own: SpellBonus,
    creature: CreatureTypeId,
    t: u64,
) -> i64 {
    Specialist::of(ruleset, spell, own, creature).map_or(0, |s| s.bonus(t))
}

/// The damage a stack with Fire Shield reflects onto `striker` from a melee strike on the struck
/// stack `target` whose reflected base is `first` (BAT-MAG-018; the strike's first damage stage
/// capped at the target's hit points with quirk Q-0010, else the hit points it lost): none for a
/// fire-immune striker; else `first × v %` rounded down (with quirk Q-0005 in the
/// original's single precision), then the shield's spell damage with the bonuses of the struck
/// stack's hero as the caster's and the striker's reductions and protection
/// ([`spell_damage`]; `installation.md` §7.11). No immunity or resistance applies. Without the
/// spell, or with a smaller value, a creature's own shield reflects alike (the Efreet Sultan's
/// 20 %, BAT-MAG-058; the original's 0x443130 and 0x422440).
// rule: BAT-MAG-018, BAT-MAG-058
pub(crate) fn reflection(
    state: &BattleState,
    ruleset: &BattleRuleset,
    (striker, target): (&Stack, StackId),
    first: u64,
) -> u64 {
    let shield = SpellEffect::Timed(TimedChange::FireShield);
    let fire_immune = ruleset
        .creature(striker.creature)
        .is_some_and(|c| c.flags.contains(CreatureFlag::FireImmune));
    let active = state.effects.of(target).iter().find_map(|e| {
        ruleset
            .spell(e.spell)
            .filter(|spell| spell.magic.effects.contains(&shield))
            .map(|spell| (e.spell, spell, e.value))
    });
    let innate = innate(ruleset, state.at(target).creature, TimedChange::FireShield)
        .map(|(id, spell, value)| (id, spell, u16::from(value)));
    let found = match (active, innate) {
        (Some(a), Some(i)) => Some(if i.2 > a.2 { i } else { a }),
        (a, i) => a.or(i),
    };
    let Some((id, spell, percent)) = found.filter(|_| !fire_immune) else {
        return 0;
    };
    let value = if ruleset.quirks().is_enabled(SINGLE_PRECISION_RATES) {
        let factor = X87::ratio(u32::from(percent), 100, SINGLE_BITS).unwrap_or(X87::ZERO);
        X87::int(first).round(SINGLE_BITS).times(factor).trunc()
    } else {
        mul_div_floor(first, u64::from(percent), 100).unwrap_or(u64::MAX)
    };
    let side = state.at(target).controller();
    let hero = state.heroes[side.index()]
        .as_ref()
        .map(|h| &h.magic.bonuses);
    spell_damage(
        ruleset,
        ((id, spell), hero),
        value,
        (striker.creature, &striker.spells.school_protection),
    )
    .1
}

/// `damage` after the target's protection against `spell` (BAT-MAG-013; `installation.md` §7.8,
/// §7.10): among the spell's schools whose protection the target has, the first by the schools'
/// protection rank with quirk Q-0007 (as the original checks them), else the strongest; the
/// damage times its share, rounded down, as `f32` values with quirk Q-0005.
// rule: BAT-MAG-013
fn protected(
    ruleset: &BattleRuleset,
    spell: &BattleSpell,
    damage: u64,
    protection: &[u8; MAX_SPELL_SCHOOLS],
) -> u64 {
    let rank = |l: &h3_content::SchoolId| {
        ruleset
            .spell_schools()
            .get(l.index())
            .map_or(u8::MAX, |s| s.protection_rank)
    };
    let shares = spell.schools.iter().filter_map(|l| {
        let share = protection[l.index().min(MAX_SPELL_SCHOOLS - 1)];
        (share < 100).then(|| (rank(l), share))
    });
    let chosen = if ruleset.quirks().is_enabled(FIRST_PROTECTION) {
        shares.min_by_key(|&(rank, _)| rank)
    } else {
        shares.min_by_key(|&(_, share)| share)
    };
    let Some((_, share)) = chosen else {
        return damage;
    };
    if ruleset.quirks().is_enabled(SINGLE_PRECISION_RATES) {
        let factor = X87::ratio(u32::from(share), 100, SINGLE_BITS).unwrap_or(X87::ZERO);
        X87::int(damage).round(SINGLE_BITS).times(factor).trunc()
    } else {
        damage.saturating_mul(u64::from(share)) / 100
    }
}

/// The share of a hostile spell of `caster`'s side that stack `s` lets through (BAT-MAG-008) in
/// the fixed point `one`, as `(share, one)`: 1 − its creature's magic resistance minus the
/// resistance of its controller's hero (BAT-MAG-026), at least 0; all of it when the immunities
/// are negated (BAT-MAG-066: the original returns before both). With quirk Q-0005 the values are
/// single precision and the difference is rounded to it, as the original's x87 code computes it
/// (`installation.md` §7.8).
fn share(
    state: &BattleState,
    ruleset: &BattleRuleset,
    caster: BattleSide,
    s: StackId,
) -> (u64, u64) {
    let single = ruleset.quirks().is_enabled(SINGLE_PRECISION_RATES);
    let one = scale(single);
    // rule: BAT-MAG-066
    if Guard::of(state, caster, s).negated {
        return (one, one);
    }
    let stack = state.at(s);
    let creature = ruleset
        .abilities(stack.creature)
        .iter()
        .find_map(|a| match *a {
            Ability::MagicResistance { percent } => Some(percent_rate(single, percent)),
            _ => None,
        })
        .unwrap_or(one);
    let hero = state.heroes[stack.controller().index()]
        .as_ref()
        .map_or(0, |h| h.magic.resistance);
    let share = creature.saturating_sub(hero);
    (if single { round_single(share) } else { share }, one)
}

/// Whether a hostile spell of `caster`'s side has a chance above zero at stack `s` (BAT-MAG-008):
/// its [`share`] is above zero and no aura keeps all of it off; always when the immunities are
/// negated, which lifts the aura too (BAT-MAG-066: the wiki and VCMI against the code reading
/// until an experiment settles it, BAT-Q21).
fn may_affect(
    state: &BattleState,
    ruleset: &BattleRuleset,
    caster: BattleSide,
    s: StackId,
) -> bool {
    Guard::of(state, caster, s).negated
        || (share(state, ruleset, caster, s).0 > 0 && aura(state, ruleset, s) != Some(100))
}

/// 1 − `percent` as a rate in the fixed point of [`scale`].
fn percent_rate(single: bool, percent: u8) -> u64 {
    rate(single, u16::from(100 - percent.min(100)) * 100)
}

/// The strongest magic resistance aura of a living stack adjacent to stack `s` whose controller
/// is `s`'s own side, percent (BAT-MAG-008, BAT-MAG-026; the original's aura lists compare the
/// Unicorn's side through its Hypnotize with the other stack's own side, 0x43EA70): a hypnotized
/// stack keeps its side's aura, hypnotized Unicorns protect the side that commands them.
fn aura(state: &BattleState, ruleset: &BattleRuleset, s: StackId) -> Option<u8> {
    let field = Field::new(ruleset.battle().field);
    let stack = state.at(s);
    let footprint = stack.footprint(field);
    let side = stack.side;
    stack_ids(state)
        .filter(|&n| n != s)
        .map(|n| state.at(n))
        .filter(|n| n.is_alive() && n.controller() == side)
        .filter_map(|n| {
            let percent = ruleset
                .abilities(n.creature)
                .iter()
                .filter_map(|a| match *a {
                    Ability::MagicResistanceAura { percent } => Some(percent),
                    _ => None,
                })
                .max()?;
            field
                .touches(n.footprint(field), footprint)
                .then_some(percent)
        })
        .max()
}

/// The chance in percent that a cast of `spell` by `caster`'s side affects stack `s`
/// (BAT-MAG-008): always for a positive spell, one that ignores immunity (Dispel) and when the
/// immunities are negated (BAT-MAG-066, the aura too: BAT-Q21); otherwise its [`share`] times 1 − the
/// strongest [`aura`], as a whole percentage rounded down (with quirk Q-0005, 10 % hero
/// resistance leaves 89 %).
// rule: BAT-MAG-008
pub(crate) fn affect_chance(
    state: &BattleState,
    ruleset: &BattleRuleset,
    caster: BattleSide,
    spell: &BattleSpell,
    s: StackId,
) -> u32 {
    // rule: BAT-MAG-066
    if spell.polarity == SpellPolarity::Positive
        || spell.magic.ignores_immunity
        || Guard::of(state, caster, s).negated
    {
        return 100;
    }
    let single = ruleset.quirks().is_enabled(SINGLE_PRECISION_RATES);
    let (share, one) = share(state, ruleset, caster, s);
    let aura = aura(state, ruleset, s).map_or(one, |p| percent_rate(single, p));
    let chance = u128::from(share) * u128::from(aura) * 100 / (u128::from(one) * u128::from(one));
    u32::try_from(chance).unwrap_or(100).min(100)
}

/// The factor of the original's effect test 0x5A8090 for `spell` of `caster`'s side at stack `s`
/// as the x87 holds it (BAT-MAG-008; `installation.md` §7.8): 0 when the spell cannot affect the
/// stack ([`affects_stack`]); 1 for a positive spell, one that ignores immunity and when the
/// immunities are negated; else [`share`] times 1 − the strongest [`aura`] (with quirk Q-0005
/// their single-precision values, else the exact rates, each rounded to the double), the product
/// at 53 bits. The computer's estimate of a hero's area spell takes it (0x41F920).
pub(crate) fn effect_factor(
    state: &BattleState,
    ruleset: &BattleRuleset,
    caster: BattleSide,
    spell: SpellRef<'_>,
    s: StackId,
) -> X87 {
    if !affects_stack(state, ruleset, caster, spell, s) {
        return X87::ZERO;
    }
    // rule: BAT-MAG-066
    if spell.1.polarity == SpellPolarity::Positive
        || spell.1.magic.ignores_immunity
        || Guard::of(state, caster, s).negated
    {
        return X87::ONE;
    }
    let single = ruleset.quirks().is_enabled(SINGLE_PRECISION_RATES);
    let (share, _) = share(state, ruleset, caster, s);
    // A rate at the scale: a single-precision numerator, or basis points (at most `FULL_BP`).
    let value = |v: u64| {
        if single {
            X87::fixed(v)
        } else {
            let v = u32::try_from(v).unwrap_or(u32::MAX);
            X87::ratio(v, u32::from(FULL_BP), DOUBLE_BITS).unwrap_or(X87::ZERO)
        }
    };
    let f = value(share);
    aura(state, ruleset, s).map_or(f, |p| f.times(value(percent_rate(single, p))))
}

/// Rolls whether stack `s` resists `spell` (BAT-MAG-008): one draw from 1 to 100 when the chance
/// to be affected is neither 0 nor 100.
pub(crate) fn resists(
    state: &mut BattleState,
    ruleset: &BattleRuleset,
    caster: BattleSide,
    spell: &BattleSpell,
    s: StackId,
) -> bool {
    let chance = affect_chance(state, ruleset, caster, spell, s);
    match chance {
        0 => true,
        100 => false,
        _ => state.rng.below(100) >= chance,
    }
}

/// Whether `cast` is an area spell aimed at a hex or a chain: the spells whose damage the
/// original computes in handlers of their own, which take the acting side's hero's bonuses also
/// for a stack's cast (BAT-MAG-048; `installation.md` §7.21: 0x5A4970, 0x5A6360; Armageddon's
/// handler too, which no creature casts).
fn spreads(cast: &Cast<'_>) -> bool {
    matches!(cast.target, CastTarget::Hex(_)) || cast.spell.magic.chain.is_some()
}

/// Applies `cast`'s effects to stack `s` at the value `value`: its damage, reported, then, if the
/// stack still lives, Cure's restoring of what Poison took (BAT-MAG-044), its dispel
/// (BAT-MAG-016), its heal (BAT-MAG-017) and its timed changes (BAT-MAG-009), in this order (Cure
/// resets the hit points and ends the effects before it heals, `installation.md` §7.10).
fn affect<S: Sink>(
    state: &mut BattleState,
    ruleset: &BattleRuleset,
    cast: &Cast<'_>,
    (s, value): (StackId, u64),
    sink: &mut S,
) {
    if cast.spell.magic.effects.contains(&SpellEffect::Damage) {
        let target = state.at(s);
        let protection = target.spells.school_protection;
        // rule: BAT-MAG-048
        // A hero's cast takes its hero's bonuses. A stack's area, chain or battle-wide spell
        // takes those of its controller's hero: the original's handlers of their own fetch the
        // acting side's hero (0x5A4970, 0x5A6360, `installation.md` §7.21). A stack's spell at
        // one stack takes none (0x5A78E0 without a hero).
        let hero = if cast.by.is_none() {
            Some(&cast.bonuses)
        } else if spreads(cast) {
            state.heroes[cast.side.index()]
                .as_ref()
                .map(|h| &h.magic.bonuses)
        } else {
            None
        };
        let caster_ref = ((cast.id, cast.spell), hero);
        let (caster, damage) =
            spell_damage(ruleset, caster_ref, value, (target.creature, &protection));
        let kills = apply_damage(state.at_mut(s), damage);
        sink.emit(Event::public(EventKind::SpellDamage {
            target: s,
            value,
            caster,
            damage,
            kills,
        }));
        // rule: BAT-MAG-024
        effects::damaged(state, ruleset, s, sink);
    }
    if !state.at(s).is_alive() {
        return;
    }
    // rule: BAT-MAG-044
    if cast.spell.magic.restores_health {
        crate::vitality::restore(state, s);
        effects::recompute(state, ruleset, s);
    }
    if cast.spell.magic.dispels() {
        effects::dispel(state, ruleset, cast.spell, s, sink);
    }
    if cast.spell.magic.effects.contains(&SpellEffect::Heal) {
        // rule: BAT-MAG-017
        let own = cast.bonuses.spell(cast.id);
        let creature = state.at(s).creature;
        // rule: BAT-MAG-020
        let heal = value
            .saturating_add_signed(specialist_bonus(ruleset, cast.spell, own, creature, value));
        let stack = state.at_mut(s);
        let lost = stack.max_hp.saturating_sub(stack.top_hp);
        let healed = u32::try_from(heal).unwrap_or(u32::MAX).min(lost);
        stack.top_hp += healed;
        if healed > 0 {
            sink.emit(Event::public(EventKind::SpellHealed {
                target: s,
                hit_points: healed,
            }));
        }
    }
    if cast.spell.magic.is_timed() {
        let timed = TimedCast {
            id: cast.id,
            spell: cast.spell,
            mastery: cast.mastery,
            rounds: cast.rounds,
            own: cast.bonuses.spell(cast.id),
        };
        effects::apply(state, ruleset, timed, s, sink);
    }
}

/// Reports that stack `s` resisted.
fn resisted<S: Sink>(s: StackId, sink: &mut S) {
    sink.emit(Event::public(EventKind::SpellResisted { target: s }));
}

/// A stack's centre on the original's battle screen (BAT-MAG-006): hexes 44 × 42 pixels, even
/// rows shifted half a hex to the right, a two-hex stack's centre half a hex from its position
/// towards its other hex. (The original also lifts each stack by half its sprite's height, which
/// is not modelled, OPEN-QUESTIONS Q43.)
fn screen_centre(state: &BattleState, field: Field, s: StackId) -> (i64, i64) {
    let stack = state.at(s);
    let (col, row) = field.coords(stack.hex);
    let shift = if row % 2 == 0 { 22 } else { 0 };
    let other = match (stack.is_two_hex(), stack.side) {
        (false, _) => 0,
        (true, BattleSide::Attacker) => 22,
        (true, BattleSide::Defender) => -22,
    };
    (i64::from(44 * col + shift + other), i64::from(42 * row))
}

/// The order in which Chain Lightning looks for its next stack (BAT-MAG-006): the defender's
/// stacks first, then the attacker's.
const CHAIN_ORDER: [BattleSide; 2] = [BattleSide::Defender, BattleSide::Attacker];

/// The next stack Chain Lightning jumps to from `from` (BAT-MAG-006): among the living stacks
/// not yet struck, the defender's first, then the attacker's, each by id, those the spell
/// affects (each rolling for resistance, so a resisting stack is passed over for this jump)
/// and the first at the smallest screen distance, rounded down to whole pixels ([`nearest`]).
// rule: BAT-MAG-006
fn next_in_chain(
    state: &mut BattleState,
    ruleset: &BattleRuleset,
    (caster, spell): (BattleSide, SpellRef<'_>),
    from: StackId,
    struck: &[bool; STACK_CAPACITY],
) -> Option<StackId> {
    let mut found = [false; STACK_CAPACITY];
    for side in CHAIN_ORDER {
        for s in stack_ids(state) {
            let candidate = state.at(s).side == side
                && !struck[s.index()]
                && affects_stack(state, ruleset, caster, spell, s);
            if candidate && !resists(state, ruleset, caster, spell.1, s) {
                found[s.index()] = true;
            }
        }
    }
    nearest(state, ruleset, from, &found)
}

/// The stack of `found` nearest to `from` on the battle screen (BAT-MAG-006): in
/// [`CHAIN_ORDER`], each side's by id, the first at the smallest distance, rounded down to whole
/// pixels.
fn nearest(
    state: &BattleState,
    ruleset: &BattleRuleset,
    from: StackId,
    found: &[bool; STACK_CAPACITY],
) -> Option<StackId> {
    let field = Field::new(ruleset.battle().field);
    let (x, y) = screen_centre(state, field, from);
    let mut best: Option<(u64, StackId)> = None;
    for side in CHAIN_ORDER {
        for s in stack_ids(state) {
            if state.at(s).side != side || !found[s.index()] {
                continue;
            }
            let (cx, cy) = screen_centre(state, field, s);
            let d = ((cx - x).pow(2) + (cy - y).pow(2)).unsigned_abs().isqrt();
            if best.is_none_or(|(b, _)| d < b) {
                best = Some((d, s));
            }
        }
    }
    best.map(|(_, s)| s)
}

/// The stacks a chain spell strikes from its first stack as the computer estimates it, each
/// with the value it takes (0x437190 with 0x5A61F0 not for real, `installation.md` §7.43; E23
/// T4a): the rules' chain (BAT-MAG-006: the strikes at the mastery, each further stack taking
/// its share of the last value) without resistance rolls, each next stack one the spell can
/// affect at all (its effect test's factor above 0).
#[derive(Debug, Clone)]
pub(crate) struct ChainEstimate<'a> {
    state: &'a BattleState,
    ruleset: &'a BattleRuleset,
    side: BattleSide,
    spell: SpellRef<'a>,
    next: Option<(StackId, u64)>,
    left: u8,
    struck: [bool; STACK_CAPACITY],
}

impl<'a> ChainEstimate<'a> {
    /// `side`'s chain of `spell` at `mastery` from stack `first` with the value `value`; none for
    /// a spell without a chain.
    pub(crate) fn new(
        (state, ruleset): (&'a BattleState, &'a BattleRuleset),
        side: BattleSide,
        (spell, mastery): (SpellRef<'a>, u8),
        (first, value): (StackId, u64),
    ) -> Self {
        let left = spell
            .1
            .magic
            .chain
            .map_or(0, |c| c.targets[usize::from(mastery.min(MAX_MASTERY))]);
        Self {
            state,
            ruleset,
            side,
            spell,
            next: Some((first, value)),
            left,
            struck: [false; STACK_CAPACITY],
        }
    }
}

impl Iterator for ChainEstimate<'_> {
    type Item = (StackId, u64);

    fn next(&mut self) -> Option<Self::Item> {
        let (current, value) = self.next.filter(|_| self.left > 0)?;
        self.left -= 1;
        self.struck[current.index()] = true;
        let mut found = [false; STACK_CAPACITY];
        if self.left > 0 {
            for s in stack_ids(self.state) {
                found[s.index()] = !self.struck[s.index()]
                    && effect_factor(self.state, self.ruleset, self.side, self.spell, s)
                        > X87::ZERO;
            }
        }
        let share = self.spell.1.magic.chain.map_or(0, |c| c.factor_bp);
        self.next = nearest(self.state, self.ruleset, current, &found)
            .map(|s| (s, value * u64::from(share) / u64::from(FULL_BP)));
        Some((current, value))
    }
}

/// The stacks `side`'s cast of `spell` at `mastery` aimed at `hex` reaches (BAT-MAG-007), by id:
/// the computer's estimate of an area spell takes the rules' area (0x437040 with 0x5A4920; E23
/// T4a; a stack the spell cannot affect would add nothing to it).
pub(crate) fn area_estimate<'a>(
    (state, ruleset): (&'a BattleState, &'a BattleRuleset),
    side: BattleSide,
    (spell, mastery): (SpellRef<'a>, u8),
    hex: Hex,
) -> impl Iterator<Item = StackId> + 'a {
    area_stacks(state, ruleset, side, (spell, mastery), hex)
}

/// The innate spell of `creature` with the timed change `change` (ability `innate_spell`): the
/// spell and its value, the first such ability's (BAT-MAG-057, BAT-MAG-058).
pub(crate) fn innate(
    ruleset: &BattleRuleset,
    creature: CreatureTypeId,
    change: TimedChange,
) -> Option<(SpellId, &BattleSpell, u8)> {
    ruleset.abilities(creature).iter().find_map(|a| match *a {
        Ability::InnateSpell { spell, value } => ruleset
            .spell(spell)
            .filter(|s| s.magic.effects.contains(&SpellEffect::Timed(change)))
            .map(|s| (spell, s, value)),
        _ => None,
    })
}

/// Magic Mirror's chance on `stack`, percent: the spell's while it is active (BAT-MAG-022), at
/// least an innate mirror's (the Faerie Dragon's 20 %, BAT-MAG-057; the original's 0x4487F0).
// rule: BAT-MAG-057
pub(crate) fn mirror_chance(ruleset: &BattleRuleset, stack: &Stack) -> u8 {
    let innate = innate(ruleset, stack.creature, TimedChange::MagicMirror).map_or(0, |i| i.2);
    stack.spells.mirror.max(innate)
}

/// Where a hero's `cast`, aimed at stack `s`, goes (BAT-MAG-022): with Magic Mirror active on an
/// enemy stack, or its creature's own (BAT-MAG-057), a draw below its chance sends the spell to
/// a living stack of the caster's side drawn with equal chances, and a `SpellRedirected` event
/// reports it; otherwise it stays (`None`). A spell that ignores immunity (Dispel) is never sent
/// away. Not asked for a creature's cast, which is never sent away ([`reach_stacks`],
/// BAT-MAG-049). (The original draws its chance for every hero's cast at an enemy stack.) The
/// redirect leaves out the caster's side's turrets ([`living`], BAT-SIE-019).
// rule: BAT-MAG-022
fn mirror<S: Sink>(
    state: &mut BattleState,
    ruleset: &BattleRuleset,
    cast: &Cast<'_>,
    s: StackId,
    sink: &mut S,
) -> Option<StackId> {
    let stack = state.at(s);
    let chance = u32::from(mirror_chance(ruleset, stack));
    if stack.side == cast.side
        || chance == 0
        || cast.spell.magic.ignores_immunity
        || state.rng.below(100) >= chance
    {
        return None;
    }
    // The caster's side has a living stack while the battle goes on.
    let count = u32::try_from(living(state, cast.side).count()).unwrap_or(u32::MAX);
    let k = usize::try_from(state.rng.below(count)).unwrap_or(0);
    let to = living(state, cast.side).nth(k).unwrap_or(s);
    sink.emit(Event::public(EventKind::SpellRedirected { from: s, to }));
    Some(to)
}

/// Applies a validated cast: the hero pays the cost and counts the cast, or the casting stack uses
/// one of its casts (BAT-MAG-048), then the spell takes effect (BAT-MAG-004 to BAT-MAG-008), the
/// stacks it killed may be reborn (BAT-MAG-059) and, after a hero's cast, the other side's
/// channellers give their hero a share of the cost (BAT-MAG-055); the battle may end. The active
/// stack keeps its turn after its hero's cast.
pub(crate) fn apply_cast<S: Sink>(
    state: &mut BattleState,
    ruleset: &BattleRuleset,
    cast: Cast<'_>,
    sink: &mut S,
) {
    let cast = if cast.draws {
        draw_useful(state, ruleset, cast)
    } else {
        cast
    };
    match cast.by {
        // rule: BAT-MAG-053
        Some(stack) if cast.automatic => sink.emit(Event::public(EventKind::PeriodicCast {
            stack,
            spell: cast.id,
        })),
        Some(stack) => {
            // rule: BAT-MAG-048
            casters::spend(state, stack);
            sink.emit(Event::public(EventKind::StackCast {
                stack,
                spell: cast.id,
                target: cast.target,
            }));
        }
        None => {
            // Validated: the side has a hero with the mana.
            for hero in state.heroes[cast.side.index()].iter_mut() {
                hero.view.mana -= cast.cost;
                hero.view.casts += 1;
            }
            sink.emit(Event::public(EventKind::SpellCast {
                side: cast.side,
                spell: cast.id,
                target: cast.target,
                second: cast.second,
            }));
            eagle_eye(state, cast.side, cast.id);
        }
    }
    take_effect(state, ruleset, &cast, sink);
    // rule: BAT-MAG-059
    raise::rebirths(state, ruleset, sink);
    if cast.by.is_none() {
        // rule: BAT-MAG-055
        crate::mana::channel(state, ruleset, (cast.side, cast.cost), sink);
    }
    // The battle was not over before the cast: it ends at most once.
    check_end(state, sink);
    if state.result.is_some() {
        state.active = None;
    }
}

/// The effect of the validated `cast` (BAT-MAG-004 to BAT-MAG-039): obstacles placed or removed,
/// stacks raised, cloned or summoned, or the spell's effects on the stacks it reaches.
fn take_effect<S: Sink>(
    state: &mut BattleState,
    ruleset: &BattleRuleset,
    cast: &Cast<'_>,
    sink: &mut S,
) {
    let magic = &cast.spell.magic;
    if magic.quakes() {
        // rule: BAT-SIE-015
        // The hits are the spell's value for the mastery, no power (the original's 0x5A7C80).
        let hits = cast.spell.effect[usize::from(cast.mastery)];
        // Validated: a siege (`may_quake`).
        state.siege.into_iter().for_each(|siege| {
            crate::siege::earthquake(state, siege, u32::from(hits), sink);
        });
        return;
    }
    if magic.removes_obstacles() {
        // rule: BAT-MAG-034
        // Validated: the target is a hex with something to remove.
        let hex = cast.target.hex().unwrap_or_default();
        obstacles::remove_at(state, cast.side, (magic, cast.mastery), hex, sink);
        return;
    }
    if let (Some(kind), Some(shape)) = (magic.obstacle_kind(), &magic.obstacle) {
        // rule: BAT-MAG-027
        let field = Field::new(ruleset.battle().field);
        let hexes = match cast.target {
            // Validated: the shape fits.
            CastTarget::Hex(h) => {
                obstacles::shape_hexes(field, shape, (cast.side, cast.mastery), h)
                    .unwrap_or_default()
            }
            // A random shape (validated: target none).
            _ => obstacles::random_hexes(
                state,
                ruleset,
                cast.side,
                shape.hexes[usize::from(cast.mastery)],
            ),
        };
        let placed = (cast.id, kind, shape);
        obstacles::place(
            state,
            placed,
            cast.side,
            (hexes.as_slice(), cast.value),
            sink,
        );
        return;
    }
    if magic.raises() || magic.sacrifices() {
        // Validated: the target is a stack, and a sacrifice's second target its victim.
        let s = cast.target.stack().unwrap_or_default();
        let (hit_points, temporary) = if magic.sacrifices() {
            // rule: BAT-MAG-036
            let victim = cast.second.stack().unwrap_or_default();
            (
                raise::sacrifice(state, ruleset, victim, cast.value, sink),
                false,
            )
        } else {
            // rule: BAT-MAG-035
            let own = (cast.spell, cast.bonuses.spell(cast.id));
            (
                raise::raise_value(state, ruleset, own, cast.value, s),
                magic.temporary_at(cast.mastery),
            )
        };
        raise::raise(state, ruleset, (s, hit_points), temporary, sink);
    } else if magic.clones() {
        // rule: BAT-MAG-038
        // Validated: the target is a stack.
        let s = cast.target.stack().unwrap_or_default();
        created::clone(state, ruleset, (s, cast.rounds), sink);
    } else if magic.summons() {
        // rule: BAT-MAG-039
        // Validated: the spell summons a creature.
        let creature = cast.spell.summons.unwrap_or_default();
        let effect = cast.spell.effect[usize::from(cast.mastery)];
        let count = u32::from(effect) * u32::from(cast.power);
        created::summon(state, ruleset, (cast.side, creature), count, sink);
    } else {
        reach_stacks(state, ruleset, cast, sink);
    }
    if cast
        .spell
        .magic
        .clears_obstacles_from
        .is_some_and(|m| cast.mastery >= m)
    {
        obstacles::clear(state, sink);
    }
}

/// The stacks a cast that does not raise creatures reaches, and its effects on them (BAT-MAG-004
/// to BAT-MAG-009, BAT-MAG-022, BAT-MAG-029): its target stack, for a hero's cast mirrored and
/// rolling first, its chain, Teleport's move, or the stacks of its area or of the battle.
fn reach_stacks<S: Sink>(
    state: &mut BattleState,
    ruleset: &BattleRuleset,
    cast: &Cast<'_>,
    sink: &mut S,
) {
    let spell = (cast.id, cast.spell);
    // Only a hero's spell is ever sent away by a Magic Mirror (BAT-MAG-022): a creature's cast
    // always meets the stack it is aimed at (the original hands the redirect to a hero's cast
    // alone, 0x479036 against 0x47936E, `installation.md` §7.21; so does VCMI's `isReflected`).
    // rule: BAT-MAG-049
    let target = match cast.target {
        CastTarget::Stack(s) if cast.by.is_none() => {
            CastTarget::Stack(mirror(state, ruleset, cast, s, sink).unwrap_or(s))
        }
        other => other,
    };
    // The stack a hero's stack spell is aimed at rolls first: if it resists, the whole spell does
    // (BAT-MAG-006, BAT-MAG-008); a stack a Magic Mirror sent the spell to rolls alike, and
    // resists if the spell cannot affect it (BAT-MAG-022). A creature's cast has no such roll
    // (BAT-MAG-048, the original's 0x5A02EA). An area spell has none either: every stack in its
    // area rolls once (BAT-MAG-007; batch 2a R3).
    let rolls = cast.by.is_none();
    if let CastTarget::Stack(s) = target
        && (!affects_stack(state, ruleset, cast.side, spell, s)
            || !admits(state, ruleset, cast.side, (spell, cast.mastery), s)
            || (rolls && resists(state, ruleset, cast.side, cast.spell, s)))
    {
        resisted(s, sink);
    } else {
        match (target, cast.spell.magic.chain) {
            (CastTarget::Stack(first), Some(chain)) => {
                // rule: BAT-MAG-006
                let mut struck = [false; STACK_CAPACITY];
                let mut value = cast.value;
                let mut current = first;
                for _ in 0..chain.targets[usize::from(cast.mastery)] {
                    affect(state, ruleset, cast, (current, value), sink);
                    struck[current.index()] = true;
                    let Some(next) =
                        next_in_chain(state, ruleset, (cast.side, spell), current, &struck)
                    else {
                        break;
                    };
                    current = next;
                    value = value * u64::from(chain.factor_bp) / u64::from(FULL_BP);
                }
            }
            (CastTarget::Stack(s), None) if cast.spell.magic.teleports() => {
                // Validated: the second target is a hex.
                let to = cast.second.hex().unwrap_or(state.at(s).hex);
                teleport(state, ruleset, s, to, sink);
            }
            (CastTarget::Stack(s), None) => affect(state, ruleset, cast, (s, cast.value), sink),
            (target, _) => {
                // rule: BAT-MAG-007
                let mut hits = [false; STACK_CAPACITY];
                for s in stack_ids(state) {
                    hits[s.index()] = match target {
                        CastTarget::Hex(h) => {
                            area_stacks(state, ruleset, cast.side, (spell, cast.mastery), h)
                                .any(|a| a == s)
                        }
                        _ => reachable(state, ruleset, cast.side, (spell, cast.mastery), s),
                    };
                }
                for s in stack_ids(state).filter(|s| hits[s.index()]) {
                    if !state.at(s).is_alive() {
                        // Its original's death took a clone (BAT-MAG-038).
                        continue;
                    }
                    if resists(state, ruleset, cast.side, cast.spell, s) {
                        resisted(s, sink);
                    } else {
                        affect(state, ruleset, cast, (s, cast.value), sink);
                    }
                }
            }
        }
    }
}

/// Moves stack `s` to the hex `to` of its validated second target (Teleport, BAT-MAG-029): no
/// hex it lands on acts on it (`installation.md` §7.15). First the stacks it binds are released
/// and its own Bind ends (BAT-MAG-045; the original's 0x5A1B88 and 0x5A1B91).
// rule: BAT-MAG-029
fn teleport<S: Sink>(
    state: &mut BattleState,
    ruleset: &BattleRuleset,
    s: StackId,
    to: Hex,
    sink: &mut S,
) {
    // rule: BAT-MAG-045
    crate::binds::release(state, ruleset, s, sink);
    crate::binds::end(state, ruleset, s, sink);
    let from = core::mem::replace(&mut state.at_mut(s).hex, to);
    sink.emit(Event::public(EventKind::Teleported { stack: s, from, to }));
}

/// Whether the spell of the obstacle `o` hurts stack `s` (the original's `f > 0`, 0x59FCD0 and
/// 0x59FB40, `installation.md` §7.15, §7.17): the spell can affect the stack (BAT-MAG-005,
/// BAT-MAG-021) and its share after resistance is above 0 (BAT-MAG-008); there is no resistance
/// roll.
pub(crate) fn hurts(
    state: &BattleState,
    ruleset: &BattleRuleset,
    o: &SpellObstacle,
    s: StackId,
) -> bool {
    ruleset.spell(o.spell).is_some_and(|spell| {
        affects_stack(state, ruleset, o.side, (o.spell, spell), s)
            && share(state, ruleset, o.side, s).0 > 0
    })
}

/// The damage of the obstacle `fire` with its spell `spell` to stack `s` whether or not the spell
/// hurts it (BAT-MAG-028, BAT-MAG-033; the original's 0x5A78E0 with the hero of the side that cast
/// it as the caster, the stack's reductions and protection), as [`spell_damage`] gives it: the
/// value after the caster's bonuses and the damage. The computer's Fire Wall scores estimate it
/// without the test (`computer::fire_wall_damage`).
pub(crate) fn obstacle_damage(
    state: &BattleState,
    ruleset: &BattleRuleset,
    (fire, spell): (&SpellObstacle, &BattleSpell),
    s: StackId,
) -> (u64, u64) {
    let hero = state.heroes[fire.side.index()]
        .as_ref()
        .map(|h| &h.magic.bonuses);
    let target = state.at(s);
    spell_damage(
        ruleset,
        ((fire.spell, spell), hero),
        fire.value,
        (target.creature, &target.spells.school_protection),
    )
}

/// The damage of the obstacle `fire` (Fire Wall, BAT-MAG-028, or a mine, BAT-MAG-033; the
/// original's 0x59FCD0 and 0x59FB40) to stack `s`: none unless its spell [`hurts`] the stack;
/// else the obstacle's value as the spell's damage with the bonuses of the hero of the side that
/// cast it as the caster's and the stack's reductions and protection ([`spell_damage`]),
/// reported; any damage ends what it ends (BAT-MAG-024).
// rule: BAT-MAG-028
pub(crate) fn burn<S: Sink>(
    state: &mut BattleState,
    ruleset: &BattleRuleset,
    fire: &SpellObstacle,
    s: StackId,
    sink: &mut S,
) {
    let found = ruleset
        .spell(fire.spell)
        .filter(|_| hurts(state, ruleset, fire, s));
    let Some(spell) = found else {
        return;
    };
    let (caster, damage) = obstacle_damage(state, ruleset, (fire, spell), s);
    let kills = apply_damage(state.at_mut(s), damage);
    sink.emit(Event::public(EventKind::SpellDamage {
        target: s,
        value: fire.value,
        caster,
        damage,
        kills,
    }));
    effects::damaged(state, ruleset, s, sink);
}
