//! Tests of the cast on the adventure map through the game (E08 T142a; increment 15's design note
//! §14, E08 decision 414; GAM-SPL-001, ADV-SPL-001): the game's checks in the original's order,
//! each refusal changing nothing, the cost at the mastery on the hero's ground spent after the
//! adventure's effect, and the events. Batch E15's rows as sources: E15-4 (Cursed Ground and the
//! anti-magic garrison refuse free), E15-5 (Magic Plains casts at Expert), E15-11 (a re-cast).
//! Since E08 T143a Summon Boat and Scuttle Boat (ADV-SPL-003, ADV-SPL-004): their chance drawn
//! from the caster's player's stream, a charged failure told as `CastFailed`, a refusal drawing
//! nothing. Since E08 T143b Dimension Door (ADV-SPL-007 to ADV-SPL-009): the hero relocated for
//! its movement and spell points, a wrong target refused free or, under `ADV-QUIRK-55`, charged.

use h3_adventure::{
    AdventureEvent, AdventureEventKind, AdventureSpell, AdventureSpellDef, BoatSetup, CastError,
    CastFailure, DaySpells, Holds, ObjectSetup, Scenery, SpellParams, TileMask, Viewers,
};
use h3_content::SpellId;
use h3_heroes::HeroChange;

use super::decisions::{move_hero, play};
use super::rewards::answered;
use super::*;
use crate::MapCastError;

const RED: PlayerId = PlayerId::new(0);
const BLUE: PlayerId = PlayerId::new(1);
const HERO: HeroId = HeroId::new(0);

/// The core of [`core_pack`] with a level-5 air spell of the adventure category, `t:door`, and
/// since E08 T143a two of level 1 and 2 with the original's chance by mastery, `t:summon` and
/// `t:scuttle`, under `profile`: the pack's, or since E08 T143b `t:charged` (`ADV-QUIRK-55` on).
fn cast_core(profile: &str) -> Ruleset {
    let mut pack = core_pack(false);
    pack.profiles.push(ProfileDef {
        key: key("t:charged"),
        quirks: vec![("quirk.adventure.door_failure_charged".to_string(), true)],
    });
    for (k, level) in [("t:summon", 1), ("t:scuttle", 2)] {
        pack.spells.push(SpellDef {
            key: key(k),
            name: "Boat".into(),
            category: SpellCategory::Adventure,
            level,
            schools: vec![key("t:air")],
            cost: [8, 7, 7, 7],
            power: 0,
            effect: [50, 50, 75, 100],
            granted_by_name_only: false,
        });
    }
    pack.spells.push(SpellDef {
        key: key("t:door"),
        name: "Door".into(),
        category: SpellCategory::Adventure,
        level: 5,
        schools: vec![key("t:air")],
        cost: [25, 25, 25, 20],
        power: 0,
        effect: [1, 2, 3, 4],
        granted_by_name_only: false,
    });
    Ruleset::compile(&[pack], profile).unwrap()
}

/// The test rules with the casts' content: `t:gate` (an adventure spell of air, level 3, 5 or 4
/// points) acts as Fly, `t:door` as Dimension Door, `t:summon` as Summon Boat and `t:scuttle` as
/// Scuttle Boat; two grounds, `t:cursed_ground` (no spell above level 1) and `t:plains_ground`
/// (air at Expert).
fn cast_rules() -> GameRuleset {
    cast_rules_under("t:faithful")
}

/// [`cast_rules`] under the core's `profile` ([`cast_core`]).
fn cast_rules_under(profile: &str) -> GameRuleset {
    let core = Arc::new(cast_core(profile));
    let mut pack = adventure_pack(vec![88, 25], 2);
    pack.spells = vec![
        AdventureSpellDef {
            spell: key("t:gate"),
            effect: AdventureSpell::Fly,
        },
        AdventureSpellDef {
            spell: key("t:door"),
            effect: AdventureSpell::DimensionDoor,
        },
        AdventureSpellDef {
            spell: key("t:summon"),
            effect: AdventureSpell::SummonBoat,
        },
        AdventureSpellDef {
            spell: key("t:scuttle"),
            effect: AdventureSpell::ScuttleBoat,
        },
    ];
    pack.params.as_mut().unwrap().spells = Some(SpellParams {
        door_points: [300, 300, 300, 200],
        target_range: [9, 8],
        boats_most: NonZeroU16::new(64).unwrap(),
    });
    for (ground, field) in [
        ("t:cursed_ground", "t:cursed"),
        ("t:plains_ground", "t:plains"),
    ] {
        pack.object_types.push(ObjectTypeDef {
            key: key(ground),
            name: "Ground".into(),
            visit: Visit::Enter,
            from_above: true,
            kind_mark: false,
            kind: ObjectKindDef::Ground {
                battlefield: key(field),
                no_native: false,
            },
        });
    }
    let part = AdventureRuleset::compile(&core, &[pack]).unwrap();
    let battle = battle_part_with(&core, 7, |pack| {
        let plain = pack.battlefields[0].clone();
        pack.battlefields.push(BattlefieldDef {
            key: key("t:cursed"),
            max_spell_level: Some(1),
            ..plain.clone()
        });
        pack.battlefields.push(BattlefieldDef {
            key: key("t:plains"),
            expert_schools: vec![key("t:air")],
            ..plain
        });
    });
    GameRuleset::new(
        Arc::clone(&core),
        Arc::new(part),
        Arc::new(battle),
        Arc::new(TownRuleset::empty(&core)),
    )
    .unwrap()
}

fn spell(rules: &GameRuleset, k: &str) -> SpellId {
    rules.core().spell_by_key(k).unwrap()
}

/// A ground of type `k` lying on `tile`.
fn ground(rules: &GameRuleset, k: &str, tile: Pos) -> ObjectSetup {
    ObjectSetup {
        anchor: tile,
        blocked: TileMask::default(),
        visitable: TileMask::default(),
        object_type: rules.adventure().object_type_by_key(k),
        amount: None,
        drawn: TileMask::from_bits(1),
        monster: None,
        guards: None,
        scenery: Scenery::Other,
        holds: Holds::Nothing,
        owner: None,
    }
}

/// [`setup`]'s game with `objects`, red's hero 0 at (0, 0) knowing `spells` with `mana` spell
/// points.
fn game(rules: &GameRuleset, spells: &[&str], mana: u16, objects: Vec<ObjectSetup>) -> GameState {
    let mut setup = setup();
    setup.heroes[0].details.spells = Some(spells.iter().map(|k| spell(rules, k)).collect());
    setup.map.objects = objects;
    let mut state = GameState::new(rules, &setup, 7).unwrap();
    let points = HeroChange::SetSpellPoints(mana);
    state.change_hero(rules.core(), HERO, None, points, &mut NoopSink);
    state
}

fn cast(rules: &GameRuleset, k: &str, target: Option<Pos>) -> GameCommand {
    GameCommand::Cast {
        hero: HERO,
        spell: spell(rules, k),
        target,
    }
}

fn mana(state: &GameState) -> u16 {
    state.heroes()[0].mana
}

/// [`game`]'s map with the sea (terrain 2) on (1, 0), (1, 1), (0, 1) and (3, 0), a nobody's boat
/// on (3, 0); red's hero 0 on (0, 0) knows the two boat spells (at None: a chance of 50) with 200
/// points, in a game of `seed`. Both players explore the whole 4 × 4 map.
fn sea_game(rules: &GameRuleset, seed: u64) -> GameState {
    let mut setup = setup();
    for (x, y) in [(1, 0), (1, 1), (0, 1), (3, 0)] {
        setup.map.tiles[y * 4 + x].terrain = TerrainId::from_raw(2);
    }
    setup.map.boats = vec![BoatSetup {
        tile: Pos::new(3, 0, 0),
        kind: 0,
    }];
    let spells = ["t:summon", "t:scuttle"].map(|k| spell(rules, k));
    setup.heroes[0].details.spells = Some(spells.to_vec());
    let mut state = GameState::new(rules, &setup, seed).unwrap();
    let points = HeroChange::SetSpellPoints(200);
    state.change_hero(rules.core(), HERO, None, points, &mut NoopSink);
    state
}

/// Red casts Summon Boat until one cast's draw fails and one's passes: each cast takes one draw of
/// 1 to 100 from red's stream against the chance of 50, tells `SpellCast` for 8 points, then
/// `effect`'s events for a pass (given the state before) or `CastFailed` for a failure, then the
/// points; `pass` checks the state after a pass. Every cast spends its points.
fn summon_until_both(
    state: &mut GameState,
    rules: &GameRuleset,
    effect: impl Fn(&GameState) -> Vec<GameEvent>,
    pass: impl Fn(&GameState),
) {
    let spell = spell(rules, "t:summon");
    let (mut passed, mut failed) = (false, false);
    while !(passed && failed) {
        let mut probe = state.stream(RED);
        let fails = probe.range_inclusive(1, 100) > 50;
        let points = mana(state);
        let mut expected = vec![GameEvent::SpellCast {
            player: RED,
            hero: HERO,
            spell,
            mastery: 0,
            cost: 8,
        }];
        if fails {
            expected.push(GameEvent::CastFailed {
                player: RED,
                hero: HERO,
                spell,
                reason: CastFailure::Chance,
            });
        } else {
            expected.extend(effect(state));
        }
        expected.push(GameEvent::HeroChanged {
            player: RED,
            hero: HERO,
            change: HeroChange::SetSpellPoints(points - 8),
        });
        let events = play(state, rules, RED, cast(rules, "t:summon", None)).unwrap();
        assert_eq!(events, expected);
        assert_eq!(
            state.stream(RED),
            probe,
            "one draw from the caster's stream"
        );
        if !fails {
            pass(state);
        }
        (passed, failed) = (passed || !fails, failed || fails);
    }
}

// covers: GAM-SPL-001, ADV-SPL-003
#[test]
fn summon_boat_draws_its_chance_from_the_caster_s_stream_and_a_failure_is_charged() {
    // The nobody's boat comes beside the hero, onto the first free water from up, clockwise:
    // (1, 0), then (1, 1) while it lies on (1, 0); a failed draw is charged (GENRLTXT 337).
    let rules = cast_rules();
    let mut state = sea_game(&rules, 7);
    let both = Viewers::NONE.with(RED).with(BLUE);
    let to = |state: &GameState| {
        if state.adventure().boat(0).unwrap().tile == Pos::new(1, 0, 0) {
            Pos::new(1, 1, 0)
        } else {
            Pos::new(1, 0, 0)
        }
    };
    let effect = |state: &GameState| {
        vec![GameEvent::Adventure(AdventureEvent {
            kind: AdventureEventKind::BoatSummoned {
                boat: 0,
                from: Some(state.adventure().boat(0).unwrap().tile),
                to: Some(to(state)),
                facing: 2,
                seen_from: both,
                seen_to: both,
            },
            seen_by: both,
        })]
    };
    let pass = |state: &GameState| {
        let at = state.adventure().boat(0).unwrap().tile;
        assert!(at == Pos::new(1, 0, 0) || at == Pos::new(1, 1, 0));
        assert!(state.adventure().is_empty_boat(at));
    };
    summon_until_both(&mut state, &rules, effect, pass);
    // Seen by both players as told: a projection keeps the ends each sees.
    let blue = Viewers::NONE.with(BLUE);
    let summoned = effect(&state).remove(0);
    let at = state.adventure().boat(0).unwrap().tile;
    assert_eq!(
        project(&summoned, Viewer::Player(BLUE)),
        Some(GameEvent::Adventure(AdventureEvent {
            kind: AdventureEventKind::BoatSummoned {
                boat: 0,
                from: Some(at),
                to: Some(to(&state)),
                facing: 2,
                seen_from: blue,
                seen_to: blue,
            },
            seen_by: blue,
        }))
    );
}

// covers: GAM-SPL-001, ADV-SPL-004
#[test]
fn scuttle_boat_draws_its_chance_and_a_refusal_draws_nothing() {
    // A target without an empty boat is refused, free, drawing nothing; the boat's tile takes
    // the draw: failed, charged (GENRLTXT 338), the boat kept; passed, the boat sunk, told to
    // both. The game's seed is the first whose red stream fails, then passes, a chance of 50.
    let rules = cast_rules();
    let tile = Pos::new(3, 0, 0);
    let seed = (0..)
        .find(|&seed| {
            let mut draws = sea_game(&rules, seed).stream(RED);
            draws.range_inclusive(1, 100) > 50 && draws.range_inclusive(1, 100) <= 50
        })
        .unwrap();
    let mut state = sea_game(&rules, seed);
    let before = (state.state_hash(), state.stream(RED));
    assert_eq!(
        play(
            &mut state,
            &rules,
            RED,
            cast(&rules, "t:scuttle", Some(Pos::new(1, 0, 0)))
        ),
        Err(GameCommandError::Cast(MapCastError::Adventure(
            CastError::NoBoat
        )))
    );
    assert_eq!((state.state_hash(), state.stream(RED)), before);
    let scuttle = spell(&rules, "t:scuttle");
    let cast_on = |state: &mut GameState| {
        play(state, &rules, RED, cast(&rules, "t:scuttle", Some(tile))).unwrap()
    };
    let told = |middle: GameEvent, left: u16| {
        vec![
            GameEvent::SpellCast {
                player: RED,
                hero: HERO,
                spell: scuttle,
                mastery: 0,
                cost: 8,
            },
            middle,
            GameEvent::HeroChanged {
                player: RED,
                hero: HERO,
                change: HeroChange::SetSpellPoints(left),
            },
        ]
    };
    let failed = GameEvent::CastFailed {
        player: RED,
        hero: HERO,
        spell: scuttle,
        reason: CastFailure::Chance,
    };
    assert_eq!(cast_on(&mut state), told(failed, 192));
    assert!(state.adventure().is_empty_boat(tile));
    // The caster's owner alone is told the failure.
    assert_eq!(project(&failed, Viewer::Player(RED)), Some(failed));
    assert_eq!(project(&failed, Viewer::Player(BLUE)), None);
    let sunk = GameEvent::Adventure(AdventureEvent {
        kind: AdventureEventKind::BoatScuttled { tile, boat: 0 },
        seen_by: Viewers::NONE.with(RED).with(BLUE),
    });
    assert_eq!(cast_on(&mut state), told(sunk, 184));
    assert!(state.adventure().boat(0).unwrap().gone);
    assert!(!state.adventure().is_empty_boat(tile));
    // Blue sees the boat go; a sunk boat is no target.
    let seen = Some(GameEvent::Adventure(AdventureEvent {
        kind: AdventureEventKind::BoatScuttled { tile, boat: 0 },
        seen_by: Viewers::NONE.with(BLUE),
    }));
    assert_eq!(project(&sunk, Viewer::Player(BLUE)), seen);
    assert_eq!(
        play(
            &mut state,
            &rules,
            RED,
            cast(&rules, "t:scuttle", Some(tile))
        ),
        Err(GameCommandError::Cast(MapCastError::Adventure(
            CastError::NoBoat
        )))
    );
}

/// [`game`]'s map with the sea (terrain 2) on (1, 0): red's hero 0 on (0, 0) knows Dimension
/// Door with 60 spell points; both players explore the whole map.
fn door_game(rules: &GameRuleset) -> GameState {
    let mut state = {
        let mut setup = setup();
        setup.map.tiles[1].terrain = TerrainId::from_raw(2);
        setup.heroes[0].details.spells = Some(vec![spell(rules, "t:door")]);
        GameState::new(rules, &setup, 7).unwrap()
    };
    let points = HeroChange::SetSpellPoints(60);
    state.change_hero(rules.core(), HERO, None, points, &mut NoopSink);
    state
}

/// Red's hero 0's tile, movement points, spell points and casts of the day.
fn door_caster(state: &GameState) -> (Pos, u32, u16, u8) {
    let hero = state.adventure().hero(HERO).unwrap();
    (
        hero.pos,
        hero.movement,
        mana(state),
        state.adventure().day_spells(HERO).doors,
    )
}

// covers: GAM-SPL-001, ADV-SPL-007, ADV-SPL-009
#[test]
fn dimension_door_moves_the_hero_for_its_movement_and_spell_points() {
    // At None (no Air Magic): 25 spell points and 300 movement points, the hero on the target,
    // told as a teleporter's arrival before the points; the day's one cast, then refused (339).
    let rules = cast_rules();
    let mut state = door_game(&rules);
    let (_, points, ..) = door_caster(&state);
    let door = spell(&rules, "t:door");
    let target = Pos::new(3, 1, 0);
    let events = play(
        &mut state,
        &rules,
        RED,
        cast(&rules, "t:door", Some(target)),
    )
    .unwrap();
    assert_eq!(
        events[0],
        GameEvent::SpellCast {
            player: RED,
            hero: HERO,
            spell: door,
            mastery: 0,
            cost: 25,
        }
    );
    assert!(matches!(
        events[1],
        GameEvent::Adventure(AdventureEvent {
            kind: AdventureEventKind::HeroTeleported {
                hero: HERO,
                from: Some(Pos { x: 0, y: 0, .. }),
                to: Some(Pos { x: 3, y: 1, .. }),
                ..
            },
            ..
        })
    ));
    assert_eq!(
        events[2],
        GameEvent::HeroChanged {
            player: RED,
            hero: HERO,
            change: HeroChange::SetSpellPoints(35),
        }
    );
    assert_eq!(door_caster(&state), (target, points - 300, 35, 1));
    assert_eq!(
        play(
            &mut state,
            &rules,
            RED,
            cast(&rules, "t:door", Some(Pos::new(0, 0, 0)))
        ),
        Err(GameCommandError::Cast(MapCastError::Adventure(
            CastError::NoCastsLeft
        )))
    );
}

// covers: GAM-SPL-001, ADV-SPL-009
#[test]
fn dimension_door_s_wrong_target_is_refused_free_or_charged_under_adv_quirk_55() {
    // Off (the pack's profile): water from land is not offered, nothing spent. On: the cast is
    // accepted and fails (GENRLTXT 71), told as `CastFailed` after `SpellCast`, the spell
    // points, the movement points and the cast spent, the hero where it was.
    let water = Some(Pos::new(1, 0, 0));
    let rules = cast_rules();
    let mut state = door_game(&rules);
    let before = state.state_hash();
    assert_eq!(
        play(&mut state, &rules, RED, cast(&rules, "t:door", water)),
        Err(GameCommandError::Cast(MapCastError::Adventure(
            CastError::WrongKind
        )))
    );
    assert_eq!(state.state_hash(), before);

    let rules = cast_rules_under("t:charged");
    let mut state = door_game(&rules);
    let (start, points, ..) = door_caster(&state);
    let events = play(&mut state, &rules, RED, cast(&rules, "t:door", water)).unwrap();
    let failed = GameEvent::CastFailed {
        player: RED,
        hero: HERO,
        spell: spell(&rules, "t:door"),
        reason: CastFailure::Target,
    };
    assert_eq!(events[1], failed);
    assert_eq!(door_caster(&state), (start, points - 300, 35, 1));
}

// covers: GAM-SPL-001, ADV-SPL-001, ADV-SPL-005
#[test]
fn a_cast_of_fly_spends_its_cost_after_its_effect_and_tells_its_owner() {
    let rules = cast_rules();
    let mut state = game(&rules, &["t:gate"], 12, vec![]);
    let gate = spell(&rules, "t:gate");
    let events = play(&mut state, &rules, RED, cast(&rules, "t:gate", None)).unwrap();
    // Without Air Magic at None (0): the first column's 5 points.
    let fly = DaySpells {
        fly: Some(0),
        ..DaySpells::NONE
    };
    assert_eq!(
        events,
        vec![
            GameEvent::SpellCast {
                player: RED,
                hero: HERO,
                spell: gate,
                mastery: 0,
                cost: 5,
            },
            GameEvent::HeroChanged {
                player: RED,
                hero: HERO,
                change: HeroChange::SetSpellPoints(7),
            },
            GameEvent::Adventure(AdventureEvent {
                kind: AdventureEventKind::DaySpellsChanged {
                    hero: HERO,
                    spells: fly,
                },
                seen_by: Viewers::NONE.with(RED),
            }),
        ]
    );
    assert_eq!(state.adventure().day_spells(HERO), fly);
    assert_eq!(mana(&state), 7);
    // Seen by its owner alone.
    let blue = Viewer::Player(BLUE);
    assert!(events.iter().all(|e| project(e, blue).is_none()));
    // The next day clears it, told to the owner.
    play(&mut state, &rules, RED, GameCommand::EndTurn).unwrap();
    let events = play(&mut state, &rules, BLUE, GameCommand::EndTurn).unwrap();
    assert!(events.contains(&GameEvent::Adventure(AdventureEvent {
        kind: AdventureEventKind::DaySpellsChanged {
            hero: HERO,
            spells: DaySpells::NONE,
        },
        seen_by: Viewers::NONE.with(RED),
    })));
    assert_eq!(state.adventure().day_spells(HERO), DaySpells::NONE);
}

// covers: GAM-SPL-001, HER-SPL-003, HER-SPL-008
#[test]
fn a_raising_ground_casts_at_expert_for_its_cost() {
    // E15-5: Magic Plains raise air to Expert: Fly at 3 for the last column's 4 points.
    let rules = cast_rules();
    let plains = ground(&rules, "t:plains_ground", Pos::new(0, 0, 0));
    let mut state = game(&rules, &["t:gate"], 4, vec![plains]);
    let events = play(&mut state, &rules, RED, cast(&rules, "t:gate", None)).unwrap();
    assert!(matches!(
        events[0],
        GameEvent::SpellCast {
            mastery: 3,
            cost: 4,
            ..
        }
    ));
    assert_eq!(state.adventure().day_spells(HERO).fly, Some(3));
    assert_eq!(mana(&state), 0);
}

// covers: GAM-SPL-001, ADV-SPL-001
#[test]
fn a_cast_is_refused_in_the_original_s_order_and_changes_nothing() {
    let rules = cast_rules();
    let refused = |error| Err(GameCommandError::Cast(error));
    let gate = spell(&rules, "t:gate");
    let known = ["t:gate", "t:gust", "t:door"];
    let mut state = game(&rules, &known, 30, vec![]);
    // Another player's hero, a hero off the map.
    for hero in [HeroId::new(1), HeroId::new(9)] {
        let command = GameCommand::Cast {
            hero,
            spell: gate,
            target: None,
        };
        assert_eq!(
            play(&mut state, &rules, RED, command),
            refused(MapCastError::NotYourHero(hero))
        );
    }
    // Out of turn.
    assert_eq!(
        play(&mut state, &rules, BLUE, cast(&rules, "t:gate", None)),
        Err(GameCommandError::NotYourTurn(BLUE))
    );
    // A spell the hero lacks, a battle spell, a target where none belongs or none where one
    // does.
    let mut unknown = game(&rules, &["t:gust"], 30, vec![]);
    assert_eq!(
        play(&mut unknown, &rules, RED, cast(&rules, "t:gate", None)),
        refused(MapCastError::Unknown(gate))
    );
    assert_eq!(
        play(&mut state, &rules, RED, cast(&rules, "t:gust", None)),
        refused(MapCastError::NotOnMap(spell(&rules, "t:gust")))
    );
    let tile = Some(Pos::new(1, 1, 0));
    for command in [cast(&rules, "t:gate", tile), cast(&rules, "t:door", None)] {
        assert_eq!(
            play(&mut state, &rules, RED, command),
            refused(MapCastError::Target)
        );
    }
    // The points below the cost; the spell's own test last (Dimension Door onto the caster's own
    // tile).
    let mut poor = game(&rules, &["t:gate"], 4, vec![]);
    assert_eq!(
        play(&mut poor, &rules, RED, cast(&rules, "t:gate", None)),
        refused(MapCastError::SpellPoints)
    );
    let own = Some(Pos::new(0, 0, 0));
    assert_eq!(
        play(&mut state, &rules, RED, cast(&rules, "t:door", own)),
        refused(MapCastError::Adventure(CastError::NotFree))
    );
    // A re-cast at the same mastery (`ADV-QUIRK-54` off in the test profile).
    play(&mut state, &rules, RED, cast(&rules, "t:gate", None)).unwrap();
    assert_eq!(
        play(&mut state, &rules, RED, cast(&rules, "t:gate", None)),
        refused(MapCastError::Adventure(CastError::NotHigher))
    );
}

// covers: GAM-SPL-001, ADV-SPL-001
#[test]
fn cursed_ground_and_an_anti_magic_garrison_refuse_free_after_the_cost() {
    // E15-4: on Cursed Ground a spell above level 1 is refused, nothing spent; in an anti-magic
    // garrison every spell. The cost comes first (decision 414): without the points the cast is
    // refused for them, as the original's book refuses it.
    let rules = cast_rules();
    let cursed = ground(&rules, "t:cursed_ground", Pos::new(0, 0, 0));
    let mut state = game(&rules, &["t:gate"], 30, vec![cursed.clone()]);
    assert_eq!(
        play(&mut state, &rules, RED, cast(&rules, "t:gate", None)),
        Err(GameCommandError::Cast(MapCastError::Ground))
    );
    let mut poor = game(&rules, &["t:gate"], 2, vec![cursed]);
    assert_eq!(
        play(&mut poor, &rules, RED, cast(&rules, "t:gate", None)),
        Err(GameCommandError::Cast(MapCastError::SpellPoints))
    );
    // Red's own anti-magic garrison at (1, 0): stepped into, its exchange closed.
    let entrance = Pos::new(1, 0, 0);
    let garrison = ObjectSetup {
        scenery: Scenery::default(),
        anchor: entrance,
        blocked: TileMask::from_bits(1),
        visitable: TileMask::from_bits(1),
        object_type: rules.adventure().object_type_by_key("t:anti_garrison"),
        amount: None,
        drawn: TileMask::default(),
        monster: None,
        guards: None,
        holds: Holds::Garrison {
            army: vec![],
            removable: true,
        },
        owner: Some(RED),
    };
    let mut state = game(&rules, &["t:gate"], 30, vec![garrison]);
    play(&mut state, &rules, RED, move_hero(0, &[entrance])).unwrap();
    assert_eq!(state.adventure().hero(HERO).unwrap().pos, entrance);
    assert_eq!(
        play(&mut state, &rules, RED, cast(&rules, "t:gate", None)),
        Err(GameCommandError::DecisionOpen)
    );
    play(&mut state, &rules, RED, answered(Answer::Close)).unwrap();
    assert_eq!(
        play(&mut state, &rules, RED, cast(&rules, "t:gate", None)),
        Err(GameCommandError::Cast(MapCastError::AntiMagic))
    );
}

#[test]
fn the_cast_s_errors_explain_themselves() {
    let cases = [
        (
            MapCastError::NotYourHero(HeroId::new(3)),
            "hero 3 is not the player's on the map",
        ),
        (
            MapCastError::Unknown(SpellId::from_raw(2)),
            "the hero cannot cast spell 2",
        ),
        (
            MapCastError::NotOnMap(SpellId::from_raw(1)),
            "spell 1 is not cast on the map",
        ),
        (
            MapCastError::Target,
            "the cast's target does not fit its spell",
        ),
        (
            MapCastError::SpellPoints,
            "the hero's spell points are below the cost",
        ),
        (
            MapCastError::Ground,
            "the ground allows no spell of that level",
        ),
        (
            MapCastError::AntiMagic,
            "no spell is cast in an anti-magic garrison",
        ),
        (
            MapCastError::Adventure(CastError::InBoat),
            "Fly is not cast in a boat",
        ),
        (
            MapCastError::Adventure(CastError::NotYourHero(HeroId::new(4))),
            "hero 4 is not the player's on the map",
        ),
        (
            MapCastError::Adventure(CastError::AlreadyOn),
            "the spell's level is on already",
        ),
        (
            MapCastError::Adventure(CastError::NotHigher),
            "the cast would not raise the hero's level",
        ),
        (
            MapCastError::Adventure(CastError::NotFree),
            "the target is blocked or an entrance",
        ),
    ];
    for (error, text) in cases {
        assert_eq!(error.to_string(), text);
        assert_eq!(GameCommandError::Cast(error).to_string(), text);
    }
}
