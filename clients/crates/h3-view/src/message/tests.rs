use super::*;
use alloc::string::ToString;
use alloc::vec;
use h3_assets::fnt::Glyph;

/// A font whose every glyph advances `advance`, `height` high.
fn font(advance: i32, height: u8) -> Font {
    Font {
        height,
        y_offset: 0,
        glyphs: vec![
            Glyph {
                left: 0,
                width: advance,
                right: 0,
                pixels: Vec::new(),
            };
            256
        ],
    }
}

fn assets() -> MessageLayout {
    MessageLayout {
        background: "diboxbck.pcx".to_string(),
        frame: "dialgbox.def".to_string(),
        button_back: "Box64x30.pcx".to_string(),
        ok: "iOkay.def".to_string(),
        cancel: "iCancel.def".to_string(),
        font: "medfont.fnt".to_string(),
        caption_font: "smalfont.fnt".to_string(),
        font_palette: "game.pal".to_string(),
        spell_pictures: "spellScr.def".to_string(),
        cursor: "crdeflt.def".to_string(),
    }
}

fn label(message: &MessageBox, id: u16) -> &Label {
    message
        .dialog
        .items
        .iter()
        .find_map(|i| match i {
            Item::Label(l) if l.id == id => Some(l),
            _ => None,
        })
        .unwrap()
}

/// The items of a kind, as (id, corner, size, picture or DEF, frame).
fn tiles(message: &MessageBox) -> Vec<(u16, [i32; 2], [i32; 2])> {
    message
        .dialog
        .items
        .iter()
        .filter_map(|i| match i {
            Item::Picture(p) if p.picture == "diboxbck.pcx" => Some((p.id, p.at, p.size.unwrap())),
            _ => None,
        })
        .collect()
}

fn pieces(message: &MessageBox) -> Vec<([i32; 2], u32)> {
    message
        .dialog
        .items
        .iter()
        .filter_map(|i| match i {
            Item::Def(d) if d.def.as_deref() == Some("dialgbox.def") => Some((d.at, d.frame)),
            _ => None,
        })
        .collect()
}

#[test]
fn a_short_text_takes_the_least_box() {
    let medium = font(8, 16);
    let small = font(5, 10);
    let m = layout(&assets(), (&medium, &small), "x", Style::Ok, None, 3);
    // 256 x 128 (16 + 60 + 50 = 126, rounded up), centred.
    assert_eq!(m.dialog.place, [272, 236]);
    assert_eq!(m.dialog.shadow, Some([256, 128]));
    // One tile cut to the box's height; the corners, the top and bottom edges at 64 and 128.
    assert_eq!(tiles(&m), [(200, [0, 0], [256, 128])]);
    assert_eq!(
        pieces(&m),
        [
            ([0, 0], 0),
            ([192, 0], 1),
            ([0, 64], 2),
            ([192, 64], 3),
            ([64, 0], 6),
            ([64, 64], 7),
            ([128, 0], 6),
            ([128, 64], 7),
        ]
    );
    // The text: 20 in, the box's width less 50, its top shared from 30.
    let text = label(&m, TEXT);
    assert_eq!((text.at, text.size), ([20, 31], [206, 16]));
    assert_eq!(
        (text.colour, text.align, text.font.as_str()),
        (1, 1, "medfont.fnt")
    );
    assert_eq!(m.shown.texts[&TEXT], "x");
    // The OK alone, centred (Enter its hotkey: the battle's table); its back one out.
    let ok = m.dialog.item(OK).unwrap();
    assert!(matches!(ok, Item::Button(b) if b.at == [96, 67] && b.def == "iOkay.def"));
    assert!(
        matches!(m.dialog.item(0x7866), Some(Item::Picture(p)) if p.at == [95, 66] && p.size == Some([66, 32]))
    );
    // The frame's pieces in the viewing player's colours, the tiles in their own.
    assert_eq!(m.shown.players.len(), 8);
    assert!(m.shown.players.values().all(|&p| p == 3));
    assert!(!m.shown.players.contains_key(&200));
}

#[test]
fn a_question_has_an_ok_and_a_cancel() {
    let medium = font(8, 16);
    let small = font(5, 10);
    let m = layout(&assets(), (&medium, &small), "x", Style::OkCancel, None, 0);
    assert!(matches!(m.dialog.item(0x7869), Some(Item::Picture(p)) if p.at == [54, 66]));
    assert!(matches!(m.dialog.item(YES), Some(Item::Button(b)) if b.at == [55, 67]));
    assert!(matches!(m.dialog.item(0x786A), Some(Item::Picture(p)) if p.at == [136, 66]));
    assert!(
        matches!(m.dialog.item(NO), Some(Item::Button(b)) if b.at == [137, 67] && b.def == "iCancel.def")
    );
    assert!(m.dialog.item(OK).is_none());
}

#[test]
fn a_long_text_widens_then_wraps() {
    let medium = font(8, 16);
    let small = font(5, 10);
    // 100 words of 4 glyphs (32 wide, a space 8). At 256: 6 words a line, 17 lines, 272 high: too
    // high, so 384: 9 words a line (352), 12 lines, 192 high, 576 <= 768: wrapped there.
    let text = vec!["xxxx"; 100].join(" ");
    let m = layout(&assets(), (&medium, &small), &text, Style::Ok, None, 0);
    // 352 + 50 rounded up to 448; 192 + 110 = 302 rounded up to 320.
    assert_eq!(m.dialog.shadow, Some([448, 320]));
    assert_eq!(m.dialog.place, [176, 140]);
    let t = label(&m, TEXT);
    assert_eq!((t.at, t.size), ([20, 39], [398, 192]));
    // A full tile, the strips at the right and the bottom, the corner's piece of a tile.
    assert_eq!(
        tiles(&m),
        [
            (200, [0, 0], [256, 256]),
            (201, [256, 0], [192, 256]),
            (202, [0, 256], [256, 64]),
            (203, [256, 256], [192, 64]),
        ]
    );
    // The edges every 64 between the corners, the left and right ones too.
    let p = pieces(&m);
    assert_eq!(p.len(), 4 + 2 * 5 + 2 * 3);
    assert!(p.contains(&([0, 192], 4)) && p.contains(&([384, 192], 5)));
    assert!(p.contains(&([320, 0], 6)) && p.contains(&([320, 256], 7)));
    // Two full tiles across a box 640 wide.
    let text = vec!["xxxx"; 600].join(" ");
    let m = layout(&assets(), (&medium, &small), &text, Style::Ok, None, 0);
    assert_eq!(
        tiles(&m)[..3],
        [
            (200, [0, 0], [256, 256]),
            (201, [256, 0], [256, 256]),
            (202, [512, 0], [128, 256]),
        ]
    );
}

#[test]
fn a_text_at_its_widest_wraps_there_and_its_height_stops_at_200() {
    let medium = font(8, 16);
    let small = font(5, 10);
    // 600 words: at 578 still too high, so wrapped at 578 (14 words a line), 200 high at most.
    let text = vec!["xxxx"; 600].join(" ");
    let m = layout(&assets(), (&medium, &small), &text, Style::Ok, None, 0);
    let t = label(&m, TEXT);
    assert_eq!(t.size, [590, 200]);
    assert_eq!(m.dialog.shadow, Some([640, 320]));
    // Wider than high enough: wrapped at the width it reached (3 x height above 2 x width).
    let text = ["xxxxxxxxxxxxxxx"; 5].join(" ");
    let m = layout(&assets(), (&medium, &small), &text, Style::Ok, None, 0);
    // At 256: 2 words a line (248 wide), 3 lines, 48 high: 144 <= 512, wrapped; 298 rounded up
    // to 320.
    assert_eq!(label(&m, TEXT).size, [270, 48]);
    // A text whose lines fit at its widest word: its widest line.
    let m = layout(&assets(), (&medium, &small), "xxxx\nxx", Style::Ok, None, 0);
    assert_eq!(label(&m, TEXT).size, [206, 32]);
}

#[test]
fn a_picture_and_its_caption_sit_above_the_buttons() {
    let medium = font(8, 16);
    let small = font(5, 10);
    let picture = Picture {
        def: "spellScr.def".to_string(),
        frame: 9,
        size: [83, 61],
        caption: "View Air".to_string(),
    };
    let m = layout(
        &assets(),
        (&medium, &small),
        "x",
        Style::Ok,
        Some(&picture),
        0,
    );
    // 16 + 60 + 20 + 63 + 10 + 50 = 219, rounded up to 256.
    assert_eq!(m.dialog.shadow, Some([256, 256]));
    assert_eq!(label(&m, TEXT).at, [20, 30 + (256 - 219) / 2]);
    // The slot 85 x 63, centred, its caption (85 wide: the picture's) 30 above the buttons' room.
    let def = m
        .dialog
        .items
        .iter()
        .find_map(|i| match i {
            Item::Def(d) if d.id == PICTURE => Some(d),
            _ => None,
        })
        .unwrap();
    assert_eq!((def.at, def.size, def.frame), ([85, 103], [85, 63], 9));
    let caption = label(&m, PICTURE);
    assert_eq!((caption.at, caption.size), ([85, 166], [85, 10]));
    assert_eq!(caption.font, "smalfont.fnt");
    assert_eq!(m.shown.texts[&PICTURE], "View Air");
    // The picture keeps its own colours.
    assert!(!m.shown.players.contains_key(&PICTURE));
}

#[test]
fn a_caption_widens_while_it_is_too_high() {
    let small = font(5, 20);
    let slot = |caption: &str, size| {
        Slot::of(
            &small,
            &Picture {
                def: String::new(),
                frame: 0,
                size,
                caption: caption.to_string(),
            },
        )
    };
    // None: no caption.
    assert_eq!(slot("", [10, 10]).caption, [0, 0]);
    // A word wider than 50: its width + 2; its spaces counted in its lines, not its words.
    assert_eq!(slot("xxxxxxxxxxxx", [10, 10]).caption, [62, 20]);
    assert_eq!(slot("  xxxxxxxxxxxx ", [10, 10]).caption, [62, 40]);
    // Two lines at 50 (40 high > 33): widened to 75, one line.
    assert_eq!(slot("xxxxx xxxxx", [10, 10]).caption, [75, 20]);
    // At most 110, whatever its lines.
    assert_eq!(
        slot("xxxxxxxxxxxxxxxxxxxxxxxxxxxxxx", [10, 10]).caption,
        [110, 40]
    );
    let many = ["xxxxx"; 12].join(" ");
    assert_eq!(slot(&many, [10, 10]).caption, [110, 80]);
    // Not below the picture's width + 2.
    assert_eq!(slot("x", [70, 10]).caption, [72, 20]);
}

#[test]
fn a_button_clicks_when_released_over_it_or_by_its_key() {
    let medium = font(8, 16);
    let small = font(5, 10);
    let m = layout(&assets(), (&medium, &small), "x", Style::OkCancel, None, 0);
    let [x, y] = m.dialog.place;
    let ok = [x + 60, y + 70];
    let cancel = [x + 140, y + 70];
    let mut open = Open::new(m);
    // A press off the buttons, a release, a move and an unknown key do nothing.
    assert_eq!(open.press([x + 5, y + 5]), Outcome::Nothing);
    assert_eq!(open.release(), Outcome::Nothing);
    assert_eq!(open.moved(ok), Outcome::Nothing);
    assert_eq!(open.key_up("A"), Outcome::Nothing);
    assert_eq!(open.key_up("Return"), Outcome::Nothing);
    // Pressed: down; off it: up; back on: down; released over it: clicked.
    assert_eq!(open.press(ok), Outcome::Changed);
    assert!(open.message.shown.state(YES).pressed);
    assert_eq!(open.press(cancel), Outcome::Nothing);
    assert_eq!(open.key(Key::Cancel, "Escape"), Outcome::Nothing);
    assert_eq!(open.moved([x + 61, y + 71]), Outcome::Nothing);
    assert_eq!(open.moved(cancel), Outcome::Changed);
    assert!(!open.message.shown.state(YES).pressed);
    assert_eq!(open.moved(ok), Outcome::Changed);
    assert_eq!(open.release(), Outcome::Closed(YES));
    // Released off it: up, no click.
    assert_eq!(open.press(cancel), Outcome::Changed);
    assert_eq!(open.moved(ok), Outcome::Changed);
    assert_eq!(open.release(), Outcome::Changed);
    assert!(!open.message.shown.state(NO).pressed);
    // Its key: down, then its key's release clicks; another key's release does nothing.
    assert_eq!(open.key(Key::Cancel, "Escape"), Outcome::Changed);
    assert!(open.message.shown.state(NO).pressed);
    assert_eq!(open.moved(ok), Outcome::Nothing);
    assert_eq!(open.release(), Outcome::Nothing);
    assert_eq!(open.key_up("Return"), Outcome::Nothing);
    assert_eq!(open.key_up("Escape"), Outcome::Closed(NO));
    // The OK by whatever key gives it.
    assert_eq!(open.key(Key::Ok, "Space"), Outcome::Changed);
    assert!(open.message.shown.state(YES).pressed);
    assert_eq!(open.key_up("Return"), Outcome::Nothing);
    assert_eq!(open.key_up("Space"), Outcome::Closed(YES));
    // A box of OK alone: no Cancel; its OK by OK's key.
    let alone = layout(&assets(), (&medium, &small), "x", Style::Ok, None, 0);
    let mut open = Open::new(alone);
    assert_eq!(open.key(Key::Cancel, "Escape"), Outcome::Nothing);
    assert_eq!(open.key(Key::Ok, "Return"), Outcome::Changed);
    assert_eq!(open.key_up("Return"), Outcome::Closed(OK));
}

#[test]
fn the_layout_section_reads_from_toml() {
    let section: MessageLayout = toml::from_str(
        r#"
        background = "diboxbck.pcx"
        frame = "dialgbox.def"
        button_back = "Box64x30.pcx"
        ok = "iOkay.def"
        cancel = "iCancel.def"
        font = "medfont.fnt"
        caption_font = "smalfont.fnt"
        font_palette = "game.pal"
        spell_pictures = "spellScr.def"
        cursor = "crdeflt.def"
        "#,
    )
    .unwrap();
    assert_eq!(section, assets());
    assert_eq!(i32_of(i64::MAX), i32::MAX);
}
