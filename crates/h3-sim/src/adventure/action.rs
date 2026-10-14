//! The adventure's action encoding (ADR-0017 point 8, E08 T9a), factorized as the battle's casts
//! are (ADR-0014 point 9), in two phases (encoding v3, E08 T19b1): a player in a battle acts in
//! the **battle phase**, everyone else in the **map phase** ([`Phase`], [`phase`]).
//!
//! In the map phase an action is an *entry*, and for a move a destination tile in the *tile
//! head* ([`MapAction`]). Entries (independent of the map and the ruleset, so agents see a fixed
//! number):
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
//! when a hero of a lower id joins or leaves the player (a battle's loser leaves the map). The
//! tile head is a tile of the map, `(level × height + row) × width + column`, the observation's
//! order; its size is the map's tile count ([`crate::adventure::AdventureEnv::tile_count`]). After
//! an entry that takes no tile (`EndTurn`, the answers) the tile head allows exactly 0, as the
//! battle's heads after a non-cast entry (a policy that always samples the head never meets an
//! empty mask on a legal entry). A move's destination is a tile the hero's reachable-tiles query
//! lists, not its own; a tile another player's hero stands on is one (the visit is a battle,
//! GAM-BAT-004). The action carries no path: the command takes the path the query chose
//! (`h3_game::legal_moves`). A client may send any other path the hero can pay (`apply` checks
//! each step); an agent of this encoding moves along the query's paths only. The heads' masks are
//! conditioned on the entry ([`legal_mask`], [`tile_mask`]); an illegal entry's tile mask is
//! empty. While the player has an open decision its answers are its only legal entries
//! (GAM-DEC-001: `h3_game::legal_commands` lists them alone and `h3_game::legal_moves` gives
//! nothing), each with tile 0 alone.
//!
//! In the battle phase an action is one of the battle lane's encoding (`crate::action`, its own
//! version: entries and the spell and target heads), for the side the player owns; its masks
//! ([`battle_mask`], [`spell_mask`], [`target_mask`], [`second_mask`]) are that encoding's for
//! the player's side, empty while the other side is to act, and every map entry is masked (ADR-0017
//! point 8). Each phase's masks are empty in the other phase.
//!
//! Version 2 (E08 T14b) appended the answers. Version 3 (E08 T19b1) added the battle phase: since
//! E08 T19a a move onto another player's hero starts a battle, which version 2 could not play, so
//! it left such tiles out of the masks and the decoder (`ADV-STUB-4`, closed).

use h3_adventure::{AdventureCommand, Pos, Reach};
use h3_battle::{BattleState, LegalCommands};
use h3_contracts::{BattleSide, HeroId, PlayerId};
use h3_game::{Actor, Answer, GameCommand, GameRuleset, GameState, legal_commands, legal_moves};

use crate::action as battle;

/// Version of this encoding; bump on any change of the layout above.
pub const ACTION_ENCODING_VERSION: u32 = 3;

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

/// The phase a player acts in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Phase {
    /// On the map: [`MapAction`]s.
    Adventure,
    /// In its battle: the battle's actions for the side it owns.
    Battle,
}

/// An action of the map phase: an entry and its tile head (0 for an entry that takes no tile).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct MapAction {
    /// The entry.
    pub entry: usize,
    /// The destination tile of a move, `(level × height + row) × width + column`; 0 otherwise.
    pub tile: u32,
}

/// An action of either phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
    /// An action of the map phase.
    Adventure(MapAction),
    /// An action of the battle phase: the battle lane's encoding (`crate::action`).
    Battle(battle::Action),
}

impl Action {
    /// Ends the turn.
    pub const fn end_turn() -> Self {
        Self::map(END_TURN, 0)
    }

    /// Moves the hero in `slot` to `tile`.
    pub const fn move_to(slot: usize, tile: u32) -> Self {
        Self::map(MOVE_BASE + slot, tile)
    }

    /// Accepts the open decision.
    pub const fn accept() -> Self {
        Self::map(ACCEPT, 0)
    }

    /// Declines the open decision.
    pub const fn decline() -> Self {
        Self::map(DECLINE, 0)
    }

    /// The map phase's action of `entry` with `tile`.
    pub const fn map(entry: usize, tile: u32) -> Self {
        Self::Adventure(MapAction { entry, tile })
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

/// The phase `player` acts in: the battle phase while it is in an open battle (GAM-BAT-001; its
/// first open decision is the battle), else the map phase.
pub fn phase(state: &GameState, player: PlayerId) -> Phase {
    if state.battle(player).is_some() {
        Phase::Battle
    } else {
        Phase::Adventure
    }
}

/// The open battle of `player` and the side it owns there, if it is in one.
pub fn battle_of(state: &GameState, player: PlayerId) -> Option<(&BattleState, BattleSide)> {
    let battle = state.battle(player)?;
    Some((battle.state(), battle.side_of(player)?))
}

/// The legal moves of the hero in `slot` of `player` ([`legal_moves`]), if any tile is reachable
/// but its own.
pub fn moves(
    state: &GameState,
    rules: &GameRuleset,
    player: PlayerId,
    slot: usize,
) -> Option<Reach> {
    let hero = hero_in_slot(state, player, slot)?;
    legal_moves(state, rules, Actor::Player(player), hero).filter(|r| r.tiles().next().is_some())
}

/// Fills `mask` with the legal map entries of `player`: the commands [`legal_commands`] lists
/// (`EndTurn` while its turn runs, or with an open decision its answers alone: `Decline`, and
/// `Accept` while the offer stands, E08 T15a), and each hero slot with a tile to move to
/// ([`legal_moves`]: none with a decision open); none in the battle phase, where the listed
/// commands are the battle's and no hero moves.
pub fn legal_mask(state: &GameState, rules: &GameRuleset, player: PlayerId, mask: &mut EntryMask) {
    mask.fill(false);
    for command in legal_commands(state, rules, Actor::Player(player)) {
        if let Some(entry) = tileless_entry(&command) {
            mask[entry] = true;
        }
    }
    for slot in 0..HERO_SLOTS {
        mask[MOVE_BASE + slot] = moves(state, rules, player, slot).is_some();
    }
}

/// Fills `mask` (resized to the tile count) with the legal tiles of `player` after choosing
/// `entry`: 0 alone after a legal entry that takes no tile (`EndTurn`, an answer), the reachable
/// tiles of a hero's legal move but its own; none after an illegal entry or in the battle phase.
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

/// Fills `mask` with the legal entries of the battle phase for `player`: its battle's
/// ([`battle::legal_mask`]) for the side it owns while that side is to act; none otherwise.
/// `legal` is scratch space.
pub fn battle_mask(
    state: &GameState,
    rules: &GameRuleset,
    player: PlayerId,
    legal: &mut LegalCommands,
    mask: &mut battle::ActionMask,
) {
    mask.fill(false);
    if let Some((battle, side)) = battle_of(state, player) {
        battle::legal_mask(battle, rules.battle(), side, legal, mask);
    }
}

/// Fills `mask` with the legal spells of the battle phase for `player` after `entry`
/// ([`battle::spell_mask`]); none outside a battle.
pub fn spell_mask(
    state: &GameState,
    rules: &GameRuleset,
    player: PlayerId,
    entry: usize,
    legal: &mut LegalCommands,
    mask: &mut battle::SpellMask,
) {
    mask.fill(false);
    if let Some((battle, side)) = battle_of(state, player) {
        battle::spell_mask(battle, rules.battle(), side, entry, legal, mask);
    }
}

/// Fills `mask` with the legal targets of the battle phase for `player` after `entry` and
/// `spell` ([`battle::target_mask`]); none outside a battle.
pub fn target_mask(
    state: &GameState,
    rules: &GameRuleset,
    player: PlayerId,
    prefix: (usize, u16),
    legal: &mut LegalCommands,
    mask: &mut battle::TargetMask,
) {
    mask.fill(false);
    if let Some((battle, side)) = battle_of(state, player) {
        battle::target_mask(battle, rules.battle(), side, prefix, legal, mask);
    }
}

/// Fills `mask` with the legal second targets of the battle phase for `player` after `entry`,
/// `spell` and `target` ([`battle::second_mask`]); none outside a battle.
pub fn second_mask(
    state: &GameState,
    rules: &GameRuleset,
    player: PlayerId,
    prefix: (usize, u16, u16),
    legal: &mut LegalCommands,
    mask: &mut battle::TargetMask,
) {
    mask.fill(false);
    if let Some((battle, side)) = battle_of(state, player) {
        battle::second_mask(battle, rules.battle(), side, prefix, legal, mask);
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

/// The entry of a command that takes no tile: `EndTurn` or an answer.
const fn tileless_entry(command: &GameCommand) -> Option<usize> {
    match command {
        GameCommand::EndTurn => Some(END_TURN),
        GameCommand::Answer(Answer::Accept) => Some(ACCEPT),
        GameCommand::Answer(Answer::Decline) => Some(DECLINE),
        GameCommand::Adventure(_) | GameCommand::Battle(_) => None,
    }
}

/// The command `player` issues with `action`, if the action names one. In the map phase:
/// `EndTurn` or an answer with tile 0, or a move of one of its heroes to a tile its
/// reachable-tiles query lists (not its own) along the query's path. In the battle phase: the
/// battle command the battle's encoding decodes the action to ([`battle::decode`]), for the
/// player's battle. Whether the command may be issued now (whose turn it is, an open decision,
/// the side to act) is `apply`'s.
pub fn decode(
    state: &GameState,
    rules: &GameRuleset,
    player: PlayerId,
    action: Action,
) -> Option<GameCommand> {
    let action = match action {
        Action::Adventure(action) => action,
        Action::Battle(action) => {
            let (battle, _) = battle_of(state, player)?;
            return battle::decode(battle, rules.battle(), action).map(GameCommand::Battle);
        }
    };
    if let Some(command) = tileless(action.entry) {
        return (action.tile == 0).then_some(command);
    }
    let hero = hero_in_slot(state, player, action.entry.checked_sub(MOVE_BASE)?)?;
    let to = tile_pos(state, action.tile)?;
    let reach = state
        .adventure()
        .reach(rules.core(), rules.adventure(), state.heroes(), hero)?;
    let path = reach.path(to).filter(|path| !path.is_empty())?;
    Some(GameCommand::Adventure(AdventureCommand::MoveHero {
        hero,
        path,
    }))
}

/// The action that names `command` of `player`, if one does: `EndTurn` and the answers by their
/// entries (tile 0), a move of one of its heroes by the hero's slot and the path's last tile, a
/// battle command by the battle's encoding for the player's battle ([`battle::encode`]).
/// [`decode`] gives the command back when a move's path is the one the hero's query chose; a
/// move along another path decodes to the query's path to the same tile.
pub fn encode(
    state: &GameState,
    rules: &GameRuleset,
    player: PlayerId,
    command: &GameCommand,
) -> Option<Action> {
    let (hero, path) = match command {
        GameCommand::Adventure(AdventureCommand::MoveHero { hero, path }) => (*hero, path),
        GameCommand::Battle(command) => {
            let (battle, _) = battle_of(state, player)?;
            return battle::encode(battle, rules.battle(), *command).map(Action::Battle);
        }
        other => return tileless_entry(other).map(|entry| Action::map(entry, 0)),
    };
    let slot = (0..HERO_SLOTS).find(|&slot| hero_in_slot(state, player, slot) == Some(hero))?;
    let tile = tile_index(state, *path.last()?)?;
    Some(Action::move_to(slot, tile))
}
