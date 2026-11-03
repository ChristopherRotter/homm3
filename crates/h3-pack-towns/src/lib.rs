//! Adapter: parses the packs' `towns.toml` files (one pack file per context, ADR-0006's E08
//! amendment) into the town context's definitions, which `h3-towns` compiles into its ruleset part
//! (ADR-0028 point 5): `compile(texts, profile)` over the enabled packs' files in dependency order
//! onto a compiled core, overrides as ADR-0006's; a pack folder without the file reads as an empty
//! one (`packs/README.md`). The base pack holds the eight factions' town records, the AB pack
//! Conflux's, the SoD pack its overrides.
//!
//! So far a skeleton (E09 T4b): the parser, the compiler's entry and the format (documented here,
//! as `h3-pack-adventure` documents `adventure.toml`) come with E09 T5a (E09 increment 1's design
//! note §2).
#![forbid(unsafe_code)]
#![warn(missing_docs)]
