//! The town's other rules (E04c T8; `docs/formats/installation.md` §7.32): the controls at a
//! town (BAT-SIE-024), the town's morale and luck (BAT-SIE-025), Teleport across the moat and the
//! walls (BAT-SIE-026), no spell obstacle on a wall (BAT-SIE-027), a created stack through the
//! closed gate (BAT-SIE-028) and the tactics phase at a town (BAT-TAC-001), on the test ruleset's
//! siege field (the original's hexes; `sieges.rs`' header has the rows' wall columns).

use std::vec;
use std::vec::Vec;

use h3_content_battle::BattleRuleset;
use h3_contracts::{
    BattleHero, BattleResult, BattleSetup, BattleSide, FortLevel, Moat, Obstacle, Siege,
};

use super::spells::{arena_on, mage, spell, stack};
use super::support::*;
use crate::magic::{CastTargets, cast_targets, second_targets};
use crate::siege::damage_part;
use crate::{
    BattleState, CastTarget, Command, CommandError, Event, EventKind, Hex, StackId, observe,
};

use BattleSide::{Attacker, Defender};

/// The town of `faction` with `fort` and `moat`, nothing else.
fn town(r: &BattleRuleset, faction: &str, fort: FortLevel, moat: Moat) -> Siege {
    Siege {
        faction: r.faction_by_key(faction).unwrap(),
        fort,
        moat,
        structures: 0,
        escape_tunnel: false,
        morale: 0,
        luck: 0,
    }
}

/// A battle at `town` with the stacks placed at `(x, y)`, as `arena_on`; the first attacker
/// stack is active.
fn at_town(
    r: &BattleRuleset,
    town: Option<Siege>,
    heroes: [Option<BattleHero>; 2],
    attacker: &[(&'static str, u32, i32, i32)],
    defender: &[(&'static str, u32, i32, i32)],
    edit: impl FnOnce(&mut BattleSetup),
) -> BattleState {
    arena_on(r, heroes, attacker, defender, 1, |s| {
        s.siege = town;
        s.funds = [1_000_000; 2];
        edit(s);
    })
}

/// A hero of the test ruleset with `spells` at the air mastery `air`, mana to spare.
fn caster(r: &BattleRuleset, spells: &[&str], air: u8) -> BattleHero {
    let mut h = mage(r, spells);
    h.magic.masteries = masteries(r, [air, 0, 0, 0]);
    h.magic.mana = 100;
    h
}

// ---------------------------------------------------------------- BAT-SIE-024

// covers: BAT-SIE-024, BAT-FLE-001, BAT-FLE-002
#[test]
fn a_town_s_defender_never_surrenders_and_retreats_only_through_the_escape_tunnel() {
    let r = ruleset();
    let heroes = [Some(hero(&r)), Some(hero(&r))];
    let start = |siege: Option<Siege>| {
        at_town(
            &r,
            siege,
            heroes.clone(),
            &[("fixed", 5, 2, 5)],
            &[("fixed", 5, 14, 5)],
            |_| {},
        )
    };
    let controls = |s: &BattleState| {
        BattleSide::BOTH.map(|side| (s.may_retreat(side), s.surrender_possible(&r, side)))
    };
    // Away from a town both sides may leave either way.
    assert_eq!(controls(&start(None)), [(true, true); 2]);
    // At a town, with a fort or without one, the attacker keeps both and the defender neither.
    let castle = town(&r, "t:castle", FortLevel::None, Moat::Absent);
    for fort in [FortLevel::None, FortLevel::Fort, FortLevel::Citadel] {
        let s = start(Some(Siege { fort, ..castle }));
        assert_eq!(controls(&s), [(true, true), (false, false)], "{fort:?}");
        for viewer in BattleSide::BOTH {
            let o = observe(&s, &r, viewer);
            assert_eq!(
                (o.may_retreat, o.surrender_possible),
                ([true, false], [true, false])
            );
        }
        let mut t = s.clone();
        let d = defender(&t);
        activate(&mut t, d);
        let before = t.clone();
        assert_eq!(
            act(&mut t, &r, Command::Retreat),
            Err(CommandError::CannotRetreat)
        );
        assert_eq!(
            act(&mut t, &r, Command::Surrender),
            Err(CommandError::CannotSurrender)
        );
        assert_eq!(t, before);
        assert!(!legal(&t, &r, Defender).iter().any(|c| c.leaves_battle()));
        // The attacker may surrender at a town.
        let mut t = s.clone();
        assert!(act(&mut t, &r, Command::Surrender).is_ok());
        assert_eq!(t.result(), Some(BattleResult::Surrender(Attacker)));
    }
    // The Escape Tunnel lets the defender retreat, but not surrender.
    let tunnel = Siege {
        escape_tunnel: true,
        ..castle
    };
    let s = start(Some(tunnel));
    assert_eq!(controls(&s), [(true, true), (true, false)]);
    let mut t = s.clone();
    let d = defender(&t);
    activate(&mut t, d);
    assert_eq!(
        act(&mut t, &r, Command::Surrender),
        Err(CommandError::CannotSurrender)
    );
    assert!(act(&mut t, &r, Command::Retreat).is_ok());
    assert_eq!(t.result(), Some(BattleResult::Retreat(Defender)));
    // The tunnel is part of the state.
    assert_ne!(start(Some(castle)).state_hash(), s.state_hash());
}

/// The id of the defender's first stack (after the attacker's army and its Catapult at a fort).
fn defender(s: &BattleState) -> u8 {
    let i = s
        .stacks()
        .iter()
        .position(|st| st.side == Defender)
        .unwrap();
    u8::try_from(i).unwrap()
}

/// The legal commands of `side` now.
fn legal(s: &BattleState, r: &BattleRuleset, side: BattleSide) -> Vec<Command> {
    let mut out = crate::LegalCommands::new();
    crate::legal_commands(s, r, side, &mut out);
    out.as_slice().to_vec()
}

// ---------------------------------------------------------------- BAT-SIE-025

// covers: BAT-SIE-025
#[test]
fn a_town_raises_its_defender_s_morale_and_luck() {
    let r = ruleset();
    let start = |morale: i8, luck: i8| {
        let castle = Siege {
            morale,
            luck,
            ..town(&r, "t:castle", FortLevel::Fort, Moat::Absent)
        };
        at_town(
            &r,
            Some(castle),
            [None, None],
            &[("fixed", 5, 2, 5)],
            &[("fixed", 5, 14, 5), ("skeleton", 5, 14, 7)],
            |_| {},
        )
    };
    let values = |s: &BattleState| {
        s.stacks()
            .iter()
            .map(|st| (st.morale, st.luck))
            .collect::<Vec<_>>()
    };
    // Without the town's terms: one alignment +1 for the attacker; the defender's two
    // alignments and its undead (BAT-MOR-002), the skeleton without morale.
    let plain = values(&start(0, 0));
    assert_eq!(plain, [(1, 0), (-1, 0), (0, 0)]);
    // The Tavern (+1) and the Brotherhood of the Sword (+2), the Fountain of Fortune (+2): the
    // defender's stacks only, the undead still without morale, clamped as any morale and luck.
    assert_eq!(values(&start(1, 0)), [(1, 0), (0, 0), (0, 0)]);
    assert_eq!(values(&start(3, 2)), [(1, 0), (2, 2), (0, 2)]);
    assert_eq!(values(&start(9, 9)), [(1, 0), (3, 3), (0, 3)]);
    // The setup's terms are the battle's from its start.
    assert_ne!(start(1, 0).state_hash(), start(0, 0).state_hash());
    // Batch 3g's CM and RL: three alignments (−1) and the Brotherhood of the Sword (+2, D-21)
    // give +1; the Fountain of Fortune +2.
    let mixed = at_town(
        &r,
        Some(Siege {
            morale: 2,
            luck: 2,
            ..town(&r, "t:castle", FortLevel::Fort, Moat::Absent)
        }),
        [None, None],
        &[("fixed", 5, 2, 5)],
        &[
            ("pikeman_c", 5, 14, 3),
            ("imp", 5, 14, 5),
            ("serf", 5, 14, 7),
        ],
        |_| {},
    );
    assert_eq!(values(&mixed), [(1, 0), (1, 2), (1, 2), (1, 2)]);
}

// ---------------------------------------------------------------- BAT-SIE-026

/// The destinations Teleport at `air` offers for the attacker's stack 0 at `(2, 5)` at `siege`,
/// after `edit`; the hexes of `(x, y)` among them.
fn teleport_reaches(
    r: &BattleRuleset,
    siege: Siege,
    air: u8,
    edit: impl FnOnce(&mut BattleState),
    cells: &[(i32, i32)],
) -> Vec<bool> {
    teleport_offers(r, siege, (Attacker, (2, 5)), air, edit, cells)
}

/// The destinations Teleport at `air` offers `side`'s hero, at its stack's turn, for that stack at
/// `from` (the other side's alone at `(2, 5)` or `(14, 9)`, without a hero) at `siege`, after
/// `edit`; the hexes of `(x, y)` among them.
fn teleport_offers(
    r: &BattleRuleset,
    siege: Siege,
    (side, (x, y)): (BattleSide, (i32, i32)),
    air: u8,
    edit: impl FnOnce(&mut BattleState),
    cells: &[(i32, i32)],
) -> Vec<bool> {
    let mut heroes = [None, None];
    heroes[side.index()] = Some(caster(r, &["teleport"], air));
    let mut armies = [[("fixed", 5, 2, 5)], [("fixed", 5, 14, 9)]];
    armies[side.index()] = [("fixed", 5, x, y)];
    let mut s = at_town(r, Some(siege), heroes, &armies[0], &armies[1], |_| {});
    let mover = if side == Attacker { 0 } else { defender(&s) };
    activate(&mut s, mover);
    edit(&mut s);
    let mut out = CastTargets::new();
    let teleport = (spell(r, "teleport"), stack(mover));
    second_targets(&s, r, side, teleport, &mut out);
    cells
        .iter()
        .map(|&(x, y)| out.as_slice().contains(&CastTarget::Hex(at(r, x, y))))
        .collect()
}

// covers: BAT-SIE-026, BAT-MAG-029
#[test]
fn teleport_crosses_the_moat_from_advanced_and_the_walls_at_expert() {
    let r = ruleset();
    let citadel = town(&r, "t:castle", FortLevel::Citadel, Moat::Damaging);
    // Outside (8, 5), the moat hex of row 4 (9, 4), inside (13, 5), (14, 2).
    let cells = [(8, 5), (9, 4), (13, 5), (14, 2)];
    let reach = |siege, air| teleport_reaches(&r, siege, air, |_| {}, &cells);
    assert_eq!(reach(citadel, 1), [true, false, false, false]);
    assert_eq!(reach(citadel, 2), [true, true, false, false]);
    assert_eq!(reach(citadel, 3), [true, true, true, true]);
    // Without a moat (a Fort) the moat hex is an outside hex: Basic reaches it, not inside.
    let fort = Siege {
        fort: FortLevel::Fort,
        moat: Moat::Absent,
        ..citadel
    };
    assert_eq!(reach(fort, 1), [true, true, false, false]);
    // Without a fort nothing stands in the way: every mastery reaches inside.
    let open = Siege {
        fort: FortLevel::None,
        moat: Moat::Absent,
        ..citadel
    };
    assert_eq!(reach(open, 1), [true, true, true, true]);
    // The Fortress's outer moat row counts as outside: Basic reaches it, not the inner row.
    let fortress = Siege {
        faction: r.faction_by_key("t:inferno").unwrap(),
        ..citadel
    };
    assert_eq!(
        teleport_reaches(&r, fortress, 1, |_| {}, &[(8, 4), (9, 4)]),
        [true, false]
    );
}

// covers: BAT-SIE-026
#[test]
fn below_expert_teleport_follows_the_shot_s_line_through_the_walls() {
    let r = ruleset();
    let fort = town(&r, "t:castle", FortLevel::Fort, Moat::Absent);
    // Along row 5 through the gate's hex (11, 5) to (13, 5); to (12, 4) behind the wall at
    // (10, 4).
    let cells = [(13, 5), (12, 4)];
    let none = |_: &mut BattleState| {};
    assert_eq!(teleport_reaches(&r, fort, 2, none, &cells), [false, false]);
    // The open gate lets the line through, not the wall beside it.
    let open = |s: &mut BattleState| {
        let mut siege = s.siege.unwrap();
        siege.gate_open = true;
        s.siege = Some(siege);
    };
    assert_eq!(teleport_reaches(&r, fort, 2, open, &cells), [true, false]);
    // A destroyed wall (target 2, hex 78 at (10, 4)) lets the line through on its row.
    let broken = |s: &mut BattleState| damage_part(s, 2, 9, &mut Vec::<Event>::new());
    assert_eq!(teleport_reaches(&r, fort, 2, broken, &cells), [false, true]);
    // A cast at a refused hex is refused before the mana is spent.
    let mut s = at_town(
        &r,
        Some(fort),
        [Some(caster(&r, &["teleport"], 1)), None],
        &[("fixed", 5, 2, 5)],
        &[("fixed", 5, 14, 9)],
        |_| {},
    );
    let before = s.clone();
    let go = |x, y| Command::Cast {
        spell: spell(&r, "teleport"),
        target: stack(0),
        second: CastTarget::Hex(at(&r, x, y)),
    };
    assert_eq!(
        act(&mut s, &r, go(13, 5)),
        Err(CommandError::InvalidSpellTarget)
    );
    assert_eq!(s, before);
    let events = act(&mut s, &r, go(8, 5)).unwrap();
    assert!(
        events
            .iter()
            .any(|e| matches!(e.kind, EventKind::Teleported { .. }))
    );
}

// covers: BAT-SIE-026
#[test]
fn below_expert_teleport_tests_the_line_inside_the_walls_too() {
    let r = ruleset();
    let fort = town(&r, "t:castle", FortLevel::Fort, Moat::Absent);
    // The defender's stack inside, from (11, 4) (hex 79) to (11, 6) (hex 113): the line passes
    // the closed gate's hex (11, 5) (96), though the move crosses no wall (the original tests
    // every move, 0x5A3760 to 0x5A37AF; VCMI only a crossing, D-20).
    let inside = (Defender, (11, 4));
    let reach =
        |air, edit: fn(&mut BattleState)| teleport_offers(&r, fort, inside, air, edit, &[(11, 6)]);
    assert_eq!(reach(2, |_| {}), [false]);
    assert_eq!(reach(3, |_| {}), [true]);
    // The open gate lets the line through.
    let open = |s: &mut BattleState| {
        let mut siege = s.siege.unwrap();
        siege.gate_open = true;
        s.siege = Some(siege);
    };
    assert_eq!(reach(2, open), [true]);
}

// covers: BAT-SIE-026, BAT-SIE-005
#[test]
fn teleport_follows_batch_3g_and_lands_in_the_moat_without_its_damage() {
    let r = ruleset();
    let citadel = town(&r, "t:castle", FortLevel::Citadel, Moat::Damaging);
    // Batch 3g's rows TB, TV, TE: from hex 86 (1, 5) to the inside hex 64 (13, 3), the moat hex
    // 61 (10, 3) and the outside hex 91 (6, 5).
    let cells = [(13, 3), (10, 3), (6, 5)];
    let reach = |air| teleport_offers(&r, citadel, (Attacker, (1, 5)), air, |_| {}, &cells);
    assert_eq!(reach(1), [false, false, true]);
    assert_eq!(reach(2), [false, true, true]);
    assert_eq!(reach(3), [true, true, true]);
    // TV: Advanced Teleport lands on the damaging moat's hex without its damage.
    let mut s = at_town(
        &r,
        Some(citadel),
        [Some(caster(&r, &["teleport"], 2)), None],
        &[("fixed", 5, 1, 5)],
        &[("fixed", 5, 14, 9)],
        |_| {},
    );
    let before = s.stacks()[0];
    let moat = at(&r, 10, 3);
    let go = Command::Cast {
        spell: spell(&r, "teleport"),
        target: stack(0),
        second: CastTarget::Hex(moat),
    };
    let events = act(&mut s, &r, go).unwrap();
    let after = s.stacks()[0];
    assert_eq!(after.hex, moat);
    assert_eq!((after.count, after.top_hp), (before.count, before.top_hp));
    assert!(
        !events
            .iter()
            .any(|e| matches!(e.kind, EventKind::MoatDamage { .. }))
    );
}

// ---------------------------------------------------------------- BAT-SIE-027

// covers: BAT-SIE-027, BAT-MAG-027
#[test]
fn no_spell_obstacle_goes_on_a_standing_wall_or_the_gate() {
    let r = ruleset();
    let fort = town(&r, "t:castle", FortLevel::Fort, Moat::Absent);
    let start = || {
        at_town(
            &r,
            Some(fort),
            [Some(caster(&r, &["wall", "quicksand"], 3)), None],
            &[("fixed", 5, 2, 5)],
            &[("fixed", 5, 14, 9)],
            |_| {},
        )
    };
    let force_field = |s: &BattleState, x, y| {
        let mut out = CastTargets::new();
        cast_targets(s, &r, Attacker, spell(&r, "wall"), &mut out);
        out.as_slice().contains(&CastTarget::Hex(at(&r, x, y)))
    };
    let s = start();
    // Its shape at Expert: the target hex, its upper right neighbour and that one's upper left.
    // From (10, 5) the drawbridge's hex: the wall hex (10, 4), then (10, 3); from (6, 5) clear of
    // the walls.
    assert!(force_field(&s, 6, 5));
    assert!(!force_field(&s, 10, 5));
    assert!(!force_field(&s, 11, 5));
    // A destroyed wall's hex takes it; the gate's never, destroyed or not.
    let mut t = s.clone();
    damage_part(&mut t, 2, 9, &mut Vec::<Event>::new());
    assert!(force_field(&t, 10, 5));
    let mut t = s.clone();
    damage_part(&mut t, 3, 9, &mut Vec::<Event>::new());
    assert!(t.siege.unwrap().gate_destroyed());
    assert!(!force_field(&t, 11, 5));
    assert!(crate::siege::walled(&t).contains(at(&r, 11, 5)));
    // Quicksand's draws never land on a wall: Expert's eight pits over many seeds.
    let walls = crate::siege::walled(&s);
    for seed in 0..64u64 {
        let mut t = s.clone();
        t.rng = h3_kernel::Rng::for_stream(seed, h3_kernel::StreamId::BATTLE);
        act(
            &mut t,
            &r,
            Command::Cast {
                spell: spell(&r, "quicksand"),
                target: CastTarget::None,
                second: CastTarget::None,
            },
        )
        .unwrap();
        assert!(
            !t.spell_obstacles()
                .iter()
                .any(|o| o.hexes.intersects(walls))
        );
    }
}

// ---------------------------------------------------------------- BAT-SIE-028

/// Obstacles on every standable hex of the columns `from..=to` but those in `keep` and the
/// walls (a siege's natural obstacles keep off them).
fn blockers(r: &BattleRuleset, (from, to): (i32, i32), keep: &[(i32, i32)]) -> Vec<Obstacle> {
    let walls = [12, 12, 11, 11, 10, 11, 10, 11, 11, 12, 12];
    (0..11)
        .flat_map(|y| (from..=to).map(move |x| (x, y)))
        .filter(|&(x, y)| x != walls[usize::try_from(y).unwrap()] && !keep.contains(&(x, y)))
        .map(|(x, y)| Obstacle {
            hexes: vec![at(r, x, y).0],
            anchor: None,
            kind: None,
        })
        .collect()
}

// covers: BAT-SIE-028, BAT-MAG-039
#[test]
fn the_defender_s_summon_passes_the_closed_gate_and_opens_it() {
    let r = ruleset();
    let fort = town(&r, "t:castle", FortLevel::Fort, Moat::Absent);
    // Columns 12 to 15 full and column 11 too but for the gate's hex (11, 5): the defender's
    // elemental can only stand on the gate's hex.
    // The defender's own stack stands on (14, 9).
    let obstacles = blockers(&r, (11, 15), &[(11, 5), (14, 9)]);
    let summoner = {
        let mut h = mage(&r, &["sprites"]);
        h.magic.masteries = masteries(&r, [3, 3, 3, 3]);
        h.magic.mana = 100;
        h
    };
    let mut s = at_town(
        &r,
        Some(fort),
        [None, Some(summoner)],
        &[("fixed", 5, 2, 5)],
        &[("fixed", 5, 14, 9)],
        |setup| setup.obstacles = obstacles,
    );
    // The closed gate lets the defender's placements through, not the attacker's.
    let field = field(&r);
    assert!(
        !s.blocked_as(field, StackId(u8::MAX), Some(Defender))
            .contains(Hex(96))
    );
    assert!(
        s.blocked_as(field, StackId(u8::MAX), Some(Attacker))
            .contains(Hex(96))
    );
    let d = defender(&s);
    activate(&mut s, d);
    let events = act(
        &mut s,
        &r,
        Command::Cast {
            spell: spell(&r, "sprites"),
            target: CastTarget::None,
            second: CastTarget::None,
        },
    )
    .unwrap();
    let summoned = s.stacks().len() - 1;
    assert_eq!(s.stacks()[summoned].hex, at(&r, 11, 5));
    // The gate opens before the stack is reported, and stays open while it stands there.
    let gate = events
        .iter()
        .position(|e| e.kind == EventKind::GateOpened)
        .unwrap();
    let created = events
        .iter()
        .position(|e| matches!(e.kind, EventKind::StackCreated { .. }))
        .unwrap();
    assert!(gate < created);
    assert!(s.siege.unwrap().gate_open);
}

// covers: BAT-SIE-028, BAT-MAG-038
#[test]
fn a_defender_s_clone_may_stand_on_the_closed_gate_and_opens_nothing() {
    let r = ruleset();
    let fort = town(&r, "t:castle", FortLevel::Fort, Moat::Absent);
    let field = field(&r);
    // The original at (12, 5), every hex of rings 1 and 2 but the gate's (11, 5) blocked.
    let origin = at(&r, 12, 5);
    let around: Vec<(i32, i32)> = (0..187u8)
        .map(Hex)
        .filter(|&h| h != origin && field.distance(h, origin) <= 2)
        .map(|h| field.coords(h))
        .collect();
    let obstacles: Vec<Obstacle> = blockers(&r, (1, 15), &[])
        .into_iter()
        .filter(|o| {
            let (x, y) = field.coords(Hex(o.hexes[0]));
            around.contains(&(x, y)) && (x, y) != (11, 5)
        })
        .collect();
    let cloner = {
        let mut h = mage(&r, &["copy"]);
        h.magic.masteries = masteries(&r, [3, 3, 3, 3]);
        h.magic.mana = 100;
        h
    };
    let mut s = at_town(
        &r,
        Some(fort),
        [None, Some(cloner)],
        &[("fixed", 5, 2, 5)],
        &[("fixed", 5, 12, 5)],
        |setup| setup.obstacles = obstacles,
    );
    let d = defender(&s);
    activate(&mut s, d);
    let events = act(
        &mut s,
        &r,
        Command::Cast {
            spell: spell(&r, "copy"),
            target: stack(d),
            second: CastTarget::None,
        },
    )
    .unwrap();
    let clone = s.stacks().len() - 1;
    assert_eq!(s.stacks()[clone].hex, at(&r, 11, 5));
    assert!(!events.iter().any(|e| e.kind == EventKind::GateOpened));
    assert!(!s.siege.unwrap().gate_open);
}

// ---------------------------------------------------------------- BAT-TAC-001

// covers: BAT-TAC-001
#[test]
fn a_town_has_its_tactics_phase() {
    let r = ruleset();
    let mut tactician = hero(&r);
    tactician.tactics = 3;
    for fort in [FortLevel::None, FortLevel::Castle] {
        let siege = town(&r, "t:castle", fort, Moat::Absent);
        let setup = BattleSetup {
            heroes: [Some(tactician.clone()), None],
            siege: Some(siege),
            ..setup(&r, &[(0, "fixed", 5)], &[(0, "fixed", 5)])
        };
        let s = BattleState::new(&r, &setup, 1).unwrap();
        assert_eq!(s.tactics().map(|t| t.side), Some(Attacker), "{fort:?}");
    }
}

// covers: BAT-SIE-026, BAT-MAG-026
#[test]
fn teleport_passes_the_closed_gate_for_the_caster_s_side_only() {
    use crate::Status;
    let r = ruleset();
    let fort = town(&r, "t:castle", FortLevel::Fort, Moat::Absent);
    let heroes = [
        Some(caster(&r, &["teleport"], 3)),
        Some(caster(&r, &["teleport"], 3)),
    ];
    let mut s = at_town(
        &r,
        Some(fort),
        heroes,
        &[("fixed", 5, 2, 5), ("fixed", 5, 2, 7)],
        &[("fixed", 5, 14, 9)],
        |_| {},
    );
    let gate = at(&r, 11, 5);
    let reaches = |s: &BattleState, side: BattleSide, id: u8| {
        let mut out = CastTargets::new();
        second_targets(s, &r, side, (spell(&r, "teleport"), stack(id)), &mut out);
        out.as_slice().contains(&CastTarget::Hex(gate))
    };
    // The attacker's Expert Teleport reaches anywhere inside but not the closed gate's hex.
    assert!(!reaches(&s, Attacker, 0));
    // Its own stack under the defender's Hypnotize: still the attacker's cast, still not the gate.
    let st = &mut s.stacks.as_mut_slice()[1];
    st.spells.status = st.spells.status.with(Status::HYPNOTIZED, true);
    let mut out = CastTargets::new();
    cast_targets(&s, &r, Attacker, spell(&r, "teleport"), &mut out);
    assert!(out.as_slice().contains(&stack(1)));
    assert!(!reaches(&s, Attacker, 1));
    // The defender's own cast on its stack passes the closed gate.
    let d = defender(&s);
    activate(&mut s, d);
    assert!(reaches(&s, Defender, d));
}

// The computer's retreat test finds its army walled out (0x41E440; E23 T6): the attacker at a
// town with walls and a closed gate, none of its living stacks flying, shooting walls (its
// Catapult among them) or able to shoot, every wall standing and every living defender but the
// turrets inside the walls.
// covers: BAT-AI-024
#[test]
fn the_computer_finds_an_army_walled_out_of_a_closed_town() {
    use crate::computer::walled_out;
    let r = ruleset();
    let fort = town(&r, "t:castle", FortLevel::Fort, Moat::Absent);
    let led = [Some(hero(&r)), Some(hero(&r))];
    // The attacker's stacks at `(x, y)`, the defender's 5 inside the walls (hex 100); the
    // attacker's Catapult (stack 2 at a town) dead unless `catapult`.
    let battle =
        |town: Option<Siege>, attacker: &[(&'static str, u32, i32, i32)], catapult: bool| {
            let mut s = at_town(
                &r,
                town,
                led.clone(),
                attacker,
                &[("fixed", 5, 14, 5)],
                |_| {},
            );
            for st in s.stacks.as_mut_slice() {
                if st.is_catapult() && !catapult {
                    st.count = 0;
                }
                if st.side == Defender {
                    st.hex = Hex(100);
                }
            }
            s
        };
    let walkers: &[(&'static str, u32, i32, i32)] = &[("fixed", 5, 1, 5), ("slow", 3, 1, 7)];
    let s = battle(Some(fort), walkers, false);
    assert!(walled_out(&s, &r, Attacker));
    assert!(!walled_out(&s, &r, Defender));
    // Its Catapult shoots the walls.
    assert!(!walled_out(
        &battle(Some(fort), walkers, true),
        &r,
        Attacker
    ));
    // Away from a town, at one without a fort.
    assert!(!walled_out(&battle(None, walkers, false), &r, Attacker));
    let open = town(&r, "t:castle", FortLevel::None, Moat::Absent);
    assert!(!walled_out(
        &battle(Some(open), walkers, false),
        &r,
        Attacker
    ));
    // A flyer, a shooter or a wall shooter among the attacker's living stacks.
    for unit in ["flyer", "archer", "cyclops"] {
        let s = battle(Some(fort), &[("fixed", 5, 1, 5), (unit, 1, 1, 7)], false);
        assert!(!walled_out(&s, &r, Attacker), "{unit}");
    }
    // A dead one does not count.
    let mut s = battle(Some(fort), &[("fixed", 5, 1, 5), ("flyer", 1, 1, 7)], false);
    s.stacks.as_mut_slice()[1].count = 0;
    assert!(walled_out(&s, &r, Attacker));
    // The gate open, a wall down, a defender outside the walls.
    let mut gate = battle(Some(fort), walkers, false);
    gate.siege.as_mut().unwrap().gate_open = true;
    assert!(!walled_out(&gate, &r, Attacker));
    let mut breach = battle(Some(fort), walkers, false);
    damage_part(&mut breach, 1, 2, &mut crate::NoopSink);
    assert!(!walled_out(&breach, &r, Attacker));
    let mut outside = battle(Some(fort), walkers, false);
    let defender = outside.stacks.as_mut_slice().last_mut().unwrap();
    defender.hex = Hex(5 * 17 + 4);
    assert!(!walled_out(&outside, &r, Attacker));
    defender_dies(&mut outside);
    assert!(walled_out(&outside, &r, Attacker));
}

/// The defender's last stack dies.
fn defender_dies(s: &mut BattleState) {
    let defender = s.stacks.as_mut_slice().last_mut().unwrap();
    defender.count = 0;
}
