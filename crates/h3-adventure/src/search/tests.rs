//! Tests of the computer's search (E31 T2i0; CPU-PTH-001 to -005), each expectation computed by
//! hand from the original's rules as `search.rs` cites them: grass costs 100 a step, 141
//! diagonally (100 with 100 to 141 points), swamp 175 (247), the fly and water-walk prices 140 at
//! level 1, a boat 100 (141); the day's maximum 1 500 on land (a pikeman's speed 4) and at sea.

extern crate std;

use std::vec;
use std::vec::Vec;

use h3_content::{FactionId, Formation, SkillId};
use h3_contracts::ArmyStack;

use super::*;
use crate::ruleset::{
    BankDef, BankLevelDef, BankQuestion, MonolithWay, ObjectKindDef, ObjectTypeDef,
};
use crate::setup::{HeroSetup, Holds, ObjectSetup, Scenery, TileMask};
use crate::testing::{
    BARRENS, FLYING, PIKEMAN, WATER_WALKING, coasted, hero_at, hero_details, hero_skills, key,
    lying, movement_rules_with, started, world,
};

type Rules = (Ruleset, AdventureRuleset);

const RED: PlayerId = PlayerId::new(0);
const BLUE: PlayerId = PlayerId::new(1);
const HERO: HeroId = HeroId::new(1);

fn p(x: u16, y: u16) -> Pos {
    Pos::new(x, y, 0)
}

/// [`movement_rules_with`] (faithful) with [`hero_skills`] (the flyer's and the water walker's
/// levels) and four types appended: a town (entered, not from above), a garrison and an
/// anti-magic one (entered, from above) and a coast bank, the Shipwreck (visited from a
/// neighbour, from above).
fn rules() -> Rules {
    movement_rules_with(
        true,
        16,
        |core| core.skills = hero_skills(),
        |part| {
            let ty = |k: &str, visit, from_above, kind| ObjectTypeDef {
                key: key(k),
                name: "Object".into(),
                visit,
                from_above,
                kind_mark: false,
                kind,
            };
            let level = BankLevelDef {
                chance: 100,
                guards: vec![(key("t:peasant"), 10)],
                upgrade: 0,
                resources: vec![],
                creature: None,
                artifacts: [0; 4],
            };
            part.object_types.extend([
                ty(
                    "t:town",
                    Visit::Enter,
                    false,
                    ObjectKindDef::Town {
                        boat_offsets: Vec::new(),
                    },
                ),
                ty(
                    "t:garrison",
                    Visit::Enter,
                    true,
                    ObjectKindDef::Garrison { anti_magic: false },
                ),
                ty(
                    "t:anti_garrison",
                    Visit::Enter,
                    true,
                    ObjectKindDef::Garrison { anti_magic: true },
                ),
                ty(
                    "t:shipwreck",
                    Visit::Neighbour,
                    true,
                    ObjectKindDef::Bank(BankDef {
                        question: BankQuestion::Search,
                        levels: vec![level],
                        looted: None,
                        deck: true,
                        coast: true,
                    }),
                ),
            ]);
            // The teleporters (CPU-PTH-006), entered and from above.
            let monolith = |k: &str, channel, way| {
                let kind = ObjectKindDef::Monolith { channel, way };
                ty(k, Visit::Enter, true, kind)
            };
            part.object_types.extend([
                monolith("t:entrance", 0, MonolithWay::Entrance),
                monolith("t:entrance_1", 1, MonolithWay::Entrance),
                monolith("t:exit", 0, MonolithWay::Exit),
                monolith("t:two_way", 1, MonolithWay::TwoWay),
                ty(
                    "t:underground_gate",
                    Visit::Enter,
                    true,
                    ObjectKindDef::SubterraneanGate,
                ),
                ty(
                    "t:whirlpool",
                    Visit::Enter,
                    true,
                    ObjectKindDef::Whirlpool { divisor: 2 },
                ),
            ]);
        },
    )
}

/// A teleporter of type `k` on `at`.
fn teleporter(rules: &Rules, k: &str, at: Pos) -> ObjectSetup {
    object(rules, k, at, None, Holds::Nothing)
}

/// The started game of `rows` on two levels of equal height with `heroes` and `objects` added,
/// every tile explored.
fn deep(
    rules: &Rules,
    rows: &[&str],
    heroes: Vec<HeroSetup>,
    objects: Vec<ObjectSetup>,
) -> AdventureState {
    let mut setup = world(rows, 2, heroes);
    setup.map.objects.extend(objects);
    let mut state = started(rules, &setup).unwrap();
    state.explore_all();
    state
}

/// An object of type `k` blocking and visited on `at`, of `owner`, holding `holds`.
fn object(rules: &Rules, k: &str, at: Pos, owner: Option<PlayerId>, holds: Holds) -> ObjectSetup {
    ObjectSetup {
        scenery: if matches!(holds, Holds::Town { .. }) {
            Scenery::Ownable
        } else {
            Scenery::default()
        },
        anchor: at,
        blocked: TileMask::from_bits(1),
        visitable: TileMask::from_bits(1),
        object_type: rules.1.object_type_by_key(k),
        amount: None,
        drawn: TileMask::default(),
        monster: None,
        guards: None,
        holds,
        owner,
    }
}

/// A town of `owner` on `at`.
fn town(rules: &Rules, at: Pos, owner: Option<PlayerId>) -> ObjectSetup {
    let holds = Holds::Town {
        formation: Formation::Loose,
    };
    object(rules, "t:town", at, owner, holds)
}

/// A garrison of type `k` and `owner` on `at`, holding a peasant.
fn garrison(rules: &Rules, k: &str, at: Pos, owner: Option<PlayerId>) -> ObjectSetup {
    let army = vec![ArmyStack {
        slot: 0,
        creature: crate::testing::PEASANT,
        count: 1,
    }];
    let holds = Holds::Garrison {
        army,
        removable: true,
    };
    object(rules, k, at, owner, holds)
}

/// A monster of five of the first creature on `at` (the world's `M`, anywhere).
fn monster(at: Pos) -> ObjectSetup {
    ObjectSetup {
        anchor: at,
        ..world(&["M"], 1, vec![]).map.objects.remove(0)
    }
}

/// The started game of `rows` (the coast flagged, [`coasted`]) with `heroes` and `objects`
/// added, every tile explored.
fn game(
    rules: &Rules,
    rows: &[&str],
    heroes: Vec<HeroSetup>,
    objects: Vec<ObjectSetup>,
) -> AdventureState {
    let mut setup = coasted(rows, heroes);
    for object in objects {
        if matches!(object.holds, Holds::Town { .. }) {
            setup.town_factions.push(FactionId::from_raw(0));
            setup.garrisons.push(Vec::new());
        }
        setup.map.objects.push(object);
    }
    let mut state = started(rules, &setup).unwrap();
    state.explore_all();
    state
}

/// [`game`] with red's hero [`HERO`], a pikeman, on `at` alone.
fn alone(rules: &Rules, rows: &[&str], at: Pos, objects: Vec<ObjectSetup>) -> AdventureState {
    game(rules, rows, vec![hero_at(1, 0, at, &[PIKEMAN])], objects)
}

/// The game's store with [`HERO`]'s details of `skills` (index 1).
fn store(skills: &[(SkillId, u8)]) -> Vec<Arc<HeroDetails>> {
    vec![Arc::new(hero_details(&[])), Arc::new(hero_details(skills))]
}

/// Red's search of [`HERO`] from `from` in `mode` with 1 000 points, no bound, the computer's
/// fog, the whole map.
fn start(from: Pos, mode: SearchMode) -> SearchStart {
    SearchStart {
        player: RED,
        hero: HERO,
        from,
        boat: false,
        bound: 0,
        points: 1000,
        mode,
        fog: SearchFog::Computer,
        window: None,
        jumps: SearchJumps::default(),
    }
}

/// The search of `start` on `state` with the store `heroes` and `asks`.
fn search_with<A: SearchAsks>(
    rules: &Rules,
    state: &AdventureState,
    heroes: &[Arc<HeroDetails>],
    start: &SearchStart,
    asks: &A,
) -> ComputerSearch {
    state
        .computer_search(&rules.0, &rules.1, heroes, start, asks)
        .unwrap()
}

/// [`search_with`] without details and asks.
fn search(rules: &Rules, state: &AdventureState, start: &SearchStart) -> ComputerSearch {
    search_with(rules, state, &[], start, &NoAsks)
}

/// The marks of a place reached in `direction` from a place.
const fn placed(direction: u32) -> u32 {
    REACHED | PLACE | FROM_PLACE | direction << DIRECTION_SHIFT
}

/// The caller's numbers by tile: the danger and the fight answers, 0 elsewhere, and the boat
/// marks with their purchase's change of the key.
#[derive(Default)]
struct Asks {
    danger: Vec<(Pos, i32)>,
    answers: Vec<(Pos, i32)>,
    boats: Vec<(Pos, i32)>,
}

/// The value of `pos` in `list`, 0 without one.
fn value(list: &[(Pos, i32)], pos: Pos) -> i32 {
    list.iter()
        .find(|(at, _)| *at == pos)
        .map_or(0, |&(_, value)| value)
}

impl SearchAsks for Asks {
    fn danger(&self, pos: Pos) -> i32 {
        value(&self.danger, pos)
    }

    fn fight_answer(&self, pos: Pos) -> i32 {
        value(&self.answers, pos)
    }

    fn boat_mark(&self, pos: Pos) -> Option<i32> {
        self.boats
            .iter()
            .find(|(at, _)| *at == pos)
            .map(|&(_, change)| change)
    }
}

/// Red's search of [`HERO`] from `from` in `mode` with `jumps` and `asks`, the store `heroes`.
fn jumping<A: SearchAsks>(
    rules: &Rules,
    state: &AdventureState,
    heroes: &[Arc<HeroDetails>],
    (from, mode, jumps): (Pos, SearchMode, SearchJumps),
    asks: &A,
) -> ComputerSearch {
    let start = SearchStart {
        jumps,
        ..start(from, mode)
    };
    search_with(rules, state, heroes, &start, asks)
}

/// The jumps of Dimension Door alone at 300 points a leg.
fn door() -> SearchJumps {
    SearchJumps {
        door: true,
        door_price: 300,
        ..SearchJumps::default()
    }
}

/// The marks of a node relaxed from a place in `direction` (no place itself).
const fn passed(direction: u32) -> u32 {
    REACHED | FROM_PLACE | direction << DIRECTION_SHIFT
}

/// A Door leg's offset `(dx, dy)` in the marks.
const fn offset(dx: i32, dy: i32) -> u32 {
    (((dy & 0x1F) << 5 | (dx & 0x1F)) as u32) << DOOR_OFFSET_SHIFT
}

/// A corridor of five grass tiles between rock, `middle` on its third.
fn corridor(middle: char) -> [std::string::String; 3] {
    let mut row: std::string::String = "ggggg".into();
    row.replace_range(2..3, &std::string::String::from(middle));
    ["#####".into(), row, "#####".into()]
}

fn rows(rows: &[std::string::String; 3]) -> [&str; 3] {
    [rows[0].as_str(), rows[1].as_str(), rows[2].as_str()]
}

// covers: CPU-PTH-001
#[test]
fn the_start_node_and_its_steps_are_reached_in_the_original_s_order() {
    let rules = rules();
    let state = alone(&rules, &["ggg", "ggg", "ggg"], p(1, 1), vec![]);
    let found = search(&rules, &state, &start(p(1, 1), SearchMode::Local));
    let first = SearchNode {
        pos: p(1, 1),
        marks: placed(2),
        from: p(1, 1),
        guard: None,
        key: 0,
        danger: 0,
        cost: 0,
        cost2: 0,
        points: 1000,
    };
    assert_eq!(found.node(p(1, 1)), Some(first));
    let up = SearchNode {
        pos: p(1, 0),
        marks: placed(0),
        cost: 100,
        cost2: 100,
        points: 900,
        ..first
    };
    assert_eq!(found.node(p(1, 0)), Some(up));
    let diagonal = found.node(p(2, 0)).unwrap();
    assert_eq!(
        (diagonal.marks, diagonal.cost, diagonal.points),
        (placed(1), 141, 859)
    );
    // First-reach order: the start, then its steps in the eight directions.
    let order: Vec<Pos> = found.reached().map(|node| node.pos).collect();
    let expected = [
        (1, 1),
        (1, 0),
        (2, 0),
        (2, 1),
        (2, 2),
        (1, 2),
        (0, 2),
        (0, 1),
        (0, 0),
    ];
    assert_eq!(order, expected.map(|(x, y)| p(x, y)));
    assert_eq!(
        found.reached().next_back().map(|node| node.pos),
        Some(p(0, 0))
    );
    assert_eq!(found.pass_node(p(1, 0)), None);
    assert_eq!(found.node(p(5, 5)), None);
    assert_eq!(found.node(Pos::new(0, 0, 1)), None);
    assert!(!found.short());
    // A hero not on the map has no search; the asks' defaults.
    let missing = SearchStart {
        hero: HeroId::new(7),
        ..start(p(1, 1), SearchMode::Local)
    };
    assert!(
        state
            .computer_search(&rules.0, &rules.1, &[], &missing, &NoAsks)
            .is_none()
    );
    assert_eq!(
        (NoAsks.danger(p(0, 0)), NoAsks.boat_mark(p(0, 0))),
        (0, None)
    );
}

// covers: CPU-PTH-001, CPU-PTH-002
#[test]
fn a_better_second_cost_replaces_the_first_and_a_repeated_entry_is_skipped() {
    // (2, 1) is first reached from the swamp at 100 + 247 and then from (1, 1) at 141 + 100,
    // whose entry goes in right behind the first: the pop skips it for the old one, which
    // reloads the cell's final node.
    let rules = rules();
    let state = alone(&rules, &["gw#", "ggg"], p(0, 0), vec![]);
    let found = search(&rules, &state, &start(p(0, 0), SearchMode::Local));
    let corner = found.node(p(2, 1)).unwrap();
    assert_eq!(
        (
            corner.from,
            corner.marks,
            corner.cost,
            corner.cost2,
            corner.points
        ),
        (p(1, 1), placed(2), 241, 241, 759)
    );
    let order: Vec<Pos> = found.reached().map(|node| node.pos).collect();
    assert_eq!(order, vec![p(0, 0), p(1, 0), p(1, 1), p(0, 1), p(2, 1)]);
}

// covers: CPU-PTH-001, CPU-PTH-002
#[test]
fn the_open_list_keeps_the_original_s_order_and_drops_its_best_at_500() {
    let rules = rules();
    let state = alone(&rules, &["ggg", "ggg", "ggg"], p(1, 1), vec![]);
    let start = start(p(1, 1), SearchMode::Local);
    let (search, mut frame) = state
        .prepare(&rules.0, &rules.1, &[], &start, &NoAsks)
        .unwrap();
    search.begin(&mut frame);
    let origin = frame.bounds.index(p(1, 1), true).unwrap();
    let source = frame.load(origin);
    let step = |frame: &mut Frame, pos: Pos, price: u32, key: i32| {
        let mut node = SearchNode { pos, ..source };
        search.relax(frame, &source, &mut node, (0, price), (key, None, false));
    };
    step(&mut frame, p(1, 0), 100, 0);
    step(&mut frame, p(2, 0), 141, 0);
    step(&mut frame, p(0, 1), 100, 0);
    step(&mut frame, p(0, 0), 100, -5);
    let cells = |frame: &Frame| -> Vec<Pos> {
        frame
            .open
            .iter()
            .map(|entry| frame.bounds.pos(entry.cell as usize))
            .collect()
    };
    // Ascending S, then descending second cost, a new equal entry before the older ones.
    assert_eq!(
        cells(&frame),
        vec![p(0, 0), p(2, 0), p(0, 1), p(1, 0), p(1, 1)]
    );
    let low = Entry {
        score: -1000,
        cost2: 0,
        cell: 0,
    };
    frame.open.splice(0..0, vec![low; 495]);
    assert_eq!(frame.open.len(), 500);
    // At 500 the back, the best (the start's), goes before the insert.
    step(&mut frame, p(2, 1), 100, 0);
    assert_eq!(frame.open.len(), 500);
    assert_eq!(
        cells(&frame)[495..],
        [p(0, 0), p(2, 0), p(2, 1), p(0, 1), p(1, 0)]
    );
}

// covers: CPU-PTH-002
#[test]
fn the_bound_refuses_a_first_reach_past_it_and_marks_the_search_short() {
    let rules = rules();
    let owned = corridor('g');
    let rows = rows(&owned);
    let state = alone(&rules, &rows, p(0, 1), vec![]);
    let bounded = |bound| {
        let start = SearchStart {
            bound,
            ..start(p(0, 1), SearchMode::Local)
        };
        search(&rules, &state, &start)
    };
    let found = bounded(250);
    assert_eq!(found.node(p(2, 1)).map(|node| node.cost), Some(200));
    assert_eq!(found.node(p(3, 1)), None);
    assert!(found.short());
    let found = bounded(300);
    assert_eq!(found.node(p(3, 1)).map(|node| node.cost), Some(300));
    assert!(found.short());
    let found = bounded(0);
    assert_eq!(found.node(p(4, 1)).map(|node| node.cost), Some(400));
    assert!(!found.short());
}

// covers: CPU-PTH-001, CPU-PTH-002, CPU-PTH-003
#[test]
fn past_the_day_s_points_the_danger_orders_and_a_higher_score_replaces() {
    // 100 points: (1, 0) keeps its buffer's -5 on the day; past the day (2, 0) and (2, 1) take
    // the lesser of theirs and the source's, -5 from (1, 0), 0 from the swamp (1, 1), whose
    // higher score replaces them at a higher cost, (2, 0) past the bound of 300 too.
    let rules = rules();
    let state = alone(&rules, &["ggg", "gwg"], p(0, 0), vec![]);
    let asks = Asks {
        danger: vec![(p(0, 0), -2), (p(1, 0), -5)],
        ..Asks::default()
    };
    let start = SearchStart {
        points: 100,
        bound: 300,
        ..start(p(0, 0), SearchMode::Local)
    };
    let found = search_with(&rules, &state, &[], &start, &asks);
    assert_eq!(found.node(p(0, 0)).map(|node| node.danger), Some(-2));
    let near = found.node(p(1, 0)).unwrap();
    assert_eq!((near.danger, near.cost, near.points), (-5, 100, 0));
    // The diagonal with 100 points costs the straight step.
    assert_eq!(found.node(p(1, 1)).map(|node| node.cost), Some(100));
    let far = found.node(p(2, 0)).unwrap();
    assert_eq!(
        (
            far.from, far.marks, far.danger, far.cost, far.cost2, far.points
        ),
        (p(1, 1), placed(1), 0, 347, 347, 1500)
    );
    let side = found.node(p(2, 1)).unwrap();
    assert_eq!(
        (side.from, side.danger, side.cost, side.points),
        (p(1, 1), 0, 275, 1500)
    );
    assert!(!found.short());
    let order: Vec<Pos> = found.reached().map(|node| node.pos).collect();
    assert_eq!(
        order,
        vec![p(0, 0), p(1, 0), p(1, 1), p(0, 1), p(2, 0), p(2, 1)]
    );
}

// covers: CPU-PTH-002
#[test]
fn a_danger_at_minus_500_million_past_the_day_refuses_the_node() {
    let rules = rules();
    let state = alone(&rules, &["gggg"], p(0, 0), vec![]);
    let asks = Asks {
        danger: vec![(p(1, 0), -499_999_999), (p(2, 0), REFUSED)],
        ..Asks::default()
    };
    let start = SearchStart {
        points: 0,
        ..start(p(0, 0), SearchMode::Local)
    };
    let found = search_with(&rules, &state, &[], &start, &asks);
    let near = found.node(p(1, 0)).unwrap();
    assert_eq!((near.danger, near.points), (-499_999_999, 1500));
    assert_eq!(found.node(p(2, 0)), None);
    assert_eq!(found.node(p(3, 0)), None);
}

// covers: CPU-PTH-003
#[test]
fn a_diagonal_costs_the_straight_step_inside_the_window() {
    let rules = rules();
    let state = alone(&rules, &["gg", "gg"], p(0, 0), vec![]);
    let with = |points| {
        let start = SearchStart {
            points,
            ..start(p(0, 0), SearchMode::Local)
        };
        search(&rules, &state, &start).node(p(1, 1)).unwrap()
    };
    assert_eq!((with(120).cost, with(120).points), (100, 20));
    assert_eq!((with(1000).cost, with(1000).points), (141, 859));
}

// covers: CPU-PTH-001, CPU-PTH-003
#[test]
fn a_flyer_passes_a_blocked_tile_as_a_pass_node_while_its_points_last() {
    let rules = rules();
    let heroes = store(&[(FLYING, 1)]);
    for middle in ['B', 'V'] {
        let owned = corridor(middle);
        let rows = rows(&owned);
        let state = alone(&rules, &rows, p(1, 1), vec![]);
        let with = |points| {
            let start = SearchStart {
                points,
                ..start(p(1, 1), SearchMode::Local)
            };
            search_with(&rules, &state, &heroes, &start, &NoAsks)
        };
        let found = with(1000);
        assert_eq!(
            found.node(p(1, 1)).map(|node| node.marks),
            Some(placed(2) | FLY)
        );
        assert_eq!(found.node(p(2, 1)), None);
        let over = found.pass_node(p(2, 1)).unwrap();
        assert_eq!(
            (over.marks, over.cost, over.points),
            (REACHED | FROM_PLACE | FLY | 2 << DIRECTION_SHIFT, 100, 900)
        );
        // From the blocked tile the fly price, 140; the place beyond has no place before it.
        // A visitable tile without a built visit is priced as an entrance (map item `+0xC`
        // bit 0x40 set): the lesser of 100 and 140.
        let beyond = found.node(p(3, 1)).unwrap();
        let price = if middle == 'B' { 140 } else { 100 };
        assert_eq!(
            (beyond.marks, beyond.cost, beyond.from),
            (
                REACHED | PLACE | FLY | 2 << DIRECTION_SHIFT,
                100 + price,
                p(2, 1)
            )
        );
        // 150 points: the pass node's 50 do not pay 140, and a pass node starts no day.
        assert_eq!(with(150).node(p(3, 1)), None);
        // 50 points: the step starts a day from the start, a walking one, which a blocked
        // tile stops.
        assert_eq!(with(50).pass_node(p(2, 1)), None);
        // A walker never steps onto it.
        let walker = search(&rules, &state, &start(p(1, 1), SearchMode::Local));
        assert_eq!(walker.node(p(3, 1)), None);
    }
}

// covers: CPU-PTH-003
#[test]
fn a_new_day_prices_the_step_again_without_flying_and_gives_the_whole_maximum() {
    let rules = rules();
    let heroes = store(&[(FLYING, 1)]);
    let state = alone(&rules, &["wgg"], p(0, 0), vec![]);
    let with = |points| {
        let start = SearchStart {
            points,
            ..start(p(0, 0), SearchMode::Local)
        };
        let found = search_with(&rules, &state, &heroes, &start, &NoAsks);
        let next = found.node(p(1, 0)).unwrap();
        (next.marks & FLY, next.cost, next.points)
    };
    // From swamp a flight costs 140, the lesser of it and 175.
    assert_eq!(with(1000), (FLY, 140, 860));
    // Its 100 points do not pay it: a new day, priced 175 on foot, the whole 1 500.
    assert_eq!(with(100), (0, 175, 1500));
}

// covers: CPU-PTH-003
#[test]
fn a_water_walker_crosses_water_over_pass_nodes() {
    let rules = rules();
    let heroes = store(&[(WATER_WALKING, 1)]);
    let state = alone(&rules, &["g~~g"], p(0, 0), vec![]);
    let found = search_with(
        &rules,
        &state,
        &heroes,
        &start(p(0, 0), SearchMode::Local),
        &NoAsks,
    );
    let first = found.pass_node(p(1, 0)).unwrap();
    assert_eq!(
        (first.marks, first.cost),
        (
            REACHED | FROM_PLACE | WATER_WALK | 2 << DIRECTION_SHIFT,
            100
        )
    );
    assert_eq!(found.node(p(1, 0)), None);
    assert_eq!(found.pass_node(p(2, 0)).map(|node| node.cost), Some(240));
    let shore = found.node(p(3, 0)).unwrap();
    assert_eq!(
        (shore.marks, shore.cost),
        (REACHED | PLACE | WATER_WALK | 2 << DIRECTION_SHIFT, 380)
    );
    let walker = search(&rules, &state, &start(p(0, 0), SearchMode::Local));
    assert_eq!(walker.pass_node(p(1, 0)), None);
    assert_eq!(walker.node(p(3, 0)), None);
}

// covers: CPU-PTH-003
#[test]
fn the_fog_follows_the_policy_and_the_mode() {
    // Red's hero on (0, 0) of a row; (2, 0) and (3, 0) unexplored; a town of red's in a pocket.
    let rules = rules();
    let rows = ["gggg", "####", "g###"];
    let mut state = alone(&rules, &rows, p(0, 0), vec![]);
    let mut owning = alone(
        &rules,
        &rows,
        p(0, 0),
        vec![town(&rules, p(0, 2), Some(RED))],
    );
    for state in [&mut state, &mut owning] {
        let (grid, _) = state.explored_mut(RED);
        grid.set(p(2, 0), false);
        grid.set(p(3, 0), false);
    }
    let heroes = store(&[(FLYING, 1)]);
    let reach = |state: &AdventureState, fog, mode, heroes: &[Arc<HeroDetails>]| {
        let start = SearchStart {
            fog,
            ..start(p(0, 0), mode)
        };
        let found = search_with(&rules, state, heroes, &start, &NoAsks);
        (
            found.node(p(2, 0)).is_some(),
            found.node(p(3, 0)).is_some(),
            found,
        )
    };
    use SearchFog::{Computer, Human};
    use SearchMode::{Local, Threat};
    // The computer's fog: one step into it; on from it while the player owns no town.
    let (into, on, _) = reach(&state, Computer, Local, &[]);
    assert_eq!((into, on), (true, true));
    let (into, on, _) = reach(&owning, Computer, Local, &[]);
    assert_eq!((into, on), (true, false));
    let (_, _, flown) = reach(&owning, Computer, Local, &heroes);
    assert!(flown.pass_node(p(3, 0)).is_some());
    // The human's: never into it on foot; mode 3 has no fog.
    let (into, _, _) = reach(&state, Human, Local, &[]);
    assert!(!into);
    let (into, on, _) = reach(&owning, Human, Threat, &[]);
    assert_eq!((into, on), (true, true));
}

// covers: CPU-PTH-003
#[test]
fn an_object_not_visited_from_above_closes_its_top_both_ways() {
    let rules = rules();
    let rows = ["ggg", "gCg", "ggg"];
    // On the chart: a walker goes up only round it; a flyer flies up, its place kept.
    let state = alone(&rules, &rows, p(1, 1), vec![]);
    let walker = search(&rules, &state, &start(p(1, 1), SearchMode::Local));
    let up = walker.node(p(1, 0)).unwrap();
    assert_eq!((up.from, up.cost), (p(2, 1), 241));
    assert_eq!(walker.node(p(0, 0)).map(|node| node.from), Some(p(0, 1)));
    let heroes = store(&[(FLYING, 1)]);
    let flyer = search_with(
        &rules,
        &state,
        &heroes,
        &start(p(1, 1), SearchMode::Local),
        &NoAsks,
    );
    let up = flyer.node(p(1, 0)).unwrap();
    assert_eq!(
        (up.from, up.cost, up.marks),
        (p(1, 1), 100, placed(0) | FLY)
    );
    // Above the chart: never down onto it, but from the side.
    let state = alone(&rules, &rows, p(1, 0), vec![]);
    let walker = search(&rules, &state, &start(p(1, 0), SearchMode::Local));
    let chart = walker.node(p(1, 1)).unwrap();
    assert_eq!(
        (chart.from, chart.cost, chart.marks & STOP),
        (p(2, 1), 241, STOP)
    );
}

// covers: CPU-PTH-003, CPU-PTH-005
#[test]
fn a_walker_steps_onto_a_boat_a_hero_or_a_coast_bank_on_water_and_boards_from_mode_4() {
    let rules = rules();
    let state = alone(&rules, &["gb~~"], p(0, 0), vec![]);
    let local = search(&rules, &state, &start(p(0, 0), SearchMode::Local));
    let boat = local.node(p(1, 0)).unwrap();
    assert_eq!(
        (boat.marks, boat.cost, boat.points),
        (placed(2) | STOP, 100, 900)
    );
    assert_eq!(local.node(p(2, 0)), None);
    // Mode 4: the boarding adds the day's rest to the cost and goes on at sea; the boat's cell
    // keeps its cost and points.
    let plan = search(&rules, &state, &start(p(0, 0), SearchMode::Plan));
    let boat = plan.node(p(1, 0)).unwrap();
    assert_eq!(
        (boat.marks, boat.cost, boat.points),
        (placed(2) | STOP | BOAT, 100, 900)
    );
    let sea = plan.node(p(2, 0)).unwrap();
    assert_eq!(
        (sea.marks, sea.cost, sea.cost2, sea.points, sea.danger),
        (placed(2) | BOAT, 1100, 200, 1400, 0)
    );
    assert_eq!(plan.node(p(3, 0)).map(|node| node.cost), Some(1200));
    // Another hero in a boat and a coast bank's entrance: stops, never gone past (an empty boat
    // beyond).
    let mut state = game(
        &rules,
        &["gb~", "g~~", "g~b"],
        vec![
            hero_at(1, 0, p(0, 0), &[PIKEMAN]),
            hero_at(2, 1, p(0, 1), &[PIKEMAN]),
        ],
        vec![object(&rules, "t:shipwreck", p(1, 2), None, Holds::Nothing)],
    );
    state.put_aboard(HeroId::new(2), 0);
    let found = search(&rules, &state, &start(p(0, 0), SearchMode::Threat));
    assert_eq!(
        found.node(p(1, 0)).map(|node| node.marks & STOP),
        Some(STOP)
    );
    assert_eq!(
        found.node(p(1, 2)).map(|node| node.marks & STOP),
        Some(STOP)
    );
    assert_eq!(found.node(p(2, 2)), None);
    // Any other entrance on water a walker never steps onto; plain water neither.
    assert_eq!(found.node(p(1, 1)), None);
    let state = alone(&rules, &["g~", "g~"], p(0, 0), vec![monster(p(1, 0))]);
    let found = search(&rules, &state, &start(p(0, 0), SearchMode::Local));
    assert_eq!(found.node(p(1, 0)), None);
    // A flyer passes over it, a stop without a place, which has no flying copy.
    let heroes = store(&[(FLYING, 1)]);
    let found = search_with(
        &rules,
        &state,
        &heroes,
        &start(p(0, 0), SearchMode::Local),
        &NoAsks,
    );
    let over = found.pass_node(p(1, 0)).unwrap();
    assert_eq!(over.marks & (STOP | PLACE), STOP);
}

// covers: CPU-PTH-001, CPU-PTH-003
#[test]
fn a_boat_sails_water_but_onto_a_boat_or_a_diagonal_past_land_and_lands_on_a_free_coast() {
    let rules = rules();
    let rows = ["b~b", "g~~", "P~~"];
    let mut state = alone(&rules, &rows, p(0, 1), vec![]);
    state.put_aboard(HERO, 0);
    let sail = |points| {
        let start = SearchStart {
            boat: true,
            points,
            ..start(p(0, 0), SearchMode::Local)
        };
        search(&rules, &state, &start)
    };
    let found = sail(1000);
    assert_eq!(
        found.node(p(0, 0)).map(|node| node.marks),
        Some(placed(2) | BOAT)
    );
    let east = found.node(p(1, 0)).unwrap();
    assert_eq!((east.marks, east.cost), (placed(2) | BOAT, 100));
    // Onto the other boat never; the diagonal past the coast never: round it.
    assert_eq!(found.node(p(2, 0)), None);
    let below = found.node(p(1, 1)).unwrap();
    assert_eq!((below.from, below.cost), (p(1, 0), 200));
    assert_eq!(found.node(p(2, 1)).map(|node| node.cost), Some(241));
    assert_eq!(
        found.node(p(2, 2)).map(|node| (node.from, node.cost)),
        Some((p(1, 1), 341))
    );
    // The free coast is a step kept in the boat, a leaf; the pile's coast none.
    let landed = found.node(p(0, 1)).unwrap();
    assert_eq!((landed.marks, landed.cost), (placed(4) | BOAT, 100));
    assert_eq!(found.node(p(0, 2)), None);
    // A boat's new day starts with the sea's maximum.
    assert_eq!(sail(50).node(p(1, 0)).map(|node| node.points), Some(1500));
}

// covers: CPU-PTH-005
#[test]
fn a_border_guard_and_gate_pass_by_the_current_player_s_key() {
    let rules = rules();
    let reach = |middle, mode, key| {
        let owned = corridor(middle);
        let rows = rows(&owned);
        let mut state = alone(&rules, &rows, p(0, 1), vec![]);
        if key {
            state.give_key(RED, 0);
        }
        let found = search(&rules, &state, &start(p(0, 1), mode));
        assert!(found.node(p(2, 1)).is_some());
        found.node(p(3, 1)).is_some()
    };
    use SearchMode::{Local, Plan};
    // A guard: from mode 4, by the key.
    assert!(!reach('K', Local, true));
    assert!(reach('K', Plan, true));
    assert!(!reach('K', Plan, false));
    // A gate: in every mode, by the key.
    assert!(reach('X', Local, true));
    assert!(!reach('X', Local, false));
    // The current player's key, not the searched hero's owner's: blue's in mode 3.
    let owned = corridor('X');
    let rows = rows(&owned);
    let mut state = game(
        &rules,
        &rows,
        vec![
            hero_at(1, 0, p(0, 1), &[PIKEMAN]),
            hero_at(2, 1, p(4, 1), &[PIKEMAN]),
        ],
        vec![],
    );
    state.give_key(RED, 0);
    let blue = SearchStart {
        hero: HeroId::new(2),
        ..start(p(4, 1), SearchMode::Threat)
    };
    assert!(search(&rules, &state, &blue).node(p(1, 1)).is_some());
}

// covers: CPU-PTH-005
#[test]
fn a_stop_passes_by_its_class_and_the_mode() {
    let rules = rules();
    // A grass tile below (3, 1) for a monster beside the third tile.
    let rows = ["#####", "ggggg", "###g#"];
    let reach = |objects: Vec<ObjectSetup>, heroes: Vec<HeroSetup>, mode| {
        let heroes = [vec![hero_at(1, 0, p(0, 1), &[PIKEMAN])], heroes].concat();
        let state = game(&rules, &rows, heroes, objects);
        let found = search(&rules, &state, &start(p(0, 1), mode));
        assert!(found.node(p(2, 1)).is_some());
        found.node(p(3, 1)).is_some()
    };
    use SearchMode::{Claims, Local, Plan, Threat};
    let at = p(2, 1);
    // A garrison: the hero's team's in every mode, any other from mode 4.
    assert!(reach(
        vec![garrison(&rules, "t:garrison", at, Some(RED))],
        vec![],
        Local
    ));
    assert!(!reach(
        vec![garrison(&rules, "t:garrison", at, Some(BLUE))],
        vec![],
        Local
    ));
    assert!(!reach(
        vec![garrison(&rules, "t:garrison", at, None)],
        vec![],
        Claims
    ));
    assert!(reach(
        vec![garrison(&rules, "t:garrison", at, None)],
        vec![],
        Plan
    ));
    // Another hero: another team's from mode 3.
    assert!(!reach(vec![], vec![hero_at(2, 1, at, &[PIKEMAN])], Claims));
    assert!(reach(vec![], vec![hero_at(2, 1, at, &[PIKEMAN])], Threat));
    assert!(!reach(vec![], vec![hero_at(2, 0, at, &[PIKEMAN])], Threat));
    // A monster: from mode 4.
    assert!(!reach(vec![monster(at)], vec![], Threat));
    assert!(reach(vec![monster(at)], vec![], Plan));
    // A town: from mode 3, unless a guard stops it there.
    assert!(!reach(vec![town(&rules, at, Some(BLUE))], vec![], Claims));
    assert!(reach(vec![town(&rules, at, Some(BLUE))], vec![], Threat));
    let guarded = || vec![town(&rules, at, None), monster(p(3, 2))];
    assert!(!reach(guarded(), vec![], Threat));
    assert!(reach(guarded(), vec![], Plan));
    // Any other: entered or not.
    let tower = world(&["O"], 1, vec![]).map.objects.remove(0);
    assert!(reach(
        vec![ObjectSetup {
            anchor: at,
            ..tower
        }],
        vec![],
        Local
    ));
    let pile = world(&["P"], 1, vec![]).map.objects.remove(0);
    assert!(!reach(
        vec![ObjectSetup { anchor: at, ..pile }],
        vec![],
        Plan
    ));
}

// covers: CPU-PTH-004
#[test]
fn below_mode_4_a_guarded_node_steps_only_onto_its_guard() {
    let rules = rules();
    let rows = ["##M", "ggg", "##g"];
    let state = alone(&rules, &rows, p(0, 1), vec![]);
    let local = search(&rules, &state, &start(p(0, 1), SearchMode::Local));
    assert_eq!(local.node(p(2, 0)).map(|node| node.from), Some(p(1, 1)));
    assert_eq!(local.node(p(2, 1)), None);
    assert_eq!(local.node(p(2, 2)), None);
    // Mode 4 weighs the guard instead; a flight and the start are never held.
    let plan = search(&rules, &state, &start(p(0, 1), SearchMode::Plan));
    assert!(plan.node(p(2, 2)).is_some());
    let heroes = store(&[(FLYING, 1)]);
    let flight = search_with(
        &rules,
        &state,
        &heroes,
        &start(p(0, 1), SearchMode::Local),
        &NoAsks,
    );
    assert!(flight.node(p(2, 1)).is_some());
    let state = alone(&rules, &rows, p(1, 1), vec![]);
    let from_guarded = search(&rules, &state, &start(p(1, 1), SearchMode::Local));
    assert!(from_guarded.node(p(2, 1)).is_some());
}

// covers: CPU-PTH-004
#[test]
fn the_guard_scan_goes_by_columns_and_leaves_out_the_one_met() {
    let rules = rules();
    // Monsters at (0, 2) and (1, 0) round (1, 1), and one on water at (2, 1).
    let rows = ["gg~", "gg~", "gg~"];
    let state = alone(&rules, &rows, p(0, 0), vec![]);
    let objects = vec![monster(p(0, 2)), monster(p(1, 0)), monster(p(2, 1))];
    let guarded = game(
        &rules,
        &rows,
        vec![hero_at(1, 0, p(1, 1), &[PIKEMAN])],
        objects,
    );
    let start = start(p(1, 1), SearchMode::Local);
    let (search, _) = guarded
        .prepare(&rules.0, &rules.1, &[], &start, &NoAsks)
        .unwrap();
    assert_eq!(search.guarding(p(1, 1), None), Some(p(0, 2)));
    assert_eq!(search.guarding(p(1, 1), Some(p(0, 2))), Some(p(1, 0)));
    assert_eq!(search.guarding(p(1, 1), Some(p(1, 0))), Some(p(0, 2)));
    // On water only the water's monster guards.
    assert_eq!(search.guarding(p(2, 2), None), Some(p(2, 1)));
    let (plain, _) = state
        .prepare(&rules.0, &rules.1, &[], &start, &NoAsks)
        .unwrap();
    assert_eq!(plain.guarding(p(1, 1), None), None);
}

// covers: CPU-PTH-004
#[test]
fn from_mode_4_the_guards_lower_the_key_once_each_or_refuse() {
    let rules = rules();
    let rows = ["#M###", "ggggg", "###M#"];
    let state = alone(&rules, &rows, p(0, 1), vec![]);
    let asks = Asks {
        answers: vec![(p(1, 0), -10), (p(3, 2), -20)],
        ..Asks::default()
    };
    let found = search_with(
        &rules,
        &state,
        &[],
        &start(p(0, 1), SearchMode::Plan),
        &asks,
    );
    let at = |x, y| found.node(p(x, y)).map(|node| (node.key, node.guard));
    // The first monster weighed at its tile and beside it, the second beside both.
    assert_eq!(at(1, 0), Some((-10, Some(p(1, 0)))));
    assert_eq!(at(1, 1), Some((-10, Some(p(1, 0)))));
    assert_eq!(at(2, 1), Some((-30, Some(p(3, 2)))));
    assert_eq!(at(3, 2), Some((-30, Some(p(3, 2)))));
    assert_eq!(at(4, 1), Some((-30, Some(p(3, 2)))));
    // At -500 000 000 neither the monster nor a tile it guards is a step.
    let asks = Asks {
        answers: vec![(p(1, 0), REFUSED)],
        ..Asks::default()
    };
    let found = search_with(
        &rules,
        &state,
        &[],
        &start(p(0, 1), SearchMode::Plan),
        &asks,
    );
    assert_eq!(found.node(p(1, 0)), None);
    assert_eq!(found.node(p(1, 1)), None);
    // A flyer passes over the refused one's guarded tile.
    let heroes = store(&[(FLYING, 1)]);
    let found = search_with(
        &rules,
        &state,
        &heroes,
        &start(p(0, 1), SearchMode::Plan),
        &asks,
    );
    assert_eq!(found.node(p(1, 1)), None);
    assert!(found.pass_node(p(1, 1)).is_some());
    // A chart's guard scan leaves out the row above it.
    let rows = ["##M##", "ggCgg", "#####"];
    let state = alone(&rules, &rows, p(1, 1), vec![]);
    let asks = Asks {
        answers: vec![(p(2, 0), -10)],
        ..Asks::default()
    };
    let found = search_with(
        &rules,
        &state,
        &[],
        &start(p(1, 1), SearchMode::Plan),
        &asks,
    );
    assert_eq!(found.node(p(2, 1)).map(|node| node.key), Some(0));
    assert_eq!(
        found.node(p(3, 1)).map(|node| (node.key, node.guard)),
        Some((-10, Some(p(2, 0))))
    );
}

// covers: CPU-PTH-004
#[test]
fn from_mode_4_a_step_onto_a_hero_or_a_garrison_is_weighed() {
    let rules = rules();
    let owned = corridor('g');
    let rows = rows(&owned);
    let state = game(
        &rules,
        &rows,
        vec![
            hero_at(1, 0, p(0, 1), &[PIKEMAN]),
            hero_at(2, 1, p(2, 1), &[PIKEMAN]),
        ],
        vec![garrison(&rules, "t:garrison", p(4, 1), Some(BLUE))],
    );
    let asks = Asks {
        answers: vec![(p(2, 1), -50), (p(4, 1), REFUSED)],
        ..Asks::default()
    };
    let found = search_with(
        &rules,
        &state,
        &[],
        &start(p(0, 1), SearchMode::Plan),
        &asks,
    );
    assert_eq!(
        found.node(p(2, 1)).map(|node| (node.key, node.guard)),
        Some((-50, Some(p(2, 1))))
    );
    assert_eq!(found.node(p(3, 1)).map(|node| node.key), Some(-50));
    assert_eq!(found.node(p(4, 1)), None);
}

// covers: CPU-PTH-003
#[test]
fn from_a_pass_node_no_stop_onto_a_neighbour_s_object_or_a_town_but_the_flying_copy() {
    let rules = rules();
    let heroes = store(&[(FLYING, 1)]);
    let rows = ["#####", "gBg##", "#####"];
    let reach = |object: ObjectSetup| {
        let state = alone(&rules, &rows, p(0, 1), vec![object]);
        search_with(
            &rules,
            &state,
            &heroes,
            &start(p(0, 1), SearchMode::Local),
            &NoAsks,
        )
    };
    let at = p(2, 1);
    let found = reach(monster(at));
    assert_eq!(found.node(at), None);
    let copy = found.pass_node(at).unwrap();
    assert_eq!((copy.marks & (STOP | PLACE), copy.cost), (0, 240));
    let found = reach(town(&rules, at, None));
    assert_eq!(found.node(at), None);
    assert!(found.pass_node(at).is_some());
    let tower = world(&["O"], 1, vec![]).map.objects.remove(0);
    let found = reach(ObjectSetup {
        anchor: at,
        ..tower
    });
    assert_eq!(found.node(at).map(|node| node.marks & STOP), Some(STOP));
    assert_eq!(found.pass_node(at).map(|node| node.marks & STOP), Some(0));
}

// covers: CPU-PTH-001
#[test]
fn a_flyer_on_a_tile_it_may_not_stop_on_starts_from_a_pass_node() {
    let rules = rules();
    let heroes = store(&[(FLYING, 1)]);
    let rows = ["gB~", "gPg"];
    let from = |at: Pos, heroes: &[Arc<HeroDetails>]| {
        let mut state = alone(&rules, &rows, p(0, 0), vec![]);
        state.hero_mut(HERO).unwrap().pos = at;
        let found = search_with(
            &rules,
            &state,
            heroes,
            &start(at, SearchMode::Local),
            &NoAsks,
        );
        (found.node(at).map(|node| node.from), found.pass_node(at))
    };
    // The start a pass node; the pile's place only reached later, from a neighbour.
    for at in [p(1, 0), p(2, 0), p(1, 1)] {
        let (place, pass) = from(at, &heroes);
        assert!(place != Some(at), "{at:?}");
        let pass = pass.unwrap();
        assert_eq!(
            (pass.marks, pass.from),
            (REACHED | FLY | 2 << DIRECTION_SHIFT, at)
        );
    }
    assert_eq!(from(p(1, 1), &heroes).0, Some(p(2, 1)));
    assert_eq!(from(p(0, 0), &heroes).0, Some(p(0, 0)));
    // A walker's start is a place wherever it is.
    assert_eq!(from(p(1, 0), &[]).0, Some(p(1, 0)));
    // A flyer in a boat never flies from it, nor is its place tested.
    let mut state = alone(&rules, &["gb"], p(0, 0), vec![]);
    state.put_aboard(HERO, 0);
    let start = SearchStart {
        boat: true,
        ..start(p(1, 0), SearchMode::Local)
    };
    let found = search_with(&rules, &state, &heroes, &start, &NoAsks);
    assert_eq!(
        found.node(p(1, 0)).map(|node| node.marks),
        Some(placed(2) | BOAT)
    );
}

// covers: CPU-PTH-002
#[test]
fn the_box_holds_the_search() {
    let rules = rules();
    let rows = ["ggggg"; 5];
    let state = alone(&rules, &rows, p(2, 2), vec![]);
    let boxed = |window| {
        let start = SearchStart {
            window: Some(window),
            ..start(p(2, 2), SearchMode::Local)
        };
        search(&rules, &state, &start)
    };
    let found = boxed(SearchWindow {
        left: 1,
        top: 1,
        right: 4,
        bottom: 4,
    });
    assert_eq!(found.reached().count(), 9);
    for outside in [p(0, 0), p(4, 4), p(0, 2), p(2, 4)] {
        assert_eq!(found.node(outside), None);
    }
    let whole = boxed(SearchWindow {
        left: 0,
        top: 0,
        right: 99,
        bottom: 99,
    });
    assert_eq!(whole.reached().count(), 25);
    let away = boxed(SearchWindow {
        left: 3,
        top: 3,
        right: 5,
        bottom: 5,
    });
    assert_eq!(away.reached().count(), 0);
}

// covers: CPU-PTH-002, CPU-PTH-003
#[test]
fn mode_4_prices_by_the_jumps_levels_and_marks_the_tiles_without_magic() {
    let rules = rules();
    let heroes = store(&[(FLYING, 1)]);
    let owned = corridor('B');
    let rows = rows(&owned);
    let state = alone(&rules, &rows, p(1, 1), vec![]);
    let plan = |jumps| {
        let start = SearchStart {
            jumps,
            ..start(p(1, 1), SearchMode::Plan)
        };
        search_with(&rules, &state, &heroes, &start, &NoAsks)
    };
    let fast = SearchJumps {
        fly_level: Some(3),
        ..SearchJumps::default()
    };
    assert_eq!(plan(fast).node(p(3, 1)).map(|node| node.cost), Some(200));
    let slow = search_with(
        &rules,
        &state,
        &heroes,
        &start(p(1, 1), SearchMode::Local),
        &NoAsks,
    );
    assert_eq!(slow.node(p(3, 1)).map(|node| node.cost), Some(240));
    // Cursed Ground and an anti-magic garrison's entrance, with a jump in mode 4.
    let rows = ["gggg"];
    let objects = vec![
        lying(BARRENS, p(1, 0), 1),
        garrison(&rules, "t:anti_garrison", p(3, 0), Some(RED)),
    ];
    let state = alone(&rules, &rows, p(0, 0), objects);
    let marked = |mode, jumps| {
        let start = SearchStart {
            jumps,
            ..start(p(0, 0), mode)
        };
        let found = search(&rules, &state, &start);
        [0, 1, 2, 3].map(|x| found.node(p(x, 0)).map(|node| node.marks & NO_MAGIC))
    };
    let walk = SearchJumps {
        water_walk: true,
        ..SearchJumps::default()
    };
    let none = Some(0);
    assert_eq!(
        marked(SearchMode::Plan, walk),
        [none, Some(NO_MAGIC), none, Some(NO_MAGIC)]
    );
    assert_eq!(marked(SearchMode::Replan, walk), [none; 4]);
    assert_eq!(marked(SearchMode::Plan, SearchJumps::default()), [none; 4]);
}

/// Five grass tiles over five more, rock between: a one-way entrance of channel 0 on (2, 0),
/// its exits on (1, 2) and (3, 2), and `objects`.
fn channel(rules: &Rules, heroes: Vec<HeroSetup>, objects: Vec<ObjectSetup>) -> AdventureState {
    let rows = ["ggggg", "#####", "ggggg", "####g"];
    let tiles = [
        ("t:entrance", p(2, 0)),
        ("t:exit", p(1, 2)),
        ("t:exit", p(3, 2)),
    ];
    let teleporters = tiles.map(|(k, at)| teleporter(rules, k, at));
    let heroes = [vec![hero_at(1, 0, p(0, 0), &[PIKEMAN])], heroes].concat();
    game(
        rules,
        &rows,
        heroes,
        [teleporters.to_vec(), objects].concat(),
    )
}

// covers: CPU-PTH-006
#[test]
fn from_mode_3_a_one_way_entrance_relaxes_its_channel_s_exits_in_order() {
    let rules = rules();
    let state = channel(&rules, vec![], vec![]);
    // The entrance a stop at 200; its two exits a copy each at no price, the second cost 100
    // more for the list's second tile, the entrance's direction, marks and points.
    let exit = |x| SearchNode {
        pos: p(x, 2),
        marks: placed(2),
        from: p(2, 0),
        guard: None,
        key: 0,
        danger: 0,
        cost: 200,
        cost2: 300,
        points: 800,
    };
    for mode in [SearchMode::Threat, SearchMode::Plan, SearchMode::Replan] {
        let found = search(&rules, &state, &start(p(0, 0), mode));
        let entrance = found.node(p(2, 0)).unwrap();
        assert_eq!((entrance.marks, entrance.cost2), (placed(2) | STOP, 200));
        assert_eq!(found.node(p(1, 2)), Some(exit(1)), "{mode:?}");
        assert_eq!(found.node(p(3, 2)), Some(exit(3)), "{mode:?}");
        let order: Vec<Pos> = found.reached().take(5).map(|node| node.pos).collect();
        assert_eq!(order, [p(0, 0), p(1, 0), p(2, 0), p(1, 2), p(3, 2)]);
        // The list's first exit is expanded first (an equal entry goes before the old one).
        assert_eq!(found.node(p(2, 2)).map(|node| node.from), Some(p(1, 2)));
    }
    for mode in [SearchMode::Local, SearchMode::Claims] {
        let found = search(&rules, &state, &start(p(0, 0), mode));
        assert!(found.node(p(2, 0)).is_some());
        assert_eq!(found.node(p(1, 2)), None, "{mode:?}");
    }
    // An entrance whose channel has no exit leads nowhere.
    let lone = vec![teleporter(&rules, "t:entrance_1", p(2, 0))];
    let state = alone(&rules, &["ggggg", "#####", "ggggg"], p(0, 0), lone);
    let found = search(&rules, &state, &start(p(0, 0), SearchMode::Plan));
    assert!(found.node(p(2, 0)).is_some());
    assert_eq!(found.reached().count(), 3);
}

// covers: CPU-PTH-006
#[test]
fn from_mode_4_the_exits_guards_lower_every_exit_s_key_or_drop_the_channel() {
    // A monster on (4, 3) guards the second exit, (3, 2).
    let rules = rules();
    let state = channel(&rules, vec![], vec![monster(p(4, 3))]);
    let with = |mode, answer| {
        let asks = Asks {
            answers: vec![(p(4, 3), answer)],
            ..Asks::default()
        };
        let found = search_with(&rules, &state, &[], &start(p(0, 0), mode), &asks);
        let at = |x| {
            found
                .node(p(x, 2))
                .map(|node| (node.key, node.guard, node.cost2))
        };
        (at(1), at(3))
    };
    // The lowest answer, -40, on both exits; the guard the exit's, else the entrance's (none).
    for mode in [SearchMode::Plan, SearchMode::Replan] {
        assert_eq!(
            with(mode, -40),
            (Some((-40, None, 300)), Some((-40, Some(p(4, 3)), 300)))
        );
        assert_eq!(
            with(mode, 25),
            (Some((0, None, 300)), Some((0, Some(p(4, 3)), 300)))
        );
        // A fight lost at -500 000 000 drops the whole channel.
        assert_eq!(with(mode, REFUSED), (None, None));
    }
    // Mode 3 asks nothing: the entrance's key and guard.
    assert_eq!(
        with(SearchMode::Threat, REFUSED),
        (Some((0, None, 300)), Some((0, None, 300)))
    );
    // Heroes on both exits, neither weighed: another team's an exit and a stop, a team-mate's
    // none (and the guard refuses its tile to the walk).
    let heroes = vec![
        hero_at(2, 1, p(1, 2), &[PIKEMAN]),
        hero_at(3, 0, p(3, 2), &[PIKEMAN]),
    ];
    let state = channel(&rules, heroes, vec![monster(p(4, 3))]);
    let asks = Asks {
        answers: vec![(p(4, 3), REFUSED)],
        ..Asks::default()
    };
    let found = search_with(
        &rules,
        &state,
        &[],
        &start(p(0, 0), SearchMode::Plan),
        &asks,
    );
    assert_eq!(
        found
            .node(p(1, 2))
            .map(|node| (node.marks, node.key, node.cost)),
        Some((placed(2) | STOP, 0, 200))
    );
    assert_eq!(found.node(p(3, 2)), None);
}

// covers: CPU-PTH-006
#[test]
fn a_guard_at_a_teleporter_s_entrance_stops_its_passage_in_mode_3_and_is_weighed_from_mode_4() {
    // A monster on (3, 0) guards the entrance (2, 0), not the tile before it.
    let rules = rules();
    let rows = ["gggg", "####", "gggg"];
    let objects = vec![
        teleporter(&rules, "t:entrance", p(2, 0)),
        teleporter(&rules, "t:exit", p(1, 2)),
        monster(p(3, 0)),
    ];
    let state = alone(&rules, &rows, p(0, 0), objects);
    let threat = search(&rules, &state, &start(p(0, 0), SearchMode::Threat));
    assert!(threat.node(p(2, 0)).is_some());
    assert_eq!(threat.node(p(1, 2)), None);
    // From mode 4 the step onto the entrance weighed its guard; the passage leaves it out.
    let asks = Asks {
        answers: vec![(p(3, 0), -25)],
        ..Asks::default()
    };
    let plan = search_with(
        &rules,
        &state,
        &[],
        &start(p(0, 0), SearchMode::Plan),
        &asks,
    );
    assert_eq!(
        plan.node(p(1, 2))
            .map(|node| (node.key, node.guard, node.cost)),
        Some((-25, Some(p(3, 0)), 200))
    );
    // On the start, at no cost, the guard is never asked.
    let mut state = state;
    state.hero_mut(HERO).unwrap().pos = p(2, 0);
    let found = search(&rules, &state, &start(p(2, 0), SearchMode::Threat));
    assert!(found.node(p(1, 2)).is_some());
}

// covers: CPU-PTH-006
#[test]
fn a_two_way_monolith_leads_to_its_other_members_skipping_a_team_mate_s_hero() {
    // Channel 1's members (2, 0), (1, 2), (3, 2), (5, 2) in the walk's order; red's hero 3 on
    // (3, 2), blue's hero 2 on (5, 2); a monster beside (1, 2).
    let rules = rules();
    let rows = ["ggggggg", "#######", "ggggggg", "g######"];
    let members = [p(2, 0), p(1, 2), p(3, 2), p(5, 2)];
    let mut objects: Vec<ObjectSetup> = members
        .iter()
        .map(|&at| teleporter(&rules, "t:two_way", at))
        .collect();
    objects.push(monster(p(0, 3)));
    let heroes = vec![
        hero_at(1, 0, p(0, 0), &[PIKEMAN]),
        hero_at(2, 1, p(5, 2), &[PIKEMAN]),
        hero_at(3, 0, p(3, 2), &[PIKEMAN]),
    ];
    let state = game(&rules, &rows, heroes, objects);
    let asks = Asks {
        answers: vec![(p(0, 3), -60)],
        ..Asks::default()
    };
    for mode in [SearchMode::Threat, SearchMode::Plan] {
        let found = search_with(&rules, &state, &[], &start(p(0, 0), mode), &asks);
        // Four members: the second cost 300 more; no guard weighed for a two-way monolith.
        let member = found.node(p(1, 2)).unwrap();
        assert_eq!(
            (
                member.marks,
                member.cost,
                member.cost2,
                member.key,
                member.guard
            ),
            (placed(2), 200, 500, 0, None)
        );
        // The team-mate's tile only walked onto.
        assert_eq!(found.node(p(3, 2)).map(|node| node.cost), Some(400));
        // Another team's hero is an exit and a stop, gone past from mode 3.
        let hero = found.node(p(5, 2)).unwrap();
        assert_eq!((hero.marks, hero.cost2), (placed(2) | STOP, 500));
        assert!(found.node(p(6, 2)).is_some());
        // Never the entrance itself again: its place is the walk's.
        assert_eq!(found.node(p(2, 0)).map(|node| node.from), Some(p(1, 0)));
    }
}

/// A sea row with rock between its Whirlpools over a row of rock between a coast and the sea,
/// Red's hero aboard on (0, 0): the first Whirlpool on (1, 0) and (1, 1), the others on (3, 0),
/// (5, 0) and (7, 0), `objects` and a boat on `row`'s `b`.
fn whirlpools(rules: &Rules, row: &str, objects: Vec<ObjectSetup>) -> AdventureState {
    let first = ObjectSetup {
        blocked: TileMask::from_bits(1 | 1 << 8),
        visitable: TileMask::from_bits(1 | 1 << 8),
        ..teleporter(rules, "t:whirlpool", p(1, 1))
    };
    let others = [3, 5, 7].map(|x| teleporter(rules, "t:whirlpool", p(x, 0)));
    let objects = [vec![first], others.to_vec(), objects].concat();
    let mut state = alone(rules, &[row, "g~#####~"], p(0, 1), objects);
    state.put_aboard(HERO, 0);
    state
}

// covers: CPU-PTH-006
#[test]
fn a_whirlpool_leads_to_the_other_whirlpools_tiles_each_exit_s_key_500_lower() {
    let rules = rules();
    let afloat = |mode| SearchStart {
        boat: true,
        ..start(p(0, 0), mode)
    };
    // Five tiles in the list: the second cost 64 more; the own Whirlpool's tiles excluded.
    let state = whirlpools(&rules, "b~#~#~#~", vec![]);
    let found = search(&rules, &state, &afloat(SearchMode::Threat));
    let exit = |x| SearchNode {
        pos: p(x, 0),
        marks: placed(2) | BOAT,
        from: p(1, 0),
        guard: None,
        key: 0,
        danger: 0,
        cost: 100,
        cost2: 164,
        points: 900,
    };
    assert_eq!(
        found.node(p(3, 0)),
        Some(SearchNode {
            key: -500,
            ..exit(3)
        })
    );
    assert_eq!(
        found.node(p(5, 0)),
        Some(SearchNode {
            key: -1000,
            ..exit(5)
        })
    );
    assert_eq!(
        found.node(p(7, 0)),
        Some(SearchNode {
            key: -1500,
            ..exit(7)
        })
    );
    assert_eq!(found.node(p(1, 1)), None);
    // A monster on the water beside the last guards it alone.
    let state = whirlpools(&rules, "b~#~#~#~", vec![monster(p(7, 1))]);
    let with = |mode, answer| {
        let asks = Asks {
            answers: vec![(p(7, 1), answer)],
            ..Asks::default()
        };
        let found = search_with(&rules, &state, &[], &afloat(mode), &asks);
        let at = |x| found.node(p(x, 0)).map(|node| (node.key, node.guard));
        (at(3), at(5), at(7))
    };
    assert_eq!(
        with(SearchMode::Threat, -70),
        (Some((-500, None)), Some((-1000, None)), Some((-1500, None)))
    );
    assert_eq!(
        with(SearchMode::Plan, -70),
        (
            Some((-570, None)),
            Some((-1070, None)),
            Some((-1570, Some(p(7, 1))))
        )
    );
    assert_eq!(with(SearchMode::Plan, REFUSED), (None, None, None));
    // Never from the start: a boat on a Whirlpool goes no further at once.
    let mut state = whirlpools(&rules, "b~#~#~#~", vec![]);
    state.hero_mut(HERO).unwrap().pos = p(3, 0);
    let start = SearchStart {
        from: p(3, 0),
        ..afloat(SearchMode::Threat)
    };
    let found = search(&rules, &state, &start);
    assert_eq!((found.node(p(5, 0)), found.reached().count()), (None, 1));
}

// covers: CPU-PTH-006
#[test]
fn a_subterranean_gate_leads_to_its_pair_its_costs_and_key_kept() {
    // Gates on (2, 0) of both levels; a monster on (3, 1) guards the upper one.
    let rules = rules();
    let rows = ["gggg", "gggg", "gggg", "gggg"];
    let gates = |others: Vec<ObjectSetup>| {
        let pair = [Pos::new(2, 0, 0), Pos::new(2, 0, 1)];
        let gates = pair.map(|at| teleporter(&rules, "t:underground_gate", at));
        [gates.to_vec(), others].concat()
    };
    let below = |x, y| Pos::new(x, y, 1);
    let hero = || hero_at(1, 0, p(0, 0), &[PIKEMAN]);
    let state = deep(&rules, &rows, vec![hero()], gates(vec![]));
    let found = search(&rules, &state, &start(p(0, 0), SearchMode::Threat));
    let pair = SearchNode {
        pos: below(2, 0),
        marks: placed(2),
        from: p(2, 0),
        guard: None,
        key: 0,
        danger: 0,
        cost: 200,
        cost2: 200,
        points: 800,
    };
    assert_eq!(found.node(below(2, 0)), Some(pair));
    assert_eq!(
        search(&rules, &state, &start(p(0, 0), SearchMode::Claims)).node(below(2, 0)),
        None
    );
    // A hero on the pair: a stop, gone past for another team's only.
    for (owner, past) in [(1, true), (0, false)] {
        let heroes = vec![hero(), hero_at(2, owner, below(2, 0), &[PIKEMAN])];
        let state = deep(&rules, &rows, heroes, gates(vec![]));
        let found = search(&rules, &state, &start(p(0, 0), SearchMode::Threat));
        assert_eq!(
            found.node(below(2, 0)).map(|node| node.marks),
            Some(placed(2) | STOP)
        );
        assert_eq!(found.node(below(1, 0)).is_some(), past);
    }
    // Guarded: mode 3 stops at the gate; from mode 4 the pair keeps the key and guard weighed.
    let state = deep(&rules, &rows, vec![hero()], gates(vec![monster(p(3, 1))]));
    let threat = search(&rules, &state, &start(p(0, 0), SearchMode::Threat));
    assert!(threat.node(p(2, 0)).is_some());
    assert_eq!(threat.node(below(2, 0)), None);
    let asks = Asks {
        answers: vec![(p(3, 1), -20)],
        ..Asks::default()
    };
    let plan = search_with(
        &rules,
        &state,
        &[],
        &start(p(0, 0), SearchMode::Plan),
        &asks,
    );
    assert_eq!(
        plan.node(below(2, 0))
            .map(|node| (node.key, node.guard, node.cost2)),
        Some((-20, Some(p(3, 1)), 200))
    );
    // A gate without a pair leads nowhere.
    let lone = vec![teleporter(&rules, "t:underground_gate", p(2, 0))];
    let state = deep(&rules, &rows, vec![hero()], lone);
    let found = search(&rules, &state, &start(p(0, 0), SearchMode::Threat));
    assert!(found.node(p(2, 0)).is_some());
    assert_eq!(found.node(below(2, 0)), None);
}

// covers: CPU-PTH-001, CPU-PTH-006
#[test]
fn from_mode_3_the_start_on_a_monolith_or_a_gate_goes_through_it_at_once() {
    let rules = rules();
    // The hero on the entrance (0, 0): its exits relaxed before the loop, the visit mark set,
    // the second cost 50 more than the list's.
    let rows = ["ggg", "###", "ggg"];
    let objects = vec![
        teleporter(&rules, "t:entrance", p(0, 0)),
        teleporter(&rules, "t:exit", p(0, 2)),
        teleporter(&rules, "t:exit", p(2, 2)),
    ];
    let state = alone(&rules, &rows, p(0, 0), objects);
    let found = search(&rules, &state, &start(p(0, 0), SearchMode::Threat));
    let exit = SearchNode {
        pos: p(0, 2),
        marks: placed(2) | VISIT_HERE,
        from: p(0, 0),
        guard: None,
        key: 0,
        danger: 0,
        cost: 0,
        cost2: 150,
        points: 1000,
    };
    assert_eq!(found.node(p(0, 2)), Some(exit));
    let order: Vec<Pos> = found.reached().take(3).map(|node| node.pos).collect();
    assert_eq!(order, [p(0, 0), p(0, 2), p(2, 2)]);
    // The start node itself is no stop and steps on.
    assert_eq!(found.node(p(0, 0)).map(|node| node.marks), Some(placed(2)));
    assert!(found.node(p(1, 0)).is_some());
    assert_eq!(
        search(&rules, &state, &start(p(0, 0), SearchMode::Claims)).node(p(0, 2)),
        None
    );
    // A two-way monolith, the gate's pair alike.
    let state = deep(
        &rules,
        &["ggg", "ggg"],
        vec![hero_at(1, 0, p(1, 0), &[PIKEMAN])],
        [Pos::new(1, 0, 0), Pos::new(1, 0, 1)]
            .map(|at| teleporter(&rules, "t:underground_gate", at))
            .to_vec(),
    );
    let found = search(&rules, &state, &start(p(1, 0), SearchMode::Threat));
    assert_eq!(
        found
            .node(Pos::new(1, 0, 1))
            .map(|node| (node.marks, node.cost2)),
        Some((placed(2) | VISIT_HERE, 50))
    );
    let rows = ["g#g"];
    let objects = [p(0, 0), p(2, 0)].map(|at| teleporter(&rules, "t:two_way", at));
    let state = alone(&rules, &rows, p(0, 0), objects.to_vec());
    let found = search(&rules, &state, &start(p(0, 0), SearchMode::Threat));
    assert_eq!(
        found.node(p(2, 0)).map(|node| (node.marks, node.cost2)),
        Some((placed(2) | VISIT_HERE, 150))
    );
}

// covers: CPU-PTH-007
#[test]
fn a_door_leg_starts_at_rock_crosses_it_at_no_price_and_lands_on_a_place() {
    let rules = rules();
    let state = alone(&rules, &["g#gg"], p(0, 0), vec![]);
    let plan = |jumps| {
        jumping(
            &rules,
            &state,
            &[],
            (p(0, 0), SearchMode::Plan, jumps),
            &NoAsks,
        )
    };
    let found = plan(door());
    // The leg's start: 300 points, the second cost 100 for the rock and 500 for the jump, the
    // points the step's (rock's: 0).
    let leg = SearchNode {
        pos: p(1, 0),
        marks: passed(2) | DOOR | offset(1, 0),
        from: p(0, 0),
        guard: None,
        key: 0,
        danger: 0,
        cost: 300,
        cost2: 900,
        points: 1000,
    };
    assert_eq!(found.pass_node(p(1, 0)), Some(leg));
    // Its landing, a place, the leg's start its source, its offset 2.
    let landed = SearchNode {
        pos: p(2, 0),
        marks: placed(2) | DOOR | offset(2, 0),
        cost2: 1000,
        ..leg
    };
    assert_eq!(found.node(p(2, 0)), Some(landed));
    // The landing's own steps are no leg's.
    let on = found.node(p(3, 0)).unwrap();
    assert_eq!((on.marks, on.cost, on.from), (placed(2), 400, p(2, 0)));
    // At Expert 200; without the spell, in mode 5 or from Cursed Ground, no leg.
    let expert = SearchJumps {
        door_price: 200,
        ..door()
    };
    assert_eq!(plan(expert).node(p(2, 0)).map(|node| node.cost), Some(200));
    assert_eq!(plan(SearchJumps::default()).node(p(2, 0)), None);
    let replan = jumping(
        &rules,
        &state,
        &[],
        (p(0, 0), SearchMode::Replan, door()),
        &NoAsks,
    );
    assert_eq!(replan.node(p(2, 0)), None);
    let cursed = vec![lying(BARRENS, p(0, 0), 1)];
    let state = alone(&rules, &["g#gg"], p(0, 0), cursed);
    let found = jumping(
        &rules,
        &state,
        &[],
        (p(0, 0), SearchMode::Plan, door()),
        &NoAsks,
    );
    assert_eq!(
        found.node(p(0, 0)).map(|node| node.marks & NO_MAGIC),
        Some(NO_MAGIC)
    );
    assert_eq!(found.pass_node(p(1, 0)), None);
}

// covers: CPU-PTH-007
#[test]
fn a_door_leg_goes_9_columns_and_8_rows_from_its_start_at_most() {
    let rules = rules();
    let reach = |rows: &[&str], from: Pos, to: Pos| {
        let state = alone(&rules, rows, from, vec![]);
        let found = jumping(
            &rules,
            &state,
            &[],
            (from, SearchMode::Plan, door()),
            &NoAsks,
        );
        found
            .node(to)
            .map(|node| (node.marks & DOOR_OFFSET_MASK, node.from))
    };
    let row = |rock: usize| std::format!("g{}g", "#".repeat(rock));
    assert_eq!(
        reach(&[row(8).as_str()], p(0, 0), p(9, 0)),
        Some((offset(9, 0), p(0, 0)))
    );
    assert_eq!(reach(&[row(9).as_str()], p(0, 0), p(10, 0)), None);
    // Westwards and northwards the offsets are two's complement.
    assert_eq!(
        reach(&[row(8).as_str()], p(9, 0), p(0, 0)),
        Some((offset(-9, 0), p(9, 0)))
    );
    let column = |rock: usize| [vec!["g"], vec!["#"; rock], vec!["g"]].concat();
    assert_eq!(
        reach(&column(7), p(0, 8), p(0, 0)),
        Some((offset(0, -8), p(0, 8)))
    );
    assert_eq!(reach(&column(8), p(0, 9), p(0, 0)), None);
}

// covers: CPU-PTH-007
#[test]
fn a_door_leg_lands_on_no_entrance_and_crosses_water_as_a_jump() {
    let rules = rules();
    // Onto a tower's tile (entered) the leg only passes, no stop; beyond it it lands (the
    // tower's place is walked onto from there).
    let state = alone(&rules, &["g#Og"], p(0, 0), vec![]);
    let found = jumping(
        &rules,
        &state,
        &[],
        (p(0, 0), SearchMode::Plan, door()),
        &NoAsks,
    );
    assert_eq!(found.node(p(2, 0)).map(|node| node.from), Some(p(3, 0)));
    let over = found.pass_node(p(2, 0)).unwrap();
    assert_eq!(over.marks, passed(2) | DOOR | offset(2, 0));
    assert_eq!(
        found.node(p(3, 0)).map(|node| (node.marks, node.cost2)),
        Some((placed(2) | DOOR | offset(3, 0), 1100))
    );
    // From a blocked tile, a leg rather than a flight; its points the step's (100 paid).
    let jumps = SearchJumps {
        fly: true,
        fly_level: Some(1),
        ..door()
    };
    let state = alone(&rules, &["gBg"], p(0, 0), vec![]);
    let found = jumping(
        &rules,
        &state,
        &[],
        (p(0, 0), SearchMode::Plan, jumps),
        &NoAsks,
    );
    let leg = found.pass_node(p(1, 0)).unwrap();
    assert_eq!(
        (leg.marks, leg.cost, leg.cost2, leg.points),
        (passed(2) | DOOR | offset(1, 0), 300, 800, 900)
    );
    // At sea from land: a leg before the spells; over water a jump's only, landing beyond.
    let state = alone(&rules, &["g~~g"], p(0, 0), vec![]);
    let found = jumping(
        &rules,
        &state,
        &[],
        (p(0, 0), SearchMode::Plan, jumps),
        &NoAsks,
    );
    let leg = found.pass_node(p(1, 0)).unwrap();
    assert_eq!(
        (leg.marks, leg.cost, leg.cost2, leg.points),
        (passed(2) | DOOR | offset(1, 0), 300, 800, 900)
    );
    assert_eq!(
        found.pass_node(p(2, 0)).map(|node| node.marks),
        Some(passed(2) | DOOR | offset(2, 0))
    );
    assert_eq!(
        found
            .node(p(3, 0))
            .map(|node| (node.marks, node.cost, node.cost2)),
        Some((placed(2) | DOOR | offset(3, 0), 300, 1000))
    );
}

// covers: CPU-PTH-007
#[test]
fn a_door_leg_from_a_boat_lands_on_water_only() {
    // From the boat on (0, 0) onto (1, 0), which the human fog leaves only flown over: the leg;
    // on the water beyond it lands in the boat; onto the coast from there only a new leg, never
    // a landing.
    let rules = rules();
    let mut state = alone(&rules, &["b~~g", "###g"], p(3, 1), vec![]);
    state.put_aboard(HERO, 0);
    state.explored_mut(RED).0.set(p(1, 0), false);
    let start = SearchStart {
        boat: true,
        fog: SearchFog::Human,
        jumps: door(),
        ..start(p(0, 0), SearchMode::Plan)
    };
    let found = search(&rules, &state, &start);
    assert_eq!(
        found.pass_node(p(1, 0)).map(|node| node.marks),
        Some(passed(2) | BOAT | DOOR | offset(1, 0))
    );
    let landed = found.node(p(2, 0)).unwrap();
    assert_eq!(
        (landed.marks, landed.cost),
        (placed(2) | BOAT | DOOR | offset(2, 0), 300)
    );
    assert_eq!(found.node(p(3, 0)), None);
    assert_eq!(
        found
            .pass_node(p(3, 0))
            .map(|node| (node.marks, node.from, node.cost)),
        Some((passed(2) | BOAT | DOOR | offset(1, 0), p(2, 0), 600))
    );
}

// covers: CPU-PTH-007
#[test]
fn a_fly_stretch_starts_at_a_blocked_tile_with_the_day_s_maximum_less_its_price() {
    let rules = rules();
    let fly = SearchJumps {
        fly: true,
        fly_level: Some(1),
        ..SearchJumps::default()
    };
    let state = alone(&rules, &["gBgg"], p(0, 0), vec![]);
    let found = jumping(
        &rules,
        &state,
        &[],
        (p(0, 0), SearchMode::Plan, fly),
        &NoAsks,
    );
    // From grass the lesser of 100 and the fly price 140; 1 500 less it.
    let over = found.pass_node(p(1, 0)).unwrap();
    assert_eq!(
        (over.marks, over.cost, over.cost2, over.points),
        (passed(2) | FLY, 100, 600, 1400)
    );
    // From the blocked tile the fly price.
    let beyond = found.node(p(2, 0)).unwrap();
    assert_eq!(
        (beyond.marks, beyond.cost, beyond.cost2, beyond.points),
        (REACHED | PLACE | FLY | 2 << DIRECTION_SHIFT, 240, 740, 1260)
    );
    // Not in mode 5, not from Cursed Ground.
    let replan = jumping(
        &rules,
        &state,
        &[],
        (p(0, 0), SearchMode::Replan, fly),
        &NoAsks,
    );
    assert_eq!(replan.pass_node(p(1, 0)), None);
    // From a boat no stretch (the human fog: the water ahead only flown over), but a leg.
    let mut state = alone(&rules, &["b~~", "##g"], p(2, 1), vec![]);
    state.put_aboard(HERO, 0);
    state.explored_mut(RED).0.set(p(1, 0), false);
    let afloat = |jumps| {
        let start = SearchStart {
            boat: true,
            fog: SearchFog::Human,
            jumps,
            ..start(p(0, 0), SearchMode::Plan)
        };
        search(&rules, &state, &start)
    };
    assert_eq!(afloat(fly).pass_node(p(1, 0)), None);
    assert_eq!(
        afloat(door()).pass_node(p(1, 0)).map(|node| node.marks),
        Some(passed(2) | BOAT | DOOR | offset(1, 0))
    );
}

// covers: CPU-PTH-007
#[test]
fn a_sea_spell_is_fly_or_water_walk_by_the_higher_level_with_500_unless_worn() {
    let rules = rules();
    let state = alone(&rules, &["gg~~"], p(1, 0), vec![]);
    let spell = |heroes: &[Arc<HeroDetails>], jumps, points| {
        let start = SearchStart {
            jumps,
            points,
            ..start(p(1, 0), SearchMode::Plan)
        };
        let found = search_with(&rules, &state, heroes, &start, &NoAsks);
        found
            .pass_node(p(2, 0))
            .map(|node| (node.marks & (FLY | WATER_WALK), node.cost2, node.points))
    };
    let jumps = |fly, water_walk, levels: (Option<u8>, Option<u8>)| SearchJumps {
        fly,
        water_walk,
        fly_level: levels.0,
        water_walk_level: levels.1,
        ..SearchJumps::default()
    };
    // Priced from grass at 100; the points the day's maximum less it.
    assert_eq!(
        spell(&[], jumps(true, false, (Some(1), None)), 1000),
        Some((FLY, 600, 1400))
    );
    assert_eq!(
        spell(&[], jumps(false, true, (None, Some(1))), 1000),
        Some((WATER_WALK, 600, 1400))
    );
    // Equal levels: Water Walk; the higher fly level: Fly, though only Water Walk is castable.
    assert_eq!(
        spell(&[], jumps(true, false, (Some(1), Some(1))), 1000),
        Some((WATER_WALK, 600, 1400))
    );
    assert_eq!(
        spell(&[], jumps(false, true, (Some(2), Some(1))), 1000),
        Some((FLY, 600, 1400))
    );
    assert_eq!(spell(&[], SearchJumps::default(), 1000), None);
    // A hero flying or water walking on its own, its lift lost at a new day before the water:
    // no surcharge.
    let wings = store(&[(FLYING, 1)]);
    assert_eq!(
        spell(&wings, jumps(true, false, (Some(1), None)), 50),
        Some((FLY, 100, 1400))
    );
    let boots = store(&[(WATER_WALKING, 1)]);
    assert_eq!(
        spell(&boots, jumps(false, true, (None, Some(1))), 50),
        Some((WATER_WALK, 100, 1400))
    );
    // None from Cursed Ground.
    let cursed = alone(&rules, &["gg~~"], p(1, 0), vec![lying(BARRENS, p(1, 0), 1)]);
    let start = SearchStart {
        jumps: jumps(true, true, (Some(1), Some(1))),
        ..start(p(1, 0), SearchMode::Plan)
    };
    assert_eq!(search(&rules, &cursed, &start).pass_node(p(2, 0)), None);
}

// covers: CPU-PTH-007
#[test]
fn a_boat_node_is_summoned_or_bought_at_a_mark_from_mode_4_with_the_day_s_rest() {
    let rules = rules();
    let state = alone(&rules, &["g~~"], p(0, 0), vec![]);
    let summon = SearchJumps {
        summon_boat: true,
        ..SearchJumps::default()
    };
    let marked = Asks {
        boats: vec![(p(1, 0), -700)],
        ..Asks::default()
    };
    let boat = |mode, jumps, asks: &Asks, points| {
        let start = SearchStart {
            jumps,
            points,
            ..start(p(0, 0), mode)
        };
        let found = search_with(&rules, &state, &[], &start, asks);
        let at = |x| {
            found
                .node(p(x, 0))
                .map(|n| (n.marks, n.key, n.cost, n.cost2, n.points))
        };
        (at(1), at(2))
    };
    let none = Asks::default();
    // Summoned: the day's 1 000 points its price, the second cost 500 more, the sea's maximum.
    let sea = placed(2) | BOAT;
    assert_eq!(
        boat(SearchMode::Plan, summon, &none, 1000),
        (
            Some((sea, 0, 1000, 1500, 1500)),
            Some((sea, 0, 1100, 1600, 1400))
        )
    );
    // Points short of the step: those and the land's maximum.
    assert_eq!(
        boat(SearchMode::Plan, summon, &none, 50).0,
        Some((sea, 0, 1550, 2050, 1500))
    );
    // At a mark, bought: the purchase off the key; mode 5 alike, its summons ignored.
    let bought = Some((sea, -700, 1000, 1500, 1500));
    assert_eq!(
        boat(SearchMode::Plan, SearchJumps::default(), &marked, 1000).0,
        bought
    );
    assert_eq!(boat(SearchMode::Replan, summon, &marked, 1000).0, bought);
    assert_eq!(boat(SearchMode::Replan, summon, &none, 1000).0, None);
    // Below mode 4 the bare price and the step's points.
    assert_eq!(
        boat(SearchMode::Local, summon, &marked, 1000).0,
        Some((sea, -700, 100, 600, 900))
    );
    assert_eq!(boat(SearchMode::Local, summon, &none, 1000).0, None);
    // From Cursed Ground no summons, but a mark still.
    let cursed = alone(&rules, &["g~~"], p(0, 0), vec![lying(BARRENS, p(0, 0), 1)]);
    let at = |asks: &Asks| {
        let start = SearchStart {
            jumps: summon,
            ..start(p(0, 0), SearchMode::Plan)
        };
        let found = search_with(&rules, &cursed, &[], &start, asks);
        found.node(p(1, 0)).map(|node| node.key)
    };
    assert_eq!(at(&none), None);
    assert_eq!(at(&marked), Some(-700));
}

// covers: CPU-PTH-005, CPU-PTH-007
#[test]
fn from_mode_4_a_landing_and_a_boarding_take_the_day_s_rest() {
    let rules = rules();
    let mut state = alone(&rules, &["b~g", "##g"], p(2, 1), vec![]);
    state.put_aboard(HERO, 0);
    let land = |mode, points| {
        let start = SearchStart {
            boat: true,
            points,
            ..start(p(0, 0), mode)
        };
        let found = search(&rules, &state, &start);
        found
            .node(p(2, 0))
            .map(|node| (node.marks, node.cost, node.cost2, node.points))
    };
    // The 900 points left the landing's price; the land's maximum after it.
    for mode in [SearchMode::Plan, SearchMode::Replan] {
        assert_eq!(land(mode, 1000), Some((placed(2), 1000, 1000, 1500)));
        // Short of the step: those 50 and the sea's maximum.
        assert_eq!(land(mode, 150), Some((placed(2), 1650, 1650, 1500)));
    }
    // Below mode 4 a step kept in the boat.
    assert_eq!(
        land(SearchMode::Local, 1000),
        Some((placed(2) | BOAT, 200, 200, 800))
    );
    // The boarding of an empty boat (CPU-PTH-005) in mode 5 as in mode 4.
    let state = alone(&rules, &["gb~"], p(0, 0), vec![]);
    let found = search(&rules, &state, &start(p(0, 0), SearchMode::Replan));
    assert_eq!(
        found
            .node(p(2, 0))
            .map(|node| (node.marks, node.cost, node.points)),
        Some((placed(2) | BOAT, 1100, 1400))
    );
}
