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

use h3_assets::fnt::Font;
use h3_formats::lod::LodArchive;
use h3_formats::text::decode_windows_1252;
use h3_map::Limits;

use h3_map::model::{Body, EventBody, Pos, RawMap};
use h3_map::writer::ObjectTemplate;
use h3_map_view::art::MapArt;
use h3_map_view::batch::{self, Batch};
use h3_map_view::compose::{self, ComposeError, Composer, FORMAT};
use h3_map_view::damage::{Held, History, drop_held};
use h3_map_view::finds::{self, Find, Gain, Told};
use h3_map_view::hover::Hover;
use h3_map_view::layout::{AdventureLayout, CursorArt};
use h3_map_view::live::{
    Buttons, Calendar, Facing, Lifted, Live, Moment, Selected, Status, Walking,
};
use h3_map_view::minimap::Shown;
use h3_map_view::objects::{self, DrawLists, Shapes};
use h3_map_view::scene::{self, Scene, TileIndex, ViewAt};
use h3_map_view::sequence::{self, DEFAULT_HERO_SPEED, DEFAULT_SCROLL_SPEED, Sequence};
use h3_map_view::situation::Situation;
use h3_map_view::sound::{Sound, TileSounds};
use h3_map_view::tables::Tables;
use h3_map_view::timeline::{
    Action, Computer, Cursor, DEFAULT_CURSOR, Direction, Dissolve, Draw, Found, Hero, NextTurn,
    Start, TownBoxes,
};
use h3_map_view::{status, turn};
use h3_render::Bugs;
use h3_render::screen::{Screen, Widening};
use h3_ui::cursor::{Cursor as Sprite, Image};
use h3_ui::message::{self, MessageBox, Picture, Style};
use h3_ui::text::Args;

use crate::{files, render, situation};

/// Where the harness parks the pointer (`h3adv.py`'s `PARK`, over the right panel): a sequence's
/// start, and a situation's in the window.
pub const PARK: [i32; 2] = [700, 300];

/// A template's cells across and down (8 × 6, `Shape::new`'s).
const CELLS: [u8; 2] = [8, 6];

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
    // The mouse-over's facts (E24 M3 T6a1) select a hero too: the cursor reads it.
    let (hero, paths) = if walks || painter.live.hover.is_some() {
        let live = &painter.live;
        let hero =
            sequence::selected_hero(live).map_err(|e| format!("{}: {e}", live_path.display()))?;
        painter.selected = sequence::selected_in(live).ok();
        let paths = if walks {
            sequence.paths(recorded.as_deref().unwrap_or_default())?
        } else {
            Vec::new()
        };
        (Some(hero), paths)
    } else {
        (None, Vec::new())
    };
    let sound = painter.sound(hero.map(|h| h.at))?;
    // End Turn's question and the turns' messages (E24 M4 T1a); a computer's walks from the
    // recorded `hero` lines (E24 M4 T1b2).
    let lines = sequence.heroes(recorded.as_deref().unwrap_or_default())?;
    let (question, turns) = painter
        .boxes(&lines)
        .map_err(|e| format!("{}: {e}", live_path.display()))?;
    // The finds a walk tells and the turn without a town's boxes (E24 M4 T2a2).
    let hover = painter.hover()?;
    let finds = painter.finds(&raw, hover.as_deref())?;
    let town = painter.town()?;
    let start = Start {
        view,
        size: i32::from(situation.size),
        scroll_speed: sequence.scroll_speed()?,
        hero_speed: sequence.hero_speed()?,
        pointer: PARK,
        hero,
        paths,
        // The path shown at 0 (E24 M3 T6b1: the staging's selection's, a sequence of choices').
        path: painter.live.path.clone(),
        live: false,
        sound,
        kingdom: painter.live.kingdom.clone(),
        // The selected hero's points memory held (E24 M3 T4b): the recording's.
        points: sequence.points(recorded.as_deref().unwrap_or_default())?,
        buttons: painter.live.buttons,
        // The status window's kind the recording asked at 0 (E24 M3 T5b2).
        asked: sequence.asked()?,
        hover,
        levels: painter.art.levels,
        heroes: heroes(&painter.live),
        index: painter.selected,
        chosen: painter.live.chosen.clone(),
        player: painter.live.viewer,
        move_reminder: sequence.move_reminder()?,
        question: Some(question),
        turns,
        computer: Computer {
            speed: sequence.computer_speed()?,
            blackout: sequence.blackout()?,
        },
        finds,
        town: Some(town),
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
    let hero = sequence::selected_hero(&painter.live).ok();
    painter.selected = sequence::selected_in(&painter.live).ok();
    let sound = painter.sound(hero.map(|h| h.at))?;
    let (question, turns) = painter.boxes(&[])?;
    let hover = painter.hover()?;
    let finds = painter.finds(&raw, hover.as_deref())?;
    let town = painter.town()?;
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
        points: Vec::new(),
        buttons: painter.live.buttons,
        asked: None,
        hover,
        levels: painter.art.levels,
        heroes: heroes(&painter.live),
        index: painter.selected,
        chosen: painter.live.chosen.clone(),
        player: painter.live.viewer,
        // The harness's settings (E24 M4 T1a, *provisional*: the window's option is M5's).
        move_reminder: false,
        question: Some(question),
        turns,
        computer: Computer::default(),
        finds,
        town: Some(town),
    };
    Ok(Setup {
        painter,
        start,
        actions: BTreeMap::new(),
        recorded: None,
    })
}

/// The live part's heroes on their tiles and facings (the timeline's [`Start::heroes`]).
fn heroes(live: &Live) -> Vec<Hero> {
    live.heroes
        .iter()
        .map(|h| Hero {
            at: h.at,
            facing: Direction::ALL[usize::from(h.facing.number()) % Direction::ALL.len()],
        })
        .collect()
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
    /// Each level's objects by their tiles, for the live part's objects at 0 less the objects
    /// taken off the map by the draw last drawn (E24 M4 T2a2 part B: `gone`, each level's index
    /// built again), and the other set's last indexes (a dissolve's two draws, one after the
    /// other).
    index: Vec<TileIndex>,
    gone: Arc<Vec<usize>>,
    spare: Option<(Arc<Vec<usize>>, Vec<TileIndex>)>,
    batch: Batch,
    added: BTreeSet<String>,
    /// The frame the last view was drawn into, and that view's draw and buttons.
    composer: Composer,
    shown: Option<(Draw, Buttons)>,
    /// What each composition changed (E24 M3 T2b2), and a list of rectangles kept for its space.
    history: History,
    rects: Vec<[i32; 4]>,
    widening: Widening,
    /// The cursors' frames read, by the mouse manager's type and the frame.
    cursors: BTreeMap<[u32; 2], Sprite>,
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
        raw: &RawMap,
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
            gone: Arc::new(Vec::new()),
            spare: None,
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
        painter.add([
            assets.layout.cursor.def.to_ascii_uppercase(),
            assets.layout.cursor.default_def.to_ascii_uppercase(),
            assets.layout.message.font.to_ascii_uppercase(),
        ])?;
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

    /// The map's mouse-over from 0 (E24 M3 T6a1; design §4.10): its tiles from every level's draw
    /// lists, the live part's facts; none without them.
    fn hover(&self) -> Result<Option<Arc<Hover>>, String> {
        if self.live.hover.is_none() {
            return Ok(None);
        }
        let lists = (0..self.art.levels)
            .map(|z| {
                let z = u8::try_from(z).map_err(|_| format!("level {z}"))?;
                DrawLists::build(&self.art, &self.live, &self.shapes, z).map_err(|e| e.to_string())
            })
            .collect::<Result<Vec<_>, String>>()?;
        Ok(Hover::new(
            &self.art,
            &self.live,
            &self.assets.tables.records,
            &lists,
        ))
    }

    /// The boxes End Turn opens (E24 M4 T1a, design §5.1): its question (GENRLTXT 56, style 2) and
    /// each of the live part's turns with its week's or month's message at a week's start (style
    /// 1), laid out in the box's font, framed in the viewer's colours; since E24 M4 T1b2 each
    /// turn's heroes standing and a computer's walks from the recorded `hero` lines (`lines`).
    fn boxes(&self, lines: &[(u32, Hero)]) -> Result<(Arc<MessageBox>, Vec<NextTurn>), String> {
        let assets = self.assets;
        let layout = &assets.layout.message;
        let font = self
            .batch
            .font(&layout.font)
            .ok_or_else(|| format!("{}: not decoded", layout.font))?;
        let texts = &assets.tables.texts;
        let framed = (self.live.viewer, assets.bugs);
        let laid = |text: &str, style| Arc::new(message::layout(layout, font, text, style, framed));
        let question = laid(&turn::question(texts), Style::OkCancel);
        let art = &assets.tables.map.turn;
        // The week of the Deity's growth, its first creature's, read when a turn needs it.
        let deity = self.live.turns.iter().any(|t| t.week == Calendar::Special);
        let growth = if deity {
            crate::texts::growth(assets.archives, art.deity[0])?
        } else {
            0
        };
        let live = &self.live;
        let turns = live
            .turns
            .iter()
            .enumerate()
            .map(|(i, t)| {
                Ok(NextTurn {
                    turn: t.clone(),
                    message: turn::message(texts, art, (t.kingdom.date, t.week, t.month), growth)
                        .map(|text| laid(&text, Style::Ok)),
                    heroes: turn::placed(&live.heroes, &live.boats, t)?,
                    walks: turn::walks(&live.turns, i, &live.heroes, lines)?,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        Ok((question, turns))
    }

    /// The pictures of `told` (`finds::picture`: each DEF decoded for its size, its caption in
    /// the small font decoded too; E24 M4 T2a2).
    fn pictures(&mut self, told: &Told) -> Result<Vec<Picture>, String> {
        let assets = self.assets;
        let layout = &assets.layout.message;
        let (art, texts) = (&assets.tables.map.finds, &assets.tables.texts);
        let shown = told
            .pictures
            .iter()
            .map(|&(kind, subtype)| {
                finds::picture(art, texts, kind, subtype)
                    .ok_or_else(|| format!("a picture of type {kind}: not built"))
            })
            .collect::<Result<Vec<_>, String>>()?;
        self.add(
            shown
                .iter()
                .map(|p| p.def.to_ascii_uppercase())
                .chain([layout.caption_font.to_ascii_uppercase()]),
        )?;
        let pictures = shown
            .into_iter()
            .map(|p| {
                let sprite = self
                    .batch
                    .sprite(&p.def)
                    .ok_or_else(|| format!("{}: not decoded", p.def))?;
                let side = |v: u32| i32::try_from(v).unwrap_or(i32::MAX);
                Ok(Picture {
                    size: [side(sprite.def.width), side(sprite.def.height)],
                    def: p.def,
                    frame: p.frame,
                    caption: p.caption,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        Ok(pictures)
    }

    /// The message box's text font and caption font, decoded.
    fn fonts(&self) -> Result<(&Font, &Font), String> {
        let layout = &self.assets.layout.message;
        let font = |name: &str| {
            self.batch
                .font(name)
                .ok_or_else(|| format!("{name}: not decoded"))
        };
        Ok((font(&layout.font)?, font(&layout.caption_font)?))
    }

    /// A box of a find or of a turn without a town (E24 M4 T2a2, design §5.3): `told`'s text and
    /// pictures in an OK box of `message_box` 0x4F6570 (`message::told`), framed in the viewer's
    /// colours.
    fn told(&mut self, told: &Told) -> Result<Arc<MessageBox>, String> {
        let pictures = self.pictures(told)?;
        let assets = self.assets;
        Ok(Arc::new(message::told(
            &assets.layout.message,
            self.fonts()?,
            &told.text,
            Style::Ok,
            &pictures,
            (self.live.viewer, assets.bugs),
        )))
    }

    /// An event's box (E24 M4 T2a2 part B, design §5.3): `told`'s text and up to eight pictures
    /// in a box of `message_box_pictures` 0x4F7690 (`message::pictures`).
    fn pictured(&mut self, told: &Told) -> Result<Arc<MessageBox>, String> {
        let pictures = self.pictures(told)?;
        let assets = self.assets;
        Ok(Arc::new(message::pictures(
            &assets.layout.message,
            self.fonts()?,
            &told.text,
            &pictures,
            (self.live.viewer, assets.bugs),
        )))
    }

    /// The selected hero's name (E24 M4 T2a2 part B: an event's texts' `%s`): the status
    /// window's hero's, its own name the map's, else its type's; empty without one.
    fn hero_name(&self) -> String {
        let texts = &self.assets.tables.texts;
        match &self.live.kingdom.status {
            Some(Status::Hero(hero)) => match self.art.hero_names.get(&hero.hero) {
                Some(own) if hero.own_name => own.clone(),
                _ => texts.text(&status::hero_key(hero.hero), &Args::new()),
            },
            _ => String::new(),
        }
    }

    /// Whether the selected hero's spell points are at their maximum (a Magic Well's facts;
    /// *provisional*: `finds::full_mana` by the status window's Knowledge, no Intelligence).
    fn full_mana(&self) -> bool {
        let kingdom = &self.live.kingdom;
        let mana = match kingdom.selected {
            Selected::Hero(i) => kingdom.heroes.get(i).map(|h| h.mana),
            _ => None,
        };
        match (&kingdom.status, mana) {
            (Some(Status::Hero(hero)), Some(mana)) => finds::full_mana(mana, hero.skills[3]),
            _ => false,
        }
    }

    /// The finds a walk's end or a key's step tells (E24 M4 T2a2, design §5.3): each object of a
    /// find's class by its entrance tile (the mouse-over's tiles), its facts from the live part's
    /// `[hover]` facts and a sign's text from the map `raw` (*provisional*: `Find::of`, the facts
    /// at 0), its first visit's box and its later one's; a pile's and an artifact's with a
    /// message or guards left out (their question: T2b); then each event's boxes (part B,
    /// [`Painter::event`]); none without the facts.
    fn finds(&mut self, raw: &RawMap, hover: Option<&Hover>) -> Result<Vec<Found>, String> {
        let Some(hover) = hover else {
            return Ok(Vec::new());
        };
        let assets = self.assets;
        let (art, texts) = (&assets.tables.map.finds, &assets.tables.texts);
        let facts = hover.facts();
        let size = u8::try_from(self.art.size).map_err(|_| "the map's size".to_owned())?;
        let levels = u8::try_from(self.art.levels).map_err(|_| "the map's levels".to_owned())?;
        let (hero, full) = (self.hero_name(), self.full_mana());
        let mut out = Vec::new();
        for z in 0..levels {
            for y in 0..size {
                for x in 0..size {
                    let at = [i32::from(x), i32::from(y), i32::from(z)];
                    let pos = Pos { x, y, z };
                    let Some(tile) = hover.base(at) else {
                        continue;
                    };
                    let Some(o) = tile.object.filter(|_| tile.entrance) else {
                        continue;
                    };
                    let body = raw.objects.get(o).map(|object| &object.body);
                    let asks = matches!(
                        body,
                        Some(
                            Body::Resource {
                                message: Some(_),
                                ..
                            } | Body::Artifact(Some(_))
                        )
                    );
                    if asks || !finds::CLASSES.contains(&tile.class) {
                        continue;
                    }
                    let text = match body {
                        Some(Body::Sign(text)) => Some(decode_windows_1252(text.as_bytes())),
                        _ => None,
                    };
                    let Some(find) = Find::of(
                        [tile.class, tile.subclass],
                        tile.word,
                        facts.visits.get(o).copied(),
                        facts.contents.get(&o),
                        (text.as_deref(), i32::from(x)),
                    ) else {
                        continue;
                    };
                    let find = find.full(full);
                    let first = match find.told(art, texts) {
                        Some(told) => vec![(self.told(&told)?, Gain::default())],
                        None => Vec::new(),
                    };
                    let again = match find.visited().told(art, texts) {
                        Some(told) if !find.taken() => Some(self.told(&told)?),
                        _ => None,
                    };
                    out.push(Found {
                        at: pos,
                        first,
                        again,
                        gain: find.gain(art),
                        found: find.found(texts),
                        taken: find.taken().then_some(o),
                        event: false,
                    });
                }
            }
        }
        // The events: drawn nowhere, so on no tile's record (`place_object_tiles` 0x505B20 marks
        // their tiles only); each by its template's first visitable cell.
        for (o, object) in raw.objects.iter().enumerate() {
            let Body::Event(event) = &object.body else {
                continue;
            };
            let Some((template, art)) =
                objects::template(&self.art, o).zip(self.art.objects.get(o))
            else {
                continue;
            };
            let cell = (0..CELLS[0])
                .flat_map(|dx| (0..CELLS[1]).map(move |dy| (dx, dy)))
                .find(|&(dx, dy)| template.visitable(dx, dy));
            let at = cell.and_then(|(dx, dy)| {
                Some(Pos {
                    x: art.anchor.x.checked_sub(dx)?,
                    y: art.anchor.y.checked_sub(dy)?,
                    z: art.anchor.z,
                })
            });
            if let Some(at) = at
                && let Some(found) = self.event(at, event, &hero)?
            {
                out.push(found);
            }
        }
        Ok(out)
    }

    /// An event's find (E24 M4 T2a2 part B, design §5.3: `event_fire` 0x4A0ED0 after the step onto
    /// it, `grant_rewards` 0x49FA90's boxes for the hero named `hero`, the map's text in the
    /// first; the scenario's test after them): none when it does not fire for the viewer's colour
    /// or has guards (a battle: not built). *Provisional:* told once (one the map leaves after a
    /// visit has nothing more; one it keeps would give its rewards again, not built).
    fn event(&mut self, at: Pos, event: &EventBody, hero: &str) -> Result<Option<Found>, String> {
        let fires = event.players & (1 << (self.live.viewer % 8)) != 0;
        let guarded = event.message.as_ref().is_some_and(|m| m.guards.is_some());
        if !fires || guarded {
            return Ok(None);
        }
        let text = event
            .message
            .as_ref()
            .map(|m| decode_windows_1252(m.text.as_bytes()))
            .unwrap_or_default();
        let texts = &self.assets.tables.texts;
        let (boxes, gain) = finds::rewards(&event.rewards, texts, hero, &text);
        let first = boxes
            .iter()
            .map(|reward| Ok((self.pictured(&reward.told)?, reward.shows)))
            .collect::<Result<Vec<_>, String>>()?;
        Ok(Some(Found {
            at,
            first,
            again: None,
            gain,
            found: None,
            taken: None,
            event: true,
        }))
    }

    /// The viewer's boxes of a turn without a town (E24 M4 T2a2, design §5.3): the warning and
    /// the days left's for each of the tables' days, the player named as CLT-Q100 (b) has it (the
    /// colour's name: the viewer has no player name), the original's empty name with `--bugs
    /// original`.
    fn town(&mut self) -> Result<TownBoxes, String> {
        let assets = self.assets;
        let texts = &assets.tables.texts;
        let (player, original) = (self.live.viewer, assets.bugs == Bugs::Original);
        let lost = self.told(&finds::town_lost(texts, player, original))?;
        let left = (1..=i32::from(assets.tables.map.turn.town_days))
            .map(|days| self.told(&finds::days_left(texts, player, days, original)))
            .collect::<Result<Vec<_>, String>>()?;
        Ok(TownBoxes { lost, left })
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
        // Compared by reference, the draw cloned only on a miss (T7b2, F1-5: a walk's path at
        // every frame).
        if self
            .shown
            .as_ref()
            .is_some_and(|(d, b)| d == draw && *b == buttons)
        {
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
        self.shown = Some((draw.clone(), buttons));
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
        // The map's heroes as the draw has them (E24 M4 T1b2: a computer's moves), then the
        // selected hero as the draw has it (E24 M3 T6b1: a hero chosen is another; the one chosen
        // away from keeps the tile and facing its last draw gave it).
        for (live, hero) in self.live.heroes.iter_mut().zip(draw.heroes.iter()) {
            live.at = hero.at;
            live.facing = Facing::from_number(u8::try_from(hero.facing.number()).unwrap_or(0))
                .unwrap_or(live.facing);
        }
        match (draw.selected, self.selected) {
            (Some(index), _) => self.walk(index, draw),
            // A town chosen (no hero selected): the path cleared, the hero put back.
            (None, Some(_)) => {
                self.live.path.clone_from(&draw.path);
                self.live.walking = Walking {
                    offset: draw.offset,
                    lifted: None,
                };
            }
            // No hero selected from the start: the live part's as it is.
            (None, None) => {}
        }
        // The minimap as last drawn (E24 M3 T3b): its box's corner and the selected hero's tile.
        self.live.minimap = Some(Shown {
            view: draw.minimap.view,
            hero: draw.minimap.index.zip(draw.minimap.hero),
        });
        // The buttons pressed (E24 M3 T6b1).
        self.live.pressed.clone_from(&draw.pressed);
        // A computer player's turn's buttons and shield (E24 M4 T1b2).
        self.live.computer = draw.computer;
        // The lists as last shown (E24 M3 T4b).
        if self.live.kingdom != *draw.kingdom {
            self.live.kingdom.clone_from(&draw.kingdom);
        }
        self.live.buttons = Some(buttons);
        // The status bar's hint as last shown (E24 M3 T6a1).
        self.live.hint = Some(draw.hint.clone());
        // The objects taken off the map (E24 M4 T2a2 part B).
        self.taken(&draw.gone)?;
        let assets = self.assets;
        let z = draw.view.z;
        let index = self
            .index
            .get(usize::from(z))
            .ok_or(format!("level {z} is not on the map"))?;
        let mut scene = scene::scene_in(
            index,
            &self.art,
            &self.live,
            assets.tables,
            assets.layout,
            &self.shapes,
            draw.view,
            assets.bugs,
        )
        .map_err(|e| e.to_string())?;
        // The message box over everything (E24 M4 T1a): its shadow and its items.
        if let Some(message) = &draw.message {
            let draws = message
                .dialog
                .draws(&message.shown, &assets.layout.player_palette);
            scene
                .draws
                .extend(draws.into_iter().map(scene::Draw::Dialog));
        }
        Ok(scene)
    }

    /// The live part's objects standing but those `gone` (E24 M4 T2a2 part B: taken off the map,
    /// `remove_map_object` 0x4AABB0), each level's index built again for them (TRAPS: "A level's
    /// index holds for the objects it was built for"), or taken back from the last other set.
    fn taken(&mut self, gone: &Arc<Vec<usize>>) -> Result<(), String> {
        if self.gone == *gone {
            return Ok(());
        }
        for (i, object) in self.live.objects.iter_mut().enumerate() {
            if self.gone.contains(&i) {
                object.standing = true;
            }
            if gone.contains(&i) {
                object.standing = false;
            }
        }
        let index = match self.spare.take() {
            Some((spare, index)) if spare == *gone => index,
            _ => {
                let assets = self.assets;
                (0..self.art.levels)
                    .map(|z| {
                        let z = u8::try_from(z).map_err(|_| format!("level {z}"))?;
                        TileIndex::build(
                            &self.art,
                            &self.live,
                            assets.tables,
                            assets.layout,
                            &self.shapes,
                            z,
                            assets.bugs,
                        )
                        .map_err(|e| e.to_string())
                    })
                    .collect::<Result<Vec<_>, String>>()?
            }
        };
        let old = core::mem::replace(&mut self.index, index);
        self.spare = Some((core::mem::replace(&mut self.gone, Arc::clone(gone)), old));
        Ok(())
    }

    /// A dissolve's screen (E24 M4 T2a2 part B, design §5.3): the view of the draw before, its
    /// rectangle `step` eighths of the way to `drawn`'s (`h3_render::Screen::dissolve_rect`).
    fn dissolved(
        &mut self,
        drawn: &Draw,
        dissolve: &Dissolve,
        buttons: Buttons,
    ) -> Result<Screen, String> {
        let mut screen = self.view(&dissolve.before, buttons)?.clone();
        let after = Draw {
            dissolve: None,
            ..drawn.clone()
        };
        let now = self.view(&after, buttons)?;
        screen.dissolve_rect(now, dissolve.rect, dissolve.step);
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

    /// The screen a draw, the buttons and a cursor show: the view and the panels, the cursor over
    /// them.
    pub fn screen(
        &mut self,
        drawn: &Draw,
        buttons: Buttons,
        cursor: Cursor,
    ) -> Result<Screen, String> {
        let mut screen = match &drawn.dissolve {
            Some(dissolve) => self.dissolved(drawn, dissolve, buttons)?,
            None => self.view(drawn, buttons)?.clone(),
        };
        let Cursor {
            kind,
            frame,
            at: [x, y],
            shown,
        } = cursor;
        if shown {
            let assets = self.assets;
            let cursor = cursor_in(
                &mut self.cursors,
                &self.batch,
                &assets.layout.cursor,
                [kind, frame],
            )?;
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
        let mut screen = match &drawn.dissolve {
            // A dissolve's (E24 M4 T2a2 part B): both draws composed afresh.
            Some(dissolve) => {
                let mut before = self.composed(&dissolve.before, buttons)?;
                let after = Draw {
                    dissolve: None,
                    ..drawn.clone()
                };
                let now = self.composed(&after, buttons)?;
                before.dissolve_rect(&now, dissolve.rect, dissolve.step);
                before
            }
            None => self.composed(drawn, buttons)?,
        };
        if cursor.shown {
            let assets = self.assets;
            let sprite = cursor_in(
                &mut self.cursors,
                &self.batch,
                &assets.layout.cursor,
                [cursor.kind, cursor.frame],
            )?;
            sprite.draw(&mut screen, cursor.at[0], cursor.at[1]);
        }
        Ok(screen)
    }

    /// A draw's view composed afresh, with no frame kept.
    fn composed(&mut self, drawn: &Draw, buttons: Buttons) -> Result<Screen, String> {
        let scene = self.scene(drawn, buttons)?;
        compose::compose(&scene, &self.batch, self.assets.layout.frame).map_err(|e| e.to_string())
    }

    /// [`Painter::screen`] widened into `out` (E24 M3 T2b1, T2b2, *provisional*): the view's
    /// frame widened by the table, only what changed since the frame `out` holds when the
    /// history still has it (the rectangles composed since, and its cursor's); then only what
    /// lies under the cursor cut out, drawn on and widened over it; no copy of the frame. A
    /// dissolve's step (E24 M4 T2a2 part B) widened whole, the frame then no composition's.
    pub fn frame_into(
        &mut self,
        drawn: &Draw,
        buttons: Buttons,
        cursor: Cursor,
        out: &mut Widened,
    ) -> Result<(), String> {
        if drawn.dissolve.is_some() {
            let screen = self.screen(drawn, buttons, cursor)?;
            self.widening.widen(&screen, &mut out.xrgb);
            out.held = None;
            return Ok(());
        }
        self.view(drawn, buttons)?;
        let screen = self.composer.screen();
        let now = self.history.now();
        self.rects.clear();
        let since = out.held.is_some_and(|held| {
            out.xrgb.len() == screen.pixels().len()
                && self.history.since(held, now, &mut self.rects)
        });
        if since {
            // Each pixel widened once: the compositions since repeat their strips (E24 M3 T7's
            // review, F2-7).
            drop_held(&mut self.rects);
            for rect in &self.rects {
                self.widening
                    .widen_rect(screen, rect.map(i64::from), &mut out.xrgb);
            }
        } else {
            self.widening.widen(screen, &mut out.xrgb);
        }
        out.held = Some(Held {
            composed: now,
            cursor: None,
        });
        let Cursor {
            kind,
            frame,
            at: [x, y],
            shown,
        } = cursor;
        if !shown {
            return Ok(());
        }
        let assets = self.assets;
        let sprite = cursor_in(
            &mut self.cursors,
            &self.batch,
            &assets.layout.cursor,
            [kind, frame],
        )?;
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
        self.history.changed(a, b, out)
    }

    /// The cursor's frame as an image (the hardware cursor's, E24 M2 T4b; since E24 M4 T1a by
    /// the mouse manager's type too).
    pub fn cursor_image(&mut self, kind: u32, frame: u32) -> Result<Image, String> {
        let assets = self.assets;
        let sprite = cursor_in(
            &mut self.cursors,
            &self.batch,
            &assets.layout.cursor,
            [kind, frame],
        )?;
        Ok(sprite.image(FORMAT))
    }
}

/// The layout's cursor's frame, read once into `cursors`: the adventure cursor's, or the default
/// one's (the mouse manager's type 0, a message box's: E24 M4 T1a).
fn cursor_in<'c>(
    cursors: &'c mut BTreeMap<[u32; 2], Sprite>,
    batch: &Batch,
    art: &CursorArt,
    [kind, frame]: [u32; 2],
) -> Result<&'c Sprite, String> {
    match cursors.entry([kind, frame]) {
        Entry::Occupied(known) => Ok(known.into_mut()),
        Entry::Vacant(new) => {
            let (def, [hx, hy]) = if kind == DEFAULT_CURSOR {
                (&art.default_def, art.default_hot)
            } else {
                let hot = usize::try_from(frame)
                    .ok()
                    .and_then(|f| art.hot.get(f).copied())
                    .unwrap_or_default();
                (&art.def, hot)
            };
            let hot = (
                u32::try_from(hx).unwrap_or(0),
                u32::try_from(hy).unwrap_or(0),
            );
            let sprite = batch
                .sprite(&def.to_ascii_uppercase())
                .and_then(|s| Sprite::read(s, frame, hot, FORMAT))
                .ok_or_else(|| format!("{def}: no frame {frame}"))?;
            Ok(new.insert(sprite))
        }
    }
}
