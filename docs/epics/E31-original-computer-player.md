# E31 — The original's computer player

Status: **in progress** since 2026-11-23 (T1a1a; refined 2026-11-21, AI-Q22); increment 1 closed
2026-11-24 (AI-Q31), increments 2 to 7 cut; increment 2 refined 2026-11-25 (AI-Q32: ADR-0030
accepted, eleven tasks), T2a to T2c done, T2d next; started after E25's increment 10 (AI-Q23) · Lane: **ai** · Follows E23 (the battle AI) and E26 (the
fair system actor) for battle · Prerequisite of E13 (its target and ladder opponent) ·
Consumers: the env (a computer seat), the server (a computer player in a hosted game), the shell
(a game's computer players), the adventure lane (ADV-STUB-8, ADV-STUB-9)

## Goal

The adventure turns of a player that the original's computer plays, read from `Heroes3.exe`
(GOG Complete, SoD's code) and ported as a policy outside the game state, as E23 did for battle:
which hero moves where, what it visits and how it answers, what each town builds and recruits,
which heroes are hired, when a turn ends. **Faithful first**: it reads what the original reads,
hidden state included where the original reads it. **Then fair**: a variant that decides only on
its player's projection (`docs/06-security.md`, "Information hiding"), as E26 did for battle.
Every step it takes is one of the game's validated commands.

## Decisions at the refinement (the user, 2026-11-21, AI-Q22)

1. **Reading first, spot checks** (E23's way). The code reading is the main source. An
   experiment row (the original under Wine, `experiments/agent/`) confirms only a reading at
   risk (`docs/04-rules-and-sources.md` §2). Beside them a few **short traces**: a computer
   player's first days on two or three maps compared decision by decision, where the engine's
   state can be made to match the original's. No trace corpus as a gate. Not chosen: reading
   and spot checks without traces; a trace oracle gating every increment (E14's way).
2. **The computer's own rules are engine rules, built by the lanes that own them.** The survey
   lists every asymmetry. A rule (keyed on the player's `human` flag, `GameSetup`'s, which the
   engine already has) goes to the owning lane's inbox and is built there; a choice becomes the
   policy's legal command. The policy issues only validated `GameCommand`s. Not chosen: the ai
   lane building them `cross-lane`; dropping what no command can express.
3. **Faithful, then fair** (E13 decision 4's reading confirmed). The faithful port is a target
   and a ladder opponent only and trains no agent (ADR-0026 point 8, as amended for battle). The
   fair variant may train and becomes the game's default computer player; the faithful one stays
   a game option, as E26's all-seeing battle actor did.
4. **The order:** after E25's increment 10 (T43's measurement and T44), as AI-Q21 set; no
   interleaving within the lane.
5. **The AI's memory is decided after the reading.** T1 lists the fields that persist between
   turns and who reads them; T1b's ADR draft decides, leaning toward the policy's own memory
   outside the game state (kept by its host beside the journal, rebuilt by replaying it), with any
   field that a rule reads moved into the state.
6. **Port now, stub the gaps.** Everything is read; what the engine can play is ported; a part
   whose rules are missing (E09's increments 6 to 8, E08's increment 15, E10, E12's quests)
   stays an `AI-STUB-<n>` naming the epic that lifts it, and the ai lane's inbox takes the port
   when that lane delivers. E31 does not wait for them.
7. **Strength waits for games that end.** Before E10's increment 1, E31 checks whole games to a
   day limit: legal commands only, no stall, a time budget per turn. The fair-versus-faithful
   head-to-head (as E26's) is the last increment, once a game can end. Not chosen: a day-limit
   score as a proxy; no strength measurement.

**Agent decisions** (change freely):

- **SoD's code only.** What RoE's and AB's executables do differently is E11's.
- **All five difficulties**, as the original reads `[game]`'s difficulty.
- **A module of `h3-game` for now** (`h3_game::computer`, the ai lane's by path; the user at the
  checkpoint, AI-Q31), refactored into a crate of its own later; its surface, the policy's
  signature and the memory are ADR-0030's (accepted at increment 2's start, AI-Q32). A computer player's battles are played by the
  host's battle seat: E23's port in the faithful variant, E26's fair actor in the fair one. Every
  battle is fought until the battle lane decides the original's quick battle (§3.6 row 1; the
  draft's point 7, AI-Q31).
- **Sources:** the code reading (primary); the original under Wine (observation); H3API's names
  and record layouts (pointers only); VCMI's adventure AI is a different design and is not
  ported (a pointer for what an object is worth at most).
- A code reading is insight only; nothing printed by `exe.py` is committed (Q29).

## Closed increments

Their scope, task lists and the first look are in `E31-archive.md`; each task's record in
`E31-log.md`.

| Increment | Closed | Tasks | Content |
|---|---|---|---|
| 1 | 2026-11-24 | T1a1a to T1a4, T1b | the reading's map (255 functions of three ranges, each in a phase) and the survey `docs/rules/surveys/E31-computer-player.md` (§0 to §5); the ADR draft (ADR-0030 since AI-Q32); the design note `E31-design.md`; increments 2 to 7 cut (AI-Q31) |

## Increments 2 to 7 (cut at T1b; confirmed at the checkpoint, AI-Q31)

ADR-0030 (`docs/adr/0030-original-computer-player.md`, the draft until AI-Q32; cited as "the
draft" in increments 3 to 7 below) and the design note (`E31-design.md`: the traces §1, the experiments §2, the seam §3, the memory §4).
Each increment is refined (its tasks cut by size) at its start.

2. **The module and the frame.** Before its code, one `shared/` pull request: `h3-game`'s
   `src/computer/` the ai lane's by path, `computer.toml` and `quirks-ai.md` (the draft's point 1) and
   the signed `X87` (point 9). Then `choose` and `Memory` (points 2, 5); the turn's loop (§1.1 to §1.6:
   the start's records and weights, the heroes' order, both passes, the done marks, the end); the
   planning (§1.9) over the object values of the classes the engine has (§2.1, §2.3 to §2.5; the
   rest `AI-STUB`s by epic, survey §5.4); the simulated combat (§2.2) for the fight estimates; the
   walk (§1.10) as `MoveHero` on the engine's pathfinder with the marks' overlay (the adventure
   lane's API), the spells and jumps stubbed until E08 increment 15. **Exit:** (1) the draft's
   point 3 property tests on random maps and five shipped maps at all five difficulties; (2) trace
   T-A's days 1 to 3: every destination that is not noise-bound and every walk equal; (3) the
   benchmark gated and the turn's cost recorded; (4) every routine's `CPU-` rule id, coverage 100 %;
   (5) the stubs listed in `docs/stubs.toml`.
3. **The towns.** The middle (§1.5, §3.1, §3.2: the build choice, the trades and Marketplaces, the
   hires, the allies' shares) and the town visit (§3.3: the garrison and the swap, the recruits,
   the guild, the stock, the Blacksmith, the University, the book, the Grail), the defence
   recruits of the start and the middle. **Exit:** trace T-C's days 1 to 7 equal in every town
   decision; the property tests over the towns; E09's increments 6 to 8 stubbed where unbuilt.
4. **The answers and the fights.** Every answer of §3.4 and §3.5 (the visits' dialogs, the joins
   and the pursued monster, the Seer, the markets, the level-up), ADV-STUB-8 and ADV-STUB-9 lifted
   with the adventure lane, the battles through the draft's point 2 (E23's port, the retreat
   facts), the attacker's split; AI-QUIRK-1 registered with the first slip built (each slip
   blind-confirmed first). **Exit:** every open decision answered by the policy on the shipped
   maps (`default_answer` only behind a stub); trace T-A to day 7 equal, its fights included; each
   slip built tested both ways.
5. **Whole games to a day limit.** Every shipped map with computer players only to a day limit
   (28 days), the env's and the server's computer seats (their lanes' inboxes), the budget set at
   its checkpoint (the draft's point 13), T-B and experiments X1 and X2, the stubs' list current.
   **Exit:** no refusal, fallback outside the stubs, or stall on any shipped map to the limit at
   any difficulty; the budget held; the traces' and experiments' results recorded.
6. **The fair variant.** The player's world (`h3-game`'s lanes, their inboxes) and `choose_fair`
   (the draft's point 14), the hidden reads of §5.2 replaced one by one. **Exit:** on random
   states, two games with one projection give the same command; increment 5's games pass with
   the fair variant.
7. **The measurement and the closure** (after E10's increment 1): fair against faithful head to
   head on 1v1 small maps (as E26), the turn's cost, the callers' defaults switched to the fair one
   by their lanes; the closure.

## Not in E31

The learned adventure agent (E13). The game's end and setup (E10). The computer players' rules
themselves, which the owning lanes build (decision 2). RoE's and AB's differences (E11). VCMI's
or HotA's AI, but as pointers.

## Risks

- **Size:** about 95 KB read in 255 functions (increment 1); the port is cut by phase, and each
  increment's tasks by size at its refinement.
- **State to match for traces:** the engine's state must equal the original's at the trace's
  start. The traces run on compiled maps without setup draws and without battles between
  computers (`E31-design.md` §1); the noise-bound destinations are compared only as candidates.
- **The engine's gaps:** much of what the AI values is not yet a rule (decision 6; the survey §5.4:
  40, most of them E09's increments 6 to 8 and E10's). The stubs keep the port honest; the
  measurement waits for E10.
- **The rules the lanes have not decided** (the survey §5.3's nine rows): the policy plans as a
  human meanwhile; a townless computer that ignores the fog in the original keeps to explored
  tiles in the port (ADV-MOV-008) until the adventure lane decides row 9.

## Increment 2's tasks (refined 2026-11-25, AI-Q32)

**Done first:** the `shared/` pull request #1163 (`shared/e31-module`): the module's paths for the
ai lane (`h3-game`'s `src/computer/`, `computer.toml`, `quirks-ai.md`, the scoped verify with the
adventure lane's crates), the context `computer` in `check-rules`, the ledger with AI-QUIRK-1
reserved and the exploit-fix entries AI-QUIRK-2 to -4, `h3_kernel::arith::SignedX87` with its
audit, ADR-0030 numbered.

**The measurement** (the address map's sizes of the functions each part ports, at about 2.7 bytes
an instruction as T1a4 measured): about 71 KB, 26 000 instructions, 63 % of the reading. **The cut**
(the user, AI-Q32): eleven tasks by dependency, the leaves first, about one session each (T2a and T2e
small enough to pair). Each task ports with its `CPU-` rule ids, its tests and coverage at 100 %,
its gaps as `AI-STUB`s by epic (survey §5.4) in `docs/stubs.toml`, its values in `X87` /
`SignedX87`; a slip is ported as the rule until AI-QUIRK-1 (increment 4). The instruction counts
are the original's, from `docs/exe/map.toml`'s sizes.

| Task | Content | Functions (survey §) | ≈ instructions |
|---|---|---|---|
| T2a | **The frame**: `choose` and `Memory` (its canonical encoding and hash, ADR-0030 points 2, 5), `POLICY_VERSION`; the turn's loop (`ai_player_turn` 0x525E80: the phases, the heroes' order `ai_next_hero` 0x526A90, both passes, the done marks, the end), each hero's turn stubbed until T2i and T2j (done at once, then `EndTurn`); the property-test harness (random maps and five shipped maps at all five difficulties: legal, ends within a bound, a function of state, memory and stream); the benchmark's workload (`benches/ai/`); the map-client lane told the entry point (`MCL-STUB-5`) | 9 (§1.1 to §1.4, §1.6) | 1 400 |
| T2b | **The simulated combat, the sides and the melee**: `sim_combat_start` 0x423EE0, `sim_side_build` 0x424120, the walls, the speed, the damage taken and spread, the wipe, the hold, the finish, the engagement, the run, the stack lists, `sim_apply_losses` | 18 (§2.2) | 2 700 |
| T2c | **The simulated combat's spells and Necromancy**, `ai_army_fight_sim` 0x427330 (the fight estimate the values call) | 13 (§2.2) | 2 600 |
| T2d | **The start's values**: `ai_turn_start` 0x4297C0, `ai_turn_start_values` 0x527960, the resource weights (`ai_resource_weights` 0x428740), the recruit reserve, the eyes, the heroes' effectiveness, the vision's value, the threat fold over the enemies' reach (`ai_enemy_reach_fold` 0x4280E0, `ai_town_capture_sim`), the difficulty's blend, the terrain shares | 14 (§1.5, §1.6, §2.5) | 2 300 |
| T2e | **The spells' and skills' values**: `ai_spell_value` 0x5273D0 and its kinds, the caster's record, `ai_secondary_skill_value` 0x524690, the schools, the University's | 11 (§2) | 1 700 |
| T2f | **The artifacts**: the effects' table (`ai_artifact_effects_build` 0x434100, the effects of §4.1), `ai_hero_artifact_value` 0x4336C0, the values to a hero and to a player | 37 (§4.1, §2.4) | 2 900 |
| T2g | **The army and town values**: the army plans (`ai_army_plan_*`, `ai_stack_marginal_value` 0x42C830, the exchange and purchase values; AI-QUIRK-2), the dwelling's, the town's and its buildings', the weighted cost | 22 (§2.3, §2.4) | 3 400 |
| T2h | **The object value** `ai_object_value` 0x528040 over the classes the engine has (§2.1's 36 value functions, the guarded object's fight), the rest `AI-STUB`s by epic | 39 (§2.1) | 4 300 less the gaps |
| T2i | **The planning**: the danger map `ai_danger_map` 0x42DE50, the candidates `ai_hero_candidates` 0x42EDD0, the claims, the candidate's score, the destination `ai_hero_destination` 0x42E0B0 with its noise (76 to 100, the caller's stream), the class tables | 11 (§1.9) | 2 900 |
| T2j | **The walk**: `ai_hero_walk` 0x42FEE0 and its steps as `MoveHero` on the engine's pathfinder (the boat marks' overlay the adventure lane's API, stubbed until it lands; the spells and jumps stubbed until E08 increment 15), the boat's purchase; exit criteria (1), (3) to (5): the property tests over whole turns, the benchmark gated and the turn's cost recorded, the stubs listed | 8 (§1.10) | 2 200 |
| T2k | **Trace T-A's harness and days 1 to 3** (`E31-design.md` §1: the map compiled by `h3-experiment`, the original under Wine through the capture harness with `h3mem.py`'s readers, `h3 ai trace`, the diff); exit criterion (2) | the harness | — |

## Tasks done

Increment 2's (increment 1's: `E31-archive.md`).

- **T2a** (2026-11-25): the frame: `h3_game::computer::choose`, `Memory` (encoding v1),
  `POLICY_VERSION` 1, the turn's passes, the order of heroes, a hero's turn with its garrison step
  (CPU-TRN-001 to -005; AI-STUB-2 to -6); `h3_sim::computer::play` and its property tests on the
  generated maps at every difficulty, every shipped map in the original-data check; `h3 ai
  computer`, the workload `computer-turns-towns-36`. Record: `E31-log.md` T2a.
- **T2b** (2026-11-25): the simulated combat's sides and melee in `h3_game::computer`'s
  `sim.rs` (CPU-SIM-001 to -005): a side and its hero's reads, the losses, the damage and the
  stand-off, the lookahead, the run; the casts AI-STUB-7 (T2c, which also takes
  `sim_combat_start`'s casting inputs and `sim_stack_take_value`); §5.5's #63 and #64 from the
  blind re-read. Record: `E31-log.md` T2b.
- **T2c** (2026-11-25, not cut, AI-Q34): the simulated combat moved to a public
  `h3_battle::estimate` (`cross-lane`; the battle lane's module, on contract inputs) with its casts
  (CPU-SIM-006 to -010: the choice, the damage spells with their chains and areas, the mass damage,
  the other spells, the raises, on the battle's own spell helpers); the fight estimate and
  Necromancy in `h3_game::computer`'s `fight.rs` (CPU-SIM-011, -012); AI-STUB-7 lifted, AI-STUB-8
  (the aggressiveness, 1.0) and AI-STUB-9 (the conditions, E10) registered; §5.5's #65 and #66.
  Record: `E31-log.md` T2c.
