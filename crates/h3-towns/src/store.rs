//! The store of towns (ADR-0028 point 4; E09 increment 1's design note §4): every town's record
//! by `h3_contracts::TownId`, held by the game state beside the adventure's.

use alloc::sync::Arc;
use alloc::vec::Vec;

use h3_contracts::TownId;
use h3_kernel::{CanonicalHash, StateHasher};

use crate::TOWN_STATE_HASH_VERSION;
use crate::record::{Town, TownEvent, TownStatic};

/// The most towns a store holds: one per [`TownId`].
pub const MAX_TOWNS: usize = 1 << 16;

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
}

impl CanonicalHash for Towns {
    /// Encoding v1 (E09 T6a; the first): version; the number of towns; each town in `TownId`
    /// order: its faction's handle, the bits of its ever-built and forbidden sets; then each
    /// town's static part in `TownId` order: its name (0 for none, else 1 and the text), its
    /// events (length, then each: [`hash_event`]).
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_u32(TOWN_STATE_HASH_VERSION);
        let (towns, statics) = self.parts();
        h.write_len(towns.len());
        for town in towns {
            town.faction().hash_into(h);
            h.write_u64(town.built().bits());
            h.write_u64(town.forbidden().bits());
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
    use h3_contracts::PlayerId;
    use h3_kernel::hash::hash_of;

    use super::*;
    use crate::building::{BuildingId, BuildingSet};

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
        assert_eq!(base, 0xA02D_E3DA_4142_AB03);
        assert_eq!(
            hash_of(&Towns::default()),
            hash_of(&Towns::new(Vec::new()).unwrap())
        );
        let changed: [fn(&mut Parts); 15] = [
            |t| t[1].0 = Town::new(FactionId::from_raw(7), set(&[0]), BuildingSet::EMPTY),
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
}
