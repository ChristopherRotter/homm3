//! The whole game's published language (E16 M5 part 1, SRV-Q25; ADR-0018 points 5 and 7 as
//! amended): what a player sees of a game beyond its battle. M5 T2a: the adventure map
//! ([`Adventure`]); T2b the rest of the observation.
//!
//! The same rules as [`super`]'s: the protocol's own types, translated by [`crate::map::game`]
//! with every engine field named; serde's default enum form with `deny_unknown_fields`; content
//! by the ruleset's dense handles; a 64-bit value as a [`WireU64`]. A whole map's tiles travel as
//! a palette and an index per tile ([`Adventure::tiles`]): a tile's view is a few fields, of which
//! a map holds a few dozen distinct values, and one packed number cannot hold every handle the
//! engine allows below JavaScript's 2^53.
//!
//! Provisional (E16 M5 part 1's review): the shapes, the palette.

use alloc::vec::Vec;

use serde::{Deserialize, Serialize};

use super::WireU64;

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
/// boats it sees, and what it knows of objects.
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
    /// The objects still on the map that it sees, in the map's order.
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
