//! The lobby (E16 M2 T1): offers, game ids, the limits in total and per origin, the list and the
//! games' updates; the map offers and the whole games (E16 M5 T9a).

use std::string::ToString;
use std::vec;
use std::vec::Vec;

use super::full;
use crate::lobby::{Limits, Lobby};
use crate::map;
use crate::wire::game::{MapGameInfo, MapIdentity, MapOffer, SeatedPlayer};
use crate::wire::{GameInfo, Offer, Phase, Refusal, Seat, Seated, ServerMessage, WireU64};

fn offers() -> Vec<Offer> {
    let (ruleset, _) = full();
    vec![
        map::offer("pvp", "full", &ruleset, vec![]),
        map::offer("bot", "full", &ruleset, vec![Seat::Defender]),
    ]
}

/// A map offer of two players (by hand: the lobby reads only its id and players).
fn maps() -> Vec<MapOffer> {
    vec![MapOffer {
        id: "duel".to_string(),
        name: "Duel".to_string(),
        size: 36,
        levels: 2,
        players: vec![0, 1],
        difficulty: 1,
        packs: vec![],
        profile: "base:faithful".to_string(),
        ruleset_hash: WireU64(7),
        map: MapIdentity {
            content_hash: WireU64(9),
            import_version: 1,
        },
    }]
}

fn lobby(games: usize, games_per_origin: usize) -> Lobby<u8> {
    Lobby::new(
        offers(),
        maps(),
        Limits {
            games,
            games_per_origin,
        },
    )
}

/// The lobby's list: its battles and its whole games, the offers checked.
fn listed(lobby: &Lobby<u8>) -> (Vec<GameInfo>, Vec<MapGameInfo>) {
    let ServerMessage::Games {
        seq: 4,
        offers: listed,
        games,
        maps: listed_maps,
        map_games,
    } = lobby.list(4)
    else {
        panic!("not a list")
    };
    assert_eq!((listed, listed_maps), (offers(), maps()));
    (games, map_games)
}

fn games(lobby: &Lobby<u8>) -> Vec<GameInfo> {
    listed(lobby).0
}

fn map_games(lobby: &Lobby<u8>) -> Vec<MapGameInfo> {
    listed(lobby).1
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
                seated: vec![],
                ladder: None,
            },
            GameInfo {
                game: 2,
                offer: "bot".to_string(),
                phase: Phase::Waiting,
                open: vec![Seat::Attacker],
                spectators: false,
                seated: vec![],
                ladder: None,
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
    // E16 M3 T3: the host names the seats that belong to accounts.
    let alice = vec![Seated {
        seat: Seat::Attacker,
        display: "Alice".to_string(),
        bot: false,
    }];
    lobby.seat(1, alice.clone());
    lobby.seat(9, alice.clone());
    assert_eq!(
        (games(&lobby)[0].seated.clone(), games(&lobby).len()),
        (alice, 2)
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
        seated: vec![],
        ladder: None,
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

#[test]
fn ids_handed_out_in_earlier_lives_are_skipped() {
    // E16 M4 T3b: the server's `next-game` says ids up to 41 were handed out; a lower mark
    // changes nothing.
    let mut lobby = lobby(3, 3);
    lobby.skip(41);
    lobby.skip(2);
    assert_eq!(lobby.create(1, "pvp", Seat::Attacker, true), Ok((42, 0)));
    lobby.skip(u32::MAX);
    assert_eq!(
        lobby.create(1, "pvp", Seat::Attacker, true),
        Err(Refusal::TooManyGames),
        "no id is handed out twice"
    );
}

#[test]
fn a_draining_lobby_refuses_every_creation_and_counts_the_games_in_play() {
    let mut lobby = lobby(8, 8);
    assert_eq!(lobby.create(1, "pvp", Seat::Attacker, true), Ok((1, 0)));
    assert_eq!(lobby.create(1, "pvp", Seat::Attacker, true), Ok((2, 0)));
    assert_eq!(lobby.create(2, "bot", Seat::Attacker, true), Ok((3, 1)));
    lobby.update(2, (Phase::Running, vec![], true));
    lobby.update(3, (Phase::Paused, vec![], false));
    assert_eq!(
        lobby.in_play(),
        2,
        "a waiting and a running game; not a paused one"
    );
    lobby.drain();
    assert_eq!(
        lobby.create(3, "pvp", Seat::Attacker, true),
        Err(Refusal::Draining)
    );
    assert_eq!(
        lobby.create(3, "pvp", Seat::Spectator, true),
        Err(Refusal::NoSpectators)
    );
    lobby.update(2, (Phase::Over, vec![], true));
    lobby.remove(1);
    assert_eq!(lobby.in_play(), 0, "over and gone");
    assert_eq!(games(&lobby).len(), 2);
}

#[test]
fn a_matchs_game_counts_against_the_servers_limit_only() {
    let mut lobby = lobby(3, 1);
    assert_eq!(lobby.create(1, "pvp", Seat::Attacker, true), Ok((1, 0)));
    // The origin's limit is no match's concern; the server's is.
    assert_eq!(lobby.create_ranked(0, ("arena.1", "arena"), true), Ok(2));
    assert_eq!(lobby.create_ranked(0, ("arena.2", "arena"), false), Ok(3));
    assert_eq!(
        lobby.create_ranked(0, ("arena.1", "arena"), true),
        Err(Refusal::TooManyGames)
    );
    let ranked = GameInfo {
        game: 2,
        offer: "arena.1".to_string(),
        phase: Phase::Waiting,
        open: vec![],
        spectators: true,
        seated: vec![],
        ladder: Some("arena".to_string()),
    };
    assert_eq!(games(&lobby)[1], ranked);
    // A game over makes room.
    lobby.update(2, (Phase::Over, vec![], true));
    assert_eq!(lobby.create_ranked(0, ("arena.1", "arena"), true), Ok(4));
    lobby.drain();
    assert_eq!(
        lobby.create_ranked(0, ("arena.1", "arena"), true),
        Err(Refusal::Draining)
    );
}

#[test]
fn whole_games_are_created_from_map_offers_and_listed() {
    let mut lobby = lobby(8, 8);
    assert!(map_games(&lobby).is_empty());
    assert_eq!(lobby.create_map(1, "duel", 0), Ok((1, 0)));
    assert_eq!(lobby.create(1, "pvp", Seat::Attacker, true), Ok((2, 0)));
    assert_eq!(lobby.create_map(2, "duel", 1), Ok((3, 0)), "one id space");
    let waiting = |game| MapGameInfo {
        game,
        offer: "duel".to_string(),
        phase: Phase::Waiting,
        open: vec![0, 1],
        seated: vec![],
    };
    assert_eq!(map_games(&lobby), [waiting(1), waiting(3)]);
    assert_eq!(games(&lobby).len(), 1, "the battle beside them");
    assert_eq!(lobby.in_play(), 3);
    // A whole game reports where it is and who holds its seats; a battle's reports leave it be,
    // and its own leave a battle be.
    lobby.update_map(1, (Phase::Running, vec![1]));
    let ann = vec![SeatedPlayer {
        player: 0,
        display: "Ann".to_string(),
    }];
    lobby.seat_map(1, ann.clone());
    lobby.update(1, (Phase::Over, vec![], true));
    lobby.seat(1, vec![]);
    lobby.update_map(2, (Phase::Paused, vec![]));
    lobby.seat_map(2, ann.clone());
    lobby.update_map(9, (Phase::Paused, vec![]));
    lobby.seat_map(9, ann.clone());
    let running = MapGameInfo {
        phase: Phase::Running,
        open: vec![1],
        seated: ann,
        ..waiting(1)
    };
    assert_eq!(map_games(&lobby), [running, waiting(3)]);
    let battle = &games(&lobby)[0];
    assert_eq!((battle.phase, battle.seated.len()), (Phase::Waiting, 0));
    // A paused whole game is not in play; a removed one leaves the list.
    lobby.update_map(3, (Phase::Paused, vec![]));
    assert_eq!(lobby.in_play(), 2);
    lobby.remove(1);
    assert_eq!(map_games(&lobby).len(), 1);
}

#[test]
fn map_creations_are_refused_with_their_reason() {
    let mut lobby = lobby(3, 2);
    for (offer, player, refusal) in [
        ("nothing", 0, Refusal::NoSuchOffer),
        ("pvp", 0, Refusal::NoSuchOffer),
        ("duel", 2, Refusal::NoSuchSeat),
    ] {
        assert_eq!(lobby.create_map(1, offer, player), Err(refusal), "{offer}");
    }
    assert_eq!(
        lobby.create(1, "duel", Seat::Attacker, true),
        Err(Refusal::NoSuchOffer),
        "a map offer is no battle's"
    );
    // The limits count battles and whole games alike: the origin's, then the total's.
    assert_eq!(lobby.create(1, "pvp", Seat::Attacker, true), Ok((1, 0)));
    assert_eq!(lobby.create_map(1, "duel", 0), Ok((2, 0)));
    assert_eq!(lobby.create_map(1, "duel", 0), Err(Refusal::TooManyGames));
    assert_eq!(lobby.create_map(2, "duel", 1), Ok((3, 0)));
    assert_eq!(lobby.create_map(3, "duel", 0), Err(Refusal::TooManyGames));
    lobby.drain();
    assert_eq!(lobby.create_map(3, "duel", 0), Err(Refusal::Draining));
}
