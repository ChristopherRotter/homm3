//! The computer's odds (0x420A80; `docs/formats/installation.md` §7.38, §7.39; BAT-AI-008):
//! whether its side is the stronger at range. Per side, over its living stacks but the blind,
//! petrified and paralysed and the turrets, the strengths (0x442E60 with the context's least
//! bonuses) of those that can shoot now, plus its hero's value ([`computer::hero_value`]: its best
//! damage spell against the side's whole strength) and, for the defender at a town with a Citadel
//! or Castle, its turrets: the keep's count `a` times a creature's AI value while the keep stands,
//! and with a Castle `trunc((a + 1) × value / 2)` for each standing tower. The side is the stronger
//! when its sum exceeds the other's. The whole strength feeds only the hero's value.
//!
//! The turrets' value follows BAT-QUIRK-25 (slip 20): the original reads the AI value of
//! creature 2 (the Archers in the harvested packs), not the turret's own; without the slip the
//! turret creature's own AI value.
//!
//! It decides three of the walker's branches (BAT-AI-005, BAT-AI-011): a plan's value alone for a
//! negative melee value, holding the walls, and giving up a losing attack for the fallbacks.

use h3_battle::computer::{self, Context};
use h3_battle::{BattleSide, BattleState, StackId, Status};
use h3_content::{CreatureTypeId, ORIGINAL_AI_SLIPS};
use h3_content_battle::{BattleRuleset, WallKind};
use h3_contracts::FortLevel;

/// The creature whose AI value the original gives a turret (0x420C59: creature 2's +0x40).
const TURRET_VALUE_CREATURE: usize = 2;

/// Whether `side` is the stronger at range in the context `ctx` (0x420A80; BAT-AI-008).
// rule: BAT-AI-008
pub(crate) fn stronger(
    state: &BattleState,
    ruleset: &BattleRuleset,
    ctx: &Context,
    side: BattleSide,
) -> bool {
    let mut range = [0i32; 2];
    for s in BattleSide::BOTH {
        let mut whole = 0i32;
        let able = (0u8..)
            .zip(state.stacks())
            .filter(|(_, t)| t.side == s && t.is_alive() && !t.is_turret())
            .filter(|(_, t)| !t.spells.status.contains(Status::NO_TURNS));
        for (i, _) in able {
            let id = StackId(i);
            let v = computer::strength(state, ruleset, ctx, id);
            whole = whole.wrapping_add(v);
            if computer::shoots(state, ruleset, id) {
                range[s.index()] = range[s.index()].wrapping_add(v);
            }
        }
        let hero = computer::hero_value(state, ruleset, s, whole);
        range[s.index()] = range[s.index()].wrapping_add(hero);
    }
    let defender = BattleSide::Defender.index();
    range[defender] = range[defender].wrapping_add(turrets(state, ruleset));
    range[side.index()] > range[side.opponent().index()]
}

/// The defender's turrets in the odds (0x420C20 to 0x420CB4): at a Citadel or Castle, with `a`
/// the keep's count and `v` the AI value (creature 2's with BAT-QUIRK-25, slip 20; else the
/// turret creature's), `a × v` while the keep stands and at a Castle `trunc((a + 1) × v / 2)` for
/// each standing tower, as 32-bit integers. The keep's count is its stack's (the siege field's
/// `turret_count` plus the town's structures, BAT-SIE-016); 0 without a turret creature.
pub(crate) fn turrets(state: &BattleState, ruleset: &BattleRuleset) -> i32 {
    let Some(siege) = computer::siege(state).filter(|s| s.fort >= FortLevel::Citadel) else {
        return 0;
    };
    let targets = (0..siege.targets).filter_map(|t| siege.kind(t).map(|k| (t, k)));
    let keep = targets
        .clone()
        .find(|&(_, k)| k == WallKind::Keep)
        .map(|(t, _)| t);
    let turret = keep
        .and_then(|t| siege.turret(t))
        .and_then(|id| state.stack(id));
    let a = turret.map_or(0, |s| i32::try_from(s.base_count).unwrap_or(i32::MAX));
    let creature = if ruleset.quirks().is_enabled(ORIGINAL_AI_SLIPS) {
        CreatureTypeId::from_index(TURRET_VALUE_CREATURE)
    } else {
        turret.map(|s| s.creature)
    };
    let v = creature
        .and_then(|c| ruleset.creature_info(c))
        .map_or(0, |info| i32::try_from(info.ai_value).unwrap_or(i32::MAX));
    let mut sum = 0i32;
    if keep.is_some_and(|t| siege.stands(t)) {
        sum = sum.wrapping_add(v.wrapping_mul(a));
    }
    if siege.fort == FortLevel::Castle {
        let towers = targets.filter(|&(t, k)| k == WallKind::Tower && siege.stands(t));
        for _ in towers {
            sum = sum.wrapping_add(v.wrapping_mul(a.wrapping_add(1)) / 2);
        }
    }
    sum
}
