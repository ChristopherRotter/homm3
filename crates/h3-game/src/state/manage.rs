//! The management commands (GAM-MAN-001; E08 T51a, `E08-design.md` §10): a player's commands on
//! its heroes on the map in its turn, outside a decision; a command between two armies needs
//! them together (a meeting's two heroes, a hero and its army dialog's group: T51b). The game
//! checks the actor, the turn and the decision, which armies are together, then the context's
//! rules (the adventure's for armies, ADR-0016 point 3; `h3-heroes`' for artifacts, ADR-0013
//! point 14) before anything changes, then tells the owner alone.

use alloc::vec::Vec;

use h3_adventure::{ArmyCommand, ArmyCommandError, ArmyRef};
use h3_content::Ruleset;
use h3_contracts::{HeroId, PlayerId};
use h3_heroes::{
    ArtifactMove, HeroChange, HeroDetails, assemble_changes, disassemble_changes, move_changes,
};

use super::GameState;
use crate::GameRuleset;
use crate::command::{GameCommandError, GameEvent, Sink};
use crate::manage::{Manage, ManageError};

impl GameState {
    /// `Manage`: a management command of `player` (GAM-MAN-001; E08 T51a): in its turn, without an
    /// open decision, on its heroes on the map; an army command between two armies is refused
    /// (`NotTogether`) until a meeting or an army dialog holds them together (T51b). The army
    /// commands change the adventure's armies, each slot named told (`StackChanged`); the
    /// formation and the flag the hero on the map (`FormationSet`, `TacticsSet`); the artifact
    /// commands the hero's details through `h3-heroes`' changes, each told (`HeroChanged`). Seen
    /// by the owner alone (ADR-0016 point 6).
    ///
    /// # Errors
    ///
    /// The turn's and the decision's (`check_ready`), then [`ManageError`]; nothing changed.
    // rule: GAM-MAN-001
    pub(crate) fn manage<S: Sink>(
        &mut self,
        rules: &GameRuleset,
        player: PlayerId,
        command: Manage,
        sink: &mut S,
    ) -> Result<(), GameCommandError> {
        self.check_ready(player)?;
        self.manage_checked(rules, player, command, sink)
            .map_err(GameCommandError::Manage)
    }

    /// [`Self::manage`] after the turn's and the decision's checks.
    fn manage_checked<S: Sink>(
        &mut self,
        rules: &GameRuleset,
        player: PlayerId,
        command: Manage,
        sink: &mut S,
    ) -> Result<(), ManageError> {
        let core = rules.core();
        match command {
            Manage::Army(command) => {
                let hero = alone(&command)?;
                self.adventure
                    .manage_army(rules.adventure(), player, &command, None)
                    .map_err(ManageError::of_army)?;
                let (from, to) = command.slots();
                for at in core::iter::once(from).chain(to) {
                    let stack = self
                        .adventure
                        .hero(hero)
                        .and_then(|h| h.army.iter().find(|s| s.slot == at.slot))
                        .copied();
                    sink.emit(GameEvent::StackChanged { player, at, stack });
                }
            }
            Manage::Formation { hero, formation } => {
                self.adventure
                    .set_formation(player, hero, formation)
                    .map_err(ManageError::of_army)?;
                sink.emit(GameEvent::FormationSet {
                    player,
                    hero,
                    formation,
                });
            }
            Manage::Tactics { hero, on } => {
                self.adventure
                    .set_tactics(player, hero, on)
                    .map_err(ManageError::of_army)?;
                sink.emit(GameEvent::TacticsSet { player, hero, on });
            }
            Manage::Artifact { from, to } => {
                let details = self.own_details(player, from.0)?;
                self.own_details(player, to.0)?;
                if to.0 != from.0 {
                    return Err(ManageError::NotTogether);
                }
                let moved = ArtifactMove {
                    from: from.1,
                    to: to.1,
                };
                let changes = move_changes(details, None, core, moved)
                    .map_err(ManageError::Artifact)?
                    .from;
                self.change_all(core, player, from.0, changes, sink);
            }
            Manage::Assemble { hero, at } => {
                let changes = assemble_changes(self.own_details(player, hero)?, core, at)
                    .map_err(ManageError::Combine)?;
                self.change_all(core, player, hero, changes, sink);
            }
            Manage::Disassemble { hero, at } => {
                let changes = disassemble_changes(self.own_details(player, hero)?, core, at)
                    .map_err(ManageError::Combine)?;
                self.change_all(core, player, hero, changes, sink);
            }
        }
        Ok(())
    }

    /// The details of `player`'s hero `hero` on the map.
    pub(crate) fn own_details(
        &self,
        player: PlayerId,
        hero: HeroId,
    ) -> Result<&HeroDetails, ManageError> {
        // Every hero on the map has its details in the store (GAM-HER-001).
        self.adventure
            .hero(hero)
            .filter(|h| h.owner == player)
            .and(self.hero(hero))
            .ok_or(ManageError::NotYourHero(hero))
    }

    /// Applies `changes` to `hero`, checked in order by `h3-heroes`, each told to `player`.
    fn change_all<S: Sink>(
        &mut self,
        core: &Ruleset,
        player: PlayerId,
        hero: HeroId,
        changes: Vec<HeroChange>,
        sink: &mut S,
    ) {
        for change in changes {
            self.change_hero(core, hero, Some(player), change, sink);
        }
    }
}

/// The one hero whose army `command` changes outside a meeting and an army dialog: its source
/// army must be a hero's (the group is an open army dialog's, `NoGroup`), and it names no other
/// army (`NotTogether`; E08 T51a, ADR-0017 point 6).
pub(crate) fn alone(command: &ArmyCommand) -> Result<HeroId, ManageError> {
    match command.armies() {
        (ArmyRef::Hero(hero), None) => Ok(hero),
        (ArmyRef::Group, None) => Err(ManageError::Army(ArmyCommandError::NoGroup)),
        (_, Some(_)) => Err(ManageError::NotTogether),
    }
}
