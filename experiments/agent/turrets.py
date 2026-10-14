#!/usr/bin/env python3
"""The arrow towers played by an agent in the original (E04c T7c, batch 3e; docs/experiments/
README.md, "Agents running a batch themselves"). *Provisional.* Builds on `catapult.py` (batch
3d) and `siege.py` (3c): the same harness, hot seat (Red and Blue human, so the agent commands
both armies), the clock stepped 10 ms at a time and the game's memory read after each step.

Batch 3f (E04c T7d, the automatic keep's target) runs on the same plans: every stack defends.

For each map (one encounter each, `h3-experiment E04c-batch-3e.toml --only <id>`): the agent
steps the only Red hero north into Blue's town and plays the encounter's plan: at each turn of a
stack it commands, the plan's next step for that stack in that round, else Defend. Rounds are
counted as in `catapult.py` (the living creatures' acted bits, cleared at a round's start). The
turrets (creature 149: the keep at 254, the upper tower at 255, the lower at 251) act on their
own unless the game waits for a command: a turret of a human side that is the active stack while
nothing read changes for 3 s of game time is taken as waiting (Artillery), and the plan's step for
it (or a shot at the first enemy creature) is played; a click is aimed at the target's hex, a Wait
is the key `w` (the panel's Wait button at 720, 579 if the key starts nothing).

Besides `catapult.py`'s state it reads each stack's shots (+0xD8, the creature record's copy),
Air Shield's rounds left (+0x208: +0x198 + 4 x spell 28) and the battle manager's last activated
stack (+0x132B8 side, +0x132BC record). Writes <out>/<map>/timeline.jsonl (every change),
hits.jsonl (every change of a stack's hit points, with the round, the active and the last
activated stack), turns.jsonl (every change of the active stack), steps.jsonl (the agent's
steps) and summary.json (the turrets' records and each turret's shot: round, turret, order
among the round's activations, target, hit points lost, Air Shield on the target), and prints
one JSON line per step. Only reads the game; never the installation.

  turrets.py <out-dir> <map.h3m>...

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

TURRET = 149
TURRET_NAMES = {254: "keep", 255: "upper tower", 251: "lower tower"}
SHOTS = 0xD8  # creature_info (+0x74) + the record's shots (+0x64)
AIR_SHIELD = 0x198 + 4 * 28
LAST_SIDE, LAST_RECORD = 0x132B8, 0x132BC
WAIT_BUTTON = (720, 579)  # the battle panel's Wait (696..743 x 561..597)
TURRET_REST = 300  # steps of 10 ms: a turret resting this long waits for a command
ALLOWED_WORN = {0, 3, 4, 5, 6}  # spell book, the war machines

# The plans: dicts with `side`, `record` ("keep" for the defender's keep), `round`, `action`
# (cast, shoot, wait, defend) and `argument` ("titans": the attacker's first creature stack).
PLANS = {
    "K": {"rounds": 8, "steps": []},
    "AS": {"rounds": 6, "steps": [
        {"side": 0, "record": 0, "round": 1, "action": "cast", "argument": None},
    ]},
    "AR": {"rounds": 10, "steps": []},
    "AC": {"rounds": 10, "steps": []},
    "AT": {"rounds": 4, "steps": [
        {"side": 1, "record": "keep", "round": 1, "action": "shoot", "argument": "titans"},
        {"side": 1, "record": "keep", "round": 2, "action": "wait", "argument": None},
        {"side": 1, "record": "keep", "round": 2, "action": "shoot", "argument": "titans"},
        {"side": 1, "record": "keep", "round": 3, "action": "shoot", "argument": "titans"},
        {"side": 1, "record": "keep", "round": 4, "action": "shoot", "argument": "titans"},
    ]},
    "AI": {"rounds": 3, "steps": []},
    # Batch 3f (E04c T7d): the automatic keep's target; every stack defends.
    "FV": {"rounds": 2, "steps": []},
    "TU": {"rounds": 2, "steps": []},
    "DE": {"rounds": 2, "steps": []},
    "DN": {"rounds": 2, "steps": []},
    "BA": {"rounds": 2, "steps": []},
}


def read_state(mem):
    state = catapult.read_state(mem)
    m = mem.manager()
    state["last"] = [mem.i32(m + LAST_SIDE), mem.i32(m + LAST_RECORD)]
    for s in state["stacks"]:
        base = m + h3mem.STACKS + s["side"] * h3mem.SIDE_STRIDE + s["record"] * h3mem.STACK_SIZE
        s["shots"] = mem.i32(base + SHOTS)
        s["air_shield"] = mem.i32(base + AIR_SHIELD)
    return state


def stack(state, side, record):
    return next(s for s in state["stacks"] if s["side"] == side and s["record"] == record)


def record_of(state, side, record):
    if record == "keep":
        return next((s["record"] for s in state["stacks"]
                     if s["side"] == side and s["creature"] == TURRET and s["hex"] == 254), None)
    return record


def human(mem, side):
    return bool(mem.read(mem.manager() + siege.HUMAN + side, 1)[0])


def valid(state, pair):
    return any(s["side"] == pair[0] and s["record"] == pair[1] for s in state["stacks"])


def first_enemy(state, side):
    return next(s for s in state["stacks"] if s["side"] != side and s["count"] > 0
                and not s["dead"] and s["creature"] not in siege.MACHINES)


class Run(siege.Run):
    def __init__(self, game, mem, out):
        super().__init__(game, mem, out)
        self.hits = open(out / "hits.jsonl", "w")
        self.turns_file = open(out / "turns.jsonl", "w")
        self.walls = open(out / "walls.jsonl", "w")
        self.previous = None
        self.turns = []  # every activation: round, side, record, creature, hex, t
        self.hit_list = []
        self.rest = 0  # steps the state has not changed

    def step(self, ms=10):
        self.t = self.game.clock(f"step {ms}")
        state = read_state(self.mem)
        cleared = all(not s["flags"] & catapult.ACTED for s in state["stacks"]
                      if s["count"] > 0 and not s["dead"] and s["creature"] not in siege.MACHINES)
        if cleared and not self.cleared:
            self.round += 1
        self.cleared = cleared
        state["round"] = self.round
        p = self.previous
        # An activation: a change of the last activated stack (+0x132B8/+0x132BC; the active
        # stack pointer +0x132C8 is set only for a moment for an automatic turret, never seen
        # for the keep).
        # The same stack activated again in a new round (a commanded keep that acted last
        # in the round before) shows only as the pointer set to it again.
        again = (p is not None and self.turns and state["active"] is not None
                 and p["active"] != state["active"] and state["active"] == state["last"]
                 and self.turns[-1]["round"] != self.round)
        if valid(state, state["last"]) and (p is None or p["last"] != state["last"] or again):
            a = stack(state, *state["last"])
            turn = {"t": self.t, "round": self.round, "side": a["side"], "record": a["record"],
                    "creature": a["creature"], "hex": a["hex"], "pointer": state["active"]}
            self.turns.append(turn)
            self.turns_file.write(json.dumps(turn) + "\n")
            self.turns_file.flush()
        if p is not None:
            for before, after in zip(p["stacks"], state["stacks"]):
                if (before["hp"], before["count"]) != (after["hp"], after["count"]):
                    hit = {"t": self.t, "round": self.round, "active": state["active"],
                           "last": state["last"], "side": after["side"],
                           "record": after["record"], "creature": after["creature"],
                           "hex": after["hex"], "hp": [before["hp"], after["hp"]],
                           "lost": before["hp"] - after["hp"],
                           "count": [before["count"], after["count"]],
                           "air_shield": [before["air_shield"], after["air_shield"]],
                           "turn_index": len(self.turns) - 1}
                    self.hit_list.append(hit)
                    self.hits.write(json.dumps(hit) + "\n")
                    self.hits.flush()
            if p["parts"] != state["parts"]:
                lost = {q: p["parts"][q] - state["parts"][q] for q in range(18)
                        if p["parts"][q] != state["parts"][q]}
                self.walls.write(json.dumps({"t": self.t, "round": self.round,
                                             "active": state["active"], "lost": lost}) + "\n")
                self.walls.flush()
        k = siege.key(state)
        if k != self.last:
            self.timeline.write(json.dumps({"t": self.t, **state}) + "\n")
            self.last = k
            self.rest = 0
        else:
            self.rest += 1
        self.previous = state
        return state

    def commanded(self, state):
        """The active stack if the agent commands it now: a human side's creature once the game
        rests 100 ms, a human side's turret (or other machine but the Catapult) once it rests
        3 s (waiting for a command)."""
        acting = state["active"]
        if acting is None and valid(state, state["last"]):
            last = stack(state, *state["last"])
            if last["creature"] in siege.MACHINES:
                acting = state["last"]  # a machine waiting with the pointer unset
        if acting is None:
            return None
        side, record = acting
        if not human(self.mem, side):
            return None
        creature = stack(state, side, record)["creature"]
        if creature == catapult.CATAPULT:
            return None
        if creature in siege.MACHINES:
            return (side, record) if self.rest >= TURRET_REST else None
        return (side, record) if self.rest >= 10 else None

    def until_turn(self, limit_ms=60000):
        for _ in range(limit_ms // 10):
            state = self.step()
            if self.commanded(state) is not None:
                return state
        raise TimeoutError(f"no turn within {limit_ms} ms (round {self.round})")

    def cast(self, target_hex):
        """The hero's first spell; a target click on `target_hex` if the cast waits for one."""
        g = self.game
        g.key("c")
        for _ in range(50):
            self.step(10)
        g.move(*catapult.FIRST_SPELL)
        self.step(10)
        g.click(*catapult.FIRST_SPELL)
        g.move(*siege.PARK)
        for _ in range(50):
            state = self.step(10)
        titans = stack(state, 0, 0)
        if titans["air_shield"] <= 0:
            x, y = h3drive.hex_point(target_hex)
            g.move(x, y)
            self.step(10)
            g.click(x, y)
            g.move(*siege.PARK)
            return "target clicked"
        return "mass"


def play(game, out, plan):
    mem = h3mem.Mem()
    run = Run(game, mem, out)
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
        side, record = run.commanded(before)
        active = stack(before, side, record)
        is_machine = active["creature"] in siege.MACHINES
        waited_ms = run.rest * 10
        found = next((s for s in todo if not s["done"] and s["side"] == side
                      and record_of(before, side, s["record"]) == record
                      and s["round"] == run.round), None)
        if found:
            action, argument = found["action"], found["argument"]
            found["done"] = True
        elif is_machine:
            action, argument = "shoot", "titans"
        else:
            action, argument = "defend", None
        round_now = run.round
        note = None
        target = None
        hits_from = len(run.hit_list)
        if action == "shoot":
            target = first_enemy(before, side)
            run.act("move", target["hex"])  # a click on the target's hex
        elif action == "cast":
            note = run.cast(stack(before, side, record)["hex"])
        else:
            run.act(action, argument)
        started, now = False, None
        for _ in range(500 if is_machine else 150):
            now = run.step()
            if siege.key(now) != siege.key(before):
                started = True
                break
        if not started and action == "wait":
            game.click(*WAIT_BUTTON)
            game.move(*siege.PARK)
            note = "key w started nothing: the panel's Wait button clicked"
            for _ in range(500):
                now = run.step()
                if siege.key(now) != siege.key(before):
                    started = True
                    break
        step = {"round": round_now, "side": side, "record": record,
                "creature": active["creature"], "action": action, "argument": argument,
                "target": [target["side"], target["record"]] if target else None,
                "planned": bool(found), "refused": not started, "waited_ms": waited_ms,
                "note": note, "t": run.t, "before": before}
        if not started:
            game.key("Escape")
            if is_machine:
                target = first_enemy(before, side)
                run.act("move", target["hex"])
            else:
                run.act("defend", None)
        # The step's end: the active stack changes, or (a hero's cast) the stack rests again.
        end = now
        for _ in range(6000):
            if end["active"] not in ([side, record], None) or end["round"] != round_now:
                break
            if end["active"] is None and end["last"] != [side, record]:
                break
            if action == "cast" and run.rest >= 10:
                break
            end = run.step()
        step["after"] = end
        step["hits_in_turn"] = run.hit_list[hits_from:]
        steps.append(step)
        summary = siege.summary({**step, "before": dict(before, walls=before["parts"]),
                                 "after": dict(end, walls=end["parts"])})
        summary["creature"] = active["creature"]
        summary["waited_ms"] = waited_ms
        summary["note"] = note
        print(json.dumps(summary), flush=True)
    with open(out / "steps.jsonl", "w") as f:
        for s in steps:
            f.write(json.dumps(s, default=str) + "\n")
    game.dump(str(out / "end.xwd"))
    write_summary(out, opening, run, steps)
    return steps


def write_summary(out, opening, run, steps):
    turrets = [{"side": s["side"], "record": s["record"], "hex": s["hex"], "count": s["count"],
                "shots": s["shots"], "name": TURRET_NAMES.get(s["hex"])}
               for s in opening["stacks"] if s["creature"] == TURRET]
    army = [{"side": s["side"], "record": s["record"], "creature": s["creature"],
             "hex": s["hex"], "count": s["count"], "hp": s["hp"]}
            for s in opening["stacks"] if s["creature"] != TURRET]
    order = {}
    for turn in run.turns:
        order.setdefault(turn["round"], []).append(
            TURRET_NAMES.get(turn["hex"], "?") if turn["creature"] == TURRET
            else f"{turn['side']}:{turn['record']}:{turn['creature']}")
    shots = []
    for hit in run.hit_list:
        turn = run.turns[hit["turn_index"]] if hit["turn_index"] >= 0 else None
        by = bool(turn) and turn["creature"] == TURRET
        index = None
        if turn:
            same_round = [t for t in run.turns if t["round"] == turn["round"]]
            index = same_round.index(turn) + 1
        shots.append({"round": hit["round"],
                      "by": TURRET_NAMES.get(turn["hex"]) if by else (
                          f"{turn['side']}:{turn['record']}:{turn['creature']}" if turn else None),
                      "turret": bool(by), "order": index, "last": hit["last"],
                      "target": [hit["side"], hit["record"], hit["creature"]],
                      "lost": hit["lost"], "hp": hit["hp"], "count": hit["count"],
                      "air_shield": hit["air_shield"]})
    end = run.previous
    with open(out / "summary.json", "w") as f:
        json.dump({"worn": opening["worn"], "turrets": turrets, "army": army,
                   "turret_shots_end": [s["shots"] for s in end["stacks"]
                                        if s["creature"] == TURRET],
                   "turret_counts_end": [s["count"] for s in end["stacks"]
                                         if s["creature"] == TURRET],
                   "order": order, "hits": shots,
                   "machine_steps": [{k: s[k] for k in ("round", "record", "creature", "action",
                                                        "refused", "waited_ms", "note")}
                                     for s in steps if s["creature"] in siege.MACHINES]},
                  f, indent=1)


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
