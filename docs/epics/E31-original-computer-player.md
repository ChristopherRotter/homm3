# E31 — The original's computer player

Status: **in progress** since 2026-11-23 (T1a1a; refined 2026-11-21, AI-Q22); increment 1 closed
2026-11-24 (AI-Q31), increments 2 to 7 cut, increment 2 next; started after E25's increment 10 (AI-Q23) · Lane: **ai** · Follows E23 (the battle AI) and E26 (the
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
  signature and the memory are the ADR draft's. A computer player's battles are played by the
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
| 1 | 2026-11-24 | T1a1a to T1a4, T1b | the reading's map (255 functions of three ranges, each in a phase) and the survey `docs/rules/surveys/E31-computer-player.md` (§0 to §5); the ADR draft `docs/lanes/ai/adr-drafts/original-computer-player.md`; the design note `E31-design.md`; increments 2 to 7 cut (AI-Q31) |

## Increments 2 to 7 (cut at T1b; confirmed at the checkpoint, AI-Q31)

The ADR draft (`docs/lanes/ai/adr-drafts/original-computer-player.md`, cited as "the draft") and
the design note (`E31-design.md`: the traces §1, the experiments §2, the seam §3, the memory §4).
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

## Tasks done

Increment 2's, once refined (increment 1's: `E31-archive.md`).
