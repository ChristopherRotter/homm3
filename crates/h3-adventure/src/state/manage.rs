//! A player's management of its heroes on the map (ADR-0016 point 3; E08 T50, increment 6's design
//! note §7 in `E08-log.md`): the army commands ([`crate::ArmyCommand`], planned by `crate::army`),
//! the split's range, the formation and the tactics flag. The game routes them and checks the turn
//! and the decision that allows two armies together (a meeting's, an army dialog's, since E08
//! T114a2 a map garrison's exchange; ADR-0017 points 1 and 6, T51b); here each hero named must be
//! the player's on the map, the group must be given, a map garrison must be one, and each army's
//! bar owner decides what may leave and enter it (the one owner rule, ADV-GAR-003, D-61).

use alloc::vec::Vec;
use core::ops::RangeInclusive;

use h3_content::Formation;
use h3_contracts::{ArmyStack, HeroId, PlayerId};

use super::AdventureState;
use crate::army::{self, ArmyCommand, ArmyCommandError, ArmyRef, ArmySlot};
use crate::ruleset::AdventureRuleset;
use crate::setup::Holds;

/// An army a command names, found: a hero's by its index among the heroes, the group, or since E08
/// T114a2 a map garrison's by its object index, with whether its **bar owner** is the acting
/// player (the one owner rule, ADV-GAR-003; increment 12's design note §13: a stack leaves only
/// an own bar, and enters another's, nobody's included, only into an empty slot). The flag rides
/// in the garrison's variant, so the enum keeps its size (`bench-check`, manage-army-12).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Held {
    Hero(usize),
    Group,
    MapGarrison { index: usize, own: bool },
}

impl Held {
    /// Whether the army's last stack never leaves it (ADV-ARM-002): a hero's; the group and a
    /// stationary army may be emptied (increment 12's design note §13).
    const fn keeps_last(self) -> bool {
        matches!(self, Self::Hero(_))
    }

    /// Whether the army's bar owner is the acting player: a hero's (its owner, the player's) and
    /// the group (its dialog's owner, the player) always; a map garrison's as found.
    const fn own(self) -> bool {
        match self {
            Self::Hero(_) | Self::Group => true,
            Self::MapGarrison { own, .. } => own,
        }
    }
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

    /// The army `army` of `player`, with whether its bar owner is `player` (ADV-GAR-003,
    /// [`Held::own`]): its hero's (its owner's, the player's), or the group when one is open (its
    /// dialog's owner's, the player's); a map garrison at its object index (E08 T114a2), its
    /// owner's when removable and nobody's when not (the original passes −1, survey
    /// `E08-garrisons-bonuses.md` §6); never a town's garrison (E09 T8b2b: not managed before the
    /// town screen; its bar owner, its town's owner, is the town lane's hook, E09 T28a).
    // rule: ADV-GAR-003
    #[inline]
    fn held(&self, player: PlayerId, army: ArmyRef, open: bool) -> Result<Held, ArmyCommandError> {
        match army {
            ArmyRef::Hero(id) => self.own_hero(player, id).map(Held::Hero),
            ArmyRef::Group if open => Ok(Held::Group),
            ArmyRef::Group => Err(ArmyCommandError::NoGroup),
            ArmyRef::Garrison(town) => Err(ArmyCommandError::Garrison(town)),
            ArmyRef::MapGarrison(index) => self.map_garrison(player, index),
        }
    }

    /// [`Self::held`]'s map garrison at object index `index` (E08 T114a2): out of line, so a
    /// hero's lookup stays as small as before (`bench-check`, manage-army-12: +3.7 % inlined).
    #[cold]
    #[inline(never)]
    fn map_garrison(&self, player: PlayerId, index: u32) -> Result<Held, ArmyCommandError> {
        let index = usize::try_from(index).unwrap_or(usize::MAX);
        match self.object(index).map(|object| &object.holds) {
            Some(Holds::Garrison { removable, .. }) => {
                let own = *removable && self.owner(index) == Some(player);
                Ok(Held::MapGarrison { index, own })
            }
            _ => Err(ArmyCommandError::NoGarrison),
        }
    }

    /// The stacks of the army `held`, `group` being the open army dialog's: always inlined, a
    /// map garrison's read out of line (`bench-check`, manage-army-12: a hero's and the group's
    /// reads stay as before E08 T114a2).
    #[inline(always)]
    fn stacks<'a>(&'a self, held: Held, group: Option<&'a [ArmyStack]>) -> &'a [ArmyStack] {
        match held {
            Held::Hero(index) => &self.heroes[index].army,
            Held::Group => group.unwrap_or_default(),
            Held::MapGarrison { index, .. } => self.garrison_stacks(index),
        }
    }

    /// [`Self::garrison_army`] out of line, for [`Self::stacks`].
    #[cold]
    #[inline(never)]
    fn garrison_stacks(&self, index: usize) -> &[ArmyStack] {
        self.garrison_army(index)
    }

    /// The one owner rule's refusal of `command` on its armies `source` and `target` (ADV-GAR-003;
    /// E08 T114a2): a stack taken from a bar not the player's (`NotOwn`), or put onto a stack in
    /// such a bar (`OnlyEmpty`; a slot past the army's left to the plan's `NoSlot`). Asked only
    /// when a bar is not the player's (a map garrison's), out of line.
    #[cold]
    #[inline(never)]
    fn check_bar_owners(
        &self,
        command: &ArmyCommand,
        (source, target): (Held, Option<Held>),
        group: Option<&[ArmyStack]>,
    ) -> Result<(), ArmyCommandError> {
        if !source.own() {
            return Err(ArmyCommandError::NotOwn);
        }
        match (target, command.slots()) {
            (Some(held), (_, Some(to)))
                if !held.own() && self.stacks(held, group).iter().any(|s| s.slot == to.slot) =>
            {
                Err(ArmyCommandError::OnlyEmpty)
            }
            _ => Ok(()),
        }
    }

    /// Applies `player`'s army command (ADV-ARM-001 to -005, -007; E08 T50): each hero it names
    /// must be the player's on the map, and the group, if named, is `group`, the open army
    /// dialog's (the game holds it, GAM-DEC-004); a map garrison it names must be one (E08
    /// T114a2); the one owner rule holds (ADV-GAR-003: [`ArmyCommandError::NotOwn`],
    /// [`ArmyCommandError::OnlyEmpty`]); the plan ([`crate::army`]) is checked whole, then the
    /// armies change, a map garrison's written as its army ([`Self::garrison_army`]). The game
    /// checks that the armies may meet (one hero's, a meeting's two, a hero's and its army
    /// dialog's group: T51b; a hero's and its exchange's garrison: T114a2) and tells the owner
    /// (T51a). Emits nothing.
    ///
    /// # Errors
    ///
    /// An [`ArmyCommandError`]; nothing changed.
    // rule: ADV-ARM-001, ADV-ARM-002, ADV-ARM-003, ADV-ARM-004, ADV-ARM-005, ADV-ARM-007
    // rule: ADV-GAR-003
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
                Held::MapGarrison { index, .. } => self.set_army_left(index, stacks),
            }
        }
        Ok(())
    }

    /// The armies `command` names, found, the one owner rule checked (ADV-GAR-003; E08 T114a2:
    /// a stack taken only from an own bar, `NotOwn`; into another bar only onto an empty slot,
    /// `OnlyEmpty`, a slot beyond the army's left to the plan's `NoSlot`), and its plan
    /// ([`crate::army`]), checked whole; nothing changes.
    // rule: ADV-GAR-003
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
        if !source.own() || target.is_some_and(|held| !held.own()) {
            self.check_bar_owners(command, (source, target), group)?;
        }
        let rules = (part.params().army_slots(), part.params().max_stack_count());
        let plan = army::plan(
            command,
            (self.stacks(source, group), source.keeps_last()),
            target.map(|held| self.stacks(held, group)),
            rules,
        )?;
        Ok(((source, target), plan))
    }

    /// The army commands `player` may give on its army `from` but the splits (GAM-MAN-002; E08
    /// T51a, ADR-0017 point 6: listed per army, as a hero's moves are): each stack's moves, in
    /// slot order, onto every other slot of `from`, then, with `to`, onto every slot of `to` (a
    /// meeting's other hero, an army dialog's group: the game names them), then the dismissals,
    /// each as [`Self::manage_army`] would accept it with `group` as the open army dialog's (the
    /// one owner rule included, ADV-GAR-003: none from a bar not the player's). None when `from`
    /// is not the player's or the group is not given. A split's counts are
    /// [`Self::split_range`]'s.
    // rule: ADV-ARM-001, ADV-ARM-002, ADV-ARM-004, ADV-GAR-003
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
    /// creature; since E08 T114a2 a stack in a bar not the player's, or a stack at `to` in such a
    /// bar: the one owner rule, ADV-GAR-003). The game's query for a client
    /// (`h3_game::split_range`, E08 T51a).
    // rule: ADV-ARM-003, ADV-GAR-003
    pub fn split_range(
        &self,
        part: &AdventureRuleset,
        player: PlayerId,
        (from, to): (ArmySlot, ArmySlot),
        group: Option<&[ArmyStack]>,
    ) -> Option<RangeInclusive<u32>> {
        let most = part.params().max_stack_count();
        let open = group.is_some();
        let source_army = self.held(player, from.army, open).ok()?;
        let source = self
            .stacks(source_army, group)
            .iter()
            .find(|s| s.slot == from.slot)
            .copied()?;
        // A split onto its own slot, or onto another creature, is none; a slot beyond the
        // army's holds no stack.
        let target_army = self.held(player, to.army, open).ok()?;
        let target = self
            .stacks(target_army, group)
            .iter()
            .find(|s| s.slot == to.slot)
            .copied();
        let same = from == to;
        let other = target.is_some_and(|t| t.creature != source.creature);
        let fits = to.slot < part.params().army_slots();
        // The one owner rule (ADV-GAR-003): from an own bar only, into another's only an empty
        // slot (a hero's and the group's bars are the player's).
        if !source_army.own() || (target.is_some() && !target_army.own()) {
            return None;
        }
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
