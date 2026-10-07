//! Dispel (BAT-MAG-016) and Cure (BAT-MAG-016, BAT-MAG-017), E04b T3b1b: the effects they end,
//! their targets by mastery, Dispel's indifference to immunity and resistance, Cure's heal.

use std::vec::Vec;

use h3_content::Ruleset;
use h3_contracts::BattleHero;

use super::spells::{arena, cast, mage, spell, stack};
use super::support::*;
use crate::effects::TimedCast;
use crate::magic::affect_chance;
use crate::{
    BattleState, CastTarget, CastTargets, CommandError, Event, EventKind, StackId, cast_targets,
    castable_spells,
};

/// A mage with Dispel, Cure and Haste, the air mastery `mastery` and enough mana.
fn adept(r: &Ruleset, mastery: u8) -> BattleHero {
    let mut h = mage(r, &["dispel", "cure", "haste"]);
    h.magic.schools[0] = mastery;
    h.magic.mana = 100;
    h
}

/// Gives stack `id` the timed effect of `name` for five rounds.
fn give(s: &mut BattleState, r: &Ruleset, id: u8, name: &str) {
    let id_of = spell(r, name);
    let cast = TimedCast {
        id: id_of,
        spell: r.spell(id_of).unwrap(),
        mastery: 1,
        rounds: 5,
    };
    crate::effects::apply(s, r, cast, StackId(id), &mut Vec::<Event>::new());
}

/// The names of the spells whose effects stack `id` has, by spell.
fn on(s: &BattleState, r: &Ruleset, id: u8) -> Vec<std::string::String> {
    s.effects()
        .iter()
        .filter(|e| e.stack == StackId(id))
        .map(|e| r.spell(e.spell).unwrap().key.as_str()[2..].into())
        .collect()
}

/// The ended effects (`(stack, spell name)`) and the heals (`(stack, hit points)`) of events.
type EndsAndHeals = (Vec<(u8, std::string::String)>, Vec<(u8, u32)>);

/// The ended effects (`(stack, spell name)`) and the heals (`(stack, hit points)`) in `events`.
fn ends_and_heals(r: &Ruleset, events: &[Event]) -> EndsAndHeals {
    let mut ends = Vec::new();
    let mut heals = Vec::new();
    for e in events {
        match e.kind {
            EventKind::EffectEnded { target, spell } => {
                ends.push((target.0, r.spell(spell).unwrap().key.as_str()[2..].into()));
            }
            EventKind::SpellHealed { target, hit_points } => heals.push((target.0, hit_points)),
            _ => {}
        }
    }
    (ends, heals)
}

/// The stack targets of `name` for the attacker now.
fn targets(s: &BattleState, r: &Ruleset, name: &str) -> Vec<CastTarget> {
    let mut out = CastTargets::new();
    cast_targets(
        s,
        r,
        h3_contracts::BattleSide::Attacker,
        spell(r, name),
        &mut out,
    );
    out.as_slice().to_vec()
}

// covers: BAT-MAG-016
#[test]
fn dispel_ends_every_effect_but_the_spared_and_permanent_ones() {
    let r = ruleset();
    let mut s = arena(
        &r,
        [Some(adept(&r, 1)), None],
        &[("fixed", 1, 3, 5), ("fixed", 1, 3, 7)],
        &[("fixed", 1, 13, 5)],
        1,
    );
    // Below Expert only a stack with an effect is a target, and below Advanced only an own one.
    assert!(
        !castable_spells(&s, &r, h3_contracts::BattleSide::Attacker).contains(spell(&r, "dispel"))
    );
    for name in ["skin", "sloth", "ray", "bless"] {
        give(&mut s, &r, 0, name);
    }
    give(&mut s, &r, 2, "bless");
    assert_eq!(targets(&s, &r, "dispel"), [stack(0)]);
    assert_eq!(
        act(&mut s, &r, cast(&r, "dispel", stack(2))),
        Err(CommandError::InvalidSpellTarget)
    );
    let defense = s.stacks()[0].defense_now();
    let events = act(&mut s, &r, cast(&r, "dispel", stack(0))).unwrap();
    // Stone Skin and Bless end; Slow is spared, Disrupting Ray is permanent.
    let (ends, heals) = ends_and_heals(&r, &events);
    assert_eq!(ends, [(0, "skin".into()), (0, "bless".into())]);
    assert!(heals.is_empty());
    assert_eq!(on(&s, &r, 0), ["sloth", "ray"]);
    assert!(
        s.stacks()[0].defense_now() < defense,
        "Stone Skin's defense is given back"
    );
    // A stack whose only effects stay is still a target; the cast ends nothing.
    activate(&mut s, 0);
    s.heroes[0].as_mut().unwrap().view.casts = 0;
    let events = act(&mut s, &r, cast(&r, "dispel", stack(0))).unwrap();
    assert_eq!(ends_and_heals(&r, &events).0, []);
}

// covers: BAT-MAG-016, BAT-MAG-005, BAT-MAG-008
#[test]
fn dispel_reaches_the_enemy_from_advanced_and_every_stack_at_expert_without_immunity() {
    let r = ruleset();
    let mut s = arena(
        &r,
        [Some(adept(&r, 2)), None],
        &[("fixed", 1, 3, 5), ("wyrm", 1, 3, 7)],
        &[
            ("dwarf", 1, 13, 5),
            ("fixed", 1, 13, 7),
            ("unicorn", 1, 13, 9),
        ],
        1,
    );
    for id in [1, 2, 4] {
        give(&mut s, &r, id, "bless");
    }
    // Advanced: any stack with an effect, the spell-immune wyrm included; the dwarf (magic
    // resistance) next to a unicorn (aura) is always affected.
    assert_eq!(targets(&s, &r, "dispel"), [stack(1), stack(2), stack(4)]);
    let dispel = r.spell(spell(&r, "dispel")).unwrap();
    assert_eq!(affect_chance(&s, &r, dispel, StackId(2)), 100);
    let events = act(&mut s, &r, cast(&r, "dispel", stack(2))).unwrap();
    assert_eq!(ends_and_heals(&r, &events).0, [(2, "bless".into())]);
    assert!(
        !events
            .iter()
            .any(|e| matches!(e.kind, EventKind::SpellResisted { .. }))
    );
    // Expert: every stack of both sides, no target.
    s.heroes[0].as_mut().unwrap().magic.schools[0] = 3;
    s.heroes[0].as_mut().unwrap().view.casts = 0;
    activate(&mut s, 0);
    give(&mut s, &r, 3, "sloth");
    assert_eq!(targets(&s, &r, "dispel"), [CastTarget::None]);
    let events = act(&mut s, &r, cast(&r, "dispel", CastTarget::None)).unwrap();
    assert_eq!(
        ends_and_heals(&r, &events).0,
        [(1, "bless".into()), (4, "bless".into())],
        "the dispel spares Slow"
    );
    // At Expert it is castable without any effect on the field.
    s.heroes[0].as_mut().unwrap().view.casts = 0;
    activate(&mut s, 0);
    let mut empty = arena(
        &r,
        [Some(adept(&r, 3)), None],
        &[("fixed", 1, 3, 5)],
        &[("fixed", 1, 13, 5)],
        1,
    );
    assert!(
        castable_spells(&empty, &r, h3_contracts::BattleSide::Attacker)
            .contains(spell(&r, "dispel"))
    );
    empty.heroes[0].as_mut().unwrap().magic.schools[0] = 2;
    assert!(
        !castable_spells(&empty, &r, h3_contracts::BattleSide::Attacker)
            .contains(spell(&r, "dispel"))
    );
}

// covers: BAT-MAG-016, BAT-MAG-017
#[test]
fn cure_ends_its_spells_then_heals_the_top_creature_without_raising() {
    let r = ruleset();
    let mut s = arena(
        &r,
        [Some(adept(&r, 1)), None],
        &[("tank", 3, 3, 5), ("tank", 3, 3, 7), ("wyrm", 1, 3, 9)],
        &[("tank", 3, 13, 5)],
        1,
    );
    for name in ["sloth", "curse", "skin", "ray"] {
        give(&mut s, &r, 0, name);
    }
    s.stacks.as_mut_slice()[0].top_hp = 50;
    s.stacks.as_mut_slice()[3].top_hp = 50;
    // Own stacks only, healthy ones too; the wyrm is immune to a level 1 spell.
    assert_eq!(targets(&s, &r, "cure"), [stack(0), stack(1)]);
    // Power 3: 3 × 5 + 10 = 25 hit points; Curse and Slow end first.
    let events = act(&mut s, &r, cast(&r, "cure", stack(0))).unwrap();
    let (ends, heals) = ends_and_heals(&r, &events);
    assert_eq!(ends, [(0, "sloth".into()), (0, "curse".into())]);
    assert_eq!(heals, [(0, 25)]);
    let first = events
        .iter()
        .position(|e| matches!(e.kind, EventKind::EffectEnded { .. }));
    let heal = events
        .iter()
        .position(|e| matches!(e.kind, EventKind::SpellHealed { .. }));
    assert!(first < heal);
    assert_eq!(on(&s, &r, 0), ["ray", "skin"]);
    assert_eq!((s.stacks()[0].count, s.stacks()[0].top_hp), (3, 75));
    // At most up to the full hit points; a healthy stack is healed by nothing, silently.
    s.stacks.as_mut_slice()[0].top_hp = 90;
    s.heroes[0].as_mut().unwrap().view.casts = 0;
    activate(&mut s, 0);
    let events = act(&mut s, &r, cast(&r, "cure", stack(0))).unwrap();
    assert_eq!(ends_and_heals(&r, &events).1, [(0, 10)]);
    assert_eq!((s.stacks()[0].count, s.stacks()[0].top_hp), (3, 100));
    s.heroes[0].as_mut().unwrap().view.casts = 0;
    activate(&mut s, 0);
    let events = act(&mut s, &r, cast(&r, "cure", stack(1))).unwrap();
    assert_eq!(ends_and_heals(&r, &events), (Vec::new(), Vec::new()));
    // Expert: every own stack it can affect, by 3 × 5 + 30; not the enemy.
    s.heroes[0].as_mut().unwrap().magic.schools[0] = 3;
    s.heroes[0].as_mut().unwrap().view.casts = 0;
    activate(&mut s, 0);
    s.stacks.as_mut_slice()[0].top_hp = 1;
    s.stacks.as_mut_slice()[1].top_hp = 80;
    let events = act(&mut s, &r, cast(&r, "cure", CastTarget::None)).unwrap();
    assert_eq!(ends_and_heals(&r, &events).1, [(0, 45), (1, 20)]);
    assert_eq!(s.stacks()[3].top_hp, 50);
}
