//! The game's rules (ADR-0015 point 2): the core ruleset and one compiled part per context, each
//! immutable and `Arc`-shared, bundled once so every context reads the same rules.

use alloc::sync::Arc;
use core::fmt;

use h3_adventure::AdventureRuleset;
use h3_content::Ruleset;

/// The core ruleset and the contexts' parts of one game.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameRuleset {
    core: Arc<Ruleset>,
    adventure: Arc<AdventureRuleset>,
}

/// The adventure part was compiled on another core ruleset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RulesetMismatch;

impl fmt::Display for RulesetMismatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("the adventure part was compiled on another core ruleset")
    }
}

impl core::error::Error for RulesetMismatch {}

impl GameRuleset {
    /// Bundles a core ruleset with the adventure part compiled on it.
    ///
    /// # Errors
    ///
    /// [`RulesetMismatch`] if the part was compiled on another core (by content hash).
    pub fn new(
        core: Arc<Ruleset>,
        adventure: Arc<AdventureRuleset>,
    ) -> Result<Self, RulesetMismatch> {
        if adventure.core_hash() == core.content_hash() {
            Ok(Self { core, adventure })
        } else {
            Err(RulesetMismatch)
        }
    }

    /// The core ruleset.
    pub fn core(&self) -> &Ruleset {
        &self.core
    }

    /// The adventure part.
    pub fn adventure(&self) -> &AdventureRuleset {
        &self.adventure
    }
}
