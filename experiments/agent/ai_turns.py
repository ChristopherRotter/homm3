#!/usr/bin/env python3
"""One line per stack turn of a run of experiments/agent/ai.py (E23 T7): what the evaluation of
batches 4a to 4c reads. *Provisional.*

  ai_turns.py <run folder> [--side 0|1] [--actions]

Prints the opening (each stack: side, record, creature, count, hex, morale, luck; the setup's
human bytes, heroes, difficulty, the players' gold) and then per turn: the round, the stack, its
creature, frozen (bad morale, Fear) or the actions the battle manager held while the turn was
open (named, with +0x40, +0x44, +0x48), the drawn spell of a Faerie Dragon, and what changed
(hexes, counts, spells). The turns come from the turn flags, which miss a computer's turn that
starts and ends between two reads; `--actions` lists instead every new action the battle
manager held (the timeline: its time, round, the active stack, each stack's count and hex, a
Faerie Dragon's drawn spell, the walls' hit points in a siege), the evaluation's evidence. Reads
only the run's files.
"""

import json
import sys
from pathlib import Path

ACTIONS = {1: "hero cast", 2: "move", 3: "defend", 4: "retreat", 5: "surrender",
           6: "move and attack", 7: "strike or shot", 8: "wait", 9: "wall shot",
           10: "creature cast", 11: "heal", 12: "nothing"}
TARGET_PART = [5, 6, 8, 9, 10, 12, 13, 14]
WALLS = {1: "upper", 2: "mid-upper", 3: "gate", 4: "mid-lower", 5: "lower"}


def main(args):
    if not args:
        sys.exit(__doc__)
    folder = Path(args[0])
    side = int(args[args.index("--side") + 1]) if "--side" in args else None
    opening = json.loads((folder / "opening.json").read_text())
    print("facts:", json.dumps(opening.get("facts")))
    for s in opening["stacks"]:
        print(f"  side {s['side']} record {s['record']}: creature {s['creature']} x {s['count']} "
              f"at {s['hex']}, morale {s['morale']}, luck {s['luck']}")
    if "--actions" in args:
        last = None
        for line in (folder / "timeline.jsonl").read_text().splitlines():
            r = json.loads(line)
            if r["action"][0] and r["action"] != last:
                stacks = " ".join(f"[{s['side']},{s['record']}]{s['count']}@{s['hex']}"
                                  + (f"d{s['drawn']}" if s["creature"] == 134 else "")
                                  for s in r["stacks"])
                walls = ""
                if "walls" in r:
                    walls = " walls " + " ".join(
                        f"{WALLS[t]} {r['walls'][TARGET_PART[t]]}" for t in WALLS)
                print(f"t {r['t']} round {r['round']} active {r['active']}: "
                      f"{ACTIONS.get(r['action'][0], r['action'][0])} {r['action'][1:]}; "
                      f"{stacks}{walls}")
            last = r["action"] if r["action"][0] else last
        return
    for line in (folder / "turns.jsonl").read_text().splitlines():
        t = json.loads(line)
        if side is not None and t["side"] != side:
            continue
        what = f"frozen ({t['frozen']})" if t["frozen"] else ", ".join(
            f"{ACTIONS.get(a[0], a[0])} {a[1]} {a[2]} {a[3]}" for a in t["actions"]) or "-"
        changes = []
        for c in t["changes"]:
            parts = [f"{k} {v[0]}->{v[1]}" for k, v in c.items()
                     if k in ("hex", "count") and isinstance(v, list)]
            if "spells" in c:
                added = sorted(set(c["spells"][1]) - set(c["spells"][0]))
                if added:
                    parts.append(f"spells +{','.join(added)}")
            for bit in ("waited", "defending", "dead"):
                if c.get(bit) is True:
                    parts.append(bit)
            if parts:
                changes.append(f"[{c['side']},{c['record']}] " + " ".join(parts))
        print(f"round {t['round']} [{t['side']},{t['record']}] creature {t['creature']}: {what}"
              + (f"; {'; '.join(changes)}" if changes else ""))


if __name__ == "__main__":
    main(sys.argv[1:])
