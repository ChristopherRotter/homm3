//! The heroes leading the sides (BAT-HER-001): what the battle checks, keeps and shows, the
//! hero's attack and defense on its stacks (BAT-HER-002), its morale and luck effects
//! (BAT-HER-003), its damage bonuses (BAT-HER-004), its statistic effects (BAT-HER-005), its
//! trait effects (BAT-HER-006) and its alignment groups (BAT-HER-007).

use std::vec;
use std::vec::Vec;

use h3_content::{
    Affects, CreatureFlag, CreatureSelector, CreatureStat, CreatureTypeId, HeroTypeId, SchoolId,
    SpellId, SpellScope,
};
use h3_contracts::{
    BallistaControl, BattleEffect, BattleHero, DamageBonuses, MAX_BATTLE_EFFECTS, StatOp,
    TentControl,
};

use super::support::*;
use crate::{
    BattleSetup, BattleSide, BattleState, Command, EventKind, HeroError, HeroView, OwnHeroView,
    SetupError, StackId, observe,
};

fn with_heroes(r: &h3_content::Ruleset, heroes: [Option<BattleHero>; 2]) -> BattleSetup {
    BattleSetup {
        heroes,
        ..setup(r, &[(0, "fixed", 10)], &[(0, "fixed", 10)])
    }
}

/// Every effect variant, the selectors by creature type, flag and all.
fn every_effect(r: &h3_content::Ruleset) -> Vec<BattleEffect> {
    let fixed = CreatureSelector::Creature(r.creature_by_key("t:fixed").unwrap());
    vec![
        BattleEffect::Morale {
            amount: 1,
            affects: Affects::Own,
        },
        BattleEffect::Luck {
            amount: -2,
            affects: Affects::Enemy,
        },
        BattleEffect::MoraleAtMost {
            value: 0,
            affects: Affects::Both,
        },
        BattleEffect::LuckAtMost {
            value: 0,
            affects: Affects::Enemy,
        },
        BattleEffect::StackStat {
            creatures: fixed,
            stat: CreatureStat::Speed,
            op: StatOp::Add,
            amount: 1,
        },
        BattleEffect::StackStat {
            creatures: CreatureSelector::All,
            stat: CreatureStat::HitPoints,
            op: StatOp::PercentBp,
            amount: 2500,
        },
        BattleEffect::Regeneration {
            creatures: CreatureSelector::Flag(CreatureFlag::Alive),
            hit_points: 50,
        },
        BattleEffect::NoRangePenalty { creatures: fixed },
        BattleEffect::ShootWhenAdjacent {
            creatures: CreatureSelector::Flag(CreatureFlag::Shooter),
        },
        BattleEffect::NoRetreat,
        BattleEffect::AlignmentGroup {
            factions: h3_contracts::FactionGroup::new(&[
                h3_content::FactionId::from_raw(0),
                h3_content::FactionId::from_raw(1),
            ])
            .unwrap(),
        },
        BattleEffect::SpellDamage {
            scope: SpellScope::All,
            rate: Rate::bp(1500),
        },
        BattleEffect::SpellDamage {
            scope: SpellScope::School(SchoolId::from_raw(0)),
            rate: Rate::bp(5000),
        },
        BattleEffect::SpellDamage {
            scope: SpellScope::Spell(r.spell_by_key("t:bolt").unwrap()),
            rate: Rate::bp(5000),
        },
        BattleEffect::SpellSpecialty {
            spell: r.spell_by_key("t:bolt").unwrap(),
            bp_per_level: 300,
            level: 20,
        },
        BattleEffect::MagicResistance {
            rate: Rate::bp(1000),
        },
    ]
}

// covers: BAT-HER-001
#[test]
fn heroes_are_kept_and_shown_to_both_sides() {
    let r = ruleset();
    let view = HeroView {
        hero_type: r.hero_type_by_key("t:orrin").unwrap(),
        attack: 3,
        defense: 2,
        power: 1,
        knowledge: 1,
        mana: 15,
        max_mana: 10,
        casts: 0,
        commands_ballista: false,
        commands_tent: false,
        spell_book: false,
    };
    let s = BattleState::new(&r, &with_heroes(&r, [Some(hero(&r)), None]), 1).unwrap();
    assert_eq!(s.hero(BattleSide::Attacker), Some(view));
    assert_eq!(s.hero(BattleSide::Defender), None);
    for viewer in BattleSide::BOTH {
        assert_eq!(observe(&s, &r, viewer).heroes, [Some(view), None]);
    }
    // The hero and its private Tactics level are part of the state hash.
    let bare = BattleState::new(&r, &with_heroes(&r, [None, None]), 1).unwrap();
    assert_ne!(s.state_hash(), bare.state_hash());
    // Equal levels cancel (no tactics phase, BAT-TAC-001), so only the levels differ.
    let mut tactician = hero(&r);
    tactician.tactics = 3;
    let both = BattleState::new(&r, &with_heroes(&r, [Some(hero(&r)), Some(hero(&r))]), 1).unwrap();
    let other = BattleState::new(
        &r,
        &with_heroes(&r, [Some(tactician.clone()), Some(tactician)]),
        1,
    )
    .unwrap();
    assert_eq!((both.tactics(), other.tactics()), (None, None));
    assert_ne!(both.state_hash(), other.state_hash());
}

// covers: BAT-HER-001, BAT-FLE-002
#[test]
fn the_private_part_of_a_hero_shows_only_to_its_side() {
    let r = ruleset();
    let damage = DamageBonuses {
        ranged: Rate::bp(1000),
        melee: Rate::bp(2000),
        reduction: Rate::bp(500),
    };
    let mut commander = hero(&r);
    commander.tactics = 2;
    commander.damage = damage;
    let ballista = BallistaControl {
        shots: 2,
        double_damage_chance_bp: 7500,
    };
    let tent = TentControl {
        heal_bonus: Rate::bp(10_000),
    };
    commander.war_machines.ballista = Some(ballista);
    commander.war_machines.tent = Some(tent);
    commander.surrender_discount_bp = 2000;
    let setup = BattleSetup {
        funds: [5000, 7],
        ..with_heroes(&r, [Some(commander), Some(hero(&r))])
    };
    let s = BattleState::new(&r, &setup, 1).unwrap();
    // Both sides see which war machines a hero commands, and that both may surrender.
    let [attacker, defender] = BattleSide::BOTH.map(|viewer| observe(&s, &r, viewer));
    assert_eq!(attacker.heroes, defender.heroes);
    let public = attacker
        .heroes
        .map(|h| h.map(|h| (h.commands_ballista, h.commands_tent)));
    assert_eq!(public, [Some((true, true)), Some((false, false))]);
    assert_eq!(attacker.surrender_possible, [true, true]);
    assert_eq!(defender.surrender_possible, [true, true]);
    // Each side sees its own private part with its price: half of 10 fixed at 60 gold, less the
    // attacker's 20 % discount (BAT-FLE-002).
    assert_eq!(
        attacker.own_hero,
        Some(OwnHeroView {
            tactics: 2,
            damage,
            ballista: Some(ballista),
            tent: Some(tent),
            surrender_discount_bp: 2000,
            funds: 5000,
            surrender_price: Some(240),
            spell_book: crate::SpellSet::default(),
            masteries: crate::Masteries::default(),
            spell_bonuses: crate::SpellBonuses::default(),
        })
    );
    let own = defender.own_hero.unwrap();
    assert_eq!((own.tactics, own.ballista, own.tent), (0, None, None));
    assert_eq!((own.surrender_discount_bp, own.funds), (0, 7));
    assert_eq!(own.surrender_price, Some(300));
    // A side without a hero has no private part.
    let lone = BattleState::new(&r, &with_heroes(&r, [Some(hero(&r)), None]), 1).unwrap();
    assert!(observe(&lone, &r, BattleSide::Attacker).own_hero.is_some());
    assert_eq!(observe(&lone, &r, BattleSide::Defender).own_hero, None);
}

// covers: BAT-HER-001
#[test]
fn values_no_real_hero_could_have_are_accepted() {
    let r = ruleset();
    let mut odd = hero(&r);
    [odd.attack, odd.defense, odd.power, odd.knowledge] = [255, 200, 0, 0];
    odd.tactics = 3;
    odd.damage.ranged = Rate::bp(u16::MAX.into());
    odd.damage.melee = Rate::bp(u16::MAX.into());
    odd.damage.reduction = Rate::bp(10_000);
    odd.surrender_discount_bp = 10_000;
    odd.war_machines.ballista = Some(BallistaControl {
        shots: 255,
        double_damage_chance_bp: 10_000,
    });
    odd.effects = every_effect(&r);
    odd.effects
        .resize(MAX_BATTLE_EFFECTS, BattleEffect::NoRetreat);
    odd.magic.spell_book = Some(vec![r.spell_by_key("t:bolt").unwrap()]);
    odd.magic.mana = u16::MAX;
    odd.magic.max_mana = 0;
    odd.magic.masteries = masteries(&r, [3, 0, 0, 0]);
    let s = BattleState::new(&r, &with_heroes(&r, [Some(odd.clone()), Some(odd)]), 1).unwrap();
    assert_eq!(s.hero(BattleSide::Defender).unwrap().attack, 255);
}

/// A change to a valid hero and the error it causes.
type Case = (fn(&h3_content::Ruleset, &mut BattleHero), HeroError);

// covers: BAT-HER-001
#[test]
fn unsound_heroes_are_rejected() {
    let cases: Vec<Case> = vec![
        (
            |_, h| h.hero_type = HeroTypeId::from_raw(99),
            HeroError::UnknownHeroType,
        ),
        (|_, h| h.tactics = 4, HeroError::TacticsLevel),
        (
            |_, h| h.damage.reduction = Rate::bp(10_001),
            HeroError::DamageReduction,
        ),
        (
            |_, h| {
                h.damage.reduction = Rate {
                    bp: 10_000,
                    single: core::num::NonZeroU32::new(0x3f80_0001),
                };
            },
            HeroError::DamageReduction,
        ),
        (
            |_, h| {
                h.war_machines.ballista = Some(BallistaControl {
                    shots: 0,
                    double_damage_chance_bp: 0,
                });
            },
            HeroError::BallistaControl,
        ),
        (
            |_, h| {
                h.war_machines.ballista = Some(BallistaControl {
                    shots: 1,
                    double_damage_chance_bp: 10_001,
                });
            },
            HeroError::BallistaControl,
        ),
        (
            |_, h| h.surrender_discount_bp = 10_001,
            HeroError::SurrenderDiscount,
        ),
        (
            |_, h| h.effects = vec![BattleEffect::NoRetreat; MAX_BATTLE_EFFECTS + 1],
            HeroError::TooManyEffects,
        ),
        (
            |r, h| {
                h.effects = every_effect(r);
                h.effects.push(BattleEffect::NoRangePenalty {
                    creatures: CreatureSelector::Creature(CreatureTypeId::from_raw(999)),
                });
            },
            HeroError::UnknownCreature,
        ),
        (
            |_, h| {
                h.effects = vec![BattleEffect::AlignmentGroup {
                    factions: h3_contracts::FactionGroup::new(&[
                        h3_content::FactionId::from_raw(0),
                        h3_content::FactionId::from_raw(99),
                    ])
                    .unwrap(),
                }];
            },
            HeroError::UnknownFaction,
        ),
        (
            |r, h| h.magic.masteries = vec![(r.spell_by_key("t:bolt").unwrap(), 4)],
            HeroError::MasteryLevel,
        ),
        (
            |r, h| h.magic.masteries = vec![(r.spell_by_key("t:bolt").unwrap(), 0)],
            HeroError::MasteryLevel,
        ),
        (
            |_, h| h.magic.masteries = vec![(SpellId::from_raw(999), 1)],
            HeroError::UnknownSpell,
        ),
        (
            |r, h| h.magic.masteries = vec![(r.spell_by_key("t:bolt").unwrap(), 1); 2],
            HeroError::DuplicateSpell,
        ),
        (
            |_, h| {
                h.effects = vec![BattleEffect::SpellDamage {
                    scope: SpellScope::School(SchoolId::from_raw(2)),
                    rate: Rate::bp(1),
                }];
            },
            HeroError::UnknownSchool,
        ),
        (
            |_, h| {
                h.effects = vec![BattleEffect::SpellDamage {
                    scope: SpellScope::Spell(SpellId::from_raw(99)),
                    rate: Rate::bp(1),
                }];
            },
            HeroError::UnknownSpell,
        ),
        (
            |_, h| {
                h.effects = vec![BattleEffect::SpellSpecialty {
                    spell: SpellId::from_raw(99),
                    bp_per_level: 1,
                    level: 1,
                }];
            },
            HeroError::UnknownSpell,
        ),
        (
            |_, h| h.magic.spell_book = Some(vec![SpellId::from_raw(99)]),
            HeroError::UnknownSpell,
        ),
        (
            |r, h| {
                let bolt = r.spell_by_key("t:bolt").unwrap();
                h.magic.spell_book = Some(vec![bolt, bolt]);
            },
            HeroError::DuplicateSpell,
        ),
        (
            |r, h| h.magic.spell_book = Some(vec![r.spell_by_key("t:gaze").unwrap()]),
            HeroError::NotAHeroSpell,
        ),
    ];
    let r = ruleset();
    for (change, error) in cases {
        for side in BattleSide::BOTH {
            let mut bad = hero(&r);
            change(&r, &mut bad);
            let mut heroes = [Some(hero(&r)), Some(hero(&r))];
            heroes[side.index()] = Some(bad);
            assert_eq!(
                BattleState::new(&r, &with_heroes(&r, heroes), 1),
                Err(SetupError::InvalidHero { side, error })
            );
        }
    }
}

// covers: BAT-HER-005
#[test]
fn statistic_effects_change_the_selected_stacks_of_the_side_at_setup() {
    use CreatureSelector::{All, Flag};
    use CreatureStat::{Attack, Defense, HitPoints, MaxDamage, MinDamage, Speed};
    use StatOp::{Add, PercentBp, PercentBpDown};
    let r = ruleset();
    let id = |k: &str| r.creature_by_key(k).unwrap();
    let squires = CreatureSelector::Creature(id("t:squire"));
    let stat = |creatures, stat, op, amount| BattleEffect::StackStat {
        creatures,
        stat,
        op,
        amount,
    };
    let alive = Flag(CreatureFlag::Alive);
    let mut attacker = hero(&r);
    [attacker.attack, attacker.defense] = [0, 0];
    attacker.effects = vec![
        // Elixir of Life: +25 % of a living creature's own hit points, rounded down, and its
        // parts' +4 for every creature, war machines included.
        stat(alive, HitPoints, PercentBpDown, 2500),
        stat(All, HitPoints, Add, 4),
        // +2 speed for every creature, but not for war machines.
        stat(All, Speed, Add, 2),
        // One step of a squire specialty: +5 % attack and defense of the creature's own value,
        // rounded up, for squires and their upgrade.
        stat(squires, Attack, PercentBp, 500),
        stat(squires, Defense, PercentBp, 500),
        // Speed and damage shares: +20 % speed rounded up (5 → +1, 6 → +2), +50 % maximum
        // damage rounded down (9 → +4, 10 → +5) on top of the additions.
        stat(squires, Speed, PercentBp, 2000),
        stat(squires, MinDamage, Add, 5),
        stat(squires, MaxDamage, Add, -1),
        stat(squires, MaxDamage, PercentBpDown, 5000),
        BattleEffect::Morale {
            amount: 1,
            affects: Affects::Own,
        },
    ];
    let mut defender = hero(&r);
    [defender.attack, defender.defense] = [0, 0];
    defender.effects = vec![
        // Negative shares: −0.5 rounded down to −1 attack, −0.6 rounded up to 0 defense.
        stat(alive, Attack, PercentBpDown, -500),
        stat(alive, Defense, PercentBp, -500),
        // Hit points stay at least 1, speed and damage at least 0; a minimum damage above the
        // maximum is lowered to it.
        stat(All, HitPoints, Add, i16::MIN),
        stat(All, Speed, Add, -100),
        stat(All, MaxDamage, Add, -100),
        stat(alive, MinDamage, PercentBp, -5000),
    ];
    let s = BattleSetup {
        heroes: [Some(attacker), None],
        war_machines: [vec![id("t:ballista"), id("t:cart"), id("t:tent")], vec![]],
        ..setup(
            &r,
            &[(0, "squire", 1), (1, "paladin", 1), (2, "fixed", 1)],
            &[(0, "squire", 1)],
        )
    };
    // (attack, defense, damage, hit points, top hit points, speed)
    let stats = |s: &BattleSetup| -> Vec<(u16, u16, [u16; 2], u32, u32, u8)> {
        let b = BattleState::new(&r, s, 1).unwrap();
        b.stacks()
            .iter()
            .map(|s| {
                let damage = [s.damage_min, s.damage_max];
                (s.attack, s.defense, damage, s.max_hp, s.top_hp, s.speed)
            })
            .collect()
    };
    // Stacks: the attacker's squire, paladin, fixed, Ballista, cart and tent, the defender's squire.
    assert_eq!(
        stats(&s),
        [
            (11, 13, [11, 12], 35 + 8 + 4, 47, 8),
            (13, 13, [12, 14], 35 + 8 + 4, 47, 10),
            (10, 10, [2, 2], 14, 14, 7),
            (10, 10, [2, 3], 254, 254, 0),
            (0, 5, [0, 0], 104, 104, 0),
            (0, 0, [0, 0], 79, 79, 0),
            (10, 12, [6, 9], 35, 35, 5),
        ]
    );
    let s = BattleSetup {
        heroes: [None, Some(defender)],
        ..s
    };
    assert_eq!(stats(&s).get(6..), Some(&[(9, 12, [0, 0], 1, 1, 0)][..]));
}

// covers: BAT-HER-006
#[test]
fn trait_effects_reach_the_selected_stacks_of_the_side() {
    use crate::StackTraits;
    let r = ruleset();
    let id = |k: &str| r.creature_by_key(k).unwrap();
    let shooters = CreatureSelector::Flag(CreatureFlag::Shooter);
    let mut h = hero(&r);
    h.effects = vec![
        BattleEffect::Morale {
            amount: 1,
            affects: Affects::Own,
        },
        BattleEffect::Regeneration {
            creatures: CreatureSelector::Flag(CreatureFlag::Alive),
            hit_points: 50,
        },
        BattleEffect::Regeneration {
            creatures: CreatureSelector::Creature(id("t:squire")),
            hit_points: 70,
        },
        BattleEffect::NoRangePenalty {
            creatures: shooters,
        },
        BattleEffect::ShootWhenAdjacent {
            creatures: shooters,
        },
    ];
    let s = BattleSetup {
        heroes: [Some(h), None],
        war_machines: [vec![id("t:ballista")], vec![]],
        ..setup(
            &r,
            &[
                (0, "archer", 10),
                (1, "squire", 1),
                (2, "paladin", 1),
                (3, "fixed", 1),
            ],
            &[(0, "archer", 10)],
        )
    };
    let b = BattleState::new(&r, &s, 1).unwrap();
    let both = |s: &crate::Stack| {
        [
            StackTraits::NO_DISTANCE_PENALTY,
            StackTraits::SHOOTS_WHEN_ADJACENT,
        ]
        .map(|t| s.has(t))
    };
    // Stacks: archer, squire, paladin, fixed, Ballista; the defender's archer. The squires' 70
    // beats the living creatures' 50; the archer is not alive; the Ballista is a shooter.
    assert_eq!(
        b.stacks()
            .iter()
            .map(|s| (both(s), s.regeneration))
            .collect::<Vec<_>>(),
        [
            ([true, true], 0),
            ([false, false], 70),
            ([false, false], 70),
            ([false, false], 0),
            ([true, true], 0),
            ([false, false], 0),
        ]
    );
}

// covers: BAT-HER-006, BAT-TRN-005, BAT-RNG-001, BAT-RNG-002
#[test]
fn hero_traits_let_shooters_shoot_far_and_close_and_stacks_regenerate() {
    let r = ruleset();
    let shooters = CreatureSelector::Flag(CreatureFlag::Shooter);
    let mut bow = hero(&r);
    [bow.attack, bow.defense] = [0, 0];
    bow.effects = vec![
        BattleEffect::NoRangePenalty {
            creatures: shooters,
        },
        BattleEffect::ShootWhenAdjacent {
            creatures: shooters,
        },
    ];
    let mut s = BattleState::new(
        &r,
        &BattleSetup {
            heroes: [Some(bow), None],
            ..setup(
                &r,
                &[(0, "archer", 10)],
                &[(0, "fixed", 10), (1, "slow", 1)],
            )
        },
        1,
    )
    .unwrap();
    activate(&mut s, 0);
    for st in &mut s.stacks.as_mut_slice()[..1] {
        [st.damage_min, st.damage_max] = [10, 10];
    }
    // archer attack 6 against fixed defense 10: 90 at any distance (half beyond ten hexes
    // without the hero, BAT-RNG-002).
    let damage = |s: &BattleState, x: i32| {
        let mut t = s.clone();
        place(&mut t, &r, 1, x, 5);
        let shot = Command::Shoot {
            target: StackId(1),
            at: t.stacks()[1].hex,
        };
        act(&mut t, &r, shot)
            .unwrap()
            .iter()
            .find_map(|e| match e.kind {
                EventKind::Struck { damage, .. } => Some(damage),
                _ => None,
            })
    };
    assert_eq!(damage(&s, 15), Some(90));
    // Next to an enemy it still shoots (BAT-RNG-001 would block it).
    place(&mut s, &r, 2, 2, 5);
    assert_eq!(damage(&s, 9), Some(90));
    // A hero's regeneration restores at most its hit points per turn; a creature's own
    // regeneration everything (BAT-TRN-005). The squire has 135 hit points, 10 left.
    let mut tough = hero(&r);
    tough.effects = vec![
        BattleEffect::Regeneration {
            creatures: CreatureSelector::All,
            hit_points: 50,
        },
        BattleEffect::StackStat {
            creatures: CreatureSelector::All,
            stat: CreatureStat::HitPoints,
            op: StatOp::Add,
            amount: 100,
        },
    ];
    let mut s = BattleState::new(
        &r,
        &BattleSetup {
            heroes: [Some(tough.clone()), Some(tough)],
            ..setup(&r, &[(0, "squire", 1)], &[(0, "wight", 5)])
        },
        1,
    )
    .unwrap();
    s.stacks.as_mut_slice()[0].top_hp = 10;
    s.stacks.as_mut_slice()[1].top_hp = 3;
    let regenerated = |events: &[crate::Event]| -> Vec<(u8, u32)> {
        events
            .iter()
            .filter_map(|e| match e.kind {
                EventKind::Regenerated { stack, hit_points } => Some((stack.0, hit_points)),
                _ => None,
            })
            .collect()
    };
    // The squire acts first, then the wight (110 hit points) regains all 107.
    assert_eq!(
        regenerated(&act(&mut s, &r, Command::Defend).unwrap()),
        [(1, 107)]
    );
    assert_eq!(
        regenerated(&act(&mut s, &r, Command::Defend).unwrap()),
        [(0, 50)]
    );
    assert_eq!(s.stacks()[0].top_hp, 60);
}

// covers: BAT-HER-007, BAT-MOR-002
#[test]
fn alignment_groups_count_their_factions_as_one_alignment() {
    let r = ruleset();
    let faction = |k: &str| {
        let i = r
            .factions()
            .iter()
            .position(|f| f.key.as_str() == k)
            .unwrap();
        h3_content::FactionId::from_index(i).unwrap()
    };
    let group = |keys: &[&str]| BattleEffect::AlignmentGroup {
        factions: h3_contracts::FactionGroup::new(
            &keys.iter().map(|k| faction(k)).collect::<Vec<_>>(),
        )
        .unwrap(),
    };
    // Castle, inferno and neutral creatures: three alignments, −1 for every stack of each side.
    let army = [(0, "pikeman_c", 1), (1, "imp", 1), (2, "serf", 1)];
    let morale = |effects: Vec<BattleEffect>| {
        let mut h = hero(&r);
        h.effects = effects;
        let s = BattleSetup {
            heroes: [Some(h), None],
            ..setup(&r, &army, &army)
        };
        let b = BattleState::new(&r, &s, 1).unwrap();
        b.stacks().iter().map(|s| s.morale).collect::<Vec<_>>()
    };
    assert_eq!(morale(vec![]), [-1; 6]);
    // Castle and inferno as one: two alignments (0) for the hero's side only.
    assert_eq!(
        morale(vec![group(&["t:castle", "t:inferno"])]),
        [0, 0, 0, -1, -1, -1]
    );
    // Groups that share a faction join, even one the army lacks.
    assert_eq!(
        morale(vec![
            group(&["t:castle", "t:necropolis"]),
            group(&["t:necropolis", "t:inferno"]),
        ]),
        [0, 0, 0, -1, -1, -1]
    );
    // A group without the castle leaves three alignments.
    assert_eq!(
        morale(vec![
            BattleEffect::NoRetreat,
            group(&["t:inferno", "t:necropolis"])
        ]),
        [-1; 6]
    );
}

// covers: BAT-HER-002
#[test]
fn a_hero_adds_its_attack_and_defense_to_every_stack_of_its_side() {
    let r = ruleset();
    let ballista = r.creature_by_key("t:ballista").unwrap();
    let stats = |heroes: [Option<BattleHero>; 2]| -> Vec<(u16, u16)> {
        let s = BattleSetup {
            heroes,
            terrain: r.terrain_by_key("t:plain"),
            war_machines: [vec![ballista], vec![ballista]],
            ..setup(
                &r,
                &[(0, "pikeman_c", 1), (1, "serf", 1)],
                &[(0, "serf", 1)],
            )
        };
        let b = BattleState::new(&r, &s, 1).unwrap();
        b.stacks().iter().map(|s| (s.attack, s.defense)).collect()
    };
    // Stacks: the attacker's pikeman, serf and Ballista, then the defender's serf and Ballista.
    let bare = stats([None, None]);
    assert_eq!(bare[..2], [(11, 11), (10, 10)]);
    // Orrin (attack 3, defense 2) leads the attacker: its army and its war machine gain, on top
    // of the native-terrain bonus; the defender's stacks keep their values.
    let led = stats([Some(hero(&r)), None]);
    let gained: Vec<(u16, u16)> = bare[..3].iter().map(|&(a, d)| (a + 3, d + 2)).collect();
    assert_eq!(led[..3], gained[..]);
    assert_eq!(led[3..], bare[3..]);
    // Values no real hero could have are added as they are.
    let mut odd = hero(&r);
    [odd.attack, odd.defense] = [255, 255];
    assert_eq!(stats([None, Some(odd)])[3], (265, 265));

    // The attack advantage reads the sums (BAT-DMG-002): 10 fixed (2 damage each) strike 10
    // fixed for 20 without heroes, for 23 with Orrin's attack 3 (+15 %), for 21 when the
    // defender's Orrin adds defense 2 (+5 %).
    let damage = |heroes: [Option<BattleHero>; 2]| {
        let s = BattleSetup {
            heroes,
            ..setup(&r, &[(0, "fixed", 10)], &[(0, "fixed", 10)])
        };
        let mut b = BattleState::new(&r, &s, 1).unwrap();
        place(&mut b, &r, 0, 7, 5);
        place(&mut b, &r, 1, 8, 5);
        activate(&mut b, 0);
        let attack = Command::Attack {
            target: StackId(1),
            from: at(&r, 7, 5),
        };
        act(&mut b, &r, attack)
            .unwrap()
            .iter()
            .find_map(|e| match e.kind {
                EventKind::Struck { damage, .. } => Some(damage),
                _ => None,
            })
            .unwrap()
    };
    assert_eq!(damage([None, None]), 20);
    assert_eq!(damage([Some(hero(&r)), None]), 23);
    assert_eq!(damage([Some(hero(&r)), Some(hero(&r))]), 21);
}

// covers: BAT-HER-003
#[test]
fn hero_morale_and_luck_effects_reach_the_sides_they_name() {
    use Affects::{Both, Enemy, Own};
    use BattleEffect::{Luck, LuckAtMost, Morale, MoraleAtMost};
    let r = ruleset();
    let start = |attacker: Vec<BattleEffect>, defender: Vec<BattleEffect>| {
        let led = |effects| {
            Some(BattleHero {
                effects,
                ..hero(&r)
            })
        };
        let s = BattleSetup {
            heroes: [led(attacker), led(defender)],
            ..setup(&r, &[(0, "fixed", 1)], &[(0, "minotaur", 1)])
        };
        let b = BattleState::new(&r, &s, 1).unwrap();
        let values: Vec<(i8, i8)> = b.stacks().iter().map(|s| (s.morale, s.luck)).collect();
        (values, b)
    };
    // Without effects: one alignment on each side (+1), no luck.
    assert_eq!(start(vec![], vec![]).0, [(1, 0), (1, 0)]);
    // Changes add up over both heroes: the attacker's own +2 morale and +2 luck with the
    // defender's −1 luck for its enemy; on the defender's side its own +1 morale and the
    // attacker's −1 for its enemy cancel. A hero's other effects change neither.
    let (values, _) = start(
        vec![
            Morale {
                amount: 2,
                affects: Own,
            },
            Luck {
                amount: 2,
                affects: Own,
            },
            Morale {
                amount: -1,
                affects: Enemy,
            },
            BattleEffect::NoRetreat,
        ],
        vec![
            Morale {
                amount: 1,
                affects: Own,
            },
            Luck {
                amount: -1,
                affects: Enemy,
            },
        ],
    );
    assert_eq!(values, [(3, 1), (1, 0)]);
    // Caps: the attacker's morale at most 0 for both sides (Spirit of Oppression) beats the
    // Minotaur's minimum +1 (the upper bound wins, KER-MOD-001) and its luck at most 0 for the
    // enemy beats the defender's own +3; on the attacker's side the smaller cap holds, the
    // defender's morale at most −1 for its enemy.
    let (values, b) = start(
        vec![
            Morale {
                amount: 2,
                affects: Own,
            },
            Luck {
                amount: 2,
                affects: Own,
            },
            MoraleAtMost {
                value: 0,
                affects: Both,
            },
            LuckAtMost {
                value: 0,
                affects: Enemy,
            },
        ],
        vec![
            Luck {
                amount: 3,
                affects: Own,
            },
            MoraleAtMost {
                value: -1,
                affects: Enemy,
            },
        ],
    );
    assert_eq!(values, [(-1, 2), (0, 0)]);
    // One change and one cap at most per side and stat: alignment twice and six from the heroes.
    assert_eq!(b.modifiers.iter().count(), 8);
    // The most the heroes insert: every kind for every side from both heroes gives a change and
    // a cap of morale and luck on each side, 8 in all (MODIFIER_CAPACITY).
    let every: Vec<BattleEffect> = Affects::ALL
        .into_iter()
        .flat_map(|affects| {
            [
                Morale { amount: 1, affects },
                Luck { amount: 1, affects },
                MoraleAtMost { value: 3, affects },
                LuckAtMost { value: 3, affects },
            ]
        })
        .collect();
    let (values, b) = start(every.clone(), every);
    assert_eq!(b.modifiers.iter().count(), 2 + 8);
    // Morale 1 + 4 (two heroes, own and both / enemy and both), capped at 3; luck 4 → 3.
    assert_eq!(values, [(3, 3), (3, 3)]);
}

// covers: BAT-HER-004
#[test]
fn hero_damage_bonuses_raise_the_sides_strikes_and_reduce_the_strikes_against_it() {
    // Heroes without attack and defense, with the given damage bonuses (basis points).
    let led = |r: &h3_content::Ruleset, ranged_bp, melee_bp, reduction_bp| {
        let mut h = hero(r);
        [h.attack, h.defense] = [0, 0];
        h.damage = DamageBonuses {
            ranged: Rate::bp(ranged_bp),
            melee: Rate::bp(melee_bp),
            reduction: Rate::bp(reduction_bp),
        };
        Some(h)
    };
    // Melee: 10 fixed (2 damage, attack = defense) strike 10 fixed; the target's 8 survivors
    // retaliate.
    let melee = |r: &h3_content::Ruleset, heroes: [Option<BattleHero>; 2]| {
        let mut b = BattleState::new(r, &with_heroes(r, heroes), 1).unwrap();
        place(&mut b, r, 0, 7, 5);
        place(&mut b, r, 1, 8, 5);
        activate(&mut b, 0);
        let attack = Command::Attack {
            target: StackId(1),
            from: at(r, 7, 5),
        };
        act(&mut b, r, attack)
            .unwrap()
            .iter()
            .filter_map(|e| match e.kind {
                EventKind::Struck { damage, .. } => Some(damage),
                _ => None,
            })
            .collect::<Vec<u64>>()
    };
    let r = ruleset();
    assert_eq!(melee(&r, [None, None]), [20, 16]);
    // The attacker's Offense +20 % raises its strike (24 kills 2), not the retaliation; its
    // Archery does not count in melee; its Armorer 50 % halves the retaliation.
    assert_eq!(melee(&r, [led(&r, 10_000, 2000, 5000), None]), [24, 8]);
    // The defender's hero: Offense raises the retaliation, Armorer lowers the attack.
    assert_eq!(melee(&r, [None, led(&r, 0, 5000, 2500)]), [15, 27]);
    // A shot of 10 archers (base 20..30, defense 10 against attack 6: −10 %, across the field:
    // −50 %) at 10 fixed: the shooter's side adds its ranged bonus only, the target's side
    // reduces.
    let shot = |heroes: [Option<BattleHero>; 2]| {
        let s = BattleSetup {
            heroes,
            ..setup(&r, &[(0, "archer", 10)], &[(0, "fixed", 10)])
        };
        let b = BattleState::new(&r, &s, 1).unwrap();
        crate::combat::shot_damage_range(&b, &r, StackId(0), StackId(1))
    };
    assert_eq!(shot([None, None]), (9, 13));
    assert_eq!(shot([led(&r, 10_000, 0, 0), None]), (18, 27));
    assert_eq!(shot([led(&r, 0, 10_000, 0), None]), (9, 13));
    assert_eq!(shot([None, led(&r, 10_000, 10_000, 5000)]), (4, 6));
    // The Ballista is the side's stack too: its shot (stack 1; damage 2..3, −50 % across the
    // field) gets the ranged bonus.
    let ballista = r.creature_by_key("t:ballista").unwrap();
    let machine_shot = |heroes: [Option<BattleHero>; 2]| {
        let s = BattleSetup {
            heroes,
            war_machines: [vec![ballista], vec![]],
            ..setup(&r, &[(0, "archer", 10)], &[(0, "fixed", 10)])
        };
        let b = BattleState::new(&r, &s, 1).unwrap();
        crate::combat::shot_damage_range(&b, &r, StackId(1), StackId(2))
    };
    assert_eq!(machine_shot([None, None]), (1, 1));
    assert_eq!(machine_shot([led(&r, 10_000, 0, 0), None]), (2, 3));
    // Under quirk Q-0005 the reduction is a single-precision rate: 50 fixed (base 100) against
    // Basic Armorer's 5 % deal 94 instead of 95 (batch 1a R3).
    let armorer = |profile: &str| {
        let r = ruleset_with(profile, |_| {});
        let s = BattleSetup {
            heroes: [None, led(&r, 0, 0, 500)],
            ..setup(&r, &[(0, "fixed", 50)], &[(0, "fixed", 100)])
        };
        let mut b = BattleState::new(&r, &s, 1).unwrap();
        place(&mut b, &r, 0, 7, 5);
        place(&mut b, &r, 1, 8, 5);
        activate(&mut b, 0);
        let attack = Command::Attack {
            target: StackId(1),
            from: at(&r, 7, 5),
        };
        act(&mut b, &r, attack)
            .unwrap()
            .iter()
            .find_map(|e| match e.kind {
                EventKind::Struck { damage, .. } => Some(damage),
                _ => None,
            })
    };
    assert_eq!(armorer("t:faithful"), Some(95));
    assert_eq!(armorer("t:single"), Some(94));
}

mod isolation {
    use proptest::prelude::{any, prop_assert, prop_assert_eq, proptest};
    use proptest::test_runner::Config;

    use super::*;

    /// What a side's stacks start with apart from morale and luck: statistics, traits,
    /// regeneration and places.
    fn statistics(
        s: &BattleState,
        side: BattleSide,
    ) -> Vec<(CreatureTypeId, crate::Hex, [u32; 9])> {
        s.stacks()
            .iter()
            .filter(|st| st.side == side)
            .map(|st| {
                let values = [
                    st.count,
                    st.max_hp,
                    u32::from(st.attack),
                    u32::from(st.defense),
                    u32::from(st.speed),
                    u32::from(st.damage_min),
                    u32::from(st.damage_max),
                    u32::from(st.shots),
                    st.traits.bits() << 16 | u32::from(st.regeneration),
                ];
                (st.creature, st.hex, values)
            })
            .collect()
    }

    fn morale_and_luck(s: &BattleState, side: BattleSide) -> Vec<(i8, i8)> {
        s.stacks()
            .iter()
            .filter(|st| st.side == side)
            .map(|st| (st.morale, st.luck))
            .collect()
    }

    proptest! {
        #![proptest_config(Config::with_cases(64))]

        /// Hero isolation (E04a exit criterion 4): whatever the attacker's hero (primary skills,
        /// any subset of every effect kind), the defender's stacks start with the same
        /// statistics, traits and regeneration as without it (BAT-HER-002, BAT-HER-005,
        /// BAT-HER-006); their morale and luck change only through effects on the enemy
        /// (BAT-HER-003, BAT-HER-007).
        #[test]
        fn a_hero_changes_only_its_own_sides_stacks(skills: [u8; 4], subset in any::<u16>()) {
            let r = ruleset();
            let effects: Vec<BattleEffect> = every_effect(&r)
                .into_iter()
                .enumerate()
                .filter(|(i, _)| subset & (1 << i) != 0)
                .map(|(_, e)| e)
                .collect();
            let on_enemy = effects.iter().any(|e| matches!(e,
                BattleEffect::Morale { affects, .. }
                | BattleEffect::Luck { affects, .. }
                | BattleEffect::MoraleAtMost { affects, .. }
                | BattleEffect::LuckAtMost { affects, .. } if *affects != Affects::Own));
            let mut led = hero(&r);
            [led.attack, led.defense, led.power, led.knowledge] = skills;
            led.effects = effects;
            let armies = setup(
                &r,
                &[(0, "fixed", 10), (1, "archer", 5), (2, "wight", 3)],
                &[(0, "fixed", 10), (1, "archer", 5), (2, "wight", 3)],
            );
            let with = BattleSetup { heroes: [Some(led), None], ..armies.clone() };
            let with = BattleState::new(&r, &with, 1).unwrap();
            let without = BattleState::new(&r, &armies, 1).unwrap();
            let defender = BattleSide::Defender;
            prop_assert_eq!(statistics(&with, defender), statistics(&without, defender));
            prop_assert!(
                on_enemy || morale_and_luck(&with, defender) == morale_and_luck(&without, defender)
            );
            // Not vacuous: the sixth effect (+25 % hit points for all) changes the hero's side.
            let attacker = BattleSide::Attacker;
            prop_assert!(
                subset & (1 << 5) == 0 || statistics(&with, attacker) != statistics(&without, attacker)
            );
        }
    }
}
