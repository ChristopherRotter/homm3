//! The adventure's action encoding (ADR-0017 point 8, E08 T9a), factorized as the battle's casts
//! are (ADR-0014 point 9): an *entry*, and for a move a destination tile in the *tile head*.
//!
//! Entries, version 2 (independent of the map and the ruleset, so agents see a fixed number):
//!
//! | entry | action |
//! |---|---|
//! | 0 | End the turn |
//! | 1 + s, s < 8 | Move the player's hero in slot `s` to the tile in the tile head |
//! | 9 | Accept the player's open decision (E08 T14b: a Cartographer's offer) |
//! | 10 | Decline it |
//!
//! A player's hero slots are its heroes on the map in the order of their ids, the first
//! [`HERO_SLOTS`] (the original allows a player eight heroes on the map); a hero's slot changes
//! when a hero of a lower id joins or leaves the player (later increments). The tile head is a
//! tile of the map, `(level × height + row) × width + column`, the observation's order; its size
//! is the map's tile count ([`crate::adventure::AdventureEnv::tile_count`]). After an entry that
//! takes no tile (`EndTurn`, the answers) the tile head allows exactly 0, as the battle's heads
//! after a non-cast entry (a policy that always samples the head never meets an empty mask on a
//! legal entry). A move's destination is a tile the hero's reachable-tiles query lists, not its
//! own; the action carries no path: the command takes the path the query chose
//! (`h3_game::legal_moves`). A client may send any other path the hero can pay (`apply` checks
//! each step); an agent of this encoding moves along the query's paths only. The heads' masks are
//! conditioned on the entry ([`legal_mask`], [`tile_mask`]); an illegal entry's tile mask is
//! empty. While the player has an open decision its answers are its only legal entries
//! (GAM-DEC-001: `h3_game::legal_commands` lists them alone and `h3_game::legal_moves` gives
//! nothing), each with tile 0 alone.
//!
//! Version 2 (E08 T14b) appended the answers.

use h3_adventure::{AdventureCommand, Pos, Reach};
use h3_contracts::{HeroId, PlayerId};
use h3_game::{Actor, Answer, GameCommand, GameRuleset, GameState, legal_commands, legal_moves};

/// Version of this encoding; bump on any change of the layout above.
pub const ACTION_ENCODING_VERSION: u32 = 2;

/// Hero slots per player: the most heroes the original allows a player on the map.
pub const HERO_SLOTS: usize = 8;

/// The entry that ends the turn.
pub const END_TURN: usize = 0;

/// The entry of the hero in slot 0's move; slot `s` is `MOVE_BASE + s`.
pub const MOVE_BASE: usize = 1;

/// The entry that accepts the player's open decision.
pub const ACCEPT: usize = MOVE_BASE + HERO_SLOTS;

/// The entry that declines the player's open decision.
pub const DECLINE: usize = ACCEPT + 1;

/// Number of entries.
pub const ENTRY_COUNT: usize = DECLINE + 1;

/// Which entries are legal.
pub type EntryMask = [bool; ENTRY_COUNT];

/// An action: an entry and its tile head (0 for an entry that takes no tile).
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

    /// Accepts the open decision.
    pub const fn accept() -> Self {
        Self {
            entry: ACCEPT,
            tile: 0,
        }
    }

    /// Declines the open decision.
    pub const fn decline() -> Self {
        Self {
            entry: DECLINE,
            tile: 0,
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

/// Fills `mask` with the legal entries of `player`: the commands [`legal_commands`] lists
/// (`EndTurn` while its turn runs, or with an open decision its two answers alone), and each hero
/// slot with a tile to move to ([`legal_moves`]: none with a decision open).
pub fn legal_mask(state: &GameState, rules: &GameRuleset, player: PlayerId, mask: &mut EntryMask) {
    mask.fill(false);
    let commands = legal_commands(state, rules, Actor::Player(player));
    for action in commands.iter().filter_map(|c| encode(state, player, c)) {
        mask[action.entry] = true;
    }
    for slot in 0..HERO_SLOTS {
        mask[MOVE_BASE + slot] = moves(state, rules, player, slot).is_some();
    }
}

/// Fills `mask` (resized to the tile count) with the legal tiles of `player` after choosing
/// `entry`: 0 alone after a legal entry that takes no tile (`EndTurn`, an answer), the reachable
/// tiles but its own of a hero's legal move; none after an illegal entry.
pub fn tile_mask(
    state: &GameState,
    rules: &GameRuleset,
    player: PlayerId,
    entry: usize,
    mask: &mut Vec<bool>,
) {
    mask.clear();
    mask.resize(tile_count(state), false);
    if let Some(command) = tileless(entry) {
        mask[0] = legal_commands(state, rules, Actor::Player(player)).contains(&command);
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

/// The command of an entry that takes no tile: `EndTurn` or an answer.
const fn tileless(entry: usize) -> Option<GameCommand> {
    match entry {
        END_TURN => Some(GameCommand::EndTurn),
        ACCEPT => Some(GameCommand::Answer(Answer::Accept)),
        DECLINE => Some(GameCommand::Answer(Answer::Decline)),
        _ => None,
    }
}

/// The command `player` issues with `action`, if the action names one: `EndTurn` or an answer
/// with tile 0, or a move of one of its heroes to a tile its reachable-tiles query lists (not its
/// own) along the query's path. Whether the command may be issued now (whose turn it is, an open
/// decision) is `apply`'s.
pub fn decode(
    state: &GameState,
    rules: &GameRuleset,
    player: PlayerId,
    action: Action,
) -> Option<GameCommand> {
    if let Some(command) = tileless(action.entry) {
        return (action.tile == 0).then_some(command);
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

/// The action that names `command` of `player`, if one does: `EndTurn` and the answers by their
/// entries (tile 0), a move of one of its heroes by the hero's slot and the path's last tile.
/// [`decode`] gives the command back when a move's path is the one the hero's query chose; a
/// move along another path decodes to the query's path to the same tile.
pub fn encode(state: &GameState, player: PlayerId, command: &GameCommand) -> Option<Action> {
    let (hero, path) = match command {
        GameCommand::EndTurn => return Some(Action::end_turn()),
        GameCommand::Answer(Answer::Accept) => return Some(Action::accept()),
        GameCommand::Answer(Answer::Decline) => return Some(Action::decline()),
        GameCommand::Adventure(AdventureCommand::MoveHero { hero, path }) => (*hero, path),
    };
    let slot = (0..HERO_SLOTS).find(|&slot| hero_in_slot(state, player, slot) == Some(hero))?;
    let tile = tile_index(state, *path.last()?)?;
    Some(Action::move_to(slot, tile))
}
