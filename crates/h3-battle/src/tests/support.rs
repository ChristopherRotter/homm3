//! Test fixtures: a small ruleset built in code (the battle crate cannot depend on the pack
//! adapter) and helpers to set up and drive battles.

use std::string::String;
use std::vec;
use std::vec::Vec;

use std::sync::Arc;

use h3_content::{
    ContentKey, ContentPack, CreatureDef, CreatureFlag, CreatureFlags, CreatureInfoDef, FactionDef,
    Formation, HeroClassDef, HeroTypeDef, PackManifest, ProfileDef, ResourceDef, Ruleset,
    SpellCategory, SpellSchoolDef, StartingStackDef, TerrainDef,
};
use h3_content_battle::{
    AbilityDef, AuraTarget, BallisticsRecord, BattleCompileError, BattleCreatureDef, BattlePack,
    BattleParamsDef, BattleRuleset, BattleSpellDef, BattlefieldDef, EffectDuration, EffectEnd,
    FactionChangeDef, HexStep, ObstacleDef, ObstacleKind, ObstacleShape, SiegeDef, SiegeFieldDef,
    SpecialistBonus, SpellAi, SpellArea, SpellChain, SpellEffect, SpellMagic, SpellPolarity,
    SpellTarget, SurrenderDef, TargetConditions, TargetSide, TimedChange, Usefulness, WallKind,
    WallTarget,
};
pub use h3_contracts::Rate;
use h3_contracts::{Army, ArmyStack, BattleHero, BattleSetup, BattleSide};

use crate::{BattleState, Command, Event, Field, Hex, StackId, apply};

/// A test spell as one row: the core's data row and the battle's, as the packs' text has them
/// (R01); [`SpellDef::split`] parts them.
#[derive(Debug, Clone)]
pub struct SpellDef {
    pub key: ContentKey,
    pub name: String,
    pub category: SpellCategory,
    pub level: u8,
    pub schools: Vec<ContentKey>,
    pub cost: [u16; 4],
    pub power: u16,
    pub effect: [u16; 4],
    pub granted_by_name_only: bool,
    pub polarity: SpellPolarity,
    pub magic: SpellMagic,
    pub counters: Vec<ContentKey>,
    pub dispels: Vec<ContentKey>,
    pub spares: Vec<ContentKey>,
    pub tests_as: Option<ContentKey>,
    pub summons: Option<ContentKey>,
}

impl SpellDef {
    /// The core's row and the battle's.
    pub fn split(self) -> (h3_content::SpellDef, BattleSpellDef) {
        (
            h3_content::SpellDef {
                key: self.key.clone(),
                name: self.name,
                category: self.category,
                level: self.level,
                schools: self.schools,
                cost: self.cost,
                power: self.power,
                effect: self.effect,
                granted_by_name_only: self.granted_by_name_only,
            },
            BattleSpellDef {
                key: self.key,
                polarity: self.polarity,
                magic: self.magic,
                counters: self.counters,
                dispels: self.dispels,
                spares: self.spares,
                tests_as: self.tests_as,
                summons: self.summons,
                ai: SpellAi::default(),
            },
        )
    }
}

/// A test creature's traits: its core attributes and its battle row's abilities, shots and casts.
struct Traits {
    info: CreatureInfoDef,
    abilities: Vec<AbilityDef>,
    shots: u16,
    casts: u8,
}

/// `(name, attack, defense, damage_min, damage_max, hit_points, speed, traits)`; traits are
/// creature flag names, ability names (`teleports`, `jousting=5`, `hate=fixed:50`, …),
/// `fireball` / `death_cloud` for an area shot, and `shots=N`.
pub type Creature = (
    &'static str,
    u16,
    u16,
    u16,
    u16,
    u32,
    u8,
    &'static [&'static str],
);

pub const CREATURES: &[Creature] = &[
    ("fixed", 10, 10, 2, 2, 10, 5, &[]),
    ("slow", 1, 1, 1, 1, 5, 2, &[]),
    ("fast", 5, 5, 1, 1, 3, 9, &[]),
    ("tank", 0, 20, 1, 1, 100, 4, &[]),
    ("range", 5, 5, 1, 9, 10, 5, &[]),
    ("spear", 4, 5, 1, 3, 10, 4, &[]),
    ("brute", 7, 4, 2, 5, 18, 5, &[]),
    ("wisp", 3, 2, 1, 2, 4, 9, &[]),
    ("wide", 6, 6, 2, 4, 25, 5, &["two_hex"]),
    ("flyer", 5, 3, 1, 3, 8, 7, &["flying"]),
    ("dragon", 12, 12, 5, 8, 60, 8, &["two_hex", "flying"]),
    ("porter", 8, 6, 2, 4, 15, 6, &["teleports"]),
    ("archer", 6, 3, 2, 3, 10, 4, &["shooter", "shots=12"]),
    (
        "sniper",
        6,
        3,
        2,
        3,
        10,
        4,
        &[
            "shooter",
            "shots=3",
            "strikes_twice",
            "no_melee_penalty",
            "no_distance_penalty",
        ],
    ),
    (
        "magog",
        7,
        4,
        2,
        4,
        13,
        6,
        &["shooter", "shots=24", "area_attack", "fireball"],
    ),
    (
        "lich",
        13,
        10,
        11,
        13,
        30,
        6,
        &[
            "shooter",
            "shots=12",
            "area_attack",
            "death_cloud",
            "undead",
        ],
    ),
    ("salamander", 5, 5, 2, 3, 12, 5, &["fire_immune", "alive"]),
    // Melee abilities (BAT-ATK-002..004, BAT-RET-002/003, BAT-DMG-004..009, BAT-TRN-005): all
    // 10/10 with damage 2 and 10 HP, so against `fixed` or `serf` a strike deals 2 per creature.
    ("serf", 10, 10, 2, 2, 10, 5, &["alive"]),
    ("twin", 10, 10, 2, 2, 10, 5, &["strikes_twice"]),
    ("naga", 10, 10, 2, 2, 10, 5, &["no_enemy_retaliation"]),
    (
        "griffin",
        10,
        10,
        2,
        2,
        10,
        5,
        &["flying", "extra_retaliations=1"],
    ),
    (
        "royal",
        10,
        10,
        2,
        2,
        10,
        5,
        &["flying", "unlimited_retaliations"],
    ),
    (
        "hydra",
        10,
        10,
        2,
        2,
        10,
        5,
        &[
            "two_hex",
            "multi_headed",
            "attacks_all_adjacent",
            "no_enemy_retaliation",
        ],
    ),
    (
        "cerberus",
        10,
        10,
        2,
        2,
        10,
        5,
        &[
            "two_hex",
            "multi_headed",
            "three_headed_attack",
            "no_enemy_retaliation",
        ],
    ),
    ("drake", 10, 10, 2, 2, 10, 5, &["two_hex", "breath_attack"]),
    (
        "harpy",
        10,
        10,
        2,
        2,
        10,
        6,
        &["flying", "strike_and_return"],
    ),
    ("knight", 10, 10, 2, 2, 10, 7, &["jousting=5"]),
    ("pikeman", 10, 10, 2, 2, 10, 4, &["charge_immunity"]),
    ("behemoth", 10, 10, 2, 2, 10, 5, &["defense_reduction=40"]),
    ("dread", 10, 10, 2, 2, 10, 5, &["death_blow=20"]),
    ("doom", 10, 10, 2, 2, 10, 5, &["death_blow=100"]),
    (
        "vampire",
        10,
        10,
        2,
        2,
        10,
        5,
        &["life_drain=100", "undead"],
    ),
    ("gorgon", 10, 10, 2, 2, 10, 5, &["death_stare=10"]),
    ("starer", 10, 10, 2, 2, 10, 5, &["death_stare=100"]),
    ("wight", 10, 10, 2, 2, 10, 5, &["regeneration", "undead"]),
    ("hater", 10, 10, 2, 2, 10, 5, &["hate=fixed:50"]),
    // Morale and luck (BAT-MOR-001..004, BAT-LCK-001/002).
    (
        "pikeman_c",
        10,
        10,
        2,
        2,
        10,
        5,
        &["faction=castle", "alive"],
    ),
    (
        "angel",
        10,
        10,
        2,
        2,
        10,
        5,
        &["faction=castle", "alive", "morale_aura=1:allies"],
    ),
    (
        "archangel",
        10,
        10,
        2,
        2,
        10,
        5,
        &["faction=castle", "alive", "morale_aura=1:allies"],
    ),
    ("imp", 10, 10, 2, 2, 10, 5, &["faction=inferno", "alive"]),
    (
        "devil",
        10,
        10,
        2,
        2,
        10,
        5,
        &["faction=inferno", "alive", "luck_aura=-1:enemies"],
    ),
    (
        "skeleton",
        10,
        10,
        2,
        2,
        10,
        5,
        &["faction=necropolis", "undead", "no_morale"],
    ),
    (
        "bone_dragon",
        10,
        10,
        2,
        2,
        10,
        5,
        &[
            "faction=necropolis",
            "undead",
            "no_morale",
            "morale_aura=-1:enemies",
        ],
    ),
    (
        "minotaur",
        10,
        10,
        2,
        2,
        10,
        5,
        &["alive", "minimum_morale=1"],
    ),
    (
        "halfling",
        10,
        10,
        2,
        2,
        10,
        5,
        &["alive", "minimum_luck=1"],
    ),
    ("lucky", 10, 10, 2, 2, 10, 5, &["alive", "minimum_luck=3"]),
    // War machines (BAT-WAR-001..004).
    (
        "ballista",
        10,
        10,
        2,
        3,
        250,
        0,
        &[
            "two_hex",
            "shooter",
            "war_machine",
            "no_morale",
            "shots=24",
            "ai=600",
        ],
    ),
    (
        "tent",
        0,
        0,
        0,
        0,
        75,
        0,
        // The minimum morale has no effect (no morale); it puts another ability before first aid.
        &[
            "two_hex",
            "war_machine",
            "no_morale",
            "minimum_morale=1",
            "first_aid=1:25",
        ],
    ),
    (
        "cart",
        0,
        5,
        0,
        0,
        100,
        0,
        &["war_machine", "no_morale", "ammo_supply"],
    ),
    ("valuable", 10, 10, 2, 2, 10, 5, &["alive", "ai=1000"]),
    ("gadget", 0, 0, 0, 0, 10, 0, &["war_machine"]),
    // A shooting war machine with an area shot: it can hit its own side (BAT-END-002).
    (
        "firebomb",
        10,
        10,
        50,
        50,
        100,
        0,
        &[
            "shooter",
            "war_machine",
            "no_morale",
            "area_attack",
            "fireball",
            "shots=4",
        ],
    ),
    // A creature and its upgrade (hero statistic effects, BAT-HER-005).
    (
        "squire",
        10,
        12,
        6,
        9,
        35,
        5,
        &["alive", "upgrades=paladin"],
    ),
    ("paladin", 12, 12, 7, 10, 35, 6, &["alive"]),
    // A level-7 creature (spell specialties divide by the creature's level, BAT-MAG-004).
    ("titan", 20, 20, 10, 20, 300, 7, &["alive", "level=7"]),
    // Battle magic (BAT-MAG-004 to BAT-MAG-008): all 10/10 with damage 2 and 10 HP.
    (
        "dwarf",
        10,
        10,
        2,
        2,
        10,
        5,
        &["alive", "magic_resistance=20"],
    ),
    (
        "unicorn",
        10,
        10,
        2,
        2,
        10,
        5,
        &["alive", "magic_resistance_aura=20"],
    ),
    (
        "wyrm",
        10,
        10,
        2,
        2,
        10,
        5,
        &["alive", "spell_level_immunity=1"],
    ),
    ("golem", 10, 10, 2, 2, 10, 5, &["spell_damage_reduction=75"]),
    (
        "ember",
        10,
        10,
        2,
        2,
        10,
        5,
        &[
            "alive",
            "school_immunity=fire",
            "spell_immunity=rally",
            "spell_vulnerability=bolt:100",
        ],
    ),
    // Timed spells (BAT-MAG-009 to 011): immune to mind spells.
    ("statue", 10, 10, 2, 2, 10, 5, &["mind_immune"]),
    // Immune to Blind absolutely and to the bolt as a normal immunity (BAT-MAG-066: a Troglodyte's
    // Blind, an elemental's bolt).
    (
        "troglo",
        10,
        10,
        2,
        2,
        10,
        5,
        &["alive", "absolute_immunity=blind", "spell_immunity=bolt"],
    ),
    // Slayer (BAT-MAG-014): the three king levels.
    ("lord", 10, 10, 2, 2, 50, 4, &["king_1"]),
    ("duke", 10, 10, 2, 2, 50, 4, &["king_2"]),
    ("prince", 10, 10, 2, 2, 50, 4, &["king_3"]),
    // Spell specialists (BAT-MAG-020): creature levels 3 and 5.
    ("tier3", 10, 10, 2, 2, 10, 5, &["level=3"]),
    ("tier5", 10, 10, 2, 2, 10, 5, &["level=5"]),
    // Raising (BAT-MAG-035/036): a living two-hex creature, a living one immune to spells up to
    // level 4 (the Gold Dragons), an undead one, a living one with much health.
    ("ogre", 8, 8, 3, 5, 40, 5, &["two_hex", "alive"]),
    (
        "gilded",
        10,
        10,
        2,
        2,
        50,
        5,
        &["alive", "spell_level_immunity=4"],
    ),
    ("ghoul", 10, 10, 2, 2, 10, 5, &["undead"]),
    ("seraph", 20, 20, 2, 2, 200, 5, &["alive"]),
    // Created stacks (BAT-MAG-037 to BAT-MAG-039): a summoned creature that is not alive, of
    // level 2.
    ("sprite", 6, 6, 2, 3, 15, 7, &["level=2"]),
    // Spells after a strike (BAT-MAG-040 to BAT-MAG-042): `spell:chance:mastery:rounds` plus
    // `:living` and `:unless`; a damage spell `spell:chance:per creature`.
    (
        "blinder",
        10,
        10,
        2,
        2,
        10,
        5,
        &["alive", "spell_after_strike=blind:100:0:3"],
    ),
    (
        "hexer",
        10,
        10,
        2,
        2,
        10,
        5,
        &["alive", "spell_after_strike=curse:25:0:3:living"],
    ),
    (
        "weakener",
        10,
        10,
        2,
        2,
        10,
        5,
        &["alive", "spell_after_strike=weaken:100:0:3"],
    ),
    (
        "fly",
        10,
        10,
        2,
        2,
        10,
        5,
        &[
            "alive",
            "spell_after_strike=unbless:100:0:3",
            "spell_after_strike=weaken:100:2:3:unless",
        ],
    ),
    (
        "stormbird",
        10,
        10,
        2,
        2,
        10,
        5,
        &["alive", "spell_damage_after_strike=bolt:100:10"],
    ),
    (
        "gazer",
        10,
        10,
        2,
        2,
        10,
        5,
        &["alive", "spell_after_strike=petrify:100:0:3"],
    ),
    (
        "stinger",
        10,
        10,
        2,
        2,
        10,
        5,
        &["alive", "spell_after_strike=paralysis:100:0:3"],
    ),
    (
        "rotter",
        10,
        10,
        2,
        2,
        10,
        5,
        &["alive", "spell_after_strike=plague:100:0:3:living"],
    ),
    (
        "ancient",
        10,
        10,
        2,
        2,
        10,
        5,
        &["alive", "spell_after_strike=wither:100:0:3:living"],
    ),
    (
        "wyvern",
        10,
        10,
        2,
        2,
        10,
        5,
        &["alive", "spell_after_strike=venom:100:0:3:living"],
    ),
    (
        "twinblinder",
        10,
        10,
        2,
        2,
        10,
        5,
        &["alive", "strikes_twice", "spell_after_strike=blind:100:0:3"],
    ),
    // Bind, Acid Breath and the extra damage (BAT-MAG-045 to BAT-MAG-047): `:defended`; extra
    // damage `chance:percent`.
    (
        "binder",
        10,
        10,
        2,
        2,
        10,
        3,
        &["alive", "spell_after_strike=tether:100:0:3"],
    ),
    (
        "corroder",
        10,
        10,
        2,
        4,
        10,
        5,
        &[
            "alive",
            "spell_after_strike=acid:100:0:3:defended",
            "damage_after_strike=100:50",
        ],
    ),
    (
        "nibbler",
        10,
        10,
        1,
        1,
        10,
        5,
        &["alive", "damage_after_strike=20:50"],
    ),
    // Attack and defense below Disease's 2 (BAT-MAG-009, T7a review).
    ("peasant", 1, 1, 1, 1, 10, 3, &["alive"]),
    // Casters (BAT-MAG-048): Haste at Advanced with power 6, twice a battle (an Ogre Mage's
    // Bloodlust); Weaken at Basic with power 4, once (a negative spell at enemies).
    (
        "conjurer",
        10,
        10,
        2,
        2,
        10,
        5,
        &["alive", "casts=2", "spell_caster=haste:2:6"],
    ),
    (
        "warlock",
        10,
        10,
        2,
        2,
        10,
        5,
        &["alive", "casts=1", "spell_caster=weaken:0:4"],
    ),
    // Random casters (BAT-MAG-049), a Faerie Dragon's power per creature at Advanced: a two-hex
    // one that draws among four spells, and one-hex ones that always draw the same spell: `bolt`
    // (negative, at a stack), `chain` (neutral, at a stack), `nova` (neutral, an area at a hex),
    // `haste` (positive, at an own stack).
    (
        "dragonet",
        10,
        10,
        2,
        2,
        100,
        5,
        &[
            "two_hex",
            "alive",
            "casts=3",
            "random_spell_caster=2:5",
            "drawn_spell=bolt:22",
            "drawn_spell=nova:21",
            "drawn_spell=ring:10",
            "drawn_spell=chain:5",
        ],
    ),
    (
        "boltling",
        10,
        10,
        2,
        2,
        100,
        5,
        &[
            "alive",
            "casts=2",
            "random_spell_caster=2:5",
            "drawn_spell=bolt:1",
        ],
    ),
    (
        "chainling",
        10,
        10,
        2,
        2,
        100,
        5,
        &[
            "alive",
            "casts=2",
            "random_spell_caster=2:5",
            "drawn_spell=chain:1",
        ],
    ),
    (
        "novaling",
        10,
        10,
        2,
        2,
        100,
        5,
        &[
            "alive",
            "casts=2",
            "random_spell_caster=2:5",
            "drawn_spell=nova:1",
        ],
    ),
    // A random caster that ignores a hero's NoCasting (the Faerie Dragon after the wiki and VCMI,
    // BAT-MAG-061, BAT-Q21).
    (
        "faerie",
        10,
        10,
        2,
        2,
        100,
        5,
        &[
            "alive",
            "casts=2",
            "random_spell_caster=2:5",
            "drawn_spell=bolt:1",
            "ignores_magic_block",
        ],
    ),
    (
        "hasteling",
        10,
        10,
        2,
        2,
        100,
        5,
        &[
            "alive",
            "casts=2",
            "random_spell_caster=2:5",
            "drawn_spell=haste:1",
        ],
    ),
    // A helpful caster (BAT-MAG-050), a Master Genie at Advanced with power 6, with a spell of
    // each usefulness.
    (
        "jinn",
        10,
        10,
        2,
        2,
        40,
        5,
        &[
            "alive",
            "flying",
            "casts=3",
            "helpful_spell_caster=2:6",
            "drawn_spell=shield:1",
            "drawn_spell=airshield:1",
            "drawn_spell=airward:1",
            "drawn_spell=cure:1",
            "drawn_spell=bless:1",
            "drawn_spell=lust:1",
            "drawn_spell=aim:1",
            "drawn_spell=prayer:1",
            "drawn_spell=slayer:1",
        ],
    ),
    // Survey S-18: a Diamond Golem keeps 5 % of a spell's damage, a Gold Golem 15 %.
    (
        "diamond",
        10,
        10,
        2,
        2,
        60,
        5,
        &["spell_damage_reduction=95"],
    ),
    (
        "goldgolem",
        10,
        10,
        2,
        2,
        50,
        5,
        &["spell_damage_reduction=85"],
    ),
    // The raisers (BAT-MAG-051, BAT-MAG-052): an Archangel's 100 hit points per creature under
    // `rise`'s test, a Pit Lord's 50 per creature raising imps (health 10) from corpses.
    (
        "herald",
        10,
        10,
        2,
        2,
        50,
        9,
        &[
            "two_hex",
            "alive",
            "casts=1",
            "morale_aura=1:allies",
            "stack_raiser=rise:100",
        ],
    ),
    (
        "fiend",
        10,
        10,
        2,
        2,
        45,
        5,
        &["alive", "casts=1", "corpse_raiser=imp:50"],
    ),
    // A periodic caster (BAT-MAG-053), an Enchanter: on its own once in three round starts of
    // its side, at Expert with power 3, the original's table on the test spells (Haste, Air
    // Shield, Slow, Stone Skin, Bloodlust, Bless, Cure, Weakness).
    (
        "chanter",
        10,
        10,
        2,
        2,
        30,
        9,
        &[
            "alive",
            "shooter",
            "shots=32",
            "periodic_spell_caster=3:3:3",
            "drawn_spell=haste:15",
            "drawn_spell=airshield:10",
            "drawn_spell=sloth:10",
            "drawn_spell=skin:15",
            "drawn_spell=lust:5",
            "drawn_spell=bless:15",
            "drawn_spell=cure:10",
            "drawn_spell=weaken:4",
        ],
    ),
    // A periodic caster of Bloodlust and Cure only, which count on a melee and on a wounded stack
    // (its ability listed after its spells).
    (
        "luster",
        10,
        10,
        2,
        2,
        30,
        9,
        &[
            "alive",
            "drawn_spell=lust:5",
            "drawn_spell=cure:10",
            "periodic_spell_caster=3:3:3",
        ],
    ),
    // The mana creatures (BAT-MAG-054 to BAT-MAG-056): a Pegasus (+2 for the enemy hero's
    // spells), Mages (−2 for their own hero's) and a stronger reduction, Familiars (20 % of the
    // other hero's cost) and a stronger channel, a Wraith (2 mana at its activations; a castle
    // creature for good morale; its ability after another, as the base pack's).
    (
        "pegasus",
        10,
        10,
        2,
        2,
        30,
        8,
        &["alive", "flying", "spell_cost=2:enemies"],
    ),
    (
        "magus",
        10,
        10,
        2,
        2,
        30,
        5,
        &["alive", "spell_cost=-2:allies"],
    ),
    (
        "sage",
        10,
        10,
        2,
        2,
        30,
        5,
        &["alive", "spell_cost=-5:allies"],
    ),
    (
        "familiar",
        10,
        10,
        2,
        2,
        10,
        7,
        &["alive", "mana_channel=20"],
    ),
    (
        "acolyte",
        10,
        10,
        2,
        2,
        10,
        4,
        &["alive", "mana_channel=50"],
    ),
    (
        "wraith",
        10,
        10,
        2,
        2,
        30,
        7,
        &["faction=castle", "alive", "regeneration", "mana_drain=2"],
    ),
    // The creature defences (BAT-MAG-057, BAT-MAG-058, BAT-DMG-010): a Tank with its own Fire
    // Shield of 25 % (between the spell's 20 and 30), Magic Mirrors of 20 % and 100 %, and the
    // elementals' halves against mind-immune creatures and against itself and the Tank.
    (
        "sultan",
        0,
        20,
        1,
        1,
        100,
        4,
        &["alive", "innate_spell=fireshield:25"],
    ),
    (
        "fairy",
        10,
        10,
        2,
        2,
        10,
        5,
        &["alive", "innate_spell=mirror:20"],
    ),
    (
        "pixie",
        0,
        20,
        1,
        1,
        100,
        4,
        &["alive", "innate_spell=mirror:100"],
    ),
    (
        "psychic",
        10,
        10,
        2,
        2,
        10,
        5,
        &["mind_immune", "damage_penalty_by_flag=mind_immune:50"],
    ),
    (
        "magical",
        10,
        10,
        2,
        2,
        10,
        5,
        &["damage_penalty=magical:50", "damage_penalty=tank:50"],
    ),
    (
        "psionic",
        10,
        10,
        2,
        2,
        10,
        5,
        &[
            "damage_penalty=statue:30",
            "damage_penalty_by_flag=mind_immune:50",
        ],
    ),
    // The Phoenix's rebirth (BAT-MAG-059): a fifth with its one cast; a firebird with two casts
    // comes back with a third, twice.
    (
        "phoenix",
        10,
        10,
        2,
        2,
        10,
        6,
        &["alive", "flying", "charge_immunity", "rebirth=5", "casts=1"],
    ),
    (
        "firebird",
        10,
        10,
        2,
        2,
        10,
        6,
        &["alive", "rebirth=3", "casts=2"],
    ),
    // Fear (BAT-MAG-060): an azure that nearly always frightens (the compiler refuses a certain
    // fear: tests pick seeds whose roll is not the one in a hundred), a spook that rarely does.
    (
        "azure",
        10,
        10,
        2,
        2,
        10,
        6,
        &["alive", "charge_immunity", "fear=99"],
    ),
    ("spook", 10, 10, 2, 2, 10, 4, &["alive", "fear=1"]),
    // A thunderbolt by chance (BAT-MAG-041).
    (
        "squall",
        10,
        10,
        2,
        2,
        10,
        5,
        &["alive", "spell_damage_after_strike=bolt:50:10"],
    ),
    // A two-hex shooter (Medusas; BAT-RNG-002), the archer's numbers.
    (
        "medusa",
        6,
        3,
        2,
        3,
        10,
        4,
        &["two_hex", "shooter", "shots=12"],
    ),
    // A shooter the walls do not halve (BAT-SIE-008, the Mage's kind).
    (
        "mage_s",
        6,
        3,
        2,
        3,
        10,
        4,
        &["shooter", "shots=12", "no_wall_penalty"],
    ),
    // The siege's Catapult (BAT-SIE-009): the original's (`CRTRAITS.TXT`).
    (
        "catapult",
        10,
        10,
        0,
        0,
        1000,
        0,
        &[
            "two_hex",
            "shooter",
            "catapult",
            "war_machine",
            "no_morale",
            "shots=24",
        ],
    ),
    // Wall shooters (BAT-SIE-013): the Cyclops and the Cyclops King.
    (
        "cyclops",
        15,
        12,
        16,
        20,
        70,
        6,
        &["alive", "shooter", "catapult", "shots=16", "wall_shots=1"],
    ),
    (
        "cyclops_king",
        17,
        13,
        16,
        20,
        70,
        8,
        &["alive", "shooter", "catapult", "shots=24", "wall_shots=2"],
    ),
];

/// A town's turret (BAT-SIE-016): the original's Arrow Tower (`CRTRAITS.TXT`). Only
/// [`turret_ruleset`] has it, so the other sieges of the tests have no turrets.
const TURRET: Creature = (
    "arrow_tower",
    10,
    5,
    2,
    4,
    100,
    0,
    &[
        "shooter",
        "war_machine",
        "mind_immune",
        "no_morale",
        "shots=99",
        "turret",
    ],
);

/// The traits of a test creature as creature info: flags, abilities, shots and faction.
fn parse_traits(traits: &[&str]) -> Traits {
    let mut flags = Vec::new();
    let mut abilities = Vec::new();
    let mut shots = 0;
    let mut faction = None;
    let mut ai = 0;
    let mut upgrades = Vec::new();
    let mut level = 1;
    let mut casts = 0;
    for t in traits {
        let (name, value) = t.split_once('=').unwrap_or((t, "0"));
        let n = || value.parse::<u16>().unwrap();
        let signed = || value.parse::<i8>().unwrap();
        let aura = || {
            let (amount, affects) = value.split_once(':').unwrap();
            let affects = match affects {
                "allies" => AuraTarget::Allies,
                _ => AuraTarget::Enemies,
            };
            (amount.parse::<i8>().unwrap(), affects)
        };
        let ability = match name {
            "ai" => {
                ai = value.parse().unwrap();
                continue;
            }
            "first_aid" => {
                let (min, max) = value.split_once(':').unwrap();
                AbilityDef::FirstAid {
                    min_hp: min.parse().unwrap(),
                    max_hp: max.parse().unwrap(),
                }
            }
            "ammo_supply" => AbilityDef::AmmoSupply,
            "turret" => AbilityDef::Turret,
            "ignores_magic_block" => AbilityDef::IgnoresMagicBlock,
            "faction" => {
                faction = Some(ContentKey::new("t", value).unwrap());
                continue;
            }
            "upgrades" => {
                upgrades.push(ContentKey::new("t", value).unwrap());
                continue;
            }
            "level" => {
                level = value.parse().unwrap();
                continue;
            }
            "morale_aura" => {
                let (amount, affects) = aura();
                AbilityDef::MoraleAura { amount, affects }
            }
            "luck_aura" => {
                let (amount, affects) = aura();
                AbilityDef::LuckAura { amount, affects }
            }
            "minimum_morale" => AbilityDef::MinimumMorale { value: signed() },
            "minimum_luck" => AbilityDef::MinimumLuck { value: signed() },
            "teleports" => AbilityDef::Teleports,
            "no_distance_penalty" => AbilityDef::NoDistancePenalty,
            "no_wall_penalty" => AbilityDef::NoWallPenalty,
            "wall_shots" => AbilityDef::WallShots {
                level: u8::try_from(n()).unwrap(),
            },
            "fireball" => AbilityDef::AreaShot {
                effect: h3_content_battle::AreaShot::Fireball,
            },
            "death_cloud" => AbilityDef::AreaShot {
                effect: h3_content_battle::AreaShot::DeathCloud,
            },
            "hate" => {
                let (target, percent) = value.split_once(':').unwrap();
                AbilityDef::Hate {
                    target: ContentKey::new("t", target).unwrap(),
                    percent: percent.parse().unwrap(),
                }
            }
            "jousting" => AbilityDef::Jousting {
                percent_per_hex: n(),
            },
            "charge_immunity" => AbilityDef::ChargeImmunity,
            "defense_reduction" => AbilityDef::DefenseReduction {
                percent: u8::try_from(n()).unwrap(),
            },
            "death_blow" => AbilityDef::DeathBlow {
                chance_percent: u8::try_from(n()).unwrap(),
            },
            "life_drain" => AbilityDef::LifeDrain { percent: n() },
            "regeneration" => AbilityDef::Regeneration,
            "death_stare" => AbilityDef::DeathStare {
                chance_percent: u8::try_from(n()).unwrap(),
            },
            "extra_retaliations" => AbilityDef::ExtraRetaliations {
                count: u8::try_from(n()).unwrap(),
            },
            "unlimited_retaliations" => AbilityDef::UnlimitedRetaliations,
            "strike_and_return" => AbilityDef::StrikeAndReturn,
            "attacks_all_adjacent" => AbilityDef::AttacksAllAdjacent,
            "three_headed_attack" => AbilityDef::ThreeHeadedAttack,
            "magic_resistance" => AbilityDef::MagicResistance {
                percent: u8::try_from(n()).unwrap(),
            },
            "magic_resistance_aura" => AbilityDef::MagicResistanceAura {
                percent: u8::try_from(n()).unwrap(),
            },
            "spell_level_immunity" => AbilityDef::SpellLevelImmunity {
                max_level: u8::try_from(n()).unwrap(),
            },
            "spell_damage_reduction" => AbilityDef::SpellDamageReduction {
                percent: u8::try_from(n()).unwrap(),
            },
            "school_immunity" => AbilityDef::SchoolImmunity {
                school: ContentKey::new("t", value).unwrap(),
            },
            "spell_immunity" => AbilityDef::SpellImmunity {
                spell: ContentKey::new("t", value).unwrap(),
                absolute: false,
            },
            // An immunity no negation lifts (BAT-MAG-066).
            "absolute_immunity" => AbilityDef::SpellImmunity {
                spell: ContentKey::new("t", value).unwrap(),
                absolute: true,
            },
            "spell_vulnerability" => {
                let (spell, percent) = value.split_once(':').unwrap();
                AbilityDef::SpellVulnerability {
                    spell: ContentKey::new("t", spell).unwrap(),
                    percent: percent.parse().unwrap(),
                }
            }
            "spell_after_strike" => {
                let mut parts = value.split(':');
                let mut next = || parts.next().unwrap_or("");
                let spell = ContentKey::new("t", next()).unwrap();
                let (chance, mastery, rounds) = (next(), next(), next());
                let rest: Vec<&str> = core::iter::from_fn(|| Some(next()))
                    .take_while(|p| !p.is_empty())
                    .collect();
                AbilityDef::SpellAfterStrike {
                    spell,
                    chance_percent: chance.parse().unwrap(),
                    mastery: mastery.parse().unwrap(),
                    rounds: rounds.parse().unwrap(),
                    living: rest.contains(&"living"),
                    unless_active: rest.contains(&"unless"),
                    defended: rest.contains(&"defended"),
                }
            }
            "damage_after_strike" => {
                let (chance, percent) = value.split_once(':').unwrap();
                AbilityDef::DamageAfterStrike {
                    chance_percent: chance.parse().unwrap(),
                    percent: percent.parse().unwrap(),
                }
            }
            "spell_damage_after_strike" => {
                let mut parts = value.split(':');
                let mut next = || parts.next().unwrap();
                AbilityDef::SpellDamageAfterStrike {
                    spell: ContentKey::new("t", next()).unwrap(),
                    chance_percent: next().parse().unwrap(),
                    per_creature: next().parse().unwrap(),
                }
            }
            "shots" => {
                shots = n();
                continue;
            }
            "casts" => {
                casts = value.parse().unwrap();
                continue;
            }
            "spell_caster" => {
                let mut parts = value.split(':');
                let mut next = || parts.next().unwrap();
                AbilityDef::SpellCaster {
                    spell: ContentKey::new("t", next()).unwrap(),
                    mastery: next().parse().unwrap(),
                    power: next().parse().unwrap(),
                }
            }
            "random_spell_caster" => {
                let (mastery, per_creature) = value.split_once(':').unwrap();
                AbilityDef::RandomSpellCaster {
                    mastery: mastery.parse().unwrap(),
                    power_per_creature: per_creature.parse().unwrap(),
                }
            }
            "helpful_spell_caster" => {
                let (mastery, power) = value.split_once(':').unwrap();
                AbilityDef::HelpfulSpellCaster {
                    mastery: mastery.parse().unwrap(),
                    power: power.parse().unwrap(),
                }
            }
            "stack_raiser" => {
                let (spell, hit_points) = value.split_once(':').unwrap();
                AbilityDef::StackRaiser {
                    spell: ContentKey::new("t", spell).unwrap(),
                    hit_points_per_creature: hit_points.parse().unwrap(),
                }
            }
            "periodic_spell_caster" => {
                let mut parts = value.split(':').map(|p| p.parse().unwrap());
                let mut next = || parts.next().unwrap();
                AbilityDef::PeriodicSpellCaster {
                    rounds: next(),
                    mastery: next(),
                    power: next(),
                }
            }
            "corpse_raiser" => {
                let (creature, hit_points) = value.split_once(':').unwrap();
                AbilityDef::CorpseRaiser {
                    creature: ContentKey::new("t", creature).unwrap(),
                    hit_points_per_creature: hit_points.parse().unwrap(),
                }
            }
            "spell_cost" => {
                let (amount, affects) = aura();
                AbilityDef::SpellCost { amount, affects }
            }
            "mana_channel" => AbilityDef::ManaChannel {
                percent: value.parse().unwrap(),
            },
            "fear" => AbilityDef::Fear {
                percent: value.parse().unwrap(),
            },
            "rebirth" => AbilityDef::Rebirth {
                divisor: value.parse().unwrap(),
            },
            "mana_drain" => AbilityDef::ManaDrain {
                amount: value.parse().unwrap(),
            },
            "innate_spell" => {
                let (spell, value) = value.split_once(':').unwrap();
                AbilityDef::InnateSpell {
                    spell: ContentKey::new("t", spell).unwrap(),
                    value: value.parse().unwrap(),
                }
            }
            "damage_penalty" => {
                let (target, percent) = value.split_once(':').unwrap();
                AbilityDef::DamagePenalty {
                    target: ContentKey::new("t", target).unwrap(),
                    percent: percent.parse().unwrap(),
                }
            }
            "damage_penalty_by_flag" => {
                let (flag, percent) = value.split_once(':').unwrap();
                AbilityDef::DamagePenaltyByFlag {
                    flag: CreatureFlag::from_name(flag).unwrap(),
                    percent: percent.parse().unwrap(),
                }
            }
            "drawn_spell" => {
                let (spell, weight) = value.split_once(':').unwrap();
                AbilityDef::DrawnSpell {
                    spell: ContentKey::new("t", spell).unwrap(),
                    weight: weight.parse().unwrap(),
                }
            }
            flag => {
                flags.push(String::from(flag));
                continue;
            }
        };
        abilities.push(ability);
    }
    Traits {
        info: CreatureInfoDef {
            flags,
            faction,
            ai_value: ai,
            upgrades,
            level,
            ..CreatureInfoDef::default()
        },
        abilities,
        shots,
        casts,
    }
}

pub fn battle_params() -> BattleParamsDef {
    BattleParamsDef {
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
        // Morale and luck never trigger by default (tests stay independent of them); morale and
        // luck tests and the property tests use `ruleset_with`.
        morale_dice: 24,
        good_morale_chances: vec![0, 0, 0],
        bad_morale_chances: vec![0, 0, 0],
        luck_dice: 24,
        good_luck_chances: vec![0, 0, 0],
        war_machine_positions: vec![[1, 3], [1, 1], [1, 9]],
        loose_formation: vec![
            vec![5],
            vec![3, 7],
            vec![2, 5, 8],
            vec![1, 4, 6, 9],
            vec![0, 2, 5, 8, 10],
            vec![0, 2, 4, 6, 8, 10],
            vec![0, 2, 4, 5, 6, 8, 10],
        ],
        // The original's tight formation (VCMI layout).
        tight_formation: vec![
            vec![5],
            vec![4, 6],
            vec![4, 5, 6],
            vec![2, 4, 6, 8],
            vec![2, 4, 5, 6, 8],
            vec![0, 2, 4, 6, 8, 10],
            vec![0, 2, 4, 5, 6, 8, 10],
        ],
        tactics_zone_columns: vec![3, 5, 7],
        surrender: Some(SurrenderDef {
            resource: ContentKey::new("t", "gold").unwrap(),
            cost_percent: 50,
        }),
        hero_casts_per_round: 1,
        side_stack_limit: 20,
        siege: Some(siege_field()),
    }
}

/// The siege's field of the test rulesets (E04c T5): the original's (`installation.md` §7.27,
/// §7.28).
pub fn siege_field() -> SiegeFieldDef {
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

/// The test sieges (BAT-SIE-001): `t:castle`'s moat does 70, `t:inferno`'s 90 with a second row
/// (the Fortress's shape), `t:necropolis` lays `t:landmine`'s mines instead, at least 150.
pub fn siege_defs() -> Vec<SiegeDef> {
    let key = |k: &str| ContentKey::new("t", k).unwrap();
    vec![
        SiegeDef {
            faction: key("castle"),
            moat_damage: 70,
            second_moat: false,
            mines: None,
            hit_points: vec![2; 8],
        },
        SiegeDef {
            faction: key("inferno"),
            moat_damage: 90,
            second_moat: true,
            mines: None,
            hit_points: vec![2; 8],
        },
        SiegeDef {
            faction: key("necropolis"),
            moat_damage: 150,
            second_moat: false,
            mines: Some(key("landmine")),
            hit_points: vec![2; 8],
        },
    ]
}

/// Gold costs of some test creatures (BAT-FLE-002): those of the original's Pikemen, Pixies and
/// war machines; the others cost nothing.
pub const COSTS: &[(&str, u32)] = &[
    ("fixed", 60),
    ("flyer", 25),
    ("ballista", 2500),
    ("cart", 1000),
    ("tent", 750),
];

fn obstacle(
    name: &str,
    terrains: &[&str],
    width: u8,
    height: u8,
    blocked: &[(i8, i8)],
    absolute: bool,
) -> ObstacleDef {
    ObstacleDef {
        key: ContentKey::new("t", name).unwrap(),
        terrains: terrains
            .iter()
            .map(|t| ContentKey::new("t", t).unwrap())
            .collect(),
        battlefields: Vec::new(),
        width,
        height,
        blocked: blocked.to_vec(),
        absolute,
    }
}

pub fn ruleset() -> BattleRuleset {
    ruleset_with("t:faithful", |_| {})
}

/// The test ruleset's special battlefields (E04c T2b): Cursed Ground's, Magic Plains' and an
/// elemental field's rules, the morale and luck of Holy Ground and Clover Field by faction, a
/// shore with its own obstacle (`reef`), two boats' fixed hexes and a deck without rules.
fn battlefield_defs() -> Vec<BattlefieldDef> {
    let key = |n: &str| ContentKey::new("t", n).unwrap();
    let field = |n: &str| BattlefieldDef {
        key: key(n),
        own_obstacles: false,
        blocked: vec![],
        morale: vec![],
        luck: vec![],
        no_morale: false,
        no_luck: false,
        no_native: false,
        max_spell_level: None,
        no_creature_casts: false,
        expert_schools: vec![],
        expert_other_casts: false,
    };
    let change = |factions: &[&str], amount| FactionChangeDef {
        factions: factions.iter().map(|f| key(f)).collect(),
        amount,
    };
    vec![
        BattlefieldDef {
            own_obstacles: true,
            no_morale: true,
            no_luck: true,
            no_native: true,
            max_spell_level: Some(1),
            no_creature_casts: true,
            ..field("cursed")
        },
        BattlefieldDef {
            expert_schools: vec![key("air"), key("fire")],
            expert_other_casts: true,
            ..field("plains")
        },
        BattlefieldDef {
            expert_schools: vec![key("fire")],
            ..field("fiery")
        },
        BattlefieldDef {
            morale: vec![change(&["castle"], 1), change(&["inferno"], -1)],
            ..field("holy")
        },
        BattlefieldDef {
            luck: vec![change(&["necropolis"], 2)],
            ..field("clover")
        },
        BattlefieldDef {
            own_obstacles: true,
            ..field("shore")
        },
        BattlefieldDef {
            blocked: vec![26, 27, 43, 44],
            ..field("boats")
        },
        field("deck"),
    ]
}

/// The masteries of a hero with these school levels (by school handle): every spell of `r` above
/// none at the best of its schools' levels, the Heroes context's rule (HER-SPL-003) that the
/// seam carries per spell since E04b T8a (BAT-Q19).
pub fn masteries(r: &BattleRuleset, schools: [u8; 4]) -> Vec<(h3_content::SpellId, u8)> {
    r.spells()
        .iter()
        .enumerate()
        .filter_map(|(i, spell)| {
            let best = spell
                .schools
                .iter()
                .map(|l| schools[l.index().min(3)])
                .max();
            let mastery = best.unwrap_or(0);
            (mastery > 0).then(|| (h3_content::SpellId::from_index(i).unwrap(), mastery))
        })
        .collect()
}

/// Sets the masteries of `side`'s hero (by index) in `s` as [`masteries`] gives them.
pub fn set_masteries(s: &mut BattleState, side: usize, r: &BattleRuleset, schools: [u8; 4]) {
    if let Some(h) = s.heroes[side].as_mut() {
        h.magic.masteries = crate::magic::Masteries::of(&masteries(r, schools));
    }
}

/// The test ruleset with profile `profile` (`t:faithful` without quirks, `t:quirky` with
/// Q-0003, `t:single` with Q-0005) and edited battle parameters.
pub fn ruleset_with(profile: &str, edit: impl FnOnce(&mut BattleParamsDef)) -> BattleRuleset {
    ruleset_from(profile, edit, |_, _| {})
}

/// The core and battle definitions of test creatures `table`.
fn creature_defs(table: &[Creature]) -> (Vec<CreatureDef>, Vec<BattleCreatureDef>) {
    let mut creatures = Vec::new();
    let mut battle_creatures = Vec::new();
    for &(name, attack, defense, damage_min, damage_max, hit_points, speed, traits) in table {
        let key = ContentKey::new("t", name).unwrap();
        let traits = parse_traits(traits);
        battle_creatures.push(BattleCreatureDef {
            key: key.clone(),
            attack,
            defense,
            damage_min,
            damage_max,
            hit_points,
            shots: traits.shots,
            casts: traits.casts,
            abilities: traits.abilities,
        });
        creatures.push(CreatureDef {
            key,
            speed,
            info: CreatureInfoDef {
                cost: COSTS
                    .iter()
                    .filter(|(n, _)| *n == name)
                    .map(|&(_, gold)| (ContentKey::new("t", "gold").unwrap(), gold))
                    .collect(),
                ..traits.info
            },
        });
    }
    (creatures, battle_creatures)
}

/// The test ruleset (profile `profile`) with the town's turret (BAT-SIE-016, E04c T7): the
/// sieges with a Citadel or Castle set the keep and the towers up.
pub fn turret_ruleset(profile: &str) -> BattleRuleset {
    ruleset_from(profile, |_| {}, add_turret)
}

/// Adds the town's turret ([`TURRET`]) to a test pack's creatures, as [`turret_ruleset`] has it.
pub fn add_turret(core: &mut ContentPack, battle: &mut BattlePack) {
    let (mut c, mut b) = creature_defs(&[TURRET]);
    core.creatures.append(&mut c);
    battle.creatures.append(&mut b);
}

/// The test ruleset (profile `profile`) with a test's own creatures `extra` beside
/// [`CREATURES`], which stays unchanged for every other test.
pub fn ruleset_with_creatures(profile: &str, extra: &[Creature]) -> BattleRuleset {
    ruleset_from(
        profile,
        |_| {},
        |core, battle| {
            let (mut c, mut b) = creature_defs(extra);
            core.creatures.append(&mut c);
            battle.creatures.append(&mut b);
        },
    )
}

/// [`turret_ruleset`] with Fight Values `(name, value)`, the computer's worth of a creature
/// (E04c T7d, BAT-SIE-021; the other test creatures have none).
pub fn fighting_turret_ruleset(profile: &str, fights: &[(&str, u32)]) -> BattleRuleset {
    ruleset_from(
        profile,
        |_| {},
        |core, battle| {
            add_turret(core, battle);
            for def in &mut core.creatures {
                if let Some(&(_, value)) = fights.iter().find(|(n, _)| def.key.name() == *n) {
                    def.info.fight_value = value;
                }
            }
        },
    )
}

/// [`ruleset_with`], with the pack's core and battle parts patched before they are compiled
/// (review repros only).
pub fn ruleset_from(
    profile: &str,
    edit: impl FnOnce(&mut BattleParamsDef),
    patch: impl FnOnce(&mut ContentPack, &mut BattlePack),
) -> BattleRuleset {
    try_ruleset_from(profile, edit, patch).unwrap()
}

/// [`ruleset_from`], or the compiler's refusal.
pub fn try_ruleset_from(
    profile: &str,
    edit: impl FnOnce(&mut BattleParamsDef),
    patch: impl FnOnce(&mut ContentPack, &mut BattlePack),
) -> Result<BattleRuleset, BattleCompileError> {
    let (creatures, battle_creatures) = creature_defs(CREATURES);
    let mut battle = battle_params();
    edit(&mut battle);
    let (spells, battle_spells): (Vec<_>, Vec<_>) = SPELLS
        .iter()
        .map(|&spell| spell_def(spell))
        .chain(TIMED.iter().map(|&spell| timed_def(spell)))
        .chain(other_defs())
        .chain(control_defs())
        .chain(field_defs())
        .chain(raise_defs())
        .chain(create_defs())
        .chain(strike_defs())
        .chain(quake_defs())
        .map(SpellDef::split)
        .unzip();
    let manifest = PackManifest {
        id: String::from("t"),
        version: String::from("1"),
        depends: vec![],
    };
    let mut battle_pack = BattlePack {
        obstacles: vec![
            obstacle("stone", &["plain", "lava"], 2, 1, &[(0, 0), (1, 0)], false),
            obstacle(
                "cliff",
                &["lava"],
                0,
                0,
                &[(5, 4), (6, 4), (7, 4), (8, 4)],
                true,
            ),
            obstacle("tower", &["tall"], 1, 10, &[(0, 0)], false),
            ObstacleDef {
                battlefields: vec![ContentKey::new("t", "shore").unwrap()],
                ..obstacle("reef", &[], 2, 1, &[(0, 0), (1, 0)], false)
            },
        ],
        battlefields: battlefield_defs(),
        sieges: siege_defs(),
        creatures: battle_creatures,
        spells: battle_spells,
        battle: Some(battle),
        ..BattlePack::new(manifest.clone())
    };
    let mut pack = ContentPack {
        creatures,
        factions: [
            ("castle", Some("plain")),
            ("inferno", Some("lava")),
            ("necropolis", None),
        ]
        .iter()
        .map(|&(f, native)| FactionDef {
            key: ContentKey::new("t", f).unwrap(),
            name: String::from(f),
            native_terrain: native.map(|t| ContentKey::new("t", t).unwrap()),
        })
        .collect(),
        terrains: ["plain", "lava", "void", "tall"]
            .iter()
            .map(|t| TerrainDef {
                key: ContentKey::new("t", t).unwrap(),
                name: String::from(*t),
            })
            .collect(),
        profiles: vec![
            ProfileDef {
                key: ContentKey::new("t", "faithful").unwrap(),
                quirks: vec![],
            },
            ProfileDef {
                key: ContentKey::new("t", "quirky").unwrap(),
                quirks: vec![(String::from("quirk.battle.no_luck_without_hero"), true)],
            },
            ProfileDef {
                key: ContentKey::new("t", "single").unwrap(),
                quirks: vec![(String::from("quirk.battle.single_precision_rates"), true)],
            },
            ProfileDef {
                key: ContentKey::new("t", "unlimited").unwrap(),
                quirks: vec![(String::from("quirk.battle.unlimited_ballista_shots"), true)],
            },
            ProfileDef {
                key: ContentKey::new("t", "first").unwrap(),
                quirks: vec![(String::from("quirk.battle.first_protection"), true)],
            },
            ProfileDef {
                key: ContentKey::new("t", "frenzy").unwrap(),
                quirks: vec![(String::from("quirk.battle.frenzy_reduced_defense"), true)],
            },
            ProfileDef {
                key: ContentKey::new("t", "frenzy_single").unwrap(),
                quirks: vec![
                    (String::from("quirk.battle.frenzy_reduced_defense"), true),
                    (String::from("quirk.battle.single_precision_rates"), true),
                ],
            },
            ProfileDef {
                key: ContentKey::new("t", "counterstrike").unwrap(),
                quirks: vec![(String::from("quirk.battle.counterstrike_extra_round"), true)],
            },
            ProfileDef {
                key: ContentKey::new("t", "fireshield").unwrap(),
                quirks: vec![(String::from("quirk.battle.fire_shield_first_stage"), true)],
            },
            ProfileDef {
                key: ContentKey::new("t", "early").unwrap(),
                quirks: vec![(String::from("quirk.battle.early_mass_version"), true)],
            },
            ProfileDef {
                key: ContentKey::new("t", "hypnosis").unwrap(),
                quirks: vec![(
                    String::from("quirk.battle.mass_cure_skips_hypnotized"),
                    true,
                )],
            },
            ProfileDef {
                key: ContentKey::new("t", "anchors").unwrap(),
                quirks: vec![(String::from("quirk.battle.obstacle_anchors"), true)],
            },
            ProfileDef {
                key: ContentKey::new("t", "draws").unwrap(),
                quirks: vec![(String::from("quirk.battle.draws_avoid_hidden"), true)],
            },
            ProfileDef {
                key: ContentKey::new("t", "sacrifice").unwrap(),
                quirks: vec![(
                    String::from("quirk.battle.sacrifice_tests_as_resurrection"),
                    true,
                )],
            },
            ProfileDef {
                key: ContentKey::new("t", "clone_on_original").unwrap(),
                quirks: vec![(String::from("quirk.battle.clone_on_its_original"), true)],
            },
            ProfileDef {
                key: ContentKey::new("t", "first_stack").unwrap(),
                quirks: vec![(String::from("quirk.battle.periodic_first_stack"), true)],
            },
            ProfileDef {
                key: ContentKey::new("t", "fort_ballista").unwrap(),
                quirks: vec![(
                    String::from("quirk.battle.no_defender_ballista_at_fort"),
                    true,
                )],
            },
            ProfileDef {
                key: ContentKey::new("t", "berserk_position").unwrap(),
                quirks: vec![(
                    String::from("quirk.battle.berserk_walks_to_position_hex"),
                    true,
                )],
            },
            ProfileDef {
                key: ContentKey::new("t", "slips").unwrap(),
                quirks: vec![(String::from("quirk.battle.original_ai_slips"), true)],
            },
            ProfileDef {
                key: ContentKey::new("t", "let_off").unwrap(),
                quirks: vec![(String::from("quirk.battle.computer_morale_let_off"), true)],
            },
            ProfileDef {
                key: ContentKey::new("t", "table_breath").unwrap(),
                quirks: vec![(String::from("quirk.battle.breath_skips_side_columns"), true)],
            },
            ProfileDef {
                key: ContentKey::new("t", "nowhere").unwrap(),
                quirks: vec![(
                    String::from("quirk.battle.retaliation_at_a_dead_striker"),
                    true,
                )],
            },
            ProfileDef {
                key: ContentKey::new("t", "rear_raise").unwrap(),
                quirks: vec![(String::from("quirk.battle.raise_by_rear_hex"), true)],
            },
            ProfileDef {
                key: ContentKey::new("t", "original").unwrap(),
                quirks: vec![
                    (String::from("quirk.battle.first_protection"), true),
                    (String::from("quirk.battle.single_precision_rates"), true),
                    (String::from("quirk.battle.fire_shield_first_stage"), true),
                ],
            },
        ],
        // Fire's protection is checked before air's (quirk Q-0007), against the pack order.
        spell_schools: [("air", 1), ("fire", 0)]
            .iter()
            .map(|&(name, protection_rank)| SpellSchoolDef {
                key: ContentKey::new("t", name).unwrap(),
                name: String::from(name),
                protection_rank,
            })
            .collect(),
        spells,
        hero_classes: vec![HeroClassDef {
            key: ContentKey::new("t", "knight").unwrap(),
            name: String::from("Knight"),
            faction: ContentKey::new("t", "castle").unwrap(),
            primary: [2, 2, 1, 1],
            primary_gain: [[35, 45, 10, 10], [30, 30, 20, 20]],
            skill_weights: vec![],
            offer_intervals: [6, 4],
        }],
        heroes: vec![HeroTypeDef {
            key: ContentKey::new("t", "orrin").unwrap(),
            name: String::from("Orrin"),
            class: ContentKey::new("t", "knight").unwrap(),
            skills: vec![],
            spell_book: false,
            spells: vec![],
            army: vec![StartingStackDef {
                creature: ContentKey::new("t", "fixed").unwrap(),
                count: [1, 2],
            }],
            war_machines: vec![],
            specialty: vec![],
        }],
        resources: vec![ResourceDef {
            key: ContentKey::new("t", "gold").unwrap(),
            name: String::from("Gold"),
        }],
        ..ContentPack::new(manifest)
    };
    patch(&mut pack, &mut battle_pack);
    let core = Ruleset::compile(&[pack], profile)?;
    BattleRuleset::compile(Arc::new(core), &[battle_pack])
}

/// A test spell: name, category, schools, target and side.
pub type SpellSpec = (
    &'static str,
    SpellCategory,
    &'static [&'static str],
    SpellTarget,
    TargetSide,
);

/// The test spells (BAT-MAG-001 to BAT-MAG-008), all level 1, costing 5/4/4/4 and dealing 10
/// per power plus 10/10/20/30, negative unless said otherwise: `bolt` damages one enemy stack;
/// `smite` any stack that is not undead, in two schools; `gaze` is a creature ability; `ward`
/// has no effects and `portal` is an adventure spell; `nova` damages the stacks within one hex
/// of a hex; `rally` (positive) reaches only own stacks; `chain` strikes an enemy stack and jumps
/// to two more of either side (three from Advanced), each taking half the value (neutral, aimed
/// only at enemies like Chain Lightning); `storm` strikes every
/// stack (neutral); `ring` the stacks one hex from a hex, not on it.
pub const SPELLS: &[SpellSpec] = &[
    (
        "bolt",
        SpellCategory::Combat,
        &["air"],
        SpellTarget::Stack,
        TargetSide::Enemy,
    ),
    (
        "gaze",
        SpellCategory::Ability,
        &[],
        SpellTarget::None,
        TargetSide::Any,
    ),
    (
        "smite",
        SpellCategory::Combat,
        &["air", "fire"],
        SpellTarget::Stack,
        TargetSide::Any,
    ),
    (
        "ward",
        SpellCategory::Combat,
        &["fire"],
        SpellTarget::None,
        TargetSide::Any,
    ),
    (
        "portal",
        SpellCategory::Adventure,
        &["air"],
        SpellTarget::None,
        TargetSide::Any,
    ),
    (
        "nova",
        SpellCategory::Combat,
        &["fire"],
        SpellTarget::Hex,
        TargetSide::Any,
    ),
    (
        "rally",
        SpellCategory::Combat,
        &["air"],
        SpellTarget::Stack,
        TargetSide::Own,
    ),
    (
        "chain",
        SpellCategory::Combat,
        &["air"],
        SpellTarget::Stack,
        TargetSide::Enemy,
    ),
    (
        "storm",
        SpellCategory::Combat,
        &["fire"],
        SpellTarget::None,
        TargetSide::Any,
    ),
    (
        "ring",
        SpellCategory::Combat,
        &["fire"],
        SpellTarget::Hex,
        TargetSide::Any,
    ),
];

/// A timed test spell (BAT-MAG-009 to 011): name, positive, changes, effect by mastery, the
/// spell it counters, mass at Expert, a mind spell, war machines excluded.
pub type TimedSpec = (
    &'static str,
    bool,
    &'static [TimedChange],
    [u16; 4],
    Option<&'static str>,
    bool,
    bool,
    bool,
);

/// The timed test spells, all level 1 in the air school (`fireward` in the fire school), costing
/// 5/4/4/4, after the original's (survey §14, §15): `haste` and `sloth` (Slow) counter each
/// other; `ray` (Disrupting Ray) is permanent and cumulative; `bless` and `curse` counter each
/// other; `shield`, `airshield`, `airward` and `fireward` protect, `slayer` is Slayer,
/// `counter` Counterstrike, `fireshield` (fire school) Fire Shield and `frenzy` Frenzy (through
/// the stack's next turn); `antimagic` keeps spells below level 1, 1, 2, 2 off a stack (an
/// Anti-Magic), `mirror` redirects with 50, 50, 100, 100 % (a Magic Mirror) and `gloom` is a
/// level-2 Sorrow.
pub const TIMED: &[TimedSpec] = &[
    (
        "haste",
        true,
        &[TimedChange::Speed],
        [3, 3, 5, 5],
        Some("sloth"),
        true,
        false,
        true,
    ),
    (
        "sloth",
        false,
        &[TimedChange::SpeedPercent],
        [75, 75, 50, 50],
        Some("haste"),
        true,
        false,
        true,
    ),
    (
        "prayer",
        true,
        &[
            TimedChange::Attack,
            TimedChange::Defense,
            TimedChange::Speed,
        ],
        [2, 2, 4, 4],
        None,
        true,
        false,
        false,
    ),
    (
        "weaken",
        false,
        &[TimedChange::AttackLoss],
        [3, 3, 6, 6],
        None,
        true,
        false,
        false,
    ),
    (
        "ray",
        false,
        &[TimedChange::DefenseLoss],
        [3, 3, 4, 5],
        None,
        false,
        false,
        false,
    ),
    (
        "skin",
        true,
        &[TimedChange::Defense],
        [3, 3, 6, 6],
        None,
        true,
        false,
        false,
    ),
    (
        "lust",
        true,
        &[TimedChange::MeleeAttack],
        [3, 3, 6, 6],
        None,
        true,
        false,
        true,
    ),
    (
        "aim",
        true,
        &[TimedChange::RangedAttack],
        [3, 3, 6, 6],
        None,
        true,
        false,
        false,
    ),
    (
        "bless",
        true,
        &[TimedChange::MaximumDamage],
        [0, 0, 1, 1],
        Some("curse"),
        true,
        false,
        false,
    ),
    (
        "curse",
        false,
        &[TimedChange::MinimumDamage],
        [0, 0, 1, 1],
        Some("bless"),
        true,
        false,
        false,
    ),
    (
        "mirth",
        true,
        &[TimedChange::Morale],
        [1, 1, 2, 2],
        None,
        true,
        true,
        false,
    ),
    (
        "sorrow",
        false,
        &[TimedChange::MoraleLoss],
        [1, 1, 2, 2],
        None,
        true,
        true,
        false,
    ),
    (
        "fortune",
        true,
        &[TimedChange::Luck],
        [1, 1, 2, 2],
        None,
        true,
        false,
        false,
    ),
    (
        "misfortune",
        false,
        &[TimedChange::LuckLoss],
        [1, 1, 2, 2],
        None,
        true,
        false,
        false,
    ),
    (
        "shield",
        true,
        &[TimedChange::MeleeProtection],
        [85, 85, 70, 70],
        None,
        true,
        false,
        false,
    ),
    (
        "airshield",
        true,
        &[TimedChange::RangedProtection],
        [75, 75, 50, 50],
        None,
        true,
        false,
        false,
    ),
    (
        "airward",
        true,
        &[TimedChange::SchoolProtection],
        [50, 50, 50, 50],
        None,
        true,
        false,
        false,
    ),
    (
        "fireward",
        true,
        &[TimedChange::SchoolProtection],
        [70, 70, 50, 50],
        None,
        true,
        false,
        false,
    ),
    (
        "slayer",
        true,
        &[TimedChange::Slayer],
        [8, 8, 8, 8],
        None,
        false,
        false,
        false,
    ),
    (
        "counter",
        true,
        &[TimedChange::Retaliations],
        [1, 1, 2, 2],
        None,
        true,
        false,
        true,
    ),
    (
        "fireshield",
        true,
        &[TimedChange::FireShield],
        [20, 20, 25, 30],
        None,
        false,
        false,
        false,
    ),
    (
        "frenzy",
        true,
        &[TimedChange::Frenzy],
        [100, 100, 150, 200],
        None,
        false,
        true,
        false,
    ),
    (
        "antimagic",
        true,
        &[TimedChange::SpellImmunity],
        [1, 1, 2, 2],
        None,
        false,
        false,
        false,
    ),
    (
        "mirror",
        true,
        &[TimedChange::MagicMirror],
        [50, 50, 100, 100],
        None,
        false,
        false,
        false,
    ),
    (
        "gloom",
        false,
        &[TimedChange::MoraleLoss],
        [1, 1, 2, 2],
        None,
        false,
        false,
        false,
    ),
];

/// The definition of a timed test spell.
fn timed_def(
    (name, positive, changes, effect, counter, mass, mind, no_machines): TimedSpec,
) -> SpellDef {
    let permanent = name == "ray";
    SpellDef {
        key: ContentKey::new("t", name).unwrap(),
        name: String::from(name),
        category: SpellCategory::Combat,
        level: if name == "gloom" { 2 } else { 1 },
        schools: vec![
            ContentKey::new(
                "t",
                if name == "fireward" || name == "fireshield" {
                    "fire"
                } else {
                    "air"
                },
            )
            .unwrap(),
        ],
        cost: [5, 4, 4, 4],
        power: 0,
        effect,
        polarity: if positive {
            SpellPolarity::Positive
        } else {
            SpellPolarity::Negative
        },
        magic: SpellMagic {
            target: SpellTarget::Stack,
            side: if positive {
                TargetSide::Own
            } else {
                TargetSide::Enemy
            },
            conditions: TargetConditions {
                requires: CreatureFlags::default(),
                excludes: if no_machines {
                    CreatureFlags::default().with(CreatureFlag::WarMachine)
                } else {
                    CreatureFlags::default()
                },
                needs_damage: false,
            },
            effects: changes.iter().map(|&c| SpellEffect::Timed(c)).collect(),
            mind,
            mass_from: mass.then_some(3),
            duration: if permanent {
                EffectDuration::Permanent
            } else if name == "frenzy" {
                EffectDuration::OwnTurn
            } else {
                EffectDuration::Rounds
            },
            cumulative: permanent,
            // The original's specialist rules (`installation.md` §7.12).
            specialist: match name {
                "haste" | "prayer" | "weaken" | "skin" | "lust" | "aim" => {
                    Some(SpecialistBonus::CreatureLevel([3, 3, 2, 2, 1, 1, 0]))
                }
                "slayer" => Some(SpecialistBonus::CreatureLevel([4, 3, 2, 1, 0, 0, 0])),
                "ray" => Some(SpecialistBonus::Fixed(2)),
                "fortune" => Some(SpecialistBonus::Total(3)),
                _ => None,
            },
            // When a helpful caster finds it useful (BAT-MAG-050), as the original's spells.
            useful: match name {
                "shield" | "fireshield" => Usefulness::EnemyMelee,
                "airshield" => Usefulness::EnemyShooter,
                "airward" | "fireward" | "antimagic" | "mirror" => Usefulness::EnemySpellBook,
                "lust" => Usefulness::Melee,
                "aim" => Usefulness::Shooting,
                "prayer" => Usefulness::NotActed,
                "slayer" => Usefulness::EnemyKing,
                _ => Usefulness::Always,
            },
            ..SpellMagic::default()
        },
        counters: counter
            .map(|c| vec![ContentKey::new("t", c).unwrap()])
            .unwrap_or_default(),
        dispels: vec![],
        spares: vec![],
        tests_as: None,
        summons: None,
        granted_by_name_only: false,
    }
}

/// The test Earthquake (`t:quake`, E04c T6, BAT-SIE-015): the original's 2, 2, 3, 4 hits by
/// mastery, of the air school.
pub fn quake_defs() -> [SpellDef; 1] {
    [SpellDef {
        key: ContentKey::new("t", "quake").unwrap(),
        name: String::from("quake"),
        category: SpellCategory::Combat,
        level: 3,
        schools: vec![ContentKey::new("t", "air").unwrap()],
        cost: [20, 17, 17, 17],
        power: 1,
        effect: [2, 2, 3, 4],
        polarity: SpellPolarity::Neutral,
        magic: SpellMagic {
            effects: vec![SpellEffect::Earthquake],
            ..SpellMagic::default()
        },
        counters: vec![],
        dispels: vec![],
        spares: vec![],
        tests_as: None,
        summons: None,
        granted_by_name_only: false,
    }]
}

/// The test Dispel (`t:dispel`, like the original's: own stacks below Advanced, a target with an
/// effect, no immunity or resistance, mass at Expert; it spares `t:sloth`), Cure (`t:cure`:
/// power × 5 + 10, 10, 20, 30; it ends `t:curse`, `t:sloth` and `t:hypnotize` only) and Dispel
/// Helpful Spells (`t:unbless`: a creature ability ending positive spells' effects, BAT-MAG-042).
pub fn other_defs() -> [SpellDef; 3] {
    let key = |k: &str| ContentKey::new("t", k).unwrap();
    let def = |name: &str, polarity, effect, magic| SpellDef {
        key: key(name),
        name: String::from(name),
        category: SpellCategory::Combat,
        level: 1,
        schools: vec![key("air")],
        cost: [5, 4, 4, 4],
        power: if name == "cure" { 5 } else { 0 },
        effect,
        polarity,
        magic,
        counters: vec![],
        dispels: vec![],
        spares: vec![],
        tests_as: None,
        summons: None,
        granted_by_name_only: false,
    };
    [
        SpellDef {
            spares: vec![key("sloth")],
            ..def(
                "dispel",
                SpellPolarity::Neutral,
                [0; 4],
                SpellMagic {
                    target: SpellTarget::Stack,
                    side: TargetSide::Own,
                    any_side_from: Some(2),
                    needs_effect: true,
                    ignores_immunity: true,
                    mass_from: Some(3),
                    effects: vec![SpellEffect::Dispel],
                    clears_obstacles_from: Some(3),
                    ..SpellMagic::default()
                },
            )
        },
        SpellDef {
            dispels: vec![key("curse"), key("sloth"), key("hypnotize")],
            ..def(
                "cure",
                SpellPolarity::Positive,
                [10, 10, 20, 30],
                SpellMagic {
                    target: SpellTarget::Stack,
                    side: TargetSide::Own,
                    mass_from: Some(3),
                    effects: vec![SpellEffect::Dispel, SpellEffect::Heal],
                    restores_health: true,
                    useful: Usefulness::Wounded,
                    ..SpellMagic::default()
                },
            )
        },
        SpellDef {
            category: SpellCategory::Ability,
            level: 0,
            schools: vec![],
            ..def(
                "unbless",
                SpellPolarity::Negative,
                [0; 4],
                SpellMagic {
                    target: SpellTarget::Stack,
                    side: TargetSide::Enemy,
                    ignores_immunity: true,
                    dispels_positive: true,
                    effects: vec![SpellEffect::Dispel],
                    ..SpellMagic::default()
                },
            )
        },
    ]
}

/// The strike's statuses (BAT-MAG-043, BAT-MAG-044), creature abilities sparing war machines:
/// `t:petrify` (Stone Gaze: level 3, ended by any damage), `t:paralysis` (Paralyze: level 4, a
/// 25 % retaliation, ended by any damage), `t:plague` (Disease: level 2, −2 attack and defense),
/// `t:wither` (Age: level 5) and `t:venom` (Poison: level 0, living creatures only); cast without
/// a test (ignoring immunity), `t:tether` (Bind: 255 rounds) and `t:acid` (Acid Breath: −3
/// defense for the battle, cumulative, also on war machines) (BAT-MAG-045, BAT-MAG-046).
pub fn strike_defs() -> Vec<SpellDef> {
    let key = |k: &str| ContentKey::new("t", k).unwrap();
    let def = |name: &str, level, effect, changes: &[TimedChange], ends, requires| SpellDef {
        key: key(name),
        name: String::from(name),
        category: SpellCategory::Ability,
        level,
        schools: vec![],
        cost: [0; 4],
        power: 0,
        effect,
        polarity: SpellPolarity::Negative,
        magic: SpellMagic {
            target: SpellTarget::Stack,
            side: TargetSide::Enemy,
            conditions: TargetConditions {
                requires,
                excludes: CreatureFlags::default().with(CreatureFlag::WarMachine),
                needs_damage: false,
            },
            effects: changes.iter().map(|&c| SpellEffect::Timed(c)).collect(),
            ends,
            ..SpellMagic::default()
        },
        counters: vec![],
        dispels: vec![],
        spares: vec![],
        tests_as: None,
        summons: None,
        granted_by_name_only: false,
    };
    let none = CreatureFlags::default();
    vec![
        def(
            "petrify",
            3,
            [0; 4],
            &[TimedChange::Petrify],
            EffectEnd::Damaged,
            none,
        ),
        def(
            "paralysis",
            4,
            [25; 4],
            &[TimedChange::Paralyze],
            EffectEnd::Damaged,
            none,
        ),
        def(
            "plague",
            2,
            [2; 4],
            &[TimedChange::AttackLoss, TimedChange::DefenseLoss],
            EffectEnd::Never,
            none,
        ),
        def(
            "wither",
            5,
            [0; 4],
            &[TimedChange::Age],
            EffectEnd::Never,
            none,
        ),
        def(
            "venom",
            0,
            [0; 4],
            &[TimedChange::Poison],
            EffectEnd::Never,
            none.with(CreatureFlag::Alive),
        ),
        {
            let mut tether = def(
                "tether",
                0,
                [0; 4],
                &[TimedChange::Bind],
                EffectEnd::Never,
                none,
            );
            tether.magic.duration = EffectDuration::Fixed(255);
            tether.magic.ignores_immunity = true;
            tether
        },
        {
            let mut acid = def(
                "acid",
                0,
                [3; 4],
                &[TimedChange::DefenseLoss],
                EffectEnd::Never,
                none,
            );
            acid.magic.duration = EffectDuration::Permanent;
            acid.magic.cumulative = true;
            acid.magic.ignores_immunity = true;
            acid.magic.conditions.excludes = none;
            acid
        },
    ]
}

/// The test Forgetfulness (`t:forget`: level 3, enemy shooters, a mind spell sparing war
/// machines, the mass version from Advanced with quirk Q-0011, else at Expert), Blind
/// (`t:blind`: level 2, 50, 50, 25, 0 %, ended by any damage, sparing the undead), Berserk
/// (`t:berserk`: level 4, the stacks of either side within 0, 0, 1, 2 hexes of its hex, 255
/// rounds, ended by the stack's own attack) and Hypnotize (`t:hypnotize`: level 3, power × 25 +
/// 10, 10, 20, 50 full hit points, never on a mirrored stack); each of the last two ends the
/// other.
pub fn control_defs() -> Vec<SpellDef> {
    let key = |k: &str| ContentKey::new("t", k).unwrap();
    let flags = |list: &[CreatureFlag]| {
        list.iter()
            .fold(CreatureFlags::default(), |set, &f| set.with(f))
    };
    let def = |name: &str, level, effect, magic| SpellDef {
        key: key(name),
        name: String::from(name),
        category: SpellCategory::Combat,
        level,
        schools: vec![key("air")],
        cost: [5, 4, 4, 4],
        power: 0,
        effect,
        polarity: SpellPolarity::Negative,
        magic,
        counters: vec![],
        dispels: vec![],
        spares: vec![],
        tests_as: None,
        summons: None,
        granted_by_name_only: false,
    };
    let machines = TargetConditions {
        requires: CreatureFlags::default(),
        excludes: flags(&[CreatureFlag::WarMachine]),
        needs_damage: false,
    };
    let enemy = |change, conditions| SpellMagic {
        target: SpellTarget::Stack,
        side: TargetSide::Enemy,
        mind: true,
        conditions,
        effects: vec![SpellEffect::Timed(change)],
        ..SpellMagic::default()
    };
    vec![
        def(
            "forget",
            3,
            [0; 4],
            SpellMagic {
                mass_from: Some(2),
                fixed_mass_from: Some(3),
                ..enemy(
                    TimedChange::Forgetfulness,
                    TargetConditions {
                        requires: flags(&[CreatureFlag::Shooter]),
                        excludes: flags(&[CreatureFlag::WarMachine]),
                        needs_damage: false,
                    },
                )
            },
        ),
        def(
            "blind",
            2,
            [50, 50, 25, 0],
            SpellMagic {
                ends: EffectEnd::Damaged,
                ..enemy(
                    TimedChange::Blind,
                    TargetConditions {
                        requires: CreatureFlags::default(),
                        excludes: flags(&[CreatureFlag::Undead, CreatureFlag::WarMachine]),
                        needs_damage: false,
                    },
                )
            },
        ),
        SpellDef {
            counters: vec![key("hypnotize")],
            ..def(
                "berserk",
                4,
                [0, 0, 1, 2],
                SpellMagic {
                    target: SpellTarget::Hex,
                    side: TargetSide::Any,
                    area: Some(SpellArea {
                        inner: 0,
                        radius: [0, 0, 1, 2],
                    }),
                    duration: EffectDuration::Fixed(255),
                    ends: EffectEnd::OwnAttack,
                    ..enemy(TimedChange::Berserk, machines)
                },
            )
        },
        SpellDef {
            counters: vec![key("berserk")],
            power: 25,
            ..def(
                "hypnotize",
                3,
                [10, 10, 20, 50],
                SpellMagic {
                    health_limit: true,
                    refuses: vec![TimedChange::MagicMirror],
                    ..enemy(TimedChange::Hypnotize, machines)
                },
            )
        },
    ]
}

/// The test Force Field (`t:wall`: level 3, the target hex and its upper right neighbour, from
/// Advanced also that one's upper left, for either side; 2 rounds), Fire Wall (`t:firewall`: a
/// fire spell of level 2, power × 10 + 10, 10, 20, 50 damage, the same hexes mirrored for the
/// defender, one obstacle per hex, 2 rounds), Teleport (`t:teleport`: level 3, an own stack
/// that is no war machine to a free hex), Quicksand (`t:quicksand`: level 2, 4, 4, 6 or 8 pits
/// on random free hexes for the whole battle, hidden from the other side) and Land Mine
/// (`t:landmine`: a fire spell of level 3, as many mines as Quicksand's pits, each doing power ×
/// 10 + 25, 25, 50, 100 damage to a stack of the other side that enters it unseen).
pub fn field_defs() -> Vec<SpellDef> {
    let key = |k: &str| ContentKey::new("t", k).unwrap();
    let def = |name: &str, (level, school), polarity, magic| SpellDef {
        key: key(name),
        name: String::from(name),
        category: SpellCategory::Combat,
        level,
        schools: vec![key(school)],
        cost: [5, 4, 4, 4],
        power: 0,
        effect: [0; 4],
        polarity,
        magic,
        counters: vec![],
        dispels: vec![],
        spares: vec![],
        tests_as: None,
        summons: None,
        granted_by_name_only: false,
    };
    let shape = |mirrored, separate| ObstacleShape {
        steps: vec![HexStep::TopRight, HexStep::TopLeft],
        hexes: [2, 2, 3, 3],
        random: false,
        mirrored,
        separate,
        rounds: 2,
    };
    let obstacle = |kind, shape| SpellMagic {
        target: SpellTarget::Hex,
        obstacle: Some(shape),
        effects: vec![SpellEffect::Obstacle(kind)],
        ..SpellMagic::default()
    };
    vec![
        def(
            "wall",
            (3, "air"),
            SpellPolarity::Neutral,
            obstacle(ObstacleKind::Wall, shape(false, false)),
        ),
        SpellDef {
            power: 10,
            effect: [10, 10, 20, 50],
            ..def(
                "firewall",
                (2, "fire"),
                SpellPolarity::Neutral,
                obstacle(ObstacleKind::Fire, shape(true, true)),
            )
        },
        def(
            "teleport",
            (3, "air"),
            SpellPolarity::Positive,
            SpellMagic {
                target: SpellTarget::Stack,
                side: TargetSide::Own,
                conditions: TargetConditions {
                    requires: CreatureFlags::default(),
                    excludes: CreatureFlags::default().with(CreatureFlag::WarMachine),
                    needs_damage: false,
                },
                effects: vec![SpellEffect::Teleport],
                ..SpellMagic::default()
            },
        ),
        def(
            "quicksand",
            (2, "air"),
            SpellPolarity::Neutral,
            SpellMagic {
                obstacle: Some(ObstacleShape {
                    steps: vec![],
                    hexes: [4, 4, 6, 8],
                    random: true,
                    mirrored: false,
                    separate: true,
                    rounds: 0,
                }),
                effects: vec![SpellEffect::Obstacle(ObstacleKind::Pit)],
                ..SpellMagic::default()
            },
        ),
        SpellDef {
            power: 10,
            effect: [25, 25, 50, 100],
            ..def(
                "landmine",
                (3, "fire"),
                SpellPolarity::Neutral,
                SpellMagic {
                    obstacle: Some(ObstacleShape {
                        steps: vec![],
                        hexes: [4, 4, 6, 8],
                        random: true,
                        mirrored: false,
                        separate: true,
                        rounds: 0,
                    }),
                    effects: vec![SpellEffect::Obstacle(ObstacleKind::Mine)],
                    ..SpellMagic::default()
                },
            )
        },
        def(
            "remove",
            (2, "air"),
            SpellPolarity::Neutral,
            SpellMagic {
                target: SpellTarget::Hex,
                effects: vec![SpellEffect::RemoveObstacle],
                removes_from: [Some(3), Some(2), Some(3), Some(3)],
                ..SpellMagic::default()
            },
        ),
    ]
}

/// The raising spells (BAT-MAG-035, BAT-MAG-036): `rise` (Resurrection: an air spell of level 4,
/// 50 per power, temporary below Advanced), `animate` (Animate Dead: undead only, permanent) and
/// `offering` (Sacrifice: fire, level 5, tested as `rise` under quirk BAT-QUIRK-5).
pub fn raise_defs() -> Vec<SpellDef> {
    let key = |k: &str| ContentKey::new("t", k).unwrap();
    let flags = |names: &[CreatureFlag]| {
        names
            .iter()
            .fold(CreatureFlags::default(), |set, &f| set.with(f))
    };
    let def = |name: &str, (level, school), (power, effect), magic| SpellDef {
        key: key(name),
        name: String::from(name),
        category: SpellCategory::Combat,
        level,
        schools: vec![key(school)],
        cost: [5, 4, 4, 4],
        power,
        effect,
        polarity: SpellPolarity::Positive,
        magic,
        counters: vec![],
        dispels: vec![],
        spares: vec![],
        tests_as: None,
        summons: None,
        granted_by_name_only: false,
    };
    let magic = |requires, excludes, effect| SpellMagic {
        target: SpellTarget::Stack,
        side: TargetSide::Own,
        conditions: TargetConditions {
            requires,
            excludes,
            needs_damage: false,
        },
        effects: vec![effect],
        ..SpellMagic::default()
    };
    let machine = flags(&[CreatureFlag::WarMachine]);
    vec![
        def(
            "rise",
            (4, "air"),
            (50, [40, 40, 80, 160]),
            SpellMagic {
                temporary_below: Some(2),
                ..magic(flags(&[CreatureFlag::Alive]), machine, SpellEffect::Raise)
            },
        ),
        def(
            "animate",
            (3, "air"),
            (50, [30, 30, 60, 160]),
            magic(flags(&[CreatureFlag::Undead]), machine, SpellEffect::Raise),
        ),
        SpellDef {
            tests_as: Some(key("rise")),
            summons: None,
            granted_by_name_only: false,
            ..def(
                "offering",
                (5, "fire"),
                (1, [3, 3, 6, 10]),
                magic(
                    flags(&[CreatureFlag::Alive]),
                    flags(&[CreatureFlag::Undead, CreatureFlag::WarMachine]),
                    SpellEffect::Sacrifice,
                ),
            )
        },
    ]
}

/// The spells that create stacks (BAT-MAG-037 to BAT-MAG-039): `copy` (Clone: an air spell of
/// level 4, of creatures up to level 1, 1, 2, 3 by mastery), `sprites` and `serfs` (Summon
/// Elemental: fire, level 5, 2, 2, 3, 4 creatures per power of `sprite` and of `serf`).
pub fn create_defs() -> Vec<SpellDef> {
    let key = |k: &str| ContentKey::new("t", k).unwrap();
    let def = |name: &str, (level, school), polarity, magic| SpellDef {
        key: key(name),
        name: String::from(name),
        category: SpellCategory::Combat,
        level,
        schools: vec![key(school)],
        cost: [5, 4, 4, 4],
        power: 0,
        effect: [1, 1, 2, 3],
        polarity,
        magic,
        counters: vec![],
        dispels: vec![],
        spares: vec![],
        tests_as: None,
        summons: None,
        granted_by_name_only: false,
    };
    let summon = |name: &str, creature: &str| SpellDef {
        effect: [2, 2, 3, 4],
        summons: Some(key(creature)),
        granted_by_name_only: false,
        ..def(
            name,
            (5, "fire"),
            SpellPolarity::Neutral,
            SpellMagic {
                effects: vec![SpellEffect::Summon],
                ..SpellMagic::default()
            },
        )
    };
    vec![
        def(
            "copy",
            (4, "air"),
            SpellPolarity::Positive,
            SpellMagic {
                target: SpellTarget::Stack,
                side: TargetSide::Own,
                conditions: TargetConditions {
                    requires: CreatureFlags::default(),
                    excludes: CreatureFlags::default().with(CreatureFlag::WarMachine),
                    needs_damage: false,
                },
                effects: vec![SpellEffect::Clone],
                ..SpellMagic::default()
            },
        ),
        summon("sprites", "sprite"),
        summon("serfs", "serf"),
    ]
}

/// The definition of a test spell.
fn spell_def((name, category, schools, target, side): SpellSpec) -> SpellDef {
    let castable = !matches!(name, "gaze" | "ward" | "portal");
    let excludes = if name == "smite" {
        CreatureFlags::default().with(CreatureFlag::Undead)
    } else {
        CreatureFlags::default()
    };
    SpellDef {
        key: ContentKey::new("t", name).unwrap(),
        name: String::from(name),
        category,
        level: u8::from(category != SpellCategory::Ability),
        schools: schools
            .iter()
            .map(|s| ContentKey::new("t", s).unwrap())
            .collect(),
        cost: [5, 4, 4, 4],
        power: 10,
        effect: [10, 10, 20, 30],
        polarity: match name {
            "rally" => SpellPolarity::Positive,
            // Chain Lightning and the area spells are neutral, as the original's (§7.8).
            "chain" | "storm" | "nova" | "ring" => SpellPolarity::Neutral,
            _ => SpellPolarity::Negative,
        },
        magic: SpellMagic {
            target,
            area: (target == SpellTarget::Hex).then_some(SpellArea {
                inner: u8::from(name == "ring"),
                radius: [1; 4],
            }),
            chain: (name == "chain").then_some(SpellChain {
                targets: [3, 3, 4, 4],
                factor_bp: 5000,
            }),
            side,
            conditions: TargetConditions {
                requires: CreatureFlags::default(),
                excludes,
                needs_damage: false,
            },
            effects: if castable {
                vec![SpellEffect::Damage]
            } else {
                vec![]
            },
            ..SpellMagic::default()
        },
        counters: vec![],
        dispels: vec![],
        spares: vec![],
        tests_as: None,
        summons: None,
        granted_by_name_only: false,
    }
}

/// `(slot, creature name, count)`.
pub type Spec = (u8, &'static str, u32);

pub fn army(rules: &BattleRuleset, spec: &[Spec]) -> Army {
    Army {
        stacks: spec
            .iter()
            .map(|&(slot, name, count)| ArmyStack {
                slot,
                creature: rules.creature_by_key(&std::format!("t:{name}")).unwrap(),
                count,
            })
            .collect(),
    }
}

/// A battle hero of the test ruleset's only hero type with some primary skills and mana, without
/// Tactics (so that no tactics phase runs, BAT-TAC-001).
pub fn hero(rules: &BattleRuleset) -> BattleHero {
    let mut hero = BattleHero::new(rules.hero_type_by_key("t:orrin").unwrap());
    [hero.attack, hero.defense, hero.power, hero.knowledge] = [3, 2, 1, 1];
    hero.magic.mana = 15;
    hero.magic.max_mana = 10;
    hero
}

/// A hero without skills, statistics, book or mana: a side brings its war machines only with a
/// hero (BAT-WAR-001).
pub fn bare_hero(rules: &BattleRuleset) -> BattleHero {
    BattleHero::new(rules.hero_type_by_key("t:orrin").unwrap())
}

/// `setup` with a [`bare_hero`] for each side that lists war machines and has no hero.
pub fn lead_machines(rules: &BattleRuleset, mut setup: BattleSetup) -> BattleSetup {
    for (hero, machines) in setup.heroes.iter_mut().zip(&setup.war_machines) {
        if hero.is_none() && !machines.is_empty() {
            *hero = Some(bare_hero(rules));
        }
    }
    setup
}

pub fn setup(rules: &BattleRuleset, attacker: &[Spec], defender: &[Spec]) -> BattleSetup {
    BattleSetup {
        attacker: army(rules, attacker),
        defender: army(rules, defender),
        obstacles: vec![],
        heroes: [None, None],
        terrain: None,
        battlefield: None,
        anti_magic: false,
        siege: None,
        war_machines: [vec![], vec![]],
        formations: [Formation::Loose; 2],
        funds: [0; 2],
        ..BattleSetup::default()
    }
}

pub fn battle(
    rules: &BattleRuleset,
    attacker: &[Spec],
    defender: &[Spec],
    seed: u64,
) -> BattleState {
    BattleState::new(rules, &setup(rules, attacker, defender), seed).unwrap()
}

/// A battle with obstacles at the given `(x, y)` hexes.
pub fn battle_with_obstacles(
    rules: &BattleRuleset,
    attacker: &[Spec],
    defender: &[Spec],
    obstacles: &[(i32, i32)],
) -> BattleState {
    let mut s = setup(rules, attacker, defender);
    s.obstacles = blocked(obstacles.iter().map(|&(x, y)| at(rules, x, y).0).collect());
    BattleState::new(rules, &s, 1).unwrap()
}

/// One obstacle without an anchor over `hexes` (it cannot be removed).
pub fn blocked(hexes: Vec<u8>) -> Vec<h3_contracts::Obstacle> {
    vec![h3_contracts::Obstacle {
        hexes,
        anchor: None,
        kind: None,
    }]
}

pub fn field(rules: &BattleRuleset) -> Field {
    Field::new(rules.battle().field)
}

pub fn at(rules: &BattleRuleset, x: i32, y: i32) -> Hex {
    field(rules).hex(x, y).unwrap()
}

/// A walk of stack `id` along row `y` from column `from` to column `to`, as its `Moved` events
/// report it: one `(stack, from, to)` per hex (BAT-MOV-003).
pub fn row_walk(rules: &BattleRuleset, id: u8, y: i32, from: i32, to: i32) -> Vec<(u8, Hex, Hex)> {
    let step = if to < from { -1 } else { 1 };
    let mut out = Vec::new();
    let mut x = from;
    while x != to {
        out.push((id, at(rules, x, y), at(rules, x + step, y)));
        x += step;
    }
    out
}

/// Moves stack `id` to `(x, y)` directly (test setup only).
pub fn place(state: &mut BattleState, rules: &BattleRuleset, id: u8, x: i32, y: i32) {
    state.stacks.as_mut_slice()[usize::from(id)].hex = at(rules, x, y);
}

/// Makes `id` the active stack (test setup only).
pub fn activate(state: &mut BattleState, id: u8) {
    state.active = Some(StackId(id));
}

/// Applies `command` for the side to act, returning the events.
pub fn act(
    state: &mut BattleState,
    rules: &BattleRuleset,
    command: Command,
) -> Result<Vec<Event>, crate::CommandError> {
    let side = state.side_to_act().unwrap_or(BattleSide::Attacker);
    let mut events = Vec::new();
    apply(state, rules, side, command, &mut events)?;
    Ok(events)
}

/// Lets the active stack defend until `id` is active.
pub fn advance_to(state: &mut BattleState, rules: &BattleRuleset, id: u8) {
    for _ in 0..64 {
        if state.active() == Some(StackId(id)) {
            return;
        }
        act(state, rules, Command::Defend).unwrap();
    }
    panic!("stack {id} never became active");
}

/// The order in which stacks act when everybody defends, for `turns` turns.
pub fn order(state: &mut BattleState, rules: &BattleRuleset, turns: usize) -> Vec<u8> {
    (0..turns)
        .map(|_| {
            let id = state.active().unwrap().0;
            act(state, rules, Command::Defend).unwrap();
            id
        })
        .collect()
}
