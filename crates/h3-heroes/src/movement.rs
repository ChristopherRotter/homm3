//! What the adventure map reads of a hero to move and scout (E08 T39; ADR-0013 point 13,
//! HER-ADV-003 to HER-ADV-007; survey `docs/rules/surveys/E08-hero-movement.md` §1, §2, §4): the
//! day's land points grown by the hero's rates and flat bonuses, a creature's speed for the slowest
//! stack, the Pathfinding, flying and water-walking levels, and the scouting radius. Pure functions
//! of the details, the core ruleset and the adventure's inputs (the land table's entry, the base
//! radius); when the map reads them is the adventure's (ADV-MOV-001, -002, ADV-FOG-001). No skill or
//! artifact is read by name (ADR-0016 point 4): the effect kinds `land_movement`,
//! `land_movement_points`, `pathfinding`, `flying`, `water_walking`, `scouting_radius` and the
//! creature specialty's `speed`. Every artifact type counts once, worn or as a part of a worn
//! combination, one that needs a skill only with it (the original's `hero_wears_artifact`
//! 0x4D91F0 is a test per type).

use h3_content::{CreatureTypeId, HeroEffect, Ruleset};

use crate::derive::{Count, effects, grown};
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
