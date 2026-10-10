//! The raw model of a map file: what the file says, in the file's own codes, faithful to its
//! format version (RoE, AB, SoD) and without ruleset handles (ADR-0015 point 4). Codes (hero
//! types, artifacts, creatures, terrains, …) stay numbers here; the import maps them onto content
//! keys. Layout notes: `docs/formats/h3m.md`.

use core::fmt;

/// Format version of a map file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Version {
    /// The Restoration of Erathia, `0x0E`.
    RoE,
    /// Armageddon's Blade, `0x15`.
    Ab,
    /// The Shadow of Death, `0x1C`.
    Sod,
}

impl Version {
    /// The version's code at the start of a map file.
    pub const fn code(self) -> u32 {
        match self {
            Self::RoE => 0x0E,
            Self::Ab => 0x15,
            Self::Sod => 0x1C,
        }
    }

    /// The version with this code, if it is one the reader supports.
    pub const fn from_code(code: u32) -> Option<Self> {
        match code {
            0x0E => Some(Self::RoE),
            0x15 => Some(Self::Ab),
            0x1C => Some(Self::Sod),
            _ => None,
        }
    }

    /// Hero types in the format's tables (RoE 128, AB and SoD 156).
    pub const fn hero_types(self) -> usize {
        match self {
            Self::RoE => 128,
            Self::Ab | Self::Sod => 156,
        }
    }

    /// Artifacts in the format's tables (RoE 127, AB 129, SoD 144).
    pub const fn artifacts(self) -> usize {
        match self {
            Self::RoE => 127,
            Self::Ab => 129,
            Self::Sod => 144,
        }
    }

    /// A hero's artifact slots (SoD adds the fifth miscellaneous slot).
    pub const fn artifact_slots(self) -> usize {
        match self {
            Self::RoE | Self::Ab => 18,
            Self::Sod => 19,
        }
    }

    /// Bytes of a faction bit set (AB adds the Conflux).
    pub const fn faction_bytes(self) -> usize {
        match self {
            Self::RoE => 1,
            Self::Ab | Self::Sod => 2,
        }
    }

    /// Bytes of an artifact or creature code (one in RoE, two from AB on).
    pub const fn id_bytes(self) -> usize {
        match self {
            Self::RoE => 1,
            Self::Ab | Self::Sod => 2,
        }
    }
}

/// A string as stored: its bytes, Windows-1252 in the game's own maps (other language editions
/// use other code pages, so the reader does not decode it).
#[derive(Clone, Default, PartialEq, Eq, Hash)]
pub struct Text(pub Vec<u8>);

impl Text {
    /// The stored bytes.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl fmt::Debug for Text {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&String::from_utf8_lossy(&self.0), f)
    }
}

/// A bit set as stored: bit `i % 8` of byte `i / 8` is element `i`. What a set bit means (allowed
/// or banned) is said where the set is used.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct Bits(pub Vec<u8>);

impl Bits {
    /// Whether element `i` is set (false beyond the stored bytes).
    pub fn get(&self, i: usize) -> bool {
        self.0.get(i / 8).is_some_and(|b| b & (1 << (i % 8)) != 0)
    }

    /// The stored bytes.
    pub fn bytes(&self) -> &[u8] {
        &self.0
    }
}

/// A tile position: column, row and level (0 surface, 1 underground).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Pos {
    /// Column.
    pub x: u8,
    /// Row.
    pub y: u8,
    /// Level.
    pub z: u8,
}

/// A map file, read up to the end of its terrain (E08 T3a; templates, objects and timed events
/// follow in T3b, ADV-STUB-1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawMap {
    /// The header: version, size, name, difficulty.
    pub header: Header,
    /// The eight player colours, red first.
    pub players: [Player; 8],
    /// The victory condition.
    pub victory: Victory,
    /// The loss condition.
    pub loss: Loss,
    /// Alliances, when the map defines teams.
    pub teams: Option<Teams>,
    /// Hero types available to the taverns and the random pool (bit set = allowed); 16 bytes in
    /// RoE, 20 from AB on.
    pub allowed_heroes: Bits,
    /// Hero types reserved for campaign placeholders (AB and SoD).
    pub placeholders: Vec<u8>,
    /// Heroes restricted to some players (SoD).
    pub disposed_heroes: Vec<DisposedHero>,
    /// Artifacts the map bans (bit set = banned); none in RoE, 17 bytes in AB, 18 in SoD.
    pub banned_artifacts: Bits,
    /// Spells the map bans (bit set = banned; SoD, 9 bytes).
    pub banned_spells: Bits,
    /// Secondary skills the map bans (bit set = banned; SoD, 4 bytes).
    pub banned_skills: Bits,
    /// Tavern rumors.
    pub rumors: Vec<Rumor>,
    /// Hero types the map customises for every appearance (SoD), in hero type order.
    pub custom_heroes: Vec<CustomHero>,
    /// The tiles of every level.
    pub terrain: Terrain,
}

/// The map file's header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header {
    /// Format version.
    pub version: Version,
    /// The "any players" flag (false only for maps nobody can play).
    pub any_players: bool,
    /// Side length in tiles (maps are square).
    pub size: u32,
    /// Whether the map has an underground level.
    pub two_levels: bool,
    /// Map name.
    pub name: Text,
    /// Map description.
    pub description: Text,
    /// Difficulty code (0 easy … 4 impossible).
    pub difficulty: u8,
    /// Hero level limit (AB and SoD; 0 = none, and always 0 in RoE).
    pub level_limit: u8,
}

impl Header {
    /// Number of levels (1 or 2).
    pub const fn levels(&self) -> usize {
        if self.two_levels { 2 } else { 1 }
    }
}

/// One player colour.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Player {
    /// A human may play it.
    pub human: bool,
    /// The computer may play it.
    pub computer: bool,
    /// What the map says about the player; `None` when nobody may play it (the file then holds
    /// a fixed run of unused bytes).
    pub details: Option<PlayerDetails>,
}

/// A playable colour's settings.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PlayerDetails {
    /// Computer behaviour code (VCMI 1.7.5: −1 none, 0 random, 1 warrior, 2 builder, 3 explorer).
    pub ai_tactic: i8,
    /// SoD's byte before the town set (the editor's "town choice offered"; 0 before SoD).
    pub town_choice: u8,
    /// Factions the player may start with (bit per faction; 8 bits in RoE, 9 from AB on).
    pub allowed_towns: u16,
    /// The player's town is chosen at random.
    pub random_town: bool,
    /// The player's main town, if the map names one.
    pub main_town: Option<MainTown>,
    /// The player's starting hero is random.
    pub random_hero: bool,
    /// The player's main hero, if the map customises one.
    pub main_hero: Option<MainHero>,
    /// AB's and SoD's unexplained byte before the hero list.
    pub unknown: u8,
    /// The player's heroes placed on the map, with their names (AB and SoD).
    pub heroes: Vec<PlayerHero>,
}

/// A player's main town.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MainTown {
    /// Whether the game generates a hero at the town at the start (always in RoE).
    pub generate_hero: bool,
    /// AB's and SoD's town type byte (unused by the game; 0 in RoE).
    pub town_type: u8,
    /// The town's position.
    pub pos: Pos,
}

/// A player's main hero as the header customises it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MainHero {
    /// Hero type code.
    pub hero_type: u8,
    /// Portrait code (`0xFF` = the hero type's own).
    pub portrait: u8,
    /// Name (empty = the hero type's own).
    pub name: Text,
}

/// A hero the header lists for a player.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PlayerHero {
    /// Hero type code.
    pub hero_type: u8,
    /// Name.
    pub name: Text,
}

/// The map's victory condition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Victory {
    /// Defeat all enemies.
    Standard,
    /// A special goal.
    Special {
        /// The standard victory also counts.
        allow_standard: bool,
        /// The goal applies to computer players too.
        applies_to_ai: bool,
        /// The goal.
        goal: VictoryGoal,
    },
}

/// A special victory goal (codes 0–10 of the format).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VictoryGoal {
    /// 0: acquire an artifact (code: one byte in RoE, two from AB on).
    AcquireArtifact {
        /// Artifact code.
        artifact: u16,
    },
    /// 1: accumulate creatures.
    AccumulateCreatures {
        /// Creature code (one byte in RoE, two from AB on).
        creature: u16,
        /// How many.
        count: i32,
    },
    /// 2: accumulate a resource.
    AccumulateResources {
        /// Resource code.
        resource: u8,
        /// How much.
        amount: i32,
    },
    /// 3: upgrade a town.
    UpgradeTown {
        /// The town (`0xFF` coordinates = any).
        pos: Pos,
        /// Hall level code (0 town hall … 3 capitol).
        hall: u8,
        /// Castle level code (0 fort … 2 castle).
        castle: u8,
    },
    /// 4: build the grail structure.
    BuildGrail {
        /// The town (`0xFF` coordinates = any).
        pos: Pos,
    },
    /// 5: defeat a hero.
    DefeatHero {
        /// The hero's position.
        pos: Pos,
    },
    /// 6: capture a town.
    CaptureTown {
        /// The town's position.
        pos: Pos,
    },
    /// 7: defeat a monster.
    DefeatMonster {
        /// The monster's position.
        pos: Pos,
    },
    /// 8: flag all creature dwellings.
    FlagDwellings,
    /// 9: flag all mines.
    FlagMines,
    /// 10: transport an artifact to a town.
    TransportArtifact {
        /// Artifact code (one byte in every version).
        artifact: u8,
        /// The town's position.
        pos: Pos,
    },
}

/// The map's loss condition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Loss {
    /// Lose all towns and heroes.
    Standard,
    /// 0: lose a town.
    LoseTown {
        /// The town's position.
        pos: Pos,
    },
    /// 1: lose a hero.
    LoseHero {
        /// The hero's position.
        pos: Pos,
    },
    /// 2: the time runs out.
    TimeExpires {
        /// Days.
        days: u16,
    },
}

/// The map's alliances.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Teams {
    /// Number of teams (at least 1).
    pub count: u8,
    /// Each colour's team code.
    pub of_player: [u8; 8],
}

/// A hero the map restricts to some players (SoD).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DisposedHero {
    /// Hero type code.
    pub hero_type: u8,
    /// Portrait code.
    pub portrait: u8,
    /// Name.
    pub name: Text,
    /// Players who may hire the hero (bit per colour).
    pub players: u8,
}

/// A tavern rumor.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Rumor {
    /// The editor's name for it.
    pub name: Text,
    /// Its text.
    pub text: Text,
}

/// A hero type the map customises (SoD's hero settings).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CustomHero {
    /// Hero type code.
    pub hero_type: u8,
    /// Experience, if customised.
    pub experience: Option<u32>,
    /// Secondary skills (skill code, level code), if customised.
    pub secondary_skills: Option<Vec<(u8, u8)>>,
    /// Artifacts, if customised.
    pub artifacts: Option<Artifacts>,
    /// Biography, if customised.
    pub biography: Option<Text>,
    /// Sex code (`0xFF` default, 0 male, 1 female).
    pub sex: u8,
    /// Spells (bit set = known; 9 bytes), if customised.
    pub spells: Option<Bits>,
    /// Attack, defence, spell power, knowledge, if customised.
    pub primary_skills: Option<[u8; 4]>,
}

/// A hero's artifacts as stored: one code per slot (`None` = empty), then the backpack.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Artifacts {
    /// The worn slots in the format's order (18 before SoD, 19 in SoD).
    pub worn: Vec<Option<u16>>,
    /// The backpack, in order.
    pub backpack: Vec<Option<u16>>,
}

/// The tiles of every level, level by level, row by row.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Terrain {
    /// Side length in tiles.
    pub size: usize,
    /// Number of levels.
    pub levels: usize,
    /// `size × size × levels` tiles; tile (x, y, z) is at `(z × size + y) × size + x`.
    pub tiles: Vec<Tile>,
}

impl Terrain {
    /// The tile at `pos`, if it is on the map.
    pub fn tile(&self, pos: Pos) -> Option<&Tile> {
        let (x, y, z) = (usize::from(pos.x), usize::from(pos.y), usize::from(pos.z));
        if x < self.size && y < self.size && z < self.levels {
            self.tiles.get((z * self.size + y) * self.size + x)
        } else {
            None
        }
    }
}

/// One tile as stored (7 bytes).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Tile {
    /// Terrain code (0 dirt … 9 rock).
    pub terrain: u8,
    /// Terrain picture.
    pub view: u8,
    /// River code (0 none, 1 clear, 2 icy, 3 muddy, 4 lava).
    pub river: u8,
    /// River picture.
    pub river_dir: u8,
    /// Road code (0 none, 1 dirt, 2 gravel, 3 cobblestone).
    pub road: u8,
    /// Road picture.
    pub road_dir: u8,
    /// Mirroring and coast flags.
    pub flags: u8,
}
