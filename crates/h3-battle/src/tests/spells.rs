//! Damage spells against stacks (E04b T2d): immunities (BAT-MAG-005), Chain Lightning
//! (BAT-MAG-006), area and battle-wide spells (BAT-MAG-007), magic resistance (BAT-MAG-008) and
//! the target's side of the damage (BAT-MAG-004).

use std::vec::Vec;

use h3_content::{Ruleset, SpellId};
use h3_contracts::{BattleEffect, BattleHero, BattleSetup, BattleSide};

use super::support::*;
use crate::magic::affect_chance;
use crate::{
    BattleState, CastTarget, CastTargets, Command, CommandError, Event, EventKind, Hex, StackId,
    cast_targets, castable_spells,
};

fn spell(r: &Ruleset, name: &str) -> SpellId {
    r.spell_by_key(&std::format!("t:{name}")).unwrap()
}

/// A hero with spell power 3, 15 mana and `spells` in its book.
fn mage(r: &Ruleset, spells: &[&str]) -> BattleHero {
    let mut h = hero(r);
    h.power = 3;
    h.magic.spell_book = Some(spells.iter().map(|s| spell(r, s)).collect());
    h
}

/// A battle with heroes and the stacks placed at `(x, y)`: `attacker` and `defender` as
/// `(creature, count, x, y)`. The first attacker stack is active.
fn arena(
    r: &Ruleset,
    heroes: [Option<BattleHero>; 2],
    attacker: &[(&'static str, u32, i32, i32)],
    defender: &[(&'static str, u32, i32, i32)],
    seed: u64,
) -> BattleState {
    let specs = |list: &[(&'static str, u32, i32, i32)]| -> Vec<Spec> {
        list.iter()
            .enumerate()
            .map(|(i, &(name, count, ..))| (u8::try_from(i).unwrap(), name, count))
            .collect()
    };
    let setup = BattleSetup {
        heroes,
        ..setup(r, &specs(attacker), &specs(defender))
    };
    let mut s = BattleState::new(r, &setup, seed).unwrap();
    for (i, &(_, _, x, y)) in attacker.iter().chain(defender).enumerate() {
        place(&mut s, r, u8::try_from(i).unwrap(), x, y);
    }
    activate(&mut s, 0);
    s
}

fn cast(r: &Ruleset, name: &str, target: CastTarget) -> Command {
    Command::Cast {
        spell: spell(r, name),
        target,
    }
}

fn stack(id: u8) -> CastTarget {
    CastTarget::Stack(StackId(id))
}

/// The damage events as `(target, damage)` and the resisted stacks, in order.
fn effects(events: &[Event]) -> (Vec<(u8, u64)>, Vec<u8>) {
    let mut damage = Vec::new();
    let mut resisted = Vec::new();
    for e in events {
        match e.kind {
            EventKind::SpellDamage {
                target, damage: d, ..
            } => damage.push((target.0, d)),
            EventKind::SpellResisted { target } => resisted.push(target.0),
            _ => {}
        }
    }
    (damage, resisted)
}

// covers: BAT-MAG-003, BAT-MAG-004, BAT-MAG-005
#[test]
fn immunities_by_level_school_and_spell_and_the_targets_reductions() {
    let r = ruleset();
    let book = ["bolt", "smite", "nova", "rally"];
    let start = || {
        arena(
            &r,
            [Some(mage(&r, &book)), None],
            &[("fixed", 1, 3, 5), ("ember", 5, 4, 7)],
            &[
                ("wyrm", 5, 10, 5),
                ("ember", 5, 12, 5),
                ("golem", 5, 12, 7),
                ("fixed", 5, 14, 3),
            ],
            1,
        )
    };
    let mut s = start();
    // Ids: 0 fixed, 1 ember (attacker); 2 wyrm, 3 ember, 4 golem, 5 fixed (defender).
    for command in [
        // The wyrm is immune to level-1 spells, the ember to fire spells (smite is air and
        // fire) and to rally.
        cast(&r, "bolt", stack(2)),
        cast(&r, "smite", stack(3)),
        cast(&r, "rally", stack(1)),
        // A hex spell may not be aimed at a side column.
        cast(&r, "nova", CastTarget::Hex(at(&r, 0, 5))),
        cast(&r, "nova", CastTarget::Hex(at(&r, 16, 5))),
    ] {
        let before = s.clone();
        assert_eq!(
            act(&mut s, &r, command),
            Err(CommandError::InvalidSpellTarget),
            "{command:?}"
        );
        assert_eq!(s, before);
    }
    let mut targets = CastTargets::new();
    cast_targets(
        &s,
        &r,
        BattleSide::Attacker,
        spell(&r, "bolt"),
        &mut targets,
    );
    assert_eq!(targets.as_slice(), [stack(3), stack(4), stack(5)]);
    cast_targets(
        &s,
        &r,
        BattleSide::Attacker,
        spell(&r, "rally"),
        &mut targets,
    );
    assert_eq!(targets.as_slice(), [stack(0)]);
    // The ember takes double damage from bolt (40 → 80), the golem a quarter (40 → 10); the
    // event carries the value, the caster's damage and the damage dealt (batch 2a R5).
    let events = act(&mut s, &r, cast(&r, "bolt", stack(3))).unwrap();
    assert_eq!(effects(&events).0, [(3, 80)]);
    let mut s = start();
    let events = act(&mut s, &r, cast(&r, "bolt", stack(4))).unwrap();
    let damage: Vec<EventKind> = events
        .iter()
        .map(|e| e.kind)
        .filter(|k| matches!(k, EventKind::SpellDamage { .. }))
        .collect();
    assert_eq!(
        damage,
        [EventKind::SpellDamage {
            target: StackId(4),
            value: 40,
            caster: 40,
            damage: 10,
            kills: 1,
        }]
    );
    // Nova (fire) at the golem's hex: the golem and the wyrm's neighbour... the ember next to
    // it is passed over (fire immune), the golem takes its quarter.
    let mut s = start();
    let (damage, resisted) =
        effects(&act(&mut s, &r, cast(&r, "nova", CastTarget::Hex(at(&r, 12, 7)))).unwrap());
    assert_eq!((damage, resisted), (std::vec![(4, 10)], std::vec![]));
    // Nova aimed at an immune ember: the ember is spared, the stack beside it struck (batch 2a
    // R3c); aimed where its area reaches nobody, it is cast for nothing (R3b).
    let mut s = arena(
        &r,
        [Some(mage(&r, &book)), None],
        &[("fixed", 1, 3, 5)],
        &[("ember", 5, 12, 5), ("fixed", 5, 13, 5)],
        1,
    );
    let (damage, _) =
        effects(&act(&mut s, &r, cast(&r, "nova", CastTarget::Hex(at(&r, 12, 5)))).unwrap());
    assert_eq!(damage, [(2, 40)]);
    let mut s = start();
    let (damage, resisted) =
        effects(&act(&mut s, &r, cast(&r, "nova", CastTarget::Hex(at(&r, 7, 1)))).unwrap());
    assert!(damage.is_empty() && resisted.is_empty());
    assert_eq!(s.hero(BattleSide::Attacker).unwrap().casts, 1);
}

// covers: BAT-MAG-008
#[test]
fn resistance_adds_the_heros_to_the_creatures_and_the_aura_multiplies() {
    let r = ruleset();
    let single = ruleset_with("t:single", |_| {});
    let warded = |bp: &[u16]| {
        let mut h = hero(&r);
        h.effects = bp
            .iter()
            .map(|&bp| BattleEffect::MagicResistance { bp })
            .collect();
        h
    };
    let bolt = r.spell(spell(&r, "bolt")).unwrap();
    let rally = r.spell(spell(&r, "rally")).unwrap();
    let chance = |rules: &Ruleset, defender: Option<BattleHero>, id: u8| {
        let s = arena(
            rules,
            [Some(mage(rules, &["bolt"])), defender],
            &[("fixed", 1, 2, 5)],
            &[
                ("fixed", 5, 12, 1),
                ("dwarf", 5, 12, 5),
                ("unicorn", 5, 13, 5),
                ("dwarf", 5, 12, 9),
            ],
            1,
        );
        let spell = rules.spell(spell(rules, "bolt")).unwrap();
        affect_chance(&s, rules, spell, StackId(id))
    };
    // Ids: 1 fixed, 2 dwarf next to the unicorn (3), 4 dwarf alone.
    assert_eq!(chance(&r, None, 1), 100);
    assert_eq!(chance(&r, None, 4), 80);
    // The aura of an adjacent ally multiplies (0.8 × 0.8); the unicorn itself has none.
    assert_eq!(chance(&r, None, 2), 64);
    assert_eq!(chance(&r, None, 3), 100);
    // The hero's resistance subtracts from the creature's share, its sources added.
    assert_eq!(chance(&r, Some(warded(&[500])), 4), 75);
    assert_eq!(chance(&r, Some(warded(&[500, 500])), 1), 90);
    assert_eq!(chance(&r, Some(warded(&[8000])), 4), 0);
    assert_eq!(chance(&r, Some(warded(&[20_000])), 1), 0);
    // Single precision (quirk Q-0005): 1 − 0.1 is 0.89999998, so 89 %; 1 − 0.05 gives 94 %.
    assert_eq!(chance(&single, Some(warded(&[1000])), 1), 89);
    assert_eq!(chance(&single, Some(warded(&[500, 500])), 1), 89);
    assert_eq!(chance(&single, Some(warded(&[500])), 1), 94);
    assert_eq!(chance(&single, Some(warded(&[500])), 4), 75);
    assert_eq!(chance(&single, None, 2), 64);
    assert_eq!(chance(&single, None, 1), 100);
    // A positive spell is never resisted.
    let s = arena(
        &r,
        [Some(mage(&r, &["rally"])), None],
        &[("dwarf", 1, 2, 5)],
        &[("fixed", 1, 12, 5)],
        1,
    );
    assert_eq!(affect_chance(&s, &r, rally, StackId(0)), 100);
    assert_eq!(affect_chance(&s, &r, bolt, StackId(0)), 80);
}

/// Casts `name` at `target` in battles with seeds `1..=n` against Dwarves (id 1) beside a stack
/// without resistance (id 2); returns how often the Dwarves resisted and how often the spell did
/// nothing at all.
fn dwarves_resisting(r: &Ruleset, name: &str, target: CastTarget, n: u64) -> (usize, usize) {
    let (mut dwarves, mut nothing) = (0, 0);
    for seed in 1..=n {
        let mut s = arena(
            r,
            [Some(mage(r, &[name])), None],
            &[("fixed", 1, 2, 5)],
            &[("dwarf", 5, 12, 5), ("fixed", 5, 13, 5)],
            seed,
        );
        let events = act(&mut s, r, cast(r, name, target)).unwrap();
        // The cast is paid and counted either way.
        assert_eq!(s.hero(BattleSide::Attacker).unwrap().casts, 1);
        let (damage, resisted) = effects(&events);
        dwarves += usize::from(resisted.contains(&1));
        nothing += usize::from(damage.is_empty());
    }
    (dwarves, nothing)
}

// covers: BAT-MAG-006, BAT-MAG-007, BAT-MAG-008
#[test]
fn a_stack_aimed_at_that_resists_stops_a_stack_spell_but_not_an_area_spell() {
    let r = ruleset();
    // Dwarves resist one cast in five. Bolt at them does nothing when they resist.
    let (dwarves, nothing) = dwarves_resisting(&r, "bolt", stack(1), 60);
    assert!(dwarves > 3 && dwarves == nothing, "{dwarves} {nothing}");
    // Nova at their hex: they roll once like every stack in the area, and the stack beside them
    // is struck every time (the original's code and batch 2a R3).
    let (dwarves, nothing) = dwarves_resisting(&r, "nova", CastTarget::Hex(at(&r, 12, 5)), 60);
    assert!(dwarves > 3 && nothing == 0, "{dwarves} {nothing}");
}

// covers: BAT-MAG-007
#[test]
fn area_and_battle_wide_spells_strike_every_stack_they_reach() {
    let r = ruleset();
    let book = ["nova", "ring", "storm"];
    let start = || {
        arena(
            &r,
            [Some(mage(&r, &book)), None],
            &[("fixed", 5, 9, 5), ("wide", 2, 3, 5)],
            &[
                ("fixed", 5, 10, 5),
                ("fixed", 5, 11, 5),
                ("wyrm", 5, 10, 6),
                ("lich", 5, 9, 4),
            ],
            1,
        )
    };
    // Ids: 0 fixed (attacker, (9,5)), 1 wide; 2 fixed (10,5), 3 fixed (11,5), 4 wyrm, 5 lich.
    // Nova at (10,5): every stack within one hex, both sides, but the immune wyrm.
    let mut s = start();
    let (damage, _) =
        effects(&act(&mut s, &r, cast(&r, "nova", CastTarget::Hex(at(&r, 10, 5)))).unwrap());
    assert_eq!(damage, [(0, 40), (2, 40), (3, 40), (5, 40)]);
    // Ring at (10,5) spares the stack on its centre hex.
    let mut s = start();
    let (damage, _) =
        effects(&act(&mut s, &r, cast(&r, "ring", CastTarget::Hex(at(&r, 10, 5)))).unwrap());
    assert_eq!(damage, [(0, 40), (3, 40), (5, 40)]);
    // Storm strikes every stack but the wyrm, its caster's included; a two-hex stack once.
    let mut s = start();
    let (damage, _) = effects(&act(&mut s, &r, cast(&r, "storm", CastTarget::None)).unwrap());
    assert_eq!(damage, [(0, 40), (1, 40), (2, 40), (3, 40), (5, 40)]);
    // A battle-wide spell that reaches nobody cannot be cast: only wyrms left. Hex spells can
    // still be aimed at any hex.
    let mut s = arena(
        &r,
        [Some(mage(&r, &book)), None],
        &[("wyrm", 1, 2, 5)],
        &[("wyrm", 1, 12, 5)],
        1,
    );
    let castable: Vec<SpellId> = castable_spells(&s, &r, BattleSide::Attacker)
        .iter()
        .collect();
    assert_eq!(castable, [spell(&r, "nova"), spell(&r, "ring")]);
    assert_eq!(
        act(&mut s, &r, cast(&r, "storm", CastTarget::None)),
        Err(CommandError::InvalidSpellTarget)
    );
}

// covers: BAT-MAG-006
#[test]
fn chain_lightning_jumps_to_the_nearest_stack_on_the_screen() {
    let r = ruleset();
    let chain = |s: &mut BattleState, first: u8| {
        effects(&act(s, &r, cast(&r, "chain", stack(first))).unwrap()).0
    };
    // From (8,5): a stack two hexes right (88 pixels) and one two rows down in the same column
    // (84 pixels; hex distance 2 for both) — the chain takes the closer on the screen. Each jump
    // halves the value: 40, 20, 10.
    let mut s = arena(
        &r,
        [Some(mage(&r, &["chain"])), None],
        &[("fixed", 1, 2, 1)],
        &[("tank", 5, 8, 5), ("tank", 5, 10, 5), ("tank", 5, 8, 7)],
        1,
    );
    assert_eq!(chain(&mut s, 1), [(1, 40), (3, 20), (2, 10)]);
    // Equal distances: the defender's stacks come first, then by id; the chain may strike the
    // caster's own stacks. At Advanced it strikes four.
    let mut adept = mage(&r, &["chain"]);
    adept.magic.schools = [2, 0, 0, 0];
    let mut s = arena(
        &r,
        [Some(adept), None],
        &[("tank", 5, 6, 5), ("tank", 5, 10, 5)],
        &[("tank", 5, 8, 5), ("tank", 5, 12, 5)],
        1,
    );
    // Ids: 0 (6,5), 1 (10,5) attacker; 2 (8,5), 3 (12,5) defender. Advanced: 3 × 10 + 20 = 50.
    // From 2, stacks 0 and 1 are 88 pixels away: the attacker's lower id first; then from 0
    // the nearest left is 1 (176), then 3.
    assert_eq!(chain(&mut s, 2), [(2, 50), (0, 25), (1, 12), (3, 6)]);
    // Immune stacks are passed over, and the chain ends when nobody is left.
    let mut s = arena(
        &r,
        [Some(mage(&r, &["chain"])), None],
        &[("fixed", 1, 2, 1)],
        &[("tank", 5, 8, 5), ("wyrm", 5, 9, 5), ("tank", 5, 12, 5)],
        1,
    );
    assert_eq!(chain(&mut s, 1), [(1, 40), (3, 20), (0, 10)]);
    // Stacks that always resist (their hero's resistance) are never jumped to: the caster's
    // own here, next to the target.
    let mut warded = mage(&r, &["chain"]);
    warded.effects = std::vec![BattleEffect::MagicResistance { bp: 10_000 }];
    let mut s = arena(
        &r,
        [Some(warded), None],
        &[("tank", 5, 9, 5), ("tank", 5, 2, 9)],
        &[("tank", 5, 10, 5), ("tank", 5, 13, 5)],
        1,
    );
    assert_eq!(chain(&mut s, 2), [(2, 40), (3, 20)]);
    // The chain is aimed only at enemies (batch 2a R1), and not at a stack that always
    // resists: not at the defender's stacks when their hero resists everything.
    let mut s = arena(
        &r,
        [Some(mage(&r, &["chain"])), None],
        &[("tank", 5, 2, 5)],
        &[("tank", 5, 10, 5)],
        1,
    );
    assert_eq!(
        act(&mut s, &r, cast(&r, "chain", stack(0))),
        Err(CommandError::InvalidSpellTarget)
    );
    let mut warded = hero(&r);
    warded.effects = std::vec![BattleEffect::MagicResistance { bp: 10_000 }];
    let s = arena(
        &r,
        [Some(mage(&r, &["chain", "bolt", "nova"])), Some(warded)],
        &[("tank", 5, 2, 5)],
        &[("tank", 5, 10, 5)],
        1,
    );
    let castable: Vec<SpellId> = castable_spells(&s, &r, BattleSide::Attacker)
        .iter()
        .collect();
    assert_eq!(castable, [spell(&r, "nova")]);
}

// covers: BAT-MAG-006, BAT-MAG-008
#[test]
fn chain_lightning_passes_over_a_stack_that_resists_a_jump() {
    let r = ruleset();
    // Dwarves next to the first target resist some jumps: the chain then goes on to the
    // fixed stack further away, keeping its count (three stacks).
    let mut passed_over = 0;
    for seed in 1..=40 {
        let mut s = arena(
            &r,
            [Some(mage(&r, &["chain"])), None],
            &[("fixed", 1, 2, 1)],
            &[("tank", 5, 8, 5), ("dwarf", 5, 9, 5), ("tank", 5, 12, 5)],
            seed,
        );
        let (damage, resisted) = effects(&act(&mut s, &r, cast(&r, "chain", stack(1))).unwrap());
        assert!(resisted.is_empty(), "a jump is passed over silently");
        assert_eq!(damage.len(), 3);
        if damage[1].0 == 3 {
            passed_over += 1;
        }
    }
    assert!((2..=20).contains(&passed_over), "{passed_over}");
}

// covers: BAT-MAG-003, BAT-MAG-007
#[test]
fn hex_spells_list_every_hex_they_may_be_aimed_at() {
    let r = ruleset();
    let s = arena(
        &r,
        [Some(mage(&r, &["nova", "ring"])), None],
        &[("fixed", 1, 2, 1)],
        &[("ember", 1, 8, 5), ("fixed", 1, 12, 9)],
        1,
    );
    let field = field(&r);
    let mut targets = CastTargets::new();
    let hexes = |targets: &CastTargets| -> Vec<Hex> {
        targets
            .iter()
            .map(|t| match *t {
                CastTarget::Hex(h) => h,
                other => panic!("{other:?}"),
            })
            .collect()
    };
    // Nova and ring: every hex outside the side columns, empty ones far from every stack and
    // the immune ember's included (batch 2a R3b, R3c; the original's cursor test).
    let standable: Vec<Hex> = (0u8..)
        .take(field.hex_count())
        .map(Hex)
        .filter(|&h| field.is_standable(h))
        .collect();
    for name in ["nova", "ring"] {
        cast_targets(&s, &r, BattleSide::Attacker, spell(&r, name), &mut targets);
        assert_eq!(hexes(&targets), standable, "{name}");
    }
    assert!(standable.contains(&at(&r, 8, 5)) && !standable.contains(&at(&r, 0, 5)));
    assert_eq!(
        castable_spells(&s, &r, BattleSide::Attacker).iter().count(),
        2
    );
}
