#!/usr/bin/env python3
"""The automatic Ballista's target played by an agent in the original (E04c T7e, batch 3h;
docs/experiments/README.md, "Agents running a batch themselves"). *Provisional.* Builds on
`turrets.py` (batch 3e/3f): the same harness and records, hot seat (Red and Blue human, so the
agent commands both armies), the clock stepped 10 ms at a time and the game's memory read after
each step; but the battle starts on the open map: the only Red hero steps east into the Blue hero
(`h3drive.into_battle`).

Every stack defends, a commanded Ballista too (the plans below; `turrets.play` would shoot with
it). Red's Ballista without Artillery acts on its own: its shot is a drop of a stack's hit points
while it is the battle's last activated stack (+0x132B8/+0x132BC), as batch 3f read the keep's.
Writes `turrets.py`'s files per map (timeline.jsonl, hits.jsonl, turns.jsonl, steps.jsonl,
summary.json) and prints one JSON line per Red Ballista shot: round, target (side, record,
creature), hit points lost. Only reads the game; never the installation.

  ballista.py <out-dir> <map.h3m>...

Needs clients/capture/setup.sh run once.
"""

import json
import sys
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import turrets  # noqa: E402
from siege import h3adv, h3drive, h3mem  # noqa: E402
import siege  # noqa: E402

BALLISTA = 146
ROUNDS = 2


def defend_machines(rounds):
    """Plan steps: every machine a side commands defends (Blue's records 0..7, both sides)."""
    return [{"side": side, "record": record, "round": r, "action": "defend", "argument": None}
            for r in range(1, rounds + 1) for side in (0, 1) for record in range(8)]


PLANS = {name: {"rounds": ROUNDS, "steps": defend_machines(ROUNDS)}
         for name in ("TI", "TU", "DE", "DN", "AR", "AN")}


def shots(folder):
    """Red's Ballista's shots from `turrets.write_summary`'s hits."""
    summary = json.loads((folder / "summary.json").read_text())
    return [{"round": h["round"], "target": h["target"], "lost": h["lost"], "count": h["count"]}
            for h in summary["hits"]
            if h["by"] and h["by"].startswith("0:") and h["by"].endswith(f":{BALLISTA}")]


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
                h3drive.into_battle(game)
                worn = siege.worn(h3mem.Mem())
                extra = [sorted(set(w) - turrets.ALLOWED_WORN) for w in worn]
                if any(extra):
                    print(f"{name}: attempt {attempt}: bonus artifacts {extra}", file=sys.stderr,
                          flush=True)
                    continue
                turrets.play(game, folder, plan)
                for shot in shots(folder):
                    print(json.dumps({"map": name, **shot}), flush=True)
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
