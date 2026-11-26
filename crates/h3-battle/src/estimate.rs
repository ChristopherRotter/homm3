//! The computer's simulated combat (E31 T2b and T2c; AI-Q34; survey
//! `docs/rules/surveys/E31-computer-player.md` §2.2): the original's estimate of a fight in
//! army-value units, not hit points, with no battle. The adventure's computer player
//! (`h3_game::computer`'s fight estimate, `ai_army_fight_sim` 0x427330) calls it; the quick
//! battle between computers will too (ADR-0030 point 6). Its rules are the computer's (`CPU-SIM`,
//! `docs/rules/catalog/computer.toml`); it lives here because its casts take the battle's own
//! spell rules (the creature's part of the effect test, the target's damage factors, the caster's
//! bonuses, the mastery on a field, the cost; ADR-0027 point 6's pattern).
//!
//! Read from `sim_combat_start` 0x423EE0, `sim_side_build` 0x424120 and `sim_side_walls` 0x424790
//! (a side, CPU-SIM-001), the losses (`sim_spread_losses` 0x426170, `sim_side_take_damage`
//! 0x426300, `sim_side_wipe` 0x4262B0, `sim_apply_losses` 0x424880; CPU-SIM-002), the damage and
//! the stand-off (`sim_side_damage` 0x426390, `sim_side_hold_value` 0x426450, `sim_combat_finish`
//! 0x4264D0; CPU-SIM-003), the engagement's lookahead (`sim_engage_now` 0x4267C0; CPU-SIM-004),
//! the run (`sim_combat_run` 0x426BC0; CPU-SIM-005) and the casts (`sim_cast_spell` 0x425BD0 and
//! its values: CPU-SIM-006 to -010). The stack vector's container code (0x4276C0, 0x427750,
//! 0x427780, the quicksort 0x427A90 and 0x427C30 that seven slots never reach) is a `Vec`; the
//! build's insertion sort a sort by unit value with the slot as its tie-breaker (it is stable,
//! the stacks built in slot order).
//!
//! Beside it the computer's values of a hero's spells (`caster`, E31 T2e, AI-Q36): the adventure
//! computer's caster record and spell values ([`CasterRecord`], CPU-SPL-001 to -006), which the
//! battle AI's odds read too; and the value of a morale or luck change (`morale`, E31 T2f,
//! AI-Q37: [`change_value`], BAT-AI-016), which the battle AI's evaluators and the computer's
//! artifact values read.
//!
//! Every value is the original's x87 arithmetic (`X87`, `SignedX87`: 53 bits, each sum truncated
//! by `crt_ftol` where the code truncates, the f32 stores) and its 32-bit integers, wrapping where
//! the code's `imul`, `add` and `sub` wrap. Spells are told by their data as the original tells
//! them by their numbers (the AI kinds, effects, chains and areas; `SpellAi`), as
//! `h3_battle_ai`'s evaluators do.

use alloc::sync::Arc;
use alloc::vec::Vec;
use core::num::NonZeroU32;

use h3_content::{
    CreatureFlag, CreatureSelector, CreatureStat, CreatureTypeId, FULL_BP, SINGLE_PRECISION_RATES,
    SpellCategory, SpellId,
};
use h3_content_battle::{
    Ability, BattleRuleset, BattleSpell, BattlefieldId, SpellAiKind, SpellEffect, SpellPolarity,
    TimedChange,
};
use h3_contracts::{ArmyStack, BattleEffect, BattleHero, FortLevel, StatOp};
use h3_kernel::arith::{DOUBLE_BITS, SINGLE_BITS, SignedX87, X87};
use h3_kernel::order::sort_by_unique_key;

use crate::computer::takes_stack;
use crate::magic::{self, Guard, HeroMagic, SpellRef, SpellSet};
use crate::mana;

mod caster;
mod morale;
#[cfg(test)]
mod tests;

pub use caster::CasterRecord;
pub use morale::{LIMIT as MORALE_LIMIT, change_value, luck_values, morale_values};
pub(crate) use caster::{SpellCaster, best as best_spell_value};

/// The rounds a simulated combat fights before its stand-off (`sim_combat_run`'s 1 to 3).
const ROUNDS: i32 = 3;

/// The most rounds to contact (`sim_side_build`'s cap) and the lookahead's latest engagement
/// round (`sim_engage_now`'s first): a side "fights with all its stacks" at this round.
const FAR: i32 = 4;

/// The army slots a side's build reads (`sim_side_build`'s seven).
const SLOTS: u8 = 7;

/// The highest Spell Power the casts count (`sim_cast_spell`'s clamp to 1..99).
const MAX_POWER: u8 = 99;

/// The rounds of a timed spell the other spells' values count at most (0x423C9C's 5).
const MAX_ROUNDS: i32 = 5;

/// The hops of a chain after its first stack, each at half the damage (0x424E69's three).
const CHAIN_HOPS: usize = 3;

/// The highest mastery whose Dispel the original takes for a single stack (0x42567D, 0x425727:
/// Advanced; at Expert both sides' stacks count).
const ADVANCED: u8 = 2;

/// What the simulated combat reads of a side's hero: the hero as a battle reads it and its
/// available spells.
#[derive(Debug, Clone, Copy)]
pub struct EstimateHero<'a> {
    /// The hero (`h3_heroes::derive_battle_hero` with no visit bonuses: none of the reads takes
    /// one).
    pub hero: &'a BattleHero,
    /// Its available spells (`h3_heroes::available_spells`: the record's +0x430 bytes), with or
    /// without a spell book: the walls' Earthquake and the casts read them.
    pub spells: &'a [SpellId],
}

/// The battle tile as the simulated combat reads it (`sim_combat_start` 0x423EE0's
/// `tile_special_ground` 0x4FCE20): the special ground's battlefield and an anti-magic garrison.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Ground {
    /// The special ground's battlefield, if the tile has one (its highest spell level, Cursed
    /// Ground's 1, and the schools it casts at Expert).
    pub battlefield: Option<BattlefieldId>,
    /// Whether an anti-magic garrison stands there: no side casts.
    pub no_casting: bool,
}

/// What a simulated side reads of a hero (the record's bytes `sim_side_build` 0x424120 and
/// `sim_side_walls` 0x424790 read, and the three hero functions they call), from the battle hero.
#[derive(Debug, Clone, PartialEq, Eq)]
struct HeroReads {
    /// Its effective Attack (+0x476), a signed byte the build clamps to 0..99.
    attack: u8,
    /// Its effective Defense (+0x477), as Attack.
    defense: u8,
    /// Its Tactics level (+0xDC).
    tactics: u8,
    /// Its Ballistics level (+0xD3): its Catapult control's, 0 without.
    ballistics: u8,
    /// Earthquake among its available spells (+0x43E): a spell with the `Earthquake` effect.
    earthquake: bool,
    /// `hero_archery` 0x4E4160: the Archery bonus as a single-precision value, the battle hero's
    /// ranged damage rate (the skill grown by its specialty, the bows that need the skill; 0
    /// without it).
    archery: X87,
    /// `hero_creature_speed_bonus` 0x4E5AA0: the sum of its speed changes that select every
    /// creature (Necklace of Swiftness, Ring of the Wayfarer, Cape of Velocity, a speed
    /// specialty); a creature specialty's speed is not one.
    speed: i32,
    /// `hero_creature_health_bonus` 0x4E5B80's inputs: its hit-point changes, each with the
    /// creatures it selects (Ring of Vitality, Ring of Life, Vial of Lifeblood; Elixir of Life's
    /// quarter of a living creature's hit points).
    hit_points: Vec<(CreatureSelector<CreatureTypeId>, StatOp, i16)>,
}

impl HeroReads {
    /// What a side reads of `hero`.
    fn of(hero: EstimateHero<'_>, ruleset: &BattleRuleset) -> Self {
        let EstimateHero { hero, spells } = hero;
        let earthquake = spells.iter().any(|&spell| {
            ruleset
                .spell(spell)
                .is_some_and(|s| s.magic.effects.contains(&SpellEffect::Earthquake))
        });
        let ranged = hero.damage.ranged;
        let archery = ranged
            .single
            .map(NonZeroU32::get)
            .map(X87::from_f32_bits)
            .unwrap_or(X87::ratio(ranged.bp, FULL_BP.into(), SINGLE_BITS).unwrap_or(X87::ZERO));
        Self {
            attack: hero.attack,
            defense: hero.defense,
            tactics: hero.tactics,
            ballistics: hero.war_machines.catapult.map_or(0, |c| c.level),
            earthquake,
            archery,
            speed: hero.effects.iter().map(all_speed).sum(),
            hit_points: hero.effects.iter().filter_map(hit_point_change).collect(),
        }
    }

    /// `hero_creature_health_bonus` 0x4E5B80: the hit points the hero adds to `creature` of
    /// `base` hit points, each change that selects it: an addition, or a share of `base` in
    /// basis points rounded down (the Elixir's quarter, `sar` of the count's hit points).
    fn health_bonus(&self, ruleset: &BattleRuleset, creature: CreatureTypeId, base: u32) -> i64 {
        self.hit_points
            .iter()
            .filter(|(creatures, _, _)| creatures.matches(ruleset, creature))
            .map(|&(_, op, amount)| match op {
                StatOp::Add => i64::from(amount),
                StatOp::PercentBp | StatOp::PercentBpDown => {
                    i64::from(base) * i64::from(amount) / i64::from(FULL_BP)
                }
            })
            .sum()
    }
}

/// A battle effect's speed change for every creature (the effect's amount), else 0.
fn all_speed(effect: &BattleEffect) -> i32 {
    match *effect {
        BattleEffect::StackStat {
            creatures: CreatureSelector::All,
            stat: CreatureStat::Speed,
            op: StatOp::Add,
            amount,
        } => i32::from(amount),
        _ => 0,
    }
}

/// A battle effect's hit-point change, with the creatures it selects.
fn hit_point_change(
    effect: &BattleEffect,
) -> Option<(CreatureSelector<CreatureTypeId>, StatOp, i16)> {
    match *effect {
        BattleEffect::StackStat {
            creatures,
            stat: CreatureStat::HitPoints,
            op,
            amount,
        } => Some((creatures, op, amount)),
        _ => None,
    }
}

/// A stack of a simulated side (`sim_combat_stack`, `docs/exe/structs.toml`).
#[derive(Debug, Clone, PartialEq, Eq)]
struct Stack {
    /// Its army slot (+0x00).
    slot: u8,
    /// Its creature (+0x04).
    creature: CreatureTypeId,
    /// Creatures alive (+0x08): the ceiling of the value over the unit value after a loss.
    count: i32,
    /// Creatures at the start (+0x0C): what a raise may bring back.
    start: i32,
    /// The creature's speed and the hero's bonus (+0x10).
    speed: i32,
    /// The share of its value dealt in a round of melee (+0x18).
    melee: X87,
    /// The share of its value in the stand-off (+0x20).
    hold: X87,
    /// A shooter's share of its value in a round of shots (+0x28), 0 for any other.
    shot: X87,
    /// The value of a hit point (+0x30): the unit value over the creature's hit points with the
    /// hero's bonus; what a spell's damage in hit points is worth (an other spell's slip turns it,
    /// [`change`]).
    per_hp: SignedX87,
    /// Rounds to contact (+0x38): 0 a shooter, else 1 to 4.
    contact: i32,
    /// The value of one creature (+0x3C).
    unit: i32,
    /// The stack's value (+0x40): unit value times count at the start, less its losses.
    value: i32,
}

/// A side's hero as its casts and its enemy's read it (`sim_combat`'s +0x24 and what
/// `sim_cast_spell` reads of it), kept once for the lookahead's copies.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Caster {
    /// The battle's record of its magic (`HeroMagic::of`: masteries, spell bonuses, resistance,
    /// defences, the negation, the duration bonus, the level limit, `NoCasting`).
    magic: HeroMagic,
    /// Its Spell Power clamped to 1..99 (`sim_cast_spell`'s power).
    power: i32,
    /// Its rounds: the power and the duration bonus (`hero_duration_bonus` 0x4E4DB0).
    duration: i32,
    /// Its available battle spells (spells 10 to 69 of the original's +0x430).
    spells: SpellSet,
}

/// What a side keeps of its hero, its army and the tile for the casts.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Data {
    /// Its hero, if it has one.
    hero: Option<Caster>,
    /// Whether it may cast (`sim_combat`'s +0x18): a hero with a spell book, neither hero's
    /// `NoCasting`, no anti-magic garrison.
    may_cast: bool,
    /// The highest spell level it casts: the special ground's (Cursed Ground's 1) and either
    /// hero's level limit (Recanter's Cloak's 2).
    level: u8,
    /// The special ground's battlefield (+0x10).
    ground: Option<BattlefieldId>,
    /// Its army (+0x28): its creatures change the enemy's spell costs and its own.
    army: Vec<ArmyStack>,
}

/// One side of a simulated combat (`sim_combat`, `docs/exe/structs.toml`): its stacks, weakest
/// unit first, its strength and its spell points.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Side {
    /// The stacks, by unit value, ties in slot order.
    stacks: Vec<Stack>,
    /// The sum of the stacks' values less every loss taken (+0x1C): 0 the side is beaten.
    strength: i32,
    /// Its hero's spell points (+0x14), less each cast's cost.
    mana: i32,
    /// Its hero, army and tile, shared by the lookahead's copies.
    data: Arc<Data>,
}

/// The walls a side faces (`sim_side_walls` 0x424790; CPU-SIM-001): with a town's Fort 4, a
/// Citadel 5, a Castle 6; none when the side's hero has Earthquake; else the level less the
/// hero's Ballistics, at least 2; at most 4 in every case. `(walls, level)`.
// rule: CPU-SIM-001
fn walls(fort: FortLevel, hero: Option<&HeroReads>) -> (bool, i32) {
    let level = match fort {
        FortLevel::None => 0,
        FortLevel::Fort => 4,
        FortLevel::Citadel => 5,
        FortLevel::Castle => 6,
    };
    let (walls, level) = match hero {
        Some(hero) if hero.earthquake => (false, 0),
        Some(hero) if level > 0 => (true, (level - i32::from(hero.ballistics)).max(2)),
        _ => (level > 0, level),
    };
    (walls, level.min(FAR))
}

/// `sqrt(1 + 0.05 n)` as `sim_side_build` computes it (each step in double precision).
fn skill_root(n: u16) -> X87 {
    let twentieth = X87::ratio(1, 20, DOUBLE_BITS).unwrap_or(X87::ZERO);
    X87::int(n.into()).times(twentieth).plus(X87::ONE).sqrt()
}

/// One primary skill as `sim_side_build` counts it: its signed byte above 99 as 99, above 0 as
/// itself, else 0.
fn counted(value: u8) -> u16 {
    match i8::from_ne_bytes([value]) {
        v if v > 99 => 99,
        v if v > 0 => v.unsigned_abs().into(),
        _ => 0,
    }
}

/// A simulated side (`sim_combat_start` 0x423EE0, `sim_side_build` 0x424120; CPU-SIM-001,
/// CPU-SIM-006): `army` of the side's hero `heroes.0`, against `heroes.1`, facing a town of
/// `fort` (`FortLevel::None` without one), on `ground`, its value scaled by `factor`. Tactics is
/// the hero's level less the opponent's, at least 0; the factor is multiplied by sqrt(1 + 0.05 a)
/// × sqrt(1 + 0.05 d), a the hero's Attack over the opponent's Defense and d its Defense over the
/// opponent's Attack (each clamped to 0..99 and the difference at least 0; without an opponent
/// its own), not without a hero. Per army slot with a creature, in slot order: its speed and its
/// hit points with the hero's bonuses; its unit value ftol(fight value × sqrt(hit points with the
/// bonus / hit points) × factor) and its value per hit point the unit value over the hit points
/// with the bonus; its value that times its count, summed into the strength (32 bits); rounds to
/// contact 0 for a shooter, else (speed + 2 × (7 − Tactics)) / speed, at most 4, raised to the
/// walls' level for a walker; shares 0.2 of melee and 1 of the stand-off, for a shooter 0.1 and
/// 0.7 without `no_melee_penalty`, and of shots the hero's Archery / 5 (0.2 without a hero),
/// twice for `strikes_twice`, halved facing walls without the battle part's `no_wall_penalty`
/// (the original names the Arch Mage, 35, alone). The stacks sorted by unit value, ties in slot
/// order. Its spell points the hero's (0 without one); it may cast with a hero that has a spell
/// book while neither hero has `NoCasting` and no anti-magic garrison stands on the tile (the
/// original's Cursed Ground test of a game before SoD never holds: its version is SoD's); its
/// spells' highest level the ground's and both heroes' limits.
// rule: CPU-SIM-001, CPU-SIM-006
pub fn side(
    ruleset: &BattleRuleset,
    army: &[ArmyStack],
    factor: X87,
    heroes: (Option<EstimateHero<'_>>, Option<EstimateHero<'_>>),
    fort: FortLevel,
    ground: Ground,
) -> Side {
    let reads = (
        heroes.0.map(|h| HeroReads::of(h, ruleset)),
        heroes.1.map(|h| HeroReads::of(h, ruleset)),
    );
    let (own, opponent) = (reads.0.as_ref(), reads.1.as_ref());
    let (facing_walls, wall_level) = walls(fort, own);
    let level = |hero: Option<&HeroReads>| hero.map_or(0, |h| i32::from(h.tactics));
    let tactics = (level(own) - level(opponent)).max(0);
    let factor = own.map_or(factor, |hero| {
        let (attack, defense) = (counted(hero.attack), counted(hero.defense));
        let (a, d) = opponent.map_or((attack, defense), |o| {
            (
                attack.saturating_sub(counted(o.defense)),
                defense.saturating_sub(counted(o.attack)),
            )
        });
        skill_root(a).times(skill_root(d)).times(factor)
    });
    let fifth = X87::ratio(1, 5, DOUBLE_BITS).unwrap_or(X87::ZERO);
    let shot = own.map_or(fifth, |hero| {
        hero.archery.over(X87::int(5)).unwrap_or(X87::ZERO)
    });
    let speed_bonus = own.map_or(0, |hero| hero.speed);
    let mut stacks = Vec::new();
    let mut strength = 0_i32;
    for slot in 0..SLOTS {
        let Some(stack) = army.iter().find(|s| s.slot == slot) else {
            continue;
        };
        let creature = stack.creature;
        let record = ruleset.creature(creature);
        let flags = record.map(|r| r.flags).unwrap_or_default();
        let hit_points = record.map_or(0, |r| r.hit_points);
        let bonus = own.map_or(0, |hero| hero.health_bonus(ruleset, creature, hit_points));
        let health = u64::try_from(i64::from(hit_points) + bonus).unwrap_or(0);
        let speed = i32::from(record.map_or(0, |r| r.speed)).wrapping_add(speed_bonus);
        let fight = ruleset.creature_info(creature).map_or(0, |c| c.fight_value);
        let root = X87::int(health)
            .over(X87::int(hit_points.into()))
            .unwrap_or(X87::ZERO)
            .sqrt();
        let unit = SignedX87::from(root.times(X87::int(fight.into())).times(factor)).ftol();
        let per_hp = int(unit)
            .over(SignedX87::int(i64::try_from(health).unwrap_or(i64::MAX)))
            .unwrap_or(SignedX87::ZERO);
        let count = i32::from_ne_bytes(stack.count.to_ne_bytes());
        let value = unit.wrapping_mul(count);
        let contact = if flags.contains(CreatureFlag::Shooter) {
            0
        } else {
            let rounds = speed
                .wrapping_add(2 * (7 - tactics))
                .checked_div(speed)
                .unwrap_or(FAR)
                .min(FAR);
            if wall_level > rounds && !flags.contains(CreatureFlag::Flying) {
                wall_level
            } else {
                rounds
            }
        };
        let mut melee = fifth;
        let mut hold = X87::ONE;
        let mut shots = X87::ZERO;
        if contact == 0 {
            if !flags.contains(CreatureFlag::NoMeleePenalty) {
                melee = X87::ratio(1, 10, DOUBLE_BITS).unwrap_or(X87::ZERO);
                hold = X87::ratio(7, 10, DOUBLE_BITS).unwrap_or(X87::ZERO);
            }
            shots = shot;
            if flags.contains(CreatureFlag::StrikesTwice) {
                shots = shots.plus(shots);
            }
            let spared = ruleset
                .abilities(creature)
                .contains(&Ability::NoWallPenalty);
            if facing_walls && !spared {
                shots = shots.over(X87::int(2)).unwrap_or(X87::ZERO);
            }
        }
        strength = strength.wrapping_add(value);
        stacks.push(Stack {
            slot,
            creature,
            count,
            start: count,
            speed,
            melee,
            hold,
            shot: shots,
            per_hp,
            contact,
            unit,
            value,
        });
    }
    // The slots are unique: the key is.
    let _ = sort_by_unique_key(&mut stacks, |s| (s.unit, s.slot));
    let single = ruleset.quirks().is_enabled(SINGLE_PRECISION_RATES);
    let magic = |hero: Option<EstimateHero<'_>>| {
        hero.map(|h| HeroMagic::of(h.hero, single).unwrap_or_default())
    };
    let (own_magic, other_magic) = (magic(heroes.0), magic(heroes.1));
    let limit = |m: Option<&HeroMagic>| m.map_or(u8::MAX, |m| m.level_limit);
    let field = ground.battlefield.and_then(|b| ruleset.battlefield(b));
    let level = field
        .map_or(u8::MAX, |f| f.max_spell_level)
        .min(limit(own_magic.as_ref()))
        .min(limit(other_magic.as_ref()));
    let blocked = |m: Option<&HeroMagic>| m.is_some_and(|m| m.no_casting);
    let may_cast = heroes.0.is_some_and(|h| h.hero.magic.spell_book.is_some())
        && !blocked(own_magic.as_ref())
        && !blocked(other_magic.as_ref())
        && !ground.no_casting;
    let hero = heroes.0.zip(own_magic).map(|(h, magic)| {
        let power = i32::from(h.hero.power.clamp(1, MAX_POWER));
        let mut spells = SpellSet::default();
        for &id in h.spells {
            if ruleset
                .spell(id)
                .is_some_and(|s| s.category == SpellCategory::Combat)
            {
                spells.insert(id);
            }
        }
        Caster {
            power,
            duration: power.wrapping_add(i32::from(magic.duration_bonus)),
            magic,
            spells,
        }
    });
    let mana = heroes.0.map_or(0, |h| i32::from(h.hero.magic.mana));
    Side {
        stacks,
        strength,
        mana,
        data: Arc::new(Data {
            hero,
            may_cast,
            level,
            ground: ground.battlefield,
            army: army.to_vec(),
        }),
    }
}

/// `count`'s ceiling of `value` over `unit` as the code divides (`idiv`, toward 0, after a
/// 32-bit `lea`); 0 for a unit value of 0, which no stack with value left has.
fn ceiling(value: i32, unit: i32) -> i32 {
    value
        .wrapping_add(unit)
        .wrapping_sub(1)
        .checked_div(unit)
        .unwrap_or(0)
}

/// `n` as an x87 value (`fild`).
fn int(n: i32) -> SignedX87 {
    SignedX87::int(n.into())
}

/// Takes `amount` of value from `stack` (`sim_stack_take_value` 0x423EA0, and the same steps
/// inlined in the casts' applications; CPU-SIM-007): above its value the stack's value and count
/// go to 0 and its value is taken; else its value less the amount, its count the ceiling of that
/// over its unit value. The value taken.
// rule: CPU-SIM-007
fn take_value(stack: &mut Stack, amount: i32) -> i32 {
    if stack.value < amount {
        let all = stack.value;
        stack.value = 0;
        stack.count = 0;
        all
    } else {
        stack.value -= amount;
        stack.count = ceiling(stack.value, stack.unit);
        amount
    }
}

impl Side {
    /// The side's strength (+0x1C): 0 or less when it is beaten.
    pub const fn strength(&self) -> i32 {
        self.strength
    }

    /// The highest speed among the living stacks, 0 with none (`sim_side_max_speed` 0x424960):
    /// the faster side casts first.
    fn max_speed(&self) -> i32 {
        self.stacks
            .iter()
            .filter(|s| s.count > 0)
            .fold(0, |most, s| most.max(s.speed))
    }

    /// Kills the side (`sim_side_wipe` 0x4262B0; CPU-SIM-002): its strength and every stack's
    /// count and value 0.
    // rule: CPU-SIM-002
    fn wipe(&mut self) {
        self.strength = 0;
        for stack in &mut self.stacks {
            stack.count = 0;
            stack.value = 0;
        }
    }

    /// Spreads `loss` over the stacks whose rounds to contact lie in `lo..=hi`
    /// (`sim_spread_losses` 0x426170; CPU-SIM-002): each in order, the weakest first, with value
    /// left takes ftol(its value × the loss left / the group's value left), its count the
    /// ceiling of its value left over its unit value; a share above its value kills it and only
    /// its value counts; it stops when the loss or the group is used up. The loss not taken.
    // rule: CPU-SIM-002
    fn spread(&mut self, loss: i32, lo: i32, hi: i32) -> i32 {
        let within = |stack: &Stack| (lo..=hi).contains(&stack.contact);
        let mut group = self
            .stacks
            .iter()
            .filter(|s| within(s))
            .fold(0_i32, |sum, s| sum.wrapping_add(s.value));
        let mut loss = loss;
        for stack in self.stacks.iter_mut().filter(|s| within(s)) {
            if stack.value <= 0 {
                continue;
            }
            let share = int(stack.value)
                .times(int(loss))
                .over(int(group))
                .map_or(0, SignedX87::ftol);
            group = group.wrapping_sub(stack.value);
            loss = loss.wrapping_sub(take_value(stack, share));
            if loss <= 0 || group <= 0 {
                break;
            }
        }
        loss
    }

    /// A loss for the side (`sim_side_take_damage` 0x426300; CPU-SIM-002): its strength less
    /// `loss`; at 0 or below the side is wiped; else the loss spread first over its stacks in
    /// contact by `round` (rounds 1 to `round`), the rest over all of them.
    // rule: CPU-SIM-002
    fn take_damage(&mut self, loss: i32, round: i32) {
        self.strength = self.strength.wrapping_sub(loss);
        if self.strength <= 0 {
            self.wipe();
            return;
        }
        let left = self.spread(loss, 1, round);
        if left != 0 {
            self.spread(left, 0, FAR);
        }
    }

    /// The side's damage in a round (`sim_side_damage` 0x426390; CPU-SIM-003): over its stacks
    /// whose rounds to contact are at most `round`, the last first, count × unit value × share,
    /// the sum truncated after each stack: the shot share for a shooter unless `melee`, else
    /// the melee share. (0, false) the shots alone, (r, false) an engaging side's, (4, true) a
    /// side's fighting with all its stacks in melee.
    // rule: CPU-SIM-003
    fn damage(&self, round: i32, melee: bool) -> i32 {
        self.stacks
            .iter()
            .rev()
            .filter(|s| s.contact <= round)
            .fold(0, |sum, s| {
                let share = if !melee && s.contact == 0 {
                    s.shot
                } else {
                    s.melee
                };
                int(s.count.wrapping_mul(s.unit))
                    .times(share.into())
                    .plus(int(sum))
                    .ftol()
            })
    }

    /// The side's hold value (`sim_side_hold_value` 0x426450; CPU-SIM-003): over its stacks,
    /// the last first, value × hold share, the sum truncated after each.
    // rule: CPU-SIM-003
    fn hold(&self) -> i32 {
        self.stacks.iter().rev().fold(0, |sum, s| {
            int(s.value).times(s.hold.into()).plus(int(sum)).ftol()
        })
    }

    /// The army after the fight (`sim_apply_losses` 0x424880; CPU-SIM-002): with the strength
    /// at 0 none; else each stack of `army` with its simulated stack's count, a count of 0
    /// cleared. The original's flag that sends a beaten hero out of the game belongs to the
    /// quick battle's result (`ai_quick_battle`, ADR-0030 point 7), not the estimate.
    // rule: CPU-SIM-002
    pub fn survivors(&self, army: &[ArmyStack]) -> Vec<ArmyStack> {
        if self.strength == 0 {
            return Vec::new();
        }
        army.iter()
            .map(|stack| ArmyStack {
                count: self
                    .stacks
                    .iter()
                    .find(|s| s.slot == stack.slot)
                    .map_or(stack.count, |s| u32::try_from(s.count).unwrap_or(0)),
                ..*stack
            })
            .filter(|stack| stack.count > 0)
            .collect()
    }

    /// The side's hero's magic as a stack's controller reads it, if it has a hero.
    fn magic(&self) -> Option<&HeroMagic> {
        self.data.hero.as_ref().map(|h| &h.magic)
    }
}

/// A hold value as the stand-off stores it (an f32).
fn single(hold: i32) -> SignedX87 {
    int(hold).round(SINGLE_BITS)
}

/// The end of a simulated combat (`sim_combat_finish` 0x4264D0, and `sim_engage_now`'s copy;
/// CPU-SIM-003): with both hold values above 0 the larger wins, the other is wiped and the
/// winner loses ftol(f32(loser / winner + 0.05) × loser), spread over its stacks (its strength
/// first, as a loss in round 0); equal values: `a` is wiped.
// rule: CPU-SIM-003
fn finish(a: &mut Side, b: &mut Side) {
    let (hold_a, hold_b) = (single(a.hold()), single(b.hold()));
    if hold_a == SignedX87::ZERO || hold_b == SignedX87::ZERO {
        return;
    }
    let twentieth = SignedX87::from(X87::ratio(1, 20, DOUBLE_BITS).unwrap_or(X87::ZERO));
    let loss = |loser: SignedX87, winner: SignedX87| {
        loser
            .over(winner)
            .unwrap_or(SignedX87::ZERO)
            .plus(twentieth)
            .round(SINGLE_BITS)
            .times(loser)
            .ftol()
    };
    if hold_a > hold_b {
        b.wipe();
        a.take_damage(loss(hold_b, hold_a), 0);
    } else {
        a.wipe();
        b.take_damage(loss(hold_a, hold_b), 0);
    }
}

/// The round's casts: the faster side's first (`sim_side_max_speed`; equal speeds `first`'s),
/// then the other's, each [`cast`].
fn casts(ruleset: &BattleRuleset, first: &mut Side, second: &mut Side, round: i32) {
    if first.max_speed() < second.max_speed() {
        cast(ruleset, second, first, round);
        cast(ruleset, first, second, round);
    } else {
        cast(ruleset, first, second, round);
        cast(ruleset, second, first, round);
    }
}

/// A spell a side considers (`sim_spell`, filled by 0x436980: `docs/exe/structs.toml`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Candidate {
    /// The spell (+0x00).
    spell: SpellId,
    /// The caster's mastery of it on the ground (+0x04).
    mastery: u8,
    /// The caster's Spell Power, 1 to 99 (+0x08).
    power: i32,
    /// The caster's rounds (+0x0C).
    duration: i32,
    /// The stack it is aimed at (+0x14): the caster's own or the enemy's by its kind.
    target: Option<usize>,
    /// Dispel's enemy stack at Advanced when it beats the own one (+0x18).
    second: Option<usize>,
    /// Its value (+0x1C); the highest wins.
    value: i32,
}

/// How the casts treat a spell, by its AI kinds and effects (`sim_cast_spell`'s switch on the
/// flags' bits 0x1F8000 and its tests of the spell's number).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    /// Damage to one stack and its chain or area (`Damage`; `DamageOnce` in round 1 alone).
    Damage,
    /// Damage to every stack of both sides (`DamageAll`).
    Mass,
    /// Any other battle spell (`Effect`).
    Other,
    /// Resurrection and Animate Dead: a `Stacks` spell that raises (the original's 38, 39).
    Raise,
}

/// The cast's kind of `spell` in `round`, if the casts value it: one AI kind alone (the
/// original's switch takes the bits as one value: Titan's Lightning Bolt's two are worth 0),
/// `DamageOnce` only in round 1, a `Stacks` spell only when it raises.
fn kind(spell: &BattleSpell, round: i32) -> Option<Kind> {
    match spell.ai.kinds.only()? {
        SpellAiKind::Damage => Some(Kind::Damage),
        SpellAiKind::DamageOnce => (round == 1).then_some(Kind::Damage),
        SpellAiKind::DamageAll => Some(Kind::Mass),
        SpellAiKind::Effect => Some(Kind::Other),
        SpellAiKind::Stacks => spell
            .magic
            .effects
            .contains(&SpellEffect::Raise)
            .then_some(Kind::Raise),
        SpellAiKind::Special => None,
    }
}

/// One side's cast in a round (`sim_cast_spell` 0x425BD0; CPU-SIM-006): only with a strength, spell
/// points and casting allowed. Every available battle spell in handle order (the original's 10
/// to 69) up to the side's highest level whose cost at the caster's mastery on the ground
/// (`hero_spell_mastery` 0x4E5080, `hero_spell_cost` 0x4E5240 with both armies) its points pay is
/// valued by its kind ([`kind`]: [`damage_value`], [`mass_value`], [`other_value`],
/// [`raise_value`]); the highest value above 0 wins, ties to the lower spell. Its cost leaves the
/// points; an enemy with a hero and a living stack with `mana_channel` (the Familiars) gains its
/// percent of the cost, rounded down, once (the stacks from the last); then it is applied
/// ([`damage_apply`], [`mass_apply`], [`other_apply`], [`raise_apply`]).
// rule: CPU-SIM-006
fn cast(ruleset: &BattleRuleset, caster: &mut Side, enemy: &mut Side, round: i32) {
    let data = Arc::clone(&caster.data);
    let Some(hero) = data.hero.as_ref().filter(|_| data.may_cast) else {
        return;
    };
    if caster.strength == 0 || caster.mana == 0 {
        return;
    }
    let change = mana::armies_cost_change(ruleset, &data.army, &enemy.data.army);
    let magic = &hero.magic;
    let mut best: Option<(Candidate, Kind, i32, SpellRef<'_>)> = None;
    for (id, spell) in hero
        .spells
        .iter()
        .filter_map(|id| ruleset.spell(id).map(|spell| (id, spell)))
    {
        if spell.level > data.level {
            continue;
        }
        let mastery = magic::mastery_at(ruleset, data.ground, &hero.magic, id);
        let cost = i32::from(mana::cost(spell, mastery, change));
        if cost > caster.mana {
            continue;
        }
        let Some(kind) = kind(spell, round) else {
            continue;
        };
        let mut candidate = Candidate {
            spell: id,
            mastery,
            power: hero.power,
            duration: hero.duration,
            target: None,
            second: None,
            value: 0,
        };
        let spell = (id, spell);
        match kind {
            Kind::Damage => damage_value(ruleset, enemy, (magic, spell), &mut candidate),
            Kind::Mass => mass_value(ruleset, (caster, enemy), (magic, spell), &mut candidate),
            Kind::Other => other_value(ruleset, (caster, enemy), (magic, spell), &mut candidate),
            Kind::Raise => raise_value(ruleset, caster, (magic, spell), &mut candidate),
        }
        if candidate.value > best.map_or(0, |(b, _, _, _)| b.value) {
            best = Some((candidate, kind, cost, spell));
        }
    }
    let Some((best, kind, cost, spell)) = best else {
        return;
    };
    caster.mana = caster.mana.wrapping_sub(cost);
    if enemy.data.hero.is_some() {
        let channel = enemy.stacks.iter().rev().find_map(|s| {
            ruleset
                .abilities(s.creature)
                .iter()
                .find_map(|a| match *a {
                    Ability::ManaChannel { percent } => Some(percent),
                    _ => None,
                })
                .filter(|_| s.count > 0)
        });
        if let Some(percent) = channel {
            enemy.mana = enemy
                .mana
                .wrapping_add(cost.wrapping_mul(percent.into()) / 100);
        }
    }
    match kind {
        Kind::Damage => damage_apply(ruleset, enemy, (magic, spell), &best),
        Kind::Mass => mass_apply(ruleset, (caster, enemy), (magic, spell), &best),
        Kind::Other => other_apply(ruleset, (caster, enemy), (magic, spell), &best),
        Kind::Raise => raise_apply(ruleset, caster, (magic, spell), &best),
    }
}

/// A spell's base value for a candidate (0x436930 with the spell's +0x30): its effect at the
/// mastery plus its power factor times the Spell Power (32 bits).
fn base(spell: &BattleSpell, c: &Candidate) -> i32 {
    let effect = spell
        .effect
        .get(usize::from(c.mastery))
        .copied()
        .unwrap_or(0);
    i32::from(effect).wrapping_add(i32::from(spell.power).wrapping_mul(c.power))
}

/// The value `damage` of `spell` takes from `stack` (the chain at 0x424DB7 and its copies, and
/// `sim_mass_damage_stack_loss` 0x423DE0; CPU-SIM-007): 0 for a stack without value; else the
/// creature's factor of the spell under both heroes (`creature_spell_factor` 0x44A4D0, with
/// `caster`'s and the stack's controller `own`'s magic) times f32(`damage`), truncated; the
/// target's factors (`creature_spell_damage_factor` 0x44B4B0); the caster's bonuses without a
/// stack (`hero_spell_damage_bonus` 0x4E5760 given none: the Orbs and Sorcery); that times the
/// stack's value per hit point, truncated, at most the stack's value. The original's tests of
/// each step for 0 return what the next step makes of 0.
// rule: CPU-SIM-007
fn loss(
    ruleset: &BattleRuleset,
    (id, spell): SpellRef<'_>,
    damage: i32,
    stack: &Stack,
    (caster, own): (&HeroMagic, Option<&HeroMagic>),
) -> i32 {
    if stack.value == 0 {
        return 0;
    }
    let guard = Guard::of_heroes(Some(caster), own);
    let resistance = own.map_or(0, |h| h.resistance);
    let f = magic::creature_factor(ruleset, (id, spell), stack.creature, guard, resistance);
    let first = SignedX87::from(f)
        .times(int(damage).round(SINGLE_BITS))
        .ftol();
    let target = magic::creature_damage(
        ruleset,
        id,
        u64::try_from(first).unwrap_or(0),
        stack.creature,
    );
    let bonus = magic::untargeted_bonus(ruleset, spell, &caster.bonuses, target);
    let worth = int(i32::try_from(bonus).unwrap_or(i32::MAX))
        .times(stack.per_hp)
        .ftol();
    stack.value.min(worth)
}

/// A damage spell's value (`sim_damage_spell_value` 0x424D20; CPU-SIM-007): the enemy's stack
/// it takes the most from ([`loss`] at its base value; the stacks from the last, ties to the
/// later one); with that above 0, a chain adds its hops ([`chain_value`]) and an area spell
/// the stacks beside it ([`area_value`]).
// rule: CPU-SIM-007
fn damage_value(
    ruleset: &BattleRuleset,
    enemy: &Side,
    (hero, spell): (&HeroMagic, SpellRef<'_>),
    c: &mut Candidate,
) {
    let damage = base(spell.1, c);
    let heroes = (hero, enemy.magic());
    for (i, stack) in enemy.stacks.iter().enumerate().rev() {
        let v = loss(ruleset, spell, damage, stack, heroes);
        if v > c.value {
            c.value = v;
            c.target = Some(i);
        }
    }
    let Some(target) = c.target.filter(|_| c.value > 0) else {
        return;
    };
    if spell.1.magic.chain.is_some() {
        chain_value(ruleset, enemy, spell, (target, damage), c, heroes);
    } else if let Some(area) = spell.1.magic.area {
        let n = area
            .radius
            .get(usize::from(c.mastery))
            .copied()
            .unwrap_or(0);
        area_value(ruleset, enemy, spell, (target, damage, n), c, heroes);
    }
}

/// The next stack of a chain (0x4249D0; CPU-SIM-007): none for a damage of 0; else the nearest
/// stack before `previous` in the enemy's order (weaker), not yet `hit`, that `damage` takes
/// something from; else the nearest after it.
// rule: CPU-SIM-007
fn next_in_chain(
    ruleset: &BattleRuleset,
    enemy: &Side,
    spell: SpellRef<'_>,
    (hit, previous, damage): (u32, usize, i32),
    heroes: (&HeroMagic, Option<&HeroMagic>),
) -> Option<usize> {
    if damage == 0 {
        return None;
    }
    let takes = |&i: &usize| {
        hit & (1 << i) == 0 && loss(ruleset, spell, damage, &enemy.stacks[i], heroes) > 0
    };
    (0..previous)
        .rev()
        .find(takes)
        .or_else(|| (previous + 1..enemy.stacks.len()).find(takes))
}

/// A chain's hops in its value (0x424E69; CPU-SIM-007): three, each at half the damage before
/// (toward 0), to [`next_in_chain`] of the stack before, its [`loss`] added.
// rule: CPU-SIM-007
fn chain_value(
    ruleset: &BattleRuleset,
    enemy: &Side,
    spell: SpellRef<'_>,
    (target, damage): (usize, i32),
    c: &mut Candidate,
    heroes: (&HeroMagic, Option<&HeroMagic>),
) {
    let (mut hit, mut previous, mut damage) = (1_u32 << target, target, damage);
    for _ in 0..CHAIN_HOPS {
        damage /= 2;
        let Some(next) = next_in_chain(ruleset, enemy, spell, (hit, previous, damage), heroes)
        else {
            return;
        };
        let v = loss(ruleset, spell, damage, &enemy.stacks[next], heroes);
        c.value = c.value.wrapping_add(v);
        hit |= 1 << next;
        previous = next;
    }
}

/// An area spell's neighbours in its value (`sim_damage_spell_area_value` 0x424BF0;
/// CPU-SIM-007): the enemy's stacks in order whose army slot is next to the target's (one
/// apart) and that the damage takes something from, up to `n` of them (the area's radius at the
/// mastery: Inferno's 2, the others' 1), each [`loss`] added.
// rule: CPU-SIM-007
fn area_value(
    ruleset: &BattleRuleset,
    enemy: &Side,
    spell: SpellRef<'_>,
    (target, damage, n): (usize, i32, u8),
    c: &mut Candidate,
    heroes: (&HeroMagic, Option<&HeroMagic>),
) {
    let slot = enemy.stacks[target].slot;
    let mut left = i32::from(n);
    for stack in enemy.stacks.iter().filter(|s| s.slot.abs_diff(slot) == 1) {
        let v = loss(ruleset, spell, damage, stack, heroes);
        if v > 0 {
            c.value = c.value.wrapping_add(v);
            left -= 1;
            if left == 0 {
                return;
            }
        }
    }
}

/// A damage spell cast (`sim_damage_spell_apply` 0x425260, 0x424FB0, 0x425100; CPU-SIM-007): the
/// target loses its [`loss`] ([`take_value`]), the enemy's strength what it took; then a chain's
/// three hops as its value found them, each on the stacks as they stand, and an area's
/// neighbours alike.
// rule: CPU-SIM-007
fn damage_apply(
    ruleset: &BattleRuleset,
    enemy: &mut Side,
    (hero, spell): (&HeroMagic, SpellRef<'_>),
    c: &Candidate,
) {
    // A chosen damage spell has a target.
    let target = c.target.unwrap_or(0);
    let damage = base(spell.1, c);
    let data = Arc::clone(&enemy.data);
    let heroes = (hero, data.hero.as_ref().map(|h| &h.magic));
    let hit = |enemy: &mut Side, i: usize, damage: i32| {
        let v = loss(ruleset, spell, damage, &enemy.stacks[i], heroes);
        let taken = take_value(&mut enemy.stacks[i], v);
        enemy.strength = enemy.strength.wrapping_sub(taken);
    };
    hit(enemy, target, damage);
    if spell.1.magic.chain.is_some() {
        let (mut mask, mut previous, mut damage) = (1_u32 << target, target, damage);
        for _ in 0..CHAIN_HOPS {
            damage /= 2;
            let Some(next) = next_in_chain(ruleset, enemy, spell, (mask, previous, damage), heroes)
            else {
                return;
            };
            hit(enemy, next, damage);
            mask |= 1 << next;
            previous = next;
        }
    } else if let Some(area) = spell.1.magic.area {
        let slot = enemy.stacks[target].slot;
        let mut left = i32::from(
            area.radius
                .get(usize::from(c.mastery))
                .copied()
                .unwrap_or(0),
        );
        for i in 0..enemy.stacks.len() {
            if enemy.stacks[i].slot.abs_diff(slot) != 1 {
                continue;
            }
            if loss(ruleset, spell, damage, &enemy.stacks[i], heroes) > 0 {
                hit(enemy, i, damage);
                left -= 1;
                if left == 0 {
                    return;
                }
            }
        }
    }
}

/// A mass damage spell's value (`sim_cast_spell` 0x425D4B, `sim_mass_damage_enemy_loss`
/// 0x4253E0; CPU-SIM-008): the [`loss`] of every own stack (the caster's hero the stacks'
/// controller too) and of every enemy stack, each summed; the enemy's less the own while the own
/// is below both the side's strength and the enemy's.
// rule: CPU-SIM-008
fn mass_value(
    ruleset: &BattleRuleset,
    (caster, enemy): (&Side, &Side),
    (hero, spell): (&HeroMagic, SpellRef<'_>),
    c: &mut Candidate,
) {
    let damage = base(spell.1, c);
    let sum = |side: &Side, own: Option<&HeroMagic>| {
        side.stacks.iter().rev().fold(0_i32, |sum, s| {
            sum.wrapping_add(loss(ruleset, spell, damage, s, (hero, own)))
        })
    };
    let own = sum(caster, Some(hero));
    let theirs = sum(enemy, enemy.magic());
    if own < caster.strength && own < theirs {
        c.value = theirs.wrapping_sub(own);
    }
}

/// A mass damage spell cast (`sim_cast_spell` 0x425F7A to 0x4260EA; CPU-SIM-008): over the
/// caster's stacks from the last a running total of their [`loss`]es that is never reset
/// (0x425FF6): a stack whose value is below it dies and the total becomes its value, else it
/// loses the total; the side's strength loses the total at each stack. Over the enemy's stacks
/// from the last a total of the value the stack before took and the stack's own [`loss`],
/// taken from the stack ([`take_value`], 0x4260C6); the enemy's strength loses what each took.
// rule: CPU-SIM-008
fn mass_apply(
    ruleset: &BattleRuleset,
    (caster, enemy): (&mut Side, &mut Side),
    (hero, spell): (&HeroMagic, SpellRef<'_>),
    c: &Candidate,
) {
    let damage = base(spell.1, c);
    let mut running = 0_i32;
    for i in (0..caster.stacks.len()).rev() {
        let stack = &mut caster.stacks[i];
        running = running.wrapping_add(loss(ruleset, spell, damage, stack, (hero, Some(hero))));
        if stack.value < running {
            running = stack.value;
            stack.value = 0;
            stack.count = 0;
        } else {
            stack.value -= running;
            stack.count = ceiling(stack.value, stack.unit);
        }
        caster.strength = caster.strength.wrapping_sub(running);
    }
    let their_data = Arc::clone(&enemy.data);
    let theirs = their_data.hero.as_ref().map(|h| &h.magic);
    let mut carried = 0_i32;
    for i in (0..enemy.stacks.len()).rev() {
        let stack = &mut enemy.stacks[i];
        carried = carried.wrapping_add(loss(ruleset, spell, damage, stack, (hero, theirs)));
        carried = take_value(stack, carried);
        enemy.strength = enemy.strength.wrapping_sub(carried);
    }
}

/// Whether a spell only counters the enemy's spells (the original's 30 to 37: the Protections,
/// Anti-Magic, Dispel, Magic Mirror, Cure), told by its effects.
fn counters_casting(spell: &BattleSpell) -> bool {
    spell.magic.effects.iter().any(|e| {
        matches!(
            e,
            SpellEffect::Dispel
                | SpellEffect::Timed(
                    TimedChange::SchoolProtection
                        | TimedChange::SpellImmunity
                        | TimedChange::MagicMirror
                )
        )
    })
}

/// Whether a spell is Dispel (the original's 35): it ends spells and does not heal (Cure does).
fn is_dispel(spell: &BattleSpell) -> bool {
    let effects = &spell.magic.effects;
    effects.contains(&SpellEffect::Dispel) && !effects.contains(&SpellEffect::Heal)
}

/// A stack's part in an other spell's value (`sim_stack_spell_value` 0x423C80; CPU-SIM-009): 0
/// for a stack without value; else ftol(f32(AI value at the mastery × min(rounds, 5) × the
/// stack's value) × f32(the creature's factor under the caster's and the controller's heroes,
/// [`magic::creature_factor`]) / 500), the product in 32 bits.
// rule: CPU-SIM-009
fn stack_value(
    ruleset: &BattleRuleset,
    (id, spell): SpellRef<'_>,
    c: &Candidate,
    stack: &Stack,
    (caster, own): (&HeroMagic, Option<&HeroMagic>),
) -> i32 {
    if stack.value == 0 {
        return 0;
    }
    let ai = spell
        .ai
        .values
        .get(usize::from(c.mastery))
        .copied()
        .unwrap_or(0);
    let product = i32::from(ai)
        .wrapping_mul(c.duration.min(MAX_ROUNDS))
        .wrapping_mul(stack.value);
    let guard = Guard::of_heroes(Some(caster), own);
    let resistance = own.map_or(0, |h| h.resistance);
    let f = magic::creature_factor(ruleset, (id, spell), stack.creature, guard, resistance);
    int(product)
        .round(SINGLE_BITS)
        .times(SignedX87::from(f.round(SINGLE_BITS)))
        .over(int(500))
        .map_or(0, SignedX87::ftol)
}

/// A stack's value `v` into an other spell's candidate: summed for a mass version, else the
/// largest kept with its stack (ties to the later one valued).
fn pick(c: &mut Candidate, (i, v): (usize, i32), mass: bool) {
    if mass {
        c.value = c.value.wrapping_add(v);
    } else if v > c.value {
        c.value = v;
        c.target = Some(i);
    }
}

/// An other spell's value (`sim_other_spell_value` 0x425510; CPU-SIM-009): nothing for a spell
/// that only counters the enemy's spells when the enemy cannot cast ([`counters_casting`]).
/// Dispel: its value on the own stacks under the caster's hero, and from Advanced on the enemy's
/// too, the stacks from the last, by [`stack_value`]; a mass version sums, else the best stack
/// is kept ([`pick`]); at Advanced an enemy stack that beats the own value moves to the second
/// target. A positive spell values the own stacks, any other the enemy's, alike.
// rule: CPU-SIM-009
fn other_value(
    ruleset: &BattleRuleset,
    (caster, enemy): (&Side, &Side),
    (hero, spell): (&HeroMagic, SpellRef<'_>),
    c: &mut Candidate,
) {
    if !enemy.data.may_cast && counters_casting(spell.1) {
        return;
    }
    let mass = !takes_stack(ruleset, spell.0, c.mastery);
    let value = |side: &Side, own: Option<&HeroMagic>, c: &mut Candidate| {
        for (i, s) in side.stacks.iter().enumerate().rev() {
            pick(c, (i, stack_value(ruleset, spell, c, s, (hero, own))), mass);
        }
    };
    if is_dispel(spell.1) {
        value(caster, Some(hero), c);
        if c.mastery < ADVANCED {
            return;
        }
        let own = c.value;
        value(enemy, enemy.magic(), c);
        if c.mastery == ADVANCED && own < c.value {
            c.second = c.target;
            c.target = None;
        }
    } else if spell.1.polarity == SpellPolarity::Positive {
        value(caster, Some(hero), c);
    } else {
        value(enemy, enemy.magic(), c);
    }
}

/// An other spell's change of a stack by its value `v` (0x4258A0; CPU-SIM-009): its value and
/// its unit value up (`positive`) or down by `v` and by unit × `v` / value (64 bits, toward 0; 0
/// for a stack without value, where the original divides by 0: never chosen), and its value per
/// hit point the new value over the old value times the old value per hit point: the original's
/// slip (survey §5.5 #65, its hit points would be the old value over it), ported as read. The
/// side's strength stays.
// rule: CPU-SIM-009
fn change(stack: &mut Stack, v: i32, positive: bool) {
    let old = stack.value;
    let product = int(old).times(stack.per_hp);
    let q = (i64::from(stack.unit) * i64::from(v))
        .checked_div(i64::from(old))
        .unwrap_or(0);
    // The quotient's low 32 bits (the original keeps `eax`).
    let q = q as i32;
    if positive {
        stack.value = old.wrapping_add(v);
        stack.unit = stack.unit.wrapping_add(q);
    } else {
        stack.value = old.wrapping_sub(v);
        stack.unit = stack.unit.wrapping_sub(q);
    }
    stack.per_hp = int(stack.value).over(product).unwrap_or(SignedX87::ZERO);
}

/// An other spell on one side's stacks (0x4258A0; CPU-SIM-009): its aimed stack ([`change`] by
/// its [`stack_value`], whatever it is), or for a mass version every stack with value whose
/// value is above 0, the stacks from the last; the caster's hero `caster` and the side's own.
// rule: CPU-SIM-009
fn other_on(
    ruleset: &BattleRuleset,
    side: &mut Side,
    spell: SpellRef<'_>,
    (c, target): (&Candidate, Option<usize>),
    (caster, positive): (&HeroMagic, bool),
) {
    let data = Arc::clone(&side.data);
    let heroes = (caster, data.hero.as_ref().map(|h| &h.magic));
    if takes_stack(ruleset, spell.0, c.mastery) {
        // A chosen aimed spell has its stack on this side.
        let aimed = target.and_then(|i| side.stacks.get_mut(i));
        aimed.into_iter().for_each(|stack| {
            let v = stack_value(ruleset, spell, c, stack, heroes);
            change(stack, v, positive);
        });
        return;
    }
    for stack in side.stacks.iter_mut().rev() {
        let v = stack_value(ruleset, spell, c, stack, heroes);
        if v > 0 {
            change(stack, v, positive);
        }
    }
}

/// An other spell cast (`sim_other_spell_apply_call` 0x425B10; CPU-SIM-009): Dispel at Expert
/// on the own stacks as a positive spell and on the enemy's as a negative one; below Expert on
/// its own target, or else on its enemy target as a negative spell. A positive spell on the own
/// stacks, any other on the enemy's ([`other_on`]).
// rule: CPU-SIM-009
fn other_apply(
    ruleset: &BattleRuleset,
    (caster, enemy): (&mut Side, &mut Side),
    (hero, spell): (&HeroMagic, SpellRef<'_>),
    c: &Candidate,
) {
    if is_dispel(spell.1) {
        if c.mastery > ADVANCED {
            other_on(ruleset, caster, spell, (c, c.target), (hero, true));
            other_on(ruleset, enemy, spell, (c, c.target), (hero, false));
        } else if c.target.is_some() {
            other_on(ruleset, caster, spell, (c, c.target), (hero, true));
        } else {
            other_on(ruleset, enemy, spell, (c, c.second), (hero, false));
        }
    } else if spell.1.polarity == SpellPolarity::Positive {
        other_on(ruleset, caster, spell, (c, c.target), (hero, true));
    } else {
        other_on(ruleset, enemy, spell, (c, c.target), (hero, false));
    }
}

/// What a raise brings back to `stack` in value (0x423D00; CPU-SIM-010): nothing for a stack
/// that lost no creatures or that the spell cannot affect (the creature's factor under the
/// caster's hero 0); else its base value with the caster's specialty (`hero_spell_specialty_bonus`
/// 0x4E5FF0 by the creature's level) in hit points, times the stack's value per hit point,
/// truncated, over its unit value (toward 0; 0 for a unit value of 0, where the original
/// divides by 0), at most the creatures it lost, times its unit value.
// rule: CPU-SIM-010
fn raised(
    ruleset: &BattleRuleset,
    (id, spell): SpellRef<'_>,
    c: &Candidate,
    stack: &Stack,
    hero: &HeroMagic,
) -> i32 {
    if stack.start <= stack.count {
        return 0;
    }
    let guard = Guard::of_heroes(Some(hero), Some(hero));
    let f = magic::creature_factor(ruleset, (id, spell), stack.creature, guard, hero.resistance);
    if f == X87::ZERO {
        return 0;
    }
    let value = base(spell, c);
    let specialty = magic::specialist_bonus(
        ruleset,
        spell,
        hero.bonuses.spell(id),
        stack.creature,
        u64::try_from(value).unwrap_or(0),
    );
    let value = value.wrapping_add(i32::try_from(specialty).unwrap_or(0));
    let creatures = int(value)
        .times(stack.per_hp)
        .ftol()
        .checked_div(stack.unit)
        .unwrap_or(0);
    stack
        .start
        .wrapping_sub(stack.count)
        .min(creatures)
        .wrapping_mul(stack.unit)
}

/// A raise's value (`sim_cast_spell` 0x425DF8; CPU-SIM-010): the own stack it brings back the
/// most to ([`raised`]; the stacks from the last, ties to the later one).
// rule: CPU-SIM-010
fn raise_value(
    ruleset: &BattleRuleset,
    caster: &Side,
    (hero, spell): (&HeroMagic, SpellRef<'_>),
    c: &mut Candidate,
) {
    for (i, stack) in caster.stacks.iter().enumerate().rev() {
        let v = raised(ruleset, spell, c, stack, hero);
        if v > c.value {
            c.value = v;
            c.target = Some(i);
        }
    }
}

/// A raise cast (`sim_cast_spell` 0x42610D; CPU-SIM-010): its stack gains [`raised`] over its
/// unit value creatures (toward 0) and that times its unit value in value; the side's strength
/// stays (survey §5.5 #66).
// rule: CPU-SIM-010
fn raise_apply(
    ruleset: &BattleRuleset,
    caster: &mut Side,
    (hero, spell): (&HeroMagic, SpellRef<'_>),
    c: &Candidate,
) {
    // A chosen raise has a target.
    let raising = c.target.and_then(|i| caster.stacks.get_mut(i));
    raising.into_iter().for_each(|stack| {
        let v = raised(ruleset, spell, c, stack, hero);
        let n = v.checked_div(stack.unit).unwrap_or(0);
        stack.count = stack.count.wrapping_add(n);
        stack.value = stack.value.wrapping_add(stack.unit.wrapping_mul(n));
    });
}

/// Whether `side` engages `enemy` in `round` (`sim_engage_now` 0x4267C0; CPU-SIM-004): not
/// when none of its living walkers reaches the enemy by then (rounds to contact 1 to `round`);
/// else a lookahead on copies for each engagement round e from 4 down to `round`: until e both
/// cast and only shoot, the losses over all stacks; from e to round 3 both cast, the side fights
/// with its stacks in contact (e's), taking its losses on them first, the enemy with all its
/// stacks in melee; then the stand-off ([`finish`]). Its score is the side's strength left, or
/// minus the enemy's when it has none; the highest score wins, a tie to the later e. True when
/// that is `round`.
// rule: CPU-SIM-004
fn engages(ruleset: &BattleRuleset, side: &Side, enemy: &Side, round: i32) -> bool {
    let reaches = side
        .stacks
        .iter()
        .any(|s| (1..=round).contains(&s.contact) && s.count > 0);
    if !reaches {
        return false;
    }
    let mut best = (0, FAR);
    for engage in (round..=FAR).rev() {
        let (mut own, mut other) = (side.clone(), enemy.clone());
        let mut at = round;
        while at < engage && own.strength > 0 && other.strength > 0 {
            casts(ruleset, &mut own, &mut other, at);
            let (dealt, taken) = (own.damage(0, false), other.damage(0, false));
            own.take_damage(taken, 0);
            other.take_damage(dealt, 0);
            at += 1;
        }
        let mut at = engage;
        while at < FAR && own.strength > 0 && other.strength > 0 {
            casts(ruleset, &mut own, &mut other, at);
            let (dealt, taken) = (own.damage(at, false), other.damage(FAR, true));
            own.take_damage(taken, at);
            other.take_damage(dealt, 0);
            at += 1;
        }
        finish(&mut own, &mut other);
        let score = if own.strength == 0 {
            other.strength.wrapping_neg()
        } else {
            own.strength
        };
        if engage == FAR || best.0 < score {
            best = (score, engage);
        }
    }
    best.1 == round
}

/// A simulated combat of `a` (the attacker) against `b` (`sim_combat_run` 0x426BC0;
/// CPU-SIM-005), both changed: rounds 1 to 3 while both strengths are above 0: each side's
/// choice ([`engages`]), the round's casts, then both sides' damage at once: both engaging or
/// only `b` (0x426D17), both with all their stacks in melee; only `a`, `a` with its stacks in
/// contact and `b` with all, `a`'s losses on its stacks in contact first; neither, the shots
/// alone; every other loss over all the stacks. Then the stand-off ([`finish`]).
// rule: CPU-SIM-005
pub fn run(ruleset: &BattleRuleset, a: &mut Side, b: &mut Side) {
    for round in 1..=ROUNDS {
        if a.strength <= 0 || b.strength <= 0 {
            break;
        }
        let engaging = (engages(ruleset, a, b, round), engages(ruleset, b, a, round));
        casts(ruleset, a, b, round);
        let (dealt, taken, contact) = match engaging {
            (true, false) => (a.damage(round, false), b.damage(FAR, true), round),
            (true, true) | (false, true) => (a.damage(FAR, true), b.damage(FAR, true), 0),
            (false, false) => (a.damage(0, false), b.damage(0, false), 0),
        };
        a.take_damage(taken, contact);
        b.take_damage(dealt, 0);
    }
    finish(a, b);
}
