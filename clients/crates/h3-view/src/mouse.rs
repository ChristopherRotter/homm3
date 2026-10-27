//! The mouse over the battle as a still (E17 M3 T6c-1; `docs/epics/E17-design.md` §9.9): where the
//! mouse is (`screen_to_hex` 0x4647A0), the cursor's kind and frame (`battle_cursor` 0x476490, the
//! attack's slots 0x474690 and 0x474A00), the status line's hint (`battle_status_line` 0x492840,
//! a control's `SetHints`) and the stack the mouse marks (`battle_mouse_stack` 0x477C20).
//!
//! The rules' answers (where the acting stack may move, whom it may strike from where, shoot or
//! heal) are the legal commands the server lists ([`Legal`]); what the executable's cursor decides
//! on its own (the slot inside the hex, the broken arrow's distance from the shooter's facing) is
//! ported here. Over time (E17 M3 T6c-2, design §9.10: the hint against the log's new lines, a
//! stack marked starting its fidget, a target left over from the last hex, the turns): [`track`].
//! A siege's (E17 M4 T2c-2, design §10.2): the keep's and the upper tower's rectangles, the
//! Catapult's targets (16), the turrets' info (22) and the walls' broken arrow (15). The command a
//! press gives (E17 M5 T2a, design §9.11): [`click`] over the battlefield (`battle_click_action`
//! 0x476BD0), [`button`] for the panel's Wait and Defend.

use alloc::collections::BTreeSet;
use alloc::string::String;

use h3_protocol::wire::{Command, Observation, Side, Siege, Stack};
use h3_render::Bugs;

use crate::layout::BattleLayout;
use crate::scene::Draw;
use crate::siege::{
    FIELD_HEXES, GATE_HEX, KEEP_TARGET, TARGET_AIMS, TARGET_PARTS, TOWER_TARGETS, WALL_HEXES,
};
use crate::text::{Args, Catalogue};

pub mod line;
pub mod track;

/// The battlefield's columns.
pub const COLUMNS: u8 = 17;
/// Its rows.
pub const ROWS: u8 = 11;
/// Its hexes.
const HEXES: i32 = 187;
/// The distance beyond which a shot is halved (`shot_full_damage_distance` 0x467600).
const FULL_DAMAGE_DISTANCE: i32 = 10;
/// The fort levels from which the upper tower's rectangle and the lower tower's hex (a Castle)
/// and the keep's rectangle (a Citadel) are targets (`battle_cursor` 0x476579, 0x4765E5).
const CASTLE: u8 = 3;
const CITADEL: u8 = 2;
/// The original ids of the shooters whose shots the walls never halve (`shot_wall_test`
/// 0x46753D): the Mage, the Arch Mage, the Enchanter, the Sharpshooter, the arrow tower.
pub const WALL_EXEMPT: [u16; 5] = [34, 35, 136, 137, 149];
/// The Catapult's original id: over a stack it aims as over an empty hex (0x4766B2).
pub const CATAPULT: u16 = 145;

/// Where the mouse is over the battlefield (`screen_to_hex`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Spot {
    /// A hex.
    Hex(u8),
    /// A side's hero's rectangle ("hexes" 0xFC and 0xFD).
    Hero(Side),
    /// The keep's rectangle ("hex" 0xFE; E17 M4 T2c-2).
    Keep,
    /// The upper tower's rectangle ("hex" 0xFF).
    Tower,
    /// Nothing.
    Nothing,
}

/// Where the point `[x, y]` of the screen is on the battlefield (`screen_to_hex` 0x4647A0): a
/// hero's rectangle, else with `rects` the keep's or the upper tower's (E17 M4 T2c-2: set in
/// every battle, 0x462700 and 0x462730; the client takes them in a siege only, the original's
/// everywhere for the comparisons, CLT-Q21), else the hex whose box holds it, a point above a
/// box's slanted top edge counting for the row above.
#[must_use]
pub fn spot(layout: &BattleLayout, [x, y]: [i32; 2], rects: bool) -> Spot {
    let mouse = &layout.mouse;
    let within = |[left, top, right, bottom]: [i32; 4]| {
        (left..=right).contains(&x) && (top..=bottom).contains(&y)
    };
    for (side, rect) in [Side::Attacker, Side::Defender]
        .into_iter()
        .zip(mouse.heroes)
    {
        if within(rect) {
            return Spot::Hero(side);
        }
    }
    if rects {
        if within(mouse.keep) {
            return Spot::Keep;
        }
        if within(mouse.tower) {
            return Spot::Tower;
        }
    }
    let [w, h] = layout.hex_step;
    let mut sx = x - layout.hex_origin[0];
    let sy = y - layout.hex_origin[1];
    let mut row = sy / h;
    if row & 1 == 0 {
        sx -= layout.even_row_shift;
    }
    let mut column = sx / w;
    let width = w * i32::from(COLUMNS);
    let height = h * i32::from(ROWS) + mouse.slant;
    if !(0..width).contains(&sx) || !(0..height).contains(&sy) {
        return Spot::Nothing;
    }
    let (rx, ry) = (sx % w, sy % h);
    if ry < mouse.slant && ry < (rx - w / 2).abs() / 2 {
        row -= 1;
        if rx < w / 2 {
            if row & 1 == 0 {
                column -= 1;
            }
        } else if row & 1 == 1 {
            column += 1;
        }
    }
    if (0..i32::from(ROWS)).contains(&row) && (0..i32::from(COLUMNS)).contains(&column) {
        u8::try_from(row * i32::from(COLUMNS) + column).map_or(Spot::Nothing, Spot::Hex)
    } else {
        Spot::Nothing
    }
}

/// The hex next to `hex` in `direction` (0 up right, 1 right, 2 down right, 3 down left, 4 left,
/// 5 up left), the manager's table at +0x13468 (built by 0x4640A0): none off the field, in a side
/// column, or from a side column towards its edge.
#[must_use]
pub fn neighbour(hex: u8, direction: u8) -> Option<u8> {
    let (row, column) = (hex / COLUMNS, hex % COLUMNS);
    let edge = COLUMNS - 1;
    if (column == 0 && direction >= 3) || (column == edge && direction <= 2) {
        return None;
    }
    let odd = row & 1 == 1;
    let h = i32::from(hex);
    let next = match (direction, odd) {
        (0, true) | (5, false) => h - 17,
        (0, false) => h - 16,
        (1, _) => h + 1,
        (2, true) | (3, false) => h + 17,
        (2, false) => h + 18,
        (3, true) => h + 16,
        (4, _) => h - 1,
        _ => h - 18,
    };
    let next = u8::try_from(next).ok().filter(|&n| i32::from(n) < HEXES)?;
    (next % COLUMNS != 0 && next % COLUMNS != edge).then_some(next)
}

/// The distance between two hexes (`hex_distance` 0x469670).
#[must_use]
pub fn distance(a: u8, b: u8) -> i32 {
    let (y1, x1) = (i32::from(a / COLUMNS), i32::from(a % COLUMNS));
    let (y2, x2) = (i32::from(b / COLUMNS), i32::from(b % COLUMNS));
    let p = (y1 + 1) / 2 - (y2 + 1) / 2 - x1 + x2;
    let q = y2 / 2 - y1 / 2 - x1 + x2;
    if (p < 0) == (q < 0) {
        p.abs().max(q.abs())
    } else {
        p.abs() + q.abs()
    }
}

/// The acting stack's legal commands as the cursor asks them, and what its own tests need to
/// know of the acting stack (E17 M4 T2c-2: whether it is the Catapult, whether the walls halve
/// its shots).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Legal {
    /// The positions it may walk or fly to.
    moves: BTreeSet<u8>,
    /// The stacks it may strike in melee, each with a position to strike from.
    attacks: BTreeSet<(u8, u8)>,
    /// The stacks it may shoot, each with a hex to aim at (an area shot's: either of a two-hex
    /// target's).
    shots: BTreeSet<(u8, u8)>,
    /// The stacks it may heal.
    heals: BTreeSet<u8>,
    /// The wall-shot targets it may shoot (a wall shooter of the attacker's: the targets
    /// standing).
    walls: BTreeSet<u8>,
    /// It may wait (E17 M5 T2a: the panel's button gives it).
    wait: bool,
    /// It may defend.
    defend: bool,
    /// It is the Catapult (its original id [`CATAPULT`]).
    pub catapult: bool,
    /// The walls never halve its shots (`shot_wall_test`'s shooters, [`WALL_EXEMPT`], or its
    /// hero's bow: the wire's `no_distance_penalty`).
    pub wall_exempt: bool,
}

impl Legal {
    /// The legal commands the server lists for the acting stack.
    #[must_use]
    pub fn new(commands: &[Command]) -> Self {
        let mut legal = Self::default();
        for command in commands {
            match *command {
                Command::Move { to } => {
                    legal.moves.insert(to);
                }
                Command::Attack { target, from } => {
                    legal.attacks.insert((target, from));
                }
                Command::Shoot { target, at } => {
                    legal.shots.insert((target, at));
                }
                Command::Heal { target } => {
                    legal.heals.insert(target);
                }
                Command::ShootWall { part } => {
                    legal.walls.insert(part);
                }
                Command::Wait {} => legal.wait = true,
                Command::Defend {} => legal.defend = true,
                _ => {}
            }
        }
        legal
    }

    /// Whether it may shoot `target`.
    fn shoots(&self, target: u8) -> bool {
        self.shots
            .range((target, 0)..=(target, u8::MAX))
            .next()
            .is_some()
    }

    /// The hex to aim a shot at `target` at with the mouse over `hex`: that hex where it is one
    /// (an area shot's), else the target's (E17 M5 T2a: the original's shot is at the mouse's hex,
    /// 0x476C7C).
    fn aim(&self, target: u8, hex: u8) -> Option<u8> {
        if self.shots.contains(&(target, hex)) {
            return Some(hex);
        }
        self.shots
            .range((target, 0)..=(target, u8::MAX))
            .next()
            .map(|&(_, at)| at)
    }
}

/// Which way the stacks face (+0x44; E17 M3 T6c-2): a two-hex stack's front hex and a shooter's
/// distance count from it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum Facing {
    /// Each side's own way: the attacker's to the right, the defender's to the left (the opening).
    #[default]
    Sides,
    /// The stacks facing left now (the timeline's poses: a stack turned in the battle).
    Left(BTreeSet<u8>),
}

impl Facing {
    /// Whether `stack` faces left.
    #[must_use]
    pub fn left(&self, stack: &Stack) -> bool {
        match self {
            Self::Sides => stack.side == Side::Defender,
            Self::Left(left) => left.contains(&stack.id),
        }
    }
}

/// The battle as the cursor asks it: the stacks, the acting stack's legal commands and which way
/// the stacks face; the profile (the siege's rectangles in every battle in the original's, the
/// comparisons': CLT-Q21).
#[derive(Debug, Clone, Copy)]
pub struct Field<'a> {
    /// The battle.
    pub observation: &'a Observation,
    /// The acting stack's legal commands.
    pub legal: &'a Legal,
    /// Which way the stacks face.
    pub facing: &'a Facing,
    /// The profile.
    pub bugs: Bugs,
}

impl Field<'_> {
    /// Whether the mouse finds the keep's and the upper tower's rectangles (CLT-Q21): in a
    /// siege, a town with a fort (E17 M4 T3a, R2-2: a town without one is no siege, as the
    /// drawing's); in the original's profile in every battle.
    fn rects(&self) -> bool {
        self.fort() > 0 || self.bugs == Bugs::Original
    }

    /// The siege's fort level (0 none: no siege, or no fort).
    fn fort(&self) -> u8 {
        self.observation.siege.as_ref().map_or(0, |s| s.fort)
    }

    /// The acting stack.
    fn acting(&self) -> Option<&Stack> {
        let id = self.observation.active?;
        self.observation.stacks.iter().find(|s| s.id == id)
    }

    /// A stack's hexes: its position and, two hexes wide, its front hex by its facing
    /// (`stack_other_hex` 0x4466A0).
    fn hexes(&self, stack: &Stack) -> [u8; 2] {
        let other = if !stack.two_hex {
            stack.hex
        } else if self.facing.left(stack) {
            stack.hex.saturating_sub(1)
        } else {
            stack.hex.saturating_add(1)
        };
        [stack.hex, other]
    }

    /// The living stack on `hex`.
    pub(crate) fn at(&self, hex: u8) -> Option<&Stack> {
        self.observation
            .stacks
            .iter()
            .find(|s| s.alive && self.hexes(s).contains(&hex))
    }
}

/// The cursor's kind (`battle_cursor`'s answer; its number is the combat cursor's frame but for
/// an attack, whose frame is its slot's, and the other side's hero, frame 5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// Nothing to do there (0).
    Nothing,
    /// Walk there (1).
    Move,
    /// Fly there (2).
    Fly,
    /// Shoot the stack (3).
    Shoot,
    /// The acting side's hero (4).
    Hero,
    /// A stack to look at (5).
    Query,
    /// The panel's pointer (6, `battle_buttons` 0x47551F).
    Pointer,
    /// Strike the stack (7).
    Attack,
    /// Shoot the stack at half damage (15).
    BrokenArrow,
    /// A wall-shot target for the Catapult or a Cyclops (16; E17 M4 T2c-2).
    Wall,
    /// Heal the stack with the First Aid Tent (17).
    Heal,
    /// The other side's hero (21).
    EnemyHero,
    /// A turret to look at (22, the query's frame; E17 M4 T2c-2).
    Turret,
}

impl Kind {
    /// The executable's number of the kind.
    #[must_use]
    pub const fn code(self) -> u32 {
        match self {
            Self::Nothing => 0,
            Self::Move => 1,
            Self::Fly => 2,
            Self::Shoot => 3,
            Self::Hero => 4,
            Self::Query => 5,
            Self::Pointer => 6,
            Self::Attack => 7,
            Self::BrokenArrow => 15,
            Self::Wall => 16,
            Self::Heal => 17,
            Self::EnemyHero => 21,
            Self::Turret => 22,
        }
    }
}

/// The status line's hint (`battle_status_line`'s text, or a control's).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Hint {
    /// Empty.
    Empty,
    /// "Move %s here." (GENRLTXT:295), the acting stack by its number.
    Move(u8),
    /// "Fly %s here." (296).
    Fly(u8),
    /// "Attack %s" (221), the target by its number: a strike's and, while the army info is off,
    /// a shot's.
    Attack(u8),
    /// "View %s info." (298), the stack's singular name.
    View(u8),
    /// "View Hero Stats" (418).
    Hero,
    /// "View Enemy Hero Stats" (419).
    EnemyHero,
    /// "Apply first aid to the %s" (420), the patient by its number.
    Heal(u8),
    /// "No shots left!" (299): over nothing, for a shooter without shots whose target is still
    /// set (E17 M3 T6c-2).
    NoShots,
    /// A control's hint: its key.
    Control(String),
    /// A hero's spell aimed (E17 M3 T7b-1, *provisional*: `enter_player_cast` 0x59EC50's loop
    /// is not read): "Cast %s on %s" (28) over a stack by its number, "Cast %s" (27) elsewhere on
    /// the battlefield; the spell by its handle.
    Cast(u16, Option<u8>),
    /// A wall-shot target (E17 M4 T2c-2): "Attack %s" (221) with the target's part's name for the
    /// town's type (WALLS.TXT, the parts' +0x1C, 0x492E70).
    Wall(u8),
    /// "View arrow tower info" (157; E17 M4 T2c-2).
    Turret,
}

impl Hint {
    /// Whether it replaces the status line over the log's new lines (`battle_log_add`'s third
    /// argument: an attack's or a shot's; design §9.8; a cast's not, as `arrow`'s frames show).
    #[must_use]
    pub const fn forced(&self) -> bool {
        matches!(self, Self::Attack(_))
    }

    /// The hint's text in `catalogue`'s words, the stacks of `observation` named by `creature`
    /// (a stack's creature key in the catalogue: `creature.base:pikeman`), a spell by `spell`
    /// (its key: `spell.base:slow`).
    #[must_use]
    pub fn text(
        &self,
        catalogue: &Catalogue,
        observation: &Observation,
        namers: Namers<'_>,
    ) -> String {
        let (creature, spell, part) = namers;
        let stack = |id: u8| observation.stacks.iter().find(|s| s.id == id);
        let name = |id: u8, single: bool| {
            stack(id).and_then(|s| {
                let key = creature(s)?;
                let count = if single { 1 } else { i64::from(s.count) };
                Some(catalogue.text(&key, &Args::new().count(count)))
            })
        };
        let with = |key: &str, id: u8, single: bool| {
            let creature = name(id, single).unwrap_or_default();
            catalogue.text(key, &Args::new().text("creature", &creature))
        };
        match self {
            Self::Empty => String::new(),
            Self::Move(id) => with("battle.hint.move", *id, false),
            Self::Fly(id) => with("battle.hint.fly", *id, false),
            Self::Attack(id) => with("battle.hint.attack", *id, false),
            Self::View(id) => with("battle.hint.view", *id, true),
            Self::Heal(id) => with("battle.hint.heal", *id, false),
            Self::Wall(t) => {
                let name = observation
                    .siege
                    .as_ref()
                    .zip(TARGET_PARTS.get(usize::from(*t)))
                    .and_then(|(s, &p)| part(s.faction, p))
                    .map_or_else(String::new, |key| catalogue.text(&key, &Args::new()));
                catalogue.text("battle.hint.attack", &Args::new().text("creature", &name))
            }
            Self::Turret => catalogue.text("battle.hint.turret", &Args::new()),
            Self::Hero => catalogue.text("battle.hint.hero", &Args::new()),
            Self::EnemyHero => catalogue.text("battle.hint.enemy-hero", &Args::new()),
            Self::NoShots => catalogue.text("battle.hint.no-shots", &Args::new()),
            Self::Control(key) => catalogue.text(key, &Args::new()),
            Self::Cast(handle, target) => {
                let spell = spell(*handle)
                    .map_or_else(String::new, |key| catalogue.text(&key, &Args::new()));
                match target {
                    Some(id) => {
                        let creature = name(*id, false).unwrap_or_default();
                        let args = Args::new()
                            .text("spell", &spell)
                            .text("creature", &creature);
                        catalogue.text("battle.hint.cast-on", &args)
                    }
                    None => {
                        let args = Args::new().text("spell", &spell);
                        catalogue.text("battle.hint.cast", &args)
                    }
                }
            }
        }
    }
}

/// What names a hint's stacks, spells and walls (E17 M3 T7b-1): a stack's creature key in the
/// catalogue (`creature.base:pikeman`), a spell's by its handle (`spell.base:slow`), a town's
/// wall part by the town's faction handle and the part's number (E17 M4 T2c-2:
/// `wall.base:castle.6`).
pub type Namers<'a> = (
    &'a dyn Fn(&Stack) -> Option<String>,
    &'a dyn Fn(u16) -> Option<String>,
    &'a dyn Fn(u16, usize) -> Option<String>,
);

/// The mouse at a point: the cursor, the status line's hint and the stack the mouse marks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pointer {
    /// The cursor's kind.
    pub kind: Kind,
    /// The combat cursor's frame.
    pub frame: u32,
    /// The status line's hint.
    pub hint: Hint,
    /// The stack the mouse marks (its outline 0x70, §4.2).
    pub marked: Option<u8>,
}

impl Pointer {
    /// The cursor at `point` ([`cursor`]).
    #[must_use]
    pub fn cursor(&self, layout: &BattleLayout, point: [i32; 2]) -> Draw {
        cursor(layout, self.frame, point)
    }
}

/// The combat cursor's `frame` at `point` (`SetCursor(frame, 2)`, 0x50CCA0): its hot spot
/// (0x50CEE0; none beyond the table), its canvas's corner the point less it.
#[must_use]
pub fn cursor(layout: &BattleLayout, frame: u32, [x, y]: [i32; 2]) -> Draw {
    let hot = usize::try_from(frame)
        .ok()
        .and_then(|f| layout.mouse.hot.get(f).copied())
        .unwrap_or_default();
    Draw::Cursor {
        def: layout.mouse.cursor.clone(),
        frame,
        x,
        y,
        hot,
    }
}

/// The mouse at `point` of the screen in the battle `observation` shows, its acting stack's
/// commands `legal` (`battle_buttons`' mouse move 0x4753A4): over the battlefield its spot's
/// kind, over the panel the pointer and the hint of the control under it. A still: each side
/// faces its own way and no target is left over ([`pointer_in`]); `bugs` the profile (CLT-Q21).
#[must_use]
pub fn pointer(
    observation: &Observation,
    legal: &Legal,
    layout: &BattleLayout,
    (point, bugs): ([i32; 2], Bugs),
) -> Pointer {
    let field = Field {
        observation,
        legal,
        facing: &Facing::Sides,
        bugs,
    };
    pointer_in(&field, layout, point, &mut None)
}

/// As [`pointer`] in `field`, the acting stack's target `aim` (+0x10, +0x14) as `battle_cursor`
/// leaves it: cleared over a hex of the field, set over an enemy the stack may shoot or strike,
/// kept over nothing, a side column or a hero (E17 M3 T6c-2); over nothing a shooter without
/// shots whose target is still set shows "No shots left!".
#[must_use]
pub fn pointer_in(
    field: &Field<'_>,
    layout: &BattleLayout,
    point: [i32; 2],
    aim: &mut Option<u8>,
) -> Pointer {
    let [width, height] = layout.mouse.field;
    if !(0..width).contains(&point[0]) || !(0..height).contains(&point[1]) {
        let hint = layout
            .panel
            .item_at(point)
            .and_then(crate::ui::Item::hint)
            .map_or(Hint::Empty, |key| Hint::Control(key.into()));
        return Pointer {
            kind: Kind::Pointer,
            frame: layout.mouse.panel_frame,
            hint,
            marked: None,
        };
    }
    let spot = spot(layout, point, field.rects());
    let marked = match spot {
        Spot::Hex(hex) => field.at(hex).map(|s| s.id),
        _ => None,
    };
    let (kind, target) = kind(field, spot, aim);
    let frame = match (kind, spot) {
        (Kind::Attack, Spot::Hex(hex)) => {
            let slot = slot(layout, point, hex);
            slots_in(field, hex)[slot]
        }
        (Kind::EnemyHero | Kind::Turret, _) => Kind::Query.code(),
        _ => kind.code(),
    };
    let acting = field.acting();
    let hint = match (kind, target, acting) {
        (Kind::Move, _, Some(s)) => Hint::Move(s.id),
        (Kind::Fly, _, Some(s)) => Hint::Fly(s.id),
        (Kind::Shoot | Kind::BrokenArrow | Kind::Attack, Some(id), _) => Hint::Attack(id),
        (Kind::Hero, ..) => Hint::Hero,
        (Kind::EnemyHero, ..) => Hint::EnemyHero,
        (Kind::Query, Some(id), _) => Hint::View(id),
        (Kind::Heal, Some(id), _) => Hint::Heal(id),
        (Kind::Wall, Some(t), _) => Hint::Wall(t),
        (Kind::Turret, ..) => Hint::Turret,
        (Kind::Nothing, _, Some(s)) if s.shooter && s.shots == 0 && aim.is_some() => Hint::NoShots,
        _ => Hint::Empty,
    };
    Pointer {
        kind,
        frame,
        hint,
        marked,
    }
}

/// The cursor's kind at `spot` and what it is about (`battle_cursor` 0x476490): a stack, or for
/// [`Kind::Wall`] the wall-shot target; the acting stack's target `aim` cleared on a hex of the
/// field (0x476686) and set over an enemy (0x47679D) until neither a shot nor a strike reaches it
/// (0x476837): the modelled cases (casters and network play are not: design §9.9). A siege's
/// (E17 M4 T2c-2): over the upper tower's rectangle with a Castle and the keep's with a Citadel
/// a wall shooter's target while it stands (16, its stack target cleared), else the turret's
/// info (22); over an empty hex (any for the Catapult) a standing target's aim hex, the towers'
/// with a Castle and the keep's with a Citadel (16); over hex 183 with a Castle where no move
/// goes, the lower tower's info (22).
fn kind(field: &Field<'_>, spot: Spot, aim: &mut Option<u8>) -> (Kind, Option<u8>) {
    let (observation, legal) = (field.observation, field.legal);
    let Some(acting) = field.acting() else {
        return (Kind::Nothing, None);
    };
    let side = acting.controller;
    let hex = match spot {
        Spot::Nothing => return (Kind::Nothing, None),
        Spot::Hero(hero) => {
            let index = usize::from(hero == Side::Defender);
            let kind = match observation.sides.get(index).and_then(|s| s.hero.as_ref()) {
                None => Kind::Nothing,
                Some(_) if hero == side => Kind::Hero,
                Some(_) => Kind::EnemyHero,
            };
            return (kind, None);
        }
        Spot::Keep | Spot::Tower => {
            let (target, fort) = if spot == Spot::Keep {
                (KEEP_TARGET, CITADEL)
            } else {
                (TOWER_TARGETS[0], CASTLE)
            };
            if field.fort() < fort {
                return (Kind::Nothing, None);
            }
            return match wall(legal, target) {
                Some(t) => {
                    *aim = None;
                    (Kind::Wall, Some(t))
                }
                None => (Kind::Turret, None),
            };
        }
        Spot::Hex(hex) => hex,
    };
    let column = hex % COLUMNS;
    if column == 0 || column == COLUMNS - 1 {
        return (Kind::Nothing, None);
    }
    *aim = None;
    let stack = field.at(hex).filter(|_| !legal.catapult);
    if let Some(stack) = stack {
        let id = Some(stack.id);
        if observation.tactics.is_some() || stack.id == acting.id {
            return (Kind::Query, id);
        }
        if stack.side == side {
            let kind = if legal.heals.contains(&stack.id) {
                Kind::Heal
            } else {
                Kind::Query
            };
            return (kind, id);
        }
        *aim = id;
        if legal.shoots(stack.id) {
            // The arrow tower shoots in full (0x4767B4); a stack off the field is a turret.
            let kind = if acting.hex < FIELD_HEXES && halved(field, acting, (stack, hex)) {
                Kind::BrokenArrow
            } else {
                Kind::Shoot
            };
            return (kind, id);
        }
        if legal.attacks.iter().any(|&(t, _)| t == stack.id) {
            return (Kind::Attack, id);
        }
        *aim = None;
        return (Kind::Nothing, None);
    }
    // A wall-shot target's aim hex (0x47687B), while the target stands: the lower tower's with a
    // Castle (the upper tower's and the keep's aims are their rectangles, tested at 0x4768BF).
    let aimed = TARGET_AIMS.iter().position(|&a| a == hex).filter(|&t| {
        let fort = if TOWER_TARGETS.contains(&t) {
            CASTLE
        } else {
            1
        };
        field.fort() >= fort
    });
    if let Some(t) = aimed.and_then(|t| wall(legal, t)) {
        return (Kind::Wall, Some(t));
    }
    // An empty hex the reach map marks (+0x20E a position, +0x20F a two-hex stack's front hex,
    // 0x4B2E69, 0x4B2EF4).
    let [position, front] = field.hexes(acting);
    let back = i32::from(hex) - (i32::from(front) - i32::from(position));
    let reached = legal.moves.contains(&hex)
        || (acting.two_hex && u8::try_from(back).is_ok_and(|b| legal.moves.contains(&b)));
    match (reached, acting.flies) {
        // The lower tower's hex with a Castle (0x476975).
        (false, _) if hex == TARGET_AIMS[TOWER_TARGETS[1]] && field.fort() >= CASTLE => {
            (Kind::Turret, None)
        }
        (false, _) => (Kind::Nothing, None),
        (true, true) => (Kind::Fly, None),
        (true, false) => (Kind::Move, None),
    }
}

/// The wall-shot target `t` (an index of [`TARGET_AIMS`]) when the acting stack may shoot it:
/// a wall shooter of the attacker's outside the tactics phase, the target standing (0x476586 to
/// 0x4765C7; the server lists it then).
fn wall(legal: &Legal, t: usize) -> Option<u8> {
    // `t` is one of the eight targets.
    let t = u8::try_from(t).unwrap_or(u8::MAX);
    legal.walls.contains(&t).then_some(t)
}

/// The cursor's own half-damage tests of a shot at `target` with the mouse over `hex`: the
/// siege's walls (`shot_wall_test` 0x467510 from 0x4767D6, E17 M4 T2c-2: [`walls_halve`] from the
/// shooter's position to the mouse's hex), then the distance (`shot_full_damage_distance`
/// 0x467600 from 0x4767E6): without the shooter turned to the target (its front hex by its
/// facing now), no shot of a shooter without the distance penalty is halved; else halved when
/// both of the target's hexes are farther than 10.
fn halved(field: &Field<'_>, shooter: &Stack, (target, hex): (&Stack, u8)) -> bool {
    let walls = field.observation.siege.as_ref();
    if walls.is_some_and(|s| !field.legal.wall_exempt && walls_halve(s, shooter.hex, hex)) {
        return true;
    }
    if shooter.no_distance_penalty {
        return false;
    }
    let from = field.hexes(shooter)[1];
    field
        .hexes(target)
        .iter()
        .all(|&h| distance(from, h) > FULL_DAMAGE_DISTANCE)
}

/// Whether the walls of `siege` halve a shot from `from` at `to` (`shot_wall_test` 0x467510 past
/// its shooter's tests; BAT-SIE-008): a fort, `from` left of its row's wall hex and `to` at it or
/// right of it ([`WALL_HEXES`]), and a wall on the straight line between them ([`line::blocked`]:
/// the gate's hex while the gate is closed, any other wall hex while it stands).
#[must_use]
pub fn walls_halve(siege: &Siege, from: u8, to: u8) -> bool {
    let wall = |hex: u8| WALL_HEXES.get(usize::from(hex / COLUMNS)).copied();
    let outside = wall(from).is_some_and(|w| from < w);
    let inside = wall(to).is_some_and(|w| to >= w);
    let blocks = |hex: u8| {
        if hex == GATE_HEX {
            !siege.gate_open
        } else {
            siege.walls.contains(&hex)
        }
    };
    siege.fort > 0 && outside && inside && line::blocked(from, to, &blocks)
}

/// The slot of the mouse inside the hex it is on (`cursor_attack_from_hex` 0x474A00): its angle
/// from the hex's centre, |dx| / |dy| against 0.58 and 1.73 (in integers: exact for the distances
/// a hex holds), twelve slots clockwise in four quarters.
#[must_use]
pub fn slot(layout: &BattleLayout, [x, y]: [i32; 2], hex: u8) -> usize {
    let [left, top] = layout.hex_box(hex / COLUMNS, hex % COLUMNS);
    let dx = x - (left + layout.mouse.centre[0]);
    let dy = y - (top + layout.mouse.centre[1]);
    let (ax, ay) = (dx.abs(), dy.abs());
    // r = |dx| / |dy| (100 when dy = 0) against 0.58 = 29 / 50 and 1.73 = 173 / 100.
    let above_low = ay == 0 || 50 * ax > 29 * ay;
    let above_high = ay == 0 || 100 * ax > 173 * ay;
    match (dx >= 0, dy >= 0) {
        (true, false) => usize::from(above_low) + usize::from(above_high),
        (true, true) => 3 + usize::from(!above_high) + usize::from(!above_low),
        (false, true) => 6 + usize::from(above_low) + usize::from(above_high),
        (false, false) => 9 + usize::from(!above_high) + usize::from(!above_low),
    }
}

/// The cursor's frame in each of the twelve slots of the acting stack's attack on the stack on
/// `hex` (`cursor_attack_slots` 0x474690; 0 for none): slot s takes the neighbour s / 2 as the
/// position to strike from when the attack from there is legal, a sword pointing at the hex
/// (7 + (s / 2 + 3) mod 6); a two-hex stack also tries the position beside it, the vertical
/// swords (13 from below, 14 from above) when that one is taken; a slot without one copies the
/// nearest slot's by the executable's measure.
#[must_use]
pub fn slots(observation: &Observation, legal: &Legal, hex: u8) -> [u32; 12] {
    let field = Field {
        observation,
        legal,
        facing: &Facing::Sides,
        bugs: Bugs::Fixed,
    };
    slots_in(&field, hex)
}

/// As [`slots`] in `field`: the acting stack's facing decides which position beside it a two-hex
/// stack tries (E17 M3 T6c-2).
#[must_use]
pub fn slots_in(field: &Field<'_>, hex: u8) -> [u32; 12] {
    attack_slots(field, hex).map(|s| s.frame)
}

/// A slot of the acting stack's attack on a stack (`cursor_attack_slots` 0x474690): the cursor's
/// frame (+0x133CC; 0 for none) and the position the attack is made from (+0x133FC; E17 M5 T2a).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Slot {
    /// The combat cursor's frame.
    pub frame: u32,
    /// The position to strike from.
    pub from: Option<u8>,
}

/// The twelve slots of the acting stack's attack on the stack on `hex` in `field` ([`slots`];
/// design §9.11): a slot's position is the neighbour it tries, a two-hex stack's the one beside it
/// for the directions right and left when that is taken, else the preferred one (its sword) or
/// the other (a vertical sword); a slot without one copies the nearest slot's frame and position
/// together.
#[must_use]
pub fn attack_slots(field: &Field<'_>, hex: u8) -> [Slot; 12] {
    let legal = field.legal;
    let mut slots = [Slot::default(); 12];
    let (Some(acting), Some(target)) = (field.acting(), field.at(hex)) else {
        return slots;
    };
    let ok = |p: i32| {
        u8::try_from(p)
            .ok()
            .filter(|&p| legal.attacks.contains(&(target.id, p)))
    };
    let right = !field.facing.left(acting);
    for (s, slot) in slots.iter_mut().enumerate() {
        let direction = u8::try_from(s / 2).unwrap_or(0);
        let Some(next) = neighbour(hex, direction) else {
            continue;
        };
        let next = i32::from(next);
        let first = ok(next);
        let sword = 7 + (u32::from(direction) + 3) % 6;
        if first.is_some() {
            *slot = Slot {
                frame: sword,
                from: first,
            };
        }
        if !acting.two_hex {
            continue;
        }
        let beside = next - if right { 1 } else { -1 };
        let second = ok(beside);
        if first.is_none() && second.is_none() {
            continue;
        }
        slot.frame = sword;
        if direction == 1 || direction == 4 {
            // Right and left: the position beside when it is taken (0x47494E).
            slot.from = second.or(first);
            continue;
        }
        // The position by facing and side first (0x4748E0): whether the beside one is preferred.
        let beside_first = !right == (direction <= 2);
        let (preferred, other) = if beside_first {
            (second, first)
        } else {
            (first, second)
        };
        let vertical = if direction == 2 || direction == 3 {
            13
        } else {
            14
        };
        *slot = if preferred.is_none() || (other.is_some() && matches!(s, 0 | 5 | 6 | 11)) {
            Slot {
                frame: vertical,
                from: other,
            }
        } else {
            Slot {
                frame: sword,
                from: preferred,
            }
        };
    }
    for s in 0..12 {
        if slots[s].frame != 0 {
            continue;
        }
        let mut best = 12;
        for j in 0..12 {
            if j == s {
                continue;
            }
            let mut d = i32::try_from(s.abs_diff(j)).unwrap_or(0);
            if d > 6 {
                d = 6 - d;
            }
            if slots[j].frame != 0 && d < best {
                best = d;
                slots[s] = slots[j];
            }
        }
    }
    slots
}

/// The panel's Wait button (`battle_buttons` 0x4751B8: action 8).
pub const WAIT: u16 = 2009;
/// The panel's Defend button (0x474EEE: action 3).
pub const DEFEND: u16 = 2010;

/// The command a left press at `point` gives in `field` (E17 M5 T2a; `battle_click_action`
/// 0x476BD0, design §9.11): over the battlefield by the cursor's kind there, a walk or a flight
/// to the hex (a two-hex stack's to the position beside it when the hex is that position's front
/// hex, 0x476C5F), a shot at the stack (its aim the hex where the target may be aimed at there),
/// a strike from the slot's position, a wall shot, a heal; none for the others (the stack's info,
/// the spell book: their dialogs) and outside the battlefield. As the original's, no press walks a
/// two-hex stack one hex back where it may walk two (CLT-Q25: copied, an option later).
#[must_use]
pub fn click(field: &Field<'_>, layout: &BattleLayout, point: [i32; 2]) -> Option<Command> {
    let [width, height] = layout.mouse.field;
    if !(0..width).contains(&point[0]) || !(0..height).contains(&point[1]) {
        return None;
    }
    let spot = spot(layout, point, field.rects());
    let (kind, target) = kind(field, spot, &mut None);
    let legal = field.legal;
    match (kind, spot) {
        (Kind::Move | Kind::Fly, Spot::Hex(hex)) => {
            let acting = field.acting()?;
            let [position, front] = field.hexes(acting);
            let back = i32::from(hex) - (i32::from(front) - i32::from(position));
            let to = u8::try_from(back)
                .ok()
                .filter(|b| acting.two_hex && legal.moves.contains(b))
                .unwrap_or(hex);
            Some(Command::Move { to })
        }
        (Kind::Shoot | Kind::BrokenArrow, Spot::Hex(hex)) => {
            let target = target?;
            let at = legal.aim(target, hex)?;
            Some(Command::Shoot { target, at })
        }
        (Kind::Attack, Spot::Hex(hex)) => {
            let from = attack_slots(field, hex)[slot(layout, point, hex)].from?;
            Some(Command::Attack {
                target: target?,
                from,
            })
        }
        (Kind::Wall, _) => Some(Command::ShootWall { part: target? }),
        (Kind::Heal, _) => Some(Command::Heal { target: target? }),
        _ => None,
    }
}

/// The command the panel's button `id` gives when it is clicked (its hotkey's release too:
/// `def_button_message` 0x456190) for the acting stack's commands `legal` (E17 M5 T2a): Wait and
/// Defend when listed (a disabled button gives none).
#[must_use]
pub fn button(legal: &Legal, id: u16) -> Option<Command> {
    match id {
        WAIT if legal.wait => Some(Command::Wait {}),
        DEFEND if legal.defend => Some(Command::Defend {}),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
