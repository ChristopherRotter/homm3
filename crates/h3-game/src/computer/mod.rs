//! The original's computer player on the adventure map (E31; `docs/epics/E31-original-computer-
//! player.md`): the turns of a player the original's computer plays, read from `Heroes3.exe`
//! (SoD's code) and ported as a policy outside the game state, one validated [`GameCommand`]
//! a call. Its rules are `CPU-` ids (`docs/rules/catalog/computer.toml`), its quirks `AI-QUIRK-`
//! (`docs/rules/quirks-ai.md`), its values in the original's x87 arithmetic
//! (`h3_kernel::arith::X87`, `SignedX87`).
//!
//! The ai lane's by path (`lanes.toml`; the user, AI-Q31): a module of `h3-game` for now, using
//! only the crate's public surface and the crates below it, so that its later move into a crate of
//! its own changes no choice. Its surface (`choose`, `Memory`) is built from E31 increment 2;
//! the design: ADR-0030 (`docs/adr/0030-original-computer-player.md`).
//!
//! [`GameCommand`]: crate::GameCommand
