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
      captures a change needs"). Since E24 M2 T2 the recorded sequences too: each one's frames
      drawn by the viewer from its kept live part (timelines/<id>-live.toml) at the times its
      screen changes (`h3-map-view sequences <id> --frames`, which also checks its timeline), the
      digests against clients/map/timelines/drawn.txt; the walks too since E24 M2 T3b. <dir>
      defaults to clients/out/map-drawn.
  h3map.py frames [--only <id>] [--out <dir>] [--reuse]
      every sequence of clients/map/sequences.toml (or one; E24 M2 T1a1): its situation staged
      as `still` stages it, its 0 set to the next millisecond (the palette tick's, the redraw's
      and the mouse poll's next times written there), then 1-ms steps to its `ms`, each screen
      whose view changed kept with the state memory holds; its timeline (the changes of the
      view, the objects' steps, the palette steps and where the screen changed) against
      clients/map/timelines/<id>.txt (`ok`, `CHANGED`, `new`), and the viewer's frame at 0
      drawn with the live part read then against the original's; exit 1 unless each is as
      recorded and exact at 0. A run as recorded or new and exact at 0 keeps that live part
      (timelines/<id>-live.toml). Since E24 M2 T1a2 a sequence's options (the scroll and hero
      speeds) are written once it is staged and its inputs (keys pressed and released, the
      pointer moved) given at their milliseconds; the timeline has the cursor too. Since E24 M2
      T2 the frame check: the viewer's frames at the times the original's screen changed
      (`h3-map-view sequences <id> --frames`), each against the original's outside the panels,
      the counts (`frames.txt`) against clients/map/timelines/<id>-frames.txt; a walk's is M2
      T3's. --reuse keeps the original's run of an earlier `frames` (its screens, its live part)
      and only compares again. <dir> defaults to clients/out/map-frames.

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
# What the viewer does not draw yet (E24 M1; since E24 M3 T2a `clients/map/masks.toml`, which the
# viewer's `sequences` reads too): its rectangles (x0, y0, x1, y1), the last two excluded.
# `panels`: what T5 and T6 draw of the adventure screen's panels (the status window, the amounts
# and the date, the status bar's hint), the situations' and the sequences' (since E24 M3 T4b one
# list again: T4a's `stills` left the lists out for the situations alone).
# `hint` (E24 M3 T6a1): the status bar's hint, left out but of the items `hinted` lists.
_MASKS = tomllib.loads((CLIENTS / "map/masks.toml").read_text())
HINTED = set(_MASKS.pop("hinted", []))
MASKS = {name: [tuple(r) for r in rects] for name, rects in _MASKS.items()}
# What `lists` writes into a situation's hero's record (E24 M3 T4a, T5b1).
HERO_WRITES = {"points", "mana", "skills", "morale", "luck", "army"}
# The status window's kinds the viewer draws (E24 M3 T5b1: a hero's and a town's, design §4.8;
# T5b2 the date's and the kingdom's, §4.9).
STATUS_DRAWN = (1, 2, 3, 4)
# How long a situation's kind asked lasts (E24 M3 T5b2): far past any still.
STATUS_LONG = 10_000_000


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


def panel_rects(ident):
    """The panels' mask of the situation or sequence `ident` (E24 M3 T6a1): `panels`, and the
    status bar's hint (`hint`) unless `hinted` lists it."""
    return MASKS["panels"] + ([] if ident in HINTED else MASKS["hint"])


def masked_for(ident, shape=(600, 800)):
    """`panel_rects(ident)` as a mask of `shape`."""
    m = np.zeros(shape, dtype=bool)
    for x0, y0, x1, y1 in panel_rects(ident):
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


def town_index(mem, at):
    """The index (of the game's towns) of the town anchored on `at` of the surface (E24 M3 T4a;
    the record's place +0x5 is its entrance, two columns left of its anchor: design §4.5)."""
    first, end = (mem.u32(mem.game() + h3mem.TOWNS + i) for i in (0, 4))
    for index in range((end - first) // h3mem.TOWN_SIZE):
        if mem.town(index)["place"] == [at[0] - 2, at[1], 0]:
            return index
    raise RuntimeError(f"no town is anchored on {tuple(at)}")


def lists(game, mem, sit):
    """The lists' writes and selection a situation gives (E24 M3 T4a; design §4.5): Red's
    heroes' points and mana and Red's towns' day written into their records, then the hero or
    town it selects chosen from its list, which scrolls that list to show it and redraws both
    (hero_select 0x417B20, town_select 0x417830). A selection the turn's start made already would
    redraw nothing: refused. Since M3 T5a Red's resources and the game's date written before it
    too: a hero's selection draws the resource bar again (0x417F48; the viewer's parser asks for
    one with them). Since M3 T5b1 what the status window shows (design §4.8): Red's heroes'
    primary skills, morale and luck bonuses and armies, Red's towns' garrisons and garrison
    heroes (a hero of Red's standing elsewhere: only the window reads it), written before the
    selection builds the window again."""
    for hero in sit.get("heroes", []):
        if set(hero) & HERO_WRITES:
            x, y = (hero.get("walk") or [hero["at"]])[-1]
            index = hero_at(mem, x, y)["index"]
            if "points" in hero:
                mem.write_points(index, hero["points"])
            if "mana" in hero:
                mem.write_mana(index, hero["mana"])
            # The status window's (E24 M3 T5b1; design §4.8).
            if "skills" in hero:
                mem.write_skills(index, hero["skills"])
            if "morale" in hero or "luck" in hero:
                mem.write_bonuses(index, hero.get("morale", 0), hero.get("luck", 0))
            if "army" in hero:
                mem.write_army(mem.hero_record(index) + h3mem.ARMY, hero["army"])
    for town in sit.get("towns", []):
        if town.get("built"):
            mem.write_town_built(town_index(mem, town["at"]))
        if "garrison" in town or "garrison_hero" in town:
            record = mem.town(town_index(mem, town["at"]))["at"]
            if "garrison" in town:
                mem.write_army(record + h3mem.TOWN_ARMY, town["garrison"])
            if "garrison_hero" in town:
                hero = hero_at(mem, *town["garrison_hero"])["index"]
                mem.write_bytes(record + h3mem.TOWN_GARRISON_HERO, struct.pack("<i", hero))
    if "resources" in sit:
        mem.write_resources(0, sit["resources"])
    if "date" in sit:
        mem.write_date(*sit["date"])
    select = sit.get("select")
    if not select:
        return
    colour = mem.current_player()
    before = mem.kingdom(colour)["selected"]
    if "hero" in select:
        h3adv.select(game, mem, hero_at(mem, *select["hero"])["name"])
    else:
        h3adv.select_town(game, mem, town_index(mem, select["town"]))
    if mem.kingdom(colour)["selected"] == before:
        raise RuntimeError(f"the selection {select} is the turn start's: the lists not redrawn")


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
        lists(game, mem, sit)
        if not start:
            mem.write_explored(vz, hidden_test(hidden), mem.current_player())
        set_view(game, mem, vx, vy, vz)
        show_path(game, mem, sit, (vx, vy, vz))
        # What choosing each hero and town shows (E24 M3 T6b1; design §4.13), when the situation
        # asks: each chosen in turn, then the staging's selection, lists and view again.
        if sit.get("choose"):
            choose_each(game, mem, (vx, vy, vz))
        # After the view moved: the selected hero is back on its tile (hero_on). The redraws
        # after this show the facings and the boats' colours.
        for x, y, facing in turn:
            mem.write_facing(x, y, 0, facing)
        for boat in sit.get("boats", []):
            if "colour" in boat:
                mem.write_boat_colour(*boat["at"], 0, boat["colour"])
        # The status window's (E24 M3 T5b2; design §4.9): the teams the kingdom's window reads,
        # then the kind asked, built by the next view draw.
        if "teams" in sit:
            mem.write_bytes(mem.game() + h3mem.TEAMS, bytes(sit["teams"]))
        # The visit marks a situation gives (E24 M3 T6a2a; design §4.11), for the hero selected
        # as staged: the hint reads them at each change of the tile under the pointer; since
        # T6a2b (§4.12) the selected hero's own and a mine's or a bank's guards too.
        for visit in sit.get("visits", []):
            mem.write_visit(mem.current_player(), *visit["at"], visit.get("known", False),
                            visit.get("visited"), visit.get("hero"), visit.get("guards"))
        if "status" in sit:
            ask_status(mem, sit["status"], max(mem.map_due()) + STATUS_LONG)
            h3adv.wait_for(lambda: mem.status_shown() == sit["status"],
                           lambda: f"the status window's kind {sit['status']} built "
                                   f"({mem.status_shown()} shown)")
    except BaseException:
        game.stop()
        raise
    return game, mem, sit, turn


def choose_each(game, mem, view):
    """What choosing each of Red's heroes and towns shows (E24 M3 T6b1; design §4.13), kept as
    `mem.chosen` for `live_part`'s `[chosen]`: each one chosen from its list in turn (h3adv's
    `select` and `select_town`, which scroll the list as they need), its status window read once
    it is built (`status_toml`), a hero's path shown (its destination's, `make_hero_path`) and its
    tile; then the staging's selection chosen again, each list's first slot and the view set back
    as they were, the path checked to be the one shown before. The clock runs."""
    colour = mem.current_player()
    before = mem.kingdom(colour)
    path = mem.path_arrows()
    heroes = mem.player(colour)["heroes"]
    towns = mem.town_list(colour)
    chosen = {"heroes": [], "towns": []}
    for hero in heroes:
        record = mem.hero(hero)
        h3adv.select(game, mem, record["name"])
        h3adv.wait_for(lambda: mem.status_shown() == 3 and mem.status(colour)["hero"] == hero,
                       lambda: f"hero {hero}'s status window ({mem.status_shown()} shown)")
        chosen["heroes"].append({"tile": record["tile"], "status": status_toml(mem, colour),
                                 "path": mem.path_arrows()})
    for town in towns:
        h3adv.select_town(game, mem, town)
        h3adv.wait_for(lambda: mem.status_shown() == 4,
                       lambda: f"town {town}'s status window ({mem.status_shown()} shown)")
        chosen["towns"].append({"at": list(mem.town(town)["place"]),
                                "status": status_toml(mem, colour)})
    # The staging's selection again (a hero's makes its path again), then the lists' first slots
    # by their arrows and the view.
    kind, place = before["selected"]
    if kind == 1:
        h3adv.select(game, mem, mem.hero(heroes[place])["name"])
    elif kind == 2:
        h3adv.select_town(game, mem, towns[place])
    for list_index, (up, down) in enumerate([(h3adv.LIST_UP, h3adv.LIST_DOWN),
                                             (h3adv.TOWN_UP, h3adv.TOWN_DOWN)]):
        for _ in range(16):
            top = mem.kingdom(colour)["tops"][list_index]
            want = before["tops"][list_index]
            if top == want:
                break
            game.click(*(up if top > want else down), 0.4)
    game.move(*h3adv.PARK)
    set_view(game, mem, *view)
    after = mem.kingdom(colour)
    if after["selected"] != before["selected"] or after["tops"] != before["tops"]:
        raise RuntimeError(f"the staging's lists not set back: {after} after {before}")
    if sorted(mem.path_arrows()) != sorted(path):
        raise RuntimeError(f"the path {mem.path_arrows()} after choosing each, {path} before")
    mem.chosen = chosen


def chosen_toml(mem, heroes):
    """`mem.chosen` as `render --live`'s `[chosen]` table's lines (E24 M3 T6b1): each hero's
    index among the live part's `heroes` (rows [x, y, level, ...]), its status and its path; each
    town's place and status."""
    tiles = [h[:3] for h in heroes]
    rows = []
    for hero in mem.chosen["heroes"]:
        if hero["tile"] not in tiles:
            raise RuntimeError(f"a chosen hero on {hero['tile']} is not among the live part's")
        rows.append(f"{{ hero = {tiles.index(hero['tile'])}, status = {hero['status']}, "
                    f"path = {json.dumps(hero['path'])} }}")
    towns = [f"{{ at = {json.dumps(t['at'])}, status = {t['status']} }}"
             for t in mem.chosen["towns"]]
    return ("\n[chosen]\nheroes = [" + ", ".join(rows) + "]\ntowns = [" + ", ".join(towns)
            + "]\n")


def ask_status(mem, kind, end, rebuild=False):
    """Asks the status window for `kind` until the clock time `end` (E24 M3 T5b2; design §4.9),
    the selection's window read first unless a kind asked before read it (`mem.selection`, the
    live part's `selection`: what a timed kind's end builds again); with `rebuild` the window
    built afresh by the next view draw."""
    if not getattr(mem, "selection", None):
        mem.selection = status_toml(mem, mem.current_player())
    mem.ask_status(kind, end, rebuild)


def over_view(point):
    """Whether a point of the screen lies over the view's item (8, 8, 592 x 544)."""
    return 8 <= point[0] < 600 and 8 <= point[1] < 552


def status_bar_text(mem):
    """The status bar's text (the adventure dialog's item 200, E24 M3 T6a1), None when empty."""
    return next((i["text"] for i in mem.dialog_items() if i["id"] == 200), None)


def look_over_view(game, mem, sit, points):
    """The selected hero's costs of the view's tiles under `points` (E24 M3 T6a1; design §4.10):
    the pointer moved over each in turn (`adventure_mouse_move` runs the pathfinder for the
    selected hero to the tile under it, `movement_calculations_mouse` 0x4194A0, which stops once
    that tile is reached: its node holds the cost the cursor's days read), each one's cost read
    (`Mem.node_cost`; one the run did not reach left out, as a tile the cursor finds no path to),
    kept as `mem.reach` for `live_part`; then the pointer over the kingdom overview's button and
    back to the parking (its item looked at again: the status bar's hint as before), the view's
    column and row forgotten (the manager's +0xEC and +0xF0 -1, as after a walk), so that a
    sequence's first move over the view looks at its tile. The clock runs."""
    before = status_bar_text(mem)
    vx, vy, vz = sit["view"]
    reach = {}
    for x, y in points:
        tile = (vx + x // 32, vy + y // 32)
        game.move(x, y)
        h3adv.wait_for(lambda: mem.mouse_tile()[:2] == tile,
                       lambda: f"the mouse over the view's tile {tile}")
        game.clock("step 0", settle_reads=200)
        cost = mem.node_cost(*tile, vz)
        if cost is not None:
            reach[tile] = cost
    mem.reach = reach
    game.move(690, 212)
    game.move(*h3adv.PARK)
    manager = mem.u32(h3mem.ADVENTURE_MANAGER_POINTER)
    mem.write_u32(manager + 0xEC, 0xFFFFFFFF)
    mem.write_u32(manager + 0xF0, 0xFFFFFFFF)
    h3adv.wait_for(lambda: status_bar_text(mem) == before,
                   lambda: f"the status bar's hint as before the look ({status_bar_text(mem)!r})")


def live_part(mem, sit, turn, k, k_rivers, kingdom=None):
    """The live part the screen shows right after a redraw at the frozen clock (the palette steps
    `k`, `k_rivers` read by `cycle_step`), as `render --live` takes it: `live.toml`'s lines. The
    redraw drew with the manager's +0x100 before it grew it (design §1.6): the screen shows the
    step before the one memory holds. The objects in the map's order: their anchors (the viewer
    checks them against its map's) and phases. Each object's template in memory: a random one's
    is the copy a new game made (design §2.7), whose DEF, class and subclass the viewer draws it
    as; a random dwelling's the dwelling's it became, a town's its picture by its buildings
    (§2.9). Each object's owner as the view finds it (T5b1). The viewer's explored tiles and the
    path shown (T5b2). The viewer's kingdom the panels show, move hero's and next hero's states as
    the original last set them and the mouse's point, where the cursor is drawn (M3 T2a); the
    kingdom as `kingdom` gives it when the panels show another than memory's (M3 T4b: a
    sequence's `points` written after the hero list was drawn); the status window shown (M3
    T5b1) and, when a kind was asked, the selection's window read before (M3 T5b2,
    `ask_status`). Checks the heroes and boats against what the situation staged."""
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
            "kingdom": kingdom_toml(kingdom or mem.kingdom(mem.current_player())),
            "status": status_toml(mem, mem.current_player()),
            **({"selection": mem.selection} if getattr(mem, "selection", None) else {}),
            "buttons": mem.buttons(),
            "pointer": mem.cursor()[2:],
            "situation": json.dumps(situation_digest(sit))}
    check_staged(sit, turn, heroes, boats)
    # The game's facts the mouse-over reads (E24 M3 T6a1; design §4.10), a table of its own last.
    hover = mem.hover(mem.current_player(), ids_from("HOTRAITS.TXT", 2),
                      ids_from("TownName.txt", 0), sit["view"][2], getattr(mem, "reach", None))
    # What choosing each hero and town shows (E24 M3 T6b1), a table before the last.
    chosen = chosen_toml(mem, heroes) if getattr(mem, "chosen", None) else ""
    return ("".join(f"{key} = {value}\n" for key, value in live.items()) + chosen
            + "\n[hover]\n"
            + "".join(f"{key} = {json.dumps(value)}\n" for key, value in hover.items()))


def kingdom_toml(kingdom):
    """`Mem.kingdom` as `render --live`'s `kingdom`, an inline TOML table (E24 M3 T2a)."""
    return "{ " + ", ".join(f"{key} = {value}" for key, value in kingdom.items()) + " }"


def ids_from(table, first):
    """A text table's first fields of its non-blank records from `first`, in order: the original
    ids' records as the catalogue's families count them (`h3_ui::text::keys`)."""
    return [r[0] for r in h3lod.table(table)[first:] if any(f.strip() for f in r)]


def status_toml(mem, colour):
    """`Mem.status` as `render --live`'s `status`, an inline TOML table (E24 M3 T5b1; design
    §4.8): a town's name as its record in `TownName.txt` (the catalogue's `town-name` family: the
    harness keeps numbers, never the text, ADR-0009); a hero's `own_name` 1 when its record's name
    is not its type's (a shipped map's hero: the viewer takes the map's). A town whose name the
    table lacks (a map's own) is the harness's failure (the town's own names are M4's), as is a
    kind the viewer does not draw yet (another player's turn, a message's: M4's), since the status
    window is compared. The date's and the kingdom's (M3 T5b2) as `Mem.status` gives them."""
    status = mem.status(colour)
    if status["kind"] not in STATUS_DRAWN:
        raise RuntimeError(f"the status window shows the kind {status['kind']}: not drawn yet")
    if status["kind"] == 3:
        name = mem.hero(status["hero"])["name"]
        status = {"kind": 3, "hero": status["hero"],
                  "own_name": int(ids_from("HOTRAITS.TXT", 2)[status["hero"]] != name),
                  **{k: v for k, v in status.items() if k not in ("kind", "hero")}}
    elif status["kind"] == 4:
        names = ids_from("TownName.txt", 0)
        if status["name"] not in names:
            raise RuntimeError("a town's name is not in TownName.txt")
        status["name"] = names.index(status["name"])
    return "{ " + ", ".join(f"{key} = {value}" for key, value in status.items()) + " }"


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
    mask = masked_for(situation_path.stem, cap.shape)
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


def view_changed(before, after, ident):
    """The box `[x0, y0, x1, y1]` (x1, y1 exclusive) of the pixels that differ between two
    screens outside the panels' mask of sequence `ident` (since E24 M3 T2a the whole screen but
    the panels' mask; since T6a1 `panel_rects`; None: none)."""
    keep = ~masked_for(ident, before.shape)
    return h3drive.changed_box(np.where(keep, before, 0), np.where(keep, after, 0))


def map_timeline(states):
    """The changes over a sequence (E24 M2 T1a1), one line each, numbers only: `<ms> view <x>
    <y> <level>` (the view's top-left tile), `<ms> objects <n>` (the objects' frame steps memory
    holds, the manager's +0x100, counted from the sequence's 0), `<ms> cycles <terrains>
    <rivers>` (the palette steps memory holds, each kind's ticks counted from 0), since T1a2
    `<ms> cursor <type> <frame> <x> <y>` (the cursor's DEF type and frame and the point it is
    drawn at, `Mem.cursor`), since M3 T2a `<ms> buttons <move> <next>` (move hero's and next
    hero's states, 1 enabled: `Mem.buttons`), since M3 T4b `<ms> points <n>` (the selected hero's
    movement points, its record's +0x4D: `Mem.points`; none without one) and `<ms> screen
    <x0>,<y0>,<x1>,<y1>` (where the screen outside the panels' mask changed since the screen kept
    before: each such screen is kept; x1, y1 exclusive).
    The first state, at 0, gives the view, the objects, the cycles, the cursor, the buttons and
    the points; its screen is
    always kept, without a line. Since T1b the walk's lines, only where they change (at 0 they are
    the situation's: the selected hero standing on its tile, the offset 0, its path's arrows):
    `<ms> hero <x> <y> <level> <facing>` (the current player's selected hero, `Mem.selected`),
    `<ms> walk <group> <frame> <x> <y>` (the moving hero's group and the frame pass 9 draws next,
    the view's pixel offset: `Mem.walk`) and `<ms> arrows <x>,<y>,<level>,<arrow> ...` (the path's
    arrows left, `Mem.path_arrows`' order; nothing after `arrows` once none is left)."""
    lines, last = [], {}
    for state in states:
        t = state["t"]
        for key in ("view", "objects", "cycles", "cursor", "buttons", "points"):
            if key in state and last.get(key) != state[key]:
                lines.append(f"{t} {key} " + " ".join(map(str, state[key])))
                last[key] = state[key]
        for key in ("hero", "walk"):
            if key in last and last[key] != state[key]:
                # No hero selected (E24 M3 T6b1: a town chosen): the line without numbers.
                lines.append(f"{t} {key} {' '.join(map(str, state[key]))}".rstrip())
            last[key] = state[key]
        if "arrows" in last and last["arrows"] != state["arrows"]:
            lines.append(f"{t} arrows" + "".join(" " + ",".join(map(str, a))
                                                 for a in state["arrows"]))
        last["arrows"] = state["arrows"]
        if state.get("changed"):
            lines.append(f"{t} screen " + ",".join(map(str, state["changed"])))
    return lines


# A sequence's options (E24 M2 T1a2, design §3.4): written into the game's settings once its
# situation is staged (both are read where they act: the scroll speed at each scroll), with the
# values the original holds (the registry's, settings_read 0x50B7B0; its own check of them,
# 0x50B260, the ranges).
# T1b: `points`, the selected hero's movement points (its record's +0x4D, as if it had walked;
# `None` for the address: written by `Mem.write_points`), so that a walk stops on the way or a hero
# that boarded a boat (which takes the day's points) sails.
OPTIONS = {"scroll_speed": (h3mem.SCROLL_SPEED, range(0, 3)),
           "hero_speed": (h3mem.WALK_SPEED, range(1, 5)),
           "points": (None, range(0, 4000))}
# A sequence's inputs (T1a2), each at a millisecond of the sequence: a key pressed or released
# (xdotool's names, X keysyms, `ctrl+` holding the Control key with it: `ctrl+Up` presses both,
# releases both) and the pointer moved to a point of the screen (800 x 600); since T1b a left
# click at a point (the pointer moved there, the button pressed and released: a tile clicked
# shows the selected hero's path to it, clicked again it walks).
# Since E24 M3 T6b1 the left button pressed and released apart (`press`, `release`: a panel's
# button held down between them).
INPUTS = ("keydown", "keyup", "pointer", "click", "press", "release")
# The keys a sequence may press (M2 T6a, the reviewer's R3-7: xdotool only warns about a name it
# does not know and exits 0, so a typo recorded a sequence without its input): the eight
# direction keys, as `h3_map_view::sequence`'s DIRECTION_KEYS, with or without `ctrl+`; since
# E24 M3 T6b1 the panels' buttons' hotkeys (`h3_map_view::timeline::Hotkey`'s letters).
DIRECTION_KEYS = ("Up", "Prior", "Right", "Next", "Down", "End", "Left", "Home")
HOTKEYS = ("k", "u", "q", "z", "w", "m", "c", "a", "o", "h", "e")


def known_key(name):
    """Whether `name` is a key a sequence may press: a direction key, `ctrl+` before it or not,
    or a hotkey's letter."""
    return name.removeprefix("ctrl+") in DIRECTION_KEYS or name in HOTKEYS


def sequence_inputs(entry):
    """A sequence's options and inputs, checked (T1a2): the options among `OPTIONS` in their
    ranges; each input one of `INPUTS` at a millisecond from 1 to `ms` (0 is the still the frame
    check compares), in order of time, a key released only once pressed, a pointer inside the
    screen. Returns the options [(address, value)] and the inputs by time {t: [(kind, value)]}."""
    unknown = set(entry) - {"id", "situation", "ms", "why", "inputs", "status", *OPTIONS}
    if unknown:
        raise ValueError(f"{entry['id']}: unknown keys {sorted(unknown)}")
    # The status window's kind asked at 0 and its end (E24 M3 T5b2; design §4.9).
    if "status" in entry:
        status = entry["status"]
        if not (len(status) == 2 and status[0] in (1, 2) and 1 <= status[1] <= 600_000):
            raise ValueError(f"{entry['id']}: `status` is [1 or 2, an end from 1 to 600000 ms]: "
                             f"{status}")
    options = []
    for name, (address, values) in OPTIONS.items():
        if name in entry:
            if entry[name] not in values:
                raise ValueError(f"{entry['id']}: `{name}` {entry[name]} not in {values}")
            options.append((address, entry[name]))
    inputs, down, last = {}, set(), 1
    for i in entry.get("inputs", []):
        kinds = [k for k in INPUTS if k in i]
        if len(kinds) != 1 or set(i) != {"at", kinds[0]}:
            raise ValueError(f"{entry['id']}: an input is `at` and one of {INPUTS}: {i}")
        kind, value, t = kinds[0], i[kinds[0]], i["at"]
        if kind in ("keydown", "keyup") and not known_key(value):
            raise ValueError(f"{entry['id']}: the key `{value}` at {t}: not one of {DIRECTION_KEYS} "
                             f"(with `ctrl+` or not) or {HOTKEYS}")
        if not last <= t <= entry["ms"]:
            raise ValueError(f"{entry['id']}: an input at {t}: out of order or outside 1 to "
                             f"{entry['ms']}")
        if kind == "keydown":
            down.add(value)
        elif kind == "keyup":
            if value not in down:
                raise ValueError(f"{entry['id']}: `{value}` released at {t}, never pressed")
            down.discard(value)
        elif not (len(value) == 2 and 0 <= value[0] < h3drive.WIDTH
                  and 0 <= value[1] < h3drive.HEIGHT):
            raise ValueError(f"{entry['id']}: the {kind} at {value}: off the screen")
        inputs.setdefault(t, []).append((kind, value))
        last = t
    return options, inputs


def give_input(game, kind, value):
    """One input to the original (T1a2): xdotool's `keydown`, `keyup` or `mousemove`; since T1b
    a click (`mousemove`, then the left button's `click`, no wait between: the clock stands)."""
    if kind == "pointer":
        game.move(*value)
    elif kind == "click":
        game.move(*value)
        h3drive.run("xdotool", "click", "1")
    elif kind in ("press", "release"):
        # E24 M3 T6b1: the move, then the button's own message.
        game.move(*value)
        h3drive.run("xdotool", "mousedown" if kind == "press" else "mouseup", "1")
    else:
        h3drive.run("xdotool", kind, value)


def sequence(entry, out):
    """One sequence in the original (E24 M2 T1a1): its situation staged as `still` stages it, the
    clock frozen once both the palette tick and the view's redraw lie ahead of it; then the
    sequence's 0 set to the next millisecond: the tick's and the redraw's next times written
    there, and the mouse poll's (the harness's own timers, as E17's `align_mouse_poll`: the
    original's phase between them is the map's load's real time, so a run starts like every
    other; the scroll's last time is the clock's while the pointer is parked). Its options (T1a2: the speeds; T1b: the selected hero's
    points) are written before. Then `ms` + 1 steps of 1 ms (0 to `ms`), each followed by the
    inputs at that millisecond (T1a2; the game then reads the frozen clock 500 times more: it
    handles them at that time)
    and the state memory holds and the screen: every screen whose view (outside the panels)
    changed is kept
    (`<out>/frames/t<ms>.npz`, local), its state in `<out>/frames.jsonl` (every step's in
    `states.jsonl`, with the tick's and the redraw's next times, `due`). Writes the timeline
    (`<out>/timeline.txt`, `map_timeline`), the live part the screen shows at 0
    (`<out>/live.toml`, `live_part`) and the viewer's frame at 0 drawn with it against the
    original's (`<out>/ours.png`, `diff.png`); returns the states and that frame's differing
    pixels."""
    situation_path = SITUATIONS / f"{entry['situation']}.toml"
    options, inputs = sequence_inputs(entry)
    # An earlier run's files go first (M2 T6a, the reviewer's R3-3): a run that failed midway
    # left them beside its own partial screens, and `--reuse` took the mix as one run.
    kept = out / "frames"
    shutil.rmtree(kept, ignore_errors=True)
    for name in ("frames.jsonl", "states.jsonl", "live.toml", "timeline.txt", "ours.png",
                 "diff.png"):
        (out / name).unlink(missing_ok=True)
    game, mem, sit, turn = stage(situation_path, out)
    down = set()
    try:
        kept.mkdir(parents=True)
        # Keys a killed run left down stay down on the display (M2 T6a, the reviewer's R3-4):
        # Control would turn the walks' keys into scrolls.
        h3drive.run("xdotool", "keyup", "ctrl", *DIRECTION_KEYS)
        # The objects' phases fixed (T1b): the load gives each a random one (design §1.6), and
        # where an object's frames repeat a picture the screen's changed box at a redraw moved
        # with it from run to run (the frame steps too: below). The live part at 0 reads them.
        mem.write_phases(0)
        # The kingdom as the panels show it (E24 M3 T4b, design §4.5): the hero list was drawn
        # before the selected hero's `points` are written, and nothing draws it again until a
        # step's end or a lift that centres the view; the live part at 0 keeps it as shown, the
        # timeline's `points` give memory's.
        shown = mem.kingdom(mem.current_player())
        for address, value in options:
            if address is None:
                index, _ = mem.selected()
                if index < 0:
                    raise RuntimeError("`points` without a selected hero")
                mem.write_points(index, value)
            else:
                mem.write_u32(address, value)
        # The reach of the tiles the pointer visits, for the points written (E24 M3 T6a1).
        look_over_view(game, mem, sit, [v for t in sorted(inputs) for k, v in inputs[t]
                                        if k in ("pointer", "click") and over_view(v)])
        # The display's own key repeat is off (T1a2): it repeats a held key in real time, which
        # a step of the virtual clock spans many times over; the original repeats nothing itself
        # (a key held is one press, design §3.4).
        h3drive.run("xset", "r", "off")
        game.clock("step 100")
        now = game.clock("freeze")
        # As `capture`: a deadline the frozen clock passed is met at the next idle step first.
        h3adv.wait_for(lambda: min(mem.map_due()) > now,
                       lambda: f"the deadlines past the frozen clock at {now} ({mem.map_due()})")
        t0 = now + 1
        # The objects' frame steps 0 (T1b): with the phases (above) an object's frame at the
        # redraw at 0 is the same in every run; the count since the load is its real time's.
        mem.write_frame_steps(0)
        mem.write_u32(h3mem.PALETTE_TICK_TIME, t0)
        mem.write_u32(h3mem.FRAME_DEADLINE, t0)
        # The scroll's last time is not written (M2 T6a, the reviewer's R3-10): with the pointer
        # parked outside the edge zones `edge_scroll` sets it to the clock at every call
        # (0x419978), so the 70 ms run from the millisecond the pointer enters a zone.
        mem.align_mouse_poll(t0)
        # The status window's kind a sequence asks (E24 M3 T5b2; design §4.9): until its end
        # after 0, built afresh by the redraw at 0 (the date's from its first picture).
        if "status" in entry:
            kind, end = entry["status"]
            ask_status(mem, kind, t0 + end, rebuild=True)
        if game.clock("step 1", settle_reads=500) != t0:
            raise RuntimeError("the clock did not step to the sequence's 0")
        # The tick and the redraw came at 0 (each moved its next time on), so the screen shows
        # the live part read now (a still's moment, as `capture` steps to).
        h3adv.wait_for(lambda: mem.map_due() == (t0 + 200, t0 + 180),
                       lambda: f"the tick and the redraw at the sequence's 0 ({t0}: next "
                               f"{mem.map_due()})")
        # The date's window steps its picture at the view's draws from when it was built: one
        # built before 0 steps at times the viewer does not know (E24 M3 T5b2).
        shown_kind = mem.status_shown()
        if shown_kind == 1 and entry.get("status", [0])[0] != 1:
            raise RuntimeError("the date's status window at 0 without the sequence's `status`")
        if "status" in entry and shown_kind != entry["status"][0]:
            raise RuntimeError(f"the status window's kind {shown_kind} at 0, not the one asked")
        base = {"objects": mem.frame_steps(), "cycles": cycle_now(mem)}
        counted = {"cycles": base["cycles"], "ticks": (0, 0)}
        live = live_part(mem, sit, turn, *base["cycles"], kingdom=shown)

        def state(t, pix, before):
            k = cycle_now(mem)
            ticks = tuple(n + (now_k - was) % 36 for n, now_k, was
                          in zip(counted["ticks"], k, counted["cycles"]))
            counted.update(cycles=k, ticks=ticks)
            index, hero = mem.selected()
            s = {"t": t - t0, "view": list(mem.view()) + [mem.view_level()],
                 "objects": [mem.frame_steps() - base["objects"]], "cycles": list(ticks),
                 "cursor": mem.cursor(), "due": [d - t0 for d in mem.map_due()],
                 "hero": hero, "walk": mem.walk(), "arrows": mem.path_arrows(),
                 "buttons": mem.buttons()}
            if index >= 0:
                s["points"] = [mem.points(index)]
            box = None if before is None else view_changed(before, pix, entry["id"])
            if before is None or box:
                s["changed"] = box
                s["frame"] = f"t{t - t0:05d}.npz"
                np.savez_compressed(kept / s["frame"], screen=pix)
            return s

        first = game.settled()
        states, last = [state(t0, first, None)], first
        for i in range(entry["ms"]):
            t = game.clock("step 1", settle_reads=500)
            # Each step one millisecond (M2 T6a, the reviewer's R3-6): an input at a skipped one
            # would be lost unseen.
            if t != t0 + i + 1:
                raise RuntimeError(f"the clock stepped to {t - t0}, not {i + 1}")
            for kind, value in inputs.get(t - t0, []):
                give_input(game, kind, value)
                if kind in ("keydown", "keyup"):
                    (down.add if kind == "keydown" else down.discard)(value)
                elif kind in ("press", "release"):
                    (down.add if kind == "press" else down.discard)("button")
                    game.clock("step 0", settle_reads=500)
                elif kind == "click":
                    # The click handled before the next input (T1b): a walk at hero speed 4 ends
                    # in it and reads the pointer (GetCursorPos) at its end, which a pointer
                    # input given at once would race.
                    game.clock("step 0", settle_reads=500)
            if t - t0 in inputs:
                game.clock("step 0", settle_reads=500)
            pix = game.settled()
            states.append(state(t, pix, last))
            if "frame" in states[-1]:
                last = pix
    finally:
        # A key left down stays down on the display for the next game (T1a2); the game stopped
        # whatever the release meets (M2 T6a, the reviewer's R3-4).
        try:
            for value in down:
                if value == "button":
                    h3drive.run("xdotool", "mouseup", "1")
                else:
                    h3drive.run("xdotool", "keyup", value)
            game.move(*h3adv.PARK)
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
    n, _ = h3cap.compare(first, h3cap.png565(out / "ours.png"),
                         masked_for(entry["id"], first.shape), out / "diff.png")
    return states, n


def ours_frames(ident, here, live):
    """The viewer's frames of a sequence (E24 M2 T2): `h3-map-view sequences <id> --frames
    <here> --live <live>` writes `<here>/ours/t<ms>.png` at every time its screen changed and
    every time `<here>/frames/` holds one of the original's, and its timeline (`ours.txt`); its
    own verdict on the timeline is its first line. Returns that line."""
    shutil.rmtree(here / "ours", ignore_errors=True)
    done = subprocess.run([str(MAP_VIEW), "sequences", ident, "--frames", str(here),
                           "--live", str(live)], capture_output=True, text=True)
    said = (done.stdout.strip() or done.stderr.strip()).splitlines()
    if done.returncode not in (0, 1) or not said:
        raise RuntimeError(f"h3-map-view sequences {ident}: {done.stderr.strip()}")
    return said[0]


def frame_check(entry, here):
    """The frame check (E24 M2 T2; the epic's M2 decision): the viewer's frames at the times the
    original kept a screen (`ours_frames` with this run's live part at 0), each against the
    original's outside the panels: `frames.txt`, a line `<ms> <differing pixels>` per kept
    screen, `diff/t<ms>.png` for one that differs (red the differing pixels, blue the masked
    ones), against `timelines/<id>-frames.txt` (`ok`, `CHANGED`, `new`: a new one is copied there
    by hand once a second run gives the same). Returns the verdict, the viewer's line on its
    timeline, the frames and how many differ."""
    said = ours_frames(entry["id"], here, here / "live.toml")
    diffs = here / "diff"
    shutil.rmtree(diffs, ignore_errors=True)
    lines, differ = [], 0
    for npz in sorted((here / "frames").glob("t*.npz")):
        cap = np.load(npz)["screen"]
        png = here / "ours" / f"{npz.stem}.png"
        if not png.exists():
            raise RuntimeError(f"the viewer drew no frame at {npz.stem}")
        ours = h3cap.png565(png)
        mask = masked_for(entry["id"], cap.shape)
        n, _ = h3cap.compare(cap, ours, mask)
        if n:
            diffs.mkdir(exist_ok=True)
            h3cap.compare(cap, ours, mask, diffs / f"{npz.stem}.png")
            differ += 1
        lines.append(f"{int(npz.stem[1:])} {n}")
    text = "\n".join(lines) + "\n"
    (here / "frames.txt").write_text(text)
    recorded = TIMELINES / f"{entry['id']}-frames.txt"
    verdict = ("new" if not recorded.exists()
               else "ok" if recorded.read_text() == text else "CHANGED")
    return verdict, said, len(lines), differ


def reused(entry, here):
    """An earlier run's states and its frame at 0 compared again (`--reuse`, E24 M2 T2): the
    original's screens, states and live part at 0 as that run kept them under `here`."""
    if not all((here / name).exists() for name in ("states.jsonl", "live.toml",
                                                     "frames/t00000.npz")):
        raise RuntimeError(f"no whole earlier run under {here} (run `frames --only "
                           f"{entry['id']}`)")
    sit = tomllib.loads((SITUATIONS / f"{entry['situation']}.toml").read_text())
    # The situation as that run staged it (M2 T6a, the reviewer's R3-3).
    if tomllib.loads((here / "live.toml").read_text()).get("situation") != situation_digest(sit):
        raise RuntimeError(f"STALE: the situation changed since the run under {here} (run "
                           f"`frames --only {entry['id']}`)")
    states = [json.loads(line) for line in (here / "states.jsonl").read_text().splitlines()]
    (here / "timeline.txt").write_text("\n".join(map_timeline(states)) + "\n")
    first = np.load(here / "frames" / "t00000.npz")["screen"]
    draw_ours(here / "map.h3m", sit, here / "live.toml", here / "ours.png")
    n, _ = h3cap.compare(first, h3cap.png565(here / "ours.png"),
                         masked_for(entry["id"], first.shape), here / "diff.png")
    return states, n


def run_frames(rest):
    """`frames [--only <id>] [--out <dir>]`: each sequence of `clients/map/sequences.toml` (or
    one) recorded (`sequence`); its timeline against the one recorded in
    `clients/map/timelines/<id>.txt` (`ok`, `CHANGED`, `new`), and the viewer's frame at 0 against
    the original's (the live part at 0 is a still's: exact, as the capture set). A run as recorded
    or new and exact at 0 keeps its live part at 0 as `timelines/<id>-live.toml` (for the frame
    check, M2 T2). Exit 1 unless every sequence is as recorded, its frame at 0 exact, its frames
    as recorded and the viewer's timeline as recorded."""
    only = rest[rest.index("--only") + 1] if "--only" in rest else None
    base = Path(rest[rest.index("--out") + 1]) if "--out" in rest else OUT_FRAMES
    reuse = "--reuse" in rest
    entries = tomllib.loads(SEQUENCES.read_text())["sequence"]
    if only:
        entries = [e for e in entries if e["id"] == only]
        if not entries:
            sys.exit(f"no sequence `{only}` in {SEQUENCES.name}")
    failed = 0
    for e in entries:
        here = base / e["id"]
        try:
            states, n = reused(e, here) if reuse else sequence(e, here)
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
        if verdict == "CHANGED" or n != 0:
            # Only a run exact at 0 keeps its live part (M2 T6a, the reviewer's R3-5): `drawn`
            # takes it as its reference.
            why = "not as recorded" if verdict == "CHANGED" else "its frame at 0 not exact"
            print(f"{e['id']}: {why}, its live part not kept ({here})", flush=True)
        else:
            TIMELINES.mkdir(exist_ok=True)
            shutil.copy(here / "live.toml", TIMELINES / f"{e['id']}-live.toml")
        # The frame check (T2; the walks since T3b): every kept screen against the viewer's at
        # its time.
        try:
            checked, said, frames, differ = frame_check(e, here)
        except Exception as err:  # noqa: BLE001 (any failure fails this sequence alone)
            failed += 1
            print(f"{e['id']}: the frame check FAILED ({type(err).__name__}: {err})", flush=True)
            continue
        print(f"{e['id']}: the viewer: {said}", flush=True)
        print(f"{e['id']}: the frame check: {frames} frames, {differ} differing, {checked} "
              f"({here / 'frames.txt'})", flush=True)
        # The viewer's own timeline counts too (M2 T6a, the reviewer's R3-1).
        failed += checked != "ok" or not said.endswith("as recorded")
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
        sequences = [e["id"] for e in tomllib.loads(SEQUENCES.read_text())["sequence"]]
        if not entries and only not in sequences:
            sys.exit(f"no situation or sequence `{only}` (set.toml, sequences.toml)")
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
        got[ident] = h3drive.png_digest(here / "ours.png", panel_rects(ident))
        before = recorded.get(ident)
        verdict = "new" if before is None else ("same" if before == got[ident] else "CHANGED")
        failed += verdict != "same"
        print(f"{ident}: {verdict}", flush=True)
    seq_failed, seq_broken = drawn_sequences(rest, base, record)
    failed += seq_failed
    broken += seq_broken
    if record:
        if got:
            recorded.update(got)
            lines = ["# Ours at each situation's last kept live part (h3map.py drawn --record): "
                     "the",
                     "# digest of its pixels outside the masks (h3drive.png_digest, h3map.MASKS),",
                     "# `<id> <sha1, 16 digits>`."]
            lines += [f"{ident} {recorded[ident]}" for ident in sorted(recorded)]
            digests_path.write_text("\n".join(lines) + "\n")
        print(f"drawn: {len(got)} situations recorded in {digests_path}"
              + (f"; {broken} not drawn, their digests left as they were" if broken else ""))
        return 1 if broken else 0
    print(f"drawn: {len(entries)} situations and the sequences, {failed} not the same; under "
          f"{base}")
    return 1 if failed else 0


def drawn_sequences(rest, base, record):
    """`drawn` over the recorded sequences (E24 M2 T2): each one's frames drawn by the viewer from
    its kept live part (`timelines/<id>-live.toml`, written by `frames` from a run as recorded or
    new) at the times its own screen changes (`ours_frames`: `h3-map-view sequences` without the
    original's frames, which also checks the timeline against the recorded one), each frame's
    digest outside the panels against `timelines/drawn.txt` (`<id> t<ms> <digest>`), the walks
    too since M2 T3b. `--only` names a situation or a sequence. Returns how many are not the same and how
    many could not be drawn (with `--record`, the digests of those drawn are written)."""
    only = rest[rest.index("--only") + 1] if "--only" in rest else None
    entries = [e for e in tomllib.loads(SEQUENCES.read_text())["sequence"]
               if only in (None, e["id"])]
    digests_path = TIMELINES / "drawn.txt"
    recorded = {}
    if digests_path.exists():
        for line in digests_path.read_text().splitlines():
            if line and not line.startswith("#"):
                ident, frame, digest = line.split()
                recorded[(ident, frame)] = digest
    got, failed, broken = {}, 0, 0
    for e in entries:
        ident = e["id"]
        kept = TIMELINES / f"{ident}-live.toml"
        sit = tomllib.loads((SITUATIONS / f"{e['situation']}.toml").read_text())
        if not kept.exists() or tomllib.loads(kept.read_text()).get("situation") != \
                situation_digest(sit):
            why = "no kept live part" if not kept.exists() else "STALE (the situation changed)"
            print(f"sequences/{ident}: {why}: `frames --only {ident}`", flush=True)
            failed += 1
            broken += 1
            continue
        here = base / "sequences" / ident
        shutil.rmtree(here, ignore_errors=True)
        here.mkdir(parents=True)
        try:
            said = ours_frames(ident, here, kept)
        except RuntimeError as err:
            print(f"sequences/{ident}: the viewer failed ({last_lines(err)})", flush=True)
            failed += 1
            broken += 1
            continue
        mine = {(ident, png.stem): h3drive.png_digest(png, panel_rects(ident))
                for png in sorted((here / "ours").glob("t*.png"))}
        before = {k: v for k, v in recorded.items() if k[0] == ident}
        verdict = ("new" if not before else "same" if before == mine else "CHANGED")
        timeline_same = said.endswith("as recorded")
        failed += verdict != "same" or not timeline_same
        print(f"sequences/{ident}: {len(mine)} frames {verdict}; the viewer: {said}", flush=True)
        # A viewer's run not as recorded, or one that failed partway, is not recorded (M2 T6a,
        # the reviewer's R3-2): its digests stay as they were.
        if timeline_same:
            got.update(mine)
        else:
            broken += 1
    if record and got:
        kept_ids = {ident for ident, _ in got}
        recorded = {k: v for k, v in recorded.items() if k[0] not in kept_ids}
        recorded.update(got)
        lines = ["# Ours at each sequence's kept live part at the times its screen changes",
                 "# (h3map.py drawn --record; E24 M2 T2): the digest of each frame's pixels",
                 "# outside the panels (h3drive.png_digest), `<id> t<ms> <sha1, 16 digits>`."]
        lines += [f"{i} {f} {recorded[(i, f)]}" for i, f in sorted(recorded)]
        digests_path.write_text("\n".join(lines) + "\n")
        print(f"drawn: {len(got)} frames of {len(kept_ids)} sequences recorded in {digests_path}")
    return failed, broken


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
