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
object:x,y (the README's "Boats and the sea").

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
    bad = game.out / "not-dismissed.xwd"
    game.dump(bad)  # E08 T57: what showed, for the record
    raise TimeoutError(f"dialogs keep coming or the adventure map does not show (the display: "
                       f"{bad})")


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
                game.dump(game.out / "auto-stalled.xwd")
                raise TimeoutError("auto combat does not start")
            time.sleep(0.2)
        else:
            return


# The war machines (145 Catapult to 149 Arrow Tower): they act by themselves (no Artillery, no
# Ballistics, no First Aid: the computer aims them), so the plans never give them orders.
MACHINES = range(145, 150)
# Where an attack on a hex is clicked: right of its centre (h3drive.hex_point), so the attacker
# comes from the right (E4-9: the Ballista stands on the attacker's first two columns).
ATTACK_FROM_RIGHT = 14
TURN_SETTLE = 0.5  # seconds between a stack's turn showing in memory and its order


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
    raise RuntimeError(f"no attack cursor over hexes {hexes} (cursor frames {seen})")


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
    other stack defends, then `then` plays the rest; the war machines act by themselves. The
    record holds the battlefield at the start (`field`: h3mem.battlefield)."""
    if plan is None:
        raise RuntimeError("a battle the step does not plan (`battle`)")
    if isinstance(plan, dict):
        if set(plan) != {"attack", "by", "then"} or plan["by"] not in SIDES \
                or plan["then"] not in ("auto", "attack"):
            raise ValueError(f"unknown battle plan `{plan}`")
        action, side = "first", SIDES.index(plan["by"])
    else:
        words = plan.split()
        action, side = words[-1], SIDES.index(words[0]) if len(words) == 2 else 0
        if action not in ("auto", "attack", *BATTLE_KEYS) or len(words) > 2:
            raise ValueError(f"unknown battle plan `{plan}`")
    game.move(*BATTLE_PARK)
    wait_for(lambda: mem.stacks(), "the battle's stacks", 10)
    time.sleep(0.5)
    record = {"plan": plan, "field": mem.battlefield(), "start": stacks(mem),
              # E08 T68a: the hexes an obstacle blocks at the opening (batch E8's openings).
              "blocked": mem.obstacle_hexes(),
              # E08 T57: every stack's record at the opening (army slot, morale, luck, ...).
              "opening": [{k: st[k] for k in ("side", "army_slot", "hex", "creature", "count",
                                                "morale", "luck")} for st in mem.stacks()]}
    end = time.monotonic() + limit
    # E08 T47a (batch E6's E6-6): a tactics phase (battle manager +0x13D68 at the start) is ended
    # at once, as the templates say: its message's OK, then its Start button.
    record["tactics"] = mem.tactics_phase()
    if record["tactics"]:
        end_tactics_phase(game, mem, end)
    if action == "first":
        attack_until_gone(game, mem, side, plan["attack"], end)
        if find_button(game, "battle_ok"):
            pass
        elif plan["then"] == "auto":
            auto_combat(game, mem, end)
        else:
            attack_to_the_end(game, mem, end)
    elif action == "attack":
        attack_to_the_end(game, mem, end)
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
           choice=None):
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
    keeps its pictures with their captions (`message_pictures`, parallel to `messages`)."""
    events = {"dialogs": 0, "questions": [], "battles": [], "level_ups": [],
              "level_up_dialogs": [], "messages": [], "screens": [], "actions": [],
              "choices": [], "message_pictures": []}
    choices = [level_up] if isinstance(level_up, str) else list(level_up)
    picks = [] if choice is None else [choice] if isinstance(choice, str) else list(choice)
    last, since, known = None, time.monotonic(), time.monotonic()
    end = time.monotonic() + 600
    while time.monotonic() < end:
        kind = screen_kind(mem)
        if kind in ("exchange", "army"):
            events["screens"].append(kind)
            if actions and not events["actions"]:
                events["actions"] = run_actions(game, mem, index, actions)
            if (now := screen_kind(mem)) in ("exchange", "army"):
                close_screen(game, mem, now, screen_heroes(mem, now, index))
            last, since, known = None, time.monotonic(), time.monotonic()
            continue
        pix = game.pixels()
        if game.matches("adventure", pix) or in_battle(game, pix):
            known = time.monotonic()
        elif time.monotonic() - known > STRANGE:
            bad = game.out / "not-adventure.xwd"
            game.dump(bad)
            raise TimeoutError(f"neither the adventure map, a battle nor a dialog for {STRANGE} s "
                               f"(the game over?; the display: {bad})")
        if in_battle(game, pix):
            # E08 T68b (batch E8b's E8-13, E8-14): the hero's tile as the battle opens.
            at = mem.hero(index)["tile"]
            events["battles"].append({**fight(game, mem, battle), "hero_tile": at})
            last, since = None, time.monotonic()
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
            events["questions"].append({"record": record, "kind": kind, "answer": said})
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
         choice=None, once=False, again_if=None):
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
    game.click(*hover(game, mem, x, y, z), 0.3)
    game.move(*PARK)
    time.sleep(0.3)
    path = mem.path()
    # The game walks on a click on the hero's current destination: a path to this tile set
    # before makes the first click walk.
    first_walked = observe(mem, index) != before
    again = not once and (again_if is None or len(path) == again_if["marks"])
    if not first_walked and again:
        game.click(*hover(game, mem, x, y, z), 0.3)
        game.move(*PARK)
    after, events = settle(game, mem, index, answer, battle, level_up, actions, choice)
    colours = sorted(set(path.values()))
    return {
        "hero": name, "index": index, "from": before["tile"], "target": list(target),
        "tile": after["tile"], "marks": len(path), "mark": path.get((x, y, z), "none"),
        "path": colours, "tiles": sorted(path, key=lambda t: (t[2], t[1], t[0])),
        "arrived": after["tile"] == [x, y, z],
        "moved": after["tile"] != before["tile"], "points_before": before["points"],
        "points": after["points"], "first_click_walked": first_walked,
        "again": "yes" if again else "no", **events,
    }


def press(game, mem, name, key):
    """Selects the hero, centres the view on it and presses `key` once (E08 T68a, batch E8a's
    E8-7: the arrow key that steps the selected hero), waiting until it rests (settle); returns
    the observation: `key` says whether the hero stepped, the view scrolled or nothing moved."""
    hero = select(game, mem, name)
    index = hero["index"]
    centre(game, mem, *hero["tile"])
    game.move(*PARK)
    time.sleep(0.3)
    before, view = observe(mem, index), mem.view()
    game.key(key, 0.5)
    after, events = settle(game, mem, index)
    moved = after["tile"] != before["tile"]
    return {"hero": name, "index": index, "from": before["tile"], "tile": after["tile"],
            "key": "stepped" if moved else "scrolled" if mem.view() != view else "nothing",
            "moved": moved, "points_before": before["points"], "points": after["points"],
            **events}


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
                0x643854: "army", 0x643C14: "creature", 0x63B8F8: "split", 0x63DB40: "message"}
# Each screen's items by name (survey E08-army §1.1, §2.3, §3.1-§3.4, §4, §5.1: the procedures'
# item ids; checked under Wine, E08 T47a). A name `<part>:<n>` is the part's n-th (an army slot
# from 0 at the left, a worn position by its number 0-18, a backpack place from 0 at the left of
# the five shown); a range's first id here. The hero screen (`hero_screen_proc` 0x4DD2D0): army
# slots 68-74, worn positions 2-20, backpack places 40-44, its arrows 77 (left: the last becomes
# the first) and 78 (right: the first becomes the last), loose 122, tight 124, the tactics
# formation 126, split 127, close 30720. The exchange (`swap_manager_proc` 0x5AFDF0; left the
# visitor, right the visited): armies 13-19 and 20-26, worn positions 27-45 and 46-64, backpack
# places 89-93 and 94-98, their arrows 99 and 101, 100 and 102, the split buttons 103 and 104,
# close 30720. The army dialog (0x5D0910): the group's slots (upper) 115-121, the hero's army
# (lower) 140-146, split 154, OK 30722. The creature dialog (0x4C6C50): dismiss 30723, OK 30722.
# The split dialog (0x449E90): OK 30722, cancel 30721, the slider 6, the counts 4 (the source)
# and 5 (the target). A message: OK 30722; a question: yes 30725, no 30726; since E08 T57 a
# choice's two pictures' frames left 30729 (0x7809) and right 30730 (0x780A). The battle: the
# tactics phase's Start 30722.
SCREEN_ITEMS = {
    "hero": {"army": 68, "worn": 2, "backpack": 40, "backpack left": 77, "backpack right": 78,
             "loose": 122, "tight": 124, "tactics": 126, "split": 127, "close": 30720},
    "exchange": {"left army": 13, "right army": 20, "left worn": 27, "right worn": 46,
                 "left backpack": 89, "right backpack": 94, "left backpack left": 99,
                 "left backpack right": 101, "right backpack left": 100,
                 "right backpack right": 102, "left split": 103, "right split": 104,
                 "close": 30720},
    "army": {"upper": 115, "lower": 140, "split": 154, "ok": 30722},
    "creature": {"dismiss": 30723, "ok": 30722},
    "split": {"ok": 30722, "cancel": 30721},
    "message": {"ok": 30722, "yes": 30725, "no": 30726, "left": 30729, "right": 30730},
    "battle": {"start": 30722},
}
# The counts each screen shows under its army slots (text items, first id): the hero screen's
# 61-67, the exchange's 65-71 and 72-78, the army dialog's 108-114 and 133-139.
SHOWN_COUNTS = {("hero", "army"): 61, ("exchange", "left army"): 65,
                ("exchange", "right army"): 72, ("army", "upper"): 108, ("army", "lower"): 133}
SPLIT_SLIDER, SPLIT_COUNTS = 6, (4, 5)
SHOWN = 0x4  # an item's state bit while it is drawn (an empty slot's count, an empty place: clear)
QUESTION = (30725, 30726)  # a message's yes and no
EXCHANGE_MANAGER, EXCHANGE_HEROES = 0x6A3D30, 0x40  # swap_manager_create 0x5AE500: this
SCREEN_ACTIONS = ("click", "right", "slider")


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
    with " the " in it GENRLTXT:16, `%s the %s`); `?` for neither."""
    text = mem.dialog_text()
    lines = [" ".join(line.split()) for line in [text, *text.split("\n")]]
    for has_text, own in mem.messages():
        if has_text and " ".join(own.split()) in lines:
            return f'the map\'s text "{own}"'
    return h3lod.identify(text) or "?"


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


def run_actions(game, mem, step_hero, actions):
    """Plays a step's `actions` in the screens showing (E08 T47a): each action one `click` (a list
    of item names, clicked in order on whichever screen shows at each), `right` (an item's name)
    or `slider = "max"` (the split dialog's), or none (only reads); a question that comes is
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
               "before": snapshot(mem, step_hero)}
        names = action.get("click", [])
        for name in [names] if isinstance(names, str) else names:
            heroes = screen_heroes(mem, screen_kind(mem), step_hero)
            click_item(game, mem, name, heroes)
            pass_dialogs(game, mem, obs, action.get("answer"), heroes)
        if "right" in action:
            heroes = screen_heroes(mem, screen_kind(mem), step_hero)
            obs["shown"], asked = right_click(game, mem, action["right"], heroes,
                                              action.get("answer"))
            if asked is not None:
                obs["questions"].append({"record": asked, "kind": "other",
                                         "answer": action["answer"]})
            pass_dialogs(game, mem, obs, action.get("answer"), heroes)
        if "slider" in action:
            if action["slider"] != "max":
                raise ValueError(f"`slider = {action['slider']!r}`: only \"max\"")
            slider_max(game, mem)
        obs["screen"] = screen_kind(mem)
        obs["after"] = snapshot(mem, step_hero)
        obs["record"] = {field: value(obs, what, mem)
                         for field, what in action.get("record", {}).items()}
        obs["notes"] = [f"{label}: {got}" for label, what in action.get("note", {}).items()
                        if (got := value(obs, what, mem)) != "none"]
        done.append(obs)
    return done


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
    """Closes a hero screen, exchange or army dialog an action left open (its close or OK)."""
    if screen_kind(mem) == kind:
        click_item(game, mem, "ok" if kind == "army" else "close", heroes)


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


def hero_value(mem, hero, field):
    """A hero's detail (`hero:<field>`): experience, level, primary (attack/defense/power/
    knowledge), army, skill:<name> (none, basic, advanced, expert), on_map (yes/no), tile; since
    E08 T32b1 skills (`Leadership expert, Wisdom basic` in the hero's order, `none`), seed,
    wisdom_offer, school_offer (hero +0x8F, +0x90, +0x3F), artifacts (worn: `Ballista (war
    machine 1), …` in slot order, `none`) and backpack (in its order, `none`); since E08 T32b2
    equipment (the worn artifacts but the war machines and the spell book)."""
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
    a coastal tile without an object."""
    tile = mem.object_at(x, y, z)
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
              "boats", "drawn", "boat_record")


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
        # E08 T57: several observations in one field, joined by `; `.
        return "; ".join(value(obs, w, mem) for w in what)
    if what in obs.get("pre", {}):
        return obs["pre"][what]  # E08 T57: read right before the step (`pre_read`)
    if what.startswith("before:"):
        if what.split(":")[1] in LIVE_READS:
            # The snapshot holds no tiles: the read would be the state after the step.
            raise ValueError(f"`{what}`: read a tile before a step in a step of its own")
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
            return f"{mem.object_at(x, y, z)['word']:#010x}"
        return object_text(mem, x, y, z)
    if what == "heroes:experience":
        return pool_experience(mem)
    if what == "question":
        return ", ".join(q["kind"] for q in obs.get("questions", [])) or "none"
    if what == "questions":
        return ", ".join(q["record"] or "?" for q in obs.get("questions", [])) or "none"
    battles = obs.get("battles", [])
    if what == "battle":
        return yes[bool(battles)]
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
    section with several tables without labels: batch E8b's E8-8)."""
    lines, section, out, columns, rows = template.splitlines(), None, [], [], []
    for line in lines:
        m = re.match(r"## (\S+) ", line)
        if m:
            section, columns = m.group(1), []
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
            if m and (section, m.group(1)) in values:
                line = f"- {m.group(1)}: {values.pop((section, m.group(1)))}"
            elif m and m.group(1) == "anything odd" and notes.get(section):
                line = f"- anything odd: {'; '.join(notes.pop(section))}"
            cells = table_cells(line)
            if cells is not None and TABLE_RULE.fullmatch(line.strip()) and out:
                columns = table_cells(out[-1]) or []
                if columns and (section, "rows:" + columns[0].strip()) in values:
                    rows = list(values.pop((section, "rows:" + columns[0].strip())))
            elif cells is not None and len(cells) == len(columns) > 1:
                label = cells[0].strip()
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
        difficulty = recipe.get("difficulty", "normal")
        restarts = []
        while True:
            game.start(map_path)
            to_hot_seat(game, recipe["players"], difficulty)
            mem = h3mem.Mem()
            if mem.difficulty() != difficulty:
                raise RuntimeError(f"the game's difficulty is {mem.difficulty()}, not {difficulty}")
            # E08 T57: a new game whose draws miss what the recipe requires is started again.
            missing = unmet(recipe, mem)
            if not missing:
                break
            restarts.append(missing)
            if len(restarts) >= RESTARTS:
                raise RuntimeError(f"{RESTARTS} new games, none met the recipe's `require`")
            game.stop()
        for step in (s for step in recipe["step"] for s in expand(step, mem)):
            if step.get("end_turn"):
                obs = {"end_turn": True, "player": end_turn(game, mem)}
            else:
                prepared = None
                if "walk" in step and (step.get("days") or step.get("near")):
                    prepared = ready(game, mem, step["hero"], step["walk"], step.get("days", 0),
                                     step.get("near", False))
                index = select(game, mem, step["hero"])["index"]
                before = snapshot(mem, index)
                pre = pre_read(step, before, mem)
                if sum(k in step for k in ("walk", "equip", "screen", "key")) > 1:
                    raise ValueError("a step walks, equips, opens a screen or presses a key "
                                     "(E08 T37a, T47a, T68a)")
                # E08 T68a (batch E8a's E8-3): `only_if = "aboard"` plays the step only while its
                # hero is in a boat (status bit 18); else its fields read `not played`.
                skip = None
                if step.get("only_if") not in (None, "aboard"):
                    raise ValueError(f"`only_if = {step['only_if']!r}`: only \"aboard\"")
                if step.get("only_if") == "aboard" and not before["hero"]["status"] & h3mem.IN_BOAT:
                    skip = f"not played: {step['hero']} is not in a boat"
                if "actions" in step and "walk" not in step and "screen" not in step:
                    raise ValueError("`actions` play in a screen a walk or `screen` opens")
                if skip:
                    obs = {"hero": step["hero"], "skipped": skip}
                elif "walk" in step:
                    obs = walk(game, mem, step["hero"], step["walk"], step.get("answer"),
                               step.get("battle"), step.get("level_up", "left"),
                               step.get("actions"), step.get("choice"), step.get("once", False),
                               step.get("again_if"))
                    if prepared:
                        # E08 T57: the route set by `ready` makes the first click walk.
                        obs["first_click_walked"] &= not prepared.pop("clicked")
                        obs.update({k: v for k, v in prepared.items() if v})
                elif "key" in step:
                    obs = press(game, mem, step["hero"], step["key"])
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
            if step.get("dump"):
                # E08 T68a: the screen after the step (`dump = "<name>"`: <out>/<name>.xwd), for an
                # agent to look at (batch E8a-water's E8-5: what the view draws under the hero).
                game.dump(out / f"{step['dump']}.xwd")
                obs["dump"] = str(out / f"{step['dump']}.xwd")
            # E08 T47a: an action's records and notes were read right after it; the step's
            # after the step.
            obs["record"] = {}
            for done in obs.get("actions", []):
                obs["record"].update(done["record"])
                notes.setdefault(step["encounter"], []).extend(done["notes"])
            obs["record"].update({field: ([[obs["skipped"]]] if field.startswith("row:")
                                          else obs["skipped"]) if obs.get("skipped")
                                  else record_value(obs, field, what, mem)
                                  for field, what in step.get("record", {}).items()})
            steps.append(obs)
            log.write(json.dumps(obs) + "\n")
            log.flush()
            print(json.dumps(obs), flush=True)
            for field, got in obs["record"].items():
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
            for label, what in step.get("note", {}).items():
                if (got := value(obs, what, mem)) != "none":
                    notes.setdefault(step["encounter"], []).append(f"{label}: {got}")
    finally:
        game.stop()
        log.close()
    header = (f"> Played by the capture harness on {date.today().isoformat()} "
              f"(`clients/capture/h3adv.py batch {shown(recipe_path)}`): "
              "the same Complete 4.0 `Heroes3.exe` under Wine, without the HD mod; hot seat, "
              f"difficulty {recipe.get('difficulty', 'normal').capitalize()}, the starting bonus "
              "random, Show Route on; read from the game's memory."
              + (f" Started again {len(restarts)} time(s) for the recipe's `require` "
                 f"({'; '.join(', '.join(m) for m in restarts)} missing)." if restarts else ""))
    filled = fill(Path(template).read_text(), values, notes, header)
    result = out / f"{name}.md"
    result.write_text(filled)
    print(f"batch: {len(steps)} steps; {result}")
    return result


def records(text):
    """A filled template's record lines: {(section, field or row label): value}, a cell of a
    table of more than one value column under `<label> | <column>` (E08 T37a)."""
    found, section, recording, columns = {}, None, False, []
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
                columns = cells  # the header
            elif not TABLE_RULE.fullmatch(line.strip()) and len(cells) == len(columns) > 1:
                for j in range(1, len(cells)):
                    found[cell_key(section, cells[0].strip(), columns, j)] = cells[j].strip()
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
