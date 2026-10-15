#!/usr/bin/env python3
"""The siege's other rules played by an agent in the original (E04c T8, batch 3g;
docs/experiments/README.md, "Agents running a batch themselves"). *Provisional.* Builds on
`catapult.py` (batch 3d) and `siege.py` (3c): the same harness, hot seat (Red and Blue human, so
the agent commands both armies), the clock stepped 10 ms at a time and the game's memory read
after each step; rounds counted by the living creatures' acted bits.

For each map (one encounter each, `h3-experiment E04c-batch-3g.toml --only <id>`): the agent
steps the only Red hero north into Blue's town, reads the opening (each stack's morale +0x4E8 and
luck +0x4EC, set by 0x469750; the battle manager's town +0x53C8, fort level +0x132F4, moat flag
+0x53A8 and tactics flag +0x13D68; the town record's built sets +0x150 and +0x158; each hero's
secondary skills +0xC9 and morale and luck bonuses +0x11A, +0x11B) and plays the plan: at a planned turn the side's hero casts the
first spell of its book (`c`, then the book's first slot), clicks the stack's own hex if the plan
says so (Teleport's first target), then tries the plan's hexes in order until one starts
something within 1.5 s of game time (a refused click changes nothing read); with none accepted
the cast is cancelled (Escape). Every other turn defends.

At every turn the agent commands it also reads the battle window's surrender (2001) and retreat
(2002) buttons: the window is the battle manager's +0x132FC, its items a list from +0x2C (next
+0x8, id an `i16` at +0x10); each button's bytes +0x10 to +0x40 are written as they are, for the
state 0x1000 that 0x4782D0 adds to a disabled button (H3API's AddControlState).

Writes <out>/<map>/timeline.jsonl (every change), opening.json, controls.jsonl (the buttons at
every commanded turn), steps.jsonl (the agent's steps, each tried hex with its outcome) and
summary.json, and prints one JSON line per step. Only reads the game; never the installation.

  town.py <out-dir> <map.h3m>...

Needs clients/capture/setup.sh run once.
"""

import json
import struct
import sys
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import catapult  # noqa: E402
import siege  # noqa: E402
from siege import h3adv, h3drive, h3mem  # noqa: E402

MORALE, LUCK = 0x4E8, 0x4EC
TOWN, FORT, MOAT, TACTICS = 0x53C8, 0x132F4, 0x53A8, 0x13D68
WINDOW, FIRST_ITEM, NEXT_ITEM, ITEM_ID = 0x132FC, 0x2C, 0x8, 0x10
SURRENDER, RETREAT = 2001, 2002
ALLOWED_WORN = {0, 3, 4, 5, 6}  # spell book, the war machines

# The plans: `rounds` to play and the casts: `round`, `side`, `record` (army slot order), whether
# to click the stack's own hex first (`stack`) and the hexes to try in order (`try`).
TRY = [64, 61, 91]  # inside the walls, the moat hex of row 3, outside
PLANS = {
    "CM": {"rounds": 2, "casts": []},
    "ET": {"rounds": 1, "casts": []},
    "RL": {"rounds": 1, "casts": []},
    "TB": {"rounds": 2, "casts": [{"round": 1, "side": 0, "record": 0, "stack": True, "try": TRY}]},
    "TV": {"rounds": 2, "casts": [{"round": 1, "side": 0, "record": 0, "stack": True, "try": TRY}]},
    "TE": {"rounds": 2, "casts": [{"round": 1, "side": 0, "record": 0, "stack": True, "try": TRY}]},
    "FF": {"rounds": 1, "casts": [{"round": 1, "side": 0, "record": 0, "stack": False,
                                   "try": [95, 96, 91]}]},
}


def read_state(mem):
    state = catapult.read_state(mem)
    m = mem.manager()
    for s in state["stacks"]:
        base = m + h3mem.STACKS + s["side"] * h3mem.SIDE_STRIDE + s["record"] * h3mem.STACK_SIZE
        s["morale"] = mem.i32(base + MORALE)
        s["luck"] = mem.i32(base + LUCK)
    return state


def controls(mem):
    """The surrender and retreat buttons' bytes +0x10 to +0x40 (hex), by id."""
    window = mem.u32(mem.manager() + WINDOW)
    found = {}
    item = mem.u32(window + FIRST_ITEM) if window else 0
    for _ in range(400):
        if not item:
            break
        raw = mem.read(item, 0x40)
        ident = struct.unpack_from("<h", raw, ITEM_ID)[0]
        if ident in (SURRENDER, RETREAT):
            found[ident] = raw[0x10:0x40].hex()
        item = struct.unpack_from("<I", raw, NEXT_ITEM)[0]
    return found


def stack(state, side, record):
    return next(s for s in state["stacks"] if s["side"] == side and s["record"] == record)


class Run(catapult.Run):
    def __init__(self, game, mem, out):
        super().__init__(game, mem, out, False)
        self.controls = open(out / "controls.jsonl", "w")

    def step(self, ms=10):
        self.t = self.game.clock(f"step {ms}")
        state = read_state(self.mem)
        cleared = all(not s["flags"] & catapult.ACTED for s in state["stacks"]
                      if s["count"] > 0 and not s["dead"] and s["creature"] not in siege.MACHINES)
        if cleared and not self.cleared:
            self.round += 1
        self.cleared = cleared
        state["round"] = self.round
        k = siege.key(state)
        if k != self.last:
            self.timeline.write(json.dumps({"t": self.t, **state}) + "\n")
            self.last = k
        return state

    def click(self, hex_id):
        x, y = h3drive.hex_point(hex_id)
        self.game.move(x, y)
        self.step(10)  # the cursor's shape follows the mouse at the next frame
        self.game.click(x, y)
        self.game.move(*siege.PARK)

    def cast(self, plan, own_hex):
        """The hero's first spell, its own stack first if `plan["stack"]`, then each hex of
        `plan["try"]` until one starts something; returns the outcomes."""
        g = self.game
        g.key("c")
        for _ in range(50):
            self.step(10)
        g.move(*catapult.FIRST_SPELL)
        self.step(10)
        g.click(*catapult.FIRST_SPELL)
        g.move(*siege.PARK)
        for _ in range(50):
            self.step(10)
        if plan["stack"]:
            self.click(own_hex)
            for _ in range(50):
                self.step(10)
        tried = []
        for hex_id in plan["try"]:
            before = self.step(10)
            self.click(hex_id)
            started = False
            for _ in range(150):
                now = self.step(10)
                if siege.key(now) != siege.key(before):
                    started = True
                    break
            tried.append({"hex": hex_id, "accepted": started})
            if started:
                break
        if not any(t["accepted"] for t in tried):
            g.key("Escape")
            for _ in range(50):
                self.step(10)
        return tried


def play(game, out, plan):
    mem = h3mem.Mem()
    run = Run(game, mem, out)
    m = mem.manager()
    opening = run.step()
    opening["worn"] = siege.worn(mem)
    opening["town"] = mem.u32(m + TOWN) != 0
    opening["fort"] = mem.i32(m + FORT)
    opening["moat_flag"] = mem.read(m + MOAT, 1)[0]
    opening["tactics"] = mem.read(m + TACTICS, 1)[0]
    # The town record's built sets (+0x150: an upgrade's predecessor left out; +0x158: every
    # built one; batch 3g CM, D-21) and the heroes' skills and morale and luck bonuses.
    town = mem.u32(m + TOWN)
    if town:
        opening["town_sets"] = [mem.read(town + o, 8).hex() for o in (0x150, 0x158)]
    opening["heroes"] = []
    for side in (0, 1):
        hero = mem.u32(m + siege.HEROES + 4 * side)
        opening["heroes"].append(None if not hero else {
            "skills": list(mem.read(hero + 0xC9, 28)),
            "morale_bonus": struct.unpack("<b", mem.read(hero + 0x11A, 1))[0],
            "luck_bonus": struct.unpack("<b", mem.read(hero + 0x11B, 1))[0]})
    with open(out / "opening.json", "w") as f:
        json.dump(opening, f)
    todo = [dict(c, done=False) for c in plan["casts"]]
    steps, seen = [], []
    while True:
        try:
            before = run.until_turn()
        except TimeoutError:
            # The battle ended (TV: the Swordsmen die in the moat in round 3): the steps so far.
            if not all(c["done"] for c in todo):
                raise
            break
        if run.round > plan["rounds"]:
            break
        side, record = catapult.commanded(mem, before, False)
        buttons = {"round": run.round, "side": side, "record": record, "t": run.t,
                   "buttons": controls(mem)}
        run.controls.write(json.dumps(buttons) + "\n")
        run.controls.flush()
        seen.append(buttons)
        found = next((c for c in todo if not c["done"] and c["side"] == side
                      and c["record"] == record and c["round"] == run.round), None)
        step = {"round": run.round, "side": side, "record": record, "t": run.t,
                "before": before}
        if found:
            found["done"] = True
            step["action"] = "cast"
            step["tried"] = run.cast(found, stack(before, side, record)["hex"])
            # The cast leaves the stack its turn: wait for it to rest again.
            after = run.until_turn()
        else:
            step["action"] = "defend"
            run.act("defend", None)
            after = before
            for _ in range(300):
                after = run.step()
                if siege.key(after) != siege.key(before):
                    break
        step["after"] = after
        steps.append(step)
        print(json.dumps({"round": step["round"], "stack": [side, record],
                          "action": step["action"], "tried": step.get("tried"),
                          "buttons": buttons["buttons"],
                          "changes": siege.summary({**step, "action": step["action"],
                                                    "argument": None, "refused": False,
                                                    "before": dict(before, walls=before["parts"]),
                                                    "after": dict(after, walls=after["parts"])}
                                                   )["changes"]}), flush=True)
    with open(out / "steps.jsonl", "w") as f:
        for s in steps:
            f.write(json.dumps(s) + "\n")
    game.dump(str(out / "end.xwd"))
    with open(out / "summary.json", "w") as f:
        json.dump({"worn": opening["worn"], "town": opening["town"], "fort": opening["fort"],
                   "town_sets": opening.get("town_sets"), "heroes": opening["heroes"],
                   "moat_flag": opening["moat_flag"], "tactics": opening["tactics"],
                   "stacks": [{k: s[k] for k in ("side", "record", "creature", "hex", "count",
                                                 "hp", "morale", "luck")}
                              for s in opening["stacks"]],
                   "buttons": seen,
                   "casts": [{"round": s["round"], "stack": [s["side"], s["record"]],
                              "tried": s["tried"],
                              "hex": [stack(s["before"], s["side"], s["record"])["hex"],
                                      stack(s["after"], s["side"], s["record"])["hex"]],
                              "hp": [stack(s["before"], s["side"], s["record"])["hp"],
                                     stack(s["after"], s["side"], s["record"])["hp"]],
                              "objects": [len(s["before"]["objects"]),
                                          len(s["after"]["objects"])]}
                             for s in steps if s["action"] == "cast"],
                   "hp_by_step": [{"round": s["round"], "stack": [s["side"], s["record"]],
                                   "action": s["action"],
                                   "hp": [[x["hp"] for x in s["before"]["stacks"]],
                                          [x["hp"] for x in s["after"]["stacks"]]]}
                                  for s in steps]},
                  f, indent=1)
    return steps


def main(args):
    if len(args) < 2:
        sys.exit(__doc__)
    out = Path(args[0])
    for map_path in args[1:]:
        name = Path(map_path).stem.rsplit("-", 1)[-1]
        plan = PLANS[name]
        for attempt in range(1, 9):
            folder = out / name / f"attempt-{attempt}"
            folder.mkdir(parents=True, exist_ok=True)
            game = h3drive.Game(folder)
            started = time.monotonic()
            try:
                game.start(map_path)
                h3adv.to_hot_seat(game, ["Red", "Blue"])
                catapult.into_battle(game)
                worn = siege.worn(h3mem.Mem())
                extra = [sorted(set(w) - ALLOWED_WORN) for w in worn]
                if any(extra):
                    print(f"{name}: attempt {attempt}: bonus artifacts {extra}", file=sys.stderr,
                          flush=True)
                    continue
                play(game, folder, plan)
                print(f"{name}: played (attempt {attempt}, {time.monotonic() - started:.0f} s)",
                      file=sys.stderr, flush=True)
                break
            except Exception as error:  # noqa: BLE001
                print(f"{name}: attempt {attempt}: {error!r}", file=sys.stderr, flush=True)
                time.sleep(1)
                continue
            finally:
                game.stop()


if __name__ == "__main__":
    main(sys.argv[1:])
