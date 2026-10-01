//! Shared kernel: ids, fixed-capacity storage, deterministic PRNG, canonical state hashing, numerics,
//! modifiers (ADR-0012).
//!
//! Logic crate: pure, deterministic, `no_std` + `alloc` (ADR-0003, ADR-0005). The lint header
//! below is required and checked by `cargo xtask check-arch`.
#![no_std]
#![forbid(unsafe_code)]
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::todo,
    clippy::unimplemented,
    clippy::unreachable,
    clippy::float_arithmetic,
    clippy::print_stdout,
    clippy::print_stderr,
    clippy::exit
)]
#![warn(missing_docs)]

pub mod arith;
pub mod fixed;
pub mod handle;
pub mod hash;
pub mod modifier;
pub mod num;
pub mod order;
pub mod rng;

pub use fixed::{CapacityError, FixedVec};
pub use handle::Handle;
pub use hash::{CanonicalHash, StateHasher};
pub use modifier::{Modifier, ModifierOp, ModifierPool, SourceKey};
pub use rng::{Rng, StreamId};
