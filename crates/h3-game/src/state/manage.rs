//! The management commands (GAM-MAN-001; E08 T51a, T51b; increment 6's design note §10 in
//! `E08-log.md`): a player's commands on its heroes on the map in its turn, outside a decision one
//! hero at a time; a command between two armies needs them together, a meeting's two heroes
//! (GAM-DEC-003) or a hero and its army dialog's group (GAM-DEC-004), and while one is open the
//! commands name only what it holds. The game checks the actor, the turn and the decision, which
//! armies are together, then the context's rules (the adventure's for armies, ADR-0016 point 3;
//! `h3-heroes`' for artifacts, ADR-0013 point 14) before anything changes, then tells the owner
//! alone.

use alloc::vec::Vec;

use h3_adventure::{ArmyCommand, ArmyCommandError, ArmyRef, ArmySlot};
use h3_content::Ruleset;
use h3_contracts::{ArmyStack, HeroId, PlayerId};
use h3_heroes::{
    ArtifactMove, HeroChange, HeroDetails, assemble_changes, disassemble_changes, move_changes,
};

use super::{Decision, GameState, Together};
use crate::GameRuleset;
use crate::command::{Actor, GameCommandError, GameEvent, Sink};
use crate::manage::{Manage, ManageError};

impl GameState {
    /// `Manage`: a management command of `player` (GAM-MAN-001; E08 T51a, T51b): in its turn, on
    /// its heroes on the map, without an open decision one hero at a time; with a meeting open on
    /// its two heroes, also between them; with an army dialog open on its hero and the group, also
    /// between them ([`Self::check_managing`], [`Together::admits`]). The army commands
    /// change the adventure's armies and the group, each slot named told (`StackChanged`); the
    /// formation and the flag the hero on the map (`FormationSet`, `TacticsSet`); the artifact
    /// commands the heroes' details through `h3-heroes`' changes, each told (`HeroChanged`). Seen
    /// by the owner alone (ADR-0016 point 6).
    ///
    /// # Errors
    ///
    /// [`Self::check_managing`]'s, `DecisionOpen` for a hero the open decision does not hold,
    /// then [`ManageError`]; nothing changed.
    // rule: GAM-MAN-001
    pub(crate) fn manage<S: Sink>(
        &mut self,
        rules: &GameRuleset,
        player: PlayerId,
        command: Manage,
        sink: &mut S,
    ) -> Result<(), GameCommandError> {
        let together = self.check_managing(player)?;
        match command {
            Manage::Army(army) => together.admits(army.armies())?,
            Manage::Artifact { from, to } => together.admits_heroes([from.0, to.0].into_iter())?,
            Manage::Formation { hero, .. }
            | Manage::Tactics { hero, .. }
            | Manage::Assemble { hero, .. }
            | Manage::Disassemble { hero, .. } => together.admits_heroes(core::iter::once(hero))?,
        }
        self.manage_checked(rules, player, together, command, sink)
            .map_err(GameCommandError::Manage)
    }

    /// Checks that `player` may give a management command now and returns what its commands may
    /// name together (GAM-MAN-001; E08 T51b): its turn is running, its first open decision, if
    /// any, is a meeting or an army dialog (another is `DecisionOpen`), and the game waits for no
    /// other player's level-up.
    pub(crate) fn check_managing(&self, player: PlayerId) -> Result<Together, GameCommandError> {
        self.check_acting(player)?;
        let together = self
            .together(player)
            .ok_or(GameCommandError::DecisionOpen)?;
        self.check_held(Actor::Player(player))?;
        Ok(together)
    }

    /// [`Self::manage`] after the turn's, the decision's and the armies' checks.
    fn manage_checked<S: Sink>(
        &mut self,
        rules: &GameRuleset,
        player: PlayerId,
        together: Together,
        command: Manage,
        sink: &mut S,
    ) -> Result<(), ManageError> {
        let core = rules.core();
        match command {
            Manage::Army(command) => self.manage_army(rules, player, &command, sink)?,
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
                let other = self.own_details(player, to.0)?;
                // Between two heroes only in a meeting (GAM-DEC-003): the moving hero's changes,
                // then the receiving hero's (HER-ART-005).
                let other = (to.0 != from.0).then_some(other);
                if other.is_some() && !matches!(together, Together::Meeting(_)) {
                    return Err(ManageError::NotTogether);
                }
                let moved = ArtifactMove {
                    from: from.1,
                    to: to.1,
                };
                let changes =
                    move_changes(details, other, core, moved).map_err(ManageError::Artifact)?;
                self.change_all(core, player, from.0, changes.from, sink);
                self.change_all(core, player, to.0, changes.to, sink);
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

    /// An army command of `player` whose armies are together: the adventure applies it with the
    /// open army dialog's group, if any (written back only when it applies), then each slot it
    /// names is told with what it holds after it.
    fn manage_army<S: Sink>(
        &mut self,
        rules: &GameRuleset,
        player: PlayerId,
        command: &ArmyCommand,
        sink: &mut S,
    ) -> Result<(), ManageError> {
        // The open army dialog, its player's first decision (`check_managing`): its index and
        // hero, and a copy of its group the adventure changes.
        let dialog = self.first(player).zip(self.army_dialog_holder(player));
        let held = dialog.map(|(at, (hero, _, holder))| (at, hero, holder));
        let mut group = dialog.map(|(_, (_, group, _))| group.to_vec());
        self.adventure
            .manage_army(rules.adventure(), player, command, group.as_mut())
            .map_err(ManageError::of_army)?;
        if let Some(((at, hero, holder), group)) = held.zip(group) {
            let owner = player;
            let dialog = Decision::ArmyDialog {
                owner,
                hero,
                group,
                holder,
            };
            self.replace(at, dialog);
        }
        let (from, to) = command.slots();
        for at in core::iter::once(from).chain(to) {
            let stack = self.stack_at(player, at);
            sink.emit(GameEvent::StackChanged { player, at, stack });
        }
        Ok(())
    }

    /// The stack at `at`: in its hero's army on the map, or in `player`'s open army dialog's group.
    fn stack_at(&self, player: PlayerId, at: ArmySlot) -> Option<ArmyStack> {
        let army = match at.army {
            ArmyRef::Hero(hero) => self.adventure.hero(hero).map(|h| &h.army[..]),
            ArmyRef::Group => self.army_dialog(player).map(|(_, group)| group),
        };
        army.and_then(|army| army.iter().find(|s| s.slot == at.slot))
            .copied()
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

impl Together {
    /// Checks that a command may name `heroes` (E08 T51b): any outside a decision (each one's
    /// owner checked after), else only heroes the open meeting or army dialog holds.
    ///
    /// # Errors
    ///
    /// `DecisionOpen` for a hero it does not hold.
    pub(crate) fn admits_heroes(
        self,
        mut heroes: impl Iterator<Item = HeroId>,
    ) -> Result<(), GameCommandError> {
        let held = |hero: HeroId| match self {
            Self::Alone => true,
            Self::Meeting(held) => held.contains(&hero),
            Self::Dialog(held) => held == hero,
        };
        if heroes.all(held) {
            Ok(())
        } else {
            Err(GameCommandError::DecisionOpen)
        }
    }

    /// Checks that an army command's armies (`ArmyCommand::armies`) may meet (ADR-0017 point 6;
    /// E08 T51a, T51b): outside a decision one hero's army (the group alone is `NoGroup`, two
    /// armies `NotTogether`); in a meeting its two heroes' (the group is `NoGroup`); in an army
    /// dialog its hero's and the group; then that the open decision holds each hero named
    /// ([`Self::admits_heroes`]).
    ///
    /// # Errors
    ///
    /// `Manage(Army(NoGroup))`, `Manage(NotTogether)` or `DecisionOpen`.
    // rule: GAM-MAN-001
    pub(crate) fn admits(
        self,
        (from, to): (ArmyRef, Option<ArmyRef>),
    ) -> Result<(), GameCommandError> {
        let no_group = ManageError::Army(ArmyCommandError::NoGroup);
        let error = match (self, (from, to)) {
            (Self::Alone, (ArmyRef::Hero(_), None)) | (Self::Dialog(_), _) => None,
            (Self::Alone, (_, Some(_))) => Some(ManageError::NotTogether),
            (Self::Alone | Self::Meeting(_), (ArmyRef::Group, _) | (_, Some(ArmyRef::Group))) => {
                Some(no_group)
            }
            (Self::Meeting(_), _) => None,
        };
        error.map_or(Ok(()), |error| Err(GameCommandError::Manage(error)))?;
        let heroes = [Some(from), to].into_iter().flatten();
        self.admits_heroes(heroes.filter_map(|army| match army {
            ArmyRef::Hero(hero) => Some(hero),
            ArmyRef::Group => None,
        }))
    }
}
