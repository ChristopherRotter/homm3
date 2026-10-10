//! proptest strategies for raw models that a file of their version can hold.

use proptest::collection::vec;
use proptest::prelude::*;
use proptest::sample::subsequence;

use crate::model::{
    Artifacts, Bits, CustomHero, DisposedHero, Header, Loss, MainHero, MainTown, Player,
    PlayerDetails, PlayerHero, Pos, RawMap, Rumor, Teams, Terrain, Text, Tile, Version, Victory,
    VictoryGoal,
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
                        },
                    )
            })
        })
        .boxed()
}
