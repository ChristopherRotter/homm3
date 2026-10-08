//! Creatures that cast a spell as their stack's action (E04b T7b1; ADR-0014 point 3,
//! BAT-MAG-048), after the original's code (`docs/formats/installation.md` §7.21): the action
//! names only its target; the stack's `spell_caster` ability gives the spell, the mastery and the
//! power, and each cast uses one of the casts its creature has per battle. The casts left are kept
//! beside the stacks (the hot paths never read them).

use h3_content::{Ability, CreatureTypeId, Ruleset, SpellId};
use h3_contracts::BattleSide;
use h3_kernel::{CanonicalHash, StateHasher};

use crate::error::CommandError;
use crate::magic::{self, Cast, CastTarget, CastTargets};
use crate::state::{BattleState, STACK_CAPACITY, Stack, StackId};

/// What a creature casts as its action: its `spell_caster` ability's spell, mastery and power.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CreatureSpell {
    /// The spell.
    pub(crate) id: SpellId,
    /// The mastery it is cast at.
    pub(crate) mastery: u8,
    /// The caster's power.
    pub(crate) power: u8,
}

/// What `creature` casts as its stack's action, if it is a caster (BAT-MAG-048).
pub(crate) fn creature_spell(ruleset: &Ruleset, creature: CreatureTypeId) -> Option<CreatureSpell> {
    ruleset.abilities(creature).iter().find_map(|a| match *a {
        Ability::SpellCaster {
            spell,
            mastery,
            power,
        } => Some(CreatureSpell {
            id: spell,
            mastery,
            power,
        }),
        _ => None,
    })
}

/// The casts left per stack, by stack id (the original's stack field +0xDC).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Casts {
    /// The casts each stack has left this battle.
    pub(crate) left: [u8; STACK_CAPACITY],
}

impl Default for Casts {
    fn default() -> Self {
        Self {
            left: [0; STACK_CAPACITY],
        }
    }
}

impl Casts {
    /// The casts of `stacks` at the battle's start: their creatures' casts per battle.
    pub(crate) fn at_start(ruleset: &Ruleset, stacks: &[Stack]) -> Self {
        let mut out = Self::default();
        for (left, stack) in out.left.iter_mut().zip(stacks) {
            *left = per_battle(ruleset, stack.creature);
        }
        out
    }
}

impl CanonicalHash for Casts {
    /// Per stack, the casts left (v33).
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_bytes(&self.left);
    }
}

/// The casts per battle of `creature` (its pack's `casts`).
pub(crate) fn per_battle(ruleset: &Ruleset, creature: CreatureTypeId) -> u8 {
    ruleset.creature_info(creature).map_or(0, |i| i.casts)
}

/// The active stack and its spell, if `actor` commands it outside the tactics phase and it may
/// cast now (BAT-MAG-048): its creature is a caster and it has a cast left.
fn ready(
    state: &BattleState,
    ruleset: &Ruleset,
    actor: BattleSide,
) -> Option<(StackId, CreatureSpell)> {
    // Most stacks have no casts: test that first (the random agent asks at every step).
    let id = state.active.filter(|id| state.casts.left[id.index()] > 0)?;
    let stack = state.at(id);
    if state.tactics.is_some() || stack.controller() != actor {
        return None;
    }
    creature_spell(ruleset, stack.creature).map(|spell| (id, spell))
}

/// The spell stack `id` casts as its action, if its creature is a caster (BAT-MAG-048), whether or
/// not it may cast now.
pub fn stack_spell(state: &BattleState, ruleset: &Ruleset, id: StackId) -> Option<SpellId> {
    state
        .stack(id)
        .and_then(|s| creature_spell(ruleset, s.creature))
        .map(|spell| spell.id)
}

/// The spell the active stack casts if `actor` commands its cast now (BAT-MAG-048): `None` if it
/// may not cast (not `actor`'s stack, the tactics phase, not a caster, no casts left). Whether it
/// has a target is [`creature_cast_targets`]'s.
pub fn creature_cast_spell(
    state: &BattleState,
    ruleset: &Ruleset,
    actor: BattleSide,
) -> Option<SpellId> {
    ready(state, ruleset, actor).map(|(_, spell)| spell.id)
}

/// Fills `out` with every target [`crate::apply`] accepts for `actor`'s
/// [`Command::CreatureCast`](crate::Command::CreatureCast) now, stacks by id (BAT-MAG-048):
/// empty if the active stack may not cast ([`creature_cast_spell`]).
pub fn creature_cast_targets(
    state: &BattleState,
    ruleset: &Ruleset,
    actor: BattleSide,
    out: &mut CastTargets,
) {
    out.clear();
    let Some((caster, spell)) = ready(state, ruleset, actor) else {
        return;
    };
    for target in (0u8..)
        .take(state.stacks.len())
        .map(|s| CastTarget::Stack(StackId(s)))
    {
        if magic::creature_target(state, ruleset, actor, (caster, spell), target) {
            // At most `STACK_CAPACITY` targets.
            let _ = out.push(target);
        }
    }
}

/// Validates `actor`'s creature cast at `target` (BAT-MAG-048): the active stack, which `actor`
/// commands outside the tactics phase, is a caster with a cast left, and `target` is a target of
/// its spell.
pub(crate) fn plan<'r>(
    state: &BattleState,
    ruleset: &'r Ruleset,
    actor: BattleSide,
    target: CastTarget,
) -> Result<Cast<'r>, CommandError> {
    let (caster, spell) = ready(state, ruleset, actor).ok_or(CommandError::CannotCast)?;
    magic::plan_creature_cast(state, ruleset, actor, (caster, spell), target)
}

/// Uses one of stack `s`'s casts (the original lowers them first, 0x448260).
pub(crate) fn spend(state: &mut BattleState, s: StackId) {
    let left = &mut state.casts.left[s.index()];
    *left = left.saturating_sub(1);
}
