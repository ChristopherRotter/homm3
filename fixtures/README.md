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
| `full-s3-a1367.toml` | full | 3 | 1367 (E04b T9b, replacing T7c3's `full-s3-a1798`; since T9b, the casting workload, both heroes cast opening spells (Brea a mass Hasten, Corwin a mass Drag and Bane) and roll Eagle Eye at each of the other's casts, Corwin's stacks resist 10 %, each side has a Pendant and Brea six spells at Advanced, in every full replay, with the rebirth and Fear of T7c3, the creature defences of T7c2a and the mana creatures of T7c1: here Brea wins, the rare result; three rebirths; a resisted spell; mines or pits go off; re-recorded in E23 T1c, the walks and the berserk turns on the original's pathfinder: from command 151 another game, Brea still wins, in 316 commands, with three rebirths, a resisted spell and two passed turns) |
| `full-s3-a242.toml` | full | 3 | 242 (E04b T11b, replacing `full-s3-a955`, whose game lost its mirrored spell with the review's fixes: a mirror sends a hero's spell away; bad morale; a stack is sacrificed to raise another; a raise) |
| `full-s3-a1766.toml` | full | 3 | 1766 (E04b T9b, replacing `full-s3-a116`: a stack is sacrificed to raise another; a stack is created and vanishes; the Fiends raise Spearmen from a corpse; a resisted spell) |
| `full-s3-a303.toml` | full | 3 | 303 (E04b T9b, replacing `full-s3-a762`: a stack is created and vanishes; Blink; mines or pits go off; creatures cast; a raise) |
| `full-s3-a2378.toml` | full | 3 | 2378 (E04b T9b, replacing `full-s3-a959`: sixteen mines or pits go off; the Fiends raise Spearmen from a corpse; a spell heals seven times; re-recorded in E23 T1c, as `full-s3-a1367`: from command 142 another game, the defender still wins, in 480 commands, fourteen mines or pits going off, the Fiends' raise and eight spell heals) |
| `field-s5.toml` | field | 5 | 6 (E04c T2b: the skirmish's armies on the special battlefield `synthetic:hallow`, its own obstacle type drawn, the vale's morale raised) |
| `siege-s7.toml` | siege | 7 | 8 (E04c T5: the skirmish's armies at the vale's Citadel with a damaging moat of two rows; re-recorded in E04c T6, when Brea leads the attacker with her Catapult at Advanced Ballistics, Hurlers and Quake: her Catapult's shots miss onto other walls and bring down two of them and the keep, the Hurlers destroy the gate, two Quakes hit an absent tower (BAT-QUIRK-22) and bring down the other two walls; re-recorded in E04c T7, when the vale's Citadel sets up its keep of seven Spires, 4 + the town's three structures, which shoots first every round on its own, BAT-SIE-016; the attacker still wins, in 90 commands; re-recorded in E04c T7d, when the keep aims as the original's computer, BAT-SIE-020: the attacker still wins, in 104 commands; re-recorded in E04c T8, when the vale's Tavern and fountain raise its defenders' morale and luck by 1, BAT-SIE-025: the attacker still wins, in 224 commands; re-recorded in E04c T10a, when the Catapult's stack id comes before the Ballista's, as the original's records (D-23): the same 89 commands, new hashes; re-recorded in E04c T10b, the review's fixes (the attacker wins a speed tie after the Catapult's turn, the commanded Catapult neither waits nor defends, the computer's weak walkers keep out of the moat): the defender wins, in 146 commands, with four wall shots and two Quakes) |

**Only a hash moved** (a quirk flag that comes or goes, BAT-QUIRK-23, BAT-QUIRK-24): keep the
replay's setup and commands and change the `ruleset_hash` in its header, then `h3 verify` it.
Re-running its spec is no substitute: since E04c T10b's obstacle draw (17a0489b) `duel-s1` and
`field-s5` get other obstacles than the ones they store (E23 T1c).

**Only the state hash moved** (`BATTLE_STATE_HASH_VERSION`, E23 T5b2b's v52, E23 T6's v53; E23
T6 also the replay format v30, the setup's retreat facts and the attacker's ground): the
commands stay. The loop below re-records the eight others with the same commands; `duel-s1` and
`field-s5` (E23 T6: all ten) are rehashed from their stored setup and commands by a throwaway test
in `crates/h3-cli/tests`
(`ReplayDoc::parse`, `into_replay`, the header `ReplayHeader::current`, each command applied with
the checkpoints taken, `replay_to_toml`), then `h3 verify`. Check that only hash lines and the
version moved. The adventure lane's replays hold battles too: re-record them with their options
(`crates/h3-cli/tests/adventure_replays.rs`), `cross-lane`.

Regenerate (only when a change *intentionally* alters game behavior, the synthetic pack, or an
algorithm/encoding version — say so in the commit message). To pick a new replay's agent seed,
count the events per agent seed with a throwaway test in `crates/h3-cli/tests` (`run_random`, then
re-apply the commands with an event sink):

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
