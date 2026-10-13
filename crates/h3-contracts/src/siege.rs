//! A battle at a town (E04c, ADR-0023): the facts the battle takes from the town, derived before
//! the seam by whoever starts the battle (the town context, E09; until then scenarios and the
//! CLI), as the battlefield is (ADR-0022).

use h3_content::FactionId;

/// The town's fort level, the highest of Fort, Citadel and Castle it has built (the original's
/// +0x132F4; `docs/formats/installation.md` §7.27).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FortLevel {
    /// No fort: the battle is fought on an ordinary battlefield, but still at the town.
    None,
    /// A Fort: the walls and the gate.
    Fort,
    /// A Citadel: also the keep and the moat.
    Citadel,
    /// A Castle: also the upper and lower towers.
    Castle,
}

/// The town's moat as the battle meets it. The game's version decides it (the original's
/// `mapKind`: a game under RoE or AB rules keeps their moat, survey S-29, not a quirk; BAT-Q41):
/// before SoD a moat does no damage and a Stronghold has none. A Tower's land mines take its
/// moat's place, and the same value decides their damage (E04c T5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Moat {
    /// No moat: below a Citadel, or a Stronghold in a game under RoE or AB rules.
    Absent,
    /// A moat that does no damage (a game under RoE or AB rules).
    Harmless,
    /// A moat that damages a stack entering it (a game under SoD rules).
    Damaging,
}

/// The town of a battle fought at a town ([`crate::BattleSetup::siege`]): derived facts, not the
/// town's buildings (ADR-0023 point 2). Public.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Siege {
    /// The town's faction, a handle of the battle's ruleset: the battle part's siege record for
    /// it gives the walls', the moat's and the turrets' details.
    pub faction: FactionId,
    /// The town's fort level.
    pub fort: FortLevel,
    /// The town's moat.
    pub moat: Moat,
    /// The town's built buildings that strengthen its turrets, counted as the original counts
    /// them (0x5BF4F0: the building ids 0–43 the town has built among its mask +0x160). The keep's
    /// creature count follows from it (4 + this in the original): a battle rule (E04c T7).
    pub structures: u8,
    /// The town has an Escape Tunnel: its defender may retreat.
    pub escape_tunnel: bool,
    /// The town's morale bonus to the defender's stacks (the Tavern, the Brotherhood of the
    /// Sword), a side modifier like a hero's.
    pub morale: i8,
    /// The town's luck bonus to the defender's stacks (the Fountain of Fortune), a side modifier
    /// like a hero's.
    pub luck: i8,
}
