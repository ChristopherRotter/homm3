//! The screens' notice in play (E24 M5 T4c, design §5.28, `MCL-STUB-7`): a screen of M6 or M7
//! without a decision (the hero screen, the town screen, the spell book, the kingdom overview,
//! the quest log, the adventure options) shows its notice where the host's facts give it, its OK
//! closing it; a recording refuses the click as before, a live view without a host does nothing.

use super::*;

use h3_ui::message::Style;

use super::turns::laid;
use crate::notice::Notice;
use crate::reach::Hosted;

/// The host's facts with each screen's notice, its text its key.
fn notices() -> Hosted {
    Hosted {
        notices: Notice::SCREENS
            .map(|n| (n, laid(n.key(), Style::Ok)))
            .to_vec(),
        ..Hosted::default()
    }
}

/// The box the loop shows at `t`, and the box of `notice`.
fn shows(l: &Loop, t: u32, notice: Notice) -> bool {
    l.state(t).drawn.message == Some(laid(notice.key(), Style::Ok))
}

/// Enter on the open box at `t`.
fn enter(l: &mut Loop, t: u32) {
    l.run(t, &[Action::BoxKey(Key::Ok), Action::BoxKeyUp(Key::Ok)])
        .unwrap();
}

#[test]
fn the_panels_screens_in_play_show_their_notices() {
    let (start, _, _, _) = choosing("choose-lists");
    let r = rules(Bugs::Original);
    let mut play = start.clone();
    play.live = true;
    play.host = Some(notices());
    let mut l = Loop::start(r.clone(), &play).unwrap();
    // The selected hero's slot: the hero screen's notice, closed by its OK.
    l.run(1, &[Action::Press([641, 228])]).unwrap();
    assert!(shows(&l, 1, Notice::Hero));
    enter(&mut l, 2);
    assert!(l.boxed.is_none() && l.state(2).drawn.message.is_none());
    // A town chosen, then its slot again: the town screen's.
    l.run(3, &[Action::Press([771, 228])]).unwrap();
    assert_eq!(l.kingdom.selected, Selected::Town(0));
    assert!(l.boxed.is_none());
    l.run(4, &[Action::Press([771, 228])]).unwrap();
    assert!(shows(&l, 4, Notice::Town));
    enter(&mut l, 5);
    // The buttons without a screen: the kingdom overview, the quest log, the adventure options,
    // cast a spell's book.
    let mut t = 6;
    for (id, notice) in [
        (panels::id::KINGDOM, Notice::Kingdom),
        (panels::id::QUESTS, Notice::Quests),
        (panels::id::ADVENTURE, Notice::Adventure),
        (panels::id::CAST, Notice::Book),
    ] {
        l.command(t, id).unwrap();
        assert!(shows(&l, t, notice), "{notice:?}");
        enter(&mut l, t + 1);
        assert!(l.boxed.is_none(), "{notice:?}");
        t += 2;
    }
    // Without the notice in the host's facts: nothing, as live without one.
    let mut bare = play.clone();
    bare.host = Some(Hosted::default());
    let mut l = Loop::start(r.clone(), &bare).unwrap();
    l.run(1, &[Action::Press([641, 228])]).unwrap();
    l.command(2, panels::id::KINGDOM).unwrap();
    assert!(l.boxed.is_none());
    // A recording's: refused, as before.
    let mut l = Loop::start(r, &start).unwrap();
    assert_eq!(
        l.command(1, panels::id::QUESTS),
        Err(Unplayable::Command(panels::id::QUESTS))
    );
    assert_eq!(
        l.command(1, panels::id::ADVENTURE),
        Err(Unplayable::Command(panels::id::ADVENTURE))
    );
}

#[test]
fn the_selected_hero_or_a_town_of_the_viewers_clicked_on_the_map() {
    // `hovering`'s map: Red's hero A on (4, 3) selected, Red's town's body at (2, 2), Blue's
    // garrison's at (2, 4).
    let mut start = hovering_start(true);
    start.host = Some(notices());
    let r = rules(Bugs::Fixed);
    let mut l = Loop::start(r.clone(), &start).unwrap();
    l.run(1, &[Action::Click(on(4, 3))]).unwrap();
    assert!(shows(&l, 1, Notice::Hero));
    enter(&mut l, 2);
    // A hero selected: the town is a click's path, not its screen.
    l.run(3, &[Action::Click(on(2, 2))]).unwrap();
    assert!(l.boxed.is_none());
    // A town selected: a town of the viewer's its screen's notice; another's nothing.
    let mut town = start.clone();
    town.hero = None;
    town.kingdom.selected = Selected::Town(0);
    let mut l = Loop::start(r.clone(), &town).unwrap();
    l.run(1, &[Action::Click(on(2, 4))]).unwrap();
    assert!(l.boxed.is_none());
    l.run(2, &[Action::Click(on(2, 2))]).unwrap();
    assert!(shows(&l, 2, Notice::Town));
    // No mouse-over's facts: no town known.
    town.hover = None;
    let mut l = Loop::start(r.clone(), &town).unwrap();
    l.run(1, &[Action::Click(on(2, 2))]).unwrap();
    assert!(l.boxed.is_none());
    // Without the host's notices: the click as before (its reach asked of the host).
    let mut bare = start.clone();
    bare.host = Some(Hosted::default());
    bare.index = Some(0);
    let mut l = Loop::start(r, &bare).unwrap();
    l.run(1, &[Action::Click(on(4, 3))]).unwrap();
    assert!(l.boxed.is_none());
    assert!(l.reach_asked().is_some());
}
