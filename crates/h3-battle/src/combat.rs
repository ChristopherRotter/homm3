//! Attacks and their sequences: melee attacks with their abilities (BAT-ATK-001..004,
//! BAT-RET-001..003, BAT-DMG-004..009), shots (BAT-RNG-001..004) and the end of the battle
//! (BAT-END-001).
//!
//! An *attack* is one swing of a stack: one death-blow roll, a strike on the target, strikes on
//! the secondary targets its attack pattern hits, then life drain and death stare. A melee attack
//! (BAT-ATK-001) is the attack, the target's retaliation, and for creatures that strike twice a
//! second attack.

use h3_content::{
    Ability, BattleParams, CompiledAbility, CreatureFlag, FIRE_SHIELD_FIRST_STAGE,
    FRENZY_REDUCED_DEFENSE, FULL_BP, Ruleset, SINGLE_PRECISION_RATES, TimedChange,
};
use h3_contracts::{BattleResult, BattleSide};
use h3_kernel::arith::mul_div_floor;

use crate::damage::{
    Strike, apply_damage, base_damage, effective_defense, frenzied, reduced_defense, resurrect,
};
use crate::effects::{self, DamageRoll, Status};
use crate::event::{Event, EventKind, Sink, StrikeFlags};
use crate::geometry::{Direction, Field, Footprint, Hex, HexSet};
use crate::hero::{self, HeroRecord};
use crate::magic::Specialist;
use crate::state::{
    BattleState, STACK_CAPACITY, Stack, StackFlags, StackId, StackTraits, UNLIMITED_RETALIATIONS,
};
use crate::{magic, morale, war};

/// The secondary targets of an attack, by stack index.
type Targets = [bool; STACK_CAPACITY];

/// No secondary targets.
const NO_TARGETS: Targets = [false; STACK_CAPACITY];

/// Whether stack `id` is next to a living enemy (its footprint touches one; enemies as
/// BAT-MAG-025/026 have them), which keeps a shooter from shooting (BAT-RNG-001).
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
fn has_ability(ruleset: &Ruleset, stack: &Stack, f: fn(&CompiledAbility) -> bool) -> bool {
    ruleset.abilities(stack.creature).iter().any(f)
}

/// One strike of `attacker` on `target`: base damage, factors, damage applied, event. Returns the
/// hit points the target lost (the damage, capped at what it had) and, for a melee strike on a
/// stack with Fire Shield, the damage it reflects (BAT-MAG-018): from the strike's first damage
/// stage capped at the target's hit points with quirk Q-0010, as the original, else from the hit
/// points lost.
fn strike<S: Sink>(
    state: &mut BattleState,
    ruleset: &Ruleset,
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
    let (inputs, flags) = strike_inputs(
        ruleset,
        &state.heroes,
        (&striker, &victim),
        (flags, terms),
        blow,
    );
    let staged = staged(ruleset);
    let damage = inputs.damage(base, params, staged);
    let lost = damage.min(victim.total_hp());
    // rule: BAT-MAG-018
    let reflected = if victim.spells.fire_shield > 0 && !flags.contains(StrikeFlags::RANGED) {
        // Quirk Q-0010: the original reflects a share of the first damage stage.
        let from = if ruleset.quirks().is_enabled(FIRE_SHIELD_FIRST_STAGE) {
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
fn staged(ruleset: &Ruleset) -> bool {
    ruleset.quirks().is_enabled(SINGLE_PRECISION_RATES)
}

/// The damage inputs of a strike of `striker` on `victim` (BAT-DMG-002, the abilities that
/// change them and the heroes' damage bonuses), and its flags with the luck and death-blow flags
/// added.
fn strike_inputs(
    ruleset: &Ruleset,
    heroes: &[Option<HeroRecord>; 2],
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
            _ => {}
        }
    }
    // rule: BAT-MAG-011
    let mut attack = striker.attack_now(ranged);
    // rule: BAT-MAG-014
    if slays(ruleset, striker.spells.slayer.1, victim) {
        attack += i32::from(striker.spells.slayer.0) + slayer_bonus;
    }
    // rule: BAT-MAG-019
    if striker.spells.frenzy > 0 {
        attack = frenzied(
            attack,
            frenzy_defense(ruleset, striker, victim),
            striker.spells.frenzy,
            staged,
        );
    }
    let mut inputs = Strike::new(attack, defense);
    inputs.blessing = blessing;
    inputs.hate_percent = hate_percent;
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
    // rule: BAT-MAG-024
    inputs.retaliation_percent = blow.retaliation_percent;
    // rule: BAT-MAG-012
    inputs.protection_percent = u16::from(if ranged {
        victim.spells.ranged_protection
    } else {
        victim.spells.melee_protection
    });
    // rule: BAT-HER-004, BAT-MAG-026
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
fn specialist_terms(state: &BattleState, ruleset: &Ruleset, s: StackId) -> SpecialistTerms {
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
/// ignores (BAT-DMG-006), as the original computes it (`installation.md` §7.11).
fn frenzy_defense(ruleset: &Ruleset, striker: &Stack, victim: &Stack) -> i32 {
    let mut defense = effective_defense(striker, ruleset.battle());
    if ruleset.quirks().is_enabled(FRENZY_REDUCED_DEFENSE) {
        for ability in ruleset.abilities(victim.creature) {
            if let Ability::DefenseReduction { percent } = *ability {
                defense = reduced_defense(defense, percent, staged(ruleset));
            }
        }
    }
    defense
}

/// Whether Slayer cast at `mastery` counts against `victim`'s creature (BAT-MAG-014): king 1 at
/// any mastery, king 2 from Advanced, king 3 at Expert (`installation.md` §7.10). Without Slayer
/// its bonus is 0, so the answer does not matter.
fn slays(ruleset: &Ruleset, mastery: u8, victim: &Stack) -> bool {
    ruleset.creature(victim.creature).is_some_and(|c| {
        c.flags.contains(CreatureFlag::King1)
            || (c.flags.contains(CreatureFlag::King2) && mastery >= 2)
            || (c.flags.contains(CreatureFlag::King3) && mastery >= 3)
    })
}

/// The least and greatest damage a shot of `attacker` would deal to `target` now, without luck,
/// death blows and double damage (the base damage of `count` creatures is between `count · min`
/// and `count · max` for every sampling, BAT-DMG-001; fixed while blessed or cursed,
/// BAT-MAG-011).
pub(crate) fn shot_damage_range(
    state: &BattleState,
    ruleset: &Ruleset,
    attacker: StackId,
    target: StackId,
) -> (u64, u64) {
    let striker = state.at(attacker);
    let blow = Blow {
        charge: 0,
        death_blow: false,
        lucky: false,
        double_damage: false,
        retaliation_percent: 100,
    };
    let terms = specialist_terms(state, ruleset, attacker);
    let (inputs, _) = strike_inputs(
        ruleset,
        &state.heroes,
        (striker, state.at(target)),
        (StrikeFlags::RANGED, terms),
        blow,
    );
    let (params, staged) = (ruleset.battle(), staged(ruleset));
    let n = u64::from(striking_count(striker));
    let (min, max) = (striker.damage_min, striker.damage_max);
    let (low, high) = striker
        .spells
        .damage
        .fixed(min, max)
        .map_or((u64::from(min), u64::from(max)), |each| (each, each));
    (
        inputs.damage(n.saturating_mul(low), params, staged),
        inputs.damage(n.saturating_mul(high), params, staged),
    )
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
        let near = victim.footprint(field).hexes().any(|h| {
            field.distance(striker.hex, h) <= u32::from(params.ranged_full_damage_distance)
        });
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
/// (the only one luck applies to, Q18), the strikes on the others in stack-id order, then life
/// drain (BAT-DMG-008) and death stare (BAT-DMG-009).
fn attack<S: Sink>(
    state: &mut BattleState,
    ruleset: &Ruleset,
    pair: (StackId, StackId),
    others: &Targets,
    (flags, charge, retaliation_percent): (StrikeFlags, u8, u16),
    sink: &mut S,
) {
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
        let (lost, r) = strike(state, ruleset, (attacker, id), secondary, blow, sink);
        heal = heal.saturating_add(drained(state, id, lost));
        if every {
            reflected = reflected.saturating_add(r);
        }
    }
    life_drain(state, attacker, heal, sink);
    if stare > 0 {
        death_stare(state, pair, stare, sink);
    }
    if reflected > 0 {
        reflect(state, ruleset, attacker, reflected, sink);
    }
}

/// Deals the `damage` its targets' Fire Shields reflect to `stack` (BAT-MAG-018).
// rule: BAT-MAG-018
fn reflect<S: Sink>(
    state: &mut BattleState,
    ruleset: &Ruleset,
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
}

/// The secondary targets of a melee attack of `attacker` on `target` (BAT-ATK-003), from the
/// current positions: every adjacent enemy for creatures that attack all adjacent enemies; for
/// three-headed attacks, enemies on the two hexes beside the attacked hex towards the attacker;
/// for breath attacks, any stack on the hex behind the attacked hex. Never the target or the
/// attacker itself; enemies as BAT-MAG-025/026 have them (a berserk Hydra strikes every
/// adjacent stack).
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
/// starts at the attacker's head, or at its tail if the head does not touch the target; the
/// attack direction points at the target's tail if that is adjacent, else at its head. Each
/// pattern is a walk of directions turned relative to the attack direction; a walk whose first
/// step would enter the attacker starts one step further, on the attacked hex. Hexes of the
/// attacker are never hit.
fn pattern_hexes(field: Field, me: &Stack, fp: Footprint, victim: Footprint) -> (HexSet, HexSet) {
    let mut enemy = HexSet::new();
    let mut any = HexSet::new();
    let head = Footprint {
        head: fp.head,
        tail: None,
    };
    // A footprint that touches the target with its tail only is two-hex.
    let origin = if field.touches(head, victim) {
        fp.head
    } else {
        fp.tail.unwrap_or(fp.head)
    };
    // The origin touches the target, so one of the directions exists.
    let dir = victim
        .tail
        .and_then(|t| field.direction(origin, t))
        .or_else(|| field.direction(origin, victim.head))
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
/// or none if the target was blind, BAT-MAG-024), for creatures that strike twice a second attack
/// (BAT-ATK-002), the return to `start` of creatures that strike and return (BAT-ATK-004), the
/// end of the attacker's Berserk (BAT-MAG-025), then the end-of-battle check (BAT-END-001).
pub(crate) fn melee<S: Sink>(
    state: &mut BattleState,
    ruleset: &Ruleset,
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
    let blind = effects::blind_share(state, ruleset, target);
    for i in 0..times {
        if !state.at(attacker).is_alive() || !state.at(target).is_alive() {
            break;
        }
        let others = melee_secondaries(state, field, attacker, target);
        let charge = if i == 0 { charge } else { 0 };
        let pair = (attacker, target);
        attack(
            state,
            ruleset,
            pair,
            &others,
            (StrikeFlags::NONE, charge, 100),
            sink,
        );
        if i == 0 {
            retaliate(state, ruleset, (target, attacker), blind, sink);
        }
    }
    // rule: BAT-ATK-004
    let me = state.at(attacker);
    if me.has(StackTraits::STRIKE_AND_RETURN) && me.is_alive() && me.hex != start {
        let from = me.hex;
        state.at_mut(attacker).hex = start;
        sink.emit(Event::public(EventKind::Moved {
            stack: attacker,
            from,
            to: start,
        }));
    }
    effects::attacked(state, ruleset, attacker, sink);
    check_end(state, sink);
}

/// The retaliation of `retaliator` on `attacker` (BAT-RET-001): if both live, the attacker's
/// strikes allow retaliation (BAT-RET-002) and the retaliator has a retaliation left this round
/// (BAT-RET-003), it attacks back with its full attack pattern. A retaliator that was blind when
/// attacked (`blind`: Blind's share) deals that share of its damage, and none at 0, keeping its
/// retaliation (BAT-MAG-024).
// rule: BAT-RET-001, BAT-RET-002, BAT-RET-003, BAT-MAG-024
fn retaliate<S: Sink>(
    state: &mut BattleState,
    ruleset: &Ruleset,
    (retaliator, attacker): (StackId, StackId),
    blind: Option<u16>,
    sink: &mut S,
) {
    let r = state.at(retaliator);
    let a = state.at(attacker);
    // War machines never retaliate (BAT-WAR-001).
    if !r.is_alive()
        || !a.is_alive()
        || a.has(StackTraits::NO_ENEMY_RETALIATION)
        || r.has(StackTraits::WAR_MACHINE)
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
/// target lives and shots are left), each using a shot (none below 0, quirk Q-0004; the
/// controller's Ammo Cart supplies them); area shots also strike every other living stack around
/// `at` that the area does not spare (BAT-RNG-004). Shots are not retaliated. Then the end of the
/// shooter's Berserk (BAT-MAG-025) and the end-of-battle check.
// rule: BAT-RNG-001
pub(crate) fn shoot<S: Sink>(
    state: &mut BattleState,
    ruleset: &Ruleset,
    shooter: StackId,
    target: StackId,
    at: Hex,
    sink: &mut S,
) {
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
        if !ammo_supplied(state, state.at(shooter).controller()) {
            let s = state.at_mut(shooter);
            s.shots = s.shots.saturating_sub(1);
        }
        let others = area(state, field, shooter, target, at);
        let pair = (shooter, target);
        let how = (StrikeFlags::RANGED, 0, 100);
        attack(state, ruleset, pair, &others, how, sink);
    }
    effects::attacked(state, ruleset, shooter, sink);
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
    let spared: fn(&Stack) -> bool = if me.has(StackTraits::DEATH_CLOUD) {
        |s| !s.has(StackTraits::ALIVE)
    } else {
        |s| s.has(StackTraits::FIRE_IMMUNE)
    };
    core::array::from_fn(|i| {
        state.stacks.get(i).is_some_and(|s| {
            s.is_alive()
                && i != target.index()
                && !spared(s)
                && s.footprint(field)
                    .hexes()
                    .any(|h| field.distance(h, at) <= 1)
        })
    })
}

/// The battle ends when a side has no living stacks other than war machines (BAT-WAR-001): the
/// other side wins (BAT-END-001), or nobody if neither side has any left (BAT-END-002).
// rule: BAT-END-001, BAT-END-002
pub(crate) fn check_end<S: Sink>(state: &mut BattleState, sink: &mut S) {
    let alive = |side: BattleSide| {
        state
            .stacks
            .iter()
            .any(|s: &Stack| s.side == side && s.is_alive() && !s.has(StackTraits::WAR_MACHINE))
    };
    let result = match (alive(BattleSide::Attacker), alive(BattleSide::Defender)) {
        (true, true) => None,
        (true, false) => Some(BattleResult::Victory(BattleSide::Attacker)),
        (false, true) => Some(BattleResult::Victory(BattleSide::Defender)),
        (false, false) => Some(BattleResult::Draw),
    };
    if let Some(r) = result {
        state.result = Some(r);
        sink.emit(Event::public(EventKind::BattleEnded { result: r }));
    }
}
