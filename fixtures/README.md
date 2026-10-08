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
| `full-s3-a51.toml` | full | 3 | 51 (E04b T7b1, replacing T7a3's `full-s3-a471`: Brea casts Snare and Blink, Corwin Mire and Uproot; her mines and his pits go off; Brutes blind, curse, petrify, paralyse, sicken, age, poison, dispel, zap, bind and etch after their strikes, with their extra damage; Wisps cast Hasten twice) |
| `full-s3-a168.toml` | full | 3 | 168 (E04b T7b1, replacing T7a3's `full-s3-a53`: Corwin casts Mire, whose pits go off, and conjures Shades (Summon Elemental); he wins with Shades alive; Wisps cast Hasten) |
| `full-s3-a42.toml` | full | 3 | 42 (E04b T7b1, replacing T7a3's `full-s3-a6`: a stack is sacrificed to raise another (Offer); a stack is raised for good; a Wisp casts Hasten) |
| `full-s3-a225.toml` | full | 3 | 225 (E04b T7b1, replacing T7a3's `full-s3-a126`: Brea's Revive raises creatures for the battle only, and she wins; Wisps cast Hasten) |
| `full-s3-a123.toml` | full | 3 | 123 (E04b T7b1, replacing T7a3's `full-s3-a235`: a stack is copied (Clone), and the clone vanishes; Corwin conjures Shades and wins with them alive, which the outcome leaves out; a Wisp casts Hasten) |

Regenerate (only when a change *intentionally* alters game behavior, the synthetic pack, or an
algorithm/encoding version — say so in the commit message):

```sh
cargo build -p h3-cli
for spec in "duel 1 1 duel-s1" "skirmish 7 8 skirmish-s7" "full 3 4 full-s3" "full 3 51 full-s3-a51" \
    "full 3 168 full-s3-a168" "full 3 42 full-s3-a42" "full 3 225 full-s3-a225" "full 3 123 full-s3-a123"; do
  set -- $spec
  ./target/debug/h3 run --scenario packs/synthetic/scenarios/$1.toml --seed $2 --agent-seed $3 \
    --max-steps 2000 --checkpoint-every 25 --record fixtures/replays/$4.toml
done
```
