//! The game's rules (ADR-0015 point 2): the core ruleset and one compiled part per context, each
//! immutable and `Arc`-shared, bundled once so every context reads the same rules.

use alloc::sync::Arc;
use core::fmt;

use h3_adventure::AdventureRuleset;
use h3_content::Ruleset;
use h3_content_battle::BattleRuleset;

/// The core ruleset and the contexts' parts of one game: the adventure's and, since E08 T19a, the
/// battle's (ADR-0017 point 11).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameRuleset {
    core: Arc<Ruleset>,
    adventure: Arc<AdventureRuleset>,
    battle: Arc<BattleRuleset>,
}

/// The parts do not fit the core: one was compiled on another core ruleset, or the adventure part
/// names a battlefield of a battle on the coast ([`h3_adventure::BattleTerrains`]) the battle part
/// lacks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RulesetMismatch;

impl fmt::Display for RulesetMismatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("the ruleset parts do not fit the core ruleset or each other")
    }
}

impl core::error::Error for RulesetMismatch {}

impl GameRuleset {
    /// Bundles a core ruleset with the adventure and battle parts compiled on it.
    ///
    /// # Errors
    ///
    /// [`RulesetMismatch`] if a part was compiled on another core (by content hash), or the
    /// adventure part's coast battlefield is not the battle part's.
    pub fn new(
        core: Arc<Ruleset>,
        adventure: Arc<AdventureRuleset>,
        battle: Arc<BattleRuleset>,
    ) -> Result<Self, RulesetMismatch> {
        let hash = core.content_hash();
        let field = adventure.params().battle_terrains().is_none_or(|t| {
            battle
                .battlefield_by_key(t.coast_battlefield.as_str())
                .is_some()
        });
        if adventure.core_hash() == hash && battle.core_hash() == hash && field {
            Ok(Self {
                core,
                adventure,
                battle,
            })
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

    /// The battle part (E08 T19a).
    pub fn battle(&self) -> &BattleRuleset {
        &self.battle
    }
}
