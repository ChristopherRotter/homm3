//! The scenario screen's load mode (E27 M4 T4b1; design §3.1 to §3.5, §38): the screen over
//! `scn-load`'s three saves ([`load`]): its items, the list, the panel's name and date, the
//! save's difficulty, the sorts' stale pixels in the original's profile, the right press; Delete's
//! question and its end, the notice with no saves, Load's end (T4b2).

use super::*;

/// The labels drawn for item `id`, in order.
fn labels(draws: &[Draw], id: u16) -> Vec<String> {
    draws
        .iter()
        .filter_map(|d| match d {
            Draw::Label { item, text, .. } if item.id == id => Some(text.clone()),
            _ => None,
        })
        .collect()
}

/// The difficulty's buttons' frames drawn, easy first.
fn difficulty_frames(draws: &[Draw]) -> Vec<u32> {
    (0..5)
        .filter_map(|k| {
            let def = format!("gspbut{}.def", k + 3);
            frame(draws, &def, [506 + 32 * k, 456])
        })
        .collect()
}

#[test]
fn the_load_mode_opens_on_its_list_of_saves() {
    for bugs in [Bugs::Original, Bugs::Fixed] {
        let s = load(bugs);
        assert!(s.is_load());
        assert!(s.showing());
        let draws = s.draws("PLAYERS.PAL");
        // The title, Load on Begin's place, no panel buttons, no allies' or enemies' labels.
        assert_eq!(label(&draws, TITLE).as_deref(), Some("Load a game"));
        assert_eq!(frame(&draws, "scnrlod.def", [414, 535]), Some(0));
        assert_eq!(frame(&draws, "scnrbeg.def", [414, 535]), None);
        for id in [LIST, ADVANCED, RANDOM, ALLIES_LABEL, ENEMIES_LABEL] {
            assert!(s.shown.state(id).hidden, "{id}");
        }
        assert!(flags(&draws).is_empty());
        // Key 2's order (the format, then the file's name), the files' names in the rows; the
        // first selected: no save is named the kept `NEWGAME`.
        assert_eq!(names(&draws), ["Week_one.GM1", "AUTOSAVE.GM1", "small.GM1"]);
        assert_eq!(s.rows(), (Some(0), 0));
        // The panel: the map's name, the file's local time, the save's difficulty selected on the
        // disabled buttons (frame 3; the others frame 2) and its rating.
        assert_eq!(label(&draws, NAME).as_deref(), Some("Middletown"));
        assert_eq!(label(&draws, DATE).as_deref(), Some("3/17/2002 - 9:07"));
        assert_eq!(label(&draws, RATING).as_deref(), Some("130%"));
        assert_eq!(difficulty_frames(&draws), [2, 2, 3, 2, 2]);
        assert_eq!(s.difficulty(), 2);
    }
    // A new game's screen shows no date.
    let s = scenario(Bugs::Fixed);
    assert!(!s.is_load());
    assert_eq!(label(&s.draws(""), DATE).as_deref(), Some(""));
}

#[test]
fn the_autosave_shows_its_name_and_leads_the_name_sort() {
    let mut s = load(Bugs::Fixed);
    let down = Command::List(Scroll::Down);
    s.key(None, Some(down), "Down");
    let draws = s.draws("");
    assert_eq!(label(&draws, NAME).as_deref(), Some("AUTOSAVE"));
    assert_eq!(label(&draws, DATE).as_deref(), Some("1/1/2002 - 4:09"));
    assert_eq!(label(&draws, RATING).as_deref(), Some("160%"));
    assert_eq!(difficulty_frames(&draws), [2, 2, 2, 3, 2]);
    // The name's column: the autosave first, then the files' names; again, the other way.
    click(&mut s, [216, 106]);
    assert_eq!(
        names(&s.draws("")),
        ["AUTOSAVE.GM1", "small.GM1", "Week_one.GM1"]
    );
    click(&mut s, [216, 106]);
    assert_eq!(
        names(&s.draws("")),
        ["Week_one.GM1", "small.GM1", "AUTOSAVE.GM1"]
    );
    // The size's: small, medium, large.
    click(&mut s, [74, 106]);
    assert_eq!(
        names(&s.draws("")),
        ["small.GM1", "AUTOSAVE.GM1", "Week_one.GM1"]
    );
    assert_eq!(label(&s.draws(""), NAME).as_deref(), Some("Arrogance"));
    assert_eq!(s.difficulty(), 0);
}

#[test]
fn a_sort_leaves_the_buttons_and_draws_the_rating_over_the_last_in_the_original_s_profile() {
    // Week_one (Hard) selected; the name's sort selects the autosave (Expert) and redraws part.
    let mut s = load(Bugs::Original);
    click(&mut s, [216, 106]);
    let draws = s.draws("");
    assert_eq!(labels(&draws, RATING), ["130%", "160%"]);
    assert_eq!(difficulty_frames(&draws), [2, 2, 3, 2, 2]);
    // A popup's close redraws whole.
    assert!(s.right([521, 478]).changed);
    assert_eq!(s.over, Over::Help(107));
    assert!(s.right_release().changed);
    let draws = s.draws("");
    assert_eq!(labels(&draws, RATING), ["160%"]);
    assert_eq!(difficulty_frames(&draws), [2, 2, 2, 3, 2]);
    // The client's draws whole.
    let mut s = load(Bugs::Fixed);
    click(&mut s, [216, 106]);
    let draws = s.draws("");
    assert_eq!(labels(&draws, RATING), ["160%"]);
    assert_eq!(difficulty_frames(&draws), [2, 2, 2, 3, 2]);
}

#[test]
fn a_disabled_difficulty_takes_no_press_and_the_panel_no_right_press() {
    let mut s = load(Bugs::Original);
    click(&mut s, [216, 106]);
    // No item where Show Available Scenarios is in a new game: nothing, the stale pixels kept.
    assert_eq!(s.right([514, 91]), Did::default());
    assert_eq!(s.over, Over::Nothing);
    assert!(s.stale());
    // A disabled difficulty: no press, its help on a right press.
    assert!(!click(&mut s, [521, 478]).changed);
    assert_eq!(s.difficulty(), 3);
    assert!(s.stale());
}

#[test]
fn the_panel_buttons_hotkeys_do_nothing_in_the_load_mode() {
    for id in [LIST, RANDOM, ADVANCED] {
        let mut s = load(Bugs::Fixed);
        assert_eq!(s.key(None, Some(Command::Dialog(id)), "S"), Did::default());
        s.key_up("S");
        assert!(s.showing());
        assert!(!s.random_showing() && !s.advanced_showing());
    }
}

/// The box over the screen's text, its lines joined.
fn box_text(s: &Scenario) -> String {
    let Over::Notice(open) = &s.over else {
        return String::new();
    };
    open.message
        .dialog
        .draws(&open.message.shown, "")
        .iter()
        .filter_map(|d| match d {
            Draw::Label { text, .. } => Some(text.clone()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// The box's button `id` pressed and released by the key `by` (Return OK, Escape Cancel).
fn answer(s: &mut Scenario, key: message::Key, by: &str) -> Did {
    let did = s.key(None, Some(Command::Message(key)), by);
    did.with(s.key_up(by))
}

#[test]
fn delete_asks_about_the_selected_save_and_its_cancel_keeps_it() {
    for bugs in [Bugs::Original, Bugs::Fixed] {
        let mut s = load(bugs);
        assert!(s.key(None, Some(Command::Delete), "Delete").changed);
        s.key_up("Delete");
        assert_eq!(s.context(), Context::Question);
        assert!(!s.stubbed());
        assert!(s.question().is_some());
        assert_eq!(s.asked, Asked::Delete);
        assert!(box_text(&s).contains("Delete Week_one.GM1?"));
        // Cancel: the box closed, nothing deleted, the screen as it was (no redraw: the stale
        // pixels the opening left are none).
        let before = s.shown.clone();
        assert!(answer(&mut s, message::Key::Cancel, "Escape").changed);
        assert_eq!(s.over, Over::Nothing);
        assert_eq!(s.asked, Asked::Told);
        assert_eq!(s.take_deleted(), None);
        assert_eq!(s.shown, before);
        assert_eq!(
            names(&s.draws("")),
            ["Week_one.GM1", "AUTOSAVE.GM1", "small.GM1"]
        );
    }
}

#[test]
fn delete_s_ok_removes_the_save_and_sorts_by_the_version_in_the_last_direction() {
    // As `scn-load`: the size's column descending, then Week_one's Delete and OK: the
    // version's order descending, SoD's small before AB's autosave, the first selected with the
    // partial redraw's stale buttons and rating in the original's profile.
    // The original's first size press after the build sorts descending (its recorded key 1).
    let mut s = load(Bugs::Original);
    click(&mut s, [74, 106]);
    assert_eq!(
        names(&s.draws("")),
        ["Week_one.GM1", "AUTOSAVE.GM1", "small.GM1"]
    );
    s.key(None, Some(Command::Delete), "Delete");
    s.key_up("Delete");
    assert!(answer(&mut s, message::Key::Ok, "Return").changed);
    assert_eq!(s.over, Over::Nothing);
    assert_eq!(s.take_deleted().as_deref(), Some("Week_one.GM1"));
    assert_eq!(s.take_deleted(), None);
    let draws = s.draws("");
    assert_eq!(names(&draws), ["small.GM1", "AUTOSAVE.GM1"]);
    assert_eq!(s.rows(), (Some(0), 0));
    assert_eq!(label(&draws, NAME).as_deref(), Some("Arrogance"));
    assert_eq!(s.difficulty(), 0);
    assert!(s.stale());
    assert_eq!(labels(&draws, RATING), ["130%", "80%"]);
    assert_eq!(difficulty_frames(&draws), [2, 2, 3, 2, 2]);
    // The recorded key kept: the size's next press flips its order back to ascending.
    click(&mut s, [74, 106]);
    assert_eq!(names(&s.draws("")), ["small.GM1", "AUTOSAVE.GM1"]);
    // The client's draws whole; ascending, the version's order is AB's, then SoD's.
    let mut s = load(Bugs::Fixed);
    s.key(None, Some(Command::Delete), "Delete");
    s.key_up("Delete");
    answer(&mut s, message::Key::Ok, "Return");
    let draws = s.draws("");
    assert_eq!(names(&draws), ["AUTOSAVE.GM1", "small.GM1"]);
    assert_eq!(labels(&draws, RATING), ["160%"]);
    assert_eq!(difficulty_frames(&draws), [2, 2, 2, 3, 2]);
    assert!(!s.shown.state(BEGIN).disabled);
}

#[test]
fn deleting_the_last_save_empties_the_panel_and_disables_load() {
    for bugs in [Bugs::Original, Bugs::Fixed] {
        let mut s = load(bugs);
        for _ in 0..3 {
            s.key(None, Some(Command::Delete), "Delete");
            s.key_up("Delete");
            answer(&mut s, message::Key::Ok, "Return");
            assert!(s.take_deleted().is_some());
        }
        let draws = s.draws("");
        assert!(names(&draws).is_empty());
        assert_eq!(s.rows(), (None, 0));
        assert!(s.shown.state(BEGIN).disabled);
        assert!(!s.stale());
        for id in [NAME, DATE, RATING] {
            assert_eq!(label(&draws, id).as_deref(), Some(""), "{id}");
        }
        // Load, Enter, Delete and Down do nothing.
        assert!(!click(&mut s, [497, 555]).changed);
        assert_eq!(
            s.key(None, Some(Command::Dialog(BEGIN)), "Return"),
            Did::default()
        );
        s.key_up("Return");
        assert_eq!(s.key(None, Some(Command::Delete), "Delete"), Did::default());
        s.key(None, Some(Command::List(Scroll::Down)), "Down");
        assert_eq!(s.over, Over::Nothing);
        assert_eq!(s.loaded(), None);
        // The next opening shows the notice with no saves (the menus keep the list as left).
        let mut menus = crate::shell::menu::tests::menus(bugs);
        menus.load = load(bugs);
        menus.load.keep(&s);
        let opened = menus.load.opened(Hovers::default());
        assert_eq!(opened.asked, Asked::NoSaves);
    }
}

#[test]
fn an_opening_with_no_saves_shows_its_notice_whose_ok_disables_load() {
    for bugs in [Bugs::Original, Bugs::Fixed] {
        let layout = parse_layout(LAYOUT).unwrap();
        let mut s = scenario_of(Vec::new(), bugs).loading(&layout.scenario.load_begin, 0);
        assert_eq!(s.asked, Asked::NoSaves);
        assert!(box_text(&s).contains("No saved games"));
        assert!(s.question().is_some() && !s.stubbed());
        assert_eq!(s.context(), Context::Question);
        // Load disabled under it; its OK closes it and redraws whole.
        assert!(s.shown.state(BEGIN).disabled);
        assert!(answer(&mut s, message::Key::Ok, "Return").changed);
        assert_eq!(s.over, Over::Nothing);
        assert!(s.shown.state(BEGIN).disabled);
        assert_eq!(frame(&s.draws(""), "scnrlod.def", [414, 535]), Some(2));
        // No difficulty chosen in the original's profile (no map at the opening).
        let chosen = DIFFICULTIES
            .iter()
            .filter(|&&id| s.shown.state(id).selected)
            .count();
        assert_eq!(chosen, usize::from(bugs == Bugs::Fixed));
        // Without its text, none.
        let mut texts = s.texts.clone();
        texts.no_saves = None;
        let mut quiet = scenario_of(Vec::new(), bugs);
        quiet.texts = texts;
        let quiet = quiet.loading(&layout.scenario.load_begin, 0);
        assert_eq!(quiet.over, Over::Nothing);
        assert!(quiet.shown.state(BEGIN).disabled);
    }
}

#[test]
fn load_ends_the_screen_with_the_selected_save() {
    // Load clicked, Enter, and a row's double press.
    let mut s = load(Bugs::Fixed);
    let did = click(&mut s, [497, 555]);
    assert_eq!(did.ended, Some(Ended::Scenario(true)));
    assert_eq!(s.loaded(), Some("Week_one.GM1"));
    assert!(s.begun().is_none());
    let mut s = load(Bugs::Fixed);
    let did = s.key(None, Some(Command::Dialog(BEGIN)), "Return");
    let did = did.with(s.key_up("Return"));
    assert_eq!(did.ended, Some(Ended::Scenario(true)));
    let mut s = load(Bugs::Fixed);
    s.clock(1000);
    click(&mut s, [216, 135]);
    let did = click(&mut s, [216, 135]);
    assert_eq!(did.ended, Some(Ended::Scenario(true)));
    assert_eq!(s.loaded(), Some("Week_one.GM1"));
    // Without Delete's text, nothing; a new game's Delete nothing, its Begin no load.
    let mut s = load(Bugs::Fixed);
    s.texts.delete = None;
    assert_eq!(s.key(None, Some(Command::Delete), "Delete"), Did::default());
    let mut s = scenario(Bugs::Fixed);
    assert_eq!(s.key(None, Some(Command::Delete), "Delete"), Did::default());
    assert_eq!(s.loaded(), None);
    // An answer with no box open, and a box closed with no save selected, change nothing.
    let mut s = load(Bugs::Fixed);
    s.asked = Asked::Delete;
    s.selected = None;
    s.closed(message::YES);
    assert_eq!(s.take_deleted(), None);
    s.selected = Some(7);
    s.asked = Asked::Delete;
    s.closed(message::YES);
    assert_eq!(s.take_deleted(), None);
    assert_eq!(s.load(), Did::default());
}

#[test]
fn an_opening_keeps_the_save_and_shows_the_times_given() {
    let mut s = load(Bugs::Fixed);
    s.key(None, Some(Command::List(Scroll::Down)), "Down");
    s.key(None, Some(Command::List(Scroll::Down)), "Down");
    let mut menus = crate::shell::menu::tests::menus(Bugs::Fixed);
    menus.load = s.clone();
    // The time of one save, then of every one, at the next opening.
    menus.load.set_time(Some("SMALL.gm1"), 0);
    let opened = menus.load.opened(Hovers::default());
    assert_eq!(opened.rows(), (Some(2), 0));
    assert_eq!(opened.entry().map(|e| e.file.as_str()), Some("small.GM1"));
    assert_eq!(
        label(&opened.draws(""), DATE).as_deref(),
        Some("1/1/1970 - 5:30")
    );
    menus.load.set_time(None, 3600);
    let opened = menus.load.opened(Hovers::default());
    assert_eq!(
        label(&opened.draws(""), DATE).as_deref(),
        Some("1/1/1970 - 6:30")
    );
    // A new game's screen keeps no save, a load's no map.
    let mut new = scenario(Bugs::Fixed);
    let map = new.kept.clone();
    new.keep(&opened);
    assert_eq!(new.kept, map);
    let mut again = load(Bugs::Fixed);
    assert_eq!(again.kept, "Week_one.GM1");
    again.keep(&scenario(Bugs::Fixed));
    assert_eq!(again.kept, "Week_one.GM1");
    again.keep(&opened);
    assert_eq!(again.kept, "small.GM1");
    // Listed anew: none, the panel empty.
    again.set_entries(Vec::new());
    let opened = again.opened(Hovers::default());
    assert_eq!(opened.rows(), (None, 0));
    assert_eq!(label(&opened.draws(""), DATE).as_deref(), Some(""));
}

#[test]
fn the_date_writes_its_pattern_s_placeholders_and_keeps_another() {
    let fields = local(1_016_336_220, KOLKATA);
    assert_eq!(
        date("{{day}}.{{month}}.{{year}} {{hour}}:{{minute}}", fields),
        "17.3.2002 9:07"
    );
    assert_eq!(date("{{week}} {{year}}", fields), "{{week}} 2002");
}
