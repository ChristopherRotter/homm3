//! Special battlefields and obstacle types (E04c T2b): the battlefields' rules from the gap-fill
//! file, the obstacle types from the executable's tables (`docs/formats/installation.md` §7.26,
//! equal to VCMI's `obstacles.json`). A type goes to `sod` when it appears on a battlefield of
//! `sod` (it then has no terrain), else to `base`.

use std::fmt::Write as _;

use h3_formats::exe::ObstacleData;

use crate::gapfill::{BattlefieldSpec, FactionChangeSpec};
use crate::key_of;

/// Columns of the original's battlefield (the hex numbers of the tables).
const FIELD_COLUMNS: i16 = 17;
/// The packs a battlefield may belong to.
const PACKS: [&str; 2] = ["base", "sod"];
/// Where absolute obstacles' keys start (VCMI's numbering, which the packs follow).
pub const ABSOLUTE_BASE: usize = 100;

/// The battlefield and obstacle tables of `base` and `sod`, as pack text.
#[derive(Debug, Default)]
pub struct Texts {
    /// `base`'s.
    pub base: String,
    /// `sod`'s.
    pub sod: String,
}

impl Texts {
    fn of(&mut self, pack: &str) -> &mut String {
        if pack == "sod" {
            &mut self.sod
        } else {
            &mut self.base
        }
    }
}

/// Writes the battlefields of `specs` and the obstacle types of `data`. `terrains` are the
/// `TERRNAME.txt` names (bit `t` of a type's terrain bits), `towns` the `TownType.txt` names with
/// the pack that defines each.
///
/// # Errors
/// A battlefield in an unknown pack, twice, with an unknown town, two battlefields with one
/// field, an obstacle bit without a terrain or battlefield, a type both on a `sod` battlefield and
/// elsewhere.
pub fn write(
    data: &ObstacleData,
    specs: &[BattlefieldSpec],
    terrains: &[String],
    towns: &[(String, &'static str)],
) -> Result<Texts, String> {
    let mut out = Texts::default();
    for (i, b) in specs.iter().enumerate() {
        if !PACKS.contains(&b.pack.as_str()) {
            return Err(format!(
                "gapfill: battlefield `{}` in pack `{}`",
                b.id, b.pack
            ));
        }
        if specs[..i].iter().any(|o| o.id == b.id) {
            return Err(format!("gapfill: battlefield `{}` twice", b.id));
        }
        if b.field.is_some() && specs[..i].iter().any(|o| o.field == b.field) {
            return Err(format!(
                "gapfill: two battlefields with field {:?}",
                b.field
            ));
        }
        write_battlefield(out.of(&b.pack), b, data, towns)?;
    }
    // The pack and key of the battlefield with field bit `f`, of the terrain with bit `t`.
    let field = |f: usize| {
        specs
            .iter()
            .find(|b| b.field.map(usize::from) == Some(f))
            .map(|b| (b.pack.as_str(), b.id.clone()))
            .ok_or_else(|| format!("obstacle tables: field bit {f} without a battlefield"))
    };
    let terrain = |t: usize| {
        terrains
            .get(t)
            .map(|n| key_of(n))
            .ok_or_else(|| format!("obstacle tables: terrain bit {t} without a terrain"))
    };
    let bits = |mask: u16| (0..16).filter(move |b| mask & (1 << b) != 0);
    let mut write_type = |id: usize, terrain_bits: u16, field_bits: u16, body: &str| {
        let ts = bits(terrain_bits)
            .map(terrain)
            .collect::<Result<Vec<_>, _>>()?;
        let fs = bits(field_bits).map(field).collect::<Result<Vec<_>, _>>()?;
        let sod = fs.iter().any(|(p, _)| *p == "sod");
        if sod && (!ts.is_empty() || fs.iter().any(|(p, _)| *p != "sod")) {
            return Err(format!(
                "obstacle type {id}: on a sod battlefield and on base terrains or battlefields"
            ));
        }
        let text = out.of(if sod { "sod" } else { "base" });
        let _ = write!(text, "\n[[obstacle]]\nid = \"obstacle_{id}\"\n");
        if !ts.is_empty() {
            let _ = writeln!(text, "terrains = [{}]", list(&ts));
        }
        if !fs.is_empty() {
            let keys: Vec<String> = fs.into_iter().map(|(_, k)| k).collect();
            let _ = writeln!(text, "battlefields = [{}]", list(&keys));
        }
        text.push_str(body);
        Ok::<(), String>(())
    };
    for (i, t) in data.types.iter().enumerate() {
        // The original's offsets count on a 17-column field from the anchor: dy rows up or down,
        // rounded so that an offset reaches at most 8 columns to either side.
        let blocked: Vec<String> = t
            .blocked
            .iter()
            .map(|&o| {
                let o = i16::from(o);
                let dy = (o + 8).div_euclid(FIELD_COLUMNS);
                format!("[{}, {dy}]", o - FIELD_COLUMNS * dy)
            })
            .collect();
        let body = format!(
            "width = {}\nheight = {}\nblocked = [{}]\n",
            t.width,
            t.height,
            blocked.join(", ")
        );
        write_type(i, t.terrains, t.fields, &body)?;
    }
    for (j, a) in data.absolute.iter().enumerate() {
        let blocked: Vec<String> = a
            .blocked
            .iter()
            .map(|&h| {
                let h = i32::from(h);
                let c = i32::from(FIELD_COLUMNS);
                format!("[{}, {}]", h % c, h / c)
            })
            .collect();
        let body = format!("absolute = true\nblocked = [{}]\n", blocked.join(", "));
        write_type(ABSOLUTE_BASE + j, a.terrains, a.fields, &body)?;
    }
    Ok(out)
}

/// A list of quoted keys.
fn list(keys: &[String]) -> String {
    keys.iter()
        .map(|k| format!("\"{k}\""))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Writes one `[[battlefield]]` table.
fn write_battlefield(
    out: &mut String,
    b: &BattlefieldSpec,
    data: &ObstacleData,
    towns: &[(String, &'static str)],
) -> Result<(), String> {
    let _ = write!(out, "\n[[battlefield]]\nid = \"{}\"\n", b.id);
    if b.field.is_some() {
        out.push_str("own_obstacles = true\n");
    }
    if b.boats {
        let hexes: Vec<String> = data.boats.iter().map(u16::to_string).collect();
        let _ = writeln!(out, "blocked = [{}]", hexes.join(", "));
    }
    for (name, changes) in [("morale", &b.morale), ("luck", &b.luck)] {
        if changes.is_empty() {
            continue;
        }
        let entries = changes
            .iter()
            .map(|c| change(c, &b.pack, towns))
            .collect::<Result<Vec<_>, _>>()?;
        let _ = writeln!(out, "{name} = [{}]", entries.join(", "));
    }
    for (flag, set) in [
        ("no_morale", b.no_morale),
        ("no_luck", b.no_luck),
        ("no_native", b.no_native),
        ("no_creature_casts", b.no_creature_casts),
        ("expert_other_casts", b.expert_other_casts),
    ] {
        if set {
            let _ = writeln!(out, "{flag} = true");
        }
    }
    if let Some(level) = b.max_spell_level {
        let _ = writeln!(out, "max_spell_level = {level}");
    }
    if !b.expert_schools.is_empty() {
        let schools: Vec<String> = b
            .expert_schools
            .iter()
            .map(|s| {
                if b.pack == "base" {
                    s.clone()
                } else {
                    format!("base:{s}")
                }
            })
            .collect();
        let _ = writeln!(out, "expert_schools = [{}]", list(&schools));
    }
    Ok(())
}

/// One morale or luck change in pack syntax, its towns as faction keys seen from `pack`.
fn change(
    c: &FactionChangeSpec,
    pack: &str,
    towns: &[(String, &'static str)],
) -> Result<String, String> {
    let keys = c
        .towns
        .iter()
        .map(|name| {
            let key = key_of(name);
            towns
                .iter()
                .find(|(k, _)| *k == key)
                .map(|(k, owner)| {
                    if *owner == pack {
                        k.clone()
                    } else {
                        format!("{owner}:{k}")
                    }
                })
                .ok_or_else(|| format!("gapfill: unknown town `{name}`"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(format!(
        "{{ factions = [{}], amount = {} }}",
        list(&keys),
        c.amount
    ))
}
