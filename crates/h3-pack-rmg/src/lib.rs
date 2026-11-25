//! Adapter: parses the packs' `rmg.toml` files into the map generation context's tables, which
//! `h3-rmg` compiles into its rmg part (ADR-0029 point 5): `packs/ab/rmg.toml` the AB template
//! table, `objects.txt`'s template lines and `rand_trn.txt`'s rows as the game reads them,
//! `packs/sod/rmg.toml` SoD's additions and the generator's constants read from the executable.
//!
//! So far: nothing (E14 T2a made the crate). The parser and its format come with T2b.
#![forbid(unsafe_code)]
#![warn(missing_docs)]
