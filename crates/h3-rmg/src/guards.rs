//! The guards (survey `docs/rules/surveys/E14-generator.md` §2.3, "The guards"): a connection's
//! guard value (RMG-CON-018) and the guard monster (`rmg_guard_monster_create` 0x540B20,
//! RMG-CON-021) that steps 6, 8 and 10 create, from the creature table as the generator reads it
//! ([`guard_creatures`]).

use alloc::vec::Vec;

use h3_content::Ruleset;

use crate::generate::{Stream, Trace};
use crate::objects::{MONSTERS, ObjectFault, Objects};
use crate::part::Zone;
use crate::settings::MapFormat;

/// The call site of a guard monster's creature draw (`rmg_guard_monster_create`,
/// `rmg_guard_creature_draw`): `rand() %` the fitting creatures, the r-th from the top.
pub const SITE_GUARD_CREATURE: u32 = 0x0054_0C61;

/// The call site of a guard monster's count raised (`rmg_guard_count_plus_draw`): `rand() % s`.
pub const SITE_GUARD_COUNT_PLUS: u32 = 0x0054_0CCE;

/// The call site of a guard monster's count lowered (`rmg_guard_count_minus_draw`), right after.
pub const SITE_GUARD_COUNT_MINUS: u32 = 0x0054_0CD8;

/// The creature ids the guards read: 0 to 144 (`rmg_guard_monster_create`'s array of 145).
pub const GUARD_CREATURES: usize = 145;

/// The last creature RoE filters (116; its walk still starts at 144, RMG-QUIRK-10).
const ROE_LAST: usize = 116;

/// A guard's least value (below it 0, none).
const LEAST_GUARD: i32 = 2000;

// stub: RMG-STUB-3 — the four guard tables are pack data by ADR-0029 point 5 (`packs/sod/rmg.toml`'s
// `[generator]`, harvested `cross-lane`); kept here, with their addresses, until harvested.
/// `rmg_guard_threshold_low` 0x6823F0 by monster strength 0 to 5.
const THRESHOLD_LOW: [i32; 6] = [50_000, 2_500, 1_500, 1_000, 500, 0];
/// `rmg_guard_threshold_high` 0x682408.
const THRESHOLD_HIGH: [i32; 6] = [50_000, 7_500, 7_500, 7_500, 5_000, 5_000];
/// `rmg_guard_factor_low` 0x682420.
const FACTOR_LOW: [i32; 6] = [0, 2, 3, 4, 6, 6];
/// `rmg_guard_factor_high` 0x682438.
const FACTOR_HIGH: [i32; 6] = [0, 2, 3, 4, 4, 6];

/// The packs' creature keys by the original's creature id, 0 to 144 (`h3_map`'s import codes,
/// the packs' `# original id`; empty: the four ids CRTRAITS.TXT leaves "NOT USED"). The packs'
/// handles are not the original's ids: `base` lists the war machines (145 to 149) after 117,
/// `ab` its creatures from 118 after them.
const CREATURE_KEYS: [&str; GUARD_CREATURES] = [
    "base:pikeman",
    "base:halberdier",
    "base:archer",
    "base:marksman",
    "base:griffin",
    "base:royal_griffin",
    "base:swordsman",
    "base:crusader",
    "base:monk",
    "base:zealot",
    "base:cavalier",
    "base:champion",
    "base:angel",
    "base:archangel",
    "base:centaur",
    "base:centaur_captain",
    "base:dwarf",
    "base:battle_dwarf",
    "base:wood_elf",
    "base:grand_elf",
    "base:pegasus",
    "base:silver_pegasus",
    "base:dendroid_guard",
    "base:dendroid_soldier",
    "base:unicorn",
    "base:war_unicorn",
    "base:green_dragon",
    "base:gold_dragon",
    "base:gremlin",
    "base:master_gremlin",
    "base:stone_gargoyle",
    "base:obsidian_gargoyle",
    "base:stone_golem",
    "base:iron_golem",
    "base:mage",
    "base:arch_mage",
    "base:genie",
    "base:master_genie",
    "base:naga",
    "base:naga_queen",
    "base:giant",
    "base:titan",
    "base:imp",
    "base:familiar",
    "base:gog",
    "base:magog",
    "base:hell_hound",
    "base:cerberus",
    "base:demon",
    "base:horned_demon",
    "base:pit_fiend",
    "base:pit_lord",
    "base:efreeti",
    "base:efreet_sultan",
    "base:devil",
    "base:arch_devil",
    "base:skeleton",
    "base:skeleton_warrior",
    "base:walking_dead",
    "base:zombie",
    "base:wight",
    "base:wraith",
    "base:vampire",
    "base:vampire_lord",
    "base:lich",
    "base:power_lich",
    "base:black_knight",
    "base:dread_knight",
    "base:bone_dragon",
    "base:ghost_dragon",
    "base:troglodyte",
    "base:infernal_troglodyte",
    "base:harpy",
    "base:harpy_hag",
    "base:beholder",
    "base:evil_eye",
    "base:medusa",
    "base:medusa_queen",
    "base:minotaur",
    "base:minotaur_king",
    "base:manticore",
    "base:scorpicore",
    "base:red_dragon",
    "base:black_dragon",
    "base:goblin",
    "base:hobgoblin",
    "base:wolf_rider",
    "base:wolf_raider",
    "base:orc",
    "base:orc_chieftain",
    "base:ogre",
    "base:ogre_mage",
    "base:roc",
    "base:thunderbird",
    "base:cyclops",
    "base:cyclops_king",
    "base:behemoth",
    "base:ancient_behemoth",
    "base:gnoll",
    "base:gnoll_marauder",
    "base:lizardman",
    "base:lizard_warrior",
    "base:gorgon",
    "base:mighty_gorgon",
    "base:serpent_fly",
    "base:dragon_fly",
    "base:basilisk",
    "base:greater_basilisk",
    "base:wyvern",
    "base:wyvern_monarch",
    "base:hydra",
    "base:chaos_hydra",
    "base:air_elemental",
    "base:earth_elemental",
    "base:fire_elemental",
    "base:water_elemental",
    "base:gold_golem",
    "base:diamond_golem",
    "ab:pixie",
    "ab:sprite",
    "ab:psychic_elemental",
    "ab:magic_elemental",
    "",
    "ab:ice_elemental",
    "",
    "ab:magma_elemental",
    "",
    "ab:storm_elemental",
    "",
    "ab:energy_elemental",
    "ab:firebird",
    "ab:phoenix",
    "ab:azure_dragon",
    "ab:crystal_dragon",
    "ab:faerie_dragon",
    "ab:rust_dragon",
    "ab:enchanter",
    "ab:sharpshooter",
    "ab:halfling",
    "ab:peasant",
    "ab:boar",
    "ab:mummy",
    "ab:nomad",
    "ab:rogue",
    "ab:troll",
];

/// A creature as the guards read the creature table (`creature_table_pointer` 0x6747B0's records
/// of 0x74 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GuardCreature {
    /// +0x0: its town, Castle 0 to Conflux 8, -1 neutral.
    pub faction: i32,
    /// +0x4: its level from 0 (the packs' less 1), -1 an id without a creature.
    pub level: i32,
    /// +0x40: its AI value.
    pub ai_value: i32,
    /// +0x6C and +0x70: the adventure map's least and greatest count.
    pub counts: [i32; 2],
}

impl GuardCreature {
    /// An id without a creature (122, 124, 126 and 128, CRTRAITS.TXT's "NOT USED"; a key the
    /// ruleset lacks): Conflux's, level -1 and zeros as the table holds them, never fitting.
    pub const UNUSED: Self = Self {
        faction: 8,
        level: -1,
        ai_value: 0,
        counts: [0, 0],
    };
}

/// The creature table the guards read, by the original's creature id, 0 to 144, from `core`'s
/// creatures ([`CREATURE_KEYS`]): the faction's index (the packs' factions are Castle to Conflux
/// in the original's order), the level less 1, the AI value, the adventure map's counts.
// rule: RMG-CON-021
pub fn guard_creatures(core: &Ruleset) -> Vec<GuardCreature> {
    CREATURE_KEYS
        .iter()
        .map(|key| {
            core.creature_by_key(key)
                .and_then(|id| core.creature(id))
                .map_or(GuardCreature::UNUSED, |c| GuardCreature {
                    faction: c.faction.map_or(-1, |f| f.index() as i32),
                    level: i32::from(c.level) - 1,
                    ai_value: c.ai_value.cast_signed(),
                    counts: c.adventure_count.map(i32::from),
                })
        })
        .collect()
}

/// What a guard monster reads besides the zone.
#[derive(Debug, Clone, Copy)]
pub(crate) struct GuardInput<'a> {
    /// The creature table by the original's id ([`guard_creatures`]; an id past it never fits).
    pub creatures: &'a [GuardCreature],
    /// The map's format (the generator's +0x8: RoE filters the creatures 0 to 116).
    pub format: MapFormat,
    /// The generator's monster strength (+0x10BC), 1 to 5 (RMG-SET-005).
    pub strength: i32,
    /// `quirk.rmg.roe_guard_walk` (RMG-QUIRK-10): RoE's draw walks from creature 144, not from the
    /// last it filtered.
    pub roe_guard_walk: bool,
}

/// A guard monster placed (`rmg_monster_object`, 0x2C bytes; its +0x24 is 3 for every guard).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Monster {
    /// Its object, by its index among the placed objects.
    pub object: usize,
    /// +0x1C: the generator's object serial (+0xF44) when it was created.
    pub serial: i32,
    /// +0x20: its count.
    pub count: i32,
}

/// A guard monster created, not yet placed: its prototype, serial and count.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct NewMonster {
    pub prototype: usize,
    pub serial: i32,
    pub count: i32,
}

/// A connection's guard value for its value `value` and the generator's monster strength `s`
/// (`rmg_connection_by_land` 0x54138A, the other ways' alike; not wide): (value − low[s]) ×
/// low factor[s] / 4 when above the low threshold, plus (value − high[s]) × high factor[s] / 4
/// when above the high one (32-bit products, the quarters truncating), 0 under 2000. `s` is 1 to
/// 5 (RMG-SET-005's clamp; the port clamps to the tables, which the original never leaves).
// rule: RMG-CON-018
pub(crate) fn guard_value(value: i32, s: i32) -> i32 {
    let s = s.clamp(0, 5) as usize;
    let mut g = 0i32;
    if value > THRESHOLD_LOW[s] {
        g = value
            .wrapping_sub(THRESHOLD_LOW[s])
            .wrapping_mul(FACTOR_LOW[s])
            / 4;
    }
    if value > THRESHOLD_HIGH[s] {
        g = g.wrapping_add(
            value
                .wrapping_sub(THRESHOLD_HIGH[s])
                .wrapping_mul(FACTOR_HIGH[s])
                / 4,
        );
    }
    if g < LEAST_GUARD { 0 } else { g }
}

/// `rmg_guard_monster_create` 0x540B20: a guard monster worth `value` for a zone `zone` whose
/// zone generator's town type is `town_type`. The towns allowed: that town type alone when the
/// zone's monsters match its town (+0x94) and it has one, else the zone's neutral byte and its
/// nine monster towns (+0x95 to +0x9E; a town type past Conflux allows none, the original writing
/// past its array). Per creature id the last monster prototype of that subclass (class 54's
/// vector, sorted); for RoE those from 118 dropped. From the last creature filtered (116 for RoE,
/// else 144) down to 0, a creature fits when AI × ((low + high) / 2) ≤ `value` ≤ AI × 100
/// (32-bit), its level is 0 or more and its town allowed (a faction past Conflux not); one
/// that does not loses its prototype. None fits: no monster, no draw. Else `rand() %` the
/// fitting ones (0x540C61) r, the creature the (r + 1)-th with a prototype from 144 down (from
/// RoE's last filtered without `roe_guard_walk`, RMG-QUIRK-10); its count n = (`value` + AI / 2)
/// / AI, and with s = n / 4 + 1 > 1, n + `rand() % s` (0x540CCE) − `rand() % s` (0x540CD8). The
/// monster takes the generator's serial (+0xF44) and raises its prototype's use count.
///
/// The original's faults ([`ObjectFault::Creature`], recorded in `objects`): no monster.
// rule: RMG-CON-021
pub(crate) fn create<T: Trace>(
    stream: &mut Stream<'_, T>,
    input: &GuardInput<'_>,
    objects: &mut Objects,
    value: i32,
    zone: &Zone,
    town_type: i32,
) -> Option<NewMonster> {
    // Neutral, then Castle to Conflux.
    let mut allowed = [false; 10];
    if zone.monster_match && town_type != -1 {
        if let Some(a) = usize::try_from(town_type.wrapping_add(1))
            .ok()
            .and_then(|i| allowed.get_mut(i))
        {
            *a = true;
        }
    } else {
        allowed[0] = zone.neutral_monsters;
        for (t, a) in allowed[1..].iter_mut().enumerate() {
            *a = zone.monster_towns & (1 << t) != 0;
        }
    }
    let mut map = [None; GUARD_CREATURES];
    for &p in objects.group(MONSTERS) {
        match map.get_mut(objects.prototypes()[p].shape.subclass as usize) {
            Some(m) => *m = Some(p),
            None => {
                objects.fail(ObjectFault::Creature);
                return None;
            }
        }
    }
    let roe = input.format.version() < 1;
    let last = if roe {
        map[ROE_LAST + 2..].fill(None);
        ROE_LAST
    } else {
        GUARD_CREATURES - 1
    };
    let mut fitting = 0i32;
    for c in (0..=last).rev() {
        let fits = input.creatures.get(c).is_some_and(|k| {
            let mid = k.counts[0].wrapping_add(k.counts[1]) / 2;
            k.ai_value.wrapping_mul(mid) <= value
                && value <= k.ai_value.wrapping_mul(100)
                && k.level >= 0
                && usize::try_from(k.faction.wrapping_add(1))
                    .ok()
                    .and_then(|i| allowed.get(i))
                    .is_some_and(|&a| a)
        });
        if fits {
            fitting += 1;
        } else {
            map[c] = None;
        }
    }
    if fitting == 0 {
        return None;
    }
    // `rand()` is at most 0x7FFF: the signed remainder is the unsigned one.
    let r = (stream.draw(SITE_GUARD_CREATURE) as i32 % fitting) as usize;
    let top = if roe && !input.roe_guard_walk {
        last
    } else {
        GUARD_CREATURES - 1
    };
    let chosen = (0..=top)
        .rev()
        .filter_map(|c| map[c].map(|p| (c, p)))
        .nth(r);
    let Some((c, prototype)) = chosen else {
        objects.fail(ObjectFault::Creature);
        return None;
    };
    let ai = input.creatures.get(c).map_or(0, |k| k.ai_value);
    let Some(n) = value.wrapping_add(ai / 2).checked_div(ai) else {
        objects.fail(ObjectFault::Creature);
        return None;
    };
    let s = (n / 4).wrapping_add(1);
    let mut count = n;
    if s > 1 {
        let plus = stream.draw(SITE_GUARD_COUNT_PLUS) as i32 % s;
        let minus = stream.draw(SITE_GUARD_COUNT_MINUS) as i32 % s;
        count = n.wrapping_add(plus.wrapping_sub(minus));
    }
    let serial = objects.take_serial();
    objects.created(prototype);
    Some(NewMonster {
        prototype,
        serial,
        count,
    })
}

#[cfg(test)]
mod tests;
