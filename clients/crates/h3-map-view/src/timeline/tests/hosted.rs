//! The hot seat's host in play (E24 M5 T4a, design §5.26): a live view asks it the selected
//! hero's reach where it stands (a click's path, the cursor over the view) and each walk past the
//! engine's known ones (a path's, a key's step); until it answers the action changes nothing, and
//! the loop run again from before it plays it with the answer. Since E24 M5 T4b (design §5.27) a
//! hosted question's answer and a hosted level-up's offer the same way.

use super::*;

use h3_ui::message::Style;

use super::turns::laid;
use crate::hover::frame;
use crate::levels::{self, LevelUp, Offer};
use crate::live::KingdomHero;
use crate::reach::{
    AnswerAsked, AnswerKey, Hosted, LevelAsked, Pick, Reach, RevisitAsked, Stand, WalkAsked,
};
use crate::replay::Replay;

fn at(x: u8, y: u8) -> Pos {
    Pos { x, y, z: 0 }
}

/// `walking`'s loop at hero speed 3, live with a host and no walk known, its hero selected.
fn hosted() -> Loop {
    let mut start = walking_start(3, Vec::new());
    start.live = true;
    start.heroes = start.hero.into_iter().collect();
    start.index = Some(0);
    start.moves = Some(Vec::new());
    start.host = Some(Hosted::default());
    start.kingdom.heroes = vec![KingdomHero {
        points: 1560,
        ..KingdomHero::default()
    }];
    start.kingdom.selected = Selected::Hero(0);
    Loop::start(rules(Bugs::Fixed), &start).unwrap()
}

/// The stand of the selected hero after `walks` walks.
fn stand(walks: usize) -> Stand {
    Stand {
        hero: 0,
        walks,
        turns: 0,
        answers: 0,
    }
}

/// A reach of [`east`]'s path, each tile at 100 points a step.
fn east_reach() -> Arc<Reach> {
    Arc::new(Reach {
        paths: vec![east()],
        costs: (18..=21)
            .map(|x| ((x, 17), 100 * (x as u32 - 17)))
            .collect(),
    })
}

/// The engine's walk of [`east`]: its points at its last step.
fn east_walk() -> Move {
    Move {
        tiles: vec![at(18, 17), at(19, 17), at(20, 17), at(21, 17)],
        points: vec![None, None, None, Some(1160)],
        revealed: vec![Vec::new(); 4],
        sea: None,
        battle: None,
        boats: Vec::new(),
        stops: None,
    }
}

#[test]
fn a_click_asks_the_reach_where_the_hero_stands() {
    let click = Action::Click(tile(21, 17));
    let mut l = hosted();
    let before = l.clone();
    l.run(1, &[click]).unwrap();
    // Asked: nothing shown yet.
    assert_eq!(l.reach_asked(), Some((1, stand(0))));
    assert!(l.state(1).arrows.is_empty());
    // Given, the click played again: the engine's path.
    let mut l = before;
    l.add_reach(stand(0), &east_reach());
    assert_eq!(l.reach_asked(), None);
    l.run(1, &[click]).unwrap();
    assert_eq!(l.state(1).arrows.len(), 4);
    assert_eq!(l.reach_asked(), None);
    // A tile it does not reach: the pointer's move only.
    l.run(2, &[Action::Click(tile(10, 10))]).unwrap();
    assert_eq!(l.state(2).arrows.len(), 4);
}

#[test]
fn a_path_s_walk_asks_the_host_then_plays_its_facts() {
    let click = Action::Click(tile(21, 17));
    let mut l = hosted();
    l.add_reach(stand(0), &east_reach());
    l.run(1, &[click]).unwrap();
    let before = l.clone();
    // Its destination clicked again: the walk asked, the hero still standing, its path shown.
    l.run(2, &[click]).unwrap();
    let asked = WalkAsked {
        at: 2,
        stand: stand(0),
        tiles: vec![at(18, 17), at(19, 17), at(20, 17), at(21, 17)],
    };
    assert_eq!(l.walk_asked(), Some(&asked));
    let s = l.state(2);
    assert_eq!((s.hero.unwrap().at, s.arrows.len()), (at(17, 17), 4));
    // The host's facts given: the walk played as a record's.
    let mut l = before;
    l.add_walks(&[east_walk()], &[]);
    l.run(2, &[click]).unwrap();
    assert_eq!(l.walk_asked(), None);
    for t in 3..=600 {
        l.run(t, &[]).unwrap();
    }
    let s = l.state(600);
    assert_eq!((s.hero.unwrap().at, s.points), (at(21, 17), Some(1160)));
    // Standing again: the next click asks the reach of its new stand.
    l.run(601, &[Action::Click(tile(22, 17))]).unwrap();
    assert_eq!(l.reach_asked(), Some((601, stand(1))));
}

#[test]
fn a_key_s_step_the_engine_refuses_is_nothing() {
    let step = Action::Step(Direction::North);
    let mut l = hosted();
    let before = l.clone();
    l.run(1, &[step]).unwrap();
    let tiles = vec![at(17, 16)];
    let asked = l.walk_asked().unwrap().clone();
    assert_eq!(asked.tiles, tiles);
    // Refused: played again, nothing is asked and nothing changes; again later from the same
    // stand, refused at once.
    let mut l = before.clone();
    l.refuse_walk(&asked);
    assert_eq!(l.walk_asked(), None);
    l.run(1, &[step]).unwrap();
    let mut quiet = before.clone();
    quiet.run(1, &[]).unwrap();
    assert_eq!(l.walk_asked(), None);
    assert_eq!(l.state(1), quiet.state(1));
    l.run(2, &[step]).unwrap();
    assert_eq!(l.walk_asked(), None);
    // Taken: a step as a record's.
    let mut l = before;
    let north = Move {
        tiles: tiles.clone(),
        points: vec![Some(1460)],
        revealed: vec![Vec::new()],
        ..east_walk()
    };
    l.add_walks(&[north], &[]);
    l.run(1, &[step]).unwrap();
    for t in 2..=300 {
        l.run(t, &[]).unwrap();
    }
    assert_eq!(l.state(300).hero.unwrap().at, at(17, 16));
}

#[test]
fn the_cursor_over_the_view_by_the_host_s_reach() {
    let mut l = hovering(true, Bugs::Fixed);
    l.index = Some(0);
    l.moves = Some(Arc::new(Vec::new()));
    l.host = Some(Arc::new(Hosted::default()));
    let before = l.clone();
    // The monster's tile: the reach asked, the cursor not set yet.
    l.run(1, &[Action::Pointer(on(5, 4))]).unwrap();
    assert_eq!(l.reach_asked(), Some((1, stand(0))));
    let cursor = |reach: Reach| {
        let mut l = before.clone();
        l.add_reach(stand(0), &Arc::new(reach));
        l.run(1, &[Action::Pointer(on(5, 4))]).unwrap();
        l.state(1).cursor.frame
    };
    // Reached today: the attack's first day, the start's facts' second not read.
    let today = Reach {
        paths: Vec::new(),
        costs: [((5, 4), 50)].into_iter().collect(),
    };
    assert_eq!(cursor(today), frame::ATTACK);
    // Not reached today: the arrow (MCL-Q37: no later day's cost).
    assert_eq!(cursor(Reach::default()), rules(Bugs::Fixed).arrow);
}

#[test]
fn a_loop_without_a_host_keeps_no_answer() {
    // A recording's (no engine walks, no host): a walk's facts keep only the finds, a refusal
    // and a reach nothing.
    let mut l = walking(3, vec![east()]);
    let asked = WalkAsked {
        at: 1,
        stand: stand(0),
        tiles: vec![at(18, 17)],
    };
    l.add_walks(&[east_walk()], &[]);
    l.refuse_walk(&asked);
    l.add_reach(stand(0), &east_reach());
    assert_eq!((l.moves.as_ref(), l.host.as_ref()), (None, None));
    assert_eq!((l.walk_asked(), l.reach_asked()), (None, None));
    // E24 M5 T4b: an answer and a level-up's next given to it keep nothing either (no find 7).
    let key = AnswerKey {
        find: 7,
        nth: 0,
        pick: Pick::No,
    };
    l.add_answer(key, &Answer::default(), &[]);
    let level = LevelAsked {
        at: 1,
        hero: 0,
        level: 2,
        offer: 0,
    };
    l.add_level(&level, &[]);
    assert_eq!(
        (l.answer_asked(), l.level_asked(), l.host.as_ref()),
        (None, None, None)
    );
}

/// A level-up to level 2 with one offer (chosen at once), its dialog laid out with the test
/// layout, the host's to answer.
fn hosted_level() -> LevelBox {
    let layout =
        AdventureLayout::parse(include_str!("../../../../../map/layouts/adventure.toml")).unwrap();
    let up = LevelUp {
        hero: 16,
        level: 2,
        primary: 1,
        offers: vec![Offer { skill: 9, level: 1 }],
    };
    LevelBox {
        hosted: true,
        level: 2,
        dialog: Arc::new(levels::dialog(
            &layout.level_up,
            &up,
            BTreeMap::new(),
            "hpl016rn.pcx",
            0,
        )),
        offers: up.offers.clone(),
        primary: 1,
        chosen: None,
    }
}

#[test]
fn a_replay_asks_the_host_its_answers_and_plays_them_again_from_before() {
    // E24 M5 T4b (design §5.27): the engine's walk to (21, 17) known, its end's visit asking a
    // question whose answers are the host's; the answer's experience opens a level-up the host
    // answers too.
    let mut start = walking_start(3, Vec::new());
    start.live = true;
    start.heroes = start.hero.into_iter().collect();
    start.index = Some(0);
    start.moves = Some(vec![east_walk()]);
    start.host = Some(Hosted {
        reaches: [(0, 0), (1, 0), (1, 1), (1, 2)]
            .map(|(walks, answers)| {
                let reach = if walks == 0 {
                    east_reach()
                } else {
                    Arc::default()
                };
                (
                    Stand {
                        answers,
                        ..stand(walks)
                    },
                    reach,
                )
            })
            .to_vec(),
        ..Hosted::default()
    });
    start.kingdom.heroes = vec![KingdomHero {
        points: 1560,
        ..KingdomHero::default()
    }];
    start.kingdom.selected = Selected::Hero(0);
    start.finds = vec![Found {
        at: at(21, 17),
        first: Vec::new(),
        again: None,
        gain: Gain::default(),
        found: None,
        taken: None,
        event: false,
        blocks: None,
        ask: Some(Question {
            hosted: Some(0),
            asked: laid("OPEN?", Style::OkCancel),
            chooses: false,
            yes: None,
            no: None,
            attacker: None,
            second: None,
        }),
        flag: None,
        fight: None,
        walk: Some(0),
        levels: Vec::new(),
    }];
    let mut r = Replay::new(rules(Bugs::Fixed), &start, BTreeMap::new()).unwrap();
    let click = Action::Click(tile(21, 17));
    r.give(1, click);
    r.give(2, click);
    assert!(r.at(900).unwrap().drawn.message.is_some());
    // Its OK at 1000: asked, the box closed and nothing shown of the answer.
    let enter = [Action::BoxKey(Key::Ok), Action::BoxKeyUp(Key::Ok)];
    for action in enter {
        r.give(1000, action);
    }
    assert!(r.at(1000).unwrap().drawn.message.is_none());
    let key = AnswerKey {
        find: 0,
        nth: 0,
        pick: Pick::Yes,
    };
    let asked = r.answer_asked().unwrap();
    assert_eq!(
        asked,
        AnswerAsked {
            at: 1000,
            asked: key
        }
    );
    // Given with the level its experience opens: the answer's box at once.
    let shows = Gain {
        experience: 1000,
        ..Gain::default()
    };
    let answer = Answer {
        boxes: vec![(laid("FOUND.", Style::Ok), Gain::default())],
        gain: shows,
        ..Answer::default()
    };
    r.add_answer(asked, &answer, &[hosted_level()]);
    assert_eq!(r.answer_asked(), None);
    assert!(r.at(1000).unwrap().drawn.message.is_some());
    // Its close at 1100: the level-up's dialog (one offer, chosen at once); its OK at 1200 asked.
    for action in enter {
        r.give(1100, action);
    }
    assert!(r.at(1100).unwrap().drawn.message.is_some());
    for action in enter {
        r.give(1200, action);
    }
    r.at(1200).unwrap();
    let level = r.level_asked().unwrap();
    let want = LevelAsked {
        at: 1200,
        hero: 0,
        level: 2,
        offer: 0,
    };
    assert_eq!(level, want);
    // Given no level after it: closed, the visit over.
    r.add_level(&level, &[]);
    assert_eq!(r.level_asked(), None);
    assert!(r.at(1300).unwrap().drawn.message.is_none());
    assert_eq!(r.answer_asked(), None);
}

#[test]
fn a_visit_s_level_ups_and_no_box_open_at_its_walk_s_end() {
    // E24 M5 T4b (design §5.27): the Tree of Knowledge's free level, live: no box, its level-up's
    // dialog at the walk's end, the visit over at its close.
    let mut start = walking_start(3, Vec::new());
    start.live = true;
    start.heroes = start.hero.into_iter().collect();
    start.index = Some(0);
    start.moves = Some(vec![east_walk()]);
    start.host = Some(Hosted {
        reaches: [0, 1].map(|walks| (stand(walks), east_reach())).to_vec(),
        ..Hosted::default()
    });
    start.kingdom.heroes = vec![KingdomHero {
        points: 1560,
        ..KingdomHero::default()
    }];
    start.kingdom.selected = Selected::Hero(0);
    start.finds = vec![Found {
        at: at(21, 17),
        first: Vec::new(),
        again: None,
        gain: Gain {
            experience: 1000,
            ..Gain::default()
        },
        found: None,
        taken: None,
        event: false,
        blocks: None,
        ask: None,
        flag: None,
        fight: None,
        walk: Some(0),
        levels: vec![LevelBox {
            hosted: false,
            ..hosted_level()
        }],
    }];
    let mut l = Loop::start(rules(Bugs::Fixed), &start).unwrap();
    let click = Action::Click(tile(21, 17));
    l.run(1, &[click]).unwrap();
    l.run(2, &[click]).unwrap();
    for t in 3..=900 {
        l.run(t, &[]).unwrap();
    }
    assert!(l.boxed.is_some());
    l.run(901, &[Action::BoxKey(Key::Ok), Action::BoxKeyUp(Key::Ok)])
        .unwrap();
    assert!(l.boxed.is_none());
    assert_eq!(l.level_asked(), None);
}

#[test]
fn space_on_an_entrance_asks_the_host_its_revisit() {
    // E24 M5 T4b (design §5.27): the hero on the monster's entrance (`hovering`'s (5, 4)), live
    // with a host: Space asks the engine's revisit; given its find, told (its box); given none,
    // nothing; the next Space (a new stand) asks again.
    let mut l = hovering(true, Bugs::Fixed);
    l.index = Some(0);
    l.moves = Some(Arc::new(Vec::new()));
    let reaches = (0..3).map(|answers| {
        (
            Stand {
                answers,
                ..stand(0)
            },
            Arc::default(),
        )
    });
    l.host = Some(Arc::new(Hosted {
        reaches: reaches.collect(),
        ..Hosted::default()
    }));
    let hero = l.hero.as_mut().unwrap();
    hero.at = at(5, 4);
    let before = l.clone();
    l.run(1, &[Action::Visit]).unwrap();
    let asked = l.revisit_asked().unwrap();
    assert_eq!(
        asked,
        RevisitAsked {
            at: 1,
            stand: stand(0)
        }
    );
    assert!(l.boxed.is_none());
    // Given a find with a box: told at once.
    let found = Found {
        at: at(5, 4),
        first: vec![(laid("AGAIN.", Style::Ok), Gain::default())],
        again: None,
        gain: Gain::default(),
        found: None,
        taken: None,
        event: false,
        blocks: None,
        ask: None,
        flag: None,
        fight: None,
        walk: Some(crate::engine::REVISIT),
        levels: Vec::new(),
    };
    let mut told = before.clone();
    told.add_revisit(&asked, Some(&found));
    assert_eq!(told.revisit_asked(), None);
    told.run(1, &[Action::Visit]).unwrap();
    assert!(told.boxed.is_some());
    // Given none: nothing; Space again, a new stand, asked again.
    let mut l = before;
    l.add_revisit(&asked, None);
    l.run(1, &[Action::Visit]).unwrap();
    assert!((l.boxed.is_none(), l.revisit_asked()) == (true, None));
    l.run(2, &[Action::Visit]).unwrap();
    let again = Stand {
        answers: 1,
        ..stand(0)
    };
    assert_eq!(l.revisit_asked().map(|a| a.stand), Some(again));
}
