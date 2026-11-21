# E13 — Adventure AI (hierarchical)

Status: **refined in shape, not started** (the user, 2026-11-21, AI-Q21) · Lane: **ai** (moved
from the adventure lane by AI-Q21) · Starts when **E10 is done whole** and **E31** (the
original's computer player) is done; its increments are cut at its start, with the measurements
of that time.

## Goal

An adventure agent that plays a whole game of the original's rules on real maps, through the
adventure env (`AdventureEnv`, `h3-sim/src/adventure/`), and beats the original's computer
player (E31). It decides at the level of goals. A deterministic executor turns each goal into the
game's legal commands. Its battles are played by a battle policy. It is legitimate: it reads only
its player's projection (`docs/06-security.md`, "Information hiding"), as the battle agent does
(ADR-0025 point 5).

## Decisions at the refinement (the user, 2026-11-21, AI-Q21)

1. **A game that ends is E10's, all of it.** E13 does not start before E10 (victory and defeat,
   teams, diplomacy, the setup beyond the import) is done. No sim-only end condition and no
   piece of E10 pulled forward for E13. Not chosen: the original's standard defeat rule pulled
   forward alone; a day limit with a score in the sim layer.
2. **The ai lane owns E13** (`lanes.toml`): the env, its encodings, the C ABI and the trainer are
   already its own. It takes E13 after E25's increment 10 and after E31. Not chosen: a new lane
   in parallel (the GPU would be shared, `h3-sim` and `python/` split); the adventure lane.
3. **The target is the original's computer player, ported first** as its own epic, **E31**, in
   the ai lane. It reads the adventure AI of `Heroes3.exe` (the computer players' turns) and
   ports it as a policy outside the game state, as E23 did for battle. Its code reading can
   start before E10 is done. Not chosen: heuristic bots and self-play alone as the target; VCMI's
   AI as the reference (both may still be ladder rungs).
4. **Faithful first, fair later** (E31). The port reads first what the original reads, which may
   be state hidden from its player: the fog of war is widely reported to be ignored, and the code
   reading confirms or refutes this. A fair variant then decides only on its player's projection,
   as E26 did for battle. *Agent's reading, as ADR-0026 point 8 amended for battle (E25 decision
   2):* the faithful one is a target and ladder opponent only and trains no agent; the fair one
   may train and becomes the game's default computer player. Confirmed at E31's refinement.
5. **Macro-actions over a deterministic executor** (`docs/07-simulation-and-ai.md` §3). The
   policy chooses a goal, for example: go to an object, a hero or a town; build; recruit; split or
   merge armies; trade; end the hero's or the player's turn. An executor in `h3-sim` (pathing,
   sequencing, a visit's answers) carries it out as the game's legal commands, so every step is
   still validated by the game. The flat per-tile action encoding (v17 today) stays for replays,
   tools and the random agent. The macro set, its masks and the executor's design are the first
   increment's.
6. **Battles: the fair system actor in training, the battle agent at evaluation.** During
   training the fair system actor (the battle lane's, E26) plays every side of every battle,
   inside the engine's step. At evaluation the battle agent (E25's current best, `e25-structure-1`
   #2 today) plays the adventure agent's own battles, so strength is measured with the real
   battle AI. Whether to fight or flee is the adventure's question (`docs/07` §1).
7. **The reward: the game's outcome, with an annealed shaping.** The objective is winning (+1,
   −1, a draw or a cut game 0). A potential-based shaping term (for example army value, towns,
   heroes and income against the opponent's) helps early training and anneals to zero, so the
   final agent optimizes only the outcome. The potentials and the schedule are decided at E13's
   start.
8. **First scale: 1v1 on the shipped small maps.** Two players on S-size (36 × 36) shipped maps,
   the others closed, a held-out set for evaluation. More players, teams and larger maps come in
   later increments; generated maps join when there are generators: E14 (the original's random
   map generator, the rmg lane) and E30 (the custom objective-driven generator, built on
   E14). The epic is no longer titled "training on generated maps".

## Prerequisites (each another epic's or lane's)

- **E10** whole (the adventure lane): the game's end, the teams, the setup's choices.
- **E31** (this lane): the faithful computer player, then the fair one.
- **E09** (the town lane): the towns' economy far enough that a game is the original's
  (construction, growth, recruitment, the Tavern and the mage guild exist; travel and the Grail
  are increment 7).
- **The adventure observation encoding** (none exists, `docs/08-work-process.md`, "Versions"):
  E13's first increment, unless E31's fair variant needs it first.
- The AI queue's inbox note for E13 (the env's ABI: both mask heads read in one step; the current
  state's queries in the env).

## Increments (sketch; cut at E13's start)

1. The observation encoding, the macro-actions and their executor, the reward with its shaping,
   the C ABI for the adventure batch; scripted baselines (random goals, a greedy collector, a
   rusher) and an evaluation runner against E31's computer player.
2. A first training run of 1v1 on small maps; its reading at the checkpoint.
3. Onwards as the measurements say: the network for the map (spatial and entity parts), the
   league, more players and teams, larger maps, generated maps (E14, E30).

## Exit criteria (to set at E13's start)

A win rate against E31's fair computer player above a target the user sets with the first
measurements, on held-out shipped small maps, with the battle agent playing the agent's battles;
the agent still beats every scripted baseline. Verified as E06 and E25 were (the evaluation gate,
ADR-0026 point 7).

## Not in E13

The original's computer player itself (E31). The game's end and setup (E10). The map generators
(E14, E30). A stronger battle agent (E25). Simultaneous turns (E08, not placed).

## Risks

- **Waiting long.** E13 waits on E10 and E31, both large; E31's code reading of the adventure AI
  is likely bigger than E23's.
- **Horizon and cost.** A small map's game runs for weeks of game days, with hundreds of steps on
  the random agent (463 a game on `e08-trades-36` over 28 days, E08 T135b1). The executor and the
  in-engine system actor keep the policy's decisions few. Sample throughput on the user's CPU and
  GTX 1070 sets how far 1v1 on small maps goes.
- **Shaping that does not anneal away** can leave a collector that does not win: the shaping is
  potential-based and goes to zero by a fixed point of the run.
- **The shipped small maps are few** (fewer than the 160 shipped maps in all) and fixed: they risk
  overfitting until generated maps (E14, E30) arrive. A held-out set from the start makes this visible.
