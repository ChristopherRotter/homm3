//! A map set up to play and its drawing (E24 M2 T2 and T3b in `sequences`; shared with `window`
//! since T4a): a recorded sequence's or a situation's map, its live part at 0, the timeline's
//! start and actions, and the [`Painter`] that draws what the screen shows at each change (the
//! view at a draw, the cursor over it). A sequence's walk (M2 T3a, T3b) is played with the hero the
//! live part at 0 selects and the paths its recorded timeline shows; each draw moves that hero,
//! lifts it, slides the view by the walk's offset and shows the path's arrows left (the live
//! part's `walking`). A situation (the capture set's stills) stands still: its kept live part
//! (`situations/kept/<id>.toml`, `h3map.py still`) when there is one, else the map's start with
//! every tile explored but its `hidden` rectangles; a shipped map's (`map`) read from the
//! installation copy's `Maps`.

use std::collections::btree_map::Entry;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use std::sync::Arc;

use h3_formats::lod::LodArchive;
use h3_map::Limits;
use h3_map::model::Pos;
use h3_map::writer::ObjectTemplate;
use h3_map_view::art::MapArt;
use h3_map_view::batch::{self, Batch};
use h3_map_view::compose::{self, ComposeError, Composer, FORMAT};
use h3_map_view::damage::History;
use h3_map_view::layout::{AdventureLayout, CursorArt};
use h3_map_view::live::{Buttons, Facing, Lifted, Live, Moment, Walking};
use h3_map_view::objects::{self, DrawLists, Shapes};
use h3_map_view::scene::{self, Scene, TileIndex, ViewAt};
use h3_map_view::sequence::{self, DEFAULT_HERO_SPEED, DEFAULT_SCROLL_SPEED, Sequence};
use h3_map_view::situation::Situation;
use h3_map_view::sound::{Sound, TileSounds};
use h3_map_view::tables::Tables;
use h3_map_view::timeline::{Action, Cursor, Draw, Start};
use h3_render::Bugs;
use h3_render::screen::{Screen, Widening};
use h3_ui::cursor::{Cursor as Sprite, Image};

use crate::{files, render, situation};

/// Where the harness parks the pointer (`h3adv.py`'s `PARK`, over the right panel): a sequence's
/// start, and a situation's in the window.
pub const PARK: [i32; 2] = [700, 300];

/// How many compositions the history keeps: a frame widened into a buffer older than that is
/// widened whole (E24 M3 T2b2).
const HISTORY: usize = 64;

/// The most assets a view decodes as it is drawn (each one the scene drawn again).
const MOST_DECODED: usize = 64;

/// A frame widened for the window (E24 M3 T2b2, *provisional*): its pixels, and what they hold.
#[derive(Debug, Default)]
pub struct Widened {
    pub xrgb: Vec<u32>,
    pub held: Option<Held>,
}

/// What a frame widened holds: the composition it was widened from (its number in the
/// painter's history) and the cursor's rectangle drawn over it, if any.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Held {
    pub composed: u64,
    pub cursor: Option<[i32; 4]>,
}

/// What every map drawn shares: the committed tables and layout, the installation copy's archives
/// and the game's object templates, whether the original's drawing bugs are reproduced.
pub struct Assets<'a> {
    pub tables: &'a Tables,
    pub layout: &'a AdventureLayout,
    pub archives: &'a [LodArchive<'a>],
    pub templates: &'a [ObjectTemplate],
    pub bugs: Bugs,
}

/// A map set up to play.
pub struct Setup<'a> {
    pub painter: Painter<'a>,
    /// The timeline's start.
    pub start: Start,
    /// The actions at their milliseconds (a situation's none).
    pub actions: BTreeMap<u32, Vec<Action>>,
    /// The recorded timeline (`timelines/<id>.txt`), if any.
    pub recorded: Option<String>,
}

/// A recorded sequence (`<map>/sequences.toml`) set up: its situation's map built by the writer,
/// its live part at 0 (`live`, else `<map>/timelines/<id>-live.toml`).
pub fn for_sequence<'a>(
    assets: &'a Assets<'a>,
    map: &Path,
    install: &Path,
    sequence: &Sequence,
    live: Option<&Path>,
) -> Result<Setup<'a>, String> {
    let walks = sequence.walks();
    let path = map
        .join("situations")
        .join(format!("{}.toml", sequence.situation));
    let text = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let situation = Situation::parse(&text).map_err(|e| format!("{}: {e}", path.display()))?;
    let view = view_at(situation.view, &path)?;
    let recorded_path = map.join("timelines").join(format!("{}.txt", sequence.id));
    let recorded = fs::read_to_string(&recorded_path).ok();
    let bytes = situation::map_bytes(&situation, install, assets.templates)?;
    let raw = h3_map::parse(&bytes, &Limits::default()).map_err(|e| e.to_string())?;
    let live_path = live.map_or_else(
        || {
            map.join("timelines")
                .join(format!("{}-live.toml", sequence.id))
        },
        Path::to_path_buf,
    );
    let mut painter = Painter::new(assets, &raw, Some(&live_path), view, &[])?;
    // A walk's hero is the live part's selected one, its paths the ones the recording shows.
    let (hero, paths) = if walks {
        let live = &painter.live;
        let hero = sequence::selected(&live.heroes, live.viewer)
            .map_err(|e| format!("{}: {e}", live_path.display()))?;
        painter.selected = sequence::selected_index(&live.heroes, live.viewer).ok();
        let paths = sequence.paths(recorded.as_deref().unwrap_or_default())?;
        (Some(hero), paths)
    } else {
        (None, Vec::new())
    };
    let sound = painter.sound(hero.map(|h| h.at))?;
    let start = Start {
        view,
        size: i32::from(situation.size),
        scroll_speed: sequence.scroll_speed()?,
        hero_speed: sequence.hero_speed()?,
        pointer: PARK,
        hero,
        paths,
        path: Vec::new(),
        live: false,
        sound,
        kingdom: painter.live.kingdom.clone(),
        buttons: painter.live.buttons,
    };
    Ok(Setup {
        painter,
        start,
        actions: sequence.actions()?,
        recorded,
    })
}

/// A situation (`<map>/situations/<id>.toml`) set up to stand still: its map (the writer's, or the
/// shipped one it names), its kept live part if any, the original's default speeds, the pointer
/// parked; the viewer's hero selected when it has only one (*provisional*: M4 selects by the
/// game), its path shown as the still shows it (E24 M2 T4b: the window's live input steps it).
pub fn for_situation<'a>(
    assets: &'a Assets<'a>,
    map: &Path,
    install: &Path,
    id: &str,
) -> Result<Setup<'a>, String> {
    let path = map.join("situations").join(format!("{id}.toml"));
    let failed = |why: &dyn std::fmt::Display| format!("{}: {why}", path.display());
    let text = fs::read_to_string(&path).map_err(|e| failed(&e))?;
    let table: toml::Table = text.parse().map_err(|e| failed(&e))?;
    let (raw, view, hidden) = if let Some(name) = table.get("map") {
        // A shipped map loaded whole (E24 M1 T4b): its view, the rest the kept live part's.
        let name = name
            .as_str()
            .ok_or_else(|| failed(&"`map` is not a file name"))?;
        let file = install.join("Maps").join(name);
        let bytes = fs::read(&file).map_err(|e| format!("{}: {e}", file.display()))?;
        let raw = h3_map::read(&bytes, &Limits::default())
            .map_err(|e| format!("{}: {e}", file.display()))?;
        let view: Option<Vec<i32>> = table.get("view").and_then(|v| {
            v.as_array()?
                .iter()
                .map(|n| n.as_integer().and_then(|n| i32::try_from(n).ok()))
                .collect()
        });
        let view = view
            .and_then(|v| <[i32; 3]>::try_from(v).ok())
            .ok_or_else(|| failed(&"`view` is not [x, y, level]"))?;
        (raw, view, Vec::new())
    } else {
        let situation = Situation::parse(&text).map_err(|e| failed(&e))?;
        let bytes = situation::map_bytes(&situation, install, assets.templates)?;
        let raw = h3_map::parse(&bytes, &Limits::default()).map_err(|e| e.to_string())?;
        (raw, situation.view, situation.hidden)
    };
    let view = view_at(view, &path)?;
    let kept = map.join("situations/kept").join(format!("{id}.toml"));
    let live = kept.exists().then_some(kept);
    let mut painter = Painter::new(assets, &raw, live.as_deref(), view, &hidden)?;
    let (heroes, viewer) = (&painter.live.heroes, painter.live.viewer);
    let hero = sequence::selected(heroes, viewer).ok();
    painter.selected = sequence::selected_index(heroes, viewer).ok();
    let sound = painter.sound(hero.map(|h| h.at))?;
    let start = Start {
        view,
        size: i32::try_from(painter.art.size).unwrap_or(i32::MAX),
        scroll_speed: DEFAULT_SCROLL_SPEED,
        hero_speed: DEFAULT_HERO_SPEED,
        pointer: PARK,
        hero,
        paths: Vec::new(),
        path: painter.live.path.clone(),
        live: false,
        sound,
        kingdom: painter.live.kingdom.clone(),
        buttons: painter.live.buttons,
    };
    Ok(Setup {
        painter,
        start,
        actions: BTreeMap::new(),
        recorded: None,
    })
}

/// A situation's view, on the surface.
fn view_at([x, y, z]: [i32; 3], path: &Path) -> Result<ViewAt, String> {
    Ok(ViewAt {
        x,
        y,
        z: u8::try_from(z).map_err(|_| format!("{}: the view's level", path.display()))?,
    })
}

/// A map's drawing: its art, its live part at 0, the shapes and the assets decoded as the draws
/// need them, each level's objects indexed by their tiles; the last view drawn kept in the
/// composer's frame (E24 M3 T2b1: no copy of it).
pub struct Painter<'a> {
    assets: &'a Assets<'a>,
    art: MapArt,
    live: Live,
    /// The live part's moment at 0: the draws' steps count from it.
    base: Moment,
    shapes: Shapes,
    /// Each level's objects by their tiles, for the live part's objects at 0 (which no draw
    /// changes).
    index: Vec<TileIndex>,
    batch: Batch,
    added: BTreeSet<String>,
    /// The frame the last view was drawn into, and that view's draw and buttons.
    composer: Composer,
    shown: Option<(Draw, Buttons)>,
    /// What each composition changed (E24 M3 T2b2), and a list of rectangles kept for its space.
    history: History,
    rects: Vec<[i32; 4]>,
    widening: Widening,
    cursors: BTreeMap<u32, Sprite>,
    /// A walk's hero (its index in the live part's heroes): each draw puts it where the draw has
    /// it, lifted or on its tile; none for a map that walks no hero.
    selected: Option<usize>,
}

impl<'a> Painter<'a> {
    /// The map's drawing from its raw map, its live part at 0 read from `live` (the harness's,
    /// `render --live`'s format) or else the map's start with every tile explored but the
    /// `hidden` rectangles.
    fn new(
        assets: &'a Assets<'a>,
        raw: &h3_map::RawMap,
        live: Option<&Path>,
        view: ViewAt,
        hidden: &[[i32; 4]],
    ) -> Result<Self, String> {
        let tables = assets.tables;
        let mut art = MapArt::from_raw(raw).map_err(|e| e.to_string())?;
        let mut start = Live::at_start(raw, &art, 0, &tables.map.towns);
        match live {
            Some(path) => {
                render::read_live(
                    &path.to_path_buf(),
                    view,
                    &mut art,
                    &mut start,
                    assets.templates,
                )?;
            }
            None => {
                start.moment = Moment::at(0, tables.map.cycle_ms, tables.map.frame_ms);
                render::hide(&art, &mut start, view.z, hidden);
            }
        }
        let archives = assets.archives;
        let mut shapes = files::load_shapes(archives, &objects::pictures(&art, &start))?;
        let heroes: Vec<String> = tables
            .map
            .heroes
            .defs()
            .into_iter()
            .map(|(d, _)| d)
            .collect();
        files::load_frames(archives, &heroes, &mut shapes)?;
        let base = start.moment;
        let index = (0..art.levels)
            .map(|z| {
                let z = u8::try_from(z).map_err(|_| format!("level {z}"))?;
                TileIndex::build(&art, &start, tables, assets.layout, &shapes, z, assets.bugs)
                    .map_err(|e| e.to_string())
            })
            .collect::<Result<Vec<_>, String>>()?;
        let composer = Composer::new(assets.layout.frame).map_err(|e| e.to_string())?;
        let mut painter = Self {
            assets,
            art,
            live: start,
            base,
            shapes,
            index,
            batch: Batch::default(),
            added: BTreeSet::new(),
            composer,
            shown: None,
            history: History::new(HISTORY),
            rects: Vec::new(),
            widening: Widening::new(FORMAT),
            cursors: BTreeMap::new(),
            selected: None,
        };
        painter.add([assets.layout.cursor.def.to_ascii_uppercase()])?;
        painter.preload()?;
        Ok(painter)
    }

    /// Decodes every asset a view of the map draws at 0, each level scanned view by view (E24 M2
    /// T4b3, *provisional*): a walk or a scroll then finds its pictures decoded rather than
    /// decoding them between its frames (a path's arrows shown later, or an object's other
    /// picture, are still decoded when first drawn).
    fn preload(&mut self) -> Result<(), String> {
        let assets = self.assets;
        let view = &assets.layout.view;
        let (across, down) = (
            (view.columns[1] - view.columns[0]).max(1),
            (view.rows[1] - view.rows[0]).max(1),
        );
        let size = i32::try_from(self.art.size).map_err(|_| "the map's size".to_owned())?;
        let mut names = BTreeSet::new();
        for index in &self.index {
            let z = index.level();
            let mut y = 0;
            while y < size {
                let mut x = 0;
                while x < size {
                    let scene = scene::scene_in(
                        index,
                        &self.art,
                        &self.live,
                        assets.tables,
                        assets.layout,
                        &self.shapes,
                        ViewAt { x, y, z },
                        assets.bugs,
                    )
                    .map_err(|e| e.to_string())?;
                    names.extend(batch::names(&scene));
                    x += across;
                }
                y += down;
            }
        }
        self.add(names)
    }

    /// The map's sound from 0 (E24 M2 T5; design §3.11): each tile's ambient sound from every
    /// level's draw lists, the selected `hero`'s tile left to the timeline; none without the
    /// tables' sounds.
    fn sound(&self, hero: Option<Pos>) -> Result<Option<Sound>, String> {
        let Some(rules) = &self.assets.tables.sound else {
            return Ok(None);
        };
        let lists = (0..self.art.levels)
            .map(|z| {
                let z = u8::try_from(z).map_err(|_| format!("level {z}"))?;
                DrawLists::build(&self.art, &self.live, &self.shapes, z).map_err(|e| e.to_string())
            })
            .collect::<Result<Vec<_>, String>>()?;
        let tiles = TileSounds::build(&self.art, &self.live, &lists, rules, hero, self.assets.bugs);
        Ok(Some(Sound::new(
            Arc::new(rules.clone()),
            Arc::new(tiles),
            self.assets.bugs,
        )))
    }

    /// Decodes the named assets not decoded yet.
    fn add(&mut self, names: impl IntoIterator<Item = String>) -> Result<(), String> {
        for name in names {
            if self.added.contains(&name) {
                continue;
            }
            let (_, data) = files::find(self.assets.archives, &name)?
                .ok_or(format!("{name}: in no archive"))?;
            self.batch.add(&name, &data).map_err(|e| e.to_string())?;
            self.added.insert(name);
        }
        Ok(())
    }

    /// The view as a draw shows it, with the buttons a walk sets, drawn into the composer's
    /// frame unless it holds that view already; what it changed recorded (E24 M3 T2b2).
    fn view(&mut self, draw: &Draw, buttons: Buttons) -> Result<&Screen, String> {
        let key = (draw.clone(), buttons);
        if self.shown.as_ref() == Some(&key) {
            return Ok(self.composer.screen());
        }
        self.shown = None;
        let mut decoded = 0;
        loop {
            let scene = self.scene(draw, buttons)?;
            let composed = self.composer.compose_owned(scene, &self.batch).map(|_| ());
            self.history.push(self.composer.damage());
            match composed {
                Ok(()) => break,
                // An asset the preload did not decode (a path's arrows shown later, an object's
                // other picture): decoded now and the scene drawn again (E24 M3 T2b2: no scene's
                // names looked for a frame).
                Err(ComposeError::Missing(name))
                    if decoded < MOST_DECODED && self.decodes(&name) =>
                {
                    decoded += 1;
                }
                Err(e) => return Err(e.to_string()),
            }
        }
        self.shown = Some(key);
        Ok(self.composer.screen())
    }

    /// Whether the asset `name` (as a scene spells it) was not decoded and now is.
    fn decodes(&mut self, name: &str) -> bool {
        let upper = name.to_ascii_uppercase();
        !self.added.contains(&upper) && self.add([upper]).is_ok()
    }

    /// The scene a draw shows with the buttons a walk sets.
    fn scene(&mut self, draw: &Draw, buttons: Buttons) -> Result<Scene, String> {
        let base = self.base;
        self.live.moment = Moment {
            cycles: base.cycles.wrapping_add(draw.cycles),
            river_cycles: base.river_cycles.wrapping_add(draw.cycles),
            frames: base.frames.wrapping_add(draw.frames),
        };
        if let Some(index) = self.selected {
            self.walk(index, draw);
        }
        self.live.buttons = Some(buttons);
        let assets = self.assets;
        let z = draw.view.z;
        let index = self
            .index
            .get(usize::from(z))
            .ok_or(format!("level {z} is not on the map"))?;
        scene::scene_in(
            index,
            &self.art,
            &self.live,
            assets.tables,
            assets.layout,
            &self.shapes,
            draw.view,
            assets.bugs,
        )
        .map_err(|e| e.to_string())
    }

    /// A walk's part of a draw in the live part: the selected hero's tile and facing, lifted at
    /// the view's middle or on its tile, the view's offset, the path's arrows left.
    fn walk(&mut self, index: usize, draw: &Draw) {
        let live = &mut self.live;
        if let (Some(hero), Some(at)) = (live.heroes.get_mut(index), draw.hero) {
            hero.at = at.at;
            hero.facing = Facing::from_number(u8::try_from(at.facing.number()).unwrap_or(0))
                .unwrap_or(hero.facing);
        }
        live.path.clone_from(&draw.path);
        live.walking = Walking {
            offset: draw.offset,
            lifted: draw.lifted.map(|[group, frame]| Lifted {
                hero: index,
                group,
                frame,
            }),
        };
    }

    /// The screen a draw, the buttons and a cursor show: the view and the panels, the cursor over
    /// them.
    pub fn screen(
        &mut self,
        drawn: &Draw,
        buttons: Buttons,
        cursor: Cursor,
    ) -> Result<Screen, String> {
        let mut screen = self.view(drawn, buttons)?.clone();
        let Cursor {
            frame,
            at: [x, y],
            shown,
        } = cursor;
        if shown {
            let assets = self.assets;
            let cursor = cursor_in(&mut self.cursors, &self.batch, &assets.layout.cursor, frame)?;
            cursor.draw(&mut screen, x, y);
        }
        Ok(screen)
    }

    /// [`Painter::screen`] composed afresh, with no frame kept (E24 M3 T2b2: the window's check
    /// of the frames it made).
    pub fn fresh(
        &mut self,
        drawn: &Draw,
        buttons: Buttons,
        cursor: Cursor,
    ) -> Result<Screen, String> {
        let scene = self.scene(drawn, buttons)?;
        let mut screen = compose::compose(&scene, &self.batch, self.assets.layout.frame)
            .map_err(|e| e.to_string())?;
        if cursor.shown {
            let assets = self.assets;
            let sprite = cursor_in(
                &mut self.cursors,
                &self.batch,
                &assets.layout.cursor,
                cursor.frame,
            )?;
            sprite.draw(&mut screen, cursor.at[0], cursor.at[1]);
        }
        Ok(screen)
    }

    /// [`Painter::screen`] widened into `out` (E24 M3 T2b1, T2b2, *provisional*): the view's
    /// frame widened by the table, only what changed since the frame `out` holds when the
    /// history still has it (the rectangles composed since, and its cursor's); then only what
    /// lies under the cursor cut out, drawn on and widened over it; no copy of the frame.
    pub fn frame_into(
        &mut self,
        drawn: &Draw,
        buttons: Buttons,
        cursor: Cursor,
        out: &mut Widened,
    ) -> Result<(), String> {
        self.view(drawn, buttons)?;
        let screen = self.composer.screen();
        let now = self.history.now();
        self.rects.clear();
        let since = out.held.filter(|held| {
            out.xrgb.len() == screen.pixels().len()
                && self.history.between(held.composed, now, &mut self.rects)
        });
        match since {
            Some(held) => {
                self.rects.extend(held.cursor);
                for rect in &self.rects {
                    self.widening
                        .widen_rect(screen, rect.map(i64::from), &mut out.xrgb);
                }
            }
            None => self.widening.widen(screen, &mut out.xrgb),
        }
        out.held = Some(Held {
            composed: now,
            cursor: None,
        });
        let Cursor {
            frame,
            at: [x, y],
            shown,
        } = cursor;
        if !shown {
            return Ok(());
        }
        let assets = self.assets;
        let sprite = cursor_in(&mut self.cursors, &self.batch, &assets.layout.cursor, frame)?;
        let rect = sprite.rect(x, y);
        if let Some(mut under) = screen.cut(rect) {
            let (left, top) = (rect[0].max(0), rect[1].max(0));
            let at = |v: i32, from: i64| i32::try_from(i64::from(v) - from).unwrap_or(i32::MIN);
            sprite.draw(&mut under, at(x, left), at(y, top));
            self.widening
                .widen_onto(&under, [left, top], &mut out.xrgb, screen.width());
            let side = |v: i64| i32::try_from(v).unwrap_or(i32::MAX);
            let (w, h) = (i64::from(under.width()), i64::from(under.height()));
            out.held = Some(Held {
                composed: now,
                cursor: Some([side(left), side(top), side(left + w), side(top + h)]),
            });
        }
        Ok(())
    }

    /// Pushes to `out` what differs between two frames widened (E24 M3 T2b2): the rectangles
    /// composed between theirs and both cursors'; `false` when the history no longer has them.
    pub fn changed(&self, a: Held, b: Held, out: &mut Vec<[i32; 4]>) -> bool {
        let known = self.history.between(a.composed, b.composed, out);
        if known {
            out.extend(a.cursor);
            out.extend(b.cursor);
        }
        known
    }

    /// The cursor's frame as an image (the hardware cursor's, E24 M2 T4b).
    pub fn cursor_image(&mut self, frame: u32) -> Result<Image, String> {
        let assets = self.assets;
        Ok(cursor_in(&mut self.cursors, &self.batch, &assets.layout.cursor, frame)?.image(FORMAT))
    }
}

/// The layout's cursor's frame, read once into `cursors`.
fn cursor_in<'c>(
    cursors: &'c mut BTreeMap<u32, Sprite>,
    batch: &Batch,
    art: &CursorArt,
    frame: u32,
) -> Result<&'c Sprite, String> {
    match cursors.entry(frame) {
        Entry::Occupied(known) => Ok(known.into_mut()),
        Entry::Vacant(new) => {
            let [hx, hy] = usize::try_from(frame)
                .ok()
                .and_then(|f| art.hot.get(f).copied())
                .unwrap_or_default();
            let hot = (
                u32::try_from(hx).unwrap_or(0),
                u32::try_from(hy).unwrap_or(0),
            );
            let sprite = batch
                .sprite(&art.def.to_ascii_uppercase())
                .and_then(|s| Sprite::read(s, frame, hot, FORMAT))
                .ok_or_else(|| format!("{}: no frame {frame}", art.def))?;
            Ok(new.insert(sprite))
        }
    }
}
