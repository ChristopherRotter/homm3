//! The map's sound (E24 M2 T5, *provisional*; `docs/epics/E24-design.md` §3.11): which sounds the
//! original plays on the adventure map and when, as cues for the window to play.
//!
//! Three kinds. **The terrain music**: the track of the terrain at the view's middle, asked for
//! when the map's screen starts, when `hero_select` centres the view on the selected hero and at a
//! step's end, each only when that terrain differs from the one asked for last (the manager's
//! +0x58); the window fades the old track out and the new one in (`music_stop_fade` 0x59B080,
//! `music_thread` 0x59A840). **The step sounds**: one looping sample from the path's walk's start
//! to its end, changed only where the music's terrain changes, at the lifted hero's frame 0
//! (`hero_move_step` 0x481344, pass 9's 0x47FF88). **The ambient sounds**: four slots of looping
//! samples, the nearest objects' within three tiles of the view's middle, each at a volume by its
//! distance (`ambient_sounds_at` 0x4183D0, `ambient_sound_visit` 0x418C10).
//!
//! [`SoundArt`] is the asset tables' `[map.sound]`, [`SoundRules`] the tables resolved,
//! [`TileSounds`] each tile's ambient sound and terrain (`tile_ambient_sound` 0x418620 on the
//! object a tile records, 0x505610's walks), [`Sound`] the original's state, driven by the
//! timeline's loop at the moments above and giving its [`Cue`]s. The effects' and the music's
//! settings are taken as on (nothing plays at 0 in the original). The client keeps a bit for each
//! ambient sound found, the comparisons the original's 32 (MCL-Q8, [`Bugs`]); the client's shore
//! is the coast's water, the comparisons' the original's one picture more (MCL-Q9).

use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;

use h3_map::model::Pos;
use h3_render::Bugs;
use serde::Deserialize;

use crate::art::{MapArt, TileArt, WATER};
use crate::live::Live;
use crate::objects::{DrawLists, Entry, template};

/// The ambient loops' slots (the manager's +0x228).
pub const SLOTS: usize = 4;
/// The rings the ambient scan visits around the view's middle: Chebyshev distance 1 to this.
pub const RINGS: i32 = 3;
/// A slot's distance when it is empty or its sound was not found (0x7F).
pub const FAR: u8 = 127;
/// A slot whose distance is above this after a scan is stopped (0x4185A9).
pub const KEPT: u8 = 5;
/// A hero's class: its tile sounds as nothing while it stands there.
pub const HERO_CLASS: u32 = 34;
/// The dwellings' class: their sound is their creature's.
pub const DWELLING: u32 = 17;
/// The mines' class.
pub const MINE: u32 = 53;
/// AB's abandoned mine as the map names it (the game loads it as a mine).
pub const AB_MINE: u32 = 220;
/// RoE's abandoned mine's subclass.
pub const ABANDONED: u32 = 7;
/// The class a tile without an object keeps (0x505610's reset), and on coastal land.
pub const NO_CLASS: u32 = 0;
/// The class coastal land keeps without an object.
pub const COAST_CLASS: u32 = 3;
/// The x of a packed tile that names none (0x3FF in 10 bits: −1).
pub const NO_TILE: i32 = -1;

/// The asset tables' `[map.sound]` (E24 M2 T5): the ambient loops by the original's index, their
/// volumes by distance and the rules of `tile_ambient_sound` 0x418620 by class; the flying step
/// sound. The terrains' music and step sounds are on their `[[terrain]]` entries. Each loop is
/// named by its WAV name, which `loops` gives an index.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SoundArt {
    /// The step sound of a hero flying (`horse10.wav`, the manager's +0x388).
    pub flying: String,
    /// The ambient loops by index (`ambient_loop_names` 0x65F794).
    pub loops: Vec<String>,
    /// The loops' volume by distance 0 to [`RINGS`] (`ambient_volumes` 0x63A64C).
    pub volumes: Vec<u8>,
    /// The shore's loop, before any object: the coast's water (the client's, MCL-Q9), or water of
    /// a picture below `shore_pictures` (the original's).
    pub shore: String,
    /// The first water picture that is not the shore's in the original's test (21, one more than
    /// the coast's [`crate::art::SHORE_PICTURES`]: MCL-Q9, the comparisons' only).
    pub shore_pictures: u8,
    /// The map's classes the game loads as others, `[map's, game's]` (`object_class_values`
    /// 0x63A6E4).
    pub loaded_as: Vec<[u32; 2]>,
    /// The ownable classes (`class_is_ownable` 0x40FCA0): a tile without an entrance records the
    /// topmost one blocking it first (0x505610).
    pub ownable: Vec<u32>,
    /// The class a tile without an entrance or a blocking object records when an object of it
    /// lies there (124, the hole).
    pub hole: u32,
    /// An entrance tile's loop by class, a subclass's entry for that subclass only; the first that
    /// matches.
    pub objects: Vec<ClassSound>,
    /// A dwelling's creature by subclass (`dwelling_creatures` 0x63D570).
    pub dwelling_creatures: Vec<u32>,
    /// A dwelling's loop by its creature.
    pub creatures: Vec<CreatureSound>,
    /// A dwelling's loop for a creature `creatures` lacks.
    pub creature_default: String,
    /// A mine's loop by its type (the subclass, 0 wood … 6 gold).
    pub mines: Vec<String>,
    /// An abandoned mine's loop.
    pub abandoned: String,
    /// A tile's loop without an entrance by the class it records (the volcano's).
    pub blocking: Vec<ClassSound>,
    /// The special grounds `tile_special_ground` 0x4FCE20 finds on a tile of no class, the first
    /// from the top.
    pub grounds: Vec<u32>,
    /// Their loops; a ground without one has none.
    pub ground_loops: Vec<ClassSound>,
}

/// A loop by class (and subclass).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClassSound {
    /// The class (the game's).
    pub class: u32,
    /// The subclass, if only that one.
    #[serde(default)]
    pub subclass: Option<u32>,
    /// The loop's name.
    #[serde(rename = "loop")]
    pub sound: String,
}

/// A loop by a dwelling's creatures.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreatureSound {
    /// The creatures.
    pub creatures: Vec<u32>,
    /// The loop's name.
    #[serde(rename = "loop")]
    pub sound: String,
}

/// The tables' sounds resolved: names by index and the rules by number.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SoundRules {
    /// The ambient loops' names by index.
    pub loops: Vec<String>,
    /// The music's track name by terrain code (none: the music goes on).
    pub music: BTreeMap<u8, String>,
    /// The step sound's name by terrain code.
    pub steps: BTreeMap<u8, String>,
    /// The flying hero's step sound.
    pub flying: String,
    volumes: Vec<u8>,
    shore: u8,
    shore_pictures: u8,
    loaded_as: BTreeMap<u32, u32>,
    ownable: Vec<u32>,
    hole: u32,
    objects: Vec<(u32, Option<u32>, u8)>,
    dwelling_creatures: Vec<u32>,
    creatures: BTreeMap<u32, u8>,
    creature_default: u8,
    mines: Vec<u8>,
    abandoned: u8,
    blocking: BTreeMap<u32, u8>,
    grounds: Vec<u32>,
    ground_loops: BTreeMap<u32, u8>,
}

impl SoundRules {
    /// The rules of `art` with the terrains' music and step sounds (by code).
    ///
    /// # Errors
    /// What breaks a rule: a WAV name without its extension, more than 256 loops or one named
    /// twice, a loop `loops` does not name, a volume for each distance 0 to [`RINGS`] missing.
    pub fn new(
        art: &SoundArt,
        music: BTreeMap<u8, String>,
        steps: BTreeMap<u8, String>,
    ) -> Result<Self, String> {
        for name in art.loops.iter().chain(steps.values()).chain([&art.flying]) {
            wav(name)?;
        }
        if art.loops.len() > usize::from(u8::MAX) + 1 {
            return Err(format!("{} loops, more than 256", art.loops.len()));
        }
        let index = |name: &str| -> Result<u8, String> {
            let found: Vec<usize> = art
                .loops
                .iter()
                .enumerate()
                .filter(|(_, l)| l.eq_ignore_ascii_case(name))
                .map(|(i, _)| i)
                .collect();
            match found[..] {
                // At most 256 loops (above): every index fits.
                [i] => Ok(u8::try_from(i).unwrap_or(u8::MAX)),
                [] => Err(format!("`{name}` is not one of the loops")),
                _ => Err(format!("`{name}` is named twice among the loops")),
            }
        };
        let rings = usize::try_from(RINGS).unwrap_or(0);
        if art.volumes.len() <= rings {
            return Err(format!(
                "{} volumes, not one for each distance 0 to {RINGS}",
                art.volumes.len()
            ));
        }
        let by_class = |list: &[ClassSound]| -> Result<BTreeMap<u32, u8>, String> {
            list.iter()
                .map(|c| Ok((c.class, index(&c.sound)?)))
                .collect()
        };
        let mut creatures = BTreeMap::new();
        for c in &art.creatures {
            let sound = index(&c.sound)?;
            for &creature in &c.creatures {
                creatures.insert(creature, sound);
            }
        }
        Ok(Self {
            loops: art.loops.clone(),
            music,
            steps,
            flying: art.flying.clone(),
            volumes: art.volumes.clone(),
            shore: index(&art.shore)?,
            shore_pictures: art.shore_pictures,
            loaded_as: art
                .loaded_as
                .iter()
                .map(|&[map, game]| (map, game))
                .collect(),
            ownable: art.ownable.clone(),
            hole: art.hole,
            objects: art
                .objects
                .iter()
                .map(|c| Ok((c.class, c.subclass, index(&c.sound)?)))
                .collect::<Result<_, String>>()?,
            dwelling_creatures: art.dwelling_creatures.clone(),
            creatures,
            creature_default: index(&art.creature_default)?,
            mines: art
                .mines
                .iter()
                .map(|m| index(m))
                .collect::<Result<_, _>>()?,
            abandoned: index(&art.abandoned)?,
            blocking: by_class(&art.blocking)?,
            grounds: art.grounds.clone(),
            ground_loops: by_class(&art.ground_loops)?,
        })
    }

    /// Every WAV the rules name (the loops, the step sounds), as named.
    #[must_use]
    pub fn wavs(&self) -> Vec<String> {
        let mut all: Vec<String> = self.loops.clone();
        all.extend(self.steps.values().cloned());
        all.push(self.flying.clone());
        all
    }

    /// The class the game loads a map's `class` as.
    #[must_use]
    pub fn game_class(&self, class: u32) -> u32 {
        self.loaded_as.get(&class).copied().unwrap_or(class)
    }

    /// A loop's volume at `distance` (0 at a distance the table lacks).
    #[must_use]
    pub fn volume(&self, distance: u8) -> u8 {
        self.volumes
            .get(usize::from(distance))
            .copied()
            .unwrap_or(0)
    }

    /// An entrance tile's loop: by its object's game class, subclass and map class (an abandoned
    /// mine).
    fn entrance(&self, class: u32, subclass: u32, map_class: u32) -> Option<u8> {
        match class {
            DWELLING => {
                let creature = usize::try_from(subclass)
                    .ok()
                    .and_then(|s| self.dwelling_creatures.get(s));
                Some(
                    creature
                        .and_then(|c| self.creatures.get(c))
                        .copied()
                        .unwrap_or(self.creature_default),
                )
            }
            MINE if map_class == AB_MINE || subclass == ABANDONED => Some(self.abandoned),
            MINE => usize::try_from(subclass)
                .ok()
                .and_then(|s| self.mines.get(s))
                .copied(),
            _ => self
                .objects
                .iter()
                .find(|(c, s, _)| *c == class && s.is_none_or(|s| s == subclass))
                .map(|&(_, _, sound)| sound),
        }
    }
}

/// A WAV name: something before `.wav`.
fn wav(name: &str) -> Result<(), String> {
    let upper = name.to_ascii_uppercase();
    if upper.len() <= 4 || !upper.ends_with(".WAV") {
        return Err(format!("`{name}` is not a .WAV name"));
    }
    Ok(())
}

/// Each tile's terrain and ambient sound, as the game holds them after the load: the sound of the
/// object the tile records (0x505610's walks of its draw list from the top), heroes standing
/// aside.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TileSounds {
    size: i32,
    levels: i32,
    terrains: Vec<u8>,
    sounds: Vec<Option<u8>>,
    /// The tiles where a hero stands (its class, 34, recorded there) but the selected hero's,
    /// which the timeline follows; sorted.
    heroes: Vec<[i32; 3]>,
}

impl TileSounds {
    /// The tiles of `art` with its live part, one draw list per level (`lists[z]`; a level
    /// without one sounds as its tiles' terrain alone), the hero `selected` left to the
    /// timeline; the shore as the client takes it or as the original does (`bugs`, MCL-Q9).
    #[must_use]
    pub fn build(
        art: &MapArt,
        live: &Live,
        lists: &[DrawLists],
        rules: &SoundRules,
        selected: Option<Pos>,
        bugs: Bugs,
    ) -> Self {
        let size = i32::try_from(art.size).unwrap_or(0);
        let levels = i32::try_from(art.levels).unwrap_or(0);
        let mut terrains = Vec::with_capacity(art.tiles.len());
        let mut sounds = Vec::with_capacity(art.tiles.len());
        // The tiles level by level, row by row (`MapArt::tile`'s order).
        let side = art.size.max(1);
        for (i, tile) in art.tiles.iter().enumerate() {
            let (x, y, z) = (i % side, i / side % side, i / (side * side));
            let entries = lists.get(z).map_or(&[][..], |l| {
                l.at(
                    i32::try_from(x).unwrap_or(-1),
                    i32::try_from(y).unwrap_or(-1),
                )
            });
            terrains.push(tile.terrain.code);
            sounds.push(tile_sound(art, live, rules, tile, entries, bugs));
        }
        let mut heroes: Vec<[i32; 3]> = live
            .heroes
            .iter()
            .filter(|h| Some(h.at) != selected)
            .map(|h| at(h.at))
            .collect();
        heroes.sort_unstable();
        Self {
            size,
            levels,
            terrains,
            sounds,
            heroes,
        }
    }

    /// The index of tile `[x, y, z]`, if it lies on the map.
    fn index(&self, [x, y, z]: [i32; 3]) -> Option<usize> {
        let on = |v: i32, n: i32| (0..n).contains(&v);
        if !(on(x, self.size) && on(y, self.size) && on(z, self.levels)) {
            return None;
        }
        usize::try_from((z * self.size + y) * self.size + x).ok()
    }

    /// The terrain of a tile on the map.
    #[must_use]
    pub fn terrain(&self, tile: [i32; 3]) -> Option<u8> {
        self.index(tile).and_then(|i| self.terrains.get(i)).copied()
    }

    /// The ambient sound of a tile on the map: none where a hero stands (`standing`, the selected
    /// hero on its tile, not lifted; the others always).
    #[must_use]
    pub fn sound(&self, tile: [i32; 3], standing: Option<Pos>) -> Option<u8> {
        if standing.is_some_and(|p| at(p) == tile) || self.heroes.binary_search(&tile).is_ok() {
            return None;
        }
        self.index(tile).and_then(|i| self.sounds.get(i)).copied()?
    }
}

/// A position as a tile's coordinates.
fn at(p: Pos) -> [i32; 3] {
    [i32::from(p.x), i32::from(p.y), i32::from(p.z)]
}

/// A tile's ambient sound (`tile_ambient_sound` 0x418620) from its draw list, bottom to top.
fn tile_sound(
    art: &MapArt,
    live: &Live,
    rules: &SoundRules,
    tile: &TileArt,
    entries: &[Entry],
    bugs: Bugs,
) -> Option<u8> {
    // The shore (MCL-Q9): the client's the coast's water (below picture 20, as the map's load
    // takes it); the original's test one picture more (below the tables' 21, 0x41865A), so open
    // water's first variant sounded as the shore.
    let shore = match bugs {
        Bugs::Fixed => tile.coast(),
        Bugs::Original => tile.terrain.picture < rules.shore_pictures,
    };
    if tile.terrain.code == WATER && shore {
        return Some(rules.shore);
    }
    // The object a tile records (0x505610): the topmost whose cell there is visitable.
    let classes = |e: &Entry| {
        template(art, e.object).map(|t| {
            let drawn = live.objects.get(e.object).and_then(|o| o.drawn.as_ref());
            let (class, subclass) = drawn.map_or((t.class, t.subclass), |d| (d.class, d.subclass));
            (rules.game_class(class), subclass, class)
        })
    };
    let blocks = |e: &&Entry| template(art, e.object).is_some_and(|t| !t.passable(e.dx, e.dy));
    let top = || entries.iter().rev();
    if let Some((class, subclass, map_class)) = top()
        .find(|e| template(art, e.object).is_some_and(|t| t.visitable(e.dx, e.dy)))
        .and_then(classes)
    {
        return rules.entrance(class, subclass, map_class);
    }
    // Without an entrance: the topmost ownable object blocking it, else the topmost blocking one,
    // else the hole, else none (on coastal land the coast's class).
    let class_of = |e: &Entry| classes(e).map(|(c, _, _)| c);
    let class = top()
        .filter(blocks)
        .find(|e| class_of(e).is_some_and(|c| rules.ownable.contains(&c)))
        .or_else(|| top().find(blocks))
        .and_then(class_of)
        .or_else(|| top().filter_map(class_of).find(|&c| c == rules.hole))
        .unwrap_or(if tile.coast() { COAST_CLASS } else { NO_CLASS });
    if class == NO_CLASS {
        // The special ground (`tile_special_ground` 0x4FCE20): the first from the top.
        let ground = top()
            .filter_map(class_of)
            .find(|c| rules.grounds.contains(c))?;
        return rules.ground_loops.get(&ground).copied();
    }
    rules.blocking.get(&class).copied()
}

/// What the original's sound manager is asked to do (design §3.11).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Cue {
    /// `play_music` with a track's name: the one playing fades out over 900 ms (its place kept),
    /// then this one fades in where it last stopped.
    Music(String),
    /// A step sound starts looping (category 0's one sample: what played there stops).
    Step(String),
    /// The step sound stops.
    StepStop,
    /// A slot's loop starts at a volume (0 to 127).
    Loop {
        /// The slot (0 to [`SLOTS`] − 1).
        slot: usize,
        /// The WAV.
        name: String,
        /// The volume.
        volume: u8,
    },
    /// A slot's loop stops.
    LoopStop {
        /// The slot.
        slot: usize,
    },
    /// A slot's loop takes a volume.
    LoopVolume {
        /// The slot.
        slot: usize,
        /// The volume.
        volume: u8,
    },
}

/// An ambient slot (the manager's +0x228 entries): a sound and its distance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Slot {
    sound: Option<u8>,
    distance: u8,
}

/// The original's sound state on the map and the cues given since they were last taken.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sound {
    rules: Arc<SoundRules>,
    tiles: Arc<TileSounds>,
    /// The terrain whose music was asked for last (+0x58).
    terrain: Option<u8>,
    /// The track asked for last.
    asked: Option<String>,
    slots: [Slot; SLOTS],
    /// The sounds found by the last scan (+0x224): a bit a sound, or with the original's bugs
    /// bit `1 << sound` of 32, the count mod 32 (x86's: MCL-Q8).
    found: u128,
    /// The original's shared bits or the client's set (MCL-Q8).
    bugs: Bugs,
    /// The step sound playing (`step_sound_playing` 0x6968E0) and queued (0x6968E4).
    step: Option<String>,
    next: Option<String>,
    cues: Vec<(u32, Cue)>,
}

impl Sound {
    /// Silence, before the map's screen starts; the found sounds kept as the original keeps them
    /// or as the client does (`bugs`, MCL-Q8).
    #[must_use]
    pub fn new(rules: Arc<SoundRules>, tiles: Arc<TileSounds>, bugs: Bugs) -> Self {
        Self {
            rules,
            tiles,
            terrain: None,
            asked: None,
            slots: [Slot {
                sound: None,
                distance: FAR,
            }; SLOTS],
            found: 0,
            bugs,
            step: None,
            next: None,
            cues: Vec::new(),
        }
    }

    /// The cues given since the last take, with their milliseconds, in order.
    pub fn take(&mut self) -> Vec<(u32, Cue)> {
        core::mem::take(&mut self.cues)
    }

    /// The map's screen starts (0x419B3F): the music of the terrain at the view's `middle` (tile
    /// 0's when off the map), the ambient sounds around it, every slot restarted.
    pub fn start(&mut self, t: u32, middle: [i32; 3], standing: Option<Pos>) {
        let terrain = self
            .tiles
            .terrain(middle)
            .or_else(|| self.tiles.terrain([0, 0, 0]));
        self.terrain = terrain;
        self.music(t, terrain);
        self.ambient(t, middle, true, standing);
    }

    /// `hero_select` (0x4180A0): the ambient sounds around the view's middle, every slot
    /// restarted; when it `centred` the view on the hero, the music of the hero's tile's terrain
    /// if it differs.
    pub fn select(&mut self, t: u32, middle: [i32; 3], hero: Pos, centred: bool) {
        self.ambient(t, middle, true, None);
        let terrain = self.tiles.terrain(at(hero));
        if centred && terrain != self.terrain {
            self.terrain = terrain;
            self.music(t, terrain);
        }
    }

    /// The path's walk starts (0x407D03): the start tile's step sound.
    pub fn walk_start(&mut self, t: u32, hero: Pos) {
        let sound = self.step_sound(hero);
        self.play_step(t, sound);
    }

    /// A step's end (0x48117A to 0x48141B): the walk's `last` stops the step sound; where the
    /// terrain at the view's `middle` differs from the music's, its music and the `hero`'s tile's
    /// step sound, at once at the lifted hero's `frame` 0, else at its wrap; the ambient sounds
    /// around the middle, the slots kept.
    pub fn step_end(&mut self, t: u32, middle: [i32; 3], hero: Pos, last: bool, frame: u32) {
        if last {
            self.stop_step(t);
        }
        let terrain = self.tiles.terrain(middle);
        if terrain != self.terrain {
            self.terrain = terrain;
            self.music(t, terrain);
            let sound = self.step_sound(hero);
            if frame == 0 {
                self.stop_step(t);
                self.play_step(t, sound);
            } else {
                self.next = sound;
            }
        }
        self.ambient(t, middle, false, None);
    }

    /// Pass 9 stepped the lifted hero's frame to 0 (0x47FF88): the queued step sound, the one
    /// playing stopped.
    pub fn wrap(&mut self, t: u32) {
        if let Some(sound) = self.next.take() {
            self.stop_step(t);
            self.play_step(t, Some(sound));
        }
    }

    /// `hero_walk_end` (0x47F825): the step sound stopped, none queued.
    pub fn walk_end(&mut self, t: u32) {
        self.stop_step(t);
    }

    /// A hero's step sound on `hero`'s tile (M2: no hero flies; M4 the engine's).
    fn step_sound(&self, hero: Pos) -> Option<String> {
        self.tiles
            .terrain(at(hero))
            .and_then(|terrain| self.rules.steps.get(&terrain).cloned())
    }

    fn play_step(&mut self, t: u32, sound: Option<String>) {
        if let Some(name) = &sound {
            self.cues.push((t, Cue::Step(name.clone())));
        }
        self.step = sound;
    }

    fn stop_step(&mut self, t: u32) {
        if self.step.take().is_some() {
            self.cues.push((t, Cue::StepStop));
        }
        self.next = None;
    }

    /// `music_track_play` of a terrain's track: nothing without one (rock) or when it is the one
    /// asked for last.
    fn music(&mut self, t: u32, terrain: Option<u8>) {
        let Some(name) = terrain.and_then(|c| self.rules.music.get(&c)) else {
            return;
        };
        if self.asked.as_ref() != Some(name) {
            self.asked = Some(name.clone());
            self.cues.push((t, Cue::Music(name.clone())));
        }
    }

    /// `ambient_sounds_at` (0x4183D0) around `middle`, the slots restarted with `stop`.
    fn ambient(&mut self, t: u32, middle: [i32; 3], stop: bool, standing: Option<Pos>) {
        for (i, slot) in self.slots.iter_mut().enumerate() {
            if slot.sound.is_some() {
                if stop {
                    slot.sound = None;
                    self.cues.push((t, Cue::LoopStop { slot: i }));
                }
                slot.distance = FAR;
            }
        }
        let [x, y, z] = middle;
        if x == NO_TILE {
            return;
        }
        self.found = 0;
        for pass in 1..=2 {
            self.visit(t, middle, 0, pass, standing);
            for d in 1..=RINGS {
                let distance = u8::try_from(d).unwrap_or(FAR);
                for i in 0..2 * d {
                    for tile in [
                        [x - d + i, y - d, z],
                        [x + d, y - d + i, z],
                        [x + d - i, y + d, z],
                        [x - d, y + d - i, z],
                    ] {
                        self.visit(t, tile, distance, pass, standing);
                    }
                }
            }
        }
        let (rules, found, bugs, cues) = (&self.rules, self.found, self.bugs, &mut self.cues);
        for (i, slot) in self.slots.iter_mut().enumerate() {
            let Some(sound) = slot.sound else {
                continue;
            };
            if slot.distance > KEPT {
                slot.sound = None;
                cues.push((t, Cue::LoopStop { slot: i }));
            } else if found & bit(sound, bugs) != 0 {
                let volume = rules.volume(slot.distance);
                cues.push((t, Cue::LoopVolume { slot: i, volume }));
            }
        }
    }

    /// `ambient_sound_visit` (0x418C10): a tile at `distance` in `pass` 1 or 2.
    fn visit(&mut self, t: u32, tile: [i32; 3], distance: u8, pass: u8, standing: Option<Pos>) {
        let Some(sound) = self.tiles.sound(tile, standing) else {
            return;
        };
        if let Some(slot) = self.slots.iter_mut().find(|s| s.sound == Some(sound)) {
            if slot.distance > distance {
                slot.distance = distance;
                self.found |= bit(sound, self.bugs);
            }
            return;
        }
        if pass == 1 {
            return;
        }
        // The first slot of the greatest distance above this one.
        let mut taken = None;
        let mut most = distance;
        for (i, slot) in self.slots.iter().enumerate() {
            if slot.distance > most {
                most = slot.distance;
                taken = Some(i);
            }
        }
        let Some((i, slot)) = taken.and_then(|i| self.slots.get_mut(i).map(|s| (i, s))) else {
            return;
        };
        if slot.sound.is_some() {
            self.cues.push((t, Cue::LoopStop { slot: i }));
        }
        *slot = Slot {
            sound: Some(sound),
            distance,
        };
        let name = self
            .rules
            .loops
            .get(usize::from(sound))
            .cloned()
            .unwrap_or_default();
        let volume = self.rules.volume(distance);
        self.cues.push((
            t,
            Cue::Loop {
                slot: i,
                name,
                volume,
            },
        ));
        self.found ^= bit(sound, self.bugs);
    }
}

/// A sound's bit in the found set: its own (the client's: the 70 loops in 128 bits, one past
/// them none), or as the original's `1 << sound` in 32 bits, where x86 takes the shift's count
/// mod 32 and sounds 32 apart share a bit (MCL-Q8: one's XOR clears the other's, which keeps its
/// earlier volume).
fn bit(sound: u8, bugs: Bugs) -> u128 {
    let at = match bugs {
        Bugs::Fixed => u32::from(sound),
        Bugs::Original => u32::from(sound) % 32,
    };
    1u128.checked_shl(at).unwrap_or(0)
}

/// A cue as a line of text (`h3-map-view`'s reports): the millisecond and what it asks.
#[must_use]
pub fn line(t: u32, cue: &Cue) -> String {
    match cue {
        Cue::Music(name) => format!("{t} music {name}"),
        Cue::Step(name) => format!("{t} step {name}"),
        Cue::StepStop => format!("{t} step-stop"),
        Cue::Loop { slot, name, volume } => format!("{t} loop {slot} {name} {volume}"),
        Cue::LoopStop { slot } => format!("{t} loop-stop {slot}"),
        Cue::LoopVolume { slot, volume } => format!("{t} loop-volume {slot} {volume}"),
    }
}

#[cfg(test)]
pub(crate) mod tests;
