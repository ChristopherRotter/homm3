//! Fear (E04b T7c3; BAT-MAG-060, quirk BAT-QUIRK-18), after the original's code
//! (`docs/formats/installation.md` §7.21, "Fear, read for T7c3").

use std::vec;
use std::vec::Vec;

use h3_content::{BattleParamsDef, Ruleset};

use super::spells::spell;
use super::support::*;
use crate::morale::frightened;
use crate::state::StackFlags;
use crate::{BattleState, Command, Event, EventKind, StackId};

/// Puts stack `id` under the other side's control (Hypnotize, BAT-MAG-026).
fn hypnotize(s: &mut BattleState, r: &Ruleset, id: u8) {
    let id_spell = spell(r, "hypnotize");
    let cast = crate::effects::TimedCast {
        id: id_spell,
        spell: r.spell(id_spell).unwrap(),
        mastery: 0,
        rounds: 5,
        own: crate::magic::SpellBonus::default(),
    };
    crate::effects::apply(s, r, cast, StackId(id), &mut Vec::<Event>::new());
}

/// The frozen stacks in `events`.
fn frozen(events: &[Event]) -> Vec<u8> {
    events
        .iter()
        .filter_map(|e| match e.kind {
            EventKind::Frightened { stack } => Some(stack.0),
            _ => None,
        })
        .collect()
}

/// Good morale that always triggers.
fn always(b: &mut BattleParamsDef) {
    b.morale_dice = 255;
    b.good_morale_chances = std::vec![255, 255, 255];
}

// covers: BAT-MAG-060, BAT-MOR-004
#[test]
fn fear_freezes_the_other_sides_stacks_but_not_at_the_battles_first_activation() {
    let r = ruleset();
    // The fast stack (speed 9) acts first: the battle's first activation has no freezing checks.
    let mut s = battle(
        &r,
        &[(0, "fast", 1), (1, "skeleton", 1)],
        &[(0, "azure", 1)],
        1,
    );
    assert_eq!(s.active(), Some(StackId(0)));
    // The azure never frightens itself, a skeleton has no morale and is never frozen.
    let events = act(&mut s, &r, Command::Defend).unwrap();
    assert_eq!((frozen(&events), s.active()), (vec![], Some(StackId(2))));
    let events = act(&mut s, &r, Command::Defend).unwrap();
    assert_eq!((frozen(&events), s.active()), (vec![], Some(StackId(1))));
    // In round 2 the fast stack is frozen at its pick: it loses its turn and has acted.
    let events = act(&mut s, &r, Command::Defend).unwrap();
    assert_eq!(frozen(&events), vec![0]);
    assert!(s.stacks()[0].flags.contains(StackFlags::ACTED));
    assert_eq!((s.round(), s.active()), (2, Some(StackId(2))));
}

// covers: BAT-MAG-060
#[test]
fn fear_takes_the_largest_chance_of_the_living_frighteners_outside_the_wait_phase() {
    let r = ruleset();
    let (mut scared, mut spooked) = (0, 0);
    for seed in 0..1000 {
        let mut s = battle(
            &r,
            &[(0, "serf", 1)],
            &[(0, "spook", 1), (1, "azure", 1)],
            seed,
        );
        // The azure's 99 %, the larger of the two chances.
        scared += usize::from(frightened(&mut s, &r, StackId(0)));
        // Outside the normal phase nothing freezes.
        s.wait_phase = true;
        assert!(!frightened(&mut s, &r, StackId(0)));
        s.wait_phase = false;
        // Without the azure, the spook's 1 %.
        s.stacks.as_mut_slice()[2].count = 0;
        spooked += usize::from(frightened(&mut s, &r, StackId(0)));
        // Without either, none.
        s.stacks.as_mut_slice()[1].count = 0;
        assert!(!frightened(&mut s, &r, StackId(0)));
    }
    assert!((970..1000).contains(&scared), "{scared}");
    assert!((1..30).contains(&spooked), "{spooked}");
}

/// E04b T11b review (F-robust-4, F-r2b-stack-spells-7 point 2): a pack cannot take every
/// stack's turns away for ever, which kept one command from returning (`turn::activate` picks
/// round after round until a stack awaits a command). The reviewer's case was a permanent Blind
/// on a side's only stack, whose certain Fear froze the other side's only stack at every pick:
/// neither compiles now, as a certain bad-morale freeze never did (BAT-MOR-004).
// covers: BAT-MAG-024, BAT-MAG-043, BAT-MAG-060
#[test]
fn a_pack_cannot_take_every_turn_away_for_ever() {
    use h3_content::{AbilityDef, CompileError, ContentKey, EffectDuration};
    let key = |name: &str| ContentKey::new("t", name).unwrap();
    // A timed effect that takes a stack's turns away must end.
    for spell in ["blind", "petrify", "paralysis"] {
        let refused = try_ruleset_from(
            "t:faithful",
            |_| {},
            |pack| {
                let def = pack.spells.iter_mut().find(|s| s.key == key(spell));
                def.unwrap().magic.duration = EffectDuration::Permanent;
            },
        );
        assert_eq!(
            refused.err(),
            Some(CompileError::InvalidContent {
                kind: "spell",
                key: std::format!("t:{spell}"),
                reason: "an effect that takes a stack's turns away cannot be permanent",
            })
        );
    }
    // A fear must leave the other side a chance to act.
    let refused = try_ruleset_from(
        "t:faithful",
        |_| {},
        |pack| {
            let azure = pack.creatures.iter_mut().find(|c| c.key == key("azure"));
            azure.unwrap().info.abilities = vec![AbilityDef::Fear { percent: 100 }];
        },
    );
    assert_eq!(
        refused.err(),
        Some(CompileError::InvalidCreature {
            key: std::string::String::from("t:azure"),
            reason: "fear percent must be 1..=99",
        })
    );
}

// covers: BAT-MAG-060
#[test]
fn a_hypnotized_azure_frightens_the_side_that_controls_it() {
    // BAT-QUIRK-18 (the rule): the frightened stack's side is its controller's, the azures count
    // by their real side.
    let r = ruleset();
    let mut s = battle(
        &r,
        &[(0, "serf", 1)],
        &[(0, "azure", 1), (1, "fixed", 1)],
        1,
    );
    hypnotize(&mut s, &r, 1);
    // The attacker controls the azure now, which still frightens the attacker's stacks and spares
    // the defender's.
    assert!(frightened(&mut s, &r, StackId(0)));
    assert!(!frightened(&mut s, &r, StackId(2)));
    // A hypnotized defender stack fights for the attacker: its own side's azure frightens it.
    hypnotize(&mut s, &r, 2);
    assert!(frightened(&mut s, &r, StackId(2)));
}

// covers: BAT-MAG-060, BAT-MOR-003
#[test]
fn fear_can_freeze_a_good_morale_turn_but_not_in_the_wait_phase() {
    let r = ruleset_with("t:faithful", always);
    // Castle pikemen (+1 morale) against a azure.
    let mut s = battle(&r, &[(0, "pikeman_c", 5)], &[(0, "azure", 1)], 1);
    place(&mut s, &r, 0, 2, 5);
    activate(&mut s, 0);
    let events = act(&mut s, &r, Command::Move { to: at(&r, 3, 5) }).unwrap();
    assert!(
        events
            .iter()
            .any(|e| e.kind == EventKind::GoodMorale { stack: StackId(0) })
    );
    assert_eq!((frozen(&events), s.active()), (vec![0], Some(StackId(1))));
    assert!(s.stacks()[0].flags.contains(StackFlags::ACTED));
    // A stack that acts from waiting keeps its good-morale turn.
    let mut s = battle(&r, &[(0, "pikeman_c", 5)], &[(0, "azure", 1)], 1);
    place(&mut s, &r, 0, 2, 5);
    activate(&mut s, 0);
    s.stacks.as_mut_slice()[0]
        .flags
        .set(StackFlags::WAITED, true);
    s.wait_phase = true;
    let events = act(&mut s, &r, Command::Move { to: at(&r, 3, 5) }).unwrap();
    assert_eq!((frozen(&events), s.active()), (vec![], Some(StackId(0))));
}

// covers: BAT-MAG-060, BAT-MOR-004
#[test]
fn the_first_pick_after_a_tactics_phase_is_checked() {
    let r = ruleset();
    // Round 1's first pick follows the tactics phase: the original asks with the checks on.
    let mut tactician = super::spells::mage(&r, &[]);
    tactician.tactics = 2;
    let setup = h3_contracts::BattleSetup {
        heroes: [Some(tactician), None],
        ..setup(&r, &[(0, "fast", 1)], &[(0, "azure", 1)])
    };
    let mut s = BattleState::new(&r, &setup, 1).unwrap();
    let events = act(&mut s, &r, Command::EndTactics).unwrap();
    assert_eq!((frozen(&events), s.active()), (vec![0], Some(StackId(1))));
}

// covers: BAT-MAG-060, BAT-MAG-025, BAT-MOR-003
#[test]
fn fear_can_freeze_a_berserk_stacks_good_morale_turn() {
    let r = ruleset_with("t:faithful", always);
    let mut s = battle(&r, &[(0, "slow", 5)], &[(0, "azure", 1)], 1);
    place(&mut s, &r, 0, 2, 5);
    place(&mut s, &r, 1, 12, 5);
    activate(&mut s, 0);
    let id = spell(&r, "berserk");
    let cast = crate::effects::TimedCast {
        id,
        spell: r.spell(id).unwrap(),
        mastery: 1,
        rounds: 5,
        own: crate::magic::SpellBonus::default(),
    };
    crate::effects::apply(&mut s, &r, cast, StackId(0), &mut Vec::<Event>::new());
    s.stacks.as_mut_slice()[0].morale = 1;
    let mut events = Vec::new();
    crate::turn::berserk_now(&mut s, &r, &mut events);
    assert!(
        events
            .iter()
            .any(|e| e.kind == EventKind::GoodMorale { stack: StackId(0) })
    );
    assert_eq!((frozen(&events), s.active()), (vec![0], Some(StackId(1))));
    assert!(s.stacks()[0].flags.contains(StackFlags::ACTED));
}
