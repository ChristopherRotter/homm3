//! A game's bounds over loopback (E16 M3 T4a): a game that reaches the server's limit of
//! commands without an end is void, also one a restart restores past a lowered limit; a waiting
//! game none of whose seats is held leaves, its spectators told.

use super::*;

/// The journals in the data directory `data`'s void directory.
fn voided(data: &Path) -> usize {
    std::fs::read_dir(data.join("void"))
        .map(|listing| listing.count())
        .unwrap_or(0)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_game_past_its_command_limit_is_void() {
    let config = config(Limits {
        commands_per_game: 6,
        ..fast()
    });
    let running = start(&config).await.unwrap();
    let (client, game, _) = created(&running, "bot", Seat::Attacker).await;
    let mut watcher = greeted(&running).await;
    watcher.join(game, Seat::Spectator, None).await.unwrap();
    let (_, seen) = tokio::time::timeout(GAME_TIME, play(client, 3))
        .await
        .unwrap();
    assert_eq!(
        seen.last(),
        Some(&ServerMessage::Goodbye {
            reason: GoodbyeReason::CommandLimit
        }),
        "{seen:?}"
    );
    assert!(
        !seen
            .iter()
            .any(|m| matches!(m, ServerMessage::GameEnded { .. })),
        "no end"
    );
    loop {
        match watcher.recv().await {
            Some(ServerMessage::Goodbye { reason }) => {
                assert_eq!(reason, GoodbyeReason::CommandLimit);
                break;
            }
            Some(_) => {}
            None => panic!("no goodbye"),
        }
    }
    listed_as(&running, game, None).await;
    assert!(!journal_file(&config.server.data, game).exists());
    assert_eq!(voided(&config.server.data), 1, "the journal is void");
    assert_eq!(archive_page(&running, 0).await.0, 0, "no replay");
    running.shutdown().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_restored_game_past_a_lowered_limit_is_void() {
    let config = config(fast());
    let running = start(&config).await.unwrap();
    let (mut client, game, _) = created(&running, "bot", Seat::Attacker).await;
    play_turns(&mut client, 5, 3).await;
    // The server stops; it starts again with a limit the game is past.
    let restarted = Config {
        server: Server {
            data: crashed(&config.server.data),
            ..config.server.clone()
        },
        limits: Limits {
            commands_per_game: 2,
            ..fast()
        },
        ..config.clone()
    };
    let again = start(&restarted).await.unwrap();
    listed_as(&again, game, None).await;
    assert_eq!(voided(&restarted.server.data), 1);
    again.shutdown().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_waiting_game_its_players_left_leaves_with_its_spectators() {
    let config = config(fast());
    let running = start(&config).await.unwrap();
    let (creator, game, _) = created(&running, "pvp", Seat::Attacker).await;
    let mut watcher = greeted(&running).await;
    watcher.join(game, Seat::Spectator, None).await.unwrap();
    assert!(matches!(
        watcher.recv().await,
        Some(ServerMessage::Joined {
            seat: Seat::Spectator,
            ..
        })
    ));
    assert!(matches!(
        watcher.recv().await,
        Some(ServerMessage::Snapshot { .. })
    ));
    // The spectator does not keep the game listed once its creator left.
    creator.close().await;
    assert_eq!(goodbye(&mut watcher).await, GoodbyeReason::Abandoned);
    listed_as(&running, game, None).await;
    assert!(!journal_file(&config.server.data, game).exists());
    assert_eq!(
        voided(&config.server.data),
        0,
        "nothing void: it never started"
    );
    running.shutdown().await;
}
