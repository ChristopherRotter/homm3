//! Creatures that cast a spell as their stack's action (E04b T7b1, T7b2a; ADR-0014 point 3,
//! BAT-MAG-048, BAT-MAG-049), after the original's code (`docs/formats/installation.md` §7.21):
//! the action names only its target; the stack's `spell_caster` ability gives the spell, the
//! mastery and the power, or a random caster (the Faerie Dragon) draws its spell at each
//! activation from its `drawn_spell` entries; each cast uses one of the casts its creature has
//! per battle. The casts left and the active stack's drawn spell are kept beside the stacks (the
//! hot paths never read them).

use h3_content::{Ability, Ruleset, SpellId};
use h3_contracts::BattleSide;
use h3_kernel::{CanonicalHash, StateHasher};

use crate::error::CommandError;
use crate::event::{Event, EventKind, Sink, Visibility};
use crate::magic::{self, Cast, CastTarget, CastTargets};
use crate::state::{BattleState, STACK_CAPACITY, Stack, StackId};

/// What a creature casts as its action: its `spell_caster` ability's spell, mastery and power, or
/// a random caster's drawn spell at its mastery with its power per creature times its creatures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CreatureSpell {
    /// The spell.
    pub(crate) id: SpellId,
    /// The mastery it is cast at.
    pub(crate) mastery: u8,
    /// The caster's power.
    pub(crate) power: u32,
    /// Whether the stack drew the spell (BAT-MAG-049): it may be aimed at any stack but the
    /// caster that the spell can affect, a hex spell at that stack's hex.
    pub(crate) drawn: bool,
}

/// What stack `id` casts as its action now, if its creature is a caster (BAT-MAG-048): its
/// `spell_caster` ability's spell; for a random caster the spell it drew at this activation, if it
/// is the active stack and drew one (BAT-MAG-049).
pub(crate) fn stack_cast(
    state: &BattleState,
    ruleset: &Ruleset,
    id: StackId,
) -> Option<CreatureSpell> {
    let stack = state.stack(id)?;
    ruleset
        .abilities(stack.creature)
        .iter()
        .find_map(|a| match *a {
            Ability::SpellCaster {
                spell,
                mastery,
                power,
            } => Some(CreatureSpell {
                id: spell,
                mastery,
                power: u32::from(power),
                drawn: false,
            }),
            Ability::RandomSpellCaster {
                mastery,
                power_per_creature,
            } => state
                .casts
                .drawn
                .filter(|&(s, _)| s == id && state.active == Some(id))
                .map(|(_, spell)| CreatureSpell {
                    id: spell,
                    mastery,
                    power: u32::from(power_per_creature).saturating_mul(stack.count),
                    drawn: true,
                }),
            _ => None,
        })
}

/// The casts left per stack, by stack id (the original's stack field +0xDC), and the active
/// stack's drawn spell (+0x4E0).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Casts {
    /// The casts each stack has left this battle.
    pub(crate) left: [u8; STACK_CAPACITY],
    /// The spell a random caster drew at its activation, with the stack (BAT-MAG-049); none after
    /// the activation of any other stack.
    pub(crate) drawn: Option<(StackId, SpellId)>,
}

impl Default for Casts {
    fn default() -> Self {
        Self {
            left: [0; STACK_CAPACITY],
            drawn: None,
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
    /// Per stack, the casts left (v33); the drawn spell, if any: its stack and spell (v34).
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_bytes(&self.left);
        self.drawn.map(|(s, _)| s.0).hash_into(h);
        self.drawn.map(|(_, spell)| spell.raw()).hash_into(h);
    }
}

/// The casts per battle of `creature` (its pack's `casts`).
pub(crate) fn per_battle(ruleset: &Ruleset, creature: h3_content::CreatureTypeId) -> u8 {
    ruleset.creature_info(creature).map_or(0, |i| i.casts)
}

/// At the activation of stack `id` for its side's command (BAT-MAG-049; the original's 0x465330
/// and 0x447510, `installation.md` §7.21): a random caster with a cast left draws its spell from
/// its creature's `drawn_spell` entries, each with the chance of its weight among their sum, and
/// its controller is told (`SpellDrawn`, private to it, ADR-0014 point 8); any other activation
/// forgets the last drawn spell. (The tactics phase activates no stack for a command, as the
/// original's activation does nothing there.)
// rule: BAT-MAG-049
pub(crate) fn activated<S: Sink>(
    state: &mut BattleState,
    ruleset: &Ruleset,
    id: StackId,
    sink: &mut S,
) {
    state.casts.drawn = None;
    // Most stacks have no casts: test that first.
    if state.casts.left[id.index()] == 0 {
        return;
    }
    let stack = *state.at(id);
    let abilities = ruleset.abilities(stack.creature);
    if !abilities
        .iter()
        .any(|a| matches!(a, Ability::RandomSpellCaster { .. }))
    {
        return;
    }
    let weights = || {
        abilities.iter().filter_map(|a| match *a {
            Ability::DrawnSpell { spell, weight } => Some((spell, u32::from(weight))),
            _ => None,
        })
    };
    // A random caster has drawn spells of weight at least 1 (the compiler checks).
    let total = weights().map(|(_, w)| w).sum::<u32>();
    let mut r = state.rng.below(total);
    let drawn = weights()
        .find(|&(_, w)| {
            let hit = r < w;
            r = r.saturating_sub(w);
            hit
        })
        .map(|(spell, _)| (id, spell));
    state.casts.drawn = drawn;
    drawn.into_iter().for_each(|(stack_id, spell)| {
        sink.emit(Event {
            visibility: Visibility::Side(stack.controller()),
            kind: EventKind::SpellDrawn {
                stack: stack_id,
                spell,
            },
        });
    });
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
    stack_cast(state, ruleset, id).map(|spell| (id, spell))
}

/// The spell stack `id` casts as its action, if its creature is a caster (BAT-MAG-048), whether or
/// not it may cast now; for a random caster the spell it drew, while it is the active stack
/// (BAT-MAG-049).
pub fn stack_spell(state: &BattleState, ruleset: &Ruleset, id: StackId) -> Option<SpellId> {
    stack_cast(state, ruleset, id).map(|spell| spell.id)
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
    magic::creature_targets(state, ruleset, actor, (caster, spell), out);
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
