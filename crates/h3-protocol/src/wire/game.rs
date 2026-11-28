//! The whole game's published language (E16 M5 part 1, SRV-Q25; ADR-0018 points 5 and 7 as
//! amended): what a player sees of a game ([`GameObservation`]), its adventure map
//! ([`Adventure`]) and its battle (the battle's [`super::Observation`]); what it commands
//! ([`GameCommand`]) and why a command was rejected ([`GameReject`], T3a); what happened, as it
//! sees it ([`GameEvent`], T3b); how a viewer keeps its observation current by its events
//! ([`GameObservation::fold`], T3c) and by what it newly sees ([`GameReveal`], T5c); what the
//! game is made of ([`GameManifest`], T4); and what a seat asks of the server's rules
//! ([`GameQuery`], [`GameAnswer`], M5 T11).
//!
//! The same rules as [`super`]'s: the protocol's own types, translated by [`crate::map::game`]
//! with every engine field named; serde's default enum form with `deny_unknown_fields`; content
//! by the ruleset's dense handles; a 64-bit value as a [`WireU64`]. A whole map's tiles travel as
//! a palette and an index per tile ([`Adventure::tiles`]): a tile's view is a few fields, of which
//! a map holds a few dozen distinct values, and one packed number cannot hold every handle the
//! engine allows below JavaScript's 2^53.
//!
//! Provisional (E16 M5 part 1's review): the shapes, the palette.

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use serde::{Deserialize, Serialize};

use super::WireU64;

mod clock;
mod command;
mod event;
mod fold;
mod lobby;
mod manifest;
mod query;

pub use clock::{Duel, GameClock, OfferClock, Timer};
pub use command::{
    AdventureCommand, AdventureReject, AltarAnswer, AltarReject, Answer, ArenaChoice, ArmyCommand,
    ArmyRef, ArmyReject, ArmySlot, ArtifactAt, ArtifactPlace, CastReject, ChestChoice,
    CombineReject, Consent, GameCommand, GameReject, HillFortAnswer, HillFortReject, HireAnswer,
    HireReject, Manage, ManageReject, MarketAnswer, MarketReject, MoveReject, SchoolChoice,
    SkillOffer, SpellReject, StandReject, SystemCommand, TavernAnswer, TavernReject, TownCommand,
    TownHeroChange, TownReject, UniversityAnswer,
};
pub use event::{AdventureEvent, CastFailure, GameEvent, HeroChange, NothingWhy, ShipyardBlock};
pub use lobby::{Computer, MapGameInfo, MapOffer, SeatedPlayer};
pub use manifest::{GameKeyTables, GameManifest, GameRecordHeader, MapIdentity};
pub use query::{DIRECTIONS, GameAnswer, GameQuery, Moves, TownOffer};

/// A tile of the adventure map: its column, its row and its level (0 the surface, 1 the
/// underground).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pos {
    /// The column, from 0 at the left.
    pub x: u16,
    /// The row, from 0 at the top.
    pub y: u16,
    /// The level.
    pub level: u8,
}

/// An object of the map by its type and its anchor, as the engine names an object to a viewer
/// (ADR-0016 point 6): never by its index in the map's list, which would tell how many objects
/// the viewer has not seen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObjectRef {
    /// The object's type handle.
    pub object_type: u16,
    /// Its anchor: the bottom right tile its type's masks are relative to.
    pub anchor: Pos,
}

/// An amount of one resource.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Amount {
    /// The resource's handle.
    pub resource: u16,
    /// How much of it.
    pub amount: u32,
}

/// A stack of an army outside battle: a hero's, a garrison's, a monster's group.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArmyStack {
    /// Its army slot, from 0.
    pub slot: u8,
    /// The creature's handle.
    pub creature: u16,
    /// How many, at least 1.
    pub count: u32,
}

/// How a hero's army stands at a battle's start.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Formation {
    /// Spread from top to bottom.
    Loose,
    /// Clustered around the middle rows.
    Tight,
}

/// The adventure map as one player sees it (`h3_adventure::AdventureState::observe`,
/// ADV-FOG-004 to ADV-FOG-006): its tiles by what the player explored, the heroes, objects and
/// boats it sees, what it knows of objects, its team's kind marks and its own garrison heroes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Adventure {
    /// The player this view is for.
    pub viewer: u8,
    /// The map's width in tiles.
    pub width: u16,
    /// Its height in tiles.
    pub height: u16,
    /// Its levels: 1, or 2 with an underground.
    pub levels: u8,
    /// The words of [`Self::explored`] and [`Self::fringe`] per row: `width` divided by 64,
    /// rounded up.
    pub stride: u16,
    /// The tiles the player explored, level by level, row by row, `stride` words a row, the
    /// tile of column `x` as bit `x % 64` of word `x / 64` (bits past the width clear).
    pub explored: Vec<WireU64>,
    /// The fringe, laid out as [`Self::explored`]: the tiles not explored with an explored one
    /// among the eight around them on their level (ADV-FOG-005).
    pub fringe: Vec<WireU64>,
    /// The distinct tile views, in the order the tiles first use them.
    pub palette: Vec<Tile>,
    /// Each tile's view as its index in [`Self::palette`], in tile order (level, then row, then
    /// column): [`Self::tile`].
    pub tiles: Vec<u32>,
    /// The heroes it sees, by id: its own and those on a tile it sees them on.
    pub heroes: Vec<MapHero>,
    /// The objects still on the map that it sees, in tile order ([`Object::place`]; E16 M5 T5c),
    /// not the map's: a fold places an object it newly sees there ([`GameReveal`]), where the
    /// map's order would need each object's index, which counts the objects before it, seen or
    /// not (`docs/06-security.md`). A client finds an object's entry in the map's file by its
    /// type and anchor.
    pub objects: Vec<Object>,
    /// The Cartographer types the player has bought, in type order (ADV-VIS-001).
    pub bought: Vec<u16>,
    /// The keys it holds, bit `c` for the colour `c` (ADV-VIS-003).
    pub keys: u8,
    /// What it learned of objects still on the map by its own visits (ADV-VIS-006), in the
    /// map's order.
    pub known: Vec<KnownObject>,
    /// The empty boats it sees, in tile order.
    pub boats: Vec<Boat>,
    /// The object types whose kind mark its team holds, in type order (ADV-BON-009, E08 T115b).
    pub kinds: Vec<u16>,
    /// Its own garrison heroes, by town (E09 T28a, v64): off the map, each with its town in place
    /// of a tile; another viewer sees none.
    pub garrison_heroes: Vec<GarrisonHero>,
}

/// A hero standing in its town's garrison as its owner sees it (E09 T28a, v64).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GarrisonHero {
    /// The town.
    pub town: u16,
    /// The hero's id.
    pub id: u16,
    /// Its army, points and marks, as a hero on the map's own view.
    pub own: OwnMapHero,
}

impl Adventure {
    /// The view of the tile at `index` in tile order (level, then row, then column); `None`
    /// outside the map.
    pub fn tile(&self, index: usize) -> Option<&Tile> {
        self.tiles
            .get(index)
            .and_then(|&i| usize::try_from(i).ok())
            .and_then(|i| self.palette.get(i))
    }
}

/// A tile as a player sees it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Tile {
    /// Nothing: neither explored nor on the fringe.
    Hidden {},
    /// Its terrain, road and river, not what objects cover there: a tile of the fringe without
    /// `ADV-QUIRK-1` (ADV-FOG-005).
    Ground {
        /// The terrain's handle.
        terrain: u16,
        /// The road's handle, if any.
        road: Option<u16>,
        /// The river's handle, if any.
        river: Option<u16>,
    },
    /// Its terrain, road and river and what objects still on the map cover there: an explored
    /// tile (ADV-FOG-004), or a tile of the fringe under `ADV-QUIRK-1`.
    Seen {
        /// The terrain's handle.
        terrain: u16,
        /// The road's handle, if any.
        road: Option<u16>,
        /// The river's handle, if any.
        river: Option<u16>,
        /// An object's blocked tile (ADV-OBJ-001).
        blocked: bool,
        /// An object's visitable tile (ADV-OBJ-001).
        visitable: bool,
        /// Its top ground takes the native price away (Cursed Ground, ADV-MOV-004).
        no_native: bool,
        /// Its top ground is a ground of the sea (Favorable Winds, ADV-SEA-004).
        winds: bool,
        /// A hole lies on it: no landing onto it (ADV-SEA-003).
        hole: bool,
    },
}

/// A hero on the adventure map as a player sees it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MapHero {
    /// The hero's id.
    pub id: u16,
    /// Its owner.
    pub owner: u8,
    /// Its tile.
    pub pos: Pos,
    /// Its boat's type when it is in a boat (ADV-FOG-006: public, the original draws the hero's
    /// flag on its boat); never the boat's index, owner or captain.
    pub boat: Option<u8>,
    /// Its army and movement points, for its owner only (ADV-FOG-006).
    pub own: Option<OwnMapHero>,
}

/// What a hero's owner sees of it on the adventure map beyond the others' view.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OwnMapHero {
    /// Its army, by slot.
    pub army: Vec<ArmyStack>,
    /// Its movement points left today.
    pub movement: u32,
    /// Its day's maximum (ADR-0016 point 3).
    pub max_movement: u32,
    /// Its army's formation in battle.
    pub formation: Formation,
    /// Its tactics flag (ADV-ARM-006): off, it denies the tactics phase it would get
    /// (GAM-BAT-009).
    pub tactics: bool,
    /// The objects it holds the once-per-hero mark of (ADV-VIS-006), in the map's order.
    pub marks: Vec<ObjectRef>,
    /// The bonuses it holds until the end of its next battle (HER-ADV-008; v57).
    pub bonuses: Bonuses,
    /// Its timed marks: the object types (handles) whose day's or week's mark it holds, sorted
    /// (v57).
    pub timed: Vec<u16>,
    /// Its day spells (ADV-SPL-002; E08 T142a, v82).
    pub spells: DaySpells,
}

/// A hero's spells of the day (`h3_adventure::DaySpells`, ADV-SPL-002; E08 T142a, v82): its
/// owner's alone.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DaySpells {
    /// The flying level Fly set (the mastery 0 to 3), none when off.
    pub fly: Option<u8>,
    /// The water-walking level Water Walk set, none when off.
    pub water_walk: Option<u8>,
    /// The Dimension Door casts of the day.
    pub doors: u8,
}

/// An object as a player sees it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Object {
    /// The object's type handle.
    pub object_type: u16,
    /// Its anchor: the bottom right tile its masks are relative to.
    pub anchor: Pos,
    /// The tiles no hero enters: `h3_adventure::TileMask`'s bits, 8 columns by 6 rows (bit
    /// `row × 8 + column`) ending at the anchor.
    pub blocked: WireU64,
    /// The tiles a hero visits it from, laid out as [`Self::blocked`].
    pub visitable: WireU64,
    /// The tiles its picture is drawn on beyond those (where a special ground lies,
    /// ADV-GRD-001), laid out as [`Self::blocked`].
    pub drawn: WireU64,
    /// What the player sees of a wandering monster (ADV-FOG-006); none for every other object.
    pub monster: Option<Monster>,
    /// Its owner (ADV-OWN-003); none for a neutral object and one that is not ownable.
    pub owner: Option<u8>,
    /// A mine's production a day, for a player of its owner's team only (ADV-OWN-003).
    pub production: Option<Amount>,
    /// A mine's guards as the player sees them (ADV-OWN-003); none for every other object.
    pub guards: Option<Guards>,
    /// A weekly object's visited mark, the player's own (ADV-OWN-003): "visited once", never
    /// "emptied"; false for every other object.
    pub visited: bool,
    /// A Shipyard's boat tile, for a player of its owner's team only (ADV-SHY-001, ADV-OWN-003).
    pub boat_tile: Option<Pos>,
    /// The town it is (ADR-0028 point 3), whose view is the game's under that id.
    pub town: Option<u16>,
}

impl Object {
    /// Its place among a view's objects ([`Adventure::objects`]): its anchor's level, row and
    /// column, then its type and masks. Two objects share a place only as two special grounds of
    /// one type with the same masks, which the setup lets stack on one anchor (E08 T20a2a) and
    /// which are alike in every field.
    pub const fn place(&self) -> (u8, u16, u16, u16, u64, u64, u64) {
        (
            self.anchor.level,
            self.anchor.y,
            self.anchor.x,
            self.object_type,
            self.blocked.0,
            self.visitable.0,
            self.drawn.0,
        )
    }
}

/// A monster as a player sees it on the map.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Monster {
    /// The creature's handle.
    pub creature: u16,
    /// The word of its count: 0 the first (the original's "few"), each next word from the
    /// next threshold of the monsters' parameters (ADV-FOG-006).
    pub count_word: u8,
}

/// A mine's guards as a player sees them (ADV-OWN-003): the word of their total count and their
/// creature when they are of one; their stacks only to the owner's team (ADR-0016 point 6).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Guards {
    /// Their creature when all their stacks are of one; none for several.
    pub creature: Option<u16>,
    /// The word of their total count, every stack counted.
    pub count_word: u8,
    /// Their stacks by slot, for a player of the owner's team only.
    pub army: Option<Vec<ArmyStack>>,
}

/// What a player learned of an object by a visit of its own (ADV-VIS-006).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KnownObject {
    /// The object's type handle.
    pub object_type: u16,
    /// Its anchor.
    pub anchor: Pos,
    /// What the player knows of it.
    pub known: Known,
}

/// What a visit told its hero's owner of an object (ADV-VIS-006), as the original's right-click
/// shows it to the visitor's team.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Known {
    /// A Shrine's spell; none for one without a spell to give.
    Spell {
        /// The spell's handle.
        spell: Option<u16>,
    },
    /// A Witch Hut's skill; none for one whose every skill is banned.
    Skill {
        /// The skill's handle.
        skill: Option<u16>,
    },
    /// A gift's drawn price (the Tree of Knowledge's; empty: free).
    Price {
        /// The price, by resource.
        price: Vec<Amount>,
    },
    /// A University's skills.
    Skills {
        /// The skills' handles, in the draw's order.
        skills: Vec<u16>,
    },
    /// Whether a find is emptied, as it stands now.
    Emptied {
        /// It is emptied.
        emptied: bool,
    },
    /// That the player's team visited it, nothing more (a Fountain of Fortune, a Magic Spring;
    /// E08 T115b).
    Visited,
}

/// An empty boat as a player sees it (ADV-FOG-008): never its owner or captain, nor its index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Boat {
    /// Its tile.
    pub tile: Pos,
    /// Its type (appearance only).
    pub kind: u8,
    /// Its facing in the original's order of eight directions.
    pub facing: u8,
}

/// What a viewer sees now that it did not see through its explored tiles when it was last told
/// them (`h3_game::reveal`, SRV-Q26 (a); E16 M5 T5c): the game's `GameReveal`, each part as
/// [`GameObservation`] shows it. A viewer's observation folded with its events
/// ([`GameObservation::fold`]) and then this ([`GameObservation::reveal`]) is its observation
/// again.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GameReveal {
    /// The adventure map's part.
    pub adventure: AdventureReveal,
    /// The heroes of [`AdventureReveal::heroes`] with their types, in their order, as
    /// [`GameObservation::heroes`] holds them (none is the viewer's own: none has details).
    pub heroes: Vec<HeroInfo>,
    /// The viewer's views of [`AdventureReveal::towns`], in their order, as
    /// [`GameObservation::towns`] holds them (TWN-VEW-001 to -004).
    pub towns: Vec<Town>,
    /// The viewer's views of the map garrisons a tile of which it explored since, in their
    /// places' order, as [`GameObservation::garrisons`] holds them (ADV-GAR-006; protocol v54).
    pub garrisons: Vec<MapGarrison>,
}

/// The adventure map's part of a [`GameReveal`] (`h3_adventure::AdventureReveal`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdventureReveal {
    /// The player it is for.
    pub viewer: u8,
    /// The views the tiles whose sight changed take now, each once, in the order the areas'
    /// tiles first use them (E16 M5 T7a1: packed as [`Adventure::palette`]; one entry per tile
    /// made a whole map's reveal 60 times its snapshot).
    pub palette: Vec<RevealedView>,
    /// The tiles whose sight changed (explored, on the fringe or neither): for each level that
    /// has any, by level, the smallest rectangle holding them, each tile's view an index into
    /// [`Self::palette`].
    pub areas: Vec<RevealedArea>,
    /// The heroes the viewer sees now on a tile where it did not see heroes, by id: never its own.
    pub heroes: Vec<MapHero>,
    /// The objects still on the map it sees now and did not, in tile order ([`Object::place`]).
    pub objects: Vec<Object>,
    /// The empty boats it sees now on a tile where it did not see heroes, in tile order.
    pub boats: Vec<Boat>,
    /// The towns a tile of which it has explored now and none before (TWN-VEW-001), by id: their
    /// views are [`GameReveal::towns`]. Under `ADV-QUIRK-1` a town's object may have shown on the
    /// fringe before, so it is here and not in [`Self::objects`].
    pub towns: Vec<u16>,
}

/// What a tile whose sight changed is now ([`AdventureReveal::palette`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RevealedView {
    /// Whether the viewer has explored it now; a tile not explored whose view is not
    /// [`Tile::Hidden`] is on its fringe (ADV-FOG-005).
    pub explored: bool,
    /// What the viewer sees of it now, as [`Adventure::tile`] shows it.
    pub view: Tile,
}

/// The tiles of one level whose sight changed ([`AdventureReveal::areas`]): the smallest
/// rectangle holding them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RevealedArea {
    /// Its top left tile, on its level.
    pub corner: Pos,
    /// Its columns.
    pub width: u16,
    /// Its tiles row by row from the corner, as many rows as `width` divides them into: 0 a tile
    /// whose sight did not change, `n` one whose view is [`AdventureReveal::palette`]'s `n`-th
    /// (from 1).
    pub tiles: Vec<u32>,
}

/// What one player may know of a game (`h3_game::observe`, ADR-0017 points 1 and 2;
/// `docs/06-security.md`, "Information hiding"): the public facts, its own treasury, timed events
/// and open decision, the heroes it sees with its own heroes' details, the adventure map, the
/// towns, the map garrisons and its battle as it sees them. The seed, the streams and the state hash never show
/// (ADR-0017 point 5).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GameObservation {
    /// The player this observation is for.
    pub viewer: u8,
    /// The day, from 1.
    pub day: u32,
    /// The month's kind (GAM-CAL-003): public, as the original's month's message.
    pub month: Month,
    /// The week's kind (TWN-GRO-002): public, as the original's week's message.
    pub week: Week,
    /// The players, by id.
    pub players: Vec<Player>,
    /// The players of the group whose turn is running.
    pub turn_group: Vec<u8>,
    /// The viewer's treasury, by resource handle (GAM-RES-001); empty for a viewer that is not a
    /// player of the game.
    pub treasury: Vec<u32>,
    /// The map's timed events that fired at the viewer's turn start today, by their messages in
    /// the map's order (GAM-CAL-004); empty before its turn starts. Other players' never show.
    pub events_today: Vec<u16>,
    /// The viewer's first open decision but a battle (GAM-DEC-001 to GAM-DEC-008); other
    /// players' never show. While it is open the viewer's only legal commands are its answers,
    /// and in a meeting or an army dialog the management commands on what it holds.
    pub decision: Option<Decision>,
    /// The heroes the viewer sees on the adventure map, by id: each hero's type, and the details
    /// of its own only (GAM-HER-001).
    pub heroes: Vec<HeroInfo>,
    /// The viewer's list of heroes (GAM-HER-003; protocol v81, E08 T141a): its heroes on the map in
    /// the order they joined it, its garrison heroes not among them.
    pub hero_list: Vec<u16>,
    /// The adventure map as the viewer sees it.
    pub adventure: Adventure,
    /// The towns the viewer sees, by town id (TWN-VEW-001 to -003): each town a tile of which
    /// it has explored, its object among [`Self::adventure`]'s ([`Object::town`]).
    pub towns: Vec<Town>,
    /// The map garrisons the viewer sees, in their places' order ([`MapGarrison::place`];
    /// ADV-GAR-006, protocol v54): each garrison a tile of which it has explored, its object among
    /// [`Self::adventure`]'s by type and anchor.
    pub garrisons: Vec<MapGarrison>,
    /// The viewer's two tavern heroes by slot (TWN-TAV-002; protocol v70, E09 T37a): each as the
    /// tavern's held hero screen shows it, none for an empty slot. Another player's never show.
    pub tavern: [Option<TavernHero>; 2],
    /// The battle of the viewer's first open decision, as its side sees it (GAM-BAT-001); other
    /// players' battles never show.
    pub battle: Option<super::Observation>,
}

/// A month's kind (GAM-CAL-003): drawn on each month's first day after the first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Month {
    /// A normal month.
    Normal {},
    /// The month of a creature.
    Creature {
        /// The creature's handle.
        creature: u16,
    },
    /// The Plague.
    Plague {},
}

/// A week's kind (TWN-GRO-002): drawn on each week's first day after the first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Week {
    /// A normal week.
    Normal {},
    /// The week of a creature: its dwellings grow more.
    Creature {
        /// The creature's handle.
        creature: u16,
    },
    /// A building's week (the Deity of Fire's): the dwellings of either of its level's creatures
    /// grow more.
    Of {
        /// The level's creature's handle and its upgrade's, if any.
        creatures: [Option<u16>; 2],
        /// What each such dwelling grows more.
        bonus: u16,
    },
}

/// A player as every viewer sees it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Player {
    /// Its colour, 0 red to 7 pink.
    pub colour: u8,
    /// Its turn is running: its group's turn has started and it has not ended its turn.
    pub acting: bool,
}

/// An open decision as its owner sees it; a battle is [`GameObservation::battle`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Decision {
    /// An offer (GAM-DEC-001), answered yes or no.
    Offer {
        /// The offer.
        offer: Offer,
    },
    /// A level-up with two offers (GAM-DEC-002).
    LevelUp {
        /// The level-up.
        level_up: LevelUp,
    },
    /// A meeting of two of the viewer's heroes (GAM-DEC-003), closed by the viewer; meanwhile it
    /// manages both, also between them.
    Meeting {
        /// The visitor, then the visited hero.
        heroes: [u16; 2],
    },
    /// An army dialog (GAM-DEC-004), closed by the viewer, which loses the group's rest;
    /// meanwhile it manages the hero and the group, also between them. Also an own mine's Leave
    /// Guards dialog (ADV-MIN-002): its group the mine's guards, which the close keeps there.
    ArmyDialog {
        /// The hero the group joined, or the visiting hero on the mine's entrance.
        hero: u16,
        /// The group's stacks, by slot.
        group: Vec<ArmyStack>,
        /// A Leave Guards dialog's mine; none for a join's group.
        mine: Option<ObjectRef>,
    },
    /// A visit's choice (GAM-DEC-005): a treasure chest's, the Arena's or a School's.
    Choice {
        /// The choice.
        choice: Choice,
    },
    /// A University (GAM-DEC-006): its skills with their price, and each slot's state for the
    /// hero (an open one may be bought while the treasury can pay).
    University {
        /// The University.
        visit: UniversityVisit,
        /// Each skill's slot state for the hero, in the skills' order.
        slots: Vec<SlotState>,
    },
    /// A hire dialog (GAM-DEC-008): its army, its source and the source's creatures.
    Hire {
        /// The dialog.
        hire: Hire,
    },
    /// A map garrison's exchange (GAM-DEC-009, E08 T114a2), closed by the viewer; meanwhile it
    /// manages the hero's army and the garrison's, also between them, by the one owner rule.
    Exchange {
        /// The visiting hero, on the garrison's entrance.
        hero: u16,
        /// The garrison's object index, which the army commands name ([`ArmyRef::MapGarrison`]).
        garrison: u32,
        /// The garrison by type and anchor.
        object: ObjectRef,
        /// Whether the viewer may take its troops back (its removable flag).
        removable: bool,
        /// The garrison's stacks, by slot.
        army: Vec<ArmyStack>,
    },
    /// A map Tavern (GAM-DEC-010, E09 T39a, v73), answered with a hire of one of the viewer's
    /// tavern heroes (the observation's `tavern`, its offers) or its closing.
    Tavern {
        /// The visiting hero.
        hero: u16,
        /// The Tavern by type and anchor.
        object: ObjectRef,
    },
    /// A market (GAM-DEC-011, ADV-MKT-008; E08 T133b, v75): what its window offers with the
    /// rates, answered with a trade, a transfer, a sale, a purchase or leaving it.
    Market {
        /// The window.
        market: MarketWindow,
    },
    /// An Altar of Sacrifice (GAM-DEC-012, ADV-ALT-001 to -003; E08 T134a, v76): what its window
    /// offers with the experience, answered with a sacrifice or leaving it.
    Altar {
        /// The window.
        altar: AltarWindow,
    },
    /// A Hill Fort (GAM-DEC-013, ADV-HFT-001, -002; E08 T134b1, v77): each stack's row with its
    /// upgrade, price and state, and the whole's, answered with a row's upgrade, all, or leaving
    /// it.
    HillFort {
        /// The window.
        fort: HillFortWindow,
    },
}

/// A Hill Fort's decision as its owner sees it (`h3_game::HillFortView`; E08 T134b1, v77): the
/// hero, the Hill Fort by type and anchor, each stack's row with its state against the owner's
/// treasury, the whole's price and state (the server computes them, ADR-0004).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HillFortWindow {
    /// The visiting hero.
    pub hero: u16,
    /// The Hill Fort by type and anchor.
    pub object: ObjectRef,
    /// Each stack of the hero's army in its order.
    pub rows: Vec<HillFortRow>,
    /// The whole's price: the sum of the rows' that upgrade.
    pub all: Vec<Amount>,
    /// The whole's state.
    pub all_state: HillFortState,
}

/// A row of a Hill Fort's window (`h3_game::HillFortRowView`; E08 T134b1, v77).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HillFortRow {
    /// The hero's army's slot.
    pub slot: u8,
    /// The stack's creature's handle.
    pub creature: u16,
    /// The stack's count.
    pub count: u32,
    /// The creature's upgrade's handle, none without one.
    pub upgrade: Option<u16>,
    /// The upgrade's price for the stack (none for a free one).
    pub price: Vec<Amount>,
    /// Its state.
    pub state: HillFortState,
}

/// A Hill Fort's row's or whole's state (`h3_game::HillFortState`; E08 T134b1, v77): the
/// original's 0, 1 and 2.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum HillFortState {
    /// No upgrade.
    NoUpgrade {},
    /// The treasury pays the price.
    Affordable {},
    /// The treasury does not pay it.
    TooDear {},
}

/// An Altar of Sacrifice's decision as its owner sees it (`h3_game::AltarView`; E08 T134a, v76):
/// the hero, the Altar by type and anchor, the screens its hero's class's faction allows, the most
/// places, each artifact's and each stack's experience the server's answers give, the experience
/// to the hero's next level.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AltarWindow {
    /// The visiting hero.
    pub hero: u16,
    /// The Altar by type and anchor.
    pub object: ObjectRef,
    /// Whether the hero sacrifices artifacts.
    pub artifacts: bool,
    /// Whether the hero sacrifices creatures.
    pub creatures: bool,
    /// The most artifacts a sacrifice takes.
    pub places: u8,
    /// With the artifacts' screen, each artifact of the hero: the worn ones by position, then the
    /// backpack's.
    pub items: Vec<AltarItem>,
    /// With the creatures' screen, each stack of the hero's army.
    pub stacks: Vec<AltarStack>,
    /// The experience the hero lacks for its next level.
    pub to_next_level: u32,
}

/// An artifact in an Altar's window (`h3_game::AltarArtifact`; E08 T134a, v76).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AltarItem {
    /// Where the hero holds it.
    pub place: ArtifactPlace,
    /// The artifact's handle.
    pub artifact: u16,
    /// Its experience with Learning, none for one the Altar never takes.
    pub experience: Option<u32>,
}

/// A stack in an Altar's window (`h3_game::AltarStack`; E08 T134a, v76).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AltarStack {
    /// The hero's army's slot.
    pub slot: u8,
    /// A creature's experience before Learning.
    pub each: u32,
    /// The most a sacrifice takes of it.
    pub most: u32,
    /// The experience of the most with Learning.
    pub experience: u32,
}

/// A market's decision as its owner sees it (`h3_game::MarketView`; ADV-MKT-008, E08 T133b, v75):
/// the hero, the market by type and anchor, its screens and count, and for each screen it shows
/// the rates the server's answers use; a Black Market's stock to its visitor alone.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MarketWindow {
    /// The window's hero: a map market's visitor, a town's visitor or garrison hero, none possible
    /// at a town (E09 T58a1, v85).
    pub hero: Option<u16>,
    /// The market's place: a map market by type and anchor, or a town (E09 T58a1, v85).
    pub place: MarketSeen,
    /// Its screens, in its kind's order.
    pub screens: Vec<MarketScreen>,
    /// The Marketplaces it trades as.
    pub count: u8,
    /// Each pair of resources' lot, the given then the wanted in the core's order.
    pub trades: Vec<TradeLot>,
    /// Each stack of the hero's army, its lot per resource.
    pub sales: Vec<SaleLots>,
    /// The Black Market's stock, each entry's artifact (none once bought) and price per resource.
    pub stock: Vec<StockEntry>,
    /// A town's artifact sale: each place of the hero's that sells, its artifact and price per
    /// resource (E09 T58b, v86).
    pub artifact_sales: Vec<ArtifactSale>,
    /// The transfer's recipients: every other player of the game.
    pub recipients: Vec<u8>,
}

/// An artifact's sale in a town's window (`h3_game::ArtifactSaleOffer`; E09 T58b, v86).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArtifactSale {
    /// Where the hero holds it.
    pub place: ArtifactPlace,
    /// The artifact's handle.
    pub artifact: u16,
    /// Its price per resource.
    pub prices: Vec<Amount>,
}

/// A market's place (`h3_game::MarketSeen`; E09 T58a1, v85): a market on the map by type and
/// anchor, or a town's Marketplace, Artifact Merchants or Freelancer's Guild by the town.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MarketSeen {
    /// A market on the map.
    Object {
        /// The market by type and anchor.
        object: ObjectRef,
    },
    /// A town.
    Town {
        /// The town.
        town: u16,
    },
}

/// A market's screen (`h3_game::MarketScreen`; E08 T133b, v75).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MarketScreen {
    /// Resources for resources.
    Trade,
    /// Resources to another player, 1 : 1.
    Transfer,
    /// Creatures for resources.
    Sell,
    /// Resources for an artifact of the stock.
    Buy,
    /// The hero's artifact for resources (a town's with the Artifact Merchants; E09 T58b, v86).
    SellArtifact,
}

/// A rate (`h3_game::Lot`; E08 T133b, v75): so many given for so many wanted, one of them 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lot {
    /// How many of the given one lot takes.
    pub give: u32,
    /// How many of the wanted one lot gives.
    pub take: u32,
}

/// A trade's lot (`h3_game::TradeOffer`; E08 T133b, v75).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TradeLot {
    /// The resource given.
    pub give: u16,
    /// The resource wanted.
    pub take: u16,
    /// The lot.
    pub lot: Lot,
}

/// A stack's lots (`h3_game::SaleOffer`; E08 T133b, v75).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SaleLots {
    /// The hero's army's slot.
    pub slot: u8,
    /// The lot per resource it sells for.
    pub lots: Vec<ResourceLot>,
}

/// A lot in a resource (E08 T133b, v75).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceLot {
    /// The resource's handle.
    pub resource: u16,
    /// The lot: creatures given, units gained.
    pub lot: Lot,
}

/// A Black Market's entry (`h3_game::StockEntry`; E08 T133b, v75).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StockEntry {
    /// The artifact's handle, none once bought.
    pub artifact: Option<u16>,
    /// Its price per resource.
    pub prices: Vec<Amount>,
}

/// An offer a visit makes to its visitor's owner, answered yes or no (ADR-0017 point 6), its
/// object named by type and anchor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Offer {
    /// The hero whose visit made it.
    pub hero: u16,
    /// The object's type handle.
    pub object_type: u16,
    /// The object's anchor.
    pub anchor: Pos,
    /// What is on offer.
    pub kind: OfferKind,
}

/// What an offer offers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OfferKind {
    /// A Cartographer's reveal of its type's terrains, for its price (ADV-VIS-001).
    Reveal {
        /// The price, by resource.
        price: Vec<Amount>,
    },
    /// A border guard's passage (ADV-VIS-004).
    Pass {},
    /// A wandering monster's offer to join for free (ADV-MON-008).
    Join {},
    /// A wandering monster's offer to join for a price (ADV-MON-008).
    JoinForGold {
        /// The price, by resource.
        price: Vec<Amount>,
    },
    /// A wandering monster's offer to flee (ADV-MON-008): accepted, the hero pursues it.
    Pursue {},
    /// A guarded pile's, artifact's, scroll's or mine's question (ADV-PIK-004, ADV-PIK-007,
    /// ADV-PIK-008, ADV-MIN-002): accepted, the hero fights its guards.
    Fight {},
    /// A once-per-hero gift's price (ADV-TCH-003: the Tree of Knowledge's).
    Gift {
        /// The price, by resource.
        price: Vec<Amount>,
    },
    /// A find's question (ADV-PIK-011: the Warrior's Tomb's "search?").
    Search {},
    /// Pandora's Box's question (ADV-PIK-013).
    OpenBox {},
    /// The Pyramid's question (ADV-TCH-008).
    SearchPyramid {},
    /// A Shipyard's boat for its price (ADV-SHY-002).
    Boat {
        /// The price, by resource.
        price: Vec<Amount>,
    },
    /// The hire question of a dwelling or a Refugee Camp (ADV-DWL-004, ADV-RFG-001,
    /// ADV-HIR-001).
    Hire {
        /// The creatures it names: the dwelling's, in its kind's order; a camp's week's one.
        creatures: Vec<u16>,
    },
    /// A dwelling's guards' question (ADV-DWL-004): accepted, the hero fights them.
    Guarded {
        /// The creature of the guards' first occupied slot.
        creature: u16,
        /// The word of the guards' total count.
        count_word: u8,
    },
}

/// A level-up with two offers as its owner sees it (GAM-DEC-002): what the original's dialog
/// shows. The hero's level seed never shows (ADR-0017 point 5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LevelUp {
    /// The hero.
    pub hero: u16,
    /// The new level.
    pub level: u8,
    /// The primary skill it raises by 1.
    pub primary: PrimarySkill,
    /// The offered skills' handles, the dialog's left and right.
    pub offers: [u16; 2],
}

/// A primary skill.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrimarySkill {
    /// Attack.
    Attack,
    /// Defense.
    Defense,
    /// Spell power.
    Power,
    /// Knowledge.
    Knowledge,
}

/// A choice a visit opens for its hero's owner (ADR-0017 point 6), its object named by type and
/// anchor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Choice {
    /// The hero whose visit opened it.
    pub hero: u16,
    /// The object's type handle.
    pub object_type: u16,
    /// The object's anchor.
    pub anchor: Pos,
    /// What is chosen between.
    pub kind: ChoiceKind,
}

/// What a [`Choice`] chooses between.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChoiceKind {
    /// A treasure chest's (ADV-PIK-005): first its gold to the player, second the experience to
    /// the hero.
    Chest {
        /// The gold: its resource and amount.
        gold: Amount,
        /// The experience.
        experience: u32,
    },
    /// A once-per-hero gift's (ADV-TCH-001: the Arena; ADV-TCH-002: the Schools): `amount`
    /// added to the first or the second skill for the price.
    Primary {
        /// The two skills, in the dialog's order.
        skills: [PrimarySkill; 2],
        /// The gain.
        amount: u8,
        /// The price, by resource (empty: free).
        price: Vec<Amount>,
        /// The player may leave the choice, taking neither.
        leave: bool,
    },
}

/// A University a visit opens (ADV-TCH-007): its skills in the draw's order, each sold at the
/// price.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UniversityVisit {
    /// The hero whose visit opened it.
    pub hero: u16,
    /// The object's type handle.
    pub object_type: u16,
    /// The object's anchor.
    pub anchor: Pos,
    /// The skills' handles.
    pub skills: Vec<u16>,
    /// The price of each, by resource.
    pub price: Vec<Amount>,
}

/// The state of a University's slot for a hero.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SlotState {
    /// The hero knows the skill.
    Known,
    /// The hero knows the most skills (eight).
    NoRoom,
    /// The hero's class weighs the skill 0 at a level-up.
    Barred,
    /// The hero may buy the skill: it is learned at Basic.
    Open,
}

/// A hire dialog as its owner sees it (GAM-DEC-008): its source as [`HireSeen`] names it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Hire {
    /// The army the bought creatures join.
    pub target: HireTarget,
    /// The source.
    pub source: HireSeen,
    /// The source's creatures in its order.
    pub creatures: Vec<HireCreature>,
}

/// The army a hire dialog's creatures join.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HireTarget {
    /// The army of the hero on the source's entrance.
    Hero {
        /// The hero.
        hero: u16,
    },
    /// A town's garrison (E09 T27, TWN-REC-001).
    Garrison {
        /// The town.
        town: u16,
    },
}

/// A hire dialog's source (`h3_game::HireSeen`; E09 T27): a map object by type and anchor, or a
/// town's creature index or Portal of Summoning by the town.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HireSeen {
    /// A dwelling or a Refugee Camp on the map.
    Object {
        /// The object's type handle.
        object_type: u16,
        /// The object's anchor.
        anchor: Pos,
    },
    /// A town's dwelling or horde: one of its fourteen creature indexes (`level - 1`, `+ 7` the
    /// upgrade).
    Town {
        /// The town.
        town: u16,
        /// The creature index.
        index: u8,
    },
    /// A town's Portal of Summoning (TWN-REC-002).
    Portal {
        /// The town.
        town: u16,
    },
}

/// A creature a hire dialog sells.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HireCreature {
    /// The creature's handle.
    pub creature: u16,
    /// How many the source holds now.
    pub count: u32,
    /// The cost of one, by resource (each resource it costs more than 0 of).
    pub cost: Vec<Amount>,
}

/// What a viewer knows of a hero beyond the adventure map's view of it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HeroInfo {
    /// The hero's id.
    pub id: u16,
    /// Its hero type's handle: identity and class, public as the original's portrait.
    pub hero_type: u16,
    /// Its details, for its owner only; since v88 also for the visitor's owner while its meeting
    /// with this hero, an ally's, is open (GAM-TEM-003).
    pub details: Option<HeroDetails>,
    /// Its army by slot, for the visitor's owner while its meeting with this hero, an ally's, is
    /// open (GAM-TEM-003, v88); none otherwise (an own hero's is [`MapHero::own`]'s).
    pub army: Option<Vec<ArmyStack>>,
}

/// A part of the meeting's ally's hero as the visitor's owner sees it while the meeting is open
/// (`h3_game::MeetingPart`; GAM-TEM-003, protocol v88): told after its `MeetingOpened`, in this
/// order, so that they fold into the hero's [`HeroInfo`] details and army whole; the meeting's
/// close ends that view.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MeetingPart {
    /// First, once: what no list holds ([`HeroDetails`]' level, experience, base primary skills,
    /// spell points, offered levels and how many worn positions).
    Head {
        /// Its level.
        level: u8,
        /// Its experience points.
        experience: u32,
        /// Its base primary skills: attack, defense, power, knowledge.
        primary: [u8; 4],
        /// Its spell points.
        mana: u16,
        /// The levels last offered.
        offered: OfferLevels,
        /// The positions' count.
        positions: u8,
    },
    /// A stack of its army, by slot.
    Stack {
        /// The stack.
        stack: ArmyStack,
    },
    /// A secondary skill with its level, in order.
    Skill {
        /// The skill and its level.
        skill: SkillLevel,
    },
    /// An artifact at its place: the worn ones by position, then the backpack's in order.
    Artifact {
        /// Where it is.
        place: ArtifactPlace,
        /// The artifact.
        artifact: Artifact,
    },
    /// A spell it has learned, by handle, in its details' order.
    Spell {
        /// The spell.
        spell: u16,
    },
}

/// The lasting details of a hero (ADR-0013 point 1), its base values: what artifacts and the
/// specialty add is derived.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HeroDetails {
    /// The hero type's handle.
    pub hero_type: u16,
    /// Its level, at least 1: the levels given so far.
    pub level: u8,
    /// Its experience points.
    pub experience: u32,
    /// The levels at which a level-up last offered Wisdom and a magic school (HER-LVL-004).
    pub offered: OfferLevels,
    /// Its base primary skills: attack, defense, power, knowledge.
    pub primary: [u8; 4],
    /// Its secondary skills with their levels, in display order.
    pub skills: Vec<SkillLevel>,
    /// Its worn artifacts by position, one entry per position of the ruleset's slots.
    pub worn: Vec<Option<Artifact>>,
    /// Its backpack's artifacts, in backpack order.
    pub backpack: Vec<Artifact>,
    /// The spells it has learned, by handle.
    pub spells: Vec<u16>,
    /// Its spell points.
    pub mana: u16,
}

/// The levels at which a level-up last offered Wisdom and a magic school (HER-LVL-004).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OfferLevels {
    /// The level at which Wisdom was last offered.
    pub wisdom: u8,
    /// The level at which a magic school was last offered.
    pub school: u8,
}

/// A secondary skill and its level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkillLevel {
    /// The skill's handle.
    pub skill: u16,
    /// Its level: 1 Basic to 3 Expert.
    pub level: u8,
}

/// An artifact a hero wears or carries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Artifact {
    /// The artifact type's handle.
    pub artifact: u16,
    /// The spell of a scroll; none for every other artifact (HER-ART-003).
    pub spell: Option<u16>,
}

/// What a tavern hero's owner sees of it without a list (`h3_game::TavernHero`; TWN-TAV-002,
/// protocol v70, E09 T37a): the tavern dialog's and the offer's held hero screen's parts. Its
/// learned spells, its offered levels and its points never show.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TavernHeroHead {
    /// The hero's id.
    pub id: u16,
    /// Its hero type's handle.
    pub hero_type: u16,
    /// Its level.
    pub level: u8,
    /// Its experience points.
    pub experience: u32,
    /// Its base primary skills: attack, defense, power, knowledge.
    pub primary: [u8; 4],
    /// Its spell points.
    pub mana: u16,
    /// How many artifacts its backpack carries (its screen shows the first five).
    pub carried: u16,
    /// Its army's formation.
    pub formation: Formation,
    /// Its tactics flag.
    pub tactics: bool,
}

/// A part of a tavern hero's view that is one of a list (`h3_game::TavernPart`; protocol v70):
/// the army's stacks by slot, the secondary skills in order, the worn artifacts by position, then
/// the backpack's first five places.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TavernPart {
    /// A stack of its army.
    Stack {
        /// The stack.
        stack: ArmyStack,
    },
    /// A secondary skill with its level.
    Skill {
        /// The skill and its level.
        skill: SkillLevel,
    },
    /// An artifact at its place.
    Artifact {
        /// Where it is: a position, or one of the backpack's first places.
        place: ArtifactPlace,
        /// The artifact.
        artifact: Artifact,
    },
}

/// A part of a hired hero's details its tavern slot's view did not show (`h3_game::HiredPart`;
/// TWN-TAV-003, protocol v71): told to its owner after its `HeroHired`, in this order, so that the
/// slot's view folded with them is the hero's details whole.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HiredPart {
    /// First, once: the levels at which a level-up last offered Wisdom and a magic school, and
    /// how many positions its worn artifacts have ([`HeroDetails::worn`]'s length).
    Details {
        /// The levels last offered.
        offered: OfferLevels,
        /// The positions' count.
        positions: u8,
    },
    /// An artifact of its backpack past the view's first five places, in backpack order.
    Carried {
        /// The artifact.
        artifact: Artifact,
    },
    /// A spell it has learned, by handle, in its details' order.
    Spell {
        /// The spell.
        spell: u16,
    },
}

/// A tavern hero as its owner sees it (`h3_game::TavernHeroView`; TWN-TAV-002, protocol v70): its
/// parts without a list, then its lists in [`TavernPart`]'s order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TavernHero {
    /// Its parts without a list.
    pub hero: TavernHeroHead,
    /// Its army, by slot.
    pub army: Vec<ArmyStack>,
    /// Its secondary skills with their levels, in order.
    pub skills: Vec<SkillLevel>,
    /// Its worn artifacts by position, then the backpack's first five, each with its place.
    pub artifacts: Vec<(ArtifactPlace, Artifact)>,
}

/// A row of the Thieves' Guild (`h3_game::GuildRow`; TWN-TAV-008, protocol v74), in the original's
/// order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GuildRow {
    /// The towns the player owns.
    Towns,
    /// Its heroes on the map.
    Heroes,
    /// Its resources of the town part's first guild row (the original's gold).
    Gold,
    /// Those of the second (the original's wood and ore).
    Common,
    /// Those of the third (the original's mercury, sulfur, crystal and gems).
    Rare,
    /// The obelisks it visited.
    Obelisks,
    /// Its heroes' artifacts on the map.
    Artifacts,
    /// Its army strength.
    Army,
    /// Its gold of a day's start as it stands now.
    Income,
}

/// A row as the guild shows it (`h3_game::GuildRanking`; protocol v74): its kind and each
/// player's place by player id, 0 the first, equal values sharing one place.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GuildRanking {
    /// The row.
    pub row: GuildRow,
    /// Each player's place, by player id.
    pub places: Vec<u8>,
}

/// A player's best hero as the guild shows it (`h3_game::GuildHero`; TWN-TAV-009, protocol v74).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GuildHero {
    /// Its type, by handle.
    pub hero_type: u16,
    /// From level 2 its attack, defense, power and knowledge, each 0 to 99.
    pub skills: Option<[u8; 4]>,
}

/// What the guild shows of one player beside the rows (`h3_game::GuildPlayer`; TWN-TAV-009,
/// protocol v74).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GuildPlayer {
    /// From level 1 its best hero.
    pub hero: Option<GuildHero>,
    /// From level 4 its best creature, by handle.
    pub creature: Option<u16>,
}

/// The Thieves' Guild as its viewer sees it (`h3_game::ThievesGuildView`; TWN-TAV-007 to -009,
/// protocol v74): the answer to the guild's `GameQuery`, or what a Den of Thieves'
/// `ThievesGuildSeen` and its parts tell ([`Self::see`]). Never in the observation: a snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ThievesGuild {
    /// Its level, 0 to 5.
    pub level: u8,
    /// The rows the level shows, in [`GuildRow`]'s order.
    pub rows: Vec<GuildRanking>,
    /// Every player of the game, by id.
    pub players: Vec<GuildPlayer>,
}

/// A part of the Thieves' Guild a Den of Thieves' visit tells (`h3_game::GuildPart`; protocol
/// v74): after its `ThievesGuildSeen`, every shown row's places (the rows in order, each row's
/// players by id), then every player's parts by id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GuildPart {
    /// A player's place in a row.
    Place {
        /// The row.
        row: GuildRow,
        /// The player placed.
        of: u8,
        /// Its place, 0 the first.
        place: u8,
    },
    /// What the guild shows of a player beside the rows.
    Player {
        /// The player.
        of: u8,
        /// Its best hero and best creature.
        shown: GuildPlayer,
    },
}

impl ThievesGuild {
    /// The guild of `level` before its parts (a `ThievesGuildSeen`): no row, no player.
    #[must_use]
    pub const fn at(level: u8) -> Self {
        Self {
            level,
            rows: Vec::new(),
            players: Vec::new(),
        }
    }

    /// Adds a part after the ones before it (`GuildPartSeen`): a place to its row, a place of a
    /// row other than the last one's beginning that row; a player's parts to the players. From
    /// [`Self::at`] of the told level, the parts in order give the told guild.
    pub fn see(&mut self, part: GuildPart) {
        match part {
            GuildPart::Place { row, of: _, place } => match self.rows.last_mut() {
                Some(ranking) if ranking.row == row => ranking.places.push(place),
                _ => self.rows.push(GuildRanking {
                    row,
                    places: vec![place],
                }),
            },
            GuildPart::Player { of: _, shown } => self.players.push(shown),
        }
    }
}

/// The bonuses a hero on the map holds until the end of its next battle (HER-ADV-008; v57, E08
/// T115a: the hero on the map's, `h3_adventure::HeroBonuses`, no longer its details').
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bonuses {
    /// The morale no once-mark names, added to its side in its next battle.
    pub morale: i8,
    /// The luck no once-mark names, added to its side in its next battle.
    pub luck: i8,
    /// The once-marks with the morale and luck each visit gave, sorted by mark, unique.
    pub once: Vec<VisitBonus>,
}

/// A visit's battle bonus under its once-mark (`h3_heroes::VisitBonus`; v57).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VisitBonus {
    /// The once-mark (the adventure's content names it).
    pub mark: u8,
    /// The morale the visit gave.
    pub morale: i8,
    /// The luck the visit gave.
    pub luck: i8,
}

/// A map garrison as a viewer sees it (ADV-GAR-006; protocol v54, E08 T114b): its type and
/// anchor (its object among the view's), its owner, its army at the viewer's level (the levels of
/// a town's garrison, TWN-VEW-002, -003) and, for its owner's team, its removable flag.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MapGarrison {
    /// Its object type's handle.
    pub object_type: u16,
    /// Its object's anchor.
    pub anchor: Pos,
    /// Its owner (its flag); none for a neutral garrison.
    pub owner: Option<u8>,
    /// Its army as the viewer's level shows it.
    pub garrison: Garrison,
    /// Whether its owner may take its troops back, for the owner's team only; none for every
    /// other viewer.
    pub removable: Option<bool>,
}

impl MapGarrison {
    /// Its place among a view's map garrisons ([`GameObservation::garrisons`]): its anchor's
    /// level, row and column, then its type (no two garrisons of one type share an anchor).
    pub const fn place(&self) -> (u8, u16, u16, u16) {
        (
            self.anchor.level,
            self.anchor.y,
            self.anchor.x,
            self.object_type,
        )
    }
}

/// A town as a viewer sees it (TWN-VEW-001 to -004): what the original shows another player on
/// the map, in the hint and in the right-click popup, and its owner's team more; since protocol
/// v39 (E09 T18b2a) its day's mark and its owner's buildings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Town {
    /// The town's id.
    pub town: u16,
    /// Its object's anchor.
    pub anchor: Pos,
    /// Its owner (its flag); none for a neutral town.
    pub owner: Option<u8>,
    /// Its faction's handle.
    pub faction: u16,
    /// Its name as the map gives it; none for a town the map leaves unnamed, whose name is the
    /// client's.
    pub name: Option<String>,
    /// Its fort level: 0 none, 1 Fort, 2 Citadel, 3 Castle.
    pub fort: u8,
    /// It has a Capitol.
    pub capitol: bool,
    /// It was built in today (public).
    pub built_today: bool,
    /// Its garrison as the viewer's level shows it.
    pub garrison: Garrison,
    /// The popup's top level's facts, for the owner's team or a hero's Visions in reach; none
    /// for every other viewer.
    pub owner_facts: Option<TownOwnerFacts>,
    /// Its buildings, for its owner alone; none for every other viewer.
    pub buildings: Option<TownBuildings>,
}

/// A town's buildings as its owner sees them (TWN-VEW-004), each set a bit per building of its
/// faction's town record (bit `i` the building of index `i`, a decimal string as every 64-bit
/// value): the standing ones derive from the first with the town part. Since v55 (E09 T26b) its
/// available creatures and its Portal of Summoning, the town screen's; since v78 (E09 T47b) its
/// mage guild's shown spells, the Aurora's mark and the Mana Vortex's week flag, the guild
/// window's.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TownBuildings {
    /// Every building it has had, the superseded ones too.
    pub built: WireU64,
    /// The buildings the map forbids it.
    pub forbidden: WireU64,
    /// Its available creatures at each of its fourteen indexes: the seven levels' dwellings, then
    /// their upgrades (`h3_game::CREATURE_INDEXES`).
    pub available: [u16; h3_game::CREATURE_INDEXES],
    /// What its Portal of Summoning holds; none while it holds nothing (or stands not).
    pub portal: Option<Summoned>,
    /// What its mage guild shows (TWN-GLD-003), by spell level 1 to 5: the spells' handles in
    /// slot order; none above the guild level, never a slot the guild does not show (the spare
    /// without a Library).
    pub guild: [Vec<u16>; 5],
    /// It has ever built an Aurora Borealis (TWN-GLD-004): every spell is taught there.
    pub aurora: bool,
    /// Its Mana Vortex's week flag (TWN-GLD-009), kept in every town; it matters only where a
    /// Mana Vortex stands.
    pub vortex: bool,
}

/// What a Portal of Summoning holds (TWN-GRO-007): a creature of one of its owner's dwellings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Summoned {
    /// The creature's handle.
    pub creature: u16,
    /// How many.
    pub count: u16,
}

/// A town's garrison as a viewer sees it (TWN-VEW-002, -003): by the viewer's level, its stacks
/// in slot order (never their slots) from level 1, each stack's size word from level 2, its
/// count at level 3; nothing at level 0.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Garrison {
    /// The viewer's level, 0 to 3.
    pub level: u8,
    /// The stacks as the level shows them, in slot order; none at level 0.
    pub stacks: Vec<GarrisonStack>,
    /// The garrison by slot, for a viewer of the owner's team only.
    pub army: Option<Vec<ArmyStack>>,
}

/// A garrison's stack as a viewer's level shows it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GarrisonStack {
    /// The creature's handle (from level 1).
    pub creature: u16,
    /// The word of its count (from level 2).
    pub count_word: Option<u8>,
    /// Its count (at level 3).
    pub count: Option<u32>,
}

/// The facts of a town the popup shows only at its top level (TWN-VEW-003).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TownOwnerFacts {
    /// Its hall level, 1 the Village Hall to 4 the Capitol.
    pub hall: u8,
    /// Its day's income, by resource.
    pub income: Vec<Amount>,
    /// Whether a hero stands in its garrison (TWN-VEW-001, the popup's mark; E09 T28b2, v67):
    /// the garrison's stacks are then that hero's army.
    pub garrison_hero: bool,
}
