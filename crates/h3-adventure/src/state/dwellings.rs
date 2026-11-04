//! The random dwellings when the game is built (ADV-DWL-005, E08 T94a; increment 10's design note
//! §5): each one's draw among its outcomes in the static map's placement walk, the names its
//! outcomes take, and the tiles a hero may not stand on beside one, whatever it draws.

use alloc::borrow::Cow;
use alloc::collections::BTreeMap;

use h3_content::FactionId;
use h3_kernel::Rng;

use super::{StaticMap, pick};
use crate::ruleset::{AdventureRuleset, ObjectTypeId};
use crate::setup::{Holds, ObjectSetup, Pos, TileMask};

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
