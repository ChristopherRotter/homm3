//! The heroes leading the sides (ADR-0013; BAT-HER-001): what the battle checks of a battle
//! hero when it starts and the part of it the battle keeps.
//!
//! The battle takes a hero as given and checks only what its rules need to run soundly
//! (OPEN-QUESTIONS Q31): references of the ruleset, the size of the effects list, and values in
//! the ranges where the rules are defined. Values no real hero could have (attack 200) are
//! accepted; keeping a hero within the game's limits is the derivation's job (HER-BAT-001).

use core::num::NonZeroU32;

use h3_content::{
    CreatureFlag, CreatureSelector, CreatureStat, CreatureType, CreatureTypeId, FULL_BP,
    HeroTypeId, MAX_SKILL_LEVEL, Ruleset, SINGLE_PRECISION_RATES, SpellCategory, SpellScope,
};
use h3_contracts::{
    BallistaControl, BattleEffect, BattleHero, BattleSide, DamageBonuses, MAX_BATTLE_EFFECTS, Rate,
    StatOp, TentControl,
};
use h3_kernel::{CanonicalHash, StateHasher};

use crate::error::{HeroError, SetupError};
use crate::magic::{HeroMagic, Masteries, SpellBonuses, SpellSet};
use crate::state::StackTraits;

/// What both sides see of a side's hero (ADR-0013 point 9, ADR-0014 point 8): identity, primary
/// skills, mana, its casts this round and which war machines the hero commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeroView {
    /// The hero type.
    pub hero_type: HeroTypeId,
    /// Attack.
    pub attack: u8,
    /// Defense.
    pub defense: u8,
    /// Spell power.
    pub power: u8,
    /// Knowledge.
    pub knowledge: u8,
    /// Spell points.
    pub mana: u16,
    /// Maximum spell points.
    pub max_mana: u16,
    /// Spells cast this round (BAT-MAG-001).
    pub casts: u8,
    /// The hero commands its side's Ballista (BAT-WAR-006); how it shoots is private.
    pub commands_ballista: bool,
    /// The hero commands its side's First Aid Tent (BAT-WAR-007); how it heals is private.
    pub commands_tent: bool,
    /// The hero carries a spell book (public, ADR-0014 point 8: the Master Genie's test reads it,
    /// BAT-MAG-050); the spells in it are private.
    pub spell_book: bool,
}

/// What only the hero's own side sees of it (ADR-0013 point 9, ADR-0014 point 8;
/// `Observation::own_hero`): the values of its skills, its surrender discount, its funds and the
/// price of its surrender, its spell book, spell masteries and spell bonuses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OwnHeroView {
    /// Tactics level, 0..=3 (BAT-TAC-001).
    pub tactics: u8,
    /// Damage bonuses of the side's strikes and the reduction of strikes against it
    /// (BAT-HER-004).
    pub damage: DamageBonuses,
    /// How the commanded Ballista shoots (BAT-WAR-006).
    pub ballista: Option<BallistaControl>,
    /// How the commanded First Aid Tent heals (BAT-WAR-007).
    pub tent: Option<TentControl>,
    /// Discount on the price of the side's surrender, basis points (BAT-FLE-002).
    pub surrender_discount_bp: u16,
    /// What the side's owner can pay for a surrender (`BattleSetup.funds`).
    pub funds: u32,
    /// The price of the side's surrender now, if it may surrender at all (BAT-FLE-002); it may
    /// surrender now if its funds cover the price.
    pub surrender_price: Option<u32>,
    /// The spells in its spell book (none without a book; BAT-MAG-001).
    pub spell_book: SpellSet,
    /// Its mastery of each spell (BAT-MAG-002).
    pub masteries: Masteries,
    /// Its spell-damage bonuses (BAT-MAG-004).
    pub spell_bonuses: SpellBonuses,
}

/// The part of a side's hero the battle keeps in its state. Later tasks add what their rules
/// read (damage bonuses, war-machine control, …) together with the rule. The state hash covers
/// the view's machine flags through `ballista` and `tent`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HeroRecord {
    /// The public part.
    pub(crate) view: HeroView,
    /// Tactics level, 0..=3 (private to the side; the tactics phase it gives is public,
    /// BAT-TAC-001).
    pub(crate) tactics: u8,
    /// Damage bonuses of the side's strikes and reduction of the strikes against it
    /// (BAT-HER-004; private to the side).
    pub(crate) damage: DamageBonuses,
    /// The side commands its Ballista (Artillery, BAT-WAR-006; private to the side).
    pub(crate) ballista: Option<BallistaControl>,
    /// The side commands its First Aid Tent (First Aid, BAT-WAR-007; private to the side).
    pub(crate) tent: Option<TentControl>,
    /// The hero has a `NoRetreat` effect (Shackles of War): no side may retreat or surrender
    /// (BAT-FLE-001; public through the sides' options).
    pub(crate) no_retreat: bool,
    /// Discount on the price of the side's surrender, basis points (BAT-FLE-002; private).
    pub(crate) surrender_discount_bp: u16,
    /// What the side's owner can pay for a surrender (`BattleSetup.funds`, BAT-FLE-002; private).
    pub(crate) funds: u32,
    /// Spell book, masteries, spell bonuses and the spells cast (E04b; BAT-MAG-001 to 004).
    pub(crate) magic: HeroMagic,
}

/// Whether `side` may retreat (BAT-FLE-001): it has a hero, and no hero in the battle has a
/// `NoRetreat` effect (also against a side without a hero, batch 1b B6).
// rule: BAT-FLE-001
pub(crate) fn may_retreat(heroes: &[Option<HeroRecord>; 2], side: BattleSide) -> bool {
    heroes[side.index()].is_some() && !heroes.iter().flatten().any(|h| h.no_retreat)
}

/// The changes a hero's `StackStat` effects make to the statistics of a stack of creature `id`
/// (BAT-HER-005), in the order of [`CreatureStat::ALL`]: every effect whose creatures include `id`
/// adds its amount, or its share in basis points of the creature's own value (rounded up for
/// `PercentBp`, down for `PercentBpDown`). War machines keep their speed (batch 1b R8).
// rule: BAT-HER-005
pub(crate) fn stat_changes(
    ruleset: &Ruleset,
    hero: Option<&BattleHero>,
    (id, creature): (CreatureTypeId, &CreatureType),
) -> [i64; 6] {
    let own = |stat| match stat {
        CreatureStat::Attack => i64::from(creature.attack),
        CreatureStat::Defense => i64::from(creature.defense),
        CreatureStat::Speed => i64::from(creature.speed),
        CreatureStat::HitPoints => i64::from(creature.hit_points),
        CreatureStat::MinDamage => i64::from(creature.damage_min),
        CreatureStat::MaxDamage => i64::from(creature.damage_max),
    };
    let machine = creature.flags.contains(CreatureFlag::WarMachine);
    let mut changes = [0i64; 6];
    let effects = hero.map(|h| h.effects.as_slice()).unwrap_or_default();
    for effect in effects {
        let BattleEffect::StackStat {
            creatures,
            stat,
            op,
            amount,
        } = *effect
        else {
            continue;
        };
        if !creatures.matches(ruleset, id) || (machine && stat == CreatureStat::Speed) {
            continue;
        }
        let amount = i64::from(amount);
        let bp = i64::from(FULL_BP);
        let change = match op {
            StatOp::Add => amount,
            StatOp::PercentBp => -(-own(stat) * amount).div_euclid(bp),
            StatOp::PercentBpDown => (own(stat) * amount).div_euclid(bp),
        };
        // `CreatureStat::ALL` lists every statistic.
        let i = CreatureStat::ALL
            .iter()
            .position(|&s| s == stat)
            .unwrap_or_default();
        changes[i] += change;
    }
    changes
}

/// A stack's traits and regeneration with its hero's trait effects (BAT-HER-006): for creature
/// `id`, a matching `NoRangePenalty` adds the trait `NO_DISTANCE_PENALTY`, a matching
/// `ShootWhenAdjacent` the trait `SHOOTS_WHEN_ADJACENT`, and the regeneration is the most hit
/// points of the matching `Regeneration` effects (0 without one).
// rule: BAT-HER-006
pub(crate) fn trait_effects(
    ruleset: &Ruleset,
    hero: Option<&BattleHero>,
    id: CreatureTypeId,
    mut traits: StackTraits,
) -> (StackTraits, u16) {
    let mut regeneration = 0;
    let effects = hero.map(|h| h.effects.as_slice()).unwrap_or_default();
    for effect in effects {
        match *effect {
            BattleEffect::Regeneration {
                creatures,
                hit_points,
            } if creatures.matches(ruleset, id) => regeneration = regeneration.max(hit_points),
            BattleEffect::NoRangePenalty { creatures } if creatures.matches(ruleset, id) => {
                traits = traits.with(StackTraits::NO_DISTANCE_PENALTY, true);
            }
            BattleEffect::ShootWhenAdjacent { creatures } if creatures.matches(ruleset, id) => {
                traits = traits.with(StackTraits::SHOOTS_WHEN_ADJACENT, true);
            }
            _ => {}
        }
    }
    (traits, regeneration)
}

/// The damage bonuses of `side` (none without a hero; BAT-HER-004).
pub(crate) fn damage(heroes: &[Option<HeroRecord>; 2], side: BattleSide) -> DamageBonuses {
    heroes[side.index()].map(|h| h.damage).unwrap_or_default()
}

/// How `side`'s hero commands its Ballista, if it does (BAT-WAR-006).
pub(crate) fn ballista(
    heroes: &[Option<HeroRecord>; 2],
    side: BattleSide,
) -> Option<BallistaControl> {
    heroes[side.index()].and_then(|h| h.ballista)
}

/// How `side`'s hero commands its First Aid Tent, if it does (BAT-WAR-007).
pub(crate) fn tent(heroes: &[Option<HeroRecord>; 2], side: BattleSide) -> Option<TentControl> {
    heroes[side.index()].and_then(|h| h.tent)
}

impl CanonicalHash for HeroRecord {
    fn hash_into(&self, h: &mut StateHasher) {
        let v = &self.view;
        v.hero_type.hash_into(h);
        for value in [v.attack, v.defense, v.power, v.knowledge, self.tactics] {
            h.write_u8(value);
        }
        h.write_u16(v.mana);
        h.write_u16(v.max_mana);
        h.write_u8(v.casts);
        let d = &self.damage;
        for r in [d.ranged, d.melee, d.reduction] {
            hash_rate(h, r);
        }
        h.write_u8(u8::from(self.ballista.is_some()));
        if let Some(b) = self.ballista {
            h.write_u8(b.shots);
            h.write_u16(b.double_damage_chance_bp);
        }
        h.write_u8(u8::from(self.tent.is_some()));
        if let Some(t) = self.tent {
            hash_rate(h, t.heal_bonus);
        }
        h.write_u8(u8::from(self.no_retreat));
        h.write_u16(self.surrender_discount_bp);
        h.write_u32(self.funds);
        self.magic.hash_into(h);
        h.write_bool(v.spell_book);
    }
}

/// Hashes one of a hero's rates: its basis points, then its single-precision value's bits (0
/// without; state hash v23).
pub(crate) fn hash_rate(h: &mut StateHasher, r: Rate) {
    h.write_u32(r.bp);
    h.write_u32(r.single.map_or(0, NonZeroU32::get));
}

/// Checks `side`'s hero for soundness and returns what the battle keeps of it, with the side's
/// `funds`.
// rule: BAT-HER-001
pub(crate) fn check(
    ruleset: &Ruleset,
    side: BattleSide,
    hero: &BattleHero,
    funds: u32,
) -> Result<HeroRecord, SetupError> {
    let invalid = |error| SetupError::InvalidHero { side, error };
    if ruleset.hero_type(hero.hero_type).is_none() {
        return Err(invalid(HeroError::UnknownHeroType));
    }
    if hero.tactics > MAX_SKILL_LEVEL {
        return Err(invalid(HeroError::TacticsLevel));
    }
    let reduction = hero.damage.reduction;
    // Positive binary32 patterns order as their values; 0x3f80_0000 is 1.
    if reduction.bp > FULL_BP.into() || reduction.single.is_some_and(|b| b.get() > 0x3f80_0000) {
        return Err(invalid(HeroError::DamageReduction));
    }
    if hero
        .war_machines
        .ballista
        .is_some_and(|b| b.shots == 0 || b.double_damage_chance_bp > FULL_BP)
    {
        return Err(invalid(HeroError::BallistaControl));
    }
    if hero.surrender_discount_bp > FULL_BP {
        return Err(invalid(HeroError::SurrenderDiscount));
    }
    if hero.effects.len() > MAX_BATTLE_EFFECTS {
        return Err(invalid(HeroError::TooManyEffects));
    }
    if hero
        .effects
        .iter()
        .filter_map(creature_of)
        .any(|c| ruleset.creature(c).is_none())
    {
        return Err(invalid(HeroError::UnknownCreature));
    }
    let faction_count = ruleset.factions().len();
    if hero
        .effects
        .iter()
        .filter_map(|e| match e {
            BattleEffect::AlignmentGroup { factions } => Some(factions.factions()),
            _ => None,
        })
        .flatten()
        .any(|f| f.index() >= faction_count)
    {
        return Err(invalid(HeroError::UnknownFaction));
    }
    let magic = &hero.magic;
    let school_count = ruleset.spell_schools().len();
    let unknown_school = hero.effects.iter().any(|e| {
        matches!(e, BattleEffect::SpellDamage { scope: SpellScope::School(l), .. }
            if l.index() >= school_count)
    });
    if unknown_school {
        return Err(invalid(HeroError::UnknownSchool));
    }
    for (i, &(id, level)) in magic.masteries.iter().enumerate() {
        ruleset
            .spell(id)
            .ok_or_else(|| invalid(HeroError::UnknownSpell))?;
        if magic
            .masteries
            .get(..i)
            .unwrap_or_default()
            .iter()
            .any(|&(s, _)| s == id)
        {
            return Err(invalid(HeroError::DuplicateSpell));
        }
        if !(1..=MAX_SKILL_LEVEL).contains(&level) {
            return Err(invalid(HeroError::MasteryLevel));
        }
    }
    if hero.effects.iter().any(|e| match *e {
        BattleEffect::SpellDamage {
            scope: SpellScope::Spell(id),
            ..
        }
        | BattleEffect::SpellSpecialty { spell: id, .. }
        | BattleEffect::SpellImmunity { spell: id } => ruleset.spell(id).is_none(),
        _ => false,
    }) {
        return Err(invalid(HeroError::UnknownSpell));
    }
    let book = magic.spell_book.as_deref().unwrap_or_default();
    for (i, &id) in book.iter().enumerate() {
        let spell = ruleset
            .spell(id)
            .ok_or_else(|| invalid(HeroError::UnknownSpell))?;
        if book.get(..i).unwrap_or_default().contains(&id) {
            return Err(invalid(HeroError::DuplicateSpell));
        }
        if spell.category == SpellCategory::Ability {
            return Err(invalid(HeroError::NotAHeroSpell));
        }
    }
    Ok(HeroRecord {
        view: HeroView {
            hero_type: hero.hero_type,
            attack: hero.attack,
            defense: hero.defense,
            power: hero.power,
            knowledge: hero.knowledge,
            mana: magic.mana,
            max_mana: magic.max_mana,
            casts: 0,
            commands_ballista: hero.war_machines.ballista.is_some(),
            commands_tent: hero.war_machines.tent.is_some(),
            spell_book: hero.magic.spell_book.is_some(),
        },
        tactics: hero.tactics,
        damage: hero.damage,
        ballista: hero.war_machines.ballista,
        tent: hero.war_machines.tent,
        no_retreat: hero.effects.contains(&BattleEffect::NoRetreat),
        surrender_discount_bp: hero.surrender_discount_bp,
        funds,
        magic: HeroMagic::of(hero, ruleset.quirks().is_enabled(SINGLE_PRECISION_RATES))
            .map_err(invalid)?,
    })
}

/// The creature type an effect names, if any.
fn creature_of(effect: &BattleEffect) -> Option<CreatureTypeId> {
    let selector = match effect {
        BattleEffect::StackStat { creatures, .. }
        | BattleEffect::Regeneration { creatures, .. }
        | BattleEffect::NoRangePenalty { creatures }
        | BattleEffect::ShootWhenAdjacent { creatures } => creatures,
        BattleEffect::Morale { .. }
        | BattleEffect::Luck { .. }
        | BattleEffect::MoraleAtMost { .. }
        | BattleEffect::LuckAtMost { .. }
        | BattleEffect::NoRetreat
        | BattleEffect::AlignmentGroup { .. }
        | BattleEffect::SpellDamage { .. }
        | BattleEffect::SpellSpecialty { .. }
        | BattleEffect::MagicResistance { .. }
        | BattleEffect::SpellDuration { .. }
        | BattleEffect::NoCasting
        | BattleEffect::SpellLevelLimit { .. }
        | BattleEffect::SpellImmunity { .. }
        | BattleEffect::SpellLevelImmunity { .. }
        | BattleEffect::MindImmunity
        | BattleEffect::NegateImmunities => return None,
    };
    match selector {
        CreatureSelector::Creature(c) => Some(*c),
        CreatureSelector::All | CreatureSelector::Flag(_) => None,
    }
}
