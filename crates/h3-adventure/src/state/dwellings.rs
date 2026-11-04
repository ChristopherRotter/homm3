//! The dwellings and the Refugee Camps (E08 T94a, T94b; increment 10's design note §5, §6, §9):
//! each random dwelling's draw among its outcomes in the static map's placement walk, the names its
//! outcomes take, and the tiles a hero may not stand on beside one, whatever it draws (T94a); each
//! dwelling's week as it stands, its counts and guards, and each camp's creature and count, with
//! their reads, checks and part of the state hash (T94b). The week's step and the first week's
//! draws are `calendar.rs`' and `economy.rs`'.

use alloc::borrow::Cow;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use h3_content::{CreatureTypeId, FactionId, Ruleset};
use h3_contracts::ArmyStack;
use h3_kernel::{CanonicalHash, Rng, StateHasher};

use super::{AdventureState, StaticMap, pick};
use crate::ruleset::{AdventureRuleset, MAX_DWELLING_CREATURES, ObjectKind, ObjectTypeId};
use crate::setup::{Holds, ObjectSetup, Pos, TileMask};

/// The most creatures a dwelling has, and so the most stacks its guards hold (ADV-DWL-001: one to
/// four creatures, the guards a stack of each of level 5 or more, in slots 0 up): the part's own
/// bound, so the two cannot diverge.
const SLOTS: usize = MAX_DWELLING_CREATURES;

/// A dwelling's week as it stands (ADV-DWL-002, ADV-DWL-003; E08 T94b, increment 10's design note
/// §6): each creature's count, and its guards by slot (the kind's full guards, a battle's
/// survivors, or none once beaten). Inline and `Copy`, so a write copies the block without an
/// allocation per dwelling (99 on the largest shipped map).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DwellingWeek {
    /// Each creature's count, by the kind's creature order; 0 past its creatures.
    counts: [u32; SLOTS],
    /// How many creatures the kind has.
    creatures: u8,
    /// The guards by slot; none for an empty slot.
    guards: [Option<ArmyStack>; SLOTS],
}

impl DwellingWeek {
    /// A week of these counts, by creature order, and these guards, each in its slot: counts past
    /// [`SLOTS`] and a stack of a slot past them are left out (a dwelling's never are: ADV-DWL-001
    /// bounds its creatures, and so its guards, to four).
    fn new(
        counts: impl IntoIterator<Item = u32>,
        guards: impl IntoIterator<Item = ArmyStack>,
    ) -> Self {
        let mut week = Self {
            counts: [0; SLOTS],
            creatures: 0,
            guards: [None; SLOTS],
        };
        for (cell, count) in week.counts.iter_mut().zip(counts) {
            *cell = count;
            week.creatures += 1;
        }
        for stack in guards {
            if let Some(cell) = week.guards.get_mut(usize::from(stack.slot)) {
                *cell = Some(stack);
            }
        }
        week
    }

    /// Each creature's count, by the kind's creature order.
    pub(crate) fn counts(&self) -> &[u32] {
        &self.counts[..usize::from(self.creatures)]
    }

    /// The guards' stacks, by slot.
    pub(crate) fn guards(&self) -> impl Iterator<Item = &ArmyStack> {
        self.guards.iter().flatten()
    }

    /// Whether it fits `full`, its dwelling's week as it begins: as many creatures, each count at
    /// most the full one, each guards' stack in a slot of the full guards, of its creature, with 1
    /// to its full count (a battle's survivors; ADV-DWL-002).
    fn within(&self, full: &Self) -> bool {
        self.creatures == full.creatures
            && self
                .counts
                .iter()
                .zip(&full.counts)
                .all(|(count, most)| count <= most)
            && self.guards.iter().zip(&full.guards).all(|pair| match pair {
                (None, _) => true,
                (Some(stack), Some(most)) => {
                    stack.slot == most.slot
                        && stack.creature == most.creature
                        && (1..=most.count).contains(&stack.count)
                }
                (Some(_), None) => false,
            })
    }
}

impl CanonicalHash for DwellingWeek {
    /// The counts as their number and each; the guards as their number of stacks and each stack by
    /// slot: slot, creature, count.
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_len(self.counts().len());
        for &count in self.counts() {
            h.write_u32(count);
        }
        h.write_len(self.guards().count());
        for stack in self.guards() {
            h.write_u8(stack.slot);
            stack.creature.hash_into(h);
            h.write_u32(stack.count);
        }
    }
}

/// The week of a dwelling of `kind` as it begins (ADV-DWL-002, ADV-DWL-003; E08 T94b, increment
/// 10's design note §6; survey `E08-dwellings.md` §2.2, §6.1, the original's `dwelling_week_refill`
/// 0x4B8A60): each creature's count its growth, the guards the kind's full guards (derived when the
/// part compiles, ADV-DWL-001), one stack each in creature order, slots 0 up. None for every other
/// kind.
// rule: ADV-DWL-002, ADV-DWL-003
pub(super) fn full_week(core: &Ruleset, kind: Option<&ObjectKind>) -> Option<DwellingWeek> {
    let Some(ObjectKind::Dwelling {
        creatures, guards, ..
    }) = kind
    else {
        return None;
    };
    let counts = creatures.iter().map(|&creature| growth(core, creature));
    let guards = (0..)
        .zip(guards)
        .map(|(slot, &(creature, count))| ArmyStack {
            slot,
            creature,
            count,
        });
    Some(DwellingWeek::new(counts, guards))
}

/// A creature's growth (ADV-DWL-002, ADV-RFG-001): its core info's, 0 for a creature the core does
/// not have (never: a dwelling's and a random monster's are the core's).
fn growth(core: &Ruleset, creature: CreatureTypeId) -> u32 {
    core.creature(creature)
        .map_or(0, |info| u32::from(info.growth))
}

/// A Refugee Camp's week as it stands (ADV-RFG-001; E08 T94b, increment 10's design note §6): its
/// creature and the count left (a purchase lowers it, T95).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Camp {
    /// The week's creature.
    pub(crate) creature: CreatureTypeId,
    /// How many are left to hire.
    pub(crate) count: u32,
}

/// A camp's week drawn (ADV-RFG-001, ADV-MON-003; E08 T94b, increment 10's design note §9; survey
/// `E08-dwellings.md` §6.2, §7, the original's `random_creature_by_level` 0x4C92C0): its creature
/// uniform among the random monsters of its kind's `levels` ([`AdventureRuleset::random_monsters`],
/// in the core's order), its count that creature's growth. None for levels no creature has, which
/// the part refuses for a camp's kind.
// rule: ADV-RFG-001, ADV-MON-003
pub(super) fn draw_camp(
    core: &Ruleset,
    part: &AdventureRuleset,
    levels: [u8; 2],
    draws: &mut Rng,
) -> Option<Camp> {
    let pool: Vec<CreatureTypeId> = part.random_monsters(core, levels).collect();
    pick(&pool, draws).map(|&creature| Camp {
        creature,
        count: growth(core, creature),
    })
}

/// A dwelling type a random dwelling may become (increment 10's design note §5): the type of one
/// level of one faction's table, its footprint and its anchor's shift.
#[derive(Clone, Copy)]
struct Outcome {
    /// The dwelling type.
    object_type: ObjectTypeId,
    /// Its template's impassable cells.
    blocked: TileMask,
    /// Its template's entrances.
    visitable: TileMask,
    /// How far right the random dwelling's anchor moves ([`crate::ObjectKind::Dwelling`]'s
    /// `shift`).
    shift: u8,
}

impl Outcome {
    /// The random dwelling's `anchor` moved right by the shift: the original's anchor, which may
    /// lie one column past the map's right edge (the static map ignores a cell off the map as it
    /// does past the top and left). Saturating: an anchor at the last column a position holds has
    /// every cell off any map either way.
    const fn anchor(self, anchor: Pos) -> Pos {
        Pos::new(
            anchor.x.saturating_add(self.shift as u16),
            anchor.y,
            anchor.level,
        )
    }

    /// The tiles of `map` it covers from the random dwelling's `anchor`: its blocked and visitable
    /// cells from the moved anchor.
    fn tiles(self, map: &StaticMap, anchor: Pos) -> impl Iterator<Item = Pos> + use<'_> {
        let anchor = self.anchor(anchor);
        TileMask::from_bits(self.blocked.bits() | self.visitable.bits())
            .offsets()
            .filter_map(move |(column, row)| map.offset_pos(anchor, column, row))
    }
}

/// The type of `level` (1-based) in `faction`'s table and its footprint, if the faction has a
/// table and the level is in it. The compiler makes every table's type a dwelling
/// (`compile_random_dwelling`); a type that is not one gives none.
fn outcome(part: &AdventureRuleset, faction: FactionId, level: u32) -> Option<Outcome> {
    let at = usize::try_from(level).unwrap_or(0).wrapping_sub(1);
    part.random_dwelling(faction)
        .and_then(|row| row.get(at))
        .and_then(|&object_type| {
            part.object_type(object_type)
                .and_then(|ty| ty.kind.dwelling_footprint())
                .map(|(blocked, visitable, shift)| Outcome {
                    object_type,
                    blocked,
                    visitable,
                    shift,
                })
        })
}

/// Every outcome of what a random dwelling holds, faction by faction in the set's order, level by
/// level from the lowest; none for anything else.
fn outcomes<'a>(
    part: &'a AdventureRuleset,
    holds: &'a Holds,
) -> impl Iterator<Item = Outcome> + 'a {
    let (factions, (low, high)) = match holds {
        Holds::RandomDwelling { factions, levels } => (factions.as_slice(), *levels),
        _ => (&[][..], (1, 0)),
    };
    factions.iter().flat_map(move |&faction| {
        (u32::from(low)..=u32::from(high)).filter_map(move |level| outcome(part, faction, level))
    })
}

/// `object` as the game places it (ADV-DWL-005; increment 10's design note §5): a random dwelling
/// (the setup gives one, and only one, [`Holds::RandomDwelling`], checked by `check_holds`) draws a
/// faction uniform among its set, then a level uniform in its range, and becomes the faction's
/// table's dwelling type of that level, as a random resource becomes its drawn pile type
/// (ADV-PIK-002): that type, the type's footprint as its masks, its anchor moved right by the
/// type's shift, holding nothing and drawn on no further tile, as a placed dwelling; its owner and
/// scenery stay the setup's. Nothing remembers it was random. Every other object as it is, with no
/// draw. Called in the placement walk, in object order, before every other draw of the static map;
/// E09's random towns, when built, draw their placeholders before it (ADR-0017 point 4).
// rule: ADV-DWL-005
pub(super) fn resolve<'a>(
    part: &AdventureRuleset,
    object: &'a ObjectSetup,
    draws: &mut Rng,
) -> Cow<'a, ObjectSetup> {
    let Holds::RandomDwelling {
        factions,
        levels: (low, high),
    } = &object.holds
    else {
        return Cow::Borrowed(object);
    };
    let faction = pick(factions, draws).copied();
    let level = draws.range_inclusive(u32::from(*low), u32::from(*high));
    let drawn = faction.and_then(|faction| outcome(part, faction, level));
    Cow::Owned(drawn.map_or(object.clone(), |outcome| ObjectSetup {
        anchor: outcome.anchor(object.anchor),
        blocked: outcome.blocked,
        visitable: outcome.visitable,
        object_type: Some(outcome.object_type),
        drawn: TileMask::default(),
        holds: Holds::Nothing,
        ..object.clone()
    }))
}

/// The names `object` may take in play, a player's type and anchor (ADR-0016 point 6), for the
/// static map's name check (`SetupError::SharedAnchor`): a typed object's own unless it is a
/// `ground` (nothing names one); a random dwelling's, every outcome's type and moved anchor, so
/// that the drawn one is checked and whether the setup is valid does not depend on the draw (an
/// agent's call, E08 T94a, as [`refused_tiles`]).
// rule: ADV-DWL-005
pub(super) fn names<'a>(
    part: &'a AdventureRuleset,
    object: &'a ObjectSetup,
    ground: bool,
) -> impl Iterator<Item = (u16, Pos)> + 'a {
    let random = matches!(object.holds, Holds::RandomDwelling { .. });
    let own = object
        .object_type
        .filter(|_| !ground && !random)
        .map(|id| (id.raw(), object.anchor));
    own.into_iter().chain(
        outcomes(part, &object.holds)
            .map(|outcome| (outcome.object_type.raw(), outcome.anchor(object.anchor))),
    )
}

/// The tiles of `map` no hero of the setup may stand on, each with the first random dwelling of
/// `objects` (the setup's, unresolved) that refuses it (ADV-DWL-005; increment 10's design note §5,
/// an agent's call): a random dwelling's visitable tiles, and every tile one of its outcomes covers
/// from its moved anchor, for each faction of its set and each level of its range. Whatever the
/// draw, a hero there could stand inside the dwelling, so the setup is refused before the hero's
/// tile is checked on the resolved map, and whether it is valid does not depend on the draw.
// rule: ADV-DWL-005
pub(super) fn refused_tiles(
    part: &AdventureRuleset,
    map: &StaticMap,
    objects: &[ObjectSetup],
) -> BTreeMap<Pos, usize> {
    let mut refused = BTreeMap::new();
    for (index, object) in objects.iter().enumerate() {
        let random = matches!(object.holds, Holds::RandomDwelling { .. });
        let entrances = object
            .visitable
            .offsets()
            .filter(|_| random)
            .filter_map(|(column, row)| map.offset_pos(object.anchor, column, row));
        let covered =
            outcomes(part, &object.holds).flat_map(|outcome| outcome.tiles(map, object.anchor));
        for pos in entrances.chain(covered) {
            refused.entry(pos).or_insert(index);
        }
    }
    refused
}

impl AdventureState {
    /// The week of the dwelling at object index `index` as it stands (E08 T94b): its entry of the
    /// week's block, else its week as it began; none for an object that is no dwelling.
    fn dwelling_week(&self, index: usize) -> Option<&DwellingWeek> {
        let weeks = self.dwellings.as_deref().unwrap_or(&self.map.full_weeks);
        self.map
            .dwelling_index(index)
            .and_then(|place| weeks.get(place))
    }

    /// The counts of the dwelling at object index `index` as they stand (ADV-DWL-002,
    /// ADV-DWL-003; E08 T94b), by its kind's creature order: each creature's growth from the
    /// build and each week's step until a purchase lowers it (T95). None for an object that is no
    /// dwelling. Never shown to a viewer (ADV-OWN-003).
    // rule: ADV-DWL-002, ADV-DWL-003
    pub fn dwelling_counts(&self, index: usize) -> Option<&[u32]> {
        self.dwelling_week(index).map(DwellingWeek::counts)
    }

    /// The guards of the dwelling at object index `index` as they stand (ADV-DWL-002,
    /// ADV-DWL-003; E08 T94b), by slot: its kind's full guards from the build and each week's
    /// step, a battle's survivors, none once beaten (T95). None for a dwelling without guards
    /// (every level-1 to -4 one) and for an object that is no dwelling. Never shown to a viewer
    /// (ADV-OWN-003).
    // rule: ADV-DWL-002, ADV-DWL-003
    pub fn dwelling_guards(&self, index: usize) -> Vec<ArmyStack> {
        self.dwelling_week(index)
            .map(|week| week.guards().copied().collect())
            .unwrap_or_default()
    }

    /// The creature and the count of the Refugee Camp at object index `index` as they stand
    /// (ADV-RFG-001; E08 T94b): drawn when the game is built and at each week's step, the count
    /// lowered by a purchase (T95). None for an object that is no camp. Never shown to a viewer
    /// (ADV-OWN-003).
    // rule: ADV-RFG-001
    pub fn camp(&self, index: usize) -> Option<(CreatureTypeId, u32)> {
        self.camps
            .as_deref()
            .zip(self.map.camp_index(index))
            .and_then(|(camps, place)| camps.get(place))
            .map(|camp| (camp.creature, camp.count))
    }

    /// The dwellings' invariants (E08 T94b; increment 10's design note §6), which every command
    /// keeps: no week's block while every dwelling is as its week began, else an entry for each
    /// dwelling, one of them changed (the field's representation: equality reads it); each count
    /// at most its growth, each guards' stack in a slot of the full guards, of its creature, with
    /// 1 to its full count.
    pub fn check_dwellings(&self) -> bool {
        let full = &self.map.full_weeks;
        self.dwellings.as_deref().is_none_or(|weeks| {
            weeks.len() == full.len()
                && weeks != &full[..]
                && weeks.iter().zip(full).all(|(week, full)| week.within(full))
        })
    }

    /// The camps' invariants (E08 T94b; increment 10's design note §6), which every command keeps:
    /// a creature and a count for each camp, none for a map without camps; each creature one of
    /// its kind's levels' random monsters (ADV-MON-003), its count at most its growth.
    pub fn check_camps(&self, core: &Ruleset, part: &AdventureRuleset) -> bool {
        let Some(camps) = self.camps.as_deref() else {
            return self.map.camps.is_empty();
        };
        camps.len() == self.map.camps.len()
            && camps
                .iter()
                .zip(&self.map.camp_levels)
                .all(|(camp, &levels)| {
                    part.random_monsters(core, levels)
                        .any(|creature| creature == camp.creature)
                        && camp.count <= growth(core, camp.creature)
                })
    }

    /// The dwellings' and the camps' part of the state hash (since adventure state hash v20, E08
    /// T94b): the dwellings as their number and each one's week as it stands ([`DwellingWeek`]'s
    /// encoding: its counts, then its guards), so no block and a block of full entries hash
    /// alike; then the camps as 0 for none, else 1, their number and each one's creature and
    /// count.
    pub(super) fn hash_dwellings(&self, h: &mut StateHasher) {
        let weeks = self.dwellings.as_deref().unwrap_or(&self.map.full_weeks);
        h.write_len(weeks.len());
        for week in weeks {
            week.hash_into(h);
        }
        match self.camps.as_deref() {
            None => h.write_u8(0),
            Some(camps) => {
                h.write_u8(1);
                h.write_len(camps.len());
                for camp in camps {
                    camp.creature.hash_into(h);
                    h.write_u32(camp.count);
                }
            }
        }
    }
}

#[cfg(test)]
impl AdventureState {
    /// Writes the counts and the guards of the dwelling at object index `index` (tests of the
    /// dwellings' state and hash, E08 T94b, until T95's visits write them), keeping the field's
    /// representation: the first write makes the block (every entry full, then the change), a
    /// later one copies it, and a write that leaves every dwelling as its week began drops it.
    pub(crate) fn set_dwelling(&mut self, index: usize, counts: &[u32], guards: &[ArmyStack]) {
        let full = &self.map.full_weeks;
        let mut weeks = self.dwellings.as_deref().unwrap_or(full).to_vec();
        let place = self.map.dwelling_index(index).expect("a dwelling");
        weeks[place] = DwellingWeek::new(counts.iter().copied(), guards.iter().copied());
        self.dwellings = (weeks[..] != full[..]).then(|| weeks.into());
    }

    /// Writes the dwellings' block as it is, normalised or not (tests of the invariants and the
    /// hash, E08 T94b): every dwelling's week as it began.
    pub(crate) fn set_full_dwellings_block(&mut self) {
        self.dwellings = Some(self.map.full_weeks.clone().into());
    }

    /// Writes the creature and the count of the camp at object index `index` (tests of the
    /// camps' invariants and hash, E08 T94b, until T95's purchases lower the count).
    pub(crate) fn set_camp(&mut self, index: usize, creature: CreatureTypeId, count: u32) {
        let place = self.map.camp_index(index).expect("a camp");
        let mut camps = self.camps.as_deref().unwrap_or_default().to_vec();
        camps[place] = Camp { creature, count };
        self.camps = Some(camps.into());
    }
}
