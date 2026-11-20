//! The Hill Forts on the map (E08 T134b1; increment 14's design note §4, §10, §11, §15; survey
//! `docs/rules/surveys/E08-trades.md` §13): each stack's row of a visitor's army (ADV-HFT-002:
//! its upgrade in the enabled packs and that upgrade's price for the stack, the original's
//! `hill_fort_rows` 0x4E7EB0 and `upgrade_cost` 0x54E750), and the adventure's checks and change
//! of an upgrade, which the game pays (ADR-0017 point 10, as the market's and the hire's: the
//! treasury is the game's). The visit itself is `AdventureState::visit`'s
//! (`AdventureOutcome::service`, after the team's kind mark, ADV-HFT-001).

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use h3_content::{CreatureTypeId, ResourceId, Ruleset};
use h3_contracts::{ArmyStack, HeroId};

use super::{AdventureState, HeroOnMap};
use crate::army::{ArmyRef, ArmySlot};
use crate::command::{HillFortError, HillFortVisit};
use crate::market::MarketParams;
use crate::ruleset::{AdventureRuleset, HillFort, ObjectKind};

/// The rules a Hill Fort's checks read: the core and the ruleset part.
type Rules<'a> = (&'a Ruleset, &'a AdventureRuleset);

/// A row of a Hill Fort's window (ADV-HFT-002; E08 T134b1, increment 14's design note §11, §15):
/// a stack of the visitor's army, its creature's upgrade and that upgrade's price for the whole
/// stack. A row without an upgrade is the original's state 0; which of the others the treasury
/// affords is the game's (GAM-DEC-013: states 1 and 2), read at each step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HillFortRow {
    /// The hero's army's slot.
    pub slot: u8,
    /// The stack's creature.
    pub creature: CreatureTypeId,
    /// The stack's count.
    pub count: u32,
    /// The creature's upgrade in the ruleset (the first of its upgrades), none for a creature
    /// without one (a neutral, an upgraded one, or under the base pack alone the four elementals:
    /// E08 decision 378).
    pub upgrade: Option<CreatureTypeId>,
    /// The upgrade's price for the stack, by resource in the core's order, each above 0: none for
    /// a free one (a level whose gold percent is 0, the original's level 1) or without an upgrade.
    pub price: Vec<(ResourceId, u32)>,
}

/// An upgrade [`AdventureState::check_upgrade`] or [`AdventureState::check_upgrade_all`] accepted
/// (ADV-HFT-002; E08 T134b1): the stack it upgrades and its price, which the game checks against
/// the treasury and pays (GAM-DEC-013, ADR-0004) before [`AdventureState::upgrade`] makes it. It
/// holds for the state that checked it; not `Clone`, so one check upgrades once (the market's
/// form).
#[derive(Debug, PartialEq, Eq)]
pub struct HillFortUpgrade {
    /// The upgrading hero.
    hero: HeroId,
    /// Its army's slot.
    slot: u8,
    /// The creature the stack becomes.
    upgrade: CreatureTypeId,
    /// The price, by resource in the core's order ([`HillFortRow::price`]).
    price: Vec<(ResourceId, u32)>,
}

impl HillFortUpgrade {
    /// The hero.
    pub const fn hero(&self) -> HeroId {
        self.hero
    }

    /// The price, by resource in the core's order (none for a free one).
    pub fn price(&self) -> &[(ResourceId, u32)] {
        &self.price
    }
}

/// The price of upgrading `count` `creature`s into `upgrade` at `fort` (ADV-HFT-002; increment
/// 14's design note §4, survey §13: the original's `upgrade_cost` 0x54E750 and
/// `hill_fort_gold_factors` 0x63EB4C, D-74 the original's): for each resource max(0, the
/// upgrade's cost − the creature's) × the count, then the gold alone (the part's cost resource,
/// [`MarketParams::cost`]: the original's gold) times its level's percent, rounded down once for
/// the stack; a level whose percent is 0 free whole (the original computes no price at level 1).
/// The percents are quarters, so this is the original's `trunc(f32(gold) × factor)` for a gold
/// below 2^24 (*agent's call*, design note §4). In 64 bits, saturated to 32 (as the hire's price).
// rule: ADV-HFT-002
fn upgrade_price(
    (core, part): Rules<'_>,
    fort: &HillFort,
    (creature, upgrade): (CreatureTypeId, CreatureTypeId),
    count: u32,
) -> Vec<(ResourceId, u32)> {
    let (from, to) = (core.creature(creature), core.creature(upgrade));
    // A level outside the dwellings' seven pays whole (none in the packs: every level 1 to 7).
    let level = from.map_or(0, |info| usize::from(info.level));
    let percent = level
        .checked_sub(1)
        .and_then(|at| fort.gold.get(at))
        .map_or(100, |&percent| u64::from(percent));
    if percent == 0 {
        return Vec::new();
    }
    let gold = part.params().market().map(MarketParams::cost);
    let base = |resource: ResourceId| {
        from.and_then(|info| info.cost.iter().find(|&&(r, _)| r == resource))
            .map_or(0, |&(_, each)| each)
    };
    let cost = to.map_or(&[][..], |info| &info.cost[..]);
    let by_resource: BTreeMap<ResourceId, u32> = cost
        .iter()
        .map(|&(resource, each)| {
            let more = u64::from(each.saturating_sub(base(resource))) * u64::from(count);
            let scaled = if Some(resource) == gold {
                more * percent / 100
            } else {
                more
            };
            (resource, u32::try_from(scaled).unwrap_or(u32::MAX))
        })
        .filter(|&(_, amount)| amount > 0)
        .collect();
    by_resource.into_iter().collect()
}

/// The row of `stack` at `fort` ([`HillFortRow`]).
fn row(rules: Rules<'_>, fort: &HillFort, stack: &ArmyStack) -> HillFortRow {
    let upgrade = rules.0.upgrades(stack.creature).first().copied();
    let price = upgrade.map_or_else(Vec::new, |upgrade| {
        upgrade_price(rules, fort, (stack.creature, upgrade), stack.count)
    });
    HillFortRow {
        slot: stack.slot,
        creature: stack.creature,
        count: stack.count,
        upgrade,
        price,
    }
}

impl AdventureState {
    /// The Hill Fort `visit` opened: its kind and its hero on the map ([`HillFortError::Visit`]
    /// else).
    fn hill_fort<'a>(
        &'a self,
        part: &'a AdventureRuleset,
        visit: &HillFortVisit,
    ) -> Result<(&'a HillFort, &'a HeroOnMap), HillFortError> {
        let kind = self
            .object(visit.index)
            .and_then(|object| object.object_type)
            .and_then(|ty| part.object_type(ty))
            .map(|ty| &ty.kind);
        match (kind, self.hero(visit.hero)) {
            (Some(ObjectKind::HillFort(fort)), Some(hero)) => Ok((fort, hero)),
            _ => Err(HillFortError::Visit),
        }
    }

    /// The rows of a Hill Fort's window as the state stands (ADV-HFT-002; E08 T134b1, increment
    /// 14's design note §11, §15; survey `E08-trades.md` §13, the original's `hill_fort_rows`
    /// 0x4E7EB0): one per stack of the visitor's army in its order, each with its creature's
    /// upgrade in the ruleset and the upgrade's price for the stack ([`HillFortRow`]). Read at
    /// each view and answer, never stored.
    ///
    /// # Errors
    ///
    /// The visit's hero is not on the map or its object no Hill Fort ([`HillFortError::Visit`]).
    // rule: ADV-HFT-001, ADV-HFT-002
    pub fn hill_fort_rows(
        &self,
        rules: Rules<'_>,
        visit: &HillFortVisit,
    ) -> Result<Vec<HillFortRow>, HillFortError> {
        let (fort, hero) = self.hill_fort(rules.1, visit)?;
        Ok(hero
            .army
            .iter()
            .map(|stack| row(rules, fort, stack))
            .collect())
    }

    /// Checks the upgrade of the stack at `slot` (ADV-HFT-002; E08 T134b1, increment 14's design
    /// note §11, §15; survey §13, the original's `hill_fort_upgrade_slot` 0x4E8500): its row's
    /// upgrade and price. The treasury is the game's (GAM-DEC-013: `CannotPay`).
    ///
    /// # Errors
    ///
    /// In this order: the visit ([`HillFortError::Visit`]), a slot without a stack or a stack
    /// without an upgrade ([`HillFortError::NoUpgrade`]); nothing changed.
    // rule: ADV-HFT-002
    pub fn check_upgrade(
        &self,
        rules: Rules<'_>,
        visit: &HillFortVisit,
        slot: u8,
    ) -> Result<HillFortUpgrade, HillFortError> {
        let (fort, hero) = self.hill_fort(rules.1, visit)?;
        let row = hero
            .army
            .iter()
            .find(|stack| stack.slot == slot)
            .map(|stack| row(rules, fort, stack));
        let Some((upgrade, price)) = row.and_then(|row| Some((row.upgrade?, row.price))) else {
            return Err(HillFortError::NoUpgrade(slot));
        };
        Ok(HillFortUpgrade {
            hero: visit.hero,
            slot,
            upgrade,
            price,
        })
    }

    /// Checks the upgrade of every stack that upgrades, in slot order (ADV-HFT-002; E08 T134b1,
    /// increment 14's design note §15; survey §13, the original's All, item 204): each row's
    /// upgrade and price, which the game sums against the treasury (the whole's state) and pays
    /// row by row.
    ///
    /// # Errors
    ///
    /// In this order: the visit ([`HillFortError::Visit`]), no stack that upgrades
    /// ([`HillFortError::Nothing`]: the whole's state 0); nothing changed.
    // rule: ADV-HFT-002
    pub fn check_upgrade_all(
        &self,
        rules: Rules<'_>,
        visit: &HillFortVisit,
    ) -> Result<Vec<HillFortUpgrade>, HillFortError> {
        let upgrades: Vec<HillFortUpgrade> = self
            .hill_fort_rows(rules, visit)?
            .into_iter()
            .filter_map(|row| {
                Some(HillFortUpgrade {
                    hero: visit.hero,
                    slot: row.slot,
                    upgrade: row.upgrade?,
                    price: row.price,
                })
            })
            .collect();
        if upgrades.is_empty() {
            return Err(HillFortError::Nothing);
        }
        Ok(upgrades)
    }

    /// Makes `upgrade` (ADV-HFT-002; E08 T134b1), which a check gave for this state as it stands
    /// and the game paid: the stack's creature replaced by its upgrade, the count kept. The slot
    /// the game tells (`StackChanged`); no adventure event.
    // rule: ADV-HFT-002
    pub fn upgrade(&mut self, upgrade: HillFortUpgrade) -> ArmySlot {
        let HillFortUpgrade {
            hero,
            slot,
            upgrade: creature,
            ..
        } = upgrade;
        // The checked hero and stack: no branch for one the check did not find.
        self.hero_mut(hero)
            .into_iter()
            .flat_map(|on_map| on_map.army.iter_mut())
            .filter(|stack| stack.slot == slot)
            .for_each(|stack| stack.creature = creature);
        ArmySlot {
            army: ArmyRef::Hero(hero),
            slot,
        }
    }
}
