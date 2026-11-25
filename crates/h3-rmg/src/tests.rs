//! Unit tests of the settings, the templates kept and the positions (E14 T2c). `generate` over
//! the packs' tables is tested in `h3-pack-rmg`, which compiles them.

extern crate std;

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use std::string::ToString;

use h3_kernel::rng::CrtRand;

use super::*;

fn zone(id: i32, kind: ZoneKind, humans: [i32; 2], players: [i32; 2], ownership: i32) -> Zone {
    Zone {
        id,
        kind,
        base_size: 10,
        humans,
        players,
        ownership,
        player_towns: [0; 4],
        neutral_towns: [0; 4],
        same_type: false,
        town_types: 0x1FF,
        mines: [0; 7],
        mine_density: [0; 7],
        terrain_match: false,
        terrains: 1,
        strength: Strength::Average,
        monster_match: false,
        neutral_monsters: false,
        monster_towns: 0x1FF,
        treasure: [[0; 3]; 3],
    }
}

fn connection(zones: [i32; 2], humans: [i32; 2], players: [i32; 2]) -> Connection {
    Connection {
        zones,
        value: 100,
        wide: false,
        border_guard: false,
        humans,
        players,
    }
}

fn template(sizes: [i32; 2], zones: Vec<Zone>, connections: Vec<Connection>) -> Template {
    Template {
        name: String::from("t"),
        sizes,
        zones,
        connections,
    }
}

const ANY: [i32; 2] = [0, 8];

fn choices() -> Choices {
    let mut humans = [false; COLOURS];
    humans[0] = true;
    Choices {
        size: MapSize::Medium,
        two_levels: false,
        humans,
        towns: [None; COLOURS],
        players: Some(2),
        teams: Some(2),
        computers: Some(0),
        computer_teams: Some(0),
        water: Water::Normal,
        monsters: Some(Monsters::Normal),
        format: MapFormat::SoD,
    }
}

fn settings() -> Settings {
    resolve_random(&choices(), &mut CrtRand::new(0)).unwrap()
}

// covers: RMG-SET-004, RMG-SET-005
#[test]
fn the_choices_have_the_originals_numbers() {
    assert_eq!(
        [MapSize::Small, MapSize::Medium, MapSize::Large, MapSize::ExtraLarge].map(MapSize::side),
        [36, 72, 108, 144]
    );
    assert_eq!(
        [Water::None, Water::Normal, Water::Islands, Water::Random].map(Water::value),
        [0, 1, 2, 3]
    );
    let monsters = [Monsters::Weak, Monsters::Normal, Monsters::Strong];
    assert_eq!(monsters.map(Monsters::parameter), [-1, 0, 1]);
    assert_eq!(monsters.map(Monsters::strength), [2, 3, 4]);
    assert_eq!(
        [-1, 0, 1, 2, 3, 4].map(MapFormat::from_setup),
        [
            MapFormat::RoE,
            MapFormat::RoE,
            MapFormat::AB,
            MapFormat::SoD,
            MapFormat::SoD,
            MapFormat::RoE
        ]
    );
    assert_eq!(
        [MapFormat::RoE, MapFormat::AB, MapFormat::SoD].map(MapFormat::version),
        [0, 1, 2]
    );
}

// covers: RMG-SET-004
#[test]
fn settings_the_tab_cannot_give_are_refused() {
    let ok = settings();
    assert_eq!(ok.check(), Ok(()));
    let mut humans = [true; COLOURS];
    humans[7] = false;
    let cases: [(Settings, SettingsError); 9] = [
        (Settings { players: 0, ..ok }, SettingsError::Players(0)),
        (Settings { players: 9, ..ok }, SettingsError::Players(9)),
        (
            Settings { humans, ..ok },
            SettingsError::Humans {
                humans: 7,
                players: 2,
            },
        ),
        (
            Settings { teams: 0, ..ok },
            SettingsError::Teams {
                teams: 0,
                players: 2,
            },
        ),
        (
            Settings { teams: 3, ..ok },
            SettingsError::Teams {
                teams: 3,
                players: 2,
            },
        ),
        (
            Settings {
                computers: 7,
                ..ok
            },
            SettingsError::Computers {
                computers: 7,
                players: 2,
            },
        ),
        (
            Settings {
                computer_teams: 1,
                ..ok
            },
            SettingsError::ComputerTeams {
                teams: 1,
                computers: 0,
            },
        ),
        (
            Settings {
                computers: 3,
                computer_teams: 4,
                ..ok
            },
            SettingsError::ComputerTeams {
                teams: 4,
                computers: 3,
            },
        ),
        (
            Settings {
                towns: [None, None, Some(9), None, None, None, None, None],
                ..ok
            },
            SettingsError::Town { colour: 2, town: 9 },
        ),
    ];
    for (settings, error) in cases {
        assert_eq!(settings.check(), Err(error));
        assert!(!error.to_string().is_empty());
    }
    let fine = Settings {
        computers: 3,
        computer_teams: 3,
        towns: [Some(8); COLOURS],
        ..ok
    };
    assert_eq!(fine.check(), Ok(()));
    assert_eq!(fine.human_count(), 1);
}

// covers: RMG-SET-001, RMG-SET-002, RMG-SET-003
#[test]
fn random_choices_are_drawn_as_the_original_draws_them() {
    // `srand(1)`: 41, 18467, 6334, 26500, 19169 (the C runtime's first draws).
    let random = Choices {
        players: None,
        teams: None,
        computers: None,
        computer_teams: None,
        monsters: None,
        ..choices()
    };
    let mut rand = CrtRand::new(1);
    let s = resolve_random(&random, &mut rand).unwrap();
    // 1 + 41 % 8; 18467 % 2 + 1; 6334 % 7 (at most 8 - 2); 26500 % 6 + 1; 19169 % 3.
    assert_eq!(
        (s.players, s.teams, s.computers, s.computer_teams, s.monsters),
        (2, 2, 6, 5, Monsters::Strong)
    );
    assert_eq!(rand.rand(), 15724, "five draws");
    // Each monster strength; no computers draw no computer teams.
    let only = Choices {
        monsters: None,
        ..choices()
    };
    for (k, expected) in [Monsters::Weak, Monsters::Normal, Monsters::Strong]
        .into_iter()
        .enumerate()
    {
        let seed = (0u32..)
            .find(|&s| CrtRand::new(s).rand() % 3 == k as u32)
            .unwrap();
        let s = resolve_random(&only, &mut CrtRand::new(seed)).unwrap();
        assert_eq!(s.monsters, expected);
    }
    let none = Choices {
        computers: Some(0),
        computer_teams: None,
        ..choices()
    };
    let mut rand = CrtRand::new(7);
    let s = resolve_random(&none, &mut rand).unwrap();
    assert_eq!((s.computers, s.computer_teams), (0, 0));
    assert_eq!(rand, CrtRand::new(7), "nothing drawn");
}

// covers: RMG-SET-001, RMG-SET-002
#[test]
fn chosen_counts_are_adjusted_as_the_original_adjusts_them() {
    let resolve = |c: Choices| resolve_random(&c, &mut CrtRand::new(3));
    // Teams 0 or above the players: the players.
    for teams in [0, 5] {
        let s = resolve(Choices {
            teams: Some(teams),
            ..choices()
        })
        .unwrap();
        assert_eq!(s.teams, 2);
    }
    // Computers above 8 less the players; computer teams 0: the computers.
    let s = resolve(Choices {
        computers: Some(7),
        computer_teams: Some(0),
        ..choices()
    })
    .unwrap();
    assert_eq!((s.computers, s.computer_teams), (6, 6));
    let s = resolve(Choices {
        computers: Some(3),
        computer_teams: Some(2),
        ..choices()
    })
    .unwrap();
    assert_eq!((s.computers, s.computer_teams), (3, 2));
    // Eight players: no computers, whatever is chosen, and no draw.
    let mut rand = CrtRand::new(3);
    let s = resolve_random(
        &Choices {
            players: Some(8),
            teams: Some(4),
            computers: None,
            computer_teams: None,
            ..choices()
        },
        &mut rand,
    )
    .unwrap();
    assert_eq!((s.players, s.teams, s.computers, s.computer_teams), (8, 4, 0, 0));
    assert_eq!(rand, CrtRand::new(3));
}

// covers: RMG-SET-001, RMG-SET-002
#[test]
fn choices_the_original_would_go_wrong_on_are_refused() {
    let resolve = |c: Choices| resolve_random(&c, &mut CrtRand::new(3));
    // No players: the teams' draw divides by them (RMG-QUIRK-3).
    assert_eq!(
        resolve(Choices {
            players: Some(0),
            teams: None,
            ..choices()
        }),
        Err(SettingsError::Players(0))
    );
    // Computer teams above the computers: the original gives them the players' count
    // (RMG-QUIRK-6).
    assert_eq!(
        resolve(Choices {
            computers: Some(3),
            computer_teams: Some(5),
            ..choices()
        }),
        Err(SettingsError::ComputerTeams {
            teams: 5,
            computers: 3
        })
    );
    // More humans than the chosen players.
    let mut humans = [false; COLOURS];
    humans[..3].fill(true);
    assert_eq!(
        resolve(Choices {
            humans,
            ..choices()
        }),
        Err(SettingsError::Humans {
            humans: 3,
            players: 2
        })
    );
}

// covers: RMG-TPL-005
#[test]
fn the_size_index_is_the_originals() {
    let size = |side, levels, water| Fit::new(side, levels, water, 2, 1).size;
    assert_eq!(size(36, 1, Water::None), 1);
    assert_eq!(size(72, 2, Water::Normal), 8);
    assert_eq!(size(144, 2, Water::None), 32);
    assert_eq!(size(36, 1, Water::Islands), 1, "halved to 0, raised to 1");
    assert_eq!(size(72, 1, Water::Islands), 2);
    assert_eq!(size(108, 2, Water::Islands), 9);
    assert_eq!(Fit::new(36, 1, Water::None, 3, 2).total, 5);
}

// covers: RMG-TPL-004, RMG-TPL-005
#[test]
fn a_template_is_kept_by_its_range_and_its_zones_that_fit() {
    let fit = Fit::new(72, 1, Water::Normal, 2, 1);
    let starts = || {
        vec![
            zone(1, ZoneKind::HumanStart, [1, 2], [2, 3], 1),
            zone(2, ZoneKind::HumanStart, [2, 2], [3, 3], 2),
            zone(3, ZoneKind::ComputerStart, ANY, ANY, 3),
        ]
    };
    assert!(fit.keeps(&template([4, 4], starts(), vec![])), "4 in 4 to 4");
    assert!(!fit.keeps(&template([5, 8], starts(), vec![])));
    assert!(!fit.keeps(&template([1, 3], starts(), vec![])));
    // A zone outside its ranges is not counted: one human start fewer.
    for (humans, players) in [([3, 4], ANY), ([0, 1], ANY), (ANY, [4, 8]), (ANY, [0, 2])] {
        let mut zones = starts();
        zones[1] = zone(2, ZoneKind::HumanStart, humans, players, 2);
        assert!(!fit.keeps(&template([1, 8], zones, vec![])));
    }
    // Too few starts for the total; treasures and junctions count for nothing.
    let mut zones = starts();
    zones[2] = zone(3, ZoneKind::Treasure, ANY, ANY, 0);
    zones.push(zone(4, ZoneKind::Junction, ANY, ANY, 0));
    assert!(!fit.keeps(&template([1, 8], zones, vec![])));
    let kept = kept_templates(
        &[
            template([5, 8], starts(), vec![]),
            template([1, 8], starts(), vec![]),
            template([4, 4], starts(), vec![]),
        ],
        fit,
    );
    assert_eq!(kept, [1, 2]);
}

// covers: RMG-TPL-003, RMG-TPL-004
#[test]
fn a_kept_templates_zones_are_renumbered_and_its_connections_filtered() {
    let fit = Fit::new(72, 1, Water::Normal, 2, 0);
    let zones = vec![
        zone(7, ZoneKind::HumanStart, ANY, ANY, 1),
        zone(8, ZoneKind::Treasure, [3, 8], ANY, 0),
        zone(9, ZoneKind::HumanStart, ANY, ANY, 2),
        zone(9, ZoneKind::Treasure, ANY, ANY, 0),
    ];
    let connections = vec![
        connection([7, 9], ANY, ANY),
        connection([8, 9], ANY, ANY),
        connection([9, 8], ANY, ANY),
        connection([9, 7], [3, 8], ANY),
        connection([9, 7], ANY, [0, 1]),
        connection([7, 7], ANY, ANY),
    ];
    let t = template([1, 8], zones, connections);
    let kept = KeptTemplate::new(3, &t, fit, MapFormat::SoD, true);
    assert_eq!(kept.index, 3);
    assert_eq!(kept.zones.iter().map(|z| z.id).collect::<Vec<_>>(), [0, 1, 2]);
    assert_eq!(
        kept.zones.iter().map(|z| z.ownership).collect::<Vec<_>>(),
        [1, 2, 0]
    );
    // Zone 8 dropped: the connections naming it go too; id 9 is its first zone.
    assert_eq!(
        kept.connections
            .iter()
            .map(|c| c.zones)
            .collect::<Vec<_>>(),
        [[0, 1], [0, 0]]
    );
    assert_eq!(kept.connections[0].row, t.connections[0]);
    assert!(kept.zones.iter().all(|z| z.town_types == 0x1FF && z.monster_towns == 0x1FF));
    // RoE: Conflux's town type goes, and Fortress's monster town (RMG-QUIRK-11) or Conflux's.
    let roe = KeptTemplate::new(3, &t, fit, MapFormat::RoE, true);
    assert!(roe.zones.iter().all(|z| z.town_types == 0xFF && z.monster_towns == 0x17F));
    let fixed = KeptTemplate::new(3, &t, fit, MapFormat::RoE, false);
    assert!(fixed.zones.iter().all(|z| z.town_types == 0xFF && z.monster_towns == 0xFF));
    let ab = KeptTemplate::new(3, &t, fit, MapFormat::AB, true);
    assert_eq!(ab.zones, kept.zones);
}

fn starts(owners: &[(ZoneKind, i32)]) -> Vec<Zone> {
    (1..)
        .zip(owners)
        .map(|(id, &(kind, ownership))| zone(id, kind, ANY, ANY, ownership))
        .collect()
}

// covers: RMG-TPL-008
#[test]
fn the_players_take_the_human_starts_and_the_computers_the_rest() {
    use ZoneKind::{ComputerStart as C, HumanStart as H, Treasure as T};
    let mut humans = [false; COLOURS];
    humans[2] = true;
    humans[5] = true;
    // Human starts at positions 1 and 4 (and 6, left over); computer starts at 0 and 2.
    let zones = starts(&[(T, 0), (H, 5), (C, 3), (H, 2), (C, 1), (H, 7)]);
    // Humans 2 and 5 first, then 0, 1, 3, ...: three players, two computers.
    assert_eq!(
        positions(&zones, &humans, 3, 2, true),
        Ok([
            Some(1),
            Some(2),
            Some(3),
            None,
            Some(5),
            None,
            Some(0),
            None
        ])
    );
    assert_eq!(
        positions(&zones, &humans, 3, 3, true),
        Err(PositionError::Exhausted)
    );
    // The colours in order: humans first.
    assert_eq!(
        positions(&zones, &humans, 2, 0, true),
        Ok([None, Some(2), None, None, Some(5), None, None, None])
    );
}

// covers: RMG-TPL-008
#[test]
fn a_start_without_ownership_marks_the_eighth_human_start() {
    use ZoneKind::{ComputerStart as C, HumanStart as H};
    let humans = [true, false, false, false, false, false, false, false];
    // AB's 2SM2i(2): human starts at 0 and 1 and two without Ownership (RMG-QUIRK-24).
    let zones = starts(&[(H, 1), (H, 2), (H, 0), (C, 0)]);
    assert_eq!(
        positions(&zones, &humans, 3, 0, true),
        Ok([Some(0), Some(1), None, None, None, None, None, Some(2)])
    );
    // Without the flag such zones mark nothing: the starts run out.
    assert_eq!(
        positions(&zones, &humans, 3, 0, false),
        Err(PositionError::Exhausted)
    );
    assert_eq!(
        positions(&zones, &humans, 2, 0, false),
        Ok([Some(0), Some(1), None, None, None, None, None, None])
    );
    // A fourth player runs past the starts (the original then clears the drawn template's
    // index); so do computers without a start left.
    assert_eq!(
        positions(&zones, &humans, 4, 0, true),
        Err(PositionError::Exhausted)
    );
    assert_eq!(
        positions(&zones, &humans, 2, 1, true),
        Err(PositionError::Exhausted)
    );
}

// covers: RMG-TPL-008
#[test]
fn positions_outside_the_arrays_are_refused() {
    use ZoneKind::{ComputerStart as C, HumanStart as H};
    let humans = [true; COLOURS];
    for (ownership, position) in [(9, 8), (-1, -2), (i32::MIN, i32::MAX)] {
        let zones = starts(&[(H, 1), (C, ownership)]);
        assert_eq!(
            positions(&zones, &humans, 1, 1, true),
            Err(PositionError::Outside { zone: 1, position })
        );
    }
}

fn tables() -> GeneratorTables {
    GeneratorTables {
        heroes_roe: vec![true, true, false, true],
        heroes_expansions: vec![true, false, true, true],
        heroes_campaign: vec![false, false, false, true],
        map_limits: vec![(26, 200), (6, 200), (26, 5), (240, 1)],
        zone_limits: vec![(113, 3)],
    }
}

// covers: RMG-SET-008
#[test]
fn heroes_are_banned_by_the_formats_byte_and_the_campaign_byte() {
    let t = tables();
    assert_eq!(
        t.banned_heroes(MapFormat::RoE),
        [false, false, true, true]
    );
    for format in [MapFormat::AB, MapFormat::SoD] {
        assert_eq!(t.banned_heroes(format), [false, true, false, true]);
    }
}

// covers: RMG-SET-009
#[test]
fn the_limits_are_set_from_the_last_pair_to_the_first() {
    let limits = tables().type_limits();
    // Type 26's first pair wins; type 240 is past the table and sets nothing.
    assert_eq!((limits.map[26], limits.map[6], limits.map[7]), (200, 200, NO_LIMIT));
    assert_eq!((limits.zone[113], limits.zone[26]), (3, NO_LIMIT));
    let mut expected = [NO_LIMIT; OBJECT_TYPES];
    expected[26] = 200;
    expected[6] = 200;
    assert_eq!(limits.map, expected);
}

#[test]
fn no_trace_keeps_nothing() {
    let mut t = NoTrace;
    t.draw(SITE_TEMPLATE, 1);
    t.step_end(0);
    assert_eq!(t, NoTrace);
    assert_eq!((SITE_WATER, SITE_TEMPLATE), (0x537C89, 0x549975));
    assert_eq!(RMG_GENERATOR_VERSION, 1);
}
