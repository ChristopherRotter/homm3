//! Tests of the towns' growth in the game's day (E09 T26a1, the town epic's design note §41 to
//! §43, §46, §47): each town's inputs from the owners, their dwellings on the map and the heroes'
//! Legion (TWN-GRO-001), the week's growth on its first day (TWN-GRO-003), the month's (TWN-GRO-004)
//! and a town event's creatures (TWN-EVT-004).

use alloc::vec;
use alloc::vec::Vec;
use core::num::{NonZeroU8, NonZeroU16};

use h3_adventure::{
    DwellingParams, Holds, MonthParamsDef, ObjectKindDef, ObjectSetup, ObjectTypeDef, Pos, Scenery,
    TileMask, Visit,
};
use h3_content::{ArtifactClass, ArtifactDef, HeroEffect};
use h3_contracts::{PlayerId, TownId};
use h3_heroes::{ArtifactInstance, wearing};
use h3_kernel::{Rng, StreamId};
use h3_towns::{
    BuildingId, BuildingsSetup, GrowthInputs, SlotSet, TownEventSetup, TownFaction, TownPack,
    TownSetup,
};

use super::command::{BLUE, RED, end_turn};
use super::{faction, rules_with_adventure, town_object, town_pack, towns_core_pack, towns_setup};
use crate::tests::key;
use crate::{
    GameEvent, GameRuleset, GameSetup, GameState, HeroArtifactsSetup, MonthKind, Viewer, WeekKind,
    apply, observe,
};

pub(super) const T0: TownId = TownId::new(0);
pub(super) const T1: TownId = TownId::new(1);
pub(super) const T2: TownId = TownId::new(2);
pub(super) const T3: TownId = TownId::new(3);
/// The part's level 1 and level 2 dwellings (the imps' growth 15, the gogs' 8).
pub(super) const DWELLING_1: usize = 7;
const DWELLING_2: usize = 8;

/// The towns' rules with the Legs of Legion (`t:legs`, +5 at level 1), the Statue (`t:statue`,
/// the owner's share 5 000), a dwelling `t:hut` of the imps and the gogs, and the month's odds
/// `odds` (the gogs the months' creature).
fn rules(odds: [u8; 3]) -> GameRuleset {
    rules_on(odds, |_| {})
}

/// [`rules`] with the town part edited by `edit` (E09 T26a2: the week's chance, a `week_of`
/// building).
pub(super) fn rules_on(odds: [u8; 3], edit: impl FnOnce(&mut TownPack)) -> GameRuleset {
    let mut towns = town_pack(false);
    // E09 T26b: the neutral T3's garrison never grows here (its week draws its odds alone; the
    // garrison week is `garrison.rs`' tests').
    towns.params.as_mut().unwrap().garrison_week.odds = [0, 0];
    edit(&mut towns);
    let mut core = towns_core_pack(false);
    for (k, effect) in [
        (
            "t:legs",
            HeroEffect::TownGrowth {
                level: 1,
                amount: 5,
            },
        ),
        ("t:statue", HeroEffect::OwnerTownGrowth { bp: 5_000 }),
    ] {
        core.artifacts.push(ArtifactDef {
            key: key(k),
            name: "Artifact".into(),
            class: ArtifactClass::Treasure,
            cost: vec![],
            slot: Some(key("t:ring_slot")),
            war_machine: None,
            needs_skill: None,
            effects: vec![effect],
            parts: vec![],
        });
    }
    // A month's creature has an adventure count (its monsters' draw); since E09 T26a2 the imps
    // too (a building's week's month).
    core.creatures[0].info.adventure_count = [2, 3];
    core.creatures[1].info.adventure_count = [2, 3];
    rules_with_adventure(core, &towns, |part| {
        part.object_types.push(ObjectTypeDef {
            key: key("t:hut"),
            name: "Hut".into(),
            visit: Visit::Enter,
            from_above: false,
            kind_mark: false,
            kind: ObjectKindDef::Dwelling {
                creatures: vec![key("t:imp"), key("t:gog")],
                blocked: vec![(0, 0)],
                visitable: vec![(0, 0)],
            },
        });
        let params = part.params.as_mut().unwrap();
        params.dwellings = Some(DwellingParams {
            guard_level: 7,
            guard_factor: NonZeroU16::new(3).unwrap(),
            free_level: 1,
        });
        params.months = Some(MonthParamsDef {
            odds,
            creatures: vec![key("t:gog")],
            monster: key("t:monster"),
            spawn_one_in: NonZeroU16::new(1).unwrap(),
            spawn_factor: NonZeroU8::new(2).unwrap(),
            spawn_aggression: [10, 10],
        });
    })
}

/// A hut on `(x, y)` owned by `owner`.
fn hut(rules: &GameRuleset, (x, y): (u16, u16), owner: Option<PlayerId>) -> ObjectSetup {
    ObjectSetup {
        anchor: Pos::new(x, y, 0),
        blocked: TileMask::from_bits(1),
        visitable: TileMask::from_bits(1),
        object_type: rules.adventure().object_type_by_key("t:hut"),
        amount: None,
        drawn: TileMask::default(),
        monster: None,
        guards: None,
        scenery: Scenery::default(),
        holds: Holds::Nothing,
        owner,
    }
}

/// The towns' setup with four towns of `t:town` and only their Village Halls: red's T0 on (1, 1),
/// its visitor red's hero wearing the Legs and the Statue; blue's T1 on (3, 0); red's T2 on
/// (0, 3); the neutral T3 on (2, 0). A hut of red's on (3, 2) and two neutral ones on (0, 2) and
/// (2, 3).
pub(super) fn setup(rules: &GameRuleset) -> GameSetup {
    let castle = faction(rules, "t:town");
    let mut setup = towns_setup(rules);
    let bare = TownSetup {
        faction: TownFaction::Fixed(castle),
        buildings: BuildingsSetup::Custom {
            built: SlotSet::EMPTY,
            forbidden: SlotSet::EMPTY,
        },
        garrison: None,
        name: None,
        events: vec![],
    };
    setup.towns = vec![bare; 4];
    setup.map.objects = vec![
        town_object(rules, (1, 1), Some(0)),
        town_object(rules, (3, 0), Some(1)),
        town_object(rules, (0, 3), Some(0)),
        town_object(rules, (2, 0), None),
        hut(rules, (3, 2), Some(RED)),
        hut(rules, (0, 2), None),
        hut(rules, (2, 3), None),
    ];
    let core = rules.core();
    let worn: Vec<ArtifactInstance> = ["t:legs", "t:statue"]
        .map(|k| core.artifact_by_key(k).unwrap().into())
        .to_vec();
    let red = &mut setup.heroes[0];
    red.pos = Pos::new(1, 1, 0);
    red.details.artifacts = Some(HeroArtifactsSetup {
        worn: wearing(core, &worn),
        backpack: vec![],
    });
    setup
}

/// The game of [`setup`] with the level 1 dwelling built in every town and the level 2 one in
/// T0 (the build's first creatures: 15 imps, 8 gogs).
pub(super) fn built(rules: &GameRuleset, setup: &GameSetup) -> GameState {
    built_seeded(rules, setup, 7)
}

/// [`built`] with the master seed `seed` (E09 T26b).
pub(super) fn built_seeded(rules: &GameRuleset, setup: &GameSetup, seed: u64) -> GameState {
    let mut game = GameState::new(rules, setup, seed).unwrap();
    let dwellings = [(T0, DWELLING_1), (T0, DWELLING_2), (T1, DWELLING_1)]
        .into_iter()
        .chain([T2, T3].map(|town| (town, DWELLING_1)));
    for (town, index) in dwellings {
        let building = BuildingId::from_index(index).unwrap();
        let towns = game.towns_mut();
        towns.build(town, building, rules.towns(), rules.core(), false);
    }
    game
}

/// The game's day `day` started: both players' turns ended until then.
pub(super) fn to_day(rules: &GameRuleset, game: &mut GameState, day: u32) {
    while game.day() < day {
        end_turn(rules, game, RED);
        end_turn(rules, game, BLUE);
    }
}

/// Each town's imps and gogs.
fn counts(game: &GameState) -> [(u16, u16); 4] {
    [T0, T1, T2, T3].map(|town| {
        let town = game.towns().get(town).unwrap();
        (town.available(0), town.available(1))
    })
}

// covers: TWN-GRO-001
#[test]
fn a_towns_inputs_are_its_owners_share_its_visitors_legion_and_its_owners_dwellings() {
    let rules = rules([1, 0, 0]);
    let game = built(&rules, &setup(&rules));
    let owned = |share_bp, legion: u16, imps: u16| {
        let mut inputs = GrowthInputs {
            owned: true,
            share_bp,
            ..GrowthInputs::NEUTRAL
        };
        inputs.legion[0] = legion;
        inputs.dwellings[0] = imps;
        inputs
    };
    // Red's T0: the Statue's share, the visitor's Legs, red's hut (its first creature the imp;
    // the town's level 1 has no upgrade, so nothing at index 7). Blue's T1: nothing (no Legion,
    // no hut). Red's T2: the share (any hero of red's) and the hut, no visitor. The neutral T3:
    // nothing (the neutral huts count for nobody).
    assert_eq!(
        crate::town::growth::inputs(&game, &rules),
        [
            owned(5_000, 5, 1),
            owned(0, 0, 0),
            owned(5_000, 0, 1),
            GrowthInputs::NEUTRAL,
        ]
    );
}

// covers: TWN-GRO-001, TWN-GRO-003, GAM-CAL-002
#[test]
fn every_town_grows_its_week_on_the_week_s_first_day_a_neutral_one_half() {
    let rules = rules([1, 0, 0]);
    let mut game = built(&rules, &setup(&rules));
    let start = [(15, 8), (15, 0), (15, 0), (15, 0)];
    assert_eq!(counts(&game), start);
    // Nothing during the week.
    to_day(&rules, &mut game, 7);
    assert_eq!(counts(&game), start);
    // Day 8: T0 15 + 7 + 5 + 1 imps and 8 + 4 gogs; T1 15; T2 15 + 7 + 1; T3 15 ÷ 2. (E09
    // T26a2: seed 7's towns' stream draws no creature's week on days 8, 15 and 22.)
    to_day(&rules, &mut game, 8);
    assert_eq!(game.week(), WeekKind::Normal);
    assert_eq!(counts(&game), [(43, 20), (30, 0), (38, 0), (22, 0)]);
    to_day(&rules, &mut game, 15);
    assert_eq!(counts(&game), [(71, 32), (45, 0), (61, 0), (29, 0)]);
    // A normal month changes nothing more.
    to_day(&rules, &mut game, 29);
    assert_eq!(game.month(), MonthKind::Normal);
    assert_eq!(counts(&game), [(127, 56), (75, 0), (107, 0), (43, 0)]);
}

// covers: TWN-GRO-004, GAM-CAL-003
#[test]
fn a_month_doubles_its_creature_s_counts_and_the_plague_halves_after_the_week() {
    // The gogs' month: T0's gogs double after day 29's week (8 + 4 × 12 = 56), nothing else.
    let rules = rules([0, 1, 0]);
    let mut game = built(&rules, &setup(&rules));
    to_day(&rules, &mut game, 29);
    assert_eq!(
        game.month(),
        MonthKind::Creature(rules.core().creature_by_key("t:gog").unwrap())
    );
    assert_eq!(counts(&game), [(127, 112), (75, 0), (107, 0), (43, 0)]);
    // The Plague: each count less its growth, then halved; the neutral T3 less half its growth
    // (`quirk.town.neutral_plague_full_growth` off in this profile, TWN-QUIRK-9).
    let rules = rules_plague();
    let mut plague = built(&rules, &setup(&rules));
    to_day(&rules, &mut plague, 29);
    assert_eq!(plague.month(), MonthKind::Plague);
    assert_eq!(
        counts(&plague),
        [
            ((127 - 28) / 2, (56 - 12) / 2),
            ((75 - 15) / 2, 0),
            ((107 - 23) / 2, 0),
            ((43 - 7) / 2, 0)
        ]
    );
}

/// [`rules`] with the Plague every month.
fn rules_plague() -> GameRuleset {
    rules([0, 0, 1])
}

// covers: TWN-EVT-004
#[test]
fn a_town_event_s_creatures_join_its_dwelling_s_count_after_its_buildings() {
    let rules = rules([1, 0, 0]);
    let mut setup = setup(&rules);
    // Red's T2 on day 2: 3 imps and 4 gogs; the gogs' dwelling is not built, so they are lost.
    setup.towns[2].events = vec![TownEventSetup {
        players: vec![RED],
        human: true,
        computer: true,
        first_day: 2,
        interval: 0,
        resources: vec![],
        buildings: SlotSet::EMPTY,
        creatures: [3, 4, 0, 0, 0, 0, 0],
    }];
    let mut game = built(&rules, &setup);
    assert_eq!(counts(&game)[2], (15, 0));
    // The event fires at red's turn start on day 2: the imps added, no week's growth.
    to_day(&rules, &mut game, 2);
    assert_eq!(counts(&game), [(15, 8), (15, 0), (18, 0), (15, 0)]);
}

/// [`rules_on`] whose every week but a month's first is a creature's (E09 T26a2).
fn rules_creature_weeks(odds: [u8; 3]) -> GameRuleset {
    rules_on(odds, |towns| {
        towns.params.as_mut().unwrap().week.creature_chance = [1, 1];
    })
}

/// The game's day `day` started, the events every player was told on the way.
pub(super) fn to_day_told(rules: &GameRuleset, game: &mut GameState, day: u32) -> Vec<GameEvent> {
    let mut events = Vec::new();
    while game.day() < day {
        for player in [RED, BLUE] {
            let command = crate::GameCommand::EndTurn;
            apply(
                game,
                rules,
                crate::Actor::Player(player),
                command,
                &mut events,
            )
            .unwrap();
        }
    }
    events
}

// covers: TWN-GRO-002, TWN-GRO-003, GAM-CAL-002, ADV-DWL-003
#[test]
fn a_creature_s_week_is_drawn_from_the_towns_stream_told_and_grown() {
    let rules = rules_creature_weeks([1, 0, 0]);
    let mut game = built(&rules, &setup(&rules));
    assert_eq!(game.week(), WeekKind::Normal);
    // The draw on day 8: the towns' stream of seed 7, the chance's draw and the creature's.
    let (core, part) = (rules.core(), rules.towns());
    let mut stream = Rng::for_stream(7, StreamId::TOWNS);
    let kind = h3_towns::draw_week(game.towns(), part, core, false, false, &mut stream);
    let [imp, gog] = ["t:imp", "t:gog"].map(|k| core.creature_by_key(k).unwrap());
    assert!(kind == WeekKind::Creature(imp) || kind == WeekKind::Creature(gog));
    let events = to_day_told(&rules, &mut game, 8);
    assert_eq!(game.week(), kind);
    let started = GameEvent::WeekStarted { week: 2, kind };
    let at = events.iter().position(|e| *e == started).unwrap();
    assert_eq!(events[at - 1], GameEvent::DayStarted { day: 8 });
    for viewer in [RED, BLUE] {
        assert_eq!(observe(&game, &rules, Viewer::Player(viewer)).week, kind);
        assert_eq!(
            crate::project(&started, Viewer::Player(viewer)),
            Some(started)
        );
    }
    // The week's creature's dwellings grow the part's bonus more (5, halved in the neutral T3).
    let (i, g) = (
        u16::from(kind == WeekKind::Creature(imp)) * 5,
        u16::from(kind == WeekKind::Creature(gog)) * 5,
    );
    assert_eq!(
        counts(&game),
        [
            (43 + i, 20 + g),
            (30 + i, 0),
            (38 + i, 0),
            (15 + (15 + i) / 2, 0)
        ]
    );
    // ADV-DWL-003 (the adventure lane's E08 T97a note): the map's huts of the week's creature
    // stay at their growth (the imps' 15, the gogs' 8).
    for hut in 4..7 {
        assert_eq!(game.adventure().dwelling_counts(hut), Some(&[15, 8][..]));
    }
    // A month's first week draws no creature: day 29 is a normal week.
    to_day(&rules, &mut game, 29);
    assert_eq!(game.week(), WeekKind::Normal);
}

// covers: TWN-GRO-002, GAM-CAL-003
#[test]
fn a_town_s_week_leaves_the_game_s_stream_where_it_was() {
    // E09 decision 75: the week's draws are the towns' stream's, so the game's (the adventure's
    // week steps, the month's kind and its monsters) is the same whether the weeks draw a
    // creature or not.
    let creature = rules_creature_weeks([1, 1, 1]);
    let normal = rules_on([1, 1, 1], |towns| {
        towns.params.as_mut().unwrap().week.creature_chance = [1, u16::MAX];
    });
    let mut differ = false;
    for seed in 0..8 {
        let setup = setup(&creature);
        let mut a = GameState::new(&creature, &setup, seed).unwrap();
        let mut b = GameState::new(&normal, &setup, seed).unwrap();
        to_day(&creature, &mut a, 29);
        to_day(&normal, &mut b, 29);
        assert_eq!(a.month(), b.month(), "seed {seed}");
        assert_eq!(a.adventure().state_hash(), b.adventure().state_hash());
        differ |= a.state_hash() != b.state_hash();
    }
    assert!(differ, "the towns' week differs");
}

/// [`rules_on`] whose Grail is level 1's `week_of` building (the Deity of Fire's role).
fn rules_deity(odds: [u8; 3]) -> GameRuleset {
    rules_on(odds, |towns| {
        let grail = towns.towns[0].buildings.iter_mut().find(|b| b.roles.grail);
        grail.unwrap().roles.week_of = Some(1);
    })
}

// covers: TWN-GRO-002, TWN-GRO-003, TWN-GRO-004, GAM-CAL-003
#[test]
fn a_building_s_week_is_every_week_and_makes_the_month_its_creature_s_without_doubling() {
    // The neutral T3's Grail: every week the imps' (no upgrade), the bonus their growth, 15; the
    // month a normal one by its odds, the imps' by the week.
    let rules = rules_deity([1, 0, 0]);
    let mut game = built(&rules, &setup(&rules));
    let grail = BuildingId::from_index(9).unwrap();
    game.towns_mut()
        .build(T3, grail, rules.towns(), rules.core(), false);
    let imp = rules.core().creature_by_key("t:imp").unwrap();
    let deity = WeekKind::Of {
        creatures: [Some(imp), None],
        bonus: 15,
    };
    to_day(&rules, &mut game, 8);
    assert_eq!(game.week(), deity);
    // Every town's imps 15 more; T3's (its Grail's growth 15 + 7) halved after the bonus.
    assert_eq!(counts(&game), [(58, 20), (45, 0), (53, 0), (33, 0)]);
    let events = to_day_told(&rules, &mut game, 29);
    assert_eq!(game.week(), deity);
    let imps = MonthKind::Creature(imp);
    assert_eq!(game.month(), imps);
    assert!(events.contains(&GameEvent::MonthStarted {
        month: 2,
        kind: imps
    }));
    // Four weeks of 43 more at T0, no doubling; the imps' monsters on the map.
    assert_eq!(
        counts(&game),
        [
            (15 + 4 * 43, 56),
            (15 + 4 * 30, 0),
            (15 + 4 * 38, 0),
            (15 + 4 * 18, 0)
        ]
    );
    assert!(!game.adventure().added_objects().is_empty());
    // Every month a creature's by its odds: still the imps', not the gogs' (no creature drawn).
    let rules = rules_deity([0, 1, 0]);
    let mut game = built(&rules, &setup(&rules));
    game.towns_mut()
        .build(T3, grail, rules.towns(), rules.core(), false);
    to_day(&rules, &mut game, 29);
    assert_eq!(game.month(), imps);
}
