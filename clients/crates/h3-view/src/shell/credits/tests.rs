use super::*;
use alloc::vec;
use alloc::vec::Vec;

use crate::shell::Screen;
use crate::shell::look::{self, parse_layout};
use crate::shell::menu::{Arrow, Front};

const LAYOUT: &str = include_str!("../../../../../layouts/shell.toml");

/// The shell's layout's `[credits]` (E27 M4 T2).
fn look() -> CreditsLook {
    CreditsLook {
        picture: "mainmenu.pcx".into(),
        at: [460, 0],
        window: [460, 10, 328, 580],
        font: "Credits.fnt".into(),
        font_palette: "game.pal".into(),
        colour: 0x11C,
        align: 5,
        closing_font: "smalfont.fnt".into(),
        closing_colour: 0xD,
        closing_align: 8,
        closing_below: 435,
        step: 2,
        every: 100,
    }
}

fn texts() -> Texts {
    Texts {
        text: "{Created By:}\nsomeone".into(),
        closing: "the end".into(),
    }
}

/// The installation's credits: 153 lines of `Credits.fnt`'s 31 rows (§36.1).
pub(crate) fn credits() -> Credits {
    credits_in(Bugs::Fixed)
}

/// [`credits`] in the profile `bugs`.
fn credits_in(bugs: Bugs) -> Credits {
    Credits::new(look(), texts(), (153 * 31, 31), bugs)
}

/// The text's part in `draws`: the rows it copies and where (none when it draws none).
fn part(draws: &[Draw]) -> Option<([i32; 4], [i32; 2])> {
    draws.iter().find_map(|d| match d {
        Draw::Part { part, x, y, .. } => Some((*part, [*x, *y])),
        _ => None,
    })
}

/// Whether `draws` hold the closing text.
fn closing(draws: &[Draw]) -> bool {
    draws.iter().any(|d| matches!(d, Draw::Label { .. }))
}

/// The credits after `n` steps (the clock read at the opening, then every 100 ms).
fn stepped(n: u32) -> (Credits, Did) {
    let mut c = credits();
    assert_eq!(c.clock(100), Did::default());
    let mut did = Did::default();
    for k in 1..=n {
        did = c.clock(100 + 100 * k);
    }
    (c, did)
}

#[test]
fn the_credits_open_on_the_menus_background_with_mainmenu_pcx_alone() {
    let c = credits();
    let draws = c.draws();
    assert_eq!(
        draws,
        vec![Draw::Picture {
            picture: "mainmenu.pcx".into(),
            x: 460,
            y: 0,
            player: None,
            size: None,
        }]
    );
    assert_eq!(c.context(), Context::Credits);
    let hovers = Hovers {
        main: Some(104),
        game_type: Some(101),
        arrow: Arrow::Adventure,
    };
    let opened = c.opened(hovers);
    assert_eq!(opened.hovers(), hovers);
    assert_eq!(opened.draws(), draws);
}

#[test]
fn the_text_comes_in_from_the_windows_bottom_two_rows_a_step() {
    // The first step 100 ms after the opening: the picture's first two rows at the window's
    // bottom (§36.2: `credits-scroll`'s frames).
    let (c, did) = stepped(1);
    assert_eq!(did, Did::CHANGED);
    assert_eq!(part(&c.draws()), Some(([0, 0, 328, 2], [460, 588])));
    let draws = c.draws();
    let Some(Draw::Part { item, text, .. }) = draws.get(1) else {
        panic!("no part: {draws:?}");
    };
    // The text's picture: the window's width, its lines and a line high, in its font.
    assert_eq!(item.size, [328, 153 * 31 + 31]);
    assert_eq!(
        (item.font.as_str(), item.font_palette.as_str()),
        ("Credits.fnt", "game.pal")
    );
    assert_eq!((item.colour, item.align), (0x11C, 5));
    assert_eq!(text, "{Created By:}\nsomeone");
    let (c, _) = stepped(12);
    assert_eq!(part(&c.draws()), Some(([0, 0, 328, 24], [460, 566])));
    // The 290th step shows the window whole from the picture's top.
    let (c, _) = stepped(290);
    assert_eq!(part(&c.draws()), Some(([0, 0, 328, 580], [460, 10])));
    assert!(!closing(&c.draws()));
}

#[test]
fn then_it_moves_up_until_its_end_meets_the_windows_bottom() {
    // 153 lines of 31: rows to 4 743, the window 580 high; the first row shown moves on while it
    // is under 4 163 (to 4 164: 2 082 steps).
    let (c, _) = stepped(291);
    assert_eq!(part(&c.draws()), Some(([0, 2, 328, 580], [460, 10])));
    let (c, _) = stepped(290 + 2082);
    assert_eq!(part(&c.draws()), Some(([0, 4164, 328, 580], [460, 10])));
    assert!(!closing(&c.draws()));
}

#[test]
fn then_it_leaves_at_the_top_the_closing_text_once_under_435_rows() {
    let end = 290 + 2082;
    // The first step out: 578 rows from 4 166.
    let (c, _) = stepped(end + 1);
    assert_eq!(part(&c.draws()), Some(([0, 4166, 328, 578], [460, 10])));
    // 436 rows: no closing text yet; 434 (the 73rd step out, 244.6 s): the closing text over
    // the window, its lines at the bottom.
    let (c, _) = stepped(end + 72);
    assert_eq!(part(&c.draws()).map(|p| p.0[3]), Some(436));
    assert!(!closing(&c.draws()));
    let (c, _) = stepped(end + 73);
    assert_eq!(part(&c.draws()).map(|p| p.0[3]), Some(434));
    let draws = c.draws();
    let Some(Draw::Label {
        item, x, y, text, ..
    }) = draws.last()
    else {
        panic!("no closing text: {draws:?}");
    };
    assert_eq!(([*x, *y], item.size), ([460, 10], [328, 580]));
    assert_eq!(
        (item.font.as_str(), item.colour, item.align),
        ("smalfont.fnt", 0xD, 8)
    );
    assert_eq!(text, "the end");
    // 0 rows and then −2: the closing text alone; the step after ends the credits.
    let (c, did) = stepped(end + 290);
    assert_eq!((part(&c.draws()), closing(&c.draws())), (None, true));
    assert_eq!(did, Did::CHANGED);
    let (c, did) = stepped(end + 291);
    assert_eq!((part(&c.draws()), closing(&c.draws())), (None, true));
    assert_eq!(did, Did::CHANGED);
    let (_, did) = stepped(end + 292);
    assert_eq!(
        did,
        Did {
            ended: Some(Ended::Done),
            ..Did::CHANGED
        }
    );
}

#[test]
fn a_step_is_due_100_ms_after_the_last_one_reading_at_a_time() {
    // The original's profile: the next step 100 ms after the reading that ran the last.
    let mut c = credits_in(Bugs::Original);
    assert_eq!(c.clock(u32::MAX - 50), Did::default());
    // Not yet due, then due across the clock's wrap.
    assert_eq!(c.clock(u32::MAX), Did::default());
    assert_eq!(c.clock(49), Did::CHANGED);
    assert_eq!(part(&c.draws()).map(|p| p.0[3]), Some(2));
    // A late reading runs one step; the next is due 100 ms after it.
    assert_eq!(c.clock(1_000), Did::CHANGED);
    assert_eq!(part(&c.draws()).map(|p| p.0[3]), Some(4));
    assert_eq!(c.clock(1_099), Did::default());
    assert_eq!(c.clock(1_100), Did::CHANGED);
}

/// E27 M4 T6 (the review's A2, CLT-Q129 (b)'s pattern): in the client's profile a step is due
/// 100 ms after the last one's due time, so a reading's lateness does not add up, and a late
/// window catches up a step a reading.
#[test]
fn the_client_s_steps_are_due_100_ms_after_the_last_one_s_due_time() {
    let mut c = credits_in(Bugs::Fixed);
    assert_eq!(c.clock(u32::MAX - 50), Did::default());
    assert_eq!(c.clock(u32::MAX), Did::default());
    assert_eq!(c.clock(49), Did::CHANGED);
    // A reading 9 ms late keeps the pace: the next due at 249, not 258.
    assert_eq!(c.clock(158), Did::CHANGED);
    assert_eq!(c.clock(248), Did::default());
    assert_eq!(c.clock(249), Did::CHANGED);
    assert_eq!(part(&c.draws()).map(|p| p.0[3]), Some(6));
    // Late by 650 ms: one step a reading until the pace is back (due 449, 549, ..., 1049).
    for t in 1_000..1_007 {
        assert_eq!(c.clock(t), Did::CHANGED, "{t}");
    }
    assert_eq!(c.clock(1_007), Did::default());
    assert_eq!(c.clock(1_049), Did::CHANGED);
    assert_eq!(part(&c.draws()).map(|p| p.0[3]), Some(2 * 11));
}

#[test]
fn a_key_but_f4_a_left_or_a_right_press_ends_the_credits() {
    let c = credits();
    let done = Did {
        ended: Some(Ended::Done),
        ..Did::default()
    };
    assert_eq!(c.key("F4"), Did::default());
    assert_eq!(c.key("f4"), Did::default());
    assert_eq!(c.key("Space"), done);
    assert_eq!(c.key("C"), done);
    assert_eq!(c.press(), done);
}

#[test]
fn a_short_text_still_comes_in_and_leaves() {
    // A text shorter than the window: in, no step up, out (crafted: the installation's is
    // longer).
    let mut c = Credits::new(look(), texts(), (31, 31), Bugs::Fixed);
    c.clock(0);
    let mut steps = 0;
    while c.clock((steps + 1) * 100).ended.is_none() {
        steps += 1;
    }
    assert_eq!(steps, 290 + 291);
}

#[test]
fn the_texts_lines_are_counted_at_the_windows_width_in_its_font() {
    // A font of glyphs 100 wide and 31 high: three to a line of 328, so `1 1 1 1` is two lines;
    // the text's own newline another line.
    let mut glyphs = vec![h3_assets::fnt::Glyph::default(); 256];
    for c in *b"1 " {
        glyphs[usize::from(c)].width = 100;
        glyphs[usize::from(c)].pixels = vec![0; 3100];
    }
    let font = Font {
        height: 31,
        y_offset: 0,
        glyphs,
    };
    let laid = |text: &str| {
        let texts = Texts {
            text: text.into(),
            closing: String::new(),
        };
        let c = Credits::laid(look(), texts, &font, Bugs::Fixed);
        (c.height, c.line)
    };
    assert_eq!(laid("1 1 1 1"), (62, 31));
    assert_eq!(laid("1\n1"), (62, 31));
    assert_eq!(laid("1"), (31, 31));
}

#[test]
fn the_originals_background_shows_its_keys_pixels_black_from_the_first_step() {
    // §36.3: in the original's profile only, once the first step put the saved background back.
    let mut c = Credits::new(look(), texts(), (153 * 31, 31), Bugs::Original);
    let keyed = |c: &Credits| {
        c.draws().iter().any(|d| {
            *d == Draw::Keyed {
                rect: [460, 10, 328, 580],
            }
        })
    };
    assert!(!keyed(&c));
    c.clock(0);
    c.clock(100);
    assert!(keyed(&c));
    assert!(!keyed(&c.opened(Hovers::default())));
    let (fixed, _) = stepped(1);
    assert!(!keyed(&fixed));
}

#[test]
fn the_shell_opens_them_from_the_main_menu_and_their_front_takes_the_inputs() {
    let mut menus = crate::shell::menu::tests::menus(Bugs::Fixed);
    let main = menus.front(Screen::MainMenu, None, [300, 560]).unwrap();
    let hovers = main.hovers();
    let mut front = menus
        .front(Screen::Credits, Some(main), [300, 560])
        .unwrap();
    assert!(matches!(&front, Front::Credits(_)));
    assert_eq!(front.hovers(), hovers);
    assert_eq!(front.context(), Context::Credits);
    assert_eq!(front.contexts(), vec![Context::Credits]);
    assert!(!front.typing());
    assert_eq!(front.typed('a'), Did::default());
    assert_eq!(front.question(), None);
    // The clock steps them; a move, the releases and a key's up do nothing, F4 neither.
    assert_eq!(front.clock(100), Did::default());
    assert_eq!(front.clock(200), Did::CHANGED);
    assert_eq!(front.moved([0, 0]), Did::default());
    assert_eq!(front.release(), Did::default());
    assert_eq!(front.right_release(), Did::default());
    assert_eq!(front.key_up("C"), Did::default());
    assert_eq!(front.key(None, None, "F4"), Did::default());
    let done = Some(Ended::Done);
    assert_eq!(front.key(None, None, "Escape").ended, done);
    assert_eq!(front.press([1, 1]).ended, done);
    assert_eq!(front.right([1, 1]).ended, done);
    // Their look: the main menu's background and their draws, the cursor; none on another
    // screen. Their pictures and fonts are in the start's batch.
    let layout = parse_layout(LAYOUT).unwrap();
    let shown = look::look(&layout, Screen::Credits, Some(&front)).unwrap();
    let Front::Credits(c) = &front else {
        panic!("the credits");
    };
    assert_eq!(shown.background, "GamSelBk.pcx");
    assert_eq!(shown.draws, c.draws());
    assert!(shown.cursor && shown.over.is_empty() && shown.stale.is_empty());
    assert_eq!(shown.shift, 0);
    assert_eq!(look::look(&layout, Screen::MainMenu, Some(&front)), None);
    let names: Vec<String> = look::names(&layout);
    for name in ["mainmenu.pcx", "Credits.fnt", "smalfont.fnt", "game.pal"] {
        assert!(names.iter().any(|n| n == name), "{name}");
    }
    assert_eq!(layout.credits, look());
}
