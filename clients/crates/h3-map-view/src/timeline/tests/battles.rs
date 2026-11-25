//! A battle's switch and its result on the map (E24 M4 T5b2, design §5.21): the map's fade out,
//! the battle's fade in from its first frame, its result's OK, the battle's fade out, the map
//! back and faded in; then a beaten monster taken off the map, a retreated hero gone and the next
//! chosen, or a mine's guards' boxes and its flag.

use super::*;

use h3_ui::message::Style;

use super::turns::{laid, started_by};

/// The recorded sequences (`clients/map/sequences.toml`).
const SEQUENCES: &str = include_str!("../../../../../map/sequences.toml");

/// The sequences' situations, timelines and kept live parts.
const BATTLING: [(&str, &str, &str, &str); 3] = [
    (
        "battle-won",
        include_str!("../../../../../map/situations/seq-battle-won.toml"),
        include_str!("../../../../../map/timelines/battle-won.txt"),
        include_str!("../../../../../map/timelines/battle-won-live.toml"),
    ),
    (
        "battle-retreat",
        include_str!("../../../../../map/situations/seq-battle-retreat.toml"),
        include_str!("../../../../../map/timelines/battle-retreat.txt"),
        include_str!("../../../../../map/timelines/battle-retreat-live.toml"),
    ),
    (
        "battle-mine",
        include_str!("../../../../../map/situations/seq-battle-mine.toml"),
        include_str!("../../../../../map/timelines/battle-mine.txt"),
        include_str!("../../../../../map/timelines/battle-mine-live.toml"),
    ),
];

/// The monster's tile and the mine's entrance: above the hero's (5, 5).
const AT: Pos = Pos { x: 5, y: 4, z: 0 };

/// The monster or the mine: the map's fifth object (three heroes and a town before it).
const OBJECT: usize = 4;

/// The battle's number (the record's `GameEvents`').
const BATTLE: u32 = 4;

/// A fight's facts.
fn fight(winner: i32, after: AfterBattle) -> Fight {
    Fight {
        attacker: 278,
        battle: BATTLE,
        winner,
        after,
        experience: 15,
    }
}

/// A sequence's find as the binary builds it from a record (the boxes' texts here their own): the
/// Peasants fought at once, beaten or retreated from; the mine's question, its OK the battle, its
/// boxes after it and the flag.
fn battled(id: &str) -> Found {
    let plain = Found {
        at: AT,
        first: Vec::new(),
        again: None,
        gain: Gain::default(),
        found: None,
        taken: None,
        event: false,
        blocks: Some(OBJECT),
        ask: None,
        flag: None,
        fight: None,
        walk: None,
        levels: Vec::new(),
    };
    match id {
        "battle-won" => Found {
            fight: Some(fight(0, AfterBattle::Taken)),
            ..plain
        },
        "battle-retreat" => Found {
            fight: Some(fight(1, AfterBattle::Lost)),
            ..plain
        },
        _ => Found {
            blocks: None,
            ask: Some(Question {
                asked: laid("Guards?", Style::OkCancel),
                chooses: false,
                yes: Some(Answer {
                    boxes: vec![
                        (laid("Beaten.", Style::Ok), Gain::default()),
                        (laid("Yours now.", Style::Ok), Gain::default()),
                    ],
                    fight: Some(fight(0, AfterBattle::Boxes)),
                    flag: Some((OBJECT, 0)),
                    ..Answer::default()
                }),
                no: None,
                attacker: None,
                second: None,
            }),
            ..plain
        },
    }
}

/// A battle sequence's start, its actions, its end and its timeline.
fn battling(id: &str) -> (Start, BTreeMap<u32, Vec<Action>>, u32, &'static str) {
    let sequences = Sequences::parse(SEQUENCES).unwrap();
    let entry = BATTLING.iter().find(|(b, ..)| *b == id).unwrap();
    let (mut start, actions, ms, timeline) = started_by(sequences.get(id).unwrap(), entry);
    start.finds = vec![battled(id)];
    (start, actions, ms, timeline)
}

#[test]
fn replays_the_recorded_battles() {
    for (id, ..) in BATTLING {
        let (start, actions, ms, timeline) = battling(id);
        let states = play(rules(Bugs::Original), &start, &actions, ms).unwrap();
        let got = lines(&states, &BTreeMap::new());
        let want = without_screens(timeline);
        for (n, (g, w)) in got.iter().zip(&want).enumerate() {
            assert_eq!(g, w, "{id}: line {n}");
        }
        assert_eq!(got.len(), want.len(), "{id}");
    }
}

/// The loop of a battle sequence run to `until` with its actions.
fn run_to(id: &str, until: u32) -> Loop {
    let (start, actions, _, _) = battling(id);
    run_start(&start, &actions, until)
}

/// The loop of `start` run to `until` with `actions`.
fn run_start(start: &Start, actions: &BTreeMap<u32, Vec<Action>>, until: u32) -> Loop {
    let mut l = Loop::start(rules(Bugs::Original), start).unwrap();
    for t in 1..=until {
        l.run(t, actions.get(&t).map_or(&[][..], Vec::as_slice))
            .unwrap();
    }
    l
}

/// The switch a draw shows.
const fn switch(battle: Option<u32>, shift: u8, ended: bool) -> Option<Switch> {
    Some(Switch {
        battle,
        shift,
        ended,
    })
}

#[test]
fn the_switch_fades_the_map_out_the_battle_in_and_back() {
    // `battle-won`: the attack at 100 (the busy cursor), the map's shifts 1 and 2 at 150 and 200
    // of the screen as it showed; at 250 the combat cursor and the battle's first frame at its
    // shift 2, 1 at 300, whole at 350 (its line, the battle handed over then); Enter's release at
    // 560 its fade out, the battle's last frame (not kept); at 710 the map back at its shift 2
    // with the adventure cursor, whole at 810.
    let shown = |until| {
        let l = run_to("battle-won", until);
        (l.drawn().switch, l.cursor().kind)
    };
    assert_eq!(shown(100), (None, DEFAULT_CURSOR));
    assert_eq!(shown(149), (None, DEFAULT_CURSOR));
    assert_eq!(shown(150), (switch(None, 1, false), DEFAULT_CURSOR));
    assert_eq!(shown(200), (switch(None, 2, false), DEFAULT_CURSOR));
    assert_eq!(shown(250), (switch(Some(BATTLE), 2, false), BATTLE_CURSOR));
    assert_eq!(shown(300), (switch(Some(BATTLE), 1, false), BATTLE_CURSOR));
    let l = run_to("battle-won", 350);
    assert_eq!(l.drawn().switch, switch(Some(BATTLE), 0, false));
    assert_eq!(l.state(350).battle, Some(0));
    assert_eq!(l.state(351).battle, None);
    assert_eq!(l.battles(), [(350, BATTLE)]);
    assert_eq!(shown(559), (switch(Some(BATTLE), 0, false), BATTLE_CURSOR));
    assert_eq!(shown(560), (switch(Some(BATTLE), 0, true), BATTLE_CURSOR));
    assert_eq!(shown(660), (switch(Some(BATTLE), 2, true), BATTLE_CURSOR));
    assert_eq!(shown(710), (switch(None, 2, false), CURSOR_TYPE));
    assert_eq!(shown(760), (switch(None, 1, false), CURSOR_TYPE));
    let l = run_to("battle-won", 810);
    assert_eq!(l.drawn().switch, None);
    assert!(l.switching.is_none() && l.dissolving.is_some());
    // The walk's end lists the switched battle no more.
    let l = run_to("battle-won", 1600);
    assert_eq!(l.battles(), [(350, BATTLE)]);
    assert!(l.drawn().gone.contains(&OBJECT));
}

#[test]
fn a_battle_puts_the_hero_on_its_tile_until_the_map_is_back() {
    // E24 M5 T1a (design §5.22): `adventure_hero_stop` at the battle's start draws the hero on its
    // tile (no pass 9 blend) in the screen the map's fade out starts from; the map comes back
    // with it there and the mouse hidden, and the fade in's end lifts it again, the mouse shown.
    let l = run_to("battle-won", 99);
    assert!(l.drawn().lifted.is_none());
    let l = run_to("battle-won", 100);
    assert!(l.drawn().hero.is_some() && l.drawn().lifted.is_none());
    let before = l.switching.as_ref().map(|s| s.before.lifted);
    assert_eq!(before, Some(None));
    for t in [710, 760] {
        let l = run_to("battle-won", t);
        assert!(l.drawn().lifted.is_none() && !l.cursor().shown, "{t}");
    }
    // At 810 the monster's dissolve starts from the view with the hero lifted again.
    let l = run_to("battle-won", 810);
    let before = l.dissolving.as_ref().map(|d| d.before.lifted.is_some());
    assert_eq!(before, Some(true));
    // A mine's guards: the view drawn at the question's close, then the objects stepped (the
    // drawn steps as before, memory's one more).
    let l = run_to("battle-mine", 1060);
    let early = run_to("battle-mine", 1059);
    assert!(l.drawn().lifted.is_none());
    assert_eq!(l.drawn().frames, early.drawn().frames);
    assert_eq!((early.state(1059).objects, l.state(1060).objects), (0, 1));
    let l = run_to("battle-mine", 1810);
    assert!(l.drawn().lifted.is_some() && l.cursor().shown);
}

#[test]
fn a_retreat_keeps_sleep_and_cast_until_the_next_choice() {
    // E24 M5 T1a (design §5.22): `battle-retreat` at 810 the view drawn with the hero lifted, then
    // it leaves; sleep and cast a spell drawn as its selection left them until the town is
    // chosen at 1330.
    let l = run_to("battle-retreat", 810);
    let before = l.dissolving.as_ref().map(|d| d.before.lifted.is_some());
    assert_eq!(before, Some(true));
    assert!(l.buttons().left);
    let off = |l: &Loop| panels::disabled(&l.kingdom, l.buttons(), 1);
    assert!(off(&l).contains(&(panels::id::SLEEP, false)));
    assert!(off(&l).contains(&(panels::id::CAST, false)));
    // The next hero chosen at the dissolve's end: the buttons as its selection's.
    let l = run_to("battle-retreat", 1330);
    assert!(!l.buttons().left && matches!(l.kingdom.selected, Selected::Hero(_)));
}

#[test]
fn the_host_closes_the_battle_and_nothing_else_counts_meanwhile() {
    // The host's close as Enter's release does; a step during the fade and a pointer during the
    // battle do nothing but move the mouse; the close outside a battle nothing.
    let (start, mut actions, _, _) = battling("battle-won");
    actions.retain(|&t, _| t < 500);
    actions.insert(200, vec![Action::Step(Direction::North)]);
    actions.insert(400, vec![Action::Pointer([600, 300])]);
    actions.insert(420, vec![Action::BattleClosed]);
    let l = run_start(&start, &actions, 470);
    assert_eq!(l.drawn().switch, switch(Some(BATTLE), 1, true));
    assert_eq!(l.pointer, [600, 300]);
    let mut l = run_start(&start, &BTreeMap::new(), 50);
    l.run(51, &[Action::BattleClosed]).unwrap();
    assert!(l.switching.is_none());
}

#[test]
fn a_battle_whose_monster_stands_opens_its_level_ups_then_ends_the_visit() {
    // E24 M4 T6 (the reviewer's R2-3): a fight whose record leaves the monster standing
    // (`AfterBattle::Stays`): the experience shown at the map's fade in's end, the visit's own
    // level-ups then (before, none: the next find's), the visit's end after their last.
    let (mut start, mut actions, _, _) = battling("battle-won");
    actions.retain(|&t, _| t < 810);
    start.finds[0].fight = Some(fight(0, AfterBattle::Stays));
    start.finds[0].levels = vec![LevelBox {
        level: 2,
        dialog: laid("Level 2.", Style::Ok),
        offers: Vec::new(),
        primary: 1,
        chosen: None,
    }];
    let l = run_start(&start, &actions, 810);
    assert!(l.boxed() && l.asks());
    actions.insert(900, vec![Action::BoxKey(Key::Ok)]);
    actions.insert(960, vec![Action::BoxKeyUp(Key::Ok)]);
    let l = run_start(&start, &actions, 1000);
    assert!(!l.boxed() && l.dissolving.is_none());
    assert!(!l.drawn().gone.contains(&OBJECT));
    // A recording's hero whose experience reaches a level with no dialog: refused there.
    start.finds[0].levels.clear();
    start.experience = Some(990);
    let mut l = Loop::start(rules(Bugs::Original), &start).unwrap();
    let refused = (1..=810).find_map(|t| {
        l.run(t, actions.get(&t).map_or(&[][..], Vec::as_slice))
            .err()
            .map(|why| (t, why))
    });
    assert_eq!(refused, Some((810, Unplayable::LevelUp(2))));
}

#[test]
fn a_retreat_s_hero_leaves_and_the_next_is_chosen() {
    // `battle-retreat`: at 810 the hero deselected, drawn nowhere from its dissolve's first step,
    // its slot gone at once, the second hero chosen at the dissolve's end.
    let l = run_to("battle-retreat", 810);
    assert_eq!(l.state(810).hero, None);
    assert_eq!(l.drawn().left, [0]);
    assert_eq!(l.kingdom.heroes.len(), 1);
    let l = run_to("battle-retreat", 1330);
    assert_eq!(
        l.state(1330).hero.map(|h| h.at),
        Some(Pos { x: 30, y: 5, z: 0 })
    );
    // With no hero left the first town is chosen; with none either, nothing.
    let (mut start, actions, _, _) = battling("battle-retreat");
    start.kingdom.heroes.truncate(1);
    start.chosen.heroes.truncate(1);
    start.chosen.towns = vec![Some(ChosenTown {
        at: Pos { x: 16, y: 12, z: 0 },
        status: crate::live::TownStatus {
            name: crate::live::TownName::Record(0),
            buildings: 0,
            garrison: false,
            army: [None; 7],
            silo: Vec::new(),
            income: 0,
        },
    })];
    let l = run_start(&start, &actions, 1330);
    assert_eq!(l.kingdom.selected, Selected::Town(0));
    // A town chosen disables sleep and cast a spell again (E24 M5 T1a).
    let off = panels::disabled(&l.kingdom, l.buttons(), 1);
    assert!(off.contains(&(panels::id::SLEEP, true)) && off.contains(&(panels::id::CAST, true)));
    start.chosen.towns.clear();
    let l = run_start(&start, &actions, 1330);
    assert_eq!(l.kingdom.selected, Selected::None);
}

#[test]
fn a_mine_s_guards_boxes_and_flag_after_the_battle() {
    // `battle-mine`: the question's OK at 1060 the switch (the objects stepped once), the battle
    // whole at 1310; at the map's fade in's end, 1810, the guards' box, then the flag's; the flag
    // raised as the last closes, at 2960.
    let l = run_to("battle-mine", 1060);
    assert_eq!(l.state(1060).objects, 1);
    assert!(l.switching.is_some());
    let l = run_to("battle-mine", 1810);
    assert!(l.drawn().message.is_some() && l.owners.is_empty());
    let l = run_to("battle-mine", 2959);
    assert!(l.owners.is_empty());
    let l = run_to("battle-mine", 2960);
    assert_eq!(l.owners.to_vec(), [(OBJECT, 0)]);
}

#[test]
fn what_a_battle_s_end_leaves() {
    // A monster that stands, a taken one without an object, a guards' battle without its answer:
    // the visit ended, the experience shown.
    let mut l = run_to("battle-won", 1);
    for after in [AfterBattle::Stays, AfterBattle::Taken, AfterBattle::Boxes] {
        let switching = Switching {
            phase: Phase::Map,
            start: 1,
            before: Arc::new(l.drawn().clone()),
            find: 1,
            fight: fight(0, after),
            rest: None,
        };
        l.after_battle(2, switching).unwrap();
        assert!(l.dissolving.is_none() && l.boxed.is_none(), "{after:?}");
    }
    // Nothing under way: no step, no phase, no back.
    l.switch_step(3).unwrap();
    l.rephase(3, Phase::Battle);
    l.switch_back(3);
    assert!(l.switching.is_none());
}

#[test]
fn a_find_s_battles() {
    // Its own fight's and its question's answers', in order.
    let won = battled("battle-won");
    assert_eq!(won.battles(), [BATTLE]);
    let mut mine = battled("battle-mine");
    assert_eq!(mine.battles(), [BATTLE]);
    if let Some(question) = &mut mine.ask {
        question.no = Some(Answer {
            fight: Some(Fight {
                battle: 9,
                ..fight(1, AfterBattle::Stays)
            }),
            ..Answer::default()
        });
    }
    assert_eq!(mine.battles(), [BATTLE, 9]);
    assert!(battled("battle-won").ask.is_none());
    let plain = Found { fight: None, ..won };
    assert!(plain.battles().is_empty());
}

#[test]
fn a_battle_s_line() {
    // The battle shown whole: its winner, before the screen's line.
    let (start, actions, ms, _) = battling("battle-retreat");
    let states = play(rules(Bugs::Original), &start, &actions, ms).unwrap();
    let screens = BTreeMap::from([(350, [0, 0, 800, 600])]);
    let got = lines(&states, &screens);
    let at = got.iter().position(|l| l == "350 battle 1").unwrap();
    assert_eq!(got[at + 1], "350 screen 0,0,800,600");
}

#[test]
fn what_a_battle_s_result_refuses() {
    // A guards' answer with no box whose experience reaches a level the loop has no dialog for:
    // refused where the map comes back (1810).
    let (mut start, actions, _, _) = battling("battle-mine");
    start.experience = Some(0);
    if let Some(Answer { boxes, fight, .. }) =
        start.finds[0].ask.as_mut().and_then(|q| q.yes.as_mut())
    {
        boxes.clear();
        if let Some(fight) = fight {
            fight.experience = 5000;
        }
    }
    let mut l = Loop::start(rules(Bugs::Original), &start).unwrap();
    let refused = (1..=1810).find_map(|t| {
        l.run(t, actions.get(&t).map_or(&[][..], Vec::as_slice))
            .err()
            .map(|why| (t, why))
    });
    assert_eq!(refused, Some((1810, Unplayable::LevelUp(2))));
    // A retreated hero's next one without its facts: refused at the dissolve's end (1330).
    let (mut start, actions, _, _) = battling("battle-retreat");
    start.chosen.heroes.truncate(1);
    let mut l = Loop::start(rules(Bugs::Original), &start).unwrap();
    let refused = (1..=1330).find_map(|t| {
        l.run(t, actions.get(&t).map_or(&[][..], Vec::as_slice))
            .err()
            .map(|why| (t, why))
    });
    assert_eq!(refused.map(|(t, _)| t), Some(1330));
}

/// A one-tile walk onto `to` as a record gives it, with its battle if any.
fn record_walk(to: Pos, battle: Option<u32>) -> Move {
    Move {
        tiles: vec![to],
        points: vec![Some(1460)],
        revealed: vec![Vec::new()],
        sea: None,
        battle,
        boats: Vec::new(),
        stops: None,
    }
}

/// A battle sequence's start as a record's (E24 M4 T6, the reviewer's R5-11): the engine's walks
/// `moves` with their paths (each one arrow, the destination), no recorded input.
fn as_record(id: &str, moves: Vec<Move>) -> Start {
    let (mut start, ..) = battling(id);
    start.paths = moves
        .iter()
        .map(|m| {
            m.tiles
                .iter()
                .map(|&at| Arrow {
                    at,
                    arrow: DESTINATION,
                })
                .collect()
        })
        .collect();
    start.moves = Some(moves);
    start
}

/// The record player's input for `commands` from `start` (the heroes 1 and 2 in slots 0 and 1),
/// and the loop run with it to the record's end.
fn directed(start: &Start, commands: &[Command]) -> (crate::director::Directed, Loop) {
    let d = crate::director::direct(
        rules(Bugs::Original),
        start,
        commands,
        &[(1, 0), (2, 1)],
        crate::director::Pace::default(),
        20_000,
    )
    .unwrap();
    let l = run_start(start, &d.actions, d.end);
    (d, l)
}

use crate::engine::Command;

#[test]
fn a_record_s_walk_onto_a_monster_fights_it_from_beside() {
    // E24 M4 T6 (the reviewer's R5-1, R5-3, R5-7): the record's walk onto the monster's tile is
    // its visit from beside (the hero stays on (5, 5), turned north), the switch as a key's
    // step's; the record player waits through it and closes the battle where it shows whole, that
    // millisecond's change the battle's fade out's first; the monster taken off, the record at its
    // end with the map idle; the battle the map's own, listed once.
    let start = as_record("battle-won", vec![record_walk(AT, Some(BATTLE))]);
    let walk = Command::Walk { hero: 1, to: AT };
    let (d, l) = directed(&start, &[walk]);
    let [(whole, n)] = l.battles() else {
        panic!("one battle handed over: {:?}", l.battles());
    };
    assert_eq!(*n, BATTLE);
    assert!(d.battles.is_empty());
    assert!(d.actions[whole].contains(&Action::BattleClosed));
    let at_whole = run_start(&start, &d.actions, *whole);
    assert_eq!(at_whole.drawn().switch, switch(Some(BATTLE), 0, true));
    assert_eq!(at_whole.state(*whole).battle, Some(0));
    assert_eq!(
        l.hero.map(|h| (h.at, h.facing)),
        Some((Pos { x: 5, y: 5, z: 0 }, Direction::North))
    );
    assert!(l.drawn().gone.contains(&OBJECT));
    assert!(l.idle() && l.walks_left() == 0);
}

#[test]
fn a_record_s_mine_guards_battle_is_handed_over_once_and_its_boxes_answered() {
    // E24 M4 T6 (R5-2, R5-3): the mine's question answered, its battle handed over by the switch
    // alone (none of the record player's own), the switch waited through and closed, the boxes
    // after it given their OK, the flag raised at the record's end.
    let start = as_record("battle-mine", vec![record_walk(AT, None)]);
    let accept = Command::Answer(h3_protocol::wire::game::Answer::Offer {
        consent: h3_protocol::wire::game::Consent::Accept,
    });
    let commands = [
        Command::Walk { hero: 1, to: AT },
        accept,
        Command::Battle(BATTLE),
    ];
    let (d, l) = directed(&start, &commands);
    assert!(d.battles.is_empty());
    assert_eq!(l.battles().len(), 1);
    assert!(l.switching.is_none() && l.boxed.is_none());
    assert_eq!(l.owners.to_vec(), [(OBJECT, 0)]);
}

#[test]
fn a_record_s_hero_gone_after_a_battle_moves_the_later_heroes_up() {
    // E24 M4 T6 (R5-6's note): the first hero retreats from the monster; the second, chosen
    // then, is the list's first: its walk's choice is its slot now, nothing chosen again.
    let east = Pos { x: 31, y: 5, z: 0 };
    let start = as_record(
        "battle-retreat",
        vec![record_walk(AT, Some(BATTLE)), record_walk(east, None)],
    );
    let commands = [
        Command::Walk { hero: 1, to: AT },
        Command::Walk { hero: 2, to: east },
    ];
    let (d, l) = directed(&start, &commands);
    assert_eq!(l.slots_left(), [0]);
    let chosen: Vec<_> = d
        .actions
        .values()
        .flatten()
        .filter(|a| matches!(a, Action::Choose(_)))
        .collect();
    assert_eq!(chosen, [&Action::Choose(0), &Action::Choose(0)]);
    assert_eq!(l.hero.map(|h| h.at), Some(east));
}

#[test]
fn what_a_record_s_battle_leaves() {
    // E24 M4 T6 (R5-4, R3-2): a monster's by what its end took off the map; a mine's guards'
    // boxes only after the hero's win, the hero gone where the record takes it.
    assert_eq!(AfterBattle::of_fight(true, false), AfterBattle::Taken);
    assert_eq!(AfterBattle::of_fight(false, true), AfterBattle::Lost);
    assert_eq!(AfterBattle::of_fight(true, true), AfterBattle::Both);
    assert_eq!(AfterBattle::of_fight(false, false), AfterBattle::Stays);
    assert_eq!(AfterBattle::of_guards(0, false), AfterBattle::Boxes);
    assert_eq!(AfterBattle::of_guards(1, true), AfterBattle::Lost);
    assert_eq!(AfterBattle::of_guards(-1, true), AfterBattle::Lost);
    assert_eq!(AfterBattle::of_guards(1, false), AfterBattle::Stays);
}

#[test]
fn a_mine_s_guards_beaten_the_hero_leaves_with_no_box_and_no_flag() {
    // E24 M4 T6 (R5-4, R3-2): the question's OK, the battle lost: no guards' box, no mine's box,
    // no flag; the hero gone at the map's fade in's end as a retreat's, the next one chosen.
    let (mut start, actions, _, _) = battling("battle-mine");
    if let Some(answer) = start.finds[0].ask.as_mut().and_then(|q| q.yes.as_mut()) {
        answer.boxes.clear();
        answer.flag = None;
        if let Some(fight) = &mut answer.fight {
            fight.winner = 1;
            fight.after = AfterBattle::of_guards(1, true);
        }
    }
    let l = run_start(&start, &actions, 1810);
    assert_eq!(l.state(1810).hero, None);
    assert!(l.boxed.is_none() && l.owners.is_empty());
    let l = run_start(&start, &actions, 2400);
    assert!(l.hero.is_some() && l.owners.is_empty() && l.boxed.is_none());
}

#[test]
fn a_draw_takes_the_hero_then_the_monster() {
    // E24 M4 T6 (R5-4): the hero's dissolve first (gone at 810, the next chosen at its end), then
    // the monster's (`killfade.82M`, 8 steps of 65 ms), the mouse shown at its end.
    let (mut start, actions, _, _) = battling("battle-retreat");
    start.finds[0].fight = Some(fight(-1, AfterBattle::of_fight(true, true)));
    let l = run_start(&start, &actions, 810);
    assert_eq!(l.state(810).hero, None);
    assert!(!l.drawn().gone.contains(&OBJECT));
    let l = run_start(&start, &actions, 1330);
    assert!(l.hero.is_some() && l.dissolving.is_some());
    assert!(l.drawn().gone.contains(&OBJECT));
    let l = run_start(&start, &actions, 1900);
    assert!(l.dissolving.is_none() && l.cursor().shown);
    // An action given during the monster's dissolve waits for its end, which refuses it.
    let mut l = run_start(&start, &actions, 1400);
    l.run(1401, &[Action::Walk(Pos { x: 1, y: 1, z: 0 })])
        .unwrap();
    let refused = (1402..=1900).find_map(|t| l.run(t, &[]).err());
    assert_eq!(refused, Some(Unplayable::NoPath([1, 1, 0])));
}
