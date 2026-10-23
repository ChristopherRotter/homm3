//! The computer's retreat estimates its retreat test reads from the battle's private state
//! (`h3-battle-ai`'s retreat test, 0x41E570; `docs/formats/installation.md` §7.55, §7.60;
//! BAT-AI-024): the price it expects a surrender to cost (0x477A00) and whether its side's army
//! is walled out of a siege (0x41E440). No rule reads them (ADR-0027 point 6).

use h3_content::{FULL_BP, SINGLE_PRECISION_RATES};
use h3_content_battle::{BattleRuleset, WallKind};
use h3_contracts::{BattleSide, FortLevel};
use h3_kernel::arith::{DOUBLE_BITS, SINGLE_BITS, X87, single_precision};

use crate::combat::can_shoot;
use crate::geometry::Field;
use crate::siege;
use crate::state::{BattleState, StackId, StackTraits};

/// The records of a side the original's price and strengths walk (records 0 to 19).
pub(crate) const PRICED_RECORDS: usize = 20;

/// The surrender price `side`'s computer expects (0x477A00; BAT-AI-024): over its first
/// [`PRICED_RECORDS`] stacks, those alive and not created in the battle (summoned stacks and
/// clones) with more creatures than the battle's temporary ones (Basic Resurrection's,
/// BAT-MAG-035) add their creatures but the temporary ones times the creature's cost in the
/// ruleset's surrender resource, a war machine's and a turret's too, summed as the original's
/// 32-bit integers; the sum times the ruleset's surrender share, truncated (the original
/// halves), as an `f32`, times one minus the hero's surrender discount (an `f32` under quirk
/// Q-0005, as the price's rule takes it, BAT-FLE-002), truncated. 0 when the ruleset prices no
/// surrender. Unlike the battle's price (BAT-FLE-002) it leaves the temporary creatures out and
/// takes the share before the discount (BAT-Q28 row 7: the original's price, which the
/// computer's estimate is).
// rule: BAT-AI-024
pub(crate) fn price(state: &BattleState, ruleset: &BattleRuleset, side: BattleSide) -> i32 {
    let Some(params) = ruleset.battle().surrender else {
        return 0;
    };
    let cost = |s: &crate::state::Stack| {
        ruleset.creature_info(s.creature).map_or(0i32, |info| {
            info.cost
                .iter()
                .filter(|(resource, _)| *resource == params.resource)
                .fold(0i32, |sum, &(_, amount)| {
                    sum.wrapping_add(i32::try_from(amount).unwrap_or(i32::MAX))
                })
        })
    };
    let mut real = 0i32;
    for (i, s) in (0u8..)
        .zip(state.stacks.iter())
        .filter(|(_, s)| s.side == side)
        .take(PRICED_RECORDS)
    {
        let temporary = state.raised.temporary[usize::from(i)];
        if s.is_created() || s.count <= temporary {
            continue;
        }
        let creatures = i32::try_from(s.count - temporary).unwrap_or(i32::MAX);
        real = real.wrapping_add(cost(s).wrapping_mul(creatures));
    }
    // The share as the original's halving, towards 0.
    let share = i64::from(real) * i64::from(params.cost_percent) / 100;
    let discount = state.heroes[side.index()].map_or(0, |h| h.surrender_discount_bp);
    let factor = if ruleset.quirks().is_enabled(SINGLE_PRECISION_RATES) {
        let rate = single_precision(discount, FULL_BP).map_or(X87::ZERO, X87::fixed);
        X87::ONE.minus(rate)
    } else {
        X87::ratio(
            u32::from(FULL_BP.saturating_sub(discount)),
            u32::from(FULL_BP),
            DOUBLE_BITS,
        )
        .unwrap_or(X87::ZERO)
    };
    let magnitude = X87::int(share.unsigned_abs())
        .round(SINGLE_BITS)
        .times(factor)
        .trunc();
    let magnitude = i32::try_from(magnitude).unwrap_or(i32::MAX);
    if share < 0 { -magnitude } else { magnitude }
}

/// Whether `side`'s army is walled out of the siege (0x41E440; BAT-AI-024): `side` is the
/// attacker at a town with walls whose gate is closed, none of its living stacks flies, shoots
/// walls (the Catapult, a stack with `wall_shots`) or can shoot now, every wall stands (the
/// wall-shot targets of kind wall) and every living defender stack but the turrets stands inside
/// the walls (the original's 0x467460 at its position hex).
// rule: BAT-AI-024
pub(crate) fn walled_out(state: &BattleState, ruleset: &BattleRuleset, side: BattleSide) -> bool {
    let Some(s) = state.siege.as_ref().filter(|s| s.fort >= FortLevel::Fort) else {
        return false;
    };
    if s.gate_open || side != BattleSide::Attacker {
        return false;
    }
    let field = Field::new(ruleset.battle().field);
    let free = (0u8..).zip(state.stacks.iter()).all(|(i, o)| {
        o.side != side
            || !o.is_alive()
            || !(o.has(StackTraits::FLIES)
                || siege::shooter_level(state, ruleset, StackId(i)).is_some()
                || can_shoot(state, field, StackId(i)))
    });
    let walls = (0..s.targets)
        .filter(|&t| s.kind(t) == Some(WallKind::Wall))
        .all(|t| s.stands(t));
    free && walls
        && state
            .stacks
            .iter()
            .filter(|o| o.side == BattleSide::Defender && o.is_alive() && !o.is_turret())
            .all(|o| s.inside(o.hex))
}
