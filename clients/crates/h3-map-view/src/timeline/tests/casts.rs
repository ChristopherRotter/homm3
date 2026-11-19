//! A spell cast on the map (E24 M4 T2d3, design §5.8): the cast button's hotkey C opens the spell
//! book over the map lifted and shown, a click on its place closes it and casts Summon Boat: the
//! boat in sight dissolved off its tile, the boat beside the hero dissolved in, the cost spent
//! (MCL-Q14).

use super::*;

use crate::live::{Cast, Explored, LiveBoat, Moment, Summoned, Walking};
use crate::sound::Cue;

use super::turns::started_by;

/// The sequences' situations, timelines and kept live parts.
const CASTING: [(&str, &str, &str, &str); 3] = [
    (
        "cast-boat",
        include_str!("../../../../../map/situations/seq-cast-boat.toml"),
        include_str!("../../../../../map/timelines/cast-boat.txt"),
        include_str!("../../../../../map/timelines/cast-boat-live.toml"),
    ),
    (
        "cast-boat-far",
        include_str!("../../../../../map/situations/seq-cast-boat-far.toml"),
        include_str!("../../../../../map/timelines/cast-boat-far.txt"),
        include_str!("../../../../../map/timelines/cast-boat-far-live.toml"),
    ),
    (
        "cast-boat-new",
        include_str!("../../../../../map/situations/seq-cast-boat-new.toml"),
        include_str!("../../../../../map/timelines/cast-boat-new.txt"),
        include_str!("../../../../../map/timelines/cast-boat-new-live.toml"),
    ),
];

/// The live part's heroes, boats and casts as the binary's live part holds them.
fn live_part(text: &str) -> Live {
    let table: toml::Table = text.parse().unwrap();
    Live {
        objects: Vec::new(),
        heroes: LiveHero::read_all(&table["heroes"]).unwrap(),
        boats: LiveBoat::read_all(&table["boats"]).unwrap(),
        viewer: 0,
        explored: Explored::all(36, 1, true),
        path: Vec::new(),
        moment: Moment::default(),
        walking: Walking::default(),
        kingdom: Kingdom::default(),
        buttons: None,
        minimap: None,
        hover: None,
        hint: None,
        chosen: Chosen::default(),
        pressed: Vec::new(),
        turns: Vec::new(),
        computer: false,
        level_ups: Vec::new(),
        casts: Cast::read_all(&table["cast"]).unwrap(),
    }
}

/// A cast sequence's start (its boats and casts the live part's), its actions, its end and its
/// timeline.
fn casting(id: &str) -> (Start, BTreeMap<u32, Vec<Action>>, u32, &'static str) {
    let sequences = Sequences::parse(SEQUENCES).unwrap();
    let entry = CASTING.iter().find(|(c, ..)| *c == id).unwrap();
    let (mut start, actions, ms, timeline) = started_by(sequences.get(id).unwrap(), entry);
    let live = live_part(entry.3);
    start.boats = Boats::of(&live);
    start.casts = live.casts;
    (start, actions, ms, timeline)
}

#[test]
fn replays_the_recorded_casts() {
    for (id, ..) in CASTING {
        let (start, actions, ms, timeline) = casting(id);
        let states = play(rules(Bugs::Original), &start, &actions, ms).unwrap();
        let got = lines(&states, &BTreeMap::new());
        let want = without_screens(timeline);
        for (n, (g, w)) in got.iter().zip(&want).enumerate() {
            assert_eq!(g, w, "{id}: line {n}");
        }
        assert_eq!(got.len(), want.len(), "{id}");
    }
}

/// The loop of a cast sequence run to `until` under `bugs`, its start changed by `change`.
fn run_with(id: &str, until: u32, bugs: Bugs, change: impl FnOnce(&mut Start)) -> Loop {
    let (mut start, actions, _, _) = casting(id);
    change(&mut start);
    let mut l = Loop::start(rules(bugs), &start).unwrap();
    for t in 1..=until {
        l.run(t, actions.get(&t).map_or(&[][..], Vec::as_slice))
            .unwrap();
    }
    l
}

/// The same, as recorded.
fn run_to(id: &str, until: u32) -> Loop {
    run_with(id, until, Bugs::Original, |_| {})
}

/// The tile (x, y) of the surface.
fn tile(x: u8, y: u8) -> Pos {
    Pos { x, y, z: 0 }
}

/// The boat beside the hero after the cast: (9, 11), facing right.
fn summoned(kind: u8) -> LiveBoat {
    LiveBoat {
        at: tile(9, 11),
        kind,
        facing: Facing::Right,
    }
}

/// The boat's box at (14, 8) and the one beside the hero at (9, 11), the view's corner (0, 4).
const OFF: [i64; 4] = [416, 96, 545, 193];
const ON: [i64; 4] = [256, 192, 385, 289];

#[test]
fn the_book_holds_the_map_from_the_key_up_to_the_click() {
    // `cast-boat`: C's key up at 160 lifts the hero (the view already centred on it), shows the
    // view with the status bar empty and opens the book; until the click at 300 the palettes
    // step (200) and nothing is drawn, the objects' redraw due at 180 held.
    let l = run_to("cast-boat", 159);
    assert!(!l.state(159).book && l.drawn().lifted.is_none());
    let l = run_to("cast-boat", 160);
    let opened = l.drawn().clone();
    assert!(l.state(160).book && opened.lifted.is_some());
    assert_eq!(opened.hint, Hint::Empty);
    assert_eq!(opened.pressed, Vec::<u16>::new());
    assert_eq!((l.cursor().kind, l.cursor().frame), (CURSOR_TYPE, 0));
    let l = run_to("cast-boat", 299);
    let s = l.state(299);
    assert!(s.book);
    assert_eq!((s.cycles, s.objects), (1, 0));
    assert_eq!(s.drawn, opened);
    // The click closes it: the default cursor's frame 1 where the mouse manager last drew it,
    // shown through the cast.
    let l = run_to("cast-boat", 300);
    let s = l.state(300);
    assert!(!s.book);
    assert_eq!(
        s.cursor,
        Cursor {
            kind: DEFAULT_CURSOR,
            frame: 1,
            at: [700, 300],
            shown: true,
        }
    );
    assert_eq!(l.state(827).cursor, s.cursor);
    let s = run_to("cast-boat", 828).state(828);
    assert_eq!((s.cursor.kind, s.cursor.frame), (CURSOR_TYPE, 0));
}

#[test]
fn the_books_inputs_but_a_click_are_dropped() {
    // A step, a scroll, a key, a press and a move while the book shows: nothing moves, the mouse's
    // point read (the poll draws the cursor there).
    let mut l = run_to("cast-boat", 200);
    let before = l.state(200);
    l.run(
        201,
        &[
            Action::Step(Direction::North),
            Action::Scroll(Direction::East),
            Action::Key(Hotkey::NextHero),
            Action::Press([500, 300]),
            Action::Release([500, 300]),
            Action::Pointer([4, 300]),
        ],
    )
    .unwrap();
    for t in 202..=240 {
        l.run(t, &[]).unwrap();
    }
    let s = l.state(240);
    assert!(s.book && l.dissolving.is_none());
    assert_eq!(
        (s.view, s.hero, s.drawn),
        (before.view, before.hero, before.drawn)
    );
    assert_eq!(s.cursor.at, [4, 300]);
}

#[test]
fn the_cast_refused_without_its_facts() {
    // No `[[cast]]` table: the cast button's command not built; live it does nothing.
    let (mut start, actions, _, _) = casting("cast-boat");
    start.casts.clear();
    let mut l = Loop::start(rules(Bugs::Original), &start).unwrap();
    for t in 1..160 {
        l.run(t, actions.get(&t).map_or(&[][..], Vec::as_slice))
            .unwrap();
    }
    assert_eq!(
        l.run(160, &actions[&160]),
        Err(Unplayable::Command(panels::id::CAST))
    );
    let l = run_with("cast-boat", 400, Bugs::Original, |start| start.live = true);
    assert!(!l.state(400).book && l.drawn().lifted.is_none());
    // A table of another millisecond: the click refused.
    let (mut start, actions, _, _) = casting("cast-boat");
    start.casts[0].at = 301;
    let mut l = Loop::start(rules(Bugs::Original), &start).unwrap();
    for t in 1..300 {
        l.run(t, actions.get(&t).map_or(&[][..], Vec::as_slice))
            .unwrap();
    }
    assert_eq!(l.run(300, &actions[&300]), Err(Unplayable::Cast(300)));
    assert_eq!(
        Unplayable::Cast(300).to_string(),
        "the cast at 300 ms: its facts not given"
    );
    // Its cost spent at another millisecond, or never: the end refused.
    for spent in [Some(829), None] {
        let (mut start, actions, _, _) = casting("cast-boat");
        start.casts[0].spent = spent;
        let mut l = Loop::start(rules(Bugs::Original), &start).unwrap();
        for t in 1..828 {
            l.run(t, actions.get(&t).map_or(&[][..], Vec::as_slice))
                .unwrap();
        }
        assert_eq!(l.run(828, &[]), Err(Unplayable::Cast(828)));
    }
}

#[test]
fn only_summon_boat_with_one_boat_is_cast() {
    let refused = |change: fn(&mut Cast)| {
        let (mut start, actions, _, _) = casting("cast-boat");
        change(&mut start.casts[0]);
        let mut l = Loop::start(rules(Bugs::Original), &start).unwrap();
        for t in 1..300 {
            l.run(t, actions.get(&t).map_or(&[][..], Vec::as_slice))
                .unwrap();
        }
        l.run(300, &actions[&300])
    };
    assert_eq!(refused(|c| c.spell = 1), Err(Unplayable::Spell(1)));
    assert_eq!(refused(|c| c.boats.clear()), Err(Unplayable::Spell(0)));
    assert_eq!(
        refused(|c| c.boats.push(c.boats[0])),
        Err(Unplayable::Spell(0))
    );
    assert_eq!(
        Unplayable::Spell(1).to_string(),
        "spell 1's cast on the map (not built)"
    );
}

#[test]
fn the_boat_in_sight_dissolves_off_then_in_beside_the_hero() {
    // `cast-boat`: at 300 the boat off (14, 8), its box dissolving from the screen as the book
    // left it into the view without it, a step each 33 ms, the palettes cycling on.
    let shown = run_to("cast-boat", 299).drawn().clone();
    let l = run_to("cast-boat", 300);
    let off = l.drawn().clone();
    let dissolve = off.dissolve.clone().unwrap();
    assert_eq!((dissolve.rect, dissolve.step), (OFF, 0));
    assert_eq!((*dissolve.before).clone(), shown);
    assert_eq!(dissolve.from, None);
    assert!(off.boats.empty.is_empty());
    assert_eq!(off.cycles, 1);
    for (t, step) in [(332, 0), (333, 1), (531, 7), (563, 7)] {
        let d = run_to("cast-boat", t).drawn().dissolve.clone().unwrap();
        assert_eq!((t, d.rect, d.step), (t, OFF, step));
    }
    // At 564 the boat beside the hero, its box dissolving from the view the first dissolve drew
    // (the back buffer's) into the view with the boat, over the screen as that one left it.
    let l = run_to("cast-boat", 564);
    let on = l.drawn().clone();
    let dissolve = on.dissolve.clone().unwrap();
    assert_eq!((dissolve.rect, dissolve.step), (ON, 0));
    let back = Draw {
        dissolve: None,
        ..off.clone()
    };
    assert_eq!(dissolve.from.as_deref(), Some(&back));
    let earlier = dissolve.before.dissolve.clone().unwrap();
    assert_eq!((earlier.rect, earlier.step), (OFF, DISSOLVE_DONE));
    assert_eq!(*earlier.before, shown);
    assert_eq!(on.boats.empty, [summoned(1)]);
    assert_eq!(on.cycles, 2);
    let d = run_to("cast-boat", 795).drawn().dissolve.clone().unwrap();
    assert_eq!((d.rect, d.step), (ON, 7));
    // At 828 the end: the view drawn whole, the objects two steps on.
    let l = run_to("cast-boat", 828);
    assert_eq!(l.drawn().dissolve, None);
    assert_eq!(l.state(828).objects, 2);
    assert!(l.dissolving.is_none());
}

#[test]
fn a_boat_out_of_sight_or_made_dissolves_in_alone() {
    // `cast-boat-far`: the boat on (10, 1) above the view moved in the click's millisecond, its
    // arrival dissolving from the screen at once, ending at 564; `cast-boat-new`: one made, of
    // type 0.
    for (id, kind) in [("cast-boat-far", 1), ("cast-boat-new", 0)] {
        let shown = run_to(id, 299).drawn().clone();
        let l = run_to(id, 300);
        let dissolve = l.drawn().dissolve.clone().unwrap();
        assert_eq!((dissolve.rect, dissolve.step), (ON, 0), "{id}");
        assert_eq!(*dissolve.before, shown, "{id}");
        assert_eq!(dissolve.from, None, "{id}");
        assert_eq!(l.drawn().boats.empty, [summoned(kind)], "{id}");
        let l = run_to(id, 564);
        assert_eq!(l.drawn().dissolve, None, "{id}");
        assert_eq!(l.state(564).objects, 2, "{id}");
    }
}

#[test]
fn the_cost_spent_shows_in_the_window_and_the_slot_by_the_profile() {
    // MCL-Q14 seen: the original's slot shows its rectangle alone, so the mana bar stays as at 0
    // (10) while the status window, built again, shows the new mana (3); the client shows the
    // slot whole.
    let l = run_to("cast-boat", 827);
    assert_eq!(l.kingdom.heroes[0].mana, 10);
    for (bugs, slot) in [(Bugs::Original, 10), (Bugs::Fixed, 3)] {
        let l = run_with("cast-boat", 828, bugs, |_| {});
        assert_eq!(l.kingdom.heroes[0].mana, 3, "{bugs:?}");
        let drawn = l.drawn();
        assert_eq!(drawn.kingdom.heroes[0].mana, slot, "{bugs:?}");
        assert_eq!(drawn.kingdom.heroes[0].points, 1630, "{bugs:?}");
        let Some(Status::Hero(window)) = &drawn.kingdom.status else {
            panic!("{bugs:?}: no hero's window");
        };
        assert_eq!(window.mana, Some(3), "{bugs:?}");
        assert_eq!(
            l.chosen.heroes[0].as_ref().map(|c| c.status.mana),
            Some(Some(3))
        );
    }
}

/// The effects a cast sequence gives to `until`, its start with the map's sound.
fn effects(id: &str, until: u32) -> Vec<String> {
    let mut l = run_with(id, until, Bugs::Original, |start| {
        let tiles = crate::sound::tests::tiles(36, |_, _| 1, |_, _| None);
        start.sound = Some(crate::sound::tests::sound(tiles));
    });
    l.take_cues()
        .iter()
        .filter(|(_, c)| matches!(c, Cue::Effect(_)))
        .map(|(t, c)| crate::sound::line(*t, c))
        .collect()
}

#[test]
fn summon_boats_sound_as_its_boat_dissolves_in() {
    // The press's at C's key down, the spell's at its arrival's dissolve's start (the sound's
    // wait with sound on not built).
    let press = |t: u32| alloc::format!("{t} effect button.wav");
    assert_eq!(
        effects("cast-boat", 900),
        [press(100), "564 effect summboat.wav".to_string()]
    );
    assert_eq!(
        effects("cast-boat-new", 900),
        [press(100), "300 effect summboat.wav".to_string()]
    );
}

#[test]
fn a_boats_box_cut_to_the_clip_and_the_tiles_in_sight() {
    let l = run_to("cast-boat", 1);
    // The view's corner (0, 4): the first column and row cut at 8, the last column at 615.
    assert_eq!(l.boat_box(tile(0, 4)), [8, 8, 97, 65]);
    assert_eq!(l.boat_box(tile(18, 20)), [544, 480, 616, 552]);
    assert!(l.in_view(tile(0, 4)) && l.in_view(tile(18, 20)));
    for out in [tile(19, 10), tile(9, 21), tile(9, 3)] {
        assert!(!l.in_view(out), "{out:?}");
    }
    assert!(!l.in_view(Pos { x: 9, y: 10, z: 1 }));
    // A boat moved from a tile out of sight is taken off it all the same.
    let l = run_to("cast-boat-far", 300);
    assert_eq!(l.boats.empty, [summoned(1)]);
}

#[test]
fn a_second_cast_with_the_hero_lifted() {
    // After the first cast the hero stays lifted: C's key up at 1060 opens the book without a
    // lift or a centring; the second boat, made, dissolves in from the click at 1200 to 1464.
    let (mut start, mut actions, _, _) = casting("cast-boat");
    let mut made = start.casts[0].clone();
    made.at = 1200;
    made.spent = Some(1464);
    made.mana = 0;
    made.boats = vec![Summoned {
        from: None,
        to: LiveBoat {
            at: tile(10, 11),
            kind: 0,
            facing: Facing::Right,
        },
    }];
    start.casts.push(made);
    actions.insert(1000, vec![Action::Key(Hotkey::Cast)]);
    actions.insert(1060, vec![Action::KeyUp(Hotkey::Cast)]);
    actions.insert(1200, vec![Action::Click([246, 123])]);
    let states = play(rules(Bugs::Original), &start, &actions, 1500).unwrap();
    let at = |t: u32| &states[usize::try_from(t).unwrap()];
    assert!(at(1059).drawn.lifted.is_some() && !at(1059).book);
    assert!(at(1060).book && at(1199).book && !at(1200).book);
    assert_eq!(at(1060).view, at(1059).view);
    assert_eq!(at(1200).drawn.dissolve.as_ref().map(|d| d.step), Some(0));
    assert_eq!(at(1463).drawn.dissolve.as_ref().map(|d| d.step), Some(7));
    assert_eq!(at(1464).drawn.dissolve, None);
    assert_eq!(at(1464).drawn.boats.empty.len(), 2);
}

#[test]
fn a_casts_end_with_no_hero_selected_spends_nothing() {
    // Its hero chosen away from meanwhile (*provisional*: nothing does in the original): no mana
    // spent, no slot drawn, no window of the hero's built.
    let mut l = run_to("cast-boat", 500);
    l.kingdom.selected = Selected::None;
    l.chosen.heroes.clear();
    for t in 501..=828 {
        l.run(t, &[]).unwrap();
    }
    assert_eq!(l.kingdom.heroes[0].mana, 10);
    assert_eq!(l.drawn().kingdom.heroes[0].mana, 10);
}
