//! What every derivation reads of a town (E09 increment 1's design note §6): its faction's record
//! in the part and its two sets, the buildings **standing** and the buildings **ever built**. Each
//! building role is read over one of them (`h3-pack-towns`' format doc lists which).

use crate::building::{BuildingId, BuildingSet};
use crate::record::Town;
use crate::ruleset::{BuildingRoles, TownRecord, TownRuleset};

/// A town read with its faction's record in the part.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Roles<'a> {
    record: Option<&'a TownRecord>,
    built: BuildingSet,
    standing: BuildingSet,
}

impl<'a> Roles<'a> {
    /// `town` read with its faction's record in `part`. Without one (a town the store never
    /// holds: the setup gives a town only a faction with a record) no building has a role, so
    /// every derivation gives a town without buildings.
    pub(crate) fn of(town: &Town, part: &'a TownRuleset) -> Self {
        let record = part.town(town.faction());
        Self {
            record,
            built: town.built(),
            standing: record.map_or(BuildingSet::EMPTY, |r| town.standing(r)),
        }
    }

    /// The faction's record, if the part has one.
    pub(crate) const fn record(&self) -> Option<&'a TownRecord> {
        self.record
    }

    /// The buildings standing: ever built and not superseded by an ever-built one.
    pub(crate) const fn standing(&self) -> BuildingSet {
        self.standing
    }

    /// The buildings ever built.
    pub(crate) const fn built(&self) -> BuildingSet {
        self.built
    }

    /// The buildings of `set` with their roles, in index order; a building the record lacks has
    /// none and is left out.
    pub(crate) fn roles(
        &self,
        set: BuildingSet,
    ) -> impl Iterator<Item = (BuildingId, &'a BuildingRoles)> + 'a {
        let buildings = self.record.map_or(&[][..], |r| r.buildings.as_slice());
        set.iter()
            .filter_map(move |id| buildings.get(id.index()).map(|b| (id, &b.roles)))
    }

    /// The standing buildings with their roles, in index order.
    pub(crate) fn standing_roles(
        &self,
    ) -> impl Iterator<Item = (BuildingId, &'a BuildingRoles)> + 'a {
        self.roles(self.standing)
    }

    /// The buildings ever built with their roles, in index order.
    pub(crate) fn built_roles(&self) -> impl Iterator<Item = (BuildingId, &'a BuildingRoles)> + 'a {
        self.roles(self.built)
    }
}
