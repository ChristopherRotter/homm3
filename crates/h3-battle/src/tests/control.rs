//! Control (E04b T4c): Berserk (BAT-MAG-025) and Hypnotize (BAT-MAG-026), after the original's
//! code (`docs/formats/installation.md` §7.14).

use std::string::String;
use std::vec;
use std::vec::Vec;

use h3_content::Ruleset;
use h3_contracts::{BattleHero, BattleSide, Rate};

use super::spells::{arena, cast, mage, spell, stack};
use super::support::*;
use crate::effects::TimedCast;
use crate::state::StackFlags;
use crate::{
    BattleState, CastTarget, CastTargets, Command, CommandError, Event, EventKind, LegalCommands,
    StackId, Status, cast_targets, legal_commands,
};

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

/// A mage with `spells`, the air mastery `mastery` and enough mana.
fn adept(r: &Ruleset, spells: &[&str], mastery: u8) -> BattleHero {
    let mut h = mage(r, spells);
    h.magic.masteries = masteries(r, [mastery, 0, 0, 0]);
    h.magic.mana = 200;
    h
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

/// The strikes `events` report, as `(attacker, target)`.
fn strikes(events: &[Event]) -> Vec<(u8, u8)> {
    events
        .iter()
        .filter_map(|e| match e.kind {
            EventKind::Struck {
                attacker, target, ..
            } => Some((attacker.0, target.0)),
            _ => None,
        })
        .collect()
}

/// The stack targets of `name` for `side` now.
fn targets(s: &BattleState, r: &Ruleset, side: BattleSide, name: &str) -> Vec<CastTarget> {
    let mut out = CastTargets::new();
    cast_targets(s, r, side, spell(r, name), &mut out);
    out.as_slice().to_vec()
}

/// The legal commands of `side` now.
fn legal(s: &BattleState, r: &Ruleset, side: BattleSide) -> Vec<Command> {
    let mut out = LegalCommands::new();
    legal_commands(s, r, side, &mut out);
    out.as_slice().to_vec()
}

/// Whether `commands` attack stack `id`.
fn attacks(commands: &[Command], id: u8) -> bool {
    commands
        .iter()
        .any(|c| matches!(c, Command::Attack { target, .. } if target.0 == id))
}

/// Whether `commands` shoot at stack `id`.
fn shoots(commands: &[Command], id: u8) -> bool {
    commands
        .iter()
        .any(|c| matches!(c, Command::Shoot { target, .. } if target.0 == id))
}

/// Whether stack `id` has `status`.
fn has(s: &BattleState, id: u8, status: Status) -> bool {
    s.stacks()[usize::from(id)].spells.status.contains(status)
}

/// The active berserk stack takes its automatic turn now; returns the events.
fn berserk_now(s: &mut BattleState, r: &Ruleset) -> Vec<Event> {
    let mut events = Vec::new();
    crate::turn::berserk_now(s, r, &mut events);
    events
}

// covers: BAT-MAG-025
#[test]
fn berserk_reaches_both_sides_around_its_hex_and_ends_hypnotize() {
    let r = ruleset();
    let place = |mastery| {
        let mut s = arena(
            &r,
            [Some(adept(&r, &["berserk"], mastery)), None],
            &[("fixed", 5, 2, 1), ("fixed", 5, 6, 5)],
            &[("fixed", 5, 7, 5), ("fixed", 5, 9, 5)],
            1,
        );
        give(&mut s, &r, (2, "hypnotize"), 1);
        act(
            &mut s,
            &r,
            cast(&r, "berserk", CastTarget::Hex(at(&r, 7, 5))),
        )
        .unwrap();
        s
    };
    // Basic: the stack on the hex only, which loses its Hypnotize.
    let s = place(1);
    let berserk: Vec<bool> = (0..4).map(|i| has(&s, i, Status::BERSERK)).collect();
    assert_eq!(berserk, [false, false, true, false]);
    assert_eq!(on(&s, &r, 2), ["berserk"]);
    // Advanced: one hex around, the caster's own stack too.
    let s = place(2);
    let berserk: Vec<bool> = (0..4).map(|i| has(&s, i, Status::BERSERK)).collect();
    assert_eq!(berserk, [false, true, true, false]);
    // It lasts 255 rounds whatever the caster's power.
    let entry = s.effects().iter().find(|e| e.stack == StackId(1)).unwrap();
    assert_eq!(entry.rounds, Some(255));
    // Hypnotize ends Berserk in turn.
    let mut s = place(1);
    give(&mut s, &r, (2, "hypnotize"), 1);
    assert_eq!(on(&s, &r, 2), ["hypnotize"]);
}

// covers: BAT-MAG-025
#[test]
fn a_berserk_stack_attacks_the_nearest_stack_on_its_own_turn() {
    let r = ruleset();
    // Ids: 0 fixed (berserk), 1 fixed (attacker); 2 brute, 3 slow (defender).
    let mut s = arena(
        &r,
        [None, None],
        &[("fixed", 10, 5, 5), ("fixed", 10, 7, 5)],
        &[("brute", 5, 2, 1), ("slow", 5, 12, 5)],
        1,
    );
    give(&mut s, &r, (0, "berserk"), 1);
    // Its own side may not attack it (E04b T11b review, F-r3-control-battlefield-2: this test
    // had the attacker's brute attack it), nor its other stacks; the other side may, as ever.
    activate(&mut s, 1);
    let commands = legal(&s, &r, BattleSide::Attacker);
    assert!(!attacks(&commands, 0) && attacks(&commands, 3));
    let own = Command::Attack {
        target: StackId(0),
        from: at(&r, 6, 5),
    };
    assert_eq!(act(&mut s, &r, own), Err(CommandError::NotAnEnemy));
    activate(&mut s, 2);
    assert!(attacks(&legal(&s, &r, BattleSide::Defender), 0));
    let attack = Command::Attack {
        target: StackId(0),
        from: at(&r, 4, 5),
    };
    let events = act(&mut s, &r, attack).unwrap();
    // The brute's attack and the retaliation, which does not end Berserk; then the berserk
    // stack's own turn: it attacks the nearest stack, the brute next to it, and its Berserk ends.
    assert_eq!(strikes(&events), [(2, 0), (0, 2), (0, 2), (2, 0)]);
    let ended = events.iter().position(|e| {
        e.kind
            == EventKind::EffectEnded {
                target: StackId(0),
                spell: spell(&r, "berserk"),
            }
    });
    assert!(
        ended.unwrap()
            > events
                .iter()
                .rposition(|e| matches!(e.kind, EventKind::Struck { .. }))
                .unwrap()
    );
    assert!(s.stacks()[0].flags.contains(StackFlags::ACTED));
    assert_eq!(s.active(), Some(StackId(1)));
    // Without the brute next to it, it walks to its own stack, the nearest, and attacks it.
    let mut s = arena(
        &r,
        [None, None],
        &[("fixed", 10, 5, 5), ("fixed", 10, 7, 5)],
        &[("slow", 5, 12, 5)],
        1,
    );
    give(&mut s, &r, (0, "berserk"), 1);
    let events = berserk_now(&mut s, &r);
    assert!(events.contains(&Event::public(EventKind::Moved {
        stack: StackId(0),
        from: at(&r, 5, 5),
        to: at(&r, 6, 5),
    })));
    assert_eq!(strikes(&events), [(0, 1), (1, 0)]);
    assert!(!has(&s, 0, Status::BERSERK));
}

/// E04b T11b review (F-r3-control-battlefield-2): a berserk stack is not its own side's to
/// attack or shoot (the original's cursor compares the target's own side with the acting side
/// and has no Berserk test, the reviewers' reading of 0x476490; VCMI's `battleCanAttackUnit`
/// has none either), while the other side attacks and shoots it as ever. Next to another stack it
/// counts as that stack's enemy whatever the sides (the two callers of the original's 0x442880):
/// it blocks a shooter and a multi-headed strike reaches it.
// covers: BAT-MAG-025, BAT-RNG-001, BAT-ATK-003
#[test]
fn a_berserk_stack_is_no_target_of_its_own_side_and_an_enemy_next_to_any_stack() {
    let r = ruleset();
    // Ids: 0 archer above the berserk fixed (1), 2 archer far away (attacker); 3 archer beside
    // the berserk stack, 4 archer far away (defender). Each archer beside it touches no other
    // stack.
    let mut s = arena(
        &r,
        [None, None],
        &[("archer", 5, 7, 4), ("fixed", 5, 8, 5), ("archer", 5, 2, 1)],
        &[("archer", 5, 9, 5), ("archer", 5, 14, 8)],
        1,
    );
    let distance =
        |a: (i32, i32), b: (i32, i32)| field(&r).distance(at(&r, a.0, a.1), at(&r, b.0, b.1));
    assert_eq!((distance((7, 4), (8, 5)), distance((7, 4), (9, 5))), (1, 2));
    give(&mut s, &r, (1, "berserk"), 1);
    let shot = |target: u8, (x, y): (i32, i32)| Command::Shoot {
        target: StackId(target),
        at: at(&r, x, y),
    };
    // Its own side's archers beside it are blocked and may not strike it either.
    let commands = legal(&s, &r, BattleSide::Attacker);
    assert!(!commands.iter().any(|c| matches!(c, Command::Shoot { .. })));
    assert!(!attacks(&commands, 1) && attacks(&commands, 3));
    assert_eq!(
        act(&mut s, &r, shot(4, (14, 8))),
        Err(CommandError::CannotShoot)
    );
    // Those far away shoot the other side's stacks only.
    activate(&mut s, 2);
    let commands = legal(&s, &r, BattleSide::Attacker);
    assert!(shoots(&commands, 3) && shoots(&commands, 4) && !shoots(&commands, 1));
    assert_eq!(
        act(&mut s, &r, shot(1, (8, 5))),
        Err(CommandError::NotAnEnemy)
    );
    // The other side's archers beside it are blocked too and strike it; those far away shoot it.
    activate(&mut s, 3);
    let commands = legal(&s, &r, BattleSide::Defender);
    assert!(!commands.iter().any(|c| matches!(c, Command::Shoot { .. })));
    assert!(attacks(&commands, 1));
    activate(&mut s, 4);
    assert!(shoots(&legal(&s, &r, BattleSide::Defender), 1));
    // A hydra strikes the berserk stack of its own side beside it with the stack it attacks.
    let mut s = arena(
        &r,
        [None, None],
        &[("hydra", 10, 7, 5), ("fixed", 10, 6, 4)],
        &[("fixed", 10, 8, 5)],
        1,
    );
    give(&mut s, &r, (1, "berserk"), 1);
    let attack = Command::Attack {
        target: StackId(2),
        from: at(&r, 7, 5),
    };
    let events = act(&mut s, &r, attack).unwrap();
    assert_eq!(strikes(&events)[..2], [(0, 2), (0, 1)]);
}

// covers: BAT-MAG-025
#[test]
fn a_berserk_shooter_shoots_the_nearest_stack_and_ties_are_drawn() {
    let r = ruleset();
    let mut s = arena(
        &r,
        [None, None],
        &[("archer", 10, 5, 5), ("fixed", 5, 8, 5)],
        &[("fixed", 5, 13, 5)],
        1,
    );
    give(&mut s, &r, (0, "berserk"), 1);
    let events = berserk_now(&mut s, &r);
    assert_eq!(strikes(&events), [(0, 1)]);
    assert!(events.iter().any(|e| matches!(
        e.kind,
        EventKind::Struck { flags, .. } if flags.contains(crate::StrikeFlags::RANGED)
    )));
    assert!(!has(&s, 0, Status::BERSERK));
    // Between two stacks equally near it draws with equal chances.
    let mut seen = [false; 3];
    for seed in 1..40 {
        let mut s = arena(
            &r,
            [None, None],
            &[("fixed", 10, 7, 5)],
            &[("slow", 5, 6, 5), ("slow", 5, 8, 5)],
            seed,
        );
        give(&mut s, &r, (0, "berserk"), 1);
        let events = berserk_now(&mut s, &r);
        seen[usize::from(strikes(&events)[0].1)] = true;
    }
    assert_eq!(seen, [false, true, true]);
}

// covers: BAT-MAG-025
#[test]
fn a_berserk_stack_out_of_reach_walks_towards_its_target_or_passes() {
    let r = ruleset();
    // A walker beyond its speed: two steps on its way, no attack, still berserk.
    let mut s = arena(
        &r,
        [None, None],
        &[("slow", 5, 2, 5)],
        &[("fixed", 5, 12, 5)],
        1,
    );
    give(&mut s, &r, (0, "berserk"), 1);
    let events = berserk_now(&mut s, &r);
    assert!(events.contains(&Event::public(EventKind::Moved {
        stack: StackId(0),
        from: at(&r, 2, 5),
        to: at(&r, 4, 5),
    })));
    assert!(strikes(&events).is_empty());
    assert!(has(&s, 0, Status::BERSERK));
    assert!(s.stacks()[0].flags.contains(StackFlags::ACTED));
    // A flyer: the position within its speed nearest to the attack position.
    let mut s = arena(
        &r,
        [None, None],
        &[("flyer", 5, 1, 5)],
        &[("fixed", 5, 14, 5)],
        1,
    );
    give(&mut s, &r, (0, "berserk"), 1);
    let events = berserk_now(&mut s, &r);
    assert!(events.contains(&Event::public(EventKind::Moved {
        stack: StackId(0),
        from: at(&r, 1, 5),
        to: at(&r, 8, 5),
    })));
    // Unable to move, or with no stack to attack, its turn passes.
    let mut s = arena(
        &r,
        [None, None],
        &[("slow", 5, 2, 5)],
        &[("fixed", 5, 12, 5)],
        1,
    );
    give(&mut s, &r, (0, "berserk"), 1);
    s.stacks.as_mut_slice()[0].spells.speed = 0;
    let events = berserk_now(&mut s, &r);
    assert!(events.contains(&Event::public(EventKind::TurnPassed { stack: StackId(0) })));
    let mut s = arena(
        &r,
        [None, None],
        &[("slow", 5, 2, 5)],
        &[("fixed", 5, 12, 5)],
        1,
    );
    give(&mut s, &r, (0, "berserk"), 1);
    s.stacks.as_mut_slice()[1].count = 0;
    let events = berserk_now(&mut s, &r);
    assert!(events.contains(&Event::public(EventKind::TurnPassed { stack: StackId(0) })));
}

// covers: BAT-MAG-025
#[test]
fn a_cast_that_makes_the_active_stack_berserk_lets_it_act_at_once() {
    let r = ruleset();
    let mut s = arena(
        &r,
        [Some(adept(&r, &["berserk"], 2)), None],
        &[("fixed", 10, 5, 5), ("fixed", 5, 2, 1)],
        &[("slow", 5, 7, 5)],
        1,
    );
    let events = act(
        &mut s,
        &r,
        cast(&r, "berserk", CastTarget::Hex(at(&r, 6, 5))),
    )
    .unwrap();
    // Both stacks next to the hex are berserk; the active one attacks the slow stack at once.
    assert!(strikes(&events).first() == Some(&(0, 2)));
    assert!(!has(&s, 0, Status::BERSERK));
    assert!(s.stacks()[0].flags.contains(StackFlags::ACTED));
    assert_ne!(s.active(), Some(StackId(0)));
    // A cast that leaves the active stack alone changes nothing about its turn.
    let mut s = arena(
        &r,
        [Some(adept(&r, &["berserk"], 1)), None],
        &[("fixed", 10, 5, 5)],
        &[("slow", 5, 9, 5)],
        1,
    );
    act(
        &mut s,
        &r,
        cast(&r, "berserk", CastTarget::Hex(at(&r, 9, 5))),
    )
    .unwrap();
    assert_eq!(s.active(), Some(StackId(0)));
}

// covers: BAT-MAG-026
#[test]
fn hypnotize_needs_full_hit_points_within_its_value_and_no_mirror() {
    let r = ruleset();
    let arena_at = |mastery| {
        let mut s = arena(
            &r,
            [Some(adept(&r, &["hypnotize"], mastery)), None],
            &[("fixed", 5, 2, 5)],
            &[("fixed", 8, 9, 3), ("fixed", 9, 9, 5), ("fixed", 8, 9, 7)],
            1,
        );
        // Power 3: 75 + 10 at Basic, 75 + 50 at Expert. The wounded stack's top creature does
        // not count less.
        s.stacks.as_mut_slice()[2].top_hp = 1;
        give(&mut s, &r, (3, "mirror"), 1);
        s
    };
    let s = arena_at(1);
    assert_eq!(
        targets(&s, &r, BattleSide::Attacker, "hypnotize"),
        [stack(1)]
    );
    let s = arena_at(3);
    assert_eq!(
        targets(&s, &r, BattleSide::Attacker, "hypnotize"),
        [stack(1), stack(2)]
    );
}

// covers: BAT-MAG-026
#[test]
fn a_hypnotized_stack_fights_for_the_other_side() {
    let r = ruleset();
    let mut hero = adept(&r, &["hypnotize"], 3);
    hero.damage.melee = Rate::bp(5000);
    let mut s = arena(
        &r,
        [Some(hero), None],
        &[("fixed", 5, 2, 5)],
        &[("fixed", 5, 9, 5), ("fixed", 5, 11, 5)],
        1,
    );
    act(&mut s, &r, cast(&r, "hypnotize", stack(1))).unwrap();
    assert_eq!(s.stacks()[1].controller(), BattleSide::Attacker);
    act(&mut s, &r, Command::Defend).unwrap();
    // Its turn is the attacker's: it may attack its own side's stack, not the attacker's.
    assert_eq!(s.active(), Some(StackId(1)));
    assert_eq!(s.side_to_act(), Some(BattleSide::Attacker));
    assert!(legal(&s, &r, BattleSide::Defender).is_empty());
    let commands = legal(&s, &r, BattleSide::Attacker);
    assert!(attacks(&commands, 2) && !attacks(&commands, 0));
    let attack = Command::Attack {
        target: StackId(2),
        from: at(&r, 10, 5),
    };
    let events = act(&mut s, &r, attack).unwrap();
    // With the attacker's hero's +50 % melee damage: 5 × 2 × 1.5.
    let damage = events.iter().find_map(|e| match e.kind {
        EventKind::Struck {
            attacker: StackId(1),
            damage,
            ..
        } => Some(damage),
        _ => None,
    });
    assert_eq!(damage, Some(15));
    // While its side's only stack fights for the other side, the battle goes on.
    let mut s = arena(
        &r,
        [Some(adept(&r, &["hypnotize"], 3)), None],
        &[("fixed", 5, 2, 5)],
        &[("fixed", 5, 9, 5)],
        1,
    );
    act(&mut s, &r, cast(&r, "hypnotize", stack(1))).unwrap();
    act(&mut s, &r, Command::Defend).unwrap();
    assert_eq!(s.result(), None);
    assert!(!attacks(&legal(&s, &r, BattleSide::Attacker), 0));
}

/// E04b T11b review (F-r3-control-battlefield-1, scenarios a and b): who may attack or shoot a
/// stack is decided by the acting stack's controller against the other stack's own side, as the
/// original's cursor and pathfinder (the reviewers' reading of 0x476490) and VCMI's
/// `battleMatchOwner` have it; batch 2c row H1: "Blue's stacks cannot attack the hypnotized
/// stack". The engine compared the two controllers, the other way round on both counts.
// covers: BAT-MAG-026
#[test]
fn a_hypnotized_stack_is_attacked_by_the_side_that_controls_it_and_not_by_its_own() {
    let r = ruleset();
    // Ids: 0 fixed, 1 archer (attacker); 2 fixed (hypnotized), 3 fixed, 4 archer (defender).
    let mut s = arena(
        &r,
        [None, None],
        &[("fixed", 5, 8, 5), ("archer", 5, 2, 2)],
        &[
            ("fixed", 5, 9, 5),
            ("fixed", 5, 10, 5),
            ("archer", 5, 14, 8),
        ],
        1,
    );
    give(&mut s, &r, (2, "hypnotize"), 1);
    // The attacker, who controls it, may attack and shoot it like any stack of the defender.
    let commands = legal(&s, &r, BattleSide::Attacker);
    assert!(attacks(&commands, 2) && attacks(&commands, 3));
    activate(&mut s, 1);
    let commands = legal(&s, &r, BattleSide::Attacker);
    assert!(shoots(&commands, 2) && shoots(&commands, 3) && !shoots(&commands, 0));
    // Its own side may neither attack nor shoot it.
    activate(&mut s, 3);
    let commands = legal(&s, &r, BattleSide::Defender);
    assert!(!attacks(&commands, 2) && attacks(&commands, 0));
    let attack = Command::Attack {
        target: StackId(2),
        from: at(&r, 10, 5),
    };
    assert_eq!(act(&mut s, &r, attack), Err(CommandError::NotAnEnemy));
    activate(&mut s, 4);
    let commands = legal(&s, &r, BattleSide::Defender);
    assert!(!shoots(&commands, 2) && shoots(&commands, 0));
    let shot = Command::Shoot {
        target: StackId(2),
        at: at(&r, 9, 5),
    };
    assert_eq!(act(&mut s, &r, shot), Err(CommandError::NotAnEnemy));
    // The hypnotized stack itself attacks the stacks of its own side, another hypnotized one
    // among them, and none of the side that commands it.
    give(&mut s, &r, (3, "hypnotize"), 1);
    activate(&mut s, 2);
    let commands = legal(&s, &r, BattleSide::Attacker);
    assert!(attacks(&commands, 3) && !attacks(&commands, 0));
    // The attacker's attack on the stack it controls is carried out.
    activate(&mut s, 0);
    let attack = Command::Attack {
        target: StackId(2),
        from: at(&r, 8, 5),
    };
    let events = act(&mut s, &r, attack).unwrap();
    assert_eq!(strikes(&events).first(), Some(&(0, 2)));
}

/// E04b T11b review (F-r3-control-battlefield-1, scenario c): next to another stack a
/// hypnotized stack counts as the enemy of the stacks whose controller is not its own side (the
/// two callers of the original's 0x442880, which takes the first stack's controller and the
/// second stack's own side): it blocks the shooters of the side that controls it, not those of
/// its own side, and a multi-headed strike of the side that controls it reaches it.
// covers: BAT-MAG-026, BAT-RNG-001, BAT-ATK-003
#[test]
fn a_hypnotized_stack_blocks_and_is_struck_as_a_stack_of_its_own_side() {
    let r = ruleset();
    // Ids: 0 archer (attacker); 1 fixed (hypnotized) between it and 2 archer, 3 fixed (defender).
    let mut s = arena(
        &r,
        [None, None],
        &[("archer", 5, 7, 5)],
        &[("fixed", 5, 8, 5), ("archer", 5, 9, 5), ("fixed", 5, 14, 1)],
        1,
    );
    give(&mut s, &r, (1, "hypnotize"), 1);
    // The attacker's archers are blocked by the stack their side controls, and strike it.
    let commands = legal(&s, &r, BattleSide::Attacker);
    assert!(!commands.iter().any(|c| matches!(c, Command::Shoot { .. })));
    assert!(attacks(&commands, 1));
    let shot = Command::Shoot {
        target: StackId(3),
        at: at(&r, 14, 1),
    };
    assert_eq!(act(&mut s, &r, shot), Err(CommandError::CannotShoot));
    // The defender's archers beside it are free to shoot, though not at it.
    activate(&mut s, 2);
    let commands = legal(&s, &r, BattleSide::Defender);
    assert!(shoots(&commands, 0) && !shoots(&commands, 1));
    assert!(!commands.iter().any(|c| matches!(c, Command::Attack { .. })));
    // The attacker's hydra strikes the hypnotized stack beside it with the stack it attacks.
    let mut s = arena(
        &r,
        [None, None],
        &[("hydra", 10, 7, 5)],
        &[("fixed", 10, 8, 5), ("fixed", 10, 6, 4)],
        1,
    );
    give(&mut s, &r, (2, "hypnotize"), 1);
    let attack = Command::Attack {
        target: StackId(1),
        from: at(&r, 7, 5),
    };
    let events = act(&mut s, &r, attack).unwrap();
    assert_eq!(strikes(&events), [(0, 1), (0, 2)]);
    // The defender's hydra spares its own side's hypnotized stack beside it.
    let mut s = arena(
        &r,
        [None, None],
        &[("fixed", 10, 7, 5)],
        &[("hydra", 10, 8, 5), ("fixed", 10, 8, 4)],
        1,
    );
    assert_eq!(field(&r).distance(at(&r, 8, 4), at(&r, 8, 5)), 1);
    give(&mut s, &r, (2, "hypnotize"), 1);
    activate(&mut s, 1);
    let attack = Command::Attack {
        target: StackId(0),
        from: at(&r, 8, 5),
    };
    let events = act(&mut s, &r, attack).unwrap();
    assert_eq!(strikes(&events), [(1, 0)]);
}

// covers: BAT-MAG-026
#[test]
fn mass_versions_pass_over_hypnotized_stacks_and_expert_cure_only_with_q0012() {
    // The defender's hero cures with `mastery`; the result: whether its hypnotized stack still
    // is, and whether its other mass spell hastened it.
    let run = |profile: &str, mastery: u8, aim: CastTarget| {
        let r = ruleset_with(profile, |_| {});
        let mut s = arena(
            &r,
            [None, Some(adept(&r, &["cure", "haste"], mastery))],
            &[("fixed", 5, 2, 5)],
            &[("fixed", 5, 9, 5), ("fixed", 5, 10, 7)],
            1,
        );
        give(&mut s, &r, (1, "hypnotize"), 1);
        activate(&mut s, 2);
        act(&mut s, &r, cast(&r, "cure", aim)).unwrap();
        let hypnotized = has(&s, 1, Status::HYPNOTIZED);
        crate::turn::start_round(&mut s, &r, &mut Vec::<Event>::new());
        activate(&mut s, 2);
        let hasted = if mastery == 3 {
            give(&mut s, &r, (1, "hypnotize"), 1);
            act(&mut s, &r, cast(&r, "haste", CastTarget::None)).unwrap();
            on(&s, &r, 1).contains(&"haste".into())
        } else {
            false
        };
        (hypnotized, hasted)
    };
    // Expert Cure reaches the caster's hypnotized stack without quirk Q-0012, not with it; the
    // mass versions of the timed spells never do (Expert Dispel always does: the next test).
    assert_eq!(run("t:faithful", 3, CastTarget::None), (false, false));
    assert_eq!(run("t:hypnosis", 3, CastTarget::None), (true, false));
    // A single Cure from its owner ends Hypnotize in every profile.
    assert_eq!(run("t:faithful", 1, stack(1)), (false, false));
    assert_eq!(run("t:hypnosis", 1, stack(1)), (false, false));
}

/// E04b T11b review (F-r2a-2, F-r3-control-battlefield-3): Expert Dispel reaches hypnotized
/// stacks in every profile. The original's Dispel has a loop of its own over both sides' stacks
/// (the reviewers' reading of its handler 0x5A15CE), which tests no Hypnotize, unlike the mass
/// handler of the timed spells and Expert Cure's loop; VCMI's Dispel has no such exception
/// either. With quirk Q-0012 the engine passed over them, as if Dispel were Cure: a mass Dispel,
/// the natural answer to Hypnotize, did nothing to it.
// covers: BAT-MAG-016, BAT-MAG-026
#[test]
fn expert_dispel_reaches_hypnotized_stacks_in_every_profile() {
    for profile in ["t:faithful", "t:hypnosis"] {
        let r = ruleset_with(profile, |_| {});
        // Ids: 0 fixed (attacker); 1 fixed (hypnotized and hastened), 2 fixed (defender). What
        // stack 1 is left with after an Expert Dispel of the hero among `heroes`, cast in the
        // turn of stack `active`: its effects and its controller.
        let after = |heroes: [Option<BattleHero>; 2], active: u8| {
            let mut s = arena(
                &r,
                heroes,
                &[("fixed", 5, 2, 5)],
                &[("fixed", 5, 9, 5), ("fixed", 5, 10, 7)],
                1,
            );
            give(&mut s, &r, (1, "hypnotize"), 1);
            give(&mut s, &r, (1, "haste"), 1);
            assert_eq!(on(&s, &r, 1), ["haste", "hypnotize"]);
            activate(&mut s, active);
            act(&mut s, &r, cast(&r, "dispel", CastTarget::None)).unwrap();
            (on(&s, &r, 1), s.stacks()[1].controller())
        };
        let hero = || Some(adept(&r, &["dispel"], 3));
        // Whether its owner's hero casts or the hero of the side that controls it: its Hypnotize
        // ends, and its other effect too.
        let free = (Vec::new(), BattleSide::Defender);
        assert_eq!(
            [after([None, hero()], 2), after([hero(), None], 0)],
            [free.clone(), free],
            "{profile}"
        );
    }
}

// covers: BAT-MAG-026
#[test]
fn the_controllers_hero_resists_spells_on_a_hypnotized_stack() {
    let r = ruleset();
    let mut hero = adept(&r, &["bolt"], 1);
    hero.effects
        .push(h3_contracts::BattleEffect::MagicResistance {
            rate: Rate::bp(10_000),
        });
    let mut s = arena(
        &r,
        [Some(hero), None],
        &[("fixed", 5, 2, 5)],
        &[("fixed", 5, 9, 5), ("fixed", 5, 12, 5)],
        1,
    );
    assert_eq!(
        targets(&s, &r, BattleSide::Attacker, "bolt"),
        [stack(1), stack(2)]
    );
    // Hypnotized, it is still the defender's stack to the side tests, so the attacker's bolt
    // may be aimed at it; but its controller's hero resists all of it, so it is no target.
    give(&mut s, &r, (1, "hypnotize"), 1);
    assert_eq!(targets(&s, &r, BattleSide::Attacker, "bolt"), [stack(2)]);
}

// covers: BAT-MAG-025
#[test]
fn good_morale_gives_a_berserk_stack_another_turn() {
    let r = ruleset_with("t:faithful", |b| {
        b.morale_dice = 255;
        b.good_morale_chances = vec![255, 255, 255];
    });
    let good = |events: &[Event]| {
        events
            .iter()
            .filter(|e| matches!(e.kind, EventKind::GoodMorale { .. }))
            .count()
    };
    // Still berserk after walking, its extra turn is automatic too: two walks.
    let mut s = arena(
        &r,
        [None, None],
        &[("slow", 5, 2, 5)],
        &[("fixed", 5, 12, 5)],
        1,
    );
    give(&mut s, &r, (0, "berserk"), 1);
    s.stacks.as_mut_slice()[0].morale = 1;
    let events = berserk_now(&mut s, &r);
    assert_eq!(good(&events), 1);
    assert_eq!(s.stacks()[0].hex, at(&r, 6, 5));
    assert!(s.stacks()[0].flags.contains(StackFlags::ACTED));
    // After its attack Berserk is over: the extra turn is its side's to command.
    let mut s = arena(
        &r,
        [None, None],
        &[("fixed", 5, 5, 5)],
        &[("fixed", 5, 6, 5)],
        1,
    );
    give(&mut s, &r, (0, "berserk"), 1);
    s.stacks.as_mut_slice()[0].morale = 1;
    let events = berserk_now(&mut s, &r);
    assert_eq!(good(&events), 1);
    assert_eq!(s.active(), Some(StackId(0)));
    assert!(!has(&s, 0, Status::BERSERK));
    assert!(!s.stacks()[0].flags.contains(StackFlags::ACTED));
}
