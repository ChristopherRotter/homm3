//! Tests of the cast on the adventure map through the game (E08 T142a; increment 15's design note
//! §14, E08 decision 414; GAM-SPL-001, ADV-SPL-001): the game's checks in the original's order,
//! each refusal changing nothing, the cost at the mastery on the hero's ground spent after the
//! adventure's effect, and the events. Batch E15's rows as sources: E15-4 (Cursed Ground and the
//! anti-magic garrison refuse free), E15-5 (Magic Plains casts at Expert), E15-11 (a re-cast).

use h3_adventure::{
    AdventureEvent, AdventureEventKind, AdventureSpell, AdventureSpellDef, CastError, DaySpells,
    Holds, ObjectSetup, Scenery, SpellParams, TileMask, Viewers,
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

/// The core of [`core_pack`] with a level-5 air spell of the adventure category, `t:door`.
fn cast_core() -> Ruleset {
    let mut pack = core_pack(false);
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
    Ruleset::compile(&[pack], "t:faithful").unwrap()
}

/// The test rules with the casts' content: `t:gate` (an adventure spell of air, level 3, 5 or 4
/// points) acts as Fly and `t:door` as Dimension Door; two grounds, `t:cursed_ground` (no spell
/// above level 1) and `t:plains_ground` (air at Expert).
fn cast_rules() -> GameRuleset {
    let core = Arc::new(cast_core());
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
    // The points below the cost; the spell's own test last (Dimension Door not built yet,
    // `ADV-STUB-22`).
    let mut poor = game(&rules, &["t:gate"], 4, vec![]);
    assert_eq!(
        play(&mut poor, &rules, RED, cast(&rules, "t:gate", None)),
        refused(MapCastError::SpellPoints)
    );
    assert_eq!(
        play(&mut state, &rules, RED, cast(&rules, "t:door", tile)),
        refused(MapCastError::Adventure(CastError::NotBuilt(
            AdventureSpell::DimensionDoor
        )))
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
            MapCastError::Adventure(CastError::NotBuilt(AdventureSpell::SummonBoat)),
            "`summon_boat` is not cast on the map yet",
        ),
    ];
    for (error, text) in cases {
        assert_eq!(error.to_string(), text);
        assert_eq!(GameCommandError::Cast(error).to_string(), text);
    }
}
