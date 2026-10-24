//! Tests of the army commands (E08 T50; ADV-ARM-001 to -007): the plan on one army, between two
//! and with the group; the split's range; the state's command, query, formation and flag.

extern crate std;

use std::format;
use std::vec;
use std::vec::Vec;

use h3_content::{CreatureTypeId, Formation, Ruleset};
use h3_contracts::{ArmyStack, HeroId, PlayerId};
use h3_kernel::hash::hash_of;
use proptest::prelude::{prop_assert, prop_assert_eq, proptest};
use proptest::strategy::Strategy;

use super::*;
use crate::ruleset::AdventureRuleset;
use crate::setup::Pos;
use crate::state::AdventureState;
use crate::testing::{GNOLL, MAX_STACK, PEASANT, PIKEMAN, WOLF, hero_at, movement_rules, world};

const RED: PlayerId = PlayerId::new(0);
const BLUE: PlayerId = PlayerId::new(1);
/// Seven slots and the test rulesets' most.
const RULES: (u8, u32) = (7, MAX_STACK);

fn stack(slot: u8, creature: CreatureTypeId, count: u32) -> ArmyStack {
    ArmyStack {
        slot,
        creature,
        count,
    }
}

fn hero(id: u16, slot: u8) -> ArmySlot {
    ArmySlot {
        army: ArmyRef::Hero(HeroId::new(id)),
        slot,
    }
}

fn group(slot: u8) -> ArmySlot {
    ArmySlot {
        army: ArmyRef::Group,
        slot,
    }
}

fn moved(from: ArmySlot, to: ArmySlot) -> ArmyCommand {
    ArmyCommand::Move { from, to }
}

fn split(from: ArmySlot, to: ArmySlot, count: u32) -> ArmyCommand {
    ArmyCommand::Split { from, to, count }
}

/// The plan of `command` on one hero's army.
fn one(command: ArmyCommand, army: &[ArmyStack]) -> Result<Vec<ArmyStack>, ArmyCommandError> {
    plan(&command, (army, true), None, RULES).map(|(army, other)| {
        assert_eq!(other, None);
        army
    })
}

/// The plan of `command` from a hero's army (`hero`) or the group's onto another army.
fn two(
    command: ArmyCommand,
    (source, hero): (&[ArmyStack], bool),
    target: &[ArmyStack],
) -> Result<(Vec<ArmyStack>, Vec<ArmyStack>), ArmyCommandError> {
    plan(&command, (source, hero), Some(target), RULES)
        .map(|(source, target)| (source, target.unwrap()))
}

// covers: ADV-ARM-001, ADV-ARM-002
#[test]
fn a_move_within_an_army_fills_an_empty_slot_merges_or_swaps() {
    let army = [
        stack(0, PEASANT, 10),
        stack(2, GNOLL, 3),
        stack(4, PEASANT, 5),
    ];
    // Into an empty slot: the stack keeps its count, the army its slot order.
    assert_eq!(
        one(moved(hero(1, 4), hero(1, 6)), &army),
        Ok(vec![
            stack(0, PEASANT, 10),
            stack(2, GNOLL, 3),
            stack(6, PEASANT, 5)
        ])
    );
    assert_eq!(
        one(moved(hero(1, 2), hero(1, 1)), &army),
        Ok(vec![
            stack(0, PEASANT, 10),
            stack(1, GNOLL, 3),
            stack(4, PEASANT, 5)
        ])
    );
    // Onto a stack of the same creature: one stack in the target's slot.
    assert_eq!(
        one(moved(hero(1, 4), hero(1, 0)), &army),
        Ok(vec![stack(0, PEASANT, 15), stack(2, GNOLL, 3)])
    );
    // Onto another creature: the two swap.
    assert_eq!(
        one(moved(hero(1, 0), hero(1, 2)), &army),
        Ok(vec![
            stack(0, GNOLL, 3),
            stack(2, PEASANT, 10),
            stack(4, PEASANT, 5)
        ])
    );
    // A single stack moves within its army.
    assert_eq!(
        one(moved(hero(1, 0), hero(1, 3)), &[stack(0, WOLF, 1)]),
        Ok(vec![stack(3, WOLF, 1)])
    );
}

// covers: ADV-ARM-001, ADV-ARM-002, ADV-ARM-007
#[test]
fn a_move_is_refused_off_the_slots_from_nothing_onto_itself_or_past_the_most() {
    let army = [stack(0, PEASANT, MAX_STACK - 5), stack(1, PEASANT, 6)];
    let refused = [
        (moved(hero(1, 7), hero(1, 0)), ArmyCommandError::NoSlot(7)),
        (moved(hero(1, 0), hero(1, 9)), ArmyCommandError::NoSlot(9)),
        (moved(hero(1, 3), hero(1, 0)), ArmyCommandError::NoStack(3)),
        (moved(hero(1, 0), hero(1, 0)), ArmyCommandError::SameSlot),
        (
            moved(hero(1, 1), hero(1, 0)),
            ArmyCommandError::TooManyCreatures,
        ),
    ];
    for (command, error) in refused {
        assert_eq!(one(command, &army), Err(error), "{command:?}");
    }
    // Up to the most it merges.
    assert_eq!(
        one(
            moved(hero(1, 1), hero(1, 0)),
            &[stack(0, PEASANT, MAX_STACK - 6), stack(1, PEASANT, 6)]
        ),
        Ok(vec![stack(0, PEASANT, MAX_STACK)])
    );
}

// covers: ADV-ARM-002, ADV-ARM-005
#[test]
fn between_two_armies_a_hero_keeps_its_last_stack_and_counts_stay_whole() {
    let red = [stack(0, PEASANT, 10), stack(3, GNOLL, 3)];
    let blue = [stack(0, WOLF, 2), stack(1, PEASANT, 4)];
    let both = |command, red: &[ArmyStack], blue: &[ArmyStack]| two(command, (red, true), blue);
    // Into an empty slot of the other hero.
    assert_eq!(
        both(moved(hero(1, 3), hero(2, 5)), &red, &blue),
        Ok((
            vec![stack(0, PEASANT, 10)],
            vec![stack(0, WOLF, 2), stack(1, PEASANT, 4), stack(5, GNOLL, 3)]
        ))
    );
    // Into its stack of the creature.
    assert_eq!(
        both(moved(hero(1, 0), hero(2, 1)), &red, &blue),
        Ok((
            vec![stack(3, GNOLL, 3)],
            vec![stack(0, WOLF, 2), stack(1, PEASANT, 14)]
        ))
    );
    // A swap, each stack taking the other's slot.
    assert_eq!(
        both(moved(hero(1, 3), hero(2, 0)), &red, &blue),
        Ok((
            vec![stack(0, PEASANT, 10), stack(3, WOLF, 2)],
            vec![stack(0, GNOLL, 3), stack(1, PEASANT, 4)]
        ))
    );
    // A hero's last stack neither moves to an empty slot nor merges away; it swaps.
    let alone = [stack(2, GNOLL, 3)];
    for command in [moved(hero(1, 2), hero(2, 6)), moved(hero(1, 2), hero(2, 1))] {
        let blue = [stack(0, WOLF, 2), stack(1, GNOLL, 4)];
        assert_eq!(
            both(command, &alone, &blue),
            Err(ArmyCommandError::LastStack)
        );
    }
    assert_eq!(
        both(moved(hero(1, 2), hero(2, 0)), &alone, &blue),
        Ok((
            vec![stack(2, WOLF, 2)],
            vec![stack(0, GNOLL, 3), stack(1, PEASANT, 4)]
        ))
    );
    // The same slot of two armies is two slots; a merge past the most is refused.
    let full = [stack(0, PEASANT, MAX_STACK)];
    assert_eq!(
        both(moved(hero(1, 0), hero(2, 0)), &red, &full),
        Err(ArmyCommandError::TooManyCreatures)
    );
    // A-15 fixed: counts above 16 bits keep their width when moved or swapped (ADV-ARM-005).
    let big = [stack(0, PEASANT, 40_000), stack(1, GNOLL, 1)];
    let other = [stack(0, WOLF, 50_000)];
    let wide = (7, 1_000_000);
    assert_eq!(
        plan(
            &moved(hero(1, 0), hero(2, 0)),
            (&big, true),
            Some(&other),
            wide
        ),
        Ok((
            vec![stack(0, WOLF, 50_000), stack(1, GNOLL, 1)],
            Some(vec![stack(0, PEASANT, 40_000)])
        ))
    );
    assert_eq!(
        plan(
            &moved(hero(1, 0), hero(2, 3)),
            (&big, true),
            Some(&other),
            wide
        ),
        Ok((
            vec![stack(1, GNOLL, 1)],
            Some(vec![stack(0, WOLF, 50_000), stack(3, PEASANT, 40_000)])
        ))
    );
}

// covers: ADV-ARM-002, ADV-ARM-004
#[test]
fn the_group_has_no_protected_stack() {
    let found = [stack(0, PIKEMAN, 30)];
    let red = [stack(0, PEASANT, 10), stack(1, PIKEMAN, 2)];
    // Its last stack goes to a hero's empty slot or merges into a hero's stack.
    assert_eq!(
        two(moved(group(0), hero(1, 4)), (&found, false), &red),
        Ok((
            vec![],
            vec![
                stack(0, PEASANT, 10),
                stack(1, PIKEMAN, 2),
                stack(4, PIKEMAN, 30)
            ]
        ))
    );
    assert_eq!(
        two(moved(group(0), hero(1, 1)), (&found, false), &red),
        Ok((vec![], vec![stack(0, PEASANT, 10), stack(1, PIKEMAN, 32)]))
    );
    // A hero's stack goes to the group, but not its last.
    assert_eq!(
        two(moved(hero(1, 0), group(5)), (&red, true), &found),
        Ok((
            vec![stack(1, PIKEMAN, 2)],
            vec![stack(0, PIKEMAN, 30), stack(5, PEASANT, 10)]
        ))
    );
    assert_eq!(
        two(
            moved(hero(1, 0), group(5)),
            (&[stack(0, PEASANT, 1)], true),
            &found
        ),
        Err(ArmyCommandError::LastStack)
    );
    // The group's last stack is dismissed; a hero's is not.
    let dismiss = |at| ArmyCommand::Dismiss { at };
    assert_eq!(
        plan(&dismiss(group(0)), (&found, false), None, RULES),
        Ok((vec![], None))
    );
    assert_eq!(
        one(dismiss(hero(1, 0)), &[stack(0, PEASANT, 1)]),
        Err(ArmyCommandError::LastStack)
    );
}

// covers: ADV-ARM-004
#[test]
fn a_dismissed_stack_leaves_its_army() {
    let army = [stack(0, PEASANT, 10), stack(3, GNOLL, 3)];
    let dismiss = |slot| ArmyCommand::Dismiss { at: hero(1, slot) };
    assert_eq!(one(dismiss(0), &army), Ok(vec![stack(3, GNOLL, 3)]));
    assert_eq!(one(dismiss(1), &army), Err(ArmyCommandError::NoStack(1)));
    assert_eq!(one(dismiss(7), &army), Err(ArmyCommandError::NoSlot(7)));
}

// covers: ADV-ARM-003, ADV-ARM-007
#[test]
fn a_split_leaves_both_stacks_within_one_and_the_most() {
    // The original's [1, t − 1] (E08 decision 98), narrowed by the most.
    assert_eq!(split_range(10, 0, MAX_STACK), 1..=9);
    assert_eq!(split_range(6, 4, MAX_STACK), 1..=9);
    assert!(split_range(1, 0, MAX_STACK).is_empty());
    assert_eq!(split_range(2, 0, MAX_STACK), 1..=1);
    assert_eq!(split_range(MAX_STACK, 10, MAX_STACK), 10..=MAX_STACK);
    assert_eq!(
        split_range(MAX_STACK, MAX_STACK, MAX_STACK),
        MAX_STACK..=MAX_STACK
    );
    assert_eq!(
        split_range(u32::MAX, u32::MAX, u32::MAX),
        u32::MAX..=u32::MAX
    );
    assert!(split_range(0, 0, MAX_STACK).is_empty());
    let army = [
        stack(0, PEASANT, 10),
        stack(2, GNOLL, 3),
        stack(4, PEASANT, 5),
    ];
    // Into an empty slot, and onto a stack of the creature (`count` is the target's after).
    assert_eq!(
        one(split(hero(1, 0), hero(1, 1), 4), &army),
        Ok(vec![
            stack(0, PEASANT, 6),
            stack(1, PEASANT, 4),
            stack(2, GNOLL, 3),
            stack(4, PEASANT, 5)
        ])
    );
    assert_eq!(
        one(split(hero(1, 0), hero(1, 4), 14), &army),
        Ok(vec![
            stack(0, PEASANT, 1),
            stack(2, GNOLL, 3),
            stack(4, PEASANT, 14)
        ])
    );
    let refused = [
        (split(hero(1, 0), hero(1, 0), 4), ArmyCommandError::SameSlot),
        (
            split(hero(1, 0), hero(1, 2), 4),
            ArmyCommandError::OtherCreature,
        ),
        (
            split(hero(1, 0), hero(1, 1), 0),
            ArmyCommandError::SplitCount(0),
        ),
        (
            split(hero(1, 0), hero(1, 1), 10),
            ArmyCommandError::SplitCount(10),
        ),
        (
            split(hero(1, 0), hero(1, 4), 15),
            ArmyCommandError::SplitCount(15),
        ),
        (
            split(hero(1, 1), hero(1, 3), 1),
            ArmyCommandError::NoStack(1),
        ),
        (
            split(hero(1, 0), hero(1, 8), 1),
            ArmyCommandError::NoSlot(8),
        ),
    ];
    for (command, error) in refused {
        assert_eq!(one(command, &army), Err(error), "{command:?}");
    }
    // Between two armies; a hero's single creature cannot split, but a split never empties
    // the source, so the last stack may split.
    let red = [stack(0, PEASANT, 10)];
    let blue = [stack(3, PEASANT, 2)];
    assert_eq!(
        two(split(hero(1, 0), hero(2, 3), 7), (&red, true), &blue),
        Ok((vec![stack(0, PEASANT, 5)], vec![stack(3, PEASANT, 7)]))
    );
    assert_eq!(
        two(
            split(hero(1, 0), hero(2, 0), 1),
            (&[stack(0, PEASANT, 1)], true),
            &[]
        ),
        Err(ArmyCommandError::SplitCount(1))
    );
    // Wide counts split exactly.
    assert_eq!(
        plan(
            &split(hero(1, 0), hero(1, 1), 1_000_000),
            (
                &[stack(0, PEASANT, 1_000_000), stack(1, PEASANT, 999_999)],
                true
            ),
            None,
            (7, 1_000_000)
        ),
        Ok((
            vec![stack(0, PEASANT, 999_999), stack(1, PEASANT, 1_000_000)],
            None
        ))
    );
}

#[test]
fn the_errors_read_well() {
    let cases = [
        (
            ArmyCommandError::NotYourHero(HeroId::new(3)),
            "hero 3 is not the player's",
        ),
        (ArmyCommandError::NoGroup, "no army dialog's group is open"),
        (
            ArmyCommandError::NoSlot(7),
            "slot 7 is beyond the army's slots",
        ),
        (ArmyCommandError::NoStack(2), "slot 2 holds no stack"),
        (
            ArmyCommandError::SameSlot,
            "the stack would go onto its own slot",
        ),
        (
            ArmyCommandError::TooManyCreatures,
            "the stack would hold more than a stack holds",
        ),
        (
            ArmyCommandError::LastStack,
            "a hero's last stack would leave its army",
        ),
        (
            ArmyCommandError::OtherCreature,
            "the split's target holds another creature",
        ),
        (
            ArmyCommandError::SplitCount(4),
            "a split cannot leave 4 in its target",
        ),
    ];
    for (error, text) in cases {
        assert_eq!(format!("{error}"), text);
    }
}

/// Red's heroes 1 (two stacks) and 3 (one) side by side, Blue's hero 2.
fn heroes() -> ((Ruleset, AdventureRuleset), AdventureState) {
    let rules = movement_rules(true);
    let setup = world(
        &["gggg", "gggg"],
        1,
        vec![
            hero_at(1, 0, Pos::new(0, 0, 0), &[PEASANT, GNOLL]),
            hero_at(2, 1, Pos::new(3, 1, 0), &[WOLF]),
            hero_at(3, 0, Pos::new(1, 0, 0), &[PIKEMAN]),
        ],
    );
    let state = crate::testing::started(&rules, &setup).unwrap();
    (rules, state)
}

fn army_of(state: &AdventureState, id: u16) -> Vec<ArmyStack> {
    state.hero(HeroId::new(id)).unwrap().army.clone()
}

// covers: ADV-ARM-001, ADV-ARM-002, ADV-ARM-004
#[test]
fn the_state_applies_a_command_on_its_players_heroes_and_the_group() {
    let (rules, mut state) = heroes();
    let part = &rules.1;
    // Within a hero, then between two of the player's heroes.
    state
        .manage_army(part, RED, &moved(hero(1, 1), hero(1, 5)), None)
        .unwrap();
    assert_eq!(
        army_of(&state, 1),
        [stack(0, PEASANT, 1), stack(5, GNOLL, 1)]
    );
    state
        .manage_army(part, RED, &moved(hero(1, 5), hero(3, 2)), None)
        .unwrap();
    assert_eq!(army_of(&state, 1), [stack(0, PEASANT, 1)]);
    assert_eq!(
        army_of(&state, 3),
        [stack(0, PIKEMAN, 1), stack(2, GNOLL, 1)]
    );
    // With the group: the hero takes its stack, the group keeps the rest.
    let mut found = vec![stack(0, WOLF, 7), stack(2, PIKEMAN, 4)];
    state
        .manage_army(part, RED, &moved(group(0), hero(1, 1)), Some(&mut found))
        .unwrap();
    assert_eq!(found, [stack(2, PIKEMAN, 4)]);
    assert_eq!(
        army_of(&state, 1),
        [stack(0, PEASANT, 1), stack(1, WOLF, 7)]
    );
    let dismiss = ArmyCommand::Dismiss { at: group(2) };
    state
        .manage_army(part, RED, &dismiss, Some(&mut found))
        .unwrap();
    assert!(found.is_empty());
    // A rejected command changes nothing: another player's hero, a hero not on the map, the
    // group without a dialog, the plan's own refusal.
    let before = state.clone();
    let mut found = vec![stack(0, WOLF, 7)];
    let refused = [
        (
            moved(hero(2, 0), hero(2, 1)),
            ArmyCommandError::NotYourHero(HeroId::new(2)),
        ),
        (
            moved(hero(1, 0), hero(2, 1)),
            ArmyCommandError::NotYourHero(HeroId::new(2)),
        ),
        (
            moved(hero(9, 0), hero(1, 1)),
            ArmyCommandError::NotYourHero(HeroId::new(9)),
        ),
        (moved(hero(1, 0), group(1)), ArmyCommandError::NoGroup),
        (moved(hero(1, 4), hero(3, 1)), ArmyCommandError::NoStack(4)),
    ];
    for (command, error) in refused {
        assert_eq!(state.manage_army(part, RED, &command, None), Err(error));
        assert_eq!(state, before);
    }
    assert_eq!(
        state.manage_army(part, RED, &moved(group(3), hero(1, 1)), Some(&mut found)),
        Err(ArmyCommandError::NoStack(3))
    );
    assert_eq!((&state, &found[..]), (&before, &[stack(0, WOLF, 7)][..]));
    // Blue manages its own.
    state
        .manage_army(part, BLUE, &moved(hero(2, 0), hero(2, 6)), None)
        .unwrap();
    assert_eq!(army_of(&state, 2), [stack(6, WOLF, 1)]);
}

// covers: ADV-ARM-003, ADV-ARM-007
#[test]
fn the_split_query_gives_the_range_of_a_possible_split() {
    let (rules, mut state) = heroes();
    let part = &rules.1;
    state
        .manage_army(part, RED, &moved(hero(1, 1), group(4)), Some(&mut vec![]))
        .unwrap();
    let mut found = vec![stack(0, PEASANT, 9)];
    state
        .manage_army(part, RED, &moved(group(0), hero(1, 0)), Some(&mut found))
        .unwrap();
    assert_eq!(army_of(&state, 1), [stack(0, PEASANT, 10)]);
    let range =
        |from, to, group: Option<&[ArmyStack]>| state.split_range(part, RED, (from, to), group);
    let found = [stack(1, PEASANT, 5), stack(2, GNOLL, 1)];
    assert_eq!(range(hero(1, 0), hero(1, 3), None), Some(1..=9));
    assert_eq!(range(hero(1, 0), hero(3, 3), None), Some(1..=9));
    assert_eq!(range(hero(1, 0), group(1), Some(&found)), Some(1..=14));
    assert_eq!(range(group(1), hero(1, 0), Some(&found)), Some(1..=14));
    // None: the same slot, another creature, a single creature, beyond the slots, no stack,
    // another player's hero, the group without a dialog.
    for (from, to) in [
        (hero(1, 0), hero(1, 0)),
        (hero(1, 0), group(2)),
        (group(2), group(3)),
        (hero(1, 0), hero(1, 7)),
        (hero(1, 3), hero(1, 4)),
        (hero(1, 0), hero(2, 1)),
        (hero(2, 0), hero(2, 1)),
    ] {
        assert_eq!(range(from, to, Some(&found)), None, "{from:?} {to:?}");
    }
    assert_eq!(range(hero(1, 0), group(0), None), None);
    // A split the query allows applies: the group's peasants into the hero's stack, and a part of
    // the hero's to the other hero.
    let mut found = found.to_vec();
    state
        .manage_army(
            part,
            RED,
            &split(group(1), hero(1, 0), 14),
            Some(&mut found),
        )
        .unwrap();
    assert_eq!(found, [stack(1, PEASANT, 1), stack(2, GNOLL, 1)]);
    state
        .manage_army(part, RED, &split(hero(1, 0), hero(3, 3), 4), None)
        .unwrap();
    assert_eq!(army_of(&state, 1), [stack(0, PEASANT, 10)]);
    assert_eq!(
        army_of(&state, 3),
        [stack(0, PIKEMAN, 1), stack(3, PEASANT, 4)]
    );
}

/// Every move from `armies[0]`'s slots onto the slots of `armies`, then every dismissal of
/// `armies[0]`'s, as the state accepts them on a copy (with `group` as the open dialog's).
fn accepted(
    state: &AdventureState,
    part: &AdventureRuleset,
    armies: &[ArmyRef],
    group: Option<&[ArmyStack]>,
) -> Vec<ArmyCommand> {
    let slots = part.params().army_slots();
    let slot = |army, slot| ArmySlot { army, slot };
    let mut all = Vec::new();
    for source in 0..slots {
        for &army in armies {
            for target in 0..slots {
                all.push(moved(slot(armies[0], source), slot(army, target)));
            }
        }
    }
    all.extend((0..slots).map(|at| ArmyCommand::Dismiss {
        at: slot(armies[0], at),
    }));
    all.retain(|command| {
        let mut copy = state.clone();
        let mut found = group.map(<[ArmyStack]>::to_vec);
        copy.manage_army(part, RED, command, found.as_mut()).is_ok()
    });
    all
}

// covers: ADV-ARM-001, ADV-ARM-002, ADV-ARM-004, GAM-MAN-002
#[test]
fn the_listing_gives_every_move_and_dismissal_the_state_accepts() {
    let (rules, state) = heroes();
    let part = &rules.1;
    let (one, three) = (ArmyRef::Hero(HeroId::new(1)), ArmyRef::Hero(HeroId::new(3)));
    // Hero 1's two stacks: each onto the six other slots (an empty one, or the other stack's,
    // a swap), and each dismissed.
    let within = state.army_commands(part, RED, (one, None), None);
    assert_eq!(within.len(), 2 * 6 + 2);
    assert_eq!(within[0], moved(hero(1, 0), hero(1, 1)));
    assert_eq!(within[12], ArmyCommand::Dismiss { at: hero(1, 0) });
    assert_eq!(within, accepted(&state, part, &[one], None));
    // With hero 3: onto its seven slots too.
    let met = state.army_commands(part, RED, (one, Some(three)), None);
    assert_eq!(met.len(), 2 * (6 + 7) + 2);
    assert_eq!(met, accepted(&state, part, &[one, three], None));
    // Hero 3's last stack: moved within its army or swapped with hero 1's stacks, never
    // dismissed nor moved to an empty slot of hero 1.
    let last = state.army_commands(part, RED, (three, Some(one)), None);
    assert_eq!(last.len(), 6 + 2);
    assert_eq!(last, accepted(&state, part, &[three, one], None));
    // The group: its stacks onto its own slots and the hero's, each dismissed.
    let found = [stack(0, WOLF, 7), stack(2, PIKEMAN, 4)];
    let group = state.army_commands(part, RED, (ArmyRef::Group, Some(one)), Some(&found));
    assert_eq!(group.len(), 2 * (6 + 7) + 2);
    assert_eq!(
        group,
        accepted(&state, part, &[ArmyRef::Group, one], Some(&found))
    );
    // None for another player's hero, a hero not on the map or the group without a dialog;
    // toward such an army, the army's own commands alone.
    for armies in [
        (ArmyRef::Hero(HeroId::new(2)), None),
        (ArmyRef::Hero(HeroId::new(9)), None),
        (ArmyRef::Group, None),
    ] {
        assert_eq!(state.army_commands(part, RED, armies, None), []);
    }
    for to in [ArmyRef::Hero(HeroId::new(2)), ArmyRef::Group] {
        assert_eq!(
            state.army_commands(part, RED, (one, Some(to)), None),
            within
        );
    }
    // The slots a command changes.
    assert_eq!(
        moved(hero(1, 0), hero(3, 2)).slots(),
        (hero(1, 0), Some(hero(3, 2)))
    );
    assert_eq!(
        split(group(0), hero(1, 1), 2).slots(),
        (group(0), Some(hero(1, 1)))
    );
    assert_eq!(
        ArmyCommand::Dismiss { at: hero(1, 4) }.slots(),
        (hero(1, 4), None)
    );
}

// covers: ADV-ARM-006
#[test]
fn the_formation_and_the_tactics_flag_are_set_at_no_cost_and_hashed() {
    let (rules, mut state) = heroes();
    let red = HeroId::new(1);
    // The flag is on at the start (the original's 2).
    assert!(state.heroes().iter().all(|h| h.tactics));
    let points = state.hero(red).unwrap().movement;
    let hash = hash_of(&state);
    state.set_tactics(RED, red, false).unwrap();
    assert!(!state.hero(red).unwrap().tactics);
    let off = hash_of(&state);
    assert_ne!(off, hash);
    state.set_formation(RED, red, Formation::Tight).unwrap();
    assert_eq!(state.hero(red).unwrap().formation, Formation::Tight);
    assert_ne!(hash_of(&state), off);
    assert_eq!(state.hero(red).unwrap().movement, points);
    // Again: nothing more changes.
    let set = state.clone();
    state.set_tactics(RED, red, false).unwrap();
    state.set_formation(RED, red, Formation::Tight).unwrap();
    assert_eq!(state, set);
    // The owner sees the flag.
    let seen = state.observe(&rules.0, &rules.1, RED);
    assert_eq!(
        seen.hero(red).unwrap().own.as_ref().map(|own| own.tactics),
        Some(false)
    );
    // Another player's hero: refused, nothing changed.
    let blue = HeroId::new(2);
    assert_eq!(
        state.set_tactics(RED, blue, false),
        Err(ArmyCommandError::NotYourHero(blue))
    );
    assert_eq!(
        state.set_formation(RED, blue, Formation::Loose),
        Err(ArmyCommandError::NotYourHero(blue))
    );
    assert_eq!(state, set);
}

/// A command among the slots of hero 1 (army 0), hero 2 (1) and the group (2).
fn make(kind: u8, (a, sa): (u8, u8), (b, sb): (u8, u8), count: u32) -> ArmyCommand {
    let slot = |army: u8, slot: u8| ArmySlot {
        army: match army {
            0 => ArmyRef::Hero(HeroId::new(1)),
            1 => ArmyRef::Hero(HeroId::new(2)),
            _ => ArmyRef::Group,
        },
        slot,
    };
    match kind {
        0 => moved(slot(a, sa), slot(b, sb)),
        1 => split(slot(a, sa), slot(b, sb), count),
        _ => ArmyCommand::Dismiss { at: slot(a, sa) },
    }
}

/// Creatures per creature type in the three armies.
fn totals(armies: &[Vec<ArmyStack>; 3]) -> [u64; 4] {
    let mut out = [0; 4];
    for stack in armies.iter().flatten() {
        out[usize::from(stack.creature.raw())] += u64::from(stack.count);
    }
    out
}

fn army_strategy() -> impl Strategy<Value = Vec<ArmyStack>> {
    proptest::collection::btree_map(0u8..7, (0u16..4, 1u32..=MAX_STACK), 1..=7).prop_map(|map| {
        map.into_iter()
            .map(|(slot, (creature, count))| stack(slot, CreatureTypeId::from_raw(creature), count))
            .collect()
    })
}

proptest! {
    /// Any sequence of commands on two heroes' armies and a group keeps every invariant: each
    /// army in slot order with distinct slots below seven, every count from 1 to the most, a
    /// hero's army never empty; a move or split keeps each creature's total, a dismissal takes
    /// its stack only; a move that is no merge, undone by the move back, restores both armies.
    #[test]
    fn commands_keep_the_armies_whole(
        start in (army_strategy(), army_strategy(), proptest::collection::vec(
            (0u16..4, 1u32..=MAX_STACK), 0..=7)),
        commands in proptest::collection::vec(
            (0u8..3, (0u8..3, 0u8..8), (0u8..3, 0u8..8), 0u32..=2 * MAX_STACK), 1..40),
    ) {
        let (red, blue, found) = start;
        let found: Vec<ArmyStack> = found
            .into_iter()
            .zip(0u8..)
            .map(|((creature, count), slot)| stack(slot, CreatureTypeId::from_raw(creature), count))
            .collect();
        let mut armies = [red, blue, found];
        for (kind, from, to, count) in commands {
            let command = make(kind, from, to, count);
            let (source, target) = (usize::from(from.0), usize::from(to.0));
            let other = (kind < 2 && source != target).then_some(target);
            let result = plan(
                &command,
                (&armies[source], source < 2),
                other.map(|t| &armies[t][..]),
                RULES,
            );
            let Ok((a, b)) = result else { continue };
            let before = (armies.clone(), totals(&armies));
            armies[source] = a;
            if let (Some(t), Some(b)) = (other, b) {
                armies[t] = b;
            }
            for (index, army) in armies.iter().enumerate() {
                prop_assert!(index == 2 || !army.is_empty());
                prop_assert!(army.windows(2).all(|w| w[0].slot < w[1].slot));
                prop_assert!(army.iter().all(|s| s.slot < 7 && (1..=MAX_STACK).contains(&s.count)));
            }
            let after = totals(&armies);
            if kind == 2 {
                let gone = before.0[source].iter().find(|s| s.slot == from.1).unwrap();
                let mut expected = before.1;
                expected[usize::from(gone.creature.raw())] -= u64::from(gone.count);
                prop_assert_eq!(after, expected);
            } else {
                prop_assert_eq!(after, before.1);
            }
            let merged = before.0[other.unwrap_or(source)]
                .iter()
                .any(|s| s.slot == to.1 && before.0[source].iter().any(|t| {
                    t.slot == from.1 && t.creature == s.creature
                }));
            if kind == 0 && !merged {
                let mut undone = armies.clone();
                let back = make(0, to, from, 0);
                let (a, b) = plan(
                    &back,
                    (&undone[target], target < 2),
                    other.map(|_| &undone[source][..]),
                    RULES,
                ).unwrap();
                undone[target] = a;
                if let Some(b) = b {
                    undone[source] = b;
                }
                prop_assert_eq!(undone, before.0);
            }
        }
    }
}
