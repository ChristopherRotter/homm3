//! Stacks a battle creates (E04b T6b): what they share (BAT-MAG-037), Clone (BAT-MAG-038, quirk
//! BAT-QUIRK-7) and Summon Elemental (BAT-MAG-039), after the original's code
//! (`docs/formats/installation.md` §7.19).

use std::vec;
use std::vec::Vec;

use h3_content::Ruleset;
use h3_contracts::{BattleHero, BattleResult, BattleSide};

use super::spells::{arena, mage, spell, stack};
use super::support::*;
use crate::damage::apply_damage;
use crate::effects;
use crate::geometry::Field;
use crate::natural::NaturalObstacles;
use crate::raise::mark_corpses;
use crate::{
    BattleState, CREATED_SLOT, CastTarget, CastTargets, Command, CommandError, Event, EventKind,
    NO_CORPSE, StackFlags, StackId, StackTraits, cast_targets, castable_spells, second_targets,
};

/// A mage with the spells that create stacks and Sacrifice, air mastery `air`, fire mastery
/// `fire`, power 3, 100 mana.
fn creator(r: &Ruleset, air: u8, fire: u8) -> BattleHero {
    let mut h = mage(r, &["copy", "sprites", "serfs", "offering"]);
    h.magic.schools[0] = air;
    h.magic.schools[1] = fire;
    h.magic.mana = 100;
    h
}

/// The stacks `side` may aim `name` at now, by id.
fn targets(s: &BattleState, r: &Ruleset, side: BattleSide, name: &str) -> Vec<u8> {
    let mut out = CastTargets::new();
    cast_targets(s, r, side, spell(r, name), &mut out);
    out.iter()
        .filter_map(|t| match *t {
            CastTarget::Stack(id) => Some(id.0),
            _ => None,
        })
        .collect()
}

/// The side to act casts `name` at `target`; its hero may cast again this round.
fn cast(
    s: &mut BattleState,
    r: &Ruleset,
    name: &str,
    target: CastTarget,
) -> Result<Vec<Event>, CommandError> {
    let events = act(
        s,
        r,
        Command::Cast {
            spell: spell(r, name),
            target,
            second: CastTarget::None,
        },
    )?;
    for hero in s.heroes.iter_mut().flatten() {
        hero.view.casts = 0;
    }
    Ok(events)
}

/// The created stacks as `(stack, count, hex, original)`.
fn created(events: &[Event]) -> Vec<(u8, u32, u8, Option<u8>)> {
    events
        .iter()
        .filter_map(|e| match e.kind {
            EventKind::StackCreated {
                stack,
                count,
                hex,
                original,
                ..
            } => Some((stack.0, count, hex.0, original.map(|o| o.0))),
            _ => None,
        })
        .collect()
}

/// The vanished clones as `(stack, creatures)`.
fn vanished(events: &[Event]) -> Vec<(u8, u32)> {
    events
        .iter()
        .filter_map(|e| match e.kind {
            EventKind::Vanished { stack, creatures } => Some((stack.0, creatures)),
            _ => None,
        })
        .collect()
}

/// Deals `damage` to stack `id` as a strike would.
fn hurt(s: &mut BattleState, r: &Ruleset, id: u8, damage: u64) -> Vec<Event> {
    let mut events = Vec::new();
    apply_damage(s.at_mut(StackId(id)), damage);
    effects::damaged(s, r, StackId(id), &mut events);
    events
}

// covers: BAT-MAG-038, BAT-MAG-037, BAT-END-003
#[test]
fn clone_copies_an_own_stack_of_a_level_its_mastery_allows() {
    let r = ruleset();
    let mut s = arena(
        &r,
        [Some(creator(&r, 1, 0)), None],
        &[("serf", 10, 5, 5), ("tier3", 4, 5, 8), ("gilded", 2, 5, 2)],
        &[("serf", 1, 14, 5)],
        1,
    );
    // Basic: creatures up to level 1; the gilded ones are immune to level 4, the enemy is no
    // target. Expert: up to level 3.
    assert_eq!(targets(&s, &r, BattleSide::Attacker, "copy"), vec![0]);
    s.heroes[0].as_mut().unwrap().magic.schools[0] = 3;
    assert_eq!(targets(&s, &r, BattleSide::Attacker, "copy"), vec![0, 1]);
    let events = cast(&mut s, &r, "copy", stack(0)).unwrap();
    // The first place: top right of the serfs (an attacker's stack).
    assert_eq!(created(&events), vec![(4, 10, at(&r, 5, 4).0, Some(0))]);
    let (serfs, clone) = (s.stacks()[0], s.stacks()[4]);
    assert_eq!(
        (clone.count, clone.base_count, clone.slot, clone.side),
        (10, 10, CREATED_SLOT, BattleSide::Attacker)
    );
    assert_eq!(
        (clone.attack, clone.defense, clone.speed, clone.max_hp),
        (serfs.attack, serfs.defense, serfs.speed, serfs.max_hp)
    );
    assert!(clone.has(StackTraits::CLONE) && clone.is_created() && !serfs.is_created());
    assert_eq!(clone.flags, StackFlags::default());
    // Power 3, no duration bonus: three rounds.
    assert_eq!(
        (s.clone_rounds(StackId(4)), s.clone_rounds(StackId(0))),
        (3, 0)
    );
    // Neither a cloned stack nor a clone is a target.
    assert_eq!(targets(&s, &r, BattleSide::Attacker, "copy"), vec![1]);
    // The outcome leaves the clone out.
    hurt(&mut s, &r, 3, 1000);
    crate::combat::check_end(&mut s, &mut Vec::<Event>::new());
    let outcome = s.outcome().unwrap();
    assert_eq!(outcome.result, BattleResult::Victory(BattleSide::Attacker));
    assert_eq!(outcome.stacks[0].len(), 3);
}

// covers: BAT-MAG-038
#[test]
fn a_clone_goes_to_the_first_free_hex_along_the_rays() {
    let r = ruleset();
    let hero = creator(&r, 1, 0);
    let place_of = |attacker: &[(&'static str, u32, i32, i32)], target: u8| {
        let mut s = arena(
            &r,
            [Some(hero.clone()), Some(hero.clone())],
            attacker,
            &[("serf", 1, 10, 5)],
            1,
        );
        let events = cast(&mut s, &r, "copy", stack(target)).unwrap();
        created(&events).first().map(|c| c.2)
    };
    // From top right clockwise for the attacker, right and left of it skipped in ring 1.
    assert_eq!(place_of(&[("serf", 5, 5, 5)], 0), Some(at(&r, 5, 4).0));
    assert_eq!(
        place_of(&[("serf", 5, 5, 5), ("serf", 1, 5, 4)], 0),
        Some(at(&r, 5, 6).0)
    );
    // A two-hex stack's clone stands on the row above, its tail top right of the stack's tail.
    assert_eq!(place_of(&[("ogre", 2, 6, 5)], 0), Some(at(&r, 6, 4).0));
    // The defender's clone: from top left anticlockwise.
    let mut s = arena(
        &r,
        [Some(hero.clone()), Some(hero)],
        &[("serf", 1, 2, 5)],
        &[("serf", 5, 10, 5)],
        1,
    );
    activate(&mut s, 1);
    let events = cast(&mut s, &r, "copy", stack(1)).unwrap();
    assert_eq!(created(&events), vec![(2, 5, at(&r, 9, 4).0, Some(1))]);
}

// covers: BAT-MAG-038
#[test]
fn a_crowded_two_hex_stack_is_never_covered_by_its_clone() {
    // Quirk BAT-QUIRK-7 (fixed): rows 3 to 7 blocked from column 9 on but for the ogres (tail
    // 13, head 14) and the hex in front of them. The original would take the ogres' head as the
    // clone's position in ring 2; the engine goes on to ring 3, top right.
    let r = ruleset();
    let mut s = arena(
        &r,
        [Some(creator(&r, 1, 0)), None],
        &[("ogre", 2, 14, 5)],
        &[("serf", 1, 1, 0)],
        1,
    );
    let free = [at(&r, 13, 5), at(&r, 14, 5), at(&r, 15, 5)];
    let walls: Vec<u8> = (9..16)
        .flat_map(|x| (3..8).map(move |y| (x, y)))
        .map(|(x, y)| at(&r, x, y))
        .filter(|h| !free.contains(h))
        .map(|h| h.0)
        .collect();
    let field = Field::new(r.battle().field);
    s.natural = NaturalObstacles::new(field, &blocked(walls), false).unwrap();
    let events = cast(&mut s, &r, "copy", stack(0)).unwrap();
    assert_eq!(created(&events), vec![(2, 2, at(&r, 15, 2).0, Some(0))]);
    // Fully walled in: no place, the mana is spent all the same.
    let mut s = arena(
        &r,
        [Some(creator(&r, 1, 0)), None],
        &[("serf", 2, 14, 5)],
        &[("serf", 1, 1, 0)],
        1,
    );
    let walls: Vec<u8> = (0..field.hex_count())
        .map(|h| u8::try_from(h).unwrap())
        .filter(|&h| field.is_standable(crate::geometry::Hex(h)))
        .filter(|&h| h != at(&r, 14, 5).0 && h != at(&r, 1, 0).0)
        .collect();
    s.natural = NaturalObstacles::new(field, &blocked(walls), false).unwrap();
    let mana = s.hero(BattleSide::Attacker).unwrap().mana;
    let events = cast(&mut s, &r, "copy", stack(0)).unwrap();
    assert_eq!((created(&events), s.stacks().len()), (vec![], 2));
    assert!(s.hero(BattleSide::Attacker).unwrap().mana < mana);
}

// covers: BAT-MAG-038
#[test]
fn a_clone_vanishes_when_its_rounds_run_out_or_its_original_dies() {
    let r = ruleset();
    let new = |r: &Ruleset| {
        let mut s = arena(
            r,
            [Some(creator(r, 1, 0)), None],
            &[("serf", 10, 5, 5)],
            &[("serf", 1, 14, 5)],
            1,
        );
        cast(&mut s, r, "copy", stack(0)).unwrap();
        s
    };
    // Three rounds: at the third round start it is gone, without a corpse, and its original may
    // be cloned again.
    let mut s = new(&r);
    let mut events = Vec::new();
    crate::turn::start_round(&mut s, &r, &mut events);
    crate::turn::start_round(&mut s, &r, &mut events);
    assert_eq!((s.clone_rounds(StackId(2)), vanished(&events)), (1, vec![]));
    crate::turn::start_round(&mut s, &r, &mut events);
    assert_eq!(vanished(&events), vec![(2, 10)]);
    assert_eq!(
        (
            s.stacks()[2].count,
            s.corpse(StackId(2)),
            s.clone_rounds(StackId(2))
        ),
        (0, NO_CORPSE, 0)
    );
    assert_eq!(targets(&s, &r, BattleSide::Attacker, "copy"), vec![0]);
    // Any damage kills every creature of a clone, which frees its original.
    let mut s = new(&r);
    hurt(&mut s, &r, 2, 1);
    assert_eq!((s.stacks()[2].count, s.clone_rounds(StackId(2))), (0, 0));
    mark_corpses(&mut s);
    assert_eq!(s.corpse(StackId(2)), NO_CORPSE);
    assert_eq!(targets(&s, &r, BattleSide::Attacker, "copy"), vec![0]);
    // A dying original takes its clone with it.
    let mut s = new(&r);
    let events = hurt(&mut s, &r, 0, 1000);
    assert_eq!(vanished(&events), vec![(2, 10)]);
}

// covers: BAT-MAG-039, BAT-MAG-037
#[test]
fn summon_elemental_places_effect_times_power_creatures_in_the_first_free_column() {
    let r = ruleset();
    let hero = creator(&r, 0, 1);
    let mut s = arena(
        &r,
        [Some(hero.clone()), Some(hero)],
        &[("serf", 1, 5, 5)],
        &[("serf", 1, 10, 5)],
        1,
    );
    assert_eq!(targets(&s, &r, BattleSide::Attacker, "sprites"), vec![]);
    let events = cast(&mut s, &r, "sprites", CastTarget::None).unwrap();
    // Basic: 2 per power, 6 sprites, in column 1, with its hero's attack and defense.
    let made = created(&events);
    assert_eq!(
        (made.len(), made[0].0, made[0].1, made[0].3),
        (1, 2, 6, None)
    );
    let sprite = s.stacks()[2];
    assert_eq!(Field::new(r.battle().field).coords(sprite.hex).0, 1);
    assert_eq!(
        (
            sprite.attack,
            sprite.defense,
            sprite.slot,
            sprite.base_count
        ),
        (6 + 3, 6 + 2, CREATED_SLOT, 6)
    );
    assert!(sprite.is_created() && !sprite.has(StackTraits::CLONE));
    // The side's summons are bound to the sprites: no serfs, but more sprites in a new stack.
    let castable = castable_spells(&s, &r, BattleSide::Attacker);
    assert!(castable.contains(spell(&r, "sprites")) && !castable.contains(spell(&r, "serfs")));
    assert_eq!(
        cast(&mut s, &r, "serfs", CastTarget::None),
        Err(CommandError::InvalidSpellTarget)
    );
    cast(&mut s, &r, "sprites", CastTarget::None).unwrap();
    assert_eq!(s.stacks().len(), 4);
    // The defender summons in column 15; with column 15 full, column 14.
    activate(&mut s, 1);
    let field = Field::new(r.battle().field);
    let column: Vec<u8> = (0..11).map(|y| at(&r, 15, y).0).collect();
    s.natural = NaturalObstacles::new(field, &blocked(column), false).unwrap();
    cast(&mut s, &r, "serfs", CastTarget::None).unwrap();
    let serfs = s.stacks()[4];
    assert_eq!(
        (serfs.side, field.coords(serfs.hex).0, serfs.count),
        (BattleSide::Defender, 14, 6)
    );
    // A new stack acts in the round it appears (none of the round's flags).
    assert_eq!(serfs.flags, StackFlags::default());
}

// covers: BAT-MAG-037
#[test]
fn a_side_creates_stacks_below_the_limit_and_reuses_dead_created_ones() {
    let r = ruleset();
    let mut s = arena(
        &r,
        [Some(creator(&r, 1, 1)), None],
        &[("serf", 10, 5, 5)],
        &[("serf", 1, 14, 5)],
        1,
    );
    let sprites = |s: &mut BattleState| {
        s.heroes[0].as_mut().unwrap().view.mana = 100;
        created(&cast(s, &r, "sprites", CastTarget::None).unwrap())[0].0
    };
    assert_eq!(
        [sprites(&mut s), sprites(&mut s), sprites(&mut s)],
        [2, 3, 4]
    );
    // A new stack takes the first dead created stack's slot, which starts afresh.
    hurt(&mut s, &r, 4, 1000);
    hurt(&mut s, &r, 3, 1000);
    mark_corpses(&mut s);
    assert_eq!(sprites(&mut s), 3);
    assert_eq!((s.stacks().len(), s.corpse(StackId(3))), (5, 0));
    // The limit is 20 stacks of a side, dead ones included (the next cast reuses 4).
    assert_eq!(sprites(&mut s), 4);
    for _ in 0..16 {
        sprites(&mut s);
    }
    let side = |s: &BattleState| {
        s.stacks()
            .iter()
            .filter(|t| t.side == BattleSide::Attacker)
            .count()
    };
    assert_eq!(side(&s), 20);
    let castable = castable_spells(&s, &r, BattleSide::Attacker);
    assert!(!castable.contains(spell(&r, "sprites")) && !castable.contains(spell(&r, "copy")));
    hurt(&mut s, &r, 4, 1000);
    assert!(!castable_spells(&s, &r, BattleSide::Attacker).contains(spell(&r, "sprites")));
}

// covers: BAT-MAG-037, BAT-END-001, BAT-END-002
#[test]
fn a_side_left_with_only_created_stacks_does_not_win() {
    let r = ruleset();
    let mut s = arena(
        &r,
        [Some(creator(&r, 0, 1)), None],
        &[("serf", 1, 5, 5)],
        &[("serf", 1, 14, 5)],
        1,
    );
    cast(&mut s, &r, "sprites", CastTarget::None).unwrap();
    // The serfs die: the sprites fight on.
    hurt(&mut s, &r, 0, 1000);
    let mut events = Vec::new();
    crate::combat::check_end(&mut s, &mut events);
    assert_eq!(s.result(), None);
    // The enemy dies: both lose.
    hurt(&mut s, &r, 1, 1000);
    crate::combat::check_end(&mut s, &mut events);
    assert_eq!(s.result(), Some(BattleResult::Draw));
    let outcome = s.outcome().unwrap();
    assert_eq!(
        (outcome.stacks[0].len(), outcome.survivors[0].stacks.len()),
        (1, 0)
    );
}

// covers: BAT-MAG-036, BAT-MAG-037, BAT-MAG-038
#[test]
fn created_stacks_are_no_victims_and_a_sacrificed_original_takes_its_clone() {
    let r = ruleset();
    let mut s = arena(
        &r,
        [Some(creator(&r, 1, 3)), None],
        &[("serf", 10, 5, 5), ("serf", 4, 5, 8)],
        &[("serf", 1, 14, 5)],
        1,
    );
    cast(&mut s, &r, "copy", stack(1)).unwrap();
    cast(&mut s, &r, "sprites", CastTarget::None).unwrap();
    s.stacks.as_mut_slice()[0].count = 5;
    let mut out = CastTargets::new();
    let offering = (spell(&r, "offering"), stack(0));
    second_targets(&s, &r, BattleSide::Attacker, offering, &mut out);
    assert_eq!(out.as_slice(), [stack(1)]);
    let events = act(
        &mut s,
        &r,
        Command::Cast {
            spell: offering.0,
            target: stack(0),
            second: stack(1),
        },
    )
    .unwrap();
    assert_eq!(vanished(&events), vec![(3, 4)]);
    assert_eq!(s.stacks()[0].count, 10);
}
