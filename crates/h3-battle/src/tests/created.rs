//! Stacks a battle creates (E04b T6b): what they share (BAT-MAG-037), Clone (BAT-MAG-038, quirk
//! BAT-QUIRK-7) and Summon Elemental (BAT-MAG-039), after the original's code
//! (`docs/formats/installation.md` §7.19).

use std::vec;
use std::vec::Vec;

use h3_content_battle::BattleRuleset;
use h3_contracts::{BattleHero, BattleResult, BattleSide, StackOrigin, StackRecord};

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
fn creator(r: &BattleRuleset, air: u8, fire: u8) -> BattleHero {
    let mut h = mage(r, &["copy", "sprites", "serfs", "offering"]);
    h.magic.masteries = masteries(r, [air, fire, 0, 0]);
    h.magic.mana = 100;
    h
}

/// The stacks `side` may aim `name` at now, by id.
fn targets(s: &BattleState, r: &BattleRuleset, side: BattleSide, name: &str) -> Vec<u8> {
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
    r: &BattleRuleset,
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
fn hurt(s: &mut BattleState, r: &BattleRuleset, id: u8, damage: u64) -> Vec<Event> {
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
    set_masteries(&mut s, 0, &r, [3, 0, 0, 0]);
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
    // Observations link the clone and its original (public, E04b T9a).
    let links = crate::observe(&s, &r, BattleSide::Defender).clone_links;
    assert_eq!(
        (links[0], links[4], links[1]),
        (Some(StackId(4)), Some(StackId(0)), None)
    );
    // Power 3, no duration bonus: three rounds.
    assert_eq!(
        (s.clone_rounds(StackId(4)), s.clone_rounds(StackId(0))),
        (3, 0)
    );
    // Neither a cloned stack nor a clone is a target.
    assert_eq!(targets(&s, &r, BattleSide::Attacker, "copy"), vec![1]);
    // The outcome reports the clone as created, after the army, and leaves it out of the
    // survivors.
    hurt(&mut s, &r, 3, 1000);
    crate::combat::check_end(&mut s, &mut Vec::<Event>::new());
    let outcome = s.outcome().unwrap();
    assert_eq!(outcome.result, BattleResult::Victory(BattleSide::Attacker));
    let origins: Vec<_> = outcome.stacks[0].iter().map(|r| r.origin).collect();
    assert_eq!(
        origins,
        [0, 1, 2]
            .map(StackOrigin::Army)
            .into_iter()
            .chain([StackOrigin::Created])
            .collect::<Vec<_>>()
    );
    assert_eq!(
        (outcome.stacks[0][3].start, outcome.stacks[0][3].end),
        (10, 10)
    );
    assert_eq!(outcome.survivors[0].stacks.len(), 3);
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
    // Left of it is skipped in ring 1 too (E04b T11b review, r4a F-10): with the top right, the
    // bottom right and the bottom left neighbour taken the clone goes top left, not left.
    assert_eq!(
        place_of(
            &[
                ("serf", 5, 5, 5),
                ("serf", 1, 5, 4),
                ("serf", 1, 5, 6),
                ("serf", 1, 4, 6)
            ],
            0
        ),
        Some(at(&r, 4, 4).0)
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

/// An arena with `ogres` (tail 13, head 14, row 5) of the attacker's under a Clone caster, the
/// serfs of the defender at (1, 0), every standable hex but the ogres', `free` and the serfs'
/// blocked when `free` is given, else rows 3 to 7 from column 9 on but the ogres' hexes and the
/// hex in front of them.
fn walled(r: &BattleRuleset, ogres: u32, free: Option<&[(i32, i32)]>) -> BattleState {
    let mut s = arena(
        r,
        [Some(creator(r, 1, 0)), None],
        &[("ogre", ogres, 14, 5)],
        &[("serf", 1, 1, 0)],
        1,
    );
    let field = Field::new(r.battle().field);
    let keep: Vec<_> = [(13, 5), (14, 5), (1, 0)]
        .iter()
        .chain(free.unwrap_or(&[(15, 5)]))
        .map(|&(x, y)| at(r, x, y))
        .collect();
    let walls: Vec<u8> = match free {
        Some(_) => (0..field.hex_count())
            .map(|h| crate::geometry::Hex(u8::try_from(h).unwrap()))
            .filter(|&h| field.is_standable(h))
            .collect::<Vec<_>>(),
        None => (9..16)
            .flat_map(|x| (3..8).map(move |y| (x, y)))
            .map(|(x, y)| at(r, x, y))
            .collect(),
    }
    .into_iter()
    .filter(|h| !keep.contains(h))
    .map(|h| h.0)
    .collect();
    s.natural = NaturalObstacles::new(field, &blocked(walls), false).unwrap();
    s
}

/// The originals Clone turned into clones as `(stack, rounds)`.
fn turned(events: &[Event]) -> Vec<(u8, u16)> {
    events
        .iter()
        .filter_map(|e| match e.kind {
            EventKind::TurnedSummoned { stack, rounds } => Some((stack.0, rounds)),
            _ => None,
        })
        .collect()
}

// covers: BAT-MAG-038
#[test]
fn a_crowded_two_hex_stack_is_never_covered_by_its_clone() {
    // Quirk BAT-QUIRK-7 fixed: rows 3 to 7 blocked from column 9 on but for the ogres and the hex
    // in front of them. The original would take the ogres' head as the clone's position in ring
    // 2; the engine goes on to ring 3, top right.
    let r = ruleset();
    let mut s = walled(&r, 2, None);
    let events = cast(&mut s, &r, "copy", stack(0)).unwrap();
    assert_eq!(created(&events), vec![(2, 2, at(&r, 15, 2).0, Some(0))]);
    assert_eq!(turned(&events), vec![]);
    // Fully walled in: no place, the mana is spent all the same.
    let field = Field::new(r.battle().field);
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

// covers: BAT-MAG-038, BAT-MAG-037, BAT-END-003
#[test]
fn with_quirk_bat_quirk_7_a_crowded_original_becomes_its_own_clone() {
    // The clone's place in ring 2 is the ogres' head: no clone, the ogres are summoned for the
    // clone's three rounds and no longer of the army; damage does not take all of them.
    let r = ruleset_with("t:clone_on_original", |_| {});
    let mut s = walled(&r, 3, None);
    let events = cast(&mut s, &r, "copy", stack(0)).unwrap();
    assert_eq!((created(&events), turned(&events)), (vec![], vec![(0, 3)]));
    let ogres = s.stacks()[0];
    assert!(ogres.is_created() && !ogres.has(StackTraits::CLONE));
    assert_eq!((s.stacks().len(), s.clone_rounds(StackId(0))), (2, 3));
    // Not clonable again.
    assert_eq!(targets(&s, &r, BattleSide::Attacker, "copy"), vec![]);
    let health = u64::from(ogres.max_hp);
    hurt(&mut s, &r, 0, health);
    assert_eq!(s.stacks()[0].count, 2);
    // Its rounds run out: it vanishes with its two creatures, without a corpse, and the battle
    // ends with the attacker's army gone (at that round start: E04b T11b review, F-r4a-3; this
    // test called `check_end` by hand); the outcome reports it as created, with the creature
    // it lost and the two it vanished with.
    let mut events = Vec::new();
    for _ in 0..3 {
        assert!(!s.is_over());
        crate::turn::start_round(&mut s, &r, &mut events);
    }
    assert_eq!(vanished(&events), vec![(0, 2)]);
    assert_eq!((s.stacks()[0].count, s.corpse(StackId(0))), (0, NO_CORPSE));
    let outcome = s.outcome().unwrap();
    assert_eq!(outcome.result, BattleResult::Victory(BattleSide::Defender));
    let ogre = r.creature_by_key("t:ogre").unwrap();
    assert_eq!(
        outcome.stacks[0],
        vec![StackRecord {
            origin: StackOrigin::Created,
            creature: ogre,
            start: 3,
            end: 2,
        }]
    );
    assert!(outcome.survivors[0].stacks.is_empty());
}

/// E04b T11b review (F-r4a-3): with quirk BAT-QUIRK-7 a stack that became its own clone can be
/// its side's last one when its rounds run out at a round's start. The battle ends there, as the
/// original asks for the winner after every action whatever its kind (the reviewer's reading of
/// its action dispatcher, 0x479BDC); it used to go on for ever with only the other side to act.
// covers: BAT-END-001, BAT-MAG-038, BAT-TRN-003
#[test]
fn the_battle_ends_when_a_round_start_takes_a_sides_last_stack() {
    let r = ruleset_with("t:clone_on_original", |_| {});
    // The attacker's only stack, and one serf of the defender without a hero.
    let mut s = walled(&r, 3, None);
    let events = cast(&mut s, &r, "copy", stack(0)).unwrap();
    assert_eq!(turned(&events), vec![(0, 3)]);
    // Both stacks defend, round after round.
    let mut events = Vec::new();
    for _ in 0..12 {
        if s.is_over() {
            break;
        }
        events.extend(act(&mut s, &r, Command::Defend).unwrap());
    }
    // The ogres vanish at the third round start after the cast, and with them the attacker's
    // army: the defender wins there and then, and no stack is active.
    assert_eq!(vanished(&events), vec![(0, 3)]);
    assert_eq!(
        s.result(),
        Some(BattleResult::Victory(BattleSide::Defender))
    );
    assert_eq!((s.round(), s.active()), (4, None));
    let last: Vec<EventKind> = events.iter().rev().take(3).rev().map(|e| e.kind).collect();
    assert_eq!(
        last,
        [
            EventKind::RoundStarted { round: 4 },
            EventKind::Vanished {
                stack: StackId(0),
                creatures: 3
            },
            EventKind::BattleEnded {
                result: BattleResult::Victory(BattleSide::Defender)
            },
        ]
    );
}

/// E04b T11b review (F-r4a-4): with quirk BAT-QUIRK-7 a stack that became its own clone keeps
/// its record when its rounds run out, like every created stack that vanishes so: the original
/// leaves its count, so the stack creation 0x47A100 passes over the record and the end of the
/// battle still counts its losses. The engine gave the record, the army stack's own, to the next
/// created stack, and the outcome lost the stack.
// covers: BAT-MAG-037, BAT-MAG-038, BAT-END-003
#[test]
fn with_quirk_bat_quirk_7_a_vanished_original_keeps_its_record() {
    let r = ruleset_with("t:clone_on_original", |_| {});
    let mut s = walled(&r, 3, None);
    // Sprites (stack 2) keep the attacker in the battle; the ogres become their own clone and
    // lose one creature before their rounds run out.
    let sprites =
        |s: &mut BattleState| created(&cast(s, &r, "sprites", CastTarget::None).unwrap())[0].0;
    assert_eq!(sprites(&mut s), 2);
    let events = cast(&mut s, &r, "copy", stack(0)).unwrap();
    assert_eq!(turned(&events), vec![(0, 3)]);
    let health = u64::from(s.stacks()[0].max_hp);
    hurt(&mut s, &r, 0, health);
    let mut events = Vec::new();
    for _ in 0..3 {
        crate::turn::start_round(&mut s, &r, &mut events);
    }
    assert_eq!(vanished(&events), vec![(0, 2)]);
    assert!(!s.is_over());
    // The next summoned stack takes a new record, not the ogres'.
    activate(&mut s, 2);
    assert_eq!(sprites(&mut s), 3);
    let ogre = r.creature_by_key("t:ogre").unwrap();
    assert_eq!((s.stacks()[0].creature, s.stacks()[0].count), (ogre, 0));
    // The outcome still reports the ogres: one lost, two vanished with the stack.
    hurt(&mut s, &r, 1, 1000);
    crate::combat::check_end(&mut s, &mut Vec::<Event>::new());
    let outcome = s.outcome().unwrap();
    assert_eq!(outcome.result, BattleResult::Draw);
    let sprite = r.creature_by_key("t:sprite").unwrap();
    let record = |creature, start, end| StackRecord {
        origin: StackOrigin::Created,
        creature,
        start,
        end,
    };
    assert_eq!(
        outcome.stacks[0],
        vec![
            record(ogre, 3, 2),
            record(sprite, 6, 6),
            record(sprite, 6, 6)
        ]
    );
}

// covers: BAT-MAG-038
#[test]
fn a_clone_never_shares_a_hex_with_its_original() {
    // Quirk BAT-QUIRK-8, in every profile: only the ogres' hexes and the two hexes behind them
    // are free. With BAT-QUIRK-7 the first place that fits in ring 2 has its front hex on the
    // ogres' tail; it is skipped, and the clone stands right behind them.
    for profile in ["t:faithful", "t:clone_on_original"] {
        let r = ruleset_with(profile, |_| {});
        let mut s = walled(&r, 2, Some(&[(12, 5), (11, 5)]));
        let events = cast(&mut s, &r, "copy", stack(0)).unwrap();
        assert_eq!(
            created(&events),
            vec![(2, 2, at(&r, 12, 5).0, Some(0))],
            "{profile}"
        );
    }
}

// covers: BAT-MAG-038
#[test]
fn the_search_from_a_two_hex_stacks_head_skips_the_way_to_its_tail_in_rings_1_and_2() {
    // E04b T11b review (r4a F-10): only the ogres' hexes (tail (13, 5), head (14, 5)) and a
    // place three hexes to their left are free. From the head, the hexes towards the tail are
    // not tried in rings 1 and 2: there the place would be the ogres' own position, which with
    // quirk BAT-QUIRK-7 counts as free and would turn them into their own clone. So the search
    // reaches ring 3, whose way from the tail comes first, and the clone stands on the free
    // place, in either profile.
    for profile in ["t:faithful", "t:clone_on_original"] {
        let r = ruleset_with(profile, |_| {});
        let mut s = walled(&r, 2, Some(&[(10, 5), (11, 5)]));
        let events = cast(&mut s, &r, "copy", stack(0)).unwrap();
        assert_eq!(
            (created(&events), turned(&events)),
            (vec![(2, 2, at(&r, 11, 5).0, Some(0))], vec![]),
            "{profile}"
        );
    }
    // Ring 3 does try the way from the head to the tail. With the free place one hex further
    // left (ring 4) the search comes to the ogres' own position first: with the quirk they
    // become their own clone for the cast's three rounds; without it their hexes are taken and
    // the clone stands on the free place.
    let place = at(&ruleset(), 10, 5).0;
    for (profile, expected) in [
        ("t:faithful", (vec![(2, 2, place, Some(0))], vec![])),
        ("t:clone_on_original", (vec![], vec![(0, 3)])),
    ] {
        let r = ruleset_with(profile, |_| {});
        let mut s = walled(&r, 2, Some(&[(9, 5), (10, 5)]));
        let events = cast(&mut s, &r, "copy", stack(0)).unwrap();
        assert_eq!((created(&events), turned(&events)), expected, "{profile}");
    }
}

// covers: BAT-MAG-038
#[test]
fn a_clone_vanishes_when_its_rounds_run_out_or_its_original_dies() {
    let r = ruleset();
    let new = |r: &BattleRuleset| {
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
    // Observations show which creature each side's summons are bound to (E04b T9a).
    let summoned = crate::observe(&s, &r, BattleSide::Defender).summoned;
    assert_eq!(summoned, [Some(sprite.creature), None]);
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

// covers: BAT-MAG-039
#[test]
fn a_summoned_stack_takes_a_random_fitting_row_and_none_without_a_place() {
    // E04b T11b review (r4a F-10): the row is drawn with equal chances among the rows of the
    // first column where the stack fits. Rows 0 to 2 of the attacker's column 1 are blocked:
    // over 320 battles each of the other eight rows is drawn about forty times.
    let r = ruleset();
    let field = Field::new(r.battle().field);
    let start = |seed: u64, walls: Vec<u8>| {
        let mut s = arena(
            &r,
            [Some(creator(&r, 0, 1)), None],
            &[("serf", 1, 5, 5)],
            &[("serf", 1, 10, 5)],
            seed,
        );
        s.natural = NaturalObstacles::new(field, &blocked(walls), false).unwrap();
        s
    };
    let mut rows = [0u32; 11];
    for seed in 0..320 {
        let mut s = start(seed, (0..3).map(|y| at(&r, 1, y).0).collect());
        let events = cast(&mut s, &r, "sprites", CastTarget::None).unwrap();
        let (x, y) = field.coords(crate::geometry::Hex(created(&events)[0].2));
        assert_eq!(x, 1);
        rows[usize::try_from(y).unwrap()] += 1;
    }
    assert_eq!(rows[..3], [0, 0, 0]);
    assert!(rows[3..].iter().all(|n| (20..=60).contains(n)), "{rows:?}");
    // No place in any column: nothing is created, the cast is paid for all the same (4 at
    // Basic), and the side's summons are bound to the creature from then on.
    let walls: Vec<u8> = (0..field.hex_count())
        .map(|h| u8::try_from(h).unwrap())
        .filter(|&h| field.is_standable(crate::geometry::Hex(h)))
        .filter(|&h| h != at(&r, 5, 5).0 && h != at(&r, 10, 5).0)
        .collect();
    let mut s = start(1, walls);
    let mana = s.hero(BattleSide::Attacker).unwrap().mana;
    let events = cast(&mut s, &r, "sprites", CastTarget::None).unwrap();
    assert_eq!((created(&events), s.stacks().len()), (vec![], 2));
    assert_eq!(s.hero(BattleSide::Attacker).unwrap().mana, mana - 4);
    let sprite = r.creature_by_key("t:sprite").unwrap();
    assert_eq!(
        crate::observe(&s, &r, BattleSide::Defender).summoned,
        [Some(sprite), None]
    );
    let castable = castable_spells(&s, &r, BattleSide::Attacker);
    assert!(castable.contains(spell(&r, "sprites")) && !castable.contains(spell(&r, "serfs")));
}

// covers: BAT-END-003, BAT-MAG-037, BAT-MAG-038
#[test]
fn the_outcome_reports_each_created_stack_with_its_losses() {
    // BAT-Q7, BAT-Q13 (batch 2d NE: Necromancy counts the Pikemen, their clone and the Earth
    // Elementals, per stack): the summoned sprites die and a second summon takes their slot,
    // which then reports only the second stack, as the original's records; the clone dies with
    // its original and has lost all its creatures.
    let r = ruleset();
    let mut s = arena(
        &r,
        [Some(creator(&r, 1, 1)), None],
        &[("serf", 10, 5, 5)],
        &[("serf", 1, 14, 5)],
        1,
    );
    let first = created(&cast(&mut s, &r, "sprites", CastTarget::None).unwrap());
    hurt(&mut s, &r, first[0].0, 1000);
    let second = created(&cast(&mut s, &r, "sprites", CastTarget::None).unwrap());
    assert_eq!(second[0].0, first[0].0);
    let clone = created(&cast(&mut s, &r, "copy", stack(0)).unwrap());
    assert_eq!(clone[0].0, 3);
    hurt(&mut s, &r, 0, 1000);
    hurt(&mut s, &r, 1, 1000);
    crate::combat::check_end(&mut s, &mut Vec::<Event>::new());
    let outcome = s.outcome().unwrap();
    assert_eq!(outcome.result, BattleResult::Draw);
    let (serf, sprite) = (
        r.creature_by_key("t:serf").unwrap(),
        s.stacks()[usize::from(first[0].0)].creature,
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
            record(StackOrigin::Army(0), serf, 10, 0),
            record(StackOrigin::Created, sprite, second[0].1, second[0].1),
            record(StackOrigin::Created, serf, 10, 0),
        ]
    );
    assert!(outcome.survivors[0].stacks.is_empty());
}

// covers: BAT-MAG-037, BAT-MAG-044
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
    // A poisoned created stack dies; its slot's new stack keeps nothing of it (BAT-MAG-044).
    s.vitality.base[3] = s.stacks()[3].max_hp;
    s.vitality.poison[3] = 3;
    hurt(&mut s, &r, 4, 1000);
    hurt(&mut s, &r, 3, 1000);
    mark_corpses(&mut s);
    assert_eq!(sprites(&mut s), 3);
    assert_eq!((s.stacks().len(), s.corpse(StackId(3))), (5, 0));
    assert_eq!((s.vitality.base[3], s.vitality.poison[3]), (0, 0));
    let full = s.stacks()[3].max_hp;
    effects::recompute(&mut s, &r, StackId(3));
    assert_eq!(s.stacks()[3].max_hp, full);
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

/// E04b T11b review (F-r4a-4): a created stack whose rounds ran out keeps its record. The
/// original's stack creation takes an empty record or one that holds a dead summoned stack with
/// no creatures (the reviewer's reading of 0x47A100, 0x47A131 to 0x47A14A; `installation.md`
/// §7.19), and a stack that vanishes at the end of its rounds keeps its count (its death leaves
/// it, which BAT-END-003 relies on): only a killed created stack's record is taken again. The
/// engine reused any dead created stack's record, so a later created stack took the expired
/// stack's id and its place in the turn order, and the outcome lost the expired stack's record.
// covers: BAT-MAG-037, BAT-MAG-038, BAT-END-003, BAT-TRN-001
#[test]
fn a_created_stack_whose_rounds_ran_out_keeps_its_record() {
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
    // A clone of the serfs (stack 2), then sprites (stack 3); the clone's three rounds run out.
    let clone = created(&cast(&mut s, &r, "copy", stack(0)).unwrap())[0].0;
    assert_eq!((clone, sprites(&mut s)), (2, 3));
    let mut events = Vec::new();
    for _ in 0..3 {
        crate::turn::start_round(&mut s, &r, &mut events);
    }
    assert_eq!(vanished(&events), vec![(2, 10)]);
    assert!(!s.stacks()[2].is_alive());
    // The next created stack takes a new record and not the clone's: its id is the last.
    activate(&mut s, 0);
    assert_eq!(sprites(&mut s), 4);
    assert_eq!(s.stacks().len(), 5);
    // So of two created stacks of equal speed the earlier one acts first (BAT-TRN-001: by id).
    let both: Vec<u8> = order(&mut s.clone(), &r, 8)
        .into_iter()
        .filter(|id| [3, 4].contains(id))
        .collect();
    assert_eq!(both, [3, 4, 3, 4]);
    // A killed created stack's record is taken as before: the first sprites die and the next
    // stack takes their record, passing over the clone's.
    hurt(&mut s, &r, 3, 1000);
    mark_corpses(&mut s);
    assert_eq!(sprites(&mut s), 3);
    assert_eq!(s.stacks().len(), 5);
    // The outcome keeps the clone's record with the creatures it vanished with, which were
    // not killed (BAT-END-003).
    hurt(&mut s, &r, 1, 1000);
    crate::combat::check_end(&mut s, &mut Vec::<Event>::new());
    let outcome = s.outcome().unwrap();
    let (serf, sprite) = (
        r.creature_by_key("t:serf").unwrap(),
        r.creature_by_key("t:sprite").unwrap(),
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
            record(StackOrigin::Army(0), serf, 10, 10),
            record(StackOrigin::Created, serf, 10, 10),
            record(StackOrigin::Created, sprite, 6, 6),
            record(StackOrigin::Created, sprite, 6, 6),
        ]
    );
}

/// E04b T11b review (F-r4a-4): the records of created stacks whose rounds ran out stay in use,
/// so they count towards the side's stack limit (the original's records in use never fall): a
/// side that keeps cloning runs out of records. The engine reused the expired clone's record and
/// never reached the limit so.
// covers: BAT-MAG-037, BAT-MAG-038
#[test]
fn created_stacks_whose_rounds_ran_out_count_towards_the_limit() {
    let r = ruleset_with("t:faithful", |b| b.side_stack_limit = 3);
    let mut s = arena(
        &r,
        [Some(creator(&r, 1, 0)), None],
        &[("serf", 10, 5, 5)],
        &[("serf", 1, 14, 5)],
        1,
    );
    // Two clones, each after the one before ran out: with the army's stack, three records.
    for id in [2, 3] {
        s.heroes[0].as_mut().unwrap().view.mana = 100;
        let events = cast(&mut s, &r, "copy", stack(0)).unwrap();
        assert_eq!(created(&events)[0].0, id);
        let mut events = Vec::new();
        for _ in 0..3 {
            crate::turn::start_round(&mut s, &r, &mut events);
        }
        assert_eq!(vanished(&events), vec![(id, 10)]);
        activate(&mut s, 0);
    }
    // The side is at its limit: no third clone, though the original stack is free again.
    assert_eq!(s.stacks().len(), 4);
    assert_eq!(targets(&s, &r, BattleSide::Attacker, "copy"), vec![]);
    assert!(!castable_spells(&s, &r, BattleSide::Attacker).contains(spell(&r, "copy")));
    assert_eq!(
        cast(&mut s, &r, "copy", stack(0)),
        Err(CommandError::InvalidSpellTarget)
    );
}

// covers: BAT-MAG-037, BAT-MAG-010
#[test]
fn a_reused_record_starts_without_the_dead_stacks_permanent_effects() {
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
    assert_eq!(sprites(&mut s), 2);
    // An Expert Disrupting Ray (for the rest of the battle) and a Haste on the summoned stack.
    for name in ["ray", "haste"] {
        let id = spell(&r, name);
        let timed = effects::TimedCast {
            id,
            spell: r.spell(id).unwrap(),
            mastery: 3,
            rounds: 5,
            own: crate::magic::SpellBonus::default(),
        };
        effects::apply(&mut s, &r, timed, StackId(2), &mut Vec::<Event>::new());
    }
    let whole = s.stacks()[2].defense_now() + 5;
    assert_eq!(whole, i32::from(s.stacks()[2].defense));
    // It dies and keeps the ray as a dead stack (BAT-MAG-010) ...
    hurt(&mut s, &r, 2, 1000);
    effects::remove_dead(&mut s, &r);
    mark_corpses(&mut s);
    let rays = |s: &BattleState| s.effects.of(StackId(2)).len();
    assert_eq!((rays(&s), s.stacks()[2].defense_now()), (1, whole - 5));
    // ... but the stack that takes its record is set up from scratch (the original's 0x47A100).
    assert_eq!(sprites(&mut s), 2);
    let new = s.stacks()[2];
    assert_eq!((rays(&s), new.defense_now()), (0, whole));
    assert_eq!(new.spells, effects::SpellValues::new(new.speed));
    effects::recompute(&mut s, &r, StackId(2));
    assert_eq!(s.stacks()[2], new);
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
        (2, 0)
    );
    assert_eq!(outcome.stacks[0][1].origin, StackOrigin::Created);
}

// covers: BAT-FLE-002, BAT-MAG-037
#[test]
fn the_surrender_price_leaves_created_stacks_out() {
    // Batch 2d CL1: 800 before and after a clone of the 10 Archers (SU: 910 before and after a
    // summon); war machines still count (batch 1b R4). Ten `fixed` at 60: 300.
    let r = ruleset();
    let mut s = arena(
        &r,
        [Some(creator(&r, 1, 0)), Some(mage(&r, &[]))],
        &[("fixed", 10, 5, 5)],
        &[("serf", 1, 14, 5)],
        1,
    );
    assert_eq!(s.surrender_price(&r, BattleSide::Attacker), Some(300));
    let events = cast(&mut s, &r, "copy", stack(0)).unwrap();
    assert_eq!(created(&events).len(), 1);
    assert_eq!(s.surrender_price(&r, BattleSide::Attacker), Some(300));
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

/// The stacks struck, in order.
fn struck(events: &[Event]) -> Vec<u8> {
    events
        .iter()
        .filter_map(|e| match e.kind {
            EventKind::Struck { target, .. } | EventKind::SpellDamage { target, .. } => {
                Some(target.0)
            }
            _ => None,
        })
        .collect()
}

// covers: BAT-MAG-021, BAT-MAG-018, BAT-MAG-038
#[test]
fn a_clone_takes_no_anti_magic_reflects_one_hit_point_and_is_struck_no_more_once_gone() {
    let r = ruleset();
    let mut guard = mage(&r, &["antimagic", "copy"]);
    guard.magic.mana = 100;
    // A Hydra (head at (7, 5)) next to the defender's serfs at (8, 5) and their clone, top left of
    // them at (7, 4).
    let new = |r: &BattleRuleset| {
        let mut s = arena(
            r,
            [None, Some(guard.clone())],
            &[("hydra", 10, 7, 5)],
            &[("serf", 1, 8, 5)],
            1,
        );
        crate::created::clone(&mut s, r, (StackId(1), 3), &mut Vec::<Event>::new());
        assert_eq!(s.stacks()[2].hex, at(r, 7, 4));
        s
    };
    let mut s = new(&r);
    activate(&mut s, 1);
    assert_eq!(targets(&s, &r, BattleSide::Defender, "antimagic"), vec![1]);
    // The serfs die first and take their clone with them: the Hydra's other head strikes nothing.
    activate(&mut s, 0);
    let events = act(
        &mut s,
        &r,
        Command::Attack {
            target: StackId(1),
            from: at(&r, 7, 5),
        },
    )
    .unwrap();
    assert_eq!(
        (struck(&events), vanished(&events)),
        (vec![1], vec![(2, 1)])
    );
    // A clone with a Fire Shield reflects a share of one hit point: nothing.
    let mut s = new(&r);
    let shield = spell(&r, "fireshield");
    let cast = effects::TimedCast {
        id: shield,
        spell: r.spell(shield).unwrap(),
        mastery: 3,
        rounds: 5,
        own: crate::magic::SpellBonus::default(),
    };
    effects::apply(&mut s, &r, cast, StackId(2), &mut Vec::<Event>::new());
    activate(&mut s, 0);
    let events = act(
        &mut s,
        &r,
        Command::Attack {
            target: StackId(2),
            from: at(&r, 7, 5),
        },
    )
    .unwrap();
    assert_eq!(struck(&events), vec![2, 1]);
    assert!(
        !events
            .iter()
            .any(|e| matches!(e.kind, EventKind::Reflected { .. }))
    );
}

// covers: BAT-MAG-038
#[test]
fn a_battle_wide_spell_passes_over_a_clone_gone_with_its_original() {
    let r = ruleset();
    let mut hero = creator(&r, 1, 0);
    hero.magic.spell_book = Some(vec![spell(&r, "copy"), spell(&r, "storm")]);
    let mut s = arena(
        &r,
        [Some(hero), None],
        &[("serf", 1, 5, 5)],
        &[("serf", 20, 14, 5)],
        1,
    );
    cast(&mut s, &r, "copy", stack(0)).unwrap();
    let events = cast(&mut s, &r, "storm", CastTarget::None).unwrap();
    assert!(!struck(&events).contains(&2));
    assert_eq!(vanished(&events), vec![(2, 1)]);
}
