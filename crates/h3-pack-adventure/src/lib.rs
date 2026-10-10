//! Adapter: parses the packs' `adventure.toml` files (one pack file per context, ADR-0006's E08
//! amendment) into the adventure context's definitions, which `h3-adventure` compiles into its
//! ruleset part (ADR-0015 point 2). The files are hand-written source the harvester never touches
//! (E08 decision 10); parsing is strict like `h3-pack`'s, and the compiler validates everything
//! again.
//!
//! Skeleton (E08 T2): the format arrives with the adventure content in T4.
