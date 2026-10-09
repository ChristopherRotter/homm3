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
| `full-s3-a1798.toml` | full | 3 | 1798 (E04b T7c3, replacing T7c2a's `full-s3-a676`; the scenario has the rebirth and Fear since T7c3: Brutes of both sides come back at a fifth once a battle, Corwin's Drakes frighten Brea's stacks (10 %), with the creature defences of T7c2a and the mana creatures of T7c1, in every full replay: here Brea wins; three rebirths; mines or pits go off; a stack is created; casters cast) |
| `full-s3-a1788.toml` | full | 3 | 1788 (E04b T7c3, replacing T7c2a's `full-s3-a1386`: a Drake sends a spell away; a stack is sacrificed to raise another; a stack is created; two rebirths, eight freezes; the Chanters' casts) |
| `full-s3-a116.toml` | full | 3 | 116 (E04b T7c3, replacing T7c2a's `full-s3-a1473`: a Drake sends a spell away; two rebirths, eleven freezes; mines or pits go off often; the Chanters' casts; casters cast) |
| `full-s3-a762.toml` | full | 3 | 762 (E04b T7c3, replacing T7c2a's `full-s3-a471`: a Drake sends a spell away; seventeen freezes; fifteen Chanter casts; two raises) |
| `full-s3-a959.toml` | full | 3 | 959 (E04b T7c3, replacing T7c2a's `full-s3-a673`: a Drake sends a spell away; three rebirths; casters cast) |

Regenerate (only when a change *intentionally* alters game behavior, the synthetic pack, or an
algorithm/encoding version — say so in the commit message):

```sh
cargo build -p h3-cli
for spec in "duel 1 1 duel-s1" "skirmish 7 8 skirmish-s7" "full 3 4 full-s3" "full 3 1798 full-s3-a1798" \
    "full 3 1788 full-s3-a1788" "full 3 116 full-s3-a116" "full 3 762 full-s3-a762" "full 3 959 full-s3-a959"; do
  set -- $spec
  ./target/debug/h3 run --scenario packs/synthetic/scenarios/$1.toml --seed $2 --agent-seed $3 \
    --max-steps 2000 --checkpoint-every 25 --record fixtures/replays/$4.toml
done
```
