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
| `full-s3-a1367.toml` | full | 3 | 1367 (E04b T9b, replacing T7c3's `full-s3-a1798`; since T9b, the casting workload, both heroes cast opening spells (Brea a mass Hasten, Corwin a mass Drag and Bane) and roll Eagle Eye at each of the other's casts, Corwin's stacks resist 10 %, each side has a Pendant and Brea six spells at Advanced, in every full replay, with the rebirth and Fear of T7c3, the creature defences of T7c2a and the mana creatures of T7c1: here Brea wins, the rare result; three rebirths; a resisted spell; mines or pits go off) |
| `full-s3-a242.toml` | full | 3 | 242 (E04b T11b, replacing `full-s3-a955`, whose game lost its mirrored spell with the review's fixes: a mirror sends a hero's spell away; bad morale; a stack is sacrificed to raise another; a raise) |
| `full-s3-a1766.toml` | full | 3 | 1766 (E04b T9b, replacing `full-s3-a116`: a stack is sacrificed to raise another; a stack is created and vanishes; the Fiends raise Spearmen from a corpse; a resisted spell) |
| `full-s3-a303.toml` | full | 3 | 303 (E04b T9b, replacing `full-s3-a762`: a stack is created and vanishes; Blink; mines or pits go off; creatures cast; a raise) |
| `full-s3-a2378.toml` | full | 3 | 2378 (E04b T9b, replacing `full-s3-a959`: sixteen mines or pits go off; the Fiends raise Spearmen from a corpse; a spell heals seven times) |
| `field-s5.toml` | field | 5 | 6 (E04c T2b: the skirmish's armies on the special battlefield `synthetic:hallow`, its own obstacle type drawn, the vale's morale raised) |
| `siege-s7.toml` | siege | 7 | 8 (E04c T5: the skirmish's armies at the vale's Citadel with a damaging moat of two rows; re-recorded in E04c T6, when Brea leads the attacker with her Catapult at Advanced Ballistics, Hurlers and Quake: her Catapult's shots miss onto other walls and bring down two of them and the keep, the Hurlers destroy the gate, two Quakes hit an absent tower (BAT-QUIRK-22) and bring down the other two walls; re-recorded in E04c T7, when the vale's Citadel sets up its keep of seven Spires, 4 + the town's three structures, which shoots first every round on its own, BAT-SIE-016; the attacker still wins, in 90 commands; re-recorded in E04c T7d, when the keep aims as the original's computer, BAT-SIE-020: the attacker still wins, in 104 commands) |

Regenerate (only when a change *intentionally* alters game behavior, the synthetic pack, or an
algorithm/encoding version — say so in the commit message):

```sh
cargo build -p h3-cli
for spec in "duel 1 1 duel-s1" "skirmish 7 8 skirmish-s7" "full 3 4 full-s3" "full 3 1367 full-s3-a1367" \
    "full 3 242 full-s3-a242" "full 3 1766 full-s3-a1766" "full 3 303 full-s3-a303" "full 3 2378 full-s3-a2378" \
    "field 5 6 field-s5" "siege 7 8 siege-s7"; do
  set -- $spec
  ./target/debug/h3 run --scenario packs/synthetic/scenarios/$1.toml --seed $2 --agent-seed $3 \
    --max-steps 2000 --checkpoint-every 25 --record fixtures/replays/$4.toml
done
```
