//! The army info windows on the mouse's move (E17 M6 T4b2b1, `docs/epics/E17-design.md` §12.15):
//! the battle dialog's six windows (`combat_dialog_create` 0x472483 to 0x4725D4, each with its
//! shown flag), shown by the mouse entering a new hex at Creature Info's levels 1 and 2 and by the
//! right press over a hero, hidden by a new hex, a left press over the battlefield, the key T and
//! every action's start, drawn at the end of every `battle_redraw` while the level is not 0.
//!
//! [`Windows`] is their state at a time: which are shown, what filled them and the level. The
//! battle's mouse over time ([`crate::mouse::track`]) changes it at the inputs and the log's
//! entries; a still ([`Hover`]) at the harness's moves and presses at the opening. A hero's
//! window is filled as the right press fills it ([`crate::hero_info`]). The stacks' four windows
//! (E17 M6 T4b2b2: kind 1 "all statistics" at level 1, kind 2 "spells only" at level 2, by the
//! stack's owner's side; `army_info_stack_create` 0x46CEB0) are filled by [`stack_window`]
//! (`army_info_stack_set` 0x46DC30) from what the stats dialog shows of the stack
//! ([`StackInfo::of`]), so the two agree.

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use h3_protocol::wire::{Observation, Side};
use h3_render::Bugs;
use h3_ui::text::{Args, Catalogue};
use h3_ui::ui::Shown;
use serde::Deserialize;

use crate::hero_info::HeroInfoLayout;
use crate::layout::BattleLayout;
use crate::message::MessageBox;
use crate::mouse::{Facing, Field, Legal, Spot, spot};
use crate::stats::{Pair, Stats};

/// A stack's window's background (`CCrPop.pcx` for kind 1, `SpellInf.pcx` for kind 2), in the
/// colours of its side's hero's player (the ids are the executable's, from 0x898).
pub const STACK_BACKGROUND: u16 = 2200;
/// The creature's portrait (`TwCrPort.def`, its frame the creature + 2; kind 1).
pub const STACK_PORTRAIT: u16 = 2201;
/// The first row's label; its value is the next id, then the next rows' pairs (attack, defense,
/// damage, health: 2202 to 2209; kind 1).
pub const STACK_ROWS: u16 = 2202;
/// Morale's label (kind 1).
pub const STACK_MORALE_LABEL: u16 = 2210;
/// Morale's picture (`imrls.def`; kind 1).
pub const STACK_MORALE: u16 = 2211;
/// Luck's label (kind 1).
pub const STACK_LUCK_LABEL: u16 = 2212;
/// Luck's picture (`ilcks.def`; kind 1).
pub const STACK_LUCK: u16 = 2213;
/// The count, over the portrait's bottom right (`Verd10B.fnt`; kind 1).
pub const STACK_COUNT: u16 = 2214;
/// The first of the three spells' icons (`spellint.def`, 2215 to 2217).
pub const STACK_SPELLS: u16 = 2215;
/// The text under the icons: "No active spells" (GENRLTXT 675) with none, else empty.
pub const STACK_NO_SPELLS: u16 = 2218;
/// How many spells a stack's window shows: the list's last three.
pub const SPELLS_SHOWN: usize = 3;

/// The rows' words' keys, attack to health (GENRLTXT 381, 382, 387, 390).
const ROW_WORDS: [&str; 4] = [
    "battle.hero-info.attack",
    "battle.hero-info.defense",
    "battle.army-info.damage",
    "battle.army-info.health",
];

/// The stacks' windows' layouts (the layout's `[army_info]`; `combat_dialog_create`'s
/// 0x472483 to 0x4725D4): each kind's places by side and its items from its corner (*provisional*:
/// the hero's window's [`HeroInfoLayout`], the same two fields).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArmyInfoLayout {
    /// Kind 1, "all statistics", 78 × 288: the attacker's side's at (1, 267), the defender's at
    /// (721, 267).
    pub all: HeroInfoLayout,
    /// Kind 2, "spells only", 78 × 126: at (1, 429) and (721, 429).
    pub spells: HeroInfoLayout,
}

/// Which of a stack's two windows: kind 1 (level 1, every statistic) or kind 2 (level 2, its
/// spells alone; GENRLTXT 403 and 404, the options' boxes 233 and 234).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// "All Statistics" (+0x7C, +0x80).
    All,
    /// "Spells Only" (+0x84, +0x88).
    Spells,
}

impl Kind {
    /// The kind Creature Info's level `level` shows over a stack (none at 0).
    #[must_use]
    pub const fn at_level(level: u8) -> Option<Self> {
        match level {
            0 => None,
            1 => Some(Self::All),
            _ => Some(Self::Spells),
        }
    }
}

/// A stack's window while shown: which of the four (its owner's side's, the kind the level gave
/// it), the stack it shows and the turn whose battle fills it (*provisional*: the window is
/// filled again from that battle at every frame, as the heroes').
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StackShown {
    /// The stack's owner's side (+0xF4): the window's place.
    pub side: Side,
    /// The kind.
    pub kind: Kind,
    /// The stack's id.
    pub stack: u8,
    /// The turn (0 the opening's).
    pub turn: usize,
}

/// A window drawn: a side's hero's with the turn that filled it, or a stack's.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Drawn {
    /// Side's hero's window (+0x74, +0x78), filled by the turn's battle.
    Hero(Side, usize),
    /// A stack's (+0x7C to +0x88).
    Stack(StackShown),
}

/// The army info windows at a time: those shown (their flags, each with what filled it) and
/// Creature Info's level ("Combat Army Info Level" 0x6987C8), which draws the shown ones while
/// not 0 (§12.14).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Windows {
    /// Each side's hero's window while shown (the attacker's first: +0x74 at (1, 135), +0x78 at
    /// (721, 135)): the turn whose battle filled it (0 the opening's).
    pub heroes: [Option<usize>; 2],
    /// The stack's window shown (E17 M6 T4b2b2): one of the four at most, since only a new hex
    /// shows one and it hides all six first.
    pub stack: Option<StackShown>,
    /// Creature Info's level, 0 to 2.
    pub level: u8,
}

impl Windows {
    /// None shown, at `level`.
    #[must_use]
    pub const fn at_level(level: u8) -> Self {
        Self {
            heroes: [None; 2],
            stack: None,
            level,
        }
    }

    /// Every window hidden (`army_info_hide` 0x46CE70 and `army_info_stack_hide` 0x46DF80 on the
    /// six); the level kept.
    pub const fn hide(&mut self) {
        self.heroes = [None; 2];
        self.stack = None;
    }

    /// The mouse entering a new hex `spot` (0x47557D to 0x475879; [`Spot::Nothing`] for −1) in
    /// turn `turn`'s battle `field` (its stacks where they stand and face): every window hidden,
    /// then at most one shown while the level is not 0: over a side's hero's rectangle (0xFC,
    /// 0xFD) with a hero there, its window, filled by the turn's battle (`army_info_hero_set`,
    /// then `army_info_show`); over a hex a living stack stands on (`hex_stack`, either hex of a
    /// two-hex one: moving between them hides and shows it again, oddity 10), its window by its
    /// owner's side, kind 1 at level 1 and kind 2 at level 2 (`army_info_stack_set`, then
    /// `army_info_stack_show`).
    pub fn enter(&mut self, spot: Spot, field: &Field<'_>, turn: usize) {
        self.hide();
        let Some(kind) = Kind::at_level(self.level) else {
            return;
        };
        match spot {
            Spot::Hero(side) if field.observation.sides[index(side)].hero.is_some() => {
                self.heroes[index(side)] = Some(turn);
            }
            Spot::Hex(hex) => {
                self.stack = field.at(hex).map(|s| StackShown {
                    side: s.side,
                    kind,
                    stack: s.id,
                    turn,
                });
            }
            // Anything else: none.
            Spot::Hero(_) | Spot::Keep | Spot::Tower | Spot::Nothing => {}
        }
    }

    /// The right button pressed over side `side`'s hero's rectangle in turn `turn` and released
    /// (`battle_right_click` 0x476A0B, 0x476A81, then 0x476A1D): its window filled and shown while
    /// held, then hidden at level 0; at 1 and 2 it stays shown, as a hover's (oddity 6: one shown
    /// already is filled again, nothing drawn anew).
    pub fn held(&mut self, side: Side, turn: usize) {
        self.heroes[index(side)] = (self.level != 0).then_some(turn);
    }

    /// Whether the right button held over side `side`'s hero shows its window in profile `bugs`
    /// (CLT-Q50, design §12.15's oddity 4): the original's shows nothing when the window is shown
    /// already at level 0 (still flagged, `army_info_show` does nothing and the redraws at 0 skip
    /// it); the client's shows it whatever (fixed, the user's answer of 2026-11-04).
    #[must_use]
    pub const fn held_shows(&self, side: Side, bugs: Bugs) -> bool {
        matches!(bugs, Bugs::Fixed) || self.level != 0 || self.heroes[index(side)].is_none()
    }

    /// The windows drawn (each `battle_redraw`'s end): the shown ones while the level is not 0,
    /// in the battle dialog's order (the heroes', then the stack's), each with what fills it;
    /// none at level 0. A stack's window keeps its kind through a change of the level (§12.14
    /// oddity 6: kind 1 shown at level 1 is drawn at 2 until a new hex).
    pub fn drawn(&self) -> impl Iterator<Item = Drawn> + '_ {
        [Side::Attacker, Side::Defender]
            .into_iter()
            .zip(self.heroes)
            .filter_map(|(side, turn)| Some(Drawn::Hero(side, turn?)))
            .chain(self.stack.map(Drawn::Stack))
            .filter(|_| self.level != 0)
    }
}

/// What a stack's window shows of it (`army_info_stack_set` 0x46DC30's fill, design §12.15
/// "Their fill").
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StackInfo {
    /// The background's player colour (+0x22 of the stack's side's hero; none: the viewing
    /// player's, as the stats dialog's).
    pub player: u8,
    /// The creature's original id (the portrait's frame is it + 2).
    pub creature: u32,
    /// Attack: the creature's own (its record's +0x54) and the stack's now (`stack_attack_against`
    /// with no target; when it can shoot now the higher of its ranged and its melee one, as the
    /// stats dialog's).
    pub attack: Pair,
    /// Defense: the creature's own (+0x58) and the stack's now (`stack_defense_against`(no
    /// striker, 1)).
    pub defense: Pair,
    /// The damage's range as the stats dialog shows it (a Ballista's each end times its hero's
    /// attack plus 1): the client's profile's (CLT-Q48, fixed).
    pub damage: [i32; 2],
    /// The stack's own range (+0xD0, +0xD4: a war machine's its record's): the original's.
    pub own_damage: [i32; 2],
    /// Hit points per creature with their bonuses (+0xC0).
    pub health: i32,
    /// Morale (+0x4E8; its picture's frame held to −3..=3, + 3).
    pub morale: i32,
    /// Luck (+0x4EC; as morale).
    pub luck: i32,
    /// The count (+0x4C: the windows show only between actions, when +0x50 is −1).
    pub count: u32,
    /// The spells on the stack, their original ids, in its list's order (+0x420).
    pub spells: Vec<u32>,
}

impl StackInfo {
    /// The window's numbers from the stats dialog's `stats` of the stack (so the two agree), its
    /// creature's original id `creature`, the background's `player` and the stack's own damage
    /// range `own_damage` (the original's Dmg row).
    #[must_use]
    pub fn of(stats: &Stats, (creature, player): (u32, u8), own_damage: [i32; 2]) -> Self {
        Self {
            player,
            creature,
            attack: stats.attack,
            defense: stats.defense,
            damage: stats.damage,
            own_damage,
            health: stats.health.now,
            morale: stats.morale,
            luck: stats.luck,
            count: stats.count,
            spells: stats.effects.iter().map(|e| e.spell).collect(),
        }
    }
}

/// The stack's window `shown` (its side's and kind's) showing `info`, its words from
/// `catalogue`, in profile `bugs` (`army_info_stack_set` 0x46DC30): the background in the
/// player's colours, and in kind 1 the portrait, the rows Att and Def "%d(%d)" (even when equal,
/// oddity 9), Dmg "%d" or "%d-%d", Health "%d", morale's and luck's frames and the count; in both
/// the list's last three spells' icons (frame the spell + 1, the rest 0) and "No active spells"
/// without any. The Dmg row is the stats dialog's range in the client's profile, the stack's own
/// in the original's (CLT-Q48, the user's answer of 2026-11-04: a Ballista's).
#[must_use]
pub fn stack_window(
    layout: &ArmyInfoLayout,
    catalogue: &Catalogue,
    (side, kind): (Side, Kind),
    info: &StackInfo,
    bugs: Bugs,
) -> MessageBox {
    let placed = match kind {
        Kind::All => &layout.all,
        Kind::Spells => &layout.spells,
    };
    let mut dialog = placed.dialog.clone();
    dialog.place = placed.places[index(side)];
    let number =
        |value: i64| catalogue.text("battle.hint.number", &Args::new().number("value", value));
    let mut texts = BTreeMap::new();
    let mut frames = BTreeMap::new();
    if kind == Kind::All {
        let label = |key: &str| {
            let word = catalogue.text(key, &Args::new());
            catalogue.text("battle.hero-info.label", &Args::new().text("word", &word))
        };
        let pair = |p: Pair| {
            let args = Args::new()
                .number("own", i64::from(p.own))
                .number("now", i64::from(p.now));
            catalogue.text("battle.army-info.pair", &args)
        };
        let [low, high] = match bugs {
            Bugs::Fixed => info.damage,
            Bugs::Original => info.own_damage,
        };
        let damage = if low == high {
            number(i64::from(low))
        } else {
            let args = Args::new()
                .number("low", i64::from(low))
                .number("high", i64::from(high));
            catalogue.text("battle.army-info.range", &args)
        };
        let values = [
            pair(info.attack),
            pair(info.defense),
            damage,
            number(i64::from(info.health)),
        ];
        for (k, (key, value)) in ROW_WORDS.iter().zip(values).enumerate() {
            let id = STACK_ROWS + 2 * u16::try_from(k).unwrap_or(0);
            texts.insert(id, label(key));
            texts.insert(id + 1, value);
        }
        texts.insert(STACK_MORALE_LABEL, label("battle.hero-info.morale"));
        texts.insert(STACK_LUCK_LABEL, label("battle.hero-info.luck"));
        texts.insert(STACK_COUNT, number(i64::from(info.count)));
        let frame = |v: i32| u32::try_from(v.clamp(-3, 3) + 3).unwrap_or(3);
        frames.insert(STACK_PORTRAIT, info.creature + 2);
        frames.insert(STACK_MORALE, frame(info.morale));
        frames.insert(STACK_LUCK, frame(info.luck));
    }
    // The list's last three, from index max(size − 3, 0); the icons left frame 0.
    let last = &info.spells[info.spells.len().saturating_sub(SPELLS_SHOWN)..];
    for k in 0..SPELLS_SHOWN {
        let id = STACK_SPELLS + u16::try_from(k).unwrap_or(0);
        frames.insert(id, last.get(k).map_or(0, |spell| spell + 1));
    }
    let none = if last.is_empty() {
        catalogue.text("battle.army-info.no-spells", &Args::new())
    } else {
        String::new()
    };
    texts.insert(STACK_NO_SPELLS, none);
    let shown = Shown {
        texts,
        frames,
        players: [(STACK_BACKGROUND, info.player)].into(),
        ..Shown::default()
    };
    MessageBox { dialog, shown }
}

/// The side's index in the battle's pairs (the attacker's first).
const fn index(side: Side) -> usize {
    match side {
        Side::Attacker => 0,
        Side::Defender => 1,
    }
}

/// Whether `point` of the screen is over the battlefield (x 0 to 799, y 0 to 555: the move's
/// test, 0x475402; the left press's, 0x4752BB).
#[must_use]
pub fn on_field(layout: &BattleLayout, [x, y]: [i32; 2]) -> bool {
    let [width, height] = layout.mouse.field;
    (0..width).contains(&x) && (0..height).contains(&y)
}

/// The battle's mouse as the army info windows follow it in a still (E17 M6 T4b2b1,
/// *provisional*): the harness's moves and presses at the opening, its battle the opening's (turn
/// 0), its level the still's (`frames.toml`'s `army_info`; the key F5 cycles it). The window's
/// battle follows the same rules over time in [`crate::mouse::track`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Hover {
    /// The mouse's hex (+0x132D4; none: −1).
    hex: Option<Spot>,
    /// The windows.
    pub windows: Windows,
    /// Creature Info's level when the battle last wrote the status line (a new hex's, E17 M6
    /// T4b2b2, *provisional*: the still's line is its mouse's rest's, worded at this level; F5
    /// after it rewrites nothing, §12.14 oddity 6).
    pub line_level: u8,
}

impl Hover {
    /// The mouse nowhere yet, no window shown, at `level`.
    #[must_use]
    pub const fn new(level: u8) -> Self {
        Self {
            hex: None,
            windows: Windows::at_level(level),
            line_level: level,
        }
    }

    /// The battle takes a move to `point` over `observation`'s battle (each side facing its own
    /// way, a still's), the keep's and the upper tower's rectangles found with `rects`
    /// (CLT-Q21): out of the battlefield every window hidden and the hex forgotten (0x475402);
    /// over the hex it is over already nothing changes (0x475544); over a new one
    /// [`Windows::enter`]. Either way but the same hex the status line is written again.
    pub fn moved(
        &mut self,
        layout: &BattleLayout,
        observation: &Observation,
        (point, rects): ([i32; 2], bool),
    ) {
        if !on_field(layout, point) {
            self.windows.hide();
            self.hex = None;
            self.line_level = self.windows.level;
            return;
        }
        let at = match spot(layout, point, rects) {
            Spot::Nothing => None,
            other => Some(other),
        };
        if at.is_some() && at == self.hex {
            return;
        }
        let legal = Legal::default();
        let field = Field {
            observation,
            legal: &legal,
            facing: &Facing::Sides,
            bugs: Bugs::Original,
        };
        self.windows.enter(at.unwrap_or(Spot::Nothing), &field, 0);
        self.hex = at;
        self.line_level = self.windows.level;
    }

    /// The mouse's hex forgotten (a stats dialog's close, a human's turn's start: the move they
    /// post then counts as a new hex's).
    pub const fn forget(&mut self) {
        self.hex = None;
    }

    /// A left press at `point` (0x4752BB): over the battlefield every window hidden, the mouse's
    /// hex kept (the windows come back only with another hex).
    pub fn pressed(&mut self, layout: &BattleLayout, point: [i32; 2]) {
        if on_field(layout, point) {
            self.windows.hide();
        }
    }

    /// One of the keys C, F, S, R and O pressed with the mouse at `point` (CLT-Q49, design
    /// §12.15's oddity 3): the client's profile hides the windows as a left press there does
    /// (fixed, the user's answer of 2026-11-04); the original's keeps them through the key's book,
    /// aim or box.
    pub fn key(&mut self, layout: &BattleLayout, point: [i32; 2], bugs: Bugs) {
        if bugs == Bugs::Fixed {
            self.pressed(layout, point);
        }
    }

    /// The key F5 (§12.14): the level cycled 0, 1, 2, 0; no window shown or hidden (oddity 6).
    pub const fn cycle(&mut self) {
        self.windows.level = (self.windows.level + 1) % 3;
    }
}

#[cfg(test)]
mod tests;
