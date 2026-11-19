//! The replay's tests (E27 M2 T4, T6a, T6b1): the harness's step words, its clock, the stops and
//! the menus' inputs.

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

use h3_render::Bugs;

use super::{
    ClipEnd, INPUT_STEP, Input, Replay, Step, Stop, Take, UNTIL_LIMIT, UNTIL_STEP, parse, shown,
    skips,
};
use crate::shell::menu::Front;
use crate::shell::menu::Kind as MenuKind;
use crate::shell::menu::tests::menus;
use crate::shell::start::{Header, Op, Pace, Part, Start};
use crate::shell::{Choice, Ended, Game, Kind, PlayEnd, Request, Screen, Shell};

fn names(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| (*s).to_string()).collect()
}

#[test]
fn the_steps_read_as_the_harness_reads_them() {
    let read = |s: &str| parse(s).unwrap();
    assert_eq!(
        read("until:main_menu"),
        Step::Until {
            screens: names(&["main_menu"]),
            limit: UNTIL_LIMIT
        }
    );
    assert_eq!(
        read("until:turn_dialog|adventure:500"),
        Step::Until {
            screens: names(&["turn_dialog", "adventure"]),
            limit: 500
        }
    );
    assert_eq!(read("check:main_menu"), Step::Check("main_menu".into()));
    assert_eq!(read("step:40"), Step::Step { ms: 40, by: 40 });
    assert_eq!(read("step:2000:10"), Step::Step { ms: 2000, by: 10 });
    assert_eq!(read("step:30:0"), Step::Step { ms: 30, by: 30 });
    let inputs = [
        ("move:643,70", Input::Move([643, 70])),
        ("click:-1,2", Input::Click([-1, 2])),
        ("click-wait:10,20:1500", Input::Click([10, 20])),
        ("down:1,2", Input::Down([1, 2])),
        ("up", Input::Up),
        ("right:3,4", Input::Right([3, 4])),
        ("right-up", Input::RightUp),
        ("key:Escape", Input::Key("Escape".into())),
        ("key-down:F7", Input::KeyDown("F7".into())),
        ("key-up:F7", Input::KeyUp("F7".into())),
        ("type:a:b c", Input::Type("a:b c".into())),
    ];
    for (text, input) in inputs {
        assert_eq!(read(text), Step::Input(input), "{text}");
    }
    assert_eq!(read("shot:menu"), Step::Shot("menu".into()));
    assert_eq!(
        read("clip:logo:6000:1"),
        Step::Clip {
            name: "logo".into(),
            end: ClipEnd::After(6000),
            step: 1
        }
    );
    assert_eq!(
        read("clip:fade:200"),
        Step::Clip {
            name: "fade".into(),
            end: ClipEnd::After(200),
            step: UNTIL_STEP
        }
    );
    assert_eq!(
        read("clip-until:clips:main_menu:10:200000"),
        Step::Clip {
            name: "clips".into(),
            end: ClipEnd::Shown {
                screens: names(&["main_menu"]),
                limit: 200_000
            },
            step: 10
        }
    );
    assert_eq!(
        read("clip-until:to:high_scores::900"),
        Step::Clip {
            name: "to".into(),
            end: ClipEnd::Shown {
                screens: names(&["high_scores"]),
                limit: 900
            },
            step: UNTIL_STEP
        }
    );
    assert_eq!(
        read("clip-until:end:main_menu"),
        Step::Clip {
            name: "end".into(),
            end: ClipEnd::Shown {
                screens: names(&["main_menu"]),
                limit: UNTIL_LIMIT
            },
            step: UNTIL_STEP
        }
    );
    assert_eq!(read("settle:2.5"), Step::Pause);
    assert_eq!(read("wait:1"), Step::Pause);
    for verb in [
        "freeze",
        "press",
        "point",
        "until-button",
        "games-time",
        "poke",
    ] {
        assert_eq!(read(&format!("{verb}:x")), Step::Harness(verb.into()));
    }
}

#[test]
fn a_step_not_the_harness_s_is_refused() {
    let bad = [
        "jump:1",
        "until:",
        "until:main_menu:soon",
        "check:",
        "step:",
        "step:10:x",
        "move:1",
        "move:x,2",
        "move:1,y",
        "click-wait:1",
        "key:",
        "shot:",
        "clip::10",
        "clip:a",
        "clip:a:10:0",
        "clip-until:a::10",
        "clip-until:a:main_menu:x",
        "clip-until::main_menu",
        "clip-until:a:main_menu:10:x",
        "click:1",
        "down:1",
        "right:1",
        "key-down:",
        "key-up:",
        "type:",
    ];
    for step in bad {
        assert!(parse(step).is_err(), "{step}");
    }
    assert_eq!(
        parse("jump:1").unwrap_err(),
        "the step `jump:1`: an unknown verb"
    );
    assert_eq!(parse("clip:a:10:0").unwrap_err(), "a clip's step of 0 ms");
}

#[test]
fn the_probes_names_are_the_shell_s_screens() {
    assert_eq!(shown(Screen::MainMenu, false, "main_menu"), Some(true));
    assert_eq!(shown(Screen::Start, false, "main_menu"), Some(false));
    assert_eq!(
        shown(Screen::GameType(Game::New), false, "new_game_menu"),
        Some(true)
    );
    assert_eq!(
        shown(Screen::GameType(Game::Load), false, "load_game_menu"),
        Some(true)
    );
    assert_eq!(
        shown(Screen::Multiplayer(Game::Load), false, "multiplayer"),
        Some(true)
    );
    assert_eq!(
        shown(
            Screen::Scenario(Game::New, Kind::Single),
            false,
            "scenario_list"
        ),
        Some(true)
    );
    assert_eq!(shown(Screen::HighScores, false, "high_scores"), Some(true));
    assert_eq!(shown(Screen::MainMenu, false, "multiplayer"), Some(false));
    assert_eq!(shown(Screen::MainMenu, false, "scenario_list"), Some(false));
    assert_eq!(shown(Screen::MainMenu, false, "adventure"), None);
    // The hot seat's names over the Multiplayer dialog (E27 M2 T7).
    let mp = Screen::Multiplayer(Game::New);
    assert_eq!(shown(mp, true, "hotseat_names"), Some(true));
    assert_eq!(shown(mp, true, "multiplayer"), Some(false));
    assert_eq!(shown(mp, false, "hotseat_names"), Some(false));
    assert_eq!(shown(Screen::MainMenu, true, "hotseat_names"), Some(false));
}

#[test]
fn the_multiplayer_dialog_and_the_hot_seat_take_the_harness_s_inputs() {
    // E27 M2 T7: the dialog over New Game's menu as it ended; typing by scan code, capitals with
    // Shift; the names' ring; OK to the hot seat's scenario screen (M3 T4a).
    let steps = [
        "until:main_menu",
        "click:643,70",
        "click:649,180",
        "check:multiplayer",
        "right:578,157",
        "right-up",
        "type:Ab",
        "key:BackSpace",
        "click:578,157",
        "check:hotseat_names",
        "key:Tab",
        "type:C",
        "key:shift+Tab",
        "down:345,450",
        "up",
        "until:scenario_list",
    ];
    // The hot seat's scenario screen takes the names (M3 T4a).
    let (_, stop, r) = replay(false, &steps);
    let scenario = Screen::Scenario(Game::New, Kind::Multiplayer);
    assert_eq!((stop, r.screen()), (None, scenario));
    assert!(matches!(r.front(), Some(Front::Scenario(s)) if s.hot_seat()));
    let (_, stop, r) = replay(false, &steps[..13]);
    assert_eq!(stop, None);
    let Some(Front::Multiplayer(m)) = r.front() else {
        panic!("the dialog")
    };
    let hot_seat = m.hot_seat().unwrap();
    assert_eq!(
        (hot_seat.names(), hot_seat.focus()),
        (vec![String::from("A"), String::from("C")], 0)
    );
    assert_eq!(r.menu(), None);
    // A network mode's notice is the client's: a shot under it stops the replay.
    let (seen, stop, _) = replay(
        false,
        &[
            "click:643,70",
            "click:649,180",
            "click:578,214",
            "shot:failed",
        ],
    );
    assert!(seen.is_empty());
    assert_eq!(stop, Some(Stop::Stub(Screen::Multiplayer(Game::New))));
    // M2's review: under it a move moves the pointer alone; any other input stops the replay,
    // the notice left as it was (Enter would close it).
    let opened = ["click:643,70", "click:649,180", "click:578,214"];
    let (_, stop, r) = replay(false, &[&opened[..], &["move:100,110"]].concat());
    assert_eq!((stop, r.pointer()), (None, [100, 110]));
    for input in [
        "key:Return",
        "click:400,300",
        "right:400,300",
        "up",
        "type:a",
    ] {
        let (_, stop, r) = replay(false, &[&opened[..], &[input]].concat());
        assert_eq!(
            stop,
            Some(Stop::Stub(Screen::Multiplayer(Game::New))),
            "{input}"
        );
        assert!(
            matches!(r.front(), Some(Front::Multiplayer(m)) if m.noticing()),
            "{input}"
        );
    }
    // Escape ends the dialog: New Game's menu again.
    let (_, stop, r) = replay(false, &["click:643,70", "click:649,180", "key:Escape"]);
    assert_eq!((stop, r.screen()), (None, Screen::GameType(Game::New)));
}

#[test]
fn the_client_s_replay_types_characters_and_reads_shift_chords() {
    // E27 M2 T7b: in the client's profile a typed text is the window's text input (`ö` typed,
    // no US key needed), the edits' keys and the ring the key table's commands.
    let steps = [
        "until:main_menu",
        "click:643,70",
        "click:649,180",
        "type:Jö",
        "key:Home",
        "type:ß",
        "click:578,157",
        "key:Tab",
        "type:C",
        "key:shift+Tab",
        "key:End",
        "key:BackSpace",
    ];
    let menus = menus(Bugs::Fixed);
    let r = Replay::new(Shell::new(false), [400, 300], INPUT_STEP, None, &menus);
    let (_, stop, r) = replay_from(r, &steps);
    assert_eq!(stop, None);
    let Some(Front::Multiplayer(m)) = r.front() else {
        panic!("the dialog")
    };
    assert_eq!(m.name(), "ßJö");
    let hot_seat = m.hot_seat().unwrap();
    assert_eq!(
        (hot_seat.names(), hot_seat.focus()),
        (vec![String::from("ßJ"), String::from("C")], 0)
    );
    // A text typed where nothing types: its keys alone, which give nothing on a menu.
    let r = Replay::new(Shell::new(false), [400, 300], INPUT_STEP, None, &menus);
    let (_, stop, r) = replay_from(r, &["until:main_menu", "type:x"]);
    assert_eq!((stop, r.screen()), (None, Screen::MainMenu));
}

/// What a replay of `steps` hands over, each moment with its time and pointer, and how it ends.
fn replay(intro: bool, steps: &[&str]) -> (Vec<String>, Option<Stop>, Replay) {
    replay_from(
        Replay::new(
            Shell::new(intro),
            [400, 300],
            INPUT_STEP,
            None,
            &menus(Bugs::Original),
        ),
        steps,
    )
}

/// The same from a replay made by the caller.
fn replay_from(mut r: Replay, steps: &[&str]) -> (Vec<String>, Option<Stop>, Replay) {
    let mut seen = Vec::new();
    for step in steps {
        let step = parse(step).unwrap();
        let mut take = |r: &Replay, t: Take<'_>| {
            seen.push(format!("{t:?} t{} {:?}", r.time(), r.pointer()));
            Ok(())
        };
        if let Some(stop) = r.play(&step, &mut take).unwrap() {
            return (seen, Some(stop), r);
        }
    }
    (seen, None, r)
}

#[test]
fn the_main_menu_s_shots_come_at_the_harness_s_times() {
    let (seen, stop, r) = replay(
        false,
        &[
            "until:main_menu",
            "shot:menu",
            "move:643,70",
            "shot:new-hover",
            "settle:2",
            "move:300,560",
            "shot:new-left",
            "step:25:10",
            "step:0",
            "check:main_menu",
            "shot:later",
        ],
    );
    assert_eq!(stop, None);
    assert_eq!(
        seen,
        [
            "Shot(\"menu\") t0 [400, 300]",
            "Shot(\"new-hover\") t100 [643, 70]",
            "Shot(\"new-left\") t200 [300, 560]",
            "Shot(\"later\") t225 [300, 560]",
        ]
    );
    assert_eq!((r.time(), r.screen()), (225, Screen::MainMenu));
}

#[test]
fn a_clip_keeps_a_frame_a_step_until_its_end() {
    let (seen, stop, _) = replay(false, &["step:5", "clip:c:25:10"]);
    assert_eq!(stop, None);
    assert_eq!(
        seen,
        [
            "Clip { name: \"c\", first: true } t5 [400, 300]",
            "Clip { name: \"c\", first: false } t15 [400, 300]",
            "Clip { name: \"c\", first: false } t25 [400, 300]",
            "Clip { name: \"c\", first: false } t35 [400, 300]",
        ]
    );
    // A screen's clip ends at the frame where it shows.
    let (seen, stop, _) = replay(false, &["clip-until:c:main_menu:10:100"]);
    assert_eq!((seen.len(), stop), (2, None));
    // Or at its limit, the screen not shown.
    let (seen, stop, r) = replay(false, &["clip-until:c:new_game_menu:10:30"]);
    assert_eq!(seen.len(), 4);
    assert_eq!(
        stop,
        Some(Stop::NotShown {
            screens: names(&["new_game_menu"]),
            screen: Screen::MainMenu
        })
    );
    assert_eq!(r.time(), 30);
}

#[test]
fn what_the_viewer_cannot_do_stops_the_replay() {
    let not_shown = |screens: &[&str]| {
        Some(Stop::NotShown {
            screens: names(screens),
            screen: Screen::MainMenu,
        })
    };
    let cases: [(bool, &[&str], Option<Stop>); 6] = [
        (
            false,
            &["poke:game+4:00"],
            Some(Stop::Harness("poke".into())),
        ),
        (
            false,
            &["until:adventure"],
            Some(Stop::Unknown("adventure".into())),
        ),
        (
            false,
            &["check:map_frame"],
            Some(Stop::Unknown("map_frame".into())),
        ),
        (
            false,
            &["check:new_game_menu"],
            not_shown(&["new_game_menu"]),
        ),
        (
            false,
            &["clip-until:c:turn_dialog"],
            Some(Stop::Unknown("turn_dialog".into())),
        ),
        (
            false,
            &["until:new_game_menu:20"],
            not_shown(&["new_game_menu"]),
        ),
    ];
    for (intro, steps, stop) in cases {
        let (seen, got, _) = replay(intro, steps);
        assert!(seen.is_empty(), "{steps:?}");
        assert_eq!(got, stop, "{steps:?}");
    }
    // `until` checks, then steps: 20 ms waited three times.
    assert_eq!(replay(false, &["until:new_game_menu:20"]).2.time(), 30);
    // A screen not built yet (play until E24 M5): nothing drawn or waited for on it; the pointer
    // moves over it.
    let mut shell = Shell::new(false);
    shell.end(Ended::MainMenu(Request::NewGame)).unwrap();
    shell.end(Ended::GameType(Choice::Single)).unwrap();
    shell.end(Ended::Scenario(true)).unwrap();
    // Play's restart (M4; a begin's play is built since E27 M3 T6).
    shell
        .end(Ended::Play(PlayEnd::Request(Request::Restart)))
        .unwrap();
    let not_built = Some(Stop::NotBuilt(Screen::Play { restart: true }));
    for steps in [
        &["until:main_menu"][..],
        &["shot:menu"],
        &["clip:c:10:1"],
        &["key:space"],
        &["move:1,2", "up"],
    ] {
        let r = Replay::new(
            shell.clone(),
            [0, 0],
            INPUT_STEP,
            None,
            &menus(Bugs::Original),
        );
        let (seen, got, _) = replay_from(r, steps);
        assert!(seen.is_empty(), "{steps:?}");
        assert_eq!(got, not_built, "{steps:?}");
    }
    // A stub's screen (E27 M2 T6b2) is the client's: shown to `until` and `check`, but no menu
    // opens on it and the replay stops at its shots and inputs as at one not built.
    // (Credits since E27 M4 T1a built High Scores: the main menu's C.)
    let (seen, got, r) = replay(false, &["until:main_menu", "key:c", "move:1,2"]);
    assert!(seen.is_empty());
    assert_eq!((got, r.screen(), r.menu()), (None, Screen::Credits, None));
    for steps in [&["shot:credits"][..], &["key:Return"], &["clip:c:10:1"]] {
        let mut all = vec!["until:main_menu", "key:c"];
        all.extend(steps);
        let (seen, got, _) = replay(false, &all);
        assert!(seen.is_empty(), "{steps:?}");
        assert_eq!(got, Some(Stop::NotBuilt(Screen::Credits)), "{steps:?}");
    }
    // High Scores' Reset shows the client's notice (CLT-STUB-14): a shot under it stops.
    let (_, got, r) = replay(
        false,
        &[
            "until:main_menu",
            "click:643,304",
            "check:high_scores",
            "click:746,213",
            "move:1,2",
        ],
    );
    assert_eq!((got, r.screen()), (None, Screen::HighScores));
    let (_, got, _) = replay(
        false,
        &[
            "until:main_menu",
            "click:643,304",
            "click:746,213",
            "shot:question",
        ],
    );
    assert_eq!(got, Some(Stop::Stub(Screen::HighScores)));
}

/// A start of three Smacker clips of `frames` frames each at 66.66 ms.
fn start(frames: u32) -> (Start, Vec<Op>) {
    let header = Header {
        size: [640, 480],
        frames,
        pace: Pace::Smacker(6666),
    };
    let mut ops = Vec::new();
    (
        Start::new([Some(header); 3], false, Bugs::Original, 0, &mut ops),
        ops,
    )
}

#[test]
fn the_start_plays_on_the_replay_s_clock_and_ends_at_the_main_menu() {
    // A clip's frames 67 ms apart at 1 ms steps; three frames, the fade's 150 ms after the
    // last: each clip 351 ms; the menu at 1053 ms.
    let r = Replay::new(
        Shell::new(true),
        [400, 300],
        INPUT_STEP,
        Some(start(3)),
        &menus(Bugs::Original),
    );
    assert_eq!(r.ops(), [Op::Black]);
    assert!(r.start().is_some());
    let (seen, stop, r) = replay_from(r, &["clip:logo:300:1", "clip-until:clips:main_menu:1"]);
    assert_eq!(stop, None);
    assert_eq!(seen.len(), 300 + 1 + 753 + 1);
    assert_eq!(
        (r.time(), r.screen(), r.start()),
        (1053, Screen::MainMenu, None)
    );
    let frames = r
        .ops()
        .iter()
        .filter(|op| matches!(op, Op::Frame(..)))
        .count();
    assert_eq!(frames, 9);
    assert_eq!(r.ops().last(), Some(&Op::Black));
    // Without the start's clips the start ends at once.
    let r = Replay::new(
        Shell::new(true),
        [400, 300],
        INPUT_STEP,
        None,
        &menus(Bugs::Original),
    );
    assert_eq!((r.screen(), r.ops()), (Screen::MainMenu, &[][..]));
}

#[test]
fn a_press_or_a_key_skips_the_start_and_a_move_does_not() {
    // A key at 2000 ms (the 3DO logo's): its fade from 2000, the input's step to 2100, the
    // fade's last two steps, the main menu at 2200 (`start-skip-key`'s times).
    let r = Replay::new(
        Shell::new(true),
        [400, 300],
        INPUT_STEP,
        Some(start(100)),
        &menus(Bugs::Original),
    );
    let (seen, stop, r) = replay_from(
        r,
        &[
            "step:2000:10",
            "shot:before",
            "key:space",
            "until:main_menu",
            "shot:menu",
        ],
    );
    assert_eq!(stop, None);
    assert_eq!(
        seen,
        [
            "Shot(\"before\") t2000 [400, 300]",
            "Shot(\"menu\") t2200 [400, 300]"
        ]
    );
    assert_eq!(r.ops().last(), Some(&Op::Black));
    assert_eq!(r.op_times().len(), r.ops().len());
    // E27 M2 T5b1: with its sound the logo's starts at 10 ms and stops at the key (2000 ms),
    // each op with its time.
    let header = Header {
        size: [640, 480],
        frames: 100,
        pace: Pace::Smacker(6666),
    };
    let mut ops = Vec::new();
    let sounding = Start::with_sound(
        [Some(header); 3],
        [Some(88_200); 3],
        false,
        Bugs::Original,
        0,
        &mut ops,
    );
    let r = Replay::new(
        Shell::new(true),
        [400, 300],
        INPUT_STEP,
        Some((sounding, ops)),
        &menus(Bugs::Original),
    );
    let (_, stop, r) = replay_from(r, &["step:2000:10", "key:space", "until:main_menu"]);
    assert_eq!(stop, None);
    let sound: Vec<(Op, u32)> = r
        .ops()
        .iter()
        .zip(r.op_times())
        .filter(|(op, _)| matches!(op, Op::Sound(_) | Op::Quiet(_)))
        .map(|(op, t)| (*op, *t))
        .collect();
    assert_eq!(
        sound,
        [(Op::Sound(Part::Logo), 10), (Op::Quiet(Part::Logo), 2000)]
    );
    for (input, skips_it) in [
        ("move:1,2", false),
        ("key:F4", false),
        ("key-up:space", false),
        ("up", false),
        ("right-up", false),
        ("click:1,2", true),
        ("down:1,2", true),
        ("right:1,2", true),
        ("key-down:Return", true),
        ("type:a", true),
    ] {
        let Step::Input(i) = parse(input).unwrap() else {
            panic!("{input}")
        };
        assert_eq!(skips(&i), skips_it, "{input}");
        let r = Replay::new(
            Shell::new(true),
            [400, 300],
            INPUT_STEP,
            Some(start(100)),
            &menus(Bugs::Original),
        );
        let (_, stop, r) = replay_from(r, &["step:500:10", input, "step:300:10"]);
        assert_eq!(stop, None);
        let menu = if skips_it {
            Screen::MainMenu
        } else {
            Screen::Start
        };
        assert_eq!(r.screen(), menu, "{input}");
    }
}

#[test]
fn an_input_during_the_last_fade_reaches_the_main_menu() {
    // CLT-Q130 (a): `H3Intro`'s fade from 903 ms (its third frame at 1 ms steps): the N given at
    // 960 waits and opens New Game's menu once the main menu is up; given at 300, during the 3DO
    // logo's fade (NWCLogo's loop flushes the queue), it is lost.
    for (at, screen) in [(960, Screen::GameType(Game::New)), (300, Screen::MainMenu)] {
        let r = Replay::new(
            Shell::new(true),
            [400, 300],
            INPUT_STEP,
            Some(start(3)),
            &menus(Bugs::Original),
        );
        let wait = format!("step:{at}:1");
        let (_, stop, r) = replay_from(r, &[&wait, "key:n", "step:2000:1"]);
        assert_eq!((stop, r.screen()), (None, screen), "{at}");
    }
    // A move there moves the pointer at once, nothing kept.
    let r = Replay::new(
        Shell::new(true),
        [400, 300],
        INPUT_STEP,
        Some(start(3)),
        &menus(Bugs::Original),
    );
    let (_, stop, r) = replay_from(r, &["step:960:1", "move:1,2", "step:2000:1"]);
    assert_eq!(
        (stop, r.screen(), r.pointer()),
        (None, Screen::MainMenu, [1, 2])
    );
}

#[test]
fn a_release_with_nothing_held_passes_on_a_screen_at_rest() {
    // `start-skip-right`: the right press skipped H3Intro, its release reaches the main menu.
    let (seen, stop, r) = replay(false, &["right-up", "up", "key-up:space", "shot:menu"]);
    assert_eq!(stop, None);
    assert_eq!(seen, ["Shot(\"menu\") t300 [400, 300]"]);
    assert_eq!(r.screen(), Screen::MainMenu);
}

#[test]
fn the_caller_s_failure_ends_the_replay() {
    for (step, after) in [("shot:menu", 0), ("clip:c:30", 0), ("clip:c:30", 1)] {
        let mut r = Replay::new(
            Shell::new(false),
            [0, 0],
            INPUT_STEP,
            None,
            &menus(Bugs::Original),
        );
        let mut count = 0;
        let mut take = |_: &Replay, _: Take<'_>| {
            count += 1;
            if count > after {
                Err(String::from("no frame"))
            } else {
                Ok(())
            }
        };
        assert_eq!(
            r.play(&parse(step).unwrap(), &mut take),
            Err(String::from("no frame")),
            "{step}"
        );
    }
}

#[test]
fn the_main_menu_takes_the_harness_s_inputs() {
    // `menu-main`'s New Game: the hover, the press, the release off it, the popup.
    let (seen, stop, r) = replay(
        false,
        &[
            "until:main_menu",
            "move:643,70",
            "down:643,70",
            "move:300,560",
            "up",
            "shot:left",
            "right:643,70",
            "shot:popup",
            "right-up",
        ],
    );
    assert_eq!(stop, None);
    assert_eq!(seen.len(), 2);
    let menu = r.menu().unwrap();
    assert_eq!((menu.hovered(), menu.cursor()), (Some(101), true));
    assert_eq!(r.time(), 600);
    // `menu-quit`: Quit's question by the button, cancelled; again by Escape, escaped.
    let (_, stop, r) = replay(false, &["click:643,520", "check:main_menu"]);
    assert_eq!(stop, None);
    assert!(r.menu().unwrap().asking());
    let (_, _, r) = replay(false, &["key:Escape", "key-down:Escape", "key-up:Escape"]);
    assert!(!r.menu().unwrap().asking());
    // A key no command takes changes nothing; a hotkey, typed or given, ends the menu.
    let (_, stop, r) = replay(false, &["key:F4", "type:1"]);
    assert_eq!((stop, r.screen()), (None, Screen::MainMenu));
    for steps in [
        &["click:643,70"][..],
        &["type:n"],
        &["key:n"],
        &["key-down:N", "key-up:N"],
    ] {
        let (_, _, r) = replay(false, steps);
        assert_eq!(r.screen(), Screen::GameType(Game::New), "{steps:?}");
        assert_eq!(
            r.menu().map(crate::shell::menu::Menu::kind),
            Some(MenuKind::GameType(Game::New))
        );
    }
    // The question's OK ends the shell.
    let (_, _, r) = replay(false, &["key:Escape", "key:Return"]);
    assert_eq!(r.screen(), Screen::End);
}

#[test]
fn with_the_effects_on_each_press_s_sample_is_listed() {
    // `menu-sound`: New Game's press at 0 ms, on the harness's first channel.
    let r = Replay::new(
        Shell::new(false),
        [400, 300],
        INPUT_STEP,
        None,
        &menus(Bugs::Original),
    )
    .with_sound(151);
    let (_, _, r) = replay_from(r, &["until:main_menu", "click:643,70"]);
    let played: Vec<(u32, usize, u32)> = r
        .samples()
        .iter()
        .map(|p| (p.time, p.channel, p.length))
        .collect();
    assert_eq!(played, [(0, 0, 151)]);
    // Then Back's press 100 ms later, the first still playing: the next channel (`sound2`).
    let (_, _, r) = replay_from(
        r,
        &["check:new_game_menu", "click:645,517", "check:main_menu"],
    );
    let played: Vec<(u32, usize)> = r.samples().iter().map(|p| (p.time, p.channel)).collect();
    assert_eq!(played, [(0, 0), (100, 1)]);
    // Without the effects none is listed.
    let (_, _, r) = replay(false, &["click:643,70"]);
    assert!(r.samples().is_empty());
}

#[test]
fn the_game_type_menus_take_the_harness_s_inputs() {
    // `menu-new`: New Game's menu opens with none hovered, the pointer where the click left it.
    let (seen, stop, r) = replay(
        false,
        &[
            "until:main_menu",
            "click:643,70",
            "check:new_game_menu",
            "shot:menu",
            "move:649,65",
            "down:649,65",
            "move:300,560",
            "up",
            "right:649,180",
            "shot:popup",
            "right-up",
        ],
    );
    assert_eq!(stop, None);
    assert_eq!(seen[0], "Shot(\"menu\") t100 [643, 70]");
    let menu = r.menu().unwrap();
    assert_eq!(menu.kind(), MenuKind::GameType(Game::New));
    assert_eq!(menu.hovered(), Some(102));
    // Back's click and Load Game's Escape: the main menu again, opened anew, its procedure's
    // last item the one New Game or Load Game was chosen by (CLT-Q128), none drawn hovered.
    for (steps, last) in [
        (&["click:643,70", "click:645,517"][..], 101),
        (
            &["click:645,192", "check:load_game_menu", "key:Escape"],
            102,
        ),
    ] {
        let (_, stop, r) = replay(false, steps);
        assert_eq!((stop, r.screen()), (None, Screen::MainMenu), "{steps:?}");
        let menu = r.menu().unwrap();
        assert_eq!(menu.hovered(), Some(last));
        assert!(
            !menu
                .dialog_draws("")
                .iter()
                .any(|d| matches!(d, h3_ui::ui::Draw::Button { frame: 3, .. }))
        );
    }
    // Single: the scenario screen, no menu on it (M3's).
    let (_, _, r) = replay(false, &["key:n", "key:s"]);
    assert_eq!(r.screen(), Screen::Scenario(Game::New, Kind::Single));
    assert_eq!(r.menu(), None);
    let (_, _, r) = replay(false, &["key:l", "key:m"]);
    assert_eq!(r.screen(), Screen::Multiplayer(Game::Load));
    // A key no command takes in the menu changes nothing.
    let (_, _, r) = replay(false, &["key:n", "key:n"]);
    assert_eq!(r.screen(), Screen::GameType(Game::New));
}

#[test]
fn the_scenario_screen_takes_the_original_s_background_draws_in_turn() {
    // E27 M3 T1a: `scn-new`'s three openings, the third past the draws given (0).
    let menus = menus(Bugs::Original);
    let r = Replay::new(Shell::new(false), [400, 300], INPUT_STEP, None, &menus)
        .with_backgrounds(&[1, 0]);
    let open = [
        "click:643,70",
        "click:649,65",
        "move:790,300",
        "until:scenario_list",
    ];
    let mut steps = vec!["until:main_menu"];
    steps.extend(open);
    let (_, stop, r) = replay_from(r, &steps);
    assert_eq!(stop, None);
    let background = |r: &Replay| match r.front() {
        Some(Front::Scenario(s)) => Some(s.background()),
        _ => None,
    };
    assert_eq!(background(&r), Some(1));
    let again = [
        "click:667,555",
        "move:790,300",
        "until:new_game_menu",
        "click:649,65",
    ];
    let (_, stop, r) = replay_from(r, &again);
    assert_eq!((stop, background(&r)), (None, Some(0)));
    let (_, stop, r) = replay_from(r, &again);
    assert_eq!((stop, background(&r)), (None, Some(0)));
    assert_eq!(r.screen(), Screen::Scenario(Game::New, Kind::Single));
}

#[test]
fn begin_with_a_random_map_chosen_stops_the_replay_under_the_client_s_stub() {
    // E27 M3 T5b: the original generates the map; the client's notice is its stub.
    let steps = [
        "until:main_menu",
        "click:643,70",
        "click:649,65",
        "move:790,300",
        "until:scenario_list",
        "click:514,115",
        "move:790,300",
        "shot:tab",
        "click-wait:497,555:3000",
        "move:790,300",
    ];
    let (seen, stop, r) = replay(false, &steps);
    assert_eq!((seen.len(), stop), (1, None));
    assert!(matches!(r.front(), Some(Front::Scenario(s)) if s.stubbed()));
    let (seen, stop, _) = replay_from(r, &["shot:begin"]);
    assert!(seen.is_empty());
    assert_eq!(
        stop,
        Some(Stop::Stub(Screen::Scenario(Game::New, Kind::Single)))
    );
}

#[test]
fn the_list_takes_xdotool_s_page_keys_as_the_adapter_s() {
    // E27 M3 T1b1: `scn-list-keys`' Next and Prior are the adapter's PageDown and PageUp; the
    // screen's double press is timed from its opening on the replay's clock.
    let mut menus = menus(Bugs::Original);
    menus.scenario = crate::shell::scenario::tests::scenario_of(
        crate::shell::scenario::tests::many(40),
        Bugs::Original,
    );
    let r = Replay::new(Shell::new(false), [400, 300], INPUT_STEP, None, &menus);
    let rows = |r: &Replay| match r.front() {
        Some(Front::Scenario(s)) => Some(s.rows()),
        _ => None,
    };
    let steps = [
        "until:main_menu",
        "click:643,70",
        "click:649,65",
        "move:790,300",
        "until:scenario_list",
        "click:514,91",
        "key:Next",
    ];
    let (_, stop, r) = replay_from(r, &steps);
    assert_eq!((stop, rows(&r)), (None, Some((Some(17), 17))));
    let (_, stop, r) = replay_from(r, &["key:Prior"]);
    assert_eq!((stop, rows(&r)), (None, Some((Some(0), 0))));
    // A row's click selects it; another row's 500 ms later selects it; another's 100 ms later
    // begins in the original's profile (E27 M3 T6: play's start, its loading screen).
    let (_, stop, r) = replay_from(r, &["click:216,235"]);
    assert_eq!((stop, rows(&r)), (None, Some((Some(4), 0))));
    let (_, stop, r) = replay_from(r, &["step:500", "click:216,260"]);
    assert_eq!((stop, rows(&r)), (None, Some((Some(5), 0))));
    let (_, stop, r) = replay_from(r, &["click:216,235"]);
    assert_eq!((stop, r.screen()), (None, Screen::Play { restart: false }));
}

/// E27 M3 T6: Begin over a map: play's start at once on the frozen clock, its fade's steps at
/// the click (the screen as it is), 100 ms on (the input's step) and 150 ms on (`adv-begin`'s
/// `bar` clip), then its notice, where the replay stops.
#[test]
fn begin_shows_the_full_bar_fading_then_play_s_notice_stops_the_replay() {
    let (_, stop, r) = replay(
        false,
        &[
            "until:main_menu",
            "click:643,70",
            "click:649,65",
            "until:scenario_list",
        ],
    );
    assert_eq!(stop, None);
    let begun = r.time();
    let mut r = r;
    let mut seen = Vec::new();
    for step in ["click-wait:497,555:3000", "clip:bar:90"] {
        let step = parse(step).unwrap();
        let mut take = |r: &Replay, t: Take<'_>| {
            let shift = match r.front() {
                Some(Front::Play(p)) => (p.count(), p.shift()),
                other => panic!("{other:?}"),
            };
            seen.push((format!("{t:?}"), r.time() - begun, shift));
            Ok(())
        };
        assert_eq!(r.play(&step, &mut take).unwrap(), None);
    }
    assert_eq!(r.screen(), Screen::Play { restart: false });
    let shifts: Vec<(u32, (u8, u8))> = seen.iter().map(|(_, t, s)| (*t, *s)).collect();
    assert_eq!(shifts.first(), Some(&(100, (20, 1))));
    assert!(shifts.contains(&(150, (20, 2))));
    assert_eq!(shifts.last(), Some(&(190, (20, 2))));
    // At 200 the notice: a shot or a wait stops the replay (the client's stub).
    let (_, stop, _) = replay_from(r, &["step:10", "shot:map"]);
    assert_eq!(stop, Some(Stop::Stub(Screen::Play { restart: false })));
}
