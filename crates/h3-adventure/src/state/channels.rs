//! The teleporters' channels (E08 T104; ADV-TEL-002, increment 11's design note §5): built when
//! the game is built, after the static map's tiles and objects (the random dwellings resolved,
//! every type final), from the typed objects alone and **without a draw**, so no setup draw moves
//! (ADR-0017 point 4). Static, behind the static map's `Arc` and so shared by forks (decision 6);
//! empty for a map without teleporters. Read by a teleporter's visit alone (T105a), never by the
//! reach (D-58). They follow from the objects, which the static map's hash covers, so the hash
//! leaves them out.
//!
//! The original's setup (`setup_map_objects` 0x4C0CC0, survey `E08-teleporters.md` §3.1) walks
//! the tiles level by level, row by row, column by column and fills a word per tile by the tile's
//! class, the last object placed there; so each list here holds **tiles** in the walk's order (the
//! static map's tile index, not `Pos`' derived order), a tile listed for the last object in map
//! order visited from it when that object is a teleporter. The checks before (A-42, `ADV-QUIRK-44`)
//! refuse a teleporter off its surface.

use alloc::collections::BTreeMap;
use alloc::vec;
use alloc::vec::Vec;

use crate::ruleset::{AdventureRuleset, MonolithWay, ObjectKind, Passage};
use crate::setup::{Pos, SetupError};

use super::StaticMap;

/// A Subterranean Gate's entry (one of the tiles it is visited from) and the entry it leads to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GateEntry {
    /// The entry.
    pub tile: Pos,
    /// The entry it is paired with ([`Channels::gates`]); none: it leads nowhere.
    pub pair: Option<Pos>,
}

/// A tile of the Whirlpools' list ([`Channels::whirlpools`]): a Whirlpool's visitable tile with
/// its Whirlpool, the original's word the six tiles share (survey §3.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WhirlpoolTile {
    /// The tile.
    pub tile: Pos,
    /// Its Whirlpool's index among the static map's objects.
    pub object: usize,
}

/// The teleporters' channels and the gates' pairs of a static map (ADV-TEL-002;
/// [`StaticMap::channels`]), each list in the walk's order: level, row, column.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Channels {
    /// Each one-way channel's exits, by its number (a monolith of way [`MonolithWay::Exit`]):
    /// what an entrance of that number leads to. An entrance is in no list (the original's case
    /// 43 has none).
    exits: BTreeMap<u8, Vec<Pos>>,
    /// Each two-way channel's members, by its number (way [`MonolithWay::TwoWay`]): a member
    /// leads to the others.
    two_way: BTreeMap<u8, Vec<Pos>>,
    /// Every Whirlpool's tiles, each with its Whirlpool.
    whirlpools: Vec<WhirlpoolTile>,
    /// Every gate's entries, each with its pair.
    gates: Vec<GateEntry>,
}

impl Channels {
    /// Whether the map holds no teleporter's entry.
    pub fn is_empty(&self) -> bool {
        self.exits.is_empty()
            && self.two_way.is_empty()
            && self.whirlpools.is_empty()
            && self.gates.is_empty()
    }

    /// The exits of the one-way channel `channel` (none for a channel without one).
    pub fn exits(&self, channel: u8) -> &[Pos] {
        self.exits.get(&channel).map_or(&[], Vec::as_slice)
    }

    /// The members of the two-way channel `channel` (none for a channel without one).
    pub fn two_way(&self, channel: u8) -> &[Pos] {
        self.two_way.get(&channel).map_or(&[], Vec::as_slice)
    }

    /// The one-way channels with an exit, by number, each with its exits.
    pub fn one_way_channels(&self) -> impl Iterator<Item = (u8, &[Pos])> {
        self.exits
            .iter()
            .map(|(&channel, exits)| (channel, exits.as_slice()))
    }

    /// The two-way channels, by number, each with its members.
    pub fn two_way_channels(&self) -> impl Iterator<Item = (u8, &[Pos])> {
        self.two_way
            .iter()
            .map(|(&channel, members)| (channel, members.as_slice()))
    }

    /// Every Whirlpool's tiles in the walk's order, each with its Whirlpool.
    pub fn whirlpools(&self) -> &[WhirlpoolTile] {
        &self.whirlpools
    }

    /// Every gate's entries in the walk's order, each with its pair.
    pub fn gates(&self) -> &[GateEntry] {
        &self.gates
    }

    /// The entry the gate's entry `tile` is paired with; none for an entry without a pair or a
    /// tile that is no gate's entry.
    pub fn gate_pair(&self, tile: Pos) -> Option<Pos> {
        self.gates
            .binary_search_by_key(&walk_key(tile), |gate| walk_key(gate.tile))
            .ok()
            .and_then(|place| self.gates[place].pair)
    }
}

/// `pos`' place in the walk's order: level, row, column.
const fn walk_key(pos: Pos) -> (u8, u16, u16) {
    (pos.level, pos.y, pos.x)
}

/// The surface a teleporter's visitable tiles lie on (A-42): water for a Whirlpool, land for a
/// monolith and a gate; none for another kind.
const fn surface(kind: &ObjectKind) -> Option<Surface> {
    match kind {
        ObjectKind::Monolith { .. } | ObjectKind::SubterraneanGate => Some(Surface::Land),
        ObjectKind::Whirlpool { .. } => Some(Surface::Water),
        _ => None,
    }
}

/// What a teleporter's visitable tiles must be.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Surface {
    /// Walked on foot ([`Passage::Land`]).
    Land,
    /// Sailed ([`Passage::Water`]).
    Water,
}

/// Checks the teleporters' surfaces and builds the channels of `map` (its objects placed and
/// resolved): ADV-TEL-002, ADV-TEL-001's A-42.
///
/// # Errors
///
/// [`SetupError::TeleporterSurface`] for the first teleporter in object order with a visitable
/// tile of the map off its surface (A-42, `ADV-QUIRK-44`): a monolith's or a Subterranean Gate's
/// on water or rock, a Whirlpool's on land or rock. A jump between surfaces would put a hero in a
/// boat on land or a walker on water (or either on rock), which no rule handles; the original's
/// case is unread (survey §1: none of the shipped maps holds one, every monolith and gate on land,
/// every Whirlpool tile on water).
// rule: ADV-TEL-001, ADV-TEL-002
pub(super) fn build(map: &StaticMap, part: &AdventureRuleset) -> Result<Channels, SetupError> {
    let kind = |index: usize| {
        map.objects[index]
            .object_type
            .and_then(|id| part.object_type(id))
            .map(|ty| &ty.kind)
    };
    let mut any = false;
    for (index, object) in map.objects.iter().enumerate() {
        let Some(surface) = kind(index).and_then(surface) else {
            continue;
        };
        any = true;
        let off = object
            .visitable
            .offsets()
            .filter_map(|(column, row)| map.offset(object.anchor, column, row))
            .any(|tile| {
                let passage = part.passage(map.tiles[tile].terrain);
                match surface {
                    Surface::Land => !matches!(passage, Some(Passage::Land(_))),
                    Surface::Water => !matches!(passage, Some(Passage::Water(_))),
                }
            });
        if off {
            return Err(SetupError::TeleporterSurface(index));
        }
    }
    let mut channels = Channels::default();
    if !any {
        return Ok(channels);
    }
    let mut gates = Vec::new();
    // The tiles objects are visited from, by tile index: the walk's order; each with the last
    // object in map order visited from it (every list of `visits` holds one at least).
    let tops = map
        .visits
        .iter()
        .filter_map(|(&tile, objects)| objects.last().map(|&object| (tile, object)));
    for (tile, object) in tops {
        let pos = map.pos_of(tile);
        match kind(object) {
            Some(&ObjectKind::Monolith {
                channel,
                way: MonolithWay::Exit,
            }) => channels.exits.entry(channel).or_default().push(pos),
            Some(&ObjectKind::Monolith {
                channel,
                way: MonolithWay::TwoWay,
            }) => channels.two_way.entry(channel).or_default().push(pos),
            Some(ObjectKind::SubterraneanGate) => gates.push(pos),
            Some(ObjectKind::Whirlpool { .. }) => {
                channels
                    .whirlpools
                    .push(WhirlpoolTile { tile: pos, object });
            }
            _ => {}
        }
    }
    channels.gates = pair_gates(&gates);
    Ok(channels)
}

/// The gates' pairs (ADV-TEL-002; D-56 as the original, `gates_pair` 0x4C0B60, survey §3.2):
/// over `entries` in the walk's order, each entry still unpaired takes the unpaired entry after it
/// on another level with the least truncated distance `⌊√(dx² + dy²)⌋` (`dx`, `dy` the entries'
/// columns and rows apart), the first in the walk's order on a tie, and both are paired; one left
/// over has none. The original takes the root on a double and truncates it, which is the integer
/// square root for every map size (exact), so no float (ADR-0003). In the walk's order the levels
/// come one after another, so the entries after one on another level are those of the levels
/// after its own. Quadratic in the gates: about 17 M comparisons for the 8 000 of a crafted XL map
/// with a gate on every fifth tile (`build-gates-xl-144`), a few hundred on the shipped maps.
// rule: ADV-TEL-002
fn pair_gates(entries: &[Pos]) -> Vec<GateEntry> {
    let mut pairs: Vec<Option<usize>> = vec![None; entries.len()];
    for (i, entry) in entries.iter().enumerate() {
        if pairs[i].is_some() {
            continue;
        }
        let after = entries.partition_point(|other| other.level <= entry.level);
        // The least truncated distance so far and its entry.
        let mut nearest: Option<(u64, usize)> = None;
        for (j, other) in entries.iter().enumerate().skip(after) {
            if pairs[j].is_some() {
                continue;
            }
            let dx = u64::from(entry.x.abs_diff(other.x));
            let dy = u64::from(entry.y.abs_diff(other.y));
            let squared = dx * dx + dy * dy;
            // ⌊√squared⌋ < d exactly when squared < d², so the loop takes no root.
            if nearest.is_none_or(|(distance, _)| squared < distance * distance) {
                nearest = Some((squared.isqrt(), j));
            }
        }
        if let Some((_, j)) = nearest {
            pairs[i] = Some(j);
            pairs[j] = Some(i);
        }
    }
    entries
        .iter()
        .zip(pairs)
        .map(|(&tile, pair)| GateEntry {
            tile,
            pair: pair.map(|j| entries[j]),
        })
        .collect()
}
