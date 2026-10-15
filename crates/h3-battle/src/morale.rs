//! Morale and luck (BAT-MOR-001..004, BAT-LCK-001/002, BAT-HER-003; a town's, BAT-SIE-025).
//!
//! Battle-wide sources (army composition, auras, the heroes' effects) are modifiers on the sides
//! (ADR-0012), set up once when the battle starts and kept until it ends (auras persist after
//! their bearers die). A stack's morale and luck are materialised from them and its creature's
//! minimums; the hot path reads the plain fields. Rolls use the ruleset's roll tables (Tier 2).
//!
//! Source keys: fixed keys for army alignment and undead; an aura uses `Ability::aura_key`, which
//! is equal for equal aura values, so equal auras (Angels and Archangels) count once; fixed keys
//! for the heroes' changes (`Add`, both heroes summed) and caps (`AtMost`, the smallest), so a
//! change and a cap never replace each other. Creature minimums never change during a battle, so
//! they are applied as `AtLeast` when the stats are materialised instead of being stored in the
//! pool.

use alloc::collections::BTreeSet;
use alloc::vec::Vec;

use h3_content::{Affects, CreatureFlag, CreatureTypeId, NO_LUCK_WITHOUT_HERO};
use h3_content_battle::{Ability, AuraTarget, BattleRuleset, Battlefield, MAX_AURA_VALUES};
use h3_contracts::{BattleEffect, BattleHero, BattleSide, Siege};
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
/// army, one modifier per distinct aura value of the ruleset (auras with equal values replace
/// each other; the compiler caps the distinct values at [`MAX_AURA_VALUES`]), the heroes'
/// change and cap of morale and of luck (4), and a town's morale and luck (2, the defender's
/// only, counted for both sides).
pub const MODIFIER_CAPACITY: usize = 2 * (2 + MAX_AURA_VALUES + 4 + 2);

/// The battle's modifiers (ADR-0012).
pub type Modifiers = ModifierPool<Bearer, Stat, Expiry, MODIFIER_CAPACITY>;

/// Source of the army-alignment morale (BAT-MOR-002).
const ARMY_ALIGNMENT: SourceKey = SourceKey(1);
/// Source of the undead-in-the-army morale (BAT-MOR-002).
const ARMY_UNDEAD: SourceKey = SourceKey(2);
/// Source of the heroes' morale and luck changes reaching a side, summed (BAT-HER-003).
const HERO_CHANGE: SourceKey = SourceKey(3);
/// Source of the heroes' morale and luck caps reaching a side, the smallest (BAT-HER-003).
const HERO_CAP: SourceKey = SourceKey(4);
/// Source of a town's morale and luck for its defender (BAT-SIE-025).
const TOWN: SourceKey = SourceKey(5);

/// A side-wide battle modifier that adds `amount`.
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

/// A hero effect on morale or luck: the stat, whether it caps, the amount and whose stacks.
fn morale_or_luck(effect: &BattleEffect) -> Option<(Stat, bool, i8, Affects)> {
    match *effect {
        BattleEffect::Morale { amount, affects } => Some((Stat::Morale, false, amount, affects)),
        BattleEffect::Luck { amount, affects } => Some((Stat::Luck, false, amount, affects)),
        BattleEffect::MoraleAtMost { value, affects } => Some((Stat::Morale, true, value, affects)),
        BattleEffect::LuckAtMost { value, affects } => Some((Stat::Luck, true, value, affects)),
        _ => None,
    }
}

/// Whether an effect of `hero`'s hero on `affects` reaches the stacks of `bearer`.
fn reaches(affects: Affects, hero: BattleSide, bearer: BattleSide) -> bool {
    match affects {
        Affects::Own => bearer == hero,
        Affects::Enemy => bearer != hero,
        Affects::Both => true,
    }
}

/// Inserts the heroes' morale and luck effects as side modifiers: per side and stat, the sum of
/// the changes of both heroes that reach it (`Add`) and the smallest of their caps (`AtMost`)
/// (BAT-HER-003).
// rule: BAT-HER-003
fn hero_modifiers(state: &mut BattleState, heroes: &[Option<BattleHero>; 2]) {
    for bearer in BattleSide::BOTH {
        for stat in [Stat::Morale, Stat::Luck] {
            let (mut change, mut cap) = (None::<i32>, None::<i32>);
            for (side, hero) in BattleSide::BOTH.into_iter().zip(heroes) {
                let effects = hero
                    .iter()
                    .flat_map(|h| &h.effects)
                    .filter_map(morale_or_luck);
                for (s, caps, amount, affects) in effects {
                    if s != stat || !reaches(affects, side, bearer) {
                        continue;
                    }
                    let amount = i32::from(amount);
                    if caps {
                        cap = Some(cap.map_or(amount, |c| c.min(amount)));
                    } else {
                        // At most 2 × MAX_BATTLE_EFFECTS amounts of an `i8`: no overflow.
                        change = Some(change.unwrap_or(0) + amount);
                    }
                }
            }
            // The pool is sized for these (MODIFIER_CAPACITY).
            if let Some(amount) = change {
                let _ = state
                    .modifiers
                    .insert(side_modifier(bearer, stat, amount, HERO_CHANGE));
            }
            if let Some(amount) = cap {
                let _ = state.modifiers.insert(Modifier {
                    op: ModifierOp::AtMost,
                    ..side_modifier(bearer, stat, amount, HERO_CAP)
                });
            }
        }
    }
}

/// What makes an alignment for the army-alignment morale (BAT-MOR-002): a faction (`None`:
/// neutral creatures) or one of the hero's joined alignment groups (BAT-HER-007).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum AlignmentKey {
    /// A faction's raw id, `None` for neutral creatures.
    Faction(Option<u16>),
    /// The index of a joined group.
    Group(usize),
}

/// The alignment groups of `hero` (BAT-HER-007) as sets of faction raw ids, groups that share a
/// faction joined into one.
// rule: BAT-HER-007
fn alignment_groups(hero: Option<&BattleHero>) -> Vec<BTreeSet<u16>> {
    let mut joined: Vec<BTreeSet<u16>> = Vec::new();
    let effects = hero.map(|h| h.effects.as_slice()).unwrap_or_default();
    for effect in effects {
        let BattleEffect::AlignmentGroup { factions } = effect else {
            continue;
        };
        let mut group: BTreeSet<u16> = factions.factions().iter().map(|f| f.raw()).collect();
        let (overlapping, rest): (Vec<_>, Vec<_>) =
            joined.into_iter().partition(|g| !g.is_disjoint(&group));
        for g in overlapping {
            group.extend(g);
        }
        joined = rest;
        joined.push(group);
    }
    joined
}

/// Sets up the battle-wide modifiers of a new battle and materialises every stack's morale and
/// luck: army alignment and undead (BAT-MOR-002), morale and luck auras (BAT-MOR-001,
/// BAT-LCK-001), the heroes' morale and luck effects (BAT-HER-003) and the town's for its
/// defender (BAT-SIE-025).
// rule: BAT-MOR-001, BAT-MOR-002, BAT-LCK-001
pub(crate) fn setup(
    state: &mut BattleState,
    ruleset: &BattleRuleset,
    heroes: &[Option<BattleHero>; 2],
    town: Option<&Siege>,
) {
    for side in BattleSide::BOTH {
        // War machines are not part of the army (VCMI counts the army's slots).
        let army = || {
            state
                .stacks
                .iter()
                .filter(move |s| s.side == side && !s.has(StackTraits::WAR_MACHINE))
        };
        // Neutral creatures (no faction) count as one alignment; the factions of the hero's
        // alignment groups as one each (BAT-HER-007).
        let groups = alignment_groups(heroes[side.index()].as_ref());
        let alignments: BTreeSet<AlignmentKey> = army()
            .map(|s| {
                let faction = ruleset
                    .creature_info(s.creature)
                    .and_then(|i| i.faction)
                    .map(Handle::raw);
                faction
                    .and_then(|f| groups.iter().position(|g| g.contains(&f)))
                    .map_or(AlignmentKey::Faction(faction), AlignmentKey::Group)
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
    hero_modifiers(state, heroes);
    town_modifiers(state, town);
    materialize(state, ruleset);
}

/// Inserts a town's morale and luck (`Siege.morale`, `Siege.luck`: the Tavern, the Brotherhood
/// of the Sword, the Fountain of Fortune, as the town context derives them) as changes of its
/// defender's side (BAT-SIE-025; the original's 0x44AE60 and 0x44B2D0 add them last to the base
/// values, before the clamp). The original passes the town to the stacks the defender controls
/// (0x469750, after every action): here a hypnotized stack keeps its own side's, as its other
/// morale and luck (BAT-MAG-026).
// rule: BAT-SIE-025
fn town_modifiers(state: &mut BattleState, town: Option<&Siege>) {
    let Some(town) = town else {
        return;
    };
    for (stat, amount) in [(Stat::Morale, town.morale), (Stat::Luck, town.luck)] {
        if amount != 0 {
            // The pool is sized for these (MODIFIER_CAPACITY).
            let _ = state.modifiers.insert(side_modifier(
                BattleSide::Defender,
                stat,
                i32::from(amount),
                TOWN,
            ));
        }
    }
}

/// Recomputes every stack's morale and luck: the changes on its side, those of its timed
/// effects (Mirth, Sorrow, Fortune, Misfortune; BAT-MAG-011) and the battlefield's for its
/// creature's faction (BAT-TER-003) summed without a clamp, as the original's stack routines ask
/// the base routines not to clamp (0x43DF20, 0x43E000), then the side's caps and its creature's
/// minimums, then clamped once to the range the rolls read: morale to `[-bad levels, good
/// levels]`, luck to `[0, good levels]` (there is no bad luck before HotA). The original clamps
/// where it uses the value (the rolls, the stack window); the stack's fields hold the clamped
/// value, which the rolls and the observation read. Creatures with flag `no_morale` have morale
/// 0 (BAT-MOR-001, BAT-LCK-001); on a battlefield with `no_morale` or `no_luck` every stack has
/// 0 (BAT-TER-003: the original's 0x43E00F and 0x43DF2B return 0 before anything else).
// rule: BAT-MOR-001, BAT-LCK-001, BAT-MAG-011, BAT-TER-003
pub(crate) fn materialize(state: &mut BattleState, ruleset: &BattleRuleset) {
    let params = ruleset.battle();
    let pool = state.modifiers;
    let field = state.battlefield.and_then(|b| ruleset.battlefield(b));
    let good_morale = i32::from(params.good_morale.levels());
    let bad_morale = i32::from(params.bad_morale.levels());
    let good_luck = i32::from(params.good_luck.levels());
    for s in state.stacks.as_mut_slice() {
        let bearer = Bearer::Side(s.side);
        let abilities = ruleset.abilities(s.creature);
        let value = |stat: Stat, own: i32| {
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
            evaluate(own, mods.chain(minimum))
        };
        let faction = ruleset.creature_info(s.creature).and_then(|i| i.faction);
        let (field_morale, field_luck) = field.map_or((0, 0), |f| {
            (
                Battlefield::change(&f.morale, faction),
                Battlefield::change(&f.luck, faction),
            )
        });
        let morale = value(Stat::Morale, i32::from(s.spells.morale) + field_morale)
            .clamp(-bad_morale, good_morale);
        let luck = value(Stat::Luck, i32::from(s.spells.luck) + field_luck).clamp(0, good_luck);
        // Both are within ±MAX_ROLL_LEVELS.
        s.morale = if s.has(StackTraits::NO_MORALE) || field.is_some_and(|f| f.no_morale) {
            0
        } else {
            i8::try_from(morale).unwrap_or(0)
        };
        s.luck = if field.is_some_and(|f| f.no_luck) {
            0
        } else {
            i8::try_from(luck).unwrap_or(0)
        };
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
pub(crate) fn good_morale(state: &mut BattleState, ruleset: &BattleRuleset, id: StackId) -> bool {
    let s = state.at(id);
    if s.morale <= 0 || !s.is_alive() || s.flags.contains(StackFlags::HAD_MORALE) {
        return false;
    }
    let params = ruleset.battle();
    let chance = params.good_morale.chance(s.morale.unsigned_abs());
    roll(state, params.morale_dice, chance)
}

/// Whether stack `id`, picked to act outside the round's wait phase, freezes from bad morale:
/// negative morale and the roll succeeds (BAT-MOR-004). In the wait phase nobody freezes, not
/// even a stack that never waited (one whose blindness ended there): the original's pick skips
/// the check for the whole phase (+0x13DE4, the site 0x465283), as it skips Fear.
// rule: BAT-MOR-004
pub(crate) fn bad_morale(state: &mut BattleState, ruleset: &BattleRuleset, id: StackId) -> bool {
    let s = state.at(id);
    if s.morale >= 0 || state.wait_phase {
        return false;
    }
    let params = ruleset.battle();
    let chance = params.bad_morale.chance(s.morale.unsigned_abs());
    roll(state, params.morale_dice, chance)
}

/// Whether stack `id`, picked to act outside the round's wait phase, freezes from Fear
/// (BAT-MAG-060; the original's 0x464D40): unless its creature has no morale or frightens itself,
/// the living stacks with `fear` on the side opposite its controller, by their real side (a
/// hypnotized Azure Dragon frightens its controller's stacks: BAT-QUIRK-18, the rule), give one
/// roll at their largest chance.
// rule: BAT-MAG-060
pub(crate) fn frightened(state: &mut BattleState, ruleset: &BattleRuleset, id: StackId) -> bool {
    let s = state.at(id);
    if state.wait_phase || s.has(StackTraits::NO_MORALE) || s.has(StackTraits::FEAR) {
        return false;
    }
    let enemy = s.controller().opponent();
    let percent = state
        .stacks
        .iter()
        .filter(|t| t.has(StackTraits::FEAR) && t.side == enemy && t.is_alive())
        .map(|t| fear(ruleset, t.creature))
        .max()
        .unwrap_or(0);
    percent > 0 && state.rng.below(100) < u32::from(percent)
}

/// The chance of `creature`'s `fear` ability, 0 without one.
fn fear(ruleset: &BattleRuleset, creature: CreatureTypeId) -> u8 {
    ruleset
        .abilities(creature)
        .iter()
        .find_map(|a| match a {
            Ability::Fear { percent } => Some(*percent),
            _ => None,
        })
        .unwrap_or(0)
}

/// Whether the attack of stack `id` is a lucky strike: positive luck, the side may have luck
/// (quirk Q-0003: not without a hero), and the roll succeeds (BAT-LCK-002).
// rule: BAT-LCK-002
pub(crate) fn lucky(state: &mut BattleState, ruleset: &BattleRuleset, id: StackId) -> bool {
    let s = state.at(id);
    let heroless = !state.has_hero(s.side);
    if s.luck <= 0 || (heroless && ruleset.quirks().is_enabled(NO_LUCK_WITHOUT_HERO)) {
        return false;
    }
    let params = ruleset.battle();
    let chance = params.good_luck.chance(s.luck.unsigned_abs());
    roll(state, params.luck_dice, chance)
}
