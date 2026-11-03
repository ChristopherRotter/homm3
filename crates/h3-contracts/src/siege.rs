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

/// The town's moat as the battle meets it. The enabled packs decide it, not a game version (ADR-0023
/// point 2, E09 decision 14; BAT-Q41's rule): from a Citadel the kind of the town's faction's town
/// record in the town part, below it `Absent`. The base pack's records give RoE's and AB's moats
/// (harmless, a Stronghold none), the SoD pack overrides them with SoD's (damaging, a Stronghold's
/// too); the original's own version from its scenario dialog is E09 decision 14's recorded
/// deviation. A Tower's land mines take its moat's place, and the same value decides their damage
/// (E04c T5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Moat {
    /// No moat: below a Citadel, or a faction's town record without one (the base pack's
    /// Stronghold).
    Absent,
    /// A moat that does no damage (the base and AB packs' records).
    Harmless,
    /// A moat that damages a stack entering it (the SoD pack's records).
    Damaging,
}

/// The town of a battle fought at a town ([`crate::BattleSetup::siege`]): derived facts, not the
/// town's buildings, but those that change its defender's morale and luck, named for viewers
/// (ADR-0023 point 2). Public.
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
    /// The town's buildings that change the defender's morale and luck (the Tavern, the
    /// Brotherhood of the Sword, the Fountain of Fortune), each with its change: summed, a side
    /// modifier like a hero's (BAT-SIE-025). Each names its building for viewers (CLT-Q40,
    /// BAT-Q64; public, as the original's and VCMI's descriptions list them).
    pub bonuses: TownBonuses,
}

impl Siege {
    /// The town's morale for its defender: its buildings' changes summed.
    pub fn morale(&self) -> i32 {
        self.bonuses.iter().map(|b| i32::from(b.morale)).sum()
    }

    /// The town's luck for its defender: its buildings' changes summed.
    pub fn luck(&self) -> i32 {
        self.bonuses.iter().map(|b| i32::from(b.luck)).sum()
    }
}

/// Most entries in [`Siege::bonuses`].
pub const MAX_TOWN_BONUSES: usize = 8;

/// A siege town's building that changes its defender's morale or luck (BAT-SIE-025).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TownBonus {
    /// The building: its index in the town's faction's town record (`h3_towns::BuildingId`),
    /// for viewers (CLT-Q40): no battle rule reads it.
    pub building: u8,
    /// Its morale change.
    pub morale: i8,
    /// Its luck change.
    pub luck: i8,
}

/// Up to [`MAX_TOWN_BONUSES`] town bonuses in a fixed array, so that [`Siege`] stays `Copy` (as
/// [`crate::FactionGroup`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TownBonuses {
    /// Number of bonuses in use.
    len: u8,
    /// The bonuses, then default ones.
    items: [TownBonus; MAX_TOWN_BONUSES],
}

impl TownBonuses {
    /// No bonus.
    pub const NONE: Self = Self {
        len: 0,
        items: [TownBonus {
            building: 0,
            morale: 0,
            luck: 0,
        }; MAX_TOWN_BONUSES],
    };

    /// The bonuses `bonuses`, in the order given, or `None` if they are more than
    /// [`MAX_TOWN_BONUSES`].
    pub fn new(bonuses: &[TownBonus]) -> Option<Self> {
        let mut items = [TownBonus::default(); MAX_TOWN_BONUSES];
        items.get_mut(..bonuses.len())?.copy_from_slice(bonuses);
        Some(Self {
            // At most `MAX_TOWN_BONUSES`, which fits.
            len: u8::try_from(bonuses.len()).unwrap_or(u8::MAX),
            items,
        })
    }

    /// The bonuses, in the order given.
    pub fn as_slice(&self) -> &[TownBonus] {
        self.items.get(..usize::from(self.len)).unwrap_or_default()
    }

    /// The bonuses, in the order given.
    pub fn iter(&self) -> core::slice::Iter<'_, TownBonus> {
        self.as_slice().iter()
    }
}
