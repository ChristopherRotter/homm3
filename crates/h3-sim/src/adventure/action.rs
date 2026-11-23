//! The adventure's action encoding (ADR-0017 point 8, E08 T9a), factorized as the battle's casts
//! are (ADR-0014 point 9), in two phases (encoding v3, E08 T19b1): a player in a battle acts in
//! the **battle phase**, everyone else in the **map phase** ([`Phase`], [`phase`]).
//!
//! In the map phase an action is an *entry*, and for a move a destination tile in the *tile
//! head* ([`MapAction`]). Entries (independent of the map and the ruleset, so agents see a fixed
//! number):
//!
//! | entry | action |
//! |---|---|
//! | 0 | End the turn |
//! | 1 + s, s < 8 | Move the player's hero in slot `s` to the tile in the tile head |
//! | 9 | Accept the player's open offer (E08 T14b: a Cartographer's offer) |
//! | 10 | Decline it, or close or leave the player's open decision (E08 T63a) |
//! | 11 | The open decision's first answer (E08 T30: a level-up's left offer; T63a) |
//! | 12 | Its second answer (E08 T30: a level-up's right offer; T63a) |
//! | 13 + s, s < 4 | Buy the skill in slot `s` of the player's open University (E08 T63a) |
//! | 17 | Build a boat at the player's or an ally's Shipyard anchored on the tile in the tile head (E08 T85b2b) |
//! | 18 + c, c < 4 | Buy the most of the creature in place `c` of the player's open hire dialog (E08 T95b) |
//! | 22 + s, s < 8 | The hero in slot `s` revisits the entered object it stands on (E08 T105b2) |
//! | 30 | Build the building in the building head in the town whose entrance is the tile in the tile head (E09 T19c1) |
//! | 31 | Open the hire dialog of the building in the building head in the town whose entrance is the tile in the tile head (E09 T27) |
//! | 32 + m, m < 3 | Move a hero between the two bars of the town whose entrance is the tile in the tile head: `m` 0 the visitor into the garrison, 1 the garrison hero out, 2 the two swapped (E09 T28a) |
//! | 35 + s, s < 2 | Hire the hero in slot `s` of the player's tavern at the town whose entrance is the tile in the tile head (E09 T38a2) |
//! | 37 + p, p < 42 | Trade one lot of the pair `p` at the player's open market: given `p / 6`, wanted the `p % 6`-th of the others, in the core's order (E08 T135a) |
//! | 79 + s, s < 7 | Sell one lot of the stack in army slot `s` for the market parameters' cost resource (the original's gold) at the player's open market (E08 T135a) |
//! | 86 + e, e < 7 | Buy the entry `e` of the player's open Black Market, paid in the cost resource (E08 T135a) |
//! | 93 | Sacrifice the artifacts' all at the player's open Altar (E08 T135a) |
//! | 94 | Sacrifice the creatures' all at the player's open Altar (E08 T135a) |
//! | 95 + s, s < 7 | Upgrade the stack in army slot `s` at the player's open Hill Fort (E08 T135a) |
//! | 102 | Upgrade every stack that upgrades at the player's open Hill Fort (E08 T135a) |
//! | 103 | Open the town screen of the town whose entrance is the tile in the tile head: its Mana Vortex, its guild's spells (E09 T49a2) |
//!
//! A player's hero slots are its heroes on the map in the order of their ids, the first
//! [`HERO_SLOTS`] (the original allows a player eight heroes on the map); a hero's slot changes
//! when a hero of a lower id joins or leaves the player (a battle's loser leaves the map). The
//! tile head is a tile of the map, `(level × height + row) × width + column`, the observation's
//! order; its size is the map's tile count ([`crate::adventure::AdventureEnv::tile_count`]). After
//! an entry that takes no tile (`EndTurn`, the answers, the hire dialog's purchases, a revisit) the
//! tile head allows exactly 0, as the
//! battle's heads after a non-cast entry (a policy that always samples the head never meets an
//! empty mask on a legal entry). The *building head* (E09 T19c1) is an index of a town's faction's
//! record of buildings, [`BUILDING_COUNT`] (`h3_towns::MAX_BUILDINGS`, 64) wide: after [`BUILD`]
//! and a town's entrance it allows the buildings of the builds [`legal_commands`] lists for that
//! town ([`building_mask`]: the hall's offer the owner's treasury pays, one build a town a day,
//! TWN-BLD-001 to -004), after [`RECRUIT`] those of its recruitments (E09 T27: a standing
//! dwelling, horde, or Portal holding a creature, TWN-REC-001, -002), after another entry and a
//! tile its masks allow exactly 0, and a building other than 0 decodes to no command with any
//! other entry. After [`BUILD_BOAT`] it allows the anchors of the Shipyards
//! whose `BuildBoat` [`legal_commands`] lists (ADV-SHY-003: the player's or an ally's, its boat
//! tile free, the price in the treasury; the Shipyard as the observation names it, by its
//! anchor; the setup refuses two typed objects of one type on one anchor, and the original's
//! packs have one Shipyard type). After [`BUILD`] it allows the entrances of the towns of the
//! listed builds (E09 T19c1: the player's own towns, the entrance its hero enters by, TWN-VIS-001;
//! one town an entrance), after [`RECRUIT`] those of the listed recruitments (E09 T27), after a
//! [`TOWN_HEROES`] entry those of the listed hero moves of its kind (E09 T28a), after a
//! [`TAVERN_HIRE`] entry those of the listed hires of its slot (E09 T38a2), after [`TOWN_OPEN`]
//! those of the listed openings (E09 T49a2). A move's destination is a tile the hero's reachable-tiles query
//! lists, not its own; a tile another player's hero stands on is one (the visit is a battle,
//! GAM-BAT-004). The action carries no path: the command takes the path the query chose
//! (`h3_game::legal_moves`). A client may send any other path the hero can pay (`apply` checks
//! each step); an agent of this encoding moves along the query's paths only. The heads' masks are
//! conditioned on the entry ([`legal_mask`], [`tile_mask`]); an illegal entry's tile mask is
//! empty. While the player has an open decision its answers are its only legal entries
//! (GAM-DEC-001: `h3_game::legal_commands` lists them alone and `h3_game::legal_moves` gives
//! nothing), each with tile 0 alone. The game's answers are typed by the decision's kind
//! (`h3_game::Answer`, E08 T63a); an entry names the open decision's answer of its role
//! ([`answer_entry`]): an offer's `Accept` and `Decline`, a level-up's two offers (GAM-DEC-002), a
//! meeting's or an army dialog's `Close` (GAM-DEC-003, GAM-DEC-004), a chest's, the Arena's and a
//! School's two answers and a School's leaving (GAM-DEC-005), a University's purchases of the
//! first [`UNIVERSITY_SLOTS`] slots and its leaving (GAM-DEC-006), a hire dialog's purchase of the
//! most of each of the first [`HIRE_SLOTS`] creatures in its source's order (the `Buy`
//! `legal_commands` lists for it, so an entry is legal when that creature's most is 1 or more) and
//! its `Close` (GAM-DEC-008), a map Tavern's hires of the two slots and its `Close` (GAM-DEC-010,
//! version 13), a market's `Leave` (GAM-DEC-011, version 14), an Altar's `Leave` (GAM-DEC-012,
//! version 15), a Hill Fort's `Leave` (GAM-DEC-013, version 16); since version 17 (E08 T135a) a
//! market's trades of one lot by their pairs, its sales of one lot for the cost resource by their
//! slots and its purchases in it by their entries, an Altar's two alls, a Hill Fort's upgrades by
//! their slots and its `All`, each the answer `legal_commands` lists for it. With no open decision
//! of that role the entry decodes to the offer's or the level-up's answer, or the purchase (a hire
//! entry's: a `Buy` of 0, [`hire_answer`]), or the trade's, sale's, purchase's or upgrade's
//! answer its entry names, or an Altar's empty sacrifice ([`answer_of`]), which `apply` rejects.
//!
//! In the battle phase an action is one of the battle lane's encoding (`crate::action`, its own
//! version: entries and the spell and target heads), for the side the player owns; its masks
//! ([`battle_mask`], [`spell_mask`], [`target_mask`], [`second_mask`]) are that encoding's for
//! the player's side, empty while the other side is to act, and every map entry is masked (ADR-0017
//! point 8). Each phase's masks are empty in the other phase.
//!
//! Who acts is a [`Seat`]: a player, or the **neutral seat** (E08 T20b3a), which plays a battle's
//! unowned side (a wandering monster's) as the system actor (`h3_game::Actor::System`). It acts in
//! the battle phase only, with the same encoding for the side to act of the first battle whose
//! side to act no player owns (`h3_game::GameState::unowned_battle`); its map masks are empty. The
//! seat is not part of an action: the layout above is unchanged (still version 3).
//!
//! Version 2 (E08 T14b) appended the answers. Version 3 (E08 T19b1) added the battle phase: since
//! E08 T19a a move onto another player's hero starts a battle, which version 2 could not play, so
//! it left such tiles out of the masks and the decoder (`ADV-STUB-4`, closed). Version 4 (E08
//! T30) appended a level-up's two answers. Version 5 (E08 T63a, the adventure lane's `cross-lane`
//! change with the user's consent) moved the answers onto the typed ones by role and appended a
//! University's purchases. Version 6 (E08 T85b2b, the adventure lane's `cross-lane` change with
//! the user's consent, decision 203; ADR-0017 point 8) appended `BuildBoat` (ADV-SHY-003), whose
//! tile head names the Shipyard by its anchor's tile: one action per Shipyard the player may use.
//! Version 7 (E08 T95b, the adventure lane's `cross-lane` change with the user's consent, decision
//! 243) appended the hire dialog's four purchases: the most only; an amount head is the AI lane's.
//! Version 8 (E08 T105b2, the adventure lane's `cross-lane` change with the user's consent,
//! decision 268; increment 11's design note §14) appended a `Revisit` per hero slot (ADV-TEL-008:
//! the entered object under the hero visited again, legal where [`legal_commands`] lists it, at 0
//! points too, so a turn may revisit without end: an env's rollout needs a step cap, the AI
//! lane's call, design note §18). A jump needs no entry: a teleporter is a destination among the
//! reachable tiles (ADR-0017 point 8). Version 9 (E09 T19c1, the town lane's `cross-lane` change
//! with the user's consent, E09 decisions 7, 58, 62; the town epic's design note §32; ADR-0017
//! point 8) appended [`BUILD`] and the building head ([`MapAction::building`]): a town's build
//! (`TownCommand::Build`, TWN-BLD-001) by its town's entrance and its building's index in the
//! town's record, one action per listed build; no hero, no visit, as the command. Version 10 (E09
//! T27, the town lane's `cross-lane` change with the user's consent, E09 decision 92; the town
//! epic's design note §55) appended [`RECRUIT`]: a town's recruitment (`TownCommand::Recruit`,
//! TWN-REC-001, -002) by its town's entrance and its building's index, with the two heads of
//! [`BUILD`]; the dialog it opens is the hire dialog's, whose purchases are its four entries.
//! Version 11 (E09 T28a, the town lane's `cross-lane` change with the user's consent, E09 decision
//! 93; the town epic's design note §55) appended the three [`TOWN_HEROES`] entries: a hero's move
//! between a town's two bars (`TownCommand::Heroes`, TWN-ROW-002 to -004) by its town's entrance
//! in the tile head, the building head 0. A garrison hero is off the map: it holds no hero slot.
//! The army commands between the bars have no entry, as every management command (decision 99).
//! Version 12 (E09 T38a2, the town lane's `cross-lane` change with the user's consent, E09
//! decision 143; the town epic's design note §76; ADR-0017 point 8) appended the two
//! [`TAVERN_HIRE`] entries (the design note's `HIRE + s`): a town's hire of the hero in the
//! player's tavern slot `s` (`TownCommand::Hire`, TWN-TAV-003) by its town's entrance in the tile
//! head, the building head 0; one action per listed `Hire`. A hero hired takes the player's next
//! hero slot by its id. Version 13 (E09 T39a, the town lane's `cross-lane` change with the user's
//! consent, E09 decision 149; the town epic's design note §76) maps the map Tavern's decision
//! (GAM-DEC-010) onto the answers by role, no new entry: the hire of slot 0 its first, of slot 1
//! its second, its `Close` the closing role. Version 14 (E08 T133b, the adventure lane's
//! `cross-lane` change with the user's consent; increment 14's design note §18) maps a market's
//! decision (GAM-DEC-011) onto the closing role, no new entry: its `Leave`; its trades, sales and
//! purchases have none until E08 T135a appends them. Version 15 (E08 T134a, the same) maps an
//! Altar of Sacrifice's decision (GAM-DEC-012) onto the closing role, no new entry: its `Leave`;
//! its sacrifices have none until T135a appends its two alls. Version 16 (E08 T134b1, the same)
//! maps a Hill Fort's decision (GAM-DEC-013) onto the closing role, no new entry: its `Leave`;
//! its upgrades have none until T135a appends them. Version 17 (E08 T135a, the same; decision 372,
//! increment 14's design note §18) appends 66 entries for the answers `legal_commands` lists for
//! the trades and services, each with tile 0 alone, legal where it lists the answer: a market's
//! [`MARKET_TRADE`] (one lot of each pair of the first [`MARKET_RESOURCES`] resources, the given
//! then the wanted in the core's order), [`MARKET_SELL`] (one lot of a slot's stack for the market
//! parameters' cost resource) and [`MARKET_BUY`] (an entry of the stock, paid in it); an Altar's
//! [`ALTAR_ARTIFACTS`] and [`ALTAR_CREATURES`] (its two alls, `h3_game`'s listing: every artifact
//! it takes up to its places, every creature but one of the lowest occupied slot); a Hill Fort's
//! [`HILL_FORT_UPGRADE`] (a slot's stack) and [`HILL_FORT_ALL`]. A market's transfer, another
//! lot, resource or set has no entry (an amount head stays the AI lane's, as the hire dialog's);
//! the closing role stays each kind's `Leave`. The factory needs none: its offer's and its hire
//! dialog's entries. Version 18 (E09 T49a2, the town lane's `cross-lane` change with the user's
//! consent, E09 decision 193; the town epic's design note §95) appends [`TOWN_OPEN`]: a town
//! screen's opening (`TownCommand::Open`, TWN-GLD-006) by its town's entrance in the tile head, the
//! building head 0; one action per listed `Open`, which `legal_commands` lists only where it
//! would change something (the Mana Vortex taken or a spell learned). Appended after the trades'
//! and services' entries rather than beside the taverns' (the design note's place, written
//! before version 17), so that no entry moves. Version 19 (E09 T49b, the same, E09 decision 194)
//! appends [`SPELL_BOOK`]: a spell book's purchase at a town's Mage Guild
//! (`TownCommand::BuySpellBook`, TWN-GLD-007) by its town's entrance in the tile head, the
//! building head 0; one action per listed purchase, which `legal_commands` lists where it would
//! be accepted (a standing guild, a hero there without a book, the price in the treasury).

use h3_adventure::{ALTAR_SLOTS, AdventureCommand, AltarPlaces, MarketParams, Pos, Reach};
use h3_battle::{BattleState, LegalCommands};
use h3_content::{CreatureTypeId, ResourceId};
use h3_contracts::TownId;
use h3_contracts::{BattleSide, HeroId, PlayerId};
use h3_game::{
    Actor, AltarAnswer, Answer, ArenaChoice, ChestChoice, Consent, GameCommand, GameRuleset,
    GameState, HillFortAnswer, HireAnswer, MarketAnswer, SchoolChoice, SkillOffer, TavernAnswer,
    UniversityAnswer, legal_commands, legal_moves,
};
use h3_towns::{BuildingId, MAX_BUILDINGS, TownCommand, TownHeroChange};

use crate::action as battle;

/// Version of this encoding; bump on any change of the layout above.
pub const ACTION_ENCODING_VERSION: u32 = 19;

/// Hero slots per player: the most heroes the original allows a player on the map.
pub const HERO_SLOTS: usize = 8;

/// The entry that ends the turn.
pub const END_TURN: usize = 0;

/// The entry of the hero in slot 0's move; slot `s` is `MOVE_BASE + s`.
pub const MOVE_BASE: usize = 1;

/// The entry that accepts the player's open offer.
pub const ACCEPT: usize = MOVE_BASE + HERO_SLOTS;

/// The entry that declines the player's open offer, closes its meeting, army dialog (E08 T63a) or
/// hire dialog (E08 T95b), or leaves its School or University (E08 T63a).
pub const DECLINE: usize = ACCEPT + 1;

/// The entry of the first answer of the player's open two-way decision: a level-up's first offer
/// (E08 T30), a chest's gold, the Arena's first skill, a School's first skill (E08 T63a).
pub const FIRST: usize = DECLINE + 1;

/// The entry of the second answer of the player's open two-way decision: a level-up's second
/// offer (E08 T30), a chest's experience, the Arena's second skill, a School's second (E08 T63a).
pub const SECOND: usize = FIRST + 1;

/// The University's slots the entries buy (E08 T63a): the original's dialog's four (a pack's
/// University with more offers sells the rest to no agent of this encoding).
pub const UNIVERSITY_SLOTS: usize = 4;

/// The entry that buys the skill in slot 0 of the player's open University; slot `s` is
/// `BUY_BASE + s` (E08 T63a).
pub const BUY_BASE: usize = SECOND + 1;

/// The entry that builds a boat at the player's or an ally's Shipyard anchored on the tile in the
/// tile head (E08 T85b2b; ADV-SHY-003).
pub const BUILD_BOAT: usize = BUY_BASE + UNIVERSITY_SLOTS;

/// The hire dialog's creatures the entries buy (E08 T95b): the original's dialog's four (a pack's
/// dwelling with more creatures sells the rest to no agent of this encoding).
pub const HIRE_SLOTS: usize = 4;

/// The entry that buys the most of the creature in place 0 of the player's open hire dialog, in
/// its source's order; place `c` is `HIRE_BASE + c` (E08 T95b).
pub const HIRE_BASE: usize = BUILD_BOAT + 1;

/// The entry of the hero in slot 0's revisit of the entered object it stands on; slot `s` is
/// `REVISIT_BASE + s` (E08 T105b2; ADV-TEL-008).
pub const REVISIT_BASE: usize = HIRE_BASE + HIRE_SLOTS;

/// The entry that builds the building in the building head in the town whose entrance is the
/// tile in the tile head (E09 T19c1; TWN-BLD-001).
pub const BUILD: usize = REVISIT_BASE + HERO_SLOTS;

/// The entry that opens the hire dialog of the building in the building head in the town whose
/// entrance is the tile in the tile head (E09 T27; TWN-REC-001, -002).
pub const RECRUIT: usize = BUILD + 1;

/// The entry of the first hero move between the two bars of the town whose entrance is the tile
/// in the tile head (E09 T28a; TWN-ROW-002 to -004): `TOWN_HEROES` the visitor into the garrison,
/// `+ 1` the garrison hero out, `+ 2` the two swapped ([`TOWN_HERO_MOVES`]).
pub const TOWN_HEROES: usize = RECRUIT + 1;

/// The hero moves between a town's bars, in their entries' order from [`TOWN_HEROES`] (E09 T28a).
pub const TOWN_HERO_MOVES: [TownHeroChange; 3] = [
    TownHeroChange::Enter,
    TownHeroChange::Leave,
    TownHeroChange::Swap,
];

/// A player's tavern slots the entries hire from (E09 T38a2; TWN-SET-005: the original's two).
pub const TAVERN_SLOTS: usize = 2;

/// The entry that hires the hero in slot 0 of the player's tavern at the town whose entrance is
/// the tile in the tile head; slot `s` is `TAVERN_HIRE + s` (E09 T38a2; TWN-TAV-003; the design
/// note's `HIRE + s`).
pub const TAVERN_HIRE: usize = TOWN_HEROES + TOWN_HERO_MOVES.len();

/// The resources whose pairs [`MARKET_TRADE`]'s entries trade (E08 T135a): the original's seven,
/// the first in the core's order (a pack's further resources trade with no agent of this
/// encoding).
pub const MARKET_RESOURCES: usize = 7;

/// The pairs of two of the first [`MARKET_RESOURCES`] resources (E08 T135a): 42.
pub const MARKET_PAIRS: usize = MARKET_RESOURCES * (MARKET_RESOURCES - 1);

/// The army slots the entries of a sale and of an upgrade name (E08 T135a): the original's seven.
pub const ARMY_SLOTS: usize = 7;

/// The Black Market's entries the purchases name (E08 T135a): the original's seven.
pub const MARKET_STOCK: usize = 7;

/// The entry that trades one lot of the pair 0 at the player's open market; pair `p` is
/// `MARKET_TRADE + p`, given the `p / 6`-th resource for the `p % 6`-th of the others, in the
/// core's order ([`market_pair`]; E08 T135a, ADV-MKT-003).
pub const MARKET_TRADE: usize = TAVERN_HIRE + TAVERN_SLOTS;

/// The entry that sells one lot of the stack in army slot 0 for the market parameters' cost
/// resource at the player's open market; slot `s` is `MARKET_SELL + s` (E08 T135a, ADV-MKT-005).
pub const MARKET_SELL: usize = MARKET_TRADE + MARKET_PAIRS;

/// The entry that buys the entry 0 of the player's open Black Market, paid in the market
/// parameters' cost resource; entry `e` is `MARKET_BUY + e` (E08 T135a, ADV-MKT-007).
pub const MARKET_BUY: usize = MARKET_SELL + ARMY_SLOTS;

/// The entry that sacrifices the artifacts' all at the player's open Altar (E08 T135a,
/// ADV-ALT-002): the sacrifice `legal_commands` lists.
pub const ALTAR_ARTIFACTS: usize = MARKET_BUY + MARKET_STOCK;

/// The entry that sacrifices the creatures' all at the player's open Altar (E08 T135a,
/// ADV-ALT-003): the sacrifice `legal_commands` lists.
pub const ALTAR_CREATURES: usize = ALTAR_ARTIFACTS + 1;

/// The entry that upgrades the stack in army slot 0 at the player's open Hill Fort; slot `s` is
/// `HILL_FORT_UPGRADE + s` (E08 T135a, ADV-HFT-002).
pub const HILL_FORT_UPGRADE: usize = ALTAR_CREATURES + 1;

/// The entry that upgrades every stack that upgrades at the player's open Hill Fort (E08 T135a,
/// ADV-HFT-002).
pub const HILL_FORT_ALL: usize = HILL_FORT_UPGRADE + ARMY_SLOTS;

/// The entry that opens the town screen of the town whose entrance is the tile in the tile head
/// (E09 T49a2; TWN-GLD-006): its Mana Vortex to its garrison hero, then its visitor, then both
/// learning its guild's spells.
pub const TOWN_OPEN: usize = HILL_FORT_ALL + 1;

/// The entry that buys a spell book at the Mage Guild of the town whose entrance is the tile in
/// the tile head, for its visitor, else its garrison hero (E09 T49b; TWN-GLD-007).
pub const SPELL_BOOK: usize = TOWN_OPEN + 1;

/// Number of entries.
pub const ENTRY_COUNT: usize = SPELL_BOOK + 1;

/// The building head's size (E09 T19c1): the most buildings of a town's record
/// (`h3_towns::MAX_BUILDINGS`), each named by its index there.
pub const BUILDING_COUNT: usize = MAX_BUILDINGS;

/// Which entries are legal.
pub type EntryMask = [bool; ENTRY_COUNT];

/// Which buildings of the building head are legal (E09 T19c1).
pub type BuildingMask = [bool; BUILDING_COUNT];

/// Who acts (ADR-0017 point 8): a player, or the neutral seat, which plays a battle's unowned side
/// (a wandering monster's) as the system actor (E08 T20b3a).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Seat {
    /// A player: its turn's actions on the map, or its side's in its battle.
    Player(PlayerId),
    /// The neutral seat: the unowned side's actions in the first battle whose side to act no
    /// player owns ([`GameState::unowned_battle`]).
    Neutral,
}

impl Seat {
    /// The actor the seat's commands are issued as: the player, or the system actor.
    pub const fn actor(self) -> Actor {
        match self {
            Self::Player(player) => Actor::Player(player),
            Self::Neutral => Actor::System,
        }
    }

    /// The player of the seat; none for the neutral seat.
    pub const fn player(self) -> Option<PlayerId> {
        match self {
            Self::Player(player) => Some(player),
            Self::Neutral => None,
        }
    }
}

impl From<PlayerId> for Seat {
    fn from(player: PlayerId) -> Self {
        Self::Player(player)
    }
}

/// The phase a seat acts in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Phase {
    /// On the map: [`MapAction`]s.
    Adventure,
    /// In its battle: the battle's actions for the side it owns.
    Battle,
}

/// An action of the map phase: an entry, its tile head (0 for an entry that takes no tile) and
/// its building head (0 but for [`BUILD`], E09 T19c1, and [`RECRUIT`], E09 T27).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct MapAction {
    /// The entry.
    pub entry: usize,
    /// The destination tile of a move, `(level × height + row) × width + column`, a Shipyard's
    /// anchor or a town's entrance; 0 otherwise.
    pub tile: u32,
    /// The building of a build or a recruitment: its index in the town's faction's record; 0
    /// otherwise.
    pub building: u8,
}

/// An action of either phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
    /// An action of the map phase.
    Adventure(MapAction),
    /// An action of the battle phase: the battle lane's encoding (`crate::action`).
    Battle(battle::Action),
}

impl Action {
    /// Ends the turn.
    pub const fn end_turn() -> Self {
        Self::map(END_TURN, 0)
    }

    /// Moves the hero in `slot` to `tile`.
    pub const fn move_to(slot: usize, tile: u32) -> Self {
        Self::map(MOVE_BASE + slot, tile)
    }

    /// Accepts the open decision.
    pub const fn accept() -> Self {
        Self::map(ACCEPT, 0)
    }

    /// Declines the open decision.
    pub const fn decline() -> Self {
        Self::map(DECLINE, 0)
    }

    /// Takes the open two-way decision's first answer (E08 T30, T63a).
    pub const fn first() -> Self {
        Self::map(FIRST, 0)
    }

    /// Takes the open two-way decision's second answer (E08 T30, T63a).
    pub const fn second() -> Self {
        Self::map(SECOND, 0)
    }

    /// Buys the skill in `slot` of the open University (E08 T63a).
    pub const fn buy(slot: usize) -> Self {
        Self::map(BUY_BASE + slot, 0)
    }

    /// Builds a boat at the Shipyard anchored on `tile` (E08 T85b2b).
    pub const fn build_boat(tile: u32) -> Self {
        Self::map(BUILD_BOAT, tile)
    }

    /// Buys the most of the creature in `place` of the open hire dialog (E08 T95b).
    pub const fn hire(place: usize) -> Self {
        Self::map(HIRE_BASE + place, 0)
    }

    /// The hero in `slot` revisits the entered object it stands on (E08 T105b2).
    pub const fn revisit(slot: usize) -> Self {
        Self::map(REVISIT_BASE + slot, 0)
    }

    /// Builds the building at `building` of the town's record in the town whose entrance is
    /// `tile` (E09 T19c1).
    pub const fn build(tile: u32, building: u8) -> Self {
        Self::Adventure(MapAction {
            entry: BUILD,
            tile,
            building,
        })
    }

    /// Opens the hire dialog of the building at `building` of the town's record in the town whose
    /// entrance is the tile at `tile` (E09 T27).
    pub const fn recruit(tile: u32, building: u8) -> Self {
        Self::Adventure(MapAction {
            entry: RECRUIT,
            tile,
            building,
        })
    }

    /// Moves a hero between the two bars of the town whose entrance is the tile at `tile`
    /// (E09 T28a): `change`'s entry from [`TOWN_HEROES`].
    pub fn town_heroes(tile: u32, change: TownHeroChange) -> Self {
        Self::map(TOWN_HEROES + town_hero_index(change), tile)
    }

    /// Hires the hero in `slot` of the player's tavern at the town whose entrance is the tile at
    /// `tile` (E09 T38a2): [`TAVERN_HIRE`] plus the slot.
    pub const fn tavern_hire(tile: u32, slot: usize) -> Self {
        Self::map(TAVERN_HIRE + slot, tile)
    }

    /// Opens the town screen of the town whose entrance is the tile at `tile` (E09 T49a2):
    /// [`TOWN_OPEN`].
    pub const fn town_open(tile: u32) -> Self {
        Self::map(TOWN_OPEN, tile)
    }

    /// Buys a spell book at the Mage Guild of the town whose entrance is the tile at `tile` (E09
    /// T49b): [`SPELL_BOOK`].
    pub const fn spell_book(tile: u32) -> Self {
        Self::map(SPELL_BOOK, tile)
    }

    /// The map phase's action of `entry` with `tile` (building 0).
    pub const fn map(entry: usize, tile: u32) -> Self {
        Self::Adventure(MapAction {
            entry,
            tile,
            building: 0,
        })
    }
}

/// The map's number of tiles: the size of the tile head.
pub fn tile_count(state: &GameState) -> usize {
    let map = state.adventure().map();
    usize::from(map.width()) * usize::from(map.height()) * usize::from(map.levels())
}

/// The tile head's index of `pos`, if it is on the map.
pub fn tile_index(state: &GameState, pos: Pos) -> Option<u32> {
    let map = state.adventure().map();
    let (width, height) = (u32::from(map.width()), u32::from(map.height()));
    (pos.x < map.width() && pos.y < map.height() && pos.level < map.levels())
        .then(|| (u32::from(pos.level) * height + u32::from(pos.y)) * width + u32::from(pos.x))
}

/// The tile at the tile head's `index`, if it is on the map.
pub fn tile_pos(state: &GameState, index: u32) -> Option<Pos> {
    let map = state.adventure().map();
    let (width, height) = (u32::from(map.width()), u32::from(map.height()));
    let level = u8::try_from(index / (width * height)).ok()?;
    let rest = index % (width * height);
    let x = u16::try_from(rest % width).ok()?;
    let y = u16::try_from(rest / width).ok()?;
    (level < map.levels()).then(|| Pos::new(x, y, level))
}

/// The tile head's index of `town`'s entrance (E09 T19c1): the first tile its object is visited
/// from, where its visitor stands (TWN-VIS-001); none for a town the map does not have.
pub fn town_tile(state: &GameState, town: TownId) -> Option<u32> {
    let map = state.adventure().map();
    map.town_object(town)
        .and_then(|index| map.objects()[index].entrance(map))
        .and_then(|pos| tile_index(state, pos))
}

/// The town whose entrance is the tile at the tile head's `index` (E09 T19c1), if one's is.
fn town_of_tile(state: &GameState, index: u32) -> Option<TownId> {
    state.adventure().map().town_at(tile_pos(state, index)?)
}

/// The hero in `slot` of `player`: its heroes in the order of their ids.
pub fn hero_in_slot(state: &GameState, player: PlayerId, slot: usize) -> Option<HeroId> {
    (slot < HERO_SLOTS)
        .then(|| {
            state
                .adventure()
                .heroes()
                .iter()
                .filter(|hero| hero.owner == player)
                .nth(slot)
                .map(|hero| hero.id)
        })
        .flatten()
}

/// The phase `seat` acts in: for a player the battle phase while it is in an open battle
/// (GAM-BAT-001; its first open decision is the battle), else the map phase; the battle phase for
/// the neutral seat, which has no other.
pub fn phase(state: &GameState, seat: impl Into<Seat>) -> Phase {
    match seat.into() {
        Seat::Player(player) if state.battle(player).is_none() => Phase::Adventure,
        _ => Phase::Battle,
    }
}

/// The open battle of `seat` and its side there: a player's battle and the side it owns, if it is
/// in one; for the neutral seat the first battle whose side to act no player owns, and that side.
pub fn battle_of(state: &GameState, seat: impl Into<Seat>) -> Option<(&BattleState, BattleSide)> {
    match seat.into() {
        Seat::Player(player) => {
            let battle = state.battle(player)?;
            Some((battle.state(), battle.side_of(player)?))
        }
        Seat::Neutral => {
            let battle = state.unowned_battle()?.state();
            Some((battle, battle.side_to_act()?))
        }
    }
}

/// The legal moves of the hero in `slot` of `seat`'s player ([`legal_moves`]), if any tile is
/// reachable but its own; none for the neutral seat.
pub fn moves(
    state: &GameState,
    rules: &GameRuleset,
    seat: impl Into<Seat>,
    slot: usize,
) -> Option<Reach> {
    let player = seat.into().player()?;
    let hero = hero_in_slot(state, player, slot)?;
    legal_moves(state, rules, Actor::Player(player), hero).filter(|r| r.tiles().next().is_some())
}

/// Fills `mask` with the legal map entries of `seat`: the commands [`legal_commands`] lists
/// (`EndTurn` while its turn runs, and [`BUILD_BOAT`] when it lists a `BuildBoat`, E08 T85b2b; a
/// hero's revisit entry when it lists the hero's `Revisit`, E08 T105b2; [`BUILD`] when it lists a
/// town's build, E09 T19c1, [`RECRUIT`] a town's recruitment, E09 T27, a [`TOWN_HEROES`] entry
/// a hero move of its kind, E09 T28a, a [`TAVERN_HIRE`] entry a hire of its slot, E09 T38a2,
/// [`TOWN_OPEN`] a town's opening, E09 T49a2, [`SPELL_BOOK`] a spell book's purchase, E09 T49b;
/// or with an open decision its answers alone: `Decline`, and `Accept` while the offer stands,
/// E08 T15a; a level-up's `First` and `Second`, E08 T30; a hire dialog's purchases by their
/// creatures' places, E08 T95b), and each hero slot with a tile to move to ([`legal_moves`]: none
/// with a decision open); none in the battle phase, where the listed commands are the battle's
/// and no hero moves, so none for the neutral seat.
pub fn legal_mask(
    state: &GameState,
    rules: &GameRuleset,
    seat: impl Into<Seat>,
    mask: &mut EntryMask,
) {
    let seat = seat.into();
    mask.fill(false);
    let listed = legal_commands(state, rules, seat.actor());
    let hire = listed_hire(state, rules, seat, &listed);
    for command in &listed {
        if let Some(entry) = command_entry(state, seat, command, &hire) {
            mask[entry] = true;
        }
    }
    for slot in 0..HERO_SLOTS {
        mask[MOVE_BASE + slot] = moves(state, rules, seat, slot).is_some();
    }
}

/// Fills `mask` (resized to the tile count) with the legal tiles of `seat` after choosing
/// `entry`: 0 alone after a legal entry that takes no tile (`EndTurn`, an answer, a revisit), the
/// reachable tiles of a hero's legal move but its own, the anchors of the Shipyards of the listed
/// `BuildBoat`s after [`BUILD_BOAT`] (E08 T85b2b), the entrances of the towns of the listed builds
/// after [`BUILD`] (E09 T19c1, [`town_tile`]) and of the listed recruitments after [`RECRUIT`]
/// (E09 T27), of the listed hero moves of a [`TOWN_HEROES`] entry's kind after it (E09 T28a), and
/// of the listed hires of a [`TAVERN_HIRE`] entry's slot after it (E09 T38a2), and of the listed
/// openings after [`TOWN_OPEN`] (E09 T49a2), and of the listed spell books' purchases after
/// [`SPELL_BOOK`] (E09 T49b); none after an illegal entry or in the battle phase
/// (none for the neutral seat).
pub fn tile_mask(
    state: &GameState,
    rules: &GameRuleset,
    seat: impl Into<Seat>,
    entry: usize,
    mask: &mut Vec<bool>,
) {
    let seat = seat.into();
    mask.clear();
    mask.resize(tile_count(state), false);
    if tileless(entry) {
        let listed = legal_commands(state, rules, seat.actor());
        let hire = listed_hire(state, rules, seat, &listed);
        mask[0] = listed
            .iter()
            .any(|command| command_entry(state, seat, command, &hire) == Some(entry));
    } else if entry == BUILD_BOAT {
        for command in legal_commands(state, rules, seat.actor()) {
            if let GameCommand::Adventure(AdventureCommand::BuildBoat { anchor, .. }) = command
                && let Some(index) = tile_index(state, anchor)
            {
                mask[index as usize] = true;
            }
        }
    } else if entry == BUILD || entry == RECRUIT {
        for command in legal_commands(state, rules, seat.actor()) {
            if let Some((town, _)) = town_command(entry, &command)
                && let Some(index) = town_tile(state, town)
            {
                mask[index as usize] = true;
            }
        }
    } else if let Some(change) = town_hero_move(entry) {
        for command in legal_commands(state, rules, seat.actor()) {
            if let GameCommand::Town(TownCommand::Heroes {
                town,
                change: listed,
            }) = command
                && listed == change
                && let Some(index) = town_tile(state, town)
            {
                mask[index as usize] = true;
            }
        }
    } else if let Some(slot) = tavern_slot(entry) {
        for command in legal_commands(state, rules, seat.actor()) {
            if let GameCommand::Town(TownCommand::Hire { town, slot: listed }) = command
                && listed == slot
                && let Some(index) = town_tile(state, town)
            {
                mask[index as usize] = true;
            }
        }
    } else if entry == TOWN_OPEN || entry == SPELL_BOOK {
        for command in legal_commands(state, rules, seat.actor()) {
            if let Some(town) = guild_command(entry, &command)
                && let Some(index) = town_tile(state, town)
            {
                mask[index as usize] = true;
            }
        }
    } else if let Some(reach) = entry
        .checked_sub(MOVE_BASE)
        .and_then(|slot| moves(state, rules, seat, slot))
    {
        for (pos, _) in reach.tiles() {
            if let Some(index) = tile_index(state, pos) {
                mask[index as usize] = true;
            }
        }
    }
}

/// Fills `mask` with the legal buildings of `seat` after choosing `entry` and `tile` (E09 T19c1):
/// after [`BUILD`] the buildings of the builds [`legal_commands`] lists for the town whose
/// entrance is `tile`, by their indices in its record (none for a tile with no such town), after
/// [`RECRUIT`] those of its recruitments (E09 T27); after another entry 0 alone when `tile` is in its tile mask ([`tile_mask`]); none otherwise, in the
/// battle phase or for the neutral seat.
pub fn building_mask(
    state: &GameState,
    rules: &GameRuleset,
    seat: impl Into<Seat>,
    entry: usize,
    tile: u32,
    mask: &mut BuildingMask,
) {
    let seat = seat.into();
    mask.fill(false);
    if entry == BUILD || entry == RECRUIT {
        let Some(at) = town_of_tile(state, tile) else {
            return;
        };
        for command in legal_commands(state, rules, seat.actor()) {
            if let Some((town, building)) = town_command(entry, &command)
                && town == at
            {
                mask[building.index()] = true;
            }
        }
    } else {
        let mut tiles = Vec::new();
        tile_mask(state, rules, seat, entry, &mut tiles);
        mask[0] = tiles.get(tile as usize).copied().unwrap_or(false);
    }
}

/// The town of `command` when it is the town command of `entry`: [`TOWN_OPEN`]'s opening (E09
/// T49a2), [`SPELL_BOOK`]'s purchase (E09 T49b).
fn guild_command(entry: usize, command: &GameCommand) -> Option<TownId> {
    match *command {
        GameCommand::Town(TownCommand::Open { town }) if entry == TOWN_OPEN => Some(town),
        GameCommand::Town(TownCommand::BuySpellBook { town }) if entry == SPELL_BOOK => Some(town),
        _ => None,
    }
}

/// The town and building of `command` when it is the town command of `entry`: [`BUILD`]'s build
/// (E09 T19c1), [`RECRUIT`]'s recruitment (E09 T27).
fn town_command(entry: usize, command: &GameCommand) -> Option<(TownId, BuildingId)> {
    match *command {
        GameCommand::Town(TownCommand::Build { town, building }) if entry == BUILD => {
            Some((town, building))
        }
        GameCommand::Town(TownCommand::Recruit { town, building }) if entry == RECRUIT => {
            Some((town, building))
        }
        _ => None,
    }
}

/// Fills `mask` with the legal entries of the battle phase for `seat` ([`battle_of`]): its
/// battle's ([`battle::legal_mask`]) for its side while that side is to act; none otherwise.
/// `legal` is scratch space.
pub fn battle_mask(
    state: &GameState,
    rules: &GameRuleset,
    seat: impl Into<Seat>,
    legal: &mut LegalCommands,
    mask: &mut battle::ActionMask,
) {
    mask.fill(false);
    if let Some((battle, side)) = battle_of(state, seat) {
        battle::legal_mask(battle, rules.battle(), side, legal, mask);
    }
}

/// Fills `mask` with the legal spells of the battle phase for `seat` after `entry`
/// ([`battle::spell_mask`]); none outside a battle.
pub fn spell_mask(
    state: &GameState,
    rules: &GameRuleset,
    seat: impl Into<Seat>,
    entry: usize,
    legal: &mut LegalCommands,
    mask: &mut battle::SpellMask,
) {
    mask.fill(false);
    if let Some((battle, side)) = battle_of(state, seat) {
        battle::spell_mask(battle, rules.battle(), side, entry, legal, mask);
    }
}

/// Fills `mask` with the legal targets of the battle phase for `seat` after `entry` and
/// `spell` ([`battle::target_mask`]); none outside a battle.
pub fn target_mask(
    state: &GameState,
    rules: &GameRuleset,
    seat: impl Into<Seat>,
    prefix: (usize, u16),
    legal: &mut LegalCommands,
    mask: &mut battle::TargetMask,
) {
    mask.fill(false);
    if let Some((battle, side)) = battle_of(state, seat) {
        battle::target_mask(battle, rules.battle(), side, prefix, legal, mask);
    }
}

/// Fills `mask` with the legal second targets of the battle phase for `seat` after `entry`,
/// `spell` and `target` ([`battle::second_mask`]); none outside a battle.
pub fn second_mask(
    state: &GameState,
    rules: &GameRuleset,
    seat: impl Into<Seat>,
    prefix: (usize, u16, u16),
    legal: &mut LegalCommands,
    mask: &mut battle::TargetMask,
) {
    mask.fill(false);
    if let Some((battle, side)) = battle_of(state, seat) {
        battle::second_mask(battle, rules.battle(), side, prefix, legal, mask);
    }
}

/// Whether `entry` takes no tile: `EndTurn`, an answer (a hire dialog's purchase too; since E08
/// T135a the trades' and services') or a revisit (E08 T105b2); not [`TOWN_OPEN`] (E09 T49a2) nor
/// [`SPELL_BOOK`] (E09 T49b).
const fn tileless(entry: usize) -> bool {
    entry == END_TURN
        || (entry >= ACCEPT && entry < BUILD_BOAT)
        || (entry >= HIRE_BASE && entry < BUILD)
        || (entry >= MARKET_TRADE && entry < TOWN_OPEN)
}

/// The [`MARKET_TRADE`] entry of the pair (`give`, `take`) (E08 T135a): `MARKET_TRADE + give ×
/// 6` plus the place of `take` among the other resources, in the core's order; none for a
/// resource past the first [`MARKET_RESOURCES`] or a resource for itself.
pub const fn trade_entry(give: ResourceId, take: ResourceId) -> Option<usize> {
    let (give, take) = (give.index(), take.index());
    if give >= MARKET_RESOURCES || take >= MARKET_RESOURCES || give == take {
        return None;
    }
    let other = if take > give { take - 1 } else { take };
    Some(MARKET_TRADE + give * (MARKET_RESOURCES - 1) + other)
}

/// The pair at `index` among [`MARKET_TRADE`]'s entries (E08 T135a): the given and the wanted
/// resource ([`trade_entry`]'s inverse); none past [`MARKET_PAIRS`].
pub fn market_pair(index: usize) -> Option<(ResourceId, ResourceId)> {
    if index >= MARKET_PAIRS {
        return None;
    }
    let give = index / (MARKET_RESOURCES - 1);
    let other = index % (MARKET_RESOURCES - 1);
    let take = if other >= give { other + 1 } else { other };
    let resource = |at: usize| ResourceId::from_index(at).unwrap_or_default();
    Some((resource(give), resource(take)))
}

/// The entry of `answer` by its role (E08 T63a; the module's table): an offer's acceptance; a
/// declined offer, a closed meeting, army dialog or hire dialog (E08 T95b), a School or a
/// University left; the first or the second of a level-up's, a chest's, the Arena's or a School's
/// answers; a University's purchase of one of its first [`UNIVERSITY_SLOTS`] slots (none for a
/// later one); since version 13 (E09 T39a) a map Tavern's `Close` the closing role, its hire of
/// slot 0 the first, of slot 1 the second (none for another slot); since version 14 (E08 T133b) a
/// market's `Leave` the closing role (none for its other answers until T135a); since version 15
/// (E08 T134a) an Altar's `Leave` the closing role (none for its sacrifices until T135a); since
/// version 16 (E08 T134b1) a Hill Fort's `Leave` the closing role; since version 17 (E08 T135a)
/// a market's trade of one lot by its pair ([`trade_entry`]: none for another lot or a pair past
/// the entries), its sale of one lot by its slot and its purchase by its entry (whatever the
/// resource: the listed one is the cost resource's, [`encode`] names no other), an Altar's
/// sacrifices by their kind (whatever the set: the listed one is the all), a Hill Fort's upgrade
/// by its slot and its `All` (none for a slot or an entry past the entries). None for a hire
/// dialog's purchase, whose entry is its creature's place in the dialog's source, which the answer
/// does not carry ([`hire_entry`], E08 T95b), and for a market's transfer.
pub const fn answer_entry(answer: Answer) -> Option<usize> {
    match answer {
        Answer::Offer(Consent::Accept) => Some(ACCEPT),
        Answer::Offer(Consent::Decline)
        | Answer::Close
        | Answer::School(SchoolChoice::Leave)
        | Answer::University(UniversityAnswer::Leave)
        | Answer::Hire(HireAnswer::Close)
        | Answer::Tavern(TavernAnswer::Close)
        | Answer::Market(MarketAnswer::Leave)
        | Answer::Altar(AltarAnswer::Leave)
        | Answer::HillFort(HillFortAnswer::Leave) => Some(DECLINE),
        Answer::LevelUp(SkillOffer::First)
        | Answer::Chest(ChestChoice::Gold)
        | Answer::Arena(ArenaChoice::Attack)
        | Answer::School(SchoolChoice::First)
        | Answer::Tavern(TavernAnswer::Hire { slot: 0 }) => Some(FIRST),
        Answer::LevelUp(SkillOffer::Second)
        | Answer::Chest(ChestChoice::Experience)
        | Answer::Arena(ArenaChoice::Defense)
        | Answer::School(SchoolChoice::Second)
        | Answer::Tavern(TavernAnswer::Hire { slot: 1 }) => Some(SECOND),
        Answer::University(UniversityAnswer::Buy(slot)) => {
            let slot = slot as usize;
            if slot < UNIVERSITY_SLOTS {
                Some(BUY_BASE + slot)
            } else {
                None
            }
        }
        Answer::Market(MarketAnswer::Trade {
            give,
            take,
            lots: 1,
        }) => trade_entry(give, take),
        Answer::Market(MarketAnswer::Sell { slot, lots: 1, .. }) => {
            below(slot as usize, ARMY_SLOTS, MARKET_SELL)
        }
        Answer::Market(MarketAnswer::Buy { entry, .. }) => {
            below(entry as usize, MARKET_STOCK, MARKET_BUY)
        }
        Answer::Altar(AltarAnswer::Artifacts(_)) => Some(ALTAR_ARTIFACTS),
        Answer::Altar(AltarAnswer::Creatures(_)) => Some(ALTAR_CREATURES),
        Answer::HillFort(HillFortAnswer::Upgrade(slot)) => {
            below(slot as usize, ARMY_SLOTS, HILL_FORT_UPGRADE)
        }
        Answer::HillFort(HillFortAnswer::All) => Some(HILL_FORT_ALL),
        Answer::Hire(HireAnswer::Buy { .. })
        | Answer::Tavern(TavernAnswer::Hire { .. })
        | Answer::Market(_) => None,
    }
}

/// `base + at` for an `at` below `count`; none past it.
const fn below(at: usize, count: usize, base: usize) -> Option<usize> {
    if at < count { Some(base + at) } else { None }
}

/// Whether `answer`'s entry names one answer of its role among several (E08 T135a: a market's
/// sale or purchase, whose resource the entry leaves to the listing; an Altar's sacrifice, whose
/// set or amounts it does): [`encode`] gives its entry only for the answer [`legal_commands`]
/// lists.
const fn listed_only(answer: &Answer) -> bool {
    matches!(
        answer,
        Answer::Market(MarketAnswer::Sell { .. } | MarketAnswer::Buy { .. })
            | Answer::Altar(AltarAnswer::Artifacts(_) | AltarAnswer::Creatures(_))
    )
}

/// The answer the trade's or service's entry `entry` (from [`MARKET_TRADE`], below
/// [`ENTRY_COUNT`]) names when [`legal_commands`] lists none of its role (E08 T135a): the trade of
/// one lot of its pair, the sale of one lot of its slot's stack and the purchase of its entry in
/// the market parameters' cost resource (resource 0 without the parameters, where no market
/// opens), the Hill Fort's upgrade of its slot or `All`, which `apply` accepts or rejects with the
/// reason (no decision, the wrong kind, the price, the lot, the stack), or an Altar's empty
/// sacrifice, which it rejects.
fn service_answer(rules: &GameRuleset, entry: usize) -> Answer {
    let cost = rules
        .adventure()
        .params()
        .market()
        .map_or(ResourceId::default(), MarketParams::cost);
    // Each base's entries are fewer than 256.
    let slot = |base: usize| u8::try_from(entry - base).unwrap_or(u8::MAX);
    match entry {
        ALTAR_ARTIFACTS => Answer::Altar(AltarAnswer::Artifacts(AltarPlaces::NONE)),
        ALTAR_CREATURES => Answer::Altar(AltarAnswer::Creatures([0; ALTAR_SLOTS])),
        HILL_FORT_ALL => Answer::HillFort(HillFortAnswer::All),
        _ if entry >= HILL_FORT_UPGRADE => {
            Answer::HillFort(HillFortAnswer::Upgrade(slot(HILL_FORT_UPGRADE)))
        }
        _ if entry >= MARKET_BUY => Answer::Market(MarketAnswer::Buy {
            entry: slot(MARKET_BUY),
            pay: cost,
        }),
        _ if entry >= MARKET_SELL => Answer::Market(MarketAnswer::Sell {
            slot: slot(MARKET_SELL),
            take: cost,
            lots: 1,
        }),
        _ => {
            let (give, take) = market_pair(entry - MARKET_TRADE).unwrap_or_default();
            Answer::Market(MarketAnswer::Trade {
                give,
                take,
                lots: 1,
            })
        }
    }
}

/// The creatures of `player`'s open hire dialog in its source's order (E08 T95b): its source's
/// stock (`GameState::hire_dialog`, `GameState::hire_stock`: since E09 T27 a town's too), as its owner's view
/// (`DecisionView::Hire`) lists them; none when its first open decision is no hire dialog. Read
/// only for a hire entry or a listed purchase ([`listed_hire`]).
fn hire_creatures(state: &GameState, rules: &GameRuleset, player: PlayerId) -> Vec<CreatureTypeId> {
    state
        .hire_dialog(player)
        .and_then(|dialog| state.hire_stock(rules, dialog))
        .unwrap_or_default()
        .into_iter()
        .map(|(creature, _)| creature)
        .collect()
}

/// [`hire_creatures`] of `seat`'s player when `listed`, its legal commands, hold a hire dialog's
/// purchase; none otherwise, without reading the view.
fn listed_hire(
    state: &GameState,
    rules: &GameRuleset,
    seat: Seat,
    listed: &[GameCommand],
) -> Vec<CreatureTypeId> {
    let buys = listed.iter().any(|command| {
        matches!(
            command,
            GameCommand::Answer(Answer::Hire(HireAnswer::Buy { .. }))
        )
    });
    match seat.player() {
        Some(player) if buys => hire_creatures(state, rules, player),
        _ => Vec::new(),
    }
}

/// The entry of a hire dialog's purchase of `creature` (E08 T95b): its place among `hire`, the
/// dialog's creatures in its source's order ([`hire_creatures`]); none past [`HIRE_SLOTS`] or for
/// a creature the dialog does not sell.
fn hire_entry(hire: &[CreatureTypeId], creature: CreatureTypeId) -> Option<usize> {
    hire.iter()
        .take(HIRE_SLOTS)
        .position(|&sold| sold == creature)
        .map(|place| HIRE_BASE + place)
}

/// The answer the hire entry of `place` names for `player` (E08 T95b): a `Buy` of the
/// creature in `place` of its open hire dialog, in its source's order, of the most, the amount of
/// the `Buy` [`legal_commands`] lists for it; while none is listed (its most is 0: no count left,
/// no room, the treasury short; or the game waits) a `Buy` of 0 of it, and with no creature in
/// that place (no hire dialog open, or `place` past its source's creatures) a `Buy` of 0 of
/// creature handle 0, a placeholder: each of which `apply` rejects (the amount, no decision, the
/// wrong kind or a creature the source does not sell).
fn hire_answer(state: &GameState, rules: &GameRuleset, player: PlayerId, place: usize) -> Answer {
    let creature = hire_creatures(state, rules, player).get(place).copied();
    let amount = creature.and_then(|creature| {
        legal_commands(state, rules, Actor::Player(player))
            .into_iter()
            .find_map(|command| match command {
                GameCommand::Answer(Answer::Hire(HireAnswer::Buy {
                    creature: listed,
                    amount,
                })) if listed == creature => Some(amount),
                _ => None,
            })
    });
    Answer::Hire(HireAnswer::Buy {
        creature: creature.unwrap_or_default(),
        amount: amount.unwrap_or(0),
    })
}

/// The answer an answer entry names for `player` (E08 T63a): the answer of its role that
/// [`legal_commands`] lists for its open decision, else the entry's answer to an offer or a
/// level-up, or its purchase, which `apply` rejects with the reason (no decision, the wrong
/// kind, the slot or the price), or since E08 T135a a trade's or service's
/// ([`service_answer`]); a hire entry's: [`hire_answer`] (E08 T95b).
fn answer_of(
    state: &GameState,
    rules: &GameRuleset,
    player: PlayerId,
    entry: usize,
) -> Option<Answer> {
    if let Some(place) = entry
        .checked_sub(HIRE_BASE)
        .filter(|&place| place < HIRE_SLOTS)
    {
        return Some(hire_answer(state, rules, player, place));
    }
    legal_commands(state, rules, Actor::Player(player))
        .into_iter()
        .find_map(|command| match command {
            GameCommand::Answer(answer) if answer_entry(answer) == Some(entry) => Some(answer),
            _ => None,
        })
        .or_else(|| match entry {
            ACCEPT => Some(Answer::Offer(Consent::Accept)),
            DECLINE => Some(Answer::Offer(Consent::Decline)),
            FIRST => Some(Answer::LevelUp(SkillOffer::First)),
            SECOND => Some(Answer::LevelUp(SkillOffer::Second)),
            _ if entry >= MARKET_TRADE => Some(service_answer(rules, entry)),
            _ => u8::try_from(entry - BUY_BASE)
                .ok()
                .map(|slot| Answer::University(UniversityAnswer::Buy(slot))),
        })
}

/// The entry of a command that takes no tile: `EndTurn` or an answer ([`answer_entry`]; a hire
/// dialog's purchase by its creature's place among `hire`, the dialog's creatures, [`hire_entry`]).
fn tileless_entry(command: &GameCommand, hire: &[CreatureTypeId]) -> Option<usize> {
    match command {
        GameCommand::EndTurn => Some(END_TURN),
        GameCommand::Answer(Answer::Hire(HireAnswer::Buy { creature, .. })) => {
            hire_entry(hire, *creature)
        }
        GameCommand::Answer(answer) => answer_entry(*answer),
        // E08 T51a: the management commands have no entry (decision 99: the AI lane's call); a
        // town's build takes a tile ([`command_entry`]'s, E09 T19c1); the system's own commands
        // (E16 M5 T14) are the host's, never listed for a seat.
        GameCommand::Adventure(_)
        | GameCommand::Battle(_)
        | GameCommand::Manage(_)
        | GameCommand::Town(_)
        | GameCommand::System(_) => None,
    }
}

/// The entry of a command [`legal_commands`] lists outside a battle for `seat`:
/// [`tileless_entry`]'s (with `hire`, the open hire dialog's creatures), [`BUILD_BOAT`] for a
/// `BuildBoat` (E08 T85b2b), a `Revisit`'s by its hero's slot ([`revisit_entry`], E08 T105b2),
/// [`BUILD`] for a town's build (E09 T19c1), [`RECRUIT`] for its recruitment (E09 T27), a
/// [`TOWN_HEROES`] entry for a hero move (E09 T28a), a [`TAVERN_HIRE`] entry for a hire (E09
/// T38a2), [`TOWN_OPEN`] for an opening (E09 T49a2), or [`SPELL_BOOK`] for a spell book's
/// purchase (E09 T49b).
fn command_entry(
    state: &GameState,
    seat: Seat,
    command: &GameCommand,
    hire: &[CreatureTypeId],
) -> Option<usize> {
    match command {
        GameCommand::Adventure(AdventureCommand::BuildBoat { .. }) => Some(BUILD_BOAT),
        GameCommand::Adventure(AdventureCommand::Revisit { hero }) => {
            revisit_entry(state, seat.player()?, *hero)
        }
        GameCommand::Town(TownCommand::Build { .. }) => Some(BUILD),
        GameCommand::Town(TownCommand::Recruit { .. }) => Some(RECRUIT),
        GameCommand::Town(TownCommand::Heroes { change, .. }) => {
            Some(TOWN_HEROES + town_hero_index(*change))
        }
        GameCommand::Town(TownCommand::Hire { slot, .. }) => tavern_entry(*slot),
        GameCommand::Town(TownCommand::Open { .. }) => Some(TOWN_OPEN),
        GameCommand::Town(TownCommand::BuySpellBook { .. }) => Some(SPELL_BOOK),
        other => tileless_entry(other, hire),
    }
}

/// The place of `change` among [`TOWN_HERO_MOVES`] (E09 T28a).
const fn town_hero_index(change: TownHeroChange) -> usize {
    match change {
        TownHeroChange::Enter => 0,
        TownHeroChange::Leave => 1,
        TownHeroChange::Swap => 2,
    }
}

/// The hero move of a [`TOWN_HEROES`] entry (E09 T28a); none for another entry.
fn town_hero_move(entry: usize) -> Option<TownHeroChange> {
    entry
        .checked_sub(TOWN_HEROES)
        .and_then(|m| TOWN_HERO_MOVES.get(m).copied())
}

/// The [`TAVERN_HIRE`] entry of a hire of tavern slot `slot` (E09 T38a2); none past the slots.
fn tavern_entry(slot: u8) -> Option<usize> {
    let slot = usize::from(slot);
    (slot < TAVERN_SLOTS).then_some(TAVERN_HIRE + slot)
}

/// The tavern slot of a [`TAVERN_HIRE`] entry (E09 T38a2); none for another entry.
fn tavern_slot(entry: usize) -> Option<u8> {
    entry
        .checked_sub(TAVERN_HIRE)
        .filter(|&s| s < TAVERN_SLOTS)
        .and_then(|s| u8::try_from(s).ok())
}

/// The revisit entry of `player`'s hero `hero` (E08 T105b2): [`REVISIT_BASE`] plus its slot among
/// the player's heroes ([`hero_in_slot`]'s order); none for a hero past the slots, off the map or
/// another player's.
fn revisit_entry(state: &GameState, player: PlayerId, hero: HeroId) -> Option<usize> {
    state
        .adventure()
        .heroes()
        .iter()
        .filter(|h| h.owner == player)
        .position(|h| h.id == hero)
        .filter(|&slot| slot < HERO_SLOTS)
        .map(|slot| REVISIT_BASE + slot)
}

/// The `BuildBoat` of `player` at the Shipyard anchored on `anchor` (E08 T85b2b): one the
/// adventure lets it use now (`AdventureState::buildable_shipyards`: its own or an ally's, the
/// boat tile free), the first in object order; the game's own checks (the turn, an open
/// decision, the price) are `apply`'s.
fn build_boat(
    state: &GameState,
    rules: &GameRuleset,
    player: PlayerId,
    anchor: Pos,
) -> Option<GameCommand> {
    state
        .adventure()
        .buildable_shipyards(rules.adventure(), player)
        .find(|&(_, at, _)| at == anchor)
        .map(|(object_type, anchor, _)| {
            GameCommand::Adventure(AdventureCommand::BuildBoat {
                object_type,
                anchor,
            })
        })
}

/// The command `seat` issues with `action`, if the action names one. In the map phase:
/// `EndTurn` or an answer with tile 0 (a hire entry's: [`hire_answer`], E08 T95b), a `Revisit` of
/// the hero in its entry's slot with tile 0 (E08 T105b2), a move of one of its player's heroes to
/// a tile its reachable-tiles query lists (not its own) along the query's path, a `BuildBoat`
/// at a Shipyard its player may use, anchored on the tile ([`build_boat`], E08 T85b2b), or a
/// town's build of the building at the building head's index in the town whose entrance is the
/// tile (E09 T19c1), or since E09 T27 its recruitment there, or since E09 T28a a hero's move
/// between its bars of a [`TOWN_HEROES`] entry's kind (building 0), or since E09 T38a2 its hire of
/// the hero in the player's tavern slot of a [`TAVERN_HIRE`] entry (building 0), or since E09
/// T49a2 its opening with [`TOWN_OPEN`] (building 0), or since E09 T49b its spell book's purchase
/// with [`SPELL_BOOK`] (building 0) (a building other than 0 with another entry
/// names none); none for the neutral seat. In the battle phase: the battle command the battle's encoding decodes the
/// action to ([`battle::decode`]), for the seat's battle ([`battle_of`]). Whether the command may
/// be issued now (whose turn it is, an open decision, the price, the side to act) is `apply`'s.
pub fn decode(
    state: &GameState,
    rules: &GameRuleset,
    seat: impl Into<Seat>,
    action: Action,
) -> Option<GameCommand> {
    let seat = seat.into();
    let action = match action {
        Action::Adventure(action) => action,
        Action::Battle(action) => {
            let (battle, _) = battle_of(state, seat)?;
            return battle::decode(battle, rules.battle(), action).map(GameCommand::Battle);
        }
    };
    let player = seat.player()?;
    if let Some(change) = town_hero_move(action.entry) {
        let town = town_of_tile(state, action.tile)?;
        return (action.building == 0)
            .then_some(GameCommand::Town(TownCommand::Heroes { town, change }));
    }
    if let Some(slot) = tavern_slot(action.entry) {
        let town = town_of_tile(state, action.tile)?;
        return (action.building == 0)
            .then_some(GameCommand::Town(TownCommand::Hire { town, slot }));
    }
    if action.entry == TOWN_OPEN || action.entry == SPELL_BOOK {
        let town = town_of_tile(state, action.tile)?;
        let command = if action.entry == TOWN_OPEN {
            TownCommand::Open { town }
        } else {
            TownCommand::BuySpellBook { town }
        };
        return (action.building == 0).then_some(GameCommand::Town(command));
    }
    if action.entry == BUILD || action.entry == RECRUIT {
        let town = town_of_tile(state, action.tile)?;
        let building = BuildingId::from_index(usize::from(action.building))?;
        return Some(GameCommand::Town(if action.entry == BUILD {
            TownCommand::Build { town, building }
        } else {
            TownCommand::Recruit { town, building }
        }));
    }
    if action.building != 0 {
        return None;
    }
    if tileless(action.entry) {
        if action.tile != 0 {
            return None;
        }
        let revisit = action.entry.checked_sub(REVISIT_BASE);
        if let Some(slot) = revisit.filter(|&slot| slot < HERO_SLOTS) {
            let hero = hero_in_slot(state, player, slot)?;
            return Some(GameCommand::Adventure(AdventureCommand::Revisit { hero }));
        }
        if action.entry == END_TURN {
            return Some(GameCommand::EndTurn);
        }
        return answer_of(state, rules, player, action.entry).map(GameCommand::Answer);
    }
    if action.entry == BUILD_BOAT {
        return build_boat(state, rules, player, tile_pos(state, action.tile)?);
    }
    let hero = hero_in_slot(state, player, action.entry.checked_sub(MOVE_BASE)?)?;
    let to = tile_pos(state, action.tile)?;
    let reach = state
        .adventure()
        .reach(rules.core(), rules.adventure(), state.heroes(), hero)?;
    let path = reach.path(to).filter(|path| !path.is_empty())?;
    Some(GameCommand::Adventure(AdventureCommand::MoveHero {
        hero,
        path,
    }))
}

/// The action that names `command` of `seat`, if one does: `EndTurn` and the answers by their
/// entries (tile 0; a hire dialog's purchase by its creature's place when it is the `Buy`
/// [`legal_commands`] lists for it, of the most: an entry buys no other amount, E08 T95b), a move
/// of one of its player's heroes by the hero's slot and the path's last tile, a `BuildBoat` by
/// its Shipyard's anchor (E08 T85b2b), a `Revisit` by its hero's slot (tile 0, E08 T105b2), a
/// town's build by its town's entrance and its building's index (E09 T19c1), its recruitment the
/// same way (E09 T27), a hero's move between its bars by its kind's entry and the town's entrance
/// (E09 T28a), a hire by its slot's entry and the town's entrance (E09 T38a2), an opening by
/// [`TOWN_OPEN`] and the town's entrance (E09 T49a2), a spell book's purchase by [`SPELL_BOOK`]
/// and the town's entrance (E09 T49b; none of these for the neutral seat), a
/// battle command by the
/// battle's encoding for the seat's battle ([`battle::encode`]). [`decode`] gives the command back
/// when a move's path is the one the hero's query chose (a move along another path decodes to the
/// query's path to the same tile) and a `BuildBoat`'s Shipyard is one the player may use.
pub fn encode(
    state: &GameState,
    rules: &GameRuleset,
    seat: impl Into<Seat>,
    command: &GameCommand,
) -> Option<Action> {
    let seat = seat.into();
    if let GameCommand::Battle(command) = command {
        let (battle, _) = battle_of(state, seat)?;
        return battle::encode(battle, rules.battle(), *command).map(Action::Battle);
    }
    let player = seat.player()?;
    let (hero, path) = match command {
        GameCommand::Adventure(AdventureCommand::MoveHero { hero, path }) => (*hero, path),
        GameCommand::Adventure(AdventureCommand::BuildBoat { anchor, .. }) => {
            return tile_index(state, *anchor).map(Action::build_boat);
        }
        GameCommand::Adventure(AdventureCommand::Revisit { hero }) => {
            return revisit_entry(state, player, *hero).map(|entry| Action::map(entry, 0));
        }
        GameCommand::Town(TownCommand::Build { town, building }) => {
            return town_tile(state, *town).map(|tile| Action::build(tile, building.raw()));
        }
        GameCommand::Town(TownCommand::Recruit { town, building }) => {
            return town_tile(state, *town).map(|tile| Action::recruit(tile, building.raw()));
        }
        GameCommand::Town(TownCommand::Heroes { town, change }) => {
            return town_tile(state, *town).map(|tile| Action::town_heroes(tile, *change));
        }
        GameCommand::Town(TownCommand::Hire { town, slot }) => {
            tavern_entry(*slot)?;
            return town_tile(state, *town)
                .map(|tile| Action::tavern_hire(tile, usize::from(*slot)));
        }
        GameCommand::Town(TownCommand::Open { town }) => {
            return town_tile(state, *town).map(Action::town_open);
        }
        GameCommand::Town(TownCommand::BuySpellBook { town }) => {
            return town_tile(state, *town).map(Action::spell_book);
        }
        GameCommand::Answer(Answer::Hire(HireAnswer::Buy { .. })) => {
            let listed = legal_commands(state, rules, seat.actor());
            let hire = if listed.contains(command) {
                hire_creatures(state, rules, player)
            } else {
                Vec::new()
            };
            return tileless_entry(command, &hire).map(|entry| Action::map(entry, 0));
        }
        // E08 T135a: a sale's, a purchase's or a sacrifice's entry names the listed one alone.
        GameCommand::Answer(answer) if listed_only(answer) => {
            let listed = legal_commands(state, rules, seat.actor()).contains(command);
            return listed
                .then(|| tileless_entry(command, &[]))
                .flatten()
                .map(|entry| Action::map(entry, 0));
        }
        other => return tileless_entry(other, &[]).map(|entry| Action::map(entry, 0)),
    };
    let slot = (0..HERO_SLOTS).find(|&slot| hero_in_slot(state, player, slot) == Some(hero))?;
    let tile = tile_index(state, *path.last()?)?;
    Some(Action::move_to(slot, tile))
}
