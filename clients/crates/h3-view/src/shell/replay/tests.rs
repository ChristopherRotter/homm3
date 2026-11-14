//! The replay's tests (E27 M2 T4): the harness's step words, its clock and the stops.

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use super::{
    ClipEnd, INPUT_STEP, Input, Replay, Step, Stop, Take, UNTIL_LIMIT, UNTIL_STEP, parse, shown,
    skips,
};
use crate::shell::start::{Header, Op, Pace, Start};
use crate::shell::{Ended, Game, Kind, Request, Screen, Shell};

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
    assert_eq!(shown(Screen::MainMenu, "main_menu"), Some(true));
    assert_eq!(shown(Screen::Start, "main_menu"), Some(false));
    assert_eq!(
        shown(Screen::GameType(Game::New), "new_game_menu"),
        Some(true)
    );
    assert_eq!(
        shown(Screen::GameType(Game::Load), "load_game_menu"),
        Some(true)
    );
    assert_eq!(
        shown(Screen::Multiplayer(Game::Load), "multiplayer"),
        Some(true)
    );
    assert_eq!(
        shown(Screen::Scenario(Game::New, Kind::Single), "scenario_list"),
        Some(true)
    );
    assert_eq!(shown(Screen::HighScores, "high_scores"), Some(true));
    assert_eq!(shown(Screen::MainMenu, "multiplayer"), Some(false));
    assert_eq!(shown(Screen::MainMenu, "scenario_list"), Some(false));
    assert_eq!(shown(Screen::MainMenu, "adventure"), None);
}

/// What a replay of `steps` hands over, each moment with its time and pointer, and how it ends.
fn replay(intro: bool, steps: &[&str]) -> (Vec<String>, Option<Stop>, Replay) {
    replay_from(
        Replay::new(Shell::new(intro), [400, 300], INPUT_STEP, None),
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
    let cases: [(bool, &[&str], Option<Stop>); 8] = [
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
        (
            false,
            &["down:643,70"],
            Some(Stop::AtRest(Screen::MainMenu)),
        ),
        (false, &["key:Escape"], Some(Stop::AtRest(Screen::MainMenu))),
    ];
    for (intro, steps, stop) in cases {
        let (seen, got, _) = replay(intro, steps);
        assert!(seen.is_empty(), "{steps:?}");
        assert_eq!(got, stop, "{steps:?}");
    }
    // `until` checks, then steps: 20 ms waited three times.
    assert_eq!(replay(false, &["until:new_game_menu:20"]).2.time(), 30);
    // A screen not built yet (New Game's menu until T6): nothing drawn or waited for on it.
    let mut shell = Shell::new(false);
    shell.end(Ended::MainMenu(Request::NewGame)).unwrap();
    let not_built = Some(Stop::NotBuilt(Screen::GameType(Game::New)));
    for steps in [
        &["until:new_game_menu"][..],
        &["shot:menu"],
        &["clip:c:10:1"],
        &["key:space"],
    ] {
        let r = Replay::new(shell.clone(), [0, 0], INPUT_STEP, None);
        let (seen, got, _) = replay_from(r, steps);
        assert!(seen.is_empty(), "{steps:?}");
        assert_eq!(got, not_built, "{steps:?}");
    }
}

/// A start of three Smacker clips of `frames` frames each at 66.66 ms.
fn start(frames: u32) -> (Start, Vec<Op>) {
    let header = Header {
        size: [640, 480],
        frames,
        pace: Pace::Smacker(6666),
    };
    let mut ops = Vec::new();
    (Start::new([Some(header); 3], false, 0, &mut ops), ops)
}

#[test]
fn the_start_plays_on_the_replay_s_clock_and_ends_at_the_main_menu() {
    // A clip's frames 67 ms apart at 1 ms steps; three frames, the fade's 150 ms after the
    // last: each clip 351 ms; the menu at 1053 ms.
    let r = Replay::new(Shell::new(true), [400, 300], INPUT_STEP, Some(start(3)));
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
    let r = Replay::new(Shell::new(true), [400, 300], INPUT_STEP, None);
    assert_eq!((r.screen(), r.ops()), (Screen::MainMenu, &[][..]));
}

#[test]
fn a_press_or_a_key_skips_the_start_and_a_move_does_not() {
    // A key at 2000 ms (the 3DO logo's): its fade from 2000, the input's step to 2100, the
    // fade's last two steps, the main menu at 2200 (`start-skip-key`'s times).
    let r = Replay::new(Shell::new(true), [400, 300], INPUT_STEP, Some(start(100)));
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
        let r = Replay::new(Shell::new(true), [400, 300], INPUT_STEP, Some(start(100)));
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
        let mut r = Replay::new(Shell::new(false), [0, 0], INPUT_STEP, None);
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
