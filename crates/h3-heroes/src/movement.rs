//! What the adventure map reads of a hero to move and scout (E08 T39; ADR-0013 point 13,
//! HER-ADV-003 to HER-ADV-007; survey `docs/rules/surveys/E08-hero-movement.md` §1, §2, §4): the
//! day's land points grown by the hero's rates and flat bonuses, a creature's speed for the slowest
//! stack, the Pathfinding, flying and water-walking levels, and the scouting radius. Pure functions
//! of the details, the core ruleset and the adventure's inputs (the land table's entry, the base
//! radius); when the map reads them is the adventure's (ADV-MOV-001, -002, ADV-FOG-001). No skill
//! or artifact is read by name (ADR-0016 point 4): the effect kinds `land_movement`,
//! `land_movement_points`, `pathfinding`, `flying`, `water_walking`, `scouting_radius` and the
//! creature specialty's `speed`. Every artifact type counts once, worn or as a part of a worn
//! combination, one that needs a skill only with it (the original's `hero_wears_artifact`
//! 0x4D91F0 is a test per type). Since E08 T71 the day's sea points and free boarding
//! (HER-ADV-009; ADR-0013 points 13.6, 13.7; survey `docs/rules/surveys/E08-sea.md` §3.1, §4.2):
//! the kinds `navigation`, `sea_movement_points` and `free_boarding`, Navigation's skill
//! specialty.

use h3_content::{CreatureTypeId, FULL_BP, HeroEffect, Ruleset};
use h3_kernel::arith::mul_div_floor;

use crate::derive::{Count, effects, grown, specialty};
use crate::details::HeroDetails;

/// The day's land points from `base`, the land table's entry the adventure gives (HER-ADV-003,
/// E08 T39): `base` grown by the sum of the hero's `land_movement` rates (Logistics: 10 %, 20 %,
/// 30 %; each skill's grown by the hero's specialties for it, 5 % of itself per level for the
/// Logistics specialists), rounded down, then the `land_movement_points` of the worn artifact types
/// added (Boots of Speed 600, Equestrian's Gloves 300), saturating. With quirk Q-0005 as the
/// original's `hero_max_land_movement` (0x4E4990) computes the growth: the rate `f32(r ×
/// (f32(level) × 0.05 + 1))`, the sum in single precision, `trunc((1 + sum) × base)` in its x87
/// arithmetic, so a specialist can get one point less than the exact basis points give (D-22,
/// batch E5's E5-2); without a specialty both give `⌊base × (10 + Logistics level) / 10⌋`.
pub fn land_points(details: &HeroDetails, core: &Ruleset, base: u32) -> u32 {
    // rule: HER-ADV-003
    let points = grown(details, core, base, |effect| match *effect {
        HeroEffect::LandMovement { bp } => Some(bp),
        _ => None,
    });
    effects(details, core, Count::OncePerType)
        .filter_map(|effect| match *effect {
            HeroEffect::LandMovementPoints { points } => Some(u32::from(points)),
            _ => None,
        })
        .fold(
            u32::try_from(points).unwrap_or(u32::MAX),
            u32::saturating_add,
        )
}

/// The day's sea points from `table`, the adventure's sea points by Navigation level (0 to 3;
/// the original's `sea_movement_by_navigation` 0x698AEC: 1 500, 2 250, 3 000, 3 750; HER-ADV-009,
/// E08 T71): the entry at the highest of the hero's `navigation` levels (0 without one); then,
/// for each skill the hero has whose effects at its level give a Navigation level, its skill
/// specialties' `bp_per_level` of the table's first entry per hero level, summed and rounded down
/// once (⌊level × 1 500 × 500 ÷ 10 000⌋, 75 a level for the original's Navigation specialists,
/// who get nothing without the skill); then the `sea_movement_points` of every effect, each worn
/// artifact type once, worn or as a part of a worn combination (the Necklace of Ocean Guidance
/// 1 000, the Sea Captain's Hat 500, the Admiral's Hat both through its parts); saturating. The
/// army's speeds do not count. Exact integers in every profile (ADR-0013 point 13.6: the
/// original's `hero_max_land_movement` 0x4E4990 with its sea argument adds `level × table[0] ÷
/// 20` in integers). Lighthouses and a computer player's bonus are the caller's.
pub fn sea_points(details: &HeroDetails, core: &Ruleset, table: &[u32; 4]) -> u32 {
    // rule: HER-ADV-009
    let mut level = 0;
    let mut flat = 0_u32;
    for effect in effects(details, core, Count::OncePerType) {
        match *effect {
            HeroEffect::Navigation { level: l } => level = level.max(l),
            HeroEffect::SeaMovementPoints { points } => {
                flat = flat.saturating_add(u32::from(points))
            }
            _ => {}
        }
    }
    let specialty = specialty(details, core);
    let navigates = |&&(skill, at): &&(_, u8)| {
        core.skill(skill).is_some_and(|s| {
            s.effects(at)
                .iter()
                .any(|e| matches!(e, HeroEffect::Navigation { .. }))
        })
    };
    let bp: u64 = details
        .skills
        .iter()
        .filter(navigates)
        .flat_map(|&(id, _)| {
            specialty.iter().filter_map(move |effect| match *effect {
                HeroEffect::SkillSpecialty {
                    skill,
                    bp_per_level,
                } if skill == id => Some(u64::from(bp_per_level)),
                _ => None,
            })
        })
        .sum();
    let per_level = u64::from(details.level) * u64::from(table[0]);
    let specialist = mul_div_floor(per_level, bp, FULL_BP.into()).unwrap_or(u64::MAX);
    let entry = u64::from(table[usize::from(level.min(3))]);
    u32::try_from(entry.saturating_add(specialist))
        .unwrap_or(u32::MAX)
        .saturating_add(flat)
}

/// Whether a boarding or a landing converts the hero's points left (HER-ADV-009, E08 T71;
/// ADR-0013 point 13.7): one of its effects is `free_boarding`, from a worn artifact type or a
/// part of a worn combination (one that needs a skill only with it), a skill or the specialty.
/// The packs give it to the Admiral's Hat alone, the combination and not its parts: the
/// original tests the Hat (`hero_wears_artifact` 0x4D91F0 with 136), so the Necklace of Ocean
/// Guidance and the Sea Captain's Hat worn apart do not convert. The conversion itself is the
/// adventure's (ADR-0016 point 3).
pub fn free_boarding(details: &HeroDetails, core: &Ruleset) -> bool {
    // rule: HER-ADV-009
    effects(details, core, Count::OncePerType).any(|e| matches!(e, HeroEffect::FreeBoarding))
}

/// The speed a stack of `creature` counts with in the hero's army for the day's points
/// (HER-ADV-004, E08 T39): the creature's speed in the core (its table's: no artifact and no
/// `creature_stat` effect counts), plus the `speed` of each of the hero's creature specialties
/// whose creature is `creature` or has it as a direct upgrade (the original's "one faster" for
/// the specialty's creature and its upgrade, `hero_max_land_movement` 0x4E4990), saturating;
/// `u8::MAX` for a creature the core lacks (the fastest, as ADV-MOV-001 counts it). Not modelled:
/// the original's four elementals without an upgrade on a RoE map (the packs decide upgrades).
pub fn movement_speed(details: &HeroDetails, core: &Ruleset, creature: CreatureTypeId) -> u8 {
    // rule: HER-ADV-004
    core.creature(creature).map_or(u8::MAX, |info| {
        effects(details, core, Count::OncePerType)
            .filter_map(|effect| match *effect {
                HeroEffect::CreatureSpecialty {
                    creature: specialty,
                    speed,
                    ..
                } if specialty == creature || core.upgrades(specialty).contains(&creature) => {
                    Some(speed)
                }
                _ => None,
            })
            .fold(info.speed, u8::saturating_add)
    })
}

/// A hero's levels for the price and the passage of its steps (E08 T39), derived once per search
/// or move, never per step (the reachable tiles' hot loop): [`movement`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Movement {
    /// The Pathfinding level (HER-ADV-007): the highest of the `pathfinding` effects, 0 without
    /// (the skill's 1 to 3; the terrains' prices by level, ADV-MOV-003).
    pub pathfinding: u8,
    /// The flying level (HER-ADV-005): the highest of the `flying` effects, `None` without (Angel
    /// Wings 3; the index of the adventure's fly prices, ADV-MOV-009).
    pub flying: Option<u8>,
    /// The water-walking level (HER-ADV-005): the highest of the `water_walking` effects, `None`
    /// without (Boots of Levitation 3; the index of the water-walk prices, ADV-MOV-010).
    pub water_walking: Option<u8>,
}

/// The hero's [`Movement`] in one pass over its effects (HER-ADV-005, HER-ADV-007; E08 T39): the
/// secondary skills' at their levels, the worn artifact types' and the specialty's; several
/// sources of a kind give the highest. The original's `hero_step_cost` (0x4B18C0) takes the
/// Pathfinding level from the hero's skill 0 (+0xC9), the fly level from +0x112 forced to 3 by
/// Angel Wings (72) and the water-walk level from +0x116 forced to 3 by Boots of Levitation (90),
/// each −1 without a source; the spells' levels (Fly, Water Walk) come with them later.
pub fn movement(details: &HeroDetails, core: &Ruleset) -> Movement {
    // rule: HER-ADV-005, HER-ADV-007
    let highest = |known: Option<u8>, level: u8| Some(known.map_or(level, |k| k.max(level)));
    let mut out = Movement::default();
    for effect in effects(details, core, Count::OncePerType) {
        match *effect {
            HeroEffect::Pathfinding { level } => out.pathfinding = out.pathfinding.max(level),
            HeroEffect::Flying { level } => out.flying = highest(out.flying, level),
            HeroEffect::WaterWalking { level } => {
                out.water_walking = highest(out.water_walking, level)
            }
            _ => {}
        }
    }
    out
}

/// The hero's scouting radius from `base`, the adventure's (5 in the original packs; HER-ADV-006,
/// E08 T39): `base` plus the `scouting_radius` tiles of the hero's effects (Scouting 1, 2, 3; the
/// Speculum and the Spyglass 1 each, each type once), saturating. A skill specialty grows nothing
/// here (ADR-0013 point 13.4: the original's Scouting specialty has no SoD hero, and its three
/// sources disagree on it). The original's `hero_scouting_radius` (0x4E4070): `skill_scouting`
/// (5, 6, 7, 8) at the Scouting level, +1 for each of the Speculum (52) and the Spyglass (53).
pub fn scouting_radius(details: &HeroDetails, core: &Ruleset, base: u8) -> u8 {
    // rule: HER-ADV-006
    effects(details, core, Count::OncePerType)
        .filter_map(|effect| match *effect {
            HeroEffect::ScoutingRadius { tiles } => Some(tiles),
            _ => None,
        })
        .fold(base, u8::saturating_add)
}
