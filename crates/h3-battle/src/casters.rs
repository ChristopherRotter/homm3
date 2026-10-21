//! Creatures that cast a spell as their stack's action (E04b T7b1, T7b2a; ADR-0014 point 3,
//! BAT-MAG-048, BAT-MAG-049), after the original's code (`docs/formats/installation.md` §7.21):
//! the action names only its target; the stack's `spell_caster` ability gives the spell, the
//! mastery and the power, a random caster (the Faerie Dragon) draws its spell at each activation
//! from its `drawn_spell` entries, and a helpful caster (the Master Genie, E04b T7b2b) draws at the
//! cast among those useful for its target (BAT-MAG-050); the raisers raise without a spell, the
//! Archangel another stack of its side (BAT-MAG-051), the Pit Lord Demons from a corpse
//! (BAT-MAG-052, E04b T7b3); each cast uses one of the casts its creature has per battle. A
//! periodic caster (the Enchanter, BAT-MAG-053, E04b T7b4) takes no action for it: it casts on
//! its own at its activation, once in so many rounds of its side. The casts left, the active
//! stack's drawn spell and each side's rounds since its periodic cast are kept beside the stacks
//! (the hot paths never read them).

use h3_content::{CreatureFlag, CreatureTypeId, PERIODIC_FIRST_STACK, SpellId};
use h3_content_battle::BattleSpell;
use h3_content_battle::{Ability, BattleRuleset, SpellPolarity, Usefulness};
use h3_contracts::BattleSide;
use h3_kernel::{CanonicalHash, StateHasher};

use crate::combat;
use crate::created;
use crate::effects::Status;
use crate::error::CommandError;
use crate::event::{Event, EventKind, Sink, Visibility};
use crate::geometry::Field;
use crate::magic::{self, Cast, CastTarget, CastTargets};
use crate::raise;
use crate::state::{BattleState, STACK_CAPACITY, Stack, StackFlags, StackId, StackTraits};

/// The spell a stack's cast names before its resolution (ADR-0014 points 3 and 9; BAT-MAG-048 to
/// BAT-MAG-050).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CastSpell {
    /// This spell: a caster's own (BAT-MAG-048) or the one it drew (BAT-MAG-049).
    Named(SpellId),
    /// None yet: the stack draws it at the cast among the spells useful for its target (the
    /// Master Genie, BAT-MAG-050).
    AtResolution,
    /// None: the stack raises without a spell (the Archangel, the Pit Lord; BAT-MAG-051,
    /// BAT-MAG-052).
    Raise,
}

impl CastSpell {
    /// The spell, if the cast names one before its resolution.
    pub const fn named(self) -> Option<SpellId> {
        match self {
            Self::Named(id) => Some(id),
            Self::AtResolution | Self::Raise => None,
        }
    }
}

/// What a raiser raises as its stack's action (BAT-MAG-051, BAT-MAG-052).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Raiser {
    /// Another stack of its side, living or dead, under the raise spell `test`'s test, by
    /// `per_creature` hit points per creature of the raiser (the Archangel).
    Stack {
        /// The raise spell whose test applies.
        test: SpellId,
        /// Hit points per creature of the raiser.
        per_creature: u16,
    },
    /// A stack of `creature` from a corpse of its side (the Pit Lord's Demons).
    Corpse {
        /// The creature it raises.
        creature: CreatureTypeId,
        /// Hit points per creature of the raiser.
        per_creature: u16,
    },
}

/// What a stack of `creature` raises as its action, if its creature is a raiser.
pub(crate) fn raiser(ruleset: &BattleRuleset, creature: CreatureTypeId) -> Option<Raiser> {
    ruleset.abilities(creature).iter().find_map(|a| match *a {
        Ability::StackRaiser {
            spell,
            hit_points_per_creature,
        } => Some(Raiser::Stack {
            test: spell,
            per_creature: hit_points_per_creature,
        }),
        Ability::CorpseRaiser {
            creature,
            hit_points_per_creature,
        } => Some(Raiser::Corpse {
            creature,
            per_creature: hit_points_per_creature,
        }),
        _ => None,
    })
}

/// What a creature casts as its action: its `spell_caster` ability's spell, mastery and power, a
/// random caster's drawn spell at its mastery with its power per creature times its creatures, or
/// a helpful caster's draw at the cast.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CreatureSpell {
    /// The spell, or the draw at the cast.
    pub(crate) spell: CastSpell,
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
/// is the active stack and drew one (BAT-MAG-049); for a helpful caster a draw at the cast
/// (BAT-MAG-050). A battlefield may raise the mastery (BAT-TER-005).
pub(crate) fn stack_cast(
    state: &BattleState,
    ruleset: &BattleRuleset,
    id: StackId,
) -> Option<CreatureSpell> {
    let stack = state.stack(id)?;
    // rule: BAT-TER-005
    let mastery = |m| magic::other_mastery(state, m);
    ruleset
        .abilities(stack.creature)
        .iter()
        .find_map(|a| match *a {
            Ability::SpellCaster {
                spell,
                mastery: m,
                power,
            } => Some(CreatureSpell {
                spell: CastSpell::Named(spell),
                mastery: mastery(m),
                power: u32::from(power),
                drawn: false,
            }),
            Ability::HelpfulSpellCaster { mastery: m, power } => Some(CreatureSpell {
                spell: CastSpell::AtResolution,
                mastery: mastery(m),
                power: u32::from(power),
                drawn: false,
            }),
            Ability::StackRaiser { .. } | Ability::CorpseRaiser { .. } => Some(CreatureSpell {
                spell: CastSpell::Raise,
                mastery: 0,
                power: 0,
                drawn: false,
            }),
            Ability::RandomSpellCaster {
                mastery: m,
                power_per_creature,
            } => state
                .casts
                .drawn
                .filter(|&(s, _)| s == id && state.active == Some(id))
                .map(|(_, spell)| CreatureSpell {
                    spell: CastSpell::Named(spell),
                    mastery: mastery(m),
                    power: u32::from(power_per_creature).saturating_mul(stack.count),
                    drawn: true,
                }),
            _ => None,
        })
}

/// The casts left per stack, by stack id (the original's stack field +0xDC), the active stack's
/// drawn spell (+0x4E0) and each side's rounds since its periodic cast (the battle's +0x132A0).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Casts {
    /// The casts each stack has left this battle.
    pub(crate) left: [u8; STACK_CAPACITY],
    /// The spell a random caster drew at its activation, with the stack (BAT-MAG-049); none after
    /// the activation of any other stack.
    pub(crate) drawn: Option<(StackId, SpellId)>,
    /// Per side, the round starts since its periodic casters cast, at most 255 (BAT-MAG-053):
    /// every battle starts with 255, ready (BAT-QUIRK-14: the original keeps the count of the
    /// battle before).
    pub(crate) periodic: [u8; 2],
}

impl Default for Casts {
    fn default() -> Self {
        Self {
            left: [0; STACK_CAPACITY],
            drawn: None,
            periodic: [u8::MAX; 2],
        }
    }
}

impl Casts {
    /// The casts of `stacks` at the battle's start: their creatures' casts per battle.
    pub(crate) fn at_start(ruleset: &BattleRuleset, stacks: &[Stack]) -> Self {
        let mut out = Self::default();
        for (left, stack) in out.left.iter_mut().zip(stacks) {
            *left = per_battle(ruleset, stack.creature);
        }
        out
    }
}

impl CanonicalHash for Casts {
    /// Per stack, the casts left (v33); the drawn spell, if any: its stack and spell (v34); per
    /// side, the rounds since its periodic cast (v37).
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_bytes(&self.left);
        self.drawn.map(|(s, _)| s.0).hash_into(h);
        self.drawn.map(|(_, spell)| spell.raw()).hash_into(h);
        h.write_bytes(&self.periodic);
    }
}

/// The casts per battle of `creature` (its pack's `casts`).
pub(crate) fn per_battle(ruleset: &BattleRuleset, creature: CreatureTypeId) -> u8 {
    ruleset.creature(creature).map_or(0, |c| c.casts)
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
    ruleset: &BattleRuleset,
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

/// At a round start (BAT-MAG-053; the original's 0x4760F8): each side's rounds since its periodic
/// cast rise by 1, up to 255.
// rule: BAT-MAG-053
pub(crate) fn round_started(state: &mut BattleState) {
    for rounds in &mut state.casts.periodic {
        *rounds = rounds.saturating_add(1);
    }
}

/// At the activation of stack `id` for its turn (BAT-MAG-053; the original's 0x465330 and
/// 0x447FE0, `installation.md` §7.21): a periodic caster whose controller's side has had its
/// creature's `rounds` round starts since its last periodic cast casts on its own a spell drawn
/// by weight among its creature's `drawn_spell` entries that count ([`counts`]), at its mastery
/// with its power and no target ([`magic::periodic_cast`]: the mass version, no cast used), and
/// that side's count returns to 0. If none counts, nothing is cast and the count stays. The
/// stack's turn goes on (called for the activations of `turn`, which skip a stack that lost its
/// turns during its own action).
// rule: BAT-MAG-053
pub(crate) fn periodic<S: Sink>(
    state: &mut BattleState,
    ruleset: &BattleRuleset,
    id: StackId,
    sink: &mut S,
) {
    let stack = *state.at(id);
    // Most stacks are no periodic casters: test that first.
    if !stack.has(StackTraits::PERIODIC_CASTER) {
        return;
    }
    let side = stack.controller();
    let abilities = ruleset.abilities(stack.creature);
    // A periodic caster has its ability (the trait comes from it).
    let (rounds, mastery, power) = abilities
        .iter()
        .find_map(|a| match *a {
            Ability::PeriodicSpellCaster {
                rounds,
                mastery,
                power,
            } => Some((rounds, mastery, power)),
            _ => None,
        })
        .unwrap_or((u8::MAX, 0, 0));
    // rule: BAT-MAG-061, BAT-TER-004
    if state.casts.periodic[side.index()] < rounds || magic::creature_casting_forbidden(state) {
        return;
    }
    let first_only = ruleset.quirks().is_enabled(PERIODIC_FIRST_STACK);
    // The original counts the weights, then walks its table again with the same test.
    let total = periodic_spells(state, ruleset, (id, side), first_only)
        .map(|(_, w)| w)
        .sum::<u32>();
    if total == 0 {
        return;
    }
    let mut r = state.rng.below(total);
    let drawn = periodic_spells(state, ruleset, (id, side), first_only).find(|&(_, w)| {
        let hit = r < w;
        r = r.saturating_sub(w);
        hit
    });
    drawn.into_iter().for_each(|(spell, _)| {
        state.casts.periodic[side.index()] = 0;
        // rule: BAT-TER-005
        let mastery = magic::other_mastery(state, mastery);
        let cast = magic::periodic_cast(side, (id, spell), (mastery, power));
        magic::apply_cast(state, ruleset, cast, sink);
    });
}

/// The drawn spells of the periodic caster `caster`'s creature that count for `side`
/// ([`counts`]), with their weights, in their creature's order (BAT-MAG-053).
fn periodic_spells<'s, 'r: 's>(
    state: &'s BattleState,
    ruleset: &'r BattleRuleset,
    (caster, side): (StackId, BattleSide),
    first_only: bool,
) -> impl Iterator<Item = ((SpellId, &'r BattleSpell), u32)> + 's {
    ruleset
        .abilities(state.at(caster).creature)
        .iter()
        .filter_map(move |a| match *a {
            Ability::DrawnSpell { spell, weight } => ruleset
                .spell(spell)
                .filter(|sp| counts(state, ruleset, (side, first_only), (spell, sp)))
                .map(|sp| ((spell, sp), u32::from(weight))),
            _ => None,
        })
}

/// Whether `spell` counts for a periodic caster of `side` (BAT-MAG-053; the original's 0x447FE0):
/// it is useful ([`useful`], for `side`) on a stack of the side it aims at, `side` or, for a
/// negative spell, the other side; with quirk BAT-QUIRK-13 (`first_only`) on that side's first
/// stack, its lowest id, alive or not (the original tests its record 0, survey S-12).
// rule: BAT-MAG-053
fn counts(
    state: &BattleState,
    ruleset: &BattleRuleset,
    (side, first_only): (BattleSide, bool),
    spell: (SpellId, &BattleSpell),
) -> bool {
    let aimed = if spell.1.polarity == SpellPolarity::Negative {
        side.opponent()
    } else {
        side
    };
    let mut stacks = (0u8..)
        .take(state.stacks.len())
        .map(StackId)
        .filter(|&s| state.at(s).side == aimed);
    if first_only {
        // Quirk BAT-QUIRK-13 (survey S-12).
        stacks
            .next()
            .is_some_and(|s| useful(state, ruleset, side, spell, s))
    } else {
        stacks.any(|s| useful(state, ruleset, side, spell, s))
    }
}

/// The active stack and its spell, if `actor` commands it outside the tactics phase and it may
/// cast now (BAT-MAG-048): its creature is a caster, it has a cast left and no hero in the battle
/// forbids casting and the battlefield allows creature casts, unless its creature has
/// `ignores_magic_block` (BAT-MAG-061, BAT-TER-004: the Faerie Dragon's key path, the user's
/// BAT-Q32 (2)).
fn ready(
    state: &BattleState,
    ruleset: &BattleRuleset,
    actor: BattleSide,
) -> Option<(StackId, CreatureSpell)> {
    // Most stacks have no casts: test that first (the random agent asks at every step).
    let id = state.active.filter(|id| state.casts.left[id.index()] > 0)?;
    let stack = state.at(id);
    // rule: BAT-MAG-061, BAT-TER-004
    let blocked = || {
        magic::creature_casting_forbidden(state)
            && !ruleset
                .abilities(stack.creature)
                .contains(&Ability::IgnoresMagicBlock)
    };
    if state.tactics.is_some() || stack.controller() != actor || blocked() {
        return None;
    }
    stack_cast(state, ruleset, id).map(|spell| (id, spell))
}

/// The spell stack `id` casts as its action, if its creature is a caster (BAT-MAG-048), whether or
/// not it may cast now; for a random caster the spell it drew, while it is the active stack
/// (BAT-MAG-049); for a helpful caster none before the cast (BAT-MAG-050).
pub fn stack_spell(state: &BattleState, ruleset: &BattleRuleset, id: StackId) -> Option<CastSpell> {
    stack_cast(state, ruleset, id).map(|spell| spell.spell)
}

/// The spell the active stack casts if `actor` commands its cast now (BAT-MAG-048 to
/// BAT-MAG-050): `None` if it may not cast (not `actor`'s stack, the tactics phase, not a caster,
/// no casts left). Whether it has a target is [`creature_cast_targets`]'s.
pub fn creature_cast_spell(
    state: &BattleState,
    ruleset: &BattleRuleset,
    actor: BattleSide,
) -> Option<CastSpell> {
    ready(state, ruleset, actor).map(|(_, spell)| spell.spell)
}

/// Whether `spell` is useful on stack `s` for a helpful caster of `side` (BAT-MAG-050; the
/// original's 0x447A80, `installation.md` §7.21): it is not active on the stack, it can affect the
/// stack (`magic::affectable`: a positive spell an own stack, immunities, Anti-Magic), and its
/// [`Usefulness`] holds. The enemy is the other side; its stacks are those it owns.
// rule: BAT-MAG-050
pub(crate) fn useful(
    state: &BattleState,
    ruleset: &BattleRuleset,
    side: BattleSide,
    (id, spell): (SpellId, &BattleSpell),
    s: StackId,
) -> bool {
    let field = Field::new(ruleset.battle().field);
    let stack = state.at(s);
    let enemies = || {
        (0u8..)
            .take(state.stacks.len())
            .map(StackId)
            .filter(move |&e| state.at(e).side != side)
    };
    // An enemy that can fight: alive, not blind, petrified or paralysed, and no war machine that
    // heals or supplies (the original leaves out creatures 147 and 148).
    let fighters = || {
        enemies().filter(|&e| {
            let enemy = state.at(e);
            enemy.is_alive()
                && !enemy.spells.status.contains(Status::NO_TURNS)
                && !enemy.has(StackTraits::FIRST_AID)
                && !enemy.has(StackTraits::AMMO_SUPPLY)
        })
    };
    !state.effects.of(s).iter().any(|e| e.spell == id)
        && magic::affectable(state, ruleset, side, (id, spell), s)
        && match spell.magic.useful {
            Usefulness::Always => true,
            Usefulness::EnemyMelee => fighters().any(|e| !combat::can_shoot(state, field, e)),
            Usefulness::EnemyShooter => fighters().any(|e| combat::can_shoot(state, field, e)),
            Usefulness::EnemySpellBook => state
                .hero(side.opponent())
                .is_some_and(|hero| hero.spell_book),
            Usefulness::Wounded => stack.top_hp < stack.max_hp,
            Usefulness::Melee => !combat::can_shoot(state, field, s),
            Usefulness::Shooting => combat::can_shoot(state, field, s),
            Usefulness::NotActed => !stack.flags.contains(StackFlags::ACTED),
            // The original's test: dead stacks count, and only the first king level.
            Usefulness::EnemyKing => enemies().any(|e| {
                ruleset
                    .creature(state.at(e).creature)
                    .is_some_and(|c| c.flags.contains(CreatureFlag::King1))
            }),
        }
}

/// The drawn spells of stack `caster`'s creature that are useful on stack `s` for `side`
/// ([`useful`]), with their weights, in their creature's order (BAT-MAG-050).
pub(crate) fn useful_spells<'a>(
    state: &'a BattleState,
    ruleset: &'a BattleRuleset,
    side: BattleSide,
    (caster, s): (StackId, StackId),
) -> impl Iterator<Item = (SpellId, u32)> + 'a {
    ruleset
        .abilities(state.at(caster).creature)
        .iter()
        .filter_map(move |a| match *a {
            Ability::DrawnSpell { spell, weight } => ruleset
                .spell(spell)
                .filter(|sp| useful(state, ruleset, side, (spell, sp), s))
                .map(|_| (spell, u32::from(weight))),
            _ => None,
        })
}

/// Per stack, how many spells the active stack's cast could draw for it, if `viewer` commands a
/// helpful caster that may cast now (0 for every stack otherwise and for stacks that are no
/// target; BAT-MAG-050, the user: shown to agents, BAT-Q11).
// rule: BAT-MAG-050
pub(crate) fn candidates(
    state: &BattleState,
    ruleset: &BattleRuleset,
    viewer: BattleSide,
) -> [u8; STACK_CAPACITY] {
    let mut out = [0; STACK_CAPACITY];
    let Some((caster, spell)) =
        ready(state, ruleset, viewer).filter(|(_, spell)| spell.spell == CastSpell::AtResolution)
    else {
        return out;
    };
    for (i, n) in out.iter_mut().enumerate().take(state.stacks.len()) {
        let s = StackId(u8::try_from(i).unwrap_or(u8::MAX));
        let target = CastTarget::Stack(s);
        if magic::creature_target(state, ruleset, viewer, (caster, spell), target) {
            let count = useful_spells(state, ruleset, viewer, (caster, s)).count();
            *n = u8::try_from(count).unwrap_or(u8::MAX);
        }
    }
    out
}

/// Fills `out` with every target [`crate::apply`] accepts for `actor`'s
/// [`Command::CreatureCast`](crate::Command::CreatureCast) now, stacks by id (BAT-MAG-048):
/// empty if the active stack may not cast ([`creature_cast_spell`]).
pub fn creature_cast_targets(
    state: &BattleState,
    ruleset: &BattleRuleset,
    actor: BattleSide,
    out: &mut CastTargets,
) {
    out.clear();
    let Some((caster, spell)) = ready(state, ruleset, actor) else {
        return;
    };
    magic::creature_targets(state, ruleset, actor, (caster, spell), out);
}

/// A creature's cast as its action, validated ([`plan`]).
pub(crate) enum CreatureAction<'r> {
    /// A spell cast (BAT-MAG-048 to BAT-MAG-050).
    Cast(Cast<'r>),
    /// A raise of the stack or corpse `1` by the raiser `0` (BAT-MAG-051, BAT-MAG-052).
    Raise(StackId, StackId),
}

/// Validates `actor`'s creature cast at `target` (BAT-MAG-048): the active stack, which `actor`
/// commands outside the tactics phase, is a caster with a cast left, and `target` is a target of
/// its spell or its raise.
pub(crate) fn plan<'r>(
    state: &BattleState,
    ruleset: &'r BattleRuleset,
    actor: BattleSide,
    target: CastTarget,
) -> Result<CreatureAction<'r>, CommandError> {
    let (caster, spell) = ready(state, ruleset, actor).ok_or(CommandError::CannotCast)?;
    if spell.spell == CastSpell::Raise {
        return target
            .stack()
            .filter(|_| raise_target(state, ruleset, actor, caster, target))
            .map(|t| CreatureAction::Raise(caster, t))
            .ok_or(CommandError::InvalidSpellTarget);
    }
    magic::plan_creature_cast(state, ruleset, actor, (caster, spell), target)
        .map(CreatureAction::Cast)
}

/// Whether `target` is a target of the raiser `caster`'s raise for `side`, its controller
/// (BAT-MAG-051, BAT-MAG-052; the original's 0x4476C0 and 0x4473D0, case 0, whose cursor never
/// offers the caster, `installation.md` §7.21): another stack; for a stack raiser one that its
/// test's raise spell with `per_creature` × the raiser's creatures hit points may raise
/// ([`raise::raisable`]: of `side`, living or a corpse on top, having lost creatures, whose
/// health the hit points reach); for a corpse raiser a corpse of `side` it may raise
/// ([`raise::corpse_raisable`]) into at least one creature ([`corpse_count`]), while `side` has a
/// slot for the new stack ([`created::has_slot`]: its template, and a killed created stack's slot
/// or room).
// rule: BAT-MAG-051, BAT-MAG-052
pub(crate) fn raise_target(
    state: &BattleState,
    ruleset: &BattleRuleset,
    side: BattleSide,
    caster: StackId,
    target: CastTarget,
) -> bool {
    let raiser_stack = state.at(caster);
    let Some(t) = target
        .stack()
        .filter(|&t| t != caster && state.stack(t).is_some())
    else {
        return false;
    };
    raiser(ruleset, raiser_stack.creature).is_some_and(|r| match r {
        Raiser::Stack { test, per_creature } => ruleset.spell(test).is_some_and(|spell| {
            let hit_points = u64::from(per_creature) * u64::from(raiser_stack.count);
            let test = raise::creature_test((test, spell), hit_points);
            raise::raisable(state, ruleset, side, test, t)
        }),
        Raiser::Corpse {
            creature,
            per_creature,
        } => {
            raise::corpse_raisable(state, ruleset, side, t)
                && corpse_count(state, ruleset, (caster, t), (creature, per_creature)) > 0
                && created::has_slot(state, ruleset, (side, creature))
        }
    })
}

/// How many creatures of `creature` the corpse raiser `caster` raises from the corpse of stack
/// `corpse` (BAT-MAG-052; the original's 0x447330): `min(start, min(per_creature × the raiser's
/// creatures, start × health) div the creature's health)`, with the corpse's start count and its
/// creatures' health and the raised creature's base health.
pub(crate) fn corpse_count(
    state: &BattleState,
    ruleset: &BattleRuleset,
    (caster, corpse): (StackId, StackId),
    (creature, per_creature): (CreatureTypeId, u16),
) -> u32 {
    let dead = state.at(corpse);
    let start = u64::from(dead.base_count);
    let budget = u64::from(per_creature) * u64::from(state.at(caster).count);
    let health = ruleset
        .creature(creature)
        .map_or(1, |c| c.hit_points)
        .max(1);
    let raised = budget.min(start * u64::from(dead.max_hp)) / u64::from(health);
    u32::try_from(raised.min(start)).unwrap_or(u32::MAX)
}

/// The creatures the raiser `caster` raises from stack `target` by the computer's count (the
/// original's 0x447330, `installation.md` §7.43; E23 T4a): a stack raiser's `min(lost creatures,
/// per creature × its creatures div the target's hit points per creature)`, whole creatures only;
/// a corpse raiser's [`corpse_count`]; 0 for a stack that raises nothing.
pub(crate) fn raise_count(
    state: &BattleState,
    ruleset: &BattleRuleset,
    (caster, target): (StackId, StackId),
) -> u32 {
    let (raiser_stack, t) = (state.at(caster), state.at(target));
    raiser(ruleset, raiser_stack.creature).map_or(0, |r| match r {
        Raiser::Stack { per_creature, .. } => {
            let budget = u64::from(per_creature) * u64::from(raiser_stack.count);
            let whole = budget / u64::from(t.max_hp.max(1));
            let lost = u64::from(t.base_count.saturating_sub(t.count));
            u32::try_from(whole.min(lost)).unwrap_or(u32::MAX)
        }
        Raiser::Corpse {
            creature,
            per_creature,
        } => corpse_count(state, ruleset, (caster, target), (creature, per_creature)),
    })
}

/// The raiser `caster` raises stack or corpse `target` as its action (BAT-MAG-051,
/// BAT-MAG-052): one of its casts is used (`CreatureRaise`); a stack raiser raises the stack by
/// its hit points for good (the original's 0x5A7560 with temporary 0), a corpse raiser raises a
/// stack of its creature for its controller ([`created::raise_corpse`]).
// rule: BAT-MAG-051, BAT-MAG-052
pub(crate) fn raise<S: Sink>(
    state: &mut BattleState,
    ruleset: &BattleRuleset,
    (caster, target): (StackId, StackId),
    sink: &mut S,
) {
    spend(state, caster);
    sink.emit(Event::public(EventKind::CreatureRaise {
        stack: caster,
        target,
    }));
    let stack = *state.at(caster);
    // Validated: the caster is a raiser.
    raiser(ruleset, stack.creature)
        .into_iter()
        .for_each(|r| match r {
            Raiser::Stack { per_creature, .. } => {
                let hit_points = u64::from(per_creature) * u64::from(stack.count);
                raise::raise(state, ruleset, (target, hit_points), false, sink);
            }
            Raiser::Corpse {
                creature,
                per_creature,
            } => {
                let count =
                    corpse_count(state, ruleset, (caster, target), (creature, per_creature));
                let side = stack.controller();
                created::raise_corpse(state, ruleset, (side, target), (creature, count), sink);
            }
        });
}

/// Uses one of stack `s`'s casts (the original lowers them first, 0x448260).
pub(crate) fn spend(state: &mut BattleState, s: StackId) {
    let left = &mut state.casts.left[s.index()];
    *left = left.saturating_sub(1);
}
