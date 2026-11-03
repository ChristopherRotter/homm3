//! A faction's buildings by index (ADR-0028 point 5): [`BuildingId`] and [`BuildingSet`], the sets
//! the town part holds (what a building supersedes) and a town's record will hold (the buildings
//! built and forbidden; E09 increment 1's design note §4).

/// The most buildings a faction's town record holds: a [`BuildingSet`] is one `u64`.
pub const MAX_BUILDINGS: usize = 64;

/// A building of a faction's town record: its index in the record's list, below
/// [`MAX_BUILDINGS`]. It names a building only together with its faction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BuildingId(u8);

impl BuildingId {
    /// The building at `index` of a record's list, if `index` is below [`MAX_BUILDINGS`].
    pub fn from_index(index: usize) -> Option<Self> {
        u8::try_from(index)
            .ok()
            .filter(|&raw| usize::from(raw) < MAX_BUILDINGS)
            .map(Self)
    }

    /// Its index in the record's list.
    pub fn index(self) -> usize {
        usize::from(self.0)
    }

    /// Its index as a byte (the canonical hashes' encoding).
    pub const fn raw(self) -> u8 {
        self.0
    }
}

/// A set of one faction's buildings, a bit per [`BuildingId`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct BuildingSet(u64);

impl BuildingSet {
    /// No building.
    pub const EMPTY: Self = Self(0);

    /// The set's bits, bit `i` for the building of index `i` (the canonical hash's encoding).
    pub const fn bits(self) -> u64 {
        self.0
    }

    /// Whether the set holds `building`.
    pub const fn contains(self, building: BuildingId) -> bool {
        self.0 & (1 << building.0) != 0
    }

    /// The set with `building` added.
    #[must_use]
    pub const fn with(self, building: BuildingId) -> Self {
        Self(self.0 | 1 << building.0)
    }

    /// The buildings of either set.
    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_id_is_below_the_most_buildings() {
        assert_eq!(BuildingId::from_index(63).map(BuildingId::index), Some(63));
        assert_eq!(BuildingId::from_index(63).map(BuildingId::raw), Some(63));
        assert!(BuildingId::from_index(MAX_BUILDINGS).is_none());
        assert!(BuildingId::from_index(300).is_none());
    }

    #[test]
    fn a_set_holds_its_buildings() {
        let ids = [0, 5, 63].map(|i| BuildingId::from_index(i).unwrap());
        let set = BuildingSet::EMPTY
            .with(ids[0])
            .union(BuildingSet::EMPTY.with(ids[2]));
        assert!(set.contains(ids[0]) && set.contains(ids[2]) && !set.contains(ids[1]));
        assert_eq!(set.bits(), 1 | 1 << 63);
        assert_eq!(BuildingSet::default(), BuildingSet::EMPTY);
    }
}
