//! The combat options dialog (E17 M6 T4a, `docs/epics/E17-design.md` §12.12): the panel's
//! Options (2003, its key O) opens it while the battle takes a command (`battle_buttons`' case
//! 2003, `battle_combat_options` 0x4686B0, modal). Its items are the layout's (`[options]`, built
//! by `combat_options_create` 0x46E3B0); [`Open`] is the dialog on the screen: the fill of every
//! item from the [`Settings`], the clicks as `combat_options_proc` 0x46F7B0 takes them, the right
//! button's help ([`help_key`]), the press samples, until the OK's click (or Enter or Escape)
//! closes it.
//!
//! The settings are the original's globals (`settings_names` 0x63FF74): the client keeps them in
//! its settings file ([`Settings::read`], [`Settings::write`]: *provisional*, a `[battle]` table
//! of the registry's value names), read as `settings_read` does (a first run's values without a
//! music volume, `settings_first_run` 0x50B4D0) and put into their ranges by `settings_check`
//! 0x50B260 ([`Settings::check`]).
//!
//! **In effect:** the combat speed (the window's battle takes it for every animation after the
//! close) and the two volumes ([`sound_volume`], the original's 0x5996C0, on the window's music
//! and effects at once; T4a), the five Auto Combat options (T4b1), the army info level (T4b2a),
//! the hex grid and the movement and the cursor shadows (T4c). **Kept, not in effect yet:** the
//! spell book's animation (the page-turn videos are not built). The battle's keys F5 to F8
//! change four of them outside the dialog: F5 the army info level
//! ([`Settings::cycle_army_info`]), F6 to F8 the grid and the shadows; the window writes the
//! settings file at each and at the dialog's close after a click.
//!
//! The original's oddities (design §12.12, "Oddities") are built as the original's but two, which
//! the client's profile fixes ([`Bugs::Fixed`]; the original's for the comparisons, the stills):
//! a box or a bar acts on a left release inside it alone, where the original's acts on one
//! anywhere (CLT-Q43 (2)), and no bar is ever refused, where the original's opens the no-sound
//! box for a column at volume 0 without the audio device (CLT-Q44 (c): the settings kept for a
//! device later). In both: the speed buttons' one sample (a DEF button's own; CLT-Q74, E17 M7
//! T1a), the hidden bars taking clicks and right presses, the effects bars writing the last
//! effects volume (the music bars leave the last music volume), the level's pair that can be
//! emptied.

use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use h3_render::Bugs;
use h3_ui::text::{Args, Catalogue};
use h3_ui::ui::{Dialog, Item, Shown, State};
use serde::Deserialize;

use crate::message::{MessageBox, OK};

/// The background (`comopbck.pcx`), in the acting side's player's colours.
pub const BACKGROUND: u16 = 200;
/// Default (`codefaul.def`).
pub const DEFAULT: u16 = 201;
/// The first music bar (volume 0); the next nine follow (202 to 211).
pub const MUSIC: u16 = 202;
/// The first effects bar (volume 0); the next nine follow (212 to 221).
pub const EFFECTS: u16 = 212;
/// The first Auto Combat box (creatures); spells, the Catapult, the Ballista and the First Aid
/// Tent follow (225 to 229).
pub const AUTO: u16 = 225;
/// The first combat speed button (speed 0); 1 and 2 follow (230 to 232).
pub const SPEEDS: u16 = 230;
/// The army info level's boxes: level 1 (all statistics), then level 2 (spells only).
pub const LEVELS: u16 = 233;
/// The left column's first box (the grid); the movement shadow, the cursor shadow and the spell
/// book's animation follow (235 to 238).
pub const BOXES: u16 = 235;
/// The first text: the client's own ids, 240 to 256, in the layout's order (the executable's are
/// all −1: *provisional*).
pub const TEXTS: u16 = 240;

/// A column's bars.
const BARS: u16 = 10;
/// The texts' keys, in the layout's order from [`TEXTS`]: the title, the speeds', the music's and
/// the effects' headings (GENRLTXT 393 to 396), the left column's labels (405 to 407, 578), the
/// Auto Combat heading (397) and labels (399 to 401, 152, 402), the army info heading (398) and
/// labels (403, 404).
const TEXT_KEYS: [&str; 17] = [
    "battle.options.title",
    "battle.options.speed",
    "battle.options.music",
    "battle.options.effects",
    "battle.options.grid",
    "battle.options.movement-shade",
    "battle.options.cursor-shade",
    "battle.options.spell-book",
    "battle.options.auto",
    "battle.options.auto-creatures",
    "battle.options.auto-spells",
    "battle.options.auto-catapult",
    "battle.options.auto-ballista",
    "battle.options.auto-first-aid-tent",
    "battle.options.info",
    "battle.options.info-all",
    "battle.options.info-spells",
];
/// The help keys' names of the Auto Combat boxes, the left column's boxes and the levels' boxes.
const AUTO_NAMES: [&str; 5] = [
    "auto-creatures",
    "auto-spells",
    "auto-catapult",
    "auto-ballista",
    "auto-first-aid-tent",
];
const BOX_NAMES: [&str; 4] = ["grid", "movement-shade", "cursor-shade", "spell-book"];
const LEVEL_NAMES: [&str; 2] = ["info-all", "info-spells"];

/// The no-sound box's text's key (GENRLTXT 151).
pub const NO_SOUND: &str = "battle.options.no-sound";

/// The dialog's layout (the layout's `[options]`).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OptionsLayout {
    /// The press sample (`button.wav`: `button_sample` 0x694DA4, the DEF buttons' and the
    /// procedure's).
    pub sound: String,
    /// The dialog at (159, 84) with its shadow, its items in the constructor's order (from its
    /// corner).
    pub dialog: Dialog,
}

/// The settings the dialog shows and sets (`settings_names` 0x63FF74; the original's globals).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Settings {
    /// Combat Speed (0x6987EC): 0 to 2.
    pub speed: u8,
    /// Music Volume (0x698760): 0 to 9.
    pub music: u8,
    /// Sound Volume, the effects' (0x698764): 0 to 9.
    pub effects: u8,
    /// Last Music Volume (0x698768): the music volume as read; no bar writes it.
    pub last_music: u8,
    /// Last Sound Volume (0x69876C): the effects volume as read, and as the effects bars set it.
    pub last_effects: u8,
    /// Show Combat Grid (0x6987BC).
    pub grid: bool,
    /// Combat Shade Level, the movement shadow (0x6987C4).
    pub movement_shade: bool,
    /// Show Combat Mouse Hex, the cursor shadow (0x6987C0).
    pub cursor_shade: bool,
    /// Animate SpellBook (0x698788).
    pub spell_book: bool,
    /// Combat Auto Creatures, Combat Auto Spells, Combat Catapult, Combat Ballista, Combat First
    /// Aid Tent (0x698794 to 0x6987A4).
    pub auto: [bool; 5],
    /// Combat Army Info Level (0x6987C8): 0 to 2.
    pub army_info: u8,
}

/// The settings file's names of the settings (its `[battle]` table; the registry's value names,
/// `settings_names` 0x63FF74, in snake case).
const NAMES: [&str; 15] = [
    "combat_speed",
    "music_volume",
    "sound_volume",
    "last_music_volume",
    "last_sound_volume",
    "show_combat_grid",
    "combat_shade_level",
    "show_combat_mouse_hex",
    "animate_spellbook",
    "combat_auto_creatures",
    "combat_auto_spells",
    "combat_catapult",
    "combat_ballista",
    "combat_first_aid_tent",
    "combat_army_info_level",
];

impl Settings {
    /// A first run's (`settings_first_run` 0x50B4D0): the combat options as Default sets them
    /// ([`Settings::defaults`]), the volumes and their last copies 5.
    pub const FIRST_RUN: Self = Self {
        speed: 0,
        music: 5,
        effects: 5,
        last_music: 5,
        last_effects: 5,
        grid: false,
        movement_shade: false,
        cursor_shade: false,
        spell_book: true,
        auto: [true; 5],
        army_info: 0,
    };

    /// The capture harness's (`clients/capture/setup.sh`'s registry): the volumes, the grid, both
    /// shadows and the spell book's animation 0, Combat Auto Creatures 1, every other value
    /// missing (0: `settings_read` with a Music Volume value calls no first run).
    pub const CAPTURE: Self = Self {
        speed: 0,
        music: 0,
        effects: 0,
        last_music: 0,
        last_effects: 0,
        grid: false,
        movement_shade: false,
        cursor_shade: false,
        spell_book: false,
        auto: [true, false, false, false, false],
        army_info: 0,
    };

    /// Default's (`combat_options_defaults` 0x50B700): the spell book's animation on, the grid,
    /// both shadows, the army info level and the combat speed 0, the five Auto Combat options on;
    /// the volumes kept.
    pub const fn defaults(&mut self) {
        self.spell_book = true;
        self.grid = false;
        self.cursor_shade = false;
        self.movement_shade = false;
        self.army_info = 0;
        self.speed = 0;
        self.auto = [true; 5];
    }

    /// The battle's key F5 (E17 M6 T4b2a, design §12.14; `battle_buttons`' case 63, 0x475902):
    /// Combat Army Info Level cycled 0, 1, 2, 0; the level now. The window draws the frame again
    /// and writes the settings at once.
    pub const fn cycle_army_info(&mut self) -> u8 {
        self.army_info = (self.army_info + 1) % 3;
        self.army_info
    }

    /// The settings from the values read, by their names in the file ([`NAMES`]; a value not
    /// read is 0), put into their ranges as `settings_check` 0x50B260 does: the flags their low
    /// bit; the army info level and the speed 0 to 2, else 0; the volumes 0 to 9, else 5, each
    /// copied into its last copy (the last values read are not kept).
    #[must_use]
    pub fn check(value: impl Fn(&str) -> i64) -> Self {
        let flag = |name: &str| value(name) & 1 == 1;
        let ranged = |name: &str, most: i64, otherwise: u8| {
            u8::try_from(value(name))
                .ok()
                .filter(|&v| i64::from(v) <= most)
                .unwrap_or(otherwise)
        };
        let (music, effects) = (ranged(NAMES[1], 9, 5), ranged(NAMES[2], 9, 5));
        Self {
            speed: ranged(NAMES[0], 2, 0),
            music,
            effects,
            last_music: music,
            last_effects: effects,
            grid: flag(NAMES[5]),
            movement_shade: flag(NAMES[6]),
            cursor_shade: flag(NAMES[7]),
            spell_book: flag(NAMES[8]),
            auto: [
                flag(NAMES[9]),
                flag(NAMES[10]),
                flag(NAMES[11]),
                flag(NAMES[12]),
                flag(NAMES[13]),
            ],
            army_info: ranged(NAMES[14], 2, 0),
        }
    }

    /// The settings a settings file's text holds (`settings_read` 0x50B7B0, *provisional*: its
    /// `[battle]` table, the other tables left to their readers): none, or a table without
    /// `music_volume`, gives a first run's ([`Settings::FIRST_RUN`]); else its values checked
    /// ([`Settings::check`]). A name it does not know or a value that is not an integer is left
    /// out as the registry's missing value (another client version's file, a hand edit; E17 M6
    /// T7b, R4-5, *provisional*), each told in a line beside the settings.
    ///
    /// # Errors
    /// Not TOML, `battle` not a table.
    pub fn read(text: Option<&str>) -> Result<(Self, Vec<String>), String> {
        let mut notes = Vec::new();
        let Some(text) = text else {
            return Ok((Self::FIRST_RUN, notes));
        };
        let file: toml::Table = toml::from_str(text).map_err(|e| e.to_string())?;
        let empty = toml::Table::new();
        let table = match file.get("battle") {
            None => &empty,
            Some(value) => value
                .as_table()
                .ok_or_else(|| String::from("`battle` is not a table"))?,
        };
        let mut values = BTreeMap::new();
        for (name, value) in table {
            if !NAMES.contains(&name.as_str()) {
                notes.push(format!("battle: no setting `{name}`, left out"));
                continue;
            }
            let Some(v) = value.as_integer() else {
                notes.push(format!("battle.{name}: not an integer, taken as missing"));
                continue;
            };
            values.insert(name.as_str(), v);
        }
        if !values.contains_key(NAMES[1]) {
            return Ok((Self::FIRST_RUN, notes));
        }
        let settings = Self::check(|name| values.get(name).copied().unwrap_or(0));
        Ok((settings, notes))
    }

    /// The settings file's text (`settings_write` 0x50BE10: every setting; *provisional*): its
    /// `[battle]` table, flags as 0 or 1.
    #[must_use]
    pub fn write(&self) -> String {
        let values: [u8; 15] = [
            self.speed,
            self.music,
            self.effects,
            self.last_music,
            self.last_effects,
            u8::from(self.grid),
            u8::from(self.movement_shade),
            u8::from(self.cursor_shade),
            u8::from(self.spell_book),
            u8::from(self.auto[0]),
            u8::from(self.auto[1]),
            u8::from(self.auto[2]),
            u8::from(self.auto[3]),
            u8::from(self.auto[4]),
            self.army_info,
        ];
        let mut text = String::from(
            "# The battle client's settings (h3-view window; E17 M6 T4a): the original's registry\n\
             # values, written as the combat options dialog closes after a click, at the keys F5\n\
             # to F8.\n[battle]\n",
        );
        for (name, value) in NAMES.iter().zip(values) {
            text.push_str(&format!("{name} = {value}\n"));
        }
        text
    }
}

/// A sound's volume on Miles' scale (0 to 127) at a volume setting (`sound_volume` 0x5996C0): for
/// a setting of 1 to 10, `volume` × (setting + 1) / 10, at least 1, at most 127; 0 otherwise.
#[must_use]
pub fn sound_volume(volume: u8, setting: u8) -> u8 {
    if !(1..=10).contains(&setting) {
        return 0;
    }
    let v = u32::from(volume) * (u32::from(setting) + 1) / 10;
    u8::try_from(v.clamp(1, 127)).unwrap_or(127)
}

/// The key of the help the right button shows over item `id` (`combat_options_hints` 0x6A55A8's
/// right-click field, `Help.txt` 392 for the OK, else 392 + (id − 200)): the OK, Default, a bar
/// (hidden ones too), a box, a speed (the chosen one too); none over the picture, the texts and
/// anything else.
#[must_use]
pub fn help_key(id: u16) -> Option<String> {
    let name = match id {
        OK => String::from("ok"),
        DEFAULT => String::from("default"),
        MUSIC..=211 => format!("music-{}", id - MUSIC),
        EFFECTS..=221 => format!("effects-{}", id - EFFECTS),
        AUTO..=229 => String::from(AUTO_NAMES[usize::from(id - AUTO)]),
        SPEEDS..=232 => format!("speed-{}", id - SPEEDS),
        LEVELS..=234 => String::from(LEVEL_NAMES[usize::from(id - LEVELS)]),
        BOXES..=238 => String::from(BOX_NAMES[usize::from(id - BOXES)]),
        _ => return None,
    };
    Some(format!("battle.options.help.{name}"))
}

/// What an input did to the open dialog.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Did {
    /// The dialog changed: draw it again.
    pub changed: bool,
    /// A DEF button's own press sample (`button_press` 0x456860).
    pub button_sound: bool,
    /// The procedure's press sample (a press on a DEF picture 225 to 238: the boxes; 0x46FDB1,
    /// volume 0x40). A speed button's press sends the procedure none: its own loop takes the
    /// mouse until the release (CLT-Q74).
    pub box_sound: bool,
    /// A bar clicked at its column's volume 0 without the audio device, in the original's profile:
    /// GENRLTXT 151 in an OK box, nothing changed (0x46FE42; CLT-Q44 (c): never in the client's).
    pub no_sound: bool,
    /// The OK's click closed the dialog.
    pub closed: bool,
}

/// What the mouse or a key holds down.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Held {
    /// A DEF button by the mouse, and whether the mouse is over it.
    Button { id: u16, over: bool },
    /// A DEF picture (a bar or a box) by the mouse, and whether the mouse is over it: its release
    /// clicks it over it (the client's profile, CLT-Q43 (2)) or anywhere (the original's).
    Picture { id: u16, over: bool },
    /// The OK by its key.
    Key(String),
}

/// The dialog on the screen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Open {
    /// The settings as its clicks left them.
    pub settings: Settings,
    /// The dialog and what its items show.
    pub window: MessageBox,
    /// Anything was clicked (+0x4C: every handled click, Default and an unchanged volume too):
    /// the settings are saved as it closes.
    pub clicked: bool,
    /// The background's player.
    player: u8,
    /// The texts, filled once.
    texts: BTreeMap<u16, String>,
    /// The client's audio device opened (the sound manager's digital driver, +0x3C).
    audio: bool,
    /// The profile: the client's fixes or the original's oddities 2 and 7.
    bugs: Bugs,
    held: Option<Held>,
}

impl Open {
    /// The dialog of `layout` showing `settings`, its background in `player`'s colours, its words
    /// from `catalogue`; `audio`: the client's audio device opened; `bugs` the profile.
    #[must_use]
    pub fn new(
        layout: &OptionsLayout,
        catalogue: &Catalogue,
        settings: Settings,
        (player, audio, bugs): (u8, bool, Bugs),
    ) -> Self {
        let texts = (TEXTS..)
            .zip(TEXT_KEYS)
            .map(|(id, key)| (id, catalogue.text(key, &Args::new())))
            .collect();
        let mut open = Self {
            settings,
            window: MessageBox {
                dialog: layout.dialog.clone(),
                shown: Shown::default(),
            },
            clicked: false,
            player,
            texts,
            audio,
            bugs,
            held: None,
        };
        open.fill();
        open
    }

    /// Every item from the settings (the constructor's states, and each handled click's set
    /// again): the chosen speed disabled (its frame 2), each box its option's frame (0 off, 1 on;
    /// a level's box on at its level), each column's bar at its volume alone shown with its frame
    /// the volume. Never while a button is held (a click releases it first).
    fn fill(&mut self) {
        let s = self.settings;
        let mut shown = Shown {
            players: [(BACKGROUND, self.player)].into(),
            texts: self.texts.clone(),
            ..Shown::default()
        };
        for k in 0..3 {
            let disabled = u16::from(s.speed) == k;
            shown.states.insert(
                SPEEDS + k,
                State {
                    disabled,
                    ..State::default()
                },
            );
        }
        let boxes = [
            (BOXES, s.grid),
            (BOXES + 1, s.movement_shade),
            (BOXES + 2, s.cursor_shade),
            (BOXES + 3, s.spell_book),
            (LEVELS, s.army_info == 1),
            (LEVELS + 1, s.army_info == 2),
        ];
        let auto = (AUTO..).zip(s.auto);
        for (id, on) in boxes.into_iter().chain(auto) {
            shown.frames.insert(id, u32::from(on));
        }
        for (first, volume) in [(MUSIC, s.music), (EFFECTS, s.effects)] {
            for k in 0..BARS {
                let id = first + k;
                if k == u16::from(volume) {
                    shown.frames.insert(id, u32::from(k));
                } else {
                    shown.states.insert(
                        id,
                        State {
                            hidden: true,
                            ..State::default()
                        },
                    );
                }
            }
        }
        self.window.shown = shown;
    }

    /// The item at `point` (on the screen) of the kinds that take the mouse here: a DEF button
    /// or a DEF picture, hidden ones too (a DEF picture takes the mouse by its bit 2 alone).
    fn item_at(&self, point: [i32; 2]) -> Option<&Item> {
        self.window
            .dialog
            .item_at(point)
            .filter(|i| matches!(i, Item::Button(_) | Item::Def(_)))
    }

    /// The left button pressed at `point`: a DEF button there goes down (not the chosen speed,
    /// disabled) with its own press sample alone, a speed's too (CLT-Q74: its loop sends the
    /// procedure no press; design §12.12's oddity 1 read again); a bar or a box is held,
    /// unchanged, a box's press sounding.
    pub fn press(&mut self, point: [i32; 2]) -> Did {
        if self.held.is_some() {
            return Did::default();
        }
        let item = self
            .item_at(point)
            .map(|i| (i.id(), matches!(i, Item::Button(_))));
        let Some((id, button)) = item else {
            return Did::default();
        };
        if !button {
            self.held = Some(Held::Picture { id, over: true });
            return Did {
                box_sound: (AUTO..=BOXES + 3).contains(&id),
                ..Did::default()
            };
        }
        if self.window.shown.state(id).disabled {
            return Did::default();
        }
        self.held = Some(Held::Button { id, over: true });
        set_pressed(&mut self.window.shown, id, true);
        Did {
            changed: true,
            button_sound: true,
            ..Did::default()
        }
    }

    /// The mouse moved to `point`: the DEF button it holds follows it on or off, pressed again
    /// with its sample as it comes back (`def_button_message`'s loop, 0x4563A2: `button_press`;
    /// the procedure gets nothing meanwhile, so no second sample; E17 M6 T7b, R4-3); the bar or
    /// box it holds notes whether the mouse is over it, nothing drawn (a picture's press changes
    /// nothing).
    pub fn moved(&mut self, point: [i32; 2]) -> Did {
        let over_id = |id: u16| self.item_at(point).is_some_and(|i| i.id() == id);
        let (id, over) = match self.held {
            Some(Held::Picture { id, .. }) => {
                self.held = Some(Held::Picture {
                    id,
                    over: over_id(id),
                });
                return Did::default();
            }
            Some(Held::Button { id, over }) => (id, over),
            Some(Held::Key(_)) | None => return Did::default(),
        };
        let now = over_id(id);
        if now == over {
            return Did::default();
        }
        self.held = Some(Held::Button { id, over: now });
        set_pressed(&mut self.window.shown, id, now);
        Did {
            changed: true,
            button_sound: now,
            ..Did::default()
        }
    }

    /// The left button released: the DEF button held clicks when the mouse is still over it,
    /// else goes up; the bar or box held clicks when the mouse is over it, in the original's
    /// profile wherever the mouse is (oddity 2: `dialog_def_message`'s release tests no box;
    /// CLT-Q43 (2), fixed in the client's).
    pub fn release(&mut self) -> Did {
        let anywhere = self.bugs == Bugs::Original;
        match self.held.take_if(|h| !matches!(h, Held::Key(_))) {
            Some(Held::Button { id, over: true }) => self.click(id),
            Some(Held::Picture { id, over }) if over || anywhere => self.click(id),
            Some(Held::Button { id, .. }) => {
                set_pressed(&mut self.window.shown, id, false);
                Did {
                    changed: true,
                    ..Did::default()
                }
            }
            _ => Did::default(),
        }
    }

    /// The OK's key down (Enter or Escape; `by` the key whose release clicks): the OK goes down
    /// with its press sample (`button_press` from 0x4565E7; E17 M6 T7b, R4-3).
    pub fn key(&mut self, by: &str) -> Did {
        if self.held.is_some() {
            return Did::default();
        }
        self.held = Some(Held::Key(String::from(by)));
        set_pressed(&mut self.window.shown, OK, true);
        Did {
            changed: true,
            button_sound: true,
            ..Did::default()
        }
    }

    /// A key up: the OK its key held clicks.
    pub fn key_up(&mut self, by: &str) -> Did {
        match self.held.take_if(|h| *h == Held::Key(String::from(by))) {
            Some(_) => self.click(OK),
            None => Did::default(),
        }
    }

    /// The key of the help the right button pressed at `point` shows ([`help_key`]): over a DEF
    /// button or a DEF picture, whatever its state; none while the mouse holds a DEF button,
    /// whose own loop drops the right press (`def_button_message` 0x456338 to 0x45641C: only the
    /// moves and the releases act there; E17 M6 T7b, R4-2).
    #[must_use]
    pub fn help_at(&self, point: [i32; 2]) -> Option<String> {
        if matches!(self.held, Some(Held::Button { .. })) {
            return None;
        }
        help_key(self.item_at(point)?.id())
    }

    /// Lets what the mouse (`by` none) or the key `by` holds go without its click, a DEF button
    /// drawn up again: the release the right button's help popup took (`popup_run_held` 0x602A40
    /// takes a left release as its end and drops a key's). The original keeps the press (a box or
    /// a bar then clicks at the next left release anywhere, the OK stays drawn pressed); the
    /// client's profile lets it go, as CLT-Q39's stats dialog (`message::Open::let_go`; E17 M6
    /// T7b, R4-2). No still chords them, so the original's is not built for the comparisons.
    pub fn let_go(&mut self, by: Option<&str>) -> Did {
        let held = self.held.take_if(|h| match h {
            Held::Key(key) => by == Some(key.as_str()),
            Held::Button { .. } | Held::Picture { .. } => by.is_none(),
        });
        let id = match held {
            Some(Held::Button { id, .. }) => id,
            Some(Held::Key(_)) => OK,
            Some(Held::Picture { .. }) | None => return Did::default(),
        };
        set_pressed(&mut self.window.shown, id, false);
        Did {
            changed: true,
            ..Did::default()
        }
    }

    /// Item `id` clicked (0x200/0xD, the switch at 0x46F88D): the OK closes; Default sets the
    /// defaults; a bar its column's volume and its last copy for the effects (or, in the
    /// original's profile at the column's volume 0 without the audio device, the no-sound box and
    /// nothing else: oddity 7; the client's never refuses, CLT-Q44 (c)); a box toggles its
    /// option, a level's box sets its level or 0 when it is the one set; a speed is chosen. Each
    /// handled click marks the dialog clicked and fills it again.
    fn click(&mut self, id: u16) -> Did {
        let s = &mut self.settings;
        let level = |s: &Settings, l: u8| if s.army_info == l { 0 } else { l };
        let at = |first: u16| u8::try_from(id - first).unwrap_or(0);
        match id {
            OK => {
                set_pressed(&mut self.window.shown, OK, false);
                return Did {
                    changed: true,
                    closed: true,
                    ..Did::default()
                };
            }
            DEFAULT => s.defaults(),
            MUSIC..=211 | EFFECTS..=221 => {
                let music = id < EFFECTS;
                let volume = if music { s.music } else { s.effects };
                if volume == 0 && !self.audio && self.bugs == Bugs::Original {
                    return Did {
                        no_sound: true,
                        ..Did::default()
                    };
                }
                if music {
                    s.music = at(MUSIC);
                } else {
                    s.effects = at(EFFECTS);
                    s.last_effects = s.effects;
                }
            }
            AUTO..=229 => {
                let i = usize::from(id - AUTO);
                s.auto[i] = !s.auto[i];
            }
            SPEEDS..=232 => s.speed = at(SPEEDS),
            LEVELS => s.army_info = level(s, 1),
            234 => s.army_info = level(s, 2),
            BOXES => s.grid = !s.grid,
            236 => s.movement_shade = !s.movement_shade,
            237 => s.cursor_shade = !s.cursor_shade,
            238 => s.spell_book = !s.spell_book,
            _ => return Did::default(),
        }
        self.clicked = true;
        self.fill();
        Did {
            changed: true,
            ..Did::default()
        }
    }
}

/// Item `id` drawn pressed or released in `shown`, its other state bits kept.
fn set_pressed(shown: &mut Shown, id: u16, pressed: bool) {
    let state = shown.state(id);
    shown.states.insert(id, State { pressed, ..state });
}

#[cfg(test)]
mod tests;
