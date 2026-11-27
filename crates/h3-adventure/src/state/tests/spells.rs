//! Tests of the heroes' day spells and the adventure's half of a cast (E08 T142a; increment 15's
//! design note §8, §10.3, §10.4, §12; ADV-SPL-002, ADV-SPL-005, ADV-SPL-006, ADV-MOV-009,
//! ADV-MOV-010): Fly's and Water Walk's tests under `ADV-QUIRK-54` and A-49 (`ADV-QUIRK-57`), the
//! levels the paths read, the drop under `ADV-QUIRK-17`, the new day and a boarding, the block's
//! one encoding, the owner's view and the events told after a command. Batch E15's rows as
//! sources: E15-11 (a re-cast free, Angel Wings, a boarding, Fly aboard), E15-12 (Water Walk cast
//! aboard holds after landing).

use alloc::sync::Arc;

use h3_heroes::{HeroDetails, Movement};

use super::*;
use crate::command::{AdventureCommand, AdventureCommandError, AdventureEvent, AdventureEventKind};
use crate::ruleset::AdventureSpell;
use crate::testing::{
    ADMIRAL, FLYING, PEASANT, WATER_WALKING, coasted, hero_at, hero_details, hero_rules,
    hero_rules_with, hero_skills, sea_skills, stream, world,
};

const RED: PlayerId = PlayerId::new(0);
const BLUE: PlayerId = PlayerId::new(1);
const HERO: HeroId = HeroId::new(1);

fn p(x: u16, y: u16) -> Pos {
    Pos::new(x, y, 0)
}

/// The store: hero 1 with `skills` (the test skills [`FLYING`] and [`WATER_WALKING`] at 3 stand
/// for Angel Wings and Boots of Levitation), hero 0 and blue's hero 2 without.
fn store(skills: &[(h3_content::SkillId, u8)]) -> Vec<Arc<HeroDetails>> {
    vec![
        Arc::new(hero_details(&[])),
        Arc::new(hero_details(skills)),
        Arc::new(hero_details(&[])),
    ]
}

/// The started state of `rows` under [`hero_rules`], red's hero 1 (peasants: 1300 points) on
/// `at`, blue's hero 2 in the last row's first tile.
fn spell_world(rules: &(Ruleset, AdventureRuleset), rows: &[&str], at: Pos) -> AdventureState {
    let last = u16::try_from(rows.len() - 1).unwrap();
    let heroes = vec![
        hero_at(1, 0, at, &[PEASANT]),
        hero_at(2, 1, p(0, last), &[PEASANT]),
    ];
    crate::testing::started(rules, &world(rows, 1, heroes)).unwrap()
}

/// The adventure's cast of `effect` at `mastery` without a target (the core spell unread by Fly
/// and Water Walk).
pub(super) const fn untargeted(effect: AdventureSpell, mastery: u8) -> Cast {
    Cast {
        spell: h3_content::SpellId::from_raw(0),
        effect,
        mastery,
        target: None,
    }
}

/// Red's check of hero 1's cast of `spell` at `mastery` with `store`.
fn check(
    state: &AdventureState,
    rules: &(Ruleset, AdventureRuleset),
    store: &[Arc<HeroDetails>],
    (spell, mastery): (AdventureSpell, u8),
) -> Result<(), CastError> {
    state.check_cast(
        (&rules.0, &rules.1),
        store,
        (RED, HERO),
        untargeted(spell, mastery),
    )
}

/// Checks and makes red's hero 1's cast, as the game does; its events dropped.
fn cast(
    state: &mut AdventureState,
    rules: &(Ruleset, AdventureRuleset),
    store: &[Arc<HeroDetails>],
    (spell, mastery): (AdventureSpell, u8),
) -> Result<(), CastError> {
    check(state, rules, store, (spell, mastery))?;
    let cast = untargeted(spell, mastery);
    let made = state.apply_cast(
        (&rules.0, &rules.1, store),
        (RED, HERO),
        cast,
        &mut stream(),
        &mut Vec::<AdventureEvent>::new(),
    );
    assert_eq!(made, Ok(()));
    Ok(())
}

/// Moves red's hero 1 along `path`; the events.
fn walk(
    state: &mut AdventureState,
    rules: &(Ruleset, AdventureRuleset),
    store: &[Arc<HeroDetails>],
    path: &[Pos],
) -> Result<Vec<AdventureEvent>, AdventureCommandError> {
    let mut events = Vec::new();
    let command = AdventureCommand::MoveHero {
        hero: HERO,
        path: path.to_vec(),
    };
    state.apply(
        &rules.0,
        &rules.1,
        store,
        (RED, Weekday::FIRST, &mut stream()),
        &command,
        &mut events,
    )?;
    Ok(events)
}

fn row(x: u16) -> Vec<Pos> {
    (1..=x).map(|x| p(x, 0)).collect()
}

fn at(state: &AdventureState) -> (Pos, u32) {
    let hero = state.hero(HERO).unwrap();
    (hero.pos, hero.movement)
}

const fn fly(level: u8) -> DaySpells {
    DaySpells {
        fly: Some(level),
        ..DaySpells::NONE
    }
}

const fn water_walk(level: u8) -> DaySpells {
    DaySpells {
        water_walk: Some(level),
        ..DaySpells::NONE
    }
}

// covers: ADV-SPL-005, ADV-MOV-009
#[test]
fn fly_sets_the_flying_level_its_steps_and_reach_read() {
    // §10.3, §8: Fly at Advanced (2) makes a walker a flyer for the day: over the obstacles and
    // the water at the level's 120 a step, the lesser 100 from grass (ADV-MOV-009).
    let rules = hero_rules(true);
    let rows = ["gBB~~gg", "ggggggg"];
    let none = store(&[]);
    let mut state = spell_world(&rules, &rows, p(0, 0));
    assert!(walk(&mut state.clone(), &rules, &none, &row(5)).is_err());
    assert_eq!(state.day_spells(HERO), DaySpells::NONE);
    assert_eq!(
        cast(&mut state, &rules, &none, (AdventureSpell::Fly, 2)),
        Ok(())
    );
    assert_eq!(state.day_spells(HERO), fly(2));
    let reach = state.reach(&rules.0, &rules.1, &none, HERO).unwrap();
    assert_eq!(reach.points_left(p(5, 0)), Some(720));
    walk(&mut state, &rules, &none, &row(5)).unwrap();
    assert_eq!(at(&state), (p(5, 0), 720));
}

// covers: ADV-SPL-006, ADV-MOV-010
#[test]
fn water_walk_sets_the_water_walking_level_its_steps_read() {
    // §10.4: Water Walk at None (0) crosses plain water at 140 a step from water.
    let rules = hero_rules(true);
    let rows = ["g~~g", "gggg"];
    let none = store(&[]);
    let mut state = spell_world(&rules, &rows, p(0, 0));
    assert_eq!(
        cast(&mut state, &rules, &none, (AdventureSpell::WaterWalk, 0)),
        Ok(())
    );
    assert_eq!(state.day_spells(HERO), water_walk(0));
    walk(&mut state, &rules, &none, &row(3)).unwrap();
    // 100 from grass onto the water, 140 from each water tile.
    assert_eq!(at(&state), (p(3, 0), 1300 - 100 - 140 - 140));
}

// covers: ADV-SPL-005, ADV-SPL-006
#[test]
fn a_recast_is_refused_free_under_adv_quirk_54_and_only_a_higher_one_is_taken_off() {
    // E15-11: the second Fly free and nothing, and with Angel Wings nothing (`faithful`); off,
    // only a cast above the level now (the artifacts' 3 included) is taken.
    for faithful in [true, false] {
        let rules = hero_rules(faithful);
        let none = store(&[]);
        let mut state = spell_world(&rules, &["ggg", "ggg"], p(0, 0));
        for spell in [AdventureSpell::Fly, AdventureSpell::WaterWalk] {
            cast(&mut state, &rules, &none, (spell, 1)).unwrap();
            let before = state.clone();
            let again = cast(&mut state, &rules, &none, (spell, 2));
            if faithful {
                assert_eq!(again, Err(CastError::AlreadyOn));
                assert_eq!(state, before, "a refused cast changed the state");
            } else {
                assert_eq!(again, Ok(()));
                assert_eq!(
                    cast(&mut state, &rules, &none, (spell, 2)),
                    Err(CastError::NotHigher)
                );
            }
        }
        let expected = if faithful { 1 } else { 2 };
        assert_eq!(
            state.day_spells(HERO),
            DaySpells {
                fly: Some(expected),
                water_walk: Some(expected),
                doors: 0,
            }
        );
        // Angel Wings and Boots of Levitation: never taken, the artifacts' level 3.
        let worn = store(&[(FLYING, 3), (WATER_WALKING, 3)]);
        let fresh = spell_world(&rules, &["ggg", "ggg"], p(0, 0));
        let refused = if faithful {
            CastError::AlreadyOn
        } else {
            CastError::NotHigher
        };
        for spell in [AdventureSpell::Fly, AdventureSpell::WaterWalk] {
            assert_eq!(check(&fresh, &rules, &worn, (spell, 3)), Err(refused));
        }
    }
}

// covers: ADV-SPL-005, ADV-SPL-006
#[test]
fn aboard_fly_is_refused_and_water_walk_is_cast_without_its_tests() {
    // E15-11: Fly aboard refused (GENRLTXT 193), free; E15-12 (A-49, `ADV-QUIRK-57`): Water Walk
    // aboard is cast, even with the Boots and again at its level, under both profiles (the
    // re-cast tests skipped aboard), and its level holds after landing the same day.
    for faithful in [true, false] {
        let rules = hero_rules(faithful);
        let rows = ["gb~", "ggg"];
        let worn = store(&[(WATER_WALKING, 3)]);
        let mut state = spell_world(&rules, &rows, p(0, 0));
        walk(&mut state, &rules, &worn, &[p(1, 0)]).unwrap();
        assert!(state.hero(HERO).unwrap().boat.is_some());
        assert_eq!(
            check(&state, &rules, &worn, (AdventureSpell::Fly, 3)),
            Err(CastError::InBoat)
        );
        for _ in 0..2 {
            assert_eq!(
                cast(&mut state, &rules, &worn, (AdventureSpell::WaterWalk, 1)),
                Ok(()),
                "faithful {faithful}"
            );
        }
        // At sea the level prices nothing (a boat's mode, ADV-SEA-004); a landing writes no day
        // spell (`land` leaves them), so it holds ashore the same day.
        assert_eq!(state.day_spells(HERO), water_walk(1));
    }
}

/// A Water Walk cast aboard prices the hero's steps from plain water at its level once it lands
/// (A-49, `ADV-QUIRK-57`; ADV-SPL-006, ADV-MOV-010).
// covers: ADV-SPL-006, ADV-MOV-010
#[test]
fn water_walk_cast_aboard_prices_the_water_at_its_level_after_landing() {
    for faithful in [true, false] {
        // Free boarding keeps the points over the landing.
        let rules = hero_rules_with(faithful, [hero_skills(), sea_skills()].concat());
        let admiral = store(&[(ADMIRAL, 1)]);
        let heroes = vec![
            hero_at(1, 0, p(0, 0), &[PEASANT]),
            hero_at(2, 1, p(0, 1), &[PEASANT]),
        ];
        let setup = coasted(&["gb~~g", "ggggg"], heroes);
        let mut state = crate::testing::started(&rules, &setup).unwrap();
        walk(&mut state, &rules, &admiral, &[p(1, 0)]).unwrap();
        cast(&mut state, &rules, &admiral, (AdventureSpell::WaterWalk, 2)).unwrap();
        walk(&mut state, &rules, &admiral, &[p(2, 1)]).unwrap();
        let (ashore, points) = at(&state);
        assert_eq!(ashore, p(2, 1));
        assert!(state.hero(HERO).unwrap().boat.is_none());
        // 100 from grass onto the water, then the level's 120 from each water tile (not None's 140).
        let from_water = rules.1.params().water_walk_cost(2);
        assert_eq!(from_water, 120);
        walk(&mut state, &rules, &admiral, &[p(2, 0), p(3, 0), p(4, 0)]).unwrap();
        assert_eq!(at(&state), (p(4, 0), points - 100 - 2 * from_water));
    }
}

// covers: ADV-SPL-002
#[test]
fn a_boarding_clears_fly_and_water_walk_and_the_new_day_clears_every_spell() {
    // §12: boarding clears +0x112 and +0x116 (E15-11: Fly 0 → −1), the door's casts stay; the
    // new day clears them all, a garrison's hero's too.
    let rules = hero_rules(true);
    let rows = ["gb~", "ggg"];
    let none = store(&[]);
    let mut state = spell_world(&rules, &rows, p(0, 0));
    let all = DaySpells {
        fly: Some(1),
        water_walk: Some(2),
        doors: 3,
    };
    state.set_day_spells(HERO, all);
    state.set_day_spells(HeroId::new(2), fly(0));
    walk(&mut state, &rules, &none, &[p(1, 0)]).unwrap();
    assert_eq!(
        state.day_spells(HERO),
        DaySpells {
            doors: 3,
            ..DaySpells::NONE
        }
    );
    // A boarding with no spell on leaves the block as it is.
    let held = state.day_spell_book();
    let mut sailed = state.clone();
    sailed.board_day_spells(HERO);
    assert_eq!(sailed.day_spell_book(), held);
    state.start_day(&rules.0, &rules.1, &none);
    assert_eq!(state.day_spells(HERO), DaySpells::NONE);
    assert_eq!(state.day_spells(HeroId::new(2)), DaySpells::NONE);
    assert_eq!(state.day_spell_book(), None);
}

// covers: ADV-SPL-002
#[test]
fn the_day_spells_block_keeps_only_heroes_with_some() {
    // No spells set for a hero without any leave no block; a hero's last spells cleared drop it.
    let rules = hero_rules(true);
    let mut state = spell_world(&rules, &["ggg", "ggg"], p(0, 0));
    state.set_day_spells(HERO, DaySpells::NONE);
    assert_eq!(state.day_spell_book(), None);
    let door = DaySpells {
        doors: 1,
        ..DaySpells::NONE
    };
    state.set_day_spells(HERO, door);
    assert_eq!(state.day_spells(HERO), door);
    state.set_day_spells(HERO, DaySpells::NONE);
    assert_eq!(state.day_spell_book(), None);
}

// covers: ADV-SPL-001
#[test]
fn other_heroes_are_refused() {
    // Another player's hero or one off the map is not the caster's.
    let rules = hero_rules(true);
    let none = store(&[]);
    let state = spell_world(&rules, &["ggg", "ggg"], p(0, 0));
    for (player, hero) in [(BLUE, HERO), (RED, HeroId::new(2)), (RED, HeroId::new(9))] {
        assert_eq!(
            state.check_cast(
                (&rules.0, &rules.1),
                &none,
                (player, hero),
                untargeted(AdventureSpell::Fly, 0)
            ),
            Err(CastError::NotYourHero(hero))
        );
    }
}

// covers: ADV-SPL-002, ADV-MOV-006
#[test]
fn the_drop_to_0_prices_with_the_spells_levels_under_adv_quirk_17() {
    // The original's drop reads the spells' fields (+0x112, +0x116): under `faithful` a flyer by
    // the spell loses points below the fly price from an obstacle, a flyer by the artifacts
    // prices a walk there (`ADV-QUIRK-17`); `strict-fixed` prices both alike.
    let rules = hero_rules(true);
    let movement = Movement {
        flying: Some(0),
        ..Movement::default()
    };
    let core = &rules.0;
    let part = &rules.1;
    let army = [ArmyStack {
        slot: 0,
        creature: PEASANT,
        count: 1,
    }];
    let tile = StaticTile {
        terrain: TerrainId::from_raw(0),
        road: None,
        river: None,
        blocked: true,
        visitable: false,
        no_native: false,
        winds: false,
        hole: false,
    };
    let by_spell = Pricing::new(core, part, &army, (&movement, fly(0)));
    let by_wings = Pricing::new(core, part, &army, (&movement, DaySpells::NONE));
    // From a blocked grass tile the fly price at level 0 is 140, the walk's 100.
    assert_eq!(by_spell.left_after(&tile, 120), 0);
    assert_eq!(by_wings.left_after(&tile, 120), 120);
    let strict = hero_rules(false);
    let by_wings = Pricing::new(&strict.0, &strict.1, &army, (&movement, DaySpells::NONE));
    assert_eq!(by_wings.left_after(&tile, 120), 0);
}

// covers: ADV-SPL-002, ADV-SPL-010
#[test]
fn the_day_spells_are_hashed_kept_in_one_encoding_and_shown_to_their_owner_alone() {
    let rules = hero_rules(true);
    let none = store(&[]);
    let mut state = spell_world(&rules, &["ggg", "ggg"], p(0, 0));
    let start = state.clone();
    let own = state.own_heroes();
    cast(&mut state, &rules, &none, (AdventureSpell::Fly, 1)).unwrap();
    assert_ne!(state.state_hash(), start.state_hash());
    // Told after the command, to the owner alone.
    let mut events = Vec::new();
    own.tell(&state, &mut events);
    assert_eq!(
        events,
        vec![AdventureEvent {
            kind: AdventureEventKind::DaySpellsChanged {
                hero: HERO,
                spells: fly(1),
            },
            seen_by: Viewers::NONE.with(RED),
        }]
    );
    assert_eq!(events[0].project(BLUE), None);
    // The owner's view shows them; the other player's shows no own view.
    let red = state.observe(&rules.0, &rules.1, RED);
    assert_eq!(red.hero(HERO).unwrap().own.as_ref().unwrap().spells, fly(1));
    let blue = state.observe(&rules.0, &rules.1, BLUE);
    assert!(blue.hero(HERO).is_none_or(|view| view.own.is_none()));
    // Cleared again, the block goes: the state and its hash are the start's.
    state.set_day_spells(HERO, DaySpells::NONE);
    assert_eq!(state.day_spell_book(), None);
    assert_eq!(state, start);
    assert_eq!(state.state_hash(), start.state_hash());
    // A second hero's entry beside the first, then the first's removed.
    state.set_day_spells(HeroId::new(2), fly(0));
    state.set_day_spells(HERO, fly(2));
    state.set_day_spells(HERO, DaySpells::NONE);
    assert_eq!(state.day_spells(HeroId::new(2)), fly(0));
    assert_eq!(state.day_spell_book().map(|book| book.len()), Some(1));
}
