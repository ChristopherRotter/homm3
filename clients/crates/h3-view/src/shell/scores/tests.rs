//! High Scores' tests (E27 M4 T1a): the creatures a score earns, the defaults' tables, the draw of
//! both tabs, the buttons, keys and ends, the creatures' steps and the hold's rules as
//! `scores-tabs` recorded them, the original's stale edges, Reset's notice (CLT-STUB-14).

use alloc::boxed::Box;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

use h3_render::Bugs;
use h3_ui::ui::{Draw, Item};

use super::{
    CAMPAIGN, EXIT, Picture, Pictures, RESET, ROWS, Record, STANDARD, Scores, ScoresLook, Tab,
    Tables, Texts, atoi,
};
use crate::message::{self, Key, MessageBox, Style};
use crate::shell::keys::{Command, Context};
use crate::shell::look::{self, parse_layout};
use crate::shell::menu::tests::font;
use crate::shell::menu::{BOX_PLAYER, Did, Front, Hovers};
use crate::shell::{Ended, Screen};

const LAYOUT: &str = include_str!("../../../../../layouts/shell.toml");

/// The layout's look.
fn look() -> ScoresLook {
    parse_layout(LAYOUT).unwrap().high_scores
}

/// A `CampHigh.txt` of the client's own words: a header, eleven scenarios' rows (`S<i>`, `L<i>`,
/// days 10 i, score 200 − 10 i), a header, eleven campaigns' (`C<i>`, `K<i>`, days 100 i, score
/// 1000 − 75 i).
fn records() -> Vec<Vec<String>> {
    let row = |rank: usize, player: String, land: String, days: usize, score: usize| {
        vec![
            rank.to_string(),
            player,
            land,
            days.to_string(),
            score.to_string(),
        ]
    };
    let header = || vec![String::from("Rank"), String::from("Player")];
    let mut records = vec![header()];
    for i in 0..ROWS {
        records.push(row(
            i + 1,
            format!("S{i}"),
            format!("L{i}"),
            10 * i,
            200 - 10 * i,
        ));
    }
    records.push(header());
    for i in 0..ROWS {
        records.push(row(
            i + 1,
            format!("C{i}"),
            format!("K{i}"),
            100 * i,
            1000 - 75 * i,
        ));
    }
    records
}

/// Every creature of the layout's table drawn by `c<creature>.def` of 4 frames, but creature 90
/// (the scenarios' first row's and the campaigns' first row's) of 0 frames and creature 46 with
/// no picture.
fn pictures() -> Pictures {
    look()
        .creatures
        .iter()
        .filter(|&&[_, c]| c != 46)
        .map(|&[_, c]| {
            let frames = if c == 90 { 0 } else { 4 };
            (
                c,
                Picture {
                    def: format!("c{c}.def"),
                    frames,
                },
            )
        })
        .collect()
}

/// Reset's notice laid out in fonts of 8-wide glyphs.
fn notice(bugs: Bugs) -> MessageBox {
    let layout = parse_layout(LAYOUT).unwrap();
    let (medium, small) = (font(8, 16), font(5, 10));
    message::layout(
        &layout.message,
        (&medium, &small),
        "Not yet.",
        Style::Ok,
        None,
        (BOX_PLAYER, bugs),
    )
}

/// The dialog as it opens: the layout's look, headings `Rank` to `Score`, [`records`]' tables,
/// [`pictures`].
pub(crate) fn scores(bugs: Bugs) -> Scores {
    let texts = Texts {
        rank: String::from("Rank"),
        player: String::from("Player"),
        land: String::from("Land"),
        days: String::from("Days"),
        campaign: String::from("Campaign"),
        score: String::from("Score"),
    };
    let tables = Tables::defaults(&records()).unwrap();
    Scores::new(look(), texts, tables, pictures(), notice(bugs), bugs)
}

/// The middle of button `id`.
fn middle(id: u16) -> [i32; 2] {
    let look = look();
    let b = look
        .dialog
        .items
        .iter()
        .find_map(|i| match i {
            Item::Button(b) if b.id == id => Some(b.clone()),
            _ => None,
        })
        .unwrap();
    [b.at[0] + b.size[0] / 2, b.at[1] + b.size[1] / 2]
}

/// The middle of the open notice's OK.
fn ok_point(scores: &Scores) -> [i32; 2] {
    let open = scores.open.as_ref().unwrap();
    let dialog = &open.message.dialog;
    let b = dialog
        .items
        .iter()
        .find_map(|i| match i {
            Item::Button(b) if b.id == message::OK => Some(b.clone()),
            _ => None,
        })
        .unwrap();
    [
        dialog.place[0] + b.at[0] + b.size[0] / 2,
        dialog.place[1] + b.at[1] + b.size[1] / 2,
    ]
}

/// A click at `point`: a press and a release.
fn click(scores: &mut Scores, point: [i32; 2]) -> Did {
    let pressed = scores.press(point);
    pressed.with(scores.release())
}

/// The labels' texts drawn, in order.
fn texts(scores: &Scores) -> Vec<String> {
    scores
        .draws("PLAYERS.PAL")
        .into_iter()
        .filter_map(|d| match d {
            Draw::Label { text, .. } => Some(text),
            _ => None,
        })
        .collect()
}

/// The creatures drawn: their DEF, frame and top edge.
fn creatures(scores: &Scores) -> Vec<(String, u32, i32)> {
    scores
        .draws("PLAYERS.PAL")
        .into_iter()
        .filter_map(|d| match d {
            Draw::Button {
                def,
                frame,
                y,
                x: 649,
                ..
            } => Some((def, frame, y)),
            _ => None,
        })
        .collect()
}

#[test]
fn a_score_earns_the_first_pair_not_below_it_a_campaign_s_divided_by_5() {
    let look = look();
    assert_eq!(look.creatures.len(), 118);
    // The defaults' (`CampHigh.txt`): 200 Ogre (90), 1 Imp (42); a campaign's 1000 is 200.
    assert_eq!(look.creature(200, Tab::Scenarios), Some(90));
    assert_eq!(look.creature(199, Tab::Scenarios), Some(33));
    assert_eq!(look.creature(1, Tab::Scenarios), Some(42));
    assert_eq!(look.creature(1000, Tab::Campaigns), Some(90));
    assert_eq!(look.creature(200, Tab::Campaigns), Some(0));
    // A campaign's score rounded toward 0: 9 / 5 is 1, −9 / 5 is −1 (both the first pair's).
    assert_eq!(look.creature(9, Tab::Campaigns), Some(42));
    assert_eq!(look.creature(-9, Tab::Campaigns), Some(42));
    // The last pair's 32767; past it the last's (CLT-Q157, the client's).
    assert_eq!(look.creature(32_767, Tab::Scenarios), Some(13));
    assert_eq!(look.creature(40_000, Tab::Scenarios), Some(13));
    let none = ScoresLook {
        creatures: Vec::new(),
        ..look
    };
    assert_eq!(none.creature(1, Tab::Scenarios), None);
}

#[test]
fn the_defaults_are_camp_high_s_records_1_to_11_and_13_to_23() {
    let tables = Tables::defaults(&records()).unwrap();
    assert_eq!(tables.scenarios.len(), ROWS);
    assert_eq!(tables.campaigns.len(), ROWS);
    assert_eq!(
        tables.of(Tab::Scenarios)[2],
        Record {
            player: String::from("S2"),
            land: String::from("L2"),
            days: 20,
            score: 180,
        }
    );
    assert_eq!(tables.of(Tab::Campaigns)[10].score, 250);
    // A record short of a field (its player, land, days or score), or too few records: none.
    for fields in 1..5 {
        let mut short = records();
        short[5].truncate(fields);
        assert_eq!(Tables::defaults(&short), None, "{fields}");
    }
    assert_eq!(Tables::defaults(&records()[..23]), None);
}

#[test]
fn atoi_reads_c_s_way() {
    assert_eq!(atoi("200"), 200);
    assert_eq!(atoi(" \t42"), 42);
    assert_eq!(atoi("-7"), -7);
    assert_eq!(atoi("+3"), 3);
    assert_eq!(atoi("12ab"), 12);
    assert_eq!(atoi(""), 0);
    assert_eq!(atoi("x1"), 0);
    assert_eq!(atoi("4294967297"), 1);
}

#[test]
fn it_opens_on_the_scenarios_their_button_selected_their_creatures_shown() {
    let scores = scores(Bugs::Fixed);
    assert_eq!(scores.tab(), Tab::Scenarios);
    assert_eq!(scores.background(), "hiscore.pcx");
    assert_eq!(scores.context(), Context::HighScores);
    assert!(!scores.noticing());
    assert_eq!(scores.hovers(), Hovers::default());
    assert_eq!(scores.tables().scenarios[0].player, "S0");
    assert_eq!(scores.frames(Tab::Scenarios), [0; ROWS]);
    assert!(scores.shown.state(STANDARD).selected);
    assert!(!scores.shown.state(CAMPAIGN).selected);
    // The headings, then each row: rank, player, land, days, score.
    let texts = texts(&scores);
    assert_eq!(texts.len(), 5 + 5 * ROWS);
    assert_eq!(&texts[..5], ["Rank", "Player", "Land", "Days", "Score"]);
    assert_eq!(&texts[5..10], ["1", "S0", "L0", "0", "200"]);
    assert_eq!(&texts[55..], ["11", "S10", "L10", "100", "100"]);
    // The boxes: the headings at y 11, the rows from 53 by 50, each 26 high; colour 1, centred.
    let labels: Vec<(i32, i32, [i32; 2], u32, u32)> = scores
        .draws("")
        .into_iter()
        .filter_map(|d| match d {
            Draw::Label { item, x, y, .. } => Some((x, y, item.size, item.colour, item.align)),
            _ => None,
        })
        .collect();
    assert_eq!(labels[0], (88, 11, [58, 26], 1, 5));
    assert_eq!(labels[4], (601, 11, [52, 26], 1, 5));
    assert_eq!(labels[5], (88, 53, [58, 26], 1, 5));
    assert_eq!(labels[59], (601, 553, [52, 26], 1, 5));
    // The scenarios' creatures by their scores: 200 Ogre (90, its DEF of no frames), 190 46 (no
    // picture: none drawn), then 180 …
    let drawn = creatures(&scores);
    assert_eq!(drawn.len(), ROWS - 1);
    assert_eq!(drawn[0], (String::from("c90.def"), 0, 26));
    assert_eq!(drawn[1].2, 126);
    // The buttons over the texts: Exit, Reset, Campaign, Standard (its frame 3).
    let buttons: Vec<(String, u32)> = scores
        .draws("")
        .into_iter()
        .filter_map(|d| match d {
            Draw::Button { def, frame, x, .. } if x != 649 => Some((def, frame)),
            _ => None,
        })
        .collect();
    assert_eq!(
        buttons,
        [
            (String::from("HiScExt.def"), 0),
            (String::from("HiScRes.def"), 0),
            (String::from("HiScCam.def"), 0),
            (String::from("HiScSta.def"), 3),
        ]
    );
}

#[test]
fn campaign_and_standard_switch_the_tab_by_the_mouse_and_their_keys() {
    let mut scores = scores(Bugs::Fixed);
    let did = click(&mut scores, middle(CAMPAIGN));
    assert!(did.changed && did.sample);
    assert_eq!(did.ended, None);
    assert_eq!(scores.tab(), Tab::Campaigns);
    assert_eq!(scores.background(), "hiscore2.pcx");
    assert!(scores.shown.state(CAMPAIGN).selected);
    assert!(!scores.shown.state(STANDARD).selected);
    // The campaigns' headings and rows: rank, player, campaign, score.
    let texts = texts(&scores);
    assert_eq!(texts.len(), 4 + 4 * ROWS);
    assert_eq!(&texts[..4], ["Rank", "Player", "Campaign", "Score"]);
    assert_eq!(&texts[4..8], ["1", "C0", "K0", "1000"]);
    // Their creatures (1000 / 5 = 200: Ogre, 90), the scenarios' hidden.
    let drawn = creatures(&scores);
    assert_eq!(drawn.len(), ROWS);
    assert_eq!(drawn[0].0, "c90.def");
    // S presses Standard at its key down, clicks it at its key up.
    let down = scores.key(Some(Command::Dialog(STANDARD)), "S");
    assert!(down.changed && down.sample);
    assert_eq!(scores.tab(), Tab::Campaigns);
    assert!(scores.shown.state(STANDARD).pressed);
    let up = scores.key_up("S");
    assert!(up.changed);
    assert_eq!(scores.tab(), Tab::Scenarios);
    // C the campaigns again; Standard pressed again over the scenarios keeps them.
    scores.key(Some(Command::Dialog(CAMPAIGN)), "C");
    scores.key_up("C");
    assert_eq!(scores.tab(), Tab::Campaigns);
    click(&mut scores, middle(CAMPAIGN));
    assert_eq!(scores.tab(), Tab::Campaigns);
    // A key no command gives, or another dialog's command, does nothing.
    assert_eq!(scores.key(None, "X"), Did::default());
    assert_eq!(
        scores.key(Some(Command::Dialog(RESET)), "R"),
        Did::default()
    );
    assert_eq!(scores.key_up("X"), Did::default());
}

#[test]
fn exit_s_click_and_its_keys_at_their_press_end_it() {
    let done = Some(Ended::Done);
    let mut scores = scores(Bugs::Fixed);
    assert_eq!(click(&mut scores, middle(EXIT)).ended, done);
    let mut scores = super::tests::scores(Bugs::Fixed);
    assert_eq!(
        scores.key(Some(Command::Dialog(EXIT)), "Escape").ended,
        done
    );
    // A press released off a button clicks nothing.
    let mut scores = super::tests::scores(Bugs::Fixed);
    scores.press(middle(EXIT));
    assert!(scores.moved([400, 560]).changed);
    let did = scores.release();
    assert!(did.changed);
    assert_eq!(did.ended, None);
    // A press on nothing, a creature or the right button: nothing.
    assert_eq!(scores.press([400, 560]), Did::default());
    assert_eq!(scores.right(), Did::default());
    assert_eq!(scores.right_release(), Did::default());
    let creature = scores.press([681, 58]);
    assert!(!creature.changed);
    assert!(!scores.release().changed);
    // The right release ends a hold as the left's (a click over the button).
    scores.press(middle(EXIT));
    assert_eq!(scores.right_release().ended, done);
}

#[test]
fn the_creatures_step_once_more_than_200_ms_passed_on_the_tab_shown() {
    let mut scores = scores(Bugs::Fixed);
    // The first reading starts the timer; 200 ms later nothing, 210 a step.
    assert!(!scores.clock(1000));
    assert!(!scores.clock(1200));
    assert!(scores.clock(1210));
    // Row 0's DEF has no frames: back to 0 at once; row 1 has no picture; row 2 steps.
    assert_eq!(scores.frames(Tab::Scenarios)[..3], [0, 0, 1]);
    for t in [1420, 1630, 1840] {
        assert!(scores.clock(t));
    }
    // Four frames: 1, 2, 3, 0.
    assert_eq!(scores.frames(Tab::Scenarios)[2], 0);
    assert_eq!(scores.frames(Tab::Campaigns), [0; ROWS]);
    let drawn = creatures(&scores);
    assert_eq!(drawn[1], (String::from("c114.def"), 0, 126));
    assert!(scores.clock(2050));
    assert_eq!(creatures(&scores)[1].1, 1);
    // The campaigns' tab: theirs step from where they were, the scenarios' kept.
    click(&mut scores, middle(CAMPAIGN));
    assert!(scores.clock(2260));
    assert_eq!(scores.frames(Tab::Campaigns)[1], 1);
    assert_eq!(scores.frames(Tab::Scenarios)[2], 1);
}

#[test]
fn no_step_while_the_mouse_holds_a_button_its_release_runs_the_one_due() {
    let mut scores = scores(Bugs::Fixed);
    scores.clock(0);
    // `scores-tabs`' Exit: pressed at 4400 (last step 4300 here 0), held past the step's time.
    scores.press(middle(EXIT));
    assert!(!scores.clock(300));
    scores.moved([400, 560]);
    assert!(!scores.clock(400));
    // Released at 400 (off the button): the step due runs then.
    let did = scores.release();
    assert!(did.changed);
    assert_eq!(scores.frames(Tab::Scenarios)[2], 1);
    assert!(!scores.clock(500));
    assert!(scores.clock(700));
    // A release with no step due steps nothing; a key's hold does not stop the steps.
    scores.press(middle(RESET));
    scores.moved([400, 560]);
    scores.release();
    assert_eq!(scores.frames(Tab::Scenarios)[2], 2);
    scores.key(Some(Command::Dialog(STANDARD)), "S");
    assert!(scores.clock(1000));
    // The right release that ends a hold runs it too.
    scores.key_up("S");
    scores.press(middle(EXIT));
    assert!(!scores.clock(1300));
    scores.moved([400, 560]);
    assert!(scores.right_release().changed);
    assert_eq!(scores.frames(Tab::Scenarios)[2], 0);
}

#[test]
fn the_original_s_press_shows_the_button_s_box_alone_its_edges_stale() {
    let mut original = scores(Bugs::Original);
    assert_eq!(original.stale(""), Vec::new());
    original.press(middle(CAMPAIGN));
    let stale = original.stale("");
    assert_eq!(stale.len(), 1);
    assert_eq!(stale[0].rects, [[31, 313, 42, 1], [72, 114, 1, 199]]);
    assert!(matches!(
        &stale[0].draws[..],
        [Draw::Button { def, frame: 0, .. }] if def == "HiScCam.def"
    ));
    // Off the button it is up again, as last shown whole.
    original.moved([400, 560]);
    assert_eq!(original.stale(""), Vec::new());
    original.release();
    // A key's press (no hold of the mouse: the steps go on) until a step shows it whole.
    original.key(Some(Command::Dialog(STANDARD)), "S");
    assert_eq!(original.stale("").len(), 1);
    original.clock(0);
    original.clock(210);
    assert_eq!(original.stale(""), Vec::new());
    // The client's profile draws it whole.
    let mut fixed = scores(Bugs::Fixed);
    fixed.press(middle(CAMPAIGN));
    assert_eq!(fixed.stale(""), Vec::new());
}

#[test]
fn reset_shows_the_client_s_notice_until_its_ok() {
    let mut scores = scores(Bugs::Fixed);
    assert_eq!(scores.box_draws(""), Vec::new());
    let did = click(&mut scores, middle(RESET));
    assert!(did.changed);
    assert!(scores.noticing());
    assert_eq!(scores.context(), Context::Question);
    assert!(!scores.box_draws("").is_empty());
    // The dialog's inputs and steps go nowhere meanwhile.
    scores.clock(0);
    assert!(!scores.clock(1000));
    assert_eq!(
        scores.key(Some(Command::Dialog(EXIT)), "Escape"),
        Did::default()
    );
    assert_eq!(scores.moved(middle(EXIT)), Did::default());
    assert_eq!(scores.right_release(), Did::default());
    // A press of its OK released off it keeps it.
    let ok = ok_point(&scores);
    scores.press(ok);
    scores.moved([0, 0]);
    assert!(scores.release().changed);
    assert!(scores.noticing());
    // Its OK by the mouse (followed off and on, the right release ending its hold) closes it.
    assert!(scores.press(ok).changed);
    assert!(scores.moved([0, 0]).changed);
    scores.moved(ok);
    assert!(scores.right_release().changed);
    assert!(!scores.noticing());
    // By Enter, at its key up; a press off its button does nothing.
    click(&mut scores, middle(RESET));
    assert_eq!(scores.press([0, 0]), Did::default());
    assert!(
        scores
            .key(Some(Command::Message(Key::Ok)), "Return")
            .changed
    );
    assert_eq!(scores.release(), Did::default());
    assert!(scores.key_up("Return").changed);
    assert!(!scores.noticing());
}

#[test]
fn it_opens_anew_on_the_scenarios_its_newest_rows_and_the_menus_items_kept() {
    let mut scores = scores(Bugs::Fixed).with_newest(Tab::Scenarios, 1);
    click(&mut scores, middle(CAMPAIGN));
    scores.clock(0);
    scores.clock(300);
    let hovers = Hovers {
        main: Some(103),
        ..Hovers::default()
    };
    let opened = scores.opened(hovers);
    assert_eq!(opened.tab(), Tab::Scenarios);
    assert_eq!(opened.frames(Tab::Campaigns), [0; ROWS]);
    assert_eq!(opened.hovers(), hovers);
    // The newest row in colour 8, the others in 1.
    let colours: Vec<u32> = opened
        .draws("")
        .into_iter()
        .filter_map(|d| match d {
            Draw::Label { item, y, .. } if item.size[0] == 58 => Some((y, item.colour)),
            _ => None,
        })
        .map(|(_, c)| c)
        .collect();
    assert_eq!(colours[..4], [1, 1, 8, 1]);
}

#[test]
fn the_shell_opens_it_from_the_main_menu_and_its_front_takes_the_inputs() {
    let mut menus = crate::shell::menu::tests::menus(Bugs::Fixed);
    let hovers = Hovers {
        main: Some(103),
        ..Hovers::default()
    };
    let main = menus.front(Screen::MainMenu, None, [643, 304]).unwrap();
    let mut front = menus
        .front(Screen::HighScores, Some(main), [643, 304])
        .unwrap();
    assert!(matches!(&front, Front::Scores(_)));
    assert_eq!(front.hovers().arrow, hovers.arrow);
    assert_eq!(front.context(), Context::HighScores);
    assert_eq!(front.contexts(), vec![Context::HighScores]);
    assert!(!front.typing());
    assert_eq!(front.typed('a'), Did::default());
    front.clock(0);
    assert!(front.clock(210));
    assert!(front.moved([0, 0]) == Did::default());
    assert!(front.press(middle(CAMPAIGN)).changed);
    assert!(front.release().changed);
    assert_eq!(front.right([0, 0]), Did::default());
    assert_eq!(front.right_release(), Did::default());
    assert!(
        front
            .key(None, Some(Command::Dialog(STANDARD)), "S")
            .changed
    );
    assert!(front.key_up("S").changed);
    // Its look: the tab's picture, the texts and dialog, the cursor; none on another screen.
    let layout = parse_layout(LAYOUT).unwrap();
    let shown = look::look(&layout, Screen::HighScores, Some(&front)).unwrap();
    assert_eq!(shown.background, "hiscore.pcx");
    assert!(shown.cursor);
    assert_eq!(shown.over, Vec::new());
    assert_eq!(shown.stale, Vec::new());
    assert_eq!(look::look(&layout, Screen::MainMenu, Some(&front)), None);
    // Both pictures, its font and the buttons are in the start's batch (the creatures are the
    // adapter's).
    let names = look::names(&layout);
    for name in [
        "hiscore.pcx",
        "hiscore2.pcx",
        "medfont.fnt",
        "HiScExt.def",
        "HiScSta.def",
    ] {
        assert!(names.iter().any(|n| n == name), "{name}");
    }
    let box_front = Front::Scores(Box::new({
        let mut s = scores(Bugs::Fixed);
        click(&mut s, middle(RESET));
        s
    }));
    let shown = look::look(&layout, Screen::HighScores, Some(&box_front)).unwrap();
    assert!(!shown.over.is_empty());
}
