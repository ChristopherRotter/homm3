# Rule catalog

One TOML file per bounded context: `battle.toml`, `adventure.toml`, `town.toml`, `heroes.toml`, `magic.toml`, `game.toml`, `kernel.toml`.

Format, status meanings and traceability rules: `docs/04-rules-and-sources.md` §3.

ID scheme: `<CTX>-<AREA>-<NNN>` with CTX in `BAT`, `ADV`, `TWN`, `HER`, `MAG`, `GAM`, `KER`. Example: `BAT-DMG-001`. IDs are never reused; retired rules keep their ID with `status = "retired"` and a note.

The catalog files are created by the first epic that needs them (E01 creates `battle.toml` and `kernel.toml`). `cargo xtask check-rules` validates them.
