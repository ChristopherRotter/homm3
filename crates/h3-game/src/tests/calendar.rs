//! Tests of the game's day advance (E08 T85b; increment 9's design note §14; survey
//! `E08-economy.md` §7, §8, §12): day 1 runs no new day; each later day pays every player its
//! mines' and heroes' income, a week's first day its armies' weekly resources too, after the week's
//! step; a month's first day draws the month's kind, and a creature's month adds its monsters;
//! every hero regains its spell points.

use h3_adventure::{
    AdventureEvent, AdventureEventKind, Holds, MonsterSetup, MonthParamsDef, ObjectSetup, TileMask,
};
use h3_content::{CreatureAbility, ResourceId};
use h3_heroes::ArtifactInstance;

use super::*;
use crate::observe::Viewer;
use crate::{MonthKind, observe, project};

const RED: PlayerId = PlayerId::new(0);
const BLUE: PlayerId = PlayerId::new(1);
const GOLD: ResourceId = ResourceId::from_raw(0);
const GOG: CreatureTypeId = CreatureTypeId::from_raw(1);

/// The core of [`rules`](super::rules) with a purse of 500 gold a day in the ring slot, imps of 2
/// to 3 on the map (a month's creature's count) and gogs that give 3 gold a week (as the Crystal
/// Dragons' crystal).
fn core() -> Arc<Ruleset> {
    let mut pack = core_pack(false);
    pack.artifacts.push(ArtifactDef {
        effects: vec![HeroEffect::DailyResource {
            resource: key("t:gold"),
            amount: 500,
        }],
        ..artifact("t:purse", "t:ring_slot")
    });
    pack.creatures[0].info.adventure_count = [2, 3];
    pack.creatures[1].info.abilities = vec![CreatureAbility::WeeklyResource {
        resource: key("t:gold"),
        amount: 3,
    }];
    Arc::new(Ruleset::compile(&[pack], "t:faithful").unwrap())
}

/// The rules of [`core`] with these months (none: every month a normal one).
fn rules(months: Option<MonthParamsDef>) -> GameRuleset {
    let core = core();
    let mut pack = adventure_pack(vec![88, 25], 2);
    pack.params.as_mut().unwrap().months = months;
    let part = AdventureRuleset::compile(&core, &[pack]).unwrap();
    let battle = battle_part(&core, 7);
    GameRuleset::new(core, Arc::new(part), Arc::new(battle)).unwrap()
}

/// Months of these odds whose creature is the imp, a monster on every free land tile.
fn months(odds: [u8; 3]) -> Option<MonthParamsDef> {
    Some(MonthParamsDef {
        odds,
        creatures: vec![key("t:imp")],
        monster: key("t:monster"),
        spawn_one_in: NonZeroU16::new(1).unwrap(),
        spawn_factor: NonZeroU8::new(2).unwrap(),
        spawn_aggression: [10, 10],
    })
}

/// An object of type `k` at `anchor`, blocking and visited there.
fn object(rules: &GameRuleset, k: &str, anchor: Pos) -> ObjectSetup {
    ObjectSetup {
        scenery: Default::default(),
        anchor,
        blocked: TileMask::from_bits(1),
        visitable: TileMask::from_bits(1),
        object_type: rules.adventure().object_type_by_key(k),
        amount: None,
        drawn: TileMask::default(),
        monster: None,
        guards: None,
        holds: Holds::Nothing,
        owner: None,
    }
}

/// [`setup`] with red's hero at (0, 0) wearing the purse, blue's at (2, 2) with gogs in two
/// stacks; red owns a gold mine at (3, 0) and an abandoned mine of gold at (0, 3), blue a gold
/// mine at (2, 0); 10 gogs wander at (1, 3).
fn game(rules: &GameRuleset) -> GameState {
    let core = rules.core();
    let purse: ArtifactInstance = core.artifact_by_key("t:purse").unwrap().into();
    let gogs = |slot, count| ArmyStack {
        slot,
        creature: GOG,
        count,
    };
    let mut setup = setup();
    setup.heroes[0].details.artifacts = Some(HeroArtifactsSetup {
        worn: wearing(core, &[purse]),
        backpack: vec![],
    });
    setup.heroes[1].army = HeroArmy::Custom(vec![gogs(0, 2), gogs(1, 1)]);
    let owned = |k, anchor, owner| ObjectSetup {
        owner: Some(owner),
        ..object(rules, k, anchor)
    };
    setup.map.objects = vec![
        owned("t:mine", Pos::new(3, 0, 0), RED),
        ObjectSetup {
            holds: Holds::Resources(vec![GOLD]),
            ..owned("t:abandoned", Pos::new(0, 3, 0), RED)
        },
        owned("t:mine", Pos::new(2, 0, 0), BLUE),
        ObjectSetup {
            monster: Some(MonsterSetup {
                creature: Some(GOG),
                count: 10,
                disposition: 5,
                never_flees: false,
                no_growth: false,
                treasure: vec![],
                artifact: None,
            }),
            ..object(rules, "t:monster", Pos::new(1, 3, 0))
        },
    ];
    GameState::new(rules, &setup, 7).unwrap()
}

/// Ends both players' turns until day `day` starts; the last day's events.
fn until(state: &mut GameState, rules: &GameRuleset, day: u32) -> Vec<GameEvent> {
    let mut events = Vec::new();
    while state.day() < day {
        events.clear();
        for player in [RED, BLUE] {
            state.end_turn(rules, player, &mut events).unwrap();
        }
    }
    events
}

/// The gold a day's events gave each player.
fn gold(events: &[GameEvent]) -> [u32; 2] {
    let mut gold = [0; 2];
    for event in events {
        if let GameEvent::ResourceGained { player, amount, .. } = *event {
            gold[player.index()] += amount;
        }
    }
    gold
}

// covers: GAM-CAL-002, GAM-RES-003, GAM-RES-004
#[test]
fn day_one_pays_nothing_and_every_later_day_pays_the_mines_and_the_heroes_income() {
    let rules = rules(None);
    let mut state = game(&rules);
    // Day 1 (difficulty 1: 2 000 gold each) pays nothing: the original's start runs no new day.
    assert_eq!(state.treasury(RED), Some(&[2000][..]));
    assert_eq!(state.treasury(BLUE), Some(&[2000][..]));
    // Day 2, before the first turn: red's two mines (the abandoned one at its drawn gold's 1 000)
    // and its hero's purse, one event of 2 500; blue's mine. No week starts.
    let events = until(&mut state, &rules, 2);
    assert_eq!(
        events[3..],
        [
            GameEvent::DayStarted { day: 2 },
            GameEvent::ResourceGained {
                player: RED,
                resource: GOLD,
                amount: 2500,
            },
            GameEvent::ResourceGained {
                player: BLUE,
                resource: GOLD,
                amount: 1000,
            },
            GameEvent::TurnStarted { player: RED },
        ]
    );
    assert_eq!(state.treasury(RED), Some(&[4500][..]));
    assert_eq!(state.treasury(BLUE), Some(&[3000][..]));
    // Each day the same, a flag of the day paying from the next (none here).
    assert_eq!(gold(&until(&mut state, &rules, 7)), [2500, 1000]);
}

// covers: GAM-CAL-002, GAM-RES-005, ADV-MON-013
#[test]
fn a_weeks_first_day_runs_the_week_step_and_pays_the_weekly_resources_once() {
    let rules = rules(None);
    let mut state = game(&rules);
    let monster = 3;
    let events = until(&mut state, &rules, 7);
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, GameEvent::WeekStarted { .. }))
    );
    assert_eq!(state.adventure().monster_count(monster), Some(10));
    // Day 8: the week (the gogs grow by a tenth in sixteenths: 160 + 16, 11), then the income
    // with blue's gogs' 3 gold once for its two stacks.
    let events = until(&mut state, &rules, 8);
    assert_eq!(events[3], GameEvent::DayStarted { day: 8 });
    assert_eq!(events[4], GameEvent::WeekStarted { week: 2 });
    assert_eq!(state.adventure().monster_count(monster), Some(11));
    assert_eq!(gold(&events), [2500, 1003]);
    // Day 9 pays no weekly resource; day 15 starts week 3, no month yet.
    assert_eq!(gold(&until(&mut state, &rules, 9)), [2500, 1000]);
    let events = until(&mut state, &rules, 15);
    assert_eq!(events[4], GameEvent::WeekStarted { week: 3 });
    assert_eq!(gold(&events), [2500, 1003]);
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, GameEvent::MonthStarted { .. }))
    );
}

// covers: GAM-CAL-002, GAM-CAL-003, ADV-MON-014
#[test]
fn a_months_first_day_draws_its_kind_and_a_creatures_month_adds_its_monsters() {
    // Without the months' parameters every month is a normal one.
    let rules = rules(None);
    let mut state = game(&rules);
    let events = until(&mut state, &rules, 29);
    assert_eq!(state.month(), MonthKind::Normal);
    assert_eq!(
        events[4..6],
        [
            GameEvent::WeekStarted { week: 5 },
            GameEvent::MonthStarted {
                month: 2,
                kind: MonthKind::Normal,
            },
        ]
    );
    // Only the Plague: drawn, no monster added. The kind is in the state hash: a normal month
    // drawn from the same stream position (one draw each) hashes otherwise.
    let rules = rules_with_odds([0, 0, 1]);
    let mut state = game(&rules);
    until(&mut state, &rules, 29);
    assert_eq!(state.month(), MonthKind::Plague);
    assert!(state.adventure().added_objects().is_empty());
    let normal_rules = rules_with_odds([1, 0, 0]);
    let mut normal = game(&normal_rules);
    until(&mut normal, &normal_rules, 29);
    assert_eq!(normal.month(), MonthKind::Normal);
    assert_ne!(normal.state_hash(), state.state_hash());
    // Only a creature's: the imp's month (the observation's, every viewer's), its monsters on
    // every free land tile (one in one): 16 tiles but the rock, the four objects' and the two
    // heroes'. Its events after the month's, as the original's month adds them after its message.
    let rules = rules_with_odds([0, 1, 0]);
    let mut state = game(&rules);
    assert_eq!(state.month(), MonthKind::Normal);
    let events = until(&mut state, &rules, 29);
    let imps = MonthKind::Creature(IMP);
    assert_eq!(state.month(), imps);
    let started = GameEvent::MonthStarted {
        month: 2,
        kind: imps,
    };
    assert_eq!(events[5], started);
    let added = |e: &GameEvent| {
        matches!(
            e,
            GameEvent::Adventure(AdventureEvent {
                kind: AdventureEventKind::MonsterAdded { .. },
                ..
            })
        )
    };
    assert!(events[6..15].iter().all(added));
    assert_eq!(state.adventure().added_objects().len(), 9);
    for viewer in [RED, BLUE] {
        let viewer = Viewer::Player(viewer);
        assert_eq!(observe(&state, &rules, viewer).month, imps);
        assert_eq!(project(&started, viewer), Some(started));
        let week = GameEvent::WeekStarted { week: 5 };
        assert_eq!(project(&week, viewer), Some(week));
    }
}

fn rules_with_odds(odds: [u8; 3]) -> GameRuleset {
    rules(months(odds))
}

// covers: HER-MAN-002
#[test]
fn every_hero_regains_its_spell_points_at_a_new_day() {
    let rules = rules(None);
    let mut state = game(&rules);
    let (red, blue) = (HeroId::new(0), HeroId::new(1));
    let full = state.hero(blue).unwrap().mana;
    assert!(full > 1);
    // Day 2: red's hero, at 0, regains the base 1; blue's, full, keeps its points.
    state.set_mana(red, 0);
    let fork = state.clone();
    until(&mut state, &rules, 2);
    assert_eq!(state.hero(red).unwrap().mana, 1);
    assert_eq!(state.hero(blue).unwrap().mana, full);
    // A fork sharing the store before the write keeps its points.
    assert_eq!(fork.hero(red).unwrap().mana, 0);
    // Points above the maximum stay; with every hero full no write happens.
    state.set_mana(red, full + 5);
    until(&mut state, &rules, 3);
    assert_eq!(state.hero(red).unwrap().mana, full + 5);
    let before = state.clone();
    until(&mut state, &rules, 4);
    assert_eq!(before.heroes().as_ptr(), state.heroes().as_ptr());
    assert!(Arc::ptr_eq(&before.heroes()[0], &state.heroes()[0]));
}

// covers: GAM-CAL-004
#[test]
fn the_maps_timed_events_are_in_the_state_hash() {
    let rules = rules(None);
    let plain = game(&rules);
    let mut setup = setup();
    setup.events = vec![TimedEventSetup {
        players: vec![RED],
        human: true,
        computer: false,
        first_day: 2,
        interval: 0,
        resources: vec![(GOLD, -100)],
        message: 0,
    }];
    let with = GameState::new(&rules, &setup, 7).unwrap();
    setup.events.clear();
    let without = GameState::new(&rules, &setup, 7).unwrap();
    assert_ne!(with.state_hash(), without.state_hash());
    assert_ne!(plain.state_hash(), without.state_hash());
}
