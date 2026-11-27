//! The markets on the map (E08 T133a1; increment 14's design note part B): a Black Market's stock
//! drawn when the game is built (ADV-MKT-006, design note §9), called by
//! `draws::resolve_rewards`' second walk in object order, and its sold entries in the visits'
//! block (§8). In a module of its own, as the banks' draw (the release build's codegen units,
//! `TRAPS.md`). Since E08 T133a2 the adventure's checks and changes of a market's answers (§11):
//! a sale (ADV-MKT-005) and a purchase (ADV-MKT-007, A-46 fixed as `ADV-QUIRK-48`), which the
//! game checks against the treasury, pays and makes (ADR-0017 point 10, as the hire's); the visit
//! itself is `AdventureState::visit`'s (`AdventureOutcome::service`). Since E08 T133b a trade's
//! lot (ADV-MKT-003), a transfer's screen (ADV-MKT-004) and what the window offers its visitor
//! (ADV-MKT-008, `MarketOffers`), the game's decision's (GAM-DEC-011). Since E09 T58a1 (the town
//! lane's, `cross-lane`; the town epic's design note §104 to §106) every check reads a resolved
//! window ([`MarketWindow`]: a map market's from its kind, a town's from the game), so a town's
//! window trades at the trader's count, sells its hero's creatures and sells the Artifact
//! Merchants' stock (TWN-MKT-002 to -004).

use alloc::sync::Arc;
use alloc::vec::Vec;

use h3_content::{ArtifactTypeId, ResourceId, Ruleset};
use h3_contracts::HeroId;
use h3_heroes::{HeroChange, HeroDetails, check_change};
use h3_kernel::Rng;

use super::pool::ArtifactPool;
use super::{AdventureState, HeroOnMap, Resolved};
use crate::army::{ArmyRef, ArmySlot};
use crate::command::{MarketError, MarketPlace, MarketVisit};
use crate::market::Lot;
use crate::movement::details_of;
use crate::ruleset::{AdventureRuleset, Market, MarketScreen, ObjectKind};

/// The rules a purchase's check reads: the core, the ruleset part and the game's store of hero
/// details.
type Rules<'a> = (&'a Ruleset, &'a AdventureRuleset, &'a [Arc<HeroDetails>]);

/// A sale [`AdventureState::check_sale`] accepted (ADV-MKT-005; E08 T133a2, increment 14's design
/// note §11): so many creatures of a hero's stack for so much of a resource, which the game adds
/// to the treasury (saturating, GAM-RES-001) once [`AdventureState::sell`] lowered the stack (the
/// events' order, design note §13: `StackChanged`, then `ResourceGained`; E08 T133b). It
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
/// design note §11): a Black Market's entry (since E09 T58a1 the Artifact Merchants' too, TWN-MKT-004) for its price in a resource, which the game checks
/// against the treasury and pays, then gives the artifact to the hero (`HeroChange::AddArtifact`,
/// which the check found room for) and makes the purchase with [`AdventureState::buy`]. Not
/// `Clone`, so one check buys once.
#[derive(Debug, PartialEq, Eq)]
pub struct MarketPurchase {
    /// The buying hero.
    hero: HeroId,
    /// The market's place: a Black Market's object, or a town's Artifact Merchants (E09 T58a1).
    place: MarketPlace,
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

/// What a market's window offers its visitor (ADV-MKT-008; E08 T133b, increment 14's design
/// note §12, §13): its kind's screens and count, and for each screen it shows the rates the
/// state's answer would use, so a client and the env need no copy of the computation (ADR-0004:
/// the answers are checked against the state again, never against this). Read at each view, never
/// kept: a viewer keeps nothing of a market it visited (§12).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MarketOffers {
    /// The kind's screens, in its order.
    pub screens: Vec<MarketScreen>,
    /// The Marketplaces it trades as (the rates' row).
    pub count: u8,
    /// With [`MarketScreen::Trade`]: each pair of two resources the parameters value, the given
    /// then the wanted in the core's order, and its lot (ADV-MKT-003; 42 with the base pack's
    /// seven). Empty without the screen.
    pub trades: Vec<TradeOffer>,
    /// With [`MarketScreen::Sell`]: each stack of the hero's army in its order, its lot per
    /// resource (ADV-MKT-005). Empty without the screen.
    pub sales: Vec<SaleOffer>,
    /// With [`MarketScreen::Buy`]: the Black Market's stock in its order, each entry's artifact
    /// and price per resource, none for a sold one (ADV-MKT-007). Empty without the screen.
    pub stock: Vec<StockEntry>,
}

/// A trade's rate in a market's window (ADV-MKT-003; E08 T133b): so many of `give` for so many of
/// `take` per lot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TradeOffer {
    /// The resource given.
    pub give: ResourceId,
    /// The resource wanted.
    pub take: ResourceId,
    /// The lot.
    pub lot: Lot,
}

/// A stack's sale in a market's window (ADV-MKT-005; E08 T133b): its slot and its lot (creatures
/// given, units gained) per resource it sells for, in the core's order; none for a creature of no
/// cost in the parameters' cost resource.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaleOffer {
    /// The hero's army's slot.
    pub slot: u8,
    /// The lot per resource.
    pub lots: Vec<(ResourceId, Lot)>,
}

/// A Black Market's entry in its window (ADV-MKT-007; E08 T133b): the artifact and its price per
/// resource in the core's order, or none and no price once bought (shown empty, design note §8).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StockEntry {
    /// The artifact, none once bought.
    pub artifact: Option<ArtifactTypeId>,
    /// Its price per resource (none for an artifact of no cost, or a price that rounds to 0).
    pub prices: Vec<(ResourceId, u32)>,
}

/// The amount of `resource` in `cost` (0 for none): a creature's or an artifact's value for the
/// market, its cost in the market parameters' cost resource (increment 14's design note §2: "a
/// creature of gold cost c", "an artifact of gold cost c"; ADV-Q60 (a)).
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

/// A market's window as its answers and its view read it (ADV-MKT-001, GAM-DEC-011; E09 T58a1,
/// the town epic's design note §104): the visit (its hero and its place), the screens it shows and
/// the Marketplaces it trades as. A map market's is the state's ([`AdventureState::market_window`]:
/// its kind's screens and count); a town's is the game's to resolve (the screens its buildings
/// and its hero allow, the trader's count: `h3_game`'s `town/market.rs`), as the town's hire
/// dialog's stock is. The stock is its place's: a Black Market's own with its sold entries, the
/// Artifact Merchants' at a town (TWN-MKT-004). Resolved at each answer and view, never kept.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MarketWindow<'p> {
    /// The visit: the window's hero and its place.
    pub visit: MarketVisit,
    /// The screens it shows, in their order.
    pub screens: &'p [MarketScreen],
    /// The Marketplaces it trades as (the rates' row): a map market's kind's, a town's trader's.
    pub count: u8,
}

impl MarketWindow<'_> {
    /// Whether the window shows `screen`.
    pub fn shows(&self, screen: MarketScreen) -> bool {
        self.screens.contains(&screen)
    }

    /// Checks that the window shows `screen` (E08 T133b; increment 14's design note §13): a
    /// transfer's first check (ADV-MKT-004), whose recipient and amount are the game's; refused
    /// with [`MarketError::Screen`].
    pub fn check_screen(&self, screen: MarketScreen) -> Result<(), MarketError> {
        if self.shows(screen) {
            Ok(())
        } else {
            Err(MarketError::Screen)
        }
    }
}

impl AdventureState {
    /// The window of `visit` to a market on the map (ADV-MKT-001; E08 T133a2, since E09 T58a1 a
    /// window of its own): its kind's screens and count, its hero on the map. Refused with
    /// [`MarketError::Visit`] for a hero that is not on the map, an object that is not a market,
    /// and a town's place (the game resolves a town's window).
    pub fn market_window<'p>(
        &self,
        part: &'p AdventureRuleset,
        visit: &MarketVisit,
    ) -> Result<MarketWindow<'p>, MarketError> {
        let MarketPlace::Object { index, .. } = visit.place else {
            return Err(MarketError::Visit);
        };
        let hero = visit.hero.and_then(|hero| self.hero(hero));
        let kind = self
            .object(index)
            .and_then(|object| object.object_type)
            .and_then(|ty| part.object_type(ty))
            .map(|ty| &ty.kind);
        let (Some(_), Some(ObjectKind::Market(market))) = (hero, kind) else {
            return Err(MarketError::Visit);
        };
        Ok(MarketWindow {
            visit: *visit,
            screens: &market.screens,
            count: market.count,
        })
    }

    /// The window's hero with its army: a map market's visitor on the map, a town's visitor or
    /// garrison hero (E09 T58a1); [`MarketError::Visit`] for a window without one (defensive: a
    /// town's window shows no screen that needs one without it) or a hero gone.
    fn window_hero(&self, window: &MarketWindow<'_>) -> Result<&HeroOnMap, MarketError> {
        window
            .visit
            .hero
            .and_then(|hero| {
                self.hero(hero)
                    .or_else(|| self.garrisoned(hero).map(|(_, on)| on))
            })
            .ok_or(MarketError::Visit)
    }

    /// The artifacts of `place`'s stock in its order, none for an entry bought (a Black Market's,
    /// its visits' block's sold bit) or empty (the Artifact Merchants', a purchase's or a draw
    /// that found none: TWN-MKT-004, `TWN-QUIRK-21`); none at all for a place without a stock.
    fn market_stock(
        &self,
        place: MarketPlace,
    ) -> impl Iterator<Item = Option<ArtifactTypeId>> + '_ {
        let (object, merchants) = match place {
            MarketPlace::Object { index, .. } => {
                let stock = self.object(index).map_or(&[][..], |o| o.resolved.stock());
                (Some((index, stock)), &[][..])
            }
            MarketPlace::Town(_) => (None, self.merchants_stock()),
        };
        let own = object.into_iter().flat_map(move |(index, stock)| {
            (0..=u8::MAX)
                .zip(stock)
                .map(move |(entry, &artifact)| (!self.is_sold(index, entry)).then_some(artifact))
        });
        own.chain(merchants.iter().copied())
    }

    /// The lot of a trade of `give` for `take` in `window` (ADV-MKT-003; E08 T133b, increment
    /// 14's design note §13; survey `E08-trades.md` §5, the original's `market_resources_proc`
    /// 0x5ECFE0): the market parameters' at the window's count (`MarketParams::resource_lot`,
    /// ADV-MKT-002; a town's the trader's, TWN-MKT-003). Refused, in this order: a window without
    /// trades ([`MarketError::Screen`]), a resource for itself ([`MarketError::SameResource`]), a
    /// resource the parameters do not value ([`MarketError::NoPrice`]). The lots the treasury can
    /// give are the game's.
    // rule: ADV-MKT-003, TWN-MKT-003
    pub fn trade_lot(
        &self,
        part: &AdventureRuleset,
        window: &MarketWindow<'_>,
        (give, take): (ResourceId, ResourceId),
    ) -> Result<Lot, MarketError> {
        window.check_screen(MarketScreen::Trade)?;
        if give == take {
            return Err(MarketError::SameResource);
        }
        part.params()
            .market()
            .and_then(|rates| rates.resource_lot(window.count, give, take))
            .ok_or(MarketError::NoPrice)
    }

    /// What `window` offers (ADV-MKT-008; E08 T133b, increment 14's design note §12, §13; E09
    /// T58a1 at a town, TWN-MKT-002): its screens and count, a trade's lot for each pair of
    /// distinct resources the parameters value (given, then wanted, in the core's order), each
    /// stack of the window's hero's army with its lot per resource it sells for, and each entry
    /// of the stock with its price per resource (none once bought or emptied), each only for a
    /// screen the window shows (the sales none without a hero).
    // rule: ADV-MKT-008, ADV-MKT-002
    pub fn market_offers(
        &self,
        (core, part): (&Ruleset, &AdventureRuleset),
        window: &MarketWindow<'_>,
    ) -> MarketOffers {
        let rates = part.params().market();
        let count = window.count;
        let resources = || (0..core.resources().len()).filter_map(ResourceId::from_index);
        let cost = |of: &[(ResourceId, u32)]| rates.map_or(0, |r| cost_in(of, r.cost()));
        let trades = resources()
            .filter(|_| window.shows(MarketScreen::Trade))
            .flat_map(|give| resources().map(move |take| (give, take)))
            .filter_map(|(give, take)| {
                let lot = rates.and_then(|r| r.resource_lot(count, give, take))?;
                Some(TradeOffer { give, take, lot })
            })
            .collect();
        let army = self
            .window_hero(window)
            .ok()
            .filter(|_| window.shows(MarketScreen::Sell))
            .map_or(&[][..], |hero| hero.army.as_slice());
        let sales = army
            .iter()
            .map(|stack| {
                let value = core
                    .creature(stack.creature)
                    .map_or(0, |creature| cost(&creature.cost));
                let lots = resources()
                    .filter_map(|take| {
                        let lot = rates.and_then(|r| r.creature_lot(count, value, take))?;
                        Some((take, lot))
                    })
                    .collect();
                SaleOffer {
                    slot: stack.slot,
                    lots,
                }
            })
            .collect();
        let stock = self
            .market_stock(window.visit.place)
            .filter(|_| window.shows(MarketScreen::Buy))
            .map(|artifact| {
                let value = artifact
                    .and_then(|a| core.artifacts().get(a.index()))
                    .map_or(0, |ty| cost(&ty.cost));
                let prices = resources()
                    .filter(|_| artifact.is_some())
                    .filter_map(|pay| {
                        let price = rates.and_then(|r| r.artifact_price(count, value, pay))?;
                        Some((pay, price))
                    })
                    .collect();
                StockEntry { artifact, prices }
            })
            .collect();
        MarketOffers {
            screens: window.screens.to_vec(),
            count,
            trades,
            sales,
            stock,
        }
    }

    /// Checks a sale of `lots` lots of the creatures in slot `slot` of `window`'s hero's army for
    /// `take` (ADV-MKT-005; E08 T133a2, increment 14's design note §11; survey `E08-trades.md`
    /// §6, the original's `market_sell_creatures_proc` 0x5EE890; E09 T58a1 at a town's
    /// Freelancer's Guild, TWN-MKT-003), each creature valued at its cost in the market
    /// parameters' cost resource (`MarketParams::cost`, the original's gold; E08 T133b, ADV-Q60
    /// (a)). The lot is the market parameters' at the window's count
    /// (`MarketParams::creature_lot`, ADV-MKT-002): `give` creatures for `take` units, one of
    /// them 1. Refused, in this order: a window without the sale ([`MarketError::Screen`]), its
    /// hero gone ([`MarketError::Visit`]), a slot without a stack ([`MarketError::Slot`]), no lot
    /// ([`MarketError::NoPrice`]: a creature of no cost, a resource the market does not value),
    /// lots of 0 or more creatures than the stack's ([`MarketError::Amount`]), the army's lone
    /// stack sold whole ([`MarketError::LastCreature`]: the army keeps a creature, so the most is
    /// the count less one when the army's other slots are empty).
    // rule: ADV-MKT-005, TWN-MKT-003
    pub fn check_sale(
        &self,
        (core, part): (&Ruleset, &AdventureRuleset),
        window: &MarketWindow<'_>,
        (slot, take, lots): (u8, ResourceId, u32),
    ) -> Result<Sale, MarketError> {
        window.check_screen(MarketScreen::Sell)?;
        let hero = self.window_hero(window)?;
        let stack = hero
            .army
            .iter()
            .find(|stack| stack.slot == slot)
            .ok_or(MarketError::Slot(slot))?;
        let lot = part
            .params()
            .market()
            .and_then(|rates| {
                let cost = core
                    .creature(stack.creature)
                    .map_or(0, |creature| cost_in(&creature.cost, rates.cost()));
                rates.creature_lot(window.count, cost, take)
            })
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
            hero: hero.id,
            slot,
            count: u32::try_from(count).unwrap_or(stack.count),
            gain: (take, u32::try_from(gain).unwrap_or(u32::MAX)),
        })
    }

    /// Makes `sale` (ADV-MKT-005; E08 T133a2), which [`Self::check_sale`] gave for this state as
    /// it stands, before the game adds its gain: the hero's stack loses the creatures sold, its
    /// slot emptied at 0 (since E09 T58a1 a town's garrison hero's too). Returns the slot that
    /// changed, which the game tells (`StackChanged`); no adventure event (increment 14's design
    /// note §12).
    // rule: ADV-MKT-005
    pub fn sell(&mut self, sale: Sale) -> ArmySlot {
        // The checked hero and slot: no branch for a hero or a slot the check did not find.
        let on_map = self.hero(sale.hero).is_some();
        let hero = if on_map {
            self.hero_mut(sale.hero)
        } else {
            self.garrisoned_mut(sale.hero)
        };
        hero.into_iter().for_each(|hero| {
            let sold = hero.army.iter_mut().filter(|stack| stack.slot == sale.slot);
            sold.for_each(|stack| stack.count = stack.count.saturating_sub(sale.count));
            hero.army.retain(|stack| stack.count > 0);
        });
        ArmySlot {
            army: ArmyRef::Hero(sale.hero),
            slot: sale.slot,
        }
    }

    /// Checks a purchase of the stock's entry `entry` paid in `pay` (ADV-MKT-007; E08 T133a2,
    /// increment 14's design note §11; survey `E08-trades.md` §7.2, the original's
    /// `market_buy_artifact_proc` 0x5ED9E0; E09 T58a1 the Artifact Merchants' stock at a town,
    /// TWN-MKT-004), the artifact valued at its cost in the market parameters' cost resource
    /// (`MarketParams::cost`, the original's gold; E08 T133b, ADV-Q60 (a)). The price is the
    /// market parameters' at the window's count (`MarketParams::artifact_price`, ADV-MKT-002; a
    /// town's the trader's, at least 1, so never free: `TWN-QUIRK-17`). Refused, in this order: a
    /// window without the purchase ([`MarketError::Screen`]), an entry past the stock, bought or
    /// empty ([`MarketError::Entry`]), no price ([`MarketError::NoPrice`]: an artifact of no cost,
    /// a price that rounds to 0, a resource the market does not value), a window without a hero
    /// ([`MarketError::Visit`], defensive), and a hero that cannot receive the artifact
    /// ([`MarketError::NoRoom`]: `HeroChange::AddArtifact`'s check, HER-CHG-002, a position that
    /// takes it else the backpack below its most; not the pickups' full-backpack count, A-25).
    /// The original pays and empties the entry whatever its `hero_give_artifact` did, so a hero
    /// without room lost the price and the artifact (A-46, `ADV-QUIRK-48`): fixed (E08 decision
    /// 366), refused before paying.
    // rule: ADV-MKT-007, TWN-MKT-004
    pub fn check_buy(
        &self,
        (core, part, heroes): Rules<'_>,
        window: &MarketWindow<'_>,
        (entry, pay): (u8, ResourceId),
    ) -> Result<MarketPurchase, MarketError> {
        window.check_screen(MarketScreen::Buy)?;
        let place = window.visit.place;
        let artifact = self
            .market_stock(place)
            .nth(usize::from(entry))
            .flatten()
            .ok_or(MarketError::Entry(entry))?;
        let price = part
            .params()
            .market()
            .and_then(|rates| {
                let cost = core
                    .artifacts()
                    .get(artifact.index())
                    .map_or(0, |ty| cost_in(&ty.cost, rates.cost()));
                rates.artifact_price(window.count, cost, pay)
            })
            .ok_or(MarketError::NoPrice)?;
        let hero = window.visit.hero.ok_or(MarketError::Visit)?;
        let change = HeroChange::AddArtifact(artifact.into());
        if !details_of(heroes, hero).is_some_and(|d| check_change(d, core, change).is_ok()) {
            return Err(MarketError::NoRoom);
        }
        Ok(MarketPurchase {
            hero,
            place,
            entry,
            artifact,
            price: (pay, price),
        })
    }

    /// Makes `purchase` (ADV-MKT-007; E08 T133a2), which [`Self::check_buy`] gave for this state
    /// as it stands, once the game paid its price and gave the artifact: a Black Market's entry is
    /// sold (the visits' block's bit, increment 14's design note §8), shown empty to the market's
    /// later visitors and never bought again; the Artifact Merchants' is emptied for every town
    /// and player until the next month's draw (TWN-MKT-004; E09 T58a1).
    // rule: ADV-MKT-007, TWN-MKT-004
    pub fn buy(&mut self, purchase: MarketPurchase) {
        match purchase.place {
            MarketPlace::Object { index, .. } => self.visits_mut().sell(index, purchase.entry),
            MarketPlace::Town(_) => self.empty_merchants_entry(purchase.entry),
        }
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
