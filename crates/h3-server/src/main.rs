//! `h3-server <config.toml>`: hosts the configured battles for untrusted clients over WebSocket
//! (E16, ADR-0018); the configuration's format is in `h3_server::config`.
//!
//! `h3-server rebind <config.toml> <login> <public-key>`: the operator binds an account to a new
//! key while the server is stopped (E16 M3 T2b, `h3_server::rebind`). Provisional: the command.
//!
//! `h3-server card <config.toml>`: prints the running server's signed card on standard output, for
//! a web page (E16 M6 T2, `h3_server::card`): the card it gives a client's `Describe`, which holds
//! for `[discovery] card_ms`. Provisional: the command.
//!
//! `h3-server list-key <file>`: prints the public key of the list key in `<file>`, writing a new
//! one there first if there is none (E16 M6 T5, `h3_server::list_key`). `h3-server sign-list --key
//! <file> <list.toml>`: prints the signed list of the TOML list of servers, for a server's
//! `<data>/lists/` (`h3_server::sign_list`). Provisional: the commands.
//!
//! Stopping (E16 M2 T5a): the first SIGINT (Ctrl-C) or SIGTERM drains the server: no new game,
//! the waiting ones end, the running ones play on; once none runs, it stops. A second signal
//! stops it at once. Stopped, every connection hears `Goodbye { Shutdown }` and the journals
//! stay: the running and the paused games restore at the next start. Provisional: the signals.

use std::ffi::OsString;
use std::path::PathBuf;
use std::process::ExitCode;

use h3_server::{Config, start};

fn main() -> ExitCode {
    let args: Vec<OsString> = std::env::args_os().skip(1).collect();
    match &args[..] {
        [path] => serve(path),
        [command, path, login, key] if command == "rebind" => rebind(path, login, key),
        [command, path] if command == "card" => card(path),
        [command, path] if command == "list-key" => {
            print_or_fail("list-key", h3_server::list_key(&PathBuf::from(path)))
        }
        [command, flag, key, list] if command == "sign-list" && flag == "--key" => print_or_fail(
            "sign-list",
            h3_server::sign_list(&PathBuf::from(key), &PathBuf::from(list)),
        ),
        _ => {
            eprintln!(
                "usage: h3-server <config.toml>\n       h3-server rebind <config.toml> <login> <public-key>\n       h3-server card <config.toml>\n       h3-server list-key <file>\n       h3-server sign-list --key <file> <list.toml>\n{} (protocol {})",
                h3_server::server_name(),
                h3_protocol::PROTOCOL_VERSION
            );
            ExitCode::from(2)
        }
    }
}

/// The configuration at `path`, or why not (on standard error).
fn load(path: &OsString) -> Option<Config> {
    Config::load(&PathBuf::from(path))
        .map_err(|e| eprintln!("h3-server: {e}"))
        .ok()
}

/// The operator's rebind (E16 M3 T2b).
fn rebind(path: &OsString, login: &OsString, key: &OsString) -> ExitCode {
    let Some(config) = load(path) else {
        return ExitCode::FAILURE;
    };
    let (login, key) = (login.to_string_lossy(), key.to_string_lossy());
    match h3_server::rebind(&config, &login, &key) {
        Ok(display) => {
            eprintln!(
                "h3-server: `{login}` ({display}) is bound to the new key: from the next start only it logs in"
            );
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("h3-server: rebind: {e}");
            ExitCode::FAILURE
        }
    }
}

/// `text` on standard output, or why the command `command` failed on standard error.
fn print_or_fail(command: &str, text: Result<String, h3_server::Error>) -> ExitCode {
    match text {
        Ok(text) => {
            println!("{text}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("h3-server: {command}: {e}");
            ExitCode::FAILURE
        }
    }
}

/// The operator's card (E16 M6 T2): the running server's, on standard output.
fn card(path: &OsString) -> ExitCode {
    let Some(config) = load(path) else {
        return ExitCode::FAILURE;
    };
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build();
    let card = match runtime {
        Ok(runtime) => runtime.block_on(h3_server::card(&config)),
        Err(e) => Err(h3_server::Error(format!("runtime: {e}"))),
    };
    match card {
        Ok(card) => {
            println!("{card}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("h3-server: card: {e}");
            ExitCode::FAILURE
        }
    }
}

/// Runs the server until it is stopped.
fn serve(path: &OsString) -> ExitCode {
    let Some(config) = load(path) else {
        return ExitCode::FAILURE;
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
