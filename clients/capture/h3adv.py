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
import h3lod
import h3mem
from h3drive import ROOT, Game

# -- coordinates (800 x 600; the README's table) --------------------------------------------------
MULTIPLAYER = (640, 185)  # new game menu
HOT_SEAT = (580, 152)  # the multiplayer dialog's first button
NAME_FIELDS = (400, 186, 30)  # the hot-seat names: x, the first field's y, the step
NAMES_OK = (340, 444)
BEGIN = (497, 553)  # scenario list
DIFFICULTY_BUTTONS = (521, 478, 32)  # the scenario list's player difficulty: Easy's centre, the step
PARK = (700, 300)  # over the right panel: no edge scrolls, no tile hovered
HERO_LIST = (638, 227, 32, 5)  # the hero list's portraits: x, the first one's y, the step, rows
LIST_UP, LIST_DOWN = (638, 203), (638, 379)  # the hero list's arrows (E17 H2)
MAP_AREA = (8, 8, 18, 17)  # the map's screen area: its top-left, the tiles across and down
TILE = 32
RADAR = (630, 26, 144)  # the minimap: its top-left and side in pixels
LEVEL_BUTTON = (725, 211)  # the panel's surface/underground button (E17 H2)
CENTRE = (9, 8)  # a minimap click puts the clicked tile here in the view
QUIET = 1.5  # seconds without a change: a walk or a visit is over
STRANGE = 30  # seconds on an unknown screen without a dialog: the batch stops


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
def to_hot_seat(game, names, difficulty="normal"):
    """From the start to the first human's adventure map of the only map in Maps/, every name a
    human player (hot seat, at least two; colours from red on, as the map allows them), the
    players' difficulty chosen in the scenario list (E17 H1; its default Normal)."""
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
    x, y, step = DIFFICULTY_BUTTONS
    game.click(x + step * h3mem.DIFFICULTIES.index(difficulty), y, 0.6)
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
    """Selects the current player's hero `name` from the hero list. With more than five heroes
    (E17 H2) the list is scrolled to its top first (three clicks on its up arrow: eight heroes at
    most), then down until the hero's row shows; the selection is checked in memory."""
    hero = mem.hero_named(name)
    player = mem.player(mem.current_player())
    if hero["owner"] != player["colour"]:
        raise ValueError(f"{name} is not the current player's hero")
    if player["selected"] == hero["index"]:
        return hero
    x, y, step, rows = HERO_LIST
    row = player["heroes"].index(hero["index"])
    if len(player["heroes"]) > rows:
        for _ in range(len(player["heroes"]) - rows):
            game.click(*LIST_UP, 0.4)
        for _ in range(max(0, row - rows + 1)):
            game.click(*LIST_DOWN, 0.4)
        row = min(row, rows - 1)
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


def show_level(game, mem, level):
    """Shows the surface (0) or the underground (1) by the panel's level button (E17 H2)."""
    if mem.view_level() == level:
        return
    if level >= mem.levels():
        raise ValueError(f"the map has no level {level}")
    game.click(*LEVEL_BUTTON, 0.5)
    game.move(*PARK)
    wait_for(lambda: mem.view_level() == level, f"the view on level {level}")


def centre(game, mem, x, y, level=0):
    """Centres the view on tile (x, y) of a level by a click on the minimap (which shows the level
    the view shows)."""
    show_level(game, mem, level)
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


def hover(game, mem, x, y, level=0):
    """Puts the mouse on tile (x, y) of the level the view shows and waits until the game sees it
    there."""
    point = tile_point(mem, x, y)
    game.move(*point)
    wait_for(lambda: mem.mouse_tile() == (x, y, level), f"the mouse on {(x, y, level)}")
    return point


def observe(mem, index):
    """The hero by index (its type, which stays when it leaves the map) and every player's
    resources: what a step waits on and records."""
    hero = mem.hero(index)
    return {"tile": hero["tile"], "points": hero["points"], "owner": hero["owner"],
            "resources": {colour: mem.player(colour)["resources"] for colour in range(8)
                          if mem.player(colour)["heroes"] or colour == hero["owner"]}}


# -- questions and battles (E17 H1) -----------------------------------------------------------
# A question's kind by the record its text was written from (ADVEVENT.TXT; survey E08-battles §3.2):
# the monster's offer to join for free, to join for gold (one record, or two together), to flee.
KINDS = {"ADVEVENT:86": "join", "ADVEVENT:88": "join for gold", "ADVEVENT:89+90": "join for gold",
         "ADVEVENT:91": "flee"}
BATTLE_PARK = (420, 580)  # over the battle's panel, off the battlefield
SIDES = ("attacker", "defender")
BATTLE_KEYS = {"retreat": "r", "surrender": "s"}
# The level-up dialog's two offered skills from its OK's corner, the OK disabled until one is
# chosen (E17 H2, `[button.ok_disabled]`): a step's `level_up`, default the left one.
LEVEL_UP_CHOICES = {"left": (-150, -64), "right": (-52, -64)}


def reply(answer, kind):
    """The step's answer to a question of `kind`: `answer` itself ("yes", "no") or its entry for
    the kind (`other` for any other); None when it has none."""
    if isinstance(answer, dict):
        return answer.get(kind, answer.get("other"))
    return answer


def stacks(mem):
    """The battle's stacks: [side, creature, count, hex] in the records' order."""
    return [[s["side"], s["creature"], s["count"], s["hex"]] for s in mem.stacks()]


def auto_combat(game, mem, end, stall=10.0):
    """Turns auto combat on once the battle takes orders: a key before a stack is active is lost
    (a game's first battle starts more slowly: E08 T22a3, every run of batch E3b). The key goes
    again when the same stack stays active with every stack unchanged for `stall` seconds (auto
    combat's computer acts in the same millisecond); never while auto combat runs, which a
    second key would stop."""
    while True:
        while mem.globals()["active"] is None:
            if find_button(game, "battle_ok"):
                return  # over before any stack's turn
            if time.monotonic() > end:
                raise TimeoutError("no stack's turn for auto combat")
            time.sleep(0.1)
        game.key("a", 0.5)
        state = (mem.globals()["active"], stacks(mem))
        quiet = time.monotonic() + stall
        while (mem.globals()["active"], stacks(mem)) == state:
            if time.monotonic() > quiet:
                break  # the key was lost: press it again
            if time.monotonic() > end:
                raise TimeoutError("auto combat does not start")
            time.sleep(0.2)
        else:
            return


def fight(game, mem, plan, limit=300.0):
    """Plays the battle on the screen as `plan` says and returns its record: the stacks at the
    start and the end, the outcome and how it was played. `plan`: "auto" (auto combat: the
    computer plays every human side; setup.sh's `Combat Auto Creatures`), or "retreat" or
    "surrender", optionally "defender retreat" or "defender surrender" (default the attacker):
    every other stack defends until one of that side's stacks acts, which then retreats or
    surrenders; the confirmation's OK is clicked."""
    if plan is None:
        raise RuntimeError("a battle the step does not plan (`battle`)")
    words = plan.split()
    action, side = words[-1], SIDES.index(words[0]) if len(words) == 2 else 0
    if action not in ("auto", *BATTLE_KEYS) or len(words) > 2:
        raise ValueError(f"unknown battle plan `{plan}`")
    game.move(*BATTLE_PARK)
    wait_for(lambda: mem.stacks(), "the battle's stacks", 10)
    time.sleep(0.5)
    record = {"plan": plan, "start": stacks(mem)}
    end = time.monotonic() + limit
    if action == "auto":
        auto_combat(game, mem, end)
    else:
        while True:
            if time.monotonic() > end:
                raise TimeoutError(f"no stack of the {SIDES[side]} acted")
            active = mem.globals()["active"]
            if active is None or find_button(game, "battle_ok"):
                time.sleep(0.1)
                if find_button(game, "battle_ok"):
                    break  # over before that side acted
                continue
            if active[0] == side:
                game.key(BATTLE_KEYS[action], 1.0)
                found = find_button(game, "ok")
                if not found:
                    raise RuntimeError(f"no confirmation after the {action} key")
                click_button(game, "ok", found[0])
                game.move(*BATTLE_PARK)
                break
            game.key("d", 0.3)
            wait_for(lambda a=active: mem.globals()["active"] != a, "the next stack's turn", 30)
    while not find_button(game, "battle_ok"):
        if time.monotonic() > end:
            raise TimeoutError("the battle does not end")
        time.sleep(0.2)
    time.sleep(0.3)
    record["end"] = stacks(mem)
    record.update(mem.battle_outcome())
    click_button(game, "battle_ok", find_button(game, "battle_ok")[0])
    return record


def settle(game, mem, index, answer=None, battle=None, level_up="left"):
    """Waits until the hero, its points and the players' resources stay the same for QUIET
    seconds on the adventure map (the walk and any visit over), on the way answering each question
    as `answer` says, playing each battle as `battle` says and clicking any other dialog's OK;
    returns the observation and what happened: the dialogs passed, the questions (record, kind,
    answer), the battles (fight's records) and the level-ups (the skill chosen: `level_up`)."""
    events = {"dialogs": 0, "questions": [], "battles": [], "level_ups": []}
    last, since, known = None, time.monotonic(), time.monotonic()
    end = time.monotonic() + 600
    while time.monotonic() < end:
        pix = game.pixels()
        if game.matches("adventure", pix) or game.matches("battle", pix):
            known = time.monotonic()
        elif time.monotonic() - known > STRANGE:
            bad = game.out / "not-adventure.xwd"
            game.dump(bad)
            raise TimeoutError(f"neither the adventure map, a battle nor a dialog for {STRANGE} s "
                               f"(the game over?; the display: {bad})")
        if game.matches("battle", pix):
            events["battles"].append(fight(game, mem, battle))
            last, since = None, time.monotonic()
            continue
        waiting = find_button(game, "ok_disabled", pix)
        if waiting:
            dx, dy = LEVEL_UP_CHOICES[level_up]
            game.click(waiting[0][0] + dx, waiting[0][1] + dy, 0.5)
            game.move(*PARK)
            events["level_ups"].append(level_up)
            last, since, known = None, time.monotonic(), time.monotonic()
            continue
        oks, cancels = find_button(game, "ok", pix), find_button(game, "cancel", pix)
        if oks or cancels:
            known = time.monotonic()
        if oks and cancels:
            record = h3lod.identify(mem.dialog_text())
            kind = KINDS.get(record, "other")
            said = reply(answer, kind)
            if said not in ("yes", "no"):
                raise RuntimeError(f"a question ({record or 'its text not identified'}, {kind}) "
                                   "the step does not answer (`answer`)")
            events["questions"].append({"record": record, "kind": kind, "answer": said})
            click_button(game, *(("ok", oks[0]) if said == "yes" else ("cancel", cancels[0])))
            last, since = None, time.monotonic()
            continue
        if oks:
            click_button(game, "ok", oks[0])
            events["dialogs"] += 1
            last, since = None, time.monotonic()
            continue
        now = observe(mem, index)
        if now != last or not game.matches("adventure", pix):
            last, since = now, time.monotonic()
        elif time.monotonic() - since >= QUIET:
            return now, events
        time.sleep(0.05)
    raise TimeoutError(f"hero {index} does not come to rest")


def walk(game, mem, name, target, answer=None, battle=None, level_up="left"):
    """Selects the hero, clicks the target tile once (the path shows: its marks are read) and
    again (the hero goes, or not, or visits what stands there), waits until it rests, answering
    questions and playing battles on the way (settle); returns the observation."""
    hero = select(game, mem, name)
    index = hero["index"]
    x0, y0, z0 = hero["tile"]
    x, y, z = (*target, 0) if len(target) == 2 else target
    # Both ends in the view when they share a level, else the target alone (E17 H2).
    centre(game, mem, *(((x0 + x) // 2, (y0 + y) // 2) if z == z0 else (x, y)), z)
    before = observe(mem, index)
    game.click(*hover(game, mem, x, y, z), 0.3)
    game.move(*PARK)
    time.sleep(0.3)
    path = mem.path()
    # The game walks on a click on the hero's current destination: a path to this tile set
    # before makes the first click walk.
    first_walked = observe(mem, index) != before
    if not first_walked:
        game.click(*hover(game, mem, x, y, z), 0.3)
        game.move(*PARK)
    after, events = settle(game, mem, index, answer, battle, level_up)
    colours = sorted(set(path.values()))
    return {
        "hero": name, "index": index, "from": before["tile"], "target": list(target),
        "tile": after["tile"], "marks": len(path), "mark": path.get((x, y, z), "none"),
        "path": colours, "arrived": after["tile"] == [x, y, z],
        "moved": after["tile"] != before["tile"], "points_before": before["points"],
        "points": after["points"], "first_click_walked": first_walked, **events,
    }


# -- a batch ---------------------------------------------------------------------------------
COLOURS = ("red", "blue", "tan", "green", "orange", "purple", "teal", "pink")
LEVELS = ("none", "basic", "advanced", "expert")


def snapshot(mem, index):
    """What a step's record may read before and after it: the hero by index and every player's
    resources by colour."""
    hero = mem.hero(index)
    players = {COLOURS[c]: mem.player(c)["resources"] for c in range(8) if mem.player(c)["heroes"]
               or c == hero["owner"]}
    return {"hero": hero, "resources": players}


def army_text(mem, army):
    return ", ".join(f"{count} {mem.creature_name(creature, count)}" for creature, count in army
                     if count) or "none"


def hero_value(mem, hero, field):
    """A hero's detail (`hero:<field>`): experience, level, primary (attack/defense/power/
    knowledge), army, skill:<name> (none, basic, advanced, expert), on_map (yes/no), tile."""
    if field == "experience":
        return str(hero["experience"])
    if field == "level":
        return str(hero["hero_level"])
    if field == "primary":
        return "/".join(map(str, hero["primary"]))
    if field == "army":
        return army_text(mem, hero["army"])
    if field.startswith("skill:"):
        name = field.split(":", 1)[1]
        if name not in h3mem.SKILLS:
            raise ValueError(f"unknown secondary skill `{name}`")
        return LEVELS[hero["skills"].get(name, 0)]
    if field == "on_map":
        return {True: "yes", False: "no"}[hero["owner"] >= 0]
    if field == "tile":
        return ",".join(map(str, hero["tile"]))
    raise ValueError(f"unknown hero detail `{field}`")


def pool_experience(mem):
    """The experience of the heroes off the map (E3-7: the start's draws): their number, the
    lowest and the highest, and how many hold each of these."""
    found = [h["experience"] for h in mem.all_heroes() if h["owner"] < 0]
    if not found:
        return "none"
    low, high = min(found), max(found)
    return (f"{len(found)} heroes: {low} to {high}; {found.count(low)} at {low}, "
            f"{found.count(high)} at {high}")


def stacks_text(mem, stacks, side=None):
    """A battle's stacks by side (`attacker 20 Archangels | defender 3 Pikemen, 2 Halberdiers`),
    or one side's."""
    sides = [side] if side is not None else [0, 1]
    parts = []
    for s in sides:
        text = army_text(mem, [(c, n) for sd, c, n, _ in stacks if sd == s])
        parts.append(text if side is not None else f"{SIDES[s]} {text}")
    return " | ".join(parts)


def value(obs, what, mem):
    """A record's value from a step's observation (the game's memory right after the step for
    `monster:`, `aggression:` and another hero's details)."""
    yes = {True: "yes", False: "no"}
    if what.startswith("before:"):
        return value({"before": obs["before"], "after": obs["before"]}, what.split(":", 1)[1], mem)
    after = obs["after"]
    if what in ("arrived", "moved"):
        return yes[obs[what]]
    if what == "points" and "points" not in obs:
        return str(after["hero"]["points"])
    if what in ("mark", "marks", "points"):
        return str(obs[what])
    if what == "points_if_moved":
        return str(obs["points"]) if obs["moved"] else ""
    if what.startswith("resource:"):
        _, name, *colour = what.split(":")
        owner = colour[0] if colour else COLOURS[obs["before"]["hero"]["owner"]]
        return str(after["resources"][owner][name])
    if what == "difficulty":
        return mem.difficulty()
    if what == "date":
        return "month {}, week {}, day {}".format(*mem.date())
    if what == "level_ups":
        return str(len(obs.get("level_ups", [])))
    if what == "heroes:experience":
        return pool_experience(mem)
    if what == "question":
        return ", ".join(q["kind"] for q in obs.get("questions", [])) or "none"
    if what == "questions":
        return ", ".join(q["record"] or "?" for q in obs.get("questions", [])) or "none"
    battles = obs.get("battles", [])
    if what == "battle":
        return yes[bool(battles)]
    if what.split(":")[0] in ("stacks", "hexes", "winner", "retreated", "surrendered"):
        if not battles:
            return "no battle"
        fought = battles[-1]
        kind, *rest = what.split(":")
        if kind in ("winner", "retreated", "surrendered"):
            # A winner of -1 is none (E17 M3 T6c-2: it read `defender`, SIDES[-1]).
            value = fought[kind]
            return "none" if value is None or value < 0 else SIDES[value]
        moment, side = rest[0], SIDES.index(rest[1]) if len(rest) > 1 else None
        if kind == "stacks":
            return stacks_text(mem, fought[moment], side)
        return ", ".join(str(h) for sd, _, _, h in fought[moment] if side in (None, sd))
    if what.startswith("hero:"):
        return hero_value(mem, after["hero"], what.split(":", 1)[1])
    if m := re.match(r"hero\[(.+)\]:(.+)$", what):
        found = [h for h in mem.all_heroes() if h["name"] == m.group(1)]
        if len(found) != 1:
            raise ValueError(f"{len(found)} heroes named {m.group(1)!r}")
        return hero_value(mem, found[0], m.group(2))
    if m := re.match(r"(monster|aggression|never_flees):(\d+),(\d+)(?:,(\d))?$", what):
        found = mem.monster(int(m.group(2)), int(m.group(3)), int(m.group(4) or 0))
        if found is None:
            return "none"
        if m.group(1) == "monster":
            return army_text(mem, [(found["creature"], found["count"])])
        return str(found["aggression"]) if m.group(1) == "aggression" else yes[found["never_flees"]]
    raise ValueError(f"unknown observation `{what}`")


def oddities(obs, expect):
    notes = []
    if "marks" in expect and obs.get("marks") != expect["marks"]:
        notes.append(f"the path to {obs['target']} had {obs['marks']} marks, not {expect['marks']}")
    if "path" in expect and obs.get("path") != [expect["path"]]:
        notes.append(f"the path to {obs['target']} was {'/'.join(obs['path']) or 'not shown'}, "
                     f"not all {expect['path']}")
    if obs.get("dialogs") and not expect.get("dialogs"):
        notes.append(f"{obs['dialogs']} dialog(s) after the walk to {obs['target']}")
    if obs.get("first_click_walked"):
        notes.append(f"the first click on {obs['target']} walked (a path to it was set before)")
    if "question" in expect:
        asked = ", ".join(q["kind"] for q in obs.get("questions", [])) or "none"
        if asked != expect["question"]:
            notes.append(f"the questions were {asked}, not {expect['question']}")
    if "battle" in expect and bool(obs.get("battles")) != expect["battle"]:
        notes.append(f"{'a' if obs.get('battles') else 'no'} battle")
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


def shown(path):
    """A path as the repository names it (absolute outside it)."""
    return path.relative_to(ROOT) if path.is_relative_to(ROOT) else path


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
        difficulty = recipe.get("difficulty", "normal")
        to_hot_seat(game, recipe["players"], difficulty)
        mem = h3mem.Mem()
        if mem.difficulty() != difficulty:
            raise RuntimeError(f"the game's difficulty is {mem.difficulty()}, not {difficulty}")
        for step in recipe["step"]:
            if step.get("end_turn"):
                obs = {"end_turn": True, "player": end_turn(game, mem)}
            else:
                index = select(game, mem, step["hero"])["index"]
                before = snapshot(mem, index)
                obs = (walk(game, mem, step["hero"], step["walk"], step.get("answer"),
                            step.get("battle"), step.get("level_up", "left"))
                       if "walk" in step else {"hero": step["hero"]})
                obs.update(before=before, after=snapshot(mem, index))
            obs["encounter"] = step.get("encounter")
            obs["record"] = {field: value(obs, what, mem)
                             for field, what in step.get("record", {}).items()}
            steps.append(obs)
            log.write(json.dumps(obs) + "\n")
            log.flush()
            print(json.dumps(obs), flush=True)
            for field, got in obs["record"].items():
                values[(step["encounter"], field)] = got
            for note in oddities(obs, step.get("expect", {})):
                notes.setdefault(step["encounter"], []).append(note)
    finally:
        game.stop()
        log.close()
    header = (f"> Played by the capture harness on {date.today().isoformat()} "
              f"(`clients/capture/h3adv.py batch {shown(recipe_path)}`): "
              "the same Complete 4.0 `Heroes3.exe` under Wine, without the HD mod; hot seat, "
              f"difficulty {recipe.get('difficulty', 'normal').capitalize()}, the starting bonus "
              "random, Show Route on; read from the game's memory.")
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
