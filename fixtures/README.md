# Fixtures

## `replays/` — golden replays (determinism)

Random-vs-random battles on the synthetic pack, recorded with `h3 run ... --record`. Each file
holds the ruleset identity, algorithm versions, seed, setup, command log, a state hash every 25
commands and the final hash. `crates/h3-cli/tests/replay_fixtures.rs` re-runs them on every
`cargo test`; `cargo xtask determinism` also runs them in release and with
`-C target-cpu=native`. Running the same test on another OS (Windows, macOS) is the
cross-platform determinism check (ADR-0003 §11, E01 exit criterion 2).

| File | Scenario | Seed | Agent seed |
|---|---|---|---|
| `duel-s1.toml` | duel | 1 | 1 |
| `skirmish-s7.toml` | skirmish | 7 | 8 |
| `full-s3.toml` | full | 3 | 4 |
| `full-s3-a213.toml` | full | 3 | 213 (E04b T7a2, replacing T7a1's `full-s3-a385`: Brea casts Snare and Blink, Corwin Mire, Revive and Uproot; her mines and his pits go off; Brutes blind, curse, petrify, paralyse, sicken, age, poison, dispel and zap after their strikes) |
| `full-s3-a804.toml` | full | 3 | 804 (E04b T7a2, replacing T7a1's `full-s3-a1059`: Corwin casts Mire, whose pits go off, and conjures twelve Shades (Summon Elemental); he wins with Shades alive) |
| `full-s3-a196.toml` | full | 3 | 196 (E04b T7a2, replacing T7a1's `full-s3-a124`: Brea sacrifices a stack to raise another (Offer); Corwin raises a stack for good (Revive at Expert)) |
| `full-s3-a61.toml` | full | 3 | 61 (E04b T7a2, replacing T7a1's `full-s3-a300`: Brea's Revive without Stone magic raises 4 creatures for the battle only; she wins with them, which the outcome leaves out) |
| `full-s3-a514.toml` | full | 3 | 514 (E04b T7a2, replacing T7a1's `full-s3-a301`: Brea copies a stack of 25 (Clone), and the clone vanishes; Corwin conjures Shades and wins with them alive, which the outcome leaves out) |

Regenerate (only when a change *intentionally* alters game behavior, the synthetic pack, or an
algorithm/encoding version — say so in the commit message):

```sh
cargo build -p h3-cli
for spec in "duel 1 1 duel-s1" "skirmish 7 8 skirmish-s7" "full 3 4 full-s3" "full 3 213 full-s3-a213" \
    "full 3 804 full-s3-a804" "full 3 196 full-s3-a196" "full 3 61 full-s3-a61" "full 3 514 full-s3-a514"; do
  set -- $spec
  ./target/debug/h3 run --scenario packs/synthetic/scenarios/$1.toml --seed $2 --agent-seed $3 \
    --max-steps 2000 --checkpoint-every 25 --record fixtures/replays/$4.toml
done
```
