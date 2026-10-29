//! The live part of a map (E24 M1 T2, *provisional*: T4 and T5 settle what the drawing needs):
//! what changes in play and what the map file leaves to the game's start. Which objects stand,
//! what a random object became (the original draws them at the game's start, and the harness saw
//! the draws differ from run to run: E17 design §6.3), the owners, the heroes and the boats
//! (they have no draw lists, E24 design §2.3), the viewer and the tiles it has explored (§1.5),
//! the path shown (§2.10) and the moment (§1.6). In M1 a comparison fills it from the original's
//! memory (`h3mem.py`); [`Live::at_start`] gives what a map's start holds without a draw; from M4
//! on the engine fills it.

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use h3_map::RawMap;
use h3_map::model::{Body, Buildings, Pos};

use crate::art::{CELLS, MapArt, TemplateArt};
use crate::minimap::Shown;
use crate::tables::{self, TownArt};

/// The live part.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Live {
    /// One per map object, in the map's order ([`MapArt::objects`]).
    pub objects: Vec<LiveObject>,
    /// The heroes on the map.
    pub heroes: Vec<LiveHero>,
    /// The boats without a hero.
    pub boats: Vec<LiveBoat>,
    /// The viewing player's colour (0–7): the explored tiles' bit, the corner gems' frame, the
    /// frame picture's colours (E24 design §1.1, §1.5, §1.9).
    pub viewer: u8,
    /// The tiles the viewer has explored.
    pub explored: Explored,
    /// The path shown, its arrows by tile (E24 M1 T5b2, design §2.10); empty when none is shown.
    pub path: Vec<Arrow>,
    /// The moment drawn.
    pub moment: Moment,
    /// The walk's part of the draw (E24 M2 T3b, design §3.5, §3.8): the view's pixel offset and
    /// the selected hero lifted off its tile; none in a still (`Walking::default`).
    pub walking: Walking,
    /// What the adventure screen's panels show of the viewer's kingdom (E24 M3 T2a, design
    /// §4.1).
    pub kingdom: Kingdom,
    /// Move hero's and next hero's states as a timeline last set them (E24 M3 T2a: a walk
    /// changes them at its own moments, `h3_map_view::timeline`); none: as the kingdom and the
    /// path give them ([`Buttons::of`]).
    pub buttons: Option<Buttons>,
    /// The minimap as a timeline last drew it (E24 M3 T3b, design §4.4: a scroll, a hero's
    /// selection and each step's end draw it again, a step's frames do not); none: at the view
    /// and with the heroes where they stand (a still).
    pub minimap: Option<Shown>,
}

/// What the adventure screen's panels show of the viewing player's kingdom (E24 M3 T2a, design
/// §4.1; *provisional*: T4a added the lists' contents, T5a the resources and the date): its
/// heroes and towns in their lists' order, the one selected, the lists' first slots, whether its
/// quest log holds a quest, its resources and the game's date. A comparison reads them from the
/// original's memory (`h3mem.py`); [`Live::at_start`] gives a map's start; from M4 on the engine.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Kingdom {
    /// The heroes, in the hero list's order (the player's +0x8).
    pub heroes: Vec<KingdomHero>,
    /// The towns, in the town list's order (the player's +0x40, +0x3E of them).
    pub towns: Vec<KingdomTown>,
    /// The hero or town selected (the player's +0x4, +0x3F).
    pub selected: Selected,
    /// The first hero and the first town the lists show (the dialog's +0x64, +0x68).
    pub tops: [usize; 2],
    /// A seer hut's or quest guard's quest the player visited (`quest_log_state` 0x4039B0).
    pub quests: bool,
    /// The amounts of the seven resources in the original's order, wood, mercury, ore, sulfur,
    /// crystal, gems and gold (the player's +0x9C; E24 M3 T5a, design §4.7).
    pub resources: [i32; 7],
    /// The game's date (E24 M3 T5a), which the resource bar shows beside the amounts.
    pub date: Date,
}

/// The game's date (E24 M3 T5a, design §4.7): the month, the week of the month and the day of the
/// week, each from 1 (the game's +0x1F642, +0x1F640, +0x1F63E); a map's start the first of each.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Date {
    /// The month.
    pub month: u16,
    /// The week of the month.
    pub week: u16,
    /// The day of the week.
    pub day: u16,
}

impl Default for Date {
    fn default() -> Self {
        Self {
            month: 1,
            week: 1,
            day: 1,
        }
    }
}

/// A hero of the kingdom's.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct KingdomHero {
    /// Asleep (its +0x11C).
    pub asleep: bool,
    /// It can still move today: its points at least the cheapest step from its tile
    /// (`hero_can_move` 0x4E5F30).
    pub movable: bool,
    /// Its portrait (+0x34), an index into the tables' portraits (E24 M3 T4a).
    pub portrait: u8,
    /// Its movement points left (+0x4D).
    pub points: i32,
    /// Its mana (+0x18).
    pub mana: i32,
}

/// A town of the kingdom's, as the town list shows it (E24 M3 T4a, design §4.5).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct KingdomTown {
    /// Its faction (+0x4: 0 Castle … 8 Conflux).
    pub faction: u8,
    /// It has a Fort (or a Citadel or Castle: its every-built set +0x158 keeps the Fort's bit).
    pub fort: bool,
    /// A building was built in it today (+0x2).
    pub built: bool,
}

/// What the player has selected.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Selected {
    /// Nothing.
    #[default]
    None,
    /// The hero at this place of the hero list.
    Hero(usize),
    /// The town at this place of the town list.
    Town(usize),
}

/// The two buttons a walk changes (design §4.1): move hero and next hero, enabled or not.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Buttons {
    /// Move hero (0x4191A2, 0x407E2C).
    pub move_hero: bool,
    /// Next hero (`next_hero_button_state` 0x419450).
    pub next_hero: bool,
}

impl Kingdom {
    /// Whether a hero is awake and can still move (`next_hero_exists` 0x4BAB90).
    #[must_use]
    pub fn next_hero(&self) -> bool {
        self.heroes.iter().any(|h| h.movable && !h.asleep)
    }

    /// A live part's `kingdom` as the harness writes it (`h3mem.py`'s `Mem.kingdom`): `{ heroes =
    /// [[asleep, movable, portrait, points, mana], ...], towns = [[faction, fort, built], ...],
    /// selected = [<0 none, 1 a hero, 2 a town>, <its place in its list>], tops = [<heroes>,
    /// <towns>], quests = <0 or 1>, resources = [<7 amounts>], date = [<month>, <week>, <day>]
    /// }`, each flag 0 or 1; none for anything else (a selection past its list among them).
    #[must_use]
    pub fn read(value: &toml::Value) -> Option<Self> {
        fn pair(v: &toml::Value) -> Option<[&toml::Value; 2]> {
            match v.as_array()?.as_slice() {
                [a, b] => Some([a, b]),
                _ => None,
            }
        }
        let table = value.as_table()?;
        let number = |v: &toml::Value| v.as_integer().and_then(|n| usize::try_from(n).ok());
        let int = |v: &toml::Value| v.as_integer().and_then(|n| i32::try_from(n).ok());
        let byte = |v: &toml::Value| v.as_integer().and_then(|n| u8::try_from(n).ok());
        let flag = |v: &toml::Value| match v.as_integer()? {
            0 => Some(false),
            1 => Some(true),
            _ => None,
        };
        let heroes = table
            .get("heroes")?
            .as_array()?
            .iter()
            .map(|h| match h.as_array()?.as_slice() {
                [asleep, movable, portrait, points, mana] => Some(KingdomHero {
                    asleep: flag(asleep)?,
                    movable: flag(movable)?,
                    portrait: byte(portrait)?,
                    points: int(points)?,
                    mana: int(mana)?,
                }),
                _ => None,
            })
            .collect::<Option<Vec<_>>>()?;
        let towns = table
            .get("towns")?
            .as_array()?
            .iter()
            .map(|t| match t.as_array()?.as_slice() {
                [faction, fort, built] => Some(KingdomTown {
                    faction: byte(faction)?,
                    fort: flag(fort)?,
                    built: flag(built)?,
                }),
                _ => None,
            })
            .collect::<Option<Vec<_>>>()?;
        let [kind, at] = pair(table.get("selected")?)?;
        let at = number(at)?;
        let selected = match number(kind)? {
            0 => Selected::None,
            1 if at < heroes.len() => Selected::Hero(at),
            2 if at < towns.len() => Selected::Town(at),
            _ => return None,
        };
        let [heroes_top, towns_top] = pair(table.get("tops")?)?;
        let resources = table
            .get("resources")?
            .as_array()?
            .iter()
            .map(int)
            .collect::<Option<Vec<_>>>()?;
        let word = |v: &toml::Value| v.as_integer().and_then(|n| u16::try_from(n).ok());
        let date = match table.get("date")?.as_array()?.as_slice() {
            [month, week, day] => Date {
                month: word(month)?,
                week: word(week)?,
                day: word(day)?,
            },
            _ => return None,
        };
        Some(Self {
            heroes,
            towns,
            selected,
            tops: [number(heroes_top)?, number(towns_top)?],
            quests: flag(table.get("quests")?)?,
            resources: resources.try_into().ok()?,
            date,
        })
    }
}

impl Buttons {
    /// As a still shows them: move hero with a hero selected and its path holding a green arrow
    /// (a step it pays today: `make_hero_path` 0x418DD0 at 0x4191A2), next hero with a hero awake
    /// and able to move.
    #[must_use]
    pub fn of(kingdom: &Kingdom, path: &[Arrow]) -> Self {
        let selected = matches!(kingdom.selected, Selected::Hero(_));
        Self {
            move_hero: selected && path.iter().any(|a| (1..=GREEN).contains(&a.arrow)),
            next_hero: kingdom.next_hero(),
        }
    }
}

/// The last green arrow (1 to 25 a step the hero pays today, 26 to 50 one it does not: design
/// §2.10).
pub const GREEN: u16 = 25;

/// The walk's part of a draw (the adventure manager's +0xF4, +0xF8, +0x1EC, +0x1FC, +0x200;
/// design §3.5): what the timeline's draws give (`h3_map_view::timeline::Draw`), from M4 the
/// engine's walk.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Walking {
    /// The view's pixel offset, x and y: every pass adds it to every tile's place (a walk's frames
    /// slide the map under the lifted hero); the lifted hero's places and the corner gems keep
    /// theirs.
    pub offset: [i32; 2],
    /// The hero lifted off its tile, drawn at the view's middle (passes 5, 7 and 9); none while
    /// every hero stands on its tile.
    pub lifted: Option<Lifted>,
}

/// The hero lifted off its tile (+0x1EC; design §3.5): the selected hero of a human player from
/// its first step until a scroll puts it back.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Lifted {
    /// The hero, by its index in [`Live::heroes`] (its tile is the view's middle, its facing
    /// mirrors its pictures).
    pub hero: usize,
    /// Its group (+0x1FC): a walking one (5 to 9) during a step, its standing one (0 to 4) else.
    pub group: u32,
    /// Its frame in the group (+0x200), counted on by every draw of a step.
    pub frame: u32,
}

/// One arrow of the path shown (E24 M1 T5b2; design §2.2, §2.10): the adventure manager's
/// +0x48 by tile while its +0x4C shows a path, the selected hero's to the tile clicked. Passes 6
/// and 8 draw frame `arrow` − 1 of the tables' `path` DEF: its shadow under the bodies, its
/// pixels over them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Arrow {
    /// The tile.
    pub at: Pos,
    /// The arrow (1–50; 1 to 25 green, a tile the hero reaches this turn, 26 to 50 red).
    pub arrow: u16,
}

/// One object's live state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveObject {
    /// Still on the map (a picked-up pile or a beaten monster is not).
    pub standing: bool,
    /// What a random object became (`None`: not random, or not drawn yet).
    pub drawn: Option<Drawn>,
    /// The owner's colour (0–7), `None` for no owner.
    pub owner: Option<u8>,
    /// The object's animation phase (the 12-byte object record's +0xA, random at load; §1.6).
    pub phase: u8,
}

/// What a random object became (E24 M1 T4b; design §2.7). A new game copies the object's
/// template and gives the copy this DEF, class and subclass (0x4C9990), keeping the template's
/// size, masks, passability and priority; the object keeps the entries it was placed with. The
/// original draws `def`'s frames with the random template's masks, and names an artifact's DEF by
/// its id, which from 128 on is the picture of the artifact before it (MCL-Q3): the client draws
/// `own` with its own masks, the comparisons the original's (`h3_render::Bugs::Original`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Drawn {
    /// The DEF the copy names.
    pub def: String,
    /// The DEF of the game's template for the class and subclass it became (`Objects.txt`'s last
    /// such, as 0x505EA0 finds a monster's), if the game has one.
    pub own: Option<String>,
    /// The class it became (an artifact 5, a monster 54, a resource pile 79, a town 98, …).
    pub class: u32,
    /// Its subclass.
    pub subclass: u32,
}

/// A hero on the map.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LiveHero {
    /// The tile it stands on (its template's visitable cell, not the anchor).
    pub at: Pos,
    /// Its class (0–17: its map DEF `ah00_` … `ah17_`; the hero record's +0x30, design §2.3).
    pub class: u8,
    /// Its owner's colour (0–7): its flag.
    pub owner: u8,
    /// Where it faces.
    pub facing: Facing,
    /// The boat it sails, if it is in one.
    pub boat: Option<Boat>,
}

/// A boat's pictures (design §2.3: the waves, the flag, the boat by its type).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Boat {
    /// Its type (0–2: `ab01_` … `ab03_`).
    pub kind: u8,
    /// Its flag's colour (0–7).
    pub colour: u8,
}

/// A boat without a hero.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LiveBoat {
    /// The tile it lies on.
    pub at: Pos,
    /// Its type (0–2).
    pub kind: u8,
    /// Where it faces.
    pub facing: Facing,
}

/// Where a hero or boat faces, in the original's numbering (the hero record's +0x47, the boat
/// record's +0x1B; design §2.3, §2.8): each draws its group of the tables' `groups` (0x4D9110,
/// 0x4D9160: up 0, up-right 1, right 2, down-right 3, down 4, the left ones those of the right
/// ones), mirrored above 4.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Facing {
    /// 0.
    Up,
    /// 1.
    UpRight,
    /// 2.
    Right,
    /// 3.
    DownRight,
    /// 4.
    Down,
    /// 5.
    DownLeft,
    /// 6.
    Left,
    /// 7.
    UpLeft,
}

impl Facing {
    /// Every facing, in the original's order.
    pub const ALL: [Self; 8] = [
        Self::Up,
        Self::UpRight,
        Self::Right,
        Self::DownRight,
        Self::Down,
        Self::DownLeft,
        Self::Left,
        Self::UpLeft,
    ];

    /// The original's number.
    #[must_use]
    pub fn number(self) -> u8 {
        self as u8
    }

    /// The facing of the original's number, if it is one.
    #[must_use]
    pub fn from_number(number: u8) -> Option<Self> {
        Self::ALL.get(usize::from(number)).copied()
    }

    /// Drawn mirrored (the number above 4).
    #[must_use]
    pub fn mirrored(self) -> bool {
        self.number() > 4
    }
}

/// The tiles the viewer has explored (its bit of each tile's vision word, design §1.5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Explored {
    size: usize,
    levels: usize,
    tiles: Vec<bool>,
}

impl Explored {
    /// Every tile of a map of `size` and `levels` explored, or none.
    #[must_use]
    pub fn all(size: usize, levels: usize, explored: bool) -> Self {
        Self {
            size,
            levels,
            tiles: vec![explored; size.saturating_mul(size).saturating_mul(levels)],
        }
    }

    /// The map's size and levels it is of.
    #[must_use]
    pub fn size(&self) -> (usize, usize) {
        (self.size, self.levels)
    }

    fn index(&self, x: i32, y: i32, z: u8) -> Option<usize> {
        let (x, y) = (usize::try_from(x).ok()?, usize::try_from(y).ok()?);
        let z = usize::from(z);
        (x < self.size && y < self.size && z < self.levels)
            .then(|| (z * self.size + y) * self.size + x)
    }

    /// Whether the viewer has explored (x, y, z); a tile off the map has not been (the
    /// unexplored tiles' mask counts it so, design §1.5).
    #[must_use]
    pub fn explored(&self, x: i32, y: i32, z: u8) -> bool {
        self.index(x, y, z)
            .and_then(|i| self.tiles.get(i).copied())
            .unwrap_or(false)
    }

    /// Marks (x, y, z) explored or not; a tile off the map is left alone.
    pub fn set(&mut self, x: i32, y: i32, z: u8, explored: bool) {
        if let Some(tile) = self.index(x, y, z).and_then(|i| self.tiles.get_mut(i)) {
            *tile = explored;
        }
    }
}

/// The moment a still shows (design §1.6).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Moment {
    /// The palette cycles' steps since the terrains' DEFs were loaded (k: one step rotates every
    /// animated terrain's and river's ranges by one).
    pub cycles: u32,
    /// The rivers' palette steps: the terrains' on a clock of the viewer's; in the original one
    /// fewer when a tick came between the terrains' load and the rivers' (the load's progress
    /// bar runs the tick, design §1.6; E24 M1 T6a), for the whole game.
    pub river_cycles: u32,
    /// The objects' frame steps at the draw (the manager's +0x100): an object's frame is its
    /// phase plus this, modulo its group's frames; a hero's this alone.
    pub frames: u32,
}

impl Moment {
    /// The moment `ms` after the DEFs were loaded on a clock that never runs late: a cycle step
    /// every `cycle_ms`, a frame step every `frame_ms` (the tables' `[map]`; a step of 0 never
    /// comes). The original polls both and catches no late step up (design §1.6), so its moment
    /// at a time is at most this; a comparison reads the original's from memory (M1 T3b2).
    #[must_use]
    pub fn at(ms: u32, cycle_ms: u32, frame_ms: u32) -> Self {
        let cycles = ms.checked_div(cycle_ms).unwrap_or(0);
        Self {
            cycles,
            river_cycles: cycles,
            frames: ms.checked_div(frame_ms).unwrap_or(0),
        }
    }
}

/// The hero objects' class (34; design §2.1).
pub const HERO_CLASS: u32 = 34;
/// The boats' class (8).
pub const BOAT_CLASS: u32 = 8;
/// The towns' class (98; a random town is 77).
pub const TOWN_CLASS: u32 = 98;

impl Live {
    /// The live part at a map's start, as far as it needs no draw: every object standing and
    /// undrawn, phase 0 (the original's are random: a comparison reads them), the owners the map
    /// gives (a colour above 7, `0xFF` among them, is no owner); the heroes placed with an owner
    /// and a class (a random hero, class 70, needs a draw and is left out), facing right, on land
    /// (as the original's new game has them, a hero on water too: E24 M1 T5a, design §2.8); the
    /// boats of the map, facing right; every town (not a random one) with its picture by its
    /// buildings at the start (`towns`; E24 M1 T5b1, design §2.9: a new game redraws every town
    /// so, as a copy of its template); every tile explored (*provisional*: what a start reveals
    /// is the engine's, a comparison reads it); no path shown; the moment 0; no walk.
    #[must_use]
    pub fn at_start(raw: &RawMap, art: &MapArt, viewer: u8, towns: &TownArt) -> Self {
        let mut heroes = Vec::new();
        let mut boats = Vec::new();
        // The viewer's heroes' portraits and towns, in the map's order (the kingdom below).
        let mut portraits = Vec::new();
        let mut towns_kept = Vec::new();
        let mut objects = Vec::with_capacity(art.objects.len());
        for (object, raw_object) in art.objects.iter().zip(&raw.objects) {
            let template = art.templates.get(object.template);
            let at = template.and_then(|t| entrance(object.anchor, t));
            let class = template.map(|t| (t.class, t.subclass));
            match (class, at, &raw_object.body) {
                (Some((HERO_CLASS, subclass)), Some(at), Body::Hero(hero)) => {
                    if let (Ok(class), Some(owner)) = (u8::try_from(subclass), colour(hero.owner)) {
                        if owner == viewer {
                            portraits.push(hero.portrait.unwrap_or(hero.hero_type));
                        }
                        heroes.push(LiveHero {
                            at,
                            class,
                            owner,
                            facing: Facing::Right,
                            boat: None,
                        });
                    }
                }
                (Some((BOAT_CLASS, subclass)), Some(at), _) => {
                    if let Ok(kind) = u8::try_from(subclass) {
                        boats.push(LiveBoat {
                            at,
                            kind,
                            facing: Facing::Right,
                        });
                    }
                }
                _ => {}
            }
            if let (Some(t), Body::Town(town)) = (template, &raw_object.body)
                && t.class == TOWN_CLASS
                && colour(town.owner) == Some(viewer)
            {
                let built = built_at_start(&town.buildings);
                towns_kept.push(KingdomTown {
                    faction: u8::try_from(t.subclass).unwrap_or(u8::MAX),
                    fort: [tables::FORT, tables::CITADEL, tables::CASTLE]
                        .iter()
                        .any(|&id| built >> id & 1 == 1),
                    built: false,
                });
            }
            let drawn = match (template, &raw_object.body) {
                (Some(t), Body::Town(town)) if t.class == TOWN_CLASS => towns
                    .picture(t.subclass, built_at_start(&town.buildings))
                    .filter(|def| !def.eq_ignore_ascii_case(&t.def))
                    .map(|def| Drawn {
                        def: def.into(),
                        own: None,
                        class: TOWN_CLASS,
                        subclass: t.subclass,
                    }),
                _ => None,
            };
            objects.push(LiveObject {
                standing: true,
                drawn,
                owner: owner(&raw_object.body),
                phase: 0,
            });
        }
        // The viewer's kingdom (*provisional*, the engine's from M4): its heroes in the map's
        // order, awake and able to move, each with its portrait (the map's or its type's; a random
        // hero's type 0xFF until the engine draws it) and no points or mana known; its towns in
        // the map's order, none built in today; the first hero selected, else the first town; no
        // resources (the difficulty's starting ones are the engine's rule) on the first day.
        let selected = match (portraits.len(), towns_kept.len()) {
            (0, 0) => Selected::None,
            (0, _) => Selected::Town(0),
            _ => Selected::Hero(0),
        };
        let kingdom = Kingdom {
            heroes: portraits
                .into_iter()
                .map(|portrait| KingdomHero {
                    asleep: false,
                    movable: true,
                    portrait,
                    points: 0,
                    mana: 0,
                })
                .collect(),
            towns: towns_kept,
            selected,
            tops: [0, 0],
            quests: false,
            resources: [0; 7],
            date: Date::default(),
        };
        Self {
            objects,
            heroes,
            boats,
            viewer,
            explored: Explored::all(art.size, art.levels, true),
            path: Vec::new(),
            moment: Moment::default(),
            walking: Walking::default(),
            kingdom,
            buttons: None,
            minimap: None,
        }
    }
}

/// The tile of a template's first visitable cell (cells by row from the anchor's, each row from
/// the anchor's column leftwards), if it has one on the map's coordinates.
fn entrance(anchor: Pos, template: &TemplateArt) -> Option<Pos> {
    (0..CELLS.1)
        .flat_map(|dy| (0..CELLS.0).map(move |dx| (dx, dy)))
        .find(|&(dx, dy)| template.visitable(dx, dy))
        .and_then(|(dx, dy)| {
            Some(Pos {
                x: anchor.x.checked_sub(dx)?,
                y: anchor.y.checked_sub(dy)?,
                z: anchor.z,
            })
        })
}

/// The buildings of a town's picture it has at a new game, by the game's ids (bit `i` building
/// `i`; design §2.9): the standard set's Fort if it has one (0x5C0C0F); the chosen ones' Capitol,
/// Fort, Citadel and Castle, the map's buildings 2 to 5 (the map-to-game table 0x6888C0, the same
/// for every faction).
fn built_at_start(buildings: &Buildings) -> u64 {
    let bit = |id: u32| 1u64 << id;
    match buildings {
        Buildings::Standard { fort } => u64::from(*fort) * bit(tables::FORT),
        Buildings::Custom { built, .. } => [
            (2, tables::CAPITOL),
            (3, tables::FORT),
            (4, tables::CITADEL),
            (5, tables::CASTLE),
        ]
        .into_iter()
        .filter(|&(map, _)| built[0] >> map & 1 == 1)
        .fold(0, |set, (_, id)| set | bit(id)),
    }
}

/// An owner as the map stores it: 0–7 a colour, anything else none.
fn colour(owner: impl Into<u32>) -> Option<u8> {
    u8::try_from(owner.into()).ok().filter(|&c| c < 8)
}

/// The owner a map object's body gives.
fn owner(body: &Body) -> Option<u8> {
    match body {
        Body::Town(town) => colour(town.owner),
        Body::Hero(hero) => colour(hero.owner),
        Body::Mine { owner } | Body::Owned { owner } | Body::Garrison { owner, .. } => {
            colour(*owner)
        }
        Body::RandomDwelling(dwelling) => colour(dwelling.owner),
        Body::HeroPlaceholder { owner, .. } => colour(*owner),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
