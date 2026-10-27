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
//! | 9 | Accept the player's open offer (E08 T14b: a Cartographer's offer) |
//! | 10 | Decline it; close a meeting or an army dialog; leave a School or a University (E08 T63a) |
//! | 11 | The first answer of the player's open two-way decision: a level-up's first offer (E08 T30: the dialog's left), a chest's gold, the Arena's first skill, a School's first (T63a) |
//! | 12 | Its second answer: a level-up's second offer (the dialog's right), a chest's experience, the Arena's second skill, a School's second (T63a) |
//! | 13 + s, s < 4 | Buy the skill in slot `s` of the player's open University (E08 T63a) |
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
//! nothing), each with tile 0 alone. The game's answers are typed by the decision's kind
//! (`h3_game::Answer`, E08 T63a); an entry names the open decision's answer of its role
//! ([`answer_entry`]): an offer's `Accept` and `Decline`, a level-up's two offers (GAM-DEC-002), a
//! meeting's or an army dialog's `Close` (GAM-DEC-003, GAM-DEC-004), a chest's, the Arena's and a
//! School's two answers and a School's leaving (GAM-DEC-005), a University's purchases of the
//! first [`UNIVERSITY_SLOTS`] slots and its leaving (GAM-DEC-006). With no open decision of that
//! role the entry decodes to the offer's or the level-up's answer, or the purchase, which `apply`
//! rejects.
//!
//! In the battle phase an action is one of the battle lane's encoding (`crate::action`, its own
//! version: entries and the spell and target heads), for the side the player owns; its masks
//! ([`battle_mask`], [`spell_mask`], [`target_mask`], [`second_mask`]) are that encoding's for
//! the player's side, empty while the other side is to act, and every map entry is masked (ADR-0017
//! point 8). Each phase's masks are empty in the other phase.
//!
//! Who acts is a [`Seat`]: a player, or the **neutral seat** (E08 T20b3a), which plays a battle's
//! unowned side (a wandering monster's) as the system actor (`h3_game::Actor::System`). It acts in
//! the battle phase only, with the same encoding for the side to act of the first battle whose
//! side to act no player owns (`h3_game::GameState::unowned_battle`); its map masks are empty. The
//! seat is not part of an action: the layout above is unchanged (still version 3).
//!
//! Version 2 (E08 T14b) appended the answers. Version 3 (E08 T19b1) added the battle phase: since
//! E08 T19a a move onto another player's hero starts a battle, which version 2 could not play, so
//! it left such tiles out of the masks and the decoder (`ADV-STUB-4`, closed). Version 4 (E08
//! T30) appended a level-up's two answers. Version 5 (E08 T63a, the adventure lane's `cross-lane`
//! change with the user's consent) moved the answers onto the typed ones by role and appended a
//! University's purchases.

use h3_adventure::{AdventureCommand, Pos, Reach};
use h3_battle::{BattleState, LegalCommands};
use h3_contracts::{BattleSide, HeroId, PlayerId};
use h3_game::{
    Actor, Answer, ArenaChoice, ChestChoice, Consent, GameCommand, GameRuleset, GameState,
    SchoolChoice, SkillOffer, UniversityAnswer, legal_commands, legal_moves,
};

use crate::action as battle;

/// Version of this encoding; bump on any change of the layout above.
pub const ACTION_ENCODING_VERSION: u32 = 5;

/// Hero slots per player: the most heroes the original allows a player on the map.
pub const HERO_SLOTS: usize = 8;

/// The entry that ends the turn.
pub const END_TURN: usize = 0;

/// The entry of the hero in slot 0's move; slot `s` is `MOVE_BASE + s`.
pub const MOVE_BASE: usize = 1;

/// The entry that accepts the player's open offer.
pub const ACCEPT: usize = MOVE_BASE + HERO_SLOTS;

/// The entry that declines the player's open offer, closes its meeting or army dialog, or leaves
/// its School or University (E08 T63a).
pub const DECLINE: usize = ACCEPT + 1;

/// The entry of the first answer of the player's open two-way decision: a level-up's first offer
/// (E08 T30), a chest's gold, the Arena's first skill, a School's first skill (E08 T63a).
pub const FIRST: usize = DECLINE + 1;

/// The entry of the second answer of the player's open two-way decision: a level-up's second
/// offer (E08 T30), a chest's experience, the Arena's second skill, a School's second (E08 T63a).
pub const SECOND: usize = FIRST + 1;

/// The University's slots the entries buy (E08 T63a): the original's dialog's four (a pack's
/// University with more offers sells the rest to no agent of this encoding).
pub const UNIVERSITY_SLOTS: usize = 4;

/// The entry that buys the skill in slot 0 of the player's open University; slot `s` is
/// `BUY_BASE + s` (E08 T63a).
pub const BUY_BASE: usize = SECOND + 1;

/// Number of entries.
pub const ENTRY_COUNT: usize = BUY_BASE + UNIVERSITY_SLOTS;

/// Which entries are legal.
pub type EntryMask = [bool; ENTRY_COUNT];

/// Who acts (ADR-0017 point 8): a player, or the neutral seat, which plays a battle's unowned side
/// (a wandering monster's) as the system actor (E08 T20b3a).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Seat {
    /// A player: its turn's actions on the map, or its side's in its battle.
    Player(PlayerId),
    /// The neutral seat: the unowned side's actions in the first battle whose side to act no
    /// player owns ([`GameState::unowned_battle`]).
    Neutral,
}

impl Seat {
    /// The actor the seat's commands are issued as: the player, or the system actor.
    pub const fn actor(self) -> Actor {
        match self {
            Self::Player(player) => Actor::Player(player),
            Self::Neutral => Actor::System,
        }
    }

    /// The player of the seat; none for the neutral seat.
    pub const fn player(self) -> Option<PlayerId> {
        match self {
            Self::Player(player) => Some(player),
            Self::Neutral => None,
        }
    }
}

impl From<PlayerId> for Seat {
    fn from(player: PlayerId) -> Self {
        Self::Player(player)
    }
}

/// The phase a seat acts in.
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

    /// Takes the open two-way decision's first answer (E08 T30, T63a).
    pub const fn first() -> Self {
        Self::map(FIRST, 0)
    }

    /// Takes the open two-way decision's second answer (E08 T30, T63a).
    pub const fn second() -> Self {
        Self::map(SECOND, 0)
    }

    /// Buys the skill in `slot` of the open University (E08 T63a).
    pub const fn buy(slot: usize) -> Self {
        Self::map(BUY_BASE + slot, 0)
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

/// The phase `seat` acts in: for a player the battle phase while it is in an open battle
/// (GAM-BAT-001; its first open decision is the battle), else the map phase; the battle phase for
/// the neutral seat, which has no other.
pub fn phase(state: &GameState, seat: impl Into<Seat>) -> Phase {
    match seat.into() {
        Seat::Player(player) if state.battle(player).is_none() => Phase::Adventure,
        _ => Phase::Battle,
    }
}

/// The open battle of `seat` and its side there: a player's battle and the side it owns, if it is
/// in one; for the neutral seat the first battle whose side to act no player owns, and that side.
pub fn battle_of(state: &GameState, seat: impl Into<Seat>) -> Option<(&BattleState, BattleSide)> {
    match seat.into() {
        Seat::Player(player) => {
            let battle = state.battle(player)?;
            Some((battle.state(), battle.side_of(player)?))
        }
        Seat::Neutral => {
            let battle = state.unowned_battle()?.state();
            Some((battle, battle.side_to_act()?))
        }
    }
}

/// The legal moves of the hero in `slot` of `seat`'s player ([`legal_moves`]), if any tile is
/// reachable but its own; none for the neutral seat.
pub fn moves(
    state: &GameState,
    rules: &GameRuleset,
    seat: impl Into<Seat>,
    slot: usize,
) -> Option<Reach> {
    let player = seat.into().player()?;
    let hero = hero_in_slot(state, player, slot)?;
    legal_moves(state, rules, Actor::Player(player), hero).filter(|r| r.tiles().next().is_some())
}

/// Fills `mask` with the legal map entries of `seat`: the commands [`legal_commands`] lists
/// (`EndTurn` while its turn runs, or with an open decision its answers alone: `Decline`, and
/// `Accept` while the offer stands, E08 T15a; a level-up's `First` and `Second`, E08 T30), and each hero slot with a tile to move to
/// ([`legal_moves`]: none with a decision open); none in the battle phase, where the listed
/// commands are the battle's and no hero moves, so none for the neutral seat.
pub fn legal_mask(
    state: &GameState,
    rules: &GameRuleset,
    seat: impl Into<Seat>,
    mask: &mut EntryMask,
) {
    let seat = seat.into();
    mask.fill(false);
    for command in legal_commands(state, rules, seat.actor()) {
        if let Some(entry) = tileless_entry(&command) {
            mask[entry] = true;
        }
    }
    for slot in 0..HERO_SLOTS {
        mask[MOVE_BASE + slot] = moves(state, rules, seat, slot).is_some();
    }
}

/// Fills `mask` (resized to the tile count) with the legal tiles of `seat` after choosing
/// `entry`: 0 alone after a legal entry that takes no tile (`EndTurn`, an answer), the reachable
/// tiles of a hero's legal move but its own; none after an illegal entry or in the battle phase
/// (none for the neutral seat).
pub fn tile_mask(
    state: &GameState,
    rules: &GameRuleset,
    seat: impl Into<Seat>,
    entry: usize,
    mask: &mut Vec<bool>,
) {
    let seat = seat.into();
    mask.clear();
    mask.resize(tile_count(state), false);
    if tileless(entry) {
        mask[0] = legal_commands(state, rules, seat.actor())
            .iter()
            .any(|command| tileless_entry(command) == Some(entry));
    } else if let Some(reach) = entry
        .checked_sub(MOVE_BASE)
        .and_then(|slot| moves(state, rules, seat, slot))
    {
        for (pos, _) in reach.tiles() {
            if let Some(index) = tile_index(state, pos) {
                mask[index as usize] = true;
            }
        }
    }
}

/// Fills `mask` with the legal entries of the battle phase for `seat` ([`battle_of`]): its
/// battle's ([`battle::legal_mask`]) for its side while that side is to act; none otherwise.
/// `legal` is scratch space.
pub fn battle_mask(
    state: &GameState,
    rules: &GameRuleset,
    seat: impl Into<Seat>,
    legal: &mut LegalCommands,
    mask: &mut battle::ActionMask,
) {
    mask.fill(false);
    if let Some((battle, side)) = battle_of(state, seat) {
        battle::legal_mask(battle, rules.battle(), side, legal, mask);
    }
}

/// Fills `mask` with the legal spells of the battle phase for `seat` after `entry`
/// ([`battle::spell_mask`]); none outside a battle.
pub fn spell_mask(
    state: &GameState,
    rules: &GameRuleset,
    seat: impl Into<Seat>,
    entry: usize,
    legal: &mut LegalCommands,
    mask: &mut battle::SpellMask,
) {
    mask.fill(false);
    if let Some((battle, side)) = battle_of(state, seat) {
        battle::spell_mask(battle, rules.battle(), side, entry, legal, mask);
    }
}

/// Fills `mask` with the legal targets of the battle phase for `seat` after `entry` and
/// `spell` ([`battle::target_mask`]); none outside a battle.
pub fn target_mask(
    state: &GameState,
    rules: &GameRuleset,
    seat: impl Into<Seat>,
    prefix: (usize, u16),
    legal: &mut LegalCommands,
    mask: &mut battle::TargetMask,
) {
    mask.fill(false);
    if let Some((battle, side)) = battle_of(state, seat) {
        battle::target_mask(battle, rules.battle(), side, prefix, legal, mask);
    }
}

/// Fills `mask` with the legal second targets of the battle phase for `seat` after `entry`,
/// `spell` and `target` ([`battle::second_mask`]); none outside a battle.
pub fn second_mask(
    state: &GameState,
    rules: &GameRuleset,
    seat: impl Into<Seat>,
    prefix: (usize, u16, u16),
    legal: &mut LegalCommands,
    mask: &mut battle::TargetMask,
) {
    mask.fill(false);
    if let Some((battle, side)) = battle_of(state, seat) {
        battle::second_mask(battle, rules.battle(), side, prefix, legal, mask);
    }
}

/// Whether `entry` takes no tile: `EndTurn` or an answer.
const fn tileless(entry: usize) -> bool {
    entry == END_TURN || (entry >= ACCEPT && entry < ENTRY_COUNT)
}

/// The entry of `answer` by its role (E08 T63a; the module's table): an offer's acceptance; a
/// declined offer, a closed meeting or army dialog, a School or a University left; the first or
/// the second of a level-up's, a chest's, the Arena's or a School's answers; a University's
/// purchase of one of its first [`UNIVERSITY_SLOTS`] slots (none for a later one).
pub const fn answer_entry(answer: Answer) -> Option<usize> {
    match answer {
        Answer::Offer(Consent::Accept) => Some(ACCEPT),
        Answer::Offer(Consent::Decline)
        | Answer::Close
        | Answer::School(SchoolChoice::Leave)
        | Answer::University(UniversityAnswer::Leave) => Some(DECLINE),
        Answer::LevelUp(SkillOffer::First)
        | Answer::Chest(ChestChoice::Gold)
        | Answer::Arena(ArenaChoice::Attack)
        | Answer::School(SchoolChoice::First) => Some(FIRST),
        Answer::LevelUp(SkillOffer::Second)
        | Answer::Chest(ChestChoice::Experience)
        | Answer::Arena(ArenaChoice::Defense)
        | Answer::School(SchoolChoice::Second) => Some(SECOND),
        Answer::University(UniversityAnswer::Buy(slot)) => {
            let slot = slot as usize;
            if slot < UNIVERSITY_SLOTS {
                Some(BUY_BASE + slot)
            } else {
                None
            }
        }
    }
}

/// The answer an answer entry names for `seat`'s player (E08 T63a): the answer of its role that
/// [`legal_commands`] lists for its open decision, else the entry's answer to an offer or a
/// level-up, or its purchase, which `apply` rejects with the reason (no decision, the wrong
/// kind, the slot or the price).
fn answer_of(state: &GameState, rules: &GameRuleset, seat: Seat, entry: usize) -> Option<Answer> {
    legal_commands(state, rules, seat.actor())
        .into_iter()
        .find_map(|command| match command {
            GameCommand::Answer(answer) if answer_entry(answer) == Some(entry) => Some(answer),
            _ => None,
        })
        .or(match entry {
            ACCEPT => Some(Answer::Offer(Consent::Accept)),
            DECLINE => Some(Answer::Offer(Consent::Decline)),
            FIRST => Some(Answer::LevelUp(SkillOffer::First)),
            SECOND => Some(Answer::LevelUp(SkillOffer::Second)),
            _ => u8::try_from(entry - BUY_BASE)
                .ok()
                .map(|slot| Answer::University(UniversityAnswer::Buy(slot))),
        })
}

/// The entry of a command that takes no tile: `EndTurn` or an answer ([`answer_entry`]).
const fn tileless_entry(command: &GameCommand) -> Option<usize> {
    match command {
        GameCommand::EndTurn => Some(END_TURN),
        GameCommand::Answer(answer) => answer_entry(*answer),
        // E08 T51a: the management commands have no entry (decision 99: the AI lane's call).
        GameCommand::Adventure(_) | GameCommand::Battle(_) | GameCommand::Manage(_) => None,
    }
}

/// The command `seat` issues with `action`, if the action names one. In the map phase:
/// `EndTurn` or an answer with tile 0, or a move of one of its player's heroes to a tile its
/// reachable-tiles query lists (not its own) along the query's path; none for the neutral seat.
/// In the battle phase: the battle command the battle's encoding decodes the action to
/// ([`battle::decode`]), for the seat's battle ([`battle_of`]). Whether the command may be issued
/// now (whose turn it is, an open decision, the side to act) is `apply`'s.
pub fn decode(
    state: &GameState,
    rules: &GameRuleset,
    seat: impl Into<Seat>,
    action: Action,
) -> Option<GameCommand> {
    let seat = seat.into();
    let action = match action {
        Action::Adventure(action) => action,
        Action::Battle(action) => {
            let (battle, _) = battle_of(state, seat)?;
            return battle::decode(battle, rules.battle(), action).map(GameCommand::Battle);
        }
    };
    let player = seat.player()?;
    if tileless(action.entry) {
        if action.tile != 0 {
            return None;
        }
        if action.entry == END_TURN {
            return Some(GameCommand::EndTurn);
        }
        return answer_of(state, rules, seat, action.entry).map(GameCommand::Answer);
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

/// The action that names `command` of `seat`, if one does: `EndTurn` and the answers by their
/// entries (tile 0), a move of one of its player's heroes by the hero's slot and the path's last
/// tile (none of these for the neutral seat), a battle command by the battle's encoding for the
/// seat's battle ([`battle::encode`]). [`decode`] gives the command back when a move's path is
/// the one the hero's query chose; a move along another path decodes to the query's path to the
/// same tile.
pub fn encode(
    state: &GameState,
    rules: &GameRuleset,
    seat: impl Into<Seat>,
    command: &GameCommand,
) -> Option<Action> {
    let seat = seat.into();
    if let GameCommand::Battle(command) = command {
        let (battle, _) = battle_of(state, seat)?;
        return battle::encode(battle, rules.battle(), *command).map(Action::Battle);
    }
    let player = seat.player()?;
    let (hero, path) = match command {
        GameCommand::Adventure(AdventureCommand::MoveHero { hero, path }) => (*hero, path),
        other => return tileless_entry(other).map(|entry| Action::map(entry, 0)),
    };
    let slot = (0..HERO_SLOTS).find(|&slot| hero_in_slot(state, player, slot) == Some(hero))?;
    let tile = tile_index(state, *path.last()?)?;
    Some(Action::move_to(slot, tile))
}
