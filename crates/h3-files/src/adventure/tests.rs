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
        // Format v19 (E09 T39a): a map Tavern's answers.
        (
            PlayerId::new(2),
            answer(Answer::Tavern(TavernAnswer::Hire { slot: 1 })),
            "2 answer tavern hire 1",
        ),
        (
            PlayerId::new(0),
            answer(Answer::Tavern(TavernAnswer::Close)),
            "0 answer tavern close",
        ),
        // Format v21 (E08 T133b): a market's answers, resources by index.
        (
            PlayerId::new(1),
            answer(Answer::Market(MarketAnswer::Trade {
                give: ResourceId::from_raw(6),
                take: ResourceId::from_raw(0),
                lots: 3,
            })),
            "1 answer market trade 6 0 3",
        ),
        (
            PlayerId::new(0),
            answer(Answer::Market(MarketAnswer::Transfer {
                resource: ResourceId::from_raw(2),
                amount: 400,
                to: PlayerId::new(3),
            })),
            "0 answer market transfer 2 400 3",
        ),
        (
            PlayerId::new(2),
            answer(Answer::Market(MarketAnswer::Sell {
                slot: 4,
                take: ResourceId::from_raw(6),
                lots: 12,
            })),
            "2 answer market sell 4 6 12",
        ),
        (
            PlayerId::new(0),
            answer(Answer::Market(MarketAnswer::Buy {
                entry: 6,
                pay: ResourceId::from_raw(5),
            })),
            "0 answer market buy 6 5",
        ),
        // Format v28 (E09 T58b): a town's artifact sale by its place.
        (
            PlayerId::new(1),
            answer(Answer::Market(MarketAnswer::SellArtifact {
                place: ArtifactPlace::Worn(Position(13)),
                take: ResourceId::from_raw(6),
            })),
            "1 answer market sell-artifact w13 6",
        ),
        (
            PlayerId::new(0),
            answer(Answer::Market(MarketAnswer::SellArtifact {
                place: ArtifactPlace::Backpack(2),
                take: ResourceId::from_raw(0),
            })),
            "0 answer market sell-artifact b2 0",
        ),
        (
            PlayerId::new(0),
            answer(Answer::Market(MarketAnswer::Leave)),
            "0 answer market leave",
        ),
        // Format v22 (E08 T134a): an Altar's answers, the places in the change's order.
        (
            PlayerId::new(1),
            answer(Answer::Altar(AltarAnswer::Artifacts(
                AltarPlaces::from_bits(1 << 2 | 1 << 18, 1 << 63 | 1),
            ))),
            "1 answer altar artifacts w2 w18 b63 b0",
        ),
        (
            PlayerId::new(0),
            answer(Answer::Altar(AltarAnswer::Artifacts(AltarPlaces::NONE))),
            "0 answer altar artifacts",
        ),
        (
            PlayerId::new(2),
            answer(Answer::Altar(AltarAnswer::Creatures([
                0, 5, 0, 0, 12, 0, 1,
            ]))),
            "2 answer altar creatures 0 5 0 0 12 0 1",
        ),
        (
            PlayerId::new(0),
            answer(Answer::Altar(AltarAnswer::Leave)),
            "0 answer altar leave",
        ),
        // Format v23 (E08 T134b1): a Hill Fort's answers.
        (
            PlayerId::new(1),
            answer(Answer::HillFort(HillFortAnswer::Upgrade(6))),
            "1 answer hill-fort upgrade 6",
        ),
        (
            PlayerId::new(0),
            answer(Answer::HillFort(HillFortAnswer::All)),
            "0 answer hill-fort all",
        ),
        (
            PlayerId::new(0),
            answer(Answer::HillFort(HillFortAnswer::Leave)),
            "0 answer hill-fort leave",
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
        // Format v15 (E09 T27): a town's recruitment, by its town and building.
        (
            PlayerId::new(0),
            GameCommand::Town(TownCommand::Recruit {
                town: TownId::new(1),
                building: BuildingId::from_index(30).unwrap(),
            }),
            "0 town recruit 1 30",
        ),
        // Format v16 (E09 T28a): a hero's move between a town's bars, by its town and move.
        (
            PlayerId::new(1),
            GameCommand::Town(TownCommand::Heroes {
                town: TownId::new(3),
                change: TownHeroChange::Enter,
            }),
            "1 town heroes 3 enter",
        ),
        (
            PlayerId::new(0),
            GameCommand::Town(TownCommand::Heroes {
                town: TownId::new(0),
                change: TownHeroChange::Leave,
            }),
            "0 town heroes 0 leave",
        ),
        (
            PlayerId::new(2),
            GameCommand::Town(TownCommand::Heroes {
                town: TownId::new(7),
                change: TownHeroChange::Swap,
            }),
            "2 town heroes 7 swap",
        ),
        // Format v24 (E09 T49a2): a town screen's opening, by its town.
        (
            PlayerId::new(1),
            GameCommand::Town(TownCommand::Open {
                town: TownId::new(3),
            }),
            "1 town open 3",
        ),
        // Format v25 (E09 T49b): a spell book's purchase, by its town.
        (
            PlayerId::new(2),
            GameCommand::Town(TownCommand::BuySpellBook {
                town: TownId::new(4),
            }),
            "2 town book 4",
        ),
        // Format v27 (E09 T58a1): a market window's opening at a town, by its town.
        (
            PlayerId::new(1),
            GameCommand::Town(TownCommand::Market {
                town: TownId::new(5),
            }),
            "1 town market 5",
        ),
        // Format v29 (E09 T59a): a war machine's purchase by its town and building, a boat's build
        // by its town.
        (
            PlayerId::new(1),
            GameCommand::Town(TownCommand::BuyWarMachine {
                town: TownId::new(2),
                building: BuildingId::from_index(16).unwrap(),
            }),
            "1 town machine 2 16",
        ),
        (
            PlayerId::new(0),
            GameCommand::Town(TownCommand::BuildBoat {
                town: TownId::new(6),
            }),
            "0 town boat 6",
        ),
        // Format v30 (E09 T59b1): a town's University by its town and hero.
        (
            PlayerId::new(1),
            GameCommand::Town(TownCommand::University {
                town: TownId::new(6),
                hero: HeroId::new(3),
            }),
            "1 town university 6 3",
        ),
        // Format v26 (E08 T142a): a cast on the map, by its hero and spell, its target if any.
        (
            PlayerId::new(0),
            GameCommand::Cast {
                hero: HeroId::new(1),
                spell: SpellId::from_raw(6),
                target: None,
            },
            "0 cast 1 6",
        ),
        (
            PlayerId::new(1),
            GameCommand::Cast {
                hero: HeroId::new(2),
                spell: SpellId::from_raw(8),
                target: Some(Pos::new(12, 4, 1)),
            },
            "1 cast 2 8 12,4,1",
        ),
        // Format v12 (E08 T105b2): a revisit, by its hero.
        (
            PlayerId::new(1),
            GameCommand::Adventure(AdventureCommand::Revisit {
                hero: HeroId::new(4),
            }),
            "1 revisit 4",
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
        // E08 T114a2 (format v13): a map garrison's army by its object index, in its exchange.
        (
            PlayerId::new(1),
            manage(Manage::Army(ArmyCommand::Split {
                from: slot(ArmyRef::Hero(hero), 2),
                to: slot(ArmyRef::MapGarrison(417), 6),
                count: 3,
            })),
            "1 army split 12:2 m417:6 3",
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
        // Format v18 (E09 T38b2): a hero's dismissal.
        (
            PlayerId::new(1),
            manage(Manage::Dismiss { hero }),
            "1 dismiss 12",
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
    // Format v14 (E16 M5 T14): the system actor's own commands, for a player.
    let player = PlayerId::new(1);
    for (command, text) in [
        (SystemCommand::EndTurn { player }, "system for 1 end-turn"),
        (
            SystemCommand::Answer { player },
            "system for 1 answer default",
        ),
        (
            SystemCommand::Battle {
                player,
                command: Command::Defend,
            },
            "system for 1 battle defend",
        ),
    ] {
        let command = GameCommand::System(command);
        assert_eq!(format_command(Actor::System, &command), text);
        assert_eq!(parse_command(text).unwrap(), (Actor::System, command));
    }
    // A player's, which the game refuses, is written all the same.
    let ended = GameCommand::System(SystemCommand::EndTurn { player });
    assert_eq!(
        format_command(Actor::Player(player), &ended),
        "1 for 1 end-turn"
    );
    assert_eq!(
        parse_command("1 for 1 end-turn").unwrap(),
        (Actor::Player(player), ended)
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
        "0 revisit",
        "0 revisit h1",
        "0 revisit 70000",
        "0 revisit 1 2",
        "0 town",
        "0 town build",
        "0 town build 2",
        "0 town raze 2 7",
        "0 town open",
        "0 town open x",
        "0 town open 70000",
        "0 town open 2 now",
        "0 town book",
        "0 town book x",
        "0 town book 70000",
        "0 town book 2 now",
        "0 town market",
        "0 town market x",
        "0 town market 2 now",
        "0 town boat",
        "0 town boat x",
        "0 town boat 2 now",
        "0 town machine 2",
        "0 town machine x 16",
        "0 town machine 2 x",
        "0 town machine 2 16 now",
        "0 town university 2",
        "0 town university x 1",
        "0 town university 2 x",
        "0 town university 2 1 now",
        "0 cast",
        "0 cast 1",
        "0 cast h1 6",
        "0 cast 1 x",
        "0 cast 1 70000",
        "0 cast 1 6 12,4",
        "0 cast 1 6 12,4,1 now",
        "0 town build x 7",
        "0 town build 70000 7",
        "0 town build 2 x",
        "0 town build 2 64",
        "0 town build 2 7 now",
        "0 town recruit 2",
        "0 town recruit 2 64",
        "0 town recruit 2 7 now",
        "0 town heroes",
        "0 town heroes 2",
        "0 town heroes x enter",
        "0 town heroes 70000 enter",
        "0 town heroes 2 visit",
        "0 town heroes 2 enter now",
        "0 battle",
        "0 battle fly 3",
        "0 battle A wait",
        "0 battle wait now",
        "system",
        "systems battle wait",
        "system for",
        "system for x end-turn",
        "system for 256 end-turn",
        "system for 1",
        "system for 1 move 1 3,4,0",
        "system for 1 end-turn now",
        "system for 1 answer",
        "system for 1 answer offer decline",
        "system for 1 answer default now",
        "system for 1 battle",
        "system for 1 battle fly 3",
        "0 army",
        "0 army jump 1:0",
        "0 army move 1:0",
        "0 army move 1-0 1:1",
        "0 army move x:0 1:1",
        "0 army move 1:0 1:x",
        "0 army move 1:0 m:1",
        "0 army move 1:0 mx:1",
        "0 army move m1:x 1:0",
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
        "0 dismiss",
        "0 dismiss x",
        "0 dismiss 1 2",
        "0 answer market",
        "0 answer market barter",
        "0 answer market trade 6 0",
        "0 answer market trade x 0 3",
        "0 answer market trade 6 x 3",
        "0 answer market trade 6 0 x",
        "0 answer market transfer 2 400",
        "0 answer market transfer x 400 3",
        "0 answer market transfer 2 x 3",
        "0 answer market transfer 2 400 x",
        "0 answer market sell 4 6",
        "0 answer market sell x 6 12",
        "0 answer market sell 4 x 12",
        "0 answer market sell 4 6 x",
        "0 answer market buy 6",
        "0 answer market buy x 5",
        "0 answer market buy 6 x",
        "0 answer market sell-artifact",
        "0 answer market sell-artifact w3",
        "0 answer market sell-artifact x3 6",
        "0 answer market sell-artifact w3 x",
        "0 answer market sell-artifact w3 6 1",
        "0 answer market leave now",
        "0 answer altar",
        "0 answer altar burn",
        "0 answer altar artifacts w2 w2",
        "0 answer altar artifacts w32",
        "0 answer altar artifacts b64",
        "0 answer altar artifacts x2",
        "0 answer altar artifacts wx",
        "0 answer altar artifacts w",
        "0 answer altar creatures 0 5 0 0 12 0",
        "0 answer altar creatures 0 5 0 0 12 0 1 1",
        "0 answer altar creatures 0 5 x 0 12 0 1",
        "0 answer altar leave now",
        "0 answer hill-fort",
        "0 answer hill-fort upgrade",
        "0 answer hill-fort upgrade x",
        "0 answer hill-fort upgrade 1 2",
        "0 answer hill-fort all now",
        "0 answer hill-fort leave now",
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

/// Since format v20 (E27 M4 T3a, CLT-Q156 (a)): the players' choices round-trip through the file
/// (a fixed faction, a random one or none; a pick or the draw's) and a game set up with them
/// verifies from its file; a file without them reads as the import's; a faction neither an index
/// nor `random` is malformed.
#[test]
fn the_players_choices_round_trip_through_the_file() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let ids = ["base", "ab", "sod"].map(String::from);
    let rules = load_rules(&root.join("packs"), &ids, "base:faithful").unwrap();
    let path = "fixtures/maps/e09-towns-36.h3m";
    let (map, import) = load_map(&root, path, &rules).unwrap();
    let red = PlayerId::new(0);
    let mut setup = import.clone();
    let faction = setup.players[0].start.factions.iter().last().unwrap();
    setup.players[0].start.choice = Some(FactionChoice::Fixed(faction));
    setup.players[0].start.pick = setup.offer(&rules, red).first().copied();
    assert!(setup.players[0].start.pick.is_some());
    let commands = vec![(Actor::Player(red), GameCommand::EndTurn)];
    let replay = Replay::record_commands(&rules, (&setup, map), 3, commands, 0).unwrap();
    let text = replay_to_toml(&replay).unwrap();
    let red_entry = format!(
        "[[players]]\ncolour = 0\nhuman = true\nfaction = {}\nhero = {}\n",
        faction.raw(),
        setup.players[0].start.pick.unwrap().raw()
    );
    assert!(text.contains(&red_entry), "{text}");
    assert_eq!(parse_replay(&text).unwrap(), replay);
    let dir = std::env::temp_dir().join(format!("h3-files-players-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("r.toml");
    std::fs::write(&file, &text).unwrap();
    let verified = verify_file(&file, &root.join("packs"), &root).unwrap();
    assert_eq!(verified.final_hash, replay.final_hash);
    std::fs::remove_dir_all(&dir).unwrap();
    // Every kind of choice round-trips, verified or not.
    let mut kinds = replay.clone();
    kinds.players[0].faction = Some(FactionChoice::Random);
    kinds.players[0].hero = None;
    kinds.players[1].faction = None;
    let text = replay_to_toml(&kinds).unwrap();
    assert!(text.contains("faction = \"random\"\n"), "{text}");
    assert_eq!(parse_replay(&text).unwrap(), kinds);
    // A file without them: the import's.
    let bare = &text[..text.find("[[players]]").unwrap()];
    assert!(parse_replay(bare).unwrap().players.is_empty());
    let e = parse_replay(&text.replacen("\"random\"", "\"any\"", 1)).unwrap_err();
    assert!(
        e.to_string().contains("bad faction `any` of colour 0"),
        "{e}"
    );
}
