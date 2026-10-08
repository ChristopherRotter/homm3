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
| `full-s3-a1914.toml` | full | 3 | 1914 (E04b T7b2b, replacing T7b2a's `full-s3-a759`: Brea casts Snare, Corwin Mire and Uproot; her mines and his pits go off; Brutes blind, curse, petrify, paralyse, sicken, age, poison, dispel, zap, bind and etch after their strikes, with their extra damage; Drakes draw at each activation; a Djinn casts Boon, drawn at the cast) |
| `full-s3-a1535.toml` | full | 3 | 1535 (E04b T7b2b, replacing T7b2a's `full-s3-a804`: Corwin casts Mire, whose pits go off, and conjures Shades (Summon Elemental); he wins with Shades alive; a Wisp casts Hasten, Drakes Arc and Cinders, a Djinn Boon) |
| `full-s3-a1886.toml` | full | 3 | 1886 (E04b T7b2b, replacing T7b2a's `full-s3-a1522`: a stack is sacrificed to raise another (Offer); a stack is raised for good; a Wisp casts Hasten, a Drake Spark, a Djinn Hallow) |
| `full-s3-a463.toml` | full | 3 | 463 (E04b T7b2b, replacing T7b2a's `full-s3-a1432`: Brea's Revive raises creatures for the battle only, her mines go off, and she wins; a Djinn casts Aegis) |
| `full-s3-a1825.toml` | full | 3 | 1825 (E04b T7b2b, replacing T7b2a's `full-s3-a155`: a stack is copied (Clone), and the clone vanishes; Corwin conjures Shades and wins with them alive, which the outcome leaves out; Drakes cast Spark, a Djinn Aegis) |

Regenerate (only when a change *intentionally* alters game behavior, the synthetic pack, or an
algorithm/encoding version — say so in the commit message):

```sh
cargo build -p h3-cli
for spec in "duel 1 1 duel-s1" "skirmish 7 8 skirmish-s7" "full 3 4 full-s3" "full 3 1914 full-s3-a1914" \
    "full 3 1535 full-s3-a1535" "full 3 1886 full-s3-a1886" "full 3 463 full-s3-a463" "full 3 1825 full-s3-a1825"; do
  set -- $spec
  ./target/debug/h3 run --scenario packs/synthetic/scenarios/$1.toml --seed $2 --agent-seed $3 \
    --max-steps 2000 --checkpoint-every 25 --record fixtures/replays/$4.toml
done
```
