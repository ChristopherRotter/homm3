//! The record player (E24 M4 T4c, design §5.17): its two actions, the loop's waits and
//! `crate::director`'s input from a record's commands.

use super::*;

use h3_protocol::wire::game::{Answer, ChestChoice, Consent, SkillOffer};
use h3_ui::message::Style;

use crate::director::{self, Directed, Pace};
use crate::engine::Command;

pub(super) fn at(x: u8, y: u8) -> Pos {
    Pos { x, y, z: 0 }
}

/// Four tiles east of the hero in their order, the last the green cross.
pub(super) fn east_in_order() -> Vec<Arrow> {
    vec![
        arrow(18, 17, 12),
        arrow(19, 17, 12),
        arrow(20, 17, 12),
        arrow(21, 17, 1),
    ]
}

/// The walk east as a record gives it.
pub(super) fn east_walk() -> Move {
    Move {
        tiles: vec![at(18, 17), at(19, 17), at(20, 17), at(21, 17)],
        points: vec![None, None, None, Some(1160)],
        revealed: vec![Vec::new(); 4],
        sea: None,
        battle: None,
        boats: Vec::new(),
    }
}

/// `walking_start`'s start with its hero in the kingdom's list and the walk east as a record's.
pub(super) fn record_start() -> Start {
    let mut start = walking_start(4, vec![east_in_order()]);
    start.kingdom.heroes = vec![KingdomHero {
        points: 1560,
        ..KingdomHero::default()
    }];
    start.kingdom.selected = Selected::Hero(0);
    start.moves = Some(vec![east_walk()]);
    start
}

#[test]
fn a_walk_given_by_its_last_tile() {
    let mut l = Loop::start(rules(Bugs::Fixed), &record_start()).unwrap();
    assert!(l.idle() && !l.boxed());
    assert_eq!(l.walks_left(), 1);
    // The selected hero chosen again: nothing.
    l.run(1, &[Action::Choose(0)]).unwrap();
    assert_eq!(l.kingdom.selected, Selected::Hero(0));
    // A tile no path leads to: refused, live nothing.
    assert_eq!(
        l.run(2, &[Action::Walk(at(5, 5))]),
        Err(Unplayable::NoPath([5, 5, 0]))
    );
    l.run(3, &[Action::Walk(at(21, 17))]).unwrap();
    // At speed 4 the walk ends in its millisecond.
    assert_eq!(l.hero.map(|h| h.at), Some(at(21, 17)));
    assert_eq!(l.walks_left(), 0);
    assert!(l.idle());
    let mut live = record_start();
    live.live = true;
    let mut l = Loop::start(rules(Bugs::Fixed), &live).unwrap();
    l.run(1, &[Action::Walk(at(5, 5))]).unwrap();
    // A walk's path with a gap: no path.
    let mut gap = record_start();
    gap.paths = vec![vec![arrow(19, 17, 1)]];
    let mut l = Loop::start(rules(Bugs::Fixed), &gap).unwrap();
    assert!(l.run(1, &[Action::Walk(at(19, 17))]).is_err());
    assert!(Action::Choose(0).point().is_none() && !Action::Walk(at(1, 1)).ends_walk());
    let walks = Loop::start(rules(Bugs::Fixed), &walking_start(1, Vec::new())).unwrap();
    assert_eq!(walks.walks_left(), 0);
}

#[test]
fn a_walk_that_began_a_battle_hands_it_over_at_its_end() {
    // E24 M4 T5b1 (design §5.20): the walk's end, in its millisecond at speed 4, names the
    // record's `GameEvents` that began the battle; the replay tells it from there.
    let mut start = record_start();
    if let Some(moves) = &mut start.moves {
        moves[0].battle = Some(7);
    }
    let mut l = Loop::start(rules(Bugs::Fixed), &start).unwrap();
    assert!(l.battles().is_empty());
    l.run(3, &[Action::Walk(at(21, 17))]).unwrap();
    assert_eq!(l.battles(), [(3, 7)]);
    // A walk without one hands none over.
    let mut l = Loop::start(rules(Bugs::Fixed), &record_start()).unwrap();
    l.run(3, &[Action::Walk(at(21, 17))]).unwrap();
    assert!(l.battles().is_empty());
    // The replay of the same input tells it once it ran that far.
    let actions = BTreeMap::from([(3, vec![Action::Walk(at(21, 17))])]);
    let mut replay = crate::replay::Replay::new(rules(Bugs::Fixed), &start, actions).unwrap();
    assert!(replay.battles().is_empty());
    replay.at(10).unwrap();
    assert_eq!(replay.battles(), [(3, 7)]);
}

#[test]
fn a_hero_chosen_by_its_place() {
    let (start, _, _, _) = choosing("choose-lists");
    let r = rules(Bugs::Original);
    let mut l = Loop::start(r.clone(), &start).unwrap();
    let Selected::Hero(i) = l.kingdom.selected else {
        panic!("a hero selected");
    };
    let other = (i + 1) % 2;
    l.run(1, &[Action::Choose(other)]).unwrap();
    assert_eq!(l.kingdom.selected, Selected::Hero(other));
    let mut unknown = start;
    unknown.chosen.heroes[2] = None;
    let mut l = Loop::start(r, &unknown).unwrap();
    assert_eq!(
        l.run(1, &[Action::Choose(2)]),
        Err(Unplayable::Record([-1, -1, -1]))
    );
}

#[test]
fn a_records_commands_as_input() {
    let pace = Pace::default();
    assert_eq!((pace.wait, pace.hold), (400, 60));
    let r = rules(Bugs::Fixed);
    let start = record_start();
    let walk = Command::Walk {
        hero: 4,
        to: at(21, 17),
    };
    let directed = director::direct(
        r.clone(),
        &start,
        &[
            Command::Other,
            walk,
            Command::Answer(Answer::Chest {
                choice: ChestChoice::Gold,
            }),
        ],
        &[(4, 0)],
        pace,
        5000,
    )
    .unwrap();
    // The screen's command nothing; the walk's hero chosen after the wait, its walk after another;
    // an answer with no box open nothing; the end after the walk's wait.
    let Directed { actions, end, .. } = directed;
    let given: Vec<(u32, Action)> = actions
        .iter()
        .flat_map(|(&t, a)| a.iter().map(move |&a| (t, a)))
        .collect();
    assert_eq!(
        given,
        [(803, Action::Choose(0)), (1203, Action::Walk(at(21, 17)))]
    );
    assert_eq!(end, 2005);
    // A walk's hero not listed; a walk not taken; the limit.
    assert_eq!(
        director::direct(r.clone(), &start, &[walk], &[], pace, 5000),
        Err((401, Unplayable::Record([21, 17, 0])))
    );
    assert_eq!(
        director::direct(r.clone(), &start, &[], &[], pace, 5000),
        Err((401, Unplayable::Untaken(1)))
    );
    assert_eq!(
        director::direct(r.clone(), &start, &[walk], &[(4, 0)], pace, 900),
        Err((900, Unplayable::Record([-1, -1, -1])))
    );
    // A loop it cannot start, a millisecond it cannot play.
    let mut broken = start.clone();
    broken.pointer = [100, 100];
    assert_eq!(
        director::direct(r.clone(), &broken, &[walk], &[(4, 0)], pace, 5000),
        Err((0, Unplayable::OverView([100, 100])))
    );
    let nowhere = Command::Walk {
        hero: 4,
        to: at(5, 5),
    };
    assert_eq!(
        director::direct(r.clone(), &start, &[nowhere], &[(4, 0)], pace, 5000),
        Err((802, Unplayable::NoPath([5, 5, 0])))
    );
    assert_eq!(
        Unplayable::Record([-1, -1, -1]).to_string(),
        "the record not at its end by the limit"
    );
    assert_eq!(
        Unplayable::Record([1, 2, 0]).to_string(),
        "a record's walk to (1, 2, 0) of a hero not listed"
    );
    assert_eq!(
        Unplayable::Untaken(2).to_string(),
        "the record at its end with 2 walks not taken"
    );
}

#[test]
fn a_records_answers_in_its_boxes() {
    // End Turn's question open (Move Reminder, a hero able to move): each answer's key.
    let r = rules(Bugs::Fixed);
    let mut start = record_start();
    start.moves = None;
    start.live = true;
    start.move_reminder = true;
    start.kingdom.heroes[0].movable = true;
    start.question = Some(turns::laid("TURN?", Style::OkCancel));
    for (answer, keys) in [
        (
            Some(Answer::Offer {
                consent: Consent::Decline,
            }),
            [Action::BoxKey(Key::Cancel), Action::BoxKeyUp(Key::Cancel)],
        ),
        (
            Some(Answer::Offer {
                consent: Consent::Accept,
            }),
            [Action::BoxKey(Key::Ok), Action::BoxKeyUp(Key::Ok)],
        ),
        (
            Some(Answer::LevelUp {
                offer: SkillOffer::Second,
            }),
            [Action::Offer(1), Action::Up],
        ),
        (None, [Action::BoxKey(Key::Ok), Action::BoxKeyUp(Key::Ok)]),
    ] {
        let mut commands = vec![Command::EndTurn];
        commands.extend(answer.map(Command::Answer));
        let got = director::direct(r.clone(), &start, &commands, &[], Pace::default(), 3000);
        let given: Vec<Action> = match &got {
            Ok(d) => d.actions.values().flatten().copied().collect::<Vec<_>>(),
            Err((_, why)) => panic!("{answer:?}: {why}"),
        }
        .into_iter()
        .skip(2)
        .take(2)
        .collect();
        assert_eq!(given, keys, "{answer:?}");
    }
}

#[test]
fn a_battle_an_answer_began_is_handed_over_where_its_box_closes() {
    // E24 M4 T5b1 (design §5.20): an answer given in its box, at its key's release; an answer
    // with no box open, where it is passed over.
    let r = rules(Bugs::Fixed);
    let mut start = record_start();
    start.moves = None;
    start.live = true;
    start.move_reminder = true;
    start.kingdom.heroes[0].movable = true;
    start.question = Some(turns::laid("TURN?", Style::OkCancel));
    let accept = Command::Answer(Answer::Offer {
        consent: Consent::Accept,
    });
    let commands = [Command::EndTurn, accept, Command::Battle(9)];
    let d = director::direct(r.clone(), &start, &commands, &[], Pace::default(), 3000).unwrap();
    let release = d
        .actions
        .iter()
        .find(|(_, a)| a.contains(&Action::BoxKeyUp(Key::Ok)))
        .map(|(&t, _)| t)
        .unwrap();
    assert_eq!(d.battles, [(release, 9)]);
    // No box open: the answer passed over and its battle at once, no wait of its own.
    let walk = Command::Walk {
        hero: 4,
        to: at(21, 17),
    };
    let chest = Command::Answer(Answer::Chest {
        choice: ChestChoice::Gold,
    });
    let pace = Pace::default();
    let d = director::direct(
        r.clone(),
        &record_start(),
        &[walk, chest, Command::Battle(5)],
        &[(4, 0)],
        pace,
        5000,
    )
    .unwrap();
    // As `a_records_commands_as_input`'s without its screen's command: a wait less.
    assert_eq!(d.end, 1604);
    assert_eq!(d.battles.len(), 1);
    assert_eq!(d.battles[0].1, 5);
    assert!((803..1604).contains(&d.battles[0].0));
    // A battle no answer began: nothing, at a command's wait.
    let d = director::direct(
        r,
        &record_start(),
        &[walk, Command::Battle(5)],
        &[(4, 0)],
        pace,
        5000,
    )
    .unwrap();
    assert!(d.battles.is_empty());
}
