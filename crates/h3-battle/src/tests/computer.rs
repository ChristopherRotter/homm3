//! The computer's numbers the walker's hex scores read (E23 T3b, BAT-AI-007; `docs/formats/
//! installation.md` §7.36): an enemy's reach with its speed + 1 (0x420260's map), a Fire Wall's
//! damage, a damaging moat, and the enemy hero's best area spell (0x41F920), on the test
//! ruleset. The policy's use of them: `h3-battle-ai`'s tests and the harvested
//! `harvested_computer.rs`.

use std::vec::Vec;

use h3_content::SpellId;
use h3_content_battle::{BattleRuleset, ObstacleKind};
use h3_contracts::{BattleEffect, BattleHero, BattleSetup, BattleSide, FortLevel, Moat, Siege};

use super::spells::{arena, mage, spell};
use super::support::*;
use crate::geometry::{Field, Hex};
use crate::{BattleState, Event, StackId, computer};

/// Places `side`'s obstacles of `kind` (the spell `name`'s) on `cells` with `value`, as a cast
/// would (test setup).
fn obstacles(
    s: &mut BattleState,
    r: &BattleRuleset,
    (name, kind): (&str, ObstacleKind),
    side: BattleSide,
    (cells, value): (&[(i32, i32)], u64),
) {
    let id: SpellId = spell(r, name);
    let shape = r.spell(id).unwrap().magic.obstacle.clone().unwrap();
    let hexes: Vec<Hex> = cells.iter().map(|&(x, y)| at(r, x, y)).collect();
    crate::obstacles::place(
        s,
        (id, kind, &shape),
        side,
        (&hexes, value),
        &mut Vec::<Event>::new(),
    );
}

// covers: BAT-AI-007
#[test]
fn an_enemys_reach_counts_with_its_speed_plus_one() {
    let r = ruleset();
    let f = Field::new(r.battle().field);
    // The defender's walker (speed 5) at (10, 5); the attacker's stacks at (4, 5), whose
    // neighbour (5, 5) it reaches at 5, and at (2, 5), out of reach.
    let s = arena(
        &r,
        [None, None],
        &[("fixed", 1, 2, 5), ("fixed", 1, 4, 5)],
        &[("fixed", 1, 10, 5)],
        1,
    );
    let enemy = StackId(2);
    let map = computer::reach_map(&s, &r, enemy);
    let from = at(&r, 10, 5);
    // Its own position and every free hex off the side columns within 6 are reached, none
    // further.
    for hex in (0u8..).take(f.hex_count()).map(Hex) {
        let (x, _) = f.coords(hex);
        let free = s.stacks().iter().all(|st| st.hex != hex) && (1..=15).contains(&x);
        let near = f.distance(from, hex) <= 6;
        assert_eq!(map.reached(hex), hex == from || (free && near), "{hex:?}");
    }
    assert_eq!(map.mark(at(&r, 4, 5)), Some(5));
    assert_eq!(map.mark(at(&r, 2, 5)), None);
    // The walker's own map (range 127) marks the far stack too.
    let own = computer::map(&s, &r, enemy, BattleSide::Defender);
    assert!(own.mark(at(&r, 2, 5)).is_some());
}

// covers: BAT-AI-007, BAT-MAG-032
#[test]
fn a_stop_costs_the_rest_of_a_turn_of_speed_plus_one_and_marks_nothing_then() {
    let r = ruleset();
    // A corridor on row 5 (rows 4 and 6 blocked from x = 3 to 12): the defender's two-hex walker
    // (speed 5) at (10, 5) covers (10, 5) and (9, 5); its pit at (7, 5). Its position (8, 5)
    // covers the pit: entered at 1 + the rest of the turn. With speed + 1 (6) as range and speed
    // that is 6, and from there its front hex lies on a stop: nothing is marked (0x4B378D), so the
    // attacker at (6, 5) is not; the walker's own map (speed 5, range 127) reaches (8, 5) at 5 and
    // marks it there.
    let walls: Vec<(i32, i32)> = (3..=12).flat_map(|x| [(x, 4), (x, 6)]).collect();
    let mut s = battle_with_obstacles(&r, &[(0, "fixed", 1)], &[(0, "wide", 1)], &walls);
    place(&mut s, &r, 0, 6, 5);
    place(&mut s, &r, 1, 10, 5);
    obstacles(
        &mut s,
        &r,
        ("quicksand", ObstacleKind::Pit),
        BattleSide::Defender,
        (&[(7, 5)], 0),
    );
    let enemy = StackId(1);
    let reach = computer::reach_map(&s, &r, enemy);
    assert!(reach.reached(at(&r, 8, 5)) && reach.is_stop(at(&r, 7, 5)));
    assert_eq!(reach.cost(at(&r, 8, 5)), 6);
    assert_eq!(reach.mark(at(&r, 6, 5)), None);
    let own = computer::map(&s, &r, enemy, BattleSide::Defender);
    assert_eq!(own.cost(at(&r, 8, 5)), 5);
    assert_eq!(own.mark(at(&r, 6, 5)), Some(5));
}

// covers: BAT-AI-007, BAT-MAG-028
#[test]
fn a_fire_walls_damage_and_whether_it_hurts() {
    let r = ruleset();
    let mut s = arena(
        &r,
        [None, None],
        &[("fixed", 1, 2, 5)],
        &[("ember", 1, 10, 5), ("fixed", 1, 12, 5)],
        1,
    );
    obstacles(
        &mut s,
        &r,
        ("firewall", ObstacleKind::Fire),
        BattleSide::Attacker,
        (&[(8, 5)], 30),
    );
    let wall = at(&r, 8, 5);
    // Without a hero, the wall's value: a fire-immune stack is not hurt.
    assert_eq!(
        computer::fire_wall_damage(&s, &r, wall, StackId(2)),
        Some((30, true))
    );
    assert_eq!(
        computer::fire_wall_damage(&s, &r, wall, StackId(1)),
        Some((30, false))
    );
    assert_eq!(
        computer::fire_wall_damage(&s, &r, at(&r, 9, 5), StackId(2)),
        None
    );
}

/// A battle at the test town's Citadel with `moat`.
fn moated(r: &BattleRuleset, moat: Moat) -> BattleState {
    let setup = BattleSetup {
        siege: Some(Siege {
            faction: r.faction_by_key("t:castle").unwrap(),
            fort: FortLevel::Citadel,
            moat,
            structures: 0,
            escape_tunnel: false,
            morale: 0,
            luck: 0,
        }),
        ..setup(r, &[(0, "fixed", 1)], &[(0, "fixed", 1)])
    };
    BattleState::new(r, &setup, 5).unwrap()
}

// covers: BAT-AI-007, BAT-SIE-005
#[test]
fn a_damaging_moat_with_its_hexes_and_damage() {
    let r = turret_ruleset("t:faithful");
    let mut s = moated(&r, Moat::Damaging);
    let town = s.siege.unwrap();
    let damage = u64::from(r.siege(town.faction).unwrap().moat_damage);
    assert!(damage > 0);
    assert_eq!(computer::damaging_moat(&s), Some((town.moat(), damage)));
    // The gate open: the drawbridge's hex leaves the moat.
    let mut open = town;
    open.gate_open = true;
    s.siege = Some(open);
    let (hexes, _) = computer::damaging_moat(&s).unwrap();
    assert!(hexes.len() < town.moat().len());
    // A harmless moat, none at all, the open field: none.
    assert_eq!(computer::damaging_moat(&moated(&r, Moat::Harmless)), None);
    assert_eq!(computer::damaging_moat(&moated(&r, Moat::Absent)), None);
    let field = BattleState::new(&r, &setup(&r, &[(0, "fixed", 1)], &[(0, "fixed", 1)]), 5);
    assert_eq!(computer::damaging_moat(&field.unwrap()), None);
}

/// A mage with `spells`, spell power 3 and `mana`.
fn caster(r: &BattleRuleset, spells: &[&str], mana: u16) -> BattleHero {
    let mut h = mage(r, spells);
    h.magic.mana = mana;
    h
}

// covers: BAT-AI-007, BAT-MAG-004, BAT-MAG-008
#[test]
fn the_heros_best_area_spell_on_a_stack() {
    let best = |profile: &str, hero: BattleHero, target: &'static str| {
        let r = ruleset_with(profile, |_| {});
        let s = arena(
            &r,
            [Some(hero), None],
            &[("fixed", 1, 2, 5)],
            &[(target, 1, 10, 5)],
            1,
        );
        computer::area_spell_damage(&s, &r, BattleSide::Attacker, StackId(1))
    };
    let r = ruleset();
    // nova: power 3 × 10 + 10 = 40 at mastery 0; it costs 5.
    assert_eq!(best("t:faithful", caster(&r, &["nova"], 15), "fixed"), 40);
    assert_eq!(best("t:faithful", caster(&r, &["nova"], 5), "fixed"), 40);
    assert_eq!(best("t:faithful", caster(&r, &["nova"], 4), "fixed"), 0);
    // Not an area aimed at a hex, or none at all: none.
    assert_eq!(best("t:faithful", caster(&r, &["chain"], 15), "fixed"), 0);
    assert_eq!(best("t:faithful", caster(&r, &[], 15), "fixed"), 0);
    // Immune: f = 0. Resistant (20 %): trunc(40 × 0.8), exact or in single precision, 32.
    assert_eq!(best("t:faithful", caster(&r, &["nova"], 15), "ember"), 0);
    assert_eq!(best("t:faithful", caster(&r, &["nova"], 15), "dwarf"), 32);
    assert_eq!(best("t:single", caster(&r, &["nova"], 15), "dwarf"), 32);
    // The Orb of Vulnerability negates the resistance (BAT-MAG-066): f = 1.
    let mut orb = caster(&r, &["nova"], 15);
    orb.effects.push(BattleEffect::NegateImmunities);
    assert_eq!(best("t:faithful", orb, "dwarf"), 40);
    // Without a hero, or while casting is forbidden: none.
    let s = arena(
        &r,
        [None, None],
        &[("fixed", 1, 2, 5)],
        &[("fixed", 1, 10, 5)],
        1,
    );
    assert_eq!(
        computer::area_spell_damage(&s, &r, BattleSide::Attacker, StackId(1)),
        0
    );
    let mut s = arena(
        &r,
        [Some(caster(&r, &["nova"], 15)), None],
        &[("fixed", 1, 2, 5)],
        &[("fixed", 1, 10, 5)],
        1,
    );
    s.limits.no_casting = true;
    assert_eq!(
        computer::area_spell_damage(&s, &r, BattleSide::Attacker, StackId(1)),
        0
    );
}

// covers: BAT-AI-007, BAT-MAG-008
#[test]
fn an_aura_beside_the_stack_shrinks_the_factor() {
    // Beside a unicorn's 20 % aura: 40 × 0.8 = 32; with the dwarf's own 20 % also: 40 × 0.8 ×
    // 0.8, 25.6, 25; in single precision 0.8 is 0.800000011920929: 25.6000007629, 25.
    for profile in ["t:faithful", "t:single"] {
        let r = ruleset_with(profile, |_| {});
        let value = |target: &'static str| {
            let s = arena(
                &r,
                [Some(caster(&r, &["nova"], 15)), None],
                &[("fixed", 1, 2, 5)],
                &[(target, 1, 10, 5), ("unicorn", 1, 11, 5)],
                1,
            );
            computer::area_spell_damage(&s, &r, BattleSide::Attacker, StackId(1))
        };
        assert_eq!(value("fixed"), 32, "{profile}");
        assert_eq!(value("dwarf"), 25, "{profile}");
    }
}

// covers: BAT-AI-007
#[test]
fn the_context_with_its_least_attack_for_both_bonuses() {
    let r = ruleset();
    let s = arena(
        &r,
        [None, None],
        &[("fixed", 1, 2, 5)],
        &[("fixed", 1, 10, 5)],
        1,
    );
    let mut ctx = crate::aim::context(&s, &r, Field::new(r.battle().field), BattleSide::Attacker);
    ctx.min_attack = 3;
    ctx.min_defense = 7;
    let both = ctx.attack_for_both();
    assert_eq!((both.min_attack, both.min_defense), (3, 3));
    assert_eq!(both.strengths(), ctx.strengths());
}
