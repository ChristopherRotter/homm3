//! A hero's magic in battle (E04b T2b–T2d; ADR-0014 points 2, 4 and 8): spell books, casting by
//! a hero (BAT-MAG-001), mastery and cost (BAT-MAG-002), targets (BAT-MAG-003), the damage of
//! damage spells (BAT-MAG-004), immunities (BAT-MAG-005), Chain Lightning (BAT-MAG-006), area and
//! battle-wide spells (BAT-MAG-007) and magic resistance (BAT-MAG-008).
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

use h3_content::{
    Ability, CreatureTypeId, FULL_BP, MAX_FIELD_HEXES, MAX_SPELL_SCHOOLS, MAX_SPELLS, Ruleset,
    SINGLE_PRECISION_RATES, Spell, SpellCategory, SpellEffect, SpellId, SpellPolarity, SpellScope,
    SpellTarget, TargetSide,
};
use h3_contracts::{BattleEffect, BattleHero, BattleSide};
use h3_kernel::arith::{SINGLE_FRAC_BITS, mul_div_floor, round_single, single_precision};
use h3_kernel::{CanonicalHash, FixedVec, StateHasher};

use crate::combat::check_end;
use crate::damage::apply_damage;
use crate::error::{CommandError, HeroError};
use crate::event::{Event, EventKind, Sink};
use crate::geometry::{Field, Footprint, Hex};
use crate::hero::HeroRecord;
use crate::state::{BattleState, STACK_CAPACITY, StackId};

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
    /// More damage from every spell, basis points (Sorcery).
    pub all_bp: u32,
    /// More damage from the spells of each school, basis points, by school handle (the Orbs).
    pub school_bp: [u32; MAX_SPELL_SCHOOLS],
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
                BattleEffect::SpellDamage { scope, bp } => match scope {
                    SpellScope::All => {
                        out.all_bp = out.all_bp.saturating_add(bp.into());
                        continue;
                    }
                    SpellScope::School(l) => {
                        // Validated: the school exists, and there are at most four.
                        let i = l.index().min(MAX_SPELL_SCHOOLS - 1);
                        out.school_bp[i] = out.school_bp[i].saturating_add(bp.into());
                        continue;
                    }
                    SpellScope::Spell(s) => (s, u32::from(bp), 0, 0),
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

    /// The bonuses for `spell` (zero without any).
    fn spell(&self, spell: SpellId) -> SpellBonus {
        self.spells
            .iter()
            .find(|b| b.spell == spell)
            .copied()
            .unwrap_or_default()
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
        for effect in &hero.effects {
            if let BattleEffect::MagicResistance { bp } = *effect {
                let sum = resistance.saturating_add(rate(single, bp));
                resistance = if single { round_single(sum) } else { sum };
            }
        }
        Ok(Self {
            book,
            schools: hero.magic.schools,
            bonuses: SpellBonuses::of(hero).ok_or(HeroError::TooManySpellBonuses)?,
            resistance: resistance.min(scale(single)),
            cast: SpellSet::default(),
        })
    }
}

impl CanonicalHash for HeroMagic {
    /// Book bits, schools, the bonuses (every spell, by school, the single spells as length and
    /// spell, damage, growth per level, level), the resistance, the cast spells' bits.
    fn hash_into(&self, h: &mut StateHasher) {
        for word in self.book.bits().into_iter().chain(self.cast.bits()) {
            h.write_u64(word);
        }
        h.write_bytes(&self.schools);
        let b = &self.bonuses;
        h.write_u32(b.all_bp);
        for bp in b.school_bp {
            h.write_u32(bp);
        }
        h.write_len(b.spells.len());
        for s in &b.spells {
            s.spell.hash_into(h);
            h.write_u32(s.damage_bp);
            h.write_u32(s.per_level_bp);
            h.write_u8(s.level);
        }
        h.write_u64(self.resistance);
    }
}

/// What a cast is aimed at (ADR-0014 point 3; the second target of Teleport and Sacrifice
/// joins with those spells).
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
    /// `power × power factor + effect[mastery]` (BAT-MAG-004).
    value: u64,
    /// The caster's bonuses (BAT-MAG-004).
    bonuses: SpellBonuses,
}

/// Whether the battle implements every effect of `spell` (damage since E04b T2; spells without
/// effects are not castable yet).
fn implemented(spell: &Spell) -> bool {
    let effects = &spell.magic.effects;
    !effects.is_empty() && effects.iter().all(|e| matches!(e, SpellEffect::Damage))
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
/// meets the spell's conditions and is not immune to it by the spell's level, one of its schools
/// or the spell itself. (The Orb of Vulnerability and mind spells join with their tasks.)
// rule: BAT-MAG-005
fn affects(ruleset: &Ruleset, (id, spell): SpellRef<'_>, creature: CreatureTypeId) -> bool {
    let flags = ruleset
        .creature(creature)
        .map(|c| c.flags)
        .unwrap_or_default();
    spell.magic.conditions.admits(flags)
        && !ruleset.abilities(creature).iter().any(|a| match *a {
            Ability::SpellLevelImmunity { max_level } => spell.level <= max_level,
            Ability::SchoolImmunity { school } => spell.schools.contains(&school),
            Ability::SpellImmunity { spell } => spell == id,
            _ => false,
        })
}

/// Whether `side`'s cast of `spell` may reach stack `s` (BAT-MAG-003, BAT-MAG-005): a living
/// stack on the side the spell may reach that the spell can affect.
fn reachable(
    state: &BattleState,
    ruleset: &Ruleset,
    side: BattleSide,
    spell: SpellRef<'_>,
    s: StackId,
) -> bool {
    let stack = state.at(s);
    let side_ok = match spell.1.magic.side {
        TargetSide::Any => true,
        TargetSide::Enemy => stack.side != side,
        TargetSide::Own => stack.side == side,
    };
    stack.is_alive() && side_ok && affects(ruleset, spell, stack.creature)
}

/// Every stack id of the battle.
fn stack_ids(state: &BattleState) -> impl Iterator<Item = StackId> + use<> {
    (0u8..).take(state.stacks.len()).map(StackId)
}

/// The living stack covering `hex`, if any.
fn occupant(state: &BattleState, field: Field, hex: Hex) -> Option<StackId> {
    stack_ids(state).find(|&s| {
        let stack = state.at(s);
        stack.is_alive() && stack.footprint(field).covers(hex)
    })
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
        reachable(state, ruleset, side, spell, s)
            && in_area(field, hex, range, state.at(s).footprint(field))
    })
}

/// Whether `hex` is a target of `side`'s cast of a hex spell (BAT-MAG-003): a hex a stack may
/// stand on, whose stack (if any) the spell can reach, and whose area reaches a stack.
fn hex_target(
    state: &BattleState,
    ruleset: &Ruleset,
    side: BattleSide,
    (spell, mastery): (SpellRef<'_>, u8),
    hex: Hex,
) -> bool {
    let field = Field::new(ruleset.battle().field);
    field.is_standable(hex)
        && occupant(state, field, hex).is_none_or(|s| reachable(state, ruleset, side, spell, s))
        && area_stacks(state, ruleset, side, (spell, mastery), hex)
            .next()
            .is_some()
}

/// Checks that `target` is a target of `side`'s cast of `spell` at `mastery` (BAT-MAG-003): a
/// stack the spell can reach for a stack spell; for a hex spell a hex of [`hex_target`]; nothing
/// for a battle-wide spell, which must reach at least one stack.
// rule: BAT-MAG-003
fn check_target(
    state: &BattleState,
    ruleset: &Ruleset,
    side: BattleSide,
    (spell, mastery): (SpellRef<'_>, u8),
    target: CastTarget,
) -> Result<(), CommandError> {
    let ok = match (spell.1.magic.target, target) {
        (SpellTarget::Stack, CastTarget::Stack(s)) => {
            state.stack(s).is_some() && reachable(state, ruleset, side, spell, s)
        }
        (SpellTarget::Hex, CastTarget::Hex(h)) => {
            hex_target(state, ruleset, side, (spell, mastery), h)
        }
        (SpellTarget::None, CastTarget::None) => {
            stack_ids(state).any(|s| reachable(state, ruleset, side, spell, s))
        }
        _ => false,
    };
    if ok {
        Ok(())
    } else {
        Err(CommandError::InvalidSpellTarget)
    }
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
    target: CastTarget,
) -> Result<Cast<'r>, CommandError> {
    let (spell, hero, mastery, cost) = check_spell(state, ruleset, side, id)?;
    check_target(state, ruleset, side, ((id, spell), mastery), target)?;
    Ok(Cast {
        side,
        id,
        spell,
        cost,
        mastery,
        target,
        value: base_value(hero, spell, mastery),
        bonuses: hero.magic.bonuses,
    })
}

/// Whether a stack of `actor` awaits a command outside the tactics phase (when a hero may cast).
fn may_cast_now(state: &BattleState, actor: BattleSide) -> bool {
    state.tactics.is_none() && state.side_to_act() == Some(actor)
}

/// Fills `out` with every target [`crate::apply`] accepts for `actor`'s hero casting `spell` now
/// (BAT-MAG-001 to BAT-MAG-003): stacks by id, hexes by index, or none. Empty if the hero may
/// not cast the spell now.
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
    let field = Field::new(ruleset.battle().field);
    let hexes = (0u8..)
        .take(field.hex_count())
        .map(|h| CastTarget::Hex(Hex(h)));
    let stacks = stack_ids(state).map(CastTarget::Stack);
    for target in core::iter::once(CastTarget::None)
        .chain(stacks)
        .chain(hexes)
    {
        if check_target(state, ruleset, actor, ((spell, s), mastery), target).is_ok() {
            // At most `MAX_CAST_TARGETS` targets by construction.
            let _ = out.push(target);
        }
    }
}

/// The hexes at a distance in `(inner, radius)` from `centre`.
fn around(field: Field, centre: Hex, (inner, radius): (u32, u32)) -> impl Iterator<Item = Hex> {
    let (x, y) = field.coords(centre);
    let r = i32::try_from(radius).unwrap_or(i32::MAX);
    (y.saturating_sub(r)..=y.saturating_add(r))
        .flat_map(move |row| {
            (x.saturating_sub(r + 1)..=x.saturating_add(r + 1)).map(move |col| (col, row))
        })
        .filter_map(move |(col, row)| field.hex(col, row))
        .filter(move |&h| {
            let d = field.distance(centre, h);
            inner <= d && d <= radius
        })
}

/// Whether `actor`'s cast of `spell` at `mastery` has a target ([`cast_targets`] not empty,
/// computed without listing every hex): a hex spell needs a hex of [`hex_target`] near a stack
/// it can reach.
fn has_target(
    state: &BattleState,
    ruleset: &Ruleset,
    actor: BattleSide,
    (spell, mastery): (SpellRef<'_>, u8),
) -> bool {
    let mut stacks = stack_ids(state).filter(|&s| reachable(state, ruleset, actor, spell, s));
    if spell.1.magic.target != SpellTarget::Hex {
        return stacks.next().is_some();
    }
    let field = Field::new(ruleset.battle().field);
    let range = area(spell.1, mastery);
    stacks.any(|s| {
        state.at(s).footprint(field).hexes().any(|f| {
            around(field, f, range).any(|h| hex_target(state, ruleset, actor, (spell, mastery), h))
        })
    })
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

/// The damage of `cast` with the value `value` on a stack of `creature` (BAT-MAG-004), in the
/// original's order (`installation.md` §7.8): the caster's bonuses (× (1 + the best Orb among
/// the spell's schools) × (1 + Sorcery), kept fractional and rounded down once; then the
/// spell's own bonus of that, rounded down, and a specialist's `⌈per level × (hero level div
/// creature level) × that⌉`), then the target's (one spell's extra damage, rounded down; the
/// spell damage reduction, rounded down). The rates are exact, or single precision with quirk
/// Q-0005.
// rule: BAT-MAG-004
fn spell_damage(ruleset: &Ruleset, cast: &Cast<'_>, value: u64, creature: CreatureTypeId) -> u64 {
    let single = ruleset.quirks().is_enabled(SINGLE_PRECISION_RATES);
    let one = u128::from(scale(single));
    let b = &cast.bonuses;
    let bp = |sum: u32| rate(single, u16::try_from(sum).unwrap_or(u16::MAX));
    let school = cast
        .spell
        .schools
        .iter()
        .map(|l| b.school_bp[l.index().min(MAX_SPELL_SCHOOLS - 1)])
        .max()
        .unwrap_or(0);
    let factor = (one + u128::from(bp(school))) * (one + u128::from(bp(b.all_bp)));
    let t = u64::try_from(u128::from(value) * factor / (one * one)).unwrap_or(u64::MAX);
    let own = b.spell(cast.id);
    let level = u64::from(
        ruleset
            .creature_info(creature)
            .map_or(1, |c| c.level)
            .max(1),
    );
    let steps = u64::from(own.level) / level;
    let specialty = (u128::from(t) * u128::from(own.per_level_bp) * u128::from(steps))
        .div_ceil(u128::from(FULL_BP));
    let fixed = u128::from(t) * u128::from(own.damage_bp) / u128::from(FULL_BP);
    let mut damage = u64::try_from(u128::from(t) + specialty + fixed).unwrap_or(u64::MAX);
    for ability in ruleset.abilities(creature) {
        match *ability {
            Ability::SpellVulnerability { spell, percent } if spell == cast.id => {
                damage = grow(damage, u64::from(percent) * 100);
            }
            Ability::SpellDamageReduction { percent } => {
                damage = damage.saturating_mul(100 - u64::from(percent.min(100))) / 100;
            }
            _ => {}
        }
    }
    damage
}

/// The chance in percent that a cast of `spell` affects stack `s` (BAT-MAG-008): always for a
/// positive spell; otherwise the share the stack's creature lets through (1 − its magic
/// resistance), minus its side hero's resistance, at least 0, times 1 − the strongest magic
/// resistance aura of an adjacent allied stack, as a whole percentage rounded down. With quirk
/// Q-0005 the values are single precision and the difference is rounded to it, as the
/// original's x87 code computes it (`installation.md` §7.8): 10 % hero resistance leaves 89 %.
// rule: BAT-MAG-008
pub(crate) fn affect_chance(
    state: &BattleState,
    ruleset: &Ruleset,
    spell: &Spell,
    s: StackId,
) -> u32 {
    if spell.polarity == SpellPolarity::Positive {
        return 100;
    }
    let single = ruleset.quirks().is_enabled(SINGLE_PRECISION_RATES);
    let one = scale(single);
    let field = Field::new(ruleset.battle().field);
    let stack = state.at(s);
    let percent_rate = |percent: u8| rate(single, u16::from(100 - percent.min(100)) * 100);
    let creature = ruleset
        .abilities(stack.creature)
        .iter()
        .find_map(|a| match *a {
            Ability::MagicResistance { percent } => Some(percent_rate(percent)),
            _ => None,
        })
        .unwrap_or(one);
    let hero = state.heroes[stack.side.index()]
        .as_ref()
        .map_or(0, |h| h.magic.resistance);
    let share = creature.saturating_sub(hero);
    let share = if single { round_single(share) } else { share };
    let footprint = stack.footprint(field);
    let aura = stack_ids(state)
        .filter(|&n| n != s)
        .map(|n| state.at(n))
        .filter(|n| {
            n.is_alive() && n.side == stack.side && field.touches(n.footprint(field), footprint)
        })
        .flat_map(|n| ruleset.abilities(n.creature))
        .filter_map(|a| match *a {
            Ability::MagicResistanceAura { percent } => Some(percent),
            _ => None,
        })
        .max()
        .map_or(one, percent_rate);
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

/// Damages stack `s` with `cast` at the value `value` and reports it.
fn strike<S: Sink>(
    state: &mut BattleState,
    ruleset: &Ruleset,
    cast: &Cast<'_>,
    (s, value): (StackId, u64),
    sink: &mut S,
) {
    let damage = spell_damage(ruleset, cast, value, state.at(s).creature);
    let kills = apply_damage(state.at_mut(s), damage);
    sink.emit(Event::public(EventKind::SpellDamage {
        target: s,
        damage,
        kills,
    }));
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
            let stack = state.at(s);
            let candidate = stack.side == side
                && stack.is_alive()
                && !struck[s.index()]
                && affects(ruleset, spell, stack.creature);
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
    }));
    let field = Field::new(ruleset.battle().field);
    let spell = (cast.id, cast.spell);
    // A stack aimed at rolls first: if it resists, the whole spell does (BAT-MAG-007).
    let aimed = match cast.target {
        CastTarget::Stack(s) => Some(s),
        CastTarget::Hex(h) => occupant(state, field, h),
        CastTarget::None => None,
    };
    if let Some(s) = aimed
        && resists(state, ruleset, cast.spell, s)
    {
        resisted(s, sink);
    } else {
        match (cast.target, cast.spell.magic.chain) {
            (CastTarget::Stack(first), Some(chain)) => {
                // rule: BAT-MAG-006
                let mut struck = [false; STACK_CAPACITY];
                let mut value = cast.value;
                let mut current = first;
                for _ in 0..chain.targets[usize::from(cast.mastery)] {
                    strike(state, ruleset, &cast, (current, value), sink);
                    struck[current.index()] = true;
                    let Some(next) = next_in_chain(state, ruleset, spell, current, &struck) else {
                        break;
                    };
                    current = next;
                    value = value * u64::from(chain.factor_bp) / u64::from(FULL_BP);
                }
            }
            (CastTarget::Stack(s), None) => strike(state, ruleset, &cast, (s, cast.value), sink),
            (target, _) => {
                // rule: BAT-MAG-007
                let mut hits = [false; STACK_CAPACITY];
                for s in stack_ids(state) {
                    hits[s.index()] = match target {
                        CastTarget::Hex(h) => {
                            area_stacks(state, ruleset, cast.side, (spell, cast.mastery), h)
                                .any(|a| a == s)
                        }
                        _ => reachable(state, ruleset, cast.side, spell, s),
                    };
                }
                for s in stack_ids(state).filter(|s| hits[s.index()]) {
                    if resists(state, ruleset, cast.spell, s) {
                        resisted(s, sink);
                    } else {
                        strike(state, ruleset, &cast, (s, cast.value), sink);
                    }
                }
            }
        }
    }
    check_end(state, sink);
    if state.result.is_some() {
        state.active = None;
    }
}
