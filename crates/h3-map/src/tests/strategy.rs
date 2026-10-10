//! proptest strategies for raw models that a file of their version can hold.

use proptest::collection::vec;
use proptest::prelude::*;
use proptest::sample::subsequence;

use crate::model::{
    Army, Artifacts, Bits, Body, Buildings, CustomHero, DisposedHero, EventBody, Header, HeroBody,
    Loss, MainHero, MainTown, Message, Mission, MonsterBody, MonsterTreasure, Object, Player,
    PlayerDetails, PlayerHero, Pos, Quest, RandomDwelling, RawMap, Reward, Rewards, Rumor,
    SeerTask, Stack, Teams, Template, Terrain, Text, Tile, TimedEvent, TownBody, TownEvent,
    Version, Victory, VictoryGoal,
};

pub fn version() -> impl Strategy<Value = Version> {
    prop_oneof![Just(Version::RoE), Just(Version::Ab), Just(Version::Sod)]
}

fn text() -> impl Strategy<Value = Text> {
    vec(any::<u8>(), 0..12).prop_map(Text)
}

fn bits(len: usize) -> impl Strategy<Value = Bits> {
    vec(any::<u8>(), len).prop_map(Bits)
}

fn pos() -> impl Strategy<Value = Pos> {
    any::<[u8; 3]>().prop_map(|[x, y, z]| Pos { x, y, z })
}

/// An artifact or creature code the version can store.
fn id(v: Version) -> BoxedStrategy<u16> {
    if v == Version::RoE {
        (0..=0xFFu16).boxed()
    } else {
        any::<u16>().boxed()
    }
}

/// An artifact slot: a code below the version's "none", or empty.
fn artifact(v: Version) -> BoxedStrategy<Option<u16>> {
    if v == Version::RoE {
        proptest::option::of(0..0xFFu16).boxed()
    } else {
        proptest::option::of(0..0xFFFFu16).boxed()
    }
}

fn header(v: Version) -> BoxedStrategy<Header> {
    (
        any::<bool>(),
        1..=4u32,
        any::<bool>(),
        text(),
        text(),
        any::<u8>(),
        any::<u8>(),
    )
        .prop_map(
            move |(any_players, size, two_levels, name, description, difficulty, limit)| Header {
                version: v,
                any_players,
                size,
                two_levels,
                name,
                description,
                difficulty,
                level_limit: if v == Version::RoE { 0 } else { limit },
            },
        )
        .boxed()
}

fn main_town(v: Version) -> impl Strategy<Value = MainTown> {
    (any::<bool>(), any::<u8>(), pos()).prop_map(move |(generate_hero, town_type, pos)| {
        if v == Version::RoE {
            MainTown {
                generate_hero: true,
                town_type: 0,
                pos,
            }
        } else {
            MainTown {
                generate_hero,
                town_type,
                pos,
            }
        }
    })
}

fn details(v: Version) -> BoxedStrategy<PlayerDetails> {
    let heroes = if v == Version::RoE {
        Just(Vec::new()).boxed()
    } else {
        vec(
            (any::<u8>(), text()).prop_map(|(hero_type, name)| PlayerHero { hero_type, name }),
            0..3,
        )
        .boxed()
    };
    let main_hero =
        (0..0xFFu8, any::<u8>(), text()).prop_map(|(hero_type, portrait, name)| MainHero {
            hero_type,
            portrait,
            name,
        });
    (
        (any::<i8>(), any::<u8>(), any::<u16>(), any::<bool>()),
        proptest::option::of(main_town(v)),
        any::<bool>(),
        proptest::option::of(main_hero),
        any::<u8>(),
        heroes,
    )
        .prop_map(
            move |(
                (ai_tactic, choice, towns, random_town),
                main_town,
                random_hero,
                main_hero,
                unknown,
                heroes,
            )| {
                PlayerDetails {
                    ai_tactic,
                    town_choice: if v == Version::Sod { choice } else { 0 },
                    allowed_towns: if v == Version::RoE {
                        towns & 0xFF
                    } else {
                        towns
                    },
                    random_town,
                    main_town,
                    random_hero,
                    main_hero,
                    unknown: if v == Version::RoE { 0 } else { unknown },
                    heroes,
                }
            },
        )
        .boxed()
}

fn player(v: Version) -> BoxedStrategy<Player> {
    (any::<bool>(), any::<bool>(), details(v))
        .prop_map(|(human, computer, details)| Player {
            human,
            computer,
            details: (human || computer).then_some(details),
        })
        .boxed()
}

fn victory(v: Version) -> BoxedStrategy<Victory> {
    let goal = prop_oneof![
        id(v).prop_map(|artifact| VictoryGoal::AcquireArtifact { artifact }),
        (id(v), any::<i32>())
            .prop_map(|(creature, count)| VictoryGoal::AccumulateCreatures { creature, count }),
        (any::<u8>(), any::<i32>())
            .prop_map(|(resource, amount)| VictoryGoal::AccumulateResources { resource, amount }),
        (pos(), any::<u8>(), any::<u8>())
            .prop_map(|(pos, hall, castle)| VictoryGoal::UpgradeTown { pos, hall, castle }),
        pos().prop_map(|pos| VictoryGoal::BuildGrail { pos }),
        pos().prop_map(|pos| VictoryGoal::DefeatHero { pos }),
        pos().prop_map(|pos| VictoryGoal::CaptureTown { pos }),
        pos().prop_map(|pos| VictoryGoal::DefeatMonster { pos }),
        Just(VictoryGoal::FlagDwellings),
        Just(VictoryGoal::FlagMines),
        (any::<u8>(), pos())
            .prop_map(|(artifact, pos)| VictoryGoal::TransportArtifact { artifact, pos }),
    ];
    prop_oneof![
        Just(Victory::Standard),
        (any::<bool>(), any::<bool>(), goal).prop_map(|(allow_standard, applies_to_ai, goal)| {
            Victory::Special {
                allow_standard,
                applies_to_ai,
                goal,
            }
        }),
    ]
    .boxed()
}

fn loss() -> impl Strategy<Value = Loss> {
    prop_oneof![
        Just(Loss::Standard),
        pos().prop_map(|pos| Loss::LoseTown { pos }),
        pos().prop_map(|pos| Loss::LoseHero { pos }),
        any::<u16>().prop_map(|days| Loss::TimeExpires { days }),
    ]
}

fn artifacts(v: Version) -> impl Strategy<Value = Artifacts> {
    (vec(artifact(v), v.artifact_slots()), vec(artifact(v), 0..3))
        .prop_map(|(worn, backpack)| Artifacts { worn, backpack })
}

fn custom_hero(v: Version, hero_type: u8) -> BoxedStrategy<CustomHero> {
    (
        proptest::option::of(any::<u32>()),
        proptest::option::of(vec(any::<(u8, u8)>(), 0..3)),
        proptest::option::of(artifacts(v)),
        proptest::option::of(text()),
        any::<u8>(),
        proptest::option::of(bits(9)),
        proptest::option::of(any::<[u8; 4]>()),
    )
        .prop_map(
            move |(
                experience,
                secondary_skills,
                artifacts,
                biography,
                sex,
                spells,
                primary_skills,
            )| {
                CustomHero {
                    hero_type,
                    experience,
                    secondary_skills,
                    artifacts,
                    biography,
                    sex,
                    spells,
                    primary_skills,
                }
            },
        )
        .boxed()
}

fn custom_heroes(v: Version) -> BoxedStrategy<Vec<CustomHero>> {
    if v != Version::Sod {
        return Just(Vec::new()).boxed();
    }
    subsequence((0..156u8).collect::<Vec<_>>(), 0..3)
        .prop_flat_map(move |types| {
            types
                .into_iter()
                .map(|t| custom_hero(v, t))
                .collect::<Vec<_>>()
        })
        .boxed()
}

fn terrain(header: &Header) -> BoxedStrategy<Terrain> {
    let size = usize::try_from(header.size).unwrap();
    let levels = header.levels();
    vec(any::<[u8; 7]>(), size * size * levels)
        .prop_map(move |tiles| Terrain {
            size,
            levels,
            tiles: tiles
                .into_iter()
                .map(|t| Tile {
                    terrain: t[0],
                    view: t[1],
                    river: t[2],
                    river_dir: t[3],
                    road: t[4],
                    road_dir: t[5],
                    flags: t[6],
                })
                .collect(),
        })
        .boxed()
}

/// Lists only the version stores, and empty otherwise.
fn only<T: core::fmt::Debug + Clone + 'static>(
    present: bool,
    s: impl Strategy<Value = Vec<T>> + 'static,
) -> BoxedStrategy<Vec<T>> {
    if present {
        s.boxed()
    } else {
        Just(Vec::new()).boxed()
    }
}

pub fn raw_map() -> BoxedStrategy<RawMap> {
    version()
        .prop_flat_map(|v| {
            header(v).prop_flat_map(move |header| {
                let disposed = (any::<u8>(), any::<u8>(), text(), any::<u8>()).prop_map(
                    |(hero_type, portrait, name, players)| DisposedHero {
                        hero_type,
                        portrait,
                        name,
                        players,
                    },
                );
                let rumor = (text(), text()).prop_map(|(name, text)| Rumor { name, text });
                let teams = (1..=u8::MAX, any::<[u8; 8]>())
                    .prop_map(|(count, of_player)| Teams { count, of_player });
                let banned = match v {
                    Version::RoE => 0,
                    Version::Ab => 17,
                    Version::Sod => 18,
                };
                let sod = usize::from(v == Version::Sod);
                (
                    Just(header.clone()),
                    [
                        player(v),
                        player(v),
                        player(v),
                        player(v),
                        player(v),
                        player(v),
                        player(v),
                        player(v),
                    ],
                    (victory(v), loss(), proptest::option::of(teams)),
                    bits(v.hero_types().div_ceil(8)),
                    only(v != Version::RoE, vec(any::<u8>(), 0..3)),
                    only(v == Version::Sod, vec(disposed, 0..3)),
                    (bits(banned), bits(9 * sod), bits(4 * sod)),
                    vec(rumor, 0..3),
                    custom_heroes(v),
                    terrain(&header),
                    object_layer(v),
                )
                    .prop_map(
                        |(
                            header,
                            players,
                            (victory, loss, teams),
                            allowed_heroes,
                            placeholders,
                            disposed_heroes,
                            (banned_artifacts, banned_spells, banned_skills),
                            rumors,
                            custom_heroes,
                            terrain,
                            (templates, objects, events),
                        )| RawMap {
                            header,
                            players,
                            victory,
                            loss,
                            teams,
                            allowed_heroes,
                            placeholders,
                            disposed_heroes,
                            banned_artifacts,
                            banned_spells,
                            banned_skills,
                            rumors,
                            custom_heroes,
                            terrain,
                            templates,
                            objects,
                            events,
                        },
                    )
            })
        })
        .boxed()
}

// --- templates, objects, timed events (T3b) ---

/// One (class, subclass) per body the reader tells apart, plus a decoration.
const CLASSES: [(u32, u32); 36] = [
    (124, 0),
    (26, 0),
    (34, 1),
    (62, 0),
    (70, 0),
    (54, 12),
    (71, 0),
    (59, 0),
    (91, 0),
    (83, 0),
    (113, 0),
    (81, 0),
    (33, 0),
    (219, 0),
    (5, 7),
    (65, 0),
    (93, 0),
    (76, 0),
    (79, 6),
    (77, 0),
    (98, 2),
    (53, 6),
    (53, 7),
    (220, 7),
    (17, 3),
    (42, 0),
    (87, 0),
    (216, 0),
    (217, 0),
    (218, 0),
    (88, 0),
    (6, 0),
    (36, 0),
    (36, 1000),
    (215, 0),
    (214, 0),
];

fn template() -> impl Strategy<Value = Template> {
    (
        text(),
        any::<([u8; 6], [u8; 6])>(),
        any::<(u16, u16)>(),
        proptest::sample::select(CLASSES.to_vec()),
        any::<(u8, u8)>(),
    )
        .prop_map(
            |(
                def,
                (passable, visitable),
                (terrains, editor_terrains),
                (class, subclass),
                (group, priority),
            )| {
                Template {
                    def,
                    passable,
                    visitable,
                    terrains,
                    editor_terrains,
                    class,
                    subclass,
                    group,
                    priority,
                }
            },
        )
}

fn stack(v: Version) -> BoxedStrategy<Stack> {
    (id(v), any::<u16>())
        .prop_map(|(creature, count)| Stack { creature, count })
        .boxed()
}

/// A creature code below the version's "none".
fn creature(v: Version) -> BoxedStrategy<u16> {
    if v == Version::RoE {
        (0..0xFFu16).boxed()
    } else {
        (0..0xFFFFu16).boxed()
    }
}

fn army(v: Version) -> BoxedStrategy<Army> {
    vec(
        proptest::option::of(
            (creature(v), any::<u16>()).prop_map(|(creature, count)| Stack { creature, count }),
        ),
        7,
    )
    .boxed()
}

fn message(v: Version) -> BoxedStrategy<Option<Message>> {
    proptest::option::of(
        (text(), proptest::option::of(army(v))).prop_map(|(text, guards)| Message { text, guards }),
    )
    .boxed()
}

fn rewards(v: Version) -> BoxedStrategy<Rewards> {
    (
        (any::<u32>(), any::<i32>(), any::<i8>(), any::<i8>()),
        any::<[i32; 7]>(),
        any::<[u8; 4]>(),
        vec(any::<(u8, u8)>(), 0..3),
        vec(id(v), 0..3),
        vec(any::<u8>(), 0..3),
        vec(stack(v), 0..3),
    )
        .prop_map(
            |(
                (experience, mana, morale, luck),
                resources,
                primary,
                skills,
                artifacts,
                spells,
                creatures,
            )| {
                Rewards {
                    experience,
                    mana,
                    morale,
                    luck,
                    resources,
                    primary,
                    skills,
                    artifacts,
                    spells,
                    creatures,
                }
            },
        )
        .boxed()
}

fn mission(v: Version) -> BoxedStrategy<Mission> {
    prop_oneof![
        any::<u32>().prop_map(Mission::Level),
        any::<[u8; 4]>().prop_map(Mission::PrimarySkills),
        any::<u32>().prop_map(Mission::KillHero),
        any::<u32>().prop_map(Mission::KillCreature),
        vec(id(v), 0..3).prop_map(Mission::Artifacts),
        vec(stack(v), 0..3).prop_map(Mission::Army),
        any::<[u32; 7]>().prop_map(Mission::Resources),
        any::<u8>().prop_map(Mission::Hero),
        any::<u8>().prop_map(Mission::Player),
    ]
    .boxed()
}

fn quest(v: Version) -> BoxedStrategy<Quest> {
    (mission(v), any::<i32>(), text(), text(), text())
        .prop_map(
            |(mission, deadline, first_visit, next_visit, completed)| Quest {
                mission,
                deadline,
                first_visit,
                next_visit,
                completed,
            },
        )
        .boxed()
}

fn reward(v: Version) -> BoxedStrategy<Reward> {
    prop_oneof![
        Just(Reward::Nothing),
        any::<u32>().prop_map(Reward::Experience),
        any::<u32>().prop_map(Reward::Mana),
        any::<i8>().prop_map(Reward::Morale),
        any::<i8>().prop_map(Reward::Luck),
        any::<(u8, u32)>().prop_map(|(resource, amount)| Reward::Resource { resource, amount }),
        any::<(u8, u8)>().prop_map(|(skill, amount)| Reward::PrimarySkill { skill, amount }),
        any::<(u8, u8)>().prop_map(|(skill, level)| Reward::SecondarySkill { skill, level }),
        id(v).prop_map(Reward::Artifact),
        any::<u8>().prop_map(Reward::Spell),
        stack(v).prop_map(Reward::Creatures),
    ]
    .boxed()
}

fn seer_hut(v: Version) -> BoxedStrategy<Body> {
    let quest = if v == Version::RoE {
        (0..0xFFu16)
            .prop_map(|a| Quest {
                mission: Mission::Artifacts(vec![a]),
                deadline: -1,
                first_visit: Text::default(),
                next_visit: Text::default(),
                completed: Text::default(),
            })
            .boxed()
    } else {
        quest(v)
    };
    proptest::option::of((quest, reward(v)).prop_map(|(quest, reward)| SeerTask { quest, reward }))
        .prop_map(Body::SeerHut)
        .boxed()
}

fn hero(v: Version) -> BoxedStrategy<Body> {
    let ab = v != Version::RoE;
    let sod = v == Version::Sod;
    (
        (
            any::<u32>(),
            any::<u8>(),
            any::<u8>(),
            proptest::option::of(text()),
        ),
        (
            proptest::option::of(any::<u32>()),
            proptest::option::of(any::<u8>()),
        ),
        (
            proptest::option::of(vec(any::<(u8, u8)>(), 0..3)),
            proptest::option::of(army(v)),
            any::<u8>(),
        ),
        (proptest::option::of(artifacts(v)), any::<u8>()),
        (proptest::option::of(text()), any::<u8>(), any::<u8>()),
        (
            proptest::option::of(bits(9)),
            proptest::option::of(any::<[u8; 4]>()),
        ),
    )
        .prop_map(
            move |(
                (identifier, owner, hero_type, name),
                (experience, portrait),
                (secondary_skills, army, formation),
                (artifacts, patrol),
                (biography, sex, spell),
                (spells, primary_skills),
            )| {
                Body::Hero(Box::new(HeroBody {
                    identifier: ab.then_some(identifier),
                    owner,
                    hero_type,
                    name,
                    experience: if sod {
                        experience
                    } else {
                        Some(experience.unwrap_or(0))
                    },
                    portrait,
                    secondary_skills,
                    army,
                    formation,
                    artifacts,
                    patrol,
                    biography: biography.filter(|_| ab),
                    sex: if ab { sex } else { u8::MAX },
                    spell: (v == Version::Ab).then_some(spell),
                    spells: spells.filter(|_| sod),
                    primary_skills: primary_skills.filter(|_| sod),
                }))
            },
        )
        .boxed()
}

fn monster(v: Version) -> BoxedStrategy<Body> {
    let treasure =
        (text(), any::<[i32; 7]>(), artifact(v)).prop_map(|(text, resources, artifact)| {
            MonsterTreasure {
                text,
                resources,
                artifact,
            }
        });
    (
        any::<u32>(),
        any::<u16>(),
        any::<u8>(),
        proptest::option::of(treasure),
        any::<(bool, bool)>(),
    )
        .prop_map(
            move |(identifier, count, disposition, treasure, (never_flees, no_growth))| {
                Body::Monster(MonsterBody {
                    identifier: (v != Version::RoE).then_some(identifier),
                    count,
                    disposition,
                    treasure,
                    never_flees,
                    no_growth,
                })
            },
        )
        .boxed()
}

fn timed_event(v: Version) -> BoxedStrategy<TimedEvent> {
    (
        text(),
        text(),
        any::<[i32; 7]>(),
        any::<u8>(),
        any::<(bool, bool)>(),
        any::<(u16, u16)>(),
    )
        .prop_map(
            move |(name, message, resources, players, (human, computer), (first_day, interval))| {
                TimedEvent {
                    name,
                    message,
                    resources,
                    players,
                    human: human || v != Version::Sod,
                    computer,
                    first_day,
                    interval,
                }
            },
        )
        .boxed()
}

fn town(v: Version) -> BoxedStrategy<Body> {
    let buildings = prop_oneof![
        any::<bool>().prop_map(|fort| Buildings::Standard { fort }),
        any::<([u8; 6], [u8; 6])>()
            .prop_map(|(built, forbidden)| Buildings::Custom { built, forbidden }),
    ];
    let event = (timed_event(v), any::<[u8; 6]>(), any::<[u16; 7]>()).prop_map(
        |(event, buildings, creatures)| TownEvent {
            event,
            buildings,
            creatures,
        },
    );
    (
        (any::<u32>(), any::<u8>(), proptest::option::of(text())),
        (proptest::option::of(army(v)), any::<u8>(), buildings),
        (bits(9), bits(9)),
        vec(event, 0..2),
        any::<u8>(),
    )
        .prop_map(
            move |(
                (identifier, owner, name),
                (garrison, formation, buildings),
                (required, banned_spells),
                events,
                alignment,
            )| {
                let ab = v != Version::RoE;
                Body::Town(Box::new(TownBody {
                    identifier: ab.then_some(identifier),
                    owner,
                    name,
                    garrison,
                    formation,
                    buildings,
                    required_spells: ab.then_some(required),
                    banned_spells,
                    events,
                    alignment: if v == Version::Sod {
                        alignment
                    } else {
                        u8::MAX
                    },
                }))
            },
        )
        .boxed()
}

fn random_dwelling(v: Version, class: u32) -> BoxedStrategy<Body> {
    let faction = (prop_oneof![Just(0), any::<u32>()], bits(v.faction_bytes()))
        .prop_map(|(town, set)| (town, (town == 0).then_some(set)));
    (any::<u32>(), faction, any::<(u8, u8)>())
        .prop_map(move |(owner, faction, levels)| {
            Body::RandomDwelling(RandomDwelling {
                owner,
                faction: (class != 218).then_some(faction),
                levels: (class != 217).then_some(levels),
            })
        })
        .boxed()
}

/// A body the template's class carries in this version.
fn body(v: Version, t: &Template) -> BoxedStrategy<Body> {
    let owner = || any::<u32>().prop_map(|owner| Body::Owned { owner }).boxed();
    match (t.class, t.subclass) {
        (26, _) => (message(v), rewards(v), any::<u8>(), any::<(bool, bool)>())
            .prop_map(
                |(message, rewards, players, (computer, remove_after_visit))| {
                    Body::Event(EventBody {
                        message,
                        rewards,
                        players,
                        computer,
                        remove_after_visit,
                    })
                },
            )
            .boxed(),
        (34 | 62 | 70, _) => hero(v),
        (54 | 71, _) => monster(v),
        (59 | 91, _) => text().prop_map(Body::Sign).boxed(),
        (83, _) => seer_hut(v),
        (113, _) => bits(if v == Version::RoE { 0 } else { 4 })
            .prop_map(Body::WitchHut)
            .boxed(),
        (81, _) => any::<(i8, u8)>()
            .prop_map(|(kind, code)| Body::Scholar { kind, code })
            .boxed(),
        (33 | 219, _) => (any::<u32>(), army(v), any::<bool>())
            .prop_map(move |(owner, army, removable)| Body::Garrison {
                owner,
                army,
                removable: removable || v == Version::RoE,
            })
            .boxed(),
        (5 | 65, _) => message(v).prop_map(Body::Artifact).boxed(),
        (93, _) => (message(v), any::<u32>())
            .prop_map(|(message, spell)| Body::Scroll { message, spell })
            .boxed(),
        (76 | 79, _) => (message(v), any::<u32>())
            .prop_map(|(message, amount)| Body::Resource { message, amount })
            .boxed(),
        (77 | 98, _) => town(v),
        (53, s) if s < 7 => any::<u32>().prop_map(|owner| Body::Mine { owner }).boxed(),
        (53 | 220, _) => bits(4).prop_map(Body::AbandonedMine).boxed(),
        (17 | 42 | 87, _) => owner(),
        (c @ 216..=218, _) => random_dwelling(v, c),
        (88, _) => any::<u32>()
            .prop_map(|spell| Body::Shrine { spell })
            .boxed(),
        (6, _) => (message(v), rewards(v))
            .prop_map(|(message, rewards)| Body::Pandora { message, rewards })
            .boxed(),
        (36, s) if s < 1000 => any::<i32>()
            .prop_map(|radius| Body::Grail { radius })
            .boxed(),
        (215, _) => proptest::option::of(quest(v))
            .prop_map(Body::QuestGuard)
            .boxed(),
        (214, _) => (
            any::<u8>(),
            prop_oneof![Just(u8::MAX), any::<u8>()],
            any::<u8>(),
        )
            .prop_map(|(owner, hero_type, rank)| Body::HeroPlaceholder {
                owner,
                hero_type,
                power_rank: (hero_type == u8::MAX).then_some(rank),
            })
            .boxed(),
        _ => Just(Body::None).boxed(),
    }
}

/// Templates, objects that refer to them with fitting bodies, and the map's timed events.
pub fn object_layer(v: Version) -> BoxedStrategy<(Vec<Template>, Vec<Object>, Vec<TimedEvent>)> {
    vec(template(), 1..4)
        .prop_flat_map(move |templates| {
            let objects = vec(
                (0..templates.len(), any::<[u8; 3]>()).prop_flat_map({
                    let templates = templates.clone();
                    move |(i, [x, y, z])| {
                        body(v, &templates[i]).prop_map(move |body| Object {
                            pos: Pos { x, y, z },
                            template: u32::try_from(i).unwrap(),
                            body,
                        })
                    }
                }),
                0..4,
            );
            (Just(templates), objects, vec(timed_event(v), 0..2))
        })
        .boxed()
}
