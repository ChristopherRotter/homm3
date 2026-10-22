//! The computer's numbers the walker's hex scores read (E23 T3b, BAT-AI-007; `docs/formats/
//! installation.md` §7.36): an enemy's reach with its speed + 1 (0x420260's map), a Fire Wall's
//! damage, a damaging moat, and the enemy hero's best area spell (0x41F920), on the test
//! ruleset; the creature casts' (E23 T4a, BAT-AI-012, BAT-AI-013; §7.43): a damage spell's
//! value on a stack (0x436E30), the raise's count and a raised creature's worth, the estimated
//! chain and area, the casts' battlefield. The policy's use of them: `h3-battle-ai`'s tests and
//! the harvested `harvested_computer.rs` and `harvested_computer_casts.rs`.

use std::vec;
use std::vec::Vec;

use h3_content::SpellId;
use h3_content_battle::{BattleRuleset, ObstacleKind, SpellAi, SpellAiKind, SpellAiKinds};
use h3_contracts::{
    BattleEffect, BattleHero, BattleSetup, BattleSide, FortLevel, Moat, Rate, Siege,
};

use super::spells::{arena, arena_on, mage, spell};
use super::support::*;
use crate::geometry::{Field, Hex};
use crate::{
    BattleState, CastSpell, CastTarget, Command, Event, EventKind, StackId, Status, computer,
};

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

/// The test ruleset (`profile`) with `bolt`'s AI values at 25 for every mastery and the AI kinds
/// `kinds` (the odds count the kind `damage` alone, 0x420A80).
fn odds_ruleset(profile: &str, kinds: &[SpellAiKind]) -> BattleRuleset {
    let kinds = kinds
        .iter()
        .fold(SpellAiKinds::default(), |set, &k| set.with(k));
    ruleset_from(
        profile,
        |_| {},
        |_, battle| {
            for def in &mut battle.spells {
                if def.key.name() == "bolt" {
                    def.ai = SpellAi {
                        kinds,
                        values: [25; 4],
                    };
                }
            }
        },
    )
}

/// A mage with `bolt` at Expert, spell power 10 and a maximum of `max_mana` spell points, and
/// `effects`.
fn bolter(r: &BattleRuleset, max_mana: u16, effects: &[BattleEffect]) -> BattleHero {
    let mut h = mage(r, &["bolt"]);
    h.power = 10;
    h.magic.max_mana = max_mana;
    h.magic.masteries = vec![(spell(r, "bolt"), 3)];
    h.effects.extend_from_slice(effects);
    h
}

/// The hero's value of the attacker's hero `hero` against the side's strength `strength`, with
/// `attacker` its army.
fn hero_value(
    r: &BattleRuleset,
    hero: Option<BattleHero>,
    attacker: &[(&'static str, u32, i32, i32)],
    strength: i32,
) -> i32 {
    let s = arena(r, [hero, None], attacker, &[("fixed", 1, 10, 5)], 1);
    computer::hero_value(&s, r, BattleSide::Attacker, strength)
}

// covers: BAT-AI-008
#[test]
fn the_heros_value_in_the_odds() {
    for profile in ["t:faithful", "t:single"] {
        let r = odds_ruleset(profile, &[SpellAiKind::Damage]);
        let army: &[(&'static str, u32, i32, i32)] = &[("fixed", 1, 2, 5)];
        let value = |h: BattleHero, s: i32| hero_value(&r, Some(h), army, s);
        // installation.md §7.39's example: b = 25 × (10 + 3) = 325, X = 0.65 (row 3), Y =
        // 1.452 X + 0.492 = 1.4358 below Z = 4.53 X (100 / 4 = 25 casts, at most 13): 7179.
        assert_eq!(value(bolter(&r, 100, &[]), 5000), 7179, "{profile}");
        // One cast (4 of 4 mana): Z = 0.83 X = 0.5395 is lower: trunc(5000 × 0.5395) = 2697.
        assert_eq!(value(bolter(&r, 4, &[]), 5000), 2697, "{profile}");
        // The cost above the maximum mana (not the points left): 0.
        assert_eq!(value(bolter(&r, 3, &[]), 5000), 0, "{profile}");
        // From X = 4.8 on, Y = 4: at most four times the side's strength.
        assert_eq!(value(bolter(&r, 100, &[]), 100), 400, "{profile}");
        // A side without strength: 0 (the original's NaN).
        assert_eq!(value(bolter(&r, 100, &[]), 0), 0, "{profile}");
        // Sorcery +50 %: d = trunc(325 × 1.5) = 487, X = 0.974 (row 2), Y = 0.987 X + 0.842:
        // trunc(5000 × 1.803338) = 9016.
        let sorcery = BattleEffect::SpellDamage {
            scope: h3_content::SpellScope::All,
            rate: Rate::bp(5000),
        };
        assert_eq!(value(bolter(&r, 100, &[sorcery]), 5000), 9016, "{profile}");
        // The hero's own spell level limit (Recanter's Cloak: here 0) bars the spell.
        let limit = BattleEffect::SpellLevelLimit { max_level: 0 };
        assert_eq!(value(bolter(&r, 100, &[limit]), 5000), 0, "{profile}");
    }
    let r = odds_ruleset("t:faithful", &[SpellAiKind::Damage]);
    // Without a spell book, a hero, or while casting is forbidden: 0.
    let mut no_book = bolter(&r, 100, &[]);
    no_book.magic.spell_book = None;
    let army: &[(&'static str, u32, i32, i32)] = &[("fixed", 1, 2, 5)];
    assert_eq!(hero_value(&r, Some(no_book), army, 5000), 0);
    assert_eq!(hero_value(&r, None, army, 5000), 0);
    let mut s = arena(
        &r,
        [Some(bolter(&r, 100, &[])), None],
        army,
        &[("fixed", 1, 10, 5)],
        1,
    );
    s.limits.no_casting = true;
    assert_eq!(computer::hero_value(&s, &r, BattleSide::Attacker, 5000), 0);
    // No army changes the cost here (0x4E5240 given no enemy army skips both): the hero's own
    // Mages (−2) do not make 2 mana enough for a cost of 4, the enemy's Pegasi (+2) do not make
    // 4 too little.
    let mages: &[(&'static str, u32, i32, i32)] = &[("fixed", 1, 2, 5), ("magus", 1, 2, 7)];
    assert_eq!(hero_value(&r, Some(bolter(&r, 2, &[])), mages, 5000), 0);
    assert_eq!(hero_value(&r, Some(bolter(&r, 4, &[])), mages, 5000), 2697);
    let s = arena(
        &r,
        [Some(bolter(&r, 4, &[])), None],
        army,
        &[("fixed", 1, 10, 5), ("pegasus", 1, 10, 7)],
        1,
    );
    assert_eq!(
        computer::hero_value(&s, &r, BattleSide::Attacker, 5000),
        2697
    );
    // Not the kind `damage` (no AI kind): not counted; `damage` with another kind: 1.
    let r = odds_ruleset("t:faithful", &[]);
    assert_eq!(hero_value(&r, Some(bolter(&r, 100, &[])), army, 5000), 0);
    let r = odds_ruleset("t:faithful", &[SpellAiKind::Damage, SpellAiKind::Special]);
    assert_eq!(hero_value(&r, Some(bolter(&r, 100, &[])), army, 5000), 1);
}

// covers: BAT-AI-009
#[test]
fn a_turns_map_reaches_within_the_speed() {
    let r = ruleset();
    // A fixed stack (speed 5) marks an enemy within its speed only, so a stop marks nothing.
    let s = arena(
        &r,
        [None, None],
        &[("fixed", 1, 2, 5)],
        &[("fixed", 1, 4, 5), ("fixed", 1, 14, 5)],
        1,
    );
    let speed = u16::from(s.stacks()[0].speed_now());
    let map = computer::turn_map(&s, &r, StackId(0), BattleSide::Attacker);
    assert!(map.mark(at(&r, 4, 5)).is_some_and(|c| c <= speed));
    assert_eq!(map.mark(at(&r, 14, 5)), None);
    assert!(!computer::kept_out(&s, &r, StackId(0)));
    assert!(computer::siege(&s).is_none());
    // The turns to its record's enemy at a speed of 0: 1 (0x448BD0).
    let ctx = computer::context(&s, &r, BattleSide::Attacker);
    let threat = computer::threat(&s, &r, &ctx, StackId(0)).unwrap();
    assert_eq!(threat.turns_at(0), 1);
}

/// The test ruleset with a Fight Value of 100 for every creature (the computer's worth; the test
/// creatures have none).
fn fighting() -> BattleRuleset {
    ruleset_from(
        "t:faithful",
        |_| {},
        |core, _| {
            for def in &mut core.creatures {
                def.info.fight_value = 100;
            }
        },
    )
}

// covers: BAT-AI-013
#[test]
fn the_computers_value_of_a_spells_damage_on_a_stack() {
    let r = fighting();
    let bolt = spell(&r, "bolt");
    // The attacker's mage against five `fixed` (10 hit points each) and a troglo, immune to the
    // bolt.
    let mut s = arena(
        &r,
        [Some(mage(&r, &["bolt"])), None],
        &[("fixed", 1, 2, 5)],
        &[("fixed", 5, 10, 5), ("troglo", 1, 10, 3)],
        1,
    );
    let ctx = computer::context(&s, &r, BattleSide::Attacker);
    let value = |s: &BattleState, damage, bonuses, id| {
        computer::spell_damage_value(s, &r, &ctx, (bolt, damage), bonuses, StackId(id))
    };
    let kills = |s: &BattleState, damage| {
        computer::damage_value(s, &r, &ctx, (StackId(1), damage), (false, false))
    };
    // 30 kills three of the five: their value; 100 is capped at their 50 hit points.
    assert!(kills(&s, 30) > 0);
    assert_eq!(value(&s, 30, false, 1), kills(&s, 30));
    assert_eq!(value(&s, 100, false, 1), kills(&s, 50));
    // The immune troglo: the effect test's factor 0, nothing; a spell the ruleset lacks, nothing.
    assert_eq!(value(&s, 30, false, 2), 0);
    let missing = SpellId::from_index(r.spells().len()).unwrap();
    assert_eq!(
        computer::spell_damage_value(&s, &r, &ctx, (missing, 30), false, StackId(1)),
        0
    );
    // The side's hero's bonuses with `bonuses` only: 50 % on every spell makes 20 into 30.
    s.heroes[0].as_mut().unwrap().magic.bonuses.all = Rate::bp(5000);
    assert_eq!(value(&s, 20, true, 1), kills(&s, 30));
    assert_eq!(value(&s, 20, false, 1), kills(&s, 20));
    // A stack that loses its turns, worth `w`: a value `v` below it is `2v − w`; its whole hit
    // points are worth `w` itself.
    let fixed = &mut s.stacks.as_mut_slice()[1];
    fixed.spells.status = fixed.spells.status.with(Status::NO_TURNS, true);
    let w = computer::strength(&s, &r, &ctx, StackId(1));
    assert_eq!(value(&s, 20, false, 1), 2 * kills(&s, 20) - w);
    assert_eq!(value(&s, 50, false, 1), w);
    // A dead stack: nothing.
    s.stacks.as_mut_slice()[1].count = 0;
    assert_eq!(value(&s, 30, false, 1), 0);
}

// covers: BAT-AI-012
#[test]
fn the_computers_raise_count_and_a_raised_creatures_worth() {
    let r = fighting();
    // The defender's three heralds (100 hit points each, an Archangel's), two fiends (50, a Pit
    // Lord's, raising imps of 10) and eight `fixed` (10 each) that lost five; the attacker's
    // `fixed` and an imp.
    let mut s = arena(
        &r,
        [None, None],
        &[("fixed", 1, 2, 5), ("imp", 1, 2, 7)],
        &[
            ("herald", 3, 10, 1),
            ("fiend", 2, 10, 3),
            ("fixed", 8, 10, 5),
        ],
        1,
    );
    s.stacks.as_mut_slice()[4].count = 3;
    let count = |s: &BattleState, caster, target| {
        computer::raise_count(s, &r, StackId(caster), StackId(target))
    };
    // The heralds: min(5 lost, 300 / 10) whole creatures; one herald with seven lost min(7,
    // 100 / 10).
    assert_eq!(count(&s, 2, 4), 5);
    s.stacks.as_mut_slice()[2].count = 1;
    s.stacks.as_mut_slice()[4].count = 1;
    assert_eq!(count(&s, 2, 4), 7);
    // A target of 30 hit points per creature: 100 / 30 = 3 whole ones.
    s.stacks.as_mut_slice()[4].max_hp = 30;
    s.stacks.as_mut_slice()[4].top_hp = 30;
    assert_eq!(count(&s, 2, 4), 3);
    // The fiends from the dead `fixed`: min(8, min(100, 8 × 30) div 10) = 8; one fiend 5.
    s.stacks.as_mut_slice()[4].count = 0;
    assert_eq!(count(&s, 3, 4), 8);
    s.stacks.as_mut_slice()[3].count = 1;
    assert_eq!(count(&s, 3, 4), 5);
    // A stack that raises nothing.
    assert_eq!(count(&s, 0, 4), 0);
    // A raised imp is worth what an imp of the side on the field is (the same hero's bonuses);
    // a creature the battle creates for nobody, nothing.
    let ctx = computer::context(&s, &r, BattleSide::Attacker);
    let imp = r.creature_by_key("t:imp").unwrap();
    let raised = computer::created_worth(&s, &r, &ctx, BattleSide::Attacker, imp);
    assert!(raised > h3_kernel::arith::X87::ZERO);
    assert_eq!(raised, computer::worth(&s, &r, &ctx, StackId(1), false));
    let fixed = r.creature_by_key("t:fixed").unwrap();
    assert_eq!(
        computer::created_worth(&s, &r, &ctx, BattleSide::Attacker, fixed),
        h3_kernel::arith::X87::ZERO
    );
}

// covers: BAT-AI-013, BAT-MAG-006, BAT-MAG-007
#[test]
fn the_computers_chain_and_area_are_the_rules_without_rolls() {
    let r = ruleset();
    let chain = spell(&r, "chain");
    let nova = spell(&r, "nova");
    let attacker = &[("fixed", 1, 2, 5)];
    let defender = &[
        ("fixed", 9, 10, 5),
        ("fixed", 9, 11, 3),
        ("fixed", 9, 12, 7),
        ("fixed", 9, 14, 5),
    ];
    let mut s = arena(
        &r,
        [Some(mage(&r, &["chain"])), None],
        attacker,
        defender,
        1,
    );
    // The chain at mastery 0 (three stacks) from (10, 5), each next taking half: the stacks and
    // values the cast strikes.
    let estimate: Vec<(StackId, u64)> =
        computer::chain(&s, &r, BattleSide::Attacker, (chain, 0), (StackId(1), 40)).collect();
    assert_eq!(estimate.len(), 3);
    assert_eq!(
        estimate.iter().map(|&(_, v)| v).collect::<Vec<_>>(),
        vec![40, 20, 10]
    );
    let cast = Command::Cast {
        spell: chain,
        target: CastTarget::Stack(StackId(1)),
        second: CastTarget::None,
    };
    let struck: Vec<(StackId, u64)> = act(&mut s, &r, cast)
        .unwrap()
        .iter()
        .filter_map(|e| match e.kind {
            EventKind::SpellDamage { target, value, .. } => Some((target, value)),
            _ => None,
        })
        .collect();
    assert_eq!(estimate, struck);
    // From Advanced four, the first stack's value kept; a spell without a chain strikes only its
    // stack, a spell the ruleset lacks none.
    let s = arena(&r, [None, None], attacker, defender, 1);
    assert_eq!(
        computer::chain(&s, &r, BattleSide::Attacker, (chain, 2), (StackId(1), 40)).count(),
        4
    );
    let bolt = spell(&r, "bolt");
    assert_eq!(
        computer::chain(&s, &r, BattleSide::Attacker, (bolt, 2), (StackId(1), 40)).count(),
        0
    );
    let missing = SpellId::from_index(r.spells().len()).unwrap();
    assert_eq!(
        computer::chain(&s, &r, BattleSide::Attacker, (missing, 2), (StackId(1), 40)).count(),
        0
    );
    // The nova reaches the stacks within a hex of its centre: at (11, 4) those at (10, 5) or
    // (11, 3) beside it, by id.
    let field = Field::new(r.battle().field);
    let centre = at(&r, 11, 4);
    let reached: Vec<StackId> =
        computer::area_stacks(&s, &r, BattleSide::Attacker, (nova, 0), centre).collect();
    let near: Vec<StackId> = (0u8..)
        .zip(s.stacks())
        .filter(|(_, st)| field.distance(st.hex, centre) <= 1)
        .map(|(i, _)| StackId(i))
        .collect();
    assert!(!reached.is_empty());
    assert_eq!(reached, near);
    assert_eq!(
        computer::area_stacks(&s, &r, BattleSide::Attacker, (missing, 0), at(&r, 11, 4)).count(),
        0
    );
}

// covers: BAT-AI-012, BAT-AI-013, BAT-TER-004, BAT-TER-005
#[test]
fn the_creature_casts_on_the_battlefields() {
    let r = ruleset();
    let on = |field: Option<&str>| {
        arena_on(
            &r,
            [None, None],
            &[("fixed", 1, 2, 5)],
            &[("fixed", 1, 10, 5)],
            1,
            |s| {
                s.battlefield = field.map(|f| h3_content::ContentKey::new("t", f).unwrap());
            },
        )
    };
    let (bolt, nova) = (spell(&r, "bolt"), spell(&r, "nova"));
    // Magic Plains: every creature cast at Expert, and the air and fire schools for a hero.
    let plains = on(Some("plains"));
    assert_eq!(computer::creature_mastery(&plains, 2), 3);
    assert!(computer::field_raises(&plains, &r, bolt));
    assert!(!computer::creature_casts_blocked(&plains));
    // An elemental field raises its school only: fire, not the air bolt.
    let fiery = on(Some("fiery"));
    assert_eq!(computer::creature_mastery(&fiery, 2), 2);
    assert!(computer::field_raises(&fiery, &r, nova));
    assert!(!computer::field_raises(&fiery, &r, bolt));
    // Ordinary ground raises nothing.
    let plain = on(None);
    assert!(!computer::field_raises(&plain, &r, nova));
    // Cursed Ground, and a hero's or a garrison's ban, block the creature casts.
    assert!(computer::creature_casts_blocked(&on(Some("cursed"))));
    let mut banned = on(None);
    banned.limits.no_casting = true;
    assert!(computer::creature_casts_blocked(&banned));
    // A cast names its spell only when drawn or its own.
    assert_eq!(CastSpell::Named(bolt).named(), Some(bolt));
    assert_eq!(CastSpell::AtResolution.named(), None);
    assert_eq!(CastSpell::Raise.named(), None);
}

/// Puts the timed spell `name` at `mastery` on stack `id` for five rounds, as a cast would (test
/// setup).
fn give(s: &mut BattleState, r: &BattleRuleset, (id, name): (u8, &str), mastery: u8) {
    let spell_id = spell(r, name);
    let cast = crate::effects::TimedCast {
        id: spell_id,
        spell: r.spell(spell_id).unwrap(),
        mastery,
        rounds: 5,
        own: crate::magic::SpellBonus::default(),
    };
    crate::effects::apply(s, r, cast, StackId(id), &mut Vec::<Event>::new());
}

// covers: BAT-AI-016
#[test]
fn a_removals_copy_finds_the_stack_itself_among_its_berserk_targets() {
    // The original's removal values a lone copy of a record (0x437A00): 0x445490 tells the
    // battle's records from it by pointer, so the stack's own record is a target, the nearest
    // (E23 T4d, survey slip 24). A walker at (5, 5) beside an enemy at (6, 5), another far.
    let r = ruleset();
    let s = arena(
        &r,
        [None, None],
        &[("fixed", 5, 5, 5)],
        &[("fixed", 5, 6, 5), ("fixed", 5, 12, 2)],
        1,
    );
    let me = StackId(0);
    let (normal, normal_map) = computer::berserk_search(&s, &r, me);
    let (copy, copy_map) = computer::berserk_search_of_copy(&s, &r, me);
    assert_eq!(normal.collect::<Vec<_>>(), [StackId(1)]);
    // The record itself at the search's start (cost 0), with the neighbour marked at 0, in record
    // order; the copy's search is the stack's (the pathfinder tells a hex's stack by side and
    // index, 0x446C40).
    assert_eq!(copy.collect::<Vec<_>>(), [me, StackId(1)]);
    let (normal_map, copy_map) = (normal_map.unwrap(), copy_map.unwrap());
    for hex in (0..=u8::MAX).map(Hex) {
        assert_eq!(normal_map.position(hex), copy_map.position(hex));
        assert_eq!(normal_map.mark(hex), copy_map.mark(hex));
    }
    // With no neighbour the record alone; a shooter by hex distance, the record at 0.
    let s = arena(
        &r,
        [None, None],
        &[("fixed", 5, 2, 5)],
        &[("fixed", 5, 12, 2)],
        1,
    );
    assert_eq!(
        computer::berserk_search(&s, &r, me).0.collect::<Vec<_>>(),
        [StackId(1)]
    );
    assert_eq!(
        computer::berserk_search_of_copy(&s, &r, me)
            .0
            .collect::<Vec<_>>(),
        [me]
    );
    let s = arena(
        &r,
        [None, None],
        &[("archer", 5, 2, 5)],
        &[("fixed", 5, 12, 2)],
        1,
    );
    let (copy, map) = computer::berserk_search_of_copy(&s, &r, me);
    assert_eq!(copy.collect::<Vec<_>>(), [me]);
    assert!(map.is_none());
}

// covers: BAT-AI-016
#[test]
fn a_removals_copy_strikes_the_stack_itself() {
    // The copy without the battle record's Bless strikes the record (its mean damage), which
    // strikes back blessed (its maximum): each party read from its own state (E23 T4d).
    let r = ruleset();
    let mut battle = arena_on(
        &r,
        [None, None],
        &[("range", 10, 5, 5)],
        &[("fixed", 5, 12, 2)],
        1,
        |s| s.difficulty = 1,
    );
    give(&mut battle, &r, (0, "bless"), 0);
    let me = StackId(0);
    let mut copy = battle.clone();
    computer::end_spell(&mut copy, &r, (me, spell(&r, "bless")), true);
    let hp = computer::hit_points(&battle, me);
    let (a, t) = computer::exchange_with_itself(
        (&battle, &copy),
        &r,
        BattleSide::Attacker,
        (me, (hp, hp)),
        false,
    );
    let dealt = computer::damage_estimate(&copy, &r, (me, me), 10, false, 0);
    let left = (hp - dealt).div_ceil(10);
    let back = computer::damage_estimate(&battle, &r, (me, me), left, false, 0);
    assert!(back > computer::damage_estimate(&copy, &r, (me, me), left, false, 0));
    assert_eq!((a, t), (hp - back, hp - dealt));
    // On one state it is the stack's exchange with itself.
    let same = computer::exchange_with_itself(
        (&battle, &battle),
        &r,
        BattleSide::Attacker,
        (me, (hp, hp)),
        false,
    );
    let one = computer::exchange(
        &battle,
        &r,
        BattleSide::Attacker,
        ((me, hp), (me, hp)),
        (false, 0),
    );
    assert_eq!(same, one);
    // A shooter's copy that can shoot shoots: no retaliation.
    let battle = arena(
        &r,
        [None, None],
        &[("archer", 10, 2, 5)],
        &[("fixed", 5, 12, 2)],
        1,
    );
    let hp = computer::hit_points(&battle, me);
    let shot = computer::exchange_with_itself(
        (&battle, &battle),
        &r,
        BattleSide::Attacker,
        (me, (hp, hp)),
        true,
    );
    let dealt = computer::damage_estimate(&battle, &r, (me, me), 10, true, 0);
    assert_eq!(shot, (hp, hp - dealt));
}

// covers: BAT-AI-016
#[test]
fn a_two_hex_shooters_copy_is_blocked_by_the_stack_itself() {
    // The battle's record of a two-hex shooter stands beside its lone copy, at the copy's other
    // hex (0x469600 → 0x442880 by pointer): the copy is blocked where either is berserk or the
    // copy is hypnotized (E23 T4d, survey slip 24).
    let r = ruleset();
    let me = StackId(0);
    let blocked = |name: &'static str, spells: &[&str], ended: &[&str]| {
        let mut battle = arena(
            &r,
            [None, None],
            &[(name, 10, 3, 5)],
            &[("fixed", 5, 12, 2)],
            1,
        );
        for spell_name in spells {
            give(&mut battle, &r, (0, spell_name), 0);
        }
        let mut copy = battle.clone();
        for spell_name in ended {
            computer::end_spell(&mut copy, &r, (me, spell(&r, spell_name)), true);
        }
        computer::block_by_itself(&mut copy, &battle, me);
        !computer::shoots(&copy, &r, me)
    };
    // The record berserk, the copy not: blocked; the copy hypnotized: blocked; neither: free.
    assert!(blocked("medusa", &["berserk"], &["berserk"]));
    assert!(blocked("medusa", &["hypnotize", "bless"], &["bless"]));
    assert!(!blocked("medusa", &["hypnotize"], &["hypnotize"]));
    assert!(!blocked("medusa", &["bless"], &["bless"]));
    // A one-hex shooter has no other hex.
    assert!(!blocked("archer", &["berserk"], &["berserk"]));
}

// covers: BAT-SIE-008, BAT-AI-019
#[test]
fn the_computer_s_wall_test_asks_from_a_shooter_s_position() {
    // The computer's Earthquake value tests the walls from a shooter's position (0x43BAC3), the
    // rule from the hex the shooter turns to (survey slip 29). A two-hex shooter stands with its
    // rear outside at (9, 4) and its front on row 4's destroyed wall hex (10, 4), and shoots up at
    // (13, 2) inside: from its rear the line meets row 3's wall, from its front the shot starts at
    // the walls' line and nothing halves it.
    let r = ruleset();
    let fort = Siege {
        faction: r.faction_by_key("t:castle").unwrap(),
        fort: FortLevel::Fort,
        moat: Moat::Absent,
        structures: 0,
        escape_tunnel: false,
        morale: 0,
        luck: 0,
    };
    let at_fort = |shooter: &'static str| {
        let mut s = arena_on(
            &r,
            [None, None],
            &[(shooter, 10, 9, 4)],
            &[("fixed", 10, 13, 2)],
            1,
            |s| s.siege = Some(fort),
        );
        crate::siege::damage_part(&mut s, 2, u8::MAX, &mut crate::NoopSink);
        assert!(!s.siege.unwrap().walls().contains(Hex(78)));
        s
    };
    let (me, enemy) = (StackId(0), StackId(1));
    let s = at_fort("medusa");
    assert_eq!(s.stacks()[0].hex, at(&r, 9, 4));
    assert!(computer::walls_halve_from_position(&s, me, enemy));
    assert!(!computer::walls_halve(&s, me, enemy));
    // A one-hex shooter there shoots from its position either way; a shooter the walls never
    // halve, neither.
    let s = at_fort("archer");
    assert!(computer::walls_halve_from_position(&s, me, enemy));
    assert!(computer::walls_halve(&s, me, enemy));
    let s = at_fort("mage_s");
    assert!(!computer::walls_halve_from_position(&s, me, enemy));
    assert!(!computer::walls_halve(&s, me, enemy));
    // On the open field, nothing.
    let s = arena(
        &r,
        [None, None],
        &[("archer", 10, 9, 4)],
        &[("fixed", 10, 13, 2)],
        1,
    );
    assert!(!computer::walls_halve_from_position(&s, me, enemy));
}
