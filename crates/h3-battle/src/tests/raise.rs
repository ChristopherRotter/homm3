//! Raising spells (E04b T6a): Resurrection and Animate Dead (BAT-MAG-035), Sacrifice
//! (BAT-MAG-036, quirk BAT-QUIRK-5) and the corpses they find, after the original's code
//! (`docs/formats/installation.md` §7.18).

use std::vec;
use std::vec::Vec;

use h3_content::Ruleset;
use h3_contracts::{BattleHero, BattleResult, BattleSide};

use super::spells::{arena, mage, spell, stack};
use super::support::*;
use crate::effects::{self, Status, TimedCast};
use crate::raise::mark_corpses;
use crate::{
    BattleState, CastTarget, CastTargets, Command, CommandError, Event, EventKind, NO_CORPSE,
    StackId, cast_targets, castable_spells, second_targets,
};

/// A mage with the raising spells, air mastery `air`, fire mastery `fire`, power 3, 100 mana.
fn raiser(r: &Ruleset, air: u8, fire: u8) -> BattleHero {
    let mut h = mage(r, &["rise", "animate", "offering"]);
    h.magic.masteries = masteries(r, [air, fire, 0, 0]);
    h.magic.mana = 100;
    h
}

/// Kills stack `id` as a command would, and marks its corpse.
fn kill(s: &mut BattleState, r: &Ruleset, id: u8) {
    let stack = &mut s.stacks.as_mut_slice()[usize::from(id)];
    stack.count = 0;
    stack.top_hp = 0;
    effects::remove_dead(s, r);
    mark_corpses(s);
}

/// The stacks the attacker may aim `name` at now, by id.
fn targets(s: &BattleState, r: &Ruleset, name: &str) -> Vec<u8> {
    let mut out = CastTargets::new();
    cast_targets(s, r, BattleSide::Attacker, spell(r, name), &mut out);
    out.iter()
        .filter_map(|t| match *t {
            CastTarget::Stack(id) => Some(id.0),
            _ => None,
        })
        .collect()
}

/// The victims the attacker may sacrifice to raise stack `target`, by id.
fn victims(s: &BattleState, r: &Ruleset, target: u8) -> Vec<u8> {
    let mut out = CastTargets::new();
    let cast = (spell(r, "offering"), stack(target));
    second_targets(s, r, BattleSide::Attacker, cast, &mut out);
    out.iter()
        .filter_map(|t| match *t {
            CastTarget::Stack(id) => Some(id.0),
            _ => None,
        })
        .collect()
}

/// The attacker casts `name` at `target` with the second target `second`.
fn cast(
    s: &mut BattleState,
    r: &Ruleset,
    name: &str,
    (target, second): (CastTarget, CastTarget),
) -> Result<Vec<Event>, CommandError> {
    act(
        s,
        r,
        Command::Cast {
            spell: spell(r, name),
            target,
            second,
        },
    )
}

/// The raise events as `(target, hit points, creatures, temporary)`.
fn raised(events: &[Event]) -> Vec<(u8, u64, u32, bool)> {
    events
        .iter()
        .filter_map(|e| match e.kind {
            EventKind::SpellRaised {
                target,
                hit_points,
                creatures,
                temporary,
            } => Some((target.0, hit_points, creatures, temporary)),
            _ => None,
        })
        .collect()
}

// covers: BAT-MAG-035, BAT-END-003
#[test]
fn resurrection_raises_a_wounded_stack_for_the_battle_below_advanced() {
    let r = ruleset();
    let hero = raiser(&r, 1, 0);
    let mut s = arena(
        &r,
        [Some(hero), None],
        &[("serf", 10, 2, 5), ("fixed", 3, 2, 7)],
        &[("serf", 1, 14, 5)],
        1,
    );
    // Five serfs left, the top one with 4 of 10 hit points: 44 of 100.
    s.stacks.as_mut_slice()[0].count = 5;
    s.stacks.as_mut_slice()[0].top_hp = 4;
    assert_eq!(targets(&s, &r, "rise"), vec![0]);
    // Basic: 3 × 50 + 40 = 190 hit points, capped at the start's 100: five creatures back, for
    // the battle only.
    let events = cast(&mut s, &r, "rise", (stack(0), CastTarget::None)).unwrap();
    assert_eq!(raised(&events), vec![(0, 56, 5, true)]);
    let serfs = s.stacks()[0];
    assert_eq!(
        (serfs.count, serfs.top_hp, s.temporary(StackId(0))),
        (10, 10, 5)
    );
    // A full stack is no target, wounded or not, nor is a stack of creatures that are not alive.
    s.stacks.as_mut_slice()[0].top_hp = 3;
    s.heroes[0].as_mut().unwrap().view.casts = 0;
    assert_eq!(targets(&s, &r, "rise"), Vec::<u8>::new());
    s.stacks.as_mut_slice()[1].count = 1;
    assert_eq!(targets(&s, &r, "rise"), Vec::<u8>::new());
    // The temporary creatures are left out of the end count (S-20's winner keeps the rest).
    kill(&mut s, &r, 2);
    crate::combat::check_end(&mut s, &mut Vec::<Event>::new());
    let outcome = s.outcome().unwrap();
    assert_eq!(outcome.result, BattleResult::Victory(BattleSide::Attacker));
    assert_eq!(outcome.stacks[0][0].end, 5);
    assert_eq!(outcome.survivors[0].stacks[0].count, 5);
}

// covers: BAT-MAG-035
#[test]
fn resurrection_from_advanced_raises_for_good_and_needs_one_creatures_health() {
    let r = ruleset();
    let mut s = arena(
        &r,
        [Some(raiser(&r, 2, 0)), None],
        &[("seraph", 2, 2, 5)],
        &[("serf", 1, 14, 5)],
        1,
    );
    s.stacks.as_mut_slice()[0].count = 1;
    // Advanced: 3 × 50 + 80 = 230 hit points reach a seraph's 200.
    assert_eq!(targets(&s, &r, "rise"), vec![0]);
    let events = cast(&mut s, &r, "rise", (stack(0), CastTarget::None)).unwrap();
    assert_eq!(raised(&events), vec![(0, 200, 1, false)]);
    assert_eq!((s.stacks()[0].count, s.temporary(StackId(0))), (2, 0));
    // Basic: 190 hit points do not: the seraphs are no target.
    s.stacks.as_mut_slice()[0].count = 1;
    set_masteries(&mut s, 0, &r, [1, 0, 0, 0]);
    s.heroes[0].as_mut().unwrap().view.casts = 0;
    assert_eq!(targets(&s, &r, "rise"), Vec::<u8>::new());
    assert_eq!(
        cast(&mut s, &r, "rise", (stack(0), CastTarget::None)),
        Err(CommandError::InvalidSpellTarget)
    );
}

// covers: BAT-MAG-035, BAT-MAG-010
#[test]
fn a_dead_stack_comes_back_on_its_hexes_without_effects_and_keeps_its_round() {
    let r = ruleset();
    let mut s = arena(
        &r,
        [Some(raiser(&r, 3, 0)), None],
        &[("fixed", 1, 2, 3), ("serf", 4, 5, 5)],
        &[("serf", 1, 14, 5)],
        1,
    );
    let haste = spell(&r, "haste");
    let timed = TimedCast {
        id: haste,
        spell: r.spell(haste).unwrap(),
        mastery: 1,
        rounds: 5,
        own: crate::magic::SpellBonus::default(),
    };
    effects::apply(&mut s, &r, timed, StackId(1), &mut Vec::<Event>::new());
    s.stacks.as_mut_slice()[1]
        .flags
        .set(crate::StackFlags::ACTED, true);
    kill(&mut s, &r, 1);
    assert_eq!(s.corpse(StackId(1)), 1);
    assert_eq!(targets(&s, &r, "rise"), vec![1]);
    // Expert: 3 × 50 + 160 = 310 hit points, capped at 40: all four back, for good.
    let events = cast(&mut s, &r, "rise", (stack(1), CastTarget::None)).unwrap();
    assert_eq!(raised(&events), vec![(1, 40, 4, false)]);
    let serfs = s.stacks()[1];
    assert_eq!(
        (serfs.count, serfs.hex, s.corpse(StackId(1))),
        (4, at(&r, 5, 5), 0)
    );
    assert!(s.effects.of(StackId(1)).is_empty());
    assert_eq!(serfs.spells.speed, serfs.speed);
    assert!(serfs.flags.contains(crate::StackFlags::ACTED));
    // The active stack kept its turn.
    assert_eq!(s.active(), Some(StackId(0)));
}

// covers: BAT-MAG-035, BAT-MAG-010, BAT-MAG-009
#[test]
fn a_raised_stack_keeps_the_permanent_effects_it_died_with() {
    let r = ruleset();
    let mut s = arena(
        &r,
        [Some(raiser(&r, 3, 0)), None],
        &[("fixed", 1, 2, 3), ("serf", 4, 5, 5)],
        &[("serf", 1, 14, 5)],
        1,
    );
    // An Expert Disrupting Ray (−5 for the rest of the battle) and a Haste on the serfs.
    for (name, mastery) in [("ray", 3), ("haste", 1)] {
        let id = spell(&r, name);
        let timed = TimedCast {
            id,
            spell: r.spell(id).unwrap(),
            mastery,
            rounds: 5,
            own: crate::magic::SpellBonus::default(),
        };
        effects::apply(&mut s, &r, timed, StackId(1), &mut Vec::<Event>::new());
    }
    let lowered = i32::from(s.stacks()[1].defense) - 5;
    assert_eq!(s.stacks()[1].defense_now(), lowered);
    kill(&mut s, &r, 1);
    // The death ended Haste, not the ray (review r2a F-3, r5b F-2: the original's end routine
    // refuses Disrupting Ray, and neither its death nor its raise gives the defense back).
    let ray = vec![(spell(&r, "ray"), None, -5)];
    let entries = |s: &BattleState| -> Vec<_> {
        let of = s.effects.of(StackId(1)).iter();
        of.map(|e| (e.spell, e.rounds, e.defense)).collect()
    };
    assert_eq!(entries(&s), ray);
    assert_eq!(s.stacks()[1].defense_now(), lowered);
    // The ray's entry is public while the stack is dead, as it is on a living one.
    let seen = crate::observe(&s, &r, BattleSide::Defender).effects;
    assert_eq!(seen.len(), 1);
    let events = cast(&mut s, &r, "rise", (stack(1), CastTarget::None)).unwrap();
    assert_eq!(raised(&events), vec![(1, 40, 4, false)]);
    let serfs = s.stacks()[1];
    assert_eq!(entries(&s), ray);
    assert_eq!(
        (serfs.defense_now(), serfs.spells.speed),
        (lowered, serfs.speed)
    );
}

// covers: BAT-MAG-035
#[test]
fn a_corpse_is_raised_only_from_a_free_hex_and_from_the_top() {
    let r = ruleset();
    let mut s = arena(
        &r,
        [Some(raiser(&r, 3, 0)), None],
        &[
            ("fixed", 1, 2, 1),
            ("serf", 2, 5, 5),
            ("serf", 3, 7, 5),
            ("ghoul", 2, 9, 5),
            ("ogre", 2, 11, 7),
        ],
        &[("serf", 1, 14, 1)],
        1,
    );
    // Serfs 1 and 2 die on the same hex, 2 last: only 2 is on top.
    kill(&mut s, &r, 1);
    place(&mut s, &r, 2, 5, 5);
    kill(&mut s, &r, 2);
    assert_eq!((s.corpse(StackId(1)), s.corpse(StackId(2))), (1, 2));
    assert_eq!(targets(&s, &r, "rise"), vec![2]);
    // A corpse that does not pass the test does not cover one below it: a ghoul on top.
    place(&mut s, &r, 3, 5, 5);
    kill(&mut s, &r, 3);
    assert_eq!(targets(&s, &r, "rise"), vec![2]);
    assert_eq!(targets(&s, &r, "animate"), vec![3]);
    // A living stack on the hex hides every corpse there.
    place(&mut s, &r, 0, 5, 5);
    assert_eq!(targets(&s, &r, "rise"), Vec::<u8>::new());
    place(&mut s, &r, 0, 2, 1);
    // A dead two-hex stack needs both its hexes free.
    kill(&mut s, &r, 4);
    let ogre = s.stacks()[4];
    assert_eq!(targets(&s, &r, "rise"), vec![2, 4]);
    let tail = ogre
        .footprint(field(&r))
        .hexes()
        .find(|&h| h != ogre.hex)
        .unwrap();
    s.stacks.as_mut_slice()[0].hex = tail;
    assert_eq!(targets(&s, &r, "rise"), vec![2]);
}

// covers: BAT-MAG-035
#[test]
fn animate_dead_raises_only_undead_and_lasts() {
    let r = ruleset();
    let mut s = arena(
        &r,
        [Some(raiser(&r, 1, 0)), None],
        &[("ghoul", 5, 2, 5), ("serf", 5, 2, 7)],
        &[("ghoul", 3, 14, 5)],
        1,
    );
    s.stacks.as_mut_slice()[0].count = 2;
    s.stacks.as_mut_slice()[1].count = 2;
    s.stacks.as_mut_slice()[2].count = 1;
    // Only the caster's undead.
    assert_eq!(targets(&s, &r, "animate"), vec![0]);
    // Basic: 3 × 50 + 30 = 180, capped: three back for good.
    let events = cast(&mut s, &r, "animate", (stack(0), CastTarget::None)).unwrap();
    assert_eq!(raised(&events), vec![(0, 30, 3, false)]);
    assert_eq!(s.temporary(StackId(0)), 0);
}

// covers: BAT-MAG-036
#[test]
fn sacrifice_kills_its_victim_without_a_corpse_and_raises_its_target() {
    let r = ruleset();
    let mut s = arena(
        &r,
        [Some(raiser(&r, 0, 1)), None],
        &[("fixed", 1, 2, 1), ("seraph", 3, 5, 5), ("serf", 5, 7, 5)],
        &[("serf", 1, 14, 5)],
        1,
    );
    kill(&mut s, &r, 1);
    // Without quirk BAT-QUIRK-5 the dead seraphs are a target by Sacrifice's own test.
    assert_eq!(targets(&s, &r, "offering"), vec![1]);
    // Victims: a stack of living creatures of the side, other than the target.
    assert_eq!(victims(&s, &r, 1), vec![2]);
    // Basic: (10 + 3 × 1 + 3) × 5 = 80 hit points: one seraph with 80.
    let events = cast(&mut s, &r, "offering", (stack(1), stack(2))).unwrap();
    let sacrificed: Vec<_> = events
        .iter()
        .filter_map(|e| match e.kind {
            EventKind::Sacrificed { victim, creatures } => Some((victim.0, creatures)),
            _ => None,
        })
        .collect();
    assert_eq!(sacrificed, vec![(2, 5)]);
    assert_eq!(raised(&events), vec![(1, 80, 1, false)]);
    let seraphs = s.stacks()[1];
    assert_eq!((seraphs.count, seraphs.top_hp), (1, 80));
    // The victim left no corpse: nothing raises it.
    assert_eq!(s.corpse(StackId(2)), NO_CORPSE);
    s.heroes[0].as_mut().unwrap().view.casts = 0;
    set_masteries(&mut s, 0, &r, [3, 1, 0, 0]);
    assert_eq!(targets(&s, &r, "rise"), vec![1]);
}

// covers: BAT-MAG-036
#[test]
fn sacrifice_victims_are_own_living_controlled_stacks() {
    let r = ruleset();
    let mut s = arena(
        &r,
        [Some(raiser(&r, 0, 1)), None],
        &[
            ("serf", 5, 2, 1),
            ("serf", 2, 2, 3),
            ("ghoul", 2, 2, 5),
            ("serf", 2, 2, 7),
        ],
        &[("serf", 1, 14, 5)],
        1,
    );
    s.stacks.as_mut_slice()[0].count = 3;
    // An undead stack is no victim, nor is a hypnotized one, nor an enemy, nor the target.
    let hypnotized = s.stacks()[3].spells.status.with(Status::HYPNOTIZED, true);
    s.stacks.as_mut_slice()[3].spells.status = hypnotized;
    assert_eq!(victims(&s, &r, 0), vec![1]);
    assert_eq!(
        cast(&mut s, &r, "offering", (stack(0), stack(4))),
        Err(CommandError::InvalidSpellTarget)
    );
    assert_eq!(
        cast(&mut s, &r, "offering", (stack(0), stack(0))),
        Err(CommandError::InvalidSpellTarget)
    );
    assert_eq!(
        cast(&mut s, &r, "offering", (stack(0), CastTarget::None)),
        Err(CommandError::InvalidSpellTarget)
    );
    // Without a victim the spell has no target.
    s.stacks.as_mut_slice()[1].spells.status = hypnotized;
    assert!(!castable_spells(&s, &r, BattleSide::Attacker).contains(spell(&r, "offering")));
}

// covers: BAT-MAG-036, BAT-MAG-001
#[test]
fn sacrificing_the_active_stack_ends_its_turn() {
    let r = ruleset();
    let mut s = arena(
        &r,
        [Some(raiser(&r, 0, 1)), None],
        &[("serf", 2, 2, 1), ("serf", 4, 2, 5)],
        &[("serf", 1, 14, 5)],
        1,
    );
    s.stacks.as_mut_slice()[1].count = 1;
    assert_eq!(s.active(), Some(StackId(0)));
    cast(&mut s, &r, "offering", (stack(1), stack(0))).unwrap();
    assert!(!s.stacks()[0].is_alive());
    assert_ne!(s.active(), Some(StackId(0)));
    assert_eq!(s.stacks()[1].count, 4);
}

// covers: BAT-MAG-036
#[test]
fn with_bat_quirk_5_sacrifice_tests_its_target_as_resurrection() {
    let r = ruleset_with("t:sacrifice", |_| {});
    let mut s = arena(
        &r,
        [Some(raiser(&r, 0, 1)), None],
        &[
            ("fixed", 1, 2, 1),
            ("seraph", 3, 5, 5),
            ("gilded", 2, 7, 5),
            ("serf", 5, 9, 5),
        ],
        &[("serf", 1, 14, 5)],
        1,
    );
    s.stacks.as_mut_slice()[1].count = 2;
    s.stacks.as_mut_slice()[2].count = 1;
    // Resurrection's hit points at the hero's air mastery (none: 3 × 50 + 40 = 190) do not
    // reach a seraph's 200, and Resurrection's level 4 does not reach the gilded (S-15).
    assert_eq!(targets(&s, &r, "offering"), Vec::<u8>::new());
    // At Advanced air (230) the seraphs are a target; the gilded never are.
    set_masteries(&mut s, 0, &r, [2, 1, 0, 0]);
    assert_eq!(targets(&s, &r, "offering"), vec![1]);
    // Without the quirk both are, by Sacrifice's own level 5.
    let plain = ruleset();
    let mut t = s.clone();
    set_masteries(&mut t, 0, &r, [0, 1, 0, 0]);
    assert_eq!(targets(&t, &plain, "offering"), vec![1, 2]);
}

// covers: BAT-MAG-036
#[test]
fn with_bat_quirk_5_the_raised_stack_passes_sacrifices_own_test_too() {
    let r = ruleset_with("t:sacrifice", |_| {});
    // Expert air: Resurrection's 3 × 50 + 160 = 310 hit points reach every creature here.
    let mut hero = mage(&r, &["rise", "offering", "serfs"]);
    hero.magic.masteries = masteries(&r, [3, 1, 0, 0]);
    hero.magic.mana = 200;
    let mut s = arena(
        &r,
        [Some(hero), None],
        &[
            ("fixed", 1, 2, 1),
            ("ember", 3, 5, 5),
            ("serf", 5, 7, 5),
            ("serf", 5, 9, 5),
        ],
        &[("serf", 1, 14, 5)],
        1,
    );
    s.stacks.as_mut_slice()[1].count = 1;
    s.stacks.as_mut_slice()[2].count = 4;
    // The embers (living, immune to the fire school) pass Resurrection's test, by which the
    // original finds the stack; it then asks Sacrifice's own test of the stack it found (the
    // target test 0x5A39C0 calls 0x5A8090 for the spell itself), and a fire spell does not
    // affect them. (With the quirk only Resurrection's test was asked: review r4a F-1.)
    assert_eq!(targets(&s, &r, "rise"), vec![1, 2]);
    assert_eq!(targets(&s, &r, "offering"), vec![2]);
    assert_eq!(
        cast(&mut s, &r, "offering", (stack(1), stack(3))),
        Err(CommandError::InvalidSpellTarget)
    );
    // The stack is found by Resurrection's test alone: dead embers on top of dead serfs are
    // found and then refused, and the serfs below them are not looked at.
    kill(&mut s, &r, 2);
    place(&mut s, &r, 1, 7, 5);
    kill(&mut s, &r, 1);
    assert_eq!(targets(&s, &r, "rise"), vec![1]);
    assert_eq!(targets(&s, &r, "offering"), Vec::<u8>::new());
    // Without the quirk Sacrifice's own test alone picks the corpse: the serfs.
    let plain = ruleset();
    assert_eq!(targets(&s, &plain, "offering"), vec![2]);
    // Nor is a stack the battle created a target (the original's flag `0x400000`), though
    // Resurrection restores it: six summoned serfs, four of them lost.
    cast(&mut s, &r, "serfs", (CastTarget::None, CastTarget::None)).unwrap();
    s.heroes[0].as_mut().unwrap().view.casts = 0;
    assert!(s.stacks()[5].is_created());
    assert_eq!((s.stacks()[5].count, s.stacks()[5].base_count), (6, 6));
    s.stacks.as_mut_slice()[5].count = 2;
    assert_eq!(targets(&s, &r, "rise"), vec![1, 5]);
    assert_eq!(targets(&s, &r, "offering"), Vec::<u8>::new());
}

// covers: BAT-MAG-001
#[test]
fn a_cast_that_kills_the_active_stack_ends_its_turn() {
    let r = ruleset();
    let mut hero = mage(&r, &["storm"]);
    hero.magic.mana = 100;
    let mut s = arena(
        &r,
        [Some(hero), None],
        &[("serf", 1, 2, 1), ("tank", 1, 2, 5)],
        &[("tank", 1, 14, 5)],
        1,
    );
    // 3 × 10 + 10 = 40 damage to every stack: the active serf dies, its turn passes.
    let events = cast(&mut s, &r, "storm", (CastTarget::None, CastTarget::None)).unwrap();
    assert!(!s.stacks()[0].is_alive());
    assert_eq!(s.active(), Some(StackId(2)));
    assert!(
        events
            .iter()
            .any(|e| e.kind == EventKind::TurnStarted { stack: StackId(2) })
    );
}

// covers: BAT-MAG-035, BAT-MAG-036, BAT-MAG-021
#[test]
fn anti_magic_keeps_a_raise_off_a_living_stack_and_a_victim_off_a_sacrifice() {
    let r = ruleset();
    let mut s = arena(
        &r,
        [Some(raiser(&r, 1, 1)), None],
        &[("fixed", 1, 2, 1), ("serf", 5, 2, 5), ("serf", 5, 2, 7)],
        &[("serf", 1, 14, 5)],
        1,
    );
    s.stacks.as_mut_slice()[1].count = 3;
    s.stacks.as_mut_slice()[2].count = 3;
    assert_eq!(targets(&s, &r, "rise"), vec![1, 2]);
    assert_eq!(victims(&s, &r, 1), vec![2]);
    // Anti-Magic above the spells' levels (4, 5) on stack 2: no raise, no sacrifice.
    s.stacks.as_mut_slice()[2].spells.spell_immunity = 6;
    assert_eq!(targets(&s, &r, "rise"), vec![1]);
    assert_eq!(victims(&s, &r, 1), Vec::<u8>::new());
}

// covers: BAT-MAG-035
#[test]
fn a_corpse_under_a_wall_or_below_corpses_that_do_not_count() {
    let r = ruleset();
    let mut s = arena(
        &r,
        [Some(raiser(&r, 3, 0)), None],
        &[("fixed", 1, 2, 1), ("serf", 2, 5, 5), ("ogre", 2, 9, 5)],
        &[("serf", 1, 14, 1), ("serf", 1, 13, 9)],
        1,
    );
    // The own serfs die on (5, 5); an enemy serf dies there later: it does not hide them.
    kill(&mut s, &r, 1);
    place(&mut s, &r, 3, 5, 5);
    kill(&mut s, &r, 3);
    assert_eq!(targets(&s, &r, "rise"), vec![1]);
    // A two-hex corpse above them whose other hex is held does not hide them either.
    place(&mut s, &r, 2, 5, 5);
    kill(&mut s, &r, 2);
    let ogre = s.stacks()[2];
    let tail = ogre
        .footprint(field(&r))
        .hexes()
        .find(|&h| h != ogre.hex)
        .unwrap();
    assert_eq!(targets(&s, &r, "rise"), vec![2]);
    s.stacks.as_mut_slice()[4].hex = tail;
    assert_eq!(targets(&s, &r, "rise"), vec![1]);
    // A wall over the hex (Force Field) keeps every corpse there.
    let wall = spell(&r, "wall");
    let shape = r.spell(wall).unwrap().magic.obstacle.clone().unwrap();
    let hexes = [at(&r, 5, 5)];
    crate::obstacles::place(
        &mut s,
        (wall, h3_content::ObstacleKind::Wall, &shape),
        BattleSide::Defender,
        (&hexes, 0),
        &mut Vec::<Event>::new(),
    );
    assert_eq!(targets(&s, &r, "rise"), Vec::<u8>::new());
}

// covers: BAT-MAG-035, BAT-MAG-020
#[test]
fn a_specialist_raises_more_and_reaches_more_health() {
    let r = ruleset();
    let mut hero = raiser(&r, 1, 0);
    // A level-6 specialist in Resurrection: 3 % of the hit points per level step.
    hero.effects
        .push(h3_contracts::BattleEffect::SpellSpecialty {
            spell: spell(&r, "rise"),
            bp_per_level: 300,
            level: 6,
        });
    let mut s = arena(
        &r,
        [Some(hero), None],
        &[("seraph", 3, 2, 5)],
        &[("serf", 1, 14, 5)],
        1,
    );
    s.stacks.as_mut_slice()[0].count = 1;
    // 190 hit points plus ⌈190 × 3 % × 6⌉ = 35: 225 reach a seraph's 200.
    assert_eq!(targets(&s, &r, "rise"), vec![0]);
    let events = cast(&mut s, &r, "rise", (stack(0), CastTarget::None)).unwrap();
    assert_eq!(raised(&events), vec![(0, 225, 2, true)]);
}

// covers: BAT-MAG-036
#[test]
fn with_bat_quirk_5_a_dead_target_takes_resurrections_test_too() {
    let r = ruleset_with("t:sacrifice", |_| {});
    let mut s = arena(
        &r,
        [Some(raiser(&r, 0, 1)), None],
        &[("fixed", 1, 2, 1), ("seraph", 1, 5, 5), ("serf", 5, 9, 5)],
        &[("serf", 1, 14, 5)],
        1,
    );
    kill(&mut s, &r, 1);
    // 190 hit points of Resurrection do not reach a seraph's 200: no target, dead or alive.
    assert_eq!(targets(&s, &r, "offering"), Vec::<u8>::new());
    set_masteries(&mut s, 0, &r, [2, 1, 0, 0]);
    assert_eq!(targets(&s, &r, "offering"), vec![1]);
}

// covers: BAT-MAG-035
#[test]
fn a_raised_stack_that_had_not_acted_gets_its_turn() {
    let r = ruleset();
    let mut s = arena(
        &r,
        [Some(raiser(&r, 3, 0)), None],
        &[("fixed", 1, 2, 1), ("serf", 4, 5, 5)],
        &[("tank", 1, 14, 5)],
        1,
    );
    kill(&mut s, &r, 1);
    cast(&mut s, &r, "rise", (stack(1), CastTarget::None)).unwrap();
    let order = order(&mut s, &r, 3);
    assert!(order.contains(&1), "{order:?}");
}

// covers: BAT-MAG-035
#[test]
fn stacks_killed_in_automatic_turns_of_one_command_lie_in_their_order() {
    let r = ruleset();
    let mut s = arena(
        &r,
        [Some(raiser(&r, 3, 0)), None],
        &[("fixed", 1, 2, 1), ("serf", 2, 5, 5), ("serf", 2, 7, 5)],
        &[("serf", 1, 14, 5)],
        1,
    );
    // Stack 2 dies, then stack 1 on the same hex, each marked before the next activation.
    place(&mut s, &r, 2, 5, 5);
    s.stacks.as_mut_slice()[2].count = 0;
    s.stacks.as_mut_slice()[2].top_hp = 0;
    crate::turn::activate_next(&mut s, &r, &mut Vec::<Event>::new());
    s.stacks.as_mut_slice()[1].count = 0;
    s.stacks.as_mut_slice()[1].top_hp = 0;
    crate::turn::activate_next(&mut s, &r, &mut Vec::<Event>::new());
    assert_eq!((s.corpse(StackId(2)), s.corpse(StackId(1))), (1, 2));
    assert_eq!(targets(&s, &r, "rise"), vec![1]);
}

// covers: BAT-MAG-035, BAT-END-003
#[test]
fn a_side_left_with_creatures_raised_for_the_battle_only_keeps_one() {
    let r = ruleset();
    let mut setup = setup(&r, &[(0, "serf", 6), (1, "serf", 4)], &[(0, "serf", 1)]);
    setup.heroes[0] = Some(raiser(&r, 1, 0));
    // A Ballista on the side: the original counts it and keeps nothing (S-20); the engine
    // counts only army stacks (BAT-QUIRK-6).
    setup.war_machines[0].push(r.creature_by_key("t:ballista").unwrap());
    let mut s = BattleState::new(&r, &setup, 1).unwrap();
    place(&mut s, &r, 0, 2, 1);
    place(&mut s, &r, 1, 2, 5);
    activate(&mut s, 0);
    // Both serf stacks die; Basic Resurrection raises them for the battle only.
    kill(&mut s, &r, 0);
    kill(&mut s, &r, 1);
    for id in [0, 1] {
        s.heroes[0].as_mut().unwrap().view.casts = 0;
        cast(&mut s, &r, "rise", (stack(id), CastTarget::None)).unwrap();
    }
    assert_eq!((s.temporary(StackId(0)), s.temporary(StackId(1))), (6, 4));
    let defender = s
        .stacks()
        .iter()
        .position(|st| st.side == BattleSide::Defender)
        .unwrap();
    kill(&mut s, &r, u8::try_from(defender).unwrap());
    crate::combat::check_end(&mut s, &mut Vec::<Event>::new());
    let outcome = s.outcome().unwrap();
    assert_eq!(outcome.result, BattleResult::Victory(BattleSide::Attacker));
    // The last living army stack keeps one creature; the Ballista survives as it is.
    let ends: Vec<u32> = outcome.stacks[0].iter().map(|rec| rec.end).collect();
    assert_eq!(ends, vec![0, 1, 1]);
    assert_eq!(outcome.survivors[0].stacks.len(), 1);
    assert_eq!(
        (
            outcome.survivors[0].stacks[0].slot,
            outcome.survivors[0].stacks[0].count
        ),
        (1, 1)
    );
}
