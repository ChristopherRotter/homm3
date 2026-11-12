//! The town part's files (E09 T5b2b2; design `docs/epics/E09-design.md` §3, the epic's decisions
//! 23 and 27 to 30; survey `docs/rules/surveys/E09-towns.md` §16): the `towns.toml` of `base` (the
//! RoE factions' records and the `[town_params]`), `ab` (Conflux's record) and `sod` (the moats as
//! SoD plays them, overrides), from the executable's town tables (`h3_formats::exe::towns`,
//! `docs/formats/installation.md` §7.61), the buildings' names (`BldgNeut.txt`, `BldgSpec.txt`,
//! `Dwelling.txt`; names only, Q13) and the gap-fill's `[towns]`.
//!
//! A record holds the faction's listed buildings and the Village Hall (not the hall companions 23,
//! 27 to 29 of Rampart, Necropolis and Conflux, which no list holds, nor the ship 20: decision 27);
//! each building's `supersedes` as the common and the faction's supersede lists give them
//! (direct); each upgraded horde forbidden with its horde and built once its horde and its level's
//! upgraded dwelling are (decision 29); the roles (the dwellings' by their ids, the others from
//! the gap-fill); the seven levels with their upgrades; the Resource Silo's resources. Keys come
//! from the names; an upgraded horde, named as its horde, gets the prefix `upg_` (the upgraded
//! dwellings' "Upg." names give theirs). Since E09 T16b (design §25; survey
//! `docs/rules/surveys/E09-construction.md` §5, §6): each building's `requires`, its direct
//! requirements in the faction's list order; its `cost` from `Building.txt` as
//! `building_costs_load` 0x5C14C0 reads it ([`cost_row`]; the zero columns dropped, the Village
//! Hall's 1 gold kept); the role `shipyard` on the building [`SHIPYARD`]. Since E09 T25a (design
//! §38; survey `docs/rules/surveys/E09-growth.md` §1): each horde's role `horde` from the horde
//! table (its level by its creature's, upgraded for the upgrades' buildings), its growth checked
//! equal to its creature's `horde_growth`, which the engine reads (one fact, held once); the gap-fill's
//! roles `portal` and `week_of`; `roe = false` on the `ab` pack's records; the growth's
//! parameters from the gap-fill.
//!
//! The files are compiled with `h3-pack-towns` on the harvested core files in every allowed
//! combination before they are returned.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use h3_formats::exe::towns::{FactionTowns, HORDE_RECORDS, TOWN_CREATURES, TownData};
use h3_formats::text::TextTable;

use crate::gapfill::{GapFill, TownSiege, Towns};
use crate::{Inputs, RESOURCE_COUNT, creature_rows, key_of, quote};

/// The buildings' text tables of `H3bitmap.lod`: the names (survey §16.7) and the costs.
#[derive(Debug, Clone)]
pub struct Tables {
    /// `BldgNeut.txt`: the ids 0 to 14 by row, the Blacksmith (16) in row 17.
    pub neutral: TextTable,
    /// `BldgSpec.txt`: [`SPECIAL_ROWS`] rows a faction, the ids of [`SPECIAL_ORDER`].
    pub special: TextTable,
    /// `Dwelling.txt`: [`DWELLING_ROWS`] rows a faction, the dwellings 30 to 43.
    pub dwellings: TextTable,
    /// `Building.txt`: the costs, seven columns in `RESTYPES.TXT`'s order, its rows as
    /// [`cost_row`] picks them (E09 T16b).
    pub costs: TextTable,
}

/// The Village Hall: in no faction's list, every town's from the start (§16.1, §5.3).
pub const VILLAGE_HALL: u8 = 10;
/// The ids no record holds, and no list may: the ship, the hall companions (decision 27).
pub const LEFT_OUT: [u8; 4] = [20, 27, 28, 29];
/// The Shipyard: the building with the role `shipyard` (design §25, decision 46).
pub const SHIPYARD: u8 = 6;
/// The first of a faction's special buildings, 17 to 25 in `Building.txt`'s faction rows.
const FIRST_SPECIAL: u8 = 17;
/// `Building.txt`'s first row (from 0) of the first faction's special buildings 17 to 25.
const SPECIAL_COSTS: usize = 4;
/// `Building.txt`'s rows from one faction's special buildings to the next's: nine, then two.
const SPECIAL_COSTS_STRIDE: usize = 11;
/// `Building.txt`'s first row of the neutral buildings 0 to 16, the same for every faction.
const NEUTRAL_COSTS: usize = 104;
/// `Building.txt`'s first row of the first faction's dwellings 30 to 43.
const DWELLING_COSTS: usize = 125;
/// `Building.txt`'s rows from one faction's dwellings to the next's: fourteen, then two.
const DWELLING_COSTS_STRIDE: usize = 16;
/// The Blacksmith, named by `BldgNeut.txt`'s row [`BLACKSMITH_ROW`].
const BLACKSMITH: u8 = 16;
/// `BldgNeut.txt`'s row (from 0) of the Blacksmith.
const BLACKSMITH_ROW: usize = 16;
/// The ids `BldgNeut.txt` names by its rows 0 to 14.
const NEUTRAL_IDS: u8 = 15;
/// The ids of a faction's `BldgSpec.txt` rows, in their order.
pub const SPECIAL_ORDER: [u8; 11] = [17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 15];
/// `BldgSpec.txt`'s rows a faction.
pub const SPECIAL_ROWS: usize = SPECIAL_ORDER.len();
/// The level-1 dwelling; the level's dwelling is this + the level's index.
const DWELLING: u8 = 30;
/// The upgraded level-1 dwelling.
const UPGRADED_DWELLING: u8 = 37;
/// Levels of a faction.
const LEVELS: u8 = 7;
/// `Dwelling.txt`'s rows a faction.
pub const DWELLING_ROWS: usize = 2 * LEVELS as usize;
/// The role fields, in the order a building's record writes them (`h3-pack-towns`' table).
const ROLES: [&str; 15] = [
    "hall",
    "fort",
    "tavern",
    "silo",
    "grail",
    "shipyard",
    "horde",
    "portal",
    "week_of",
    "dwelling",
    "morale",
    "luck",
    "escape_tunnel",
    "defender",
    "reveal",
];
/// The packs' files, in load order, with their header's line.
const PACKS: [(&str, &str); 3] = [
    (
        "base",
        "The Restoration of Erathia's town records and the town part's parameters.",
    ),
    (
        "ab",
        "Armageddon's Blade's town record, Conflux, and its Visions creature, the Rogue.",
    ),
    (
        "sod",
        "The Shadow of Death: the moats as SoD plays them (overrides).",
    ),
];

/// A key as a pack's file writes it: bare in its own pack, `pack:key` in another.
fn reference(own: &str, key: &str, file: &str) -> Result<String, String> {
    if own == file {
        Ok(key.to_owned())
    } else if file == "base" {
        Err(format!("towns: `{own}:{key}` is outside base's closure"))
    } else {
        Ok(format!("{own}:{key}"))
    }
}

/// A table's first column at `row`, trimmed, or an error naming the table.
fn cell<'a>(table: &'a TextTable, row: usize, what: &str) -> Result<&'a str, String> {
    let name = table
        .records
        .get(row)
        .and_then(|r| r.first())
        .map_or("", |c| c.trim());
    if name.is_empty() {
        Err(format!("{what}: no name in row {}", row + 1))
    } else {
        Ok(name)
    }
}

/// A building's name: `BldgNeut.txt` for the common ids, `BldgSpec.txt` for the faction's
/// specials and the Resource Silo, `Dwelling.txt` for the dwellings (survey §16.7).
fn building_name(tables: &Tables, faction: usize, id: u8) -> Result<&str, String> {
    if id < NEUTRAL_IDS && id != 15 {
        cell(&tables.neutral, usize::from(id), "BldgNeut.txt")
    } else if id == BLACKSMITH {
        cell(&tables.neutral, BLACKSMITH_ROW, "BldgNeut.txt")
    } else if let Some(k) = SPECIAL_ORDER.iter().position(|&s| s == id) {
        cell(&tables.special, faction * SPECIAL_ROWS + k, "BldgSpec.txt")
    } else if (DWELLING..DWELLING + 2 * LEVELS).contains(&id) {
        cell(
            &tables.dwellings,
            faction * DWELLING_ROWS + usize::from(id - DWELLING),
            "Dwelling.txt",
        )
    } else {
        Err(format!("towns: building id {id} has no name table"))
    }
}

/// What every record needs beside its faction's tables.
struct Ctx<'a> {
    spec: &'a Towns,
    tables: &'a Tables,
    data: &'a TownData,
    /// Faction names (`TownType.txt`).
    factions: &'a [String],
    /// A creature id's key and pack.
    creatures: BTreeMap<u16, (String, &'static str)>,
    /// A creature id's horde growth (`CRTRAITS.TXT`), which a horde's record must match.
    horde_growth: BTreeMap<u16, u16>,
    /// The resources' keys, in `RESTYPES.TXT`'s order (the silo's).
    resources: Vec<String>,
    /// The merged roles by faction (index) and building.
    roles: BTreeMap<(usize, u8), BTreeMap<String, toml::Value>>,
}

/// Runs the town harvest: `(pack id, towns.toml text)` for `base`, `ab` and `sod`, checked to
/// compile on the core files of `packs` (a [`crate::Harvest`]'s) in every allowed combination.
///
/// # Errors
/// No `[towns]` in the gap-fill; the tables, the names and the gap-fill disagreeing (a count, a
/// missing name or building, a key twice, a slot twice, a horde without its dwelling); the files
/// failing to compile.
pub fn harvest_towns(
    inputs: &Inputs,
    gap: &GapFill,
    packs: &[(&'static str, (String, String))],
    report: &mut Vec<String>,
) -> Result<Vec<(&'static str, String)>, String> {
    let spec = gap
        .towns
        .as_ref()
        .ok_or("gapfill: no [towns] (the town harvest's facts)")?;
    let data = &inputs.town_tables;
    let factions: Vec<String> = inputs
        .towns
        .records
        .iter()
        .map(|r| r[0].trim().to_owned())
        .collect();
    if data.factions.len() != factions.len() {
        return Err(format!(
            "towns: the executable has {} factions' tables, TownType.txt {} rows",
            data.factions.len(),
            factions.len()
        ));
    }
    if data.horde_buildings.len() != HORDE_RECORDS {
        return Err("towns: the hordes' buildings are not four".into());
    }
    let ab_creatures: BTreeSet<&str> = gap.ab.creatures.iter().map(String::as_str).collect();
    let rows = creature_rows(&inputs.sod_creatures)?;
    let horde_growth = rows
        .iter()
        .enumerate()
        .filter_map(|(id, r)| Some((u16::try_from(id).ok()?, r.horde_growth)))
        .collect();
    let creatures = rows
        .iter()
        .enumerate()
        .filter(|(_, r)| !r.name.starts_with("NOT USED"))
        .map(|(id, r)| {
            let pack = if ab_creatures.contains(r.name.as_str()) {
                "ab"
            } else {
                "base"
            };
            let id = u16::try_from(id).map_err(|_| "towns: creature id".to_owned())?;
            Ok((id, (key_of(&r.name), pack)))
        })
        .collect::<Result<_, String>>()?;
    let resources: Vec<String> = inputs
        .resources
        .records
        .iter()
        .take(RESOURCE_COUNT)
        .map(|r| key_of(&r[0]))
        .collect();
    let faction_index = |name: &str| {
        factions
            .iter()
            .position(|f| f == name)
            .ok_or_else(|| format!("gapfill: towns: unknown town `{name}`"))
    };
    let mut roles: BTreeMap<(usize, u8), BTreeMap<String, toml::Value>> = BTreeMap::new();
    for role in &spec.role {
        let targets = match &role.faction {
            Some(name) => vec![faction_index(name)?],
            None => (0..factions.len()).collect(),
        };
        for (name, value) in &role.roles {
            if !ROLES.contains(&name.as_str())
                || ["dwelling", "shipyard", "horde"].contains(&name.as_str())
            {
                return Err(format!(
                    "gapfill: towns: building {}: `{name}` is not a role it may give",
                    role.building
                ));
            }
            for &f in &targets {
                let merged = roles.entry((f, role.building)).or_default();
                if merged.insert(name.clone(), value.clone()).is_some() {
                    return Err(format!(
                        "gapfill: towns: {} building {}: `{name}` twice",
                        factions[f], role.building
                    ));
                }
            }
        }
    }
    for f in &spec.moat.faction {
        faction_index(&f.faction)?;
    }
    let ctx = Ctx {
        spec,
        tables: &inputs.town_names,
        data,
        factions: &factions,
        creatures,
        horde_growth,
        resources,
        roles,
    };
    let ab_factions: BTreeSet<&str> = gap.ab.factions.iter().map(String::as_str).collect();
    let mut texts: BTreeMap<&str, String> =
        PACKS.iter().map(|&(id, line)| (id, header(line))).collect();
    let base = texts.entry("base").or_default();
    base.push_str(&params(&ctx)?);
    for (pack, text) in visions(&ctx)? {
        texts.entry(pack).or_default().push_str(&text);
    }
    for (f, t) in data.factions.iter().enumerate() {
        let pack = if ab_factions.contains(factions[f].as_str()) {
            "ab"
        } else {
            "base"
        };
        let record = record(&ctx, f, t, pack)?;
        texts.entry(pack).or_default().push_str(&record);
        let sod = texts.entry("sod").or_default();
        sod.push_str(&moat_override(&ctx, f, pack)?);
    }
    let files: Vec<(&'static str, String)> = PACKS
        .iter()
        .map(|&(id, _)| (id, texts.remove(id).unwrap_or_default()))
        .collect();
    check(packs, &files)?;
    report.push("towns: the hordes' growth checked against the creatures' (E09 T25a)".into());
    Ok(files)
}

fn header(line: &str) -> String {
    format!(
        "# GENERATED by h3-harvest (E09 T5b2b2) from the user's installation (Complete 4.0) and\n\
         # crates/h3-harvest/data/gapfill.toml.\n\
         # {line}\n\
         # Do not edit by hand: change the harvester or the gap-fill file and re-run it\n\
         # (`cargo run -p h3-harvest -- --install original-data/install --out packs`).\n\
         # Gameplay data and names only (OPEN-QUESTIONS Q13); sources: docs/formats/installation.md\n\
         # 7.61, docs/rules/surveys/E09-towns.md 16, E09-construction.md 5 and 6; format:\n\
         # crates/h3-pack-towns.\n"
    )
}

/// The `[town_params]` (the base pack's).
fn params(ctx: &Ctx<'_>) -> Result<String, String> {
    let p = &ctx.spec.params;
    let gold = ctx
        .resources
        .iter()
        .find(|r| **r == key_of(&p.gold))
        .ok_or_else(|| format!("gapfill: towns: unknown resource `{}`", p.gold))?;
    let demoted = key_of(building_name(ctx.tables, 0, p.capitol_demoted_to)?);
    let garrison = &ctx.data.garrison;
    if garrison.len() != 4 {
        return Err("towns: the neutral garrison has not four levels".into());
    }
    let rows = garrison
        .iter()
        .map(|g| {
            let fits = u8::try_from(g.chance).is_ok()
                && u16::try_from(g.least).is_ok()
                && u16::try_from(g.most).is_ok();
            if fits {
                Ok(format!(
                    "  {{ chance = {}, count = [{}, {}] }}",
                    g.chance, g.least, g.most
                ))
            } else {
                Err("towns: a neutral garrison row does not fit the format".to_owned())
            }
        })
        .collect::<Result<Vec<_>, _>>()?;
    let [h1, h2, h3, h4] = p.hall_gold;
    let g = &ctx.spec.growth;
    let w = &g.garrison_week;
    let weights: Vec<String> = w.weights.iter().map(u8::to_string).collect();
    Ok(format!(
        "\n[town_params]\n\
         gold = {}\n\
         hall_gold = [{h1}, {h2}, {h3}, {h4}]\n\
         grail_gold = {}\n\
         reveal_radius = {}\n\
         level_2_dwelling_chance = {}\n\
         neutral_garrison = [\n{},\n]\n\
         capitol_demoted_to = {}\n\
         visions_factor = {}\n\
         visions_min_radius = {}\n\
         week_creature_chance = [{}, {}]\n\
         week_creature_bonus = {}\n\
         garrison_week = {{ odds = [{}, {}], out_of = {}, weights = [{}], weeks = {}, upgrade = {} }}\n",
        quote(gold),
        p.grail_gold,
        p.reveal_radius,
        ctx.data.dwelling_2_chance,
        rows.join(",\n"),
        quote(&demoted),
        ctx.spec.visions.factor,
        ctx.spec.visions.min_radius,
        g.week_creature_chance[0],
        g.week_creature_chance[1],
        g.week_creature_bonus,
        w.odds[0],
        w.odds[1],
        w.out_of,
        weights.join(", "),
        w.weeks,
        w.upgrade,
    ))
}

/// Each pack's `[town_visions]` (E09 T8c2): the gap-fill's creatures, each in its own pack's file.
fn visions(ctx: &Ctx<'_>) -> Result<BTreeMap<&'static str, String>, String> {
    let mut by_pack: BTreeMap<&'static str, Vec<String>> = BTreeMap::new();
    for id in &ctx.spec.visions.creatures {
        let (key, pack) = ctx
            .creatures
            .get(id)
            .ok_or_else(|| format!("gapfill: towns: unknown Visions creature {id}"))?;
        by_pack.entry(pack).or_default().push(quote(key));
    }
    Ok(by_pack
        .into_iter()
        .map(|(pack, keys)| {
            let text = format!("\n[town_visions]\ncreatures = [{}]\n", keys.join(", "));
            (pack, text)
        })
        .collect())
}

/// The siege fields of a faction under RoE and AB rules and under SoD rules.
fn sieges(ctx: &Ctx<'_>, f: usize) -> (TownSiege, TownSiege) {
    let moat = &ctx.spec.moat;
    moat.faction
        .iter()
        .find(|s| s.faction == ctx.factions[f])
        .map_or((moat.roe_ab, moat.sod), |s| (s.roe_ab, s.sod))
}

/// The SoD pack's override of a faction's record, when SoD plays its siege otherwise.
fn moat_override(ctx: &Ctx<'_>, f: usize, pack: &str) -> Result<String, String> {
    let (old, new) = sieges(ctx, f);
    let mut out = String::new();
    if old == new {
        return Ok(out);
    }
    let faction = reference(pack, &key_of(&ctx.factions[f]), "sod")?;
    let _ = write!(out, "\n[[town_override]]\nfaction = {}\n", quote(&faction));
    if old.moat != new.moat {
        let _ = writeln!(out, "moat = {}", quote(new.moat.word()));
    }
    if old.siege_obstacles != new.siege_obstacles {
        let _ = writeln!(out, "siege_obstacles = {}", new.siege_obstacles);
    }
    Ok(out)
}

/// A building being written.
#[derive(Default)]
struct Building {
    key: String,
    name: String,
    slot: Option<usize>,
    supersedes: Vec<u8>,
    forbidden_with: Option<u8>,
    auto_with: Vec<u8>,
    requires: Vec<u8>,
    cost: Vec<(String, u32)>,
}

/// The row of `Building.txt` that holds building `id`'s cost in faction `f`'s town, as
/// `building_costs_load` 0x5C14C0 fills its three tables and `building_cost` 0x5C1080 picks from
/// them (survey `E09-construction.md` §5): the neutral rows below 17, the faction's rows 17 to 25,
/// its dwellings' rows 30 to 43; none for 26 to 29 (past the faction's nine rows; no path asks
/// their cost).
fn cost_row(f: usize, id: u8) -> Option<usize> {
    let n = usize::from(id);
    match id {
        0..FIRST_SPECIAL => Some(NEUTRAL_COSTS + n),
        FIRST_SPECIAL..26 => {
            Some(SPECIAL_COSTS + SPECIAL_COSTS_STRIDE * f + n - usize::from(FIRST_SPECIAL))
        }
        DWELLING..44 => {
            Some(DWELLING_COSTS + DWELLING_COSTS_STRIDE * f + n - usize::from(DWELLING))
        }
        _ => None,
    }
}

/// Building `id`'s cost in faction `f`'s town: each of `resources` (`RESTYPES.TXT`'s order, the
/// file's columns) its row gives, the zero columns dropped; empty without a row.
fn cost(
    table: &TextTable,
    resources: &[String],
    f: usize,
    id: u8,
) -> Result<Vec<(String, u32)>, String> {
    let Some(row) = cost_row(f, id) else {
        return Ok(Vec::new());
    };
    let mut cost = Vec::new();
    for (r, resource) in resources.iter().enumerate() {
        let text = table
            .records
            .get(row)
            .and_then(|record| record.get(r))
            .map_or("", |c| c.trim());
        let n: u32 = text.parse().map_err(|_| {
            format!(
                "Building.txt row {}, column {}: `{text}` is not a number",
                row + 1,
                r + 1
            )
        })?;
        if n > 0 {
            cost.push((resource.clone(), n));
        }
    }
    Ok(cost)
}

/// A faction's `[[town]]` record.
#[allow(
    clippy::too_many_lines,
    reason = "one record, its parts in the format's order"
)]
fn record(ctx: &Ctx<'_>, f: usize, t: &FactionTowns, pack: &str) -> Result<String, String> {
    let faction = &ctx.factions[f];
    let err = |what: String| format!("towns: {faction}: {what}");
    // The buildings: the faction's list and the Village Hall.
    let mut buildings: BTreeMap<u8, Building> = BTreeMap::new();
    for id in std::iter::once(VILLAGE_HALL).chain(t.buildings.iter().map(|e| e.building)) {
        if buildings.contains_key(&id) || LEFT_OUT.contains(&id) {
            return Err(err(format!("its list holds building {id}")));
        }
        buildings.insert(id, Building::default());
    }
    let hordes = &ctx.data.horde_buildings;
    let upgraded_hordes = [hordes[1], hordes[3]];
    let mut keys = BTreeSet::new();
    for (&id, b) in &mut buildings {
        b.name = building_name(ctx.tables, f, id)?.to_owned();
        b.key = key_of(&b.name);
        if upgraded_hordes.contains(&id) {
            b.key = format!("upg_{}", b.key);
        }
        if !keys.insert(b.key.clone()) {
            return Err(err(format!("the key `{}` twice", b.key)));
        }
    }
    // The requirements (direct, in the list's order) and the costs (E09 T16b); a building with a
    // cost row has a cost.
    for e in &t.buildings {
        if let Some(&r) = e.listed.iter().find(|r| !buildings.contains_key(r)) {
            return Err(err(format!(
                "building {} requires building {r}, not its",
                e.building
            )));
        }
        if let Some(b) = buildings.get_mut(&e.building) {
            b.requires.clone_from(&e.listed);
        }
    }
    for (&id, b) in &mut buildings {
        b.cost = cost(&ctx.tables.costs, &ctx.resources, f, id)?;
        if b.cost.is_empty() && cost_row(f, id).is_some() {
            return Err(err(format!("building {id}'s cost row is blank")));
        }
    }
    // The map's slots; a slot naming a building the faction lacks (a Castle's fifth guild
    // level) gives none.
    for (slot, id) in t.map_building_ids.iter().enumerate() {
        let Some(b) = id.and_then(|id| buildings.get_mut(&id)) else {
            continue;
        };
        if b.slot.replace(slot).is_some() {
            return Err(err(format!("building {} in two slots", b.key)));
        }
    }
    // The supersede lists: the common one's entries of its buildings, then its own.
    for (entry, own) in ctx
        .data
        .common_supersedes
        .iter()
        .map(|e| (e, false))
        .chain(t.supersedes.iter().map(|e| (e, true)))
    {
        if !buildings.contains_key(&entry.building) {
            if own {
                return Err(err(format!(
                    "its supersede list names building {}, not its",
                    entry.building
                )));
            }
            continue;
        }
        for &s in &entry.listed {
            if !buildings.contains_key(&s) {
                return Err(err(format!(
                    "building {} supersedes building {s}, not its",
                    entry.building
                )));
            }
            let list = &mut buildings
                .get_mut(&entry.building)
                .ok_or_else(|| err(String::from("supersede list")))?
                .supersedes;
            if !list.contains(&s) {
                list.push(s);
            }
        }
    }
    // The hordes: a horde supersedes its level's dwelling; its upgrade is forbidden with it and
    // built with it and the level's upgraded dwelling (decision 29). Each grows at its level's
    // index, the upgrade's at the upgrade's (E09 T25a; survey `E09-growth.md` §1 step 5), its
    // record's growth its creature's horde growth.
    if t.creatures.len() != TOWN_CREATURES || t.hordes.len() != HORDE_RECORDS {
        return Err(err(String::from("the creatures or hordes do not fit")));
    }
    let mut horde_roles: BTreeMap<u8, (u8, bool)> = BTreeMap::new();
    for pair in [0, 2] {
        let (horde, upgrade) = (hordes[pair], hordes[pair + 1]);
        if !buildings.contains_key(&horde) {
            if buildings.contains_key(&upgrade) {
                return Err(err(format!("building {upgrade} without its horde {horde}")));
            }
            continue;
        }
        let creature = t.hordes[pair]
            .creature
            .ok_or_else(|| err(format!("horde {horde} without a creature")))?;
        let level = t.creatures[..usize::from(LEVELS)]
            .iter()
            .position(|&c| c == creature)
            .and_then(|l| u8::try_from(l).ok())
            .ok_or_else(|| err(format!("horde {horde}'s creature is no level's")))?;
        let dwelling = DWELLING + level;
        if !buildings[&horde].supersedes.contains(&dwelling) {
            return Err(err(format!(
                "horde {horde} does not supersede its level's dwelling {dwelling}"
            )));
        }
        if ctx.horde_growth.get(&creature) != Some(&t.hordes[pair].growth) {
            return Err(err(format!(
                "horde {horde}'s growth is not its creature's horde growth"
            )));
        }
        horde_roles.insert(horde, (level + 1, false));
        if let Some(b) = buildings.get_mut(&upgrade) {
            b.forbidden_with = Some(horde);
            b.auto_with = vec![horde, UPGRADED_DWELLING + level];
            horde_roles.insert(upgrade, (level + 1, true));
        }
    }
    // The creatures by level, the silo, the siege's fields.
    let mut levels = Vec::new();
    for l in 0..usize::from(LEVELS) {
        let mut level = Vec::new();
        for c in [t.creatures[l], t.creatures[l + usize::from(LEVELS)]] {
            let (key, own) = ctx
                .creatures
                .get(&c)
                .ok_or_else(|| err(format!("creature {c} is not harvested")))?;
            level.push(quote(&reference(own, key, pack)?));
        }
        levels.push(format!("[{}]", level.join(", ")));
    }
    let silo = t
        .silo
        .iter()
        .zip(&ctx.resources)
        .filter(|(n, _)| **n > 0)
        .map(|(n, r)| Ok(format!("{} = {n}", toml_key(&reference("base", r, pack)?))))
        .collect::<Result<Vec<_>, String>>()?;
    let (siege, _) = sieges(ctx, f);
    let mut out = String::new();
    let _ = write!(
        out,
        "\n[[town]]\nfaction = {}\nmoat = {}\n",
        quote(&key_of(faction)),
        quote(siege.moat.word())
    );
    if siege.siege_obstacles {
        out.push_str("siege_obstacles = true\n");
    }
    if !silo.is_empty() {
        let _ = writeln!(out, "silo = {{ {} }}", silo.join(", "));
    }
    let _ = writeln!(out, "creatures = [\n  {},\n]", levels.join(",\n  "));
    if pack == "ab" {
        out.push_str("roe = false\n");
    }
    // The buildings, by id.
    let key = |id: &u8| quote(&buildings[id].key);
    let mut given: BTreeSet<u8> = BTreeSet::new();
    for (&id, b) in &buildings {
        let _ = write!(
            out,
            "\n  [[town.building]]\n  key = {}\n  name = {}\n",
            quote(&b.key),
            quote(&b.name)
        );
        if let Some(slot) = b.slot {
            let _ = writeln!(out, "  slot = {slot}");
        }
        if !b.supersedes.is_empty() {
            let list: Vec<String> = b.supersedes.iter().map(key).collect();
            let _ = writeln!(out, "  supersedes = [{}]", list.join(", "));
        }
        if let Some(h) = &b.forbidden_with {
            let _ = writeln!(out, "  forbidden_with = {}", key(h));
        }
        if !b.auto_with.is_empty() {
            let list: Vec<String> = b.auto_with.iter().map(key).collect();
            let _ = writeln!(out, "  auto_with = [{}]", list.join(", "));
        }
        if !b.requires.is_empty() {
            let list: Vec<String> = b.requires.iter().map(key).collect();
            let _ = writeln!(out, "  requires = [{}]", list.join(", "));
        }
        if !b.cost.is_empty() {
            let cost = b
                .cost
                .iter()
                .map(|(r, n)| Ok(format!("{} = {n}", toml_key(&reference("base", r, pack)?))))
                .collect::<Result<Vec<_>, String>>()?;
            let _ = writeln!(out, "  cost = {{ {} }}", cost.join(", "));
        }
        let mut roles = ctx.roles.get(&(f, id)).cloned().unwrap_or_default();
        if !roles.is_empty() {
            given.insert(id);
        }
        if id == SHIPYARD {
            roles.insert("shipyard".into(), toml::Value::Boolean(true));
        }
        if let Some(&(level, upgraded)) = horde_roles.get(&id) {
            roles.insert(
                "horde".into(),
                toml::Value::Array(vec![
                    toml::Value::Integer(i64::from(level)),
                    toml::Value::Boolean(upgraded),
                ]),
            );
        }
        if (DWELLING..DWELLING + 2 * LEVELS).contains(&id) {
            let upgraded = id >= UPGRADED_DWELLING;
            let level =
                id - if upgraded {
                    UPGRADED_DWELLING
                } else {
                    DWELLING
                } + 1;
            roles.insert(
                "dwelling".into(),
                toml::Value::Array(vec![
                    toml::Value::Integer(i64::from(level)),
                    toml::Value::Boolean(upgraded),
                ]),
            );
        }
        for role in ROLES {
            if let Some(value) = roles.get(role) {
                let _ = writeln!(out, "  {role} = {}", inline(value));
            }
        }
    }
    if let Some(&(_, id)) = ctx
        .roles
        .keys()
        .find(|&&(rf, id)| rf == f && !given.contains(&id))
    {
        return Err(err(format!(
            "the gap-fill gives roles to building {id}, not its"
        )));
    }
    Ok(out)
}

/// A key as TOML writes it: bare when it can be, else quoted (`"base:wood"`).
fn toml_key(key: &str) -> String {
    let bare = !key.is_empty()
        && key
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
    if bare { key.to_owned() } else { quote(key) }
}

/// A role's value as one line of TOML (an inline table for `defender`).
fn inline(value: &toml::Value) -> String {
    match value {
        toml::Value::Table(t) => {
            let fields: Vec<String> = t
                .iter()
                .map(|(k, v)| format!("{k} = {}", inline(v)))
                .collect();
            format!("{{ {} }}", fields.join(", "))
        }
        toml::Value::Array(a) => {
            let items: Vec<String> = a.iter().map(inline).collect();
            format!("[{}]", items.join(", "))
        }
        toml::Value::String(s) => quote(s),
        other => other.to_string(),
    }
}

/// Compiles the town files on the core files in every allowed combination and both profiles.
fn check(
    packs: &[(&'static str, (String, String))],
    towns: &[(&'static str, String)],
) -> Result<(), String> {
    let text = |id: &str| -> Result<(&str, &str), String> {
        let core = packs
            .iter()
            .find(|(p, _)| *p == id)
            .map(|(_, (core, _))| core.as_str())
            .ok_or_else(|| format!("towns: no core pack `{id}`"))?;
        let town = towns
            .iter()
            .find(|(p, _)| *p == id)
            .map_or("", |(_, t)| t.as_str());
        Ok((core, town))
    };
    for set in [&["base"][..], &["base", "ab"], &["base", "ab", "sod"]] {
        let texts = set
            .iter()
            .map(|id| text(id))
            .collect::<Result<Vec<_>, _>>()?;
        for profile in ["base:faithful", "base:strict-fixed"] {
            h3_pack_towns::compile(&texts, profile).map_err(|e| {
                format!(
                    "generated town files do not compile ({}, {profile}): {e}",
                    set.join("+")
                )
            })?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
