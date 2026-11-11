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
        resources: "resour82.def".to_string(),
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
    let m = layout(
        &assets(),
        (&medium, &small),
        "x",
        Style::Ok,
        None,
        (3, Bugs::Original),
    );
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
    // The client's centred on the box (CLT-Q28 (1)): 25 in, the same size; nothing else moves.
    let fixed = layout(
        &assets(),
        (&medium, &small),
        "x",
        Style::Ok,
        None,
        (3, Bugs::Fixed),
    );
    let centred = label(&fixed, TEXT);
    assert_eq!((centred.at, centred.size), ([25, 31], [206, 16]));
    assert_eq!(2 * centred.at[0] + centred.size[0], 256);
    assert_eq!(tiles(&fixed), tiles(&m));
    assert_eq!(pieces(&fixed), pieces(&m));
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
    let m = layout(
        &assets(),
        (&medium, &small),
        "x",
        Style::OkCancel,
        None,
        (0, Bugs::Original),
    );
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
    let m = layout(
        &assets(),
        (&medium, &small),
        &text,
        Style::Ok,
        None,
        (0, Bugs::Original),
    );
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
    let m = layout(
        &assets(),
        (&medium, &small),
        &text,
        Style::Ok,
        None,
        (0, Bugs::Original),
    );
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
    let m = layout(
        &assets(),
        (&medium, &small),
        &text,
        Style::Ok,
        None,
        (0, Bugs::Original),
    );
    let t = label(&m, TEXT);
    assert_eq!(t.size, [590, 200]);
    assert_eq!(m.dialog.shadow, Some([640, 320]));
    // Wider than high enough: wrapped at the width it reached (3 x height above 2 x width).
    let text = ["xxxxxxxxxxxxxxx"; 5].join(" ");
    let m = layout(
        &assets(),
        (&medium, &small),
        &text,
        Style::Ok,
        None,
        (0, Bugs::Original),
    );
    // At 256: 2 words a line (248 wide), 3 lines, 48 high: 144 <= 512, wrapped; 298 rounded up
    // to 320.
    assert_eq!(label(&m, TEXT).size, [270, 48]);
    // A text whose lines fit at its widest word: its widest line.
    let m = layout(
        &assets(),
        (&medium, &small),
        "xxxx\nxx",
        Style::Ok,
        None,
        (0, Bugs::Original),
    );
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
        (0, Bugs::Original),
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
fn a_caption_wider_than_its_picture_is_centred_on_it() {
    let medium = font(8, 16);
    let small = font(5, 10);
    // A slot 85 wide, a caption of one word 112 wide cut to 110 (one line at 110).
    let mut picture = Picture {
        def: "spellScr.def".to_string(),
        frame: 0,
        size: [83, 61],
        caption: "x".repeat(22),
    };
    let m = layout(
        &assets(),
        (&medium, &small),
        "x",
        Style::Ok,
        Some(&picture),
        (0, Bugs::Original),
    );
    // The row (110 + 40) under the least width: 256.
    assert_eq!(m.dialog.shadow, Some([256, 256]));
    // The slot at (256 - 85) / 2; the caption (85 - 110) / 2 = -12 from it (C division).
    let caption = label(&m, PICTURE);
    assert_eq!((caption.at, caption.size), ([73, 166], [110, 10]));
    // A picture wider than the text: the row makes the box (252 + 40 rounded up to 320), the
    // caption still at most 110.
    picture.size = [250, 61];
    let m = layout(
        &assets(),
        (&medium, &small),
        "x",
        Style::Ok,
        Some(&picture),
        (0, Bugs::Original),
    );
    assert_eq!(m.dialog.shadow, Some([320, 256]));
    assert_eq!(label(&m, PICTURE).at, [34 + 71, 166]);
}

/// The ids of the frame's items: every item's but the text's, the buttons' and the picture's.
fn frame_ids(message: &MessageBox) -> Vec<u16> {
    message
        .dialog
        .items
        .iter()
        .map(Item::id)
        .filter(|&id| !matches!(id, TEXT | OK | 0x7866 | PICTURE))
        .collect()
}

#[test]
fn a_box_stays_on_the_screen_whatever_its_text() {
    let medium = font(8, 16);
    let small = font(5, 10);
    // One word of 10 000 glyphs (a server's name): broken at 578 (73 glyphs a line, the long
    // word's lost glyph; its widest line the last, 72), 200 high.
    let word = "x".repeat(10_000);
    let m = layout(
        &assets(),
        (&medium, &small),
        &word,
        Style::Ok,
        None,
        (0, Bugs::Original),
    );
    assert_eq!(m.dialog.shadow, Some([640, 320]));
    assert_eq!(m.dialog.place, [80, 140]);
    assert_eq!(label(&m, TEXT).size, [590, 200]);
    assert_eq!(frame_ids(&m), (200..232).collect::<Vec<_>>());
    // A picture and a caption past the screen: the box at 768 x 576, its frame's ids 200 to 246.
    let picture = Picture {
        def: "spellScr.def".to_string(),
        frame: 0,
        size: [2_000, 61],
        caption: ["xxxxx"; 1_000].join(" "),
    };
    let m = layout(
        &assets(),
        (&medium, &small),
        &word,
        Style::Ok,
        Some(&picture),
        (0, Bugs::Original),
    );
    assert_eq!(m.dialog.shadow, Some([768, 576]));
    assert_eq!(m.dialog.place, [16, 12]);
    assert_eq!(label(&m, TEXT).size, [718, 200]);
    assert_eq!(frame_ids(&m), (200..247).collect::<Vec<_>>());
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
    let m = layout(
        &assets(),
        (&medium, &small),
        "x",
        Style::OkCancel,
        None,
        (0, Bugs::Original),
    );
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
    // Pressed: down, its own loop running (E17 M7 T2); off it: up; back on: down; released over
    // it: clicked.
    assert!(!open.holds_mouse());
    assert_eq!(open.press(ok), Outcome::Changed);
    assert!(open.message.shown.state(YES).pressed && open.holds_mouse());
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
    // Its key: down (no loop of its own), then its key's release clicks; another key's release
    // does nothing.
    assert_eq!(open.key(Key::Cancel, "Escape"), Outcome::Changed);
    assert!(open.message.shown.state(NO).pressed && !open.holds_mouse());
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
    let alone = layout(
        &assets(),
        (&medium, &small),
        "x",
        Style::Ok,
        None,
        (0, Bugs::Original),
    );
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
        resources = "resour82.def"
        cursor = "crdeflt.def"
        "#,
    )
    .unwrap();
    assert_eq!(section, assets());
    assert_eq!(i32_of(i64::MAX), i32::MAX);
}

#[test]
fn the_frame_takes_the_acting_players_colours_in_the_clients_profile() {
    // CLT-Q28 (2): the original's the viewing player's, the client's the acting side's player.
    assert_eq!(framed(Bugs::Original, 0, Some(1)), 0);
    assert_eq!(framed(Bugs::Fixed, 0, Some(1)), 1);
    assert_eq!(framed(Bugs::Fixed, 2, None), 2);
}

#[test]
fn a_popup_has_no_buttons_and_a_narrower_least_box() {
    // Style 4 (E17 M6 T2a2b1): at least 128 wide, no buttons nor their room.
    let medium = font(8, 16);
    let small = font(5, 10);
    let m = layout(
        &assets(),
        (&medium, &small),
        "x",
        Style::Popup,
        None,
        (3, Bugs::Original),
    );
    // 128 x 128 (16 + 60 = 76, rounded up to 128), centred.
    assert_eq!(m.dialog.shadow, Some([128, 128]));
    assert_eq!(m.dialog.place, [336, 236]);
    assert!(!m.dialog.items.iter().any(|i| matches!(i, Item::Button(_))));
    // The text centred in the height left by no buttons: 30 + (128 - 76) / 2.
    assert_eq!(label(&m, TEXT).at[1], 56);
}

/// `n` spells' pictures (83 × 61, slots 85 × 63), captions of two glyphs.
fn spells(n: u32) -> Vec<Picture> {
    (0..n)
        .map(|i| Picture {
            def: "spellScr.def".to_string(),
            frame: i,
            size: [83, 61],
            caption: alloc::format!("S{i}"),
        })
        .collect()
}

/// Each picture's DEF item's corner and the box's size, in slot order.
fn corners(message: &MessageBox) -> (Vec<[i32; 2]>, [i32; 2]) {
    let defs = (0..SLOTS)
        .filter_map(|slot| match message.dialog.item(slot_ids(slot)[0]) {
            Some(Item::Def(d)) => Some(d.at),
            _ => None,
        })
        .collect();
    (defs, message.dialog.shadow.unwrap())
}

/// The box of `pictures` in the profile `bugs`.
fn in_profile(pictures: &[Picture], bugs: Bugs) -> MessageBox {
    let medium = font(8, 16);
    let small = font(5, 10);
    super::pictures(&assets(), (&medium, &small), "x", pictures, (2, bugs))
}

/// The box of `pictures`, the same in both profiles (CLT-Q77: they part only where a caption is
/// wider than its picture in a row of three or four, or a second row's captions are higher than
/// the first's).
fn with_pictures(pictures: &[Picture]) -> MessageBox {
    let original = in_profile(pictures, Bugs::Original);
    assert_eq!(in_profile(pictures, Bugs::Fixed), original);
    original
}

#[test]
fn one_to_four_pictures_make_one_row_placed_by_its_count() {
    // One: as the one picture of `layout`, the text at x 25 (the call's corner: centred).
    let one = with_pictures(&spells(1));
    assert_eq!(corners(&one), (vec![[85, 103]], [256, 256]));
    assert_eq!(label(&one, TEXT).at, [25, 30 + (256 - 219) / 2]);
    // Two: the row 2 × 85 + 40 + 40 = 250 wide, a box of 256; the pair split around the centre
    // by the gap (the overhang 0 and 40).
    assert_eq!(
        corners(&with_pictures(&spells(2))).0,
        [[23, 103], [148, 103]]
    );
    // Three: 375, a box of 384; the middle one centred, 40 beside it.
    assert_eq!(
        corners(&with_pictures(&spells(3))),
        (vec![[24, 103], [149, 103], [274, 103]], [384, 256])
    );
    // Four: 500, a box of 512.
    assert_eq!(
        corners(&with_pictures(&spells(4))),
        (
            vec![[26, 103], [151, 103], [276, 103], [401, 103]],
            [512, 256]
        )
    );
}

#[test]
fn five_to_eight_pictures_make_two_rows_the_first_the_larger() {
    // Five: 3 + 2, the box 384 × 320 (16 + 60 + 20 + 73 + 20 + 73 + 50 = 312); the first row's
    // captions raised by the second row's 20 + 73 above the buttons' room's 30.
    let five = with_pictures(&spells(5));
    assert_eq!(
        corners(&five),
        (
            vec![[24, 74], [149, 74], [274, 74], [87, 167], [212, 167]],
            [384, 320]
        )
    );
    assert_eq!(label(&five, TEXT).at, [25, 34]);
    // Each caption under its picture, centred on it (85 wide: the picture's).
    let caption = label(&five, slot_ids(3)[1]);
    assert_eq!((caption.at, caption.size), ([87, 230], [85, 10]));
    assert_eq!(five.shown.texts[&slot_ids(4)[1]], "S4");
    // Six: 3 + 3; seven: 4 + 3; eight: 4 + 4.
    let (six, _) = corners(&with_pictures(&spells(6)));
    assert_eq!(six[3..], [[24, 167], [149, 167], [274, 167]]);
    let (seven, size) = corners(&with_pictures(&spells(7)));
    assert_eq!(size, [512, 320]);
    assert_eq!(seven[..4], [[26, 74], [151, 74], [276, 74], [401, 74]]);
    assert_eq!(seven[4..], [[88, 167], [213, 167], [338, 167]]);
    let (eight, _) = corners(&with_pictures(&spells(8)));
    assert_eq!(eight[4..], [[26, 167], [151, 167], [276, 167], [401, 167]]);
    // A ninth is not shown (eight slots).
    let (nine, _) = corners(&with_pictures(&spells(9)));
    assert_eq!(nine, eight);
    // The pictures keep their own colours; the frame's pieces take the player's.
    let m = with_pictures(&spells(8));
    assert!(m.shown.players.keys().all(|&id| id < PICTURES));
    assert!(m.shown.players.values().all(|&p| p == 2));
}

/// `n` pictures 38 × 18 (slots 40 × 20) with captions of one word 72 wide: `widest` 72, the
/// overhang 32.
fn wide_captions(n: u32) -> Vec<Picture> {
    (0..n)
        .map(|i| Picture {
            def: "spellScr.def".to_string(),
            frame: i,
            size: [38, 18],
            caption: "x".repeat(14),
        })
        .collect()
}

/// Each picture's x in `message`.
fn xs(message: &MessageBox) -> Vec<i32> {
    corners(message).0.iter().map(|at| at[0]).collect()
}

#[test]
fn wide_captions_spread_the_pictures_by_their_overhang() {
    // Four: the row 4 × 72 + 3 × 72 + 40 = 544, a box of 576 in both profiles.
    let original = in_profile(&wide_captions(4), Bugs::Original);
    let fixed = in_profile(&wide_captions(4), Bugs::Fixed);
    assert_eq!(corners(&original).1, [576, 192]);
    assert_eq!(corners(&fixed).1, [576, 192]);
    // The middle two the gap with the overhang apart; the executable's outer ones 40 from them
    // (the captions 8 apart there), the client's the gap too (CLT-Q77 (b): 40 apart).
    assert_eq!(xs(&original), [132, 212, 324, 404]);
    assert_eq!(xs(&fixed), [100, 212, 324, 436]);
    // The captions centred on their pictures, past them by half the overhang.
    let caption = label(&original, slot_ids(0)[1]);
    assert_eq!((caption.at[0], caption.size), (116, [72, 10]));
    assert_eq!(label(&fixed, slot_ids(0)[1]).at[0], 84);
    // Three: 3 × 72 + 2 × 72 + 40 = 400, a box of 448; the middle one centred, the executable's
    // others 40 from it (0x4F6129: the overhang left out), the client's the gap.
    let original = in_profile(&wide_captions(3), Bugs::Original);
    let fixed = in_profile(&wide_captions(3), Bugs::Fixed);
    assert_eq!(corners(&original).1, [448, 192]);
    assert_eq!(corners(&fixed).1, [448, 192]);
    assert_eq!(xs(&original), [124, 204, 284]);
    assert_eq!(xs(&fixed), [92, 204, 316]);
    // Two: 2 × 72 + 72 + 40 = 256; the gap between them, the same in both.
    assert_eq!(xs(&with_pictures(&wide_captions(2))), [52, 164]);
}

#[test]
fn a_second_rows_heights_place_the_rows_below_the_first() {
    // Five, the second row's captions two lines high: the rows raised by the second row's
    // heights. The executable's second row's captions lowered by them (0x4F626E), so they end 20
    // above the buttons' room, its pictures 30 below the first row's captions; the client's by
    // the first row's caption height (CLT-Q77 (b)): 30 above the room, 20 below.
    let mut five = spells(5);
    for p in &mut five[3..] {
        p.caption = "xxxxxxxxxx xxxxxxxxxx".to_string();
    }
    let original = in_profile(&five, Bugs::Original);
    let (at, size) = corners(&original);
    assert_eq!(size, [384, 384]);
    assert_eq!(at[0][1], 191 - 63);
    assert_eq!(at[3][1], 294 - 63);
    let caption = label(&original, slot_ids(3)[1]);
    assert_eq!((caption.at[1], caption.size), (294, [85, 20]));
    let fixed = in_profile(&five, Bugs::Fixed);
    let (at, size) = corners(&fixed);
    assert_eq!(size, [384, 384]);
    // The first row where the executable has it, its captions 191 to 201; the second's pictures
    // from 221, its captions 284 to 304, the room starting at 384 − 50 = 334.
    assert_eq!(at[0][1], 191 - 63);
    assert_eq!(at[3][1], 201 + 20);
    let caption = label(&fixed, slot_ids(3)[1]);
    assert_eq!((caption.at[1], caption.size), (334 - 30 - 20, [85, 20]));
    // The first row's captions higher instead: the executable's second row raised by its own
    // (lower) captions, ending 40 above the room; the client's 30.
    let mut five = spells(5);
    for p in &mut five[..3] {
        p.caption = "xxxxxxxxxx xxxxxxxxxx".to_string();
    }
    let bottom = |m: &MessageBox| {
        let c = label(m, slot_ids(3)[1]);
        c.at[1] + c.size[1]
    };
    let (original, fixed) = (
        in_profile(&five, Bugs::Original),
        in_profile(&five, Bugs::Fixed),
    );
    assert_eq!(corners(&original).1, corners(&fixed).1);
    let room = corners(&fixed).1[1] - 50;
    assert_eq!([bottom(&original), bottom(&fixed)], [room - 40, room - 30]);
}

#[test]
fn an_empty_caption_has_no_item() {
    // Only a caption that is not empty (`message_box_build`'s picture items); the slot keeps
    // its place.
    let mut two = spells(2);
    two[1].caption = String::new();
    let m = with_pictures(&two);
    assert!(m.dialog.item(slot_ids(1)[1]).is_none());
    assert!(!m.shown.texts.contains_key(&slot_ids(1)[1]));
    assert_eq!(corners(&m).0, [[23, 103], [148, 103]]);
    // Without pictures: the text's box alone, an OK.
    let none = with_pictures(&[]);
    assert_eq!(corners(&none), (Vec::new(), [256, 128]));
    assert!(none.dialog.item(OK).is_some());
}
