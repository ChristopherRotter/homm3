//! The store of towns (ADR-0028 point 4; E09 increment 1's design note §4): every town's record
//! by `h3_contracts::TownId`, held by the game state beside the adventure's.

use alloc::sync::Arc;
use alloc::vec::Vec;

use h3_contracts::{FortLevel, TownId};
use h3_kernel::{CanonicalHash, StateHasher};

use crate::TOWN_STATE_HASH_VERSION;
use crate::build::built_with;
use crate::building::BuildingId;
use crate::capture::{capitol_limit, standing_capitol};
use crate::record::{Town, TownEvent, TownStatic};
use crate::ruleset::TownRuleset;
use crate::siege::fort_level;

/// The most towns a store holds: one per [`TownId`].
pub const MAX_TOWNS: usize = 1 << 16;

/// A town's public facts (E09 design §26, §31): what every viewer who sees the town learns of its
/// record, so that the game tells a change of them after a build.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TownFacts {
    /// Its fort level ([`crate::fort_level`]).
    pub fort: FortLevel,
    /// Whether it has a Capitol, a standing hall-4 building.
    pub capitol: bool,
    /// Whether it was built in today ([`Town::built_today`]).
    pub built_today: bool,
}

impl TownFacts {
    /// The facts of `town`, read with `part`.
    fn of(town: &Town, part: &TownRuleset) -> Self {
        Self {
            fort: fort_level(town, part),
            capitol: standing_capitol(town, part).is_some(),
            built_today: town.built_today(),
        }
    }
}

/// Every town's record by [`TownId`] (its [`Town`] and its [`TownStatic`]), shared between forks
/// and copied on its first write, as the store of heroes (ADR-0017 point 1): a fork costs one
/// reference count, a write copies the list of references and the one town, the static parts are
/// shared by every fork. A map without towns has an empty store, which a fork does not count.
/// Nothing sees the sharing; its canonical hash ([`crate::TOWN_STATE_HASH_VERSION`]) folds in each
/// town in `TownId` order.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Towns(Option<Arc<Store>>);

/// A store with towns.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Store {
    /// Each town's record, by `TownId`.
    towns: Vec<Arc<Town>>,
    /// Each town's static part, by `TownId`.
    statics: Arc<[TownStatic]>,
}

impl Towns {
    /// The store of `towns`, the town of `TownId` `i` at index `i` (the map's towns in object
    /// order, ADR-0028 point 3), or `None` if they are more than [`MAX_TOWNS`].
    pub fn new(towns: Vec<(Town, TownStatic)>) -> Option<Self> {
        (towns.len() <= MAX_TOWNS).then(|| {
            let (towns, statics): (Vec<_>, Vec<_>) = towns
                .into_iter()
                .map(|(town, part)| (Arc::new(town), part))
                .unzip();
            Self((!towns.is_empty()).then(|| {
                Arc::new(Store {
                    towns,
                    statics: statics.into(),
                })
            }))
        })
    }

    /// The towns' records and static parts, by `TownId`.
    fn parts(&self) -> (&[Arc<Town>], &[TownStatic]) {
        match &self.0 {
            Some(store) => (&store.towns, &store.statics),
            None => (&[], &[]),
        }
    }

    /// The number of towns.
    pub fn len(&self) -> usize {
        self.parts().0.len()
    }

    /// Whether the store holds no town.
    pub fn is_empty(&self) -> bool {
        self.0.is_none()
    }

    /// The record of `town`, if the store holds it.
    pub fn get(&self, town: TownId) -> Option<&Town> {
        self.parts().0.get(town.index()).map(Arc::as_ref)
    }

    /// The static part of `town`, if the store holds it.
    pub fn town_static(&self, town: TownId) -> Option<&TownStatic> {
        self.parts().1.get(town.index())
    }

    /// Whether `town` has a Capitol: a standing hall-4 building (public, E09 increment 1's design
    /// note §12); false for a town the store lacks.
    pub fn capitol(&self, town: TownId, part: &TownRuleset) -> bool {
        self.get(town)
            .and_then(|t| standing_capitol(t, part))
            .is_some()
    }

    /// The public facts of `town` ([`TownFacts`]), if the store holds it.
    pub fn facts(&self, town: TownId, part: &TownRuleset) -> Option<TownFacts> {
        self.get(town).map(|t| TownFacts::of(t, part))
    }

    /// Builds `building` in `town` (E09 design §26, §27): its ever-built set becomes
    /// [`built_with`]'s, and its day's mark is set when `mark` (a build of its owner's) and kept
    /// otherwise (a town event's never sets or clears it); its public facts before the build,
    /// so that the game tells their change. No test of the hall's: the game checks the building
    /// first (`check_build`, E09 T17b) or builds a town event's without one. `None` for a town the
    /// store lacks or a building outside its faction's record, which changes nothing (never in
    /// play: the game checked both). Copies only the one town (ADR-0028 point 4).
    // rule: TWN-BLD-004
    // rule: TWN-BLD-005
    pub fn build(
        &mut self,
        town: TownId,
        building: BuildingId,
        part: &TownRuleset,
        mark: bool,
    ) -> Option<TownFacts> {
        let (before, built) = self
            .get(town)
            .and_then(|t| Some((TownFacts::of(t, part), built_with(t, part, building)?)))?;
        self.write(town, |t| t.rebuilt(built, t.built_today() || mark));
        Some(before)
    }

    /// Starts a day for the towns (E09 design §26, §30: the original's `new_day` towns' loop
    /// 0x4C8067): clears every set day's mark. Only the towns built in the day before are
    /// written; a store without one copies nothing.
    // rule: TWN-BLD-004
    pub fn start_day(&mut self) {
        self.0
            .iter_mut()
            .filter(|store| store.towns.iter().any(|t| t.built_today()))
            .for_each(|store| {
                Arc::make_mut(store)
                    .towns
                    .iter_mut()
                    .filter(|t| t.built_today())
                    .for_each(|t| *t = Arc::new(t.rebuilt(t.built(), false)));
            });
    }

    /// The capture's rule of `town`'s day's mark (E09 design §26, §30; survey construction §3,
    /// `town_change_owner` 0x4C63B8, decision 43): cleared when `clear_mark` (the game's: the old
    /// owner's team has no human and the new owner's has one), otherwise kept, so that a town a
    /// human built in today cannot be built in again today by the human who took it. Whether the
    /// town changed; a town the store lacks or without a mark changes nothing. The Capitol's limit
    /// stays [`Towns::apply_capitol_limit`].
    // rule: TWN-BLD-004
    pub fn captured(&mut self, town: TownId, clear_mark: bool) -> bool {
        let clear = clear_mark && self.get(town).is_some_and(Town::built_today);
        if clear {
            self.write(town, |t| t.rebuilt(t.built(), false));
        }
        clear
    }

    /// Applies the Capitol's limit to `town` at its capture ([`capitol_limit`]; E09 increment 1's
    /// design note §11: the game's, after the adventure's capture), `holds_another` whether the
    /// new owner holds another town with a Capitol (`false` for a neutral one); whether the town
    /// changed. A town the store lacks changes nothing.
    // rule: TWN-CAP-002
    pub fn apply_capitol_limit(
        &mut self,
        town: TownId,
        part: &TownRuleset,
        holds_another: bool,
    ) -> bool {
        let change = self
            .get(town)
            .map(|t| (t.built(), capitol_limit(t, part, holds_another)))
            .filter(|(built, limited)| built != limited);
        change
            .iter()
            .for_each(|&(_, limited)| self.write(town, |t| t.rebuilt(limited, t.built_today())));
        change.is_some()
    }

    /// Writes `town`'s record as `change` makes it of the old one (only the rules write it,
    /// design note §4): the first write of a fork copies the list of references, then the one town
    /// gets a new record; every other town and the static parts stay shared. A town the store
    /// lacks changes nothing (and copies nothing).
    pub(crate) fn write(&mut self, town: TownId, change: impl Fn(&Town) -> Town) {
        let at = town.index();
        self.0
            .iter_mut()
            .filter(|store| at < store.towns.len())
            .flat_map(|store| Arc::make_mut(store).towns.get_mut(at))
            .for_each(|slot| *slot = Arc::new(change(slot)));
    }
}

impl CanonicalHash for Towns {
    /// Encoding v2 (E09 T17a; v1 E09 T6a, the first): version; the number of towns; each town in
    /// `TownId` order: its faction's handle, the bits of its ever-built and forbidden sets, its
    /// day's mark (v2); then each town's static part in `TownId` order: its name (0 for none,
    /// else 1 and the text), its events (length, then each: `hash_event`).
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_u32(TOWN_STATE_HASH_VERSION);
        let (towns, statics) = self.parts();
        h.write_len(towns.len());
        for town in towns {
            town.faction().hash_into(h);
            h.write_u64(town.built().bits());
            h.write_u64(town.forbidden().bits());
            h.write_bool(town.built_today());
        }
        for part in statics {
            match &part.name {
                None => h.write_u8(0),
                Some(name) => {
                    h.write_u8(1);
                    name.as_str().hash_into(h);
                }
            }
            h.write_len(part.events.len());
            for event in &part.events {
                hash_event(event, h);
            }
        }
    }
}

/// A town event in the hash: its players (length, then each id), whether it fires for a human
/// and for a computer player, its first day, its interval, its resources (length, then each
/// handle and amount), its buildings' bits, its seven creature counts.
fn hash_event(event: &TownEvent, h: &mut StateHasher) {
    h.write_len(event.players.len());
    for player in &event.players {
        h.write_u8(player.raw());
    }
    h.write_bool(event.human);
    h.write_bool(event.computer);
    h.write_u16(event.first_day);
    h.write_u16(event.interval);
    h.write_len(event.resources.len());
    for &(resource, amount) in &event.resources {
        resource.hash_into(h);
        h.write_i32(amount);
    }
    h.write_u64(event.buildings.bits());
    for &count in &event.creatures {
        h.write_u16(count);
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use std::vec;
    use std::vec::Vec;

    use h3_content::{FactionId, ResourceId};
    use h3_contracts::{Moat, PlayerId};
    use h3_kernel::hash::hash_of;

    use super::*;
    use crate::building::{BuildingId, BuildingSet};
    use crate::fixture::{
        CAPITOL, CASTLE, CITADEL, CITY_HALL, FORT, GOLD, KEEP, ORE, SILO, TAVERN, TOWN_HALL,
        VILLAGE_HALL, WILD, WOOD, part, town, town_of,
    };
    use crate::{daily_income, has_tavern, siege};

    fn set(indices: &[usize]) -> BuildingSet {
        indices.iter().fold(BuildingSet::EMPTY, |set, &i| {
            set.with(BuildingId::from_index(i).unwrap())
        })
    }

    fn event() -> TownEvent {
        TownEvent {
            players: vec![PlayerId::new(0), PlayerId::new(3)],
            human: true,
            computer: false,
            first_day: 3,
            interval: 7,
            resources: vec![
                (ResourceId::from_raw(6), 500),
                (ResourceId::from_raw(0), -5),
            ],
            buildings: set(&[4]),
            creatures: [10, 0, 0, 0, 0, 0, 2],
        }
    }

    /// What a store is made of.
    type Parts = Vec<(Town, TownStatic)>;

    /// Two towns: a named one with an event, an unnamed one without.
    fn towns() -> Parts {
        vec![
            (
                Town::new(FactionId::from_raw(0), set(&[0, 1, 4]), set(&[9])),
                TownStatic {
                    name: Some("Tirlan".into()),
                    events: vec![event()],
                },
            ),
            (
                Town::new(FactionId::from_raw(8), set(&[0]), BuildingSet::EMPTY),
                TownStatic::default(),
            ),
        ]
    }

    #[test]
    fn a_store_holds_its_towns_by_id() {
        let store = Towns::new(towns()).unwrap();
        assert_eq!((store.len(), store.is_empty()), (2, false));
        let second = TownId::new(1);
        assert_eq!(
            store.get(second).map(Town::faction),
            Some(FactionId::from_raw(8))
        );
        assert_eq!(store.town_static(TownId::new(0)).unwrap().events, [event()]);
        assert_eq!(store.town_static(second), Some(&TownStatic::default()));
        assert!(store.get(TownId::new(2)).is_none() && store.town_static(TownId::new(2)).is_none());
    }

    #[test]
    fn a_map_without_towns_has_the_empty_store() {
        let store = Towns::new(Vec::new()).unwrap();
        assert_eq!(store, Towns::default());
        assert_eq!((store.len(), store.is_empty()), (0, true));
        assert!(store.get(TownId::new(0)).is_none() && store.town_static(TownId::new(0)).is_none());
        let too_many = vec![towns().remove(1); MAX_TOWNS + 1];
        assert!(Towns::new(too_many).is_none());
    }

    #[test]
    fn a_fork_shares_the_store() {
        let store = Towns::new(towns()).unwrap();
        let fork = store.clone();
        assert!(matches!((&store.0, &fork.0), (Some(a), Some(b)) if Arc::ptr_eq(a, b)));
        assert_eq!(fork, store);
    }

    #[test]
    fn the_hash_covers_every_town_and_static_part() {
        let base = hash_of(&Towns::new(towns()).unwrap());
        // Pinned: a change of the encoding bumps `TOWN_STATE_HASH_VERSION` and this value.
        assert_eq!(base, 0x1F7C_E05E_64B8_2246);
        assert_eq!(
            hash_of(&Towns::default()),
            hash_of(&Towns::new(Vec::new()).unwrap())
        );
        let changed: [fn(&mut Parts); 16] = [
            |t| t[1].0 = Town::new(FactionId::from_raw(7), set(&[0]), BuildingSet::EMPTY),
            |t| t[1].0 = t[1].0.rebuilt(set(&[0]), true),
            |t| t[1].0 = Town::new(FactionId::from_raw(8), set(&[0, 1]), BuildingSet::EMPTY),
            |t| t[1].0 = Town::new(FactionId::from_raw(8), set(&[0]), set(&[2])),
            |t| t[1].1.name = Some("Tirlan".into()),
            |t| t[0].1.name = Some("Tirlan2".into()),
            |t| t[0].1.name = None,
            |t| t[0].1.events.clear(),
            |t| t[0].1.events[0].players.truncate(1),
            |t| t[0].1.events[0].human = false,
            |t| t[0].1.events[0].computer = true,
            |t| t[0].1.events[0].first_day = 4,
            |t| t[0].1.events[0].interval = 0,
            |t| t[0].1.events[0].resources[1].1 = -6,
            |t| t[0].1.events[0].buildings = BuildingSet::EMPTY,
            |t| t[0].1.events[0].creatures[6] = 3,
        ];
        for (i, change) in changed.iter().enumerate() {
            let mut towns = towns();
            change(&mut towns);
            assert_ne!(hash_of(&Towns::new(towns).unwrap()), base, "change {i}");
        }
        let mut one = towns();
        one.pop();
        assert_ne!(hash_of(&Towns::new(one).unwrap()), base);
    }

    const HALLS: [usize; 4] = [VILLAGE_HALL, TOWN_HALL, CITY_HALL, CAPITOL];

    /// Three Keep towns: two with a Capitol (the second named, a Fort forbidden), one with the
    /// Village Hall.
    fn keep_towns() -> Towns {
        let named = TownStatic {
            name: Some("Tirlan".into()),
            events: Vec::new(),
        };
        Towns::new(vec![
            (town(&HALLS), TownStatic::default()),
            (Town::new(KEEP, set(&HALLS), set(&[FORT])), named),
            (town(&[VILLAGE_HALL]), TownStatic::default()),
        ])
        .unwrap()
    }

    /// The two stores' lists, which both hold towns.
    fn stores<'a>(a: &'a Towns, b: &'a Towns) -> (&'a Arc<Store>, &'a Arc<Store>) {
        (a.0.as_ref().unwrap(), b.0.as_ref().unwrap())
    }

    #[test]
    fn a_capitol_is_a_standing_hall_4_building() {
        let part = part();
        let store = Towns::new(vec![
            (town(&HALLS), TownStatic::default()),
            (town(&HALLS[..3]), TownStatic::default()),
            (town_of(WILD, &HALLS), TownStatic::default()),
        ])
        .unwrap();
        let capitol = |i: u16| store.capitol(TownId::new(i), &part);
        assert_eq!(
            [capitol(0), capitol(1), capitol(2), capitol(3)],
            [true, false, false, false]
        );
    }

    #[test]
    fn the_capitol_s_limit_writes_one_town_of_a_fork() {
        // covers: TWN-CAP-002
        let part = part();
        let store = keep_towns();
        let mut fork = store.clone();
        let second = TownId::new(1);
        assert!(fork.apply_capitol_limit(second, &part, true));
        // The fork's town is demoted (faction and forbidden set kept); the old store keeps its.
        let demoted = Town::new(KEEP, set(&HALLS[..3]), set(&[FORT]));
        assert_eq!(fork.get(second), Some(&demoted));
        assert_eq!(store.get(second).map(Town::built), Some(set(&HALLS)));
        assert!(store.capitol(second, &part) && !fork.capitol(second, &part));
        // The list is copied; the other towns and the static parts are still shared.
        let (old, new) = stores(&store, &fork);
        assert!(!Arc::ptr_eq(old, new) && Arc::ptr_eq(&old.statics, &new.statics));
        assert!(Arc::ptr_eq(&old.towns[0], &new.towns[0]));
        assert!(Arc::ptr_eq(&old.towns[2], &new.towns[2]));
        assert!(!Arc::ptr_eq(&old.towns[1], &new.towns[1]));
        assert_ne!(hash_of(&fork), hash_of(&store));
        // A later write of the same fork copies no list again.
        let list = Arc::as_ptr(stores(&fork, &fork).0);
        assert!(fork.apply_capitol_limit(TownId::new(0), &part, true));
        assert_eq!(Arc::as_ptr(stores(&fork, &fork).0), list);
    }

    #[test]
    fn a_limit_without_a_change_writes_nothing() {
        let part = part();
        let store = keep_towns();
        let mut fork = store.clone();
        // No other Capitol held (or a neutral captor), no Capitol, no such town.
        assert!(!fork.apply_capitol_limit(TownId::new(0), &part, false));
        assert!(!fork.apply_capitol_limit(TownId::new(2), &part, true));
        assert!(!fork.apply_capitol_limit(TownId::new(3), &part, true));
        fork.write(TownId::new(3), |t| t.rebuilt(set(&[0]), true));
        let (old, new) = stores(&store, &fork);
        assert!(Arc::ptr_eq(old, new));
        let mut empty = Towns::default();
        assert!(!empty.apply_capitol_limit(TownId::new(0), &part, true));
        empty.write(TownId::new(0), |t| t.rebuilt(set(&[0]), true));
        assert_eq!(empty, Towns::default());
    }

    fn id(index: usize) -> BuildingId {
        BuildingId::from_index(index).unwrap()
    }

    const fn facts(fort: FortLevel, capitol: bool, built_today: bool) -> TownFacts {
        TownFacts {
            fort,
            capitol,
            built_today,
        }
    }

    #[test]
    fn a_build_writes_one_town_of_a_fork_and_sets_its_mark() {
        // covers: TWN-BLD-004
        // covers: TWN-BLD-005
        let part = part();
        let store = keep_towns();
        let mut fork = store.clone();
        let third = TownId::new(2);
        // The facts before the build; after it, its ever-built set and its mark.
        let before = fork.build(third, id(TOWN_HALL), &part, true);
        assert_eq!(before, Some(facts(FortLevel::None, false, false)));
        assert_eq!(
            fork.get(third),
            Some(&town(&[VILLAGE_HALL]).rebuilt(set(&[VILLAGE_HALL, TOWN_HALL]), true))
        );
        assert_eq!(
            fork.facts(third, &part),
            Some(facts(FortLevel::None, false, true))
        );
        // The old store keeps its town; the list is copied, the other towns and the static parts
        // are still shared.
        assert_eq!(store.get(third), Some(&town(&[VILLAGE_HALL])));
        let (old, new) = stores(&store, &fork);
        assert!(!Arc::ptr_eq(old, new) && Arc::ptr_eq(&old.statics, &new.statics));
        assert!(Arc::ptr_eq(&old.towns[0], &new.towns[0]));
        assert!(Arc::ptr_eq(&old.towns[1], &new.towns[1]));
        assert!(!Arc::ptr_eq(&old.towns[2], &new.towns[2]));
        assert_ne!(hash_of(&fork), hash_of(&store));
        // A later build of the same fork copies no list again.
        let list = Arc::as_ptr(stores(&fork, &fork).0);
        let first = fork.build(TownId::new(0), id(FORT), &part, true);
        assert_eq!(first, Some(facts(FortLevel::None, true, false)));
        assert_eq!(Arc::as_ptr(stores(&fork, &fork).0), list);
    }

    #[test]
    fn a_town_event_s_build_keeps_the_mark_and_tests_nothing() {
        // covers: TWN-BLD-004
        let part = part();
        let mut towns = keep_towns();
        let first = TownId::new(0);
        // Unmarked, an event's build leaves the mark clear (the day's build free).
        let before = towns.build(first, id(FORT), &part, false);
        assert_eq!(before, Some(facts(FortLevel::None, true, false)));
        assert_eq!(towns.get(first).map(Town::built_today), Some(false));
        // Marked by the owner's build, an event's build keeps it set.
        towns.build(first, id(CITADEL), &part, true);
        let before = towns.build(first, id(CASTLE), &part, false);
        assert_eq!(before, Some(facts(FortLevel::Citadel, true, true)));
        assert_eq!(
            towns.facts(first, &part),
            Some(facts(FortLevel::Castle, true, true))
        );
        // The store tests nothing: the second town's forbidden Fort is built.
        let second = TownId::new(1);
        towns.build(second, id(FORT), &part, false);
        let built = towns
            .get(second)
            .map(|t| (t.built().contains(id(FORT)), t.forbidden()));
        assert_eq!(built, Some((true, set(&[FORT]))));
    }

    #[test]
    fn a_build_outside_the_store_or_the_record_changes_nothing() {
        let part = part();
        let store = keep_towns();
        let mut fork = store.clone();
        // A town the store lacks; a building the Keep's record lacks.
        assert_eq!(fork.build(TownId::new(3), id(FORT), &part, true), None);
        assert_eq!(fork.build(TownId::new(0), id(40), &part, true), None);
        let (old, new) = stores(&store, &fork);
        assert!(Arc::ptr_eq(old, new));
        assert_eq!(fork.facts(TownId::new(3), &part), None);
        // A faction without a record; a store without towns.
        let mut wild = Towns::new(vec![(town_of(WILD, &[]), TownStatic::default())]).unwrap();
        assert_eq!(wild.build(TownId::new(0), id(0), &part, true), None);
        let mut empty = Towns::default();
        assert_eq!(empty.build(TownId::new(0), id(0), &part, true), None);
        assert_eq!(empty, Towns::default());
    }

    #[test]
    fn a_day_s_start_clears_the_marks_of_the_towns_built_in() {
        // covers: TWN-BLD-004
        let part = part();
        let mut towns = keep_towns();
        towns.build(TownId::new(0), id(FORT), &part, true);
        towns.build(TownId::new(2), id(TOWN_HALL), &part, true);
        let store = towns.clone();
        towns.start_day();
        assert!((0..3).all(|i| towns.get(TownId::new(i)).is_some_and(|t| !t.built_today())));
        // Only the marked towns are written; their buildings are kept.
        let (old, new) = stores(&store, &towns);
        assert!(Arc::ptr_eq(&old.towns[1], &new.towns[1]));
        assert!(!Arc::ptr_eq(&old.towns[0], &new.towns[0]));
        assert!(!Arc::ptr_eq(&old.towns[2], &new.towns[2]));
        let built = |towns: &Towns| towns.get(TownId::new(2)).map(Town::built);
        assert_eq!(built(&towns), built(&store));
        // A day without a marked town copies nothing, nor does a store without towns.
        let unmarked = towns.clone();
        towns.start_day();
        let (old, new) = stores(&unmarked, &towns);
        assert!(Arc::ptr_eq(old, new));
        let mut empty = Towns::default();
        empty.start_day();
        assert_eq!(empty, Towns::default());
    }

    #[test]
    fn a_capture_clears_the_mark_only_when_told() {
        // covers: TWN-BLD-004
        let part = part();
        let mut towns = keep_towns();
        let first = TownId::new(0);
        towns.build(first, id(FORT), &part, true);
        let store = towns.clone();
        // Kept (a human's town taken by a human): nothing is written.
        assert!(!towns.captured(first, false));
        let (old, new) = stores(&store, &towns);
        assert!(Arc::ptr_eq(old, new));
        assert_eq!(towns.get(first).map(Town::built_today), Some(true));
        // The Capitol's limit keeps the mark too.
        assert!(towns.apply_capitol_limit(first, &part, true));
        assert_eq!(
            towns.facts(first, &part),
            Some(facts(FortLevel::Fort, false, true))
        );
        // Cleared: only the mark changes.
        let built = towns.get(first).map(Town::built);
        assert!(towns.captured(first, true));
        assert_eq!(towns.get(first).map(Town::built_today), Some(false));
        assert_eq!(towns.get(first).map(Town::built), built);
        // A town without a mark, or one the store lacks, changes nothing.
        assert!(!towns.captured(first, true));
        assert!(!towns.captured(TownId::new(1), true));
        assert!(!towns.captured(TownId::new(3), true));
    }

    #[test]
    fn increment_1_s_reads_follow_a_build_at_once() {
        // covers: TWN-BLD-005
        let part = part();
        let at = TownId::new(0);
        let mut towns = Towns::new(vec![(town(&[VILLAGE_HALL]), TownStatic::default())]).unwrap();
        let read = |towns: &Towns| towns.get(at).cloned().unwrap();
        // A Town Hall: the day's gold.
        assert_eq!(daily_income(&read(&towns), &part), [(GOLD, 500)]);
        towns.build(at, id(TOWN_HALL), &part, true);
        assert_eq!(daily_income(&read(&towns), &part), [(GOLD, 1000)]);
        // A Citadel: the Fort with it, the record's moat in the siege.
        assert_eq!(siege(&read(&towns), &part).moat, Moat::Absent);
        towns.build(at, id(CITADEL), &part, true);
        let walls = siege(&read(&towns), &part);
        assert_eq!(
            (walls.fort, walls.moat),
            (FortLevel::Citadel, Moat::Damaging)
        );
        // A Tavern: its owner's view level of the other towns (TWN-VEW-002).
        assert!(!has_tavern(&read(&towns), &part));
        towns.build(at, id(TAVERN), &part, true);
        assert!(has_tavern(&read(&towns), &part));
        // A silo: its resources beside the hall's gold.
        towns.build(at, id(SILO), &part, true);
        let paid = [(WOOD, 1), (ORE, 1), (GOLD, 1000)];
        assert_eq!(daily_income(&read(&towns), &part), paid);
        // The Capitol: public at once, the City Hall with it.
        assert!(!towns.capitol(at, &part));
        towns.build(at, id(CAPITOL), &part, true);
        assert!(towns.capitol(at, &part));
        assert_eq!(daily_income(&read(&towns), &part)[2], (GOLD, 4000));
    }
}
