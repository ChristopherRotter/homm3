//! A player's management of its heroes on the map (ADR-0016 point 3; E08 T50, `E08-design.md`
//! §7): the army commands ([`crate::ArmyCommand`], planned by `crate::army`), the split's range,
//! the formation and the tactics flag. The game routes them and checks the turn and the decision
//! that allows two armies together (a meeting's, an army dialog's; ADR-0017 points 1 and 6, T51b);
//! here each hero named must be the player's on the map, and the group must be given.

use alloc::vec::Vec;
use core::ops::RangeInclusive;

use h3_content::Formation;
use h3_contracts::{ArmyStack, HeroId, PlayerId};

use super::AdventureState;
use crate::army::{self, ArmyCommand, ArmyCommandError, ArmyRef, ArmySlot};
use crate::ruleset::AdventureRuleset;

/// An army a command names, found: a hero's by its index among the heroes, or the group.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Held {
    Hero(usize),
    Group,
}

impl AdventureState {
    /// The index of `player`'s hero `id` on the map.
    fn own_hero(&self, player: PlayerId, id: HeroId) -> Result<usize, ArmyCommandError> {
        self.heroes
            .binary_search_by_key(&id, |h| h.id)
            .ok()
            .filter(|&index| self.heroes[index].owner == player)
            .ok_or(ArmyCommandError::NotYourHero(id))
    }

    /// The army `army` of `player`: its hero's, or the group when one is open.
    fn held(&self, player: PlayerId, army: ArmyRef, open: bool) -> Result<Held, ArmyCommandError> {
        match army {
            ArmyRef::Hero(id) => self.own_hero(player, id).map(Held::Hero),
            ArmyRef::Group if open => Ok(Held::Group),
            ArmyRef::Group => Err(ArmyCommandError::NoGroup),
        }
    }

    /// Applies `player`'s army command (ADV-ARM-001 to -005, -007; E08 T50): each hero it names
    /// must be the player's on the map, and the group, if named, is `group`, the open army
    /// dialog's (the game holds it, GAM-DEC-004); the plan ([`crate::army`]) is checked whole,
    /// then the armies change. The game checks that the armies may meet (one hero's, a meeting's
    /// two, a hero's and its army dialog's group: T51b) and tells the owner (T51a). Emits nothing.
    ///
    /// # Errors
    ///
    /// An [`ArmyCommandError`]; nothing changed.
    // rule: ADV-ARM-001, ADV-ARM-002, ADV-ARM-003, ADV-ARM-004, ADV-ARM-005, ADV-ARM-007
    pub fn manage_army(
        &mut self,
        part: &AdventureRuleset,
        player: PlayerId,
        command: &ArmyCommand,
        group: Option<&mut Vec<ArmyStack>>,
    ) -> Result<(), ArmyCommandError> {
        let open = group.is_some();
        let mut none = Vec::new();
        let group = group.unwrap_or(&mut none);
        let ((source, target), (source_army, target_army)) =
            self.plan_army(part, player, command, open.then_some(&group[..]))?;
        for (held, stacks) in core::iter::once((source, source_army)).chain(target.zip(target_army))
        {
            match held {
                Held::Hero(index) => self.heroes[index].army = stacks,
                Held::Group => *group = stacks,
            }
        }
        Ok(())
    }

    /// The armies `command` names, found, and its plan ([`crate::army`]), checked whole; nothing
    /// changes.
    fn plan_army(
        &self,
        part: &AdventureRuleset,
        player: PlayerId,
        command: &ArmyCommand,
        group: Option<&[ArmyStack]>,
    ) -> Result<((Held, Option<Held>), army::Plan), ArmyCommandError> {
        let open = group.is_some();
        let (from, to) = command.armies();
        let source = self.held(player, from, open)?;
        let target = to.map(|to| self.held(player, to, open)).transpose()?;
        let rules = (part.params().army_slots(), part.params().max_stack_count());
        let read = |held: Held| match held {
            Held::Hero(index) => &self.heroes[index].army[..],
            Held::Group => group.unwrap_or_default(),
        };
        let plan = army::plan(
            command,
            (read(source), source != Held::Group),
            target.map(read),
            rules,
        )?;
        Ok(((source, target), plan))
    }

    /// The army commands `player` may give on its army `from` but the splits (GAM-MAN-002; E08
    /// T51a, ADR-0017 point 6: listed per army, as a hero's moves are): each stack's moves, in
    /// slot order, onto every other slot of `from`, then, with `to`, onto every slot of `to` (a
    /// meeting's other hero, an army dialog's group: the game names them), then the dismissals,
    /// each as [`Self::manage_army`] would accept it with `group` as the open army dialog's. None
    /// when `from` is not the player's or the group is not given. A split's counts are
    /// [`Self::split_range`]'s.
    // rule: ADV-ARM-001, ADV-ARM-002, ADV-ARM-004
    pub fn army_commands(
        &self,
        part: &AdventureRuleset,
        player: PlayerId,
        (from, to): (ArmyRef, Option<ArmyRef>),
        group: Option<&[ArmyStack]>,
    ) -> Vec<ArmyCommand> {
        let slots = part.params().army_slots();
        let slot = |army, slot| ArmySlot { army, slot };
        let targets = core::iter::once(from).chain(to);
        let moves = (0..slots).flat_map(|source| {
            targets.clone().flat_map(move |army| {
                (0..slots).map(move |target| ArmyCommand::Move {
                    from: slot(from, source),
                    to: slot(army, target),
                })
            })
        });
        let dismissals = (0..slots).map(|at| ArmyCommand::Dismiss { at: slot(from, at) });
        moves
            .chain(dismissals)
            .filter(|command| self.plan_army(part, player, command, group).is_ok())
            .collect()
    }

    /// The counts a split of `player`'s stack at `from` with `to` may leave in `to` (ADV-ARM-003,
    /// ADV-ARM-007; [`army::split_range`]), `group` being the open army dialog's: none when no
    /// split is possible (a hero not the player's, the group not open, a slot beyond the army's
    /// or holding no stack at `from`, the same slot, another creature at `to`, a single
    /// creature). The game's query for a client (`h3_game::split_range`, E08 T51a).
    pub fn split_range(
        &self,
        part: &AdventureRuleset,
        player: PlayerId,
        (from, to): (ArmySlot, ArmySlot),
        group: Option<&[ArmyStack]>,
    ) -> Option<RangeInclusive<u32>> {
        let most = part.params().max_stack_count();
        let open = group.is_some();
        let army = |slot: ArmySlot| {
            self.held(player, slot.army, open)
                .ok()
                .map(|held| match held {
                    Held::Hero(index) => &self.heroes[index].army[..],
                    Held::Group => group.unwrap_or_default(),
                })
        };
        let source = army(from)?.iter().find(|s| s.slot == from.slot).copied()?;
        // A split onto its own slot, or onto another creature, is none; a slot beyond the
        // army's holds no stack.
        let target = army(to)?.iter().find(|s| s.slot == to.slot).copied();
        let same = from == to;
        let other = target.is_some_and(|t| t.creature != source.creature);
        let fits = to.slot < part.params().army_slots();
        let range = army::split_range(source.count, target.map_or(0, |t| t.count), most);
        (!same && !other && fits && !range.is_empty()).then_some(range)
    }

    /// Sets `player`'s hero `hero`'s formation (ADV-ARM-006; E08 T50): at no cost, any number of
    /// times.
    ///
    /// # Errors
    ///
    /// [`ArmyCommandError::NotYourHero`]; nothing changed.
    // rule: ADV-ARM-006
    pub fn set_formation(
        &mut self,
        player: PlayerId,
        hero: HeroId,
        formation: Formation,
    ) -> Result<(), ArmyCommandError> {
        let index = self.own_hero(player, hero)?;
        self.heroes[index].formation = formation;
        Ok(())
    }

    /// Sets `player`'s hero `hero`'s tactics flag (ADV-ARM-006; E08 T50): on, the hero takes the
    /// tactics phase its Tactics gives it; off, it denies the phase in the battles where it would
    /// get it (GAM-BAT-009, the game's, T51a). At no cost.
    ///
    /// # Errors
    ///
    /// [`ArmyCommandError::NotYourHero`]; nothing changed.
    // rule: ADV-ARM-006
    pub fn set_tactics(
        &mut self,
        player: PlayerId,
        hero: HeroId,
        on: bool,
    ) -> Result<(), ArmyCommandError> {
        let index = self.own_hero(player, hero)?;
        self.heroes[index].tactics = on;
        Ok(())
    }
}
