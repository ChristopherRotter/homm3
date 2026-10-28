//! E05 T4: reward v1 against the six hand-computed outcomes of T1's design (`ai_value` of
//! `base`, p = 10; `docs/epics/E05-log.md` §3 points here), the gold currency, and its zero
//! sum.

use h3_content::HeroTypeId;
use std::sync::Arc;

use h3_contracts::{
    ArmyStack, BattleHero, BattleResult, BattleSide, FortLevel, HeroOutcome, StackOrigin,
    StackRecord,
};

use super::*;

fn base() -> BattleRuleset {
    h3_pack_battle::compile(&[h3_pack_battle::BASE_PACK], "base:faithful").unwrap()
}

/// `count` creatures of `key` in each army slot, in order.
fn army(ruleset: &BattleRuleset, stacks: &[(&str, u32)]) -> Army {
    Army {
        stacks: stacks
            .iter()
            .enumerate()
            .map(|(slot, &(key, count))| ArmyStack {
                slot: slot as u8,
                creature: ruleset.creature_by_key(key).unwrap(),
                count,
            })
            .collect(),
    }
}

fn machines(ruleset: &BattleRuleset, keys: &[&str]) -> Vec<CreatureTypeId> {
    keys.iter()
        .map(|k| ruleset.creature_by_key(k).unwrap())
        .collect()
}

/// A war machine the battle placed, and whether it stands at the end.
fn placed(ruleset: &BattleRuleset, key: &str, standing: bool) -> (CreatureTypeId, bool) {
    (ruleset.creature_by_key(key).unwrap(), standing)
}

/// A hero with `mana` spell points (nothing else is read).
fn hero(mana: u16) -> Option<BattleHero> {
    let mut hero = BattleHero::new(HeroTypeId::from_index(0).unwrap());
    hero.magic.mana = mana;
    Some(hero)
}

/// The outcome with `survivors` and the war machines the battle placed, `machines` (each with
/// whether it stands), reported as its records and survivors.
fn outcome(
    result: BattleResult,
    survivors: [Army; 2],
    machines: [Vec<(CreatureTypeId, bool)>; 2],
    mana_left: [Option<u16>; 2],
) -> BattleOutcome {
    BattleOutcome {
        result,
        survivors,
        war_machines: machines.each_ref().map(|list| {
            list.iter()
                .filter(|(_, standing)| *standing)
                .map(|&(m, _)| m)
                .collect()
        }),
        stacks: machines.each_ref().map(|list| {
            list.iter()
                .map(|&(creature, standing)| StackRecord {
                    origin: StackOrigin::WarMachine,
                    creature,
                    start: 1,
                    end: u32::from(standing),
                })
                .collect()
        }),
        surrender_cost: None,
        heroes: mana_left.map(|m| {
            m.map(|mana| HeroOutcome {
                mana,
                learned: Vec::new(),
            })
        }),
        rounds: 3,
    }
}

const WON: BattleResult = BattleResult::Victory(BattleSide::Attacker);

/// `numerator / scale` as the reward reports it.
fn ratio(numerator: i128, scale: i128) -> f32 {
    (numerator as f64 / scale as f64) as f32
}

fn check(r: [f32; 2], numerator: i128, scale: i128) {
    assert_eq!(r[0], ratio(numerator, scale), "{r:?}");
    assert_eq!(r[1], -r[0]);
}

#[test]
fn a_wandering_monster() {
    // 1. 20 Swordsmen and 10 Griffins (12 410) attack 50 Skeletons (3 000); the hero casts 10
    // mana; the attacker loses 3 Swordsmen and wins: (3 000 − 1 335 − 100) / 15 410 = 0.102.
    let rules = base();
    let setup = BattleSetup {
        attacker: army(&rules, &[("base:swordsman", 20), ("base:griffin", 10)]),
        defender: army(&rules, &[("base:skeleton", 50)]),
        heroes: [hero(10), None],
        ..BattleSetup::default()
    };
    let end = outcome(
        WON,
        [
            army(&rules, &[("base:swordsman", 17), ("base:griffin", 10)]),
            Army::default(),
        ],
        [Vec::new(), Vec::new()],
        [Some(0), None],
    );
    let balance = Balance::of(&rules, &setup, &end, Currency::AiValue);
    assert_eq!(
        balance,
        Balance {
            lost: [1_335, 3_000],
            mana: [10, 0],
            scale: 15_410,
        }
    );
    let r = rewards(RewardConfig::default(), &rules, &setup, &end);
    check(r, 3_000 - 1_335 - 100, 15_410);
    assert!((r[0] - 0.102).abs() < 5e-4, "{r:?}");
}

#[test]
fn hero_against_hero_with_a_ballista() {
    // 2. 40 Orcs and a Ballista (8 280) against 60 Goblins and 15 Wolf Riders (5 550), whose hero
    // casts 15 mana; the attacker kills everything and loses 6 Orcs: 0.329; 0.285 had its
    // Ballista been destroyed too.
    let rules = base();
    let setup = BattleSetup {
        attacker: army(&rules, &[("base:orc", 40)]),
        defender: army(&rules, &[("base:goblin", 60), ("base:wolf_rider", 15)]),
        war_machines: [machines(&rules, &["base:ballista"]), Vec::new()],
        heroes: [hero(0), hero(15)],
        ..BattleSetup::default()
    };
    let survivors = [army(&rules, &[("base:orc", 34)]), Army::default()];
    let kept = outcome(
        WON,
        survivors.clone(),
        [vec![placed(&rules, "base:ballista", true)], Vec::new()],
        [Some(0), Some(0)],
    );
    let r = rewards(RewardConfig::default(), &rules, &setup, &kept);
    check(r, 5_550 - 1_152 + 150, 13_830);
    assert!((r[0] - 0.329).abs() < 5e-4, "{r:?}");
    let lost = [vec![placed(&rules, "base:ballista", false)], Vec::new()];
    let destroyed = outcome(WON, survivors, lost, [Some(0), Some(0)]);
    let r = rewards(RewardConfig::default(), &rules, &setup, &destroyed);
    check(r, 5_550 - 1_752 + 150, 13_830);
    assert!((r[0] - 0.285).abs() < 5e-4, "{r:?}");
}

#[test]
fn summons_cost_their_mana_only() {
    // 3. A hero summons for 25 mana: −250 before S; the created stack is no survivor and counts
    // nothing, whatever it killed or lost.
    let rules = base();
    let setup = BattleSetup {
        attacker: army(&rules, &[("base:swordsman", 10)]),
        defender: army(&rules, &[("base:skeleton", 10)]),
        heroes: [hero(25), None],
        ..BattleSetup::default()
    };
    let end = outcome(
        WON,
        [army(&rules, &[("base:swordsman", 10)]), Army::default()],
        [Vec::new(), Vec::new()],
        [Some(0), None],
    );
    let r = rewards(RewardConfig::default(), &rules, &setup, &end);
    check(r, 600 - 250, 5_050);
}

#[test]
fn a_siege_counts_the_armies_only() {
    // 4. A garrison of 6 000 without a hero loses to an attacker of 15 000 that loses 2 000:
    // (6 000 − 2 000) / 21 000 = 0.190; the town, its turrets and walls are in no army.
    let rules = base();
    let setup = BattleSetup {
        attacker: army(&rules, &[("base:goblin", 185), ("base:wolf_rider", 30)]),
        defender: army(&rules, &[("base:skeleton", 100)]),
        ..BattleSetup::default()
    };
    let end = outcome(
        WON,
        [
            army(&rules, &[("base:goblin", 156), ("base:wolf_rider", 28)]),
            Army::default(),
        ],
        [Vec::new(), Vec::new()],
        [None, None],
    );
    let r = rewards(RewardConfig::default(), &rules, &setup, &end);
    check(r, 4_000, 21_000);
    assert!((r[0] - 0.190).abs() < 5e-4, "{r:?}");
}

#[test]
fn a_draw_in_self_play() {
    // 5. Both armies die by the same action: (K_d − K_a + p · (m_d − m_a)) / S, each K the whole
    // army and its destroyed machines.
    let rules = base();
    let setup = BattleSetup {
        attacker: army(&rules, &[("base:griffin", 10)]),
        defender: army(&rules, &[("base:orc", 20)]),
        war_machines: [Vec::new(), machines(&rules, &["base:first_aid_tent"])],
        heroes: [hero(12), hero(30)],
        ..BattleSetup::default()
    };
    let end = outcome(
        BattleResult::Draw,
        [Army::default(), Army::default()],
        [
            Vec::new(),
            vec![placed(&rules, "base:first_aid_tent", false)],
        ],
        [Some(2), Some(10)],
    );
    let r = rewards(RewardConfig::default(), &rules, &setup, &end);
    let (k_a, k_d) = (3_510, 3_840 + 300);
    check(r, k_d - k_a + 10 * (20 - 10), k_a + k_d);
}

#[test]
fn the_machines_count_as_the_battle_placed_them() {
    // E05 T9b: a hero's Catapult, which no setup lists, counts at the start and when destroyed; a
    // listed Ballista the battle left out (BAT-QUIRK-23 at a fort) counts nothing.
    let rules = base();
    let setup = BattleSetup {
        attacker: army(&rules, &[("base:orc", 40)]),
        defender: army(&rules, &[("base:goblin", 60)]),
        war_machines: [Vec::new(), machines(&rules, &["base:ballista"])],
        ..BattleSetup::default()
    };
    let survivors = [army(&rules, &[("base:orc", 34)]), Army::default()];
    let (orcs, goblins, catapult) = (40 * 192, 60 * 60, 500);
    for (standing, lost) in [(true, 6 * 192), (false, 6 * 192 + catapult)] {
        let machines = [vec![placed(&rules, "base:catapult", standing)], Vec::new()];
        let end = outcome(WON, survivors.clone(), machines, [None, None]);
        let balance = Balance::of(&rules, &setup, &end, Currency::AiValue);
        assert_eq!(balance.lost, [lost, goblins]);
        assert_eq!(balance.scale, orcs + goblins + catapult);
    }
}

#[test]
fn a_played_siege_counts_the_machines_the_battle_placed() {
    // The same on a sampled siege the engine plays to its end: a hero defends a walled town with
    // a Ballista it keeps out of the battle, the attacker's hero brings the Catapult.
    let sampler = crate::encounter::tests::SAMPLER.clone().with_shares();
    let rules = sampler.battle();
    let [ballista, catapult] =
        ["base:ballista", "base:catapult"].map(|k| rules.creature_by_key(k).unwrap());
    let encounter = (0..)
        .map(|seed| sampler.sample(seed))
        .find(|e| {
            e.kind == crate::encounter::Kind::SiegeHero
                && e.setup
                    .siege
                    .as_ref()
                    .is_some_and(|s| s.fort != FortLevel::None)
                && e.setup.war_machines[1].contains(&ballista)
        })
        .unwrap();
    let seats = encounter.seats;
    let scenario = crate::batch::Scenario::new(Arc::clone(rules), &encounter.setup, true, 1, seats);
    let mut env = scenario.unwrap().env(1, 0).unwrap();
    crate::rollout::random_rollout(&mut env, 1, 50_000);
    let end = env.state().outcome().unwrap();
    let placed = |side: usize| -> Vec<CreatureTypeId> {
        end.stacks[side]
            .iter()
            .filter(|r| r.origin == StackOrigin::WarMachine)
            .map(|r| r.creature)
            .collect()
    };
    assert!(placed(0).contains(&catapult));
    assert!(!placed(1).contains(&ballista));
    let value = |c| creature_value(rules, Currency::AiValue, c);
    let setup = env.setup();
    let armies: i128 = [&setup.attacker, &setup.defender]
        .iter()
        .flat_map(|a| &a.stacks)
        .map(|s| i128::from(s.count) * value(s.creature))
        .sum();
    let machines: i128 = [placed(0), placed(1)].concat().into_iter().map(value).sum();
    let balance = Balance::of(rules, setup, &end, Currency::AiValue);
    assert_eq!(balance.scale, armies + machines);
}

#[test]
fn raised_demons_offset_the_loss() {
    // 6. Pit Lords raise 5 Demons (445) from their side's dead Swordsmen: the Swordsmen count
    // fully at the start, the Demons they kept at the end: 2 225 less lost.
    let rules = base();
    let setup = BattleSetup {
        attacker: army(&rules, &[("base:pit_lord", 5), ("base:swordsman", 20)]),
        defender: army(&rules, &[("base:skeleton", 50)]),
        ..BattleSetup::default()
    };
    let end = outcome(
        WON,
        [
            army(&rules, &[("base:pit_lord", 5), ("base:demon", 5)]),
            Army::default(),
        ],
        [Vec::new(), Vec::new()],
        [None, None],
    );
    let balance = Balance::of(&rules, &setup, &end, Currency::AiValue);
    assert_eq!(balance.lost, [8_900 - 2_225, 3_000]);
    check(balance.rewards(10), 3_000 - 6_675, 18_020);
}

#[test]
fn gold_prices_creatures_in_the_ruleset_money() {
    let rules = base();
    let swordsman = rules.creature_by_key("base:swordsman").unwrap();
    assert_eq!(creature_value(&rules, Currency::Gold, swordsman), 300);
    assert_eq!(creature_value(&rules, Currency::AiValue, swordsman), 445);
    let outside = CreatureTypeId::from_index(60_000).unwrap();
    assert_eq!(creature_value(&rules, Currency::AiValue, outside), 0);
    // The same battle as the first example, priced in gold, mana at 5 gold per point.
    let setup = BattleSetup {
        attacker: army(&rules, &[("base:swordsman", 20)]),
        defender: army(&rules, &[("base:swordsman", 10)]),
        heroes: [None, hero(8)],
        ..BattleSetup::default()
    };
    let end = outcome(
        WON,
        [army(&rules, &[("base:swordsman", 14)]), Army::default()],
        [Vec::new(), Vec::new()],
        [None, Some(0)],
    );
    let config = RewardConfig {
        currency: Currency::Gold,
        mana_price: 5,
    };
    check(
        rewards(config, &rules, &setup, &end),
        3_000 - 1_800 + 5 * 8,
        9_000,
    );
    // A hero that gains mana (spent < 0) is credited for it at the same price.
    let gained = outcome(
        WON,
        [army(&rules, &[("base:swordsman", 20)]), Army::default()],
        [Vec::new(), Vec::new()],
        [None, Some(20)],
    );
    check(
        rewards(config, &rules, &setup, &gained),
        3_000 - 5 * 12,
        9_000,
    );
}

#[test]
fn empty_armies_scale_by_one_and_the_default_is_ai_value_at_ten() {
    let rules = base();
    let setup = BattleSetup::default();
    let end = outcome(
        BattleResult::Draw,
        [Army::default(), Army::default()],
        [Vec::new(), Vec::new()],
        [None, None],
    );
    let balance = Balance::of(&rules, &setup, &end, Currency::Gold);
    assert_eq!(balance.scale, 1);
    assert_eq!(balance.rewards(10), [0.0, -0.0]);
    assert_eq!(
        RewardConfig::default(),
        RewardConfig {
            currency: Currency::AiValue,
            mana_price: 10,
        }
    );
    assert_eq!(Currency::default(), Currency::AiValue);
}

#[test]
fn the_potential_is_the_balance_so_far_and_reward_v1s_at_the_end() {
    // E25 T2a: a running battle's balance (`Balance::standing`, the shaping's potential) is 0 at
    // the start and zero-sum; at a decided battle's end it is the outcome's (none of these 40
    // sampled encounters of every kind, played at random to their end, meets the end's own
    // corrections of a temporary resurrection, BAT-MAG-035, -038).
    let sampler = crate::encounter::tests::SAMPLER.clone().with_shares();
    let rules = sampler.battle();
    let config = RewardConfig::default();
    let mut ended = 0;
    for seed in 0..40 {
        let encounter = sampler.sample(seed);
        let scenario = crate::batch::Scenario::new(
            Arc::clone(rules),
            &encounter.setup,
            true,
            1,
            encounter.seats,
        );
        let mut env = scenario.unwrap().env(seed, 0).unwrap();
        let start = potential(config, rules, env.setup(), env.state());
        assert_eq!(start, [0.0, 0.0], "seed {seed}");
        let mut rng = h3_kernel::Rng::for_stream(seed, h3_kernel::StreamId::AGENT);
        let mut legal = h3_battle::LegalCommands::new();
        while let Some(side) = env.to_move() {
            let command = crate::rollout::pick(&env, side, &mut rng, &mut legal);
            let _ = crate::rollout::step_command(&mut env, side, command);
            let now = potential(config, rules, env.setup(), env.state());
            assert_eq!(now[0], -now[1], "seed {seed}");
        }
        let end = env.state().outcome().unwrap();
        ended += 1;
        let standing = Balance::standing(rules, env.setup(), env.state(), Currency::AiValue);
        let of = Balance::of(rules, env.setup(), &end, Currency::AiValue);
        assert_eq!(standing, of, "seed {seed}");
        assert_eq!(
            potential(config, rules, env.setup(), env.state()),
            end_reward(&env)
        );
    }
    assert_eq!(ended, 40);
}

/// Reward v1 of `env`'s decided battle at the default settings.
fn end_reward(env: &crate::BattleEnv) -> [f32; 2] {
    let end = env.state().outcome().unwrap();
    rewards(RewardConfig::default(), env.ruleset(), env.setup(), &end)
}
