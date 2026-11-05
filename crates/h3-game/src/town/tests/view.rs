//! Tests of what a viewer sees of the towns (E09 T8c1, the town epic's design note §12, §17):
//! the towns it sees a tile of, linked from their objects, with their public facts (TWN-VEW-001),
//! the garrison by the viewer's level (TWN-VEW-002: its towns with a Tavern, at most two) and the
//! owner's team's facts (TWN-VEW-003).

use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use h3_adventure::{Pos, TileSetup};
use h3_content::{CreatureTypeId, ResourceId, TerrainId};
use h3_contracts::{ArmyStack, FortLevel, PlayerId, TownId};
use h3_towns::{
    BuildingDef, BuildingRoles, BuildingsSetup, GarrisonCreature, GarrisonStack, SlotSet,
    TownFaction,
};

use super::{
    CAPITOL_SLOT, SILO_SLOT, faction, rules_of, town_object, town_pack, towns_core, towns_rules,
    towns_rules_on, towns_setup,
};
use crate::{
    GameRuleset, GameSetup, GameState, GarrisonStackView, GarrisonView, OWNER_LEVEL, TownOwnerView,
    TownView, Viewer, observe,
};

const RED: PlayerId = PlayerId::new(0);
const BLUE: PlayerId = PlayerId::new(1);
const T0: TownId = TownId::new(0);
const T1: TownId = TownId::new(1);
const IMP: CreatureTypeId = CreatureTypeId::from_raw(0);
const GOG: CreatureTypeId = CreatureTypeId::from_raw(1);
const GOLD: ResourceId = ResourceId::from_raw(0);
/// The map slot of [`tavern_rules`]' Tavern (the h3m's).
const TAVERN_SLOT: u8 = 5;

/// [`towns_rules`] whose town record has a Tavern, which the standard set builds in an owned
/// town (TWN-SET-003).
fn tavern_rules() -> GameRuleset {
    let mut towns = town_pack(false);
    towns.towns[0].buildings.push(BuildingDef {
        key: "tavern".to_string(),
        name: "Tavern".into(),
        slot: Some(TAVERN_SLOT),
        supersedes: vec![],
        forbidden_with: None,
        auto_with: vec![],
        roles: BuildingRoles {
            tavern: true,
            ..BuildingRoles::default()
        },
    });
    rules_of(towns_core(false), &towns, false, 4)
}

/// [`towns_setup`] with blue's town holding 12 imps in slot 0 and 4 gogs in slot 1 (the test
/// part's two).
fn garrisoned(rules: &GameRuleset) -> GameSetup {
    let mut setup = towns_setup(rules);
    setup.towns[1].garrison = Some(
        [(0, IMP, 12), (1, GOG, 4)]
            .map(|(slot, creature, count)| GarrisonStack {
                slot,
                creature: GarrisonCreature::Creature(creature),
                count,
            })
            .to_vec(),
    );
    setup
}

/// A town of `t:town` with the standard set owned by `owner` at `(x, y)`, added to `setup`.
fn add_town(rules: &GameRuleset, setup: &mut GameSetup, (x, y): (u16, u16), owner: u8) {
    setup
        .map
        .objects
        .push(town_object(rules, (x, y), Some(owner)));
    setup
        .towns
        .push(super::town(TownFaction::Fixed(faction(rules, "t:town"))));
}

/// What `viewer` sees of the towns of the game `setup` builds.
fn towns_seen(rules: &GameRuleset, setup: &GameSetup, viewer: PlayerId) -> Vec<TownView> {
    let game = GameState::new(rules, setup, 7).unwrap();
    observe(&game, rules, Viewer::Player(viewer)).towns
}

/// Blue's garrison as its stacks show at `level` to a viewer of another team.
fn shown(level: u8) -> GarrisonView {
    let stack = |creature, word| GarrisonStackView {
        creature,
        count_word: (level >= 2).then_some(word),
        count: None,
    };
    GarrisonView {
        level,
        stacks: if level == 0 {
            vec![]
        } else {
            vec![stack(IMP, 2), stack(GOG, 0)]
        },
        army: None,
    }
}

// covers: TWN-VEW-001, TWN-VEW-003
#[test]
fn a_viewer_sees_each_town_with_its_public_facts_and_its_own_with_the_owners() {
    let rules = towns_rules();
    let mut setup = garrisoned(&rules);
    setup.towns[0].name = Some("Steadwick".into());
    setup.towns[0].buildings = BuildingsSetup::Custom {
        built: SlotSet::EMPTY.with(CAPITOL_SLOT).with(SILO_SLOT),
        forbidden: SlotSet::EMPTY,
    };
    let castle = faction(&rules, "t:town");
    let game = GameState::new(&rules, &setup, 7).unwrap();
    let red = observe(&game, &rules, Viewer::Player(RED));
    // The 4 × 4 map is all explored: both towns, linked from their objects, in `TownId` order.
    let linked: Vec<_> = red.adventure.objects.iter().map(|o| o.town).collect();
    assert_eq!(linked, [Some(T0), Some(T1)]);
    // Its own: the Capitol without a fort, every fact of the owner's team.
    assert_eq!(
        red.towns[0],
        TownView {
            town: T0,
            anchor: Pos::new(1, 1, 0),
            owner: Some(RED),
            faction: castle,
            name: Some("Steadwick".into()),
            fort: FortLevel::None,
            capitol: true,
            garrison: GarrisonView {
                level: OWNER_LEVEL,
                stacks: vec![],
                army: Some(vec![]),
            },
            owner_facts: Some(TownOwnerView {
                hall: 4,
                income: vec![(GOLD, 4001)],
            }),
        }
    );
    // Blue's: the standard set with its Fort, unnamed; red has no Tavern, so no garrison.
    assert_eq!(
        red.towns[1],
        TownView {
            town: T1,
            anchor: Pos::new(3, 0, 0),
            owner: Some(BLUE),
            faction: castle,
            name: None,
            fort: FortLevel::Fort,
            capitol: false,
            garrison: shown(0),
            owner_facts: None,
        }
    );
    // Blue sees its own garrison by slot with its counts and words, and red's town's public
    // facts.
    let blue = observe(&game, &rules, Viewer::Player(BLUE));
    let stack = |creature, count_word, count| GarrisonStackView {
        creature,
        count_word: Some(count_word),
        count: Some(count),
    };
    assert_eq!(
        blue.towns[1].garrison,
        GarrisonView {
            level: OWNER_LEVEL,
            stacks: vec![stack(IMP, 2, 12), stack(GOG, 0, 4)],
            army: Some(vec![
                ArmyStack {
                    slot: 0,
                    creature: IMP,
                    count: 12,
                },
                ArmyStack {
                    slot: 1,
                    creature: GOG,
                    count: 4,
                },
            ]),
        }
    );
    assert_eq!(
        blue.towns[1].owner_facts,
        Some(TownOwnerView {
            hall: 1,
            income: vec![(GOLD, 500)],
        })
    );
    let public = TownView {
        garrison: GarrisonView {
            level: 0,
            stacks: vec![],
            army: None,
        },
        owner_facts: None,
        ..red.towns[0].clone()
    };
    assert_eq!(blue.towns[0], public);
}

// covers: TWN-VEW-002
#[test]
fn the_garrisons_level_is_the_viewers_towns_with_a_tavern_at_most_two() {
    let rules = tavern_rules();
    // One red town with a Tavern: the creatures; two: their words too; three: still two.
    let mut setup = garrisoned(&rules);
    assert_eq!(towns_seen(&rules, &setup, RED)[1].garrison, shown(1));
    add_town(&rules, &mut setup, (1, 3), 0);
    assert_eq!(towns_seen(&rules, &setup, RED)[1].garrison, shown(2));
    add_town(&rules, &mut setup, (0, 3), 0);
    assert_eq!(towns_seen(&rules, &setup, RED)[1].garrison, shown(2));
    // A custom set without a Tavern counts for nothing; another player's Tavern neither.
    let mut setup = garrisoned(&rules);
    setup.towns[0].buildings = BuildingsSetup::Custom {
        built: SlotSet::EMPTY,
        forbidden: SlotSet::EMPTY,
    };
    add_town(&rules, &mut setup, (1, 3), 1);
    assert_eq!(towns_seen(&rules, &setup, RED)[1].garrison, shown(0));
    // A Tavern of a town red does not own: blue's view of red's empty garrison is by blue's two.
    let blue = &towns_seen(&rules, &setup, BLUE)[0].garrison;
    assert_eq!(
        (blue.level, blue.stacks.len(), blue.army.as_ref()),
        (2, 0, None)
    );
    // The stacks are packed: the gogs alone in slot 1 are the only stack, their slot unsaid.
    let mut setup = garrisoned(&rules);
    setup.towns[1].garrison.as_mut().unwrap().remove(0);
    let red = towns_seen(&rules, &setup, RED);
    let gogs = GarrisonStackView {
        creature: GOG,
        count_word: None,
        count: None,
    };
    assert_eq!(red[1].garrison.stacks, [gogs]);
}

// covers: TWN-VEW-002, TWN-VEW-003
#[test]
fn a_neutral_town_is_nobodys_team() {
    let rules = tavern_rules();
    let mut setup = garrisoned(&rules);
    setup.map.objects[1].owner = None;
    setup.towns[1] = super::town(TownFaction::Fixed(faction(&rules, "t:town")));
    setup.towns[1].garrison = garrisoned(&rules).towns[1].garrison.clone();
    for viewer in [RED, BLUE] {
        let town = &towns_seen(&rules, &setup, viewer)[1];
        assert_eq!(town.owner, None);
        assert_eq!(town.owner_facts, None);
        // Red's Tavern shows it the creatures; blue owns no town.
        assert_eq!(town.garrison, shown(if viewer == RED { 1 } else { 0 }));
    }
}

// covers: TWN-VEW-001
#[test]
fn a_town_no_tile_of_which_the_viewer_sees_is_not_shown() {
    let rules = towns_rules_on(false, 16);
    let mut setup = towns_setup(&rules);
    setup.map.width = 16;
    setup.map.height = 16;
    setup.map.tiles = vec![
        TileSetup {
            terrain: TerrainId::from_raw(0),
            road: None,
            river: None,
            coast: false,
        };
        256
    ];
    setup.map.objects[1].anchor = Pos::new(14, 14, 0);
    let game = GameState::new(&rules, &setup, 7).unwrap();
    let red = observe(&game, &rules, Viewer::Player(RED));
    let blue = observe(&game, &rules, Viewer::Player(BLUE));
    // Red's town's reveal and its hero's do not reach blue's town; blue's hero's reaches red's.
    assert!(!red.adventure.is_explored(Pos::new(14, 14, 0)));
    let towns = |towns: &[TownView]| towns.iter().map(|t| t.town).collect::<Vec<_>>();
    assert_eq!(towns(&red.towns), [T0]);
    assert_eq!(towns(&blue.towns), [T0, T1]);
    let linked: Vec<_> = red.adventure.objects.iter().map(|o| o.town).collect();
    assert_eq!(linked, [Some(T0)]);
}
