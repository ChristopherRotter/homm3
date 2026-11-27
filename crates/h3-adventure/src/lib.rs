//! The adventure context (ADR-0016, E08): the adventure map, heroes on the map with their armies,
//! movement and pathing, map objects as closed behaviour kinds with data parameters, and fog of
//! war; its ruleset part and that part's compiler (ADR-0015 point 2). No domain crate depends on
//! another: players, turns and the calendar belong to `h3-game`, which drives this context.
//!
//! So far: the ruleset part ([`AdventureRuleset`], E08 T4), the setup ([`AdventureSetup`]) and
//! the state with its static map ([`AdventureState`], T5a), a day's movement points and the
//! start-of-turn step (T6b), the price of a step, `MoveHero` ([`AdventureCommand`]) and the
//! reachable-tiles query ([`Reach`], T7b), the persistent grid ([`BitGrid`], T5b), each player's
//! explored tiles with the reveals, movement on explored tiles only and the events' viewers
//! ([`Viewers`], T8a), the fringe under `ADV-QUIRK-1`, the observation per player
//! ([`AdventureObservation`]) and the events' projection ([`AdventureEvent::project`], T8b1),
//! object types in the ruleset part and the setup ([`ObjectType`], T12a), the objects' contents
//! drawn at the build, the objects that left the map and the tiles they freed, and the typed
//! objects a viewer sees ([`ObjectView`], T12b), visits from a neighbour and entered (T13b, T14a),
//! the Cartographer's offer ([`Offer`]) and its acceptance (T14b), the observatories, the
//! Keymaster's tents with the keys and the border guards with their offer (T15a), the wandering
//! monsters (T20), the guarded piles ([`Guards`], T21a); what a viewer newly sees since an
//! explored grid of its own ([`AdventureState::reveal`], SRV-Q26 (a)) and the heroes' movement
//! changes a command told ([`OwnHeroes`], SRV-Q26 (b); since E08 T115a its battle bonuses and
//! timed marks too).
//! Pointer identity (`ptr_eq`, `as_ptr`) is allowed only in the persistent module
//! `src/persistent.rs` (`arch.toml`, ADR-0016 point 2).
//!
//! Logic crate: pure, deterministic, `no_std` + `alloc` (ADR-0003, ADR-0005). The lint header
//! below is required and checked by `cargo xtask check-arch`.
#![no_std]
#![forbid(unsafe_code)]
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::todo,
    clippy::unimplemented,
    clippy::unreachable,
    clippy::float_arithmetic,
    clippy::print_stdout,
    clippy::print_stderr,
    clippy::exit
)]
#![warn(missing_docs)]

extern crate alloc;

mod answer;
mod army;
mod command;
mod fog;
mod market;
mod movement;
mod observe;
mod persistent;
mod reach;
mod ruleset;
mod setup;
mod split;
mod state;
mod town;

#[cfg(test)]
mod testing;

/// An army's value and a hero's power as the wandering monsters read them (ADV-MON-007), public
/// since E09 T39b1 for the Thieves' Guild (TWN-TAV-008, -009).
pub use answer::{army_value, power};
pub use army::{ArmyCommand, ArmyCommandError, ArmyRef, ArmySlot, split_range};
pub use command::{
    ALTAR_BACKPACK_PLACES, ALTAR_SLOTS, ALTAR_WORN_PLACES, AdventureCommand, AdventureCommandError,
    AdventureEvent, AdventureEventKind, AdventureOutcome, AltarError, AltarPlaces, AltarVisit,
    BattleStart, Choice, ChoiceKind, Combatant, Defender, Drowned, Exchange, Garrison, HeroGains,
    HillFortError, HillFortVisit, HireDialog, HireError, HireSource, HireTarget, MarketError,
    MarketPlace, MarketVisit, Meeting, NoRoom, NothingWhy, Offer, OfferKind, OwnHeroes, Pick,
    PrisonVisit, ServiceVisit, ShipyardBlock, Sink, UniversityVisit, Viewers,
};
pub use market::{Lot, MarketParams, MarketParamsError};
pub use movement::{day_points, diagonal_cost, land_table_points, native_terrain, sea_day_points};
pub use observe::{
    AdventureObservation, AdventureReveal, BoatView, GarrisonHeroView, GuardsView, HeroView, Known,
    KnownObject, MonsterView, ObjectView, OwnHeroView, RevealedTile, TileView,
};
pub use persistent::BitGrid;
pub use reach::Reach;
pub use ruleset::{
    AdventureCompileError, AdventurePack, AdventureParams, AdventureParamsDef, AdventureRuleset,
    AdventureSpell, AdventureSpellDef, Altar, AltarDef, AltarFactionDef, AltarScreens,
    BANK_ARTIFACT_CLASSES, BANK_GUARDS, BANK_LEVELS, BONUS_DAYS, Bank, BankDef, BankLevel,
    BankLevelDef, BankQuestion, BattleTerrains, BattleTerrainsDef, CalendarParams, Chest, ChestDef,
    DWELLING_LEVELS, DwellingParams, Find, FindContent, FindContentDef, FindDef, Gift, GiftDef,
    Gives, HillFort, KEY_COLOURS, MAX_BANK_REWARD, MAX_DIFFICULTIES, MAX_GAIN, MAX_LAND_POINTS,
    MAX_MONSTER_COUNT, MAX_PICKUP_AMOUNT, MAX_PLAYERS, MAX_STOCK, MarkLifetime, Market,
    MarketParamsDef, MarketScreen, MonolithWay, MonsterParams, MonsterParamsDef, MonthParams,
    MonthParamsDef, ObjectKind, ObjectKindDef, ObjectType, ObjectTypeDef, ObjectTypeId, OnceBonus,
    Passage, Pickup, PickupDef, Pyramid, PyramidDef, RandomDwellingDef, RiverDef, RiverId,
    RiverKind, RoadDef, RoadId, RoadKind, SpellParams, StartingResources, StartingResourcesDef,
    TerrainMovement, TerrainMovementDef, TerrainWalker, TerrainWalkerDef, Turn, TurnDef, Visit,
    WarMachineFactory, WarMachineFactoryDef, Weekday,
};
pub use setup::{
    AdventureSetup, ArmyError, Bans, BoatSetup, EventSetup, Guards, HeroSetup, Holds, MapSetup,
    MonsterSetup, ObjectSetup, Pos, PrisonerSetup, RewardRecord, Scenery, Seat, SetupError,
    StandError, TileMask, TileSetup,
};
pub use split::{TileRand, split, split_with_second};

/// The hero model's types the setup and the kinds name (E08 T61a): a gift's choice of primary
/// skills ([`Gives::Choice`]), an artifact on an object ([`Holds::Artifact`]), a reward record's
/// hero half ([`RewardRecord::hero`]) and a Scholar's gift ([`Holds::Gift`]), so the import needs
/// no edge to the hero model.
pub use h3_heroes::{ArtifactInstance, HeroReward, PrimarySkill, ScholarGift};
pub use state::{
    AdventureState, AltarArtifact, AltarOffers, AltarStack, ArtifactSale, ArtifactSaleOffer,
    BACKGROUND_RADIUS, Boat, Cast,
    CastError, CastFailure, Channels, DaySpells, GateEntry, HeroBonuses, HeroOnMap, HillFortRow,
    HillFortUpgrade, HirePurchase, MapObject, MarketOffers, MarketPurchase, MarketWindow, Monster,
    Resolved, Sacrifice, Sacrificed, Sale, SaleOffer, StaticMap, StaticTile, StockEntry,
    TradeOffer, VisitMarks, Visits, Weekly, WhirlpoolTile, allied, hire_price,
};
pub use town::{TavernVisit, TownHeroError, TownHeroMove, TownReveal};

/// Version of the adventure state's canonical hash encoding (`docs/08-work-process.md`,
/// "Versions"; owned by the adventure lane). Bump it with any change to what the hash covers.
/// v1 (E08 T5a): the first encoding ([`AdventureState`]'s `CanonicalHash`). v2 (E08 T6b): a
/// hero's movement points. v3 (E08 T8a): each player's explored tiles. v4 (E08 T12): each
/// object's type and contents. v5 (E08 T14b): the Cartographer types each player bought. v6 (E08
/// T15a): the keys each player holds. v7 (E08 T18b): a hero's formation. v8 (E08 T19a): whether a
/// tile of the static map is on the coast. v9 (E08 T20a2a): the tiles each object lies on, each
/// monster's creature, count, aggression, flags and treasure, the armies battles left on objects
/// and the game's difficulty. v10 (E08 T21a): each guarded pile's guards. v11 (E08 T50): each
/// hero's tactics flag. v12 (E08 T54): each object's scenery. v13 (E08 T61a): what each object
/// holds from the map (increment 7's kinds) and a monster's treasure artifact. v14 (E08 T61b): what
/// the build drew for each object, and the visits' block (none, or the heroes' marks, the
/// players' knowledge and the emptied objects). v15 (E08 T62b2): each player's seat (its colour
/// and whether it counts as human). v16 (E08 T72): the boats, each hero's boat and day's maximum.
/// v17 (E08 T83a): an abandoned mine's resources in the static map's objects ([`Holds`]' tag 7).
/// v18 (E08 T83b1): what the build drew for abandoned mines and Shipyards ([`Resolved`]' tags 7
/// and 8), the ownable objects' owners and the weekly objects' contents and marks. v19 (E08
/// T83b2a): the objects added in play (in the occupancy's encoding) and the monsters' fractions.
/// v20 (E08 T94b): the dwellings' counts and guards as they stand and the Refugee Camps'
/// creatures and counts. v21 (E09 T8a1): the towns among the ownable objects (their owners in the
/// owners' block), a town's garrison on its object, a town's formation in the static map's
/// objects ([`Holds`]' tag 9). v22 (E08 T113b): a map garrison's troops and removable flag in the
/// static map's objects ([`Holds`]' tag 11). v23 (E08 T115a): each hero on the map's battle
/// bonuses and timed marks ([`HeroBonuses`]). v24 (E08 T115b): the Fountains of Fortune's values,
/// the Magic Springs' emptied entrances and the kind marks.
/// v25 (E09 T28a, the town lane's, `cross-lane`): the towns' garrison heroes after the heroes on
/// the map ([`AdventureState::town_heroes`]). v26 (E08 T124b1): the creature banks' draws in the
/// static map, their guards as drawn and [`Resolved::Bank`] (tag 9). v27 (E08 T133a1): the Black
/// Markets' stocks in the static map ([`Resolved::Stock`], tag 10) and their sold entries in the
/// visits' block. v28 (E08 T141a, decision 413): each hero on the map's key in its owner's list
/// ([`HeroOnMap::listed`]); the Prisons' prisoners in the static map ([`Resolved::Prisoner`],
/// tag 11). v29 (E08 T142a, increment 15's design note §8): the heroes' day spells by hero id
/// ([`AdventureState::day_spells`]). v30 (E09 T57, the town lane's, `cross-lane`; the town
/// epic's design note §108): the Artifact Merchants' stock and the random artifacts' marks kept
/// from the build ([`AdventureState::merchants_stock`]), last.
pub const ADVENTURE_STATE_HASH_VERSION: u32 = 30;

/// Version of the adventure ruleset part's canonical hash encoding (ADR-0015 point 2; owned by the
/// adventure lane). v2 (E08 T5a): the setup bounds and the army slots. v3 (E08 T6b): the starting
/// stack chances. v4 (E08 T12a): the object types. v5 (E08 T13a): the starting resources by
/// difficulty. v6 (E08 T19a): the battle terrains by tile. v7 (E08 T20a2a): the monster and
/// ground kinds, the monsters' parameters and the creatures no random monster becomes. v8 (E08
/// T20b1): the resource a monster's price to join is paid in. v9 (E08 T29): the terrains' costs
/// by Pathfinding level. v10 (E08 T40): the fly and water-walk prices, the terrain walkers and a
/// ground's loss of the native price (Cursed Ground's `no_native`). v11 (E08 T50): a stack's most
/// creatures. v12 (E08 T61a): increment 7's kinds and their parameters. v13 (E08 T72): the sea's
/// kinds (the find's fixed entries, its lost turn and its leaving, a message, a ground of the sea,
/// a hole), the sea points, the sea ground's share and the battle terrains at sea. v14 (E08 T83a):
/// increment 9's seven kinds (mines, abandoned mines, Lighthouses, Shipyards, Windmills, Water
/// Wheels, Gardens), the monsters' growth, the calendar and the months. v15 (E08 T83b1): the flag
/// radius. v16 (E08 T93a): the dwellings' kinds, `[params.dwellings]`, `[[random_dwelling]]`.
/// v17 (E09 T8a1): the town's kind. v18 (E08 T103a): increment 11's three kinds (the monoliths
/// with their channel and way, the Subterranean Gate, the Whirlpool with its divisor). v19 (E09
/// T18a): the town's boat offsets. v20 (E08 T113a): increment 12's seven kinds (the garrison with
/// its anti-magic flag, the bonus with its mark, days, points and drain, the Fountain of Fortune
/// with its mark and values, the Magic Well, the Magic Spring, the Stables with its points and
/// creatures, the Sirens with its mark and keep) and every type's kind mark. v21 (E08 T123a):
/// the creature bank's kind (its question, levels, looted bonus, deck and coast). v22 (E09
/// T39a): the map's Tavern's and the Den of Thieves' kinds (tags 45 and 46, no parameter). v23
/// (E08 T132a2): increment 14's four kinds (the market, the Altar of Sacrifice, the Hill Fort, the
/// War Machine Factory: tags 47 to 50), the market's parameters and the Altar's factions. v24
/// (E08 T133b, ADV-Q60 (a)): the market parameters' cost resource. v25 (E08 T141a, increment
/// 15's design note §1, §2): the Prison's and the empty kind's tags 51 and 52, no parameter. v26
/// (E08 T142a, increment 15's design note §3): the adventure spells' bindings and
/// `[params.spells]`.
pub const ADVENTURE_RULESET_HASH_VERSION: u32 = 26;
