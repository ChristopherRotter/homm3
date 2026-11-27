//! A hot seat's hand-over (E24 M5 T5, design §5.31): the recorded `seat-*` sequences replayed
//! (their timelines and their samples as recorded), then the cover's ways one by one.

use super::turns::laid;
use super::*;

use h3_ui::message::Style;

use crate::live::Turn;

/// The recorded hand-overs: their situations, timelines, kept live parts and samples.
const SEATS: [(&str, &str, &str, &str, &str); 2] = [
    (
        "seat-hand-over",
        include_str!("../../../../../map/situations/seq-seat.toml"),
        include_str!("../../../../../map/timelines/seat-hand-over.txt"),
        include_str!("../../../../../map/timelines/seat-hand-over-live.toml"),
        include_str!("../../../../../map/timelines/seat-hand-over-sounds.txt"),
    ),
    (
        "seat-new-day",
        include_str!("../../../../../map/situations/seq-seat.toml"),
        include_str!("../../../../../map/timelines/seat-new-day.txt"),
        include_str!("../../../../../map/timelines/seat-new-day-live.toml"),
        include_str!("../../../../../map/timelines/seat-new-day-sounds.txt"),
    ),
];

/// The samples' lengths on the harness's clock (the recordings').
const LENGTHS: [(&str, u32); 5] = [
    ("NewWeek.wav", 5759),
    ("newweek.wav", 5759),
    ("newday.wav", 3739),
    ("newmonth.wav", 4784),
    ("button.wav", 151),
];

/// The cover's sound's length.
const COVER: u32 = 5759;

/// A recorded hand-over's start as the binary's `for_sequence` and `Painter::hot_seat` make it:
/// the live part's kingdom, heroes, selection and turns; each human's turn covered (its seat's
/// selection's window, a computer's colour between it and the human before), its first hero
/// selected and its heroes by the colour; a box of each human's; the cover's sound's length, the
/// clock, and the game's start covered with `start`.
fn seated(
    id: &str,
) -> (
    Start,
    BTreeMap<u32, Vec<Action>>,
    u32,
    &'static str,
    &'static str,
) {
    let &(_, situation, timeline, live, sounds) = SEATS.iter().find(|s| s.0 == id).unwrap();
    let sequences = Sequences::parse(SEQUENCES).unwrap();
    let sequence = sequences.get(id).unwrap();
    let mut start = start(situation, sequence.scroll_speed().unwrap());
    (start.kingdom, start.buttons) = live_kingdom(live);
    let (heroes, viewer) = live_heroes(live);
    let table: toml::Table = live.parse().unwrap();
    let objects = table["anchors"].as_array().unwrap().len();
    let facts = hover::Facts::read(&table["hover"], objects, heroes.len(), 36, 1).unwrap();
    let index_of = |slot: usize| {
        facts
            .heroes
            .iter()
            .position(|h| h.0 == facts.hero_list[slot].0)
            .unwrap()
    };
    start.heroes = heroes
        .iter()
        .map(|h| Hero {
            at: h.at,
            facing: Direction::ALL[usize::from(h.facing.number())],
        })
        .collect();
    if let Selected::Hero(slot) = start.kingdom.selected {
        start.index = Some(index_of(slot));
        start.hero = Some(start.heroes[index_of(slot)]);
    }
    if let Some(Status::Hero(status)) = start.kingdom.status.clone() {
        start.chosen.heroes = vec![Some(ChosenHero {
            hero: index_of(0),
            status,
            path: Vec::new(),
        })];
    }
    start.player = viewer;
    start.question = Some(laid("End?", Style::OkCancel));
    start.sound = Some(crate::sound::tests::sound(crate::sound::tests::tiles(
        36,
        |_, _| 2,
        |_, _| None,
    )));
    let names = sequence.names().unwrap().unwrap();
    let mut colours: Vec<u8> = heroes.iter().map(|h| h.owner).collect();
    colours.sort_unstable();
    colours.dedup();
    let humans: Vec<u8> = colours.iter().copied().take(names.len()).collect();
    let seats: &[toml::Value] = table
        .get("seat")
        .and_then(toml::Value::as_array)
        .map_or(&[], Vec::as_slice);
    let mut before = viewer;
    start.turns = table
        .get("turn")
        .and_then(toml::Value::as_array)
        .map_or(&[][..], Vec::as_slice)
        .iter()
        .map(|t| {
            let mut turn = Turn::read(t).unwrap();
            let player = turn.player;
            let seat = seats
                .iter()
                .position(|s| s["at"].as_integer() == Some(i64::from(turn.at)));
            let selection = seat
                .and_then(|k| seats[k].get("selection"))
                .map(|s| Status::read(s).unwrap());
            let computers = (1..8)
                .map(|k| (before + k) % 8)
                .take_while(|&c| c != player)
                .any(|c| colours.contains(&c) && !humans.contains(&c));
            before = player;
            if turn.kingdom.selected == Selected::None {
                turn.kingdom.selected = Selected::Hero(0);
            }
            let list = (player != viewer).then(|| {
                heroes
                    .iter()
                    .enumerate()
                    .filter(|(_, h)| h.owner == player)
                    .map(|(i, _)| Some(i))
                    .collect()
            });
            let week = turn::starts_week(turn.kingdom.date);
            NextTurn {
                message: week.then(|| laid("Week of the Rat.", Style::Ok)),
                turn,
                heroes: Vec::new(),
                walks: Vec::new(),
                taken: Vec::new(),
                flags: Vec::new(),
                lost: Vec::new(),
                boats: Vec::new(),
                list,
                first: false,
                open: false,
                cover: Some(Cover {
                    seat,
                    computers,
                    selection,
                    events: Vec::new(),
                }),
            }
        })
        .collect();
    start.seat = Some(HandOver {
        boxes: humans
            .iter()
            .map(|&c| (c, laid("A human's turn.", Style::Ok)))
            .collect(),
        sample: Some(COVER),
        timed: sequence.timed(),
    });
    (
        start,
        sequence.actions().unwrap(),
        sequence.ms,
        timeline,
        sounds,
    )
}

/// The one-shot samples of `cues` as `<ms> <length> <loops>`, each millisecond's in order.
fn samples(cues: &[(u32, Cue)]) -> Vec<String> {
    let mut out: Vec<(u32, u32)> = cues
        .iter()
        .filter_map(|(t, cue)| match cue {
            Cue::Effect(name) | Cue::Cover(name) => {
                let length = LENGTHS.iter().find(|(n, _)| n == name).unwrap().1;
                Some((*t, length))
            }
            _ => None,
        })
        .collect();
    out.sort_unstable();
    out.iter()
        .map(|(t, l)| alloc::format!("{t} {l} 1"))
        .collect()
}

/// A recording's samples without their channels, each millisecond's in order (a game's start's
/// cover at −1 the viewer's 0, its Begin's click before it left out).
fn recorded_samples(text: &str) -> Vec<String> {
    let mut out: Vec<(u32, u32)> = text
        .lines()
        .filter_map(|line| {
            let n: Vec<i64> = line.split(' ').map(|f| f.parse().unwrap()).collect();
            let t = match n[0] {
                -1 => 0,
                t => u32::try_from(t).ok()?,
            };
            Some((t, u32::try_from(n[2]).unwrap()))
        })
        .collect();
    out.sort_unstable();
    out.iter()
        .map(|(t, l)| alloc::format!("{t} {l} 1"))
        .collect()
}

#[test]
fn replays_the_recorded_hand_overs() {
    for (id, ..) in SEATS {
        let (start, actions, ms, timeline, sounds) = seated(id);
        let (states, cues) = play_heard(rules(Bugs::Original), &start, &actions, ms).unwrap();
        let got = lines(&states, &BTreeMap::new());
        let want = without_screens(timeline);
        for (n, (g, w)) in got.iter().zip(&want).enumerate() {
            assert_eq!(g, w, "{id}: line {n}");
        }
        assert_eq!(got.len(), want.len(), "{id}");
        assert_eq!(samples(&cues), recorded_samples(sounds), "{id}");
    }
}
