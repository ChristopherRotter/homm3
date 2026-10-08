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
| `full-s3-a759.toml` | full | 3 | 759 (E04b T7b2a, replacing T7b1's `full-s3-a51`: Brea casts Snare and Blink, Corwin Mire and Uproot; her mines and his pits go off; Brutes blind, curse, petrify, paralyse, sicken, age, poison, zap, bind and etch after their strikes, with their extra damage; Drakes draw at each activation and cast Arc) |
| `full-s3-a804.toml` | full | 3 | 804 (E04b T7b2a, replacing T7b1's `full-s3-a168`: Corwin casts Mire, whose pits go off, and conjures Shades (Summon Elemental); Brea's mines go off; he wins with Shades alive; a Wisp casts Hasten, a Drake Spark) |
| `full-s3-a1522.toml` | full | 3 | 1522 (E04b T7b2a, replacing T7b1's `full-s3-a42`: a stack is sacrificed to raise another (Offer); a stack is raised for good; Corwin wins with Shades alive; Wisps cast Hasten twice; Drakes cast Cinders and Spark) |
| `full-s3-a1432.toml` | full | 3 | 1432 (E04b T7b2a, replacing T7b1's `full-s3-a225`: Brea's Revive raises creatures for the battle only, she sacrifices a stack to raise another, and she wins; a Wisp casts Hasten, a Drake Arc) |
| `full-s3-a155.toml` | full | 3 | 155 (E04b T7b2a, replacing T7b1's `full-s3-a123`: a stack is copied (Clone), and the clone vanishes; Corwin conjures Shades and wins with them alive, which the outcome leaves out; a Wisp casts Hasten; Drakes cast Cinders and Spark) |

Regenerate (only when a change *intentionally* alters game behavior, the synthetic pack, or an
algorithm/encoding version — say so in the commit message):

```sh
cargo build -p h3-cli
for spec in "duel 1 1 duel-s1" "skirmish 7 8 skirmish-s7" "full 3 4 full-s3" "full 3 759 full-s3-a759" \
    "full 3 804 full-s3-a804" "full 3 1522 full-s3-a1522" "full 3 1432 full-s3-a1432" "full 3 155 full-s3-a155"; do
  set -- $spec
  ./target/debug/h3 run --scenario packs/synthetic/scenarios/$1.toml --seed $2 --agent-seed $3 \
    --max-steps 2000 --checkpoint-every 25 --record fixtures/replays/$4.toml
done
```
