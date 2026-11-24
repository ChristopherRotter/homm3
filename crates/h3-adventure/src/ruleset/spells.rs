//! The adventure spells' data (E08 T142a; increment 15's design note §3, ADR-0006 tier 2;
//! ADV-SPL-001): which core spells act on the adventure map, each by a closed behaviour
//! ([`AdventureSpell`]: the behaviour is the engine's, the binding the packs'), and the parameters
//! of their effects (`[params.spells]`). What the core already holds of a spell (its level, its
//! schools, its cost and effect column by mastery: Summon Boat's and Scuttle Boat's chance,
//! Dimension Door's casts a day) is read through `h3_content`, never copied. A spell of the core's
//! adventure category without a binding is not castable on the map (the four information spells
//! until E12, `ADV-STUB-19`; Town Portal until E09 increment 7, `ADV-STUB-20`).

use alloc::collections::BTreeMap;
use alloc::string::String;
use core::num::NonZeroU16;

use h3_content::{ContentKey, SpellCategory, SpellId};
use h3_kernel::{CanonicalHash, StateHasher};

use super::AdventureCompileError;
use super::objects::Scope;

/// What a spell cast on the adventure map does (ADV-SPL-003 to -009; increment 15's design note
/// §10): the five behaviours the original's `adventure_spell` (0x41C490) switches on, bound to
/// core spells by the packs' `[[spell]]` entries. In the original's spell order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AdventureSpell {
    /// Brings or makes a boat beside the caster (ADV-SPL-003; the original's spell 0).
    SummonBoat,
    /// Sinks an empty boat in a window around the caster (ADV-SPL-004; spell 1).
    ScuttleBoat,
    /// Sets the caster's flying level for the day (ADV-SPL-005; spell 6).
    Fly,
    /// Sets the caster's water-walking level for the day (ADV-SPL-006; spell 7).
    WaterWalk,
    /// Moves the caster to a tile in a window around it (ADV-SPL-007 to -009; spell 8).
    DimensionDoor,
}

impl AdventureSpell {
    /// All five, in the original's order.
    pub const ALL: [Self; 5] = [
        Self::SummonBoat,
        Self::ScuttleBoat,
        Self::Fly,
        Self::WaterWalk,
        Self::DimensionDoor,
    ];

    /// The behaviour's key in the packs (`effect = "..."`).
    pub const fn key(self) -> &'static str {
        match self {
            Self::SummonBoat => "summon_boat",
            Self::ScuttleBoat => "scuttle_boat",
            Self::Fly => "fly",
            Self::WaterWalk => "water_walk",
            Self::DimensionDoor => "dimension_door",
        }
    }

    /// The behaviour of `key` in the packs, if it names one.
    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|spell| spell.key() == key)
    }

    /// Whether a cast names a target tile (increment 15's design note §14): Scuttle Boat's boat
    /// and Dimension Door's arrival; the other three act around the caster.
    pub const fn targets(self) -> bool {
        matches!(self, Self::ScuttleBoat | Self::DimensionDoor)
    }

    /// Whether the behaviour reads `[params.spells]` (Summon Boat's most boats, the window of
    /// Scuttle Boat and Dimension Door, the door's points).
    const fn needs_params(self) -> bool {
        !matches!(self, Self::Fly | Self::WaterWalk)
    }

    const fn code(self) -> u8 {
        match self {
            Self::SummonBoat => 0,
            Self::ScuttleBoat => 1,
            Self::Fly => 2,
            Self::WaterWalk => 3,
            Self::DimensionDoor => 4,
        }
    }
}

/// A binding in a pack (`[[spell]]`; increment 15's design note §3): the core spell `spell`, of
/// the core's adventure category, acts on the map as `effect`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdventureSpellDef {
    /// The core spell.
    pub spell: ContentKey,
    /// What its cast on the map does.
    pub effect: AdventureSpell,
}

/// The adventure spells' parameters (`[params.spells]`; increment 15's design note §3): required
/// when the part binds Summon Boat, Scuttle Boat or Dimension Door.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpellParams {
    /// The movement points a Dimension Door costs by mastery, 0 to 3 (ADV-SPL-009): the
    /// original's 300, 300, 300 and 200 (`adventure_spell_cast` 0x41D090; survey
    /// `E08-spells.md` §8.1).
    pub door_points: [u32; 4],
    /// The window of Dimension Door's and Scuttle Boat's targets around the caster, at most this
    /// many columns and rows away (ADV-SPL-004, ADV-SPL-008; D-76, A-54, decision 401): 9 and 8.
    /// Each from 1.
    pub target_range: [u8; 2],
    /// The most boats in use at which Summon Boat still makes one (ADV-SPL-003; the original's
    /// boat table's 64).
    pub boats_most: NonZeroU16,
}

impl SpellParams {
    pub(super) fn check(&self) -> Option<&'static str> {
        self.target_range
            .contains(&0)
            .then_some("a spells target_range entry is 0")
    }
}

/// The bindings being compiled, by core spell (each bound once).
pub(super) type Bindings = BTreeMap<SpellId, AdventureSpell>;

/// Compiles a pack's bindings into `bindings` (E08 T142a): each spell in the pack's closure, a
/// core spell of the adventure category, bound once in the whole part, each behaviour once.
pub(super) fn compile_spells(
    scope: &Scope<'_>,
    defs: &[AdventureSpellDef],
    bindings: &mut Bindings,
) -> Result<(), AdventureCompileError> {
    for def in defs {
        scope.reach(&def.spell)?;
        let key = || String::from(def.spell.as_str());
        let spell = scope
            .core
            .spell_by_key(def.spell.as_str())
            .ok_or_else(|| AdventureCompileError::UnknownSpell(key()))?;
        let invalid = |reason| AdventureCompileError::InvalidSpell { key: key(), reason };
        if scope
            .core
            .spell(spell)
            .is_none_or(|s| s.category != SpellCategory::Adventure)
        {
            return Err(invalid("not of the adventure category"));
        }
        if bindings.values().any(|&effect| effect == def.effect) {
            return Err(invalid("its effect is bound to another spell"));
        }
        if bindings.insert(spell, def.effect).is_some() {
            return Err(invalid("bound twice"));
        }
    }
    Ok(())
}

/// Checks the bindings against the parameters once every pack is in: a behaviour that reads
/// `[params.spells]` needs them.
pub(super) fn check_spells(
    bindings: &Bindings,
    params: Option<&SpellParams>,
) -> Result<(), AdventureCompileError> {
    if params.is_none() && bindings.values().any(|effect| effect.needs_params()) {
        return Err(AdventureCompileError::InvalidParams(
            "a bound Summon Boat, Scuttle Boat or Dimension Door needs [params.spells]",
        ));
    }
    Ok(())
}

/// Hashes the bindings and the parameters (the part's hash v26).
pub(super) fn hash_spells(
    h: &mut StateHasher,
    bindings: &[(SpellId, AdventureSpell)],
    params: Option<&SpellParams>,
) {
    h.write_len(bindings.len());
    for &(spell, effect) in bindings {
        spell.hash_into(h);
        h.write_u8(effect.code());
    }
    match params {
        None => h.write_u8(0),
        Some(params) => {
            h.write_u8(1);
            for points in params.door_points {
                h.write_u32(points);
            }
            for range in params.target_range {
                h.write_u8(range);
            }
            h.write_u16(params.boats_most.get());
        }
    }
}
