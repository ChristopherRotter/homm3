//! The computer's fight estimate (E31 T2c; survey `docs/rules/surveys/E31-computer-player.md`
//! §2.2): `ai_army_fight_sim` 0x427330, what a fight of one of its heroes is worth in army-value
//! units, on the simulated combat of `h3_battle::estimate` (AI-Q34), with Necromancy after a win
//! (`sim_necromancy` 0x426DF0), the experience's value (`ai_experience_value` 0x527710) and the
//! enemy's value by its owner (`ai_difficulty_blend` 0x428710). It changes nothing. Since T2d
//! beside it the town's capture estimate of the turn's start (`ai_town_capture_sim` 0x428410,
//! [`captures`]), the first caller of the simulated combat outside the tests.
//!
//! The fight estimate's callers are the values (T2h's object value) and the danger map (T2i,
//! `super::plan`).

use alloc::vec::Vec;

use h3_adventure::{HeroOnMap, Pos, army_value};
use h3_battle::estimate::{self, EstimateHero, Ground};
use h3_content::{CreatureTypeId, FULL_BP, SpellId};
use h3_contracts::{ArmyStack, BattleHero, FortLevel, TownId};
use h3_heroes::{
    BattleBonuses, HeroDetails, Necromancy, available_spells, derive_battle_hero,
    gained_experience, max_mana, necromancy, next_level_experience,
};
use h3_kernel::arith::{DOUBLE_BITS, SINGLE_BITS, SignedX87, X87};

use crate::{GameRuleset, GameState};

/// A lost fight's value (0x4274B4).
pub(super) const LOST: i32 = -1_000_000_000;

/// The computer's price of a hero (`hero_price` 0x67814C): the experience value's base.
const HERO_PRICE: i64 = 2500;

/// The army slots the original's army record holds (Necromancy's loop, `army_add_stack`).
const SLOTS: u8 = 7;

/// The enemy's factor by the difficulty when a human owns either hero or the town
/// (`ai_human_enemy_factor` 0x6604D0, quarters): 0.5, 0.5, 1.0, 1.25, 1.25.
const HUMAN_ENEMY: [u32; 5] = [2, 2, 4, 5, 5];

/// The enemy's factor among computers and neutrals (0x42738E: 1.25, in quarters).
const COMPUTER_ENEMY: u32 = 5;

/// The capture estimate's factors, in quarters (0x4284F4: the attacker's 1.25, 0x428506: the
/// garrison's 0.75; f64 constants pushed whole).
const CAPTURE_FACTORS: (u32, u32) = (5, 3);

/// Necromancy's bonus on its share (0x426E1D: the f32 0.02).
const NECROMANCY_BONUS: u32 = 0x3CA3_D70A;

/// A hero in a fight: its place and army on the map and its details.
#[derive(Debug, Clone, Copy)]
pub(super) struct Fighter<'a> {
    /// The hero on the map (its owner and army).
    pub(super) on_map: &'a HeroOnMap,
    /// Its details (its skills, artifacts, spells, level).
    pub(super) details: &'a HeroDetails,
}

/// The other side of a fight the computer estimates (`ai_army_fight_sim`'s edx and its
/// arguments): an enemy hero, its army (a hero's, a monster's, a guard's, a garrison's) and the
/// town the fight is at.
#[derive(Debug, Clone, Copy)]
pub(super) struct Opponent<'a> {
    /// The enemy hero, if one fights.
    pub(super) hero: Option<Fighter<'a>>,
    /// The army it fights with.
    pub(super) army: &'a [ArmyStack],
    /// The town whose walls the hero faces, if any.
    pub(super) town: Option<TownId>,
}

/// The enemy's factor (CPU-SIM-012): with a human owner of either hero or of the town the
/// difficulty's from `ai_human_enemy_factor` (the computer underestimates humans below Expert),
/// else 1.25.
// rule: CPU-SIM-012
fn enemy_factor(difficulty: u8, human: bool) -> X87 {
    let quarters = if human {
        HUMAN_ENEMY
            .get(usize::from(difficulty))
            .copied()
            .unwrap_or(COMPUTER_ENEMY)
    } else {
        COMPUTER_ENEMY
    };
    X87::ratio(quarters, 4, DOUBLE_BITS).unwrap_or(X87::ONE)
}

/// The weight of a colour's value by its owner (`ai_difficulty_blend` 0x428710 with the weights
/// `ai_turn_start_values` 0x527960 sets each turn; CPU-SIM-012): 0 for none; for a human
/// (`human` true) −0.4 (f32) on Easy, else (d + 1) / 4; for a computer 1.0 on Easy, else
/// 0.75 − d / 4.
// rule: CPU-SIM-012
pub(super) fn blend(difficulty: u8, human: Option<bool>) -> SignedX87 {
    let d = i64::from(difficulty);
    match (human, difficulty) {
        (None, _) => SignedX87::ZERO,
        (Some(true), 0) => SignedX87::from_f32_bits(0xBECC_CCCD),
        (Some(true), _) => SignedX87::int(d + 1)
            .over(SignedX87::int(4))
            .unwrap_or(SignedX87::ZERO),
        (Some(false), 0) => SignedX87::int(1),
        (Some(false), _) => SignedX87::int(3 - d)
            .over(SignedX87::int(4))
            .unwrap_or(SignedX87::ZERO),
    }
}

/// What one experience point is worth to a hero (`ai_experience_value` 0x527710; CPU-SIM-012):
/// (f32(2 500) + f32(its army's value after the fight)) / f32(40 × the experience to its next
/// level), in double precision; none when that is 0 (the original's infinity).
// rule: CPU-SIM-012
pub(super) fn experience_value(value: i32, next: u32) -> Option<SignedX87> {
    let f32_of = |n: i64| SignedX87::int(n).round(SINGLE_BITS);
    let gap = i32::from_ne_bytes(next.wrapping_mul(40).to_ne_bytes());
    f32_of(HERO_PRICE)
        .plus(f32_of(value.into()))
        .over(f32_of(gap.into()))
}

/// The experience the defeat of `army` gives (`army_experience_points` 0x4CA3B0 without a hero):
/// each stack's count times its creature's hit points, in 32 bits.
fn experience_points(rules: &GameRuleset, army: &[ArmyStack]) -> i32 {
    army.iter().fold(0_i32, |sum, stack| {
        let hit_points = rules
            .battle()
            .creature(stack.creature)
            .map_or(0, |c| c.hit_points);
        let count = i32::from_ne_bytes(stack.count.to_ne_bytes());
        sum.wrapping_add(count.wrapping_mul(i32::from_ne_bytes(hit_points.to_ne_bytes())))
    })
}

/// An army's value as the original sums it (`army_value` 0x44AC80, 32 bits).
pub(super) fn value_of(rules: &GameRuleset, army: &[ArmyStack]) -> i32 {
    i32::try_from(army_value(rules.core(), army)).unwrap_or(i32::MAX)
}

/// The creatures a winner's Necromancy raises from `enemy` (`sim_necromancy` 0x426DF0;
/// CPU-SIM-011), `share` its Necromancy (`hero_necromancy_power`, f32), `raised` the creature and
/// `raised_hp` its hit points, `hit_points` the hit points of a creature: nothing without a share
/// above 0; else with p = f32(share + 0.02), over the enemy's stacks as the army holds them,
/// ftol(f32(min(the creature's hit points, the raised creature's) × count) × p / f32(the raised
/// creature's)), at most the count; the sum, at least 1.
// rule: CPU-SIM-011
fn necromancy_raises(
    share: X87,
    raised_hp: u32,
    enemy: &[ArmyStack],
    hit_points: impl Fn(CreatureTypeId) -> u32,
) -> Option<u32> {
    if share == X87::ZERO {
        return None;
    }
    let p = share
        .plus(X87::from_f32_bits(NECROMANCY_BONUS))
        .round(SINGLE_BITS);
    let own = X87::int(raised_hp.into()).round(SINGLE_BITS);
    let sum = enemy.iter().fold(0_u32, |sum, stack| {
        let hp = hit_points(stack.creature).min(raised_hp);
        let health = X87::int(u64::from(hp.wrapping_mul(stack.count))).round(SINGLE_BITS);
        let raised = health.times(p).over(own).map_or(0, X87::trunc);
        sum.wrapping_add(u32::try_from(raised).unwrap_or(u32::MAX).min(stack.count))
    });
    Some(sum.max(1))
}

/// Adds `count` creatures of `creature` to `army` (`army_add_stack` 0x44ACE0 with no slot): to
/// its stack of the creature, else to its first empty slot; nothing without either.
fn add_stack(army: &mut Vec<ArmyStack>, creature: CreatureTypeId, count: u32) {
    if let Some(stack) = army.iter_mut().find(|s| s.creature == creature) {
        stack.count = stack.count.saturating_add(count);
        return;
    }
    let empty = (0..SLOTS).find(|&slot| army.iter().all(|s| s.slot != slot));
    empty.into_iter().for_each(|slot| {
        army.push(ArmyStack {
            slot,
            creature,
            count,
        });
    });
}

/// A win's value before the enemy's (0x427547; CPU-SIM-012): the experience's value less the
/// value lost (`before` less `after`, 32 bits); a net loss under a quarter of `before` counts 0.
// rule: CPU-SIM-012
fn settle(gained: i32, before: i32, after: i32) -> i32 {
    let lost = before.wrapping_sub(after);
    let net = gained.wrapping_sub(lost);
    if net < 0 && lost.wrapping_mul(4) < before {
        0
    } else {
        net
    }
}

/// The winner's Necromancy into its survivors (`sim_necromancy` 0x426DF0; CPU-SIM-011): its
/// share (`hero_necromancy_power`: its single-precision value, else its basis points as an f32)
/// raises [`necromancy_raises`] of its creature from `enemy`, added to `survivors`
/// ([`add_stack`]); nothing without Necromancy.
// rule: CPU-SIM-011
fn raise_into(
    rules: &GameRuleset,
    survivors: &mut Vec<ArmyStack>,
    necromancy: Option<Necromancy>,
    enemy: &[ArmyStack],
) {
    let battle = rules.battle();
    let hit_points = |c| battle.creature(c).map_or(0, |c| c.hit_points);
    necromancy.into_iter().for_each(|n| {
        let share = n
            .share
            .single
            .map(core::num::NonZeroU32::get)
            .map(X87::from_f32_bits)
            .unwrap_or(X87::ratio(n.share.bp, FULL_BP.into(), SINGLE_BITS).unwrap_or(X87::ZERO));
        let raised = necromancy_raises(share, hit_points(n.creature), enemy, hit_points);
        raised
            .into_iter()
            .for_each(|count| add_stack(survivors, n.creature, count));
    });
}

/// What a fighter's battle reads of it: its battle hero (no visit bonuses) with its maximum
/// spell points (HER-MAN-001: the caster record's mana, E31 T2e; the simulated combat reads the
/// points it has) and its available spells.
pub(super) fn battle_hero(rules: &GameRuleset, fighter: Fighter<'_>) -> (BattleHero, Vec<SpellId>) {
    let core = rules.core();
    let details = fighter.details;
    let mut hero = derive_battle_hero(details, BattleBonuses::NONE, core)
        .unwrap_or(BattleHero::new(details.hero_type));
    hero.magic.max_mana = max_mana(details, core);
    (hero, available_spells(details, core).unwrap_or_default())
}

/// The battle tile `pos` as the simulated combat reads it (`sim_combat_start` 0x423EE0's
/// `tile_special_ground` 0x4FCE20): its special ground's battlefield and an anti-magic garrison.
pub(super) fn ground(state: &GameState, rules: &GameRuleset, pos: Pos) -> Ground {
    let adventure = state.adventure();
    Ground {
        battlefield: adventure
            .special_ground(rules.adventure(), pos)
            .and_then(|key| rules.battle().battlefield_by_key(key.as_str())),
        no_casting: adventure.anti_magic(rules.adventure(), pos),
    }
}

/// The walls of `town` (`sim_side_walls` 0x424790 reads the town's fort): its fort level, none
/// for a town the store lacks.
fn fort(state: &GameState, rules: &GameRuleset, town: TownId) -> FortLevel {
    state.towns().get(town).map_or(FortLevel::None, |town| {
        h3_towns::fort_level(town, rules.towns())
    })
}

/// Whether an enemy hero would take `town` (`ai_town_capture_sim` 0x428410; CPU-STA-005), `army`
/// its army and `reads` its hero's battle reads ([`battle_hero`]): a simulated combat
/// ([`estimate::run`]) of the army, its factor 1.25, with its hero, facing the town's walls,
/// against the army in the town's upper bar (`town_garrison_army` 0x5C1460: its garrison hero's,
/// else its garrison), its factor 0.75, with no hero (a garrison hero's skills and spells do not
/// enter), both on the town's entrance (its special ground and anti-magic garrison); true when the
/// hero's side keeps a strength above 0. No draws.
// rule: CPU-STA-005
pub(super) fn captures(
    state: &GameState,
    rules: &GameRuleset,
    (army, reads): (&[ArmyStack], EstimateHero<'_>),
    town: TownId,
) -> bool {
    let adventure = state.adventure();
    let ground = adventure
        .map()
        .town_entrance(town)
        .map_or(Ground::default(), |pos| ground(state, rules, pos));
    let quarters = |q| X87::ratio(q, 4, DOUBLE_BITS).unwrap_or(X87::ONE);
    let (attacker, defender) = CAPTURE_FACTORS;
    let battle = rules.battle();
    let walls = fort(state, rules, town);
    let mut a = estimate::side(
        battle,
        army,
        quarters(attacker),
        (Some(reads), None),
        walls,
        ground,
    );
    let garrison = adventure.upper_bar(town);
    let mut b = estimate::side(
        battle,
        garrison,
        quarters(defender),
        (None, Some(reads)),
        FortLevel::None,
        ground,
    );
    estimate::run(battle, &mut a, &mut b);
    a.strength() > 0
}

/// What a fight of `hero` against `opponent` on `tile` is worth to the computer
/// (`ai_army_fight_sim` 0x427330; CPU-SIM-012): a simulated combat ([`estimate::run`]) of the
/// hero's army, its factor its aggressiveness (1.0, `AI-STUB-8`; halved for the victory
/// condition's hero, `AI-STUB-9`), facing the town's walls, against the opponent's army with its
/// hero, its factor [`enemy_factor`] (a human owner of either hero or of the town); the tile's
/// special ground and anti-magic garrison for both. A loss is [`LOST`]. A win: the enemy army's
/// experience points (no hero's 500) grown by the hero's Learning (`gained_experience`, HER-EXP-001)
/// times their value ([`experience_value`] of the hero's surviving army, truncated), with its
/// Necromancy's creatures added to the survivors ([`necromancy_raises`], from the enemy's whole
/// army), [`settle`]d against the value lost; with an enemy hero or a town, plus the enemy
/// army's value times [`blend`] of its owner (the town's, else the hero's; truncated). The
/// 5 000 000 for the victory condition's or the loss condition's hero waits for the conditions
/// (`AI-STUB-9`).
// rule: CPU-SIM-012
pub(super) fn fight(
    state: &GameState,
    rules: &GameRuleset,
    hero: Fighter<'_>,
    opponent: Opponent<'_>,
    tile: Option<Pos>,
) -> i32 {
    let adventure = state.adventure();
    let difficulty = adventure.difficulty();
    let town_owner = opponent.town.and_then(|town| {
        let map = adventure.map();
        map.town_object(town)
            .and_then(|index| adventure.owner(index))
    });
    let human = |player| state.human(player).unwrap_or(false);
    // A town's owner, none for a neutral one, else the hero's (0x427577).
    let owner = if opponent.town.is_some() {
        town_owner
    } else {
        opponent.hero.map(|h| h.on_map.owner)
    };
    let any_human = human(hero.on_map.owner)
        || opponent.hero.is_some_and(|h| human(h.on_map.owner))
        || town_owner.is_some_and(human);
    // stub: AI-STUB-8
    // stub: AI-STUB-9
    let own_factor = X87::ONE;
    let enemy = enemy_factor(difficulty, any_human);
    let fort = opponent
        .town
        .map_or(FortLevel::None, |town| fort(state, rules, town));
    let ground = tile.map_or(Ground::default(), |pos| ground(state, rules, pos));
    let (own, own_spells) = battle_hero(rules, hero);
    let theirs = opponent.hero.map(|h| battle_hero(rules, h));
    let own = EstimateHero {
        hero: &own,
        spells: &own_spells,
    };
    let other = theirs
        .as_ref()
        .map(|(hero, spells)| EstimateHero { hero, spells });
    let battle = rules.battle();
    let army = &hero.on_map.army;
    let mut a = estimate::side(battle, army, own_factor, (Some(own), other), fort, ground);
    let mut b = estimate::side(
        battle,
        opponent.army,
        enemy,
        (other, Some(own)),
        FortLevel::None,
        ground,
    );
    estimate::run(battle, &mut a, &mut b);
    if a.strength() == 0 {
        return LOST;
    }
    let mut survivors = a.survivors(army);
    let points = u32::try_from(experience_points(rules, opponent.army)).unwrap_or(0);
    let grown = gained_experience(hero.details, rules.core(), points);
    let grown = SignedX87::int(grown.into()).round(SINGLE_BITS);
    let next = next_level_experience(hero.details, rules.core());
    let gained = experience_value(value_of(rules, &survivors), next)
        .map_or(i32::MIN, |v| v.times(grown).ftol());
    let raising = necromancy(hero.details, rules.core());
    raise_into(rules, &mut survivors, raising, opponent.army);
    let value = settle(gained, value_of(rules, army), value_of(rules, &survivors));
    if opponent.hero.is_none() && opponent.town.is_none() {
        return value;
    }
    let weight = blend(difficulty, owner.map(human));
    let worth = SignedX87::int(value_of(rules, opponent.army).into()).round(SINGLE_BITS);
    worth
        .times(weight)
        .plus(SignedX87::int(value.into()).round(SINGLE_BITS))
        .ftol()
}

#[cfg(test)]
mod tests;
