use h3_battle::{CastTarget, Command, Hex, StackId};
use h3_content::SpellId;

use super::*;

fn manage(command: Manage) -> GameCommand {
    GameCommand::Manage(command)
}

const fn slot(army: ArmyRef, slot: u8) -> ArmySlot {
    ArmySlot { army, slot }
}

const fn answer(answer: Answer) -> GameCommand {
    GameCommand::Answer(answer)
}

#[test]
fn commands_round_trip_through_their_text() {
    let hero = HeroId::new(12);
    let cases = [
        (PlayerId::new(0), GameCommand::EndTurn, "0 end-turn"),
        // Format v7 (E08 T63a): the answers typed by the decision's kind.
        (
            PlayerId::new(2),
            answer(Answer::Offer(Consent::Accept)),
            "2 answer offer accept",
        ),
        (
            PlayerId::new(3),
            answer(Answer::Offer(Consent::Decline)),
            "3 answer offer decline",
        ),
        (
            PlayerId::new(1),
            answer(Answer::LevelUp(SkillOffer::First)),
            "1 answer level-up first",
        ),
        (
            PlayerId::new(0),
            answer(Answer::LevelUp(SkillOffer::Second)),
            "0 answer level-up second",
        ),
        (
            PlayerId::new(0),
            answer(Answer::Chest(ChestChoice::Gold)),
            "0 answer chest gold",
        ),
        (
            PlayerId::new(0),
            answer(Answer::Chest(ChestChoice::Experience)),
            "0 answer chest experience",
        ),
        (
            PlayerId::new(0),
            answer(Answer::Arena(ArenaChoice::Attack)),
            "0 answer arena attack",
        ),
        (
            PlayerId::new(0),
            answer(Answer::Arena(ArenaChoice::Defense)),
            "0 answer arena defense",
        ),
        (
            PlayerId::new(0),
            answer(Answer::School(SchoolChoice::First)),
            "0 answer school first",
        ),
        (
            PlayerId::new(0),
            answer(Answer::School(SchoolChoice::Second)),
            "0 answer school second",
        ),
        (
            PlayerId::new(0),
            answer(Answer::School(SchoolChoice::Leave)),
            "0 answer school leave",
        ),
        (
            PlayerId::new(0),
            answer(Answer::University(UniversityAnswer::Buy(3))),
            "0 answer university buy 3",
        ),
        (
            PlayerId::new(0),
            answer(Answer::University(UniversityAnswer::Leave)),
            "0 answer university leave",
        ),
        // Format v10 (E08 T95b): a hire dialog's answers.
        (
            PlayerId::new(1),
            answer(Answer::Hire(HireAnswer::Buy {
                creature: CreatureTypeId::from_raw(116),
                amount: 4_000_000,
            })),
            "1 answer hire buy 116 4000000",
        ),
        (
            PlayerId::new(0),
            answer(Answer::Hire(HireAnswer::Close)),
            "0 answer hire close",
        ),
        (PlayerId::new(0), answer(Answer::Close), "0 answer close"),
        (
            PlayerId::new(7),
            GameCommand::Adventure(AdventureCommand::MoveHero {
                hero,
                path: vec![Pos::new(3, 4, 0), Pos::new(4, 5, 1)],
            }),
            "7 move 12 3,4,0 4,5,1",
        ),
        (
            PlayerId::new(1),
            GameCommand::Adventure(AdventureCommand::MoveHero {
                hero,
                path: Vec::new(),
            }),
            "1 move 12",
        ),
        // Format v8 (E08 T84b2): a Shipyard's boat without a visit, by its type and anchor.
        (
            PlayerId::new(2),
            GameCommand::Adventure(AdventureCommand::BuildBoat {
                object_type: ObjectTypeId::from_raw(41),
                anchor: Pos::new(9, 16, 1),
            }),
            "2 build-boat 41 9,16,1",
        ),
        // Format v11 (E09 T18b1): a town's build, by its town and building.
        (
            PlayerId::new(3),
            GameCommand::Town(TownCommand::Build {
                town: TownId::new(2),
                building: BuildingId::from_index(40).unwrap(),
            }),
            "3 town build 2 40",
        ),
        // Format v3 (E08 T19b1): a battle command, the battle replays' text without the side.
        (
            PlayerId::new(1),
            GameCommand::Battle(Command::Attack {
                target: StackId(3),
                from: Hex(41),
            }),
            "1 battle attack 3 41",
        ),
        (
            PlayerId::new(0),
            GameCommand::Battle(Command::Cast {
                spell: SpellId::from_raw(15),
                target: CastTarget::Stack(StackId(2)),
                second: CastTarget::Hex(Hex(30)),
            }),
            "0 battle cast 15 stack 2 hex 30",
        ),
        (
            PlayerId::new(0),
            GameCommand::Battle(Command::Wait),
            "0 battle wait",
        ),
        // Format v6 (E08 T51a): the management commands.
        (
            PlayerId::new(0),
            manage(Manage::Army(ArmyCommand::Move {
                from: slot(ArmyRef::Hero(hero), 0),
                to: slot(ArmyRef::Hero(HeroId::new(3)), 6),
            })),
            "0 army move 12:0 3:6",
        ),
        (
            PlayerId::new(1),
            manage(Manage::Army(ArmyCommand::Split {
                from: slot(ArmyRef::Group, 2),
                to: slot(ArmyRef::Hero(hero), 1),
                count: 40000,
            })),
            "1 army split g:2 12:1 40000",
        ),
        (
            PlayerId::new(1),
            manage(Manage::Army(ArmyCommand::Dismiss {
                at: slot(ArmyRef::Hero(hero), 4),
            })),
            "1 army dismiss 12:4",
        ),
        // E09 T8b2b: a town's garrison (no command manages it: the game refuses one that names it).
        (
            PlayerId::new(1),
            manage(Manage::Army(ArmyCommand::Move {
                from: slot(ArmyRef::Garrison(TownId::new(3)), 0),
                to: slot(ArmyRef::Hero(hero), 5),
            })),
            "1 army move t3:0 12:5",
        ),
        (
            PlayerId::new(2),
            manage(Manage::Formation {
                hero,
                formation: Formation::Tight,
            }),
            "2 formation 12 tight",
        ),
        (
            PlayerId::new(2),
            manage(Manage::Formation {
                hero,
                formation: Formation::Loose,
            }),
            "2 formation 12 loose",
        ),
        (
            PlayerId::new(0),
            manage(Manage::Tactics { hero, on: false }),
            "0 tactics 12 off",
        ),
        (
            PlayerId::new(0),
            manage(Manage::Tactics { hero, on: true }),
            "0 tactics 12 on",
        ),
        (
            PlayerId::new(0),
            manage(Manage::Artifact {
                from: (hero, ArtifactPlace::Backpack(0)),
                to: (HeroId::new(3), ArtifactPlace::Worn(Position(18))),
            }),
            "0 artifact 12:b0 3:w18",
        ),
        (
            PlayerId::new(0),
            manage(Manage::Assemble {
                hero,
                at: Position(6),
            }),
            "0 assemble 12 6",
        ),
        (
            PlayerId::new(0),
            manage(Manage::Disassemble {
                hero,
                at: Position(2),
            }),
            "0 disassemble 12 2",
        ),
    ];
    for (player, command, text) in cases {
        let actor = Actor::Player(player);
        assert_eq!(format_command(actor, &command), text);
        assert_eq!(parse_command(text).unwrap(), (actor, command));
    }
    // Format v4 (E08 T20b2): the system actor, a monster's side's commands.
    let wait = GameCommand::Battle(Command::Wait);
    assert_eq!(format_command(Actor::System, &wait), "system battle wait");
    assert_eq!(
        parse_command("system battle wait").unwrap(),
        (Actor::System, wait)
    );
    for bad in [
        "",
        "x end-turn",
        "256 end-turn",
        "0 jump",
        "0",
        "0 end-turn extra",
        "0 accept",
        "0 decline",
        "0 answer first",
        "0 answer",
        "0 answer offer",
        "0 answer offer maybe",
        "0 answer offer accept now",
        "0 answer level-up third",
        "0 answer chest silver",
        "0 answer arena power",
        "0 answer school stay",
        "0 answer university",
        "0 answer university buy",
        "0 answer university buy x",
        "0 answer university buy 256",
        "0 answer university buy 1 now",
        "0 answer university stay",
        "0 answer hire",
        "0 answer hire buy",
        "0 answer hire buy 2",
        "0 answer hire buy x 3",
        "0 answer hire buy 70000 3",
        "0 answer hire buy 2 x",
        "0 answer hire buy 2 -1",
        "0 answer hire buy 2 3 now",
        "0 answer hire close now",
        "0 answer hire leave",
        "0 answer close now",
        "0 answer shrug",
        "0 move",
        "0 move h1",
        "0 move 1 3,4",
        "0 move 1 3,4,0,1",
        "0 move 1 3,4,x",
        "0 move 1 70000,4,0",
        "0 build-boat",
        "0 build-boat 41",
        "0 build-boat x 9,16,1",
        "0 build-boat 70000 9,16,1",
        "0 build-boat 41 9,16",
        "0 build-boat 41 9,16,1 now",
        "0 town",
        "0 town build",
        "0 town build 2",
        "0 town raze 2 7",
        "0 town build x 7",
        "0 town build 70000 7",
        "0 town build 2 x",
        "0 town build 2 64",
        "0 town build 2 7 now",
        "0 battle",
        "0 battle fly 3",
        "0 battle A wait",
        "0 battle wait now",
        "system",
        "systems battle wait",
        "0 army",
        "0 army jump 1:0",
        "0 army move 1:0",
        "0 army move 1-0 1:1",
        "0 army move x:0 1:1",
        "0 army move 1:0 1:x",
        "0 army split 1:0 1:1",
        "0 army split 1:0 1:1 x",
        "0 army dismiss",
        "0 army dismiss 1:0 now",
        "0 formation 1",
        "0 formation 1 wedge",
        "0 formation x loose",
        "0 tactics 1",
        "0 tactics 1 maybe",
        "0 artifact 1:w0",
        "0 artifact 1:w0 1",
        "0 artifact 1:x0 1:b0",
        "0 artifact 1:w 1:b0",
        "0 artifact 1:b0 1:wx",
        "0 artifact x:b0 1:w0",
        "0 artifact 1: 1:w0",
        "0 assemble 1",
        "0 assemble 1 x",
        "0 disassemble",
        "0 disassemble 1",
    ] {
        assert!(
            matches!(parse_command(bad), Err(AdventureFileError::Format(_))),
            "{bad}"
        );
    }
}

#[test]
fn map_paths_stay_below_the_root() {
    for ok in ["fixtures/maps/e08-duel-36.h3m", "./a.h3m", "maps/x/y.h3m"] {
        assert!(check_map_path(ok).is_ok(), "{ok}");
    }
    for bad in ["", "../x.h3m", "maps/../../x.h3m", "/etc/passwd"] {
        assert!(check_map_path(bad).is_err(), "{bad}");
    }
}
