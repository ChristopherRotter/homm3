//! Tests of the simulated combat's sides and melee (E31 T2b): a side built from its army, its
//! hero and the opponent (the factor, Tactics, the walls, the bonuses, the shares, the order), on
//! a core of the crate's test content with fight values and hit points; the losses, the damage,
//! the stand-off, the lookahead and the run on sides built by hand, their numbers from an IEEE
//! double model of the reading (each case's comment gives its sums).

use alloc::vec;
use alloc::vec::Vec;

use h3_content::{
    ArtifactDef, ArtifactSlotDef, ArtifactTypeId, ContentPack, CreatureDef, CreatureInfoDef,
    HeroEffect, HeroTypeId, SecondarySkillDef, SkillId, SpellCategory, SpellDef, SpellId,
};
use h3_content_battle::{AbilityDef, BattleCreatureDef, BattleSpellDef};
use h3_heroes::{ArtifactInstance, OfferLevels, wearing};
use proptest::prelude::*;

use super::*;
use crate::tests::{battle_part_with, core_pack, hero_type, key};

/// The test content: the crate's core with fight values (the imp 100, 4 hit points, speed 5;
/// the gog 300, 13, 2, alive; the cart 0, 100, 0) and three creatures more (the archer 150, 10,
/// 4, a living shooter; the mage 400, 25, 5, a living shooter without the melee penalty that
/// strikes twice, with `no_wall_penalty`; the bat 120, 8, 9, a living flyer); the skills Tactics,
/// Ballistics and Archery (10, 25, 50 %); artifacts of speed +1 and +2, hit points +2, Elixir's
/// quarter for the living, a bow (5 %, needing Archery); the hero types `t:fast` (speed +2) and
/// `t:shot` (Archery specialist, 5 % a level); the spell `t:quake` with the battle part's
/// `Earthquake`. `profile` the core's (`t:single` for the original's single-precision rates).
fn content(profile: &str) -> (Ruleset, BattleRuleset) {
    let mut pack: ContentPack = core_pack(false);
    let values = [("t:imp", 100, &[][..]), ("t:gog", 300, &["alive"][..])];
    for (k, fight, flags) in values {
        let creature = pack.creatures.iter_mut().find(|c| c.key == key(k)).unwrap();
        creature.info.fight_value = fight;
        creature.info.flags = flags.iter().map(|&f| f.into()).collect();
    }
    let creature = |k: &str, speed, fight, flags: &[&str]| CreatureDef {
        key: key(k),
        speed,
        info: CreatureInfoDef {
            fight_value: fight,
            flags: flags.iter().map(|&f| f.into()).collect(),
            ..CreatureInfoDef::default()
        },
    };
    pack.creatures.extend([
        creature("t:archer", 4, 150, &["shooter", "alive"]),
        creature(
            "t:mage",
            5,
            400,
            &["shooter", "no_melee_penalty", "strikes_twice", "alive"],
        ),
        creature("t:bat", 9, 120, &["flying", "alive"]),
    ]);
    let skill = |k: &str, levels: [HeroEffect<_, _, _, _, _, _>; 3]| SecondarySkillDef {
        key: key(k),
        name: "Skill".into(),
        levels: levels.map(|effect| vec![effect]),
    };
    pack.skills.extend([
        skill(
            "t:tactics",
            [1, 2, 3].map(|level| HeroEffect::Tactics { level }),
        ),
        skill(
            "t:ballistics",
            [1, 2, 3].map(|level| HeroEffect::Ballistics { level }),
        ),
        skill(
            "t:archery",
            [1000, 2500, 5000].map(|bp| HeroEffect::RangedDamage { bp }),
        ),
    ]);
    pack.artifact_slots.push(ArtifactSlotDef {
        key: key("t:misc"),
        name: "Slot".into(),
        count: 5,
        backpack: true,
    });
    let artifact = |k: &str, effect| ArtifactDef {
        key: key(k),
        name: "Artifact".into(),
        class: h3_content::ArtifactClass::Treasure,
        cost: vec![],
        slot: Some(key("t:misc")),
        war_machine: None,
        needs_skill: None,
        effects: vec![effect],
        parts: vec![],
    };
    let stat = |stat, amount| HeroEffect::CreatureStat {
        creatures: CreatureSelector::All,
        stat,
        amount,
    };
    pack.artifacts.extend([
        artifact("t:swift", stat(CreatureStat::Speed, 1)),
        artifact("t:cape", stat(CreatureStat::Speed, 2)),
        artifact("t:vital", stat(CreatureStat::HitPoints, 2)),
        artifact(
            "t:elixir",
            HeroEffect::CreatureStatPercent {
                creatures: CreatureSelector::Flag(CreatureFlag::Alive),
                stat: CreatureStat::HitPoints,
                bp: 2500,
            },
        ),
        ArtifactDef {
            needs_skill: Some(key("t:archery")),
            ..artifact("t:bow", HeroEffect::RangedDamage { bp: 500 })
        },
    ]);
    pack.heroes.extend([
        HeroTypeDef {
            specialty: vec![stat(CreatureStat::Speed, 2)],
            ..hero_type("t:fast", &[], &[])
        },
        HeroTypeDef {
            specialty: vec![HeroEffect::SkillSpecialty {
                skill: key("t:archery"),
                bp_per_level: 500,
            }],
            ..hero_type("t:shot", &[], &[])
        },
    ]);
    pack.spells.push(SpellDef {
        key: key("t:quake"),
        name: "Quake".into(),
        category: SpellCategory::Combat,
        level: 3,
        schools: vec![key("t:air")],
        cost: [20, 20, 20, 20],
        power: 0,
        effect: [1, 1, 2, 3],
        granted_by_name_only: false,
    });
    let core = Arc::new(Ruleset::compile(&[pack], profile).unwrap());
    let battle = battle_part_with(&core, 7, |battle| {
        battle.creatures.retain(|c| c.key != key("t:elf"));
        let row = |k: &str, hit_points, shots, abilities| BattleCreatureDef {
            key: key(k),
            hit_points,
            shots,
            abilities,
            ..battle.creatures[0].clone()
        };
        let rows = [
            row("t:archer", 10, 12, vec![]),
            row("t:mage", 25, 24, vec![AbilityDef::NoWallPenalty]),
            row("t:bat", 8, 0, vec![]),
        ];
        battle.creatures.extend(rows);
        let quake = BattleSpellDef {
            key: key("t:quake"),
            magic: SpellMagic {
                effects: vec![SpellEffect::Earthquake],
                ..SpellMagic::default()
            },
            ..battle.spells[0].clone()
        };
        battle.spells.push(quake);
    });
    (Arc::unwrap_or_clone(core), battle)
}

use alloc::sync::Arc;
use h3_content::HeroTypeDef;
use h3_content_battle::SpellMagic;

fn creature(core: &Ruleset, k: &str) -> CreatureTypeId {
    core.creature_by_key(k).unwrap()
}

fn skill(core: &Ruleset, k: &str) -> SkillId {
    core.skill_by_key(k).unwrap()
}

fn artifact(core: &Ruleset, k: &str) -> ArtifactInstance {
    let artifact: ArtifactTypeId = core.artifact_by_key(k).unwrap();
    ArtifactInstance {
        artifact,
        spell: None,
    }
}

/// A hero of type `ty` with Attack and Defense `primary`, `skills` (key, level), wearing
/// `items`, knowing `spells`.
fn details(
    core: &Ruleset,
    ty: &str,
    primary: [u8; 2],
    skills: &[(&str, u8)],
    items: &[&str],
    spells: &[&str],
) -> HeroDetails {
    let hero_type: HeroTypeId = core.hero_type_by_key(ty).unwrap();
    let items: Vec<ArtifactInstance> = items.iter().map(|k| artifact(core, k)).collect();
    HeroDetails {
        hero_type,
        level: 1,
        experience: 0,
        offered: OfferLevels::default(),
        primary: [primary[0], primary[1], 1, 1],
        skills: skills.iter().map(|&(k, l)| (skill(core, k), l)).collect(),
        worn: wearing(core, &items),
        backpack: vec![],
        spells: spells
            .iter()
            .map(|k| -> SpellId { core.spell_by_key(k).unwrap() })
            .collect(),
        mana: 0,
    }
}

fn stack(core: &Ruleset, slot: u8, k: &str, count: u32) -> ArmyStack {
    ArmyStack {
        slot,
        creature: creature(core, k),
        count,
    }
}

/// `v` as an x87 value exactly: a numerator over a power of two.
fn exact(num: u32, den: u32) -> X87 {
    X87::ratio(num, den, DOUBLE_BITS).unwrap()
}

fn fifth() -> X87 {
    exact(1, 5)
}

/// The shares of a walker (0.2 of melee, 1 of the stand-off, no shots).
fn walker(slot: u8, count: i32, speed: i32, contact: i32, unit: i32) -> Stack {
    Stack {
        slot,
        count,
        speed,
        melee: fifth(),
        hold: X87::ONE,
        shot: X87::ZERO,
        contact,
        unit,
        value: unit.wrapping_mul(count),
    }
}

/// A shooter without a hero (0.1 of melee, 0.7 of the stand-off, 0.2 of shots).
fn shooter(slot: u8, count: i32, speed: i32, unit: i32) -> Stack {
    Stack {
        melee: exact(1, 10),
        hold: exact(7, 10),
        shot: fifth(),
        ..walker(slot, count, speed, 0, unit)
    }
}

/// A side of `stacks` as the build leaves it: by unit value, ties in slot order, the strength
/// their sum.
fn side_of(mut stacks: Vec<Stack>) -> Side {
    let _ = sort_by_unique_key(&mut stacks, |s| (s.unit, s.slot));
    let strength = stacks.iter().map(|s| s.value).sum();
    Side { stacks, strength }
}

/// The stacks' (slot, count, value), in the side's order.
fn left(side: &Side) -> Vec<(u8, i32, i32)> {
    side.stacks
        .iter()
        .map(|s| (s.slot, s.count, s.value))
        .collect()
}

// --- the side (CPU-SIM-001) ---

// covers: CPU-SIM-001
#[test]
fn a_side_without_heroes_takes_the_fight_values_and_the_plain_shares() {
    let (core, battle) = content("t:faithful");
    let content = Content {
        core: &core,
        battle: &battle,
    };
    let army = [
        stack(&core, 0, "t:imp", 10),
        stack(&core, 2, "t:archer", 4),
        stack(&core, 1, "t:gog", 3),
    ];
    let side = side(content, &army, X87::ONE, (None, None), FortLevel::None);
    // Imp (19 / 5 = 3 rounds), archer (a shooter), gog ((2 + 14) / 2 = 8, at most 4): weakest
    // unit first.
    assert_eq!(
        side.stacks,
        vec![
            walker(0, 10, 5, 3, 100),
            shooter(2, 4, 4, 150),
            walker(1, 3, 2, 4, 300)
        ]
    );
    assert_eq!(side.strength, 2500);
}

// covers: CPU-SIM-001
#[test]
fn a_heros_skills_bonuses_and_tactics_scale_its_side() {
    let (core, battle) = content("t:faithful");
    let content = Content {
        core: &core,
        battle: &battle,
    };
    // Attack 10, Defense 5, Expert Tactics, speed +3 and +2 (the type's), hit points +2 and the
    // Elixir; against Attack 3, Defense 2 and Basic Tactics.
    let own = details(
        &core,
        "t:fast",
        [10, 5],
        &[("t:tactics", 3)],
        &["t:swift", "t:cape", "t:vital", "t:elixir"],
        &[],
    );
    let enemy = details(&core, "t:orrin", [3, 2], &[("t:tactics", 1)], &[], &[]);
    let (own, enemy) = (HeroReads::of(&own, content), HeroReads::of(&enemy, content));
    assert_eq!((own.speed, own.tactics, own.attack), (5, 3, 10));
    let army = [
        stack(&core, 0, "t:imp", 10),
        stack(&core, 1, "t:gog", 3),
        stack(&core, 2, "t:archer", 4),
    ];
    let side = side(
        content,
        &army,
        exact(5, 4),
        (Some(&own), Some(&enemy)),
        FortLevel::None,
    );
    // The factor sqrt(1.4) × sqrt(1.1) × 1.25 = 1.5512092057488571; Tactics 2. The imp (not
    // living) 6 hit points of 4: ftol(sqrt(1.5) × 100 × f) = 189, speed 10, (10 + 10) / 10 = 2
    // rounds; the gog 2 + 3 of 13: ftol(sqrt(18 / 13) × 300 × f) = 547, speed 7, 17 / 7 = 2; the
    // archer 2 + 2 of 10: ftol(sqrt(1.4) × 150 × f) = 275, its shots the hero's Archery (none)
    // over 5.
    let unarmed = Stack {
        shot: X87::ZERO,
        ..shooter(2, 4, 9, 275)
    };
    assert_eq!(
        side.stacks,
        vec![walker(0, 10, 10, 2, 189), unarmed, walker(1, 3, 7, 2, 547)]
    );
    assert_eq!(side.strength, 1890 + 1100 + 1641);
    // Without an opponent its own Attack and Defense count, and no Tactics is taken away:
    // ftol(sqrt(1.5) × 100 × sqrt(1.5) × sqrt(1.25)) = 167, (10 + 8) / 10 = 1 round.
    let alone = side_with(content, &army, &own);
    assert_eq!((alone.stacks[0].unit, alone.stacks[0].contact), (167, 1));
}

fn side_with(content: Content<'_>, army: &[ArmyStack], hero: &HeroReads) -> Side {
    side(content, army, X87::ONE, (Some(hero), None), FortLevel::None)
}

// covers: CPU-SIM-001
#[test]
fn an_opponent_without_a_hero_takes_nothing_and_one_with_more_tactics_takes_all() {
    let (core, battle) = content("t:faithful");
    let content = Content {
        core: &core,
        battle: &battle,
    };
    let enemy = details(&core, "t:orrin", [0, 0], &[("t:tactics", 2)], &[], &[]);
    let enemy = HeroReads::of(&enemy, content);
    let army = [stack(&core, 0, "t:imp", 1)];
    // No hero of its own: the factor alone, Tactics 0 − 2 held at 0: 19 / 5 = 3 rounds.
    let side = side(
        content,
        &army,
        exact(3, 2),
        (None, Some(&enemy)),
        FortLevel::None,
    );
    assert_eq!(side.stacks, vec![walker(0, 1, 5, 3, 150)]);
}

// covers: CPU-SIM-001
#[test]
fn the_walls_are_the_forts_less_ballistics_none_with_earthquake() {
    let (core, battle) = content("t:faithful");
    let content = Content {
        core: &core,
        battle: &battle,
    };
    let hero = |skills: &[(&str, u8)], spells: &[&str]| {
        HeroReads::of(
            &details(&core, "t:orrin", [0, 0], skills, &[], spells),
            content,
        )
    };
    let (plain, basic, expert) = (
        hero(&[], &[]),
        hero(&[("t:ballistics", 1)], &[]),
        hero(&[("t:ballistics", 3)], &[]),
    );
    let quake = hero(&[], &["t:quake"]);
    assert!(quake.earthquake && !plain.earthquake);
    assert_eq!(walls(FortLevel::None, None), (false, 0));
    assert_eq!(walls(FortLevel::Fort, None), (true, 4));
    assert_eq!(walls(FortLevel::Castle, None), (true, 4));
    assert_eq!(walls(FortLevel::None, Some(&expert)), (false, 0));
    assert_eq!(walls(FortLevel::Fort, Some(&plain)), (true, 4));
    assert_eq!(walls(FortLevel::Fort, Some(&basic)), (true, 3));
    assert_eq!(walls(FortLevel::Fort, Some(&expert)), (true, 2));
    assert_eq!(walls(FortLevel::Citadel, Some(&expert)), (true, 2));
    assert_eq!(walls(FortLevel::Castle, Some(&expert)), (true, 3));
    assert_eq!(walls(FortLevel::Castle, Some(&quake)), (false, 0));
}

// covers: CPU-SIM-001
#[test]
fn a_walker_waits_for_the_walls_a_flyer_does_not_and_the_shots_are_halved() {
    let (core, battle) = content("t:faithful");
    let content = Content {
        core: &core,
        battle: &battle,
    };
    // Advanced Archery and the bow: f32(0.3) / 5 = 0.06000000238418579 (0x1.eb852p-5).
    let hero = details(
        &core,
        "t:orrin",
        [0, 0],
        &[("t:archery", 2)],
        &["t:bow"],
        &[],
    );
    let hero = HeroReads::of(&hero, content);
    let army = [
        stack(&core, 0, "t:imp", 1),
        stack(&core, 1, "t:bat", 1),
        stack(&core, 2, "t:archer", 1),
        stack(&core, 3, "t:mage", 1),
    ];
    let open = side(
        content,
        &army,
        X87::ONE,
        (Some(&hero), None),
        FortLevel::None,
    );
    let walled = side(
        content,
        &army,
        X87::ONE,
        (Some(&hero), None),
        FortLevel::Fort,
    );
    let contact = |side: &Side| side.stacks.iter().map(|s| s.contact).collect::<Vec<_>>();
    // By unit: imp 100, bat 120, archer 150, mage 400. The imp's 3 rounds raised to 4, the
    // bat's (9 + 14) / 9 = 2 kept.
    assert_eq!(contact(&open), vec![3, 2, 0, 0]);
    assert_eq!(contact(&walled), vec![4, 2, 0, 0]);
    let shot = exact(2_013_266, 1 << 25);
    let (archer, mage) = (&open.stacks[2], &open.stacks[3]);
    assert_eq!(
        (archer.shot, archer.melee, archer.hold),
        (shot, exact(1, 10), exact(7, 10))
    );
    // The mage strikes twice and has no melee penalty.
    assert_eq!(
        (mage.shot, mage.melee, mage.hold),
        (exact(2_013_266, 1 << 24), fifth(), X87::ONE)
    );
    // Facing walls the archer's shots are halved, the mage's (no wall penalty) are not.
    assert_eq!(walled.stacks[2].shot, exact(2_013_266, 1 << 26));
    assert_eq!(walled.stacks[3].shot, mage.shot);
    assert_eq!(walled.stacks[1].shot, X87::ZERO);
}

// covers: CPU-SIM-001
#[test]
fn a_specialists_archery_is_the_originals_single_precision_rate() {
    let (core, battle) = content("t:single");
    let content = Content {
        core: &core,
        battle: &battle,
    };
    let mut hero = details(&core, "t:shot", [0, 0], &[("t:archery", 1)], &[], &[]);
    // At level 4 the original's f32(f32(4 × 0.05 + 1) × 0.1) is 0.12000000476837158, the basis
    // points' 1 200 a single 0.11999999731779099.
    hero.level = 4;
    hero.experience = 3200;
    let rate = derive_battle_hero(&hero, BattleBonuses::NONE, &core)
        .unwrap()
        .damage
        .ranged;
    let bits = rate.single.unwrap().get();
    let reads = HeroReads::of(&hero, content);
    assert_eq!(reads.archery, X87::from_f32_bits(bits));
    assert_ne!(
        reads.archery,
        X87::ratio(rate.bp, 10_000, SINGLE_BITS).unwrap()
    );
}

// covers: CPU-SIM-001
#[test]
fn a_creature_of_speed_0_is_four_rounds_away_and_a_huge_stack_wraps() {
    let (core, battle) = content("t:faithful");
    let content = Content {
        core: &core,
        battle: &battle,
    };
    // The cart: speed 0 (the original divides by it), fight value 0.
    let army = [
        stack(&core, 0, "t:cart", 3),
        stack(&core, 1, "t:gog", 10_000_000),
    ];
    let side = side(content, &army, X87::ONE, (None, None), FortLevel::None);
    assert_eq!((side.stacks[0].contact, side.stacks[0].unit), (FAR, 0));
    // 300 × 10 000 000 wraps in 32 bits, as the code's imul.
    let wrapped = 300_i32.wrapping_mul(10_000_000);
    assert_eq!(side.stacks[1].value, wrapped);
    assert_eq!(side.strength, wrapped);
    // A hero that does not validate (Attack above the maximum) reads as one without skills.
    let mut hero = details(&core, "t:orrin", [0, 0], &[("t:tactics", 2)], &[], &[]);
    hero.primary[0] = 200;
    assert_eq!(HeroReads::of(&hero, content).tactics, 0);
}

// --- the losses (CPU-SIM-002) ---

/// The imp 10 × 100 (3 rounds), the archer 4 × 150 (a shooter), the gog 3 × 300 (4 rounds).
fn three() -> Side {
    side_of(vec![
        walker(0, 10, 5, 3, 100),
        shooter(2, 4, 4, 150),
        walker(1, 3, 2, 4, 300),
    ])
}

// covers: CPU-SIM-002
#[test]
fn a_loss_spreads_by_value_and_a_group_takes_its_share_first() {
    // 500 over 2 500: the imp ftol(1000 × 500 / 2500) = 200 (count 8), the archer
    // ftol(600 × 300 / 1500) = 120 (480: 4), the gog the 180 left (720: 3).
    let mut side = three();
    assert_eq!(side.spread(500, 0, FAR), 0);
    assert_eq!(left(&side), vec![(0, 8, 800), (2, 4, 480), (1, 3, 720)]);
    // 1 500 on the stacks in contact by round 3 (the imp alone): it dies, 500 left.
    let mut side = three();
    assert_eq!(side.spread(1500, 1, 3), 500);
    assert_eq!(left(&side), vec![(0, 0, 0), (2, 4, 600), (1, 3, 900)]);
    // Nothing in the range: the loss stays.
    assert_eq!(three().spread(700, 1, 2), 700);
}

// covers: CPU-SIM-002
#[test]
fn a_side_takes_damage_on_its_stacks_in_contact_then_on_all_or_is_wiped() {
    // 1 500 in round 3: the imp's 1 000, then 500 over the rest (the dead imp skipped): the
    // archer ftol(600 × 500 / 1500) = 200 (400: 3), the gog 300 (600: 2).
    let mut side = three();
    side.take_damage(1500, 3);
    assert_eq!(side.strength, 1000);
    assert_eq!(left(&side), vec![(0, 0, 0), (2, 3, 400), (1, 2, 600)]);
    // Its whole strength: wiped.
    let mut side = three();
    side.take_damage(2500, 0);
    assert_eq!(side.strength, 0);
    assert_eq!(left(&side), vec![(0, 0, 0), (2, 0, 0), (1, 0, 0)]);
}

// covers: CPU-SIM-002
#[test]
fn the_survivors_keep_their_slots_and_a_beaten_side_none() {
    let (core, _) = content("t:faithful");
    let army = [
        stack(&core, 0, "t:imp", 10),
        stack(&core, 1, "t:gog", 3),
        stack(&core, 2, "t:archer", 4),
        stack(&core, 7, "t:imp", 9),
    ];
    let mut side = three();
    side.take_damage(1500, 3);
    // The imps of slot 0 died; slot 7 is not simulated and keeps its count.
    assert_eq!(
        side.survivors(&army),
        vec![
            stack(&core, 1, "t:gog", 2),
            stack(&core, 2, "t:archer", 3),
            stack(&core, 7, "t:imp", 9)
        ]
    );
    side.wipe();
    assert_eq!(side.survivors(&army), vec![]);
}

// covers: CPU-SIM-002
#[test]
fn a_count_is_the_ceiling_of_the_value_left() {
    assert_eq!(ceiling(401, 100), 5);
    assert_eq!(ceiling(400, 100), 4);
    assert_eq!(ceiling(0, 100), 0);
    assert_eq!(ceiling(7, 0), 0);
}

// --- the damage and the stand-off (CPU-SIM-003) ---

// covers: CPU-SIM-003
#[test]
fn a_sides_damage_and_hold_value_truncate_after_each_stack() {
    let side = three();
    // Shots alone: 4 × 150 × 0.2 = 120; with the imp in contact by round 3: + 10 × 100 × 0.2;
    // all in melee: 0.2 × 1 000 + 0.1 × 600 + 0.2 × 900 = 440.
    assert_eq!(side.damage(0, false), 120);
    assert_eq!(side.damage(3, false), 320);
    assert_eq!(side.damage(FAR, true), 440);
    // 1 000 + 0.7 × 600 + 900.
    assert_eq!(side.hold(), 2320);
    // The truncation after each stack: 2 × 0.7 three times gives 1, 2, 3, not 4.
    let thin = side_of(vec![
        shooter(0, 2, 4, 1),
        shooter(1, 2, 4, 1),
        shooter(2, 2, 4, 1),
    ]);
    assert_eq!(thin.hold(), 3);
}

// covers: CPU-SIM-003
#[test]
fn the_stand_off_wipes_the_weaker_and_costs_the_winner() {
    // 1 420 against 900: the winner loses ftol(f32(900 / 1420 + 0.05) × 900) = 615.
    let (mut a, mut b) = (
        side_of(vec![walker(0, 10, 5, 3, 100), shooter(2, 4, 4, 150)]),
        side_of(vec![walker(1, 3, 2, 4, 300)]),
    );
    finish(&mut a, &mut b);
    assert_eq!((a.strength, b.strength), (985, 0));
    assert_eq!(left(&a), vec![(0, 7, 616), (2, 3, 369)]);
    // 200 against 900: the first wiped, the second loses ftol(f32(200 / 900 + 0.05) × 200) = 54.
    let (mut a, mut b) = (
        side_of(vec![walker(0, 2, 5, 3, 100)]),
        side_of(vec![walker(1, 3, 2, 4, 300)]),
    );
    finish(&mut a, &mut b);
    assert_eq!((a.strength, b.strength), (0, 846));
    // Equal: the first is wiped, and the second loses ftol(f32(1.05) × 300) = 314, more than its
    // 300.
    let (mut a, mut b) = (
        side_of(vec![walker(0, 3, 5, 3, 100)]),
        side_of(vec![walker(1, 1, 2, 4, 300)]),
    );
    finish(&mut a, &mut b);
    assert_eq!((a.strength, b.strength), (0, 0));
    // A side without a hold value: no stand-off.
    let (mut a, mut b) = (side_of(vec![]), three());
    finish(&mut a, &mut b);
    assert_eq!(b, three());
}

// --- the lookahead (CPU-SIM-004) and the run (CPU-SIM-005) ---

/// Two walkers (26 × 50 in 2 rounds, 25 × 100 in 3) against 4 × 150 in 1.
fn walkers_and_one() -> (Side, Side) {
    (
        side_of(vec![walker(0, 26, 6, 2, 50), walker(1, 25, 6, 3, 100)]),
        side_of(vec![walker(0, 4, 3, 1, 150)]),
    )
}

// covers: CPU-SIM-004
#[test]
fn a_side_engages_only_with_a_walker_in_reach_and_when_the_lookahead_says_now() {
    let (a, b) = walkers_and_one();
    // Round 1: none of the walkers reaches.
    assert!(!engages(&a, &b, 1));
    // Round 3: its walkers reach and engaging now scores best; round 2 waits.
    assert!(!engages(&a, &b, 2));
    assert!(engages(&a, &b, 3));
    // The lone walker reaches from round 1 and would lose: it waits.
    assert!(!engages(&b, &a, 1));
    // Dead walkers reach nothing.
    let mut dead = a.clone();
    dead.wipe();
    assert!(!engages(&dead, &b, 3));
}

// covers: CPU-SIM-005
#[test]
fn the_run_shoots_while_neither_engages_and_fights_in_contact_when_the_attacker_does() {
    // Rounds 1 and 2 nobody engages (no shooter: no damage), round 3 the attacker does: it
    // deals 0.2 × its stacks in contact, takes 0.2 × 600 = 120 on them first; the defender is
    // wiped (no stand-off).
    let (mut a, mut b) = walkers_and_one();
    run(&mut a, &mut b);
    assert_eq!(a.strength, 3680);
    assert_eq!(left(&a), vec![(0, 26, 1259), (1, 25, 2421)]);
    assert_eq!(b.strength, 0);
}

// covers: CPU-SIM-005
#[test]
fn the_defender_engaging_alone_fights_both_sides_with_all_their_stacks() {
    // Round 3 only the defender engages (0x426D17): both deal 0.2 of all their stacks.
    let (mut a, mut b) = (
        side_of(vec![walker(0, 28, 9, 1, 100)]),
        side_of(vec![walker(0, 18, 4, 1, 100), walker(1, 29, 4, 1, 300)]),
    );
    run(&mut a, &mut b);
    assert_eq!(a.strength, 0);
    assert_eq!(b.strength, 9856);
    assert_eq!(left(&b), vec![(0, 17, 1690), (1, 28, 8166)]);
}

// covers: CPU-SIM-005
#[test]
fn both_sides_engaging_fight_with_all_their_stacks() {
    let (mut a, mut b) = (
        side_of(vec![walker(1, 2, 5, 1, 50), shooter(0, 3, 5, 150)]),
        side_of(vec![walker(1, 14, 7, 1, 50), shooter(0, 15, 7, 300)]),
    );
    run(&mut a, &mut b);
    assert_eq!(a.strength, 0);
    assert_eq!(b.strength, 5135);
    assert_eq!(left(&b), vec![(1, 14, 692), (0, 15, 4443)]);
}

// covers: CPU-SIM-005, CPU-SIM-003
#[test]
fn three_rounds_end_in_the_stand_off() {
    // Rounds 1 and 2 the shots alone, round 3 the attacker engages; both still stand: the
    // stand-off (8 600 held against ftol(0.7 × 1 193) + 1 277 = 2 112) wipes the defender.
    let (mut a, mut b) = (
        side_of(vec![walker(1, 17, 6, 1, 100), walker(0, 28, 6, 3, 300)]),
        side_of(vec![shooter(0, 21, 6, 100), walker(1, 15, 6, 1, 150)]),
    );
    run(&mut a, &mut b);
    assert_eq!(a.strength, 7976);
    assert_eq!(left(&a), vec![(1, 14, 1344), (0, 23, 6632)]);
    assert_eq!(b.strength, 0);
}

// covers: CPU-SIM-005
#[test]
fn the_faster_side_casts_first() {
    let (slow, fast) = (
        side_of(vec![walker(0, 1, 3, 1, 10)]),
        side_of(vec![walker(0, 1, 7, 1, 10)]),
    );
    assert_eq!((slow.max_speed(), fast.max_speed()), (3, 7));
    let mut dead = fast.clone();
    dead.wipe();
    assert_eq!(dead.max_speed(), 0);
    // The casts change nothing until T2c (AI-STUB-7), in either order.
    let (mut a, mut b) = (slow.clone(), fast.clone());
    casts(&mut a, &mut b, 1);
    casts(&mut b, &mut a, 1);
    assert_eq!((a, b), (slow, fast));
}

fn arb_stack() -> impl Strategy<Value = Stack> {
    (0_u8..7, 1_i32..60, 1_i32..10, 0_i32..=FAR, 1_i32..400).prop_map(
        |(slot, count, speed, contact, unit)| {
            if contact == 0 {
                shooter(slot, count, speed, unit)
            } else {
                walker(slot, count, speed, contact, unit)
            }
        },
    )
}

fn arb_side() -> impl Strategy<Value = Side> {
    proptest::collection::vec(arb_stack(), 1..5).prop_map(|stacks| {
        let mut slots: Vec<Stack> = Vec::new();
        for stack in stacks {
            if slots.iter().all(|s| s.slot != stack.slot) {
                slots.push(stack);
            }
        }
        side_of(slots)
    })
}

proptest! {
    // covers: CPU-SIM-002, CPU-SIM-005
    /// A run is a function of its sides; it leaves each stack's count the ceiling of its value
    /// and never raises a value or a strength.
    #[test]
    fn a_run_is_repeatable_and_only_takes(a in arb_side(), b in arb_side()) {
        let (mut first, mut second) = (a.clone(), b.clone());
        run(&mut first, &mut second);
        let (mut again, mut other) = (a.clone(), b.clone());
        run(&mut again, &mut other);
        prop_assert_eq!(&first, &again);
        prop_assert_eq!(&second, &other);
        for (after, before) in [(&first, &a), (&second, &b)] {
            prop_assert!(after.strength >= 0 && after.strength <= before.strength);
            for (s, t) in after.stacks.iter().zip(&before.stacks) {
                prop_assert_eq!(s.count, ceiling(s.value, s.unit));
                prop_assert!(s.value >= 0 && s.value <= t.value);
            }
        }
    }
}
