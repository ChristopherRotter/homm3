//! Robustness of the hero pipeline on the harvested SoD packs (E04a T12c review): random legal
//! hero details of every hero type, skill and artifact (scrolls with random spells, E04b T8a)
//! derive into battle heroes (HER-BAT-001 to HER-BAT-011) whose battles start and play to the end
//! with every legal command accepted (BAT-HER-001); the artifacts that give spells give the
//! original's (HER-SPL-002, HER-SPL-003).

use h3_battle::{BattleState, LegalCommands, NoopSink, apply, legal_commands, observe};
use h3_content::{
    ArtifactTypeId, CreatureFlag, CreatureTypeId, Formation, HeroEffect, HeroTypeId, Ruleset,
    SkillId, SpellCategory, SpellId,
};
use h3_contracts::{Army, ArmyStack, BattleSetup, BattleSide, MAX_BATTLE_EFFECTS};
use h3_heroes::{
    ArtifactInstance, HeroDetails, available_spells, derive_battle_hero, spell_masteries, validate,
    war_machines,
};
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
