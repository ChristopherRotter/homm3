//! The registry (E16 M2 T1): the lobby ([`h3_protocol::lobby`]) and each game's inbox, shared by
//! the connection tasks and the games' threads behind one lock. A connection task looks a game up
//! and binds to it, or creates one; a game's thread reports where it is, and leaves when it is not
//! running and no connection is bound to it. Binding and leaving happen under the same lock, so a
//! connection never binds to a game that has left.
//!
//! At start, before the server listens, the games whose journals the data directory holds are
//! restored and listed again (E16 M2 T2).
//!
//! Provisional (E16 M2 T1, T2): the lock, the origin (the client's address), a creation that fails
//! after the lobby's checks refused as `TooManyGames` (only OS randomness and the journal's file
//! can fail there).

use std::collections::BTreeMap;
use std::net::IpAddr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::{fs, io};

use h3_protocol::codec;
use h3_protocol::lobby::{self, Lobby};
use h3_protocol::wire::{GameInfo, Phase, Refusal, Seat};
use tokio::sync::{mpsc, oneshot};

use crate::game::{Offer, Prepared, ToGame};
use crate::{Error, config, journal};

/// A game's inbox: messages waiting for the game's thread.
const GAME_INBOX: usize = 1024;

/// The registry, as the tasks and threads share it.
pub(crate) type Shared = Arc<Mutex<Registry>>;

/// Locks the registry (a thread that panicked under the lock leaves it usable).
pub(crate) fn lock(shared: &Shared) -> MutexGuard<'_, Registry> {
    shared.lock().unwrap_or_else(PoisonError::into_inner)
}

/// A hosted game: its inbox and how many connections are bound to it.
#[derive(Debug)]
struct Hosted {
    to: mpsc::Sender<ToGame>,
    bound: usize,
}

/// The lobby and the hosted games.
#[derive(Debug)]
pub(crate) struct Registry {
    lobby: Lobby<IpAddr>,
    offers: Vec<Arc<Offer>>,
    games: BTreeMap<u32, Hosted>,
    server: config::Server,
}

impl Registry {
    /// A registry of `offers` and no games.
    pub(crate) fn new(
        offers: Vec<Arc<Offer>>,
        limits: lobby::Limits,
        server: config::Server,
    ) -> Self {
        let infos = offers.iter().map(|o| o.info.clone()).collect();
        Self {
            lobby: Lobby::new(infos, limits),
            offers,
            games: BTreeMap::new(),
            server,
        }
    }

    /// The answer to `ListGames`, as a frame's text.
    pub(crate) fn list(&self, seq: u32) -> String {
        codec::encode_text(&self.lobby.list(seq))
    }

    /// Binds a connection to game `game`: the game's inbox, if it is hosted.
    pub(crate) fn bind(&mut self, game: u32) -> Option<mpsc::Sender<ToGame>> {
        let hosted = self.games.get_mut(&game)?;
        hosted.bound += 1;
        Some(hosted.to.clone())
    }

    /// A connection bound to game `game` leaves it.
    pub(crate) fn unbind(&mut self, game: u32) {
        if let Some(hosted) = self.games.get_mut(&game) {
            hosted.bound = hosted.bound.saturating_sub(1);
        }
    }

    /// Where game `game` is now.
    pub(crate) fn report(&mut self, game: u32, phase: Phase, open: Vec<Seat>) {
        self.lobby.update(game, phase, open);
    }

    /// Whether game `game`, in `phase`, leaves now: when it is not running and no connection is
    /// bound to it. Then it is gone from the lobby, and its thread ends.
    pub(crate) fn close_if_idle(&mut self, game: u32, phase: Phase) -> bool {
        let idle = self.games.get(&game).is_none_or(|hosted| hosted.bound == 0);
        if idle && phase != Phase::Running {
            self.leave(game);
        }
        idle && phase != Phase::Running
    }

    /// Lists a game a restart restored, as `origin` created it, and hosts it with the inbox `to`
    /// (E16 M2 T2).
    pub(crate) fn reopen(&mut self, origin: IpAddr, info: GameInfo, to: mpsc::Sender<ToGame>) {
        let game = info.game;
        self.lobby.reopen(origin, info);
        self.games.insert(game, Hosted { to, bound: 0 });
    }

    /// Hosts a game whose inbox is `to`, outside the lobby (tests of the connection task).
    #[cfg(test)]
    pub(crate) fn host(&mut self, game: u32, to: mpsc::Sender<ToGame>) {
        self.games.insert(game, Hosted { to, bound: 0 });
    }

    /// Game `game` is gone.
    pub(crate) fn leave(&mut self, game: u32) {
        self.games.remove(&game);
        self.lobby.remove(game);
    }
}

/// Creates a game of `offer` for a client at `origin`, whose creator takes `seat`, and binds the
/// creator to it: the game's id and inbox, or why not.
pub(crate) fn create(
    shared: &Shared,
    origin: IpAddr,
    offer: &str,
    seat: Seat,
) -> Result<(u32, mpsc::Sender<ToGame>), Refusal> {
    let mut registry = lock(shared);
    let (game, index) = registry.lobby.create(origin, offer, seat)?;
    let offer = Arc::clone(&registry.offers[index]);
    let (to, inbox) = mpsc::channel(GAME_INBOX);
    let started = Prepared::new(offer, game, origin, &registry.server)
        .and_then(|prepared| prepared.spawn(inbox, Arc::clone(shared)));
    if let Err(e) = started {
        eprintln!("h3-server: game {game} not created: {e}");
        registry.lobby.remove(game);
        return Err(Refusal::TooManyGames);
    }
    registry.games.insert(
        game,
        Hosted {
            to: to.clone(),
            bound: 1,
        },
    );
    Ok((game, to))
}

/// Restores the games whose journals the data directory holds (E16 M2 T2; call before the server
/// listens): each re-runs on its own thread, which lists it again if it goes on; this returns once
/// every re-run is done. A journal that does not read, that names an offer the server no longer
/// has, or whose engine or content identity is not the offer's, is void. A game thread that does
/// not start stops the server's start.
pub(crate) async fn restore(shared: &Shared) -> Result<(), Error> {
    let data = lock(shared).server.data.clone();
    let dir = journal::dir(&data);
    let listing = match fs::read_dir(&dir) {
        Ok(listing) => listing,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(Error(format!("{}: {e}", dir.display()))),
    };
    let mut paths: Vec<PathBuf> = listing
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.extension().is_some_and(|e| e == "journal"))
        .collect();
    paths.sort();
    let mut restoring = Vec::new();
    for path in paths {
        let name = path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        let (to, inbox) = mpsc::channel(GAME_INBOX);
        let (done, finished) = oneshot::channel();
        let prepared = journal::read(&path).and_then(|read| {
            let registry = lock(shared);
            let offer = registry
                .offers
                .iter()
                .find(|o| o.id == read.start.offer)
                .cloned()
                .ok_or_else(|| format!("the server offers no `{}` now", read.start.offer))?;
            Prepared::restored(offer, &path, read, &registry.server, (to, done))
        });
        match prepared {
            Ok(prepared) => {
                prepared.spawn(inbox, Arc::clone(shared))?;
                restoring.push(finished);
            }
            Err(why) => journal::void(&data, &path, &name, &why),
        }
    }
    for finished in restoring {
        let _ = finished.await;
    }
    Ok(())
}
