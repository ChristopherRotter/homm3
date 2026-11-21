//! The town's hire (TWN-TAV-003; E09 T38a1, design note `E09-design.md` §64, §75; survey
//! `E09-tavern.md` §3) and the replacement (TWN-TAV-004, §65): `TownCommand::Hire` checks the
//! actor's turn as every town command does (`GameState::check_ready`: its turn, no open decision,
//! GAM-DEC-001), then here the town, its owner, its Tavern, the slot and its hero, the adventure's
//! room and entrance and the treasury, each refusal changing nothing and drawing nothing; then
//! the price paid, the hero placed on the town's entrance as its visitor with its away record, the
//! slot refilled from the hirer's stream. The original's `town_tavern_hire` 0x5C12E0, read whole.
//! Since E09 T48a the hired hero, then the town's garrison hero, learn the town's guild spells
//! after the placement (TWN-GLD-005, [`super::guild::learn`]; `TWN-STUB-15` lifted). Since E09 T38a2
//! the legal commands list the hires that pass ([`legal_hires`], the same checks). Since E09 T39a
//! the map Tavern's hire (GAM-DEC-010, design note §66): the tavern decision's answer
//! ([`map_hire`], [`map_hires`]), checked as the town's without the town and its visitor, the hero
//! placed on the Tavern's visitable tile, refused while a hero stands there
//! ([`TavernError::Occupied`]), no guild spells.

use alloc::sync::Arc;
use core::fmt;

use h3_adventure::{HeroOnMap, Pos, TavernVisit, TownHeroError};
use h3_content::{HeroTypeId, ResourceId, SpellId};
use h3_contracts::{HeroId, PlayerId, TownId};
use h3_heroes::{ArtifactInstance, HeroDetails, OfferLevels};
use h3_towns::{TownCommand, TownCommandError};

use super::command::owner;
use super::rows::{most_heroes, town_error};
use super::tavern::{self, Parts, TAVERN_BACKPACK_SHOWN};
use super::{guild, leave};
use crate::command::{Answer, GameCommand, GameCommandError, GameEvent, Sink, TavernAnswer};
use crate::{GameRuleset, GameState};

/// Why a map Tavern's hire was refused (GAM-DEC-010; E09 T39a, the town epic's design note §66),
/// in the order [`map_hire`] checks: the slot, the room on the map, the Tavern's tile; the
/// treasury's is [`GameCommandError::CannotPay`]. A refused hire changes nothing and leaves the
/// decision open.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TavernError {
    /// The slot is not one of the player's two.
    UnknownSlot,
    /// The slot holds no hero.
    EmptySlot,
    /// The player has the most heroes on the map (the town part's most, its towns' garrison heroes
    /// not counted).
    TooManyHeroes,
    /// A hero stands on the Tavern's visitable tile (VCMI's refusal; the original never reaches
    /// it).
    Occupied,
}

impl fmt::Display for TavernError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::UnknownSlot => "the tavern has no such slot",
            Self::EmptySlot => "the tavern slot holds no hero",
            Self::TooManyHeroes => "the player has the most heroes on the map",
            Self::Occupied => "a hero stands on the tavern's tile",
        })
    }
}

impl core::error::Error for TavernError {}

/// A part of a hired hero's details that its tavern slot's view did not show (E09 T38a1, decision
/// 141; the view is the tavern's held hero screen's, decision 134): told to its owner after its
/// `HeroHired` ([`GameEvent::HiredPartSeen`]), one per part in this order, so that the slot's
/// view folded with them is the hero's details whole.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HiredPart {
    /// What its details hold that no list shows (first, once): the levels at which a level-up
    /// last offered Wisdom and a magic school (HER-LVL-004), and how many positions its worn
    /// artifacts have (the details hold one entry per position; the view names the worn ones).
    Details {
        /// The levels last offered.
        offered: OfferLevels,
        /// The positions' count.
        positions: u8,
    },
    /// An artifact of its backpack past the view's first [`TAVERN_BACKPACK_SHOWN`] places, in
    /// backpack order.
    Carried(ArtifactInstance),
    /// A spell it has learned, in its details' order.
    Spell(SpellId),
}

impl HiredPart {
    /// The parts of a hired hero with `details`, in their order.
    pub fn of(details: &HeroDetails) -> impl Iterator<Item = Self> + '_ {
        let fixed = Self::Details {
            offered: details.offered,
            positions: u8::try_from(details.worn.len()).unwrap_or(u8::MAX),
        };
        let carried = details.backpack.iter().skip(TAVERN_BACKPACK_SHOWN);
        core::iter::once(fixed)
            .chain(carried.map(|&artifact| Self::Carried(artifact)))
            .chain(details.spells.iter().map(|&spell| Self::Spell(spell)))
    }
}

/// Runs `player`'s hire of its tavern hero in `slot` at `town` (TWN-TAV-003), the player's turn
/// checked ([`GameState::check_ready`]), in this order, each refusal changing nothing:
///
/// 1. the town in the store ([`TownCommandError::UnknownTown`]), its owner `player`
///    ([`TownCommandError::NotOwner`]: an ally's or a visitor's town hires nothing);
/// 2. its standing Tavern and the part's price ([`h3_towns::check_hire`]:
///    [`TownCommandError::NoTavern`]);
/// 3. the slot one of the player's two ([`TownCommandError::UnknownSlot`]) holding a hero
///    ([`TownCommandError::EmptySlot`]);
/// 4. the adventure's room and entrance with the town part's most heroes
///    ([`h3_adventure::AdventureState::check_tavern_hire`]: [`TownCommandError::TooManyHeroes`],
///    [`TownCommandError::EntranceHeld`]);
/// 5. the treasury holds the price ([`GameCommandError::CannotPay`], GAM-RES-001).
///
/// Then, infallibly, in the original's order: the price paid (`ResourcePaid` per resource); the
/// hero's type taken by the player's colour, its slot emptied and its returning mark cleared, its
/// away record handed to the adventure, which places the hero on the town's entrance as its
/// visitor (no reveal, no visit); [`GameEvent::HeroHired`] to the owner and every player that
/// sees the tile, then to the owner its [`GameEvent::HiredPartSeen`]s; the slot refilled
/// ([`replace`], TWN-TAV-004); then the owner's changed slot told ([`tavern::tell`]: its
/// `TavernOffered` with its parts, or its emptying); then the town's heroes learn its guild's
/// spells (TWN-GLD-005, E09 T48a, [`guild::learn`]: the hired visitor first, then the garrison
/// hero, each `HeroChanged` to its owner; the original's 0x5BE030 after `hero_place`); then the
/// town's Mana Vortex to the hired hero alone (TWN-GLD-009, E09 T49a1, [`guild::vortex_to`]: the
/// original's `town_tavern_click` 0x5D846C), its cleared flag told. The town's views do not move
/// otherwise (the visitor is the adventure's hero view); the hero's movement points come by the
/// command's `MovementChanged`. No field of the command is trusted but the town's id and the
/// slot: the price is the part's (ADR-0004).
///
/// # Errors
///
/// [`GameCommandError::Town`] with the town's refusal, or [`GameCommandError::CannotPay`].
// rule: TWN-TAV-003, TWN-GLD-005, TWN-GLD-009
pub(crate) fn hire<S: Sink>(
    state: &mut GameState,
    rules: &GameRuleset,
    player: PlayerId,
    (town, slot): (TownId, u8),
    sink: &mut S,
) -> Result<(), GameCommandError> {
    let checked = check(state, rules, player, (town, slot))?;
    place(state, rules, (player, slot), checked, sink);
    // rule: TWN-GLD-005
    guild::learn(state, rules, town, sink);
    // rule: TWN-GLD-009
    let [hired, _] = guild::heroes(state, town);
    guild::vortex_to(state, rules, town, [hired, None], sink);
    Ok(())
}

/// The tavern decision's hire (GAM-DEC-010; E09 T39a, the town epic's design note §66; the
/// original's `map_tavern_hire` 0x4D7890): `player`'s tavern hero in `slot` hired at the map Tavern
/// of `visit`, checked in this order, each refusal changing nothing and leaving the decision open:
///
/// 1. the slot one of the player's two ([`TavernError::UnknownSlot`]) holding a hero
///    ([`TavernError::EmptySlot`]);
/// 2. fewer than the town part's most heroes of the player's on the map
///    ([`TavernError::TooManyHeroes`]), then no hero on the Tavern's visitable tile
///    ([`TavernError::Occupied`]; `h3_adventure::AdventureState::check_map_tavern_hire`);
/// 3. the treasury holds the price ([`GameCommandError::CannotPay`], GAM-RES-001).
///
/// Then as a town's hire ([`hire`]: the price, the pool, `HeroHired` with its parts, the
/// replacement, the slot told) but the hero placed on the Tavern's visitable tile and no guild
/// spells; the caller closes the decision (one hire a visit, the original's dialog). No town and
/// no visitor are tested (the original sets 0x6AAA48: no visitor test).
///
/// # Errors
///
/// [`GameCommandError::Tavern`] with the refusal, or [`GameCommandError::CannotPay`].
// rule: GAM-DEC-010, TWN-TAV-004
pub(crate) fn map_hire<S: Sink>(
    state: &mut GameState,
    rules: &GameRuleset,
    player: PlayerId,
    (visit, slot): (&TavernVisit, u8),
    sink: &mut S,
) -> Result<(), GameCommandError> {
    let checked = check_map(state, rules, player, (visit, slot))?;
    place(state, rules, (player, slot), checked, sink);
    Ok(())
}

/// A checked hire's application, for the town's ([`hire`]) and the map Tavern's ([`map_hire`]):
/// the price paid, the type taken by the player's colour from `slot`, the hero placed on the
/// checked tile with its away record, `HeroHired` and its parts, the replacement, the slot told.
fn place<S: Sink>(
    state: &mut GameState,
    rules: &GameRuleset,
    (player, slot): (PlayerId, u8),
    Checked {
        price,
        hero_type,
        hero,
        pos,
    }: Checked<'_>,
    sink: &mut S,
) {
    let before = S::RECORDS.then(|| tavern::all_views(state));
    state.pay_price(player, price, sink);
    let colour = state.colour(player).unwrap_or(u8::MAX);
    let pool = Arc::make_mut(state.taverns().pool);
    let away = pool.hire((player, usize::from(slot)), hero_type, colour);
    let placed = HeroOnMap {
        id: hero,
        owner: player,
        pos,
        army: away.army,
        formation: away.formation,
        tactics: away.tactics,
        movement: away.movement,
        boat: None,
        max_movement: away.max_movement,
    };
    let (adventure, _) = state.adventure_with_heroes();
    let seen_by = adventure.place_hired(rules.core(), placed);
    sink.emit(GameEvent::HeroHired {
        player,
        hero,
        hero_type,
        slot: Some(slot),
        pos,
        seen_by,
    });
    if S::RECORDS {
        let details = state.hero(hero);
        for part in details.into_iter().flat_map(HiredPart::of) {
            sink.emit(GameEvent::HiredPartSeen { player, hero, part });
        }
    }
    replace(state, rules, player, usize::from(slot));
    if let Some(before) = before {
        tavern::tell(state, &before, sink);
    }
}

/// What a hire that passes [`check`] takes: the price, the slot's hero's type and record, the
/// town's entrance.
struct Checked<'a> {
    price: &'a [(ResourceId, u32)],
    hero_type: HeroTypeId,
    hero: HeroId,
    pos: Pos,
}

/// [`hire`]'s checks in its order, changing nothing and drawing nothing.
fn check<'a>(
    state: &GameState,
    rules: &'a GameRuleset,
    player: PlayerId,
    (town, slot): (TownId, u8),
) -> Result<Checked<'a>, GameCommandError> {
    let refused = GameCommandError::Town;
    let record = state
        .towns()
        .get(town)
        .ok_or(refused(TownCommandError::UnknownTown))?;
    if owner(state, town) != Some(player) {
        return Err(refused(TownCommandError::NotOwner));
    }
    let price = h3_towns::check_hire(record, rules.towns()).map_err(refused)?;
    let (hero_type, hero) = slot_hero(state, player, slot).map_err(|empty| {
        refused(if empty {
            TownCommandError::EmptySlot
        } else {
            TownCommandError::UnknownSlot
        })
    })?;
    let pos = state
        .adventure()
        .check_tavern_hire(player, (town, most_heroes(rules)))
        .map_err(|error| refused(town_error(error)))?;
    if !state.can_pay(player, price) {
        return Err(GameCommandError::CannotPay);
    }
    Ok(Checked {
        price,
        hero_type,
        hero,
        pos,
    })
}

/// The type and record of the hero in `player`'s tavern `slot`: `Err(false)` for a slot that is
/// not one of the two, `Err(true)` for one that holds none.
fn slot_hero(state: &GameState, player: PlayerId, slot: u8) -> Result<(HeroTypeId, HeroId), bool> {
    let pool = state.pool();
    let slots = pool.tavern(player).map_or([None; 2], |tavern| tavern.slots);
    let held = slots.get(usize::from(slot)).ok_or(false)?;
    // A slot's type has its record (the fill's).
    held.and_then(|t| pool.record(t).map(|id| (t, id)))
        .ok_or(true)
}

/// [`map_hire`]'s checks in its order, changing nothing and drawing nothing.
fn check_map<'a>(
    state: &GameState,
    rules: &'a GameRuleset,
    player: PlayerId,
    (visit, slot): (&TavernVisit, u8),
) -> Result<Checked<'a>, GameCommandError> {
    let refused = GameCommandError::Tavern;
    let (hero_type, hero) = slot_hero(state, player, slot).map_err(|empty| {
        refused(if empty {
            TavernError::EmptySlot
        } else {
            TavernError::UnknownSlot
        })
    })?;
    let pos = visit.entrance;
    state
        .adventure()
        .check_map_tavern_hire(player, (pos, most_heroes(rules)))
        .map_err(|error| {
            refused(match error {
                TownHeroError::TooManyHeroes => TavernError::TooManyHeroes,
                _ => TavernError::Occupied,
            })
        })?;
    let price = hire_price(rules);
    if !state.can_pay(player, price) {
        return Err(GameCommandError::CannotPay);
    }
    Ok(Checked {
        price,
        hero_type,
        hero,
        pos,
    })
}

/// The part's price of a hire (TWN-TAV-001's data; none without the part's parameters).
fn hire_price(rules: &GameRuleset) -> &[(ResourceId, u32)] {
    rules
        .towns()
        .params()
        .map_or(&[][..], |params| params.hire_price.as_slice())
}

/// The tavern decision's hires `player` may give now (GAM-DEC-010; E09 T39a, design note §76),
/// for [`crate::legal_commands`]: with `slots` its player-wide part ([`hire_slots`]: the price,
/// the room, the slots holding a hero), a `Hire` of each slot it names, in slot order, when no
/// hero stands on the Tavern's visitable tile; so a listed hire passes every check of
/// [`map_hire`]. None with the tile held.
// rule: GAM-DEC-010
pub(crate) fn map_hires(
    state: &GameState,
    rules: &GameRuleset,
    (player, visit): (PlayerId, &TavernVisit),
) -> impl Iterator<Item = Answer> {
    let open =
        hire_slots(state, rules, player).filter(|_| state.adventure().tile_free(visit.entrance));
    let held = open.unwrap_or([false; 2]);
    (0..TAVERN_SLOTS)
        .filter(move |&slot| held[usize::from(slot)])
        .map(|slot| Answer::Tavern(TavernAnswer::Hire { slot }))
}

/// The slots of `player`'s tavern a listing may hire from anywhere now (E09 T38a2, design §64,
/// §76): those holding a hero, when the treasury holds the part's price and the map has room for
/// another of its heroes (the part's most, the garrison heroes not counted); none when either
/// fails or no slot holds a hero. Tested once a listing ([`legal_hires`] then tests each town).
pub(crate) fn hire_slots(
    state: &GameState,
    rules: &GameRuleset,
    player: PlayerId,
) -> Option<[bool; 2]> {
    let price = hire_price(rules);
    let room = state.adventure().has_room(player, most_heroes(rules));
    if !room || !state.can_pay(player, price) {
        return None;
    }
    let pool = state.pool();
    let slots = pool.tavern(player).map_or([None; 2], |tavern| tavern.slots);
    let held = slots.map(|slot| slot.is_some_and(|t| pool.record(t).is_some()));
    held.contains(&true).then_some(held)
}

/// The hires `player` may give at `town`, one of its own, now, for [`crate::legal_commands`] in
/// its turn without a decision (E09 T38a2, design §64, §76; TWN-TAV-003): with `slots` its
/// player-wide part ([`hire_slots`]), a `Hire` of each slot it names, in slot order, when no hero
/// stands on the town's entrance and the town has a standing Tavern; so a listed hire passes
/// every check of [`hire`] (the same conditions, the cheapest first). None for a town without a
/// Tavern or with its entrance held.
// rule: TWN-TAV-003
pub(crate) fn legal_hires<'a>(
    state: &'a GameState,
    rules: &'a GameRuleset,
    (town, slots): (TownId, Option<[bool; 2]>),
) -> impl Iterator<Item = GameCommand> + 'a {
    let open = slots.filter(|_| {
        state.adventure().entrance_free(town)
            && state
                .towns()
                .get(town)
                .is_some_and(|record| h3_towns::has_tavern(record, rules.towns()))
    });
    let held = open.unwrap_or([false; 2]);
    (0..TAVERN_SLOTS)
        .filter(move |&slot| held[usize::from(slot)])
        .map(move |slot| GameCommand::Town(TownCommand::Hire { town, slot }))
}

/// A player's tavern slots (TWN-SET-005).
const TAVERN_SLOTS: u8 = 2;

/// The replacement (TWN-TAV-004; `tavern_slot_replace` 0x4C86A0, survey `E09-tavern.md` §3; design
/// note §65): at once after a hire, `player`'s emptied `slot` (0 or 1) refilled from the hirer's
/// stream (ADR-0017 point 4): the draw ([`tavern::draw`]: the other slot's hero's class to avoid,
/// **any faction**, the player's faction's weights); a type found is offered and the slot holds
/// it, its record made when it has none (the new record's draws, [`tavern::record_of`]); its spell
/// points full; its army **one creature of its type's first starting stack** (GAM-SET-001's first
/// stack at a count of 1, no draw; none for a type without one; [`leave::one_creature`]) with no
/// war machine; **no backpack equipping and no points set**: its points are its away record's
/// until its owner's next turn start (TWN-TAV-005), since E09 T38b1 a new record's the land
/// maximum of that one creature (design note §67 point 5, the agent's call: the original sets
/// every record's at the new game with its full starting army, which only a replacement hired in
/// its first turn shows). No type free: the slot stays empty.
// rule: TWN-TAV-004, GAM-SET-001
pub(crate) fn replace(state: &mut GameState, rules: &GameRuleset, player: PlayerId, slot: usize) {
    let (core, towns) = (rules.core(), rules.towns());
    let colour = state.colour(player).unwrap_or(u8::MAX);
    let Parts {
        pool,
        heroes,
        progression,
        draws,
    } = state.taverns_for(player);
    let tavern = pool.tavern(player).copied().unwrap_or_default();
    let avoid = tavern.slots[1 - slot]
        .and_then(|other| core.hero_type(other))
        .map(|ty| ty.class);
    let picked = tavern::draw(
        pool,
        (core, towns),
        (colour, tavern.faction),
        (avoid, false),
        draws,
    );
    let Some((hero_type, ty)) = picked.and_then(|t| core.hero_type(t).map(|ty| (t, ty))) else {
        return;
    };
    let pool = Arc::make_mut(pool);
    pool.offer(hero_type);
    let held = pool.tavern_mut(player);
    held.into_iter()
        .for_each(|tavern| tavern.slots[slot] = Some(hero_type));
    let store = Arc::make_mut(heroes);
    let made = pool.record(hero_type).is_none();
    let id = tavern::record_of(pool, (store, progression), (core, hero_type, ty), draws);
    // A record's id indexes the store (the build's, or pushed above).
    let details = Arc::make_mut(&mut store[usize::from(id.raw())]);
    details.mana = h3_heroes::max_mana(details, core);
    let army = leave::one_creature(ty);
    let points =
        made.then(|| h3_adventure::day_points(core, rules.adventure(), Some(details), &army, 0));
    pool.set_army(hero_type, army, true);
    // rule: TWN-TAV-004
    let away = pool.away_mut(hero_type).zip(points);
    if let Some((away, points)) = away {
        away.movement = points;
        away.max_movement = points;
    }
}
