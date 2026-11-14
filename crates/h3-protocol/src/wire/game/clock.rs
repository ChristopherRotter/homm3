//! What a player sees of a whole game's clock (E16 M5 T16, v58; SRV-Q31, SRV-Q32): its own
//! turn's timer, the chess clocks of its battle against another player's hero, its level-up's
//! time, and whether the game waits for another player's level-up (never whose, nor its offers).
//! Each time is what was left when the message was sent; a running one counts down from there on
//! the client's own clock.
//!
//! A map offer shows how its games are timed (E16 M5 T17, v60: [`OfferClock`]), before anyone
//! joins.
//!
//! Provisional (the 2b review list): the shapes; a player told only its own timers (another
//! player's turn timer would show where it fights: its timer stops in a battle between players);
//! milliseconds as 32 bits (49 days).

use serde::{Deserialize, Serialize};

/// A player's view of the game's clock: sent as `GameClock` whenever it changes otherwise than by
/// counting down, and after a join's snapshot.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GameClock {
    /// Its turn's timer, while its turn runs under a limited duration: stopped while a level-up
    /// holds the game and while it fights another player's hero; at 0 the server ends its turn
    /// (its decisions answered by their defaults, its battle played by the computer).
    pub turn: Option<Timer>,
    /// The chess clocks of its battle against another player's hero, under a limited duration:
    /// the side to act's runs; a side at 0 is the computer's for the rest of the battle.
    pub battle: Option<Duel>,
    /// Its own level-up's time while it holds the game, if the game limits it: at 0 the server
    /// answers it by its default.
    pub level_up: Option<Timer>,
    /// Whether the game waits for another player's level-up: no command of this player's is
    /// accepted meanwhile, and its timers are stopped.
    pub waits: bool,
}

/// A timer as it stood when told.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Timer {
    /// The milliseconds left.
    pub left_ms: u32,
    /// Whether it runs.
    pub running: bool,
}

/// A battle's chess clocks, by side.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Duel {
    /// The attacker's.
    pub attacker: Timer,
    /// The defender's.
    pub defender: Timer,
}

/// How a map offer's games are timed (E16 M5 T17, v60), as the lobby shows it: the session's
/// clock (`session::game::Clock`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OfferClock {
    /// A turn's duration in milliseconds (an offer's is one of the original's eleven: 1, 2, 4, 6,
    /// 8, 10, 15, 20, 25 or 30 minutes); `None`: unlimited, no turn timer and no chess clock.
    pub turn_ms: Option<u32>,
    /// A level-up's time while it holds the game; `None`: the game waits for it.
    pub level_up_ms: Option<u32>,
    /// Each side's time at the start of a battle between two players' heroes, under a limited
    /// turn duration.
    pub battle_ms: u32,
    /// The time a side gains after each of its battle commands there.
    pub increment_ms: u32,
}
