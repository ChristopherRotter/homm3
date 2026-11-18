//! The hero pool (TWN-SET-005; E09 T7b1, design note `E09-design.md` §14, ADR-0017's E09
//! paragraph) and the build's draws of the heroes: each player's random faction choice
//! (TWN-SET-001), then each random hero's type (TWN-SET-008: its owner's pick, else the draw,
//! TWN-SET-007), from the setup stream before every other draw of the build (the random towns'
//! factions next, [`super::build`], then the main towns' heroes, [`super::heroes`]).

use alloc::collections::BTreeSet;
use alloc::vec;
use alloc::vec::Vec;

use h3_content::{FactionId, Formation, HeroClassId, HeroType, HeroTypeId, Ruleset};
use h3_contracts::{ArmyStack, HeroId, PlayerId};
use h3_kernel::{CanonicalHash, Rng, StateHasher};

use super::{FactionChoice, needs_faction};
use crate::{GameRuleset, GameSetup, GameSetupError};

/// Where a hero type stands in the pool (TWN-SET-005).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HeroStatus {
    /// Free: a draw may give it.
    Free,
    /// Taken by the player of this colour: placed, drawn or reserved as its pick.
    Taken(u8),
    /// A tavern hero (E09 T37a, design note §61): a player's slot holds it, so no draw gives it
    /// (the original's 0x40, which also marks a type the map does not allow: kept apart, `Out`).
    Offered,
    /// Taken as a prisoner's.
    Prisoner,
    /// Out of the game: the map does not allow it.
    Out,
}

/// What the pool keeps of a recorded hero type off the map (the away record; E09 T37a, design
/// note §61, ADR-0016 point 3): what the adventure keeps of a hero on the map, its points left
/// and the day's maximum (set at its owner's turn start, TWN-TAV-005), its formation and tactics
/// flag and its army (every fill replaces it, TWN-TAV-002).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Away {
    /// The movement points it has left today.
    pub(crate) movement: u32,
    /// The day's maximum.
    pub(crate) max_movement: u32,
    /// Its army's formation in battle.
    pub(crate) formation: Formation,
    /// Its tactics flag.
    pub(crate) tactics: bool,
    /// Its army, by slot.
    pub(crate) army: Vec<ArmyStack>,
    /// Whether its points need its owner's next turn start (TWN-TAV-005): its army or its record
    /// changed since they were set (a fill; from E09 T38b a return). The turn start derives only
    /// these: nothing else changes what the derivation reads while the type is off the map (E09
    /// T37a, `bench-check`: a derivation at every turn start cost `day-start-builds-36` +11 %).
    pub(crate) stale: bool,
    /// Whether its record's spell points are at their maximum, its record unchanged since a fill
    /// set them so (from E09 T38b also a leave without return): the day's regain leaves such a
    /// record as it is (`h3_heroes::day_mana` never changes points at the maximum), so it skips
    /// it (E09 T37a, `bench-check`: the regain over every tavern record cost `day-start-builds-36`
    /// +55 %). A write of the record clears it.
    pub(crate) full: bool,
}

impl Away {
    /// A new record's (the agent's call, design note §61): no points until its owner's turn
    /// starts, the loose formation and the tactics flag on (as a hero placed on a map), `army`.
    pub(crate) const fn new(army: Vec<ArmyStack>) -> Self {
        Self {
            movement: 0,
            max_movement: 0,
            formation: Formation::Loose,
            tactics: true,
            army,
            stale: true,
            full: false,
        }
    }
}

/// A player's tavern part (E09 T37a, design note §61; TWN-SET-001 amended, TWN-TAV-002): its
/// faction (TWN-SET-001's resolved choice, kept; none without a choice) and its two tavern heroes,
/// each slot a hero type whose record the pool names (the agent's call: the slot names the type,
/// its entry the record), with its returning mark (the original's bit 17 of the hero's +0x105).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct Tavern {
    /// The player's faction.
    pub(crate) faction: Option<FactionId>,
    /// Its two tavern heroes' types by slot.
    pub(crate) slots: [Option<HeroTypeId>; 2],
    /// Each slot's returning mark (TD-18; set by a hero's return, E09 T38b).
    pub(crate) returning: [bool; 2],
}

/// A hero type's entry in the pool: what a draw reads (its [`Record`] beside it).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Entry {
    /// Where it stands.
    status: HeroStatus,
    /// The colours that may take it (bit `c`: colour `c`).
    mask: u8,
    /// Its hero class (the core's; the draw's counts read it without the hero types).
    class: HeroClassId,
}

/// A hero type's record in the pool (E09 T37a, design note §61), beside its [`Entry`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct Record {
    /// Its record in the store of heroes once it has one (decision 109): a placed or generated
    /// hero's id from the build, a drawn type's from its first draw; kept from then on.
    hero: Option<HeroId>,
    /// Its away record while it is a tavern hero.
    away: Option<Away>,
}

/// The hero pool (TWN-SET-005; ADR-0017's E09 paragraph): per hero type of the core ruleset, by
/// its index, where it stands, its colour mask (bit `c`: colour `c` may take it; 0xFF unless
/// the setup's masks restrict it) and since E09 T37a its record and away record; per player its
/// tavern part. Built with the game ([`HeroPool::new`]), changed by the build's draws, the
/// taverns' fills and when a hero leaves the map ([`HeroPool::free`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HeroPool {
    types: Vec<Entry>,
    records: Vec<Record>,
    players: Vec<Tavern>,
}

impl HeroPool {
    /// The pool before any draw (TWN-SET-005; `hero_pool_init` 0x4C4E30, survey `E09-towns.md`
    /// §4.2): each type the setup allows free, the rest out; each type's mask the AND of the
    /// setup's masks of it (0xFF without one). Then, whatever the allowed set, every placed hero's
    /// type taken by its owner's colour (an owner that is no player is the adventure context's
    /// check: skipped), every prisoner's taken as a prisoner, and last each player's pick
    /// reserved, taken by its colour (T-2, T-3 fixed: `TWN-QUIRK-2`, `TWN-QUIRK-3`). A type the
    /// ruleset does not have is skipped. No type has a record yet, every player an empty tavern.
    // rule: TWN-SET-005
    pub(crate) fn new(core: &Ruleset, setup: &GameSetup) -> Self {
        let entry = |ty: &HeroType| Entry {
            status: HeroStatus::Out,
            mask: 0xFF,
            class: ty.class,
        };
        let mut pool = Self {
            types: core.hero_types().iter().map(entry).collect(),
            records: vec![Record::default(); core.hero_types().len()],
            players: vec![Tavern::default(); setup.players.len()],
        };
        for &id in &setup.allowed_heroes {
            pool.set(id, HeroStatus::Free);
        }
        for &(id, mask) in &setup.hero_masks {
            if let Some(entry) = pool.types.get_mut(id.index()) {
                entry.mask &= mask;
            }
        }
        // The placed heroes' and prisoners' types before any draw (T-3 fixed, `TWN-QUIRK-3`).
        for hero in &setup.heroes {
            let owner = setup.players.get(hero.owner.index());
            if let (Some(id), Some(owner)) = (hero.hero_type, owner) {
                pool.set(id, HeroStatus::Taken(owner.colour));
            }
        }
        for &id in &setup.prisoners {
            pool.set(id, HeroStatus::Prisoner);
        }
        // The picks reserved before any draw (T-2 fixed, `TWN-QUIRK-2`).
        for player in &setup.players {
            if let Some(pick) = player.start.pick {
                pool.set(pick, HeroStatus::Taken(player.colour));
            }
        }
        pool
    }

    /// Where `id` stands; `None` for a type the ruleset does not have.
    #[cfg(test)]
    pub(crate) fn status(&self, id: HeroTypeId) -> Option<HeroStatus> {
        self.types.get(id.index()).map(|entry| entry.status)
    }

    /// Whether `id` is free (no hero on the map, in a garrison or a tavern has it, the map allows
    /// it and no prison holds it; E09 T37b).
    pub(crate) fn is_free(&self, id: HeroTypeId) -> bool {
        self.types
            .get(id.index())
            .is_some_and(|entry| entry.status == HeroStatus::Free)
    }

    /// Whether `id` is a tavern hero (a player's slot holds it; E09 T38b1).
    pub(crate) fn is_offered(&self, id: HeroTypeId) -> bool {
        self.types
            .get(id.index())
            .is_some_and(|entry| entry.status == HeroStatus::Offered)
    }

    /// `id`'s colour mask; `None` for a type the ruleset does not have.
    #[cfg(test)]
    pub(crate) fn mask(&self, id: HeroTypeId) -> Option<u8> {
        self.types.get(id.index()).map(|entry| entry.mask)
    }

    /// Sets where `id` stands (a type the ruleset does not have: nothing).
    fn set(&mut self, id: HeroTypeId, status: HeroStatus) {
        if let Some(entry) = self.types.get_mut(id.index()) {
            entry.status = status;
        }
    }

    /// `id` goes back free (TWN-SET-005: an unused pick after the main towns' heroes,
    /// [`super::main_town_heroes`]; since E09 T38b1 a hero that leaves the map is the leave's,
    /// [`Self::leave`]). Its record and away record stay.
    pub(crate) fn free(&mut self, id: HeroTypeId) {
        self.set(id, HeroStatus::Free);
    }

    /// `id` is taken by the player of `colour` (a drawn or picked hero's type, TWN-SET-007).
    pub(super) fn take(&mut self, id: HeroTypeId, colour: u8) {
        self.set(id, HeroStatus::Taken(colour));
    }

    /// `id` is a tavern hero (TWN-TAV-002).
    pub(super) fn offer(&mut self, id: HeroTypeId) {
        self.set(id, HeroStatus::Offered);
    }

    /// The hire of `hero_type`, `player`'s tavern hero in `slot` (TWN-TAV-003; `hero_place`
    /// 0x4D7900 with its flag, design note §64; E09 T38a1): the slot emptied and its returning
    /// mark cleared, the type taken by `colour` with the colour's bit added to its mask, and its
    /// away record handed over (the adventure keeps a hero on the map's: the turn's derivation and
    /// the day's regain read only the away records of heroes off the map). The caller checked the
    /// slot's type; a fill gives every offered type its away record, so none is a new record's.
    // rule: TWN-TAV-003
    pub(super) fn hire(
        &mut self,
        (player, slot): (PlayerId, usize),
        hero_type: HeroTypeId,
        colour: u8,
    ) -> Away {
        let tavern = &mut self.players[player.index()];
        tavern.slots[slot] = None;
        tavern.returning[slot] = false;
        let entry = &mut self.types[hero_type.index()];
        entry.status = HeroStatus::Taken(colour);
        entry.mask |= 1_u8.checked_shl(u32::from(colour)).unwrap_or(0);
        let away = self.records[hero_type.index()].away.take();
        away.unwrap_or(Away::new(Vec::new()))
    }

    /// `id`, a hero that left the game without returning (TWN-TAV-006; E09 T38b1, design note
    /// §67), goes back free with `away` as its away record (its record stays).
    // rule: TWN-TAV-006
    pub(super) fn leave(&mut self, id: HeroTypeId, away: Away) {
        // A hero's type is the ruleset's.
        self.types[id.index()].status = HeroStatus::Free;
        self.records[id.index()].away = Some(away);
    }

    /// `id`, a hero that left the game after its side's retreat or surrender (TWN-TAV-006; E09
    /// T38b1, design note §67), returns into `player`'s tavern in `slot` (0 or 1): the slot's hero,
    /// if any, goes free (its record and away record kept); the type offered and held by the slot,
    /// its returning mark set (TD-18), `away` its away record. A hero's owner is a player of the
    /// game and its type the ruleset's.
    // rule: TWN-TAV-006
    pub(super) fn return_to(
        &mut self,
        (player, slot): (PlayerId, usize),
        id: HeroTypeId,
        away: Away,
    ) {
        let tavern = &mut self.players[player.index()];
        let freed = tavern.slots[slot].replace(id);
        tavern.returning[slot] = true;
        if let Some(freed) = freed {
            self.set(freed, HeroStatus::Free);
        }
        self.types[id.index()].status = HeroStatus::Offered;
        self.records[id.index()].away = Some(away);
    }

    /// The build's records (E09 T37a, design note §61): each hero's type in hero order names its
    /// hero's id (`types`: the setup's heroes, then the main towns'), the first hero of a type
    /// where a setup repeats one.
    pub(crate) fn record_heroes(&mut self, types: &[(HeroTypeId, &HeroType)]) {
        for (&(id, _), hero) in types.iter().zip((0..=u16::MAX).map(HeroId::new)) {
            let record = self.records.get_mut(id.index());
            record.into_iter().for_each(|record| {
                record.hero.get_or_insert(hero);
            });
        }
    }

    /// Each player's faction, by player id (the build's, TWN-SET-001: `None` without a choice or
    /// for a `Random` one its start does not read, drawn at its first fill, TWN-TAV-002).
    pub(crate) fn set_factions(&mut self, factions: &[Option<FactionId>]) {
        for (tavern, &faction) in self.players.iter_mut().zip(factions) {
            tavern.faction = faction;
        }
    }

    /// `id`'s record, if it has one.
    pub(crate) fn record(&self, id: HeroTypeId) -> Option<HeroId> {
        self.records.get(id.index()).and_then(|record| record.hero)
    }

    /// `id`'s record becomes `hero` (a first draw's, design note §61; since E09 T37b a customised
    /// type's at the build).
    pub(crate) fn set_record(&mut self, id: HeroTypeId, hero: HeroId) {
        let record = self.records.get_mut(id.index());
        record
            .into_iter()
            .for_each(|record| record.hero = Some(hero));
    }

    /// `id`'s away record, if it has one.
    pub(crate) fn away(&self, id: HeroTypeId) -> Option<&Away> {
        self.records
            .get(id.index())
            .and_then(|record| record.away.as_ref())
    }

    /// `id`'s away record to change, if it has one.
    pub(crate) fn away_mut(&mut self, id: HeroTypeId) -> Option<&mut Away> {
        self.records
            .get_mut(id.index())
            .and_then(|record| record.away.as_mut())
    }

    /// `id`'s army becomes `army` at a fill (TWN-TAV-002): its away record's, a new one
    /// ([`Away::new`]) for a type without; its points stale, its record `full` as the fill left
    /// its spell points ([`Away::full`]).
    pub(super) fn set_army(&mut self, id: HeroTypeId, army: Vec<ArmyStack>, full: bool) {
        // A drawn type is the ruleset's.
        let record = &mut self.records[id.index()];
        let away = record.away.get_or_insert_with(|| Away::new(Vec::new()));
        away.army = army;
        away.stale = true;
        away.full = full;
    }

    /// The week's release (TWN-TAV-002; `new_day_week_events` 0x4C8780, survey `E09-tavern.md`
    /// §2.3; design note §63): per player in player order, slot 0 then slot 1, a slot holding a
    /// hero whose returning mark is clear lets it go: its type free (its record and away record
    /// kept), the slot empty. A returning hero stays (TD-18).
    // rule: TWN-TAV-002
    pub(super) fn release(&mut self) {
        for tavern in &mut self.players {
            for (slot, &returning) in tavern.slots.iter_mut().zip(&tavern.returning) {
                let released = slot.filter(|_| !returning);
                if let Some(id) = released {
                    let entry = self.types.get_mut(id.index());
                    entry.into_iter().for_each(|e| e.status = HeroStatus::Free);
                    *slot = None;
                }
            }
        }
    }

    /// Whether a slot holds a hero whose returning mark is clear ([`Self::release`] frees it).
    pub(super) fn releases(&self) -> bool {
        let held = |t: &Tavern| {
            t.slots
                .iter()
                .zip(t.returning)
                .any(|(s, r)| s.is_some() && !r)
        };
        self.players.iter().any(held)
    }

    /// The records whose spell points are at their maximum, unchanged since ([`Away::full`]).
    pub(crate) fn full_records(&self) -> BTreeSet<HeroId> {
        self.records
            .iter()
            .filter(|record| record.away.as_ref().is_some_and(|away| away.full))
            .filter_map(|record| record.hero)
            .collect()
    }

    /// Per core hero class by index, how many types of the class are free and may be taken by
    /// `colour` (the tavern's draw, TWN-TAV-001).
    pub(super) fn class_counts(&self, classes: usize, colour: u8) -> Vec<u32> {
        let mut counts = vec![0_u32; classes];
        for entry in self.types.iter().filter(|e| e.takeable(colour)) {
            let count = counts.get_mut(entry.class.index());
            count.into_iter().for_each(|count| *count += 1);
        }
        counts
    }

    /// The `n`-th (from 0) type of `class` that is free and may be taken by `colour`, in type
    /// order.
    pub(super) fn nth_free(&self, class: usize, colour: u8, n: usize) -> Option<HeroTypeId> {
        self.types
            .iter()
            .zip(0..)
            .filter(|(entry, _)| entry.class.index() == class && entry.takeable(colour))
            .nth(n)
            .and_then(|(_, index)| HeroTypeId::from_index(index))
    }

    /// The players' tavern parts, by player id.
    pub(crate) fn taverns(&self) -> &[Tavern] {
        &self.players
    }

    /// `player`'s tavern part, if the player exists.
    pub(crate) fn tavern(&self, player: PlayerId) -> Option<&Tavern> {
        self.players.get(player.index())
    }

    /// `player`'s tavern part to change, if the player exists.
    pub(crate) fn tavern_mut(&mut self, player: PlayerId) -> Option<&mut Tavern> {
        self.players.get_mut(player.index())
    }

    /// The types `colour` may take that are free, in type order, with the ruleset's records.
    pub(super) fn free_for<'r>(
        &self,
        core: &'r Ruleset,
        colour: u8,
    ) -> impl Iterator<Item = (HeroTypeId, &'r HeroType)> + use<'_, 'r> {
        core.hero_types()
            .iter()
            .zip(&self.types)
            .zip(0..)
            .filter(move |&((_, entry), _)| entry.takeable(colour))
            .filter_map(|((ty, _), index)| HeroTypeId::from_index(index).map(|id| (id, ty)))
    }

    /// The draw (TWN-SET-007; `hero_draw_of_faction` 0x4BB400, survey `E09-towns.md` §4.4): one
    /// of the free types that `colour` may take, uniform, among those of `faction`'s hero classes
    /// (the core's classes of the faction) when there is one, else among them all; drawn from
    /// `draws` only when there is one. The type is not taken here.
    // rule: TWN-SET-007
    pub(super) fn draw<'r>(
        &self,
        core: &'r Ruleset,
        faction: Option<FactionId>,
        colour: u8,
        draws: &mut Rng,
    ) -> Option<(HeroTypeId, &'r HeroType)> {
        let of_faction: Vec<_> = self
            .free_for(core, colour)
            .filter(|(_, ty)| {
                core.hero_class(ty.class)
                    .is_some_and(|class| Some(class.faction) == faction)
            })
            .collect();
        let listed = if of_faction.is_empty() {
            self.free_for(core, colour).collect()
        } else {
            of_faction
        };
        // `below(0)` draws nothing and gives 0, past an empty list.
        let count = u32::try_from(listed.len()).unwrap_or(u32::MAX);
        let drawn = usize::try_from(draws.below(count)).unwrap_or(usize::MAX);
        listed.get(drawn).copied()
    }
}

impl Entry {
    /// Whether the type is free and `colour` may take it (its mask has the colour's bit).
    fn takeable(&self, colour: u8) -> bool {
        self.status == HeroStatus::Free
            && self
                .mask
                .checked_shr(u32::from(colour))
                .is_some_and(|m| m & 1 == 1)
    }
}

impl CanonicalHash for HeroPool {
    /// The game state hash's encoding (v23, E09 T7b1; v33, E09 T37a): the number of types, then
    /// each type in index order: where it stands (0 free, 1 taken and its colour, 2 a prisoner's,
    /// 3 out, since v33 4 offered) and its colour mask, since v33 its record (0 none, 1 and its id)
    /// and its away record (0 none, 1 and its points left, the day's maximum, its formation, 1
    /// tight, its tactics flag, whether its points are stale and its record full, its army's
    /// length and each stack's slot, creature and count);
    /// since v33 the number of players, then each player's faction (0 none, 1 and its id), its two
    /// slots (0 empty, 1 and the type) and their returning marks.
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_len(self.types.len());
        for (entry, record) in self.types.iter().zip(&self.records) {
            match entry.status {
                HeroStatus::Free => h.write_u8(0),
                HeroStatus::Taken(colour) => {
                    h.write_u8(1);
                    h.write_u8(colour);
                }
                HeroStatus::Prisoner => h.write_u8(2),
                HeroStatus::Out => h.write_u8(3),
                HeroStatus::Offered => h.write_u8(4),
            }
            h.write_u8(entry.mask);
            match record.hero {
                None => h.write_u8(0),
                Some(hero) => {
                    h.write_u8(1);
                    h.write_u16(hero.raw());
                }
            }
            match &record.away {
                None => h.write_u8(0),
                Some(away) => {
                    h.write_u8(1);
                    h.write_u32(away.movement);
                    h.write_u32(away.max_movement);
                    h.write_bool(away.formation == Formation::Tight);
                    h.write_bool(away.tactics);
                    h.write_bool(away.stale);
                    h.write_bool(away.full);
                    h.write_len(away.army.len());
                    for stack in &away.army {
                        h.write_u8(stack.slot);
                        stack.creature.hash_into(h);
                        h.write_u32(stack.count);
                    }
                }
            }
        }
        h.write_len(self.players.len());
        for tavern in &self.players {
            match tavern.faction {
                None => h.write_u8(0),
                Some(faction) => {
                    h.write_u8(1);
                    faction.hash_into(h);
                }
            }
            for (slot, &returning) in tavern.slots.iter().zip(&tavern.returning) {
                match slot {
                    None => h.write_u8(0),
                    Some(hero_type) => {
                        h.write_u8(1);
                        hero_type.hash_into(h);
                    }
                }
                h.write_bool(returning);
            }
        }
    }
}

/// Each hero's type in hero order, with the ruleset's record of it.
pub(crate) type HeroTypes<'r> = Vec<(HeroTypeId, &'r HeroType)>;

/// What the heroes' draws give the rest of the build ([`draw_heroes`], then
/// [`super::main_town_heroes`]).
pub(crate) struct Drawn<'r> {
    /// The pool.
    pub(crate) pool: HeroPool,
    /// Each hero's type in hero order: the setup's heroes, then the main towns' heroes.
    pub(crate) types: HeroTypes<'r>,
    /// Each player's resolved faction by id (`None` without a choice, or a `Random` one its start
    /// does not read).
    pub(crate) factions: Vec<Option<FactionId>>,
    /// Each player's pick by id that no hero has taken yet (still taken in the pool by its
    /// colour until the main towns' heroes free it).
    pub(crate) picks: Vec<Option<HeroTypeId>>,
}

/// The pool, each hero's type in hero order and each player's resolved faction by id (`None`
/// without a choice, or a `Random` one its start does not read), the build's first draws from
/// `draws` (design note §14, E09 T7b1): every placed hero's type checked first (before any draw),
/// the pool built ([`HeroPool::new`]); then each player's `Random` faction choice in player order,
/// uniform over its factions (TWN-SET-001; the resolved factions are not kept in the game, the
/// build's random towns read them, [`super::draw_towns`]), only when its start reads it
/// ([`needs_faction`] with the towns' owners at the start, `owners` by `TownId`: a choice nothing
/// reads draws nothing, the agent's call, so a map whose players allow several factions but have
/// no towns and no random hero, as the duel's, keeps its stream); then each random hero in hero
/// order (TWN-SET-008): its owner's pick for the owner's first random hero, else the draw over
/// the owner's faction (TWN-SET-007), the type taken by the owner's colour at once. Each pick no
/// random hero took is given back unused ([`Drawn::picks`], still taken: a human player's main
/// town's hero may take it, and the main towns' heroes free the rest, the agent's call, design
/// note §14). The setup's checks of the starts ran before ([`super::check_starts`]: a random
/// hero's owner is a player with a faction choice, a `Random` choice has factions, a pick is
/// offered and free).
///
/// # Errors
///
/// [`GameSetupError::UnknownHeroType`] for the first placed hero whose type the ruleset does not
/// have; [`GameSetupError::NoFreeHero`] for the owner of the first random hero the draw finds no
/// type for (`TWN-QUIRK-4`).
// rule: TWN-SET-001, TWN-SET-008
pub(crate) fn draw_heroes<'r>(
    setup: &GameSetup,
    rules: &'r GameRuleset,
    owners: &[Option<PlayerId>],
    draws: &mut Rng,
) -> Result<Drawn<'r>, GameSetupError> {
    let core = rules.core();
    let placed: Vec<Option<(HeroTypeId, &HeroType)>> = setup
        .heroes
        .iter()
        .zip((0..=u16::MAX).map(HeroId::new))
        .map(|(hero, id)| {
            hero.hero_type
                .map(|t| {
                    core.hero_type(t)
                        .map(|ty| (t, ty))
                        .ok_or(GameSetupError::UnknownHeroType(id))
                })
                .transpose()
        })
        .collect::<Result<_, _>>()?;
    let mut pool = HeroPool::new(core, setup);
    let factions: Vec<Option<FactionId>> = setup
        .players
        .iter()
        .zip((0..=u8::MAX).map(PlayerId::new))
        .map(|(player, id)| match player.start.choice {
            Some(FactionChoice::Fixed(faction)) => Some(faction),
            Some(FactionChoice::Random) if needs_faction(setup, owners, id, player) => {
                let set = player.start.factions;
                let drawn = usize::try_from(draws.below(set.len())).unwrap_or(usize::MAX);
                set.iter().nth(drawn)
            }
            _ => None,
        })
        .collect();
    let mut picks: Vec<Option<HeroTypeId>> = setup
        .players
        .iter()
        .map(|player| player.start.pick)
        .collect();
    let mut types = Vec::with_capacity(placed.len());
    for (hero, placed) in setup.heroes.iter().zip(placed) {
        let found = match placed {
            Some(found) => found,
            None => {
                let owner = hero.owner.index();
                let colour = setup.players[owner].colour;
                let pick = picks[owner].take();
                let picked = pick.and_then(|t| core.hero_types().get(t.index()).map(|ty| (t, ty)));
                let found = picked
                    .or_else(|| pool.draw(core, factions[owner], colour, draws))
                    .ok_or(GameSetupError::NoFreeHero(hero.owner))?;
                pool.take(found.0, colour);
                found
            }
        };
        types.push(found);
    }
    Ok(Drawn {
        pool,
        types,
        factions,
        picks,
    })
}
