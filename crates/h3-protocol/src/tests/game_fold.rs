//! The game fold over the golden replays (E16 M5 T3c1; SRV-Q26, SRV-Q27): each viewer's view,
//! folded with its projected events after every command, against `observe`, part by part. A
//! battle's snapshot replaces the view's battle when it opens (SRV-Q27 (1)); after each command the
//! view is `observe`'s again, so each difference is that command's own.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::string::{String, ToString};
use std::vec;
use std::vec::Vec;
use std::{eprintln, format, fs};

use h3_contracts::PlayerId;
use h3_game::{GameRuleset, GameSetup, GameState, Viewer, apply};

use crate::map::game::{events, observe};
use crate::wire::game::*;

/// A part of a view: its name and whether two views agree on it.
type Part = (&'static str, fn(&GameObservation, &GameObservation) -> bool);

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

/// The parts, every field of the view in one of them.
const PARTS: &[Part] = &[
    ("day and month", |a, b| (a.day, a.month) == (b.day, b.month)),
    ("turns", |a, b| {
        (&a.players, &a.turn_group) == (&b.players, &b.turn_group)
    }),
    ("treasury", |a, b| a.treasury == b.treasury),
    ("timed events", |a, b| a.events_today == b.events_today),
    ("decision", |a, b| a.decision == b.decision),
    ("battle open", |a, b| {
        a.battle.is_some() == b.battle.is_some()
    }),
    ("heroes in sight", |a, b| {
        let ids = |o: &GameObservation| {
            let map: Vec<_> = o.adventure.heroes.iter().map(|h| (h.id, h.owner)).collect();
            let info: Vec<_> = o.heroes.iter().map(|h| (h.id, h.hero_type)).collect();
            (map, info)
        };
        ids(a) == ids(b)
    }),
    ("hero tiles and boats", |a, b| {
        pairs(&a.adventure.heroes, &b.adventure.heroes, |h| h.id)
            .all(|(x, y)| (x.pos, x.boat) == (y.pos, y.boat))
    }),
    ("hero details", |a, b| {
        pairs(&a.heroes, &b.heroes, |h| h.id).all(|(x, y)| x.details == y.details)
    }),
    ("armies", |a, b| own(a, b, |o| o.army.clone())),
    ("movement", |a, b| {
        own(a, b, |o| (o.movement, o.max_movement))
    }),
    ("formation and tactics", |a, b| {
        own(a, b, |o| (o.formation, o.tactics))
    }),
    ("once-marks", |a, b| own(a, b, |o| o.marks.clone())),
    ("explored tiles", |a, b| {
        let planes =
            |o: &GameObservation| (o.adventure.explored.clone(), o.adventure.fringe.clone());
        planes(a) == planes(b)
    }),
    ("tile views", |a, b| {
        (&a.adventure.palette, &a.adventure.tiles) == (&b.adventure.palette, &b.adventure.tiles)
    }),
    ("objects in sight", |a, b| {
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
    }),
    ("monsters", |a, b| objects(a, b, |o| o.monster)),
    ("object owners", |a, b| objects(a, b, |o| o.owner)),
    ("mine production", |a, b| objects(a, b, |o| o.production)),
    ("mine guards", |a, b| objects(a, b, |o| o.guards.clone())),
    ("visited marks", |a, b| objects(a, b, |o| o.visited)),
    ("shipyard tiles", |a, b| objects(a, b, |o| o.boat_tile)),
    ("known objects", |a, b| {
        a.adventure.known == b.adventure.known
    }),
    ("bought", |a, b| a.adventure.bought == b.adventure.bought),
    ("keys", |a, b| a.adventure.keys == b.adventure.keys),
    ("empty boats", |a, b| a.adventure.boats == b.adventure.boats),
    ("towns in sight", |a, b| {
        let ids = |o: &GameObservation| o.towns.iter().map(|t| t.town).collect::<Vec<_>>();
        ids(a) == ids(b)
    }),
    ("town owners", |a, b| towns(a, b, |t| t.owner)),
    ("town pictures", |a, b| {
        towns(a, b, |t| {
            (t.anchor, t.faction, t.name.clone(), t.fort, t.capitol)
        })
    }),
    ("garrisons", |a, b| towns(a, b, |t| t.garrison.clone())),
    ("town owner facts", |a, b| {
        towns(a, b, |t| t.owner_facts.clone())
    }),
];

/// The parts the events carry whole: the fold equals `observe` on them after every command of
/// every golden replay, for every viewer.
const EXACT: &[&str] = &[
    "day and month",
    "turns",
    "treasury",
    "timed events",
    "battle open",
    "hero tiles and boats",
    "formation and tactics",
    "monsters",
    "object owners",
    "mine guards",
    "visited marks",
    "keys",
    "town owners",
    "town pictures",
];

/// Where a part differed.
#[derive(Default)]
struct Differences {
    /// The viewer-commands it differed after.
    count: usize,
    /// Those of them whose explored tiles grew: a reveal, whose content no event carries
    /// (SRV-Q26 (a)).
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

/// Runs every golden adventure replay and compares each viewer's folded view with `observe`
/// after every command; returns each part's differences and the viewer-commands run.
fn run() -> (BTreeMap<&'static str, Differences>, usize) {
    let root = root();
    let dir = root.join("fixtures/replays/adventure");
    let mut files: Vec<PathBuf> = fs::read_dir(&dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|e| e == "toml"))
        .collect();
    files.sort();
    assert_eq!(files.len(), 55, "the golden adventure replays");
    let mut rules: BTreeMap<(Vec<String>, String), GameRuleset> = BTreeMap::new();
    let mut differences: BTreeMap<&'static str, Differences> = BTreeMap::new();
    let mut runs = 0;
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
        let mut state = GameState::new(rules, &setup, replay.seed).unwrap();
        let viewers: Vec<Viewer> = (0..=u8::MAX)
            .take(state.players())
            .map(|p| Viewer::Player(PlayerId::new(p)))
            .collect();
        let mut views: Vec<GameObservation> =
            viewers.iter().map(|&v| observe(&state, rules, v)).collect();
        for (step, (actor, command)) in replay.commands.iter().enumerate() {
            let mut told = Vec::new();
            apply(&mut state, rules, *actor, command.clone(), &mut told).unwrap();
            for (&viewer, view) in viewers.iter().zip(&mut views) {
                let told = events(&told, viewer);
                for event in &told {
                    view.fold(event);
                }
                let observed = observe(&state, rules, viewer);
                // The battle's snapshot when it opens (SRV-Q27 (1)).
                if told
                    .iter()
                    .any(|e| matches!(e, GameEvent::BattleStarted { .. }))
                {
                    view.battle.clone_from(&observed.battle);
                }
                let reveal = (&view.adventure.explored, &view.adventure.fringe)
                    != (&observed.adventure.explored, &observed.adventure.fringe);
                let mut any = false;
                for &(part, agree) in PARTS {
                    if agree(view, &observed) {
                        continue;
                    }
                    any = true;
                    let at = differences.entry(part).or_default();
                    at.count += 1;
                    at.replays.insert(name.clone());
                    at.with_reveal += usize::from(reveal);
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
    }
    (differences, runs)
}

#[test]
fn the_fold_against_observe_over_the_golden_replays() {
    let (differences, runs) = run();
    // The report (T3c1; T3c2 makes it the gate): `cargo test -p h3-protocol fold_against -- --nocapture`.
    #[allow(clippy::print_stderr)] // a test's report
    {
        eprintln!("{runs} viewer-commands; the parts that differed:");
        for (part, at) in &differences {
            eprintln!(
                "- {part}: {} viewer-commands ({} with a reveal) in {} replays; first {}",
                at.count,
                at.with_reveal,
                at.replays.len(),
                at.first.as_deref().unwrap_or_default()
            );
        }
    }
    for part in EXACT {
        assert!(
            !differences.contains_key(part),
            "{part}: {}",
            differences[part].first.as_deref().unwrap_or_default()
        );
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
        blocked: crate::wire::WireU64(blocked),
        visitable: crate::wire::WireU64(visitable),
        drawn: crate::wire::WireU64(0),
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
            GameEvent::WeekStarted { week: 5 },
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
                object_type: ty,
                anchor: a,
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
            GameEvent::TownChanged {
                town: 8,
                capitol: true,
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
    let bonus = |morale, luck, mark| HeroChange::Bonus { morale, luck, mark };
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
            // Once-mark 0 is set: no bonus; mark 1 is not.
            change(bonus(1, 1, Some(0))),
            change(bonus(1, 1, Some(1))),
            change(bonus(-128, 0, None)),
            change(bonus(0, 1, Some(9))),
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
    let bonuses = Bonuses {
        morale: -128,
        luck: 4,
        marks: 0b111,
    };
    assert_eq!(d.bonuses, bonuses);
    fold(&mut v, &[change(HeroChange::ClearBonuses {})]);
    let cleared = Bonuses {
        morale: 0,
        luck: 0,
        marks: 0,
    };
    assert_eq!(v.heroes[0].details.as_ref().unwrap().bonuses, cleared);
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
            GameEvent::TownChanged {
                town: 0,
                capitol: true,
            },
        ],
    );
    let own = v.adventure.heroes[0].own.clone().unwrap();
    assert_eq!(own.army, [stack(0, 2), stack(5, 3)]);
    assert_eq!(v.towns[1].garrison.army, Some(vec![stack(3, 8)]));
    assert!(v.towns[0].capitol);
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
    v.adventure.objects = vec![a, b.clone(), c.clone(), mine, town];
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
    assert_eq!(v.adventure.objects[..2], [b, c]);
    // A month's monster on the seen tile, another on the ground of the fringe.
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
    assert_eq!(v.adventure.objects[4], monster);
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
    assert_eq!(objects[5].monster.map(|m| m.count_word), Some(3));
    assert_eq!(objects[4].monster.map(|m| m.count_word), Some(2));
    assert_eq!((objects[0].owner, objects[0].visited), (Some(1), true));
    let kept = Guards {
        creature: Some(8),
        count_word: 1,
        army: Some(vec![stack(0, 9)]),
    };
    assert_eq!(objects[2].guards, Some(kept));
    fold(&mut v, &[guards(None, None)]);
    assert_eq!(v.adventure.objects[2].guards, None);
    fold(&mut v, &[guards(Some(8), Some(0))]);
    let hint = Guards {
        creature: Some(8),
        count_word: 0,
        army: None,
    };
    assert_eq!(v.adventure.objects[2].guards, Some(hint));
    fold(
        &mut v,
        &[adventure(AdventureEvent::TownCaptured {
            town: 1,
            anchor: at(2, 1),
            owner: 0,
        })],
    );
    assert_eq!(v.adventure.objects[3].owner, Some(0));
    assert_eq!(v.towns[1].owner, Some(0));
}
