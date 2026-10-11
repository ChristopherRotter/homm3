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
| `full-s3-a1367.toml` | full | 3 | 1367 (E04b T9b, replacing T7c3's `full-s3-a1798`; since T9b, the casting workload, both heroes cast opening spells (Brea a mass Hasten, Corwin a mass Drag and Bane) and roll Eagle Eye at each of the other's casts, Corwin's stacks resist 10 %, each side has a Pendant and Brea six spells at Advanced, in every full replay, with the rebirth and Fear of T7c3, the creature defences of T7c2a and the mana creatures of T7c1: here Brea wins, the rare result; three rebirths; a resisted spell; mines or pits go off; a creature casts; both heroes' Eagle Eye picks) |
| `full-s3-a955.toml` | full | 3 | 955 (E04b T9b, replacing `full-s3-a1788`: a mirror sends a spell away; a resisted spell; Blink; mines or pits go off; creatures cast) |
| `full-s3-a1766.toml` | full | 3 | 1766 (E04b T9b, replacing `full-s3-a116`: a stack is sacrificed to raise another; a clone is created and vanishes; the Fiends raise Spearmen from a corpse; a resisted spell) |
| `full-s3-a303.toml` | full | 3 | 303 (E04b T9b, replacing `full-s3-a762`: a stack is created and vanishes; Blink; mines or pits go off; creatures cast; a raise) |
| `full-s3-a804.toml` | full | 3 | 804 (E04b T9b, replacing `full-s3-a959`: twenty mines or pits go off; Blink; the Fiends raise Spearmen from a corpse; Eagle Eye picks three spells for Brea and two for Corwin) |

Regenerate (only when a change *intentionally* alters game behavior, the synthetic pack, or an
algorithm/encoding version — say so in the commit message):

```sh
cargo build -p h3-cli
for spec in "duel 1 1 duel-s1" "skirmish 7 8 skirmish-s7" "full 3 4 full-s3" "full 3 1367 full-s3-a1367" \
    "full 3 955 full-s3-a955" "full 3 1766 full-s3-a1766" "full 3 303 full-s3-a303" "full 3 804 full-s3-a804"; do
  set -- $spec
  ./target/debug/h3 run --scenario packs/synthetic/scenarios/$1.toml --seed $2 --agent-seed $3 \
    --max-steps 2000 --checkpoint-every 25 --record fixtures/replays/$4.toml
done
```
