//! The original's choice between a level-up's two offers (E08 T28, HER-LVL-006): the computer's
//! evaluation, which the original uses for every hero while a new game is set up, for a hero
//! without an owner and for a computer player's hero (`ai_level_up_choice` 0x52BBD0 with
//! `ai_secondary_skill_value` 0x524690 and `ai_takes_new_secondary_skill` 0x524DD0). Only the path
//! with the original's flag clear is built (the setup's and an unowned hero's): the flag-set
//! path reads computer-player state (E10). No draws. Survey
//! `docs/rules/surveys/E08-progression.md` §10, design `docs/epics/E08-design.md` §8.
//!
//! The original indexes its skills by id, Pathfinding 0 to First Aid 27 (`SSTRAITS.TXT`'s
//! order); the evaluation reads the ruleset's skills by that position, as the offers' draws do
//! (the base pack keeps the original's order). A skill past the 28th is worth 0, as the
//! original's switch gives for an id past 27. The values are exact: the original's 32-bit
//! products overflow only for an army worth more than 306 million (the flag-set path's, E10).

use alloc::vec::Vec;

use h3_content::{HeroClass, MAX_HERO_SKILLS, MAX_SKILL_LEVEL, Ruleset, SkillId};
use h3_kernel::arith::{DOUBLE_BITS, X87};

use crate::derive::effective_primary_from;
use crate::details::{HeroDetails, PrimarySkill};
use crate::level_up::{Choice, LevelUp, Offers};

/// The original's skill ids the evaluation tells apart (`SSTRAITS.TXT`'s order).
mod id {
    pub(super) const PATHFINDING: u16 = 0;
    pub(super) const ARCHERY: u16 = 1;
    pub(super) const LOGISTICS: u16 = 2;
    pub(super) const SCOUTING: u16 = 3;
    pub(super) const DIPLOMACY: u16 = 4;
    pub(super) const NAVIGATION: u16 = 5;
    pub(super) const LEADERSHIP: u16 = 6;
    pub(super) const WISDOM: u16 = 7;
    pub(super) const MYSTICISM: u16 = 8;
    pub(super) const LUCK: u16 = 9;
    pub(super) const BALLISTICS: u16 = 10;
    pub(super) const EAGLE_EYE: u16 = 11;
    pub(super) const NECROMANCY: u16 = 12;
    pub(super) const ESTATES: u16 = 13;
    pub(super) const FIRE_MAGIC: u16 = 14;
    pub(super) const EARTH_MAGIC: u16 = 17;
    pub(super) const SCHOLAR: u16 = 18;
    pub(super) const TACTICS: u16 = 19;
    pub(super) const ARTILLERY: u16 = 20;
    pub(super) const LEARNING: u16 = 21;
    pub(super) const OFFENSE: u16 = 22;
    pub(super) const ARMORER: u16 = 23;
    pub(super) const INTELLIGENCE: u16 = 24;
    pub(super) const SORCERY: u16 = 25;
    pub(super) const RESISTANCE: u16 = 26;
    pub(super) const FIRST_AID: u16 = 27;
}

/// Archery's factor by its level 0 to 2 (`ai_archery_values` 0x640380).
const ARCHERY_VALUES: [u64; 3] = [16, 7, 11];

/// Estates' value by its level 0 to 2 (`ai_estates_values` 0x64038C).
const ESTATES_VALUES: [u64; 3] = [725, 725, 1550];

/// The four schools' values without Wisdom, Fire to Earth (0x524690's cases 14 to 17).
const SCHOOL_VALUES: [u64; 4] = [0, 25, 3, 30];

/// The highest primary skill the evaluation counts (it holds Spell Power and Knowledge to 1–99,
/// Attack to 0–99).
const PRIMARY_CAP: u8 = 99;

/// The map's tiles the computer weighs Pathfinding and Navigation by (`ai_map_terrain_shares`
/// 0x52B9C0, survey §10.3): over the tiles of both levels that carry the map file's coast flag
/// and are not rock (only those: `ADV-QUIRK-14`), how many there are, the water tiles among them
/// and, per Pathfinding level 0 to 2, the tiles whose straight step still costs more than 100.
/// The game counts them from its static map (E08 T29); the evaluation divides them. A map
/// without a coast counts none, and both skills are then worth 0 (the original's 0 / 0).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TerrainShares {
    /// The tiles counted.
    pub tiles: u32,
    /// The water tiles among them (none where the coast flag lies on land only, as on every
    /// shipped map: Navigation is then worth 0).
    pub water: u32,
    /// Per Pathfinding level 0 to 2, the tiles among them whose step costs more than 100.
    pub rough: [u32; 3],
}

impl TerrainShares {
    /// `army × count / tiles / divisor` rounded toward zero in the original's arithmetic: the
    /// share stored as an `f64` (0x52B9C0), the product at the FPU's 53 bits, the division by 4
    /// or 2 exact, `_ftol`; 0 without tiles (the original's NaN, which `_ftol` turns into the
    /// 64-bit integer indefinite, whose low half the value keeps).
    fn part(self, count: u32, army: u32, divisor: u64) -> u64 {
        let value = X87::ratio(count, self.tiles, DOUBLE_BITS).and_then(|share| {
            X87::int(u64::from(army))
                .times(share)
                .over(X87::int(divisor))
        });
        value.map_or(0, X87::trunc)
    }
}

/// The army's worth the evaluation weighs the skills by: A, the army's, and B, its shooters'.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArmyWorth {
    /// A.
    pub army: u32,
    /// B.
    pub shooters: u32,
}

impl ArmyWorth {
    /// The flag-clear path's: 1 000 and 500 whatever the army (0x5246C4): the levels at load and
    /// an unowned hero's default (design §8, §13).
    pub const SETUP: Self = Self {
        army: 1000,
        shooters: 500,
    };
}

/// The original's choice between a level-up's two offers (HER-LVL-006): with both known or
/// both new, the offer of the higher value (the first on a tie); with an advance and a new skill,
/// the new one when it ranks among the hero's best unknown skills its free slots could still
/// take, else the advance. The hero is seen with the level's primary skill already raised, as
/// the original chooses after it (0x4DA720). One offer or none: [`Choice::First`], which learns
/// the one offer ([`LevelUp::learned`]).
pub fn original_choice(
    details: &HeroDetails,
    ruleset: &Ruleset,
    level_up: &LevelUp,
    worth: ArmyWorth,
    shares: TerrainShares,
) -> Choice {
    // rule: HER-LVL-006
    let Offers::Two(first, second) = level_up.offers else {
        return Choice::First;
    };
    let hero = Evaluation::new(details, ruleset, level_up, worth, shares);
    let first_new = hero.known(first) == 0;
    if first_new == (hero.known(second) == 0) {
        return if hero.value(first) < hero.value(second) {
            Choice::Second
        } else {
            Choice::First
        };
    }
    let (new, taken, kept) = if first_new {
        (first, Choice::First, Choice::Second)
    } else {
        (second, Choice::Second, Choice::First)
    };
    if hero.takes_new(new) { taken } else { kept }
}

/// A hero as the evaluation sees it at a level-up.
#[derive(Debug)]
pub(crate) struct Evaluation<'a> {
    details: &'a HeroDetails,
    class: Option<&'a HeroClass>,
    skills: u16,
    worth: ArmyWorth,
    shares: TerrainShares,
    attack: u64,
    power: u64,
    knowledge: u64,
}

impl<'a> Evaluation<'a> {
    /// The hero of `details` at the level of `level_up`, its primary skill raised as
    /// [`crate::HeroChange::GainLevel`] raises it: the effective skills (base and worn artifacts,
    /// HER-BAT-001's), Spell Power and Knowledge held to 1–99, Attack to 0–99.
    pub(crate) fn new(
        details: &'a HeroDetails,
        ruleset: &'a Ruleset,
        level_up: &LevelUp,
        worth: ArmyWorth,
        shares: TerrainShares,
    ) -> Self {
        let params = ruleset.hero_params();
        let primary = |skill: PrimarySkill, low: u8| {
            let base = details.primary[skill.index()];
            let value = params.map_or(base, |params| {
                let raised = if skill == level_up.primary {
                    base.saturating_add(1).min(params.primary_max)
                } else {
                    base
                };
                effective_primary_from(details, ruleset, params, skill, raised)
            });
            u64::from(value.clamp(low, PRIMARY_CAP))
        };
        let class = ruleset
            .hero_type(details.hero_type)
            .and_then(|hero| ruleset.hero_class(hero.class));
        Self {
            details,
            class,
            skills: u16::try_from(ruleset.skills().len()).unwrap_or(u16::MAX),
            worth,
            shares,
            attack: primary(PrimarySkill::Attack, 0),
            power: primary(PrimarySkill::Power, 1),
            knowledge: primary(PrimarySkill::Knowledge, 1),
        }
    }

    /// The level of a skill the hero has, 0 for one it does not know.
    pub(crate) fn known(&self, skill: SkillId) -> u8 {
        let entry = self.details.skills.iter().find(|&&(s, _)| s == skill);
        entry.map_or(0, |&(_, level)| level)
    }

    /// The class's weight of a skill (0 when absent).
    fn weight(&self, skill: SkillId) -> u8 {
        let entry = self
            .class
            .and_then(|class| class.skill_weights.iter().find(|&&(s, _)| s == skill));
        entry.map_or(0, |&(_, weight)| weight)
    }

    /// A skill's value (`ai_secondary_skill_value` 0x524690, the flag clear): 0 at Expert, and
    /// for an unknown skill when the hero knows the most skills; else by the original's id, with
    /// L the skill's level (survey §10.2).
    pub(crate) fn value(&self, skill: SkillId) -> u64 {
        let level = self.known(skill);
        if level >= MAX_SKILL_LEVEL || level == 0 && self.details.skills.len() >= MAX_HERO_SKILLS {
            return 0;
        }
        // `level` is 0 to 2 from here.
        let l = usize::from(level);
        let (a, b) = (u64::from(self.worth.army), u64::from(self.worth.shooters));
        let wisdom = self.known(SkillId::from_raw(id::WISDOM)) > 0;
        let raw = skill.raw();
        match raw {
            id::PATHFINDING => self.shares.part(self.shares.rough[l], self.worth.army, 4),
            id::ARCHERY => ARCHERY_VALUES[l] * b / 100,
            id::LOGISTICS => a / 10,
            id::SCOUTING | id::NECROMANCY | id::LEARNING => a / 20,
            id::DIPLOMACY => a / 100,
            id::NAVIGATION => self.shares.part(self.shares.water, self.worth.army, 2),
            id::LEADERSHIP | id::LUCK | id::TACTICS => a / 50,
            id::WISDOM => self.power * 25,
            id::MYSTICISM => 1,
            id::BALLISTICS => a / 8,
            id::EAGLE_EYE => 2 * u64::from(wisdom),
            id::ESTATES => ESTATES_VALUES[l],
            id::FIRE_MAGIC..=id::EARTH_MAGIC if wisdom => self.power * 10,
            id::FIRE_MAGIC..=id::EARTH_MAGIC => SCHOOL_VALUES[usize::from(raw - id::FIRE_MAGIC)],
            id::SCHOLAR => self.power * 5 * u64::from(wisdom),
            id::ARTILLERY => self.attack * 10,
            id::OFFENSE | id::ARMORER => 7 * a / 100,
            id::INTELLIGENCE => self.knowledge * 10,
            id::SORCERY => self.power * 2,
            id::RESISTANCE => a / 40,
            id::FIRST_AID => 250,
            _ => 0,
        }
    }

    /// Whether the new skill `offered` is taken over an advance (`ai_takes_new_secondary_skill`
    /// 0x524DD0): every skill of the ruleset gets a value (a known one 0, an unknown one 0 when
    /// its class weight is 0 unless it is `offered`), an exchange sort ranks them (ascending, an
    /// entry swapped with each later one of a lower value), and a walk from the highest passes
    /// known skills, takes `offered` when it meets it and counts every other unknown skill
    /// against the hero's free slots, refusing when none is left. A walk that ends takes it.
    pub(crate) fn takes_new(&self, offered: SkillId) -> bool {
        let mut ranked: Vec<(SkillId, u64)> = (0..self.skills)
            .map(|raw| {
                let skill = SkillId::from_raw(raw);
                let weighed =
                    self.known(skill) == 0 && (self.weight(skill) > 0 || skill == offered);
                (skill, if weighed { self.value(skill) } else { 0 })
            })
            .collect();
        for i in 0..ranked.len() {
            for j in i + 1..ranked.len() {
                if ranked[i].1 > ranked[j].1 {
                    ranked.swap(i, j);
                }
            }
        }
        let mut free = MAX_HERO_SKILLS.saturating_sub(self.details.skills.len());
        let mut unknown = ranked
            .iter()
            .rev()
            .map(|&(skill, _)| skill)
            .filter(|&skill| self.known(skill) == 0);
        let stop = unknown.find(|&skill| {
            skill == offered || {
                free = free.saturating_sub(1);
                free == 0
            }
        });
        stop.is_none_or(|skill| skill == offered)
    }
}
