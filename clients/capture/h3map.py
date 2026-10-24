#!/usr/bin/env python3
"""The adventure map's stills (E24 M1 T3b2, *provisional*; clients/capture/README.md, "The map's
stills"): a situation (clients/map/situations/) shown in the original and drawn by the viewer,
compared pixel for pixel outside the masks of what the viewer does not draw yet.

  h3map.py still <situation.toml> [--out <dir>] [--keep]
      builds the situation's map (`h3-map-view situation`), or takes the shipped map it names
      whole (`map`, from the installation copy's Maps: E24 M1 T4b), loads it in the original as a
      single scenario (the scenario list's first human player, Red on the built maps), walks the
      heroes the situation walks (by play, E24 M1 T5a), writes that player's explored tiles as
      the situation gives them (every tile but its `hidden` rectangles; with `explored = "start"`
      none, E24 M1 T5b2), centres the view by the minimap so that its top-left is the
      situation's `view`, shows the path of the hero the situation gives one (T5b2), writes the
      heroes' and boats' facings and the boarded boats' flag colours it gives, parks the mouse,
      freezes the clock, steps it to the next moment both
      the palette tick and the view's redraw come (so the screen shows the palettes memory
      holds) and dumps the screen (`still.xwd`);
      reads the palette cycles' step k from the cycling DEFs' palettes in memory and writes it
      with the view, the objects' frame step the screen shows, the viewing player, every
      object's anchor and phase and its template in memory (what a random object became), the
      heroes and boats the view draws, the owners, the viewer's explored tiles and the path's
      arrows (`live.toml`); draws ours (`h3-map-view render --bugs original --live live.toml`,
      `ours.png`) and prints the pixels that differ outside the masks (`diff.png`: differing
      pixels red, masked ones blue). <dir> defaults to clients/out/map/<situation>.
  h3map.py set [--only <id>] [--out <dir>]
      every situation of clients/map/situations/set.toml (or one), each count against the one
      recorded; exit 1 if any changed. Each capture as recorded or new keeps its live.toml as
      clients/map/situations/kept/<id>.toml (E24 M1 T6a).
  h3map.py drawn [--only <id>] [--out <dir>] [--record]
      without the original: ours drawn with each situation's kept live part, the digest of its
      pixels outside the masks against kept/drawn.txt (`same`, `CHANGED`, `new`); exit 1 unless
      every one is the same; --record writes the digests (E24 M1 T6a; the runbook, "Which
      captures a change needs"). <dir> defaults to clients/out/map-drawn.

Needs `clients/capture/setup.sh` and `cargo build --release -p h3-map-view` (from clients/).
"""

import json
import shutil
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
SHIPPED = CLIENTS.parent / "original-data/install/Maps"
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


# The kinds of cycling DEFs, in the order the manager loads them (design §1.6).
KINDS = ("terrain", "river")


def cycling():
    """The cycling DEFs of the map's tables: (kind, manager field, DEF, ranges)."""
    tables = tomllib.loads(TABLES.read_text())
    found = []
    for kind, field in zip(KINDS, (TERRAIN_FIELD, RIVER_FIELD)):
        for entry in tables[kind]:
            if entry.get("cycles"):
                found.append((kind, field + 4 * entry["code"], entry["def"], entry["cycles"]))
    return found


def range_fits(mem):
    """Each cycling DEF's ranges and the steps k (mod 36) each fits in memory (design §1.6): the
    range is its file's colours rotated up by k. Each (kind, DEF, first entry, steps)."""
    found = []
    for kind, field, name, ranges in cycling():
        held, original = mem.def_palette(field), def_palette565(name)
        for first, last in ranges:
            n = last - first + 1
            found.append((kind, name, first, {k for k in range(36) if all(
                held[first + j] == original[first + (j - k) % n] for j in range(n))}))
    return found


def cycle_fits(mem):
    """The palette cycles' steps k (mod 36) every range of every cycling DEF of each kind fits:
    (the terrains', the rivers')."""
    fits = {kind: set(range(36)) for kind in KINDS}
    for kind, _, _, ks in range_fits(mem):
        fits[kind] &= ks
    return tuple(fits[kind] for kind in KINDS)


def cycle_step(mem, tries=20):
    """The palette cycles' steps at the frozen clock, the terrains' and the rivers': the smallest
    k of each kind that fits, once two reads 0.1 s apart fit the same. A read can meet the tick's
    rotation half done (`palette16_rotate` moves the entries one by one; E24 M1 T3b2 saw one such
    read in 22): it is read again. The rivers' can be a step behind the terrains' for the whole
    game (E24 M1 T6a, design §1.6: the map's load runs the tick through its progress bar between
    the terrains' DEFs and the rivers', so a tick due then rotates the terrains alone); any other
    difference is no state the reading knows."""
    last = None
    for _ in range(tries):
        fits = cycle_fits(mem)
        if all(fits) and fits == last:
            terrains, rivers = (min(ks) for ks in fits)
            if (terrains - rivers) % 36 not in (0, 1):
                raise RuntimeError(f"the rivers' palettes at step {rivers}, the terrains' at "
                                   f"{terrains}: no tick between their loads explains it")
            return terrains, rivers
        last = fits
        time.sleep(0.1)
    each = "; ".join(f"{name} {first}: {sorted(ks)[:4]}" for _, name, first, ks in range_fits(mem))
    raise RuntimeError(f"the palettes in memory are no steady rotation of the files' ({last}; "
                       f"{each})")


def masked(names, shape=(600, 800)):
    m = np.zeros(shape, dtype=bool)
    for name in names:
        for x0, y0, x1, y1 in MASKS[name]:
            m[y0:y1, x0:x1] = True
    return m


def to_shipped(game, timeout=120.0):
    """From the start to a shipped map's adventure screen (a single scenario, as
    h3drive.to_scenario), closing the messages its first day shows (a timed event's: E24 M1 T4b's
    Divided Loyalties) by their OK. The screen is `adventure`: `adventure_map`'s probes hold the
    level button greyed, as a map without an underground shows it."""
    game.wait("main_menu", 90)
    game.click(640, 75, 1.0)  # New Game
    game.wait("new_game_menu")
    game.click(640, 75, 1.0)  # Single Scenario
    game.wait("scenario_list")
    game.click(497, 553, 1.0)  # Begin
    game.move(*h3adv.PARK)
    end = time.monotonic() + timeout
    while time.monotonic() < end:
        pix = game.pixels()
        if game.matches("adventure", pix) or h3adv.find_button(game, "ok", pix):
            break
        time.sleep(0.2)
    h3adv.dismiss(game)


def started(map_path, run_dir, shipped=False, tries=2):
    """The original on the map's adventure screen (a single scenario). A game that crashes on its
    way there is started once more: the original's tavern rumour at a new game reads one entry past
    its table about once in 800 games (rumour_pick_text 0x4CCF20; E24 M1 T4a saw it). So is one
    whose Wine exits at once, before the main menu, writing nothing (E24 M1 T4b saw it once in a
    set run, right after the game before it stopped)."""
    for attempt in range(1, tries + 1):
        game = h3drive.Game(run_dir)
        game.start(map_path)
        try:
            (to_shipped if shipped else h3drive.to_scenario)(game)
            return game
        except (TimeoutError, RuntimeError) as err:
            log = run_dir / "wine.log"
            text = log.read_text(errors="replace") if log.exists() else ""
            crashed = isinstance(err, TimeoutError) and "Unhandled page fault" in text
            vanished = isinstance(err, RuntimeError) and "main_menu" in str(err) and not text
            game.stop()
            if not (crashed or vanished) or attempt == tries:
                raise
            why = "crashed on its way to the map" if crashed else "exited before its menu"
            print(f"{map_path}: the original {why}; started again", file=sys.stderr, flush=True)


def arrange(game, mem, sit):
    """The heroes' walks a situation gives, by play (E24 M1 T5a: Red's heroes, each to its tiles in
    turn; a hero boards a boat by walking onto it), each checked to arrive; returns the facings to
    write [(x, y, facing)]: a hero's on its last tile, a boat's on its own."""
    turn = []
    for hero in sit.get("heroes", []):
        x, y = hero["at"]
        for target in hero.get("walk", []):
            obs = h3adv.walk(game, mem, mem.hero(mem.hero_on(x, y))["name"], target)
            if not obs["arrived"]:
                raise RuntimeError(f"the hero from {(x, y)} did not arrive at {target}: {obs}")
            x, y = target
        if "facing" in hero:
            turn.append((x, y, hero["facing"]))
    turn += [(*boat["at"], boat["facing"]) for boat in sit.get("boats", []) if "facing" in boat]
    return turn


def set_view(game, mem, vx, vy, vz):
    """The view's top-left at (vx, vy, vz) from a minimap click on the tile it puts at CENTRE;
    another view first when it is there already, so that the click redraws it."""
    cx, cy = h3adv.CENTRE
    if mem.view() == (vx, vy):
        h3adv.centre(game, mem, vx + cx + 2, vy + cy, vz)
    h3adv.centre(game, mem, vx + cx, vy + cy, vz)
    game.move(*h3adv.PARK)


def show_path(game, mem, sit, view):
    """The path a situation's hero shows (E24 M1 T5b2; design §2.10): the hero selected (the hero
    list centres the view on it), the view set again, the path's tile clicked once (as
    `h3adv.walk`'s first click), the mouse parked; checked to show an arrow on that tile."""
    for hero in sit.get("heroes", []):
        if "path" not in hero:
            continue
        x, y = (hero.get("walk") or [hero["at"]])[-1]
        tx, ty = hero["path"]
        h3adv.select(game, mem, mem.hero(mem.hero_on(x, y))["name"])
        set_view(game, mem, *view)
        game.click(*h3adv.hover(game, mem, tx, ty, view[2]), 0.3)
        game.move(*h3adv.PARK)
        h3adv.wait_for(lambda: [tx, ty, view[2]] in [a[:3] for a in mem.path_arrows()],
                       f"the path to {(tx, ty)} shown")


def write_map(situation_path, sit, map_path):
    """A situation's map: the shipped one it names, or the writer's (`h3-map-view situation`)."""
    if "map" in sit:
        shutil.copy(SHIPPED / sit["map"], map_path)
    else:
        run(MAP_VIEW, "situation", situation_path, "--out", map_path)


def draw_ours(map_path, sit, live_path, png):
    """Ours at a situation's view and hidden rectangles, with the original's live part
    (`live_path`) and its drawing bugs."""
    vx, vy, vz = sit["view"]
    args = [MAP_VIEW, "render", map_path, "--view", f"{vx},{vy},{vz}", "--live", live_path,
            "--bugs", "original", "--out", png]
    for rect in sit.get("hidden", []):
        args += ["--hidden", ",".join(map(str, rect))]
    run(*args)


def still(situation_path, out=None, keep=False):
    """Shows and draws one situation; returns the differing pixels, the compared ones and where
    the differing lie (text)."""
    situation_path = Path(situation_path)
    out = Path(out or OUT / situation_path.stem)
    out.mkdir(parents=True, exist_ok=True)
    sit = tomllib.loads(situation_path.read_text())
    map_path = out / "map.h3m"
    write_map(situation_path, sit, map_path)
    vx, vy, vz = sit["view"]
    hidden = sit.get("hidden", [])
    # `explored = "start"` (E24 M1 T5b2): the explored tiles as the new game left them, not written.
    start = sit.get("explored") == "start"
    if start and hidden:
        raise ValueError(f"{situation_path}: `hidden` with the start's explored tiles")
    game = started(map_path, out / "run", shipped="map" in sit)
    try:
        mem = h3mem.Mem()
        turn = arrange(game, mem, sit)
        if not start:
            mem.write_explored(vz, hidden_test(hidden), mem.current_player())
        set_view(game, mem, vx, vy, vz)
        show_path(game, mem, sit, (vx, vy, vz))
        # After the view moved: the selected hero is back on its tile (hero_on). The redraws
        # below show the facings and the boats' colours.
        for x, y, facing in turn:
            mem.write_facing(x, y, 0, facing)
        for boat in sit.get("boats", []):
            if "colour" in boat:
                mem.write_boat_colour(*boat["at"], 0, boat["colour"])
        game.clock("step 100")
        now = game.clock("freeze")
        # The moment: the idle step redraws the view only once the clock reaches the frame
        # deadline (every 180 ms), the tick rotates the palettes once it reaches its time (every
        # 200 ms, polled from many sites), so memory can be a step ahead of the screen (E24 M1
        # T3b2). Stepped to the later of the two, the redraw's own opening tick rotates first:
        # the screen shows the palettes memory holds. A deadline the frozen clock already passed
        # is met at the main loop's next idle step (each then lies ahead of it again): wait for
        # that first, or the steps counted below hold an extra redraw (E24 M1 T4b saw it once).
        h3adv.wait_for(lambda: min(mem.map_due()) > now,
                       f"the deadlines past the frozen clock at {now} ({mem.map_due()})")
        tick, redraw = mem.map_due()
        steps = mem.frame_steps()
        now = game.clock(f"step {max(tick, redraw) - now}")
        # The clock's 20 reads can come before the main loop's idle step has run: wait for both.
        # A second redraw at the same time can come too (E24 M1 T4b: once in eleven, the redraw
        # due before the tick): the screen then shows the last, whose step memory holds as well.
        h3adv.wait_for(lambda: mem.frame_steps() > steps and mem.map_due()[0] > tick,
                       f"the redraw and the tick at {now} ({tick}, {redraw})")
        k, k_rivers = cycle_step(mem)
        cap = game.settled()
        game.dump(out / "still.xwd")
        if cycle_step(mem) != (k, k_rivers):
            raise RuntimeError("the palettes changed at the frozen clock")
        if k != k_rivers:
            print(f"{situation_path.stem}: the rivers' palettes a step behind the terrains' (a "
                  "tick between their loads, design §1.6)", file=sys.stderr, flush=True)
        # The redraw drew with the manager's +0x100 before it grew it (design §1.6): the screen
        # shows the step before the one memory holds. The objects in the map's order: their
        # anchors (the viewer checks them against its map's) and phases.
        # Each object's template in memory: a random one's is the copy a new game made (design
        # §2.7), whose DEF, class and subclass the viewer draws it as; a random dwelling's the
        # dwelling's it became, a town's its picture by its buildings (§2.9). Each object's owner
        # as the view finds it (T5b1). The viewer's explored tiles and the path shown (T5b2).
        objects, templates = mem.map_objects(), mem.map_templates()
        heroes, boats = mem.map_heroes(vz)
        live = {"view": list(mem.view()) + [mem.view_level()], "cycles": k,
                "river_cycles": k_rivers,
                "frames": mem.frame_steps() - 1, "viewer": mem.current_player(),
                "anchors": [list(o[:3]) for o in objects],
                "phases": [o[4] for o in objects],
                "defs": json.dumps([templates[o[3]][0] for o in objects]),
                "classes": [list(templates[o[3]][1:]) for o in objects],
                "heroes": heroes, "boats": boats, "owners": mem.map_owners(),
                "explored": json.dumps(mem.explored_rows(mem.current_player())),
                "path": mem.path_arrows()}
        (out / "live.toml").write_text(
            "# The original's live part at the still (h3map.py still).\n"
            + "".join(f"{key} = {value}\n" for key, value in live.items()))
    finally:
        if not keep:
            game.stop()
    draw_ours(map_path, sit, out / "live.toml", out / "ours.png")
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
        # A situation the harness fails to show (the game, the clock or a read gave up: E24 M1
        # T4b saw three such in its runs, each passing alone) is shown once more, and says so.
        for attempt in (1, 2):
            try:
                n, compared, where = still(SITUATIONS / f"{e['id']}.toml", base / e["id"])
                break
            except (TimeoutError, RuntimeError) as err:
                if attempt == 2:
                    raise
                print(f"{e['id']}: not shown ({err}); shown again", file=sys.stderr, flush=True)
        recorded = e.get("differing")
        verdict = "new" if recorded is None else ("ok" if n == recorded else "CHANGED")
        changed += verdict != "ok"
        print(f"{e['id']}: {n} differing of {compared} (recorded {recorded}) {verdict}{where}",
              flush=True)
        keep_live(e["id"], base / e["id"] / "live.toml", verdict == "CHANGED")
    return 1 if changed else 0


def keep_live(ident, live_path, changed):
    """Keeps a still's `live.toml` (the original's live part: numbers and the DEFs' names only,
    ADR-0009) as `kept/<id>.toml` beside the set, so that `drawn` draws ours without the
    original (E24 M1 T6a). Only a capture as recorded or new (E17 M3 T9b, the reviewer: one that
    differed would let `drawn` vouch for a count no capture gave): once its count is recorded,
    `set --only <id>` again keeps it."""
    if changed:
        print(f"{ident}: not as recorded, `kept/` left as it was", flush=True)
        return
    kept = SITUATIONS / "kept" / f"{ident}.toml"
    kept.parent.mkdir(exist_ok=True)
    shutil.copy(live_path, kept)


def run_drawn(rest):
    """Ours without the original (runbook, "Which captures a change needs"; E24 M1 T6a): each
    situation of the set drawn with the live part its last capture kept (`kept/<id>.toml`, written
    by `set`), the digest of its pixels outside the masks (`h3drive.png_digest`: a change only
    under them leaves the recorded count true) compared with `kept/drawn.txt`. One line per
    situation (`same`, `CHANGED`, `new`, or no kept live part); exits 1 unless every one is the
    same. With `--record` it writes the digests instead (those not drawn kept). Needs the release
    `h3-map-view` and the user's original files only (the archives, the shipped maps)."""
    record = "--record" in rest
    only = rest[rest.index("--only") + 1] if "--only" in rest else None
    base = Path(rest[rest.index("--out") + 1]) if "--out" in rest else OUT.parent / "map-drawn"
    entries = tomllib.loads((SITUATIONS / "set.toml").read_text())["situation"]
    if only:
        entries = [e for e in entries if e["id"] == only]
        if not entries:
            sys.exit(f"no situation `{only}` in set.toml")
    digests_path = SITUATIONS / "kept" / "drawn.txt"
    recorded = {}
    if digests_path.exists():
        for line in digests_path.read_text().splitlines():
            if line and not line.startswith("#"):
                ident, digest = line.split()
                recorded[ident] = digest
    got, failed, broken = {}, 0, 0
    for e in entries:
        ident = e["id"]
        kept = SITUATIONS / "kept" / f"{ident}.toml"
        if not kept.exists():
            print(f"{ident}: no kept live part (capture it: `set --only {ident}`)", flush=True)
            failed += 1
            broken += 1
            continue
        here = base / ident
        shutil.rmtree(here, ignore_errors=True)
        here.mkdir(parents=True)
        situation_path = SITUATIONS / f"{ident}.toml"
        sit = tomllib.loads(situation_path.read_text())
        try:
            write_map(situation_path, sit, here / "map.h3m")
            draw_ours(here / "map.h3m", sit, kept, here / "ours.png")
        except RuntimeError as err:
            print(f"{ident}: the viewer failed ({str(err).splitlines()[-1]})", flush=True)
            failed += 1
            broken += 1
            continue
        got[ident] = h3drive.png_digest(here / "ours.png", MASKS["panels"])
        before = recorded.get(ident)
        verdict = "new" if before is None else ("same" if before == got[ident] else "CHANGED")
        failed += verdict != "same"
        print(f"{ident}: {verdict}", flush=True)
    if record:
        recorded.update(got)
        lines = ["# Ours at each situation's last kept live part (h3map.py drawn --record): the",
                 "# digest of its pixels outside the masks (h3drive.png_digest, h3map.MASKS),",
                 "# `<id> <sha1, 16 digits>`."]
        lines += [f"{ident} {recorded[ident]}" for ident in sorted(recorded)]
        digests_path.write_text("\n".join(lines) + "\n")
        print(f"drawn: {len(got)} recorded in {digests_path}"
              + (f"; {broken} not drawn, their digests left as they were" if broken else ""))
        return 1 if broken else 0
    print(f"drawn: {len(entries)} situations, {failed} not the same; under {base}")
    return 1 if failed else 0


def main(args):
    if args[:1] == ["still"] and len(args) >= 2:
        out = args[args.index("--out") + 1] if "--out" in args else None
        n, compared, where = still(args[1], out, "--keep" in args)
        print(f"differing: {n} of {compared} unmasked pixels{where}")
        return 0
    if args[:1] == ["set"]:
        return run_set(args[1:])
    if args[:1] == ["drawn"]:
        return run_drawn(args[1:])
    print(__doc__, file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
