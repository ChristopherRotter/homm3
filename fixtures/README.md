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
| `full-s3-a199.toml` | full | 3 | 199 (E04b T6a, replacing T5a's `full-s3-a362`: Brea casts Blink, Scorch and Snare; four mines go off) |
| `full-s3-a325.toml` | full | 3 | 325 (E04b T6a, replacing T5c2's `full-s3-a246` and T5b's `full-s3-a138`: Corwin's Uproot takes away Brea's Scorch; Snare's mines go off four times; Mire's pits stop walkers) |
| `full-s3-a52.toml` | full | 3 | 52 (E04b T6a: Brea sacrifices a stack to raise another (Offer); Corwin raises a dead stack for good (Revive at Expert)) |
| `full-s3-a149.toml` | full | 3 | 149 (E04b T6a: Brea's Revive without Stone magic raises 21 creatures for the battle only; she wins, and the outcome leaves them out) |

Regenerate (only when a change *intentionally* alters game behavior, the synthetic pack, or an
algorithm/encoding version — say so in the commit message):

```sh
cargo build -p h3-cli
for spec in "duel 1 1 duel-s1" "skirmish 7 8 skirmish-s7" "full 3 4 full-s3" "full 3 199 full-s3-a199" \
    "full 3 325 full-s3-a325" "full 3 52 full-s3-a52" "full 3 149 full-s3-a149"; do
  set -- $spec
  ./target/debug/h3 run --scenario packs/synthetic/scenarios/$1.toml --seed $2 --agent-seed $3 \
    --max-steps 2000 --checkpoint-every 25 --record fixtures/replays/$4.toml
done
```
