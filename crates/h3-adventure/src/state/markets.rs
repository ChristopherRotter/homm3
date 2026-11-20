//! The markets on the map (E08 T133a1; increment 14's design note part B): a Black Market's stock
//! drawn when the game is built (ADV-MKT-006, design note §9), called by
//! `draws::resolve_rewards`' second walk in object order, and its sold entries in the visits'
//! block (§8). In a module of its own, as the banks' draw (the release build's codegen units,
//! `TRAPS.md`). Since E08 T133a2 the adventure's checks and changes of a market's answers (§11):
//! a sale (ADV-MKT-005) and a purchase (ADV-MKT-007, A-46 fixed as `ADV-QUIRK-48`), which the
//! game checks against the treasury, pays and makes (ADR-0017 point 10, as the hire's); the visit
//! itself is `AdventureState::visit`'s (`AdventureOutcome::service`).

use alloc::sync::Arc;
use alloc::vec::Vec;

use h3_content::{ArtifactTypeId, ResourceId, Ruleset};
use h3_contracts::HeroId;
use h3_heroes::{HeroChange, HeroDetails, check_change};
use h3_kernel::Rng;

use super::draws::ArtifactPool;
use super::{AdventureState, HeroOnMap, MapObject, Resolved};
use crate::army::{ArmyRef, ArmySlot};
use crate::command::{MarketError, MarketVisit};
use crate::movement::details_of;
use crate::ruleset::{AdventureRuleset, Market, MarketScreen, ObjectKind};

/// The rules a purchase's check reads: the core, the ruleset part and the game's store of hero
/// details.
type Rules<'a> = (&'a Ruleset, &'a AdventureRuleset, &'a [Arc<HeroDetails>]);

/// A sale [`AdventureState::check_sale`] accepted (ADV-MKT-005; E08 T133a2, increment 14's design
/// note §11): so many creatures of a hero's stack for so much of a resource, which the game adds
/// to the treasury (saturating, GAM-RES-001) before [`AdventureState::sell`] lowers the stack. It
/// holds for the state that checked it; not `Clone`, so one check sells once (the hire's form).
#[derive(Debug, PartialEq, Eq)]
pub struct Sale {
    /// The selling hero.
    hero: HeroId,
    /// Its army's slot.
    slot: u8,
    /// The creatures sold.
    count: u32,
    /// The resource gained and how much.
    gain: (ResourceId, u32),
}

impl Sale {
    /// The resource the sale gives and how much (lots times the lot's units, saturating).
    pub const fn gain(&self) -> (ResourceId, u32) {
        self.gain
    }

    /// The creatures it takes from the stack.
    pub const fn count(&self) -> u32 {
        self.count
    }
}

/// A purchase [`AdventureState::check_buy`] accepted (ADV-MKT-007; E08 T133a2, increment 14's
/// design note §11): a Black Market's entry for its price in a resource, which the game checks
/// against the treasury and pays, then gives the artifact to the hero (`HeroChange::AddArtifact`,
/// which the check found room for) and makes the purchase with [`AdventureState::buy`]. Not
/// `Clone`, so one check buys once.
#[derive(Debug, PartialEq, Eq)]
pub struct MarketPurchase {
    /// The buying hero.
    hero: HeroId,
    /// The market's object index.
    index: usize,
    /// The entry bought.
    entry: u8,
    /// Its artifact.
    artifact: ArtifactTypeId,
    /// The price's resource and amount.
    price: (ResourceId, u32),
}

impl MarketPurchase {
    /// The price: its resource and amount.
    pub const fn price(&self) -> (ResourceId, u32) {
        self.price
    }

    /// The artifact bought.
    pub const fn artifact(&self) -> ArtifactTypeId {
        self.artifact
    }

    /// The hero it is given to.
    pub const fn hero(&self) -> HeroId {
        self.hero
    }
}

/// The amount of `resource` in `cost` (0 for none): a creature's or an artifact's value for the
/// market, its cost in the resource the caller names (increment 14's design note §2: "a creature
/// of gold cost c", "an artifact of gold cost c").
fn cost_in(cost: &[(ResourceId, u32)], resource: ResourceId) -> u32 {
    cost.iter()
        .find(|&&(r, _)| r == resource)
        .map_or(0, |&(_, amount)| amount)
}

/// A Black Market's stock (ADV-MKT-006, E08 T133a1; increment 14's design note §9, survey
/// `E08-trades.md` §7.1: `black_market_draw` 0x4C0E31 in `setup_map_objects`' walk): for each
/// class of its kind's stock in order (the base pack's three treasures, three minors and one
/// major), one artifact of that class from the random artifacts' pool (ADV-OBJ-006, the original's
/// `random_artifact_of_level`: the bans and the artifacts given so far marked, so the markets
/// repeat neither each other nor the map's other random artifacts until a class has none left,
/// then the marks restart from the bans); a draw that finds none (every artifact banned) leaves its
/// entry out. Never drawn again: no week, month or visit restocks it (survey §7.1).
// rule: ADV-MKT-006
pub(super) fn stock_draw(
    market: &Market,
    artifacts: &mut ArtifactPool<'_>,
    draws: &mut Rng,
) -> Resolved {
    let stock: Vec<_> = market
        .stock
        .iter()
        .filter_map(|class| artifacts.draw(&[*class], draws))
        .collect();
    Resolved::Stock(stock)
}

impl AdventureState {
    /// The market `visit` opened, if its window shows `screen`: its hero on the map
    /// ([`MarketError::Visit`] else), its object a market's ([`MarketError::Visit`] else), the
    /// screen its kind's ([`MarketError::Screen`] else).
    fn market<'p>(
        &self,
        part: &'p AdventureRuleset,
        visit: &MarketVisit,
        screen: MarketScreen,
    ) -> Result<(&'p Market, &MapObject, &HeroOnMap), MarketError> {
        let hero = self.hero(visit.hero);
        let object = self.object(visit.index);
        let kind = object
            .and_then(|object| object.object_type)
            .and_then(|ty| part.object_type(ty))
            .map(|ty| &ty.kind);
        let (Some(hero), Some(object), Some(ObjectKind::Market(market))) = (hero, object, kind)
        else {
            return Err(MarketError::Visit);
        };
        if market.shows(screen) {
            Ok((market, object, hero))
        } else {
            Err(MarketError::Screen)
        }
    }

    /// Checks a sale of `lots` lots of the creatures in slot `slot` of `visit`'s hero's army for
    /// `take` (ADV-MKT-005; E08 T133a2, increment 14's design note §11; survey `E08-trades.md`
    /// §6, the original's `market_sell_creatures_proc` 0x5EE890), each creature valued at its
    /// cost in `value` (the caller's resource for a creature's cost: the original's gold, ADV-Q60).
    /// The lot is the market parameters' at the kind's count (`MarketParams::creature_lot`,
    /// ADV-MKT-002): `give` creatures for `take` units, one of them 1. Refused, in this order: the
    /// visit ([`MarketError::Visit`]), a window without the sale ([`MarketError::Screen`]), a
    /// slot without a stack ([`MarketError::Slot`]), no lot ([`MarketError::NoPrice`]: a
    /// creature of no cost, a resource the market does not value), lots of 0 or more creatures
    /// than the stack's ([`MarketError::Amount`]), the army's lone stack sold whole
    /// ([`MarketError::LastCreature`]: the army keeps a creature, so the most is the count less
    /// one when the army's other slots are empty).
    // rule: ADV-MKT-005
    pub fn check_sale(
        &self,
        (core, part): (&Ruleset, &AdventureRuleset),
        value: ResourceId,
        visit: &MarketVisit,
        (slot, take, lots): (u8, ResourceId, u32),
    ) -> Result<Sale, MarketError> {
        let (market, _, hero) = self.market(part, visit, MarketScreen::Sell)?;
        let stack = hero
            .army
            .iter()
            .find(|stack| stack.slot == slot)
            .ok_or(MarketError::Slot(slot))?;
        let cost = core
            .creature(stack.creature)
            .map_or(0, |creature| cost_in(&creature.cost, value));
        let lot = part
            .params()
            .market()
            .and_then(|rates| rates.creature_lot(market.count, cost, take))
            .ok_or(MarketError::NoPrice)?;
        let count = u64::from(lots) * u64::from(lot.give);
        if lots == 0 || count > u64::from(stack.count) {
            return Err(MarketError::Amount(lots));
        }
        if hero.army.len() == 1 && count == u64::from(stack.count) {
            return Err(MarketError::LastCreature);
        }
        let gain = u64::from(lots) * u64::from(lot.take);
        Ok(Sale {
            hero: visit.hero,
            slot,
            count: u32::try_from(count).unwrap_or(stack.count),
            gain: (take, u32::try_from(gain).unwrap_or(u32::MAX)),
        })
    }

    /// Makes `sale` (ADV-MKT-005; E08 T133a2), which [`Self::check_sale`] gave for this state as
    /// it stands, once the game added its gain: the hero's stack loses the creatures sold, its
    /// slot emptied at 0. Returns the slot that changed, which the game tells (`StackChanged`); no
    /// adventure event (increment 14's design note §12).
    // rule: ADV-MKT-005
    pub fn sell(&mut self, sale: Sale) -> ArmySlot {
        // The checked hero and slot: no branch for a hero or a slot the check did not find.
        self.hero_mut(sale.hero).into_iter().for_each(|hero| {
            let sold = hero.army.iter_mut().filter(|stack| stack.slot == sale.slot);
            sold.for_each(|stack| stack.count = stack.count.saturating_sub(sale.count));
            hero.army.retain(|stack| stack.count > 0);
        });
        ArmySlot {
            army: ArmyRef::Hero(sale.hero),
            slot: sale.slot,
        }
    }

    /// Checks a purchase of the Black Market's entry `entry` paid in `pay` (ADV-MKT-007; E08
    /// T133a2, increment 14's design note §11; survey `E08-trades.md` §7.2, the original's
    /// `market_buy_artifact_proc` 0x5ED9E0), the artifact valued at its cost in `value` (the
    /// caller's resource for an artifact's cost: the original's gold, ADV-Q60). The price is the
    /// market parameters' at the kind's count (`MarketParams::artifact_price`, ADV-MKT-002).
    /// Refused, in this order: the visit ([`MarketError::Visit`]), a window without the purchase
    /// ([`MarketError::Screen`]), an entry past the stock or bought ([`MarketError::Entry`]), no
    /// price ([`MarketError::NoPrice`]: an artifact of no cost, a price that rounds to 0, a
    /// resource the market does not value), and a hero that cannot receive the artifact
    /// ([`MarketError::NoRoom`]: `HeroChange::AddArtifact`'s check, HER-CHG-002, a position that
    /// takes it else the backpack below its most; not the pickups' full-backpack count, A-25).
    /// The original pays and empties the entry whatever its `hero_give_artifact` did, so a hero
    /// without room lost the price and the artifact (A-46, `ADV-QUIRK-48`): fixed (E08 decision
    /// 366), refused before paying.
    // rule: ADV-MKT-007
    pub fn check_buy(
        &self,
        (core, part, heroes): Rules<'_>,
        value: ResourceId,
        visit: &MarketVisit,
        (entry, pay): (u8, ResourceId),
    ) -> Result<MarketPurchase, MarketError> {
        let (market, object, _) = self.market(part, visit, MarketScreen::Buy)?;
        let artifact = object
            .resolved
            .stock()
            .get(usize::from(entry))
            .filter(|_| !self.is_sold(visit.index, entry))
            .copied()
            .ok_or(MarketError::Entry(entry))?;
        let cost = core
            .artifacts()
            .get(artifact.index())
            .map_or(0, |ty| cost_in(&ty.cost, value));
        let price = part
            .params()
            .market()
            .and_then(|rates| rates.artifact_price(market.count, cost, pay))
            .ok_or(MarketError::NoPrice)?;
        let change = HeroChange::AddArtifact(artifact.into());
        if !details_of(heroes, visit.hero).is_some_and(|d| check_change(d, core, change).is_ok()) {
            return Err(MarketError::NoRoom);
        }
        Ok(MarketPurchase {
            hero: visit.hero,
            index: visit.index,
            entry,
            artifact,
            price: (pay, price),
        })
    }

    /// Makes `purchase` (ADV-MKT-007; E08 T133a2), which [`Self::check_buy`] gave for this state
    /// as it stands, once the game paid its price and gave the artifact: the entry is sold (the
    /// visits' block's bit, increment 14's design note §8), shown empty to the market's later
    /// visitors and never bought again.
    // rule: ADV-MKT-007
    pub fn buy(&mut self, purchase: MarketPurchase) {
        self.visits_mut().sell(purchase.index, purchase.entry);
    }

    /// Whether the entry `entry` of the Black Market at object index `index` was bought
    /// (ADV-MKT-007; increment 14's design note §8): its sold bit in the visits' block, none
    /// before a purchase.
    pub fn is_sold(&self, index: usize, entry: u8) -> bool {
        self.visits
            .as_ref()
            .is_some_and(|visits| visits.is_sold(index, entry))
    }
}
