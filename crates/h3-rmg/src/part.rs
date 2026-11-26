//! The rmg part (ADR-0029 point 5): the generator's three tables as the game reads them, compiled
//! from the enabled packs' `rmg.toml` in the core's load order, immutable, with its own hash.
//!
//! - `rmg.txt`'s templates, zones and connections in row order with the ids as read (survey
//!   `docs/rules/surveys/E14-generator.md` §3.1): the draw is over the kept templates in row order
//!   and a zone's index comes from its row, so nothing is sorted or keyed. Each pack appends its
//!   templates; a connection override changes a connection's zones, gated by a data quirk's flag
//!   when it fixes one (8XM8's ids, RMG-QUIRK-22).
//! - `objects.txt`'s template lines in file order: a pack inserts lines before a line of the
//!   table the earlier packs left (or appends them) and replaces lines of it (SoD's 105 added and
//!   two changed lines; the file's count line is the lines' number, not data).
//! - `rand_trn.txt`'s rows (the obstacles): a pack that gives them gives the whole table, each
//!   row's Adjacent and Overlap one score per row.
//!
//! The values are kept as read (`c_atoi`'s numbers, a mark's set-ness as a bit); the generator
//! applies the original's narrowing where it does. The part refers to no core content: the
//! tables index the game's town types, terrains and object classes by position, as the
//! original's code does (ADR-0029 point 5: the algorithm and those positions stay code).

use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use h3_content::{PackManifest, REGISTRY, Ruleset, override_applies};
use h3_kernel::hash::hash_of;
use h3_kernel::{CanonicalHash, StateHasher};

use crate::RMG_RULESET_HASH_VERSION;
use crate::settings::{MapFormat, TOWN_TYPES};

/// A zone's kind: the last of `rmg.txt`'s columns 4 to 7 set, a treasure zone when none is
/// (RMG-TPL-002; the original's type 0 to 3, [`ZoneKind::value`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ZoneKind {
    /// A human start (column 4), type 0.
    HumanStart,
    /// A computer start (column 5), type 1.
    ComputerStart,
    /// A treasure zone (column 6, or none set), type 2.
    Treasure,
    /// A junction (column 7), type 3.
    Junction,
}

impl ZoneKind {
    /// The original's type number (`rmg_zone` +0x4).
    pub const fn value(self) -> i32 {
        match self {
            Self::HumanStart => 0,
            Self::ComputerStart => 1,
            Self::Treasure => 2,
            Self::Junction => 3,
        }
    }
}

/// A zone's monster strength: column 55's first letter in either case, `n` none, `w` weak, `s`
/// strong, anything else (an empty cell too) average (RMG-TPL-002; [`Strength::value`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Strength {
    /// No monsters, 0.
    None,
    /// Weak, 2.
    Weak,
    /// Average, 3.
    Average,
    /// Strong, 4.
    Strong,
}

impl Strength {
    /// The original's number (`rmg_zone` +0x90).
    pub const fn value(self) -> i32 {
        match self {
            Self::None => 0,
            Self::Weak => 2,
            Self::Average => 3,
            Self::Strong => 4,
        }
    }
}

/// A template zone, one zone row of `rmg.txt` (record `rmg_zone`, `docs/exe/structs.toml`). The
/// marks are bit sets: bit `k` is the group's `k`-th column set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Zone {
    /// Column 3: the id the connections name it by.
    pub id: i32,
    /// Columns 4 to 7.
    pub kind: ZoneKind,
    /// Column 8.
    pub base_size: i32,
    /// Columns 9 and 10: the least and most human players the zone is kept for.
    pub humans: [i32; 2],
    /// Columns 11 and 12: the least and most players, humans and computers.
    pub players: [i32; 2],
    /// Column 13, Ownership as read (the start position is the value less 1).
    pub ownership: i32,
    /// Columns 14 to 17: the zone colour's least towns, least castles, town and castle density.
    pub player_towns: [i32; 4],
    /// Columns 18 to 21: the same for neutral towns.
    pub neutral_towns: [i32; 4],
    /// Column 22: towns of the same type.
    pub same_type: bool,
    /// Columns 23 to 31: the town types, Castle to Conflux (9 bits).
    pub town_types: u16,
    /// Columns 32 to 38: the least mines, wood, mercury, ore, sulfur, crystal, gems, gold.
    pub mines: [i32; 7],
    /// Columns 39 to 45: the mines' densities in the same order.
    pub mine_density: [i32; 7],
    /// Column 46: the terrain matches the town.
    pub terrain_match: bool,
    /// Columns 47 to 54: the terrains, dirt to lava (8 bits).
    pub terrains: u8,
    /// Column 55.
    pub strength: Strength,
    /// Column 56: the monsters match the town.
    pub monster_match: bool,
    /// Column 57: neutral monsters.
    pub neutral_monsters: bool,
    /// Columns 58 to 66: the monsters' town types, Castle to the ninth (9 bits).
    pub monster_towns: u16,
    /// Columns 67 to 75: three treasure bands, each its low value, high value and density.
    pub treasure: [[i32; 3]; 3],
}

/// A template connection, one connection row of `rmg.txt` (record `rmg_zone_connection`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Connection {
    /// Columns 76 and 77: the two zones' ids as `c_atoi` reads them.
    pub zones: [i32; 2],
    /// Column 78: the guard's value.
    pub value: i32,
    /// Column 79 set.
    pub wide: bool,
    /// Column 80 set: a Border Guard.
    pub border_guard: bool,
    /// Columns 81 and 82: the least and most human players it is kept for.
    pub humans: [i32; 2],
    /// Columns 83 and 84: the least and most players, humans and computers.
    pub players: [i32; 2],
}

/// A template of `rmg.txt` (record `rmg_template`): its zones and connections in row order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Template {
    /// Column 0 of its first row, line breaks as read.
    pub name: String,
    /// Columns 1 and 2: the least and greatest map size (width × height × levels / 1296).
    pub sizes: [i32; 2],
    /// Its zone rows.
    pub zones: Vec<Zone>,
    /// Its connection rows, one per row (a pair given twice is two connections).
    pub connections: Vec<Connection>,
}

/// A template line of `objects.txt`. The masks are bit sets: bit `k` is the line's `k`-th
/// character `1`, as the file gives them (`docs/formats/h3m.md`, "Template masks").
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectTemplate {
    /// The picture's name (`.def`).
    pub picture: String,
    /// The passability mask (48 bits).
    pub passability: u64,
    /// The actions mask (48 bits).
    pub actions: u64,
    /// The terrains it may stand on (9 bits).
    pub terrains: u16,
    /// The editor's terrain menu (9 bits).
    pub editor_terrains: u16,
    /// The object class.
    pub class: u32,
    /// The subclass.
    pub subclass: u32,
    /// The editor's group.
    pub group: u8,
    /// The overlay field.
    pub overlay: u8,
}

/// An obstacle, one row of `rand_trn.txt` (record `rmg_obstacle`, RMG-TPL-006).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Obstacle {
    /// Column 3: the object type.
    pub object_type: i32,
    /// Column 4: the subtype.
    pub subtype: i32,
    /// Column 6: the terrain.
    pub terrain: i32,
    /// Columns 7 to 15: the modifiers by terrain, dirt to water (the reader adds a tenth, -5000:
    /// code, not data).
    pub modifiers: [i32; 9],
    /// The Adjacent scores, one per obstacle row in order.
    pub adjacent: Vec<i32>,
    /// The Overlap scores, one per obstacle row in order.
    pub overlap: Vec<i32>,
}

/// An `objects.txt` line a pack adds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectTemplateLine {
    /// Inserted before this line of the table the earlier packs left (from 0); `None` (or the
    /// table's length) appends it. Lines before the same line keep the pack's order.
    pub before: Option<usize>,
    /// The line.
    pub template: ObjectTemplate,
}

/// An `objects.txt` line a pack replaces.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectTemplateOverride {
    /// The line of the table the earlier packs left (from 0).
    pub line: usize,
    /// Its new content.
    pub template: ObjectTemplate,
}

/// A change to a connection of a template of the pack's dependency closure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectionOverride {
    /// The template, by its index in the table as it stands after the pack's own templates.
    pub template: usize,
    /// The connection, by its index in the template's connections.
    pub connection: usize,
    /// The data quirk it fixes (`quirk.content.<name>`): applied only in profiles that turn the
    /// flag off (ADR-0007, amendment E04b T8e); `None` always applies.
    pub fixes: Option<String>,
    /// The connection's new zone ids.
    pub zones: [i32; 2],
}

/// The number of object types the generator's limits cover (`rmg_type_limit_map` 0x69CE4C and
/// `rmg_type_limit_zone` 0x69D1F4, 232 entries each).
pub const OBJECT_TYPES: usize = 232;

/// The limit an object type has before the pairs (`rmg_generator_create` 0x537D42).
pub const NO_LIMIT: i32 = 32_000;

/// The points around a zone step 1 tries (`rmg_layout_cos` 0x682500 and `rmg_layout_sin`
/// 0x682600, 32 entries each, one per 11.25°).
pub const LAYOUT_POINTS: usize = 32;

/// The terrains the generator's tables cover (dirt to lava, then water).
pub const TERRAINS: usize = 9;

/// The town types of a terrain's row in `rmg_terrain_towns` 0x682450 (four entries a terrain).
pub const TERRAIN_TOWNS: usize = 4;

/// The generator's constants read from SoD's executable (ADR-0029 point 5; `packs/sod/rmg.toml`'s
/// `[generator]`, harvested with their addresses): what the generator's constructor
/// (`rmg_generator_create` 0x537B10) sets up besides the three tables.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratorTables {
    /// Per hero id, the hero table's (0x679DD0, 0x5C bytes a hero) byte +0x38 set: one of RoE's.
    pub heroes_roe: Vec<bool>,
    /// Its byte +0x39 set: one of the expansions'.
    pub heroes_expansions: Vec<bool>,
    /// Its byte +0x3A set: a campaign hero.
    pub heroes_campaign: Vec<bool>,
    /// The per-map limits by object type (`rmg_type_limits_a` 0x640718): object type and limit,
    /// in the table's order.
    pub map_limits: Vec<(u8, i32)>,
    /// The per-zone limits (`rmg_type_limits_b` 0x640808), in the table's order.
    pub zone_limits: Vec<(u8, i32)>,
    /// Step 1's cosines (`rmg_layout_cos` 0x682500), [`LAYOUT_POINTS`] of them, in
    /// ten-thousandths: the table's doubles are the ones nearest these decimals (0.9807 for
    /// cos 11.25°).
    pub layout_cos: Vec<i32>,
    /// Its sines (`rmg_layout_sin` 0x682600), likewise.
    pub layout_sin: Vec<i32>,
    /// Per terrain (dirt to lava, water), the four town types step 1 draws a zone's second town
    /// type among (`rmg_terrain_towns` 0x682450), -1 ending a row's own (RMG-QUIRK-12).
    pub terrain_towns: Vec<[i32; TERRAIN_TOWNS]>,
    /// Per town type (Castle to Conflux), its native terrain (`rmg_town_native_terrain`
    /// 0x6408C8).
    pub native_terrains: Vec<i32>,
}

/// The limits by object type the generator places under, per map and per zone (RMG-SET-009).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeLimits {
    /// Per map.
    pub map: [i32; OBJECT_TYPES],
    /// Per zone.
    pub zone: [i32; OBJECT_TYPES],
}

impl GeneratorTables {
    /// Each hero's ban for `format`, by id (RMG-SET-008, `rmg_generator_create` 0x537D0B): banned
    /// when a campaign hero, or when not one of the format's heroes (RoE's for RoE, the
    /// expansions' for AB and SoD).
    // rule: RMG-SET-008
    pub fn banned_heroes(&self, format: MapFormat) -> Vec<bool> {
        let available = if format == MapFormat::RoE {
            &self.heroes_roe
        } else {
            &self.heroes_expansions
        };
        self.heroes_campaign
            .iter()
            .zip(available)
            .map(|(&campaign, &available)| campaign || !available)
            .collect()
    }

    /// The limits by object type (RMG-SET-009, `rmg_generator_create` 0x537D3D to 0x537D95):
    /// every type [`NO_LIMIT`], then each table's pairs from its last to its first (a type given
    /// twice keeps its first pair's limit); a type past [`OBJECT_TYPES`] (which the compiler
    /// refuses) sets nothing.
    // rule: RMG-SET-009
    pub fn type_limits(&self) -> TypeLimits {
        let fill = |pairs: &[(u8, i32)]| {
            let mut limits = [NO_LIMIT; OBJECT_TYPES];
            for &(object_type, limit) in pairs.iter().rev() {
                if let Some(slot) = limits.get_mut(usize::from(object_type)) {
                    *slot = limit;
                }
            }
            limits
        };
        TypeLimits {
            map: fill(&self.map_limits),
            zone: fill(&self.zone_limits),
        }
    }
}

/// One pack's `rmg.toml`, as `h3-pack-rmg` parses it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RmgPack {
    /// The core pack's manifest (identity and dependencies).
    pub manifest: PackManifest,
    /// The templates it appends.
    pub templates: Vec<Template>,
    /// Its changes to connections.
    pub connection_overrides: Vec<ConnectionOverride>,
    /// The `objects.txt` lines it adds.
    pub object_templates: Vec<ObjectTemplateLine>,
    /// The `objects.txt` lines it replaces.
    pub object_template_overrides: Vec<ObjectTemplateOverride>,
    /// The whole obstacle table, if it gives one (it replaces an earlier pack's).
    pub obstacles: Option<Vec<Obstacle>>,
    /// The generator's tables, if it gives them (they replace an earlier pack's).
    pub generator: Option<GeneratorTables>,
}

impl RmgPack {
    /// A pack folder without an `rmg.toml`.
    pub const fn empty(manifest: PackManifest) -> Self {
        Self {
            manifest,
            templates: Vec::new(),
            connection_overrides: Vec::new(),
            object_templates: Vec::new(),
            object_template_overrides: Vec::new(),
            obstacles: None,
            generator: None,
        }
    }
}

/// Why the rmg part does not compile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RmgCompileError {
    /// A pack (`id@version`) that is not one of the core's.
    UnknownPack(String),
    /// Two `rmg.toml` for one pack.
    DuplicatePack(String),
    /// A core pack without its `rmg.toml` definitions (an empty [`RmgPack`] stands for none).
    MissingPack(String),
    /// A pack's dependency loads after it or is not enabled.
    UnknownDependency {
        /// The pack.
        pack: String,
        /// The dependency.
        dependency: String,
    },
    /// An `objects.txt` line a pack names is past the table or not of its dependency closure.
    ObjectLine {
        /// The pack.
        pack: String,
        /// The line.
        line: usize,
    },
    /// A pack replaces one `objects.txt` line twice.
    DuplicateObjectOverride {
        /// The pack.
        pack: String,
        /// The line.
        line: usize,
    },
    /// A connection override names a template past the table or not of the pack's closure.
    Template {
        /// The pack.
        pack: String,
        /// The template.
        template: usize,
    },
    /// A connection override names a connection past its template's.
    Connection {
        /// The pack.
        pack: String,
        /// The template.
        template: usize,
        /// The connection.
        connection: usize,
    },
    /// An override fixes a flag that is not a registered data quirk.
    NotADataQuirk(String),
    /// An obstacle's Adjacent or Overlap scores are not one per obstacle row.
    ObstacleScores {
        /// The pack.
        pack: String,
        /// The obstacle row.
        obstacle: usize,
    },
    /// The generator's hero bytes are not one each per hero, or a limit's object type is past
    /// [`OBJECT_TYPES`].
    GeneratorTables(String),
}

impl fmt::Display for RmgCompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownPack(pack) => write!(f, "rmg.toml of `{pack}`, which the core has not"),
            Self::DuplicatePack(pack) => write!(f, "two rmg.toml for pack `{pack}`"),
            Self::MissingPack(pack) => write!(f, "no rmg.toml definitions for pack `{pack}`"),
            Self::UnknownDependency { pack, dependency } => {
                write!(
                    f,
                    "pack `{pack}` depends on `{dependency}`, not loaded before it"
                )
            }
            Self::ObjectLine { pack, line } => write!(
                f,
                "pack `{pack}` names objects.txt line {line}, past the table or outside its closure"
            ),
            Self::DuplicateObjectOverride { pack, line } => {
                write!(f, "pack `{pack}` replaces objects.txt line {line} twice")
            }
            Self::Template { pack, template } => write!(
                f,
                "pack `{pack}` names template {template}, past the table or outside its closure"
            ),
            Self::Connection {
                pack,
                template,
                connection,
            } => write!(
                f,
                "pack `{pack}` names connection {connection} of template {template}, which has not \
                 as many"
            ),
            Self::NotADataQuirk(flag) => {
                write!(
                    f,
                    "an override fixes `{flag}`, which is no registered data quirk flag"
                )
            }
            Self::ObstacleScores { pack, obstacle } => write!(
                f,
                "pack `{pack}`: obstacle row {obstacle}'s Adjacent and Overlap need one score per \
                 obstacle row"
            ),
            Self::GeneratorTables(pack) => write!(
                f,
                "pack `{pack}`: the generator's hero bytes need one each per hero, its limits \
                 object types below {OBJECT_TYPES}, its layout {LAYOUT_POINTS} cosines and sines, \
                 its terrain towns {TERRAINS} rows of town types (-1 to 8) and its native terrains \
                 one terrain (0 to 8) per town type"
            ),
        }
    }
}

/// The compiled rmg part: the three tables for the core's profile, with its hash (ADR-0029
/// point 5). It is not part of the core `Ruleset`: a game reads the map, not the generator's data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RmgRuleset {
    core_hash: u64,
    templates: Vec<Template>,
    object_templates: Vec<ObjectTemplate>,
    obstacles: Vec<Obstacle>,
    generator: Option<GeneratorTables>,
    hash: u64,
}

impl RmgRuleset {
    /// The part without tables on `core` (what packs without an `rmg.toml` compile to).
    pub fn empty(core: &Ruleset) -> Self {
        Self::new(core, Tables::default())
    }

    /// Compiles one rmg pack per pack of `core`, in the core's load order, with the core's
    /// profile (its data quirk flags gate the overrides).
    ///
    /// # Errors
    ///
    /// An [`RmgCompileError`] for the first check that fails.
    pub fn compile(core: &Ruleset, packs: &[RmgPack]) -> Result<Self, RmgCompileError> {
        let order = load_order(core, packs)?;
        let mut tables = Tables::default();
        for (rank, (pack, closure)) in order.iter().enumerate() {
            let scope = Scope {
                id: &pack.manifest.id,
                rank,
                closure,
            };
            tables.objects(&scope, pack)?;
            tables
                .templates
                .extend(pack.templates.iter().map(|t| (rank, t.clone())));
            tables.overrides(&scope, core, pack)?;
            if let Some(obstacles) = &pack.obstacles {
                tables.obstacles(&scope, obstacles)?;
            }
            if let Some(generator) = &pack.generator {
                tables.generator(&scope, generator)?;
            }
        }
        Ok(Self::new(core, tables))
    }

    fn new(core: &Ruleset, tables: Tables) -> Self {
        let mut part = Self {
            core_hash: core.content_hash(),
            templates: tables.templates.into_iter().map(|(_, t)| t).collect(),
            object_templates: tables.objects.into_iter().map(|(_, o)| o).collect(),
            obstacles: tables.obstacles,
            generator: tables.generator,
            hash: 0,
        };
        part.hash = hash_of(&part);
        part
    }

    /// The content hash of the core ruleset this part was compiled on.
    pub const fn core_hash(&self) -> u64 {
        self.core_hash
    }

    /// Canonical hash of the compiled part ([`RMG_RULESET_HASH_VERSION`]); with the generator's
    /// version, the profile, the settings and the seed it names a generated map's provenance
    /// (ADR-0029 point 7).
    pub const fn content_hash(&self) -> u64 {
        self.hash
    }

    /// `rmg.txt`'s templates in row order.
    pub fn templates(&self) -> &[Template] {
        &self.templates
    }

    /// `objects.txt`'s template lines in file order.
    pub fn object_templates(&self) -> &[ObjectTemplate] {
        &self.object_templates
    }

    /// `rand_trn.txt`'s obstacle rows in order.
    pub fn obstacles(&self) -> &[Obstacle] {
        &self.obstacles
    }

    /// The generator's tables; `None` without them (a ruleset without `sod` has no generator,
    /// ADR-0029 point 5).
    pub const fn generator(&self) -> Option<&GeneratorTables> {
        self.generator.as_ref()
    }
}

impl CanonicalHash for RmgRuleset {
    /// The part's canonical bytes: [`RMG_RULESET_HASH_VERSION`]; the templates (their number,
    /// each its name, sizes, zones (their number, each its fields in [`Zone`]'s order, the kind
    /// and the strength as the original's numbers) and connections (their number, each its
    /// fields in [`Connection`]'s order)); the object templates (their number, each its fields
    /// in [`ObjectTemplate`]'s order); the obstacles (their number, each its fields in
    /// [`Obstacle`]'s order, the scores with their number); the generator's tables (whether given,
    /// then each list with its number in [`GeneratorTables`]' order, a pair its type and limit,
    /// a terrain's town types without their number, E14 T3c).
    /// The core's hash is not part of it.
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_u32(RMG_RULESET_HASH_VERSION);
        h.write_len(self.templates.len());
        for t in &self.templates {
            t.name.as_str().hash_into(h);
            write_i32s(h, &t.sizes);
            h.write_len(t.zones.len());
            t.zones.iter().for_each(|z| hash_zone(h, z));
            h.write_len(t.connections.len());
            for c in &t.connections {
                write_i32s(h, &c.zones);
                h.write_i32(c.value);
                h.write_bool(c.wide);
                h.write_bool(c.border_guard);
                write_i32s(h, &c.humans);
                write_i32s(h, &c.players);
            }
        }
        h.write_len(self.object_templates.len());
        for o in &self.object_templates {
            o.picture.as_str().hash_into(h);
            h.write_u64(o.passability);
            h.write_u64(o.actions);
            h.write_u16(o.terrains);
            h.write_u16(o.editor_terrains);
            h.write_u32(o.class);
            h.write_u32(o.subclass);
            h.write_u8(o.group);
            h.write_u8(o.overlay);
        }
        h.write_len(self.obstacles.len());
        for o in &self.obstacles {
            h.write_i32(o.object_type);
            h.write_i32(o.subtype);
            h.write_i32(o.terrain);
            write_i32s(h, &o.modifiers);
            h.write_len(o.adjacent.len());
            write_i32s(h, &o.adjacent);
            h.write_len(o.overlap.len());
            write_i32s(h, &o.overlap);
        }
        h.write_bool(self.generator.is_some());
        if let Some(g) = &self.generator {
            for bytes in [&g.heroes_roe, &g.heroes_expansions, &g.heroes_campaign] {
                h.write_len(bytes.len());
                bytes.iter().for_each(|&b| h.write_bool(b));
            }
            for pairs in [&g.map_limits, &g.zone_limits] {
                h.write_len(pairs.len());
                for &(object_type, limit) in pairs {
                    h.write_u8(object_type);
                    h.write_i32(limit);
                }
            }
            for values in [&g.layout_cos, &g.layout_sin] {
                h.write_len(values.len());
                write_i32s(h, values);
            }
            h.write_len(g.terrain_towns.len());
            for row in &g.terrain_towns {
                write_i32s(h, row);
            }
            h.write_len(g.native_terrains.len());
            write_i32s(h, &g.native_terrains);
        }
    }
}

fn write_i32s(h: &mut StateHasher, values: &[i32]) {
    values.iter().for_each(|&v| h.write_i32(v));
}

fn hash_zone(h: &mut StateHasher, z: &Zone) {
    h.write_i32(z.id);
    h.write_i32(z.kind.value());
    h.write_i32(z.base_size);
    write_i32s(h, &z.humans);
    write_i32s(h, &z.players);
    h.write_i32(z.ownership);
    write_i32s(h, &z.player_towns);
    write_i32s(h, &z.neutral_towns);
    h.write_bool(z.same_type);
    h.write_u16(z.town_types);
    write_i32s(h, &z.mines);
    write_i32s(h, &z.mine_density);
    h.write_bool(z.terrain_match);
    h.write_u8(z.terrains);
    h.write_i32(z.strength.value());
    h.write_bool(z.monster_match);
    h.write_bool(z.neutral_monsters);
    h.write_u16(z.monster_towns);
    z.treasure.iter().for_each(|band| write_i32s(h, band));
}

/// The pack being compiled: its id, its place in the load order and its dependency closure (as
/// places in the load order, itself included).
struct Scope<'a> {
    id: &'a str,
    rank: usize,
    closure: &'a BTreeSet<usize>,
}

impl Scope<'_> {
    fn sees(&self, rank: usize) -> bool {
        self.closure.contains(&rank)
    }
}

/// The tables so far, each entry with the load-order place of the pack that gave it.
#[derive(Default)]
struct Tables {
    templates: Vec<(usize, Template)>,
    objects: Vec<(usize, ObjectTemplate)>,
    obstacles: Vec<Obstacle>,
    generator: Option<GeneratorTables>,
}

impl Tables {
    /// Applies a pack's `objects.txt` lines: every line it names is of the table the earlier
    /// packs left and of its closure (`before` may also be the table's length).
    fn objects(&mut self, scope: &Scope<'_>, pack: &RmgPack) -> Result<(), RmgCompileError> {
        let len = self.objects.len();
        let line_error = |line| RmgCompileError::ObjectLine {
            pack: String::from(scope.id),
            line,
        };
        let seen = |line: usize| {
            self.objects
                .get(line)
                .is_some_and(|(rank, _)| scope.sees(*rank))
        };
        let mut replaced: Vec<Option<&ObjectTemplate>> = alloc::vec![None; len];
        for o in &pack.object_template_overrides {
            if !seen(o.line) {
                return Err(line_error(o.line));
            }
            match replaced.get_mut(o.line) {
                Some(slot @ None) => *slot = Some(&o.template),
                _ => {
                    return Err(RmgCompileError::DuplicateObjectOverride {
                        pack: String::from(scope.id),
                        line: o.line,
                    });
                }
            }
        }
        // The added lines by the line they go before (`len`: appended), in the pack's order.
        let mut before: Vec<Vec<&ObjectTemplate>> = alloc::vec![Vec::new(); len + 1];
        for o in &pack.object_templates {
            let at = o.before.unwrap_or(len);
            match before.get_mut(at) {
                Some(slot) if at == len || seen(at) => slot.push(&o.template),
                _ => return Err(line_error(at)),
            }
        }
        let old = core::mem::take(&mut self.objects);
        let mut lines = Vec::with_capacity(len + pack.object_templates.len());
        let mut added = before.into_iter();
        for ((rank, template), new) in old.into_iter().zip(replaced) {
            let inserted = added.next().unwrap_or_default();
            lines.extend(inserted.into_iter().map(|t| (scope.rank, t.clone())));
            lines.push(new.map_or((rank, template), |t| (scope.rank, t.clone())));
        }
        lines.extend(added.flatten().map(|t| (scope.rank, t.clone())));
        self.objects = lines;
        Ok(())
    }

    /// Applies a pack's connection overrides, each checked in every profile and applied when its
    /// flag (if any) is off.
    fn overrides(
        &mut self,
        scope: &Scope<'_>,
        core: &Ruleset,
        pack: &RmgPack,
    ) -> Result<(), RmgCompileError> {
        for o in &pack.connection_overrides {
            let template = self
                .templates
                .get_mut(o.template)
                .filter(|(rank, _)| scope.sees(*rank))
                .map(|(_, t)| t)
                .ok_or_else(|| RmgCompileError::Template {
                    pack: String::from(scope.id),
                    template: o.template,
                })?;
            let connection = template.connections.get_mut(o.connection).ok_or_else(|| {
                RmgCompileError::Connection {
                    pack: String::from(scope.id),
                    template: o.template,
                    connection: o.connection,
                }
            })?;
            let applies = override_applies(o.fixes.as_deref(), &REGISTRY, core.quirks())
                .map_err(|_| RmgCompileError::NotADataQuirk(o.fixes.clone().unwrap_or_default()))?;
            if applies {
                connection.zones = o.zones;
            }
        }
        Ok(())
    }

    /// A pack's whole obstacle table, each row with one Adjacent and one Overlap score per row.
    fn obstacles(
        &mut self,
        scope: &Scope<'_>,
        obstacles: &[Obstacle],
    ) -> Result<(), RmgCompileError> {
        let n = obstacles.len();
        if let Some(obstacle) = obstacles
            .iter()
            .position(|o| o.adjacent.len() != n || o.overlap.len() != n)
        {
            return Err(RmgCompileError::ObstacleScores {
                pack: String::from(scope.id),
                obstacle,
            });
        }
        self.obstacles = obstacles.to_vec();
        Ok(())
    }

    /// A pack's generator's tables: the three hero lists of one length, the limits' types below
    /// [`OBJECT_TYPES`], [`LAYOUT_POINTS`] cosines and sines, [`TERRAINS`] rows of town types
    /// (-1 to 8) and a terrain (0 to 8) per town type.
    fn generator(
        &mut self,
        scope: &Scope<'_>,
        generator: &GeneratorTables,
    ) -> Result<(), RmgCompileError> {
        let heroes = generator.heroes_roe.len();
        let fits = generator.heroes_expansions.len() == heroes
            && generator.heroes_campaign.len() == heroes
            && generator
                .map_limits
                .iter()
                .chain(&generator.zone_limits)
                .all(|&(object_type, _)| usize::from(object_type) < OBJECT_TYPES)
            && generator.layout_cos.len() == LAYOUT_POINTS
            && generator.layout_sin.len() == LAYOUT_POINTS
            && generator.terrain_towns.len() == TERRAINS
            && generator
                .terrain_towns
                .iter()
                .flatten()
                .all(|town| (-1..=8).contains(town))
            && generator.native_terrains.len() == usize::from(TOWN_TYPES)
            && generator
                .native_terrains
                .iter()
                .all(|terrain| (0..=8).contains(terrain));
        if !fits {
            return Err(RmgCompileError::GeneratorTables(String::from(scope.id)));
        }
        self.generator = Some(generator.clone());
        Ok(())
    }
}

/// The packs in the core's load order, each with its closure as places in that order.
fn load_order<'a>(
    core: &Ruleset,
    packs: &'a [RmgPack],
) -> Result<Vec<(&'a RmgPack, BTreeSet<usize>)>, RmgCompileError> {
    let mut by_id: Vec<(&str, &RmgPack)> = Vec::new();
    for pack in packs {
        let id = pack.manifest.id.as_str();
        let known = core
            .packs()
            .iter()
            .any(|(core_id, version)| core_id == id && *version == pack.manifest.version);
        if !known {
            let mut text = String::from(id);
            text.push('@');
            text.push_str(&pack.manifest.version);
            return Err(RmgCompileError::UnknownPack(text));
        }
        if by_id.iter().any(|(seen, _)| *seen == id) {
            return Err(RmgCompileError::DuplicatePack(String::from(id)));
        }
        by_id.push((id, pack));
    }
    let mut order: Vec<(&RmgPack, BTreeSet<usize>)> = Vec::new();
    for (rank, (id, _)) in core.packs().iter().enumerate() {
        let pack = by_id
            .iter()
            .find(|(seen, _)| seen == id)
            .map(|(_, p)| *p)
            .ok_or_else(|| RmgCompileError::MissingPack(id.clone()))?;
        let mut closure = BTreeSet::from([rank]);
        for dependency in &pack.manifest.depends {
            let inner = order
                .iter()
                .find(|(p, _)| p.manifest.id == *dependency)
                .map(|(_, c)| c)
                .ok_or_else(|| RmgCompileError::UnknownDependency {
                    pack: id.clone(),
                    dependency: dependency.clone(),
                })?;
            closure.extend(inner.iter().copied());
        }
        order.push((pack, closure));
    }
    Ok(order)
}
