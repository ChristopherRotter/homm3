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
`walk = [x, y]` (click the tile twice: a walk, or a visit to what stands there), `equip = {
"<artifact>" = "<slot>" }` (put on from the backpack in the hero screen, E08 T37a) or `screen =
"hero"` (E08 T47a; without any of them the step only reads), `actions = [{ click = [<item>, …] |
right = "<item>" | slider = "max", answer, record, note }]` (E08 T47a: played in the hero
screen, or in the exchange or army dialog a walk opens; items by name, SCREEN_ITEMS), `record = { "<field, table row or `<row> | <column>`>" =
"<observation>" }` and `expect = { marks = <n>, path = "green" | "red" }` (a difference goes into
the encounter's "anything odd"). Observations (the README's table): arrived, moved (yes/no), mark
(the target's mark at the first click: green, red or none), marks (their number), points (the
points left), points_if_moved, resource:<name>, explored:x,y[:colour] (E08 T37a), hero:<detail>, …
Since E08 T57 (batch E7): `choice` (a choice between two pictures: left or right), `once` (the
first click only), `days` (wait for the hero's next turns), `each` with `walk = "each"` and
`select` (a grid of tiles), a list of observations per field, `+<field>`, `[[require]]` (a new game
started again until its draws hold an artifact in each grid), and the observations word:x,y,
object:x,y, choice, message_pictures, level_up_dialogs, treasury, offering:<skill>, map_kind,
opening:rows, morale:start, luck:start (the README's "Rewards on the map"). Since E08 T68a
(batch E8a): `key` (a key pressed for the selected hero), `again_if = { marks = <n> }` (the second
click only after that many marks), `only_if = "aboard"`, and the observations boat, boat[<hero>],
boat:x,y, boats:used, drawn, opening:field, battlefield:ship, blocked, again, key and a Flotsam's
object:x,y (the README's "Boats and the sea"). Since E08 T79b (batch E9b): `gold = true` (the
starting bonus Gold), an end-turn step's `record` and `note` (the new turn's messages), a repeated
row label `<label>#<n>`, and the observations income, gardens, growth:x,y, gain and the weekly
objects' object:x,y, through a hero on the entrance (the README's "The day's start"). Since E09
T3b (batch TWN-1): `new_game = true` (the map started anew), a battle table's list of creatures,
a literal ending in `: ` labelling the value after it, the town screen (noted and closed), and
the observations town:x,y:<part>, towns:every_vs_built, towns:owned_garrisons,
hero:slot:<n>, opening:slot:<n>:<side>, town_screen and unknown_screens (the README's "Towns").
Since E08 T90 (batches E10a, E10b): the hire dialog as a screen (`click = ["max", "recruit"]`,
`amount = <n>`), a step's `hint = [x, y]` and `popup = [x, y]`, a computer player's turn waited out
by `end_turn`, and the observations dwelling:x,y, teams, objects:x0,y0:x1,y1, access:x0,y0:x1,y1,
camp:x,y, hire[:opening|after][:<part>], hint, popup, hint:raw, opening:defender and ai_turns (the
README's "Dwellings, the hire dialog and a computer player's turn"). Since E08 T100a (batch E11a):
a step's `replay` (Replay Opponent Turn on the frozen clock) and `at_opening`, `only_if = { at =
[x, y] }`, `again_if = { marks = "any" }`, a `key` step's `answer` and `battle`, and the
observations replay, replay:views, replay:watched, replay:options, view:before, view:after,
view:size, explored:x0,y0..x1,y1, at_opening:<observation>, battles, exchange, army:rows and
hero:in_boat (the README's "Teleporters and Replay Opponent Turn"). Since E09 T14b (batch TWN-2):
a walk's `actions` on the town screen, the actions `hall = true` and `slot = <building id>`, the
hall and a building's dialog as screens (`click = ["build"]`), and the observations
town:x,y:mark, hall:slot:<id>, build:button, build:text and screen (the README's "Towns").
Since E08 T110 (batches E12a, E12b): a garrison's exchange as a screen (`garrison`), an action's
`row:` and `+` fields, the battle plan `{ book = "<spell>", then }`, a step's `cast`, a list of
`hint` tiles, `skip_quietly`, an end-turn step's `to_day`, the hero screen's `morale` and `luck`,
and the observations garrison:x,y[:part], garrison_dialogs, bar:selected[:before], bar:clicks,
creature_windows, anti_magic, book:<side>, fountain:x,y, spring:x,y, hero:bit:<n>,
hero:word:<offset>, cast, hints, popup:lines and dialog:items|boxes (the README's "Garrisons and
the bonus visitables").

Library use: to_hot_seat, dismiss, end_turn, select, centre, hover, walk, observe,
open_hero_screen, equip; since E08 T47a screen_kind, click_item, right_click, run_actions,
hero_screen; since E08 T57 object_text, held_artifact, choice_sides, pictures_text, expand. Needs a game
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
TOWN_LIST = (771, 227, 32, 5)  # the town list's pictures (E24 M3 T4a), as HERO_LIST
TOWN_UP, TOWN_DOWN = (771, 203), (771, 379)  # the town list's arrows
MAP_AREA = (8, 8, 18, 17)  # the map's screen area: its top-left, the tiles across and down
TILE = 32
RADAR = (630, 26, 144)  # the minimap: its top-left and side in pixels
LEVEL_BUTTON = (725, 211)  # the panel's surface/underground button (E17 H2)
CENTRE = (9, 8)  # a minimap click puts the clicked tile here in the view
QUIET = 1.5  # seconds without a change: a walk or a visit is over
AI_TURN_LIMIT = 240  # seconds a computer player's turn may take before the batch stops (E08 T90)
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


def dismiss(game, quiet=1.5, limit=60.0, mem=None, events=None):
    """Clicks every dialog's OK until none shows for `quiet` seconds on the adventure map;
    returns how many it clicked. With `mem` and `events` (E08 T79b, batch E9b's timed events at a
    turn's start) each message's record and pictures go into events["messages"] and
    events["message_pictures"] before its OK."""
    clicked, since, end = 0, time.monotonic(), time.monotonic() + limit
    while time.monotonic() < end:
        pix = game.settled()
        found = find_button(game, "ok", pix)
        if found:
            if events is not None and mem.dialog():
                events["messages"].append(message_record(mem))
                events["message_pictures"].append(pictures_text(mem))
            click_button(game, "ok", found[0])
            clicked, since = clicked + 1, time.monotonic()
        elif not game.matches("adventure", pix):
            since = time.monotonic()
        elif time.monotonic() - since >= quiet:
            return clicked
        time.sleep(0.1)
    bad = game.out / "not-dismissed.xwd"
    game.dump(bad)  # E08 T57: what showed, for the record
    raise TimeoutError(f"dialogs keep coming or the adventure map does not show (the display: "
                       f"{bad})")


# -- the game's start and turns ----------------------------------------------------------------
def to_hot_seat(game, names, difficulty="normal", gold=False):
    """From the start to the first human's adventure map of the only map in Maps/, every name a
    human player (hot seat, at least two; colours from red on, as the map allows them), the
    players' difficulty chosen in the scenario list (E17 H1; its default Normal); with `gold`
    (the battle recipes, E17 M5 T8a) the named players' starting bonus Gold, else random."""
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
    if gold:
        h3drive.gold_bonus(game, len(names))
    game.click(*BEGIN, 1.0)
    game.move(*PARK)
    game.wait("turn_dialog", 60)
    dismiss(game)


def end_turn(game, mem, events=None):
    """Ends the current player's turn (key e) and passes the dialogs up to the next human's map;
    returns the new current player. With `events` (E08 T79b) the messages of the new turn's start
    are recorded there (dismiss)."""
    player = mem.current_player()
    game.key("e", 0.5)
    end = time.monotonic() + 60
    while mem.current_player() == player:
        if time.monotonic() > end:
            raise TimeoutError("the turn did not end")
        found = find_button(game, "ok")
        if found:
            click_button(game, "ok", found[0])
        time.sleep(0.02)
    # E08 T90 (batch E10a's Tan): a computer player's turn is played by the game, which can take
    # less than a poll of 0.2 s; the harness polls every 0.02 s, notes each computer colour it
    # saw as the current player (events["ai_turns"], with the seconds it was) and goes on at the
    # next human's turn.
    end, passed, began = time.monotonic() + AI_TURN_LIMIT, None, time.monotonic()
    while True:
        current = mem.current_player()
        if passed is not None and passed != current and events is not None:
            events["ai_turns"][-1]["seconds"] = round(time.monotonic() - began, 2)
        if mem.is_human(current):
            break
        if time.monotonic() > end:
            raise TimeoutError(f"the computer player {current}'s turn does not end "
                               f"in {AI_TURN_LIMIT} s")
        if passed != current:
            passed, began = current, time.monotonic()
            if events is not None:
                events.setdefault("ai_turns", []).append({"player": passed})
        found = find_button(game, "ok")
        if found:
            click_button(game, "ok", found[0])
        time.sleep(0.02)
    dismiss(game, mem=mem, events=events)
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


def select_town(game, mem, index):
    """Selects the current player's town `index` (of the game's towns) from the town list, as
    `select` a hero (E24 M3 T4a): with more than five towns the list is scrolled to its top first,
    then down until the town's row shows; the selection is checked in memory (the player's +0x3F;
    town_select 0x417830 sets the selected hero to none). A town selected already is not clicked
    again (a click on it opens its screen)."""
    colour = mem.current_player()
    towns = mem.town_list(colour)
    if index not in towns:
        raise ValueError(f"town {index} is not the current player's ({towns})")
    if mem.player(colour)["selected"] < 0 and mem.kingdom(colour)["selected"] == [
            2, towns.index(index)]:
        return
    x, y, step, rows = TOWN_LIST
    row = towns.index(index)
    if len(towns) > rows:
        for _ in range(len(towns) - rows):
            game.click(*TOWN_UP, 0.4)
        for _ in range(max(0, row - rows + 1)):
            game.click(*TOWN_DOWN, 0.4)
        row = min(row, rows - 1)
    game.click(x, y + step * row, 0.6)
    game.move(*PARK)
    wait_for(lambda: mem.kingdom(colour)["selected"] == [2, towns.index(index)],
             f"select town {index}")


def wait_for(test, what, limit=3.0):
    """Waits until `test()` holds; a timeout raises with `what`, or with `what()` when it is a
    function (E24 M1 T6b, the reviewer's R3-9: a message naming the state at the timeout, not at
    the start)."""
    end = time.monotonic() + limit
    while not test():
        if time.monotonic() > end:
            raise TimeoutError(what() if callable(what) else what)
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
    wait_for(lambda: mem.view() == want, lambda: f"the view at {want} (it is at {mem.view()})")


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
    try:
        wait_for(lambda: mem.mouse_tile() == (x, y, level), f"the mouse on {(x, y, level)}")
    except TimeoutError:
        # E08 T57 (batch E7b's E7-23): the game finds the tile under the mouse again only when the
        # mouse enters another of the view's cells, so after the view moved under a mouse that
        # stays in the same cell it keeps the tile it saw before. The mouse goes to the next cell
        # (inside the view) and back.
        vx, _ = mem.view()
        game.move(*tile_point(mem, x + 1 if x + 1 - vx < MAP_AREA[2] else x - 1, y))
        time.sleep(0.5)
        game.move(*point)
        wait_for(lambda: mem.mouse_tile() == (x, y, level),
                 lambda: f"the mouse on {(x, y, level)} (the game sees {mem.mouse_tile()})")
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
         "ADVEVENT:91": "flee",
         # E08 T90 (batch E10a): a dwelling's guards' question (GENRLTXT 422) and its hire question
         "GENRLTXT:422": "guards", "ADVEVENT:35": "hire", "ADVEVENT:36": "hire"}
BATTLE_PARK = (420, 580)  # over the battle's panel, off the battlefield
SIDES = ("attacker", "defender")
BATTLE_KEYS = {"retreat": "r", "surrender": "s"}
# The level-up dialog's two offered skills from its OK's corner, the OK disabled until one is
# chosen (E17 H2, `[button.ok_disabled]`): a step's `level_up`, default the left one.
LEVEL_UP_CHOICES = {"left": (-150, -64), "right": (-52, -64)}


def size_word(record, mem):
    """E08 T90 (batch E10a, E10-1): the quantity word of a guards' question (GENRLTXT 422's
    second conversion, ARRAYTXT's lower-case rows from 176: `a few` 176, `several` 179, `a pack
    of` 182, ...) as {"size_word": 182}, named by its row, never its text (ADR-0009); {} for
    any other question."""
    if record != "GENRLTXT:422":
        return {}
    text, rows = mem.dialog_text(), h3lod.table("ARRAYTXT.TXT")
    found = [n for n in range(176, 215) if n < len(rows) and rows[n] and rows[n][0].strip()
             and rows[n][0] == rows[n][0].lower()
             and re.search(r"(?<![A-Za-z])" + re.escape(rows[n][0]) + r"(?![A-Za-z])", text)]
    return {"size_word": max(found, key=lambda n: len(rows[n][0])) if found else None}


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
                game.dump(game.out / "auto-stalled.xwd")
                raise TimeoutError("auto combat does not start")
            time.sleep(0.2)
        else:
            return


# The war machines (145 Catapult to 149 Arrow Tower): they act by themselves (no Artillery, no
# Ballistics, no First Aid: the computer aims them), so the plans never give them orders.
MACHINES = range(145, 150)
CATAPULT = 145  # E09 T3b: a siege's; without Ballistics it acts by itself
# Where an attack on a hex is clicked: right of its centre (h3drive.hex_point), so the attacker
# comes from the right (E4-9: the Ballista stands on the attacker's first two columns).
ATTACK_FROM_RIGHT = 14
TURN_SETTLE = 0.5  # seconds between a stack's turn showing in memory and its order


def active_creature(mem, active):
    """E09 T3b: the creature of the active stack (`[side, record]`), None when none stands so."""
    return next((s["creature"] for s in mem.stacks() if [s["side"], s["record"]] == active), None)


def hex_distance(a, b):
    """Steps between two battle hexes (17 a row, the even rows half a hex to the right)."""
    def cube(h):
        row, col = divmod(h, 17)
        q = col - (row + (row & 1)) // 2
        return q, row, -q - row
    return max(abs(u - v) for u, v in zip(cube(a), cube(b)))


def approach(game, mem, board, goal, active):
    """The active stack cannot attack its target this turn: it moves to the free hex nearest
    `goal` within its speed (hex steps: a flyer's reach; the first and last columns hold no
    creature; a two-hex stack's other hex, behind it, free too). Returns what it did."""
    stack = next(s for s in mem.stacks() if [s["side"], s["record"]] == active)
    own = {h["hex"] for h in board if h["stack"] == active}
    free = {h["hex"] for h in board if (h["stack"] is None or h["hex"] in own)
            and not h["flags"] & 0x2 and h["hex"] % 17 not in (0, 16)}
    behind = (1 if active[0] == 1 else -1) if len(own) == 2 else 0
    reach = [h for h in free if (h + behind) in free and h // 17 == (h + behind) // 17
             and h not in own and hex_distance(stack["hex"], h) <= stack["speed"]]
    if not reach:
        raise RuntimeError(f"no free hex in reach of {stack['hex']} (speed {stack['speed']})")
    to = min(reach, key=lambda h: (hex_distance(h, goal), h))
    game.click(*h3drive.hex_point(to), 0.5)
    return f"moves to hex {to} (toward hex {goal}; speed {stack['speed']})"


def neighbour(h, direction):
    """The hex next to `h` in a direction (right, left, up-right, up-left, down-right,
    down-left), None off the field (the even rows lie half a hex right of the odd ones)."""
    row, col = divmod(h, 17)
    shift = 1 if row % 2 == 0 else 0
    dr, dc = {"right": (0, 1), "left": (0, -1), "up-right": (-1, shift),
              "up-left": (-1, shift - 1), "down-right": (1, shift),
              "down-left": (1, shift - 1)}[direction]
    row, col = row + dr, col + dc
    return row * 17 + col if 0 <= row < 11 and 0 <= col < 17 else None


# Where a click on a hex points the attack: off its centre toward the side the attacker comes
# from (the game's cursor takes the direction from the mouse's place in the hex).
# The combat cursor (the mouse manager's type 2, design §9.9) shows a melee attack as one of
# its sword frames, each a direction (CRCOMBAT.DEF's frames 7 to 14): an attack is clicked only
# where it shows one, after the mouse rested over the point (the game reads the mouse on a 33 ms
# grid of its clock).
COMBAT_CURSOR, MELEE_FRAMES, HOVER = 2, range(7, 15), 0.25
STRIKE_PASSES, STRIKE_PAUSE = 4, 1.0
STRIKE_OFFSETS = {"right": (14, 0), "left": (-14, 0), "up-right": (8, -14), "up-left": (-8, -14),
                  "down-right": (8, 14), "down-left": (-8, 14)}


def strike(game, mem, active, aim):
    """The active stack attacks the stack `aim` if it can this turn (the pathfinder's mark on
    the target's hexes): the mouse over the target's hexes, off the centre toward each free
    neighbour in turn (nearest first), until the cursor shows a melee attack, clicked there; else
    it moves toward it (approach). Returns what it did."""
    board = mem.hexes()
    stack = next(s for s in mem.stacks() if [s["side"], s["record"]] == active)
    own = {h["hex"] for h in board if h["stack"] == active}
    hexes = [h["hex"] for h in board if h["stack"] == [aim["side"], aim["record"]]]
    nearest = min(hexes, key=lambda h: (hex_distance(stack["hex"], h), h))
    if not any(board[h]["attackable"] for h in hexes):
        return approach(game, mem, board, nearest, active)
    sides = []
    for target in hexes:
        for direction in STRIKE_OFFSETS:
            n = neighbour(target, direction)
            if n is not None and n % 17 not in (0, 16) and (board[n]["stack"] is None or n in own) \
                    and not board[n]["flags"] & 0x2:
                sides.append((hex_distance(stack["hex"], n), target, direction))
    seen = []
    # Several passes (E08 T32b2): a turn the memory already gives a stack may not take input yet
    # (seen once in E4-9, every position the plain pointer, frame 6: a good morale's extra turn
    # shows its animation first).
    for attempt in range(STRIKE_PASSES):
        if attempt:
            time.sleep(STRIKE_PAUSE)
        for _, target, direction in sorted(sides):
            x, y = h3drive.hex_point(target)
            dx, dy = STRIKE_OFFSETS[direction]
            game.move(x + dx, y + dy)
            time.sleep(HOVER)
            cursor = mem.globals()["cursor"]
            seen.append(cursor[1] if cursor else None)
            if cursor and cursor[0] == COMBAT_CURSOR and cursor[1] in MELEE_FRAMES:
                game.click(x + dx, y + dy, 0.2)
                late = f", pass {attempt + 1}" if attempt else ""
                return (f"attacks {mem.creature_name(aim['creature'])} at hex {target} from the "
                        f"{direction} (cursor frame {cursor[1]}{late})")
    game.dump(game.out / "no-attack-cursor.xwd")
    raise RuntimeError(f"no attack cursor over hexes {hexes} (cursor frames {seen}; the display: "
                       f"{game.out / 'no-attack-cursor.xwd'})")


def play_turns(game, mem, order, done, end, what):
    """E08 T32b1 (E4-9): plays the battle's turns until `done()` or its end: on each turn of a
    stack that is not a war machine (they act by themselves), `order(stack)` names the stack to
    attack (strike) or None to defend. `what` names the phase in the errors."""
    while not done():
        if time.monotonic() > end:
            raise TimeoutError(f"{what}: the battle goes on")
        if find_button(game, "battle_ok"):
            return
        active = mem.globals()["active"]
        if active is None:
            time.sleep(0.1)
            continue
        stack = next((s for s in mem.stacks() if [s["side"], s["record"]] == active), None)
        if stack is None or stack["creature"] in MACHINES:
            time.sleep(0.1)
            continue
        time.sleep(TURN_SETTLE)  # an order right at the turn's start may be lost
        if mem.globals()["decided"]:
            # E08 T110 (batch E12a's third run: a retaliation ended the battle, the dead
            # attacker still named active until the result window opened): decided, no order.
            time.sleep(0.1)
            continue
        if mem.globals()["active"] != active:
            # E09 T3b (batch TWN-1's 3b, the Archangels' blow that killed the Griffins): the
            # striker named the active stack again while it turns back after its attack (the
            # README's trap), the next stack's turn begun within the settle: read the turn again
            # (an order then went to the Swordsman's turn: the cursor's query over it, frame 5).
            continue
        aim = order(stack)
        if aim is None:
            game.key("d", 0.3)
            action = "defends"
        else:
            action = strike(game, mem, active, aim)
        print(f"plan: {SIDES[active[0]]} {mem.creature_name(stack['creature'])} (hex "
              f"{stack['hex']}) {action}", file=sys.stderr, flush=True)
        game.move(*BATTLE_PARK)
        try:
            wait_for(lambda a=active: mem.globals()["active"] != a or find_button(game, "battle_ok"),
                     "the next stack's turn", 30)
        except TimeoutError:
            game.dump(game.out / "plan-stalled.xwd")
            raise


def living(mem, side=None, creature=None):
    """The living stacks (of a side, of a creature by name)."""
    return [s for s in mem.stacks() if s["count"] > 0 and (side is None or s["side"] == side)
            and (creature is None or mem.creature_name(s["creature"]) == creature)]


def nearest_enemy(mem, stack):
    """The other side's living stack nearest to `stack`, creatures before war machines."""
    enemies = living(mem, 1 - stack["side"])
    return min(enemies, key=lambda s: (s["creature"] in MACHINES,
                                       hex_distance(stack["hex"], s["hex"]), s["record"]),
               default=None)


def attack_until_gone(game, mem, side, creature, end):
    """E4-9's first phase: while the other side has a living stack of `creature`, each of
    `side`'s stacks attacks it on its turn and every other stack defends."""
    if not living(mem, 1 - side, creature):
        raise RuntimeError(f"no stack of {creature} on the other side")
    play_turns(game, mem,
               lambda s: (living(mem, 1 - side, creature) or [None])[0] if s["side"] == side
               else None,
               lambda: not living(mem, 1 - side, creature), end, f"{creature} still stands")


def attack_to_the_end(game, mem, end):
    """Every stack attacks the other side's nearest one until the battle ends (two human sides:
    auto combat does not play a battle between two human players in hot seat, E08 T32b1)."""
    play_turns(game, mem, lambda s: nearest_enemy(mem, s), lambda: False, end, "attack")


def auto_again(game, mem, end, stall=8.0, settle_on=3.0, refused=3):
    """E08 T79a (batch E9a's E9-2: who plays an owned mine's guards): auto combat, turned on
    again at each stop. Auto combat is on when the battle manager's flag says so (+0x132C4), so
    the key is pressed only while it is off (a second press would stop it); each time it is
    found off again after it was on, a stop is recorded (the active stack's side, creature and
    count). When the key leaves it off `refused` times in a row (the first run: a battle between
    two human players in hot seat, as E08 T32b1 found), or it is on and the same stack stays
    active with every stack unchanged for `stall` seconds, that stop is recorded and the rest is
    played by `play_waiting` (every stack attacks the nearest enemy, each turn recorded as
    waiting for an order or not). Returns the stops: [{side, creature, count, auto}]."""
    stops, pressed, presses, state, since = [], False, 0, None, time.monotonic()
    while not find_button(game, "battle_ok"):
        if time.monotonic() > end:
            game.dump(game.out / "auto-again-stalled.xwd")
            raise TimeoutError("the battle does not end")
        active = mem.globals()["active"]
        now = (active, stacks(mem))
        if now != state:
            state, since = now, time.monotonic()
        if active is None:
            time.sleep(0.1)
            continue
        stack = next((s for s in mem.stacks() if [s["side"], s["record"]] == active), None)
        if stack is None:
            time.sleep(0.1)
            continue
        stop = {"side": SIDES[active[0]], "creature": stack["creature"], "count": stack["count"]}
        if not mem.auto_combat_on():
            if presses >= refused:
                stops.append({**stop, "auto": f"the key left auto combat off {presses} times"})
                stops += play_waiting(game, mem, end)
                break
            if pressed:
                stops.append({**stop, "auto": "off again: turned on again"})
                pressed = False
            game.key("a", 0.5)
            presses += 1
            try:
                wait_for(lambda: mem.auto_combat_on() or find_button(game, "battle_ok"),
                         "auto combat on", settle_on)
                pressed, presses = True, 0
            except TimeoutError:
                pass  # lost (before the stack took orders) or refused: pressed again
            state, since = None, time.monotonic()
            continue
        if time.monotonic() - since > stall:
            stops.append({**stop, "auto": "on, the stack waits"})
            game.key("a", 0.5)  # auto combat off before the orders
            stops += play_waiting(game, mem, end)
            break
        time.sleep(0.2)
    return stops


def play_waiting(game, mem, end, patience=2.0):
    """E08 T79a: plays the rest of a battle as attack_to_the_end does, but first waits up to
    `patience` seconds at each turn: a stack the computer plays acts by itself (recorded `acted
    without an order`), one that waits for its player's order is recorded (`waited for an
    order`) and attacks the nearest enemy (strike). Returns the turns' records."""
    turns = []
    while not find_button(game, "battle_ok"):
        if time.monotonic() > end:
            raise TimeoutError("play_waiting: the battle goes on")
        active = mem.globals()["active"]
        stack = next((s for s in mem.stacks() if [s["side"], s["record"]] == active), None) \
            if active else None
        if stack is None or stack["creature"] in MACHINES:
            time.sleep(0.1)
            continue
        turn = {"side": SIDES[active[0]], "creature": stack["creature"], "count": stack["count"]}
        try:
            wait_for(lambda a=active: mem.globals()["active"] != a
                     or find_button(game, "battle_ok"), "an action without an order", patience)
            turns.append({**turn, "auto": "acted without an order"})
            continue
        except TimeoutError:
            pass
        aim = nearest_enemy(mem, stack)
        action = strike(game, mem, active, aim) if aim else (game.key("d", 0.3) or "defends")
        turns.append({**turn, "auto": f"waited for an order: {action}"})
        game.move(*BATTLE_PARK)
        try:
            wait_for(lambda a=active: mem.globals()["active"] != a or find_button(game, "battle_ok"),
                     "the next stack's turn", 30)
        except TimeoutError:
            game.dump(game.out / "plan-stalled.xwd")
            raise
    return turns


# E08 T110 (batch E12a's E12-4): the spell book in a battle of two humans. The book's key C is
# its button's (hero_book_button 0x59E900 behind both, E17 design §7.16), which refuses with a
# message without a hero, after the round's cast, at an anti-magic garrison (+0x53C4) or with the
# Orb of Inhibition; else the book dialog (0x59BDF0, 620 x 595 at (90, 2)) opens over the battle.
BOOK_KEY, BOOK_WAIT, CAST_WAIT = "c", 1.0, 6.0
DEBUG_DUMPS = False  # True: each book turn's screen after its cast, <out>/book-<side>.xwd


def spell_id(mem, name):
    """A spell's id by its name in the game's spell table (E08 T110)."""
    for k in range(70):
        if mem.spell_name(k) == name:
            return k
    raise ValueError(f"no spell named {name!r}")


def book_turn(game, mem, active, stack, spell, place):
    """At the active stack's turn (E08 T110): the spell book's key; a refusal's message noted and
    closed; an open book's place `place` clicked and its spell aimed at the active stack's own hex
    (Bless's target), the cast waited for; then Defend. Returns the turn's record: the creature
    and count, the book (`opened`, `refused` with the message's record, `nothing`), the cast
    (`made`, `refused` with its record, `not tried`), the side's hero's spell points before and
    after, the spells on the stack after."""
    side = active[0]
    hero = mem.battle_hero(side)
    points = (lambda: mem.hero(hero)["spell_points"]) if hero is not None else (lambda: None)
    record = {"creature": stack["creature"], "count": stack["count"], "hero": hero,
              "points_before": points(), "cast": "not tried"}
    game.key(BOOK_KEY, BOOK_WAIT)
    kind = screen_kind(mem)
    if kind == "message":
        record["book"] = f"refused: {message_record(mem)}"
        click_item(game, mem, "ok", {})
    elif kind == "battle":
        record["book"] = "nothing (no dialog)"
    else:
        shown = mem.dialog()
        record["book"] = f"opened ({kind if kind != 'other' else hex(shown['vtable'])})"
        game.move(*h3drive.book_place(place))
        time.sleep(0.3)
        game.click(*h3drive.book_place(place), settle=0.8)
        kind = screen_kind(mem)
        if kind == "message":
            record["cast"] = f"refused: {message_record(mem)}"
            click_item(game, mem, "ok", {})
            if screen_kind(mem) not in ("battle", None):
                game.key("Escape", 0.8)  # the book left open after the refusal
        elif kind != "battle":
            record["cast"] = f"the book stays open ({kind})"
            game.key("Escape", 0.8)
        else:
            x, y = h3drive.hex_point(stack["hex"])
            game.move(x, y)
            time.sleep(0.4)
            game.click(x, y, 0.5)
            target = [stack["side"], stack["record"]]
            try:
                wait_for(lambda: spell in next(s["spells"] for s in mem.stacks()
                                               if [s["side"], s["record"]] == target)
                         or points() != record["points_before"], "the cast", CAST_WAIT)
            except TimeoutError:
                pass
            # The cast's animation (a stack's +0x20 set while it plays; E17 design §7.15).
            settled = time.monotonic() + CAST_WAIT
            while time.monotonic() < settled and (mem.globals()["active"] is None
                                                  or any(s["magic"] for s in mem.stacks())):
                time.sleep(0.1)
            time.sleep(0.5)
            record["cast"] = "made" if points() != record["points_before"] else "not made"
            if DEBUG_DUMPS:
                game.dump(game.out / f"book-{SIDES[side]}.xwd")
    record["points_after"] = points()
    record["spells"] = next((s["spells"] for s in mem.stacks()
                             if [s["side"], s["record"]] == [stack["side"], stack["record"]]), {})
    if screen_kind(mem) == "message":
        record["cast"] += f"; then {message_record(mem)}"
        click_item(game, mem, "ok", {})
    game.move(*BATTLE_PARK)
    time.sleep(0.3)
    if mem.globals()["active"] == active:
        game.key("d", 0.3)
        record["then"] = "defends"
    else:
        record["then"] = "its turn over"
    return record


def book_turns(game, mem, plan, end):
    """E08 T110 (batch E12a's E12-4, both sides human): at each side's first stack's turn (not a
    war machine) book_turn with the plan's `book` spell (its place in the book `place`, default 0:
    the hero's only spell), every later stack of a side already done defending, until both sides
    had theirs or the battle ends. Returns {side: the turn's record}."""
    spell = spell_id(mem, plan["book"])
    done = {}
    while len(done) < 2 and not find_button(game, "battle_ok"):
        if time.monotonic() > end:
            raise TimeoutError("the book's turns: the battle goes on")
        active = mem.globals()["active"]
        stack = next((s for s in mem.stacks() if [s["side"], s["record"]] == active), None) \
            if active else None
        if stack is None or stack["creature"] in MACHINES:
            time.sleep(0.1)
            continue
        time.sleep(TURN_SETTLE)
        if mem.globals()["active"] != active:
            continue
        if active[0] in done:
            game.key("d", 0.3)
        else:
            done[active[0]] = book_turn(game, mem, active, stack, spell, int(plan.get("place", 0)))
            print(f"book: {SIDES[active[0]]} {done[active[0]]}", file=sys.stderr, flush=True)
        try:
            wait_for(lambda a=active: mem.globals()["active"] != a or find_button(game, "battle_ok"),
                     "the next stack's turn", 30)
        except TimeoutError:
            game.dump(game.out / "book-stalled.xwd")
            raise
    return done


def book_text(mem, turn):
    """A side's spell-book turn (E08 T110): `10 Archangels: the book opened (0x...); cast made;
    spell points 20 then 15; Bless on it (1 round)`; `no turn`."""
    if turn is None:
        return "no turn"
    spells = ", ".join(f"{mem.spell_name(k)} ({r} round{'s' if r != 1 else ''})"
                       for k, r in sorted(turn["spells"].items())) or "no spell"
    return (f"{turn['count']} {mem.creature_name(turn['creature'], turn['count'])}: the book "
            f"{turn['book']}; cast {turn['cast']}; spell points {turn['points_before']} then "
            f"{turn['points_after']}; on the stack after: {spells}; then {turn['then']}")


def end_tactics_phase(game, mem, end):
    """Ends a battle's tactics phase (E08 T47a): the message that opens it (GENRLTXT 373) closed
    with its OK, then the battle's Start button (item 30722, "Start Combat") clicked, until the
    phase's flag clears."""
    while mem.tactics_phase():
        if time.monotonic() > end:
            raise TimeoutError("the tactics phase does not end")
        kind = screen_kind(mem)
        if kind in ("message", "battle"):
            click_item(game, mem, "ok" if kind == "message" else "start", {})
        time.sleep(0.2)
    game.move(*BATTLE_PARK)


def fight(game, mem, plan, limit=300.0):
    """Plays the battle on the screen as `plan` says and returns its record: the stacks at the
    start and the end, the outcome and how it was played. `plan`: "auto" (auto combat: the
    computer plays every human side; setup.sh's `Combat Auto Creatures`), or "retreat" or
    "surrender", optionally "defender retreat" or "defender surrender" (default the attacker):
    every other stack defends until one of that side's stacks acts, which then retreats or
    surrenders; the confirmation's OK is clicked. Since E08 T32b1: "attack" (every stack attacks
    the other side's nearest stack, moving toward it when out of reach, until the battle ends:
    auto combat does not play a battle between two human players in hot seat), or a table `{
    attack = "<creature>", by = "attacker" | "defender", then = "attack" | "auto" }` (E4-9): each
    stack of that side attacks the other side's stack of that creature while it stands and every
    other stack defends, then `then` plays the rest; the war machines act by themselves. Since
    E08 T79a "auto again" (auto combat turned on again at each stop: auto_again, its `stops`
    recorded). The record holds the battlefield at the start (`field`: h3mem.battlefield) and
    since E08 T79a each side's owner and controllers (`sides`: h3mem.battle_sides). Since E09
    T3b (batch TWN-1's siege won) the table's `attack` may be a list of creatures, attacked in
    turn while every other stack defends (one not standing when its turn comes passed over), and
    in a retreat or surrender the side's Catapult (a siege's; it acts by itself without
    Ballistics) is waited out: the side's first stack that is not a machine retreats."""
    if plan is None:
        raise RuntimeError("a battle the step does not plan (`battle`)")
    if isinstance(plan, dict) and "book" in plan:
        # E08 T110 (batch E12a's E12-4): `{ book = "Bless", then = "attack" }`, optionally
        # `place = <k>` (book_turns).
        if set(plan) - {"book", "then", "place"} or plan.get("then") not in ("auto", "attack"):
            raise ValueError(f"unknown battle plan `{plan}`")
        action, side = "book", 0
    elif isinstance(plan, dict):
        if set(plan) != {"attack", "by", "then"} or plan["by"] not in SIDES \
                or plan["then"] not in ("auto", "attack") \
                or not isinstance(plan["attack"], (str, list)) or plan["attack"] == []:
            raise ValueError(f"unknown battle plan `{plan}`")
        action, side = "first", SIDES.index(plan["by"])
    else:
        words = plan.split()
        if plan == "auto again":
            words = ["again"]  # E08 T79a: auto combat, on again at each stop (auto_again)
        action, side = words[-1], SIDES.index(words[0]) if len(words) == 2 else 0
        if action not in ("auto", "attack", "again", *BATTLE_KEYS) or len(words) > 2:
            raise ValueError(f"unknown battle plan `{plan}`")
    game.move(*BATTLE_PARK)
    wait_for(lambda: mem.stacks(), "the battle's stacks", 10)
    time.sleep(0.5)
    record = {"plan": plan, "field": mem.battlefield(), "start": stacks(mem),
              # E08 T68a: the hexes an obstacle blocks at the opening (batch E8's openings).
              "blocked": mem.obstacle_hexes(),
              # E08 T79a: each side's owner and whether a human (here) controls it (E9-2).
              "sides": mem.battle_sides(), "stops": [],
              # E08 T57: every stack's record at the opening (army slot, morale, luck, ...).
              "opening": [{k: st[k] for k in ("side", "army_slot", "hex", "creature", "count",
                                                "morale", "luck")} for st in mem.stacks()]}
    end = time.monotonic() + limit
    # E08 T47a (batch E6's E6-6): a tactics phase (battle manager +0x13D68 at the start) is ended
    # at once, as the templates say: its message's OK, then its Start button.
    record["tactics"] = mem.tactics_phase()
    if record["tactics"]:
        end_tactics_phase(game, mem, end)
    if action == "book":
        record["books"] = book_turns(game, mem, plan, end)
        if find_button(game, "battle_ok"):
            pass
        elif plan["then"] == "auto":
            auto_combat(game, mem, end)
        else:
            attack_to_the_end(game, mem, end)
    elif action == "first":
        if isinstance(plan["attack"], str):
            attack_until_gone(game, mem, side, plan["attack"], end)
        for creature in plan["attack"] if isinstance(plan["attack"], list) else []:
            # E09 T3b: the creatures in turn, one gone (or never there) passed over.
            if living(mem, 1 - side, creature) and not find_button(game, "battle_ok"):
                attack_until_gone(game, mem, side, creature, end)
        if find_button(game, "battle_ok"):
            pass
        elif plan["then"] == "auto":
            auto_combat(game, mem, end)
        else:
            attack_to_the_end(game, mem, end)
    elif action == "attack":
        attack_to_the_end(game, mem, end)
    elif action == "again":
        record["stops"] = auto_again(game, mem, end)
    elif action == "auto":
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
            if active[0] == side and active_creature(mem, active) == CATAPULT:
                # E09 T3b (batch TWN-1's 3a): a siege's Catapult acts by itself; its turn passes.
                time.sleep(0.1)
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


def in_battle(game, pix):
    """The battle screen, whether the acting human hero has a spell book or not: the `battle`
    probes lie on the panel's book button, which a hero with a book lights (`battle_book`, E17
    M3 T7a-2a; accepted since E08 T32b1: batch E4's magic heroes carry books)."""
    return game.matches("battle", pix) or game.matches("battle_book", pix)


def level_up_record(mem, shown):
    """A level-up dialog's record (E08 T32b1): the hero whose name its text holds (the longest
    such name of the heroes on the map) with its level (the new one: hero_level_up 0x4DA720
    raises it before the dialog), the primary skill raised and the offers left and right
    (h3mem.level_up_dialog)."""
    text = mem.dialog_text()
    named = sorted((h for h in mem.heroes() if h["name"] and h["name"] in text),
                   key=lambda h: -len(h["name"]))
    hero = named[0] if named else None
    return {"hero": hero["name"] if hero else None, "level": hero["hero_level"] if hero else None,
            **shown}


def offer_text(offer):
    """An offer as the templates take it: the skill's name (the level it would reach follows from
    the hero's skills), "none" for none."""
    return offer[0] if offer else "none"


def side_of(choice, shown):
    """A level-up choice's side: `left` or `right` as given, or a skill by name (E08 T37b, batch
    E5e's forced offers: the seed byte decides its side) on whichever side offers it, else the
    left (the dialog's record shows what was offered and taken); since E08 T57 `not <skill>`, the
    side that does not offer it (the left if neither does)."""
    if choice in LEVEL_UP_CHOICES:
        return choice
    if choice.startswith("not ") and choice[4:] in h3mem.SKILLS:
        # E08 T57 (batch E7's E7-4): the offer that is not that skill, the left if neither is.
        return "right" if (shown["left"] or [None])[0] == choice[4:] else "left"
    if choice not in h3mem.SKILLS:
        raise ValueError(f"`level_up = {choice!r}`: left, right or a secondary skill's name")
    return next((side for side in ("left", "right") if (shown[side] or [None])[0] == choice),
                "left")


def settle(game, mem, index, answer=None, battle=None, level_up="left", actions=None,
           choice=None, opening=None):
    """Waits until the hero, its points and the players' resources stay the same for QUIET
    seconds on the adventure map (the walk and any visit over), on the way answering each question
    as `answer` says, playing each battle as `battle` says and clicking any other dialog's OK;
    returns the observation and what happened: the dialogs passed, the questions (record, kind,
    answer), the battles (fight's records), the level-ups (the choice made, for each dialog with
    two offers: `level_up`, one choice or a list of them in order, the last one repeated) and
    since E08 T32b1 every level-up dialog's record (`level_up_dialogs`: hero, level, primary skill,
    offers, the one taken) and the records of the other dialogs closed with their OK
    (`messages`: `GENRLTXT:<n>` from h3lod, `?` for a text not identified). Since E08 T47a an
    exchange screen (a meeting) or an army dialog (a join without room) that opens plays the
    step's `actions` (run_actions; once) and is then closed if they left it open (its close or
    OK: no change); `screens` lists them. Since E08 T57 a message box whose OK waits for one of
    its pictures (a Treasure Chest's gold or experience) takes the side `choice` says (`left`,
    `right`, or a list of them, one per such box in order, the last one repeated), then its OK
    (`choices`: what each side showed and the side taken), and a message closed with its OK
    keeps its pictures with their captions (`message_pictures`, parallel to `messages`). Since
    E09 T3b the town screen (kind `town`, TOWN_SCREEN_VTABLE) is noted (`town_screen`) and
    closed (`town_screen_closed`: how), every unknown screen's vtable noted
    (`unknown_screens`)."""
    events = {"dialogs": 0, "questions": [], "battles": [], "level_ups": [],
              "level_up_dialogs": [], "messages": [], "screens": [], "actions": [],
              "choices": [], "message_pictures": [], "town_screen": False,
              "unknown_screens": []}
    choices = [level_up] if isinstance(level_up, str) else list(level_up)
    picks = [] if choice is None else [choice] if isinstance(choice, str) else list(choice)
    last, since, known = None, time.monotonic(), time.monotonic()
    end = time.monotonic() + 600
    while time.monotonic() < end:
        kind = screen_kind(mem)
        if kind in ("exchange", "army", "garrison", "boat", "hire"):
            # E08 T79a (batch E9a): also a mine's Leave Guards (the army dialog) and a
            # Shipyard's boat dialog. E08 T90 (batches E10a, E10b): the hire dialog, read as it
            # opens (`hires`: `opening`, before any click) and as the actions leave it (`after`,
            # None when they closed it), then closed by its Close.
            events["screens"].append(kind)
            if kind == "hire":
                events.setdefault("hires", []).append({"opening": mem.recruit_dialog()})
            if kind == "exchange":
                # E08 T100a (batch E11a's E11-6): whom the meeting shows, read as it opens (the
                # swap manager's +0x40, +0x44: left the visitor, right the visited).
                shows = screen_heroes(mem, kind, index)
                events.setdefault("exchanges", []).append(
                    {side: mem.hero(shows[side])["name"] for side in ("left", "right")})
            if kind == "garrison":
                # E08 T110 (batch E12a): where the hero stands as a garrison's exchange opens and
                # both bars then (the record by that entrance, the hero's army).
                hero = mem.hero(index)
                found = garrison_at(mem, *hero["tile"])
                events.setdefault("garrison_dialogs", []).append(
                    {"at": hero["tile"], "upper": found["army"] if found else None,
                     "lower": hero["slots"]})
            if actions and not events["actions"]:
                events["actions"] = run_actions(game, mem, index, actions)
            if kind == "hire":
                events["hires"][-1]["after"] = mem.recruit_dialog()
            if (now := screen_kind(mem)) in ("exchange", "army", "garrison", "boat", "hire"):
                close_screen(game, mem, now, screen_heroes(mem, now, index))
            last, since, known = None, time.monotonic(), time.monotonic()
            continue
        # E09 T3b (batch TWN-1): the town screen noted and closed; an unknown screen's vtable
        # noted.
        if kind == "town":
            events["town_screen"] = True
            events["screens"].append(kind)
            # E09 T14b (batch TWN-2): the step's `actions` played on the town screen once (the
            # hall, a slot, Build); a building's dialog or a hall they left open closed (Cancel,
            # the hall's close: nothing built) before the town screen.
            if actions and not events["actions"]:
                events["actions"] = run_actions(game, mem, index, actions)
                for left, item in (("build", "cancel"), ("hall", "close"), ("hire", "close")):
                    if screen_kind(mem) == left:
                        click_item(game, mem, item, {"": index})
            events["town_screen_closed"] = close_town_screen(game, mem)
            if events["town_screen_closed"] == "open":
                game.dump(game.out / "town-screen.xwd")
                raise RuntimeError("the town screen does not close (its item 30720, "
                                   f"TOWN_SCREEN_KEY; the display: {game.out / 'town-screen.xwd'})")
            last, since, known = None, time.monotonic(), time.monotonic()
            continue
        if (vtable := unknown_vtable(mem)) and vtable not in events["unknown_screens"]:
            events["unknown_screens"].append(vtable)
        pix = game.pixels()
        if game.matches("adventure", pix) or in_battle(game, pix):
            known = time.monotonic()
        elif time.monotonic() - known > STRANGE:
            bad = game.out / "not-adventure.xwd"
            game.dump(bad)
            # E09 T3b: the topmost dialog's vtable named, for a screen to be learned.
            raise TimeoutError(f"neither the adventure map, a battle nor a dialog for {STRANGE} s "
                               f"(the game over?; the display: {bad}; the topmost dialog: "
                               f"{unknown_vtable(mem) or kind})")
        if in_battle(game, pix):
            # E08 T68b (batch E8b's E8-13, E8-14): the hero's tile as the battle opens.
            at = mem.hero(index)["tile"]
            # E08 T100a (batch E11a's E11-7): the step's `at_opening` observations, read from
            # the game's memory as the first battle opens (tiles explored before it is fought).
            if opening and "at_opening" not in events:
                events["at_opening"] = {w: value({"after": {}}, w, mem) for w in opening}
            events["battles"].append({**fight(game, mem, battle), "hero_tile": at})
            # E09 T3b: the screen known up to the battle's end (a long battle left `known` stale
            # for the screen checks above).
            last, since, known = None, time.monotonic(), time.monotonic()
            continue
        waiting = find_button(game, "ok_disabled", pix)
        oks = find_button(game, "ok", pix)
        shown = mem.level_up_dialog() if waiting or oks else None
        if shown is not None:
            # A level-up dialog (E08 T32b1): two offers wait for a choice (the OK disabled), one
            # or none only for the OK. Recorded once: the dialog object is the same local of
            # hero_level_up for each level, so a dialog is known by its hero and level.
            record = level_up_record(mem, shown)
            known_dialogs = events["level_up_dialogs"]
            if not known_dialogs or (known_dialogs[-1]["hero"], known_dialogs[-1]["level"]) != (
                    record["hero"], record["level"]):
                if waiting:
                    choice = side_of(choices[min(len(events["level_ups"]), len(choices) - 1)],
                                     shown)
                    dx, dy = LEVEL_UP_CHOICES[choice]
                    game.click(waiting[0][0] + dx, waiting[0][1] + dy, 0.5)
                    game.move(*PARK)
                    events["level_ups"].append(choice)
                    record["taken"] = f"{choice} ({offer_text(shown[choice])})"
                else:
                    record["taken"] = offer_text(shown["left"])
                known_dialogs.append(record)
            elif oks and not waiting:
                click_button(game, "ok", oks[0])
                events["dialogs"] += 1
            last, since, known = None, time.monotonic(), time.monotonic()
            continue
        if waiting:
            # E08 T57: a message box whose OK waits for one of its pictures (a chest's gold or
            # experience: chest_gold_or_experience 0x4A6440), the side clicked, then its OK.
            sides = choice_sides(mem)
            if not sides:
                raise RuntimeError("a dialog whose OK waits for a choice, neither a level-up's "
                                   "nor a choice between pictures")
            if not picks:
                raise RuntimeError(f"a choice ({sides}) the step does not make (`choice`)")
            pick = picks[min(len(events["choices"]), len(picks) - 1)]
            if pick not in ("left", "right"):
                raise ValueError(f"`choice = {pick!r}`: left or right")
            click_item(game, mem, pick, None)
            wait_for(lambda: find_button(game, "ok"), "the choice's OK", 5)
            click_button(game, "ok", find_button(game, "ok")[0])
            events["choices"].append({**sides, "taken": pick})
            last, since, known = None, time.monotonic(), time.monotonic()
            continue
        cancels = find_button(game, "cancel", pix)
        if oks or cancels:
            known = time.monotonic()
        if oks and cancels:
            record = h3lod.identify(mem.dialog_text())
            kind = KINDS.get(record, "other")
            said = reply(answer, kind)
            if said not in ("yes", "no"):
                raise RuntimeError(f"a question ({record or 'its text not identified'}, {kind}) "
                                   "the step does not answer (`answer`)")
            events["questions"].append({"record": record, "kind": kind, "answer": said,
                                        **size_word(record, mem)})
            click_button(game, *(("ok", oks[0]) if said == "yes" else ("cancel", cancels[0])))
            last, since = None, time.monotonic()
            continue
        if oks:
            events["messages"].append(message_record(mem))
            events["message_pictures"].append(pictures_text(mem))
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


def approach_target(game, mem, name, target, days=0):
    """Walks the hero toward a target its player has not explored (E08 T57, batch E7's rounds of
    visits across a wide grid: the original's pathfinder finds no path onto an unexplored tile),
    as a player would: to the open tile (h3mem.open_tiles: explored, empty, not guarded) nearest
    the target, again until the target is explored; each walk may wait for the hero's next turn
    (`days`). Since E08 T68b a hero in a boat sails to open water tiles only (sea_only: batch E8b's
    legs over the water). Returns the tiles walked to and the days waited."""
    x, y, z = (*target, 0) if len(target) == 2 else target
    walked, waited = [], 0
    while True:
        hero = mem.hero_named(name)
        here = tuple(hero["tile"][:2])
        if mem.explored(x, y, z, hero["owner"]):
            return walked, waited

        def far(t, to):
            return max(abs(t[0] - to[0]), abs(t[1] - to[1]))
        open_ = mem.open_tiles(hero["owner"], z, sea_only(hero)) - {here}
        if not open_:
            raise RuntimeError(f"{name} has no open tile toward {target}")
        best = min(open_, key=lambda t: (far(t, (x, y)), far(t, here), t[1], t[0]))
        if far(best, (x, y)) >= far(here, (x, y)):
            raise RuntimeError(f"{name} at {here} comes no nearer to {target} (unexplored)")
        waited += ready(game, mem, name, [*best, z], days)["days_waited"]
        obs = walk(game, mem, name, [*best, z])
        if not obs["arrived"]:
            raise RuntimeError(f"{name} did not reach {best} on the way to {target}")
        walked.append(list(best))


def sea_only(hero):
    """True for a hero in a boat (status bit 18; E08 T68b): its open tiles are water tiles only
    (h3mem.open_tiles' `water`); None (any tile) for another."""
    return True if hero["status"] & h3mem.IN_BOAT else None


def view_both(game, mem, hero, x, y, z):
    """The view on both ends when they share a level and fit in it, else on the target alone
    (E17 H2; E08 T57: batch E7's visits 18 columns apart)."""
    x0, y0, z0 = hero["tile"]
    mid = ((x0 + x) // 2, (y0 + y) // 2)
    across, down = MAP_AREA[2:]
    fits = z == z0 and 0 <= x - mid[0] + CENTRE[0] < across and 0 <= y - mid[1] + CENTRE[1] < down
    centre(game, mem, *(mid if fits else (x, y)), z)


def ready(game, mem, name, target, days=0, near=False):
    """Gets a walk ready before its step reads anything (E08 T57, batch E7's rounds of visits):
    with `near` an unexplored target is approached first (approach_target); with `days` the
    target is clicked once and, while its path's mark is red (beyond today's points), the turns
    end until the hero's player's next one, at most `days` times (each new day the route is set
    anew: a click beside the hero, then on the target). Returns
    {approached, days_waited, clicked}: with `clicked` the walk's first click walks at once."""
    approached, waited = approach_target(game, mem, name, target, days) if near else ([], 0)
    done = {"approached": approached, "days_waited": waited, "clicked": False}
    if not days:
        return done
    x, y, z = (*target, 0) if len(target) == 2 else target
    hero = select(game, mem, name)
    view_both(game, mem, hero, x, y, z)
    game.click(*hover(game, mem, x, y, z), 0.3)
    game.move(*PARK)
    time.sleep(0.3)
    done["clicked"] = True
    while mem.path().get((x, y, z)) == "red" and done["days_waited"] < days + waited:
        while end_turn(game, mem) != hero["owner"]:
            pass
        done["days_waited"] += 1
        # The route anew for the new day's points: a click on an open tile beside the hero (a
        # route there, no walk), then one on the target (a click on the set destination walks).
        hero = select(game, mem, name)
        hx, hy, _ = hero["tile"]
        beside = sorted(t for t in mem.open_tiles(hero["owner"], z, sea_only(hero))
                        if max(abs(t[0] - hx), abs(t[1] - hy)) == 1 and t != (x, y))
        if not beside:
            raise RuntimeError(f"{name} has no open tile beside it to set its route anew")
        centre(game, mem, *beside[0], z)
        game.click(*hover(game, mem, *beside[0], z), 0.3)
        view_both(game, mem, hero, x, y, z)
        game.click(*hover(game, mem, x, y, z), 0.3)
        game.move(*PARK)
        time.sleep(0.3)
    return done


def walk(game, mem, name, target, answer=None, battle=None, level_up="left", actions=None,
         choice=None, once=False, again_if=None, opening=None):
    """Selects the hero, clicks the target tile once (the path shows: its marks are read) and
    again (the hero goes, or not, or visits what stands there), waits until it rests, answering
    questions and playing battles on the way (settle); returns the observation. With `once`
    (E08 T57, batch E7's E7-23) only the first click: the path is read and the hero stays. With
    `again_if = { marks = <n> }` (E08 T68a, batch E8a's E8-4) the second click only when the first
    showed that many marks (`again`: yes or no)."""
    hero = select(game, mem, name)
    index = hero["index"]
    x, y, z = (*target, 0) if len(target) == 2 else target
    view_both(game, mem, hero, x, y, z)
    before = observe(mem, index)
    point = hover(game, mem, x, y, z)
    # E08 T79a (batch E9a's E9-5): the cursor over the target before the first click (its type
    # and frame: an own Shipyard's tile shows cradvntr.DEF frame 6, survey E08-economy §5.3).
    time.sleep(0.1)
    cursor = mem.cursor()[:2]
    game.click(*point, 0.3)
    game.move(*PARK)
    time.sleep(0.3)
    path = mem.path()
    # The game walks on a click on the hero's current destination: a path to this tile set
    # before makes the first click walk.
    first_walked = observe(mem, index) != before
    # E08 T100a (batch E11a's E11-1, E11-13): `again_if = { marks = "any" }` clicks again when
    # the first click showed a path at all.
    wanted = None if again_if is None else again_if["marks"]
    again = not once and (wanted is None or (len(path) > 0 if wanted == "any"
                                             else len(path) == wanted))
    if not first_walked and again:
        game.click(*hover(game, mem, x, y, z), 0.3)
        game.move(*PARK)
    after, events = settle(game, mem, index, answer, battle, level_up, actions, choice, opening)
    colours = sorted(set(path.values()))
    return {
        "hero": name, "index": index, "from": before["tile"], "target": list(target),
        "tile": after["tile"], "marks": len(path), "mark": path.get((x, y, z), "none"),
        "path": colours, "tiles": sorted(path, key=lambda t: (t[2], t[1], t[0])),
        "arrived": after["tile"] == [x, y, z],
        "moved": after["tile"] != before["tile"], "points_before": before["points"],
        "points": after["points"], "first_click_walked": first_walked,
        "again": "yes" if again else "no", "cursor": cursor, **events,
    }


def press(game, mem, name, key, answer=None, battle=None, level_up="left"):
    """Selects the hero, centres the view on it and presses `key` once (E08 T68a, batch E8a's
    E8-7: the arrow key that steps the selected hero), waiting until it rests (settle); returns
    the observation: `key` says whether the hero stepped, the view scrolled or nothing moved.
    Since E08 T100a (batch E11a's E11-19: Space on a monolith, a visit again) the step's `answer`,
    `battle` and `level_up` play what the key brings, as a walk's do."""
    hero = select(game, mem, name)
    index = hero["index"]
    centre(game, mem, *hero["tile"])
    game.move(*PARK)
    time.sleep(0.3)
    before, view = observe(mem, index), mem.view()
    game.key(key, 0.5)
    after, events = settle(game, mem, index, answer, battle, level_up)
    moved = after["tile"] != before["tile"]
    return {"hero": name, "index": index, "from": before["tile"], "tile": after["tile"],
            "key": "stepped" if moved else "scrolled" if mem.view() != view else "nothing",
            "moved": moved, "points_before": before["points"], "points": after["points"],
            **events}


# -- A key again and again (E08 T100b: batch E11b's E11-15 to E11-17) ----------------------------
PRESSES_LIMIT = 60  # presses at most in one step
# The events of each press gathered into the step's (`messages`, `battles`, … read as a walk's).
PRESS_EVENTS = ("messages", "message_pictures", "questions", "battles", "level_ups",
                "level_up_dialogs", "screens", "exchanges", "choices")


def presses(game, mem, name, key, plan, answer=None, battle=None, level_up="left"):
    """A key pressed again and again (E08 T100b, batch E11b: Space on a monolith or a Whirlpool,
    a jump each): a step's `repeat = { most = <n>, first = <k>, until = { with = "<hero>" } }`
    presses `key` at most n times (PRESSES_LIMIT), each one as `press` does (the hero selected,
    the view centred, the key, what it brings played by `answer`, `battle` and `level_up`, the
    rest), read after it rests: the hero's tile before and after, its points and army, the press's
    messages, questions and battles. With `until` it stops before a press once the step's hero
    stands on the tile of the hero `with` names, also before the first (no press then). `first`
    numbers the first press's row (default 1: a table whose first row is the step onto the
    entrance starts the presses at 2). The step's observation: `presses`, `presses:<columns>`,
    `presses:changed`."""
    until = plan.get("until")
    most = plan.get("most")
    if set(plan) - {"most", "first", "until"} or not isinstance(most, int) \
            or not 0 < most <= PRESSES_LIMIT or (until is not None and set(until) != {"with"}):
        raise ValueError(f"`repeat = {plan!r}`: {{ most = <1 to {PRESSES_LIMIT}>, first = <row>, "
                         "until = { with = \"<hero>\" } }")
    first = plan.get("first", 1)
    index = mem.hero_named(name)["index"]

    def met():
        return until is not None and (mem.hero_named(until["with"])["tile"]
                                      == mem.hero(index)["tile"])

    done, events = [], {k: [] for k in PRESS_EVENTS}
    while len(done) < most and not met():
        army = mem.hero(index)["slots"]
        got = press(game, mem, name, key, answer, battle, level_up)
        for k in PRESS_EVENTS:
            events[k].extend(got.get(k, []))
        done.append({"n": first + len(done), "from": got["from"], "tile": got["tile"],
                     "key": got["key"], "points_before": got["points_before"],
                     "points": got["points"], "army_before": army,
                     "army": mem.hero(index)["slots"], "messages": got["messages"],
                     "questions": [q["record"] or "?" for q in got["questions"]],
                     "battles": len(got["battles"])})
    if until is None:
        stop = f"{len(done)} presses"
    elif met():
        stop = (f"on {until['with']}'s tile after {len(done)} presses" if done
                else f"on {until['with']}'s tile at the start: no press")
    else:
        stop = f"not on {until['with']}'s tile after {len(done)} presses"
    tile = mem.hero(index)["tile"]
    return {"hero": name, "index": index, "presses": done, "stop": stop,
            "from": done[0]["from"] if done else tile, "tile": tile, "moved": bool(done) and (
                done[0]["from"] != tile), **events}


def presses_value(obs, what, mem):
    """A `repeat` step's observations (E08 T100b): `presses` how it ended (`60 presses`, `on
    E11-17's tile after 7 presses`, `… at the start: no press`); `presses:<columns>` one table row
    per press, the columns named by `n` (the row's number), `from`, `to` (the places, place_name),
    `tile` (the tile after), `points`, `army` and `messages`; `presses:changed` the presses after
    which the points or the army differ from before, or that showed a message, a question or a
    battle (`none`)."""
    done = obs.get("presses")
    if done is None:
        raise ValueError(f"`{what}`: the step presses no key again (`repeat`)")
    if what == "presses":
        return obs["stop"]
    if what == "presses:changed":
        out = []
        for p in done:
            said = []
            if p["points"] != p["points_before"]:
                said.append(f"points {p['points_before']} to {p['points']}")
            if p["army"] != p["army_before"]:
                said.append(f"army {slots_text(mem, p['army_before'])} to "
                            f"{slots_text(mem, p['army'])}")
            said += [f"message {m}" for m in p["messages"]]
            said += [f"question {q}" for q in p["questions"]]
            said += [f"{p['battles']} battle(s)"] if p["battles"] else []
            if said:
                out.append(f"press {p['n']}: " + ", ".join(said))
        return "; ".join(out) or "none"
    columns = what.split(":", 1)[1].split(",")
    cell = {"n": lambda p: str(p["n"]), "from": lambda p: place_name(p["from"]),
            "to": lambda p: place_name(p["tile"]),
            "tile": lambda p: tile_name(p["tile"]),
            "points": lambda p: str(p["points"]), "army": lambda p: slots_text(mem, p["army"]),
            "messages": lambda p: ", ".join(p["messages"]) or "none"}
    if unknown := [c for c in columns if c not in cell]:
        raise ValueError(f"`{what}`: unknown column(s) {unknown} (of {', '.join(cell)})")
    return [[cell[c](p) for c in columns] for p in done]


def off_tile(mem, name):
    """The tile a step `walk = "off"` goes to (E08 T100b, batch E11b's E11-16: a step off a
    shared tile): the first of the hero's eight neighbours, the four straight ones first (north,
    east, south, west, then north-east, south-east, south-west, north-west), that its player may
    walk to without visiting or meeting anything (h3mem.open_tiles: explored, no object's class,
    not closed, outside every guard), of water for a hero in a boat and of land for one on foot."""
    hero = mem.hero_named(name)
    x, y, z = hero["tile"]
    open_ = mem.open_tiles(hero["owner"], z, water=bool(hero["status"] & h3mem.IN_BOAT))
    for dx, dy in ((0, -1), (1, 0), (0, 1), (-1, 0), (1, -1), (1, 1), (-1, 1), (-1, -1)):
        if (x + dx, y + dy) in open_:
            return [x + dx, y + dy] + ([z] if z else [])
    raise RuntimeError(f"{name} on {x},{y},{z}: no open neighbour to step off to")


# -- Replay Opponent Turn (E08 T100a: batch E11a's E11-18) ----------------------------------------
# adventure_command 0x409A70's control 9 (its switch 0x409AB4) runs adventure_options_run
# 0x41AB00: the modal dialog of 0x4051D0 (AdvOpts.pcx) with View World 1 (AdvView.def), the Puzzle
# Map 2 (AdvPuz.def), the scenario's information 3 (AdvInfo.def), Dig 4 (AdvDig.def), Replay
# Opponent Turn 5 (AdvTurn.def, enabled by 0x49DA70) and OK 30722 (iOk6432.def); its result 5
# calls turn_records_replay 0x49D7A0, which holds the byte 0x696A04 at 1 while it plays the
# turn's records back (read in the code, E08 T100a). The panel's button is the map dialog's
# item 9 (observed under Wine).
OPTIONS_BUTTON, REPLAY_ITEM, OPTIONS_OK = 9, 5, 30722
REPLAYING = 0x696A04
REPLAY_LIMIT = 300  # seconds a replay may take before the batch stops


def view_holds(corner, box):
    """Whether the view at `corner` (x, y, level) holds a tile of `box` ([x0, y0, x1, y1], the
    surface; MAP_AREA's whole tiles)."""
    x, y, z = corner
    x0, y0, x1, y1 = box
    across, down = MAP_AREA[2:]
    return z == 0 and x <= x1 and x0 < x + across and y <= y1 and y0 < y + down


def replay(game, mem, name, plan):
    """Replay Opponent Turn (E08 T100a, batch E11a's E11-18): with the step's hero selected, the
    panel's Adventure Options button (the map's item 9), then the dialog's Replay Opponent Turn
    (item 5), on the frozen clock: stepped 20 ms at a time (`Game.clock`), the replay's flag
    (0x696A04) and the view's top-left tile and level are read after each step, each new view
    kept in order with its time (ms from the click) and the screen, once it rests, dumped there
    (at most `dumps`, default 60: `<out>/replay-<ms>.xwd`; a dump taken without the frozen clock
    showed the view before, the first run's; `dumps = "every"` after every step, at most 150);
    `watch = [x0, y0, x1, y1]` names the surface tiles
    whose views are listed apart. A replay that does not start in 5 s of the game's time (the
    item disabled: nothing plays) leaves the dialog by its OK. Returns the observation."""
    plan = plan if isinstance(plan, dict) else {}
    hero = select(game, mem, name)
    game.move(*PARK)
    quiet_screen(game)
    if screen_kind(mem) != "map":
        raise RuntimeError(f"Replay Opponent Turn from the {screen_kind(mem)} screen")
    obs = {"hero": name, "index": hero["index"], "replay_views": [], "replay_dumps": [],
           "view_before": [*mem.view(), mem.view_level()], "watch": plan.get("watch")}
    shown = mem.dialog()["at"]
    game.click(*item_centre(mem, OPTIONS_BUTTON), 0.3)
    game.move(*PARK)
    wait_for(lambda: (mem.dialog() or {}).get("at") != shown, "the Adventure Options dialog", 5)
    quiet_screen(game)
    options = mem.dialog()
    items = {i["id"]: i for i in mem.dialog_items()}
    if REPLAY_ITEM not in items:
        raise RuntimeError(f"the dialog (vtable {options['vtable']:#x}) has no item {REPLAY_ITEM}")
    obs["options"] = {"vtable": f"{options['vtable']:#x}", "replay_state": items[REPLAY_ITEM]["state"],
                      "items": sorted(items)}
    point = item_centre(mem, REPLAY_ITEM)
    game.clock("freeze")
    game.click(*point, 0)
    game.move(*PARK)
    ms, step = 0, 20
    while not mem.read(REPLAYING, 1)[0] and ms < 5000:
        game.clock(f"step {step}")
        ms += step
    if not mem.read(REPLAYING, 1)[0]:
        game.clock("run")
        obs["replay"] = "not started"
        if (mem.dialog() or {}).get("at") == options["at"]:
            obs["replay"] = "not started: the dialog stayed open (the item disabled?)"
            game.click(*item_centre(mem, OPTIONS_OK), 0.3)
            game.move(*PARK)
        quiet_screen(game)
        obs["view_after"] = [*mem.view(), mem.view_level()]
        return obs
    began, last = ms, None
    every = plan.get("dumps") == "every"
    limit = 150 if every else plan.get("dumps", 60)
    try:
        while mem.read(REPLAYING, 1)[0]:
            if ms - began > REPLAY_LIMIT * 1000:
                raise TimeoutError(f"the replay does not end in {REPLAY_LIMIT} s of its time")
            corner = [*mem.view(), mem.view_level()]
            new = corner != last
            if new:
                last = corner
                obs["replay_views"].append({"corner": corner, "ms": ms})
            if (new or every) and len(obs["replay_dumps"]) < limit:
                game.settled()
                path = game.out / f"replay-{ms:05d}.xwd"
                game.dump(path)
                obs["replay_dumps"].append({"dump": path.name, "corner": corner, "ms": ms})
                if new:
                    obs["replay_views"][-1]["dump"] = path.name
            game.clock(f"step {step}")
            ms += step
    finally:
        game.clock("run")
    obs["replay"] = f"played ({ms - began} ms of the game's time, from {began} ms after the click)"
    quiet_screen(game, 1.0)
    obs["view_after"] = [*mem.view(), mem.view_level()]
    return obs


def replay_value(obs, what):
    """The observations of a `replay` step (E08 T100a): `replay` (played and how long, or not
    started), `replay:views` (each new view's top-left tile and level in order, `x,y,level`, `→`
    between), `replay:watched` (the views holding a tile of the step's `watch`, with their
    dumps; `none`), `replay:options` (the dialog's vtable, the item's state, its item ids),
    `view:before`, `view:after` (the view's corner before the dialog and after the replay) and
    `view:size` (MAP_AREA's whole tiles across and down)."""
    if what == "view:size":
        return f"{MAP_AREA[2]} x {MAP_AREA[3]} (the map area's whole tiles)"
    if "replay" not in obs:
        return "no replay"
    if what == "replay":
        return obs["replay"]
    if what == "replay:views":
        return " → ".join(",".join(map(str, v["corner"])) for v in obs["replay_views"]) or "none"
    if what == "replay:watched":
        box = obs.get("watch")
        if not box:
            raise ValueError("`replay:watched` needs the step's `replay = { watch = [...] }`")
        held = [v for v in obs["replay_views"] if view_holds(v["corner"], box)]
        return ", ".join(",".join(map(str, v["corner"])) + (f" ({v['dump']})" if "dump" in v
                                                              else "") for v in held) or "none"
    if what == "replay:options":
        o = obs["options"]
        return (f"vtable {o['vtable']}, item {REPLAY_ITEM} state {o['replay_state']:#x}, items "
                + " ".join(map(str, o["items"])))
    if what in ("view:before", "view:after"):
        return ",".join(map(str, obs[what.replace(":", "_")]))
    raise ValueError(f"unknown replay observation `{what}`")


# -- the hero screen (E08 T37a: batch E5's E5-3b and E5-10b) --------------------------------------
# The hero screen's artifact places (800 x 600; the README's table): the backpack's five visible
# places from the left, and each worn slot's place, by h3mem.ARTIFACT_SLOTS' names (centres of
# 44 x 44 places). Checked in the original under Wine (E08 T37a): the backpack's, the feet's and misc
# 1's; since E08 T47a every one against its item's box in memory (worn position p is item 2 + p,
# backpack place k item 40 + k: SCREEN_ITEMS), each within a pixel of the box's centre.
BACKPACK_PLACES = (425, 387, 46, 5)  # the first place's centre, the step, the places shown
SLOT_PLACES = {
    "head": (531, 52), "shoulders": (590, 264), "neck": (531, 102), "right hand": (405, 91),
    "left hand": (584, 206), "torso": (531, 153), "right ring": (453, 91),
    "left ring": (632, 206), "feet": (537, 317), "misc 1": (405, 165), "misc 2": (421, 215),
    "misc 3": (437, 265), "misc 4": (453, 315), "misc 5": (403, 318),
}


def cast_on_map(game, mem, name, plan):
    """E08 T110 (batch E12b's E12-14, E12-9): the selected hero's spell book on the adventure map
    (its key C, the panel's book button's), the book's place `place` (default 0: the hero's only
    adventure spell) clicked, the view the spell opens (View Earth's) closed with Escape, every
    message's OK clicked (its record kept). `plan`: the spell's name, or { spell, place }.
    Returns the observation: the book (`opened`, `refused` with the message's record), the screen
    the cast showed, the spell points before and after."""
    spell, place = (plan, 0) if isinstance(plan, str) else (plan["spell"], plan.get("place", 0))
    index = mem.hero_named(name)["index"]
    obs = {"hero": name, "messages": [], "cast": {"spell": spell,
                                                  "before": mem.hero(index)["spell_points"]}}
    game.move(*PARK)
    game.key(BOOK_KEY, BOOK_WAIT)
    kind = screen_kind(mem)
    if kind == "message":
        obs["cast"]["book"] = f"refused: {message_record(mem)}"
        click_item(game, mem, "ok", None)
    elif kind in ("map", None):
        obs["cast"]["book"] = "nothing (no dialog)"
    else:
        obs["cast"]["book"] = "opened"
        game.move(*h3drive.book_place(place))
        time.sleep(0.3)
        game.click(*h3drive.book_place(place), settle=1.5)
        shown = mem.dialog()
        kind = screen_kind(mem)
        obs["cast"]["view"] = (kind if kind not in ("other", None) else
                               hex(shown["vtable"]) if shown else "none")
        for _ in range(4):
            kind = screen_kind(mem)
            if kind in ("map", None):
                break
            if kind == "message":
                obs["messages"].append(message_record(mem))
                click_item(game, mem, "ok", None)
            else:
                game.key("Escape", 1.0)
        if screen_kind(mem) not in ("map", None):
            game.dump(game.out / "cast-stuck.xwd")
            raise RuntimeError(f"the cast's view does not close (the display: "
                               f"{game.out / 'cast-stuck.xwd'})")
    game.move(*PARK)
    quiet_screen(game, 0.5)
    obs["cast"]["after"] = mem.hero(index)["spell_points"]
    return obs


def open_hero_screen(game, mem, name):
    """Selects the hero and opens its hero screen by a click on it on the map."""
    hero = select(game, mem, name)
    x, y, z = hero["tile"]
    centre(game, mem, x, y, z)
    game.click(*hover(game, mem, x, y, z), 0.5)
    game.wait("hero_screen", 10)
    return hero


def equip(game, mem, name, moves):
    """Puts artifacts on from the backpack in the hero's screen (E08 T37a): for each artifact (by
    name, in `moves`' order) a click on its place in the backpack (the first five shown; the
    cursor takes it) and one on the slot (`moves`: {artifact: slot}), checked in the game's
    memory; then the screen closed (Enter) and the adventure map back. The original's own
    equipping (hero_equip_artifact 0x4E2A00) runs, as a player's would."""
    index = open_hero_screen(game, mem, name)["index"]
    for artifact, slot in moves.items():
        hero = mem.hero(index)
        names = [mem.artifact_name(a) for a in hero["backpack"]]
        if artifact not in names:
            raise ValueError(f"{name}'s backpack holds no {artifact} ({', '.join(names)})")
        place = names.index(artifact)
        x0, y0, step, shown = BACKPACK_PLACES
        if place >= shown:
            raise ValueError(f"{artifact} is the backpack's {place + 1}th: not shown")
        if slot not in SLOT_PLACES:
            raise ValueError(f"unknown slot `{slot}` ({', '.join(SLOT_PLACES)})")
        if slot in hero["worn"]:
            raise ValueError(f"{name}'s {slot} holds {mem.artifact_name(hero['worn'][slot])}")
        game.click(x0 + step * place, y0, 0.5)
        game.click(*SLOT_PLACES[slot], 0.5)
        game.move(*PARK)
        wanted = hero["backpack"][place]
        wait_for(lambda: mem.hero(index)["worn"].get(slot) == wanted,
                 f"{artifact} on {name}'s {slot}")
    game.key("Return", 0.5)
    game.wait("adventure", 10)
    return {"hero": name, "equipped": dict(moves)}


# -- screens and their items (E08 T47a: batch E6) --------------------------------------------
# The topmost dialog's kind by its vtable (h3mem.dialog), observed under Wine (E08 T47a): what a
# click lands on is found by the dialog's items in memory (their ids and boxes), never by fixed
# coordinates or probes. The adventure map and the battle are dialogs too.
SCREEN_KINDS = {0x63A5E4: "map", 0x63D528: "battle", 0x63EAE8: "hero", 0x642428: "exchange",
                0x643854: "army", 0x643C14: "creature", 0x63B8F8: "split", 0x63DB40: "message",
                0x643908: "boat", 0x63FE60: "level_up"}
# E09 T3b (batch TWN-1): the town screen, its vtable 0x64372C (town_dialog_create 0x5C34D0, the
# town manager's dialog +0x118), read live under Wine with a town opened from the town list: an
# 800 x 600 dialog whose exit button (the check mark, bottom right) is item 30720, as the hero
# screen's close. A walk notes it (`town_screen`) and closes it by that item, else the key
# TOWN_SCREEN_KEY (Return and Escape close it too); every other unknown screen's vtable is noted
# (`unknown_screens`). The level-up dialog's vtable 0x63FE60 (level_up_dialog 0x4F8880 sets it;
# noted unknown by TWN-1-3b's first runs) is named `level_up`.
TOWN_SCREEN_VTABLE = 0x64372C
TOWN_SCREEN_KEY = "Return"
SCREEN_KINDS[TOWN_SCREEN_VTABLE] = "town"
# E08 T90 (batches E10a, E10b): the hire dialog of a dwelling or Refugee Camp (H3RecruitMgr,
# manager vtable 0x640C70, recruit_dialog_init 0x551350; `read`, docs/exe/map.toml; the dialog
# itself 0x640C4C, observed). Its items: Max 532, Recruit 30722, Close 30721
# (recruit_dialog_proc 0x550940, `read`) and the slider 559 (observed; the amount's text 530, the
# Available's 521, the cost's 512), all listed by `hire:items`.
SCREEN_KINDS[h3mem.RECRUIT_DIALOG] = "hire"
# E09 T14b (batch TWN-2): the hall (hall_dialog_fill 0x461190, hall_dialog_proc 0x461AB0; the
# dialog's vtable 0x6437A0) and a building's dialog (build_dialog_run 0x5D5F30; 0x643944), read
# live under Wine. The town screen's buildings are its items, each id a building id, its box the
# picture's (shown: state 0x6); a click on the hall's (the shown one of 10 to 13) opens the hall,
# the panel's hall picture (item 158) does not. The hall's slot k: its picture 700 + k (its frame
# the building the slot shows), its name 600 + k, the bar under it 400 + k (frame 0 built, 1
# buildable, 2 not now, 3 never; a red cross 800 + k drawn with 2 and 3), close 30720. The
# building's dialog: Build 30722 (state 0x6 when it builds; 0x8 or 0x20 set, greyed: a click does
# nothing), Cancel 30721, the requirements' text item 5; Build closes the hall too.
HALL_VTABLE, BUILD_VTABLE = 0x6437A0, 0x643944
SCREEN_KINDS[HALL_VTABLE] = "hall"
SCREEN_KINDS[BUILD_VTABLE] = "build"
# E08 T110 (batch E12a): a garrison's exchange (garrison_exchange_create 0x5D0E00 writes the vtable
# 0x643890 over garrison_dialog_create 0x5CE830, the army dialog's base, `read`): the garrison's
# army the upper bar, the hero's the lower, the army dialog's items (garrison_dialog_create builds
# them for both). A walk plays the step's `actions` in it and closes it with its OK, as the army
# dialog's; `garrison_dialogs` notes the bars as it opened.
GARRISON_VTABLE = 0x643890
SCREEN_KINDS[GARRISON_VTABLE] = "garrison"
HALL_BUILDINGS, HALL_SLOT, HALL_BAR = range(10, 14), 700, 400
HALL_LOOKS = ("built", "buildable", "not now", "never")
# E09 T23 (batch TWN-3): the town screen's two portraits' boxes' top-left corners (58 x 64), the
# lower bar's (the visitor: item 124, its selection frame 125) and the upper's (the garrison hero
# 122 with its frame 123, or the town's flag 100), read live under Wine.
TOWN_PORTRAITS = ((241, 483), (241, 387))
TOWN_VISITOR_FRAME, TOWN_STATUS = 125, 151  # the visitor's selection frame; the status line
# E09 T23: the week's and the month's kinds and creatures (docs/exe/map.toml's globals).
WEEK_KIND, WEEK_CREATURE, MONTH_KIND, MONTH_CREATURE = 0x697750, 0x6983FC, 0x698834, 0x697748
BUILD_TEXT, BUILD_GREYED = 5, 0x28
# Each screen's items by name (survey E08-army §1.1, §2.3, §3.1-§3.4, §4, §5.1: the procedures'
# item ids; checked under Wine, E08 T47a). A name `<part>:<n>` is the part's n-th (an army slot
# from 0 at the left, a worn position by its number 0-18, a backpack place from 0 at the left of
# the five shown); a range's first id here. The hero screen (`hero_screen_proc` 0x4DD2D0): army
# slots 68-74, worn positions 2-20, backpack places 40-44, its arrows 77 (left: the last becomes
# the first) and 78 (right: the first becomes the last), loose 122, tight 124, the tactics
# formation 126, split 127, close 30720; since E08 T110 the morale and luck pictures 116 and 117
# (observed under Wine: 44 x 44 at (246, 191) and (304, 191), frame 3 neutral; a right click
# shows their lists). The exchange (`swap_manager_proc` 0x5AFDF0; left the
# visitor, right the visited): armies 13-19 and 20-26, worn positions 27-45 and 46-64, backpack
# places 89-93 and 94-98, their arrows 99 and 101, 100 and 102, the split buttons 103 and 104,
# close 30720. The army dialog (0x5D0910): the group's slots (upper) 115-121, the hero's army
# (lower) 140-146, split 154, OK 30722. The creature dialog (0x4C6C50): dismiss 30723, OK 30722.
# The split dialog (0x449E90): OK 30722, cancel 30721, the slider 6, the counts 4 (the source)
# and 5 (the target). A message: OK 30722; a question: yes 30725, no 30726; since E08 T57 a
# choice's two pictures' frames left 30729 (0x7809) and right 30730 (0x780A). The battle: the
# tactics phase's Start 30722. Since E08 T79a (batch E9a) the boat dialog of a Shipyard
# (shipyard_dialog_create 0x5D1EF0, its vtable 0x643908): build 30722 (iBUY30.def, the result
# 0x7802 that builds), cancel 30721 (iCancel.def). The army dialog is also a mine's Leave Guards
# (army_dialog_with 0x5D12B0: the mine's guards upper, the hero's army lower).
SCREEN_ITEMS = {
    "hero": {"army": 68, "worn": 2, "backpack": 40, "backpack left": 77, "backpack right": 78,
             "loose": 122, "tight": 124, "tactics": 126, "split": 127, "close": 30720,
             "morale": 116, "luck": 117},
    "exchange": {"left army": 13, "right army": 20, "left worn": 27, "right worn": 46,
                 "left backpack": 89, "right backpack": 94, "left backpack left": 99,
                 "left backpack right": 101, "right backpack left": 100,
                 "right backpack right": 102, "left split": 103, "right split": 104,
                 "close": 30720},
    "army": {"upper": 115, "lower": 140, "split": 154, "ok": 30722},
    "garrison": {"upper": 115, "lower": 140, "split": 154, "ok": 30722},
    "creature": {"dismiss": 30723, "ok": 30722},
    "split": {"ok": 30722, "cancel": 30721},
    "message": {"ok": 30722, "yes": 30725, "no": 30726, "left": 30729, "right": 30730},
    "battle": {"start": 30722},
    "boat": {"build": 30722, "cancel": 30721},
    "town": {"close": 30720},
    "hall": {"close": 30720},
    "build": {"build": 30722, "cancel": 30721},
    "hire": {"max": 532, "recruit": 30722, "close": 30721, "slider": 559},
}
# The counts each screen shows under its army slots (text items, first id): the hero screen's
# 61-67, the exchange's 65-71 and 72-78, the army dialog's 108-114 and 133-139.
SHOWN_COUNTS = {("hero", "army"): 61, ("exchange", "left army"): 65,
                ("exchange", "right army"): 72, ("army", "upper"): 108, ("army", "lower"): 133,
                ("garrison", "upper"): 108, ("garrison", "lower"): 133}
SPLIT_SLIDER, SPLIT_COUNTS = 6, (4, 5)
SHOWN = 0x4  # an item's state bit while it is drawn (an empty slot's count, an empty place: clear)
QUESTION = (30725, 30726)  # a message's yes and no
EXCHANGE_MANAGER, EXCHANGE_HEROES = 0x6A3D30, 0x40  # swap_manager_create 0x5AE500: this
SCREEN_ACTIONS = ("click", "right", "slider", "amount", "hall", "slot", "hero_in", "building")


# A message box's pictures by their DEF (E08 T57; observed at a Treasure Chest under Wine): a
# resource (resour82.def, frame the resource), a primary skill or experience (pskill.def, frames
# 0-3 the skills, 4 experience, 5 spell points), an artifact (artifact.def, frame the id), a
# secondary skill (secskill.def, frame 3 x skill + 3 + level - 1), a spell (spells.def), a scroll
# (spellscr.def), morale and luck (imrl82.def, ilck82.def: their frames as shown).
PSKILL_FRAMES = (*h3mem.PRIMARY_SKILLS, "experience", "spell points")


def message_record(mem):
    """The record a message box's text was written from (h3lod.identify: `ADVEVENT:86`, since E08
    T68b also `RANDSIGN:24`, a sign's or Ocean Bottle's default text), or first, since E08 T68b,
    `the map's text "..."` when it is the text one of the map's message records holds (batch
    E8b's E8-9: the map's own text, never the original's, ADR-0009; identify would name a text
    with " the " in it GENRLTXT:16, `%s the %s`); `?` for neither. Since E08 T79b also a text
    of the batch's spec (OWN_TEXTS: its timed events' and events' messages, batch E9b's E9-11),
    which the game keeps in records the harness does not read."""
    text = mem.dialog_text()
    lines = [" ".join(line.split()) for line in [text, *text.split("\n")]]
    for has_text, own in mem.messages():
        if has_text and " ".join(own.split()) in lines:
            return f'the map\'s text "{own}"'
    for own in OWN_TEXTS:
        if " ".join(own.split()) in lines:
            return f'the map\'s text "{own}"'
    found = h3lod.identify(text) or "?"
    if found == "GENRLTXT:16":
        # E08 T90 (batch E10b): the week's message (`%s the %s`): the creature it names, by the
        # game's creature table (ids 0 to 144, plural, then singular), never its text (ADR-0009).
        names = []
        for creature in range(145):
            try:
                names += [mem.creature_name(creature, 2), mem.creature_name(creature, 1)]
            except (OSError, IndexError, UnicodeDecodeError):
                pass
        hits = sorted({n for n in names if n and n in text}, key=len, reverse=True)
        found += f" [names {hits[0]}]" if hits else f" [other: {len(text.strip())} characters]"
    return found


# The batch's own message texts (E08 T79b; batch sets them from its spec): message_record names a
# message with one of them `the map's text "..."`.
OWN_TEXTS = []

# The recipe's named places (E08 T100b, batch E11b; batch sets them from its `[places]`): a name
# for a tile (`M1 = [26, 4]`) or a box of tiles (`W1 = [22, 15, 24, 16]`), each with an optional
# level last (`[x, y, level]`, `[x0, y0, x1, y1, level]`); `hero:place` and the presses' rows
# name a tile by the first that holds it.
PLACES = {}


def place_name(tile):
    """The name of the recipe's first place holding `tile` ([x, y] or [x, y, level]), else the
    tile itself (`x,y`, `x,y,level` underground)."""
    x, y, z = (*tile, 0)[:3]
    for name, where in PLACES.items():
        if len(where) in (2, 3):
            if [x, y, z] == [*where, 0][:3]:
                return name
        elif len(where) in (4, 5):
            x0, y0, x1, y1, level = [*where, 0][:5]
            if z == level and x0 <= x <= x1 and y0 <= y <= y1:
                return name
        else:
            raise ValueError(f"place {name} = {where}: [x, y(, level)] or "
                             "[x0, y0, x1, y1(, level)]")
    return tile_name(tile)


def tile_name(tile):
    """A tile as the records write it: `x,y` on the surface, `x,y,level` underground."""
    x, y, z = (*tile, 0)[:3]
    return f"{x},{y}" + (f",{z}" if z else "")


def own_texts(spec):
    """The messages a spec's map shows that are its own (E08 T79b): its timed events' and
    events' `message`s."""
    found = tomllib.loads(Path(spec).read_text()).get("map", {})
    return [e["message"] for kind in ("timed_event", "event") for e in found.get(kind, [])
            if e.get("message")]


def picture_text(mem, picture):
    """A message box's picture as the templates take it: its caption and what it shows (`1000
    gold`, `525 experience`, `Centaurs Axe`, `Advanced Wisdom`), else its caption and DEF."""
    name, frame, text = picture["def"], picture["frame"], " ".join(picture["text"].split())
    if any(c.isalpha() for c in text):
        return text  # a caption that names what it shows (`500 Spell Points`, `Advanced Wisdom`)
    if name == "resour82.def" and 0 <= frame < len(h3mem.RESOURCES):
        return f"{text} {h3mem.RESOURCES[frame]}".strip()
    if name == "pskill.def" and 0 <= frame < len(PSKILL_FRAMES):
        return f"{text} {PSKILL_FRAMES[frame]}".strip()
    if name == "artifact.def":
        return mem.artifact_name(frame)
    if name == "spells.def":
        return mem.spell_name(frame)
    if name == "spellscr.def":
        return f"Spell Scroll ({mem.spell_name(frame)})"
    if name in ("imrl82.def", "ilck82.def"):
        kind = "morale" if name.startswith("imrl") else "luck"
        return f"{text} {kind} (frame {frame})".strip()
    if name in ("secskill.def", "secsk82.def") and frame >= 3:
        skill, level = divmod(frame - 3, 3)
        return text or f"{LEVELS[level + 1].capitalize()} {h3mem.SKILLS[skill]}"
    return f"{text} ({name} {frame})".strip()


def pictures_text(mem):
    """The topmost dialog's pictures (h3mem.dialog_pictures), `none` for none."""
    return ", ".join(picture_text(mem, p) for p in mem.dialog_pictures()) or "none"


def choice_sides(mem):
    """A choice's two sides (E08 T57): {left, right: what each picture shows}, {} for a dialog
    without a choice's pictures."""
    sides = {}
    for picture in mem.dialog_pictures():
        side = {h3mem.CHOICE_FIRST: "left", h3mem.CHOICE_FIRST + 1: "right"}.get(
            picture["choice"])
        if side:
            sides[side] = picture_text(mem, picture)
    return sides if set(sides) == {"left", "right"} else {}


def screen_kind(mem):
    """The kind of the topmost dialog (SCREEN_KINDS), `other` for an unknown one, None for none."""
    shown = mem.dialog()
    return SCREEN_KINDS.get(shown["vtable"], "other") if shown else None


def hero_index(mem, pointer):
    return (pointer - mem.game() - h3mem.HEROES) // h3mem.HERO_SIZE


def screen_heroes(mem, kind, step_hero):
    """Whose screen it is (E08 T47a): the hero screen's and the army dialog's the step's hero,
    the exchange's left the visitor and right the visited (its manager +0x40, +0x44)."""
    if kind == "exchange":
        manager = mem.u32(EXCHANGE_MANAGER)
        left, right = (mem.u32(manager + EXCHANGE_HEROES + 4 * k) for k in range(2))
        return {"left": hero_index(mem, left), "right": hero_index(mem, right)}
    return {"": step_hero}


def item_id(mem, kind, name, heroes):
    """The id of the item `name` on a screen of `kind`: `<part>` or `<part>:<n>`, where for a worn
    position or a backpack place n may be an artifact's name (the position that holds it, or the
    place that shows it: the backpack's first five entries; E08 T47a)."""
    items = SCREEN_ITEMS.get(kind, {})
    part, _, arg = name.partition(":")
    if part not in items:
        raise ValueError(f"no item `{name}` on the {kind} screen ({', '.join(items)})")
    if not arg:
        return items[part]
    if arg.lstrip("-").isdigit():
        return items[part] + int(arg)
    side = part.split()[0] if kind == "exchange" else ""
    hero = mem.hero(heroes[side])
    names = {a: mem.artifact_name(a) for a in [*hero["positions"].values(), *hero["backpack"]]}
    if part.endswith("worn"):
        found = [p for p, a in hero["positions"].items() if names[a] == arg]
    elif part.endswith("backpack"):
        found = [k for k, a in enumerate(hero["backpack"][:5]) if names[a] == arg]
    else:
        raise ValueError(f"`{name}`: only a worn position or a backpack place takes a name")
    if not found:
        raise ValueError(f"`{name}`: {hero['name']} shows no {arg} there")
    return items[part] + found[0]


def item_centre(mem, wanted):
    for item in mem.dialog_items():
        if item["id"] == wanted:
            x, y, width, height = item["box"]
            return x + width // 2, y + height // 2
    raise RuntimeError(f"the topmost dialog has no item {wanted}")


def quiet_screen(game, pause=0.4):
    time.sleep(pause)
    game.settled()


def click_item(game, mem, name, heroes):
    """A left click on the item `name` of the screen showing (its box's centre); the mouse
    parked after it."""
    kind = screen_kind(mem)
    game.click(*item_centre(mem, item_id(mem, kind, name, heroes)), 0.3)
    game.move(*PARK)
    quiet_screen(game)


def question_record(mem):
    """A question showing (a message with yes and no): the record its text was written from (or
    `?`); None when none shows."""
    if screen_kind(mem) != "message":
        return None
    ids = {i["id"] for i in mem.dialog_items()}
    if not set(QUESTION) <= ids:
        return None
    return h3lod.identify(mem.dialog_text()) or "?"


def answer_question(game, mem, answer, heroes):
    """Answers the question showing with `answer` (yes or no); returns its record."""
    record = question_record(mem)
    if answer not in ("yes", "no"):
        raise RuntimeError(f"a question ({record}) the action does not answer (`answer`)")
    click_item(game, mem, answer, heroes)
    return record


# E08 T110 (batch E12b's E12-11: the hero screen's morale and luck windows): the lines of the
# last right click's window, each by the records that format it (popup_lines).
LAST_POPUP = []
LINE_TABLES = ("ARRAYTXT.TXT", "GENRLTXT.TXT")


def popup_lines(text):
    """A window's text line by line, each named by the records of LINE_TABLES that format it
    (`ARRAYTXT:96`; several `ARRAYTXT:68|93`), else `[other: n characters]` (ADR-0009: names,
    not the text)."""
    out = []
    for line in (line.strip() for line in text.split("\n")):
        if not line:
            continue
        found = []
        for name in LINE_TABLES:
            try:
                hits = h3lod.matches(name, line)
            except (KeyError, OSError):
                continue
            if hits:
                found.append(f"{name.split('.')[0]}:{'|'.join(hits)}")
        out.append(", ".join(found) or f"[other: {len(line)} characters]")
    return out


def right_click(game, mem, name, heroes, answer):
    """A right click on the item `name` (E08 T47a): the button held while the screen is read. A
    question (the assembly's, GENRLTXT 733) stays after the release and is answered; a window
    shown while the button is held (an artifact's or a creature's) goes with it. Returns what
    opened: `question <record>`, `window <record>` (`?` for a text not identified) or `none`, and
    the question's record if one came."""
    kind = screen_kind(mem)
    before = mem.dialog()["at"]
    game.move(*item_centre(mem, item_id(mem, kind, name, heroes)))
    time.sleep(0.1)
    h3drive.run("xdotool", "mousedown", "3")
    time.sleep(0.6)
    shown = mem.dialog()
    opened = shown is not None and shown["at"] != before
    record = (h3lod.identify(mem.dialog_text()) or "?") if opened else None
    # E08 T110 (batch E12b's E12-11): the window's text line by line (popup_lines).
    LAST_POPUP[:] = popup_lines(mem.dialog_text()) if opened else []
    asked = question_record(mem) if opened else None
    h3drive.run("xdotool", "mouseup", "3")
    game.move(*PARK)
    quiet_screen(game)
    if asked is not None:
        answer_question(game, mem, answer, heroes)
        return f"question {asked}", asked
    return (f"window {record}" if opened else "none"), None


def slider_max(game, mem):
    """The split dialog's slider to its maximum (toward the target): its right arrow clicked until
    the counts stop changing (E08 T47a; the arrow moves one creature a click)."""
    x, y, width, height = next(i["box"] for i in mem.dialog_items() if i["id"] == SPLIT_SLIDER)
    arrow = (x + width - height // 2, y + height // 2)
    last, same = split_counts(mem), 0
    for _ in range(100000):
        game.click(*arrow, 0.05)
        now = split_counts(mem)
        same = same + 1 if now == last else 0
        if same >= 2:
            break
        last = now
    game.move(*PARK)
    quiet_screen(game)


def hire_amount(game, mem, wanted):
    """The hire dialog's amount set to `wanted` by its slider (E08 T90, E10-8's 2): a click on
    the slider's right end adds one, on its left end takes one (observed: `h3mem.SLIDER_VTABLE`,
    id 559); the amount read from the dialog's manager (+0xB8) after each click. Fails when the
    amount cannot be reached (it stops at the most)."""
    for _ in range(1000):
        state = mem.recruit_dialog()
        if state is None:
            raise RuntimeError("no hire dialog shows (`amount`)")
        if state["amount"] == wanted:
            break
        if state["slider"] is None:
            raise RuntimeError("the hire dialog has no slider item")
        x, y, width, height = next(i["box"] for i in state["items"] if i["id"] == state["slider"])
        end = x + width - height // 2 if state["amount"] < wanted else x + height // 2
        before = state["amount"]
        game.click(end, y + height // 2, 0.15)
        if mem.recruit_dialog()["amount"] == before:
            raise RuntimeError(f"the amount stays at {before} (the most is {state['most']}) "
                               f"short of {wanted}")
    game.move(*PARK)
    quiet_screen(game)


def split_counts(mem):
    """The split dialog's two counts as it shows them, `<source> : <target>`."""
    if screen_kind(mem) != "split":
        raise RuntimeError("no split dialog shows")
    texts = {i["id"]: i["text"] for i in mem.dialog_items()}
    return " : ".join((texts.get(k) or "").strip() for k in SPLIT_COUNTS)


def shown_counts(mem, part, heroes=None):
    """The counts a screen shows under the army slots `part` (SHOWN_COUNTS): `slot: count` for
    each slot that shows one (`none`)."""
    kind = screen_kind(mem)
    if (kind, part) not in SHOWN_COUNTS:
        raise ValueError(f"the {kind} screen shows no counts of `{part}`")
    first = SHOWN_COUNTS[(kind, part)]
    texts = {i["id"]: i["text"] for i in mem.dialog_items() if i["state"] & SHOWN}
    shown = [(k, (texts.get(first + k) or "").strip()) for k in range(7)]
    return ", ".join(f"{k}: {t}" for k, t in shown if t) or "none"


def shown_backpack(mem):
    """The hero screen's five backpack places as it shows them, left to right (their pictures'
    frames: the artifacts' ids), `empty` for an empty place."""
    if screen_kind(mem) != "hero":
        raise RuntimeError("the hero screen does not show")
    frames = {i["id"]: i["frame"] for i in mem.dialog_items()
              if i["vtable"] == h3mem.PICTURE_VTABLE and i["state"] & SHOWN}
    first = SCREEN_ITEMS["hero"]["backpack"]
    hero_items = [frames.get(first + k, -1) for k in range(5)]
    return ", ".join(mem.artifact_name(a) if a > 0 else "empty" for a in hero_items)


# E08 T110 (batch E12a's E12-3, observed under Wine): the army dialog's and a garrison exchange's
# bar slots (upper 115-121, lower 140-146) have the state 0x2, a selected (highlighted) slot 0x6;
# a click on a stack selects it, a move clears it, a refused move or take leaves it.
BAR_SELECTED = 0x4


def bar_selection(mem):
    """The bar slot a garrison exchange or army dialog shows selected (E08 T110): `upper 0`,
    `lower 1`, `none`; None on another screen."""
    kind = screen_kind(mem)
    if kind not in ("army", "garrison"):
        return None
    items = SCREEN_ITEMS[kind]
    states = {i["id"]: i["state"] for i in mem.dialog_items()}
    found = [f"{part} {k}" for part in ("upper", "lower") for k in range(7)
             if states.get(items[part] + k, 0) & BAR_SELECTED]
    return ", ".join(found) or "none"


def run_actions(game, mem, step_hero, actions):
    """Plays a step's `actions` in the screens showing (E08 T47a): each action one `click` (a list
    of item names, clicked in order on whichever screen shows at each), `right` (an item's name)
    or `slider = "max"` (the split dialog's) or, since E08 T90, `amount = <n>` (the hire dialog's
    slider set to n), since E09 T14b `hall = true` (the town screen's hall opened) and `slot =
    <building id>` (the hall's slot showing it clicked: the building's dialog), since E09 T23
    `hero_in = true` (the town screen's visitor moved into the garrison: its portrait, then the
    garrison's) and `building = <building id>` (the town screen's building clicked: a dwelling's
    hire dialog), or none (only reads); a question that comes is
    answered with the action's `answer`, a message's OK clicked (its record kept). `record` and
    `note` read right after the action, as a step's do (the step's hero before and after it).
    Returns each action's observation: what it clicked, the questions' and messages' records,
    what a right click opened, the screen showing after it, its records and notes."""
    done = []
    for action in actions:
        kinds = [k for k in SCREEN_ACTIONS if k in action]
        if len(kinds) > 1 or set(action) - {*SCREEN_ACTIONS, "answer", "record", "note"}:
            raise ValueError(f"an action does one of {', '.join(SCREEN_ACTIONS)}: {action}")
        obs = {"action": action, "questions": [], "messages": [],
               "before": snapshot(mem, step_hero),
               # E08 T110 (batch E12a's E12-3): the bars' selected slot before the clicks.
               "selected_before": bar_selection(mem), "creature_windows": []}
        names = action.get("click", [])
        names = [names] if isinstance(names, str) else names
        for k, name in enumerate(names):
            heroes = screen_heroes(mem, screen_kind(mem), step_hero)
            was = screen_kind(mem)
            click_item(game, mem, name, heroes)
            pass_dialogs(game, mem, obs, action.get("answer"), heroes)
            if was in ("army", "garrison"):
                obs.setdefault("clicks", []).append((name, bar_selection(mem)))
            following = names[k + 1].partition(":")[0] if k + 1 < len(names) else None
            if was in ("army", "garrison") and screen_kind(mem) == "creature" \
                    and following not in SCREEN_ITEMS["creature"]:
                # E08 T110 (batch E12a's E12-3): a second click on a selected stack opens the
                # creature's window: noted (after which click) and closed with its OK, unless the
                # action's next click is the window's own (the harness check's dismissal).
                obs["creature_windows"].append(name)
                click_item(game, mem, "ok", heroes)
        if "right" in action:
            heroes = screen_heroes(mem, screen_kind(mem), step_hero)
            obs["shown"], asked = right_click(game, mem, action["right"], heroes,
                                              action.get("answer"))
            obs["popup_lines"] = list(LAST_POPUP)  # E08 T110
            if asked is not None:
                obs["questions"].append({"record": asked, "kind": "other",
                                         "answer": action["answer"]})
            pass_dialogs(game, mem, obs, action.get("answer"), heroes)
        if action.get("hall"):
            open_hall(game, mem)  # E09 T14b
        if "slot" in action:
            open_slot(game, mem, int(action["slot"]))  # E09 T14b
        if action.get("hero_in"):
            # E09 T23 (batch TWN-3): the visitor's portrait, then the garrison's
            heroes = screen_heroes(mem, screen_kind(mem), step_hero)
            hero_into_garrison(game, mem)
            pass_dialogs(game, mem, obs, action.get("answer"), heroes)
        if "building" in action:
            open_building(game, mem, int(action["building"]))  # E09 T23: a dwelling's hire
        if "amount" in action:
            hire_amount(game, mem, int(action["amount"]))  # E08 T90: the hire dialog's slider
        if "slider" in action:
            if action["slider"] != "max":
                raise ValueError(f"`slider = {action['slider']!r}`: only \"max\"")
            slider_max(game, mem)
        obs["screen"] = screen_kind(mem)
        obs["after"] = snapshot(mem, step_hero)
        # E08 T110 (batch E12a's E12-3): a `row:<table>` field adds a table row (record_value).
        obs["record"] = {field: record_value(obs, field, what, mem)
                         for field, what in action.get("record", {}).items()}
        obs["notes"] = [f"{label}: {got}" for label, what in action.get("note", {}).items()
                        if (got := value(obs, what, mem)) != "none"]
        done.append(obs)
    return done


def open_hall(game, mem):
    """E09 T14b (batch TWN-2): on the town screen, a click on the hall's building (the shown one of
    the items 10 to 13, its box's centre); the hall must open."""
    if screen_kind(mem) != "town":
        raise RuntimeError(f"`hall`: no town screen shows ({screen_kind(mem)})")
    shown = [i for i in mem.dialog_items() if i["id"] in HALL_BUILDINGS and i["state"] == 0x6]
    if len(shown) != 1:
        raise RuntimeError(f"`hall`: the town screen shows {len(shown)} hall buildings")
    x, y, width, height = shown[0]["box"]
    game.click(x + width // 2, y + height // 2, 0.5)
    game.move(*PARK)
    wait_for(lambda: screen_kind(mem) == "hall", lambda: "`hall`: the hall did not open "
             f"({screen_kind(mem)}; the town screen's building {shown[0]['id']} clicked)", 5.0)
    quiet_screen(game)


def hall_slot(mem, building):
    """E09 T14b: the hall's slot (from 0) whose picture shows `building`, None for none."""
    return next((i["id"] - HALL_SLOT for i in mem.dialog_items()
                 if HALL_SLOT <= i["id"] < HALL_SLOT + 100 and i["frame"] == building), None)


def open_slot(game, mem, building):
    """E09 T14b (batch TWN-2): in the hall, a click on the slot that shows `building`; its
    building's dialog must open."""
    if screen_kind(mem) != "hall":
        raise RuntimeError(f"`slot = {building}`: no hall shows ({screen_kind(mem)})")
    slot = hall_slot(mem, building)
    if slot is None:
        raise RuntimeError(f"`slot = {building}`: no slot of the hall shows that building")
    game.click(*item_centre(mem, HALL_SLOT + slot), 0.5)
    game.move(*PARK)
    wait_for(lambda: screen_kind(mem) == "build", lambda: f"`slot = {building}`: its dialog "
             f"did not open ({screen_kind(mem)})", 5.0)
    quiet_screen(game)


def hero_into_garrison(game, mem):
    """E09 T23 (batch TWN-3; survey E09-growth.md §9, move 7): on the town screen, the visitor's
    portrait (the lower bar's, TOWN_PORTRAITS) selected and the garrison's (the upper bar's: the
    garrison hero's or the town's flag) clicked; a message that comes (a refusal) is left for the
    caller. Read live under Wine: the lower portrait is item 124 (its selection frame 125, shown
    once selected), the upper the flag 100 or the garrison hero's portrait 122 (its frame 123),
    each 58 x 64; a click acts in the mode the mouse's hover set (`town_hero_hover` 0x5C7250), so
    each click waits until the screen's status line (item 151) shows the hover's text (a click
    right after one move was lost: the hover not yet run)."""
    if screen_kind(mem) != "town":
        raise RuntimeError(f"`hero_in`: no town screen shows ({screen_kind(mem)})")
    lower, upper = ((x + 29, y + 32) for x, y in TOWN_PORTRAITS)
    town_hover(game, mem, *lower)
    game.click(*lower, 0.5)
    wait_for(lambda: item_state(mem, TOWN_VISITOR_FRAME) == 0x6,
             "`hero_in`: the visitor's portrait was not selected", 3.0)
    town_hover(game, mem, *upper)
    game.click(*upper, 1.0)
    game.move(*PARK)
    quiet_screen(game)


def town_hover(game, mem, x, y, limit=5.0):
    """E09 T23: the mouse moved onto (x, y) of the town screen (from its middle, a pixel either way
    in turn) until the status line (item 151) changes: the hover ran."""
    game.move(400, 300)
    time.sleep(0.3)
    before = item_text(mem, TOWN_STATUS)
    end, k = time.monotonic() + limit, 0
    while time.monotonic() < end:
        game.move(x + k % 2, y)
        k += 1
        time.sleep(0.25)
        if (text := item_text(mem, TOWN_STATUS)) and text != before:
            return
    raise RuntimeError(f"the town screen's status line did not change over ({x}, {y})")


def item_state(mem, item):
    """E09 T23: the topmost dialog's item's state, None without it."""
    return next((i["state"] for i in mem.dialog_items() if i["id"] == item), None)


def item_text(mem, item):
    """E09 T23: the topmost dialog's text item's text, None without it."""
    return next((i["text"] for i in mem.dialog_items() if i["id"] == item), None)


def open_building(game, mem, building):
    """E09 T23 (batch TWN-3): on the town screen, a click on the building `building` (its item, id
    the building id, shown: state 0x6; at its box's centre) and the hire dialog it opens (a
    dwelling's or a horde's: survey E09-growth.md §8)."""
    if screen_kind(mem) != "town":
        raise RuntimeError(f"`building = {building}`: no town screen shows ({screen_kind(mem)})")
    shown = [i for i in mem.dialog_items() if i["id"] == building and i["state"] == 0x6]
    if len(shown) != 1:
        raise RuntimeError(f"`building = {building}`: the town screen shows it {len(shown)} times")
    x, y, width, height = shown[0]["box"]
    game.click(x + width // 2, y + height // 2, 0.5)
    game.move(*PARK)
    wait_for(lambda: screen_kind(mem) == "hire", lambda: f"`building = {building}`: no hire "
             f"dialog opened ({screen_kind(mem)})", 5.0)
    quiet_screen(game)


def hall_text(obs, what, mem):
    """E09 T14b (batch TWN-2), read live: `hall:slot:<id>` the look of the hall's slot showing
    building <id> (`built`, `buildable`, `not now`, `never`, `bar <frame>`; `no slot`, `no hall`);
    `build:button` the building's dialog's Build (`enabled (state 0x6)`, `greyed (state 0x26)`);
    `build:text` its requirements' text as the record it was written from and the building ids
    it names (`GENRLTXT:53, names 38`; h3lod.building_names of the town the step's hero visits:
    the names stay out of the record), `no dialog` without one."""
    if what.startswith("hall:"):
        if screen_kind(mem) != "hall":
            return "no hall"
        slot = hall_slot(mem, int(what.split(":")[2]))
        if slot is None:
            return "no slot"
        frame = next(i["frame"] for i in mem.dialog_items() if i["id"] == HALL_BAR + slot)
        return HALL_LOOKS[frame] if 0 <= frame < len(HALL_LOOKS) else f"bar {frame}"
    if screen_kind(mem) != "build":
        return "no dialog"
    items = {i["id"]: i for i in mem.dialog_items()}
    if what == "build:button":
        state = items[SCREEN_ITEMS["build"]["build"]]["state"]
        return f"{'greyed' if state & BUILD_GREYED else 'enabled'} (state {state:#x})"
    text = items[BUILD_TEXT]["text"] or ""
    record = h3lod.identify(text) or "?"
    hero = obs["before"]["hero"]["index"]
    town = next((t for t in mem.towns() if t["visiting_hero"] == hero), None)
    names = {} if town is None else h3lod.building_names(town["faction"])
    named = [part.strip() for line in text.split("\n")[1:] for part in line.split(",")]
    ids = sorted(b for b, n in names.items() if n in named)
    unknown = [n for n in named if n and n not in names.values()]
    return record + (f", names {', '.join(map(str, ids))}" if ids else "") + (
        f", {len(unknown)} names not known" if unknown else "")


def pass_dialogs(game, mem, obs, answer, heroes):
    """After a click: answers each question that shows (`answer`) and clicks each message's OK,
    keeping their records in `obs`."""
    while screen_kind(mem) == "message":
        if (asked := question_record(mem)) is not None:
            answer_question(game, mem, answer, heroes)
            obs["questions"].append({"record": asked, "kind": "other", "answer": answer})
        else:
            obs["messages"].append(message_record(mem))
            click_item(game, mem, "ok", heroes)


def close_screen(game, mem, kind, heroes):
    """Closes a hero screen, exchange or army dialog an action left open (its close or OK; since
    E08 T79a a boat dialog by its cancel: nothing built)."""
    if screen_kind(mem) == kind:
        click_item(game, mem, {"army": "ok", "garrison": "ok", "boat": "cancel"}.get(kind, "close"),
                   heroes)


def close_town_screen(game, mem):
    """E09 T3b (batch TWN-1): closes the town screen: a click on its exit button (item 30720),
    then, while the same dialog stays on top, the key TOWN_SCREEN_KEY. Returns how it closed:
    `click`, `key` or `open` (neither did)."""
    shown = (mem.dialog() or {}).get("at")
    game.click(*item_centre(mem, SCREEN_ITEMS["town"]["close"]), 1.0)
    game.move(*PARK)
    if (mem.dialog() or {}).get("at") != shown:
        return "click"
    game.key(TOWN_SCREEN_KEY, 1.0)
    return "key" if (mem.dialog() or {}).get("at") != shown else "open"


def unknown_vtable(mem):
    """E09 T3b: the topmost dialog's vtable in hex when SCREEN_KINDS does not know it, else None."""
    shown = mem.dialog()
    return f"{shown['vtable']:#x}" if shown and shown["vtable"] not in SCREEN_KINDS else None


def hero_screen(game, mem, name, actions):
    """A step's `screen = "hero"` (E08 T47a): the hero's screen opened (a click on it on the map),
    the actions played, the screen closed (if an action did not) and the map back."""
    index = open_hero_screen(game, mem, name)["index"]
    wait_for(lambda: screen_kind(mem) == "hero", f"{name}'s hero screen")
    done = run_actions(game, mem, index, actions)
    close_screen(game, mem, "hero", {"": index})
    game.wait("adventure", 10)
    return {"hero": name, "index": index, "actions": done}


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


def slot_text(mem, creature, count):
    """E09 T3b: an army slot as memory holds it: `5 Pikemen`, `empty` (creature -1, count 0), a
    count without a creature `5 (no creature)`."""
    if creature < 0:
        return f"{count} (no creature)" if count else "empty"
    return f"{count} {mem.creature_name(creature, count)}"


def slots_text(mem, slots):
    """E09 T3b: an army's slots that hold something (`slot 0: 12 Goblins, slot 2: 5 Wolf
    Riders`), `empty` for none."""
    return ", ".join(f"slot {k}: {slot_text(mem, t, c)}" for k, (t, c) in enumerate(slots)
                     if t >= 0 or c) or "empty"


def hero_value(mem, hero, field):
    """A hero's detail (`hero:<field>`): experience, level, primary (attack/defense/power/
    knowledge), army, skill:<name> (none, basic, advanced, expert), on_map (yes/no), tile; since
    E08 T32b1 skills (`Leadership expert, Wisdom basic` in the hero's order, `none`), seed,
    wisdom_offer, school_offer (hero +0x8F, +0x90, +0x3F), artifacts (worn: `Ballista (war
    machine 1), …` in slot order, `none`) and backpack (in its order, `none`); since E08 T32b2
    equipment (the worn artifacts but the war machines and the spell book); since E09 T3b
    slot:<n> (one army slot)."""
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
    # E08 T32b1 (batch E4).
    if field == "skills":
        return ", ".join(f"{name} {LEVELS[level]}" for name, level in hero["skill_order"]) \
            or "none"
    if field in ("seed", "wisdom_offer", "school_offer"):
        return str(hero[field])
    if field == "artifacts":
        return ", ".join(f"{mem.artifact_name(a)} ({slot})" for slot, a in hero["worn"].items()) \
            or "none"
    if field == "backpack":
        return ", ".join(mem.artifact_name(a) for a in hero["backpack"]) or "none"
    # E08 T32b2: the worn artifacts but the war machines and the spell book (every hero wears a
    # Catapult): what a starting bonus can give.
    if field == "equipment":
        return ", ".join(f"{mem.artifact_name(a)} ({slot})" for slot, a in hero["worn"].items()
                         if not slot.startswith("war machine") and slot != "spell book") or "none"
    # E08 T37a (batch E5): the points left (hero +0x4D) and the day's points (+0x49, the movement
    # bar's length), read without a walk: at a turn start, the points the turn start set.
    if field in ("points", "day_points"):
        return str(hero[field])
    # E08 T47a (batch E6): the army slot by slot, as memory holds it (`0: 10 Pikemen, 4: -25536
    # Peasants`; a count without a creature `2: -25536 (no creature)`), the worn artifacts by
    # position number (`9: Wizard's Well, 16: Catapult`), the flags (+0x48: `2 (loose, tactics
    # formation on)`) and the learned spells (+0x3EA, in the spells' order).
    if field == "slots":
        return ", ".join(f"{k}: {c} " + (mem.creature_name(t, c) if t >= 0 else "(no creature)")
                         for k, (t, c) in enumerate(hero["slots"]) if t >= 0 or c) or "none"
    # E09 T3b (batch TWN-1): one army slot (`slot:<n>`, 0 to 6: `4 Swordsmen`, `empty`).
    if m := re.match(r"slot:([0-6])$", field):
        return slot_text(mem, *hero["slots"][int(m.group(1))])
    if field == "positions":
        return ", ".join(f"{p}: {mem.artifact_name(a)}" for p, a in hero["positions"].items()) \
            or "none"
    if field == "flags":
        flags = hero["flags"]
        return (f"{flags} ({'tight' if flags & 1 else 'loose'}, tactics formation "
                f"{'on' if flags & 2 else 'off'})")
    if field == "spells":
        return ", ".join(mem.spell_name(s) for s in hero["spells"]) or "none"
    # E08 T57 (batch E7): the spell points (+0x18), the morale and luck bonuses until the next
    # battle's end (+0x11A, +0x11B, signed), the status flags (+0x105, hex: bit 21 0x200000 a
    # Warrior's Tomb's penalty taken, bit 12 0x1000 a looted Pyramid's) and the backpack's count
    # (+0x3D4).
    if field in ("spell_points", "morale_bonus", "luck_bonus", "backpack_count"):
        return str(hero[field])
    if field == "status":
        return f"{hero['status']:#x}"
    # E08 T110 (batch E12b): one bit of the status (`bit:<n>`: set, clear) and a u32 of the
    # record read live (`word:<offset>`, hex: the School of War bits +0x7B).
    if m := re.match(r"bit:(\d+)$", field):
        return "set" if hero["status"] >> int(m.group(1)) & 1 else "clear"
    if m := re.match(r"word:(0x[0-9a-fA-F]+|\d+)$", field):
        offset = int(m.group(1), 0)
        address = mem.game() + h3mem.HEROES + h3mem.HERO_SIZE * hero["index"] + offset
        return f"{mem.u32(address):#010x}"
    # E08 T100a (batch E11a): whether the hero is in a boat (status bit 18, h3mem.IN_BOAT).
    if field == "in_boat":
        return "yes" if hero["status"] & h3mem.IN_BOAT else "no"
    # E08 T100b (batch E11b): the recipe's place the hero stands on (place_name: `W2`, else the
    # tile) and its tile as a table's cell (`x,y`; tile_name).
    if field == "place":
        return place_name(hero["tile"])
    if field == "at":
        return tile_name(hero["tile"])
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


def signed(bits, width):
    return bits - (1 << width) * (bits >> (width - 1) & 1)


def held_artifact(tile):
    """The artifact a Treasure Chest, Corpse, Wagon or Warrior's Tomb holds by its word (E08 T57;
    visit_treasure_chest 0x4A6520, visit_corpse 0x4A5480, visit_wagon 0x4A69B0,
    visit_warriors_tomb 0x4A7C30), None for none or another class. Since E08 T68b a Sea Chest's
    (82: the kind in bits 0-2 2, the artifact in bits 3-12; visit_sea_chest 0x4A5030) and a
    Shipwreck Survivor's (86: the word; 0x4A52A0), survey E08-sea.md §8.3, §8.4."""
    kind, word = tile["class"], tile["word"]
    if kind == 101 and word >> 10 & 1:
        return word & 0x3FF
    if kind == 22 and word >> 16 & 1:
        return word >> 6 & 0x3FF
    if kind == 105 and word >> 13 & 3 == 3:
        return word >> 15 & 0x3FF
    if kind == 108 and word & 1:
        return word >> 13 & 0x3FF
    if kind == 82 and word & 7 == 2:
        return word >> 3 & 0x3FF
    if kind == 86:
        return word & 0x3FF
    return None


# A Flotsam's gains by its word (E08 T68a; survey E08-sea.md §8.2: the visit's switch 0x4A2470).
FLOTSAM = {0: "nothing", 1: "5 wood", 2: "5 wood, 200 gold", 3: "10 wood, 500 gold"}


def boat_text(mem, boat):
    """A boat's record (E08 T68a; structs.toml "boat"): `tile 20,4,0; in use 1; type 0; facing 2;
    owner red; captain E8-1; aboard 1` (the owner `none` for -1, the captain by its hero's name
    among the 156, `none` for -1)."""
    captain, colour = boat["hero"], boat["colour"]
    return (f"tile {','.join(map(str, boat['tile']))}; in use {boat['used']}; type {boat['type']}; "
            f"facing {boat['facing']}; owner {COLOURS[colour] if 0 <= colour < 8 else 'none'}; "
            f"captain {mem.hero(captain)['name'] if 0 <= captain < h3mem.HERO_COUNT else 'none'}"
            f"; aboard {boat['aboard']}")


def boat_value(mem, hero, what):
    """`boat` (the boat in use whose captain is the step's hero, the one aboard first, else the
    last: E08 T68a), `boat[<hero>]` the same for a hero named, `boat:x,y[,z]` the boat in use
    whose record stands on the tile, `boats:used` how many are in use; `none` for no boat. Since
    E08 T68b `boat_record:x,y[,z]` every record on the tile, in use or not (batch E8b's E8-11 and
    E8-12: a beaten hero's boat freed), ` | ` between them."""
    boats = mem.boats()
    if what == "boats:used":
        return str(sum(1 for b in boats if b["used"]))
    if m := re.match(r"boat_record:(\d+),(\d+)(?:,(\d))?$", what):
        tile = [int(m.group(1)), int(m.group(2)), int(m.group(3) or 0)]
        return " | ".join(boat_text(mem, b) for b in boats if b["tile"] == tile) or "none"
    if m := re.match(r"boat:(\d+),(\d+)(?:,(\d))?$", what):
        tile = [int(m.group(1)), int(m.group(2)), int(m.group(3) or 0)]
        found = [b for b in boats if b["used"] and b["tile"] == tile]
    else:
        found = sorted((b for b in boats if b["used"] and b["hero"] == hero["index"]),
                       key=lambda b: b["aboard"])
    return boat_text(mem, found[-1]) if found else "none"


def drawn_text(mem, hero):
    """How the view draws the hero (E08 T68a, batch E8a-water's E8-5), by h3mem.map_heroes' rule
    read from the hero's record (the original lifts its selected hero off its tile at a turn's
    start, so the tile shows no hero then): `in a boat (type t, flag c)` when its status has bit
    18 and a boat in use names it aboard, `in a boat (no boat record)` for the bit alone, else
    `on foot`."""
    if not hero["status"] & h3mem.IN_BOAT:
        return "on foot"
    found = [b for b in mem.boats() if b["used"] and b["hero"] == hero["index"] and b["aboard"]]
    if not found:
        return "in a boat (no boat record)"
    return f"in a boat (type {found[0]['type']}, flag {found[0]['colour']})"


def colour_name(colour):
    return COLOURS[colour] if 0 <= colour < len(COLOURS) else "none"


def mine_text(mem, x, y, z=0):
    """A mine's or Lighthouse's record by its entrance (E08 T79a, batch E9a; h3mem.mines):
    `record 3; owner red; resource crystal; abandoned 0; guards 0: 150 Troglodytes` (a
    Lighthouse's resource `Lighthouse (100)`; the guards slot by slot, `none`); `none` without a
    record there."""
    found = [m for m in mem.mines() if m["entrance"] == [x, y, z]]
    if not found:
        return "none"
    mine = found[0]
    kind = mine["resource"]
    resource = ("Lighthouse (100)" if kind == h3mem.LIGHTHOUSE else h3mem.RESOURCES[kind]
                if 0 <= kind < len(h3mem.RESOURCES) else str(kind))
    guards = ", ".join(f"{k}: {c} " + (mem.creature_name(t, c) if t >= 0 else "(no creature)")
                       for k, (t, c) in enumerate(mine["guards"]) if t >= 0 or c) or "none"
    return (f"record {mine['index']}; owner {colour_name(mine['owner'])}; resource {resource}; "
            f"abandoned {mine['abandoned']}; guards {guards}")


def bonus_word_text(mem, kind, x, y, z=0):
    """E08 T110 (batch E12b): a Fountain of Fortune's tile word, `value +2; team bits none (word
    0x00004000)` (its bits 13 to 16 signed, survey E08-garrisons-bonuses §10.6; the team bits 5 to
    12 by colour), or a Magic Spring's entrance, `full (bit 6 set; word 0x00000040)`; ` (under
    <hero>)` when a hero stands there (the word under it)."""
    tile = mem.tile_word(x, y, z)
    word = tile["under_word"] if "hero" in tile else tile["word"]
    under = f" (under {mem.hero(tile['hero'])['name']})" if "hero" in tile else ""
    if kind == "spring":
        return (f"{'full (bit 6 set' if word >> 6 & 1 else 'emptied (bit 6 clear'}; word "
                f"{word:#010x}){under}")
    value = word >> 13 & 0xF
    value -= 16 * (value > 7)
    teams = ", ".join(COLOURS[k] for k in range(8) if word >> (5 + k) & 1) or "none"
    return f"value {value:+d}; team bits {teams} (word {word:#010x}){under}"


def garrison_at(mem, x, y, z=0):
    """E08 T110 (batch E12a): the garrison's record whose entrance is (x, y, z) (h3mem.garrisons),
    None for none."""
    return next((g for g in mem.garrisons() if g["entrance"] == [x, y, z]), None)


def army_slots_text(mem, slots):
    """An army's slots as a bar shows them, slot by slot (`0: 2 Pikemen, 1: 10 Archangels`; a
    count without a creature `2: 5 (no creature)`), `empty` for none (E08 T110)."""
    return ", ".join(f"{k}: {c} " + (mem.creature_name(t, c) if t >= 0 else "(no creature)")
                     for k, (t, c) in enumerate(slots) if t >= 0 or c) or "empty"


def garrison_text(mem, x, y, z=0, part=None):
    """A garrison's record by its entrance (E08 T110, batch E12a; h3mem.garrisons, game +0x4E3AC):
    `record 2; owner red; stacks 0: 5 Pikemen; removable 0; entrance 29,6` (`stacks empty`);
    `garrison:x,y[,level]:owner|stacks|removable|entrance|index` one field of it; `none` without
    a record there."""
    found = garrison_at(mem, x, y, z)
    if found is None:
        return "none"
    fields = {"index": str(found["index"]), "owner": colour_name(found["owner"]),
              "stacks": army_slots_text(mem, found["army"]),
              "removable": str(found["removable"]), "entrance": tile_name(found["entrance"])}
    if part:
        return fields[part]
    return (f"record {fields['index']}; owner {fields['owner']}; stacks {fields['stacks']}; "
            f"removable {fields['removable']}; entrance {fields['entrance']}")


def dwelling_text(mem, x, y, z=0, part=None):
    """A dwelling's record by its entrance (E08 T90, batches E10a, E10b; h3mem.dwellings, game
    +0x4E39C): `record 3; class 20.1; owner none; creatures 0: 14 Pikemen (the slots with a
    creature); guards 0: 3 Green Dragons, 1: ...` (`guards none`); `none` without a record there."""
    found = mem.dwelling_at(x, y, z)
    if found is None:
        return "none"
    creatures = ", ".join(f"{k}: {c} " + mem.creature_name(t, c)
                          for k, (t, c) in enumerate(zip(found["creatures"], found["counts"]))
                          if t >= 0) or "none"
    guards = ", ".join(f"{k}: {c} " + (mem.creature_name(t, c) if t >= 0 else "(no creature)")
                       for k, (t, c) in enumerate(found["guards"]) if t >= 0 or c) or "none"
    # E08 T90 (batch E10a): `dwelling:x,y:owner|creatures|guards` is one field of the record.
    if part:
        return {"owner": colour_name(found["owner"]), "creatures": creatures,
                "guards": guards}[part]
    return (f"record {found['index']}; class {found['class']}.{found['subclass']}; "
            f"owner {colour_name(found['owner'])}; creatures {creatures}; guards {guards}")


def teams_text(mem):
    """The players' team bytes (E08 T90; game +0x1F879, by colour): `red 0, blue 0, tan 2, ...`."""
    return ", ".join(f"{COLOURS[c]} {t}" for c, t in enumerate(mem.teams()))


def objects_text(mem, x0, y0, x1, y1, z=0):
    """The map's objects anchored in a rectangle (E08 T90, E10-7; h3mem.objects): `12,30 class
    17 subclass 3` for each, in the map's order; `none`."""
    found = [o for o in mem.objects() if o["anchor"][2] == z
             and x0 <= o["anchor"][0] <= x1 and y0 <= o["anchor"][1] <= y1]
    return "; ".join(f"{o['anchor'][0]},{o['anchor'][1]} class {o['class']} subclass "
                     f"{o['subclass']}" for o in found) or "none"


def access_text(mem, x0, y0, x1, y1, z=0):
    """The tiles of a rectangle whose access byte has the blocked bit (0x01) and those with the
    visitable bit (0x10), E08 T90, E10-7 (h3mem.access): `blocked 12,30 13,30; visitable 13,30`."""
    blocked, visitable = [], []
    for y in range(y0, y1 + 1):
        for x in range(x0, x1 + 1):
            found = mem.access(x, y, z)
            if found["blocked"]:
                blocked.append(f"{x},{y}")
            if found["visitable"]:
                visitable.append(f"{x},{y}")
    return f"blocked {' '.join(blocked) or 'none'}; visitable {' '.join(visitable) or 'none'}"


def camp_text(mem, x, y, z=0):
    """A Refugee Camp's words (E08 T90, E10-9; h3mem.camp): `tile 19,10,0; creature 14
    (Pikemen); count word 7` (the tile's subclass +0x22 and word +0x0, on the entrance)."""
    found = mem.camp(x, y, z)
    try:
        name = f" ({mem.creature_name(found['creature'], found['word'])})"
    except (OSError, IndexError, UnicodeDecodeError):
        name = ""
    return (f"tile {','.join(map(str, found['tile']))}; creature {found['creature']}{name}; "
            f"count word {found['word']}")


def hire_text(state, part=None, mem=None):
    """The hire dialog as `h3mem.recruit_dialog` read it (E08 T90): `most 3; amount 0; ally 0;
    selected 0 of 14 (counts 14, 0, 0, 0); max enabled, recruit enabled, close enabled`; `closed`
    for None; part `items` lists every item (`532:0x4/0x..`, id and state), a part from
    most, amount, ally, enabled and counts one field; since E09 T23 (batch TWN-3) `creatures`
    each creature it offers with the count it reads (`14 Halberdiers, 14 Pikemen`), the one
    selected first."""
    if state is None:
        return "closed"
    if part == "creatures":
        return ", ".join(slot_text(mem, t, c) for c, t in zip(state["counts"], state["types"])
                         if t >= 0) or "none"
    if part == "items":
        return ", ".join(f"{i['id']}:{i['state']:#x}" for i in state["items"])
    if part == "enabled":
        return ", ".join(f"{k} {'enabled' if v else 'disabled'}" for k, v in state["enabled"].items())
    if part == "counts":
        return ", ".join(str(c) for c, t in zip(state["counts"], state["types"]) if t >= 0)
    if part:
        return str(state[part])
    return (f"most {state['most']}; amount {state['amount']}; ally {state['ally']}; selected "
            f"{state['index']} (counts {hire_text(state, 'counts')}); {hire_text(state, 'enabled')}")


# A hint or popup's pieces (E08 T90, E10-10): never the original's text, only which of the names
# the game holds it was made of.
TEXT_REST = re.compile(r"[\s\-,.():%\d]*")


def text_parts(mem, text, tile):
    """A hint's or popup's text described by what it holds (E08 T90, E10-10; ADR-0009: the
    harness names, it does not reproduce): the names the game holds for the object on `tile`
    (its class's name, a dwelling's name by CrGen1 or CrGen4's row, the owner's "Owned by"
    text) replaced by `[label]`, numbers and punctuation kept, any other text `[other: n
    characters]`. `none` for an empty text."""
    if not text:
        return "none"
    x, y, z = tile
    found = mem.object_at(x, y, z)
    pieces = [(f"class {found['class']} name", mem.object_name(found["class"]))]
    dwelling = mem.dwelling_at(x, y, z)
    if dwelling is not None:
        table = "CRGEN1.TXT" if dwelling["class"] == 17 else "CRGEN4.TXT"
        try:
            rows = h3lod.table(table)
            pieces.append((f"{table[:6]} row {dwelling['subclass']} name",
                           rows[dwelling["subclass"]][0]))
        except (KeyError, OSError, IndexError):
            pass
        pieces.append((f"owner {colour_name(dwelling['owner'])} \"owned by\"",
                       mem.owned_name(dwelling["owner"])))
    for colour in range(8):
        pieces.append((f"owned-by text of {COLOURS[colour]}", mem.owned_name(colour)))
    # E08 T110 (batch E12b's E12-16, E12-17): the visit marks (GENRLTXT 353 visited, 354 not
    # visited) and the bonus lines (XTRAINFO's records, E24 design §4.11) by their records.
    for name, rows in (("GENRLTXT", (353, 354)), ("XTRAINFO", None)):
        try:
            found = h3lod.table(f"{name}.TXT")
        except (KeyError, OSError):
            continue
        for row in range(len(found)) if rows is None else rows:
            if row < len(found) and found[row]:
                pieces.append((f"{name} {row}", found[row][0]))
    pieces = [(label, piece.strip()) for label, piece in pieces if piece and piece.strip()]
    out, rest = [], text
    while rest:
        hits = [(rest.find(piece), -len(piece), label, piece) for label, piece in pieces
                if piece in rest]
        if not hits:
            break
        at, _, label, piece = min(hits)
        out.append(rest[:at])
        out.append(f"[{label}]")
        rest = rest[at + len(piece):]
    out.append(rest)
    return "".join(part if TEXT_REST.fullmatch(part) or part.startswith("[")
                   else f"[other: {len(part.strip())} characters]"
                   for part in out).replace("\n", " / ")


HINT_WAIT = 3.0


def hint_at(game, mem, x, y, level=0):
    """The status bar's hint with the mouse on tile (x, y) (E08 T90, E10-10; h3mem.status_bar,
    the adventure dialog's item 200 and the message buffer 0x6973D8), after the clock's quiet
    pause: {bar, buffer, parts}, parts by `text_parts`."""
    # E08 T110 (batch E12b's first run: one hint read empty, one the tile before's): the mouse
    # parked first, then the text read until it is not empty and two reads 0.3 s apart agree (at
    # most HINT_WAIT seconds).
    game.move(*PARK)
    time.sleep(0.3)
    try:
        hover(game, mem, x, y, level)
    except ValueError:
        centre(game, mem, x, y, level)
        hover(game, mem, x, y, level)
    quiet_screen(game, 0.5)
    shown, end = mem.status_bar(), time.monotonic() + HINT_WAIT
    while time.monotonic() < end:
        time.sleep(0.3)
        again = mem.status_bar()
        if again == shown and (again["bar"] or again["buffer"]):
            break
        shown = again
    text = shown["bar"] if shown["bar"] else shown["buffer"] or ""
    if not mem.explored(x, y, level):
        # E08 T110 (batch E12b's E12-16: the Mermaids' entrance before): over a tile the current
        # player has not explored the mouse-over shows no hint (adventure_mouse_move 0x40E360:
        # the arrow alone), so the bar keeps the text before.
        return {**shown, "parts": "the tile is not explored: no hint (the bar kept the text "
                                  f"before: {text_parts(mem, text, (x, y, level))})"}
    return {**shown, "parts": text_parts(mem, text, (x, y, level))}


def popup_at(game, mem, x, y, level=0):
    """The text of the window a right click on tile (x, y) shows (E08 T90, E10-10; the button
    held while the screen is read, then released, as `right_click`): {opened, record, parts}:
    whether a dialog opened over the map, the record h3lod identifies its text as (`?`) and
    `text_parts`. A message left after the release is closed by its OK."""
    try:
        hover(game, mem, x, y, level)
    except ValueError:
        centre(game, mem, x, y, level)
        hover(game, mem, x, y, level)
    before = (mem.dialog() or {}).get("at")
    h3drive.run("xdotool", "mousedown", "3")
    time.sleep(0.8)
    shown = mem.dialog()
    opened = shown is not None and shown["at"] != before
    text = mem.dialog_text() if opened else ""
    record = (h3lod.identify(text) or "?") if opened else None
    h3drive.run("xdotool", "mouseup", "3")
    game.move(*PARK)
    quiet_screen(game)
    if screen_kind(mem) == "message":
        click_item(game, mem, "ok", None)
    return {"opened": opened, "record": record, "parts": text_parts(mem, text, (x, y, level))}


def town_text(mem, x, y, z, part):
    """E09 T3b (batch TWN-1): a part of the record of the town whose entrance is (x, y, z)
    (h3mem.town_at; docs/exe/structs.toml "town"): `owner` (+0x1: `red`, …, `neutral`),
    `visitor` (+0x10: the hero's name, `none`), `built` and `every` (+0x150, +0x158: the building
    ids ascending, `5, 7, 10, 30`, `none`), since E09 T14b `mark` (+0x2, the day's mark: `0`, `1`),
    `garrison` (+0xE0: `slot 0: 12 Goblins, slot 2: 5
    Wolf Riders`, `empty`) or `slot:<n>` (`5 Pikemen`, `empty`); `no town` without one there."""
    town = mem.town_at(x, y, z)
    if town is None:
        return "no town"
    if part == "owner":
        return "neutral" if town["owner"] < 0 else colour_name(town["owner"])
    if part == "visitor":
        hero = town["visiting_hero"]
        return mem.hero(hero)["name"] if hero >= 0 else "none"
    if part in ("built", "every"):
        return ", ".join(map(str, town["built" if part == "built" else "every_built"])) or "none"
    if part == "mark":
        return str(town["mark"])  # E09 T14b: the day's mark (+0x2)
    if part == "garrison":
        return slots_text(mem, town["army"])
    # E09 T23 (batch TWN-3): the available creatures (+0x16) and the external dwellings' counts
    # (+0x118) by the faction's index (`base 14, 9, 0, 0, 0, 0, 0; upgraded 0, …`), one index
    # (`count:<i>`), the Portal's creature and count (+0x3C, +0x40: `3 Troglodytes`, `none
    # (count 0)`) and the garrison hero (+0xC: its name, `none`).
    if part in ("counts", "dwellings"):
        n = town["recruits" if part == "counts" else "dwellings"]
        return (f"base {', '.join(map(str, n[:7]))}; upgraded {', '.join(map(str, n[7:]))}")
    if part.startswith("count:"):
        return str(town["recruits"][int(part.split(":")[1])])
    if part == "portal":
        creature, count = town["portal"]
        return slot_text(mem, creature, count) if creature >= 0 else f"none (count {count})"
    if part == "garrison_hero":
        hero = town["garrison_hero"]
        return mem.hero(hero)["name"] if hero >= 0 else "none"
    return slot_text(mem, *town["army"][int(part.split(":")[1])])


def towns_text(mem, what):
    """E09 T3b (batch TWN-1): over every town (h3mem.towns), by entrance: `every_vs_built` the
    towns whose every-built set (+0x158) is not their built one (+0x150), `(7, 7): every {5, 7,
    10, 30} built {5, 10, 30}; …` (`none`); `owned_garrisons` the owned towns' garrisons (+0xE0):
    `all 25 empty`, or `1 of 25 with a stack: (20, 61): slot 0: 1 Cavalier, …; …`."""
    def at(town):
        x, y, z = town["place"]
        return f"({x}, {y}" + (f", {z})" if z else ")")

    found = mem.towns()
    if what == "every_vs_built":
        return "; ".join(f"{at(t)}: every {{{', '.join(map(str, t['every_built']))}}} built "
                         f"{{{', '.join(map(str, t['built']))}}}" for t in found
                         if t["every_built"] != t["built"]) or "none"
    owned = [t for t in found if t["owner"] >= 0]
    held = [t for t in owned if any(c >= 0 or n for c, n in t["army"])]
    if not held:
        return f"all {len(owned)} empty"
    return (f"{len(held)} of {len(owned)} with a stack: "
            + "; ".join(f"{at(t)}: {slots_text(mem, t['army'])}" for t in held))


SHIPYARD_CLASS = 87


def shipyard_word(word):
    """A Shipyard tile's word (survey E08-economy §2.1, §5.1): the owner in its low byte (signed,
    0xFF none), the boat tile's column and row in its next two (0xFF 0xFF none)."""
    owner = word & 0xFF
    column, row = word >> 8 & 0xFF, word >> 16 & 0xFF
    tile = "none" if (column, row) == (0xFF, 0xFF) else f"{column},{row}"
    return f"owner {colour_name(owner - 256 if owner > 127 else owner)}, boat tile {tile}"


def tile_text(mem, x, y, z=0):
    """A tile's own word and class (E08 T79a, batch E9a's E9-5: each of a Shipyard's three
    tiles; h3mem.tile_word): `0xffff32ff class 87.0 (owner red, boat tile 50,31)`; a hero's tile
    names the hero and the word and class under it (its record's +0x14, +0xC)."""
    tile = mem.tile_word(x, y, z)
    word, kind = tile["word"], tile["class"]
    text = f"{word:#010x} class {kind}.{tile['subclass']}"
    if kind == SHIPYARD_CLASS:
        text += f" ({shipyard_word(word)})"
    if "hero" in tile:
        under, below = tile["under_word"], tile["under_class"]
        text += (f"; hero {mem.hero(tile['hero'])['name']}, under it {under:#010x} class "
                 f"{below}" + (f" ({shipyard_word(under)})" if below == SHIPYARD_CLASS else ""))
    return text


def object_text(mem, x, y, z=0):
    """What the object anchored at (x, y, z) holds, from its word (E08 T57, batch E7; survey
    E08-rewards.md and map.toml's visit functions): an artifact's name (5, the subclass); a
    chest's `artifact <name>` or `<n> gold` (bit 10, bits 0-9; bits 11-14 in 500s); a Corpse's
    `artifact <name>` or `empty` (bit 16, bits 6-15); a Wagon's `artifact <name>`, `<n>
    <resource>` (bits 0-4, bits 25-28) or `empty` (bit 13 clear; bit 14, bits 15-24); a Warrior's
    Tomb's `artifact <name>` or `empty` (bit 0, bits 13-22); a Shrine's spell (88-90, bits 13-22
    signed); a Witch Hut's skill (113, bits 13-19 signed); a University's four skills (104, the
    game's Universities at bits 13-24); a Pyramid's `guarded` or `looted` and its spell (63, bit 0,
    bits 13-20); `Pandora's Box`, `event`, `Spell Scroll`; `none` without an object (class 0).
    Since E08 T68a a Flotsam's gains (29, the word 0-3: survey E08-sea.md §8.2); since E08 T68b
    a Sea Chest's (82: `Sea Chest: nothing`, `1500 gold` or `artifact <name>, 1000 gold` by the
    kind in bits 0-2, §8.3), a Survivor's artifact (86, §8.4) and an Ocean Bottle's message (59:
    `the map's text "..."` or `no text`, by its record, §8.5) and `none (class 3: the coast)` for
    a coastal tile without an object. Since E08 T79b (batch E9b; survey E08-economy.md §9) a
    Windmill's (112: `Windmill: 4 sulfur` or `empty (sulfur)`, bits 13-16 and 0-3), a Water
    Wheel's (109: `500 gold` or `empty`, bits 0-4 in 500s) and a Mystical Garden's (55: `Garden 0:
    full, 5 gems` or `empty (500 gold)`, bit 10, bits 6-9, the index bits 0-4), each with its visited
    marks (bits 5-12, the teams; the Garden's are the players' masks, `gardens`); and an object a
    hero stands on is read through the hero's record (object_tile: `... (under E9-8)`)."""
    tile = object_tile(mem, x, y, z)
    text = object_word_text(mem, tile)
    return f"{text} (under {tile['hero']})" if "hero" in tile else text


def object_tile(mem, x, y, z=0):
    """mem.object_at, and when a hero stands on the object's entrance the class and word under it
    from the hero's record (+0xC, +0x14; E08 T79b, batch E9b's E9-8: a hero left on a Windmill's
    entrance until the week's start), with the hero's name; the subclass is then unknown (-1)."""
    tile = mem.object_at(x, y, z)
    if tile["class"] == h3mem.HERO_OBJECT:
        under = mem.tile_word(*tile["tile"])
        if "hero" in under:
            tile = {**tile, "class": under["under_class"], "word": under["under_word"],
                    "subclass": -1, "hero": mem.hero(under["hero"])["name"]}
    return tile


# The weekly objects' classes (E08 T79b; survey E08-economy.md §9).
WINDMILL, WATER_WHEEL, GARDEN = 112, 109, 55


def object_word_text(mem, tile):
    """What an object holds by its class and word (object_text)."""
    kind, word = tile["class"], tile["word"]
    artifact = held_artifact(tile)
    if kind in (0, -1):
        return "none"
    if kind == 3:
        # E08 T68b (batch E8b's E8-11): a coastal tile without an object (the land a boat lands
        # on, and a shore's water tile once its boat left: tile_refresh_object 0x505610).
        return "none (class 3: the coast)"
    if kind == 5:
        return mem.artifact_name(tile["subclass"])
    if kind in (22, 101, 105, 108) and artifact is not None:
        return f"artifact {mem.artifact_name(artifact)}"
    if kind == 101:
        return f"{500 * (word >> 11 & 0xF)} gold"
    if kind == 105 and word >> 13 & 1:
        return f"{word & 0x1F} {h3mem.RESOURCES[word >> 25 & 0xF]}"
    if kind in (22, 105, 108):
        return "empty"
    if kind in (88, 89, 90):
        spell = signed(word >> 13 & 0x3FF, 10)
        return mem.spell_name(spell) if spell >= 0 else "none"
    if kind == 113:
        skill = signed(word >> 13 & 0x7F, 7)
        return h3mem.SKILLS[skill] if 0 <= skill < len(h3mem.SKILLS) else "none"
    if kind == 104:
        found = mem.universities()
        index = word >> 13 & 0xFFF
        if index >= len(found):
            return f"no University {index}"
        return ", ".join(h3mem.SKILLS[k] for k in found[index])
    if kind == 63:
        return f"{'guarded' if word & 1 else 'looted'}, {mem.spell_name(word >> 13 & 0xFF)}"
    if kind == 29:
        return "Flotsam: " + FLOTSAM.get(word, f"word {word} (nothing)")
    if kind in (WINDMILL, WATER_WHEEL, GARDEN):
        marks = f"marks {word >> 5 & 0xFF:#04x}"
        if kind == WINDMILL:
            drawn = signed(word & 0xF, 4)
            resource = h3mem.RESOURCES[drawn] if 0 <= drawn < 7 else f"resource {drawn}"
            amount = word >> 13 & 0xF
            return f"Windmill: {f'{amount} {resource}' if amount else f'empty ({resource})'}; {marks}"
        if kind == WATER_WHEEL:
            gold = 500 * (word & 0x1F)
            return f"Water Wheel: {f'{gold} gold' if gold else 'empty'}; {marks}"
        resource = word >> 6 & 0xF
        gain = {5: "5 gems", 6: "500 gold"}.get(resource, f"resource {resource}")
        return f"Garden {word & 0x1F}: {'full, ' + gain if word >> 10 & 1 else f'empty ({gain})'}"
    if kind == 82:
        chest = word & 7
        if chest == 2:
            return f"Sea Chest: artifact {mem.artifact_name(artifact)}, 1000 gold"
        return "Sea Chest: " + {0: "nothing", 1: "1500 gold"}.get(chest, f"kind {chest}")
    if kind == 86:
        return f"Survivor: artifact {mem.artifact_name(artifact)}"
    if kind == 59:
        found = mem.messages()
        if not 0 <= word < len(found):
            return f"Ocean Bottle: no message record {word}"
        has_text, text = found[word]
        return f'Ocean Bottle: the map\'s text "{text}"' if has_text else "Ocean Bottle: no text"
    return {6: "Pandora's Box", 26: "event", 93: "Spell Scroll"}.get(kind, f"class {kind}")


# The observations read from the game's memory as it stands, not from a step's snapshot: under
# `before:` they are read right before the step (E08 T57: `pre_read`), else they would show the
# state after it.
LIVE_READS = ("explored", "word", "object", "monster", "aggression", "never_flees", "boat",
              "boats", "drawn", "boat_record", "mine", "tile", "income", "gardens", "growth",
              "town", "towns",  # E09 T3b: the towns' records
              "dwelling", "teams", "objects", "access", "camp", "hire")  # E08 T90


def observations(what):
    """The observations a record's value names: one, a list of them, a `row:` field's cells (a
    cell itself one or a list) and a grid's rows (`{ rows = [...] }`; E08 T68b)."""
    if isinstance(what, dict):
        what = what.get("rows", [])
    if isinstance(what, list):
        return [w for item in what for w in observations(item)]
    return [what]


def record_value(obs, field, what, mem):
    """A record field's value from the step's observation (value); for a `row:<table>` field
    (E08 T68b) the rows it adds to that table (fill), each a list of cells: one row, or a grid's
    rows (`{ rows = [...] }`, expand)."""
    if not field.startswith("row:"):
        return value(obs, what, mem)
    if isinstance(what, str):
        # E08 T79a (batch E9a's E9-1): an observation that gives rows itself (`opening:slots:`).
        got = value(obs, what, mem)
        return got if isinstance(got, list) else [[got]]
    rows = what["rows"] if isinstance(what, dict) else [what]
    return [[value(obs, cell, mem) for cell in row] for row in rows]


def pre_read(step, before, mem):
    """The step's `before:` observations (its `record`'s and `note`'s), read right before the
    step from its snapshot and the game's memory as it stands (E08 T57: an object's word before a
    visit)."""
    wanted = [w for what in [*step.get("record", {}).values(), *step.get("note", {}).values()]
              for w in observations(what) if w.startswith("before:")]
    obs = {"before": before, "after": before}
    return {w: value(obs, w.split(":", 1)[1], mem) for w in wanted}


def opening_rows(mem, battle):
    """A battle's stacks at its opening (E08 T57: batch E7's E7-16 and E7-22): side, army slot,
    hex, creature and count, a row each in the records' order."""
    return [[SIDES[s["side"]], str(s["army_slot"]), str(s["hex"]),
             mem.creature_name(s["creature"], s["count"]), str(s["count"])]
            for s in battle["opening"]]


def value(obs, what, mem):
    """A record's value from a step's observation (the game's memory right after the step for
    `monster:`, `aggression:` and another hero's details)."""
    yes = {True: "yes", False: "no"}
    if isinstance(what, str) and what.startswith("text:"):
        return what[5:]  # E08 T68b: a literal (a row's tile)
    if isinstance(what, list):
        # E08 T57: several observations in one field, joined by `; `; since E09 T3b a literal
        # ending in `: ` labels the value after it (`["text:game 1: ", "towns:every_vs_built"]`:
        # `game 1: none`).
        out = ""
        for k, w in enumerate(what):
            label = k > 0 and isinstance(what[k - 1], str) and what[k - 1].startswith("text:") \
                and what[k - 1].endswith(": ")
            out += ("" if k == 0 or label else "; ") + value(obs, w, mem)
        return out
    if what in obs.get("pre", {}):
        return obs["pre"][what]  # E08 T57: read right before the step (`pre_read`)
    if what.startswith("before:"):
        # E09 T3b: another hero's details (`hero[<name>]:`) are read live too.
        if what.split(":")[1] in LIVE_READS or what.split(":")[1].startswith("hero["):
            # The snapshot holds no tiles: the read would be the state after the step.
            raise ValueError(f"`{what}`: read a tile before a step in a step of its own")
        return value({"before": obs["before"], "after": obs["before"]}, what.split(":", 1)[1], mem)
    after = obs.get("after")  # E08 T79b: an end-turn step has no hero
    if what in ("arrived", "moved"):
        return yes[obs[what]]
    if what == "points" and "points" not in obs:
        return str(after["hero"]["points"])
    if what in ("mark", "marks", "points"):
        return str(obs[what])
    if what == "points_if_moved":
        return str(obs["points"]) if obs["moved"] else ""
    # E08 T37b (batch E5b's E5-11): the tiles the path's marks lay on at the first click, by row
    # then column (`31,12 31,13 31,14 31,15`), or those of one row (`path_tiles:14`); `none`.
    if m := re.match(r"path_tiles(?::(\d+))?$", what):
        row = m.group(1)
        return " ".join(f"{x},{y}" + (f",{z}" if z else "") for x, y, z in obs["tiles"]
                        if row is None or y == int(row)) or "none"
    if what.startswith("resource:"):
        _, name, *colour = what.split(":")
        owner = colour[0] if colour else COLOURS[obs["before"]["hero"]["owner"]]
        return str(after["resources"][owner][name])
    if what == "difficulty":
        return mem.difficulty()
    if what == "map_kind":
        return mem.map_kind()  # E08 T57: RoE, AB or SoD (game +0x1F698)
    if what == "date":
        return "month {}, week {}, day {}".format(*mem.date())
    if what in ("draws", "turn_draws") or what.startswith(("draws:", "turn_draws:")):
        return draws_text(obs, what)  # E09 T23 (batch TWN-3)
    if what == "kinds":
        # E09 T23 (batch TWN-3): the week's and the month's kinds and creatures as the game holds
        # them (week_kind 0x697750, week_creature 0x6983FC, month_kind 0x698834, month_creature
        # 0x697748; survey E09-growth.md §2, §3: 0 normal, 1 a creature's, 2 the Deity's or the
        # Plague; a normal one's creature its name index).
        week, month = mem.i32(WEEK_KIND), mem.i32(MONTH_KIND)
        week_c, month_c = mem.i32(WEEK_CREATURE), mem.i32(MONTH_CREATURE)
        def named(kind, creature):
            return (f"{kind} ({mem.creature_name(creature, 1)})" if kind == 1 and creature >= 0
                    else f"{kind} (name {creature})" if kind == 0 else str(kind))
        return f"week {named(week, week_c)}, month {named(month, month_c)}"
    if what == "level_ups":
        return str(len(obs.get("level_up_dialogs", [])))
    # E08 T32b1: the n-th level-up dialog of the step (from 1): level, primary, left, right, taken.
    if m := re.match(r"level_up:(\d+):(level|primary|left|right|taken|hero)$", what):
        dialogs = obs.get("level_up_dialogs", [])
        n, part = int(m.group(1)), m.group(2)
        if not 1 <= n <= len(dialogs):
            return "no dialog"
        dialog = dialogs[n - 1]
        if part in ("left", "right"):
            return offer_text(dialog[part])
        return str(dialog[part])
    if what == "messages":
        return ", ".join(obs.get("messages", [])) or "none"
    # E08 T57: the step's level-up dialogs in one line (`level 2: Wisdom or Logistics, took left
    # (Wisdom)`), a player's treasury (`30 wood, …, 30900 gold`) and how many of the game's
    # Universities offer a skill (`offering:Necromancy`: `2 of 30`).
    if what == "level_up_dialogs":
        return "; ".join(f"level {d['level']}: {offer_text(d['left'])} or "
                         f"{offer_text(d['right'])}, took {d['taken']}"
                         for d in obs.get("level_up_dialogs", [])) or "none"
    if what == "treasury_changed":
        # E08 T68b (batch E8b's E8-9): whether the step changed the hero's player's treasury.
        owner = COLOURS[obs["before"]["hero"]["owner"]]
        return yes[obs["before"]["resources"][owner] != after["resources"][owner]]
    if m := re.match(r"treasury(?::([a-z]+))?$", what):
        owner = m.group(1) or COLOURS[obs["before"]["hero"]["owner"]]
        return ", ".join(f"{n} {k}" for k, n in after["resources"][owner].items())
    if m := re.match(r"offering:(.+)$", what):
        found = mem.universities()
        skill = h3mem.SKILLS.index(m.group(1))
        return f"{sum(skill in u for u in found)} of {len(found)}"
    # E08 T57 (batch E7): each message's pictures with their captions (`ADVEVENT:175: 1000
    # experience, Expert Learning`), and the choices made between two pictures (`1000 gold or
    # 525 experience: right`; `choice:left`, `choice:right` what a side showed, `choice:taken`).
    if what == "message_pictures":
        return "; ".join(f"{m}: {p}" for m, p in zip(obs.get("messages", []),
                                                     obs.get("message_pictures", []))
                         if p != "none") or "none"
    if m := re.match(r"choice(?::(left|right|taken))?$", what):
        made = obs.get("choices", [])
        if not made:
            return "none"
        if m.group(1):
            return ", ".join(c[m.group(1)] for c in made)
        return ", ".join(f"{c['left']} or {c['right']}: {c['taken']}" for c in made)
    if m := re.match(r"(word|object):(\d+),(\d+)(?:,(\d))?$", what):
        x, y, z = int(m.group(2)), int(m.group(3)), int(m.group(4) or 0)
        if m.group(1) == "word":
            # E08 T79b: through a hero on the entrance (object_tile).
            tile = object_tile(mem, x, y, z)
            return f"{tile['word']:#010x}" + (f" (under {tile['hero']})" if "hero" in tile else "")
        return object_text(mem, x, y, z)
    if what == "heroes:experience":
        return pool_experience(mem)
    if what == "question":
        return ", ".join(q["kind"] for q in obs.get("questions", [])) or "none"
    if what == "questions":
        return ", ".join(q["record"] or "?" for q in obs.get("questions", [])) or "none"
    if what == "answers":
        # E08 T100b (batch E11b's E11-14): each question's record with the answer given.
        return ", ".join(f"{q['record'] or '?'}: {q['answer']}"
                         for q in obs.get("questions", [])) or "none"
    # E08 T90 (batch E10a): the guards' questions' size words by their ARRAYTXT row.
    if what == "size_words":
        return ", ".join(f"ARRAYTXT:{q['size_word']}" if q["size_word"] is not None else
                         "no ARRAYTXT row from 176 found" for q in obs.get("questions", [])
                         if "size_word" in q) or "none"
    battles = obs.get("battles", [])
    if what == "battle":
        return yes[bool(battles)]
    # E08 T100a (batch E11a's E11-19: which guard a step and a jump met, in order): every battle
    # of the step, the hero's tile as it opened, the defender's stacks and the winner.
    if what == "battles":
        return "; ".join(
            f"{k}. at {','.join(map(str, b['hero_tile']))}: defender "
            f"{stacks_text(mem, b['start'], 1)}, winner "
            f"{'none' if b.get('winner') is None or b['winner'] < 0 else SIDES[b['winner']]}"
            for k, b in enumerate(battles, 1)) or "none"
    # E08 T32b1: the last battle's field (its special field, `none`; or its terrain) and whether
    # a creature's stack that was there at its start was gone at its end (a destroyed machine).
    if what in ("battlefield", "battlefield:terrain"):
        if not battles:
            return "no battle"
        field = battles[-1]["field"]
        return field["terrain"] if what.endswith("terrain") else field["special"] or "none"
    if m := re.match(r"destroyed:(.+)$", what):
        if not battles:
            return "no battle"
        fought = battles[-1]
        names = {c for _, c, _, _ in fought["start"] if mem.creature_name(c) == m.group(1)}
        if not names:
            return "not there"
        alive = any(c in names and n > 0 for _, c, n, _ in fought["end"])
        return yes[not alive]
    if what.split(":")[0] in ("stacks", "hexes", "winner", "retreated", "surrendered"):
        if not battles:
            return "no battle"
        fought = battles[-1]
        kind, *rest = what.split(":")
        if kind in ("winner", "retreated", "surrendered"):
            # A winner of -1 is none (E17 M3 T6c-2: it read `defender`, SIDES[-1]).
            # Not named `value`: a local of that name made every `before:` read above fail
            # (UnboundLocalError; E08 T32b1).
            got = fought[kind]
            return "none" if got is None or got < 0 else SIDES[got]
        moment, side = rest[0], SIDES.index(rest[1]) if len(rest) > 1 else None
        if kind == "stacks":
            return stacks_text(mem, fought[moment], side)
        return ", ".join(str(h) for sd, _, _, h in fought[moment] if side in (None, sd))
    # E08 T47a (batch E6): whether the last battle opened in its tactics phase (battle manager
    # +0x13D68 at its start); an action's right click (what opened); the screen showing and the
    # record of the topmost dialog's text (a title: GENRLTXT 36); the split dialog's counts; the
    # counts a screen shows under an army's slots; the hero screen's five backpack places.
    if what == "tactics":
        return yes[battles[-1]["tactics"]] if battles else "no battle"
    # E08 T57 (batch E7): the last battle's stacks at its opening as table rows (side, slot, hex,
    # creature, count: a record of `rows`), and their morale or luck as set at the start
    # (`morale:start[:<side>]`: `20 Pikemen -3, 10 Centaurs -3`).
    if what == "opening:rows":
        return opening_rows(mem, battles[-1]) if battles else []
    # E08 T79a (batch E9a): a side's stacks at the last battle's opening, as table rows (army
    # slot, creature, count) or in one line (`slot 0: 50 Troglodytes, slot 1: ...`).
    if m := re.match(r"opening:(slots|stacks):(attacker|defender)$", what):
        side = SIDES.index(m.group(2))
        found = [s for s in battles[-1]["opening"] if s["side"] == side] if battles else []
        if m.group(1) == "slots":
            return [[str(s["army_slot"]), mem.creature_name(s["creature"], s["count"]),
                     str(s["count"])] for s in found]
        return ", ".join(f"slot {s['army_slot']}: {s['count']} "
                         f"{mem.creature_name(s['creature'], s['count'])}" for s in found) \
            or ("no battle" if not battles else "none")
    # E09 T3b (batch TWN-1's merge): the last battle's opening stack of a side whose army slot is
    # n (`opening:slot:<n>:<side>`: `4 Swordsmen`, `none`).
    if m := re.match(r"opening:slot:(\d+):(attacker|defender)$", what):
        if not battles:
            return "no battle"
        side, slot = SIDES.index(m.group(2)), int(m.group(1))
        found = [s for s in battles[-1]["opening"] if s["side"] == side and s["army_slot"] == slot]
        return ", ".join(f"{s['count']} {mem.creature_name(s['creature'], s['count'])}"
                         for s in found) or "none"
    if m := re.match(r"(morale|luck):start(?::(attacker|defender))?$", what):
        if not battles:
            return "no battle"
        side = SIDES.index(m.group(2)) if m.group(2) else None
        return ", ".join(f"{s['count']} {mem.creature_name(s['creature'], s['count'])} "
                         f"{s[m.group(1)]}" for s in battles[-1]["opening"]
                         if side in (None, s["side"]) and s["count"] > 0) or "none"
    if what == "shown":
        return obs.get("shown", "none")
    if what == "screen":
        return screen_kind(mem) or "none"
    if what == "dialog":
        return (h3lod.identify(mem.dialog_text()) or "?") if mem.dialog() else "none"
    if what == "split:counts":
        return split_counts(mem)
    # E08 T110 (batch E12a's E12-3): the bars' selected slot after an action (`bar:selected`) and
    # before its clicks (`bar:selected:before`), the creature windows its clicks opened.
    if what in ("bar:selected", "bar:selected:before"):
        got = obs.get("selected_before") if what.endswith("before") else bar_selection(mem)
        return got or "no bars"
    if what == "popup:lines":
        # E08 T110: the lines of an action's right-click window, each by its records.
        return " / ".join(obs.get("popup_lines", [])) or "none"
    if what == "bar:clicks":
        # E08 T110: each click of the action with the slot selected after it (`lower:1: lower 1
        # selected, upper:1: none selected`).
        return ", ".join(f"{n}: {sel} selected" for n, sel in obs.get("clicks", [])) or "none"
    if what == "creature_windows":
        return ", ".join(f"after the click on {n} (closed with its OK)"
                         for n in obs.get("creature_windows", [])) or "none"
    if m := re.match(r"dialog:(items|boxes)(?::(\d+)-(\d+))?$", what):
        # E08 T110: the topmost dialog's items (`id:state:frame`; `boxes` also each box and
        # vtable), those of an id range only: a probe for a new screen's items.
        low, high = (int(m.group(2)), int(m.group(3))) if m.group(2) else (0, 1 << 16)
        return " ".join(f"{i['id']}:{i['state']:#x}:{i['frame']}"
                        + (f":{i['box']}:{i['vtable']:#x}" if m.group(1) == "boxes" else "")
                        for i in mem.dialog_items() if low <= i["id"] <= high) or "none"
    if what == "shown:backpack":
        return shown_backpack(mem)
    if what.startswith("shown:"):
        return shown_counts(mem, what.split(":", 1)[1])
    if what.startswith("hero:"):
        return hero_value(mem, after["hero"], what.split(":", 1)[1])
    if m := re.match(r"hero\[(.+)\]:(.+)$", what):
        found = [h for h in mem.all_heroes() if h["name"] == m.group(1)]
        if len(found) != 1:
            raise ValueError(f"{len(found)} heroes named {m.group(1)!r}")
        return hero_value(mem, found[0], m.group(2))
    # E08 T100a (batch E11a's E11-6, E11-7, E11-18): the tiles of a box, `explored:x0,y0..x1,y1`
    # (and `,level`, `:colour` as below), row by row: `43,29 no, 44,29 no, …`.
    if m := re.match(r"explored:(\d+),(\d+)\.\.(\d+),(\d+)(,\d)?(:[a-z]+)?$", what):
        x0, y0, x1, y1 = map(int, m.groups()[:4])
        rest = (m.group(5) or "") + (m.group(6) or "")
        return ", ".join(f"{x},{y} " + value(obs, f"explored:{x},{y}{rest}", mem)
                         for y in range(y0, y1 + 1) for x in range(x0, x1 + 1))
    # E08 T37a (batch E5's E5-9, E5-10, E5-11): whether a player has explored a tile, its bit in
    # the tile's vision word; the player the step's hero's owner unless named.
    if m := re.match(r"explored:(\d+),(\d+)(?:,(\d))?(?::([a-z]+))?$", what):
        if m.group(4) is not None and m.group(4) not in COLOURS:
            raise ValueError(f"unknown colour `{m.group(4)}`")
        colour = COLOURS.index(m.group(4)) if m.group(4) else after["hero"]["owner"]
        if colour < 0:
            raise ValueError(f"`{what}`: the step's hero is off the map; name the player")
        return yes[mem.explored(int(m.group(1)), int(m.group(2)), int(m.group(3) or 0), colour)]
    # E08 T68a (batch E8a): the boats' records (boat_value), what the view draws on a hero's
    # tile, the last battle's opening (terrain, special field, ship flag, the obstacles' hexes),
    # whether a walk's second click was made (`again_if`) and what an arrow key did (`key`).
    if what in ("boat", "boats:used") or re.match(r"boat(:\d+,\d+(,\d)?|\[.+\])$", what) \
            or re.match(r"boat_record:\d+,\d+(,\d)?$", what):
        hero = after["hero"]
        if m := re.match(r"boat\[(.+)\]$", what):
            hero, what = mem.hero_named(m.group(1)), "boat"
        return boat_value(mem, hero, what)
    if what == "drawn":
        return drawn_text(mem, after["hero"])
    # E08 T110 (batch E12a's E12-4): the last battle's anti-magic flag at its opening (+0x53C4:
    # `set (1)`, `clear (0)`) and a side's spell-book turn (`book:attacker`, `book:defender`:
    # book_text).
    if what == "anti_magic":
        if not battles:
            return "no battle"
        flag = battles[-1]["field"]["anti_magic"]
        return f"{'set' if flag else 'clear'} ({flag})"
    if m := re.match(r"book:(attacker|defender)$", what):
        if not battles:
            return "no battle"
        return book_text(mem, battles[-1].get("books", {}).get(SIDES.index(m.group(1))))
    if what in ("opening:field", "battlefield:ship", "blocked"):
        if not battles:
            return "no battle"
        fought = battles[-1]
        field, blocked = fought["field"], ", ".join(map(str, fought["blocked"])) or "none"
        if what == "battlefield:ship":
            return str(field["ship"])
        if what == "blocked":
            return blocked
        return (f"terrain {field['terrain']}, special field {field['special'] or 'none'}, "
                f"ship flag {field['ship']}, blocked hexes {blocked}")
    if what in ("again", "key"):
        return obs.get(what, "none")
    # E08 T100b (batch E11b): a `repeat` step's presses (presses_value), the Whirlpools' list in
    # its order (`16 tiles: 22,15 23,15 …`), a walk's target (the tile `walk = "off"` chose) and
    # the current player's selected hero (`none`).
    if what.startswith("presses"):
        return presses_value(obs, what, mem)
    if what == "whirlpools":
        found = mem.whirlpools()
        return f"{len(found)} tiles: " + " ".join(tile_name(t) for t in found)
    if what == "target":
        return tile_name(obs["target"]) if "target" in obs else "none"
    if what == "selected":
        chosen = mem.player(mem.current_player())["selected"]
        return mem.hero(chosen)["name"] if chosen >= 0 else "none"
    # E08 T100b: `tile:` over a box (`tile:x0,y0..x1,y1[,level]`, row by row, each `x,y` and its
    # tile_text), on the step's hero's tile before the step (`tile:from`) or after it
    # (`tile:here`), or on another hero's tile now (`tile:hero[<name>]`), each `x,y: <tile_text>`.
    if m := re.match(r"tile:(\d+),(\d+)\.\.(\d+),(\d+)(?:,(\d))?$", what):
        x0, y0, x1, y1 = map(int, m.groups()[:4])
        z = int(m.group(5) or 0)
        return ", ".join(f"{x},{y} {tile_text(mem, x, y, z)}"
                         for y in range(y0, y1 + 1) for x in range(x0, x1 + 1))
    if m := re.match(r"tile:(from|here|hero\[(.+)\])$", what):
        if m.group(1) == "from":
            tile = obs["before"]["hero"]["tile"]
        elif m.group(1) == "here":
            tile = after["hero"]["tile"]
        else:
            tile = mem.hero_named(m.group(2))["tile"]
        return f"{tile_name(tile)}: {tile_text(mem, *tile)}"
    # E08 T100a (batch E11a): the meetings a step opened (the heroes the exchange showed), the
    # step's hero's army slot by slot before and after it as table rows (slot, creature, count
    # before, count after; the slots that held something), a replay's observations.
    if what == "exchange":
        return "; ".join(f"{e['left']} (left, the visitor) and {e['right']} (right)"
                         for e in obs.get("exchanges", [])) or "none"
    if what == "army:rows":
        old, new = obs["before"]["hero"]["slots"], after["hero"]["slots"]
        return [[str(k), mem.creature_name(o[0] if o[0] >= 0 else n[0], 2), str(o[1]), str(n[1])]
                for k, (o, n) in enumerate(zip(old, new)) if o[0] >= 0 or n[0] >= 0]
    if what.startswith("replay") or what.startswith("view:"):
        return replay_value(obs, what)
    if what.startswith("at_opening:"):
        return obs.get("at_opening", {}).get(what.split(":", 1)[1], "no battle (or not read)")
    if what == "battle:tile":
        # E08 T68b: the step's hero's tile as the last battle opened (batch E8b's E8-13, E8-14).
        return ",".join(map(str, battles[-1]["hero_tile"])) if battles else "no battle"
    if m := re.match(r"(monster|aggression|never_flees):(\d+),(\d+)(?:,(\d))?$", what):
        found = mem.monster(int(m.group(2)), int(m.group(3)), int(m.group(4) or 0))
        if found is None:
            return "none"
        if m.group(1) == "monster":
            return army_text(mem, [(found["creature"], found["count"])])
        return str(found["aggression"]) if m.group(1) == "aggression" else yes[found["never_flees"]]
    # E08 T79a (batch E9a): a mine's or Lighthouse's record by its entrance (`mine:x,y[,level]`),
    # a tile's own word (`tile:x,y[,level]`), the cursor over a walk's target before its first
    # click, the screens a step's walk opened, the last battle's sides at its opening (owner,
    # human, human here) and the stops of `battle = "auto again"`.
    if m := re.match(r"(mine|tile):(\d+),(\d+)(?:,(\d))?$", what):
        x, y, z = int(m.group(2)), int(m.group(3)), int(m.group(4) or 0)
        return mine_text(mem, x, y, z) if m.group(1) == "mine" else tile_text(mem, x, y, z)
    # E08 T90 (batches E10a, E10b): a dwelling's record by its entrance (`dwelling:x,y[,level]`),
    # the players' team bytes (`teams`), the map's objects anchored in a rectangle with their
    # class and subclass (`objects:x0,y0:x1,y1[,level]`) and its tiles' blocked and visitable
    # bits (`access:x0,y0:x1,y1[,level]`), a Refugee Camp's words (`camp:x,y[,level]`), the hire
    # dialog now (`hire`, `hire:<most|amount|ally|enabled|counts|items>`) and as the step's
    # settle saw it open and after the actions (`hire:opening[:<part>]`, `hire:after[:<part>]`,
    # the last one; `none` when no hire dialog opened), the status bar's hint and the popup of a
    # right click on a tile the step read (`hint`, `popup`: `text_parts`, `hint:raw` whether
    # the bar and the buffer agree), the battle's defending side's owner
    # (`opening:defender`) and the computer players' turns an end-turn step passed (`ai_turns`).
    if m := re.match(r"dwelling:(\d+),(\d+)(?:,(\d))?(?::(owner|creatures|guards))?$", what):
        return dwelling_text(mem, int(m.group(1)), int(m.group(2)), int(m.group(3) or 0),
                             m.group(4))
    if what == "teams":
        return teams_text(mem)
    # E08 T110 (batch E12b): a Fountain of Fortune's tile word (`fountain:x,y`: the value, bits
    # 13 to 16 signed, and the team bits 5 to 12) and a Magic Spring's entrance (`spring:x,y`:
    # bit 6 full), each the tile's own word (h3mem.tile_word; under a hero standing there).
    if m := re.match(r"(fountain|spring):(\d+),(\d+)(?:,(\d))?$", what):
        return bonus_word_text(mem, m.group(1), int(m.group(2)), int(m.group(3)),
                               int(m.group(4) or 0))
    # E08 T110 (batch E12a): a garrison's record by its entrance (`garrison:x,y[,level][:part]`,
    # garrison_text), live; the bars of each garrison exchange the step's walk opened, as it
    # opened (`at 8,3: upper 0: 2 Pikemen; lower 0: 10 Archangels`).
    if m := re.match(r"garrison:(\d+),(\d+)(?:,(\d))?(?::(owner|stacks|removable|entrance|"
                     r"index))?$", what):
        return garrison_text(mem, int(m.group(1)), int(m.group(2)), int(m.group(3) or 0),
                             m.group(4))
    if what == "garrison_dialogs":
        return "; ".join(f"at {tile_name(d['at'])}: upper "
                         f"{'no record' if d['upper'] is None else army_slots_text(mem, d['upper'])}"
                         f"; lower {army_slots_text(mem, d['lower'])}"
                         for d in obs.get("garrison_dialogs", [])) or "none"
    if m := re.match(r"(objects|access):(\d+),(\d+):(\d+),(\d+)(?:,(\d))?$", what):
        x0, y0, x1, y1 = (int(m.group(k)) for k in range(2, 6))
        return (objects_text if m.group(1) == "objects" else access_text)(
            mem, x0, y0, x1, y1, int(m.group(6) or 0))
    if m := re.match(r"camp:(\d+),(\d+)(?:,(\d))?$", what):
        return camp_text(mem, int(m.group(1)), int(m.group(2)), int(m.group(3) or 0))
    if m := re.match(r"hire(?::(opening|after))?(?::([a-z]+))?$", what):
        which, part = m.group(1), m.group(2)
        if which is None:
            return hire_text(mem.recruit_dialog(), part, mem)
        hires = obs.get("hires", [])
        return hire_text(hires[-1][which], part, mem) if hires else "none"
    if what in ("hint", "popup"):
        if what not in obs:
            return "none"
        found = obs[what]
        return found["parts"] if what == "hint" else (
            f"{found['record']}: {found['parts']}" if found["opened"] else "no window")
    if what == "cast":
        # E08 T110 (batch E12b): a `cast` step's book and spell points (`the book opened; View
        # Earth: the view 0x...; spell points 20 then 18`).
        cast = obs.get("cast")
        if not cast:
            return "none"
        view = f"; the view {cast['view']}" if "view" in cast else ""
        return (f"the book {cast['book']}; {cast['spell']}{view}; spell points {cast['before']} "
                f"then {cast['after']}")
    if what == "hints":
        # E08 T110 (batch E12b): each hint of the step's list, `x,y: <parts>`.
        return "; ".join(f"{tile_name(h['tile'])}: {h['parts']}"
                         for h in obs.get("hints", [])) or "none"
    if what == "hint:raw":
        found = obs.get("hint")
        return "none" if not found else ("same" if found["bar"] == found["buffer"]
                                         else f"bar {found['bar'] is not None}, buffer "
                                              f"{found['buffer'] is not None}, differ")
    if what == "opening:defender":
        if not battles:
            return "no battle"
        side = battles[-1]["sides"][1]
        return f"{colour_name(side['owner'])} (human {side['human']}, here {side['here']})"
    if what == "ai_turns":
        return "; ".join(f"{COLOURS[t['player']]} ({t.get('seconds', 0)} s)"
                         for t in obs.get("ai_turns", [])) or "none"
    if what == "cursor":
        cursor = obs.get("cursor")
        return f"type {cursor[0]}, frame {cursor[1]}" if cursor else "none"
    if what == "screens":
        return ", ".join(obs.get("screens", [])) or "none"
    if what == "opening:sides":
        if not battles:
            return "no battle"
        return "; ".join(f"{SIDES[k]} {colour_name(side['owner'])} (human {side['human']}, "
                         f"here {side['here']})" for k, side in enumerate(battles[-1]["sides"]))
    if what == "stops":
        if not battles:
            return "no battle"
        return "; ".join(f"{stop['side']} {stop['count']} "
                         f"{mem.creature_name(stop['creature'], stop['count'])} ({stop['auto']})"
                         for stop in battles[-1]["stops"]) or "none"
    # E08 T79b (batch E9b): a player's day's income (+0x108: `0 wood, 5 mercury, ...`), its
    # Mystical Gardens' mask (+0xB8: `0x00000001 (Gardens 0)`), a monster's count with its
    # sixteenths (bits 27-30: `47 Pikemen and 4/16 (756 sixteenths), grows`) and what the step
    # changed in its hero's player's treasury (`+4 sulfur, +500 gold`; `none`). The player is
    # the step hero's owner or, in an end-turn step, the new current player, unless named.
    if m := re.match(r"(income|gardens)(?::([a-z]+))?$", what):
        colour = player_colour(obs, m.group(2), mem)
        found = mem.player(colour)
        if m.group(1) == "income":
            return ", ".join(f"{n} {k}" for k, n in found["income"].items())
        mask = found["gardens"]
        bits = ", ".join(str(b) for b in range(32) if mask >> b & 1)
        return f"{mask:#010x} ({f'Gardens {bits}' if bits else 'none'})"
    if m := re.match(r"growth:(\d+),(\d+)(?:,(\d))?$", what):
        found = mem.monster(int(m.group(1)), int(m.group(2)), int(m.group(3) or 0))
        if found is None:
            return "none"
        count, part = found["count"], found["sixteenths"]
        return (f"{count} {mem.creature_name(found['creature'], count)} and {part}/16 "
                f"({16 * count + part} sixteenths), "
                + ("does not grow" if found["fixed_count"] else "grows"))
    if m := re.match(r"gain(?::([a-z]+))?$", what):
        owner = COLOURS[player_colour(obs, m.group(1), mem)]
        was, now = obs["before"]["resources"][owner], after["resources"][owner]
        return ", ".join(f"{now[k] - was[k]:+d} {k}" for k in now if now[k] != was[k]) or "none"
    # E09 T3b (batch TWN-1): a town's record by its entrance (`town:x,y[,level]:<part>`: owner,
    # visitor, built, every, garrison, slot:<n>; town_text), read live (`before:` right before
    # the step); over every town the every-built sets that differ from the built ones and the
    # owned towns' garrisons (towns_text).
    if m := re.match(r"town:(\d+),(\d+)(?:,(\d))?:(owner|visitor|built|every|mark|garrison|"
                     r"slot:[0-6]|counts|count:\d+|portal|garrison_hero|dwellings)$", what):
        return town_text(mem, int(m.group(1)), int(m.group(2)), int(m.group(3) or 0), m.group(4))
    if what in ("towns:every_vs_built", "towns:owned_garrisons"):
        return towns_text(mem, what.split(":")[1])
    # E09 T14b (batch TWN-2): the hall's slots and the building's dialog, live (hall_text); the
    # screen an action left (`screen`: town, hall, build, …).
    if re.match(r"hall:slot:\d+$", what) or what in ("build:button", "build:text"):
        return hall_text(obs, what, mem)
    if what == "screen":
        return obs.get("screen") or "none"
    if what == "town_screen_closed":
        return obs.get("town_screen_closed", "none")  # E09 T14b: click, key or open
    # E09 T3b (batch TWN-1's TWN-1-3b, TWN-1-4): whether the town screen opened during the step
    # (settle), and the unknown screens' vtables the step met.
    if what == "town_screen":
        return "yes" if obs.get("town_screen") else "no"
    if what == "unknown_screens":
        return ", ".join(obs.get("unknown_screens", [])) or "none"
    raise ValueError(f"unknown observation `{what}`")


def absolute_day(mem):
    """The game's day from 1 (E08 T110): (month - 1) x 28 + (week - 1) x 7 + the day."""
    month, week, day = mem.date()
    return (month - 1) * 28 + (week - 1) * 7 + day


def player_colour(obs, named, mem):
    """The colour index an observation names (`red`, …), else the step's hero's owner, else (an
    end-turn step) the current player (E08 T79b)."""
    if named is not None:
        if named not in COLOURS:
            raise ValueError(f"unknown colour `{named}`")
        return COLOURS.index(named)
    hero = (obs.get("before") or {}).get("hero")
    return hero["owner"] if hero else mem.current_player()


def oddities(obs, expect):
    notes = []
    if "marks" in expect and obs.get("marks") != expect["marks"]:
        notes.append(f"the path to {obs['target']} had {obs['marks']} marks, not {expect['marks']}")
    if "path" in expect and obs.get("path") != [expect["path"]]:
        notes.append(f"the path to {obs['target']} was {'/'.join(obs['path']) or 'not shown'}, "
                     f"not all {expect['path']}")
    if obs.get("dialogs") and not expect.get("dialogs"):
        notes.append(f"{obs['dialogs']} dialog(s) after the walk to {obs['target']}")
    if obs.get("days_waited"):
        notes.append(f"the walk to {obs['target']} waited {obs['days_waited']} day(s) before its "
                     "step: beyond the day's points")
    if obs.get("first_click_walked"):
        notes.append(f"the first click on {obs['target']} walked (a path to it was set before)")
    if "question" in expect:
        asked = ", ".join(q["kind"] for q in obs.get("questions", [])) or "none"
        if asked != expect["question"]:
            notes.append(f"the questions were {asked}, not {expect['question']}")
    if "battle" in expect and bool(obs.get("battles")) != expect["battle"]:
        notes.append(f"{'a' if obs.get('battles') else 'no'} battle")
    return notes


TABLE_RULE = re.compile(r"\|(\s*-+\s*\|)+")


def table_cells(line):
    """A table row's cells, as written between its bars (None if the line is no row)."""
    line = line.strip()
    return line[1:-1].split("|") if len(line) > 1 and line[0] == line[-1] == "|" else None


def repeated(seen, label):
    """A table row's label as its cells' keys name it (E08 T79b): the label itself the first time
    the table has it, `<label>#<n>` the n-th time (`seen` counts them per table)."""
    seen[label] = seen.get(label, 0) + 1
    return label if seen[label] == 1 else f"{label}#{seen[label]}"


def cell_key(section, label, columns, j):
    """The record key of a row's j-th cell: its label in a table of one value column, `<label> |
    <column>` in a wider one (E08 T37a: batch E5's probes, walks and heroes by column)."""
    return (section, label if len(columns) <= 2 else f"{label} | {columns[j].strip()}")


def fill(template, values, notes, header):
    """The template with each section's fields and table rows filled in: an empty cell by its
    row's label, in a table of more than one value column by `<label> | <column>` (E08 T37a).
    Since E08 T57 a section's `rows` (a list of rows, opening:rows) fill its table's rows without
    a label in order, with more rows added after the last when they are not enough; since E08
    T68b a section's `rows:<first column>` fill the table whose first column is named so (a
    section with several tables without labels: batch E8b's E8-8); since E08 T79a the n-th such
    table of a section (n from 2) by `rows:<first column>#<n>` (batch E9a's E9-1: two tables of
    slots); since E08 T79b a row whose label an earlier row of its table has by `<label>#<n>`
    (n from 2: batch E9b's two rows a day, `1 | wood` and `1#2 | wood`)."""
    lines, section, out, columns, rows = template.splitlines(), None, [], [], []
    tables, seen = {}, {}
    for line in lines:
        m = re.match(r"## (\S+) ", line)
        if m:
            section, columns, tables = m.group(1), [], {}
            rows = list(values.pop((section, "rows"), []))
        cells = table_cells(line)
        if rows and cells is None and columns and out and table_cells(out[-1]) is not None:
            out += ["| " + " | ".join(r) + " |" for r in rows]  # the table ended first
            rows = []
        if rows and cells is not None and len(cells) == len(columns) > 1 \
                and not any(c.strip() for c in cells):
            out.append("| " + " | ".join(rows.pop(0)) + " |")
            continue
        if section is not None:
            m = re.match(r"- (.+): ?$", line)
            if m and m.group(1) == "anything odd" and (section, m.group(1)) in values \
                    and notes.get(section):
                # E08 T110: a recipe's own `+anything odd` lines and the notes, both kept.
                line = (f"- anything odd: {values.pop((section, m.group(1)))}; "
                        f"{'; '.join(notes.pop(section))}")
            elif m and (section, m.group(1)) in values:
                line = f"- {m.group(1)}: {values.pop((section, m.group(1)))}"
            elif m and m.group(1) == "anything odd" and notes.get(section):
                line = f"- anything odd: {'; '.join(notes.pop(section))}"
            cells = table_cells(line)
            if cells is not None and TABLE_RULE.fullmatch(line.strip()) and out:
                columns, seen = table_cells(out[-1]) or [], {}
                first = columns[0].strip() if columns else None
                tables[first] = tables.get(first, 0) + 1
                key = (section, f"rows:{first}" + (f"#{tables[first]}" if tables[first] > 1
                                                   else ""))
                if columns and key in values:
                    rows = list(values.pop(key))
            elif cells is not None and len(cells) == len(columns) > 1:
                label = repeated(seen, cells[0].strip())
                for j in range(1, len(cells)):
                    key = cell_key(section, label, columns, j)
                    if not cells[j].strip() and key in values:
                        cells[j] = f" {values.pop(key)} "
                line = "|" + "|".join(cells) + "|"
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


def each_tiles(spec):
    """The tiles of a step's `each = { from = [x, y], to = [x, y], step = [dx, dy] }` (E08 T57),
    by row then column."""
    (x0, y0), (x1, y1) = spec["from"], spec["to"]
    dx, dy = spec.get("step", [1, 1])
    return [(x, y) for y in range(y0, y1 + 1, dy) for x in range(x0, x1 + 1, dx)]


def nearest_first(start, tiles):
    """The tiles in the order a hero at `start` visits them nearest first (by the larger of the
    two distances, ties by row then column)."""
    left, order, at = list(tiles), [], start
    while left:
        nxt = min(left, key=lambda t: (max(abs(t[0] - at[0]), abs(t[1] - at[1])), t[1], t[0]))
        left.remove(nxt)
        order.append(nxt)
        at = nxt
    return order


def substitute(item, x, y):
    """`{x}` and `{y}` replaced in a recipe's strings, lists and tables."""
    if isinstance(item, str):
        return item.replace("{x}", str(x)).replace("{y}", str(y))
    if isinstance(item, list):
        return [substitute(i, x, y) for i in item]
    if isinstance(item, dict):
        return {substitute(k, x, y): substitute(v, x, y) for k, v in item.items()}
    return item


def chosen_tiles(chosen, tiles, mem, start):
    """The tiles a step's `select` keeps (E08 T57; T68b), in the order the hero visits them,
    nearest first (nearest_first): those whose object holds an artifact by its word (`holds =
    "artifact"`, held_artifact) or whose object_text starts with `is` (or any object), the `count`
    nearest of them (default all); then the `controls` nearest whose object holds none; then for
    each text of `then` the nearest whose object_text starts with it (batch E8b's E8-6: every
    artifact chest, then one of kind 1 and one of kind 0)."""
    unknown = set(chosen) - {"holds", "is", "count", "controls", "then"}
    if unknown or chosen.get("holds", "artifact") != "artifact" or (
            "holds" in chosen and "is" in chosen):
        raise ValueError(f"`select = {chosen}`: holds = \"artifact\" or is = \"<object text>\", "
                         "count, controls, then")
    texts = {t: object_text(mem, *t) for t in tiles}
    if "holds" in chosen:
        wanted = [t for t in tiles if held_artifact(mem.object_at(*t)) is not None]
    else:
        wanted = [t for t in tiles if texts[t] != "none"
                  and texts[t].startswith(chosen.get("is", ""))]
    picked = nearest_first(start, wanted)[:chosen.get("count", len(wanted))]
    if chosen.get("controls"):
        others = [t for t in tiles if t not in wanted and texts[t] != "none"]
        picked += nearest_first(picked[-1] if picked else start, others)[:chosen["controls"]]
    for text in chosen.get("then", []):
        left = [t for t in tiles if t not in picked and texts[t].startswith(text)]
        if left:
            picked += nearest_first(picked[-1] if picked else start, left)[:1]
    return picked


def expand(step, mem):
    """A step over a grid of tiles (E08 T57, batch E7): `each` names the tiles; a step without a
    walk reads its `record` at each (one step), one with `walk = "each"` walks to each in turn.
    `select` keeps some of the tiles, in the order the hero visits them (chosen_tiles), read when
    the step comes. A record's `row:<table>` field (E08 T68b) reads one table row per tile; a
    step without a walk reads its `note` at each tile too (E08 T68b: E8-6's chests at the
    start)."""
    if "each" not in step:
        return [step]
    tiles = each_tiles(step["each"])
    rest = {k: v for k, v in step.items() if k not in ("each", "select")}
    if "walk" not in step:
        merged = dict(rest)
        merged["record"] = {}
        for x, y in tiles:
            for k, v in substitute(step.get("record", {}), x, y).items():
                if k.startswith("row:"):
                    merged["record"].setdefault(k, {"rows": []})["rows"].append(v)
                else:
                    merged["record"][k] = v
        merged["note"] = {k: v for x, y in tiles
                          for k, v in substitute(step.get("note", {}), x, y).items()}
        return [merged]
    if step["walk"] != "each":
        raise ValueError("a step with `each` walks to each tile (`walk = \"each\"`) or none")
    if "select" in step:
        start = tuple(mem.hero_named(step["hero"])["tile"][:2])
        tiles = chosen_tiles(step["select"], tiles, mem, start)
    return [{**substitute(rest, x, y), "walk": [x, y]} for x, y in tiles]


def unmet(recipe, mem):
    """The recipe's `[[require]]` entries the game does not meet (E08 T57, the spec's restart
    rule): each `{ each = {...}, holds = "artifact" }` needs one of its tiles' objects to hold
    an artifact by its word."""
    missing = []
    for need in recipe.get("require", []):
        if need.get("holds") != "artifact":
            raise ValueError(f"`require = {need}`: only holds = \"artifact\"")
        if not any(held_artifact(mem.object_at(*t)) is not None for t in each_tiles(need["each"])):
            missing.append(need.get("what", str(need["each"])))
    return missing


RESTARTS = 20  # new games at most until the `require` entries are met


def start_game(game, map_path, recipe, restarts):
    """Starts the map in hot seat as the recipe says (its `players`, `difficulty`, `gold`; the
    difficulty checked in memory) and returns the game's memory (a fresh h3mem.Mem); since E08
    T57 again, up to RESTARTS times in all, until its draws meet the recipe's `require` (each
    new game that misses added to `restarts`). The batch's first start and, since E09 T3b, each
    `new_game` step."""
    difficulty = recipe.get("difficulty", "normal")
    while True:
        game.start(map_path)
        # E08 T79b: `gold = true` gives each player the starting bonus Gold (E17 M5 T8a).
        to_hot_seat(game, recipe["players"], difficulty, gold=recipe.get("gold", False))
        mem = h3mem.Mem()
        if mem.difficulty() != difficulty:
            raise RuntimeError(f"the game's difficulty is {mem.difficulty()}, not {difficulty}")
        # E08 T57: a new game whose draws miss what the recipe requires is started again.
        missing = unmet(recipe, mem)
        if not missing:
            return mem
        restarts.append(missing)
        if len(restarts) >= RESTARTS:
            raise RuntimeError(f"{RESTARTS} new games, none met the recipe's `require`")
        game.stop()


def force_draws(game, rolls=None, roll=None):
    """E09 T23 (batch TWN-3): the adventure map's draws forced through xdd.dll (h3drive.ROLLS; the
    runbook's "Forced rolls"): `rolls = { <name> = <value> | "natural" }` makes every later draw
    of each return the value (or be drawn again), `roll = { <name> = <value> | [<a>, <b>, ...] }`
    its next draw (or draws, in order). A command xdd.dll refuses stops the batch."""
    for name, value in (rolls or {}).items():
        if name not in h3drive.ROLLS:
            raise ValueError(f"`rolls`: {name}: the rolls are {', '.join(h3drive.ROLLS)}")
        before = (game.ack() or (0,) * h3drive.ACK_FIELDS)[6]
        game.clock(f"rolls {name} {value}")
        if game.ack(need=True)[6] > before:
            raise RuntimeError(f"xdd.dll refused `rolls {name} {value}`")
    for name, value in (roll or {}).items():
        if name not in h3drive.ROLLS:
            raise ValueError(f"`roll`: {name}: the rolls are {', '.join(h3drive.ROLLS)}")
        game.roll(name, value=value)


def logged_draws(game):
    """E09 T23 (batch TWN-3): the draws xdd.dll logged since the game started (its file `draws`,
    `name value` per line; h3drive.LOGGED), as (name, value) pairs in order."""
    path = Path(game.clock_dir) / "draws"
    if not path.exists():
        return []
    return [(line.split()[0], int(line.split()[1])) for line in path.read_text().splitlines()
            if len(line.split()) == 2]


# E09 T23: the draws logged during the batch's last end-turn step (`turn_draws`).
TURN_DRAWS = []


def draws_text(obs, what):
    """E09 T23 (batch TWN-3): the draws logged during the step (`draws`: `garrison_chance_fort 37,
    garrison_sum_1 0, …`, `none`), or those whose name starts with a prefix (`draws:portal`);
    `turn_draws[:<prefix>]` those of the batch's last end-turn step (a later step's record)."""
    prefix = what.split(":", 1)[1] if ":" in what else ""
    drawn = TURN_DRAWS if what.startswith("turn_draws") else obs.get("draws", [])
    return ", ".join(f"{n} {v}" for n, v in drawn if n.startswith(prefix)) or "none"


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
    beside = map_path.with_name(f"{name}-template.md")
    if template is None and beside.exists() and f"**Stamp `{want}`.**" in beside.read_text():
        # E09 T3b: the template h3-experiment wrote beside the given map, of the same stamp, is
        # the one filled (no build: the engine's workspace may be busy or absent).
        template = beside
    if template is None:
        template = out / f"{name}-template.md"
        compile_spec(spec, out, name)  # the template; the given map stays the one played
    values, notes, steps = {}, {}, []
    OWN_TEXTS[:] = own_texts(spec)  # E08 T79b: the map's own messages by their text
    PLACES.clear()
    PLACES.update(recipe.get("places", {}))  # E08 T100b: the recipe's named places
    log = open(out / "steps.jsonl", "w")
    game = Game(out)
    try:
        restarts = []
        mem = start_game(game, map_path, recipe, restarts)
        force_draws(game, recipe.get("rolls"))  # E09 T23: the recipe's, at each new game
        # The generator reads `mem` at each step's expansion (a closure over this function's
        # variable, not its value at the start): after a `new_game` step the next steps expand
        # with the new game's memory (E09 T3b).
        for step in (s for step in recipe["step"] for s in expand(step, mem)):
            # E09 T23 (batch TWN-3): a step's forced draws (`rolls`, `roll`) and the date it
            # writes (`date = [month, week, day]`: game +0x1F63E) before it plays, and the draws
            # xdd.dll logs while it plays (`draws`); a new game's after it started.
            if not step.get("new_game"):
                force_draws(game, step.get("rolls"), step.get("roll"))
                if "date" in step:
                    mem.write_date(*step["date"])
            drawn = len(logged_draws(game))
            if step.get("new_game"):
                # E09 T3b (batch TWN-1's games 2 and 3): the game stopped and the map started
                # anew as at the batch's start (a new game, not a load: its draws made again);
                # `record` and `note` as an end-turn step's.
                game.stop()
                mem.file.close()
                mem = start_game(game, map_path, recipe, restarts)
                force_draws(game, recipe.get("rolls"))
                force_draws(game, step.get("rolls"), step.get("roll"))
                if "date" in step:
                    mem.write_date(*step["date"])
                drawn = 0
                obs = {"new_game": True, "player": mem.current_player()}
                obs["after"] = {"resources": {COLOURS[c]: mem.player(c)["resources"]
                                              for c in range(8) if mem.player(c)["heroes"]}}
            elif step.get("end_turn"):
                # E08 T79b: the new turn's messages (`messages`, `message_pictures`) and every
                # player's resources after it, for the step's `record` and `note`.
                obs = {"end_turn": True, "messages": [], "message_pictures": []}
                obs["player"] = end_turn(game, mem, obs)
                # E08 T110 (batch E12b): `to_day = <n>` ends turns until the recipe's first
                # player's turn of the game's day n (from 1; month, week and day: 28 a month);
                # the messages of every turn passed kept.
                first = COLOURS.index(recipe["players"][0].lower())
                while step.get("to_day") and not (
                        obs["player"] == first and absolute_day(mem) == int(step["to_day"])):
                    if absolute_day(mem) > int(step["to_day"]):
                        raise RuntimeError(f"`to_day = {step['to_day']}`: day "
                                           f"{absolute_day(mem)} passed it")
                    obs["player"] = end_turn(game, mem, obs)
                obs["after"] = {"resources": {COLOURS[c]: mem.player(c)["resources"]
                                              for c in range(8) if mem.player(c)["heroes"]}}
            elif step.get("look"):
                # E08 T100b (batch E11b's E11-16: a turn's start, before a hero is selected): the
                # view centred on a hero's tile by the minimap (nothing selected, nothing else
                # clicked), for the step's live reads and its `dump`; `record` and `note` as an
                # end-turn step's.
                if "hero" in step:
                    raise ValueError("a `look` step has no `hero`: it selects nobody")
                # Its `before:` reads are read before the view moves (a scroll redraws the view:
                # whether it changes a tile's record is what they and the reads after show).
                pre = pre_read(step, {}, mem)
                centre(game, mem, *mem.hero_named(step["look"])["tile"])
                obs = {"look": step["look"], "messages": [], "message_pictures": [], "pre": pre}
                obs["after"] = {"resources": {COLOURS[c]: mem.player(c)["resources"]
                                              for c in range(8) if mem.player(c)["heroes"]}}
            else:
                prepared = None
                if "walk" in step and (step.get("days") or step.get("near")):
                    prepared = ready(game, mem, step["hero"], step["walk"], step.get("days", 0),
                                     step.get("near", False))
                index = select(game, mem, step["hero"])["index"]
                before = snapshot(mem, index)
                pre = pre_read(step, before, mem)
                acts = ("walk", "equip", "screen", "key", "replay", "cast")
                if sum(k in step for k in acts) > 1 or (
                        ("hint" in step or "popup" in step) and any(k in step for k in acts)):
                    raise ValueError("a step walks, equips, opens a screen, presses a key "
                                     "(E08 T37a, T47a, T68a), reads a hint or popup (E08 T90) "
                                     "or replays the opponents' turn (E08 T100a)")
                # E08 T68a (batch E8a's E8-3): `only_if = "aboard"` plays the step only while its
                # hero is in a boat (status bit 18); else its fields read `not played`.
                # E08 T100a (batch E11a): `only_if = { at = [x, y] or [x, y, level] }` plays it
                # only while its hero stands there (a row's branch after a jump).
                # E08 T100b (batch E11b's E11-16: W1's tile taken or not): with `at`, `empty =
                # [x, y]` plays it only while no hero stands on that tile (its class not 34),
                # `occupied = [x, y]` only while one does.
                skip, only = None, step.get("only_if")
                if only not in (None, "aboard") and not (
                        isinstance(only, dict) and "at" in only
                        and set(only) <= {"at", "empty", "occupied"}):
                    raise ValueError(f"`only_if = {only!r}`: \"aboard\" or {{ at = [x, y] }}, "
                                     "with empty = [x, y] or occupied = [x, y]")
                if only == "aboard" and not before["hero"]["status"] & h3mem.IN_BOAT:
                    skip = f"not played: {step['hero']} is not in a boat"
                if isinstance(only, dict):
                    at = [*only["at"], 0][:3]
                    if list(before["hero"]["tile"]) != at:
                        skip = (f"not played: {step['hero']} stands on "
                                f"{','.join(map(str, before['hero']['tile']))}, not "
                                f"{','.join(map(str, at))}")
                    for want in ("empty", "occupied"):
                        if want in only and not skip:
                            tile = [*only[want], 0][:3]
                            taken = mem.tile(*tile)["class"] == h3mem.HERO_OBJECT
                            if taken == (want == "empty"):
                                skip = (f"not played: a hero stands on {tile_name(tile)}"
                                        if taken else f"not played: no hero on {tile_name(tile)}")
                if "repeat" in step and "key" not in step:
                    raise ValueError("`repeat` presses a step's `key` again (E08 T100b)")
                if "actions" in step and "walk" not in step and "screen" not in step:
                    raise ValueError("`actions` play in a screen a walk or `screen` opens")
                if skip:
                    obs = {"hero": step["hero"], "skipped": skip}
                elif "walk" in step:
                    # E08 T100b (batch E11b's E11-16): `walk = "off"` steps onto the hero's first
                    # open neighbour (off_tile), chosen as the step starts.
                    target = off_tile(mem, step["hero"]) if step["walk"] == "off" else step["walk"]
                    obs = walk(game, mem, step["hero"], target, step.get("answer"),
                               step.get("battle"), step.get("level_up", "left"),
                               step.get("actions"), step.get("choice"), step.get("once", False),
                               step.get("again_if"), step.get("at_opening"))
                    if prepared:
                        # E08 T57: the route set by `ready` makes the first click walk.
                        obs["first_click_walked"] &= not prepared.pop("clicked")
                        obs.update({k: v for k, v in prepared.items() if v})
                elif "hint" in step or "popup" in step:
                    # E08 T90 (batch E10b's E10-10): the mouse on a tile (`hint = [x, y]`: the
                    # status bar's text) and a right click on a tile (`popup = [x, y]`: the
                    # window it shows), in that order; the observations `hint` and `popup`.
                    obs = {"hero": step["hero"]}
                    if "hint" in step and isinstance(step["hint"][0], list):
                        # E08 T110 (batch E12b): several tiles' hints in order (`hints`).
                        obs["hints"] = [{**hint_at(game, mem, *tile), "tile": tile}
                                        for tile in step["hint"]]
                        obs["hint"] = obs["hints"][0]
                    elif "hint" in step:
                        obs["hint"] = hint_at(game, mem, *step["hint"])
                        obs["hints"] = [{**obs["hint"], "tile": step["hint"]}]
                    if "popup" in step:
                        obs["popup"] = popup_at(game, mem, *step["popup"])
                elif "key" in step and "repeat" in step:
                    # E08 T100b (batch E11b): the key again and again, a row each (presses).
                    obs = presses(game, mem, step["hero"], step["key"], step["repeat"],
                                  step.get("answer"), step.get("battle"),
                                  step.get("level_up", "left"))
                elif "key" in step:
                    obs = press(game, mem, step["hero"], step["key"], step.get("answer"),
                                step.get("battle"), step.get("level_up", "left"))
                elif "replay" in step:
                    obs = replay(game, mem, step["hero"], step["replay"])
                elif "cast" in step:
                    obs = cast_on_map(game, mem, step["hero"], step["cast"])  # E08 T110
                elif "equip" in step:
                    obs = equip(game, mem, step["hero"], step["equip"])
                elif "screen" in step:
                    if step["screen"] != "hero":
                        raise ValueError(f"`screen = {step['screen']!r}`: only \"hero\"")
                    obs = hero_screen(game, mem, step["hero"], step.get("actions", []))
                else:
                    obs = {"hero": step["hero"]}
                obs.update(before=before, after=snapshot(mem, index), pre=pre)
            obs["encounter"] = step.get("encounter")
            obs["draws"] = logged_draws(game)[drawn:]
            if step.get("end_turn"):
                TURN_DRAWS[:] = obs["draws"]
            if step.get("dump"):
                # E08 T68a: the screen after the step (`dump = "<name>"`: <out>/<name>.xwd), for an
                # agent to look at (batch E8a-water's E8-5: what the view draws under the hero).
                game.dump(out / f"{step['dump']}.xwd")
                obs["dump"] = str(out / f"{step['dump']}.xwd")
            # E08 T47a: an action's records and notes were read right after it; the step's
            # after the step.
            obs["record"] = {}
            for done in obs.get("actions", []):
                for field, got in done["record"].items():
                    # E08 T110: the actions' rows of one table all kept, in order, and a `+`
                    # field's values joined.
                    if field.startswith("row:") and field in obs["record"]:
                        obs["record"][field] = obs["record"][field] + got
                    elif field.startswith("+") and field in obs["record"]:
                        obs["record"][field] = f"{obs['record'][field]}; {got}"
                    else:
                        obs["record"][field] = got
                notes.setdefault(step["encounter"], []).extend(done["notes"])
            # E08 T110 (batch E12a's E12-1): a step `skip_quietly = true` that is not played
            # records nothing (a branch of a row that rarely plays).
            quiet = obs.get("skipped") and step.get("skip_quietly")
            obs["record"].update({} if quiet else {
                field: ([[obs["skipped"]]] if field.startswith("row:") else obs["skipped"])
                if obs.get("skipped") else record_value(obs, field, what, mem)
                for field, what in step.get("record", {}).items()})
            steps.append(obs)
            log.write(json.dumps(obs) + "\n")
            log.flush()
            print(json.dumps(obs), flush=True)
            for field, got in obs["record"].items():
                if obs.get("skipped") and not field.startswith(("row:", "+")) \
                        and (step["encounter"], field) in values:
                    # E08 T100a (batch E11a's E11-8: two branches of a row, one played): a skipped
                    # step leaves a field another step filled.
                    continue
                if field.startswith("row:"):
                    # E08 T68b: rows added to the table whose first column is named so.
                    values.setdefault((step["encounter"], "rows:" + field[4:]), []).extend(got)
                    continue
                if field.startswith("+"):
                    # E08 T57: a field several steps add to, `; ` between (`none` left out).
                    key = (step["encounter"], field[1:])
                    if got != "none" or key not in values:
                        values[key] = got if values.get(key, "none") == "none" \
                            else f"{values[key]}; {got}"
                    continue
                values[(step["encounter"], field)] = got
            for note in oddities(obs, step.get("expect", {})):
                notes.setdefault(step["encounter"], []).append(note)
            # E08 T32b2: `note` reads like `record`, under "anything odd" unless it reads `none`
            # (batch E4: a worn artifact, the starting bonus's trap).
            for label, what in ({} if quiet else step.get("note", {})).items():
                if (got := value(obs, what, mem)) != "none":
                    notes.setdefault(step["encounter"], []).append(f"{label}: {got}")
    finally:
        game.stop()
        log.close()
    header = (f"> Played by the capture harness on {date.today().isoformat()} "
              f"(`clients/capture/h3adv.py batch {shown(recipe_path)}`): "
              "the same Complete 4.0 `Heroes3.exe` under Wine, without the HD mod; hot seat, "
              f"difficulty {recipe.get('difficulty', 'normal').capitalize()}, the starting bonus "
              f"{'gold' if recipe.get('gold') else 'random'}, Show Route on; read from the game's "
              "memory."
              + (f" Started again {len(restarts)} time(s) for the recipe's `require` "
                 f"({'; '.join(', '.join(m) for m in restarts)} missing)." if restarts else "")
              # E09 T3b: the new games the recipe's steps started (batch TWN-1's games 2, 3).
              + (f" The map started anew {games} more time(s) by the recipe's `new_game` steps "
                 "(new games, no save loaded)."
                 if (games := sum(1 for s in steps if s.get("new_game"))) else ""))
    filled = fill(Path(template).read_text(), values, notes, header)
    result = out / f"{name}.md"
    result.write_text(filled)
    print(f"batch: {len(steps)} steps; {result}")
    return result


def records(text):
    """A filled template's record lines: {(section, field or row label): value}, a cell of a
    table of more than one value column under `<label> | <column>` (E08 T37a), a repeated label
    as fill keys it (`<label>#<n>`, E08 T79b)."""
    found, section, recording, columns, seen = {}, None, False, [], {}
    lines = text.splitlines()
    for i, line in enumerate(lines):
        m = re.match(r"## (\S+) ", line)
        cells = table_cells(line)
        if m:
            section, recording, columns = m.group(1), False, []
        elif line.strip() == "**Record**":
            recording = True
        elif recording and (m := re.match(r"- (.+?): ?(.*)$", line)):
            found[(section, m.group(1))] = m.group(2).strip()
        elif recording and cells is not None:
            if i + 1 < len(lines) and TABLE_RULE.fullmatch(lines[i + 1].strip()):
                columns, seen = cells, {}  # the header
            elif not TABLE_RULE.fullmatch(line.strip()) and len(cells) == len(columns) > 1:
                label = repeated(seen, cells[0].strip())  # E08 T79b: as fill keys it
                for j in range(1, len(cells)):
                    found[cell_key(section, label, columns, j)] = cells[j].strip()
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
