//! The screens' notice in play (E24 M5 T4c, *provisional*; `docs/epics/E24-design.md` §5.28;
//! `MCL-STUB-7`): a screen of M6 or M7 (the hero screen, the town screen, the spell book, a visit's
//! window) or a battle (M5 T6) that play does not build shows a box in the client's own words with
//! one OK ([`Notice::key`]: the client's own pack, no original form), in play alone: a recording
//! shows what the original showed, so there the click stays refused as before.
//!
//! - **A decision the view does not ask** ([`Notice::of`]): the engine's window a visit or an
//!   answer opens (a hire dialog, a University, a market, an Altar of Sacrifice, a Hill Fort, a map
//!   Tavern, a meeting, an army dialog, a garrison's exchange) and the offers the view does not
//!   build (a guarded object's or a dwelling's guards' fight: the battles, M5 T6; a Shipyard's boat;
//!   a dwelling's or a Refugee Camp's hire question): its notice, whose OK closes the decision
//!   through the host with its closing answer ([`closing`]: the offer declined, the window left or
//!   closed), so play goes on.
//! - **A screen without a decision** ([`Notice::SCREENS`]): the hero screen (the selected hero's
//!   portrait in the hero list, the hero clicked on the map), the town screen (the selected town's
//!   picture, the town clicked on the map, a hero's entry into a town), the spell book (cast a
//!   spell), the kingdom overview, the quest log and the adventure options: its OK closes it.

use h3_protocol::wire::game::{
    AltarAnswer, Answer, Consent, Decision, HillFortAnswer, HireAnswer, MarketAnswer, OfferKind,
    TavernAnswer, UniversityAnswer,
};

/// A screen, window or battle play does not build (the module's documentation), by its notice.
// stub: MCL-STUB-7 (the screens of M6 and M7 a notice in play)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Notice {
    /// The hero screen (M6).
    Hero,
    /// The town screen (M7).
    Town,
    /// The spell book (M6).
    Book,
    /// The kingdom overview.
    Kingdom,
    /// The quest log.
    Quests,
    /// The adventure options (the view of the world, the puzzle map, the dig among them).
    Adventure,
    /// A hire dialog, or a dwelling's or a Refugee Camp's hire question (GAM-DEC-008).
    Recruit,
    /// A Shipyard's boat (ADV-SHY-002).
    Shipyard,
    /// A University (GAM-DEC-006).
    University,
    /// A market (GAM-DEC-011).
    Market,
    /// An Altar of Sacrifice (GAM-DEC-012).
    Altar,
    /// A Hill Fort (GAM-DEC-013).
    HillFort,
    /// A map Tavern (GAM-DEC-010).
    Tavern,
    /// A meeting of two heroes (GAM-DEC-003).
    Meeting,
    /// An army dialog (GAM-DEC-004: creatures with no room, a mine's guards left).
    Army,
    /// A map garrison's exchange (GAM-DEC-009).
    Garrison,
    /// A battle a question offers (a guarded object's, a mine's, a dwelling's guards': M5 T6).
    Battle,
}

/// The string catalogue's keys the notices show (the map client's own texts, `clients/text/map/`),
/// in [`Notice::ALL`]'s order.
pub const KEYS: [&str; 17] = [
    "map.stub.hero",
    "map.stub.town",
    "map.stub.book",
    "map.stub.kingdom",
    "map.stub.quests",
    "map.stub.adventure",
    "map.stub.recruit",
    "map.stub.shipyard",
    "map.stub.university",
    "map.stub.market",
    "map.stub.altar",
    "map.stub.hill-fort",
    "map.stub.tavern",
    "map.stub.meeting",
    "map.stub.army",
    "map.stub.garrison",
    "map.stub.battle",
];

impl Notice {
    /// Every notice, in its keys' order ([`KEYS`]).
    pub const ALL: [Self; 17] = [
        Self::Hero,
        Self::Town,
        Self::Book,
        Self::Kingdom,
        Self::Quests,
        Self::Adventure,
        Self::Recruit,
        Self::Shipyard,
        Self::University,
        Self::Market,
        Self::Altar,
        Self::HillFort,
        Self::Tavern,
        Self::Meeting,
        Self::Army,
        Self::Garrison,
        Self::Battle,
    ];

    /// The screens without a decision (the module's documentation): their notices are laid out
    /// for a live view at its start (`crate::reach::Hosted::notices`).
    pub const SCREENS: [Self; 6] = [
        Self::Hero,
        Self::Town,
        Self::Book,
        Self::Kingdom,
        Self::Quests,
        Self::Adventure,
    ];

    /// Its key in the string catalogue.
    #[must_use]
    pub const fn key(self) -> &'static str {
        KEYS[self as usize]
    }

    /// The notice of `decision` where the view does not ask it: an offer of a fight (a guarded
    /// object's, a mine's, a dwelling's guards'), a Shipyard's boat or a hire question, and every
    /// window but a level-up's; none for a question or choice the view asks
    /// (`crate::engine::visits::opened`) and a level-up (its dialog's).
    #[must_use]
    pub const fn of(decision: &Decision) -> Option<Self> {
        Some(match decision {
            Decision::Offer { offer } => match offer.kind {
                OfferKind::Fight {} | OfferKind::Guarded { .. } => Self::Battle,
                OfferKind::Boat { .. } => Self::Shipyard,
                OfferKind::Hire { .. } => Self::Recruit,
                _ => return None,
            },
            Decision::LevelUp { .. } | Decision::Choice { .. } => return None,
            Decision::Meeting { .. } => Self::Meeting,
            Decision::ArmyDialog { .. } => Self::Army,
            Decision::University { .. } => Self::University,
            Decision::Hire { .. } => Self::Recruit,
            Decision::Exchange { .. } => Self::Garrison,
            Decision::Tavern { .. } => Self::Tavern,
            Decision::Market { .. } => Self::Market,
            Decision::Altar { .. } => Self::Altar,
            Decision::HillFort { .. } => Self::HillFort,
        })
    }
}

/// The answer that closes `decision` where the view shows its notice ([`Notice::of`]): an offer
/// declined (its fight not fought, its boat not built, its creatures not hired), a University, a
/// market, an Altar and a Hill Fort left, a hire dialog and a map Tavern closed, a meeting, an army
/// dialog and a garrison's exchange closed (`Close`: the group's rest lost, a mine's guards kept);
/// none for a decision the view asks.
#[must_use]
pub const fn closing(decision: &Decision) -> Option<Answer> {
    if Notice::of(decision).is_none() {
        return None;
    }
    Some(match decision {
        Decision::University { .. } => Answer::University {
            answer: UniversityAnswer::Leave {},
        },
        Decision::Hire { .. } => Answer::Hire {
            answer: HireAnswer::Close {},
        },
        Decision::Tavern { .. } => Answer::Tavern {
            answer: TavernAnswer::Close {},
        },
        Decision::Market { .. } => Answer::Market {
            answer: MarketAnswer::Leave {},
        },
        Decision::Altar { .. } => Answer::Altar {
            answer: AltarAnswer::Leave {},
        },
        Decision::HillFort { .. } => Answer::HillFort {
            answer: HillFortAnswer::Leave {},
        },
        Decision::Meeting { .. } | Decision::ArmyDialog { .. } | Decision::Exchange { .. } => {
            Answer::Close {}
        }
        _ => Answer::Offer {
            consent: Consent::Decline,
        },
    })
}

#[cfg(test)]
mod tests;
