#!/usr/bin/env python3
"""The harness against a batch the user played (E04c T3; docs/experiments/README.md, "Agents
running a batch themselves": cross-check the first run). Batch 1a's R1 (E04a, Q23): six Archangel
stacks of 1 to 6 attack the Gold Golems once each; the user's log showed 105, 210, 315, 420, 525
and 630 damage (the x87 at 53 bits: exact). Here each attack is a click at the frozen clock on the
left, upper-left or lower-left part of a Golem stack's hex (three attacks on each of the two
stacks), then the clock runs until the next stack's turn; the damage is
the Golems' lost hit points (count x 50 - top creature's loss, structs.toml). *Provisional.*

  crosscheck_r1.py <out-dir> <R1.h3m>    (the map: h3-experiment E04a-batch-1a.toml --only R1)
"""

import json
import struct
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "clients" / "capture"))

import h3drive  # noqa: E402
import h3mem  # noqa: E402

ACTIVE_STACK = 0x132C8  # battle manager: the stack whose turn it is
GOLEM_HP = 50


def stacks(mem):
    manager, result = mem.manager(), []
    for side in (0, 1):
        for record in range(mem.i32(manager + h3mem.SIDE_COUNTS + 4 * side)):
            base = manager + h3mem.STACKS + side * h3mem.SIDE_STRIDE + record * h3mem.STACK_SIZE
            raw = mem.read(base, 0x60)
            result.append(
                {
                    "base": base,
                    "side": side,
                    "hex": struct.unpack_from("<i", raw, 0x38)[0],
                    "count": struct.unpack_from("<i", raw, 0x4C)[0],
                    "lost": struct.unpack_from("<i", raw, 0x58)[0],
                }
            )
    return result


def golem_hp(all_stacks):
    return sum(s["count"] * GOLEM_HP - s["lost"] for s in all_stacks if s["side"] == 1 and s["count"])


# Points in a hex's left, upper-left and lower-left parts: the attack comes from that neighbour.
SECTORS = [(-14, 0), (-8, -14), (-8, 14)]


def aim(hex_id, attack):
    """The k-th attack's point: a fresh neighbour for each of a target's three attacks."""
    x, y = h3drive.hex_point(hex_id)
    dx, dy = SECTORS[attack % 3]
    return x + dx, y + dy


def main(args):
    if len(args) != 2:
        sys.exit(__doc__)
    out, map_path = Path(args[0]), args[1]
    game = h3drive.Game(out)
    results = []
    try:
        game.start(map_path)
        h3drive.to_scenario(game)
        h3drive.into_battle(game)
        mem = h3mem.Mem()
        for attack in range(8):
            now = stacks(mem)
            active = mem.u32(mem.manager() + ACTIVE_STACK)
            attacker = next((s for s in now if s["base"] == active), None)
            if attacker is None or attacker["side"] != 0:
                break
            golems = [s for s in now if s["side"] == 1 and s["count"]]
            target = golems[min(attack // 3, len(golems) - 1)]
            before = golem_hp(now)
            game.move(*aim(target["hex"], attack))
            time.sleep(0.4)
            game.click(*aim(target["hex"], attack))
            game.move(420, 580)
            # Until another stack's turn has begun (the pointer is cleared while the action
            # runs), at most 8 s of game time (a good-morale turn keeps the same stack).
            bases = {s["base"] for s in now}
            for _ in range(80):
                game.clock("step 100")
                if mem.u32(mem.manager() + ACTIVE_STACK) in bases - {active}:
                    break
            after = golem_hp(stacks(mem))
            results.append({"archangels": attacker["count"], "damage": before - after})
            print(json.dumps(results[-1]), flush=True)
    finally:
        game.stop()
    (out / "r1.json").write_text(json.dumps(results) + "\n")


if __name__ == "__main__":
    main(sys.argv[1:])
