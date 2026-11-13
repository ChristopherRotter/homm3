//! The town context's command and its errors (ADR-0028 point 6, E09 decision 45; E09 design §27,
//! §29): `h3-game` wraps [`TownCommand`] as its `GameCommand::Town`, checks the turn, the town and
//! its owner, asks [`crate::check_build`] for the town's part (the hall's tests and the price),
//! checks the treasury, then pays and builds through [`crate::Towns::build`]; since E09 T27 a
//! recruitment opens the hire dialog of what a building sells ([`crate::recruit_source`]). A
//! refused command changes nothing.

use core::fmt;

use h3_contracts::TownId;

use crate::building::BuildingId;

/// A command to the town context, issued by a player (E09 decision 45). Increment 3's recruitment
/// and increment 5's services add their variants; exhaustive, as the game's other commands, so
/// that the game and the wire match every variant (SRV-Q25; E09 T18b1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
// rule: TWN-BLD-001, TWN-REC-001
pub enum TownCommand {
    /// Builds `building` in `town`, the player's, as its hall offers it ([`crate::buildable`]),
    /// for the part's price (TWN-BLD-001): no hero, no visit, no field trusted but the two ids.
    Build {
        /// The town.
        town: TownId,
        /// The building, of the town's faction's record.
        building: BuildingId,
    },
    /// Opens the hire dialog of what `building` of `town`, the player's, sells (TWN-REC-001, -002;
    /// E09 T27, design §49): a standing dwelling, horde or Portal of Summoning
    /// ([`crate::recruit_source`]), its creatures bought into the town's garrison. No hero, no
    /// visit: the original's town screen opens from the town list at any time.
    Recruit {
        /// The town.
        town: TownId,
        /// The building, of the town's faction's record.
        building: BuildingId,
    },
}

/// Why a town command was refused; the state is unchanged. The game gives the first two
/// ([`Self::UnknownTown`], [`Self::NotOwner`]); for a build [`crate::check_build`] the others from
/// [`Self::UnknownBuilding`] to [`Self::BuiltToday`], the first that applies in the order they are
/// listed (E09 design §27: the hall's texts' order, the day's mark last); for a recruitment
/// [`crate::recruit_source`] [`Self::UnknownBuilding`], then the last three (E09 T27).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TownCommandError {
    /// The town is not in the game's store.
    UnknownTown,
    /// The player does not own the town: only its owner builds (TD-11, E09 decision 42).
    NotOwner,
    /// The building is not of the town's faction's record.
    UnknownBuilding,
    /// It was ever built: it stands, or a building that supersedes it does.
    Built,
    /// It is forbidden in this town (the map's set, the Shipyard without water).
    Forbidden,
    /// The Grail's building, which the hall never builds (the Grail's path, increment 6).
    Grail,
    /// Not every building it requires was ever built.
    Requirements,
    /// A Capitol (a hall-4 building) while the player holds another.
    Capitol,
    /// A horde the hall offers as this building, its level's upgraded horde (its upgraded
    /// dwelling was ever built): one command names one outcome.
    OfferedAs(BuildingId),
    /// The town was built in today (its day's mark, TWN-BLD-004).
    BuiltToday,
    /// The building sells no creatures: it is no dwelling, horde or Portal of Summoning
    /// (TWN-REC-001).
    NotRecruiting,
    /// The building does not stand (never built, or superseded), or its base index's count moved
    /// to the upgrade's (TWN-REC-001).
    NotStanding,
    /// The Portal of Summoning holds nothing (TWN-REC-002: the original's message, no draw).
    NothingSummoned,
}

impl fmt::Display for TownCommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownTown => f.write_str("no such town"),
            Self::NotOwner => f.write_str("the town is not the player's"),
            Self::UnknownBuilding => f.write_str("the town's faction has no such building"),
            Self::Built => f.write_str("the building was built already"),
            Self::Forbidden => f.write_str("the building is forbidden in this town"),
            Self::Grail => f.write_str("the Grail's building is not built by the hall"),
            Self::Requirements => f.write_str("a building it requires was never built"),
            Self::Capitol => f.write_str("the player holds a Capitol already"),
            Self::OfferedAs(upgrade) => {
                write!(f, "the hall offers it as building {}", upgrade.index())
            }
            Self::BuiltToday => f.write_str("the town was built in today"),
            Self::NotRecruiting => f.write_str("the building sells no creatures"),
            Self::NotStanding => f.write_str("the building does not stand to sell creatures"),
            Self::NothingSummoned => f.write_str("the Portal of Summoning holds nothing"),
        }
    }
}

impl core::error::Error for TownCommandError {}

#[cfg(test)]
mod tests {
    extern crate std;

    use std::string::ToString;

    use super::*;

    #[test]
    fn every_error_has_a_message() {
        let upgrade = BuildingId::from_index(21).unwrap();
        let errors = [
            TownCommandError::UnknownTown,
            TownCommandError::NotOwner,
            TownCommandError::UnknownBuilding,
            TownCommandError::Built,
            TownCommandError::Forbidden,
            TownCommandError::Grail,
            TownCommandError::Requirements,
            TownCommandError::Capitol,
            TownCommandError::OfferedAs(upgrade),
            TownCommandError::BuiltToday,
            TownCommandError::NotRecruiting,
            TownCommandError::NotStanding,
            TownCommandError::NothingSummoned,
        ];
        for error in errors {
            assert!(!error.to_string().is_empty(), "{error:?}");
        }
        assert_eq!(
            TownCommandError::OfferedAs(upgrade).to_string(),
            "the hall offers it as building 21"
        );
    }

    #[test]
    fn a_build_names_its_town_and_building() {
        // covers: TWN-BLD-001
        let building = BuildingId::from_index(1).unwrap();
        let command = TownCommand::Build {
            town: TownId::new(2),
            building,
        };
        let TownCommand::Build { town, building: b } = command else {
            panic!("{command:?}");
        };
        assert_eq!((town, b), (TownId::new(2), building));
        let recruit = TownCommand::Recruit {
            town: TownId::new(3),
            building,
        };
        assert_ne!(recruit, command);
    }
}
