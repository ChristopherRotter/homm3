//! `h3-server <config.toml>`: hosts the configured battle for untrusted clients over WebSocket
//! (E16, ADR-0018); the configuration's format is in `h3_server::config`. Runs until it is
//! stopped; the replay is written when the battle ends.

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
                let _ = running.listener.await;
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("h3-server: {e}");
                ExitCode::FAILURE
            }
        }
    })
}
