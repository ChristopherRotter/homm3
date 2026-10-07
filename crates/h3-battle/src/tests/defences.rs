//! The spell defences (E04b T4a): Anti-Magic (BAT-MAG-021) and Magic Mirror (BAT-MAG-022), after
//! the original's code (`docs/formats/installation.md` §7.13).

use std::string::String;
use std::vec;
use std::vec::Vec;

use h3_content::Ruleset;
use h3_contracts::{BattleEffect, BattleHero, BattleSide, Rate};

use super::spells::{arena, cast, mage, spell, stack};
use super::support::*;
use crate::effects::TimedCast;
use crate::magic::kept_off;
use crate::state::Stack;
use crate::{
    BattleState, CastTarget, CastTargets, CommandError, Event, EventKind, StackId, cast_targets,
};

/// A mage with `spells`, the air mastery `mastery` and enough mana.
fn adept(r: &Ruleset, spells: &[&str], mastery: u8) -> BattleHero {
    let mut h = mage(r, spells);
    h.magic.schools[0] = mastery;
    h.magic.mana = 200;
    h
}

/// Gives stack `id` the timed effect of `name` at `mastery` for five rounds.
fn give(s: &mut BattleState, r: &Ruleset, (id, name): (u8, &str), mastery: u8) {
    let spell_id = spell(r, name);
    let cast = TimedCast {
        id: spell_id,
        spell: r.spell(spell_id).unwrap(),
        mastery,
        rounds: 5,
        own: crate::magic::SpellBonus::default(),
    };
    crate::effects::apply(s, r, cast, StackId(id), &mut Vec::<Event>::new());
}

/// The names of the spells whose effects stack `id` has, sorted.
fn on(s: &BattleState, r: &Ruleset, id: u8) -> Vec<String> {
    let mut names: Vec<String> = s
        .effects()
        .iter()
        .filter(|e| e.stack == StackId(id))
        .map(|e| r.spell(e.spell).unwrap().key.as_str()[2..].into())
        .collect();
    names.sort();
    names
}

/// Starts a new round and gives the turn to stack `id`.
fn next_round(s: &mut BattleState, r: &Ruleset, id: u8) {
    crate::turn::start_round(s, r, &mut Vec::<Event>::new());
    activate(s, id);
}

/// The stack targets of `name` for `side` now.
fn targets(s: &BattleState, r: &Ruleset, side: BattleSide, name: &str) -> Vec<CastTarget> {
    let mut out = CastTargets::new();
    cast_targets(s, r, side, spell(r, name), &mut out);
    out.as_slice().to_vec()
}

/// What `events` report: redirections `(from, to)`, spell damage by target, resisted stacks and
/// effects applied, by target.
type Report = (Vec<(u8, u8)>, Vec<u8>, Vec<u8>, Vec<u8>);

/// See [`Report`].
fn report(events: &[Event]) -> Report {
    let mut out: Report = Default::default();
    for e in events {
        match e.kind {
            EventKind::SpellRedirected { from, to } => out.0.push((from.0, to.0)),
            EventKind::SpellDamage { target, .. } => out.1.push(target.0),
            EventKind::SpellResisted { target } => out.2.push(target.0),
            EventKind::EffectApplied { target, .. } => out.3.push(target.0),
            _ => {}
        }
    }
    out
}

// covers: BAT-MAG-021
#[test]
fn anti_magic_ends_the_negative_spells_below_its_level_and_keeps_spells_off() {
    let r = ruleset();
    let own = ["antimagic", "bless", "cure", "dispel"];
    let enemy = ["bolt", "curse", "nova", "storm"];
    let mut s = arena(
        &r,
        [Some(adept(&r, &own, 2)), Some(adept(&r, &enemy, 0))],
        &[("tank", 5, 3, 5), ("tank", 5, 4, 5)],
        &[("tank", 5, 13, 5)],
        1,
    );
    for name in ["curse", "sloth", "ray", "gloom", "skin"] {
        give(&mut s, &r, (0, name), 1);
    }
    // At Advanced it keeps spells below level 2 off the stack and ends the negative ones it
    // keeps off: Curse and Slow. Disrupting Ray never ends, Gloom is level 2, Stone Skin
    // positive.
    let events = act(&mut s, &r, cast(&r, "antimagic", stack(0))).unwrap();
    let ended: Vec<String> = events
        .iter()
        .filter_map(|e| match e.kind {
            EventKind::EffectEnded {
                target: StackId(0),
                spell,
            } => Some(r.spell(spell).unwrap().key.as_str()[2..].into()),
            _ => None,
        })
        .collect();
    assert_eq!(ended.len(), 2);
    assert!(ended.contains(&"curse".into()) && ended.contains(&"sloth".into()));
    assert_eq!(on(&s, &r, 0), ["antimagic", "gloom", "ray", "skin"]);
    assert_eq!(s.stacks()[0].spells.spell_immunity, 2);
    // Its own side's spells are kept off too, a second Anti-Magic included; Dispel is not.
    next_round(&mut s, &r, 0);
    for name in ["bless", "cure", "antimagic"] {
        assert_eq!(
            targets(&s, &r, BattleSide::Attacker, name),
            [stack(1)],
            "{name}"
        );
    }
    assert_eq!(
        act(&mut s, &r, cast(&r, "cure", stack(0))),
        Err(CommandError::InvalidSpellTarget)
    );
    // The enemy's: a stack spell may not be aimed at it, an area or battle-wide spell passes
    // over it.
    activate(&mut s, 2);
    assert_eq!(
        targets(&s, &r, BattleSide::Defender, "bolt"),
        [stack(1)],
        "bolt"
    );
    assert_eq!(
        targets(&s, &r, BattleSide::Defender, "curse"),
        [stack(1)],
        "curse"
    );
    let hex = s.stacks()[0].hex;
    let events = act(&mut s, &r, cast(&r, "nova", CastTarget::Hex(hex))).unwrap();
    assert_eq!(report(&events).1, [1]);
    next_round(&mut s, &r, 2);
    let events = act(&mut s, &r, cast(&r, "storm", CastTarget::None)).unwrap();
    assert_eq!(report(&events).1, [1, 2]);
    // Dispel reaches it and ends it; then the stack takes spells again.
    activate(&mut s, 0);
    act(&mut s, &r, cast(&r, "dispel", stack(0))).unwrap();
    assert_eq!(s.stacks()[0].spells.spell_immunity, 0);
    next_round(&mut s, &r, 0);
    assert_eq!(
        targets(&s, &r, BattleSide::Attacker, "bless"),
        [stack(0), stack(1)]
    );
}

// covers: BAT-MAG-021
#[test]
fn a_basic_anti_magic_keeps_nothing_of_level_1_off() {
    let r = ruleset();
    let mut s = arena(
        &r,
        [Some(adept(&r, &["antimagic", "bless"], 1)), None],
        &[("tank", 5, 3, 5)],
        &[("tank", 5, 13, 5)],
        1,
    );
    give(&mut s, &r, (0, "curse"), 1);
    act(&mut s, &r, cast(&r, "antimagic", stack(0))).unwrap();
    assert_eq!(on(&s, &r, 0), ["antimagic", "curse"]);
    assert_eq!(s.stacks()[0].spells.spell_immunity, 1);
    // A second cast keeps the first value.
    next_round(&mut s, &r, 0);
    act(&mut s, &r, cast(&r, "antimagic", stack(0))).unwrap();
    assert_eq!(s.stacks()[0].spells.spell_immunity, 1);
    next_round(&mut s, &r, 0);
    act(&mut s, &r, cast(&r, "bless", stack(0))).unwrap();
}

// covers: BAT-MAG-021
#[test]
fn anti_magic_never_keeps_a_creature_ability_off() {
    let r = ruleset();
    let mut stack = Stack::default();
    stack.spells.spell_immunity = 6;
    let of = |name: &str| r.spell(spell(&r, name)).unwrap();
    assert!(kept_off(&stack, of("bolt")));
    assert!(!kept_off(&stack, of("gaze")), "a creature ability");
    assert!(!kept_off(&stack, of("dispel")), "Dispel ignores it");
    stack.spells.spell_immunity = 1;
    assert!(!kept_off(&stack, of("bolt")));
}

// covers: BAT-MAG-022
#[test]
fn magic_mirror_sends_a_spell_at_the_stack_to_a_stack_of_the_casters_side() {
    let r = ruleset();
    let book = ["bolt", "curse", "chain", "nova", "bless", "mirror"];
    let start = |attacker: &[(&'static str, u32, i32, i32)]| {
        let mut s = arena(
            &r,
            [Some(adept(&r, &book, 0)), None],
            attacker,
            &[("tank", 5, 13, 5), ("tank", 5, 13, 7)],
            1,
        );
        // The defender's first stack mirrors every spell (100 % at Expert).
        give(
            &mut s,
            &r,
            (u8::try_from(attacker.len()).unwrap(), "mirror"),
            3,
        );
        s
    };
    // With one stack, the attacker's damage spell comes back to it.
    let mut s = start(&[("tank", 5, 3, 5)]);
    assert_eq!(s.stacks()[1].spells.mirror, 100);
    let events = act(&mut s, &r, cast(&r, "bolt", stack(1))).unwrap();
    assert_eq!(report(&events), (vec![(1, 0)], vec![0], vec![], vec![]));
    assert!(s.stacks()[0].top_hp < 100);
    // A negative timed spell affects the caster's own stack.
    next_round(&mut s, &r, 0);
    let events = act(&mut s, &r, cast(&r, "curse", stack(1))).unwrap();
    assert_eq!(report(&events), (vec![(1, 0)], vec![], vec![], vec![0]));
    assert_eq!(on(&s, &r, 0), ["curse"]);
    // Chain Lightning starts its jumps at the new stack.
    next_round(&mut s, &r, 0);
    let events = act(&mut s, &r, cast(&r, "chain", stack(1))).unwrap();
    let (moved, damaged, ..) = report(&events);
    assert_eq!(moved, [(1, 0)]);
    assert_eq!(damaged.first(), Some(&0));
    // An area spell centred on it is not sent away.
    next_round(&mut s, &r, 0);
    let hex = s.stacks()[1].hex;
    let events = act(&mut s, &r, cast(&r, "nova", CastTarget::Hex(hex))).unwrap();
    assert!(report(&events).0.is_empty());
    // Nor is a spell aimed at a mirrored stack of the caster's side.
    let mut s = start(&[("tank", 5, 3, 5)]);
    give(&mut s, &r, (0, "mirror"), 3);
    let events = act(&mut s, &r, cast(&r, "bless", stack(0))).unwrap();
    assert_eq!(report(&events), (vec![], vec![], vec![], vec![0]));
    // A stack the spell cannot affect resists it: here the only own stack, immune to level 1.
    let mut s = start(&[("wyrm", 5, 3, 5)]);
    let events = act(&mut s, &r, cast(&r, "bolt", stack(1))).unwrap();
    assert_eq!(report(&events), (vec![(1, 0)], vec![], vec![0], vec![]));
}

// covers: BAT-MAG-022
#[test]
fn the_stack_a_mirror_chose_rolls_for_resistance_and_dispel_is_never_sent_away() {
    let r = ruleset();
    // The caster's hero makes its own stacks resist everything: the spell sent back fails there,
    // while the stack aimed at never rolls.
    let mut warded = adept(&r, &["bolt"], 0);
    warded.effects = std::vec![BattleEffect::MagicResistance {
        rate: Rate::bp(10_000)
    }];
    let mut s = arena(
        &r,
        [Some(warded), None],
        &[("tank", 5, 3, 5)],
        &[("tank", 5, 13, 5)],
        1,
    );
    give(&mut s, &r, (1, "mirror"), 3);
    let events = act(&mut s, &r, cast(&r, "bolt", stack(1))).unwrap();
    assert_eq!(report(&events), (vec![(1, 0)], vec![], vec![0], vec![]));
    // Dispel at Advanced reaches enemy stacks and is never sent away.
    let mut s = arena(
        &r,
        [Some(adept(&r, &["dispel"], 2)), None],
        &[("tank", 5, 3, 5)],
        &[("tank", 5, 13, 5)],
        1,
    );
    give(&mut s, &r, (1, "mirror"), 3);
    let events = act(&mut s, &r, cast(&r, "dispel", stack(1))).unwrap();
    assert!(report(&events).0.is_empty());
    assert!(on(&s, &r, 1).is_empty());
}

// covers: BAT-MAG-022
#[test]
fn a_mirror_redirects_with_its_chance_to_any_living_stack_of_the_caster() {
    let r = ruleset();
    let (mut sent, mut to) = (0, [0u32; 3]);
    let casts = 400;
    for seed in 0..casts {
        let mut s = arena(
            &r,
            [Some(adept(&r, &["bolt"], 0)), None],
            &[("tank", 5, 3, 5), ("tank", 5, 3, 7), ("tank", 5, 3, 9)],
            &[("tank", 5, 13, 5)],
            seed,
        );
        // 50 % at mastery none; the caster's third stack is dead and never chosen.
        give(&mut s, &r, (3, "mirror"), 0);
        crate::damage::apply_damage(s.at_mut(StackId(2)), 1_000);
        let events = act(&mut s, &r, cast(&r, "bolt", stack(3))).unwrap();
        if let [(3, t)] = report(&events).0.as_slice() {
            sent += 1;
            to[usize::from(*t)] += 1;
        }
    }
    assert!((170..=230).contains(&sent), "{sent} of {casts}");
    assert_eq!(to[2], 0);
    assert!(to[0] > 70 && to[1] > 70, "{to:?}");
}
