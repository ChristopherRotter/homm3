//! Adapter: parses the packs' `adventure.toml` files (one pack file per context, ADR-0006's E08
//! amendment) into the adventure context's definitions, which `h3-adventure` compiles into its
//! ruleset part (ADR-0015 point 2). The files are hand-written source the harvester never touches
//! (E08 decision 10); parsing is strict like `h3-pack`'s (unknown fields are errors, numbers must
//! fit their types), and the compiler validates everything again.
//!
//! # Format (E08 T4)
//!
//! ```toml
//! [params]                         # exactly one enabled pack provides them
//! native_cost = 100                # a step on terrain native to the whole army (ADV-MOV-004)
//! land_points = [1300, 1360, 1430] # a day's points on land by the slowest speed, from speed 0;
//!                                  # faster armies take the last entry (ADV-MOV-001)
//! scouting_radius = 5              # without bonuses (ADV-FOG-001)
//! max_map_size = 144               # setup bounds (E08 T5a): the widest and highest map,
//! max_levels = 2                   # its levels and the players of a game
//! max_players = 8
//! army_slots = 7                   # the slots of a hero's army
//! max_stack_count = 1000000        # E08 T50: a stack's most creatures (ADV-ARM-007), at least
//!                                  # MAX_MONSTER_COUNT
//! starting_stack_chances = [88, 25] # percent for the 2nd, 3rd... starting slot of a hero's
//!                                  # starting army; the 1st always (GAM-SET-001, E08 T6b)
//! fly_cost = [140, 140, 120, 100]  # E08 T40: a flying hero's straight step by its flying level
//!                                  # 0 to 3 (ADV-MOV-009)
//! water_walk_cost = [140, 140, 120, 100] # a water walker's step from water by its level
//!                                  # (ADV-MOV-010)
//! sea_points = [1500, 2250, 3000, 3750] # E08 T72: a day's points at sea by Navigation level
//!                                  # 0 to 3 (HER-ADV-009), none 0
//! sea_ground = [2, 3]              # a boat's step price from a tile a ground of the sea tops,
//!                                  # times 2 / 3 truncated (ADV-SEA-004); 1 <= first <= second
//!
//! [params.battle]                  # optional (E08 T19a, GAM-BAT-001): a battle on the map is
//! underground = "subterranean"     # fought on this terrain underground, else on a coast tile
//! coast = "desert"                 # on this one with this battlefield (a key of the battle
//! coast_battlefield = "shore"      # part), else on the tile's terrain; E08 T72: with a hero
//! sea = "water"                    # in a boat on this terrain, on the deck with one, on the
//! deck = "deck"                    # boats with both (battle part keys)
//! boats = "boats"
//!
//! [params.monsters]                # optional (E08 T20a2a): without it a map holds no monster
//! aggression = [[-4, -4], [1, 7]]  # bounds by disposition code, from 0 (ADV-MON-002)
//! count_words = [5, 10, 20]        # where each word of a count after the first begins
//!                                  # (ADV-FOG-006)
//! price_resource = "gold"          # a group's price to join: its creature's cost in this
//!                                  # resource times its count (ADV-MON-008, E08 T20b1)
//!
//! not_random_monsters = ["azure_dragon"] # top-level, before any table (E08 T20a2a): creatures
//!                                  # no random monster becomes (ADV-MON-003)
//!
//! [[terrain]]                      # every core terrain needs one entry, from any pack of
//! terrain = "grass"                # its closure; `name` means `<own pack>:name`
//! cost = 100                       # a straight step (ADV-MOV-003)
//! # passage = "land"               # land (default) or water need a cost; impassable has none
//! # pathfinding = [125, 100, 100]  # E08 T29, land only: the cost with Pathfinding at Basic,
//!                                  # Advanced and Expert, none above the one below; left out:
//!                                  # the cost at every level
//!
//! [[road]]
//! id = "dirt_road"
//! name = "Dirt road"
//! cost = 75                        # a straight step along it (ADV-MOV-003)
//!
//! [[river]]                        # appearance only
//! id = "clear_river"
//! name = "Clear river"
//!
//! [[terrain_walker]]               # E08 T40 (ADV-MOV-003): an army with a stack of the creature
//! creature = "nomad"               # counts the terrain as the other for every price of a step
//! terrain = "base:desert"          # from it; both land terrains, one walker per terrain
//! counts_as = "base:dirt"
//!
//! [[object_type]]                  # E08 T12a; ADR-0016 point 4, ADV-OBJ-002
//! id = "pile_gold"
//! name = "Gold"
//! visit = "neighbour"              # or "enter"
//! from_above = true                # visitable from the three tiles above
//! # exactly one kind table; resources, terrains and types: `name` means `<own pack>:name`
//! pickup = { resource = "gold", amount = [5, 10], multiplier = 100 } # multiplier: 1 if left out
//! # pickup_one_of = ["pile_wood", "pile_ore"]                       # a type drawn at setup
//! # pickup_paired = { amount = [4, 6], resources = ["wood"], paired = "gold", multiplier = 100 }
//! # reveal_terrain = { terrains = ["water"], price = { gold = 1000 } }
//! # reveal_area = { radius = 20 }
//! # key_tent = { key = 0 }         # also key_guard, key_gate (from_above = true); keys 0-7
//! # monster = {}                   # E08 T20a2a, visit = "neighbour": the map names the creature,
//! # monster = { levels = [1, 7] } # or a random monster's levels (ADV-MON-001, ADV-MON-003)
//! # ground = { battlefield = "cursed_ground" } # a special ground (ADV-GRD-001); with
//! #                                # `no_native = true` (E08 T40) a step from a tile it tops
//! #                                # costs no army the native price (ADV-MOV-004)
//! # E08 T61a, increment 7's kinds (classes: "treasure", "minor", "major", "relic", "special";
//! # primary skills: "attack", "defense", "power", "knowledge"; prices and gifts as `price`):
//! # chest = { resource = "gold", gold = [[32, 1000]], artifact = ["treasure"],
//! #           full_backpack = 1000, experience_offset = 500 } # [chance, amount] pairs
//! # artifact = {}                  # the map names it; or artifact = { classes = ["relic"] }
//! # scroll = {}                    # also scholar, pandora_box, event: no parameters
//! # gift = { gives = { primary = [0, 1, 0, 0] } } # or { choice = { skills = ["attack",
//! #                                # "defense"], amount = 2 } }, { experience = 1000 }, "level";
//! #                                # prices = [{}, { gold = 2000 }] (one drawn), admits = 10,
//! #                                # leave = true
//! # shrine = { level = 1 }
//! # witch_hut = { roe = ["archery"], roe_fixed = ["archery", "first_aid"] } # skills
//! # university = { offers = 4, price = { gold = 2000 } }
//! # find = { draws = [{ chance = 20, holds = { artifact = ["minor"] } },
//! #          { chance = 80, holds = "nothing" }], full_backpack = { give = { gold = 1000 } } }
//! #                                # holds: "nothing", { artifact = [...] }, { resources =
//! #                                # { amount = [1, 5], resources = ["wood"] } }, E08 T72
//! #                                # { fixed = { resources = { gold = 1000 }, artifact =
//! #                                # ["treasure"] } } (the artifact optional); full_backpack:
//! #                                # { give = {...} }, "drawn", "stays", "lost"; question = true;
//! #                                # bonus = { morale = -3, luck = 0, mark = 0 }; leaves = true
//! # message = { leaves = true }    # E08 T72: the Ocean Bottle (leaves: false if left out)
//! # sea_ground = {}                # E08 T72: also hole = {}, no parameters
//! # pyramid = { split = { creature = "gold_golem", count = 40 }, fixed = { creature =
//! #             "diamond_golem", stacks = [10, 10] }, level = 5, looted = { luck = -2, mark = 1 } }
//!
//! [[difficulty]]                   # E08 T13a, GAM-RES-002: the starting resources by the game's
//!                                  # difficulty, from 0 (easy); exactly one enabled pack gives
//!                                  # them; resources as above, a resource left out starts at 0
//! human = { wood = 30, gold = 30000 }
//! computer = { wood = 5, gold = 5000 }
//! ```
//!
//! Costs are at least 1. A pack without the file has an empty adventure part; every enabled pack
//! has one, so the part shares the core's load order and dependency closures.

use std::fmt;
use std::num::{NonZeroU8, NonZeroU16};

use std::collections::BTreeMap;
use std::num::NonZeroU32;

use h3_adventure::{
    AdventureCompileError, AdventurePack, AdventureParamsDef, AdventureRuleset, BattleTerrainsDef,
    ChestDef, FindContentDef, FindDef, GiftDef, Gives, MonsterParamsDef, ObjectKindDef,
    ObjectTypeDef, OnceBonus, Passage, PickupDef, PrimarySkill, PyramidDef, RiverDef, RoadDef,
    StartingResourcesDef, TerrainMovementDef, TerrainWalkerDef, TurnDef, Visit,
};
use h3_content::{ArtifactClass, ContentKey, KeyError, PackManifest, Ruleset};
use h3_pack::PackError;
use serde::Deserialize;

/// The synthetic pack's adventure file (tests).
pub const SYNTHETIC_ADVENTURE: &str = include_str!("../../../packs/synthetic/adventure.toml");
/// The synthetic expansion's adventure file (tests).
pub const SYNTHETIC_EXT_ADVENTURE: &str =
    include_str!("../../../packs/synthetic-ext/adventure.toml");
/// The Restoration of Erathia's adventure file.
pub const BASE_ADVENTURE: &str = include_str!("../../../packs/base/adventure.toml");
/// Armageddon's Blade's adventure file.
pub const AB_ADVENTURE: &str = include_str!("../../../packs/ab/adventure.toml");
/// The Shadow of Death's adventure file.
pub const SOD_ADVENTURE: &str = include_str!("../../../packs/sod/adventure.toml");

/// Errors of this adapter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdventurePackError {
    /// The text is not valid TOML for the expected schema.
    Syntax(String),
    /// A key or id is malformed.
    Key(KeyError),
    /// A terrain entry's cost does not fit its passage (land and water need one, impassable
    /// has none); the payload is the terrain as written.
    TerrainCost(String),
    /// An object type does not have exactly one kind table; the payload is its id as written.
    ObjectKind(String),
    /// A core pack does not parse, or the core packs do not compile.
    Core(PackError),
    /// The adventure packs do not compile into the adventure part.
    Compile(AdventureCompileError),
}

impl fmt::Display for AdventurePackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Syntax(e) => write!(f, "syntax: {e}"),
            Self::Key(e) => write!(f, "{e}"),
            Self::TerrainCost(t) => write!(
                f,
                "terrain `{t}`: land and water need a cost, an impassable terrain has none"
            ),
            Self::ObjectKind(id) => {
                write!(f, "object type `{id}` needs exactly one kind table")
            }
            Self::Core(e) => write!(f, "core packs: {e}"),
            Self::Compile(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for AdventurePackError {}

impl From<KeyError> for AdventurePackError {
    fn from(e: KeyError) -> Self {
        Self::Key(e)
    }
}

impl From<PackError> for AdventurePackError {
    fn from(e: PackError) -> Self {
        Self::Core(e)
    }
}

impl From<AdventureCompileError> for AdventurePackError {
    fn from(e: AdventureCompileError) -> Self {
        Self::Compile(e)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AdventureFile {
    #[serde(default)]
    not_random_monsters: Vec<String>,
    params: Option<ParamsDto>,
    #[serde(default)]
    terrain: Vec<TerrainDto>,
    #[serde(default)]
    road: Vec<RoadDto>,
    #[serde(default)]
    river: Vec<RiverDto>,
    #[serde(default)]
    object_type: Vec<ObjectTypeDto>,
    #[serde(default)]
    difficulty: Vec<DifficultyDto>,
    #[serde(default)]
    terrain_walker: Vec<TerrainWalkerDto>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TerrainWalkerDto {
    creature: String,
    terrain: String,
    counts_as: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DifficultyDto {
    human: BTreeMap<String, u32>,
    computer: BTreeMap<String, u32>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ParamsDto {
    native_cost: NonZeroU16,
    land_points: Vec<u32>,
    scouting_radius: u8,
    max_map_size: NonZeroU16,
    max_levels: NonZeroU8,
    max_players: NonZeroU8,
    army_slots: NonZeroU8,
    max_stack_count: NonZeroU32,
    starting_stack_chances: Vec<u8>,
    #[serde(default)]
    battle: Option<BattleDto>,
    #[serde(default)]
    monsters: Option<MonstersDto>,
    fly_cost: [NonZeroU16; 4],
    water_walk_cost: [NonZeroU16; 4],
    sea_points: [u32; 4],
    sea_ground: [u16; 2],
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MonstersDto {
    aggression: Vec<[i8; 2]>,
    count_words: Vec<u32>,
    price_resource: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BattleDto {
    underground: String,
    coast: String,
    coast_battlefield: String,
    sea: String,
    deck: String,
    boats: String,
}

#[derive(Deserialize, Default, Clone, Copy)]
#[serde(rename_all = "snake_case")]
enum PassageDto {
    #[default]
    Land,
    Water,
    Impassable,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TerrainDto {
    terrain: String,
    #[serde(default)]
    passage: PassageDto,
    cost: Option<NonZeroU16>,
    pathfinding: Option<[NonZeroU16; 3]>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RoadDto {
    id: String,
    name: String,
    cost: NonZeroU16,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RiverDto {
    id: String,
    name: String,
}

#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
enum VisitDto {
    Enter,
    Neighbour,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ObjectTypeDto {
    id: String,
    name: String,
    visit: VisitDto,
    from_above: bool,
    pickup: Option<PickupDto>,
    pickup_one_of: Option<Vec<String>>,
    pickup_paired: Option<PairedDto>,
    reveal_terrain: Option<RevealTerrainDto>,
    reveal_area: Option<RevealAreaDto>,
    key_tent: Option<KeyDto>,
    key_guard: Option<KeyDto>,
    key_gate: Option<KeyDto>,
    monster: Option<MonsterDto>,
    ground: Option<GroundDto>,
    chest: Option<ChestDto>,
    artifact: Option<ArtifactDto>,
    scroll: Option<EmptyDto>,
    gift: Option<GiftDto>,
    shrine: Option<ShrineDto>,
    witch_hut: Option<WitchHutDto>,
    scholar: Option<EmptyDto>,
    university: Option<UniversityDto>,
    find: Option<FindDto>,
    pandora_box: Option<EmptyDto>,
    event: Option<EmptyDto>,
    pyramid: Option<PyramidDto>,
    message: Option<MessageDto>,
    sea_ground: Option<EmptyDto>,
    hole: Option<EmptyDto>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MessageDto {
    #[serde(default)]
    leaves: bool,
}

/// A kind table without parameters (`scroll = {}`).
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EmptyDto {}

#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
enum ClassDto {
    Treasure,
    Minor,
    Major,
    Relic,
    Special,
}

impl From<ClassDto> for ArtifactClass {
    fn from(class: ClassDto) -> Self {
        match class {
            ClassDto::Treasure => Self::Treasure,
            ClassDto::Minor => Self::Minor,
            ClassDto::Major => Self::Major,
            ClassDto::Relic => Self::Relic,
            ClassDto::Special => Self::Special,
        }
    }
}

fn classes(list: Vec<ClassDto>) -> Vec<ArtifactClass> {
    list.into_iter().map(ArtifactClass::from).collect()
}

#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
enum PrimaryDto {
    Attack,
    Defense,
    Power,
    Knowledge,
}

impl From<PrimaryDto> for PrimarySkill {
    fn from(skill: PrimaryDto) -> Self {
        match skill {
            PrimaryDto::Attack => Self::Attack,
            PrimaryDto::Defense => Self::Defense,
            PrimaryDto::Power => Self::Power,
            PrimaryDto::Knowledge => Self::Knowledge,
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ChestDto {
    resource: String,
    gold: Vec<(u8, u32)>,
    #[serde(default)]
    artifact: Vec<ClassDto>,
    full_backpack: u32,
    experience_offset: u32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ArtifactDto {
    classes: Option<Vec<ClassDto>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
enum GivesDto {
    Primary([u8; 4]),
    Choice { skills: [PrimaryDto; 2], amount: u8 },
    Experience(u32),
    Level,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GiftDto {
    gives: GivesDto,
    #[serde(default)]
    prices: Vec<BTreeMap<String, u32>>,
    admits: Option<u8>,
    #[serde(default)]
    leave: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ShrineDto {
    level: u8,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WitchHutDto {
    roe: Vec<String>,
    roe_fixed: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct UniversityDto {
    offers: u8,
    price: BTreeMap<String, u32>,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
enum HoldsDto {
    Nothing,
    Artifact(Vec<ClassDto>),
    Resources {
        amount: [u32; 2],
        resources: Vec<String>,
    },
    Fixed {
        resources: BTreeMap<String, u32>,
        artifact: Option<Vec<ClassDto>>,
    },
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DrawDto {
    chance: u8,
    holds: HoldsDto,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
enum TurnDto {
    Give(BTreeMap<String, u32>),
    Drawn,
    Stays,
    Lost,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BonusDto {
    #[serde(default)]
    morale: i8,
    #[serde(default)]
    luck: i8,
    mark: u8,
}

impl From<BonusDto> for OnceBonus {
    fn from(b: BonusDto) -> Self {
        Self {
            morale: b.morale,
            luck: b.luck,
            mark: b.mark,
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FindDto {
    draws: Vec<DrawDto>,
    full_backpack: Option<TurnDto>,
    #[serde(default)]
    question: bool,
    bonus: Option<BonusDto>,
    #[serde(default)]
    leaves: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GuardDto {
    creature: String,
    count: u32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StacksDto {
    creature: String,
    stacks: Vec<u32>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PyramidDto {
    split: GuardDto,
    fixed: StacksDto,
    level: u8,
    looted: BonusDto,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MonsterDto {
    levels: Option<[u8; 2]>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GroundDto {
    battlefield: String,
    #[serde(default)]
    no_native: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PickupDto {
    resource: String,
    amount: [u32; 2],
    multiplier: Option<NonZeroU32>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PairedDto {
    amount: [u32; 2],
    resources: Vec<String>,
    paired: String,
    multiplier: Option<NonZeroU32>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RevealTerrainDto {
    terrains: Vec<String>,
    #[serde(default)]
    price: BTreeMap<String, u32>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RevealAreaDto {
    radius: u8,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct KeyDto {
    key: u8,
}

/// An object type's definition: its one kind table with its references resolved.
fn object_type(pack: &str, t: ObjectTypeDto) -> Result<ObjectTypeDef, AdventurePackError> {
    let refs = |list: Vec<String>| {
        list.iter()
            .map(|r| reference(pack, r))
            .collect::<Result<Vec<_>, _>>()
    };
    let one = NonZeroU32::MIN;
    let mut kinds = Vec::new();
    if let Some(p) = t.pickup {
        kinds.push(ObjectKindDef::Pickup(PickupDef::Resource {
            resource: reference(pack, &p.resource)?,
            amount: p.amount,
            multiplier: p.multiplier.unwrap_or(one),
        }));
    }
    if let Some(list) = t.pickup_one_of {
        kinds.push(ObjectKindDef::Pickup(PickupDef::OneOf(refs(list)?)));
    }
    if let Some(p) = t.pickup_paired {
        kinds.push(ObjectKindDef::Pickup(PickupDef::Paired {
            amount: p.amount,
            resources: refs(p.resources)?,
            paired: reference(pack, &p.paired)?,
            multiplier: p.multiplier.unwrap_or(one),
        }));
    }
    if let Some(r) = t.reveal_terrain {
        kinds.push(ObjectKindDef::RevealTerrain {
            terrains: refs(r.terrains)?,
            price: r
                .price
                .iter()
                .map(|(k, &a)| Ok((reference(pack, k)?, a)))
                .collect::<Result<_, KeyError>>()?,
        });
    }
    kinds.extend(
        t.reveal_area
            .map(|r| ObjectKindDef::RevealArea { radius: r.radius }),
    );
    kinds.extend(t.key_tent.map(|k| ObjectKindDef::KeyTent { key: k.key }));
    kinds.extend(t.key_guard.map(|k| ObjectKindDef::KeyGuard { key: k.key }));
    kinds.extend(t.key_gate.map(|k| ObjectKindDef::KeyGate { key: k.key }));
    kinds.extend(
        t.monster
            .map(|m| ObjectKindDef::Monster { levels: m.levels }),
    );
    if let Some(g) = t.ground {
        kinds.push(ObjectKindDef::Ground {
            battlefield: reference(pack, &g.battlefield)?,
            no_native: g.no_native,
        });
    }
    let amounts = |map: BTreeMap<String, u32>| {
        map.iter()
            .map(|(k, &a)| Ok((reference(pack, k)?, a)))
            .collect::<Result<Vec<_>, KeyError>>()
    };
    if let Some(c) = t.chest {
        kinds.push(ObjectKindDef::Chest(ChestDef {
            resource: reference(pack, &c.resource)?,
            gold: c.gold,
            artifact: classes(c.artifact),
            full_backpack: c.full_backpack,
            experience_offset: c.experience_offset,
        }));
    }
    kinds.extend(t.artifact.map(|a| ObjectKindDef::Artifact {
        classes: a.classes.map(classes),
    }));
    kinds.extend(t.scroll.map(|_| ObjectKindDef::Scroll));
    if let Some(g) = t.gift {
        kinds.push(ObjectKindDef::Gift(GiftDef {
            gives: match g.gives {
                GivesDto::Primary(gains) => Gives::Primary(gains),
                GivesDto::Choice { skills, amount } => Gives::Choice {
                    skills: skills.map(PrimarySkill::from),
                    amount,
                },
                GivesDto::Experience(amount) => Gives::Experience(amount),
                GivesDto::Level => Gives::Level,
            },
            prices: g
                .prices
                .into_iter()
                .map(amounts)
                .collect::<Result<_, _>>()?,
            admits: g.admits,
            leave: g.leave,
        }));
    }
    kinds.extend(t.shrine.map(|s| ObjectKindDef::Shrine { level: s.level }));
    if let Some(w) = t.witch_hut {
        kinds.push(ObjectKindDef::WitchHut {
            roe: refs(w.roe)?,
            roe_fixed: refs(w.roe_fixed)?,
        });
    }
    kinds.extend(t.scholar.map(|_| ObjectKindDef::Scholar));
    if let Some(u) = t.university {
        kinds.push(ObjectKindDef::University {
            offers: u.offers,
            price: amounts(u.price)?,
        });
    }
    if let Some(f) = t.find {
        let mut draws = Vec::with_capacity(f.draws.len());
        for draw in f.draws {
            draws.push((
                draw.chance,
                match draw.holds {
                    HoldsDto::Nothing => FindContentDef::Nothing,
                    HoldsDto::Artifact(list) => FindContentDef::Artifact(classes(list)),
                    HoldsDto::Resources { amount, resources } => FindContentDef::Resources {
                        amount,
                        resources: refs(resources)?,
                    },
                    HoldsDto::Fixed {
                        resources,
                        artifact,
                    } => FindContentDef::Fixed {
                        resources: amounts(resources)?,
                        artifact: artifact.map(classes),
                    },
                },
            ));
        }
        kinds.push(ObjectKindDef::Find(FindDef {
            draws,
            full_backpack: match f.full_backpack {
                None => None,
                Some(TurnDto::Give(map)) => Some(TurnDef::Give(amounts(map)?)),
                Some(TurnDto::Drawn) => Some(TurnDef::Drawn),
                Some(TurnDto::Stays) => Some(TurnDef::Stays),
                Some(TurnDto::Lost) => Some(TurnDef::Lost),
            },
            question: f.question,
            bonus: f.bonus.map(OnceBonus::from),
            leaves: f.leaves,
        }));
    }
    kinds.extend(t.pandora_box.map(|_| ObjectKindDef::PandoraBox));
    kinds.extend(t.event.map(|_| ObjectKindDef::Event));
    if let Some(p) = t.pyramid {
        kinds.push(ObjectKindDef::Pyramid(PyramidDef {
            split: (reference(pack, &p.split.creature)?, p.split.count),
            fixed: (reference(pack, &p.fixed.creature)?, p.fixed.stacks),
            level: p.level,
            looted: OnceBonus::from(p.looted),
        }));
    }
    kinds.extend(
        t.message
            .map(|m| ObjectKindDef::Message { leaves: m.leaves }),
    );
    kinds.extend(t.sea_ground.map(|_| ObjectKindDef::SeaGround));
    kinds.extend(t.hole.map(|_| ObjectKindDef::Hole));
    let [kind] = <[ObjectKindDef; 1]>::try_from(kinds)
        .map_err(|_| AdventurePackError::ObjectKind(t.id.clone()))?;
    Ok(ObjectTypeDef {
        key: ContentKey::new(pack, &t.id)?,
        name: t.name,
        visit: match t.visit {
            VisitDto::Enter => Visit::Enter,
            VisitDto::Neighbour => Visit::Neighbour,
        },
        from_above: t.from_above,
        kind,
    })
}

/// Resolves a reference: `name` means `<own pack>:name`, `pack:name` is taken as written.
fn reference(pack: &str, text: &str) -> Result<ContentKey, KeyError> {
    if text.contains(':') {
        ContentKey::parse(text)
    } else {
        ContentKey::new(pack, text)
    }
}

/// Parses the `adventure.toml` of the core pack with this manifest (`""` for a pack without the
/// file).
pub fn parse_adventure(
    manifest: &PackManifest,
    text: &str,
) -> Result<AdventurePack, AdventurePackError> {
    let file: AdventureFile =
        toml::from_str(text).map_err(|e| AdventurePackError::Syntax(e.to_string()))?;
    let pack = manifest.id.as_str();
    let terrains = file
        .terrain
        .into_iter()
        .map(|t| {
            let passage = match (t.passage, t.cost) {
                (PassageDto::Land, Some(cost)) => Passage::Land(cost),
                (PassageDto::Water, Some(cost)) => Passage::Water(cost),
                (PassageDto::Impassable, None) => Passage::Impassable,
                _ => return Err(AdventurePackError::TerrainCost(t.terrain)),
            };
            Ok(TerrainMovementDef {
                terrain: reference(pack, &t.terrain)?,
                passage,
                pathfinding: t.pathfinding,
            })
        })
        .collect::<Result<_, AdventurePackError>>()?;
    let roads = file
        .road
        .into_iter()
        .map(|r| {
            Ok(RoadDef {
                key: ContentKey::new(pack, &r.id)?,
                name: r.name,
                cost: r.cost,
            })
        })
        .collect::<Result<_, KeyError>>()?;
    let rivers = file
        .river
        .into_iter()
        .map(|r| {
            Ok(RiverDef {
                key: ContentKey::new(pack, &r.id)?,
                name: r.name,
            })
        })
        .collect::<Result<_, KeyError>>()?;
    let object_types = file
        .object_type
        .into_iter()
        .map(|t| object_type(pack, t))
        .collect::<Result<_, _>>()?;
    let amounts = |row: &BTreeMap<String, u32>| {
        row.iter()
            .map(|(k, &a)| Ok((reference(pack, k)?, a)))
            .collect::<Result<Vec<_>, KeyError>>()
    };
    let starting_resources = file
        .difficulty
        .iter()
        .map(|d| {
            Ok(StartingResourcesDef {
                human: amounts(&d.human)?,
                computer: amounts(&d.computer)?,
            })
        })
        .collect::<Result<_, KeyError>>()?;
    let not_random_monsters = file
        .not_random_monsters
        .iter()
        .map(|k| reference(pack, k))
        .collect::<Result<_, KeyError>>()?;
    let terrain_walkers = file
        .terrain_walker
        .iter()
        .map(|w| {
            Ok(TerrainWalkerDef {
                creature: reference(pack, &w.creature)?,
                terrain: reference(pack, &w.terrain)?,
                counts_as: reference(pack, &w.counts_as)?,
            })
        })
        .collect::<Result<_, KeyError>>()?;
    Ok(AdventurePack {
        terrains,
        roads,
        rivers,
        object_types,
        starting_resources,
        not_random_monsters,
        terrain_walkers,
        params: file
            .params
            .map(|p| {
                let battle = p
                    .battle
                    .map(|b| {
                        Ok::<_, KeyError>(BattleTerrainsDef {
                            underground: reference(pack, &b.underground)?,
                            coast: reference(pack, &b.coast)?,
                            coast_battlefield: reference(pack, &b.coast_battlefield)?,
                            sea: reference(pack, &b.sea)?,
                            deck: reference(pack, &b.deck)?,
                            boats: reference(pack, &b.boats)?,
                        })
                    })
                    .transpose()?;
                let monsters = p
                    .monsters
                    .map(|m| {
                        Ok::<_, KeyError>(MonsterParamsDef {
                            aggression: m.aggression,
                            count_words: m.count_words,
                            price_resource: reference(pack, &m.price_resource)?,
                        })
                    })
                    .transpose()?;
                Ok::<_, KeyError>(AdventureParamsDef {
                    native_cost: p.native_cost,
                    land_points: p.land_points,
                    scouting_radius: p.scouting_radius,
                    max_map_size: p.max_map_size,
                    max_levels: p.max_levels,
                    max_players: p.max_players,
                    army_slots: p.army_slots,
                    max_stack_count: p.max_stack_count,
                    starting_stack_chances: p.starting_stack_chances,
                    battle,
                    monsters,
                    fly_cost: p.fly_cost,
                    water_walk_cost: p.water_walk_cost,
                    sea_points: p.sea_points,
                    sea_ground: p.sea_ground,
                })
            })
            .transpose()?,
        ..AdventurePack::new(manifest.clone())
    })
}

/// Parses and compiles packs given as `(core.toml, adventure.toml)` texts (`""` for a pack
/// without an adventure file) with the profile `profile`: the core ruleset and its adventure part.
pub fn compile(
    packs: &[(&str, &str)],
    profile: &str,
) -> Result<(Ruleset, AdventureRuleset), AdventurePackError> {
    let core_packs = packs
        .iter()
        .map(|(core, _)| h3_pack::parse_pack(core))
        .collect::<Result<Vec<_>, _>>()?;
    let core = Ruleset::compile(&core_packs, profile).map_err(PackError::from)?;
    let adventure = core_packs
        .iter()
        .zip(packs)
        .map(|(core_pack, (_, text))| parse_adventure(&core_pack.manifest, text))
        .collect::<Result<Vec<_>, _>>()?;
    let part = AdventureRuleset::compile(&core, &adventure)?;
    Ok((core, part))
}

/// The three original packs, RoE to SoD, as `(core.toml, adventure.toml)` texts.
pub const ORIGINAL_PACKS: [(&str, &str); 3] = [
    (h3_pack::BASE_CORE, BASE_ADVENTURE),
    (h3_pack::AB_CORE, AB_ADVENTURE),
    (h3_pack::SOD_CORE, SOD_ADVENTURE),
];

#[cfg(test)]
mod tests;
