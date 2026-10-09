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
| `full-s3-a676.toml` | full | 3 | 676 (E04b T7c2a, replacing T7c1's `full-s3-a1159`; the scenario has creature defences since T7c2a: Corwin's Drakes mirror spells (20 %), his Fiends reflect melee strikes (20 %), Brea's Djinns deal half damage to two-hex stacks and Corwin's Chanters to Djinns, and the mana creatures of T7c1, in every full replay: here a Drake sends a spell away; the Seraph's and the Fiends' raises; the Chanters' casts; casters cast) |
| `full-s3-a1386.toml` | full | 3 | 1386 (E04b T7c2a, replacing T7c1's `full-s3-a416`: Brea wins; mines or pits go off; the Chanters' casts; casters cast) |
| `full-s3-a1473.toml` | full | 3 | 1473 (E04b T7c2a, replacing T7c1's `full-s3-a255`: a Drake sends a spell away; a stack is sacrificed to raise another (Offer); mines or pits go off; the Chanters' casts; casters cast) |
| `full-s3-a471.toml` | full | 3 | 471 (E04b T7c2a, replacing T7c1's `full-s3-a48`: a Drake sends a spell away; a stack is copied and its clone vanishes; mines or pits go off; the Chanters' casts) |
| `full-s3-a673.toml` | full | 3 | 673 (E04b T7c2a, replacing T7c1's `full-s3-a1023`: a Drake sends a spell away; Corwin conjures Shades; ten Chanter casts) |

Regenerate (only when a change *intentionally* alters game behavior, the synthetic pack, or an
algorithm/encoding version — say so in the commit message):

```sh
cargo build -p h3-cli
for spec in "duel 1 1 duel-s1" "skirmish 7 8 skirmish-s7" "full 3 4 full-s3" "full 3 676 full-s3-a676" \
    "full 3 1386 full-s3-a1386" "full 3 1473 full-s3-a1473" "full 3 471 full-s3-a471" "full 3 673 full-s3-a673"; do
  set -- $spec
  ./target/debug/h3 run --scenario packs/synthetic/scenarios/$1.toml --seed $2 --agent-seed $3 \
    --max-steps 2000 --checkpoint-every 25 --record fixtures/replays/$4.toml
done
```
