//! The casting limits (E04b T8b1): a hero's `NoCasting` forbids every cast of both sides
//! (BAT-MAG-061, the Orb of Inhibition), its `SpellLevelLimit` every spell above the level
//! (BAT-MAG-062, Recanter's Cloak), after the original's code (`docs/formats/installation.md`
//! §7.23).

use std::vec;
use std::vec::Vec;

use h3_content_battle::BattleRuleset;
use h3_contracts::{BattleEffect, BattleHero, BattleSetup, BattleSide};

use super::spells::{arena, cast, mage, spell, stack};
use super::support::*;
use crate::CastSpell::{Named, Raise};
use crate::damage::apply_damage;
use crate::effects;
use crate::raise::mark_corpses;
use crate::{
    BattleState, CastTarget, CastTargets, Command, CommandError, Event, EventKind, SpellSet,
    StackId, Status, castable_spells, creature_cast_spell, creature_cast_targets, stack_spell,
};

const ORB: BattleEffect = BattleEffect::NoCasting;
const CLOAK: BattleEffect = BattleEffect::SpellLevelLimit { max_level: 2 };

/// A hero without spells whose effects are `effects`.
fn wearer(r: &BattleRuleset, effects: &[BattleEffect]) -> Option<BattleHero> {
    let mut h = hero(r);
    h.effects = effects.to_vec();
    Some(h)
}

/// The spells of `names` as a set.
fn set(r: &BattleRuleset, names: &[&str]) -> SpellSet {
    let mut out = SpellSet::default();
    for name in names {
        out.insert(spell(r, name));
    }
    out
}

/// The stacks the active stack of `side` may cast at or raise now, by id or hex index.
fn targets(s: &BattleState, r: &BattleRuleset, side: BattleSide) -> Vec<u8> {
    let mut out = CastTargets::new();
    creature_cast_targets(s, r, side, &mut out);
    out.iter()
        .map(|t| match *t {
            CastTarget::Stack(id) => id.0,
            CastTarget::Hex(h) => h.0,
            CastTarget::None => u8::MAX,
        })
        .collect()
}

/// Deals `damage` to stack `id` as a strike would, and marks the corpses.
fn hurt(s: &mut BattleState, r: &BattleRuleset, id: u8, damage: u64) {
    let mut events = Vec::new();
    apply_damage(s.at_mut(StackId(id)), damage);
    effects::damaged(s, r, StackId(id), &mut events);
    mark_corpses(s);
}

/// The periodic casts among `events`.
fn periodic(events: &[Event]) -> usize {
    events
        .iter()
        .filter(|e| matches!(e.kind, EventKind::PeriodicCast { .. }))
        .count()
}

/// The active stack attacks `target` from `(x, y)`.
fn hit(s: &mut BattleState, r: &BattleRuleset, target: u8, (x, y): (i32, i32)) -> Vec<Event> {
    act(
        s,
        r,
        Command::Attack {
            target: StackId(target),
            from: at(r, x, y),
        },
    )
    .unwrap()
}

// ---------------------------------------------------------------- BAT-MAG-061

// covers: BAT-MAG-061
#[test]
fn a_hero_without_casting_forbids_both_heroes_spells() {
    let r = ruleset();
    for wearer_side in [0, 1] {
        let mut heroes = [Some(mage(&r, &["bolt"])), Some(mage(&r, &["bolt"]))];
        heroes[wearer_side].as_mut().unwrap().effects.push(ORB);
        let mut s = arena(
            &r,
            heroes,
            &[("fixed", 10, 5, 5)],
            &[("fixed", 10, 9, 5)],
            1,
        );
        assert!(castable_spells(&s, &r, BattleSide::Attacker).is_empty());
        let before = s.clone();
        assert_eq!(
            act(&mut s, &r, cast(&r, "bolt", stack(1))),
            Err(CommandError::CastingForbidden)
        );
        assert_eq!(s, before);
        activate(&mut s, 1);
        assert!(castable_spells(&s, &r, BattleSide::Defender).is_empty());
        assert_eq!(
            act(&mut s, &r, cast(&r, "bolt", stack(0))),
            Err(CommandError::CastingForbidden)
        );
    }
    // Without it both may cast.
    let mut s = arena(
        &r,
        [Some(mage(&r, &["bolt"])), Some(mage(&r, &["bolt"]))],
        &[("fixed", 10, 5, 5)],
        &[("fixed", 10, 9, 5)],
        1,
    );
    assert_eq!(
        castable_spells(&s, &r, BattleSide::Attacker),
        set(&r, &["bolt"])
    );
    act(&mut s, &r, cast(&r, "bolt", stack(1))).unwrap();
}

// covers: BAT-MAG-061, BAT-MAG-048, BAT-MAG-051
#[test]
fn a_hero_without_casting_forbids_both_sides_creature_casts_and_raises() {
    let r = ruleset();
    // The enemy's hero: its own side's casters may not cast either.
    for heroes in [[None, wearer(&r, &[ORB])], [wearer(&r, &[ORB]), None]] {
        let mut s = arena(
            &r,
            heroes.clone(),
            &[("conjurer", 5, 2, 3), ("fixed", 3, 2, 7)],
            &[("warlock", 5, 13, 5)],
            1,
        );
        // The stack keeps its spell; it may not cast it now.
        assert_eq!(
            stack_spell(&s, &r, StackId(0)),
            Some(Named(spell(&r, "haste")))
        );
        assert_eq!(creature_cast_spell(&s, &r, BattleSide::Attacker), None);
        assert!(targets(&s, &r, BattleSide::Attacker).is_empty());
        let before = s.clone();
        assert_eq!(
            act(&mut s, &r, Command::CreatureCast { target: stack(1) }),
            Err(CommandError::CannotCast)
        );
        assert_eq!(s, before);
        activate(&mut s, 2);
        assert_eq!(creature_cast_spell(&s, &r, BattleSide::Defender), None);
        // An Archangel raises nothing.
        let mut s = arena(
            &r,
            heroes,
            &[("herald", 2, 4, 5), ("serf", 10, 8, 3)],
            &[("fixed", 3, 13, 5)],
            1,
        );
        hurt(&mut s, &r, 1, 30);
        assert_eq!(creature_cast_spell(&s, &r, BattleSide::Attacker), None);
        assert_eq!(
            act(&mut s, &r, Command::CreatureCast { target: stack(1) }),
            Err(CommandError::CannotCast)
        );
    }
    // Without it the Archangel raises the serfs.
    let mut s = arena(
        &r,
        [None, None],
        &[("herald", 2, 4, 5), ("serf", 10, 8, 3)],
        &[("fixed", 3, 13, 5)],
        1,
    );
    hurt(&mut s, &r, 1, 30);
    assert_eq!(
        creature_cast_spell(&s, &r, BattleSide::Attacker),
        Some(Raise)
    );
    assert_eq!(targets(&s, &r, BattleSide::Attacker), vec![1]);
}

// covers: BAT-MAG-061, BAT-MAG-049
#[test]
fn a_random_caster_still_draws_but_may_not_cast_unless_it_ignores_the_block() {
    // The original's draw at the activation (0x447510) does not ask whether magic is allowed;
    // the cast's test (0x4476C0) does.
    let r = ruleset();
    let mut s = arena(
        &r,
        [None, wearer(&r, &[ORB])],
        &[("fixed", 3, 2, 3), ("boltling", 2, 2, 7)],
        &[("fixed", 4, 13, 5)],
        1,
    );
    let mut events = Vec::new();
    while s.active() != Some(StackId(1)) {
        events.extend(act(&mut s, &r, Command::Defend).unwrap());
    }
    assert!(
        events
            .iter()
            .any(|e| matches!(e.kind, EventKind::SpellDrawn { stack, .. } if stack == StackId(1)))
    );
    assert_eq!(
        stack_spell(&s, &r, StackId(1)),
        Some(Named(spell(&r, "bolt")))
    );
    assert_eq!(creature_cast_spell(&s, &r, BattleSide::Attacker), None);
    // A creature that ignores the block casts (the Faerie Dragon after the wiki and VCMI; the code
    // reading refuses it, BAT-Q21).
    let mut s = arena(
        &r,
        [None, wearer(&r, &[ORB])],
        &[("fixed", 3, 2, 3), ("faerie", 2, 2, 7)],
        &[("fixed", 4, 13, 5)],
        1,
    );
    while s.active() != Some(StackId(1)) {
        act(&mut s, &r, Command::Defend).unwrap();
    }
    assert_eq!(
        creature_cast_spell(&s, &r, BattleSide::Attacker),
        Some(Named(spell(&r, "bolt")))
    );
    assert_eq!(targets(&s, &r, BattleSide::Attacker), [2]);
    act(&mut s, &r, Command::CreatureCast { target: stack(2) }).unwrap();
}

// covers: BAT-MAG-061, BAT-MAG-053
#[test]
fn a_hero_without_casting_stops_the_periodic_casts_and_their_count_stays() {
    let r = ruleset();
    let start = |heroes: [Option<BattleHero>; 2]| {
        let setup = BattleSetup {
            heroes,
            ..setup(
                &r,
                &[(0, "chanter", 5), (1, "fixed", 3)],
                &[(0, "fixed", 4)],
            )
        };
        let mut events = Vec::new();
        let mut s = BattleState::start(&r, &setup, 3, &mut events).unwrap();
        for _ in 0..6 {
            events.extend(act(&mut s, &r, Command::Defend).unwrap());
        }
        (s, events)
    };
    let (s, events) = start([None, wearer(&r, &[ORB])]);
    assert_eq!(periodic(&events), 0);
    assert_eq!(s.periodic_rounds(BattleSide::Attacker), u8::MAX);
    // Without it the Enchanters cast at their first activation.
    let (_, events) = start([None, None]);
    assert!(periodic(&events) > 0);
}

// covers: BAT-MAG-061, BAT-MAG-043
#[test]
fn the_strikes_creature_magic_is_no_cast_and_stays() {
    let r = ruleset();
    let mut s = arena(
        &r,
        [wearer(&r, &[ORB]), None],
        &[("gazer", 5, 7, 5)],
        &[("fixed", 10, 8, 5)],
        1,
    );
    hit(&mut s, &r, 1, (7, 5));
    assert!(s.at(StackId(1)).spells.status.contains(Status::PETRIFIED));
}

// ---------------------------------------------------------------- BAT-MAG-062

// covers: BAT-MAG-062
#[test]
fn a_spell_level_limit_bars_both_heroes_spells_above_it() {
    let r = ruleset();
    for wearer_side in [0, 1] {
        let mut heroes = [
            Some(mage(&r, &["rise", "bolt", "animate"])),
            Some(mage(&r, &["rise"])),
        ];
        heroes[wearer_side].as_mut().unwrap().effects.push(CLOAK);
        let mut s = arena(&r, heroes, &[("serf", 10, 5, 5)], &[("fixed", 10, 9, 5)], 1);
        hurt(&mut s, &r, 0, 30);
        // Resurrection (4) and Animate Dead (3) are barred, the bolt (1) is not.
        assert_eq!(
            castable_spells(&s, &r, BattleSide::Attacker),
            set(&r, &["bolt"])
        );
        let before = s.clone();
        assert_eq!(
            act(&mut s, &r, cast(&r, "rise", stack(0))),
            Err(CommandError::SpellLevelBarred)
        );
        assert_eq!(s, before);
        act(&mut s, &r, cast(&r, "bolt", stack(1))).unwrap();
    }
    // Without it the hurt serfs can be resurrected.
    let mut s = arena(
        &r,
        [Some(mage(&r, &["rise"])), None],
        &[("serf", 10, 5, 5)],
        &[("fixed", 10, 9, 5)],
        1,
    );
    hurt(&mut s, &r, 0, 30);
    assert_eq!(
        castable_spells(&s, &r, BattleSide::Attacker),
        set(&r, &["rise"])
    );
}

// covers: BAT-MAG-062, BAT-MAG-040, BAT-MAG-043
#[test]
fn a_spell_level_limit_bars_the_strikes_spells_above_it() {
    let r = ruleset();
    // Stone Gaze (level 3) is barred, Disease (level 2) is not.
    let mut s = arena(
        &r,
        [None, wearer(&r, &[CLOAK])],
        &[("gazer", 5, 7, 5), ("rotter", 5, 7, 3)],
        &[("serf", 10, 8, 5), ("serf", 10, 8, 3)],
        1,
    );
    let events = hit(&mut s, &r, 2, (7, 5));
    assert!(
        !events
            .iter()
            .any(|e| matches!(e.kind, EventKind::StrikeSpell { .. }))
    );
    assert!(!s.at(StackId(2)).spells.status.contains(Status::PETRIFIED));
    activate(&mut s, 1);
    let events = hit(&mut s, &r, 3, (7, 3));
    let plague = spell(&r, "plague");
    assert!(
        events
            .iter()
            .any(|e| matches!(e.kind, EventKind::StrikeSpell { spell: p, .. } if p == plague))
    );
}

// covers: BAT-MAG-062, BAT-MAG-048, BAT-MAG-049, BAT-MAG-051, BAT-MAG-052, BAT-MAG-053
#[test]
fn a_spell_level_limit_bars_creature_casts_and_the_archangels_raise() {
    let r = ruleset();
    let cloak = || wearer(&r, &[CLOAK]);
    // An Archangel's raise tests as Resurrection (level 4): no target; a Pit Lord's has no
    // spell test and raises.
    let mut s = arena(
        &r,
        [cloak(), None],
        &[("herald", 2, 4, 5), ("serf", 10, 8, 3)],
        &[("fixed", 3, 13, 5)],
        1,
    );
    hurt(&mut s, &r, 1, 30);
    assert_eq!(
        creature_cast_spell(&s, &r, BattleSide::Attacker),
        Some(Raise)
    );
    assert!(targets(&s, &r, BattleSide::Attacker).is_empty());
    assert_eq!(
        act(&mut s, &r, Command::CreatureCast { target: stack(1) }),
        Err(CommandError::InvalidSpellTarget)
    );
    let mut s = arena(
        &r,
        [None, cloak()],
        &[("fiend", 2, 5, 5), ("serf", 8, 8, 3)],
        &[("fixed", 3, 13, 5)],
        1,
    );
    hurt(&mut s, &r, 1, 1000);
    assert_eq!(targets(&s, &r, BattleSide::Attacker), vec![1]);
    // A limit below the casters' spells (level 1) bars their casts: no target for a caster, a
    // random caster or the Enchanters' periodic cast.
    let none = || wearer(&r, &[BattleEffect::SpellLevelLimit { max_level: 0 }]);
    let s = arena(
        &r,
        [None, none()],
        &[("conjurer", 5, 2, 3), ("fixed", 3, 2, 7)],
        &[("fixed", 5, 13, 5)],
        1,
    );
    assert!(creature_cast_spell(&s, &r, BattleSide::Attacker).is_some());
    assert!(targets(&s, &r, BattleSide::Attacker).is_empty());
    let mut s = arena(
        &r,
        [none(), None],
        &[("fixed", 3, 2, 3), ("novaling", 2, 2, 7)],
        &[("fixed", 4, 13, 5)],
        1,
    );
    while s.active() != Some(StackId(1)) {
        act(&mut s, &r, Command::Defend).unwrap();
    }
    assert!(creature_cast_spell(&s, &r, BattleSide::Attacker).is_some());
    assert!(targets(&s, &r, BattleSide::Attacker).is_empty());
    let setup = BattleSetup {
        heroes: [none(), None],
        ..setup(
            &r,
            &[(0, "chanter", 5), (1, "fixed", 3)],
            &[(0, "fixed", 4)],
        )
    };
    let mut events = Vec::new();
    let s = BattleState::start(&r, &setup, 3, &mut events).unwrap();
    assert_eq!(periodic(&events), 0);
    assert_eq!(s.periodic_rounds(BattleSide::Attacker), u8::MAX);
}
