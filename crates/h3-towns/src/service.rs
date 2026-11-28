//! The town's services (E09 T59a, the town epic's design note §110, §111; survey
//! `E09-trades.md` §4, §5): the town part's half of a war machine's purchase at the Blacksmith or
//! the Stronghold's Ballista Yard (TWN-SMT-001) and of a boat built at the Shipyard
//! (TWN-SHY-001); since E09 T59b1 of the Magic University's opening (TWN-UNI-001, design §112).
//! The game checks the rest (the town's owner, its hero and what the hero holds, the adventure's
//! spot, the treasury, the map's bans), pays the price and gives the machine, launches the boat or
//! opens the University (ADR-0028 point 6, as a build's).

use h3_content::{ArtifactTypeId, ResourceId, Ruleset, SkillId};

use crate::building::{BuildingId, BuildingSet};
use crate::command::TownCommandError;
use crate::record::Town;
use crate::ruleset::{ServiceRoles, TownRecord, TownRuleset};

/// What a building sells as a war machine (TWN-SMT-001; design §110): the machine's artifact and
/// its price.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WarMachineSale<'a> {
    /// The artifact given to the hero (a core artifact with a `war_machine` creature: the
    /// building's `war_machine`, TWN-TRD-001).
    pub artifact: ArtifactTypeId,
    /// The price: the cost of the artifact's war machine creature, in the core's creature record's
    /// order, each resource charged as itself (`TWN-QUIRK-22`).
    pub price: &'a [(ResourceId, u32)],
}

/// What the town's Shipyard builds (TWN-SHY-001; design §111): the part's boat and its price.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoatSale<'a> {
    /// The price, in the core's resource order (`TownParams::boat_price`, TWN-TRD-001).
    pub price: &'a [(ResourceId, u32)],
    /// The boat's type, appearance only (`TownParams::boat_type`).
    pub boat_type: u8,
}

/// What the town's Magic University teaches (TWN-UNI-001; E09 T59b1, design §112): the part's
/// skills and the price of each.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UniversitySale<'a> {
    /// The skills, in the slots' order (`TownParams::university_skills`, TWN-TRD-001: the
    /// original's Fire, Air, Water and Earth Magic), before the game leaves out a school the map
    /// bans (`TWN-QUIRK-16`'s flag off).
    pub skills: &'a [SkillId],
    /// The price of each, in the core's resource order (`TownParams::university_price`: the
    /// original's 2 000 gold).
    pub price: &'a [(ResourceId, u32)],
}

/// What the legal listing offers at a town (E09 T59a, design §115): its standing buildings that
/// sell a war machine ([`war_machine_buildings`]), its boat where a Shipyard stands
/// ([`check_boat`]) and since E09 T59b1 its University where one stands ([`check_university`]),
/// read together (the record and its standing set once a town: `bench-check`'s
/// `legal-towns-36`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServiceOffers<'a> {
    /// The standing buildings that sell a war machine, in the record's order.
    pub machines: BuildingSet,
    /// The boat, where a Shipyard stands.
    pub boat: Option<BoatSale<'a>>,
    /// The University, where one stands.
    pub university: Option<UniversitySale<'a>>,
}

/// [`war_machine_buildings`], [`check_boat`] and [`check_university`] of `town` at once
/// (TWN-SMT-001, TWN-SHY-001, TWN-UNI-001; E09 T59a, T59b1, design §115: the legal listing's
/// question to the town part): the same answers, the record's derived roles and the town's
/// superseded buildings read once.
// rule: TWN-SMT-001, TWN-SHY-001, TWN-UNI-001
pub fn service_offers<'a>(town: &Town, part: &'a TownRuleset) -> ServiceOffers<'a> {
    let read = part.town_with_service_roles(town.faction());
    let roles = read.map_or(ServiceRoles::default(), |(_, roles)| roles);
    let standing = read.map_or(BuildingSet::EMPTY, |(record, roles)| {
        let of = roles.smiths.union(roles.shipyard).union(roles.university);
        standing(town, record, roles, of)
    });
    let params = part.params();
    let boat = params
        .filter(|_| standing.intersects(roles.shipyard))
        .map(|params| BoatSale {
            price: &params.boat_price,
            boat_type: params.boat_type,
        });
    let university = params
        .filter(|_| standing.intersects(roles.university))
        .map(|params| UniversitySale {
            skills: &params.university_skills,
            price: &params.university_price,
        });
    ServiceOffers {
        machines: standing.intersection(roles.smiths),
        boat,
        university,
    }
}

/// The town part's test of a war machine's purchase at `building` of `town` (TWN-SMT-001; E09
/// T59a, design §110; survey §4.3, §4.6, the original's `town_blacksmith_buy` 0x5D1D30 from the
/// town screen's building 16 or a Stronghold's 22), in this order, each refusal changing nothing:
/// the building of the town's faction's record, else [`TownCommandError::UnknownBuilding`]; one
/// that sells a war machine (its `war_machine`, TWN-TRD-001: the artifact, its creature in `core`
/// with its cost), else [`TownCommandError::NoWarMachine`]; standing (built and not superseded),
/// else [`TownCommandError::NotStanding`]. Then the machine and its price: the creature's cost,
/// every resource charged as itself (**`TWN-QUIRK-22`**, T-25 fixed: the original tests the gold
/// alone and subtracts each of the seven costs from the gold; with the shipped tables a machine
/// costs gold only, the Ballista 2 500, the Ammo Cart 1 000, the First Aid Tent 750). No day mark,
/// no stock: one machine an opening only because the hero then holds it.
///
/// # Errors
///
/// The refusals above; the town is unchanged in every case.
// rule: TWN-SMT-001
pub fn check_war_machine<'a>(
    town: &Town,
    part: &TownRuleset,
    core: &'a Ruleset,
    building: BuildingId,
) -> Result<WarMachineSale<'a>, TownCommandError> {
    let read = part.town_with_service_roles(town.faction());
    let Some((record, roles, of)) = read.and_then(|(record, roles)| {
        record
            .building(building)
            .map(|of| (record, roles, of.war_machine))
    }) else {
        return Err(TownCommandError::UnknownBuilding);
    };
    let sale = of
        .and_then(|artifact| {
            core.artifact(artifact)
                .and_then(|a| a.war_machine)
                .and_then(|creature| core.creature(creature))
                .map(|info| WarMachineSale {
                    artifact,
                    price: &info.cost,
                })
        })
        .ok_or(TownCommandError::NoWarMachine)?;
    if standing(town, record, roles, BuildingSet::EMPTY.with(building)).is_empty() {
        return Err(TownCommandError::NotStanding);
    }
    Ok(sale)
}

/// The buildings of `town` that sell a war machine and stand (TWN-SMT-001; E09 T59a, design
/// §110, §115: the legal listing's purchases, in the record's order; a Stronghold's Blacksmith
/// and Ballista Yard both, survey §4.6): the record's derived roles, no walk over the town's
/// buildings. None for a town of a faction without a record.
// rule: TWN-SMT-001
pub fn war_machine_buildings(town: &Town, part: &TownRuleset) -> BuildingSet {
    part.town_with_service_roles(town.faction())
        .map_or(BuildingSet::EMPTY, |(record, roles)| {
            standing(town, record, roles, roles.smiths)
        })
}

/// The town part's test of a boat at `town`'s Shipyard (TWN-SHY-001; E09 T59a, design §111;
/// survey §5.2, the original's case 6 of `town_screen_proc` 0x5D36B4): a standing building of role
/// `shipyard`, else [`TownCommandError::NotStanding`]; then the part's price and boat type (the
/// original's code: 1 000 gold, 10 wood, type 1 from every town). The spot and whatever stands on
/// it are the adventure's (TWN-SET-011); the ship (20) is derived from it, never a building of
/// the record (T-27).
///
/// # Errors
///
/// [`TownCommandError::NotStanding`]; the town is unchanged.
// rule: TWN-SHY-001
pub fn check_boat<'a>(
    town: &Town,
    part: &'a TownRuleset,
) -> Result<BoatSale<'a>, TownCommandError> {
    let stands = part
        .town_with_service_roles(town.faction())
        .is_some_and(|(record, roles)| !standing(town, record, roles, roles.shipyard).is_empty());
    // A part with a record has its parameters (`TownCompileError::MissingParams`).
    part.params()
        .filter(|_| stands)
        .map(|params| BoatSale {
            price: &params.boat_price,
            boat_type: params.boat_type,
        })
        .ok_or(TownCommandError::NotStanding)
}

/// The town part's test of the Magic University's opening at `town` (TWN-UNI-001; E09 T59b1,
/// design §112, decision 210; survey `E09-trades.md` §6.1, §6.3: the original's case 21 of
/// `town_screen_proc` 0x5D3963, the Conflux's building clickable while it stands, then
/// `town_university_open` 0x5D2DA0 running the map's University dialog): a standing building of
/// role `university`, else [`TownCommandError::NoUniversity`]; then the part's skills and price
/// (the original's code: Fire, Air, Water and Earth Magic at 2 000 gold each). The student, the
/// map's bans (`TWN-QUIRK-16`) and the slots are the game's and the hero's (GAM-DEC-006,
/// HER-CHG-004).
///
/// # Errors
///
/// [`TownCommandError::NoUniversity`]; the town is unchanged.
// rule: TWN-UNI-001
pub fn check_university<'a>(
    town: &Town,
    part: &'a TownRuleset,
) -> Result<UniversitySale<'a>, TownCommandError> {
    let stands = part
        .town_with_service_roles(town.faction())
        .is_some_and(|(record, roles)| !standing(town, record, roles, roles.university).is_empty());
    // A part with a record has its parameters (`TownCompileError::MissingParams`).
    part.params()
        .filter(|_| stands)
        .map(|params| UniversitySale {
            skills: &params.university_skills,
            price: &params.university_price,
        })
        .ok_or(TownCommandError::NoUniversity)
}

/// The buildings of `of` (of the services' roles in `record`, read as `roles`) that stand in
/// `town`: ever built, and superseded by none of its built buildings that can supersede one.
fn standing(town: &Town, record: &TownRecord, roles: ServiceRoles, of: BuildingSet) -> BuildingSet {
    let built = town.built();
    let superseded = built
        .intersection(roles.superseding)
        .iter()
        .filter_map(|id| record.building(id))
        .fold(BuildingSet::EMPTY, |set, b| set.union(b.supersedes));
    built.intersection(of).difference(superseded)
}

#[cfg(test)]
mod tests;
