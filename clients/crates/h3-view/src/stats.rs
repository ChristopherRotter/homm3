//! A stack's stats dialog (`stack_stats_dialog` 0x468860, its constructor 0x5F3360 and the
//! creature dialog's procedure 0x5F4850; E17 M6 T2a1, `docs/epics/E17-design.md` §12.1): the
//! creature's picture in its town's frame, its name and count, its numbers (each its own and the
//! stack's), its morale and luck, the last three spells on it, an OK, an empty status line, then
//! a Faerie Dragon's cast button or the creature's ability text. [`place`] puts it next to the
//! stack's hex, [`layout`] builds its items as `ui` items in the executable's order from what it
//! shows ([`Stats`], the binary's `stats` derives it from a seat's observation and the rules),
//! [`Steps`] plays its creature over time as the procedure's idle tick does.
//!
//! *Not built yet* (M6 T2a2): the modal dialog's input (the OK, the cast button, the mouse's
//! hints, the right button's descriptions); the right button's dialog held over a stack (M6 T2b).

use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use serde::Deserialize;

use crate::message::MessageBox;
use h3_ui::ui::{Button, Def, Dialog, Frames, Item, Label, Picture, Shown, Text};

/// The background (`CrStkPU.pcx`), in the colours of the stack's controlling hero's owner.
pub const BACKGROUND: u16 = 200;
/// The creature (its battle DEF, group 2, the flags 0x12: kept at +0xB4 for the idle tick).
pub const CREATURE: u16 = 201;
/// The creature's town's frame (`CrBkg*.pcx`).
pub const TOWN: u16 = 202;
/// The creature's name.
pub const NAME: u16 = 203;
/// The count.
pub const COUNT: u16 = 204;
/// The first line's label (attack); each line takes a label and a value, the lines' ids in
/// order from it: attack 205 and 206, defense 207 and 208, shots 209 and 210, damage 211 and
/// 212, health 213 and 214, health left 215 and 216, speed 217 and 218.
pub const LINES: u16 = 205;
/// Morale (`imrl42.def`).
pub const MORALE: u16 = 219;
/// Luck (`ilck42.def`).
pub const LUCK: u16 = 220;
/// The first effect (`spellint.def`), the others 222 and 223.
pub const EFFECTS: u16 = 221;
/// The status line.
pub const STATUS: u16 = 224;
/// The OK's back (`Box64x30.pcx`, the modal dialog's).
pub const OK_BACK: u16 = 225;
/// The OK (`iOKAY.def`): a message box's ([`crate::message::OK`]), Escape and Enter.
pub const OK: u16 = crate::message::OK;
/// A Faerie Dragon's cast button (`icm005.def`; its click ends the dialog with 301).
pub const CAST: u16 = 301;
/// The items the executable gives the id −1 take ids of their own here, so that their texts can
/// be told apart ([`Shown::texts`]): the cast button's back, then the ability text.
const OWN_IDS: u16 = 0x100;
/// The cast button's back.
pub const CAST_BACK: u16 = OWN_IDS;
/// The ability text.
pub const ABILITY: u16 = OWN_IDS + 1;
/// The most effects shown: the stack's list's last three (0x5F65B0).
pub const EFFECTS_SHOWN: usize = 3;

/// The dialog's size (0x5F3388: 298 × 311); it casts a shadow (the base flags 0x12).
pub const SIZE: [i32; 2] = [298, 311];
/// Its corner from the stack's hex's anchor (0x4688AD: 149 left, 140 up), and the most its
/// corner takes (0x4688C1: 502, 0x4688D6: 275); the least is 0.
const FROM_ANCHOR: [i32; 2] = [149, 140];
const MOST: [i32; 2] = [502, 275];
/// The lines' boxes (0x5F5340 to 0x5F62F0): the left edge, the width and height, each line's
/// top; the damage's value sits a row above its label (0x5F59A2).
const LINE_X: i32 = 154;
const LINE_SIZE: [i32; 2] = [122, 17];
const LINE_TOPS: [i32; 7] = [48, 66, 85, 104, 123, 142, 161];
const DAMAGE_VALUE_TOP: i32 = 103;
const DAMAGE_LINE: usize = 3;
/// A line's label's and value's alignments (left, right) and colour.
const LABEL_ALIGN: u32 = 4;
const VALUE_ALIGN: u32 = 6;
const LINE_COLOUR: u32 = 1;
/// The creature's place and size (0x5F50B8: 21, 48, 100 × 130), its group (2: standing).
const PICTURE_AT: [i32; 2] = [21, 48];
const PICTURE_SIZE: [i32; 2] = [100, 130];
/// The group the creature's item starts in (standing) and the time between its steps.
pub const STANDING: u32 = 2;
/// The idle tick's period (0x5F4FFE: 100 ms).
pub const STEP_MS: u32 = 100;

/// The dialog's assets and fonts (the layout's `[stats]`): the executable's literals.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StatsLayout {
    /// The background (`CrStkPU.pcx`, 298 × 311).
    pub picture: String,
    /// The creature's frame by town (0x68290C): the neutral one first, then the towns in their
    /// order (Castle to Conflux).
    pub towns: Vec<String>,
    /// The texts' font (`smalfont.fnt`).
    pub font: String,
    /// The count's font (`Verd10B.fnt`).
    pub count_font: String,
    /// The palette the fonts' colours come from (`game.pal`).
    pub font_palette: String,
    /// Morale's pictures (`imrl42.def`, frame the morale + 3).
    pub morale: String,
    /// Luck's pictures (`ilck42.def`, frame the luck + 3).
    pub luck: String,
    /// The DEFs whose frames of the same names the original draws in morale's and luck's
    /// places in their colours (CLT-Q36: `imrlB.def`, `ilckB.def`; none: their own).
    #[serde(default)]
    pub morale_frames: Option<String>,
    /// Luck's (see `morale_frames`).
    #[serde(default)]
    pub luck_frames: Option<String>,
    /// The spells' small pictures (`spellint.def`, frame the spell's original id + 1).
    pub effects: String,
    /// The status line's picture (`VARBack.pcx`).
    pub status: String,
    /// The OK's back (`Box64x30.pcx`).
    pub button_back: String,
    /// The OK (`iOKAY.def`).
    pub ok: String,
    /// The cast button's back (`Box46x32.pcx`).
    pub cast_back: String,
    /// The cast button (`icm005.def`).
    pub cast: String,
}

/// A number the creature has of its own and the stack has now: shown "%d" when they are the
/// same, else "%d(%d)" (0x5F53C9).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Pair {
    /// The creature's own (its record's).
    pub own: i32,
    /// The stack's.
    pub now: i32,
}

impl Pair {
    /// The pair's text.
    #[must_use]
    pub fn text(self) -> String {
        if self.own == self.now {
            format!("{}", self.own)
        } else {
            format!("{}({})", self.own, self.now)
        }
    }
}

/// What the dialog shows of a stack (the binary derives it from a seat's observation).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Stats {
    /// The creature's name: the singular for exactly one creature, else the plural.
    pub name: String,
    /// The creature's battle DEF.
    pub def: String,
    /// The creature's town (0 Castle to 8 Conflux), none for a neutral creature.
    pub town: Option<u8>,
    /// The stack's count (none shown at 0).
    pub count: u32,
    /// Attack: the higher of the ranged and the melee one when the stack can shoot now.
    pub attack: Pair,
    /// Defense (0 while frenzied).
    pub defense: Pair,
    /// A shooter's shots: its own and those left.
    pub shots: Option<Pair>,
    /// The damage's range.
    pub damage: [i32; 2],
    /// Hit points, the creature's own and the stack's.
    pub health: Pair,
    /// The top creature's hit points.
    pub health_left: i32,
    /// Speed (each below 0 shown as 0; a war machine's now 0).
    pub speed: Pair,
    /// The stack's morale (its frame clamped to ±3).
    pub morale: i32,
    /// The stack's luck (its frame clamped to ±3).
    pub luck: i32,
    /// The spells on the stack in its list's order, by original id (the last three shown).
    pub effects: Vec<u32>,
    /// The creature's ability text (none: no text).
    pub ability: Option<String>,
    /// The cast button instead of the ability text (a Faerie Dragon of the side acting, with
    /// casts left, outside the tactics phase, in the modal dialog).
    pub cast: bool,
    /// The modal dialog (a left press): with its OK.
    pub modal: bool,
}

/// The lines' labels (`PRISKILL.TXT`'s two, `GENRLTXT.TXT`'s five).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Texts {
    /// "Attack Skill".
    pub attack: String,
    /// "Defense Skill".
    pub defense: String,
    /// "Shots".
    pub shots: String,
    /// "Damage".
    pub damage: String,
    /// "Health".
    pub health: String,
    /// "Health Left".
    pub health_left: String,
    /// "Speed".
    pub speed: String,
}

impl Texts {
    /// The labels from `catalogue` (the keys `stats.*`).
    #[must_use]
    pub fn of(catalogue: &h3_ui::text::Catalogue) -> Self {
        let text = |key: &str| catalogue.text(key, &h3_ui::text::Args::new());
        Self {
            attack: text("stats.attack"),
            defense: text("stats.defense"),
            shots: text("stats.shots"),
            damage: text("stats.damage"),
            health: text("stats.health"),
            health_left: text("stats.health-left"),
            speed: text("stats.speed"),
        }
    }
}

/// The dialog's corner on the screen for a stack whose hex's anchor is `anchor` (0x46888A to
/// 0x4688DE).
#[must_use]
pub fn place(anchor: [i32; 2]) -> [i32; 2] {
    [0, 1].map(|i| anchor[i].saturating_sub(FROM_ANCHOR[i]).clamp(0, MOST[i]))
}

/// The dialog at `place` laid out as the executable builds it (0x5F3360, the items in its
/// order), its background in the colours of `player`.
#[must_use]
pub fn layout(
    layout: &StatsLayout,
    texts: &Texts,
    stats: &Stats,
    place: [i32; 2],
    player: u8,
) -> MessageBox {
    let mut items = Vec::new();
    let mut shown_texts = BTreeMap::new();
    let mut label = |id: u16, at: [i32; 2], size: [i32; 2], (font, colour, align), text: &str| {
        shown_texts.insert(id, String::from(text));
        Item::Label(Label {
            id,
            at,
            size,
            font: String::from(font),
            font_palette: layout.font_palette.clone(),
            colour,
            align,
            hint: None,
            repeat: None,
        })
    };
    let def = |id: u16, (def, frames_of): (&str, Option<&String>), at, size, frame| {
        Item::Def(Def {
            id,
            def: Some(String::from(def)),
            at,
            size,
            frame,
            mirrored: false,
            group: 0,
            creature: false,
            frames_of: frames_of.cloned(),
            repeat: None,
        })
    };
    let picture = |id: u16, picture: &str, at: [i32; 2], size: [i32; 2]| {
        Item::Picture(Picture {
            id,
            picture: String::from(picture),
            at,
            size: Some(size),
            hint: None,
        })
    };
    let font = layout.font.as_str();
    items.push(picture(BACKGROUND, &layout.picture, [0, 0], SIZE));
    // The name (0x5F361A).
    items.push(label(NAME, [20, 21], [258, 19], (font, 7, 5), &stats.name));
    // The picture (0x5F5060): the town's frame, the creature, the count when above 0.
    let town = stats.town.map_or(0, |t| usize::from(t) + 1);
    let frame = layout.towns.get(town).map_or("", String::as_str);
    items.push(picture(TOWN, frame, PICTURE_AT, PICTURE_SIZE));
    items.push(Item::Def(Def {
        id: CREATURE,
        def: Some(stats.def.clone()),
        at: PICTURE_AT,
        size: PICTURE_SIZE,
        frame: 0,
        mirrored: false,
        group: STANDING,
        creature: true,
        frames_of: None,
        repeat: None,
    }));
    if stats.count > 0 {
        items.push(label(
            COUNT,
            [21, 160],
            [100, 20],
            (layout.count_font.as_str(), 4, 10),
            &format!("{}", stats.count),
        ));
    }
    // The lines (0x5F5340 to 0x5F62F0): a shooter's shots only.
    let damage = if stats.damage[0] == stats.damage[1] {
        format!("{}", stats.damage[0])
    } else {
        format!("{} - {}", stats.damage[0], stats.damage[1])
    };
    let speed = Pair {
        own: stats.speed.own.max(0),
        now: stats.speed.now.max(0),
    };
    let lines = [
        (Some(&texts.attack), stats.attack.text()),
        (Some(&texts.defense), stats.defense.text()),
        (
            stats.shots.map(|_| &texts.shots),
            stats.shots.map(Pair::text).unwrap_or_default(),
        ),
        (Some(&texts.damage), damage),
        (Some(&texts.health), stats.health.text()),
        (Some(&texts.health_left), format!("{}", stats.health_left)),
        (Some(&texts.speed), speed.text()),
    ];
    for (k, ((name, value), top)) in lines.into_iter().zip(LINE_TOPS).enumerate() {
        let Some(name) = name else { continue };
        let id = LINES + 2 * u16::try_from(k).unwrap_or(0);
        let value_top = if k == DAMAGE_LINE {
            DAMAGE_VALUE_TOP
        } else {
            top
        };
        items.push(label(
            id,
            [LINE_X, top],
            LINE_SIZE,
            (font, LINE_COLOUR, LABEL_ALIGN),
            name,
        ));
        items.push(label(
            id + 1,
            [LINE_X, value_top],
            LINE_SIZE,
            (font, LINE_COLOUR, VALUE_ALIGN),
            &value,
        ));
    }
    // Morale and luck (0x5F3742, 0x5F3882), their frames clamped.
    let clamped = |v: i32| u32::try_from(v.clamp(-3, 3) + 3).unwrap_or(0);
    items.push(def(
        MORALE,
        (&layout.morale, layout.morale_frames.as_ref()),
        [23, 189],
        [42, 38],
        clamped(stats.morale),
    ));
    items.push(def(
        LUCK,
        (&layout.luck, layout.luck_frames.as_ref()),
        [77, 189],
        [42, 38],
        clamped(stats.luck),
    ));
    // The effects (0x5F65B0): the list's last three.
    let first = stats.effects.len().saturating_sub(EFFECTS_SHOWN);
    for (k, &spell) in stats.effects.iter().skip(first).enumerate() {
        let k16 = u16::try_from(k).unwrap_or(0);
        let x = 127 + 52 * i32::from(k16);
        items.push(def(
            EFFECTS + k16,
            (&layout.effects, None),
            [x, 186],
            [48, 36],
            spell + 1,
        ));
    }
    // The modal dialog's OK (0x5F6870).
    if stats.modal {
        items.push(picture(OK_BACK, &layout.button_back, [215, 237], [66, 32]));
        items.push(Item::Button(Button {
            id: OK,
            def: layout.ok.clone(),
            at: [216, 238],
            size: [64, 30],
            frames: Frames::default(),
            hint: None,
        }));
    }
    // The status line (0x5F394D), empty.
    items.push(Item::Text(Text {
        id: STATUS,
        picture: layout.status.clone(),
        at: [7, 285],
        size: [284, 19],
        font: String::from(font),
        font_palette: layout.font_palette.clone(),
        colour: 1,
        align: 1,
        hint: None,
    }));
    // A Faerie Dragon's cast button (0x5F39EE, 0x5F3A5D), else the ability text (0x5F3AB4).
    if stats.cast {
        items.push(Item::Picture(Picture {
            id: CAST_BACK,
            picture: layout.cast_back.clone(),
            at: [74, 236],
            size: Some([48, 34]),
            hint: None,
        }));
        items.push(Item::Button(Button {
            id: CAST,
            def: layout.cast.clone(),
            at: [75, 237],
            size: [48, 36],
            frames: Frames::default(),
            hint: None,
        }));
    } else if let Some(text) = &stats.ability {
        items.push(label(ABILITY, [20, 232], [192, 41], (font, 4, 0), text));
    }
    let mut players = BTreeMap::new();
    players.insert(BACKGROUND, player);
    MessageBox {
        dialog: Dialog {
            place,
            shadow: Some(SIZE),
            items,
        },
        shown: Shown {
            texts: shown_texts,
            players,
            ..Shown::default()
        },
    }
}

/// The groups a creature's step draws its next one from after a group ends, with their weights
/// out of 100 (`dialog_creature_step` 0x4EB060); a war machine's (types 145 to 148,
/// `dialog_war_machine_step` 0x4EB250).
const NEXT: [(u32, u32); 11] = [
    (0, 65),
    (2, 15),
    (4, 5),
    (1, 5),
    (3, 4),
    (11, 1),
    (12, 1),
    (13, 1),
    (14, 1),
    (15, 1),
    (16, 1),
];
const NEXT_WAR_MACHINE: [(u32, u32); 4] = [(2, 94), (14, 2), (15, 2), (16, 2)];
/// The walk's start and end groups, played before and after a walk drawn or ended (0x4EB1C3).
const MOVE_START: u32 = 20;
const MOVE_END: u32 = 21;
const MOVING: u32 = 0;

/// The dialog's creature as its steps play it (`dialog_creature_step` 0x4EB060): its group and
/// frame, the group a walk's end leads to, and the viewer's own generator for the groups drawn
/// (the original's is the battle's rolls', which the viewer cannot follow: §7.2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Steps {
    /// Each group's frame count in the creature's DEF (a group not listed: none).
    frames: BTreeMap<u32, u32>,
    /// A war machine (its own weights).
    war_machine: bool,
    /// The group shown.
    pub group: u32,
    /// The frame shown.
    pub frame: u32,
    /// The group after the walk's end (+0x40).
    after: u32,
    /// The generator's state.
    rng: u32,
}

impl Steps {
    /// The creature as the dialog opens: group 2's frame 0; `frames` each group's frame count,
    /// `war_machine` whether its type is one, `seed` the viewer's generator's seed.
    #[must_use]
    pub const fn new(frames: BTreeMap<u32, u32>, war_machine: bool, seed: u32) -> Self {
        Self {
            frames,
            war_machine,
            group: STANDING,
            frame: 0,
            after: 0,
            rng: seed,
        }
    }

    fn count(&self, group: u32) -> u32 {
        self.frames.get(&group).copied().unwrap_or(0)
    }

    /// The C library's `rand()` (15 bits of a linear congruential sequence), the viewer's own.
    fn rand(&mut self) -> u32 {
        self.rng = self.rng.wrapping_mul(214_013).wrapping_add(2_531_011);
        (self.rng >> 16) & 0x7FFF
    }

    /// One step: the group's next frame, or after its last the next group.
    pub fn step(&mut self) {
        let next = self.frame.saturating_add(1);
        let count = self.count(self.group);
        if next < count {
            self.frame = next;
            return;
        }
        self.frame = 0;
        if !self.war_machine {
            if self.group == MOVE_START {
                self.group = MOVING;
                return;
            }
            if self.group == MOVE_END {
                self.group = self.after;
                return;
            }
        }
        let weights: &[(u32, u32)] = if self.war_machine {
            &NEXT_WAR_MACHINE
        } else {
            &NEXT
        };
        // A group the DEF lacks is drawn again (a DEF without any of them: standing, where the
        // original would draw for ever).
        let mut drawn = STANDING;
        for _ in 0..1000 {
            let roll = 1 + self.rand() % 100;
            let mut sum = 0;
            let group = weights
                .iter()
                .find(|&&(_, w)| {
                    sum += w;
                    roll <= sum
                })
                .map_or(STANDING, |&(g, _)| g);
            if self.count(group) > 0 {
                drawn = group;
                break;
            }
        }
        if self.war_machine {
            self.group = drawn;
            return;
        }
        if drawn == MOVING && self.group != MOVING && self.count(MOVE_START) > 0 {
            self.group = MOVE_START;
        } else if drawn != MOVING && self.group == MOVING && self.count(MOVE_END) > 0 {
            self.after = drawn;
            self.group = MOVE_END;
        } else {
            self.group = drawn;
        }
    }

    /// The steps made `since` ms after the dialog opened when the first comes `first` ms after
    /// it (the battle's frame deadline then; the next each [`STEP_MS`] later).
    #[must_use]
    pub const fn made(first: u32, since: u32) -> u32 {
        if since < first {
            0
        } else {
            (since - first) / STEP_MS + 1
        }
    }
}

#[cfg(test)]
mod tests;
