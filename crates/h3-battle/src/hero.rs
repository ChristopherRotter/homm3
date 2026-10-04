//! The heroes leading the sides (ADR-0013; BAT-HER-001): what the battle checks of a battle
//! hero when it starts and the part of it the battle keeps.
//!
//! The battle takes a hero as given and checks only what its rules need to run soundly
//! (OPEN-QUESTIONS Q31): references of the ruleset, the size of the effects list, and values in
//! the ranges where the rules are defined. Values no real hero could have (attack 200) are
//! accepted; keeping a hero within the game's limits is the derivation's job (HER-BAT-001).

use h3_content::{CreatureSelector, FULL_BP, HeroTypeId, MAX_SKILL_LEVEL, Ruleset, SpellCategory};
use h3_contracts::{
    BallistaControl, BattleEffect, BattleHero, BattleSide, DamageBonuses, MAX_BATTLE_EFFECTS,
    TentControl,
};
use h3_kernel::{CanonicalHash, StateHasher};

use crate::error::{HeroError, SetupError};

/// What both sides see of a side's hero (ADR-0013 point 9): identity, primary skills and mana.
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
}

/// The part of a side's hero the battle keeps in its state. Later tasks add what their rules
/// read (damage bonuses, war-machine control, …) together with the rule.
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
}

/// Whether `side` may retreat (BAT-FLE-001): it has a hero, and no hero in the battle has a
/// `NoRetreat` effect (also against a side without a hero, batch 1b B6).
// rule: BAT-FLE-001
pub(crate) fn may_retreat(heroes: &[Option<HeroRecord>; 2], side: BattleSide) -> bool {
    heroes[side.index()].is_some() && !heroes.iter().flatten().any(|h| h.no_retreat)
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
        let d = &self.damage;
        for value in [d.ranged_bp, d.melee_bp, d.reduction_bp] {
            h.write_u16(value);
        }
        h.write_u8(u8::from(self.ballista.is_some()));
        if let Some(b) = self.ballista {
            h.write_u8(b.shots);
            h.write_u16(b.double_damage_chance_bp);
        }
        h.write_u8(u8::from(self.tent.is_some()));
        if let Some(t) = self.tent {
            h.write_u32(t.heal_bonus_bp);
        }
        h.write_u8(u8::from(self.no_retreat));
    }
}

/// Checks `side`'s hero for soundness and returns what the battle keeps of it.
// rule: BAT-HER-001
pub(crate) fn check(
    ruleset: &Ruleset,
    side: BattleSide,
    hero: &BattleHero,
) -> Result<HeroRecord, SetupError> {
    let invalid = |error| SetupError::InvalidHero { side, error };
    if ruleset.hero_type(hero.hero_type).is_none() {
        return Err(invalid(HeroError::UnknownHeroType));
    }
    if hero.tactics > MAX_SKILL_LEVEL {
        return Err(invalid(HeroError::TacticsLevel));
    }
    if hero.damage.reduction_bp > FULL_BP {
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
    let magic = &hero.magic;
    if magic.schools.iter().any(|&level| level > MAX_SKILL_LEVEL) {
        return Err(invalid(HeroError::SchoolLevel));
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
        },
        tactics: hero.tactics,
        damage: hero.damage,
        ballista: hero.war_machines.ballista,
        tent: hero.war_machines.tent,
        no_retreat: hero.effects.contains(&BattleEffect::NoRetreat),
    })
}

/// The creature type an effect names, if any.
fn creature_of(effect: &BattleEffect) -> Option<h3_content::CreatureTypeId> {
    let selector = match effect {
        BattleEffect::StackStat { creatures, .. }
        | BattleEffect::Regeneration { creatures, .. }
        | BattleEffect::NoRangePenalty { creatures }
        | BattleEffect::ShootWhenAdjacent { creatures } => creatures,
        BattleEffect::Morale { .. }
        | BattleEffect::Luck { .. }
        | BattleEffect::MoraleAtMost { .. }
        | BattleEffect::LuckAtMost { .. }
        | BattleEffect::NoRetreat => return None,
    };
    match selector {
        CreatureSelector::Creature(c) => Some(*c),
        CreatureSelector::All | CreatureSelector::Flag(_) => None,
    }
}
