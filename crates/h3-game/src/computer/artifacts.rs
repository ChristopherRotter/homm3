//! The computer's values of artifacts (E31 T2f, AI-Q37; survey
//! `docs/rules/surveys/E31-computer-player.md` §4.1, §2.4): the effects' table the original
//! builds at its start (`ai_artifact_effects_build` 0x434100, [`TABLE`], CPU-ART-001), the 24
//! effect codes' values (`ai_artifact_effect_*` 0x432510 to 0x433580, [`Effect`], CPU-ART-002 to
//! -008), an artifact's value to a hero (`ai_hero_artifact_value` 0x4336C0, [`hero_value`],
//! CPU-ART-009), to a hero as it would wear it (`ai_artifact_value_to_hero` 0x4339E0,
//! [`value_to_hero`], CPU-ART-010), to a player (`ai_artifact_value_to_player` 0x433AA0,
//! CPU-ART-011) and the turn start's mean of every artifact (`ai_turn_start_values` 0x527960's
//! +0x164, [`mean`], CPU-STA-008).
//!
//! The artifacts are told by their ids as the original tells them: the core's handles are the
//! original's ids for the original packs' 141 artifacts (the base, AB and SoD packs in order), as
//! `skills.rs` reads the skills' (the base pack's creatures, the cloak's undead, are the
//! original's too; its schools are in the base pack's order). Code 18's spell, whose handle is
//! not the original's id (SoD appends Titan's Lightning Bolt), is the one its artifact grants.
//!
//! Not here: `ai_hero_equipped_value` 0x433C60 (the hero's value as equipped) goes with its
//! callers, the artifact exchange of increment 3 (the user at T2f, AI-Q37); the victory
//! condition's artifact (5 000 000, `AI-STUB-16`, E10); the hero's reads its owner's towns would
//! change (the Grail structures' morale and luck, the Necromancy buildings; the share capped at
//! 1.0: `AI-STUB-17`).

use alloc::vec::Vec;

use h3_battle::estimate::{CasterRecord, change_value, luck_values, morale_values};
use h3_content::{
    Affects, ArtifactTypeId, CreatureFlag, CreatureTypeId, FULL_BP, HeroEffect, SchoolId, SkillId,
    SpellGrant, SpellId,
};
use h3_contracts::{BattleEffect, FortLevel, PlayerId, TownId};
use h3_heroes::{ArtifactInstance, BattleBonuses, Position, derive_battle_hero};
use h3_kernel::arith::{SINGLE_BITS, SignedX87, X87};
use h3_towns::{Town, TownRecord, index_creature};

use super::fight::{self, Fighter};
use super::spells::{self, SpellGains};
use super::start::Record;
use super::turn::{counted, primary_sum_of, towns_of};
use crate::{GameRuleset, GameState};

#[cfg(test)]
mod tests;

/// The original's ids of the four artifacts `ai_hero_artifact_value` values itself (its
/// switch's cases 1, 4, 5 and 6): the Spell Scroll, the Ballista, the Ammo Cart, the First Aid
/// Tent.
const SCROLL: u16 = 1;
const BALLISTA: u16 = 4;
const AMMO_CART: u16 = 5;
const TENT: u16 = 6;

/// The original's ids of the skills the values read: Wisdom (+0xD0), Necromancy (+0xD5),
/// Artillery (+0xDD).
const WISDOM: SkillId = SkillId::from_raw(7);
const NECROMANCY: SkillId = SkillId::from_raw(12);
const ARTILLERY: SkillId = SkillId::from_raw(20);

/// The Cloak of the Undead King's creatures by their original ids (0x4333A0: the Skeleton 56 its
/// base, the Walking Dead 58, the Wight 60 and the Lich 64 by the Necromancy level 1, 2 and 3).
const UNDEAD: [u16; 4] = [56, 58, 60, 64];

/// The artifacts the turn start's mean counts (0x5279C3 to 0x527A08: 7 to 143, less those the
/// setup disables, 141 to 143 by `artifact_table_load` 0x44CD50's `artifacts_disabled` 0x63B98C).
const MEAN_FROM: usize = 7;
const MEAN_TO: usize = 140;

/// The least a player values an artifact (0x433AC7: 10).
const LEAST_TO_PLAYER: i32 = 10;

/// The f32 the FPU stores for 0 / 0, its integer indefinite's float (the mean of no artifact).
const INDEFINITE: u32 = 0xFFC0_0000;

/// A magic school as the original's masks name it (the spell table's +0x1C: 1 Air, 2 Fire,
/// 4 Water, 8 Earth), by its handle in the base pack's order (Earth 0, Water 1, Fire 2, Air 3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum School {
    Air,
    Fire,
    Water,
    Earth,
}

impl School {
    /// The school's handle (the base pack's order).
    const fn id(self) -> SchoolId {
        SchoolId::from_raw(match self {
            Self::Earth => 0,
            Self::Water => 1,
            Self::Fire => 2,
            Self::Air => 3,
        })
    }
}

/// One effect of the computer's artifact effects table (`ai_artifact_effects` 0x692E18, 144
/// vectors of objects `ai_artifact_effect`: +4 p, +8 q), by its code 0 to 23 and the method its
/// vtable's slot 1 names (0x63B6B0). Each is valued for a hero ([`Effect::value`]) as
/// `ai_hero_artifact_value` 0x4336C0 calls it: with `worn` (its dl: set for an artifact the hero
/// wears, clear for a candidate) and `counted` (its third argument: set where the caller's own
/// value already counts the primary skills, the best spell, morale and luck). `A` below is the
/// army's value ([`Valued::army`]); every product and sum is 32 bits, every division truncates
/// toward 0.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Effect {
    /// Code 0 (`ai_artifact_effect_attack_defense` 0x4325A0): p points of Attack and Defense.
    AttackDefense(i32),
    /// Code 1 (`…_spell_power` 0x4325E0): p Spell Power.
    SpellPower(i32),
    /// Code 2 (`…_knowledge` 0x432610): p Knowledge.
    Knowledge(i32),
    /// Code 3 (`…_morale` 0x432780): p morale.
    Morale(i32),
    /// Code 4 (`…_luck` 0x4327F0): p luck.
    Luck(i32),
    /// Code 5 (`…_scouting` 0x432510): p, the day's movement's share in hundredths.
    Scouting(i32),
    /// Code 6 (`…_necromancy` 0x432640): p hundredths of Necromancy.
    Necromancy(i32),
    /// Code 7 (`…_army_share` 0x432560): p hundredths of the army.
    ArmyShare(i32),
    /// Code 8 (`…_movement` 0x4326E0): p hundredths of movement.
    Movement(i32),
    /// Code 9 (`…_mana` 0x432720): p hundredths of the army for its mana.
    Mana(i32),
    /// Code 10 (`…_duration` 0x432860): p rounds of the spells.
    Duration(i32),
    /// Code 11 (`…_school_power` 0x432890): p hundredths more Spell Power for the school's
    /// damage spells (the orbs, p 100).
    SchoolPower(i32, School),
    /// Code 12 (`…_school_spells` 0x432C20): the school's spells (the tomes).
    SchoolSpells(School),
    /// Code 13 (`…_spell_block` 0x432A50): no casting (p 0, the Orb of Inhibition) or p hundredths
    /// of the enemy's spells blocked (the Recanter's Cloak, 3).
    SpellBlock(i32),
    /// Code 14 (`…_enemy_morale` 0x432B20): the enemy's morale (the Spirit of Oppression).
    EnemyMorale,
    /// Code 15 (`…_enemy_luck` 0x432BA0): the enemy's luck (the Hourglass of the Evil Hour).
    EnemyLuck,
    /// Code 16 (`…_resource` 0x432D20): p of resource q a day (by the original's index).
    Resource(i32, u8),
    /// Code 17 (`…_growth` 0x432D70): q creatures more a week of the level p (from 0).
    Growth(u8, i32),
    /// Code 18 (`…_spell` 0x432F90): the spell p (Titan's Thunder's bolt, 57), the one its
    /// artifact's own `spells` effect grants.
    Spell(u16),
    /// Code 19 (`…_shooters` 0x4330B0): p hundredths of the shooters' value.
    Shooters(i32),
    /// Code 20 (`…_alliance` 0x433130): the alliance, p fixed by the build at 8.
    Alliance(i32),
    /// Code 21 (`…_undead_king` 0x4333A0): the Cloak of the Undead King, p fixed by the build at
    /// 30.
    UndeadKing(i32),
    /// Code 22 (`…_living` 0x433520): the living creatures (the Elixir of Life).
    Living,
    /// Code 23 (`…_town_growth` 0x433580): every town's growth (the Statue of Legion).
    TownGrowth,
}

use Effect::{
    Alliance, ArmyShare, AttackDefense, Duration, EnemyLuck, EnemyMorale, Growth, Knowledge,
    Living, Luck, Mana, Morale, Movement, Necromancy, Resource, SchoolPower, SchoolSpells,
    Scouting, Shooters, Spell, SpellBlock, SpellPower, TownGrowth, UndeadKing,
};

/// The computer's artifact effects by the original's artifact id (CPU-ART-001): what
/// `ai_artifact_effects_build` 0x434100 builds from `ai_artifact_effect_list` 0x63AC7C (each
/// artifact once, its effects in the list's order; the codes 11's school first, then p; the codes
/// 16's and 17's p, then q; 20's p 8 and 21's p 30 set by the build), read whole at E31 T2f.
/// 135 artifacts have effects; the Spell Book, the Scroll, the Catapult, the three machines and
/// 141 to 143 none (the scroll and the machines are valued by [`hero_value`] itself).
// rule: CPU-ART-001
pub(super) const TABLE: [&[Effect]; 144] = [
    &[], // 0 Spell Book
    &[], // 1 Spell Scroll
    &[
        Resource(5000, 6),
        Growth(0, 5),
        Growth(1, 4),
        Growth(2, 3),
        Growth(3, 2),
        Growth(4, 1),
        Growth(5, 1),
        Growth(6, 1),
    ], // 2 The Grail
    &[], // 3 Catapult
    &[], // 4 Ballista
    &[], // 5 Ammo Cart
    &[], // 6 First Aid Tent
    &[AttackDefense(1)], // 7 Centaurs Axe
    &[AttackDefense(2)], // 8 Blackshard of the Dead Knight
    &[AttackDefense(3)], // 9 Greater Gnoll's Flail
    &[AttackDefense(4)], // 10 Ogre's Club of Havoc
    &[AttackDefense(5)], // 11 Sword of Hellfire
    &[AttackDefense(9)], // 12 Titan's Gladius
    &[AttackDefense(1)], // 13 Shield of the Dwarven Lords
    &[AttackDefense(2)], // 14 Shield of the Yawning Dead
    &[AttackDefense(3)], // 15 Buckler of the Gnoll King
    &[AttackDefense(4)], // 16 Targ of the Rampaging Ogre
    &[AttackDefense(5)], // 17 Shield of the Damned
    &[AttackDefense(9)], // 18 Sentinel's Shield
    &[Knowledge(1)], // 19 Helm of the Alabaster Unicorn
    &[Knowledge(2)], // 20 Skull Helmet
    &[Knowledge(3)], // 21 Helm of Chaos
    &[Knowledge(4)], // 22 Crown of the Supreme Magi
    &[Knowledge(5)], // 23 Hellstorm Helmet
    &[Knowledge(12), SpellPower(-3)], // 24 Thunder Helmet
    &[SpellPower(1)], // 25 Breastplate of Petrified Wood
    &[SpellPower(2)], // 26 Rib Cage
    &[SpellPower(3)], // 27 Scales of the Greater Basilisk
    &[SpellPower(4)], // 28 Tunic of the Cyclops King
    &[SpellPower(5)], // 29 Breastplate of Brimstone
    &[SpellPower(12), Knowledge(-3)], // 30 Titan's Cuirass
    &[AttackDefense(2), SpellPower(1), Knowledge(1)], // 31 Armor of Wonder
    &[AttackDefense(4), SpellPower(2), Knowledge(2)], // 32 Sandals of the Saint
    &[AttackDefense(6), SpellPower(3), Knowledge(3)], // 33 Celestial Necklace of Bliss
    &[AttackDefense(8), SpellPower(4), Knowledge(4)], // 34 Lion's Shield of Courage
    &[AttackDefense(10), SpellPower(5), Knowledge(5)], // 35 Sword of Judgement
    &[AttackDefense(12), SpellPower(6), Knowledge(6)], // 36 Helm of Heavenly Enlightenment
    &[AttackDefense(2)], // 37 Quiet Eye of the Dragon
    &[AttackDefense(4)], // 38 Red Dragon Flame Tongue
    &[AttackDefense(6)], // 39 Dragon Scale Shield
    &[AttackDefense(8)], // 40 Dragon Scale Armor
    &[SpellPower(1), Knowledge(1)], // 41 Dragonbone Greaves
    &[SpellPower(2), Knowledge(2)], // 42 Dragon Wing Tabard
    &[SpellPower(3), Knowledge(3)], // 43 Necklace of Dragonteeth
    &[SpellPower(4), Knowledge(4)], // 44 Crown of Dragontooth
    &[Luck(1), Morale(1)], // 45 Still Eye of the Dragon
    &[Luck(1)], // 46 Clover of Fortune
    &[Luck(1)], // 47 Cards of Prophecy
    &[Luck(1)], // 48 Ladybird of Luck
    &[Morale(1)], // 49 Badge of Courage
    &[Morale(1)], // 50 Crest of Valor
    &[Morale(1)], // 51 Glyph of Gallantry
    &[Scouting(1)], // 52 Speculum
    &[Scouting(1)], // 53 Spyglass
    &[Necromancy(5)], // 54 Amulet of the Undertaker
    &[Necromancy(10)], // 55 Vampire's Cowl
    &[Necromancy(15)], // 56 Dead Man's Boots
    &[ArmyShare(1)], // 57 Garniture of Interference
    &[ArmyShare(3)], // 58 Surcoat of Counterpoise
    &[ArmyShare(5)], // 59 Boots of Polarity
    &[Shooters(2)], // 60 Bow of Elven Cherrywood
    &[Shooters(5)], // 61 Bowstring of the Unicorn's Mane
    &[Shooters(7)], // 62 Angel Feather Arrows
    &[ArmyShare(1)], // 63 Bird of Perception
    &[ArmyShare(2)], // 64 Stoic Watchman
    &[ArmyShare(3)], // 65 Emblem of Cognizance
    &[ArmyShare(1)], // 66 Statesman's Medal
    &[ArmyShare(2)], // 67 Diplomat's Ring
    &[ArmyShare(3)], // 68 Ambassador's Sash
    &[ArmyShare(5)], // 69 Ring of the Wayfarer
    &[Movement(12)], // 70 Equestrian's Gloves
    &[Movement(25)], // 71 Necklace of Ocean Guidance
    &[Movement(50)], // 72 Angel Wings
    &[Mana(2)], // 73 Charm of Mana
    &[Mana(4)], // 74 Talisman of Mana
    &[Mana(6)], // 75 Mystic Orb of Mana
    &[Duration(1)], // 76 Collar of Conjuring
    &[Duration(2)], // 77 Ring of Conjuring
    &[Duration(3)], // 78 Cape of Conjuring
    &[SchoolPower(100, School::Air)], // 79 Orb of the Firmament
    &[SchoolPower(100, School::Earth)], // 80 Orb of Silt
    &[SchoolPower(100, School::Fire)], // 81 Orb of Tempestuous Fire
    &[SchoolPower(100, School::Water)], // 82 Orb of Driving Rain
    &[SpellBlock(3)], // 83 Recanter's Cloak
    &[EnemyMorale], // 84 Spirit of Oppression
    &[EnemyLuck], // 85 Hourglass of the Evil Hour
    &[SchoolSpells(School::Fire)], // 86 Tome of Fire Magic
    &[SchoolSpells(School::Air)], // 87 Tome of Air Magic
    &[SchoolSpells(School::Water)], // 88 Tome of Water Magic
    &[SchoolSpells(School::Earth)], // 89 Tome of Earth Magic
    &[ArmyShare(5)], // 90 Boots of Levitation
    &[Shooters(5)], // 91 Golden Bow
    &[ArmyShare(2)], // 92 Sphere of Permanence
    &[ArmyShare(10)], // 93 Orb of Vulnerability
    &[ArmyShare(1)], // 94 Ring of Vitality
    &[ArmyShare(1)], // 95 Ring of Life
    &[ArmyShare(2)], // 96 Vial of Lifeblood
    &[ArmyShare(5)], // 97 Necklace of Swiftness
    &[Movement(25)], // 98 Boots of Speed
    &[ArmyShare(10)], // 99 Cape of Velocity
    &[ArmyShare(1)], // 100 Pendant of Dispassion
    &[ArmyShare(10)], // 101 Pendant of Second Sight
    &[ArmyShare(1)], // 102 Pendant of Holiness
    &[ArmyShare(5)], // 103 Pendant of Life
    &[ArmyShare(5)], // 104 Pendant of Death
    &[ArmyShare(1)], // 105 Pendant of Free Will
    &[ArmyShare(10)], // 106 Pendant of Negativity
    &[ArmyShare(1)], // 107 Pendant of Total Recall
    &[ArmyShare(10)], // 108 Pendant of Courage
    &[Resource(1, 4)], // 109 Everflowing Crystal Cloak
    &[Resource(1, 5)], // 110 Ring of Infinite Gems
    &[Resource(1, 1)], // 111 Everpouring Vial of Mercury
    &[Resource(1, 2)], // 112 Inexhaustible Cart of Ore
    &[Resource(1, 3)], // 113 Eversmoking Ring of Sulfur
    &[Resource(1, 0)], // 114 Inexhaustible Cart of Lumber
    &[Resource(1000, 6)], // 115 Endless Sack of Gold
    &[Resource(750, 6)], // 116 Endless Bag of Gold
    &[Resource(500, 6)], // 117 Endless Purse of Gold
    &[Growth(1, 5)], // 118 Legs of Legion
    &[Growth(2, 4)], // 119 Loins of Legion
    &[Growth(3, 3)], // 120 Torso of Legion
    &[Growth(4, 2)], // 121 Arms of Legion
    &[Growth(5, 1)], // 122 Head of Legion
    &[Movement(5)], // 123 Sea Captain's Hat
    &[Mana(20)], // 124 Spellbinder's Hat
    &[ArmyShare(5)], // 125 Shackles of War
    &[SpellBlock(0)], // 126 Orb of Inhibition
    &[AttackDefense(10)], // 127 Vial of Dragon Blood
    &[AttackDefense(6), SpellPower(3), Knowledge(6)], // 128 Armageddon's Blade
    &[Alliance(8)], // 129 Angelic Alliance
    &[UndeadKing(30)], // 130 Cloak of the Undead King
    &[Living], // 131 Elixir of Life
    &[ArmyShare(15), AttackDefense(6), Shooters(10)], // 132 Armor of the Damned
    &[TownGrowth], // 133 Statue of Legion
    &[
        AttackDefense(12),
        SpellPower(6),
        Knowledge(6),
        ArmyShare(10),
    ], // 134 Power of the Dragon Father
    &[Spell(57)], // 135 Titan's Thunder
    &[Movement(12)], // 136 Admiral's Hat
    &[Shooters(7)], // 137 Bow of the Sharpshooter
    &[Mana(10)], // 138 Wizard's Well
    &[Duration(50)], // 139 Ring of the Magi
    &[
        Resource(4, 4),
        Resource(4, 5),
        Resource(4, 1),
        Resource(4, 3),
    ], // 140 Cornucopia
    &[], // 141
    &[], // 142
    &[], // 143
];

/// What the artifact values read of a hero, read once for all its values (the original reads
/// each from the hero, its record and the game at every call; nothing between two calls changes
/// them): the hero, its army's value, its spell gains and its owner's resource weights from the
/// turn start's record, its caster record, its available spells, its own morale and luck.
pub(super) struct Valued<'a> {
    state: &'a GameState,
    rules: &'a GameRuleset,
    hero: Fighter<'a>,
    /// The army's value (`army_value` 0x44AC80 of +0x91, 32 bits).
    army: i32,
    /// Its spell gains as the turn start's record holds them (+0x47E Spell Power, +0x482 the
    /// rounds, +0x486 Knowledge; CPU-STA-007); all 0 for a hero the record does not list.
    gains: SpellGains,
    /// Its owner's resource weights (`ai_player_records` +0x60, CPU-STA-003), by resource.
    weights: &'a [SignedX87],
    /// Its caster record (`ai_caster_record` 0x526D40, which codes 11, 12, 18 and the scroll
    /// build at each call; CPU-SPL-001).
    caster: CasterRecord,
    /// Its available spells (+0x430: the learned, its scrolls' and its artifacts' spells).
    available: Vec<SpellId>,
    /// Its own morale (`hero_morale` 0x4E39B0 without the clamp).
    morale: i32,
    /// Its own luck (`hero_luck` 0x4E36C0 without the clamp).
    luck: i32,
}

impl<'a> Valued<'a> {
    /// The reads of `hero`, its spell gains from `record` and its owner's resource weights
    /// `weights` (`Record::weights`).
    pub(super) fn new(
        state: &'a GameState,
        rules: &'a GameRuleset,
        (record, weights): (&Record, &'a [SignedX87]),
        hero: Fighter<'a>,
    ) -> Self {
        let id = hero.on_map.id;
        let gains = record
            .heroes
            .binary_search_by_key(&id, |&(h, _)| h)
            .map_or(SpellGains::default(), |at| record.heroes[at].1.spells);
        let (morale, luck) = own_morale_and_luck(state, rules, hero);
        Self {
            state,
            rules,
            hero,
            army: fight::value_of(rules, &hero.on_map.army),
            gains,
            weights,
            caster: spells::record(state, rules, hero),
            available: h3_heroes::available_spells(hero.details, rules.core()).unwrap_or_default(),
            morale,
            luck,
        }
    }

    /// The level of the hero's skill `skill`, 0 without it.
    fn skill(&self, skill: SkillId) -> u8 {
        self.hero
            .details
            .skills
            .iter()
            .find(|&&(id, _)| id == skill)
            .map_or(0, |&(_, level)| level)
    }

    /// Its Spell Power as the codes hold it (+0x478 as a signed byte, 1 to 99).
    fn power(&self) -> i32 {
        let [_, _, power, _] = h3_heroes::primary_skills(self.hero.details, self.rules.core());
        i32::from(counted(power, 1))
    }

    /// Whether it has learned `spell` (+0x3EA).
    fn learned(&self, spell: SpellId) -> bool {
        self.hero.details.spells.contains(&spell)
    }

    /// Whether `spell` is among its available ones (+0x430).
    fn has(&self, spell: SpellId) -> bool {
        self.available.contains(&spell)
    }

    /// The AI value and count of each stack of its army whose creature has `flag`.
    fn stacks_with(&self, flag: CreatureFlag) -> impl Iterator<Item = (i32, i32)> + '_ {
        let core = self.rules.core();
        self.hero.on_map.army.iter().filter_map(move |stack| {
            core.creature(stack.creature)
                .filter(|info| info.flags.contains(flag))
                .map(|info| (word(info.ai_value), word(stack.count)))
        })
    }
}

/// A 32-bit unsigned value as the original's signed register holds it.
const fn word(value: u32) -> i32 {
    i32::from_ne_bytes(value.to_ne_bytes())
}

/// Whether a morale or luck effect on `affects` reaches the hero's own stacks.
fn own_side(affects: Affects) -> bool {
    affects != Affects::Enemy
}

/// The hero's own morale and luck as `hero_morale` 0x4E39B0 and `hero_luck` 0x4E36C0 compute them
/// without their clamp (CPU-ART-003): the morale and luck its battle hero carries to its own side
/// (`h3_heroes::derive_battle_hero` with its battle bonuses on the map: its skills, artifacts,
/// specialty and visits), summed; its luck 0 when it carries a luck cap (the Hourglass of the
/// Evil Hour: the original's test of artifact 85). Neither cap of the enemy's is read. The
/// Grail structures' +2 are not in the engine (`AI-STUB-17`).
// rule: CPU-ART-003
fn own_morale_and_luck(state: &GameState, rules: &GameRuleset, hero: Fighter<'_>) -> (i32, i32) {
    // stub: AI-STUB-17
    let marks = state.adventure().visit_marks(hero.on_map.id);
    let bonuses: BattleBonuses<'_> = marks.bonuses().battle();
    derive_battle_hero(hero.details, bonuses, rules.core()).map_or((0, 0), |battle| {
        let (mut morale, mut luck, mut capped) = (0_i32, 0_i32, false);
        for effect in &battle.effects {
            match *effect {
                BattleEffect::Morale {
                    amount, affects, ..
                } if own_side(affects) => morale += i32::from(amount),
                BattleEffect::Luck {
                    amount, affects, ..
                } if own_side(affects) => luck += i32::from(amount),
                BattleEffect::LuckAtMost { affects, .. } if own_side(affects) => capped = true,
                _ => {}
            }
        }
        (morale, if capped { 0 } else { luck })
    })
}

/// The slip of `ai_morale_change_value` 0x435850 (survey §5.5 #3: a rise from below 0 valued as
/// crossing 0), ported as read until AI-QUIRK-1 (increment 4).
const SLIP: bool = true;

/// The hero's Necromancy share's room (0x432640, 0x4333A0): `h = trunc((1.0 − s) × 100.0)` of its
/// share `s` (`hero_necromancy_power` 0x4E3CD0 in single precision: `h3_heroes::necromancy`,
/// capped at 1.0 and 0 with nothing to raise, `AI-STUB-17`), then `p + min(h, 0)` for a worn artifact, `min(p, h)` for a
/// candidate.
fn room(v: &Valued<'_>, p: i32, worn: bool) -> i32 {
    let share = h3_heroes::necromancy(v.hero.details, v.rules.core()).map_or(X87::ZERO, |n| {
        n.share
            .single
            .map(core::num::NonZeroU32::get)
            .map(X87::from_f32_bits)
            .unwrap_or(X87::ratio(n.share.bp, FULL_BP.into(), SINGLE_BITS).unwrap_or(X87::ZERO))
    });
    let h = SignedX87::ONE
        .minus(share.into())
        .times(SignedX87::int(100))
        .ftol();
    if worn {
        p.wrapping_add(h.min(0))
    } else {
        p.min(h)
    }
}

/// Code 6's value of `p` hundredths (0x432640's tail, 0x4333A0's): `A × n / 250` of the room `n`
/// ([`room`]), 0 when `n` is not above 0.
fn raised(v: &Valued<'_>, p: i32, worn: bool) -> i32 {
    let n = room(v, p, worn);
    if n > 0 {
        v.army.wrapping_mul(n) / 250
    } else {
        0
    }
}

/// A morale or luck code's product (0x432780, 0x432B20): `trunc(x × A)` in double precision.
fn of_army(x: SignedX87, army: i32) -> i32 {
    x.times(SignedX87::int(army.into())).ftol()
}

impl Effect {
    /// The effect's value to the hero of `v` (its method, slot 1 of its vtable), `artifact` the
    /// artifact whose effect it is, `worn` and `counted` as [`Effect`] says (CPU-ART-002 to -008).
    // rule: CPU-ART-002, CPU-ART-003, CPU-ART-004, CPU-ART-005
    pub(super) fn value(
        self,
        v: &Valued<'_>,
        artifact: ArtifactTypeId,
        worn: bool,
        counted: bool,
    ) -> i32 {
        let a = v.army;
        match self {
            // Codes 0, 1, 2 and 10: nothing where the caller counts the hero's own values.
            AttackDefense(_) | SpellPower(_) | Knowledge(_) | Duration(_) | Morale(_) | Luck(_)
                if counted =>
            {
                0
            }
            AttackDefense(p) => a.wrapping_mul(p) / 40,
            SpellPower(p) => v.gains.power.wrapping_mul(p),
            Knowledge(p) => v.gains.knowledge.wrapping_mul(p),
            Duration(p) => v.gains.rounds.wrapping_mul(p),
            // Codes 3 and 4: the change by p from its own value, less p for a worn one.
            Morale(p) => {
                let m = if worn {
                    v.morale.wrapping_sub(p)
                } else {
                    v.morale
                };
                of_army(change_value(m, p, morale_values(), SLIP), a)
            }
            Luck(p) => {
                let l = if worn { v.luck.wrapping_sub(p) } else { v.luck };
                of_army(change_value(l, p, luck_values(), SLIP), a)
            }
            Scouting(p) => word(v.hero.on_map.max_movement).wrapping_mul(p) / 100,
            Necromancy(p) => {
                if v.skill(NECROMANCY) == 0 {
                    0
                } else {
                    raised(v, p, worn)
                }
            }
            ArmyShare(p) => a.wrapping_mul(p) / 100,
            Movement(p) => a.wrapping_add(2500).wrapping_mul(p) / 100,
            Mana(p) => {
                if v.gains.power == 0 || v.skill(WISDOM) == 0 {
                    0
                } else {
                    a.wrapping_mul(p) / 100
                }
            }
            SpellBlock(p) => spell_block(v, p, worn, counted),
            EnemyMorale => enemy(v, (morale_values(), v.morale), counted),
            EnemyLuck => enemy(v, (luck_values(), v.luck), counted),
            Shooters(p) => {
                let sum = v
                    .stacks_with(CreatureFlag::Shooter)
                    .fold(0_i32, |s, (ai, n)| s.wrapping_add(ai.wrapping_mul(n)));
                sum.wrapping_mul(p) / 100
            }
            Living => {
                v.stacks_with(CreatureFlag::Alive)
                    .fold(0_i32, |s, (ai, n)| s.wrapping_add(ai.wrapping_mul(n)))
                    / 8
            }
            UndeadKing(p) => undead_king(v, p, worn),
            other => other.of_the_game(v, artifact, worn, counted),
        }
    }

    /// The values that read the game beyond the hero (CPU-ART-006 to -008): its spells, its
    /// owner's weights, towns and heroes.
    // rule: CPU-ART-006, CPU-ART-007, CPU-ART-008
    fn of_the_game(
        self,
        v: &Valued<'_>,
        artifact: ArtifactTypeId,
        worn: bool,
        counted: bool,
    ) -> i32 {
        match self {
            SchoolPower(p, school) => school_power(v, (p, school), worn, counted),
            SchoolSpells(school) => school_spells(v, school, worn, counted),
            Spell(_) => {
                granted(v.rules, artifact).map_or(0, |spell| spell_value(v, spell, worn, counted))
            }
            Resource(p, q) => {
                let weight = v
                    .weights
                    .get(usize::from(q))
                    .copied()
                    .unwrap_or(SignedX87::ZERO);
                SignedX87::int(p.into())
                    .times(weight)
                    .times(SignedX87::int(3))
                    .ftol()
            }
            Growth(level, q) => growth(v, (level, q), counted),
            Alliance(p) => alliance(v, artifact, p, counted),
            _ => town_growth(v),
        }
    }
}

/// Code 13 (`ai_artifact_effect_spell_block` 0x432A50; CPU-ART-004): `v = A / 5` for p 0 (the
/// Orb of Inhibition), else `A / 8`; for a worn one the caller does not count, less 50 (p 0) or
/// 25 per Spell Power point (held 1 to 99; §4.1 (h): a candidate keeps its whole value).
fn spell_block(v: &Valued<'_>, p: i32, worn: bool, counted: bool) -> i32 {
    let value = if p == 0 { v.army / 5 } else { v.army / 8 };
    if counted || !worn {
        return value;
    }
    let per: i32 = if p == 0 { 50 } else { 25 };
    value.wrapping_sub(per.wrapping_mul(v.power()))
}

/// Codes 14 and 15 (`ai_artifact_effect_enemy_morale` 0x432B20, `…_enemy_luck` 0x432BA0;
/// CPU-ART-003): `v = trunc(c(0, 2) × A)`, the enemy's +2 assumed (no enemy read), with `c` the
/// change value of `values`; where the caller does not count and the hero's own value `own`
/// held to ±3 is above 0, `trunc(c(m, −m) × A + v)` of that value `m` (its own lost: the luck's
/// read 0 with the Hourglass worn, §4.1 (i)).
fn enemy(v: &Valued<'_>, (values, own): ((X87, X87), i32), counted: bool) -> i32 {
    let a = SignedX87::int(v.army.into());
    let base = change_value(0, 2, values, SLIP).times(a).ftol();
    let m = own.clamp(-3, 3);
    if counted || m <= 0 {
        return base;
    }
    change_value(m, -m, values, SLIP)
        .times(a)
        .plus(SignedX87::int(base.into()))
        .ftol()
}

/// Code 21 (`ai_artifact_effect_undead_king` 0x4333A0; CPU-ART-005): without Necromancy code 6's
/// value with p ([`raised`], the skill not asked); with it that value `w` times
/// `r = f32((f32(AI c) − f32(AI s)) / f32(AI s))` of the Skeleton `s` and the creature `c` of its
/// level (the Walking Dead, the Wight, the Lich for 1, 2, 3: [`UNDEAD`]; a level past 3 reads the
/// Lich, the original's an argument, unreachable: §4.1 (l)), `trunc(f32(w) × r)`; the FPU's
/// indefinite (`i32::MIN`) when the Skeleton is worth 0.
fn undead_king(v: &Valued<'_>, p: i32, worn: bool) -> i32 {
    let level = usize::from(v.skill(NECROMANCY));
    if level == 0 {
        return raised(v, p, worn);
    }
    let ai = |raw| {
        SignedX87::int(ai_value(v.rules, CreatureTypeId::from_raw(raw)).into()).round(SINGLE_BITS)
    };
    let base = ai(UNDEAD[0]);
    let creature = ai(UNDEAD[level.min(3)]);
    let w = SignedX87::int(raised(v, p, worn).into()).round(SINGLE_BITS);
    creature
        .minus(base)
        .over(base)
        .map_or(i32::MIN, |r| w.times(r.round(SINGLE_BITS)).ftol())
}

/// Whether `spell` is of the school `school`.
fn of_school(rules: &GameRuleset, spell: SpellId, school: SchoolId) -> bool {
    rules
        .core()
        .spell(spell)
        .is_some_and(|s| s.schools.contains(&school))
}

/// The spell `artifact`'s own `spells` effect grants by name (Titan's Thunder's bolt: code 18's
/// p, the original's 57, whose handle is not the original's id); none for another artifact.
fn granted(rules: &GameRuleset, artifact: ArtifactTypeId) -> Option<SpellId> {
    let effects = rules
        .core()
        .artifact(artifact)
        .map_or(&[][..], |a| &a.effects[..]);
    effects.iter().find_map(|e| match *e {
        HeroEffect::Spells {
            grant: SpellGrant::Spell(spell),
        } => Some(spell),
        _ => None,
    })
}

/// Code 11 (`ai_artifact_effect_school_power` 0x432890; CPU-ART-006): 0 where counted or without
/// power (CPU-SPL-001); else with `s` its Spell Power (1 to 99) the powers `(100 s / (100 + p),
/// s)` for a worn orb, `(s, s (100 + p) / 100)` for a candidate (§4.1 (g)), and the largest, from
/// 0, of the second power's value less the first's (the caster record's power changed,
/// CPU-SPL-002) over its available damage spells (the AI's damage kinds, the original's flag
/// 0x200) of the school. Every orb's p is 100: the least for a negative p (0x4329DB) never runs.
fn school_power(v: &Valued<'_>, (p, school): (i32, School), worn: bool, counted: bool) -> i32 {
    if counted || v.caster.power <= 0 {
        return 0;
    }
    let s = v.power();
    let bonus = p.wrapping_add(100);
    let (low, high) = if worn {
        (s.wrapping_mul(100).checked_div(bonus).unwrap_or(0), s)
    } else {
        (s, s.wrapping_mul(bonus) / 100)
    };
    let (rules, battle) = (v.rules, v.rules.battle());
    let school = school.id();
    let mut record = v.caster.clone();
    v.available
        .iter()
        .copied()
        .filter(|&spell| of_school(rules, spell, school))
        .filter(|&spell| {
            battle
                .spell(spell)
                .is_some_and(|s| spells::damage_kinds(s.ai.kinds))
        })
        .fold(0, |best, spell| {
            record.power = low;
            let before = record.value(battle, spell);
            record.power = high;
            best.max(record.value(battle, spell).wrapping_sub(before))
        })
}

/// Code 12 (`ai_artifact_effect_school_spells` 0x432C20; CPU-ART-006): 0 where counted or without
/// power; else the largest, from 0, of the values (CPU-SPL-002) of the school's spells the hero
/// has not learned and, for a candidate, has not available (no ban read). The original's loop
/// stops before the creatures' abilities (70 on), which the engine's content gives no school.
fn school_spells(v: &Valued<'_>, school: School, worn: bool, counted: bool) -> i32 {
    if counted || v.caster.power <= 0 {
        return 0;
    }
    let (rules, battle) = (v.rules, v.rules.battle());
    let school = school.id();
    (0..=u16::MAX)
        .take(rules.core().spells().len())
        .map(SpellId::from_raw)
        .filter(|&spell| of_school(rules, spell, school))
        .filter(|&spell| !v.learned(spell) && (worn || !v.has(spell)))
        .fold(0, |best, spell| best.max(v.caster.value(battle, spell)))
}

/// Code 18 (`ai_artifact_effect_spell` 0x432F90; CPU-ART-006) of the spell its artifact grants
/// ([`granted`]): 0 where counted, for a spell it has
/// learned, a candidate's spell it has available, or without power (§4.1 (e): Titan's Thunder's
/// bolt is 0 to a hero without a book though the artifact gives one); else the spell's value.
fn spell_value(v: &Valued<'_>, spell: SpellId, worn: bool, counted: bool) -> i32 {
    if counted || v.learned(spell) || (!worn && v.has(spell)) || v.caster.power <= 0 {
        return 0;
    }
    v.caster.value(v.rules.battle(), spell)
}

/// The creature of a town's level `level` (from 0) when its dwelling is ever built, its upgrade's
/// when the upgraded one is too (the building bits 30 + level and 37 + level); none else.
fn dwelling_creature(town: &Town, record: &TownRecord, level: u8) -> Option<CreatureTypeId> {
    let level = level.wrapping_add(1);
    let (mut base, mut upgraded) = (false, false);
    for building in town.built().iter().filter_map(|id| record.building(id)) {
        if let Some(d) = building.roles.dwelling.filter(|d| d.level == level) {
            base |= !d.upgraded;
            upgraded |= d.upgraded;
        }
    }
    let index = usize::from(level - 1) + if upgraded { MAX_LEVELS } else { 0 };
    base.then(|| index_creature(record, index)).flatten()
}

/// The creature levels of a town record's indexes (`h3_towns::CREATURE_INDEXES` is twice it).
const MAX_LEVELS: usize = h3_towns::MAX_CREATURE_LEVELS;

/// The town `town` and its faction's record, none for a town the store or the part lacks.
fn town_and_record<'a>(
    state: &'a GameState,
    rules: &'a GameRuleset,
    town: TownId,
) -> Option<(&'a Town, &'a TownRecord)> {
    let part = rules.towns();
    state
        .towns()
        .get(town)
        .and_then(|built| part.town(built.faction()).map(|record| (built, record)))
}

/// The AI value of `creature` (+0x40), 0 for one the core lacks.
fn ai_value(rules: &GameRuleset, creature: CreatureTypeId) -> i32 {
    rules
        .core()
        .creature(creature)
        .map_or(0, |c| word(c.ai_value))
}

/// Code 17 (`ai_artifact_effect_growth` 0x432D70; CPU-ART-007): where counted, on the gate of a
/// town (`town_at_entrance` 0x4BB870) with level p's dwelling, the dwelling's creature's AI value
/// (the upgrade's when built) times q if the hero is the town's garrison hero, else 1 (§4.1 (j));
/// 0 elsewhere. Not counted: the largest such product, from 0, over the owner's towns.
fn growth(v: &Valued<'_>, (level, q): (u8, i32), counted: bool) -> i32 {
    let (state, rules) = (v.state, v.rules);
    let adventure = state.adventure();
    let of = |town| {
        town_and_record(state, rules, town)
            .and_then(|(built, record)| dwelling_creature(built, record, level))
            .map(|c| ai_value(rules, c).wrapping_mul(q))
    };
    let hero = v.hero.on_map;
    if counted {
        return adventure.map().town_at(hero.pos).map_or(0, |town| {
            of(town).map_or(0, |value| {
                if adventure.town_hero(town).map(|h| h.id) == Some(hero.id) {
                    value
                } else {
                    1
                }
            })
        });
    }
    towns_of(state, hero.owner).filter_map(of).fold(0, i32::max)
}

/// Code 23 (`ai_artifact_effect_town_growth` 0x433580; CPU-ART-007): over the owner's towns and
/// their levels whose dwelling is ever built, `AI × (g + castle) / 2` of the level's creature (the
/// upgrade's when built), `g` its growth (+0x44) and `castle` the castle's part
/// (`town_castle_growth` 0x5BF570: `g` with a Castle standing, `g / 2` with a Citadel), summed.
fn town_growth(v: &Valued<'_>) -> i32 {
    let (state, rules) = (v.state, v.rules);
    let (core, part) = (rules.core(), rules.towns());
    let mut sum = 0_i32;
    let towns = towns_of(state, v.hero.on_map.owner);
    for (built, record) in towns.filter_map(|town| town_and_record(state, rules, town)) {
        let fort = h3_towns::fort_level(built, part);
        let creatures = (0..=u8::MAX)
            .take(MAX_LEVELS)
            .filter_map(|level| dwelling_creature(built, record, level));
        for creature in creatures {
            let g = core.creature(creature).map_or(0, |c| i32::from(c.growth));
            let castle = match fort {
                FortLevel::Castle => g,
                FortLevel::Citadel => g / 2,
                _ => 0,
            };
            let ai = ai_value(rules, creature);
            sum = sum.wrapping_add(ai.wrapping_mul(g.wrapping_add(castle)) / 2);
        }
    }
    sum
}

/// Code 20 (`ai_artifact_effect_alliance` 0x433130; CPU-ART-008): the AI value times the count of
/// every creature of the owner's heroes on the map and its towns' upper bars
/// (`town_garrison_army` 0x5C1460) whose faction is in the alliance (`alignment_join_towns`
/// 0x44A460: the factions of the artifact's own `alignment_group`; a neutral creature none, as
/// the RoE game's elementals), summed; `sum × 5 / 100`, plus `A × p / 40` where not counted.
fn alliance(v: &Valued<'_>, artifact: ArtifactTypeId, p: i32, counted: bool) -> i32 {
    let (state, rules) = (v.state, v.rules);
    let core = rules.core();
    let joined: Vec<_> = core
        .artifact(artifact)
        .into_iter()
        .flat_map(|a| &a.effects)
        .filter_map(|e| match e {
            HeroEffect::AlignmentGroup { factions } => Some(factions),
            _ => None,
        })
        .flatten()
        .copied()
        .collect();
    let owner = v.hero.on_map.owner;
    let adventure = state.adventure();
    let heroes = adventure.heroes().iter().filter(|h| h.owner == owner);
    let armies = heroes
        .map(|h| &h.army[..])
        .chain(adventure.upper_bars(owner));
    let sum = armies.flatten().fold(0_i32, |sum, stack| {
        core.creature(stack.creature)
            .filter(|c| c.faction.is_some_and(|f| joined.contains(&f)))
            .map_or(sum, |c| {
                sum.wrapping_add(word(c.ai_value).wrapping_mul(word(stack.count)))
            })
    });
    let own = if counted {
        0
    } else {
        v.army.wrapping_mul(p) / 40
    };
    sum.wrapping_mul(5).wrapping_div(100).wrapping_add(own)
}

/// The value of artifact `item` to the hero of `v` (`ai_hero_artifact_value` 0x4336C0;
/// CPU-ART-009), `worn` and `counted` as [`Effect`] says: the Spell Scroll its spell's gain
/// (CPU-SPL-007; 0 for a spell it has learned, or a candidate's it has available); the Ballista
/// `q s / (q + s)` with `s = s₀ + Artillery × s₀ / 2`, `s₀ = trunc(500 √(Attack + 1))` (Attack
/// held 0 to 99) and `q = A × (primary sum + 40) / 40` (32 bits: §2.4's overflow); the Ammo Cart
/// the shooters' `AI × count / 40` summed stack by stack; the First Aid Tent the largest, from 0,
/// of its stacks' one creature's AI value, scaled by `h / hp` where the tent heals less than its
/// hit points, `h = trunc((1 + First Aid's bonus) × 25.0)`; any other the sum of its effects'
/// values ([`TABLE`]) and, for a combination, of each part's (the victory condition's artifact's
/// 5 000 000 first: `AI-STUB-16`).
// rule: CPU-ART-009
pub(super) fn hero_value(v: &Valued<'_>, item: ArtifactInstance, worn: bool, counted: bool) -> i32 {
    let artifact = item.artifact;
    match artifact.raw() {
        SCROLL => item.spell.map_or(0, |spell| {
            if v.learned(spell) || (!worn && v.has(spell)) {
                0
            } else {
                spells::gain(v.rules, &v.caster, spell)
            }
        }),
        BALLISTA => ballista(v),
        AMMO_CART => v
            .stacks_with(CreatureFlag::Shooter)
            .fold(0_i32, |sum, (ai, n)| {
                sum.wrapping_add(ai.wrapping_mul(n) / 40)
            }),
        TENT => tent(v),
        _ => {
            // stub: AI-STUB-16
            let core = v.rules.core();
            let parts = core.artifact(artifact).map_or(&[][..], |a| &a.parts[..]);
            core::iter::once(artifact)
                .chain(parts.iter().copied())
                .fold(0_i32, |sum, id| {
                    let effects = TABLE.get(id.index()).copied().unwrap_or_default();
                    effects.iter().fold(sum, |sum, effect| {
                        sum.wrapping_add(effect.value(v, id, worn, counted))
                    })
                })
        }
    }
}

/// The Ballista's value ([`hero_value`]; 0x43373E to 0x4337D1).
fn ballista(v: &Valued<'_>) -> i32 {
    let (details, core) = (v.hero.details, v.rules.core());
    let (attack, _) = h3_heroes::attack_defense(details, core);
    let root = X87::int(u64::from(counted(attack, 0)) + 1).sqrt();
    let s0 = SignedX87::from(root.times(X87::int(500))).ftol();
    let s = s0.wrapping_add(i32::from(v.skill(ARTILLERY)).wrapping_mul(s0) / 2);
    let sum = i32::from(primary_sum_of(details, core));
    let q = v.army.wrapping_mul(sum.wrapping_add(40)) / 40;
    q.wrapping_mul(s)
        .checked_div(q.wrapping_add(s))
        .unwrap_or(0)
}

/// The First Aid Tent's value ([`hero_value`]; 0x433837 to 0x4338C7, `hero_first_aid` 0x4E4920:
/// 1 plus the skill's bonus, in single precision as its battle hero's tent control holds it).
fn tent(v: &Valued<'_>) -> i32 {
    let (battle_hero, _) = fight::battle_hero(v.rules, v.hero);
    let bonus = battle_hero.war_machines.tent.map_or(X87::ZERO, |tent| {
        let rate = tent.heal_bonus;
        rate.single
            .map(core::num::NonZeroU32::get)
            .map(X87::from_f32_bits)
            .unwrap_or(X87::ratio(rate.bp, FULL_BP.into(), SINGLE_BITS).unwrap_or(X87::ZERO))
    });
    let heals = word(u32::try_from(X87::ONE.plus(bonus).times(X87::int(25)).trunc()).unwrap_or(0));
    let battle = v.rules.battle();
    v.hero.on_map.army.iter().fold(0, |best, stack| {
        let ai = ai_value(v.rules, stack.creature);
        let hp = battle
            .creature(stack.creature)
            .map_or(0, |c| word(c.hit_points));
        let healed = if heals >= hp {
            ai
        } else {
            ai.wrapping_mul(heals) / hp
        };
        best.max(healed)
    })
}

/// The worn artifact a candidate `artifact` would replace at position `at`
/// (`hero_can_replace_artifact` 0x4E2840 there): the position holds one, and with it taken off
/// the candidate fits there (`h3_heroes::takes`: the position's slot, the combinations' locks).
fn replaces(
    worn: &[Option<ArtifactInstance>],
    rules: &GameRuleset,
    artifact: ArtifactTypeId,
    at: usize,
) -> Option<ArtifactInstance> {
    let item = worn.get(at).copied().flatten()?;
    let mut without = worn.to_vec();
    without[at] = None;
    h3_heroes::takes(&without, rules.core(), artifact, Position(at)).then_some(item)
}

/// The value of artifact `item` to the hero of `v` as it would wear it (`ai_artifact_value_to_hero`
/// 0x4339E0; CPU-ART-010), `counted` as [`Effect`] says: its value as a candidate
/// ([`hero_value`]), at least 0, when a position takes it (`h3_heroes::first_position`); else
/// that value less the worn value of the artifact at the last position it would replace
/// ([`replaces`]; 0 for none), at least 0.
// rule: CPU-ART-010
pub(super) fn value_to_hero(v: &Valued<'_>, item: ArtifactInstance, counted: bool) -> i32 {
    let value = hero_value(v, item, false, counted).max(0);
    let details = v.hero.details;
    if h3_heroes::first_position(details, v.rules.core(), item.artifact).is_some() {
        return value;
    }
    let worn = &details.worn;
    let replaced = (0..worn.len())
        .rev()
        .find_map(|at| replaces(worn, v.rules, item.artifact, at))
        .map_or(0, |old| hero_value(v, old, true, counted));
    value.wrapping_sub(replaced).max(0)
}

/// Each hero of `player` on the map (the original's player list, +0x8) with its reads.
fn players_heroes<'a>(
    state: &'a GameState,
    rules: &'a GameRuleset,
    (record, weights): (&Record, &'a [SignedX87]),
    player: PlayerId,
) -> Vec<Valued<'a>> {
    state
        .adventure()
        .heroes()
        .iter()
        .filter(|hero| hero.owner == player)
        .filter_map(|on_map| {
            state.hero(on_map.id).map(|details| {
                Valued::new(state, rules, (record, weights), Fighter { on_map, details })
            })
        })
        .collect()
}

/// The value of `item` to the player of `heroes` (`ai_artifact_value_to_player` 0x433AA0;
/// CPU-ART-011): the largest of 10 and its values to each of the player's heroes on the map as
/// candidates not counted ([`value_to_hero`]).
// rule: CPU-ART-011
pub(super) fn value_to_player(heroes: &[Valued<'_>], item: ArtifactInstance) -> i32 {
    heroes
        .iter()
        .map(|v| value_to_hero(v, item, false))
        .fold(LEAST_TO_PLAYER, i32::max)
}

/// The player's mean artifact value at its turn start (`ai_turn_start_values` 0x527960's +0x164;
/// CPU-STA-008), its record `record` with its heroes' spell gains: the values to the player
/// ([`value_to_player`]) of every artifact 7 to 140 the core has (no scroll's spell), summed in
/// 32 bits, over their count, stored as an f32; the FPU's indefinite with none (0 / 0). The map's
/// bans are not read: the setup's flag the original tests (+0x1C) disables only 141 to 143 (a
/// ruleset without an artifact, RoE's, counts fewer).
// rule: CPU-STA-008
pub(super) fn mean(
    state: &GameState,
    rules: &GameRuleset,
    record: &Record,
    player: PlayerId,
) -> u32 {
    let weights = record.weights(rules);
    let heroes = players_heroes(state, rules, (record, &weights), player);
    let artifacts = rules.core().artifacts().len().min(MEAN_TO + 1);
    let (sum, count) = (MEAN_FROM..artifacts)
        .filter_map(ArtifactTypeId::from_index)
        .fold((0_i32, 0_i32), |(sum, count), artifact| {
            let item = ArtifactInstance::from(artifact);
            (sum.wrapping_add(value_to_player(&heroes, item)), count + 1)
        });
    SignedX87::int(sum.into())
        .over(SignedX87::int(count.into()))
        .map_or(INDEFINITE, |m| m.round(SINGLE_BITS).to_f32_bits())
}
