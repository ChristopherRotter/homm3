//! Artifacts (E04a T3b, pack format v3): equipment slots and artifact types, harvested from
//! `ARTRAITS.TXT` and `ArtSlots.txt` of both archives and the executable's artifact primary-skill
//! table (`docs/formats/installation.md` §6, §7.3), with the gap-fill file's `[artifacts]`
//! section for membership, war machines, combinations and the other effects.
//!
//! Slots: the positions of `ArtSlots.txt` form slot kinds ([`SLOT_KINDS`]); every artifact must
//! flag either no position or exactly all positions of one kind, so a kind is a set of
//! interchangeable positions. The AB-era table has 18 positions (four misc), SoD 19: `base` gets
//! the AB-era counts and `sod` overrides what changed. Rows 0–128 must agree between the two
//! tables except for the SoD-only positions.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use h3_formats::text::TextTable;

use crate::gapfill::{AbilitySpec, GapFill};
use crate::heroes::{Context, HeroPacks, reference};
use crate::{key_of, quote};

/// `ArtSlots.txt` position names and the slot kind (key, name) each belongs to.
pub const SLOT_KINDS: [(&str, &str, &str); 19] = [
    ("Head", "head", "Head"),
    ("Shoulders", "shoulders", "Shoulders"),
    ("Neck", "neck", "Neck"),
    ("Right Hand", "right_hand", "Right Hand"),
    ("Left Hand", "left_hand", "Left Hand"),
    ("Torso", "torso", "Torso"),
    ("Right Ring", "ring", "Ring"),
    ("Left Ring", "ring", "Ring"),
    ("Feet", "feet", "Feet"),
    ("Misc 1", "misc", "Misc"),
    ("Misc 2", "misc", "Misc"),
    ("Misc 3", "misc", "Misc"),
    ("Misc 4", "misc", "Misc"),
    ("Misc 5", "misc", "Misc"),
    ("War Machine 1", "war_machine_1", "War Machine 1"),
    ("War Machine 2", "war_machine_2", "War Machine 2"),
    ("War Machine 3", "war_machine_3", "War Machine 3"),
    ("War Machine 4", "war_machine_4", "War Machine 4"),
    ("Spell Book", "spell_book", "Spell Book"),
];

/// The artifact text tables.
#[derive(Debug, Clone)]
pub struct Tables {
    /// `ARTRAITS.TXT` of the SoD archive.
    pub sod: TextTable,
    /// `ARTRAITS.TXT` of the AB-era archive.
    pub ab: TextTable,
    /// `ArtSlots.txt` of the SoD archive.
    pub sod_slots: TextTable,
    /// `ArtSlots.txt` of the AB-era archive.
    pub ab_slots: TextTable,
}

/// One parsed artifact row.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Row {
    name: String,
    cost: Option<u32>,
    /// Flagged positions, by name.
    positions: BTreeSet<String>,
    class: &'static str,
}

/// The slot kind of a position name.
fn kind_of(position: &str) -> Result<(&'static str, &'static str), String> {
    SLOT_KINDS
        .iter()
        .find(|(p, _, _)| *p == position)
        .map(|(_, k, n)| (*k, *n))
        .ok_or_else(|| format!("ArtSlots.txt: unknown position `{position}`"))
}

/// The position names of an `ArtSlots.txt`.
fn positions(table: &TextTable, what: &str) -> Result<Vec<String>, String> {
    let names: Vec<String> = table
        .records
        .iter()
        .filter_map(|r| r.first())
        .map(|n| n.trim().to_owned())
        .filter(|n| !n.is_empty())
        .collect();
    if names.is_empty() {
        return Err(format!("{what}: no positions"));
    }
    Ok(names)
}

/// The artifact rows of an `ARTRAITS.TXT` (the header row names the position columns).
fn rows(table: &TextTable, positions: &[String], what: &str) -> Result<Vec<Row>, String> {
    let header = table
        .records
        .get(1)
        .ok_or_else(|| format!("{what}: no header row"))?;
    let column = |name: &str| {
        header
            .iter()
            .position(|h| h.trim() == name)
            .ok_or_else(|| format!("{what}: no column `{name}`"))
    };
    let columns: Vec<(String, usize)> = positions
        .iter()
        .map(|p| Ok((p.clone(), column(p)?)))
        .collect::<Result<_, String>>()?;
    let class_col = header
        .iter()
        .position(|h| h.trim().starts_with("Class"))
        .ok_or_else(|| format!("{what}: no class column"))?;
    let mut out = Vec::new();
    for r in table.records.iter().skip(2) {
        let name = r.first().map_or("", |n| n.trim());
        if name.is_empty() {
            continue;
        }
        let cell = |c: usize| r.get(c).map_or("", |v| v.trim());
        let cost = match cell(1) {
            "" => None,
            text => Some(
                text.parse()
                    .map_err(|_| format!("{what}: `{name}` has cost `{text}`"))?,
            ),
        };
        let positions = columns
            .iter()
            .filter(|(_, c)| cell(*c) == "x")
            .map(|(p, _)| p.clone())
            .collect();
        let class = match cell(class_col) {
            "S" => "special",
            "T" => "treasure",
            "N" => "minor",
            "J" => "major",
            "R" => "relic",
            other => return Err(format!("{what}: `{name}` has class `{other}`")),
        };
        out.push(Row {
            name: name.to_owned(),
            cost,
            positions,
            class,
        });
    }
    Ok(out)
}

/// The slot kind an artifact occupies: none if no position is flagged, else the kind whose
/// positions (in `all`) are exactly the flagged ones.
fn slot_of(row: &Row, all: &[String]) -> Result<Option<&'static str>, String> {
    let Some(first) = row.positions.iter().next() else {
        return Ok(None);
    };
    let (kind, _) = kind_of(first)?;
    let of_kind: BTreeSet<String> = all
        .iter()
        .filter(|p| kind_of(p).is_ok_and(|(k, _)| k == kind))
        .cloned()
        .collect();
    if of_kind == row.positions {
        Ok(Some(kind))
    } else {
        Err(format!(
            "ARTRAITS.TXT: `{}` flags {:?}, not the positions of one slot",
            row.name, row.positions
        ))
    }
}

/// Renders a gap-fill effect: creature and faction names become keys, the rest is copied.
fn render(spec: &AbilitySpec, pack: &str, ctx: Context<'_>, what: &str) -> Result<String, String> {
    if spec.creature.is_some() {
        return Err(format!(
            "gapfill: {what}: artifact effects take no `creature`"
        ));
    }
    let mut parts = vec![format!("kind = {}", quote(&spec.kind))];
    // Selector and statistic first, as in the pack docs; the rest in key order.
    let mut params: Vec<(&String, &toml::Value)> = spec.params.iter().collect();
    params.sort_by_key(|(k, _)| {
        (
            !matches!(k.as_str(), "creatures" | "stat"),
            k.as_str() != "creatures",
        )
    });
    for (k, v) in params {
        let value = match (k.as_str(), v) {
            ("creatures", toml::Value::Table(t)) => match (t.get("creature"), t.get("flag")) {
                (Some(toml::Value::String(name)), None) if t.len() == 1 => {
                    let c = ctx
                        .creatures
                        .values()
                        .find(|c| c.name == *name)
                        .ok_or_else(|| format!("gapfill: {what}: unknown creature `{name}`"))?;
                    format!(
                        "{{ creature = {} }}",
                        quote(&reference(&c.key, c.pack, pack, what)?)
                    )
                }
                (None, Some(toml::Value::String(flag))) if t.len() == 1 => {
                    format!("{{ flag = {} }}", quote(flag))
                }
                _ => return Err(format!("gapfill: {what}: malformed `creatures`")),
            },
            ("factions", toml::Value::Array(list)) => {
                let keys =
                    list.iter()
                        .map(|n| {
                            let name = n.as_str().unwrap_or_default();
                            let key = key_of(name);
                            let (_, owner) =
                                ctx.towns.iter().find(|(t, _)| *t == key).ok_or_else(|| {
                                    format!("gapfill: {what}: unknown town `{name}`")
                                })?;
                            Ok(quote(&reference(&key, owner, pack, what)?))
                        })
                        .collect::<Result<Vec<_>, String>>()?;
                format!("[{}]", keys.join(", "))
            }
            (_, v) => v.to_string(),
        };
        parts.push(format!("{k} = {value}"));
    }
    Ok(format!("{{ {} }}", parts.join(", ")))
}

/// Runs the artifact harvest.
///
/// `primary` is the executable's table (one entry per SoD row); `skills` the secondary skill
/// names in `SSTRAITS.TXT` order (keys of `base`).
///
/// # Errors
/// Malformed or inconsistent tables, rows the slot kinds cannot express, gap-fill entries that
/// name unknown artifacts, creatures, towns or skills, and artifacts without a modelled effect
/// that no `pending` list names.
#[allow(
    clippy::too_many_lines,
    reason = "one linear pipeline, split by comments"
)]
pub fn harvest_artifacts(
    tables: &Tables,
    primary: &[[i8; 4]],
    skills: &[String],
    gap: &GapFill,
    ctx: Context<'_>,
    report: &mut Vec<String>,
) -> Result<HeroPacks, String> {
    let ga = &gap.artifacts;
    let mut out = HeroPacks::default();

    // Slots: kinds in order of first appearance; `base` counts the AB-era positions, `sod`
    // overrides changed counts and adds new kinds.
    let ab_pos = positions(&tables.ab_slots, "H3ab_bmp/ArtSlots.txt")?;
    let sod_pos = positions(&tables.sod_slots, "ArtSlots.txt")?;
    if !sod_pos.starts_with(&ab_pos) {
        return Err("ArtSlots.txt: the AB-era positions are not a prefix of SoD's".into());
    }
    let count = |list: &[String]| -> Result<BTreeMap<&'static str, (usize, &'static str)>, String> {
        let mut m: BTreeMap<&'static str, (usize, &'static str)> = BTreeMap::new();
        for p in list {
            let (k, n) = kind_of(p)?;
            m.entry(k).or_insert((0, n)).0 += 1;
        }
        Ok(m)
    };
    let (ab_counts, sod_counts) = (count(&ab_pos)?, count(&sod_pos)?);
    let mut kinds_in_order: Vec<&'static str> = Vec::new();
    for p in &sod_pos {
        let (k, _) = kind_of(p)?;
        if !kinds_in_order.contains(&k) {
            kinds_in_order.push(k);
        }
    }
    // Slot kinds whose artifacts never go to the backpack (HER-ART-002).
    if let Some(k) = ga
        .no_backpack
        .iter()
        .find(|k| !kinds_in_order.contains(&k.as_str()))
    {
        return Err(format!(
            "gapfill: no_backpack names unknown slot kind `{k}`"
        ));
    }
    let flag = |k: &str| {
        if ga.no_backpack.iter().any(|n| n == k) {
            "backpack = false\n"
        } else {
            ""
        }
    };
    for k in &kinds_in_order {
        let (sod_n, name) = sod_counts[k];
        let Some(&(ab_n, _)) = ab_counts.get(k) else {
            let _ = write!(
                out.sod,
                "\n[[artifact_slot]]\nid = \"{k}\"\nname = {}\ncount = {sod_n}\n{}",
                quote(name),
                flag(k)
            );
            continue;
        };
        let _ = write!(
            out.base,
            "\n[[artifact_slot]]\nid = \"{k}\"\nname = {}\ncount = {ab_n}\n{}",
            quote(name),
            flag(k)
        );
        if ab_n != sod_n {
            let _ = write!(
                out.sod,
                "\n[[artifact_slot_override]]\ntarget = \"base:{k}\"\ncount = {sod_n}\n"
            );
            report.push(format!(
                "slot `{k}`: {ab_n} positions before SoD, {sod_n} in SoD"
            ));
        }
    }
    let slot_pack = |k: &str| {
        if ab_counts.contains_key(k) {
            "base"
        } else {
            "sod"
        }
    };

    // Rows: the SoD table, checked against the AB-era table and the executable.
    let sod_rows = rows(&tables.sod, &sod_pos, "ARTRAITS.TXT")?;
    let ab_rows = rows(&tables.ab, &ab_pos, "H3ab_bmp/ARTRAITS.TXT")?;
    if sod_rows.len() != primary.len() {
        return Err(format!(
            "ARTRAITS.TXT: {} artifacts, the executable has {}",
            sod_rows.len(),
            primary.len()
        ));
    }
    let sod_only: BTreeSet<&str> = sod_pos[ab_pos.len()..].iter().map(String::as_str).collect();
    for (id, (sod, ab)) in sod_rows.iter().zip(&ab_rows).enumerate() {
        let mut ab_view = sod.clone();
        ab_view.positions.retain(|p| !sod_only.contains(p.as_str()));
        if *ab != ab_view {
            return Err(format!(
                "artifact id {id} `{}` differs between H3ab_bmp and H3bitmap",
                sod.name
            ));
        }
    }
    let names: Vec<&str> = sod_rows.iter().map(|r| r.name.as_str()).collect();
    let id_of = |name: &str, what: &str| {
        names
            .iter()
            .position(|n| *n == name)
            .ok_or_else(|| format!("gapfill: {what}: unknown artifact `{name}`"))
    };

    // Membership.
    let mut pack_of: Vec<&'static str> = vec!["sod"; sod_rows.len()];
    for p in pack_of.iter_mut().take(ab_rows.len()) {
        *p = "base";
    }
    for name in &ga.ab {
        let id = id_of(name, "artifacts.ab")?;
        if id >= ab_rows.len() {
            return Err(format!(
                "gapfill: artifacts.ab: `{name}` is not in the AB-era table"
            ));
        }
        pack_of[id] = "ab";
    }
    let mut skipped = BTreeSet::new();
    for name in &ga.unused {
        skipped.insert(id_of(name, "artifacts.unused")?);
    }
    report.push(format!(
        "artifacts not harvested (unused traces): {}",
        ga.unused.join(", ")
    ));
    let mut keys: Vec<String> = Vec::with_capacity(names.len());
    let mut seen = BTreeSet::new();
    for name in &names {
        let key = key_of(name);
        if !seen.insert(key.clone()) {
            return Err(format!("duplicate artifact key `{key}`"));
        }
        keys.push(key);
    }
    let artifact_ref = |id: usize, pack: &str, what: &str| -> Result<String, String> {
        if skipped.contains(&id) {
            return Err(format!("gapfill: {what}: `{}` is not harvested", names[id]));
        }
        reference(&keys[id], pack_of[id], pack, what)
    };

    // Gap-fill facts by artifact id.
    let mut machines: BTreeMap<usize, String> = BTreeMap::new();
    for w in &ga.war_machines {
        let id = id_of(&w.artifact, "artifacts.war_machines")?;
        let (_, c) = ctx
            .creatures
            .iter()
            .find(|(_, c)| c.name == w.creature)
            .ok_or_else(|| format!("gapfill: war machine `{}` is not harvested", w.creature))?;
        if !c.war_machine {
            return Err(format!("gapfill: `{}` is not a war machine", w.creature));
        }
        let r = reference(&c.key, c.pack, pack_of[id], &w.artifact)?;
        if machines.insert(id, r).is_some() {
            return Err(format!("gapfill: `{}` has two war machines", w.artifact));
        }
    }
    let mut parts: BTreeMap<usize, Vec<String>> = BTreeMap::new();
    for c in &ga.combinations {
        let id = id_of(&c.artifact, "artifacts.combinations")?;
        let list = c
            .parts
            .iter()
            .map(|p| artifact_ref(id_of(p, &c.artifact)?, pack_of[id], &c.artifact))
            .collect::<Result<Vec<_>, String>>()?;
        if parts.insert(id, list).is_some() {
            return Err(format!("gapfill: `{}` is listed twice", c.artifact));
        }
    }
    let mut effects: BTreeMap<usize, (Vec<String>, Option<String>)> = BTreeMap::new();
    for e in &ga.effects {
        for name in &e.artifacts {
            let id = id_of(name, "artifacts.effects")?;
            let pack = pack_of[id];
            let list = e
                .effects
                .iter()
                .map(|s| render(s, pack, ctx, name))
                .collect::<Result<Vec<_>, String>>()?;
            let skill = e
                .needs_skill
                .as_ref()
                .map(|s| {
                    skills
                        .iter()
                        .find(|n| *n == s)
                        .ok_or_else(|| format!("gapfill: `{name}` needs unknown skill `{s}`"))
                        .and_then(|n| reference(&key_of(n), "base", pack, name))
                })
                .transpose()?;
            if effects.insert(id, (list, skill)).is_some() {
                return Err(format!("gapfill: effects of `{name}` are listed twice"));
            }
        }
    }
    let mut pending: BTreeMap<usize, Vec<&str>> = BTreeMap::new();
    for p in &ga.pending {
        for name in &p.artifacts {
            pending
                .entry(id_of(name, "artifacts.pending")?)
                .or_default()
                .push(p.epic.as_str());
        }
    }

    // Artifacts.
    let mut uncovered = Vec::new();
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    for (id, row) in sod_rows.iter().enumerate() {
        if skipped.contains(&id) {
            continue;
        }
        let pack = pack_of[id];
        let what = format!("artifact `{}`", row.name);
        let mut list = Vec::new();
        let [a, d, p, k] = primary[id];
        if [a, d, p, k] != [0; 4] {
            let values: Vec<String> = [
                ("attack", a),
                ("defense", d),
                ("power", p),
                ("knowledge", k),
            ]
            .iter()
            .filter(|(_, v)| *v != 0)
            .map(|(n, v)| format!("{n} = {v}"))
            .collect();
            list.push(format!(
                "{{ kind = \"primary_skills\", {} }}",
                values.join(", ")
            ));
        }
        let (extra, skill) = effects.remove(&id).unwrap_or_default();
        list.extend(extra);
        let machine = machines.get(&id);
        let combo = parts.get(&id);
        let waits = pending.get(&id);
        if list.is_empty() && machine.is_none() && combo.is_none() && waits.is_none() {
            uncovered.push(row.name.clone());
        }
        *counts.entry(pack).or_default() += 1;
        let text = out.pack(pack);
        let mut comment = format!("original id {id}");
        if let Some(epics) = waits {
            let _ = write!(comment, "; pending: {}", epics.join(", "));
        }
        let _ = write!(
            text,
            "\n[[artifact]]\nid = \"{}\"  # {comment}\nname = {}\n",
            keys[id],
            quote(&row.name)
        );
        let _ = writeln!(text, "class = \"{}\"", row.class);
        if let Some(cost) = row.cost.filter(|c| *c > 0) {
            let _ = writeln!(
                text,
                "cost = {{ {} = {cost} }}",
                quote(&reference("gold", "base", pack, &what)?)
            );
        }
        if let Some(slot) = slot_of(row, &sod_pos)? {
            let _ = writeln!(
                text,
                "slot = \"{}\"",
                reference(slot, slot_pack(slot), pack, &what)?
            );
        }
        if let Some(m) = machine {
            let _ = writeln!(text, "war_machine = \"{m}\"");
        }
        if let Some(s) = skill {
            let _ = writeln!(text, "needs_skill = \"{s}\"");
        }
        if !list.is_empty() {
            let _ = writeln!(text, "effects = [{}]", list.join(", "));
        }
        if let Some(c) = combo {
            let quoted: Vec<String> = c.iter().map(|p| quote(p)).collect();
            let _ = writeln!(text, "parts = [{}]", quoted.join(", "));
        }
    }
    if let Some(name) = effects.keys().next().map(|&id| names[id]) {
        return Err(format!("gapfill: `{name}` is not harvested"));
    }
    if !uncovered.is_empty() {
        return Err(format!(
            "artifacts without a modelled effect and not pending: {}",
            uncovered.join(", ")
        ));
    }
    let per_pack: Vec<String> = counts.iter().map(|(p, n)| format!("{p} {n}")).collect();
    report.push(format!("artifacts per pack: {}", per_pack.join(", ")));
    for p in &ga.pending {
        report.push(format!(
            "artifacts pending {} ({}): {}",
            p.epic,
            p.what,
            p.artifacts.len()
        ));
    }
    report.push(
        "ARTRAITS: texts not harvested; combination costs are blank in the original table".into(),
    );
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slot_kinds_cover_every_position_once() {
        let positions: BTreeSet<&str> = SLOT_KINDS.iter().map(|(p, _, _)| *p).collect();
        assert_eq!(positions.len(), SLOT_KINDS.len());
        assert_eq!(kind_of("Left Ring").unwrap(), ("ring", "Ring"));
        assert!(kind_of("Tail").is_err());
    }
}
