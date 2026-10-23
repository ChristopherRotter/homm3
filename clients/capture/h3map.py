#!/usr/bin/env python3
"""The adventure map's stills (E24 M1 T3b2, *provisional*; clients/capture/README.md, "The map's
stills"): a situation (clients/map/situations/) shown in the original and drawn by the viewer,
compared pixel for pixel outside the masks of what the viewer does not draw yet.

  h3map.py still <situation.toml> [--out <dir>] [--keep]
      builds the situation's map (`h3-map-view situation`), loads it in the original as a single
      scenario (Red human), writes Red's explored tiles as the situation gives them (every tile
      but its `hidden` rectangles), centres the view by the minimap so that its top-left is the
      situation's `view`, parks the mouse, freezes the clock, steps it to the next moment both
      the palette tick and the view's redraw come (so the screen shows the palettes memory
      holds) and dumps the screen (`still.xwd`);
      reads the palette cycles' step k from the cycling DEFs' palettes in memory and writes it
      with the view, the objects' frame step the screen shows and every object's anchor and
      phase (`live.toml`); draws ours (`h3-map-view render --bugs original --live live.toml`,
      `ours.png`) and prints the pixels that differ outside the masks (`diff.png`: differing
      pixels red, masked ones blue). <dir> defaults to clients/out/map/<situation>.
  h3map.py set [--only <id>] [--out <dir>]
      every situation of clients/map/situations/set.toml (or one), each count against the one
      recorded; exit 1 if any changed.

Needs `clients/capture/setup.sh` and `cargo build --release -p h3-map-view` (from clients/).
"""

import struct
import subprocess
import sys
import time
import tomllib
import zlib
from pathlib import Path

import numpy as np

import h3adv
import h3cap
import h3drive
import h3lod
import h3mem

CLIENTS = Path(__file__).resolve().parents[1]
MAP_VIEW = CLIENTS / "target/release/h3-map-view"
SITUATIONS = CLIENTS / "map/situations"
TABLES = CLIENTS / "map/assets/base.toml"
OUT = CLIENTS / "out/map"
SPRITES = "H3sprite.lod"
# The manager's DEF fields of a terrain and a river by their .h3m code (design §1.3).
TERRAIN_FIELD, RIVER_FIELD = h3mem.TERRAIN_DEFS, h3mem.RIVER_DEFS
# What the viewer does not draw yet (E24 M1): its rectangles (x0, y0, x1, y1), the last two
# excluded. `panels`: the adventure screen's panels over the frame's picture (M3: the minimap,
# the hero and town lists, the buttons, the status window, the status bar, the resource bar and
# the date), and the parked cursor over the right panel.
MASKS = {
    "panels": [
        (608, 0, 800, 600),
        (0, 552, 608, 600),
    ],
}


def run(*args):
    done = subprocess.run([str(a) for a in args], capture_output=True, text=True)
    if done.returncode:
        raise RuntimeError(f"{' '.join(map(str, args))}: {done.stderr.strip() or done.stdout.strip()}")
    return done.stdout


def hidden_test(rectangles):
    """explored(x, y): every tile but those of the rectangles (corners inclusive)."""
    boxes = [(min(a, c), min(b, d), max(a, c), max(b, d)) for a, b, c, d in rectangles]

    def explored(x, y):
        return not any(x0 <= x <= x1 and y0 <= y <= y1 for x0, y0, x1, y1 in boxes)
    return explored


def pack565(rgb):
    r, g, b = rgb
    return (r >> 3) << 11 | (g >> 2) << 5 | b >> 3


def def_palette565(name):
    """A DEF's palette from the run directory's sprite archive, in 5-6-5 (the executable's
    conversion 0x5226D0 truncates each channel)."""
    offset, size, packed = h3lod.directory(SPRITES)[name.lower()]
    with open(h3lod.DATA / SPRITES, "rb") as f:
        f.seek(offset)
        raw = f.read(packed or size)
    data = zlib.decompress(raw) if packed else raw
    return [pack565(data[16 + 3 * i:19 + 3 * i]) for i in range(256)]


def cycling():
    """The cycling DEFs of the map's tables: (manager field, DEF, ranges)."""
    tables = tomllib.loads(TABLES.read_text())
    found = []
    for kind, field in (("terrain", TERRAIN_FIELD), ("river", RIVER_FIELD)):
        for entry in tables[kind]:
            if entry.get("cycles"):
                found.append((field + 4 * entry["code"], entry["def"], entry["cycles"]))
    return found


def cycle_fits(mem):
    """The palette cycles' steps k (mod 36) the original's palettes in memory fit (design §1.6):
    each range of every cycling DEF is its file's colours rotated up by k."""
    fits = set(range(36))
    for field, name, ranges in cycling():
        held, original = mem.def_palette(field), def_palette565(name)
        for first, last in ranges:
            n = last - first + 1
            fits &= {k for k in range(36)
                     if all(held[first + j] == original[first + (j - k) % n] for j in range(n))}
    return fits


def cycle_step(mem, tries=20):
    """The palette cycles' step k at the frozen clock: the smallest k that fits, once two reads
    0.1 s apart fit the same. A read can meet the tick's rotation half done (`palette16_rotate`
    moves the entries one by one; E24 M1 T3b2 saw one such read in 22): it is read again."""
    last = None
    for _ in range(tries):
        fits = cycle_fits(mem)
        if fits and fits == last:
            return min(fits)
        last = fits
        time.sleep(0.1)
    raise RuntimeError(f"the palettes in memory are no steady rotation of the files' ({last})")


def masked(names, shape=(600, 800)):
    m = np.zeros(shape, dtype=bool)
    for name in names:
        for x0, y0, x1, y1 in MASKS[name]:
            m[y0:y1, x0:x1] = True
    return m


def started(map_path, run_dir, tries=2):
    """The original on the map's adventure screen (a single scenario). A game that crashes on its
    way there is started once more: the original's tavern rumour at a new game reads one entry past
    its table about once in 800 games (rumour_pick_text 0x4CCF20; E24 M1 T4a saw it)."""
    for attempt in range(1, tries + 1):
        game = h3drive.Game(run_dir)
        game.start(map_path)
        try:
            h3drive.to_scenario(game)
            return game
        except TimeoutError:
            log = run_dir / "wine.log"
            crashed = log.exists() and "Unhandled page fault" in log.read_text(errors="replace")
            game.stop()
            if not crashed or attempt == tries:
                raise
            print(f"{map_path}: the original crashed on its way to the map; started again",
                  file=sys.stderr, flush=True)


def still(situation_path, out=None, keep=False):
    """Shows and draws one situation; returns the differing pixels, the compared ones and where
    the differing lie (text)."""
    situation_path = Path(situation_path)
    out = Path(out or OUT / situation_path.stem)
    out.mkdir(parents=True, exist_ok=True)
    sit = tomllib.loads(situation_path.read_text())
    map_path = out / "map.h3m"
    run(MAP_VIEW, "situation", situation_path, "--out", map_path)
    vx, vy, vz = sit["view"]
    hidden = sit.get("hidden", [])
    game = started(map_path, out / "run")
    try:
        mem = h3mem.Mem()
        mem.write_explored(vz, hidden_test(hidden), mem.current_player())
        # The view's top-left from a minimap click on the tile it puts at CENTRE; another view
        # first when it is there already, so that the click redraws it.
        cx, cy = h3adv.CENTRE
        if mem.view() == (vx, vy):
            h3adv.centre(game, mem, vx + cx + 2, vy + cy, vz)
        h3adv.centre(game, mem, vx + cx, vy + cy, vz)
        game.move(*h3adv.PARK)
        game.clock("step 100")
        now = game.clock("freeze")
        # The moment: the idle step redraws the view only once the clock reaches the frame
        # deadline (every 180 ms), the tick rotates the palettes once it reaches its time (every
        # 200 ms, polled from many sites), so memory can be a step ahead of the screen (E24 M1
        # T3b2). Stepped to the later of the two, the redraw's own opening tick rotates first:
        # the screen shows the palettes memory holds.
        tick, redraw = mem.map_due()
        steps = mem.frame_steps()
        now = game.clock(f"step {max(tick, redraw) - now}")
        # The clock's 20 reads can come before the main loop's idle step has run: wait for both.
        h3adv.wait_for(lambda: mem.frame_steps() == steps + 1 and mem.map_due()[0] > tick,
                       f"the redraw and the tick at {now} ({tick}, {redraw})")
        k = cycle_step(mem)
        cap = game.settled()
        game.dump(out / "still.xwd")
        if cycle_step(mem) != k:
            raise RuntimeError("the palettes changed at the frozen clock")
        # The redraw drew with the manager's +0x100 before it grew it (design §1.6): the screen
        # shows the step before the one memory holds. The objects in the map's order: their
        # anchors (the viewer checks them against its map's) and phases.
        objects = mem.map_objects()
        live = {"view": list(mem.view()) + [mem.view_level()], "cycles": k,
                "frames": mem.frame_steps() - 1,
                "anchors": [list(o[:3]) for o in objects],
                "phases": [o[4] for o in objects]}
        (out / "live.toml").write_text(
            "# The original's live part at the still (h3map.py still).\n"
            + "".join(f"{key} = {value}\n" for key, value in live.items()))
    finally:
        if not keep:
            game.stop()
    args = [MAP_VIEW, "render", map_path, "--view", f"{vx},{vy},{vz}", "--live",
            out / "live.toml", "--bugs", "original", "--out", out / "ours.png"]
    for rect in hidden:
        args += ["--hidden", ",".join(map(str, rect))]
    run(*args)
    ours = h3cap.png565(out / "ours.png")
    mask = masked(["panels"], cap.shape)
    n, where = h3cap.compare(cap, ours, mask, out / "diff.png")
    return n, int((~mask).sum()), where


def run_set(rest):
    only = rest[rest.index("--only") + 1] if "--only" in rest else None
    base = Path(rest[rest.index("--out") + 1]) if "--out" in rest else OUT
    entries = tomllib.loads((SITUATIONS / "set.toml").read_text())["situation"]
    if only:
        entries = [e for e in entries if e["id"] == only]
        if not entries:
            sys.exit(f"no situation `{only}` in set.toml")
    changed = 0
    for e in entries:
        n, compared, where = still(SITUATIONS / f"{e['id']}.toml", base / e["id"])
        recorded = e.get("differing")
        verdict = "new" if recorded is None else ("ok" if n == recorded else "CHANGED")
        changed += verdict != "ok"
        print(f"{e['id']}: {n} differing of {compared} (recorded {recorded}) {verdict}{where}",
              flush=True)
    return 1 if changed else 0


def main(args):
    if args[:1] == ["still"] and len(args) >= 2:
        out = args[args.index("--out") + 1] if "--out" in args else None
        n, compared, where = still(args[1], out, "--keep" in args)
        print(f"differing: {n} of {compared} unmasked pixels{where}")
        return 0
    if args[:1] == ["set"]:
        return run_set(args[1:])
    print(__doc__, file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
