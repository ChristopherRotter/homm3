//! The spells a creature casts on the stack it strikes in melee (E04b T7a; ADR-0012, ADR-0014
//! points 1 and 3): Unicorns' Blind, Black and Dread Knights' and Mummies' Curse, the flies'
//! Dispel Helpful Spells and Weakness, the Thunderbird's thunderbolt.
//!
//! The original's strike routine (0x441610, `docs/formats/installation.md` §7.20) runs them
//! after the strike's damage, for every melee strike (the attack, the retaliation, the second
//! strike) and never for shots: a creature's cast without a hero at the ability's mastery, its
//! timed effects lasting the ability's rounds (the original's power argument), with the target's
//! immunities, Anti-Magic, magic resistance and resistance aura as for a hero's spell, but
//! without the test that keeps a negative spell off the caster's own side (a hypnotized or
//! berserk striker's own stacks may be struck) and without Magic Mirror.

use h3_content::{Ability, Ruleset, Spell, SpellId};

use crate::damage::apply_damage;
use crate::effects::{self, TimedCast};
use crate::event::{Event, EventKind, Sink};
use crate::magic::{self, SpellBonus, SpellBonuses};
use crate::state::{BattleState, StackId, StackTraits};

/// Casts the spells of `striker`'s abilities on `target`, which it struck in melee, in ability
/// order (BAT-MAG-040, BAT-MAG-041). Returns whether one of them stops the target's turns
/// (Blind, Stone Gaze, Paralyze): the target then does not retaliate (the original's strike answers "no
/// retaliation", `installation.md` §7.20).
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
                defended: _,
            } => {
                // A compiled ability names a spell of the ruleset.
                let landed = ruleset.spell(spell).filter(|data| {
                    let test = (chance_percent, living, unless_active);
                    spell_after_strike(state, ruleset, (spell, data), test, target)
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
                    damage_lands(state, ruleset, (spell, data), chance_percent, target)
                });
                if let Some(data) = landed {
                    let pair = (striker, target);
                    thunderbolt(state, ruleset, (spell, data), per_creature, pair, sink);
                }
            }
            _ => {}
        }
    }
    stopped
}

/// Whether a spell after a strike lands on `target` (BAT-MAG-040): a stack of a living creature
/// if the ability says so; the ability's chance (one draw unless 100 %); the target alive; not
/// already under the spell if the ability says so; then the creature's test: for a dispel of
/// positive spells an effect it ends (BAT-MAG-042), otherwise the spell can affect the stack
/// (immunities, Anti-Magic) and the stack does not resist it (its own draw; BAT-MAG-005,
/// BAT-MAG-008, BAT-MAG-021).
// rule: BAT-MAG-040
fn spell_after_strike(
    state: &mut BattleState,
    ruleset: &Ruleset,
    (id, spell): (SpellId, &Spell),
    (chance, living, unless_active): (u8, bool, bool),
    target: StackId,
) -> bool {
    if living && !state.at(target).has(StackTraits::ALIVE) {
        return false;
    }
    if chance < 100 && state.rng.below(100) >= u32::from(chance) {
        return false;
    }
    let active = |state: &BattleState| state.effects.of(target).iter().any(|e| e.spell == id);
    if !state.at(target).is_alive() || (unless_active && active(state)) {
        return false;
    }
    if spell.magic.dispels_positive {
        // rule: BAT-MAG-042
        return state
            .effects
            .of(target)
            .iter()
            .any(|e| effects::dispel_ends(ruleset, spell, e));
    }
    magic::affects_stack(state, ruleset, (id, spell), target)
        && !magic::resists(state, ruleset, spell, target)
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
    target: StackId,
) -> bool {
    state.at(target).is_alive()
        && (chance >= 100 || state.rng.below(100) < u32::from(chance))
        && magic::affects_stack(state, ruleset, (id, spell), target)
        && !magic::resists(state, ruleset, spell, target)
}

/// Deals a damage spell after a strike that landed (BAT-MAG-041): `per_creature` per creature of
/// the striker's stack as the spell's damage without a hero (the target's extra damage,
/// reduction and protection, BAT-MAG-004), reported, then what damage ends (BAT-MAG-024,
/// BAT-MAG-038).
fn thunderbolt<S: Sink>(
    state: &mut BattleState,
    ruleset: &Ruleset,
    (id, spell): (SpellId, &Spell),
    per_creature: u16,
    (striker, target): (StackId, StackId),
    sink: &mut S,
) {
    sink.emit(Event::public(EventKind::StrikeSpell {
        striker,
        target,
        spell: id,
    }));
    let value = u64::from(state.at(striker).count) * u64::from(per_creature);
    let victim = state.at(target);
    let protection = victim.spells.school_protection;
    let (caster, damage) = magic::spell_damage(
        ruleset,
        ((id, spell), &SpellBonuses::default()),
        value,
        (victim.creature, &protection),
    );
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
