//! The whole game's commands and rejections on the wire (E16 M5 T3a): every command maps to the
//! wire and back unchanged, every rejection maps losslessly (a test-only inverse gives the
//! engine's error back), both round-trip, for hand-built lists with every variant and for the
//! legal commands, the played commands and the rejections of games played at random on the
//! fixture maps.

use std::collections::BTreeSet;
use std::string::String;
use std::vec;
use std::vec::Vec;

use h3_adventure::{AdventureCommand, AdventureCommandError, StandError};
use h3_battle::{CommandError, Hex};
use h3_content::{CreatureTypeId, Formation as EngineFormation};
use h3_contracts::{HeroId, PlayerId, TownId};
use h3_game::{
    Actor, Answer, ArenaChoice, ArmyCommand, ArmyCommandError, ArmyRef, ArmySlot, ArtifactPlace,
    BuildingId, ChestChoice, Consent, GameCommand, GameCommandError, HireAnswer, HireError, Manage,
    ManageError, NoopSink, Position, SchoolChoice, SkillOffer, TownCommand, TownCommandError,
    UniversityAnswer,
};
use h3_heroes::{CombineError, MoveError};
use h3_kernel::handle::Handle;
use h3_kernel::rng::{Rng, StreamId};
use h3_sim::adventure::AdventureEnv;
use h3_sim::adventure::rollout::choose;

use super::game::{RULES, round_trips};
use super::mapping::ENGINE_ERRORS;
use crate::map::from_engine;
use crate::map::game::{command, command_to_engine, reject};
use crate::wire::game as w;

fn slot(army: ArmyRef, slot: u8) -> ArmySlot {
    ArmySlot { army, slot }
}

/// A command of every kind, with every variant of the parts they hold.
pub(super) fn every_command() -> Vec<GameCommand> {
    let hero = HeroId::new(3);
    let at = h3_adventure::Pos::new(4, 5, 1);
    let mut commands = vec![
        GameCommand::EndTurn,
        GameCommand::Adventure(AdventureCommand::MoveHero {
            hero,
            path: vec![at, h3_adventure::Pos::new(5, 6, 1)],
        }),
        GameCommand::Adventure(AdventureCommand::MoveHero {
            hero,
            path: Vec::new(),
        }),
        GameCommand::Adventure(AdventureCommand::BuildBoat {
            object_type: Handle::from_raw(12),
            anchor: at,
        }),
        GameCommand::Adventure(AdventureCommand::Revisit { hero }),
        GameCommand::Battle(h3_battle::Command::Wait),
        GameCommand::Battle(h3_battle::Command::Move { to: Hex(17) }),
    ];
    let answers = [
        Answer::Offer(Consent::Accept),
        Answer::Offer(Consent::Decline),
        Answer::LevelUp(SkillOffer::First),
        Answer::LevelUp(SkillOffer::Second),
        Answer::Chest(ChestChoice::Gold),
        Answer::Chest(ChestChoice::Experience),
        Answer::Arena(ArenaChoice::Attack),
        Answer::Arena(ArenaChoice::Defense),
        Answer::School(SchoolChoice::First),
        Answer::School(SchoolChoice::Second),
        Answer::School(SchoolChoice::Leave),
        Answer::University(UniversityAnswer::Buy(2)),
        Answer::University(UniversityAnswer::Leave),
        Answer::Hire(HireAnswer::Buy {
            creature: CreatureTypeId::from_raw(40),
            amount: 9,
        }),
        Answer::Hire(HireAnswer::Close),
        Answer::Close,
    ];
    commands.extend(answers.map(GameCommand::Answer));
    let manage = [
        Manage::Army(ArmyCommand::Move {
            from: slot(ArmyRef::Hero(hero), 0),
            to: slot(ArmyRef::Group, 1),
        }),
        Manage::Army(ArmyCommand::Split {
            from: slot(ArmyRef::Garrison(TownId::new(2)), 3),
            to: slot(ArmyRef::Hero(hero), 4),
            count: 7,
        }),
        Manage::Army(ArmyCommand::Dismiss {
            at: slot(ArmyRef::Hero(hero), 6),
        }),
        Manage::Formation {
            hero,
            formation: EngineFormation::Loose,
        },
        Manage::Formation {
            hero,
            formation: EngineFormation::Tight,
        },
        Manage::Tactics { hero, on: true },
        Manage::Tactics { hero, on: false },
        Manage::Artifact {
            from: (hero, ArtifactPlace::Worn(Position(5))),
            to: (HeroId::new(4), ArtifactPlace::Backpack(2)),
        },
        Manage::Assemble {
            hero,
            at: Position(1),
        },
        Manage::Disassemble {
            hero,
            at: Position(0),
        },
    ];
    commands.extend(manage.map(GameCommand::Manage));
    commands.push(GameCommand::Town(TownCommand::Build {
        town: TownId::new(3),
        building: BuildingId::from_index(9).unwrap(),
    }));
    commands
}

/// A rejection of every kind, with every variant of the parts they hold.
fn every_rejection() -> Vec<GameCommandError> {
    use GameCommandError as E;
    let hero = HeroId::new(2);
    let mut errors = vec![
        E::UnknownPlayer(PlayerId::new(9)),
        E::NotYourTurn(PlayerId::new(1)),
        E::DecisionOpen,
        E::NoDecision,
        E::OfferGone,
        E::BattleOpen,
        E::NoBattle,
        E::NotYourSide,
        E::NotForSystem,
        E::WrongAnswer,
        E::WaitingForLevelUp(PlayerId::new(0)),
        E::SlotClosed,
        E::CannotPay,
        E::Manage(ManageError::NotYourHero(hero)),
        E::Manage(ManageError::NotTogether),
    ];
    errors.extend(ENGINE_ERRORS.map(E::Battle));
    {
        use AdventureCommandError as A;
        let stand = [
            StandError::Outside,
            StandError::Terrain,
            StandError::Blocked,
            StandError::Occupied,
            StandError::Visited,
        ];
        errors.extend(
            stand
                .map(|error| A::Blocked { step: 4, error })
                .into_iter()
                .chain([
                    A::NotYourHero(hero),
                    A::EmptyPath,
                    A::NotNeighbour { step: 0 },
                    A::Unexplored { step: 1 },
                    A::Visit { step: 2 },
                    A::FromAbove { step: 3 },
                    A::PastVisit { step: 5 },
                    A::NoStop { step: 6 },
                    A::TooFewPoints {
                        step: 7,
                        cost: 150,
                        points: 99,
                    },
                    A::Boat { step: 8 },
                    A::Landing { step: 9 },
                    A::Coast { step: 10 },
                    A::UnknownObject,
                    A::NotOwner,
                    A::NoBoatTile,
                    A::BoatTileTaken,
                    A::NothingToRevisit,
                ])
                .map(E::Adventure),
        );
    }
    {
        use ArmyCommandError as A;
        errors.extend(
            [
                A::NotYourHero(hero),
                A::NoGroup,
                A::NoSlot(7),
                A::NoStack(3),
                A::SameSlot,
                A::TooManyCreatures,
                A::LastStack,
                A::OtherCreature,
                A::SplitCount(0),
                A::Garrison(TownId::new(5)),
            ]
            .map(|error| E::Manage(ManageError::Army(error))),
        );
    }
    let worn = ArtifactPlace::Worn(Position(8));
    let backpack = ArtifactPlace::Backpack(3);
    errors.extend(
        [
            MoveError::NoArtifact(worn),
            MoveError::NoArtifact(backpack),
            MoveError::Unmovable(Handle::from_raw(1)),
            MoveError::SamePlace,
            MoveError::CannotPlace(worn),
            MoveError::CannotReturn(backpack),
        ]
        .map(|error| E::Manage(ManageError::Artifact(error))),
    );
    errors.extend(
        [
            CombineError::NoArtifact(Position(2)),
            CombineError::Incomplete(Position(4)),
            CombineError::NotCombination(Position(6)),
        ]
        .map(|error| E::Manage(ManageError::Combine(error))),
    );
    errors.extend(
        [
            HireError::NoSource,
            HireError::NotOffered(CreatureTypeId::from_raw(13)),
            HireError::Amount(0),
            HireError::NoRoom,
        ]
        .map(E::Hire),
    );
    errors.extend(
        [
            TownCommandError::UnknownTown,
            TownCommandError::NotOwner,
            TownCommandError::UnknownBuilding,
            TownCommandError::Built,
            TownCommandError::Forbidden,
            TownCommandError::Grail,
            TownCommandError::Requirements,
            TownCommandError::Capitol,
            TownCommandError::OfferedAs(BuildingId::from_index(37).unwrap()),
            TownCommandError::BuiltToday,
        ]
        .map(E::Town),
    );
    errors
}

// The rejections' inverse: the wire back to the engine's error, field by field (test code,
// independent of the mapping it checks; the battle's codes by the battle's own mapping, which
// `tests/mapping.rs` checks one to one).

fn step_back(step: u32) -> usize {
    usize::try_from(step).unwrap()
}

fn place_back(place: w::ArtifactPlace) -> ArtifactPlace {
    match place {
        w::ArtifactPlace::Worn { position } => ArtifactPlace::Worn(Position(position.into())),
        w::ArtifactPlace::Backpack { index } => ArtifactPlace::Backpack(index.into()),
    }
}

fn adventure_back(error: w::AdventureReject) -> AdventureCommandError {
    use AdventureCommandError as A;
    use w::AdventureReject as R;
    match error {
        R::NotYourHero { hero } => A::NotYourHero(HeroId::new(hero)),
        R::EmptyPath {} => A::EmptyPath,
        R::NotNeighbour { step } => A::NotNeighbour {
            step: step_back(step),
        },
        R::Unexplored { step } => A::Unexplored {
            step: step_back(step),
        },
        R::Blocked { step, error } => A::Blocked {
            step: step_back(step),
            error: match error {
                w::StandReject::Outside => StandError::Outside,
                w::StandReject::Terrain => StandError::Terrain,
                w::StandReject::Blocked => StandError::Blocked,
                w::StandReject::Occupied => StandError::Occupied,
                w::StandReject::Visited => StandError::Visited,
            },
        },
        R::Visit { step } => A::Visit {
            step: step_back(step),
        },
        R::FromAbove { step } => A::FromAbove {
            step: step_back(step),
        },
        R::PastVisit { step } => A::PastVisit {
            step: step_back(step),
        },
        R::NoStop { step } => A::NoStop {
            step: step_back(step),
        },
        R::TooFewPoints { step, cost, points } => A::TooFewPoints {
            step: step_back(step),
            cost,
            points,
        },
        R::Boat { step } => A::Boat {
            step: step_back(step),
        },
        R::Landing { step } => A::Landing {
            step: step_back(step),
        },
        R::Coast { step } => A::Coast {
            step: step_back(step),
        },
        R::UnknownObject {} => A::UnknownObject,
        R::NotOwner {} => A::NotOwner,
        R::NoBoatTile {} => A::NoBoatTile,
        R::BoatTileTaken {} => A::BoatTileTaken,
        R::NothingToRevisit {} => A::NothingToRevisit,
    }
}

fn manage_back(error: w::ManageReject) -> ManageError {
    match error {
        w::ManageReject::NotYourHero { hero } => ManageError::NotYourHero(HeroId::new(hero)),
        w::ManageReject::NotTogether {} => ManageError::NotTogether,
        w::ManageReject::Army { error } => ManageError::Army({
            use ArmyCommandError as A;
            use w::ArmyReject as R;
            match error {
                R::NotYourHero { hero } => A::NotYourHero(HeroId::new(hero)),
                R::NoGroup {} => A::NoGroup,
                R::NoSlot { slot } => A::NoSlot(slot),
                R::NoStack { slot } => A::NoStack(slot),
                R::SameSlot {} => A::SameSlot,
                R::TooManyCreatures {} => A::TooManyCreatures,
                R::LastStack {} => A::LastStack,
                R::OtherCreature {} => A::OtherCreature,
                R::SplitCount { count } => A::SplitCount(count),
                R::Garrison { town } => A::Garrison(TownId::new(town)),
            }
        }),
        w::ManageReject::Artifact { error } => ManageError::Artifact(match error {
            w::MoveReject::NoArtifact { place } => MoveError::NoArtifact(place_back(place)),
            w::MoveReject::Unmovable { artifact } => {
                MoveError::Unmovable(Handle::from_raw(artifact))
            }
            w::MoveReject::SamePlace {} => MoveError::SamePlace,
            w::MoveReject::CannotPlace { place } => MoveError::CannotPlace(place_back(place)),
            w::MoveReject::CannotReturn { place } => MoveError::CannotReturn(place_back(place)),
        }),
        w::ManageReject::Combine { error } => ManageError::Combine(match error {
            w::CombineReject::NoArtifact { at } => CombineError::NoArtifact(Position(at.into())),
            w::CombineReject::Incomplete { at } => CombineError::Incomplete(Position(at.into())),
            w::CombineReject::NotCombination { at } => {
                CombineError::NotCombination(Position(at.into()))
            }
        }),
    }
}

/// The engine's error of a rejection; none for the session's own (E16 M5 T8a).
fn reject_back(reject: w::GameReject) -> Option<GameCommandError> {
    use GameCommandError as E;
    use w::GameReject as R;
    Some(match reject {
        R::NotSeated {} | R::NotStarted {} => return None,
        R::UnknownPlayer { player } => E::UnknownPlayer(PlayerId::new(player)),
        R::NotYourTurn { player } => E::NotYourTurn(PlayerId::new(player)),
        R::DecisionOpen {} => E::DecisionOpen,
        R::NoDecision {} => E::NoDecision,
        R::OfferGone {} => E::OfferGone,
        R::Adventure { error } => E::Adventure(adventure_back(error)),
        R::BattleOpen {} => E::BattleOpen,
        R::NoBattle {} => E::NoBattle,
        R::NotYourSide {} => E::NotYourSide,
        R::Battle { code } => E::Battle(battle_back(code)),
        R::NotForSystem {} => E::NotForSystem,
        R::WrongAnswer {} => E::WrongAnswer,
        R::WaitingForLevelUp { player } => E::WaitingForLevelUp(PlayerId::new(player)),
        R::Manage { error } => E::Manage(manage_back(error)),
        R::SlotClosed {} => E::SlotClosed,
        R::CannotPay {} => E::CannotPay,
        R::Hire { error } => E::Hire(match error {
            w::HireReject::NoSource {} => HireError::NoSource,
            w::HireReject::NotOffered { creature } => {
                HireError::NotOffered(CreatureTypeId::from_raw(creature))
            }
            w::HireReject::Amount { amount } => HireError::Amount(amount),
            w::HireReject::NoRoom {} => HireError::NoRoom,
        }),
        R::Town { error } => E::Town(match error {
            w::TownReject::UnknownTown {} => TownCommandError::UnknownTown,
            w::TownReject::NotOwner {} => TownCommandError::NotOwner,
            w::TownReject::UnknownBuilding {} => TownCommandError::UnknownBuilding,
            w::TownReject::Built {} => TownCommandError::Built,
            w::TownReject::Forbidden {} => TownCommandError::Forbidden,
            w::TownReject::Grail {} => TownCommandError::Grail,
            w::TownReject::Requirements {} => TownCommandError::Requirements,
            w::TownReject::Capitol {} => TownCommandError::Capitol,
            w::TownReject::OfferedAs { building } => {
                TownCommandError::OfferedAs(BuildingId::from_index(usize::from(building)).unwrap())
            }
            w::TownReject::BuiltToday {} => TownCommandError::BuiltToday,
        }),
    })
}

/// The battle's engine error of a code: the one the battle's one-to-one mapping gives it.
fn battle_back(code: crate::wire::RejectCode) -> CommandError {
    let found: Vec<CommandError> = ENGINE_ERRORS
        .into_iter()
        .filter(|&e| from_engine::reject(e) == code)
        .collect();
    assert_eq!(found.len(), 1, "{code:?}");
    found[0]
}

/// The variant names of a value's `Debug` text, outermost first: what a list covers.
fn kinds<T: core::fmt::Debug>(value: &T) -> String {
    std::format!("{value:?}")
        .split(|c: char| !(c.is_alphanumeric() || c == '_'))
        .filter(|word| word.starts_with(|c: char| c.is_ascii_uppercase()))
        .collect::<Vec<_>>()
        .join(".")
}

#[test]
fn every_command_maps_to_the_wire_and_back_and_round_trips() {
    for c in every_command() {
        let wire = command(&c);
        round_trips(&wire);
        assert_eq!(command_to_engine(wire), Ok(c));
    }
}

#[test]
fn a_building_beyond_every_record_is_refused_before_the_engine() {
    let build = |building| w::GameCommand::Town {
        command: w::TownCommand::Build { town: 3, building },
    };
    let refused = w::GameReject::Town {
        error: w::TownReject::UnknownBuilding {},
    };
    assert_eq!(command_to_engine(build(64)), Err(refused));
    assert_eq!(command_to_engine(build(u8::MAX)), Err(refused));
    assert!(command_to_engine(build(63)).is_ok());
}

#[test]
fn every_rejection_maps_back_and_round_trips() {
    let errors = every_rejection();
    // 15 of the game's own, 31 of the battle's, 22 of the adventure's (E08 T105b2:
    // `NothingToRevisit`), 10 of the army's, 6 of an artifact's move, 3 of a combination's, 4 of a
    // hire's, 10 of a town's.
    assert_eq!(errors.len(), 101);
    let wire: BTreeSet<String> = errors.iter().map(|&e| kinds(&reject(e))).collect();
    assert_eq!(wire.len(), errors.len(), "distinct on the wire");
    for e in errors {
        let wire = reject(e);
        round_trips(&wire);
        assert_eq!(reject_back(wire), Some(e));
    }
    // The session's own (E16 M5 T8a) are no engine error's.
    for code in [w::GameReject::NotSeated {}, w::GameReject::NotStarted {}] {
        round_trips(&code);
        assert!(!wire.contains(&kinds(&code)));
        assert_eq!(reject_back(code), None);
    }
}

fn text<T: serde::Serialize>(value: &T) -> String {
    serde_json::to_string(value).unwrap()
}

#[test]
fn the_commands_json_is_serdes_default_form() {
    let at = w::Pos {
        x: 3,
        y: 1,
        level: 0,
    };
    let path = w::GameCommand::Adventure {
        command: w::AdventureCommand::MoveHero {
            hero: 1,
            path: vec![at],
        },
    };
    assert_eq!(
        text(&path),
        r#"{"Adventure":{"command":{"MoveHero":{"hero":1,"path":[{"x":3,"y":1,"level":0}]}}}}"#
    );
    assert_eq!(text(&w::GameCommand::EndTurn {}), r#"{"EndTurn":{}}"#);
    let yes = w::GameCommand::Answer {
        answer: w::Answer::Offer {
            consent: w::Consent::Accept,
        },
    };
    assert_eq!(
        text(&yes),
        r#"{"Answer":{"answer":{"Offer":{"consent":"Accept"}}}}"#
    );
    let split = w::Manage::Army {
        command: w::ArmyCommand::Split {
            from: w::ArmySlot {
                army: w::ArmyRef::Hero { hero: 2 },
                slot: 0,
            },
            to: w::ArmySlot {
                army: w::ArmyRef::Group {},
                slot: 1,
            },
            count: 5,
        },
    };
    assert_eq!(
        text(&split),
        r#"{"Army":{"command":{"Split":{"from":{"army":{"Hero":{"hero":2}},"slot":0},"to":{"army":{"Group":{}},"slot":1},"count":5}}}}"#
    );
    let rejected = w::GameReject::Adventure {
        error: w::AdventureReject::Blocked {
            step: 2,
            error: w::StandReject::Occupied,
        },
    };
    assert_eq!(
        text(&rejected),
        r#"{"Adventure":{"error":{"Blocked":{"step":2,"error":"Occupied"}}}}"#
    );
    assert_eq!(
        text(&w::GameReject::Battle {
            code: crate::wire::RejectCode::NotYourTurn
        }),
        r#"{"Battle":{"code":"NotYourTurn"}}"#
    );
    // Unknown fields are refused, in a command's variants as in its structs.
    for extra in [
        r#"{"EndTurn":{"player":1}}"#,
        r#"{"Adventure":{"command":{"MoveHero":{"hero":1,"path":[],"owner":0}}}}"#,
        r#"{"Manage":{"manage":{"Tactics":{"hero":1,"on":true,"player":0}}}}"#,
        r#"{"Manage":{"manage":{"Artifact":{"from":{"hero":1,"place":{"Worn":{"position":0}},"x":1},"to":{"hero":1,"place":{"Backpack":{"index":0}}}}}}}"#,
    ] {
        assert!(
            serde_json::from_str::<w::GameCommand>(extra).is_err(),
            "{extra}"
        );
    }
}

/// Plays `steps` random steps of `map`: before each, every actor's legal commands and every
/// hero's management commands map back; the played command maps back; issued by every other
/// actor, its rejection maps back. Returns the rejections' kinds met.
fn check_played(map: &[u8], seed: u64, steps: usize, met: &mut BTreeSet<String>) -> usize {
    let rules = &*RULES;
    let m = h3_map::read(map, &h3_map::Limits::default()).unwrap();
    let setup = std::sync::Arc::new(h3_map::import(&m, rules).unwrap());
    let mut env = AdventureEnv::new(rules.clone(), setup, seed, 28).unwrap();
    let mut rng = Rng::for_stream(seed, StreamId::AGENT);
    let back = |c: &GameCommand| {
        let wire = command(c);
        assert_eq!(command_to_engine(wire.clone()).as_ref(), Ok(c));
        wire
    };
    let mut checked = 0;
    let players: Vec<Actor> = (0..=u8::MAX)
        .take(env.state().players())
        .map(|p| Actor::Player(PlayerId::new(p)))
        .collect();
    let actors: Vec<Actor> = players.iter().copied().chain([Actor::System]).collect();
    for step in 0..steps {
        let state = env.state();
        for &actor in &actors {
            for c in h3_game::legal_commands(state, rules, actor) {
                back(&c);
                checked += 1;
            }
        }
        for hero in state.adventure().heroes() {
            let actor = Actor::Player(hero.owner);
            for m in h3_game::legal_management(state, rules, actor, hero.id) {
                back(&GameCommand::Manage(m));
                checked += 1;
            }
        }
        let Some(seat) = env.to_move().next() else {
            break;
        };
        let action = choose(&env, seat, &mut rng);
        let mut probe = env.fork();
        let (_, played) = probe.step_decoded(seat, action, &mut NoopSink).unwrap();
        let wire = back(&played);
        if step % 25 == 0 {
            round_trips(&wire);
        }
        for &actor in actors.iter().filter(|&&a| a != seat.actor()) {
            let mut state = env.state().clone();
            if let Err(e) = h3_game::apply(&mut state, rules, actor, played.clone(), &mut NoopSink)
            {
                assert_eq!(reject_back(reject(e)), Some(e), "step {step}");
                met.insert(kinds(&e));
                checked += 1;
            }
        }
        env = probe;
    }
    checked
}

#[test]
fn a_played_games_commands_and_rejections_map_back() {
    let maps: [(&[u8], u64); 3] = [
        (
            include_bytes!("../../../../fixtures/maps/e08-duel-36.h3m"),
            11,
        ),
        (
            include_bytes!("../../../../fixtures/maps/e09-towns-36.h3m"),
            12,
        ),
        (
            include_bytes!("../../../../fixtures/maps/e08-sea-36.h3m"),
            13,
        ),
    ];
    let mut met = BTreeSet::new();
    for (map, seed) in maps {
        let checked = check_played(map, seed, 300, &mut met);
        assert!(checked >= 1_000, "{checked}");
    }
    // Played commands of one actor issued by another: out of turn, by the system actor, a
    // battle's for the other side or without a battle, an answer without a decision.
    assert!(met.len() >= 5, "{met:?}");
}
