# E31 — design note (working detail; not binding)

The handover beside ADR-0030 (`docs/adr/0030-original-computer-player.md`, the draft until AI-Q32). Each
task that builds a section deletes it (`docs/adr/README.md`, "Keep ADRs short"); the file goes when
E31 closes. The reading: `docs/rules/surveys/E31-computer-player.md` (cited as §n).

## 1. The traces (decision 1; written at T1b)

A few short traces, a computer player's first days compared decision by decision where the
engine's state can be made to equal the original's. Not a gate corpus.

**Maps:** three small maps compiled by `h3-experiment` (`experiments/specs/E31-trace-*.toml`, the
ai lane's), everything placed so that no setup placeholder draws: no random object, monster, hero,
town, resource or pile; each placed hero with its army, skills, spells and experience given; no
wandering monster within reach in the trace's days (their aggression is a setup draw) and no battle
between two computers (§2.2: the original simulates it, decision 7 of the ADR draft); one human
seat far away that only ends its turns.

| Trace | Map | Difficulty | Days | Compares | With |
|---|---|---|---|---|---|
| T-A | one computer town (Castle) and two heroes, mines, piles, chests, a few visitable objects the engine has | Normal | 1–7 | the heroes' order, each destination and end tile, the visits and their answers, the resources | increment 2 (days 1–3), 4 (to day 7) |
| T-B | T-A's with a Grail and an Expert computer | Expert | 1–4 | T-A's fields, and the generator's state after the turn start (experiment X1) | increment 2 or 3 |
| T-C | two computer towns, a Tavern, a Marketplace, the gold short for the first builds | Hard | 1–7 | the builds, the trades, the hires, the recruits, the garrison, the guild's and the stock's buys | increment 3 |

**What is compared:** after each computer turn, the heroes (tile, points, the destination fields
+0x35 to +0x41, army), the player (resources, the record at 0x692950 + 0x98 × colour: the weights),
the towns (built, garrison, the visitor). The draw-free decisions (builds, hires, recruits, trades,
answers: §4.4 has no draw in them) must be equal. A destination is equal when the best candidate's
score is more than 100/76 times the second's (the noise of §1.9 cannot reorder them); otherwise it
is recorded as noise-bound, and only its membership in the candidates is compared.

**Harness:** the original under Wine through the capture harness (`clients/capture/`, a virtual
clock, hot seat; `docs/experiments/README.md`, "Agents running a batch themselves"), the human
seat's turns ended by a recipe, and new `h3mem.py` readers for the fields above (the common
harness; a small addition at increment 2 with the map-client lane told). The engine side:
`h3 ai trace <map> --days <n>` (`h3-cli`'s `src/ai/`) prints the same fields per computer turn; a
script diffs the two. The traces' outputs stay uncommitted (original output, ADR-0009); their
results go into the rule catalog's sources (`observation`).

## 2. The confirmation experiments (the second look, T1b)

Two readings decide a classification and want the original's behaviour seen (Q30):

- **X1, the turn start's reseed** (§4.4): on T-B, the C runtime's seed read after
  `player_puzzle_target` 0x4BAE50 returns on days 2 to 4: expected colour × 424 909 + 423 869 while
  no piece is uncovered, the same on each day.
- **X2, the records across games** (§4.6): in one process, a first game to a computer's turn
  start; then a second game loaded in which a computer hero levels up as a defender before its
  player's first turn (the human attacks it on day 1 and loses); the record's gold weight (+0x90)
  read at the level-up: expected the first game's last value, not 0 or a fresh one.

Both ride on the trace harness (increment 2 or 3). Until they report, the port does what ADR
draft points 5 and 8 say (no reseed, a record from the first turn start).

## 3. What the port needs from the other lanes (the seam)

| Need | Where | Lane | When |
|---|---|---|---|
| The pathfinder's query with an overlay (the boat marks, §1.3, §1.10) and the original's fog test for a computer (ADV-MOV-008) | `h3_adventure::reach` | adventure | increment 2 |
| Each object's contents and guards read as the original reads them (the faithful policy reads the whole state) | `h3-adventure`'s public accessors | adventure | increments 2, 4 |
| A town's buildable set, prices and growth as the original's build choice reads them | `h3-towns`, `legal_commands` | town | increment 3 |
| The hero's derived values (the spells and masteries, the movement) | `h3-heroes` (ADR-0013 point 2) | adventure | increment 2 |
| The level-up's and the retreat's computer facts (ADV-STUB-8, ADV-STUB-9) | `h3-game` | adventure | increment 4 |
| The rule rows of §3.6 decided (the quick battle, Dimension Door's points, the Castle Gate, the Shipyard, the patrol, the artifact price, the defender's split) | their lanes | battle, adventure, town | their decision; the policy plans as a human until then |
| A computer seat in the env and the server | `h3-sim`, `h3-server` | ai, server | increment 5 |
| The player's world for the fair variant | `h3-game` | adventure (with town, battle) | increment 6 |

## 4. The memory (ADR draft point 5)

`Memory` per computer player, from §4.6: carried between its turns: per hero the destination
(+0x35 to +0x40) and the points beyond (+0x41); within a turn: the phase cursor (start, first
pass, middle, second pass), the heroes done (the original's sleep +0x11C), the must-go (+0x43),
the record's weights (+0x128 f64[7], their means +0x160, +0x164), the Grail target (+0x39), the
Eye value (+0x04), the gold weight (+0x90), the danger map, the threats (+0x11D per enemy hero, a
town's +0x3) and the hero values (+0x109, +0x47A to +0x48E, f32). The hero values are read in other
players' turns by the quick battle (§4.6); the policy's own answers in other turns read the record
of its last start. The patrol (+0x44 to +0x46) is the map's (the adventure lane's state).
