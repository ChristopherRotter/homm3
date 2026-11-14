//! The server's published language and session core (E16, ADR-0018): the wire messages, their
//! bounded codec and the mapping to and from the engine's types, and the sans-I/O session core
//! that hosts one game for its seats. Nothing here touches a socket, a clock or a file: the host
//! (`h3-server`) passes bytes and the time in and sends what comes out.
//!
//! - [`wire`]: the published language, every message in both directions (M1 T1);
//! - [`codec`]: one JSON document per frame, bounded and panic-free (M1 T1);
//! - [`map`]: the mapping to and from the battle's commands, events and observation, which
//!   projects everything outbound for its viewer (M1 T1);
//! - [`connection`]: a connection's own messages, the hello and the ping (M1 T2);
//! - [`session`]: the session core, one hosted battle: seats, tokens, the authoritative loop, the
//!   bot hook, the viewers' records and the journal (M1 T2);
//! - [`lobby`]: the server's offers and games, created within limits and listed (M2 T1);
//! - [`account`]: login and display names, keys as hex, the bytes a login signs (M3 T1);
//! - [`ladder`]: the ladders' queues, the matcher and a match's answers (M4 T2);
//! - [`card`]: the server's signed card, its bounds and its reading (M6 T2), the chain of vouching
//!   of the cards it relays (M6 T3);
//! - [`list`]: lists of servers signed by a list's key, their bounds and their reading (M6 T5).
//!
//! Logic crate in the adapter layer (ADR-0018 point 1): pure, deterministic, `no_std` + `alloc`
//! (ADR-0003, ADR-0005). The lint header below is required and checked by `cargo xtask check-arch`.
#![no_std]
#![forbid(unsafe_code)]
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::todo,
    clippy::unimplemented,
    clippy::unreachable,
    clippy::float_arithmetic,
    clippy::print_stdout,
    clippy::print_stderr,
    clippy::exit
)]
#![warn(missing_docs)]

extern crate alloc;
#[cfg(test)]
extern crate std;

pub mod account;
pub mod card;
pub mod codec;
pub mod connection;
pub mod ladder;
pub mod list;
pub mod lobby;
pub mod map;
pub mod session;
pub mod wire;

/// The version of the wire protocol: sent in the hello and bumped on every change of the messages
/// or their encoding (ADR-0018 point 4). Before 1.0 a server accepts exactly its own version.
/// v21 (R01, the content split): the manifest and the offers carry the battle part's hash beside
/// the core's. v22 (E17 M1, CLT-Q3): the observation names the natural obstacles' types, the key
/// tables list the obstacle types. v23: a two-hex stack's position (its hex, the hexes of moves
/// and attacks) is its rear hex, as in the original (BAT-GEO-002). v24 (E16 M6 T1, ADR-0021 point
/// 1): the hello names a nonce and the address the client dialed; the welcome carries the server's
/// key and its signature. v25 (E16 M6 T2, ADR-0021 point 2): `Describe` and `Described`, the
/// server's signed card. v26 (E16 M6 T3, ADR-0021 point 3): `ListPeers` and `Peers`, the cards a
/// server relays. v27 (E16 M6 T5, ADR-0021 point 5): `Lists` and `ListsAre`, the signed lists of
/// servers a server serves. v28 (E04c T2b, ADR-0022): the observation names the special
/// battlefield and the anti-magic garrison; the key tables list the battlefields. v29 (E17
/// CLT-Q1): a walker's `Moved` comes hex by hex, one per position of its path (BAT-MOV-003); the
/// message is unchanged, its meaning is new. v30 (E04c T5, ADR-0023): the observation names the
/// town the battle is fought at (`wire::Siege`); the events `GateOpened`, `GateClosed`,
/// `MoatDamage`. v31 (E04c T6, ADR-0023 point 6): the wall shots: the command `ShootWall`, the
/// events `WallShot`, `WallQuaked`, `WallDestroyed`, the rejections `CannotShootWalls` and
/// `NoSuchPart`, the town's wall parts, a hero's command of its Catapult. v32 (E04c T7d, ADR-0023
/// point 7): the observation names the game's difficulty and, per side, whether its player is
/// human. v33 (E04c T7e, ADR-0013 point 4): the own hero's Ballista names its Artillery level.
/// v34 (the server lane's inbox, 2026-10-19): the query `CreatureCastTargets`; a side's
/// `may_retreat` is as the viewer knows it (a town's Escape Tunnel shows to its defender only,
/// BAT-Q50 (4)): the field is unchanged, its meaning is new. v35 (CLT-Q4, CLT-Q6): for viewers,
/// the observation names the battlefield picture's variant, each side's player colour and a
/// hero's sex where its map gave it one. v36 (E17 M5 T5a, BAT-MAG-037): the event `NotCreated`,
/// a clone or summoned stack that found no place. v37 (CLT-Q40): public, for viewers: each
/// side's hero's morale and luck effects with their sources, a siege town's morale and luck
/// buildings and each stack's unclamped morale and luck; a stack's `morale` and `luck` are
/// unchanged (their documentation said "before the timed effects", they always held them). v38
/// (the server lane's inbox, CLT-Q20, CLT-STUB-3): the messages are unchanged, their order is
/// new: every viewer hears the battle's start's events in an `Events` message when the game
/// starts, before any command, and its last `Snapshot`, the battle as it ended, before
/// `GameEnded` (a seat's own hero's Eagle Eye picks, which no event shows). v39 (E16 M5 T5a): the
/// whole game's messages `GameSnapshot`, `GameEvents`, `GameTurn`, `GameAccepted` and
/// `GameRejected`, which an in-process game session sends its players (`session::game`). v40
/// (E08 T105b2, ADV-TEL-008): the game's adventure command `Revisit` (the entered object under a
/// hero visited again) and its rejection `NothingToRevisit`. v41 (E09 T18c, TWN-EVT-001): the
/// game's event `TownEventFired`, a town event fired at its owner's turn start. v42 (SRV-Q26 (b),
/// the adventure lane's inbox): the game's adventure event `MovementChanged`, a hero's movement
/// points left and day's maximum after a command that changed them, its owner's alone. v43 (E16
/// M5 T5c, SRV-Q26 (a)): the whole game's message `GameReveal`, what a viewer newly sees after a
/// command that explored tiles for it (`h3_game::reveal`); a view's objects in tile order
/// (`wire::game::Object::place`), no longer the map's: the field is unchanged, its order is new;
/// a game's battle's last `Snapshot`, as each owner's side saw it when it ended (SRV-Q27 (1),
/// `h3_game::Sink::battle_ended`), right after the `GameEvents` that end it: the message is
/// unchanged, its order is new. v44 (E16 M5 T7a1): a reveal's tiles packed
/// (`wire::game::AdventureReveal`: a palette of their views and one rectangle per level, an index
/// per tile, no longer a list of tiles each with its view); a game's player may get a fresh
/// `GameSnapshot` after a command (`session::game`: its record is cut at its snapshots). v45 (E16
/// M5 T8a): a whole game's seats over the wire: the client messages `GameJoin` (a player's seat,
/// taken by account or token as a battle's) and `GameCommand`, the answer `GameJoined`, the
/// refusal `NoSuchSeat`, the session's rejections `NotSeated` and `NotStarted` (`GameRejected`,
/// numbered 0 for a connection without a seat); a re-taken seat gets a fresh `GameSnapshot`. v46
/// (E16 M5 T8b): a seat's commands numbered against resends (`GameCommand`'s `seq` strictly
/// increasing, a resend answered as the first time or rejected `Stale`), `GameJoined`'s `last`, the
/// seat's last number; the host's pause of a whole game (`Pause`, another seat's ask told as
/// `GamePauseAsked`, rejected `AlreadyAsked` once asked; a paused game rejects `NotStarted`). v47
/// (E16 M5 T9a): whole games in the lobby: `Games` lists the server's map offers
/// (`wire::game::MapOffer`) and its whole games (`wire::game::MapGameInfo`), the client message
/// `CreateMapGame` creates one and takes a player's seat in it. v48 (E16 M5 T9b): a whole game's
/// seat downloads its record (`FetchRecord`, answered `Record` with the seat's player's record, its
/// first line a `wire::game::GameRecordHeader`; `GameRejected { NotSeated }` without a seat). v49
/// (E16 M5 T10): a map offer names the computer that plays a whole game's unowned battle sides
/// (`wire::game::MapOffer`'s `computer`, `wire::game::Computer`), whose commands the session
/// tells as the system actor's. v50 (E16 M5 T11): a whole game's queries, the client message
/// `GameQuery` (`wire::game::GameQuery`: a hero's moves, a town's hall, a battle's targets), the
/// answer `GameAnswer` (unnumbered, not recorded; `wire::game::Moves`, a path tree), and at the
/// game's start each player's town events that fired at its turn start inside the build, in a
/// `GameEvents` before any turn: the message is unchanged, its order is new. v51 (E16 M5 T12): a
/// whole game's fresh `GameSnapshot` past half a record's bound comes inside a battle too
/// (`GameObservation::battle` holds it), and a battle's own `Snapshot` opens only a battle the
/// player does not hold yet, by its identity: the messages are unchanged, their order is new. v52
/// (E08 T114a2, the adventure lane's, `cross-lane`): a map garrison's exchange
/// (`wire::game::Decision::Exchange`, the event `ExchangeOpened`), its army in the army commands
/// (`wire::game::ArmyRef::MapGarrison`) and the one owner rule's refusals
/// (`wire::game::ArmyReject`'s `NoGarrison`, `NotOwn`, `OnlyEmpty`).
/// v53 (E09 T26a2): the week's kind (TWN-GRO-002): `WeekStarted`'s `kind` and the game
/// observation's `week` (`wire::game::Week`), the fold keeping it. v54 (E08 T114b, the adventure
/// lane's, `cross-lane`): the map garrisons a viewer sees (`wire::game::GameObservation`'s and
/// `GameReveal`'s `garrisons`, `wire::game::MapGarrison`) and their changes (the events
/// `GarrisonChanged`, `GarrisonStackSeen`). v55 (E09 T26b, the
/// town lane's, `cross-lane`): a town's owner sees its available creatures and its Portal of
/// Summoning (TWN-VEW-004: `wire::game::TownBuildings`' `available` and `portal`,
/// `wire::game::Summoned`), told as `TownChanged` after the week, the month, a town event's
/// creatures and each Portal's draw: the messages' order is new where a neutral garrison's week or
/// a count changes a view.
/// v56 (E16 M5 T14, `cross-lane`): the system actor's own commands for a player
/// (`wire::game::GameCommand::System`, `wire::game::SystemCommand`: its turn ended, its
/// decision's default answer, its battle side's command) and their refusal from a seat
/// (`wire::game::GameReject::SystemOnly`). v57 (E08 T115a, the adventure lane's, `cross-lane`):
/// the battle bonuses are the hero on the map's (`wire::game::OwnMapHero`'s `bonuses`, with the
/// once-marks' amounts, `wire::game::VisitBonus`, and its `timed` marks), no longer the details'
/// (`HeroDetails`' `bonuses` and the changes `Bonus`, `ClearBonuses` gone); their changes are the
/// adventure events `BonusChanged`, `OnceMarkChanged` and `TimedMarkChanged`, the owner's alone,
/// after `MovementChanged`.
/// v58 (E16 M5 T16): the whole game's clock (`ServerMessage::GameClock`, `wire::game::GameClock`:
/// a player's turn timer, its battle's chess clocks against another player's hero, its level-up's
/// time, the game waiting for another's level-up); the system's commands at a timer's end (a
/// turn ended, a decision's default answer, a timed-out battle side played by the computer).
/// v59 (E09 T27, the town lane's, `cross-lane`): a town's recruitment
/// (TWN-REC-001, -002): the town command `Recruit`, its refusals `NotRecruiting`, `NotStanding`
/// and `NothingSummoned`, a hire dialog's town garrison target (`wire::game::HireTarget::Garrison`)
/// and its source named as `wire::game::HireSeen` (a map object, a town's index or its Portal) in
/// the dialog's view and in `HireOpened`, in place of the object's type and anchor.
/// v60 (E16 M5 T17, after the town lane's T27 took v59): a map offer shows how its games are
/// timed (`wire::game::MapOffer`'s `clock`, `wire::game::OfferClock`: the turn's duration, the
/// level-up's time, a battle's allowance between players).
pub const PROTOCOL_VERSION: u32 = 60;

/// The version of the record's format (`wire::RecordHeader`, the `Record` message): bumped when
/// its layout changes. v3 (CLT-Q20, CLT-STUB-3): a seat's record holds the start's `Events` before
/// its first command's, and the last `Snapshot` before `GameEnded`; an `Events` message is no
/// longer always a command's. v4 (E16 M5 T5b): a whole game's record, its header
/// `wire::game::GameRecordHeader` (the game's manifest and the player), every further line one
/// numbered message the player was sent (`session::game`); a battle's record is unchanged. v5
/// (E16 M5 T11, as v3 for a battle): a whole game's record may hold the start's `GameEvents` (its
/// player's town events that fired inside the build) before its first command's; a `GameEvents`
/// message is no longer always a command's.
pub const RECORD_FORMAT_VERSION: u32 = 5;

#[cfg(test)]
mod tests;
