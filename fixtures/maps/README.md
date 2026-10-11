# Generated maps

Adventure-lane fixtures (`lanes.toml`): `.h3m` files our own code generates, committed once, so
that tests, golden replays and benchmarks play a real map file without the original game's maps
(those stay local, `docs/formats/h3m.md`; the user dropped hand-made editor maps, E08 2026-10-11).

Each file is laid out by `crates/h3-map/src/tests/fixtures.rs` from a plan built with a fixed
generator, written gzip-compressed as the game stores maps. That file's test reads each committed
map and checks that it is still its plan's map and imports. A change of a committed map changes
its content hash and so invalidates every replay that names it (ADV-Q2): regenerate only on
purpose, with `H3_WRITE_FIXTURES=1 cargo test -p h3-map --lib tests::fixtures`, and say why in the
commit message.

| File | Map | Played by |
|---|---|---|
| `e08-duel-36.h3m` | SoD, 36 × 36 with an underground; red and blue with two heroes each, one per level, their types' starting armies; terrain in 6 × 6 patches, a road per level, 72 blockers | `h3-sim`'s adventure env tests (E08 T9a); T9b's random-play benchmark and golden replays (decision 11) |
