//! A town's two rows (TWN-ROW-002 to -004; E09 T28a, the town epic's design note §52, §53; survey
//! `E09-growth.md` §9): the garrison hero, a hero standing in its town's garrison off the map with
//! its whole record, and the three moves of the town screen's portraits between the visitor, the
//! hero on the entrance (TWN-VIS-002), and the garrison: into the garrison, out of it, the swap.
//! The game checks the actor's turn and the town's owner first (`h3-game`'s town module, which
//! wraps them as `TownCommand::Heroes`); each refusal here changes nothing.

use alloc::sync::Arc;
use alloc::vec::Vec;

use h3_content::Ruleset;
use h3_contracts::{ArmyStack, HeroId, PlayerId, TownId};
use h3_kernel::{StateHasher, order};

use crate::army::{ArmyRef, ArmySlot};
use crate::command::{AdventureEvent, AdventureEventKind, Sink, Viewers};
use crate::ruleset::AdventureRuleset;
use crate::setup::Pos;
use crate::state::{AdventureState, HeroOnMap};

/// The towns' garrison heroes, by town in `TownId` order: each its town and its record as it
/// was on the map ([`AdventureState::town_heroes`]).
pub(crate) type TownHeroes = Vec<(TownId, HeroOnMap)>;

/// A hero's move between a town's two bars (TWN-ROW-002 to -004; the game's
/// `h3_towns::TownHeroChange`, which it maps one to one).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TownHeroMove {
    /// The visitor into the garrison (TWN-ROW-002).
    Enter,
    /// The garrison hero out onto the entrance (TWN-ROW-003).
    Leave,
    /// The two change places (TWN-ROW-004).
    Swap,
}

/// `Enter`'s checked move: the town's object index, the visitor's place among the heroes on the
/// map, its army after the absorb.
struct Entering {
    index: usize,
    at: usize,
    army: Vec<ArmyStack>,
}

/// Why a hero's move between a town's two bars was refused (TWN-ROW-002 to -004); nothing
/// changed. The game gives the town's refusals first (no such town, not the player's).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TownHeroError {
    /// The map has no such town.
    UnknownTown,
    /// A hero stands in the town's garrison already (TWN-ROW-002).
    GarrisonHeld,
    /// No hero of the player's stands on the town's entrance (TWN-ROW-002, -004; an ally's
    /// visitor moves no hero, TD-17).
    NoVisitor,
    /// The garrison's stacks find no slot in the visitor's army, even after merging the visitor's
    /// own stacks of one creature (TWN-ROW-002: GENRLTXT 276).
    ArmiesDoNotFit,
    /// A stack would hold more creatures than a stack's most (TWN-ROW-002, ADV-ARM-007).
    TooManyCreatures,
    /// No hero stands in the town's garrison (TWN-ROW-003, -004).
    NoGarrisonHero,
    /// A hero stands on the town's entrance (TWN-ROW-003).
    EntranceHeld,
    /// The player has the most heroes on the map already (TWN-ROW-003: GENRLTXT 19).
    TooManyHeroes,
    /// The garrison hero's army holds no stack (TWN-ROW-003: GENRLTXT 20).
    EmptyArmy,
}

impl AdventureState {
    /// `town`'s garrison hero, if one stands there (E09 T28a).
    pub fn town_hero(&self, town: TownId) -> Option<&HeroOnMap> {
        let all = self.town_heroes();
        all.binary_search_by_key(&town, |&(id, _)| id)
            .ok()
            .map(|at| &all[at].1)
    }

    /// The town whose garrison hero `hero` is, with its record, if it is one (E09 T28a).
    pub fn garrisoned(&self, hero: HeroId) -> Option<(TownId, &HeroOnMap)> {
        self.town_heroes()
            .iter()
            .find(|(_, on)| on.id == hero)
            .map(|(town, on)| (*town, on))
    }

    /// The place of `player`'s garrison hero `hero` among the towns' garrison heroes (E09 T28a:
    /// the army commands' and the management's lookup after the heroes on the map's).
    pub(crate) fn own_town_hero(&self, player: PlayerId, hero: HeroId) -> Option<usize> {
        self.town_heroes()
            .iter()
            .position(|(_, on)| on.id == hero && on.owner == player)
    }

    /// The garrison hero at `at` among the towns' garrison heroes, to change (the block copied
    /// when a fork shares it).
    pub(crate) fn town_hero_at_mut(&mut self, at: usize) -> Option<&mut HeroOnMap> {
        let (_, block) = self.rows_mut();
        block
            .as_mut()
            .and_then(|all| Arc::make_mut(all).get_mut(at))
            .map(|(_, on)| on)
    }

    /// Hashes the towns' garrison heroes (adventure state hash v25): their number, then each its
    /// town and its record as a hero on the map's ([`Self::hash_hero`]).
    pub(crate) fn hash_town_heroes(&self, h: &mut StateHasher) {
        let all = self.town_heroes();
        h.write_len(all.len());
        for (town, hero) in all {
            h.write_u16(town.raw());
            self.hash_hero(h, hero);
        }
    }

    /// Checks `player`'s hero move `change` at `town` as [`Self::move_town_hero`] would make it,
    /// `most` the most heroes a player has on the map (the town part's): the legal moves'
    /// query (TWN-ROW-002 to -004). Nothing changes.
    ///
    /// # Errors
    ///
    /// [`Self::move_town_hero`]'s refusal.
    pub fn check_town_hero(
        &self,
        part: &AdventureRuleset,
        player: PlayerId,
        (town, most): (TownId, u8),
        change: TownHeroMove,
    ) -> Result<(), TownHeroError> {
        match change {
            TownHeroMove::Enter => self.check_enter(part, player, town).map(|_| ()),
            TownHeroMove::Leave => self.check_leave(player, (town, most)).map(|_| ()),
            TownHeroMove::Swap => self.check_swap(player, town).map(|_| ()),
        }
    }

    /// Which of the three moves `player` may make at `town` now, in [`TownHeroMove`]'s order
    /// (`Enter`, `Leave`, `Swap`), each as [`Self::check_town_hero`] answers it: the legal
    /// listing's query, read cheaply (`bench-check`, legal-towns-36: three checks a town computed
    /// its entrance from its template three times): the entrance the static map's
    /// ([`crate::StaticMap::town_entrance`]), the visitor the player's hero on it. `most` is the
    /// most heroes a player has on the map (the town part's). The town's owner is the game's check.
    pub fn town_hero_moves(
        &self,
        part: &AdventureRuleset,
        player: PlayerId,
        (town, most): (TownId, u8),
    ) -> [bool; 3] {
        let Ok((index, entrance)) = self.town_entrance(town) else {
            return [false; 3];
        };
        let visitor = self.own_visitor(player, entrance).ok();
        self.moves_at(part, player, (town, most), (index, visitor))
    }

    /// [`Self::town_hero_moves`]' three at `town`, its object at `index`, `visitor` the place of
    /// the player's hero on its entrance among the heroes on the map.
    fn moves_at(
        &self,
        part: &AdventureRuleset,
        player: PlayerId,
        (town, most): (TownId, u8),
        (index, visitor): (usize, Option<usize>),
    ) -> [bool; 3] {
        let Some(held) = self.town_hero(town) else {
            let enter = visitor.is_some_and(|at| {
                let params = part.params();
                let rules = (params.army_slots(), params.max_stack_count());
                let garrison = self.town_garrison(index).unwrap_or_default();
                absorb(rules, &mut self.heroes()[at].army.clone(), garrison).is_ok()
            });
            return [enter, false, false];
        };
        let own = held.owner == player;
        let leave = own && visitor.is_none() && self.check_leave(player, (town, most)).is_ok();
        [false, leave, own && visitor.is_some()]
    }

    /// Makes `player`'s hero move `change` at `town` (TWN-ROW-002 to -004; the town epic's design
    /// note §53), `most` the most heroes a player has on the map (the town part's), each checked
    /// whole first:
    ///
    /// - [`TownHeroMove::Enter`] (TWN-ROW-002; the original's `town_visitor_to_garrison` 0x4B9FC0
    ///   with `army_absorb` 0x44B620): the player's visitor becomes the town's garrison hero, the
    ///   town's garrison joining its army ([`absorb`], all or nothing). Refused, in this order: a
    ///   garrison hero there ([`TownHeroError::GarrisonHeld`]), no visitor of the player's
    ///   ([`TownHeroError::NoVisitor`]), the armies' refusals. Its record leaves the map into the
    ///   town with its army, points, day's maximum, formation and flag (its marks stay where they
    ///   are, by hero id), the garrison is written empty, and the viewers of its tile are told
    ///   ([`AdventureEventKind::GarrisonEntered`], as a hero's leaving the map).
    /// - [`TownHeroMove::Leave`] (TWN-ROW-003; `town_hero_out_click` 0x5D5220,
    ///   `town_garrison_to_visitor` 0x5BE390): the player's garrison hero comes out onto the
    ///   entrance as the town's visitor, its record as it was kept (its points included), on land.
    ///   Refused, in this order: no garrison hero of the player's
    ///   ([`TownHeroError::NoGarrisonHero`]), a hero on the entrance
    ///   ([`TownHeroError::EntranceHeld`]), `most` heroes of the player's on the map already
    ///   ([`TownHeroError::TooManyHeroes`]), an army without a stack ([`TownHeroError::EmptyArmy`]:
    ///   T-5's case only). Told to its owner and the viewers of the tile
    ///   ([`AdventureEventKind::GarrisonLeft`], as a hero's entering the map).
    /// - [`TownHeroMove::Swap`] (TWN-ROW-004; `town_swap_heroes` 0x5BE450): the player's visitor
    ///   and garrison hero change places, each with its army and record. Refused, in this order:
    ///   no garrison hero of the player's ([`TownHeroError::NoGarrisonHero`]), no visitor of the
    ///   player's ([`TownHeroError::NoVisitor`]; an ally's moves no hero, TD-17 by the reading).
    ///   Told as the visitor's entering, then the garrison hero's leaving.
    ///
    /// Before any of them, a town the map does not have is [`TownHeroError::UnknownTown`]. Returns
    /// the army slots that changed, `Enter`'s: the hero's ([`ArmyRef::Hero`]) then the garrison's
    /// ([`ArmyRef::Garrison`]), each in slot order, which the game tells the owner.
    ///
    /// # Errors
    ///
    /// A [`TownHeroError`]; nothing changed.
    // rule: TWN-ROW-002, TWN-ROW-003, TWN-ROW-004
    pub fn move_town_hero<S: Sink>(
        &mut self,
        (core, part): (&Ruleset, &AdventureRuleset),
        player: PlayerId,
        (town, most): (TownId, u8),
        change: TownHeroMove,
        sink: &mut S,
    ) -> Result<Vec<ArmySlot>, TownHeroError> {
        match change {
            TownHeroMove::Enter => {
                let entering = self.check_enter(part, player, town)?;
                Ok(self.enter(core, part, town, entering, sink))
            }
            TownHeroMove::Leave => {
                let entrance = self.check_leave(player, (town, most))?;
                self.leave(core, player, (town, entrance), sink);
                Ok(Vec::new())
            }
            TownHeroMove::Swap => {
                let (at, entrance) = self.check_swap(player, town)?;
                self.swap(core, player, (town, entrance), at, sink);
                Ok(Vec::new())
            }
        }
    }

    /// `Enter`'s checks: the town's object index, the visitor's place and its army after the
    /// absorb.
    fn check_enter(
        &self,
        part: &AdventureRuleset,
        player: PlayerId,
        town: TownId,
    ) -> Result<Entering, TownHeroError> {
        let (index, entrance) = self.town_entrance(town)?;
        if self.town_hero(town).is_some() {
            return Err(TownHeroError::GarrisonHeld);
        }
        let at = self.own_visitor(player, entrance)?;
        let params = part.params();
        let rules = (params.army_slots(), params.max_stack_count());
        let garrison = self.town_garrison(index).unwrap_or_default();
        let mut army = self.heroes()[at].army.clone();
        absorb(rules, &mut army, garrison)?;
        Ok(Entering { index, at, army })
    }

    /// `Enter`, checked ([`Self::check_enter`]).
    fn enter<S: Sink>(
        &mut self,
        core: &Ruleset,
        part: &AdventureRuleset,
        town: TownId,
        Entering { index, at, army }: Entering,
        sink: &mut S,
    ) -> Vec<ArmySlot> {
        let slots_of = part.params().army_slots();
        let garrison = self.town_garrison(index).unwrap_or_default().to_vec();
        let mut slots = Vec::new();
        let hero = &self.heroes()[at];
        let (id, owner, pos) = (hero.id, hero.owner, hero.pos);
        changed(&mut slots, ArmyRef::Hero(id), slots_of, (&hero.army, &army));
        changed(
            &mut slots,
            ArmyRef::Garrison(town),
            slots_of,
            (&garrison, &[]),
        );
        let seen_by = self.viewers(core, owner, pos);
        if !garrison.is_empty() {
            self.set_army_left(index, Vec::new());
        }
        let (heroes, block) = self.rows_mut();
        let mut hero = heroes.remove(at);
        hero.army = army;
        place(block, town, hero);
        sink.emit(AdventureEvent {
            kind: AdventureEventKind::GarrisonEntered {
                hero: id,
                town,
                pos,
            },
            seen_by,
        });
        slots
    }

    /// `Leave`'s checks: the entrance.
    fn check_leave(
        &self,
        player: PlayerId,
        (town, most): (TownId, u8),
    ) -> Result<Pos, TownHeroError> {
        let (_, entrance) = self.town_entrance(town)?;
        let hero = self
            .town_hero(town)
            .filter(|hero| hero.owner == player)
            .ok_or(TownHeroError::NoGarrisonHero)?;
        if self.heroes().iter().any(|on| on.pos == entrance) {
            return Err(TownHeroError::EntranceHeld);
        }
        let on_map = self.heroes().iter().filter(|on| on.owner == player).count();
        if on_map >= usize::from(most) {
            return Err(TownHeroError::TooManyHeroes);
        }
        if hero.army.is_empty() {
            return Err(TownHeroError::EmptyArmy);
        }
        Ok(entrance)
    }

    /// `Leave`, checked ([`Self::check_leave`]).
    fn leave<S: Sink>(
        &mut self,
        core: &Ruleset,
        player: PlayerId,
        (town, entrance): (TownId, Pos),
        sink: &mut S,
    ) {
        let seen_by = self.viewers(core, player, entrance);
        let (heroes, block) = self.rows_mut();
        let hero = take(block, town);
        let id = hero.id;
        arrive(heroes, hero, entrance);
        sink.emit(AdventureEvent {
            kind: AdventureEventKind::GarrisonLeft {
                hero: id,
                town,
                pos: entrance,
            },
            seen_by,
        });
    }

    /// `Swap`'s checks: the visitor's place and the entrance.
    fn check_swap(&self, player: PlayerId, town: TownId) -> Result<(usize, Pos), TownHeroError> {
        let (_, entrance) = self.town_entrance(town)?;
        self.town_hero(town)
            .filter(|hero| hero.owner == player)
            .ok_or(TownHeroError::NoGarrisonHero)?;
        let at = self.own_visitor(player, entrance)?;
        Ok((at, entrance))
    }

    /// `Swap`, checked ([`Self::check_swap`]).
    fn swap<S: Sink>(
        &mut self,
        core: &Ruleset,
        player: PlayerId,
        (town, entrance): (TownId, Pos),
        at: usize,
        sink: &mut S,
    ) {
        let seen_by = self.viewers(core, player, entrance);
        let (heroes, block) = self.rows_mut();
        let entering = heroes.remove(at);
        let leaving = take(block, town);
        let (visitor, held) = (entering.id, leaving.id);
        place(block, town, entering);
        arrive(heroes, leaving, entrance);
        for kind in [
            AdventureEventKind::GarrisonEntered {
                hero: visitor,
                town,
                pos: entrance,
            },
            AdventureEventKind::GarrisonLeft {
                hero: held,
                town,
                pos: entrance,
            },
        ] {
            sink.emit(AdventureEvent { kind, seen_by });
        }
    }

    /// Takes `town`'s garrison hero out of the game at its town's capture (`TWN-STUB-14`, E09
    /// T28a: until T28b's rows 1 and 3 a garrison hero does not defend; its town falls to the
    /// first visitor of another team as an undefended one, the garrison hero back to the pool
    /// with its artifacts, as T-5's hero without an army, `TWN-QUIRK-11`): its record and marks
    /// gone, told to its owner alone ([`AdventureEventKind::HeroRemoved`] at the entrance: no
    /// other viewer sees it). Returns the hero, whose type the game frees; none without one.
    // stub: TWN-STUB-14 (a garrison hero's defence: E09 T28b)
    pub fn release_town_hero<S: Sink>(&mut self, town: TownId, sink: &mut S) -> Option<HeroId> {
        let entrance = self.town_entrance(town).ok()?.1;
        let owner = self.town_hero(town)?.owner;
        let (_, block) = self.rows_mut();
        let hero = take(block, town);
        self.leave_marks(hero.id);
        sink.emit(AdventureEvent {
            kind: AdventureEventKind::HeroRemoved {
                hero: hero.id,
                pos: entrance,
            },
            seen_by: Viewers::NONE.with(owner),
        });
        Some(hero.id)
    }

    /// `town`'s object index and entrance, if the map has the town and it is visited from a tile
    /// (every town's is: the setup refuses one without).
    fn town_entrance(&self, town: TownId) -> Result<(usize, Pos), TownHeroError> {
        let map = self.map();
        map.town_object(town)
            .zip(map.town_entrance(town))
            .ok_or(TownHeroError::UnknownTown)
    }

    /// The place among the heroes on the map of `player`'s hero on `entrance`, if one stands
    /// there ([`TownHeroError::NoVisitor`] else; another player's included, TD-17).
    fn own_visitor(&self, player: PlayerId, entrance: Pos) -> Result<usize, TownHeroError> {
        self.heroes()
            .iter()
            .position(|on| on.pos == entrance && on.owner == player)
            .ok_or(TownHeroError::NoVisitor)
    }
}

/// Puts `hero` into `town`'s garrison in `block`, in town order (none stands there: checked).
fn place(block: &mut Option<Arc<TownHeroes>>, town: TownId, hero: HeroOnMap) {
    let all = Arc::make_mut(block.get_or_insert_with(Arc::default));
    let at = all.partition_point(|&(id, _)| id < town);
    all.insert(at, (town, hero));
}

/// Takes `town`'s garrison hero out of `block` (one stands there: checked), the block dropped
/// when it empties.
fn take(block: &mut Option<Arc<TownHeroes>>, town: TownId) -> HeroOnMap {
    let mut all = block.take().unwrap_or_default();
    let list = Arc::make_mut(&mut all);
    let at = list
        .binary_search_by_key(&town, |&(id, _)| id)
        .unwrap_or_default();
    let (_, hero) = list.remove(at);
    if !list.is_empty() {
        *block = Some(all);
    }
    hero
}

/// Puts `hero` onto the map at `entrance`, on land, among `heroes` in id order.
fn arrive(heroes: &mut Vec<HeroOnMap>, mut hero: HeroOnMap, entrance: Pos) {
    hero.pos = entrance;
    hero.boat = None;
    let at = heroes.partition_point(|on| on.id < hero.id);
    heroes.insert(at, hero);
}

/// Adds to `slots` each slot of `army`, below `count`, whose stack differs between `before` and
/// `after`, in slot order.
fn changed(
    slots: &mut Vec<ArmySlot>,
    army: ArmyRef,
    count: u8,
    (before, after): (&[ArmyStack], &[ArmyStack]),
) {
    let at = |stacks: &[ArmyStack], slot| stacks.iter().copied().find(|s| s.slot == slot);
    slots.extend(
        (0..count)
            .filter(|&slot| at(before, slot) != at(after, slot))
            .map(|slot| ArmySlot { army, slot }),
    );
}

/// The garrison's stacks joining a visitor's army as it enters the garrison (TWN-ROW-002; survey
/// `E09-growth.md` §9, the original's `army_absorb` 0x44B620, read whole): each garrison stack in
/// slot order joins the army's first stack of its creature, else the army's first empty slot,
/// else the army's own first two stacks of one creature are merged, the later into the earlier,
/// and the slot tried again. `rules`: the army slots and a stack's most creatures (ADV-ARM-007).
/// All or nothing: `army` is changed only on success. The original's test is VCMI's
/// (`canBeMergedWith`): the creatures of both armies, each counted once, fit the slots; the steps
/// give the same.
///
/// # Errors
///
/// [`TownHeroError::ArmiesDoNotFit`] when no slot frees, [`TownHeroError::TooManyCreatures`]
/// when a join or a merge would pass a stack's most (unreachable with the original's counts).
// rule: TWN-ROW-002
pub(crate) fn absorb(
    (slots, most): (u8, u32),
    army: &mut Vec<ArmyStack>,
    garrison: &[ArmyStack],
) -> Result<(), TownHeroError> {
    // An army's slots are unique (ADR-0003 §9).
    let by_slot = |stacks: &mut [ArmyStack]| {
        let _ = order::sort_by_unique_key(stacks, |s| s.slot);
    };
    let mut joined = army.clone();
    let mut garrison = garrison.to_vec();
    by_slot(&mut garrison);
    for stack in &garrison {
        by_slot(&mut joined);
        if let Some(own) = joined.iter_mut().find(|s| s.creature == stack.creature) {
            own.count = sum(own.count, stack.count, most)?;
            continue;
        }
        loop {
            if let Some(free) = (0..slots).find(|&slot| joined.iter().all(|s| s.slot != slot)) {
                joined.push(ArmyStack {
                    slot: free,
                    ..*stack
                });
                break;
            }
            let pair = (0..joined.len()).find_map(|first| {
                (first + 1..joined.len())
                    .find(|&later| joined[later].creature == joined[first].creature)
                    .map(|later| (first, later))
            });
            let (first, later) = pair.ok_or(TownHeroError::ArmiesDoNotFit)?;
            joined[first].count = sum(joined[first].count, joined[later].count, most)?;
            joined.remove(later);
        }
    }
    by_slot(&mut joined);
    *army = joined;
    Ok(())
}

/// `a + b` creatures, at most `most` ([`TownHeroError::TooManyCreatures`] else).
fn sum(a: u32, b: u32, most: u32) -> Result<u32, TownHeroError> {
    a.checked_add(b)
        .filter(|&n| n <= most)
        .ok_or(TownHeroError::TooManyCreatures)
}

#[cfg(test)]
mod tests;
