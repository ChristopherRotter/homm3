use alloc::boxed::Box;
use alloc::vec::Vec;

use h3_render::Bugs;
use h3_ui::ui::Draw;

use super::{LoadingLook, Play, STEPS};
use crate::message::{self, Key, Style};
use crate::shell::keys::{Command, Context};
use crate::shell::look::{look, parse_layout};
use crate::shell::menu::tests::menus;
use crate::shell::menu::{Did, Front, Hovers};
use crate::shell::{Ended, Game, Kind, PlayEnd, Request, Screen};

const LAYOUT: &str = include_str!("../../../../../layouts/shell.toml");
const PLAY: Screen = Screen::Play { restart: false };

/// Play's start as the scenario screen's begin opens it.
fn play() -> Play {
    let mut all = menus(Bugs::Fixed);
    let previous = Front::Scenario(Box::new(all.scenario.opened(Hovers::default())));
    match all.front(PLAY, Some(previous), [400, 300]) {
        Some(Front::Play(play)) => *play,
        other => panic!("{other:?}"),
    }
}

/// The OK of the notice over the menu, its middle.
fn ok(play: &Play) -> [i32; 2] {
    let layout = parse_layout(LAYOUT).unwrap();
    let draws = play.menu().box_draws(&layout.player_palette);
    let message = match draws.iter().find_map(|d| match d {
        Draw::Button { def, x, y, .. } if def.eq_ignore_ascii_case(&layout.message.ok) => {
            Some([*x, *y])
        }
        _ => None,
    }) {
        Some(at) => at,
        None => panic!("no OK in {draws:?}"),
    };
    [message[0] + 10, message[1] + 10]
}

#[test]
fn the_loading_screen_s_bar_steps_to_twenty_pieces() {
    let layout = parse_layout(LAYOUT).unwrap();
    let look_of = |play: &Play| look(&layout, PLAY, Some(&Front::Play(Box::new(play.clone()))));
    let mut play = play();
    assert_eq!((play.count(), play.shift(), play.noticing()), (0, 0, false));
    // The empty screen: `loadbar.pcx`, no piece, no cursor.
    let shown = look_of(&play).unwrap();
    assert_eq!(shown.background, "loadbar.pcx");
    assert!(shown.draws.is_empty() && shown.over.is_empty() && shown.stale.is_empty());
    assert!(!shown.cursor);
    assert_eq!(shown.shift, 0);
    // Three steps: `loadprog.def`'s frames 0 to 2 at (395 + 18 i, 548).
    for _ in 0..3 {
        play.step();
    }
    let pieces: Vec<(u32, i32, i32)> = play
        .draws(&layout.loading)
        .iter()
        .map(|d| match d {
            Draw::Button {
                def, frame, x, y, ..
            } if def == "loadprog.def" => (*frame, *x, *y),
            other => panic!("{other:?}"),
        })
        .collect();
    assert_eq!(pieces, [(0, 395, 548), (1, 413, 548), (2, 431, 548)]);
    // At most twenty (`load_progress_step`'s 0x14).
    for _ in 0..30 {
        play.step();
    }
    assert_eq!(play.count(), STEPS);
    assert_eq!(look_of(&play).unwrap().draws.len(), 20);
    // Not shown on another screen.
    assert_eq!(
        look(
            &layout,
            Screen::MainMenu,
            Some(&Front::Play(Box::new(play)))
        ),
        None
    );
    let own = LoadingLook {
        background: "b.pcx".into(),
        bar: "p.def".into(),
        at: [1, 2],
        step: 3,
    };
    let mut one = Play::new(menus(Bugs::Fixed).main);
    one.step();
    one.step();
    assert!(matches!(
        one.draws(&own).as_slice(),
        [
            Draw::Button { x: 1, y: 2, .. },
            Draw::Button { x: 4, y: 2, .. }
        ]
    ));
}

#[test]
fn the_fade_steps_every_fifty_ms_then_the_notice_shows() {
    // `adv-begin`: Begin released at 1 000, the loading done at once on the frozen clock; the
    // fade's steps at 1 000 (the screen as it is), 1 100 (the input's step) and 1 150, the black
    // and the notice at 1 200.
    let mut play = play();
    play.step();
    play.loaded(1000);
    assert_eq!((play.count(), play.shift()), (STEPS, 0));
    // Loaded once: no more steps, nor a second start.
    play.step();
    play.loaded(5000);
    assert_eq!((play.count(), play.shift()), (STEPS, 0));
    assert!(!play.clock(1000));
    assert!(!play.clock(1049));
    assert!(play.clock(1100));
    assert_eq!(play.shift(), 1);
    assert!(!play.clock(1149));
    assert!(play.clock(1150));
    assert_eq!(play.shift(), 2);
    assert!(!play.clock(1199));
    assert!(play.clock(1200));
    assert!(play.noticing());
    assert_eq!(play.shift(), 0);
    assert!(!play.clock(9999));
    // A reading late by more than a step takes one step only: the next begins at it.
    let mut late = Play::new(menus(Bugs::Fixed).main);
    late.loaded(0);
    assert!(late.clock(500));
    assert_eq!(late.shift(), 1);
    assert!(!late.clock(549));
    // The look: the screen faded, no cursor; under the notice the main menu with its box.
    let layout = parse_layout(LAYOUT).unwrap();
    let shown = look(&layout, PLAY, Some(&Front::Play(Box::new(late)))).unwrap();
    assert_eq!((shown.shift, shown.cursor), (1, false));
    let shown = look(&layout, PLAY, Some(&Front::Play(Box::new(play.clone())))).unwrap();
    assert_eq!(shown.background, layout.main_menu.background);
    assert_eq!(shown.over, play.menu().box_draws(&layout.player_palette));
    assert!(shown.cursor);
    assert_eq!(shown.shift, 0);
}

#[test]
fn the_inputs_wait_for_the_notice_whose_ok_ends_play_with_the_main_menu() {
    let ended = Did {
        ended: Some(Ended::Play(PlayEnd::Request(Request::MainMenu))),
        ..Did::default()
    };
    let started = play();
    let ok_at = ok(&started);
    let mut front = Front::Play(Box::new(started));
    // Loading and fading: every input goes nowhere.
    for _ in 0..2 {
        assert_eq!(front.moved(ok_at), Did::default());
        assert_eq!(front.press(ok_at), Did::default());
        assert_eq!(front.release(), Did::default());
        assert_eq!(front.right(ok_at), Did::default());
        assert_eq!(front.right_release(), Did::default());
        let command = Some(Command::Message(Key::Ok));
        assert_eq!(front.key(None, command, "Return"), Did::default());
        assert_eq!(front.key_up("Return"), Did::default());
        assert_eq!(front.typed('a'), Did::default());
        assert!(!front.typing());
        if let Front::Play(p) = &mut front {
            p.loaded(0);
        }
    }
    assert_eq!(front.context(), Context::Question);
    assert_eq!(front.contexts(), [Context::Question]);
    assert_eq!(front.hovers(), Hovers::default());
    for t in [50, 100, 150] {
        assert!(front.clock(t).changed);
    }
    // The notice: its OK (pressed, released over it) ends play with the main menu.
    assert_eq!(front.key(None, None, "A"), Did::default());
    let _ = front.moved(ok_at);
    assert!(front.press(ok_at).sample);
    assert_eq!(front.release(), ended);
    // By Enter too; the right button over it changes nothing.
    let mut front = Front::Play(Box::new(play()));
    if let Front::Play(p) = &mut front {
        p.loaded(0);
        for t in [50, 100, 150] {
            let _ = p.clock(t);
        }
    }
    assert_eq!(front.right(ok_at), Did::default());
    assert_eq!(front.right_release(), Did::default());
    let command = Some(Command::Message(Key::Ok));
    assert!(front.key(None, command, "Return").sample);
    assert_eq!(front.key_up("Return"), ended);
}

#[test]
fn the_engine_s_refusal_replaces_the_notice() {
    let layout = parse_layout(LAYOUT).unwrap();
    let (medium, small) = (
        crate::shell::menu::tests::font(8, 16),
        crate::shell::menu::tests::font(5, 10),
    );
    let refusal = message::layout(
        &layout.message,
        (&medium, &small),
        "Refused, with a longer text than the stub's own to be told apart.",
        Style::Ok,
        None,
        (0, Bugs::Fixed),
    );
    let mut play = play();
    let before = play.menu().box_draws(&layout.player_palette);
    play.notice(refusal);
    assert_eq!(play.menu().screen(), PLAY);
    assert_ne!(play.menu().box_draws(&layout.player_palette), before);
    // Play's start opens only after a begin's screen or a word: from no screen too; a restart's
    // is not built (M4).
    let mut all = menus(Bugs::Fixed);
    assert!(matches!(
        all.front(PLAY, None, [0, 0]),
        Some(Front::Play(_))
    ));
    assert!(
        all.front(Screen::Play { restart: true }, None, [0, 0])
            .is_none()
    );
    assert_eq!(
        Screen::Play { restart: false }.built(),
        crate::shell::Built::Whole
    );
    let _ = (Game::New, Kind::Single);
}
