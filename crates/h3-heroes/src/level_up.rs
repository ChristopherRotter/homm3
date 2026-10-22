//! A level-up (E08 T27): the draw source a caller gives and the original's generator
//! (HER-LVL-002), the primary skill (HER-LVL-001; A-9, `ADV-QUIRK-13`), the two offers
//! (HER-LVL-003, HER-LVL-004) and their result, [`LevelUp`], which changes nothing until the game
//! applies the answer as [`HeroChange::GainLevel`] (HER-LVL-005). Survey
//! `docs/rules/surveys/E08-progression.md` §2, design §4 to §7 of increment 4's note (since T34b in
//! `docs/epics/E08-log.md`, "Increment 4's design note at its closure").

use core::ops::Range;

use h3_content::{HeroClass, HeroParams, MAX_HERO_SKILLS, MAX_SKILL_LEVEL, Ruleset, SkillId};
use h3_kernel::Rng;

use crate::change::HeroChange;
use crate::details::{HeroDetails, OfferLevels, PrimarySkill};
use crate::level::level_of;

/// Where a level-up's numbers come from (HER-LVL-002; ADR-0003 point 7, ADR-0013 point 12.3):
/// the caller's choice (ADR-0017 point 4), [`OriginalDraws`] or the owner's stream.
pub trait Draws {
    /// A number from `low` to `high`, both included (`low` ≤ `high`).
    fn between(&mut self, low: u32, high: u32) -> u32;
}

/// The owner's stream (`strict-fixed`): [`Rng::range_inclusive`].
impl Draws for Rng {
    fn between(&mut self, low: u32, high: u32) -> u32 {
        self.range_inclusive(low, high)
    }
}

/// The original's draws (`faithful`, `ADV-QUIRK-9`): the C runtime's `rand` (the executable's
/// `crt_rand`, 0x617E5C) seeded per level by `hero_level_up` (0x4DA801 to 0x4DA830), and its
/// `random_between` (0x50B230). The twin of `h3_adventure::split::TileRand`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OriginalDraws(u32);

impl OriginalDraws {
    /// The generator of the hero's `level`-th level (the new level) for its seed byte `seed`
    /// (1 to 255, drawn once per hero): seeded with 214 013 L + 156 823 S + 154 079 in 32 bits.
    pub const fn new(seed: u8, level: u8) -> Self {
        let state = (level as u32)
            .wrapping_mul(214_013)
            .wrapping_add((seed as u32).wrapping_mul(156_823))
            .wrapping_add(154_079);
        Self(state)
    }

    /// The generator's state: its seed until the first draw.
    pub const fn state(self) -> u32 {
        self.0
    }

    /// The next `rand()`: the state times 214 013 plus 2 531 011, bits 16 to 30 of it.
    pub const fn rand(&mut self) -> u32 {
        self.0 = self.0.wrapping_mul(214_013).wrapping_add(2_531_011);
        (self.0 >> 16) & 0x7FFF
    }
}

impl Draws for OriginalDraws {
    /// `random_between(low, high)`: `low` + `rand()` mod (`high` − `low` + 1).
    fn between(&mut self, low: u32, high: u32) -> u32 {
        // rule: HER-LVL-002
        let span = high.saturating_sub(low).saturating_add(1);
        low.saturating_add(self.rand() % span)
    }
}

/// The offers of a level (HER-LVL-003): none, one, or two in order (the dialog's left and right).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Offers {
    /// No skill can be offered: the level brings the primary skill only.
    None,
    /// One skill, learned without a choice.
    One(SkillId),
    /// Two skills: the first an advance if the hero has one to make, the second a new skill if
    /// it can learn one.
    Two(SkillId, SkillId),
}

/// Which of two offers is taken.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Choice {
    /// The first offer (the dialog's left).
    First,
    /// The second offer (the dialog's right).
    Second,
}

/// What a level-up draws (design §6): the new level, the primary skill it raises, its offers and
/// the levels at which Wisdom and a school were last offered after it. Drawing it changes
/// nothing; [`LevelUp::change`] is the answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LevelUp {
    /// The new level.
    pub level: u8,
    /// The primary skill that gains 1 (HER-LVL-001).
    pub primary: PrimarySkill,
    /// The offers (HER-LVL-003).
    pub offers: Offers,
    /// The offers' levels after this level (HER-LVL-004): the offers set them, whatever the
    /// answer.
    pub offered: OfferLevels,
}

impl LevelUp {
    /// The skill the answer learns: the one offer whatever the choice, the chosen one of two,
    /// none without an offer.
    pub const fn learned(&self, choice: Choice) -> Option<SkillId> {
        match (self.offers, choice) {
            (Offers::None, _) => None,
            (Offers::One(skill) | Offers::Two(skill, _), Choice::First)
            | (Offers::One(skill) | Offers::Two(_, skill), Choice::Second) => Some(skill),
        }
    }

    /// The answer as the hero's change (HER-LVL-005).
    pub const fn change(&self, choice: Choice) -> HeroChange {
        HeroChange::GainLevel {
            primary: self.primary,
            learned: self.learned(choice),
            offered: self.offered,
        }
    }
}

/// The hero's next level-up (HER-LVL-001 to HER-LVL-004, `hero_level_up` 0x4DA720 for one
/// level): drawn through `draws` (the primary skill, then the first offer, then the second; 1 to
/// 3 numbers) with the map's `banned` skills. `None` when no level is pending
/// ([`crate::pending_levels`]) or the ruleset lacks the hero's parameters or type.
pub fn draw_level_up(
    details: &HeroDetails,
    ruleset: &Ruleset,
    banned: &[SkillId],
    draws: &mut impl Draws,
) -> Option<LevelUp> {
    let params = ruleset.hero_params()?;
    let class = ruleset
        .hero_type(details.hero_type)
        .and_then(|hero| ruleset.hero_class(hero.class))?;
    let level = details
        .level
        .checked_add(1)
        .filter(|&level| level <= level_of(params, details.experience))?;
    let primary = draw_primary(class, level, draws);
    let hero = Offering {
        details,
        ruleset,
        params,
        class,
        level,
        banned,
    };
    // rule: HER-LVL-003
    let mut offered = details.offered;
    let first = hero
        .offer(offered, 1..MAX_SKILL_LEVEL, None, draws)
        .or_else(|| hero.offer(offered, 0..MAX_SKILL_LEVEL, None, draws));
    // rule: HER-LVL-004
    if first.is_some_and(|skill| params.schools.contains(&skill)) {
        offered.school = level;
    }
    let second = first.and_then(|first| {
        hero.offer(offered, 0..1, Some(first), draws)
            .or_else(|| hero.offer(offered, 0..MAX_SKILL_LEVEL, Some(first), draws))
            .map(|second| (first, second))
    });
    let offers = match (first, second) {
        (_, Some((first, second))) => Offers::Two(first, second),
        (Some(first), None) => Offers::One(first),
        (None, None) => Offers::None,
    };
    for skill in first.into_iter().chain(second.map(|(_, s)| s)) {
        if skill == params.wisdom {
            offered.wisdom = level;
        }
        if params.schools.contains(&skill) {
            offered.school = level;
        }
    }
    Some(LevelUp {
        level,
        primary,
        offers,
        offered,
    })
}

/// The primary skill a new `level` raises (HER-LVL-001): a draw of 1 to 100 walks the class's
/// chances (the second row from the level-up to level 10, D-15) in the skills' order.
fn draw_primary(class: &HeroClass, level: u8, draws: &mut impl Draws) -> PrimarySkill {
    // rule: HER-LVL-001
    let row = class.primary_gain[usize::from(level >= 10)];
    let mut rest = draws.between(1, 100);
    let stop = PrimarySkill::ALL.into_iter().zip(row).find(|&(_, chance)| {
        let reached = rest <= u32::from(chance);
        rest = rest.saturating_sub(u32::from(chance));
        reached
    });
    stop.map_or(PrimarySkill::Knowledge, |(skill, _)| skill)
}

/// A hero at its new level, for its offers.
struct Offering<'a> {
    details: &'a HeroDetails,
    ruleset: &'a Ruleset,
    params: &'a HeroParams,
    class: &'a HeroClass,
    level: u8,
    banned: &'a [SkillId],
}

impl Offering<'_> {
    /// The level of a skill the hero has, 0 for one it does not know.
    fn known(&self, skill: SkillId) -> u8 {
        let entry = self.details.skills.iter().find(|&&(s, _)| s == skill);
        entry.map_or(0, |&(_, level)| level)
    }

    /// The class's weight of a skill (0 when absent).
    fn weight(&self, skill: SkillId) -> u32 {
        let entry = self.class.skill_weights.iter().find(|&&(s, _)| s == skill);
        entry.map_or(0, |&(_, weight)| u32::from(weight))
    }

    /// Whether a level-up's offers of the skill whose interval is `interval` are due again
    /// (HER-LVL-004): the new level reached the level of its last offer plus the interval.
    fn due(&self, last: u8, interval: u8) -> bool {
        u16::from(self.level) >= u16::from(last) + u16::from(interval)
    }

    /// One offer (`hero_secondary_offer` 0x4DAD00): a skill whose level is in `window` (the
    /// lowest raised to 1 when the hero knows the most skills), not `excluded`. Wisdom outright
    /// when due and allowed; a school when due, drawn among the schools allowed (a known one
    /// weighing 1); else any candidate by weight (a known one at least 1, a banned one 0 or 1 if
    /// known).
    fn offer(
        &self,
        offered: OfferLevels,
        window: Range<u8>,
        excluded: Option<SkillId>,
        draws: &mut impl Draws,
    ) -> Option<SkillId> {
        let full = self.details.skills.len() >= MAX_HERO_SKILLS;
        let window = if full {
            window.start.max(1)
        } else {
            window.start
        }..window.end;
        let candidate =
            |skill: SkillId| window.contains(&self.known(skill)) && excluded != Some(skill);
        let banned = |skill: SkillId| self.banned.contains(&skill);
        let [wisdom_interval, school_interval] = self.class.offer_intervals;
        let wisdom = self.params.wisdom;
        if self.due(offered.wisdom, wisdom_interval) && candidate(wisdom) && !banned(wisdom) {
            return Some(wisdom);
        }
        let schools = &self.params.schools;
        if self.due(offered.school, school_interval)
            && !excluded.is_some_and(|s| schools.contains(&s))
        {
            let weighted = schools
                .iter()
                .filter(|&&s| candidate(s) && !banned(s))
                .map(|&s| (s, if self.known(s) > 0 { 1 } else { self.weight(s) }));
            if let Some(school) = draw_weighted(weighted, draws) {
                return Some(school);
            }
        }
        let count = u16::try_from(self.ruleset.skills().len()).unwrap_or(u16::MAX);
        let skills = (0..count).map(SkillId::from_raw);
        let weighted = skills.filter(|&s| candidate(s)).map(|s| {
            let known = self.known(s) > 0;
            let weight = match (banned(s), known) {
                (true, _) => u32::from(known),
                (false, true) => self.weight(s).max(1),
                (false, false) => self.weight(s),
            };
            (s, weight)
        });
        draw_weighted(weighted, draws)
    }
}

/// A draw by weight (0x4DAD00's loops): `between(1, total)`, then the skills in order, each
/// subtracting its weight until the rest is at or below 0. No weight at all draws nothing.
fn draw_weighted(
    weighted: impl Iterator<Item = (SkillId, u32)> + Clone,
    draws: &mut impl Draws,
) -> Option<SkillId> {
    let total: u32 = weighted.clone().map(|(_, w)| w).sum();
    if total == 0 {
        return None;
    }
    let mut rest = draws.between(1, total);
    let stop = weighted.into_iter().find(|&(_, weight)| {
        let reached = rest <= weight;
        rest = rest.saturating_sub(weight);
        reached
    });
    stop.map(|(skill, _)| skill)
}
