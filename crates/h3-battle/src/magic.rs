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
    Ability, CreatureFlag, CreatureTypeId, EARLY_MASS_VERSION, FIRST_PROTECTION, FULL_BP,
    MASS_CURE_SKIPS_HYPNOTIZED, MAX_FIELD_HEXES, MAX_SPELL_SCHOOLS, MAX_SPELLS, Ruleset,
    SINGLE_PRECISION_RATES, SpecialistBonus, Spell, SpellCategory, SpellEffect, SpellId,
    SpellPolarity, SpellScope, SpellTarget, TargetSide, TimedChange,
};
use h3_contracts::{BattleEffect, BattleHero, BattleSide, Rate};
use h3_kernel::arith::{
    SINGLE_BITS, SINGLE_FRAC_BITS, X87, mul_div_floor, round_single, single_precision,
};
use h3_kernel::{CanonicalHash, FixedVec, StateHasher};

use crate::combat::check_end;
use crate::damage::apply_damage;
use crate::effects::{self, Status, TimedCast};
use crate::error::{CommandError, HeroError};
use crate::event::{Event, EventKind, Sink};
use crate::geometry::{Field, Footprint, Hex};
use crate::hero::{HeroRecord, hash_rate};
use crate::movement;
use crate::obstacles::{self, SpellObstacle};
use crate::state::{BattleState, STACK_CAPACITY, Stack, StackId};

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
    /// Mastery per school, by school handle (private).
    pub(crate) schools: [u8; MAX_SPELL_SCHOOLS],
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
    /// The distinct spells cast so far (public through the cast events; the outcome's).
    pub(crate) cast: SpellSet,
}

impl HeroMagic {
    /// The magic of `hero` (checked by `hero::check`); `single` selects the single-precision
    /// rates of quirk Q-0005.
    pub(crate) fn of(hero: &BattleHero, single: bool) -> Result<Self, HeroError> {
        let mut book = SpellSet::default();
        for &spell in hero.magic.spell_book.as_deref().unwrap_or_default() {
            book.insert(spell);
        }
        let mut resistance = 0u64;
        let mut duration_bonus = 0u16;
        for effect in &hero.effects {
            match *effect {
                BattleEffect::MagicResistance { rate } => {
                    let sum = resistance.saturating_add(hero_rate(single, rate));
                    resistance = if single { round_single(sum) } else { sum };
                }
                BattleEffect::SpellDuration { rounds } => {
                    duration_bonus = duration_bonus.saturating_add(rounds.into());
                }
                _ => {}
            }
        }
        Ok(Self {
            book,
            schools: hero.magic.schools,
            bonuses: SpellBonuses::of(hero).ok_or(HeroError::TooManySpellBonuses)?,
            resistance: resistance.min(scale(single)),
            duration_bonus,
            cast: SpellSet::default(),
        })
    }
}

impl CanonicalHash for HeroMagic {
    /// Book bits, schools, the bonuses (every spell, by school (rates since v23), the single
    /// spells as length and spell, damage, growth per level, level), the resistance, the
    /// duration bonus (v20), the cast spells' bits.
    fn hash_into(&self, h: &mut StateHasher) {
        for word in self.book.bits().into_iter().chain(self.cast.bits()) {
            h.write_u64(word);
        }
        h.write_bytes(&self.schools);
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
    }
}

/// What a cast is aimed at (ADR-0014 point 3): its target, and its second target where the spell
/// takes one (Teleport's destination hex, BAT-MAG-029; Sacrifice's victim joins with T6).
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
    spell: &'r Spell,
    cost: u16,
    mastery: u8,
    target: CastTarget,
    /// The second target (Teleport's destination).
    second: CastTarget,
    /// `power × power factor + effect[mastery]` (BAT-MAG-004).
    value: u64,
    /// The caster's bonuses (BAT-MAG-004).
    bonuses: SpellBonuses,
    /// The duration of its timed effects: power plus the duration bonus, at least 1
    /// (BAT-MAG-010).
    rounds: u16,
}

/// Whether the battle implements the effects of `spell`: every effect kind is implemented
/// (damage since E04b T2, timed changes since T3a, dispel and heal since T3b1b), so a spell is
/// castable once it has effects.
fn implemented(spell: &Spell) -> bool {
    !spell.magic.effects.is_empty()
}

/// The mastery at which `hero` casts `spell`: the best of its schools' masteries, at most
/// Expert (BAT-MAG-002).
// rule: BAT-MAG-002
fn mastery(hero: &HeroRecord, spell: &Spell) -> u8 {
    spell
        .schools
        .iter()
        .map(|l| hero.magic.schools[l.index().min(MAX_SPELL_SCHOOLS - 1)])
        .max()
        .unwrap_or(0)
        .min(3)
}

/// Checks that `side`'s hero may cast `id` now apart from its target (BAT-MAG-001, BAT-MAG-002)
/// and returns the spell, the hero, the mastery and the cost. The caller has checked that a
/// stack of `side` awaits a command outside the tactics phase.
// rule: BAT-MAG-001
fn check_spell<'s, 'r>(
    state: &'s BattleState,
    ruleset: &'r Ruleset,
    side: BattleSide,
    id: SpellId,
) -> Result<(&'r Spell, &'s HeroRecord, u8, u16), CommandError> {
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
    if !implemented(spell) {
        return Err(CommandError::NotCastable);
    }
    let mastery = mastery(hero, spell);
    let cost = spell.cost[usize::from(mastery)];
    if hero.view.mana < cost {
        return Err(CommandError::NotEnoughMana);
    }
    Ok((spell, hero, mastery, cost))
}

/// A spell together with its handle.
type SpellRef<'r> = (SpellId, &'r Spell);

/// Whether a cast of `spell` can affect a stack of `creature` at all (BAT-MAG-005): the creature
/// meets the spell's conditions and, unless the spell ignores immunity (Dispel), is not immune
/// to it as a mind spell (flag `mind_immune`), by the spell's level, one of its schools or the
/// spell itself. (The Orb of Vulnerability joins with T8.)
// rule: BAT-MAG-005
fn affects(ruleset: &Ruleset, (id, spell): SpellRef<'_>, creature: CreatureTypeId) -> bool {
    let flags = ruleset
        .creature(creature)
        .map(|c| c.flags)
        .unwrap_or_default();
    if spell.magic.ignores_immunity {
        return spell.magic.conditions.admits(flags);
    }
    spell.magic.conditions.admits(flags)
        && !(spell.magic.mind && flags.contains(CreatureFlag::MindImmune))
        && !ruleset.abilities(creature).iter().any(|a| match *a {
            Ability::SpellLevelImmunity { max_level } => spell.level <= max_level,
            Ability::SchoolImmunity { school } => spell.schools.contains(&school),
            Ability::SpellImmunity { spell } => spell == id,
            _ => false,
        })
}

/// Whether Anti-Magic on `stack` keeps `spell` off it (BAT-MAG-021): a spell below the level it
/// bounds, whatever its polarity, unless the spell is a creature ability or ignores immunity
/// (Dispel; the original tests both before Anti-Magic, `installation.md` §7.13).
// rule: BAT-MAG-021
pub(crate) fn kept_off(stack: &Stack, spell: &Spell) -> bool {
    spell.level < stack.spells.spell_immunity
        && spell.category != SpellCategory::Ability
        && !spell.magic.ignores_immunity
}

/// Whether `spell` can affect the living stack `s` at all, wherever its caster stands
/// (BAT-MAG-005, BAT-MAG-021): its creature meets the spell's conditions and immunities, and
/// Anti-Magic does not keep the spell off it.
fn affects_stack(state: &BattleState, ruleset: &Ruleset, spell: SpellRef<'_>, s: StackId) -> bool {
    let stack = state.at(s);
    stack.is_alive() && affects(ruleset, spell, stack.creature) && !kept_off(stack, spell.1)
}

/// Whether `side`'s cast of `spell` at `mastery` may reach stack `s` (BAT-MAG-003,
/// BAT-MAG-005): a living stack on the side the spell may reach at that mastery (its own side,
/// whoever controls it: the original's side tests, `installation.md` §7.14) that the spell can
/// affect and [`admits`], unless a mass version passes over it hypnotized
/// ([`skips_hypnotized`]).
#[inline(always)]
fn reachable(
    state: &BattleState,
    ruleset: &Ruleset,
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
    // The spell's own tests and hypnotized stacks are rare: test them only when they apply.
    let own_tests = spell.1.magic.health_limit || !spell.1.magic.refuses.is_empty();
    side_ok
        && affects_stack(state, ruleset, spell, s)
        && (!own_tests || admits(state, ruleset, side, (spell, mastery), s))
        && !(stack.spells.status.contains(Status::HYPNOTIZED)
            && skips_hypnotized(state, ruleset, (spell, mastery), s))
}

/// Whether `side`'s cast of `spell` at `mastery` admits stack `s` by the spell's own tests
/// (BAT-MAG-026, `installation.md` §7.14): no active effect of a change the spell refuses
/// (Hypnotize: Magic Mirror), and with a health limit, full hit points (creatures × hit points
/// per creature, the top creature's wounds not counted) at most the cast's value with the
/// casting hero's specialist bonus for the stack (BAT-MAG-020).
// rule: BAT-MAG-026
fn admits(
    state: &BattleState,
    ruleset: &Ruleset,
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
/// hypnotized (BAT-MAG-026; the original's mass handler 0x5A66D0 and Expert Cure's loop skip
/// them, `installation.md` §7.14): with quirk Q-0012 every mass version does, without it those
/// whose dispel does not end the stack's Hypnotize (Cure's does).
fn skips_hypnotized(
    state: &BattleState,
    ruleset: &Ruleset,
    ((_, spell), mastery): (SpellRef<'_>, u8),
    s: StackId,
) -> bool {
    let mass = spell.magic.target == SpellTarget::Stack
        && target_at(ruleset, spell, mastery) == SpellTarget::None;
    if !mass {
        return false;
    }
    let hypnosis = SpellEffect::Timed(TimedChange::Hypnotize);
    let ends_it = spell.magic.dispels()
        && state.effects.of(s).iter().any(|e| {
            ruleset
                .spell(e.spell)
                .is_some_and(|sp| sp.magic.effects.contains(&hypnosis))
                && effects::dispel_ends(spell, e)
        });
    ruleset.quirks().is_enabled(MASS_CURE_SKIPS_HYPNOTIZED) || !ends_it
}

/// Whether `side`'s cast of a stack spell at `mastery` may be aimed at stack `s` (BAT-MAG-003):
/// a stack it may reach, with an active timed effect if the spell needs one (Dispel), whose
/// share left after resistance is above zero (the original's target test, `installation.md`
/// §7.8: a stack that always resists is no target).
fn aimable(
    state: &BattleState,
    ruleset: &Ruleset,
    side: BattleSide,
    (spell, mastery): (SpellRef<'_>, u8),
    s: StackId,
) -> bool {
    let magic = &spell.1.magic;
    reachable(state, ruleset, side, (spell, mastery), s)
        && (!magic.needs_effect || !state.effects.of(s).is_empty())
        && (spell.1.polarity == SpellPolarity::Positive
            || magic.ignores_immunity
            || (share(state, ruleset, s).0 > 0 && aura(state, ruleset, s) != Some(100)))
}

/// The living stacks of `side`, by id.
fn living(state: &BattleState, side: BattleSide) -> impl Iterator<Item = StackId> + '_ {
    stack_ids(state).filter(move |&s| {
        let stack = state.at(s);
        stack.side == side && stack.is_alive()
    })
}

/// Every stack id of the battle.
fn stack_ids(state: &BattleState) -> impl Iterator<Item = StackId> + use<> {
    (0u8..).take(state.stacks.len()).map(StackId)
}

/// The distances from an area's centre that the area of `spell` at `mastery` covers: from
/// `inner` to the radius (BAT-MAG-007; a hex spell without an area covers its hex).
fn area(spell: &Spell, mastery: u8) -> (u32, u32) {
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
    ruleset: &'a Ruleset,
    side: BattleSide,
    (spell, mastery): (SpellRef<'a>, u8),
    hex: Hex,
) -> impl Iterator<Item = StackId> + 'a {
    let field = Field::new(ruleset.battle().field);
    let range = area(spell.1, mastery);
    stack_ids(state).filter(move |&s| {
        reachable(state, ruleset, side, (spell, mastery), s)
            && in_area(field, hex, range, state.at(s).footprint(field))
    })
}

/// Whether `target` is a first target of `side`'s cast of `spell` at `mastery` (BAT-MAG-003): a
/// stack of [`aimable`] for a stack spell; for a hex spell any hex outside the side columns,
/// whatever stands on it (the original's cursor test, `installation.md` §7.8; batch 2a R3),
/// but for an obstacle spell a hex where its shape fits hexes free as the side sees them
/// (BAT-MAG-027, BAT-MAG-031); nothing for a battle-wide spell or a mass version (from the
/// spell's mass mastery), which must reach at least one stack, or for an obstacle spell of a
/// random shape, which is cast whether or not a hex is free (BAT-MAG-032) but for mines not
/// while the other side's natives would see them (BAT-MAG-033). Whether a second target exists
/// is not checked here.
// rule: BAT-MAG-003
fn first_target(
    state: &BattleState,
    ruleset: &Ruleset,
    side: BattleSide,
    (spell, mastery): (SpellRef<'_>, u8),
    target: CastTarget,
) -> bool {
    match (target_at(ruleset, spell.1, mastery), target) {
        (SpellTarget::Stack, CastTarget::Stack(s)) => {
            state.stack(s).is_some() && aimable(state, ruleset, side, (spell, mastery), s)
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
        (SpellTarget::None, CastTarget::None) => {
            // rule: BAT-MAG-032
            spell.1.magic.obstacle_kind().map_or_else(
                || stack_ids(state).any(|s| reachable(state, ruleset, side, (spell, mastery), s)),
                |kind| obstacles::may_place(state, side, kind),
            )
        }
        _ => false,
    }
}

/// Whether `second` is the second target of a cast of `spell` aimed at `target` (BAT-MAG-029):
/// for Teleport a hex, other than the stack's position, where the stack fits (inside the field,
/// outside the side columns, not blocked by an obstacle or another living stack; the original's
/// 0x5A3700, the same at every mastery outside sieges, `installation.md` §7.15); none for every
/// other spell.
// rule: BAT-MAG-029
fn second_target(
    state: &BattleState,
    ruleset: &Ruleset,
    spell: &Spell,
    (target, second): (CastTarget, CastTarget),
) -> bool {
    match (spell.magic.teleports(), target, second) {
        (true, CastTarget::Stack(s), CastTarget::Hex(h)) => {
            let field = Field::new(ruleset.battle().field);
            let stack = state.at(s);
            h != stack.hex && movement::fits(field, stack, h, state.blocked_for(field, s))
        }
        (false, _, CastTarget::None) => true,
        _ => false,
    }
}

/// Checks that `(target, second)` are the targets of `side`'s cast of `spell` at `mastery`
/// ([`first_target`], [`second_target`]).
fn check_target(
    state: &BattleState,
    ruleset: &Ruleset,
    side: BattleSide,
    (spell, mastery): (SpellRef<'_>, u8),
    (target, second): (CastTarget, CastTarget),
) -> Result<(), CommandError> {
    if first_target(state, ruleset, side, (spell, mastery), target)
        && second_target(state, ruleset, spell.1, (target, second))
    {
        Ok(())
    } else {
        Err(CommandError::InvalidSpellTarget)
    }
}

/// Whether a cast of `spell` aimed at `target` has a second target ([`second_target`]): every
/// target of a spell without one does.
fn has_second(state: &BattleState, ruleset: &Ruleset, spell: &Spell, target: CastTarget) -> bool {
    !spell.magic.teleports()
        || hex_targets(ruleset).any(|h| second_target(state, ruleset, spell, (target, h)))
}

/// Every hex of the field as a target.
fn hex_targets(ruleset: &Ruleset) -> impl Iterator<Item = CastTarget> + use<> {
    (0u8..)
        .take(Field::new(ruleset.battle().field).hex_count())
        .map(|h| CastTarget::Hex(Hex(h)))
}

/// What a cast of `spell` at `mastery` is aimed at (BAT-MAG-003): nothing for its mass version,
/// which starts at the original's earlier mastery with quirk Q-0011 (BAT-MAG-023: Forgetfulness
/// from Advanced), else at the fixed one.
fn target_at(ruleset: &Ruleset, spell: &Spell, mastery: u8) -> SpellTarget {
    let early = ruleset.quirks().is_enabled(EARLY_MASS_VERSION);
    spell.magic.target_at(mastery, early)
}

/// `power × power factor + effect[mastery]` of `hero`'s cast (BAT-MAG-004).
fn base_value(hero: &HeroRecord, spell: &Spell, mastery: u8) -> u64 {
    u64::from(hero.view.power) * u64::from(spell.power)
        + u64::from(spell.effect[usize::from(mastery)])
}

/// Validates `side`'s cast of `spell` at `target`; the caller has checked that a stack of
/// `side` awaits a command outside the tactics phase.
pub(crate) fn plan_cast<'r>(
    state: &BattleState,
    ruleset: &'r Ruleset,
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
    })
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
    ruleset: &Ruleset,
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
            && has_second(state, ruleset, s, target)
        {
            // At most `MAX_CAST_TARGETS` targets by construction.
            let _ = out.push(target);
        }
    }
}

/// Fills `out` with every second target [`crate::apply`] accepts for `actor`'s hero casting
/// `spell` at `target` now (BAT-MAG-029): Teleport's destination hexes, or only none for a spell
/// without a second target. Empty if `target` is no target of the cast now.
pub fn second_targets(
    state: &BattleState,
    ruleset: &Ruleset,
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
    // Only Teleport has hexes as second targets (`second_target`).
    let hexes = hex_targets(ruleset).take(if s.magic.teleports() { usize::MAX } else { 0 });
    for second in core::iter::once(CastTarget::None).chain(hexes) {
        if second_target(state, ruleset, s, (target, second)) {
            // At most `MAX_CAST_TARGETS` targets by construction.
            let _ = out.push(second);
        }
    }
}

/// Whether `actor`'s cast of `spell` has a target ([`cast_targets`] not empty, computed without
/// listing every hex where it can be avoided): a stack spell needs a stack of [`aimable`] with a
/// second target, a battle-wide spell a stack it reaches, but a random obstacle spell none (mines
/// only while the other side's natives would not see them); a hex spell always has one, but an
/// obstacle spell needs a hex where its shape fits and a remove spell a hex with an obstacle it
/// removes.
fn has_target(
    state: &BattleState,
    ruleset: &Ruleset,
    actor: BattleSide,
    (spell, mastery): (SpellRef<'_>, u8),
) -> bool {
    match target_at(ruleset, spell.1, mastery) {
        SpellTarget::Hex
            if spell.1.magic.obstacle.is_some() || spell.1.magic.removes_obstacles() =>
        {
            hex_targets(ruleset).any(|h| first_target(state, ruleset, actor, (spell, mastery), h))
        }
        SpellTarget::Hex => true,
        SpellTarget::Stack => stack_ids(state).any(|s| {
            aimable(state, ruleset, actor, (spell, mastery), s)
                && has_second(state, ruleset, spell.1, CastTarget::Stack(s))
        }),
        SpellTarget::None => spell.1.magic.obstacle_kind().map_or_else(
            || stack_ids(state).any(|s| reachable(state, ruleset, actor, (spell, mastery), s)),
            |kind| obstacles::may_place(state, actor, kind),
        ),
    }
}

/// The spells `actor`'s hero may cast now: those with at least one target in
/// [`cast_targets`] (empty if it is not `actor`'s turn, in the tactics phase or without a hero).
pub fn castable_spells(state: &BattleState, ruleset: &Ruleset, actor: BattleSide) -> SpellSet {
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

/// The damage of `cast` with the value `value` on a stack of `creature` with the school
/// protections `protection` (BAT-MAG-004) after the caster's bonuses and after the target's
/// extra damage or reduction and protection, in the original's order (`installation.md` §7.8):
/// the caster's bonuses (× (1 + the best Orb among the spell's schools) × (1 + Sorcery), kept
/// fractional and rounded down once; then the spell's own bonus of that, rounded down, and a
/// specialist's bonus of that, [`specialist_bonus`]), then the target's (one
/// spell's extra damage, rounded down; the spell damage reduction, rounded down; one protection,
/// [`protected`]). The rates are exact, or single precision with quirk Q-0005.
// rule: BAT-MAG-004
fn spell_damage(
    ruleset: &Ruleset,
    ((id, spell), b): (SpellRef<'_>, &SpellBonuses),
    value: u64,
    (creature, protection): (CreatureTypeId, &[u8; MAX_SPELL_SCHOOLS]),
) -> (u64, u64) {
    let single = ruleset.quirks().is_enabled(SINGLE_PRECISION_RATES);
    let one = u128::from(scale(single));
    let school = spell
        .schools
        .iter()
        .map(|l| hero_rate(single, b.school[l.index().min(MAX_SPELL_SCHOOLS - 1)]))
        .max()
        .unwrap_or(0);
    let factor = (one + u128::from(school)) * (one + u128::from(hero_rate(single, b.all)));
    let t = u64::try_from(u128::from(value) * factor / (one * one)).unwrap_or(u64::MAX);
    let own = b.spell(id);
    let specialty = specialist_bonus(ruleset, spell, own, creature, t);
    let fixed = u128::from(t) * u128::from(own.damage_bp) / u128::from(FULL_BP);
    let caster =
        u64::try_from(u128::from(t.saturating_add_signed(specialty)) + fixed).unwrap_or(u64::MAX);
    let mut damage = caster;
    for ability in ruleset.abilities(creature) {
        match *ability {
            Ability::SpellVulnerability { spell, percent } if spell == id => {
                damage = grow(damage, u64::from(percent) * 100);
            }
            Ability::SpellDamageReduction { percent } => {
                damage = damage.saturating_mul(100 - u64::from(percent.min(100))) / 100;
            }
            _ => {}
        }
    }
    (caster, protected(ruleset, spell, damage, protection))
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
        ruleset: &Ruleset,
        spell: &Spell,
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
    ruleset: &Ruleset,
    spell: &Spell,
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
/// ([`spell_damage`]; `installation.md` §7.11). No immunity or resistance applies.
// rule: BAT-MAG-018
pub(crate) fn reflection(
    state: &BattleState,
    ruleset: &Ruleset,
    (striker, target): (&Stack, StackId),
    first: u64,
) -> u64 {
    let shield = SpellEffect::Timed(TimedChange::FireShield);
    let fire_immune = ruleset
        .creature(striker.creature)
        .is_some_and(|c| c.flags.contains(CreatureFlag::FireImmune));
    let found = state.effects.of(target).iter().find_map(|e| {
        ruleset
            .spell(e.spell)
            .filter(|spell| spell.magic.effects.contains(&shield))
            .map(|spell| (e.spell, spell, e.value))
    });
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
    let bonuses = state.heroes[side.index()]
        .as_ref()
        .map_or_else(SpellBonuses::default, |h| h.magic.bonuses);
    spell_damage(
        ruleset,
        ((id, spell), &bonuses),
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
    ruleset: &Ruleset,
    spell: &Spell,
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

/// The share of a hostile spell that stack `s` lets through (BAT-MAG-008) in the fixed point
/// `one`, as `(share, one)`: 1 − its creature's magic resistance minus the resistance of its
/// controller's hero (BAT-MAG-026), at least 0. With quirk Q-0005 the values are single precision and the difference
/// is rounded to it, as the original's x87 code computes it (`installation.md` §7.8).
fn share(state: &BattleState, ruleset: &Ruleset, s: StackId) -> (u64, u64) {
    let single = ruleset.quirks().is_enabled(SINGLE_PRECISION_RATES);
    let one = scale(single);
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

/// 1 − `percent` as a rate in the fixed point of [`scale`].
fn percent_rate(single: bool, percent: u8) -> u64 {
    rate(single, u16::from(100 - percent.min(100)) * 100)
}

/// The strongest magic resistance aura of a living allied stack (of the same controller,
/// BAT-MAG-026) adjacent to stack `s`, percent (BAT-MAG-008).
fn aura(state: &BattleState, ruleset: &Ruleset, s: StackId) -> Option<u8> {
    let field = Field::new(ruleset.battle().field);
    let stack = state.at(s);
    let footprint = stack.footprint(field);
    let controller = stack.controller();
    stack_ids(state)
        .filter(|&n| n != s)
        .map(|n| state.at(n))
        .filter(|n| n.is_alive() && n.controller() == controller)
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

/// The chance in percent that a cast of `spell` affects stack `s` (BAT-MAG-008): always for a
/// positive spell and one that ignores immunity (Dispel); otherwise its [`share`] times 1 − the
/// strongest [`aura`], as a whole percentage rounded down (with quirk Q-0005, 10 % hero
/// resistance leaves 89 %).
// rule: BAT-MAG-008
pub(crate) fn affect_chance(
    state: &BattleState,
    ruleset: &Ruleset,
    spell: &Spell,
    s: StackId,
) -> u32 {
    if spell.polarity == SpellPolarity::Positive || spell.magic.ignores_immunity {
        return 100;
    }
    let single = ruleset.quirks().is_enabled(SINGLE_PRECISION_RATES);
    let (share, one) = share(state, ruleset, s);
    let aura = aura(state, ruleset, s).map_or(one, |p| percent_rate(single, p));
    let chance = u128::from(share) * u128::from(aura) * 100 / (u128::from(one) * u128::from(one));
    u32::try_from(chance).unwrap_or(100).min(100)
}

/// Rolls whether stack `s` resists `spell` (BAT-MAG-008): one draw from 1 to 100 when the chance
/// to be affected is neither 0 nor 100.
fn resists(state: &mut BattleState, ruleset: &Ruleset, spell: &Spell, s: StackId) -> bool {
    let chance = affect_chance(state, ruleset, spell, s);
    match chance {
        0 => true,
        100 => false,
        _ => state.rng.below(100) >= chance,
    }
}

/// Applies `cast`'s effects to stack `s` at the value `value`: its damage, reported, then, if the
/// stack still lives, its dispel (BAT-MAG-016), its heal (BAT-MAG-017) and its timed changes
/// (BAT-MAG-009), in this order (Cure ends the effects before it heals, `installation.md`
/// §7.10).
fn affect<S: Sink>(
    state: &mut BattleState,
    ruleset: &Ruleset,
    cast: &Cast<'_>,
    (s, value): (StackId, u64),
    sink: &mut S,
) {
    if cast.spell.magic.effects.contains(&SpellEffect::Damage) {
        let target = state.at(s);
        let protection = target.spells.school_protection;
        let caster_ref = ((cast.id, cast.spell), &cast.bonuses);
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
/// rows shifted half a hex to the right, a two-hex stack's centre half a hex towards its tail.
/// (The original also lifts each stack by half its sprite's height, which is not modelled,
/// OPEN-QUESTIONS Q43.)
fn screen_centre(state: &BattleState, field: Field, s: StackId) -> (i64, i64) {
    let stack = state.at(s);
    let (col, row) = field.coords(stack.hex);
    let shift = if row % 2 == 0 { 22 } else { 0 };
    let tail = match (stack.is_two_hex(), stack.side) {
        (false, _) => 0,
        (true, BattleSide::Attacker) => -22,
        (true, BattleSide::Defender) => 22,
    };
    (i64::from(44 * col + shift + tail), i64::from(42 * row))
}

/// The next stack Chain Lightning jumps to from `from` (BAT-MAG-006): among the living stacks
/// not yet struck, the defender's first, then the attacker's, each by id, those the spell
/// affects (each rolling for resistance, so a resisting stack is passed over for this jump)
/// and the first at the smallest screen distance, rounded down to whole pixels.
// rule: BAT-MAG-006
fn next_in_chain(
    state: &mut BattleState,
    ruleset: &Ruleset,
    spell: SpellRef<'_>,
    from: StackId,
    struck: &[bool; STACK_CAPACITY],
) -> Option<StackId> {
    let field = Field::new(ruleset.battle().field);
    let (x, y) = screen_centre(state, field, from);
    let mut best: Option<(u64, StackId)> = None;
    for side in [BattleSide::Defender, BattleSide::Attacker] {
        for s in stack_ids(state) {
            let candidate = state.at(s).side == side
                && !struck[s.index()]
                && affects_stack(state, ruleset, spell, s);
            if !candidate || resists(state, ruleset, spell.1, s) {
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

/// Where `cast`, aimed at stack `s`, goes (BAT-MAG-022): with Magic Mirror active on an enemy
/// stack, a draw below its chance sends the spell to a living stack of the caster's side drawn
/// with equal chances, and a `SpellRedirected` event reports it; otherwise to `s`. A spell that
/// ignores immunity (Dispel) is never sent away. (The original draws its chance for every cast at
/// an enemy stack and leaves out arrow towers, E04c; `installation.md` §7.13.)
// rule: BAT-MAG-022
fn mirror<S: Sink>(state: &mut BattleState, cast: &Cast<'_>, s: StackId, sink: &mut S) -> StackId {
    let stack = state.at(s);
    let chance = u32::from(stack.spells.mirror);
    if stack.side == cast.side
        || chance == 0
        || cast.spell.magic.ignores_immunity
        || state.rng.below(100) >= chance
    {
        return s;
    }
    // The caster's side has a living stack while the battle goes on.
    let count = u32::try_from(living(state, cast.side).count()).unwrap_or(u32::MAX);
    let k = usize::try_from(state.rng.below(count)).unwrap_or(0);
    let to = living(state, cast.side).nth(k).unwrap_or(s);
    sink.emit(Event::public(EventKind::SpellRedirected { from: s, to }));
    to
}

/// Applies a validated cast: the hero pays the cost and counts the cast, then the spell takes
/// effect (BAT-MAG-004 to BAT-MAG-008); the battle may end. The active stack keeps its turn.
pub(crate) fn apply_cast<S: Sink>(
    state: &mut BattleState,
    ruleset: &Ruleset,
    cast: Cast<'_>,
    sink: &mut S,
) {
    // Validated: the side has a hero with the mana.
    for hero in state.heroes[cast.side.index()].iter_mut() {
        hero.view.mana -= cast.cost;
        hero.view.casts += 1;
        hero.magic.cast.insert(cast.id);
    }
    sink.emit(Event::public(EventKind::SpellCast {
        side: cast.side,
        spell: cast.id,
        target: cast.target,
        second: cast.second,
    }));
    let spell = (cast.id, cast.spell);
    let magic = &cast.spell.magic;
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
    let target = match cast.target {
        CastTarget::Stack(s) => CastTarget::Stack(mirror(state, &cast, s, sink)),
        other => other,
    };
    // The stack a stack spell is aimed at rolls first: if it resists, the whole spell does
    // (BAT-MAG-006, BAT-MAG-008); a stack a Magic Mirror sent the spell to that it cannot
    // affect resists (BAT-MAG-022). An area spell has no such roll: every stack in its area
    // rolls once (BAT-MAG-007; batch 2a R3).
    if let CastTarget::Stack(s) = target
        && (!affects_stack(state, ruleset, spell, s)
            || !admits(state, ruleset, cast.side, (spell, cast.mastery), s)
            || resists(state, ruleset, cast.spell, s))
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
                    affect(state, ruleset, &cast, (current, value), sink);
                    struck[current.index()] = true;
                    let Some(next) = next_in_chain(state, ruleset, spell, current, &struck) else {
                        break;
                    };
                    current = next;
                    value = value * u64::from(chain.factor_bp) / u64::from(FULL_BP);
                }
            }
            (CastTarget::Stack(s), None) if cast.spell.magic.teleports() => {
                // Validated: the second target is a hex.
                let to = cast.second.hex().unwrap_or(state.at(s).hex);
                teleport(state, s, to, sink);
            }
            (CastTarget::Stack(s), None) => affect(state, ruleset, &cast, (s, cast.value), sink),
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
                    if resists(state, ruleset, cast.spell, s) {
                        resisted(s, sink);
                    } else {
                        affect(state, ruleset, &cast, (s, cast.value), sink);
                    }
                }
            }
        }
    }
    if cast
        .spell
        .magic
        .clears_obstacles_from
        .is_some_and(|m| cast.mastery >= m)
    {
        obstacles::clear(state, sink);
    }
    check_end(state, sink);
    if state.result.is_some() {
        state.active = None;
    }
}

/// Moves stack `s` to the hex `to` of its validated second target (Teleport, BAT-MAG-029): no
/// hex it lands on acts on it (`installation.md` §7.15).
// rule: BAT-MAG-029
fn teleport<S: Sink>(state: &mut BattleState, s: StackId, to: Hex, sink: &mut S) {
    let from = core::mem::replace(&mut state.at_mut(s).hex, to);
    sink.emit(Event::public(EventKind::Teleported { stack: s, from, to }));
}

/// Whether the spell of the obstacle `o` hurts stack `s` (the original's `f > 0`, 0x59FCD0 and
/// 0x59FB40, `installation.md` §7.15, §7.17): the spell can affect the stack (BAT-MAG-005,
/// BAT-MAG-021) and its share after resistance is above 0 (BAT-MAG-008); there is no resistance
/// roll.
pub(crate) fn hurts(state: &BattleState, ruleset: &Ruleset, o: &SpellObstacle, s: StackId) -> bool {
    ruleset.spell(o.spell).is_some_and(|spell| {
        affects_stack(state, ruleset, (o.spell, spell), s) && share(state, ruleset, s).0 > 0
    })
}

/// The damage of the obstacle `fire` (Fire Wall, BAT-MAG-028, or a mine, BAT-MAG-033; the
/// original's 0x59FCD0 and 0x59FB40) to stack `s`: none unless its spell [`hurts`] the stack;
/// else the obstacle's value as the spell's damage with the bonuses of the hero of the side that
/// cast it as the caster's and the stack's reductions and protection ([`spell_damage`]),
/// reported; any damage ends what it ends (BAT-MAG-024).
// rule: BAT-MAG-028
pub(crate) fn burn<S: Sink>(
    state: &mut BattleState,
    ruleset: &Ruleset,
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
    let bonuses = state.heroes[fire.side.index()]
        .as_ref()
        .map_or_else(SpellBonuses::default, |h| h.magic.bonuses);
    let target = state.at(s);
    let protection = target.spells.school_protection;
    let (caster, damage) = spell_damage(
        ruleset,
        ((fire.spell, spell), &bonuses),
        fire.value,
        (target.creature, &protection),
    );
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
