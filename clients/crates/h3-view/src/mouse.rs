//! The mouse over the battle as a still (E17 M3 T6c-1; `docs/epics/E17-design.md` §9.9): where the
//! mouse is (`screen_to_hex` 0x4647A0), the cursor's kind and frame (`battle_cursor` 0x476490, the
//! attack's slots 0x474690 and 0x474A00), the status line's hint (`battle_status_line` 0x492840,
//! a control's `SetHints`) and the stack the mouse marks (`battle_mouse_stack` 0x477C20).
//!
//! The rules' answers (where the acting stack may move, whom it may strike from where, shoot or
//! heal) are the legal commands the server lists ([`Legal`]); what the executable's cursor decides
//! on its own (the slot inside the hex, the broken arrow's distance from the shooter's facing) is
//! ported here. Over time (the hint against the log's new lines, a stack marked starting its
//! fidget, a target left over from the last hex) is T6c-2's.

use alloc::collections::BTreeSet;
use alloc::string::String;

use h3_protocol::wire::{Command, Observation, Side, Stack};

use crate::layout::BattleLayout;
use crate::scene::Draw;
use crate::text::{Args, Catalogue};

/// The battlefield's columns.
pub const COLUMNS: u8 = 17;
/// Its rows.
pub const ROWS: u8 = 11;
/// Its hexes.
const HEXES: i32 = 187;
/// The distance beyond which a shot is halved (`shot_full_damage_distance` 0x467600).
const FULL_DAMAGE_DISTANCE: i32 = 10;

/// Where the mouse is over the battlefield (`screen_to_hex`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Spot {
    /// A hex.
    Hex(u8),
    /// A side's hero's rectangle ("hexes" 0xFC and 0xFD).
    Hero(Side),
    /// Nothing.
    Nothing,
}

/// Where the point `[x, y]` of the screen is on the battlefield (`screen_to_hex` 0x4647A0): a
/// hero's rectangle, else the hex whose box holds it, a point above a box's slanted top edge
/// counting for the row above.
#[must_use]
pub fn spot(layout: &BattleLayout, [x, y]: [i32; 2]) -> Spot {
    let mouse = &layout.mouse;
    for (side, [left, top, right, bottom]) in [Side::Attacker, Side::Defender]
        .into_iter()
        .zip(mouse.heroes)
    {
        if (left..=right).contains(&x) && (top..=bottom).contains(&y) {
            return Spot::Hero(side);
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

/// The acting stack's legal commands as the cursor asks them.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Legal {
    /// The positions it may walk or fly to.
    moves: BTreeSet<u8>,
    /// The stacks it may strike in melee, each with a position to strike from.
    attacks: BTreeSet<(u8, u8)>,
    /// The stacks it may shoot.
    shots: BTreeSet<u8>,
    /// The stacks it may heal.
    heals: BTreeSet<u8>,
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
                Command::Shoot { target, .. } => {
                    legal.shots.insert(target);
                }
                Command::Heal { target } => {
                    legal.heals.insert(target);
                }
                _ => {}
            }
        }
        legal
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
    /// Heal the stack with the First Aid Tent (17).
    Heal,
    /// The other side's hero (21).
    EnemyHero,
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
            Self::Heal => 17,
            Self::EnemyHero => 21,
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
    /// A control's hint: its key.
    Control(String),
}

impl Hint {
    /// Whether it replaces the status line over the log's new lines (`battle_log_add`'s third
    /// argument: an attack's or a shot's; design §9.8).
    #[must_use]
    pub const fn forced(&self) -> bool {
        matches!(self, Self::Attack(_))
    }

    /// The hint's text in `catalogue`'s words, the stacks of `observation` named by `creature`
    /// (a stack's creature key in the catalogue: `creature.base:pikeman`).
    #[must_use]
    pub fn text(
        &self,
        catalogue: &Catalogue,
        observation: &Observation,
        creature: &dyn Fn(&Stack) -> Option<String>,
    ) -> String {
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
            Self::Hero => catalogue.text("battle.hint.hero", &Args::new()),
            Self::EnemyHero => catalogue.text("battle.hint.enemy-hero", &Args::new()),
            Self::Control(key) => catalogue.text(key, &Args::new()),
        }
    }
}

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
    /// The cursor at `point` (`SetCursor(frame, 2)`, 0x50CCA0): the combat cursor's frame with
    /// its hot spot (0x50CEE0; none beyond the table), its canvas's corner the point less it.
    #[must_use]
    pub fn cursor(&self, layout: &BattleLayout, [x, y]: [i32; 2]) -> Draw {
        let hot = usize::try_from(self.frame)
            .ok()
            .and_then(|f| layout.mouse.hot.get(f).copied())
            .unwrap_or_default();
        Draw::Cursor {
            def: layout.mouse.cursor.clone(),
            frame: self.frame,
            x,
            y,
            hot,
        }
    }
}

/// The mouse at `point` of the screen in the battle `observation` shows, its acting stack's
/// commands `legal` (`battle_buttons`' mouse move 0x4753A4): over the battlefield its spot's
/// kind, over the panel the pointer and the hint of the control under it.
#[must_use]
pub fn pointer(
    observation: &Observation,
    legal: &Legal,
    layout: &BattleLayout,
    point: [i32; 2],
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
    let spot = spot(layout, point);
    let marked = match spot {
        Spot::Hex(hex) => at(observation, hex).map(|s| s.id),
        _ => None,
    };
    let (kind, target) = kind(observation, legal, spot);
    let frame = match (kind, spot) {
        (Kind::Attack, Spot::Hex(hex)) => {
            let slot = slot(layout, point, hex);
            slots(observation, legal, hex)[slot]
        }
        (Kind::EnemyHero, _) => Kind::Query.code(),
        _ => kind.code(),
    };
    let acting = observation.active;
    let hint = match (kind, target, acting) {
        (Kind::Move, _, Some(id)) => Hint::Move(id),
        (Kind::Fly, _, Some(id)) => Hint::Fly(id),
        (Kind::Shoot | Kind::BrokenArrow | Kind::Attack, Some(id), _) => Hint::Attack(id),
        (Kind::Hero, ..) => Hint::Hero,
        (Kind::EnemyHero, ..) => Hint::EnemyHero,
        (Kind::Query, Some(id), _) => Hint::View(id),
        (Kind::Heal, Some(id), _) => Hint::Heal(id),
        _ => Hint::Empty,
    };
    Pointer {
        kind,
        frame,
        hint,
        marked,
    }
}

/// A stack's hexes: its position and, two hexes wide, its front hex by its side's facing
/// (`stack_other_hex` 0x4466A0; *provisional*: a stack turned in the battle is T6c-2's).
fn hexes(stack: &Stack) -> [u8; 2] {
    let other = if !stack.two_hex {
        stack.hex
    } else if stack.side == Side::Attacker {
        stack.hex.saturating_add(1)
    } else {
        stack.hex.saturating_sub(1)
    };
    [stack.hex, other]
}

/// The living stack on `hex`.
fn at(observation: &Observation, hex: u8) -> Option<&Stack> {
    observation
        .stacks
        .iter()
        .find(|s| s.alive && hexes(s).contains(&hex))
}

/// The cursor's kind at `spot` and the stack it is about (`battle_cursor` 0x476490): the modelled
/// cases (sieges, casters and network play are not: design §9.9).
fn kind(observation: &Observation, legal: &Legal, spot: Spot) -> (Kind, Option<u8>) {
    let acting = observation
        .active
        .and_then(|id| observation.stacks.iter().find(|s| s.id == id));
    let Some(acting) = acting else {
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
        Spot::Hex(hex) => hex,
    };
    let column = hex % COLUMNS;
    if column == 0 || column == COLUMNS - 1 {
        return (Kind::Nothing, None);
    }
    if let Some(stack) = at(observation, hex) {
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
        if legal.shots.contains(&stack.id) {
            let kind = if halved(acting, stack) {
                Kind::BrokenArrow
            } else {
                Kind::Shoot
            };
            return (kind, id);
        }
        if legal.attacks.iter().any(|&(t, _)| t == stack.id) {
            return (Kind::Attack, id);
        }
        return (Kind::Nothing, None);
    }
    // An empty hex the reach map marks (+0x20E a position, +0x20F a two-hex stack's front hex,
    // 0x4B2E69, 0x4B2EF4).
    let [position, front] = hexes(acting);
    let back = i32::from(hex) - (i32::from(front) - i32::from(position));
    let reached = legal.moves.contains(&hex)
        || (acting.two_hex && u8::try_from(back).is_ok_and(|b| legal.moves.contains(&b)));
    match (reached, acting.flies) {
        (false, _) => (Kind::Nothing, None),
        (true, true) => (Kind::Fly, None),
        (true, false) => (Kind::Move, None),
    }
}

/// The cursor's own half-damage test of a shot (`shot_full_damage_distance` 0x467600 from
/// 0x4767E6): without the shooter turned to the target (its front hex by its facing now), no
/// shot of a shooter without the distance penalty is halved; else halved when both of the
/// target's hexes are farther than 10. The siege's wall test (0x467510) is not modelled.
fn halved(shooter: &Stack, target: &Stack) -> bool {
    if shooter.no_distance_penalty {
        return false;
    }
    let from = hexes(shooter)[1];
    hexes(target)
        .iter()
        .all(|&h| distance(from, h) > FULL_DAMAGE_DISTANCE)
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
    let mut frames = [0_u32; 12];
    let acting = observation
        .active
        .and_then(|id| observation.stacks.iter().find(|s| s.id == id));
    let (Some(acting), Some(target)) = (acting, at(observation, hex)) else {
        return frames;
    };
    let ok = |p: i32| u8::try_from(p).is_ok_and(|p| legal.attacks.contains(&(target.id, p)));
    let right = acting.side == Side::Attacker;
    for (s, frame) in frames.iter_mut().enumerate() {
        let direction = u8::try_from(s / 2).unwrap_or(0);
        let Some(next) = neighbour(hex, direction) else {
            continue;
        };
        let next = i32::from(next);
        let first = ok(next);
        let sword = 7 + (u32::from(direction) + 3) % 6;
        if first {
            *frame = sword;
        }
        if !acting.two_hex {
            continue;
        }
        let beside = next - if right { 1 } else { -1 };
        let second = ok(beside);
        if !first && !second {
            continue;
        }
        *frame = sword;
        if direction == 1 || direction == 4 {
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
        if !preferred || (other && matches!(s, 0 | 5 | 6 | 11)) {
            *frame = vertical;
        }
    }
    for s in 0..12 {
        if frames[s] != 0 {
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
            if frames[j] != 0 && d < best {
                best = d;
                frames[s] = frames[j];
            }
        }
    }
    frames
}

#[cfg(test)]
mod tests;
