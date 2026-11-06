//! Tests of the import of the towns, the players' starts, the random heroes, the heroes moved onto
//! a town's entrance and the hero pool's inputs (E09 T7a1, map import v16; design note
//! `docs/epics/E09-design.md` §7); since E09 T8a1 (map import v18, §8) the towns' type, owners and
//! formations.

use h3_adventure::{Holds, Pos as TilePos, TileMask};
use h3_content::{FactionId, Formation, HeroTypeId};
use h3_contracts::{HeroId, PlayerId, TownId};
use h3_game::{
    BuildingsSetup, FactionChoice, FactionSet, GameRuleset, GameSetup, GarrisonCreature,
    GarrisonStack, MainTown, SlotSet, TownEventSetup, TownFaction, TownSetup,
};

use super::import::{ROE, SOD, object, template};
use super::{limits, read, sample};
use crate::import::codes;
use crate::model::{
    Body, Buildings, CustomHero, DisposedHero, HeroBody, MainTown as MapMainTown, Player,
    PlayerDetails, Pos, RawMap, Stack, Text, Tile, TimedEvent, TownBody, TownEvent, Version,
};
use crate::{ImportError, Unsupported, import};

/// A town's template: its picture blocks five tiles of its row and three of the row above, its
/// entrance two columns left of its anchor is blocked and visited (the shipped towns' masks); the
/// property plans' towns too (E09 T9b1, [`super::play::TownPlan`]).
pub(super) fn town_template(class: u32, subclass: u32) -> crate::model::Template {
    crate::model::Template {
        subclass,
        ..template(
            class,
            &[
                (0, 0),
                (1, 0),
                (2, 0),
                (3, 0),
                (4, 0),
                (1, 1),
                (2, 1),
                (3, 1),
            ],
            &[(2, 0)],
        )
    }
}

/// A town's body owned by `owner` (`0xFF` neutral), standard without a Fort, no alignment.
fn town_body(owner: u8) -> Body {
    Body::Town(Box::new(TownBody {
        owner,
        alignment: 0xFF,
        ..TownBody::default()
    }))
}

/// A hero's body of `owner` and type code `hero_type`, its sex the type's.
fn hero_body(owner: u8, hero_type: u8) -> Body {
    Body::Hero(Box::new(HeroBody {
        owner,
        hero_type,
        sex: 0xFF,
        ..HeroBody::default()
    }))
}

/// An 8 × 8 map of `v` (red plays, the sample's eight factions allowed): templates 0 a Castle
/// (class 98), 1 a random town (77), 2 a hero (34), 3 a random hero (70), 4 a prison (62), each
/// hero visited one column left of its anchor; a red Castle anchored at (6, 4), its entrance
/// (4, 4).
fn town_map(v: Version) -> RawMap {
    let mut m = sample(v);
    m.header.size = 8;
    m.terrain.size = 8;
    m.terrain.tiles = vec![Tile::default(); 64];
    m.templates = vec![
        town_template(98, 0),
        town_template(77, 0),
        template(34, &[(1, 0)], &[(1, 0)]),
        template(70, &[(1, 0)], &[(1, 0)]),
        template(62, &[(1, 0)], &[(1, 0)]),
    ];
    m.objects = vec![object(6, 4, 0, town_body(0))];
    m
}

/// The import of `m` with the original packs' rules.
fn imported(m: &RawMap) -> GameSetup {
    import(m, &SOD).unwrap()
}

fn faction(rules: &GameRuleset, key: &str) -> FactionId {
    rules.core().faction_by_key(key).unwrap()
}

fn details(m: &mut RawMap) -> &mut PlayerDetails {
    m.players[0].details.as_mut().unwrap()
}

// covers: ADV-OBJ-002, ADV-OWN-002
#[test]
fn a_town_is_typed_with_its_owner_and_formation() {
    // E09 T8a1: the classes 98 and 77 the type `town`, the template's footprint (the entrance
    // blocked and visited), the owner the player of its colour (none for neutral), the garrison's
    // formation the map's byte (0 loose, any other tight).
    let mut m = town_map(Version::Sod);
    let tight = TownBody {
        owner: 0xFF,
        alignment: 0xFF,
        formation: 2,
        ..TownBody::default()
    };
    m.objects.push(object(6, 7, 1, Body::Town(Box::new(tight))));
    let setup = imported(&m);
    let town = SOD.adventure().object_type_by_key("base:town");
    let objects = &setup.map.objects;
    assert!(town.is_some());
    assert_eq!(
        (objects[0].object_type, objects[1].object_type),
        (town, town)
    );
    assert_eq!(objects[0].visitable, TileMask::from_bits(1 << 2));
    assert_eq!(
        objects[0].blocked,
        TileMask::from_bits(0b1_1111 | 0b1110 << 8)
    );
    assert_eq!(objects[0].owner, Some(PlayerId::new(0)));
    assert_eq!(
        objects[0].holds,
        Holds::Town {
            formation: Formation::Loose
        }
    );
    assert_eq!(objects[1].owner, None);
    assert_eq!(
        objects[1].holds,
        Holds::Town {
            formation: Formation::Tight
        }
    );
    // The game builds them: one record per town of the static map, red's main town its own.
    let rules = &*SOD;
    let state = h3_game::GameState::new(rules, &setup, 3).unwrap();
    assert_eq!(state.adventure().map().towns(), [0, 1]);
    // A town of a colour nobody plays is refused, as every ownable object's.
    m.objects[1] = object(6, 7, 1, town_body(3));
    assert_eq!(import(&m, &SOD), Err(ImportError::ObjectOwner(1)));
}

// covers: TWN-SET-001
#[test]
fn a_players_factions_are_its_allowed_towns_the_rules_have() {
    let keys = [
        "base:castle",
        "base:rampart",
        "base:tower",
        "base:inferno",
        "base:necropolis",
        "base:dungeon",
        "base:stronghold",
        "base:fortress",
        "ab:conflux",
    ];
    let set = |rules: &GameRuleset, keys: &[&str]| {
        keys.iter()
            .map(|key| faction(rules, key))
            .fold(FactionSet::EMPTY, FactionSet::with)
    };
    let start = |v, allowed, rules: &GameRuleset| {
        let mut m = town_map(v);
        details(&mut m).allowed_towns = allowed;
        import(&m, rules).unwrap().players[0].start
    };
    // Several: Random; an SoD map's eight bits are eight factions (decision 14: only an
    // RoE-format map's add Conflux).
    let eight = start(Version::Sod, 0xFF, &SOD);
    assert_eq!(eight.factions, set(&SOD, &keys[..8]));
    assert_eq!(eight.choice, Some(FactionChoice::Random));
    assert_eq!(
        (eight.pick, eight.main_town, eight.random_hero),
        (None, None, false)
    );
    assert_eq!(start(Version::Ab, 0x1FF, &SOD).factions, set(&SOD, &keys));
    // An RoE-format map's player with all eight: Conflux too when the rules have it.
    assert_eq!(start(Version::RoE, 0xFF, &SOD).factions, set(&SOD, &keys));
    assert_eq!(
        start(Version::RoE, 0xFF, &ROE).factions,
        set(&ROE, &keys[..8])
    );
    assert_eq!(
        start(Version::RoE, 0x7F, &SOD).factions,
        set(&SOD, &keys[..7])
    );
    // Exactly one: Fixed.
    let castle = faction(&SOD, "base:castle");
    let one = start(Version::Sod, 0x01, &SOD);
    assert_eq!(
        (one.factions, one.choice),
        (set(&SOD, &keys[..1]), Some(FactionChoice::Fixed(castle)))
    );
    let conflux = start(Version::Sod, 0x100, &SOD);
    assert_eq!(
        conflux.choice,
        Some(FactionChoice::Fixed(faction(&SOD, "ab:conflux")))
    );
    // None (Conflux without its rules, no bit, bits past the factions): Random, which the game
    // refuses (E09 T7a2).
    for (allowed, rules) in [(0x100, &*ROE), (0, &*SOD), (0xFE00, &*SOD)] {
        let none = start(Version::Sod, allowed, rules);
        assert_eq!(
            (none.factions, none.choice),
            (FactionSet::EMPTY, Some(FactionChoice::Random))
        );
    }
    // The random-hero flag is the map's; a playing colour without details starts with none.
    let mut m = town_map(Version::Sod);
    details(&mut m).random_hero = true;
    m.players[1] = Player {
        human: false,
        computer: true,
        details: None,
    };
    let setup = imported(&m);
    assert!(setup.players[0].start.random_hero);
    assert_eq!(setup.players[1].start, h3_game::PlayerStart::default());
}

// covers: TWN-SET-001
#[test]
fn a_main_town_is_the_town_whose_entrance_the_header_names() {
    let mut m = town_map(Version::Sod);
    // A second town, random, anchored at (6, 7): its entrance (4, 7).
    m.objects.push(object(6, 7, 1, town_body(0)));
    for (pos, town) in [((4, 4), 0), ((4, 7), 1)] {
        for generate_hero in [false, true] {
            details(&mut m).main_town = Some(MapMainTown {
                generate_hero,
                town_type: 0xFF,
                pos: Pos {
                    x: pos.0,
                    y: pos.1,
                    z: 0,
                },
            });
            assert_eq!(
                imported(&m).players[0].start.main_town,
                Some(MainTown {
                    town: TownId::new(town),
                    generates_hero: generate_hero,
                })
            );
        }
    }
    // A position that is no town's entrance (the anchor, a blocked tile): typed (T-4).
    for pos in [(6, 4), (5, 4), (4, 5)] {
        details(&mut m).main_town = Some(MapMainTown {
            generate_hero: true,
            town_type: 0,
            pos: Pos {
                x: pos.0,
                y: pos.1,
                z: 0,
            },
        });
        assert_eq!(import(&m, &SOD), Err(ImportError::MainTown(0)));
    }
}

#[test]
fn a_fixed_towns_faction_is_its_subclasss() {
    let keys = [
        "base:castle",
        "base:rampart",
        "base:tower",
        "base:inferno",
        "base:necropolis",
        "base:dungeon",
        "base:stronghold",
        "base:fortress",
        "ab:conflux",
    ];
    let mut m = town_map(Version::Sod);
    for (subclass, key) in (0..).zip(keys) {
        m.templates[0].subclass = subclass;
        assert_eq!(
            imported(&m).towns[0].faction,
            TownFaction::Fixed(faction(&SOD, key))
        );
    }
    m.templates[0].subclass = 9;
    assert_eq!(import(&m, &SOD), Err(ImportError::Subclass(0)));
    m.templates[0].subclass = 8;
    assert_eq!(
        import(&m, &ROE),
        Err(ImportError::MissingContent("ab:conflux"))
    );
}

#[test]
fn a_random_towns_alignment_is_a_colour_or_none() {
    let mut m = town_map(Version::Sod);
    m.objects.push(object(6, 7, 1, town_body(0xFF)));
    let alignment = |m: &mut RawMap, byte| {
        if let Body::Town(town) = &mut m.objects[1].body {
            town.alignment = byte;
        }
        import(m, &SOD).map(|setup| setup.towns[1].faction)
    };
    assert_eq!(
        alignment(&mut m, 0xFF),
        Ok(TownFaction::Random { alignment: None })
    );
    for colour in [0, 3, 7] {
        assert_eq!(
            alignment(&mut m, colour),
            Ok(TownFaction::Random {
                alignment: Some(colour)
            })
        );
    }
    // T-1: refused (TWN-QUIRK-1).
    for byte in [8, 127, 128, 254] {
        assert_eq!(alignment(&mut m, byte), Err(ImportError::Alignment(1)));
    }
}

#[test]
fn a_towns_buildings_garrison_name_and_events_are_the_maps() {
    let mut m = town_map(Version::Sod);
    let mut army = vec![None; 7];
    army[0] = Some(Stack {
        creature: 0,
        count: 5,
    });
    // Random stacks: −2 (level 1), −3 (level 1 upgraded), −15 (level 7 upgraded).
    army[1] = Some(Stack {
        creature: u16::MAX - 1,
        count: 3,
    });
    army[2] = Some(Stack {
        creature: u16::MAX - 2,
        count: 2,
    });
    army[4] = Some(Stack {
        creature: u16::MAX - 14,
        count: 1,
    });
    // A count of 0 is an empty slot, whatever its code.
    army[6] = Some(Stack {
        creature: 9999,
        count: 0,
    });
    let event = TownEvent {
        event: TimedEvent {
            resources: [0, 0, 0, 0, 0, 0, 500],
            players: 0b101,
            human: true,
            computer: false,
            first_day: 4,
            interval: 7,
            ..TimedEvent::default()
        },
        buildings: [0x01, 0, 0, 0, 0, 0x01],
        creatures: [1, 2, 3, 4, 5, 6, 7],
    };
    m.objects[0].body = Body::Town(Box::new(TownBody {
        owner: 0,
        name: Some(Text(b"Caf\xe9 \x80\x81".to_vec())),
        garrison: Some(army),
        buildings: Buildings::Custom {
            built: [0x58, 0, 0x04, 0, 0, 0x01],
            forbidden: [0, 0x80, 0, 0, 0, 0xFF],
        },
        events: vec![event],
        alignment: 0xFF,
        ..TownBody::default()
    }));
    let town = imported(&m).towns.remove(0);
    let core = SOD.core();
    let gold = core
        .resources()
        .iter()
        .position(|r| r.key.as_str() == "base:gold")
        .and_then(h3_content::ResourceId::from_index)
        .unwrap();
    let stack = |slot, creature, count| GarrisonStack {
        slot,
        creature,
        count,
    };
    let random = |level, upgraded| GarrisonCreature::Random { level, upgraded };
    assert_eq!(
        town,
        TownSetup {
            faction: TownFaction::Fixed(faction(&SOD, "base:castle")),
            buildings: BuildingsSetup::Custom {
                built: SlotSet::from_bits(0x58 | 0x04 << 16 | 1 << 40),
                forbidden: SlotSet::from_bits(0x80 << 8 | 0x01 << 40),
            },
            garrison: Some(vec![
                stack(
                    0,
                    GarrisonCreature::Creature(core.creature_by_key("base:pikeman").unwrap()),
                    5
                ),
                stack(1, random(1, false), 3),
                stack(2, random(1, true), 2),
                stack(4, random(7, true), 1),
            ]),
            name: Some("Caf\u{e9} \u{20ac}\u{81}".to_owned()),
            events: vec![TownEventSetup {
                players: vec![PlayerId::new(0)],
                human: true,
                computer: false,
                first_day: 5,
                interval: 7,
                resources: vec![(gold, 500)],
                buildings: SlotSet::from_bits(1 | 1 << 40),
                creatures: [1, 2, 3, 4, 5, 6, 7],
            }],
        }
    );
    // RoE's one-byte codes: 0xFE level 1, 0xF1 level 7 upgraded.
    let mut m = town_map(Version::RoE);
    let mut army = vec![None; 7];
    army[3] = Some(Stack {
        creature: 0xFE,
        count: 1,
    });
    army[5] = Some(Stack {
        creature: 0xF1,
        count: 1,
    });
    m.objects[0].body = Body::Town(Box::new(TownBody {
        garrison: Some(army),
        buildings: Buildings::Standard { fort: true },
        alignment: 0xFF,
        ..TownBody::default()
    }));
    let town = imported(&m).towns.remove(0);
    assert_eq!(town.buildings, BuildingsSetup::Standard { fort: true });
    assert_eq!(
        town.garrison,
        Some(vec![
            stack(3, random(1, false), 1),
            stack(5, random(7, true), 1)
        ])
    );
    // An empty custom garrison stays one; an unknown code with a count is refused.
    for (army, expected) in [
        (vec![None; 7], Ok(Some(vec![]))),
        (
            vec![Some(Stack {
                creature: 9999,
                count: 1,
            })],
            Err(ImportError::UnknownCode {
                what: "creature",
                code: 9999,
            }),
        ),
    ] {
        let mut m = town_map(Version::Sod);
        if let Body::Town(town) = &mut m.objects[0].body {
            town.garrison = Some(army);
        }
        assert_eq!(
            import(&m, &SOD).map(|mut s| s.towns.remove(0).garrison),
            expected
        );
    }
    // An event's resource the rules lack is refused (an AB map's with RoE's rules has none: the
    // resources are RoE's), as the map's timed events' are; a town without a town's body (never
    // the reader's) is a neutral town's defaults.
    let mut m = town_map(Version::Sod);
    m.objects[0].body = Body::None;
    m.objects.push(object(6, 7, 1, Body::None));
    let setup = imported(&m);
    assert_eq!(
        setup.towns,
        [
            TownSetup {
                faction: TownFaction::Fixed(faction(&SOD, "base:castle")),
                buildings: BuildingsSetup::Standard { fort: false },
                garrison: None,
                name: None,
                events: vec![],
            },
            TownSetup {
                faction: TownFaction::Random { alignment: None },
                buildings: BuildingsSetup::Standard { fort: false },
                garrison: None,
                name: None,
                events: vec![],
            },
        ]
    );
}

// covers: TWN-SET-010
#[test]
fn a_hero_right_of_a_towns_entrance_moves_onto_it() {
    // A placed hero and a random hero right of the entrances (5, 4) and (5, 7); a hero right of
    // an entrance it stands on stays (left of it the town's blocked tile).
    let mut m = town_map(Version::Sod);
    m.objects.push(object(6, 7, 1, town_body(0)));
    m.objects.push(object(6, 4, 2, hero_body(0, 0)));
    m.objects.push(object(6, 7, 3, hero_body(0, 0xFF)));
    let setup = imported(&m);
    let tiles: Vec<TilePos> = setup.heroes.iter().map(|h| h.pos).collect();
    assert_eq!(tiles, [TilePos::new(4, 4, 0), TilePos::new(4, 7, 0)]);
    // On the entrance itself: stays there (a town's visitor).
    let mut m = town_map(Version::Sod);
    m.objects.push(object(5, 4, 2, hero_body(0, 0)));
    assert_eq!(imported(&m).heroes[0].pos, TilePos::new(4, 4, 0));
    // Any other blocked tile of a town: unsupported (the original keeps it there; none on the
    // shipped maps).
    for (x, y) in [(7, 4), (4, 3), (5, 3)] {
        let mut m = town_map(Version::Sod);
        m.objects.push(object(x, y, 2, hero_body(0, 0)));
        assert_eq!(
            import(&m, &SOD),
            Err(ImportError::Unsupported(Unsupported::HeroInTown(1)))
        );
    }
    // A prisoner right of an entrance stays a blocker where it is, out of the pool.
    let mut m = town_map(Version::Sod);
    m.objects.push(object(6, 4, 4, hero_body(0xFF, 7)));
    let setup = imported(&m);
    assert!(setup.heroes.is_empty());
    assert_eq!(setup.map.objects[1].anchor, TilePos::new(6, 4, 0));
    let hero_type = |code: usize| {
        SOD.core()
            .hero_type_by_key(codes::HERO_TYPES[code])
            .unwrap()
    };
    assert_eq!(setup.prisoners, [hero_type(7)]);
}

#[test]
fn a_random_hero_is_a_placed_hero_without_its_type() {
    let mut m = town_map(Version::Sod);
    // The map-wide entry of type 0 is not a random hero's.
    m.custom_heroes = vec![CustomHero {
        hero_type: 0,
        experience: Some(5000),
        sex: 0xFF,
        ..CustomHero::default()
    }];
    let mut army = vec![None; 7];
    army[1] = Some(Stack {
        creature: 0,
        count: 12,
    });
    m.objects.push(object(
        2,
        1,
        3,
        Body::Hero(Box::new(HeroBody {
            owner: 0,
            hero_type: 0,
            experience: Some(1234),
            army: Some(army),
            formation: 1,
            sex: 0xFF,
            ..HeroBody::default()
        })),
    ));
    let setup = imported(&m);
    let [hero] = setup.heroes.as_slice() else {
        panic!("one hero expected");
    };
    assert_eq!(hero.hero_type, None);
    assert_eq!(hero.owner, PlayerId::new(0));
    assert_eq!(hero.pos, TilePos::new(1, 1, 0));
    assert_eq!(hero.formation, Formation::Tight);
    assert_eq!(hero.details.experience, Some(1234));
    assert!(matches!(&hero.army, h3_game::HeroArmy::Custom(stacks) if stacks.len() == 1));
    // The build draws its type (TWN-SET-008, E09 T7b1): one the map allows, taken by its owner.
    let game = h3_game::GameState::new(&SOD, &setup, 1).unwrap();
    let drawn = game.hero(HeroId::new(0)).unwrap().hero_type;
    assert!(setup.allowed_heroes.contains(&drawn));
    // AB's spell byte beside a random hero's type: the spell alone (the type's own unknown).
    let mut m = town_map(Version::Ab);
    m.objects.push(object(
        2,
        1,
        3,
        Body::Hero(Box::new(HeroBody {
            owner: 0,
            hero_type: 0xFF,
            spell: Some(0),
            sex: 0xFF,
            ..HeroBody::default()
        })),
    ));
    let spells = imported(&m).heroes[0].details.spells.clone();
    assert_eq!(spells.map(|s| s.len()), Some(1));
}

// covers: TWN-SET-009
#[test]
fn a_main_town_generating_a_hero_gets_one_on_its_entrance() {
    let mut m = town_map(Version::Sod);
    details(&mut m).main_town = Some(MapMainTown {
        generate_hero: false,
        town_type: 0,
        pos: Pos { x: 4, y: 4, z: 0 },
    });
    assert_eq!(imported(&m).validate(&SOD), Ok(()));
    // E09 T7b2b: the build draws the hero's type over the player's faction and stands it on the
    // town's entrance, its id after the map's heroes.
    details(&mut m).main_town.as_mut().unwrap().generate_hero = true;
    let setup = imported(&m);
    let game = h3_game::GameState::new(&SOD, &setup, 1).unwrap();
    let id = HeroId::new(u16::try_from(setup.heroes.len()).unwrap());
    let hero = game.adventure().hero(id).unwrap();
    assert_eq!(
        (hero.owner, hero.pos),
        (PlayerId::new(0), TilePos::new(4, 4, 0))
    );
    assert!(
        setup
            .allowed_heroes
            .contains(&game.hero(id).unwrap().hero_type)
    );
}

#[test]
fn the_pools_inputs_are_the_maps_allowed_heroes_masks_and_prisoners() {
    let types = |rules: &GameRuleset, codes: &mut dyn Iterator<Item = usize>| -> Vec<HeroTypeId> {
        codes
            .map(|code| {
                rules
                    .core()
                    .hero_type_by_key(codes::HERO_TYPES[code])
                    .unwrap()
            })
            .collect()
    };
    // An SoD map's 156 bits (the four past them unread); with RoE's rules the 128 RoE heroes.
    let m = town_map(Version::Sod);
    assert_eq!(imported(&m).allowed_heroes, types(&SOD, &mut (0..156)));
    assert_eq!(
        import(&m, &ROE).unwrap().allowed_heroes,
        types(&ROE, &mut (0..128))
    );
    // An RoE map's 128 bits and the 16 Conflux heroes when the rules have them, never the 12
    // campaign heroes.
    let mut m = town_map(Version::RoE);
    m.allowed_heroes.0[0] = 0b1111_1110;
    assert_eq!(imported(&m).allowed_heroes, types(&SOD, &mut (1..144)));
    assert_eq!(
        import(&m, &ROE).unwrap().allowed_heroes,
        types(&ROE, &mut (1..128))
    );
    // SoD's disposed heroes: each type with its mask in the map's order, a type the rules lack
    // left out; an unknown code refused.
    let mut m = town_map(Version::Sod);
    let disposed = |hero_type, players| DisposedHero {
        hero_type,
        players,
        ..DisposedHero::default()
    };
    m.disposed_heroes = vec![disposed(9, 0b11), disposed(130, 0x80), disposed(2, 0xFF)];
    let masks = |rules: &GameRuleset, entries: &[(usize, u8)]| -> Vec<(HeroTypeId, u8)> {
        entries
            .iter()
            .map(|&(code, mask)| (types(rules, &mut [code].into_iter())[0], mask))
            .collect()
    };
    assert_eq!(
        imported(&m).hero_masks,
        masks(&SOD, &[(9, 0b11), (130, 0x80), (2, 0xFF)])
    );
    assert_eq!(
        import(&m, &ROE).unwrap().hero_masks,
        masks(&ROE, &[(9, 0b11), (2, 0xFF)])
    );
    m.disposed_heroes.push(disposed(200, 1));
    let unknown = Err(ImportError::UnknownCode {
        what: "hero type",
        code: 200,
    });
    assert_eq!(import(&m, &SOD).map(|s| s.hero_masks), unknown);
    // The prisoners' types in the map's order (a prison without a hero's body has none); a type
    // the rules lack left out, an unknown code refused.
    let mut m = town_map(Version::Sod);
    m.objects.push(object(2, 1, 4, hero_body(0xFF, 140)));
    m.objects.push(object(2, 3, 4, Body::None));
    m.objects.push(object(2, 6, 4, hero_body(0xFF, 3)));
    assert_eq!(
        imported(&m).prisoners,
        types(&SOD, &mut [140, 3].into_iter())
    );
    assert_eq!(
        import(&m, &ROE).unwrap().prisoners,
        types(&ROE, &mut [3].into_iter())
    );
    m.objects.push(object(4, 1, 4, hero_body(0xFF, 200)));
    assert_eq!(
        import(&m, &SOD).map(|s| s.prisoners),
        unknown.map(|_: Vec<_>| vec![])
    );
}

/// The experiment writer's town (E04c T5b), read back: its faction, buildings and garrison, and a
/// hero of its owner right of its entrance moved onto it (TWN-SET-010).
#[test]
fn a_town_from_the_writer_imports_with_its_visitor() {
    use crate::writer as w;
    let mut players: [Option<w::Player>; w::PLAYERS] = Default::default();
    players[0] = Some(w::Player {
        human: true,
        computer: true,
        towns: 1 << 6,
    });
    let town = w::Town {
        tile: (12, 9),
        template: w::ObjectTemplate::from_objects_txt(
            "AVCstrx0.def 000001110000011110001111111111111111111111111111 \
             001000000000000000000000000000000000000000000000 011111111 011111111 98 6 1 0",
        )
        .unwrap(),
        owner: Some(0),
        garrison: Some([Some((86, 30)), None, None, None, None, None, None]),
        tight_formation: false,
        buildings: w::TownBuildings::Custom {
            built: [0x58, 0, 0, 0, 0, 0],
            forbidden: [0, 0, 0, 0, 0, 0x01],
        },
    };
    let map = w::Map {
        name: "Visitor".to_owned(),
        description: String::new(),
        size: 36,
        terrain: w::Terrain::Rough,
        patches: vec![],
        roads: vec![],
        players,
        objects: vec![
            w::MapObject::Town(town),
            w::MapObject::Hero(w::Hero {
                tile: (11, 9),
                owner: 0,
                hero_type: 0,
                class: 0,
                name: None,
                experience: None,
                skills: None,
                army: None,
                tight_formation: false,
                artifacts: None,
                spells: None,
                primary: None,
            }),
        ],
        underground: None,
        level_limit: 0,
        banned_skills: Vec::new(),
        events: Vec::new(),
        teams: None,
    };
    let m = read(&w::gzip(&w::write(&map).unwrap()), &limits()).unwrap();
    let setup = imported(&m);
    let stronghold = faction(&SOD, "base:stronghold");
    assert_eq!(
        setup.players[0].start.choice,
        Some(FactionChoice::Fixed(stronghold))
    );
    let [town] = setup.towns.as_slice() else {
        panic!("one town expected");
    };
    assert_eq!(town.faction, TownFaction::Fixed(stronghold));
    assert_eq!(
        town.buildings,
        BuildingsSetup::Custom {
            built: SlotSet::from_bits(0x58),
            forbidden: SlotSet::from_bits(1 << 40),
        }
    );
    let wolf_rider = SOD.core().creature_by_key("base:wolf_rider").unwrap();
    assert_eq!(
        town.garrison,
        Some(vec![GarrisonStack {
            slot: 0,
            creature: GarrisonCreature::Creature(wolf_rider),
            count: 30,
        }])
    );
    assert_eq!(setup.heroes[0].pos, TilePos::new(10, 9, 0));
    assert_eq!(setup.validate(&SOD), Ok(()));
}
