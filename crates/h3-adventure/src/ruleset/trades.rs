//! The trades' and services' kinds (E08 T132a2; ADR-0016 point 4, increment 14's design note §1,
//! §3 to §5; survey `docs/rules/surveys/E08-trades.md`): what a market (the Trading Posts, the
//! Freelancer's Guild, the Black Market), an Altar of Sacrifice, a Hill Fort and a War Machine
//! Factory hold as data, the factions whose heroes sacrifice at an Altar, and their compilation
//! from the packs' keys. The rates are [`crate::MarketParams`]' (ADV-MKT-002, the part's
//! `[params.market]`); a Black Market's stock is drawn since E08 T133a1 (ADV-MKT-006); the
//! markets' visits came with T133, the Altar's with T134a (`crate::state`'s `altar`), the Hill
//! Fort's and the factory's come with T134b, so a step onto one of those is refused as any unbuilt
//! visit's until then.

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;
use alloc::vec::Vec;
use core::num::NonZeroU32;

use h3_content::{ArtifactClass, ArtifactTypeId, ContentKey, CreatureTypeId, FactionId, Ruleset};
use h3_kernel::{CanonicalHash, StateHasher};

use super::rewards::{Compiled, hash_classes};
use super::{AdventureCompileError, DWELLING_LEVELS, ObjectKind, ObjectType, Scope};
use crate::command::{ALTAR_BACKPACK_PLACES, ALTAR_SLOTS};

/// A screen of a market's window (ADV-MKT-001; survey §3): the original's screens 0, 1, 4 and
/// 2. The town's artifact sale (screen 3) is the town lane's to add with its Marketplace.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MarketScreen {
    /// Resources for resources (screen 0; a Trading Post's).
    Trade,
    /// Resources to another player, 1 : 1 (screen 1; a Trading Post's).
    Transfer,
    /// Creatures for resources (screen 4; the Freelancer's Guild's).
    Sell,
    /// Resources for an artifact of the market's stock (screen 2; the Black Market's).
    Buy,
}

/// A market's type (increment 14's design note §1; ADV-MKT-001): one kind for the three markets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Market {
    /// The screens its window shows: at least one, distinct.
    pub screens: Vec<MarketScreen>,
    /// The Marketplaces it trades as (the rates' row): from 1 to the market parameters' counts;
    /// every map object 5 (survey §3).
    pub count: u8,
    /// The artifact classes a Black Market's stock is drawn in, in order (survey §7.1): present
    /// exactly when [`MarketScreen::Buy`] is among the screens, none [`ArtifactClass::Special`],
    /// at most [`MAX_STOCK`].
    pub stock: Vec<ArtifactClass>,
}

/// The most entries a market's stock holds (E08 T133a1): a bit each in the visits' block's sold
/// entries (increment 14's design note §8); the original's seven.
pub const MAX_STOCK: usize = 8;

impl Market {
    /// Whether its window shows `screen`.
    pub fn shows(&self, screen: MarketScreen) -> bool {
        self.screens.contains(&screen)
    }
}

/// An Altar of Sacrifice's type (increment 14's design note §3; ADV-ALT-001 to -003).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Altar {
    /// The experience an artifact gives by its class, in [`ArtifactClass`]'s order (treasure
    /// 1 000, minor 1 500, major 3 000, relic 6 000, special 0: `altar_artifact_entry`
    /// 0x55FC30's immediates).
    pub experience: [u32; 5],
    /// The artifacts never put on the altar (the Spell Book, the Catapult: `altar_worn_click`
    /// 0x5632A0), distinct.
    pub keeps: Vec<ArtifactTypeId>,
    /// The most artifacts a sacrifice takes, from 1 (23: the altar's 22 places and the one held,
    /// `altar_slot_row_counts` 0x6412E8).
    pub places: u8,
    /// A creature's experience is `creature_factor × ⌊AI value / creature_divisor⌋` (5 and 40:
    /// `altar_set_creature_amount` 0x564C00).
    pub creature_factor: u32,
    /// The divisor of a creature's AI value, from 1.
    pub creature_divisor: NonZeroU32,
}

impl Altar {
    /// The experience an artifact of `class` gives.
    pub const fn artifact_experience(&self, class: ArtifactClass) -> u32 {
        self.experience[class_index(class)]
    }
}

/// [`Altar`] as a pack defines it: its kept artifacts by key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AltarDef {
    /// [`Altar::experience`].
    pub experience: [u32; 5],
    /// Core artifacts, distinct.
    pub keeps: Vec<ContentKey>,
    /// From 1.
    pub places: u8,
    /// [`Altar::creature_factor`].
    pub creature_factor: u32,
    /// From 1.
    pub creature_divisor: u32,
}

/// What a faction's heroes sacrifice at an Altar of Sacrifice (ADV-ALT-001; the hero's class's
/// faction, `altar_dialog_create` 0x55FDD0): artifacts, creatures or both.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AltarScreens {
    /// Whether its heroes sacrifice artifacts.
    pub artifacts: bool,
    /// Whether its heroes sacrifice creatures.
    pub creatures: bool,
}

/// A faction's [`AltarScreens`] in a pack (`[[altar_faction]]`; E08 T132a2): each pack names its
/// own factions' (the base pack the eight of RoE, the ab pack Conflux), as the random dwellings'
/// tables. With an Altar type in the part every core faction has one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AltarFactionDef {
    /// A core faction, once in the whole part.
    pub faction: ContentKey,
    /// Whether its heroes sacrifice artifacts.
    pub artifacts: bool,
    /// Whether its heroes sacrifice creatures; one of the two at least.
    pub creatures: bool,
}

/// A Hill Fort's type (increment 14's design note §4; ADV-HFT-001, -002): the percent of an
/// upgrade's gold difference it charges by creature level 1 to [`DWELLING_LEVELS`] (0, 25, 50,
/// 75, 100, 100, 100: `hill_fort_gold_factors` 0x63EB4C as percents), each a multiple of 25 to
/// 100, so the original's binary32 quarter times a gold below 2^24, truncated, is the percent's
/// integer share (a level of 0 % free whole). Which stacks upgrade is the enabled packs' (E08
/// decision 378: the creatures with an upgrade).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HillFort {
    /// The percents by level, from level 1.
    pub gold: [u8; DWELLING_LEVELS],
}

/// A War Machine Factory's type (increment 14's design note §5; ADV-WMF-001).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WarMachineFactory {
    /// The war machines it sells in the dialog's order, each with the one artifact a purchase
    /// gives (the base pack's Ballista, First Aid Tent and Ammo Cart): at least one, distinct.
    pub machines: Vec<(CreatureTypeId, ArtifactTypeId)>,
}

/// [`WarMachineFactory`] as a pack defines it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WarMachineFactoryDef {
    /// Core war machine creatures, each with exactly one artifact in the core, distinct.
    pub machines: Vec<ContentKey>,
}

/// A market's checks; its count against the parameters' counts comes with them
/// ([`check_trades`]).
// rule: ADV-MKT-001
pub(super) fn compile_market(def: &Market) -> Result<Market, &'static str> {
    let distinct = def.screens.iter().collect::<BTreeSet<_>>().len() == def.screens.len();
    if def.screens.is_empty() || !distinct {
        Err("a market's screens are none or name one twice")
    } else if def.count == 0 {
        Err("a market's count is 0")
    } else if def.shows(MarketScreen::Buy) == def.stock.is_empty() {
        Err("a market's stock is not exactly a buying one's")
    } else if def.stock.contains(&ArtifactClass::Special) {
        Err("a market's stock names the special class")
    } else if def.stock.len() > MAX_STOCK {
        Err("a market's stock holds more than 8 entries")
    } else {
        Ok(def.clone())
    }
}

/// An Altar's checks.
// rule: ADV-ALT-002, ADV-ALT-003
pub(super) fn compile_altar(scope: &Scope<'_>, def: &AltarDef) -> Compiled<Altar> {
    let keeps = def
        .keeps
        .iter()
        .map(|key| scope.artifact(key))
        .collect::<Result<Vec<_>, _>>()?;
    let distinct = keeps.iter().collect::<BTreeSet<_>>().len() == keeps.len();
    Ok(if !distinct {
        Err("an altar keeps an artifact twice")
    } else if def.places == 0 {
        Err("an altar takes no artifact")
    } else if let Some(creature_divisor) = NonZeroU32::new(def.creature_divisor) {
        Ok(Altar {
            experience: def.experience,
            keeps,
            places: def.places,
            creature_factor: def.creature_factor,
            creature_divisor,
        })
    } else {
        Err("an altar's creature divisor is 0")
    })
}

/// A Hill Fort's checks.
// rule: ADV-HFT-002
pub(super) fn compile_hill_fort(def: &HillFort) -> Result<HillFort, &'static str> {
    if def.gold.iter().any(|&p| p > 100 || p % 25 != 0) {
        Err("a Hill Fort's gold percent is above 100 or not a multiple of 25")
    } else {
        Ok(*def)
    }
}

/// A War Machine Factory's checks: each machine's one artifact found in the core.
// rule: ADV-WMF-001
pub(super) fn compile_factory(
    scope: &Scope<'_>,
    def: &WarMachineFactoryDef,
) -> Compiled<WarMachineFactory> {
    let creatures = def
        .machines
        .iter()
        .map(|key| scope.creature(key))
        .collect::<Result<Vec<_>, _>>()?;
    let distinct = creatures.iter().collect::<BTreeSet<_>>().len() == creatures.len();
    if creatures.is_empty() || !distinct {
        return Ok(Err("a factory sells no machine or one twice"));
    }
    let mut machines = Vec::with_capacity(creatures.len());
    for creature in creatures {
        let mut artifacts = (0..scope.core.artifacts().len())
            .filter_map(ArtifactTypeId::from_index)
            .filter(|&id| {
                scope
                    .core
                    .artifact(id)
                    .is_some_and(|a| a.war_machine == Some(creature))
            });
        match (
            scope.war_machine(creature),
            artifacts.next(),
            artifacts.next(),
        ) {
            (true, Some(artifact), None) => machines.push((creature, artifact)),
            _ => {
                return Ok(Err(
                    "a factory's machine is not a war machine with one artifact",
                ));
            }
        }
    }
    Ok(Ok(WarMachineFactory { machines }))
}

/// One pack's [`AltarFactionDef`] into the part's table.
// rule: ADV-ALT-001
pub(super) fn compile_altar_faction(
    scope: &Scope<'_>,
    def: &AltarFactionDef,
    table: &mut BTreeMap<FactionId, AltarScreens>,
) -> Result<(), AdventureCompileError> {
    let name = || String::from(def.faction.as_str());
    scope.reach(&def.faction)?;
    let faction = scope
        .core
        .faction_by_key(def.faction.as_str())
        .ok_or_else(|| AdventureCompileError::UnknownFaction(name()))?;
    if !def.artifacts && !def.creatures {
        return Err(AdventureCompileError::AltarFaction {
            faction: name(),
            reason: "its heroes sacrifice nothing",
        });
    }
    let screens = AltarScreens {
        artifacts: def.artifacts,
        creatures: def.creatures,
    };
    if table.insert(faction, screens).is_some() {
        return Err(AdventureCompileError::AltarFaction {
            faction: name(),
            reason: "it has a second entry",
        });
    }
    Ok(())
}

/// The trades' checks once every pack and the parameters are in: a market's count within the
/// market parameters' counts (none: no market), and with an Altar every core faction's entry; since
/// E08 T134a (decision 386) an Altar only with a core backpack of at most
/// [`ALTAR_BACKPACK_PLACES`] and at most [`ALTAR_SLOTS`] army slots, the most its answer names.
// rule: ADV-MKT-001, ADV-ALT-001
pub(super) fn check_trades(
    core: &Ruleset,
    (types, army_slots): (&[ObjectType], u8),
    counts: Option<u8>,
    altar_factions: &BTreeMap<FactionId, AltarScreens>,
) -> Result<(), AdventureCompileError> {
    for ty in types {
        if let ObjectKind::Market(market) = &ty.kind {
            let reason = match counts {
                None => "a market without [params.market]",
                Some(counts) if market.count > counts => {
                    "a market's count is past [params.market]'s rows"
                }
                Some(_) => continue,
            };
            return Err(AdventureCompileError::InvalidObjectType {
                key: String::from(ty.key.as_str()),
                reason,
            });
        }
    }
    let altar = types
        .iter()
        .find(|ty| matches!(ty.kind, ObjectKind::Altar(_)));
    if let Some(ty) = altar {
        let backpack = core.hero_params().map_or(0, |params| params.backpack);
        let reason = if usize::from(backpack) > ALTAR_BACKPACK_PLACES {
            "an altar under a backpack of more than 64"
        } else if usize::from(army_slots) > ALTAR_SLOTS {
            "an altar under more than 7 army slots"
        } else {
            ""
        };
        if !reason.is_empty() {
            return Err(AdventureCompileError::InvalidObjectType {
                key: String::from(ty.key.as_str()),
                reason,
            });
        }
    }
    let altar = altar.is_some();
    if let Some(missing) = (0..core.factions().len())
        .filter_map(FactionId::from_index)
        .find(|f| altar && !altar_factions.contains_key(f))
    {
        return Err(AdventureCompileError::AltarFaction {
            faction: core
                .factions()
                .get(missing.index())
                .map_or_else(String::new, |f| String::from(f.key.as_str())),
            reason: "it has no entry, and the part has an altar",
        });
    }
    Ok(())
}

/// The index of `class` in [`Altar::experience`].
const fn class_index(class: ArtifactClass) -> usize {
    match class {
        ArtifactClass::Treasure => 0,
        ArtifactClass::Minor => 1,
        ArtifactClass::Major => 2,
        ArtifactClass::Relic => 3,
        ArtifactClass::Special => 4,
    }
}

impl CanonicalHash for Market {
    /// The screens as length and each (0 trade, 1 transfer, 2 sell, 3 buy), the count, the stock's
    /// classes as length and each class.
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_len(self.screens.len());
        for screen in &self.screens {
            h.write_u8(match screen {
                MarketScreen::Trade => 0,
                MarketScreen::Transfer => 1,
                MarketScreen::Sell => 2,
                MarketScreen::Buy => 3,
            });
        }
        h.write_u8(self.count);
        hash_classes(&self.stock, h);
    }
}

impl CanonicalHash for Altar {
    /// The five classes' experience, the kept artifacts as length and handles, the places, the
    /// creature's factor and divisor.
    fn hash_into(&self, h: &mut StateHasher) {
        self.experience.iter().for_each(|&e| h.write_u32(e));
        h.write_len(self.keeps.len());
        self.keeps.iter().for_each(|a| a.hash_into(h));
        h.write_u8(self.places);
        h.write_u32(self.creature_factor);
        h.write_u32(self.creature_divisor.get());
    }
}

impl CanonicalHash for WarMachineFactory {
    /// The machines as length and each creature and artifact.
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_len(self.machines.len());
        for (creature, artifact) in &self.machines {
            creature.hash_into(h);
            artifact.hash_into(h);
        }
    }
}
