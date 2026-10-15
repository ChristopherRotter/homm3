//! A record's animation numbers (E17 M2 T4a): each creature's [`Look`] from the asset tables and
//! its DEF (the frames of each group, the standing frame's top edge; a shooter's projectile, T4b-1),
//! the magic animations a record shows (E17 M3 T3b) and the [`Stage`]. An adapter: it reads the
//! archives.

use std::collections::BTreeMap;

use h3_formats::lod::LodArchive;
use h3_protocol::wire::{Event, ServerMessage};
use h3_view::anim::{HEAL, Look, MagicLook, Missile, Stage};
use h3_view::batch::Batch;
use h3_view::hero::{HeroLook, Heroes};
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
        let missile = match &entry.missile {
            Some(def) => Some(missile(def, &entry.missile_angles, archives)?),
            None => None,
        };
        looks.insert(
            stack.creature,
            Look {
                frames,
                times: entry.times,
                climax: entry.attack_climax,
                missile_offsets: entry.missile_offsets,
                top,
                missile,
                ray: entry.ray.clone(),
            },
        );
    }
    Ok(looks)
}

/// The heroes of the record's battle (E17 M3 T3a): each side's hero's [`HeroLook`] (its DEF's
/// groups, its flag's frames), whether each side's player is human, and `seed` for the viewer's
/// own defeats and victories (none: none of its own, as the reference checks want).
pub fn heroes(
    record: &Record,
    tables: &Tables,
    archives: &[(&str, LodArchive<'_>)],
    seed: Option<u32>,
) -> Result<Heroes, String> {
    let snapshot = record
        .first_snapshot()
        .ok_or("the record holds no snapshot")?;
    let keys = &record.header.manifest.keys.hero_types;
    let groups = |def: &str| -> Result<BTreeMap<u32, u32>, String> {
        let mut batch = Batch::default();
        batch
            .add(def, &lookup(archives, def)?)
            .map_err(|e| e.to_string())?;
        let sprite = batch
            .sprite(def)
            .ok_or_else(|| format!("{def}: not a DEF"))?;
        Ok(sprite
            .def
            .groups
            .iter()
            .map(|g| (g.id, u32::try_from(g.frames.len()).unwrap_or(0)))
            .collect())
    };
    let mut looks = [None, None];
    for (side, view) in snapshot.sides.iter().enumerate() {
        let Some(hero) = &view.hero else {
            continue;
        };
        let key = keys
            .get(usize::from(hero.hero_type))
            .ok_or_else(|| format!("no hero type handle {}", hero.hero_type))?;
        let def = tables
            .heroes
            .get(key)
            .ok_or_else(|| format!("the asset tables have no entry for `{key}`"))?;
        let flag = groups(&tables.battle.flags[side])?;
        looks[side] = Some(HeroLook {
            frames: groups(def)?,
            flag_frames: flag.get(&0).copied().unwrap_or(0),
        });
    }
    let human = [snapshot.sides[0].human, snapshot.sides[1].human];
    Ok(Heroes::new(looks, human, tables.battle.hero_groups, seed))
}

/// A projectile: its DEF's canvas and the creature's angle for each frame of its group 0 (the
/// table holds 12; a DEF with fewer frames uses the first ones, 0x469957).
fn missile(
    def: &str,
    angles: &[i16; 12],
    archives: &[(&str, LodArchive<'_>)],
) -> Result<Missile, String> {
    let mut batch = Batch::default();
    batch
        .add(def, &lookup(archives, def)?)
        .map_err(|e| e.to_string())?;
    let sprite = batch
        .sprite(def)
        .ok_or_else(|| format!("{def}: not a DEF"))?;
    let frames = sprite.def.groups.first().map_or(0, |g| g.frames.len());
    let size = |v: u32| i32::try_from(v).unwrap_or(i32::MAX);
    Ok(Missile {
        def: def.to_owned(),
        size: [size(sprite.def.width), size(sprite.def.height)],
        angles: angles[..frames.min(angles.len())].to_vec(),
    })
}

/// The magic animations the record's events call for, by number (E17 M3 T3b): the heal's when a
/// stack is healed ([`HEAL`]); each with its DEF's frames and size.
pub fn magic(
    record: &Record,
    tables: &Tables,
    archives: &[(&str, LodArchive<'_>)],
) -> Result<BTreeMap<u8, MagicLook>, String> {
    let healed = record.messages.iter().any(|m| match m {
        ServerMessage::Events { events, .. } => {
            events.iter().any(|e| matches!(e, Event::Healed { .. }))
        }
        _ => false,
    });
    let mut looks = BTreeMap::new();
    for number in [HEAL].into_iter().filter(|_| healed) {
        let entry = tables
            .battle
            .magic
            .get(usize::from(number))
            .ok_or_else(|| format!("the asset tables have no magic animation {number}"))?;
        let mut batch = Batch::default();
        batch
            .add(&entry.def, &lookup(archives, &entry.def)?)
            .map_err(|e| e.to_string())?;
        let sprite = batch
            .sprite(&entry.def)
            .ok_or_else(|| format!("{}: not a DEF", entry.def))?;
        let frames = sprite.def.groups.first().map_or(0, |g| g.frames.len());
        let size = |v: u32| i32::try_from(v).unwrap_or(i32::MAX);
        looks.insert(
            number,
            MagicLook {
                def: entry.def.clone(),
                frames: u32::try_from(frames).unwrap_or(0),
                size: [size(sprite.def.width), size(sprite.def.height)],
                place: entry.place,
                translucent: entry.translucent,
            },
        );
    }
    Ok(looks)
}

/// The stage of the record's battle, with the magic animations `magic` ([`magic`]).
pub fn stage<'a>(
    record: &Record,
    tables: &Tables,
    layout: &'a BattleLayout,
    magic: &'a BTreeMap<u8, MagicLook>,
) -> Stage<'a> {
    Stage {
        layout,
        width: record.header.manifest.field.width,
        groups: tables.battle.groups,
        magic,
    }
}
