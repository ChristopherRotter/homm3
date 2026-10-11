//! Robustness of the hero pipeline on the harvested SoD packs (E04a T12c review): random legal
//! hero details of every hero type, skill and artifact (scrolls with random spells, E04b T8a)
//! derive into battle heroes (HER-BAT-001 to HER-BAT-011) whose battles start and play to the end
//! with every legal command accepted (BAT-HER-001), with the heroes' and the stacks' casts among
//! the commands in both quirk profiles (E04b T11b review); the artifacts that give spells give
//! the original's (HER-SPL-002, HER-SPL-003).

use h3_battle::{
    BattleState, CastTarget, CastTargets, Command, Hex, LegalCommands, NoopSink, apply,
    cast_targets, castable_spells, creature_cast_targets, generate_obstacles, legal_commands,
    observe, second_targets,
};
use h3_content::{
    Ability, ArtifactTypeId, CreatureFlag, CreatureTypeId, Formation, HeroEffect, HeroTypeId,
    Ruleset, SkillId, SpellCategory, SpellId, TerrainId,
};
use h3_contracts::{
    Army, ArmyStack, BattleHero, BattleSetup, BattleSide, MAX_BATTLE_EFFECTS, Obstacle,
};
use h3_heroes::{
    ArtifactInstance, HeroDetails, available_spells, derive_battle_hero, spell_masteries, validate,
    war_machines,
};
use h3_kernel::{Rng, StreamId};

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
        if validate(&d, r).is_err() {
            d.worn.pop();
        }
    }
    for _ in 0..pick(rng, 9) {
        let spell = hero_spells[pick(rng, hero_spells.len())];
        if !d.spells.contains(&spell) {
            d.spells.push(spell);
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

/// [`details`] of a hero that casts: it wears a Spell Book (HER-BAT-011), has learned about half
/// of the hero spells and has mana.
fn casting_details(r: &Ruleset, rng: &mut Rng) -> HeroDetails {
    let mut d = details(r, rng);
    let book = ArtifactInstance::from(r.artifact_by_key("base:spell_book").unwrap());
    if !d.worn.contains(&book) {
        d.worn.push(book);
    }
    for spell in (0..r.spells().len()).filter_map(SpellId::from_index) {
        let learnable = r.spell(spell).unwrap().category != SpellCategory::Ability;
        if learnable && rng.below(2) == 0 && !d.spells.contains(&spell) {
            d.spells.push(spell);
        }
    }
    d.mana = [20, 100, 500, 999][pick(rng, 4)];
    validate(&d, r).unwrap();
    d
}

// covers: BAT-MAG-034, BAT-GEO-003, BAT-GEO-004
/// E04b T11b review (F-robust-2) on a battlefield the engine generates (grass, the obstacle
/// stream of seed 888): its fourth obstacle blocks hexes 109 and 110 from its anchor 108, which
/// it does not block; the fifth is a one-hex obstacle placed on 108. In the default profile
/// (quirk BAT-QUIRK-3 on) the removal of the fourth clears hex 108 too, so no hex leads to the
/// fifth any more: once the others are removed, Remove Obstacle has no target and is not
/// castable (it was, without a target).
#[test]
fn remove_obstacle_keeps_a_target_while_it_is_castable() {
    let r = h3_pack::compile(
        &[h3_pack::SOD_PACK, h3_pack::AB_PACK, h3_pack::BASE_PACK],
        "base:faithful",
    )
    .unwrap();
    let remove = r.spell_by_key("base:remove_obstacle").unwrap();
    let grass = r.terrain_by_key("base:grass").unwrap();
    let mut stream = Rng::for_stream(888, StreamId::OBSTACLES);
    let obstacles = generate_obstacles(&r, grass, &mut stream);
    let piece = |hexes: &[u8], anchor| Obstacle {
        hexes: hexes.to_vec(),
        anchor: Some(anchor),
    };
    assert_eq!(
        obstacles[3..],
        [piece(&[109, 110], 108), piece(&[108], 108)]
    );
    // Ten Pikemen each; the attacker's hero knows Remove Obstacle and has mana for every cast.
    let army = || Army {
        stacks: vec![ArmyStack {
            slot: 0,
            creature: r.creature_by_key("base:pikeman").unwrap(),
            count: 10,
        }],
    };
    let mut hero = BattleHero::new(HeroTypeId::from_index(0).unwrap());
    hero.magic.mana = 999;
    hero.magic.spell_book = Some(vec![remove]);
    let setup = BattleSetup {
        attacker: army(),
        defender: army(),
        obstacles,
        heroes: [Some(hero), None],
        terrain: Some(grass),
        war_machines: [vec![], vec![]],
        formations: [Formation::Loose; 2],
        funds: [0; 2],
    };
    let mut state = BattleState::new(&r, &setup, 1).unwrap();
    let side = BattleSide::Attacker;
    let mut targets = CastTargets::new();
    // A hex of the fourth obstacle first, then one of each other obstacle that can be reached,
    // a cast a round.
    for hex in [109, 122, 73, 80] {
        assert_eq!(state.side_to_act(), Some(side));
        let cast = Command::Cast {
            spell: remove,
            target: CastTarget::Hex(Hex(hex)),
            second: CastTarget::None,
        };
        apply(&mut state, &r, side, cast, &mut NoopSink).unwrap();
        // Both stacks defend: the next round, the hero may cast again.
        apply(&mut state, &r, side, Command::Defend, &mut NoopSink).unwrap();
        apply(
            &mut state,
            &r,
            side.opponent(),
            Command::Defend,
            &mut NoopSink,
        )
        .unwrap();
    }
    // The fifth obstacle is left, blocking hex 108, and no hex leads to it.
    assert_eq!(state.removable_obstacles().len(), 1);
    assert_eq!(state.removable_obstacles()[0].hexes(), [Hex(108)]);
    cast_targets(&state, &r, side, remove, &mut targets);
    assert!(targets.is_empty(), "{:?}", targets.as_slice());
    assert!(!castable_spells(&state, &r, side).contains(remove));
}

// covers: HER-BAT-011, BAT-HER-001, BAT-MAG-001, BAT-MAG-003, BAT-MAG-048, BAT-GEO-004
/// Robustness with magic (E04b T11b review): in both quirk profiles, battles of derived SoD
/// heroes with spell books and mana on generated battlefields play with the heroes' casts (a
/// castable spell at one of its listed targets), the stacks' casts and the legal commands, each
/// accepted; armies of up to `u32::MAX` creatures. Fixed seeds: a failure reproduces.
#[test]
fn random_legal_sod_heroes_cast_and_play() {
    let mut casts = (0, 0);
    for profile in ["base:faithful", "base:strict-fixed"] {
        let sod = h3_pack::compile(
            &[h3_pack::SOD_PACK, h3_pack::AB_PACK, h3_pack::BASE_PACK],
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
        // The creatures that cast on command or raise (BAT-MAG-048 to BAT-MAG-052).
        let casters: Vec<CreatureTypeId> = pool
            .iter()
            .copied()
            .filter(|&c| {
                sod.abilities(c).iter().any(|a| {
                    matches!(
                        a,
                        Ability::SpellCaster { .. }
                            | Ability::RandomSpellCaster { .. }
                            | Ability::HelpfulSpellCaster { .. }
                            | Ability::StackRaiser { .. }
                            | Ability::CorpseRaiser { .. }
                    )
                })
            })
            .collect();
        let mut rng = Rng::from_seed(46);
        let mut legal = LegalCommands::new();
        let mut targets = CastTargets::new();
        let mut seconds = CastTargets::new();
        for game in 0..40u64 {
            let details = [
                casting_details(&sod, &mut rng),
                casting_details(&sod, &mut rng),
            ];
            let heroes = details
                .clone()
                .map(|d| derive_battle_hero(&d, &sod).unwrap());
            assert!(heroes.iter().all(|h| h.magic.spell_book.is_some()));
            // Three battles in four on a terrain with its random obstacles (BAT-GEO-004).
            let terrain = (rng.below(4) != 0)
                .then(|| TerrainId::from_index(pick(&mut rng, sod.terrains().len())))
                .flatten();
            let obstacles = terrain.map_or_else(Vec::new, |t| {
                let mut stream = Rng::for_stream(game, StreamId::OBSTACLES);
                generate_obstacles(&sod, t, &mut stream)
            });
            // Each army's first stack is a caster in half the battles.
            let mut armies = [army(&mut rng, &pool), army(&mut rng, &pool)];
            for army in &mut armies {
                if rng.below(2) == 0 {
                    army.stacks[0].creature = casters[pick(&mut rng, casters.len())];
                }
            }
            let [attacker, defender] = armies;
            let setup = BattleSetup {
                attacker,
                defender,
                obstacles,
                war_machines: [
                    war_machines(&details[0], &sod),
                    war_machines(&details[1], &sod),
                ],
                heroes: heroes.map(Some),
                terrain,
                formations: [Formation::Loose; 2],
                funds: [rng.next_u32(), rng.next_u32()],
            };
            let mut state = BattleState::new(&sod, &setup, game).unwrap();
            for _ in 0..300 {
                let Some(side) = state.side_to_act() else {
                    break;
                };
                let spells: Vec<SpellId> = castable_spells(&state, &sod, side).iter().collect();
                creature_cast_targets(&state, &sod, side, &mut targets);
                let roll = rng.below(100);
                let cmd = if !spells.is_empty() && roll < 50 {
                    let spell = spells[pick(&mut rng, spells.len())];
                    cast_targets(&state, &sod, side, spell, &mut targets);
                    assert!(!targets.is_empty(), "castable {spell:?} without a target");
                    let target = targets.as_slice()[pick(&mut rng, targets.len())];
                    second_targets(&state, &sod, side, (spell, target), &mut seconds);
                    let second = seconds.as_slice()[pick(&mut rng, seconds.len())];
                    casts.0 += 1;
                    Command::Cast {
                        spell,
                        target,
                        second,
                    }
                } else if !targets.is_empty() && roll < 85 {
                    casts.1 += 1;
                    Command::CreatureCast {
                        target: targets.as_slice()[pick(&mut rng, targets.len())],
                    }
                } else {
                    legal_commands(&state, &sod, side, &mut legal);
                    // No leaving, so that the battles run on.
                    let staying: Vec<_> = legal.iter().filter(|c| !c.leaves_battle()).collect();
                    *staying[pick(&mut rng, staying.len())]
                };
                apply(&mut state, &sod, side, cmd, &mut NoopSink).unwrap_or_else(|e| {
                    panic!("{profile} game {game}: the listed {cmd:?} is rejected: {e:?}")
                });
            }
            for side in BattleSide::BOTH {
                let _ = observe(&state, &sod, side);
            }
            assert_eq!(state.outcome().is_some(), state.is_over());
        }
    }
    // The battles did cast (about two thousand casts of the heroes, two hundred of the stacks).
    assert!(casts.0 > 1000 && casts.1 > 100, "{casts:?}");
}

/// Hero details of Solmyr wearing `worn` (keys), knowing Magic Arrow, with `skills`.
fn wearing(sod: &Ruleset, worn: &[ArtifactInstance], skills: &[(&str, u8)]) -> HeroDetails {
    HeroDetails {
        hero_type: sod.hero_type_by_key("base:solmyr").unwrap(),
        level: 10,
        experience: 0,
        primary: [1, 1, 1, 1],
        skills: skills
            .iter()
            .map(|&(k, l)| (sod.skill_by_key(k).unwrap(), l))
            .collect(),
        worn: worn.to_vec(),
        backpack: vec![],
        spells: vec![sod.spell_by_key("base:magic_arrow").unwrap()],
        mana: 0,
    }
}

// covers: HER-SPL-002, HER-SPL-003, HER-ART-003
#[test]
fn the_harvested_artifacts_give_the_originals_spells() {
    let sod = h3_pack::compile(
        &[h3_pack::SOD_PACK, h3_pack::AB_PACK, h3_pack::BASE_PACK],
        "base:faithful",
    )
    .unwrap();
    let art = |k: &str| ArtifactInstance::from(sod.artifact_by_key(k).unwrap());
    let spell = |k: &str| sod.spell_by_key(k).unwrap();
    let school = |k: &str| {
        let i = sod.spell_schools().iter().position(|s| s.key.as_str() == k);
        h3_content::SchoolId::from_index(i.unwrap()).unwrap()
    };
    let bolt = spell("sod:titans_lightning_bolt");
    let have =
        |worn: &[ArtifactInstance]| available_spells(&wearing(&sod, worn, &[]), &sod).unwrap();
    // The learned spell alone.
    assert_eq!(have(&[]), [spell("base:magic_arrow")]);
    // A Tome: every adventure and combat spell of its school, Titan's Lightning Bolt (air,
    // level 5) excluded (installation.md §7.22, 0x4D9532).
    let air = have(&[art("base:tome_of_air_magic")]);
    let expected: Vec<SpellId> = (0..sod.spells().len())
        .filter_map(SpellId::from_index)
        .filter(|&s| {
            let t = sod.spell(s).unwrap();
            s == spell("base:magic_arrow")
                || (t.category != SpellCategory::Ability
                    && t.schools.contains(&school("base:air"))
                    && s != bolt)
        })
        .collect();
    assert_eq!(air, expected);
    assert!(air.contains(&spell("base:fly")) && air.contains(&spell("base:chain_lightning")));
    // Spellbinder's Hat: every level-5 spell but Titan's Lightning Bolt.
    let hat = have(&[art("base:spellbinders_hat")]);
    assert!(hat.contains(&spell("base:implosion")) && hat.contains(&spell("base:fly")));
    assert!(!hat.contains(&spell("base:armageddon")) && !hat.contains(&spell("base:age")));
    assert!(!hat.contains(&bolt));
    assert!(
        hat.iter()
            .all(|&s| s == spell("base:magic_arrow") || sod.spell(s).unwrap().level == 5)
    );
    // Titan's Thunder names the bolt; Armageddon's Blade gives Armageddon at Expert without any
    // fire magic, the Sea Captain's Hat the two boat spells.
    assert!(have(&[art("sod:titans_thunder")]).contains(&bolt));
    let blade = wearing(&sod, &[art("ab:armageddons_blade")], &[]);
    assert!(
        available_spells(&blade, &sod)
            .unwrap()
            .contains(&spell("base:armageddon"))
    );
    assert_eq!(
        spell_masteries(&blade, &sod).unwrap(),
        [(spell("base:armageddon"), 3)]
    );
    assert_eq!(
        have(&[art("base:sea_captains_hat")]),
        [
            spell("base:summon_boat"),
            spell("base:scuttle_boat"),
            spell("base:magic_arrow")
        ]
    );
    // A worn scroll gives its spell; one in the backpack does not.
    let scroll = ArtifactInstance::scroll(
        sod.artifact_by_key("base:spell_scroll").unwrap(),
        spell("base:implosion"),
    );
    assert!(have(&[scroll]).contains(&spell("base:implosion")));
    let mut carried = wearing(&sod, &[], &[]);
    carried.backpack.push(scroll);
    assert_eq!(
        available_spells(&carried, &sod).unwrap(),
        [spell("base:magic_arrow")]
    );
    // Masteries: Basic Fire magic and the Blade: Armageddon (fire) at Expert, the other fire
    // spells at Basic; Magic Arrow (every school) at Basic too.
    let fiery = wearing(
        &sod,
        &[art("ab:armageddons_blade")],
        &[("base:fire_magic", 1)],
    );
    let masteries = spell_masteries(&fiery, &sod).unwrap();
    assert!(masteries.contains(&(spell("base:armageddon"), 3)));
    assert!(masteries.contains(&(spell("base:magic_arrow"), 1)));
    assert!(masteries.contains(&(spell("base:fireball"), 1)));
    // The battle hero's book: the available spells, only with a worn Spell Book.
    let booked = wearing(
        &sod,
        &[art("base:spell_book"), art("base:tome_of_air_magic")],
        &[("base:air_magic", 2)],
    );
    let hero = derive_battle_hero(&booked, &sod).unwrap();
    assert_eq!(hero.magic.spell_book, Some(air));
    assert!(
        hero.magic
            .masteries
            .contains(&(spell("base:chain_lightning"), 2))
    );
    let unbooked = wearing(&sod, &[art("base:tome_of_air_magic")], &[]);
    assert_eq!(
        derive_battle_hero(&unbooked, &sod)
            .unwrap()
            .magic
            .spell_book,
        None
    );
}
