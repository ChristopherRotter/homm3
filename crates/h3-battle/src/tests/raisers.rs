//! The raisers (E04b T7b3; BAT-MAG-051, BAT-MAG-052): the Archangel raises another stack of its
//! side for good, the Pit Lord raises Demons from a corpse of its side, each as its stack's
//! action, after the original's code (`docs/formats/installation.md` §7.21, "The raisers").

use std::vec;
use std::vec::Vec;

use h3_content::Ruleset;
use h3_contracts::{ArmyStack, BattleResult, BattleSide, StackOrigin, StackRecord};

use super::spells::{arena, mage, spell, stack};
use super::support::*;
use crate::CastSpell::Raise;
use crate::damage::apply_damage;
use crate::effects::{self, Status};
use crate::raise::mark_corpses;
use crate::state::StackFlags;
use crate::{
    BattleState, CREATED_SLOT, CastTarget, CastTargets, Command, CommandError, Event, EventKind,
    NO_CORPSE, StackId, StackTraits, creature_cast_spell, creature_cast_targets, stack_spell,
};

/// The stacks the active stack of `side` may raise now, by id.
fn targets(s: &BattleState, r: &Ruleset, side: BattleSide) -> Vec<u8> {
    let mut out = CastTargets::new();
    creature_cast_targets(s, r, side, &mut out);
    out.iter()
        .map(|t| match *t {
            CastTarget::Stack(id) => id.0,
            other => panic!("not a stack: {other:?}"),
        })
        .collect()
}

/// The command of a raise of stack `id`.
fn at_stack(id: u8) -> Command {
    Command::CreatureCast { target: stack(id) }
}

/// Deals `damage` to stack `id` as a strike would, and marks the corpses.
fn hurt(s: &mut BattleState, r: &Ruleset, id: u8, damage: u64) {
    let mut events = Vec::new();
    apply_damage(s.at_mut(StackId(id)), damage);
    effects::damaged(s, r, StackId(id), &mut events);
    mark_corpses(s);
}

/// The raised stacks as `(stack, creatures, temporary)`.
fn raised(events: &[Event]) -> Vec<(u8, u32, bool)> {
    events
        .iter()
        .filter_map(|e| match e.kind {
            EventKind::SpellRaised {
                target,
                creatures,
                temporary,
                ..
            } => Some((target.0, creatures, temporary)),
            _ => None,
        })
        .collect()
}

/// The stacks risen from corpses as `(stack, corpse, count, hex)`.
fn risen(events: &[Event]) -> Vec<(u8, u8, u32, u8)> {
    events
        .iter()
        .filter_map(|e| match e.kind {
            EventKind::RoseFromCorpse {
                stack,
                corpse,
                count,
                hex,
                ..
            } => Some((stack.0, corpse.0, count, hex.0)),
            _ => None,
        })
        .collect()
}

// covers: BAT-MAG-051, BAT-MAG-048
#[test]
fn an_archangel_raises_another_stack_of_its_side_for_good() {
    let r = ruleset();
    let mut s = arena(
        &r,
        [None, None],
        &[("herald", 2, 5, 5), ("serf", 10, 8, 3), ("serf", 4, 8, 7)],
        &[("fixed", 3, 13, 5)],
        1,
    );
    assert_eq!(stack_spell(&s, &r, StackId(0)), Some(Raise));
    assert_eq!(
        creature_cast_spell(&s, &r, BattleSide::Attacker),
        Some(Raise)
    );
    // Only a stack of its side that lost creatures: not the full serfs, not the enemy.
    hurt(&mut s, &r, 2, 30);
    hurt(&mut s, &r, 3, 10);
    assert_eq!(targets(&s, &r, BattleSide::Attacker), vec![2]);
    // 100 hit points per Archangel: 200 bring back the three serfs, for good.
    let events = act(&mut s, &r, at_stack(2)).unwrap();
    assert_eq!(
        events[0].kind,
        EventKind::CreatureRaise {
            stack: StackId(0),
            target: StackId(2),
        }
    );
    assert_eq!(raised(&events), vec![(2, 3, false)]);
    assert_eq!((s.stacks()[2].count, s.temporary(StackId(2))), (4, 0));
    // Its one cast is used and its turn is over.
    assert_eq!(s.casts_left(StackId(0)), 0);
    assert!(s.stacks()[0].flags.contains(StackFlags::ACTED));
    assert_ne!(s.active(), Some(StackId(0)));
}

// covers: BAT-MAG-051, BAT-MAG-035
#[test]
fn an_archangel_raises_a_corpse_and_needs_the_health_of_one_creature() {
    let r = ruleset();
    let mut s = arena(
        &r,
        [None, None],
        &[
            ("herald", 2, 5, 5),
            ("serf", 10, 8, 3),
            ("ogre", 3, 8, 7),
            ("seraph", 2, 8, 9),
        ],
        &[("fixed", 3, 13, 5)],
        1,
    );
    // The serfs lie dead; ogres (health 40) and a seraph (health 200) are wounded; the herald
    // itself lost one of its two too, but never raises itself (the original's cursor).
    hurt(&mut s, &r, 1, 1000);
    hurt(&mut s, &r, 2, 40);
    hurt(&mut s, &r, 3, 200);
    hurt(&mut s, &r, 0, 50);
    assert_eq!(s.stacks()[1].count, 0);
    // 100 hit points from the one herald left: the serfs' corpse and the ogres, not the seraph.
    assert_eq!(targets(&s, &r, BattleSide::Attacker), vec![1, 2]);
    for bad in [at_stack(3), at_stack(0), at_stack(4), at_stack(9)] {
        assert_eq!(act(&mut s, &r, bad), Err(CommandError::InvalidSpellTarget));
    }
    let events = act(&mut s, &r, at_stack(1)).unwrap();
    // A dead stack comes back with ten of its creatures (the hit points' count, at most its
    // start count).
    assert_eq!(raised(&events), vec![(1, 10, false)]);
    assert_eq!((s.stacks()[1].count, s.corpse(StackId(1))), (10, 0));
}

// covers: BAT-MAG-052, BAT-MAG-037, BAT-END-003
#[test]
fn a_pit_lord_raises_imps_from_a_corpse_of_its_side_into_its_army_slot() {
    let r = ruleset();
    let mut s = arena(
        &r,
        [None, None],
        &[("fiend", 2, 5, 5), ("serf", 8, 8, 3), ("serf", 1, 8, 7)],
        &[("fixed", 3, 13, 5), ("fixed", 2, 13, 7)],
        1,
    );
    assert_eq!(stack_spell(&s, &r, StackId(0)), Some(Raise));
    // No corpse yet; living stacks are no target.
    assert!(targets(&s, &r, BattleSide::Attacker).is_empty());
    hurt(&mut s, &r, 1, 1000);
    hurt(&mut s, &r, 3, 1000);
    // The serfs' corpse, not the enemy's.
    assert_eq!(targets(&s, &r, BattleSide::Attacker), vec![1]);
    let serfs_hex = s.stacks()[1].hex.0;
    let events = act(&mut s, &r, at_stack(1)).unwrap();
    assert_eq!(
        events[0].kind,
        EventKind::CreatureRaise {
            stack: StackId(0),
            target: StackId(1),
        }
    );
    // min(8, min(50 × 2, 8 × 10) div 10) = 8 imps on the serfs' hex, a new stack of their army
    // slot; the corpse is gone.
    assert_eq!(risen(&events), vec![(5, 1, 8, serfs_hex)]);
    let imps = s.stacks()[5];
    assert_eq!(
        (
            imps.creature,
            imps.side,
            imps.slot,
            imps.count,
            imps.base_count
        ),
        (
            r.creature_by_key("t:imp").unwrap(),
            BattleSide::Attacker,
            1,
            8,
            8
        )
    );
    assert!(imps.has(StackTraits::RAISED) && !imps.is_created());
    assert_eq!(imps.turn_slot(), CREATED_SLOT);
    assert_eq!(s.corpse(StackId(1)), NO_CORPSE);
    assert_eq!(s.casts_left(StackId(0)), 0);
    // Their side wins: the outcome reports them in the dead serfs' slot.
    hurt(&mut s, &r, 4, 1000);
    crate::combat::check_end(&mut s, &mut Vec::<Event>::new());
    let outcome = s.outcome().unwrap();
    assert_eq!(outcome.result, BattleResult::Victory(BattleSide::Attacker));
    let (fiend, serf, imp) = (
        r.creature_by_key("t:fiend").unwrap(),
        r.creature_by_key("t:serf").unwrap(),
        r.creature_by_key("t:imp").unwrap(),
    );
    let record = |origin, creature, start, end| StackRecord {
        origin,
        creature,
        start,
        end,
    };
    assert_eq!(
        outcome.stacks[0],
        vec![
            record(StackOrigin::Army(0), fiend, 2, 2),
            record(StackOrigin::Army(1), serf, 8, 0),
            record(StackOrigin::Army(2), serf, 1, 1),
            record(StackOrigin::Raised(1), imp, 8, 8),
        ]
    );
    let army = |slot, creature, count| ArmyStack {
        slot,
        creature,
        count,
    };
    assert_eq!(
        outcome.survivors[0].stacks,
        vec![army(0, fiend, 2), army(1, imp, 8), army(2, serf, 1)]
    );
}

// covers: BAT-MAG-038, BAT-MAG-052
#[test]
fn a_clone_of_raised_imps_is_not_raised_from_a_corpse() {
    let r = ruleset();
    let mut s = arena(
        &r,
        [None, None],
        &[("fiend", 2, 5, 5), ("serf", 8, 8, 3)],
        &[("fixed", 3, 13, 5)],
        1,
    );
    hurt(&mut s, &r, 1, 1000);
    act(&mut s, &r, at_stack(1)).unwrap();
    let imps = s.stacks()[3];
    assert!(imps.has(StackTraits::RAISED) && !imps.is_created());
    crate::created::clone(&mut s, &r, (StackId(3), 3), &mut Vec::<Event>::new());
    // The clone is a stack the battle created, not one raised from a corpse (it kept its
    // original's trait: review r4a F-8); its other traits are its original's.
    let clone = s.stacks()[4];
    assert!(clone.has(StackTraits::CLONE) && clone.is_created());
    assert!(!clone.has(StackTraits::RAISED));
    let created = StackTraits::SUMMONED.bits() | StackTraits::CLONE.bits();
    assert_eq!(
        clone.traits.bits(),
        (imps.traits.bits() & !StackTraits::RAISED.bits()) | created
    );
    assert!(s.stacks()[3].has(StackTraits::RAISED));
}

// covers: BAT-MAG-052
#[test]
fn a_pit_lord_needs_a_living_creatures_corpse_on_free_hexes_worth_one_creature() {
    let r = ruleset();
    let mut s = arena(
        &r,
        [None, None],
        &[
            ("fiend", 1, 5, 5),
            ("wisp", 1, 8, 1),
            ("skeleton", 5, 8, 3),
            ("serf", 5, 8, 5),
            ("serf", 5, 8, 7),
            ("fixed", 3, 8, 9),
        ],
        &[("fixed", 3, 13, 5)],
        1,
    );
    for id in 1..=4 {
        hurt(&mut s, &r, id, 1000);
    }
    // The undead skeletons' corpse is no target; the wisp's (health 4) is worth no imp: min(1,
    // min(50, 4) div 10) = 0. A living stack on the second serfs' corpse keeps it down.
    let fixed = s.stacks()[5].hex;
    s.at_mut(StackId(5)).hex = s.stacks()[4].hex;
    assert_eq!(targets(&s, &r, BattleSide::Attacker), vec![3]);
    s.at_mut(StackId(5)).hex = fixed;
    assert_eq!(targets(&s, &r, BattleSide::Attacker), vec![3, 4]);
    // A corpse that does not qualify lying on top leaves the one below it a target; one that
    // does hides it.
    let below = s.stacks()[3].hex;
    let top = s.corpse(StackId(4));
    s.at_mut(StackId(2)).hex = below;
    s.raised.corpse[2] = top + 1;
    assert_eq!(targets(&s, &r, BattleSide::Attacker), vec![3, 4]);
    s.at_mut(StackId(4)).hex = below;
    s.raised.corpse[4] = top + 2;
    assert_eq!(targets(&s, &r, BattleSide::Attacker), vec![4]);
    // Imps from five serfs: min(5, min(50, 50) div 10) = 5.
    let events = act(&mut s, &r, at_stack(4)).unwrap();
    assert_eq!(risen(&events), vec![(7, 4, 5, below.0)]);
}

// covers: BAT-MAG-052, BAT-MAG-037
#[test]
fn a_pit_lord_needs_a_slot_for_the_new_stack() {
    // The original's stack creation finds no record with 20 in use and no dead created stack,
    // and the raise then crashes (BAT-QUIRK-12): the engine refuses it. A side limit of four.
    let r = ruleset_with("t:faithful", |b| b.side_stack_limit = 4);
    let mut s = arena(
        &r,
        [Some(mage(&r, &["sprites"])), None],
        &[("fiend", 1, 5, 5), ("serf", 5, 8, 3), ("fixed", 3, 8, 7)],
        &[("fixed", 3, 13, 5)],
        1,
    );
    set_masteries(&mut s, 0, &r, [0, 1, 0, 0]);
    s.heroes[0].as_mut().unwrap().view.mana = 50;
    hurt(&mut s, &r, 1, 1000);
    assert_eq!(targets(&s, &r, BattleSide::Attacker), vec![1]);
    let summon = Command::Cast {
        spell: spell(&r, "sprites"),
        target: CastTarget::None,
        second: CastTarget::None,
    };
    act(&mut s, &r, summon).unwrap();
    assert_eq!(s.stacks().len(), 5);
    // Four stacks of the side, none a dead created one: no slot.
    assert!(targets(&s, &r, BattleSide::Attacker).is_empty());
    assert_eq!(
        act(&mut s, &r, at_stack(1)),
        Err(CommandError::InvalidSpellTarget)
    );
    // The sprites die: the imps take their slot.
    hurt(&mut s, &r, 4, 1000);
    assert_eq!(targets(&s, &r, BattleSide::Attacker), vec![1]);
    let events = act(&mut s, &r, at_stack(1)).unwrap();
    assert_eq!(risen(&events)[0].0, 4);
    assert!(s.stacks()[4].has(StackTraits::RAISED) && !s.stacks()[4].is_created());
}

/// E04b T11b review (F-r4a-4): the record of a created stack whose rounds ran out is no slot for
/// a Pit Lord's stack either: the original's stack creation takes only an empty record or a dead
/// summoned stack with no creatures (0x47A100), and a stack that vanished at the end of its
/// rounds keeps its count. The engine took any dead created stack's record.
// covers: BAT-MAG-052, BAT-MAG-037
#[test]
fn a_pit_lord_finds_no_slot_in_a_clone_whose_rounds_ran_out() {
    let r = ruleset_with("t:faithful", |b| b.side_stack_limit = 4);
    // The fiend, dead serfs and three more serfs with their clone (stack 4): four stacks, the
    // side's limit.
    let start = || {
        let mut hero = mage(&r, &["copy"]);
        hero.magic.masteries = masteries(&r, [1, 0, 0, 0]);
        let mut s = arena(
            &r,
            [Some(hero), None],
            &[("fiend", 1, 5, 5), ("serf", 5, 8, 3), ("serf", 3, 8, 7)],
            &[("fixed", 3, 13, 5)],
            1,
        );
        hurt(&mut s, &r, 1, 1000);
        let copy = Command::Cast {
            spell: spell(&r, "copy"),
            target: stack(2),
            second: CastTarget::None,
        };
        act(&mut s, &r, copy).unwrap();
        assert!(s.stacks()[4].has(StackTraits::CLONE));
        assert!(targets(&s, &r, BattleSide::Attacker).is_empty());
        s
    };
    // The clone's rounds run out: its record stays in use, and the fiend still finds no slot.
    let mut s = start();
    for _ in 0..3 {
        crate::turn::start_round(&mut s, &r, &mut Vec::<Event>::new());
    }
    activate(&mut s, 0);
    assert!(!s.stacks()[4].is_alive());
    assert!(targets(&s, &r, BattleSide::Attacker).is_empty());
    assert_eq!(
        act(&mut s, &r, at_stack(1)),
        Err(CommandError::InvalidSpellTarget)
    );
    // A killed clone leaves its record: the imps take it.
    let mut s = start();
    hurt(&mut s, &r, 4, 1);
    assert_eq!(targets(&s, &r, BattleSide::Attacker), vec![1]);
    let events = act(&mut s, &r, at_stack(1)).unwrap();
    assert_eq!(risen(&events)[0].0, 4);
}

// covers: BAT-MAG-052, BAT-MAG-048
#[test]
fn a_hypnotized_pit_lord_raises_its_controllers_dead_for_its_controller() {
    let r = ruleset();
    let mut s = arena(
        &r,
        [None, None],
        &[("serf", 5, 2, 3), ("fixed", 3, 2, 7)],
        &[("fiend", 2, 13, 5), ("serf", 5, 13, 7)],
        1,
    );
    hurt(&mut s, &r, 0, 1000);
    hurt(&mut s, &r, 3, 1000);
    let st = &mut s.stacks.as_mut_slice()[2];
    st.spells.status = st.spells.status.with(Status::HYPNOTIZED, true);
    activate(&mut s, 2);
    assert_eq!(s.side_to_act(), Some(BattleSide::Attacker));
    assert_eq!(targets(&s, &r, BattleSide::Attacker), vec![0]);
    let events = act(&mut s, &r, at_stack(0)).unwrap();
    let imps = s.stacks()[usize::from(risen(&events)[0].0)];
    assert_eq!((imps.side, imps.slot), (BattleSide::Attacker, 0));
}

// covers: BAT-MAG-052, BAT-MAG-035
#[test]
fn imps_leave_a_corpse_from_which_imps_rise_again() {
    let r = ruleset();
    let mut s = arena(
        &r,
        [None, None],
        &[("fiend", 2, 5, 3), ("fiend", 2, 5, 7), ("serf", 8, 8, 5)],
        &[("fixed", 3, 13, 5)],
        1,
    );
    hurt(&mut s, &r, 2, 1000);
    act(&mut s, &r, at_stack(2)).unwrap();
    hurt(&mut s, &r, 4, 1000);
    assert!(s.corpse(StackId(4)) > 0 && s.corpse(StackId(4)) != NO_CORPSE);
    activate(&mut s, 1);
    assert_eq!(targets(&s, &r, BattleSide::Attacker), vec![4]);
    let events = act(&mut s, &r, at_stack(4)).unwrap();
    assert_eq!(risen(&events)[0].1, 4);
    let again = s.stacks()[usize::from(risen(&events)[0].0)];
    assert_eq!((again.slot, again.count), (2, 8));
}
