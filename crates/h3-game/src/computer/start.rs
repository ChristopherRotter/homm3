//! The turn's start records (E31 T2d; survey §1.5, §1.6, §2.5): what `ai_turn_start` 0x4297C0 and
//! `ai_turn_start_values` 0x527960 write into the computer's record, its heroes and the enemy
//! heroes before the first pass ([`Record`], CPU-STA-001): the threat fold over the enemies'
//! reach with the towns' capture estimate (CPU-STA-004, -005), the recruit reserve (CPU-STA-002),
//! the resource weights (CPU-STA-003) and the heroes' experience weight (CPU-STA-006). The policy
//! keeps them in its memory (ADR-0030 point 5): the values read them in its turn, and its answers
//! read the last start's in the other players' turns.
//!
//! Not here: the Eye value, 0 while the engine has no Eye of the Magi (`AI-STUB-10`; the vision
//! circle with T2h); the threat fold's search is the engine's reach (`AI-STUB-11`) and its
//! defence recruits wait for increment 3 (`AI-STUB-12`); the heroes' spell gains (`AI-STUB-13`,
//! T2e), the artifacts' mean (`AI-STUB-14`, T2f) and the Grail target (`AI-STUB-15`, T2i). The
//! difficulty's weights are a function of the difficulty alone (`fight::blend`).

use alloc::vec;
use alloc::vec::Vec;

use h3_adventure::{HeroOnMap, allied};
use h3_battle::estimate::EstimateHero;
use h3_content::{CreatureTypeId, ResourceId};
use h3_contracts::{HeroId, PlayerId, TownId};
use h3_kernel::arith::{DOUBLE_BITS, SINGLE_BITS, SignedX87};
use h3_towns::{BuildingSet, CREATURE_INDEXES, Town, index_creature};

use super::fight::{self, Fighter};
use super::order::library_sort;
use super::turn::towns_of;
use crate::{GameRuleset, GameState};

/// The points beyond an enemy hero's day's maximum its search runs with (0x4281CC: 800).
const REACH_BEYOND: u32 = 800;

/// An enemy hero's threat when it would take one of the computer's towns, before the division by
/// their count (0x4283CF: 5 000 000).
const THREAT: i32 = 5_000_000;

/// The day of the week from which the demand counts the next week's growth (0x428934: 5).
const GROWTH_FROM: u8 = 5;

/// The stacks of the creature list whose full cost the demand adds (0x428BFD: 3).
const DEAREST: usize = 3;

/// The bounds of the Marketplaces' count the rate's row is read at (0x428C4E: 1 to 10).
const MARKETS: (u8, u8) = (1, 10);

/// The weights summed into the player's mean weight (0x428D78: the first six) and its divisor
/// (0x428D98: 5).
const MEAN: (usize, i32) = (6, 5);

/// The town record's key of the Marketplace (`town_building_allowed` 0x5C12A0 with building 14):
/// the town part gives it no role yet (the town lane's inbox).
const MARKETPLACE: &str = "marketplace";

/// The f32 bit pattern of +∞: a hero's experience weight with no experience to its next level.
const INFINITE: u32 = 0x7F80_0000;

/// What the computer's turn start writes (CPU-STA-001; survey §1.6: the record's +0x04 to +0x7F,
/// the player's +0x128 and +0x160, the heroes' +0x109 and +0x11D, the towns' +0x3), by resource in
/// the core's order where per resource. The weights (+0x60, f64) are kept as their inputs, the
/// means, the demand and the rate's row ([`Record::weights`] computes them as the start did).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct Record {
    /// The Eye value (+0x04): 0 (`AI-STUB-10`).
    pub(super) eyes: i32,
    /// The recruit reserve (+0x08, CPU-STA-002).
    pub(super) reserve: Vec<i32>,
    /// The means (+0x24): the stock plus twice the day's income (CPU-STA-003).
    pub(super) means: Vec<i32>,
    /// The demand (+0x40, CPU-STA-003).
    pub(super) demand: Vec<i32>,
    /// The market rates' row of the weights (1 to 10: the towns that may build a Marketplace).
    pub(super) markets: u8,
    /// The player's mean weight (+0x160).
    pub(super) mean: i32,
    /// Each of the player's heroes on the map with its experience weight (+0x109, an f32 bit
    /// pattern; CPU-STA-006), in ascending id order.
    pub(super) experience: Vec<(HeroId, u32)>,
    /// Each enemy hero that would take one of the player's towns, with its threat (+0x11D,
    /// CPU-STA-004; every other enemy hero's is 0), in ascending id order.
    pub(super) threats: Vec<(HeroId, i32)>,
    /// Each of the player's towns that an enemy hero would take, with how many (+0x3; every other
    /// town's is 0), in ascending id order.
    pub(super) towns: Vec<(TownId, u8)>,
}

/// The records of `player`'s turn start (`ai_turn_start` 0x4297C0, `ai_turn_start_values`
/// 0x527960; survey §1.5; CPU-STA-001), in the original's order: the Eye value (`AI-STUB-10`), the
/// defence fold (its recruits `AI-STUB-12`) and the threat fold ([`threats`]), the recruit reserve
/// ([`reserve`]), the resource weights ([`weights`]), the Grail target (`AI-STUB-15`); then each
/// hero's experience weight ([`experience`]; its spell gains `AI-STUB-13`) and the artifacts'
/// mean (`AI-STUB-14`). Nothing in it writes the state, so each reads the start's state.
// rule: CPU-STA-001
pub(super) fn record(state: &GameState, rules: &GameRuleset, player: PlayerId) -> Record {
    // stub: AI-STUB-10
    // stub: AI-STUB-12
    let (threats, towns) = threats(state, rules, player);
    let reserve = reserve(state, rules, player);
    let mut record = weights(state, rules, player);
    // stub: AI-STUB-15
    // stub: AI-STUB-13
    // stub: AI-STUB-14
    record.reserve = reserve;
    record.threats = threats;
    record.towns = towns;
    record.experience = experience(state, rules, player);
    record
}

/// The threat fold's result: [`Record::threats`] and [`Record::towns`].
type Threats = (Vec<(HeroId, i32)>, Vec<(TownId, u8)>);

/// The threat fold (`ai_enemy_reach_fold` 0x4280E0 with `ai_town_threat_vtable` 0x63B670 through
/// `ai_reach_fold_towns` 0x4282B0; CPU-STA-004): every town's count cleared and every enemy hero's
/// threat; then from each hero on the map of a player not of `player`'s team, its search with its
/// day's maximum now plus 800 (`AdventureState::reach_with`, the engine's reach: `AI-STUB-11`),
/// and for each of `player`'s towns whose entrance it reaches and that it would take
/// ([`fight::captures`]), the hero's threat 5 000 000 over `player`'s count of towns and the
/// town's count one more (a byte). The engine has no defeated player before E10.
// rule: CPU-STA-004
fn threats(state: &GameState, rules: &GameRuleset, player: PlayerId) -> Threats {
    let own: Vec<TownId> = towns_of(state, player).collect();
    let Some(threat) = i32::try_from(own.len())
        .ok()
        .and_then(|count| THREAT.checked_div(count))
    else {
        return (Vec::new(), Vec::new());
    };
    let (core, part) = (rules.core(), rules.adventure());
    let adventure = state.adventure();
    let entrances: Vec<_> = own
        .iter()
        .map(|&town| adventure.map().town_entrance(town))
        .collect();
    let mut counts = vec![0_u8; own.len()];
    let mut threats = Vec::new();
    let enemies = adventure
        .heroes()
        .iter()
        .filter(|hero| !allied(hero.owner, player))
        .filter_map(|hero| state.hero(hero.id).map(|details| (hero, details)));
    for (hero, details) in enemies {
        // stub: AI-STUB-11
        let reach = adventure
            .day_maximum_of(core, part, state.heroes(), hero.id)
            .and_then(|most| {
                let points = most.saturating_add(REACH_BEYOND);
                adventure.reach_with(core, part, state.heroes(), (hero.id, points))
            });
        let enemy = Fighter {
            on_map: hero,
            details,
        };
        // The hero's battle reads derived once, at its first town reached (`bench-check`).
        let mut attacker = None;
        let mut threatens = false;
        for ((&town, entrance), count) in own.iter().zip(&entrances).zip(&mut counts) {
            let reached = entrance
                .zip(reach.as_ref())
                .is_some_and(|(pos, reach)| reach.points_left(pos).is_some());
            if reached {
                let (battle, spells) =
                    attacker.get_or_insert_with(|| fight::battle_hero(rules, enemy));
                let reads = EstimateHero {
                    hero: battle,
                    spells,
                };
                if fight::captures(state, rules, (&hero.army, reads), town) {
                    threatens = true;
                    *count = count.wrapping_add(1);
                }
            }
        }
        if threatens {
            threats.push((hero.id, threat));
        }
    }
    let _ = h3_kernel::order::sort_by_unique_key(&mut threats, |&(hero, _)| hero);
    let towns = own
        .into_iter()
        .zip(counts)
        .filter(|&(_, count)| count > 0)
        .collect();
    (threats, towns)
}

/// A creature among a town's available ones or the demand's list (the original's 12-byte army
/// entry: the creature, the value at +4, the count at +8).
#[derive(Debug, Clone, Copy)]
struct Entry {
    /// The creature.
    creature: usize,
    /// Its value: the count times the creature's AI value, as each list computes it.
    value: i32,
    /// The count (an `i16`).
    count: i16,
}

/// A count as the original's 16-bit fields hold it.
fn short(count: u16) -> i16 {
    i16::from_ne_bytes(count.to_ne_bytes())
}

/// An amount as the original's 32-bit fields hold it.
fn long(amount: u32) -> i32 {
    i32::from_ne_bytes(amount.to_ne_bytes())
}

/// The full cost of `entry`'s stack added into `costs` by resource (its creature's cost times the
/// count, 32 bits; `creature_cost_array` 0x54E7C0's seven i32).
fn add_cost(rules: &GameRuleset, entry: Entry, costs: &mut [i32]) {
    let creature = CreatureTypeId::from_index(entry.creature)
        .and_then(|creature| rules.core().creature(creature));
    let cost = creature.map_or(&[][..], |creature| &creature.cost[..]);
    for &(resource, amount) in cost {
        costs.get_mut(resource.index()).into_iter().for_each(|sum| {
            *sum = sum.wrapping_add(long(amount).wrapping_mul(entry.count.into()));
        });
    }
}

/// The recruit reserve (`ai_recruit_reserve` 0x429AD0; CPU-STA-002): per resource the most, over
/// `player`'s towns, of what a town's two most valuable available stacks cost. A town's stacks are
/// its indexes with creatures available, each valued at the low 16 bits of its creature's AI value
/// times the count in 16 bits (wrapping), sorted lowest first by the library's `std::sort` (equal values
/// keep their index order); the last two count. A town with a single stack counts none (the
/// loop's bound is compared unsigned, 0x429CC6).
// rule: CPU-STA-002
fn reserve(state: &GameState, rules: &GameRuleset, player: PlayerId) -> Vec<i32> {
    let (core, part) = (rules.core(), rules.towns());
    let mut reserve = vec![0_i32; core.resources().len()];
    for town in towns_of(state, player).filter_map(|town| state.towns().get(town)) {
        let record = part.town(town.faction());
        let entries: Vec<Entry> = (0..CREATURE_INDEXES)
            .filter_map(|index| {
                let count = short(town.available(index));
                let creature = record.and_then(|r| index_creature(r, index))?;
                let ai = core.creature(creature).map_or(0, |c| c.ai_value);
                (count > 0).then(|| Entry {
                    creature: creature.index(),
                    value: reserve_value(ai, count),
                    count,
                })
            })
            .collect();
        for (most, cost) in reserve.iter_mut().zip(two_dearest(rules, entries)) {
            *most = (*most).max(cost);
        }
    }
    reserve
}

/// A town's stack's value in the reserve's list (0x429BD4): the low 16 bits of the creature's AI
/// value times the count, in 16 bits (wrapping), widened.
fn reserve_value(ai: u32, count: i16) -> i32 {
    let bytes = ai.to_le_bytes();
    i16::from_le_bytes([bytes[0], bytes[1]])
        .wrapping_mul(count)
        .into()
}

/// What a town's two most valuable stacks among `entries` cost by resource (0x429C02 to
/// 0x429D00): the entries sorted lowest first by the library's `std::sort`, the last two's full costs;
/// nothing for fewer than two (the loop's bound `n − 2` is compared unsigned, so one stack counts
/// none).
fn two_dearest(rules: &GameRuleset, mut entries: Vec<Entry>) -> Vec<i32> {
    library_sort(&mut entries, &|a: &Entry, b: &Entry| a.value < b.value);
    let mut costs = vec![0_i32; rules.core().resources().len()];
    let len = entries.len();
    if len >= 2 {
        entries[len - 2..]
            .iter()
            .for_each(|&entry| add_cost(rules, entry, &mut costs));
    }
    costs
}

/// The buildings `town` could build now, its day's mark aside (`town_buildable_now` 0x5C0F20,
/// which does not read the mark): the hall's test of the town without the mark
/// ([`h3_towns::buildable`]) with no Capitol held, less a Capitol that the hall's possible set
/// leaves out because its owner holds one ([`GameState::town_offer`]'s `possible`).
fn buildable_now(
    state: &GameState,
    rules: &GameRuleset,
    player: PlayerId,
    (id, town): (TownId, &Town),
) -> BuildingSet {
    let unmarked = Town::new(town.faction(), town.built(), town.forbidden());
    let part = rules.towns();
    state
        .town_offer(rules, player, id)
        .map_or(BuildingSet::EMPTY, |offer| {
            h3_towns::buildable(&unmarked, part, false).intersection(offer.possible)
        })
}

/// The means, the demand, the rate's row and the mean weight of `player` (`ai_resource_weights`
/// 0x428740; CPU-STA-003; survey §1.5):
///
/// - the means: per resource the treasury plus twice the day's income (32 bits);
/// - the demand: per resource the largest cost of a building any of its towns could build now
///   ([`buildable_now`]), plus the full costs of the first three entries of a list of every
///   creature of the core, in its order, with its available count summed over the towns' indexes
///   (from the week's fifth day each with the index's growth added, `h3_towns::growth`; 16 bits),
///   valued at the count times the AI value (32 bits) and sorted highest first by the library's
///   sort;
/// - the rate's row: the towns whose record lists a Marketplace not forbidden, at least 1 and at
///   most 10;
/// - the mean: the first six weights ([`Record::weights`]) summed, each sum truncated, over 5.
// rule: CPU-STA-003
fn weights(state: &GameState, rules: &GameRuleset, player: PlayerId) -> Record {
    let (core, part) = (rules.core(), rules.towns());
    let resources = core.resources().len();
    let week_start = state.weekday(rules).get() == 1;
    let income = state.day_income(rules, player, week_start);
    let treasury = state.treasury(player).unwrap_or_default();
    let means: Vec<i32> = (0..resources)
        .map(|r| {
            let stock = long(treasury.get(r).copied().unwrap_or(0));
            let daily = long(income.get(r).copied().unwrap_or(0));
            stock.wrapping_add(daily.wrapping_mul(2))
        })
        .collect();
    let mut demand = vec![0_i32; resources];
    let growing = state.weekday(rules).get() >= GROWTH_FROM;
    let inputs = if growing {
        state.growth_inputs(rules)
    } else {
        Vec::new()
    };
    let mut counts = vec![0_i16; core.creature_count()];
    let mut markets = 0_u8;
    let towns =
        towns_of(state, player).filter_map(|id| state.towns().get(id).map(|town| (id, town)));
    for (id, town) in towns {
        let record = part.town(town.faction());
        for building in buildable_now(state, rules, player, (id, town)).iter() {
            let cost = record
                .and_then(|r| r.building(building))
                .map_or(&[][..], |b| &b.cost[..]);
            for &(resource, amount) in cost {
                demand
                    .get_mut(resource.index())
                    .into_iter()
                    .for_each(|most| *most = (*most).max(long(amount)));
            }
        }
        let market = record.and_then(|r| r.building_by_key(MARKETPLACE));
        if market.is_some_and(|market| !town.forbidden().contains(market)) {
            markets = markets.saturating_add(1);
        }
        for index in 0..CREATURE_INDEXES {
            let mut count = short(town.available(index));
            if growing {
                let inputs = inputs.get(id.index()).copied().unwrap_or_default();
                let growth = h3_towns::growth(town, part, core, index, &inputs);
                count = count.wrapping_add(short(growth));
            }
            let creature = record.and_then(|r| index_creature(r, index));
            let slot = creature.and_then(|c| counts.get_mut(c.index()));
            slot.filter(|_| count > 0)
                .into_iter()
                .for_each(|sum| *sum = sum.wrapping_add(count));
        }
    }
    for (most, cost) in demand.iter_mut().zip(three_dearest(rules, counts)) {
        *most = most.wrapping_add(cost);
    }
    let mut record = Record {
        means,
        demand,
        markets: markets.clamp(MARKETS.0, MARKETS.1),
        ..Record::default()
    };
    let (summed, divisor) = MEAN;
    let sum = record
        .weights(rules)
        .into_iter()
        .take(summed)
        .fold(0_i32, |sum, weight| {
            SignedX87::int(sum.into()).plus(weight).ftol()
        });
    record.mean = sum / divisor;
    record
}

/// What the demand's three dearest creatures cost by resource (0x4288A3 to 0x428C06): a list of
/// every creature of the core in its order with its count of `counts` (16 bits), valued at the
/// count times its AI value (32 bits, wrapping), sorted highest first by the library's `std::sort`; the
/// full costs of its first three.
fn three_dearest(rules: &GameRuleset, counts: Vec<i16>) -> Vec<i32> {
    let core = rules.core();
    let mut entries: Vec<Entry> = (0..=u16::MAX)
        .map(usize::from)
        .zip(counts)
        .map(|(creature, count)| {
            let ai = CreatureTypeId::from_index(creature)
                .and_then(|c| core.creature(c))
                .map_or(0, |c| long(c.ai_value));
            Entry {
                creature,
                value: i32::from(count).wrapping_mul(ai),
                count,
            }
        })
        .collect();
    library_sort(&mut entries, &|a: &Entry, b: &Entry| a.value > b.value);
    let mut costs = vec![0_i32; core.resources().len()];
    for &entry in entries.iter().take(DEAREST) {
        add_cost(rules, entry, &mut costs);
    }
    costs
}

/// Whether `a` is greater than `b` (`fcomp`'s C0 and C3 both clear).
fn greater(a: SignedX87, b: SignedX87) -> bool {
    match (a.is_negative(), b.is_negative()) {
        (false, true) => true,
        (true, false) => false,
        (false, false) => a.magnitude() > b.magnitude(),
        (true, true) => a.magnitude() < b.magnitude(),
    }
}

/// A resource's weight factor (`ai_resource_weights` 0x428CA2; CPU-STA-003) from its means `m`,
/// its demand `d` and the rate `r` (an f32 widened): no demand, r; a demand at most the means,
/// ((m − d) × r + d) / m; more than the means, d, over m when m is above 1, at most 1 / r; in
/// double precision, each stored result an f64. A division by 0 of the original (no rate, or
/// means of 0 below a negative demand) leaves the factor uncapped or 0.
// rule: CPU-STA-003
fn factor(means: i32, demand: i32, rate: SignedX87) -> SignedX87 {
    let (m, d) = (SignedX87::int(means.into()), SignedX87::int(demand.into()));
    if demand == 0 {
        return rate;
    }
    if demand <= means {
        let surplus = SignedX87::int(means.wrapping_sub(demand).into());
        let sum = surplus.times(rate).plus(d).round(DOUBLE_BITS);
        return sum
            .over(m)
            .map_or(SignedX87::ZERO, |q| q.round(DOUBLE_BITS));
    }
    let share = if means > 1 {
        d.over(m).map_or(d, |q| q.round(DOUBLE_BITS))
    } else {
        d
    };
    let limit = SignedX87::ONE.over(rate).map(|l| l.round(DOUBLE_BITS));
    limit
        .filter(|&limit| greater(share, limit))
        .unwrap_or(share)
}

impl Record {
    /// The resource weights (+0x60, the player's +0x128; `ai_resource_weights` 0x428740;
    /// CPU-STA-003): per resource its [`factor`] times its value (the market's `resource_value`
    /// 0x5ECD10), in double precision, at the rate of the record's row (the market's resources'
    /// factor, an f32; none without the market's parameters).
    pub(super) fn weights(&self, rules: &GameRuleset) -> Vec<SignedX87> {
        let market = rules.adventure().params().market();
        let rate = market
            .and_then(|m| m.factor_bits(self.markets.min(m.counts())))
            .map_or(SignedX87::ZERO, |[resources, _, _]| {
                SignedX87::from_f32_bits(resources)
            });
        (0..=u16::MAX)
            .map(ResourceId::from_raw)
            .zip(self.means.iter().zip(&self.demand))
            .map(|(resource, (&means, &demand))| {
                let value = market.and_then(|m| m.value(resource)).unwrap_or(0);
                SignedX87::int(value.into())
                    .times(factor(means, demand, rate))
                    .round(DOUBLE_BITS)
            })
            .collect()
    }
}

/// The experience weight of `hero` (`ai_hero_effectiveness` 0x527760's +0x109; CPU-STA-006): what
/// one experience point is worth to it, `fight::experience_value` of its army's value stored as
/// an f32 (+∞ with no experience to its next level).
// rule: CPU-STA-006
fn experience_weight(state: &GameState, rules: &GameRuleset, hero: &HeroOnMap) -> u32 {
    let next = state.hero(hero.id).map_or(0, |details| {
        h3_heroes::next_level_experience(details, rules.core())
    });
    fight::experience_value(fight::value_of(rules, &hero.army), next)
        .map_or(INFINITE, |value| value.round(SINGLE_BITS).to_f32_bits())
}

/// Each of `player`'s heroes on the map with its experience weight (`ai_turn_start_values`
/// 0x527960 over the player's list of heroes, the garrison heroes not in it), by id.
fn experience(state: &GameState, rules: &GameRuleset, player: PlayerId) -> Vec<(HeroId, u32)> {
    let mut weights: Vec<(HeroId, u32)> = state
        .adventure()
        .heroes()
        .iter()
        .filter(|hero| hero.owner == player)
        .map(|hero| (hero.id, experience_weight(state, rules, hero)))
        .collect();
    let _ = h3_kernel::order::sort_by_unique_key(&mut weights, |&(hero, _)| hero);
    weights
}

#[cfg(test)]
mod tests;
