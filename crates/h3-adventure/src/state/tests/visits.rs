//! Tests of the visits' block (E08 T61b; design §10): a hero's marks, a player's knowledge, the
//! emptied objects, and the state's hash and reader.

use std::sync::Arc;

use h3_contracts::{HeroId, PlayerId};
use h3_kernel::hash::hash_of;

use super::*;
use crate::state::Visits;

const RED: PlayerId = PlayerId::new(0);
const BLUE: PlayerId = PlayerId::new(1);

#[test]
fn visits_keep_each_heros_marks_each_players_knowledge_and_the_emptied_objects() {
    // 40 objects: two rows of the grid.
    let mut visits = Visits::new(40, 2);
    let (one, two) = (HeroId::new(1), HeroId::new(2));
    assert!(!visits.is_marked(one, 0) && !visits.knows(RED, 0) && !visits.is_emptied(0));
    visits.mark(one, 35);
    visits.mark(two, 3);
    assert!(visits.is_marked(one, 35) && !visits.is_marked(one, 3));
    assert!(visits.is_marked(two, 3) && !visits.is_marked(two, 35));
    // An index past the objects marks nothing, and a hero without marks has none.
    visits.mark(HeroId::new(9), 40);
    assert!(!visits.is_marked(HeroId::new(9), 40));
    assert_eq!(visits, {
        let mut same = Visits::new(40, 2);
        same.mark(two, 3);
        same.mark(one, 35);
        same
    });
    visits.learn(BLUE, 39);
    assert!(visits.knows(BLUE, 39) && !visits.knows(RED, 39));
    // A player not in the game learns and knows nothing.
    visits.learn(PlayerId::new(2), 1);
    assert!(!visits.knows(PlayerId::new(2), 1));
    visits.empty(7);
    assert!(visits.is_emptied(7) && !visits.is_emptied(8));
}

#[test]
fn the_hash_sees_every_part_of_the_visits() {
    let base = Visits::new(40, 2);
    let mut hashes = BTreeSet::new();
    hashes.insert(hash_of(&base));
    let mut marked = base.clone();
    marked.mark(HeroId::new(1), 5);
    let mut other_hero = base.clone();
    other_hero.mark(HeroId::new(2), 5);
    let mut known = base.clone();
    known.learn(RED, 5);
    let mut known_blue = base.clone();
    known_blue.learn(BLUE, 5);
    let mut emptied = base.clone();
    emptied.empty(5);
    for visits in [marked, other_hero, known, known_blue, emptied] {
        assert!(hashes.insert(hash_of(&visits)));
    }
}

#[test]
fn a_state_has_no_visits_until_one_is_written_and_its_hash_sees_them() {
    let (core, part) = rules();
    let mut draws = draws();
    let state = AdventureState::new(&core, &part, &setup(), &mut draws).unwrap();
    assert_eq!(state.visits(), None);
    let mut visited = state.clone();
    let mut visits = Visits::new(state.map().objects().len(), 2);
    visits.mark(HeroId::new(1), 0);
    visited.visits = Some(Arc::new(visits.clone()));
    assert_eq!(visited.visits(), Some(&visits));
    assert_ne!(visited.state_hash(), state.state_hash());
    // An empty block is still one: its hash differs from none's.
    let mut empty = state.clone();
    empty.visits = Some(Arc::new(Visits::new(state.map().objects().len(), 2)));
    assert_ne!(empty.state_hash(), state.state_hash());
    assert_ne!(empty, state);
}
