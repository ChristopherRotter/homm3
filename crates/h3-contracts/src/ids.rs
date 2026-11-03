//! Cross-context identities (ADR-0015 point 5): the players, heroes and towns of a game, named the
//! same way by every context and by `h3-game`, which keeps the lists they index. Context-internal
//! handles stay in their crates.

/// A player of a game: its position in the game's player list, which follows the original's
/// colour order (red first). How many players a game has comes from its map (ADR-0015 point 6);
/// the setup's validation checks every id against that list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PlayerId(u8);

impl PlayerId {
    /// The player at `index` of the game's player list.
    pub const fn new(index: u8) -> Self {
        Self(index)
    }

    /// The raw index.
    pub const fn raw(self) -> u8 {
        self.0
    }

    /// The index as a `usize`, for table access.
    pub const fn index(self) -> usize {
        self.0 as usize
    }
}

/// A hero of a game: its position in the game's store of heroes (ADR-0013 point 1), which
/// `h3-game` keeps. The hero on the adventure map, the hero details and a battle's hero all name a
/// hero by it; it is not a hero type (`h3_content::HeroType`), and no type is named `Hero`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HeroId(u16);

impl HeroId {
    /// The hero at `index` of the game's store of heroes.
    pub const fn new(index: u16) -> Self {
        Self(index)
    }

    /// The raw index.
    pub const fn raw(self) -> u16 {
        self.0
    }

    /// The index as a `usize`, for table access.
    pub const fn index(self) -> usize {
        self.0 as usize
    }
}

/// A town of a game (ADR-0028 point 3): its index among the map's towns (the object classes 77 and
/// 98) in object order, dense from 0 and fixed when the game is built, since no rule creates or
/// removes a town. The adventure's town object holds it, the store of towns (`h3-towns`, kept by
/// `h3-game`) has a town's record by it, and commands, events and observations name a town by it.
/// Files and the wire write it as its number ([`TownId::raw`]), as they write a [`HeroId`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TownId(u16);

impl TownId {
    /// The town at `index` among the map's towns.
    pub const fn new(index: u16) -> Self {
        Self(index)
    }

    /// The raw index.
    pub const fn raw(self) -> u16 {
        self.0
    }

    /// The index as a `usize`, for table access.
    pub const fn index(self) -> usize {
        self.0 as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn player_ids_keep_their_index_and_colour_order() {
        let red = PlayerId::new(0);
        let blue = PlayerId::new(1);
        assert_eq!((red.raw(), blue.index()), (0, 1));
        assert!(red < blue);
    }

    #[test]
    fn hero_ids_keep_their_index() {
        let hero = HeroId::new(155);
        assert_eq!((hero.raw(), hero.index()), (155, 155));
        assert!(HeroId::new(3) < hero);
    }

    #[test]
    fn town_ids_keep_their_index_and_object_order() {
        let town = TownId::new(47);
        assert_eq!((town.raw(), town.index()), (47, 47));
        assert!(TownId::new(0) < town);
    }
}
