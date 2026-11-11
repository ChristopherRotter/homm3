//! The bot that plays both seats of `h3-view play` (E17 M2 T4b-2a): a battle fought to its end
//! in-process, for the viewer to draw frame by frame. Not an opponent: it makes a battle that
//! moves, strikes and shoots like one, deterministically (*provisional*, the M2 review list).
//! An adapter: it reads the battle through the engine's per-side functions only, as the host's
//! bots must (ADR-0018 point 6).

use h3_battle::{BattleSide, BattleState, Command, Field, LegalCommands};
use h3_content_battle::BattleRuleset;
use h3_protocol::session::Bot;

/// Shoots if it may, else strikes, else walks toward the nearest enemy; the first such command
/// the engine lists. Defends when nothing else stays in the battle.
#[derive(Debug, Default)]
pub struct Greedy;

impl Bot for Greedy {
    fn command(
        &mut self,
        state: &BattleState,
        ruleset: &BattleRuleset,
        side: BattleSide,
        legal: &LegalCommands,
    ) -> Command {
        let staying = || legal.iter().take_while(|c| !c.leaves_battle());
        // A tactics phase ends at once: its moves would never end it.
        if staying().any(|c| matches!(c, Command::EndTactics)) {
            return Command::EndTactics;
        }
        if let Some(&shot) = staying().find(|c| matches!(c, Command::Shoot { .. })) {
            return shot;
        }
        if let Some(&strike) = staying().find(|c| matches!(c, Command::Attack { .. })) {
            return strike;
        }
        let field = Field::new(ruleset.battle().field);
        let enemies: Vec<_> = state
            .stacks()
            .iter()
            .filter(|s| s.is_alive() && s.side != side)
            .map(|s| s.footprint(field))
            .collect();
        let active = state.active().and_then(|id| state.stack(id));
        let nearest = |to| {
            active
                .and_then(|s| field.footprint(to, s.is_two_hex(), side))
                .map_or(u32::MAX, |at| {
                    enemies
                        .iter()
                        .map(|e| field.footprint_distance(at, *e))
                        .min()
                        .unwrap_or(u32::MAX)
                })
        };
        let walk = staying()
            .filter_map(|c| match *c {
                Command::Move { to } => Some((nearest(to), *c)),
                _ => None,
            })
            .min_by_key(|(d, _)| *d)
            .map(|(_, c)| c);
        walk.or_else(|| staying().find(|c| !matches!(c, Command::Wait)).copied())
            .unwrap_or(Command::Defend)
    }
}
