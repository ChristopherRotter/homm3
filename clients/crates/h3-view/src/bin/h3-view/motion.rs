//! A record's animation numbers (E17 M2 T4a): each creature's [`Look`] from the asset tables and
//! its DEF (the frames of each group, the standing frame's top edge; a shooter's projectile, T4b-1),
//! the magic animations a record shows (E17 M3 T3b) and the [`Stage`]. An adapter: it reads the
//! archives.

use std::collections::BTreeMap;
use std::path::Path;

use h3_content::CreatureTypeId;
use h3_content_battle::Ability;
use h3_formats::lod::LodArchive;
use h3_protocol::wire::{Event, ServerMessage, Side};
use h3_view::anim::{
    ACID, ACID_BREATH, BAD_MORALE, BOLT, Caster, Casts, DEATH_STARE, DEATH_STARE_SPELL, FEAR,
    GOOD_LUCK, GOOD_MORALE, HEAL, LIFE_DRAIN, LIGHTNING_BOLT, Look, MagicLook, Missile, ObjectLook,
    RESISTED, Shell, SpellLook, Stage, THUNDERBOLT, TITAN_BOLT,
};
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
        let standing_frame = sprite.frame(standing, 0);
        let top = standing_frame.map_or(0, |f| f.top);
        let front =
            standing_frame.map_or(0, |f| f.left + i32::try_from(f.width).unwrap_or(i32::MAX));
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
                front,
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
            frames: groups(&def.def)?,
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
/// stack is healed ([`HEAL`]); since E17 M3 T7b-1 each cast spell's and the resisted one's
/// ([`RESISTED`]); since E17 M3 T7b-2b-2b good and bad morale's, Fear's and good luck's
/// ([`GOOD_MORALE`], [`BAD_MORALE`], [`FEAR`], [`GOOD_LUCK`]); since E17 M3 T7b-2b-3 each spell
/// after a strike's, the bolt from the sky ([`BOLT`], the Thunderbird's [`THUNDERBOLT`] after it),
/// Life Drain's, Death Stare's and the acid's ([`LIFE_DRAIN`], [`DEATH_STARE`], [`ACID`]); since
/// E17 M3 T7c-2 each spell object's placing and vanishing ones; each with its DEF's frames and
/// size.
pub fn magic(
    record: &Record,
    tables: &Tables,
    archives: &[(&str, LodArchive<'_>)],
) -> Result<BTreeMap<u8, MagicLook>, String> {
    let mut numbers = std::collections::BTreeSet::new();
    // The objects a spell places, the snapshot's too (E17 M3 T7c-2): placed and gone.
    let placed = record
        .first_snapshot()
        .into_iter()
        .flat_map(|s| s.spell_obstacles.iter().map(|o| o.spell))
        .chain(events(record).filter_map(|e| match *e {
            Event::ObstaclePlaced { spell, .. }
            | Event::ObstacleRemoved { spell, .. }
            | Event::ObstacleRevealed { spell, .. } => Some(spell),
            _ => None,
        }));
    for spell in placed {
        for o in spell_entry(record, tables, spell).map_or(&[][..], |e| &e.objects) {
            numbers.insert(o.placed);
            numbers.insert(o.vanish);
        }
    }
    for event in events(record) {
        match event {
            Event::Healed { .. } => {
                numbers.insert(HEAL);
            }
            Event::SpellResisted { .. } => {
                numbers.insert(RESISTED);
            }
            Event::GoodMorale { .. } => {
                numbers.insert(GOOD_MORALE);
            }
            Event::BadMorale { .. } => {
                numbers.insert(BAD_MORALE);
            }
            Event::Frightened { .. } => {
                numbers.insert(FEAR);
            }
            Event::Struck { flags, .. } if flags.lucky => {
                numbers.insert(GOOD_LUCK);
            }
            Event::SpellCast { spell, .. }
            | Event::StrikeSpell { spell, .. }
            | Event::StackCast { spell, .. } => {
                let entry = spell_entry(record, tables, *spell);
                if let Some(a) = entry.and_then(|e| e.animation) {
                    numbers.insert(a);
                }
                // The bolt from the sky first (E17 M3 T7b-2b-3), the Thunderbird's own after it.
                if entry.is_some_and(|e| matches!(e.original, LIGHTNING_BOLT | TITAN_BOLT)) {
                    numbers.insert(BOLT);
                    if matches!(event, Event::StrikeSpell { .. }) {
                        numbers.insert(THUNDERBOLT);
                    }
                }
            }
            // The striker's effects (E17 M3 T7b-2b-3).
            Event::LifeDrained { .. } => {
                numbers.insert(LIFE_DRAIN);
            }
            Event::DeathStared { .. } => {
                numbers.insert(DEATH_STARE);
            }
            Event::StrikeDamage { .. } => {
                numbers.insert(ACID);
            }
            _ => {}
        }
    }
    let mut looks = BTreeMap::new();
    for number in numbers {
        let entry = tables
            .battle
            .magic
            .get(usize::from(number))
            .ok_or_else(|| format!("the asset tables have no magic animation {number}"))?;
        let (size, frames) = def_numbers(&entry.def, archives)?;
        looks.insert(
            number,
            MagicLook {
                def: entry.def.clone(),
                frames,
                size,
                place: entry.place,
                translucent: entry.translucent,
            },
        );
    }
    Ok(looks)
}

/// What the record's casts draw (E17 M3 T7b-1): each cast spell's animation and projectile DEFs,
/// each side's hero's cast point and casting frames; each spell the record's stacks hold, its
/// number and sign (E17 M3 T7b-2a: the box's tint, the walk time, the handler) and its effect at
/// each side's hero's mastery, from the ruleset the record's packs compile to (`packs`; the
/// mastery of the viewer's own hero, none for the other side's, which the record does not show:
/// *provisional*); with a stack frightened, the record's creatures that frighten (the ruleset's
/// `fear`, E17 M3 T7b-2b-2b: Fear's line names them).
pub fn casts(
    record: &Record,
    tables: &Tables,
    archives: &[(&str, LodArchive<'_>)],
    packs: &Path,
) -> Result<Casts, String> {
    let groups = tables.battle.hero_groups;
    let mut casts = Casts {
        groups: [groups.cast, groups.standing],
        ..Casts::default()
    };
    let snapshot = record
        .first_snapshot()
        .ok_or("the record holds no snapshot")?;
    let mut cast = false;
    let mut frightened = false;
    let mut spells: Vec<u16> = snapshot
        .stacks
        .iter()
        .flat_map(|s| s.effects.iter().map(|e| e.spell))
        .collect();
    // A spell by its number in the executable, for the striker's effects that play one's sound
    // or name it (E17 M3 T7b-2b-3: Death Stare, the Rust Dragon's acid).
    let numbered = |original: u16| {
        let count = u16::try_from(record.header.manifest.keys.spells.len()).unwrap_or(u16::MAX);
        (0..count).find(|&h| spell_entry(record, tables, h).is_some_and(|e| e.original == original))
    };
    // The snapshot's battlefield objects' spells (E17 M3 T7c-2).
    spells.extend(snapshot.spell_obstacles.iter().map(|o| o.spell));
    for event in events(record) {
        match *event {
            Event::SpellCast { spell, .. } => {
                cast = true;
                spells.push(spell);
            }
            // The objects a spell places (E17 M3 T7c-2).
            Event::ObstaclePlaced { spell, .. }
            | Event::ObstacleRemoved { spell, .. }
            | Event::ObstacleRevealed { spell, .. } => spells.push(spell),
            // A creature's cast (E17 M3 T7c-1) draws its spell as a hero's does.
            Event::EffectApplied { spell, .. }
            | Event::StrikeSpell { spell, .. }
            | Event::StackCast { spell, .. } => {
                spells.push(spell);
            }
            Event::Frightened { .. } => frightened = true,
            Event::DeathStared { .. } => spells.extend(numbered(DEATH_STARE_SPELL)),
            Event::StrikeDamage { .. } => spells.extend(numbered(ACID_BREATH)),
            _ => {}
        }
    }
    spells.sort_unstable();
    spells.dedup();
    let manifest = &record.header.manifest;
    let ruleset = if cast || frightened {
        Some(crate::scenario::ruleset(packs, manifest)?)
    } else {
        None
    };
    if let Some(r) = ruleset.as_ref().filter(|_| frightened) {
        casts.frighteners = snapshot
            .stacks
            .iter()
            .map(|s| s.creature)
            .filter(|&c| {
                r.abilities(CreatureTypeId::from_raw(c))
                    .iter()
                    .any(|a| matches!(a, Ability::Fear { .. }))
            })
            .collect();
    }
    let viewer = usize::from(snapshot.viewer == Side::Defender);
    for spell in spells {
        let Some(entry) = spell_entry(record, tables, spell) else {
            continue;
        };
        let mut projectile = Vec::new();
        for (def, &angle) in entry.projectile.iter().zip(&entry.projectile_angles) {
            let (size, frames) = def_numbers(def, archives)?;
            projectile.push(Shell {
                def: def.clone(),
                size,
                frames,
                angle,
            });
        }
        let effect = ruleset
            .as_ref()
            .and_then(|r| r.spells().get(usize::from(spell)))
            .map_or([0; 4], |s| s.effect);
        let mut value = [effect[0]; 2];
        if let Some(own) = &snapshot.own_hero {
            let mastery = own
                .spell_book
                .iter()
                .find(|b| b.spell == spell)
                .map_or(0, |b| usize::from(b.mastery.min(3)));
            value[viewer] = effect[mastery];
        }
        let mut objects = Vec::new();
        for o in &entry.objects {
            let (_, frames) = def_numbers(&o.def, archives)?;
            objects.push(ObjectLook {
                def: o.def.clone(),
                frames,
                hexes: o.hexes,
                rows: o.rows,
                before_stacks: o.before_stacks,
                placed: o.placed,
                vanish: o.vanish,
            });
        }
        let look = SpellLook {
            original: entry.original,
            sign: entry.sign,
            value,
            animation: entry.animation,
            projectile,
            objects,
            object_sound: entry.object_sound.is_some(),
        };
        casts.spells.insert(spell, look);
    }
    let keys = &manifest.keys.hero_types;
    for (side, view) in snapshot.sides.iter().enumerate().filter(|_| cast) {
        let Some(hero) = &view.hero else {
            continue;
        };
        let key = keys
            .get(usize::from(hero.hero_type))
            .ok_or_else(|| format!("no hero type handle {}", hero.hero_type))?;
        let art = tables
            .heroes
            .get(key)
            .ok_or_else(|| format!("the asset tables have no entry for `{key}`"))?;
        let mut batch = Batch::default();
        batch
            .add(&art.def, &lookup(archives, &art.def)?)
            .map_err(|e| e.to_string())?;
        let sprite = batch
            .sprite(&art.def)
            .ok_or_else(|| format!("{}: not a DEF", art.def))?;
        let frames = sprite
            .def
            .groups
            .iter()
            .find(|g| g.id == u32::from(groups.cast))
            .map_or(0, |g| g.frames.len());
        let [x, y] = art.cast.map(i32::from);
        // 0x5A00FD to 0x5A01C2: the left hero's from its DEF's corner, the right one's mirrored
        // in its DEF's width.
        let width = i32::try_from(sprite.def.width).unwrap_or(0);
        let point = if side == 0 {
            [x - 43, y - 19]
        } else {
            [693 + width - x, y - 19]
        };
        casts.casters[side] = Some(Caster {
            point,
            before: u32::from(art.cast_frames),
            frames: u32::try_from(frames).unwrap_or(0),
        });
    }
    Ok(casts)
}

/// The record's events, in order.
fn events(record: &Record) -> impl Iterator<Item = &Event> {
    record.messages.iter().flat_map(|m| match m {
        ServerMessage::Events { events, .. } => events.as_slice(),
        _ => &[],
    })
}

/// The asset tables' entry of the spell with `handle` (none: a handle or key it does not know).
fn spell_entry<'t>(
    record: &Record,
    tables: &'t Tables,
    handle: u16,
) -> Option<&'t h3_view::tables::Spell> {
    let key = record
        .header
        .manifest
        .keys
        .spells
        .get(usize::from(handle))?;
    tables.spells.get(key)
}

/// A DEF's canvas (width, height) and the frames of its group 0.
fn def_numbers(def: &str, archives: &[(&str, LodArchive<'_>)]) -> Result<([i32; 2], u32), String> {
    let mut batch = Batch::default();
    batch
        .add(def, &lookup(archives, def)?)
        .map_err(|e| e.to_string())?;
    let sprite = batch
        .sprite(def)
        .ok_or_else(|| format!("{def}: not a DEF"))?;
    let frames = sprite.def.groups.first().map_or(0, |g| g.frames.len());
    let size = |v: u32| i32::try_from(v).unwrap_or(i32::MAX);
    Ok((
        [size(sprite.def.width), size(sprite.def.height)],
        u32::try_from(frames).unwrap_or(0),
    ))
}

/// The stage of the record's battle, with the magic animations `magic` ([`magic`]) and what its
/// casts draw ([`casts`]).
pub fn stage<'a>(
    record: &Record,
    tables: &Tables,
    layout: &'a BattleLayout,
    (magic, casts): (&'a BTreeMap<u8, MagicLook>, &'a Casts),
) -> Stage<'a> {
    Stage {
        layout,
        width: record.header.manifest.field.width,
        groups: tables.battle.groups,
        magic,
        casts,
    }
}
