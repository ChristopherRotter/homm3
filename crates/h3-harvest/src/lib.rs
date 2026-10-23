//! Harvester (E02 T4, OPEN-QUESTIONS Q8/Q13/Q15): turns the user's installation into the committed
//! TOML packs `base` (RoE, latest patch), `ab` (Armageddon's Blade) and `sod` (Shadow of Death).
//!
//! Inputs, all read-only: `Heroes3.exe` (static creature table: town, level, flags) and the text
//! tables in `Data/H3bitmap.lod` (SoD) and `Data/H3ab_bmp.lod` (AB-era), plus the committed
//! gap-fill file with labelled sources for what the installation cannot tell
//! (`data/gapfill.toml`). Output: the pack texts, each split into `core.toml` and `battle.toml`
//! ([`split`]), deterministic byte for byte, and a report of
//! everything that was not harvested or looked suspicious. The harvester checks that the packs
//! compile in every allowed expansion combination before returning them.
//!
//! Layouts: `docs/formats/installation.md`; pack format: `packs/README.md`.

pub mod artifacts;
pub mod battlefields;
pub mod gapfill;
pub mod heroes;
pub mod sieges;
pub mod split;

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::Path;

use h3_formats::exe::{CreatureRecord, HeroData, ObstacleData, SiegeData};
use h3_formats::lod::LodArchive;
use h3_formats::text::TextTable;

use crate::gapfill::{AbilitySpec, GapFill, Since};
use crate::heroes::{Context, CreatureRef};
use crate::split::split;

/// Number of creature ids in the executable's table and the creature text table.
pub const CREATURE_COUNT: usize = 150;
/// Resource columns of `CRTRAITS.TXT`, in column order (they match the first seven rows of
/// `RESTYPES.TXT`).
pub const RESOURCE_COUNT: usize = 7;
/// Fields per row of `CRTRAITS.TXT`.
pub const CRTRAITS_FIELDS: usize = 25;

/// Parsed inputs of a harvest.
#[derive(Debug, Clone)]
pub struct Inputs {
    /// Static creature table of the executable.
    pub exe: Vec<CreatureRecord>,
    /// `CRTRAITS.TXT` from the SoD archive.
    pub sod_creatures: TextTable,
    /// `CRTRAITS.TXT` from the AB-era archive.
    pub ab_creatures: TextTable,
    /// `TownType.txt`.
    pub towns: TextTable,
    /// `RESTYPES.TXT`.
    pub resources: TextTable,
    /// `TERRNAME.txt` (terrain types).
    pub terrains: TextTable,
    /// The executable's hero-related tables (E04a T3).
    pub hero_data: HeroData,
    /// The executable's obstacle tables (E04c T2b).
    pub obstacles: ObstacleData,
    /// The executable's siege tables and constants (E04c T5).
    pub siege: SiegeData,
    /// The executable's wall-shot tables (E04c T6).
    pub wall_shots: h3_formats::exe::WallShotData,
    /// `WALLS.TXT` and `BALLIST.TXT` (E04c T6).
    pub wall_tables: (TextTable, TextTable),
    /// Hero content text tables (E04a T3).
    pub hero_tables: heroes::Tables,
    /// Artifact text tables (E04a T3b).
    pub artifact_tables: artifacts::Tables,
}

impl Inputs {
    /// Reads an installation directory (or a copy of it): `Heroes3.exe`, `Data/H3bitmap.lod`,
    /// `Data/H3ab_bmp.lod`. Only reads.
    ///
    /// # Errors
    /// Missing files, unknown executable build, malformed archives or tables.
    pub fn read(install: &Path) -> Result<Self, String> {
        let read = |rel: &str| {
            let path = install.join(rel);
            std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))
        };
        let exe_bytes = read("Heroes3.exe")?;
        let exe =
            h3_formats::exe::creature_table(&exe_bytes).map_err(|e| format!("Heroes3.exe: {e}"))?;
        let hero_data =
            h3_formats::exe::hero_data(&exe_bytes).map_err(|e| format!("Heroes3.exe: {e}"))?;
        let obstacles =
            h3_formats::exe::obstacle_data(&exe_bytes).map_err(|e| format!("Heroes3.exe: {e}"))?;
        let siege =
            h3_formats::exe::siege_data(&exe_bytes).map_err(|e| format!("Heroes3.exe: {e}"))?;
        let wall_shots =
            h3_formats::exe::wall_shot_data(&exe_bytes).map_err(|e| format!("Heroes3.exe: {e}"))?;
        let sod = read("Data/H3bitmap.lod")?;
        let ab = read("Data/H3ab_bmp.lod")?;
        let table = |data: &[u8], archive: &str, name: &str| -> Result<TextTable, String> {
            let lod = LodArchive::parse(data).map_err(|e| format!("{archive}: {e}"))?;
            let bytes = lod
                .read_by_name(name)
                .map_err(|e| format!("{archive}/{name}: {e}"))?
                .ok_or_else(|| format!("{archive}: no {name}"))?;
            TextTable::parse(&bytes).map_err(|e| format!("{archive}/{name}: {e}"))
        };
        Ok(Self {
            exe,
            sod_creatures: table(&sod, "H3bitmap.lod", "CRTRAITS.TXT")?,
            ab_creatures: table(&ab, "H3ab_bmp.lod", "CRTRAITS.TXT")?,
            towns: table(&sod, "H3bitmap.lod", "TownType.txt")?,
            resources: table(&sod, "H3bitmap.lod", "RESTYPES.TXT")?,
            terrains: table(&sod, "H3bitmap.lod", "TERRNAME.txt")?,
            hero_data,
            obstacles,
            siege,
            wall_shots,
            wall_tables: (
                table(&sod, "H3bitmap.lod", "WALLS.TXT")?,
                table(&sod, "H3bitmap.lod", "BALLIST.TXT")?,
            ),
            hero_tables: heroes::Tables {
                classes: table(&sod, "H3bitmap.lod", "HCTRAITS.TXT")?,
                heroes: table(&sod, "H3bitmap.lod", "HOTRAITS.TXT")?,
                skills: table(&sod, "H3bitmap.lod", "SSTRAITS.TXT")?,
                sod_spells: table(&sod, "H3bitmap.lod", "SPTRAITS.TXT")?,
                ab_spells: table(&ab, "H3ab_bmp.lod", "SPTRAITS.TXT")?,
            },
            artifact_tables: artifacts::Tables {
                sod: table(&sod, "H3bitmap.lod", "ARTRAITS.TXT")?,
                ab: table(&ab, "H3ab_bmp.lod", "ARTRAITS.TXT")?,
                sod_slots: table(&sod, "H3bitmap.lod", "ArtSlots.txt")?,
                ab_slots: table(&ab, "H3ab_bmp.lod", "ArtSlots.txt")?,
            },
        })
    }
}

/// One harvested row of `CRTRAITS.TXT` (the harvested columns only).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    /// Singular name.
    pub name: String,
    /// Plural name.
    pub plural: String,
    /// Cost per resource column.
    pub cost: [u32; RESOURCE_COUNT],
    /// Fight value.
    pub fight_value: u32,
    /// AI value.
    pub ai_value: u32,
    /// Growth.
    pub growth: u16,
    /// Horde growth.
    pub horde_growth: u16,
    /// Hit points.
    pub hit_points: u32,
    /// Speed.
    pub speed: u8,
    /// Attack.
    pub attack: u16,
    /// Defense.
    pub defense: u16,
    /// Damage low/high.
    pub damage: [u16; 2],
    /// Shots.
    pub shots: u16,
    /// Spell casts.
    pub casts: u8,
    /// Adventure-map count low/high.
    pub adventure_count: [u16; 2],
    /// Ability text (flavor; never written to packs, compared only to report changes).
    pub ability_text: String,
    /// Attribute column ("reference only"; cross-checked against the executable's flags).
    pub attributes: String,
}

fn number<T: std::str::FromStr>(row: &[String], col: usize, line: usize) -> Result<T, String> {
    let text = row.get(col).map_or("", |s| s.trim());
    text.parse().map_err(|_| {
        format!(
            "CRTRAITS record {line}, column {}: `{text}` is not a valid number",
            col + 1
        )
    })
}

/// Parses the creature table into 150 rows (header and separator rows dropped).
///
/// # Errors
/// Wrong field or row counts, non-numeric values.
pub fn creature_rows(table: &TextTable) -> Result<Vec<Row>, String> {
    let mut rows = Vec::with_capacity(CREATURE_COUNT);
    for (i, record) in table.records.iter().enumerate().skip(2) {
        if TextTable::is_blank(record) {
            continue;
        }
        let line = i + 1;
        if record.len() != CRTRAITS_FIELDS {
            return Err(format!(
                "CRTRAITS record {line}: {} fields, expected {CRTRAITS_FIELDS}",
                record.len()
            ));
        }
        let mut cost = [0u32; RESOURCE_COUNT];
        for (r, slot) in cost.iter_mut().enumerate() {
            *slot = number(record, 2 + r, line)?;
        }
        rows.push(Row {
            name: record[0].trim().to_owned(),
            plural: record[1].trim().to_owned(),
            cost,
            fight_value: number(record, 9, line)?,
            ai_value: number(record, 10, line)?,
            growth: number(record, 11, line)?,
            horde_growth: number(record, 12, line)?,
            hit_points: number(record, 13, line)?,
            speed: number(record, 14, line)?,
            attack: number(record, 15, line)?,
            defense: number(record, 16, line)?,
            damage: [number(record, 17, line)?, number(record, 18, line)?],
            shots: number(record, 19, line)?,
            casts: number(record, 20, line)?,
            adventure_count: [number(record, 21, line)?, number(record, 22, line)?],
            ability_text: record[23].clone(),
            attributes: record[24].clone(),
        });
    }
    if rows.len() != CREATURE_COUNT {
        return Err(format!(
            "CRTRAITS: {} creature rows, expected {CREATURE_COUNT}",
            rows.len()
        ));
    }
    Ok(rows)
}

impl Row {
    /// Whether the harvested gameplay columns (everything but the two text columns) are equal.
    fn same_values(&self, other: &Self) -> bool {
        let strip = |r: &Self| Self {
            ability_text: String::new(),
            attributes: String::new(),
            ..r.clone()
        };
        strip(self) == strip(other)
    }
}

/// Executable flag bits and the pack flag names (`h3_content::CreatureFlag`). Bit `0x800` is
/// unidentified and not harvested (docs/formats/installation.md §5).
pub const FLAG_BITS: [(u32, &str); 20] = [
    (0x1, "two_hex"),
    (0x2, "flying"),
    (0x4, "shooter"),
    (0x8, "breath_attack"),
    (0x10, "alive"),
    (0x20, "catapult"),
    (0x40, "war_machine"),
    (0x80, "king_1"),
    (0x100, "king_2"),
    (0x200, "king_3"),
    (0x400, "mind_immune"),
    (0x1000, "no_melee_penalty"),
    (0x4000, "fire_immune"),
    (0x8000, "strikes_twice"),
    (0x1_0000, "no_enemy_retaliation"),
    (0x2_0000, "no_morale"),
    (0x4_0000, "undead"),
    (0x8_0000, "multi_headed"),
    (0x10_0000, "area_attack"),
    (0x8000_0000, "dragon"),
];
/// The war-machine bit: such entries are not harvested as creatures (yet).
pub const WAR_MACHINE_BIT: u32 = 0x40;
/// Bits that occur in the table but are not harvested.
pub const UNHARVESTED_BITS: u32 = 0x800;

/// Attribute-column tokens and the flag each implies.
const ATTRIBUTE_FLAGS: [(&str, &str); 14] = [
    ("DOUBLE_WIDE", "two_hex"),
    ("FLYING_ARMY", "flying"),
    ("SHOOTING_ARMY", "shooter"),
    ("IS_UNDEAD", "undead"),
    ("KING_1", "king_1"),
    ("KING_2", "king_2"),
    ("KING_3", "king_3"),
    ("IMMUNE_TO_MIND_SPELLS", "mind_immune"),
    ("IMMUNE_TO_FIRE_SPELLS", "fire_immune"),
    ("SIEGE_WEAPON", "war_machine"),
    ("CATAPULT", "catapult"),
    ("const_two_attacks", "strikes_twice"),
    ("const_free_attack", "no_enemy_retaliation"),
    ("const_no_melee_penalty", "no_melee_penalty"),
];

/// Attribute-column tokens that name abilities (E03 T2): the gap-fill file must give the creature
/// one of the listed ability kinds (a cross-check like [`ATTRIBUTE_FLAGS`]).
const ATTRIBUTE_ABILITIES: [(&str, &[&str]); 4] = [
    ("const_jousting", &["jousting"]),
    ("const_raises_morale", &["morale_aura"]),
    ("const_lowers_morale", &["morale_aura"]),
    (
        "MULTI_HEADED",
        &["attacks_all_adjacent", "three_headed_attack"],
    ),
];

/// Flag names of an executable record.
///
/// # Errors
/// A bit that is neither mapped nor known-unharvested.
pub fn flags(bits: u32) -> Result<Vec<&'static str>, String> {
    let known = FLAG_BITS
        .iter()
        .fold(UNHARVESTED_BITS, |acc, (b, _)| acc | b);
    if bits & !known != 0 {
        return Err(format!("unexpected flag bits {:#x}", bits & !known));
    }
    Ok(FLAG_BITS
        .iter()
        .filter(|(b, _)| bits & b != 0)
        .map(|(_, n)| *n)
        .collect())
}

/// Pack key from a display name: lowercase ASCII, spaces and hyphens become `_`.
#[must_use]
pub fn key_of(name: &str) -> String {
    name.trim()
        .chars()
        .filter_map(|c| match c {
            ' ' | '-' => Some('_'),
            c if c.is_ascii_alphanumeric() => Some(c.to_ascii_lowercase()),
            _ => None,
        })
        .collect()
}

/// Result of a harvest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Harvest {
    /// `(pack id, (core.toml text, battle.toml text))` for `base`, `ab`, `sod`.
    pub packs: Vec<(&'static str, (String, String))>,
    /// Notes: what was not harvested, cross-check mismatches, hints for E03.
    pub report: Vec<String>,
}

/// A creature prepared for writing.
struct Creature<'a> {
    id: usize,
    key: String,
    row: &'a Row,
    faction: Option<usize>,
    level: u8,
    flags: Vec<&'static str>,
}

/// Runs the harvest.
///
/// # Errors
/// Any inconsistency between inputs and gap-fill (see the module docs); the packs failing to
/// compile.
#[allow(
    clippy::too_many_lines,
    reason = "one linear pipeline, split by comments"
)]
pub fn harvest(inputs: &Inputs, gap: &GapFill) -> Result<Harvest, String> {
    let mut report = Vec::new();
    if inputs.exe.len() != CREATURE_COUNT {
        return Err(format!("executable table has {} records", inputs.exe.len()));
    }
    let sod_rows = creature_rows(&inputs.sod_creatures)?;
    let ab_rows = creature_rows(&inputs.ab_creatures)?;

    // Towns and resources.
    let towns: Vec<String> = inputs
        .towns
        .records
        .iter()
        .map(|r| r[0].trim().to_owned())
        .collect();
    let resources: Vec<String> = inputs
        .resources
        .records
        .iter()
        .take(RESOURCE_COUNT)
        .map(|r| r[0].trim().to_owned())
        .collect();
    if resources.len() != RESOURCE_COUNT {
        return Err(format!(
            "RESTYPES.TXT has {} rows, expected at least {RESOURCE_COUNT}",
            resources.len()
        ));
    }
    for extra in inputs.resources.records.iter().skip(RESOURCE_COUNT) {
        report.push(format!(
            "RESTYPES.TXT: row `{}` is not a resource column of CRTRAITS; not harvested",
            extra[0].trim()
        ));
    }
    let ab_towns: BTreeSet<usize> = gap
        .ab
        .factions
        .iter()
        .map(|n| {
            towns
                .iter()
                .position(|t| t == n)
                .ok_or_else(|| format!("gapfill: unknown town `{n}`"))
        })
        .collect::<Result<_, _>>()?;

    // Terrains and native terrains (E03 T7).
    let terrains: Vec<String> = inputs
        .terrains
        .records
        .iter()
        .map(|r| r[0].trim().to_owned())
        .collect();
    let terrain_key = |name: &str| -> Result<String, String> {
        terrains
            .iter()
            .find(|t| *t == name)
            .map(|t| key_of(t))
            .ok_or_else(|| format!("gapfill: unknown terrain `{name}`"))
    };
    let mut native: BTreeMap<usize, String> = BTreeMap::new();
    for n in &gap.native_terrain {
        let town = towns
            .iter()
            .position(|t| *t == n.faction)
            .ok_or_else(|| format!("gapfill: unknown town `{}`", n.faction))?;
        if native.insert(town, terrain_key(&n.terrain)?).is_some() {
            return Err(format!("gapfill: native terrain of `{}` twice", n.faction));
        }
    }
    // Creatures.
    let by_name = |name: &str| -> Result<usize, String> {
        sod_rows
            .iter()
            .position(|r| r.name == name)
            .ok_or_else(|| format!("gapfill: unknown creature `{name}`"))
    };
    let ab_ids: BTreeSet<usize> = gap
        .ab
        .creatures
        .iter()
        .map(|n| by_name(n))
        .collect::<Result<_, _>>()?;
    let roe: BTreeMap<usize, u8> = gap
        .roe_creature
        .iter()
        .map(|r| Ok((by_name(&r.name)?, r.level)))
        .collect::<Result<_, String>>()?;
    let changes: BTreeMap<usize, Since> = gap
        .table_change
        .iter()
        .map(|c| Ok((by_name(&c.name)?, c.since)))
        .collect::<Result<_, String>>()?;
    // A creature's abilities may come from several entries (one per topic, each with its
    // sources), in file order.
    let mut ability_specs: BTreeMap<usize, Vec<AbilitySpec>> = BTreeMap::new();
    for entry in &gap.creature_abilities {
        let id = by_name(&entry.creature)?;
        ability_specs
            .entry(id)
            .or_default()
            .extend(entry.abilities.iter().cloned());
    }

    let mut creatures = Vec::new();
    let mut keys = BTreeSet::new();
    for (id, (rec, row)) in inputs.exe.iter().zip(&sod_rows).enumerate() {
        let unused = row.name.starts_with("NOT USED");
        if unused != (rec.level < 0) {
            return Err(format!(
                "creature {id} `{}`: table and executable disagree on whether it is used",
                row.name
            ));
        }
        if unused {
            report.push(format!(
                "creature id {id} (`{}`): unused slot, not harvested",
                row.name
            ));
            continue;
        }
        if rec.flags & WAR_MACHINE_BIT != 0 && !gap.war_machines.harvest.contains(&row.name) {
            // Siege machines (Catapult, Arrow Tower) need siege rules (later epic).
            report.push(format!(
                "creature id {id} (`{}`): siege machine, not harvested yet",
                row.name
            ));
            continue;
        }
        let key = key_of(&row.name);
        if !keys.insert(key.clone()) {
            return Err(format!("duplicate creature key `{key}`"));
        }
        let flag_names =
            flags(rec.flags).map_err(|e| format!("creature {id} `{}`: {e}", row.name))?;
        if rec.flags & UNHARVESTED_BITS != 0 {
            report.push(format!(
                "creature `{key}`: unidentified flag bit 0x800 not harvested"
            ));
        }
        for token in row
            .attributes
            .split('|')
            .map(str::trim)
            .filter(|t| !t.is_empty() && *t != "0")
        {
            match ATTRIBUTE_FLAGS.iter().find(|(a, _)| *a == token) {
                Some((_, flag)) if !flag_names.contains(flag) => report.push(format!(
                    "creature `{key}`: attribute column says {token}, executable lacks `{flag}`"
                )),
                Some(_) => {}
                None => match ATTRIBUTE_ABILITIES.iter().find(|(a, _)| *a == token) {
                    Some((_, kinds)) => {
                        let given = ability_specs
                            .get(&id)
                            .is_some_and(|l| l.iter().any(|a| kinds.contains(&a.kind.as_str())));
                        if !given {
                            report.push(format!(
                                "creature `{key}`: attribute column says {token}, gap-fill lacks {kinds:?}"
                            ));
                        }
                    }
                    None => report.push(format!(
                        "creature `{key}`: attribute {token} not harvested (later epic)"
                    )),
                },
            }
        }
        let faction = usize::try_from(rec.town).ok();
        if faction.is_some_and(|t| t >= towns.len()) {
            return Err(format!(
                "creature `{key}`: town index {} has no TownType row",
                rec.town
            ));
        }
        let level =
            u8::try_from(rec.level + 1).map_err(|_| format!("creature `{key}`: bad level"))?;
        creatures.push(Creature {
            id,
            key,
            row,
            faction,
            level,
            flags: flag_names,
        });
    }

    // Classify rows that differ between the AB-era and SoD tables.
    for (id, (sod, ab)) in sod_rows.iter().zip(&ab_rows).enumerate() {
        let differs = !sod.same_values(ab);
        match (differs, changes.get(&id)) {
            (true, None) => {
                return Err(format!(
                    "creature `{}` differs between H3ab_bmp and H3bitmap; classify it in gapfill",
                    sod.name
                ));
            }
            (false, Some(_)) => {
                return Err(format!(
                    "gapfill: table_change `{}` does not differ",
                    sod.name
                ));
            }
            _ => {}
        }
        if sod.ability_text != ab.ability_text {
            report.push(format!(
                "creature `{}`: ability text changed between AB and SoD (abilities: E03)",
                key_of(&sod.name)
            ));
        }
    }

    // Pack membership.
    let base_towns: Vec<usize> = (0..towns.len()).filter(|t| !ab_towns.contains(t)).collect();
    for c in &creatures {
        let in_ab = ab_ids.contains(&c.id);
        if !in_ab && c.faction.is_some_and(|t| ab_towns.contains(&t)) && !roe.contains_key(&c.id) {
            return Err(format!(
                "creature `{}` is RoE but belongs to an AB faction; add a roe_creature entry",
                c.key
            ));
        }
        if in_ab && roe.contains_key(&c.id) {
            return Err(format!(
                "creature `{}` is listed as AB addition and as RoE creature",
                c.key
            ));
        }
    }
    let town_ref = |t: usize, pack: &str| {
        let owner = if ab_towns.contains(&t) { "ab" } else { "base" };
        let k = key_of(&towns[t]);
        if owner == pack {
            k
        } else {
            format!("{owner}:{k}")
        }
    };
    let row_for = |c: &Creature<'_>| -> Row {
        // `since = "sod"`: base/ab keep the AB-era row.
        if changes.get(&c.id) == Some(&Since::Sod) {
            ab_rows[c.id].clone()
        } else {
            c.row.clone()
        }
    };

    // Abilities: render each creature's list for the pack it is written to. A creature reference
    // outside that pack is written as `pack:key`; a `base` creature whose list refers to an `ab`
    // creature gets the list without those entries in `base` and the full list as an `ab`
    // override.
    let harvested: BTreeMap<usize, &str> =
        creatures.iter().map(|c| (c.id, c.key.as_str())).collect();
    // The executable's fire-immunity flag protects against the fire school's spells
    // (installation.md §7.8); the gap-fill names that school for exactly those creatures.
    for c in &creatures {
        let flagged = c.flags.contains(&"fire_immune");
        let listed = ability_specs.get(&c.id).is_some_and(|l| {
            l.iter().any(|a| {
                a.kind == "school_immunity"
                    && a.params.get("school").and_then(toml::Value::as_str) == Some("base:fire")
            })
        });
        if flagged != listed {
            return Err(format!(
                "creature `{}`: flag fire_immune and the gap-fill's fire school immunity disagree",
                c.key
            ));
        }
    }
    if let Some(id) = ability_specs.keys().find(|id| !harvested.contains_key(id)) {
        return Err(format!(
            "gapfill: abilities for `{}`, which is not harvested",
            sod_rows[*id].name
        ));
    }
    let owner = |id: usize| if ab_ids.contains(&id) { "ab" } else { "base" };
    let render = |spec: &AbilitySpec, pack: &str| -> Result<String, String> {
        let mut parts = vec![format!("kind = {}", quote(&spec.kind))];
        if let Some(name) = &spec.creature {
            let target = by_name(name)?;
            let key = harvested
                .get(&target)
                .ok_or_else(|| format!("gapfill: ability refers to `{name}`, not harvested"))?;
            let r = if owner(target) == pack {
                (*key).to_owned()
            } else {
                format!("{}:{key}", owner(target))
            };
            parts.push(format!("creature = {}", quote(&r)));
        }
        for (k, v) in &spec.params {
            parts.push(format!("{k} = {v}"));
        }
        Ok(format!("{{ {} }}", parts.join(", ")))
    };
    let refers_to_ab = |spec: &AbilitySpec| {
        spec.creature
            .as_deref()
            .and_then(|n| by_name(n).ok())
            .is_some_and(|t| ab_ids.contains(&t))
    };
    let ability_line = |specs: &[AbilitySpec], pack: &str| -> Result<String, String> {
        let items = specs
            .iter()
            .map(|a| render(a, pack))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(format!("abilities = [{}]", items.join(", ")))
    };
    // An ability naming a `sod` spell (Titan's Lightning Bolt) cannot be written before `sod`:
    // the earlier packs get the list without it, `sod` the full list as an override.
    let refers_to_sod = |spec: &AbilitySpec| {
        spec.params
            .values()
            .any(|v| v.as_str().is_some_and(|s| s.starts_with("sod:")))
    };
    let mut base_abilities: BTreeMap<usize, String> = BTreeMap::new();
    let mut ab_abilities: BTreeMap<usize, String> = BTreeMap::new();
    let mut ab_ability_overrides: BTreeMap<usize, String> = BTreeMap::new();
    let mut sod_ability_overrides: BTreeMap<usize, String> = BTreeMap::new();
    for (&id, all) in &ability_specs {
        let earlier: Vec<AbilitySpec> = all
            .iter()
            .filter(|a| !refers_to_sod(a))
            .map(AbilitySpec::clone)
            .collect();
        if earlier.len() < all.len() {
            sod_ability_overrides.insert(id, ability_line(all, "sod")?);
        }
        let specs = &earlier;
        if ab_ids.contains(&id) {
            ab_abilities.insert(id, ability_line(specs, "ab")?);
        } else if specs.iter().any(&refers_to_ab) {
            let roe: Vec<AbilitySpec> = specs
                .iter()
                .filter(|a| !refers_to_ab(a))
                .map(AbilitySpec::clone)
                .collect();
            base_abilities.insert(id, ability_line(&roe, "base")?);
            ab_ability_overrides.insert(id, ability_line(specs, "ab")?);
        } else {
            base_abilities.insert(id, ability_line(specs, "base")?);
        }
    }

    // Hero content (E04a T3) and the creature upgrade relation.
    let pack_of = |id: usize| if ab_ids.contains(&id) { "ab" } else { "base" };
    let creature_refs: BTreeMap<usize, CreatureRef> = creatures
        .iter()
        .map(|c| {
            let r = CreatureRef {
                key: c.key.clone(),
                pack: pack_of(c.id),
                war_machine: c.flags.contains(&"war_machine"),
                name: c.row.name.clone(),
            };
            (c.id, r)
        })
        .collect();
    let town_refs: Vec<(String, &'static str)> = towns
        .iter()
        .enumerate()
        .map(|(t, n)| (key_of(n), if ab_towns.contains(&t) { "ab" } else { "base" }))
        .collect();
    let upgrades = heroes::upgrades(&inputs.exe, &creature_refs, gap)?;
    // Special battlefields and obstacle types (E04c T2b).
    let fields = battlefields::write(&inputs.obstacles, &gap.battlefield, &terrains, &town_refs)?;
    let ctx = Context {
        creatures: &creature_refs,
        towns: &town_refs,
    };
    let hero_packs = heroes::harvest_heroes(
        &inputs.hero_tables,
        &inputs.hero_data,
        gap,
        ctx,
        &mut report,
    )?;
    let artifact_packs = artifacts::harvest_artifacts(
        &inputs.artifact_tables,
        &inputs.hero_data.artifact_primary,
        &heroes::skill_names(&inputs.hero_tables)?,
        gap,
        ctx,
        &mut report,
    )?;
    // Upgrades a creature's own pack can express, and the full list where a `base` creature
    // upgrades into an `ab` creature (written as an `ab` override).
    let upgrade_list = |id: usize, pack: &str, all: bool| -> Vec<String> {
        upgrades
            .get(&id)
            .into_iter()
            .flatten()
            .filter(|&&u| all || pack_of(u) == pack || pack_of(u) == "base")
            .map(|&u| {
                let key = &creature_refs[&u].key;
                if pack_of(u) == pack {
                    key.clone()
                } else {
                    format!("{}:{key}", pack_of(u))
                }
            })
            .collect()
    };
    let upgrades_later = |id: usize| {
        pack_of(id) == "base"
            && upgrades
                .get(&id)
                .is_some_and(|l| l.iter().any(|&u| pack_of(u) == "ab"))
    };

    // Data quirks (E04b T8e; ADR-0007): an upgrade the executable gives another level than its
    // base creature keeps it, and an override gated by the quirk's flag gives it the base's.
    let mut level_fixes: BTreeMap<&str, String> = BTreeMap::new();
    for fix in &gap.data_quirks.base_level {
        for name in &fix.creatures {
            let id = by_name(name)?;
            let level = |i: usize| creatures.iter().find(|c| c.id == i).map(|c| c.level);
            let base = upgrades
                .iter()
                .find(|(_, list)| list.contains(&id))
                .map(|(&b, _)| b)
                .ok_or_else(|| format!("gapfill: data quirk: `{name}` is no upgrade"))?;
            let (own, fixed) = (level(id), level(base));
            let fixed = fixed
                .filter(|&l| Some(l) != own)
                .ok_or_else(|| format!("gapfill: data quirk: `{name}` has its base's level"))?;
            let pack = pack_of(id);
            let _ = write!(
                level_fixes.entry(pack).or_default(),
                "\n[[creature_override]]\ntarget = \"{pack}:{}\"  # {name}, BAT-QUIRK-19\n\
                 fixes = \"{}\"\nlevel = {fixed}\n",
                creature_refs[&id].key,
                fix.flag
            );
        }
    }

    let header = |id: &str, depends: &[&str], what: &str| {
        let mut s = String::new();
        let _ = writeln!(
            s,
            "# GENERATED by h3-harvest (E02) from the user's installation (Complete 4.0) and"
        );
        let _ = writeln!(s, "# crates/h3-harvest/data/gapfill.toml. {what}");
        let _ = writeln!(
            s,
            "# Do not edit by hand: change the harvester or the gap-fill file and re-run it"
        );
        let _ = writeln!(
            s,
            "# (`cargo run -p h3-harvest -- --install original-data/install --out packs`)."
        );
        let _ = writeln!(
            s,
            "# Gameplay data and names only (OPEN-QUESTIONS Q13); sources: docs/formats/installation.md."
        );
        let _ = writeln!(s);
        let _ = writeln!(s, "[pack]");
        let _ = writeln!(s, "id = \"{id}\"");
        let _ = writeln!(s, "version = \"harvest-1\"");
        let deps: Vec<String> = depends.iter().map(|d| format!("\"{d}\"")).collect();
        let _ = writeln!(s, "depends = [{}]", deps.join(", "));
        s
    };

    // base
    let mut base = header(
        "base",
        &[],
        "The Restoration of Erathia, latest patch (Q15).",
    );
    for r in &resources {
        let _ = write!(
            base,
            "\n[[resource]]\nid = \"{}\"\nname = {}\n",
            key_of(r),
            quote(r)
        );
    }
    for t in &terrains {
        let _ = write!(
            base,
            "\n[[terrain]]\nid = \"{}\"\nname = {}\n",
            key_of(t),
            quote(t)
        );
    }
    let faction_entry = |out: &mut String, t: usize, terrain_pack: &str| {
        let _ = write!(
            out,
            "\n[[faction]]\nid = \"{}\"\nname = {}\n",
            key_of(&towns[t]),
            quote(&towns[t])
        );
        if let Some(k) = native.get(&t) {
            let _ = writeln!(out, "native_terrain = \"{terrain_pack}{k}\"");
        }
    };
    for &t in &base_towns {
        faction_entry(&mut base, t, "");
    }
    for c in creatures.iter().filter(|c| !ab_ids.contains(&c.id)) {
        let (faction, level) = match roe.get(&c.id) {
            Some(&level) => (None, level),
            None => (c.faction.map(|t| town_ref(t, "base")), c.level),
        };
        write_creature(
            &mut base,
            c,
            &row_for(c),
            faction.as_deref(),
            level,
            &resources,
            "base",
            base_abilities.get(&c.id).map(String::as_str),
            &upgrade_list(c.id, "base", false),
        );
    }
    base.push_str(level_fixes.get("base").map_or("", String::as_str));
    base.push_str(&hero_packs.base);
    base.push_str(&artifact_packs.base);
    base.push_str(&fields.base);
    // Sieges (E04c T5): the records, the field in `[battle]`.
    let shots = sieges::wall_shots(
        &inputs.wall_shots,
        &inputs.siege.walls,
        (&inputs.wall_tables.0, &inputs.wall_tables.1),
        town_refs.len(),
    )?;
    let sieges = sieges::write((&inputs.siege, &shots), &town_refs, &hero_packs.spell_keys)?;
    base.push_str(&sieges.base);
    write_battle(&mut base, gap);
    base.push_str(&sieges.field);
    write_hero_params(&mut base, gap, &hero_packs.skills)?;
    base.push_str(
        "\n[[profile]]\nid = \"faithful\"        # default: preserve/fix-optional quirks on\n\
         quirks = { \"quirk.battle.no_luck_without_hero\" = true, \
         \"quirk.battle.single_precision_rates\" = true, \
         \"quirk.battle.unlimited_ballista_shots\" = true, \
         \"quirk.battle.first_protection\" = true, \
         \"quirk.battle.counterstrike_extra_round\" = true, \
         \"quirk.battle.frenzy_reduced_defense\" = true, \
         \"quirk.battle.fire_shield_first_stage\" = true, \
         \"quirk.battle.early_mass_version\" = true, \
         \"quirk.battle.mass_cure_skips_hypnotized\" = true, \
         \"quirk.battle.obstacle_anchors\" = true, \
         \"quirk.battle.draws_avoid_hidden\" = true, \
         \"quirk.battle.sacrifice_tests_as_resurrection\" = true, \
         \"quirk.battle.clone_on_its_original\" = true, \
         \"quirk.battle.periodic_first_stack\" = true, \
         \"quirk.battle.no_defender_ballista_at_fort\" = true, \
         \"quirk.battle.berserk_walks_to_position_hex\" = true, \
         \"quirk.battle.original_ai_slips\" = true, \
         \"quirk.content.roland_crusader_specialty\" = true, \
         \"quirk.content.elemental_levels\" = true, \
         \"quirk.content.elementalist_offer_intervals\" = true, \
         \"quirk.adventure.diagonal_check_ignores_roads\" = true, \
         \"quirk.adventure.diagonal_check_ignores_native\" = true, \
         \"quirk.adventure.native_skips_leading_neutrals\" = true, \
         \"quirk.adventure.fringe_shows_objects\" = true, \
         \"quirk.adventure.obstacles_from_tile\" = true, \
         \"quirk.adventure.split_from_tile\" = true, \
         \"quirk.adventure.level_up_from_seed\" = true, \
         \"quirk.adventure.drop_ignores_artifacts\" = true, \
         \"quirk.adventure.radius_waits_for_a_step\" = true }    # Q-0003, Q-0005, Q-0004, Q-0006, Q-0007, Q-0008, Q-0009, Q-0010, Q-0011, Q-0012, BAT-QUIRK-3 to BAT-QUIRK-5, BAT-QUIRK-7, BAT-QUIRK-13, BAT-QUIRK-19, BAT-QUIRK-23, BAT-QUIRK-24, BAT-QUIRK-25, ADV-QUIRK-1 to ADV-QUIRK-6, ADV-QUIRK-9, ADV-QUIRK-10, ADV-QUIRK-17, ADV-QUIRK-18\n",
    );
    base.push_str(
        "\n[[profile]]\nid = \"strict-fixed\"    # all quirks fixed\n\
         quirks = { \"quirk.battle.no_luck_without_hero\" = false, \
         \"quirk.battle.single_precision_rates\" = false, \
         \"quirk.battle.unlimited_ballista_shots\" = false, \
         \"quirk.battle.first_protection\" = false, \
         \"quirk.battle.counterstrike_extra_round\" = false, \
         \"quirk.battle.frenzy_reduced_defense\" = false, \
         \"quirk.battle.fire_shield_first_stage\" = false, \
         \"quirk.battle.early_mass_version\" = false, \
         \"quirk.battle.mass_cure_skips_hypnotized\" = false, \
         \"quirk.battle.obstacle_anchors\" = false, \
         \"quirk.battle.draws_avoid_hidden\" = false, \
         \"quirk.battle.sacrifice_tests_as_resurrection\" = false, \
         \"quirk.battle.clone_on_its_original\" = false, \
         \"quirk.battle.periodic_first_stack\" = false, \
         \"quirk.battle.no_defender_ballista_at_fort\" = false, \
         \"quirk.battle.berserk_walks_to_position_hex\" = false, \
         \"quirk.battle.original_ai_slips\" = false, \
         \"quirk.content.roland_crusader_specialty\" = false, \
         \"quirk.content.elemental_levels\" = false, \
         \"quirk.content.elementalist_offer_intervals\" = false, \
         \"quirk.adventure.diagonal_check_ignores_roads\" = false, \
         \"quirk.adventure.diagonal_check_ignores_native\" = false, \
         \"quirk.adventure.native_skips_leading_neutrals\" = false, \
         \"quirk.adventure.fringe_shows_objects\" = false, \
         \"quirk.adventure.obstacles_from_tile\" = false, \
         \"quirk.adventure.split_from_tile\" = false, \
         \"quirk.adventure.level_up_from_seed\" = false, \
         \"quirk.adventure.drop_ignores_artifacts\" = false, \
         \"quirk.adventure.radius_waits_for_a_step\" = false }   # Q-0003, Q-0005, Q-0004, Q-0006, Q-0007, Q-0008, Q-0009, Q-0010, Q-0011, Q-0012, BAT-QUIRK-3 to BAT-QUIRK-5, BAT-QUIRK-7, BAT-QUIRK-13, BAT-QUIRK-19, BAT-QUIRK-23, BAT-QUIRK-24, BAT-QUIRK-25, ADV-QUIRK-1 to ADV-QUIRK-6, ADV-QUIRK-9, ADV-QUIRK-10, ADV-QUIRK-17, ADV-QUIRK-18\n",
    );

    // ab
    let mut ab = header("ab", &["base"], "Armageddon's Blade, latest patch (Q15).");
    for &t in &ab_towns {
        faction_entry(&mut ab, t, "base:");
    }
    for c in creatures.iter().filter(|c| ab_ids.contains(&c.id)) {
        let faction = c.faction.map(|t| town_ref(t, "ab"));
        write_creature(
            &mut ab,
            c,
            &row_for(c),
            faction.as_deref(),
            c.level,
            &resources,
            "ab",
            ab_abilities.get(&c.id).map(String::as_str),
            &upgrade_list(c.id, "ab", true),
        );
    }
    for c in creatures.iter().filter(|c| {
        roe.contains_key(&c.id) || ab_ability_overrides.contains_key(&c.id) || upgrades_later(c.id)
    }) {
        let _ = write!(ab, "\n[[creature_override]]\ntarget = \"base:{}\"\n", c.key);
        if roe.contains_key(&c.id) {
            let faction = c.faction.map_or_else(String::new, |t| town_ref(t, "ab"));
            let _ = writeln!(ab, "faction = \"{faction}\"");
            let _ = writeln!(ab, "level = {}", c.level);
        }
        if let Some(line) = ab_ability_overrides.get(&c.id) {
            let _ = writeln!(ab, "{line}");
        }
        if upgrades_later(c.id) {
            let list: Vec<String> = upgrade_list(c.id, "ab", true)
                .iter()
                .map(|k| format!("\"{k}\""))
                .collect();
            let _ = writeln!(ab, "upgrades = [{}]", list.join(", "));
        }
    }
    ab.push_str(level_fixes.get("ab").map_or("", String::as_str));
    ab.push_str(&hero_packs.ab);
    ab.push_str(&artifact_packs.ab);
    ab.push_str(&sieges.ab);

    // sod
    let mut sod = header("sod", &["ab"], "The Shadow of Death: changes on top of AB.");
    for c in creatures.iter().filter(|c| {
        changes.get(&c.id) == Some(&Since::Sod) || sod_ability_overrides.contains_key(&c.id)
    }) {
        let owner = if ab_ids.contains(&c.id) { "ab" } else { "base" };
        let _ = write!(
            sod,
            "\n[[creature_override]]\ntarget = \"{owner}:{}\"\n",
            c.key
        );
        if changes.get(&c.id) == Some(&Since::Sod) {
            write_row_diff(&mut sod, &ab_rows[c.id], c.row, &resources);
        }
        if let Some(line) = sod_ability_overrides.get(&c.id) {
            let _ = writeln!(sod, "{line}");
        }
    }
    sod.push_str(&hero_packs.sod);
    sod.push_str(&artifact_packs.sod);
    sod.push_str(&fields.sod);

    // Each pack splits into its core and battle files (pack format v35, R01).
    let (base, ab, sod) = (split(&base), split(&ab), split(&sod));
    // The output must compile in every allowed combination, and SoD without AB must be rejected.
    let profile = "base:faithful";
    fn pair(p: &(String, String)) -> (&str, &str) {
        (p.0.as_str(), p.1.as_str())
    }
    for set in [&[&base][..], &[&base, &ab], &[&base, &ab, &sod]] {
        let texts: Vec<(&str, &str)> = set.iter().map(|p| pair(p)).collect();
        for p in [profile, "base:strict-fixed"] {
            h3_pack_battle::compile(&texts, p)
                .map_err(|e| format!("generated packs do not compile ({p}): {e}"))?;
        }
    }
    if h3_pack_battle::compile(&[pair(&base), pair(&sod)], profile).is_ok() {
        return Err("generated sod pack compiles without ab".into());
    }
    report.push("CRTRAITS ability-text column: flavor text, not harvested (Q13)".into());
    Ok(Harvest {
        packs: vec![("base", base), ("ab", ab), ("sod", sod)],
        report,
    })
}

/// TOML basic string.
pub(crate) fn quote(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c if c.is_control() => {
                let _ = write!(out, "\\u{:04X}", u32::from(c));
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn cost_table(cost: &[u32; RESOURCE_COUNT], resources: &[String], pack: &str) -> String {
    let entries: Vec<String> = cost
        .iter()
        .zip(resources)
        .filter(|(n, _)| **n != 0)
        .map(|(n, r)| {
            let k = key_of(r);
            if pack == "base" {
                format!("{k} = {n}")
            } else {
                format!("\"base:{k}\" = {n}")
            }
        })
        .collect();
    format!("{{ {} }}", entries.join(", "))
}

#[allow(clippy::too_many_arguments, reason = "internal writer")]
fn write_creature(
    out: &mut String,
    c: &Creature<'_>,
    row: &Row,
    faction: Option<&str>,
    level: u8,
    resources: &[String],
    pack: &str,
    abilities: Option<&str>,
    upgrades: &[String],
) {
    let _ = write!(
        out,
        "\n[[creature]]\nid = \"{}\"  # original id {}\n",
        c.key, c.id
    );
    let _ = writeln!(out, "name = {}", quote(&row.name));
    let _ = writeln!(out, "plural = {}", quote(&row.plural));
    if let Some(f) = faction {
        let _ = writeln!(out, "faction = \"{f}\"");
    }
    let _ = writeln!(out, "level = {level}");
    let _ = writeln!(out, "attack = {}", row.attack);
    let _ = writeln!(out, "defense = {}", row.defense);
    let _ = writeln!(out, "damage = [{}, {}]", row.damage[0], row.damage[1]);
    let _ = writeln!(out, "hit_points = {}", row.hit_points);
    let _ = writeln!(out, "speed = {}", row.speed);
    let _ = writeln!(out, "shots = {}", row.shots);
    let _ = writeln!(out, "casts = {}", row.casts);
    let _ = writeln!(out, "cost = {}", cost_table(&row.cost, resources, pack));
    let _ = writeln!(out, "growth = {}", row.growth);
    let _ = writeln!(out, "horde_growth = {}", row.horde_growth);
    let _ = writeln!(out, "fight_value = {}", row.fight_value);
    let _ = writeln!(out, "ai_value = {}", row.ai_value);
    let _ = writeln!(
        out,
        "adventure_count = [{}, {}]",
        row.adventure_count[0], row.adventure_count[1]
    );
    let flags: Vec<String> = c.flags.iter().map(|f| format!("\"{f}\"")).collect();
    let _ = writeln!(out, "flags = [{}]", flags.join(", "));
    if let Some(line) = abilities {
        let _ = writeln!(out, "{line}");
    }
    if !upgrades.is_empty() {
        let list: Vec<String> = upgrades.iter().map(|k| format!("\"{k}\"")).collect();
        let _ = writeln!(out, "upgrades = [{}]", list.join(", "));
    }
}

fn write_row_diff(out: &mut String, old: &Row, new: &Row, resources: &[String]) {
    macro_rules! field {
        ($f:ident) => {
            if old.$f != new.$f {
                let _ = writeln!(out, "{} = {}", stringify!($f), new.$f);
            }
        };
    }
    if old.name != new.name {
        let _ = writeln!(out, "name = {}", quote(&new.name));
    }
    if old.plural != new.plural {
        let _ = writeln!(out, "plural = {}", quote(&new.plural));
    }
    field!(attack);
    field!(defense);
    if old.damage != new.damage {
        let _ = writeln!(out, "damage = [{}, {}]", new.damage[0], new.damage[1]);
    }
    field!(hit_points);
    field!(speed);
    field!(shots);
    field!(casts);
    if old.cost != new.cost {
        let _ = writeln!(out, "cost = {}", cost_table(&new.cost, resources, "sod"));
    }
    field!(growth);
    field!(horde_growth);
    field!(fight_value);
    field!(ai_value);
    if old.adventure_count != new.adventure_count {
        let _ = writeln!(
            out,
            "adventure_count = [{}, {}]",
            new.adventure_count[0], new.adventure_count[1]
        );
    }
}

/// Writes `[hero_params]`; the offered skills are named in the gap-fill file and written as
/// the keys of `skills` (key, name).
fn write_hero_params(
    out: &mut String,
    gap: &GapFill,
    skills: &[(String, String)],
) -> Result<(), String> {
    let p = &gap.heroes.params;
    let [a, d, s, k] = p.primary_min;
    let skill = |name: &str| {
        skills
            .iter()
            .find(|(_, n)| n == name)
            .map(|(key, _)| format!("\"{key}\""))
            .ok_or_else(|| format!("gapfill: hero params: unknown skill `{name}`"))
    };
    let experience: Vec<String> = p.experience.iter().map(u32::to_string).collect();
    let schools = p
        .schools
        .iter()
        .map(|n| skill(n))
        .collect::<Result<Vec<_>, _>>()?;
    let _ = write!(
        out,
        "\n# Hero parameters (Tier 2): rule ids in docs/rules/catalog/heroes.toml (sources in the\n\
         # gap-fill file).\n[hero_params]\nprimary_min = [{a}, {d}, {s}, {k}]\nprimary_max = {}\n\
         backpack = {}\nlearn_level = {}\nexperience = [{}]\nmax_level = {}\nwisdom = {}\n\
         schools = [{}]\n",
        p.primary_max,
        p.backpack,
        p.learn_level,
        experience.join(", "),
        p.max_level,
        skill(&p.wisdom)?,
        schools.join(", ")
    );
    Ok(())
}

fn write_battle(out: &mut String, gap: &GapFill) {
    let b = &gap.battle;
    let _ = write!(
        out,
        "\n# Battle parameters (Tier 2): rule ids in docs/rules/catalog/battle.toml; the formations\n\
         # are the original's loose and tight formations (sources in the gap-fill file).\n[battle]\n"
    );
    let _ = writeln!(out, "field_width = {}", b.field_width);
    let _ = writeln!(out, "field_height = {}", b.field_height);
    let _ = writeln!(out, "side_columns = {}", b.side_columns);
    let _ = writeln!(out, "army_slots = {}", b.army_slots);
    let _ = writeln!(out, "damage_roll_samples = {}", b.damage_roll_samples);
    let _ = writeln!(
        out,
        "attack_bonus_per_point_percent = {}",
        b.attack_bonus_per_point_percent
    );
    let _ = writeln!(
        out,
        "attack_bonus_cap_percent = {}",
        b.attack_bonus_cap_percent
    );
    let _ = writeln!(
        out,
        "defense_reduction_per_point_permille = {}",
        b.defense_reduction_per_point_permille
    );
    let _ = writeln!(
        out,
        "defense_reduction_cap_permille = {}",
        b.defense_reduction_cap_permille
    );
    let _ = writeln!(out, "defend_bonus_percent = {}", b.defend_bonus_percent);
    let _ = writeln!(out, "defend_bonus_min = {}", b.defend_bonus_min);
    let _ = writeln!(
        out,
        "ranged_full_damage_distance = {}",
        b.ranged_full_damage_distance
    );
    let _ = writeln!(out, "ranged_penalty_percent = {}", b.ranged_penalty_percent);
    let _ = writeln!(out, "melee_penalty_percent = {}", b.melee_penalty_percent);
    let list = |v: &[u8]| v.iter().map(u8::to_string).collect::<Vec<_>>().join(", ");
    let _ = writeln!(out, "morale_dice = {}", b.morale_dice);
    let _ = writeln!(
        out,
        "good_morale_chances = [{}]",
        list(&b.good_morale_chances)
    );
    let _ = writeln!(
        out,
        "bad_morale_chances = [{}]",
        list(&b.bad_morale_chances)
    );
    let _ = writeln!(out, "luck_dice = {}", b.luck_dice);
    let _ = writeln!(out, "good_luck_chances = [{}]", list(&b.good_luck_chances));
    let positions: Vec<String> = b
        .war_machine_positions
        .iter()
        .map(|[x, y]| format!("[{x}, {y}]"))
        .collect();
    let _ = writeln!(out, "war_machine_positions = [{}]", positions.join(", "));
    let rows = |table: &[Vec<u8>]| -> String {
        table
            .iter()
            .map(|r| format!("[{}]", list(r)))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let _ = writeln!(out, "loose_formation = [{}]", rows(&b.loose_formation));
    let _ = writeln!(out, "tight_formation = [{}]", rows(&b.tight_formation));
    let _ = writeln!(
        out,
        "tactics_zone_columns = [{}]",
        list(&b.tactics_zone_columns)
    );
    let _ = writeln!(
        out,
        "surrender = {{ resource = \"{}\", cost_percent = {} }}",
        b.surrender.resource, b.surrender.cost_percent
    );
    let _ = writeln!(out, "hero_casts_per_round = {}", b.hero_casts_per_round);
    let _ = writeln!(out, "side_stack_limit = {}", b.side_stack_limit);
}

#[cfg(test)]
mod tests;
