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
| `full-s3-a1396.toml` | full | 3 | 1396 (E04b T6b, replacing T6a's `full-s3-a199`: Brea casts Blink, Scorch and Snare; her mines and Corwin's pits go off) |
| `full-s3-a300.toml` | full | 3 | 300 (E04b T6b, replacing T6a's `full-s3-a325`: Corwin conjures twelve Shades (Summon Elemental); his Uproot takes away Brea's Scorch; Snare's mines and Mire's pits go off) |
| `full-s3-a1476.toml` | full | 3 | 1476 (E04b T6b, replacing T6a's `full-s3-a52`: Brea sacrifices a stack to raise another (Offer); Corwin raises a stack for good (Revive at Expert)) |
| `full-s3-a1436.toml` | full | 3 | 1436 (E04b T6b, replacing T6a's `full-s3-a149`: Brea's Revive without Stone magic raises creatures for the battle only; she wins with 14 of them, which the outcome leaves out) |
| `full-s3-a728.toml` | full | 3 | 728 (E04b T6b: Corwin conjures Shades; Brea copies a stack of 30 (Clone), and the clone vanishes; Corwin wins with Shades alive, which the outcome leaves out) |

Regenerate (only when a change *intentionally* alters game behavior, the synthetic pack, or an
algorithm/encoding version — say so in the commit message):

```sh
cargo build -p h3-cli
for spec in "duel 1 1 duel-s1" "skirmish 7 8 skirmish-s7" "full 3 4 full-s3" "full 3 1396 full-s3-a1396" \
    "full 3 300 full-s3-a300" "full 3 1476 full-s3-a1476" "full 3 1436 full-s3-a1436" "full 3 728 full-s3-a728"; do
  set -- $spec
  ./target/debug/h3 run --scenario packs/synthetic/scenarios/$1.toml --seed $2 --agent-seed $3 \
    --max-steps 2000 --checkpoint-every 25 --record fixtures/replays/$4.toml
done
```
