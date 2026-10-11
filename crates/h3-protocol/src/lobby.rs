//! The lobby (E16 M2 T1): the server's offers and its hosted games, sans I/O. The host keeps one
//! per server: it answers `ListGames`, creates a game for `CreateGame` within the limits, and
//! hears from each game's session where the game is ([`crate::session::Session::phase`] and
//! `open_seats`). The games themselves run in their own sessions.
//!
//! Provisional (E16 M2 T1): game ids counted from 1 and never reused; the limits count the games
//! that are not over, in total and per origin (the host's notion of where a request came from: a
//! client's address); a game stays listed until the host removes it; a restored game is listed
//! again without the limits' check (M2 T2).

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use crate::wire::{GameInfo, Offer, Phase, Refusal, Seat, ServerMessage};

/// How many games may be open (not over) at once.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limits {
    /// In total.
    pub games: usize,
    /// Per origin.
    pub games_per_origin: usize,
}

/// A listed game and where its creation came from.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Entry<O> {
    origin: O,
    info: GameInfo,
}

/// The server's offers and games; `O` names where a request came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lobby<O> {
    offers: Vec<Offer>,
    limits: Limits,
    games: BTreeMap<u32, Entry<O>>,
    /// The last id handed out.
    last: u32,
}

impl<O: Ord> Lobby<O> {
    /// A lobby for `offers`, with no games.
    pub const fn new(offers: Vec<Offer>, limits: Limits) -> Self {
        Self {
            offers,
            limits,
            games: BTreeMap::new(),
            last: 0,
        }
    }

    /// The answer to `ListGames`: the offers and the games, by id.
    pub fn list(&self, seq: u32) -> ServerMessage {
        ServerMessage::Games {
            seq,
            offers: self.offers.clone(),
            games: self.games.values().map(|e| e.info.clone()).collect(),
        }
    }

    /// Creates a game of the offer `offer` for `origin`, whose creator takes `seat`: the game's id
    /// and the offer's index, or why not. A spectator creates nothing; an offer the server does
    /// not have, a seat its bot plays, or a full server or origin, is refused.
    pub fn create(&mut self, origin: O, offer: &str, seat: Seat) -> Result<(u32, usize), Refusal> {
        if seat == Seat::Spectator {
            return Err(Refusal::NoSpectators);
        }
        let (index, found) = self
            .offers
            .iter()
            .enumerate()
            .find(|(_, o)| o.id == offer)
            .ok_or(Refusal::NoSuchOffer)?;
        if found.bots.contains(&seat) {
            return Err(Refusal::SeatTaken);
        }
        let open = self.games.values().filter(|e| e.info.phase != Phase::Over);
        let (total, here) = open.fold((0, 0), |(total, here), e| {
            (total + 1, here + usize::from(e.origin == origin))
        });
        let game = self
            .last
            .checked_add(1)
            .filter(|_| total < self.limits.games && here < self.limits.games_per_origin)
            .ok_or(Refusal::TooManyGames)?;
        self.last = game;
        let info = GameInfo {
            game,
            offer: found.id.clone(),
            phase: Phase::Waiting,
            open: [Seat::Attacker, Seat::Defender]
                .into_iter()
                .filter(|s| !found.bots.contains(s))
                .collect(),
        };
        self.games.insert(game, Entry { origin, info });
        Ok((game, index))
    }

    /// Where game `game` is now, as its session reports it.
    pub fn update(&mut self, game: u32, phase: Phase, open: Vec<Seat>) {
        if let Some(entry) = self.games.get_mut(&game) {
            entry.info.phase = phase;
            entry.info.open = open;
        }
    }

    /// Lists again a game a restart restored from its journal (E16 M2 T2), as `origin` created it;
    /// the ids handed out after it are higher. It is not checked against the limits (it was within
    /// them when it was created) and counts for every creation after.
    pub fn reopen(&mut self, origin: O, info: GameInfo) {
        self.last = self.last.max(info.game);
        self.games.insert(info.game, Entry { origin, info });
    }

    /// Game `game` is gone.
    pub fn remove(&mut self, game: u32) {
        self.games.remove(&game);
    }

    /// Whether game `game` is listed.
    pub fn contains(&self, game: u32) -> bool {
        self.games.contains_key(&game)
    }
}
