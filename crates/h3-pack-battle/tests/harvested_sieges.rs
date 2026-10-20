//! Robustness of sieges on the harvested packs (E04c T10b review): random legal SoD heroes
//! derive into battle heroes whose battles at a town of every faction's siege record, at every
//! fort level, with and without the moat and the Tower's mines, and on every special
//! battlefield, in both quirk profiles, play with every listed command accepted (the wall shots
//! of the Catapult and the Cyclopes, Earthquake and Teleport at every listed target and
//! destination among them), with armies of up to `u32::MAX` creatures.

use h3_battle::{
    BattleState, CastTargets, Command, LegalCommands, NoopSink, apply, cast_targets,
    castable_spells, creature_cast_targets, legal_commands, observe, second_targets, validate,
};
use h3_content::{
    ArtifactTypeId, CreatureFlag, CreatureTypeId, Formation, HeroEffect, HeroTypeId, SkillId,
    SpellCategory, SpellId,
};
use h3_content_battle::{Ability, BattleRuleset};
use h3_contracts::{Army, ArmyStack, BattleSetup, BattleSide, FortLevel, Moat, Siege};
use h3_heroes::{
    ArtifactInstance, HeroDetails, derive_battle_hero, validate as validate_hero, war_machines,
};
use h3_kernel::Rng;

/// A uniform index below `n` (at least 1).
fn pick(rng: &mut Rng, n: usize) -> usize {
    rng.below(u32::try_from(n).unwrap()) as usize
}

/// Valid details of a casting hero: a random hero type, level 1 to 74 with its experience (HER-EXP-002), primary skills 0 to 99,
/// Ballistics and Artillery at random levels (none to Expert) and up to six other skills, worn
/// artifacts drawn at random (each kept if the details stay valid), a Spell Book, half the
/// spells and mana.
fn details(r: &BattleRuleset, rng: &mut Rng) -> HeroDetails {
    let hero_type = HeroTypeId::from_index(pick(rng, r.hero_types().len())).unwrap();
    let level = u8::try_from(1 + rng.below(74)).unwrap();
    let mut d = HeroDetails {
        hero_type,
        level,
        experience: h3_heroes::experience_for(r.hero_params().unwrap(), level),
        offered: Default::default(),
        primary: [0; 4].map(|_| u8::try_from(rng.below(100)).unwrap()),
        skills: vec![],
        worn: vec![],
        backpack: vec![],
        spells: vec![],
        mana: 0,
    };
    for key in ["base:ballistics", "base:artillery"] {
        let level = u8::try_from(rng.below(4)).unwrap();
        if level > 0 {
            d.skills.push((r.skill_by_key(key).unwrap(), level));
        }
    }
    let skills = d.skills.len() + pick(rng, 7);
    while d.skills.len() < skills {
        let id = SkillId::from_index(pick(rng, r.skills().len())).unwrap();
        if d.skills.iter().all(|&(s, _)| s != id) {
            d.skills.push((id, u8::try_from(1 + rng.below(3)).unwrap()));
        }
    }
    let hero_spells: Vec<SpellId> = (0..r.spells().len())
        .filter_map(SpellId::from_index)
        .filter(|&s| r.spell(s).unwrap().category != SpellCategory::Ability)
        .collect();
    for _ in 0..40 {
        let artifact = ArtifactTypeId::from_index(pick(rng, r.artifacts().len())).unwrap();
        let scroll = r
            .artifact(artifact)
            .unwrap()
            .effects
            .iter()
            .any(|e| matches!(e, HeroEffect::Scroll));
        d.worn.push(if scroll {
            ArtifactInstance::scroll(artifact, hero_spells[pick(rng, hero_spells.len())])
        } else {
            artifact.into()
        });
        if validate_hero(&d, r).is_err() {
            d.worn.pop();
        }
    }
    let book = ArtifactInstance::from(r.artifact_by_key("base:spell_book").unwrap());
    if !d.worn.contains(&book) {
        d.worn.push(book);
    }
    for spell in hero_spells {
        if rng.below(2) == 0 {
            d.spells.push(spell);
        }
    }
    d.mana = [20, 100, 500, 999][pick(rng, 4)];
    validate_hero(&d, r).unwrap();
    d
}

/// One to seven stacks, half of them of `special` (the wall shooters, flyers, teleporters,
/// two-hex stacks and casters), the rest of `pool`; counts up to `u32::MAX`.
fn army(rng: &mut Rng, pool: &[CreatureTypeId], special: &[CreatureTypeId]) -> Army {
    Army {
        stacks: (0..=rng.below(7))
            .map(|slot| ArmyStack {
                slot: u8::try_from(slot).unwrap(),
                creature: if rng.below(2) == 0 {
                    special[pick(rng, special.len())]
                } else {
                    pool[pick(rng, pool.len())]
                },
                count: match rng.below(4) {
                    0 => 1 + rng.below(100),
                    1 => 60_000 + rng.below(140_000),
                    2 => 1 + rng.below(u32::MAX),
                    _ => u32::MAX,
                },
            })
            .collect(),
    }
}

/// What the games reached: wall shots, Earthquakes, Teleports, destroyed parts, battles over.
#[derive(Debug, Default)]
struct Reach {
    wall_shots: u32,
    quakes: u32,
    teleports: u32,
    destroyed: u32,
    over: u32,
}

/// Every listed command applies to a copy; every listed Earthquake and Teleport cast validates;
/// a wall shot that is not listed is rejected without a change of the state's hash.
fn probe(state: &BattleState, r: &BattleRuleset, side: BattleSide, special: &[SpellId], game: u64) {
    let mut legal = LegalCommands::new();
    legal_commands(state, r, side, &mut legal);
    for c in legal.iter() {
        let mut copy = state.clone();
        apply(&mut copy, r, side, *c, &mut NoopSink)
            .unwrap_or_else(|e| panic!("game {game}: the listed {c:?} is rejected: {e:?}"));
    }
    let hash = state.state_hash();
    for part in [0u8, 3, 7, 8, u8::MAX] {
        let c = Command::ShootWall { part };
        if legal.iter().any(|l| *l == c) {
            continue;
        }
        for actor in BattleSide::BOTH {
            let mut copy = state.clone();
            assert!(
                apply(&mut copy, r, actor, c, &mut NoopSink).is_err(),
                "game {game}: {c:?}"
            );
            assert_eq!(
                copy.state_hash(),
                hash,
                "game {game}: a rejected {c:?} changed the state"
            );
        }
    }
    let mut targets = CastTargets::new();
    let mut seconds = CastTargets::new();
    for spell in castable_spells(state, r, side).iter() {
        if !special.contains(&spell) {
            continue;
        }
        cast_targets(state, r, side, spell, &mut targets);
        assert!(
            !targets.is_empty(),
            "game {game}: castable {spell:?} without a target"
        );
        for target in targets.iter() {
            second_targets(state, r, side, (spell, *target), &mut seconds);
            assert!(
                !seconds.is_empty(),
                "game {game}: {spell:?} at {target:?} without a second target"
            );
            for second in seconds.iter() {
                let c = Command::Cast {
                    spell,
                    target: *target,
                    second: *second,
                };
                validate(state, r, side, c)
                    .unwrap_or_else(|e| panic!("game {game}: the listed {c:?} is rejected: {e:?}"));
            }
        }
    }
}

// covers: BAT-SIE-001, BAT-SIE-007, BAT-SIE-011, BAT-SIE-013, BAT-SIE-015, BAT-SIE-026, BAT-TER-002
#[test]
fn random_legal_sod_heroes_besiege_every_town() {
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
        let special: Vec<CreatureTypeId> = pool
            .iter()
            .copied()
            .filter(|&c| {
                let t = sod.creature(c).unwrap();
                t.flags.contains(CreatureFlag::Catapult)
                    || t.flags.contains(CreatureFlag::Flying)
                    || t.flags.contains(CreatureFlag::TwoHex)
                    || sod.abilities(c).iter().any(|a| {
                        matches!(
                            a,
                            Ability::Teleports
                                | Ability::WallShots { .. }
                                | Ability::SpellCaster { .. }
                                | Ability::RandomSpellCaster { .. }
                                | Ability::HelpfulSpellCaster { .. }
                                | Ability::StackRaiser { .. }
                                | Ability::CorpseRaiser { .. }
                        )
                    })
            })
            .collect();
        let quake_and_teleport =
            ["base:earthquake", "base:teleport"].map(|k| sod.spell_by_key(k).unwrap());
        let factions: Vec<_> = sod.sieges().iter().map(|s| s.faction).collect();
        let battlefields: Vec<_> = sod.battlefields().iter().map(|b| b.key.clone()).collect();
        let mut rng = Rng::from_seed(77);
        let mut legal = LegalCommands::new();
        let mut targets = CastTargets::new();
        let mut seconds = CastTargets::new();
        for game in 0..60u64 {
            let details = [details(&sod, &mut rng), details(&sod, &mut rng)];
            let heroes = details
                .clone()
                .map(|d| derive_battle_hero(&d, &sod).unwrap());
            // Three games in four at a town: the record of game's faction, each fort level,
            // each moat, any structures; else a special battlefield.
            let (siege, battlefield) = if game % 4 != 3 {
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
                    morale: i8::try_from(rng.below(4)).unwrap() - 1,
                    luck: i8::try_from(rng.below(3)).unwrap(),
                };
                (Some(town), None)
            } else {
                (
                    None,
                    Some(battlefields[pick(&mut rng, battlefields.len())].clone()),
                )
            };
            let setup = BattleSetup {
                attacker: army(&mut rng, &pool, &special),
                defender: army(&mut rng, &pool, &special),
                obstacles: vec![],
                battlefield,
                anti_magic: false,
                siege,
                war_machines: [
                    war_machines(&details[0], &sod),
                    war_machines(&details[1], &sod),
                ],
                heroes: heroes.map(Some),
                terrain: None,
                formations: [Formation::Loose, Formation::Tight].map(|_| {
                    if rng.below(2) == 0 {
                        Formation::Loose
                    } else {
                        Formation::Tight
                    }
                }),
                funds: [rng.next_u32(), rng.next_u32()],
                difficulty: u8::try_from(rng.below(5)).unwrap(),
                human: [rng.below(2) == 0, rng.below(2) == 0],
            };
            let mut state = BattleState::new(&sod, &setup, game)
                .unwrap_or_else(|e| panic!("{profile} game {game}: the setup is refused: {e:?}"));
            for step in 0..400u32 {
                let Some(side) = state.side_to_act() else {
                    reach.over += 1;
                    break;
                };
                if step % 20 == 0 {
                    probe(&state, &sod, side, &quake_and_teleport, game);
                }
                let spells: Vec<SpellId> = castable_spells(&state, &sod, side).iter().collect();
                creature_cast_targets(&state, &sod, side, &mut targets);
                let roll = rng.below(100);
                let cmd = if !spells.is_empty() && roll < 40 {
                    let wanted: Vec<SpellId> = spells
                        .iter()
                        .copied()
                        .filter(|s| quake_and_teleport.contains(s))
                        .collect();
                    let spell = if !wanted.is_empty() && rng.below(2) == 0 {
                        wanted[pick(&mut rng, wanted.len())]
                    } else {
                        spells[pick(&mut rng, spells.len())]
                    };
                    cast_targets(&state, &sod, side, spell, &mut targets);
                    assert!(!targets.is_empty(), "castable {spell:?} without a target");
                    let target = targets.as_slice()[pick(&mut rng, targets.len())];
                    second_targets(&state, &sod, side, (spell, target), &mut seconds);
                    let second = seconds.as_slice()[pick(&mut rng, seconds.len())];
                    Command::Cast {
                        spell,
                        target,
                        second,
                    }
                } else if !targets.is_empty() && roll < 55 {
                    Command::CreatureCast {
                        target: targets.as_slice()[pick(&mut rng, targets.len())],
                    }
                } else {
                    legal_commands(&state, &sod, side, &mut legal);
                    let staying: Vec<Command> = legal
                        .iter()
                        .filter(|c| !c.leaves_battle())
                        .copied()
                        .collect();
                    let walls: Vec<Command> = staying
                        .iter()
                        .copied()
                        .filter(|c| matches!(c, Command::ShootWall { .. }))
                        .collect();
                    let strikes: Vec<Command> = staying
                        .iter()
                        .copied()
                        .filter(|c| matches!(c, Command::Attack { .. } | Command::Shoot { .. }))
                        .collect();
                    if !walls.is_empty() && rng.below(3) != 0 {
                        walls[pick(&mut rng, walls.len())]
                    } else if !strikes.is_empty() && rng.below(2) == 0 {
                        strikes[pick(&mut rng, strikes.len())]
                    } else {
                        staying[pick(&mut rng, staying.len())]
                    }
                };
                let mut events = Vec::new();
                apply(&mut state, &sod, side, cmd, &mut events).unwrap_or_else(|e| {
                    panic!("{profile} game {game}: the listed {cmd:?} is rejected: {e:?}")
                });
                for e in &events {
                    match e.kind {
                        h3_battle::EventKind::WallShot { .. } => reach.wall_shots += 1,
                        h3_battle::EventKind::WallQuaked { .. } => reach.quakes += 1,
                        h3_battle::EventKind::Teleported { .. } => reach.teleports += 1,
                        h3_battle::EventKind::WallDestroyed { .. } => reach.destroyed += 1,
                        _ => {}
                    }
                }
            }
            for side in BattleSide::BOTH {
                let _ = observe(&state, &sod, side);
            }
            assert_eq!(state.outcome().is_some(), state.is_over());
        }
    }
    assert!(
        reach.wall_shots > 50
            && reach.quakes > 10
            && reach.teleports > 10
            && reach.destroyed > 20
            && reach.over > 20,
        "{reach:?}"
    );
}
