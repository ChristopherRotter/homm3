//! Tests of the towns at the build (E09 T7b2a, design note `E09-design.md` §14, §5): step 2's
//! random towns (TWN-SET-002), step 4's garrisons and buildings (TWN-SET-004, TWN-SET-003), the
//! store of towns and the start's reveal (TWN-CAP-001).

use alloc::vec;
use alloc::vec::Vec;

use h3_adventure::Pos;
use h3_content::{FactionId, TerrainId};
use h3_contracts::{ArmyStack, PlayerId, TownId};
use h3_kernel::{Rng, StreamId};
use h3_towns::{
    BuildingId, BuildingsSetup, GarrisonCreature, GarrisonStack, SlotSet, TownFaction, town_setup,
    without_water,
};

use super::{
    GRAIL_SLOT, LOOKOUT_RADIUS, LOOKOUT_SLOT, SHIPYARD, SHIPYARD_SLOT, faction, town_object,
    towns_rules, towns_rules_on, towns_rules_with, towns_setup,
};
use crate::town::{FactionChoice, FactionSet, PlayerStart};
use crate::{GameRuleset, GameSetup, GameSetupError, GameState};

const RED: PlayerId = PlayerId::new(0);
const BLUE: PlayerId = PlayerId::new(1);
const T0: TownId = TownId::new(0);
const T1: TownId = TownId::new(1);

/// The faction of town `town` in the game `setup` builds with `seed`.
fn town_faction(rules: &GameRuleset, setup: &GameSetup, seed: u64, town: TownId) -> FactionId {
    let game = GameState::new(rules, setup, seed).unwrap();
    game.towns().get(town).unwrap().faction()
}

/// [`towns_setup`] whose blue chooses `choice` among the town's and the other faction, red's
/// main town and choice as there.
fn choosing(rules: &GameRuleset, choice: FactionChoice) -> GameSetup {
    let mut setup = towns_setup(rules);
    let (castle, other) = (faction(rules, "t:town"), faction(rules, "t:other"));
    setup.players[1].start.factions = FactionSet::EMPTY.with(castle).with(other);
    setup.players[1].start.choice = Some(choice);
    setup
}

// covers: TWN-SET-002
#[test]
fn a_random_town_aligned_to_a_playing_colour_takes_its_faction() {
    let rules = towns_rules_with(true);
    let (castle, other) = (faction(&rules, "t:town"), faction(&rules, "t:other"));
    let mut setup = choosing(&rules, FactionChoice::Fixed(other));
    // Blue's town, aligned to blue.
    assert_eq!(town_faction(&rules, &setup, 3, T1), other);
    // Aligned to red, it takes red's faction, not its owner's.
    setup.towns[1].faction = TownFaction::Random { alignment: Some(0) };
    assert_eq!(town_faction(&rules, &setup, 3, T1), castle);
    // Neutral and aligned to blue.
    setup.towns[1].faction = TownFaction::Random { alignment: Some(1) };
    setup.map.objects[1].owner = None;
    assert_eq!(town_faction(&rules, &setup, 3, T1), other);
    // A fixed town keeps its faction.
    assert_eq!(town_faction(&rules, &setup, 3, T0), castle);
}

// covers: TWN-SET-002, TWN-SET-001
#[test]
fn an_owned_random_town_takes_its_owners_faction_drawn_or_fixed() {
    let rules = towns_rules_with(true);
    let (castle, other) = (faction(&rules, "t:town"), faction(&rules, "t:other"));
    let mut setup = choosing(&rules, FactionChoice::Fixed(other));
    setup.towns[1].faction = TownFaction::Random { alignment: None };
    assert_eq!(town_faction(&rules, &setup, 3, T1), other);
    // A random choice that only the town reads is drawn now (no main town, random-hero flag or
    // random hero of blue's): both factions come up over the seeds.
    setup.players[1].start.choice = Some(FactionChoice::Random);
    let factions: Vec<FactionId> = (0..24)
        .map(|seed| town_faction(&rules, &setup, seed, T1))
        .collect();
    assert!(factions.contains(&castle) && factions.contains(&other));
    // The choice is the stream's first draw (red's is fixed, no hero is random), over blue's two
    // factions; the town draws nothing more.
    for seed in 0..8 {
        let mut replay = Rng::for_stream(seed, StreamId::SETUP);
        let chosen = [castle, other][usize::try_from(replay.below(2)).unwrap()];
        assert_eq!(town_faction(&rules, &setup, seed, T1), chosen);
    }
}

// covers: TWN-SET-002
#[test]
fn a_neutral_random_town_draws_among_the_factions_with_a_town_record() {
    // Without the elf the other faction has no town record: never drawn.
    let rules = towns_rules();
    let mut setup = towns_setup(&rules);
    setup.towns[1].faction = TownFaction::Random { alignment: None };
    setup.map.objects[1].owner = None;
    setup.players[1].start = PlayerStart::default();
    let castle = faction(&rules, "t:town");
    for seed in 0..16 {
        assert_eq!(town_faction(&rules, &setup, seed, T1), castle);
    }
    // With it both have one: the town's draw is the stream's first (no choice drawn, no hero
    // random, town 0 fixed), uniform over the two in faction order.
    let rules = towns_rules_with(true);
    let other = faction(&rules, "t:other");
    let factions: Vec<FactionId> = (0..24)
        .map(|seed| town_faction(&rules, &setup, seed, T1))
        .collect();
    assert!(factions.contains(&castle) && factions.contains(&other));
    for seed in 0..8 {
        let mut replay = Rng::for_stream(seed, StreamId::SETUP);
        let drawn = [castle, other][usize::try_from(replay.below(2)).unwrap()];
        assert_eq!(town_faction(&rules, &setup, seed, T1), drawn);
    }
}

// covers: TWN-SET-002
#[test]
fn a_random_town_needs_a_faction_with_a_town_record() {
    // The game's rules without town records: a random town, aligned or not, is refused first.
    let rules = crate::tests::rules();
    for alignment in [None, Some(0), Some(7)] {
        let mut setup = crate::tests::setup();
        setup.towns = vec![super::town(TownFaction::Random { alignment })];
        setup.map.objects = vec![town_object(&rules, (1, 1), Some(0))];
        let error = setup.validate(&rules).unwrap_err();
        assert_eq!(error, GameSetupError::TownFaction(T0));
    }
}

// covers: TWN-SET-004
#[test]
fn the_garrisons_land_in_the_adventure() {
    let rules = towns_rules();
    let [imp, gog] = ["t:imp", "t:gog"].map(|k| rules.core().creature_by_key(k).unwrap());
    let mut setup = towns_setup(&rules);
    // Red's town neutral (red without a main town): its default garrison drawn, each of the
    // record's two levels at the chance 100, one creature each, in the first free slots.
    setup.map.objects[0].owner = None;
    setup.players[0].start.main_town = None;
    // Blue's a custom one: a stack of 0 leaves its slot empty, a random level-1 stack upgraded
    // is the level's base without an upgrade.
    let stack = |slot, creature, count| GarrisonStack {
        slot,
        creature,
        count,
    };
    let custom = vec![
        stack(0, GarrisonCreature::Creature(gog), 0),
        stack(
            1,
            GarrisonCreature::Random {
                level: 1,
                upgraded: true,
            },
            7,
        ),
    ];
    setup.towns[1].garrison = Some(custom);
    let game = GameState::new(&rules, &setup, 5).unwrap();
    let army = |slot, creature, count| ArmyStack {
        slot,
        creature,
        count,
    };
    let adventure = game.adventure();
    assert_eq!(
        adventure.town_garrison(0),
        Some(&[army(0, imp, 1), army(1, gog, 1)][..])
    );
    assert_eq!(adventure.town_garrison(1), Some(&[army(1, imp, 7)][..]));
    // An owned town's default garrison is empty; a custom empty one too.
    setup.towns[1].garrison = None;
    setup.towns[0].garrison = Some(Vec::new());
    let game = GameState::new(&rules, &setup, 5).unwrap();
    for index in [0, 1] {
        assert_eq!(game.adventure().town_garrison(index), Some(&[][..]));
    }
}

// covers: TWN-SET-003, TWN-SET-004
#[test]
fn the_store_holds_each_town_as_built_before_the_starting_armies() {
    let rules = towns_rules();
    let castle = faction(&rules, "t:town");
    let mut setup = towns_setup(&rules);
    setup.players[1].start.choice = Some(FactionChoice::Fixed(castle));
    // Town 1 neutral, so that it draws its garrison too; blue's hero takes its starting army,
    // drawn after the towns.
    setup.map.objects[1].owner = None;
    let record = rules.towns().town(castle).unwrap();
    let [hall, fort, dwelling] = ["b0", "b4", "b7"].map(|k| record.building_by_key(k).unwrap());
    for seed in 0..16 {
        let game = GameState::new(&rules, &setup, seed).unwrap();
        // Step 4's draws are the stream's first (no choice drawn, no hero random), town 0's then
        // town 1's: the level-2 dwelling's chance of 50 tells the order apart.
        let mut replay = Rng::for_stream(seed, StreamId::SETUP);
        let towns = game.towns();
        assert_eq!(towns.len(), 2);
        for (index, owned) in [(0, true), (1, false)] {
            let built = town_setup(
                &setup.towns[index],
                castle,
                owned,
                rules.towns(),
                rules.core(),
                &mut replay,
            )
            .unwrap();
            let id = TownId::new(u16::try_from(index).unwrap());
            // No water on the map: each Shipyard forbidden (TWN-SET-011, E09 T18a).
            let dry = without_water(&built.town, rules.towns());
            assert_eq!(towns.get(id), Some(&dry));
            assert_eq!(towns.town_static(id), Some(&built.town_static));
            assert_eq!(
                game.adventure().town_garrison(index),
                Some(&built.garrison[..])
            );
        }
        let town = towns.get(T0).unwrap();
        assert_eq!(town.faction(), castle);
        assert!(
            [hall, fort, dwelling]
                .iter()
                .all(|&b| town.built().contains(b))
        );
    }
    // A game without towns has an empty store.
    let rules = crate::tests::rules();
    let game = GameState::new(&rules, &crate::tests::setup(), 7).unwrap();
    assert!(game.towns().is_empty());
}

// covers: TWN-SET-003
#[test]
fn a_fork_shares_the_store_of_towns() {
    let rules = towns_rules();
    let game = GameState::new(&rules, &towns_setup(&rules), 7).unwrap();
    let fork = game.clone();
    assert_eq!(fork.towns(), game.towns());
    for id in [T0, T1] {
        assert!(core::ptr::eq(
            game.towns().get(id).unwrap(),
            fork.towns().get(id).unwrap()
        ));
        assert!(core::ptr::eq(
            game.towns().town_static(id).unwrap(),
            fork.towns().town_static(id).unwrap()
        ));
    }
    assert_eq!(fork.state_hash(), game.state_hash());
}

/// [`towns_setup`] on a 48 × 48 map, red's town 0 at (20, 20) with the custom set of `built`,
/// blue's choice fixed.
fn far_town(rules: &GameRuleset, built: SlotSet) -> GameSetup {
    let mut setup = towns_setup(rules);
    let tile = setup.map.tiles[0];
    setup.map.width = 48;
    setup.map.height = 48;
    setup.map.tiles = vec![tile; 48 * 48];
    setup.map.objects[0].anchor = Pos::new(20, 20, 0);
    setup.players[1].start.choice = Some(FactionChoice::Fixed(faction(rules, "t:town")));
    setup.towns[0].buildings = BuildingsSetup::Custom {
        built,
        forbidden: SlotSet::EMPTY,
    };
    setup
}

// covers: TWN-CAP-001
#[test]
fn the_start_reveals_each_owned_towns_radius() {
    let rules = towns_rules_on(false, 48);
    let explored = |setup: &GameSetup, (x, y)| {
        let game = GameState::new(&rules, setup, 7).unwrap();
        game.adventure().is_explored(RED, Pos::new(x, y, 0))
    };
    // The part's radius 5 around the entrance (the tiles within `r² + r`), far from the heroes.
    let plain = far_town(&rules, SlotSet::EMPTY);
    assert!(explored(&plain, (25, 20)) && explored(&plain, (24, 23)));
    assert!(!explored(&plain, (26, 20)) && !explored(&plain, (25, 23)));
    // A lookout ever built: its radius 20.
    let lookout = far_town(&rules, SlotSet::EMPTY.with(LOOKOUT_SLOT));
    let r = u16::from(LOOKOUT_RADIUS);
    assert!(explored(&lookout, (20 + r, 20)) && explored(&lookout, (34, 34)));
    assert!(!explored(&lookout, (21 + r, 20)) && !explored(&lookout, (35, 34)));
    // The Grail's every tile of every level (E09 T8a2): the corners of both levels.
    let mut grail = far_town(&rules, SlotSet::EMPTY.with(GRAIL_SLOT));
    grail.map.levels = 2;
    grail.map.tiles = [grail.map.tiles.clone(), grail.map.tiles.clone()].concat();
    let game = GameState::new(&rules, &grail, 7).unwrap();
    for level in 0..2 {
        for (x, y) in [(0, 0), (0, 47), (47, 0), (47, 47)] {
            assert!(game.adventure().is_explored(RED, Pos::new(x, y, level)));
        }
    }
    // A neutral town reveals nothing: not to red, nor to blue.
    let mut neutral = far_town(&rules, SlotSet::EMPTY.with(GRAIL_SLOT));
    neutral.map.objects[0].owner = None;
    neutral.players[0].start.main_town = None;
    let game = GameState::new(&rules, &neutral, 7).unwrap();
    for player in [RED, BLUE] {
        assert!(!game.adventure().is_explored(player, Pos::new(47, 47, 0)));
    }
}

// covers: TWN-SET-011
#[test]
fn a_town_without_water_loses_its_shipyard_at_the_start() {
    let rules = towns_rules_on(false, 48);
    let shipyard = BuildingId::from_index(SHIPYARD).unwrap();
    // Red's town at (20, 20) with a custom set's Shipyard; blue's standard town at (3, 0).
    let land = far_town(&rules, SlotSet::EMPTY.with(SHIPYARD_SLOT));
    let wet = |water: &[(usize, usize)]| {
        let mut setup = land.clone();
        for &(x, y) in water {
            setup.map.tiles[y * 48 + x].terrain = TerrainId::from_raw(2);
        }
        setup
    };
    let sets = |setup: &GameSetup| {
        let game = GameState::new(&rules, setup, 7).unwrap();
        let sets = |town| {
            let town = game.towns().get(town).unwrap();
            (
                town.built().contains(shipyard),
                town.forbidden().contains(shipyard),
            )
        };
        (sets(T0), sets(T1), game)
    };
    // No water by either: red's built Shipyard removed and forbidden, blue's never built
    // forbidden.
    let (red, blue, dry) = sets(&land);
    assert_eq!((red, blue), ((false, true), (false, true)));
    // Water two rows below red's entrance, one column left or right: red's Shipyard kept, its
    // spot the first offset's; blue's town, on land, still without.
    for (water, spot) in [
        (vec![(19, 22), (21, 22)], (19, 22)),
        (vec![(21, 22)], (21, 22)),
    ] {
        let (red, blue, game) = sets(&wet(&water));
        assert_eq!((red, blue), ((true, false), (false, true)));
        let spot = Pos::new(spot.0, spot.1, 0);
        assert_eq!(game.adventure().map().town_boat_spot(T0), Some(spot));
        // Blue's town as without water (the spots draw nothing).
        assert_eq!(game.towns().get(T1), dry.towns().get(T1));
    }
    // Water elsewhere by the town (beside it, one row below) gives no spot.
    let (red, _, _) = sets(&wet(&[(19, 21), (20, 22), (22, 22)]));
    assert_eq!(red, (false, true));
}
