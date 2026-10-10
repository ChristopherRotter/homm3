//! The engine's enums to the wire. The matches are exhaustive, with no wildcard arm (ADR-0018
//! point 5, amended 2026-10-10: the user chose a build error over an untested fallback): a new
//! variant of the engine's commands, events, errors, targets, damage rolls or obstacle kinds
//! fails this crate's build until its wire variant (`wire.rs`) and its arm here exist.

use h3_battle::{CastTarget, CommandError, DamageRoll as EngineRoll, EventKind};
use h3_content::ObstacleKind as EngineObstacle;

use super::{hexes, outcome, side, strike, wide};
use crate::wire::{Command, DamageRoll, Event, ObstacleKind, RejectCode, Target};

/// An engine command on the wire (the legal list, the echo of an applied command).
pub fn command(command: h3_battle::Command) -> Command {
    use h3_battle::Command as C;
    match command {
        C::Move { to } => Command::Move { to: to.0 },
        C::Attack { target, from } => Command::Attack {
            target: target.0,
            from: from.0,
        },
        C::Shoot { target, at } => Command::Shoot {
            target: target.0,
            at: at.0,
        },
        C::Wait => Command::Wait {},
        C::Defend => Command::Defend {},
        C::Heal { target } => Command::Heal { target: target.0 },
        C::EndTactics => Command::EndTactics {},
        C::Retreat => Command::Retreat {},
        C::Surrender => Command::Surrender {},
        C::Cast {
            spell,
            target: t,
            second,
        } => Command::Cast {
            spell: spell.raw(),
            target: target(t),
            second: target(second),
        },
        C::CreatureCast { target: t } => Command::CreatureCast { target: target(t) },
    }
}

/// A cast's target on the wire.
pub fn target(target: CastTarget) -> Target {
    match target {
        CastTarget::None => Target::None {},
        CastTarget::Stack(stack) => Target::Stack { stack: stack.0 },
        CastTarget::Hex(hex) => Target::Hex { hex: hex.0 },
    }
}

/// The engine's reason for a rejection as the wire's code.
pub fn reject(error: CommandError) -> RejectCode {
    use CommandError as E;
    match error {
        E::BattleOver => RejectCode::BattleOver,
        E::NotYourTurn => RejectCode::NotYourTurn,
        E::AlreadyWaited => RejectCode::AlreadyWaited,
        E::InvalidHex => RejectCode::InvalidHex,
        E::Unreachable => RejectCode::Unreachable,
        E::NoSuchTarget => RejectCode::NoSuchTarget,
        E::NotAnEnemy => RejectCode::NotAnEnemy,
        E::NotAdjacent => RejectCode::NotAdjacent,
        E::MustShoot => RejectCode::MustShoot,
        E::CannotShoot => RejectCode::CannotShoot,
        E::NotOnTarget => RejectCode::NotOnTarget,
        E::NoMelee => RejectCode::NoMelee,
        E::CannotHeal => RejectCode::CannotHeal,
        E::NotAPatient => RejectCode::NotAPatient,
        E::InTacticsPhase => RejectCode::InTacticsPhase,
        E::NoTacticsPhase => RejectCode::NoTacticsPhase,
        E::OutsideZone => RejectCode::OutsideZone,
        E::CannotRetreat => RejectCode::CannotRetreat,
        E::CannotSurrender => RejectCode::CannotSurrender,
        E::CannotAffordSurrender => RejectCode::CannotAffordSurrender,
        E::NotInSpellBook => RejectCode::NotInSpellBook,
        E::NotACombatSpell => RejectCode::NotACombatSpell,
        E::AlreadyCast => RejectCode::AlreadyCast,
        E::NotCastable => RejectCode::NotCastable,
        E::NotEnoughMana => RejectCode::NotEnoughMana,
        E::InvalidSpellTarget => RejectCode::InvalidSpellTarget,
        E::CannotCast => RejectCode::CannotCast,
        E::CastingForbidden => RejectCode::CastingForbidden,
        E::SpellLevelBarred => RejectCode::SpellLevelBarred,
    }
}

/// How a stack's damage rolls, on the wire.
pub fn damage_roll(roll: EngineRoll) -> DamageRoll {
    match roll {
        EngineRoll::Random => DamageRoll::Random {},
        EngineRoll::Maximum(value) => DamageRoll::Maximum { value },
        EngineRoll::Minimum(value) => DamageRoll::Minimum { value },
    }
}

/// A spell obstacle's kind on the wire.
pub fn obstacle_kind(kind: EngineObstacle) -> ObstacleKind {
    match kind {
        EngineObstacle::Wall => ObstacleKind::Wall {},
        EngineObstacle::Fire => ObstacleKind::Fire {},
        EngineObstacle::Pit => ObstacleKind::Pit {},
        EngineObstacle::Mine => ObstacleKind::Mine {},
    }
}

/// An event's kind on the wire. The caller has projected the event for its viewer.
pub fn event(kind: &EventKind) -> Event {
    use EventKind as K;
    match *kind {
        K::TacticsStarted { side: s, columns } => Event::TacticsStarted {
            side: side(s),
            columns,
        },
        K::TacticsEnded => Event::TacticsEnded {},
        K::RoundStarted { round } => Event::RoundStarted { round },
        K::TurnStarted { stack } => Event::TurnStarted { stack: stack.0 },
        K::Moved { stack, from, to } => Event::Moved {
            stack: stack.0,
            from: from.0,
            to: to.0,
        },
        K::Waited { stack } => Event::Waited { stack: stack.0 },
        K::Defended { stack } => Event::Defended { stack: stack.0 },
        K::Struck {
            attacker,
            target,
            damage,
            kills,
            flags,
        } => Event::Struck {
            attacker: attacker.0,
            target: target.0,
            damage: wide(damage),
            kills,
            flags: strike(flags),
        },
        K::Reflected {
            stack,
            damage,
            kills,
        } => Event::Reflected {
            stack: stack.0,
            damage: wide(damage),
            kills,
        },
        K::LifeDrained {
            stack,
            hit_points,
            resurrected,
        } => Event::LifeDrained {
            stack: stack.0,
            hit_points: wide(hit_points),
            resurrected,
        },
        K::DeathStared {
            attacker,
            target,
            kills,
        } => Event::DeathStared {
            attacker: attacker.0,
            target: target.0,
            kills,
        },
        K::Regenerated { stack, hit_points } => Event::Regenerated {
            stack: stack.0,
            hit_points,
        },
        K::Healed {
            healer,
            target,
            hit_points,
        } => Event::Healed {
            healer: healer.0,
            target: target.0,
            hit_points,
        },
        K::GoodMorale { stack } => Event::GoodMorale { stack: stack.0 },
        K::BadMorale { stack } => Event::BadMorale { stack: stack.0 },
        K::Frightened { stack } => Event::Frightened { stack: stack.0 },
        K::TurnPassed { stack } => Event::TurnPassed { stack: stack.0 },
        K::SpellCast {
            side: s,
            spell,
            target: t,
            second,
        } => Event::SpellCast {
            side: side(s),
            spell: spell.raw(),
            target: target(t),
            second: target(second),
        },
        K::StackCast {
            stack,
            spell,
            target: t,
        } => Event::StackCast {
            stack: stack.0,
            spell: spell.raw(),
            target: target(t),
        },
        K::PeriodicCast { stack, spell } => Event::PeriodicCast {
            stack: stack.0,
            spell: spell.raw(),
        },
        K::ManaChanneled { side: s, mana } => Event::ManaChanneled {
            side: side(s),
            mana,
        },
        K::ManaDrained {
            stack,
            side: s,
            mana,
        } => Event::ManaDrained {
            stack: stack.0,
            side: side(s),
            mana,
        },
        K::CreatureRaise { stack, target } => Event::CreatureRaise {
            stack: stack.0,
            target: target.0,
        },
        K::RoseFromCorpse {
            stack,
            corpse,
            creature,
            count,
            hex,
        } => Event::RoseFromCorpse {
            stack: stack.0,
            corpse: corpse.0,
            creature: creature.raw(),
            count,
            hex: hex.0,
        },
        K::SpellDrawn { stack, spell } => Event::SpellDrawn {
            stack: stack.0,
            spell: spell.raw(),
        },
        K::SpellResisted { target } => Event::SpellResisted { target: target.0 },
        K::SpellRedirected { from, to } => Event::SpellRedirected {
            from: from.0,
            to: to.0,
        },
        K::StrikeSpell {
            striker,
            target,
            spell,
        } => Event::StrikeSpell {
            striker: striker.0,
            target: target.0,
            spell: spell.raw(),
        },
        K::StrikeDamage {
            striker,
            target,
            damage,
            kills,
        } => Event::StrikeDamage {
            striker: striker.0,
            target: target.0,
            damage: wide(damage),
            kills,
        },
        K::Poisoned { stack, hit_points } => Event::Poisoned {
            stack: stack.0,
            hit_points,
        },
        K::SpellDamage {
            target,
            value,
            caster,
            damage,
            kills,
        } => Event::SpellDamage {
            target: target.0,
            value: wide(value),
            caster: wide(caster),
            damage: wide(damage),
            kills,
        },
        K::SpellHealed { target, hit_points } => Event::SpellHealed {
            target: target.0,
            hit_points,
        },
        K::EffectApplied {
            target,
            spell,
            rounds,
        } => Event::EffectApplied {
            target: target.0,
            spell: spell.raw(),
            rounds,
        },
        K::EffectEnded { target, spell } => Event::EffectEnded {
            target: target.0,
            spell: spell.raw(),
        },
        K::ObstaclePlaced {
            side: s,
            spell,
            hexes: h,
        } => Event::ObstaclePlaced {
            side: side(s),
            spell: spell.raw(),
            hexes: hexes(h),
        },
        K::ObstacleRemoved { spell, hexes: h } => Event::ObstacleRemoved {
            spell: spell.raw(),
            hexes: hexes(h),
        },
        K::NaturalObstacleRemoved { anchor, hexes: h } => Event::NaturalObstacleRemoved {
            anchor: anchor.0,
            hexes: hexes(h),
        },
        K::ObstacleRevealed {
            side: s,
            spell,
            hexes: h,
        } => Event::ObstacleRevealed {
            side: side(s),
            spell: spell.raw(),
            hexes: hexes(h),
        },
        K::ObstacleTriggered { stack, spell, hex } => Event::ObstacleTriggered {
            stack: stack.0,
            spell: spell.raw(),
            hex: hex.0,
        },
        K::SpellRaised {
            target,
            hit_points,
            creatures,
            temporary,
        } => Event::SpellRaised {
            target: target.0,
            hit_points: wide(hit_points),
            creatures,
            temporary,
        },
        K::Sacrificed { victim, creatures } => Event::Sacrificed {
            victim: victim.0,
            creatures,
        },
        K::Reborn { stack, creatures } => Event::Reborn {
            stack: stack.0,
            creatures,
        },
        K::StackCreated {
            stack,
            creature,
            count,
            hex,
            original,
        } => Event::StackCreated {
            stack: stack.0,
            creature: creature.raw(),
            count,
            hex: hex.0,
            original: original.map(|s| s.0),
        },
        K::TurnedSummoned { stack, rounds } => Event::TurnedSummoned {
            stack: stack.0,
            rounds,
        },
        K::Vanished { stack, creatures } => Event::Vanished {
            stack: stack.0,
            creatures,
        },
        K::Teleported { stack, from, to } => Event::Teleported {
            stack: stack.0,
            from: from.0,
            to: to.0,
        },
        K::BattleEnded { result } => Event::BattleEnded {
            result: outcome(result),
        },
    }
}
