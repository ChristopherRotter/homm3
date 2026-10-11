//! The spells a creature casts on the stack it strikes in melee (E04b T7a; ADR-0012, ADR-0014
//! points 1 and 3): Unicorns' Blind, Black and Dread Knights' and Mummies' Curse, the flies'
//! Dispel Helpful Spells and Weakness, the Thunderbird's thunderbolt, the strike's statuses
//! (Stone Gaze, Paralyze, Disease, Age, Poison), the Dendroids' Bind, the Rust Dragon's Acid
//! Breath and extra damage.
//!
//! The original's strike routine (0x441610, `docs/formats/installation.md` §7.20) runs them
//! after the strike's damage, for every melee strike (the attack, the retaliation, the second
//! strike) and never for shots: a creature's cast without a hero at the ability's mastery, its
//! timed effects lasting the ability's rounds (the original's power argument), with the target's
//! immunities, Anti-Magic, magic resistance and resistance aura as for a hero's spell, but
//! without the test that keeps a negative spell off the caster's own side (a hypnotized or
//! berserk striker's own stacks may be struck) and without Magic Mirror. Bind and Acid Breath
//! (spells that ignore immunity) are cast without any test.

use h3_content::{Ability, Ruleset, Spell, SpellEffect, SpellId, TimedChange};

use crate::binds;
use crate::damage::apply_damage;
use crate::effects::{self, TimedCast};
use crate::event::{Event, EventKind, Sink};
use crate::magic::{self, SpellBonus, SpellBonuses};
use crate::state::{BattleState, StackId, StackTraits};

/// What a spell after a strike needs of the struck stack besides the spell's test
/// ([`Ability::SpellAfterStrike`]).
#[derive(Debug, Clone, Copy)]
struct Conditions {
    /// The chance per strike, percent (100: no draw).
    chance: u8,
    /// A living creature.
    living: bool,
    /// Not already under the spell.
    unless_active: bool,
    /// A defense above 0.
    defended: bool,
}

/// Casts the spells of `striker`'s abilities on `target`, which it struck in melee, and deals
/// their extra damage, in ability order (BAT-MAG-040, BAT-MAG-041, BAT-MAG-045 to BAT-MAG-047).
/// Returns whether one of them stops the target's turns (Blind, Stone Gaze, Paralyze): the
/// target then does not retaliate (the original's strike answers "no retaliation",
/// `installation.md` §7.20).
pub(crate) fn cast<S: Sink>(
    state: &mut BattleState,
    ruleset: &Ruleset,
    (striker, target): (StackId, StackId),
    sink: &mut S,
) -> bool {
    let mut stopped = false;
    for ability in ruleset.abilities(state.at(striker).creature) {
        match *ability {
            Ability::SpellAfterStrike {
                spell,
                chance_percent,
                mastery,
                rounds,
                living,
                unless_active,
                defended,
            } => {
                let test = Conditions {
                    chance: chance_percent,
                    living,
                    unless_active,
                    defended,
                };
                // A compiled ability names a spell of the ruleset.
                let landed = ruleset.spell(spell).filter(|data| {
                    spell_after_strike(state, ruleset, (spell, data), test, (striker, target))
                });
                if let Some(data) = landed {
                    sink.emit(Event::public(EventKind::StrikeSpell {
                        striker,
                        target,
                        spell,
                    }));
                    let cast = (mastery, rounds);
                    stopped |= apply(state, ruleset, (spell, data), cast, target, sink);
                }
            }
            Ability::SpellDamageAfterStrike {
                spell,
                chance_percent,
                per_creature,
            } => {
                let landed = ruleset.spell(spell).filter(|data| {
                    damage_lands(
                        state,
                        ruleset,
                        (spell, data),
                        chance_percent,
                        (striker, target),
                    )
                });
                if let Some(data) = landed {
                    let pair = (striker, target);
                    thunderbolt(state, ruleset, (spell, data), per_creature, pair, sink);
                }
            }
            Ability::DamageAfterStrike {
                chance_percent,
                percent,
            } => {
                let pair = (striker, target);
                extra_damage(state, ruleset, pair, (chance_percent, percent), sink);
            }
            _ => {}
        }
    }
    stopped
}

/// Whether a spell after a strike lands on `target` (BAT-MAG-040): a stack of a living creature
/// if the ability says so; the ability's chance (one draw unless 100 %); the target alive; not
/// already under the spell, and with a defense above 0, if the ability says so (BAT-MAG-046);
/// then the creature's test: for a dispel of positive spells an effect it ends (BAT-MAG-042);
/// for a spell that ignores immunity none, but Bind only from a striker not yet among the
/// target's binders, which joins them (BAT-MAG-045); otherwise the spell can affect the stack
/// (immunities, Anti-Magic) and the stack does not resist it (its own draw; BAT-MAG-005,
/// BAT-MAG-008, BAT-MAG-021).
// rule: BAT-MAG-040
fn spell_after_strike(
    state: &mut BattleState,
    ruleset: &Ruleset,
    (id, spell): (SpellId, &Spell),
    c: Conditions,
    (striker, target): (StackId, StackId),
) -> bool {
    if c.living && !state.at(target).has(StackTraits::ALIVE) {
        return false;
    }
    if c.chance < 100 && state.rng.below(100) >= u32::from(c.chance) {
        return false;
    }
    let stack = state.at(target);
    let active = || state.effects.of(target).iter().any(|e| e.spell == id);
    // rule: BAT-MAG-046
    if !stack.is_alive()
        || (c.unless_active && active())
        || (c.defended && stack.defense_now() <= 0)
    {
        return false;
    }
    // The original tests a striker's spell for the side that controls it (0x5A8640).
    let caster = state.at(striker).controller();
    if spell.magic.dispels_positive {
        // rule: BAT-MAG-042, BAT-MAG-063
        return !magic::Guard::of(state, caster, target).immune_to(id)
            && state
                .effects
                .of(target)
                .iter()
                .any(|e| effects::dispel_ends(ruleset, spell, e));
    }
    if spell.magic.ignores_immunity {
        // rule: BAT-MAG-045
        let bind = SpellEffect::Timed(TimedChange::Bind);
        return !spell.magic.effects.contains(&bind) || binds::join(state, striker, target);
    }
    magic::affects_stack(state, ruleset, caster, (id, spell), target)
        && !magic::resists(state, ruleset, caster, spell, target)
}

/// Applies a spell after a strike to `target` at `mastery`, its timed effects lasting `rounds`
/// (BAT-MAG-040): its dispel (BAT-MAG-016, BAT-MAG-042), then its timed changes without a
/// hero's bonuses (BAT-MAG-009). Returns whether it stops the target's turns.
fn apply<S: Sink>(
    state: &mut BattleState,
    ruleset: &Ruleset,
    (id, spell): (SpellId, &Spell),
    (mastery, rounds): (u8, u8),
    target: StackId,
    sink: &mut S,
) -> bool {
    if spell.magic.dispels() {
        effects::dispel(state, ruleset, spell, target, sink);
    }
    if !spell.magic.is_timed() {
        return false;
    }
    let cast = TimedCast {
        id,
        spell,
        mastery,
        rounds: u16::from(rounds),
        own: SpellBonus::default(),
    };
    effects::apply(state, ruleset, cast, target, sink);
    spell
        .magic
        .effects
        .iter()
        .any(|e| e.timed().is_some_and(effects::stops_turns))
}

/// Whether a damage spell after a strike lands on `target` (BAT-MAG-041, the Thunderbird): the
/// stack is alive, the ability's chance (one draw unless 100 %), then the spell's test as in
/// [`spell_after_strike`].
// rule: BAT-MAG-041
fn damage_lands(
    state: &mut BattleState,
    ruleset: &Ruleset,
    (id, spell): (SpellId, &Spell),
    chance: u8,
    (striker, target): (StackId, StackId),
) -> bool {
    let caster = state.at(striker).controller();
    state.at(target).is_alive()
        && (chance >= 100 || state.rng.below(100) < u32::from(chance))
        && magic::affects_stack(state, ruleset, caster, (id, spell), target)
        && !magic::resists(state, ruleset, caster, spell, target)
}

/// Deals a damage spell after a strike that landed (BAT-MAG-041): `per_creature` per creature of
/// the striker's stack as the spell's damage without a hero (the target's extra damage,
/// reduction and protection, BAT-MAG-004); none if that is 0 (the original leaves when the
/// damage is not above 0, 0x4411FD: no report, as [`extra_damage`]). Reported, then what damage
/// ends (BAT-MAG-024, BAT-MAG-038).
fn thunderbolt<S: Sink>(
    state: &mut BattleState,
    ruleset: &Ruleset,
    (id, spell): (SpellId, &Spell),
    per_creature: u16,
    (striker, target): (StackId, StackId),
    sink: &mut S,
) {
    let value = u64::from(state.at(striker).count) * u64::from(per_creature);
    let victim = state.at(target);
    let protection = victim.spells.school_protection;
    let (caster, damage) = magic::spell_damage(
        ruleset,
        ((id, spell), &SpellBonuses::default()),
        value,
        (victim.creature, &protection),
    );
    if damage == 0 {
        return;
    }
    sink.emit(Event::public(EventKind::StrikeSpell {
        striker,
        target,
        spell: id,
    }));
    let kills = apply_damage(state.at_mut(target), damage);
    sink.emit(Event::public(EventKind::SpellDamage {
        target,
        value,
        caster,
        damage,
        kills,
    }));
    // rule: BAT-MAG-024, BAT-MAG-038
    effects::damaged(state, ruleset, target, sink);
}

/// Deals the striker's extra damage after a strike (BAT-MAG-047; the Rust Dragon, the original's
/// 0x440BC0): on a living target, the ability's chance (one draw unless 100 %), then `percent` %
/// of one roll of the striker's damage range times its creatures, truncated, without a test;
/// none if that is 0. Reported, then what damage ends (BAT-MAG-024, BAT-MAG-038).
// rule: BAT-MAG-047
fn extra_damage<S: Sink>(
    state: &mut BattleState,
    ruleset: &Ruleset,
    (striker, target): (StackId, StackId),
    (chance, percent): (u8, u8),
    sink: &mut S,
) {
    if !state.at(target).is_alive() || (chance < 100 && state.rng.below(100) >= u32::from(chance)) {
        return;
    }
    let s = *state.at(striker);
    let roll = state
        .rng
        .range_inclusive(s.damage_min.into(), s.damage_max.into());
    let damage = u64::from(roll) * u64::from(s.count) * u64::from(percent) / 100;
    if damage == 0 {
        return;
    }
    let kills = apply_damage(state.at_mut(target), damage);
    sink.emit(Event::public(EventKind::StrikeDamage {
        striker,
        target,
        damage,
        kills,
    }));
    // rule: BAT-MAG-024, BAT-MAG-038
    effects::damaged(state, ruleset, target, sink);
}
