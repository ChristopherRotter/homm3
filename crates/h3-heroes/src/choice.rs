//! The original's choice between a level-up's two offers (E08 T28, HER-LVL-006): the computer's
//! evaluation, which the original uses for every hero while a new game is set up, for a hero
//! without an owner and for a computer player's hero (`ai_level_up_choice` 0x52BBD0 with
//! `ai_secondary_skill_value` 0x524690 and `ai_takes_new_secondary_skill` 0x524DD0). The path with
//! the original's flag clear is the setup's and an unowned hero's; since E31 T2e (AI-Q36,
//! `cross-lane`) the same switch has the flag-set path the original's computer player takes
//! ([`computer_skill_value`], [`computer_takes_new_skill`], CPU-SKL-001), with what only the
//! computer knows passed in ([`ComputerReads`]: its values per unit, its owner's gold weight, the
//! schools' values of its caster record). No draws. Survey
//! `docs/rules/surveys/E08-progression.md` §10, design §8 of increment 4's note (since T34b in
//! `docs/epics/E08-log.md`, "Increment 4's design note at its closure"); the flag-set path
//! `docs/rules/surveys/E31-computer-player.md` §2.4.
//!
//! The original indexes its skills by id, Pathfinding 0 to First Aid 27 (`SSTRAITS.TXT`'s
//! order); the evaluation reads the ruleset's skills by that position, as the offers' draws do
//! (the base pack keeps the original's order). A skill past the 28th is worth 0, as the
//! original's switch gives for an id past 27. The values are the original's 32-bit integers
//! (its products wrap only for an army worth more than 306 million, which the flag-set path
//! counts).

use alloc::vec::Vec;

use h3_content::{CreatureFlag, HeroClass, MAX_HERO_SKILLS, MAX_SKILL_LEVEL, Ruleset, SkillId};
use h3_contracts::ArmyStack;
use h3_kernel::arith::{DOUBLE_BITS, SignedX87, X87};

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
const ARCHERY_VALUES: [i32; 3] = [16, 7, 11];

/// Estates' value by its level 0 to 2 (`ai_estates_values` 0x64038C).
const ESTATES_VALUES: [i32; 3] = [725, 725, 1550];

/// The four schools' values without Wisdom, Fire to Earth (0x524690's cases 14 to 17).
const SCHOOL_VALUES: [i32; 4] = [0, 25, 3, 30];

/// The counted army's worth before its stacks (0x5246C4: 1 000, and 500 for the shooters).
const COUNTED_BASE: (i32, i32) = (1000, 500);

/// The highest primary skill the evaluation counts (it holds Spell Power and Knowledge to 1–99,
/// Attack to 0–99).
const PRIMARY_CAP: u8 = 99;

/// The map's tiles the computer weighs Pathfinding and Navigation by (`ai_map_terrain_shares`
/// 0x52B9C0, survey §10.3): over the tiles of both levels that are not rock and that no object
/// blocks, water included (E08 T38b: not the coast, as T25 read it), how many there are, the
/// water tiles among them and, per Pathfinding level 0 to 2, the tiles whose straight step still
/// costs more than 100. The game counts them from its static map (ADV-MAP-002); the evaluation
/// divides them. A map without such a tile counts none, and both skills are then worth 0 (the
/// original's 0 / 0).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TerrainShares {
    /// The tiles counted.
    pub tiles: u32,
    /// The water tiles among them (none on a map without water: Navigation is then worth 0).
    pub water: u32,
    /// Per Pathfinding level 0 to 2, the tiles among them whose step costs more than 100.
    pub rough: [u32; 3],
}

impl TerrainShares {
    /// `army × count / tiles / divisor` rounded toward zero in the original's arithmetic: the
    /// share stored as an `f64` (0x52B9C0), the product at the FPU's 53 bits, the division by 4
    /// or 2 exact, `_ftol`'s low 32 bits; 0 without tiles (the original's NaN, which `_ftol`
    /// turns into the 64-bit integer indefinite, whose low half the value keeps).
    fn part(self, count: u32, army: i32, divisor: i64) -> i32 {
        let value = X87::ratio(count, self.tiles, DOUBLE_BITS).and_then(|share| {
            SignedX87::int(army.into())
                .times(SignedX87::from(share))
                .over(SignedX87::int(divisor))
        });
        value.map_or(0, SignedX87::ftol)
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
    /// an unowned hero's default (increment 4's design note §8, §13).
    pub const SETUP: Self = Self {
        army: 1000,
        shooters: 500,
    };
}

/// What the flag-set path of the evaluation reads besides the hero and the class (the original's
/// computer player, `ai_secondary_skill_value` 0x524690 with its flag set; E31 T2e, AI-Q36):
/// the hero's army counted ([`counted_worth`]), the computer's values per unit of the hero, its
/// owner's gold weight and the four schools' values of its caster record, which the computer
/// keeps (`h3_game::computer`), the war machines it wears and the map's shares.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ComputerReads {
    /// A and B of the hero's army ([`counted_worth`]).
    pub worth: (i32, i32),
    /// The hero's value of one Spell Power (its +0x47E, `ai_hero_effectiveness` 0x527760).
    pub power_gain: i32,
    /// Its value of a Knowledge point (its +0x486).
    pub knowledge_gain: i32,
    /// Its owner's weight of gold (the computer's record's +0x90, a double).
    pub gold_weight: SignedX87,
    /// The four schools' values, Fire to Earth (`ai_school_value` 0x524D20 of each).
    pub schools: [i32; 4],
    /// Whether it wears a Ballista (`hero_wears_artifact` 0x4D91F0 with 4): Artillery counts.
    pub ballista: bool,
    /// Whether it wears a First Aid Tent (with 6): First Aid counts.
    pub tent: bool,
    /// The map's terrain shares (Pathfinding, Navigation).
    pub shares: TerrainShares,
}

/// A and B of the flag-set path (0x5246D5 to 0x524722; CPU-SKL-001): 1 000 and 500, plus over
/// `army`'s stacks each one's AI value times its count, a shooter's (the creature flag
/// `shooter`) into B too, summed in 32 bits.
// rule: CPU-SKL-001
pub fn counted_worth(ruleset: &Ruleset, army: &[ArmyStack]) -> (i32, i32) {
    army.iter().fold(COUNTED_BASE, |(a, b), stack| {
        let info = ruleset.creature(stack.creature);
        let worth = info
            .map_or(0, |c| i32::from_ne_bytes(c.ai_value.to_ne_bytes()))
            .wrapping_mul(i32::from_ne_bytes(stack.count.to_ne_bytes()));
        let shooter = info.is_some_and(|c| c.flags.contains(CreatureFlag::Shooter));
        (
            a.wrapping_add(worth),
            if shooter { b.wrapping_add(worth) } else { b },
        )
    })
}

/// A skill's value to a computer player's hero (`ai_secondary_skill_value` 0x524690 with its
/// flag set; CPU-SKL-001): the flag-clear values of [`original_choice`]'s evaluation on the
/// hero's effective primary skills, with A and B the counted army's and these cases the
/// computer's: Wisdom `SP × power gain / 2`, Mysticism the knowledge gain / 10, Eagle Eye the
/// power gain / 5 (0 without Wisdom), Estates `trunc(value × gold weight)`, the four schools
/// their values whatever Wisdom, Scholar `SP × power gain / 10` (0 without Wisdom), Artillery 0
/// without a worn Ballista, Intelligence `Kn × knowledge gain / 4`, Sorcery `SP × power gain /
/// 20`, First Aid 0 without a worn Tent; 32-bit integers, each division toward 0.
// rule: CPU-SKL-001
pub fn computer_skill_value(
    details: &HeroDetails,
    ruleset: &Ruleset,
    reads: &ComputerReads,
    skill: SkillId,
) -> i32 {
    Evaluation::of(details, ruleset, None, ArmyWorth::SETUP, reads.shares)
        .original(skill, Some(reads))
}

/// Whether a computer player's hero takes the new skill `offered` over an advance
/// (`ai_takes_new_secondary_skill` 0x524DD0 with the flag set; CPU-SKL-001): its ranking of
/// [`original_choice`]'s, by [`computer_skill_value`].
// rule: CPU-SKL-001
pub fn computer_takes_new_skill(
    details: &HeroDetails,
    ruleset: &Ruleset,
    reads: &ComputerReads,
    offered: SkillId,
) -> bool {
    Evaluation::of(details, ruleset, None, ArmyWorth::SETUP, reads.shares)
        .takes_new_with(offered, Some(reads))
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
    attack: i32,
    power: i32,
    knowledge: i32,
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
        Self::of(details, ruleset, Some(level_up.primary), worth, shares)
    }

    /// The hero of `details`, its primary skill `raised` one higher if any: [`Self::new`]'s, or
    /// the hero as it is.
    fn of(
        details: &'a HeroDetails,
        ruleset: &'a Ruleset,
        raised: Option<PrimarySkill>,
        worth: ArmyWorth,
        shares: TerrainShares,
    ) -> Self {
        let params = ruleset.hero_params();
        let primary = |skill: PrimarySkill, low: u8| {
            let base = details.primary[skill.index()];
            let value = params.map_or(base, |params| {
                let raised = if Some(skill) == raised {
                    base.saturating_add(1).min(params.primary_max)
                } else {
                    base
                };
                effective_primary_from(details, ruleset, params, skill, raised)
            });
            i32::from(value.clamp(low, PRIMARY_CAP))
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
        u64::try_from(self.original(skill, None)).unwrap_or(0)
    }

    /// The original's switch (0x524732) with its flag clear (`reads` `None`) or set: 0 at Expert,
    /// and for an unknown skill when the hero knows the most skills; else by the original's id,
    /// in its 32-bit integers.
    fn original(&self, skill: SkillId, reads: Option<&ComputerReads>) -> i32 {
        let level = self.known(skill);
        if level >= MAX_SKILL_LEVEL || level == 0 && self.details.skills.len() >= MAX_HERO_SKILLS {
            return 0;
        }
        // `level` is 0 to 2 from here.
        let l = usize::from(level);
        let (a, b) = reads.map_or(
            (
                i32::from_ne_bytes(self.worth.army.to_ne_bytes()),
                i32::from_ne_bytes(self.worth.shooters.to_ne_bytes()),
            ),
            |r| r.worth,
        );
        let wisdom = self.known(SkillId::from_raw(id::WISDOM)) > 0;
        let (p, k) = (self.power, self.knowledge);
        let raw = skill.raw();
        match (raw, reads) {
            (id::PATHFINDING, _) => self.shares.part(self.shares.rough[l], a, 4),
            (id::ARCHERY, _) => ARCHERY_VALUES[l].wrapping_mul(b) / 100,
            (id::LOGISTICS, _) => a / 10,
            (id::SCOUTING | id::NECROMANCY | id::LEARNING, _) => a / 20,
            (id::DIPLOMACY, _) => a / 100,
            (id::NAVIGATION, _) => self.shares.part(self.shares.water, a, 2),
            (id::LEADERSHIP | id::LUCK | id::TACTICS, _) => a / 50,
            (id::WISDOM, None) => p * 25,
            (id::WISDOM, Some(r)) => p.wrapping_mul(r.power_gain) / 2,
            (id::MYSTICISM, None) => 1,
            (id::MYSTICISM, Some(r)) => r.knowledge_gain / 10,
            (id::BALLISTICS, _) => a / 8,
            (id::EAGLE_EYE | id::SCHOLAR, _) if !wisdom => 0,
            (id::EAGLE_EYE, None) => 2,
            (id::EAGLE_EYE, Some(r)) => r.power_gain / 5,
            (id::ESTATES, None) => ESTATES_VALUES[l],
            (id::ESTATES, Some(r)) => SignedX87::int(ESTATES_VALUES[l].into())
                .times(r.gold_weight)
                .ftol(),
            (id::FIRE_MAGIC..=id::EARTH_MAGIC, Some(r)) => {
                r.schools[usize::from(raw - id::FIRE_MAGIC)]
            }
            (id::FIRE_MAGIC..=id::EARTH_MAGIC, None) if wisdom => p * 10,
            (id::FIRE_MAGIC..=id::EARTH_MAGIC, None) => {
                SCHOOL_VALUES[usize::from(raw - id::FIRE_MAGIC)]
            }
            (id::SCHOLAR, None) => p * 5,
            (id::SCHOLAR, Some(r)) => p.wrapping_mul(r.power_gain) / 10,
            (id::ARTILLERY, Some(r)) if !r.ballista => 0,
            (id::ARTILLERY, _) => self.attack * 10,
            (id::OFFENSE | id::ARMORER, _) => a.wrapping_mul(7) / 100,
            (id::INTELLIGENCE, None) => k * 10,
            (id::INTELLIGENCE, Some(r)) => k.wrapping_mul(r.knowledge_gain) / 4,
            (id::SORCERY, None) => p * 2,
            (id::SORCERY, Some(r)) => p.wrapping_mul(r.power_gain) / 20,
            (id::RESISTANCE, _) => a / 40,
            (id::FIRST_AID, Some(r)) if !r.tent => 0,
            (id::FIRST_AID, _) => 250,
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
        self.takes_new_with(offered, None)
    }

    /// [`Self::takes_new`] with the flag clear (`reads` `None`) or set ([`Self::original`]).
    fn takes_new_with(&self, offered: SkillId, reads: Option<&ComputerReads>) -> bool {
        let mut ranked: Vec<(SkillId, i32)> = (0..self.skills)
            .map(|raw| {
                let skill = SkillId::from_raw(raw);
                let weighed =
                    self.known(skill) == 0 && (self.weight(skill) > 0 || skill == offered);
                (
                    skill,
                    if weighed {
                        self.original(skill, reads)
                    } else {
                        0
                    },
                )
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
