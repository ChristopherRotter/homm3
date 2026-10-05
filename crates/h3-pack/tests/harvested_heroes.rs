//! Robustness of the hero pipeline on the harvested SoD packs (E04a T12c review): random legal
//! hero details of every hero type, skill and artifact derive into battle heroes
//! (HER-BAT-001 to HER-BAT-010) whose battles start and play to the end with every legal command
//! accepted (BAT-HER-001).

use h3_battle::{BattleState, LegalCommands, NoopSink, apply, legal_commands, observe};
use h3_content::{
    ArtifactTypeId, CreatureFlag, CreatureTypeId, Formation, HeroTypeId, Ruleset, SkillId,
};
use h3_contracts::{Army, ArmyStack, BattleSetup, BattleSide, MAX_BATTLE_EFFECTS};
use h3_heroes::{HeroDetails, derive_battle_hero, validate, war_machines};
use h3_kernel::Rng;

/// A uniform index below `n` (at least 1).
fn pick(rng: &mut Rng, n: usize) -> usize {
    rng.below(u32::try_from(n).unwrap()) as usize
}

/// Valid details: a random hero type, level 1 to 255, primary skills 0 to 99, up to eight skills
/// at random levels, and worn artifacts drawn at random, each kept if the details stay valid.
fn details(r: &Ruleset, rng: &mut Rng) -> HeroDetails {
    let mut d = HeroDetails {
        hero_type: HeroTypeId::from_index(pick(rng, r.hero_types().len())).unwrap(),
        level: u8::try_from(1 + rng.below(255)).unwrap(),
        experience: 0,
        primary: [0; 4].map(|_| u8::try_from(rng.below(100)).unwrap()),
        skills: vec![],
        worn: vec![],
        backpack: vec![],
        spells: vec![],
        mana: 0,
    };
    let skills = pick(rng, 9);
    while d.skills.len() < skills {
        let id = SkillId::from_index(pick(rng, r.skills().len())).unwrap();
        if d.skills.iter().all(|&(s, _)| s != id) {
            d.skills.push((id, u8::try_from(1 + rng.below(3)).unwrap()));
        }
    }
    for _ in 0..40 {
        d.worn
            .push(ArtifactTypeId::from_index(pick(rng, r.artifacts().len())).unwrap());
        if validate(&d, r).is_err() {
            d.worn.pop();
        }
    }
    d
}

/// One to seven stacks of random non-war-machine creatures, a tenth of them with up to
/// `u32::MAX` creatures.
fn army(rng: &mut Rng, pool: &[CreatureTypeId]) -> Army {
    Army {
        stacks: (0..=rng.below(7))
            .map(|slot| ArmyStack {
                slot: u8::try_from(slot).unwrap(),
                creature: pool[pick(rng, pool.len())],
                count: {
                    let most = if rng.below(10) == 0 { u32::MAX } else { 500 };
                    1 + rng.below(most)
                },
            })
            .collect(),
    }
}

// covers: HER-BAT-001, BAT-HER-001
#[test]
fn random_legal_sod_heroes_derive_and_play() {
    let sod = h3_pack::compile(
        &[h3_pack::SOD_PACK, h3_pack::AB_PACK, h3_pack::BASE_PACK],
        "base:faithful",
    )
    .unwrap();
    let pool: Vec<CreatureTypeId> = (0..sod.creature_count())
        .filter_map(CreatureTypeId::from_index)
        .filter(|&c| {
            sod.creature(c)
                .is_some_and(|t| !t.flags.contains(CreatureFlag::WarMachine))
        })
        .collect();
    let mut rng = Rng::from_seed(12);
    let mut legal = LegalCommands::new();
    for game in 0..120u64 {
        let details = [details(&sod, &mut rng), details(&sod, &mut rng)];
        let heroes = details
            .clone()
            .map(|d| derive_battle_hero(&d, &sod).unwrap());
        assert!(heroes.iter().all(|h| h.effects.len() <= MAX_BATTLE_EFFECTS));
        // A quarter of the battles against an army without a hero.
        let led = rng.below(4) != 0;
        let setup = BattleSetup {
            attacker: army(&mut rng, &pool),
            defender: army(&mut rng, &pool),
            obstacles: vec![],
            war_machines: [
                war_machines(&details[0], &sod),
                if led {
                    war_machines(&details[1], &sod)
                } else {
                    vec![]
                },
            ],
            heroes: {
                let [a, d] = heroes;
                [Some(a), led.then_some(d)]
            },
            terrain: None,
            formations: [Formation::Loose; 2],
            funds: [rng.next_u32(), rng.next_u32()],
        };
        let mut state = BattleState::new(&sod, &setup, game).unwrap();
        for _ in 0..2_000 {
            let Some(side) = state.side_to_act() else {
                break;
            };
            legal_commands(&state, &sod, side, &mut legal);
            // Leaving the battle is rare, so that battles run on.
            let staying: Vec<_> = legal.iter().filter(|c| !c.leaves_battle()).collect();
            let cmd = if staying.is_empty() || rng.below(100) == 0 {
                legal.as_slice()[pick(&mut rng, legal.len())]
            } else {
                *staying[pick(&mut rng, staying.len())]
            };
            apply(&mut state, &sod, side, cmd, &mut NoopSink).unwrap();
        }
        for side in BattleSide::BOTH {
            let _ = observe(&state, &sod, side);
        }
        assert_eq!(state.outcome().is_some(), state.is_over());
    }
}
