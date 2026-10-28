//! Reward v1 (E05 T4; ADR-0025 point 8, `docs/epics/E05-log.md` §3): kill the most, lose the
//! least, spend mana sparingly, in one currency. For side s and its opponent o:
//!
//! ```text
//! r_s = (K_o − K_s − p · (m_s − m_o)) / S
//! ```
//!
//! * K, the value a side lost: its army at the start minus its survivors (army and raised
//!   stacks, so a permanent resurrection or a Pit Lord's Demons offset the loss), plus its war
//!   machines destroyed. The war machines are those the battle placed, as its records report them
//!   (a siege's Catapult, which no setup lists, included; a listed Ballista the battle leaves out,
//!   BAT-QUIRK-23, not: E05 T9b). Created stacks (summons, clones) count nothing on either side.
//! * m, the mana the side's hero spent (negative if it gained some); p, the price per spell
//!   point, in the currency.
//! * S, both sides' army and placed war machines' value at the start (at least 1), so the
//!   battle's size does not set the reward's.
//!
//! Nothing from the adventure counts (no hero, artifacts, treasure, town or gold). The sums are
//! integers (`i128`: no overflow for any setup), divided once in `f64` and reported as `f32`, so
//! `r_o = −r_s` exactly (zero-sum). It never feeds back into the battle (ADR-0003).

use h3_battle::{BattleState, StackTraits};
use h3_content::CreatureTypeId;
use h3_content_battle::BattleRuleset;
use h3_contracts::{Army, BattleOutcome, BattleSetup, BattleSide, StackOrigin, StackRecord};

/// The default price of a spell point: 10 `ai_value` (provisional, tuned in E06).
pub const DEFAULT_MANA_PRICE: u32 = 10;

/// What a creature is worth to the reward.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Currency {
    /// The creature's `ai_value` (harvested from the original's data).
    #[default]
    AiValue,
    /// The creature's cost in the ruleset's money: the resource surrenders are paid in (gold in
    /// the original's packs; other resources left out). A ruleset without surrenders has no money
    /// and every value is 0.
    Gold,
}

/// The reward's settings, reported with every run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RewardConfig {
    /// The currency.
    pub currency: Currency,
    /// The price of a spell point, p, in the currency.
    pub mana_price: u32,
}

impl Default for RewardConfig {
    fn default() -> Self {
        Self {
            currency: Currency::AiValue,
            mana_price: DEFAULT_MANA_PRICE,
        }
    }
}

/// A finished battle's balance in one currency, per side (indexed by `BattleSide::index`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Balance {
    /// The value each side lost, K.
    pub lost: [i128; 2],
    /// The mana each side's hero spent, m (0 without a hero; negative if it gained some).
    pub mana: [i128; 2],
    /// Both sides' value at the start, S (at least 1).
    pub scale: i128,
}

impl Balance {
    /// The balance of a battle that started from `setup` on `ruleset` and ended with `outcome`.
    pub fn of(
        ruleset: &BattleRuleset,
        setup: &BattleSetup,
        outcome: &BattleOutcome,
        currency: Currency,
    ) -> Self {
        let value = |creature: CreatureTypeId| creature_value(ruleset, currency, creature);
        let army = |army: &Army| -> i128 {
            army.stacks
                .iter()
                .map(|s| i128::from(s.count) * value(s.creature))
                .sum()
        };
        // The war machines the battle placed, at its start or its end.
        let machines = |records: &[StackRecord], count: fn(&StackRecord) -> u32| -> i128 {
            records
                .iter()
                .filter(|r| r.origin == StackOrigin::WarMachine)
                .map(|r| i128::from(count(r)) * value(r.creature))
                .sum()
        };
        let mut lost = [0; 2];
        let mut mana = [0; 2];
        let mut scale = 0;
        for side in 0..2 {
            let records = &outcome.stacks[side];
            let start =
                army([&setup.attacker, &setup.defender][side]) + machines(records, |r| r.start);
            let end = army(&outcome.survivors[side]) + machines(records, |r| r.end);
            lost[side] = start - end;
            scale += start;
            let spent = setup.heroes[side].as_ref().map_or(0, |hero| {
                let left = outcome.heroes[side].as_ref().map_or(0, |h| h.mana);
                i128::from(hero.magic.mana) - i128::from(left)
            });
            mana[side] = spent;
        }
        Self {
            lost,
            mana,
            scale: scale.max(1),
        }
    }

    /// A running battle's balance as if it ended now (E25 T2a: the potential of the training's
    /// shaping, [`potential`]): each side's losses by its stacks' current counts (its army's and
    /// raised stacks' and its war machines'; a created stack or a turret counts nothing), its
    /// hero's mana spent so far. At a decided battle's end it is [`Balance::of`]'s but for the
    /// end's own corrections (a temporary resurrection's creatures leave, a side left with only
    /// those keeps one: BAT-MAG-035, BAT-MAG-038), which reward v1 then pays. `state` must have
    /// started from `setup` on `ruleset`.
    pub fn standing(
        ruleset: &BattleRuleset,
        setup: &BattleSetup,
        state: &BattleState,
        currency: Currency,
    ) -> Self {
        let value = |creature: CreatureTypeId| creature_value(ruleset, currency, creature);
        let mut lost = [0; 2];
        let mut mana = [0; 2];
        let mut scale = 0;
        for side in BattleSide::BOTH {
            let i = side.index();
            let mut start: i128 = [&setup.attacker, &setup.defender][i]
                .stacks
                .iter()
                .map(|s| i128::from(s.count) * value(s.creature))
                .sum();
            let mut now = 0;
            let own = state.stacks().iter().filter(|s| s.side == side);
            for s in own.filter(|s| !s.is_turret() && !s.is_created()) {
                let v = value(s.creature);
                if s.has(StackTraits::WAR_MACHINE) {
                    start += i128::from(s.base_count) * v;
                }
                now += i128::from(s.count) * v;
            }
            lost[i] = start - now;
            scale += start;
            mana[i] = setup.heroes[i].as_ref().map_or(0, |hero| {
                let left = state.hero(side).map_or(0, |h| h.mana);
                i128::from(hero.magic.mana) - i128::from(left)
            });
        }
        Self {
            lost,
            mana,
            scale: scale.max(1),
        }
    }

    /// Each side's reward with `mana_price` per spell point: `(K_o − K_s − p · (m_s − m_o)) / S`.
    pub fn rewards(&self, mana_price: u32) -> [f32; 2] {
        let p = i128::from(mana_price);
        let attacker = self.lost[1] - self.lost[0] - p * (self.mana[0] - self.mana[1]);
        // Below 2^80 in magnitude: the conversions round, and the opponent's is the exact negation.
        let r = (attacker as f64 / self.scale as f64) as f32;
        [r, -r]
    }
}

/// The value of one `creature` in `currency` (0 for a handle outside the ruleset).
pub fn creature_value(
    ruleset: &BattleRuleset,
    currency: Currency,
    creature: CreatureTypeId,
) -> i128 {
    ruleset
        .creature_info(creature)
        .map_or(0, |info| match currency {
            Currency::AiValue => i128::from(info.ai_value),
            Currency::Gold => {
                let money = ruleset.battle().surrender.map(|s| s.resource);
                info.cost
                    .iter()
                    .filter(|(resource, _)| Some(*resource) == money)
                    .map(|&(_, amount)| i128::from(amount))
                    .sum()
            }
        })
}

/// Each side's reward for a battle that started from `setup` on `ruleset` and ended with
/// `outcome`, with `config` (see the module documentation).
pub fn rewards(
    config: RewardConfig,
    ruleset: &BattleRuleset,
    setup: &BattleSetup,
    outcome: &BattleOutcome,
) -> [f32; 2] {
    Balance::of(ruleset, setup, outcome, config.currency).rewards(config.mana_price)
}

/// Each side's potential in a running battle `state` that started from `setup` on `ruleset`
/// (E25 T2a): its reward v1 with `config` as if the battle ended now ([`Balance::standing`]), 0
/// at the start. A training's potential-based shaping adds its differences; it reads the whole
/// state, as the reward does, and is never an agent's input.
pub fn potential(
    config: RewardConfig,
    ruleset: &BattleRuleset,
    setup: &BattleSetup,
    state: &BattleState,
) -> [f32; 2] {
    Balance::standing(ruleset, setup, state, config.currency).rewards(config.mana_price)
}

#[cfg(test)]
mod tests;
