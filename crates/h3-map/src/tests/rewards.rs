//! The experiment writer's reward objects, monster treasures and RoE maps (E08 T56: batch E7's
//! maps): each reads back as written, and the map's static layer imports and validates.

use h3_game::{DEFAULT_DIFFICULTY, GameRuleset, GameSetup, PlayerSetup};

use super::import::{ROE, SOD};
use super::limits;
use crate::model::{
    Bits, Body, EventBody, Message, MonsterTreasure, RawMap, Rewards, Stack, Text, Version,
};
use crate::writer as w;
use crate::{import, import_map, read};
use h3_adventure::{ArtifactInstance, EventSetup, Guards, HeroReward, Holds, RewardRecord};
use h3_content::ResourceId;
use h3_contracts::ArmyStack;

fn template(class: u32, subclass: u32) -> w::ObjectTemplate {
    w::ObjectTemplate {
        def: format!("o{class}s{subclass}.def"),
        passable: [0xff, 0xff, 0xff, 0xff, 0xff, 0x7f],
        visitable: [0, 0, 0, 0, 0, 0x80],
        terrains: 0x1ff,
        menu: 0x1ff,
        class,
        subclass,
        group: 0,
        priority: 0,
    }
}

fn reward(tile: (u8, u8), class: u32, subclass: u32, body: w::RewardBody) -> w::MapObject {
    w::MapObject::Reward(w::RewardObject {
        tile,
        template: template(class, subclass),
        body,
    })
}

fn hero(hero_type: u8, artifacts: Option<w::HeroArtifacts>) -> w::MapObject {
    w::MapObject::Hero(w::Hero {
        tile: (2, 2),
        owner: 0,
        hero_type,
        class: 0,
        name: Some("E7".into()),
        experience: None,
        skills: Some(vec![(7, 1)]),
        army: Some([Some((1, 5)), None, None, None, None, None, None]),
        tight_formation: false,
        artifacts,
        spells: None,
        primary: None,
    })
}

fn monster(tile: (u8, u8), creature: u32, artifact: Option<u16>) -> w::MapObject {
    w::MapObject::Monster(w::Monster {
        tile,
        template: template(54, creature),
        count: 4,
        disposition: 2,
        never_flees: true,
        fixed_count: false,
        treasure: Some(w::Treasure {
            text: "Loot".into(),
            resources: [0, 0, 0, 0, 0, 0, 1000],
            artifact,
        }),
    })
}

fn map(objects: Vec<w::MapObject>) -> w::Map {
    let mut players: [Option<w::Player>; w::PLAYERS] = Default::default();
    players[0] = Some(w::Player {
        human: true,
        computer: true,
        towns: 1,
    });
    w::Map {
        name: "Rewards".into(),
        description: "E08 T56".into(),
        size: 36,
        terrain: w::Terrain::Grass,
        patches: vec![],
        roads: vec![],
        players,
        objects,
        underground: None,
        level_limit: 0,
        banned_skills: Vec::new(),
    }
}

fn guards() -> [Option<(u16, u16)>; w::ARMY_SLOTS] {
    [None, Some((0, 12)), None, None, None, None, Some((1, 3))]
}

fn raw_guards() -> Option<Vec<Option<Stack>>> {
    Some(
        guards()
            .iter()
            .map(|s| s.map(|(creature, count)| Stack { creature, count }))
            .collect(),
    )
}

fn rewards() -> Rewards {
    Rewards {
        experience: 1000,
        mana: 500,
        morale: 1,
        luck: -1,
        resources: [1, 0, 0, 0, 0, 0, 500],
        primary: [1, 0, 0, 2],
        skills: vec![(7, 2)],
        artifacts: vec![7],
        spells: vec![15],
        creatures: vec![Stack {
            creature: 0,
            count: 5,
        }],
    }
}

/// The map's static layer in a game of one player without heroes, validated (as the original
/// data's check does).
fn static_layer_validates(m: &RawMap, rules: &GameRuleset) {
    let setup = GameSetup {
        players: vec![PlayerSetup {
            colour: 0,
            human: true,
        }],
        difficulty: DEFAULT_DIFFICULTY,
        map: import_map(m, rules).unwrap(),
        heroes: Vec::new(),
        level_limit: 0,
        banned_skills: Vec::new(),
    };
    setup.validate(rules).unwrap();
}

#[test]
fn the_writer_s_reward_objects_read_back_and_import() {
    let message = |text: &str, guarded: bool| w::Message {
        text: text.into(),
        guards: guarded.then(guards),
    };
    let mut objects = vec![
        hero(0, None),
        reward(
            (5, 5),
            5,
            7,
            w::RewardBody::Artifact(Some(message("", true))),
        ),
        reward((7, 5), 65, 0, w::RewardBody::Artifact(None)),
        reward(
            (9, 5),
            93,
            0,
            w::RewardBody::Scroll {
                message: None,
                spell: 15,
            },
        ),
        reward((11, 5), 88, 0, w::RewardBody::Shrine(None)),
        reward((13, 5), 89, 0, w::RewardBody::Shrine(Some(20))),
        reward((15, 5), 113, 0, w::RewardBody::WitchHut(None)),
        reward((17, 5), 113, 0, w::RewardBody::WitchHut(Some([1, 0, 0, 8]))),
        reward(
            (19, 5),
            6,
            0,
            w::RewardBody::Pandora {
                message: Some(message("Open?", false)),
                rewards: rewards(),
            },
        ),
        reward(
            (21, 5),
            26,
            0,
            w::RewardBody::Event(w::Event {
                message: Some(message("Fight?", true)),
                rewards: Rewards {
                    resources: [0, 0, 0, 0, 0, 0, 100],
                    ..Rewards::default()
                },
                players: 0b11,
                computer: false,
                removed_after_visit: true,
            }),
        ),
        monster((23, 5), 0, Some(7)),
    ];
    for (i, class) in [22, 63, 100, 101, 104, 105, 108].into_iter().enumerate() {
        objects.push(w::MapObject::Plain(w::Plain {
            tile: (2 + 3 * u8::try_from(i).unwrap(), 12),
            template: template(class, 0),
        }));
    }
    let written = map(objects);
    let m = read(&w::gzip(&w::write(&written).unwrap()), &limits()).unwrap();
    let raw_message = |text: &[u8], guarded: bool| Message {
        text: Text(text.to_vec()),
        guards: guarded.then(raw_guards).flatten(),
    };
    let bodies: Vec<&Body> = m.objects.iter().map(|o| &o.body).collect();
    assert_eq!(*bodies[1], Body::Artifact(Some(raw_message(b"", true))));
    assert_eq!(*bodies[2], Body::Artifact(None));
    assert_eq!(
        *bodies[3],
        Body::Scroll {
            message: None,
            spell: 15
        }
    );
    assert_eq!(*bodies[4], Body::Shrine { spell: 0xff });
    assert_eq!(*bodies[5], Body::Shrine { spell: 20 });
    assert_eq!(
        *bodies[6],
        Body::WitchHut(Bits(w::WITCH_HUT_SKILLS.to_vec()))
    );
    assert_eq!(*bodies[7], Body::WitchHut(Bits(vec![1, 0, 0, 8])));
    assert_eq!(
        *bodies[8],
        Body::Pandora {
            message: Some(raw_message(b"Open?", false)),
            rewards: rewards(),
        }
    );
    assert_eq!(
        *bodies[9],
        Body::Event(EventBody {
            message: Some(raw_message(b"Fight?", true)),
            rewards: Rewards {
                resources: [0, 0, 0, 0, 0, 0, 100],
                ..Rewards::default()
            },
            players: 0b11,
            computer: false,
            remove_after_visit: true,
        })
    );
    let Body::Monster(beaten) = bodies[10] else {
        panic!("not a monster: {:?}", bodies[10])
    };
    assert_eq!(
        beaten.treasure,
        Some(MonsterTreasure {
            text: Text(b"Loot".to_vec()),
            resources: [0, 0, 0, 0, 0, 0, 1000],
            artifact: Some(7),
        })
    );
    assert!(bodies[11..].iter().all(|b| **b == Body::None));
    let classes: Vec<u32> = m.objects[11..]
        .iter()
        .map(|o| m.templates[usize::try_from(o.template).unwrap()].class)
        .collect();
    assert_eq!(classes, [22, 63, 100, 101, 104, 105, 108]);
    // The static layer imports and validates; since E08 T61a the whole map too, the monster's
    // artifact with it, each object typed with what it holds (design §7).
    static_layer_validates(&m, &SOD);
    let setup = import(&m, &SOD).unwrap();
    setup.validate(&SOD).unwrap();
    let core = SOD.core();
    let part = SOD.adventure();
    let objects = &setup.map.objects;
    let key = |i: usize| {
        objects[i]
            .object_type
            .and_then(|id| part.object_type(id))
            .map(|t| t.key.as_str())
    };
    let artifact = |key: &str| core.artifact_by_key(key).unwrap();
    let spell = |key: &str| core.spell_by_key(key).unwrap();
    let skill = |key: &str| core.skill_by_key(key).unwrap();
    let axe = artifact("base:centaurs_axe");
    // The hero is no object of the static map: the artifact is the first.
    assert_eq!(key(0), Some("base:artifact"));
    assert_eq!(objects[0].holds, Holds::Artifact(axe.into()));
    // D-29: a guarded artifact asks whatever its text.
    let stacks = |question| {
        Some(Guards {
            army: vec![
                ArmyStack {
                    slot: 1,
                    creature: core.creature_by_key("base:pikeman").unwrap(),
                    count: 12,
                },
                ArmyStack {
                    slot: 6,
                    creature: core.creature_by_key("base:halberdier").unwrap(),
                    count: 3,
                },
            ],
            question,
        })
    };
    assert_eq!(objects[0].guards, stacks(true));
    assert_eq!(
        (key(1), &objects[1].holds, &objects[1].guards),
        (Some("base:random_artifact"), &Holds::Nothing, &None)
    );
    let scroll = ArtifactInstance::scroll(artifact("base:spell_scroll"), spell("base:magic_arrow"));
    assert_eq!(
        (key(2), &objects[2].holds),
        (Some("base:spell_scroll"), &Holds::Artifact(scroll))
    );
    assert_eq!(objects[3].holds, Holds::Spell(None));
    assert_eq!(
        (key(4), &objects[4].holds),
        (
            Some("base:shrine_of_magic_gesture"),
            &Holds::Spell(Some(spell("base:frost_ring")))
        )
    );
    // The editor's default mask (all but Leadership and Necromancy), and bits 0 and 27.
    let Holds::Skills(Some(mask)) = &objects[5].holds else {
        panic!("{:?}", objects[5].holds)
    };
    assert_eq!(mask.len(), 26);
    assert!(!mask.contains(&skill("base:leadership")) && !mask.contains(&skill("base:necromancy")));
    assert_eq!(
        objects[6].holds,
        Holds::Skills(Some(vec![
            skill("base:pathfinding"),
            skill("base:first_aid")
        ]))
    );
    let resource = |i: usize| ResourceId::from_index(i).unwrap();
    let record = RewardRecord {
        hero: HeroReward {
            experience: 1000,
            primary: [1, 0, 0, 2],
            skills: vec![(skill("base:wisdom"), 2)],
            mana: 500,
            morale: 1,
            luck: -1,
            artifacts: vec![axe],
            spells: vec![spell("base:magic_arrow")],
        },
        resources: vec![(resource(0), 1), (resource(6), 500)],
        creatures: vec![ArmyStack {
            slot: 0,
            creature: core.creature_by_key("base:pikeman").unwrap(),
            count: 5,
        }],
    };
    assert_eq!(
        (key(7), &objects[7].holds, &objects[7].guards),
        (
            Some("base:pandoras_box"),
            &Holds::Record(Box::new(record)),
            &None
        )
    );
    // An event's guards are announced, never asked.
    assert_eq!(
        objects[8].holds,
        Holds::Event(Box::new(EventSetup {
            record: RewardRecord {
                resources: vec![(resource(6), 100)],
                ..RewardRecord::default()
            },
            colours: 0b11,
            computer: false,
            remove: true,
        }))
    );
    assert_eq!(objects[8].guards, stacks(false));
    assert_eq!(objects[9].monster.as_ref().unwrap().artifact, Some(axe));
    let plain = [
        "corpse",
        "pyramid",
        "learning_stone",
        "treasure_chest",
        "university",
        "wagon",
        "warriors_tomb",
    ];
    for (i, id) in (10..).zip(plain) {
        assert_eq!(key(i), Some(format!("base:{id}").as_str()));
        assert_eq!(objects[i].holds, Holds::Nothing);
    }
}

#[test]
fn the_writer_s_roe_maps_read_back_and_import() {
    let mut worn = [None; w::ARTIFACT_SLOTS];
    worn[3] = Some(7);
    let artifacts = w::HeroArtifacts {
        worn,
        backpack: vec![8, 126],
    };
    let pile = w::MapObject::Pile(w::Pile {
        tile: (9, 9),
        template: template(79, 6),
        amount: 0,
        guards: Some(w::Guards {
            army: guards(),
            question: "Q".into(),
        }),
    });
    let written = map(vec![
        hero(1, Some(artifacts)),
        monster((5, 5), 117, Some(126)),
        reward((7, 5), 113, 0, w::RewardBody::WitchHut(None)),
        reward((9, 5), 113, 0, w::RewardBody::WitchHut(None)),
        reward(
            (11, 5),
            6,
            0,
            w::RewardBody::Pandora {
                message: None,
                rewards: Rewards {
                    artifacts: vec![126],
                    creatures: vec![Stack {
                        creature: 117,
                        count: 2,
                    }],
                    ..Rewards::default()
                },
            },
        ),
        pile,
    ]);
    let m = read(
        &w::gzip(&w::write_in(&written, w::Format::RoE).unwrap()),
        &limits(),
    )
    .unwrap();
    assert_eq!((m.header.version, m.header.level_limit), (Version::RoE, 0));
    assert_eq!(m.allowed_heroes, Bits(vec![0xff; 16]));
    let red = m.players[0].details.as_ref().unwrap();
    assert_eq!(red.allowed_towns, 1);
    assert_eq!(red.main_hero.as_ref().map(|h| h.hero_type), Some(1));
    assert!(m.players[1].details.is_none());
    let Body::Hero(placed) = &m.objects[0].body else {
        panic!("not a hero: {:?}", m.objects[0].body)
    };
    assert_eq!(
        (placed.identifier, placed.experience, placed.spells.as_ref()),
        (None, Some(0), None)
    );
    assert_eq!(placed.name, Some(Text(b"E7".to_vec())));
    let carried = placed.artifacts.as_ref().unwrap();
    assert_eq!(carried.worn.len(), 18);
    assert_eq!(carried.worn[3], Some(7));
    assert_eq!(carried.backpack, [Some(8), Some(126)]);
    assert_eq!(
        placed.army.as_ref().unwrap()[0],
        Some(Stack {
            creature: 1,
            count: 5
        })
    );
    let Body::Monster(beaten) = &m.objects[1].body else {
        panic!("not a monster: {:?}", m.objects[1].body)
    };
    assert_eq!(beaten.identifier, None);
    assert_eq!(beaten.treasure.as_ref().unwrap().artifact, Some(126));
    assert_eq!(m.objects[2].body, Body::WitchHut(Bits::default()));
    let Body::Pandora { rewards, .. } = &m.objects[4].body else {
        panic!("not a box: {:?}", m.objects[4].body)
    };
    assert_eq!(rewards.artifacts, [126]);
    assert_eq!(
        rewards.creatures,
        [Stack {
            creature: 117,
            count: 2
        }]
    );
    let Body::Resource { message, .. } = &m.objects[5].body else {
        panic!("not a pile: {:?}", m.objects[5].body)
    };
    assert_eq!(message.as_ref().unwrap().guards, raw_guards());
    static_layer_validates(&m, &ROE);
    // E08 T61a: the whole map imports, the monster's artifact with it; an RoE map's Witch Huts
    // hold no mask (their type's skills by the profile, A-21).
    let setup = import(&m, &ROE).unwrap();
    setup.validate(&ROE).unwrap();
    let objects = &setup.map.objects;
    assert!(objects[0].monster.as_ref().unwrap().artifact.is_some());
    assert_eq!(objects[1].holds, Holds::Skills(None));
    assert_eq!(objects[2].holds, Holds::Skills(None));
}
