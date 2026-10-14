#!/usr/bin/env python3
"""The Catapult, Ballistics and Earthquake played by an agent in the original (E04c T6d, batch
3d; docs/experiments/README.md, "Agents running a batch themselves"). *Provisional.* Builds on
`siege.py` (batch 3c): the same harness, hot seat, the clock stepped 10 ms at a time and the
game's memory read after each step.

For each map (one encounter each, `h3-experiment E04c-batch-3d.toml --only <id>`): the agent
steps the only Red hero north into Blue's town and plays the encounter's plan: at each turn of a
stack it commands, the plan's next step for that stack whose condition holds (a round, or the
state: a wall destroyed), else Defend; under Ballistics also the Catapult's turns (it aims at the
first standing target of the step's list). Reads the wall parts' hit points (+0x13F60, 18 i32),
the gate (+0x53A4) and each stack's flags (+0x84) besides `siege.py`'s state. Writes
<out>/<map>/timeline.jsonl, steps.jsonl and walls.jsonl (each change of the parts' hit points
with the round and the stack whose turn it was) and prints one JSON line per step. Only reads
the game; never the installation.

  catapult.py <out-dir> <map.h3m>...

Needs clients/capture/setup.sh run once.
"""

import json
import struct
import sys
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import siege  # noqa: E402
from siege import h3adv, h3drive, h3mem  # noqa: E402

PARTS = 0x13F60  # the 18 parts' hit points (structs.toml; +0x13FA8 is their picture state)
FLAGS = 0x84
DEAD, ACTED = 0x200000, 0x4000000
CATAPULT = 145
# The wall-shot targets' parts (installation.md §7.29: 0x63BE60) and aim hexes.
TARGET_PART = [5, 6, 8, 9, 10, 12, 13, 14]
AIM = {1: 29, 2: 62, 3: 96, 4: 130, 5: 182}
# The spell book's first spell (the battle's book at 800 x 600, left page, first slot).
FIRST_SPELL = (243, 125)

# The plans: dicts with `side`, `record` (army slot order; "catapult" for the Catapult),
# `action`, `argument` and a condition: `round` (that round) or `when` (a state test, from that
# point on). A stack's steps are taken in order.
def mid_upper_down(state):
    return state["parts"][TARGET_PART[2]] == 0


def gate_down(state):
    return state["parts"][TARGET_PART[3]] == 0


PLANS = {
    # Castle, Citadel: Titans 0 | Peasants 0, Centaur Captains 1, Centaur Captains 2.
    "A": {
        "rounds": 16,
        "steps": [
            {"side": 1, "record": 2, "round": 1, "action": "move", "argument": 97},
            {"side": 1, "record": 1, "when": mid_upper_down, "action": "move", "argument": 62},
            {"side": 1, "record": 1, "when": mid_upper_down, "action": "move", "argument": 78},
            {"side": 1, "record": 2, "when": gate_down, "action": "move", "argument": 94},
        ],
    },
    # Castle, Fort; Advanced Ballistics: Titans 0, the Catapult | Peasants 0.
    "B": {
        "rounds": 10,
        "steps": [
            {"side": 0, "record": "catapult", "when": lambda s: True, "action": "wall",
             "argument": [1, 5, 3, 2, 4], "repeat": True},
        ],
    },
    # Castle, Citadel, Blue's hero: Titans 0 | Peasants 0, Pikemen 1.
    "E": {
        "rounds": 2,
        "steps": [
            {"side": 0, "record": 0, "round": 1, "action": "cast", "argument": None},
            {"side": 1, "record": 0, "round": 1, "action": "cast", "argument": None},
            {"side": 0, "record": 0, "round": 2, "action": "cast", "argument": None},
            {"side": 1, "record": 0, "round": 2, "action": "cast", "argument": None},
        ],
    },
}


def read_state(mem):
    state = siege.read_state(mem)
    m = mem.manager()
    state["parts"] = list(struct.unpack("<18i", mem.read(m + PARTS, 72)))
    del state["walls"]
    for s in state["stacks"]:
        base = m + h3mem.STACKS + s["side"] * h3mem.SIDE_STRIDE + s["record"] * h3mem.STACK_SIZE
        s["flags"] = mem.u32(base + FLAGS)
        s["dead"] = bool(s["flags"] & DEAD)
    return state


def record_of(state, side, record):
    if record == "catapult":
        return next((s["record"] for s in state["stacks"]
                     if s["side"] == side and s["creature"] == CATAPULT), None)
    return record


def commanded(mem, state, catapult):
    """The active stack if the agent commands it: a human side's creature, or the Catapult when
    its hero aims it (`catapult`)."""
    if state["active"] is None:
        return None
    side, record = state["active"]
    stack = next(s for s in state["stacks"] if s["side"] == side and s["record"] == record)
    if not mem.read(mem.manager() + siege.HUMAN + side, 1)[0]:
        return None
    if stack["creature"] == CATAPULT:
        return (side, record) if catapult else None
    return None if stack["creature"] in siege.MACHINES else (side, record)


class Run(siege.Run):
    def __init__(self, game, mem, out, catapult):
        super().__init__(game, mem, out)
        self.catapult = catapult
        self.walls = open(out / "walls.jsonl", "w")
        self.parts = None

    def step(self, ms=10):
        self.t = self.game.clock(f"step {ms}")
        state = read_state(self.mem)
        # A round begins when no living creature's acted bit is set (the round start clears it,
        # 0x44713F): its first stack waits for the agent with the bits clear. (`siege.py`'s turn
        # flags are set as a turn starts, so once the keep is down and the Catapult has no target
        # their clear moment lasts less than a step.)
        cleared = all(not s["flags"] & ACTED for s in state["stacks"]
                      if s["count"] > 0 and not s["dead"] and s["creature"] not in siege.MACHINES)
        if cleared and not self.cleared:
            self.round += 1
        self.cleared = cleared
        state["round"] = self.round
        if self.parts is not None and state["parts"] != self.parts:
            lost = {p: self.parts[p] - state["parts"][p] for p in range(18)
                    if self.parts[p] != state["parts"][p]}
            self.walls.write(json.dumps({"t": self.t, "round": self.round,
                                         "active": state["active"], "lost": lost,
                                         "gate": state["gate"], "parts": state["parts"]}) + "\n")
            self.walls.flush()
        self.parts = state["parts"]
        k = siege.key(state)
        if k != self.last:
            self.timeline.write(json.dumps({"t": self.t, **state}) + "\n")
            self.last = k
        return state

    def until_turn(self, limit_ms=60000):
        same, previous = 0, None
        for _ in range(limit_ms // 10):
            state = self.step()
            key = siege.key(state)
            if commanded(self.mem, state, self.catapult) is not None and key == previous:
                same += 1
                if same >= 10:
                    return state
            else:
                same = 0
            previous = key
        raise TimeoutError(f"no turn within {limit_ms} ms (round {self.round})")

    def cast(self):
        g = self.game
        g.key("c")
        for _ in range(50):
            self.step(10)
        g.move(*FIRST_SPELL)
        self.step(10)
        g.click(*FIRST_SPELL)
        g.move(*siege.PARK)


def into_battle(game):
    """`h3drive.into_battle` without the probes on the spell book's button (a hero with a book
    shows it lit)."""
    game.clock("freeze")
    game.key("Up")
    game.move(*siege.PARK)
    probes = [p for p in game.screens["battle"]["probes"] if not 630 <= p[0] <= 700]
    for _ in range(2000):
        game.clock("step 10")
        pix = game.settled()
        if all(int(pix[y, x]) == v for x, y, v in probes):
            return
    raise TimeoutError("no battle screen")


def due(step, state, round_now):
    if "round" in step:
        return step["round"] == round_now
    return step["when"](state)


def play(game, out, plan, catapult):
    mem = h3mem.Mem()
    run = Run(game, mem, out, catapult)
    steps = []
    todo = [dict(s, done=False) for s in plan["steps"]]
    opening = run.step()
    opening["worn"] = siege.worn(mem)
    with open(out / "opening.json", "w") as f:
        json.dump(opening, f)
    while True:
        before = run.until_turn()
        if run.round > plan["rounds"]:
            break
        side, record = commanded(mem, before, catapult)
        found = next((s for s in todo if not s["done"] and s["side"] == side
                      and record_of(before, side, s["record"]) == record
                      and due(s, before, run.round)), None)
        action, argument = (found["action"], found["argument"]) if found else ("defend", None)
        if found and not found.get("repeat"):
            found["done"] = True
        round_now = run.round
        aimed = None
        if action == "wall":
            standing = [t for t in argument if before["parts"][TARGET_PART[t]] > 0]
            aimed = standing[0] if standing else None
            if aimed is None:
                action = "defend"
            else:
                run.act("move", AIM[aimed])
        elif action == "cast":
            run.cast()
        else:
            run.act(action, argument)
        started, now = False, None
        for _ in range(150):
            now = run.step()
            if siege.key(now) != siege.key(before):
                started = True
                break
        step = {"round": round_now, "side": side, "record": record, "action": action,
                "argument": argument, "aimed": aimed, "planned": bool(found),
                "refused": not started, "t": run.t, "before": before}
        if not started:
            game.key("Escape")
            run.act("defend", None)
        end, gap = now, False
        for _ in range(6000):
            gap = gap or end["active"] != [side, record]
            if gap and commanded(mem, end, catapult) is not None:
                break
            if end["round"] != step["round"]:
                break
            end = run.step()
        step["after"] = end
        steps.append(step)
        summary = siege.summary({**step, "before": dict(before, walls=before["parts"]),
                                 "after": dict(end, walls=end["parts"])})
        summary["aimed"] = aimed
        summary["parts"] = {p: [before["parts"][p], end["parts"][p]] for p in range(18)
                            if before["parts"][p] != end["parts"][p]}
        summary["dead"] = [[s["side"], s["record"]] for s in end["stacks"] if s["dead"]]
        print(json.dumps(summary), flush=True)
    with open(out / "steps.jsonl", "w") as f:
        for s in steps:
            f.write(json.dumps(s, default=str) + "\n")
    game.dump(str(out / "end.xwd"))
    return steps


def main(args):
    if len(args) < 2:
        sys.exit(__doc__)
    out = Path(args[0])
    for map_path in args[1:]:
        name = Path(map_path).stem.rsplit("-", 1)[-1]
        plan = PLANS[name]
        catapult = any(s["record"] == "catapult" for s in plan["steps"])
        for attempt in range(1, 9):
            folder = out / name / f"attempt-{attempt}"
            folder.mkdir(parents=True, exist_ok=True)
            game = h3drive.Game(folder)
            try:
                game.start(map_path)
                h3adv.to_hot_seat(game, ["Red", "Blue"])
                into_battle(game)
                extra = sorted(set(siege.worn(h3mem.Mem())[0]) - {siege.CATAPULT_ARTIFACT, 0})
                if extra:
                    print(f"{name}: attempt {attempt}: bonus artifacts {extra}", file=sys.stderr,
                          flush=True)
                    continue
                play(game, folder, plan, catapult)
                print(f"{name}: played (attempt {attempt})", file=sys.stderr, flush=True)
                break
            except Exception as error:  # noqa: BLE001
                print(f"{name}: attempt {attempt}: {error!r}", file=sys.stderr, flush=True)
                time.sleep(1)
                continue
            finally:
                game.stop()


if __name__ == "__main__":
    main(sys.argv[1:])
