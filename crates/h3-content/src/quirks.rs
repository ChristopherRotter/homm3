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
    "quirk.battle.unlimited_ballista_shots",
    "quirk.battle.first_protection",
    "quirk.battle.counterstrike_extra_round",
    "quirk.battle.frenzy_reduced_defense",
    "quirk.battle.fire_shield_first_stage",
    "quirk.battle.early_mass_version",
    "quirk.battle.mass_cure_skips_hypnotized",
    "quirk.battle.natives_sight_at_start",
    "quirk.battle.attack_from_seen_pit",
]);

/// Q-0003: creatures of a side without a hero never get lucky strikes (BAT-LCK-002).
pub const NO_LUCK_WITHOUT_HERO: QuirkFlag = QuirkFlag(0);

/// Q-0005: the original's rounding: damage in the executable's truncated stages and x87
/// arithmetic, rates as the code stores them (BAT-DMG-002 and others).
pub const SINGLE_PRECISION_RATES: QuirkFlag = QuirkFlag(1);

/// Q-0004: the shooting war machine (the Ballista) shoots on without shots left (BAT-WAR-002).
pub const UNLIMITED_BALLISTA_SHOTS: QuirkFlag = QuirkFlag(2);

/// Q-0007: a damage spell of several schools meets the target's first protection by school
/// rank, not its strongest (BAT-MAG-013).
pub const FIRST_PROTECTION: QuirkFlag = QuirkFlag(3);

/// Q-0008: Counterstrike's extra retaliations are given at the round start in which the spell
/// runs out, as the reset comes before the countdown (BAT-MAG-015).
pub const COUNTERSTRIKE_EXTRA_ROUND: QuirkFlag = QuirkFlag(4);

/// Q-0009 (S-16): Frenzy's attack bonus counts the frenzied stack's defense as the struck
/// stack sees it, reduced by a Behemoth's share (BAT-MAG-019).
pub const FRENZY_REDUCED_DEFENSE: QuirkFlag = QuirkFlag(5);

/// Q-0010: Fire Shield reflects a share of the strike's first damage stage (before the struck
/// stack's defense and protections), not of the damage dealt (BAT-MAG-018).
pub const FIRE_SHIELD_FIRST_STAGE: QuirkFlag = QuirkFlag(6);

/// Q-0011 (S-10): a spell whose mass version the original starts earlier than its clean
/// version (`SpellMagic::fixed_mass_from`) has it from that earlier mastery: Forgetfulness from
/// Advanced (BAT-MAG-023).
pub const EARLY_MASS_VERSION: QuirkFlag = QuirkFlag(7);

/// Q-0012 (S-13): every mass version skips hypnotized stacks, also one whose dispel ends
/// Hypnotize (Expert Cure), which the clean version lets reach the caster's hypnotized stacks
/// (BAT-MAG-026).
pub const MASS_CURE_SKIPS_HYPNOTIZED: QuirkFlag = QuirkFlag(8);

/// BAT-QUIRK-1: a side sees the other side's hidden obstacles (Quicksand) if it had a stack on
/// its native terrain at the battle's start, given to each obstacle when it is placed; natives
/// that die or appear later change nothing (BAT-MAG-031). Off: while it has a living one.
pub const NATIVES_SIGHT_AT_START: QuirkFlag = QuirkFlag(9);

/// BAT-QUIRK-2: a walker may be commanded to attack from a pit its side sees; it stops on
/// entering the pit and the attack is skipped (BAT-MAG-032). Off: such an attack is not legal.
pub const ATTACK_FROM_SEEN_PIT: QuirkFlag = QuirkFlag(10);

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
        assert_eq!(REGISTRY.len(), 11);
        assert_eq!(
            REGISTRY.lookup("quirk.battle.no_luck_without_hero"),
            Some(NO_LUCK_WITHOUT_HERO)
        );
        assert_eq!(
            REGISTRY.lookup("quirk.battle.single_precision_rates"),
            Some(SINGLE_PRECISION_RATES)
        );
        assert_eq!(
            REGISTRY.lookup("quirk.battle.unlimited_ballista_shots"),
            Some(UNLIMITED_BALLISTA_SHOTS)
        );
        assert_eq!(
            REGISTRY.lookup("quirk.battle.first_protection"),
            Some(FIRST_PROTECTION)
        );
        assert_eq!(
            REGISTRY.lookup("quirk.battle.counterstrike_extra_round"),
            Some(COUNTERSTRIKE_EXTRA_ROUND)
        );
        assert_eq!(
            REGISTRY.lookup("quirk.battle.frenzy_reduced_defense"),
            Some(FRENZY_REDUCED_DEFENSE)
        );
        assert_eq!(
            REGISTRY.lookup("quirk.battle.fire_shield_first_stage"),
            Some(FIRE_SHIELD_FIRST_STAGE)
        );
        assert_eq!(
            REGISTRY.lookup("quirk.battle.early_mass_version"),
            Some(EARLY_MASS_VERSION)
        );
        assert_eq!(
            REGISTRY.lookup("quirk.battle.mass_cure_skips_hypnotized"),
            Some(MASS_CURE_SKIPS_HYPNOTIZED)
        );
        assert_eq!(
            REGISTRY.lookup("quirk.battle.natives_sight_at_start"),
            Some(NATIVES_SIGHT_AT_START)
        );
        assert_eq!(
            REGISTRY.lookup("quirk.battle.attack_from_seen_pit"),
            Some(ATTACK_FROM_SEEN_PIT)
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
