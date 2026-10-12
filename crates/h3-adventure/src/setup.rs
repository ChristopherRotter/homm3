//! The adventure context's setup (ADR-0015 point 4, E08 T5a): plain owned data an adventure state
//! is built and validated from, as a battle is from `BattleSetup`. A map is imported into it
//! (`h3-map`) through `h3-game`'s whole-game setup; tests build it in code. A setup satisfies
//! every invariant commands keep, checked by the same functions the commands use (one check per
//! invariant), so a map cannot start a state that play could not reach (`docs/06-security.md`,
//! "Untrusted files").

use alloc::vec::Vec;
use core::fmt;

use h3_content::TerrainId;
use h3_contracts::{ArmyStack, HeroId, PlayerId};

use crate::ruleset::{ObjectTypeId, RiverId, RoadId};

/// A tile's coordinates: the column and row from the top left, and the level (0 the surface,
/// 1 the underground).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Pos {
    /// Column.
    pub x: u16,
    /// Row.
    pub y: u16,
    /// Level.
    pub level: u8,
}

impl Pos {
    /// The tile at column `x`, row `y` of level `level`.
    pub const fn new(x: u16, y: u16, level: u8) -> Self {
        Self { x, y, level }
    }
}

/// What a tile holds that never changes after setup.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TileSetup {
    /// The terrain (a core terrain; how it is crossed is the adventure part's).
    pub terrain: TerrainId,
    /// The road on the tile, if any.
    pub road: Option<RoadId>,
    /// The river on the tile, if any.
    pub river: Option<RiverId>,
}

/// A set of tiles relative to an object's anchor, its bottom right tile (ADV-OBJ-001): the bit
/// `8 × row + column` stands for the tile `column` columns left of and `row` rows above the
/// anchor, in a block of [`TileMask::COLUMNS`] by [`TileMask::ROWS`] tiles, the original's
/// object template (the bit is the mask's character in `Objects.txt`, anchor first).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TileMask(u64);

impl TileMask {
    /// Columns of the block.
    pub const COLUMNS: u8 = 8;
    /// Rows of the block.
    pub const ROWS: u8 = 6;
    /// The bits a mask may hold.
    const ALL: u64 = (1 << (Self::COLUMNS as u32 * Self::ROWS as u32)) - 1;

    /// The mask of these bits; bits beyond the block are dropped.
    pub const fn from_bits(bits: u64) -> Self {
        Self(bits & Self::ALL)
    }

    /// The bits.
    pub const fn bits(self) -> u64 {
        self.0
    }

    /// Whether the tile `column` columns left of and `row` rows above the anchor is in the mask.
    pub const fn contains(self, column: u8, row: u8) -> bool {
        column < Self::COLUMNS
            && row < Self::ROWS
            && self.0 >> (row as u32 * Self::COLUMNS as u32 + column as u32) & 1 == 1
    }

    /// The tiles of the mask as `(column, row)` offsets from the anchor, row by row.
    pub fn offsets(self) -> impl Iterator<Item = (u8, u8)> {
        (0..Self::ROWS)
            .flat_map(|row| (0..Self::COLUMNS).map(move |column| (column, row)))
            .filter(move |&(column, row)| self.contains(column, row))
    }
}

/// An object on the map as the setup places it. Its anchor may lie past the map's right or bottom
/// edge (the map editor lets objects stick out); the tiles of its footprint off the map are
/// ignored (ADV-OBJ-001).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObjectSetup {
    /// The object's bottom right tile, which its masks are relative to.
    pub anchor: Pos,
    /// The tiles no hero enters (ADV-OBJ-001).
    pub blocked: TileMask,
    /// The tiles a hero visits it from (ADV-OBJ-001).
    pub visitable: TileMask,
    /// Its object type (ADV-OBJ-002); none for a blocker, which blocks its blocked and its
    /// visitable tiles and is never visited (ADR-0016 point 4).
    pub object_type: Option<ObjectTypeId>,
    /// The amount the map fixes for a pickup whose type takes one
    /// ([`crate::ObjectType::takes_amount`]), before the type's multiplier: from 1 to
    /// [`crate::MAX_PICKUP_AMOUNT`]. None draws it when the game is built.
    pub amount: Option<u32>,
}

/// The static map as the setup gives it: what never changes after setup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapSetup {
    /// Columns.
    pub width: u16,
    /// Rows.
    pub height: u16,
    /// Levels (1, or 2 with an underground).
    pub levels: u8,
    /// Every tile, level by level, row by row from the top, column by column from the left.
    pub tiles: Vec<TileSetup>,
    /// The objects, in the map's order.
    pub objects: Vec<ObjectSetup>,
}

/// A hero on the map as the setup places it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeroSetup {
    /// The hero (its index in `h3-game`'s store of heroes).
    pub id: HeroId,
    /// The player owning it.
    pub owner: PlayerId,
    /// The tile it stands on.
    pub pos: Pos,
    /// Its army: one or more stacks in distinct slots.
    pub army: Vec<ArmyStack>,
}

/// Everything the adventure context is built from (ADR-0015 point 4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdventureSetup {
    /// How many players the game has; players are `PlayerId` 0 to `players - 1`.
    pub players: u8,
    /// The static map.
    pub map: MapSetup,
    /// The heroes on the map, in any order (ids must be distinct).
    pub heroes: Vec<HeroSetup>,
}

/// Why a hero may not stand on a tile (ADV-MAP-001).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StandError {
    /// The tile is outside the map.
    Outside,
    /// The tile's terrain is not walked: water (without a boat) or rock.
    Terrain,
    /// An object blocks the tile and no object is visited from it.
    Blocked,
    /// Another hero stands there.
    Occupied,
}

impl fmt::Display for StandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Outside => "the tile is outside the map",
            Self::Terrain => "the terrain is not walked on land",
            Self::Blocked => "an object blocks the tile",
            Self::Occupied => "another hero stands there",
        })
    }
}

impl core::error::Error for StandError {}

/// Why a hero's army is invalid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArmyError {
    /// The army has no stacks.
    Empty,
    /// A slot is beyond the ruleset's army slots.
    SlotOutOfRange(u8),
    /// Two stacks share a slot.
    DuplicateSlot(u8),
    /// A stack has no creatures.
    EmptyStack(u8),
    /// A stack's creature type is not part of the ruleset.
    UnknownCreature(u8),
}

impl fmt::Display for ArmyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("the army has no stacks"),
            Self::SlotOutOfRange(slot) => write!(f, "slot {slot} is out of range"),
            Self::DuplicateSlot(slot) => write!(f, "slot {slot} is used twice"),
            Self::EmptyStack(slot) => write!(f, "slot {slot} has no creatures"),
            Self::UnknownCreature(slot) => write!(f, "slot {slot} has an unknown creature type"),
        }
    }
}

impl core::error::Error for ArmyError {}

/// Why an [`AdventureSetup`] cannot start a game.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetupError {
    /// The map's width, height or levels are 0 or beyond the ruleset's bounds.
    MapSize {
        /// Columns.
        width: u16,
        /// Rows.
        height: u16,
        /// Levels.
        levels: u8,
    },
    /// The tile list does not hold one entry per tile; the number of entries.
    TileCount(usize),
    /// The tile at this index has a terrain the ruleset does not know.
    UnknownTerrain(usize),
    /// The tile at this index has a road the ruleset does not know.
    UnknownRoad(usize),
    /// The tile at this index has a river the ruleset does not know.
    UnknownRiver(usize),
    /// No players, or more than the ruleset allows.
    Players(u8),
    /// The object at this index is anchored on a level the map does not have.
    ObjectLevel(usize),
    /// The object at this index has an object type the ruleset does not know.
    UnknownObjectType(usize),
    /// The object at this index has an amount its type does not take, or one outside 1 to
    /// [`crate::MAX_PICKUP_AMOUNT`].
    ObjectAmount(usize),
    /// Two heroes share this id.
    DuplicateHero(HeroId),
    /// The hero's owner is not a player of the game.
    UnknownOwner(HeroId),
    /// The hero stands where no hero may (ADV-MAP-001).
    HeroTile {
        /// The hero.
        hero: HeroId,
        /// Why.
        error: StandError,
    },
    /// The hero's army is invalid.
    Army {
        /// The hero.
        hero: HeroId,
        /// Why.
        error: ArmyError,
    },
}

impl fmt::Display for SetupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MapSize {
                width,
                height,
                levels,
            } => write!(
                f,
                "a map of {width} × {height} tiles on {levels} levels is outside the bounds"
            ),
            Self::TileCount(found) => write!(f, "{found} tiles do not fill the map"),
            Self::UnknownTerrain(tile) => write!(f, "tile {tile} has an unknown terrain"),
            Self::UnknownRoad(tile) => write!(f, "tile {tile} has an unknown road"),
            Self::UnknownRiver(tile) => write!(f, "tile {tile} has an unknown river"),
            Self::Players(n) => write!(f, "{n} players are outside the bounds"),
            Self::ObjectLevel(object) => {
                write!(f, "object {object} is anchored on a missing level")
            }
            Self::UnknownObjectType(object) => {
                write!(f, "object {object} has an unknown object type")
            }
            Self::ObjectAmount(object) => {
                write!(f, "object {object} has an amount its type does not take")
            }
            Self::DuplicateHero(hero) => write!(f, "hero {} is placed twice", hero.raw()),
            Self::UnknownOwner(hero) => {
                write!(f, "hero {}'s owner is not a player", hero.raw())
            }
            Self::HeroTile { hero, error } => write!(f, "hero {}: {error}", hero.raw()),
            Self::Army { hero, error } => write!(f, "hero {}'s army: {error}", hero.raw()),
        }
    }
}

impl core::error::Error for SetupError {}
