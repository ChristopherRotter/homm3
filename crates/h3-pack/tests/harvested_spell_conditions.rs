//! The per-spell target conditions of the original's creature test on the harvested SoD packs
//! (E04b T8b3; BAT-MAG-067, BAT-Q22): Bless and Curse spare the undead and the creatures with
//! maximum damage 0 (the First Aid Tent, the Ammo Cart) but not the Ballista, Precision needs a
//! shooter, Mirth and Sorrow spare the creatures without morale, Fortune, Misfortune and Slayer
//! the creatures with maximum damage 0, and the Orb of Vulnerability lifts none of them
//! (`docs/formats/installation.md` §7.23).

use h3_battle::{BattleState, CastTargets, Command, Event, apply, cast_targets};
use h3_content::{Formation, Ruleset};
use h3_contracts::{Army, ArmyStack, BattleEffect, BattleHero, BattleSetup, BattleSide};

fn sod() -> Ruleset {
    h3_pack::compile(
        &[h3_pack::SOD_PACK, h3_pack::AB_PACK, h3_pack::BASE_PACK],
        "base:faithful",
    )
    .unwrap()
}

/// A hero with power 10, 100 mana, the eight spells in its book and `effects`.
fn caster(r: &Ruleset, effects: Vec<BattleEffect>) -> BattleHero {
    let mut h = BattleHero::new(r.hero_type_by_key("base:solmyr").unwrap());
    h.power = 10;
    h.magic.mana = 100;
    h.magic.max_mana = 100;
    h.magic.spell_book = Some(SPELLS.iter().map(|k| r.spell_by_key(k).unwrap()).collect());
    h.effects = effects;
    h
}

const SPELLS: [&str; 8] = [
    "base:bless",
    "base:curse",
    "base:precision",
    "base:mirth",
    "base:sorrow",
    "base:fortune",
    "base:misfortune",
    "base:slayer",
];

/// Both sides field one creature of each of `CREATURES` and the three war machines; the
/// attacker's hero has `effects` and acts first.
fn battle(r: &Ruleset, effects: Vec<BattleEffect>) -> BattleState {
    let army = || Army {
        stacks: CREATURES
            .iter()
            .enumerate()
            .map(|(i, key)| ArmyStack {
                slot: u8::try_from(i).unwrap(),
                creature: r.creature_by_key(key).unwrap(),
                count: 1,
            })
            .collect(),
    };
    let machines = || {
        ["base:ballista", "base:ammo_cart", "base:first_aid_tent"]
            .iter()
            .map(|k| r.creature_by_key(k).unwrap())
            .collect::<Vec<_>>()
    };
    let defender = BattleHero::new(r.hero_type_by_key("base:orrin").unwrap());
    let setup = BattleSetup {
        attacker: army(),
        defender: army(),
        obstacles: vec![],
        heroes: [Some(caster(r, effects)), Some(defender)],
        terrain: None,
        war_machines: [machines(), machines()],
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

const CREATURES: [&str; 6] = [
    "base:pikeman",
    "base:archer",
    "base:skeleton",
    "base:stone_golem",
    "base:air_elemental",
    "base:giant",
];

/// The creatures the attacker's hero may cast `spell` at now, by key (the war machines short).
fn targets(s: &BattleState, r: &Ruleset, spell: &str) -> Vec<String> {
    let mut out = CastTargets::new();
    cast_targets(
        s,
        r,
        BattleSide::Attacker,
        r.spell_by_key(spell).unwrap(),
        &mut out,
    );
    out.iter()
        .filter_map(|t| t.stack())
        .map(|id| {
            let c = s.stack(id).unwrap().creature;
            let key = r.creature_key(c).unwrap().to_string();
            key.trim_start_matches("base:").to_owned()
        })
        .collect()
}

// covers: BAT-MAG-067, BAT-MAG-005
#[test]
fn the_per_spell_conditions_keep_the_originals_targets() {
    let r = sod();
    let s = battle(&r, vec![]);
    let all = |keys: &[&str]| keys.iter().map(|k| (*k).to_owned()).collect::<Vec<_>>();
    // Own stacks for the positive spells, the enemy's for the negative ones; the Ballista takes
    // Bless and Curse, the Tent and the Ammo Cart do not, nor do the undead.
    let damaging = [
        "pikeman",
        "archer",
        "stone_golem",
        "air_elemental",
        "giant",
        "ballista",
    ];
    assert_eq!(targets(&s, &r, "base:bless"), all(&damaging));
    assert_eq!(targets(&s, &r, "base:curse"), all(&damaging));
    // Precision: the shooters.
    assert_eq!(
        targets(&s, &r, "base:precision"),
        all(&["archer", "ballista"])
    );
    // Mirth and Sorrow: creatures with morale and no mind immunity.
    assert_eq!(targets(&s, &r, "base:mirth"), all(&["pikeman", "archer"]));
    assert_eq!(targets(&s, &r, "base:sorrow"), all(&["pikeman", "archer"]));
    // Fortune, Misfortune and Slayer: every creature with damage, the undead too.
    let with_damage = [
        "pikeman",
        "archer",
        "skeleton",
        "stone_golem",
        "air_elemental",
        "giant",
        "ballista",
    ];
    for spell in ["base:fortune", "base:misfortune", "base:slayer"] {
        assert_eq!(targets(&s, &r, spell), all(&with_damage), "{spell}");
    }
}

// covers: BAT-MAG-067, BAT-MAG-066
#[test]
fn the_orb_of_vulnerability_lifts_none_of_them() {
    let r = sod();
    let s = battle(&r, vec![BattleEffect::NegateImmunities]);
    let all = |keys: &[&str]| keys.iter().map(|k| (*k).to_owned()).collect::<Vec<_>>();
    // The Giants' mind immunity yields to the Orb; the golems' and the elementals' missing morale,
    // the skeletons' undead and the machines' do not.
    assert_eq!(
        targets(&s, &r, "base:sorrow"),
        all(&["pikeman", "archer", "giant"])
    );
    assert_eq!(
        targets(&s, &r, "base:mirth"),
        all(&["pikeman", "archer", "giant"])
    );
    assert!(!targets(&s, &r, "base:bless").contains(&"skeleton".to_owned()));
    assert!(!targets(&s, &r, "base:curse").contains(&"first_aid_tent".to_owned()));
    assert!(!targets(&s, &r, "base:misfortune").contains(&"ammo_cart".to_owned()));
    assert_eq!(
        targets(&s, &r, "base:precision"),
        all(&["archer", "ballista"])
    );
}
