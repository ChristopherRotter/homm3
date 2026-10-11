//! Batch 2a's direct-damage spells on the harvested SoD packs (E04b T11a, the fold-in; BAT-MAG-003
//! to BAT-MAG-005): R3 to R5 of `experiments/results/E04b-batch-2a.md` (SoD 4.0 Complete, plain
//! HD, 2026-10-06) replayed with the heroes the batch used, derived in the Heroes context. Also
//! E04b exit criterion 2's guard: every combat spell of the packs but Earthquake (E04c) and every
//! ability spell has effects the battle implements.

use h3_battle::{
    BattleState, CastTarget, CastTargets, Command, Event, EventKind, StackId, apply, cast_targets,
};
use h3_content::{Formation, Ruleset, SpellCategory, SpellId};
use h3_contracts::{Army, ArmyStack, BattleSetup, BattleSide};
use h3_heroes::{HeroDetails, derive_battle_hero};

fn sod() -> Ruleset {
    h3_pack::compile(
        &[h3_pack::SOD_PACK, h3_pack::AB_PACK, h3_pack::BASE_PACK],
        "base:faithful",
    )
    .unwrap()
}

fn spell(r: &Ruleset, key: &str) -> SpellId {
    r.spell_by_key(key).unwrap()
}

/// `hero` at level 1 with attack and defense 0, `power` and `knowledge`, Basic Sorcery if
/// `sorcery`, wearing a Spell Book and `worn`, knowing `spells`, with full mana.
fn details(
    r: &Ruleset,
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

fn army(r: &Ruleset, stacks: &[(&str, u32)]) -> Army {
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
    r: &Ruleset,
    details: &HeroDetails,
    defender: &[(&str, u32)],
    machines: &[&str],
) -> BattleState {
    let hero = derive_battle_hero(details, r).unwrap();
    let setup = BattleSetup {
        attacker: army(r, &[("base:pikeman", 10)]),
        defender: army(r, defender),
        obstacles: vec![],
        heroes: [Some(hero), None],
        terrain: None,
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
fn stack_of(s: &BattleState, r: &Ruleset, creature: &str) -> StackId {
    let c = r.creature_by_key(creature).unwrap();
    let i = s.stacks().iter().position(|t| t.creature == c).unwrap();
    StackId(u8::try_from(i).unwrap())
}

/// Where the attacker's hero may cast `spell` now.
fn targets(s: &BattleState, r: &Ruleset, spell: SpellId) -> Vec<CastTarget> {
    let mut out = CastTargets::new();
    cast_targets(s, r, BattleSide::Attacker, spell, &mut out);
    out.iter().copied().collect()
}

/// The attacker's hero casts `spell` at `target`: the damage of each stack struck.
fn cast(s: &mut BattleState, r: &Ruleset, spell: SpellId, target: CastTarget) -> Vec<(u8, u64)> {
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
