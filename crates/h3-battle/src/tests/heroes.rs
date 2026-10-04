//! The heroes leading the sides (BAT-HER-001): what the battle checks, keeps and shows, the
//! hero's attack and defense on its stacks (BAT-HER-002), its morale and luck effects
//! (BAT-HER-003) and its damage bonuses (BAT-HER-004).

use std::vec;
use std::vec::Vec;

use h3_content::{
    Affects, CreatureFlag, CreatureSelector, CreatureStat, CreatureTypeId, HeroTypeId, SpellId,
};
use h3_contracts::{
    BallistaControl, BattleEffect, BattleHero, DamageBonuses, MAX_BATTLE_EFFECTS, StatOp,
};

use super::support::*;
use crate::{
    BattleSetup, BattleSide, BattleState, Command, EventKind, HeroError, HeroView, SetupError,
    StackId, observe,
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
    };
    let s = BattleState::new(&r, &with_heroes(&r, [Some(hero(&r)), None]), 1).unwrap();
    assert_eq!(s.hero(BattleSide::Attacker), Some(view));
    assert_eq!(s.hero(BattleSide::Defender), None);
    for viewer in BattleSide::BOTH {
        assert_eq!(observe(&s, viewer).heroes, [Some(view), None]);
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

// covers: BAT-HER-001
#[test]
fn values_no_real_hero_could_have_are_accepted() {
    let r = ruleset();
    let mut odd = hero(&r);
    [odd.attack, odd.defense, odd.power, odd.knowledge] = [255, 200, 0, 0];
    odd.tactics = 3;
    odd.damage.ranged_bp = u16::MAX;
    odd.damage.melee_bp = u16::MAX;
    odd.damage.reduction_bp = 10_000;
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
    odd.magic.schools = [3; 4];
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
            |_, h| h.damage.reduction_bp = 10_001,
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
        (|_, h| h.magic.schools[2] = 4, HeroError::SchoolLevel),
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
            ranged_bp,
            melee_bp,
            reduction_bp,
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
