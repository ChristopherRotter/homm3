//! Tests of the dwellings' and the Refugee Camps' week (E08 T94b; increment 10's design note §6,
//! §9, §10): a dwelling full when the game is built, a written one standing until the week's
//! refill, no block and a block of full entries as one state, a fork's sharing, the invariants and
//! the hash; the camps' draws when the game is built and at each week's step, in one pass with the
//! weekly objects in object order; nothing of either in a view.

use std::collections::BTreeSet;
use std::vec::Vec;

use h3_content::{CreatureTypeId, ResourceId};
use h3_kernel::StreamId;

use super::dwellings::{
    Rules, build, build_with, mask, placed, rules, seeded, setup, type_of, views,
};
use super::*;

/// An object of type `k` on `(x, 1)`, one tile, blocked and entered.
fn one(rules: &Rules, k: &str, x: u16) -> ObjectSetup {
    placed(rules, k, (x, 1), mask(&[(0, 0)]), mask(&[(0, 0)]))
}

fn creature(rules: &Rules, k: &str) -> CreatureTypeId {
    rules.0.creature_by_key(k).unwrap()
}

fn stack(slot: u8, creature: CreatureTypeId, count: u32) -> ArmyStack {
    ArmyStack {
        slot,
        creature,
        count,
    }
}

/// The game's stream, as the week's step takes it.
fn stream(seed: u64) -> Rng {
    Rng::for_stream(seed, StreamId::GAME)
}

/// `t:d5`'s full guards: its creatures `t:c5` and `t:w5` (growth 2), 3 × 2 each, in slots 0 and 1.
fn full_guards(rules: &Rules) -> Vec<ArmyStack> {
    vec![
        stack(0, creature(rules, "t:c5"), 6),
        stack(1, creature(rules, "t:w5"), 6),
    ]
}

/// A Level-5 dwelling owned by red on (1, 1) and a level-1 one on (3, 1), neutral.
fn two_dwellings(rules: &Rules) -> Vec<ObjectSetup> {
    let mut owned = one(rules, "t:d5", 1);
    owned.owner = Some(PlayerId::new(0));
    vec![owned, one(rules, "t:d1", 3)]
}

// covers: ADV-DWL-002, ADV-RFG-001
#[test]
fn a_dwelling_starts_full_and_a_camp_with_a_creature_of_its_levels() {
    // ADV-DWL-002: each creature's count its growth, the guards 3 × growth of each creature of
    // level 5 or more, one stack each in creature order; a level-1 dwelling none. ADV-RFG-001: a
    // camp's creature among its levels' random monsters, its count the creature's growth.
    let rules = rules();
    let mut objects = two_dwellings(&rules);
    objects.extend([one(&rules, "t:camp", 5), one(&rules, "t:windmill", 7)]);
    let s = build(&rules, &setup(objects), 0).unwrap();
    assert!(s.dwellings.is_none());
    assert_eq!(s.dwelling_counts(0), Some(&[2, 2][..]));
    assert_eq!(s.dwelling_guards(0), full_guards(&rules));
    assert_eq!(s.dwelling_counts(1), Some(&[2, 2][..]));
    assert!(s.dwelling_guards(1).is_empty());
    for index in [2, 3, 99] {
        assert_eq!(s.dwelling_counts(index), None, "{index}");
        assert!(s.dwelling_guards(index).is_empty(), "{index}");
    }
    for index in [0, 3, 99] {
        assert_eq!(s.camp(index), None, "{index}");
    }
    let (drawn, count) = s.camp(2).unwrap();
    assert!(
        rules
            .1
            .random_monsters(&rules.0, [1, 3])
            .any(|c| c == drawn)
    );
    assert_eq!(count, u32::from(rules.0.creature(drawn).unwrap().growth));
    assert!(s.check_dwellings() && s.check_camps(&rules.0, &rules.1));
    // A map without dwellings and camps has neither block.
    let bare = build(&rules, &setup(vec![one(&rules, "t:windmill", 7)]), 0).unwrap();
    assert!(bare.dwellings.is_none() && bare.camps.is_none());
    assert!(bare.check_dwellings() && bare.check_camps(&rules.0, &rules.1));
}

// covers: ADV-DWL-003
#[test]
fn a_written_dwelling_stands_until_the_week_refills_every_dwelling() {
    // ADV-DWL-003: the week sets every dwelling's counts to the growth and its guards back, owned
    // or not, with no draw and no event: the block is none again.
    let rules = rules();
    let built = build(&rules, &setup(two_dwellings(&rules)), 0).unwrap();
    let w5 = creature(&rules, "t:w5");
    let mut s = built.clone();
    s.set_dwelling(0, &[1, 2], &[stack(1, w5, 4)]);
    s.set_dwelling(1, &[0, 2], &[]);
    assert_eq!(s.dwelling_counts(0), Some(&[1, 2][..]));
    assert_eq!(s.dwelling_guards(0), vec![stack(1, w5, 4)]);
    assert_eq!(s.dwelling_counts(1), Some(&[0, 2][..]));
    assert!(s.check_dwellings());
    assert_ne!(s, built);
    assert_ne!(s.state_hash(), built.state_hash());
    let mut beaten = built.clone();
    beaten.set_dwelling(0, &[2, 2], &[]);
    assert!(beaten.dwelling_guards(0).is_empty() && beaten.check_dwellings());
    for mut state in [s, beaten] {
        let mut draws = stream(3);
        let mut events = Vec::new();
        state.start_week(&rules.0, &rules.1, &mut draws, &mut events);
        assert!(events.is_empty());
        assert_eq!(draws.next_u64(), stream(3).next_u64());
        assert!(state.dwellings.is_none());
        assert_eq!(state.dwelling_counts(0), Some(&[2, 2][..]));
        assert_eq!(state.dwelling_guards(0), full_guards(&rules));
        assert_eq!(state.dwelling_counts(1), Some(&[2, 2][..]));
        assert_eq!(state, built);
        assert_eq!(state.state_hash(), built.state_hash());
    }
}

// covers: ADV-DWL-002
#[test]
fn no_block_and_a_block_of_full_entries_are_one_state() {
    // ADR-0016 point 2: a write that leaves every dwelling as its week began makes no block, so
    // equality, which reads the representation, sees the content; the hash reads each dwelling as
    // it stands, so a block of full entries hashes as none, and the invariant refuses one.
    let rules = rules();
    let built = build(&rules, &setup(two_dwellings(&rules)), 0).unwrap();
    let full = full_guards(&rules);
    let mut same = built.clone();
    same.set_dwelling(0, &[2, 2], &full);
    assert!(same.dwellings.is_none());
    assert_eq!(same, built);
    let mut back = built.clone();
    back.set_dwelling(0, &[1, 2], &full);
    assert!(back.dwellings.is_some());
    back.set_dwelling(0, &[2, 2], &full);
    assert!(back.dwellings.is_none());
    assert_eq!(back, built);
    assert_eq!(back.state_hash(), built.state_hash());
    let mut raw = built.clone();
    raw.set_full_dwellings_block();
    assert!(raw.dwellings.is_some());
    assert_eq!(raw.state_hash(), built.state_hash());
    assert!(!raw.check_dwellings());
    // Each count and each guard as they stand enter the hash (v20).
    let (c5, w5) = (creature(&rules, "t:c5"), creature(&rules, "t:w5"));
    let written: [(usize, &[u32], Vec<ArmyStack>); 6] = [
        (0, &[1, 2], full.clone()),
        (0, &[2, 1], full.clone()),
        (0, &[2, 2], vec![stack(0, c5, 6)]),
        (0, &[2, 2], vec![stack(0, c5, 5), stack(1, w5, 6)]),
        (0, &[2, 2], vec![]),
        (1, &[2, 1], vec![]),
    ];
    let mut hashes = BTreeSet::from([built.state_hash()]);
    for (index, counts, guards) in &written {
        let mut t = built.clone();
        t.set_dwelling(*index, counts, guards);
        assert!(t.check_dwellings(), "{counts:?} {guards:?}");
        hashes.insert(t.state_hash());
    }
    assert_eq!(hashes.len(), written.len() + 1);
}

// covers: ADV-DWL-002, ADV-RFG-001
#[test]
fn a_fork_shares_the_dwellings_and_the_camps_blocks() {
    let rules = rules();
    let mut objects = two_dwellings(&rules);
    objects.push(one(&rules, "t:camp", 5));
    let built = build(&rules, &setup(objects), 0).unwrap();
    let full = full_guards(&rules);
    let mut s = built.clone();
    s.set_dwelling(0, &[1, 2], &full);
    let fork = s.clone();
    assert!(Arc::ptr_eq(
        s.dwellings.as_ref().unwrap(),
        fork.dwellings.as_ref().unwrap()
    ));
    assert!(Arc::ptr_eq(
        s.camps.as_ref().unwrap(),
        fork.camps.as_ref().unwrap()
    ));
    // A later write copies the block: the fork keeps its week.
    s.set_dwelling(0, &[0, 2], &full);
    assert_eq!(fork.dwelling_counts(0), Some(&[1, 2][..]));
    assert_eq!(s.dwelling_counts(0), Some(&[0, 2][..]));
    assert_eq!(built.dwelling_counts(0), Some(&[2, 2][..]));
}

// covers: ADV-DWL-002, ADV-RFG-001
#[test]
fn the_dwellings_and_the_camps_invariants() {
    let rules = rules();
    let mut objects = two_dwellings(&rules);
    objects.push(one(&rules, "t:camp", 5));
    let built = build(&rules, &setup(objects), 0).unwrap();
    let full = full_guards(&rules);
    let (c1, c5, w5) = (
        creature(&rules, "t:c1"),
        creature(&rules, "t:c5"),
        creature(&rules, "t:w5"),
    );
    let refused: [(usize, &[u32], Vec<ArmyStack>); 6] = [
        (0, &[3, 2], full.clone()),
        (0, &[2], full.clone()),
        (0, &[2, 2], vec![stack(0, c5, 7)]),
        (0, &[2, 2], vec![stack(0, w5, 6)]),
        (0, &[2, 2], vec![stack(0, c5, 0)]),
        (1, &[2, 2], vec![stack(0, c1, 1)]),
    ];
    for (index, counts, guards) in &refused {
        let mut t = built.clone();
        t.set_dwelling(*index, counts, guards);
        assert!(!t.check_dwellings(), "{counts:?} {guards:?}");
    }
    // A stack in a slot past a dwelling's four is left out (a dwelling's guards never hold one).
    let mut past = built.clone();
    past.set_dwelling(0, &[2, 2], &[stack(0, c5, 6), stack(4, c5, 1)]);
    assert_eq!(past.dwelling_guards(0), vec![stack(0, c5, 6)]);
    assert!(past.check_dwellings());
    let mut short = built.clone();
    short.dwellings = Some(Arc::from(&built.map.full_weeks[..1]));
    assert!(!short.check_dwellings());
    // The camp: its creature among its levels' random monsters, its count at most the growth.
    let (drawn, count) = built.camp(2).unwrap();
    assert!(count > 0);
    let mut t = built.clone();
    t.set_camp(2, drawn, count - 1);
    assert!(t.check_camps(&rules.0, &rules.1));
    assert_ne!(t.state_hash(), built.state_hash());
    let other = rules
        .1
        .random_monsters(&rules.0, [1, 3])
        .find(|&c| c != drawn && rules.0.creature(c).unwrap().growth == 2)
        .unwrap();
    let mut t = built.clone();
    t.set_camp(2, other, 2);
    assert!(t.check_camps(&rules.0, &rules.1));
    assert_ne!(t.state_hash(), built.state_hash());
    for (creature, count) in [(creature(&rules, "t:c7"), 1), (drawn, count + 1)] {
        let mut t = built.clone();
        t.set_camp(2, creature, count);
        assert!(!t.check_camps(&rules.0, &rules.1), "{creature:?} {count}");
    }
    let mut none = built.clone();
    none.camps = None;
    assert!(!none.check_camps(&rules.0, &rules.1));
    let mut empty = built.clone();
    empty.camps = Some(Arc::from(Vec::new()));
    assert!(!empty.check_camps(&rules.0, &rules.1));
}

/// The draws of a Windmill (its resource among `resources`, then its amount within `amount`), a
/// camp (its creature among `pool`) and a second Windmill, in that order, apart from the code.
fn windmill_camp_windmill(
    draws: &mut Rng,
    (resources, amount): (&[ResourceId], [u32; 2]),
    pool: &[CreatureTypeId],
) -> [(Option<ResourceId>, Option<CreatureTypeId>, u32); 3] {
    let windmill = |draws: &mut Rng| {
        let n = u32::try_from(resources.len()).unwrap();
        let resource = resources[usize::try_from(draws.below(n)).unwrap()];
        (
            Some(resource),
            None,
            draws.range_inclusive(amount[0], amount[1]),
        )
    };
    let first = windmill(draws);
    let n = u32::try_from(pool.len()).unwrap();
    let camp = pool[usize::try_from(draws.below(n)).unwrap()];
    [first, (None, Some(camp), 0), windmill(draws)]
}

/// The Windmill on 0, the camp on 1 and the Windmill on 2 of `s` as
/// [`windmill_camp_windmill`]'s entries (the camp's count apart).
fn as_drawn(s: &AdventureState) -> [(Option<ResourceId>, Option<CreatureTypeId>, u32); 3] {
    let windmill = |index| {
        let weekly = s.weekly(index).unwrap();
        (Some(weekly.resource), None, weekly.amount)
    };
    [windmill(0), (None, s.camp(1).map(|c| c.0), 0), windmill(2)]
}

// covers: ADV-RFG-001, ADV-WEK-001
#[test]
fn the_camps_draw_with_the_weekly_objects_in_object_order() {
    // Increment 10's design note §9: when the game is built with the weekly objects' first week,
    // and at each week's step with their refill, in one pass in object order (Windmill, camp,
    // Windmill), so a camp's draw falls between the Windmills'; deterministic per seed.
    let rules = rules();
    let objects = vec![
        one(&rules, "t:windmill", 1),
        one(&rules, "t:camp", 3),
        one(&rules, "t:windmill", 5),
    ];
    let map = setup(objects);
    let Some(ObjectKind::Windmill { resources, amount }) = rules
        .1
        .object_type(type_of(&rules, "t:windmill"))
        .map(|ty| &ty.kind)
    else {
        panic!("t:windmill is no Windmill");
    };
    let windmill = (&resources[..], *amount);
    let pool: Vec<CreatureTypeId> = rules.1.random_monsters(&rules.0, [1, 3]).collect();
    let growth = |c: CreatureTypeId| u32::from(rules.0.creature(c).unwrap().growth);
    let mut camps = BTreeSet::new();
    for seed in 0..6 {
        let mut draws = seeded(seed);
        let mut replay = draws.clone();
        let expected = windmill_camp_windmill(&mut replay, windmill, &pool);
        let mut s = build_with(&rules, &map, &mut draws).unwrap();
        assert_eq!(as_drawn(&s), expected, "{seed}");
        assert_eq!(draws.next_u64(), replay.next_u64(), "{seed}");
        assert_eq!(build(&rules, &map, seed).unwrap(), s, "{seed}");
        let mut draws = stream(seed);
        for week in 0..3 {
            let mut replay = draws.clone();
            let expected = windmill_camp_windmill(&mut replay, windmill, &pool);
            let mut events = Vec::new();
            s.start_week(&rules.0, &rules.1, &mut draws, &mut events);
            assert!(events.is_empty());
            assert_eq!(as_drawn(&s), expected, "{seed} {week}");
            assert_eq!(draws.clone().next_u64(), replay.next_u64(), "{seed} {week}");
            let (creature, count) = s.camp(1).unwrap();
            assert_eq!(count, growth(creature));
            assert!(s.check_camps(&rules.0, &rules.1) && s.check_weekly(&rules.1));
            camps.insert(creature);
        }
    }
    // The draws differ between weeks and seeds.
    assert!(camps.len() > 1);
    // The walk asks only the weekly objects for contents: a camp's kind has none.
    let camp = rules
        .1
        .object_type(type_of(&rules, "t:camp"))
        .map(|ty| &ty.kind);
    assert_eq!(
        crate::state::economy::contents(camp, true, &mut seeded(0)),
        None
    );
    // A camp without weekly objects draws alone each week.
    let only = setup(vec![one(&rules, "t:camp", 3)]);
    let mut s = build(&rules, &only, 1).unwrap();
    let mut draws = stream(1);
    let mut replay = draws.clone();
    let n = u32::try_from(pool.len()).unwrap();
    let expected = pool[usize::try_from(replay.below(n)).unwrap()];
    s.start_week(&rules.0, &rules.1, &mut draws, &mut Vec::new());
    assert_eq!(s.camp(0), Some((expected, growth(expected))));
    assert!(s.weekly.is_none());
    assert_eq!(draws.next_u64(), replay.next_u64());
}

// covers: ADV-OWN-003
#[test]
fn a_view_shows_no_dwelling_s_counts_or_guards_and_no_camp_s_creature() {
    // Increment 10's design note §10: a viewer's view is the same whatever a dwelling's counts and
    // guards and whatever a camp drew; `guards` stays none for both.
    let rules = rules();
    let (red, blue) = (PlayerId::new(0), PlayerId::new(1));
    let mut objects = two_dwellings(&rules);
    objects.push(one(&rules, "t:camp", 5));
    let setup = setup(objects);
    let built = |seed| {
        let mut s = build(&rules, &setup, seed).unwrap();
        s.explore_all();
        s
    };
    let s = built(0);
    let seen = |s: &AdventureState| [views(s, &rules, red), views(s, &rules, blue)];
    let before = seen(&s);
    assert!(
        before
            .iter()
            .all(|v| v.len() == 3 && v.iter().all(|o| o.guards.is_none()))
    );
    let mut written = s.clone();
    written.set_dwelling(0, &[0, 1], &[]);
    assert_eq!(seen(&written), before);
    let other = (1..64).map(built).find(|t| t.camp(2) != s.camp(2)).unwrap();
    assert_ne!(other, s);
    assert_eq!(seen(&other), before);
}
