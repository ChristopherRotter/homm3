//! The per-player state of object types (ADR-0016 point 1): the Cartographer types each player
//! bought (ADV-VIS-001, E08 T14b) and the keys each player holds (ADV-VIS-003, T15a), laid out as
//! the original's flags, a byte of player bits per entry (bit `n` for the player with id `n`).
//! Only a player's own commands write its bits (ADR-0017 point 7). The state holds them as one
//! small block the forks share, none until the first purchase or key: a fork costs nothing more
//! before and a reference count after (`bench-check`), and a write copies the block.

use alloc::vec::Vec;

use h3_contracts::PlayerId;
use h3_kernel::{CanonicalHash, StateHasher};

use crate::command::bit;
use crate::ruleset::{KEY_COLOURS, ObjectTypeId};

/// The Cartographer types bought and the keys held.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Flags {
    /// The Cartographer types bought, in type order, each with its player bits (game +0x4E372, a
    /// byte per kind).
    bought: Vec<(ObjectTypeId, u8)>,
    /// The keys, by colour: the player bits of the players holding it (game +0x4E364, a byte per
    /// colour; H3API's `keymasterVisited`).
    keys: [u8; KEY_COLOURS as usize],
}

impl Flags {
    /// No type bought, no key held.
    pub(crate) const NONE: Self = Self {
        bought: Vec::new(),
        keys: [0; KEY_COLOURS as usize],
    };

    /// Whether `player` has bought `object_type`.
    pub(crate) fn has_bought(&self, player: PlayerId, object_type: ObjectTypeId) -> bool {
        self.bought
            .binary_search_by_key(&object_type, |&(ty, _)| ty)
            .is_ok_and(|at| self.bought[at].1 & bit(player) != 0)
    }

    /// The types `player` has bought, in type order.
    pub(crate) fn bought(&self, player: PlayerId) -> impl Iterator<Item = ObjectTypeId> + '_ {
        let player = bit(player);
        self.bought
            .iter()
            .filter(move |&&(_, players)| players & player != 0)
            .map(|&(ty, _)| ty)
    }

    /// `player` has bought `object_type`.
    pub(crate) fn buy(&mut self, player: PlayerId, object_type: ObjectTypeId) {
        match self
            .bought
            .binary_search_by_key(&object_type, |&(ty, _)| ty)
        {
            Ok(at) => self.bought[at].1 |= bit(player),
            Err(at) => self.bought.insert(at, (object_type, bit(player))),
        }
    }

    /// The keys `player` holds: bit `c` for the colour `c`.
    pub(crate) fn keys(&self, player: PlayerId) -> u8 {
        let player = bit(player);
        self.keys
            .iter()
            .zip(0u8..)
            .filter(|&(&players, _)| players & player != 0)
            .fold(0, |keys, (_, colour)| keys | 1 << colour)
    }

    /// Whether `player` holds the key of `colour` (none of a colour past [`KEY_COLOURS`]).
    pub(crate) fn has_key(&self, player: PlayerId, colour: u8) -> bool {
        self.keys
            .get(usize::from(colour))
            .is_some_and(|&players| players & bit(player) != 0)
    }

    /// `player` holds the key of `colour` (a colour of [`KEY_COLOURS`]; another holds nothing).
    pub(crate) fn give_key(&mut self, player: PlayerId, colour: u8) {
        self.keys
            .get_mut(usize::from(colour))
            .into_iter()
            .for_each(|players| *players |= bit(player));
    }
}

impl CanonicalHash for Flags {
    /// The types bought as length and entries in type order (the type, then its player bits);
    /// then the keys, a byte of player bits per colour.
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_len(self.bought.len());
        for (object_type, players) in &self.bought {
            object_type.hash_into(h);
            h.write_u8(*players);
        }
        for players in self.keys {
            h.write_u8(players);
        }
    }
}
