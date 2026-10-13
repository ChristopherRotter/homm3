//! Batch 2a's direct-damage spells on the harvested SoD packs (E04b T11a, the fold-in; BAT-MAG-003
//! to BAT-MAG-006): R1 to R5 of `experiments/results/E04b-batch-2a.md` (SoD 4.0 Complete, plain
//! HD, 2026-10-06) replayed with the heroes the batch used, derived in the Heroes context. Also
//! E04b exit criterion 2's guard: every combat spell of the packs but Earthquake (E04c) and every
//! ability spell has effects the battle implements.

use h3_battle::{
    BattleState, CastTarget, CastTargets, Command, Event, EventKind, Field, StackId, apply,
    cast_targets,
};
use h3_content::{Formation, HeroTypeId, SchoolId, SpellCategory, SpellId, SpellScope};
use h3_content_battle::BattleRuleset;
use h3_contracts::{Army, ArmyStack, BattleEffect, BattleHero, BattleSetup, BattleSide, Rate};
use h3_heroes::{HeroDetails, derive_battle_hero};

fn sod() -> BattleRuleset {
    sod_with("base:faithful")
}

fn sod_with(profile: &str) -> BattleRuleset {
    h3_pack_battle::compile(
        &[
            h3_pack_battle::SOD_PACK,
            h3_pack_battle::AB_PACK,
            h3_pack_battle::BASE_PACK,
        ],
        profile,
    )
    .unwrap()
}

fn spell(r: &BattleRuleset, key: &str) -> SpellId {
    r.spell_by_key(key).unwrap()
}

/// `hero` at level 1 with attack and defense 0, `power` and `knowledge`, Basic Sorcery if
/// `sorcery`, wearing a Spell Book and `worn`, knowing `spells`, with full mana.
fn details(
    r: &BattleRuleset,
    hero: &str,
    power: u8,
    knowledge: u8,
    sorcery: bool,
    worn: &[&str],
    spells: &[&str],
) -> HeroDetails {
    HeroDetails {
        hero_type: r.hero_type_by_key(hero).unwrap(),
        level: 1,
        experience: 0,
        primary: [0, 0, power, knowledge],
        skills: if sorcery {
            vec![(r.skill_by_key("base:sorcery").unwrap(), 1)]
        } else {
            vec![]
        },
        worn: core::iter::once("base:spell_book")
            .chain(worn.iter().copied())
            .map(|k| r.artifact_by_key(k).unwrap().into())
            .collect(),
        backpack: vec![],
        spells: spells.iter().map(|k| spell(r, k)).collect(),
        mana: u16::from(knowledge) * 10,
    }
}

fn army(r: &BattleRuleset, stacks: &[(&str, u32)]) -> Army {
    Army {
        stacks: stacks
            .iter()
            .enumerate()
            .map(|(slot, &(key, count))| ArmyStack {
                slot: u8::try_from(slot).unwrap(),
                creature: r.creature_by_key(key).unwrap(),
                count,
            })
            .collect(),
    }
}

/// Ten Pikemen led by the hero of `details` against `defender` (no hero), the defender's war
/// machines `machines`; the battle runs until the Pikemen await a command.
fn battle(
    r: &BattleRuleset,
    details: &HeroDetails,
    defender: &[(&str, u32)],
    machines: &[&str],
) -> BattleState {
    led(
        r,
        derive_battle_hero(details, r).unwrap(),
        defender,
        machines,
    )
}

/// [`battle`] with the battle hero `hero` as it is.
fn led(
    r: &BattleRuleset,
    hero: BattleHero,
    defender: &[(&str, u32)],
    machines: &[&str],
) -> BattleState {
    let setup = BattleSetup {
        attacker: army(r, &[("base:pikeman", 10)]),
        defender: army(r, defender),
        obstacles: vec![],
        heroes: [Some(hero), None],
        terrain: None,
        battlefield: None,
        anti_magic: false,
        siege: None,
        war_machines: [
            vec![],
            machines
                .iter()
                .map(|k| r.creature_by_key(k).unwrap())
                .collect(),
        ],
        formations: [Formation::Loose; 2],
        funds: [0; 2],
    };
    let mut s = BattleState::new(r, &setup, 1).unwrap();
    while s.side_to_act() != Some(BattleSide::Attacker) {
        let side = s.side_to_act().unwrap();
        apply(&mut s, r, side, Command::Defend, &mut Vec::<Event>::new()).unwrap();
    }
    s
}

/// The id of the first stack of `creature`.
fn stack_of(s: &BattleState, r: &BattleRuleset, creature: &str) -> StackId {
    let c = r.creature_by_key(creature).unwrap();
    let i = s.stacks().iter().position(|t| t.creature == c).unwrap();
    StackId(u8::try_from(i).unwrap())
}

/// The id of the defender's first stack.
fn defenders_first(s: &BattleState) -> StackId {
    let i = s
        .stacks()
        .iter()
        .position(|t| t.side == BattleSide::Defender)
        .unwrap();
    StackId(u8::try_from(i).unwrap())
}

/// Where the attacker's hero may cast `spell` now.
fn targets(s: &BattleState, r: &BattleRuleset, spell: SpellId) -> Vec<CastTarget> {
    let mut out = CastTargets::new();
    cast_targets(s, r, BattleSide::Attacker, spell, &mut out);
    out.iter().copied().collect()
}

/// The attacker's hero casts `spell` at `target`: the damage of each stack struck.
fn cast(
    s: &mut BattleState,
    r: &BattleRuleset,
    spell: SpellId,
    target: CastTarget,
) -> Vec<(u8, u64)> {
    let mut events = Vec::<Event>::new();
    let cast = Command::Cast {
        spell,
        target,
        second: CastTarget::None,
    };
    apply(s, r, BattleSide::Attacker, cast, &mut events).unwrap();
    events
        .iter()
        .filter_map(|e| match e.kind {
            EventKind::SpellDamage { target, damage, .. } => Some((target.0, damage)),
            _ => None,
        })
        .collect()
}

// covers: BAT-MAG-004, BAT-MAG-005, BAT-MAG-003, HER-BAT-012
#[test]
fn batch_2a_r4_r5_ice_bolt_damage_in_the_originals_order() {
    let r = sod();
    let ice = spell(&r, "base:ice_bolt");
    // R4: Alagar (Ice Bolt specialist) with power 5 and Basic Sorcery. Damage (log): 119 to the
    // Peasants, 115 to the Wolf Riders, 17 to the Gold Golems, 230 to the Fire Elementals; the
    // Gold Dragon cannot be targeted (VCMI's order gives 118, 116, 16, 231).
    let alagar = details(&r, "base:alagar", 5, 4, true, &[], &["base:ice_bolt"]);
    for (creature, count, damage) in [
        ("ab:peasant", 200, 119),
        ("base:wolf_rider", 20, 115),
        ("base:gold_golem", 5, 17),
        ("base:fire_elemental", 10, 230),
    ] {
        let mut s = battle(
            &r,
            &alagar,
            &[(creature, count), ("base:gold_dragon", 1)],
            &[],
        );
        let target = stack_of(&s, &r, creature);
        assert!(!targets(&s, &r, ice).contains(&CastTarget::Stack(stack_of(
            &s,
            &r,
            "base:gold_dragon"
        ))));
        assert_eq!(
            cast(&mut s, &r, ice, CastTarget::Stack(target)),
            [(target.0, damage)],
            "{creature}"
        );
    }
    // R5: Octavia with the Orb of Driving Rain and Basic Sorcery, power 5: 173 to the Wolf
    // Riders and 25 to the Gold Golems (VCMI: 172, 24).
    let octavia = details(
        &r,
        "base:octavia",
        5,
        2,
        true,
        &["base:orb_of_driving_rain"],
        &["base:ice_bolt"],
    );
    for (creature, count, damage) in [("base:wolf_rider", 30, 173), ("base:gold_golem", 5, 25)] {
        let mut s = battle(&r, &octavia, &[(creature, count)], &[]);
        let target = stack_of(&s, &r, creature);
        assert_eq!(
            cast(&mut s, &r, ice, CastTarget::Stack(target)),
            [(target.0, damage)],
            "{creature}"
        );
    }
}

// covers: BAT-MAG-004, HER-BAT-012
/// E04b T11b review (r1 F-2; from the executable's code at 0x4E5760 and exact rational arithmetic
/// on its single-precision values, not observed in the game): a Sorcery specialist's grown rate
/// can lie a hair below its decimal (level 12, Advanced: 0.15999999642), and the product a hair
/// below a whole number (50 × 1.15999999642 = 57.99999982). The original stores the product as a
/// single-precision number before it rounds down, and that number is the whole one: the damage
/// is the exact rates' (the product's floor was one less: 57, 46, 110).
#[test]
fn a_sorcery_specialists_product_is_stored_as_a_single_before_it_is_rounded_down() {
    for profile in ["base:faithful", "base:strict-fixed"] {
        let r = sod_with(profile);
        let arrow = spell(&r, "base:magic_arrow");
        // Sandro with Advanced Sorcery and no magic school: 10 × power + 10, grown by
        // 10 % × (1 + 5 % × level): 50 × 1.16, 40 × 1.175, 100 × 1.11.
        for (level, power, damage) in [(12, 4, 58), (15, 3, 47), (2, 9, 111)] {
            let mut sandro = details(
                &r,
                "base:sandro",
                power,
                2,
                false,
                &[],
                &["base:magic_arrow"],
            );
            sandro.level = level;
            sandro.skills = vec![(r.skill_by_key("base:sorcery").unwrap(), 2)];
            let mut s = battle(&r, &sandro, &[("base:pikeman", 100)], &[]);
            let target = defenders_first(&s);
            assert_eq!(
                cast(&mut s, &r, arrow, CastTarget::Stack(target)),
                [(target.0, damage)],
                "{profile} level {level}"
            );
        }
    }
}

// covers: BAT-MAG-004, BAT-HER-001
/// E04b T11b review (F-robust-3) with the original's spells and the default profile (quirk
/// Q-0005 on): a hero of power 20 casts Implosion (75 per power + 100 = 1 600) with the largest
/// spell damage rates the seam carries for every spell and for Earth spells. The battle accepts
/// the hero and lists the cast; applying it overflowed in the battle's spell damage (a panic).
/// The damage is the executable's arithmetic on the stored rates (429 496.71875 each), from exact
/// rational arithmetic; with exact rates it is 1 600 × (1 + 429 496.7295)² rounded down.
#[test]
fn implosion_with_huge_spell_damage_rates_does_not_overflow() {
    for (profile, damage) in [
        ("base:faithful", 295_149_280_165_888),
        ("base:strict-fixed", 295_149_279_433_048),
    ] {
        let r = sod_with(profile);
        let implosion = spell(&r, "base:implosion");
        let earth = r
            .spell_schools()
            .iter()
            .position(|s| s.key.as_str() == "base:earth")
            .and_then(SchoolId::from_index)
            .unwrap();
        let mut hero = BattleHero::new(HeroTypeId::from_index(0).unwrap());
        hero.power = 20;
        hero.magic.mana = 100;
        hero.magic.spell_book = Some(vec![implosion]);
        let rate = Rate::bp(u32::MAX);
        hero.effects = vec![
            BattleEffect::SpellDamage {
                scope: SpellScope::All,
                rate,
            },
            BattleEffect::SpellDamage {
                scope: SpellScope::School(earth),
                rate,
            },
        ];
        let mut s = led(&r, hero, &[("base:pikeman", 10)], &[]);
        let target = defenders_first(&s);
        assert!(targets(&s, &r, implosion).contains(&CastTarget::Stack(target)));
        assert_eq!(
            cast(&mut s, &r, implosion, CastTarget::Stack(target)),
            [(target.0, damage)],
            "{profile}"
        );
    }
}

// covers: BAT-MAG-006, BAT-MAG-003, BAT-TAC-001
/// Batch 2a R1 and R2 as they were played (E04b T11b review, r1 F-10): a hero of power 1 with 30
/// spell points and ten Pikemen casts Chain Lightning at the first of three Peasant stacks (100,
/// 101 and 102), which Blue's hero placed in its tactics phase (Expert Tactics). At no mastery
/// the chain strikes four stacks with 40 × 1 + 25 = 65, then half each time: 65, 32, 16, 8.
#[test]
fn batch_2a_r1_r2_chain_lightning_by_the_screen_distance_and_the_army_slot() {
    let r = sod();
    let chain = spell(&r, "base:chain_lightning");
    let field = Field::new(r.battle().field);
    // The damage of each stack struck, in order, and every stack's count afterwards: stack 0
    // the Pikemen, 1 to 3 the Peasants in their army's order, placed on `hexes` (column, row).
    let play = |red: &str, blue: &str, hexes: [(i32, i32); 3]| {
        let caster = details(&r, red, 1, 3, false, &[], &["base:chain_lightning"]);
        let tactician = HeroDetails {
            hero_type: r.hero_type_by_key(blue).unwrap(),
            level: 1,
            experience: 0,
            primary: [0; 4],
            skills: vec![(r.skill_by_key("base:tactics").unwrap(), 3)],
            worn: vec![],
            backpack: vec![],
            spells: vec![],
            mana: 0,
        };
        let peasants = [100, 101, 102].map(|count| ("ab:peasant", count));
        let setup = BattleSetup {
            attacker: army(&r, &[("base:pikeman", 10)]),
            defender: army(&r, &peasants),
            obstacles: vec![],
            heroes: [&caster, &tactician].map(|d| Some(derive_battle_hero(d, &r).unwrap())),
            terrain: None,
            battlefield: None,
            anti_magic: false,
            siege: None,
            war_machines: [vec![], vec![]],
            formations: [Formation::Loose; 2],
            funds: [0; 2],
        };
        let mut s = BattleState::new(&r, &setup, 1).unwrap();
        // Blue's tactics phase: its stacks come in their army's order (equal speeds).
        assert_eq!(
            s.tactics().map(|t| (t.side, t.columns)),
            Some((BattleSide::Defender, 7))
        );
        for (id, (x, y)) in (1u8..).zip(hexes) {
            assert_eq!(s.active(), Some(StackId(id)));
            let to = field.hex(x, y).unwrap();
            let place = Command::Move { to };
            apply(
                &mut s,
                &r,
                BattleSide::Defender,
                place,
                &mut Vec::<Event>::new(),
            )
            .unwrap();
        }
        let end = Command::EndTactics;
        apply(
            &mut s,
            &r,
            BattleSide::Defender,
            end,
            &mut Vec::<Event>::new(),
        )
        .unwrap();
        // Round 1: the Pikemen act before the Peasants. Chain Lightning may be aimed at each
        // Peasant stack and not at the caster's own Pikemen (R1, round 2).
        assert_eq!((s.round(), s.active()), (1, Some(StackId(0))));
        assert_eq!(s.hero(BattleSide::Attacker).unwrap().mana, 30);
        assert_eq!(
            targets(&s, &r, chain),
            [1, 2, 3].map(|id| CastTarget::Stack(StackId(id)))
        );
        let struck = cast(&mut s, &r, chain, CastTarget::Stack(StackId(1)));
        let counts: Vec<u32> = s.stacks().iter().map(|t| t.count).collect();
        (struck, counts)
    };
    // R1: A (the 100) in row 5, B (the 101) two hexes to its left, 88 pixels away, C (the 102)
    // two rows beneath it, 84 pixels away (the pattern one hex to the left of the plan, as it
    // was played). The bolt went A, C, B, Pikemen: A 35, C 70, B 85 left, the Pikemen all ten.
    assert_eq!(
        play("base:caitlin", "base:nagash", [(11, 4), (9, 4), (11, 6)]),
        (
            vec![(1, 65), (3, 32), (2, 16), (0, 8)],
            vec![10, 35, 85, 70]
        )
    );
    // R2: A (the 100), R (the 101) two hexes to its right and L (the 102) two hexes to its left,
    // both 88 pixels away (the pattern one hex down-left of the plan, as it was played). The
    // bolt went A, R, L, Pikemen: the earlier stack of the army first; A 35, R 69, L 86 left.
    assert_eq!(
        play("base:jenova", "base:damacon", [(12, 5), (14, 5), (10, 5)]),
        (
            vec![(1, 65), (2, 32), (3, 16), (0, 8)],
            vec![10, 35, 69, 86]
        )
    );
}

// covers: BAT-MAG-003, BAT-MAG-005, BAT-MAG-007
#[test]
fn batch_2a_r3_where_fireball_and_implosion_may_be_aimed() {
    let r = sod();
    let fireball = spell(&r, "base:fireball");
    let implosion = spell(&r, "base:implosion");
    let aine = details(
        &r,
        "base:aine",
        1,
        12,
        false,
        &[],
        &["base:fireball", "base:implosion"],
    );
    let mut s = battle(
        &r,
        &aine,
        &[
            ("base:battle_dwarf", 20),
            ("base:efreeti", 5),
            ("base:gnoll", 50),
            ("base:gnoll", 51),
        ],
        &["base:ballista"],
    );
    // (a) Implosion is refused at the Ballista, allowed at the Dwarves.
    let ballista = stack_of(&s, &r, "base:ballista");
    let implode = targets(&s, &r, implosion);
    assert!(!implode.contains(&CastTarget::Stack(ballista)));
    assert!(implode.contains(&CastTarget::Stack(stack_of(&s, &r, "base:battle_dwarf"))));
    // (b) Fireball may be aimed at an empty hex far from every stack, (c) at the fire-immune
    // Efreeti, who are spared.
    let fire = targets(&s, &r, fireball);
    let taken: Vec<_> = s.stacks().iter().map(|t| t.hex).collect();
    assert!(
        fire.iter()
            .filter_map(|t| t.hex())
            .any(|h| !taken.contains(&h))
    );
    let efreeti = stack_of(&s, &r, "base:efreeti");
    let at = CastTarget::Hex(s.stacks()[usize::from(efreeti.0)].hex);
    assert!(fire.contains(&at));
    let struck = cast(&mut s, &r, fireball, at);
    assert!(struck.iter().all(|&(id, _)| id != efreeti.0));
}

// covers: BAT-MAG-001
/// E04b exit criterion 2: every battle spell in scope can be cast. Every combat spell of the
/// harvested packs but Earthquake (sieges, E04c) has effects the battle implements
/// (`CommandError::NotCastable` otherwise); so has every ability spell but the three the
/// creatures' own abilities carry out: the Liches' Death Cloud (an area shot, BAT-RNG-004), the
/// Thunderbirds' bolt (BAT-MAG-041) and the Mighty Gorgons' Death Stare (BAT-DMG-009).
#[test]
fn every_battle_spell_but_earthquake_is_implemented() {
    let r = sod();
    let missing = |category| {
        r.spells()
            .iter()
            .filter(|s| s.category == category && s.magic.effects.is_empty())
            .map(|s| s.key.as_str())
            .collect::<Vec<_>>()
    };
    assert_eq!(missing(SpellCategory::Combat), ["base:earthquake"]);
    assert_eq!(
        missing(SpellCategory::Ability),
        ["base:death_cloud", "base:thunderbolt", "base:death_stare"]
    );
}
