//! A record's animation numbers (E17 M2 T4a): each creature's [`Look`] from the asset tables and
//! its DEF (the frames of each group, the standing frame's top edge; a shooter's projectile, T4b-1),
//! the magic animations a record shows (E17 M3 T3b) and the [`Stage`]. An adapter: it reads the
//! archives.

use std::collections::BTreeMap;
use std::path::Path;

use h3_content::{CreatureFlag, CreatureTypeId};
use h3_content_battle::Ability;
use h3_formats::lod::LodArchive;
use h3_protocol::wire::{Event, ServerMessage, Side};
use h3_render::Bugs;
use h3_view::anim::{
    ACID, ACID_BREATH, BAD_MORALE, BOLT, Body, Burst, CHAIN_LIGHTNING, CHAIN_START, Caster, Casts,
    DEATH_STARE, DEATH_STARE_SPELL, Explosion, FEAR, FIRE_SHIELD, GOOD_LUCK, GOOD_MORALE, HEAL,
    LAND_MINE, LIFE_DRAIN, LIGHTNING_BOLT, Look, MINE_BLAST, MagicLook, Missile, ObjectLook, RAISE,
    RESISTED, RESURRECTION, SACRIFICED, SHIELD_RETURN, Shell, SiegeStage, SpellLook, Stage,
    THUNDERBOLT, TITAN_BOLT,
};
use h3_view::batch::Batch;
use h3_view::hero::{HeroLook, Heroes};
use h3_view::layout::BattleLayout;
use h3_view::record::Record;
use h3_view::tables::{Creature, Tables, Town};

use crate::render::lookup;

/// The [`Look`] of every creature a living stack of the record's first snapshot shows, by its
/// handle; a siege's turrets (off the field, E17 M4 T2b) by the town's shooter's, its missile the
/// turret's.
pub fn looks(
    record: &Record,
    tables: &Tables,
    archives: &[(&str, LodArchive<'_>)],
) -> Result<BTreeMap<u16, Look>, String> {
    let snapshot = record
        .first_snapshot()
        .ok_or("the record holds no snapshot")?;
    let keys = &record.header.manifest.keys.creatures;
    let mut looks = BTreeMap::new();
    let field = record.header.manifest.field;
    let cells = u16::from(field.width) * u16::from(field.height);
    for stack in snapshot.stacks.iter().filter(|s| s.alive) {
        if looks.contains_key(&stack.creature) {
            continue;
        }
        let (entry, missile) = if u16::from(stack.hex) < cells {
            let key = keys
                .get(usize::from(stack.creature))
                .ok_or_else(|| format!("no creature handle {}", stack.creature))?;
            let entry = tables
                .creatures
                .get(key)
                .ok_or_else(|| format!("the asset tables have no entry for `{key}`"))?;
            (entry, entry.missile.clone())
        } else {
            let town = town(record, tables)?;
            (shooter(town, tables)?, Some(town.turrets.missile.clone()))
        };
        looks.insert(
            stack.creature,
            look(record, tables, archives, (entry, missile))?,
        );
    }
    // The creatures the battle creates (E17 M5 T5a): a summoned stack's, a Pit Lord's Demons.
    let made: Vec<u16> = events(record)
        .filter_map(|e| match *e {
            Event::StackCreated { creature, .. } | Event::RoseFromCorpse { creature, .. } => {
                Some(creature)
            }
            _ => None,
        })
        .collect();
    add_looks(record, tables, archives, &made, &mut looks)?;
    Ok(looks)
}

/// The looks of the creatures `made` that `looks` lacks, added (E17 M5 T5a: created stacks; a
/// battle fought as it is played adds every creature it may create, [`casts`]' bodies).
pub fn add_looks(
    record: &Record,
    tables: &Tables,
    archives: &[(&str, LodArchive<'_>)],
    made: &[u16],
    looks: &mut BTreeMap<u16, Look>,
) -> Result<(), String> {
    let keys = &record.header.manifest.keys.creatures;
    for &creature in made {
        if looks.contains_key(&creature) {
            continue;
        }
        let key = keys
            .get(usize::from(creature))
            .ok_or_else(|| format!("no creature handle {creature}"))?;
        let entry = tables
            .creatures
            .get(key)
            .ok_or_else(|| format!("the asset tables have no entry for `{key}`"))?;
        let missile = entry.missile.clone();
        looks.insert(creature, look(record, tables, archives, (entry, missile))?);
    }
    Ok(())
}

/// The creature the town's turrets show (its shooter by its original id): their look and their
/// shot's sound (E17 M4 T3a, R2-1).
pub fn shooter<'t>(town: &Town, tables: &'t Tables) -> Result<&'t Creature, String> {
    tables
        .creatures
        .values()
        .find(|c| c.original == town.turrets.shooter)
        .ok_or_else(|| format!("creature {}", town.turrets.shooter))
}

/// The town of a siege the record's battle is fought at.
pub fn town<'t>(record: &Record, tables: &'t Tables) -> Result<&'t Town, String> {
    let siege = record
        .first_snapshot()
        .and_then(|s| s.siege.as_ref())
        .ok_or("a turret outside a siege")?;
    let key = record
        .header
        .manifest
        .keys
        .factions
        .get(usize::from(siege.faction))
        .ok_or_else(|| format!("no faction handle {}", siege.faction))?;
    tables
        .towns
        .get(key)
        .ok_or_else(|| format!("the asset tables have no town `{key}`"))
}

/// A creature's [`Look`]: its DEF's groups, its standing frame's edges, its times and climax,
/// its projectile `missile` and its explosion.
fn look(
    record: &Record,
    tables: &Tables,
    archives: &[(&str, LodArchive<'_>)],
    (entry, missile): (&Creature, Option<String>),
) -> Result<Look, String> {
    let standing = u32::from(tables.battle.groups.standing);
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
    let front = standing_frame.map_or(0, |f| f.left + i32::try_from(f.width).unwrap_or(i32::MAX));
    let missile = match &missile {
        Some(def) => Some(self::missile(def, &entry.missile_angles, archives)?),
        None => None,
    };
    // The explosion its projectile ends in (E17 M3 T7c-3): its spell's handle, animation.
    let explosion = entry.explosion.and_then(|e| {
        let (spell, s) = spell_numbered(record, tables, e.spell)?;
        Some(Explosion {
            spell,
            animation: s.animation?,
            time: e.time,
        })
    });
    Ok(Look {
        frames,
        times: entry.times,
        climax: entry.attack_climax,
        missile_offsets: entry.missile_offsets,
        top,
        front,
        missile,
        ray: entry.ray.clone(),
        explosion,
    })
}

/// What a siege's shots draw (E17 M4 T2b): the town's turret points and the wall shots'
/// explosions; none outside a siege with a fort.
pub fn siege(
    record: &Record,
    tables: &Tables,
    archives: &[(&str, LodArchive<'_>)],
) -> Result<Option<SiegeStage>, String> {
    let fort = record
        .first_snapshot()
        .and_then(|s| s.siege.as_ref())
        .is_some_and(|s| s.fort > 0);
    if !fort {
        return Ok(None);
    }
    let town = town(record, tables)?;
    let point = |p: [i16; 2]| [i32::from(p[0]), i32::from(p[1])];
    let burst = |def: Option<&String>| -> Result<Burst, String> {
        let Some(def) = def else {
            return Ok(Burst::default());
        };
        let (size, frames) = def_numbers(def, archives)?;
        Ok(Burst {
            def: def.clone(),
            size,
            frames,
        })
    };
    let explosions = tables.battle.wall_explosions.as_ref();
    Ok(Some(SiegeStage {
        turrets: [
            point(town.turrets.keep),
            point(town.turrets.lower),
            point(town.turrets.upper),
        ],
        hit: burst(explosions.map(|e| &e[0]))?,
        miss: burst(explosions.map(|e| &e[1]))?,
    }))
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
        frames: u32::try_from(frames).unwrap_or(0),
        angles: angles[..frames.min(angles.len())].to_vec(),
    })
}

/// The magic animations the record's events call for, by number (E17 M3 T3b): the heal's when a
/// stack is healed ([`HEAL`]); since E17 M3 T7b-1 each cast spell's and the resisted one's
/// ([`RESISTED`]); since E17 M3 T7b-2b-2b good and bad morale's, Fear's and good luck's
/// ([`GOOD_MORALE`], [`BAD_MORALE`], [`FEAR`], [`GOOD_LUCK`]); since E17 M3 T7b-2b-3 each spell
/// after a strike's, the bolt from the sky ([`BOLT`], the Thunderbird's [`THUNDERBOLT`] after it),
/// Life Drain's, Death Stare's and the acid's ([`LIFE_DRAIN`], [`DEATH_STARE`], [`ACID`]); since
/// E17 M3 T7c-2 each spell object's placing and vanishing ones; since E17 M4 T2c-1 a Land Mine's
/// blast ([`MINE_BLAST`]); each with its DEF's frames and size. With `every` (E17 M5 T1b: a
/// battle fought as it is played, whose events are not known yet), every magic animation of the
/// asset tables.
pub fn magic(
    record: &Record,
    tables: &Tables,
    archives: &[(&str, LodArchive<'_>)],
    every: bool,
) -> Result<BTreeMap<u8, MagicLook>, String> {
    let mut numbers = std::collections::BTreeSet::new();
    if every {
        let all = u8::try_from(tables.battle.magic.len()).unwrap_or(u8::MAX);
        numbers.extend(0..all);
    }
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
    // The explosions the shooters' projectiles end in (E17 M3 T7c-3).
    let keys = &record.header.manifest.keys.creatures;
    for stack in record.first_snapshot().into_iter().flat_map(|s| &s.stacks) {
        let explosion = keys
            .get(usize::from(stack.creature))
            .and_then(|key| tables.creatures.get(key)?.explosion);
        let spell = explosion.and_then(|e| spell_numbered(record, tables, e.spell));
        numbers.extend(spell.and_then(|(_, s)| s.animation));
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
                // Chain Lightning's opening first (E17 M3 T7c-3).
                if entry.is_some_and(|e| e.original == CHAIN_LIGHTNING) {
                    numbers.insert(CHAIN_START);
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
            // The raises, Fire Shield's return, Sacrifice's victim (E17 M3 T7c-3).
            Event::SpellRaised { .. } | Event::Reborn { .. } => {
                numbers.insert(RAISE);
            }
            Event::Reflected { .. } => {
                numbers.insert(SHIELD_RETURN);
            }
            Event::Sacrificed { .. } => {
                numbers.insert(SACRIFICED);
            }
            Event::DeathStared { .. } => {
                numbers.insert(DEATH_STARE);
            }
            Event::StrikeDamage { .. } => {
                numbers.insert(ACID);
            }
            // A Land Mine's blast over the stack it hurts (E17 M4 T2c-1).
            Event::ObstacleTriggered { spell, .. }
                if spell_entry(record, tables, *spell).is_some_and(|e| e.original == LAND_MINE) =>
            {
                numbers.insert(MINE_BLAST);
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
/// `fear`, E17 M3 T7b-2b-2b: Fear's line names them). With `every` (E17 M5 T1b: a battle fought
/// as it is played), every spell's, as if each were cast and a stack frightened and a shield
/// returned.
pub fn casts(
    record: &Record,
    tables: &Tables,
    archives: &[(&str, LodArchive<'_>)],
    packs: &Path,
    every: bool,
) -> Result<Casts, String> {
    let groups = tables.battle.hero_groups;
    let mut casts = Casts {
        groups: [groups.cast, groups.standing],
        ..Casts::default()
    };
    let snapshot = record
        .first_snapshot()
        .ok_or("the record holds no snapshot")?;
    let (mut cast, mut frightened, mut reflected) = (every, every, every);
    // The creatures the battle creates (E17 M5 T5a): their bodies from the ruleset.
    let mut made: Vec<u16> = Vec::new();
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
            // Fire Shield's return names it and plays its sound; an Archangel's raise and a
            // Phoenix's rebirth play Resurrection's (E17 M3 T7c-3).
            Event::Reflected { .. } => {
                reflected = true;
                spells.extend(numbered(FIRE_SHIELD));
            }
            Event::CreatureRaise { .. } | Event::Reborn { .. } => {
                spells.extend(numbered(RESURRECTION));
            }
            Event::DeathStared { .. } => spells.extend(numbered(DEATH_STARE_SPELL)),
            Event::StackCreated { creature, .. } | Event::RoseFromCorpse { creature, .. } => {
                made.push(creature);
            }
            Event::StrikeDamage { .. } => spells.extend(numbered(ACID_BREATH)),
            _ => {}
        }
    }
    if every {
        let count = u16::try_from(record.header.manifest.keys.spells.len()).unwrap_or(u16::MAX);
        spells.extend(0..count);
    }
    spells.sort_unstable();
    spells.dedup();
    let manifest = &record.header.manifest;
    let ruleset = if cast || frightened || reflected || !made.is_empty() {
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
    // The creatures with a Fire Shield of their own (E17 M3 T7c-3: the ruleset's innate spell).
    if let Some(r) = ruleset.as_ref().filter(|_| reflected) {
        let shield = numbered(FIRE_SHIELD);
        casts.shielded = snapshot
            .stacks
            .iter()
            .map(|s| s.creature)
            .filter(|&c| {
                r.abilities(CreatureTypeId::from_raw(c)).iter().any(|a| {
                    matches!(a, Ability::InnateSpell { spell, .. } if Some(spell.raw()) == shield)
                })
            })
            .collect();
    }
    // Every creature a spell summons or a corpse raiser raises, for a battle fought as it is played.
    if let Some(r) = ruleset.as_ref().filter(|_| every) {
        made.extend(
            r.spells()
                .iter()
                .filter_map(|s| s.summons.map(CreatureTypeId::raw)),
        );
        let count = u16::try_from(manifest.keys.creatures.len()).unwrap_or(u16::MAX);
        for c in 0..count {
            for a in r.abilities(CreatureTypeId::from_raw(c)) {
                if let Ability::CorpseRaiser { creature, .. } = a {
                    made.push(creature.raw());
                }
            }
        }
    }
    if let Some(r) = ruleset.as_ref() {
        for &c in &made {
            let Some(creature) = r.creature(CreatureTypeId::from_raw(c)) else {
                continue;
            };
            let has = |flag| creature.flags.contains(flag);
            casts.bodies.insert(
                c,
                Body {
                    two_hex: has(CreatureFlag::TwoHex),
                    flies: has(CreatureFlag::Flying),
                    shooter: has(CreatureFlag::Shooter),
                },
            );
        }
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
    spell_entry_in(&record.header.manifest.keys.spells, tables, handle)
}

/// As [`spell_entry`] by the spells' `keys` (E17 M5 T3b).
pub fn spell_entry_in<'t>(
    keys: &[String],
    tables: &'t Tables,
    handle: u16,
) -> Option<&'t h3_view::tables::Spell> {
    tables.spells.get(keys.get(usize::from(handle))?)
}

/// The record's spell numbered `original` in the executable: its handle and its entry.
fn spell_numbered<'t>(
    record: &Record,
    tables: &'t Tables,
    original: u16,
) -> Option<(u16, &'t h3_view::tables::Spell)> {
    spell_numbered_in(&record.header.manifest.keys.spells, tables, original)
}

/// As [`spell_numbered`] by the spells' `keys` (E17 M5 T3b).
pub fn spell_numbered_in<'t>(
    keys: &[String],
    tables: &'t Tables,
    original: u16,
) -> Option<(u16, &'t h3_view::tables::Spell)> {
    let count = u16::try_from(keys.len()).unwrap_or(u16::MAX);
    (0..count).find_map(|h| {
        let entry = spell_entry_in(keys, tables, h)?;
        (entry.original == original).then_some((h, entry))
    })
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
/// casts draw ([`casts`]); the original's drawing bugs fixed (the comparisons with its captures
/// reproduce them: `sequences`).
pub fn stage<'a>(
    record: &Record,
    tables: &Tables,
    layout: &'a BattleLayout,
    (magic, casts, siege): (
        &'a BTreeMap<u8, MagicLook>,
        &'a Casts,
        Option<&'a SiegeStage>,
    ),
) -> Stage<'a> {
    Stage {
        layout,
        width: record.header.manifest.field.width,
        groups: tables.battle.groups,
        magic,
        casts,
        bugs: Bugs::Fixed,
        siege,
        lengths: None,
    }
}
