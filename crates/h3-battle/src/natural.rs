//! The battlefield's natural obstacles as pieces (E04b T5b; OPEN-QUESTIONS Q55): the hexes they
//! block (BAT-GEO-003) and the removable ones with their anchor hexes, which Remove Obstacle
//! takes away whole (BAT-MAG-034).
//!
//! The original keeps a usual obstacle as an object whose anchor hex carries the object's index
//! even where the obstacle does not block it (`docs/formats/installation.md` §7.15): with quirk
//! BAT-QUIRK-3 such an anchor hex may be stood on and walked through, but takes no spell obstacle
//! and leads Remove Obstacle to its obstacle. An absolute obstacle has no anchor and stays.
//!
//! The pieces live in the state by value (no allocation on a fork): an anchor and up to eight
//! hexes each, at most sixteen; the blocked hexes and the anchors are kept as sets beside them,
//! so movement and the target tests read a set.

use h3_contracts::Obstacle;
use h3_kernel::{CanonicalHash, FixedVec, StateHasher};

use crate::error::SetupError;
use crate::geometry::{Field, Hex, HexSet};

/// Most removable natural obstacles in one battle (the generator places at most twelve).
pub const MAX_REMOVABLE_OBSTACLES: usize = 16;

/// Most hexes a removable natural obstacle blocks (the original's usual obstacles block up to
/// eight).
pub const MAX_REMOVABLE_HEXES: usize = 8;

/// A removable natural obstacle: its anchor hex and the hexes it blocks (Q55).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RemovableObstacle {
    anchor: Hex,
    len: u8,
    hexes: [Hex; MAX_REMOVABLE_HEXES],
}

impl RemovableObstacle {
    /// Its anchor hex (the obstacle's bottom-left hex, BAT-GEO-004).
    pub const fn anchor(&self) -> Hex {
        self.anchor
    }

    /// The hexes it blocks, as the setup listed them.
    pub fn hexes(&self) -> &[Hex] {
        self.hexes.get(..usize::from(self.len)).unwrap_or_default()
    }

    /// The hexes it blocks, as a set.
    pub fn hex_set(&self) -> HexSet {
        self.hexes().iter().fold(HexSet::new(), |mut set, &h| {
            set.insert(h);
            set
        })
    }
}

impl CanonicalHash for RemovableObstacle {
    /// Anchor, then the blocked hexes (count, hexes).
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_u8(self.anchor.0);
        h.write_len(self.hexes().len());
        for hex in self.hexes() {
            h.write_u8(hex.0);
        }
    }
}

/// The natural obstacles of a battle (BAT-GEO-003), modelled on the original's hex records
/// (`installation.md` §7.15): a removable obstacle gives each hex it blocks, and with quirk
/// BAT-QUIRK-3 its anchor hex, its index, the obstacle placed last winning; removing one
/// (0x466B30) clears the bits and the index of its blocked hexes and the anchor bit and the index
/// of its anchor hex, so such a hex leads to no obstacle any more.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NaturalObstacles {
    /// Every hex a natural obstacle blocks.
    blocked: HexSet,
    /// The removable ones, in setup order (the order placed).
    removable: FixedVec<RemovableObstacle, MAX_REMOVABLE_OBSTACLES>,
    /// The hexes a removal cleared: no obstacle's index or anchor bit is left on them.
    cleared: HexSet,
    /// The anchor hexes that belong to their obstacles: with quirk BAT-QUIRK-3 every removable
    /// obstacle's not cleared, else none.
    anchors: HexSet,
    /// Quirk BAT-QUIRK-3: an anchor hex belongs to its obstacle.
    anchored: bool,
}

impl NaturalObstacles {
    /// Validates the setup's obstacles (BAT-GEO-003): every blocked hex standable and listed
    /// once across all of them; an anchor standable; at most [`MAX_REMOVABLE_OBSTACLES`]
    /// removable obstacles of at most [`MAX_REMOVABLE_HEXES`] hexes each. `anchored`: quirk
    /// BAT-QUIRK-3.
    // rule: BAT-GEO-003
    pub(crate) fn new(
        field: Field,
        obstacles: &[Obstacle],
        anchored: bool,
    ) -> Result<Self, SetupError> {
        let mut blocked = HexSet::new();
        let mut removable = FixedVec::new();
        for o in obstacles {
            for &raw in &o.hexes {
                let hex = Hex(raw);
                if !field.is_standable(hex) || blocked.contains(hex) {
                    return Err(SetupError::InvalidObstacle(raw));
                }
                blocked.insert(hex);
            }
            let Some(anchor) = o.anchor else {
                continue;
            };
            if !field.is_standable(Hex(anchor)) {
                return Err(SetupError::InvalidAnchor(anchor));
            }
            let mut piece = RemovableObstacle {
                anchor: Hex(anchor),
                len: 0,
                hexes: [Hex(0); MAX_REMOVABLE_HEXES],
            };
            for (slot, &raw) in piece.hexes.iter_mut().zip(&o.hexes) {
                *slot = Hex(raw);
                piece.len += 1;
            }
            if o.hexes.len() > MAX_REMOVABLE_HEXES || removable.push(piece).is_err() {
                return Err(SetupError::TooManyObstacles);
            }
        }
        let mut natural = Self {
            blocked,
            removable,
            cleared: HexSet::new(),
            anchors: HexSet::new(),
            anchored,
        };
        natural.refresh();
        Ok(natural)
    }

    /// Every hex a natural obstacle blocks.
    pub(crate) const fn blocked(&self) -> HexSet {
        self.blocked
    }

    /// The anchor hexes that belong to their obstacles (quirk BAT-QUIRK-3; none without it).
    pub(crate) const fn anchors(&self) -> HexSet {
        self.anchors
    }

    /// The removable obstacles, in setup order.
    pub(crate) fn removable(&self) -> &[RemovableObstacle] {
        self.removable.as_slice()
    }

    /// The removable obstacles, in setup order, by value (observations).
    pub(crate) const fn removable_list(
        &self,
    ) -> FixedVec<RemovableObstacle, MAX_REMOVABLE_OBSTACLES> {
        self.removable
    }

    /// Whether some hex leads to a removable obstacle ([`Self::removable_at`]): one blocks a hex
    /// (a removal never clears the blocked hexes of another), or has an anchor that belongs to it.
    pub(crate) fn any_removable(&self) -> bool {
        self.removable
            .iter()
            .any(|o| o.len > 0 || self.anchors.contains(o.anchor))
    }

    /// The index of the removable obstacle on `hex` (BAT-MAG-034): of those that block it or, with
    /// quirk BAT-QUIRK-3, have it as their anchor, the one placed last, as the original's hex
    /// holds the index of the object placed last; none on a hex a removal cleared.
    // rule: BAT-MAG-034
    pub(crate) fn removable_at(&self, hex: Hex) -> Option<usize> {
        if self.cleared.contains(hex) {
            return None;
        }
        self.removable
            .iter()
            .rposition(|o| o.hexes().contains(&hex) || (self.anchored && o.anchor == hex))
    }

    /// Removes the removable obstacle `i` (a validated index) and returns it: its blocked hexes
    /// are free and, with quirk BAT-QUIRK-3, its anchor hex is cleared too (the original's
    /// 0x466B30): another obstacle's anchor there no longer belongs to it, and a hex another
    /// obstacle blocks there stays blocked but leads to none.
    pub(crate) fn remove(&mut self, i: usize) -> RemovableObstacle {
        let gone = self.removable.get(i).copied().unwrap_or_default();
        let mut at = 0;
        self.removable.remove_if(|_| {
            at += 1;
            at - 1 == i
        });
        self.blocked = self.blocked.without(gone.hex_set());
        self.cleared = self.cleared.union(gone.hex_set());
        if self.anchored {
            self.cleared.insert(gone.anchor);
        }
        self.refresh();
        gone
    }

    /// Blocks `hex` with an obstacle that cannot be removed (tests).
    #[cfg(test)]
    pub(crate) fn block(&mut self, hex: Hex) {
        self.blocked.insert(hex);
    }

    /// Recomputes the anchors from the removable obstacles and the cleared hexes.
    fn refresh(&mut self) {
        self.anchors = if self.anchored {
            self.removable
                .iter()
                .fold(HexSet::new(), |mut set, o| {
                    set.insert(o.anchor);
                    set
                })
                .without(self.cleared)
        } else {
            HexSet::new()
        };
    }
}

impl CanonicalHash for NaturalObstacles {
    /// The blocked hexes, the removable obstacles (count, each), the cleared hexes, whether
    /// anchors belong to their obstacles (state hash v28).
    fn hash_into(&self, h: &mut StateHasher) {
        self.blocked.hash_into(h);
        self.removable.hash_into(h);
        self.cleared.hash_into(h);
        h.write_bool(self.anchored);
    }
}
