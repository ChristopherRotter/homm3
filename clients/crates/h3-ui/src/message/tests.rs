//! Origin: `h3_view::message`'s tests (E17 M5 T4a, T8b2, M6 T5c, M7 T1b, M7 T2); the pictures'
//! (E24 M4 T2a2) through [`told`] and [`pictures`].

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
    }
}

fn boxed(text: &str, style: Style, bugs: Bugs) -> MessageBox {
    layout(&assets(), &font(8, 16), text, style, (3, bugs))
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

/// The background's tiles, as (id, corner, size).
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
    let m = boxed("x", Style::Ok, Bugs::Original);
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
    let fixed = boxed("x", Style::Ok, Bugs::Fixed);
    let centred = label(&fixed, TEXT);
    assert_eq!((centred.at, centred.size), ([25, 31], [206, 16]));
    assert_eq!(tiles(&fixed), tiles(&m));
    assert_eq!(pieces(&fixed), pieces(&m));
    assert_eq!(
        (text.colour, text.align, text.font.as_str()),
        (1, 1, "medfont.fnt")
    );
    assert_eq!(m.shown.texts[&TEXT], "x");
    // The OK alone, centred; its back one out.
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
    let m = boxed("x", Style::OkCancel, Bugs::Original);
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
    // 100 words of 4 glyphs (32 wide, a space 8). At 256: 6 words a line, 17 lines, 272 high: too
    // high, so 384: 9 words a line (352), 12 lines, 192 high, 576 <= 768: wrapped there.
    let text = vec!["xxxx"; 100].join(" ");
    let m = boxed(&text, Style::Ok, Bugs::Original);
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
    let m = boxed(&text, Style::Ok, Bugs::Original);
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
    // 600 words: at 578 still too high, so wrapped at 578 (14 words a line), 200 high at most.
    let text = vec!["xxxx"; 600].join(" ");
    let m = boxed(&text, Style::Ok, Bugs::Original);
    assert_eq!(label(&m, TEXT).size, [590, 200]);
    assert_eq!(m.dialog.shadow, Some([640, 320]));
    // Wider than high enough: wrapped at the width it reached (3 x height above 2 x width).
    let text = ["xxxxxxxxxxxxxxx"; 5].join(" ");
    let m = boxed(&text, Style::Ok, Bugs::Original);
    // At 256: 2 words a line (248 wide), 3 lines, 48 high: 144 <= 512, wrapped; 298 rounded up
    // to 320.
    assert_eq!(label(&m, TEXT).size, [270, 48]);
    // A text whose lines fit at its widest word: its widest line.
    let m = boxed("xxxx\nxx", Style::Ok, Bugs::Original);
    assert_eq!(label(&m, TEXT).size, [206, 32]);
    // 7 lines of two such words (248 wide, 112 high): a box 320 x 256, a strip of tile at the
    // right, none at the bottom.
    let text = ["xxxxxxxxxxxxxxx"; 14].join(" ");
    let m = boxed(&text, Style::Ok, Bugs::Original);
    assert_eq!(m.dialog.shadow, Some([320, 256]));
    assert_eq!(
        tiles(&m),
        [(200, [0, 0], [256, 256]), (201, [256, 0], [64, 256])]
    );
}

#[test]
fn a_box_stays_on_the_screen_whatever_its_text() {
    // One word of 10 000 glyphs (a server's name): broken at 578, 200 high; its frame's ids 200 to
    // 231, below the text's and the buttons'.
    let word = "x".repeat(10_000);
    let m = boxed(&word, Style::Ok, Bugs::Original);
    assert_eq!(m.dialog.shadow, Some([640, 320]));
    assert_eq!(m.dialog.place, [80, 140]);
    assert_eq!(label(&m, TEXT).size, [590, 200]);
    let frame: Vec<u16> = m
        .dialog
        .items
        .iter()
        .map(Item::id)
        .filter(|&id| !matches!(id, TEXT | OK | 0x7866))
        .collect();
    assert_eq!(frame, (200..232).collect::<Vec<_>>());
    assert_eq!(i32_of(i64::MAX), i32::MAX);
}

#[test]
fn a_button_clicks_when_released_over_it_or_by_its_key() {
    let m = boxed("x", Style::OkCancel, Bugs::Original);
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
    // Pressed: down, its own loop running; off it: up; back on: down; released over it: clicked.
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
    let mut open = Open::new(boxed("x", Style::Ok, Bugs::Original));
    assert_eq!(open.key(Key::Cancel, "Escape"), Outcome::Nothing);
    assert_eq!(open.key(Key::Ok, "Return"), Outcome::Changed);
    assert_eq!(open.key_up("Return"), Outcome::Closed(OK));
}

#[test]
fn a_button_let_go_is_drawn_up_without_its_click() {
    let m = boxed("x", Style::OkCancel, Bugs::Original);
    let [x, y] = m.dialog.place;
    let mut open = Open::new(m);
    // Nothing held: nothing to let go.
    assert_eq!(open.let_go(None), Outcome::Nothing);
    // Held by its key: let go by that key only.
    assert_eq!(open.key(Key::Ok, "Return"), Outcome::Changed);
    assert_eq!(open.let_go(None), Outcome::Nothing);
    assert_eq!(open.let_go(Some("Return")), Outcome::Changed);
    assert!(!open.message.shown.state(YES).pressed);
    assert_eq!(open.key_up("Return"), Outcome::Nothing);
    // Held by the mouse: let go by none.
    assert_eq!(open.press([x + 60, y + 70]), Outcome::Changed);
    assert_eq!(open.let_go(None), Outcome::Changed);
    assert!(!open.holds_mouse());
    assert_eq!(open.release(), Outcome::Nothing);
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
        "#,
    )
    .unwrap();
    assert_eq!(section, assets());
    assert!(toml::from_str::<MessageLayout>("background = 1").is_err());
}

/// A spell's picture 83 × 61 (a slot 85 × 63) with its caption.
fn spell(frame: u32, caption: &str) -> Picture {
    Picture {
        def: "spellScr.def".to_string(),
        frame,
        size: [83, 61],
        caption: caption.to_string(),
    }
}

/// The box of [`told`] with `pictures` in the profile `bugs`.
fn told_with(text: &str, pictures: &[Picture], bugs: Bugs) -> MessageBox {
    let medium = font(8, 16);
    let small = font(5, 10);
    told(
        &assets(),
        (&medium, &small),
        text,
        Style::Ok,
        pictures,
        (0, bugs),
    )
}

/// The DEF item of slot `slot`.
fn picture_def(message: &MessageBox, slot: usize) -> &Def {
    match message.dialog.item(slot_ids(slot)[0]) {
        Some(Item::Def(d)) => d,
        _ => panic!("no picture in slot {slot}"),
    }
}

#[test]
fn a_picture_and_its_caption_sit_above_the_buttons() {
    let m = told_with("x", &[spell(9, "View Air")], Bugs::Original);
    // 16 + 60 + 20 + 63 + 10 + 50 = 219, rounded up to 256.
    assert_eq!(m.dialog.shadow, Some([256, 256]));
    assert_eq!(label(&m, TEXT).at, [20, 30 + (256 - 219) / 2]);
    // The slot 85 x 63, centred, its caption (85 wide: the picture's) 30 above the buttons' room.
    let def = picture_def(&m, 0);
    assert_eq!((def.at, def.size, def.frame), ([85, 103], [85, 63], 9));
    let caption = label(&m, slot_ids(0)[1]);
    assert_eq!((caption.at, caption.size), ([85, 166], [85, 10]));
    assert_eq!(caption.font, "smalfont.fnt");
    assert_eq!(m.shown.texts[&slot_ids(0)[1]], "View Air");
    // The picture keeps its own colours.
    assert!(!m.shown.players.contains_key(&slot_ids(0)[0]));
    // The client's profile centres the text alone (CLT-Q28 (1)); the picture where it was.
    let fixed = told_with("x", &[spell(9, "View Air")], Bugs::Fixed);
    assert_eq!(label(&fixed, TEXT).at, [25, 30 + (256 - 219) / 2]);
    assert_eq!(picture_def(&fixed, 0).at, [85, 103]);
    // Three at most in a box of `told`, in one row; a fourth is not shown.
    let four: Vec<Picture> = (0..4).map(|i| spell(i, "S")).collect();
    let m = told_with("x", &four, Bugs::Original);
    assert!(m.dialog.item(slot_ids(2)[0]).is_some());
    assert!(m.dialog.item(slot_ids(3)[0]).is_none());
    assert_eq!(slot_ids(0), [PICTURES, PICTURES + 1]);
    assert_eq!(CHOICE, 0x7809);
}

#[test]
fn a_caption_wider_than_its_picture_is_centred_on_it() {
    // A slot 85 wide, a caption of one word 112 wide cut to 110 (one line at 110).
    let mut picture = spell(0, &"x".repeat(22));
    let m = told_with("x", core::slice::from_ref(&picture), Bugs::Original);
    // The row (110 + 40) under the least width: 256.
    assert_eq!(m.dialog.shadow, Some([256, 256]));
    // The slot at (256 - 85) / 2; the caption (85 - 110) / 2 = -12 from it (C division).
    let caption = label(&m, slot_ids(0)[1]);
    assert_eq!((caption.at, caption.size), ([73, 166], [110, 10]));
    // A picture wider than the text: the row makes the box (252 + 40 rounded up to 320), the
    // caption still at most 110.
    picture.size = [250, 61];
    let m = told_with("x", &[picture], Bugs::Original);
    assert_eq!(m.dialog.shadow, Some([320, 256]));
    assert_eq!(label(&m, slot_ids(0)[1]).at, [34 + 71, 166]);
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

/// `n` spells' pictures (83 × 61, slots 85 × 63), captions of two glyphs.
fn spells(n: u32) -> Vec<Picture> {
    (0..n).map(|i| spell(i, &alloc::format!("S{i}"))).collect()
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
    // One: as the one picture of `told`, the text at x 25 (the call's corner: centred).
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

/// A choice's box (E24 M4 T2b2 part B) of `style` with two spells and the "or".
fn choice_of(style: Style, or: Option<&str>) -> MessageBox {
    let medium = font(8, 16);
    let small = font(5, 10);
    choice(
        &assets(),
        (&medium, &small),
        "x",
        style,
        &[spell(1, "A"), spell(2, "B")],
        or,
        (0, Bugs::Original),
    )
}

/// The button item `id`'s corner.
fn button_at(message: &MessageBox, id: u16) -> [i32; 2] {
    match message.dialog.item(id) {
        Some(Item::Button(b)) => b.at,
        _ => panic!("no button {id}"),
    }
}

#[test]
fn a_choices_box_has_its_frames_hidden_and_its_ok_disabled() {
    assert!(Style::Choose.chooses() && Style::ChooseCancel.chooses());
    assert!(!Style::Ok.chooses() && !Style::OkCancel.chooses());
    // Style 7: the OK alone as style 1's, disabled; each picture's frame one pixel out, hidden.
    let m = choice_of(Style::Choose, Some("or"));
    assert_eq!(
        (m.dialog.place, m.dialog.shadow),
        ([272, 172], Some([256, 256]))
    );
    assert_eq!(button_at(&m, OK), [96, 195]);
    assert!(m.dialog.item(CANCEL).is_none() && m.dialog.item(0x7866).is_some());
    assert!(m.shown.state(OK).disabled);
    let frames: Vec<(u16, [i32; 2], [i32; 2])> = m
        .dialog
        .items
        .iter()
        .filter_map(|i| match i {
            Item::Area(a) => Some((a.id, a.at, a.size)),
            _ => None,
        })
        .collect();
    assert_eq!(
        frames,
        [
            (CHOICE, [22, 102], [87, 65]),
            (CHOICE + 1, [147, 102], [87, 65])
        ]
    );
    assert!(m.shown.state(CHOICE).hidden && m.shown.state(CHOICE + 1).hidden);
    // The "or" as wide as its text, a line high, centred, a line above the first caption.
    let or = label(&m, OR);
    assert_eq!(
        (or.at, or.size, or.font.as_str()),
        ([120, 150], [16, 16], "medfont.fnt")
    );
    assert_eq!(m.shown.texts[&OR], "or");
    // Style 10: the Cancel first (0x7801, its back 0x7865), then the OK, placed as style 2's.
    let m = choice_of(Style::ChooseCancel, None);
    assert_eq!(
        (button_at(&m, CANCEL), button_at(&m, OK)),
        ([137, 195], [55, 195])
    );
    let ids: Vec<u16> = m.dialog.items.iter().map(Item::id).collect();
    let at = |id: u16| ids.iter().position(|&i| i == id).unwrap();
    assert!(at(0x7865) < at(CANCEL) && at(CANCEL) < at(0x7866) && at(0x7866) < at(OK));
    assert!(m.dialog.item(OR).is_none() && m.shown.state(OK).disabled);
    // Another style: no frames, the OK enabled; an "or" without pictures: none.
    let m = choice_of(Style::Ok, Some("or"));
    assert!(m.dialog.item(CHOICE).is_none() && !m.shown.state(OK).disabled);
    assert!(m.dialog.item(OR).is_some());
    let bare = choice(
        &assets(),
        (&font(8, 16), &font(5, 10)),
        "x",
        Style::Choose,
        &[],
        Some("or"),
        (0, Bugs::Original),
    );
    assert!(bare.dialog.item(OR).is_none());
}

#[test]
fn a_click_on_a_picture_chooses_it_and_enables_the_ok() {
    let mut open = Open::new(choice_of(Style::ChooseCancel, None));
    let [x, y] = open.message.dialog.place;
    let first = [x + 32, y + 112];
    let second = [x + 157, y + 112];
    let ok = [x + 60, y + 200];
    assert_eq!(open.chosen(), None);
    assert!(open.message.frame_draws("game.pal").is_empty());
    // The disabled OK takes no press and no key.
    assert_eq!(open.press(ok), Outcome::Nothing);
    assert_eq!(open.key(Key::Ok, "Return"), Outcome::Nothing);
    // The first chosen: its frame shown, the OK enabled; drawn as its outline in entry 45.
    assert_eq!(open.press(first), Outcome::Changed);
    assert_eq!(open.chosen(), Some(CHOICE));
    let shown = &open.message.shown;
    assert!(!shown.state(CHOICE).hidden && shown.state(CHOICE + 1).hidden);
    assert!(!shown.state(OK).disabled);
    let fill = |rect| Draw::Fill {
        rect,
        palette: "game.pal".to_string(),
        entry: FRAME_ENTRY,
    };
    assert_eq!(
        open.message.frame_draws("game.pal"),
        [
            fill([294, 274, 87, 1]),
            fill([294, 338, 87, 1]),
            fill([294, 275, 1, 63]),
            fill([380, 275, 1, 63]),
        ]
    );
    assert_eq!(FRAME_ENTRY, 45);
    // The second: the first hidden again.
    assert_eq!(open.press(second), Outcome::Changed);
    assert_eq!(open.chosen(), Some(CHOICE + 1));
    assert!(open.message.shown.state(CHOICE).hidden);
    assert_eq!(open.message.frame_draws("game.pal").len(), 4);
    // The OK's press and release close the box; its key too.
    assert_eq!(open.press(ok), Outcome::Changed);
    assert_eq!(open.release(), Outcome::Closed(OK));
    assert_eq!(open.key(Key::Ok, "Return"), Outcome::Changed);
    assert_eq!(open.key_up("Return"), Outcome::Closed(OK));
    // Escape the Cancel (0x7801), the OK's disabling kept while it is held.
    let mut open = Open::new(choice_of(Style::ChooseCancel, None));
    assert_eq!(open.key(Key::Cancel, "Escape"), Outcome::Changed);
    assert!(open.message.shown.state(CANCEL).pressed && open.message.shown.state(OK).disabled);
    assert_eq!(open.key_up("Escape"), Outcome::Closed(CANCEL));
}

#[test]
fn only_the_first_two_frames_choose() {
    // A third picture's frame does nothing (`dialog_default_proc` handles two); another area
    // there neither.
    let three = [spell(1, "A"), spell(2, "B"), spell(3, "C")];
    let m = choice(
        &assets(),
        (&font(8, 16), &font(5, 10)),
        "x",
        Style::Choose,
        &three,
        None,
        (0, Bugs::Original),
    );
    let Some(Item::Area(third)) = m.dialog.item(CHOICE + 2).cloned() else {
        panic!("no third frame");
    };
    let [x, y] = m.dialog.place;
    let point = [x + third.at[0] + 5, y + third.at[1] + 5];
    let mut open = Open::new(m);
    assert_eq!(open.press(point), Outcome::Nothing);
    assert_eq!(open.chosen(), None);
    // A frame shown by its state alone draws; the others' areas none.
    open.message.dialog.items.push(Item::Area(Area {
        id: 5,
        at: [0, 0],
        size: [10, 10],
        hint: None,
    }));
    assert_eq!(open.press([x + 2, y + 2]), Outcome::Nothing);
    open.message
        .shown
        .states
        .insert(CHOICE + 2, State::default());
    assert_eq!(open.message.frame_draws("game.pal").len(), 4);
    // A box of one picture: chosen, no other to hide.
    let one = choice(
        &assets(),
        (&font(8, 16), &font(5, 10)),
        "x",
        Style::Choose,
        &[spell(1, "A")],
        None,
        (0, Bugs::Original),
    );
    let Some(Item::Area(frame)) = one.dialog.item(CHOICE).cloned() else {
        panic!("no frame");
    };
    let [x, y] = one.dialog.place;
    let mut open = Open::new(one);
    let point = [x + frame.at[0] + 5, y + frame.at[1] + 5];
    assert_eq!(open.press(point), Outcome::Changed);
    assert_eq!(open.chosen(), Some(CHOICE));
    assert!(!open.message.shown.states.contains_key(&(CHOICE + 1)));
}
