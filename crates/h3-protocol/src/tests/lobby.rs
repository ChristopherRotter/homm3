//! The lobby (E16 M2 T1): offers, game ids, the limits in total and per origin, the list and the
//! games' updates.

use std::string::ToString;
use std::vec;
use std::vec::Vec;

use super::full;
use crate::lobby::{Limits, Lobby};
use crate::map;
use crate::wire::{GameInfo, Offer, Phase, Refusal, Seat, ServerMessage};

fn offers() -> Vec<Offer> {
    let (ruleset, _) = full();
    vec![
        map::offer("pvp", "full", &ruleset, vec![]),
        map::offer("bot", "full", &ruleset, vec![Seat::Defender]),
    ]
}

fn lobby(games: usize, games_per_origin: usize) -> Lobby<u8> {
    Lobby::new(
        offers(),
        Limits {
            games,
            games_per_origin,
        },
    )
}

fn games(lobby: &Lobby<u8>) -> Vec<GameInfo> {
    let ServerMessage::Games {
        seq: 4,
        offers: listed,
        games,
    } = lobby.list(4)
    else {
        panic!("not a list")
    };
    assert_eq!(listed, offers());
    games
}

#[test]
fn games_are_created_from_offers_and_listed() {
    let mut lobby = lobby(8, 8);
    assert!(games(&lobby).is_empty());
    assert_eq!(lobby.create(1, "pvp", Seat::Attacker, true), Ok((1, 0)));
    assert_eq!(lobby.create(2, "bot", Seat::Attacker, false), Ok((2, 1)));
    assert_eq!(
        games(&lobby),
        [
            GameInfo {
                game: 1,
                offer: "pvp".to_string(),
                phase: Phase::Waiting,
                open: vec![Seat::Attacker, Seat::Defender],
                spectators: true,
            },
            GameInfo {
                game: 2,
                offer: "bot".to_string(),
                phase: Phase::Waiting,
                open: vec![Seat::Attacker],
                spectators: false,
            },
        ]
    );
    assert!(lobby.contains(1) && lobby.contains(2) && !lobby.contains(3));
    // The games report where they are.
    lobby.update(1, (Phase::Running, vec![], false));
    lobby.update(9, (Phase::Over, vec![], true));
    assert_eq!(
        (games(&lobby)[0].phase, games(&lobby)[0].spectators),
        (Phase::Running, false),
        "a game reports whether it takes spectators too"
    );
    assert_eq!(
        games(&lobby).len(),
        2,
        "an unknown game is not listed by an update"
    );
    // A removed game leaves the list; ids are never reused.
    lobby.remove(1);
    assert!(!lobby.contains(1));
    assert_eq!(lobby.create(1, "pvp", Seat::Defender, true), Ok((3, 0)));
}

#[test]
fn creations_are_refused_with_their_reason() {
    let mut lobby = lobby(3, 2);
    assert_eq!(
        lobby.create(1, "pvp", Seat::Spectator, true),
        Err(Refusal::NoSpectators)
    );
    assert_eq!(
        lobby.create(1, "nothing", Seat::Attacker, true),
        Err(Refusal::NoSuchOffer)
    );
    assert_eq!(
        lobby.create(1, "bot", Seat::Defender, true),
        Err(Refusal::SeatTaken),
        "the bot's seat"
    );
    assert_eq!(lobby.create(1, "pvp", Seat::Attacker, true), Ok((1, 0)));
    assert_eq!(lobby.create(1, "pvp", Seat::Attacker, true), Ok((2, 0)));
    // The origin's limit.
    assert_eq!(
        lobby.create(1, "pvp", Seat::Attacker, true),
        Err(Refusal::TooManyGames)
    );
    // The total's.
    assert_eq!(lobby.create(2, "bot", Seat::Attacker, true), Ok((3, 1)));
    assert_eq!(
        lobby.create(3, "bot", Seat::Attacker, true),
        Err(Refusal::TooManyGames)
    );
    // A game that is over counts no more; one that is removed neither.
    lobby.update(1, (Phase::Over, vec![], true));
    assert_eq!(lobby.create(1, "pvp", Seat::Attacker, true), Ok((4, 0)));
    lobby.remove(2);
    assert_eq!(lobby.create(3, "bot", Seat::Attacker, true), Ok((5, 1)));
    assert!(
        lobby.contains(1),
        "an over game stays listed until it is removed"
    );
}

#[test]
fn restored_games_are_listed_again_and_counted() {
    let mut lobby = lobby(3, 1);
    let restored = |game| GameInfo {
        game,
        offer: "pvp".to_string(),
        phase: Phase::Running,
        open: vec![],
        spectators: true,
    };
    // Beyond the origin's limit, which the games were within when they were created.
    lobby.reopen(1, restored(7));
    lobby.reopen(1, restored(3));
    lobby.reopen(2, restored(5));
    assert_eq!(games(&lobby), [restored(3), restored(5), restored(7)]);
    assert_eq!(
        lobby.create(3, "pvp", Seat::Attacker, true),
        Err(Refusal::TooManyGames),
        "they count in total"
    );
    lobby.remove(7);
    assert_eq!(
        lobby.create(1, "pvp", Seat::Attacker, true),
        Err(Refusal::TooManyGames),
        "and for their origin"
    );
    // Ids continue after the highest restored one.
    assert_eq!(lobby.create(3, "pvp", Seat::Attacker, true), Ok((8, 0)));
}
