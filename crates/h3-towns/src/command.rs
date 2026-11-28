//! The town context's command and its errors (ADR-0028 point 6, E09 decision 45; E09 design §27,
//! §29): `h3-game` wraps [`TownCommand`] as its `GameCommand::Town`, checks the turn, the town and
//! its owner, asks [`crate::check_build`] for the town's part (the hall's tests and the price),
//! checks the treasury, then pays and builds through [`crate::Towns::build`]; since E09 T27 a
//! recruitment opens the hire dialog of what a building sells ([`crate::recruit_source`]); since
//! E09 T28a a hero moves between the town's two bars ([`TownHeroChange`], the rules the adventure
//! context's, whose records the garrison hero and the visitor are, ADR-0028 point 2); since E09
//! T38a1 a tavern hero is hired at the town's Tavern ([`crate::check_hire`], TWN-TAV-003); since
//! E09 T49a2 the town screen is opened, its heroes visiting the Mana Vortex and learning the
//! guild's spells (TWN-GLD-006); since E09 T58a1 the market window opens with the town as its
//! place ([`crate::check_market`], TWN-MKT-002); since E09 T59a a war machine is bought at the
//! Blacksmith or the Ballista Yard ([`crate::check_war_machine`], TWN-SMT-001) and a boat built at
//! the Shipyard ([`crate::check_boat`], TWN-SHY-001); since E09 T59b1 the Magic University opens
//! for the hero the command names ([`crate::check_university`], TWN-UNI-001); since E09 T59b2 a
//! stack of one of the town's armies is transformed at the Skeleton Transformer
//! ([`crate::check_transformer`], TWN-TRF-001). A refused command changes nothing.

use core::fmt;

use h3_contracts::{HeroId, TownId};

use crate::building::BuildingId;

/// A command to the town context, issued by a player (E09 decision 45). Increment 3's recruitment
/// and increment 5's services add their variants; exhaustive, as the game's other commands, so
/// that the game and the wire match every variant (SRV-Q25; E09 T18b1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
// rule: TWN-BLD-001, TWN-REC-001, TWN-ROW-002, TWN-ROW-003, TWN-ROW-004, TWN-TAV-003
// rule: TWN-GLD-006, TWN-MKT-002, TWN-SMT-001, TWN-SHY-001, TWN-UNI-001, TWN-TRF-001
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
    /// Moves a hero between the town's two bars, the town the player's (TWN-ROW-002 to -004; E09
    /// T28a, design §53: the original's town screen's portraits, `town_garrison_click`'s cases 7
    /// to 9): no visit, no movement points, no field trusted but the town's id.
    Heroes {
        /// The town.
        town: TownId,
        /// The move.
        change: TownHeroChange,
    },
    /// Hires the player's tavern hero in `slot` at `town`'s Tavern, the town the player's, for the
    /// part's price (TWN-TAV-003; E09 T38a1, design §64: the original's tavern dialog from the
    /// town screen, `town_tavern_hire` 0x5C12E0): the hero onto the town's entrance as its visitor,
    /// the slot refilled (TWN-TAV-004). No field trusted but the town's id and the slot.
    Hire {
        /// The town.
        town: TownId,
        /// The player's tavern slot: 0 or 1.
        slot: u8,
    },
    /// Opens the town screen of `town`, the player's (TWN-GLD-006; E09 T49a2, design §90,
    /// decisions 170, 175, 178: the original's town screen from the town list, `town_screen_init`
    /// 0x5C63C0 visiting the buildings, then its bars and its guild window teaching): the town's
    /// Mana Vortex to its garrison hero, then its visitor (TWN-GLD-009), then both learning the
    /// guild's spells, the visitor first (TWN-GLD-005). No visit, no movement points, no field
    /// trusted but the town's id; an opening that changes nothing is accepted and changes nothing.
    Open {
        /// The town.
        town: TownId,
    },
    /// Buys a spell book at `town`'s Mage Guild, the town the player's, for its visitor, else its
    /// garrison hero, for the part's price (TWN-GLD-007; E09 T49b, design §91, decision 176: the
    /// original's guild window, `mage_guild_open` 0x5CE560, offering the book as it opens): the
    /// book given to the hero, then the town's heroes learning the guild's spells (TWN-GLD-005).
    /// No visit, no movement points, no field trusted but the town's id.
    BuySpellBook {
        /// The town.
        town: TownId,
    },
    /// Opens the market window of `town`, the player's, with the town as its place (TWN-MKT-002;
    /// E09 T58a1, design §104, decision 209: the original's Marketplace, Artifact Merchants or
    /// Freelancer's Guild clicked on the town screen, `market_window_run` 0x5EA130): E08's market
    /// decision, offering what the town allows ([`crate::check_market`]) to the window's hero,
    /// the game's (the visitor; with `TWN-QUIRK-15`'s flag off the garrison hero too; none at a
    /// Marketplace without one), at the trader's count ([`crate::marketplaces`], TWN-MKT-001). No
    /// building named, no visit, no movement points, no field trusted but the town's id.
    Market {
        /// The town.
        town: TownId,
    },
    /// Buys the war machine that `building` of `town`, the player's, sells (TWN-SMT-001; E09 T59a,
    /// design §110, decision 211: the original's Blacksmith, or the Stronghold's Ballista Yard,
    /// clicked on the town screen, `town_blacksmith_buy` 0x5D1D30, its Buy-or-Cancel dialog the
    /// command itself) for the town's hero, the game's (the visitor; with `TWN-QUIRK-15`'s flag off
    /// the garrison hero too), at the price of the machine's creature ([`crate::check_war_machine`]):
    /// the machine's artifact given to the hero. No visit, no movement points, no mark, no field
    /// trusted but the two ids.
    BuyWarMachine {
        /// The town.
        town: TownId,
        /// The building, of the town's faction's record.
        building: BuildingId,
    },
    /// Builds a boat at `town`'s Shipyard, the town the player's, for the part's price
    /// (TWN-SHY-001; E09 T59a, design §111: the original's Shipyard clicked on the town screen,
    /// `town_screen_proc`'s case 6 0x5D36B4 with `create_boat` 0x4BB250; [`crate::check_boat`]):
    /// an empty boat of the part's type at the town's spot (TWN-SET-011), its builder the player.
    /// No hero, no visit, no mark, no field trusted but the town's id.
    BuildBoat {
        /// The town.
        town: TownId,
    },
    /// Opens the Magic University of `town`, the player's, for `hero` (TWN-UNI-001; E09 T59b1,
    /// design §112, decision 210: the original's Conflux building 21 clicked on the town screen,
    /// `town_university_open` 0x5D2DA0 running the map's University dialog): E08's University
    /// decision (GAM-DEC-006) with the town as its place, the part's skills and price
    /// ([`crate::check_university`]). `hero` is the town's visitor or its garrison hero, the
    /// player's (decision 207: the original's student, the visitor unless a stack of the upper
    /// bar is selected, the command's choice). No visit, no movement points, no mark, no field
    /// trusted but the two ids.
    University {
        /// The town.
        town: TownId,
        /// The student: the town's visitor or garrison hero.
        hero: HeroId,
    },
    /// Transforms the stack in `slot` of `army` at `town`'s Skeleton Transformer, the town the
    /// player's (TWN-TRF-001; E09 T59b2, design §113: the original's Necropolis building 22
    /// clicked on the town screen, `town_skeleton_transformer_open` 0x5D2F90 with its dialog, a
    /// stack moved to the holding side, `transformer_convert` 0x566C20, the close): the stack's
    /// creature becomes its target ([`crate::TransformerSale::target`]), its count kept, free.
    /// One command one whole stack (`TWN-QUIRK-23`, T-26 fixed by construction: no holding army
    /// to lose). No hero
    /// needed for [`TownArmy::Garrison`], no visit, no movement points, no mark, any number a
    /// turn; no field trusted but the town's id, the army and the slot.
    Transform {
        /// The town.
        town: TownId,
        /// The army: the visitor's or the upper bar's.
        army: TownArmy,
        /// The army's slot, from 0.
        slot: u8,
    },
}

/// Which of a town's armies a transformation names (TWN-TRF-001; E09 T59b2, design §113,
/// decision 207: the original's default, the visitor's army else the upper bar's, and its
/// selection override, a stack of the upper bar selected, both become the command's choice;
/// survey `E09-trades.md` §7.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TownArmy {
    /// The army of the player's hero on the town's entrance.
    Visitor,
    /// The upper bar's: the garrison hero's army where one stands, else the town's garrison
    /// (`HireTarget::Garrison`'s meaning; no hero needed).
    Garrison,
}

/// A hero's move between a town's two bars (TWN-ROW-002 to -004; E09 T28a, design §53).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TownHeroChange {
    /// The visitor into the garrison, the town's garrison joining its army (TWN-ROW-002; the
    /// original's `town_visitor_to_garrison` 0x4B9FC0 with `army_absorb` 0x44B620).
    Enter,
    /// The garrison hero out onto the entrance as the visitor (TWN-ROW-003;
    /// `town_garrison_to_visitor` 0x5BE390).
    Leave,
    /// The visitor and the garrison hero change places, each with its army (TWN-ROW-004;
    /// `town_swap_heroes` 0x5BE450).
    Swap,
}

/// Why a town command was refused; the state is unchanged. The game gives the first two
/// ([`Self::UnknownTown`], [`Self::NotOwner`]); for a build [`crate::check_build`] the others from
/// [`Self::UnknownBuilding`] to [`Self::BuiltToday`], the first that applies in the order they are
/// listed (E09 design §27: the hall's texts' order, the day's mark last); for a recruitment
/// [`crate::recruit_source`] [`Self::UnknownBuilding`], then [`Self::NotRecruiting`] to
/// [`Self::NothingSummoned`] (E09 T27); for a hero's move the adventure's rules
/// [`Self::GarrisonHeld`] to [`Self::EmptyArmy`] (E09 T28a, TWN-ROW-002 to -004); for a hire
/// [`crate::check_hire`] [`Self::NoTavern`], then the game [`Self::UnknownSlot`],
/// [`Self::EmptySlot`] (E09 T38a1); for a spell book's purchase [`crate::check_spell_book`]
/// [`Self::NoGuild`], then the game [`Self::NoHero`], [`Self::HasSpellBook`] (E09 T49b,
/// TWN-GLD-007); for a market's window [`crate::check_market`] [`Self::NoMarket`], then
/// [`Self::NoVisitor`] (E09 T58a1, TWN-MKT-002); for a war machine's purchase
/// [`crate::check_war_machine`] [`Self::UnknownBuilding`], [`Self::NoWarMachine`],
/// [`Self::NotStanding`], then the game [`Self::NoVisitor`], [`Self::HasWarMachine`] (E09 T59a,
/// TWN-SMT-001); for a boat [`crate::check_boat`] [`Self::NotStanding`], then the game
/// [`Self::BoatSpotTaken`] (E09 T59a, TWN-SHY-001); for the University [`crate::check_university`]
/// [`Self::NoUniversity`], then the game [`Self::NotTownHero`] (E09 T59b1, TWN-UNI-001); for a
/// transformation [`crate::check_transformer`] [`Self::NoTransformer`], then the game
/// [`Self::NoVisitor`], [`Self::UnknownSlot`], [`Self::EmptySlot`], [`Self::NothingToTransform`]
/// (E09 T59b2, TWN-TRF-001).
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
    /// A hero stands in the town's garrison already (TWN-ROW-002).
    GarrisonHeld,
    /// No hero of the player's stands on the town's entrance (TWN-ROW-002, -004; an ally's
    /// visitor moves no hero, TD-17).
    NoVisitor,
    /// The garrison's stacks find no slot in the visitor's army, even after merging the visitor's
    /// own stacks of one creature (TWN-ROW-002: the original's GENRLTXT 276).
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
    /// The town has no standing Tavern (TWN-TAV-003: the Tavern, or Castle's Brotherhood of the
    /// Sword over it).
    NoTavern,
    /// The slot is not one of the player's two tavern slots (TWN-TAV-003); since E09 T59b2 not
    /// one of an army's slots (TWN-TRF-001: the adventure part's army slots).
    UnknownSlot,
    /// The slot holds no hero (TWN-TAV-003: no type was free at its last fill); since E09 T59b2
    /// the army's slot holds no stack (TWN-TRF-001).
    EmptySlot,
    /// No Mage Guild stands in the town (TWN-GLD-007: a building of role `guild`, any level), or
    /// the ruleset has no spell book to sell.
    NoGuild,
    /// No hero stands in the town: neither a visitor nor a garrison hero (TWN-GLD-007).
    NoHero,
    /// The town's hero (its visitor, else its garrison hero) wears a spell book (TWN-GLD-007).
    HasSpellBook,
    /// No Marketplace, Artifact Merchants or Freelancer's Guild stands in the town (TWN-MKT-002: a
    /// building of role `marketplace`, `artifact_merchants` or `freelancers`).
    NoMarket,
    /// The building sells no war machine: it is no Blacksmith nor Ballista Yard (TWN-SMT-001).
    NoWarMachine,
    /// The town's hero holds the building's war machine already, worn or carried (TWN-SMT-001: the
    /// original's Buy disabled), or has no free position for it (defensive: a war machine's
    /// position takes it alone in the original packs).
    HasWarMachine,
    /// An empty boat or a hero stands on the town's spot (TWN-SHY-001, `TWN-QUIRK-19`: the
    /// original builds over it), or the town has none (defensive: a Shipyard stands only where
    /// the town has a spot, TWN-SET-011).
    BoatSpotTaken,
    /// No Magic University stands in the town (TWN-UNI-001: a building of role `university`).
    NoUniversity,
    /// The hero is neither the town's visitor nor its garrison hero, or not the player's
    /// (TWN-UNI-001).
    NotTownHero,
    /// No Skeleton Transformer stands in the town (TWN-TRF-001: a building of role
    /// `transformer`).
    NoTransformer,
    /// The stack's creature is its own target (TWN-TRF-001: the original's Skeleton, which keeps
    /// Convert disabled, survey `E09-trades.md` §7.5).
    NothingToTransform,
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
            Self::GarrisonHeld => f.write_str("a hero stands in the town's garrison"),
            Self::NoVisitor => f.write_str("no hero of the player's visits the town"),
            Self::ArmiesDoNotFit => f.write_str("the two armies do not fit in one"),
            Self::TooManyCreatures => f.write_str("a stack would hold too many creatures"),
            Self::NoGarrisonHero => f.write_str("no hero stands in the town's garrison"),
            Self::EntranceHeld => f.write_str("a hero stands on the town's entrance"),
            Self::TooManyHeroes => f.write_str("the player has the most heroes on the map"),
            Self::EmptyArmy => f.write_str("the garrison hero's army is empty"),
            Self::NoTavern => f.write_str("the town has no standing Tavern"),
            Self::UnknownSlot => f.write_str("no such slot"),
            Self::EmptySlot => f.write_str("the slot is empty"),
            Self::NoGuild => f.write_str("no mage guild sells a spell book in the town"),
            Self::NoHero => f.write_str("no hero stands in the town"),
            Self::HasSpellBook => f.write_str("the town's hero has a spell book"),
            Self::NoMarket => f.write_str("no market of any kind stands in the town"),
            Self::NoWarMachine => f.write_str("the building sells no war machine"),
            Self::HasWarMachine => f.write_str("the town's hero has the war machine"),
            Self::BoatSpotTaken => f.write_str("the town's boat spot is taken"),
            Self::NoUniversity => f.write_str("no Magic University stands in the town"),
            Self::NotTownHero => f.write_str("the hero is not the player's hero in the town"),
            Self::NoTransformer => f.write_str("no Skeleton Transformer stands in the town"),
            Self::NothingToTransform => f.write_str("the stack is its own transformation"),
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
            TownCommandError::GarrisonHeld,
            TownCommandError::NoVisitor,
            TownCommandError::ArmiesDoNotFit,
            TownCommandError::TooManyCreatures,
            TownCommandError::NoGarrisonHero,
            TownCommandError::EntranceHeld,
            TownCommandError::TooManyHeroes,
            TownCommandError::EmptyArmy,
            TownCommandError::NoTavern,
            TownCommandError::UnknownSlot,
            TownCommandError::EmptySlot,
            TownCommandError::NoGuild,
            TownCommandError::NoHero,
            TownCommandError::HasSpellBook,
            TownCommandError::NoMarket,
            TownCommandError::NoWarMachine,
            TownCommandError::HasWarMachine,
            TownCommandError::BoatSpotTaken,
            TownCommandError::NoUniversity,
            TownCommandError::NotTownHero,
            TownCommandError::NoTransformer,
            TownCommandError::NothingToTransform,
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
        assert_eq!(
            command,
            TownCommand::Build {
                town: TownId::new(2),
                building
            }
        );
        let recruit = TownCommand::Recruit {
            town: TownId::new(3),
            building,
        };
        assert_ne!(recruit, command);
        // covers: TWN-TAV-003
        let hire = TownCommand::Hire {
            town: TownId::new(2),
            slot: 1,
        };
        assert_ne!(hire, command);
        // covers: TWN-GLD-007
        let book = TownCommand::BuySpellBook {
            town: TownId::new(2),
        };
        assert_ne!(
            book,
            TownCommand::Open {
                town: TownId::new(2)
            }
        );
        // covers: TWN-MKT-002
        let market = TownCommand::Market {
            town: TownId::new(2),
        };
        assert_ne!(market, book);
        // covers: TWN-SMT-001
        let machine = TownCommand::BuyWarMachine {
            town: TownId::new(2),
            building,
        };
        assert_ne!(machine, command);
        // covers: TWN-SHY-001
        let boat = TownCommand::BuildBoat {
            town: TownId::new(2),
        };
        assert_ne!(boat, market);
        // covers: TWN-UNI-001
        let university = TownCommand::University {
            town: TownId::new(2),
            hero: HeroId::new(3),
        };
        assert_ne!(university, boat);
        // covers: TWN-TRF-001
        let transform = TownCommand::Transform {
            town: TownId::new(2),
            army: TownArmy::Garrison,
            slot: 6,
        };
        assert_ne!(transform, university);
        assert_ne!(
            transform,
            TownCommand::Transform {
                town: TownId::new(2),
                army: TownArmy::Visitor,
                slot: 6,
            }
        );
        assert!(TownArmy::Visitor < TownArmy::Garrison);
    }
}
