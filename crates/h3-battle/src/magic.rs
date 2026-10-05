//! A hero's magic in battle (E04b T2b; ADR-0014 points 2, 4 and 8): spell books, casting by a
//! hero (BAT-MAG-001), mastery and cost (BAT-MAG-002), targets (BAT-MAG-003) and the damage of
//! damage spells (BAT-MAG-004).
//!
//! A hero casts during the turn of one of its side's stacks that awaits a command, at most
//! `hero_casts_per_round` times per round and never by obligation; the stack still acts
//! afterwards. Casting is not part of [`crate::legal_commands`]: [`castable_spells`] and
//! [`cast_targets`] list what [`crate::apply`] accepts, spell first, then the target (the
//! factorized casting action of ADR-0014 point 9).

use h3_content::{
    CreatureTypeId, MAX_FIELD_HEXES, MAX_SPELL_SCHOOLS, MAX_SPELLS, Ruleset, Spell, SpellCategory,
    SpellEffect, SpellId, SpellScope, SpellTarget, TargetSide,
};
use h3_contracts::{BattleEffect, BattleHero, BattleSide};
use h3_kernel::arith::mul_div_floor;
use h3_kernel::{CanonicalHash, FixedVec, StateHasher};

use crate::combat::check_end;
use crate::damage::apply_damage;
use crate::error::{CommandError, HeroError};
use crate::event::{Event, EventKind, Sink};
use crate::geometry::Hex;
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
    /// More damage from the spell, basis points (Ciele's specialty).
    pub damage_bp: u32,
    /// A specialty's growth, basis points before the division by the target's creature level.
    pub specialty_bp: u32,
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
            let (spell, damage_bp, specialty_bp) = match *effect {
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
                    SpellScope::Spell(s) => (s, u32::from(bp), 0),
                },
                BattleEffect::SpellSpecialty { spell, bp } => (spell, 0, bp),
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
            bonus.specialty_bp = bonus.specialty_bp.saturating_add(specialty_bp);
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
    /// The distinct spells cast so far (public through the cast events; the outcome's).
    pub(crate) cast: SpellSet,
}

impl HeroMagic {
    /// The magic of `hero` (checked by `hero::check`).
    pub(crate) fn of(hero: &BattleHero) -> Result<Self, HeroError> {
        let mut book = SpellSet::default();
        for &spell in hero.magic.spell_book.as_deref().unwrap_or_default() {
            book.insert(spell);
        }
        Ok(Self {
            book,
            schools: hero.magic.schools,
            bonuses: SpellBonuses::of(hero).ok_or(HeroError::TooManySpellBonuses)?,
            cast: SpellSet::default(),
        })
    }
}

impl CanonicalHash for HeroMagic {
    /// Book bits, schools, the bonuses (every spell, by school, the single spells as length and
    /// spell, damage, specialty), the cast spells' bits.
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
            h.write_u32(s.specialty_bp);
        }
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

/// A cast that passed validation, with its damage computed.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Cast {
    side: BattleSide,
    spell: SpellId,
    cost: u16,
    target: StackId,
    damage: u64,
}

/// Whether the battle implements every effect and the targeting of `spell` (E04b T2b: damage
/// to one stack).
// stub: STUB-0001
fn implemented(spell: &Spell) -> bool {
    let m = &spell.magic;
    m.target == SpellTarget::Stack
        && m.chain.is_none()
        && !m.effects.is_empty()
        && m.effects.iter().all(|e| matches!(e, SpellEffect::Damage))
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
fn check_spell<'r>(
    state: &BattleState,
    ruleset: &'r Ruleset,
    side: BattleSide,
    id: SpellId,
) -> Result<(&'r Spell, HeroRecord, u8, u16), CommandError> {
    let hero = state.heroes[side.index()]
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
    let mastery = mastery(&hero, spell);
    let cost = spell.cost[usize::from(mastery)];
    if hero.view.mana < cost {
        return Err(CommandError::NotEnoughMana);
    }
    Ok((spell, hero, mastery, cost))
}

/// Checks that `target` is a target of `side`'s cast of `spell` (BAT-MAG-003): a living stack
/// on the side the spell may reach whose creature meets the spell's conditions.
// rule: BAT-MAG-003
fn check_target(
    state: &BattleState,
    ruleset: &Ruleset,
    side: BattleSide,
    spell: &Spell,
    target: CastTarget,
) -> Result<StackId, CommandError> {
    let CastTarget::Stack(id) = target else {
        return Err(CommandError::InvalidSpellTarget);
    };
    let stack = state
        .stack(id)
        .filter(|s| s.is_alive())
        .ok_or(CommandError::InvalidSpellTarget)?;
    let side_ok = match spell.magic.side {
        TargetSide::Any => true,
        TargetSide::Enemy => stack.side != side,
        TargetSide::Own => stack.side == side,
    };
    let flags = ruleset
        .creature(stack.creature)
        .map(|c| c.flags)
        .unwrap_or_default();
    if side_ok && spell.magic.conditions.admits(flags) {
        Ok(id)
    } else {
        Err(CommandError::InvalidSpellTarget)
    }
}

/// Validates `side`'s cast of `spell` at `target`; the caller has checked that a stack of
/// `side` awaits a command outside the tactics phase.
pub(crate) fn plan_cast(
    state: &BattleState,
    ruleset: &Ruleset,
    side: BattleSide,
    spell: SpellId,
    target: CastTarget,
) -> Result<Cast, CommandError> {
    let (s, hero, mastery, cost) = check_spell(state, ruleset, side, spell)?;
    let target = check_target(state, ruleset, side, s, target)?;
    let victim = state.at(target).creature;
    Ok(Cast {
        side,
        spell,
        cost,
        target,
        damage: spell_damage(ruleset, &hero, (spell, s, mastery), victim),
    })
}

/// Whether a stack of `actor` awaits a command outside the tactics phase (when a hero may cast).
fn may_cast_now(state: &BattleState, actor: BattleSide) -> bool {
    state.tactics.is_none() && state.side_to_act() == Some(actor)
}

/// Fills `out` with every target [`crate::apply`] accepts for `actor`'s hero casting `spell` now
/// (BAT-MAG-001 to BAT-MAG-003): stacks by id (later spells add none and hexes). Empty if the
/// hero may not cast the spell now.
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
    let Ok((s, ..)) = check_spell(state, ruleset, actor, spell) else {
        return;
    };
    for id in (0u8..).take(state.stacks.len()).map(StackId) {
        let target = CastTarget::Stack(id);
        if check_target(state, ruleset, actor, s, target).is_ok() {
            // At most `MAX_CAST_TARGETS` targets by construction.
            let _ = out.push(target);
        }
    }
}

/// The spells `actor`'s hero may cast now: those with at least one target in
/// [`cast_targets`] (empty if it is not `actor`'s turn, in the tactics phase or without a hero).
pub fn castable_spells(state: &BattleState, ruleset: &Ruleset, actor: BattleSide) -> SpellSet {
    let mut out = SpellSet::default();
    let Some(hero) = state.heroes[actor.index()] else {
        return out;
    };
    let mut targets = CastTargets::new();
    for spell in hero.magic.book.iter() {
        cast_targets(state, ruleset, actor, spell, &mut targets);
        if !targets.is_empty() {
            out.insert(spell);
        }
    }
    out
}

/// `value` grown by `bp` basis points of itself, rounded down (saturating).
fn grow(value: u64, bp: u32) -> u64 {
    mul_div_floor(value, 10_000 + u64::from(bp), 10_000).unwrap_or(u64::MAX)
}

/// The damage of `hero`'s cast of `spell` at `mastery` on a stack of `creature`
/// (BAT-MAG-004): `power × power factor + effect[mastery]`, then the caster's bonuses in VCMI's
/// order, each rounded down: every spell (Sorcery), this spell (Ciele), the best of the spell's
/// schools (the Orbs), then a specialty's growth divided by the creature's level.
// rule: BAT-MAG-004
fn spell_damage(
    ruleset: &Ruleset,
    hero: &HeroRecord,
    (id, spell, mastery): (SpellId, &Spell, u8),
    creature: CreatureTypeId,
) -> u64 {
    let value = u64::from(hero.view.power) * u64::from(spell.power)
        + u64::from(spell.effect[usize::from(mastery)]);
    let bonuses = &hero.magic.bonuses;
    let own = bonuses.spell(id);
    let school = spell
        .schools
        .iter()
        .map(|l| bonuses.school_bp[l.index().min(MAX_SPELL_SCHOOLS - 1)])
        .max()
        .unwrap_or(0);
    let damage = grow(grow(grow(value, bonuses.all_bp), own.damage_bp), school);
    // Creature levels are at least 1 (content validation).
    let level = u64::from(
        ruleset
            .creature_info(creature)
            .map_or(1, |c| c.level)
            .max(1),
    );
    mul_div_floor(
        damage,
        10_000 * level + u64::from(own.specialty_bp),
        10_000 * level,
    )
    .unwrap_or(u64::MAX)
}

/// Applies a validated cast: the hero pays the cost and counts the cast, then the spell takes
/// effect (BAT-MAG-004); the battle may end. The active stack keeps its turn.
pub(crate) fn apply_cast<S: Sink>(state: &mut BattleState, cast: Cast, sink: &mut S) {
    // Validated: the side has a hero with the mana.
    for hero in state.heroes[cast.side.index()].iter_mut() {
        hero.view.mana -= cast.cost;
        hero.view.casts += 1;
        hero.magic.cast.insert(cast.spell);
    }
    sink.emit(Event::public(EventKind::SpellCast {
        side: cast.side,
        spell: cast.spell,
        target: CastTarget::Stack(cast.target),
    }));
    let kills = apply_damage(state.at_mut(cast.target), cast.damage);
    sink.emit(Event::public(EventKind::SpellDamage {
        target: cast.target,
        damage: cast.damage,
        kills,
    }));
    check_end(state, sink);
    if state.result.is_some() {
        state.active = None;
    }
}
