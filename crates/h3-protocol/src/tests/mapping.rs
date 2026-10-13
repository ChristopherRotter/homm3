//! The mapping: every engine variant has its wire variant of the same name (the mapped kinds),
//! commands round-trip, and what leaves for a viewer is projected.

use std::string::String;
use std::vec;
use std::vec::Vec;

use h3_battle::{
    ActiveEffect, BattleResult, BattleSide, BattleState, CastTarget, CommandError,
    DamageRoll as EngineRoll, Event as EngineEvent, EventKind as K, Hex, HexSet, NoopSink, Opening,
    SpellObstacle as EngineObstacle, SpellSet, StackFlags, StackId, StackTraits, StrikeFlags,
    TacticsPhase, Visibility,
};
use h3_content::{CreatureTypeId, SpellId, TerrainId};
use h3_content_battle::ObstacleKind as EngineKind;
use h3_contracts::{BallistaControl, Obstacle, Rate, TentControl};

use super::{battle, full};
use crate::map;

/// The name of an enum value: its debug text up to the first character that cannot be part of a
/// name (`Move { to: Hex(3) }` is `Move`).
fn kind_name<T: core::fmt::Debug>(value: &T) -> String {
    let mut text = std::format!("{value:?}");
    let end = text
        .find(|c: char| !(c.is_alphanumeric() || c == '_'))
        .unwrap_or(text.len());
    text.truncate(end);
    text
}
use crate::wire::*;

fn hex_set(hexes: &[u8]) -> HexSet {
    let mut set = HexSet::new();
    for &h in hexes {
        set.insert(Hex(h));
    }
    set
}

/// One of every engine command.
fn engine_commands() -> Vec<h3_battle::Command> {
    use h3_battle::Command as C;
    vec![
        C::Move { to: Hex(3) },
        C::Attack {
            target: StackId(9),
            from: Hex(20),
        },
        C::Shoot {
            target: StackId(9),
            at: Hex(21),
        },
        C::Wait,
        C::Defend,
        C::Heal { target: StackId(2) },
        C::EndTactics,
        C::Retreat,
        C::Surrender,
        C::Cast {
            spell: SpellId::from_raw(4),
            target: CastTarget::Stack(StackId(8)),
            second: CastTarget::Hex(Hex(40)),
        },
        C::CreatureCast {
            target: CastTarget::None,
        },
    ]
}

/// One of every engine event kind (the mapped kinds: a kind missing here is not checked).
fn engine_events() -> Vec<K> {
    let (s, t) = (StackId(1), StackId(9));
    let spell = SpellId::from_raw(4);
    let creature = CreatureTypeId::from_raw(2);
    let hexes = hex_set(&[10, 11]);
    vec![
        K::TacticsStarted {
            side: BattleSide::Attacker,
            columns: 3,
        },
        K::TacticsEnded,
        K::RoundStarted { round: 2 },
        K::TurnStarted { stack: s },
        K::Moved {
            stack: s,
            from: Hex(1),
            to: Hex(2),
        },
        K::Waited { stack: s },
        K::Defended { stack: s },
        K::Struck {
            attacker: s,
            target: t,
            damage: 1 << 40,
            kills: 3,
            flags: StrikeFlags::LUCKY.with(StrikeFlags::RANGED),
        },
        K::Reflected {
            stack: s,
            damage: 5,
            kills: 1,
        },
        K::LifeDrained {
            stack: s,
            hit_points: 7,
            resurrected: 1,
        },
        K::DeathStared {
            attacker: s,
            target: t,
            kills: 2,
        },
        K::Regenerated {
            stack: s,
            hit_points: 9,
        },
        K::Healed {
            healer: s,
            target: t,
            hit_points: 4,
        },
        K::GoodMorale { stack: s },
        K::BadMorale { stack: s },
        K::Frightened { stack: s },
        K::TurnPassed { stack: s },
        K::SpellCast {
            side: BattleSide::Defender,
            spell,
            target: CastTarget::Stack(t),
            second: CastTarget::None,
        },
        K::StackCast {
            stack: s,
            spell,
            target: CastTarget::Hex(Hex(5)),
        },
        K::PeriodicCast { stack: s, spell },
        K::OpeningSpell {
            side: BattleSide::Defender,
            spell,
        },
        K::ManaChanneled {
            side: BattleSide::Attacker,
            mana: 3,
        },
        K::ManaDrained {
            stack: s,
            side: BattleSide::Defender,
            mana: 2,
        },
        K::CreatureRaise {
            stack: s,
            target: t,
        },
        K::RoseFromCorpse {
            stack: s,
            corpse: t,
            creature,
            count: 4,
            hex: Hex(6),
        },
        K::SpellDrawn { stack: s, spell },
        K::SpellResisted { target: t },
        K::SpellRedirected { from: s, to: t },
        K::StrikeSpell {
            striker: s,
            target: t,
            spell,
        },
        K::StrikeDamage {
            striker: s,
            target: t,
            damage: 8,
            kills: 1,
        },
        K::Poisoned {
            stack: s,
            hit_points: 3,
        },
        K::SpellDamage {
            target: t,
            value: 30,
            caster: 2,
            damage: 60,
            kills: 5,
        },
        K::SpellHealed {
            target: t,
            hit_points: 10,
        },
        K::EffectApplied {
            target: t,
            spell,
            rounds: Some(3),
        },
        K::EffectEnded { target: t, spell },
        K::ObstaclePlaced {
            side: BattleSide::Attacker,
            spell,
            hexes,
        },
        K::ObstacleRemoved { spell, hexes },
        K::NaturalObstacleRemoved {
            anchor: Hex(10),
            hexes,
        },
        K::ObstacleRevealed {
            side: BattleSide::Defender,
            spell,
            hexes,
        },
        K::ObstacleTriggered {
            stack: s,
            spell,
            hex: Hex(10),
        },
        K::SpellRaised {
            target: t,
            hit_points: 50,
            creatures: 2,
            temporary: true,
        },
        K::Sacrificed {
            victim: t,
            creatures: 6,
        },
        K::Reborn {
            stack: s,
            creatures: 1,
        },
        K::StackCreated {
            stack: StackId(20),
            creature,
            count: 12,
            hex: Hex(30),
            original: Some(s),
        },
        K::TurnedSummoned {
            stack: s,
            rounds: 2,
        },
        K::Vanished {
            stack: s,
            creatures: 3,
        },
        K::Teleported {
            stack: s,
            from: Hex(1),
            to: Hex(40),
        },
        K::GateOpened,
        K::GateClosed,
        K::MoatDamage {
            stack: s,
            hex: Hex(44),
            damage: 70,
            kills: 1,
        },
        K::BattleEnded {
            result: BattleResult::Retreat(BattleSide::Attacker),
        },
    ]
}

const ENGINE_ERRORS: [CommandError; 29] = {
    use CommandError as E;
    [
        E::BattleOver,
        E::NotYourTurn,
        E::AlreadyWaited,
        E::InvalidHex,
        E::Unreachable,
        E::NoSuchTarget,
        E::NotAnEnemy,
        E::NotAdjacent,
        E::MustShoot,
        E::CannotShoot,
        E::NotOnTarget,
        E::NoMelee,
        E::CannotHeal,
        E::NotAPatient,
        E::InTacticsPhase,
        E::NoTacticsPhase,
        E::OutsideZone,
        E::CannotRetreat,
        E::CannotSurrender,
        E::CannotAffordSurrender,
        E::NotInSpellBook,
        E::NotACombatSpell,
        E::AlreadyCast,
        E::NotCastable,
        E::NotEnoughMana,
        E::InvalidSpellTarget,
        E::CannotCast,
        E::CastingForbidden,
        E::SpellLevelBarred,
    ]
};

#[test]
fn every_engine_kind_has_its_wire_kind_of_the_same_name() {
    let events = engine_events();
    assert_eq!(events.len(), 48, "the mapped event kinds");
    for kind in &events {
        let wire = map::events(&[EngineEvent::public(*kind)], BattleSide::Attacker);
        assert_eq!(kind_name(&wire[0]), kind_name(kind));
    }
    for command in engine_commands() {
        assert_eq!(kind_name(&map::command(command)), kind_name(&command));
    }
    for error in ENGINE_ERRORS {
        assert_eq!(kind_name(&map::reject(error)), kind_name(&error));
    }
    for kind in [
        EngineKind::Wall,
        EngineKind::Fire,
        EngineKind::Pit,
        EngineKind::Mine,
    ] {
        assert_eq!(
            kind_name(&map::from_engine::obstacle_kind(kind)),
            kind_name(&kind)
        );
    }
    let rolls = [
        EngineRoll::Random,
        EngineRoll::Maximum(20),
        EngineRoll::Minimum(20),
    ];
    for roll in rolls {
        assert_eq!(
            kind_name(&map::from_engine::damage_roll(roll)),
            kind_name(&roll)
        );
    }
}

#[test]
fn the_fields_survive_the_mapping() {
    let wire = map::events(
        &[EngineEvent::public(engine_events()[7])],
        BattleSide::Defender,
    );
    assert_eq!(
        wire,
        vec![Event::Struck {
            attacker: 1,
            target: 9,
            damage: WireU64(1 << 40),
            kills: 3,
            flags: Strike {
                lucky: true,
                ranged: true,
                ..Strike::default()
            },
        }]
    );
    let every_flag = [
        StrikeFlags::RETALIATION,
        StrikeFlags::RANGED,
        StrikeFlags::SECONDARY,
        StrikeFlags::DEATH_BLOW,
        StrikeFlags::LUCKY,
        StrikeFlags::DOUBLE_DAMAGE,
    ]
    .into_iter()
    .fold(StrikeFlags::NONE, StrikeFlags::with);
    let s = map::strike(every_flag);
    assert!(s.retaliation && s.ranged && s.secondary && s.death_blow && s.lucky && s.double_damage);
    assert_eq!(map::hexes(hex_set(&[11, 10])), vec![10, 11]);
    for (result, outcome) in [
        (
            BattleResult::Victory(BattleSide::Attacker),
            Outcome::Victory {
                side: Side::Attacker,
            },
        ),
        (BattleResult::Draw, Outcome::Draw {}),
        (
            BattleResult::Retreat(BattleSide::Defender),
            Outcome::Retreat {
                side: Side::Defender,
            },
        ),
        (
            BattleResult::Surrender(BattleSide::Attacker),
            Outcome::Surrender {
                side: Side::Attacker,
            },
        ),
    ] {
        assert_eq!(map::outcome(result), outcome);
    }
    for side in [BattleSide::Attacker, BattleSide::Defender] {
        assert_eq!(map::side_to_engine(map::side(side)), side);
    }
}

#[test]
fn commands_round_trip_between_the_wire_and_the_engine() {
    for command in engine_commands() {
        assert_eq!(map::command_to_engine(map::command(command)), command);
    }
    for target in [
        CastTarget::None,
        CastTarget::Stack(StackId(3)),
        CastTarget::Hex(Hex(7)),
    ] {
        assert_eq!(map::target_to_engine(map::target(target)), target);
    }
}

#[test]
fn kind_names_are_the_variant_names() {
    assert_eq!(
        kind_name(&K::TurnStarted { stack: StackId(1) }),
        "TurnStarted"
    );
    assert_eq!(kind_name(&K::TacticsEnded), "TacticsEnded");
    assert_eq!(kind_name(&CommandError::NotYourTurn), "NotYourTurn");
}

#[test]
fn events_are_projected_for_their_viewer() {
    let hidden = K::ObstaclePlaced {
        side: BattleSide::Defender,
        spell: SpellId::from_raw(4),
        hexes: hex_set(&[50]),
    };
    let events = [
        EngineEvent {
            visibility: Visibility::Side(BattleSide::Defender),
            kind: hidden,
        },
        EngineEvent::public(K::Waited { stack: StackId(1) }),
    ];
    assert_eq!(
        map::events(&events, BattleSide::Attacker),
        vec![Event::Waited { stack: 1 }]
    );
    assert_eq!(map::events(&events, BattleSide::Defender).len(), 2);
}

#[test]
fn the_observation_carries_the_viewers_battle() {
    let (ruleset, mut setup) = full();
    let key = |k: &str| h3_content::ContentKey::parse(k).ok();
    setup.obstacles.push(Obstacle {
        hexes: vec![60, 61],
        anchor: Some(60),
        kind: key("synthetic:boulder"),
    });
    // An absolute obstacle's type (v22), here over no hex.
    setup.obstacles.push(Obstacle {
        hexes: vec![],
        anchor: None,
        kind: key("synthetic:ridge"),
    });
    let state = BattleState::new(&ruleset, &setup, 7).unwrap();
    let engine = h3_battle::observe(&state, &ruleset, BattleSide::Defender);
    let o = map::observe(&state, &ruleset, BattleSide::Defender);
    assert_eq!(o.viewer, Side::Defender);
    assert_eq!(o.round, engine.round);
    assert_eq!(o.active, engine.active.map(|a| a.0));
    assert_eq!(o.stacks.len(), engine.stacks.len());
    for (i, (wire, s)) in o.stacks.iter().zip(engine.stacks.iter()).enumerate() {
        assert_eq!(usize::from(wire.id), i);
        assert_eq!(
            (wire.count, wire.hex, wire.creature),
            (s.count, s.hex.0, s.creature.raw())
        );
        assert_eq!(wire.side, map::side(s.side));
    }
    assert!(
        o.sides.iter().all(|s| s.hero.is_some()),
        "both sides are led"
    );
    assert!(
        o.own_hero
            .as_ref()
            .is_some_and(|h| !h.spell_book.is_empty())
    );
    assert!(o.obstacles.contains(&60) && o.obstacles.contains(&61));
    assert_eq!(
        o.removable_obstacles.len(),
        engine.removable_obstacles.len()
    );
    assert!(!o.removable_obstacles.is_empty());
    // The obstacles' types, by handle (v22).
    let handle = |k: &str| ruleset.obstacle_by_key(k).map(h3_kernel::Handle::raw);
    assert!(
        o.removable_obstacles
            .iter()
            .any(|r| r.anchor == 60 && r.kind == handle("synthetic:boulder"))
    );
    assert_eq!(o.absolute_obstacle, handle("synthetic:ridge"));
    assert!(o.absolute_obstacle.is_some());
    // The attacker sees its own hero's private part, never the defender's.
    let a = map::observe(&state, &ruleset, BattleSide::Attacker);
    assert_ne!(a.own_hero, o.own_hero);
}

#[test]
fn every_part_of_an_observation_is_mapped() {
    let (ruleset, state) = battle();
    let mut o = h3_battle::observe(&state, &ruleset, BattleSide::Attacker);
    let spell = SpellId::from_raw(4);
    o.tactics = Some(TacticsPhase {
        side: BattleSide::Attacker,
        columns: 3,
    });
    o.result = Some(BattleResult::Draw);
    o.drawn = Some(spell);
    o.terrain = Some(TerrainId::from_raw(1));
    o.wait_phase = true;
    o.summoned[1] = Some(CreatureTypeId::from_raw(7));
    o.cost_change = [-2, 1];
    o.obstacles_cleared = hex_set(&[40]);
    o.clone_links[0] = Some(StackId(1));
    o.temporary[0] = 5;
    o.corpses[1] = 2;
    o.binders[0] = (1 << 1) | (1 << 40);
    o.poison[0] = 3;
    o.start_hp[0] = 25;
    let s0 = &mut o.stacks.as_mut_slice()[0];
    s0.flags.set(StackFlags::STARTED, true);
    s0.traits = s0
        .traits
        .with(StackTraits::NATIVE, true)
        .with(StackTraits::RAISED, true);
    o.effects.push(ActiveEffect {
        stack: StackId(0),
        spell,
        rounds: None,
        value: 3,
        mastery: 2,
        attack: 4,
        defense: -1,
        speed: 2,
    });
    o.spell_obstacles.push(EngineObstacle {
        spell,
        kind: EngineKind::Pit,
        side: BattleSide::Attacker,
        hexes: hex_set(&[33]),
        rounds: 2,
        value: 1 << 50,
        revealed: false,
    });
    let own = o.own_hero.as_mut().unwrap();
    own.ballista = Some(BallistaControl {
        shots: 2,
        double_damage_chance_bp: 500,
    });
    own.tent = Some(TentControl {
        heal_bonus: Rate::bp(2500),
    });
    let mut set = SpellSet::default();
    set.insert(spell);
    own.spell_bonuses.all = Rate::bp(1000);
    own.spell_bonuses.school[2] = Rate::bp(500);
    own.spell_bonuses
        .spells
        .push(h3_battle::SpellBonus {
            spell,
            damage_bp: 300,
            per_level_bp: 3,
            level: 10,
        })
        .unwrap();
    own.magic_resistance_bp = 2000;
    own.duration_bonus = 2;
    own.no_casting = true;
    own.level_limit = Some(3);
    own.defences = h3_battle::Defences {
        spells: set,
        up_to_level: Some(2),
        mind: true,
    };
    own.negates = true;
    // Brea has an opening spell of her own since the battle lane's E04b T9b (the full scenario's
    // casting workload): only this one.
    own.opening.clear();
    own.opening
        .push(Opening {
            spell,
            mastery: 3,
            power: 5,
        })
        .unwrap();
    own.opening_due = true;
    own.eagle_chance = 40;
    own.eagle_spells = set;
    own.learned = set;
    let w = map::observation(&o);
    assert!(w.wait_phase);
    assert_eq!(w.obstacles_cleared, vec![40]);
    assert_eq!(
        w.sides.map(|s| (s.summoned, s.cost_change)),
        [(None, -2), (Some(7), 1)]
    );
    let (engine, w0) = (&o.stacks.as_slice()[0], &w.stacks[0]);
    assert_eq!(
        (w0.base_count, w0.retaliations),
        (engine.base_count, engine.retaliations)
    );
    assert!(w0.started && w0.native && w0.raised);
    assert!(!w.stacks[1].started && !w.stacks[1].native && !w.stacks[1].raised);
    assert_eq!(
        (w0.clone_link, w0.temporary, w0.poison, w0.start_hp),
        (Some(1), 5, 3, 25)
    );
    assert_eq!(w0.bound_by, vec![1, 40]);
    assert_eq!((w0.corpse, w.stacks[1].corpse), (0, 2));
    assert_eq!(w.stacks[1].clone_link, None);
    assert!(w.stacks[1].bound_by.is_empty());
    assert_eq!(
        w.tactics,
        Some(Tactics {
            side: Side::Attacker,
            columns: 3
        })
    );
    assert_eq!(w.result, Some(Outcome::Draw {}));
    assert_eq!((w.drawn, w.terrain), (Some(4), Some(1)));
    assert_eq!(
        w.stacks[0].effects,
        vec![Effect {
            spell: 4,
            rounds: None,
            mastery: 2,
            value: 3,
            attack: 4,
            defense: -1,
            speed: 2,
        }]
    );
    assert!(w.stacks[1].effects.is_empty());
    assert_eq!(w.stacks[0].damage_roll, DamageRoll::Random {});
    assert_eq!(
        w.spell_obstacles,
        vec![SpellObstacle {
            spell: 4,
            kind: ObstacleKind::Pit {},
            side: Side::Attacker,
            hexes: vec![33],
            rounds: 2,
            value: WireU64(1 << 50),
            revealed: false,
        }]
    );
    let own = w.own_hero.unwrap();
    assert_eq!(
        own.ballista,
        Some(Ballista {
            shots: 2,
            double_damage_chance_bp: 500
        })
    );
    assert_eq!(own.tent_heal_bonus_bp, Some(2500));
    assert_eq!(own.spell_damage_bp, 1000);
    assert_eq!(own.school_damage_bp, vec![0, 0, 500, 0]);
    assert_eq!(
        own.spell_bonuses.last(),
        Some(&SpellBonus {
            spell: 4,
            damage_bp: 300,
            per_level_bp: 3,
            level: 10,
        })
    );
    assert_eq!(
        (own.magic_resistance_bp, own.duration_bonus, own.level_limit),
        (2000, 2, Some(3))
    );
    assert!(own.no_casting && own.negates && own.opening_due);
    assert_eq!(
        own.defences,
        Defences {
            spells: vec![4],
            up_to_level: Some(2),
            mind: true,
        }
    );
    assert_eq!(
        own.opening,
        vec![OpeningSpell {
            spell: 4,
            mastery: 3,
            power: 5,
        }]
    );
    assert_eq!(own.eagle_chance, 40);
    assert_eq!((own.eagle_spells, own.learned), (vec![4], vec![4]));
}

#[test]
fn the_legal_list_castable_spells_and_targets_come_from_the_rules() {
    let (ruleset, mut state) = battle();
    // Past the tactics phase: heroes cast only in the rounds.
    while let Some(phase) = h3_battle::observe(&state, &ruleset, BattleSide::Attacker).tactics {
        let end = h3_battle::Command::EndTactics;
        h3_battle::apply(&mut state, &ruleset, phase.side, end, &mut NoopSink).unwrap();
    }
    let actor = state.side_to_act().unwrap();
    let legal = map::legal_commands(&state, &ruleset, actor);
    assert!(!legal.is_empty());
    for command in &legal {
        let engine = map::command_to_engine(*command);
        assert_eq!(h3_battle::validate(&state, &ruleset, actor, engine), Ok(()));
    }
    let castable = map::castable(&state, &ruleset, actor);
    assert!(!castable.is_empty(), "the full scenario's heroes cast");
    let targets: Vec<Vec<Target>> = castable
        .iter()
        .map(|&spell| map::query(&state, &ruleset, actor, Query::CastTargets { spell }))
        .collect();
    assert!(targets.iter().any(|t| !t.is_empty()));
    // A second target only for Teleport and Sacrifice: none for the first spell's first target.
    let first = targets.iter().find(|t| !t.is_empty()).unwrap()[0];
    let second = map::query(
        &state,
        &ruleset,
        actor,
        Query::SecondTargets {
            spell: castable[0],
            target: first,
        },
    );
    assert!(second.len() < 1000);
}

#[test]
fn the_key_tables_follow_the_handles() {
    let (ruleset, _) = full();
    let keys = map::key_tables(&ruleset);
    assert_eq!(keys.creatures.len(), ruleset.creature_count());
    for (i, key) in keys.creatures.iter().enumerate() {
        assert_eq!(ruleset.creature_by_key(key).map(|c| c.index()), Some(i));
    }
    for (i, key) in keys.spells.iter().enumerate() {
        assert_eq!(ruleset.spell_by_key(key).map(|s| s.index()), Some(i));
    }
    let kinds = [
        &keys.skills,
        &keys.hero_types,
        &keys.artifacts,
        &keys.factions,
        &keys.terrains,
    ];
    assert!(kinds.iter().all(|k| !k.is_empty()));
    assert_eq!(keys.hero_types.len(), ruleset.hero_types().len());
    assert!(!keys.obstacles.is_empty());
    for (i, key) in keys.obstacles.iter().enumerate() {
        assert_eq!(ruleset.obstacle_by_key(key).map(|o| o.index()), Some(i));
    }
    let field = map::field(&ruleset);
    let params = ruleset.battle().field;
    assert_eq!((field.width, field.height), (params.width, params.height));
}
