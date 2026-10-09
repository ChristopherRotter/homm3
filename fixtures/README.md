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
| `full-s3-a1159.toml` | full | 3 | 1159 (E04b T7c1, replacing T7b4's `full-s3-a753`; the scenario has mana creatures since T7c1: the Brutes lower their hero's costs, Brea's Djinns raise Corwin's, the Wisps channel, Corwin's Fiends drain Brea, in every full replay; a stack is copied and its clone vanishes; the Seraph's and the Fiends' raises; the Chanters' casts; casters cast) |
| `full-s3-a416.toml` | full | 3 | 416 (E04b T7c1, replacing T7b4's `full-s3-a38`: Brea wins; a raise; the Chanters' casts; casters cast) |
| `full-s3-a255.toml` | full | 3 | 255 (E04b T7c1, replacing T7b4's `full-s3-a402`: a stack is sacrificed to raise another (Offer); the Seraph's and the Fiends' raises; mines or pits go off; the Chanters' casts) |
| `full-s3-a48.toml` | full | 3 | 48 (E04b T7c1, replacing T7b4's `full-s3-a1732`: a stack is copied and its clone vanishes; Corwin conjures Shades; mines or pits go off; a raise; twelve Chanter casts; casters cast) |
| `full-s3-a1023.toml` | full | 3 | 1023 (E04b T7c1, replacing T7b4's `full-s3-a1006`: a stack is copied and Corwin conjures Shades; the Seraph's and the Fiends' raises; mines or pits go off; casters cast) |

Regenerate (only when a change *intentionally* alters game behavior, the synthetic pack, or an
algorithm/encoding version — say so in the commit message):

```sh
cargo build -p h3-cli
for spec in "duel 1 1 duel-s1" "skirmish 7 8 skirmish-s7" "full 3 4 full-s3" "full 3 1159 full-s3-a1159" \
    "full 3 416 full-s3-a416" "full 3 255 full-s3-a255" "full 3 48 full-s3-a48" "full 3 1023 full-s3-a1023"; do
  set -- $spec
  ./target/debug/h3 run --scenario packs/synthetic/scenarios/$1.toml --seed $2 --agent-seed $3 \
    --max-steps 2000 --checkpoint-every 25 --record fixtures/replays/$4.toml
done
```
