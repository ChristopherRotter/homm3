//! Tests of the wandering monsters and the special grounds on the static map (E08 T20a2a): the
//! monsters' data and its checks, the draws when the game is built, the tiles a ground lies on
//! and a battle's ground there, the hashes, and what a viewer sees.

use std::collections::BTreeMap;

use h3_content::{ContentPack, CreatureDef, CreatureInfoDef};
use h3_kernel::hash::hash_of;

use super::*;
use crate::command::{AdventureCommand, AdventureCommandError, AdventureEvent};
use crate::ruleset::MAX_GAIN;
use crate::setup::MonsterSetup;
use crate::testing::{
    BARRENS, GNOLL, HALLOW, MONSTER, PEASANT, PIKEMAN, RANDOM_MONSTER, SAND, SWAMP, WOLF, hero_at,
    key, movement_rules_edited, world,
};

const CART: CreatureTypeId = CreatureTypeId::from_raw(4);
const RED: PlayerId = PlayerId::new(0);

/// [`crate::testing::movement_rules`] (faithful) with adventure-map ranges: the pikeman 10–20,
/// the gnoll 2–4, the peasant 30–40 and of level 2, the wolf none; and a war machine `t:cart` of
/// level 1. A random monster of level 1 is drawn among the pikeman, the gnoll and the wolf.
fn monster_rules() -> (Ruleset, AdventureRuleset) {
    movement_rules_edited(true, 16, |core: &mut ContentPack| {
        core.creatures[0].info.adventure_count = [10, 20];
        core.creatures[1].info.adventure_count = [2, 4];
        core.creatures[2].info.adventure_count = [30, 40];
        core.creatures[2].info.level = 2;
        core.creatures.push(CreatureDef {
            key: key("t:cart"),
            speed: 0,
            info: CreatureInfoDef {
                flags: vec!["war_machine".to_string()],
                ..CreatureInfoDef::default()
            },
        });
    })
}

fn fixed(creature: CreatureTypeId, count: u32, disposition: u8) -> MonsterSetup {
    MonsterSetup {
        creature: Some(creature),
        count,
        disposition,
        never_flees: false,
        no_growth: false,
        treasure: vec![],
    }
}

fn random(count: u32, disposition: u8) -> MonsterSetup {
    MonsterSetup {
        creature: None,
        ..fixed(PIKEMAN, count, disposition)
    }
}

/// A one-tile object of `object_type` at column `x` of row 0, blocking and visited on its tile.
fn object_at(x: u16, object_type: ObjectTypeId, monster: Option<MonsterSetup>) -> ObjectSetup {
    ObjectSetup {
        anchor: Pos::new(x, 0, 0),
        blocked: TileMask::from_bits(1),
        visitable: TileMask::from_bits(1),
        object_type: Some(object_type),
        amount: None,
        drawn: TileMask::default(),
        monster,
    }
}

/// A 6 × 3 grass map with these objects and no hero.
fn with(objects: Vec<ObjectSetup>) -> AdventureSetup {
    let mut setup = world(&["gggggg", "gggggg", "gggggg"], 1, vec![]);
    setup.map.objects = objects;
    setup
}

/// The four monsters of the draw test: a pikeman of a drawn count (friendly), 7 gnolls
/// (compliant), a random monster of a drawn count (savage) and 3 of a random monster
/// (aggressive).
fn four() -> AdventureSetup {
    with(vec![
        object_at(0, MONSTER, Some(fixed(PIKEMAN, 0, 1))),
        object_at(1, MONSTER, Some(fixed(GNOLL, 7, 0))),
        object_at(2, RANDOM_MONSTER, Some(random(0, 4))),
        object_at(3, RANDOM_MONSTER, Some(random(3, 2))),
    ])
}

/// A monster's drawn creature, count and aggression.
type Drawn = (CreatureTypeId, u32, i8);

fn drawn(state: &AdventureState) -> Vec<Drawn> {
    state
        .map()
        .objects()
        .iter()
        .filter_map(|o| o.monster.as_ref())
        .map(|m| (m.creature, m.count, m.aggression))
        .collect()
}

// covers: ADV-MON-002, ADV-MON-003, ADV-MON-004
#[test]
fn monsters_draw_their_creature_count_and_aggression_in_three_walks() {
    let (core, part) = monster_rules();
    let setup = four();
    let pool = [PIKEMAN, GNOLL, WOLF];
    let range = |creature| match creature {
        c if c == PIKEMAN => (10, 20),
        c if c == GNOLL => (2, 4),
        _ => (0, 0),
    };
    let mut seen = Vec::new();
    for seed in 0..60 {
        let state = AdventureState::new(
            &core,
            &part,
            &setup,
            &mut Rng::for_stream(seed, StreamId::SETUP),
        )
        .unwrap();
        // By hand: the random creatures, then the counts of 0, then the aggressions, each walk in
        // object order; no draw for a compliant or a savage monster.
        let mut d = Rng::for_stream(seed, StreamId::SETUP);
        let third = pool[d.below(3) as usize];
        let fourth = pool[d.below(3) as usize];
        let pikemen = d.range_inclusive(10, 20);
        let (low, high) = range(third);
        let third_count = d.range_inclusive(low, high).max(1);
        let friendly = 1 + i8::try_from(d.range_inclusive(0, 6)).unwrap();
        let aggressive = 1 + i8::try_from(d.range_inclusive(0, 9)).unwrap();
        let found = drawn(&state);
        assert_eq!(
            found,
            [
                (PIKEMAN, pikemen, friendly),
                (GNOLL, 7, -4),
                (third, third_count, 10),
                (fourth, 3, aggressive),
            ],
            "seed {seed}"
        );
        seen.push(found);
    }
    // Every creature of the pool comes up, never the peasant (level 2) nor the cart (a war
    // machine); a wolf, whose range is empty, comes as one.
    for creature in pool {
        assert!(seen.iter().any(|f| f[2].0 == creature), "{creature:?}");
    }
    assert!(seen.iter().all(|f| ![PEASANT, CART].contains(&f[2].0)));
    assert!(seen.iter().filter(|f| f[2].0 == WOLF).all(|f| f[2].1 == 1));
    for aggression in 1..=10 {
        assert!(seen.iter().any(|f| f[3].2 == aggression), "{aggression}");
    }
    // The rest of the data is the map's.
    let mut setup = four();
    setup.map.objects[1].monster = Some(MonsterSetup {
        never_flees: true,
        no_growth: true,
        treasure: vec![(ResourceId::from_raw(1), 500)],
        ..fixed(GNOLL, 7, 0)
    });
    let state = AdventureState::new(&core, &part, &setup, &mut draws()).unwrap();
    let gnolls = state.map().objects()[1].monster.as_ref().unwrap();
    assert!(gnolls.never_flees && gnolls.no_growth);
    assert_eq!(gnolls.treasure, [(ResourceId::from_raw(1), 500)]);
    assert!(
        state.map().objects()[0]
            .monster
            .as_ref()
            .unwrap()
            .treasure
            .is_empty()
    );
}

// covers: ADV-MON-001
#[test]
fn monster_data_must_fit_its_type() {
    let (core, part) = monster_rules();
    let build = |monster: Option<MonsterSetup>, object_type| {
        let setup = with(vec![
            object_at(0, MONSTER, Some(fixed(PIKEMAN, 5, 2))),
            object_at(1, object_type, monster),
        ]);
        AdventureState::new(&core, &part, &setup, &mut draws()).map(|_| ())
    };
    let bad = Err(SetupError::Monster(1));
    // Data exactly for a monster.
    assert_eq!(build(None, MONSTER), bad);
    assert_eq!(build(None, RANDOM_MONSTER), bad);
    assert_eq!(build(Some(fixed(PIKEMAN, 5, 2)), HALLOW), bad);
    // A fixed monster names a creature that is no war machine, a random one none.
    assert_eq!(build(Some(random(5, 2)), MONSTER), bad);
    assert_eq!(build(Some(fixed(PIKEMAN, 5, 2)), RANDOM_MONSTER), bad);
    assert_eq!(build(Some(fixed(CART, 5, 2)), MONSTER), bad);
    assert_eq!(
        build(Some(fixed(CreatureTypeId::from_raw(9), 5, 2)), MONSTER),
        bad
    );
    // The count's bound, the parameters' dispositions.
    assert_eq!(
        build(Some(fixed(PIKEMAN, MAX_MONSTER_COUNT, 4)), MONSTER),
        Ok(())
    );
    assert_eq!(
        build(Some(fixed(PIKEMAN, MAX_MONSTER_COUNT + 1, 4)), MONSTER),
        bad
    );
    assert_eq!(build(Some(fixed(PIKEMAN, 5, 5)), MONSTER), bad);
    // A treasure of known resources, each once, each from 1 to MAX_GAIN.
    let treasure = |treasure: Vec<(ResourceId, u32)>| {
        build(
            Some(MonsterSetup {
                treasure,
                ..fixed(PIKEMAN, 5, 2)
            }),
            MONSTER,
        )
    };
    let (wood, gold) = (ResourceId::from_raw(0), ResourceId::from_raw(1));
    assert_eq!(treasure(vec![(gold, MAX_GAIN), (wood, 1)]), Ok(()));
    assert_eq!(treasure(vec![(gold, 1), (gold, 2)]), bad);
    assert_eq!(treasure(vec![(gold, 0)]), bad);
    assert_eq!(treasure(vec![(gold, MAX_GAIN + 1)]), bad);
    assert_eq!(treasure(vec![(ResourceId::from_raw(2), 1)]), bad);
    assert_eq!(
        SetupError::Monster(1).to_string(),
        "object 1 has invalid monster data"
    );
}

#[test]
fn the_hashes_see_the_monsters_the_drawn_tiles_the_difficulty_and_the_armies_left() {
    let (core, part) = monster_rules();
    let state =
        |setup: &AdventureSetup| AdventureState::new(&core, &part, setup, &mut draws()).unwrap();
    let base = state(&four());
    assert_eq!(base, state(&four()));
    let mut changes: Vec<AdventureSetup> = Vec::new();
    let mut change = |f: &dyn Fn(&mut AdventureSetup)| {
        let mut s = four();
        f(&mut s);
        changes.push(s);
    };
    fn first(s: &mut AdventureSetup) -> &mut MonsterSetup {
        s.map.objects[1].monster.as_mut().unwrap()
    }
    change(&|s| first(s).creature = Some(PIKEMAN));
    change(&|s| first(s).count = 8);
    change(&|s| first(s).disposition = 4);
    change(&|s| first(s).never_flees = true);
    change(&|s| first(s).no_growth = true);
    change(&|s| first(s).treasure = vec![(ResourceId::from_raw(0), 1)]);
    change(&|s| s.map.objects[1].drawn = TileMask::from_bits(2));
    for (i, setup) in changes.iter().enumerate() {
        let other = state(setup);
        assert_ne!(
            other.map().content_hash(),
            base.map().content_hash(),
            "change {i}"
        );
        assert_ne!(other.state_hash(), base.state_hash(), "change {i}");
    }
    // The difficulty is a static input of the state, outside the static map.
    let mut easy = four();
    easy.difficulty = 1;
    let easy = state(&easy);
    assert_eq!(easy.difficulty(), 1);
    assert_eq!(base.difficulty(), 0);
    assert_eq!(easy.map().content_hash(), base.map().content_hash());
    assert_ne!(easy.state_hash(), base.state_hash());
    // An army a battle left (E08 T20b writes them) is the monster's count from then on.
    assert_eq!(base.army_left(1), None);
    assert_eq!(base.monster_count(1), Some(7));
    assert_eq!(base.monster_count(9), None);
    let mut left = base.clone();
    left.army_left = Some(Arc::new(BTreeMap::from([(
        1,
        vec![
            ArmyStack {
                slot: 0,
                creature: GNOLL,
                count: 2,
            },
            ArmyStack {
                slot: 3,
                creature: GNOLL,
                count: 1,
            },
        ],
    )])));
    assert_eq!(left.army_left(1).map(<[ArmyStack]>::len), Some(2));
    assert_eq!(left.army_left(0), None);
    assert_eq!(left.monster_count(1), Some(3));
    assert_ne!(left.state_hash(), base.state_hash());
    assert_eq!(left.state_hash(), hash_of(&left));
    let mut other = left.clone();
    other.army_left = Some(Arc::new(BTreeMap::from([(
        1,
        vec![ArmyStack {
            slot: 0,
            creature: GNOLL,
            count: 3,
        }],
    )])));
    assert_ne!(other.state_hash(), left.state_hash());
}

/// A ground of `object_type` anchored at `(x, y)` of `level`, lying on `width` × `height` tiles
/// to its left and above.
fn ground(
    object_type: ObjectTypeId,
    (x, y, level): (u16, u16, u8),
    (width, height): (u8, u8),
) -> ObjectSetup {
    let row = (1u64 << width) - 1;
    ObjectSetup {
        anchor: Pos::new(x, y, level),
        blocked: TileMask::default(),
        visitable: TileMask::default(),
        object_type: Some(object_type),
        amount: None,
        drawn: TileMask::from_bits((0..height).fold(0, |bits, r| bits | row << (8 * u32::from(r)))),
        monster: None,
    }
}

// covers: ADV-GRD-001, GAM-BAT-001
#[test]
fn a_ground_sets_the_battlefield_of_the_tiles_it_lies_on() {
    let (core, part) = monster_rules();
    let mut setup = world(
        &["gggggg", "gggggg", "gggggg", "gggggg", "gggggg", "gggggg"],
        2,
        vec![],
    );
    // The coast along the surface's top row.
    for x in 0..6 {
        setup.map.tiles[x].coast = true;
    }
    setup.map.objects = vec![
        // 3 × 2 tiles of hallow from (1, 0) to (3, 1), then 2 × 2 of barrens over (3, 1) and
        // (2, 1) to (4, 2); a hallow underground.
        ground(HALLOW, (3, 1, 0), (3, 2)),
        ground(BARRENS, (4, 2, 0), (2, 2)),
        ground(HALLOW, (1, 1, 1), (1, 1)),
    ];
    let state = AdventureState::new(&core, &part, &setup, &mut draws()).unwrap();
    let field = |x, y, level| state.battle_ground(&part, Pos::new(x, y, level));
    let hallow = Some(key("t:hallow"));
    let barrens = Some(key("t:barrens"));
    // On the coast the ground replaces the shore and keeps the coast's sand.
    assert_eq!(field(1, 0, 0), (SAND, hallow.clone()));
    assert_eq!(field(0, 0, 0), (SAND, Some(key("t:shore"))));
    // The last ground in map order wins where two lie.
    assert_eq!(field(2, 1, 0), (GRASS, hallow.clone()));
    assert_eq!(field(3, 1, 0), (GRASS, barrens.clone()));
    assert_eq!(field(4, 2, 0), (GRASS, barrens));
    assert_eq!(field(1, 2, 0), (GRASS, None));
    // Underground: the underground's terrain, the ground's field; not on another level.
    assert_eq!(field(1, 1, 1), (SWAMP, hallow));
    assert_eq!(field(0, 1, 1), (SWAMP, None));
    // Grounds of one type may share an anchor (nothing names them); monsters may not.
    let mut twins = setup.clone();
    twins.map.objects.push(ground(HALLOW, (3, 1, 0), (1, 1)));
    AdventureState::new(&core, &part, &twins, &mut draws()).unwrap();
    let monsters = with(vec![
        object_at(0, MONSTER, Some(fixed(PIKEMAN, 5, 2))),
        object_at(0, MONSTER, Some(fixed(GNOLL, 5, 2))),
    ]);
    assert_eq!(
        AdventureState::new(&core, &part, &monsters, &mut draws()),
        Err(SetupError::SharedAnchor(1))
    );
    let object = &state.map().objects()[0];
    assert!(object.lies_on(Pos::new(1, 0, 0)));
    assert!(!object.lies_on(Pos::new(1, 0, 1)));
    assert!(!object.lies_on(Pos::new(4, 1, 0)));
    assert_eq!(object.covers(Pos::new(1, 0, 0)), (false, false));
    // Its tiles are the ones it lies on; a ground blocks and is visited nowhere.
    let tiles: Vec<Pos> = object.tiles(state.map()).collect();
    assert_eq!(tiles.len(), 6);
    assert!(state.check_stand(&part, Pos::new(2, 1, 0)).is_ok());
}

// covers: ADV-FOG-006
#[test]
fn a_viewer_sees_a_monsters_creature_and_the_word_of_its_count() {
    let (core, part) = monster_rules();
    let mut setup = four();
    setup.heroes = vec![hero_at(1, 0, Pos::new(1, 2, 0), &[PIKEMAN])];
    setup.map.objects.push(ground(HALLOW, (5, 2, 0), (1, 3)));
    let state = AdventureState::new(&core, &part, &setup, &mut draws()).unwrap();
    let seen = state.observe(&core, &part, RED);
    let monsters: Vec<_> = seen
        .objects
        .iter()
        .map(|o| o.monster.map(|m| (m.creature, m.count_word)))
        .collect();
    let drawn = drawn(&state);
    let word = |count| part.params().monsters().unwrap().count_word(count);
    assert_eq!(
        monsters,
        [
            Some((PIKEMAN, word(drawn[0].1))),
            Some((GNOLL, 1)),
            Some((drawn[2].0, word(drawn[2].1))),
            Some((drawn[3].0, 0)),
            None,
        ]
    );
    // The ground shows where it lies.
    assert_eq!(seen.objects[4].drawn, setup.map.objects[4].drawn);
    // An army left changes the word.
    let mut left = state.clone();
    left.army_left = Some(Arc::new(BTreeMap::from([(
        1,
        vec![ArmyStack {
            slot: 0,
            creature: GNOLL,
            count: 50,
        }],
    )])));
    assert_eq!(
        left.observe(&core, &part, RED).objects[1]
            .monster
            .unwrap()
            .count_word,
        4
    );
    // Without the monsters' parameters every count is the first word: rules that have none
    // hold no monster, so a state of them shows only what other rules built.
}

// covers: ADV-MON-001
#[test]
fn a_monster_is_not_visited_before_its_answer_is_built() {
    let (core, part) = monster_rules();
    let mut setup = four();
    setup.heroes = vec![hero_at(1, 0, Pos::new(1, 1, 0), &[PIKEMAN])];
    let mut state = AdventureState::new(&core, &part, &setup, &mut draws()).unwrap();
    let before = state.clone();
    let step = AdventureCommand::MoveHero {
        hero: HeroId::new(1),
        path: vec![Pos::new(1, 0, 0)],
    };
    let mut events: Vec<AdventureEvent> = Vec::new();
    assert_eq!(
        state.apply(&core, &part, RED, &step, &mut events),
        Err(AdventureCommandError::Visit { step: 0 })
    );
    assert_eq!(state, before);
    assert!(events.is_empty());
    let reach = state.reach(&core, &part, HeroId::new(1)).unwrap();
    assert_eq!(reach.points_left(Pos::new(1, 0, 0)), None);
    assert!(reach.points_left(Pos::new(0, 1, 0)).is_some());
}
