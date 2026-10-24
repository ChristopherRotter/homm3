//! Attacks and their sequences: melee attacks with their abilities (BAT-ATK-001..004,
//! BAT-RET-001..003, BAT-DMG-004..009), shots (BAT-RNG-001..004) and the end of the battle
//! (BAT-END-001).
//!
//! An *attack* is one swing of a stack: one death-blow roll, a strike on the target, strikes on
//! the secondary targets its attack pattern hits, then life drain and death stare. A melee attack
//! (BAT-ATK-001) is the attack, the target's retaliation, and for creatures that strike twice a
//! second attack.

use h3_content::{
    CreatureFlag, FIRE_SHIELD_FIRST_STAGE, FRENZY_REDUCED_DEFENSE, FULL_BP, SINGLE_PRECISION_RATES,
};
use h3_content_battle::{Ability, BattleParams, BattleRuleset, CompiledAbility, TimedChange};
use h3_contracts::{BattleResult, BattleSide};
use h3_kernel::arith::mul_div_floor;

use crate::damage::{
    Strike, apply_damage, base_damage, effective_defense, frenzied, reduced_defense, resurrect,
};
use crate::effects::{self, DamageRoll, Status};
use crate::event::{Event, EventKind, Sink, StrikeFlags};
use crate::geometry::{Direction, Field, Footprint, Hex, HexSet};
use crate::hero;
use crate::magic::Specialist;
use crate::state::{
    BattleState, STACK_CAPACITY, Stack, StackFlags, StackId, StackTraits, UNLIMITED_RETALIATIONS,
};
use crate::{after_strike, binds, magic, morale, war};

/// The secondary targets of an attack, by stack index.
type Targets = [bool; STACK_CAPACITY];

/// No secondary targets.
const NO_TARGETS: Targets = [false; STACK_CAPACITY];

/// Whether stack `id` is next to a living enemy (its footprint touches one; enemies as
/// [`Stack::is_hostile_to`] has them, BAT-MAG-025/026: a berserk neighbour of any side, a stack
/// of the side its controller fights), which keeps a shooter from shooting (BAT-RNG-001).
pub(crate) fn is_blocked(state: &BattleState, field: Field, id: StackId) -> bool {
    let me = state.at(id);
    let fp = me.footprint(field);
    (0u8..).zip(state.stacks.iter()).any(|(i, s)| {
        i != id.0 && s.is_alive() && me.is_hostile_to(s) && field.touches(fp, s.footprint(field))
    })
}

/// Whether stack `id` can shoot now: a shooter with shots left that is not blocked
/// (BAT-RNG-001) and has not forgotten how (Forgetfulness from Advanced, BAT-MAG-023); war
/// machines and stacks whose hero lets them shoot while adjacent shoot even when blocked
/// (BAT-WAR-001, BAT-HER-006), and under quirk Q-0004 the shooting war machine also without
/// shots (BAT-WAR-002).
// rule: BAT-RNG-001, BAT-WAR-001, BAT-MAG-023
pub(crate) fn can_shoot(state: &BattleState, field: Field, id: StackId) -> bool {
    let s = state.at(id);
    s.has(StackTraits::SHOOTER)
        && !s.spells.status.contains(Status::NO_SHOOTING)
        && has_ammunition(s)
        && (s.has(StackTraits::WAR_MACHINE)
            || s.has(StackTraits::SHOOTS_WHEN_ADJACENT)
            || !is_blocked(state, field, id))
}

/// Whether stack `id` could shoot now with `ignore` left out of its blockers ([`can_shoot`]
/// otherwise): the computer's test of a shooter that a walker would block from a hex beside it
/// (0x4428F0 with the walker, 0x469600; E23 T3a). Kept apart from [`can_shoot`], a hot path.
pub(crate) fn can_shoot_without(
    state: &BattleState,
    field: Field,
    id: StackId,
    ignore: StackId,
) -> bool {
    let s = state.at(id);
    let fp = s.footprint(field);
    let blocked = (0u8..).zip(state.stacks.iter()).any(|(i, t)| {
        i != id.0
            && i != ignore.0
            && t.is_alive()
            && s.is_hostile_to(t)
            && field.touches(fp, t.footprint(field))
    });
    s.has(StackTraits::SHOOTER)
        && !s.spells.status.contains(Status::NO_SHOOTING)
        && has_ammunition(s)
        && (s.has(StackTraits::WAR_MACHINE) || s.has(StackTraits::SHOOTS_WHEN_ADJACENT) || !blocked)
}

/// Whether `stack` has a shot to use: shots left, or none needed (quirk Q-0004, BAT-WAR-002).
fn has_ammunition(stack: &Stack) -> bool {
    stack.shots > 0 || stack.has(StackTraits::UNLIMITED_SHOTS)
}

/// Whether a living stack of `side` supplies ammunition (BAT-WAR-004).
// rule: BAT-WAR-004
fn ammo_supplied(state: &BattleState, side: BattleSide) -> bool {
    state
        .stacks
        .iter()
        .any(|s| s.side == side && s.is_alive() && s.has(StackTraits::AMMO_SUPPLY))
}

/// The creatures of `stack` that count for the damage of its strikes: all, or for a forgetful
/// shooter half of them, rounded down, at least one, whatever the strike (BAT-MAG-023; survey
/// S-09: the original halves its melee strikes and retaliations too, the rule in every profile).
// rule: BAT-MAG-023
fn striking_count(stack: &Stack) -> u32 {
    if stack.spells.status.contains(Status::FORGETFUL) && stack.has(StackTraits::SHOOTER) {
        (stack.count / 2).max(1)
    } else {
        stack.count
    }
}

/// What all strikes of one attack share.
#[derive(Debug, Clone, Copy)]
struct Blow {
    /// Hexes travelled right before the attack (jousting, BAT-DMG-005): 0 for retaliations,
    /// second attacks and shots.
    charge: u8,
    /// The attack is a death blow (BAT-DMG-007).
    death_blow: bool,
    /// The strike is lucky (BAT-LCK-002; only the strike on the target of the attack, Q18).
    lucky: bool,
    /// A commanded war machine's double damage (BAT-WAR-006).
    double_damage: bool,
    /// The share of its damage the striker deals as a retaliation after being attacked blind,
    /// percent (BAT-MAG-024; 100: full).
    retaliation_percent: u16,
}

/// Whether the creature of `stack` has an ability matching `f`.
fn has_ability(ruleset: &BattleRuleset, stack: &Stack, f: fn(&CompiledAbility) -> bool) -> bool {
    ruleset.abilities(stack.creature).iter().any(f)
}

/// One strike of `attacker` on `target`: base damage, factors, damage applied, event. Returns the
/// hit points the target lost (the damage, capped at what it had) and, for a melee strike on a
/// stack with Fire Shield, the damage it reflects (BAT-MAG-018): from the strike's first damage
/// stage capped at the target's hit points with quirk Q-0010, as the original, else from the hit
/// points lost.
fn strike<S: Sink>(
    state: &mut BattleState,
    ruleset: &BattleRuleset,
    (attacker, target): (StackId, StackId),
    flags: StrikeFlags,
    blow: Blow,
    sink: &mut S,
) -> (u64, u64) {
    let params = ruleset.battle();
    let striker = *state.at(attacker);
    let victim = *state.at(target);
    let count = striking_count(&striker);
    // rule: BAT-MAG-011
    let base = match striker
        .spells
        .damage
        .fixed(striker.damage_min, striker.damage_max)
    {
        Some(each) => u64::from(count).saturating_mul(each),
        None => base_damage(
            &mut state.rng,
            count,
            striker.damage_min,
            striker.damage_max,
            params.damage_roll_samples,
        ),
    };
    let terms = specialist_terms(state, ruleset, attacker);
    let (inputs, flags) = strike_inputs(ruleset, state, (&striker, &victim), (flags, terms), blow);
    let staged = staged(ruleset);
    let damage = inputs.damage(base, params, staged);
    let lost = damage.min(victim.total_hp());
    // rule: BAT-MAG-018, BAT-MAG-058
    let shielded = victim.spells.fire_shield > 0 || victim.has(StackTraits::INNATE_SPELL);
    let reflected = if shielded && !flags.contains(StrikeFlags::RANGED) {
        // Quirk Q-0010: the original reflects a share of the first damage stage; of a clone's, one
        // hit point (0x422440, BAT-MAG-038).
        let from = if victim.has(StackTraits::CLONE) {
            1
        } else if ruleset.quirks().is_enabled(FIRE_SHIELD_FIRST_STAGE) {
            inputs
                .stage_one(base, params, staged)
                .min(victim.total_hp())
        } else {
            lost
        };
        magic::reflection(state, ruleset, (&striker, target), from)
    } else {
        0
    };
    let kills = apply_damage(state.at_mut(target), damage);
    sink.emit(Event::public(EventKind::Struck {
        attacker,
        target,
        damage,
        kills,
        flags,
    }));
    effects::damaged(state, ruleset, target, sink);
    (lost, reflected)
}

/// Whether damage follows the executable's stages (quirk Q-0005).
fn staged(ruleset: &BattleRuleset) -> bool {
    ruleset.quirks().is_enabled(SINGLE_PRECISION_RATES)
}

/// The damage inputs of a strike of `striker` on `victim` (BAT-DMG-002, the abilities that
/// change them, the heroes' damage bonuses and a town's moat and walls), and its flags with the
/// luck and death-blow flags added.
fn strike_inputs(
    ruleset: &BattleRuleset,
    state: &BattleState,
    (striker, victim): (&Stack, &Stack),
    (flags, (slayer_bonus, blessing)): (StrikeFlags, SpecialistTerms),
    blow: Blow,
) -> (Strike, StrikeFlags) {
    let params = ruleset.battle();
    let field = Field::new(params.field);
    let ranged = flags.contains(StrikeFlags::RANGED);
    let staged = staged(ruleset);
    // rule: BAT-MAG-019
    let mut defense = if victim.spells.frenzy > 0 {
        0
    } else {
        effective_defense(victim, params)
    };
    let mut hate_percent = 0;
    let mut creature_penalty = 0;
    let mut jousting = (0, 0);
    for ability in ruleset.abilities(striker.creature) {
        match *ability {
            // rule: BAT-DMG-004
            Ability::Hate {
                target: hated,
                percent,
            } if hated == victim.creature => hate_percent = percent,
            // rule: BAT-DMG-005
            Ability::Jousting { percent_per_hex }
                if !has_ability(ruleset, victim, |a| matches!(a, Ability::ChargeImmunity)) =>
            {
                jousting = (percent_per_hex, u32::from(blow.charge));
            }
            // rule: BAT-DMG-006
            Ability::DefenseReduction { percent } => {
                defense = reduced_defense(defense, percent, staged);
            }
            // rule: BAT-DMG-010
            Ability::DamagePenalty { target, percent } if target == victim.creature => {
                creature_penalty = creature_penalty.max(percent);
            }
            Ability::DamagePenaltyByFlag { flag, percent }
                if ruleset
                    .creature(victim.creature)
                    .is_some_and(|c| c.flags.contains(flag)) =>
            {
                creature_penalty = creature_penalty.max(percent);
            }
            _ => {}
        }
    }
    // rule: BAT-SIE-006
    // The moat's penalty after the other reductions, not for a frenzied stack (0x442590).
    defense -= crate::siege::defense_penalty(state, victim) * i32::from(victim.spells.frenzy == 0);
    // rule: BAT-MAG-011
    let mut attack = striker.attack_now(ranged);
    // rule: BAT-MAG-014
    if slays(ruleset, striker.spells.slayer.1, victim) {
        // The specialist's bonus may fill the `i32` range (E04b T11b review): saturate.
        attack = attack
            .saturating_add(striker.spells.slayer.0.into())
            .saturating_add(slayer_bonus);
    }
    // rule: BAT-MAG-019
    if striker.spells.frenzy > 0 {
        attack = frenzied(
            attack,
            frenzy_defense(state, ruleset, striker, victim),
            striker.spells.frenzy,
            staged,
        );
    }
    let mut inputs = Strike::new(attack, defense);
    inputs.blessing = blessing;
    inputs.hate_percent = hate_percent;
    inputs.creature_penalty_percent = creature_penalty;
    inputs.jousting = jousting;
    let mut flags = flags;
    // rule: BAT-LCK-002
    if blow.lucky {
        inputs.lucky = true;
        flags = flags.with(StrikeFlags::LUCKY);
    }
    // rule: BAT-DMG-007
    if blow.death_blow {
        inputs.death_blow = true;
        flags = flags.with(StrikeFlags::DEATH_BLOW);
    }
    // rule: BAT-WAR-006
    if blow.double_damage {
        inputs.double_damage = true;
        flags = flags.with(StrikeFlags::DOUBLE_DAMAGE);
    }
    inputs.penalty_percent = penalty(params, field, striker, victim, flags).unwrap_or(0);
    // rule: BAT-SIE-008
    inputs.walls = ranged
        && crate::siege::wall_penalty(
            state,
            striker,
            victim,
            striker.has(StackTraits::NO_WALL_PENALTY),
        );
    // rule: BAT-MAG-024
    inputs.retaliation_percent = blow.retaliation_percent;
    // rule: BAT-MAG-012
    inputs.protection_percent = u16::from(if ranged {
        victim.spells.ranged_protection
    } else {
        victim.spells.melee_protection
    });
    // rule: BAT-MAG-043
    inputs.petrified = victim.spells.status.contains(Status::PETRIFIED);
    // rule: BAT-HER-004, BAT-MAG-026
    let heroes = &state.heroes;
    let own = hero::damage(heroes, striker.controller());
    inputs.hero_bonus = if ranged { own.ranged } else { own.melee };
    inputs.hero_reduction = hero::damage(heroes, victim.controller()).reduction;
    (inputs, flags)
}

/// What the striker's hero adds at a strike as a specialist in spells active on the striker
/// (BAT-MAG-020): to Slayer's attack bonus, and to the first damage stage of a blessed striker.
type SpecialistTerms = (i32, Option<Specialist>);

/// The [`SpecialistTerms`] of stack `s` (BAT-MAG-020; `installation.md` §7.12): the hero of its
/// controller (BAT-MAG-026) as a specialist in its Slayer, the bonus of Slayer's value by the
/// stack's level, and in its Bless.
// rule: BAT-MAG-020
fn specialist_terms(state: &BattleState, ruleset: &BattleRuleset, s: StackId) -> SpecialistTerms {
    let striker = state.at(s);
    let blessed = matches!(striker.spells.damage, DamageRoll::Maximum(_));
    let hero = state.heroes[striker.controller().index()]
        .as_ref()
        .filter(|h| (blessed || striker.spells.slayer.0 > 0) && h.magic.bonuses.has_specialty());
    let mut terms = (0, None);
    let Some(hero) = hero else {
        return terms;
    };
    let specialties = state.effects.of(s).iter().filter_map(|e| {
        ruleset.spell(e.spell).and_then(|spell| {
            let own = hero.magic.bonuses.spell(e.spell);
            Specialist::of(ruleset, spell, own, striker.creature).map(|sp| (spell, sp, e.value))
        })
    });
    for (spell, specialist, value) in specialties {
        for change in spell.magic.effects.iter().filter_map(|c| c.timed()) {
            match change {
                TimedChange::Slayer => {
                    let bonus = specialist.bonus(value.into());
                    terms.0 =
                        i32::try_from(bonus.clamp(i32::MIN.into(), i32::MAX.into())).unwrap_or(0);
                }
                TimedChange::MaximumDamage => terms.1 = Some(specialist),
                _ => {}
            }
        }
    }
    terms
}

/// The defense a frenzied `striker` adds to its attack (BAT-MAG-019): its defense with the Defend
/// bonus and, with quirk Q-0009 (S-16), as `victim` sees it: reduced by the share a Behemoth
/// ignores (BAT-DMG-006), as the original computes it (`installation.md` §7.11); then the moat's
/// penalty while the striker stands in it, not floored (BAT-SIE-006; the original's 0x442410 asks
/// 0x442590 for the striker's own defense, `installation.md` §7.28).
fn frenzy_defense(
    state: &BattleState,
    ruleset: &BattleRuleset,
    striker: &Stack,
    victim: &Stack,
) -> i32 {
    let mut defense = effective_defense(striker, ruleset.battle());
    if ruleset.quirks().is_enabled(FRENZY_REDUCED_DEFENSE) {
        for ability in ruleset.abilities(victim.creature) {
            if let Ability::DefenseReduction { percent } = *ability {
                defense = reduced_defense(defense, percent, staged(ruleset));
            }
        }
    }
    // rule: BAT-SIE-006
    defense - crate::siege::defense_penalty(state, striker)
}

/// Whether Slayer cast at `mastery` counts against `victim`'s creature (BAT-MAG-014): king 1 at
/// any mastery, king 2 from Advanced, king 3 at Expert (`installation.md` §7.10). Without Slayer
/// its bonus is 0, so the answer does not matter.
fn slays(ruleset: &BattleRuleset, mastery: u8, victim: &Stack) -> bool {
    ruleset.creature(victim.creature).is_some_and(|c| {
        c.flags.contains(CreatureFlag::King1)
            || (c.flags.contains(CreatureFlag::King2) && mastery >= 2)
            || (c.flags.contains(CreatureFlag::King3) && mastery >= 3)
    })
}

/// The damage a shot of `attacker` would deal to `target` now from the base damage `base`: every
/// stage of a real shot, without luck, death blows and double damage ([`shot_damage_range`]; the
/// computer's damage estimate, 0x442780 → 0x443F40 with its estimate flag, BAT-WAR-008, is
/// `aim::estimate_between`).
#[cfg(test)]
pub(crate) fn shot_damage(
    state: &BattleState,
    ruleset: &BattleRuleset,
    (attacker, target): (StackId, StackId),
    base: u64,
) -> u64 {
    estimated_damage(
        state,
        ruleset,
        (state.at(attacker), attacker, state.at(target)),
        (base, 0),
        StrikeFlags::RANGED,
    )
}

/// The damage `striker`, the record of stack `attacker` or a changed copy of it (the computer's
/// copy with a bonus, 0x437800; E23 T4b1), would deal to `target`, a stack's record or a changed
/// copy of it (with a defense bonus, 0x438910; E23 T4c), now from the base damage `base` after a
/// run-up of `charge` hexes, a shot with `flags` [`StrikeFlags::RANGED`]: every stage of a real
/// strike, without luck, death blows and double damage (0x443F40 with its estimate flag, the
/// stages 0x443E30 also runs).
pub(crate) fn estimated_damage(
    state: &BattleState,
    ruleset: &BattleRuleset,
    (striker, attacker, target): (&Stack, StackId, &Stack),
    (base, charge): (u64, u8),
    flags: StrikeFlags,
) -> u64 {
    let blow = Blow {
        charge,
        death_blow: false,
        lucky: false,
        double_damage: false,
        retaliation_percent: 100,
    };
    let terms = specialist_terms(state, ruleset, attacker);
    let (inputs, _) = strike_inputs(ruleset, state, (striker, target), (flags, terms), blow);
    inputs.damage(base, ruleset.battle(), staged(ruleset))
}

/// The least and greatest damage a shot of `attacker` would deal to `target` now, without luck,
/// death blows and double damage (the base damage of `count` creatures is between `count · min`
/// and `count · max` for every sampling, BAT-DMG-001; fixed while blessed or cursed,
/// BAT-MAG-011).
#[cfg(test)]
pub(crate) fn shot_damage_range(
    state: &BattleState,
    ruleset: &BattleRuleset,
    attacker: StackId,
    target: StackId,
) -> (u64, u64) {
    let striker = state.at(attacker);
    let n = u64::from(striking_count(striker));
    let (min, max) = (striker.damage_min, striker.damage_max);
    let (low, high) = striker
        .spells
        .damage
        .fixed(min, max)
        .map_or((u64::from(min), u64::from(max)), |each| (each, each));
    let pair = (attacker, target);
    (
        shot_damage(state, ruleset, pair, n.saturating_mul(low)),
        shot_damage(state, ruleset, pair, n.saturating_mul(high)),
    )
}

/// The engine's reading of an attack's first strike before it is made (BAT-DMG-011; AI-Q6,
/// ADR-0025 point 5): what the strike on its target would do at the least and at the greatest
/// base damage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DamagePreview {
    /// The least and the greatest damage (BAT-DMG-002; as [`EventKind::Struck`] reports it,
    /// before the target's hit points cap it).
    ///
    /// [`EventKind::Struck`]: crate::EventKind::Struck
    pub damage: [u64; 2],
    /// The least and the greatest number of creatures it kills (BAT-DMG-003).
    pub kills: [u32; 2],
}

/// The preview of `attacker`'s first strike on `target`, a shot when `flags` holds `RANGED`,
/// else melee after `charge` hexes (BAT-DMG-011): every stage of the real strike, with the base
/// damage at the ends of its range (BAT-DMG-001; fixed while blessed or cursed, BAT-MAG-011) and
/// none of the rolls (luck, death blow, a commanded war machine's double damage).
// rule: BAT-DMG-011
pub(crate) fn strike_preview(
    state: &BattleState,
    ruleset: &BattleRuleset,
    (attacker, target): (StackId, StackId),
    flags: StrikeFlags,
    charge: u8,
) -> DamagePreview {
    let (striker, victim) = (state.at(attacker), state.at(target));
    if striker.is_turret() {
        return turret_preview(state, ruleset, striker, victim);
    }
    let n = u64::from(striking_count(striker));
    let (min, max) = (striker.damage_min, striker.damage_max);
    let ends = striker
        .spells
        .damage
        .fixed(min, max)
        .map_or([u64::from(min), u64::from(max)], |each| [each, each]);
    let blow = Blow {
        charge,
        death_blow: false,
        lucky: false,
        double_damage: false,
        retaliation_percent: 100,
    };
    let terms = specialist_terms(state, ruleset, attacker);
    let (inputs, _) = strike_inputs(ruleset, state, (striker, victim), (flags, terms), blow);
    let (params, staged) = (ruleset.battle(), staged(ruleset));
    let damage = ends.map(|each| inputs.damage(n.saturating_mul(each), params, staged));
    let kills = damage.map(|d| {
        let mut copy = *victim;
        apply_damage(&mut copy, d)
    });
    DamagePreview { damage, kills }
}

/// The preview of a turret's shot at `victim` (BAT-DMG-011): its sum's ends, the creature's least
/// and greatest damage times the turret's count, through the target's protections alone
/// ([`turret_inputs`], BAT-SIE-018), as [`turret_shot`] draws it (the AI lane's E06 T6a).
fn turret_preview(
    state: &BattleState,
    ruleset: &BattleRuleset,
    turret: &Stack,
    victim: &Stack,
) -> DamagePreview {
    let n = u64::from(turret.count);
    let ends = ruleset.creature(turret.creature).map_or([0, 0], |c| {
        [u64::from(c.damage_min), u64::from(c.damage_max)]
    });
    let inputs = turret_inputs(state, victim);
    let (params, staged) = (ruleset.battle(), staged(ruleset));
    let damage = ends.map(|each| inputs.damage(n.saturating_mul(each), params, staged));
    let kills = damage.map(|d| {
        let mut copy = *victim;
        apply_damage(&mut copy, d)
    });
    DamagePreview { damage, kills }
}

/// A turret's shot at `victim` takes only the target's protections, the original's stage 3
/// (BAT-SIE-018; 0x443D90): Air Shield's share, the half while petrified and the Armorer of the
/// hero of its controller. No attack or defense, luck, Bless or Curse, hero's bonus, wall or
/// range penalty (`installation.md` §7.30).
fn turret_inputs(state: &BattleState, victim: &Stack) -> Strike {
    let mut inputs = Strike::new(0, 0);
    // rule: BAT-MAG-012
    inputs.protection_percent = u16::from(victim.spells.ranged_protection);
    // rule: BAT-MAG-043
    inputs.petrified = victim.spells.status.contains(Status::PETRIFIED);
    // rule: BAT-HER-004
    inputs.hero_reduction = hero::damage(&state.heroes, victim.controller()).reduction;
    inputs
}

/// A turret's shot at `target` (BAT-SIE-018; the original's 0x465AD0): the sum of its creature's
/// damage drawn once per creature, times the target's protections ([`turret_inputs`]), at least
/// 1; one shot, none used, not retaliated; then the rebirths and the end-of-battle check.
// rule: BAT-SIE-018
fn turret_shot<S: Sink>(
    state: &mut BattleState,
    ruleset: &BattleRuleset,
    turret: StackId,
    target: StackId,
    sink: &mut S,
) {
    let me = *state.at(turret);
    let (min, max) = ruleset.creature(me.creature).map_or((0, 0), |c| {
        (u32::from(c.damage_min), u32::from(c.damage_max))
    });
    let mut sum = 0u64;
    for _ in 0..me.count {
        sum = sum.saturating_add(u64::from(state.rng.range_inclusive(min, max)));
    }
    let inputs = turret_inputs(state, state.at(target));
    let damage = inputs.damage(sum, ruleset.battle(), staged(ruleset));
    let kills = apply_damage(state.at_mut(target), damage);
    sink.emit(Event::public(EventKind::Struck {
        attacker: turret,
        target,
        damage,
        kills,
        flags: StrikeFlags::RANGED,
    }));
    effects::damaged(state, ruleset, target, sink);
    // rule: BAT-MAG-059
    crate::raise::rebirths(state, ruleset, sink);
    check_end(state, sink);
}

/// The ranged or melee penalty of a strike, in percent, if any: shots at a target farther than
/// the full-damage distance (BAT-RNG-002), or melee strikes of shooters (BAT-RNG-003).
fn penalty(
    params: &BattleParams,
    field: Field,
    striker: &Stack,
    victim: &Stack,
    flags: StrikeFlags,
) -> Option<u16> {
    if flags.contains(StrikeFlags::RANGED) {
        // rule: BAT-RNG-002
        // Between the nearest hexes of both stacks: the original turns a shooter to face its
        // target before it measures from its front hex (0x440160, 0x446720, 0x467600).
        let near = field.footprint_distance(striker.footprint(field), victim.footprint(field))
            <= u32::from(params.ranged_full_damage_distance);
        (!near && !striker.has(StackTraits::NO_DISTANCE_PENALTY))
            .then(|| u16::from(params.ranged_penalty_percent))
    } else {
        // rule: BAT-RNG-003
        (striker.has(StackTraits::SHOOTER) && !striker.has(StackTraits::NO_MELEE_PENALTY))
            .then(|| u16::from(params.melee_penalty_percent))
    }
}

/// One attack of `attacker` on `target` that also hits the `others` (BAT-ATK-003, BAT-RNG-004):
/// the luck roll (BAT-LCK-002), the death-blow roll (BAT-DMG-007) and a commanded war machine's
/// double-damage roll (BAT-WAR-006, in this order as in VCMI), the strike on the target
/// (the only one luck applies to, Q18), the strikes on the others in stack-id order, for a melee
/// attack the spells and extra damage of the attacker's abilities on the target (BAT-MAG-040 to
/// BAT-MAG-047), then life drain (BAT-DMG-008) and death stare (BAT-DMG-009). Returns whether those spells stopped
/// the target's turns (it then does not retaliate).
fn attack<S: Sink>(
    state: &mut BattleState,
    ruleset: &BattleRuleset,
    pair: (StackId, StackId),
    others: &Targets,
    (flags, charge, retaliation_percent): (StrikeFlags, u8, u16),
    sink: &mut S,
) -> bool {
    let (attacker, target) = pair;
    let abilities = ruleset.abilities(state.at(attacker).creature);
    let (mut death_blow, mut drain, mut stare) = (0u8, 0u16, 0u8);
    for ability in abilities {
        match *ability {
            Ability::DeathBlow { chance_percent } => death_blow = chance_percent,
            Ability::LifeDrain { percent } => drain = percent,
            Ability::DeathStare { chance_percent } => stare = chance_percent,
            _ => {}
        }
    }
    // Luck is rolled before the death blow, as VCMI does.
    let lucky = morale::lucky(state, ruleset, attacker);
    // rule: BAT-DMG-007
    let death_blow = death_blow > 0 && state.rng.below(100) < u32::from(death_blow);
    // rule: BAT-WAR-006
    let chance = war::commanded_ballista(state, attacker).map_or(0, |b| b.double_damage_chance_bp);
    let double_damage = chance >= FULL_BP
        || (chance > 0 && state.rng.below(u32::from(FULL_BP)) < u32::from(chance));
    let blow = Blow {
        charge,
        death_blow,
        lucky: false,
        double_damage,
        retaliation_percent,
    };
    // Life drain takes `drain` % of the hit points lost by living targets, rounded down per
    // target.
    let drained = |state: &BattleState, id: StackId, lost: u64| {
        if state.at(id).has(StackTraits::ALIVE) {
            mul_div_floor(lost, u64::from(drain), 100).unwrap_or(u64::MAX)
        } else {
            0
        }
    };
    let (lost, mut reflected) = strike(state, ruleset, pair, flags, Blow { lucky, ..blow }, sink);
    let mut heal = drained(state, target, lost);
    let secondary = flags.with(StrikeFlags::SECONDARY);
    // A multi-headed striker takes back what every stack it strikes reflects, others only the
    // target's (BAT-MAG-018).
    let me = state.at(attacker);
    let every = me.has(StackTraits::ALL_ADJACENT) || me.has(StackTraits::THREE_HEADED);
    for (i, _) in others.iter().enumerate().filter(|(_, hit)| **hit) {
        // `i < STACK_CAPACITY <= u8::MAX`.
        let id = StackId(u8::try_from(i).unwrap_or(u8::MAX));
        if !state.at(id).is_alive() {
            // Its original's death took a clone (BAT-MAG-038).
            continue;
        }
        let (lost, r) = strike(state, ruleset, (attacker, id), secondary, blow, sink);
        heal = heal.saturating_add(drained(state, id, lost));
        if every {
            reflected = reflected.saturating_add(r);
        }
    }
    // The original casts them after the strike's damage, before the striker's own effects
    // (0x441610, `installation.md` §7.20); only melee strikes have them.
    // rule: BAT-MAG-040, BAT-MAG-041
    let stopped = !flags.contains(StrikeFlags::RANGED)
        && state.at(attacker).is_alive()
        && after_strike::cast(state, ruleset, pair, sink);
    // A clone that killed its own original is gone: nothing to drain into (BAT-MAG-038).
    let heal = heal * u64::from(state.at(attacker).is_alive());
    life_drain(state, attacker, heal, sink);
    if stare > 0 {
        death_stare(state, ruleset, pair, stare, sink);
    }
    // A clone that killed its own original is gone (BAT-MAG-038).
    if reflected > 0 && state.at(attacker).is_alive() {
        reflect(state, ruleset, attacker, reflected, sink);
    }
    stopped
}

/// Deals the `damage` its targets' Fire Shields reflect to `stack` (BAT-MAG-018).
// rule: BAT-MAG-018
fn reflect<S: Sink>(
    state: &mut BattleState,
    ruleset: &BattleRuleset,
    stack: StackId,
    damage: u64,
    sink: &mut S,
) {
    let kills = apply_damage(state.at_mut(stack), damage);
    sink.emit(Event::public(EventKind::Reflected {
        stack,
        damage,
        kills,
    }));
    effects::damaged(state, ruleset, stack, sink);
}

/// Heals and resurrects `stack` by `hp` hit points drained from its targets (BAT-DMG-008).
// rule: BAT-DMG-008
fn life_drain<S: Sink>(state: &mut BattleState, stack: StackId, hp: u64, sink: &mut S) {
    let (hit_points, resurrected) = resurrect(state.at_mut(stack), hp);
    if hit_points > 0 {
        sink.emit(Event::public(EventKind::LifeDrained {
            stack,
            hit_points,
            resurrected,
        }));
    }
}

/// Above this many creatures a death stare draws its kills at once instead of rolling for each
/// creature (BAT-DMG-009, OPEN-QUESTIONS Q39): an engine bound on the work of one attack, far
/// above any real army.
pub(crate) const STARE_ROLL_LIMIT: u32 = 65_536;

/// After its attack, each creature of `attacker` kills one creature of the living, surviving
/// `target` with `chance` % probability, at most `ceil(count · chance / 100)` in total (one trial
/// per creature, stopping at the cap or once the target is wiped out). Above
/// [`STARE_ROLL_LIMIT`] creatures the kills are drawn at once from the normal approximation of
/// the same binomial law (KER-RNG-003), then capped. The kills remove whole creatures, the
/// damaged top one first, so a surviving stack's new top creature has full health (Q21,
/// observed; VCMI keeps the damage instead) (BAT-DMG-009).
// rule: BAT-DMG-009
fn death_stare<S: Sink>(
    state: &mut BattleState,
    ruleset: &BattleRuleset,
    (attacker, target): (StackId, StackId),
    chance: u8,
    sink: &mut S,
) {
    let victim = state.at(target);
    if !victim.is_alive() || !victim.has(StackTraits::ALIVE) {
        return;
    }
    let count = state.at(attacker).count;
    // At most `count · chance / 100` (rounded up) kills, and no more than the target has.
    let cap = (u64::from(count) * u64::from(chance))
        .div_ceil(100)
        .min(u64::from(victim.count));
    let successes = if count > STARE_ROLL_LIMIT {
        u64::from(state.rng.binomial_approx(count, chance)).min(cap)
    } else {
        let mut successes: u64 = 0;
        for _ in 0..count {
            if successes == cap {
                break;
            }
            if state.rng.below(100) < u32::from(chance) {
                successes += 1;
            }
        }
        successes
    };
    if successes == 0 {
        return;
    }
    let victim = state.at_mut(target);
    // `successes` is at most the target's count.
    let kills = u32::try_from(successes).unwrap_or(victim.count);
    victim.count -= kills;
    victim.top_hp = if victim.is_alive() { victim.max_hp } else { 0 };
    sink.emit(Event::public(EventKind::DeathStared {
        attacker,
        target,
        kills,
    }));
    // rule: BAT-MAG-038
    crate::created::died(state, ruleset, target, sink);
}

/// The secondary targets of a melee attack of `attacker` on `target` (BAT-ATK-003), from the
/// current positions: every adjacent enemy for creatures that attack all adjacent enemies; for
/// three-headed attacks, enemies on the two hexes beside the attacked hex towards the attacker;
/// for breath attacks, any stack on the hex behind the attacked hex. Never the target or the
/// attacker itself; enemies as [`Stack::is_hostile_to`] has them (BAT-MAG-025/026: a berserk
/// Hydra strikes every adjacent stack, any Hydra a berserk one and a hypnotized stack of the
/// side its controller fights).
// rule: BAT-ATK-003
fn melee_secondaries(
    state: &BattleState,
    field: Field,
    attacker: StackId,
    target: StackId,
) -> Targets {
    let me = state.at(attacker);
    let all_adjacent = me.has(StackTraits::ALL_ADJACENT);
    if !all_adjacent && !me.has(StackTraits::THREE_HEADED) && !me.has(StackTraits::BREATH) {
        return NO_TARGETS;
    }
    let fp = me.footprint(field);
    let (enemy_hexes, any_hexes) = pattern_hexes(field, me, fp, state.at(target).footprint(field));
    core::array::from_fn(|i| {
        state.stacks.get(i).is_some_and(|s| {
            let enemy = me.is_hostile_to(s);
            let other = s.footprint(field);
            i != target.index()
                && i != attacker.index()
                && s.is_alive()
                && ((enemy && all_adjacent && field.touches(fp, other))
                    || other
                        .hexes()
                        .any(|h| any_hexes.contains(h) || (enemy && enemy_hexes.contains(h))))
        })
    })
}

/// The hexes hit by the three-headed (enemies only) and breath (any stack) patterns of `me`, at
/// footprint `fp`, attacking the stack at `victim`.
///
/// Ported from VCMI `CBattleInfoCallback::getPotentiallyAttackableHexes` (1.7.5): the pattern
/// starts at the attacker's front hex, or at its rear hex if the front does not touch the target
/// (VCMI's head and tail); the attack direction points at the target's rear hex if that is
/// adjacent, else at its front hex. Each
/// pattern is a walk of directions turned relative to the attack direction; a walk whose first
/// step would enter the attacker starts one step further, on the attacked hex. Hexes of the
/// attacker are never hit.
fn pattern_hexes(field: Field, me: &Stack, fp: Footprint, victim: Footprint) -> (HexSet, HexSet) {
    let mut enemy = HexSet::new();
    let mut any = HexSet::new();
    let front = Footprint {
        front: fp.front,
        rear: None,
    };
    // A footprint that touches the target with its rear hex only is two-hex.
    let origin = if field.touches(front, victim) {
        fp.front
    } else {
        fp.position()
    };
    // The origin touches the target, so one of the directions exists.
    let dir = victim
        .rear
        .and_then(|t| field.direction(origin, t))
        .or_else(|| field.direction(origin, victim.front))
        .unwrap_or(Direction::Right);
    let walk = |first: u8, more: &[u8]| {
        let start = if field
            .neighbor(origin, dir.turned(first))
            .is_some_and(|h| fp.covers(h))
        {
            field.neighbor(origin, dir)
        } else {
            Some(origin)
        };
        core::iter::once(first)
            .chain(more.iter().copied())
            .fold(start, |hex, turn| {
                hex.and_then(|h| field.neighbor(h, dir.turned(turn)))
            })
            .filter(|h| !fp.covers(*h))
    };
    if me.has(StackTraits::THREE_HEADED) {
        for hex in [walk(1, &[]), walk(5, &[])].into_iter().flatten() {
            enemy.insert(hex);
        }
    }
    if me.has(StackTraits::BREATH)
        && let Some(hex) = walk(0, &[0])
    {
        any.insert(hex);
    }
    (enemy, any)
}

/// A melee attack of `attacker`, which started its turn on `start` and travelled `charge` hexes,
/// on `target` (BAT-ATK-001): the attack, the target's retaliation (BAT-RET-001..003; reduced
/// or none if the target was blind, BAT-MAG-024; none if the attack's spells stopped its turns,
/// BAT-MAG-040), for creatures that strike twice a second attack unless the attacker now loses
/// its turns (BAT-ATK-002), the return to `start` of creatures that strike and return unless the
/// attacker now loses its turns or is bound (BAT-ATK-004, BAT-MAG-045; a move that enters the
/// hexes it lands on, BAT-MAG-028, BAT-MAG-033), the end of the attacker's Berserk
/// (BAT-MAG-025), the rebirths (BAT-MAG-059), then the end-of-battle check (BAT-END-001).
pub(crate) fn melee<S: Sink>(
    state: &mut BattleState,
    ruleset: &BattleRuleset,
    attacker: StackId,
    target: StackId,
    start: Hex,
    charge: u8,
    sink: &mut S,
) {
    let field = Field::new(ruleset.battle().field);
    let me = state.at(attacker);
    // rule: BAT-ATK-002
    let times = if me.has(StackTraits::STRIKES_TWICE) && !me.has(StackTraits::SHOOTER) {
        2
    } else {
        1
    };
    // Whether the target is blind as the attack begins (the original's mark, `installation.md`
    // §7.14): the strike ends the blindness, the retaliation keeps its factor.
    let blind = effects::retaliation_share(state, ruleset, target);
    for i in 0..times {
        let me = state.at(attacker);
        // A second strike needs an attacker that does not lose its turns (a retaliating
        // Unicorn may have blinded it, a Basilisk petrified it; the original's 0x441E91,
        // `installation.md` §7.20).
        let lost = i > 0 && me.spells.status.contains(Status::NO_TURNS);
        if !me.is_alive() || !state.at(target).is_alive() || lost {
            break;
        }
        let others = melee_secondaries(state, field, attacker, target);
        let charge = if i == 0 { charge } else { 0 };
        let pair = (attacker, target);
        let stopped = attack(
            state,
            ruleset,
            pair,
            &others,
            (StrikeFlags::NONE, charge, 100),
            sink,
        );
        // rule: BAT-MAG-040
        if i == 0 && !stopped {
            retaliate(state, ruleset, (target, attacker), blind, sink);
        }
    }
    // rule: BAT-ATK-004, BAT-MAG-045
    // The original's return (0x4789F8, `installation.md` §7.20) skips a stack that the
    // retaliation blinded, petrified or paralysed; a bound one stays too (survey S-14, fixed in
    // every profile: the original flies it back still bound, clears its binder lists on the move
    // and so never releases it).
    let me = state.at(attacker);
    if me.has(StackTraits::STRIKE_AND_RETURN)
        && me.is_alive()
        && me.hex != start
        && !me.spells.status.contains(Status::NO_TURNS)
        && !binds::is_bound(state, attacker)
    {
        // The return is a move like any other and enters the hexes it lands on (the original's
        // move handler, 0x445950: Fire Wall, then Land Mine); the battle's end waits for the
        // end of this action.
        let from = me.hex;
        crate::obstacles::travel_back(state, ruleset, attacker, (from, start), sink);
    }
    effects::attacked(state, ruleset, attacker, sink);
    // rule: BAT-MAG-059
    crate::raise::rebirths(state, ruleset, sink);
    check_end(state, sink);
}

/// The retaliation of `retaliator` on `attacker` (BAT-RET-001): if both live, the attacker's
/// strikes allow retaliation (BAT-RET-002), the retaliator is not petrified (BAT-MAG-043) and
/// has a retaliation left this round (BAT-RET-003), it attacks back with its full attack
/// pattern. A retaliator that was blind or paralysed when attacked (`blind`: Blind's or
/// Paralyze's share) deals that share of its damage, and none at 0, keeping its retaliation
/// (BAT-MAG-024, BAT-MAG-043).
// rule: BAT-RET-001, BAT-RET-002, BAT-RET-003, BAT-MAG-024, BAT-MAG-043
fn retaliate<S: Sink>(
    state: &mut BattleState,
    ruleset: &BattleRuleset,
    (retaliator, attacker): (StackId, StackId),
    blind: Option<u16>,
    sink: &mut S,
) {
    let r = state.at(retaliator);
    let a = state.at(attacker);
    // War machines never retaliate (BAT-WAR-001), nor do petrified stacks (BAT-MAG-043).
    if !r.is_alive()
        || !a.is_alive()
        || a.has(StackTraits::NO_ENEMY_RETALIATION)
        || r.has(StackTraits::WAR_MACHINE)
        || r.spells.status.contains(Status::PETRIFIED)
        || r.retaliations_left == 0
        || blind == Some(0)
    {
        return;
    }
    let r = state.at_mut(retaliator);
    if r.retaliations != UNLIMITED_RETALIATIONS {
        r.retaliations_left -= 1;
        r.flags
            .set(StackFlags::RETALIATED, r.retaliations_left == 0);
    }
    let field = Field::new(ruleset.battle().field);
    let others = melee_secondaries(state, field, retaliator, attacker);
    let pair = (retaliator, attacker);
    attack(
        state,
        ruleset,
        pair,
        &others,
        (StrikeFlags::RETALIATION, 0, blind.unwrap_or(100)),
        sink,
    );
}

/// A shot at `target`, aimed at its hex `at`: one ranged attack per shot (two for shooters that
/// strike twice, the commanded Ballista's `shots`, BAT-WAR-006; further shots as long as the
/// target lives and shots are left), each using a shot (none below 0, quirk Q-0004; the Ammo
/// Cart of the shooter's own side supplies them, also while the other side controls the shooter:
/// the original asks the hero of the shooter's own side, 0x43F9E1); area shots also strike every
/// other living stack around `at` that the area does not spare (BAT-RNG-004). Shots are not
/// retaliated. Then the end of the shooter's Berserk (BAT-MAG-025), the rebirths (BAT-MAG-059)
/// and the end-of-battle check.
// rule: BAT-RNG-001
pub(crate) fn shoot<S: Sink>(
    state: &mut BattleState,
    ruleset: &BattleRuleset,
    shooter: StackId,
    target: StackId,
    at: Hex,
    sink: &mut S,
) {
    // rule: BAT-SIE-018
    if state.at(shooter).is_turret() {
        turret_shot(state, ruleset, shooter, target, sink);
        return;
    }
    let field = Field::new(ruleset.battle().field);
    let times = match war::commanded_ballista(state, shooter) {
        Some(control) => control.shots,
        None if state.at(shooter).has(StackTraits::STRIKES_TWICE) => 2,
        None => 1,
    };
    for i in 0..times {
        if i > 0 && (!state.at(target).is_alive() || !has_ammunition(state.at(shooter))) {
            break;
        }
        if !ammo_supplied(state, state.at(shooter).side) {
            let s = state.at_mut(shooter);
            s.shots = s.shots.saturating_sub(1);
        }
        let others = area(state, field, shooter, target, at);
        let pair = (shooter, target);
        let how = (StrikeFlags::RANGED, 0, 100);
        attack(state, ruleset, pair, &others, how, sink);
    }
    effects::attacked(state, ruleset, shooter, sink);
    // rule: BAT-MAG-059
    crate::raise::rebirths(state, ruleset, sink);
    check_end(state, sink);
}

/// The secondary targets of a shot: for area shots every living stack other than the primary
/// target with a hex in `at` or next to it, unless the area spares it — a fireball spares
/// fire-immune stacks, a death cloud stacks that are not alive (BAT-RNG-004).
// rule: BAT-RNG-004
fn area(state: &BattleState, field: Field, shooter: StackId, target: StackId, at: Hex) -> Targets {
    let me = state.at(shooter);
    if !me.has(StackTraits::AREA_SHOT) {
        return NO_TARGETS;
    }
    core::array::from_fn(|i| {
        state.stacks.get(i).is_some_and(|s| {
            s.is_alive()
                && i != target.index()
                && !spares(me, s)
                && s.footprint(field)
                    .hexes()
                    .any(|h| field.distance(h, at) <= 1)
        })
    })
}

/// Whether the area shot of `shooter` spares stack `s` (BAT-RNG-004): a fireball the
/// fire-immune stacks, a death cloud the stacks that are not alive. The computer's estimate of
/// the shot asks it too (`aim::area_value` without BAT-QUIRK-25).
// rule: BAT-RNG-004
pub(crate) fn spares(shooter: &Stack, s: &Stack) -> bool {
    if shooter.has(StackTraits::DEATH_CLOUD) {
        !s.has(StackTraits::ALIVE)
    } else {
        s.has(StackTraits::FIRE_IMMUNE)
    }
}

/// The battle ends when a side has no living stacks other than war machines (BAT-WAR-001): the
/// other side wins (BAT-END-001), or nobody if neither side has any left (BAT-END-002) or the
/// other side's are all created in the battle (both lose, BAT-MAG-037; the original's 0x4658B0).
// rule: BAT-END-001, BAT-END-002
pub(crate) fn check_end<S: Sink>(state: &mut BattleState, sink: &mut S) {
    let alive = |side: BattleSide| {
        state
            .stacks
            .iter()
            .any(|s: &Stack| s.side == side && s.is_alive() && !s.has(StackTraits::WAR_MACHINE))
    };
    // rule: BAT-MAG-037
    let wins = |side: BattleSide| {
        let army = |s: &Stack| {
            s.side == side && s.is_alive() && !s.is_created() && !s.has(StackTraits::WAR_MACHINE)
        };
        if state.stacks.iter().any(army) {
            BattleResult::Victory(side)
        } else {
            BattleResult::Draw
        }
    };
    let result = match (alive(BattleSide::Attacker), alive(BattleSide::Defender)) {
        (true, true) => None,
        (true, false) => Some(wins(BattleSide::Attacker)),
        (false, true) => Some(wins(BattleSide::Defender)),
        (false, false) => Some(BattleResult::Draw),
    };
    if let Some(r) = result {
        state.result = Some(r);
        sink.emit(Event::public(EventKind::BattleEnded { result: r }));
    }
}
