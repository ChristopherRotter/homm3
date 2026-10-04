//! Morale and luck (BAT-MOR-001..004, BAT-LCK-001/002).
//!
//! Battle-wide sources (army composition, auras) are modifiers on the sides (ADR-0012), set up
//! once when the battle starts and kept until it ends (auras persist after their bearers die). A
//! stack's morale and luck are materialised from them and its creature's minimums; the hot path
//! reads the plain fields. Rolls use the ruleset's roll tables (Tier 2).
//!
//! Source keys: fixed keys for army alignment and undead; an aura uses `Ability::aura_key`, which
//! is equal for equal aura values, so equal auras (Angels and Archangels) count once. Creature
//! minimums never change during a battle, so they are applied as `AtLeast` when the stats are
//! materialised instead of being stored in the pool.

use alloc::collections::BTreeSet;

use h3_content::{
    Ability, AuraTarget, CreatureFlag, MAX_AURA_VALUES, NO_LUCK_WITHOUT_HERO, Ruleset,
};
use h3_contracts::BattleSide;
use h3_kernel::modifier::evaluate;
use h3_kernel::{
    CanonicalHash, Handle, Modifier, ModifierOp, ModifierPool, SourceKey, StateHasher,
};

use crate::state::{BattleState, StackFlags, StackId, StackTraits, side_code};

/// Who carries a battle modifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bearer {
    /// Every stack of a side.
    Side(BattleSide),
}

impl Default for Bearer {
    fn default() -> Self {
        Self::Side(BattleSide::Attacker)
    }
}

impl CanonicalHash for Bearer {
    /// Encoding: kind (u8: 0 = side), then the side code.
    fn hash_into(&self, h: &mut StateHasher) {
        match self {
            Self::Side(side) => {
                h.write_u8(0);
                h.write_u8(side_code(*side));
            }
        }
    }
}

/// A stat that battle modifiers change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Stat {
    /// Morale (BAT-MOR-001).
    #[default]
    Morale,
    /// Luck (BAT-LCK-001).
    Luck,
}

impl CanonicalHash for Stat {
    /// Encoding: u8 (0 = morale, 1 = luck).
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_u8(match self {
            Self::Morale => 0,
            Self::Luck => 1,
        });
    }
}

/// When a battle modifier ends.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Expiry {
    /// At the end of the battle (army composition, auras).
    #[default]
    EndOfBattle,
}

impl CanonicalHash for Expiry {
    /// Encoding: u8 (0 = end of battle).
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_u8(0);
    }
}

/// Capacity of the battle's modifier pool. Per side at most: army alignment, undead in the
/// army, and one modifier per distinct aura value of the ruleset (auras with equal values
/// replace each other; the compiler caps the distinct values at [`MAX_AURA_VALUES`]).
pub const MODIFIER_CAPACITY: usize = 2 * (2 + MAX_AURA_VALUES);

/// The battle's modifiers (ADR-0012).
pub type Modifiers = ModifierPool<Bearer, Stat, Expiry, MODIFIER_CAPACITY>;

/// Source of the army-alignment morale (BAT-MOR-002).
const ARMY_ALIGNMENT: SourceKey = SourceKey(1);
/// Source of the undead-in-the-army morale (BAT-MOR-002).
const ARMY_UNDEAD: SourceKey = SourceKey(2);

/// A side-wide battle modifier.
fn side_modifier(
    side: BattleSide,
    stat: Stat,
    amount: i32,
    source: SourceKey,
) -> Modifier<Bearer, Stat, Expiry> {
    Modifier {
        bearer: Bearer::Side(side),
        stat,
        op: ModifierOp::Add,
        amount,
        source,
        expiry: Expiry::EndOfBattle,
    }
}

/// Sets up the battle-wide modifiers of a new battle and materialises every stack's morale and
/// luck: army alignment and undead (BAT-MOR-002), morale and luck auras (BAT-MOR-001,
/// BAT-LCK-001).
// rule: BAT-MOR-001, BAT-MOR-002, BAT-LCK-001
pub(crate) fn setup(state: &mut BattleState, ruleset: &Ruleset) {
    for side in BattleSide::BOTH {
        // War machines are not part of the army (VCMI counts the army's slots).
        let army = || {
            state
                .stacks
                .iter()
                .filter(move |s| s.side == side && !s.has(StackTraits::WAR_MACHINE))
        };
        // Neutral creatures (no faction) count as one alignment.
        let alignments: BTreeSet<Option<u16>> = army()
            .map(|s| {
                ruleset
                    .creature_info(s.creature)
                    .and_then(|i| i.faction)
                    .map(Handle::raw)
            })
            .collect();
        // Armies are never empty, so there is at least one alignment.
        let n = i32::try_from(alignments.len()).unwrap_or(i32::MAX);
        let alignment = if n == 1 { 1 } else { 2 - n };
        let undead = army().any(|s| {
            ruleset
                .creature(s.creature)
                .is_some_and(|c| c.flags.contains(CreatureFlag::Undead))
        });
        let mut add = |m| {
            // The pool is sized for every modifier set up here (MODIFIER_CAPACITY).
            let _ = state.modifiers.insert(m);
        };
        add(side_modifier(side, Stat::Morale, alignment, ARMY_ALIGNMENT));
        if undead {
            add(side_modifier(side, Stat::Morale, -1, ARMY_UNDEAD));
        }
    }
    let stacks = state.stacks;
    for s in stacks.iter() {
        for a in ruleset.abilities(s.creature) {
            let (stat, amount, affects) = match *a {
                Ability::MoraleAura { amount, affects } => (Stat::Morale, amount, affects),
                Ability::LuckAura { amount, affects } => (Stat::Luck, amount, affects),
                _ => continue,
            };
            let side = match affects {
                AuraTarget::Allies => s.side,
                AuraTarget::Enemies => s.side.opponent(),
            };
            let source = SourceKey(a.aura_key().unwrap_or_default());
            let _ = state
                .modifiers
                .insert(side_modifier(side, stat, i32::from(amount), source));
        }
    }
    materialize(state, ruleset);
}

/// Recomputes every stack's morale and luck from the modifiers on its side and its creature's
/// minimums, clamped to the roll tables' range: morale in `[-bad levels, good levels]`, luck in
/// `[0, good levels]` (there is no bad luck before HotA). Creatures with flag `no_morale` have
/// morale 0 (BAT-MOR-001, BAT-LCK-001).
// rule: BAT-MOR-001, BAT-LCK-001
pub(crate) fn materialize(state: &mut BattleState, ruleset: &Ruleset) {
    let params = ruleset.battle();
    let pool = state.modifiers;
    for s in state.stacks.as_mut_slice() {
        let bearer = Bearer::Side(s.side);
        let abilities = ruleset.abilities(s.creature);
        let value = |stat: Stat| {
            let minimum = abilities.iter().filter_map(move |a| match (stat, *a) {
                (Stat::Morale, Ability::MinimumMorale { value })
                | (Stat::Luck, Ability::MinimumLuck { value }) => {
                    Some((ModifierOp::AtLeast, i32::from(value)))
                }
                _ => None,
            });
            let mods = pool
                .iter()
                .filter(move |m| m.stat == stat && m.bearer == bearer)
                .map(|m| (m.op, m.amount));
            evaluate(0, mods.chain(minimum))
        };
        let morale = value(Stat::Morale).clamp(
            -i32::from(params.bad_morale.levels()),
            i32::from(params.good_morale.levels()),
        );
        let luck = value(Stat::Luck).clamp(0, i32::from(params.good_luck.levels()));
        // Both are within ±MAX_ROLL_LEVELS.
        s.morale = if s.has(StackTraits::NO_MORALE) {
            0
        } else {
            i8::try_from(morale).unwrap_or(0)
        };
        s.luck = i8::try_from(luck).unwrap_or(0);
    }
}

/// Rolls a `chance` out of `dice`.
fn roll(state: &mut BattleState, dice: u8, chance: u8) -> bool {
    state.rng.below(u32::from(dice)) < u32::from(chance)
}

/// Whether stack `id`, which just moved, attacked or shot (also in a wait phase), gets another
/// turn from good morale: positive morale, alive, no good-morale turn yet this round, and the
/// roll succeeds (BAT-MOR-003).
// rule: BAT-MOR-003
pub(crate) fn good_morale(state: &mut BattleState, ruleset: &Ruleset, id: StackId) -> bool {
    let s = state.at(id);
    if s.morale <= 0 || !s.is_alive() || s.flags.contains(StackFlags::HAD_MORALE) {
        return false;
    }
    let params = ruleset.battle();
    let chance = params.good_morale.chance(s.morale.unsigned_abs());
    roll(state, params.morale_dice, chance)
}

/// Whether stack `id`, activated in the normal phase, freezes from bad morale: negative morale
/// and the roll succeeds (BAT-MOR-004).
// rule: BAT-MOR-004
pub(crate) fn bad_morale(state: &mut BattleState, ruleset: &Ruleset, id: StackId) -> bool {
    let s = state.at(id);
    if s.morale >= 0 || s.flags.contains(StackFlags::WAITED) {
        return false;
    }
    let params = ruleset.battle();
    let chance = params.bad_morale.chance(s.morale.unsigned_abs());
    roll(state, params.morale_dice, chance)
}

/// Whether the attack of stack `id` is a lucky strike: positive luck, the side may have luck
/// (quirk Q-0003: not without a hero), and the roll succeeds (BAT-LCK-002).
// rule: BAT-LCK-002
pub(crate) fn lucky(state: &mut BattleState, ruleset: &Ruleset, id: StackId) -> bool {
    let s = state.at(id);
    let heroless = !state.has_hero(s.side);
    if s.luck <= 0 || (heroless && ruleset.quirks().is_enabled(NO_LUCK_WITHOUT_HERO)) {
        return false;
    }
    let params = ruleset.battle();
    let chance = params.good_luck.chance(s.luck.unsigned_abs());
    roll(state, params.luck_dice, chance)
}
