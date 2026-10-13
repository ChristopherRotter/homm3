#!/usr/bin/env python3
"""The battle's opening, read from the original's memory (E04c T3; docs/experiments/README.md,
"Agents running a batch themselves"). *Provisional.*

For each map (one encounter each, `h3-experiment ... --only <id>`): the capture harness
(clients/capture: h3drive.py starts the game under Wine on the virtual clock and steps the only
hero east into its opponent) opens the battle, and this script reads, before anything moves, the
battle manager's battlefield fields, every stack's creature, count, speed, morale, luck and
native flag (docs/exe/structs.toml; 0x469750 stores morale and luck at the battle's start,
0x462E19) and the artifacts each side's hero wears. A single scenario starts with a random
starting bonus, a third of the time an artifact for the hero (seen: Necklace of Swiftness, Clover
of Fortune, and a +1 morale in one run), which changes what is read: a battle whose Red hero
wears anything beyond `--allow` (artifact ids; default 3, the Catapult every hero has) is played
again, up to eight times. One JSON line per map on stdout. Only reads the game; never the
installation.

  opening.py <out-dir> [--allow 3,46] <map.h3m>...

Needs clients/capture/setup.sh run once (the copy in original-data/wine/game, the Wine prefix).
"""

import json
import struct
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "clients" / "capture"))

import h3drive  # noqa: E402
import h3mem  # noqa: E402

# Battle manager (structs.toml, battle_manager).
TERRAIN, SPECIAL_FIELD, ANTI_MAGIC, ON_SHIP, HEROES = 0x5394, 0x53C0, 0x53C4, 0x53C6, 0x53CC
# Hero (structs.toml, hero): 19 worn artifacts of 8 bytes (id, -1 for none; then a subtype).
WORN, WORN_SLOTS = 0x12D, 19
# Stack record (structs.toml, battle_stack): name -> (offset, struct format).
STACK = {
    "creature": (0x34, "<i"),
    "count": (0x4C, "<i"),
    "speed": (0xC4, "<i"),
    "side": (0xF4, "<i"),
    "native": (0x4D8, "<B"),
    "morale": (0x4E8, "<i"),
    "luck": (0x4EC, "<i"),
}


def read_opening(mem):
    manager = mem.manager()
    raw = mem.read(manager + TERRAIN, ON_SHIP + 1 - TERRAIN)
    state = {
        "terrain": struct.unpack_from("<i", raw, 0)[0],
        "special_field": struct.unpack_from("<i", raw, SPECIAL_FIELD - TERRAIN)[0],
        "anti_magic": raw[ANTI_MAGIC - TERRAIN],
        "on_ship": raw[ON_SHIP - TERRAIN],
        "stacks": [],
        "worn": [],
    }
    for side in (0, 1):
        hero = mem.u32(manager + HEROES + 4 * side)
        ids = [mem.i32(hero + WORN + 8 * slot) for slot in range(WORN_SLOTS)] if hero else []
        state["worn"].append([a for a in ids if a >= 0])
    for side in (0, 1):
        for record in range(mem.i32(manager + h3mem.SIDE_COUNTS + 4 * side)):
            base = (
                manager + h3mem.STACKS + side * h3mem.SIDE_STRIDE + record * h3mem.STACK_SIZE
            )
            data = mem.read(base, h3mem.STACK_SIZE)
            stack = {"record": record}
            stack.update({k: struct.unpack_from(f, data, o)[0] for k, (o, f) in STACK.items()})
            state["stacks"].append(stack)
    return state


def main(args):
    if len(args) < 2:
        sys.exit(__doc__)
    out, maps, allow = Path(args[0]), args[1:], {3}
    if maps[0] == "--allow" and len(maps) > 2:
        allow, maps = {int(a) for a in maps[1].split(",")}, maps[2:]
    for map_path in maps:
        name = Path(map_path).stem
        # A start that fails (a screen dump refused) or a bonus artifact: the battle again.
        for attempt in range(1, 9):
            game = h3drive.Game(out / name)
            try:
                game.start(map_path)
                h3drive.to_scenario(game)
                opened = h3drive.into_battle(game)
                state = read_opening(h3mem.Mem())
                game.dump(str(out / name / "opening.xwd"))
            except Exception as error:  # noqa: BLE001
                print(f"{name}: attempt {attempt}: {error!r}", file=sys.stderr, flush=True)
                continue
            finally:
                game.stop()
            extra = sorted(set(state["worn"][0]) - allow)
            if extra:
                print(f"{name}: attempt {attempt}: bonus artifacts {extra}", file=sys.stderr, flush=True)
                continue
            state.update({"map": name, "opened_ms": opened, "attempt": attempt})
            print(json.dumps(state), flush=True)
            break


if __name__ == "__main__":
    main(sys.argv[1:])
