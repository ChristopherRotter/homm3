//! A computer player's turn in sight (E24 M4 T1b2, design §5.2): its start, its heroes' walks
//! from the recording's facts, kind 5's steps, its end and the next day; the single draws at
//! speed 4 kept for the window.

use super::*;

use alloc::collections::BTreeMap;

use crate::art::{MapArt, Piece, TileArt};
use crate::live::{Explored, Live, LiveBoat, Moment, TurnStatus, Walking};
use crate::objects::{DrawLists, Shapes};
use crate::replay::Replay;
use crate::status::OtherArt;

/// E24 M4 T1b2's sequences: their situations, timelines and kept live parts.
const COMPUTING: [(&str, &str, &str, &str); 5] = [
    (
        "ai-walk",
        include_str!("../../../../../map/situations/seq-turn-ai.toml"),
        include_str!("../../../../../map/timelines/ai-walk.txt"),
        include_str!("../../../../../map/timelines/ai-walk-live.toml"),
    ),
    (
        "ai-walk-slow",
        include_str!("../../../../../map/situations/seq-turn-ai.toml"),
        include_str!("../../../../../map/timelines/ai-walk-slow.txt"),
        include_str!("../../../../../map/timelines/ai-walk-slow-live.toml"),
    ),
    (
        "ai-walk-fast",
        include_str!("../../../../../map/situations/seq-turn-ai.toml"),
        include_str!("../../../../../map/timelines/ai-walk-fast.txt"),
        include_str!("../../../../../map/timelines/ai-walk-fast-live.toml"),
    ),
    (
        "ai-walk-blackout",
        include_str!("../../../../../map/situations/seq-turn-ai.toml"),
        include_str!("../../../../../map/timelines/ai-walk-blackout.txt"),
        include_str!("../../../../../map/timelines/ai-walk-blackout-live.toml"),
    ),
    (
        "ai-walk-fog",
        include_str!("../../../../../map/situations/seq-turn-ai-fog.toml"),
        include_str!("../../../../../map/timelines/ai-walk-fog.txt"),
        include_str!("../../../../../map/timelines/ai-walk-fog-live.toml"),
    ),
];

/// A live part's heroes with their boats, as the binary reads them.
fn heroes_of(live: &str) -> Vec<LiveHero> {
    let table: toml::Table = live.parse().unwrap();
    LiveHero::read_all(&table["heroes"]).unwrap()
}

/// The mouse-over of a kept live part on the situations' 36-tile grass map without objects: its
/// facts and the viewer's explored tiles (all a computer's turn reads of it).
pub(super) fn hover_of(live: &str) -> Arc<Hover> {
    let table: toml::Table = live.parse().unwrap();
    let heroes = heroes_of(live);
    let objects = table["anchors"].as_array().unwrap().len();
    let facts = hover::Facts::read(&table["hover"], objects, heroes.len(), 36, 1).unwrap();
    let mut explored = Explored::all(36, 1, false);
    for (y, row) in table["explored"].as_array().unwrap().iter().enumerate() {
        for (x, tile) in row.as_str().unwrap().bytes().enumerate() {
            let [x, y] = [x, y].map(|v| i32::try_from(v).unwrap());
            explored.set(x, y, 0, tile == b'1');
        }
    }
    let grass = TileArt {
        terrain: Piece {
            code: 2,
            picture: 0,
            mirror_x: false,
            mirror_y: false,
        },
        river: None,
        road: None,
        flags: 0,
    };
    let art = MapArt {
        size: 36,
        levels: 1,
        tiles: vec![grass; 36 * 36],
        templates: Vec::new(),
        objects: Vec::new(),
        hero_names: BTreeMap::new(),
    };
    let live = Live {
        objects: Vec::new(),
        heroes,
        boats: Vec::new(),
        viewer: 0,
        explored,
        path: Vec::new(),
        moment: Moment::default(),
        walking: Walking::default(),
        kingdom: Kingdom::default(),
        buttons: None,
        minimap: None,
        hover: Some(facts),
        hint: None,
        chosen: Chosen::default(),
        pressed: Vec::new(),
        turns: Vec::new(),
        computer: false,
    };
    let text = include_str!("../../../../../map/assets/base.toml");
    let tables = tables::load(&[tables::parse(text).unwrap()], "base").unwrap();
    let lists = [DrawLists::build(&art, &live, &Shapes::default(), 0).unwrap()];
    Hover::new(&art, &live, &tables.records, &lists).unwrap()
}

/// A computer turn sequence's start (as the binary's `for_sequence` makes it): the kingdom and
/// the selected hero by the mouse-over's facts, the turns with their heroes standing and the
/// computer's walks from the `hero` lines, the points and the computer's options.
fn computing(id: &str) -> (Start, BTreeMap<u32, Vec<Action>>, u32, &'static str) {
    let sequences = Sequences::parse(SEQUENCES).unwrap();
    let sequence = sequences.get(id).unwrap();
    let (_, situation, timeline, live) = COMPUTING.iter().find(|(t, ..)| *t == id).unwrap();
    let mut start = start(situation, sequence.scroll_speed().unwrap());
    (start.kingdom, start.buttons) = live_kingdom(live);
    let heroes = heroes_of(live);
    let table: toml::Table = live.parse().unwrap();
    let hover = hover_of(live);
    start.heroes = heroes
        .iter()
        .map(|h| Hero {
            at: h.at,
            facing: Direction::ALL[usize::from(h.facing.number())],
        })
        .collect();
    let facts = hover.facts();
    let index = facts
        .heroes
        .iter()
        .position(|h| h.0 == facts.hero_list[0].0)
        .unwrap();
    start.index = Some(index);
    start.hero = Some(start.heroes[index]);
    start.hover = Some(hover);
    let lines = sequence.heroes(timeline).unwrap();
    let turns: Vec<Turn> = table["turn"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| Turn::read(t).unwrap())
        .collect();
    let boats = LiveBoat::read_all(&table["boats"]).unwrap();
    start.turns = turns
        .iter()
        .enumerate()
        .map(|(i, t)| NextTurn {
            turn: t.clone(),
            message: None,
            heroes: turn::placed(&heroes, &boats, t).unwrap(),
            walks: turn::walks(&turns, i, &heroes, &lines).unwrap(),
        })
        .collect();
    start.points = sequence.points(timeline).unwrap();
    start.computer = Computer {
        speed: sequence.computer_speed().unwrap(),
        blackout: sequence.blackout().unwrap(),
    };
    (start, sequence.actions().unwrap(), sequence.ms, timeline)
}

#[test]
fn replays_the_recorded_computer_turns() {
    for (id, ..) in COMPUTING {
        let (start, actions, ms, timeline) = computing(id);
        let states = play(rules(Bugs::Original), &start, &actions, ms).unwrap();
        let got = lines(&states, &BTreeMap::new());
        let want = without_screens(timeline);
        for (n, (g, w)) in got.iter().zip(&want).enumerate() {
            assert_eq!(g, w, "{id}: line {n}");
        }
        assert_eq!(got.len(), want.len(), "{id}");
    }
}

/// The loop of a recorded computer turn sequence run to `until` with its actions.
fn run_to(id: &str, until: u32) -> Loop {
    let (start, actions, _, _) = computing(id);
    let mut l = Loop::start(rules(Bugs::Original), &start).unwrap();
    for t in 1..=until {
        l.run(t, actions.get(&t).map_or(&[][..], Vec::as_slice))
            .unwrap();
    }
    l
}

#[test]
fn the_computers_screen() {
    // `ai-walk` at 160: the four buttons and the shield, the lists the viewer's with no selection,
    // kind 5 built (the crest Blue's), the view centred on Blue's hero (the map's border showing),
    // its first step's first frame; the cursor busy and hidden.
    let l = run_to("ai-walk", 160);
    let s = l.state(160);
    assert!(s.drawn.computer);
    assert_eq!(s.drawn.kingdom.selected, Selected::None);
    assert_eq!(s.drawn.kingdom.resources[6], 20_000);
    assert_eq!(
        s.drawn.kingdom.status,
        Some(Status::Turn(TurnStatus {
            crest: 1,
            sand: 0,
            glass: 0,
            value: 2560
        }))
    );
    assert_eq!(s.view, ViewAt { x: -3, y: 18, z: 0 });
    assert_eq!((s.drawn.selected, s.drawn.lifted), (Some(1), Some([7, 0])));
    assert_eq!(
        (s.cursor.kind, s.cursor.frame, s.cursor.shown),
        (DEFAULT_CURSOR, 2, false)
    );
    assert_eq!(
        (s.player, s.points, s.other),
        (1, Some(1560), Some([1, 0, 0]))
    );
    // The minimap's draw returns at once, the chooser builds nothing; the input waits.
    let mut l = l;
    let minimap = l.drawn.minimap;
    l.view = ViewAt { x: 5, y: 5, z: 0 };
    l.draw_minimap();
    l.choose(161, true);
    assert_eq!(
        (l.drawn.minimap, l.kingdom.status.clone()),
        (minimap, s.drawn.kingdom.status.clone())
    );
    l.run(
        161,
        &[
            Action::Click([300, 300]),
            Action::Key(Hotkey::EndTurn),
            Action::Press([310, 300]),
            Action::Pointer([320, 300]),
            Action::Release([330, 300]),
        ],
    )
    .unwrap();
    assert_eq!((l.pointer, l.player, l.next_turn), ([330, 300], 1, 1));
    // The next day: Blue's hero put back where its walk ended, the shield gone, the cursor the
    // adventure arrow shown again, the redraws missed caught up (two steps, the deadline 180
    // on).
    let l = run_to("ai-walk", 1660);
    let s = l.state(1660);
    assert!(!s.drawn.computer && l.ai.is_none());
    assert_eq!(l.heroes[1].at, Pos { x: 20, y: 24, z: 0 });
    assert_eq!(s.drawn.heroes[1].at, Pos { x: 20, y: 24, z: 0 });
    assert_eq!((s.cursor.kind, s.cursor.shown), (CURSOR_TYPE, true));
    assert_eq!((s.objects, l.frame_at), (2, 1840));
    assert_eq!((s.player, s.other), (0, None));
}

#[test]
fn the_unseen_steps_take_no_time() {
    // `ai-walk-fog`: the first three steps in Red's fog (no time, nothing drawn, their points
    // taken), the fourth seen by its destination's explored neighbour.
    let l = run_to("ai-walk-fog", 160);
    let s = l.state(160);
    assert_eq!(
        (s.hero.map(|h| h.at), s.points, s.view),
        (
            Some(Pos { x: 9, y: 26, z: 0 }),
            Some(1260),
            ViewAt { x: 0, y: 18, z: 0 }
        )
    );
    // Under Blackout Computer no step is seen: the whole turn in its millisecond, as at speed 4.
    let (start, actions, ..) = computing("ai-walk");
    for computer in [
        Computer {
            speed: 3,
            blackout: true,
        },
        Computer {
            speed: 4,
            blackout: false,
        },
    ] {
        let start = Start {
            computer,
            ..start.clone()
        };
        let mut l = Loop::start(rules(Bugs::Original), &start).unwrap();
        for t in 1..=160 {
            l.run(t, actions.get(&t).map_or(&[][..], Vec::as_slice))
                .unwrap();
        }
        let held = l.take_held();
        assert_eq!((l.player, l.ai.is_none()), (0, true), "{computer:?}");
        assert_eq!(l.heroes[1].at, Pos { x: 20, y: 24, z: 0 }, "{computer:?}");
        // At speed 4 each step seen drawn once, kept for the window: the view following the
        // hero, the cursor hidden; none under Blackout.
        if computer.blackout {
            assert!(held.is_empty());
        } else {
            assert_eq!(held.len(), 15);
            let first = &held[0];
            assert_eq!(
                (first.view, first.hero.map(|h| h.at), first.cursor.shown),
                (
                    ViewAt { x: -3, y: 18, z: 0 },
                    Some(Pos { x: 6, y: 26, z: 0 }),
                    false
                )
            );
            assert_eq!(
                held[14].hero.map(|h| h.at),
                Some(Pos { x: 20, y: 24, z: 0 })
            );
            assert!(
                held.iter()
                    .all(|s| s.drawn.computer && s.walk.offset == [0, 0])
            );
        }
    }
}

#[test]
fn the_single_draws_reach_the_replay_and_the_headless_play() {
    let (start, actions, ..) = computing("ai-walk");
    let start = Start {
        computer: Computer {
            speed: 4,
            blackout: false,
        },
        ..start
    };
    let (states, held) = play_held(rules(Bugs::Original), &start, &actions, 170).unwrap();
    assert_eq!(held.keys().copied().collect::<Vec<_>>(), [160]);
    assert_eq!(held[&160].len(), 15);
    assert_eq!(states[160].player, 0);
    let mut replay = Replay::new(rules(Bugs::Original), &start, actions).unwrap();
    let change = replay.at(160).unwrap();
    assert_eq!((change.t, change.held.len()), (160, 15));
    assert_eq!(change.held[3].drawn, held[&160][3].drawn);
    assert!(replay.at(159).unwrap().held.is_empty());
}

#[test]
fn a_computers_turn_it_cannot_play() {
    let end = |l: &mut Loop| {
        l.run(1, &[Action::Key(Hotkey::EndTurn)]).unwrap();
        l.run(2, &[Action::KeyUp(Hotkey::EndTurn)])
    };
    let (start, ..) = computing("ai-walk");
    // Without the mouse-over's facts (the viewer's explored tiles), or a walk's hero off the
    // map: not built, nothing changed.
    let blind = Start {
        hover: None,
        ..start.clone()
    };
    let mut l = Loop::start(rules(Bugs::Original), &blind).unwrap();
    assert_eq!(end(&mut l), Err(Unplayable::Command(12)));
    assert_eq!((l.next_turn, l.player), (0, 0));
    let mut lost = start.clone();
    lost.turns[0].walks[0].hero = 9;
    let mut l = Loop::start(rules(Bugs::Original), &lost).unwrap();
    assert_eq!(end(&mut l), Err(Unplayable::Command(12)));
    // A computer's turn with no turn after it: not built at its end; another computer's after
    // it: played on.
    let mut alone = start.clone();
    alone.turns.truncate(1);
    alone.computer.speed = 4;
    let mut l = Loop::start(rules(Bugs::Original), &alone).unwrap();
    assert_eq!(end(&mut l), Err(Unplayable::Command(12)));
    let mut twice = start.clone();
    twice.computer.speed = 4;
    twice.turns.insert(1, twice.turns[0].clone());
    let mut l = Loop::start(rules(Bugs::Original), &twice).unwrap();
    end(&mut l).unwrap();
    assert_eq!((l.next_turn, l.player), (3, 0));
    let mut l = Loop::start(
        rules(Bugs::Original),
        &Start {
            live: true,
            ..alone.clone()
        },
    )
    .unwrap();
    end(&mut l).unwrap();
    assert!(l.ai.is_none());
    // At a walk's pace the refusal comes at its end, in a later millisecond.
    let mut walking = alone;
    walking.computer.speed = 3;
    let mut l = Loop::start(rules(Bugs::Original), &walking).unwrap();
    end(&mut l).unwrap();
    let ended = (3..2000).find_map(|t| l.run(t, &[]).err().map(|e| (t, e)));
    assert_eq!(ended, Some((1502, Unplayable::Command(12))));
    // The next turn after a computer's not knowing its selection, or after it a computer's
    // turn it cannot play: not built.
    let mut unknown = start.clone();
    unknown.computer.speed = 4;
    unknown.turns[1].turn.kingdom.selected = Selected::Hero(5);
    let mut l = Loop::start(rules(Bugs::Original), &unknown).unwrap();
    assert_eq!(end(&mut l), Err(Unplayable::Command(12)));
    let mut lost = twice.clone();
    lost.turns[1].walks[0].hero = 9;
    let mut l = Loop::start(rules(Bugs::Original), &lost).unwrap();
    assert_eq!(end(&mut l), Err(Unplayable::Command(12)));
}

#[test]
fn a_computers_turn_from_a_town_selected_and_without_points() {
    // No hero of the viewer's selected at End Turn, no points recorded: the turn plays all the
    // same (the computer's hero's points the facts'); a standing hero the facts place off the
    // map's list left out.
    let (start, ..) = computing("ai-walk");
    let mut start = Start {
        index: None,
        hero: None,
        points: Vec::new(),
        ..start
    };
    let east = Hero {
        at: Pos { x: 1, y: 1, z: 0 },
        facing: Direction::East,
    };
    start.turns[1].heroes.push((9, east));
    let mut l = Loop::start(rules(Bugs::Original), &start).unwrap();
    l.run(1, &[Action::Key(Hotkey::EndTurn)]).unwrap();
    l.run(2, &[Action::KeyUp(Hotkey::EndTurn)]).unwrap();
    assert_eq!((l.player, l.state(2).points), (1, Some(1560)));
    for t in 3..=1502 {
        l.run(t, &[]).unwrap();
    }
    assert_eq!((l.player, l.heroes.len()), (0, 2));
    assert_eq!(l.heroes[1].at, Pos { x: 20, y: 24, z: 0 });
}

#[test]
fn kind_5s_step() {
    let art = OtherArt {
        step: 50,
        sand: 10,
        glass: 10,
    };
    let built = TurnStatus {
        crest: 1,
        sand: 0,
        glass: 0,
        value: 2560,
    };
    // One frame up while below (value − left) × 9 / value, never down; the sand round its 10.
    let s = art.stepped(built, 1460);
    assert_eq!((s.sand, s.glass), (1, 1));
    let s = art.stepped(TurnStatus { glass: 5, ..built }, 960);
    assert_eq!(s.glass, 5);
    let s = art.stepped(
        TurnStatus {
            glass: 9,
            sand: 9,
            ..built
        },
        3000,
    );
    assert_eq!((s.sand, s.glass), (0, 9));
    // Points left past the value count as the value; a value of 0 (a human's) holds the glass.
    assert_eq!(art.stepped(built, 9999).glass, 0);
    assert_eq!(art.stepped(TurnStatus { value: 0, ..built }, 0).glass, 0);
}

#[test]
fn a_turns_heroes_and_walks_from_its_facts() {
    let (_, _, _, live) = COMPUTING[0];
    let heroes = heroes_of(live);
    let table: toml::Table = live.parse().unwrap();
    let turns: Vec<Turn> = table["turn"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| Turn::read(t).unwrap())
        .collect();
    // The standing heroes by owner and class; the selected one lifted is not among them.
    let red = Hero {
        at: Pos { x: 4, y: 4, z: 0 },
        facing: Direction::East,
    };
    assert_eq!(turn::placed(&heroes, &[], &turns[0]), Ok(vec![(0, red)]));
    // None for a table without them (T1a's); refused: a boat moved, a hero's boat changed, a hero
    // matching none or several.
    let without = Turn {
        heroes: None,
        boats: None,
        ..turns[0].clone()
    };
    assert_eq!(turn::placed(&heroes, &[], &without), Ok(Vec::new()));
    let boat = LiveBoat {
        at: Pos { x: 1, y: 1, z: 0 },
        kind: 0,
        facing: Facing::Up,
    };
    assert!(turn::placed(&heroes, &[boat], &turns[0]).is_err());
    let mut boarded = turns[0].clone();
    if let Some(h) = boarded.heroes.as_mut() {
        h[0].boat = Some(crate::live::Boat { kind: 0, colour: 0 });
    }
    assert!(turn::placed(&heroes, &[], &boarded).is_err());
    let twins = [heroes[0], heroes[0]];
    assert!(turn::placed(&twins, &[], &turns[0]).is_err());
    // A computer's walk: its hero, its slot, its steps from its turn's hero lines to the next
    // turn's; none for a human's turn.
    let east = |x| Hero {
        at: Pos { x, y: 26, z: 0 },
        facing: Direction::East,
    };
    let lines = [(100, east(5)), (160, east(6)), (260, east(7)), (1660, red)];
    assert_eq!(
        turn::walks(&turns, 0, &heroes, &lines),
        Ok(vec![Steps {
            hero: 1,
            slot: 0,
            steps: vec![east(6), east(7)]
        }])
    );
    assert_eq!(turn::walks(&turns, 1, &heroes, &lines), Ok(Vec::new()));
    assert_eq!(turn::walks(&turns, 2, &heroes, &lines), Ok(Vec::new()));
    // The last turn's walk runs to the end; refused: none or several heroes of its colour, none
    // selected.
    assert_eq!(
        turn::walks(&turns[..1], 0, &heroes, &lines).unwrap()[0]
            .steps
            .len(),
        3
    );
    assert!(turn::walks(&turns, 0, &heroes[..1], &lines).is_err());
    let mut unselected = turns.clone();
    unselected[0].kingdom.selected = Selected::None;
    assert!(turn::walks(&unselected, 0, &heroes, &lines).is_err());
}

#[test]
fn a_turns_heroes_and_boats_read() {
    let (_, _, _, live) = COMPUTING[0];
    let table: toml::Table = live.parse().unwrap();
    let turn = table["turn"].as_array().unwrap()[1].clone();
    let read = Turn::read(&turn).unwrap();
    assert_eq!(
        read.heroes.as_ref().map(|h| h[0].at),
        Some(Pos { x: 20, y: 24, z: 0 })
    );
    assert_eq!(read.boats, Some(Vec::new()));
    let value = |text: &str| -> toml::Value {
        let t: toml::Table = toml::from_str(&format!("v = {text}")).unwrap();
        t["v"].clone()
    };
    for (key, text) in [
        ("heroes", "[[1, 2, 0, 18, 0, 0, -1, -1]]"),
        ("heroes", "[[1, 2, 0, 0, 8, 0, -1, -1]]"),
        ("heroes", "[[1, 2, 0, 0, 0, 8, -1, -1]]"),
        ("heroes", "[[1, 2, 0, 0, 0, 0, 3, 0]]"),
        ("heroes", "[[1, 2, 0, 0, 0, 0, 0, 8]]"),
        ("heroes", "[[-1, 2, 0, 0, 0, 0, -1, -1]]"),
        ("heroes", "[[1, 2, 0]]"),
        ("heroes", "[[1, -2, 0, 0, 0, 0, -1, -1]]"),
        ("heroes", "[[1, 2, -1, 0, 0, 0, -1, -1]]"),
        ("heroes", "[[1, 2, 0, 0, -1, 0, -1, -1]]"),
        ("heroes", "[1]"),
        ("heroes", "1"),
        ("boats", "[[1, 2, 0, 3, 0]]"),
        ("boats", "[[1, 2, 0, 0, 8]]"),
        ("boats", "[[1, -2, 0, 0, 0]]"),
        ("boats", "[[-1, 2, 0, 0, 0]]"),
        ("boats", "[[1, 2, -1, 0, 0]]"),
        ("boats", "[[1, 2, 0, 0, -1]]"),
        ("boats", "[1]"),
        ("boats", "[[1, 2, \"x\", 0, 0]]"),
        ("boats", "1"),
    ] {
        let mut bad = turn.as_table().unwrap().clone();
        bad.insert(key.into(), value(text));
        assert_eq!(Turn::read(&toml::Value::Table(bad)), None, "{key} = {text}");
    }
    let mut boat = turn.as_table().unwrap().clone();
    boat.insert("heroes".into(), value("[[1, 2, 0, 0, 0, 0, 1, 3]]"));
    boat.insert("boats".into(), value("[[3, 4, 0, 2, 5]]"));
    let read = Turn::read(&toml::Value::Table(boat)).unwrap();
    assert_eq!(
        read.heroes.unwrap()[0].boat,
        Some(crate::live::Boat { kind: 1, colour: 3 })
    );
    assert_eq!(read.boats.unwrap()[0].facing, Facing::DownLeft);
}
