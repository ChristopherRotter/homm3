//! A hero's magic in battle (E04b T2b, T2d): BAT-MAG-001 to BAT-MAG-004.

use std::vec;
use std::vec::Vec;

use h3_content::{SchoolId, SpellId, SpellScope};
use h3_content_battle::BattleRuleset;
use h3_contracts::{
    BallistaControl, BattleEffect, BattleHero, BattleResult, BattleSetup, BattleSide,
};

use super::support::*;
use crate::{
    BattleState, CastTarget, CastTargets, Command, CommandError, Event, EventKind, HeroError,
    SetupError, SpellBonus, SpellSet, StackId, cast_targets, castable_spells, observe,
};

fn spell(r: &BattleRuleset, name: &str) -> SpellId {
    r.spell_by_key(&std::format!("t:{name}")).unwrap()
}

/// A hero with spell power 3, 15 mana and `spells` in its book.
fn mage(r: &BattleRuleset, spells: &[&str]) -> BattleHero {
    let mut h = hero(r);
    h.power = 3;
    h.magic.spell_book = Some(spells.iter().map(|s| spell(r, s)).collect());
    h
}

fn led(
    r: &BattleRuleset,
    heroes: [Option<BattleHero>; 2],
    attacker: &[Spec],
    defender: &[Spec],
) -> BattleState {
    let setup = BattleSetup {
        heroes,
        ..setup(r, attacker, defender)
    };
    BattleState::new(r, &setup, 1).unwrap()
}

fn cast(r: &BattleRuleset, name: &str, target: u8) -> Command {
    Command::Cast {
        spell: spell(r, name),
        target: CastTarget::Stack(StackId(target)),
        second: CastTarget::None,
    }
}

/// The damage events of `events` as `(target, damage, kills)`.
fn spell_damage(events: &[Event]) -> Vec<(u8, u64, u32)> {
    events
        .iter()
        .filter_map(|e| match e.kind {
            EventKind::SpellDamage {
                target,
                damage,
                kills,
                ..
            } => Some((target.0, damage, kills)),
            _ => None,
        })
        .collect()
}

// covers: BAT-MAG-001, BAT-MAG-002, BAT-MAG-004
#[test]
fn a_hero_casts_once_per_round_and_its_stack_still_acts() {
    let r = ruleset();
    let mut s = led(
        &r,
        [Some(mage(&r, &["bolt"])), None],
        &[(0, "fixed", 10)],
        &[(0, "fixed", 10)],
    );
    let bolt = spell(&r, "bolt");
    // No mastery: 3 power × 10 + 10 = 40 damage, four of ten 10-hp creatures; 5 mana.
    let events = act(&mut s, &r, cast(&r, "bolt", 1)).unwrap();
    assert_eq!(
        events[0].kind,
        EventKind::SpellCast {
            side: BattleSide::Attacker,
            spell: bolt,
            target: CastTarget::Stack(StackId(1)),
            second: CastTarget::None,
        }
    );
    assert_eq!(spell_damage(&events), [(1, 40, 4)]);
    assert!(
        events
            .iter()
            .all(|e| e.visibility == crate::Visibility::Public)
    );
    assert_eq!(s.stacks()[1].count, 6);
    let view = s.hero(BattleSide::Attacker).unwrap();
    assert_eq!((view.mana, view.casts), (10, 1));
    // The stack keeps its turn; a second cast this round is refused, without changes.
    assert_eq!(s.active(), Some(StackId(0)));
    let before = s.clone();
    assert_eq!(
        act(&mut s, &r, cast(&r, "bolt", 1)),
        Err(CommandError::AlreadyCast)
    );
    assert_eq!(s, before);
    // A new round allows the next cast.
    act(&mut s, &r, Command::Defend).unwrap();
    act(&mut s, &r, Command::Defend).unwrap();
    assert_eq!(
        (s.round(), s.hero(BattleSide::Attacker).unwrap().casts),
        (2, 0)
    );
    act(&mut s, &r, cast(&r, "bolt", 1)).unwrap();
    assert_eq!(s.hero(BattleSide::Attacker).unwrap().mana, 5);
    // Never required: the stack acts without casting.
    act(&mut s, &r, Command::Defend).unwrap();
    // The defender has no hero: no casting.
    assert_eq!(
        act(&mut s, &r, cast(&r, "bolt", 0)),
        Err(CommandError::NotInSpellBook)
    );
}

// covers: BAT-MAG-002
#[test]
fn mastery_is_the_best_school_and_selects_cost_and_effect() {
    let r = ruleset();
    let mut h = mage(&r, &["bolt", "smite"]);
    // Air Basic, fire Expert: bolt (air) at Basic, smite (air and fire) at Expert.
    h.magic.masteries = masteries(&r, [1, 3, 0, 0]);
    let start = || {
        led(
            &r,
            [Some(h.clone()), None],
            &[(0, "fixed", 10)],
            &[(0, "tank", 10)],
        )
    };
    let mut s = start();
    let events = act(&mut s, &r, cast(&r, "smite", 1)).unwrap();
    assert_eq!(spell_damage(&events), [(1, 60, 0)], "3 × 10 + 30");
    assert_eq!(s.hero(BattleSide::Attacker).unwrap().mana, 11, "cost 4");
    let mut s = start();
    let events = act(&mut s, &r, cast(&r, "bolt", 1)).unwrap();
    assert_eq!(spell_damage(&events), [(1, 40, 0)], "3 × 10 + 10");
    // Not enough mana: nothing changes.
    let mut poor = h.clone();
    poor.magic.mana = 3;
    let mut s = led(
        &r,
        [Some(poor), None],
        &[(0, "fixed", 1)],
        &[(0, "tank", 1)],
    );
    let before = s.clone();
    assert_eq!(
        act(&mut s, &r, cast(&r, "bolt", 1)),
        Err(CommandError::NotEnoughMana)
    );
    assert_eq!(s, before);
    // Mana equal to the cost is enough (only less refuses): the cast is legal and leaves none.
    let mut exact = h.clone();
    exact.magic.mana = 4;
    let mut s = led(
        &r,
        [Some(exact), None],
        &[(0, "fixed", 1)],
        &[(0, "tank", 1)],
    );
    assert!(castable_spells(&s, &r, BattleSide::Attacker).contains(spell(&r, "bolt")));
    let events = act(&mut s, &r, cast(&r, "bolt", 1)).unwrap();
    assert_eq!(spell_damage(&events), [(1, 40, 0)]);
    let view = s.hero(BattleSide::Attacker).unwrap();
    assert_eq!((view.mana, view.casts), (0, 1));
}

// covers: BAT-MAG-001, BAT-MAG-003
#[test]
fn casts_need_a_book_a_combat_spell_and_a_valid_target() {
    let r = ruleset();
    let book = ["bolt", "smite", "ward", "portal", "nova"];
    let mut s = led(
        &r,
        [Some(mage(&r, &book)), None],
        &[(0, "fixed", 10), (1, "lich", 2)],
        &[(0, "fixed", 10), (1, "lich", 2)],
    );
    // Ids: 0 fixed, 1 lich (attacker); 2 fixed, 3 lich (defender).
    let at = |name: &str, target: CastTarget| Command::Cast {
        spell: spell(&r, name),
        target,
        second: CastTarget::None,
    };
    let stack = |id| CastTarget::Stack(StackId(id));
    for (command, error) in [
        (at("gaze", stack(2)), CommandError::NotInSpellBook),
        (at("portal", stack(2)), CommandError::NotACombatSpell),
        (at("ward", CastTarget::None), CommandError::NotCastable),
        // Nova needs a hex: a stack or nothing is no target (BAT-MAG-003).
        (at("nova", stack(2)), CommandError::InvalidSpellTarget),
        (
            at("nova", CastTarget::None),
            CommandError::InvalidSpellTarget,
        ),
        // Bolt reaches only enemies; smite any stack that is not undead.
        (at("bolt", stack(0)), CommandError::InvalidSpellTarget),
        (at("bolt", stack(9)), CommandError::InvalidSpellTarget),
        (
            at("bolt", CastTarget::None),
            CommandError::InvalidSpellTarget,
        ),
        (
            at("bolt", CastTarget::Hex(crate::Hex(100))),
            CommandError::InvalidSpellTarget,
        ),
        (at("smite", stack(3)), CommandError::InvalidSpellTarget),
    ] {
        let before = s.clone();
        assert_eq!(act(&mut s, &r, command), Err(error), "{command:?}");
        assert_eq!(s, before);
    }
    // Only the side to act casts.
    assert_eq!(
        crate::apply(
            &mut s,
            &r,
            BattleSide::Defender,
            cast(&r, "bolt", 0),
            &mut crate::NoopSink
        ),
        Err(CommandError::NotYourTurn)
    );
    // The lists: smite at every living stack but the liches, bolt at the enemy's.
    let mut targets = CastTargets::new();
    cast_targets(
        &s,
        &r,
        BattleSide::Attacker,
        spell(&r, "smite"),
        &mut targets,
    );
    assert_eq!(targets.as_slice(), [stack(0), stack(2)]);
    cast_targets(
        &s,
        &r,
        BattleSide::Attacker,
        spell(&r, "bolt"),
        &mut targets,
    );
    assert_eq!(targets.as_slice(), [stack(2), stack(3)]);
    cast_targets(
        &s,
        &r,
        BattleSide::Attacker,
        spell(&r, "nova"),
        &mut targets,
    );
    // Nova at every standable hex (BAT-MAG-003).
    let field = field(&r);
    let standable = (0u8..)
        .take(field.hex_count())
        .filter(|&h| field.is_standable(crate::Hex(h)));
    assert_eq!(targets.len(), standable.count());
    for t in targets.iter() {
        let CastTarget::Hex(h) = *t else {
            panic!("{t:?}")
        };
        assert!(field.is_standable(h));
    }
    cast_targets(
        &s,
        &r,
        BattleSide::Defender,
        spell(&r, "bolt"),
        &mut targets,
    );
    assert!(targets.is_empty());
    let castable: Vec<SpellId> = castable_spells(&s, &r, BattleSide::Attacker)
        .iter()
        .collect();
    assert_eq!(
        castable,
        [spell(&r, "bolt"), spell(&r, "smite"), spell(&r, "nova")]
    );
    assert!(castable_spells(&s, &r, BattleSide::Defender).is_empty());
    // Smite at an own stack; a dead stack is no target. Rally reaches only own stacks.
    let mut ally = led(
        &r,
        [Some(mage(&r, &["rally"])), None],
        &[(0, "fixed", 10), (1, "lich", 2)],
        &[(0, "fixed", 10)],
    );
    cast_targets(
        &ally,
        &r,
        BattleSide::Attacker,
        spell(&r, "rally"),
        &mut targets,
    );
    assert_eq!(targets.as_slice(), [stack(0), stack(1)]);
    assert_eq!(
        act(&mut ally, &r, cast(&r, "rally", 2)),
        Err(CommandError::InvalidSpellTarget)
    );
    act(&mut s, &r, at("smite", stack(0))).unwrap();
    s.stacks.as_mut_slice()[2].count = 0;
    s.heroes[0].as_mut().unwrap().view.casts = 0;
    assert_eq!(
        act(&mut s, &r, cast(&r, "bolt", 2)),
        Err(CommandError::InvalidSpellTarget)
    );
    // A book entry outside the ruleset (never from a validated setup) is refused.
    let unknown = SpellId::from_raw(100);
    s.heroes[0].as_mut().unwrap().magic.book.insert(unknown);
    assert_eq!(
        act(
            &mut s,
            &r,
            Command::Cast {
                spell: unknown,
                target: stack(3),
                second: CastTarget::None,
            }
        ),
        Err(CommandError::NotInSpellBook)
    );
}

// covers: BAT-MAG-001
#[test]
fn no_casting_in_the_tactics_phase_but_during_a_commanded_war_machines_turn() {
    let r = ruleset();
    let mut tactician = mage(&r, &["bolt"]);
    tactician.tactics = 2;
    let mut s = led(
        &r,
        [Some(tactician), None],
        &[(0, "fixed", 1)],
        &[(0, "tank", 1)],
    );
    assert!(s.tactics().is_some());
    assert_eq!(
        act(&mut s, &r, cast(&r, "bolt", 1)),
        Err(CommandError::InTacticsPhase)
    );
    let mut targets = CastTargets::new();
    cast_targets(
        &s,
        &r,
        BattleSide::Attacker,
        spell(&r, "bolt"),
        &mut targets,
    );
    assert!(targets.is_empty());
    // A Ballista its hero commands awaits a command; the hero may cast then (VCMI; batch 2a
    // X-3).
    let mut gunner = mage(&r, &["bolt"]);
    gunner.war_machines.ballista = Some(BallistaControl {
        shots: 1,
        double_damage_chance_bp: 0,
    });
    let setup = BattleSetup {
        heroes: [Some(gunner), None],
        war_machines: [vec![r.creature_by_key("t:ballista").unwrap()], vec![]],
        ..setup(&r, &[(0, "fixed", 1)], &[(0, "tank", 10)])
    };
    let mut s = BattleState::new(&r, &setup, 1).unwrap();
    while s.active() != Some(StackId(1)) {
        act(&mut s, &r, Command::Defend).unwrap();
    }
    assert_eq!(
        spell_damage(&act(&mut s, &r, cast(&r, "bolt", 2)).unwrap()).len(),
        1
    );
}

// covers: BAT-MAG-004, BAT-END-003
#[test]
fn a_spell_can_end_the_battle_and_the_outcome_lists_the_spells() {
    let r = ruleset();
    let mut s = led(
        &r,
        [
            Some(mage(&r, &["bolt", "smite"])),
            Some(mage(&r, &["bolt"])),
        ],
        &[(0, "fixed", 10)],
        &[(0, "fixed", 5)],
    );
    let events = act(&mut s, &r, cast(&r, "smite", 1)).unwrap();
    assert_eq!(spell_damage(&events), [(1, 40, 4)]);
    act(&mut s, &r, Command::Defend).unwrap();
    // The defender's hero casts on its stack's turn.
    let events = act(&mut s, &r, cast(&r, "bolt", 0)).unwrap();
    assert_eq!(spell_damage(&events), [(0, 40, 4)]);
    act(&mut s, &r, Command::Defend).unwrap();
    // Round 2: the attacker's bolt kills the last creature (40 ≥ 10 left).
    let events = act(&mut s, &r, cast(&r, "bolt", 1)).unwrap();
    assert_eq!(spell_damage(&events), [(1, 40, 1)]);
    assert_eq!(
        events.last().unwrap().kind,
        EventKind::BattleEnded {
            result: BattleResult::Victory(BattleSide::Attacker)
        }
    );
    assert!(s.is_over() && s.active().is_none());
    let outcome = s.outcome().unwrap();
    let [attacker, defender] = outcome.heroes.map(Option::unwrap);
    // Without Eagle Eye no hero picks a spell (BAT-MAG-069).
    assert_eq!((attacker.mana, attacker.learned), (5, vec![]));
    assert_eq!((defender.mana, defender.learned), (10, vec![]));
}

// covers: BAT-MAG-004
#[test]
fn the_casters_bonuses_apply_in_the_originals_order() {
    let r = ruleset();
    let bolt = spell(&r, "bolt");
    let mut h = mage(&r, &["bolt", "smite"]);
    // Advanced air: bolt 3 × 10 + 20 = 50 for 4 mana.
    h.magic.masteries = masteries(&r, [2, 0, 0, 0]);
    h.effects = vec![
        BattleEffect::SpellDamage {
            scope: SpellScope::All,
            rate: Rate::bp(1000),
        },
        BattleEffect::SpellDamage {
            scope: SpellScope::All,
            rate: Rate::bp(500),
        },
        BattleEffect::SpellDamage {
            scope: SpellScope::Spell(bolt),
            rate: Rate::bp(5000),
        },
        BattleEffect::SpellDamage {
            scope: SpellScope::School(SchoolId::from_raw(0)),
            rate: Rate::bp(5000),
        },
        BattleEffect::SpellDamage {
            scope: SpellScope::School(SchoolId::from_raw(1)),
            rate: Rate::bp(2500),
        },
        BattleEffect::SpellSpecialty {
            spell: bolt,
            bp_per_level: 300,
            level: 10,
        },
    ];
    let start = |enemy: Spec| led(&r, [Some(h.clone()), None], &[(0, "fast", 1)], &[enemy]);
    // Bolt on a level-7 titan: 50 × 1.5 (the air Orb) × 1.15 (Sorcery) = 86.25, kept fractional,
    // rounded down once to 86; then the spell's own +50 % of that (43) and the specialty's
    // ⌈3 % × (10 div 7) × 86⌉ = 3: 132.
    let mut s = start((0, "titan", 5));
    assert_eq!(
        spell_damage(&act(&mut s, &r, cast(&r, "bolt", 1)).unwrap()),
        [(1, 132, 0)]
    );
    assert_eq!(s.stacks()[1].top_hp, 168);
    // On a level-1 stack the specialty adds ⌈30 % × 86⌉ = 26: 155.
    let mut s = start((0, "tank", 5));
    assert_eq!(
        spell_damage(&act(&mut s, &r, cast(&r, "bolt", 1)).unwrap()),
        [(1, 155, 1)]
    );
    // Smite (air and fire, no bonus of its own): the better school, 86.
    let mut s = start((0, "titan", 5));
    assert_eq!(
        spell_damage(&act(&mut s, &r, cast(&r, "smite", 1)).unwrap()),
        [(1, 86, 0)]
    );
    // The single-precision rates of quirk Q-0005 lie slightly above the decimals: the same.
    let single = ruleset_with("t:single", |_| {});
    let mut s = led(
        &single,
        [Some(h.clone()), None],
        &[(0, "fast", 1)],
        &[(0, "titan", 5)],
    );
    assert_eq!(
        spell_damage(&act(&mut s, &single, cast(&single, "bolt", 1)).unwrap()),
        [(1, 132, 0)]
    );
    // A specialist below the creature's level adds nothing.
    let mut low = h.clone();
    low.effects = vec![BattleEffect::SpellSpecialty {
        spell: bolt,
        bp_per_level: 300,
        level: 6,
    }];
    let mut s = led(&r, [Some(low), None], &[(0, "fast", 1)], &[(0, "titan", 5)]);
    assert_eq!(
        spell_damage(&act(&mut s, &r, cast(&r, "bolt", 1)).unwrap()),
        [(1, 50, 0)]
    );
    // The owner sees its bonuses; both sides see the casts.
    let mut s = start((0, "titan", 5));
    act(&mut s, &r, cast(&r, "bolt", 1)).unwrap();
    let own = observe(&s, &r, BattleSide::Attacker).own_hero.unwrap();
    assert_eq!(own.spell_bonuses.all, Rate::bp(1500));
    assert_eq!(own.spell_bonuses.school, [5000, 2500, 0, 0].map(Rate::bp));
    assert_eq!(
        own.spell_bonuses.spells.as_slice(),
        [SpellBonus {
            spell: bolt,
            damage_bp: 5000,
            per_level_bp: 300,
            level: 10,
        }]
    );
    assert_eq!(own.masteries.get(bolt), 2);
    assert!(own.spell_book.contains(bolt));
    for viewer in BattleSide::BOTH {
        assert_eq!(observe(&s, &r, viewer).heroes[0].unwrap().casts, 1);
    }
}

// covers: BAT-MAG-004, HER-BAT-012
#[test]
fn a_grown_spell_bonus_counts_with_its_single_precision_value_with_quirk_q0005() {
    // OPEN-QUESTIONS Q45's example: bolt at value 200 (power 19, no mastery: 190 + 10) from a
    // hero whose spells gain 22.5 % (a Sorcery specialist's 15 % at level 10). Kept as
    // 0.2250000089… it deals 245; f32(0.225) = 0.2249999940… deals 245 too: the product
    // 244.9999988 is stored as an `f32`, which is 245.0, before it is rounded down (this pinned
    // 244, the exact product's floor: review r1 F-2); exact, 245.
    let damage = |profile: &str, rate: Rate| {
        let r = ruleset_with(profile, |_| {});
        let mut h = mage(&r, &["bolt"]);
        h.power = 19;
        h.effects = vec![BattleEffect::SpellDamage {
            scope: SpellScope::All,
            rate,
        }];
        let mut s = led(&r, [Some(h), None], &[(0, "fast", 1)], &[(0, "titan", 5)]);
        spell_damage(&act(&mut s, &r, cast(&r, "bolt", 1)).unwrap())[0].1
    };
    let grown = Rate {
        bp: 2250,
        single: core::num::NonZeroU32::new(0x3e66_6667),
    };
    assert_eq!(damage("t:single", grown), 245);
    assert_eq!(damage("t:single", Rate::bp(2250)), 245);
    assert_eq!(damage("t:faithful", grown), 245);
    // The rate's own single-precision value counts with the quirk, its basis points without:
    // one that is not its basis points' (0.25) deals 200 × 1.25 = 250.
    let other = Rate {
        bp: 2250,
        single: core::num::NonZeroU32::new(0x3e80_0000),
    };
    assert_eq!(damage("t:single", other), 250);
    assert_eq!(damage("t:faithful", other), 245);
}

// covers: BAT-MAG-004
#[test]
fn the_casters_products_are_stored_as_singles_with_quirk_q0005() {
    // The bolt of a hero with `power` whose spells gain `all` and whose air spells gain
    // `school`, in `profile`, on titans.
    let bolt = |profile: &str, power: u8, (school, all): (u32, u32)| {
        let r = ruleset_with(profile, |_| {});
        let mut h = mage(&r, &["bolt"]);
        h.power = power;
        h.effects = vec![
            BattleEffect::SpellDamage {
                scope: SpellScope::School(SchoolId::from_raw(0)),
                rate: Rate::bp(school),
            },
            BattleEffect::SpellDamage {
                scope: SpellScope::All,
                rate: Rate::bp(all),
            },
        ];
        let mut s = led(&r, [Some(h), None], &[(0, "fast", 1)], &[(0, "titan", 5)]);
        spell_damage(&act(&mut s, &r, cast(&r, "bolt", 1)).unwrap())[0].1
    };
    // Review r1 F-2 (a Sorcery specialist's 16 %, power 4: 50): f32(0.16) = 0.15999999642, so
    // the product is 57.99999982; the original stores it as an `f32`, 58.0, and rounds that down
    // (0x4E5760). The exact product's floor was 57.
    assert_eq!(bolt("t:single", 4, (0, 1600)), 58);
    assert_eq!(bolt("t:faithful", 4, (0, 1600)), 58);
    // Each product is stored: 560 × (1 + f32(0.39)) = 778.39999 is stored as 778.39996, and
    // that × 1.25 = 972.99995 as 972.99994: 972. (One store at the end would keep 973.0; the
    // exact rates give 973.)
    assert_eq!(bolt("t:single", 55, (3900, 2500)), 972);
    assert_eq!(bolt("t:faithful", 55, (3900, 2500)), 973);
    // The spell's own bonus joins the stored value, not its whole part, and the sum is stored
    // again: 4997 × (1 + f32(0.0002)) is stored as 4997.9995; with the bonus of its whole part,
    // 4997, the sum 9994.9995 is stored as 9995.0. The exact rates: 4997 + 4997.
    let caster = |profile: &str| {
        let r = ruleset_with(profile, |_| {});
        let id = spell(&r, "bolt");
        let mut bonuses = crate::magic::SpellBonuses {
            all: Rate::bp(2),
            ..Default::default()
        };
        let own = SpellBonus {
            spell: id,
            damage_bp: 10_000,
            ..SpellBonus::default()
        };
        bonuses.spells.push(own).unwrap();
        let titan = r.creature_by_key("t:titan").unwrap();
        let spell = (id, r.spell(id).unwrap());
        crate::magic::spell_damage(&r, (spell, Some(&bonuses)), 4997, (titan, &[100; 4])).0
    };
    assert_eq!(caster("t:single"), 9995);
    assert_eq!(caster("t:faithful"), 9994);
}

/// A battle of ten `fixed` each in `profile`; the attacker's hero has the bolt at power 255
/// (2 560) and gains `all` for every spell and `school` for the bolt's school.
fn bolt_caster(profile: &str, all: Rate, school: Rate) -> (BattleRuleset, BattleState) {
    let r = ruleset_with(profile, |_| {});
    let mut h = mage(&r, &["bolt"]);
    h.power = 255;
    h.magic.mana = 100;
    h.effects = vec![
        BattleEffect::SpellDamage {
            scope: SpellScope::All,
            rate: all,
        },
        BattleEffect::SpellDamage {
            scope: SpellScope::School(SchoolId::from_raw(0)),
            rate: school,
        },
    ];
    // The setup is sound for the battle (BAT-HER-001).
    let s = led(
        &r,
        [Some(h), None],
        &[(0, "fixed", 10)],
        &[(0, "fixed", 10)],
    );
    (r, s)
}

// The robustness review's tests for F-robust-3 (and r1 F-5): a hero the soundness check accepts
// (BAT-HER-001) with huge `SpellDamage` rates made `magic::spell_damage` overflow `u128` under
// quirk Q-0005 (the value times `(1 + school) × (1 + all)` over `2^40 × 2^40`), a panic.

// covers: BAT-MAG-004, BAT-HER-001
#[test]
fn huge_spell_damage_rates_do_not_overflow_the_spell_damage() {
    // Both rates are the largest basis points the seam carries.
    let max = Rate::bp(u32::MAX);
    let (r, mut s) = bolt_caster("t:single", max, max);
    // The cast is applied without a panic and kills the ten creatures. The damage follows the
    // executable's stores (the rate is stored as 429 496.71875; from exact rational arithmetic).
    let events = act(&mut s, &r, cast(&r, "bolt", 1)).unwrap();
    assert_eq!(spell_damage(&events), [(1, 472_238_801_289_216, 10)]);
    assert!(!s.stacks()[1].is_alive());
}

// covers: BAT-MAG-004, BAT-HER-001
#[test]
fn huge_spell_damage_rates_cast_with_exact_rates() {
    // The control: with exact rates (no quirk Q-0005) the same hero casts.
    let max = Rate::bp(u32::MAX);
    let (r, mut s) = bolt_caster("t:faithful", max, max);
    // 2 560 × (1 + 429 496.7295)², rounded down once.
    let events = act(&mut s, &r, cast(&r, "bolt", 1)).unwrap();
    assert_eq!(spell_damage(&events), [(1, 472_238_847_092_877, 10)]);
    assert!(!s.stacks()[1].is_alive());
}

// covers: BAT-MAG-004, BAT-HER-001
#[test]
fn spell_damage_rates_of_millions_of_percent_do_not_overflow() {
    // The smallest rates that overflowed at the bolt's 2 560: two of 33 160 000 % each,
    // (1 + 331 600)² × 2 560 > 2^48.
    let rate = Rate::bp(3_316_000_000);
    let (r, mut s) = bolt_caster("t:single", rate, rate);
    let events = act(&mut s, &r, cast(&r, "bolt", 1)).unwrap();
    assert_eq!(spell_damage(&events), [(1, 281_495_612_686_336, 10)]);
    assert!(!s.stacks()[1].is_alive());
}

// covers: BAT-MAG-004, BAT-HER-001
#[test]
fn saturated_single_precision_rates_do_not_overflow_the_factor() {
    // The other site: two single-precision values of their own that saturate the fixed point
    // (2^99 as an `f32`; any of 2^24 or more) overflowed the product of the two factors.
    let huge = Rate {
        bp: 0,
        single: core::num::NonZeroU32::new(0x7100_0000),
    };
    let (r, mut s) = bolt_caster("t:single", huge, huge);
    act(&mut s, &r, cast(&r, "bolt", 1)).unwrap();
    assert!(!s.stacks()[1].is_alive());
}

// covers: BAT-MAG-004
#[test]
fn the_spell_damage_saturates_for_every_value_and_rate() {
    // Every value and every rate the seam can carry, in both variants: the largest value with
    // the largest rates comes to the largest damage, and no bit pattern of a rate's own
    // single-precision value (the largest `f32`, an infinity, a NaN, a negative value, a
    // subnormal one) panics.
    let patterns = [
        0x7f7f_ffff,
        0x7f80_0000,
        0xffff_ffff,
        0x8000_0001,
        0x0000_0001,
    ];
    let rates = patterns
        .map(|bits| Rate {
            bp: u32::MAX,
            single: core::num::NonZeroU32::new(bits),
        })
        .into_iter()
        .chain([Rate::bp(u32::MAX), Rate::bp(0)]);
    let values = [0, 1, 4997, u64::from(u32::MAX), 1 << 48, u64::MAX];
    for profile in ["t:single", "t:faithful"] {
        let r = ruleset_with(profile, |_| {});
        let id = spell(&r, "bolt");
        let titan = r.creature_by_key("t:titan").unwrap();
        let spell = (id, r.spell(id).unwrap());
        let damage = |hero: Option<&crate::magic::SpellBonuses>, value: u64| {
            crate::magic::spell_damage(&r, (spell, hero), value, (titan, &[100; 4]))
        };
        for rate in rates.clone() {
            let mut bonuses = crate::magic::SpellBonuses {
                all: rate,
                school: [rate; 4],
                ..Default::default()
            };
            let own = SpellBonus {
                spell: id,
                damage_bp: u32::MAX,
                per_level_bp: u32::MAX,
                level: u8::MAX,
            };
            bonuses.spells.push(own).unwrap();
            for value in values {
                // The bonuses only add (every value here is one an `f32` holds or rounds up).
                let (caster, dealt) = damage(Some(&bonuses), value);
                assert!(
                    caster >= value && dealt == caster,
                    "{profile} {rate:?} {value}"
                );
            }
            let all = damage(Some(&bonuses), u64::MAX);
            assert_eq!(all, (u64::MAX, u64::MAX), "{profile} {rate:?}");
        }
        // The product alone: the largest value at the largest rates is beyond `u64` (with exact
        // rates beyond `u128` before the division), a small one is not.
        let rates = crate::magic::SpellBonuses {
            all: Rate::bp(u32::MAX),
            school: [Rate::bp(u32::MAX); 4],
            ..Default::default()
        };
        assert_eq!(damage(Some(&rates), u64::MAX).0, u64::MAX, "{profile}");
        assert_eq!(damage(Some(&rates), 1 << 40).0, u64::MAX, "{profile}");
        assert!(damage(Some(&rates), 1).0 < u64::MAX, "{profile}");
        // Without a hero the value is the damage, whatever it is: the original runs the caster's
        // stage on a hero only (0x5A78E0), so nothing is stored as an `f32`.
        let odd = (1 << 24) + 1;
        for value in values.into_iter().chain([odd, odd + 2]) {
            assert_eq!(damage(None, value).0, value, "{profile} {value}");
        }
        // A hero without bonuses: the same with exact rates; with the quirk the value passes
        // through an `f32`, which holds whole numbers up to 2^24 and rounds beyond (to nearest,
        // ties to even) and saturates.
        let plain = crate::magic::SpellBonuses::default();
        let single = profile == "t:single";
        for (value, stored) in [
            (0, 0),
            (1, 1),
            (4997, 4997),
            (1 << 24, 1 << 24),
            (odd, 1 << 24),
            (odd + 2, (1 << 24) + 4),
            (u64::from(u32::MAX), 1 << 32),
            (1 << 48, 1 << 48),
            (u64::MAX, u64::MAX),
        ] {
            let expected = if single { stored } else { value };
            assert_eq!(damage(Some(&plain), value).0, expected, "{profile} {value}");
        }
    }
}

// covers: BAT-HER-001
#[test]
fn at_most_four_spells_have_bonuses_of_their_own() {
    let r = ruleset();
    let names = ["bolt", "smite", "ward", "nova", "portal"];
    let mut h = mage(&r, &[]);
    h.effects = names
        .iter()
        .map(|n| BattleEffect::SpellSpecialty {
            spell: spell(&r, n),
            bp_per_level: 100,
            level: 1,
        })
        .collect();
    let setup = BattleSetup {
        heroes: [Some(h.clone()), None],
        ..setup(&r, &[(0, "fixed", 1)], &[(0, "fixed", 1)])
    };
    assert_eq!(
        BattleState::new(&r, &setup, 1),
        Err(SetupError::InvalidHero {
            side: BattleSide::Attacker,
            error: HeroError::TooManySpellBonuses
        })
    );
    // Four spells are fine, and a spell's damage and specialty bonuses share an entry.
    h.effects.pop();
    h.effects.push(BattleEffect::SpellDamage {
        scope: SpellScope::Spell(spell(&r, "bolt")),
        rate: Rate::bp(200),
    });
    let s = led(&r, [Some(h), None], &[(0, "fixed", 1)], &[(0, "fixed", 1)]);
    let own = observe(&s, &r, BattleSide::Attacker).own_hero.unwrap();
    assert_eq!(own.spell_bonuses.spells.len(), 4);
    assert_eq!(own.spell_bonuses.spells.as_slice()[0].damage_bp, 200);
}

#[test]
fn spell_sets_hold_up_to_128_spells() {
    let mut set = SpellSet::default();
    assert!(set.is_empty());
    for raw in [0, 63, 64, 127, 128, 500] {
        set.insert(SpellId::from_raw(raw));
    }
    assert!(!set.is_empty());
    let raws: Vec<u16> = set.iter().map(SpellId::raw).collect();
    assert_eq!(raws, [0, 63, 64, 127]);
    assert!(set.contains(SpellId::from_raw(127)) && !set.contains(SpellId::from_raw(128)));
    assert_eq!(set.bits(), [1 | (1 << 63), 1 | (1 << 63)]);
}
