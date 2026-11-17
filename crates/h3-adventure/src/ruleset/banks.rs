//! The creature banks' kind (E08 T123a; ADR-0016 point 4, increment 13's design note §1;
//! ADV-BNK-001): what a type of creature bank holds as data, its levels as the original's
//! `CrBanks.txt` gives them before the split, and their compilation from the packs' keys. The
//! draw when the game is built (the level, the artifacts, the split, the upgrade: ADV-BNK-002) is
//! the static map's since T124b1 (`state::banks`); the visit and the reward come with T124b2;
//! until then a step onto a bank is refused as any unbuilt visit's
//! ([`super::ObjectKind::effect`]).

use alloc::collections::BTreeSet;
use alloc::vec::Vec;

use h3_content::{ArtifactClass, ContentKey, CreatureTypeId};
use h3_kernel::{CanonicalHash, StateHasher};

use super::Scope;
use super::rewards::{
    Compiled, OnceBonus, Resources, check_chances, compile_resources, hash_resources,
};

/// The most levels a creature bank has (increment 13's design note §1): the original's kinds'
/// record holds four (`creature_bank_kinds` 0x695038, the levels at +0x10, 0x60 bytes each;
/// survey `docs/rules/surveys/E08-banks.md` §2).
pub const BANK_LEVELS: usize = 4;

/// The most guard stacks a bank's level names before the split: the original's four count
/// columns (`CrBanks.txt`'s columns 3, 6, 8 and 10; survey §2).
pub const BANK_GUARDS: usize = 4;

/// The largest count of a bank's reward creature: the original keeps it in a signed byte (the
/// level's +0x58, the record's +0x58; survey §2).
pub const MAX_BANK_REWARD: u8 = 127;

/// The artifact classes a bank's level counts, in the table's order (`CrBanks.txt`'s columns 22
/// to 25, the level's bytes +0x5B to +0x5E; survey §2): a level's [`BankLevel::artifacts`] by
/// position. The draw takes them last first, the relics first (ADV-BNK-002, T124b1).
pub const BANK_ARTIFACT_CLASSES: [ArtifactClass; 4] = [
    ArtifactClass::Treasure,
    ArtifactClass::Minor,
    ArtifactClass::Major,
    ArtifactClass::Relic,
];

/// How a creature bank asks its visitor (increment 13's design note §1, §6; survey §4): the two
/// visit routines of the original.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BankQuestion {
    /// The team's mark at once, then the question unless looted (`visit_creature_bank`
    /// 0x4A15A0, `visit_dragon_utopia` 0x4A2140: class 16's seven kinds and Dragon Utopia).
    Attack,
    /// The question first, looted or not, then the team's mark after a yes (`visit_search_bank`
    /// 0x4AC490: the Crypt, the Shipwreck and the Derelict Ship).
    Search,
}

/// One of a creature bank's levels (survey §2): what a bank drawn at it holds, its guards before
/// the split.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BankLevel {
    /// The chance of this level, in percent; a type's levels' chances add up to 100.
    pub chance: u8,
    /// The guards in the table's order before the split: one to [`BANK_GUARDS`] distinct
    /// creatures, none a war machine, each count from 1 to the part's `max_stack_count`.
    pub guards: Vec<(CreatureTypeId, u32)>,
    /// The chance, in percent (0 to 100), that the split's upgrade slot is upgraded.
    pub upgrade: u8,
    /// The resources a win gives: distinct, in the core's order, each from 1.
    pub resources: Resources,
    /// The creature a win gives and its count, from 1 to [`MAX_BANK_REWARD`]; none for no
    /// creature (the original's count 0).
    pub creature: Option<(CreatureTypeId, u8)>,
    /// The artifacts a win gives, a count per class of [`BANK_ARTIFACT_CLASSES`], drawn when the
    /// game is built.
    pub artifacts: [u8; 4],
}

/// A creature bank's type (increment 13's design note §1; ADV-BNK-001).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bank {
    /// How it asks its visitor.
    pub question: BankQuestion,
    /// Its levels, one to [`BANK_LEVELS`], in the table's order.
    pub levels: Vec<BankLevel>,
    /// The once-bonus a looted one's search gives (a [`BankQuestion::Search`] kind's only: −1
    /// morale under the marks 14 to 16, the original's status bits 9 to 11; part C §12).
    pub looted: Option<OnceBonus>,
    /// Whether its battle is fought on the sea terrain's deck (the original's ship flag
    /// +0x53C6, set by class for 24 and 85; survey §5).
    pub deck: bool,
    /// Whether its water entrance is reached from land and crossed by a flight as an empty
    /// boat's tile (a neighbour-visited kind's only: the Shipwreck, design note §9).
    pub coast: bool,
}

/// [`BankLevel`] as a pack defines it: creatures and resources by key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BankLevelDef {
    /// The chance, in percent: from 1, a type's levels' together 100.
    pub chance: u8,
    /// The guards: one to [`BANK_GUARDS`] distinct core creatures, none a war machine, each count
    /// from 1 (and at most `max_stack_count`, checked when every pack is in).
    pub guards: Vec<(ContentKey, u32)>,
    /// The upgrade chance, at most 100.
    pub upgrade: u8,
    /// The resources: distinct core resources, each amount from 1 to [`super::MAX_GAIN`].
    pub resources: Vec<(ContentKey, u32)>,
    /// The reward's creature, a core creature and no war machine, with its count from 1 to
    /// [`MAX_BANK_REWARD`].
    pub creature: Option<(ContentKey, u32)>,
    /// The artifacts by class of [`BANK_ARTIFACT_CLASSES`].
    pub artifacts: [u8; 4],
}

/// [`Bank`] as a pack defines it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BankDef {
    /// How it asks.
    pub question: BankQuestion,
    /// Its levels, one to [`BANK_LEVELS`].
    pub levels: Vec<BankLevelDef>,
    /// The looted bonus, a search kind's only.
    pub looted: Option<OnceBonus>,
    /// Whether its battle is on the deck.
    pub deck: bool,
    /// Whether its water entrance is visited from the coast; its type visited from a neighbour
    /// (checked with the type).
    pub coast: bool,
}

/// [`BankDef`]'s checks; the guards' counts against the part's `max_stack_count` and their stacks
/// against the army's slots come when every pack is in ([`check_bank`]).
// rule: ADV-BNK-001
pub(super) fn compile_bank(scope: &Scope<'_>, def: &BankDef) -> Compiled<Bank> {
    let mut levels = Vec::with_capacity(def.levels.len());
    for level in &def.levels {
        match compile_level(scope, level)? {
            Ok(level) => levels.push(level),
            Err(reason) => return Ok(Err(reason)),
        }
    }
    Ok(if !(1..=BANK_LEVELS).contains(&levels.len()) {
        Err("a bank has no level or more than four")
    } else if let Err(reason) = check_chances(levels.iter().map(|l| l.chance), true) {
        Err(reason)
    } else if def.looted.is_some() && def.question != BankQuestion::Search {
        Err("a bank's looted bonus is not a search's")
    } else {
        Ok(Bank {
            question: def.question,
            levels,
            looted: def.looted,
            deck: def.deck,
            coast: def.coast,
        })
    })
}

/// One level's checks (its references resolved first).
fn compile_level(scope: &Scope<'_>, def: &BankLevelDef) -> Compiled<BankLevel> {
    let guards = def
        .guards
        .iter()
        .map(|(key, count)| Ok((scope.creature(key)?, *count)))
        .collect::<Result<Vec<_>, _>>()?;
    let creature = def
        .creature
        .as_ref()
        .map(|(key, count)| Ok((scope.creature(key)?, *count)))
        .transpose()?;
    let resources = match compile_resources(scope, &def.resources)? {
        Ok(resources) => resources,
        Err(reason) => return Ok(Err(reason)),
    };
    let distinct = guards.iter().map(|g| g.0).collect::<BTreeSet<_>>().len() == guards.len();
    Ok(if !(1..=BANK_GUARDS).contains(&guards.len()) {
        Err("a bank's level has no guard or more than four")
    } else if !distinct {
        Err("a bank's level names a guard twice")
    } else if guards.iter().any(|g| scope.war_machine(g.0)) {
        Err("a bank's guard is a war machine")
    } else if guards.iter().any(|g| g.1 == 0) {
        Err("a bank's guard count is 0")
    } else if def.upgrade > 100 {
        Err("a bank's upgrade chance is above 100")
    } else if creature.is_some_and(|c| scope.war_machine(c.0)) {
        Err("a bank's reward creature is a war machine")
    } else if creature.is_some_and(|c| !(1..=u32::from(MAX_BANK_REWARD)).contains(&c.1)) {
        Err("a bank's reward count is 0 or above 127")
    } else {
        Ok(BankLevel {
            chance: def.chance,
            guards,
            upgrade: def.upgrade,
            resources,
            creature: creature.map(|(c, n)| (c, u8::try_from(n).unwrap_or(MAX_BANK_REWARD))),
            artifacts: def.artifacts,
        })
    })
}

/// The stacks a level's guards stand as after the split (survey §3, ADV-BNK-002): five for one
/// to three named stacks (their counts divided into the empty slots), else the named ones.
pub(crate) const fn bank_stacks(guards: usize) -> usize {
    if guards < BANK_GUARDS { 5 } else { guards }
}

/// A bank's levels, once the part's bounds are known: every guard count at most
/// `max_stack_count`, every level's stacks after the split within the army's slots.
pub(super) fn check_bank(
    bank: &Bank,
    army_slots: u8,
    max_stack_count: u32,
) -> Result<(), &'static str> {
    if bank
        .levels
        .iter()
        .any(|l| l.guards.iter().any(|g| g.1 > max_stack_count))
    {
        Err("a bank's guard count is above max_stack_count")
    } else if bank
        .levels
        .iter()
        .any(|l| bank_stacks(l.guards.len()) > usize::from(army_slots))
    {
        Err("a bank's guards stand as more stacks than the army's slots")
    } else {
        Ok(())
    }
}

impl CanonicalHash for Bank {
    /// The question (0 attack, 1 search), the levels as length and each (its chance, its guards
    /// as length and each creature and count, its upgrade chance, its resources, its creature as
    /// 0 for none, else 1, the creature and the count, its four artifact counts), the looted
    /// bonus as 0 for none, else 1 and the bonus, the deck flag, the coast flag.
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_u8(match self.question {
            BankQuestion::Attack => 0,
            BankQuestion::Search => 1,
        });
        h.write_len(self.levels.len());
        for level in &self.levels {
            h.write_u8(level.chance);
            h.write_len(level.guards.len());
            for (creature, count) in &level.guards {
                creature.hash_into(h);
                h.write_u32(*count);
            }
            h.write_u8(level.upgrade);
            hash_resources(&level.resources, h);
            match level.creature {
                None => h.write_u8(0),
                Some((creature, count)) => {
                    h.write_u8(1);
                    creature.hash_into(h);
                    h.write_u8(count);
                }
            }
            for &count in &level.artifacts {
                h.write_u8(count);
            }
        }
        match &self.looted {
            None => h.write_u8(0),
            Some(bonus) => {
                h.write_u8(1);
                bonus.hash_into(h);
            }
        }
        h.write_bool(self.deck);
        h.write_bool(self.coast);
    }
}
