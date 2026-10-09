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
| `full-s3-a755.toml` | full | 3 | 755 (E04b T7b3, replacing T7b2b's `full-s3-a1914`; the scenario has a Seraph and Fiends since T7b3: the Seraph raises a stack of Brea's for good, Fiends raise Spearmen from a corpse of Corwin's; a stack is copied and its clone vanishes; Revive raises creatures for the battle only; Brutes' strike spells; Drakes draw, casters cast) |
| `full-s3-a1253.toml` | full | 3 | 1253 (E04b T7b3, replacing T7b2b's `full-s3-a1535`: Brea wins; the Seraph's and the Fiends' raises; Corwin conjures Shades; mines or pits go off; Revive for the battle only) |
| `full-s3-a709.toml` | full | 3 | 709 (E04b T7b3, replacing T7b2b's `full-s3-a1886`: a stack is sacrificed to raise another (Offer); the Seraph's and the Fiends' raises; casters cast) |
| `full-s3-a631.toml` | full | 3 | 631 (E04b T7b3, replacing T7b2b's `full-s3-a463`: a stack is copied and its clone vanishes; mines or pits go off; the Seraph's and the Fiends' raises) |
| `full-s3-a373.toml` | full | 3 | 373 (E04b T7b3, replacing T7b2b's `full-s3-a1825`: Corwin conjures Shades; mines or pits go off; the Seraph's and the Fiends' raises; casters cast) |

Regenerate (only when a change *intentionally* alters game behavior, the synthetic pack, or an
algorithm/encoding version — say so in the commit message):

```sh
cargo build -p h3-cli
for spec in "duel 1 1 duel-s1" "skirmish 7 8 skirmish-s7" "full 3 4 full-s3" "full 3 755 full-s3-a755" \
    "full 3 1253 full-s3-a1253" "full 3 709 full-s3-a709" "full 3 631 full-s3-a631" "full 3 373 full-s3-a373"; do
  set -- $spec
  ./target/debug/h3 run --scenario packs/synthetic/scenarios/$1.toml --seed $2 --agent-seed $3 \
    --max-steps 2000 --checkpoint-every 25 --record fixtures/replays/$4.toml
done
```
