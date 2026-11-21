# E31 — The original's computer player

Status: **refined** 2026-11-21 (AI-Q22); increment 1 refined, increments 2 to 7 sketched; starts
after E25's increment 10 (T43, T44) · Lane: **ai** · Follows E23 (the battle AI) and E26 (the
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

## What the first look found (2026-11-21, the refinement; `scripts/exe.py`'s `Code`)

- **Entry:** `ai_player_turn` 0x525E80 (from `adventure_manager_procedure` 0x408BD8): its heroes
  one by one through `ai_hero_turn` 0x5261F0 (the order by `ai_next_hero` 0x526A90, not read),
  then a second pass over those with points left; the planning 0x42DE50 and `ai_hero_replan`
  0x42E0B0, the walk `ai_hero_walk` 0x42FEE0 and `ai_hero_step` 0x42FC50; the town's choice
  `ai_town_build_choice` 0x42AE00. Read so far only around these calls (E08, E09, E24 M4 T1b1).
- **Size:** two ranges hold the AI: **0x42A000–0x434620** (103 functions, about 41 KB: the
  planning, the walk, the artifact values) and **0x524370–0x52BD50** (87 functions, about 31
  KB: the visits' answers, the skill, spell and building values, `ai_fight_estimate` 0x528040 at
  5.7 KB). About **72 KB in 190 functions**, about four times E23's battle AI range; 22 of them
  read, 24 inferred. The part every direct caller of which is the AI's own: 135 functions, about
  61 KB. Some valuations are shared with the battle AI (`ai_caster_record` 0x526D40, the spell
  values 0x5270E0 to 0x5275B0, read by E23) and some are reached through a function table
  (0x432F90).
- **Draws:** almost none. One `random_between` call (0x42FBE9, in `ai_hero_step`), no direct
  `crt_rand`. About 750 x87 instructions, most in the values.
- **Hidden state, a first case:** the adventure pathfinder lets a computer player step from an
  explored tile into an unexplored one, or anywhere while its player record's +0x3E is above 0
  (`pathfinder_fog_test` 0x4B25D4, ADV-MOV-008): the fog of war is partly ignored.
- **The computer's own rules:** several `ai_*` calls sit inside the game's visits and buys, not
  in a choice: the 500-gold spell book bought at a town without a question (`ai_town_visit`
  0x5253D0), the computer's boat (`ai_boat_create` 0x430F80), `ai_artifact_received` 0x433B40,
  `ai_town_buy_hook` 0x526D20, the starting resources by difficulty (`starting_resources_computer`
  0x6781FC, already a rule).
- **Memory between turns:** per-player records at 0x692950 + 0x98 × player (`ai_player_values`
  0x6929E0 among them, which the level-up's default answer reads: ADV-STUB-8).

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
   whose rules are missing (E09's increments 6 and 7, E08's increment 15, E10, E12's quests)
   stays an `AI-STUB-<n>` naming the epic that lifts it, and the ai lane's inbox takes the port
   when that lane delivers. E31 does not wait for them.
7. **Strength waits for games that end.** Before E10's increment 1, E31 checks whole games to a
   day limit: legal commands only, no stall, a time budget per turn. The fair-versus-faithful
   head-to-head (as E26's) is the last increment, once a game can end. Not chosen: a day-limit
   score as a proxy; no strength measurement.

**Agent decisions** (change freely):

- **SoD's code only.** What RoE's and AB's executables do differently is E11's.
- **All five difficulties**, as the original reads `[game]`'s difficulty.
- **A pure crate of its own** in the ai lane (`h3-game-ai` proposed), registered by a `shared/`
  pull request at increment 2; its surface and the policy's signature are T1b's ADR draft. A
  computer player's battles are played by E23's port in the faithful variant and E26's fair
  actor in the fair one, through the system actor's existing battle seat.
- **Sources:** the code reading (primary); the original under Wine (observation); H3API's names
  and record layouts (pointers only); VCMI's adventure AI is a different design and is not
  ported (a pointer for what an object is worth at most).
- A code reading is insight only; nothing printed by `exe.py` is committed (Q29).

## Increment 1 — the reading's map and the survey (refined)

| Task | What | Out |
|---|---|---|
| T1a1 | The frame: `ai_player_turn`, `ai_next_hero`'s order, `ai_hero_turn`, the planning 0x42DE50 and `ai_hero_replan`, the walk and its step, the turn's end; the per-player records' layout (0x692950) into `structs.toml` | `map.toml`; the survey's §1 (the turn's phases, the records) |
| T1a2 | The values: what a hero's plan weighs (objects, towns, enemy heroes, monsters: the function table 0x432F90, `ai_fight_estimate` 0x528040, the artifact, skill, spell and army values), every read of another player's state or of an unexplored tile | `map.toml`; the survey's §2 (each value; the hidden reads listed) |
| T1a3 | The towns and the answers: `ai_town_build_choice`, `ai_town_visit` (recruit, garrison), hiring at a Tavern, the markets; every `ai_*` call inside a visit, each classified a choice or a rule | `map.toml`; the survey's §3 (the table of asymmetries for the owning lanes) |
| T1a4 | The rest of both ranges named at least `inferred` and placed in a phase; every draw and float site listed; the fields that persist between turns and their readers | `map.toml`; the survey's §4 |
| T1b | The survey `docs/rules/surveys/E31-computer-player.md` finished (each phase's rules, the hidden reads, the asymmetries, the engine's gaps per epic); the ADR draft (the crate, the seam, the memory, the commands, the battles, the floats under `kernel::num`, the fair variant's line); the asymmetries into the owning lanes' inboxes; the trace's plan (maps, days, the harness); the cut of increments 2 to 7 with exit criteria; the checkpoint AI-Q23 | survey, ADR draft, this file |

T1a's cut may change at its start, by size (E23's T1a ran in five parts).

**Exit criteria:** (1) every function of both ranges in the map, at least `inferred`, with its
phase; (2) every hidden read, draw and float site listed; (3) every asymmetry classified and
handed to its lane; (4) the survey and the ADR draft written; (5) increments 2 to 7 cut with
exit criteria and confirmed at the checkpoint.

## Later increments (sketched; cut at T1b)

2. **The crate and the frame:** `h3-game-ai` registered; a turn that ends; heroes ordered and
   walked to what the plan values most, on the engine's pathfinder; the first trace compared.
3. **The towns:** the buildings, the recruitment, the heroes hired, the garrisons.
4. **The visits and the fights:** every answer a visit asks, the fight-or-flee and join
   decisions, the battles through the system actor's seat; ADV-STUB-8 and ADV-STUB-9 lifted
   with the adventure lane.
5. **Whole games to a day limit:** every shipped map played by computer players only (legal
   commands, no stall), a benchmark and a time budget per turn, the stubs' list current; the
   remaining traces.
6. **The fair variant:** the policy on its player's projection (the fog kept, no other player's
   hidden state); its properties (it reads nothing hidden, as E26's world property).
7. **The measurement and the closure** (after E10's increment 1): fair against faithful head to
   head on 1v1 small maps, the cost of a turn, the default switched to the fair one by the
   callers' lanes; the closure.

## Not in E31

The learned adventure agent (E13). The game's end and setup (E10). The computer players' rules
themselves, which the owning lanes build (decision 2). RoE's and AB's differences (E11). VCMI's
or HotA's AI, but as pointers.

## Risks

- **Size:** about 72 KB to read, four times E23's; more is shared with the game than the battle
  AI was. Mitigation: T1a in parts by size, the map grown as E23's was.
- **State to match for traces:** the engine's state must equal the original's at the trace's
  start (the map, the setup, the neutral battles' draws). T1b plans the traces on maps where
  it can be made so; where it cannot, the reading and spot checks stand alone (decision 1).
- **The engine's gaps:** much of what the AI values is not yet a rule (decision 6). The stubs
  keep the port honest; the measurement waits for E10.
- **Memory:** state kept between turns outside the game must be rebuilt for a replay or a
  loaded game; T1b's ADR settles it (decision 5).

## Tasks done

None yet.
