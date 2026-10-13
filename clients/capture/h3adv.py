#!/usr/bin/env python3
"""The adventure map in the capture harness (E17 M2 T6, *provisional*; clients/capture/README.md,
"Experiments on the adventure map"): agents play the domain lanes' experiment batches in the
original themselves (docs/experiments/README.md). A hot-seat game with every player human, heroes
selected from the hero list, tiles clicked twice (the first click shows the path, the second
walks), turns ended; what the batch asks is read from the game's memory (h3mem.py), never from
the screen.

  h3adv.py batch <recipe.toml> [--map <map.h3m>] [--out <dir>]
                            plays a batch: compiles its spec (h3-experiment) into the map and the
                            blank results template unless --map is given, checks the map's stamp,
                            starts the map in hot seat and runs the recipe's steps; writes
                            <dir>/<batch>.md (the template filled in), <dir>/steps.jsonl (every
                            step's observation) (default <dir>: clients/out/batches/<batch>)
  h3adv.py compare <ours.md> <theirs.md>
                            the record lines of two filled templates side by side: each field
                            and table row the same or different
  h3adv.py learn-button <name> <x0,y0,x1,y1> [<dump.xwd>]
                            records the button shown in the box now (or in the dump) as
                            [button.<name>] in screens.toml: 12 probes relative to its top-left
                            corner (numbers only)

A recipe (clients/capture/batches/): `spec` (the batch's spec, from the repository's root),
`players` (the hot-seat names, one per human colour from red on) and `[[step]]`s, each either
`end_turn = true` or a hero's: `encounter` (the template's section), `hero` (its name on the map),
`walk = [x, y]` (click the tile twice: a walk, or a visit to what stands there; without it the
step only reads), `record = { "<field or table row>" = "<observation>" }` and `expect = { marks =
<n>, path = "green" | "red" }` (a difference goes into the encounter's "anything odd").
Observations: arrived, moved (yes/no), mark (the target's mark at the first click: green, red or
none), marks (their number), points (the points left), points_if_moved, resource:<name>.

Library use: to_hot_seat, dismiss, end_turn, select, centre, hover, walk, observe. Needs a game
set up by setup.sh (Show Route on), numpy, xdotool and the rights to read the game's memory.
"""

import gzip
import json
import re
import subprocess
import sys
import time
import tomllib
import zlib
from datetime import date
from pathlib import Path

import numpy as np

import h3drive
import h3mem
from h3drive import ROOT, Game

# -- coordinates (800 x 600; the README's table) --------------------------------------------------
MULTIPLAYER = (640, 185)  # new game menu
HOT_SEAT = (580, 152)  # the multiplayer dialog's first button
NAME_FIELDS = (400, 186, 30)  # the hot-seat names: x, the first field's y, the step
NAMES_OK = (340, 444)
BEGIN = (497, 553)  # scenario list
PARK = (700, 300)  # over the right panel: no edge scrolls, no tile hovered
HERO_LIST = (638, 227, 32, 5)  # the hero list's portraits: x, the first one's y, the step, rows
MAP_AREA = (8, 8, 18, 17)  # the map's screen area: its top-left, the tiles across and down
TILE = 32
RADAR = (630, 26, 144)  # the minimap: its top-left and side in pixels
CENTRE = (9, 8)  # a minimap click puts the clicked tile here in the view
QUIET = 1.5  # seconds without a change: a walk or a visit is over


def stamp(spec_path):
    """The spec's stamp (h3_experiment::stamp): the CRC-32 of its text, Windows line endings made
    Unix ones."""
    text = Path(spec_path).read_bytes().replace(b"\r\n", b"\n")
    return f"{zlib.crc32(text) & 0xFFFFFFFF:08x}"


# -- screens and buttons ---------------------------------------------------------------------------
def find_button(game, name, pix=None):
    """The top-left corners of every place the button `name` shows (screens.toml, [button])."""
    probes = game.screens["button"][name]["probes"]
    pix = game.pixels() if pix is None else pix
    dx0, dy0, v0 = probes[0]
    ys, xs = np.nonzero(pix == v0)
    height, width = pix.shape
    found = []
    for y, x in zip(ys - dy0, xs - dx0):
        if all(0 <= y + dy < height and 0 <= x + dx < width and pix[y + dy, x + dx] == v
               for dx, dy, v in probes):
            found.append((int(x), int(y)))
    return found


def click_button(game, name, at):
    width, height = game.screens["button"][name]["size"]
    game.click(at[0] + width // 2, at[1] + height // 2, 0.5)
    game.move(*PARK)


def dismiss(game, quiet=1.5, limit=60.0):
    """Clicks every dialog's OK until none shows for `quiet` seconds on the adventure map;
    returns how many it clicked."""
    clicked, since, end = 0, time.monotonic(), time.monotonic() + limit
    while time.monotonic() < end:
        pix = game.settled()
        found = find_button(game, "ok", pix)
        if found:
            click_button(game, "ok", found[0])
            clicked, since = clicked + 1, time.monotonic()
        elif not game.matches("adventure", pix):
            since = time.monotonic()
        elif time.monotonic() - since >= quiet:
            return clicked
        time.sleep(0.1)
    raise TimeoutError("dialogs keep coming or the adventure map does not show")


# -- the game's start and turns ----------------------------------------------------------------
def to_hot_seat(game, names):
    """From the start to the first human's adventure map of the only map in Maps/, every name a
    human player (hot seat; colours from red on, as the map allows them)."""
    game.wait("main_menu", 90)
    game.click(640, 75, 1.0)  # New Game
    game.wait("new_game_menu")
    game.click(*MULTIPLAYER, 1.0)
    game.wait("multiplayer")
    game.click(*HOT_SEAT, 1.0)
    game.wait("hotseat_names")
    x, y, step = NAME_FIELDS
    for i, name in enumerate(names):
        if i:
            game.click(x, y + step * i, 0.3)
        h3drive.run("xdotool", "type", "--delay", "50", name)
    game.click(*NAMES_OK, 1.0)
    game.wait("scenario_list")
    game.click(*BEGIN, 1.0)
    game.move(*PARK)
    game.wait("turn_dialog", 60)
    dismiss(game)


def end_turn(game, mem):
    """Ends the current player's turn (key e) and passes the dialogs up to the next human's map;
    returns the new current player."""
    player = mem.current_player()
    game.key("e", 0.5)
    end = time.monotonic() + 60
    while mem.current_player() == player:
        if time.monotonic() > end:
            raise TimeoutError("the turn did not end")
        found = find_button(game, "ok")
        if found:
            click_button(game, "ok", found[0])
        time.sleep(0.2)
    dismiss(game)
    return mem.current_player()


# -- the map: the view, tiles, heroes ------------------------------------------------------------
def select(game, mem, name):
    """Selects the current player's hero `name` from the hero list (the first five rows only)."""
    hero = mem.hero_named(name)
    player = mem.player(mem.current_player())
    if hero["owner"] != player["colour"]:
        raise ValueError(f"{name} is not the current player's hero")
    if player["selected"] == hero["index"]:
        return hero
    x, y, step, rows = HERO_LIST
    row = player["heroes"].index(hero["index"])
    if row >= rows:
        raise ValueError(f"{name} is row {row + 1} of the hero list: scrolling it is not built")
    game.click(x, y + step * row, 0.6)
    game.move(*PARK)
    wait_for(lambda: mem.player(player["colour"])["selected"] == hero["index"], f"select {name}")
    return hero


def wait_for(test, what, limit=3.0):
    end = time.monotonic() + limit
    while not test():
        if time.monotonic() > end:
            raise TimeoutError(what)
        time.sleep(0.05)


def centre(game, mem, x, y):
    """Centres the view on tile (x, y) by a click on the minimap."""
    left, top, size = RADAR
    side = mem.i32(mem.game() + h3mem.MAP_SIDE)
    scale = size / side
    game.click(left + int((x + 0.5) * scale), top + int((y + 0.5) * scale), 0.5)
    game.move(*PARK)
    want = (x - CENTRE[0], y - CENTRE[1])
    wait_for(lambda: mem.view() == want, f"the view at {want} (it is at {mem.view()})")


def tile_point(mem, x, y):
    """The screen point of tile (x, y)'s centre; the tile must be in the view."""
    vx, vy = mem.view()
    left, top, across, down = MAP_AREA
    if not (0 <= x - vx < across and 0 <= y - vy < down):
        raise ValueError(f"tile {(x, y)} is outside the view at {(vx, vy)}")
    return left + TILE * (x - vx) + TILE // 2, top + TILE * (y - vy) + TILE // 2


def hover(game, mem, x, y):
    """Puts the mouse on tile (x, y) and waits until the game sees it there."""
    point = tile_point(mem, x, y)
    game.move(*point)
    wait_for(lambda: mem.mouse_tile() == (x, y), f"the mouse on {(x, y)}")
    return point


def observe(mem, name, colour):
    hero = mem.hero_named(name)
    return {"tile": hero["tile"][:2], "points": hero["points"],
            "resources": mem.player(colour)["resources"]}


def settle(game, mem, name, colour):
    """Waits until the hero's tile, its points and its player's resources stay the same for
    QUIET seconds (the walk and any visit over), clicking any dialog's OK on the way; returns
    the observation and the number of dialogs."""
    last, since, dialogs = None, time.monotonic(), 0
    end = time.monotonic() + 120
    while time.monotonic() < end:
        found = find_button(game, "ok")
        if found:
            click_button(game, "ok", found[0])
            dialogs, since = dialogs + 1, time.monotonic()
        now = observe(mem, name, colour)
        if now != last:
            last, since = now, time.monotonic()
        elif time.monotonic() - since >= QUIET:
            return now, dialogs
        time.sleep(0.05)
    raise TimeoutError(f"{name} does not come to rest")


def walk(game, mem, name, target):
    """Selects the hero, clicks the target tile once (the path shows: its marks are read) and
    again (the hero goes, or not), waits until it rests; returns the observation."""
    hero = select(game, mem, name)
    colour = hero["owner"]
    (x0, y0), (x, y) = hero["tile"][:2], target
    centre(game, mem, (x0 + x) // 2, (y0 + y) // 2)
    before = observe(mem, name, colour)
    game.click(*hover(game, mem, x, y), 0.3)
    game.move(*PARK)
    time.sleep(0.3)
    path = mem.path()
    # The game walks on a click on the hero's current destination: a path to this tile set
    # before makes the first click walk.
    first_walked = observe(mem, name, colour) != before
    if not first_walked:
        game.click(*hover(game, mem, x, y), 0.3)
        game.move(*PARK)
    after, dialogs = settle(game, mem, name, colour)
    colours = sorted(set(path.values()))
    return {
        "hero": name, "from": before["tile"], "target": [x, y], "tile": after["tile"],
        "marks": len(path), "mark": path.get((x, y), "none"), "path": colours,
        "arrived": after["tile"] == [x, y], "moved": after["tile"] != before["tile"],
        "points_before": before["points"], "points": after["points"],
        "resources": after["resources"], "dialogs": dialogs, "first_click_walked": first_walked,
    }


# -- a batch ---------------------------------------------------------------------------------
def value(obs, what):
    yes = {True: "yes", False: "no"}
    if what in ("arrived", "moved"):
        return yes[obs[what]]
    if what in ("mark", "marks", "points"):
        return str(obs[what])
    if what == "points_if_moved":
        return str(obs["points"]) if obs["moved"] else ""
    if what.startswith("resource:"):
        return str(obs["resources"][what.split(":", 1)[1]])
    raise ValueError(f"unknown observation `{what}`")


def oddities(obs, expect):
    notes = []
    if "marks" in expect and obs.get("marks") != expect["marks"]:
        notes.append(f"the path to {obs['target']} had {obs['marks']} marks, not {expect['marks']}")
    if "path" in expect and obs.get("path") != [expect["path"]]:
        notes.append(f"the path to {obs['target']} was {'/'.join(obs['path']) or 'not shown'}, "
                     f"not all {expect['path']}")
    if obs.get("dialogs"):
        notes.append(f"{obs['dialogs']} dialog(s) after the walk to {obs['target']}")
    if obs.get("first_click_walked"):
        notes.append(f"the first click on {obs['target']} walked (a path to it was set before)")
    return notes


def fill(template, values, notes, header):
    """The template with each section's fields and table rows filled in."""
    lines, section, out = template.splitlines(), None, []
    for line in lines:
        m = re.match(r"## (\S+) ", line)
        if m:
            section = m.group(1)
        if section is not None:
            m = re.match(r"- (.+): ?$", line)
            if m and (section, m.group(1)) in values:
                line = f"- {m.group(1)}: {values.pop((section, m.group(1)))}"
            elif m and m.group(1) == "anything odd" and notes.get(section):
                line = f"- anything odd: {'; '.join(notes.pop(section))}"
            m = re.match(r"\| (.+?) \|\s*\|$", line)
            if m and (section, m.group(1)) in values:
                line = f"| {m.group(1)} | {values.pop((section, m.group(1)))} |"
        out.append(line)
        if line.startswith("# ") and header:
            out += ["", header]
            header = None
    if values:
        raise ValueError(f"fields the template does not have: {sorted(values)}")
    return "\n".join(out) + "\n"


def compile_spec(spec, out, name):
    """The batch's map and blank template, compiled from its spec by h3-experiment."""
    map_path, template = out / f"{name}.h3m", out / f"{name}-template.md"
    subprocess.run(["cargo", "run", "-q", "--release", "-p", "h3-experiment", "--", str(spec),
                    "--install", str(ROOT / "original-data" / "install"), "--map", str(map_path),
                    "--template", str(template)], cwd=ROOT, check=True)
    return map_path, template


def batch(recipe_path, map_path=None, out=None):
    recipe_path = Path(recipe_path).resolve()
    recipe = tomllib.loads(recipe_path.read_text())
    spec = ROOT / recipe["spec"]
    name = spec.stem
    out = Path(out).resolve() if out else ROOT / "clients" / "out" / "batches" / name
    out.mkdir(parents=True, exist_ok=True)
    if map_path:
        map_path, template = Path(map_path).resolve(), None
    else:
        map_path, template = compile_spec(spec, out, name)
    want = stamp(spec)
    if f"Stamp {want}.".encode() not in gzip.decompress(map_path.read_bytes()):
        raise SystemExit(f"{map_path} is not a build of {spec} (stamp {want}): compile it again")
    if template is None:
        template = out / f"{name}-template.md"
        compile_spec(spec, out, name)  # the template; the given map stays the one played
    values, notes, steps = {}, {}, []
    log = open(out / "steps.jsonl", "w")
    game = Game(out)
    try:
        game.start(map_path)
        to_hot_seat(game, recipe["players"])
        mem = h3mem.Mem()
        for step in recipe["step"]:
            if step.get("end_turn"):
                obs = {"end_turn": True, "player": end_turn(game, mem)}
            elif "walk" in step:
                obs = walk(game, mem, step["hero"], step["walk"])
            else:
                hero = select(game, mem, step["hero"])
                obs = observe(mem, step["hero"], hero["owner"])
            obs["encounter"] = step.get("encounter")
            steps.append(obs)
            log.write(json.dumps(obs) + "\n")
            log.flush()
            print(json.dumps(obs), flush=True)
            for field, what in step.get("record", {}).items():
                values[(step["encounter"], field)] = value(obs, what)
            for note in oddities(obs, step.get("expect", {})):
                notes.setdefault(step["encounter"], []).append(note)
    finally:
        game.stop()
        log.close()
    header = (f"> Played by the capture harness on {date.today().isoformat()} "
              f"(`clients/capture/h3adv.py batch {recipe_path.relative_to(ROOT)}`): "
              "the same Complete 4.0 `Heroes3.exe` under Wine, without the HD mod; hot seat, "
              "the scenario list's defaults (difficulty Normal, starting bonus random), Show "
              "Route on; read from the game's memory.")
    filled = fill(Path(template).read_text(), values, notes, header)
    result = out / f"{name}.md"
    result.write_text(filled)
    print(f"batch: {len(steps)} steps; {result}")
    return result


def records(text):
    """A filled template's record lines: {(section, field or row label): value}."""
    found, section, recording = {}, None, False
    for line in text.splitlines():
        m = re.match(r"## (\S+) ", line)
        if m:
            section, recording = m.group(1), False
        elif line.strip() == "**Record**":
            recording = True
        elif recording and (m := re.match(r"- (.+?): ?(.*)$", line)):
            found[(section, m.group(1))] = m.group(2).strip()
        elif recording and (m := re.match(r"\| (.+?) \|\s*(.*?)\s*\|$", line)):
            if m.group(1) != "pile":
                found[(section, m.group(1))] = m.group(2)
    return found


def compare(ours, theirs):
    a, b = records(Path(ours).read_text()), records(Path(theirs).read_text())
    keys = [k for k in a if k in b and (a[k] or b[k])]
    same = 0
    for key in keys:
        if a[key] == b[key]:
            same += 1
            print(f"same      {key[0]} | {key[1]}: {a[key]}")
        else:
            print(f"DIFFERENT {key[0]} | {key[1]}: {a[key]!r} against {b[key]!r}")
    print(f"compare: {same} of {len(keys)} filled records the same")


def learn_button(name, box, dump=None):
    """12 probes on a grid inside the box, relative to its top-left corner; the button must show
    exactly once (on the screen now, or in the dump)."""
    x0, y0, x1, y1 = box
    game = Game(ROOT / "clients" / "out" / "learn")
    pix = h3drive.xwd565(Path(dump).read_bytes()) if dump else game.pixels()
    probes = [(dx, dy, int(pix[y0 + dy, x0 + dx]))
              for dy in np.linspace(4, y1 - y0 - 4, 3).astype(int)
              for dx in np.linspace(6, x1 - x0 - 6, 4).astype(int)]
    screens = h3drive.load_screens()
    screens.setdefault("button", {})[name] = {"size": [x1 - x0, y1 - y0], "probes": probes}
    game.screens = screens
    found = find_button(game, name, pix)
    if found != [(x0, y0)]:
        raise SystemExit(f"the probes find the button at {found}, not only at {(x0, y0)}")
    h3drive.write_screens(screens)
    print(f"button {name}: {len(probes)} probes")


def main(args):
    if len(args) >= 2 and args[0] == "batch":
        opts = dict(zip(args[2::2], args[3::2]))
        if set(opts) - {"--map", "--out"} or len(args) % 2:
            sys.exit(__doc__)
        batch(args[1], opts.get("--map"), opts.get("--out"))
    elif len(args) == 3 and args[0] == "compare":
        compare(args[1], args[2])
    elif len(args) in (3, 4) and args[0] == "learn-button":
        learn_button(args[1], tuple(map(int, args[2].split(","))), *args[3:])
    else:
        sys.exit(__doc__)


if __name__ == "__main__":
    main(sys.argv[1:])
