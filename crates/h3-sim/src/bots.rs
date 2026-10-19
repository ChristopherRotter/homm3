//! The baseline bots (E05 T7, ADR-0025 point 5; `docs/epics/E05-design.md` §9): heuristic seats
//! for the evaluation runner and the agents' ladder, never training opponents (decision 8).
//!
//! A bot reads its side's observation (the public stack records), the ruleset's creature data and
//! the engine's damage preview of its own attacks (`h3_battle::preview`, BAT-DMG-011, AI-Q6),
//! never the state itself; the legal commands are what a player may click. Neither bot casts.
//! Every choice is deterministic: ties go to the command listed first.
//!
//! - [`aggressor`]: shoots the nearest enemy it can shoot, else attacks the nearest it can reach,
//!   else walks toward the nearest.
//! - [`greedy`]: takes the shot or attack whose previewed kills are worth the most `ai_value`,
//!   less its own estimate of the retaliation (the preview shows none, AI-Q6), else walks toward
//!   the most valuable enemy.

use h3_battle::{
    BattleState, Command, Field, Footprint, LegalCommands, Observation, Stack, StackId,
    StackTraits, Status, legal_commands, observe, preview,
};
use h3_content_battle::BattleRuleset;
use h3_contracts::BattleSide;

/// The command [`aggressor`] plays for `side`, the active stack's (BAT-TRN-001).
pub fn aggressor(
    state: &BattleState,
    ruleset: &BattleRuleset,
    side: BattleSide,
    legal: &mut LegalCommands,
) -> Command {
    let view = View::new(state, ruleset, side, legal);
    let Some(me) = view.active() else {
        return view.fallback();
    };
    let nearest = |targets: &mut dyn Iterator<Item = StackId>| {
        targets.min_by_key(|&t| (view.distance(&me.footprint(view.field), t), t.0))
    };
    for shot in [true, false] {
        let mut targets = view.attacks(shot).map(|(_, t)| t);
        if let Some(target) = nearest(&mut targets) {
            return view.first_attack(shot, target);
        }
    }
    let mut enemies = view.enemies();
    match nearest(&mut enemies) {
        Some(target) => view.walk_toward(me, target),
        None => view.fallback(),
    }
}

/// The command [`greedy`] plays for `side`, the active stack's.
pub fn greedy(
    state: &BattleState,
    ruleset: &BattleRuleset,
    side: BattleSide,
    legal: &mut LegalCommands,
) -> Command {
    let view = View::new(state, ruleset, side, legal);
    let Some(me) = view.active() else {
        return view.fallback();
    };
    let scored = view.legal.iter().copied().filter_map(|command| {
        let (target, ranged) = attack_target(command)?;
        let p = preview(state, ruleset, side, command)?;
        let victim = view.stack(target);
        // Twice the mean, so that every number stays an integer.
        let killed = u64::from(p.kills[0]) + u64::from(p.kills[1]);
        let gain = killed.saturating_mul(view.value(victim));
        let loss = if ranged {
            0
        } else {
            view.retaliation_loss(me, victim, killed)
        };
        Some((i128::from(gain) - i128::from(loss), command))
    });
    // The first of the best (`max_by_key` keeps the last of equals).
    let mut best: Option<(i128, Command)> = None;
    for (score, command) in scored {
        if best.is_none_or(|(b, _)| score > b) {
            best = Some((score, command));
        }
    }
    if let Some((_, command)) = best {
        return command;
    }
    let target = view.enemies().max_by_key(|&t| {
        let s = view.stack(t);
        (
            u64::from(s.count).saturating_mul(view.value(s)),
            core::cmp::Reverse(t.0),
        )
    });
    match target {
        Some(target) => view.walk_toward(me, target),
        None => view.fallback(),
    }
}

/// What a bot reads: its side's observation, the ruleset, its legal commands.
struct View<'a> {
    obs: Observation,
    ruleset: &'a BattleRuleset,
    field: Field,
    side: BattleSide,
    legal: &'a LegalCommands,
}

impl<'a> View<'a> {
    fn new(
        state: &BattleState,
        ruleset: &'a BattleRuleset,
        side: BattleSide,
        legal: &'a mut LegalCommands,
    ) -> Self {
        legal_commands(state, ruleset, side, legal);
        Self {
            obs: observe(state, ruleset, side),
            ruleset,
            field: Field::new(ruleset.battle().field),
            side,
            legal,
        }
    }

    fn stack(&self, id: StackId) -> &Stack {
        &self.obs.stacks.as_slice()[usize::from(id.0)]
    }

    /// The active stack, if it is this side's (outside the tactics phase's placement).
    fn active(&self) -> Option<Stack> {
        self.obs
            .active
            .map(|id| *self.stack(id))
            .filter(|s| s.side == self.side && self.obs.tactics.is_none())
    }

    /// The living enemy stacks on the field.
    fn enemies(&self) -> impl Iterator<Item = StackId> + '_ {
        self.obs
            .stacks
            .iter()
            .enumerate()
            .filter(|(_, s)| s.side != self.side && s.is_alive() && !s.is_turret())
            .map(|(i, _)| StackId(u8::try_from(i).unwrap_or(u8::MAX)))
    }

    /// The legal shots (`shot`) or melee attacks, with their targets.
    fn attacks(&self, shot: bool) -> impl Iterator<Item = (Command, StackId)> + '_ {
        self.legal.iter().copied().filter_map(move |c| {
            attack_target(c)
                .filter(|&(_, ranged)| ranged == shot)
                .map(|(t, _)| (c, t))
        })
    }

    fn first_attack(&self, shot: bool, target: StackId) -> Command {
        self.attacks(shot)
            .find(|&(_, t)| t == target)
            .map_or(Command::Defend, |(c, _)| c)
    }

    fn distance(&self, from: &Footprint, to: StackId) -> u32 {
        self.field
            .footprint_distance(*from, self.stack(to).footprint(self.field))
    }

    /// The legal move that ends nearest to `target`, ties to the first listed; the fallback
    /// when the stack cannot move.
    fn walk_toward(&self, me: Stack, target: StackId) -> Command {
        self.legal
            .iter()
            .copied()
            .filter_map(|c| match c {
                Command::Move { to } => self
                    .field
                    .footprint(to, me.is_two_hex(), me.side)
                    .map(|fp| (self.distance(&fp, target), c)),
                _ => None,
            })
            .min_by_key(|&(d, _)| d)
            .map_or_else(|| self.fallback(), |(_, c)| c)
    }

    /// Without an attack or a move: end the tactics phase, defend, wait, or the first command
    /// that stays in the battle (a war machine's: the Catapult's wall shot, the tent's heal).
    fn fallback(&self) -> Command {
        let first = |wanted: fn(&Command) -> bool| self.legal.iter().copied().find(wanted);
        first(|c| matches!(c, Command::EndTactics))
            .or_else(|| first(|c| matches!(c, Command::Defend)))
            .or_else(|| first(|c| matches!(c, Command::Wait)))
            .or_else(|| first(|c| !c.leaves_battle()))
            .unwrap_or(Command::Defend)
    }

    /// One creature's `ai_value` (0 outside the ruleset).
    fn value(&self, stack: &Stack) -> u64 {
        self.ruleset
            .creature_info(stack.creature)
            .map_or(0, |info| u64::from(info.ai_value))
    }

    /// Twice the bot's estimate of the value `me` loses to `victim`'s retaliation after a melee
    /// strike that kills `killed / 2` creatures (its own approximation, not the engine's: the
    /// preview shows no retaliation, AI-Q6): the survivors' mean damage, scaled by attack against
    /// defense as BAT-DMG-002's first stage does (5 % per point up to +300 %, 2.5 % down to
    /// −70 %), in creatures of `me`.
    fn retaliation_loss(&self, me: Stack, victim: &Stack, killed: u64) -> u64 {
        let retaliates = victim.retaliations_left > 0
            && !me.has(StackTraits::NO_ENEMY_RETALIATION)
            && !victim.has(StackTraits::WAR_MACHINE)
            && !victim.spells.status.contains(Status::PETRIFIED);
        let survivors_x2 = (2 * u64::from(victim.count)).saturating_sub(killed);
        if !retaliates || survivors_x2 == 0 || me.max_hp == 0 {
            return 0;
        }
        let mean_x2 = u64::from(victim.damage_min) + u64::from(victim.damage_max);
        let (a, d) = (i64::from(victim.attack), i64::from(me.defense));
        let permille = if a >= d {
            (1000 + 50 * (a - d)).min(4000)
        } else {
            (1000 - 25 * (d - a)).max(300)
        };
        // Damage × 4 (survivors and mean both doubled), then per mille.
        let damage_x4 = survivors_x2 * mean_x2 * u64::try_from(permille).unwrap_or(1000) / 1000;
        let lost_hp_x4 = damage_x4 + 4 * u64::from(me.max_hp - me.top_hp.min(me.max_hp));
        let creatures_x2 = (lost_hp_x4 / u64::from(me.max_hp) / 2).min(2 * u64::from(me.count));
        creatures_x2.saturating_mul(self.value(&me))
    }
}

/// The target of an attack or a shot, and whether it is a shot.
fn attack_target(command: Command) -> Option<(StackId, bool)> {
    match command {
        Command::Attack { target, .. } => Some((target, false)),
        Command::Shoot { target, .. } => Some((target, true)),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
