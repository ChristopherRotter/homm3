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
| `full-s3-a362.toml` | full | 3 | 362 (E04b T5a: its game casts Scorch and Blink; since T5c2 also Snare) |
| `full-s3-a246.toml` | full | 3 | 246 (E04b T5c2, replacing T5c1's `full-s3-a5`, which no longer casts Mire: Snare twice, eight mines go off under Corwin's stacks; since T5b Mire's pits stop walkers 8 times) |
| `full-s3-a138.toml` | full | 3 | 138 (E04b T5b: Corwin's Uproot takes away a hex of Brea's Scorch) |

Regenerate (only when a change *intentionally* alters game behavior, the synthetic pack, or an
algorithm/encoding version — say so in the commit message):

```sh
cargo build -p h3-cli
for spec in "duel 1 1 duel-s1" "skirmish 7 8 skirmish-s7" "full 3 4 full-s3" "full 3 362 full-s3-a362" \
    "full 3 246 full-s3-a246" "full 3 138 full-s3-a138"; do
  set -- $spec
  ./target/debug/h3 run --scenario packs/synthetic/scenarios/$1.toml --seed $2 --agent-seed $3 \
    --max-steps 2000 --checkpoint-every 25 --record fixtures/replays/$4.toml
done
```
