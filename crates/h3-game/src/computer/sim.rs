//! The simulated combat, its sides and its melee (E31 T2b; survey §2.2): the computer's estimate
//! of a fight in army-value units, not hit points, read from `sim_combat_start` 0x423EE0,
//! `sim_side_build` 0x424120 and `sim_side_walls` 0x424790 (a side, CPU-SIM-001), the losses
//! (`sim_spread_losses` 0x426170, `sim_side_take_damage` 0x426300, `sim_side_wipe` 0x4262B0,
//! `sim_apply_losses` 0x424880; CPU-SIM-002), the damage and the stand-off (`sim_side_damage`
//! 0x426390, `sim_side_hold_value` 0x426450, `sim_combat_finish` 0x4264D0; CPU-SIM-003), the
//! engagement's lookahead (`sim_engage_now` 0x4267C0; CPU-SIM-004) and the run
//! (`sim_combat_run` 0x426BC0; CPU-SIM-005). The stack vector's container code (0x4276C0,
//! 0x427750, 0x427780, the quicksort 0x427A90 and 0x427C30 that seven slots never reach) is a
//! `Vec`; the build's insertion sort a sort by unit value with the slot as its tie-breaker (it
//! is stable, the stacks built in slot order).
//!
//! Every value is the original's x87 arithmetic (`X87`, `SignedX87`: 53 bits, each sum truncated
//! by `crt_ftol` where the code truncates, the stand-offs' f32 stores) and its 32-bit integers,
//! wrapping where the code's `imul`, `add` and `sub` wrap. What the casts need (the side's spell
//! points, whether it may cast, the battle tile's ground, a stack's value per hit point and its
//! creature) comes with them in T2c, `AI-STUB-7`: until then each round's casts do nothing.
//!
//! No caller outside the tests yet: the fight estimate `ai_army_fight_sim` 0x427330 (T2c) is
//! the first.

use alloc::vec::Vec;
use core::num::NonZeroU32;

use h3_content::{CreatureFlag, CreatureSelector, CreatureStat, CreatureTypeId, FULL_BP, Ruleset};
use h3_content_battle::{Ability, BattleRuleset, SpellEffect};
use h3_contracts::{ArmyStack, BattleEffect, BattleHero, FortLevel, StatOp};
use h3_heroes::{BattleBonuses, HeroDetails, available_spells, derive_battle_hero};
use h3_kernel::arith::{DOUBLE_BITS, SINGLE_BITS, SignedX87, X87};
use h3_kernel::order::sort_by_unique_key;

use super::turn::counted;

#[cfg(test)]
mod tests;

/// The rounds a simulated combat fights before its stand-off (`sim_combat_run`'s 1 to 3).
const ROUNDS: i32 = 3;

/// The most rounds to contact (`sim_side_build`'s cap) and the lookahead's latest engagement
/// round (`sim_engage_now`'s first): a side "fights with all its stacks" at this round.
const FAR: i32 = 4;

/// The army slots a side's build reads (`sim_side_build`'s seven).
const SLOTS: u8 = 7;

/// The content a simulated combat reads: the core's creatures (their fight values) and the
/// battle part's (their hit points, speed, flags and abilities; the spells' effects).
#[derive(Debug, Clone, Copy)]
pub(super) struct Content<'a> {
    /// The core ruleset.
    pub(super) core: &'a Ruleset,
    /// The battle part.
    pub(super) battle: &'a BattleRuleset,
}

/// What a simulated side reads of a hero (the record's bytes `sim_side_build` 0x424120 and
/// `sim_side_walls` 0x424790 read, and the three hero functions they call), from the engine's
/// battle hero of its details (`h3_heroes::derive_battle_hero`, no visit bonuses: none of these
/// reads one). A hero the derivation refuses reads as a hero without skills or artifacts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct HeroReads {
    /// Its effective Attack (+0x476), a signed byte the build clamps to 0..99.
    attack: u8,
    /// Its effective Defense (+0x477), as Attack.
    defense: u8,
    /// Its Tactics level (+0xDC): the battle hero's.
    tactics: u8,
    /// Its Ballistics level (+0xD3): the battle hero's Catapult control, 0 without.
    ballistics: u8,
    /// Earthquake among its available spells (+0x43E): a spell with the battle part's
    /// `Earthquake` effect.
    earthquake: bool,
    /// `hero_archery` 0x4E4160: the Archery bonus as a single-precision value, the battle
    /// hero's ranged damage rate (the skill grown by its specialty, the bows that need the
    /// skill; 0 without it).
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
    /// What a side reads of the hero of `details`.
    pub(super) fn of(details: &HeroDetails, content: Content<'_>) -> Self {
        let hero = derive_battle_hero(details, BattleBonuses::NONE, content.core)
            .unwrap_or(BattleHero::new(details.hero_type));
        let earthquake = available_spells(details, content.core)
            .unwrap_or_default()
            .into_iter()
            .any(|spell| {
                content
                    .battle
                    .spell(spell)
                    .is_some_and(|s| s.magic.effects.contains(&SpellEffect::Earthquake))
            });
        let ranged = hero.damage.ranged;
        let archery = ranged
            .single
            .map(NonZeroU32::get)
            .map(X87::from_f32_bits)
            .unwrap_or(X87::ratio(ranged.bp, FULL_BP.into(), SINGLE_BITS).unwrap_or(X87::ZERO));
        let speed = hero.effects.iter().map(all_speed).sum();
        let hit_points = hero.effects.iter().filter_map(hit_point_change).collect();
        Self {
            attack: hero.attack,
            defense: hero.defense,
            tactics: hero.tactics,
            ballistics: hero.war_machines.catapult.map_or(0, |c| c.level),
            earthquake,
            archery,
            speed,
            hit_points,
        }
    }

    /// `hero_creature_health_bonus` 0x4E5B80: the hit points the hero adds to `creature` of
    /// `base` hit points, each change that selects it: an addition, or a share of `base` in
    /// basis points rounded down (the Elixir's quarter, `sar` of the count's hit points).
    fn health_bonus(&self, core: &Ruleset, creature: CreatureTypeId, base: u32) -> i64 {
        self.hit_points
            .iter()
            .filter(|(creatures, _, _)| creatures.matches(core, creature))
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
pub(super) struct Stack {
    /// Its army slot (+0x00).
    pub(super) slot: u8,
    /// Creatures alive (+0x08): the ceiling of the value over the unit value after a loss.
    pub(super) count: i32,
    /// The creature's speed and the hero's bonus (+0x10).
    pub(super) speed: i32,
    /// The share of its value dealt in a round of melee (+0x18).
    pub(super) melee: X87,
    /// The share of its value in the stand-off (+0x20).
    pub(super) hold: X87,
    /// A shooter's share of its value in a round of shots (+0x28), 0 for any other.
    pub(super) shot: X87,
    /// Rounds to contact (+0x38): 0 a shooter, else 1 to 4.
    pub(super) contact: i32,
    /// The value of one creature (+0x3C).
    pub(super) unit: i32,
    /// The stack's value (+0x40): unit value times count at the start, less its losses.
    pub(super) value: i32,
}

/// One side of a simulated combat (`sim_combat`, `docs/exe/structs.toml`): its stacks, weakest
/// unit first, and its strength.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Side {
    /// The stacks, by unit value, ties in slot order.
    pub(super) stacks: Vec<Stack>,
    /// The sum of the stacks' values less every loss taken (+0x1C): 0 the side is beaten.
    pub(super) strength: i32,
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

/// A simulated side (`sim_combat_start` 0x423EE0's army and walls, `sim_side_build` 0x424120;
/// CPU-SIM-001): `army` of the side's hero `heroes.0`, against `heroes.1`, facing a town of
/// `fort` (`FortLevel::None` without one), its value scaled by `factor`. Tactics is the hero's
/// level less the opponent's, at least 0; the factor is multiplied by sqrt(1 + 0.05 a) ×
/// sqrt(1 + 0.05 d), a the hero's Attack over the opponent's Defense and d its Defense over the
/// opponent's Attack (each clamped to 0..99 and the difference at least 0; without an opponent
/// its own), not without a hero. Per army slot with a creature, in slot order: its speed and its
/// hit points with the hero's bonuses; its unit value ftol(fight value × sqrt(hit points with
/// the bonus / hit points) × factor); its value that times its count, summed into the strength
/// (32 bits); rounds to contact 0 for a shooter, else (speed + 2 × (7 − Tactics)) / speed, at
/// most 4, raised to the walls' level for a walker; shares 0.2 of melee and 1 of the stand-off,
/// for a shooter 0.1 and 0.7 without `no_melee_penalty`, and of shots the hero's Archery / 5
/// (0.2 without a hero), twice for `strikes_twice`, halved facing walls without the battle
/// part's `no_wall_penalty` (the original names the Arch Mage, 35, alone). The stacks sorted by
/// unit value, ties in slot order.
// rule: CPU-SIM-001
pub(super) fn side(
    content: Content<'_>,
    army: &[ArmyStack],
    factor: X87,
    heroes: (Option<&HeroReads>, Option<&HeroReads>),
    fort: FortLevel,
) -> Side {
    let (own, opponent) = heroes;
    let (facing_walls, wall_level) = walls(fort, own);
    let level = |hero: Option<&HeroReads>| hero.map_or(0, |h| i32::from(h.tactics));
    let tactics = (level(own) - level(opponent)).max(0);
    let factor = own.map_or(factor, |hero| {
        let (attack, defense) = (counted(hero.attack, 0), counted(hero.defense, 0));
        let (a, d) = opponent.map_or((attack, defense), |o| {
            (
                attack.saturating_sub(counted(o.defense, 0)),
                defense.saturating_sub(counted(o.attack, 0)),
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
        let record = content.battle.creature(creature);
        let flags = record.map(|r| r.flags).unwrap_or_default();
        let hit_points = record.map_or(0, |r| r.hit_points);
        let bonus = own.map_or(0, |hero| {
            hero.health_bonus(content.core, creature, hit_points)
        });
        let health = u64::try_from(i64::from(hit_points) + bonus).unwrap_or(0);
        let speed = i32::from(record.map_or(0, |r| r.speed)).wrapping_add(speed_bonus);
        let fight = content.core.creature(creature).map_or(0, |c| c.fight_value);
        let root = X87::int(health)
            .over(X87::int(hit_points.into()))
            .unwrap_or(X87::ZERO)
            .sqrt();
        let unit = SignedX87::from(root.times(X87::int(fight.into())).times(factor)).ftol();
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
            let spared = content
                .battle
                .abilities(creature)
                .contains(&Ability::NoWallPenalty);
            if facing_walls && !spared {
                shots = shots.over(X87::int(2)).unwrap_or(X87::ZERO);
            }
        }
        strength = strength.wrapping_add(value);
        stacks.push(Stack {
            slot,
            count,
            speed,
            melee,
            hold,
            shot: shots,
            contact,
            unit,
            value,
        });
    }
    // The slots are unique: the key is.
    let _ = sort_by_unique_key(&mut stacks, |s| (s.unit, s.slot));
    Side { stacks, strength }
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

impl Side {
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
            let taken = if stack.value < share {
                let all = stack.value;
                stack.value = 0;
                stack.count = 0;
                all
            } else {
                stack.value -= share;
                stack.count = ceiling(stack.value, stack.unit);
                share
            };
            loss = loss.wrapping_sub(taken);
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
    pub(super) fn survivors(&self, army: &[ArmyStack]) -> Vec<ArmyStack> {
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
/// then the other's, each `sim_cast_spell` 0x425BD0.
fn casts(first: &mut Side, second: &mut Side, round: i32) {
    if first.max_speed() < second.max_speed() {
        cast(second, first, round);
        cast(first, second, round);
    } else {
        cast(first, second, round);
        cast(second, first, round);
    }
}

/// One side's cast in a round (`sim_cast_spell` 0x425BD0: the best affordable spell by its AI
/// category, T2c).
fn cast(_caster: &mut Side, _target: &mut Side, _round: i32) {
    // stub: AI-STUB-7
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
fn engages(side: &Side, enemy: &Side, round: i32) -> bool {
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
            casts(&mut own, &mut other, at);
            let (dealt, taken) = (own.damage(0, false), other.damage(0, false));
            own.take_damage(taken, 0);
            other.take_damage(dealt, 0);
            at += 1;
        }
        let mut at = engage;
        while at < FAR && own.strength > 0 && other.strength > 0 {
            casts(&mut own, &mut other, at);
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
pub(super) fn run(a: &mut Side, b: &mut Side) {
    for round in 1..=ROUNDS {
        if a.strength <= 0 || b.strength <= 0 {
            break;
        }
        let engaging = (engages(a, b, round), engages(b, a, round));
        casts(a, b, round);
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
