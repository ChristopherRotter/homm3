//! The dwellings and the Refugee Camps (E08 T94a, T94b, T95a1, T95a2; increment 10's design
//! note §5, §6, §7, §9, §12): each random dwelling's draw among its outcomes in the static map's
//! placement walk, the names its outcomes take, and the tiles a hero may not stand on beside one,
//! whatever it draws (T94a); each dwelling's week as it stands, its counts and guards, and each
//! camp's creature and count, with their reads, checks and part of the state hash (T94b); the
//! visits, the hire question and a dwelling's free creatures, with the dwellings' writer (T95a1);
//! the guards' question, their battle and its end (T95a2); the hire dialog an accepted question
//! opens, its source's stock, the room's bound and the purchase's map side (T95b). The week's step
//! and the first week's draws are `calendar.rs`' and `economy.rs`'.

use alloc::borrow::Cow;
use alloc::collections::BTreeMap;
use alloc::vec;
use alloc::vec::Vec;

use h3_content::{CreatureTypeId, FactionId, ResourceId, Ruleset};
use h3_contracts::{ArmyStack, HeroId, PlayerId};
use h3_kernel::{CanonicalHash, Rng, StateHasher};

use super::monsters::{Room, add_stack, join_at, room, room_bound};
use super::{AdventureState, StaticMap, allied, pick};
use crate::command::{
    AdventureEvent, AdventureEventKind, AdventureOutcome, Defender, HireDialog, HireError,
    HireSource, HireTarget, Offer, OfferKind, Sink, Viewers,
};
use crate::ruleset::{AdventureRuleset, Effect, MAX_DWELLING_CREATURES, ObjectKind, ObjectTypeId};
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
        week.set_guards(guards);
        week
    }

    /// Sets the guards to these stacks, each in its slot, every other slot empty (E08 T94b; T95a2:
    /// a battle's survivors, none once beaten): a stack of a slot past [`SLOTS`] is left out.
    fn set_guards(&mut self, guards: impl IntoIterator<Item = ArmyStack>) {
        self.guards = [None; SLOTS];
        for stack in guards {
            if let Some(cell) = self.guards.get_mut(usize::from(stack.slot)) {
                *cell = Some(stack);
            }
        }
    }

    /// Each creature's count, by the kind's creature order.
    pub(crate) fn counts(&self) -> &[u32] {
        &self.counts[..usize::from(self.creatures)]
    }

    /// Each creature's count, by the kind's creature order, to write
    /// ([`AdventureState::write_dwelling`]).
    pub(super) fn counts_mut(&mut self) -> &mut [u32] {
        &mut self.counts[..usize::from(self.creatures)]
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
/// level from the lowest; since E09 T11 a random dwelling tied to a town's, faction by faction of
/// every table of the part, in its order (the town's faction is the build's, so these do not
/// depend on it); none for anything else.
fn outcomes<'a>(
    part: &'a AdventureRuleset,
    holds: &'a Holds,
) -> impl Iterator<Item = Outcome> + 'a {
    let (set, tables, (low, high)) = match holds {
        Holds::RandomDwelling { factions, levels } => (factions.as_slice(), &[][..], *levels),
        Holds::LinkedDwelling { levels, .. } => (&[][..], part.random_dwellings(), *levels),
        _ => (&[][..], &[][..], (1, 0)),
    };
    let factions = set
        .iter()
        .copied()
        .chain(tables.iter().map(|&(faction, _)| faction));
    factions.flat_map(move |faction| {
        (u32::from(low)..=u32::from(high)).filter_map(move |level| outcome(part, faction, level))
    })
}

/// Whether `holds` is a random dwelling's, with a set or tied to a town (E09 T11).
const fn random(holds: &Holds) -> bool {
    matches!(
        holds,
        Holds::RandomDwelling { .. } | Holds::LinkedDwelling { .. }
    )
}

/// `object` as the game places it (ADV-DWL-005; increment 10's design note §5): a random dwelling
/// (the setup gives one, and only one, [`Holds::RandomDwelling`], checked by `check_holds`) draws a
/// faction uniform among its set, then a level uniform in its range, and becomes the faction's
/// table's dwelling type of that level, as a random resource becomes its drawn pile type
/// (ADV-PIK-002): that type, the type's footprint as its masks, its anchor moved right by the
/// type's shift, holding nothing and drawn on no further tile, as a placed dwelling; its owner and
/// scenery stay the setup's. Since E09 T11 a random dwelling tied to a town
/// ([`Holds::LinkedDwelling`]) takes its town's faction from `town_factions` (by `TownId`, the
/// setup's; the original's `random_dwellings_resolve` 0x502B60), drawing none, then its level as
/// one with a set; the static map checked that the faction has a table. Nothing remembers it was
/// random. Every other object as it is, with no draw. Called in the placement walk, in object
/// order, before every other draw of the static map; E09's random towns draw their factions before
/// this walk (ADR-0017 point 4).
// rule: ADV-DWL-005
pub(super) fn resolve<'a>(
    part: &AdventureRuleset,
    object: &'a ObjectSetup,
    town_factions: &[FactionId],
    draws: &mut Rng,
) -> Cow<'a, ObjectSetup> {
    let (faction, (low, high)) = match &object.holds {
        Holds::RandomDwelling { factions, levels } => (pick(factions, draws).copied(), *levels),
        Holds::LinkedDwelling { town, levels } => {
            (town_factions.get(town.index()).copied(), *levels)
        }
        _ => return Cow::Borrowed(object),
    };
    let level = draws.range_inclusive(u32::from(low), u32::from(high));
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
/// agent's call, E08 T94a, as [`refused_tiles`]; since E09 T11 one tied to a town's, every table's
/// outcomes, so that it does not depend on the town's faction either).
// rule: ADV-DWL-005
pub(super) fn names<'a>(
    part: &'a AdventureRuleset,
    object: &'a ObjectSetup,
    ground: bool,
) -> impl Iterator<Item = (u16, Pos)> + 'a {
    let random = random(&object.holds);
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
/// from its moved anchor, for each faction of its set (since E09 T11 of every table for one tied to a
/// town) and each level of its range. Whatever the draw, a hero there could stand inside the
/// dwelling, so the setup is refused before the hero's tile is checked on the resolved map, and
/// whether it is valid does not depend on the draw.
// rule: ADV-DWL-005
pub(super) fn refused_tiles(
    part: &AdventureRuleset,
    map: &StaticMap,
    objects: &[ObjectSetup],
) -> BTreeMap<Pos, usize> {
    let mut refused = BTreeMap::new();
    for (index, object) in objects.iter().enumerate() {
        let random = random(&object.holds);
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
    /// build and each week's step until a purchase lowers it (T95b, [`Self::hire`]). None for an object that is no
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
    /// lowered by a purchase (T95b, [`Self::hire`]). None for an object that is no camp. Never shown to a viewer
    /// (ADV-OWN-003).
    // rule: ADV-RFG-001
    pub fn camp(&self, index: usize) -> Option<(CreatureTypeId, u32)> {
        self.camps
            .as_deref()
            .zip(self.map.camp_index(index))
            .and_then(|(camps, place)| camps.get(place))
            .map(|camp| (camp.creature, camp.count))
    }

    /// Whether the state holds the dwellings' week block (E08 T94b's representation): a write left
    /// a dwelling below its week (a purchase, free creatures joined, a guards' battle) and no
    /// week's refill has filled them all since (E08 T96b2: what the `fork` benchmark's forks
    /// carry).
    pub fn dwellings_written(&self) -> bool {
        self.dwellings.is_some()
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

impl AdventureState {
    /// Writes the week of the dwelling at object index `index` as `edit` leaves it (E08 T95a1:
    /// the free creatures' counts; T95a2: the guards' battle's survivors), keeping the field's
    /// representation (increment 10's design note §6, as built): a write that changes nothing
    /// leaves the block as it is (a fork keeps sharing it), the first change makes the block
    /// (every entry full, then the change), a later one copies it, and one that leaves every
    /// dwelling as its week began drops it, so [`Self::check_dwellings`] holds after every write
    /// that keeps each count within its growth and each guards' stack within the full guards.
    /// Nothing for an object that is no dwelling.
    pub(super) fn write_dwelling(&mut self, index: usize, edit: impl FnOnce(&mut DwellingWeek)) {
        let full = &self.map.full_weeks;
        let weeks = self.dwellings.as_deref().unwrap_or(full);
        let Some(place) = self.map.dwelling_index(index) else {
            return;
        };
        let mut week = weeks[place];
        edit(&mut week);
        if week == weeks[place] {
            return;
        }
        let mut weeks = weeks.to_vec();
        weeks[place] = week;
        self.dwellings = (weeks[..] != full[..]).then(|| weeks.into());
    }

    /// The visit of the dwelling at object index `index`, of type `object_type` and with these
    /// `creatures` (its kind's, in order), by `player`'s hero `hero` on its entrance (ADV-DWL-004,
    /// ADV-OWN-001 amended; E08 T95a1, T95a2, increment 10's design note §7, §12; survey
    /// `E08-dwellings.md` §4.1 to §4.3, the original's `visit_dwelling` 0x4A18B0): one of the
    /// visitor's team (the visitor's own until E10's teams) asks the hire question at once, no
    /// flag, no guards; a neutral one or another team's whose guards stand asks the guards'
    /// question ([`OfferKind::Guarded`], [`Self::guards_question`]: the original's GENRLTXT 422),
    /// accepted their battle ([`Self::fight_for_dwelling`]); else the visit goes on at once
    /// ([`Self::enter_dwelling`]: the flag, then the hire question).
    // rule: ADV-DWL-004, ADV-OWN-001
    pub(super) fn visit_dwelling<S: Sink>(
        &mut self,
        rules: (&Ruleset, &AdventureRuleset),
        (player, hero): (PlayerId, HeroId),
        object: (usize, ObjectTypeId),
        creatures: &[CreatureTypeId],
        sink: &mut S,
    ) -> AdventureOutcome {
        if !self
            .owner(object.0)
            .is_some_and(|owner| allied(owner, player))
            && let Some(kind) = self.guards_question(rules.1, object.0)
        {
            return self.ask(hero, object, kind);
        }
        self.enter_dwelling(rules, (player, hero), object, creatures, sink)
    }

    /// The guards' question of the dwelling at object index `index` while its guards stand
    /// (ADV-DWL-004; E08 T95a2, increment 10's design note §7 step 2, §12; survey
    /// `E08-dwellings.md` §4.2, the original's GENRLTXT 422): the creature of their first occupied
    /// slot and the word of their total count ([`crate::MonsterParams::count_word`]; 0 for a part
    /// without monster parameters). VCMI reads the first stack's count: a text difference only.
    /// None while none stand, and for an object that is no dwelling.
    // rule: ADV-DWL-004
    fn guards_question(&self, part: &AdventureRuleset, index: usize) -> Option<OfferKind> {
        let mut guards = self
            .dwelling_week(index)
            .into_iter()
            .flat_map(DwellingWeek::guards);
        let first = guards.next()?;
        let total = guards.fold(first.count, |sum, stack| sum.saturating_add(stack.count));
        Some(OfferKind::Guarded {
            creature: first.creature,
            count_word: part.params().monsters().map_or(0, |m| m.count_word(total)),
        })
    }

    /// The visit's rest at the dwelling at object index `index`, of type `object_type` and with
    /// these `creatures` (its kind's, in order), for `player`'s hero `hero`, once no guards stand
    /// in its way (ADV-DWL-004, ADV-OWN-001 amended; E08 T95a1, T95a2, increment 10's design note
    /// §7): one not of the visitor's team is flagged for its owner ([`Self::flag`]:
    /// `ObjectFlagged`, then the reveal of the part's flag radius around its entrance), then the
    /// hire question ([`OfferKind::Hire`], naming its creatures in its kind's order: the
    /// original's ADVEVENT 35, or 36 for four creatures). The flag needs no answer: a declined
    /// question keeps it.
    // rule: ADV-DWL-004, ADV-OWN-001
    fn enter_dwelling<S: Sink>(
        &mut self,
        rules: (&Ruleset, &AdventureRuleset),
        (player, hero): (PlayerId, HeroId),
        object: (usize, ObjectTypeId),
        creatures: &[CreatureTypeId],
        sink: &mut S,
    ) -> AdventureOutcome {
        if !self
            .owner(object.0)
            .is_some_and(|owner| allied(owner, player))
        {
            self.flag(rules, object, player, sink);
        }
        self.ask(
            hero,
            object,
            OfferKind::Hire {
                creatures: creatures.to_vec(),
            },
        )
    }

    /// An accepted guards' question ([`OfferKind::Guarded`]) of `player`'s hero `hero` on the
    /// dwelling at object index `index`, of type `object_type` (ADV-DWL-004, GAM-BAT-001 amended;
    /// E08 T95a2, increment 10's design note §7 step 2, §12; survey `E08-dwellings.md` §4.2, the
    /// original's `visit_dwelling` 0x4A18B0), the state read anew: not of the hero's team with its
    /// guards standing, their battle, unsplit and unowned whatever its owner and however many
    /// stacks ([`Defender::Guards`]: the original's `adventure_battle` with owner −1 and no hero;
    /// VCMI's side is the owner's, D-51), fought on its entrance, where the hero stands, as any
    /// battle there
    /// ([`Self::battle_at`]: the tile's battlefield, the coast, a special ground; no type
    /// terrain); else the visit's rest ([`Self::enter_dwelling`]). Nothing when the hero is not
    /// on the map (or the type is no dwelling's).
    // rule: ADV-DWL-004, GAM-BAT-001
    pub(super) fn fight_for_dwelling<S: Sink>(
        &mut self,
        (core, part): (&Ruleset, &AdventureRuleset),
        (player, hero): (PlayerId, HeroId),
        object: (usize, ObjectTypeId),
        sink: &mut S,
    ) -> AdventureOutcome {
        let effect = part.object_type(object.1).and_then(|ty| ty.kind.effect());
        let (Some(attacker), Some(Effect::Dwelling(creatures))) = (self.hero(hero), effect) else {
            return AdventureOutcome::default();
        };
        let guards = self.dwelling_guards(object.0);
        if !self
            .owner(object.0)
            .is_some_and(|owner| allied(owner, player))
            && !guards.is_empty()
        {
            let defender = Defender::Guards { army: guards };
            return AdventureOutcome {
                battle: Some(self.battle_at(part, attacker, defender, attacker.pos)),
                ..AdventureOutcome::default()
            };
        }
        // Unreachable under sequential play (ADR-0017 point 7: one player per turn group is
        // built, and an open offer allows only its answers, so nothing changes the dwelling
        // between its question and the acceptance); kept so the infallible acceptance reads the
        // state anew, as a mine's does ([`Self::fight_for_mine`]; E08 decision 254).
        self.enter_dwelling((core, part), (player, hero), object, creatures, sink)
    }

    /// The end of the battle against the guards of the dwelling at object index `index`, fought
    /// by the visitor's hero `hero` at `tile` (ADV-DWL-004; E08 T95a2, increment 10's design note
    /// §7 step 2, §10, §12): after the visitor's side `won` the guards are gone and the visit's
    /// rest comes after the winner's levels ([`AdventureOutcome::reward_after`], GAM-DEC-007:
    /// [`Self::dwelling_won`]); else the guards' `survivors` stay as its guards, by slot as they
    /// stand (none after a draw: the next visit flags it at once). Written through
    /// [`Self::write_dwelling`]; its owner is unchanged; no event (the guards are hidden,
    /// ADV-OWN-003).
    // rule: ADV-DWL-004
    pub(super) fn end_dwelling_battle(
        &mut self,
        index: usize,
        (hero, tile): (HeroId, Pos),
        (survivors, won): (&[ArmyStack], bool),
    ) -> AdventureOutcome {
        let left = if won { &[][..] } else { survivors };
        self.write_dwelling(index, |week| week.set_guards(left.iter().copied()));
        AdventureOutcome {
            reward_after: won.then_some((hero, tile)),
            ..AdventureOutcome::default()
        }
    }

    /// The visit's rest at the dwelling at object index `index`, of type `object_type` and with
    /// these `creatures` (its kind's, in order), its guards beaten by the hero `hero`
    /// (ADV-DWL-004; E08 T95a2, increment 10's design note §7 step 2; [`Self::give_won`], after
    /// the winner's levels): for the hero's owner, the flag when it is not of its team, then the
    /// hire question ([`Self::enter_dwelling`]). Nothing when the hero is no longer on the map
    /// (or guards stand there again).
    // rule: ADV-DWL-004
    pub(super) fn dwelling_won<S: Sink>(
        &mut self,
        rules: (&Ruleset, &AdventureRuleset),
        hero: HeroId,
        object: (usize, ObjectTypeId),
        creatures: &[CreatureTypeId],
        sink: &mut S,
    ) -> AdventureOutcome {
        let beaten = self.dwelling_guards(object.0).is_empty();
        match self.hero(hero).map(|h| h.owner) {
            Some(player) if beaten => {
                self.enter_dwelling(rules, (player, hero), object, creatures, sink)
            }
            _ => AdventureOutcome::default(),
        }
    }

    /// The visit of the Refugee Camp at object index `index`, of type `object_type`, by
    /// `player`'s hero `hero` (ADV-RFG-001; E08 T95a1, increment 10's design note §7; survey
    /// `E08-dwellings.md` §7, the original's `visit_refugee_camp` 0x4A4600), any player's hero:
    /// with creatures left, the hire question naming its week's creature ([`OfferKind::Hire`]:
    /// the original's ADVEVENT 112); with none, nothing but the visitor's owner told it is
    /// deserted ([`AdventureEventKind::ObjectEmpty`]: ADVEVENT 44). No flag, no guards, no free
    /// creatures.
    // rule: ADV-RFG-001
    pub(super) fn visit_refugee_camp<S: Sink>(
        &self,
        (player, hero): (PlayerId, HeroId),
        (index, object_type): (usize, ObjectTypeId),
        sink: &mut S,
    ) -> AdventureOutcome {
        match self.camp(index) {
            Some((creature, count)) if count > 0 => self.ask(
                hero,
                (index, object_type),
                OfferKind::Hire {
                    creatures: vec![creature],
                },
            ),
            _ => {
                sink.emit(AdventureEvent {
                    kind: AdventureEventKind::ObjectEmpty {
                        object_type,
                        anchor: self.object_at(index).anchor,
                    },
                    seen_by: Viewers::NONE.with(player),
                });
                AdventureOutcome::default()
            }
        }
    }

    /// An accepted hire question `offer` for `player` on the object at object index `index`, a
    /// dwelling or a Refugee Camp (ADV-HIR-001; E08 T95a1, increment 10's design note §7 step 4,
    /// §13; survey `E08-dwellings.md` §4.4, the original's `visit_dwelling` 0x4A18B0 and
    /// `dwelling_offer_hire` 0x4ABA50): on a dwelling `player` owns when it accepts (an agent's
    /// call: the owner read at acceptance, defensive only: under sequential play nothing changes it
    /// between the question and the acceptance, E08 decision 254), its free creatures join
    /// ([`Self::give_free`]), then the hire dialog opens when its kind has a creature above the
    /// part's `free_level` (ADV-HIR-002, E08 T95b: the original's `dwelling_offer_hire` when one of
    /// level 2 or more is there; no class-17 dwelling of level 1 opens it). A camp's always opens
    /// the dialog (its question stands only with creatures left, ADV-RFG-001), nothing free. Any
    /// other dwelling's, another team's (unreachable: its visit flags it first), gives nothing; an
    /// ally's read-only dialog is E10's (design note §13: left out until its teams).
    // rule: ADV-HIR-001, ADV-HIR-002
    pub(super) fn accept_hire<S: Sink>(
        &mut self,
        (core, part): (&Ruleset, &AdventureRuleset),
        player: PlayerId,
        (index, offer): (usize, &Offer),
        sink: &mut S,
    ) -> AdventureOutcome {
        let own = self.owner(index) == Some(player);
        let free = free_level(part);
        let opens = match part
            .object_type(offer.object_type)
            .and_then(|ty| ty.kind.effect())
        {
            Some(Effect::Dwelling(creatures)) if own => {
                self.give_free((core, part), player, (index, offer), creatures, sink);
                creatures
                    .iter()
                    .any(|&creature| core.creature(creature).is_some_and(|c| c.level > free))
            }
            Some(Effect::Camp) => true,
            _ => false,
        };
        AdventureOutcome {
            hire: opens.then_some(HireDialog {
                target: HireTarget::Hero(offer.hero),
                source: HireSource::Object {
                    index,
                    object_type: offer.object_type,
                    anchor: offer.anchor,
                },
            }),
            ..AdventureOutcome::default()
        }
    }

    /// The free creatures of `player`'s dwelling at object index `index` for the hero of its
    /// accepted hire question `offer` (ADV-HIR-001, ADV-ARM-007; E08 T95a1, increment 10's
    /// design note §7 step 4; survey `E08-dwellings.md` §4.4): each of its `creatures` (its kind's,
    /// in order) of the part's `free_level` or less (its core level; 1 in the packs, the
    /// original's level 1) in turn: with a count of 0, told as none here
    /// ([`AdventureEventKind::NoneToRecruit`]: the original's GENRLTXT 423); else, where
    /// ADV-ARM-007's room is (the hero's first stack of the creature that takes the whole count
    /// within a stack's most, else its first free slot: the original's `army_add_stack`), all of
    /// it joins and its count is set to 0 ([`AdventureEventKind::CreaturesJoined`]: GENRLTXT
    /// 424); with no room none of it joins and the count is kept
    /// ([`AdventureEventKind::JoinRefused`]: GENRLTXT 426; D-52 as the original, decision 235:
    /// the hero's stacks are not merged). Each told to `player` alone. A creature above
    /// `free_level` is left to the hire dialog.
    // rule: ADV-HIR-001, ADV-ARM-007
    fn give_free<S: Sink>(
        &mut self,
        (core, part): (&Ruleset, &AdventureRuleset),
        player: PlayerId,
        (index, offer): (usize, &Offer),
        creatures: &[CreatureTypeId],
        sink: &mut S,
    ) {
        let free = free_level(part);
        let rules = (part.params().army_slots(), part.params().max_stack_count());
        let mut counts = self.dwelling_counts(index).unwrap_or_default().to_vec();
        let hero = offer.hero;
        let mut army = self
            .heroes
            .iter_mut()
            .find(|h| h.id == hero)
            .map(|h| &mut h.army);
        for (&creature, count) in creatures.iter().zip(&mut counts) {
            if !core
                .creature(creature)
                .is_some_and(|info| info.level <= free)
            {
                continue;
            }
            let kind = if *count == 0 {
                AdventureEventKind::NoneToRecruit {
                    object_type: offer.object_type,
                    anchor: offer.anchor,
                    creature,
                }
            } else if army
                .as_deref_mut()
                .is_some_and(|army| add_stack(army, rules, creature, *count))
            {
                AdventureEventKind::CreaturesJoined {
                    hero,
                    creature,
                    count: core::mem::take(count),
                }
            } else {
                AdventureEventKind::JoinRefused {
                    hero,
                    creature,
                    count: *count,
                }
            };
            sink.emit(AdventureEvent {
                kind,
                seen_by: Viewers::NONE.with(player),
            });
        }
        self.write_dwelling(index, |week| {
            week.counts_mut().copy_from_slice(&counts);
        });
    }
}

/// The part's `free_level` (ADV-HIR-001: `[params.dwellings]`, 1 in the packs), 0 for a part
/// without dwellings' parameters.
fn free_level(part: &AdventureRuleset) -> u8 {
    part.params()
        .dwellings()
        .map_or(0, |params| params.free_level)
}

/// The price of `amount` `creature`s (ADV-HIR-002; survey `E08-dwellings.md` §5.3, the original's
/// `recruit_dialog_proc` 0x550940): its core cost times the amount, saturating, for each resource
/// it costs more than 0 of, in the core's resource order (the pack lists a creature's cost in its
/// own order). A purchase's ([`HirePurchase::price`]); of 1 the cost the game's hire view shows
/// and its most divides the treasury by (GAM-DEC-008, E08 T95b).
// rule: ADV-HIR-002
pub fn hire_price(core: &Ruleset, creature: CreatureTypeId, amount: u32) -> Vec<(ResourceId, u32)> {
    let cost = core
        .creature(creature)
        .map_or(&[][..], |info| &info.cost[..]);
    let by_resource: BTreeMap<ResourceId, u32> = cost
        .iter()
        .filter(|&&(_, each)| each > 0)
        .map(|&(resource, each)| (resource, each.saturating_mul(amount)))
        .collect();
    by_resource.into_iter().collect()
}

/// A purchase in a hire dialog [`AdventureState::check_hire`] accepted (ADV-HIR-002; E08 T95b,
/// increment 10's design note §13): its price, which the game checks against the treasury and
/// pays (GAM-DEC-008, ADR-0004), and where it goes, which [`AdventureState::hire`] then makes
/// without a check of its own (the design's infallible purchase: no refusal the game could not
/// reach). It holds for the state that checked it as that state stands: nothing of the adventure
/// changes between the check and the purchase. Not `Clone`, so one check buys once; a count is
/// lowered saturating, so a purchase kept past a change of the state cannot wrap it (*agent's
/// call*, T95b's review: the game checks and buys in one command).
#[derive(Debug, PartialEq, Eq)]
pub struct HirePurchase {
    /// The price, by resource in the core's order ([`hire_price`]).
    price: Vec<(ResourceId, u32)>,
    /// The creature bought.
    creature: CreatureTypeId,
    /// How many.
    amount: u32,
    /// The source's object index.
    index: usize,
    /// The hero's place among the state's heroes.
    at: usize,
    /// The creature's place in the source's stock (its count's place).
    place: usize,
    /// Where the creatures join the hero's army.
    room: Room,
}

impl HirePurchase {
    /// The price, by resource in the core's order: the creature's core cost times the amount for
    /// each resource it costs more than 0 of ([`hire_price`]).
    pub fn price(&self) -> &[(ResourceId, u32)] {
        &self.price
    }
}

/// The purchase's map side (ADV-HIR-002, ADV-ARM-007; E08 T95b, increment 10's design note §8,
/// §13; survey `E08-dwellings.md` §5, the original's `H3RecruitMgr`): the source's stock, the
/// room's bound, the check and the purchase. The treasury is the game's (ADR-0004: the price is
/// the state's answer, nothing from the payload), as the most a player may buy, which also bounds
/// by the treasury (GAM-DEC-008).
impl AdventureState {
    /// The creatures `source` sells with their counts as they stand, in its order (ADV-HIR-002;
    /// E08 T95b, increment 10's design note §8, §13): a dwelling's kind's creatures with their
    /// week's counts ([`Self::dwelling_counts`]; distinct, ADV-DWL-001), a Refugee Camp's week's
    /// creature and count ([`Self::camp`]). None when the object at its index is not of its type
    /// and anchor, or is neither. The game's view and legal set read it at each step, as the
    /// original's dialog points at the dwelling's own counts (survey §5.1).
    // rule: ADV-HIR-002
    pub fn hire_stock(
        &self,
        part: &AdventureRuleset,
        source: &HireSource,
    ) -> Option<Vec<(CreatureTypeId, u32)>> {
        let HireSource::Object {
            index,
            object_type,
            anchor,
        } = *source;
        let effect = self
            .object(index)
            .filter(|object| object.object_type == Some(object_type) && object.anchor == anchor)
            .and_then(|_| part.object_type(object_type))
            .and_then(|ty| ty.kind.effect())?;
        match effect {
            Effect::Dwelling(creatures) => {
                let counts = self.dwelling_counts(index).unwrap_or_default();
                Some(
                    creatures
                        .iter()
                        .copied()
                        .zip(counts.iter().copied())
                        .collect(),
                )
            }
            Effect::Camp => self.camp(index).map(|camp| vec![camp]),
            _ => None,
        }
    }

    /// The most `creature`s `target`'s army has room for (ADV-ARM-007, ADV-HIR-002; E08 T95b,
    /// increment 10's design note §13): a stack's most if one of its slots is free, else the most
    /// less the smallest count among its stacks of the creature, else 0; 0 for a hero not on the
    /// map. Every amount from 1 to it finds room (a stack of the creature that takes it within the
    /// most, else a free slot: the original's `army_can_take` 0x44AC50); the game's legal set
    /// bounds the most a player may buy by it (an agent's call of design note §13: the
    /// original's slider leaves it out and refuses at Recruit, the same rule).
    // rule: ADV-ARM-007, ADV-HIR-002
    pub fn hire_room(
        &self,
        part: &AdventureRuleset,
        target: &HireTarget,
        creature: CreatureTypeId,
    ) -> u32 {
        let HireTarget::Hero(hero) = *target;
        let rules = (part.params().army_slots(), part.params().max_stack_count());
        self.hero(hero)
            .map_or(0, |hero| room_bound(&hero.army, rules, creature))
    }

    /// Checks a purchase of `amount` `creature`s in `dialog` (ADV-HIR-002, ADV-ARM-007; E08 T95b,
    /// increment 10's design note §8; survey `E08-dwellings.md` §5.3, the original's
    /// `recruit_dialog_proc` 0x550940) and gives it with its price ([`HirePurchase::price`]:
    /// [`hire_price`], the creature's core cost times the amount by resource, in the core's
    /// order), which the game checks against the treasury before and pays after (ADR-0004), then
    /// makes with [`Self::hire`]. Refused, in this order: the dialog's hero not on the map or its
    /// source not the object it names ([`HireError::NoSource`]; defensive: the game holds the
    /// dialog its outcome gave, and its hero cannot move while it is open); the creature not the
    /// source's ([`HireError::NotOffered`]); the amount 0 or above its count
    /// ([`HireError::Amount`]); no room in the army ([`HireError::NoRoom`]: the original's
    /// GENRLTXT 426, nothing bought; every amount up to [`Self::hire_room`] finds room, in the
    /// [`room`] the purchase keeps). *Agent's call:* the hero's tile is not tested against the
    /// source's entrance, as the visit put it there and nothing moves it while the dialog is open.
    // rule: ADV-HIR-002, ADV-ARM-007
    pub fn check_hire(
        &self,
        (core, part): (&Ruleset, &AdventureRuleset),
        dialog: &HireDialog,
        (creature, amount): (CreatureTypeId, u32),
    ) -> Result<HirePurchase, HireError> {
        let HireTarget::Hero(hero) = dialog.target;
        let HireSource::Object { index, .. } = dialog.source;
        let (at, stock) = self
            .heroes
            .binary_search_by_key(&hero, |h| h.id)
            .ok()
            .zip(self.hire_stock(part, &dialog.source))
            .ok_or(HireError::NoSource)?;
        let place = stock
            .iter()
            .position(|&(sold, _)| sold == creature)
            .ok_or(HireError::NotOffered(creature))?;
        if !(1..=stock[place].1).contains(&amount) {
            return Err(HireError::Amount(amount));
        }
        let rules = (part.params().army_slots(), part.params().max_stack_count());
        let room = room(&self.heroes[at].army, rules, creature, amount).ok_or(HireError::NoRoom)?;
        Ok(HirePurchase {
            price: hire_price(core, creature, amount),
            creature,
            amount,
            index,
            at,
            place,
            room,
        })
    }

    /// Makes `purchase` (ADV-HIR-002, ADV-ARM-007; E08 T95b, increment 10's design note §8, §13;
    /// survey `E08-dwellings.md` §5.3, the original's `recruit_dialog_proc` 0x550940), which
    /// [`Self::check_hire`] gave for this state as it stands, after the game paid its price
    /// (GAM-DEC-008; the treasury is never this crate's): the creatures join the hero's army where
    /// ADV-ARM-007's room is (the original's `army_add_stack`: a stack of the creature that takes
    /// them within a stack's most, else the first free slot), the source's count goes down by the
    /// amount (a dwelling's through `write_dwelling`, a camp's block copied at the write), and the
    /// hero's owner alone is told ([`AdventureEventKind::CreaturesJoined`]).
    // rule: ADV-HIR-002, ADV-ARM-007
    pub fn hire<S: Sink>(&mut self, purchase: HirePurchase, sink: &mut S) {
        let HirePurchase {
            creature,
            amount,
            index,
            at,
            place,
            room,
            ..
        } = purchase;
        let on_map = &mut self.heroes[at];
        join_at(&mut on_map.army, room, creature, amount);
        let (hero, owner) = (on_map.id, on_map.owner);
        self.lower_count(index, place, amount);
        sink.emit(AdventureEvent {
            kind: AdventureEventKind::CreaturesJoined {
                hero,
                creature,
                count: amount,
            },
            seen_by: Viewers::NONE.with(owner),
        });
    }

    /// Lowers by `amount` the count of the creature at `place` of the source at object index
    /// `index` (ADV-HIR-002): a camp's (its block copied at the write, as the week's draw writes
    /// it), else a dwelling's ([`Self::write_dwelling`], keeping T94b's representation).
    fn lower_count(&mut self, index: usize, place: usize, amount: u32) {
        match self.map.camp_index(index) {
            Some(camp) => {
                let mut camps = self.camps.as_deref().unwrap_or_default().to_vec();
                camps[camp].count = camps[camp].count.saturating_sub(amount);
                self.camps = Some(camps.into());
            }
            None => self.write_dwelling(index, |week| {
                let count = &mut week.counts_mut()[place];
                *count = count.saturating_sub(amount);
            }),
        }
    }
}

#[cfg(test)]
impl AdventureState {
    /// Writes the counts and the guards of the dwelling at object index `index` (tests of the
    /// dwellings' state, hash and writer, E08 T94b, T95a1) through the writer
    /// ([`Self::write_dwelling`]), which keeps the field's representation.
    pub(crate) fn set_dwelling(&mut self, index: usize, counts: &[u32], guards: &[ArmyStack]) {
        self.write_dwelling(index, |week| {
            *week = DwellingWeek::new(counts.iter().copied(), guards.iter().copied());
        });
    }

    /// Writes the dwellings' block as it is, normalised or not (tests of the invariants and the
    /// hash, E08 T94b): every dwelling's week as it began.
    pub(crate) fn set_full_dwellings_block(&mut self) {
        self.dwellings = Some(self.map.full_weeks.clone().into());
    }

    /// Writes the creature and the count of the camp at object index `index` (tests of the
    /// camps' invariants and hash, E08 T94b; a count of 0 for a deserted camp).
    pub(crate) fn set_camp(&mut self, index: usize, creature: CreatureTypeId, count: u32) {
        let place = self.map.camp_index(index).expect("a camp");
        let mut camps = self.camps.as_deref().unwrap_or_default().to_vec();
        camps[place] = Camp { creature, count };
        self.camps = Some(camps.into());
    }
}
