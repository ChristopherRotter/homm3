//! A stack's reach as a viewer may know it (E25 T38a, AI-Q19; `movement::reach_seen_by`, the AI
//! lane's observation): the rules' movement within the stack's speed now, its walk stopping only
//! at the pits both its controller and the viewer see (BAT-MAG-031, BAT-MAG-032).

use std::vec::Vec;

use h3_content_battle::{BattleRuleset, ObstacleKind};
use h3_contracts::{BattleSide, FortLevel, Moat, Siege};

use super::spells::{arena, arena_on, spell};
use super::support::*;
use crate::{
    BattleState, Command, Hex, HexSet, LegalCommands, SeenReach, StackId, legal_commands,
    reach_seen_by,
};

use BattleSide::{Attacker, Defender};

/// Places `side`'s obstacles of `kind` (the spell `key`'s shape) on `cells` as its cast would
/// (test setup only; the cast draws its hexes at random).
fn obstacle(
    s: &mut BattleState,
    r: &BattleRuleset,
    (key, kind): (&str, ObstacleKind),
    side: BattleSide,
    cells: &[(i32, i32)],
) {
    let id = spell(r, key);
    let shape = r.spell(id).unwrap().magic.obstacle.clone().unwrap();
    let hexes: Vec<Hex> = cells.iter().map(|&(x, y)| at(r, x, y)).collect();
    crate::obstacles::place(s, (id, kind, &shape), side, (&hexes, 40), &mut Vec::new());
}

const PIT: (&str, ObstacleKind) = ("quicksand", ObstacleKind::Pit);
const MINE: (&str, ObstacleKind) = ("landmine", ObstacleKind::Mine);

/// The hexes at `cells` as a set.
fn set(r: &BattleRuleset, cells: &[(i32, i32)]) -> HexSet {
    let mut out = HexSet::new();
    for &(x, y) in cells {
        out.insert(at(r, x, y));
    }
    out
}

/// The reach of stack `id` as `viewer` may know it.
fn seen(s: &BattleState, r: &BattleRuleset, id: u8, viewer: BattleSide) -> SeenReach {
    reach_seen_by(s, r, StackId(id), viewer)
}

/// Stack `id`'s legal moves on its turn, the position it stands on added, and the positions it
/// attacks from after a move (what [`SeenReach`]'s positions without its stops give).
fn legal(s: &BattleState, r: &BattleRuleset, id: u8) -> (HexSet, HexSet) {
    let mut t = s.clone();
    activate(&mut t, id);
    let mut out = LegalCommands::new();
    legal_commands(&t, r, t.side_to_act().unwrap(), &mut out);
    let (mut moves, mut attacks) = (HexSet::new(), HexSet::new());
    moves.insert(t.at(StackId(id)).hex);
    for c in out.iter() {
        match *c {
            Command::Move { to } => moves.insert(to),
            Command::Attack { from, .. } => attacks.insert(from),
            _ => {}
        }
    }
    (moves, attacks)
}

// covers: BAT-MAG-031, BAT-MAG-032, BAT-MOV-001
#[test]
fn a_pit_or_mine_the_viewer_does_not_see_changes_no_reach() {
    let r = ruleset();
    let base = arena(
        &r,
        [None, None],
        &[("fixed", 5, 2, 5)],
        &[("fixed", 5, 12, 5)],
        1,
    );
    // The defender's hidden pit on its stack's way and a mine: its own walk stops there.
    let mut s = base.clone();
    obstacle(&mut s, &r, PIT, Defender, &[(10, 5)]);
    obstacle(&mut s, &r, MINE, Defender, &[(11, 4), (4, 5)]);
    let own = seen(&s, &r, 1, Defender);
    assert_eq!(own.stops, set(&r, &[(10, 5)]));
    assert_ne!(own, seen(&base, &r, 1, Defender));
    assert!(!own.positions.contains(at(&r, 7, 5)));
    assert_eq!(own.positions, legal(&s, &r, 1).0);
    // The attacker sees neither: every stack's reach is the battle's without them, and the
    // attacker's own walk (which its controller plans without the pit) for the defender too.
    for id in 0..2 {
        assert_eq!(seen(&s, &r, id, Attacker), seen(&base, &r, id, Attacker));
    }
    assert_eq!(seen(&s, &r, 0, Defender), seen(&base, &r, 0, Defender));
    assert_eq!(seen(&s, &r, 0, Attacker).positions, legal(&s, &r, 0).0);
    // The attacker's worlds place them elsewhere: the same reach in each (`determinized`).
    for seed in [1, 2, 3] {
        let world = s.determinized(&r, Attacker, None, seed).unwrap();
        for id in 0..2 {
            assert_eq!(seen(&world, &r, id, Attacker), seen(&s, &r, id, Attacker));
        }
    }
}

// covers: BAT-MAG-031, BAT-MAG-032
#[test]
fn a_pit_both_sides_see_stops_the_walk_for_every_viewer() {
    let r = ruleset();
    let mut s = arena(
        &r,
        [None, None],
        &[("fixed", 5, 2, 5)],
        &[("fixed", 5, 9, 5)],
        1,
    );
    // The attacker's own pit stops its stack for itself; the defender, who does not see it,
    // knows its walk as the open field's.
    obstacle(&mut s, &r, PIT, Attacker, &[(5, 5)]);
    let plain = {
        let mut t = s.clone();
        t.spell_obstacles.retain(|_| false);
        t
    };
    let mine = seen(&s, &r, 0, Attacker);
    assert_eq!(mine.stops, set(&r, &[(5, 5)]));
    assert_eq!(mine.positions, legal(&s, &r, 0).0);
    assert_eq!(seen(&s, &r, 0, Defender), seen(&plain, &r, 0, Defender));
    // Its controller does not see it: the defender's stack walks over it in its plans.
    assert_eq!(seen(&s, &r, 1, Attacker), seen(&plain, &r, 1, Attacker));
    assert_eq!(seen(&s, &r, 1, Attacker).positions, legal(&s, &r, 1).0);
    // Revealed, it stops both stacks for both viewers, and no attack follows a move onto it.
    s.spell_obstacles.reveal_for_tests(at(&r, 5, 5));
    for (id, viewer) in [(0, Attacker), (0, Defender), (1, Attacker), (1, Defender)] {
        let reach = seen(&s, &r, id, viewer);
        assert_eq!(reach.stops, set(&r, &[(5, 5)]), "{id} {viewer:?}");
        let (moves, attacks) = legal(&s, &r, id);
        assert_eq!(reach.positions, moves);
        assert!(!attacks.contains(at(&r, 5, 5)));
    }
}

// covers: BAT-MOV-002, BAT-MAG-032
#[test]
fn a_flyer_reaches_every_free_position_within_its_speed_and_stops_nowhere() {
    let r = ruleset();
    let mut s = arena(
        &r,
        [None, None],
        &[("flyer", 5, 2, 5)],
        &[("fixed", 5, 4, 5)],
        1,
    );
    obstacle(&mut s, &r, PIT, Attacker, &[(3, 4), (3, 6)]);
    let reach = seen(&s, &r, 0, Attacker);
    assert!(reach.stops.is_empty());
    assert_eq!(reach.positions, legal(&s, &r, 0).0);
    let f = field(&r);
    let start = at(&r, 2, 5);
    for i in 0..f.hex_count() {
        let hex = Hex(u8::try_from(i).unwrap());
        let free = f.is_standable(hex) && hex != at(&r, 4, 5);
        let near = f.distance(start, hex) <= 7;
        assert_eq!(reach.positions.contains(hex), free && near, "{i}");
    }
}

// covers: BAT-GEO-002, BAT-MAG-032
#[test]
fn a_two_hex_walker_stops_where_its_footprint_enters_a_seen_pit() {
    let r = ruleset();
    let mut s = arena(
        &r,
        [None, None],
        &[("wide", 5, 5, 5)],
        &[("fixed", 5, 12, 5)],
        1,
    );
    // A position stops it when its footprint (the position and the hex in front of it) covers a
    // pit: (3, 5) and (4, 5) for the pit (4, 5), (7, 5) and (8, 5) for (8, 5).
    obstacle(&mut s, &r, PIT, Attacker, &[(4, 5), (8, 5)]);
    let reach = seen(&s, &r, 0, Attacker);
    assert_eq!(reach.stops, set(&r, &[(3, 5), (4, 5), (7, 5), (8, 5)]));
    assert_eq!(reach.positions, legal(&s, &r, 0).0);
    // The defender sees neither pit: the attacker's reach as it knows it is the open field's.
    let mut plain = s.clone();
    plain.spell_obstacles.retain(|_| false);
    assert_eq!(seen(&s, &r, 0, Defender), seen(&plain, &r, 0, Defender));
    assert!(seen(&s, &r, 0, Defender).stops.is_empty());
}

// covers: BAT-SIE-002, BAT-SIE-003
#[test]
fn the_closed_gate_passes_the_defenders_reach_only() {
    let r = ruleset();
    let fort = Siege {
        faction: r.faction_by_key("t:castle").unwrap(),
        fort: FortLevel::Fort,
        moat: Moat::Absent,
        structures: 0,
        escape_tunnel: false,
        bonuses: h3_contracts::TownBonuses::NONE,
    };
    // The defender's fast walker inside on (13, 5), the attacker's outside the walls, off the
    // bridges (on one it would keep the gate shut).
    let s = arena_on(
        &r,
        [None, None],
        &[("fast", 1, 8, 3)],
        &[("fast", 1, 13, 5)],
        1,
        |setup| setup.siege = Some(fort),
    );
    for viewer in [Attacker, Defender] {
        let inside = seen(&s, &r, 1, viewer);
        assert!(inside.positions.contains(at(&r, 7, 5)), "{viewer:?}");
        assert_eq!(inside.positions, legal(&s, &r, 1).0);
        let outside = seen(&s, &r, 0, viewer);
        assert!(!outside.positions.contains(at(&r, 11, 5)));
        assert!(!outside.positions.contains(at(&r, 13, 4)));
        assert_eq!(outside.positions, legal(&s, &r, 0).0);
    }
}

// covers: BAT-MAG-045
#[test]
fn a_bound_stack_reaches_only_where_it_stands() {
    let r = ruleset();
    let mut s = arena(
        &r,
        [None, None],
        &[("fixed", 5, 2, 5)],
        &[("fixed", 5, 12, 5)],
        1,
    );
    crate::binds::materialize(&mut s, StackId(1), true);
    let reach = seen(&s, &r, 1, Attacker);
    assert_eq!(reach.positions, set(&r, &[(12, 5)]));
    assert!(reach.stops.is_empty());
    assert_eq!(reach, seen(&s, &r, 1, Defender));
    assert_eq!(reach.positions, legal(&s, &r, 1).0);
}
