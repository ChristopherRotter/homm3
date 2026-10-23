//! Special battlefields (E04c T2b; `docs/formats/installation.md` §7.26): the setup's battlefield
//! and anti-magic garrison (BAT-TER-002), morale and luck (BAT-TER-003), the casting limits
//! (BAT-TER-004), the mastery raise (BAT-TER-005), the native bonus (BAT-TER-006) and the
//! obstacles' types (BAT-GEO-004), on the test ruleset's battlefields (`support`).

use std::string::String;
use std::vec;
use std::vec::Vec;

use h3_content::ContentKey;
use h3_content_battle::BattleRuleset;
use h3_contracts::{BattleHero, BattleSetup, BattleSide};
use h3_kernel::{Rng, StreamId};

use super::spells::{arena_on, cast, mage, spell, stack};
use super::support::*;
use crate::CastSpell::Named;
use crate::casters::stack_cast;
use crate::magic::{mastery, other_mastery};
use crate::{
    BattleState, Command, CommandError, Event, EventKind, SetupError, StackId, StackTraits, Viewer,
    castable_spells, creature_cast_spell, generate_battlefield_obstacles, generate_setup_obstacles,
    observe, observe_as,
};

fn key(name: &str) -> Option<ContentKey> {
    ContentKey::parse(name).ok()
}

/// A battle of `attacker` against `defender` on `field` (none if `None`) and terrain `terrain`.
fn on(
    r: &BattleRuleset,
    field: Option<&str>,
    terrain: Option<&str>,
    attacker: &[Spec],
    defender: &[Spec],
) -> BattleState {
    let setup = BattleSetup {
        battlefield: field.and_then(key),
        terrain: terrain.map(|t| r.terrain_by_key(t).unwrap()),
        ..setup(r, attacker, defender)
    };
    BattleState::new(r, &setup, 1).unwrap()
}

/// The `(morale, luck)` of every stack of `s`.
fn values(s: &BattleState) -> Vec<(i8, i8)> {
    s.stacks().iter().map(|s| (s.morale, s.luck)).collect()
}

/// [`arena_on`] on `field`, with or without an anti-magic garrison.
fn arena(
    r: &BattleRuleset,
    field: Option<&str>,
    anti_magic: bool,
    heroes: [Option<BattleHero>; 2],
    attacker: &[(&'static str, u32, i32, i32)],
    defender: &[(&'static str, u32, i32, i32)],
) -> BattleState {
    arena_on(r, heroes, attacker, defender, 1, |s| {
        s.battlefield = field.and_then(key);
        s.anti_magic = anti_magic;
    })
}

// ---------------------------------------------------------------- BAT-TER-002

// covers: BAT-TER-002
#[test]
fn the_battlefield_is_a_known_key_and_public() {
    let r = ruleset();
    let bad = BattleSetup {
        battlefield: key("t:nowhere"),
        ..setup(&r, &[(0, "fixed", 1)], &[(0, "fixed", 1)])
    };
    assert_eq!(
        BattleState::new(&r, &bad, 1),
        Err(SetupError::UnknownBattlefield)
    );
    let plain = on(&r, None, None, &[(0, "fixed", 1)], &[(0, "fixed", 1)]);
    let setup = BattleSetup {
        battlefield: key("t:holy"),
        anti_magic: true,
        siege: None,
        ..setup(&r, &[(0, "fixed", 1)], &[(0, "fixed", 1)])
    };
    let holy = BattleState::new(&r, &setup, 1).unwrap();
    assert_ne!(plain.state_hash(), holy.state_hash());
    let field = r.battlefield_by_key("t:holy");
    for o in [
        observe(&holy, &r, BattleSide::Attacker),
        observe(&holy, &r, BattleSide::Defender),
        observe_as(&holy, &r, Viewer::Spectator),
    ] {
        assert_eq!((o.battlefield, o.anti_magic), (field, true));
    }
    let o = observe(&plain, &r, BattleSide::Attacker);
    assert_eq!((o.battlefield, o.anti_magic), (None, false));
}

// The adventure's facts for the computer (ADR-0027 point 14, E23 T6): the attacking hero's own
// ground is a known battlefield key, the retreat facts are kept as given; both are hashed and
// neither is observed (no rule reads them).
// covers: BAT-AI-024
#[test]
fn the_adventure_facts_are_kept_hashed_and_not_observed() {
    let r = ruleset();
    let base = setup(&r, &[(0, "fixed", 1)], &[(0, "fixed", 1)]);
    let bad = BattleSetup {
        attacker_ground: key("t:nowhere"),
        ..base.clone()
    };
    assert_eq!(
        BattleState::new(&r, &bad, 1),
        Err(SetupError::UnknownBattlefield)
    );
    let facts = h3_contracts::RetreatFacts {
        experience: 3000,
        artifacts: 10,
        tavern: true,
        wanted: true,
    };
    let known = BattleSetup {
        attacker_ground: key("t:holy"),
        retreat: [h3_contracts::RetreatFacts::default(), facts],
        ..base.clone()
    };
    let plain = BattleState::new(&r, &base, 1).unwrap();
    let s = BattleState::new(&r, &known, 1).unwrap();
    assert_eq!(
        crate::computer::attacker_ground(&s),
        r.battlefield_by_key("t:holy")
    );
    assert_eq!(crate::computer::attacker_ground(&plain), None);
    assert_eq!(
        crate::computer::retreat_facts(&s, BattleSide::Defender),
        facts
    );
    assert_eq!(
        crate::computer::retreat_facts(&s, BattleSide::Attacker),
        h3_contracts::RetreatFacts::default()
    );
    assert_ne!(plain.state_hash(), s.state_hash());
    for side in BattleSide::BOTH {
        assert_eq!(observe(&s, &r, side), observe(&plain, &r, side));
    }
}

// ---------------------------------------------------------------- BAT-TER-003

// covers: BAT-TER-003, BAT-MOR-002
#[test]
fn holy_ground_and_clover_field_change_morale_and_luck_by_faction() {
    let r = ruleset();
    let armies: (&[Spec], &[Spec]) = (
        &[(0, "pikeman_c", 5)],
        &[(0, "imp", 5), (1, "skeleton", 5), (2, "fixed", 5)],
    );
    // One alignment: +1 for the castle army; three and undead for the other: -2, the skeletons
    // none.
    let base = values(&on(&r, None, None, armies.0, armies.1));
    assert_eq!(base, [(1, 0), (-2, 0), (0, 0), (-2, 0)]);
    // Holy Ground: castle +1, inferno -1, the others nothing.
    let holy = values(&on(&r, Some("t:holy"), None, armies.0, armies.1));
    assert_eq!(holy, [(2, 0), (-3, 0), (0, 0), (-2, 0)]);
    // Clover Field: necropolis +2 luck (a creature without morale keeps its luck).
    let clover = values(&on(&r, Some("t:clover"), None, armies.0, armies.1));
    assert_eq!(clover, [(1, 0), (-2, 0), (0, 2), (-2, 0)]);
}

// covers: BAT-TER-003
#[test]
fn cursed_ground_sets_every_morale_and_luck_to_zero() {
    let r = ruleset();
    // Negative morale, a minimum morale and a minimum luck: all 0 on Cursed Ground, as the
    // original's routines return before anything else (0x43E00F, 0x43DF2B).
    let attacker: &[Spec] = &[(0, "pikeman_c", 5), (1, "imp", 5), (2, "minotaur", 5)];
    let defender: &[Spec] = &[(0, "lucky", 5)];
    let plain = on(&r, None, None, attacker, defender);
    assert_eq!(values(&plain), [(-1, 0), (-1, 0), (1, 0), (1, 3)]);
    let cursed = on(&r, Some("t:cursed"), None, attacker, defender);
    assert_eq!(values(&cursed), [(0, 0); 4]);
    // The computer's luck as the record keeps it (E23 T4c, BAT-AI-016) is 0 there too.
    let lucky = StackId(3);
    assert_eq!(crate::computer::luck(&plain, &r, lucky), 3);
    assert_eq!(crate::computer::luck(&cursed, &r, lucky), 0);
}

// ---------------------------------------------------------------- BAT-TER-006

// covers: BAT-TER-006, BAT-TER-001
#[test]
fn no_stack_is_native_on_cursed_ground() {
    let r = ruleset();
    let native = |field| {
        let s = on(
            &r,
            field,
            Some("t:plain"),
            &[(0, "pikeman_c", 5)],
            &[(0, "imp", 5)],
        );
        let p = s.stacks()[0];
        (p.has(StackTraits::NATIVE), p.speed)
    };
    assert_eq!(native(None), (true, 6));
    assert_eq!(native(Some("t:holy")), (true, 6));
    assert_eq!(native(Some("t:cursed")), (false, 5));
}

// ---------------------------------------------------------------- BAT-TER-004

// covers: BAT-TER-004, BAT-MAG-062
#[test]
fn cursed_ground_bars_spells_above_level_one() {
    let r = ruleset();
    let heroes = [Some(mage(&r, &["rise", "bolt"])), None];
    let mut s = arena(
        &r,
        Some("t:cursed"),
        false,
        heroes,
        &[("serf", 10, 5, 5)],
        &[("fixed", 10, 9, 5)],
    );
    let castable = castable_spells(&s, &r, BattleSide::Attacker);
    assert!(castable.contains(spell(&r, "bolt")) && !castable.contains(spell(&r, "rise")));
    let before = s.clone();
    assert_eq!(
        act(&mut s, &r, cast(&r, "rise", stack(0))),
        Err(CommandError::SpellLevelBarred)
    );
    assert_eq!(s, before);
    act(&mut s, &r, cast(&r, "bolt", stack(1))).unwrap();
}

/// E04c T10b review (F-r1-4): the battle's level limit is the lowest of the field's and the
/// heroes' (a Recanter's Cloak's 2), whichever is lower.
// covers: BAT-TER-004, BAT-MAG-062
#[test]
fn the_fields_level_limit_and_a_heros_join_at_the_lower() {
    use h3_contracts::BattleEffect;
    // Cursed Ground's limit 1 and a modded one's 3.
    let modded = ruleset_from(
        "t:faithful",
        |_| {},
        |_, battle| {
            let cursed = battle
                .battlefields
                .iter_mut()
                .find(|b| key("t:cursed").is_some_and(|k| b.key == k))
                .unwrap();
            cursed.max_spell_level = Some(3);
        },
    );
    let limit = |r: &BattleRuleset, cloak: bool| {
        let mut wearer = mage(r, &["bolt", "gloom"]);
        if cloak {
            wearer
                .effects
                .push(BattleEffect::SpellLevelLimit { max_level: 2 });
        }
        let mut s = arena(
            r,
            Some("t:cursed"),
            false,
            [Some(wearer), None],
            &[("serf", 10, 5, 5)],
            &[("fixed", 10, 9, 5)],
        );
        let level = s.limits.max_level;
        // The level-2 Sorrow.
        let gloom = act(&mut s, r, cast(r, "gloom", stack(1))).map(|_| ());
        (level, gloom)
    };
    // The field's 1 below the cloak's 2: the level-2 spell is barred.
    let r = ruleset();
    assert_eq!(limit(&r, true), (1, Err(CommandError::SpellLevelBarred)));
    // The cloak's 2 below the field's 3: it passes; without the cloak the field's 3.
    assert_eq!(limit(&modded, true), (2, Ok(())));
    assert_eq!(limit(&modded, false).0, 3);
}

// covers: BAT-TER-004, BAT-MAG-048, BAT-MAG-049
#[test]
fn cursed_ground_stops_creature_casts_but_not_the_key_path() {
    let r = ruleset();
    let s = arena(
        &r,
        Some("t:cursed"),
        false,
        [None, None],
        &[("conjurer", 5, 2, 3), ("fixed", 3, 2, 7)],
        &[("fixed", 5, 13, 5)],
    );
    assert_eq!(creature_cast_spell(&s, &r, BattleSide::Attacker), None);
    // Elsewhere the conjurer casts.
    let s = arena(
        &r,
        Some("t:holy"),
        false,
        [None, None],
        &[("conjurer", 5, 2, 3), ("fixed", 3, 2, 7)],
        &[("fixed", 5, 13, 5)],
    );
    assert!(creature_cast_spell(&s, &r, BattleSide::Attacker).is_some());
    // A creature that ignores the block (the Faerie Dragon's key path, 0x41F890 not asked).
    let mut s = arena(
        &r,
        Some("t:cursed"),
        false,
        [None, None],
        &[("fixed", 3, 2, 3), ("faerie", 2, 2, 7)],
        &[("fixed", 4, 13, 5)],
    );
    while s.active() != Some(StackId(1)) {
        act(&mut s, &r, Command::Defend).unwrap();
    }
    assert_eq!(
        creature_cast_spell(&s, &r, BattleSide::Attacker),
        Some(Named(spell(&r, "bolt")))
    );
}

// covers: BAT-TER-004, BAT-MAG-053
#[test]
fn cursed_ground_stops_the_periodic_casts() {
    let r = ruleset();
    let periodic = |field: Option<&str>| {
        let setup = BattleSetup {
            battlefield: field.and_then(key),
            ..setup(
                &r,
                &[(0, "chanter", 5), (1, "fixed", 3)],
                &[(0, "fixed", 4)],
            )
        };
        let mut events: Vec<Event> = Vec::new();
        let mut s = BattleState::start(&r, &setup, 3, &mut events).unwrap();
        for _ in 0..6 {
            events.extend(act(&mut s, &r, Command::Defend).unwrap());
        }
        events
            .iter()
            .filter(|e| matches!(e.kind, EventKind::PeriodicCast { .. }))
            .count()
    };
    assert_eq!(periodic(Some("t:cursed")), 0);
    assert!(periodic(None) > 0);
}

// covers: BAT-TER-004, BAT-MAG-061
#[test]
fn an_anti_magic_garrison_forbids_every_cast() {
    let r = ruleset();
    let heroes = [Some(mage(&r, &["bolt"])), Some(mage(&r, &["bolt"]))];
    let mut s = arena(
        &r,
        None,
        true,
        heroes,
        &[("conjurer", 5, 5, 5)],
        &[("fixed", 10, 9, 5)],
    );
    assert!(castable_spells(&s, &r, BattleSide::Attacker).is_empty());
    assert_eq!(
        act(&mut s, &r, cast(&r, "bolt", stack(1))),
        Err(CommandError::CastingForbidden)
    );
    assert_eq!(creature_cast_spell(&s, &r, BattleSide::Attacker), None);
}

// ---------------------------------------------------------------- BAT-TER-005

// covers: BAT-TER-005, BAT-MAG-002
#[test]
fn magic_fields_raise_the_mastery_of_their_schools() {
    let r = ruleset();
    let (bolt, ward) = (spell(&r, "bolt"), spell(&r, "ward"));
    let mut hero = mage(&r, &["bolt", "ward"]);
    hero.magic.masteries = vec![(bolt, 1)];
    let at = |field: Option<&str>| {
        let s = arena(
            &r,
            field,
            false,
            [Some(hero.clone()), None],
            &[("conjurer", 5, 5, 5)],
            &[("fixed", 10, 9, 5)],
        );
        let h = s.heroes[0].unwrap();
        let creature = stack_cast(&s, &r, StackId(0)).unwrap().mastery;
        (
            mastery(&s, &r, &h, bolt),
            mastery(&s, &r, &h, ward),
            other_mastery(&s, 0),
            creature,
        )
    };
    // The bolt is air, the ward fire; the conjurer casts at Advanced.
    assert_eq!(at(None), (1, 0, 0, 2));
    // Magic Plains: every school and the other casts.
    assert_eq!(at(Some("t:plains")), (3, 3, 3, 3));
    // An elemental field: its school, for the hero only.
    assert_eq!(at(Some("t:fiery")), (1, 3, 0, 2));
}

/// The value of the timed effect of `name` on stack `id` of `s`, if it has one.
fn effect_value(s: &BattleState, r: &BattleRuleset, id: u8, name: &str) -> Option<u16> {
    s.effects()
        .iter()
        .find(|e| e.stack == StackId(id) && e.spell == spell(r, name))
        .map(|e| e.value)
}

/// E04c T10b review (F-r1-3): the strike's spells take the raise too.
// covers: BAT-TER-005, BAT-MAG-040
#[test]
fn magic_plains_cast_the_strikes_spells_at_expert() {
    let r = ruleset();
    let weakness = |field: Option<&str>| {
        let mut s = arena(
            &r,
            field,
            false,
            [None, None],
            &[("weakener", 5, 7, 5)],
            &[("fixed", 10, 8, 5)],
        );
        let attack = Command::Attack {
            target: StackId(1),
            from: at(&r, 7, 5),
        };
        act(&mut s, &r, attack).unwrap();
        effect_value(&s, &r, 1, "weaken")
    };
    // The weakener's Weakness is at mastery 0 (3), at Expert 6 on Magic Plains only (the
    // original's 0x59FEBC).
    assert_eq!(
        [None, Some("t:fiery"), Some("t:plains")].map(weakness),
        [Some(3), Some(3), Some(6)]
    );
}

/// The test ruleset with Fortune (luck 1 below Advanced, 2 from it) of the fire school and
/// without a target from Basic, and the test Bloodlust caster (`luster`) casting only it, at
/// Basic: a modded pack's periodic caster and opening spell below Expert.
fn fortune_from_basic() -> BattleRuleset {
    use h3_content_battle::AbilityDef;
    ruleset_from(
        "t:faithful",
        |_| {},
        |pack, battle| {
            let fortune = key("t:fortune").unwrap();
            let def = pack.spells.iter_mut().find(|s| s.key == fortune).unwrap();
            def.schools = vec![key("t:fire").unwrap()];
            let def = battle.spells.iter_mut().find(|s| s.key == fortune).unwrap();
            def.magic.mass_from = Some(1);
            let luster = key("t:luster").unwrap();
            let def = battle
                .creatures
                .iter_mut()
                .find(|c| c.key == luster)
                .unwrap();
            def.abilities
                .retain(|a| !matches!(a, AbilityDef::DrawnSpell { .. }));
            for a in &mut def.abilities {
                if let AbilityDef::PeriodicSpellCaster { mastery, .. } = a {
                    *mastery = 1;
                }
            }
            def.abilities.push(AbilityDef::DrawnSpell {
                spell: fortune,
                weight: 1,
            });
        },
    )
}

/// E04c T10b review (F-r1-3): the periodic casts take the raise too.
// covers: BAT-TER-005, BAT-MAG-053
#[test]
fn magic_plains_raise_the_periodic_casts_to_expert() {
    let r = fortune_from_basic();
    let fortune = |field: Option<&str>| {
        let setup = BattleSetup {
            battlefield: field.and_then(key),
            ..setup(&r, &[(0, "fixed", 5), (1, "luster", 3)], &[(0, "fixed", 4)])
        };
        let mut events: Vec<Event> = Vec::new();
        let s = BattleState::start(&r, &setup, 2, &mut events).unwrap();
        assert!(
            events
                .iter()
                .any(|e| matches!(e.kind, EventKind::PeriodicCast { .. }))
        );
        [0, 1].map(|id| effect_value(&s, &r, id, "fortune"))
    };
    // Basic Fortune's 1 on both own stacks, Expert's 2 on Magic Plains only (a fire field
    // raises only the hero's own casts).
    assert_eq!(fortune(None), [Some(1); 2]);
    assert_eq!(fortune(Some("t:fiery")), [Some(1); 2]);
    assert_eq!(fortune(Some("t:plains")), [Some(2); 2]);
}

/// E04c T10b review (F-r1-1): the opening spells are cast at Expert on Magic Plains, as every
/// cast that is not the hero's own (the original's 0x59FE98: caster kind 2 takes the argument,
/// Expert on field 1).
// covers: BAT-TER-005, BAT-MAG-068
#[test]
fn magic_plains_cast_the_opening_spells_at_expert() {
    use h3_contracts::BattleEffect;
    let r = fortune_from_basic();
    let fortune = |field: Option<&str>| {
        let mut opener = hero(&r);
        opener.effects = vec![BattleEffect::OpeningSpell {
            spell: spell(&r, "fortune"),
            mastery: 1,
            power: 10,
        }];
        let setup = BattleSetup {
            heroes: [Some(opener), None],
            battlefield: field.and_then(key),
            ..setup(&r, &[(0, "fast", 10)], &[(0, "fixed", 10)])
        };
        let s = BattleState::start(&r, &setup, 1, &mut Vec::<Event>::new()).unwrap();
        effect_value(&s, &r, 0, "fortune")
    };
    assert_eq!(
        [None, Some("t:fiery"), Some("t:plains")].map(fortune),
        [Some(1), Some(1), Some(2)]
    );
}

// ---------------------------------------------------------------- BAT-GEO-004

// covers: BAT-GEO-004
#[test]
fn battlefields_choose_the_obstacle_types() {
    let r = ruleset();
    let plain = r.terrain_by_key("t:plain").unwrap();
    let kinds = |field: Option<&str>| {
        let id = field.and_then(|f| r.battlefield_by_key(f));
        let mut rng = Rng::for_stream(5, StreamId::OBSTACLES);
        let pieces = generate_battlefield_obstacles(&r, plain, id, &mut rng);
        assert!(!pieces.is_empty());
        pieces
            .into_iter()
            .map(|p| p.kind.map(|k| String::from(k.as_str())))
            .collect::<Vec<_>>()
    };
    // The shore has its own: only the reef, never the plain's stone.
    assert!(
        kinds(Some("t:shore"))
            .iter()
            .all(|k| k.as_deref() == Some("t:reef"))
    );
    // The deck and a field without its own: the terrain's.
    for field in [None, Some("t:deck"), Some("t:holy")] {
        assert!(kinds(field).iter().all(|k| k.as_deref() == Some("t:stone")));
    }
    // Two boats: their hexes, one absolute obstacle without a type, nothing else.
    let boats = r.battlefield_by_key("t:boats");
    let mut rng = Rng::for_stream(5, StreamId::OBSTACLES);
    let pieces = generate_battlefield_obstacles(&r, plain, boats, &mut rng);
    assert_eq!(pieces.len(), 1);
    assert_eq!(
        (
            pieces[0].hexes.clone(),
            pieces[0].anchor,
            pieces[0].kind.clone()
        ),
        (vec![26, 27, 43, 44], None, None)
    );
    let setup = BattleSetup {
        obstacles: pieces,
        battlefield: key("t:boats"),
        ..setup(&r, &[(0, "fixed", 1)], &[(0, "fixed", 1)])
    };
    assert!(BattleState::new(&r, &setup, 1).is_ok());
}

// covers: BAT-GEO-004
#[test]
fn a_setups_obstacles_follow_its_terrain_and_battlefield() {
    let r = ruleset();
    let draw = |terrain: Option<&str>, field: Option<&str>| {
        let setup = BattleSetup {
            terrain: terrain.map(|t| r.terrain_by_key(t).unwrap()),
            battlefield: field.and_then(key),
            ..setup(&r, &[(0, "fixed", 1)], &[(0, "fixed", 1)])
        };
        let mut rng = Rng::for_stream(9, StreamId::OBSTACLES);
        generate_setup_obstacles(&r, &setup, &mut rng)
    };
    let mut rng = Rng::for_stream(9, StreamId::OBSTACLES);
    let plain = r.terrain_by_key("t:plain").unwrap();
    let shore = r.battlefield_by_key("t:shore");
    assert_eq!(
        draw(Some("t:plain"), Some("t:shore")),
        generate_battlefield_obstacles(&r, plain, shore, &mut rng)
    );
    // An unknown battlefield counts as none (the setup refuses it); no terrain, no obstacles.
    let mut rng = Rng::for_stream(9, StreamId::OBSTACLES);
    assert_eq!(
        draw(Some("t:plain"), Some("t:nowhere")),
        generate_battlefield_obstacles(&r, plain, None, &mut rng)
    );
    assert!(draw(None, Some("t:shore")).is_empty());
}
