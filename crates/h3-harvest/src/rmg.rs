//! The map generator's tables (E14 T2b; ADR-0029 point 5): `Rmg.txt`, `Objects.txt` and
//! `Rand_trn.txt` of AB's archive (`H3ab_bmp.lod`) into `packs/ab/rmg.toml`, SoD's
//! (`H3bitmap.lod`) additions into `packs/sod/rmg.toml` (format: `packs/README.md`, "The map
//! generator's tables"; parsed by `h3-pack-rmg`).
//!
//! The values are kept as the game reads them (survey `docs/rules/surveys/E14-generator.md`
//! §3.1, rules RMG-TPL-001, -002 and -006): `rmg.txt`'s rows from row 3, a template at
//! a row of two or more cells whose column 0 is set, a zone at 76 or more with column 3 set, a
//! connection at 85 or more with column 76 set and 77 not empty (a row may be all three); *set*
//! is not empty and not beginning with a space; numbers as `c_atoi` (0x617F09) reads them;
//! `objects.txt`'s count line, then that many lines of nine fields; `rand_trn.txt`'s rows from 3
//! to the first whose column 0 is not set, one Adjacent and one Overlap score per row. SoD's
//! pack holds what SoD's tables add to AB's: the templates after AB's (AB's must be a prefix), the
//! `objects.txt` lines inserted or changed (a changed line keeps its picture, class and subclass),
//! the obstacles if they differ. A data quirk's fix is an override gated by its flag, from the
//! gap-fill (8XM8's ids, RMG-QUIRK-22).

use std::fmt::Write as _;

use h3_formats::text::TextTable;

use crate::gapfill::{GapFill, RmgZoneIdsFix};
use crate::quote;

/// One archive's three tables.
#[derive(Debug, Clone, Default)]
pub struct Files {
    /// `Rmg.txt`.
    pub rmg: TextTable,
    /// `Objects.txt` (one field per record: the line).
    pub objects: TextTable,
    /// `Rand_trn.txt`.
    pub rand_trn: TextTable,
}

/// The executable's constants the generator's constructor reads (`rmg_generator_create`
/// 0x537B10; E14 T2c): they go into SoD's pack as `[generator]`.
#[derive(Debug, Clone, Default)]
pub struct Constants {
    /// Per hero id, its record's (the hero table 0x679DD0) bytes +0x38 (RoE's), +0x39 (the
    /// expansions') and +0x3A (a campaign hero) set.
    pub heroes: Vec<[bool; 3]>,
    /// The per-map limit pairs (0x640718): object type and limit, in the table's order.
    pub map_limits: Vec<(u8, i32)>,
    /// The per-zone limit pairs (0x640808).
    pub zone_limits: Vec<(u8, i32)>,
    /// Step 1's cosines and sines (0x682500, 0x682600) in ten-thousandths (E14 T3c).
    pub layout_cos: Vec<i32>,
    /// The sines.
    pub layout_sin: Vec<i32>,
    /// Per terrain its four town types (0x682450).
    pub terrain_towns: Vec<[i32; 4]>,
    /// Per town type its native terrain (0x6408C8).
    pub native_terrains: Vec<i32>,
}

/// The tables of both archives and the executable's constants.
#[derive(Debug, Clone, Default)]
pub struct Tables {
    /// `H3ab_bmp.lod`'s.
    pub ab: Files,
    /// `H3bitmap.lod`'s (the ones the game reads).
    pub sod: Files,
    /// `Heroes3.exe`'s.
    pub constants: Constants,
}

/// SoD's pack's `[generator]`: the constants as mark strings by hero id and pairs in order.
fn generator_section(c: &Constants) -> String {
    let marks = |k: usize| -> String {
        c.heroes
            .iter()
            .map(|h| if h[k] { '1' } else { '0' })
            .collect()
    };
    let pairs = |p: &[(u8, i32)]| -> String {
        p.iter()
            .map(|(t, l)| format!("[{t}, {l}]"))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let list =
        |v: &[i32]| -> String { v.iter().map(i32::to_string).collect::<Vec<_>>().join(", ") };
    let rows = c
        .terrain_towns
        .iter()
        .map(|row| format!("[{}]", list(row)))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "\n# The generator's constants, read from Heroes3.exe (rmg_generator_create 0x537B10; \
         ADR-0029 point 5):\n# the hero table 0x679DD0's bytes +0x38 (RoE's heroes), +0x39 (the \
         expansions') and +0x3A (campaign\n# heroes) by hero id; the limit pairs by object type \
         per map (0x640718) and per zone\n# (0x640808), object type and limit in the tables' \
         order;\n# step 1's layout points (rmg_layout_cos 0x682500, rmg_layout_sin 0x682600) in \
         ten-thousandths, the\n# doubles nearest them; per terrain, dirt to lava and water, its \
         four town types (rmg_terrain_towns\n# 0x682450); per town type its native terrain \
         (rmg_town_native_terrain 0x6408C8; E14 T3c).\n[generator]\nheroes_roe = \"{}\"\n\
         heroes_expansions = \"{}\"\nheroes_campaign = \"{}\"\nmap_limits = [{}]\n\
         zone_limits = [{}]\nlayout_cos = [{}]\nlayout_sin = [{}]\nterrain_towns = [{}]\n\
         native_terrains = [{}]\n",
        marks(0),
        marks(1),
        marks(2),
        pairs(&c.map_limits),
        pairs(&c.zone_limits),
        list(&c.layout_cos),
        list(&c.layout_sin),
        rows,
        list(&c.native_terrains),
    )
}

/// The header rows of `rmg.txt` and `rand_trn.txt`, never read.
const HEADER_ROWS: usize = 3;
/// A zone row's least cell count and a connection row's (RMG-TPL-001).
const ZONE_CELLS: usize = 76;
const CONNECTION_CELLS: usize = 85;
/// A template line's fields in `objects.txt`.
const OBJECT_FIELDS: usize = 9;

/// Whether a cell is set: present, not empty, not beginning with a space.
fn set(row: &[String], col: usize) -> bool {
    row.get(col)
        .is_some_and(|c| !c.is_empty() && !c.starts_with(' '))
}

/// A cell as the C runtime's `atoi` reads it: leading white space, a sign, the digits up to the
/// first other character; an empty or missing cell 0. A number outside `i32` is refused (the
/// tables hold none; what `c_atoi` makes of one is not read).
fn atoi(row: &[String], col: usize) -> Result<i32, String> {
    let cell = row.get(col).map_or("", String::as_str);
    let text = cell.trim_start_matches([' ', '\t', '\n', '\u{b}', '\u{c}', '\r']);
    let (negative, digits) = match text.as_bytes().first() {
        Some(b'-') => (true, &text[1..]),
        Some(b'+') => (false, &text[1..]),
        _ => (false, text),
    };
    let mut value = 0i64;
    for d in digits.bytes().take_while(u8::is_ascii_digit) {
        value = value * 10 + i64::from(d - b'0');
        if value > i64::from(i32::MAX) + 1 {
            return Err(format!("cell `{cell}` is outside i32"));
        }
    }
    let value = if negative { -value } else { value };
    i32::try_from(value).map_err(|_| format!("cell `{cell}` is outside i32"))
}

fn atois<const N: usize>(row: &[String], first: usize) -> Result<[i32; N], String> {
    let mut out = [0; N];
    for (k, v) in out.iter_mut().enumerate() {
        *v = atoi(row, first + k)?;
    }
    Ok(out)
}

fn list(values: &[i32]) -> String {
    let items: Vec<String> = values.iter().map(i32::to_string).collect();
    format!("[{}]", items.join(", "))
}

/// The set-ness of `n` cells from `first` as a mark string.
fn marks(row: &[String], first: usize, n: usize) -> String {
    (first..first + n)
        .map(|c| if set(row, c) { '1' } else { '0' })
        .collect()
}

/// A zone row as an inline table (RMG-TPL-002).
// rule: RMG-TPL-002
fn zone(row: &[String]) -> Result<String, String> {
    let kind = (4..=7)
        .rev()
        .find(|&c| set(row, c))
        .map_or("treasure", |c| {
            ["human_start", "computer_start", "treasure", "junction"][c - 4]
        });
    let strength = match row
        .get(55)
        .and_then(|c| c.chars().next())
        .map(|c| c.to_ascii_lowercase())
    {
        Some('n') => "none",
        Some('w') => "weak",
        Some('s') => "strong",
        _ => "average",
    };
    let bands: [[i32; 3]; 3] = [atois(row, 67)?, atois(row, 70)?, atois(row, 73)?];
    let bands: Vec<String> = bands.iter().map(|b| list(b)).collect();
    Ok(format!(
        "{{ id = {}, kind = \"{kind}\", base_size = {}, humans = {}, players = {}, ownership = {}, \
         player_towns = {}, neutral_towns = {}, same_type = {}, town_types = \"{}\", mines = {}, \
         mine_density = {}, terrain_match = {}, terrains = \"{}\", strength = \"{strength}\", \
         monster_match = {}, neutral_monsters = {}, monster_towns = \"{}\", treasure = [{}] }}",
        atoi(row, 3)?,
        atoi(row, 8)?,
        list(&atois::<2>(row, 9)?),
        list(&atois::<2>(row, 11)?),
        atoi(row, 13)?,
        list(&atois::<4>(row, 14)?),
        list(&atois::<4>(row, 18)?),
        set(row, 22),
        marks(row, 23, 9),
        list(&atois::<7>(row, 32)?),
        list(&atois::<7>(row, 39)?),
        set(row, 46),
        marks(row, 47, 8),
        set(row, 56),
        set(row, 57),
        marks(row, 58, 9),
        bands.join(", "),
    ))
}

/// A connection row as an inline table, with its first zone's id.
fn connection(row: &[String], first: i32) -> Result<String, String> {
    Ok(format!(
        "{{ zones = [{first}, {}], value = {}, wide = {}, border_guard = {}, humans = {}, players \
         = {} }}",
        atoi(row, 77)?,
        atoi(row, 78)?,
        set(row, 79),
        set(row, 80),
        list(&atois::<2>(row, 81)?),
        list(&atois::<2>(row, 83)?),
    ))
}

/// A template of `rmg.txt`: its `[[template]]` block and its connections whose first zone is a
/// decimal (`c_atoi` reads its integer part): each one's index and second zone.
struct Template {
    name: String,
    block: String,
    decimal_first_zones: Vec<(usize, i32)>,
}

/// `rmg.txt`'s templates in row order (RMG-TPL-001, -002).
// rule: RMG-TPL-001
fn templates(table: &TextTable) -> Result<Vec<Template>, String> {
    struct Building {
        name: String,
        head: String,
        zones: Vec<String>,
        connections: Vec<String>,
        decimals: Vec<(usize, i32)>,
    }
    let finish = |b: Building| {
        let mut block = b.head;
        for (key, rows) in [("zones", &b.zones), ("connections", &b.connections)] {
            let _ = writeln!(block, "{key} = [");
            for r in rows {
                let _ = writeln!(block, "  {r},");
            }
            block.push_str("]\n");
        }
        Template {
            name: b.name,
            block,
            decimal_first_zones: b.decimals,
        }
    };
    let mut out = Vec::new();
    let mut current: Option<Building> = None;
    for (n, row) in table.records.iter().enumerate().skip(HEADER_ROWS) {
        let line = n + 1;
        let err = |e: String| format!("Rmg.txt row {line}: {e}");
        if row.len() >= 2 && set(row, 0) {
            if let Some(done) = current.take() {
                out.push(finish(done));
            }
            current = Some(Building {
                name: row[0].clone(),
                head: format!(
                    "[[template]]\nname = {}\nsizes = {}\n",
                    quote(&row[0]),
                    list(&[atoi(row, 1).map_err(err)?, atoi(row, 2).map_err(err)?])
                ),
                zones: Vec::new(),
                connections: Vec::new(),
                decimals: Vec::new(),
            });
        }
        let is_zone = row.len() >= ZONE_CELLS && set(row, 3);
        let is_connection = row.len() >= CONNECTION_CELLS
            && set(row, 76)
            && row.get(77).is_some_and(|c| !c.is_empty());
        if !is_zone && !is_connection {
            continue;
        }
        let Some(b) = current.as_mut() else {
            return Err(err(String::from(
                "a zone or connection before any template",
            )));
        };
        if is_zone {
            b.zones.push(zone(row).map_err(err)?);
        }
        if is_connection {
            if row[77].contains('.') {
                return Err(err(String::from("a decimal second zone id")));
            }
            if row[76].contains('.') {
                b.decimals
                    .push((b.connections.len(), atoi(row, 77).map_err(err)?));
            }
            b.connections
                .push(connection(row, atoi(row, 76).map_err(err)?).map_err(err)?);
        }
    }
    out.extend(current.map(finish));
    Ok(out)
}

/// An `objects.txt` line: its identity (picture, class, subclass) and its fields as TOML.
#[derive(Clone, PartialEq, Eq)]
struct ObjectLine {
    identity: (String, String, String),
    fields: String,
}

/// `objects.txt`'s template lines in file order: the count line, then that many lines.
fn objects(table: &TextTable) -> Result<Vec<ObjectLine>, String> {
    let line = |n: usize| {
        table
            .records
            .get(n)
            .and_then(|r| r.first())
            .map(String::as_str)
    };
    let count: usize = line(0)
        .and_then(|c| c.trim().parse().ok())
        .ok_or("Objects.txt: no count line")?;
    (1..=count)
        .map(|n| {
            let text = line(n).ok_or_else(|| format!("Objects.txt: {count} lines expected"))?;
            let f: Vec<&str> = text.split(' ').collect();
            if f.len() != OBJECT_FIELDS {
                return Err(format!("Objects.txt line {}: `{text}`", n + 1));
            }
            let numbers: Result<Vec<u32>, _> = f[5..].iter().map(|x| x.parse::<u32>()).collect();
            let numbers = numbers.map_err(|e| format!("Objects.txt line {}: {e}", n + 1))?;
            Ok(ObjectLine {
                identity: (f[0].to_owned(), f[5].to_owned(), f[6].to_owned()),
                fields: format!(
                    "picture = {}, passability = \"{}\", actions = \"{}\", terrains = \"{}\", \
                     editor_terrains = \"{}\", class = {}, subclass = {}, group = {}, overlay = {}",
                    quote(f[0]),
                    f[1],
                    f[2],
                    f[3],
                    f[4],
                    numbers[0],
                    numbers[1],
                    numbers[2],
                    numbers[3]
                ),
            })
        })
        .collect()
}

/// `rand_trn.txt`'s obstacle rows (RMG-TPL-006) as `[[obstacle]]` blocks.
// rule: RMG-TPL-006
fn obstacles(table: &TextTable) -> Result<Vec<String>, String> {
    let rows: Vec<&Vec<String>> = table
        .records
        .iter()
        .skip(HEADER_ROWS)
        .take_while(|r| set(r, 0))
        .collect();
    let n = rows.len();
    rows.iter()
        .enumerate()
        .map(|(k, row)| {
            let err = |e: String| format!("Rand_trn.txt row {}: {e}", k + HEADER_ROWS + 1);
            let scores = |first: usize| -> Result<Vec<i32>, String> {
                (first..first + n)
                    .map(|c| atoi(row, c).map_err(err))
                    .collect()
            };
            Ok(format!(
                "[[obstacle]]\ntype = {}\nsubtype = {}\nterrain = {}\nmodifiers = {}\nadjacent = \
                 {}\noverlap = {}\n",
                atoi(row, 3).map_err(err)?,
                atoi(row, 4).map_err(err)?,
                atoi(row, 6).map_err(err)?,
                list(&atois::<9>(row, 7).map_err(err)?),
                list(&scores(16)?),
                list(&scores(16 + n)?),
            ))
        })
        .collect()
}

/// The gated overrides of a data quirk's zone ids (RMG-QUIRK-22): the template named once in
/// `templates`, its connections whose first zone is a decimal in row order, one fixed first zone
/// each.
fn zone_id_overrides(templates: &[Template], fix: &RmgZoneIdsFix) -> Result<String, String> {
    let mut found = templates
        .iter()
        .enumerate()
        .filter(|(_, t)| t.name == fix.template);
    let (Some((index, template)), None) = (found.next(), found.next()) else {
        return Err(format!(
            "gap-fill {}: template `{}` is not in the table once",
            fix.quirk, fix.template
        ));
    };
    if template.decimal_first_zones.len() != fix.first_zones.len() {
        return Err(format!(
            "gap-fill {}: template `{}` has {} decimal first zones, the fix {}",
            fix.quirk,
            fix.template,
            template.decimal_first_zones.len(),
            fix.first_zones.len()
        ));
    }
    let mut out = String::new();
    for (&(connection, second), &first) in template.decimal_first_zones.iter().zip(&fix.first_zones)
    {
        let _ = write!(
            out,
            "[[connection_override]]          # {}: {}'s first zone as the series continues\n\
             template = {index}\nconnection = {connection}\nfixes = {}\nzones = [{first}, \
             {second}]\n\n",
            fix.quirk,
            fix.template,
            quote(&fix.flag),
        );
    }
    Ok(out)
}

/// `objects.txt` lines, each at a line of AB's table (inserted before it, or replacing it).
type Edits = Vec<(usize, ObjectLine)>;

/// What SoD's `objects.txt` adds to AB's: each SoD line equal to the next AB line is kept; one
/// with that line's picture, class and subclass replaces it; any other is inserted before it.
/// Every AB line must be kept or replaced.
fn object_edits(ab: &[ObjectLine], sod: &[ObjectLine]) -> Result<(Edits, Edits), String> {
    let mut inserts = Vec::new();
    let mut changes = Vec::new();
    let mut i = 0;
    for line in sod {
        match ab.get(i) {
            Some(old) if old == line => i += 1,
            Some(old) if old.identity == line.identity => {
                changes.push((i, line.clone()));
                i += 1;
            }
            _ => inserts.push((i, line.clone())),
        }
    }
    if i != ab.len() {
        return Err(format!("SoD's Objects.txt drops AB's line {}", i + 2));
    }
    Ok((inserts, changes))
}

/// The header of a harvested `rmg.toml`: its archive and what it holds (one comment line each).
fn header(archive: &str, holds: &str) -> String {
    format!(
        "# Harvested by h3-harvest from {archive}'s Rmg.txt, Objects.txt and Rand_trn.txt:\n# {holds}.\n\
         # Do not edit by hand (packs/README.md). Format: packs/README.md, \"The map generator's\n\
         # tables\"; parsed by h3-pack-rmg (ADR-0029 point 5).\n\n"
    )
}

/// Harvests `ab/rmg.toml` and `sod/rmg.toml`; `report` gets one line of counts.
///
/// # Errors
///
/// A table that does not read as survey §3.1 describes; SoD's tables not extending AB's (its
/// templates after AB's, every AB line of `objects.txt` kept or changed); a gap-fill fix that does
/// not match its template.
pub fn harvest_rmg(
    tables: &Tables,
    gap: &GapFill,
    report: &mut Vec<String>,
) -> Result<Vec<(&'static str, String)>, String> {
    let ab_templates = templates(&tables.ab.rmg)?;
    let sod_templates = templates(&tables.sod.rmg)?;
    let ab_objects = objects(&tables.ab.objects)?;
    let sod_objects = objects(&tables.sod.objects)?;
    let ab_obstacles = obstacles(&tables.ab.rand_trn)?;
    let sod_obstacles = obstacles(&tables.sod.rand_trn)?;
    let extends = sod_templates.len() >= ab_templates.len()
        && ab_templates
            .iter()
            .zip(&sod_templates)
            .all(|(a, s)| a.block == s.block);
    if !extends {
        return Err(String::from(
            "SoD's Rmg.txt does not begin with AB's templates",
        ));
    }
    let (inserts, changes) = object_edits(&ab_objects, &sod_objects)?;

    let mut ab = header("H3ab_bmp.lod", "AB's three tables as the game reads them");
    ab.push_str("object_templates = [\n");
    for o in &ab_objects {
        let _ = writeln!(ab, "  {{ {} }},", o.fields);
    }
    ab.push_str("]\n");
    for t in &ab_templates {
        ab.push('\n');
        ab.push_str(&t.block);
    }
    ab.push('\n');
    for fix in &gap.data_quirks.rmg_zone_ids {
        ab.push_str(&zone_id_overrides(&ab_templates, fix)?);
    }
    for o in &ab_obstacles {
        ab.push_str(o);
        ab.push('\n');
    }

    let mut sod = header(
        "H3bitmap.lod",
        "what SoD's tables add to AB's (packs/ab/rmg.toml)",
    );
    sod.push_str(
        "# `before` and `line` count the lines of AB's objects.txt from 0 (the count line not \
         counted).\nobject_templates = [\n",
    );
    for (before, o) in &inserts {
        let _ = writeln!(sod, "  {{ before = {before}, {} }},", o.fields);
    }
    sod.push_str("]\nobject_template_overrides = [\n");
    for (line, o) in &changes {
        let _ = writeln!(sod, "  {{ line = {line}, {} }},", o.fields);
    }
    sod.push_str("]\n");
    for t in &sod_templates[ab_templates.len()..] {
        sod.push('\n');
        sod.push_str(&t.block);
    }
    if sod_obstacles != ab_obstacles {
        for o in &sod_obstacles {
            sod.push('\n');
            sod.push_str(o);
        }
    }
    sod.push_str(&generator_section(&tables.constants));
    report.push(format!(
        "rmg: ab {} templates, {} object templates, {} obstacles; sod {} templates more, {} object \
         templates added, {} changed{}",
        ab_templates.len(),
        ab_objects.len(),
        ab_obstacles.len(),
        sod_templates.len() - ab_templates.len(),
        inserts.len(),
        changes.len(),
        if sod_obstacles == ab_obstacles {
            ", the same obstacles"
        } else {
            ", its own obstacles"
        }
    ));
    Ok(vec![("ab", ab), ("sod", sod)])
}

#[cfg(test)]
mod tests;
