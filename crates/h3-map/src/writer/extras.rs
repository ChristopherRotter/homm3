//! What the map client's situations need beyond a [`Map`] (E24 M1 T3a, the map client lane's
//! additive change with the user's consent, 2026-10-23): tiles written as given, so that terrain
//! transitions, rivers and roads are written exactly, and objects of any template, each with the
//! body the map editor gives such an object left as placed. [`write_with`](super::write_with)
//! writes them; without them it writes what [`write`](super::write) wrote before, byte for byte.

use super::{
    ARMY_SLOTS, Map, ObjectTemplate, Out, RoadKind, Terrain, Town, TownBuildings, WriteError,
    out_of_range, write_army, write_town,
};

/// River kinds in map files.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiverKind {
    /// Clear river.
    Clear = 1,
    /// Icy river.
    Icy = 2,
    /// Muddy river.
    Muddy = 3,
    /// Lava river.
    Lava = 4,
}

/// A tile written as given: the seven bytes of the map's tile record (`docs/formats/h3m.md`,
/// "Tile"). It replaces what the base terrain, the patches and the roads give the tile, and its
/// terrain counts for its neighbours' coast flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GivenTile {
    /// Level: 0 the surface, 1 the underground.
    pub level: u8,
    /// Column and row.
    pub tile: (u8, u8),
    /// Terrain.
    pub terrain: Terrain,
    /// Terrain picture: the frame of the terrain's DEF (`docs/epics/E24-design.md` §1.3). Not
    /// checked: the writer knows no DEF.
    pub picture: u8,
    /// River and its picture.
    pub river: Option<(RiverKind, u8)>,
    /// Road and its picture.
    pub road: Option<(RoadKind, u8)>,
    /// Flags as given: bits 0 and 1 mirror the terrain picture left to right and top to bottom,
    /// bits 2 and 3 the river's, 4 and 5 the road's; `0x40` the coast, which the writer does not
    /// compute for a given tile.
    pub flags: u8,
}

/// An object of any template but a hero's (34, 70: [`super::Hero`]), a prison's (62) or a hero
/// placeholder's (214), whose bodies name a hero; written with the body the map editor gives
/// such an object left as placed (`write_empty_body`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnyObject {
    /// Level: 0 the surface, 1 the underground.
    pub level: u8,
    /// The object's position: the bottom-right corner of its picture. It may lie up to 7
    /// columns past the map's right edge and 5 rows past its bottom edge, as decorations in the
    /// game's own maps do (5 and 2 at most in its SoD maps, E24 M1 T3a); the game draws only the
    /// cells on the map (`docs/epics/E24-design.md` §2.1).
    pub tile: (u8, u8),
    /// Template.
    pub template: ObjectTemplate,
    /// Owner (0 = red … 7 = pink), only for a class whose body names one: the dwellings (17–20),
    /// a garrison (33, 219), a lighthouse (42), a mine (53, 220 below subclass 7), a town (77,
    /// 98), a shipyard (87) and the random dwellings (216–218). `None` is neutral.
    pub owner: Option<u8>,
}

/// Tiles as given and objects of any template (E24 M1 T3a), written by
/// [`write_with`](super::write_with) after what the [`Map`] gives.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Extras {
    /// Tiles written as given; at most one per tile.
    pub tiles: Vec<GivenTile>,
    /// Objects written after the map's own (the surface's and the underground's), in this order.
    pub objects: Vec<AnyObject>,
}

/// The classes whose bodies name a hero: a hero, a prison, a random hero, a hero placeholder.
const HERO_CLASSES: [u32; 4] = [34, 62, 70, 214];
/// How far past the map's side an object's position may lie (`x <= side + PAST_RIGHT`, the
/// right edge's column being `side - 1`): a template is at most 8 tiles wide and 6 high, so one
/// further out would have no cell on the map (its leftmost column `x - 7`, its top row `y - 5`).
const PAST_RIGHT: u16 = 6;
const PAST_BOTTOM: u16 = 4;
/// A witch hut's possible skills as the editor sets them: all 28 but Leadership (6) and
/// Necromancy (12); 213 of the 259 witch huts in the game's 56 SoD maps (E24 M1 T3a, as every
/// count below).
const WITCH_HUT_SKILLS: [u8; 4] = [0xbf, 0xef, 0xff, 0x0f];
/// An abandoned mine's possible resources as the editor sets them: all but wood (bits 1–6); 45
/// of the 54.
const ABANDONED_MINE_RESOURCES: [u8; 4] = [0x7e, 0, 0, 0];
/// A random dwelling's factions when no town decides them: all nine (SoD's two bytes; each of
/// the 28 such dwellings).
const ALL_FACTIONS: [u8; 2] = [0xff, 0x01];
/// Rewards that give nothing: experience, mana, morale, luck, seven resources, four primary
/// skills, four empty lists, eight unused bytes.
const NO_REWARDS: [u8; 54] = [0; 54];

/// Whether the body of `template`'s class names an owner (the parser's reading of it).
fn has_owner(template: &ObjectTemplate) -> bool {
    match template.class {
        53 | 220 => template.subclass < 7,
        17..=20 | 33 | 42 | 77 | 87 | 98 | 216..=219 => true,
        _ => false,
    }
}

/// The tile's position in a grid of every level's tiles.
fn grid_index(side: u8, level: u8, (x, y): (u8, u8)) -> usize {
    let side = usize::from(side);
    (usize::from(level) * side + usize::from(y)) * side + usize::from(x)
}

fn packed(level: u8, (x, y): (u8, u8)) -> u64 {
    u64::from(level) << 16 | u64::from(x) << 8 | u64::from(y)
}

pub(super) fn validate(map: &Map, extras: &Extras) -> Result<(), WriteError> {
    let levels = 1 + u8::from(map.underground.is_some());
    let side = map.size;
    let mut given = vec![false; usize::from(levels) * usize::from(side) * usize::from(side)];
    for tile in &extras.tiles {
        let (x, y) = tile.tile;
        if tile.level >= levels || x >= side || y >= side {
            return Err(out_of_range("given tile", packed(tile.level, tile.tile)));
        }
        let seen = &mut given[grid_index(side, tile.level, tile.tile)];
        if *seen {
            return Err(out_of_range(
                "tile given twice",
                packed(tile.level, tile.tile),
            ));
        }
        *seen = true;
    }
    for object in &extras.objects {
        let (x, y) = object.tile;
        let class = object.template.class;
        if object.level >= levels
            || u16::from(x) > u16::from(side) + PAST_RIGHT
            || u16::from(y) > u16::from(side) + PAST_BOTTOM
        {
            return Err(out_of_range(
                "object tile (past the edges)",
                packed(object.level, object.tile),
            ));
        }
        if HERO_CLASSES.contains(&class) {
            return Err(out_of_range("object class (names a hero)", class));
        }
        if let Some(owner) = object.owner {
            if !has_owner(&object.template) {
                return Err(out_of_range(
                    "owner of an object without one (class)",
                    class,
                ));
            }
            if map
                .players
                .get(usize::from(owner))
                .and_then(Option::as_ref)
                .is_none()
            {
                return Err(out_of_range("object owner (no playable player)", owner));
            }
        }
    }
    Ok(())
}

/// The given tiles of one level, by tile (row by row).
pub(super) fn given_tiles(map: &Map, extras: &Extras, level: u8) -> Vec<Option<GivenTile>> {
    let side = usize::from(map.size);
    let mut tiles = vec![None; side * side];
    for tile in extras.tiles.iter().filter(|t| t.level == level) {
        tiles[grid_index(map.size, 0, tile.tile)] = Some(*tile);
    }
    tiles
}

/// The tile record of a given tile.
pub(super) fn tile_bytes(tile: &GivenTile) -> [u8; 7] {
    let (river, river_picture) = tile.river.map_or((0, 0), |(kind, p)| (kind as u8, p));
    let (road, road_picture) = tile.road.map_or((0, 0), |(kind, p)| (kind as u8, p));
    [
        tile.terrain as u8,
        tile.picture,
        river,
        river_picture,
        road,
        road_picture,
        tile.flags,
    ]
}

/// The body the map editor gives an object of the template's class left as placed
/// (`docs/formats/h3m.md`, "Object bodies by class"; where the editor chooses, its choice is the
/// most common body in the game's SoD maps, E24 M1 T3a): nothing optional, no message, no
/// guards, no quest, random amounts and choices, the owner or neutral; `identifier` for a
/// monster's and a town's. The hero classes never reach it (`validate`).
pub(super) fn write_empty_body(out: &mut Out, object: &AnyObject, identifier: u32) {
    let template = &object.template;
    let owner = u32::from(object.owner.unwrap_or(0xff));
    match template.class {
        // An event: no message, no rewards, every player, not the computer, removed after its
        // visit (59 of the 103 events).
        26 => {
            out.bool(false);
            out.bytes(&NO_REWARDS);
            out.u8(0xff);
            out.bool(false);
            out.bool(true);
            out.bytes(&[0; 4]);
        }
        // A monster: a random count, aggressive (3 580 of the 4 978 monsters), no message or
        // treasure, may flee, grows.
        54 | 71..=75 | 162..=164 => {
            out.u32(identifier);
            out.u16(0);
            out.u8(2);
            out.bytes(&[0; 5]);
        }
        // An ocean bottle or a sign: an empty text.
        59 | 91 => out.bytes(&[0; 8]),
        // A seer hut without a quest: mission 0, the reward's unused byte, two unused.
        83 => out.bytes(&[0; 4]),
        113 => out.bytes(&WITCH_HUT_SKILLS),
        // A scholar: a random bonus (271 of the 290).
        81 => {
            out.u8(0xff);
            out.bytes(&[0; 7]);
        }
        // A garrison: no troops, removable (71 of the 82).
        33 | 219 => {
            out.u32(owner);
            write_army(out, &[None; ARMY_SLOTS]);
            out.bool(true);
            out.bytes(&[0; 8]);
        }
        // An artifact: no message.
        5 | 65..=69 => out.bool(false),
        // A spell scroll: no message, spell 0 (the format has no random scroll).
        93 => out.bytes(&[0; 5]),
        // A resource: no message, a random amount.
        76 | 79 => out.bytes(&[0; 9]),
        // A town: the standard buildings with a fort (284 of the 564 towns).
        77 | 98 => write_town(
            out,
            &Town {
                tile: object.tile,
                template: template.clone(),
                owner: object.owner,
                garrison: None,
                tight_formation: false,
                buildings: TownBuildings::Standard { fort: true },
            },
            identifier,
        ),
        53 | 220 if template.subclass < 7 => out.u32(owner),
        53 | 220 => out.bytes(&ABANDONED_MINE_RESOURCES),
        17..=20 | 42 | 87 => out.u32(owner),
        // A random dwelling: no town, every faction, every level (0 to 6).
        class @ 216..=218 => {
            out.u32(owner);
            if class != 218 {
                out.u32(0);
                out.bytes(&ALL_FACTIONS);
            }
            if class != 217 {
                out.bytes(&[0, 6]);
            }
        }
        // A shrine: a random spell (403 of the 413).
        88..=90 => out.u32(0xff),
        // A Pandora's box: no message, no rewards.
        6 => {
            out.bool(false);
            out.bytes(&NO_REWARDS);
        }
        // The grail: radius 0.
        36 if template.subclass < 1000 => out.u32(0),
        // A quest guard without a quest.
        215 => out.u8(0),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::smoke_like_map;
    use super::super::{
        Ground, Guards, Hero, MapObject, Monster, PLAYERS, Patch, Pile, Plain, Player, Road,
        Underground, crc32, write, write_with,
    };
    use super::*;
    use crate::model::{
        Bits, Body, Buildings, EventBody, MonsterBody, Pos, RandomDwelling, Rewards, Text, Tile,
        TownBody,
    };
    use crate::{Limits, RawMap, parse};

    /// A map of every kind the writer wrote before E24 M1 T3a.
    fn golden_map() -> Map {
        let template = |line: &str| ObjectTemplate::from_objects_txt(line).unwrap();
        let mut players: [Option<Player>; PLAYERS] = Default::default();
        players[0] = Some(Player {
            human: true,
            computer: true,
            towns: 1,
        });
        players[1] = Some(Player {
            human: false,
            computer: true,
            towns: 1 << 6,
        });
        let hero = |tile, owner, hero_type, class| {
            MapObject::Hero(Hero {
                tile,
                owner,
                hero_type,
                class,
                name: None,
                experience: Some(1000),
                skills: Some(vec![(1, 2)]),
                army: Some([Some((2, 20)), None, None, None, None, None, None]),
                tight_formation: true,
                artifacts: None,
                spells: Some([1, 0, 0, 0, 0, 0, 0, 0, 2]),
                primary: Some([1, 2, 3, 4]),
            })
        };
        let pikeman = template(
            "AvWPike.def 011111111111111111111111111111111111111111111111 \
             100000000000000000000000000000000000000000000000 111111111 000000100 54 0 2 0",
        );
        let wood = template(
            "AVTwood0.def 011111111111111111111111111111111111111111111111 \
             100000000000000000000000000000000000000000000000 111111111 111111111 79 0 0 0",
        );
        let objects = vec![
            MapObject::Ground(Ground {
                tile: (5, 5),
                template: template(
                    "AVXcg5.def 111111111111111111111111111111111111111111111111 \
                     000000000000000000000000000000000000000000000000 111111111 011111111 223 0 0 1",
                ),
            }),
            hero((10, 10), 0, 0, 0),
            MapObject::Monster(Monster {
                tile: (20, 20),
                template: pikeman,
                count: 10,
                disposition: 4,
                never_flees: true,
                fixed_count: false,
                treasure: None,
            }),
            MapObject::Pile(Pile {
                tile: (7, 9),
                template: wood.clone(),
                amount: 5,
                guards: Some(Guards {
                    army: [Some((1, 3)), None, None, None, None, None, None],
                    question: "Fight?".to_owned(),
                }),
            }),
            MapObject::Plain(Plain {
                tile: (30, 30),
                template: template(
                    "AVLtr3d0.def 011111111111111111111111111111111111111111111111 \
                     000000000000000000000000000000000000000000000000 111111111 000010101 155 0 0 0",
                ),
            }),
            MapObject::Town(Town {
                tile: (16, 12),
                template: template(
                    "AVCstrx0.def 000001110000011110001111111111111111111111111111 \
                     001000000000000000000000000000000000000000000000 011111111 011111111 98 6 1 0",
                ),
                owner: Some(1),
                garrison: None,
                tight_formation: false,
                buildings: TownBuildings::Standard { fort: true },
            }),
            hero((12, 14), 1, 1, 1),
        ];
        Map {
            name: "Golden".to_owned(),
            description: "Every kind".to_owned(),
            size: 36,
            terrain: Terrain::Grass,
            patches: vec![
                Patch {
                    terrain: Terrain::Water,
                    from: (0, 0),
                    to: (3, 2),
                },
                Patch {
                    terrain: Terrain::Lava,
                    from: (20, 3),
                    to: (25, 8),
                },
            ],
            roads: vec![
                Road {
                    kind: RoadKind::Cobblestone,
                    row: 18,
                    from: 2,
                    to: 9,
                },
                Road {
                    kind: RoadKind::Dirt,
                    row: 19,
                    from: 4,
                    to: 5,
                },
            ],
            players,
            objects,
            underground: Some(Underground {
                terrain: Terrain::Subterranean,
                patches: vec![Patch {
                    terrain: Terrain::Rock,
                    from: (0, 0),
                    to: (35, 1),
                }],
                objects: vec![
                    hero((3, 4), 0, 2, 2),
                    MapObject::Pile(Pile {
                        tile: (6, 6),
                        template: wood,
                        amount: 0,
                        guards: None,
                    }),
                ],
            }),
            level_limit: 12,
            banned_skills: vec![3, 27],
        }
    }

    fn read_back(map: &Map, extras: &Extras) -> RawMap {
        parse(&write_with(map, extras).unwrap(), &Limits::default()).unwrap()
    }

    fn with_underground() -> Map {
        let mut map = smoke_like_map();
        map.underground = Some(Underground {
            terrain: Terrain::Subterranean,
            patches: vec![],
            objects: vec![],
        });
        map
    }

    fn template(class: u32, subclass: u32) -> ObjectTemplate {
        ObjectTemplate {
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

    fn object(level: u8, tile: (u8, u8), template: ObjectTemplate) -> AnyObject {
        AnyObject {
            level,
            tile,
            template,
            owner: None,
        }
    }

    /// T3a's acceptance: a map of every kind the writer wrote before is written byte for byte as
    /// before (the length and CRC-32 of `master`'s writer at 328b808 for `golden_map`).
    #[test]
    fn maps_without_extras_are_written_as_before() {
        let bytes = write(&golden_map()).unwrap();
        assert_eq!((bytes.len(), crc32(&bytes)), (19_514, 0x4909_679c));
        assert_eq!(write_with(&golden_map(), &Extras::default()), Ok(bytes));
    }

    #[test]
    fn given_tiles_replace_what_the_map_gives_them() {
        let given = |level, tile, terrain, picture, river, road, flags| GivenTile {
            level,
            tile,
            terrain,
            picture,
            river,
            road,
            flags,
        };
        let extras = Extras {
            tiles: vec![
                // On the gravel road's second tile (row 5, columns 1–4).
                given(
                    0,
                    (2, 5),
                    Terrain::Dirt,
                    30,
                    Some((RiverKind::Lava, 11)),
                    Some((RoadKind::Cobblestone, 2)),
                    0x2a,
                ),
                given(0, (10, 20), Terrain::Water, 22, None, None, 0x03),
                given(0, (11, 20), Terrain::Grass, 50, None, None, 0),
                given(
                    0,
                    (12, 20),
                    Terrain::Swamp,
                    3,
                    Some((RiverKind::Muddy, 1)),
                    None,
                    0,
                ),
                given(
                    1,
                    (5, 5),
                    Terrain::Lava,
                    50,
                    Some((RiverKind::Icy, 3)),
                    None,
                    0x0c,
                ),
                given(
                    1,
                    (6, 5),
                    Terrain::Rough,
                    1,
                    Some((RiverKind::Clear, 0)),
                    None,
                    0x40,
                ),
            ],
            objects: vec![],
        };
        let m = read_back(&with_underground(), &extras);
        let tile = |x, y, z| *m.terrain.tile(Pos { x, y, z }).unwrap();
        let raw = |t: [u8; 7]| Tile {
            terrain: t[0],
            view: t[1],
            river: t[2],
            river_dir: t[3],
            road: t[4],
            road_dir: t[5],
            flags: t[6],
        };
        assert_eq!(tile(2, 5, 0), raw([0, 30, 4, 11, 3, 2, 0x2a]));
        assert_eq!(tile(10, 20, 0), raw([8, 22, 0, 0, 0, 0, 0x03]));
        assert_eq!(tile(12, 20, 0), raw([4, 3, 3, 1, 0, 0, 0]));
        assert_eq!(tile(5, 5, 1), raw([7, 50, 2, 3, 0, 0, 0x0c]));
        assert_eq!(tile(6, 5, 1), raw([5, 1, 1, 0, 0, 0, 0x40]));
        // The road around the given tile, the patch and the base terrain stay.
        assert_eq!((tile(1, 5, 0).road, tile(1, 5, 0).road_dir), (2, 15));
        assert_eq!((tile(3, 5, 0).road, tile(3, 5, 0).road_dir), (2, 13));
        assert_eq!(tile(3, 3, 0).terrain, 4);
        assert_eq!(tile(7, 5, 1).terrain, 6);
        // The given water makes its land neighbours coastal, but a given tile keeps its flags.
        for (x, y) in [
            (9, 19),
            (10, 19),
            (11, 19),
            (9, 20),
            (9, 21),
            (10, 21),
            (11, 21),
        ] {
            assert_eq!(
                (tile(x, y, 0).terrain, tile(x, y, 0).flags),
                (2, 0x40),
                "({x}, {y})"
            );
        }
        assert_eq!((tile(11, 20, 0).flags, tile(12, 21, 0).flags), (0, 0));
    }

    #[test]
    fn objects_of_any_template_follow_the_map_s_own() {
        let map = with_underground();
        let MapObject::Monster(pikeman) = &map.objects[1] else {
            panic!("the smoke map's second object is a monster")
        };
        let mut mine = object(0, (42, 40), template(53, 0));
        mine.owner = Some(1);
        let extras = Extras {
            tiles: vec![],
            objects: vec![
                object(0, (30, 30), template(8, 0)),
                mine,
                object(0, (21, 20), pikeman.template.clone()),
                object(1, (4, 4), template(91, 0)),
            ],
        };
        let m = read_back(&map, &extras);
        // The hero's and the monster's templates, then the boat's, the mine's and the sign's.
        let defs: Vec<_> = m.templates.iter().map(|t| t.def.0.clone()).collect();
        assert_eq!(
            defs[2..],
            [
                b"o8s0.def".to_vec(),
                b"o53s0.def".to_vec(),
                b"o91s0.def".to_vec()
            ]
        );
        let placed: Vec<_> = m.objects[2..]
            .iter()
            .map(|o| ((o.pos.x, o.pos.y, o.pos.z), o.template))
            .collect();
        assert_eq!(
            placed,
            [
                ((30, 30, 0), 2),
                ((42, 40, 0), 3),
                ((21, 20, 0), 1),
                ((4, 4, 1), 4)
            ]
        );
        assert_eq!(m.objects[2].body, Body::None);
        assert_eq!(m.objects[3].body, Body::Mine { owner: 1 });
        // The fifth object's identifier.
        let Body::Monster(monster) = &m.objects[4].body else {
            panic!("not a monster: {:?}", m.objects[4].body)
        };
        assert_eq!(monster.identifier, Some(5));
        assert_eq!(m.objects[5].body, Body::Sign(Text::default()));
    }

    /// The body each class reads back as (the parser's reading, VCMI 1.7.5's).
    fn expected(class: u32, subclass: u32, identifier: u32, owner: u8) -> Body {
        let owned = u32::from(owner);
        let dwelling = |faction, levels| {
            Body::RandomDwelling(RandomDwelling {
                owner: owned,
                faction,
                levels,
            })
        };
        let factions = Some((0, Some(Bits(vec![0xff, 0x01]))));
        match class {
            26 => Body::Event(EventBody {
                message: None,
                rewards: Rewards::default(),
                players: 0xff,
                computer: false,
                remove_after_visit: true,
            }),
            54 | 71..=75 | 162..=164 => Body::Monster(MonsterBody {
                identifier: Some(identifier),
                count: 0,
                disposition: 2,
                treasure: None,
                never_flees: false,
                no_growth: false,
            }),
            59 | 91 => Body::Sign(Text::default()),
            83 => Body::SeerHut(None),
            113 => Body::WitchHut(Bits(vec![0xbf, 0xef, 0xff, 0x0f])),
            81 => Body::Scholar { kind: -1, code: 0 },
            33 | 219 => Body::Garrison {
                owner: owned,
                army: vec![None; 7],
                removable: true,
            },
            5 | 65..=69 => Body::Artifact(None),
            93 => Body::Scroll {
                message: None,
                spell: 0,
            },
            76 | 79 => Body::Resource {
                message: None,
                amount: 0,
            },
            77 | 98 => Body::Town(Box::new(TownBody {
                identifier: Some(identifier),
                owner,
                name: None,
                garrison: None,
                formation: 0,
                buildings: Buildings::Standard { fort: true },
                required_spells: Some(Bits(vec![0; 9])),
                banned_spells: Bits(vec![0; 9]),
                events: vec![],
                alignment: 0xff,
            })),
            53 | 220 if subclass < 7 => Body::Mine { owner: owned },
            53 | 220 => Body::AbandonedMine(Bits(vec![0x7e, 0, 0, 0])),
            17..=20 | 42 | 87 => Body::Owned { owner: owned },
            216 => dwelling(factions, Some((0, 6))),
            217 => dwelling(factions, None),
            218 => dwelling(None, Some((0, 6))),
            88..=90 => Body::Shrine { spell: 0xff },
            6 => Body::Pandora {
                message: None,
                rewards: Rewards::default(),
            },
            36 if subclass < 1000 => Body::Grail { radius: 0 },
            215 => Body::QuestGuard(None),
            _ => Body::None,
        }
    }

    /// Every class but the heroes' reads back with the body the editor gives it, owned by Blue
    /// where its body names an owner, so the writer's bodies have the reader's sizes.
    #[test]
    fn every_class_reads_back_with_the_editor_s_body() {
        let map = smoke_like_map();
        let kinds = (0..=255u32)
            .filter(|class| !HERO_CLASSES.contains(class))
            .map(|class| (class, 0))
            .chain([(53, 7), (220, 7), (36, 1000)]);
        let objects: Vec<_> = kinds
            .enumerate()
            .map(|(i, (class, subclass))| {
                let mut o = object(
                    0,
                    ((i % 36) as u8, (i / 36) as u8),
                    template(class, subclass),
                );
                o.owner = has_owner(&o.template).then_some(1);
                o
            })
            .collect();
        let extras = Extras {
            tiles: vec![],
            objects: objects.clone(),
        };
        let m = read_back(&map, &extras);
        assert_eq!(m.objects.len(), 2 + objects.len());
        for (i, (o, read)) in objects.iter().zip(&m.objects[2..]).enumerate() {
            let t = &o.template;
            let read_template = &m.templates[read.template as usize];
            assert_eq!(
                (read_template.class, read_template.subclass),
                (t.class, t.subclass)
            );
            assert_eq!((read.pos.x, read.pos.y), o.tile);
            let identifier = u32::try_from(i).unwrap() + 3;
            let owner = o.owner.unwrap_or(0xff);
            assert_eq!(
                read.body,
                expected(t.class, t.subclass, identifier, owner),
                "class {} subclass {}",
                t.class,
                t.subclass
            );
        }
    }

    #[test]
    fn invalid_extras_are_rejected() {
        let error = |what, value| Err(WriteError::OutOfRange { what, value });
        let tile = |level, tile| GivenTile {
            level,
            tile,
            terrain: Terrain::Sand,
            picture: 0,
            river: None,
            road: None,
            flags: 0,
        };
        let tiles = |tiles| Extras {
            tiles,
            objects: vec![],
        };
        let objects = |objects| Extras {
            tiles: vec![],
            objects,
        };
        let map = smoke_like_map();
        assert_eq!(
            write_with(&map, &tiles(vec![tile(0, (36, 0))])),
            error("given tile", 36 << 8)
        );
        assert_eq!(
            write_with(&map, &tiles(vec![tile(0, (0, 36))])),
            error("given tile", 36)
        );
        assert_eq!(
            write_with(&map, &tiles(vec![tile(1, (0, 0))])),
            error("given tile", 1 << 16)
        );
        let twice = vec![tile(1, (3, 4)), tile(0, (3, 4)), tile(1, (3, 4))];
        assert_eq!(
            write_with(&with_underground(), &tiles(twice)),
            error("tile given twice", 1 << 16 | 3 << 8 | 4)
        );
        let boat = || object(0, (5, 5), template(8, 0));
        let at = |level, tile| AnyObject {
            level,
            tile,
            ..boat()
        };
        let past = "object tile (past the edges)";
        assert!(write_with(&map, &objects(vec![at(0, (42, 40))])).is_ok());
        assert_eq!(
            write_with(&map, &objects(vec![at(0, (43, 0))])),
            error(past, 43 << 8)
        );
        assert_eq!(
            write_with(&map, &objects(vec![at(0, (0, 41))])),
            error(past, 41)
        );
        assert_eq!(
            write_with(&map, &objects(vec![at(1, (0, 0))])),
            error(past, 1 << 16)
        );
        for class in HERO_CLASSES {
            let hero = object(0, (5, 5), template(class, 0));
            assert_eq!(
                write_with(&map, &objects(vec![hero])),
                error("object class (names a hero)", u64::from(class))
            );
        }
        let owned = |class, subclass, owner| AnyObject {
            owner: Some(owner),
            ..object(0, (5, 5), template(class, subclass))
        };
        let without = "owner of an object without one (class)";
        assert_eq!(
            write_with(&map, &objects(vec![owned(8, 0, 0)])),
            error(without, 8)
        );
        assert_eq!(
            write_with(&map, &objects(vec![owned(220, 7, 0)])),
            error(without, 220)
        );
        assert_eq!(
            write_with(&map, &objects(vec![owned(53, 1, 2)])),
            error("object owner (no playable player)", 2)
        );
        assert!(write_with(&map, &objects(vec![owned(53, 1, 1)])).is_ok());
    }
}
