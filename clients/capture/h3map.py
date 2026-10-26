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
      pixels red, masked ones blue); exit 1 if any differ. <dir> defaults to
      clients/out/map/<situation>.
  h3map.py set [--only <id>] [--out <dir>]
      every situation of clients/map/situations/set.toml (or one), each count against the one
      recorded; exit 1 if any changed, is new, failed or the viewer refused its live part (each
      said, the others still run). Each capture as recorded or new keeps its live.toml as
      clients/map/situations/kept/<id>.toml (E24 M1 T6a), with the digest of the situation it
      showed (T6b).
  h3map.py drawn [--only <id>] [--out <dir>] [--record]
      without the original: ours drawn with each situation's kept live part, the digest of its
      pixels outside the masks against kept/drawn.txt (`same`, `CHANGED`, `new`; `STALE` when the
      situation changed since its capture: its staging is no longer what was kept, T6b); exit 1
      unless every one is the same; --record writes the digests (E24 M1 T6a; the runbook, "Which
      captures a change needs"). <dir> defaults to clients/out/map-drawn.
  h3map.py frames [--only <id>] [--out <dir>]
      every sequence of clients/map/sequences.toml (or one; E24 M2 T1a1): its situation staged
      as `still` stages it, its 0 set to the next millisecond (the palette tick's, the redraw's
      and the mouse poll's next times written there), then 1-ms steps to its `ms`, each screen
      whose view changed kept with the state memory holds; its timeline (the changes of the
      view, the objects' steps, the palette steps and where the screen changed) against
      clients/map/timelines/<id>.txt (`ok`, `CHANGED`, `new`), and the viewer's frame at 0
      drawn with the live part read then against the original's; exit 1 unless each is as
      recorded and exact at 0. A run as recorded or new keeps that live part
      (timelines/<id>-live.toml). <dir> defaults to clients/out/map-frames.

Needs `clients/capture/setup.sh` and `cargo build --release -p h3-map-view` (from clients/).
"""

import hashlib
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
    # As `Game.wait` (E24 M1 T6b, the reviewer's R3-4): a game that exits says so, a display that
    # dumps no screen is waited through, a timeout raises.
    end = time.monotonic() + timeout
    fault = None
    while True:
        if game.proc is not None and game.proc.poll() is not None:
            raise RuntimeError(f"the game exited ({game.proc.returncode}) before the map showed")
        try:
            pix = game.pixels()
        except h3drive.Unreadable as err:
            fault = err
        else:
            if game.matches("adventure", pix) or h3adv.find_button(game, "ok", pix):
                break
        if time.monotonic() > end:
            raise TimeoutError("the shipped map's adventure screen did not show"
                               + (f" ({fault})" if fault else ""))
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
        except BaseException as err:
            # Any other failure on the way stops the game too, so that no later `Mem()` reads it
            # (E24 M1 T6b, the reviewer's R3-8).
            if not isinstance(err, (TimeoutError, RuntimeError)):
                game.stop()
                raise
            log = run_dir / "wine.log"
            text = log.read_text(errors="replace") if log.exists() else ""
            crashed = isinstance(err, TimeoutError) and "Unhandled page fault" in text
            vanished = isinstance(err, RuntimeError) and "main_menu" in str(err) and not text
            game.stop()
            if not (crashed or vanished) or attempt == tries:
                raise
            why = "crashed on its way to the map" if crashed else "exited before its menu"
            print(f"{map_path}: the original {why}; started again", file=sys.stderr, flush=True)


def game_mem(game):
    """The started game's memory, before any write into it (E24 M1 T6b, the reviewer's R3-1; as
    `h3drive.sequence` checks it): xdd.dll redirected every roll's and logged draw's call, each
    checked against the bytes it replaces, so the executable is the one the readings are of; and
    the process `Mem` reads is the one `game` started (its clock directory)."""
    sites = len(h3drive.ROLLS) + len(h3drive.LOGGED)
    ack = game.ack()
    if not ack or ack[5] != sites:
        raise RuntimeError(f"xdd.dll redirected {ack and ack[5]} of the {sites} rolls' and logged "
                           "draws' calls (another Heroes3.exe?)")
    mem = h3mem.Mem()
    clock = "H3CLOCK=Z:" + str(game.clock_dir.resolve()).replace("/", "\\")
    environ = Path(f"/proc/{mem.pid}/environ").read_bytes().split(b"\0")
    if clock.encode() not in environ:
        raise RuntimeError(f"the Heroes3.exe read (pid {mem.pid}) is not the game started")
    return mem


# The keys a situation that loads a shipped map whole may have (E24 M1 T4b, T5b2): the Rust
# parser sees only the writer's situations (E24 M1 T6b, the reviewer's R3-11).
SHIPPED_KEYS = {"map", "view", "explored", "hidden"}


def check_situation(situation_path, sit):
    """What the harness stages that the viewer's parser does not check (E24 M1 T6b, the
    reviewer's R3-3, R3-11): a shipped map's situation only its keys (no heroes or boats to
    stage), `explored` only "start"; a boat's facing only on a boat no hero ends on (a boarded
    boat faces as its hero)."""
    if "map" in sit:
        extra = set(sit) - SHIPPED_KEYS
        if extra:
            raise ValueError(f"{situation_path}: a shipped map's situation has no "
                             f"{', '.join(sorted(extra))}")
    if sit.get("explored", "start") != "start":
        raise ValueError(f"{situation_path}: `explored` is \"start\" or not given")
    ends = {tuple((h.get("walk") or [h["at"]])[-1]) for h in sit.get("heroes", [])}
    for boat in sit.get("boats", []):
        if "facing" in boat and tuple(boat["at"]) in ends:
            raise ValueError(f"{situation_path}: the boat at {boat['at']} is boarded: its "
                             "facing is its hero's")


def hero_at(mem, x, y):
    """The record of the player's hero on (x, y) of the surface; a situation's hero not there is
    the harness's failure (E24 M1 T6b, the reviewer's R3-2: not a TypeError)."""
    index = mem.hero_on(x, y)
    if index is None:
        raise RuntimeError(f"no hero of a player stands on {(x, y)}")
    return mem.hero(index)


def check_staged(sit, turn, heroes, boats):
    """The facings and boats' colours the still staged are what memory holds at it (E24 M1 T6b,
    the reviewer's R3-3): a write that did not stick, or turned another record, fails the
    still."""
    on = {(b[0], b[1]): ("boat", b[4], None) for b in boats}
    on.update({(h[0], h[1]): ("hero", h[4], h[7]) for h in heroes})
    for x, y, facing in turn:
        held = on.get((x, y))
        if held is None or held[1] != facing:
            raise RuntimeError(f"the facing staged on {(x, y)}: {facing}, memory {held}")
    for boat in sit.get("boats", []):
        if "colour" in boat:
            held = on.get(tuple(boat["at"]))
            if held is None or held[0] != "hero" or held[2] != boat["colour"]:
                raise RuntimeError(f"the boat's colour staged on {boat['at']}: {boat['colour']}, "
                                   f"memory {held}")


def situation_digest(sit):
    """A digest of what a situation gives (its parsed keys, comments left out): `drawn` tells a
    kept live part of another staging by it (E24 M1 T6b, the reviewer's R3-10)."""
    text = json.dumps(sit, sort_keys=True, separators=(",", ":"))
    return hashlib.sha1(text.encode()).hexdigest()[:16]


def arrange(game, mem, sit):
    """The heroes' walks a situation gives, by play (E24 M1 T5a: Red's heroes, each to its tiles in
    turn; a hero boards a boat by walking onto it), each checked to arrive; returns the facings to
    write [(x, y, facing)]: a hero's on its last tile, a boat's on its own."""
    turn = []
    for hero in sit.get("heroes", []):
        x, y = hero["at"]
        for target in hero.get("walk", []):
            obs = h3adv.walk(game, mem, hero_at(mem, x, y)["name"], target)
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
        # Two tiles over, towards the map's middle (a click past the minimap's right edge would
        # miss it: E24 M1 T6b, the reviewer).
        side = mem.i32(mem.game() + h3mem.MAP_SIDE)
        dx = 2 if vx + cx + 2 < side else -2
        h3adv.centre(game, mem, vx + cx + dx, vy + cy, vz)
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
        h3adv.select(game, mem, hero_at(mem, x, y)["name"])
        set_view(game, mem, *view)
        game.click(*h3adv.hover(game, mem, tx, ty, view[2]), 0.3)
        game.move(*h3adv.PARK)
        h3adv.wait_for(lambda: [tx, ty, view[2]] in [a[:3] for a in mem.path_arrows()],
                       lambda: f"the path to {(tx, ty)} shown ({mem.path_arrows()})")


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
    cap = capture(situation_path, out, keep)
    return compare(situation_path, out, cap)


def stage(situation_path, out):
    """Shows one situation in the original, up to its moment (E24 M2 T1a1: `capture`'s and
    `sequence`'s common part): its map written to `out` (`map.h3m`), the game started on it, the
    heroes walked, the explored tiles written, the view set, the path shown, the facings and the
    boats' colours written; the clock still running. Returns the game (the caller stops it; a
    failure here stops it first), its memory, the situation and the facings written
    (`arrange`)."""
    out.mkdir(parents=True, exist_ok=True)
    sit = tomllib.loads(situation_path.read_text())
    check_situation(situation_path, sit)
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
        mem = game_mem(game)
        turn = arrange(game, mem, sit)
        if not start:
            mem.write_explored(vz, hidden_test(hidden), mem.current_player())
        set_view(game, mem, vx, vy, vz)
        show_path(game, mem, sit, (vx, vy, vz))
        # After the view moved: the selected hero is back on its tile (hero_on). The redraws
        # after this show the facings and the boats' colours.
        for x, y, facing in turn:
            mem.write_facing(x, y, 0, facing)
        for boat in sit.get("boats", []):
            if "colour" in boat:
                mem.write_boat_colour(*boat["at"], 0, boat["colour"])
    except BaseException:
        game.stop()
        raise
    return game, mem, sit, turn


def live_part(mem, sit, turn, k, k_rivers):
    """The live part the screen shows right after a redraw at the frozen clock (the palette steps
    `k`, `k_rivers` read by `cycle_step`), as `render --live` takes it: `live.toml`'s lines. The
    redraw drew with the manager's +0x100 before it grew it (design §1.6): the screen shows the
    step before the one memory holds. The objects in the map's order: their anchors (the viewer
    checks them against its map's) and phases. Each object's template in memory: a random one's
    is the copy a new game made (design §2.7), whose DEF, class and subclass the viewer draws it
    as; a random dwelling's the dwelling's it became, a town's its picture by its buildings
    (§2.9). Each object's owner as the view finds it (T5b1). The viewer's explored tiles and the
    path shown (T5b2). Checks the heroes and boats against what the situation staged."""
    objects, templates = mem.map_objects(), mem.map_templates()
    heroes, boats = mem.map_heroes(sit["view"][2])
    live = {"view": list(mem.view()) + [mem.view_level()], "cycles": k,
            "river_cycles": k_rivers,
            "frames": mem.frame_steps() - 1, "viewer": mem.current_player(),
            "anchors": [list(o[:3]) for o in objects],
            "phases": [o[4] for o in objects],
            "defs": json.dumps([templates[o[3]][0] for o in objects]),
            "classes": [list(templates[o[3]][1:]) for o in objects],
            "heroes": heroes, "boats": boats, "owners": mem.map_owners(),
            "explored": json.dumps(mem.explored_rows(mem.current_player())),
            "path": mem.path_arrows(),
            "situation": json.dumps(situation_digest(sit))}
    check_staged(sit, turn, heroes, boats)
    return "".join(f"{key} = {value}\n" for key, value in live.items())


def capture(situation_path, out, keep=False):
    """Shows one situation in the original: its screen (`still.xwd`, the array returned) and its
    live part (`live.toml`) in `out`."""
    game, mem, sit, turn = stage(situation_path, out)
    try:
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
                       lambda: f"the deadlines past the frozen clock at {now} ({mem.map_due()})")
        tick, redraw = mem.map_due()
        steps = mem.frame_steps()
        now = game.clock(f"step {max(tick, redraw) - now}")
        # The clock's 20 reads can come before the main loop's idle step has run: wait for both.
        # A second redraw at the same time can come too (E24 M1 T4b: once in eleven, the redraw
        # due before the tick): the screen then shows the last, whose step memory holds as well.
        h3adv.wait_for(lambda: mem.frame_steps() > steps and mem.map_due()[0] > tick,
                       lambda: f"the redraw and the tick at {now} ({tick}, {redraw}; now "
                               f"{mem.map_due()}, {mem.frame_steps()} steps)")
        k, k_rivers = cycle_step(mem)
        cap = game.settled()
        game.dump(out / "still.xwd")
        # The file kept is the screen compared (E24 M1 T6b, the reviewer's R3-7).
        if not np.array_equal(h3drive.xwd565((out / "still.xwd").read_bytes()), cap):
            raise RuntimeError("the screen changed at the frozen clock")
        if cycle_step(mem) != (k, k_rivers):
            raise RuntimeError("the palettes changed at the frozen clock")
        if k != k_rivers:
            print(f"{situation_path.stem}: the rivers' palettes a step behind the terrains' (a "
                  "tick between their loads, design §1.6)", file=sys.stderr, flush=True)
        (out / "live.toml").write_text("# The original's live part at the still (h3map.py "
                                       "still).\n" + live_part(mem, sit, turn, k, k_rivers))
    finally:
        if not keep:
            game.stop()
    return cap


def compare(situation_path, out, cap):
    """Ours drawn with the live part `capture` read, compared with its screen `cap`: the
    differing pixels, the compared ones and where the differing lie (text). The viewer's refusal
    of the live part raises `Refused`."""
    sit = tomllib.loads(situation_path.read_text())
    try:
        draw_ours(out / "map.h3m", sit, out / "live.toml", out / "ours.png")
    except RuntimeError as err:
        raise Refused(last_lines(err)) from err
    ours = h3cap.png565(out / "ours.png")
    mask = masked(["panels"], cap.shape)
    n, where = h3cap.compare(cap, ours, mask, out / "diff.png")
    return n, int((~mask).sum()), where


# The recorded sequences (E24 M2 T1a1; the epic's M2 decision): their list, timelines and the live
# part each starts from.
SEQUENCES = CLIENTS / "map/sequences.toml"
TIMELINES = CLIENTS / "map/timelines"
OUT_FRAMES = CLIENTS / "out/map-frames"


def cycle_now(mem, tries=10):
    """The palette cycles' steps (the terrains', the rivers') at a settled clock (`cycle_step`
    without its waits: a step's 500 clock reads come after the tick's whole rotation); read again
    a few times if a read fits no step."""
    for _ in range(tries):
        fits = cycle_fits(mem)
        if all(fits):
            return tuple(min(ks) for ks in fits)
        time.sleep(0.01)
    raise RuntimeError(f"the palettes in memory are no steady rotation of the files' ({fits})")


def view_changed(before, after):
    """The box `[x0, y0, x1, y1]` (x1, y1 exclusive) of the pixels that differ between two
    screens outside the panels (the map's view and its frame; None: none)."""
    return h3drive.changed_box(before[:552, :608], after[:552, :608])


def map_timeline(states):
    """The changes over a sequence (E24 M2 T1a1), one line each, numbers only: `<ms> view <x>
    <y> <level>` (the view's top-left tile), `<ms> objects <n>` (the objects' frame steps memory
    holds, the manager's +0x100, counted from the sequence's 0), `<ms> cycles <terrains>
    <rivers>` (the palette steps memory holds, each kind's ticks counted from 0) and `<ms> screen
    <x0>,<y0>,<x1>,<y1>` (where the screen outside the panels changed since the screen kept
    before: each such screen is kept; x1, y1 exclusive). The first state, at 0, gives the view,
    the objects and the cycles; its screen is always kept, without a line."""
    lines, last = [], {}
    for state in states:
        t = state["t"]
        for key in ("view", "objects", "cycles"):
            if last.get(key) != state[key]:
                lines.append(f"{t} {key} " + " ".join(map(str, state[key])))
                last[key] = state[key]
        if state.get("changed"):
            lines.append(f"{t} screen " + ",".join(map(str, state["changed"])))
    return lines


def sequence(entry, out):
    """One sequence in the original (E24 M2 T1a1): its situation staged as `still` stages it, the
    clock frozen once both the palette tick and the view's redraw lie ahead of it; then the
    sequence's 0 set to the next millisecond: the tick's and the redraw's next times written
    there, and the mouse poll's (the harness's own timers, as E17's `align_mouse_poll`: the
    original's phase between them is the map's load's real time, so a run starts like every
    other). Then `ms` + 1 steps of 1 ms (0 to `ms`), each followed by the state memory holds and
    the screen: every screen whose view (outside the panels) changed is kept
    (`<out>/frames/t<ms>.npz`, local), its state in `<out>/frames.jsonl` (every step's in
    `states.jsonl`, with the tick's and the redraw's next times, `due`). Writes the timeline
    (`<out>/timeline.txt`, `map_timeline`), the live part the screen shows at 0
    (`<out>/live.toml`, `live_part`) and the viewer's frame at 0 drawn with it against the
    original's (`<out>/ours.png`, `diff.png`); returns the states and that frame's differing
    pixels."""
    situation_path = SITUATIONS / f"{entry['situation']}.toml"
    game, mem, sit, turn = stage(situation_path, out)
    kept = out / "frames"
    shutil.rmtree(kept, ignore_errors=True)
    kept.mkdir()
    (out / "frames.jsonl").unlink(missing_ok=True)
    try:
        game.clock("step 100")
        now = game.clock("freeze")
        # As `capture`: a deadline the frozen clock passed is met at the next idle step first.
        h3adv.wait_for(lambda: min(mem.map_due()) > now,
                       lambda: f"the deadlines past the frozen clock at {now} ({mem.map_due()})")
        t0 = now + 1
        mem.write_u32(h3mem.PALETTE_TICK_TIME, t0)
        mem.write_u32(h3mem.FRAME_DEADLINE, t0)
        mem.align_mouse_poll(t0)
        if game.clock("step 1", settle_reads=500) != t0:
            raise RuntimeError("the clock did not step to the sequence's 0")
        # The tick and the redraw came at 0 (each moved its next time on), so the screen shows
        # the live part read now (a still's moment, as `capture` steps to).
        h3adv.wait_for(lambda: mem.map_due() == (t0 + 200, t0 + 180),
                       lambda: f"the tick and the redraw at the sequence's 0 ({t0}: next "
                               f"{mem.map_due()})")
        base = {"objects": mem.frame_steps(), "cycles": cycle_now(mem)}
        counted = {"cycles": base["cycles"], "ticks": (0, 0)}
        live = live_part(mem, sit, turn, *base["cycles"])

        def state(t, pix, before):
            k = cycle_now(mem)
            ticks = tuple(n + (now_k - was) % 36 for n, now_k, was
                          in zip(counted["ticks"], k, counted["cycles"]))
            counted.update(cycles=k, ticks=ticks)
            s = {"t": t - t0, "view": list(mem.view()) + [mem.view_level()],
                 "objects": [mem.frame_steps() - base["objects"]], "cycles": list(ticks),
                 "due": [d - t0 for d in mem.map_due()]}
            box = None if before is None else view_changed(before, pix)
            if before is None or box:
                s["changed"] = box
                s["frame"] = f"t{t - t0:05d}.npz"
                np.savez_compressed(kept / s["frame"], screen=pix)
            return s

        first = game.settled()
        states, last = [state(t0, first, None)], first
        for _ in range(entry["ms"]):
            t = game.clock("step 1", settle_reads=500)
            pix = game.settled()
            states.append(state(t, pix, last))
            if "frame" in states[-1]:
                last = pix
    finally:
        game.stop()
    (out / "live.toml").write_text("# The original's live part at the sequence's 0 (h3map.py "
                                   "frames).\n" + live)
    with open(out / "states.jsonl", "w") as f, open(out / "frames.jsonl", "w") as g:
        for s in states:
            f.write(json.dumps(s) + "\n")
            if "frame" in s:
                g.write(json.dumps(s) + "\n")
    (out / "timeline.txt").write_text("\n".join(map_timeline(states)) + "\n")
    draw_ours(out / "map.h3m", sit, out / "live.toml", out / "ours.png")
    n, _ = h3cap.compare(first, h3cap.png565(out / "ours.png"), masked(["panels"], first.shape),
                         out / "diff.png")
    return states, n


def run_frames(rest):
    """`frames [--only <id>] [--out <dir>]`: each sequence of `clients/map/sequences.toml` (or
    one) recorded (`sequence`); its timeline against the one recorded in
    `clients/map/timelines/<id>.txt` (`ok`, `CHANGED`, `new`), and the viewer's frame at 0 against
    the original's (the live part at 0 is a still's: exact, as the capture set). A run as recorded
    or new keeps its live part at 0 as `timelines/<id>-live.toml` (for the frame check, M2 T2).
    Exit 1 unless every sequence is as recorded and its frame at 0 exact."""
    only = rest[rest.index("--only") + 1] if "--only" in rest else None
    base = Path(rest[rest.index("--out") + 1]) if "--out" in rest else OUT_FRAMES
    entries = tomllib.loads(SEQUENCES.read_text())["sequence"]
    if only:
        entries = [e for e in entries if e["id"] == only]
        if not entries:
            sys.exit(f"no sequence `{only}` in {SEQUENCES.name}")
    failed = 0
    for e in entries:
        here = base / e["id"]
        try:
            states, n = sequence(e, here)
        except Exception as err:  # noqa: BLE001 (any failure fails this sequence alone)
            failed += 1
            print(f"{e['id']}: FAILED ({type(err).__name__}: {err})", flush=True)
            continue
        got = (here / "timeline.txt").read_text()
        recorded = TIMELINES / f"{e['id']}.txt"
        verdict = ("new" if not recorded.exists()
                   else "ok" if got == recorded.read_text() else "CHANGED")
        frames = sum("frame" in s for s in states)
        print(f"{e['id']}: {len(got.splitlines())} changes, {frames} screens kept in {e['ms']} ms "
              f"{verdict}; the frame at 0: {n} differing", flush=True)
        failed += verdict != "ok" or n != 0
        if verdict == "CHANGED":
            print(f"{e['id']}: not as recorded, its live part not kept ({here / 'timeline.txt'})",
                  flush=True)
        else:
            TIMELINES.mkdir(exist_ok=True)
            shutil.copy(here / "live.toml", TIMELINES / f"{e['id']}-live.toml")
    print(f"frames: {len(entries)} sequences, {failed} not as recorded; under {base}")
    return 1 if failed else 0


class Refused(RuntimeError):
    """The viewer refused a live part the original gave (E24 M1 T6b, the reviewer's R3-2): the
    situation fails, the set goes on."""


def last_lines(err, n=3):
    """The last lines of a failed command's message (E24 M1 T6b, the reviewer's R3-6: a Rust
    panic's last line is its backtrace note)."""
    return " | ".join(str(err).strip().splitlines()[-n:])


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
        # T4b saw three such in its runs, each passing alone) is shown once more, and says so. A
        # second failure, or any other, fails the situation alone, as the viewer's refusal of
        # the live part does (E24 M1 T6b, the reviewer's R3-2): the set goes on.
        path, out = SITUATIONS / f"{e['id']}.toml", base / e["id"]
        try:
            for attempt in (1, 2):
                try:
                    cap = capture(path, out)
                    break
                except (TimeoutError, RuntimeError, OSError, struct.error) as err:
                    if attempt == 2:
                        raise
                    print(f"{e['id']}: not shown ({err}); shown again", file=sys.stderr,
                          flush=True)
            n, compared, where = compare(path, out, cap)
        except Refused as err:
            changed += 1
            print(f"{e['id']}: REFUSED by the viewer ({err})", flush=True)
            continue
        except Exception as err:  # noqa: BLE001 (any failure fails this situation alone)
            changed += 1
            print(f"{e['id']}: FAILED ({type(err).__name__}: {err})", flush=True)
            continue
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
        # A kept live part of another staging than the situation's (E24 M1 T6b, R3-10).
        shown = tomllib.loads(kept.read_text()).get("situation")
        if shown != situation_digest(sit):
            print(f"{ident}: STALE (the situation changed since its capture: `set --only "
                  f"{ident}`)", flush=True)
            failed += 1
            broken += 1
            continue
        try:
            write_map(situation_path, sit, here / "map.h3m")
            draw_ours(here / "map.h3m", sit, kept, here / "ours.png")
        except RuntimeError as err:
            print(f"{ident}: the viewer failed ({last_lines(err)})", flush=True)
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
        return 1 if n else 0
    if args[:1] == ["set"]:
        return run_set(args[1:])
    if args[:1] == ["drawn"]:
        return run_drawn(args[1:])
    if args[:1] == ["frames"]:
        return run_frames(args[1:])
    print(__doc__, file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
