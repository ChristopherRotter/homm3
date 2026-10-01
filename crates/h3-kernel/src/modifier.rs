//! Modifiers: numeric changes to a stat of a bearer (ADR-0012).
//!
//! A [`Modifier`] says "change stat `S` of bearer `B` by this operation and amount, because of
//! source `K`, until expiry event `E`". Contexts define what bearers, stats and expiry tags are;
//! the kernel stores modifiers in a fixed-capacity [`ModifierPool`] (no allocation) and evaluates
//! a stat in fixed stages whose aggregation (`+`, `max`, `min`) does not depend on the order of
//! the modifiers.
//!
//! Stacking: a source contributes at most one modifier per bearer and stat. Inserting a modifier
//! with an existing `(bearer, stat, source)` replaces the old one.

use crate::fixed::{CapacityError, FixedVec};
use crate::hash::{CanonicalHash, StateHasher};

/// How a modifier changes a stat.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum ModifierOp {
    /// Adds the amount (stage 1).
    #[default]
    Add,
    /// Adds the amount in percent of the stage-1 value, rounded down (stage 2).
    AddPercent,
    /// The value is at least the amount (stage 3).
    AtLeast,
    /// The value is at most the amount (stage 4; wins over `AtLeast`).
    AtMost,
}

impl ModifierOp {
    const fn code(self) -> u8 {
        match self {
            Self::Add => 0,
            Self::AddPercent => 1,
            Self::AtLeast => 2,
            Self::AtMost => 3,
        }
    }
}

/// Identifies where a modifier comes from (an ability kind, a spell, an artifact, …). Context-
/// defined; equal keys on the same bearer and stat do not stack.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct SourceKey(pub u32);

/// One modifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Modifier<B, S, E> {
    /// Who carries it (context-defined: a stack, a side, …).
    pub bearer: B,
    /// Which stat it changes.
    pub stat: S,
    /// How.
    pub op: ModifierOp,
    /// By how much.
    pub amount: i32,
    /// Where it comes from (stacking key).
    pub source: SourceKey,
    /// When it ends (context-defined tag, never interpreted by the kernel).
    pub expiry: E,
}

impl<B: CanonicalHash, S: CanonicalHash, E: CanonicalHash> CanonicalHash for Modifier<B, S, E> {
    /// Encoding: bearer, stat, op code (u8), amount (i32), source (u32), expiry.
    fn hash_into(&self, h: &mut StateHasher) {
        self.bearer.hash_into(h);
        self.stat.hash_into(h);
        h.write_u8(self.op.code());
        h.write_i32(self.amount);
        h.write_u32(self.source.0);
        self.expiry.hash_into(h);
    }
}

/// Evaluates a stat from its base value and the `(op, amount)` pairs that apply to it
/// (rule KER-MOD-001): `flat = base + Σ Add`; `value = flat + floor(flat · Σ AddPercent / 100)`;
/// then raised to the largest `AtLeast` and lowered to the smallest `AtMost`; saturated to `i32`.
/// The result does not depend on the order of `mods`.
// rule: KER-MOD-001
pub fn evaluate(base: i32, mods: impl IntoIterator<Item = (ModifierOp, i32)>) -> i32 {
    let mut add = 0i64;
    let mut percent = 0i64;
    let mut at_least: Option<i64> = None;
    let mut at_most: Option<i64> = None;
    for (op, amount) in mods {
        let a = i64::from(amount);
        match op {
            ModifierOp::Add => add = add.saturating_add(a),
            ModifierOp::AddPercent => percent = percent.saturating_add(a),
            ModifierOp::AtLeast => at_least = Some(at_least.map_or(a, |v| v.max(a))),
            ModifierOp::AtMost => at_most = Some(at_most.map_or(a, |v| v.min(a))),
        }
    }
    let flat = i64::from(base).saturating_add(add);
    let scaled = flat.saturating_mul(percent).div_euclid(100);
    let mut value = flat.saturating_add(scaled);
    value = at_least.map_or(value, |lo| value.max(lo));
    value = at_most.map_or(value, |hi| value.min(hi));
    i32::try_from(value.clamp(i64::from(i32::MIN), i64::from(i32::MAX))).unwrap_or_default()
}

/// A fixed-capacity set of modifiers (ADR-0012 §2). Insertion order is kept (it is part of the
/// canonical hash) but never affects evaluation.
#[derive(Clone, Copy)]
pub struct ModifierPool<B, S, E, const N: usize>
where
    B: Copy + Default,
    S: Copy + Default,
    E: Copy + Default,
{
    entries: FixedVec<Modifier<B, S, E>, N>,
}

impl<B, S, E, const N: usize> Default for ModifierPool<B, S, E, N>
where
    B: Copy + Default,
    S: Copy + Default,
    E: Copy + Default,
{
    fn default() -> Self {
        Self {
            entries: FixedVec::new(),
        }
    }
}

impl<B, S, E, const N: usize> ModifierPool<B, S, E, N>
where
    B: Copy + Default + Eq,
    S: Copy + Default + Eq,
    E: Copy + Default,
{
    /// An empty pool.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds `modifier`, replacing an existing modifier with the same bearer, stat and source
    /// (stacking rule). Fails only if the pool is full and nothing is replaced.
    pub fn insert(&mut self, modifier: Modifier<B, S, E>) -> Result<(), CapacityError> {
        let same = |m: &&mut Modifier<B, S, E>| {
            m.bearer == modifier.bearer && m.stat == modifier.stat && m.source == modifier.source
        };
        match self.entries.as_mut_slice().iter_mut().find(same) {
            Some(existing) => {
                *existing = modifier;
                Ok(())
            }
            None => self.entries.push(modifier),
        }
    }

    /// Removes every modifier for which `remove` returns true, keeping the order of the rest.
    /// Returns how many were removed.
    pub fn remove_if(&mut self, remove: impl FnMut(&Modifier<B, S, E>) -> bool) -> usize {
        self.entries.remove_if(remove)
    }

    /// Evaluates `stat` from `base` with the modifiers of bearers for which `applies` is true
    /// ([`evaluate`]).
    pub fn evaluate(&self, stat: S, base: i32, mut applies: impl FnMut(&B) -> bool) -> i32 {
        evaluate(
            base,
            self.entries
                .iter()
                .filter(|m| m.stat == stat && applies(&m.bearer))
                .map(|m| (m.op, m.amount)),
        )
    }

    /// The modifiers, in insertion order.
    pub fn iter(&self) -> core::slice::Iter<'_, Modifier<B, S, E>> {
        self.entries.iter()
    }

    /// Number of modifiers.
    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the pool is empty.
    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Maximum number of modifiers.
    pub const fn capacity(&self) -> usize {
        N
    }
}

impl<B, S, E, const N: usize> PartialEq for ModifierPool<B, S, E, N>
where
    B: Copy + Default + PartialEq,
    S: Copy + Default + PartialEq,
    E: Copy + Default + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.entries == other.entries
    }
}

impl<B, S, E, const N: usize> Eq for ModifierPool<B, S, E, N>
where
    B: Copy + Default + Eq,
    S: Copy + Default + Eq,
    E: Copy + Default + Eq,
{
}

impl<B, S, E, const N: usize> core::fmt::Debug for ModifierPool<B, S, E, N>
where
    B: Copy + Default + core::fmt::Debug,
    S: Copy + Default + core::fmt::Debug,
    E: Copy + Default + core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.entries.fmt(f)
    }
}

impl<B, S, E, const N: usize> CanonicalHash for ModifierPool<B, S, E, N>
where
    B: Copy + Default + CanonicalHash,
    S: Copy + Default + CanonicalHash,
    E: Copy + Default + CanonicalHash,
{
    /// Encoding: the modifiers as a slice (length, then each in insertion order).
    fn hash_into(&self, h: &mut StateHasher) {
        self.entries.hash_into(h);
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use crate::hash::hash_of;
    use proptest::prelude::*;
    use std::format;
    use std::vec::Vec;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    struct Unit(u8);
    impl CanonicalHash for Unit {
        fn hash_into(&self, h: &mut StateHasher) {
            h.write_u8(self.0);
        }
    }

    type Pool = ModifierPool<Unit, Unit, Unit, 4>;

    fn m(
        bearer: u8,
        stat: u8,
        op: ModifierOp,
        amount: i32,
        source: u32,
    ) -> Modifier<Unit, Unit, Unit> {
        Modifier {
            bearer: Unit(bearer),
            stat: Unit(stat),
            op,
            amount,
            source: SourceKey(source),
            expiry: Unit(0),
        }
    }

    // covers: KER-MOD-001
    #[test]
    fn stages() {
        use ModifierOp::{Add, AddPercent, AtLeast, AtMost};
        assert_eq!(evaluate(10, []), 10);
        assert_eq!(evaluate(10, [(Add, 3), (Add, -1)]), 12);
        // Percent applies to the stage-1 value and rounds down (also for negative results).
        assert_eq!(evaluate(10, [(Add, 3), (AddPercent, 20)]), 13 + 2);
        assert_eq!(evaluate(7, [(AddPercent, -50)]), 7 - 4);
        assert_eq!(evaluate(-3, [(AddPercent, 50)]), -3 - 2);
        // Lower and upper bounds; the upper bound wins.
        assert_eq!(evaluate(0, [(AtLeast, 1), (AtLeast, -2)]), 1);
        assert_eq!(evaluate(5, [(AtMost, 3), (AtMost, 4)]), 3);
        assert_eq!(evaluate(0, [(AtLeast, 2), (AtMost, 1)]), 1);
        assert_eq!(evaluate(5, [(AtLeast, 2)]), 5);
        // Saturation instead of overflow.
        assert_eq!(evaluate(i32::MAX, [(Add, i32::MAX)]), i32::MAX);
        assert_eq!(evaluate(i32::MIN, [(Add, i32::MIN)]), i32::MIN);
        assert_eq!(evaluate(i32::MAX, [(AddPercent, i32::MAX)]), i32::MAX);
    }

    // covers: KER-MOD-001
    #[test]
    fn pool_stacking_removal_and_evaluation() {
        use ModifierOp::{Add, AtLeast};
        let mut pool = Pool::new();
        assert!(pool.is_empty());
        assert_eq!(pool.capacity(), 4);
        pool.insert(m(1, 0, Add, 2, 7)).unwrap();
        // Same bearer, stat and source: replaced, not stacked.
        pool.insert(m(1, 0, Add, 3, 7)).unwrap();
        assert_eq!(pool.len(), 1);
        // Another source stacks; another bearer and another stat are separate.
        pool.insert(m(1, 0, Add, 1, 8)).unwrap();
        pool.insert(m(2, 0, AtLeast, 50, 7)).unwrap();
        pool.insert(m(1, 1, Add, 9, 7)).unwrap();
        assert_eq!(
            pool.insert(m(3, 0, Add, 1, 1)),
            Err(CapacityError { capacity: 4 })
        );
        // A replacement still works when full.
        pool.insert(m(1, 1, Add, 4, 7)).unwrap();
        assert_eq!(pool.evaluate(Unit(0), 10, |b| b.0 == 1), 14);
        assert_eq!(pool.evaluate(Unit(0), 10, |_| true), 50);
        assert_eq!(pool.evaluate(Unit(1), 10, |b| b.0 == 1), 14);
        assert_eq!(pool.evaluate(Unit(0), 10, |_| false), 10);
        assert_eq!(
            pool.remove_if(|x| x.source == SourceKey(7) && x.stat == Unit(0)),
            2
        );
        assert_eq!(
            pool.iter().map(|x| x.amount).collect::<Vec<_>>(),
            [1, 4],
            "order of the rest is kept"
        );
        assert_eq!(format!("{pool:?}").matches("Modifier").count(), 2);
    }

    #[test]
    fn hashing_and_equality() {
        let mut a = Pool::new();
        a.insert(m(1, 2, ModifierOp::AtMost, -1, 3)).unwrap();
        let mut b = Pool::new();
        b.insert(m(1, 2, ModifierOp::AtMost, -1, 3)).unwrap();
        assert_eq!(a, b);
        assert_eq!(hash_of(&a), hash_of(&b));
        let mut h = StateHasher::new();
        h.write_len(1);
        h.write_u8(1);
        h.write_u8(2);
        h.write_u8(3);
        h.write_i32(-1);
        h.write_u32(3);
        h.write_u8(0);
        assert_eq!(hash_of(&a), h.finish());
        b.insert(m(1, 2, ModifierOp::AtLeast, -1, 3)).unwrap();
        assert_ne!(a, b);
        assert_ne!(hash_of(&a), hash_of(&b));
        let codes = [
            ModifierOp::Add,
            ModifierOp::AddPercent,
            ModifierOp::AtLeast,
            ModifierOp::AtMost,
        ]
        .map(ModifierOp::code);
        assert_eq!(codes, [0, 1, 2, 3]);
        assert_eq!(ModifierOp::default(), ModifierOp::Add);
    }

    fn arb_op() -> impl Strategy<Value = ModifierOp> {
        prop_oneof![
            Just(ModifierOp::Add),
            Just(ModifierOp::AddPercent),
            Just(ModifierOp::AtLeast),
            Just(ModifierOp::AtMost),
        ]
    }

    proptest! {
        // covers: KER-MOD-001
        #[test]
        fn evaluation_is_order_independent(
            base in -1000i32..1000,
            mods in proptest::collection::vec((arb_op(), -500i32..500), 0..12),
            rotation in 0usize..12,
        ) {
            let forward = evaluate(base, mods.iter().copied());
            let backward = evaluate(base, mods.iter().rev().copied());
            let mut rotated = mods.clone();
            if !rotated.is_empty() {
                let k = rotation % rotated.len();
                rotated.rotate_left(k);
            }
            prop_assert_eq!(forward, backward);
            prop_assert_eq!(forward, evaluate(base, rotated));
        }
    }
}
