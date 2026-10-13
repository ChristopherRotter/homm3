//! The server's card (E16 M6 T2; ADR-0021 point 2, `E16-design.md` §12): its description of
//! itself, signed by its key ([`crate::key`]), which a client gets with `Describe`. `h3-protocol`
//! builds the card's text and bounds it ([`h3_protocol::card`]); here the server fills it from its
//! configuration and its state, signs it, and reads the cards it gets (the test client; the peers'
//! cards from E16 M6 T3).
//!
//! The server signs its card when asked and its last one is older than [`RESIGN`] (the counts
//! move), so a stream of `Describe`s costs one signature a minute. A card is issued at the wall
//! clock and expires `[discovery] card_ms` later; with `[discovery] counts` off, it names no counts.
//!
//! Provisional (E16 M6 T2): what `players_online` counts (the open connections, the asker's
//! included), the re-signing age.

use std::sync::{Mutex, PoisonError};
use std::time::Duration;

use h3_protocol::card::{self, CardError, MAX_CARD_BYTES};
use h3_protocol::codec;
use h3_protocol::connection::Signer;
use h3_protocol::wire::{CardContent, CardLadder, ServerCard, SignedCard, WireU64};
use tokio::time::Instant;

use crate::Error;
use crate::accounts::{Authenticator, Ed25519};

/// How long a signed card is handed out again before the server signs a new one.
pub(crate) const RESIGN: Duration = Duration::from_secs(60);

/// What the card counts as it is signed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Live {
    /// The open connections.
    pub players_online: u32,
    /// The games that wait for players or run.
    pub games_open: u32,
    /// The ladders, each with its current season's leaderboard's size.
    pub ladders: Vec<CardLadder>,
}

/// The server's card: what stays from its configuration, and the last one signed.
#[derive(Debug)]
pub(crate) struct Cards {
    /// The card without its times, its counts and its ladders.
    template: ServerCard,
    /// Whether the card names its counts (`[discovery] counts`).
    counts: bool,
    /// How long a card holds, in milliseconds (`[discovery] card_ms`).
    card_ms: u64,
    /// The last card signed, and when.
    last: Mutex<Option<(Instant, SignedCard)>>,
}

impl Cards {
    /// The cards of `template` (its times, counts and ladders are filled in at each signing), each
    /// holding `card_ms` and naming its counts if `counts`. Why not, if a card of it, with the
    /// ladders `ladders` and the largest counts, would break the card's bounds.
    pub(crate) fn new(
        template: ServerCard,
        (card_ms, counts): (u64, bool),
        ladders: Vec<CardLadder>,
    ) -> Result<Self, Error> {
        let cards = Self {
            template,
            counts,
            card_ms,
            last: Mutex::new(None),
        };
        let mut largest = cards.card(u64::MAX - card_ms, || Live {
            players_online: u32::MAX,
            games_open: u32::MAX,
            ladders,
        });
        largest.ladders.iter_mut().for_each(|l| l.season = u32::MAX);
        let text = codec::encode_text(&largest).len();
        if text > MAX_CARD_BYTES || !card::fits(&largest) {
            return Err(Error(format!(
                "the server's card breaks its bounds (h3_protocol::card): {text} bytes of at most \
                 {MAX_CARD_BYTES}; a name, at most {} rulesets and {} ladders, names, ids and \
                 versions of at most {} bytes",
                card::MAX_LISTED,
                card::MAX_LISTED,
                card::MAX_TEXT
            )));
        }
        Ok(cards)
    }

    /// The card issued at `wall` (milliseconds since the Unix epoch), with what `live` counts.
    fn card(&self, wall: u64, live: impl FnOnce() -> Live) -> ServerCard {
        let Live {
            players_online,
            games_open,
            mut ladders,
        } = live();
        if !self.counts {
            ladders.iter_mut().for_each(|l| l.players = None);
        }
        let counted = |n| self.counts.then_some(n);
        ServerCard {
            ladders,
            players_online: counted(players_online),
            games_open: counted(games_open),
            issued: WireU64(wall),
            expires: WireU64(wall.saturating_add(self.card_ms)),
            ..self.template.clone()
        }
    }

    /// The signed card at `now` (`wall` on the wall clock): the last one while it is younger than
    /// [`RESIGN`], else a new one signed by `key` with what `live` counts.
    pub(crate) fn signed(
        &self,
        (now, wall): (Instant, u64),
        key: &dyn Signer,
        live: impl FnOnce() -> Live,
    ) -> SignedCard {
        let last = self.last.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some((at, signed)) = &*last
            && now.saturating_duration_since(*at) < RESIGN
        {
            return signed.clone();
        }
        // Counted outside the lock: the counts take other locks.
        drop(last);
        let signed = card::sign(&self.card(wall, live), key);
        let mut last = self.last.lock().unwrap_or_else(PoisonError::into_inner);
        *last = Some((now, signed.clone()));
        signed
    }
}

/// The card `signed` holds at `now` (milliseconds since the Unix epoch): its own key's Ed25519
/// signature verifies over its text, it has not expired, it was issued at most
/// [`card::SKEW_MS`] ahead and keeps its bounds ([`card::read`]). Whose key it is, is the caller's
/// check.
pub(crate) fn verify(signed: &SignedCard, now: u64) -> Result<ServerCard, CardError> {
    card::read(signed, now, |key, message, signature| {
        Ed25519.verify(key, message, signature)
    })
}

/// The wall clock: milliseconds since the Unix epoch (0 before it). The issue and expiry times of
/// cards and of lists of servers are the server's one use of it; its games, timers and limits run
/// on `tokio`'s clock.
// The workspace bans the system clock for game logic (ADR-0003 §10); a card's times are not game
// logic: they say when the card was signed, for clients and other servers to judge its age.
#[allow(clippy::disallowed_types)]
pub(crate) fn wall_ms() -> u64 {
    let since = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH);
    since.map_or(0, |d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX))
}

/// Each ruleset of `content` once, in its first place.
pub(crate) fn distinct(content: impl IntoIterator<Item = CardContent>) -> Vec<CardContent> {
    let mut out: Vec<CardContent> = Vec::new();
    for c in content {
        if !out.contains(&c) {
            out.push(c);
        }
    }
    out
}

#[cfg(test)]
pub(crate) mod tests {
    use h3_protocol::account;
    use h3_protocol::card::SKEW_MS;
    use h3_protocol::wire::{CardPeer, PackRef};

    use super::*;
    use crate::key::ServerKey;

    const WALL: u64 = 1_800_000_000_000;
    const DAY: u64 = 86_400_000;

    /// A card of `key` at a loopback address, on the synthetic packs.
    pub(crate) fn template(key: &ServerKey) -> ServerCard {
        ServerCard {
            key: account::to_hex(&key.key()),
            name: String::from("Test"),
            description: String::new(),
            addresses: vec![String::from("ws://127.0.0.1:7878")],
            protocol: h3_protocol::PROTOCOL_VERSION,
            engine: String::from("0.1.0"),
            content: vec![CardContent {
                packs: vec![PackRef {
                    id: String::from("synthetic"),
                    version: String::from("1"),
                }],
                profile: String::from("synthetic:faithful"),
                ruleset_hash: WireU64(1),
                battle_ruleset_hash: WireU64(2),
            }],
            ladders: Vec::new(),
            players_online: None,
            games_open: None,
            anonymous: true,
            operator: String::new(),
            issued: WireU64(0),
            expires: WireU64(1),
            peers: Vec::new(),
        }
    }

    fn arena(players: u32) -> CardLadder {
        CardLadder {
            id: String::from("arena"),
            name: String::from("Arena"),
            season: 1,
            players: Some(players),
        }
    }

    fn live(players_online: u32) -> Live {
        Live {
            players_online,
            games_open: 2,
            ladders: vec![arena(5)],
        }
    }

    #[test]
    fn a_card_verifies_by_its_key_within_its_times() {
        let key = ServerKey::from_seed([4; 32]);
        let cards = Cards::new(template(&key), (DAY, true), vec![arena(0)]).unwrap();
        let signed = cards.signed((Instant::now(), WALL), &key, || live(3));
        let card = verify(&signed, WALL).unwrap();
        assert_eq!(card.key, account::to_hex(&key.key()));
        assert_eq!((card.issued.0, card.expires.0), (WALL, WALL + DAY));
        assert_eq!((card.players_online, card.games_open), (Some(3), Some(2)));
        assert_eq!(card.ladders, vec![arena(5)]);
        // Expired, issued too far ahead, tampered with.
        assert_eq!(verify(&signed, WALL + DAY), Err(CardError::Expired));
        assert_eq!(verify(&signed, WALL - SKEW_MS - 1), Err(CardError::Ahead));
        let mut tampered = signed.clone();
        tampered.card = tampered.card.replace("\"Test\"", "\"Tost\"");
        assert_eq!(verify(&tampered, WALL), Err(CardError::Signature));
        // A card naming this key, signed by another: forged.
        let other = ServerKey::from_seed([5; 32]);
        let forged = cards.signed((Instant::now() + RESIGN, WALL), &other, || live(3));
        assert_eq!(verify(&forged, WALL), Err(CardError::Signature));
    }

    #[test]
    fn without_counts_a_card_names_none() {
        let key = ServerKey::from_seed([4; 32]);
        let cards = Cards::new(template(&key), (DAY, false), vec![arena(0)]).unwrap();
        let signed = cards.signed((Instant::now(), WALL), &key, || live(3));
        let card = verify(&signed, WALL).unwrap();
        assert_eq!((card.players_online, card.games_open), (None, None));
        let mut quiet = arena(0);
        quiet.players = None;
        assert_eq!(card.ladders, vec![quiet]);
    }

    #[test]
    fn a_card_is_signed_again_once_a_minute_old() {
        let key = ServerKey::from_seed([4; 32]);
        let cards = Cards::new(template(&key), (DAY, true), Vec::new()).unwrap();
        let start = Instant::now();
        let first = cards.signed((start, WALL), &key, || live(1));
        let within = (start + RESIGN - Duration::from_millis(1), WALL + 1);
        let again = cards.signed(within, &key, || panic!("counted again within the minute"));
        assert_eq!(again, first);
        let later = cards.signed((start + RESIGN, WALL + 60_000), &key, || live(2));
        let card = verify(&later, WALL + 60_000).unwrap();
        assert_eq!(
            (card.issued.0, card.players_online),
            (WALL + 60_000, Some(2))
        );
    }

    #[test]
    fn a_card_beyond_its_bounds_stops_the_start() {
        let key = ServerKey::from_seed([4; 32]);
        let mut many = template(&key);
        many.peers = vec![
            CardPeer {
                key: "11".repeat(32),
                address: format!("wss://{}", "p".repeat(200)),
            };
            card::MAX_PEERS
        ];
        let error = Cards::new(many, (DAY, true), Vec::new()).unwrap_err();
        assert!(error.0.contains("bytes of at most 8192"), "{error}");
        let mut unnamed = template(&key);
        unnamed.name = String::new();
        assert!(Cards::new(unnamed, (DAY, true), Vec::new()).is_err());
        let ladders = vec![arena(0); card::MAX_LISTED + 1];
        assert!(Cards::new(template(&key), (DAY, true), ladders).is_err());
    }

    #[test]
    fn the_wall_clock_is_past_2020_and_rulesets_are_listed_once() {
        assert!(wall_ms() > 1_577_836_800_000);
        let t = template(&ServerKey::from_seed([4; 32]));
        let mut other = t.content[0].clone();
        other.profile = String::from("synthetic:fixed");
        let content = [t.content[0].clone(), other.clone(), t.content[0].clone()];
        assert_eq!(distinct(content), vec![t.content[0].clone(), other]);
    }
}
