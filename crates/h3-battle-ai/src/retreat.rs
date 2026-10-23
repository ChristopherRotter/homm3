//! The computer's retreat test (0x41E570 with 0x41E440 and 0x477A00;
//! `docs/formats/installation.md` §7.55, §7.60; BAT-AI-024): whether the side's hero leaves the
//! battle. The hero's cast asks it before its choice (0x422DA0: a yes makes the choice desperate,
//! BAT-AI-017), the stack's action again outside the tactics phase (0x477EE0: a yes is a retreat;
//! the computer never surrenders, 0x47815A).
//!
//! In its order: no without a hero; on Easy no, on Normal no at `random(1, 100)` ≤ 50 (the policy
//! plays every side as the computer's, ADR-0027 point 8); no when the side may not retreat (either
//! hero's Shackles of War, a town's defender without the Escape Tunnel; BAT-FLE-001,
//! BAT-SIE-024), when the scenario wants the hero defeated or no Tavern town could hire it again
//! (the setup's retreat facts, ADR-0027 point 14); yes when the side is walled out of a siege
//! ([`computer::walled_out`]); no when its artifacts are worth less than 1000 and its experience
//! is below 2000; then the expected surrender price ([`computer::retreat_price`]) and the
//! predictions with flag 1 ([`predict::plan_both`]): yes when no living stack of the side but the
//! war machines keeps hit points after the planned damage; no when the funds fall short of the
//! price and 2500 more; else yes when the side's share of the battle's Fight Value falls below its
//! threshold ([`threshold`], [`strengths`], [`below`]).

use h3_battle::computer::{self, Planned};
use h3_battle::{BattleSide, BattleState, StackFlags, StackId, StackTraits};
use h3_content_battle::BattleRuleset;
use h3_kernel::Rng;
use h3_kernel::arith::{DOUBLE_BITS, SINGLE_BITS, X87};

use crate::predict::{self, Overwrites};

/// The Normal difficulty, whose computer retreats only on half of its draws (0x41E5AE).
const NORMAL: u8 = 1;

/// The Normal difficulty's draw (0x41E5BC: `random(1, 100)`) and the highest value that answers
/// no.
const DRAW: (u32, u32) = (1, 100);
const NO_AT_MOST: u32 = 50;

/// The artifacts' worth and the experience below both of which the hero is not worth keeping
/// (0x41E820, 0x41E838).
const WORTH: u32 = 1000;
const EXPERIENCE: u32 = 2000;

/// The gold the side keeps beyond the price (0x41E8CC).
const RESERVE: i64 = 2500;

/// The records of a side the strengths walk (records 0 to 19, 0x41E8E9).
const RECORDS: usize = 20;

/// The thresholds by the artifacts' worth (0x41E9AB to 0x41E9D7, `f32` bit patterns): 0.16 at 0
/// or less, 0.2 above 0 (0x3E4CCCCC, a hair below the `f32` nearest 0.2), 0.21 above 5000, 0.22
/// above 10 000.
const THRESHOLDS: [(u32, u32); 3] = [(10_000, 0x3E61_47AE), (5000, 0x3E57_0A3D), (0, 0x3E4C_CCCC)];
const LEAST: u32 = 0x3E23_D70A;

/// The experience per step of the threshold's experience term (0x41E9FF, a division by 200 000)
/// and the term's `f32` (0x3CF5C28F, 0.03).
const EXPERIENCE_STEP: u32 = 200_000;
const EXPERIENCE_TERM: u32 = 0x3CF5_C28F;

/// The attacker's term (0x63AC00, `f32` 0.06).
const ATTACKER_TERM: u32 = 0x3D75_C28F;

/// The highest difficulty (Impossible), the base of the difficulty's term (0x41E9E4).
const IMPOSSIBLE: u8 = 4;

/// The retreat test's answer for `side` (0x41E570; BAT-AI-024), its draws from `rng`: the Normal
/// difficulty's and the predictions' walkers'. With `slips` the flag-1 predictions are kept when
/// the test got that far, for the hero's choice that follows (survey slip 26).
// rule: BAT-AI-024
pub(crate) fn test(
    state: &BattleState,
    ruleset: &BattleRuleset,
    side: BattleSide,
    rng: &mut Rng,
    slips: bool,
) -> Answer {
    let no = Answer::default();
    if state.hero(side).is_none() {
        return no;
    }
    let difficulty = computer::difficulty(state);
    if difficulty == 0
        || (difficulty == NORMAL && rng.range_inclusive(DRAW.0, DRAW.1) <= NO_AT_MOST)
    {
        return no;
    }
    let facts = computer::retreat_facts(state, side);
    if !state.may_retreat(side) || facts.wanted || !facts.tavern {
        return no;
    }
    if computer::walled_out(state, ruleset, side) {
        return Answer::yes(None);
    }
    if facts.artifacts < WORTH && facts.experience < EXPERIENCE {
        return no;
    }
    let price = computer::retreat_price(state, ruleset, side);
    let mut overwrites = Overwrites::default();
    let planned = predict::plan_both(state, ruleset, side, rng, slips.then_some(&mut overwrites));
    let predicted = slips.then_some(Predicted {
        planned,
        overwrites,
    });
    if beaten(state, side, &planned) {
        return Answer::yes(predicted);
    }
    let answer = |retreat| Answer { retreat, predicted };
    if i64::from(computer::funds(state, side)) < i64::from(price) + RESERVE {
        return answer(false);
    }
    let x = threshold(difficulty, side, facts.artifacts, facts.experience);
    let sums = strengths(state, ruleset, side);
    let (own, other) = (sums[side.index()], sums[side.opponent().index()]);
    answer(below(own, other, x))
}

/// The retreat test's answer (0x41E570's return) and, when it got as far as its predictions and
/// the original's slips are kept, what they left (survey slip 26).
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct Answer {
    /// The side retreats.
    pub(crate) retreat: bool,
    /// The flag-1 predictions' outcome.
    pub(crate) predicted: Option<Predicted>,
}

impl Answer {
    const fn yes(predicted: Option<Predicted>) -> Self {
        Self {
            retreat: true,
            predicted,
        }
    }
}

/// What the retreat test's predictions left on the stacks (survey slip 26): the damage planned
/// by both sides' runs and the last overwrite of each side's threat records.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Predicted {
    /// The damage planned (+0x534).
    pub(crate) planned: Planned,
    /// The last overwrite of each side's threat records.
    pub(crate) overwrites: Overwrites,
}

/// Whether the damage `planned` leaves `side` no living stack but the war machines (the turrets
/// among them) with hit points (0x41E85F to 0x41E8BB; the original's records of the side in
/// use, 0x443080 with flag 1).
fn beaten(state: &BattleState, side: BattleSide, planned: &Planned) -> bool {
    !(0u8..).zip(state.stacks()).any(|(i, s)| {
        s.side == side
            && s.is_alive()
            && !s.has(StackTraits::WAR_MACHINE)
            && !s.is_turret()
            && planned.left(state, StackId(i)) > 0
    })
}

/// The `f32` of the bit pattern `bits`, exact.
fn single(bits: u32) -> X87 {
    X87::from_f32_bits(bits)
}

/// The double nearest `num / den` (a constant of the code).
fn double(num: u32, den: u32) -> X87 {
    X87::ratio(num, den, DOUBLE_BITS).unwrap_or(X87::ZERO)
}

/// The retreat threshold `x` of `side` at `difficulty` with its artifacts' worth `artifacts` and
/// `experience` (0x41E9A8 to 0x41EA73, each step stored as an `f32`): 0.16, 0.2 above a worth of
/// 0, 0.21 above 5000, 0.22 above 10 000; less (4 − the difficulty) × 0.015 (a double); plus 0.03
/// when ⌊experience / 200 000⌋ exceeds 0.03 (as a double: from 200 000 experience on); less 0.06
/// for the attacker; at most 0.16.
pub(crate) fn threshold(difficulty: u8, side: BattleSide, artifacts: u32, experience: u32) -> X87 {
    let bits = THRESHOLDS
        .iter()
        .find(|&&(above, _)| artifacts > above)
        .map_or(LEAST, |&(_, bits)| bits);
    let steps = IMPOSSIBLE.saturating_sub(difficulty);
    let term = X87::int(u64::from(steps)).times(double(15, 1000));
    let mut x = single(bits).minus(term).round(SINGLE_BITS);
    let k = X87::int(u64::from(experience / EXPERIENCE_STEP)).round(SINGLE_BITS);
    let k = if k > double(3, 100) {
        single(EXPERIENCE_TERM)
    } else {
        k
    };
    x = k.plus(x).round(SINGLE_BITS);
    if side == BattleSide::Attacker {
        x = x.minus(single(ATTACKER_TERM)).round(SINGLE_BITS);
    }
    if x > double(16, 100) {
        single(LEAST)
    } else {
        x
    }
}

/// `v × f` truncated as the original's `_ftol` of a double gives it (0x41E92A, 0x41E966): the
/// low 32 bits, a negative `v` by its magnitude.
pub(crate) fn scaled(v: i32, f: X87) -> i32 {
    let m = computer::ftol(X87::int(u64::from(v.unsigned_abs())).times(f));
    if v < 0 { m.wrapping_neg() } else { m }
}

/// Each side's strength for the retreat test (0x41E8DD to 0x41E9A6), by side index: over its
/// first [`RECORDS`] stacks with creatures, the creatures times the creature's Fight Value (a
/// 32-bit product), 1.2 times (a double, truncated) for one that has not acted this round,
/// summed as 32-bit integers; at a town the defender's sum 1.1 times; then the other side's than
/// `side` 1.1 times in every battle.
pub(crate) fn strengths(
    state: &BattleState,
    ruleset: &BattleRuleset,
    side: BattleSide,
) -> [i32; 2] {
    let (more, most) = (double(11, 10), double(12, 10));
    let mut sums = [0i32; 2];
    for b in BattleSide::BOTH {
        let mut sum = 0i32;
        for s in state
            .stacks()
            .iter()
            .filter(|s| s.side == b)
            .take(RECORDS)
            .filter(|s| i32::try_from(s.count).is_ok_and(|c| c > 0))
        {
            let fight = ruleset
                .creature_info(s.creature)
                .map_or(0, |info| info.fight_value);
            let count = i32::try_from(s.count).unwrap_or(i32::MAX);
            let v = count.wrapping_mul(i32::try_from(fight).unwrap_or(i32::MAX));
            let v = if s.flags.contains(StackFlags::ACTED) {
                v
            } else {
                scaled(v, most)
            };
            sum = sum.wrapping_add(v);
        }
        sums[b.index()] = if b == BattleSide::Defender && computer::siege(state).is_some() {
            scaled(sum, more)
        } else {
            sum
        };
    }
    let other = side.opponent().index();
    sums[other] = scaled(sums[other], more);
    sums
}

/// Whether a side of strength `own` holds a share of the battle's strength below `x`, `other` the
/// other side's ([`strengths`]; 0x41EA73 to 0x41EAA2): `own` and the sum (32 bits) each as an
/// `f32`, their quotient stored as one, below `x`; a quotient of 0 / 0 (unordered) counts as
/// below, `own / 0` is an infinity of `own`'s sign.
pub(crate) fn below(own: i32, other: i32, x: X87) -> bool {
    let total = own.wrapping_add(other);
    if own == 0 || ((own < 0) != (total < 0) && total != 0) {
        // 0 / 0 is unordered, 0 / t and a negative quotient lie below a threshold above 0.
        return true;
    }
    let magnitude = |v: i32| X87::int(u64::from(v.unsigned_abs())).round(SINGLE_BITS);
    match magnitude(own).over(magnitude(total)) {
        Some(q) => q.round(SINGLE_BITS) < x,
        None => own < 0,
    }
}
