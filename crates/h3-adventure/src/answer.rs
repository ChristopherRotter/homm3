//! The wandering monsters' answer to a visiting hero (ADV-MON-007, E08 T20b1; survey
//! `docs/rules/surveys/E08-battles.md` §3): what the original's `monster_visit_decision`
//! (0x4A7740) reads of the hero and the monster, its arithmetic as the executable's (the hero's
//! power, 0x427650 with 0x4E5400; the ratio in single precision; the sympathy, 0x4A75C0), and the
//! order of its tests. The state turns an answer into an offer or a fight
//! (`state/monsters.rs`).

use h3_content::{CreatureTypeId, Ruleset};
use h3_contracts::ArmyStack;
use h3_heroes::{HeroDetails, MAX_JOIN_BONUS, attack_defense, join_bonus};
use h3_kernel::arith::{DOUBLE_BITS, SINGLE_BITS, X87};

/// What a monster answers a visit with (survey §3.2), before the state checks the army's room
/// and the treasury.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Answer {
    /// It fights.
    Fight,
    /// It offers to join for free.
    Join,
    /// It offers to join for its creature's price.
    JoinForGold,
    /// It offers to flee: accepted, the hero pursues and fights it; declined, it leaves.
    Pursue,
}

/// What decides a monster's answer to one hero (survey §3.1): the charisma C = St + Sy + L (the
/// strength step, the sympathy, Diplomacy), the sympathy and Diplomacy for the offers' tests, the
/// monster's aggression M and whether it never flees.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Temper {
    charisma: i32,
    sympathy: i32,
    diplomacy: i32,
    aggression: i32,
    never_flees: bool,
}

/// A wandering monster as its answer reads it: its creature, its count as it stands, its
/// aggression and whether it never flees.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Wanderer {
    pub(crate) creature: CreatureTypeId,
    pub(crate) count: u32,
    pub(crate) aggression: i8,
    pub(crate) never_flees: bool,
}

impl Temper {
    /// The temper of `monster` toward a hero with `army` and `details` (none: Attack, Defense
    /// and Diplomacy 0), in a game of difficulty `difficulty` (ADV-MON-007; survey §3.1, the
    /// executable's 0x4A7740): St from the ratio r of the hero's power ([`power`]) to the
    /// monster's strength (its creature's AI value times its count), both stored as floats and
    /// divided in single precision: 11 from r ≥ 7, ⌊2r − 2⌋ from r ≥ 1, −1 above 0.5, −2 above
    /// 0.333, else −3 (D-8: −2 at exactly 0.5; a monster of strength 0 counts as r ≥ 7); Sy
    /// ([`sympathy`]); L the hero's join bonus (`h3_heroes::join_bonus`: Diplomacy), one higher
    /// on Easy, the difficulty 0, at most 3 (D-10: every player's, where the original raises a
    /// human player's only).
    // rule: ADV-MON-007
    pub(crate) fn new(
        core: &Ruleset,
        (army, details): (&[ArmyStack], Option<&HeroDetails>),
        monster: Wanderer,
        difficulty: u8,
    ) -> Self {
        let strength = u64::from(ai_value(core, monster.creature)) * u64::from(monster.count);
        let ratio = X87::int(power(core, army, details))
            .round(SINGLE_BITS)
            .over(X87::int(strength).round(SINGLE_BITS))
            .map(|r| r.round(SINGLE_BITS));
        let step = ratio.map_or(11, strength_step);
        let sympathy = sympathy(core, army, monster.creature);
        let bonus = details.map_or(0, |details| join_bonus(details, core));
        let diplomacy = if difficulty == 0 {
            bonus.saturating_add(1).min(MAX_JOIN_BONUS)
        } else {
            bonus
        };
        let (sympathy, diplomacy) = (i32::from(sympathy), i32::from(diplomacy));
        Self {
            charisma: step + sympathy + diplomacy,
            sympathy,
            diplomacy,
            aggression: i32::from(monster.aggression),
            never_flees: monster.never_flees,
        }
    }

    /// The answer to the visit (survey §3.2, steps 1 to 3): a fight when M > C; else an offer to
    /// join for free when M ≤ Sy + L + 1, to join for gold when M ≤ Sy + 2L + 1; else as after a
    /// declined offer to join ([`Self::after_join`]).
    // rule: ADV-MON-007
    pub(crate) const fn first(self) -> Answer {
        if self.aggression > self.charisma {
            Answer::Fight
        } else if self.aggression <= self.sympathy + self.diplomacy + 1 {
            Answer::Join
        } else if self.aggression <= self.sympathy + 2 * self.diplomacy + 1 {
            Answer::JoinForGold
        } else {
            self.after_join()
        }
    }

    /// The answer when the monster offers no join or its offer was declined (survey §3.2, step
    /// 4): a fight when M = C (an insulted monster) or the monster never flees, else the offer
    /// to flee.
    // rule: ADV-MON-007
    pub(crate) const fn after_join(self) -> Answer {
        if self.aggression == self.charisma || self.never_flees {
            Answer::Fight
        } else {
            Answer::Pursue
        }
    }
}

/// The creature's AI value, 0 for one the core does not know.
fn ai_value(core: &Ruleset, creature: CreatureTypeId) -> u32 {
    core.creature(creature).map_or(0, |info| info.ai_value)
}

/// The strength step St of a ratio r (a single-precision value), as the executable compares it
/// with its double constants (0x4A780E to 0x4A7879).
fn strength_step(ratio: X87) -> i32 {
    if ratio >= X87::int(7) {
        11
    } else if ratio >= X87::ONE {
        // fadd st, st; fsub 2.0; _ftol: exact for a float r in [1, 7), at most 11.
        i32::try_from(ratio.plus(ratio).minus(X87::int(2)).trunc()).unwrap_or(11)
    } else if ratio > X87::ratio(1, 2, DOUBLE_BITS).unwrap_or(X87::ONE) {
        -1
    } else if ratio > X87::ratio(333, 1000, DOUBLE_BITS).unwrap_or(X87::ONE) {
        -2
    } else {
        -3
    }
}

/// A hero's power against a wandering monster (survey §3.1; the executable's 0x427650): its
/// army's value (each stack's creature's AI value times its count, summed; the original's sum
/// is 32-bit and would wrap above 2³¹, which no army reaches) stored as a float, times the
/// factor √((1 + 0.05 D)(1 + 0.05 A)) of its Attack A and Defense D (`h3_heroes::attack_defense`,
/// 0 to 99; 0x4E5400: each term in double precision, the root stored as a float), truncated.
// rule: ADV-MON-007
pub(crate) fn power(core: &Ruleset, army: &[ArmyStack], details: Option<&HeroDetails>) -> u64 {
    let value = army.iter().fold(0u64, |sum, stack| {
        sum.saturating_add(u64::from(ai_value(core, stack.creature)) * u64::from(stack.count))
    });
    let (attack, defense) = details.map_or((0, 0), |details| attack_defense(details, core));
    let twentieth = X87::ratio(1, 20, DOUBLE_BITS).unwrap_or(X87::ZERO);
    let term = |skill: u8| X87::int(skill.into()).times(twentieth).plus(X87::ONE);
    let factor = term(defense)
        .times(term(attack))
        .round(DOUBLE_BITS)
        .sqrt()
        .round(SINGLE_BITS);
    X87::int(value).round(SINGLE_BITS).times(factor).trunc()
}

/// The type a monster's creature is related to for sympathy (survey §3.1; the executable's
/// 0x47B1A0 and 0x47B220 through the town creature table): none for a neutral creature (no
/// faction); else its first upgrade; else the creature it is the first upgrade of, of its
/// faction and its level. The original finds both by the creature's own level, so under
/// `BAT-QUIRK-19` (`faithful`) the Ice, Magma and Energy Elementals, whose levels differ from
/// their base creatures', have none (the Water, Earth and Fire Elementals keep them as their
/// upgrades).
fn related(core: &Ruleset, creature: CreatureTypeId) -> Option<CreatureTypeId> {
    let info = core
        .creature(creature)
        .filter(|info| info.faction.is_some())?;
    core.upgrades(creature).first().copied().or_else(|| {
        (0..core.creature_count())
            .filter_map(CreatureTypeId::from_index)
            .find(|&base| {
                core.upgrades(base).first() == Some(&creature)
                    && core
                        .creature(base)
                        .is_some_and(|b| b.faction == info.faction && b.level == info.level)
            })
    })
}

/// The hero's sympathy Sy for a monster of `creature` (survey §3.1; the executable's 0x4A75C0,
/// D-9: the monster's own type counts): 0 when no stack of the army is of the creature or its
/// related type ([`related`]), 2 when those stacks hold more than half of the army's creatures by
/// count, else 1.
// rule: ADV-MON-007
pub(crate) fn sympathy(core: &Ruleset, army: &[ArmyStack], creature: CreatureTypeId) -> u8 {
    let related = related(core, creature);
    let (similar, total) = army.iter().fold((0u64, 0u64), |(similar, total), stack| {
        let alike = stack.creature == creature || Some(stack.creature) == related;
        let count = u64::from(stack.count);
        (similar + u64::from(alike) * count, total + count)
    });
    match similar {
        0 => 0,
        _ if 2 * similar > total => 2,
        _ => 1,
    }
}

#[cfg(test)]
mod tests;
