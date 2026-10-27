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

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fs;
use std::path::Path;
use std::sync::Arc;

use h3_formats::lod::LodArchive;
use h3_map::Limits;
use h3_map::model::Pos;
use h3_map::writer::ObjectTemplate;
use h3_map_view::art::MapArt;
use h3_map_view::batch::{self, Batch};
use h3_map_view::compose::{FORMAT, compose};
use h3_map_view::cursor::{Cursor as Sprite, Image};
use h3_map_view::layout::AdventureLayout;
use h3_map_view::live::{Facing, Lifted, Live, Moment, Walking};
use h3_map_view::objects::{self, DrawLists, Shapes};
use h3_map_view::scene::{self, ViewAt};
use h3_map_view::sequence::{self, DEFAULT_HERO_SPEED, DEFAULT_SCROLL_SPEED, Sequence};
use h3_map_view::situation::Situation;
use h3_map_view::sound::{Sound, TileSounds};
use h3_map_view::tables::Tables;
use h3_map_view::timeline::{Action, Cursor, Draw, Start};
use h3_render::Bugs;
use h3_render::screen::Screen;

use crate::{files, render, situation};

/// Where the harness parks the pointer (`h3adv.py`'s `PARK`, over the right panel): a sequence's
/// start, and a situation's in the window.
pub const PARK: [i32; 2] = [700, 300];

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
/// need them; the last view drawn kept.
pub struct Painter<'a> {
    assets: &'a Assets<'a>,
    art: MapArt,
    live: Live,
    /// The live part's moment at 0: the draws' steps count from it.
    base: Moment,
    shapes: Shapes,
    batch: Batch,
    added: BTreeSet<String>,
    /// The last two views drawn (the window's frame shown and the one composed ahead).
    views: VecDeque<(Draw, Screen)>,
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
        let mut painter = Self {
            assets,
            art,
            live: start,
            base,
            shapes,
            batch: Batch::default(),
            added: BTreeSet::new(),
            views: VecDeque::new(),
            cursors: BTreeMap::new(),
            selected: None,
        };
        painter.add([assets.layout.cursor.def.to_ascii_uppercase()])?;
        Ok(painter)
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
        let tiles = TileSounds::build(&self.art, &self.live, &lists, rules, hero);
        Ok(Some(Sound::new(Arc::new(rules.clone()), Arc::new(tiles))))
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

    /// The view as a draw shows it (the last two kept).
    fn view(&mut self, draw: &Draw) -> Result<Screen, String> {
        if let Some((_, screen)) = self.views.iter().find(|(d, _)| d == draw) {
            return Ok(screen.clone());
        }
        let base = self.base;
        self.live.moment = Moment {
            cycles: base.cycles.wrapping_add(draw.cycles),
            river_cycles: base.river_cycles.wrapping_add(draw.cycles),
            frames: base.frames.wrapping_add(draw.frames),
        };
        if let Some(index) = self.selected {
            self.walk(index, draw);
        }
        let assets = self.assets;
        let scene = scene::scene(
            &self.art,
            &self.live,
            assets.tables,
            assets.layout,
            &self.shapes,
            draw.view,
            assets.bugs,
        )
        .map_err(|e| e.to_string())?;
        self.add(batch::names(&scene))?;
        let screen =
            compose(&scene, &self.batch, assets.layout.frame).map_err(|e| e.to_string())?;
        if self.views.len() == 2 {
            self.views.pop_front();
        }
        self.views.push_back((draw.clone(), screen.clone()));
        Ok(screen)
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

    /// The screen a draw and a cursor show: the view, the cursor over it.
    pub fn frame(&mut self, drawn: &Draw, cursor: Cursor) -> Result<Screen, String> {
        let mut screen = self.view(drawn)?;
        let Cursor {
            frame,
            at: [x, y],
            shown,
        } = cursor;
        if !shown {
            return Ok(screen);
        }
        let cursor = self.cursor(frame)?;
        cursor.draw(&mut screen, x, y);
        Ok(screen)
    }

    /// The cursor's frame as an image (the hardware cursor's, E24 M2 T4b).
    pub fn cursor_image(&mut self, frame: u32) -> Result<Image, String> {
        Ok(self.cursor(frame)?.image(FORMAT))
    }

    /// The layout's cursor's frame, read once.
    fn cursor(&mut self, frame: u32) -> Result<&Sprite, String> {
        let art = &self.assets.layout.cursor;
        if !self.cursors.contains_key(&frame) {
            let [hx, hy] = usize::try_from(frame)
                .ok()
                .and_then(|f| art.hot.get(f).copied())
                .unwrap_or_default();
            let hot = (
                u32::try_from(hx).unwrap_or(0),
                u32::try_from(hy).unwrap_or(0),
            );
            let sprite = self
                .batch
                .sprite(&art.def.to_ascii_uppercase())
                .and_then(|s| Sprite::read(s, frame, hot, FORMAT))
                .ok_or_else(|| format!("{}: no frame {frame}", art.def))?;
            self.cursors.insert(frame, sprite);
        }
        self.cursors
            .get(&frame)
            .ok_or_else(|| format!("{}: no frame {frame}", art.def))
    }
}
