//! The heroes' spell defences and the elementals against Titan's Lightning Bolt on the harvested
//! SoD packs (E04b T8b2; HER-BAT-016, BAT-MAG-063 to BAT-MAG-066, BAT-MAG-005): the artifacts give
//! the original's defences (`docs/formats/installation.md` §7.23), Earth and Magma Elementals
//! never take the bolt and Air and Storm Elementals take it double (§7.8), and the Troglodytes'
//! Blind immunity is the one the Orb of Vulnerability leaves.

use h3_battle::{
    BattleState, CastTarget, CastTargets, Command, Event, EventKind, StackId, apply, cast_targets,
};
use h3_content::{Formation, Ruleset, SpellId};
use h3_contracts::{Army, ArmyStack, BattleEffect, BattleHero, BattleSetup, BattleSide};
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

/// A hero with power 10, 100 mana, `spells` in its book and `effects`.
fn caster(r: &Ruleset, spells: &[SpellId], effects: Vec<BattleEffect>) -> BattleHero {
    let mut h = BattleHero::new(r.hero_type_by_key("base:solmyr").unwrap());
    h.power = 10;
    h.magic.mana = 100;
    h.magic.max_mana = 100;
    h.magic.spell_book = Some(spells.to_vec());
    h.effects = effects;
    h
}

/// A battle of a Pikeman against `defender`, one creature per stack, the attacker's hero
/// `hero`; the Pikeman acts first.
fn battle(r: &Ruleset, hero: BattleHero, defender: &[&str]) -> BattleState {
    let setup = BattleSetup {
        attacker: Army {
            stacks: vec![ArmyStack {
                slot: 0,
                creature: r.creature_by_key("base:pikeman").unwrap(),
                count: 1,
            }],
        },
        defender: Army {
            stacks: defender
                .iter()
                .enumerate()
                .map(|(i, key)| ArmyStack {
                    slot: u8::try_from(i).unwrap(),
                    creature: r.creature_by_key(key).unwrap(),
                    count: 1,
                })
                .collect(),
        },
        obstacles: vec![],
        heroes: [Some(hero), None],
        terrain: None,
        war_machines: [vec![], vec![]],
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

/// The stacks the attacker's hero may cast `spell` at now, by id.
fn targets(s: &BattleState, r: &Ruleset, spell: SpellId) -> Vec<u8> {
    let mut out = CastTargets::new();
    cast_targets(s, r, BattleSide::Attacker, spell, &mut out);
    out.iter()
        .filter_map(|t| t.stack().map(|id| id.0))
        .collect()
}

// covers: BAT-MAG-005, BAT-MAG-004
#[test]
fn titans_lightning_bolt_spares_earth_and_doubles_against_air_elementals() {
    let r = sod();
    let bolt = spell(&r, "sod:titans_lightning_bolt");
    let defender = [
        "base:earth_elemental",
        "ab:magma_elemental",
        "base:air_elemental",
        "ab:storm_elemental",
        "base:gnoll",
    ];
    let mut s = battle(&r, caster(&r, &[bolt], vec![]), &defender);
    // Stacks: 0 the Pikeman, then the defender's in order.
    assert_eq!(targets(&s, &r, bolt), [3, 4, 5]);
    let damage = |events: &[Event]| {
        events.iter().find_map(|e| match e.kind {
            EventKind::SpellDamage { damage, .. } => Some(damage),
            _ => None,
        })
    };
    let cast = |target: u8| Command::Cast {
        spell: bolt,
        target: CastTarget::Stack(StackId(target)),
        second: CastTarget::None,
    };
    let mut events = Vec::new();
    apply(&mut s, &r, BattleSide::Attacker, cast(3), &mut events).unwrap();
    assert_eq!(damage(&events), Some(1200));
    let mut s = battle(&r, caster(&r, &[bolt], vec![]), &defender);
    let mut events = Vec::new();
    apply(&mut s, &r, BattleSide::Attacker, cast(5), &mut events).unwrap();
    assert_eq!(damage(&events), Some(600));
}

// covers: BAT-MAG-066, BAT-MAG-065
#[test]
fn the_orb_lifts_the_air_elementals_blind_immunity_but_not_the_troglodytes() {
    let r = sod();
    let blind = spell(&r, "base:blind");
    let defender = ["base:troglodyte", "base:air_elemental", "base:gnoll"];
    let s = battle(&r, caster(&r, &[blind], vec![]), &defender);
    assert_eq!(targets(&s, &r, blind), [3]);
    let orb = vec![BattleEffect::NegateImmunities];
    let s = battle(&r, caster(&r, &[blind], orb), &defender);
    assert_eq!(targets(&s, &r, blind), [2, 3]);
}

// covers: HER-BAT-016
#[test]
fn the_artifacts_give_the_originals_defences() {
    let r = sod();
    let worn = |keys: &[&str]| HeroDetails {
        hero_type: r.hero_type_by_key("base:solmyr").unwrap(),
        level: 1,
        experience: 0,
        primary: [0, 0, 1, 1],
        skills: vec![],
        worn: keys
            .iter()
            .map(|k| r.artifact_by_key(k).unwrap().into())
            .collect(),
        backpack: vec![],
        spells: vec![],
        mana: 0,
    };
    let effects = derive_battle_hero(
        &worn(&[
            "base:pendant_of_negativity",
            "base:sphere_of_permanence",
            "base:orb_of_vulnerability",
            "base:badge_of_courage",
        ]),
        &r,
    )
    .unwrap()
    .effects;
    let immune = |key| BattleEffect::SpellImmunity {
        spell: spell(&r, key),
    };
    // After the Badge's morale and Solmyr's specialty.
    assert_eq!(
        effects[2..],
        [
            immune("base:lightning_bolt"),
            immune("base:chain_lightning"),
            immune("base:dispel"),
            immune("base:dispel_helpful_spells"),
            BattleEffect::MindImmunity,
            BattleEffect::NegateImmunities,
        ]
    );
    let blade = derive_battle_hero(&worn(&["ab:armageddons_blade"]), &r)
        .unwrap()
        .effects;
    assert!(blade.contains(&immune("base:armageddon")));
}
