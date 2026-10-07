//! Hero content (E04a T3, pack format v3): spell schools, spells, secondary skills, hero classes,
//! hero types and the creature upgrade relation, harvested from the text tables and the
//! executable's hero tables (`docs/formats/installation.md` §6–§7), with the gap-fill file's
//! `[heroes]` section for what neither holds.
//!
//! Pack membership: everything of The Restoration of Erathia goes into `base`; the Conflux
//! classes and the 28 heroes the executable marks as not RoE go into `ab`; spells that differ
//! between the AB-era and the SoD spell table are SoD's (`sod`). RoE-era copies of the tables are
//! not in the installation; as for creatures, the AB-era rows stand for RoE (Q15).

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use h3_formats::exe::{CreatureRecord, HeroData, SkillValue};
use h3_formats::text::TextTable;

use crate::gapfill::{AbilitySpec, GapFill};
use crate::{key_of, quote};

/// Fields of `HCTRAITS.TXT` before the skill weights (name, aggression, primary skills, gains).
const HCTRAITS_FIXED: usize = 14;
/// Fields per row of `HOTRAITS.TXT`.
const HOTRAITS_FIELDS: usize = 10;
/// Fields per row of `SPTRAITS.TXT`.
const SPTRAITS_FIELDS: usize = 34;
/// Columns of the four school flags in `SPTRAITS.TXT` (header names in row 1).
const SCHOOL_COLUMNS: std::ops::Range<usize> = 3..7;

/// Skills whose battle effect the executable's value table gives (installation.md §7.4): skill
/// id, effect kind and parameter name, and whether the values are fractions (to basis points).
const TABLE_SKILLS: [(u8, &str, &str, bool); 9] = [
    (9, "luck", "amount", false),
    (6, "morale", "amount", false),
    (1, "ranged_damage", "bp", true),
    (22, "melee_damage", "bp", true),
    (23, "damage_reduction", "bp", true),
    (4, "surrender_discount", "bp", true),
    (27, "first_aid", "heal_bp", true),
    (25, "spell_damage", "bp", true),
    (26, "magic_resistance", "bp", true),
];

/// Bits of the executable's spell flag word the harvester reads (installation.md §7.6).
const SPELL_COMBAT: u32 = 0x1;
const SPELL_ADVENTURE: u32 = 0x2;
const SPELL_ABILITY: u32 = 0x8;
const SPELL_SINGLE: u32 = 0x10;
const SPELL_MASS_ADVANCED: u32 = 0x20;
const SPELL_MASS_EXPERT: u32 = 0x40;
const SPELL_AREA: u32 = 0x80;
const SPELL_DAMAGE: u32 = 0x200;
const SPELL_MIND: u32 = 0x400;
const SPELL_NO_WAR_MACHINES: u32 = 0x1000;

/// The pack ranks: content may refer to its own pack and those below it.
const PACKS: [&str; 3] = ["base", "ab", "sod"];

/// A harvested creature as the hero content sees it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatureRef {
    /// Pack key (without the pack).
    pub key: String,
    /// The pack that defines it.
    pub pack: &'static str,
    /// It is a war machine.
    pub war_machine: bool,
    /// Singular name.
    pub name: String,
}

/// What the creature harvest hands over.
#[derive(Debug, Clone, Copy)]
pub struct Context<'a> {
    /// Harvested creatures by original id.
    pub creatures: &'a BTreeMap<usize, CreatureRef>,
    /// Town keys and their packs, by town index.
    pub towns: &'a [(String, &'static str)],
}

/// The text tables of the hero content.
#[derive(Debug, Clone)]
pub struct Tables {
    /// `HCTRAITS.TXT`.
    pub classes: TextTable,
    /// `HOTRAITS.TXT`.
    pub heroes: TextTable,
    /// `SSTRAITS.TXT`.
    pub skills: TextTable,
    /// `SPTRAITS.TXT` of the SoD archive.
    pub sod_spells: TextTable,
    /// `SPTRAITS.TXT` of the AB-era archive.
    pub ab_spells: TextTable,
}

/// The harvested hero content, as pack text per pack.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HeroPacks {
    /// Text appended to `base`.
    pub base: String,
    /// Text appended to `ab`.
    pub ab: String,
    /// Text appended to `sod`.
    pub sod: String,
}

impl HeroPacks {
    /// The text of `pack` (`base`, `ab`, anything else: `sod`).
    pub(crate) fn pack(&mut self, pack: &str) -> &mut String {
        match pack {
            "base" => &mut self.base,
            "ab" => &mut self.ab,
            _ => &mut self.sod,
        }
    }
}

fn rank(pack: &str) -> usize {
    PACKS.iter().position(|p| *p == pack).unwrap_or(PACKS.len())
}

/// A reference from content of `pack` to `key` of `owner`: bare within the pack, `owner:key`
/// otherwise; an error if `owner` is above `pack`.
pub(crate) fn reference(key: &str, owner: &str, pack: &str, what: &str) -> Result<String, String> {
    if rank(owner) > rank(pack) {
        return Err(format!(
            "{what}: `{owner}:{key}` is not available to `{pack}`"
        ));
    }
    Ok(if owner == pack {
        key.to_owned()
    } else {
        format!("{owner}:{key}")
    })
}

fn number<T: std::str::FromStr>(row: &[String], col: usize, what: &str) -> Result<T, String> {
    let text = row.get(col).map_or("", |s| s.trim());
    text.parse()
        .map_err(|_| format!("{what}: column {col} `{text}` is not a number"))
}

/// The rows of a table after its two header rows that have a name; `count`, if given, is the
/// number expected.
fn named_rows<'a>(
    table: &'a TextTable,
    count: Option<usize>,
    fields: usize,
    what: &str,
) -> Result<Vec<&'a [String]>, String> {
    let rows: Vec<&[String]> = table
        .records
        .iter()
        .skip(2)
        .filter(|r| r.first().is_some_and(|n| !n.trim().is_empty()))
        .map(Vec::as_slice)
        .collect();
    if let Some(count) = count.filter(|c| *c != rows.len()) {
        return Err(format!("{what}: {} rows, expected {count}", rows.len()));
    }
    if let Some(r) = rows.iter().find(|r| r.len() < fields) {
        return Err(format!(
            "{what}: row `{}` has {} fields",
            r[0].trim(),
            r.len()
        ));
    }
    Ok(rows)
}

/// A fraction of the executable as basis points; it must be a whole number of them.
fn basis_points(v: f32, what: &str) -> Result<u16, String> {
    let bp = f64::from(v) * 10_000.0;
    let rounded = bp.round();
    if (bp - rounded).abs() > 0.01 || !(0.0..=f64::from(u16::MAX)).contains(&rounded) {
        return Err(format!("{what}: {v} is not a whole number of basis points"));
    }
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "checked to be a whole number in u16 range"
    )]
    Ok(rounded as u16)
}

fn inline(kind: &str, params: &[(&str, String)]) -> String {
    let mut parts = vec![format!("kind = {}", quote(kind))];
    parts.extend(params.iter().map(|(k, v)| format!("{k} = {v}")));
    format!("{{ {} }}", parts.join(", "))
}

fn effect_list(effects: &[String]) -> String {
    format!("[{}]", effects.join(", "))
}

fn spec_list(specs: &[AbilitySpec]) -> Result<String, String> {
    let items = specs
        .iter()
        .map(|s| {
            if s.creature.is_some() {
                return Err("gapfill: skill effects take no creature".to_owned());
            }
            let params: Vec<(&str, String)> = s
                .params
                .iter()
                .map(|(k, v)| (k.as_str(), v.to_string()))
                .collect();
            Ok(inline(&s.kind, &params))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(effect_list(&items))
}

/// The upgrade relation: consecutive even/odd ids of one town and level (the town creatures'
/// pairs), plus the gap-fill pairs. Returns upgrades by original id.
///
/// # Errors
/// Gap-fill names that are not harvested creatures, or a pair the rule also finds.
pub fn upgrades(
    exe: &[CreatureRecord],
    creatures: &BTreeMap<usize, CreatureRef>,
    gap: &GapFill,
) -> Result<BTreeMap<usize, Vec<usize>>, String> {
    let mut out: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for (i, pair) in exe.chunks_exact(2).enumerate() {
        let (a, b) = (&pair[0], &pair[1]);
        let (ia, ib) = (2 * i, 2 * i + 1);
        if a.town >= 0
            && a.level >= 0
            && (a.town, a.level) == (b.town, b.level)
            && creatures.contains_key(&ia)
            && creatures.contains_key(&ib)
        {
            out.entry(ia).or_default().push(ib);
        }
    }
    let by_name = |n: &str| {
        creatures
            .iter()
            .find(|(_, c)| c.name == n)
            .map(|(&id, _)| id)
            .ok_or_else(|| format!("gapfill: upgrade names `{n}`, not a harvested creature"))
    };
    for [from, to] in &gap.heroes.upgrades {
        let (a, b) = (by_name(from)?, by_name(to)?);
        let list = out.entry(a).or_default();
        if list.contains(&b) {
            return Err(format!(
                "gapfill: upgrade `{from}` → `{to}` is found by the rule"
            ));
        }
        list.push(b);
    }
    Ok(out)
}

/// The secondary skill names of `SSTRAITS.TXT`, in id order (their keys are `base` keys).
///
/// # Errors
/// A malformed table.
pub fn skill_names(tables: &Tables) -> Result<Vec<String>, String> {
    Ok(named_rows(&tables.skills, None, 4, "SSTRAITS.TXT")?
        .iter()
        .map(|r| r[0].trim().to_owned())
        .collect())
}

/// Runs the hero-content harvest.
///
/// # Errors
/// Writes the target of a stack spell with an optional mass version (pack format v5): target
/// and mass version from the flag word (installation.md §7.6), the side from the polarity or,
/// for a neutral spell, `side`.
fn stack_spell(
    text: &mut String,
    what: &str,
    (flags, polarity): (u32, &str),
    side: Option<&str>,
) -> Result<(), String> {
    let mass = match (
        flags & SPELL_SINGLE != 0,
        flags & SPELL_MASS_ADVANCED != 0,
        flags & SPELL_MASS_EXPERT != 0,
    ) {
        (true, false, false) => None,
        (false, true, false) => Some("advanced"),
        (false, false, true) => Some("expert"),
        _ => {
            return Err(format!(
                "{what}: flag word {flags:#x} is no single-target timed spell"
            ));
        }
    };
    if flags & SPELL_AREA != 0 {
        return Err(format!("{what}: area stack spells are not harvested yet"));
    }
    let side = match (polarity, side) {
        ("positive", None) => "own",
        ("negative", None) => "enemy",
        ("neutral", Some(side @ ("own" | "enemy"))) => side,
        _ => {
            return Err(format!(
                "{what}: no side, or a side for a spell that has one"
            ));
        }
    };
    let _ = writeln!(text, "target = \"stack\"\nside = \"{side}\"");
    if let Some(m) = mass {
        let _ = writeln!(text, "mass = \"{m}\"");
    }
    Ok(())
}

/// The pack references of the spells named `names` from a spell of pack `pack`.
fn spell_refs(
    names: &[String],
    (what, pack): (&str, &str),
    spells: &BTreeMap<String, (String, &'static str)>,
) -> Result<Vec<String>, String> {
    names
        .iter()
        .map(|c| {
            let (key, owner) = spells
                .get(c.as_str())
                .ok_or_else(|| format!("gapfill: {what} names unknown spell `{c}`"))?;
            reference(key, owner, pack, what).map(|r| quote(&r))
        })
        .collect()
}

/// Writes the battle magic of a timed spell (pack format v5, E04b T3a): the target, the side,
/// the mass version, `mind` and the war-machine exclusion from the flag word (installation.md
/// §7.6, §7.9), the changes, duration, cumulation and counters from the gap-fill entry.
fn timed_spell(
    text: &mut String,
    (what, pack): (&str, &str),
    (flags, polarity): (u32, &str),
    t: &crate::gapfill::SpellTimed,
    spells: &BTreeMap<String, (String, &'static str)>,
) -> Result<(), String> {
    stack_spell(text, what, (flags, polarity), None)?;
    if flags & SPELL_MIND != 0 {
        let _ = writeln!(text, "mind = true");
    }
    if flags & SPELL_NO_WAR_MACHINES != 0 {
        let _ = writeln!(text, "excludes = [\"war_machine\"]");
    }
    match t.duration.as_deref() {
        None => {}
        Some(d @ ("permanent" | "own_turn")) => {
            let _ = writeln!(text, "duration = \"{d}\"");
        }
        Some(other) => return Err(format!("gapfill: duration `{other}` for {what}")),
    }
    if t.cumulative {
        let _ = writeln!(text, "cumulative = true");
    }
    if !t.counters.is_empty() {
        let keys = spell_refs(&t.counters, (what, pack), spells)?;
        let _ = writeln!(text, "counters = [{}]", keys.join(", "));
    }
    if t.changes.is_empty() {
        return Err(format!("gapfill: {what} has no timed changes"));
    }
    let effects: Vec<String> = t
        .changes
        .iter()
        .map(|c| format!("{{ kind = \"timed\", change = {} }}", quote(c)))
        .collect();
    let _ = writeln!(text, "effects = [{}]", effects.join(", "));
    Ok(())
}

/// Writes the battle magic of a stack spell of the other effect kinds (pack format v7, E04b
/// T3b1b): target, mass version and side as [`stack_spell`], the rest from the gap-fill entry.
fn other_spell(
    text: &mut String,
    (what, pack): (&str, &str),
    (flags, polarity): (u32, &str),
    t: &crate::gapfill::SpellOther,
    spells: &BTreeMap<String, (String, &'static str)>,
) -> Result<(), String> {
    stack_spell(text, what, (flags, polarity), t.side.as_deref())?;
    if let Some(m) = &t.any_side {
        let _ = writeln!(text, "any_side = {}", quote(m));
    }
    for (field, on) in [
        ("needs_effect", t.needs_effect),
        ("ignores_immunity", t.ignores_immunity),
    ] {
        if on {
            let _ = writeln!(text, "{field} = true");
        }
    }
    for (field, names) in [("dispels", &t.dispels), ("spares", &t.spares)] {
        if !names.is_empty() {
            let keys = spell_refs(names, (what, pack), spells)?;
            let _ = writeln!(text, "{field} = [{}]", keys.join(", "));
        }
    }
    if t.effects.is_empty() {
        return Err(format!("gapfill: {what} has no effects"));
    }
    let effects: Vec<String> = t
        .effects
        .iter()
        .map(|k| format!("{{ kind = {} }}", quote(k)))
        .collect();
    let _ = writeln!(text, "effects = [{}]", effects.join(", "));
    Ok(())
}

/// Malformed tables, values outside the expected ranges, references the packs cannot express, or
/// gap-fill entries that do not match the tables.
#[allow(
    clippy::too_many_lines,
    reason = "one linear pipeline per kind, split by comments"
)]
pub fn harvest_heroes(
    tables: &Tables,
    exe: &HeroData,
    gap: &GapFill,
    ctx: Context<'_>,
    report: &mut Vec<String>,
) -> Result<HeroPacks, String> {
    let mut out = HeroPacks::default();
    let creature = |id: i32, pack: &str, what: &str| -> Result<String, String> {
        let c = usize::try_from(id)
            .ok()
            .and_then(|i| ctx.creatures.get(&i))
            .ok_or_else(|| format!("{what}: creature id {id} is not harvested"))?;
        reference(&c.key, c.pack, pack, what)
    };

    // Secondary skills (base): names from SSTRAITS, battle effects from the executable's value
    // tables and the gap-fill file.
    let skill_rows = named_rows(&tables.skills, None, 4, "SSTRAITS.TXT")?;
    let skills: Vec<(String, String)> = skill_rows
        .iter()
        .map(|r| (key_of(&r[0]), r[0].trim().to_owned()))
        .collect();
    let skill_key = |id: i32, what: &str| -> Result<String, String> {
        usize::try_from(id)
            .ok()
            .and_then(|i| skills.get(i))
            .map(|(k, _)| k.clone())
            .ok_or_else(|| format!("{what}: skill id {id} out of range"))
    };
    let mut skill_levels: BTreeMap<usize, [String; 3]> = BTreeMap::new();
    for (skill, kind, param, fraction) in TABLE_SKILLS {
        let (_, values) = exe
            .skill_values
            .iter()
            .find(|(s, _)| *s == skill)
            .ok_or_else(|| format!("Heroes3.exe: no value table for skill {skill}"))?;
        let name = &skills
            .get(usize::from(skill))
            .ok_or_else(|| format!("SSTRAITS.TXT: no skill {skill}"))?
            .1;
        let mut levels: [String; 3] = Default::default();
        for (level, v) in values.iter().enumerate() {
            let rendered = match (*v, fraction) {
                (SkillValue::Float(f), true) => basis_points(f, name)?.to_string(),
                (SkillValue::Integer(i), false) => i.to_string(),
                _ => {
                    return Err(format!(
                        "Heroes3.exe: skill `{name}` has the wrong value type"
                    ));
                }
            };
            if level == 0 {
                if rendered != "0" {
                    return Err(format!(
                        "Heroes3.exe: skill `{name}` has a value without the skill"
                    ));
                }
                continue;
            }
            levels[level - 1] = effect_list(&[inline(kind, &[(param, rendered)])]);
        }
        skill_levels.insert(usize::from(skill), levels);
    }
    for e in &gap.heroes.skill_effects {
        let id = skills
            .iter()
            .position(|(_, n)| *n == e.skill)
            .ok_or_else(|| format!("gapfill: unknown skill `{}`", e.skill))?;
        let levels = [
            spec_list(&e.basic)?,
            spec_list(&e.advanced)?,
            spec_list(&e.expert)?,
        ];
        if skill_levels.insert(id, levels).is_some() {
            return Err(format!("gapfill: skill `{}` has a value table", e.skill));
        }
    }
    let unmodelled: Vec<&str> = skills
        .iter()
        .enumerate()
        .filter(|(i, _)| !skill_levels.contains_key(i))
        .map(|(_, (k, _))| k.as_str())
        .collect();
    report.push(format!(
        "skills without effects yet (E04b, E04c, outcome or adventure): {}",
        unmodelled.join(", ")
    ));
    for (i, (key, name)) in skills.iter().enumerate() {
        let _ = write!(
            out.base,
            "\n[[secondary_skill]]\nid = \"{key}\"\nname = {}\n",
            quote(name)
        );
        if let Some(levels) = skill_levels.get(&i) {
            for (label, list) in ["basic", "advanced", "expert"].iter().zip(levels) {
                let _ = writeln!(out.base, "{label} = {list}");
            }
        }
    }

    // Spell schools (base) and spells: the school columns' header names, the SoD table's rows;
    // rows that differ in the AB-era table are SoD's.
    let header = tables
        .sod_spells
        .records
        .get(1)
        .ok_or("SPTRAITS.TXT: no header row")?;
    let schools: Vec<String> = SCHOOL_COLUMNS
        .map(|c| header.get(c).map_or(String::new(), |h| h.trim().to_owned()))
        .collect();
    if schools.iter().any(String::is_empty) {
        return Err("SPTRAITS.TXT: school columns without names".into());
    }
    for s in &schools {
        let _ = write!(
            out.base,
            "\n[[spell_school]]\nid = \"{}\"\nname = {}\n",
            key_of(s),
            quote(s)
        );
        // The original's protection check order (BAT-MAG-013, quirk Q-0007).
        if let Some(rank) = gap.spells.protection_order.iter().position(|o| o == s) {
            let _ = writeln!(out.base, "protection_rank = {rank}");
        }
    }
    if let Some(unknown) = gap
        .spells
        .protection_order
        .iter()
        .find(|o| !schools.contains(o))
    {
        return Err(format!(
            "gapfill: protection order names unknown school `{unknown}`"
        ));
    }
    let spell_rows =
        |table: &TextTable, what: &str| -> Result<Vec<(String, Vec<String>)>, String> {
            let mut category = "";
            let mut rows = Vec::new();
            for r in table.records.iter().skip(2) {
                let name = r.first().map_or("", |n| n.trim());
                match name {
                    "Adventure Spells" => category = "adventure",
                    "Combat Spells" => category = "combat",
                    "Creature Abilities" => category = "ability",
                    "" => {}
                    _ if category.is_empty() => {
                        return Err(format!("{what}: spell `{name}` before a group row"));
                    }
                    _ => {
                        if r.len() < SPTRAITS_FIELDS {
                            return Err(format!("{what}: spell `{name}` has {} fields", r.len()));
                        }
                        rows.push((category.to_owned(), r.clone()));
                    }
                }
            }
            let count = exe.spell_types.len();
            if rows.len() == count {
                Ok(rows)
            } else {
                Err(format!(
                    "{what}: {} spells, the executable has {count}",
                    rows.len()
                ))
            }
        };
    let mut targeting: BTreeMap<&str, &crate::gapfill::SpellTargeting> = BTreeMap::new();
    for t in &gap.spells.targeting {
        if targeting.insert(t.spell.as_str(), t).is_some() {
            return Err(format!(
                "gapfill: spell `{}` has two targeting entries",
                t.spell
            ));
        }
    }
    let mut timed: BTreeMap<&str, &crate::gapfill::SpellTimed> = BTreeMap::new();
    for t in &gap.spells.timed {
        if timed.insert(t.spell.as_str(), t).is_some() {
            return Err(format!(
                "gapfill: spell `{}` has two timed entries",
                t.spell
            ));
        }
    }
    let mut damage_spells = 0;
    let mut other: BTreeMap<&str, &crate::gapfill::SpellOther> = BTreeMap::new();
    for t in &gap.spells.other {
        if other.insert(t.spell.as_str(), t).is_some() || timed.contains_key(t.spell.as_str()) {
            return Err(format!(
                "gapfill: spell `{}` has two effect entries",
                t.spell
            ));
        }
    }
    let mut specialist: BTreeMap<&str, &toml::Value> = BTreeMap::new();
    for t in &gap.spells.specialist {
        for name in &t.spells {
            if specialist.insert(name.as_str(), &t.bonus).is_some() {
                return Err(format!(
                    "gapfill: spell `{name}` has two specialist entries"
                ));
            }
        }
    }
    let mut timed_spells = 0;
    let mut other_spells = 0;
    let mut specialists = 0;
    let sod_spells = spell_rows(&tables.sod_spells, "SPTRAITS.TXT")?;
    let ab_spells = spell_rows(&tables.ab_spells, "H3ab_bmp/SPTRAITS.TXT")?;
    // Every spell's key and pack by name (counters may name a later spell).
    let spell_packs: BTreeMap<String, (String, &'static str)> = sod_spells
        .iter()
        .zip(&ab_spells)
        .map(|((_, row), (_, ab_row))| {
            let name = row[0].trim();
            let pack = if ab_row[0].trim() == name {
                "base"
            } else {
                "sod"
            };
            (name.to_owned(), (key_of(name), pack))
        })
        .collect();
    let mut spell_keys: Vec<(String, &'static str)> = Vec::with_capacity(sod_spells.len());
    let mut seen = BTreeSet::new();
    for (id, ((category, row), (_, ab_row))) in sod_spells.iter().zip(&ab_spells).enumerate() {
        let name = row[0].trim();
        let key = key_of(name);
        if !seen.insert(key.clone()) {
            return Err(format!("duplicate spell key `{key}`"));
        }
        let pack = if ab_row[0].trim() == name {
            if ab_row[..29] != row[..29] {
                return Err(format!(
                    "spell `{name}` differs between H3ab_bmp and H3bitmap"
                ));
            }
            "base"
        } else {
            report.push(format!(
                "spell id {id}: `{}` in the AB-era table, `{name}` in SoD; the SoD row goes into `sod`",
                ab_row[0].trim()
            ));
            "sod"
        };
        let what = format!("spell `{name}`");
        let level: u8 = number(row, 2, &what)?;
        let school_keys: Vec<String> = SCHOOL_COLUMNS
            .zip(&schools)
            .filter(|(c, _)| row[*c].trim() == "x")
            .map(|(_, s)| reference(&key_of(s), "base", pack, &what).map(|r| quote(&r)))
            .collect::<Result<_, _>>()?;
        let numbers = |cols: std::ops::Range<usize>| -> Result<String, String> {
            let v = cols
                .map(|c| number::<u16>(row, c, &what).map(|n| n.to_string()))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(format!("[{}]", v.join(", ")))
        };
        let polarity = match exe.spell_types[id] {
            -1 => "negative",
            0 => "neutral",
            _ => "positive",
        };
        let text = out.pack(pack);
        let _ = write!(
            text,
            "\n[[spell]]\nid = \"{key}\"  # original id {id}\nname = {}\n",
            quote(name)
        );
        let _ = writeln!(text, "category = \"{category}\"");
        let _ = writeln!(text, "level = {level}");
        if !school_keys.is_empty() {
            let _ = writeln!(text, "schools = [{}]", school_keys.join(", "));
        }
        let _ = writeln!(text, "cost = {}", numbers(7..11)?);
        let _ = writeln!(text, "power = {}", number::<u16>(row, 11, &what)?);
        let _ = writeln!(text, "effect = {}", numbers(12..16)?);
        let _ = writeln!(text, "polarity = \"{polarity}\"");
        // Battle magic (pack format v4, E04b T2a): the damage spells' target from the flag word,
        // areas, chains and conditions from the gap-fill file; other kinds come with their tasks.
        let flags = exe
            .spell_flags
            .get(id)
            .copied()
            .ok_or_else(|| format!("{what}: no flag word"))?;
        let expected = match category.as_str() {
            "adventure" => SPELL_ADVENTURE,
            "combat" => SPELL_COMBAT,
            _ => SPELL_ABILITY,
        };
        if flags & (SPELL_COMBAT | SPELL_ADVENTURE | SPELL_ABILITY) != expected {
            return Err(format!(
                "{what}: flag word {flags:#x} does not match category {category}"
            ));
        }
        let extra = targeting.remove(name);
        let with_effects = damage_spells + timed_spells + other_spells;
        if category == "combat" && flags & SPELL_DAMAGE != 0 {
            let target = match (flags & SPELL_SINGLE != 0, flags & SPELL_AREA != 0) {
                (false, false) => "none",
                (true, false) => "stack",
                (false, true) => "hex",
                (true, true) => return Err(format!("{what}: single target and area")),
            };
            if target != "none" {
                let _ = writeln!(text, "target = \"{target}\"");
            }
            // A single-target spell that harms reaches only the enemy (the original's target
            // test, installation.md §7.8); the gap-fill file names the exceptions.
            let derived = (target == "stack" && polarity == "negative").then_some("enemy");
            let given = extra.and_then(|t| t.side.as_deref());
            match (derived, given) {
                (Some(side), None) => {
                    let _ = writeln!(text, "side = \"{side}\"");
                }
                (None, Some(side @ ("enemy" | "own"))) if target == "stack" => {
                    let _ = writeln!(text, "side = \"{side}\"");
                }
                (None, None) => {}
                _ => return Err(format!("gapfill: side for `{name}` is not allowed")),
            }
            if let Some(t) = extra {
                if let Some(area) = &t.area {
                    let _ = writeln!(text, "area = {area}");
                }
                if let Some(chain) = &t.chain {
                    let _ = writeln!(text, "chain = {chain}");
                }
                for (field, list) in [("requires", &t.requires), ("excludes", &t.excludes)] {
                    if !list.is_empty() {
                        let items: Vec<String> = list.iter().map(|f| quote(f)).collect();
                        let _ = writeln!(text, "{field} = [{}]", items.join(", "));
                    }
                }
            }
            let _ = writeln!(text, "effects = [{{ kind = \"damage\" }}]");
            damage_spells += 1;
        } else if let Some(t) = timed.remove(name) {
            if category != "combat" || extra.is_some() {
                return Err(format!("gapfill: timed entry for `{name}` is not allowed"));
            }
            timed_spell(text, (&what, pack), (flags, polarity), t, &spell_packs)?;
            timed_spells += 1;
        } else if let Some(t) = other.remove(name) {
            if category != "combat" || extra.is_some() {
                return Err(format!("gapfill: effect entry for `{name}` is not allowed"));
            }
            other_spell(text, (&what, pack), (flags, polarity), t, &spell_packs)?;
            other_spells += 1;
        } else if extra.is_some() {
            return Err(format!(
                "gapfill: targeting for `{name}`, which has no harvested effects"
            ));
        }
        if let Some(bonus) = specialist.remove(name) {
            specialists += 1;
            if damage_spells + timed_spells + other_spells == with_effects {
                return Err(format!(
                    "gapfill: specialist for `{name}`, which has no effects"
                ));
            }
            let _ = writeln!(text, "specialist = {bonus}");
        }
        spell_keys.push((key, pack));
    }
    if let Some(name) = timed
        .keys()
        .chain(other.keys())
        .chain(specialist.keys())
        .next()
    {
        return Err(format!("gapfill: effect entry for unknown spell `{name}`"));
    }
    if let Some(name) = targeting.keys().next() {
        return Err(format!("gapfill: targeting for unknown spell `{name}`"));
    }
    report.push(format!(
        "spells: target and damage effect of {damage_spells} damage spells from the flag word; \
         target, side, mass version and conditions of {timed_spells} timed spells from the flag \
         word, their changes from the gap-fill file; target and mass version of {other_spells} \
         dispel and heal spells from the flag word, the rest from the gap-fill file; \
         {specialists} specialist bonuses from the gap-fill file; other effect kinds come with \
         their tasks (E04b T4-T7)"
    ));
    report.push(
        "SPTRAITS: town chances, AI values, texts and the attribute column not harvested (E04b, E09)"
            .into(),
    );
    let spell_ref = |id: i32, pack: &str, what: &str| -> Result<String, String> {
        let (key, owner) = usize::try_from(id)
            .ok()
            .and_then(|i| spell_keys.get(i))
            .ok_or_else(|| format!("{what}: spell id {id} out of range"))?;
        reference(key, owner, pack, what)
    };

    // Hero classes: the pack of their faction (class index / 2 = town index, the tables' order).
    let class_rows = named_rows(
        &tables.classes,
        None,
        HCTRAITS_FIXED + skills.len(),
        "HCTRAITS.TXT",
    )?;
    let mut classes: Vec<(String, &'static str)> = Vec::with_capacity(class_rows.len());
    for (i, row) in class_rows.iter().enumerate() {
        let name = row[0].trim();
        let what = format!("hero class `{name}`");
        let (town, pack) = ctx
            .towns
            .get(i / 2)
            .ok_or_else(|| format!("{what}: no town {}", i / 2))?;
        let key = key_of(name);
        let n = |cols: std::ops::Range<usize>| -> Result<String, String> {
            let v = cols
                .map(|c| number::<u8>(row, c, &what).map(|n| n.to_string()))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(format!("[{}]", v.join(", ")))
        };
        let text = out.pack(pack);
        let _ = write!(
            text,
            "\n[[hero_class]]\nid = \"{key}\"\nname = {}\n",
            quote(name)
        );
        let _ = writeln!(text, "faction = \"{town}\"");
        let _ = writeln!(text, "primary = {}", n(2..6)?);
        let _ = writeln!(text, "primary_gain = [{}, {}]", n(6..10)?, n(10..14)?);
        let mut weights = Vec::new();
        for (s, (skill, _)) in skills.iter().enumerate() {
            let w: u8 = number(row, HCTRAITS_FIXED + s, &what)?;
            if w > 0 {
                weights.push(format!(
                    "{} = {w}",
                    quote(&reference(skill, "base", pack, &what)?)
                ));
            }
        }
        let _ = writeln!(text, "skill_weights = {{ {} }}", weights.join(", "));
        classes.push((key, pack));
    }
    report.push("HCTRAITS: aggression and tavern weights not harvested (E09)".into());

    // Hero types: the executable's hero and specialty tables, army sizes from HOTRAITS.
    let hero_rows = named_rows(
        &tables.heroes,
        Some(exe.heroes.len()),
        HOTRAITS_FIELDS,
        "HOTRAITS.TXT",
    )?;
    let growth = basis_points(exe.specialty_growth, "skill specialty growth")?;
    let names: Vec<&str> = hero_rows.iter().map(|r| r[0].trim()).collect();
    let mut keys = BTreeSet::new();
    let mut extras: BTreeMap<&str, Vec<&crate::gapfill::SpecialtyExtra>> = BTreeMap::new();
    for e in &gap.heroes.specialty_extra {
        if !names.contains(&e.hero.as_str()) {
            return Err(format!(
                "gapfill: specialty_extra for unknown hero `{}`",
                e.hero
            ));
        }
        extras.entry(e.hero.as_str()).or_default().push(e);
    }
    let mut unmodelled: BTreeMap<&str, usize> = BTreeMap::new();
    let mut fixed_used = 0;
    for (id, ((row, rec), spec)) in hero_rows
        .iter()
        .zip(&exe.heroes)
        .zip(&exe.specialties)
        .enumerate()
    {
        let name = names[id];
        let what = format!("hero `{name}`");
        let pack = match (rec.roe, rec.expansions) {
            (true, _) => "base",
            (false, true) => "ab",
            (false, false) => return Err(format!("{what}: neither RoE nor an expansion hero")),
        };
        let (class, class_pack) = usize::try_from(rec.class)
            .ok()
            .and_then(|c| classes.get(c))
            .ok_or_else(|| format!("{what}: class {} out of range", rec.class))?;
        let mut key = key_of(name);
        if !keys.insert(key.clone()) {
            // Two heroes share a name (Lord Haart the Knight and the Death Knight).
            key = format!("{key}_{class}");
            if !keys.insert(key.clone()) {
                return Err(format!("duplicate hero key `{key}`"));
            }
            report.push(format!("hero id {id} `{name}`: key `{key}` (name shared)"));
        }
        let text = out.pack(pack);
        let _ = write!(
            text,
            "\n[[hero]]\nid = \"{key}\"  # original id {id}\nname = {}\n",
            quote(name)
        );
        let _ = writeln!(
            text,
            "class = \"{}\"",
            reference(class, class_pack, pack, &what)?
        );
        let mut skill_list = Vec::new();
        for &(s, level) in &rec.skills {
            if s >= 0 {
                let r = reference(&skill_key(s, &what)?, "base", pack, &what)?;
                skill_list.push(format!("{{ skill = \"{r}\", level = {level} }}"));
            }
        }
        let _ = writeln!(text, "skills = [{}]", skill_list.join(", "));
        if rec.spell_book {
            let _ = writeln!(text, "spell_book = true");
        }
        if rec.spell >= 0 {
            let _ = writeln!(
                text,
                "spells = [\"{}\"]",
                spell_ref(rec.spell, pack, &what)?
            );
        }
        let mut army = Vec::new();
        let mut machines = Vec::new();
        for (j, &c) in rec.creatures.iter().enumerate() {
            let low: u16 = number(row, 1 + 3 * j, &what)?;
            let high: u16 = number(row, 2 + 3 * j, &what)?;
            if c < 0 || low == 0 {
                return Err(format!("{what}: army entry {j} is empty"));
            }
            let r = creature(c, pack, &what)?;
            let machine = usize::try_from(c)
                .ok()
                .and_then(|i| ctx.creatures.get(&i))
                .is_some_and(|m| m.war_machine);
            if machine {
                if (low, high) != (1, 1) {
                    return Err(format!("{what}: war machine with count {low}-{high}"));
                }
                machines.push(format!("\"{r}\""));
            } else {
                army.push(format!("{{ creature = \"{r}\", count = [{low}, {high}] }}"));
            }
        }
        let _ = writeln!(text, "army = [{}]", army.join(", "));
        if !machines.is_empty() {
            let _ = writeln!(text, "war_machines = [{}]", machines.join(", "));
        }
        // The specialty.
        let mut effects = Vec::new();
        let stat = |creatures: &str, stat: &str, amount: i32| {
            let mut params = Vec::new();
            if !creatures.is_empty() {
                params.push(("creatures", creatures.to_owned()));
            }
            params.push(("stat", quote(stat)));
            params.push(("amount", amount.to_string()));
            inline("creature_stat", &params)
        };
        match spec.kind {
            0 => effects.push(inline(
                "skill_specialty",
                &[
                    (
                        "skill",
                        quote(&reference(
                            &skill_key(spec.id, &what)?,
                            "base",
                            pack,
                            &what,
                        )?),
                    ),
                    ("bp_per_level", growth.to_string()),
                ],
            )),
            1 => {
                let c = usize::try_from(spec.id)
                    .ok()
                    .and_then(|i| ctx.creatures.get(&i))
                    .ok_or_else(|| format!("{what}: specialty creature {}", spec.id))?;
                let cs = &gap.heroes.creature_specialty;
                let speed = if c.war_machine {
                    cs.war_machine_speed
                } else {
                    cs.speed
                };
                effects.push(inline(
                    "creature_specialty",
                    &[
                        ("creature", quote(&creature(spec.id, pack, &what)?)),
                        ("bp_per_step", cs.bp_per_step.to_string()),
                        ("speed", speed.to_string()),
                    ],
                ));
            }
            3 => {
                let spell = quote(&spell_ref(spec.id, pack, &what)?);
                let ss = &gap.heroes.spell_specialty;
                effects.push(match ss.fixed.iter().find(|f| f.hero == name) {
                    Some(f) => {
                        fixed_used += 1;
                        inline(
                            "spell_damage",
                            &[("bp", f.bp.to_string()), ("spell", spell)],
                        )
                    }
                    None => inline(
                        "spell_specialty",
                        &[
                            ("spell", spell),
                            ("bp_per_level", ss.bp_per_level.to_string()),
                        ],
                    ),
                });
            }
            4 => {
                let sel = format!(
                    "{{ creature = {} }}",
                    quote(&creature(spec.id, pack, &what)?)
                );
                for (s, amount) in [
                    ("attack", spec.attack),
                    ("defense", spec.defense),
                    ("min_damage", spec.damage),
                    ("max_damage", spec.damage),
                ] {
                    if amount != 0 {
                        effects.push(stat(&sel, s, amount));
                    }
                }
            }
            5 => effects.push(stat("", "speed", spec.id)),
            7 => {
                for (s, amount) in [("attack", spec.attack), ("defense", spec.defense)] {
                    effects.push(stat("{ flag = \"dragon\" }", s, amount));
                }
            }
            kind => {
                let label = match kind {
                    2 => "resource (adventure)",
                    6 => "creature upgrade (adventure)",
                    _ => "none",
                };
                *unmodelled.entry(label).or_default() += 1;
            }
        }
        for e in extras.get(name).into_iter().flatten() {
            let (cid, _) = ctx
                .creatures
                .iter()
                .find(|(_, c)| c.name == e.creature)
                .ok_or_else(|| format!("gapfill: specialty_extra creature `{}`", e.creature))?;
            let id = i32::try_from(*cid).map_err(|_| "creature id".to_owned())?;
            let sel = format!("{{ creature = {} }}", quote(&creature(id, pack, &what)?));
            effects.push(stat(&sel, &e.stat, i32::from(e.amount)));
        }
        if !effects.is_empty() {
            let _ = writeln!(text, "specialty = {}", effect_list(&effects));
        }
    }
    if fixed_used != gap.heroes.spell_specialty.fixed.len() {
        return Err("gapfill: a fixed spell specialty names no spell specialist".into());
    }
    let counts: Vec<String> = unmodelled.iter().map(|(k, n)| format!("{n} {k}")).collect();
    report.push(format!("specialties not modelled: {}", counts.join(", ")));
    report.push(
        "heroes: availability (campaign heroes, RoE-only Lord Haart the Knight) not harvested (E08-E10)"
            .into(),
    );
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn references_follow_the_pack_order() {
        assert_eq!(reference("k", "base", "base", "x").unwrap(), "k");
        assert_eq!(reference("k", "base", "ab", "x").unwrap(), "base:k");
        assert!(reference("k", "sod", "ab", "x").is_err());
    }

    #[test]
    fn basis_points_need_whole_values() {
        assert_eq!(basis_points(0.1, "x").unwrap(), 1000);
        assert_eq!(basis_points(0.05, "x").unwrap(), 500);
        assert_eq!(basis_points(3.0, "x").unwrap(), 30_000);
        assert!(basis_points(0.123_45, "x").is_err());
        assert!(basis_points(-1.0, "x").is_err());
        assert!(basis_points(7.0, "x").is_err());
    }
}
