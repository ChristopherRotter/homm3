use super::*;
use alloc::format;

use h3_ui::edit::Caret;
use h3_ui::keys::Keys;

use crate::message::{self, Style};
use crate::shell::Game;
use crate::shell::keys;
use crate::shell::look::parse_layout;
use crate::shell::menu::BOX_PLAYER;
use crate::shell::menu::tests::{font, game_type};

const LAYOUT: &str = include_str!("../../../../../layouts/shell.toml");

/// The dialog as it opens (the shell's layout), each item's hint `hint <name>`, each popup and
/// the notice laid out in fonts of 8-wide glyphs, the hot seat's title `Names`.
pub(crate) fn multiplayer(bugs: Bugs) -> Multiplayer {
    let layout = parse_layout(LAYOUT).unwrap();
    let (medium, small) = (font(8, 16), font(5, 10));
    let laid = |text: &str, style| {
        message::layout(
            &layout.message,
            (&medium, &small),
            text,
            style,
            None,
            (BOX_PLAYER, bugs),
        )
    };
    let mut hints = BTreeMap::new();
    let mut help = BTreeMap::new();
    for &(id, name) in TEXTED.iter().chain(&HOT_SEAT_TEXTED) {
        hints.insert(id, format!("hint {name}"));
        help.insert(id, laid("{Help}\n\nThe help.", Style::Popup));
    }
    let texts = Texts {
        hints,
        help,
        network: laid("Not yet.", Style::Ok),
        title: String::from("Names"),
    };
    Multiplayer::new(layout.multiplayer, layout.hot_seat, texts, bugs)
}

/// The dialog over New Game's menu, the pointer parked.
fn opened(bugs: Bugs) -> Multiplayer {
    multiplayer(bugs).opened(game_type(Game::New, bugs), [400, 300])
}

/// Each button's middle on the screen.
const HOT: [i32; 2] = [578, 157];
const IPX: [i32; 2] = [578, 214];
const ONLINE_AT: [i32; 2] = [578, 442];
const CANCEL_AT: [i32; 2] = [578, 503];
const MAP: [i32; 2] = [300, 300];
const NAME_AT: [i32; 2] = [250, 500];
const OK_AT: [i32; 2] = [345, 450];
const BACK: [i32; 2] = [455, 450];

/// The hot seat's edit `k`'s middle.
const fn edit_at(k: i32) -> [i32; 2] {
    [417, 187 + 30 * k]
}

/// `text` typed as the profile takes it: the window's text input in the client's, each US key's
/// scan code in the original's.
fn typed(m: &mut Multiplayer, text: &str) {
    for c in text.chars() {
        let did = if m.bugs == Bugs::Fixed {
            m.typed(c)
        } else {
            let scan = u8::try_from(c).ok().and_then(edit::scan_of);
            m.key(scan, None, "x")
        };
        assert!(did.changed, "{c}");
    }
}

/// A key down as the window or a replay gives it (E27 M2 T7b): `name` (`shift+` holding Shift)
/// read as the original reads it and through the shell's default key table.
fn down(m: &mut Multiplayer, name: &str) -> Did {
    let (shift, bare) = keys::shifted(name);
    let command = keys::command_for(&Keys::original(), &m.contexts(), shift, bare);
    m.key(keys::scan(name), command, bare)
}

fn plain(code: u8) -> Option<Scan> {
    Some(Scan::plain(code))
}

fn click(m: &mut Multiplayer, at: [i32; 2]) -> Did {
    m.moved(at);
    m.press(at);
    m.release()
}

/// The edit drawn for item `id`, if any: its text and caret.
fn edit_of(draws: &[Draw], id: u16) -> Option<(String, Option<u32>)> {
    draws.iter().find_map(|d| match d {
        Draw::Edit {
            item, text, caret, ..
        } if item.id == id => Some((text.clone(), *caret)),
        _ => None,
    })
}

#[test]
fn the_dialog_opens_on_its_first_stage_with_the_name_focused() {
    let m = opened(Bugs::Fixed);
    assert_eq!((m.name(), m.status()), ("", ""));
    assert_eq!(m.context(), Context::Multiplayer);
    assert!(m.cursor() && !m.noticing() && m.hot_seat().is_none() && m.names().is_empty());
    assert_eq!(
        m.under().map(Menu::kind),
        Some(crate::shell::menu::Kind::GameType(Game::New))
    );
    let draws = m.draws("");
    assert_eq!(edit_of(&draws, NAME), Some((String::new(), Some(0))));
    // The status line's box filled black before its text, inside the dialog.
    let fill = draws
        .iter()
        .position(|d| matches!(d, Draw::Fill { .. }))
        .unwrap();
    assert_eq!(
        draws[fill],
        Draw::Fill {
            rect: [181, 520, 438, 18],
            palette: String::from("game.pal"),
            entry: STATUS_BACKGROUND,
        }
    );
    assert!(matches!(&draws[fill + 1], Draw::Label { item, .. } if item.id == STATUS));
    assert!(m.box_draws("").is_empty());
    assert!(multiplayer(Bugs::Fixed).under().is_none());
}

#[test]
fn a_move_writes_the_hint_of_a_new_item_under_the_mouse() {
    let mut m = opened(Bugs::Fixed);
    assert!(m.moved(HOT).changed);
    assert_eq!(m.status(), "hint hot-seat");
    assert!(!m.moved([HOT[0] + 1, HOT[1]]).changed);
    // The map's picture and the name's edit have none.
    assert!(m.moved(MAP).changed);
    assert_eq!(m.status(), "");
    assert!(!m.moved(NAME_AT).changed);
    // Off every item.
    assert!(!m.moved([5, 5]).changed);
    m.moved(CANCEL_AT);
    assert_eq!(m.status(), "hint cancel");
}

#[test]
fn a_button_held_runs_its_own_loop_and_the_hint_follows_its_end() {
    let mut m = opened(Bugs::Fixed);
    m.moved(HOT);
    let down = m.press(HOT);
    assert!(down.changed && down.sample);
    // Its moves change no hint; released off it, no command, the hover taken there.
    m.moved(MAP);
    assert_eq!(m.status(), "hint hot-seat");
    let up = m.release();
    assert!(up.changed && up.ended.is_none());
    assert!(m.hot_seat().is_none());
    assert_eq!(m.status(), "");
    // A release with nothing held changes nothing.
    assert_eq!(m.release(), Did::default());
}

#[test]
fn hot_seat_opens_the_names_with_the_dialogs_name() {
    let mut m = opened(Bugs::Fixed);
    m.moved(NAME_AT);
    typed(&mut m, "Anna");
    assert_eq!(m.name(), "Anna");
    // Enter is not the name's, nor any hotkey here.
    assert_eq!(down(&mut m, "Return"), Did::default());
    let did = click(&mut m, HOT);
    assert!(did.changed && did.ended.is_none());
    assert_eq!(m.context(), Context::HotSeat);
    let h = m.hot_seat().unwrap();
    assert_eq!((h.focus(), h.ok_enabled()), (0, false));
    let draws = m.draws("");
    assert_eq!(
        edit_of(&draws, NAMES[0]),
        Some((String::from("Anna"), Some(4)))
    );
    assert_eq!(edit_of(&draws, NAME), Some((String::from("Anna"), Some(4))));
    assert!(
        draws
            .iter()
            .any(|d| matches!(d, Draw::Label { item, text, .. }
        if item.id == TITLE && text == "Names"))
    );
    assert_eq!(
        draws
            .iter()
            .filter(|d| matches!(d, Draw::Fill { .. }))
            .count(),
        2
    );
    // The hot seat's shadow.
    assert!(draws.iter().any(|d| matches!(d, Draw::Darken { .. })));
}

#[test]
fn the_names_move_round_their_ring_and_ok_needs_two() {
    // Both profiles alike: by scan code in the original's, by the key table in the client's.
    for bugs in [Bugs::Fixed, Bugs::Original] {
        let mut m = opened(bugs);
        click(&mut m, HOT);
        typed(&mut m, "a");
        let focus = |m: &Multiplayer| m.hot_seat().unwrap().focus();
        assert!(down(&mut m, "Tab").changed);
        assert_eq!(focus(&m), 1);
        typed(&mut m, "b");
        assert!(m.hot_seat().unwrap().ok_enabled());
        down(&mut m, "BackSpace");
        assert!(!m.hot_seat().unwrap().ok_enabled());
        typed(&mut m, "c");
        down(&mut m, "Up");
        assert_eq!(focus(&m), 0);
        down(&mut m, "shift+Tab");
        assert_eq!(focus(&m), 7);
        down(&mut m, "Down");
        assert_eq!(focus(&m), 0);
        // A press on an edit gives it the focus.
        assert!(m.press(edit_at(3)).changed);
        assert_eq!(focus(&m), 3);
        assert!(m.press(edit_at(3)).changed);
        assert_eq!(focus(&m), 3);
        assert_eq!(m.hot_seat().unwrap().names(), ["a", "c"], "{bugs:?}");
    }
}

#[test]
fn the_client_types_what_the_keyboard_prints() {
    // E27 M2 T7b (decision 39): the window's text input, the edits' keys through the key table.
    let mut m = opened(Bugs::Fixed);
    assert!(m.typing());
    assert_eq!(m.contexts(), [Context::Edit, Context::Multiplayer]);
    typed(&mut m, "Jörg");
    assert_eq!(m.name(), "Jörg");
    assert!(down(&mut m, "Home").changed);
    typed(&mut m, "ß");
    assert!(down(&mut m, "End").changed);
    assert!(down(&mut m, "BackSpace").changed);
    assert_eq!(m.name(), "ßJör");
    // A brace is no change; the scan codes are not read (Tab is nothing here).
    assert!(!m.typed('{').changed);
    assert_eq!(m.key(plain(0x1E), None, "A"), Did::default());
    assert_eq!(down(&mut m, "Tab"), Did::default());
    assert_eq!(m.name(), "ßJör");
    // The hot seat's edits alike; Enter with OK disabled moves on (hot-seat-ok's command).
    click(&mut m, HOT);
    assert_eq!(m.contexts(), [Context::Edit, Context::HotSeat]);
    down(&mut m, "Left");
    typed(&mut m, "é");
    let draws = m.draws("");
    assert_eq!(
        edit_of(&draws, NAMES[0]),
        Some((String::from("ßJöér"), Some(4)))
    );
    assert!(down(&mut m, "Return").changed);
    assert_eq!(m.hot_seat().unwrap().focus(), 1);
    typed(&mut m, "Ån");
    assert!(m.hot_seat().unwrap().ok_enabled());
    for caret in [Caret::Left, Caret::Delete] {
        assert!(m.key(None, Some(Command::Edit(caret)), "x").changed);
    }
    assert_eq!(m.hot_seat().unwrap().names(), ["ßJöér", "Å"]);
    assert!(m.hot_seat().unwrap().ok_enabled());
    // A help popup over the names takes no text.
    m.right(OK_AT);
    assert!(!m.typing());
    assert_eq!(m.typed('x'), Did::default());
    m.right_release();
    // Nor the notice; nor the original's profile, whose edits read scan codes.
    let mut m = opened(Bugs::Fixed);
    click(&mut m, IPX);
    assert_eq!((m.typing(), m.contexts()), (false, vec![Context::Question]));
    assert_eq!(m.typed('x'), Did::default());
    let mut m = opened(Bugs::Original);
    assert_eq!(
        (m.typing(), m.contexts()),
        (false, vec![Context::Multiplayer])
    );
    assert_eq!(m.typed('x'), Did::default());
    assert_eq!(
        m.key(None, Some(Command::Edit(Caret::Home)), "Home"),
        Did::default()
    );
    assert_eq!(m.name(), "");
}

#[test]
fn enter_presses_an_enabled_ok_but_in_the_originals_profile() {
    // Fixed (CLT-Q107, decision 22): Enter with OK enabled is OK's hotkey.
    let mut m = opened(Bugs::Fixed);
    click(&mut m, HOT);
    typed(&mut m, "a");
    // OK disabled: Enter moves on.
    down(&mut m, "Return");
    assert_eq!(m.hot_seat().unwrap().focus(), 1);
    typed(&mut m, "b");
    let pressed = down(&mut m, "Return");
    assert!(pressed.changed && pressed.sample);
    let up = m.key_up("Return");
    assert_eq!(up.ended, Some(Ended::Multiplayer(true)));
    assert_eq!(m.names(), ["a", "b"]);
    // The original's: to the next edit.
    let mut m = opened(Bugs::Original);
    click(&mut m, HOT);
    typed(&mut m, "a");
    down(&mut m, "Tab");
    typed(&mut m, "b");
    down(&mut m, "Return");
    assert_eq!(m.hot_seat().unwrap().focus(), 2);
    assert_eq!(m.key_up("Return").ended, None);
}

#[test]
fn ok_and_cancel_end_or_close_the_names() {
    let mut m = opened(Bugs::Fixed);
    click(&mut m, HOT);
    // OK disabled takes no press.
    assert!(!m.press(OK_AT).changed);
    m.release();
    let did = click(&mut m, BACK);
    assert!(did.changed && did.ended.is_none());
    assert!(m.hot_seat().is_none());
    // Escape: the edit leaves it, Cancel's hotkey takes it (with Shift too); in the original's
    // profile its edit refuses the scan code first.
    click(&mut m, HOT);
    assert!(down(&mut m, "shift+Escape").sample);
    m.key_up("Escape");
    assert!(m.hot_seat().is_none());
    let mut original = opened(Bugs::Original);
    click(&mut original, HOT);
    assert!(down(&mut original, "Escape").sample);
    original.key_up("Escape");
    assert!(original.hot_seat().is_none());
    // Another command does nothing there.
    click(&mut m, HOT);
    assert_eq!(
        m.key(None, Some(Command::Dialog(CANCEL)), "Escape"),
        Did::default()
    );
    typed(&mut m, "a");
    down(&mut m, "Tab");
    typed(&mut m, "b");
    // Released by the right button (its own loop, CLT-Q72 (a)).
    m.moved(OK_AT);
    m.press(OK_AT);
    assert_eq!(m.right_release().ended, Some(Ended::Multiplayer(true)));
}

#[test]
fn a_right_press_shows_the_items_help_while_held() {
    let mut m = opened(Bugs::Fixed);
    assert_eq!(m.right(MAP), Did::default());
    assert!(m.right(HOT).changed);
    assert!(!m.cursor());
    assert!(!m.box_draws("").is_empty());
    assert_eq!(m.moved(MAP), Did::default());
    assert_eq!(m.right(HOT), Did::default());
    assert_eq!(m.key(plain(0x1E), None, "A"), Did::default());
    assert_eq!(m.key_up("A"), Did::default());
    assert!(m.right_release().changed);
    assert!(m.cursor());
    // A left press or release closes it.
    m.right(HOT);
    assert!(m.press(HOT).changed);
    m.right(HOT);
    assert!(m.release().changed);
    // None while a button is held.
    m.press(IPX);
    assert_eq!(m.right(HOT), Did::default());
    m.moved(MAP);
    m.release();
    // The hot seat's: OK's while disabled, then closed by the release.
    click(&mut m, HOT);
    assert_eq!(m.right([230, 110]), Did::default());
    assert!(m.right(OK_AT).changed);
    assert!(!m.cursor());
    assert!(!m.box_draws("").is_empty());
    assert_eq!(m.moved(MAP), Did::default());
    assert_eq!(m.right(OK_AT), Did::default());
    assert_eq!(m.key(plain(0x1E), None, "A"), Did::default());
    assert_eq!(m.key_up("A"), Did::default());
    assert!(m.right_release().changed);
    m.right(OK_AT);
    assert!(m.press(OK_AT).changed);
    m.right(OK_AT);
    assert!(m.release().changed);
    assert!(m.cursor());
}

#[test]
fn the_network_modes_show_the_notice_and_stay_on_the_first_stage() {
    for at in [IPX, [578, 271], [578, 328], [578, 385], ONLINE_AT] {
        let mut m = opened(Bugs::Fixed);
        let did = click(&mut m, at);
        assert!(did.changed && did.ended.is_none());
        assert!(m.noticing());
        assert_eq!(m.context(), Context::Question);
        assert!(!m.box_draws("").is_empty());
        // The notice takes the keys: its OK by Enter.
        assert_eq!(m.key(None, None, "A"), Did::default());
        let down = m.key(None, Some(Command::Message(message::Key::Ok)), "Return");
        assert!(down.sample);
        assert!(m.key_up("Return").changed);
        assert!(!m.noticing());
        // The hint under the pointer at its close.
        assert_eq!(
            m.status(),
            m.texts.hints.get(&m.part.item_at(at).unwrap()).unwrap()
        );
    }
    // Its OK by the mouse; a move over it; the right button.
    let mut m = opened(Bugs::Fixed);
    click(&mut m, IPX);
    let ok = [400, 300];
    let _ = m.moved(ok);
    assert_eq!(m.right_release(), Did::default());
    assert_eq!(m.right(ok), Did::default());
    let ok_at = notice_ok(&m);
    m.press(ok_at);
    assert!(m.right_release().changed);
    assert!(!m.noticing());
    click(&mut m, IPX);
    let ok_at = notice_ok(&m);
    m.press(ok_at);
    assert!(m.release().changed);
    assert!(!m.noticing());
}

/// The notice's OK on the screen.
fn notice_ok(m: &Multiplayer) -> [i32; 2] {
    let draws = m.box_draws("");
    draws
        .iter()
        .find_map(|d| match d {
            Draw::Button { def, x, y, .. } if def.eq_ignore_ascii_case("iOkay.def") => {
                Some([x + 10, y + 10])
            }
            _ => None,
        })
        .unwrap()
}

#[test]
fn cancel_and_escape_end_the_dialog() {
    let mut m = opened(Bugs::Fixed);
    assert_eq!(
        click(&mut m, CANCEL_AT).ended,
        Some(Ended::Multiplayer(false))
    );
    let mut m = opened(Bugs::Fixed);
    let down = m.key(plain(edit::ESCAPE), Some(Command::Dialog(CANCEL)), "Escape");
    assert!(down.sample);
    assert_eq!(m.key_up("Escape").ended, Some(Ended::Multiplayer(false)));
    // Another command does nothing here; a key without a scan code goes to the table.
    let mut m = opened(Bugs::Fixed);
    assert_eq!(
        m.key(None, Some(Command::Dialog(OK)), "Return"),
        Did::default()
    );
    assert_eq!(m.key(None, None, "F1"), Did::default());
    // A key up that holds nothing.
    assert_eq!(m.key_up("F1"), Did::default());
}

#[test]
fn the_right_button_releases_a_held_button_and_the_notice_answers_every_input() {
    // A button held by the left button and released by the right (CLT-Q72 (a)).
    let mut m = opened(Bugs::Fixed);
    m.moved(HOT);
    m.press(HOT);
    assert!(m.right_release().changed);
    assert!(m.hot_seat().is_some());
    // The notice: a release with nothing held; its OK pressed, left and released off it.
    let mut m = opened(Bugs::Fixed);
    click(&mut m, IPX);
    assert_eq!(m.release(), Did::default());
    let ok = notice_ok(&m);
    m.press(ok);
    assert!(m.moved([5, 5]).changed);
    assert!(m.release().changed);
    assert!(m.noticing());
}

#[test]
fn the_hover_keeps_a_disabled_button_that_the_taking_search_skips() {
    // M2's review (fix 7): the dialogs' search (`Part::item_at`) against `Dialog::item_taking`,
    // every third pixel. The same item everywhere over the Multiplayer dialog and over the hot
    // seat's names but at their OK while disabled: `dialog_item_at` 0x5FF3E0 does not test the
    // 0x20 `dialog_item_enable` sets (design §16.2: its hint and popup kept, recorded), the state
    // the dialogs keep as `disabled`, which `item_taking` reads as the bits 8 and 0x1000 (the
    // spell book's) and skips. So the dialogs keep their own search.
    let differ = |part: &Part| -> Vec<[i32; 2]> {
        (0..600)
            .step_by(3)
            .flat_map(|y| (0..800).step_by(3).map(move |x| [x, y]))
            .filter(|&p| part.item_at(p) != part.dialog.item_taking(p, &part.shown).map(Item::id))
            .collect()
    };
    let mut m = opened(Bugs::Fixed);
    assert_eq!(differ(&m.part), Vec::<[i32; 2]>::new());
    click(&mut m, HOT);
    let hot_seat = |m: &Multiplayer| match &m.over {
        Over::HotSeat(h) => h.part.clone(),
        _ => panic!("the names"),
    };
    let part = hot_seat(&m);
    let at = differ(&part);
    assert!(at.contains(&OK_AT));
    assert!(at.iter().all(|&p| part.item_at(p) == Some(OK)));
    typed(&mut m, "a");
    down(&mut m, "Tab");
    typed(&mut m, "b");
    assert!(m.hot_seat().unwrap().ok_enabled());
    assert_eq!(differ(&hot_seat(&m)), Vec::<[i32; 2]>::new());
}
