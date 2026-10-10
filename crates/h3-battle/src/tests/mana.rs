//! The mana creatures (E04b T7c1; BAT-MAG-054 to BAT-MAG-056), after the original's code
//! (`docs/formats/installation.md` §7.21, "The mana creatures, re-read for T7c1"): the armies
//! change the heroes' spell costs, Familiars channel a share of the other hero's cost, Wraiths
//! drain the enemy hero's mana at their activations.

use std::vec::Vec;

use h3_content::{BattleParamsDef, Ruleset, SpellId};
use h3_contracts::{BattleHero, BattleSetup, BattleSide};

use super::support::*;
use crate::{
    BattleState, CastTarget, Command, CommandError, Event, EventKind, StackFlags, StackId, Status,
};

fn spell(r: &Ruleset, name: &str) -> SpellId {
    r.spell_by_key(&std::format!("t:{name}")).unwrap()
}

/// A hero with spell power 3, 15 mana and the test Magic Arrow (`bolt`: 5 mana without a
/// mastery, 4 from Basic; power × 10 + 10 damage).
fn mage(r: &Ruleset) -> BattleHero {
    let mut h = hero(r);
    h.power = 3;
    h.magic.spell_book = Some(std::vec![spell(r, "bolt")]);
    h
}

/// Starts a battle with `heroes`, returning it and the events of its start.
fn led(
    r: &Ruleset,
    heroes: [Option<BattleHero>; 2],
    attacker: &[Spec],
    defender: &[Spec],
) -> (BattleState, Vec<Event>) {
    let setup = BattleSetup {
        heroes,
        ..setup(r, attacker, defender)
    };
    let mut events = Vec::new();
    let s = BattleState::start(r, &setup, 1, &mut events).unwrap();
    (s, events)
}

/// The attacker's first stack takes the turn and its hero casts `bolt` at stack `target`.
fn bolt(s: &mut BattleState, r: &Ruleset, target: u8) -> Result<Vec<Event>, CommandError> {
    activate(s, 0);
    act(
        s,
        r,
        Command::Cast {
            spell: spell(r, "bolt"),
            target: CastTarget::Stack(StackId(target)),
            second: CastTarget::None,
        },
    )
}

fn mana(s: &BattleState, side: BattleSide) -> u16 {
    s.hero(side).unwrap().mana
}

/// The mana the attacker's hero pays for `bolt` at stack `target` now, without changing `s`.
fn cost(s: &BattleState, r: &Ruleset, target: u8) -> u16 {
    let mut probe = s.clone();
    let before = mana(&probe, BattleSide::Attacker);
    bolt(&mut probe, r, target).unwrap();
    before - mana(&probe, BattleSide::Attacker)
}

fn channeled(events: &[Event]) -> Vec<(BattleSide, u16)> {
    events
        .iter()
        .filter_map(|e| match e.kind {
            EventKind::ManaChanneled { side, mana } => Some((side, mana)),
            _ => None,
        })
        .collect()
}

fn drained(events: &[Event]) -> Vec<(u8, BattleSide, u16)> {
    events
        .iter()
        .filter_map(|e| match e.kind {
            EventKind::ManaDrained { stack, side, mana } => Some((stack.0, side, mana)),
            _ => None,
        })
        .collect()
}

/// Puts the test spell `name`'s effect on stack `id` for 5 rounds, as a cast at no mastery would.
fn put(s: &mut BattleState, r: &Ruleset, name: &str, id: u8) {
    let id_spell = spell(r, name);
    let cast = crate::effects::TimedCast {
        id: id_spell,
        spell: r.spell(id_spell).unwrap(),
        mastery: 0,
        rounds: 5,
        own: crate::magic::SpellBonus::default(),
    };
    crate::effects::apply(s, r, cast, StackId(id), &mut Vec::<Event>::new());
}

/// Puts stack `id` under the other side's control (Hypnotize, BAT-MAG-026).
fn hypnotize(s: &mut BattleState, r: &Ruleset, id: u8) {
    put(s, r, "hypnotize", id);
    let status = s.stacks()[usize::from(id)].spells.status;
    assert!(status.contains(Status::HYPNOTIZED));
}

/// Good morale that always triggers (and bad morale that nearly always does).
fn always(b: &mut BattleParamsDef) {
    b.morale_dice = 255;
    b.good_morale_chances = std::vec![255, 255, 255];
    b.bad_morale_chances = std::vec![254, 254, 254];
}

// covers: BAT-MAG-054
#[test]
fn the_armies_change_the_spell_cost() {
    let r = ruleset();
    let heroes = || [Some(mage(&r)), Some(mage(&r))];
    // No mana creature: 5 without a mastery.
    let (s, _) = led(&r, heroes(), &[(0, "fixed", 5)], &[(0, "fixed", 5)]);
    assert_eq!(cost(&s, &r, 1), 5);
    // An enemy Pegasus: +2; the attacker's own Pegasus raises only the defender's costs.
    let (s, _) = led(&r, heroes(), &[(0, "fixed", 5)], &[(0, "pegasus", 1)]);
    assert_eq!(cost(&s, &r, 1), 7);
    let (s, _) = led(&r, heroes(), &[(0, "pegasus", 1)], &[(0, "fixed", 5)]);
    assert_eq!(cost(&s, &r, 1), 5);
    assert_eq!(s.heroes[1].unwrap().magic.cost_change, 2);
    // Own Mages: −2; with the enemy's Pegasi too, both apply, each once however many stacks.
    let (s, _) = led(&r, heroes(), &[(0, "magus", 1)], &[(0, "fixed", 5)]);
    assert_eq!(cost(&s, &r, 1), 3);
    let (s, _) = led(
        &r,
        heroes(),
        &[(0, "magus", 1), (1, "magus", 1)],
        &[(0, "pegasus", 1), (1, "pegasus", 1)],
    );
    assert_eq!(cost(&s, &r, 2), 5);
    assert_eq!(s.heroes[1].unwrap().magic.cost_change, 0);
    let (s, _) = led(
        &r,
        heroes(),
        &[(0, "fixed", 5)],
        &[(0, "pegasus", 1), (1, "pegasus", 1)],
    );
    assert_eq!(s.heroes[0].unwrap().magic.cost_change, 2);
    assert_eq!(cost(&s, &r, 1), 7);
    // The largest reduction counts, and a cost stays at least 1: Basic's 4 − 5.
    let mut basic = mage(&r);
    basic.magic.masteries = masteries(&r, [1, 0, 0, 0]);
    let (s, _) = led(
        &r,
        [Some(basic), None],
        &[(0, "magus", 1), (1, "sage", 1)],
        &[(0, "fixed", 5)],
    );
    assert_eq!(s.heroes[0].unwrap().magic.cost_change, -5);
    assert_eq!(cost(&s, &r, 2), 1);
    // A spell that costs nothing stays free (Titan's Lightning Bolt).
    let mut free = r.spell(spell(&r, "bolt")).unwrap().clone();
    free.cost = [0; 4];
    assert_eq!(crate::mana::cost(&free, 0, 2), 0);
    assert_eq!(crate::mana::cost(&free, 0, -2), 0);
    // The changed cost decides whether the hero may cast.
    let mut poor = mage(&r);
    poor.magic.mana = 6;
    let (mut s, _) = led(
        &r,
        [Some(poor), None],
        &[(0, "fixed", 5)],
        &[(0, "pegasus", 1)],
    );
    activate(&mut s, 0);
    let before = s.clone();
    assert_eq!(bolt(&mut s, &r, 1), Err(CommandError::NotEnoughMana));
    assert_eq!(s, before);
    assert!(crate::castable_spells(&s, &r, BattleSide::Attacker).is_empty());
}

// covers: BAT-MAG-054
#[test]
fn a_dead_pegasus_still_raises_the_cost() {
    // BAT-QUIRK-15: the original reads the armies, which keep the dead stack until the battle's
    // end (batch 2a R6 round 3).
    let r = ruleset();
    let (mut s, _) = led(
        &r,
        [Some(mage(&r)), None],
        &[(0, "fixed", 5)],
        &[(0, "pegasus", 1), (1, "fixed", 5)],
    );
    // 40 damage kills the 30-hit-point Pegasus.
    bolt(&mut s, &r, 1).unwrap();
    assert!(!s.stacks()[1].is_alive());
    assert_eq!(mana(&s, BattleSide::Attacker), 8);
    act(&mut s, &r, Command::Defend).unwrap();
    act(&mut s, &r, Command::Defend).unwrap();
    assert_eq!(s.round(), 2);
    assert_eq!(cost(&s, &r, 2), 7);
}

// covers: BAT-MAG-055
#[test]
fn familiars_channel_a_share_of_the_other_heros_cost() {
    let r = ruleset();
    let heroes = || [Some(mage(&r)), Some(mage(&r))];
    // A cost of 5: the defender's hero gains 5 div 5 = 1 after the spell's effect, above its
    // maximum (15 of 10).
    let (mut s, _) = led(
        &r,
        heroes(),
        &[(0, "fixed", 5)],
        &[(0, "familiar", 3), (1, "fixed", 10)],
    );
    let events = bolt(&mut s, &r, 2).unwrap();
    assert_eq!(channeled(&events), [(BattleSide::Defender, 1)]);
    let damage = events
        .iter()
        .position(|e| matches!(e.kind, EventKind::SpellDamage { .. }))
        .unwrap();
    let channel = events
        .iter()
        .position(|e| matches!(e.kind, EventKind::ManaChanneled { .. }))
        .unwrap();
    assert!(damage < channel);
    assert_eq!(mana(&s, BattleSide::Defender), 16);
    assert_eq!(mana(&s, BattleSide::Attacker), 10);
    // A cost below 5 channels nothing (Basic: 4).
    let mut basic = mage(&r);
    basic.magic.masteries = masteries(&r, [1, 0, 0, 0]);
    let (mut s, _) = led(
        &r,
        [Some(basic), Some(mage(&r))],
        &[(0, "fixed", 5)],
        &[(0, "familiar", 3), (1, "fixed", 10)],
    );
    assert!(channeled(&bolt(&mut s, &r, 2).unwrap()).is_empty());
    assert_eq!(mana(&s, BattleSide::Defender), 15);
    // The largest share counts, once, rounded down: 50 % of 5.
    let (mut s, _) = led(
        &r,
        heroes(),
        &[(0, "fixed", 5)],
        &[(0, "familiar", 3), (1, "acolyte", 3), (2, "fixed", 10)],
    );
    assert_eq!(
        channeled(&bolt(&mut s, &r, 3).unwrap()),
        [(BattleSide::Defender, 2)]
    );
    // Familiars killed by the cast channel nothing (the channel follows the effect); nor do the
    // caster's own Familiars, nor Familiars without a hero.
    let (mut s, _) = led(
        &r,
        heroes(),
        &[(0, "fixed", 5)],
        &[(0, "familiar", 3), (1, "fixed", 10)],
    );
    let events = bolt(&mut s, &r, 1).unwrap();
    assert!(channeled(&events).is_empty());
    assert!(!s.stacks()[1].is_alive() && !s.is_over());
    let (mut s, _) = led(&r, heroes(), &[(0, "familiar", 3)], &[(0, "fixed", 10)]);
    assert!(channeled(&bolt(&mut s, &r, 1).unwrap()).is_empty());
    assert_eq!(mana(&s, BattleSide::Attacker), 10);
    let (mut s, _) = led(
        &r,
        [Some(mage(&r)), None],
        &[(0, "fixed", 5)],
        &[(0, "familiar", 3), (1, "fixed", 10)],
    );
    assert!(channeled(&bolt(&mut s, &r, 2).unwrap()).is_empty());
}

// covers: BAT-MAG-055
#[test]
fn a_hypnotized_familiar_channels_for_its_controller_wherever_its_record() {
    // BAT-QUIRK-16: the original walks the attacker's records only up to the defender's count
    // (1) and would miss the Familiar in record 2.
    let r = ruleset();
    let (mut s, _) = led(
        &r,
        [Some(mage(&r)), Some(mage(&r))],
        &[(0, "fixed", 5), (1, "fixed", 5), (2, "familiar", 3)],
        &[(0, "slow", 10)],
    );
    hypnotize(&mut s, &r, 2);
    let events = bolt(&mut s, &r, 3).unwrap();
    assert_eq!(channeled(&events), [(BattleSide::Defender, 1)]);
    assert_eq!(mana(&s, BattleSide::Defender), 16);
}

// covers: BAT-MAG-056
#[test]
fn a_wraith_drains_the_enemy_hero_at_its_activations_of_the_normal_phase() {
    let r = ruleset();
    // The Wraith (speed 7) acts first: 2 of the defender's 15 mana.
    let (mut s, events) = led(
        &r,
        [None, Some(mage(&r))],
        &[(0, "wraith", 5)],
        &[(0, "slow", 4)],
    );
    assert!(s.stacks()[0].has(crate::StackTraits::MANA_DRAIN));
    assert_eq!(drained(&events), [(0, BattleSide::Defender, 2)]);
    assert_eq!(mana(&s, BattleSide::Defender), 13);
    // The other stack's activation drains nothing; the next round's does.
    let events = act(&mut s, &r, Command::Defend).unwrap();
    assert!(drained(&events).is_empty());
    let events = act(&mut s, &r, Command::Defend).unwrap();
    assert_eq!(s.round(), 2);
    assert_eq!(drained(&events), [(0, BattleSide::Defender, 2)]);
    // A Wraith that waits acts again in the wait phase, without draining.
    act(&mut s, &r, Command::Wait).unwrap();
    let events = act(&mut s, &r, Command::Defend).unwrap();
    assert_eq!(s.active(), Some(StackId(0)));
    assert!(s.stacks()[0].flags.contains(StackFlags::WAITED));
    assert!(s.wait_phase);
    assert!(drained(&events).is_empty());
    assert_eq!(mana(&s, BattleSide::Defender), 11);
    // At least 0: the last mana, then nothing to take.
    s.heroes[1].as_mut().unwrap().view.mana = 1;
    let events = act(&mut s, &r, Command::Defend).unwrap();
    assert!(!s.wait_phase);
    assert_eq!(drained(&events), [(0, BattleSide::Defender, 1)]);
    act(&mut s, &r, Command::Defend).unwrap();
    let events = act(&mut s, &r, Command::Defend).unwrap();
    assert_eq!(s.round(), 4);
    assert!(drained(&events).is_empty());
    assert_eq!(mana(&s, BattleSide::Defender), 0);
    // Without an enemy hero nothing is drained, its own hero keeps its mana.
    let (s, events) = led(
        &r,
        [Some(mage(&r)), None],
        &[(0, "wraith", 5)],
        &[(0, "slow", 4)],
    );
    assert!(drained(&events).is_empty());
    assert_eq!(mana(&s, BattleSide::Attacker), 15);
}

// covers: BAT-MAG-056
#[test]
fn a_hypnotized_wraith_drains_its_owners_hero() {
    let r = ruleset();
    let (mut s, _) = led(
        &r,
        [Some(mage(&r)), Some(mage(&r))],
        &[(0, "slow", 4)],
        &[(0, "wraith", 5)],
    );
    hypnotize(&mut s, &r, 1);
    // Round 2: the Wraith, now the attacker's, drains the defender's hero.
    act(&mut s, &r, Command::Defend).unwrap();
    let events = act(&mut s, &r, Command::Defend).unwrap();
    assert_eq!(s.round(), 2);
    assert_eq!(drained(&events), [(1, BattleSide::Defender, 2)]);
    assert_eq!(mana(&s, BattleSide::Attacker), 13);
    assert_eq!(mana(&s, BattleSide::Defender), 13);
}

// covers: BAT-MAG-056, BAT-MOR-003
#[test]
fn a_wraith_drains_again_at_a_good_morale_turn_of_the_normal_phase_only() {
    let r = ruleset_with("t:faithful", always);
    // A castle army: morale +1, good morale after its first action of each round.
    let (mut s, events) = led(
        &r,
        [None, Some(mage(&r))],
        &[(0, "wraith", 5)],
        &[(0, "slow", 4)],
    );
    assert_eq!(drained(&events).len(), 1);
    // Round 1: it waits; in the wait phase it acts and gets good morale, and drains nothing.
    act(&mut s, &r, Command::Wait).unwrap();
    act(&mut s, &r, Command::Defend).unwrap();
    assert_eq!(s.active(), Some(StackId(0)));
    let events = act(&mut s, &r, Command::Move { to: at(&r, 3, 5) }).unwrap();
    assert!(
        events
            .iter()
            .any(|e| matches!(e.kind, EventKind::GoodMorale { .. }))
    );
    assert!(drained(&events).is_empty());
    assert_eq!(mana(&s, BattleSide::Defender), 13);
    // Round 2 (its extra turn defends): its activation and its good-morale turn of the normal
    // phase both drain.
    let events = act(&mut s, &r, Command::Defend).unwrap();
    assert_eq!(s.round(), 2);
    assert_eq!(drained(&events).len(), 1);
    let events = act(&mut s, &r, Command::Move { to: at(&r, 4, 5) }).unwrap();
    assert!(
        events
            .iter()
            .any(|e| matches!(e.kind, EventKind::GoodMorale { .. }))
    );
    assert_eq!(drained(&events), [(0, BattleSide::Defender, 2)]);
    assert_eq!(mana(&s, BattleSide::Defender), 9);
}

// covers: BAT-MAG-056, BAT-MOR-003
#[test]
fn a_wraith_that_lost_its_turns_drains_nothing() {
    // Blinded by the retaliation in its own action, it gets no activation at its good-morale
    // turn: the pick passes it over while it is blind.
    let r = ruleset_with("t:faithful", always);
    let (mut s, _) = led(
        &r,
        [None, Some(mage(&r))],
        &[(0, "wraith", 5)],
        &[(0, "blinder", 4)],
    );
    place(&mut s, &r, 0, 4, 5);
    place(&mut s, &r, 1, 5, 5);
    activate(&mut s, 0);
    let before = mana(&s, BattleSide::Defender);
    let from = at(&r, 4, 5);
    let events = act(
        &mut s,
        &r,
        Command::Attack {
            target: StackId(1),
            from,
        },
    )
    .unwrap();
    assert!(
        events
            .iter()
            .any(|e| e.kind == EventKind::GoodMorale { stack: StackId(0) })
    );
    assert!(drained(&events).is_empty());
    assert_eq!(mana(&s, BattleSide::Defender), before);
    assert_eq!(s.active(), Some(StackId(1)));
}

// covers: BAT-MAG-055
#[test]
fn familiars_channel_after_an_obstacle_spell_too() {
    // The cast's common tail follows every spell kind (the original's 0x5A2058).
    let r = ruleset();
    let mut h = mage(&r);
    h.magic.spell_book = Some(std::vec![spell(&r, "quicksand")]);
    let (mut s, _) = led(
        &r,
        [Some(h), Some(mage(&r))],
        &[(0, "fixed", 5)],
        &[(0, "familiar", 3)],
    );
    activate(&mut s, 0);
    let events = act(
        &mut s,
        &r,
        Command::Cast {
            spell: spell(&r, "quicksand"),
            target: CastTarget::None,
            second: CastTarget::None,
        },
    )
    .unwrap();
    assert!(
        events
            .iter()
            .any(|e| matches!(e.kind, EventKind::ObstaclePlaced { .. }))
    );
    assert_eq!(channeled(&events), [(BattleSide::Defender, 1)]);
}

// covers: BAT-MAG-056
#[test]
fn a_wraith_the_battle_does_not_activate_drains_nothing() {
    let r = ruleset();
    let heroes = || [None, Some(mage(&r))];
    // Blind: its turn passes.
    let (mut s, _) = led(&r, heroes(), &[(0, "wraith", 5)], &[(0, "slow", 4)]);
    put(&mut s, &r, "blind", 0);
    act(&mut s, &r, Command::Defend).unwrap();
    let events = act(&mut s, &r, Command::Defend).unwrap();
    assert_eq!(s.round(), 2);
    assert!(
        events
            .iter()
            .any(|e| matches!(e.kind, EventKind::TurnPassed { .. }))
    );
    assert!(drained(&events).is_empty());
    // Frozen by bad morale (its hero's morale −3, bad morale at 254/255).
    let r = ruleset_with("t:faithful", always);
    let mut gloomy = hero(&r);
    gloomy.effects.push(h3_contracts::BattleEffect::Morale {
        amount: -3,
        affects: h3_content::Affects::Own,
    });
    let (mut s, _) = led(
        &r,
        [Some(gloomy), Some(mage(&r))],
        &[(0, "wraith", 5)],
        &[(0, "slow", 4)],
    );
    // The battle's first activation has no freezing checks (BAT-MOR-004); in round 2 it freezes.
    act(&mut s, &r, Command::Defend).unwrap();
    let events = act(&mut s, &r, Command::Defend).unwrap();
    assert!(
        events
            .iter()
            .any(|e| matches!(e.kind, EventKind::BadMorale { stack } if stack.0 == 0))
    );
    assert!(drained(&events).is_empty());
    // In the tactics phase: nothing until round 1.
    let r = ruleset();
    let mut tactician = mage(&r);
    tactician.tactics = 2;
    let (mut s, events) = led(
        &r,
        [None, Some(tactician)],
        &[(0, "wraith", 5)],
        &[(0, "slow", 4)],
    );
    assert!(s.tactics().is_some());
    assert!(drained(&events).is_empty());
    let events = act(&mut s, &r, Command::EndTactics).unwrap();
    assert_eq!(drained(&events), [(0, BattleSide::Defender, 2)]);
}

// covers: BAT-MAG-056, BAT-MAG-025
#[test]
fn a_berserk_wraith_drains_before_its_automatic_turn() {
    let r = ruleset();
    let (mut s, _) = led(
        &r,
        [None, Some(mage(&r))],
        &[(0, "wraith", 5)],
        &[(0, "slow", 4)],
    );
    put(&mut s, &r, "berserk", 0);
    assert!(s.stacks()[0].spells.status.contains(Status::BERSERK));
    act(&mut s, &r, Command::Defend).unwrap();
    let events = act(&mut s, &r, Command::Defend).unwrap();
    let drain = events
        .iter()
        .position(|e| matches!(e.kind, EventKind::ManaDrained { .. }))
        .unwrap();
    let action = events
        .iter()
        .position(|e| {
            matches!(e.kind, EventKind::Moved { stack, .. } if stack.0 == 0)
                || matches!(e.kind, EventKind::Struck { attacker, .. } if attacker.0 == 0)
        })
        .unwrap();
    assert!(drain < action);
    assert_eq!(mana(&s, BattleSide::Defender), 11);
}
