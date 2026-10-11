//! The adventure's action encoding (ADR-0017 point 8, E08 T9a), factorized as the battle's casts
//! are (ADR-0014 point 9): an *entry*, and for a move a destination tile in the *tile head*.
//!
//! Entries, version 1 (independent of the map and the ruleset, so agents see a fixed number):
//!
//! | entry | action |
//! |---|---|
//! | 0 | End the turn |
//! | 1 + s, s < 8 | Move the player's hero in slot `s` to the tile in the tile head |
//!
//! A player's hero slots are its heroes on the map in the order of their ids, the first
//! [`HERO_SLOTS`] (the original allows a player eight heroes on the map); a hero's slot changes
//! when a hero of a lower id joins or leaves the player (later increments). The tile head is a
//! tile of the map, `(level × height + row) × width + column`, the observation's order; its size
//! is the map's tile count ([`crate::adventure::AdventureEnv::tile_count`]). After `EndTurn` the
//! tile head allows exactly 0. A move's destination is a tile the hero's reachable-tiles query
//! lists, not its own; the action carries no path: the command takes the path the query chose
//! (`h3_game::legal_moves`). A client may send any other path the hero can pay (`apply` checks
//! each step); an agent of this encoding moves along the query's paths only. The heads' masks
//! are conditioned on the entry ([`legal_mask`], [`tile_mask`]).

use h3_adventure::{AdventureCommand, Pos, Reach};
use h3_contracts::{HeroId, PlayerId};
use h3_game::{Actor, GameCommand, GameRuleset, GameState, legal_commands, legal_moves};

/// Version of this encoding; bump on any change of the layout above.
pub const ACTION_ENCODING_VERSION: u32 = 1;

/// Hero slots per player: the most heroes the original allows a player on the map.
pub const HERO_SLOTS: usize = 8;

/// The entry that ends the turn.
pub const END_TURN: usize = 0;

/// The entry of the hero in slot 0's move; slot `s` is `MOVE_BASE + s`.
pub const MOVE_BASE: usize = 1;

/// Number of entries.
pub const ENTRY_COUNT: usize = MOVE_BASE + HERO_SLOTS;

/// Which entries are legal.
pub type EntryMask = [bool; ENTRY_COUNT];

/// An action: an entry and its tile head (0 for `EndTurn`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Action {
    /// The entry.
    pub entry: usize,
    /// The destination tile of a move, `(level × height + row) × width + column`; 0 otherwise.
    pub tile: u32,
}

impl Action {
    /// Ends the turn.
    pub const fn end_turn() -> Self {
        Self {
            entry: END_TURN,
            tile: 0,
        }
    }

    /// Moves the hero in `slot` to `tile`.
    pub const fn move_to(slot: usize, tile: u32) -> Self {
        Self {
            entry: MOVE_BASE + slot,
            tile,
        }
    }
}

/// The map's number of tiles: the size of the tile head.
pub fn tile_count(state: &GameState) -> usize {
    let map = state.adventure().map();
    usize::from(map.width()) * usize::from(map.height()) * usize::from(map.levels())
}

/// The tile head's index of `pos`, if it is on the map.
pub fn tile_index(state: &GameState, pos: Pos) -> Option<u32> {
    let map = state.adventure().map();
    let (width, height) = (u32::from(map.width()), u32::from(map.height()));
    (pos.x < map.width() && pos.y < map.height() && pos.level < map.levels())
        .then(|| (u32::from(pos.level) * height + u32::from(pos.y)) * width + u32::from(pos.x))
}

/// The tile at the tile head's `index`, if it is on the map.
pub fn tile_pos(state: &GameState, index: u32) -> Option<Pos> {
    let map = state.adventure().map();
    let (width, height) = (u32::from(map.width()), u32::from(map.height()));
    let level = u8::try_from(index / (width * height)).ok()?;
    let rest = index % (width * height);
    let x = u16::try_from(rest % width).ok()?;
    let y = u16::try_from(rest / width).ok()?;
    (level < map.levels()).then(|| Pos::new(x, y, level))
}

/// The hero in `slot` of `player`: its heroes in the order of their ids.
pub fn hero_in_slot(state: &GameState, player: PlayerId, slot: usize) -> Option<HeroId> {
    (slot < HERO_SLOTS)
        .then(|| {
            state
                .adventure()
                .heroes()
                .iter()
                .filter(|hero| hero.owner == player)
                .nth(slot)
                .map(|hero| hero.id)
        })
        .flatten()
}

/// The legal moves of the hero in `slot` of `player` ([`legal_moves`]), if any tile but its own
/// is reachable.
pub fn moves(
    state: &GameState,
    rules: &GameRuleset,
    player: PlayerId,
    slot: usize,
) -> Option<Reach> {
    let hero = hero_in_slot(state, player, slot)?;
    legal_moves(state, rules, Actor::Player(player), hero).filter(|r| r.tiles().next().is_some())
}

/// Fills `mask` with the legal entries of `player`: `EndTurn` while its turn runs, and each hero
/// slot with a tile to move to.
pub fn legal_mask(state: &GameState, rules: &GameRuleset, player: PlayerId, mask: &mut EntryMask) {
    let actor = Actor::Player(player);
    mask.fill(false);
    mask[END_TURN] = legal_commands(state, rules, actor).contains(&GameCommand::EndTurn);
    for slot in 0..HERO_SLOTS {
        mask[MOVE_BASE + slot] = moves(state, rules, player, slot).is_some();
    }
}

/// Fills `mask` (resized to the tile count) with the legal tiles of `player` after choosing
/// `entry`: 0 alone after a legal `EndTurn`, the reachable tiles but its own of a hero's move,
/// none after an illegal entry.
pub fn tile_mask(
    state: &GameState,
    rules: &GameRuleset,
    player: PlayerId,
    entry: usize,
    mask: &mut Vec<bool>,
) {
    mask.clear();
    mask.resize(tile_count(state), false);
    if entry == END_TURN {
        mask[0] =
            legal_commands(state, rules, Actor::Player(player)).contains(&GameCommand::EndTurn);
    } else if let Some(reach) = entry
        .checked_sub(MOVE_BASE)
        .and_then(|slot| moves(state, rules, player, slot))
    {
        for (pos, _) in reach.tiles() {
            if let Some(index) = tile_index(state, pos) {
                mask[index as usize] = true;
            }
        }
    }
}

/// The command `player` issues with `action`, if the action names one: `EndTurn` (with tile 0),
/// or a move of one of its heroes to a tile its reachable-tiles query lists (not its own) along
/// the query's path. Whether the command may be issued now (whose turn it is) is `apply`'s.
pub fn decode(
    state: &GameState,
    rules: &GameRuleset,
    player: PlayerId,
    action: Action,
) -> Option<GameCommand> {
    if action.entry == END_TURN {
        return (action.tile == 0).then_some(GameCommand::EndTurn);
    }
    let hero = hero_in_slot(state, player, action.entry.checked_sub(MOVE_BASE)?)?;
    let to = tile_pos(state, action.tile)?;
    let reach = state
        .adventure()
        .reach(rules.core(), rules.adventure(), hero)?;
    let path = reach.path(to).filter(|path| !path.is_empty())?;
    Some(GameCommand::Adventure(AdventureCommand::MoveHero {
        hero,
        path,
    }))
}
