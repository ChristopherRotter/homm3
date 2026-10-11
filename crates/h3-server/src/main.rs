//! `h3-server <config.toml>`: hosts the configured battles for untrusted clients over WebSocket
//! (E16, ADR-0018); the configuration's format is in `h3_server::config`.
//!
//! Stopping (E16 M2 T5a): the first SIGINT (Ctrl-C) or SIGTERM drains the server: no new game,
//! the waiting ones end, the running ones play on; once none runs, it stops. A second signal
//! stops it at once. Stopped, every connection hears `Goodbye { Shutdown }` and the journals
//! stay: the running and the paused games restore at the next start. Provisional: the signals.

use std::path::PathBuf;
use std::process::ExitCode;

use h3_server::{Config, start};

fn main() -> ExitCode {
    let mut args = std::env::args_os().skip(1);
    let (Some(path), None) = (args.next(), args.next()) else {
        eprintln!(
            "usage: h3-server <config.toml>\n{} (protocol {})",
            h3_server::server_name(),
            h3_protocol::PROTOCOL_VERSION
        );
        return ExitCode::from(2);
    };
    let config = match Config::load(&PathBuf::from(path)) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("h3-server: {e}");
            return ExitCode::FAILURE;
        }
    };
    let runtime = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(runtime) => runtime,
        Err(e) => {
            eprintln!("h3-server: runtime: {e}");
            return ExitCode::FAILURE;
        }
    };
    runtime.block_on(async {
        match start(&config).await {
            Ok(running) => {
                eprintln!(
                    "h3-server: {} (protocol {}) listening on ws://{}",
                    h3_server::server_name(),
                    h3_protocol::PROTOCOL_VERSION,
                    running.address
                );
                signalled().await;
                running.drain();
                eprintln!(
                    "h3-server: draining: no new game; {} in play; a second signal stops now",
                    running.in_play()
                );
                tokio::select! {
                    () = running.drained() => eprintln!("h3-server: drained"),
                    () = signalled() => eprintln!("h3-server: stopping now"),
                }
                running.shutdown().await;
                eprintln!("h3-server: stopped");
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("h3-server: {e}");
                ExitCode::FAILURE
            }
        }
    })
}

/// Resolves at a SIGINT (Ctrl-C) or, on Unix, a SIGTERM; never if neither can be listened for.
async fn signalled() {
    #[cfg(unix)]
    let terminate = async {
        use tokio::signal::unix::{SignalKind, signal};
        match signal(SignalKind::terminate()) {
            Ok(mut term) => {
                term.recv().await;
            }
            Err(_) => std::future::pending().await,
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! {
        interrupted = tokio::signal::ctrl_c() => {
            if interrupted.is_err() {
                std::future::pending::<()>().await;
            }
        }
        () = terminate => {}
    }
}
