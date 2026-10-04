//! Quirk flags (ADR-0007, docs/rules/quirks-ledger.md).
//!
//! A quirk flag toggles a known deviation of the original game. Flags that change behavior need
//! code that reads them, so the set of flags is a closed registry in code ([`REGISTRY`]); profiles
//! (data) choose which flags are on. `exploit-fix` quirks never get a flag.

/// Maximum number of quirk flags (they are stored as a bitset).
pub const MAX_QUIRKS: usize = 64;

/// A registered quirk flag.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct QuirkFlag(u8);

impl QuirkFlag {
    /// The flag's bit index.
    pub const fn index(self) -> u8 {
        self.0
    }
}

/// The closed list of quirk flags known to the engine: names like `quirk.battle.<name>`, with the
/// flag's index equal to its position.
#[derive(Debug, Clone, Copy)]
pub struct QuirkRegistry {
    names: &'static [&'static str],
}

impl QuirkRegistry {
    /// A registry over `names` (at most [`MAX_QUIRKS`] entries are addressable).
    pub const fn new(names: &'static [&'static str]) -> Self {
        Self { names }
    }

    /// Looks a flag up by name.
    pub fn lookup(&self, name: &str) -> Option<QuirkFlag> {
        self.names
            .iter()
            .take(MAX_QUIRKS)
            .position(|n| *n == name)
            .and_then(|i| u8::try_from(i).ok())
            .map(QuirkFlag)
    }

    /// Number of registered flags.
    pub fn len(&self) -> usize {
        self.names.len().min(MAX_QUIRKS)
    }

    /// Whether no flags are registered.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// The engine's quirk registry. Add entries together with the ledger entry, the rule and its
/// tests; a flag's index is its position here and must never change.
pub const REGISTRY: QuirkRegistry = QuirkRegistry::new(&[
    "quirk.battle.no_luck_without_hero",
    "quirk.battle.single_precision_rates",
]);

/// Q-0003: creatures of a side without a hero never get lucky strikes (BAT-LCK-002).
pub const NO_LUCK_WITHOUT_HERO: QuirkFlag = QuirkFlag(0);

/// Q-0005: the damage factors use the single-precision values of their rates (BAT-DMG-002).
pub const SINGLE_PRECISION_RATES: QuirkFlag = QuirkFlag(1);

/// The set of enabled quirk flags of a ruleset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Quirks {
    bits: u64,
}

impl Quirks {
    /// Whether `flag` is enabled.
    pub const fn is_enabled(self, flag: QuirkFlag) -> bool {
        self.bits & (1u64 << flag.0) != 0
    }

    /// Returns the set with `flag` set to `on`.
    pub const fn with(self, flag: QuirkFlag, on: bool) -> Self {
        let mask = 1u64 << flag.0;
        Self {
            bits: if on {
                self.bits | mask
            } else {
                self.bits & !mask
            },
        }
    }

    /// The raw bitset (for hashing and replay headers).
    pub const fn bits(self) -> u64 {
        self.bits
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST: QuirkRegistry = QuirkRegistry::new(&["quirk.battle.a", "quirk.battle.b"]);

    #[test]
    fn registry_lookup_and_bitset() {
        assert_eq!(REGISTRY.len(), 2);
        assert_eq!(
            REGISTRY.lookup("quirk.battle.no_luck_without_hero"),
            Some(NO_LUCK_WITHOUT_HERO)
        );
        assert_eq!(
            REGISTRY.lookup("quirk.battle.single_precision_rates"),
            Some(SINGLE_PRECISION_RATES)
        );
        // `new` is a const fn; call it at run time too so coverage sees it.
        let runtime = QuirkRegistry::new(&["quirk.battle.a"]);
        assert_eq!(
            runtime.lookup("quirk.battle.a").map(QuirkFlag::index),
            Some(0)
        );
        assert_eq!(TEST.len(), 2);
        assert!(!TEST.is_empty());
        let b = TEST.lookup("quirk.battle.b").unwrap();
        assert_eq!(b.index(), 1);
        assert_eq!(TEST.lookup("quirk.battle.c"), None);
        let q = Quirks::default().with(b, true);
        assert!(q.is_enabled(b));
        assert_eq!(q.bits(), 2);
        assert!(!q.with(b, false).is_enabled(b));
    }
}
