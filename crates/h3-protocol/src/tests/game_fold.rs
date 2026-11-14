//! The game fold over the golden replays (E16 M5 T3c1, T3c2, T5c; SRV-Q26, SRV-Q27): each
//! viewer's view, folded with its projected events after every command and then with what it
//! newly sees when its explored tiles grew (`h3_game::reveal` against its grid as it was last
//! told it, as the session sends it), against `observe`, part by part. A battle's snapshot
//! replaces the view's battle when it opens (SRV-Q27 (1)); after each command the view is
//! `observe`'s again, so each difference is that command's own. The gate: a part differs only
//! where a stub of its own lets it (`PARTS`), and a stubbed part still differs somewhere.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::string::{String, ToString};
use std::vec;
use std::vec::Vec;
use std::{eprintln, format, fs};

use h3_contracts::PlayerId;
use h3_game::{BitGrid, GameRuleset, GameSetup, GameState, Viewer, apply};
use h3_sim::adventure::replay::Replay;

use crate::map::game::{events, observe, reveal};
use crate::wire::WireU64;
use crate::wire::game::*;

/// A part of a view: its name, the stubs that let the fold differ from `observe` on it (none: the
/// events carry it whole), and whether two views agree on it.
struct Part {
    name: &'static str,
    stubs: &'static [&'static str],
    agree: fn(&GameObservation, &GameObservation) -> bool,
}

/// The entries of `a` and `b` with one key, in `b`'s order: a part compares what both views hold,
/// which entries they hold being another part.
fn pairs<'a, T, K: PartialEq>(
    a: &'a [T],
    b: &'a [T],
    key: impl Fn(&T) -> K + Copy + 'a,
) -> impl Iterator<Item = (&'a T, &'a T)> + 'a {
    b.iter()
        .filter_map(move |y| a.iter().find(|x| key(x) == key(y)).map(|x| (x, y)))
}

/// Whether the viewer's own heroes agree on what `f` reads of them.
fn own<T: PartialEq>(
    a: &GameObservation,
    b: &GameObservation,
    f: impl Fn(&OwnMapHero) -> T + Copy,
) -> bool {
    pairs(&a.adventure.heroes, &b.adventure.heroes, |h| h.id)
        .all(|(x, y)| x.own.as_ref().map(f) == y.own.as_ref().map(f))
}

/// Whether the objects both views see agree on what `f` reads of them.
fn objects<T: PartialEq>(
    a: &GameObservation,
    b: &GameObservation,
    f: impl Fn(&Object) -> T + Copy,
) -> bool {
    pairs(&a.adventure.objects, &b.adventure.objects, |o| {
        (o.object_type, o.anchor)
    })
    .all(|(x, y)| f(x) == f(y))
}

/// Whether the towns both views see agree on what `f` reads of them.
fn towns<T: PartialEq>(
    a: &GameObservation,
    b: &GameObservation,
    f: impl Fn(&Town) -> T + Copy,
) -> bool {
    pairs(&a.towns, &b.towns, |t| t.town).all(|(x, y)| f(x) == f(y))
}

/// The town lane's stub of a Visions holder's reach (E09 T18b2a): a viewer's level of another
/// team's town moves with no `TownChanged`, its garrison and its owner facts with it.
const VISIONS: &str = "TWN-STUB-13";

/// Whether a decision's view is one no event carries (SRV-STUB-5): an offer's, a choice's, a
/// University's or a hire dialog's.
fn untold(decision: Option<&Decision>) -> bool {
    matches!(
        decision,
        Some(
            Decision::Offer { .. }
                | Decision::Choice { .. }
                | Decision::University { .. }
                | Decision::Hire { .. }
        )
    )
}

/// The hero's details but for the parts of their own (spell points, artifacts, offered levels).
fn rest(details: &HeroDetails) -> HeroDetails {
    HeroDetails {
        offered: OfferLevels {
            wisdom: 0,
            school: 0,
        },
        worn: Vec::new(),
        backpack: Vec::new(),
        mana: 0,
        ..details.clone()
    }
}

/// Whether the details of the heroes both views hold agree on what `f` reads of them.
fn details<T: PartialEq>(
    a: &GameObservation,
    b: &GameObservation,
    f: impl Fn(&HeroDetails) -> T + Copy,
) -> bool {
    pairs(&a.heroes, &b.heroes, |h| h.id)
        .all(|(x, y)| x.details.as_ref().map(f) == y.details.as_ref().map(f))
}

/// The parts, every field of the view in one of them, each with the stubs (`docs/stubs.toml`) of
/// the gaps that let the fold differ there (SRV-Q26, SRV-Q27 (2)): the gate (E16 M5 T3c2).
const PARTS: &[Part] = &[
    Part {
        name: "day, week and month",
        stubs: &[],
        agree: |a, b| (a.day, a.week, a.month) == (b.day, b.week, b.month),
    },
    Part {
        name: "turns",
        stubs: &[],
        agree: |a, b| (&a.players, &a.turn_group) == (&b.players, &b.turn_group),
    },
    Part {
        name: "treasury",
        stubs: &[],
        agree: |a, b| a.treasury == b.treasury,
    },
    Part {
        name: "timed events",
        stubs: &[],
        agree: |a, b| a.events_today == b.events_today,
    },
    Part {
        name: "decision",
        stubs: &[],
        agree: |a, b| {
            untold(a.decision.as_ref()) || untold(b.decision.as_ref()) || a.decision == b.decision
        },
    },
    Part {
        name: "decision views",
        stubs: &["SRV-STUB-5"],
        agree: |a, b| {
            !(untold(a.decision.as_ref()) || untold(b.decision.as_ref()))
                || a.decision == b.decision
        },
    },
    Part {
        name: "battle open",
        stubs: &[],
        agree: |a, b| a.battle.is_some() == b.battle.is_some(),
    },
    Part {
        name: "heroes in sight",
        stubs: &["SRV-STUB-3"],
        agree: |a, b| {
            let ids = |o: &GameObservation| {
                let map: Vec<_> = o.adventure.heroes.iter().map(|h| (h.id, h.owner)).collect();
                let info: Vec<_> = o.heroes.iter().map(|h| (h.id, h.hero_type)).collect();
                (map, info)
            };
            ids(a) == ids(b)
        },
    },
    Part {
        name: "hero tiles and boats",
        stubs: &[],
        agree: |a, b| {
            pairs(&a.adventure.heroes, &b.adventure.heroes, |h| h.id)
                .all(|(x, y)| (x.pos, x.boat) == (y.pos, y.boat))
        },
    },
    Part {
        name: "hero details",
        stubs: &[],
        agree: |a, b| details(a, b, rest),
    },
    Part {
        name: "spell points",
        stubs: &["SRV-STUB-6"],
        agree: |a, b| details(a, b, |d| d.mana),
    },
    Part {
        name: "artifacts",
        stubs: &["SRV-STUB-7"],
        agree: |a, b| details(a, b, |d| (d.worn.clone(), d.backpack.clone())),
    },
    Part {
        name: "offered levels",
        stubs: &["SRV-STUB-8"],
        agree: |a, b| details(a, b, |d| d.offered),
    },
    Part {
        name: "armies",
        stubs: &["SRV-STUB-9"],
        agree: |a, b| own(a, b, |o| o.army.clone()),
    },
    Part {
        name: "movement",
        stubs: &[],
        agree: |a, b| own(a, b, |o| (o.movement, o.max_movement)),
    },
    Part {
        name: "formation and tactics",
        stubs: &[],
        agree: |a, b| own(a, b, |o| (o.formation, o.tactics)),
    },
    // E08 T115a (v57): the hero on the map's battle bonuses and timed marks.
    Part {
        name: "battle bonuses and timed marks",
        stubs: &[],
        agree: |a, b| own(a, b, |o| (o.bonuses.clone(), o.timed.clone())),
    },
    Part {
        name: "once-marks",
        stubs: &["SRV-STUB-4"],
        agree: |a, b| own(a, b, |o| o.marks.clone()),
    },
    Part {
        name: "explored tiles",
        stubs: &[],
        agree: |a, b| {
            let planes =
                |o: &GameObservation| (o.adventure.explored.clone(), o.adventure.fringe.clone());
            planes(a) == planes(b)
        },
    },
    Part {
        name: "tile views",
        stubs: &[],
        agree: |a, b| {
            (&a.adventure.palette, &a.adventure.tiles) == (&b.adventure.palette, &b.adventure.tiles)
        },
    },
    Part {
        name: "objects in sight",
        stubs: &[],
        agree: |a, b| {
            let ids = |o: &GameObservation| {
                o.adventure
                    .objects
                    .iter()
                    .map(|o| {
                        (
                            o.object_type,
                            o.anchor,
                            o.blocked,
                            o.visitable,
                            o.drawn,
                            o.town,
                        )
                    })
                    .collect::<Vec<_>>()
            };
            ids(a) == ids(b)
        },
    },
    Part {
        name: "monsters",
        stubs: &[],
        agree: |a, b| objects(a, b, |o| o.monster),
    },
    Part {
        name: "object owners",
        stubs: &[],
        agree: |a, b| objects(a, b, |o| o.owner),
    },
    Part {
        name: "mine production",
        stubs: &["SRV-STUB-10"],
        agree: |a, b| objects(a, b, |o| o.production),
    },
    Part {
        name: "mine guards",
        stubs: &[],
        agree: |a, b| objects(a, b, |o| o.guards.clone()),
    },
    Part {
        name: "visited marks",
        stubs: &[],
        agree: |a, b| objects(a, b, |o| o.visited),
    },
    Part {
        name: "shipyard tiles",
        stubs: &["SRV-STUB-10"],
        agree: |a, b| objects(a, b, |o| o.boat_tile),
    },
    Part {
        name: "known objects",
        stubs: &["SRV-STUB-4"],
        agree: |a, b| a.adventure.known == b.adventure.known,
    },
    Part {
        name: "bought",
        stubs: &["SRV-STUB-11"],
        agree: |a, b| a.adventure.bought == b.adventure.bought,
    },
    Part {
        name: "keys",
        stubs: &[],
        agree: |a, b| a.adventure.keys == b.adventure.keys,
    },
    Part {
        name: "empty boats",
        stubs: &[],
        agree: |a, b| a.adventure.boats == b.adventure.boats,
    },
    Part {
        name: "towns in sight",
        stubs: &[],
        agree: |a, b| {
            let ids = |o: &GameObservation| o.towns.iter().map(|t| t.town).collect::<Vec<_>>();
            ids(a) == ids(b)
        },
    },
    Part {
        name: "town owners",
        stubs: &[],
        agree: |a, b| towns(a, b, |t| t.owner),
    },
    Part {
        name: "town pictures",
        stubs: &[],
        agree: |a, b| {
            towns(a, b, |t| {
                (
                    t.anchor,
                    t.faction,
                    t.name.clone(),
                    t.fort,
                    t.capitol,
                    t.built_today,
                )
            })
        },
    },
    Part {
        name: "town buildings",
        stubs: &[],
        agree: |a, b| towns(a, b, |t| t.buildings),
    },
    Part {
        name: "garrisons",
        stubs: &[VISIONS],
        agree: |a, b| towns(a, b, |t| t.garrison.clone()),
    },
    Part {
        name: "town owner facts",
        stubs: &[VISIONS],
        agree: |a, b| towns(a, b, |t| t.owner_facts.clone()),
    },
    // E08 T114b (ADV-GAR-006): no golden replay holds a map garrison yet (increment 12's T116
    // adds them), so neither part can differ; a Visions holder's reach (`TWN-STUB-13`) joins the
    // second once one shows it.
    Part {
        name: "map garrisons in sight",
        stubs: &[],
        agree: |a, b| {
            let places = |o: &GameObservation| {
                o.garrisons
                    .iter()
                    .map(MapGarrison::place)
                    .collect::<Vec<_>>()
            };
            places(a) == places(b)
        },
    },
    Part {
        name: "map garrisons",
        stubs: &[],
        agree: |a, b| pairs(&a.garrisons, &b.garrisons, MapGarrison::place).all(|(x, y)| x == y),
    },
];

/// Where a part differed.
#[derive(Default)]
struct Differences {
    /// The viewer-commands it differed after.
    count: usize,
    /// Those of them whose explored tiles grew (a reveal folded).
    with_reveal: usize,
    /// The replays it differed in.
    replays: BTreeSet<String>,
    /// The first: its replay, command, viewer and the kinds of the events it was told.
    first: Option<String>,
}

/// The workspace's root.
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// An event's kind: its variant's name, an adventure event's within it.
fn kind(event: &GameEvent) -> String {
    let text = match event {
        GameEvent::Adventure { event } => format!("{event:?}"),
        _ => format!("{event:?}"),
    };
    text.split([' ', '{', '('])
        .next()
        .unwrap_or_default()
        .to_string()
}

/// Calls `each` with every golden adventure replay, by name: its name, its rules, its map's setup
/// at its difficulty and the replay (also the game session's test, E16 M5 T5a).
pub(super) fn golden(mut each: impl FnMut(&str, &GameRuleset, &GameSetup, &Replay)) {
    let root = root();
    let dir = root.join("fixtures/replays/adventure");
    let mut files: Vec<PathBuf> = fs::read_dir(&dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|e| e == "toml"))
        .collect();
    files.sort();
    assert_eq!(files.len(), 86, "the golden adventure replays");
    let mut rules: BTreeMap<(Vec<String>, String), GameRuleset> = BTreeMap::new();
    for file in &files {
        let name = file.file_stem().unwrap().to_string_lossy().into_owned();
        let replay = h3_files::adventure::parse_replay(&fs::read_to_string(file).unwrap()).unwrap();
        let ids: Vec<String> = replay
            .header
            .packs
            .iter()
            .map(|(id, _)| id.clone())
            .collect();
        let key = (ids, replay.header.profile.clone());
        let rules = rules.entry(key).or_insert_with_key(|(ids, profile)| {
            h3_files::adventure::load_rules(&root.join("packs"), ids, profile).unwrap()
        });
        let (_, setup) = h3_files::adventure::load_map(&root, &replay.map.path, rules).unwrap();
        let setup = GameSetup {
            difficulty: replay.difficulty,
            ..setup
        };
        each(&name, rules, &setup, &replay);
    }
}

/// The golden adventure replay `name` (E16 M5 T7a1): its rules, its map's setup at its difficulty
/// and the replay.
pub(super) fn one(name: &str) -> (GameRuleset, GameSetup, Replay) {
    let root = root();
    let file = root.join(format!("fixtures/replays/adventure/{name}.toml"));
    let replay = h3_files::adventure::parse_replay(&fs::read_to_string(file).unwrap()).unwrap();
    let ids: Vec<String> = (replay.header.packs.iter())
        .map(|(id, _)| id.clone())
        .collect();
    let rules =
        h3_files::adventure::load_rules(&root.join("packs"), &ids, &replay.header.profile).unwrap();
    let (_, setup) = h3_files::adventure::load_map(&root, &replay.map.path, &rules).unwrap();
    let setup = GameSetup {
        difficulty: replay.difficulty,
        ..setup
    };
    (rules, setup, replay)
}

/// Runs every golden adventure replay and compares each viewer's folded view with `observe`
/// after every command; returns each part's differences and the viewer-commands run.
fn run() -> (BTreeMap<&'static str, Differences>, usize) {
    let mut differences: BTreeMap<&'static str, Differences> = BTreeMap::new();
    let mut runs = 0;
    golden(|name, rules, setup, replay| {
        let name = name.to_string();
        let mut state = GameState::new(rules, setup, replay.seed).unwrap();
        let viewers: Vec<Viewer> = (0..=u8::MAX)
            .take(state.players())
            .map(|p| Viewer::Player(PlayerId::new(p)))
            .collect();
        let mut views: Vec<GameObservation> =
            viewers.iter().map(|&v| observe(&state, rules, v)).collect();
        let explored = |state: &GameState, Viewer::Player(player)| {
            state.adventure().explored(player).unwrap().clone()
        };
        let mut grids: Vec<BitGrid> = viewers.iter().map(|&v| explored(&state, v)).collect();
        for (step, (actor, command)) in replay.commands.iter().enumerate() {
            let mut told = Vec::new();
            apply(&mut state, rules, *actor, command.clone(), &mut told).unwrap();
            for ((&viewer, view), grid) in viewers.iter().zip(&mut views).zip(&mut grids) {
                let told = events(&told, viewer);
                for event in &told {
                    view.fold(event);
                }
                // What it newly sees, after the events (E16 M5 T5c).
                let now = explored(&state, viewer);
                let revealed = now != *grid;
                if revealed {
                    view.reveal(&reveal(&state, rules, viewer, grid));
                    *grid = now;
                }
                let observed = observe(&state, rules, viewer);
                // The battle's snapshot when it opens (SRV-Q27 (1)).
                if told
                    .iter()
                    .any(|e| matches!(e, GameEvent::BattleStarted { .. }))
                {
                    view.battle.clone_from(&observed.battle);
                }
                let mut any = false;
                for &Part {
                    name: part, agree, ..
                } in PARTS
                {
                    if agree(view, &observed) {
                        continue;
                    }
                    any = true;
                    let at = differences.entry(part).or_default();
                    at.count += 1;
                    at.replays.insert(name.clone());
                    at.with_reveal += usize::from(revealed);
                    at.first.get_or_insert_with(|| {
                        let kinds: BTreeSet<String> = told.iter().map(kind).collect();
                        format!("{name} command {step} viewer {}: {kinds:?}", view.viewer)
                    });
                }
                // The views' battles agree between its snapshots only by its own events.
                view.battle.clone_from(&observed.battle);
                assert!(
                    any || *view == observed,
                    "{name} command {step}: a part misses a field"
                );
                *view = observed;
                runs += 1;
            }
        }
    });
    (differences, runs)
}

#[test]
fn the_fold_against_observe_over_the_golden_replays() {
    let (differences, runs) = run();
    // The report: `cargo test -p h3-protocol fold_against -- --nocapture`.
    #[allow(clippy::print_stderr)] // a test's report
    {
        eprintln!("{runs} viewer-commands; the parts that differed:");
        for (part, at) in &differences {
            eprintln!(
                "- {part}: {} viewer-commands ({} with a reveal) in {} replays; first {}",
                at.count,
                at.with_reveal,
                at.replays.len(),
                at.first.as_deref().unwrap_or_default(),
            );
        }
    }
    // The gate (E16 M5 T3c2): a part the events and the reveals carry whole equals `observe` after
    // every command for every viewer; a stubbed part still differs somewhere, else its stubs are
    // lifted from it (and from the registry once no part and no fold names them).
    let registry = fs::read_to_string(root().join("docs/stubs.toml")).unwrap();
    for part in PARTS {
        for stub in part.stubs {
            assert!(
                registry.contains(&format!("\"{stub}\"")),
                "{}: {stub} is not in docs/stubs.toml",
                part.name
            );
        }
        let at = differences.get(part.name);
        match (part.stubs, at) {
            ([], Some(at)) => panic!(
                "{}: the fold differs from `observe` in {} viewer-commands, first {}",
                part.name,
                at.count,
                at.first.as_deref().unwrap_or_default()
            ),
            ([_, ..], None) => panic!(
                "{}: the fold equals `observe` everywhere now: lift {:?} from it",
                part.name, part.stubs
            ),
            _ => {}
        }
    }
}

/// A hand-built view (`super::game::every_game_part`) with its towns numbered 0 to 3.
fn view() -> GameObservation {
    let o = super::game::every_game_part(None, h3_game::MonthKind::Normal);
    let mut view = crate::map::game::observation(&o);
    for (id, town) in (0..).zip(&mut view.towns) {
        town.town = id;
    }
    view
}

/// Folds `events` in order.
fn fold(view: &mut GameObservation, events: &[GameEvent]) {
    for event in events {
        view.fold(event);
    }
}

/// A tile of the map at `x`, `y` on the surface.
const fn at(x: u16, y: u16) -> Pos {
    Pos { x, y, level: 0 }
}

/// A seen tile of the hand-built map with these flags.
const fn seen(blocked: bool, visitable: bool) -> Tile {
    Tile::Seen {
        terrain: 2,
        road: Some(1),
        river: Some(3),
        blocked,
        visitable,
        no_native: true,
        winds: false,
        hole: true,
    }
}

/// An object of no kind but its type, anchor and masks.
const fn object(object_type: u16, anchor: Pos, blocked: u64, visitable: u64) -> Object {
    Object {
        object_type,
        anchor,
        blocked: WireU64(blocked),
        visitable: WireU64(visitable),
        drawn: WireU64(0),
        monster: None,
        owner: None,
        production: None,
        guards: None,
        visited: false,
        boat_tile: None,
        town: None,
    }
}

/// A stack of creature 30.
const fn stack(slot: u8, count: u32) -> ArmyStack {
    ArmyStack {
        slot,
        creature: 30,
        count,
    }
}

/// An adventure event.
const fn adventure(event: AdventureEvent) -> GameEvent {
    GameEvent::Adventure { event }
}

#[test]
fn the_fold_keeps_the_calendar_the_turns_and_the_treasury() {
    let mut v = view();
    fold(
        &mut v,
        &[
            GameEvent::TurnEnded { player: 1 },
            GameEvent::TurnEnded { player: 9 },
            GameEvent::DayStarted { day: 31 },
            GameEvent::WeekStarted {
                week: 5,
                kind: Week::Of {
                    creatures: [Some(42), Some(43)],
                    bonus: 15,
                },
            },
            GameEvent::MonthStarted {
                month: 2,
                kind: Month::Plague {},
            },
            GameEvent::TurnStarted { player: 0 },
            GameEvent::TurnStarted { player: 1 },
            GameEvent::TurnStarted { player: 1 },
            GameEvent::TimedEventFired {
                player: 0,
                event: 4,
            },
            GameEvent::ResourceGained {
                player: 1,
                resource: 6,
                amount: 100,
            },
            GameEvent::ResourceGained {
                player: 1,
                resource: 9,
                amount: 100,
            },
            GameEvent::ResourceLost {
                player: 1,
                resource: 0,
                amount: 30,
            },
            GameEvent::ResourcePaid {
                player: 1,
                resource: 1,
                amount: 2,
            },
            GameEvent::ResourcePaid {
                player: 1,
                resource: 9,
                amount: 2,
            },
        ],
    );
    assert_eq!((v.day, v.month), (31, Month::Plague {}));
    let deity = Week::Of {
        creatures: [Some(42), Some(43)],
        bonus: 15,
    };
    assert_eq!(v.week, deity);
    let acting: Vec<bool> = v.players.iter().map(|p| p.acting).collect();
    assert_eq!(acting, [true, true]);
    assert_eq!(v.turn_group, [0, 1]);
    assert_eq!(v.events_today, [4]);
    assert_eq!(v.treasury, [0, 3, 10, 5, 5, 5, 7600]);
}

#[test]
fn the_fold_leaves_the_view_for_messages_and_what_no_event_carries() {
    let before = view();
    let mut v = before.clone();
    let (ty, a) = (7, at(1, 1));
    let unknown = 77;
    fold(
        &mut v,
        &[
            GameEvent::OfferOpened {
                player: 1,
                hero: 4,
                object_type: ty,
                anchor: a,
            },
            GameEvent::OfferRefused {
                player: 1,
                hero: 4,
                object_type: ty,
                anchor: a,
            },
            GameEvent::ChoiceOpened {
                player: 1,
                hero: 4,
                object_type: ty,
                anchor: a,
            },
            GameEvent::UniversityOpened {
                player: 1,
                hero: 4,
                object_type: ty,
                anchor: a,
            },
            GameEvent::HireOpened {
                player: 1,
                target: HireTarget::Hero { hero: 4 },
                source: HireSeen::Object {
                    object_type: ty,
                    anchor: a,
                },
            },
            GameEvent::BattleStarted {
                heroes: [Some(4), None],
                tile: a,
            },
            GameEvent::Battle {
                event: crate::wire::Event::RoundStarted { round: 2 },
            },
            GameEvent::CreaturesRaised {
                player: 1,
                hero: 4,
                creature: 3,
                count: 2,
            },
            GameEvent::GroupLost {
                player: 1,
                hero: 4,
                stack: stack(0, 2),
            },
            GameEvent::DecisionClosed { player: 1 },
            GameEvent::TownBuilt {
                player: 1,
                town: 1,
                building: 4,
                event: None,
            },
            GameEvent::TownEventFired {
                player: 1,
                town: 1,
                event: 0,
            },
            // Heroes without details or own view, an unknown one, a town not in sight.
            GameEvent::HeroChanged {
                player: 1,
                hero: 9,
                change: HeroChange::SetSpellPoints { points: 1 },
            },
            GameEvent::LevelGained {
                player: 1,
                hero: unknown,
                level: 2,
                primary: PrimarySkill::Attack,
                skill: None,
            },
            GameEvent::FormationSet {
                player: 1,
                hero: 9,
                formation: Formation::Loose,
            },
            GameEvent::TacticsSet {
                player: 1,
                hero: unknown,
                on: false,
            },
            GameEvent::StackChanged {
                player: 1,
                at: ArmySlot {
                    army: ArmyRef::Hero { hero: 9 },
                    slot: 0,
                },
                stack: None,
            },
            GameEvent::StackChanged {
                player: 1,
                at: ArmySlot {
                    army: ArmyRef::Group {},
                    slot: 0,
                },
                stack: None,
            },
            GameEvent::StackChanged {
                player: 1,
                at: ArmySlot {
                    army: ArmyRef::Garrison { town: 0 },
                    slot: 0,
                },
                stack: None,
            },
            GameEvent::StackChanged {
                player: 1,
                at: ArmySlot {
                    army: ArmyRef::Garrison { town: 8 },
                    slot: 0,
                },
                stack: None,
            },
            town_changed(8, Some(1)),
            GameEvent::TownStackSeen {
                player: 1,
                town: 8,
                stack: shown(3, Some(5)),
                slot: Some(0),
            },
            GameEvent::TownIncomeSeen {
                player: 1,
                town: 8,
                resource: 6,
                amount: 1000,
            },
            adventure(AdventureEvent::TilesRevealed {
                object_type: ty,
                count: 3,
            }),
            adventure(AdventureEvent::CreaturesJoined {
                hero: 4,
                creature: 3,
                count: 2,
            }),
            adventure(AdventureEvent::EventFired { hero: 4, tile: a }),
            adventure(AdventureEvent::MessageRead {
                hero: 4,
                object_type: ty,
                anchor: a,
            }),
            adventure(AdventureEvent::ObjectEmpty {
                object_type: ty,
                anchor: a,
            }),
            adventure(AdventureEvent::ShipyardBlocked {
                object_type: ty,
                anchor: a,
                reason: ShipyardBlock::Taken,
            }),
            adventure(AdventureEvent::NoneToRecruit {
                object_type: ty,
                anchor: a,
                creature: 3,
            }),
            adventure(AdventureEvent::JoinRefused {
                hero: 4,
                creature: 3,
                count: 2,
            }),
            adventure(AdventureEvent::LeadsNowhere {
                hero: 4,
                object_type: ty,
                anchor: a,
            }),
            adventure(AdventureEvent::BoatRemoved {
                hero: 4,
                boat: 1,
                at: a,
            }),
            // A hero coming into sight is not carried (SRV-Q26 (c)).
            adventure(AdventureEvent::HeroStepped {
                hero: unknown,
                from: None,
                to: Some(a),
            }),
            // Objects not in sight.
            adventure(AdventureEvent::ObjectRemoved {
                object_type: ty,
                anchor: a,
            }),
            adventure(AdventureEvent::MonsterChanged {
                object_type: ty,
                anchor: a,
                count_word: 1,
            }),
            adventure(AdventureEvent::ObjectFlagged {
                object_type: ty,
                anchor: a,
                owner: 0,
            }),
            adventure(AdventureEvent::GuardsChanged {
                object_type: ty,
                anchor: a,
                creature: None,
                count_word: None,
            }),
            adventure(AdventureEvent::ObjectMarked {
                object_type: ty,
                anchor: a,
            }),
            adventure(AdventureEvent::TownCaptured {
                town: 8,
                anchor: a,
                owner: 0,
            }),
            adventure(AdventureEvent::KeyGained { key: 9 }),
            adventure(AdventureEvent::WhirlpoolLoss {
                hero: 9,
                slot: 0,
                creature: 3,
                count: 1,
            }),
            // A hero the view does not hold as its own (SRV-Q26 (b)).
            adventure(AdventureEvent::MovementChanged {
                hero: 9,
                movement: 100,
                max_movement: 1_500,
            }),
            adventure(AdventureEvent::WhirlpoolLoss {
                hero: 4,
                slot: 7,
                creature: 3,
                count: 1,
            }),
        ],
    );
    assert_eq!(v, before);
}

#[test]
fn the_fold_opens_and_closes_the_decisions_whose_events_hold_their_view() {
    let mut v = view();
    let group = |slot| ArmySlot {
        army: ArmyRef::Group {},
        slot,
    };
    fold(
        &mut v,
        &[GameEvent::MeetingOpened {
            player: 1,
            heroes: [4, 5],
        }],
    );
    assert_eq!(v.decision, Some(Decision::Meeting { heroes: [4, 5] }));
    fold(
        &mut v,
        &[GameEvent::OfferAnswered {
            player: 1,
            object_type: 7,
            anchor: at(1, 1),
            accepted: true,
        }],
    );
    assert_eq!(v.decision, None);
    let level_up = LevelUp {
        hero: 4,
        level: 7,
        primary: PrimarySkill::Power,
        offers: [7, 99],
    };
    fold(
        &mut v,
        &[
            GameEvent::LevelUpOpened {
                player: 1,
                hero: 4,
                level: 7,
                primary: PrimarySkill::Power,
                offers: [7, 99],
            },
            // Another hero's level: the dialog stays open.
            GameEvent::LevelGained {
                player: 1,
                hero: 9,
                level: 3,
                primary: PrimarySkill::Attack,
                skill: None,
            },
        ],
    );
    assert_eq!(v.decision, Some(Decision::LevelUp { level_up }));
    let gained = |level, primary, skill| GameEvent::LevelGained {
        player: 1,
        hero: 4,
        level,
        primary,
        skill,
    };
    fold(
        &mut v,
        &[
            gained(7, PrimarySkill::Power, Some(7)),
            gained(8, PrimarySkill::Attack, Some(99)),
            gained(9, PrimarySkill::Defense, None),
            gained(10, PrimarySkill::Knowledge, None),
        ],
    );
    assert_eq!(v.decision, None);
    let details = v.heroes[0].details.clone().unwrap();
    assert_eq!((details.level, details.primary), (10, [3, 2, 4, 5]));
    let skills: Vec<(u16, u8)> = details.skills.iter().map(|s| (s.skill, s.level)).collect();
    assert_eq!(skills, [(7, 3), (21, 1), (99, 1)]);
    // An army dialog: its stacks by slot, the group managed, a second hero's opening anew.
    fold(
        &mut v,
        &[
            GameEvent::ArmyDialogOpened {
                player: 1,
                hero: 4,
                stack: stack(3, 5),
            },
            GameEvent::ArmyDialogOpened {
                player: 1,
                hero: 4,
                stack: stack(1, 2),
            },
            GameEvent::StackChanged {
                player: 1,
                at: group(0),
                stack: Some(stack(9, 4)),
            },
            GameEvent::StackChanged {
                player: 1,
                at: group(3),
                stack: None,
            },
        ],
    );
    let dialog = |hero, group| {
        Some(Decision::ArmyDialog {
            hero,
            group,
            mine: None,
        })
    };
    assert_eq!(v.decision, dialog(4, vec![stack(0, 4), stack(1, 2)]));
    fold(
        &mut v,
        &[GameEvent::ArmyDialogOpened {
            player: 1,
            hero: 5,
            stack: stack(0, 1),
        }],
    );
    assert_eq!(v.decision, dialog(5, vec![stack(0, 1)]));
    fold(&mut v, &[GameEvent::DecisionClosed { player: 1 }]);
    assert_eq!(v.decision, None);
    // Leave Guards dialogs: a mine with guards, one without, one left without, one not in
    // sight.
    let guarded = Object {
        guards: Some(Guards {
            creature: Some(30),
            count_word: 1,
            army: Some(vec![stack(0, 9)]),
        }),
        ..object(60, at(2, 0), 0, 0)
    };
    v.adventure.objects = vec![
        guarded,
        object(61, at(1, 0), 0, 0),
        object(63, at(0, 0), 0, 0),
    ];
    let leave = |object_type, anchor| GameEvent::LeaveGuardsOpened {
        player: 1,
        hero: 4,
        object_type,
        anchor,
    };
    fold(&mut v, &[leave(60, at(2, 0))]);
    let mine = ObjectRef {
        object_type: 60,
        anchor: at(2, 0),
    };
    assert_eq!(
        v.decision,
        Some(Decision::ArmyDialog {
            hero: 4,
            group: Vec::new(),
            mine: Some(mine),
        })
    );
    fold(
        &mut v,
        &[
            GameEvent::ArmyDialogOpened {
                player: 1,
                hero: 4,
                stack: stack(0, 9),
            },
            GameEvent::StackChanged {
                player: 1,
                at: group(0),
                stack: None,
            },
            GameEvent::DecisionClosed { player: 1 },
            leave(61, at(1, 0)),
            GameEvent::StackChanged {
                player: 1,
                at: group(2),
                stack: Some(stack(2, 5)),
            },
            GameEvent::DecisionClosed { player: 1 },
            leave(63, at(0, 0)),
            GameEvent::DecisionClosed { player: 1 },
            leave(62, at(0, 0)),
            GameEvent::DecisionClosed { player: 1 },
        ],
    );
    assert_eq!(v.decision, None);
    let guards: Vec<Option<Guards>> = v
        .adventure
        .objects
        .iter()
        .map(|o| o.guards.clone())
        .collect();
    let left = |creature, count_word, army| {
        Some(Guards {
            creature,
            count_word,
            army: Some(army),
        })
    };
    assert_eq!(
        guards,
        [
            left(Some(30), 1, Vec::new()),
            left(None, 0, vec![stack(2, 5)]),
            None
        ]
    );
    // E08 T114a2: a map garrison's exchange opens empty, its stacks told after it and by its
    // commands (a slot of another garrison changes nothing), and closes leaving nothing behind.
    let garrison = |index, slot| ArmySlot {
        army: ArmyRef::MapGarrison { garrison: index },
        slot,
    };
    fold(
        &mut v,
        &[
            GameEvent::ExchangeOpened {
                player: 1,
                hero: 4,
                garrison: 17,
                object_type: 33,
                anchor: at(1, 0),
                removable: false,
            },
            GameEvent::StackChanged {
                player: 1,
                at: garrison(17, 3),
                stack: Some(stack(3, 8)),
            },
            GameEvent::StackChanged {
                player: 1,
                at: garrison(17, 0),
                stack: Some(stack(0, 2)),
            },
            GameEvent::StackChanged {
                player: 1,
                at: garrison(18, 1),
                stack: Some(stack(1, 2)),
            },
        ],
    );
    assert_eq!(
        v.decision,
        Some(Decision::Exchange {
            hero: 4,
            garrison: 17,
            object: ObjectRef {
                object_type: 33,
                anchor: at(1, 0),
            },
            removable: false,
            army: vec![stack(0, 2), stack(3, 8)],
        })
    );
    fold(&mut v, &[GameEvent::DecisionClosed { player: 1 }]);
    assert_eq!(v.decision, None);
    // A battle ends with its result; its snapshot is the session's.
    assert!(v.battle.is_some());
    fold(
        &mut v,
        &[GameEvent::BattleEnded {
            heroes: [Some(4), None],
            result: crate::wire::Outcome::Draw {},
        }],
    );
    assert_eq!(v.battle, None);
}

#[test]
fn the_fold_applies_a_heros_changes() {
    let mut v = view();
    let artifact = |artifact| Artifact {
        artifact,
        spell: None,
    };
    let change = |change| GameEvent::HeroChanged {
        player: 1,
        hero: 4,
        change,
    };
    let worn = |position| ArtifactPlace::Worn { position };
    let carried = |index| ArtifactPlace::Backpack { index };
    let offered = OfferLevels {
        wisdom: 7,
        school: 7,
    };
    fold(
        &mut v,
        &[
            change(HeroChange::GainExperience { amount: 500 }),
            change(HeroChange::SetSpellPoints { points: 12 }),
            // Where it lands is the rules' (SRV-Q27 (2)).
            change(HeroChange::AddArtifact {
                artifact: artifact(70),
            }),
            change(HeroChange::RemoveArtifact { place: worn(1) }),
            change(HeroChange::RemoveArtifact { place: worn(9) }),
            change(HeroChange::RemoveArtifact { place: carried(0) }),
            change(HeroChange::RemoveArtifact { place: carried(3) }),
            change(HeroChange::PlaceArtifact {
                artifact: artifact(60),
                at: worn(0),
            }),
            change(HeroChange::PlaceArtifact {
                artifact: artifact(61),
                at: worn(9),
            }),
            change(HeroChange::PlaceArtifact {
                artifact: artifact(62),
                at: carried(5),
            }),
            change(HeroChange::PlaceArtifact {
                artifact: artifact(63),
                at: carried(0),
            }),
            change(HeroChange::LearnSpell { spell: 20 }),
            change(HeroChange::GainLevel {
                primary: PrimarySkill::Defense,
                learned: Some(21),
                offered,
            }),
            change(HeroChange::GainLevel {
                primary: PrimarySkill::Attack,
                learned: Some(30),
                offered,
            }),
            change(HeroChange::GainPrimary {
                primary: [1, 0, 0, 252],
            }),
            change(HeroChange::RaiseSkill { skill: 7, level: 1 }),
            change(HeroChange::RaiseSkill { skill: 7, level: 3 }),
            change(HeroChange::RaiseSkill {
                skill: 40,
                level: 2,
            }),
        ],
    );
    let d = v.heroes[0].details.clone().unwrap();
    assert_eq!((d.level, d.experience, d.mana), (8, 5000, 12));
    assert_eq!((d.offered, d.primary), (offered, [4, 2, 3, 255]));
    let skills: Vec<(u16, u8)> = d.skills.iter().map(|s| (s.skill, s.level)).collect();
    assert_eq!(skills, [(7, 3), (21, 2), (30, 1), (40, 2)]);
    let scroll = Artifact {
        artifact: 1,
        spell: Some(15),
    };
    assert_eq!(d.worn, [Some(artifact(60)), None, Some(scroll)]);
    assert_eq!(d.backpack, [artifact(63), artifact(62)]);
    assert_eq!(d.spells, [10, 12, 20]);
}

#[test]
fn the_fold_keeps_the_own_heroes_battle_bonuses_and_timed_marks() {
    // E08 T115a (v57): hero 4's own view holds −3 morale and +2 luck, the once-marks 0 (−3
    // morale) and 13, the timed marks 48 and 94 (`every_game_part`'s).
    let mut v = view();
    let hero = 4;
    let bonus = |mark, morale, luck| VisitBonus { mark, morale, luck };
    let once = |b, held| {
        adventure(AdventureEvent::OnceMarkChanged {
            hero,
            bonus: b,
            held,
        })
    };
    let timed = |object_type, held| {
        adventure(AdventureEvent::TimedMarkChanged {
            hero,
            object_type,
            held,
        })
    };
    let own = |v: &GameObservation| {
        let seen = v.adventure.heroes.iter().find(|h| h.id == hero).unwrap();
        seen.own.clone().unwrap()
    };
    assert_eq!(own(&v).bonuses.once, [bonus(0, -3, 0), bonus(13, 0, 0)]);
    fold(
        &mut v,
        &[
            once(bonus(0, -3, 0), false),
            timed(48, false),
            adventure(AdventureEvent::BonusChanged {
                hero,
                morale: 1,
                luck: 0,
            }),
            once(bonus(2, 1, 0), true),
            // Held already, or not held: nothing.
            once(bonus(13, 0, 0), true),
            once(bonus(5, 0, 3), false),
            timed(49, true),
            timed(94, true),
            timed(50, false),
            // A hero the view does not hold as its own.
            adventure(AdventureEvent::BonusChanged {
                hero: 9,
                morale: 5,
                luck: 5,
            }),
            adventure(AdventureEvent::OnceMarkChanged {
                hero: 9,
                bonus: bonus(1, 0, 1),
                held: true,
            }),
            adventure(AdventureEvent::TimedMarkChanged {
                hero: 9,
                object_type: 1,
                held: true,
            }),
        ],
    );
    let own = own(&v);
    assert_eq!(
        own.bonuses,
        Bonuses {
            morale: 1,
            luck: 0,
            once: vec![bonus(2, 1, 0), bonus(13, 0, 0)],
        }
    );
    assert_eq!(own.timed, [49, 94]);
}

#[test]
fn the_fold_keeps_the_own_heroes_armies_and_the_towns() {
    let mut v = view();
    let slot = |army, slot| ArmySlot { army, slot };
    let hero = ArmyRef::Hero { hero: 4 };
    let garrison = ArmyRef::Garrison { town: 1 };
    let changed = |at, stack| GameEvent::StackChanged {
        player: 1,
        at,
        stack,
    };
    let loss = |count| {
        adventure(AdventureEvent::WhirlpoolLoss {
            hero: 4,
            slot: 2,
            creature: 12,
            count,
        })
    };
    fold(
        &mut v,
        &[
            GameEvent::FormationSet {
                player: 1,
                hero: 4,
                formation: Formation::Loose,
            },
            GameEvent::TacticsSet {
                player: 1,
                hero: 4,
                on: true,
            },
            changed(slot(hero, 1), None),
            changed(slot(hero, 5), Some(stack(9, 3))),
            changed(slot(hero, 0), Some(stack(0, 2))),
            loss(5),
        ],
    );
    let own = v.adventure.heroes[0].own.clone().unwrap();
    assert_eq!((own.formation, own.tactics), (Formation::Loose, true));
    let wounded = ArmyStack {
        slot: 2,
        creature: 12,
        count: 6,
    };
    assert_eq!(own.army, [stack(0, 2), wounded, stack(5, 3)]);
    fold(
        &mut v,
        &[
            loss(6),
            changed(slot(garrison, 0), None),
            changed(slot(garrison, 3), Some(stack(3, 8))),
        ],
    );
    let own = v.adventure.heroes[0].own.clone().unwrap();
    assert_eq!(own.army, [stack(0, 2), stack(5, 3)]);
    assert_eq!(v.towns[1].garrison.army, Some(vec![stack(3, 8)]));
}

#[test]
fn the_fold_keeps_the_bonus_visitables_army_changes_and_the_kind_marks() {
    // E08 T115b (v61): the Sirens' loss as the Whirlpool's, the Stables' upgrade of a slot's
    // creature, the team's kind marks in type order; another player's hero and a held mark
    // change nothing; a visit that gave nothing is a message.
    let mut v = view();
    let before = v.adventure.heroes[0].own.clone().unwrap().army;
    let i = before.iter().position(|stack| stack.count > 1).unwrap();
    let at = before[i].slot;
    let kinds = v.adventure.kinds.clone();
    fold(
        &mut v,
        &[
            adventure(AdventureEvent::SirensLoss {
                hero: 4,
                slot: at,
                creature: before[i].creature,
                count: 1,
            }),
            adventure(AdventureEvent::CreaturesUpgraded {
                hero: 4,
                slot: at,
                from: before[i].creature,
                to: 77,
            }),
            adventure(AdventureEvent::CreaturesUpgraded {
                hero: 9,
                slot: at,
                from: 1,
                to: 2,
            }),
            adventure(AdventureEvent::KindKnown { object_type: 3 }),
            adventure(AdventureEvent::KindKnown { object_type: 50 }),
            adventure(AdventureEvent::KindKnown { object_type: 3 }),
            adventure(AdventureEvent::NothingGained {
                hero: 4,
                object_type: 3,
                anchor: Pos {
                    x: 1,
                    y: 1,
                    level: 0,
                },
                why: NothingWhy::Full,
            }),
        ],
    );
    let mut expected_army = before.clone();
    expected_army[i] = ArmyStack {
        slot: at,
        creature: 77,
        count: before[i].count - 1,
    };
    assert_eq!(
        v.adventure.heroes[0].own.clone().unwrap().army,
        expected_army
    );
    let mut expected = kinds;
    for ty in [3, 50] {
        if let Err(i) = expected.binary_search(&ty) {
            expected.insert(i, ty);
        }
    }
    assert_eq!(v.adventure.kinds, expected);
}

/// A viewer's whole view of town `town` told (`TownChanged`): its owner's, with a hall, unless
/// `hall` is none.
fn town_changed(town: u16, hall: Option<u8>) -> GameEvent {
    GameEvent::TownChanged {
        player: 1,
        town,
        owner: hall.map(|_| 1),
        fort: 2,
        capitol: hall == Some(4),
        built_today: true,
        level: if hall.is_some() { 3 } else { 1 },
        slots: hall.is_some(),
        hall,
        buildings: hall.map(|_| TownBuildings {
            built: WireU64(0b101),
            forbidden: WireU64(1 << 40),
            available: [5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2],
            portal: Some(Summoned {
                creature: 12,
                count: 3,
            }),
        }),
    }
}

/// A garrison's stack of creature `creature` as a level shows it.
fn shown(creature: u16, count: Option<u32>) -> GarrisonStack {
    GarrisonStack {
        creature,
        count_word: count.map(|_| 1),
        count,
    }
}

#[test]
fn the_fold_replaces_a_towns_view_by_its_told_one() {
    let mut v = view();
    let (anchor, faction, name) = (
        v.towns[0].anchor,
        v.towns[0].faction,
        v.towns[0].name.clone(),
    );
    let stack_seen = |town, stack, slot| GameEvent::TownStackSeen {
        player: 1,
        town,
        stack,
        slot,
    };
    let income = |town, resource, amount| GameEvent::TownIncomeSeen {
        player: 1,
        town,
        resource,
        amount,
    };
    fold(
        &mut v,
        &[
            // The owner's view: its stacks by slot, its income.
            town_changed(0, Some(4)),
            stack_seen(0, shown(30, Some(8)), Some(1)),
            stack_seen(0, shown(30, Some(2)), Some(6)),
            income(0, 0, 2),
            income(0, 6, 4000),
            // Another's: its stacks in slot order alone, no income.
            town_changed(1, None),
            stack_seen(1, shown(9, None), None),
            income(1, 6, 500),
        ],
    );
    assert_eq!(
        v.towns[0],
        Town {
            town: 0,
            anchor,
            owner: Some(1),
            faction,
            name,
            fort: 2,
            capitol: true,
            built_today: true,
            garrison: Garrison {
                level: 3,
                stacks: vec![shown(30, Some(8)), shown(30, Some(2))],
                army: Some(vec![stack(1, 8), stack(6, 2)]),
            },
            owner_facts: Some(TownOwnerFacts {
                hall: 4,
                income: vec![
                    Amount {
                        resource: 0,
                        amount: 2,
                    },
                    Amount {
                        resource: 6,
                        amount: 4000,
                    },
                ],
            }),
            buildings: Some(TownBuildings {
                built: WireU64(0b101),
                forbidden: WireU64(1 << 40),
                available: [5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2],
                portal: Some(Summoned {
                    creature: 12,
                    count: 3,
                }),
            }),
        }
    );
    let other = &v.towns[1];
    assert_eq!(
        (other.owner, other.capitol, other.buildings),
        (None, false, None)
    );
    assert_eq!(
        other.garrison,
        Garrison {
            level: 1,
            stacks: vec![shown(9, None)],
            army: None,
        }
    );
    assert_eq!(other.owner_facts, None);
}

/// A viewer's whole view of the map garrison of type `object_type` at `anchor` told
/// (`GarrisonChanged`, E08 T114b): its owner's team's, by slot with its removable flag, or
/// another viewer's at `level`.
fn garrison_changed(object_type: u16, anchor: Pos, owner: Option<u8>, level: u8) -> GameEvent {
    let ours = owner == Some(1);
    GameEvent::GarrisonChanged {
        player: 1,
        object_type,
        anchor,
        owner,
        level,
        slots: ours,
        removable: ours.then_some(true),
    }
}

#[test]
fn the_fold_replaces_a_map_garrisons_view_by_its_told_one() {
    // E08 T114b (ADV-GAR-006): the view's garrisons in their places' order: (3, 4, 0) another's
    // at level 2, (9, 4, 0) the viewer's own, (3, 2, 1) a neutral one at level 0.
    let mut v = view();
    let p = |x, y, level| Pos { x, y, level };
    let seen = |object_type, anchor, stack, slot| GameEvent::GarrisonStackSeen {
        player: 1,
        object_type,
        anchor,
        stack,
        slot,
    };
    let before = v.garrisons.clone();
    fold(
        &mut v,
        &[
            // The viewer's own one captured by another: at its taverns' level, packed.
            garrison_changed(33, p(9, 4, 0), Some(0), 1),
            seen(33, p(9, 4, 0), shown(9, None), None),
            // One it did not see yet (a reveal in the same command comes after): put in its
            // place, by slot with its flag.
            garrison_changed(33, p(5, 0, 0), Some(1), 3),
            seen(33, p(5, 0, 0), shown(30, Some(8)), Some(1)),
            seen(33, p(5, 0, 0), shown(31, Some(2)), Some(6)),
            // A stack of a garrison of another type there: nothing.
            seen(219, p(5, 0, 0), shown(9, None), None),
        ],
    );
    let garrison = |object_type, anchor, owner, garrison, removable| MapGarrison {
        object_type,
        anchor,
        owner,
        garrison,
        removable,
    };
    assert_eq!(
        v.garrisons,
        [
            garrison(
                33,
                p(5, 0, 0),
                Some(1),
                Garrison {
                    level: 3,
                    stacks: vec![shown(30, Some(8)), shown(31, Some(2))],
                    army: Some(vec![
                        stack(1, 8),
                        ArmyStack {
                            slot: 6,
                            creature: 31,
                            count: 2
                        }
                    ]),
                },
                Some(true)
            ),
            before[0].clone(),
            garrison(
                33,
                p(9, 4, 0),
                Some(0),
                Garrison {
                    level: 1,
                    stacks: vec![shown(9, None)],
                    army: None,
                },
                None
            ),
            before[2].clone(),
        ]
    );
}

/// E09 T28a (v64): a hero's move between a town's bars. The owner moves its own view between the
/// heroes on the map and its garrison heroes, a garrison hero's army changes as a hero's, and its
/// leaving the game at its town's capture removes it; another viewer sees a hero leave the map,
/// and one coming into sight is not carried (SRV-STUB-3).
#[test]
fn the_fold_moves_a_garrison_hero_between_the_map_and_its_town() {
    let mut v = view();
    assert_eq!(v.adventure.viewer, 1);
    let held = |v: &GameObservation| -> Vec<(u16, u16)> {
        v.adventure
            .garrison_heroes
            .iter()
            .map(|g| (g.town, g.id))
            .collect()
    };
    assert_eq!(held(&v), [(3, 7)]);
    let army = v.adventure.heroes[0].own.clone().unwrap().army;
    fold(
        &mut v,
        &[
            adventure(AdventureEvent::GarrisonEntered {
                hero: 4,
                town: 1,
                pos: at(0, 0),
            }),
            adventure(AdventureEvent::GarrisonEntered {
                hero: 9,
                town: 2,
                pos: at(1, 0),
            }),
            GameEvent::StackChanged {
                player: 1,
                at: ArmySlot {
                    army: ArmyRef::Hero { hero: 4 },
                    slot: 6,
                },
                stack: Some(stack(6, 3)),
            },
        ],
    );
    assert!(!v.adventure.heroes.iter().any(|h| h.id == 4 || h.id == 9));
    assert!(!v.heroes.iter().any(|info| info.id == 9));
    assert_eq!(held(&v), [(1, 4), (3, 7)]);
    let mut grown = army;
    grown.push(stack(6, 3));
    assert_eq!(v.adventure.garrison_heroes[0].own.army, grown);
    fold(
        &mut v,
        &[
            adventure(AdventureEvent::GarrisonLeft {
                hero: 7,
                town: 3,
                pos: at(2, 2),
            }),
            adventure(AdventureEvent::GarrisonLeft {
                hero: 12,
                town: 0,
                pos: at(1, 1),
            }),
            adventure(AdventureEvent::HeroRemoved {
                hero: 4,
                pos: at(0, 0),
            }),
        ],
    );
    assert_eq!(held(&v), []);
    let seen: Vec<(u16, u8, Pos, Option<u8>, bool)> = v
        .adventure
        .heroes
        .iter()
        .map(|h| (h.id, h.owner, h.pos, h.boat, h.own.is_some()))
        .collect();
    assert_eq!(
        seen,
        [
            (5, 1, at(0, 1), Some(2), true),
            (7, 1, at(2, 2), None, true)
        ]
    );
}

#[test]
fn the_fold_moves_the_heroes_and_the_boats() {
    let mut v = view();
    let step = |hero, to| AdventureEvent::HeroStepped {
        hero,
        from: Some(at(0, 0)),
        to,
    };
    fold(
        &mut v,
        &[
            adventure(step(4, Some(at(1, 1)))),
            adventure(AdventureEvent::HeroTeleported {
                hero: 9,
                from: None,
                to: Some(at(0, 0)),
            }),
            adventure(AdventureEvent::Boarded {
                hero: 5,
                boat: 1,
                at: at(1, 1),
            }),
            adventure(AdventureEvent::Boarded {
                hero: 77,
                boat: 1,
                at: at(2, 2),
            }),
        ],
    );
    let seen: Vec<(u16, Pos, Option<u8>)> = v
        .adventure
        .heroes
        .iter()
        .map(|h| (h.id, h.pos, h.boat))
        .collect();
    assert_eq!(
        seen,
        [
            (4, at(1, 1), None),
            (5, at(0, 1), Some(1)),
            (9, at(0, 0), None)
        ]
    );
    assert_eq!(v.adventure.boats, []);
    fold(
        &mut v,
        &[
            adventure(AdventureEvent::Landed {
                hero: 5,
                boat: 1,
                at: at(2, 0),
                facing: 3,
            }),
            adventure(AdventureEvent::Landed {
                hero: 77,
                boat: 2,
                at: at(0, 1),
                facing: 4,
            }),
            adventure(AdventureEvent::BoatBuilt {
                tile: at(0, 0),
                boat_type: 4,
            }),
            adventure(AdventureEvent::BoatBuilt {
                tile: at(2, 0),
                boat_type: 5,
            }),
            adventure(AdventureEvent::KeyGained { key: 3 }),
            adventure(step(9, None)),
            adventure(AdventureEvent::HeroRemoved {
                hero: 4,
                pos: at(1, 1),
            }),
        ],
    );
    let boat = |tile, kind, facing| Boat { tile, kind, facing };
    assert_eq!(
        v.adventure.boats,
        [
            boat(at(0, 0), 4, 2),
            boat(at(2, 0), 5, 2),
            boat(at(0, 1), 2, 4)
        ]
    );
    assert_eq!(v.adventure.heroes.len(), 1);
    assert_eq!(v.adventure.heroes[0].boat, None);
    assert_eq!(v.heroes, []);
    assert_eq!(v.adventure.keys, 0b1000_1001);
}

#[test]
fn the_fold_keeps_the_objects_and_the_tiles_they_cover() {
    let mut v = view();
    // A over (1, 1), (0, 1) and (0, 0), a bit off the map, visited from (1, 1); B over (0, 1);
    // C visited from (1, 1) underground; a mine and a town.
    let a = object(1, at(1, 1), 1 | 1 << 1 | 1 << 9 | 1 << 7, 1);
    let b = object(2, at(0, 1), 1, 0);
    let c = object(
        3,
        Pos {
            level: 1,
            ..at(1, 1)
        },
        0,
        1,
    );
    let mine = Object {
        guards: Some(Guards {
            creature: None,
            count_word: 2,
            army: Some(vec![stack(0, 9)]),
        }),
        ..object(4, at(2, 0), 0, 0)
    };
    let town = Object {
        town: Some(1),
        owner: Some(1),
        ..object(6, at(2, 1), 0, 0)
    };
    // In tile order (`Object::place`).
    v.adventure.objects = vec![mine.clone(), b.clone(), a, town.clone(), c.clone()];
    let known = |object_type, anchor, emptied| KnownObject {
        object_type,
        anchor,
        known: Known::Emptied { emptied },
    };
    v.adventure.known = vec![known(1, at(1, 1), true), known(2, at(0, 1), false)];
    // (0, 0) hidden, (1, 0) ground, (2, 0) and (0, 1) blocked, (1, 1) blocked and visited from,
    // (2, 1) hidden.
    let ground = v.adventure.palette[1];
    v.adventure.palette = vec![Tile::Hidden {}, ground, seen(true, false), seen(true, true)];
    v.adventure.tiles = vec![0, 1, 2, 2, 3, 0];
    let tiles = |v: &GameObservation| -> Vec<Tile> {
        (0..6).map(|i| *v.adventure.tile(i).unwrap()).collect()
    };
    let removed = adventure(AdventureEvent::ObjectRemoved {
        object_type: 1,
        anchor: at(1, 1),
    });
    fold(&mut v, &[removed]);
    // (1, 1) is free; (0, 1) stays blocked by B.
    let (hidden, blocked) = (Tile::Hidden {}, seen(true, false));
    assert_eq!(
        tiles(&v),
        [hidden, ground, blocked, blocked, seen(false, false), hidden]
    );
    assert_eq!(v.adventure.palette.len(), 4);
    assert_eq!(v.adventure.known, [known(2, at(0, 1), false)]);
    assert_eq!(v.adventure.objects, [mine, b, town, c]);
    // A month's monster on the seen tile, another on the ground of the fringe: each in its
    // place.
    let added = |anchor| {
        adventure(AdventureEvent::MonsterAdded {
            object_type: 5,
            anchor,
            creature: 9,
            count_word: 2,
        })
    };
    fold(&mut v, &[added(at(1, 1)), added(at(1, 0))]);
    assert_eq!(
        tiles(&v),
        [hidden, ground, blocked, blocked, seen(true, true), hidden]
    );
    let monster = Object {
        monster: Some(Monster {
            creature: 9,
            count_word: 2,
        }),
        ..object(5, at(1, 1), 1, 1)
    };
    assert_eq!(v.adventure.objects[3], monster);
    assert_eq!(v.adventure.objects[0].anchor, at(1, 0));
    let guards = |creature, count_word| {
        adventure(AdventureEvent::GuardsChanged {
            object_type: 4,
            anchor: at(2, 0),
            creature,
            count_word,
        })
    };
    fold(
        &mut v,
        &[
            adventure(AdventureEvent::MonsterChanged {
                object_type: 5,
                anchor: at(1, 0),
                count_word: 3,
            }),
            // B holds no monster.
            adventure(AdventureEvent::MonsterChanged {
                object_type: 2,
                anchor: at(0, 1),
                count_word: 3,
            }),
            adventure(AdventureEvent::ObjectFlagged {
                object_type: 2,
                anchor: at(0, 1),
                owner: 1,
            }),
            adventure(AdventureEvent::ObjectMarked {
                object_type: 2,
                anchor: at(0, 1),
            }),
            guards(Some(8), Some(1)),
        ],
    );
    let objects = &v.adventure.objects;
    assert_eq!(objects[0].monster.map(|m| m.count_word), Some(3));
    assert_eq!(objects[3].monster.map(|m| m.count_word), Some(2));
    assert_eq!((objects[2].owner, objects[2].visited), (Some(1), true));
    let kept = Guards {
        creature: Some(8),
        count_word: 1,
        army: Some(vec![stack(0, 9)]),
    };
    assert_eq!(objects[1].guards, Some(kept));
    fold(&mut v, &[guards(None, None)]);
    assert_eq!(v.adventure.objects[1].guards, None);
    fold(&mut v, &[guards(Some(8), Some(0))]);
    let hint = Guards {
        creature: Some(8),
        count_word: 0,
        army: None,
    };
    assert_eq!(v.adventure.objects[1].guards, Some(hint));
    fold(
        &mut v,
        &[adventure(AdventureEvent::TownCaptured {
            town: 1,
            anchor: at(2, 1),
            owner: 0,
        })],
    );
    assert_eq!(v.adventure.objects[4].owner, Some(0));
    assert_eq!(v.towns[1].owner, Some(0));
}

#[test]
fn a_reveal_puts_what_the_viewer_newly_sees_in_its_place() {
    let mut v = view();
    // The hand-built map: three columns, two rows; (0, 0) explored, (1, 0) on the fringe.
    v.adventure.explored = vec![WireU64(0b001), WireU64(0)];
    v.adventure.fringe = vec![WireU64(0b010), WireU64(0)];
    let nothing = GameReveal {
        adventure: AdventureReveal {
            viewer: v.viewer,
            palette: Vec::new(),
            areas: Vec::new(),
            heroes: Vec::new(),
            objects: Vec::new(),
            boats: Vec::new(),
            towns: Vec::new(),
        },
        heroes: Vec::new(),
        towns: Vec::new(),
        garrisons: Vec::new(),
    };
    let before = v.clone();
    v.reveal(&nothing);
    assert_eq!(v, before);
    let ground = Tile::Ground {
        terrain: 7,
        road: None,
        river: Some(1),
    };
    let tile = |explored, view| RevealedView { explored, view };
    let area = |x, y, width, tiles: &[u32]| RevealedArea {
        corner: at(x, y),
        width,
        tiles: tiles.to_vec(),
    };
    // Hero 4 is seen already (of heroes 4, 5 and 9), hero 6 is new; town 1 is seen already, town
    // 7 is new.
    let hero = |id, x| MapHero {
        id,
        owner: 1,
        pos: at(x, 0),
        boat: Some(0),
        own: None,
    };
    let info = |id, hero_type| HeroInfo {
        id,
        hero_type,
        details: None,
    };
    let town = |id: u16| Town {
        town: id,
        name: Some(String::from("Newly")),
        ..v.towns[0].clone()
    };
    let seen_already = v.adventure.objects[1].clone();
    let flagged = Object {
        owner: Some(0),
        ..seen_already.clone()
    };
    let new = object(8, at(0, 1), 1, 0);
    let boat = Boat {
        tile: at(2, 1),
        kind: 2,
        facing: 4,
    };
    let garrisons = v.garrisons.clone();
    let flag = |g: usize| MapGarrison {
        owner: Some(0),
        ..garrisons[g].clone()
    };
    let garrison = |anchor, object_type| MapGarrison {
        object_type,
        anchor,
        ..garrisons[2].clone()
    };
    v.reveal(&GameReveal {
        adventure: AdventureReveal {
            viewer: v.viewer,
            // Neither now, explored now, on the fringe now; off the map.
            palette: vec![
                tile(false, Tile::Hidden {}),
                tile(true, seen(true, false)),
                tile(false, ground),
                tile(true, seen(false, false)),
            ],
            areas: vec![
                // (0, 1)'s index names no view: it stays as it was.
                area(0, 0, 3, &[1, 2, 0, 9, 0, 3]),
                area(3, 0, 1, &[4]),
                // Beyond the sides a position holds: the second tile's column, the third's row.
                area(u16::MAX, u16::MAX, 2, &[4, 4, 4]),
                // No columns: no tiles.
                area(0, 0, 0, &[4]),
            ],
            heroes: vec![hero(4, 0), hero(6, 1)],
            objects: vec![new.clone(), flagged.clone()],
            boats: vec![boat],
            towns: vec![1, 7],
        },
        heroes: vec![info(4, 40), info(6, 60)],
        towns: vec![town(1), town(7)],
        // E08 T114b: a map garrison seen already replaced, a new one in its place.
        garrisons: vec![flag(1), garrison(at(1, 0), 219)],
    });
    assert_eq!(v.adventure.explored, [WireU64(0b010), WireU64(0)]);
    assert_eq!(v.adventure.fringe, [WireU64(0), WireU64(0b100)]);
    let views = views_of(&v);
    assert_eq!(views[..2], [Tile::Hidden {}, seen(true, false)]);
    assert_eq!(views[5], ground);
    assert_eq!(views[2..5], views_of(&before)[2..5]);
    let ids: Vec<u16> = v.adventure.heroes.iter().map(|h| h.id).collect();
    assert_eq!(ids, [4, 5, 6, 9]);
    assert_eq!(v.adventure.heroes[0], hero(4, 0));
    let infos: Vec<(u16, u16)> = v.heroes.iter().map(|h| (h.id, h.hero_type)).collect();
    assert_eq!(infos, [(4, 40), (6, 60), (9, before.heroes[1].hero_type)]);
    // The new object in its place, the one seen already replaced by its view now.
    assert_eq!(v.adventure.objects[0], new);
    assert_eq!(v.adventure.objects[2], flagged);
    assert_eq!(
        v.adventure.objects.len(),
        before.adventure.objects.len() + 1
    );
    assert!(v.adventure.boats.contains(&boat));
    let towns: Vec<(u16, Option<&str>)> = (v.towns.iter())
        .map(|t| (t.town, t.name.as_deref()))
        .collect();
    let name = |t: usize| before.towns[t].name.as_deref();
    let newly = Some("Newly");
    assert_eq!(
        towns,
        [
            (0, name(0)),
            (1, newly),
            (2, name(2)),
            (3, name(3)),
            (7, newly)
        ]
    );
    assert_eq!(
        v.garrisons,
        [
            garrison(at(1, 0), 219),
            garrisons[0].clone(),
            flag(1),
            garrisons[2].clone()
        ]
    );
}

#[test]
fn a_view_of_the_largest_sides_folds_its_far_corner_without_a_panic() {
    // A hostile or corrupt snapshot (E16 M5 T7b's review): sides whose tile count is beyond a
    // 32-bit `usize`, no tiles, an object in the far corner of the second level.
    let mut v = view();
    let corner = Pos {
        x: u16::MAX - 1,
        y: u16::MAX - 1,
        level: 1,
    };
    (v.adventure.width, v.adventure.height, v.adventure.levels) = (u16::MAX, u16::MAX, 2);
    v.adventure.tiles.clear();
    v.adventure.objects = vec![object(1, corner, 1, 1)];
    let mut far = v.clone();
    far.reveal(&GameReveal {
        adventure: AdventureReveal {
            viewer: v.viewer,
            palette: vec![RevealedView {
                explored: true,
                view: Tile::Hidden {},
            }],
            areas: vec![RevealedArea {
                corner,
                width: 1,
                tiles: vec![1],
            }],
            heroes: Vec::new(),
            objects: Vec::new(),
            boats: Vec::new(),
            towns: Vec::new(),
        },
        heroes: Vec::new(),
        towns: Vec::new(),
        garrisons: Vec::new(),
    });
    let removed = adventure(AdventureEvent::ObjectRemoved {
        object_type: 1,
        anchor: corner,
    });
    far.fold(&removed);
    // Beyond the view's tiles and planes: nothing to change there but the object.
    assert!(far.adventure.objects.is_empty());
    assert_eq!(far.adventure.explored, v.adventure.explored);
    assert_eq!(far.adventure.tiles.len(), 0);
}

/// The tile views of `v` in tile order.
fn views_of(v: &GameObservation) -> Vec<Tile> {
    (0..6).map(|i| *v.adventure.tile(i).unwrap()).collect()
}
