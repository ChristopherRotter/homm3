//! The original's battle AI on the harvested packs (E23 T2a; ADR-0027 point 3, the epic's exit
//! criterion 1): in random battles of the SoD creatures, on the terrains' ordinary fields, on
//! every special battlefield and at a town of every faction's siege record at every fort level,
//! in both quirk profiles, at every difficulty and with random heroes (Tactics, Artillery, First
//! Aid, Ballistics, spells), the computer's choice for the side to act is a command the engine
//! accepts at every step (no fallback fires), and the same state and stream give the same
//! command. Most steps play it; the others play a random legal command or a hero's cast, so the
//! games reach effects and stand-ins (`h3_battle_ai`'s stubs) the policy alone would not.

use h3_battle::{
    BattleState, CastTargets, Command, LegalCommands, NoopSink, apply, cast_targets,
    castable_spells, legal_commands, second_targets, validate,
};
use h3_content::{CreatureFlag, CreatureTypeId, Formation, HeroTypeId, SpellCategory, SpellId};
use h3_content_battle::BattleRuleset;
use h3_contracts::{Army, ArmyStack, BattleSetup, BattleSide, FortLevel, Moat, Siege};
use h3_heroes::{ArtifactInstance, HeroDetails, derive_battle_hero, war_machines};
use h3_kernel::Rng;

/// A uniform index below `n` (at least 1).
fn pick(rng: &mut Rng, n: usize) -> usize {
    rng.below(u32::try_from(n).unwrap()) as usize
}

/// Valid details of a random hero: level 1 to 30, primary skills 0 to 20, Tactics, Artillery,
/// First Aid and Ballistics each at a random level (none to Expert), each war machine's artifact
/// half the time, a spell book with half the spells and some mana.
fn details(r: &BattleRuleset, rng: &mut Rng) -> HeroDetails {
    let mut d = HeroDetails {
        hero_type: HeroTypeId::from_index(pick(rng, r.hero_types().len())).unwrap(),
        level: u8::try_from(1 + rng.below(30)).unwrap(),
        experience: 0,
        primary: [0; 4].map(|_| u8::try_from(rng.below(21)).unwrap()),
        skills: vec![],
        worn: vec![],
        backpack: vec![],
        spells: vec![],
        mana: [0, 20, 100][pick(rng, 3)],
    };
    for skill in [
        "base:tactics",
        "base:artillery",
        "base:first_aid",
        "base:ballistics",
    ] {
        let level = u8::try_from(rng.below(4)).unwrap();
        if level > 0 {
            d.skills.push((r.skill_by_key(skill).unwrap(), level));
        }
    }
    for artifact in ["base:spell_book", "base:ballista", "base:first_aid_tent"] {
        if rng.below(2) == 0 {
            d.worn
                .push(ArtifactInstance::from(r.artifact_by_key(artifact).unwrap()));
        }
    }
    d.spells = (0..r.spells().len())
        .filter_map(SpellId::from_index)
        .filter(|&s| r.spell(s).unwrap().category != SpellCategory::Ability)
        .filter(|_| rng.below(2) == 0)
        .collect();
    h3_heroes::validate(&d, r).unwrap();
    d
}

/// One to seven stacks of `pool`, counts from 1 to 200.
fn army(rng: &mut Rng, pool: &[CreatureTypeId]) -> Army {
    Army {
        stacks: (0..=rng.below(7))
            .map(|slot| ArmyStack {
                slot: u8::try_from(slot).unwrap(),
                creature: pool[pick(rng, pool.len())],
                count: 1 + rng.below(200),
            })
            .collect(),
    }
}

/// How the games went: the policy's commands by kind, and the battles that ended.
#[derive(Debug, Default)]
struct Reach {
    shots: u32,
    wall_shots: u32,
    tactics_ends: u32,
    defends: u32,
    over: u32,
}

/// A hero's random cast for `side`, if it may cast.
fn cast(
    state: &BattleState,
    r: &BattleRuleset,
    side: BattleSide,
    rng: &mut Rng,
) -> Option<Command> {
    let spells: Vec<SpellId> = castable_spells(state, r, side).iter().collect();
    if spells.is_empty() {
        return None;
    }
    let spell = spells[pick(rng, spells.len())];
    let (mut targets, mut seconds) = (CastTargets::new(), CastTargets::new());
    cast_targets(state, r, side, spell, &mut targets);
    let target = targets.as_slice()[pick(rng, targets.len())];
    second_targets(state, r, side, (spell, target), &mut seconds);
    let second = seconds.as_slice()[pick(rng, seconds.len())];
    Some(Command::Cast {
        spell,
        target,
        second,
    })
}

// covers: BAT-AI-001, BAT-AI-002
#[test]
fn the_computers_choice_is_legal_in_random_battles_of_the_harvested_creatures() {
    let mut reach = Reach::default();
    for profile in ["base:faithful", "base:strict-fixed"] {
        let sod = h3_pack_battle::compile(
            &[
                h3_pack_battle::SOD_PACK,
                h3_pack_battle::AB_PACK,
                h3_pack_battle::BASE_PACK,
            ],
            profile,
        )
        .unwrap();
        let pool: Vec<CreatureTypeId> = (0..sod.creature_count())
            .filter_map(CreatureTypeId::from_index)
            .filter(|&c| {
                sod.creature(c)
                    .is_some_and(|t| !t.flags.contains(CreatureFlag::WarMachine))
            })
            .collect();
        let factions: Vec<_> = sod.sieges().iter().map(|s| s.faction).collect();
        let battlefields: Vec<_> = sod.battlefields().iter().map(|b| b.key.clone()).collect();
        let mut rng = Rng::from_seed(23);
        let mut seat = Rng::from_seed(2023);
        let mut legal = LegalCommands::new();
        for game in 0..48u64 {
            let details = [details(&sod, &mut rng), details(&sod, &mut rng)];
            let heroes = details
                .clone()
                .map(|d| (rng.below(3) != 0).then(|| derive_battle_hero(&d, &sod).unwrap()));
            // Half the games at a town, a quarter on a special battlefield.
            let (siege, battlefield) = match game % 4 {
                0 | 1 => {
                    let fort = [
                        FortLevel::None,
                        FortLevel::Fort,
                        FortLevel::Citadel,
                        FortLevel::Castle,
                    ][pick(&mut rng, 4)];
                    let moat = if fort >= FortLevel::Citadel {
                        [Moat::Absent, Moat::Harmless, Moat::Damaging][pick(&mut rng, 3)]
                    } else {
                        Moat::Absent
                    };
                    let town = Siege {
                        faction: factions[usize::try_from(game).unwrap() % factions.len()],
                        fort,
                        moat,
                        structures: u8::try_from(rng.below(256)).unwrap(),
                        escape_tunnel: rng.below(2) == 0,
                        morale: 0,
                        luck: 0,
                    };
                    (Some(town), None)
                }
                2 => (
                    None,
                    Some(battlefields[pick(&mut rng, battlefields.len())].clone()),
                ),
                _ => (None, None),
            };
            let machines = [0, 1].map(|i| {
                if heroes[i].is_some() {
                    war_machines(&details[i], &sod)
                } else {
                    vec![]
                }
            });
            let setup = BattleSetup {
                attacker: army(&mut rng, &pool),
                defender: army(&mut rng, &pool),
                battlefield,
                siege,
                war_machines: machines,
                heroes,
                formations: [Formation::Loose, Formation::Loose],
                funds: [rng.next_u32(), rng.next_u32()],
                difficulty: u8::try_from(game % 5).unwrap(),
                human: [rng.below(2) == 0, rng.below(2) == 0],
                ..BattleSetup::default()
            };
            let mut state = BattleState::new(&sod, &setup, game)
                .unwrap_or_else(|e| panic!("{profile} game {game}: the setup is refused: {e:?}"));
            for _ in 0..300u32 {
                let Some(side) = state.side_to_act() else {
                    reach.over += 1;
                    break;
                };
                let proposed = h3_battle_ai::unchecked(&state, &sod, side, &mut seat.clone());
                validate(&state, &sod, side, proposed).unwrap_or_else(|e| {
                    panic!("{profile} game {game}: the computer's {proposed:?} is refused: {e:?}")
                });
                let mut again = seat.clone();
                let command = h3_battle_ai::choose(&state, &sod, side, &mut seat);
                assert_eq!(command, proposed, "{profile} game {game}");
                assert_eq!(
                    command,
                    h3_battle_ai::choose(&state, &sod, side, &mut again),
                    "{profile} game {game}"
                );
                assert_eq!(seat, again, "{profile} game {game}");
                let roll = rng.below(10);
                let played = if roll < 7 {
                    match command {
                        Command::Shoot { .. } => reach.shots += 1,
                        Command::ShootWall { .. } => reach.wall_shots += 1,
                        Command::EndTactics => reach.tactics_ends += 1,
                        _ => reach.defends += 1,
                    }
                    command
                } else if let Some(cast) = cast(&state, &sod, side, &mut rng).filter(|_| roll < 9) {
                    cast
                } else {
                    legal_commands(&state, &sod, side, &mut legal);
                    let staying: Vec<Command> = legal
                        .iter()
                        .copied()
                        .filter(|c| !c.leaves_battle())
                        .collect();
                    staying[pick(&mut rng, staying.len())]
                };
                apply(&mut state, &sod, side, played, &mut NoopSink).unwrap_or_else(|e| {
                    panic!("{profile} game {game}: {played:?} is rejected: {e:?}")
                });
            }
        }
    }
    assert!(
        reach.shots > 500
            && reach.wall_shots > 20
            && reach.tactics_ends > 20
            && reach.defends > 1000
            && reach.over > 30,
        "{reach:?}"
    );
}
