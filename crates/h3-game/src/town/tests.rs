//! Tests of a player's start and of the setup's checks of the starts and the towns (E09 T7a2;
//! since T8a1 the towns' count and the main town's owner); the pool and the build's draws in
//! [`pool`] (E09 T7b1); the towns' draws, the store of towns and the start's reveal in [`build`]
//! (E09 T7b2a); the main towns' heroes in [`heroes`] (E09 T7b2b); the step onto a town and its
//! capture in [`capture`] (E09 T8a2).

use alloc::format;
use alloc::string::ToString;
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use core::num::{NonZeroU8, NonZeroU16};

use h3_adventure::{AdventureRuleset, Holds, ObjectSetup, Pos, Scenery, TileMask};
use h3_content::{
    CreatureDef, FactionDef, FactionId, Formation, HeroClassDef, HeroTypeDef, HeroTypeId, Ruleset,
};
use h3_content_battle::{
    BallisticsRecord, BattleCreatureDef, SiegeDef, SiegeFieldDef, WallKind, WallTarget,
};
use h3_contracts::{HeroId, PlayerId, TownId};
use h3_towns::{
    BuildingDef, BuildingRoles, BuildingsSetup, CreatureLevelDef, Dwelling, GarrisonRow, Moat,
    Reveal, SlotSet, TownDef, TownFaction, TownPack, TownParamsDef, TownRuleset, TownSetup,
};

use super::{FactionChoice, FactionSet, MainTown, PlayerStart};
use crate::tests::{adventure_pack, battle_part_with, core_pack, hero_type, key, manifest, setup};
use crate::{GameRuleset, GameSetup, GameSetupError, GameState, PlayerSetup};

#[test]
fn a_faction_set_holds_the_factions_below_its_capacity() {
    let faction = |index| FactionId::from_index(index).unwrap();
    assert!(FactionSet::EMPTY.is_empty());
    assert_eq!(FactionSet::default(), FactionSet::EMPTY);
    let set = FactionSet::EMPTY
        .with(faction(8))
        .with(faction(0))
        .with(faction(8));
    assert_eq!((set.len(), set.is_empty()), (2, false));
    assert!(set.contains(faction(0)) && set.contains(faction(8)) && !set.contains(faction(1)));
    assert_eq!(set.iter().collect::<Vec<_>>(), [faction(0), faction(8)]);
    let last = faction(FactionSet::CAPACITY - 1);
    assert_eq!(FactionSet::EMPTY.with(last).iter().last(), Some(last));
    // A faction past the capacity is in no set.
    let past = faction(FactionSet::CAPACITY);
    assert_eq!(set.with(past), set);
    assert!(!set.with(past).contains(past));
}

#[test]
fn a_players_start_defaults_to_none_of_it() {
    assert_eq!(
        PlayerStart::default(),
        PlayerStart {
            factions: FactionSet::EMPTY,
            choice: None,
            pick: None,
            main_town: None,
            random_hero: false,
        }
    );
}

/// The town tests' core: [`core_pack`]'s with the imp and the gog `t:town`'s levels 1 and 2, the
/// faction `t:other` with the class `t:cleric`, and after the five knights the cleric `t:c0` and
/// the knights `t:k0` to `t:k16`. With `elf`, the extra creature `t:elf` is `t:other`'s level 1
/// and its copy `t:ent` its level 2.
fn towns_core(elf: bool) -> Ruleset {
    let mut pack = core_pack(elf);
    if elf {
        let ent = CreatureDef {
            key: key("t:ent"),
            ..pack.creatures[3].clone()
        };
        pack.creatures.push(ent);
    }
    // The imp, the gog, the cart (none) and the elf.
    let levels = [
        Some(("t:town", 1)),
        Some(("t:town", 2)),
        None,
        Some(("t:other", 1)),
        Some(("t:other", 2)),
    ];
    for (creature, (faction, level)) in pack
        .creatures
        .iter_mut()
        .zip(levels)
        .filter_map(|(c, l)| l.map(|l| (c, l)))
    {
        creature.info.faction = Some(key(faction));
        creature.info.level = level;
    }
    pack.factions.push(FactionDef {
        key: key("t:other"),
        name: "Other".into(),
        native_terrain: None,
    });
    let cleric = HeroClassDef {
        key: key("t:cleric"),
        faction: key("t:other"),
        ..pack.hero_classes[0].clone()
    };
    pack.hero_classes.push(cleric);
    pack.heroes.push(HeroTypeDef {
        class: key("t:cleric"),
        ..hero_type("t:c0", &[], &[])
    });
    pack.heroes
        .extend((0..17).map(|i| hero_type(&format!("t:k{i}"), &[], &[])));
    Ruleset::compile(&[pack], "t:faithful").unwrap()
}

/// The battle tests' siege field (`h3-battle`'s `tests::support::siege_field`).
fn siege_field() -> SiegeFieldDef {
    SiegeFieldDef {
        walls: vec![12, 29, 45, 62, 78, 96, 112, 130, 147, 165, 182],
        gate: 96,
        moat: vec![11, 28, 44, 61, 77, 95, 111, 129, 146, 164, 181],
        second_moat: vec![10, 27, 43, 60, 76, 94, 110, 128, 145, 163, 180],
        moat_defense_penalty: 3,
        mine_power: 3,
        teleport_moat: 2,
        teleport_walls: 3,
        catapult: 119,
        turret_count: 4,
        targets: [
            (WallKind::Tower, None, Some(255)),
            (WallKind::Wall, Some(29), None),
            (WallKind::Wall, Some(78), None),
            (WallKind::Gate, Some(96), None),
            (WallKind::Wall, Some(130), None),
            (WallKind::Wall, Some(182), None),
            (WallKind::Tower, None, Some(251)),
            (WallKind::Keep, None, Some(254)),
        ]
        .map(|(kind, wall, turret)| WallTarget { kind, wall, turret })
        .to_vec(),
        missed: vec![1, 2, 5, 4],
        automatic: vec![1, 2, 4, 5],
        fallback: vec![3, 7, 0, 6],
        castle_bonus: 1,
        ballistics: [
            ([5, 10, 25, 50], 1, [10, 60, 30]),
            ([7, 15, 30, 60], 1, [0, 50, 50]),
            ([7, 15, 30, 60], 2, [0, 50, 50]),
            ([10, 20, 40, 75], 2, [0, 0, 100]),
        ]
        .map(|(hit, shots, damage)| BallisticsRecord { hit, shots, damage })
        .to_vec(),
    }
}

/// `t:town`'s town record: the roles every record needs, its two levels the imp and the gog; with
/// `elf`, `t:other`'s too, its two levels the elf and the ent.
/// The map slots of [`town_pack`]'s Capitol (`b3`, its hall 4; its part demotes it to `b2`), its
/// Grail building (`b9`, it reveals every tile), its silo (`b10`) and its lookout (`b11`, it
/// reveals [`LOOKOUT_RADIUS`]); its other buildings have none.
pub(super) const CAPITOL_SLOT: u8 = 13;
pub(super) const GRAIL_SLOT: u8 = 26;
pub(super) const SILO_SLOT: u8 = 15;
pub(super) const LOOKOUT_SLOT: u8 = 21;
/// [`town_pack`]'s lookout's reveal radius; its part's is 5.
pub(super) const LOOKOUT_RADIUS: u8 = 20;

fn town_pack(elf: bool) -> TownPack {
    let mut roles: Vec<BuildingRoles> = (1..=4)
        .map(|level| BuildingRoles {
            hall: Some(level),
            ..BuildingRoles::default()
        })
        .collect();
    roles.extend((1..=3).map(|level| BuildingRoles {
        fort: Some(level),
        ..BuildingRoles::default()
    }));
    roles.extend((1..=2).map(|level| BuildingRoles {
        dwelling: Some(Dwelling {
            level,
            upgraded: false,
        }),
        ..BuildingRoles::default()
    }));
    roles.push(BuildingRoles {
        grail: true,
        reveal: Some(Reveal::All),
        ..BuildingRoles::default()
    });
    roles.push(BuildingRoles {
        silo: true,
        ..BuildingRoles::default()
    });
    roles.push(BuildingRoles {
        reveal: Some(Reveal::Radius(LOOKOUT_RADIUS)),
        ..BuildingRoles::default()
    });
    let buildings = roles
        .into_iter()
        .zip(0_u8..)
        .map(|(roles, i)| BuildingDef {
            key: format!("b{i}"),
            name: "Building".into(),
            slot: [
                (3, CAPITOL_SLOT),
                (9, GRAIL_SLOT),
                (10, SILO_SLOT),
                (11, LOOKOUT_SLOT),
            ]
            .into_iter()
            .find_map(|(building, slot)| (building == i).then_some(slot)),
            supersedes: vec![],
            forbidden_with: None,
            auto_with: vec![],
            roles,
        })
        .collect();
    let row = GarrisonRow {
        chance: 100,
        count: [1, 1],
    };
    let town = TownDef {
        faction: key("t:town"),
        moat: Moat::Absent,
        siege_obstacles: false,
        silo: vec![(key("t:gold"), 1)],
        creatures: vec![
            CreatureLevelDef {
                base: key("t:imp"),
                upgrade: None,
            },
            CreatureLevelDef {
                base: key("t:gog"),
                upgrade: None,
            },
        ],
        buildings,
    };
    let mut towns = vec![town.clone()];
    if elf {
        towns.push(TownDef {
            faction: key("t:other"),
            creatures: ["t:elf", "t:ent"]
                .map(|k| CreatureLevelDef {
                    base: key(k),
                    upgrade: None,
                })
                .to_vec(),
            ..town
        });
    }
    TownPack {
        params: Some(TownParamsDef {
            gold: key("t:gold"),
            hall_gold: [500, 1000, 2000, 4000],
            grail_gold: 5000,
            reveal_radius: 5,
            level_2_dwelling_chance: 50,
            neutral_garrison: [row; 4],
            capitol_demoted_to: "b2".into(),
        }),
        towns,
        ..TownPack::new(manifest())
    }
}

/// The game's rules on [`towns_core`]: `t:town` has a town record and its siege, `t:other` none.
fn towns_rules() -> GameRuleset {
    towns_rules_with(false)
}

/// [`towns_rules`]; with `elf`, `t:other` has a town record and a siege too ([`town_pack`]).
fn towns_rules_with(elf: bool) -> GameRuleset {
    towns_rules_on(elf, 4)
}

/// [`towns_rules_with`] whose adventure part takes maps of up to `side` tiles a side (E09
/// T7b2a: the start's reveal), not 4, and of two levels (E09 T8a2: a reveal of every level).
fn towns_rules_on(elf: bool, side: u16) -> GameRuleset {
    let core = Arc::new(towns_core(elf));
    let factions: &[&str] = if elf {
        &["t:town", "t:other"]
    } else {
        &["t:town"]
    };
    let battle = battle_part_with(&core, 7, |pack| {
        if elf {
            let ent = BattleCreatureDef {
                key: key("t:ent"),
                ..pack.creatures[3].clone()
            };
            pack.creatures.push(ent);
        }
        pack.battle.as_mut().unwrap().siege = Some(siege_field());
        pack.sieges = factions
            .iter()
            .map(|faction| SiegeDef {
                faction: key(faction),
                moat_damage: 70,
                second_moat: false,
                mines: None,
                hit_points: vec![2; 8],
            })
            .collect();
    });
    let towns = TownRuleset::compile(&core, &[town_pack(elf)]).unwrap();
    let mut adventure = adventure_pack(vec![88, 25], 2);
    let params = adventure.params.as_mut().unwrap();
    params.max_map_size = NonZeroU16::new(side).unwrap();
    params.max_levels = NonZeroU8::new(2).unwrap();
    GameRuleset::new(
        Arc::clone(&core),
        Arc::new(AdventureRuleset::compile(&core, &[adventure]).unwrap()),
        Arc::new(battle),
        Arc::new(towns),
    )
    .unwrap()
}

fn faction(rules: &GameRuleset, k: &str) -> FactionId {
    rules.core().faction_by_key(k).unwrap()
}

fn hero(rules: &GameRuleset, k: &str) -> HeroTypeId {
    rules.core().hero_type_by_key(k).unwrap()
}

/// A town of `faction` with the standard buildings.
fn town(faction: TownFaction) -> TownSetup {
    TownSetup {
        faction,
        buildings: BuildingsSetup::Standard { fort: true },
        garrison: None,
        name: None,
        events: vec![],
    }
}

/// A town object of the part's `t:town` on `(x, y)`, a single tile its entrance, owned by
/// `owner` at the start (E09 T8a1).
pub(super) fn town_object(
    rules: &GameRuleset,
    (x, y): (u16, u16),
    owner: Option<u8>,
) -> ObjectSetup {
    ObjectSetup {
        anchor: Pos::new(x, y, 0),
        blocked: TileMask::from_bits(1),
        visitable: TileMask::from_bits(1),
        object_type: rules.adventure().object_type_by_key("t:town"),
        amount: None,
        drawn: TileMask::default(),
        monster: None,
        guards: None,
        scenery: Scenery::Ownable,
        holds: Holds::Town {
            formation: Formation::Loose,
        },
        owner: owner.map(PlayerId::new),
    }
}

/// [`setup`] with towns that passes the checks: a `t:town` town on (1, 1) (red's and red's main
/// town, which generates no hero; red's starting hero is random, its choice `t:town`) and a random
/// town aligned to blue on (3, 0), blue's, whose choice is random among `t:town`; every hero type
/// allowed.
fn towns_setup(rules: &GameRuleset) -> GameSetup {
    let castle = faction(rules, "t:town");
    let mut setup = setup();
    setup.towns = vec![
        town(TownFaction::Fixed(castle)),
        town(TownFaction::Random { alignment: Some(1) }),
    ];
    setup.map.objects = vec![
        town_object(rules, (1, 1), Some(0)),
        town_object(rules, (3, 0), Some(1)),
    ];
    setup.players[0].start = start(castle, false);
    setup.players[1].start = PlayerStart {
        factions: FactionSet::EMPTY.with(castle),
        choice: Some(FactionChoice::Random),
        ..PlayerStart::default()
    };
    setup.allowed_heroes = (0..rules.core().hero_types().len())
        .filter_map(HeroTypeId::from_index)
        .collect();
    setup
}

/// A start with `castle` fixed, the town 0 its main town, generating a hero or not, and a random
/// starting hero without a pick.
fn start(castle: FactionId, generates_hero: bool) -> PlayerStart {
    PlayerStart {
        factions: FactionSet::EMPTY.with(castle),
        choice: Some(FactionChoice::Fixed(castle)),
        pick: None,
        main_town: Some(MainTown {
            town: TownId::new(0),
            generates_hero,
        }),
        random_hero: true,
    }
}

/// The error `edit` makes [`towns_setup`] fail with, checking it reads as `text`.
fn refused(rules: &GameRuleset, text: &str, edit: impl FnOnce(&mut GameSetup)) -> GameSetupError {
    let mut setup = towns_setup(rules);
    edit(&mut setup);
    let error = setup.validate(rules).unwrap_err();
    assert_eq!(error.to_string(), text);
    error
}

// covers: TWN-SET-001, TWN-SET-003, TWN-SET-005
#[test]
fn the_towns_and_the_pool_are_part_of_the_state() {
    let rules = towns_rules();
    let mut full = towns_setup(&rules);
    full.players[0].start.pick = Some(hero(&rules, "t:loner"));
    full.hero_masks = vec![(hero(&rules, "t:loner"), 0b01)];
    full.prisoners = vec![hero(&rules, "t:crowd")];
    full.towns[0].buildings = BuildingsSetup::Custom {
        built: SlotSet::EMPTY,
        forbidden: SlotSet::EMPTY,
    };
    assert_eq!(full.validate(&rules), Ok(()));
    let built = |s| GameState::new(&rules, s, 7).unwrap();
    let full_game = built(&full);
    // The store of towns is part of the state (E09 T7b2a): a town's static part (its name) and
    // its record (a custom set's silo, which reveals nothing and draws nothing), the adventure's
    // state the same.
    let mut named = full.clone();
    named.towns[0].name = Some("Steadwick".to_string());
    let mut silo = full.clone();
    silo.towns[0].buildings = BuildingsSetup::Custom {
        built: SlotSet::EMPTY.with(SILO_SLOT),
        forbidden: SlotSet::EMPTY,
    };
    for other in [&named, &silo] {
        let game = built(other);
        assert_ne!(game.towns(), full_game.towns());
        assert_ne!(game.state_hash(), full_game.state_hash());
        assert_eq!(
            game.adventure().state_hash(),
            full_game.adventure().state_hash()
        );
    }
    // The pool is part of the state (E09 T7b1).
    let mut plain = full.clone();
    plain.prisoners.clear();
    assert_ne!(built(&plain).state_hash(), full_game.state_hash());
}

// covers: TWN-SET-002
#[test]
fn a_fixed_towns_faction_has_a_town_record() {
    let rules = towns_rules();
    let other = faction(&rules, "t:other");
    let error = refused(&rules, "town 1's faction has no town record", |s| {
        s.towns[1].faction = TownFaction::Fixed(other);
    });
    assert_eq!(error, GameSetupError::TownFaction(TownId::new(1)));
    // A faction past the core's has none either; the first such town is named, before a town
    // aligned to no player's colour.
    let error = refused(&rules, "town 0's faction has no town record", |s| {
        s.towns[0].faction = TownFaction::Fixed(FactionId::from_raw(40));
        s.towns[1].faction = TownFaction::Fixed(other);
        s.towns
            .push(town(TownFaction::Random { alignment: Some(2) }));
        s.map.objects.push(town_object(&rules, (2, 0), None));
    });
    assert_eq!(error, GameSetupError::TownFaction(TownId::new(0)));
}

// covers: TWN-SET-002
#[test]
fn a_random_town_is_aligned_to_a_players_colour_or_none() {
    let rules = towns_rules();
    // Colour 2 is a colour of the ruleset's but of no player's.
    let error = refused(&rules, "town 1 is aligned to no player's colour", |s| {
        s.towns[1].faction = TownFaction::Random { alignment: Some(2) };
    });
    assert_eq!(error, GameSetupError::TownAlignment(TownId::new(1)));
    // Checked before the players' choices: blue without a choice is not reached.
    let error = refused(&rules, "town 1 is aligned to no player's colour", |s| {
        s.towns[1].faction = TownFaction::Random { alignment: Some(7) };
        s.players[1].start.choice = None;
        s.players[1].start.main_town = s.players[0].start.main_town;
    });
    assert_eq!(error, GameSetupError::TownAlignment(TownId::new(1)));
    let mut setup = towns_setup(&rules);
    setup.towns[1].faction = TownFaction::Random { alignment: None };
    setup
        .towns
        .push(town(TownFaction::Random { alignment: Some(0) }));
    setup.map.objects.push(town_object(&rules, (2, 0), None));
    assert_eq!(setup.validate(&rules), Ok(()));
}

// covers: TWN-SET-001
#[test]
fn a_players_choice_is_among_its_factions_which_have_town_records() {
    let rules = towns_rules();
    let castle = faction(&rules, "t:town");
    let other = faction(&rules, "t:other");
    let text = "player 0's faction choice is invalid";
    let choice = |edit: fn(&mut PlayerStart, FactionId, FactionId)| {
        let error = refused(&rules, text, |s| {
            edit(&mut s.players[0].start, castle, other)
        });
        assert_eq!(error, GameSetupError::Choice(PlayerId::new(0)));
    };
    // A fixed faction outside the set, with a town record or without.
    choice(|start, _, other| start.choice = Some(FactionChoice::Fixed(other)));
    choice(|start, _, _| start.factions = FactionSet::EMPTY);
    // A faction of the set without a town record, whatever the choice (TWN-QUIRK-4).
    choice(|start, _, other| start.factions = start.factions.with(other));
    choice(|start, _, other| {
        start.factions = start.factions.with(other);
        start.choice = Some(FactionChoice::Random);
    });
    // Random among none.
    choice(|start, _, _| {
        start.factions = FactionSet::EMPTY;
        start.choice = Some(FactionChoice::Random);
    });
    // None while the start needs a faction: a main town (generating no hero) or the random-hero
    // flag.
    choice(|start, _, _| {
        start.choice = None;
        start.random_hero = false;
    });
    choice(|start, _, _| {
        start.choice = None;
        start.main_town = None;
    });
    // A random hero of its own, refused before any draw.
    let error = refused(&rules, "player 1's faction choice is invalid", |s| {
        s.towns[1].faction = TownFaction::Random { alignment: None };
        s.map.objects[1].owner = None;
        s.players[1].start = PlayerStart::default();
        s.heroes[1].hero_type = None;
    });
    assert_eq!(error, GameSetupError::Choice(PlayerId::new(1)));
    // A random town aligned to its colour.
    let error = refused(&rules, "player 1's faction choice is invalid", |s| {
        s.players[1].start = PlayerStart::default();
    });
    assert_eq!(error, GameSetupError::Choice(PlayerId::new(1)));
    // Since E09 T7b2a a random town it owns at the start, aligned to no colour or to another
    // player's (TWN-SET-002: it takes its owner's faction unless aligned).
    for alignment in [None, Some(0)] {
        let error = refused(&rules, "player 1's faction choice is invalid", |s| {
            s.towns[1].faction = TownFaction::Random { alignment };
            s.players[1].start = PlayerStart::default();
        });
        assert_eq!(error, GameSetupError::Choice(PlayerId::new(1)));
    }
    // A start that needs no faction needs no choice (the default start): blue's town neutral.
    let mut setup = towns_setup(&rules);
    setup.towns[1].faction = TownFaction::Random { alignment: None };
    setup.map.objects[1].owner = None;
    setup.players[1].start = PlayerStart::default();
    setup.heroes[0].hero_type = None;
    assert_eq!(setup.validate(&rules), Ok(()));
    // A fixed town it owns reads no choice.
    setup.towns[1].faction = TownFaction::Fixed(castle);
    setup.map.objects[1].owner = Some(PlayerId::new(1));
    assert_eq!(setup.validate(&rules), Ok(()));
}

// covers: TWN-SET-001
#[test]
fn a_main_town_is_one_of_the_setups_towns() {
    let rules = towns_rules();
    let error = refused(
        &rules,
        "player 0's main town is not a town of its own",
        |s| {
            s.players[0].start.main_town.as_mut().unwrap().town = TownId::new(2);
        },
    );
    assert_eq!(error, GameSetupError::MainTown(PlayerId::new(0)));
    // Checked after the choices, before the picks.
    let error = refused(&rules, "player 0's faction choice is invalid", |s| {
        s.players[0].start.main_town.as_mut().unwrap().town = TownId::new(2);
        s.players[0].start.choice = None;
    });
    assert_eq!(error, GameSetupError::Choice(PlayerId::new(0)));
    let error = refused(
        &rules,
        "player 0's main town is not a town of its own",
        |s| {
            s.players[0].start.main_town.as_mut().unwrap().town = TownId::new(2);
            s.players[0].start.pick = Some(HeroTypeId::from_raw(0));
        },
    );
    assert_eq!(error, GameSetupError::MainTown(PlayerId::new(0)));
    // The last town is the setup's.
    let mut setup = towns_setup(&rules);
    setup.players[0].start.main_town.as_mut().unwrap().town = TownId::new(1);
    setup.map.objects[1].owner = Some(PlayerId::new(0));
    assert_eq!(setup.validate(&rules), Ok(()));
}

// covers: ADV-OWN-002
#[test]
fn the_towns_stand_on_the_map_with_their_owners_and_empty_garrisons() {
    // E09 T8a1 (design note §8): the static map's towns by `TownId`, owned as the setup says;
    // an owned town's default garrison is empty (TWN-SET-004, E09 T7b2a; [`build`]).
    let rules = towns_rules();
    let state = GameState::new(&rules, &towns_setup(&rules), 7).unwrap();
    let adventure = state.adventure();
    assert_eq!(adventure.map().towns(), [0, 1]);
    assert_eq!(adventure.map().town_id(1), Some(TownId::new(1)));
    assert_eq!(
        (adventure.owner(0), adventure.owner(1)),
        (Some(PlayerId::new(0)), Some(PlayerId::new(1)))
    );
    for index in [0, 1] {
        assert_eq!(adventure.town_garrison(index), Some(&[][..]));
    }
}

// covers: TWN-SET-001
#[test]
fn a_main_town_is_its_players_own() {
    // E09 T8a1 (design note §8, §13): blue's town, or a neutral one, is no main town of red's.
    let rules = towns_rules();
    let text = "player 0's main town is not a town of its own";
    let error = refused(&rules, text, |s| {
        s.players[0].start.main_town.as_mut().unwrap().town = TownId::new(1);
    });
    assert_eq!(error, GameSetupError::MainTown(PlayerId::new(0)));
    let error = refused(&rules, text, |s| s.map.objects[0].owner = None);
    assert_eq!(error, GameSetupError::MainTown(PlayerId::new(0)));
    // Blue's main town is its own.
    let mut setup = towns_setup(&rules);
    setup.players[1].start.main_town = Some(MainTown {
        town: TownId::new(1),
        generates_hero: false,
    });
    assert_eq!(setup.validate(&rules), Ok(()));
    let error = refused(
        &rules,
        "player 1's main town is not a town of its own",
        |s| {
            s.players[1].start.main_town = Some(MainTown {
                town: TownId::new(0),
                generates_hero: false,
            });
        },
    );
    assert_eq!(error, GameSetupError::MainTown(PlayerId::new(1)));
}

#[test]
fn the_setup_gives_one_town_record_per_town_of_the_map() {
    // E09 T8a1 (design note §13): the setup's towns against the objects of kind `Town`.
    let rules = towns_rules();
    let error = refused(&rules, "1 towns do not give one per town of the map", |s| {
        s.towns.pop();
    });
    assert_eq!(error, GameSetupError::TownCount(1));
    let error = refused(&rules, "3 towns do not give one per town of the map", |s| {
        s.towns.push(town(TownFaction::Random { alignment: None }));
    });
    assert_eq!(error, GameSetupError::TownCount(3));
    let error = refused(&rules, "2 towns do not give one per town of the map", |s| {
        s.map.objects.pop();
    });
    assert_eq!(error, GameSetupError::TownCount(2));
    // An object of an unknown type is no town: the count refuses before the adventure's check.
    let error = refused(&rules, "2 towns do not give one per town of the map", |s| {
        s.map.objects[1].object_type = Some(h3_adventure::ObjectTypeId::from_raw(u16::MAX));
    });
    assert_eq!(error, GameSetupError::TownCount(2));
    // Checked first, before a town's faction.
    let error = refused(&rules, "1 towns do not give one per town of the map", |s| {
        s.towns.pop();
        s.towns[0].faction = TownFaction::Fixed(faction(&rules, "t:other"));
    });
    assert_eq!(error, GameSetupError::TownCount(1));
}

// covers: TWN-SET-006
#[test]
fn the_offer_is_the_first_16_free_types_of_the_factions_classes_the_colour_may_take() {
    let rules = towns_rules();
    let red = PlayerId::new(0);
    let keys = |keys: &[&str]| keys.iter().map(|k| hero(&rules, k)).collect::<Vec<_>>();
    let knights = |range: core::ops::Range<usize>| {
        range
            .map(|i| hero(&rules, &format!("t:k{i}")))
            .collect::<Vec<_>>()
    };
    let mut setup = towns_setup(&rules);
    // Orrin is placed, the cleric of another faction: the four other knights and the first 12
    // added ones, 16 in type order.
    let mut first = keys(&["t:loner", "t:crowd", "t:gunner", "t:sage"]);
    first.extend(knights(0..12));
    assert_eq!(GameSetup::OFFERED_HEROES, 16);
    assert_eq!(setup.offer(&rules, red), first);
    // A prisoner's type is out, the next comes in; so is a type the map does not allow.
    setup.prisoners = keys(&["t:loner"]);
    setup
        .allowed_heroes
        .retain(|&t| t != hero(&rules, "t:crowd"));
    let mut next = keys(&["t:gunner", "t:sage"]);
    next.extend(knights(0..14));
    assert_eq!(setup.offer(&rules, red), next);
    // A type with a mask takes the colours whose bits every one of its masks has.
    let gunner = hero(&rules, "t:gunner");
    setup.hero_masks = vec![(gunner, 0b01)];
    assert_eq!(setup.offer(&rules, red), next);
    setup.hero_masks = vec![(gunner, 0b01), (gunner, 0b11), (gunner, 0b10)];
    assert_eq!(setup.offer(&rules, red)[0], hero(&rules, "t:sage"));
    // A colour past a mask's bits takes no masked type.
    setup.hero_masks = vec![(gunner, 0xFF)];
    setup.players[0].colour = 9;
    assert_eq!(setup.offer(&rules, red)[0], hero(&rules, "t:sage"));
    // A main town that generates a hero offers too, without the random-hero flag.
    let mut setup = towns_setup(&rules);
    setup.players[0].start.random_hero = false;
    setup.players[0]
        .start
        .main_town
        .as_mut()
        .unwrap()
        .generates_hero = true;
    assert_eq!(setup.offer(&rules, red), first);
    // Nothing for neither, for a random choice, or for a player not of the setup.
    setup.players[0].start.main_town = None;
    assert_eq!(setup.offer(&rules, red), []);
    let setup = towns_setup(&rules);
    assert_eq!(setup.offer(&rules, PlayerId::new(1)), []);
    assert_eq!(setup.offer(&rules, PlayerId::new(2)), []);
}

// covers: TWN-SET-006
#[test]
fn a_pick_is_one_of_the_offer() {
    let rules = towns_rules();
    let text = "player 0's pick is not offered";
    let pick = |k: &str| Some(hero(&rules, k));
    // The 16th is offered, the 17th not.
    let mut setup = towns_setup(&rules);
    setup.players[0].start.pick = pick("t:k11");
    assert_eq!(setup.validate(&rules), Ok(()));
    for k in ["t:k12", "t:orrin", "t:c0"] {
        let error = refused(&rules, text, |s| s.players[0].start.pick = pick(k));
        assert_eq!(error, GameSetupError::Pick(PlayerId::new(0)));
    }
    // A prisoner's type, a type the map does not allow and a type the colour may not take.
    let error = refused(&rules, text, |s| {
        s.players[0].start.pick = pick("t:loner");
        s.prisoners = vec![hero(&rules, "t:loner")];
    });
    assert_eq!(error, GameSetupError::Pick(PlayerId::new(0)));
    let error = refused(&rules, text, |s| {
        s.players[0].start.pick = pick("t:loner");
        s.allowed_heroes.clear();
    });
    assert_eq!(error, GameSetupError::Pick(PlayerId::new(0)));
    let error = refused(&rules, text, |s| {
        s.players[0].start.pick = pick("t:loner");
        s.hero_masks = vec![(hero(&rules, "t:loner"), 0b10)];
    });
    assert_eq!(error, GameSetupError::Pick(PlayerId::new(0)));
    // An empty offer: a random choice, or a start without a hero to give it to.
    let error = refused(&rules, "player 1's pick is not offered", |s| {
        s.players[1].start.pick = pick("t:loner");
    });
    assert_eq!(error, GameSetupError::Pick(PlayerId::new(1)));
    let error = refused(&rules, text, |s| {
        s.players[0].start.random_hero = false;
        s.players[0].start.pick = pick("t:loner");
    });
    assert_eq!(error, GameSetupError::Pick(PlayerId::new(0)));
}

// covers: TWN-SET-006
#[test]
fn two_players_pick_two_types() {
    let rules = towns_rules();
    let castle = faction(&rules, "t:town");
    let loner = Some(hero(&rules, "t:loner"));
    let seat = |s: &mut GameSetup| {
        s.players[1].start = start(castle, false);
        s.players[1].start.main_town.as_mut().unwrap().town = TownId::new(1);
        s.towns[1].faction = TownFaction::Fixed(castle);
        s.players[0].start.pick = loner;
    };
    let error = refused(&rules, "player 1's pick is an earlier player's", |s| {
        seat(s);
        s.players[1].start.pick = loner;
    });
    assert_eq!(error, GameSetupError::PickTaken(PlayerId::new(1)));
    let mut setup = towns_setup(&rules);
    seat(&mut setup);
    setup.players[1].start.pick = Some(hero(&rules, "t:crowd"));
    assert_eq!(setup.validate(&rules), Ok(()));
    // Every pick is checked against its offer first: a third player's pick outside its offer
    // is named before blue's taken pick.
    let error = refused(&rules, "player 2's pick is not offered", |s| {
        seat(s);
        s.players[1].start.pick = loner;
        s.players.push(PlayerSetup {
            colour: 2,
            human: false,
            start: PlayerStart {
                pick: Some(hero(&rules, "t:c0")),
                ..PlayerStart::default()
            },
        });
    });
    assert_eq!(error, GameSetupError::Pick(PlayerId::new(2)));
}

// covers: TWN-SET-008
#[test]
fn a_random_heros_owner_is_a_player() {
    let rules = towns_rules();
    let error = refused(&rules, "random hero 1's owner is no player", |s| {
        s.heroes[1].hero_type = None;
        s.heroes[1].owner = PlayerId::new(2);
    });
    assert_eq!(error, GameSetupError::RandomHeroOwner(HeroId::new(1)));
    // Checked after the picks.
    let error = refused(&rules, "player 1's pick is an earlier player's", |s| {
        s.heroes[1].hero_type = None;
        s.heroes[1].owner = PlayerId::new(2);
        s.players[1].start = start(faction(&rules, "t:town"), false);
        s.players[1].start.main_town = s.players[1].start.main_town.map(|main| MainTown {
            town: TownId::new(1),
            ..main
        });
        s.players[1].start.pick = Some(hero(&rules, "t:loner"));
        s.players[0].start.pick = Some(hero(&rules, "t:loner"));
    });
    assert_eq!(error, GameSetupError::PickTaken(PlayerId::new(1)));
    // A placed hero's owner is the adventure context's check.
    let error = refused(&rules, "adventure: hero 1's owner is not a player", |s| {
        s.heroes[1].owner = PlayerId::new(2);
    });
    assert!(matches!(error, GameSetupError::Adventure(_)));
}

#[test]
fn a_placed_heros_unknown_type_is_refused_before_any_draw() {
    let rules = towns_rules();
    let refused = |setup: &GameSetup| setup.validate(&rules).unwrap_err();
    // An unknown type is refused before any draw: a random hero and a main town's hero that
    // would find no free type are not reached.
    let mut unknown = towns_setup(&rules);
    unknown.players[0].start = start(faction(&rules, "t:town"), true);
    unknown.heroes[0].hero_type = Some(HeroTypeId::from_raw(u16::MAX));
    unknown.heroes[1].hero_type = None;
    unknown.allowed_heroes.clear();
    assert_eq!(
        refused(&unknown),
        GameSetupError::UnknownHeroType(HeroId::new(0))
    );
    // Without it, blue's random hero draws first (step 2), before red's main town's (step 3).
    unknown.heroes[0].hero_type = Some(HeroTypeId::from_raw(0));
    assert_eq!(
        refused(&unknown),
        GameSetupError::NoFreeHero(PlayerId::new(1))
    );
}

mod build;
mod capture;
mod heroes;
mod pool;
