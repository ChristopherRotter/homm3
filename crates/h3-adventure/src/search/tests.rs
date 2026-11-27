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
use crate::ruleset::{BankDef, BankLevelDef, BankQuestion, ObjectKindDef, ObjectTypeDef};
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
        },
    )
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

/// The caller's numbers by tile: the danger and the fight answers, 0 elsewhere.
#[derive(Default)]
struct Asks {
    danger: Vec<(Pos, i32)>,
    answers: Vec<(Pos, i32)>,
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
