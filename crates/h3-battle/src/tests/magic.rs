//! A hero's magic in battle (E04b T2b, T2d): BAT-MAG-001 to BAT-MAG-004.

use std::vec;
use std::vec::Vec;

use h3_content::{Ruleset, SchoolId, SpellId, SpellScope};
use h3_contracts::{
    BallistaControl, BattleEffect, BattleHero, BattleResult, BattleSetup, BattleSide,
};

use super::support::*;
use crate::{
    BattleState, CastTarget, CastTargets, Command, CommandError, Event, EventKind, HeroError,
    SetupError, SpellBonus, SpellSet, StackId, cast_targets, castable_spells, observe,
};

fn spell(r: &Ruleset, name: &str) -> SpellId {
    r.spell_by_key(&std::format!("t:{name}")).unwrap()
}

/// A hero with spell power 3, 15 mana and `spells` in its book.
fn mage(r: &Ruleset, spells: &[&str]) -> BattleHero {
    let mut h = hero(r);
    h.power = 3;
    h.magic.spell_book = Some(spells.iter().map(|s| spell(r, s)).collect());
    h
}

fn led(
    r: &Ruleset,
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

fn cast(r: &Ruleset, name: &str, target: u8) -> Command {
    Command::Cast {
        spell: spell(r, name),
        target: CastTarget::Stack(StackId(target)),
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
    h.magic.schools = [1, 3, 0, 0];
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
    // Nova at every standable hex whose area (radius 1) holds a stack it can reach.
    let field = field(&r);
    assert!(!targets.is_empty());
    for t in targets.iter() {
        let CastTarget::Hex(h) = *t else {
            panic!("{t:?}")
        };
        assert!(field.is_standable(h));
        assert!(s.stacks().iter().any(|x| field.distance(x.hex, h) <= 1));
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
                target: stack(3)
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
    assert_eq!(attacker.mana, 5);
    assert_eq!(
        attacker.spells_cast,
        [spell(&r, "bolt"), spell(&r, "smite")]
    );
    assert_eq!(
        (defender.mana, defender.spells_cast),
        (10, vec![spell(&r, "bolt")])
    );
}

// covers: BAT-MAG-004
#[test]
fn the_casters_bonuses_apply_in_the_originals_order() {
    let r = ruleset();
    let bolt = spell(&r, "bolt");
    let mut h = mage(&r, &["bolt", "smite"]);
    // Advanced air: bolt 3 × 10 + 20 = 50 for 4 mana.
    h.magic.schools = [2, 0, 0, 0];
    h.effects = vec![
        BattleEffect::SpellDamage {
            scope: SpellScope::All,
            bp: 1000,
        },
        BattleEffect::SpellDamage {
            scope: SpellScope::All,
            bp: 500,
        },
        BattleEffect::SpellDamage {
            scope: SpellScope::Spell(bolt),
            bp: 5000,
        },
        BattleEffect::SpellDamage {
            scope: SpellScope::School(SchoolId::from_raw(0)),
            bp: 5000,
        },
        BattleEffect::SpellDamage {
            scope: SpellScope::School(SchoolId::from_raw(1)),
            bp: 2500,
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
    assert_eq!(own.spell_bonuses.all_bp, 1500);
    assert_eq!(own.spell_bonuses.school_bp, [5000, 2500, 0, 0]);
    assert_eq!(
        own.spell_bonuses.spells.as_slice(),
        [SpellBonus {
            spell: bolt,
            damage_bp: 5000,
            per_level_bp: 300,
            level: 10,
        }]
    );
    assert_eq!(own.schools, [2, 0, 0, 0]);
    assert!(own.spell_book.contains(bolt));
    for viewer in BattleSide::BOTH {
        assert_eq!(observe(&s, &r, viewer).heroes[0].unwrap().casts, 1);
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
        bp: 200,
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
