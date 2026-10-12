//! The client's pure crates keep their purity header (*provisional*, E17 M0 T1). The engine's
//! `cargo xtask check-arch` does not see the client workspace; `no_std` makes I/O, clocks, threads
//! and the environment unavailable at compile time, so the check is that the header stays.

/// The crate roots of the pure crates (the libraries; `h3-view`'s binary is an adapter).
const ROOTS: [(&str, &str); 3] = [
    ("h3-assets", include_str!("../../h3-assets/src/lib.rs")),
    ("h3-render", include_str!("../../h3-render/src/lib.rs")),
    ("h3-view", include_str!("../src/lib.rs")),
];

/// Attributes every pure crate root carries.
const REQUIRED: [&str; 2] = ["#![no_std]", "#![forbid(unsafe_code)]"];

/// Lints every pure crate root denies (the engine's logic header, `arch.toml` `logic_deny`).
const DENIED: [&str; 10] = [
    "clippy::unwrap_used",
    "clippy::expect_used",
    "clippy::panic",
    "clippy::todo",
    "clippy::unimplemented",
    "clippy::unreachable",
    "clippy::float_arithmetic",
    "clippy::print_stdout",
    "clippy::print_stderr",
    "clippy::exit",
];

/// The text of the root's `#![deny(...)]` attribute, or an empty string.
fn deny_block(source: &str) -> &str {
    source
        .split_once("#![deny(")
        .and_then(|(_, rest)| rest.split_once(")]"))
        .map_or("", |(block, _)| block)
}

#[test]
fn pure_crate_roots_carry_the_header() {
    for (name, source) in ROOTS {
        for attribute in REQUIRED {
            assert!(source.contains(attribute), "{name}: missing `{attribute}`");
        }
        let block = deny_block(source);
        for lint in DENIED {
            assert!(block.contains(lint), "{name}: `{lint}` is not denied");
        }
        for relaxed in ["#![allow(", "#![expect(", "#![warn(clippy::"] {
            assert!(
                !source.contains(relaxed),
                "{name}: crate-level `{relaxed}` relaxes the header"
            );
        }
    }
}

#[test]
fn deny_block_reads_the_attribute() {
    assert_eq!(deny_block("#![deny(a, b)]\n"), "a, b");
    assert_eq!(deny_block("no attribute"), "");
}
