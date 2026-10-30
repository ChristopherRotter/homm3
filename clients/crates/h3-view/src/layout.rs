//! Screen layouts as data (E17 M1 T7; ADR-0020 point 3: the frame size and every layout are data,
//! never literals in code). One file per screen and resolution under `clients/layouts/`; the
//! native battle screen is `battle.toml` (`docs/epics/E17-design.md` §4). An HD layout is another
//! file of the same format. The format is *provisional* until the M1 review.

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use serde::Deserialize;

use h3_ui::ui::{Dialog, Text};

/// The battle screen's layout: the frame, the hex grid and where sprites stand.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BattleLayout {
    /// The frame's width and height (800 × 600).
    pub frame: [u32; 2],
    /// The top-left corner of the box of the hex in row 0, column 0, before an even row's shift
    /// (14, 86; §4.1).
    pub hex_origin: [i32; 2],
    /// A column's width and a row's height (44, 42).
    pub hex_step: [i32; 2],
    /// What an even row adds to x (22: half a hex; rows count from 0).
    pub even_row_shift: i32,
    /// Where a sprite stands, from its hex box's top-left corner (22, 42; the hex record's anchor).
    pub anchor: [i32; 2],
    /// A creature's canvas facing right has its top-left corner at the anchor minus this
    /// (196, 267); facing left it is mirrored, its left edge at the anchor's x minus the DEF's
    /// width plus this x (§4.2).
    pub stack_canvas: [i32; 2],
    /// The first and last column the pass of the layer-1 obstacles reads (1, 15; §4.6).
    pub before_stacks_columns: [u8; 2],
    /// The palette whose entries colour the outlines (`game.pal`, §4.2).
    pub outline_palette: String,
    /// The active stack's outline: that palette's entry (0x60) before its range rotates.
    pub outline_active: u8,
    /// The entries of the active outline's range, from `outline_active` on (8): each idle step
    /// rotates them by one, each taking its lower neighbour's colour (0x522950, §4.2).
    pub outline_ramp: u8,
    /// The outline of the stack the mouse marks: the palette's entry (0x70) before its range, as
    /// long as the active one's, rotates (0x43E23A; E17 M3 T6c-1).
    pub outline_mouse: u8,
    /// The heroes' canvases' top-left corners, the left side's and the right side's, which is
    /// mirrored (−43, −19; 693, −19; `battle_redraw` 0x494740, §4.5; E17 M3 T3a).
    pub hero_canvas: [[i32; 2]; 2],
    /// The flags' canvases' top-left corners, drawn before their heroes (29, 20; 755, 20).
    pub flag_canvas: [[i32; 2]; 2],
    /// The palette of the players' colours (`PLAYERS.PAL`): a player's flag takes its 32 entries
    /// from 32 × the player's colour over the DEF's entries 224–255 (0x5FFE20, §4.5).
    pub player_palette: String,
    /// The movement shade's cell (`ccellshd.pcx`, E17 M5 T4c2a, §7.23): at a shaded hex's box
    /// corner, every pixel of a non-zero index halves what lies under it (0x44E6A0).
    pub cell_shade: String,
    /// A stack's count box (§4.3; E17 M3 T4a).
    pub count_box: CountBox,
    /// The mouse over the battle (E17 M3 T6c-1; design §9.9).
    pub mouse: Mouse,
    /// The panel below the battlefield, a dialog of the executable's items (§9.5, §9.7; E17 M3
    /// T4b-2b-1, T6a): its picture, buttons and the battle log's item (`crate::panel` names
    /// them).
    pub panel: Dialog,
    /// The panel in the tactics phase (`battle_panel_tactics_create` 0x46C050; E17 M5 T4c2a,
    /// §7.23): the base's picture and buttons, Next Creature and Start Combat, no log's item.
    pub tactics_panel: Dialog,
    /// The hero's spell book (E17 M5 T3a; design §7.17; `crate::book` names its items).
    pub book: BookLayout,
    /// The message box's assets and fonts (E17 M5 T4a; design §7.19; `crate::message` lays it
    /// out).
    pub message: crate::message::MessageLayout,
    /// The result window's assets and fonts (E17 M5 T4b1; design §7.20; `crate::result` lays it
    /// out).
    pub result: crate::result::ResultLayout,
}

/// The hero's spell book (`spell_book_dialog` 0x59BDF0; E17 M5 T3a, design §7.17): its dialog and
/// the mastery ribbons' DEFs.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BookLayout {
    /// The schools by the book's index (its school bit's: air 0, fire 1, water 2, earth 3), the
    /// rulesets' school keys without their pack: the ribbons', emblems' and tabs' order.
    pub schools: Vec<String>,
    /// The mastery ribbons' DEFs by school: air, fire, water, earth (0x641D94).
    pub ribbons: Vec<String>,
    /// The cursor the book sets (`crdeflt.def`, the mouse manager's type 0, 0x67FF38).
    pub cursor: String,
    /// Its frames' hot spots (0x67FF50): the canvas's corner is the mouse less its frame's.
    pub hot: Vec<[i32; 2]>,
    /// The dialog, its loops' entries made into their items ([`Dialog::expanded`]).
    pub dialog: Dialog,
}

/// The mouse over the battle (`battle_buttons`' mouse move 0x4753A4; design §9.9): where it finds
/// a hex or a hero, and the combat cursor.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Mouse {
    /// The battlefield's part of the screen from its corner (800 × 556, 0x4753E4): the panel below.
    pub field: [i32; 2],
    /// The heroes' rectangles, the attacker's and the defender's: left, top, right and bottom,
    /// inclusive (`screen_to_hex` 0x4647A0, from 0x4626A0 and 0x4626D0).
    pub heroes: [[i32; 4]; 2],
    /// A siege's keep's rectangle ("hex" 0xFE) and its upper tower's (0xFF), the same way (set for
    /// every battle by 0x462700 and 0x462730; E17 M4 T2c-2).
    pub keep: [i32; 4],
    /// The upper tower's rectangle.
    pub tower: [i32; 4],
    /// The height of a hex box's slanted top edge (10, 0x4648B6).
    pub slant: i32,
    /// A hex's centre from its box's corner (22, 26): the attack's slot is the mouse's angle from
    /// it (0x474A54).
    pub centre: [i32; 2],
    /// The combat cursor's DEF (`crcombat.DEF`, the mouse manager's type 2, 0x67FF38).
    pub cursor: String,
    /// Its frames' hot spots (0x67FF50): the canvas's corner is the mouse less its frame's.
    pub hot: Vec<[i32; 2]>,
    /// The frame over the panel (6, 0x47551F).
    pub panel_frame: u32,
    /// The spell cursor (E17 M5 T3b, design §7.18): `crspell.DEF`, the mouse manager's type 3
    /// (0x67FF38), set by the aim over a hex it takes (`spell_target_cursor` 0x59F700).
    pub spell_cursor: String,
    /// Its frames' hot spot, the same for each (0x67FF50 + (3 × 144 + frame) × 8: (18, 27)).
    pub spell_hot: [i32; 2],
    /// Its frames (group 0's 20): the mouse's poll steps it through them in turn
    /// (`mouse_poll` 0x50D838).
    pub spell_frames: u32,
    /// The time between its frames (the poll's second grid, 100 ms: 0x50D827).
    pub spell_step: u32,
}

/// A stack's count box (`draw_battle_stack`'s part 1, 0x43E568; design §4.3): its picture beside
/// the stack, its inside tinted, the count written over it.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CountBox {
    /// The picture, index 0 transparent (`CmNumWin.pcx`, 0x463395).
    pub picture: String,
    /// The font of the count (`tiny.fnt`, 0x698A04).
    pub font: String,
    /// The palette the font's colours come from (`game.pal`, copied when the font loads,
    /// 0x55B817).
    pub font_palette: String,
    /// The box's top-left corner from the stack's anchor when the hex ahead is free: facing right
    /// (22, −15) and facing left (52, −30), whose x is negated (0x43E5B5).
    pub beside: [[i32; 2]; 2],
    /// What a two-hex stack adds to x before a left-facing one's is negated (44).
    pub two_hex: i32,
    /// When the hex ahead is taken: added to x before the negation (−37) and the y (−15).
    pub taken: [i32; 2],
    /// The tinted rectangle inside the box: its corner from the box's and its size (1, 1; 28 × 9;
    /// 0x43E682).
    pub tint_rect: [[i32; 2]; 2],
    /// The tint of a stack without spells: hue 0.75 and saturation 0.8, as the bits of the f32
    /// values the executable passes.
    pub tint: [u32; 2],
    /// The count's text box, from the box's corner (30 × 15; 0x43E776).
    pub text_size: [i32; 2],
    /// The text drawer's colour argument (4: `game.pal`'s entry 13, white).
    pub text_colour: u32,
    /// The text drawer's alignment (1: each line centred, from the box's top).
    pub text_align: u32,
}

impl CountBox {
    /// The box's top-left corner beside a stack anchored at `anchor`: facing right or not, two
    /// hexes wide or not, the hex ahead taken or not, the creature's troop-count offset `t`
    /// (added when the hex ahead is free; 0x43E5B5 to 0x43E651).
    #[must_use]
    pub fn corner(
        &self,
        anchor: [i32; 2],
        right: bool,
        wide: bool,
        taken: bool,
        t: i32,
    ) -> [i32; 2] {
        let [mut x, mut y] = self.beside[usize::from(!right)];
        if wide {
            x += self.two_hex;
        }
        if taken {
            x += self.taken[0];
            y = self.taken[1];
        } else {
            x += t;
        }
        if !right {
            x = -x;
        }
        [anchor[0] + x, anchor[1] + y]
    }
}

impl BattleLayout {
    /// The battle log's item in the panel (control 2005, `crate::panel::LOG`), if the layout has
    /// one.
    #[must_use]
    pub fn log(&self) -> Option<&Text> {
        self.panel.text(crate::panel::LOG)
    }

    /// The panel the battle `observation` shows: in the tactics phase its own (E17 M5 T4c2a,
    /// `battle_panel_tactics_create` 0x46C050), else the battle's.
    #[must_use]
    pub fn panel_in(&self, observation: &h3_protocol::wire::Observation) -> &Dialog {
        if observation.tactics.is_some() {
            &self.tactics_panel
        } else {
            &self.panel
        }
    }

    /// The top-left corner of the box of the hex in `row` and `column`.
    pub fn hex_box(&self, row: u8, column: u8) -> [i32; 2] {
        let shift = if row.is_multiple_of(2) {
            self.even_row_shift
        } else {
            0
        };
        [
            self.hex_origin[0] + self.hex_step[0] * i32::from(column) + shift,
            self.hex_origin[1] + self.hex_step[1] * i32::from(row),
        ]
    }

    /// The palette entry whose colour the active stack's outline shows after `steps` idle steps:
    /// the range rotated `steps` times (*provisional*, E17 M2 T4b-1: the opening shows one step
    /// on, design §6.2).
    pub fn outline_after(&self, steps: u32) -> u8 {
        let ramp = u32::from(self.outline_ramp.max(1));
        let back = (ramp - steps % ramp) % ramp;
        self.outline_active
            .saturating_add(u8::try_from(back).unwrap_or(0))
    }

    /// The palette entry whose colour the outline of the stack the mouse marks shows after `steps`
    /// idle steps: its range rotated as the active one's (E17 M3 T6c-1).
    pub fn mouse_outline_after(&self, steps: u32) -> u8 {
        let ramp = u32::from(self.outline_ramp.max(1));
        let back = (ramp - steps % ramp) % ramp;
        self.outline_mouse
            .saturating_add(u8::try_from(back).unwrap_or(0))
    }

    /// Where a sprite on the hex in `row` and `column` stands (the hex record's anchor).
    pub fn anchor(&self, row: u8, column: u8) -> [i32; 2] {
        let [x, y] = self.hex_box(row, column);
        [x + self.anchor[0], y + self.anchor[1]]
    }
}

/// Parses a battle layout file.
///
/// # Errors
/// The parser's message: not TOML, an unknown, missing or mistyped field; the book's `schools`
/// not four distinct names (the four the book draws, its school bits 1 to 8: R4-4).
pub fn parse_battle(text: &str) -> Result<BattleLayout, String> {
    let mut layout: BattleLayout = toml::from_str(text).map_err(|e| e.to_string())?;
    let schools = &layout.book.schools;
    let distinct = schools
        .iter()
        .enumerate()
        .all(|(i, s)| !schools[..i].contains(s));
    if schools.len() != 4 || !distinct {
        return Err(alloc::format!(
            "book.schools: {schools:?}, not the four distinct schools the book draws (air, fire, \
             water, earth by its index)"
        ));
    }
    let empty = Dialog {
        place: [0, 0],
        shadow: None,
        items: Vec::new(),
    };
    layout.book.dialog = core::mem::replace(&mut layout.book.dialog, empty).expanded();
    Ok(layout)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec::Vec;

    /// The committed native layout.
    const NATIVE: &str = include_str!("../../../layouts/battle.toml");

    #[test]
    fn the_native_layout_gives_the_executables_hex_boxes() {
        let layout = parse_battle(NATIVE).unwrap();
        assert_eq!(layout.frame, [800, 600]);
        // §4.1: left = 14 + 44c + 22e, top = 86 + 42r, e = 1 on even rows; anchor (+22, +42).
        assert_eq!(layout.hex_box(0, 0), [36, 86]);
        assert_eq!(layout.hex_box(1, 0), [14, 128]);
        assert_eq!(layout.hex_box(10, 16), [14 + 44 * 16 + 22, 86 + 420]);
        assert_eq!(layout.anchor(5, 3), [14 + 132 + 22, 86 + 210 + 42]);
        assert_eq!(layout.outline_palette, "game.pal");
        assert_eq!(layout.outline_active, 0x60);
        // §6.2: the opening (one step) shows entry 0x67 (224), the eighth step 0x60 (255).
        let shown: Vec<u8> = (0..10).map(|s| layout.outline_after(s)).collect();
        assert_eq!(
            shown,
            [0x60, 0x67, 0x66, 0x65, 0x64, 0x63, 0x62, 0x61, 0x60, 0x67]
        );
        assert_eq!(layout.hero_canvas, [[-43, -19], [693, -19]]);
        assert_eq!(layout.flag_canvas, [[29, 20], [755, 20]]);
        assert_eq!(layout.player_palette, "PLAYERS.PAL");
        // §4.3: the count box by facing, size and the hex ahead, at the anchor (100, 200).
        let b = &layout.count_box;
        let at = [100, 200];
        assert_eq!(b.corner(at, true, false, false, 0), [122, 185]);
        assert_eq!(b.corner(at, true, true, false, 3), [169, 185]);
        assert_eq!(b.corner(at, true, false, true, 3), [85, 185]);
        assert_eq!(b.corner(at, true, true, true, 0), [129, 185]);
        assert_eq!(b.corner(at, false, false, false, 0), [48, 170]);
        assert_eq!(b.corner(at, false, true, false, -2), [6, 170]);
        assert_eq!(b.corner(at, false, false, true, 0), [85, 185]);
        assert_eq!(b.corner(at, false, true, true, 0), [41, 185]);
        assert_eq!(b.tint_rect, [[1, 1], [28, 9]]);
        assert_eq!(b.tint, [0x3F40_0000, 0x3F4C_CCCD]);
        assert_eq!((b.text_size, b.text_colour, b.text_align), ([30, 15], 4, 1));
        // §9.5, §9.7: the panel at (0, 556), its log's item at its (214, 7).
        assert_eq!(layout.panel.place, [0, 556]);
        let log = layout.log().unwrap();
        assert_eq!((log.at, log.size), ([214, 7], [400, 32]));
        assert_eq!((log.colour, log.align), (1, 1));
        assert_eq!(layout.panel.items.len(), 11);
        // §7.23 (E17 M5 T4c2a): the tactics phase's panel, without the log's three items; the
        // phase's shade cell.
        let mut phase = crate::record::tests::observation();
        assert_eq!(layout.panel_in(&phase), &layout.panel);
        phase.tactics = Some(h3_protocol::wire::Tactics {
            side: h3_protocol::wire::Side::Attacker,
            columns: 3,
        });
        let tactics = layout.panel_in(&phase);
        assert_eq!((tactics.place, tactics.items.len()), ([0, 556], 10));
        assert!(tactics.text(crate::panel::LOG).is_none());
        assert_eq!(layout.cell_shade, "ccellshd.pcx");
        // §9.9: the mouse's outline from 0x70, rotated as the active one's; the mouse.
        assert_eq!(layout.mouse_outline_after(1), 0x77);
        let m = &layout.mouse;
        assert_eq!((m.field, m.slant, m.centre), ([800, 556], 10, [22, 26]));
        assert_eq!(m.heroes, [[0, 16, 57, 127], [741, 16, 799, 127]]);
        assert_eq!((m.keep, m.tower), ([742, 160, 799, 337], [564, 0, 651, 85]));
        assert_eq!((m.hot.len(), m.hot[7], m.panel_frame), (20, [22, 0], 6));
        // §7.17: the book at (90, 2), its 51 items once the loops are made items; place 11's
        // icon at the right page's last row and column, its text below it.
        let book = &layout.book;
        assert_eq!((book.dialog.place, book.dialog.items.len()), ([90, 2], 51));
        assert_eq!((book.schools.len(), book.ribbons.len()), (4, 4));
        assert_eq!((book.cursor.as_str(), book.hot.len()), ("crdeflt.def", 2));
        let area = |id| book.dialog.item(id).and_then(h3_ui::ui::Item::area);
        assert_eq!(area(224), Some(([419, 281], [78, 65])));
        assert_eq!(area(248), Some(([415, 346], [86, 36])));
        assert_eq!(area(0x7801), Some(([478, 407], [35, 42])));
        let flat = BattleLayout {
            outline_ramp: 0,
            ..layout
        };
        assert_eq!(flat.outline_after(5), 0x60);
        assert_eq!(flat.mouse_outline_after(5), 0x70);
    }

    #[test]
    fn a_bad_layout_is_an_error() {
        assert!(parse_battle("frame = [1, 2]\nwhat = 3\n").is_err());
        // R4-4: the book's schools not the four it draws, distinct: an error naming them.
        let four = "schools = [\"air\", \"fire\", \"water\", \"earth\"]";
        assert!(NATIVE.contains(four));
        for schools in [
            "[\"air\", \"fire\", \"water\"]",
            "[\"air\", \"fire\", \"water\", \"earth\", \"light\"]",
            "[\"air\", \"fire\", \"air\", \"earth\"]",
        ] {
            let text = NATIVE.replacen(four, &alloc::format!("schools = {schools}"), 1);
            let error = parse_battle(&text).unwrap_err();
            assert!(error.starts_with("book.schools: "), "{error}");
        }
    }
}
