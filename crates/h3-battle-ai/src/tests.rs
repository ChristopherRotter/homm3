//! Tests of the policy on a small ruleset built in code (an application crate takes no pack
//! adapter as a dev-dependency): the dispatch of the stack's action and the shooter (BAT-AI-001,
//! BAT-AI-002), the area shooters' value at a hex with and without the original's slip
//! (BAT-AI-003, BAT-QUIRK-25), the fallback, and a property over random battles. Sieges (the
//! Cyclops's wall shot, BAT-AI-004), special battlefields and the harvested creatures:
//! `h3-pack-battle`'s `harvested_computer.rs`.

use std::string::ToString;
use std::sync::Arc;
use std::vec;
use std::vec::Vec;

use h3_battle::{
    BattleState, Command, Field, Hex, LegalCommands, NoopSink, StackId, apply, computer,
    legal_commands, validate,
};
use h3_content::{
    ContentKey, ContentPack, CreatureDef, CreatureInfoDef, CreatureTypeId, FactionDef,
    HeroClassDef, HeroTypeDef, HeroTypeId, PackManifest, ProfileDef, ResourceDef, Ruleset,
    SpellCategory, SpellDef, SpellId, SpellSchoolDef, TerrainDef,
};
use h3_content_battle::{
    AbilityDef, AreaShot, AuraTarget, BallisticsRecord, BattleCreatureDef, BattlePack,
    BattleParamsDef, BattleRuleset, BattleSpellDef, EffectDuration, HexStep, ObstacleKind,
    ObstacleShape, SiegeDef, SiegeFieldDef, SpellAi, SpellEffect, SpellMagic, SpellPolarity,
    SpellTarget, TimedChange, Usefulness, WallKind, WallTarget,
};
use h3_contracts::{
    Army, ArmyStack, BallistaControl, BattleHero, BattleSetup, BattleSide, FortLevel, Moat, Rate,
    Siege, TentControl,
};
use h3_kernel::Rng;
use proptest::prelude::{any, prop_assert, prop_assert_eq, proptest};
use proptest::test_runner::Config;

use super::{POLICY_VERSION, checked, choose, unchecked};
use crate::walker::Walker;

mod casts;
mod defenses;
mod ends;
mod gain;
mod harms;
mod odds;
mod order;
mod predict;
mod speed;
mod spells;
mod values;

fn key(text: &str) -> ContentKey {
    ContentKey::parse(text).unwrap()
}

fn manifest() -> PackManifest {
    PackManifest {
        id: "t".into(),
        version: "1".into(),
        depends: vec![],
    }
}

/// A test creature: its key, speed, Fight Value and flags; its attack, defense, damage, hit
/// points and shots; its ability (the First Aid Tent's, an area shot).
type Creature = (
    &'static str,
    u8,
    u32,
    &'static [&'static str],
    [u16; 4],
    u32,
    u16,
    Option<AbilityDef>,
);

/// The First Aid Tent's healing.
const HEALS: Option<AbilityDef> = Some(AbilityDef::FirstAid {
    min_hp: 1,
    max_hp: 25,
});

/// The test creatures, by handle: walkers (the imp, the gog, the runner, faster than the field is
/// wide), the archer, the Ballista and the First Aid Tent; the area shooters (the magog's
/// fireball, the undead lich's death cloud); fast walkers that are fire-immune (the efreet),
/// lifeless (the golem) and undead (the zombie); a slow two-hex walker (the cavalier); a flyer
/// that strikes and returns (the harpy); the multi-headed (the two-hex hydra, the cerberus), a
/// breath attacker (the dragon), a town's turret (the tower, BAT-SIE-016), a walker with an innate
/// Fire Shield (the sultan), one that strikes twice (the crusader), kings of each level (the
/// giant, the knight, the lord), a Master Genie (the genie: three casts of the test spells), the
/// griffins with two and with unlimited retaliations, a shooter that shoots twice (the
/// marksman) and the creatures that change the heroes' spell costs (the pegasus, the mage).
const CREATURES: [Creature; 28] = [
    ("imp", 5, 100, &[], [2, 3, 1, 2], 4, 0, None),
    ("gog", 3, 900, &[], [4, 4, 2, 3], 13, 0, None),
    ("runner", 20, 300, &["alive"], [5, 5, 2, 4], 10, 0, None),
    ("archer", 4, 300, &["shooter"], [6, 3, 2, 3], 10, 12, None),
    (
        "ballista",
        0,
        600,
        &["war_machine", "two_hex", "shooter"],
        [10, 10, 2, 3],
        250,
        24,
        None,
    ),
    (
        "tent",
        0,
        50,
        &["war_machine", "two_hex"],
        [0, 0, 0, 0],
        75,
        0,
        HEALS,
    ),
    (
        "magog",
        4,
        400,
        &["shooter", "area_attack", "alive"],
        [7, 4, 2, 4],
        13,
        24,
        Some(AbilityDef::AreaShot {
            effect: AreaShot::Fireball,
        }),
    ),
    (
        "lich",
        4,
        900,
        &["shooter", "area_attack", "undead"],
        [13, 10, 11, 13],
        30,
        24,
        Some(AbilityDef::AreaShot {
            effect: AreaShot::DeathCloud,
        }),
    ),
    (
        "efreet",
        20,
        300,
        &["fire_immune", "alive"],
        [5, 5, 2, 4],
        10,
        0,
        None,
    ),
    ("golem", 20, 300, &[], [5, 5, 2, 4], 10, 0, None),
    ("zombie", 20, 300, &["undead"], [5, 5, 2, 4], 10, 0, None),
    (
        "cavalier",
        2,
        1900,
        &["two_hex", "alive"],
        [15, 15, 15, 25],
        100,
        0,
        None,
    ),
    (
        "harpy",
        6,
        150,
        &["flying", "alive"],
        [6, 5, 1, 4],
        14,
        0,
        Some(AbilityDef::StrikeAndReturn),
    ),
    (
        "hydra",
        4,
        3000,
        &["two_hex", "alive", "multi_headed"],
        [16, 18, 25, 45],
        175,
        0,
        Some(AbilityDef::AttacksAllAdjacent),
    ),
    (
        "cerberus",
        4,
        500,
        &["alive", "multi_headed"],
        [10, 8, 2, 7],
        25,
        0,
        Some(AbilityDef::ThreeHeadedAttack),
    ),
    (
        "dragon",
        9,
        3000,
        &["breath_attack", "alive"],
        [25, 25, 40, 50],
        200,
        0,
        None,
    ),
    (
        "tower",
        0,
        50,
        &["shooter", "war_machine"],
        [10, 5, 2, 4],
        100,
        99,
        Some(AbilityDef::Turret),
    ),
    (
        "sultan",
        13,
        1000,
        &["alive", "fire_immune"],
        [16, 14, 14, 20],
        90,
        0,
        None,
    ),
    (
        "crusader",
        6,
        1500,
        &["alive", "strikes_twice"],
        [12, 12, 7, 10],
        35,
        0,
        None,
    ),
    (
        "giant",
        7,
        2000,
        &["alive", "king_1"],
        [19, 16, 40, 60],
        150,
        0,
        None,
    ),
    (
        "knight",
        7,
        2000,
        &["alive", "king_2"],
        [19, 16, 40, 60],
        150,
        0,
        None,
    ),
    (
        "lord",
        7,
        2000,
        &["alive", "king_3"],
        [19, 16, 40, 60],
        150,
        0,
        None,
    ),
    (
        "genie",
        12,
        1500,
        &["alive", "flying"],
        [12, 12, 13, 16],
        40,
        0,
        None,
    ),
    (
        "griffin",
        6,
        350,
        &["alive", "flying", "two_hex"],
        [8, 8, 3, 6],
        25,
        0,
        Some(AbilityDef::ExtraRetaliations { count: 1 }),
    ),
    (
        "royal",
        9,
        450,
        &["alive", "flying", "two_hex"],
        [9, 9, 3, 6],
        25,
        0,
        Some(AbilityDef::UnlimitedRetaliations),
    ),
    (
        "marksman",
        6,
        400,
        &["alive", "shooter", "strikes_twice"],
        [6, 3, 2, 3],
        10,
        24,
        None,
    ),
    (
        "pegasus",
        8,
        250,
        &["alive", "flying", "two_hex"],
        [5, 5, 3, 5],
        20,
        0,
        Some(AbilityDef::SpellCost {
            amount: 2,
            affects: AuraTarget::Enemies,
        }),
    ),
    (
        "mage",
        5,
        500,
        &["alive", "shooter"],
        [11, 8, 7, 9],
        25,
        24,
        Some(AbilityDef::SpellCost {
            amount: -2,
            affects: AuraTarget::Allies,
        }),
    ),
];

const IMP: usize = 0;
const GOG: usize = 1;
const RUNNER: usize = 2;
const ARCHER: usize = 3;
const BALLISTA: usize = 4;
const TENT: usize = 5;
const MAGOG: usize = 6;
const LICH: usize = 7;
const EFREET: usize = 8;
const GOLEM: usize = 9;
const ZOMBIE: usize = 10;
const CAVALIER: usize = 11;
const HARPY: usize = 12;
const HYDRA: usize = 13;
const CERBERUS: usize = 14;
const DRAGON: usize = 15;
const TOWER: usize = 16;
const SULTAN: usize = 17;
const CRUSADER: usize = 18;
const GIANT: usize = 19;
const KNIGHT: usize = 20;
const LORD: usize = 21;
const GENIE: usize = 22;
const GRIFFIN: usize = 23;
const ROYAL: usize = 24;
const MARKSMAN: usize = 25;
const PEGASUS: usize = 26;
const MAGE: usize = 27;

/// A test spell of the computer's spell values (E23 T4c): its key, level, value by mastery,
/// polarity, timed changes, the mastery from which it is cast on the whole side, its usefulness
/// for the Master Genie and its school. Every one is aimed at a stack.
type TestSpell = (
    &'static str,
    u8,
    [u16; 4],
    SpellPolarity,
    &'static [TimedChange],
    Option<u8>,
    Usefulness,
    &'static str,
);

/// The test spells after the Fire Wall and Fire Shield, with the values of the original's
/// (`SPTRAITS.TXT`).
const SPELLS: [TestSpell; 16] = [
    (
        "bloodlust",
        1,
        [3, 3, 6, 6],
        SpellPolarity::Positive,
        &[TimedChange::MeleeAttack],
        Some(3),
        Usefulness::Melee,
        "fire",
    ),
    (
        "precision",
        2,
        [3, 3, 6, 6],
        SpellPolarity::Positive,
        &[TimedChange::RangedAttack],
        Some(3),
        Usefulness::Shooting,
        "fire",
    ),
    (
        "slayer",
        4,
        [8, 8, 8, 8],
        SpellPolarity::Positive,
        &[TimedChange::Slayer],
        None,
        Usefulness::EnemyKing,
        "fire",
    ),
    (
        "bless",
        1,
        [0, 0, 1, 1],
        SpellPolarity::Positive,
        &[TimedChange::MaximumDamage],
        Some(3),
        Usefulness::Always,
        "fire",
    ),
    (
        "mirth",
        3,
        [1, 1, 2, 2],
        SpellPolarity::Positive,
        &[TimedChange::Morale],
        Some(3),
        Usefulness::Always,
        "fire",
    ),
    (
        "fortune",
        2,
        [1, 1, 2, 2],
        SpellPolarity::Positive,
        &[TimedChange::Luck],
        Some(3),
        Usefulness::Always,
        "fire",
    ),
    (
        "shield",
        1,
        [85, 85, 70, 70],
        SpellPolarity::Positive,
        &[TimedChange::MeleeProtection],
        Some(3),
        Usefulness::EnemyMelee,
        "fire",
    ),
    (
        "airshield",
        3,
        [75, 75, 50, 50],
        SpellPolarity::Positive,
        &[TimedChange::RangedProtection],
        Some(3),
        Usefulness::EnemyShooter,
        "fire",
    ),
    (
        "stoneskin",
        1,
        [3, 3, 6, 6],
        SpellPolarity::Positive,
        &[TimedChange::Defense],
        Some(3),
        Usefulness::Always,
        "fire",
    ),
    (
        "counterstrike",
        4,
        [1, 1, 2, 2],
        SpellPolarity::Positive,
        &[TimedChange::Retaliations],
        Some(3),
        Usefulness::Always,
        "fire",
    ),
    (
        "frenzy",
        4,
        [100, 100, 150, 200],
        SpellPolarity::Positive,
        &[TimedChange::Frenzy],
        None,
        Usefulness::Always,
        "fire",
    ),
    (
        "haste",
        1,
        [3, 3, 5, 5],
        SpellPolarity::Positive,
        &[TimedChange::Speed],
        Some(3),
        Usefulness::Always,
        "air",
    ),
    (
        "prayer",
        4,
        [2, 2, 4, 4],
        SpellPolarity::Positive,
        &[
            TimedChange::Attack,
            TimedChange::Defense,
            TimedChange::Speed,
        ],
        Some(3),
        Usefulness::NotActed,
        "fire",
    ),
    (
        "protfire",
        1,
        [70, 70, 50, 50],
        SpellPolarity::Positive,
        &[TimedChange::SchoolProtection],
        Some(3),
        Usefulness::EnemySpellBook,
        "fire",
    ),
    (
        "protair",
        2,
        [70, 70, 50, 50],
        SpellPolarity::Positive,
        &[TimedChange::SchoolProtection],
        Some(3),
        Usefulness::EnemySpellBook,
        "air",
    ),
    (
        "mirror",
        5,
        [20, 20, 30, 40],
        SpellPolarity::Positive,
        &[TimedChange::MagicMirror],
        None,
        Usefulness::EnemySpellBook,
        "air",
    ),
];

/// A test spell of the harmful spells' and the removals' values (E23 T4c): its key, level,
/// value by mastery, power, category, polarity, effects, whether it lasts to the battle's end and
/// the spells its dispel ends. Every one is aimed at a stack; a hero's is a fire spell.
type ExtraSpell = (
    &'static str,
    u8,
    [u16; 4],
    u16,
    SpellCategory,
    SpellPolarity,
    &'static [SpellEffect],
    bool,
    &'static [&'static str],
);

/// A timed change as a spell's effect.
const fn timed(change: TimedChange) -> SpellEffect {
    SpellEffect::Timed(change)
}

/// The harmful test spells and the removals, with the values of the original's.
const EXTRA: [ExtraSpell; 17] = [
    (
        "curse",
        1,
        [0, 0, 1, 1],
        0,
        SpellCategory::Combat,
        SpellPolarity::Negative,
        &[timed(TimedChange::MinimumDamage)],
        false,
        &[],
    ),
    (
        "weakness",
        2,
        [3, 3, 6, 6],
        0,
        SpellCategory::Combat,
        SpellPolarity::Negative,
        &[timed(TimedChange::AttackLoss)],
        false,
        &[],
    ),
    (
        "ray",
        2,
        [3, 3, 4, 5],
        0,
        SpellCategory::Combat,
        SpellPolarity::Negative,
        &[timed(TimedChange::DefenseLoss)],
        true,
        &[],
    ),
    (
        "acid",
        0,
        [3, 3, 3, 3],
        0,
        SpellCategory::Ability,
        SpellPolarity::Negative,
        &[timed(TimedChange::DefenseLoss)],
        true,
        &[],
    ),
    (
        "sorrow",
        4,
        [1, 1, 2, 2],
        0,
        SpellCategory::Combat,
        SpellPolarity::Negative,
        &[timed(TimedChange::MoraleLoss)],
        false,
        &[],
    ),
    (
        "misfortune",
        3,
        [1, 1, 2, 2],
        0,
        SpellCategory::Combat,
        SpellPolarity::Negative,
        &[timed(TimedChange::LuckLoss)],
        false,
        &[],
    ),
    (
        "slow",
        1,
        [75, 75, 50, 50],
        0,
        SpellCategory::Combat,
        SpellPolarity::Negative,
        &[timed(TimedChange::SpeedPercent)],
        false,
        &[],
    ),
    (
        "berserk",
        4,
        [0, 0, 1, 2],
        0,
        SpellCategory::Combat,
        SpellPolarity::Negative,
        &[timed(TimedChange::Berserk)],
        false,
        &[],
    ),
    (
        "hypnotize",
        3,
        [10, 10, 20, 50],
        25,
        SpellCategory::Combat,
        SpellPolarity::Negative,
        &[timed(TimedChange::Hypnotize)],
        false,
        &[],
    ),
    (
        "forget",
        3,
        [0, 0, 0, 0],
        0,
        SpellCategory::Combat,
        SpellPolarity::Negative,
        &[timed(TimedChange::Forgetfulness)],
        false,
        &[],
    ),
    (
        "blind",
        2,
        [50, 50, 25, 0],
        0,
        SpellCategory::Combat,
        SpellPolarity::Negative,
        &[timed(TimedChange::Blind)],
        false,
        &[],
    ),
    (
        "paralyze",
        4,
        [25, 25, 25, 25],
        0,
        SpellCategory::Ability,
        SpellPolarity::Negative,
        &[timed(TimedChange::Paralyze)],
        false,
        &[],
    ),
    (
        "poison",
        0,
        [0, 0, 0, 0],
        0,
        SpellCategory::Ability,
        SpellPolarity::Negative,
        &[timed(TimedChange::Poison)],
        false,
        &[],
    ),
    (
        "disease",
        2,
        [2, 2, 2, 2],
        0,
        SpellCategory::Ability,
        SpellPolarity::Negative,
        &[
            timed(TimedChange::AttackLoss),
            timed(TimedChange::DefenseLoss),
        ],
        false,
        &[],
    ),
    (
        "age",
        5,
        [0, 0, 0, 0],
        0,
        SpellCategory::Ability,
        SpellPolarity::Negative,
        &[timed(TimedChange::Age)],
        false,
        &[],
    ),
    (
        "cure",
        1,
        [10, 10, 20, 30],
        5,
        SpellCategory::Combat,
        SpellPolarity::Positive,
        &[SpellEffect::Dispel, SpellEffect::Heal],
        false,
        &[
            "curse",
            "weakness",
            "slow",
            "blind",
            "poison",
            "sorrow",
            "misfortune",
        ],
    ),
    (
        "antimagic",
        3,
        [4, 4, 5, 6],
        0,
        SpellCategory::Combat,
        SpellPolarity::Positive,
        &[timed(TimedChange::SpellImmunity)],
        false,
        &[],
    ),
];

/// A hero whose book holds every test spell but the creatures' abilities at `mastery`, with 999
/// mana and power 5.
fn caster(r: &BattleRuleset, mastery: u8) -> BattleHero {
    let mut h = hero(0);
    h.power = 5;
    let book: Vec<SpellId> = SPELLS
        .iter()
        .map(|s| s.0)
        .chain(
            EXTRA
                .iter()
                .filter(|s| s.4 == SpellCategory::Combat)
                .map(|s| s.0),
        )
        .chain(["bolt"])
        .map(|k| spell(r, k))
        .collect();
    h.magic.masteries = book.iter().map(|&s| (s, mastery)).collect();
    h.magic.spell_book = Some(book);
    h.magic.mana = 999;
    h.magic.max_mana = 999;
    h
}

/// `side`'s hero casts `name` at stack `t`, at the first turn of one of its stacks where the
/// engine accepts it, every other stack defending until then.
fn cast_at(state: &mut BattleState, r: &BattleRuleset, side: BattleSide, name: &str, t: StackId) {
    let cast = Command::Cast {
        spell: spell(r, name),
        target: h3_battle::CastTarget::Stack(t),
        second: h3_battle::CastTarget::None,
    };
    for _ in 0..40 {
        if validate(state, r, side, cast).is_ok() {
            apply(state, r, side, cast, &mut NoopSink).unwrap();
            return;
        }
        let acting = state.side_to_act().unwrap();
        apply(state, r, acting, Command::Defend, &mut NoopSink).unwrap();
    }
    panic!("{name} at {t:?} never accepted");
}

/// The id of the test spell `name` in `r`.
fn spell(r: &BattleRuleset, name: &str) -> SpellId {
    r.spell_by_key(&["t:", name].concat()).unwrap()
}

/// The test creatures' AI values (the others have none): the runner's stands for the Archers'
/// (creature 2), which the odds give a turret under BAT-QUIRK-25 (slip 20), the tower's own.
const AI_VALUES: [(usize, u32); 2] = [(RUNNER, 126), (TOWER, 400)];

/// The core ruleset of the test creatures, one hero type and the profiles `t:faithful` (the
/// original's slips in its estimates, BAT-QUIRK-25) and `t:fixed` (none), in `profile`.
fn core(profile: &str) -> Ruleset {
    let mut pack = ContentPack::new(manifest());
    pack.terrains = vec![TerrainDef {
        key: key("t:grass"),
        name: "Grass".into(),
    }];
    pack.creatures = CREATURES
        .iter()
        .enumerate()
        .map(
            |(handle, &(k, speed, fight_value, flags, ..))| CreatureDef {
                key: key(&["t:", k].concat()),
                speed,
                info: CreatureInfoDef {
                    fight_value,
                    ai_value: AI_VALUES
                        .iter()
                        .find(|&&(h, _)| h == handle)
                        .map_or(0, |&(_, v)| v),
                    flags: flags.iter().map(|f| f.to_string()).collect(),
                    ..CreatureInfoDef::default()
                },
            },
        )
        .collect();
    pack.factions = vec![FactionDef {
        key: key("t:town"),
        name: "Town".into(),
        native_terrain: None,
    }];
    pack.hero_classes = vec![HeroClassDef {
        key: key("t:knight"),
        name: "Knight".into(),
        faction: key("t:town"),
        primary: [1, 1, 1, 1],
        primary_gain: [[25, 25, 25, 25], [25, 25, 25, 25]],
        skill_weights: vec![],
        offer_intervals: [6, 4],
    }];
    pack.heroes = vec![HeroTypeDef {
        key: key("t:hero"),
        name: "Hero".into(),
        class: key("t:knight"),
        skills: vec![],
        spell_book: false,
        spells: vec![],
        army: vec![],
        war_machines: vec![],
        specialty: vec![],
    }];
    pack.resources = vec![ResourceDef {
        key: key("t:gold"),
        name: "Gold".into(),
    }];
    pack.spell_schools = ["fire", "air"]
        .into_iter()
        .map(|k| SpellSchoolDef {
            key: key(&["t:", k].concat()),
            name: k.into(),
            protection_rank: 0,
        })
        .collect();
    // Fire Wall: power × 10 + 10 at Basic, two hexes.
    pack.spells = vec![
        SpellDef {
            key: key("t:firewall"),
            name: "Fire Wall".into(),
            category: SpellCategory::Combat,
            level: 2,
            schools: vec![key("t:fire")],
            cost: [5; 4],
            power: 10,
            effect: [10, 10, 20, 50],
            granted_by_name_only: false,
        },
        // Fire Shield: the sultan's own (its value 20 %).
        SpellDef {
            key: key("t:fireshield"),
            name: "Fire Shield".into(),
            category: SpellCategory::Combat,
            level: 4,
            schools: vec![key("t:fire")],
            cost: [16; 4],
            power: 0,
            effect: [20, 20, 25, 30],
            granted_by_name_only: false,
        },
    ];
    pack.spells.extend(
        SPELLS
            .iter()
            .map(|&(k, level, effect, .., school)| SpellDef {
                key: key(&["t:", k].concat()),
                name: k.into(),
                category: SpellCategory::Combat,
                level,
                schools: vec![key(&["t:", school].concat())],
                cost: [5; 4],
                power: 0,
                effect,
                granted_by_name_only: false,
            }),
    );
    pack.spells.extend(
        EXTRA
            .iter()
            .map(|&(k, level, effect, power, category, ..)| SpellDef {
                key: key(&["t:", k].concat()),
                name: k.into(),
                category,
                level,
                schools: if category == SpellCategory::Ability {
                    vec![]
                } else {
                    vec![key("t:fire")]
                },
                cost: [5; 4],
                power,
                effect,
                granted_by_name_only: false,
            }),
    );
    // The bolt: a fire damage spell aimed at a stack, power × 10 + 10 at Basic.
    pack.spells.push(SpellDef {
        key: key("t:bolt"),
        name: "Bolt".into(),
        category: SpellCategory::Combat,
        level: 1,
        schools: vec![key("t:fire")],
        cost: [5; 4],
        power: 10,
        effect: [10, 10, 20, 50],
        granted_by_name_only: false,
    });
    pack.profiles = vec![
        ProfileDef {
            key: key("t:faithful"),
            quirks: vec![("quirk.battle.original_ai_slips".into(), true)],
        },
        ProfileDef {
            key: key("t:fixed"),
            quirks: vec![],
        },
    ];
    Ruleset::compile(&[pack], profile).unwrap()
}

/// The test ruleset in the profile `t:faithful`.
fn ruleset() -> BattleRuleset {
    ruleset_as("t:faithful")
}

/// The test ruleset in `profile`: [`core`] and its battle part on the original's field, without
/// morale and luck.
fn ruleset_as(profile: &str) -> BattleRuleset {
    let creatures = CREATURES
        .iter()
        .map(
            |&(k, _, _, _, [attack, defense, min, max], hit_points, shots, ref ability)| {
                BattleCreatureDef {
                    key: key(&["t:", k].concat()),
                    attack,
                    defense,
                    damage_min: min,
                    damage_max: max,
                    hit_points,
                    shots,
                    casts: 0,
                    abilities: ability.clone().into_iter().collect(),
                }
            },
        )
        .collect::<Vec<BattleCreatureDef>>();
    // The efreet are immune to fire spells too (the Fire Wall's test).
    let mut creatures = creatures;
    creatures[EFREET]
        .abilities
        .push(AbilityDef::SchoolImmunity {
            school: key("t:fire"),
        });
    creatures[SULTAN].abilities.push(AbilityDef::InnateSpell {
        spell: key("t:fireshield"),
        value: 20,
    });
    let genie = &mut creatures[GENIE];
    genie.casts = 3;
    genie.abilities.push(AbilityDef::HelpfulSpellCaster {
        mastery: 2,
        power: 6,
    });
    genie.abilities.extend(
        ["fireshield"]
            .into_iter()
            .chain(SPELLS.iter().map(|s| s.0))
            .map(|k| AbilityDef::DrawnSpell {
                spell: key(&["t:", k].concat()),
                weight: 1,
            }),
    );
    let fireshield = BattleSpellDef {
        key: key("t:fireshield"),
        polarity: SpellPolarity::Positive,
        magic: SpellMagic {
            target: SpellTarget::Stack,
            effects: vec![SpellEffect::Timed(TimedChange::FireShield)],
            ..SpellMagic::default()
        },
        counters: vec![],
        dispels: vec![],
        spares: vec![],
        tests_as: None,
        summons: None,
        ai: SpellAi::default(),
    };
    let firewall = BattleSpellDef {
        key: key("t:firewall"),
        polarity: SpellPolarity::Neutral,
        magic: SpellMagic {
            target: SpellTarget::Hex,
            obstacle: Some(ObstacleShape {
                steps: vec![HexStep::TopRight, HexStep::TopLeft],
                hexes: [2, 2, 3, 3],
                random: false,
                mirrored: true,
                separate: true,
                rounds: 2,
            }),
            effects: vec![SpellEffect::Obstacle(ObstacleKind::Fire)],
            ..SpellMagic::default()
        },
        counters: vec![],
        dispels: vec![],
        spares: vec![],
        tests_as: None,
        summons: None,
        ai: SpellAi::default(),
    };
    let mut spells = vec![firewall, fireshield];
    spells.extend(
        SPELLS.iter().map(
            |&(k, _, _, polarity, changes, mass_from, useful, _)| BattleSpellDef {
                key: key(&["t:", k].concat()),
                polarity,
                magic: SpellMagic {
                    target: SpellTarget::Stack,
                    effects: changes.iter().map(|&c| SpellEffect::Timed(c)).collect(),
                    mass_from,
                    useful,
                    ..SpellMagic::default()
                },
                counters: vec![],
                dispels: vec![],
                spares: vec![],
                tests_as: None,
                summons: None,
                ai: SpellAi::default(),
            },
        ),
    );
    spells.extend(
        EXTRA
            .iter()
            .map(
                |&(k, _, _, _, _, polarity, effects, permanent, dispels)| BattleSpellDef {
                    key: key(&["t:", k].concat()),
                    polarity,
                    magic: SpellMagic {
                        target: SpellTarget::Stack,
                        effects: effects.to_vec(),
                        duration: if permanent {
                            EffectDuration::Permanent
                        } else {
                            EffectDuration::Rounds
                        },
                        ..SpellMagic::default()
                    },
                    counters: vec![],
                    dispels: dispels.iter().map(|d| key(&["t:", d].concat())).collect(),
                    spares: vec![],
                    tests_as: None,
                    summons: None,
                    ai: SpellAi::default(),
                },
            ),
    );
    spells.push(BattleSpellDef {
        key: key("t:bolt"),
        polarity: SpellPolarity::Negative,
        magic: SpellMagic {
            target: SpellTarget::Stack,
            effects: vec![SpellEffect::Damage],
            ..SpellMagic::default()
        },
        counters: vec![],
        dispels: vec![],
        spares: vec![],
        tests_as: None,
        summons: None,
        ai: SpellAi::default(),
    });
    let pack = BattlePack {
        creatures,
        spells,
        battle: Some(BattleParamsDef {
            field_width: 17,
            field_height: 11,
            side_columns: 1,
            army_slots: 7,
            damage_roll_samples: 10,
            attack_bonus_per_point_percent: 5,
            attack_bonus_cap_percent: 300,
            defense_reduction_per_point_permille: 25,
            defense_reduction_cap_permille: 700,
            defend_bonus_percent: 20,
            defend_bonus_min: 1,
            ranged_full_damage_distance: 10,
            ranged_penalty_percent: 50,
            melee_penalty_percent: 50,
            morale_dice: 24,
            good_morale_chances: vec![0, 0, 0],
            bad_morale_chances: vec![0, 0, 0],
            luck_dice: 24,
            good_luck_chances: vec![0, 0, 0],
            war_machine_positions: vec![[1, 3], [1, 1], [1, 9]],
            loose_formation: (1..=7u8)
                .map(|n| (0..n).map(|i| i * 11 / n).collect())
                .collect(),
            tight_formation: (1..=7u8)
                .map(|n| (0..n).map(|i| i * 11 / n).collect())
                .collect(),
            tactics_zone_columns: vec![3, 5, 7],
            surrender: None,
            hero_casts_per_round: 1,
            side_stack_limit: 20,
            siege: Some(siege_field()),
        }),
        sieges: vec![SiegeDef {
            faction: key("t:town"),
            moat_damage: 70,
            second_moat: false,
            mines: None,
            hit_points: vec![2; 8],
        }],
        ..BattlePack::new(manifest())
    };
    BattleRuleset::compile(Arc::new(core(profile)), &[pack]).unwrap()
}

/// The siege's field: the original's (`docs/formats/installation.md` §7.27, §7.28).
fn siege_field() -> SiegeFieldDef {
    SiegeFieldDef {
        walls: vec![12, 29, 45, 62, 78, 96, 112, 130, 147, 165, 182],
        gate: 96,
        moat: vec![11, 28, 44, 61, 77, 95, 111, 129, 146, 164, 181],
        second_moat: vec![10, 27, 43, 60, 76, 94, 110, 128, 145, 163, 180],
        moat_defense_penalty: 3,
        mine_power: 3,
        teleport_moat: 2,
        teleport_walls: 3,
        catapult: 119,
        turret_count: 4,
        targets: [
            (WallKind::Tower, None, Some(255)),
            (WallKind::Wall, Some(29), None),
            (WallKind::Wall, Some(78), None),
            (WallKind::Gate, Some(96), None),
            (WallKind::Wall, Some(130), None),
            (WallKind::Wall, Some(182), None),
            (WallKind::Tower, None, Some(251)),
            (WallKind::Keep, None, Some(254)),
        ]
        .map(|(kind, wall, turret)| WallTarget { kind, wall, turret })
        .to_vec(),
        missed: vec![1, 2, 5, 4],
        automatic: vec![1, 2, 4, 5],
        fallback: vec![3, 7, 0, 6],
        castle_bonus: 1,
        ballistics: [
            ([5, 10, 25, 50], 1, [10, 60, 30]),
            ([7, 15, 30, 60], 1, [0, 50, 50]),
            ([7, 15, 30, 60], 2, [0, 50, 50]),
            ([10, 20, 40, 75], 2, [0, 0, 100]),
        ]
        .map(|(hit, shots, damage)| BallisticsRecord { hit, shots, damage })
        .to_vec(),
    }
}

/// The two armies of a setup: each stack's creature handle and count.
type Armies<'a> = (&'a [(usize, u32)], &'a [(usize, u32)]);

/// [`setup`] at the town's `fort` with `moat`, its turrets' count 4 plus `structures`.
fn at_town(
    r: &BattleRuleset,
    (attacker, defender): Armies<'_>,
    (fort, moat, structures): (FortLevel, Moat, u8),
) -> BattleSetup {
    BattleSetup {
        siege: Some(Siege {
            faction: r.faction_by_key("t:town").unwrap(),
            fort,
            moat,
            structures,
            escape_tunnel: false,
            morale: 0,
            luck: 0,
        }),
        ..setup(attacker, defender)
    }
}

fn creature(handle: usize) -> CreatureTypeId {
    CreatureTypeId::from_index(handle).unwrap()
}

fn army(stacks: &[(usize, u32)]) -> Army {
    Army {
        stacks: (0u8..)
            .zip(stacks)
            .map(|(slot, &(handle, count))| ArmyStack {
                slot,
                creature: creature(handle),
                count,
            })
            .collect(),
    }
}

/// A hero with `tactics`, commanding its Ballista and its First Aid Tent.
fn hero(tactics: u8) -> BattleHero {
    let mut hero = BattleHero::new(HeroTypeId::from_index(0).unwrap());
    hero.tactics = tactics;
    hero.war_machines.ballista = Some(BallistaControl {
        level: 1,
        shots: 1,
        double_damage_chance_bp: 0,
    });
    hero.war_machines.tent = Some(TentControl {
        heal_bonus: Rate::bp(0),
    });
    hero
}

/// A setup of the two armies without heroes, on Normal.
fn setup(attacker: &[(usize, u32)], defender: &[(usize, u32)]) -> BattleSetup {
    BattleSetup {
        attacker: army(attacker),
        defender: army(defender),
        difficulty: 1,
        ..BattleSetup::default()
    }
}

/// Plays `command` for the side to act.
fn play(state: &mut BattleState, r: &BattleRuleset, command: Command) {
    let side = state.side_to_act().unwrap();
    apply(state, r, side, command, &mut NoopSink).unwrap();
}

/// The active stack's creature handle.
fn active(state: &BattleState) -> usize {
    state
        .stack(state.active().unwrap())
        .unwrap()
        .creature
        .index()
}

/// The policy's command for the side to act, from a fixed stream.
fn decide(state: &BattleState, r: &BattleRuleset) -> Command {
    choose(
        state,
        r,
        state.side_to_act().unwrap(),
        &mut Rng::from_seed(5),
    )
}

// covers: BAT-AI-001, BAT-AI-002
#[test]
fn a_shooter_shoots_the_computers_target_at_its_front_hex() {
    let r = ruleset();
    // The imps (speed 5) act first, then the defender's archers (4), then the gogs (3): the
    // imps defend, then the archers shoot.
    let mut state =
        BattleState::new(&r, &setup(&[(IMP, 10), (GOG, 2)], &[(ARCHER, 5)]), 1).unwrap();
    assert_eq!(active(&state), IMP);
    play(&mut state, &r, Command::Defend);
    assert_eq!(active(&state), ARCHER);
    let archer = state.active().unwrap();
    let target = computer::shot_target(&state, &r, archer).unwrap();
    let field = Field::new(r.battle().field);
    let at = state.stack(target).unwrap().footprint(field).front;
    assert_eq!(decide(&state, &r), Command::Shoot { target, at });
    assert!(validate(&state, &r, BattleSide::Defender, decide(&state, &r)).is_ok());
}

// covers: BAT-AI-002
#[test]
fn the_shooter_takes_the_better_value() {
    let r = ruleset();
    // One gog (Fight Value 900) and one imp (100): the archers kill the gog's worth first.
    let mut state =
        BattleState::new(&r, &setup(&[(IMP, 1), (GOG, 1)], &[(ARCHER, 20)]), 2).unwrap();
    play(&mut state, &r, Command::Defend);
    let Command::Shoot { target, .. } = decide(&state, &r) else {
        panic!("no shot");
    };
    assert_eq!(state.stack(target).unwrap().creature.index(), GOG);
}

/// The creature handle of stack `id`.
fn handle(state: &BattleState, id: StackId) -> usize {
    state.stack(id).unwrap().creature.index()
}

/// The battle of `attacker` against the defender's fast walker `escort` and its thirty area
/// shooters `shooter`, at the shooters' first turn: the walker acts first and, when `near` picks
/// a hex it may walk to from the attacker's first stack's footprint, walks there (else it
/// defends); the attacker's stacks faster than the shooters defend.
fn escorted(
    r: &BattleRuleset,
    attacker: &[(usize, u32)],
    (escort, count): (usize, u32),
    shooter: usize,
    near: Option<fn(Field, h3_battle::Footprint, Hex) -> bool>,
) -> BattleState {
    let mut state =
        BattleState::new(r, &setup(attacker, &[(escort, count), (shooter, 30)]), 8).unwrap();
    assert_eq!(active(&state), escort);
    let field = Field::new(r.battle().field);
    let first = state.stacks()[0].footprint(field);
    let mut legal = LegalCommands::new();
    legal_commands(&state, r, BattleSide::Defender, &mut legal);
    let walk = near.map(|near| {
        legal
            .iter()
            .find_map(|c| match *c {
                Command::Move { to } if near(field, first, to) => Some(Command::Move { to }),
                _ => None,
            })
            .unwrap()
    });
    play(&mut state, r, walk.unwrap_or(Command::Defend));
    while active(&state) != shooter {
        play(&mut state, r, Command::Defend);
    }
    state
}

/// Beside the attacker's first stack (its front hex).
fn beside(field: Field, first: h3_battle::Footprint, hex: Hex) -> bool {
    field.distance(hex, first.front) == 1
}

// covers: BAT-AI-003
#[test]
fn an_area_shooter_does_not_hit_its_own_side_for_less() {
    for profile in ["t:faithful", "t:fixed"] {
        let r = ruleset_as(profile);
        // Alone, the gog is the magogs' better shot (as the archers', above).
        let state = escorted(&r, &[(GOG, 1), (IMP, 1)], (RUNNER, 30), MAGOG, None);
        let Command::Shoot { target, at } = decide(&state, &r) else {
            panic!("no shot");
        };
        assert_eq!(
            (handle(&state, target), at),
            (GOG, state.stack(target).unwrap().hex)
        );
        // With thirty runners beside it, the gog can strike at once (its value is undivided),
        // but the fireball's kills among the runners cost more than it is worth: its value is
        // below 0, so it is no target; the imp is shot instead.
        let state = escorted(&r, &[(GOG, 1), (IMP, 1)], (RUNNER, 30), MAGOG, Some(beside));
        let Command::Shoot { target, .. } = decide(&state, &r) else {
            panic!("no shot");
        };
        assert_eq!(handle(&state, target), IMP, "{profile}");
        // With the gog alone there is no target: the magogs defend.
        let state = escorted(&r, &[(GOG, 1)], (RUNNER, 30), MAGOG, Some(beside));
        assert_eq!(
            unchecked(&state, &r, BattleSide::Defender, &mut Rng::from_seed(1)),
            Command::Defend
        );
    }
}

// covers: BAT-AI-003
#[test]
fn the_original_estimates_an_area_shot_without_the_immunities() {
    // The fireball spares fire-immune stacks (BAT-RNG-004), the death cloud lifeless ones; the
    // original's estimate (slip 1) counts the efreets beside the gog for the magogs and the golems
    // for the undead liches, so it finds no target: they defend. Without the slip they shoot.
    for (escort, shooter) in [(EFREET, MAGOG), (GOLEM, LICH)] {
        for (profile, shoots) in [("t:faithful", false), ("t:fixed", true)] {
            let r = ruleset_as(profile);
            let state = escorted(&r, &[(GOG, 1)], (escort, 30), shooter, Some(beside));
            let command = decide(&state, &r);
            assert_eq!(
                matches!(command, Command::Shoot { .. }),
                shoots,
                "{profile}: {command:?}"
            );
        }
    }
    // Both leave the undead out when they are beside the target (the slip: an undead shooter
    // leaves out the undead; the rule: the cloud spares them), but not the undead target itself.
    for profile in ["t:faithful", "t:fixed"] {
        let r = ruleset_as(profile);
        let state = escorted(&r, &[(GOG, 1)], (ZOMBIE, 30), LICH, Some(beside));
        assert!(
            matches!(decide(&state, &r), Command::Shoot { .. }),
            "{profile}"
        );
        let mut state = BattleState::new(&r, &setup(&[(ZOMBIE, 5)], &[(LICH, 10)]), 9).unwrap();
        play(&mut state, &r, Command::Defend);
        let Command::Shoot { target, .. } = decide(&state, &r) else {
            panic!("no shot");
        };
        assert_eq!(handle(&state, target), ZOMBIE);
    }
}

/// Beside the attacker's first stack's position (its rear hex) but not its front hex.
fn behind(field: Field, first: h3_battle::Footprint, hex: Hex) -> bool {
    first
        .rear
        .is_some_and(|rear| field.distance(hex, rear) == 1 && field.distance(hex, first.front) > 1)
}

// covers: BAT-AI-003
#[test]
fn an_area_shot_at_a_two_hex_stack_takes_its_better_hex() {
    let r = ruleset();
    let field = Field::new(r.battle().field);
    // Alone, the cavalier is worth the same at both its hexes: the shot aims at its position.
    let state = escorted(&r, &[(CAVALIER, 1), (IMP, 1)], (RUNNER, 30), MAGOG, None);
    let cavalier = state.stacks()[0].footprint(field);
    let shot = |target, at| Command::Shoot { target, at };
    assert_eq!(decide(&state, &r), shot(StackId(0), cavalier.rear.unwrap()));
    // With the runners beside its position only, its front hex is worth strictly more.
    let state = escorted(
        &r,
        &[(CAVALIER, 1), (IMP, 1)],
        (RUNNER, 30),
        MAGOG,
        Some(behind),
    );
    assert_eq!(decide(&state, &r), shot(StackId(0), cavalier.front));
}

// covers: BAT-AI-001, BAT-AI-005
#[test]
fn a_walker_attacks_and_a_blocked_shooter_strikes_back() {
    let r = ruleset();
    // The runner (speed 20) reaches the archers in its first turn and attacks them.
    let mut state = BattleState::new(&r, &setup(&[(RUNNER, 1)], &[(ARCHER, 5)]), 3).unwrap();
    assert_eq!(active(&state), RUNNER);
    let attack = decide(&state, &r);
    assert!(
        matches!(
            attack,
            Command::Attack {
                target: StackId(1),
                ..
            }
        ),
        "{attack:?}"
    );
    assert!(validate(&state, &r, BattleSide::Attacker, attack).is_ok());
    play(&mut state, &r, attack);
    // Blocked by the runner beside them, the archers cannot shoot: a walker's turn, which strikes
    // the runner from where they stand.
    assert_eq!(active(&state), ARCHER);
    let archers = state.active().unwrap();
    assert!(!computer::shoots(&state, &r, archers));
    let from = state.stack(archers).unwrap().hex;
    assert_eq!(
        decide(&state, &r),
        Command::Attack {
            target: StackId(0),
            from
        }
    );
}

// covers: BAT-AI-001, BAT-AI-002
#[test]
fn the_commanded_ballista_shoots_and_the_tent_defends() {
    let r = ruleset();
    // The runner wounds the gogs in its first turn, so that the commanded tent has a patient and
    // takes its turns (BAT-WAR-007); the gogs defend (the walker is not under test here).
    let mut setup = setup(&[(GOG, 2)], &[(RUNNER, 1)]);
    setup.heroes = [Some(hero(0)), None];
    setup.war_machines = [vec![creature(BALLISTA), creature(TENT)], vec![]];
    let mut state = BattleState::new(&r, &setup, 4).unwrap();
    let mut seen = [false; 2];
    let mut legal = LegalCommands::new();
    for _ in 0..8 {
        let Some(side) = state.side_to_act() else {
            break;
        };
        let command = if side == BattleSide::Defender {
            legal_commands(&state, &r, side, &mut legal);
            legal
                .iter()
                .copied()
                .find(|c| {
                    matches!(
                        c,
                        Command::Attack {
                            target: StackId(0),
                            ..
                        }
                    )
                })
                .unwrap_or(Command::Defend)
        } else if active(&state) == GOG {
            Command::Defend
        } else {
            decide(&state, &r)
        };
        match active(&state) {
            BALLISTA => {
                seen[0] = true;
                assert!(matches!(command, Command::Shoot { .. }), "{command:?}");
            }
            TENT => {
                seen[1] = true;
                assert_eq!(command, Command::Defend);
            }
            _ => {}
        }
        play(&mut state, &r, command);
    }
    assert_eq!(seen, [true, true]);
}

// covers: BAT-AI-001
#[test]
fn the_tactics_phase_ends_at_once() {
    let r = ruleset();
    let mut setup = setup(&[(IMP, 3)], &[(GOG, 2)]);
    setup.heroes = [Some(hero(2)), None];
    let state = BattleState::new(&r, &setup, 5).unwrap();
    assert!(state.tactics().is_some());
    assert_eq!(decide(&state, &r), Command::EndTactics);
}

// covers: BAT-AI-001
#[test]
fn the_other_side_and_a_battle_that_is_over_get_defend() {
    let r = ruleset();
    let mut state = BattleState::new(&r, &setup(&[(RUNNER, 50)], &[(IMP, 1)]), 6).unwrap();
    let other = state.side_to_act().unwrap().opponent();
    let mut rng = Rng::from_seed(1);
    assert_eq!(unchecked(&state, &r, other, &mut rng), Command::Defend);
    assert_eq!(choose(&state, &r, other, &mut rng), Command::Defend);
    for _ in 0..10 {
        let Some(side) = state.side_to_act() else {
            break;
        };
        let mut legal = LegalCommands::new();
        legal_commands(&state, &r, side, &mut legal);
        let attack = legal
            .iter()
            .copied()
            .find(|c| matches!(c, Command::Attack { .. }))
            .unwrap_or(Command::Defend);
        play(&mut state, &r, attack);
    }
    assert!(state.is_over());
    for side in BattleSide::BOTH {
        assert_eq!(unchecked(&state, &r, side, &mut rng), Command::Defend);
    }
}

// covers: BAT-AI-001
#[test]
fn a_refused_choice_falls_back_to_defend() {
    let r = ruleset();
    let state = BattleState::new(&r, &setup(&[(IMP, 3)], &[(GOG, 2)]), 7).unwrap();
    let side = state.side_to_act().unwrap();
    let refused = Command::Shoot {
        target: StackId(1),
        at: Hex(0),
    };
    assert!(validate(&state, &r, side, refused).is_err());
    assert_eq!(checked(&state, &r, side, refused), Command::Defend);
    assert_eq!(checked(&state, &r, side, Command::Wait), Command::Wait);
    assert_eq!(POLICY_VERSION, 8);
}

/// The setup of `setup` on `difficulty`.
fn on(difficulty: u8, mut setup: BattleSetup) -> BattleSetup {
    setup.difficulty = difficulty;
    setup
}

/// Whether the footprint of the active stack at `to` would touch stack `id`.
fn touches(state: &BattleState, r: &BattleRuleset, to: Hex, id: StackId) -> bool {
    let field = Field::new(r.battle().field);
    let me = state.stack(state.active().unwrap()).unwrap();
    field
        .footprint(to, me.is_two_hex(), me.side)
        .is_some_and(|fp| field.touches(fp, state.stack(id).unwrap().footprint(field)))
}

// covers: BAT-AI-005, BAT-AI-006
#[test]
fn a_harpy_flies_to_block_shooters_it_would_hardly_hurt() {
    let r = ruleset();
    for difficulty in [1, 2] {
        // Fifty archers: blocking them is worth more than one harpy's strike, so the harpy takes
        // the plan's value alone (the Harpies' larger) and only goes to the plan's hex; once
        // beside them it blocks them already, which is worth 1 (§7.37), and it strikes. The
        // archers (not under test) defend.
        let mut state = BattleState::new(
            &r,
            &on(difficulty, setup(&[(HARPY, 1)], &[(ARCHER, 50)])),
            10,
        )
        .unwrap();
        let archers = StackId(1);
        let mut moves = 0;
        loop {
            assert_eq!(active(&state), HARPY);
            let command = decide(&state, &r);
            if let Command::Move { to } = command {
                moves += 1;
                assert!(validate(&state, &r, BattleSide::Attacker, command).is_ok());
                play(&mut state, &r, command);
                play(&mut state, &r, Command::Defend);
                if touches(&state, &r, to, archers) {
                    break;
                }
                continue;
            }
            panic!("{command:?}");
        }
        assert!(moves >= 2, "{moves}");
        let from = state.stacks()[0].hex;
        assert_eq!(
            decide(&state, &r),
            Command::Attack {
                target: archers,
                from
            }
        );
        // With the hex it stands on scored high (T3b's scores, here by hand), the plan's value is
        // worth more again: it stays, so nothing is chosen.
        let mut scores = [0; 256];
        scores[usize::from(from.0)] = 1_000_000;
        let walker = Walker::new(&state, &r, (StackId(0), BattleSide::Attacker)).scored(scores);
        assert_eq!(walker.choose(&mut Rng::from_seed(1), &mut 0), None);
    }
}

/// A walker of `handles` against the defender's `enemy` on Hard, at the attacker's first turn.
fn walking(r: &BattleRuleset, handle: usize, enemy: usize) -> BattleState {
    BattleState::new(r, &on(2, setup(&[(handle, 10)], &[(enemy, 10)])), 11).unwrap()
}

// covers: BAT-AI-006
#[test]
fn the_walk_holds_back_from_worse_hexes_on_hard() {
    let r = ruleset();
    // The imps (speed 5) walk towards the gogs across the field, on Hard.
    let state = walking(&r, IMP, GOG);
    assert_eq!(active(&state), IMP);
    let me = StackId(0);
    let start = state.stacks()[0].hex;
    let far = state.stacks()[1].hex;
    let side = BattleSide::Attacker;
    let walk = |scores: [i32; 256], careful| {
        Walker::new(&state, &r, (me, side))
            .scored(scores)
            .walk(Hex(far.0 - 1), careful)
    };
    // No scores: the walk goes its whole speed.
    let Some(Command::Move { to }) = walk([0; 256], true) else {
        panic!("no walk");
    };
    let field = Field::new(r.battle().field);
    assert_eq!(field.distance(start, to), 5);
    // Every hex worse than the start: it holds back and waits for the wait phase.
    let mut scores = [-1; 256];
    scores[usize::from(start.0)] = 0;
    assert_eq!(walk(scores, true), Some(Command::Wait));
    let mut scores = [-1; 256];
    scores[usize::from(start.0)] = -2;
    // Every hex better than the start: it goes on.
    assert_eq!(walk(scores, true), Some(Command::Move { to }));
    // The last hex worse than one before it: it stops there and waits, as no end is worse than
    // the start.
    let mut scores = [0; 256];
    scores[usize::from(to.0)] = -1;
    assert_eq!(walk(scores, true), Some(Command::Wait));
    // Not careful (Normal): once an end is found it still refuses a worse hex from the step on
    // that leaves at most `5 × ⌊(n − 1) / 5⌋` of the path's `n` steps (whole turns), as the
    // fifth step does here: it stops one short and moves there.
    let Some(Command::Move { to: short }) = walk(scores, false) else {
        panic!("no walk");
    };
    assert_eq!(
        (field.distance(start, short), field.distance(short, to)),
        (4, 1)
    );
    // No path (the gogs' own hex): nothing.
    assert_eq!(walk([0; 256], false), Some(Command::Move { to }));
    assert_eq!(Walker::new(&state, &r, (me, side)).walk(far, false), None);
}

/// The handles an army draws from: every creature but the war machines.
const ARMY: [usize; 14] = [
    IMP, GOG, RUNNER, ARCHER, MAGOG, LICH, EFREET, GOLEM, ZOMBIE, CAVALIER, HARPY, HYDRA, CERBERUS,
    DRAGON,
];

proptest! {
    #![proptest_config(Config::with_cases(48))]

    /// Exit criterion 1 on the test ruleset: random battles of both sides, every difficulty,
    /// with and without heroes (Tactics, a commanded Ballista and First Aid Tent), on the field
    /// or at the town (every fort level, moat and turrets): at every step
    /// the original's choice for the side to act is a command the engine accepts (no fallback
    /// fires), and the same state and stream give the same command and leave the same stream.
    /// Two steps in three play it, the third a random legal command that stays in the battle.
    // covers: BAT-AI-001, BAT-AI-002, BAT-AI-003, BAT-AI-005, BAT-AI-010, BAT-AI-011
    #[test]
    fn the_choice_is_legal_and_the_same_for_the_same_stream(
        seed in any::<u64>(),
        draws in any::<u64>(),
        fixed: bool,
    ) {
        let r = ruleset_as(if fixed { "t:fixed" } else { "t:faithful" });
        let mut rng = Rng::from_seed(draws);
        let draw_army = |rng: &mut Rng| -> Vec<(usize, u32)> {
            (0..=rng.below(4))
                .map(|_| (ARMY[rng.below(14) as usize], 1 + rng.below(40)))
                .collect()
        };
        let attacker = draw_army(&mut rng);
        let defender = draw_army(&mut rng);
        let mut setup = setup(&attacker, &defender);
        for side in 0..2 {
            if rng.below(2) == 0 {
                setup.heroes[side] = Some(hero(u8::try_from(rng.below(4)).unwrap()));
                setup.war_machines[side] = [BALLISTA, TENT]
                    .into_iter()
                    .filter(|_| rng.below(2) == 0)
                    .map(creature)
                    .collect();
            }
            setup.human[side] = rng.below(2) == 0;
        }
        setup.difficulty = u8::try_from(rng.below(5)).unwrap();
        // One battle in three at the town (BAT-AI-005's besieger's walk, BAT-AI-011).
        if rng.below(3) == 0 {
            let fort = [FortLevel::Fort, FortLevel::Citadel, FortLevel::Castle][rng.below(3) as usize];
            let moat = if fort == FortLevel::Fort {
                Moat::Absent
            } else {
                [Moat::Absent, Moat::Harmless, Moat::Damaging][rng.below(3) as usize]
            };
            let structures = u8::try_from(rng.below(4)).unwrap();
            setup.siege = at_town(&r, (&attacker, &defender), (fort, moat, structures)).siege;
        }
        let mut state = BattleState::new(&r, &setup, seed).unwrap();
        let mut seat = Rng::from_seed(seed ^ draws);
        let mut legal = LegalCommands::new();
        for _ in 0..300 {
            let Some(side) = state.side_to_act() else {
                break;
            };
            let proposed = unchecked(&state, &r, side, &mut seat.clone());
            prop_assert!(
                validate(&state, &r, side, proposed).is_ok(),
                "refused: {proposed:?}"
            );
            let (mut a, mut b) = (seat.clone(), seat.clone());
            let command = choose(&state, &r, side, &mut a);
            prop_assert_eq!(command, choose(&state, &r, side, &mut b));
            prop_assert_eq!(command, proposed);
            prop_assert_eq!(&a, &b);
            seat = a;
            let played = if rng.below(3) == 0 {
                legal_commands(&state, &r, side, &mut legal);
                let staying: Vec<Command> =
                    legal.iter().copied().filter(|c| !c.leaves_battle()).collect();
                staying[rng.below(u32::try_from(staying.len()).unwrap()) as usize]
            } else {
                command
            };
            apply(&mut state, &r, side, played, &mut NoopSink).unwrap();
        }
    }
}

/// The walker `id`'s hex scores of `side` (BAT-AI-007).
fn scores_of(state: &BattleState, r: &BattleRuleset, id: StackId) -> [i32; 256] {
    let side = state.stack(id).unwrap().side;
    let ctx = computer::context(state, r, side);
    crate::scores::build(state, r, (id, side), &ctx).0
}

/// The floor of the walker `id`'s scores: minus the value of its whole hit points.
fn floor_of(state: &BattleState, r: &BattleRuleset, id: StackId) -> i32 {
    let ctx = computer::context(state, r, state.stack(id).unwrap().side);
    let hp = computer::hit_points(state, id);
    -computer::damage_value(state, r, &ctx, (id, hp), (false, false))
}

/// The strike value of `a` on the walker `t` in the walker's context (0x435C70, no run-up).
fn strike_on(state: &BattleState, r: &BattleRuleset, (a, t): (StackId, StackId)) -> i32 {
    let ctx = computer::context(state, r, state.stack(t).unwrap().side);
    computer::strike_value(state, r, &ctx, (a, t), 0)
}

/// The hexes of the test field.
fn all_hexes() -> impl Iterator<Item = Hex> {
    (0u8..187).map(Hex)
}

// covers: BAT-AI-007
#[test]
fn the_hexes_an_enemy_reaches_lose_its_strike_value_from_normal_on() {
    let r = ruleset();
    for difficulty in [0, 1] {
        let state =
            BattleState::new(&r, &on(difficulty, setup(&[(IMP, 10)], &[(GOG, 10)])), 12).unwrap();
        assert_eq!(active(&state), IMP);
        let (me, gog) = (StackId(0), StackId(1));
        let scores = scores_of(&state, &r, me);
        if difficulty == 0 {
            // Easy for a computer side: no reach.
            assert!(scores.iter().all(|&s| s == 0));
            continue;
        }
        // The gog (speed 3) at hex 15 reaches with speed 4: hex 11, not 10.
        let map = computer::reach_map(&state, &r, gog);
        assert_eq!(state.stacks()[1].hex, Hex(15));
        assert!(map.reached(Hex(11)) && !map.reached(Hex(10)));
        // Ten imps against the gogs are desperate: their kills count 1000 each, beyond their
        // whole value, so the reached hexes stop at the floor.
        let (t, floor) = (strike_on(&state, &r, (gog, me)), floor_of(&state, &r, me));
        assert!(-t < floor && floor < 0, "{t} {floor}");
        for hex in all_hexes() {
            let want = if map.reached(hex) { floor } else { 0 };
            assert_eq!(scores[hex.index()], want, "{hex:?}");
        }
    }
}

// covers: BAT-AI-007
#[test]
fn an_enemy_within_a_turn_of_the_side_adds_nothing() {
    let r = ruleset();
    // The runner (speed 20) goes next to the gogs: they can strike it this turn, so they add
    // nothing to the imps' scores (on Normal: no multi-headed strike, no crowding).
    let mut state =
        BattleState::new(&r, &setup(&[(RUNNER, 1), (IMP, 10)], &[(GOG, 10)]), 13).unwrap();
    assert_eq!(active(&state), RUNNER);
    play(&mut state, &r, Command::Move { to: Hex(14) });
    assert_eq!(active(&state), IMP);
    assert!(scores_of(&state, &r, StackId(1)).iter().all(|&s| s == 0));
}

// covers: BAT-AI-007
#[test]
fn a_two_hex_enemy_also_lowers_the_hex_beside_its_reach() {
    let r = ruleset();
    // The defender's cavaliers (speed 2) at 15 cover 15 and 14; with speed 3 their positions
    // reach 12 (covering 12 and 11), and the hex beside it on their facing side, 11, which no
    // position reaches, is lowered too; 10 is not.
    let state = BattleState::new(&r, &setup(&[(IMP, 10)], &[(CAVALIER, 10)]), 14).unwrap();
    let (me, cavalier) = (StackId(0), StackId(1));
    let map = computer::reach_map(&state, &r, cavalier);
    assert!(map.reached(Hex(12)) && !map.reached(Hex(11)));
    let scores = scores_of(&state, &r, me);
    let lowered = (-strike_on(&state, &r, (cavalier, me))).max(floor_of(&state, &r, me));
    assert!(lowered < 0);
    assert_eq!((scores[12], scores[11], scores[10]), (lowered, lowered, 0));
    for hex in all_hexes().filter(|&h| map.reached(h)) {
        assert_eq!(scores[hex.index()], lowered, "{hex:?}");
    }
}

// covers: BAT-AI-007
#[test]
fn the_reach_of_several_enemies_adds_up() {
    let r = ruleset();
    // Ten imps with five dragons (not desperate) against two stacks of gogs at 15 and 100 (rows
    // 0 and 5), out of reach of the dragons: where both reach, their two values add up, above
    // the floor. The dragons defend first.
    let mut state = BattleState::new(
        &r,
        &setup(&[(IMP, 10), (DRAGON, 5)], &[(GOG, 10), (GOG, 10)]),
        15,
    )
    .unwrap();
    play(&mut state, &r, Command::Defend);
    assert_eq!(active(&state), IMP);
    let me = StackId(0);
    let (a, b) = (StackId(2), StackId(3));
    let floor = floor_of(&state, &r, me);
    let (ta, tb) = (
        strike_on(&state, &r, (a, me)),
        strike_on(&state, &r, (b, me)),
    );
    assert!(ta > 0 && tb > 0 && -ta - tb > floor, "{ta} {tb} {floor}");
    let (ma, mb) = (
        computer::reach_map(&state, &r, a),
        computer::reach_map(&state, &r, b),
    );
    let scores = scores_of(&state, &r, me);
    let mut both = 0;
    for hex in all_hexes() {
        let want = match (ma.reached(hex), mb.reached(hex)) {
            (true, true) => -ta - tb,
            (true, false) => -ta,
            (false, true) => -tb,
            (false, false) => 0,
        };
        both += u32::from(ma.reached(hex) && mb.reached(hex));
        assert_eq!(scores[hex.index()], want, "{hex:?}");
    }
    assert!(both > 0);
}

/// The imps' scores after `escorts` (the attacker's first stacks, faster than everything else)
/// went to `hexes`, on `difficulty`, the imps' own side human if `human`, against `enemy`.
fn escorted_imps(
    r: &BattleRuleset,
    (escorts, hexes): (&[(usize, u32)], &[u8]),
    (imps, enemy): (u32, (usize, u32)),
    (difficulty, human): (u8, bool),
) -> (BattleState, [i32; 256]) {
    let mut army = escorts.to_vec();
    army.push((IMP, imps));
    let mut setup = on(difficulty, setup(&army, &[enemy]));
    setup.human[0] = human;
    let mut state = BattleState::new(r, &setup, 16).unwrap();
    for &to in hexes {
        play(&mut state, r, Command::Move { to: Hex(to) });
    }
    assert_eq!(active(&state), IMP);
    let me = StackId(u8::try_from(escorts.len()).unwrap());
    let scores = scores_of(&state, r, me);
    (state, scores)
}

// covers: BAT-AI-007
#[test]
fn a_multi_headed_enemy_lowers_where_it_would_strike_the_side_on_hard_for_a_computer() {
    let r = ruleset();
    // The runner goes next to the hydra's front hex (the defender's hydra at 15 covers 15 and 14):
    // the hydra can strike it this turn. On Hard for a computer side, the hexes its strike would
    // hit from where it can strike the runner lose its strike value on the imps; on Normal, or for
    // a human side, nothing.
    let hydra = (HYDRA, 1);
    let lowered = |state: &BattleState| {
        let me = StackId(1);
        (-strike_on(state, &r, (StackId(2), me))).max(floor_of(state, &r, me))
    };
    let (state, scores) = escorted_imps(&r, (&[(RUNNER, 1)], &[13]), (10, hydra), (2, false));
    let low = lowered(&state);
    assert!(low < 0);
    let hit: Vec<Hex> = all_hexes().filter(|h| scores[h.index()] != 0).collect();
    assert!(!hit.is_empty());
    let field = Field::new(r.battle().field);
    for &hex in &hit {
        assert_eq!(scores[hex.index()], low, "{hex:?}");
        // Within two hexes of the runner: around a position next to it.
        assert!(field.distance(hex, Hex(13)) <= 3, "{hex:?}");
    }
    for (difficulty, human) in [(1, false), (2, true)] {
        let (_, scores) = escorted_imps(
            &r,
            (&[(RUNNER, 1)], &[13]),
            (10, hydra),
            (difficulty, human),
        );
        assert!(scores.iter().all(|&s| s == 0), "{difficulty} {human}");
    }
    // A cerberus strikes three directions: fewer hexes.
    let (_, cerberus) = escorted_imps(&r, (&[(RUNNER, 1)], &[14]), (10, (CERBERUS, 1)), (2, false));
    let three = all_hexes().filter(|h| cerberus[h.index()] != 0).count();
    assert!(three > 0 && three < hit.len(), "{three} {}", hit.len());
    // Two stacks it can strike (the runner at 13, the efreet at 30 below it), a hundred imps
    // (above the floor): a hex the second gives again loses 1 more.
    let (state, scores) = escorted_imps(
        &r,
        (&[(RUNNER, 1), (EFREET, 1)], &[13, 30]),
        (100, hydra),
        (2, false),
    );
    let me = StackId(2);
    let t = strike_on(&state, &r, (StackId(3), me));
    assert!(-t > floor_of(&state, &r, me) + 1, "{t}");
    assert!(scores.contains(&-t) && scores.contains(&(-t - 1)), "{t}");
}

// covers: BAT-AI-007
#[test]
fn an_attackers_multi_headed_enemy_lowers_where_it_would_strike_the_defender() {
    let r = ruleset();
    // The sides swapped: the attacker's hydra (at 1, its front hex 2) faces right; the defender's
    // runner goes next to its front hex, and on Hard the defender's imps' hexes around the runner
    // that the hydra's strike would hit lose its strike value.
    let setup = on(2, setup(&[(HYDRA, 1)], &[(RUNNER, 1), (IMP, 10)]));
    let mut state = BattleState::new(&r, &setup, 16).unwrap();
    assert_eq!(active(&state), RUNNER);
    play(&mut state, &r, Command::Move { to: Hex(3) });
    assert_eq!(active(&state), IMP);
    let me = StackId(2);
    let scores = scores_of(&state, &r, me);
    let low = (-strike_on(&state, &r, (StackId(0), me))).max(floor_of(&state, &r, me));
    assert!(low < 0);
    let hit: Vec<Hex> = all_hexes().filter(|h| scores[h.index()] != 0).collect();
    assert!(!hit.is_empty());
    let field = Field::new(r.battle().field);
    for &hex in &hit {
        assert_eq!(scores[hex.index()], low, "{hex:?}");
        assert!(field.distance(hex, Hex(3)) <= 3, "{hex:?}");
    }
}

// covers: BAT-AI-007
#[test]
fn a_multi_headed_strike_that_costs_the_enemy_raises_the_hexes_with_the_slip() {
    // Five hundred imps would kill the hydra in their retaliation: its strike is worth less than
    // nothing to it. The original subtracts it with its sign untested (slip 3): the hexes gain;
    // fixed, they keep 0.
    for (profile, slip) in [("t:faithful", true), ("t:fixed", false)] {
        let r = ruleset_as(profile);
        let (state, scores) =
            escorted_imps(&r, (&[(RUNNER, 1)], &[13]), (500, (HYDRA, 1)), (2, false));
        let t = strike_on(&state, &r, (StackId(2), StackId(1)));
        assert!(t < 0, "{t}");
        let raised: Vec<i32> = scores.iter().copied().filter(|&s| s != 0).collect();
        if slip {
            assert!(
                !raised.is_empty() && raised.iter().all(|&s| s == -t),
                "{raised:?}"
            );
        } else {
            assert!(raised.is_empty(), "{raised:?}");
        }
    }
}

// covers: BAT-AI-007
#[test]
fn the_hexes_around_the_side_lose_the_area_threat_on_hard() {
    let r = ruleset();
    // The defender's magogs can shoot: their fireball is the area threat on the imps. On Hard the
    // hexes around the imps' friends lose its value, a hex next to both of them 1 more; on Normal
    // nothing. The runner goes next to the gog (at 52, row 3) to share hexes.
    let magogs = (MAGOG, 20);
    let (state, scores) = escorted_imps(
        &r,
        (&[(RUNNER, 1), (GOG, 1)], &[53]),
        (10, magogs),
        (2, false),
    );
    let me = StackId(2);
    let floor = floor_of(&state, &r, me);
    let field = Field::new(r.battle().field);
    let threat = {
        let ctx = computer::context(&state, &r, BattleSide::Attacker);
        let damage = computer::damage_estimate(&state, &r, (StackId(3), me), 20, true, 0);
        computer::damage_value(&state, &r, &ctx, (me, damage), (false, false))
    };
    assert!(threat > 0);
    let low = (-threat).max(floor);
    let near = |a: Hex| field.distance(a, state.stacks()[0].hex) == 1;
    let near_gog = |a: Hex| field.distance(a, state.stacks()[1].hex) == 1;
    for hex in all_hexes() {
        let want = match (near(hex), near_gog(hex)) {
            (true, true) => (low - 1).max(floor),
            (true, false) | (false, true) => low,
            (false, false) => 0,
        };
        assert_eq!(scores[hex.index()], want, "{hex:?}");
    }
    assert!(all_hexes().any(|h| near(h) && near_gog(h)));
    let (_, normal) = escorted_imps(
        &r,
        (&[(RUNNER, 1), (GOG, 1)], &[53]),
        (10, magogs),
        (1, false),
    );
    assert!(normal.iter().all(|&s| s == 0));
}

// covers: BAT-AI-007
#[test]
fn a_breath_attackers_line_and_an_enemy_dragons_threat() {
    let r = ruleset();
    // The imps' own dragon at 0 (it defends first): the hexes behind its neighbours, in each
    // direction, lose the value of its breath on the imps on Hard; the archers only shoot (in
    // the mask, no area threat).
    let mut state = BattleState::new(
        &r,
        &on(2, setup(&[(DRAGON, 1), (IMP, 10)], &[(ARCHER, 5)])),
        17,
    )
    .unwrap();
    assert_eq!(active(&state), DRAGON);
    play(&mut state, &r, Command::Defend);
    assert_eq!(active(&state), IMP);
    let (dragon, me) = (StackId(0), StackId(1));
    let field = Field::new(r.battle().field);
    let k = {
        let ctx = computer::context(&state, &r, BattleSide::Attacker);
        let damage = computer::damage_estimate(&state, &r, (dragon, me), 1, false, 0);
        computer::damage_value(&state, &r, &ctx, (me, damage), (false, false))
    };
    assert!(k > 0);
    let scores = scores_of(&state, &r, me);
    let at = state.stacks()[0].hex;
    for hex in all_hexes() {
        let behind = field.distance(hex, at) == 2
            && field
                .neighbors(at)
                .into_iter()
                .flatten()
                .any(|n| field.direction(at, n) == field.direction(n, hex));
        let want = if behind {
            (-k).max(floor_of(&state, &r, me))
        } else {
            0
        };
        assert_eq!(scores[hex.index()], want, "{hex:?}");
    }
    // An enemy dragon within a turn of the imps' friend breathes on them: the area threat around
    // the friend.
    let mut state = BattleState::new(
        &r,
        &on(2, setup(&[(GOG, 1), (IMP, 10)], &[(DRAGON, 1)])),
        18,
    )
    .unwrap();
    assert_eq!(active(&state), DRAGON);
    play(&mut state, &r, Command::Move { to: Hex(6) });
    assert_eq!(active(&state), IMP);
    let scores = scores_of(&state, &r, StackId(1));
    let gog = state.stacks()[0].hex;
    assert!(all_hexes().any(|h| field.distance(h, gog) == 1 && scores[h.index()] < 0));
}

// covers: BAT-AI-007
#[test]
fn a_fire_wall_is_valued_with_the_slips_or_as_the_rule() {
    // Both heroes give their stacks 4 attack and no defense: the context's least attack bonus is
    // 4, its least defense bonus 0. The attacker's hero casts Fire Wall (30 damage) at 93 and its
    // upper right neighbour, on Easy (no reach, no crowding: the hazards alone). The defender's
    // imps' scores there lose the value of its damage, valued with the least attack bonus for
    // both bonuses with the original's slips (slip 19), else with both least bonuses; the
    // efreet, immune to fire, lose it only with the slips (slip 18).
    for (profile, slips) in [("t:faithful", true), ("t:fixed", false)] {
        let r = ruleset_as(profile);
        let firewall = r.spell_by_key("t:firewall").unwrap();
        let mut mage = BattleHero::new(HeroTypeId::from_index(0).unwrap());
        mage.attack = 4;
        mage.power = 2;
        mage.magic.spell_book = Some(vec![firewall]);
        mage.magic.mana = 50;
        let mut knight = BattleHero::new(HeroTypeId::from_index(0).unwrap());
        knight.attack = 4;
        let mut setup = on(0, setup(&[(GOG, 5)], &[(IMP, 10), (EFREET, 5)]));
        setup.heroes = [Some(mage), Some(knight)];
        let mut state = BattleState::new(&r, &setup, 19).unwrap();
        play(&mut state, &r, Command::Defend);
        play(&mut state, &r, Command::Defend);
        assert_eq!(active(&state), GOG);
        let cast = Command::Cast {
            spell: firewall,
            target: h3_battle::CastTarget::Hex(Hex(93)),
            second: h3_battle::CastTarget::None,
        };
        play(&mut state, &r, cast);
        let walls: Vec<Hex> = state
            .spell_obstacles()
            .iter()
            .flat_map(|o| o.hexes.iter())
            .collect();
        assert_eq!(walls.len(), 2);
        let ctx = computer::context(&state, &r, BattleSide::Defender);
        for (id, immune) in [(StackId(1), false), (StackId(2), true)] {
            let scores = scores_of(&state, &r, id);
            for hex in all_hexes() {
                let want =
                    computer::fire_wall_damage(&state, &r, hex, id).map_or(0, |(d, hurts)| {
                        assert_eq!(hurts, !immune);
                        let value = |ctx: &computer::Context| {
                            -computer::damage_value(&state, &r, ctx, (id, d), (false, false))
                        };
                        let (orig, rule) = (value(&ctx.attack_for_both()), value(&ctx));
                        assert!(orig < 0 && rule < 0 && orig != rule, "{orig} {rule}");
                        match (slips, immune) {
                            (true, _) => orig,
                            (false, false) => rule,
                            (false, true) => 0,
                        }
                    });
                assert_eq!(scores[hex.index()], want, "{profile} {id:?} {hex:?}");
            }
        }
    }
}
