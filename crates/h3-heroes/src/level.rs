//! Experience to levels (E08 T27): a level's threshold and the level of an experience
//! (HER-EXP-002), the bound that makes level 74 the highest (A-7, `ADV-QUIRK-11`), the scenario's
//! level limit and the experience a gain gives under it (HER-EXP-003; A-8, `ADV-QUIRK-12`), and
//! the levels a hero has yet to be given. Survey `docs/rules/surveys/E08-progression.md` §1,
//! design §2, §3 of increment 4's note (since T34b in `docs/epics/E08-log.md`, "Increment 4's
//! design note at its closure").

use h3_content::HeroParams;

use crate::details::HeroDetails;

/// The experience a hero needs for `level` (HER-EXP-002): the ruleset's table of levels 1 to
/// `max_level + 1` (the pack's levels, then each step the previous one + a fifth of it, as the
/// original's truncated × 1.2, survey §1.1). Level 0 counts as 1; a level beyond the table as
/// its last entry.
pub fn experience_for(params: &HeroParams, level: u8) -> u32 {
    let at = usize::from(level.max(1)) - 1;
    let last = params.experience.last().copied().unwrap_or(0);
    params.experience.get(at).copied().unwrap_or(last)
}

/// The most experience a hero may have (HER-EXP-002, A-7): the threshold of the level after the
/// highest, less one (1 810 034 206 with the original's table), so `max_level` (74) is the
/// highest level. The original adds without a bound: from level 75's threshold its levels wrap
/// (88, 100, …), at 2³¹ − 1 its level loop never ends, and past it the experience turns negative
/// (`ADV-QUIRK-11`, fixed).
pub fn experience_bound(params: &HeroParams) -> u32 {
    experience_for(params, params.max_level.saturating_add(1)).saturating_sub(1)
}

/// The level of `experience` (HER-EXP-002): the number of thresholds at or below it (the
/// original's loop, 0x4DA743 to 0x4DA7C6), at most `max_level`.
pub fn level_of(params: &HeroParams, experience: u32) -> u8 {
    // rule: HER-EXP-002
    let reached = params
        .experience
        .iter()
        .filter(|&&t| t <= experience)
        .count();
    u8::try_from(reached)
        .unwrap_or(u8::MAX)
        .min(params.max_level)
}

/// The most experience a gain gives under the scenario's level limit (HER-EXP-003): the
/// threshold of the limit when it is 1 to `max_level`; a limit of 0 or above `max_level` is none
/// (A-8: the original computes a wrapped, often negative cap for a limit above 75,
/// `ADV-QUIRK-12`, fixed), and the cap is then the bound.
pub fn experience_cap(params: &HeroParams, level_limit: u8) -> u32 {
    // rule: HER-EXP-003
    if (1..=params.max_level).contains(&level_limit) {
        experience_for(params, level_limit)
    } else {
        experience_bound(params)
    }
}

/// The hero's experience after it gains `points` under the scenario's level limit (HER-EXP-003,
/// `hero_gain_experience` 0x4E33B0): the sum, at most the cap ([`experience_cap`]); experience
/// already above the cap is lowered to it, by any gain (D-19, 0x4E347E). The engine lowers it no
/// further than the threshold of the hero's level, which the details' validation keeps at most
/// the experience's: a hero whose level exceeds the limit, which the original keeps at its level
/// with less experience, cannot arise when the game gives every hero its experience under the
/// limit (D-21, increment 4's design note §3).
pub fn gained_under_limit(
    details: &HeroDetails,
    params: &HeroParams,
    points: u32,
    level_limit: u8,
) -> u32 {
    let cap = experience_cap(params, level_limit).max(experience_for(params, details.level));
    details.experience.saturating_add(points).min(cap)
}

/// The levels the hero's experience gives beyond those it has been given: its pending level-ups,
/// which the game gives one at a time (increment 4's design note §7).
pub fn pending_levels(details: &HeroDetails, params: &HeroParams) -> u8 {
    level_of(params, details.experience).saturating_sub(details.level)
}
