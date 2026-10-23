#!/usr/bin/env python3
"""The original's computer in a battle, played against by an agent (E23 T7, batches 4a and 4b;
docs/experiments/README.md, "Agents running a batch themselves"). *Provisional.*

  ai.py <out-dir> <recipe.toml> [<run id>...]

The recipe (experiments/agent/E23-batch-4a.toml) names the batch's spec and its runs; each run
plays one encounter of the spec on a map of its own (`h3-experiment --only <encounter>`). The
capture harness (clients/capture) starts the game under Wine in hot seat (Red and Blue human) at
the run's difficulty, steps the Red hero into its opponent (`key`, default east) and plays the
battle on the stepped clock (10 ms steps): at each turn of a stack the agent commands (a human
side's creature, no war machine) the plan's step for that stack in that round, else Defend; the
other side's stacks act on their own, as the original's computer plays them. Before the agent's
first command it lets the clock run `seed` ms (idle steps: the fidgets draw from the rules'
generator, so a run per seed gives the computer's later draws another stream). At the opening the
run's forced rolls are set (`rolls`: every draw of a roll falls or fails, the runbook's "Forced
rolls"). After `rounds` rounds, at the agent's next turn, the run's `end` (retreat) ends the
battle; a battle decided before ends the run too.

After each step the game's memory is read (docs/exe/structs.toml): the active stack, the acting
side, the battle manager's action (+0x3C the code, +0x40, +0x44, +0x48; installation.md §7.34:
1 a hero's cast, 2 a move, 3 Defend, 4 retreat, 6 a move and attack, 7 a strike or shot, 8 Wait,
9 a wall shot, 10 a creature's cast, 11 the tent's heal), the magic animation and the stacks it
shows over (30 bad morale, 15 Fear), and per stack its creature, hex, count, the top creature's
lost hit points, its flags (acted, waited, defending, dead, …), its turn flag, morale, luck and
active spells. Writes <out>/<run>/opening.json, timeline.jsonl (every change), turns.jsonl (one
line per stack's turn: who, the round, frozen or not, the actions written while it was active and
what changed) and prints one line per turn. Only reads the game; never the installation.

Needs clients/capture/setup.sh run once and `cargo build --release -p h3-experiment`.
"""

import json
import struct
import subprocess
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "clients" / "capture"))

import h3adv  # noqa: E402
import h3drive  # noqa: E402
import h3mem  # noqa: E402

# Battle manager (structs.toml, battle_manager).
ACTION = 0x3C  # i32 x 4: the action, +0x40 its spell or plan hex, +0x44 its hex, +0x48 a second
ACTIVE, ACTING_SIDE = 0x132C8, 0x132C0
HUMAN, TACTICS, HEROES = 0x54A4, 0x13D68, 0x53CC
GATE, WALLS = 0x53A4, 0x13F60  # the gate's state; the 18 parts' hit points (siege.py, catapult.py)
SIEGE = 0x132F4  # above 0 in a siege (structs.toml, siege_town's note)
WORN, WORN_SLOTS, CATAPULT_ARTIFACT = 0x12D, 19, 3
# Stack record (structs.toml, battle_stack): name -> offset (i32).
STACK = {"creature": 0x34, "hex": 0x38, "count": 0x4C, "lost": 0x58, "flags": 0x84,
         "hit_points": 0xC0, "turned": 0x4F0, "morale": 0x4E8, "luck": 0x4EC,
         "drawn": 0x4E0}  # a Faerie Dragon's drawn spell, its last activation's
CATAPULT = 145
# The wall-shot targets' parts (0x63BE60, installation.md §7.29; catapult.py) and their aim
# hexes: 1 the upper wall, 2 the mid-upper, 3 the gate, 4 the mid-lower, 5 the lower wall.
TARGET_PART = [5, 6, 8, 9, 10, 12, 13, 14]
AIM = {1: 29, 2: 62, 3: 96, 4: 130, 5: 182}
SPELL_DURATIONS, SPELLS = 0x198, 81
FLAG_BITS = {"dead": 0x200000, "summoned": 0x400000, "clone": 0x800000, "good": 0x1000000,
             "waited": 0x2000000, "acted": 0x4000000, "defending": 0x8000000}
FREEZES = {30: "bad morale", 15: "fear"}  # magic animations played over a frozen stack
MACHINES = range(145, 150)  # Catapult, Ballista, First Aid Tent, Ammo Cart, Arrow Tower
ACTIONS = {0: "none", 1: "hero cast", 2: "move", 3: "defend", 4: "retreat", 5: "surrender",
           6: "move and attack", 7: "strike or shot", 8: "wait", 9: "wall shot",
           10: "creature cast", 11: "heal", 12: "nothing"}
# The point in a hex to click for a strike from a neighbour (h3drive.hex_point + offset).
SECTOR = {"left": (-14, 0), "upper_left": (-8, -14), "lower_left": (-8, 14),
          "right": (14, 0), "upper_right": (8, -14), "lower_right": (8, 14)}
PARK = (420, 580)  # over the panel, off the battlefield


def read_state(mem):
    """The battle's state as the turns need it."""
    m = mem.manager()
    action = list(struct.unpack("<4i", mem.read(m + ACTION, 16)))
    state = {"active": h3mem.Mem.stack_at(m, mem.u32(m + ACTIVE)),
             "acting": mem.i32(m + ACTING_SIDE), "action": action,
             "magic": mem.i32(m + h3mem.MAGIC_ANIMATION), "tactics": mem.i32(m + TACTICS),
             "decided": mem.i32(m + h3mem.DECIDED) != 0, "stacks": []}
    if mem.i32(m + SIEGE) > 0:
        state["gate"] = mem.i32(m + GATE)
        state["walls"] = list(struct.unpack("<18i", mem.read(m + WALLS, 72)))
    for side in (0, 1):
        for record in range(mem.i32(m + h3mem.SIDE_COUNTS + 4 * side)):
            base = m + h3mem.STACKS + side * h3mem.SIDE_STRIDE + record * h3mem.STACK_SIZE
            raw = mem.read(base, h3mem.STACK_SIZE)
            stack = {"side": side, "record": record}
            stack.update({k: struct.unpack_from("<i", raw, o)[0] for k, o in STACK.items()})
            flags = stack.pop("flags") & 0xFFFFFFFF
            stack["bits"] = [name for name, bit in FLAG_BITS.items() if flags & bit]
            stack["hp"] = max(0, stack["count"] * stack["hit_points"] - stack["lost"])
            durations = struct.unpack_from(f"<{SPELLS}i", raw, SPELL_DURATIONS)
            stack["spells"] = {str(i): d for i, d in enumerate(durations) if d > 0}
            stack["shown"] = raw[h3mem.MAGIC_SHOWN]
            state["stacks"].append(stack)
    return state


def worn(mem):
    """Each side's hero's worn artifacts (ids; [] without a hero)."""
    m, found = mem.manager(), []
    for side in (0, 1):
        hero = mem.u32(m + HEROES + 4 * side)
        ids = [mem.i32(hero + WORN + 8 * slot) for slot in range(WORN_SLOTS)] if hero else []
        found.append([a for a in ids if a >= 0])
    return found


def opening_facts(mem):
    """The battle's setup as the computer reads it: each side's human byte and hero."""
    m = mem.manager()
    return {"human": list(mem.read(m + HUMAN, 2)),
            "heroes": [mem.u32(m + HEROES + 4 * side) != 0 for side in (0, 1)],
            "difficulty": mem.difficulty(), "worn": worn(mem)}


def commanded(mem, state, catapult=False):
    """The active stack if the agent commands it (a human side's creature, no machine; the
    Catapult too when its hero aims it, `catapult`) and it has not acted: a frozen pick never
    becomes the active stack, so the pointer still names the stack before it (E23 T7's probe)."""
    if state["active"] is None:
        return None
    side, record = state["active"]
    stack = next(s for s in state["stacks"] if s["side"] == side and s["record"] == record)
    if not mem.read(mem.manager() + HUMAN + side, 1)[0]:
        return None
    if stack["creature"] in MACHINES and not (catapult and stack["creature"] == CATAPULT):
        return None
    if "acted" in stack["bits"] and "good" not in stack["bits"]:
        return None
    return (side, record)


def key(state):
    """The state as a comparable text (a change of anything read is a change)."""
    return json.dumps(state, sort_keys=True)


class Run:
    def __init__(self, game, mem, out, catapult=False):
        self.game, self.mem, self.out, self.catapult = game, mem, out, catapult
        self.t, self.round, self.last, self.cleared = 0, 1, None, False
        self.timeline = open(out / "timeline.jsonl", "w")
        self.turns, self.turn = [], None

    def step(self, ms=10):
        self.t = self.game.clock(f"step {ms}")
        state = read_state(self.mem)
        # A round begins when every living creature's acted bit is cleared (catapult.py); the
        # machines' bits do not follow it.
        living = [s for s in state["stacks"]
                  if s["count"] > 0 and s["creature"] not in MACHINES]
        cleared = bool(living) and all("acted" not in s["bits"] for s in living)
        if cleared and not self.cleared and self.last is not None:
            self.round += 1
        self.cleared = cleared
        state["round"] = self.round
        k = key(state)
        if k != self.last:
            self.timeline.write(json.dumps({"t": self.t, **state}) + "\n")
            self.observe(state)
            self.last = k
        return state

    def observe(self, state):
        """Turns: a stack's turn starts when its turn flag is set; it was frozen when the bad
        morale or Fear animation shows over it before it acts; the actions written while it is
        the active stack are its own."""
        stacks = {(s["side"], s["record"]): s for s in state["stacks"]}
        previous = self.previous if hasattr(self, "previous") else {}
        for who, s in stacks.items():
            old = previous.get(who)
            if s["turned"] and not (old and old["turned"]) and old is not None:
                self.close_turn(state)
                self.turn = {"round": state["round"], "side": who[0], "record": who[1],
                             "creature": s["creature"], "t": self.t, "frozen": None,
                             "actions": [], "before": {k: v for k, v in stacks.items()}}
        if self.turn is not None:
            who = (self.turn["side"], self.turn["record"])
            s = stacks.get(who)
            if state["magic"] in FREEZES and s and s["shown"]:
                self.turn["frozen"] = FREEZES[state["magic"]]
            # The computer's stack turn clears the active stack before it writes its action
            # (0x4221F0): an action written while the turn is open is the turn's.
            if state["action"][0] and (
                    not self.turn["actions"] or self.turn["actions"][-1] != state["action"]):
                self.turn["actions"].append(list(state["action"]))
        self.previous = stacks

    def close_turn(self, state):
        if self.turn is None:
            return
        before = self.turn.pop("before")
        after = {(s["side"], s["record"]): s for s in state["stacks"]}
        changes = []
        for who, b in before.items():
            a = after.get(who, b)
            change = {}
            for field in ("hex", "count", "hp", "spells"):
                if a[field] != b[field]:
                    change[field] = [b[field], a[field]]
            for bit in ("waited", "defending", "dead"):
                if (bit in a["bits"]) != (bit in b["bits"]):
                    change[bit] = bit in a["bits"]
            if change:
                changes.append({"side": who[0], "record": who[1], **change})
        self.turn["changes"] = changes
        self.turn["named"] = [ACTIONS.get(a[0], str(a[0])) for a in self.turn["actions"]]
        self.turns.append(self.turn)
        print(json.dumps(self.turn), flush=True)
        self.turn = None

    def until_commanded(self, ms=10, limit_ms=300000):
        """Steps (`ms` at a time) until a stack the agent commands has its turn and the game
        rests (the state unchanged for 100 ms), or the battle is decided; returns the state."""
        same, previous = 0, None
        for _ in range(limit_ms // ms):
            state = self.step(ms)
            if state["decided"]:
                return state
            if commanded(self.mem, state, self.catapult) is not None and key(state) == previous:
                same += 1
                if same * ms >= 100:
                    return state
            else:
                same = 0
            previous = key(state)
        raise TimeoutError(f"no turn within {limit_ms} ms (round {self.round})")

    def act(self, action, argument=None, state=None):
        """Gives the active stack its command; a hex given as `{ stack = [side, record] }` is
        that stack's hex in `state`."""
        g = self.game

        def hex_of(target):
            if isinstance(target, dict):
                side, record = target["stack"]
                return next(s["hex"] for s in state["stacks"]
                            if s["side"] == side and s["record"] == record)
            return target
        if action == "wait":
            g.key("w")
        elif action == "defend":
            g.key("d")
        elif action == "retreat":
            g.key("r", 1.0)
            found = h3adv.find_button(g, "ok")
            if not found:
                raise RuntimeError("no confirmation after the retreat key")
            h3adv.click_button(g, "ok", found[0])
        elif action == "cast":  # [book place, hex or None]
            place, hex_id = argument[0], hex_of(argument[1])
            g.key("c", 1.0)
            g.click(*h3drive.book_place(place), 1.0)
            if hex_id is not None:
                x, y = h3drive.hex_point(hex_id)
                g.move(x, y)
                self.step(10)
                g.click(x, y)
        else:
            if action == "attack":
                hex_id, sector = hex_of(argument[0]), argument[1]
                x, y = h3drive.hex_point(hex_id)
                dx, dy = SECTOR[sector]
                x, y = x + dx, y + dy
            else:  # move (walk or fly), shoot: the hex itself
                x, y = h3drive.hex_point(hex_of(argument))
            g.move(x, y)
            self.step(10)  # the cursor's shape follows the mouse at the next frame
            g.click(x, y)
        g.move(*PARK)


def set_rolls(game, rolls):
    for name, how in rolls.items():
        if how == "falls":
            game.clock(f"rolls {name} {h3drive.ROLLS[name][0]}")
        elif how == "fails":
            game.rolls_fail(name)
        else:
            raise ValueError(f"roll {name}: `falls` or `fails`, not `{how}`")


def play(game, out, run):
    mem = h3mem.Mem()
    walls = run.get("catapult", [])
    r = Run(game, mem, out, bool(walls))
    opening = r.step()
    opening["facts"] = opening_facts(mem)
    with open(out / "opening.json", "w") as f:
        json.dump(opening, f)
    set_rolls(game, run.get("rolls", {}))
    plan = [dict(p, done=False) for p in run.get("plan", [])]
    first, ended = True, False
    while True:
        state = r.until_commanded(run.get("step", 10))
        if state["decided"]:
            break
        side, record = commanded(mem, state, bool(walls))
        if first and run.get("seed", 0):
            for _ in range(run["seed"] // 100):
                r.step(100)
        first = False
        if r.round > run.get("rounds", 10) and not ended:
            r.act(run.get("end", "retreat"))
            ended = True
            continue
        stack = next(s for s in state["stacks"] if s["side"] == side and s["record"] == record)
        found = next((p for p in plan if not p["done"] and p["round"] == r.round
                      and p["side"] == side and p["record"] == record), None)
        if stack["creature"] == CATAPULT:
            # The aimed Catapult: the first standing wall of the run's list, else Defend.
            standing = [t for t in walls if state["walls"][TARGET_PART[t]] > 0]
            if standing:
                r.act("move", AIM[standing[0]])
            else:
                r.act("defend")
        elif found:
            found["done"] = True
            r.act(found["action"], found.get("argument"), state)
        else:
            r.act("defend")
        # Started: the state changes within 1.5 s; else refused, and the stack defends.
        before = key(state)
        for _ in range(150):
            if key(r.step()) != before:
                break
        else:
            print(json.dumps({"refused": [side, record], "round": r.round}), flush=True)
            r.act("defend")
    for _ in range(100):
        r.step()
    r.close_turn(read_state(mem) | {"round": r.round})
    with open(out / "turns.jsonl", "w") as f:
        for t in r.turns:
            f.write(json.dumps(t) + "\n")
    with open(out / "outcome.json", "w") as f:
        json.dump(mem.battle_outcome(), f)
    game.dump(str(out / "end.xwd"))


def compile_map(spec, encounter, out):
    path = out / f"{Path(spec).stem}-{encounter}.h3m"
    subprocess.run([str(ROOT / "target" / "release" / "h3-experiment"), str(ROOT / spec),
                    "--install", str(ROOT / "original-data" / "install"), "--map", str(path),
                    "--only", encounter], check=True, capture_output=True)
    return path


def main(args):
    if len(args) < 2:
        sys.exit(__doc__)
    out, recipe = Path(args[0]), tomllib.loads(Path(args[1]).read_text())
    wanted = set(args[2:])
    for run in recipe["run"]:
        if wanted and run["id"] not in wanted:
            continue
        folder = out / run["id"]
        folder.mkdir(parents=True, exist_ok=True)
        map_path = compile_map(recipe["spec"], run.get("encounter", run["id"]), folder)
        for attempt in range(1, 9):
            game = h3drive.Game(folder / f"attempt-{attempt}")
            try:
                game.start(map_path)
                h3adv.to_hot_seat(game, ["Red", "Blue"], run.get("difficulty", "normal"))
                h3drive.into_battle(game, key=run.get("key", "Right"))
                extra = sorted(set(worn(h3mem.Mem())[0]) - {CATAPULT_ARTIFACT})
                if extra:
                    print(f"{run['id']}: attempt {attempt}: bonus artifacts {extra}",
                          file=sys.stderr, flush=True)
                    continue
                play(game, folder / f"attempt-{attempt}", run)
                print(f"{run['id']}: played (attempt {attempt})", file=sys.stderr, flush=True)
                break
            except Exception as error:  # noqa: BLE001
                print(f"{run['id']}: attempt {attempt}: {error!r}", file=sys.stderr, flush=True)
                continue
            finally:
                game.stop()


if __name__ == "__main__":
    main(sys.argv[1:])
