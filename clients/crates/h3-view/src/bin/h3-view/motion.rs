//! A record's animation numbers (E17 M2 T4a): each creature's [`Look`] from the asset tables and
//! its DEF (the frames of each group, the standing frame's top edge), and the [`Stage`]. An
//! adapter: it reads the archives.

use std::collections::BTreeMap;

use h3_formats::lod::LodArchive;
use h3_view::anim::{Look, Stage};
use h3_view::batch::Batch;
use h3_view::layout::BattleLayout;
use h3_view::record::Record;
use h3_view::tables::Tables;

use crate::render::lookup;

/// The [`Look`] of every creature a living stack of the record's first snapshot shows, by its
/// handle.
pub fn looks(
    record: &Record,
    tables: &Tables,
    archives: &[(&str, LodArchive<'_>)],
) -> Result<BTreeMap<u16, Look>, String> {
    let snapshot = record
        .first_snapshot()
        .ok_or("the record holds no snapshot")?;
    let keys = &record.header.manifest.keys.creatures;
    let standing = u32::from(tables.battle.groups.standing);
    let mut looks = BTreeMap::new();
    for stack in snapshot.stacks.iter().filter(|s| s.alive) {
        if looks.contains_key(&stack.creature) {
            continue;
        }
        let key = keys
            .get(usize::from(stack.creature))
            .ok_or_else(|| format!("no creature handle {}", stack.creature))?;
        let entry = tables
            .creatures
            .get(key)
            .ok_or_else(|| format!("the asset tables have no entry for `{key}`"))?;
        let mut batch = Batch::default();
        batch
            .add(&entry.def, &lookup(archives, &entry.def)?)
            .map_err(|e| e.to_string())?;
        let sprite = batch
            .sprite(&entry.def)
            .ok_or_else(|| format!("{}: not a DEF", entry.def))?;
        let frames = sprite
            .def
            .groups
            .iter()
            .map(|g| (g.id, u32::try_from(g.frames.len()).unwrap_or(0)))
            .collect();
        let top = sprite.frame(standing, 0).map_or(0, |f| f.top);
        looks.insert(
            stack.creature,
            Look {
                frames,
                times: entry.times,
                climax: entry.attack_climax,
                missile_offsets: entry.missile_offsets,
                top,
            },
        );
    }
    Ok(looks)
}

/// The stage of the record's battle.
pub fn stage<'a>(record: &Record, tables: &Tables, layout: &'a BattleLayout) -> Stage<'a> {
    Stage {
        layout,
        width: record.header.manifest.field.width,
        groups: tables.battle.groups,
    }
}
