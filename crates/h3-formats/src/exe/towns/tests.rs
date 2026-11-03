use super::*;
use crate::exe::build_test_image;

/// A payload under construction: the bytes and where each table starts (virtual addresses from
/// `build_test_image`'s RVA 0x1000, image base 0x400000).
#[derive(Default)]
struct Payload {
    bytes: Vec<u8>,
}

impl Payload {
    fn va(&self) -> u32 {
        0x0040_1000 + u32::try_from(self.bytes.len()).unwrap()
    }

    fn ints(&mut self, values: &[i32]) -> u32 {
        let va = self.va();
        for v in values {
            self.bytes.extend_from_slice(&v.to_le_bytes());
        }
        va
    }

    /// A list: each entry's ids and −1, then −100.
    fn list(&mut self, entries: &[&[i32]]) -> u32 {
        let mut words = Vec::new();
        for e in entries {
            words.extend_from_slice(e);
            words.push(ENTRY_END);
        }
        words.push(LIST_END);
        self.ints(&words)
    }
}

/// Two factions' tables and the shared ones; the offsets of a few values for the error cases.
struct Fixture {
    bytes: Vec<u8>,
    tables: TownTables,
}

fn fixture() -> Fixture {
    let mut p = Payload::default();
    let building_lists = vec![
        p.list(&[&[0], &[1, 0], &[30, 7, 5]]),
        p.list(&[&[7], &[30, 7]]),
    ];
    let common_supersedes = p.list(&[&[1, 0], &[37, 30]]);
    let supersede_lists = vec![p.list(&[&[18, 30], &[19, 37, 18]]), p.list(&[])];
    let mut slots = [i32::from(BUILDING_IDS); 2 * MAP_SLOTS];
    slots[0] = 11;
    slots[MAP_SLOTS + 3] = 7;
    let map_building_ids = p.ints(&slots);
    let creatures = p.ints(&(0..28).collect::<Vec<_>>());
    let hordes = p.va();
    for (creature, growth, level) in [
        (4i32, 3i16, 0i16),
        (-1, 0, 0),
        (-1, 0, 0),
        (-1, 0, 0),
        (16, 4, 0),
        (-1, 0, 0),
        (22, 2, 9),
        (-1, 0, 0),
    ] {
        p.bytes.extend_from_slice(&creature.to_le_bytes());
        p.bytes.extend_from_slice(&growth.to_le_bytes());
        p.bytes.extend_from_slice(&level.to_le_bytes());
    }
    let horde_buildings = p.ints(&[18, 19, 24, 25]);
    let silo = p.ints(&[1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0]);
    let garrison = [
        p.ints(&[33, 33, 20, 13]),
        p.ints(&[8, 5, 3, 1]),
        p.ints(&[15, 7, 5, 3]),
    ];
    let code = p.va();
    p.bytes.extend_from_slice(&[0x83, 0xF8, 30]);
    Fixture {
        bytes: p.bytes,
        tables: TownTables {
            building_lists: building_lists.leak(),
            common_supersedes,
            supersede_lists: supersede_lists.leak(),
            map_building_ids,
            creatures,
            hordes,
            horde_buildings,
            silo,
            garrison,
            dwelling_2_chance: Immediate {
                va: code,
                opcode: &[0x83, 0xF8],
            },
        },
    }
}

fn entry(building: u8, listed: &[u8]) -> BuildingEntry {
    BuildingEntry {
        building,
        listed: listed.to_vec(),
    }
}

fn read(bytes: &[u8], t: &TownTables) -> Result<TownData, FormatError> {
    let img = build_test_image(1, bytes);
    town_data_at(&PeImage::parse(&img).unwrap(), t)
}

/// The file offset in the payload of a virtual address.
fn at(va: u32) -> usize {
    usize::try_from(va - 0x0040_1000).unwrap()
}

#[test]
fn reads_the_town_tables_from_a_synthetic_image() {
    let f = fixture();
    let data = read(&f.bytes, &f.tables).unwrap();
    let mut slots_0 = vec![None; MAP_SLOTS];
    slots_0[0] = Some(11);
    let mut slots_1 = vec![None; MAP_SLOTS];
    slots_1[3] = Some(7);
    let none = HordeRecord {
        creature: None,
        growth: 0,
    };
    assert_eq!(
        data,
        TownData {
            factions: vec![
                FactionTowns {
                    buildings: vec![entry(0, &[]), entry(1, &[0]), entry(30, &[7, 5])],
                    supersedes: vec![entry(18, &[30]), entry(19, &[37, 18])],
                    map_building_ids: slots_0,
                    creatures: (0..14).collect(),
                    hordes: vec![
                        HordeRecord {
                            creature: Some(4),
                            growth: 3
                        },
                        none,
                        none,
                        none
                    ],
                    silo: vec![1, 0, 1, 0, 0, 0, 0],
                },
                FactionTowns {
                    buildings: vec![entry(7, &[]), entry(30, &[7])],
                    supersedes: vec![],
                    map_building_ids: slots_1,
                    creatures: (14..28).collect(),
                    hordes: vec![
                        HordeRecord {
                            creature: Some(16),
                            growth: 4
                        },
                        none,
                        HordeRecord {
                            creature: Some(22),
                            growth: 2
                        },
                        none
                    ],
                    silo: vec![0, 0, 0, 0, 1, 0, 0],
                },
            ],
            common_supersedes: vec![entry(1, &[0]), entry(37, &[30])],
            horde_buildings: vec![18, 19, 24, 25],
            garrison: vec![
                GarrisonLevel {
                    chance: 33,
                    least: 8,
                    most: 15
                },
                GarrisonLevel {
                    chance: 33,
                    least: 5,
                    most: 7
                },
                GarrisonLevel {
                    chance: 20,
                    least: 3,
                    most: 5
                },
                GarrisonLevel {
                    chance: 13,
                    least: 1,
                    most: 3
                },
            ],
            dwelling_2_chance: 30,
        }
    );
    // The known build's addresses: an image of another build is refused before them.
    assert!(town_data(&build_test_image(1, &f.bytes)).is_err());
}

#[test]
fn malformed_values_are_refused() {
    let f = fixture();
    let t = f.tables;
    let set = |va: u32, index: usize, v: i32| {
        let mut b = f.bytes.clone();
        let o = at(va) + 4 * index;
        b[o..o + 4].copy_from_slice(&v.to_le_bytes());
        read(&b, &t).unwrap_err()
    };
    let out = |what: &'static str, value: u64| FormatError::OutOfRange { what, value };
    // A building id outside 0 to 43 in a list, in an entry's head and among its ids.
    assert_eq!(
        set(t.building_lists[0], 0, 44),
        out("town building list", 44)
    );
    assert_eq!(
        set(t.supersede_lists[0], 1, -3),
        out("town supersede list", 3)
    );
    // The map's slot: 44 is none, 45 is refused.
    assert_eq!(set(t.map_building_ids, 5, 45), out("map building ids", 45));
    assert_eq!(set(t.creatures, 2, -1), out("town creatures", 1));
    // A horde's creature below −1, a negative growth.
    assert_eq!(set(t.hordes, 0, -2), out("town hordes", 2));
    assert_eq!(set(t.hordes, 1, 0x8000), out("horde growth", 0x8000));
    assert_eq!(set(t.horde_buildings, 3, 44), out("horde buildings", 44));
    assert_eq!(set(t.silo, 6, -1), out("town silo", 1));
    assert_eq!(set(t.garrison[1], 0, -8), out("neutral garrison", 8));
    assert_eq!(
        set(t.garrison[0], 0, 101),
        out("neutral garrison chance", 101)
    );
    // The instruction: another opcode, a chance above 100.
    let mut b = f.bytes.clone();
    b[at(t.dwelling_2_chance.va)] = 0x3D;
    assert_eq!(
        read(&b, &t).unwrap_err(),
        FormatError::BadMagic("the level-2 dwelling's instruction")
    );
    let mut b = f.bytes.clone();
    b[at(t.dwelling_2_chance.va) + 2] = 101;
    assert_eq!(
        read(&b, &t).unwrap_err(),
        out("level-2 dwelling chance", 101)
    );
}

#[test]
fn malformed_lists_and_tables_are_refused() {
    let f = fixture();
    let t = f.tables;
    let too_long = |what| FormatError::OutOfRange {
        what,
        value: MAX_LIST as u64 + 1,
    };
    // A list of 65 entries, an entry of 65 ids after its building (appended, the list's address
    // moved there).
    let mut p = Payload {
        bytes: f.bytes.clone(),
    };
    let entries = vec![[5i32]; MAX_LIST + 1];
    let long = p.list(&entries.iter().map(|e| &e[..]).collect::<Vec<_>>());
    let wide = p.list(&[&[0; MAX_LIST + 2]]);
    let lists = |first: u32| -> &'static [u32] { vec![first, t.building_lists[1]].leak() };
    for (va, case) in [(long, "entries"), (wide, "ids")] {
        let err = read(
            &p.bytes,
            &TownTables {
                building_lists: lists(va),
                ..t
            },
        )
        .unwrap_err();
        assert_eq!(err, too_long("town building list"), "{case}");
    }
    // An unterminated list runs into the end of the image.
    let mut p = Payload {
        bytes: f.bytes.clone(),
    };
    let open = p.ints(&[0, 1, 0]);
    assert!(matches!(
        read(
            &p.bytes,
            &TownTables {
                common_supersedes: open,
                ..t
            }
        )
        .unwrap_err(),
        FormatError::Truncated { .. }
    ));
    // Unmapped tables, a truncated row, a supersede list too few, a truncated instruction.
    let end = 0x0040_1000 + u32::try_from(f.bytes.len()).unwrap();
    for bad in [
        TownTables {
            map_building_ids: 0x10,
            ..t
        },
        TownTables { hordes: 0x10, ..t },
        TownTables { silo: end - 8, ..t },
        TownTables {
            hordes: end - 4,
            ..t
        },
        TownTables {
            supersede_lists: &[],
            ..t
        },
        TownTables {
            dwelling_2_chance: Immediate {
                va: end - 1,
                opcode: &[0x83, 0xF8],
            },
            ..t
        },
    ] {
        assert!(read(&f.bytes, &bad).is_err(), "{bad:?}");
    }
    assert_eq!(
        read(
            &f.bytes,
            &TownTables {
                supersede_lists: &[0x0040_1000],
                ..t
            }
        )
        .unwrap_err(),
        FormatError::OutOfRange {
            what: "town supersede lists",
            value: 1
        }
    );
}
