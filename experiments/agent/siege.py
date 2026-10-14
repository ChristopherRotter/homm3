#!/usr/bin/env python3
"""A siege played by an agent in the original (E04c T5e, batch 3c; docs/experiments/README.md,
"Agents running a batch themselves"). *Provisional.*

For each map (one encounter each, `h3-experiment E04c-batch-3c.toml --only <id>`): the capture
harness (clients/capture) starts the game under Wine in hot seat (Red and Blue human, so the
agent commands both armies), steps the only Red hero north into Blue's town and plays the
encounter's plan below, the spec's procedure as commands: at each turn of a stack the agent
commands, the plan's next step for that stack in that round (a round begins when the stacks'
turn flags +0x4F0 are cleared; the keep's arrow tower and the Catapult then act on their own),
else Defend. The clock is stepped 10 ms at a time and after each step the game's memory is read
(docs/exe/structs.toml): each stack's hex, count, the top creature's lost hit points and its turn
flag; the gate (+0x53A4), the walls' hit points (+0x13FA8), the hexes' Land Mine bits and the
battlefield's objects. Writes
<out>/<map>/timeline.jsonl (every change), steps.jsonl (each step: what was done, the state
before it and at its end) and prints one JSON line per step. Only reads the game; never the
installation.

  siege.py <out-dir> <map.h3m>...

A step's end: the next turn of a stack the agent commands, or a new round (the turn flags
cleared: the keep's and the Catapult's shots belong to no step). A click that starts nothing
within 1.5 s of game time is recorded as refused and the stack defends. A hero wearing a bonus
artifact (the random starting bonus, opening.py) plays the map again, up to eight times.

Needs clients/capture/setup.sh run once.
"""

import json
import struct
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "clients" / "capture"))

import h3adv  # noqa: E402
import h3drive  # noqa: E402
import h3mem  # noqa: E402

# Battle manager (structs.toml, battle_manager).
GATE, MOAT, HEROES, HUMAN = 0x53A4, 0x53A8, 0x53CC, 0x54A4
HEXES, HEX_SIZE, HEX_FLAGS, LAND_MINE = 0x1C4, 0x70, 0x10, 0x08
OBJECTS, OBJECT_SIZE = 0x13D58, 0x18  # vector<battle_object>: begin +0x4, end +0x8
WALLS = 0x13FA8  # the 18 parts' hit points now (+0x13F60 at the start)
ACTIVE = 0x132C8
WORN, WORN_SLOTS, CATAPULT_ARTIFACT = 0x12D, 19, 3
# Stack record (structs.toml, battle_stack): name -> offset (i32).
STACK = {"creature": 0x34, "hex": 0x38, "count": 0x4C, "lost": 0x58, "hit_points": 0xC0,
         "defense": 0xCC, "turned": 0x4F0}
CATAPULT, ARROW_TOWER = 145, 149
MACHINES = range(145, 150)  # Catapult, Ballista, First Aid Tent, Ammo Cart, Arrow Tower
# The point in a hex to click for a strike from a neighbour (h3drive.hex_point + offset).
SECTOR = {"left": (-14, 0), "upper_left": (-8, -14), "lower_left": (-8, 14),
          "right": (14, 0), "upper_right": (8, -14), "lower_right": (8, 14)}
PARK = (420, 580)  # over the panel, off the battlefield

# The plans: (round, side, record, action, argument). Records in army slot order; the spec
# (experiments/specs/E04c-batch-3c.toml) says why each step is there.
PLANS = {
    # Inferno, Citadel: Sprites 0, Magic Elementals 1, Peasants 2 | Archangel 0, Peasants 1,
    # Halflings 2.
    "M": [
        (1, 1, 0, "wait", None),
        (1, 0, 0, "move", 44),          # M1: a flyer lands in the moat
        (1, 0, 1, "move", 111),         # M2: a walker walks into the moat
        (1, 1, 0, "attack", (44, "left")),  # M3: the Archangel strikes the Sprites in the moat
        (2, 0, 0, "defend", None),      # M4: acting in place in the moat
        (2, 0, 1, "move", 146),         # M5: along the moat: around 129
        (3, 0, 1, "defend", None),      # M6: acting in place in the moat
    ],
    # Castle, Fort: Sprites 0, Titan 1, Pixies 2 | Trolls 0, Minotaur Kings 1, Peasants 2.
    "G": [
        (1, 0, 1, "shoot", 100),        # G1: through the closed gate
        (1, 0, 0, "wait", None),
        (1, 1, 1, "move", 96),          # G2: a defender walks onto the gate's hex
        (1, 1, 0, "move", 97),
        (2, 0, 1, "shoot", 97),         # G3: through the open gate (D-8)
        (2, 1, 1, "move", 76),          # G4: out of the gate
        (3, 0, 0, "move", 94),          # G5: a stack on the outer bridge
        (3, 1, 0, "move", 96),          # G6: the closed gate's passage
    ],
    # Tower, Citadel: Titans 0 | Gremlins 0.
    "T": [
        (1, 0, 0, "move", 77),          # T2: onto a mine
        (2, 0, 0, "move", 28),          # T3: over two mines onto a third
    ],
}


def read_state(mem):
    """The battle's state as the plan's steps need it."""
    m = mem.manager()
    state = {"active": None, "gate": mem.i32(m + GATE), "moat": list(mem.read(m + MOAT, 2)),
             "walls": list(struct.unpack("<18i", mem.read(m + WALLS, 72))), "stacks": []}
    pointer = mem.u32(m + ACTIVE)
    state["active"] = h3mem.Mem.stack_at(m, pointer)
    for side in (0, 1):
        for record in range(mem.i32(m + h3mem.SIDE_COUNTS + 4 * side)):
            base = m + h3mem.STACKS + side * h3mem.SIDE_STRIDE + record * h3mem.STACK_SIZE
            raw = mem.read(base, h3mem.STACK_SIZE)
            stack = {"side": side, "record": record}
            stack.update({k: struct.unpack_from("<i", raw, o)[0] for k, o in STACK.items()})
            stack["hp"] = max(0, stack["count"] * stack["hit_points"] - stack["lost"])
            state["stacks"].append(stack)
    hexes = mem.read(m + HEXES, 187 * HEX_SIZE)
    state["mine_hexes"] = [h for h in range(187) if hexes[h * HEX_SIZE + HEX_FLAGS] & LAND_MINE]
    begin, end = struct.unpack("<II", mem.read(m + OBJECTS + 4, 8))
    objects = []
    if end > begin and (end - begin) % OBJECT_SIZE == 0 and end - begin < 64 * OBJECT_SIZE:
        raw = mem.read(begin, end - begin)
        for i in range(0, len(raw), OBJECT_SIZE):
            graphic, _, anchor, owner, revealed = struct.unpack_from("<IIBbB", raw, i)
            value, rounds = struct.unpack_from("<ii", raw, i + 0xC)
            objects.append({"anchor": anchor, "owner": owner, "revealed": revealed,
                            "value": value, "rounds": rounds, "gone": graphic == 0})
    state["objects"] = objects
    return state


def worn(mem):
    """Each side's hero's worn artifacts (ids; [] without a hero)."""
    m, found = mem.manager(), []
    for side in (0, 1):
        hero = mem.u32(m + HEROES + 4 * side)
        ids = [mem.i32(hero + WORN + 8 * slot) for slot in range(WORN_SLOTS)] if hero else []
        found.append([a for a in ids if a >= 0])
    return found


def human(mem, state):
    """The active stack if the agent commands it (a human side's creature, no machine)."""
    if state["active"] is None:
        return None
    side, record = state["active"]
    stack = next(s for s in state["stacks"] if s["side"] == side and s["record"] == record)
    if stack["creature"] in MACHINES or not mem.read(mem.manager() + HUMAN + side, 1)[0]:
        return None
    return (side, record)


def key(state):
    """The state as a comparable text (a change of anything read is a change)."""
    return json.dumps(state, sort_keys=True)


class Run:
    def __init__(self, game, mem, out):
        self.game, self.mem, self.out = game, mem, out
        self.t, self.round, self.last = 0, 0, None
        self.timeline = open(out / "timeline.jsonl", "w")
        self.cleared = False

    def step(self, ms=10):
        self.t = self.game.clock(f"step {ms}")
        state = read_state(self.mem)
        # A round begins when every creature's turn flag is cleared (the machines' flags do not
        # follow it: the keep's stays set, so does a Fort's Catapult's); the keep and the
        # Catapult shoot after it.
        cleared = all(s["turned"] == 0 for s in state["stacks"]
                      if s["count"] > 0 and s["creature"] not in MACHINES)
        if cleared and not self.cleared:
            self.round += 1
        self.cleared = cleared
        state["round"] = self.round
        k = key(state)
        if k != self.last:
            self.timeline.write(json.dumps({"t": self.t, **state}) + "\n")
            self.last = k
        return state

    def until_turn(self, limit_ms=60000):
        """Steps until a stack the agent commands has its turn and the game rests (the state
        unchanged for 100 ms); returns the state."""
        same, previous = 0, None
        for _ in range(limit_ms // 10):
            state = self.step()
            if human(self.mem, state) is not None and key(state) == previous:
                same += 1
                if same >= 10:
                    return state
            else:
                same = 0
            previous = key(state)
        raise TimeoutError(f"no turn within {limit_ms} ms (round {self.round})")

    def act(self, action, argument):
        g = self.game
        if action == "wait":
            g.key("w")
        elif action == "defend":
            g.key("d")
        else:
            if action == "attack":
                hex_id, sector = argument
                x, y = h3drive.hex_point(hex_id)
                dx, dy = SECTOR[sector]
                x, y = x + dx, y + dy
            else:  # move (walk or fly), shoot: the hex itself
                x, y = h3drive.hex_point(argument)
            g.move(x, y)
            self.step(10)  # the cursor's shape follows the mouse at the next frame
            g.click(x, y)
        g.move(*PARK)


def play(game, out, plan):
    mem = h3mem.Mem()
    run = Run(game, mem, out)
    steps, todo = [], [list(p) + [False] for p in plan]
    opening = run.step()
    opening["worn"] = worn(mem)
    with open(out / "opening.json", "w") as f:
        json.dump(opening, f)
    while not all(p[-1] for p in todo):
        before = run.until_turn()
        side, record = human(mem, before)
        found = next((p for p in todo if not p[-1] and p[0] == run.round and p[1] == side
                      and p[2] == record), None)
        action, argument = (found[3], found[4]) if found else ("defend", None)
        if found:
            found[-1] = True
        round_now = run.round
        run.act(action, argument)
        # Started: the state changes within 1.5 s; else refused, and the stack defends.
        started, now = False, None
        for _ in range(150):
            now = run.step()
            if key(now) != key(before):
                started = True
                break
        step = {"round": round_now, "side": side, "record": record, "action": action,
                "argument": argument, "planned": bool(found), "refused": not started,
                "t": run.t, "before": before}
        if not started:
            run.act("defend", None)
        # The step's end: a turn the agent commands after the stack's own (a good morale's
        # too), or a new round (its turn flags cleared, before the keep and the Catapult act).
        end, gap = now, False
        for _ in range(6000):
            gap = gap or end["active"] != [side, record]
            if gap and human(mem, end) is not None:
                break
            if end["round"] != step["round"]:
                break
            end = run.step()
        step["after"] = end
        steps.append(step)
        print(json.dumps(summary(step)), flush=True)
    with open(out / "steps.jsonl", "w") as f:
        for s in steps:
            f.write(json.dumps(s) + "\n")
    game.dump(str(out / "end.xwd"))
    return steps


def summary(step):
    """A step in one line: what was done and what changed (hex, hit points; the gate)."""
    b, a = step["before"], step["after"]
    changes = []
    for sb, sa in zip(b["stacks"], a["stacks"]):
        if (sb["hex"], sb["hp"]) != (sa["hex"], sa["hp"]):
            changes.append({"side": sb["side"], "record": sb["record"], "hex": [sb["hex"], sa["hex"]],
                            "hp": [sb["hp"], sa["hp"]], "lost": sb["hp"] - sa["hp"]})
    return {"round": step["round"], "stack": [step["side"], step["record"]],
            "action": step["action"], "argument": step["argument"], "refused": step["refused"],
            "gate": [b["gate"], a["gate"]], "walls_changed": b["walls"] != a["walls"],
            "mines": [len(b["mine_hexes"]), len(a["mine_hexes"])], "changes": changes}


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
            try:
                game.start(map_path)
                h3adv.to_hot_seat(game, ["Red", "Blue"])
                h3drive.into_battle(game, key="Up")
                extra = sorted(set(worn(h3mem.Mem())[0]) - {CATAPULT_ARTIFACT})
                if extra:
                    print(f"{name}: attempt {attempt}: bonus artifacts {extra}", file=sys.stderr,
                          flush=True)
                    continue
                play(game, folder, plan)
                print(f"{name}: played (attempt {attempt})", file=sys.stderr, flush=True)
                break
            except Exception as error:  # noqa: BLE001
                print(f"{name}: attempt {attempt}: {error!r}", file=sys.stderr, flush=True)
                continue
            finally:
                game.stop()


if __name__ == "__main__":
    main(sys.argv[1:])
