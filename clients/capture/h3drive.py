#!/usr/bin/env python3
"""The capture driver (E17 M2 T1, *provisional*; clients/capture/README.md): runs the original
under Wine (after setup.sh), steers it through its menus by fixed coordinates, recognises each
screen by a few probe pixels instead of screenshots, and controls the game's clock through our
xdd.dll (xdd.c: freeze, step, run), so that every frame is taken at an exact game time.

  h3drive.py shot <out.xwd>                 dump the display now (a game must be running)
  h3drive.py learn <screen> [--in x0,y0,x1,y1] [--avoid x0,y0,x1,y1]... [--also <dump.xwd>]...
                                            record the probes of the screen shown now into
                                            screens.toml (numbers only: positions and 5-6-5
                                            values, ADR-0009), from the part of the screen that
                                            tells it from the others (--in, default all), each
                                            probe the same in every --also dump (the screen in
                                            its other states)
  h3drive.py set [<set.toml>] [--out <dir>] [--only <id>]
                                            the situation set (default clients/situations/set.toml)
                                            unattended: for each situation its map (h3-view
                                            situation), its frames in the original, our render
                                            and the masked diff of each frame against the
                                            recorded count; prints one line per frame, writes
                                            <dir>/frames.sha1 (the pixels' digests: two runs
                                            agree when these do), exits 1 on any change
                                            (default <dir>: clients/out/set)
  h3drive.py battle <map.h3m> <out-dir> [--frames <ms,...>] [--key <key>] [--keep]
                                            the common use: start a single scenario on the map,
                                            step the selected hero into the enemy with <key>
                                            (default Right), freeze the clock, find the battle's
                                            opening and dump a frame at each time (milliseconds
                                            after the opening, default 0)

  h3drive.py sequence <map.h3m> <out-dir> [--pre <ms>] [--action <a>]... [--ms <n>] [--screen]
                    [--hot-seat] [--heroes] [--screens]
                                            a sequence frame by frame (M2 T3): the opening, the
                                            clock stepped <pre> ms, the actions at the frozen
                                            clock (hex:<n> clicks a hex, click:x,y, move:x,y,
                                            key:<name>, step:<ms>, cast:<k>[:<hex>] casts the
                                            spell at the hero's book's place k, M3 T7a-2a;
                                            at:<ms> steps 1 ms at a time,
                                            each step read, up to <ms> after the opening before
                                            the actions after it: a battle in progress, M3 T1a;
                                            turn:<side>.<record> steps so until that stack's turn
                                            starts, its time written to <out>/turns.txt),
                                            then <n> steps of 1 ms; after each, the stacks'
                                            animation state read from the game's memory
                                            (h3mem.py) and with --screen a digest of the screen;
                                            writes <out>/states.jsonl and the changes,
                                            <out>/timeline.txt; --hot-seat starts the map in hot
                                            seat (both sides played by the actions, M3 T1a);
                                            --heroes adds the heroes' changes to the timeline
                                            (a hero's cast, M3 T7a-2a); --screens the screen's
                                            changes (where the screen differs from the step
                                            before: the magic animations memory does not show,
                                            M3 T7a-2b)
  h3drive.py sequences [<sequences.toml>] [--out <dir>] [--only <id>]
                                            the recorded sequences (default
                                            clients/situations/sequences.toml) unattended: each
                                            one's timeline against its recorded one; exits 1 on
                                            any change (default <dir>: clients/out/sequences)
  h3drive.py frames [<sequences.toml>] [--out <dir>] [--only <id>] [--reuse]
                                            the frame check (runbook); `frames` and `set` keep
                                            their kept frames' numbers in situations/kept/
  h3drive.py drawn [--out <dir>] [--only <id>] [--record]
                                            the viewer's frames of every sequence and situation
                                            at the times last kept, without the original (no
                                            Wine): each one's pixels outside its comparison's
                                            fixed masks against situations/kept/drawn.txt
                                            (`same`, `CHANGED`, `new`); exits 1 on a
                                            change; --record writes the digests (runbook, "Which
                                            captures a change needs"; default <dir>:
                                            clients/out/drawn)

Library use: `Game` (start, wait, click, key, clock, settled, step_until, dump, stop), the
recipes (`to_scenario`, `into_battle`, `hex_point`), `capture` and `sequence`. Needs numpy,
xdotool, xwd, Xvfb and Wine.
"""

import os
import re
import shutil
import struct
import subprocess
import sys
import time
import tomllib
from pathlib import Path

import numpy as np

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent.parent
GAME_DIR = ROOT / "original-data" / "wine" / "game"
SCREENS = HERE / "screens.toml"
DISPLAY = ":99"
WIDTH, HEIGHT = 800, 600
# Where the battle's recipes park the mouse: the game draws its own cursor, so in the panel, off
# the battlefield, beside the log's item (the layout's [log], compared since M3 T4b-2b-1) and
# below its scroll arrows, more than 50 px from the probes; hovering nothing that shows a hint.
PARK = (618, 593)
# How long `turn:` waits for the stack's turn (M3 T7a-2a): a human's stack waiting for an action
# would keep the clock stepping for ever.
TURN_LIMIT = 30000


def xwd565(data):
    """The pixels of an XWD dump of a 16-bit TrueColor display (bytes), as 5-6-5 values."""
    head = struct.unpack(">25I", data[:100])
    header_size, depth, width, height = head[0], head[3], head[4], head[5]
    byte_order, bytes_per_line, colours = head[7], head[12], head[19]
    if depth != 16 or byte_order != 0 or head[14:17] != (0xF800, 0x7E0, 0x1F):
        raise ValueError("not a little-endian 5-6-5 dump")
    start = header_size + 12 * colours
    rows = np.frombuffer(data, np.uint8, height * bytes_per_line, start)
    rows = rows.reshape(height, bytes_per_line)[:, : 2 * width]
    return rows.copy().view("<u2").astype(np.uint16)


def env():
    e = dict(os.environ)
    e.setdefault("WINEPREFIX", str(Path.home() / ".wine-h3"))
    e["WINEDEBUG"] = "-all"
    e["DISPLAY"] = DISPLAY
    return e


def run(*args, **kw):
    return subprocess.run(args, env=env(), check=True, **kw)


def xwd_bytes():
    # A dump right after the display starts can fail once (seen in M3 T2's set runs), or come
    # back as no 5-6-5 dump (twice in one set run in M3 T5b-2a, between two situations): try
    # again.
    for attempt in range(5):
        try:
            data = run("xwd", "-root", "-silent", capture_output=True).stdout
            xwd565(data)
            return data
        except (subprocess.CalledProcessError, ValueError, struct.error):
            if attempt == 4:
                raise
            time.sleep(0.5)


def load_screens():
    return tomllib.loads(SCREENS.read_text()) if SCREENS.exists() else {}


class Game:
    """One run of the original under Wine with our clock."""

    def __init__(self, out):
        self.out = Path(out)
        self.out.mkdir(parents=True, exist_ok=True)
        self.clock_dir = self.out / "clock"
        self.seq = 0
        self.screens = load_screens()
        self.proc = None

    # -- the process ------------------------------------------------------------------------
    def start(self, map_path=None):
        if subprocess.run(["pgrep", "-f", f"Xvfb {DISPLAY}"], capture_output=True).returncode:
            subprocess.Popen(
                ["Xvfb", DISPLAY, "-screen", "0", f"{WIDTH}x{HEIGHT}x16", "-nolisten", "tcp"],
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
            )
            time.sleep(1)
        if map_path:
            maps = GAME_DIR / "Maps"
            for old in maps.iterdir():
                if old.is_file():
                    old.unlink()
            shutil.copy(map_path, maps / Path(map_path).name)
        # A game left running (an earlier run's `--keep`, an interrupted batch) takes the display
        # and makes the dumps unreadable: end it first (E17 H2).
        subprocess.run(["wineserver", "-k"], env=env(), check=False)
        if self.clock_dir.exists():
            shutil.rmtree(self.clock_dir)
        self.clock_dir.mkdir()
        e = env()
        e["H3CLOCK"] = "Z:" + str(self.clock_dir.resolve()).replace("/", "\\")
        log = open(self.out / "wine.log", "w")
        self.proc = subprocess.Popen(
            ["wine", "Heroes3.exe"], cwd=GAME_DIR, env=e, stdout=log, stderr=log
        )

    def stop(self):
        subprocess.run(["wineserver", "-k"], env=env(), check=False)
        if self.proc:
            self.proc.wait(timeout=30)

    # -- the screen ---------------------------------------------------------------------------
    def pixels(self):
        return xwd565(xwd_bytes())

    def dump(self, path):
        data = xwd_bytes()
        Path(path).write_bytes(data)
        return xwd565(data)

    def matches(self, name, pix=None):
        pix = self.pixels() if pix is None else pix
        probes = self.screens[name]["probes"]
        return all(int(pix[y, x]) == v for x, y, v in probes)

    def wait(self, name, timeout=60.0):
        """Waits until the screen `name`'s probes all match (the clock running)."""
        end = time.monotonic() + timeout
        while time.monotonic() < end:
            if self.matches(name):
                return
            time.sleep(0.1)
        bad = self.out / f"not-{name}.xwd"
        self.dump(bad)
        raise TimeoutError(f"the screen `{name}` did not appear (the display: {bad})")

    # -- input --------------------------------------------------------------------------------
    def click(self, x, y, settle=0.3):
        run("xdotool", "mousemove", str(x), str(y))
        time.sleep(0.1)
        run("xdotool", "click", "1")
        time.sleep(settle)

    def move(self, x, y):
        run("xdotool", "mousemove", str(x), str(y))

    def key(self, name, settle=0.2):
        run("xdotool", "key", name)
        time.sleep(settle)

    # -- the clock ----------------------------------------------------------------------------
    def ack(self):
        try:
            seq, t, reads = (self.clock_dir / "ack").read_text().split()
            return int(seq), int(t), int(reads)
        except (OSError, ValueError):
            return None

    def clock(self, command, settle_reads=20, timeout=10.0):
        """Sends a clock command and waits until the game read the clock `settle_reads` times
        after it (idle at the new time). Returns the clock's time."""
        self.seq += 1
        tmp = self.clock_dir / "ctl.tmp"
        tmp.write_text(f"{self.seq} {command}\n")
        tmp.replace(self.clock_dir / "ctl")
        end = time.monotonic() + timeout
        while time.monotonic() < end:
            a = self.ack()
            if a and a[0] == self.seq and a[2] >= settle_reads:
                return a[1]
            time.sleep(0.002)
        raise TimeoutError(f"the clock did not settle after `{command}` (ack {self.ack()})")

    def settled(self):
        """The display once two dumps in a row agree (Wine may draw a moment after the game)."""
        last = self.pixels()
        for _ in range(20):
            time.sleep(0.02)
            pix = self.pixels()
            if np.array_equal(pix, last):
                return pix
            last = pix
        raise TimeoutError("the display does not settle")

    def step_until(self, name, step_ms=10, limit_ms=20000):
        """Steps the frozen clock by `step_ms` until the screen `name` (or one of a tuple of
        names) shows; returns its time."""
        names = (name,) if isinstance(name, str) else name
        t = None
        for _ in range(limit_ms // step_ms):
            t = self.clock(f"step {step_ms}")
            pix = self.settled()
            if any(self.matches(n, pix) for n in names):
                return t
        bad = self.out / f"not-{names[0]}.xwd"
        self.dump(bad)
        raise TimeoutError(f"the screen `{name}` did not appear within {limit_ms} ms (the display: {bad})")


# -- recipes: the common uses, by fixed coordinates (the menus never move) --------------------
def to_scenario(game):
    """From the start to the adventure map of the only map in Maps/ (a single scenario)."""
    game.wait("main_menu", 90)
    game.click(640, 75, 1.0)  # New Game
    game.wait("new_game_menu")
    game.click(640, 75, 1.0)  # Single Scenario
    game.wait("scenario_list")
    game.click(497, 553, 1.0)  # Begin
    game.move(700, 300)  # away from the edges: a cursor there scrolls the adventure map
    game.wait("adventure_map", 60)


def hex_point(hex_id):
    """A battle hex's point to click: its box (layouts/battle.toml: left 14 + 44 c, 22 more on
    even rows, top 86 + 42 r), 22 right and 26 down."""
    row, col = divmod(hex_id, 17)
    return 14 + 44 * col + (22 if row % 2 == 0 else 0) + 22, 86 + 42 * row + 26


def into_battle(game, key="Right"):
    """From the adventure map: freezes the clock, moves the selected hero with `key` into the
    enemy, and steps the clock to the battle screen (`battle_book` when the acting hero's spell
    book lights its button, M3 T7a-2a). Returns the clock's time at the opening."""
    game.clock("freeze")
    game.key(key)
    game.move(*PARK)
    return game.step_until(("battle", "battle_book"), 10)


def main(args):
    if not args:
        sys.exit(__doc__)
    if args[0] == "shot" and len(args) == 2:
        Path(args[1]).write_bytes(xwd_bytes())
    elif args[0] == "learn" and len(args) >= 2 and len(args) % 2 == 0:
        boxes = {"--in": [], "--avoid": [], "--also": []}
        for flag, value in zip(args[2::2], args[3::2]):
            if flag == "--also":
                boxes[flag].append(xwd565(Path(value).read_bytes()))
            else:
                boxes[flag].append(tuple(map(int, value.split(","))))
        inside = boxes["--in"][0] if boxes["--in"] else (0, 0, WIDTH - 1, HEIGHT - 1)
        learn(args[1], inside, boxes["--avoid"], boxes["--also"])
    elif args[0] == "battle" and len(args) >= 3:
        battle(args[1], args[2], args[3:])
    elif args[0] == "set":
        sys.exit(run_set(args[1:]))
    elif args[0] == "sequence" and len(args) >= 3:
        sequence_command(args[1], args[2], args[3:])
    elif args[0] == "sequences":
        sys.exit(run_sequences(args[1:]))
    elif args[0] == "frames":
        sys.exit(run_frames(args[1:]))
    elif args[0] == "drawn":
        sys.exit(run_drawn(args[1:]))
    else:
        sys.exit(__doc__)


def learn(name, inside, avoid, also=()):
    """Probes: 12 points of a grid that keep their value over three seconds, away from `avoid`
    and from the cursor (a box around the pointer: the game draws its own cursor)."""
    pointer = run("xdotool", "getmouselocation", "--shell", capture_output=True, text=True)
    pos = dict(line.split("=") for line in pointer.stdout.split())
    px, py = int(pos["X"]), int(pos["Y"])
    avoid = list(avoid) + [(px - 50, py - 50, px + 50, py + 50)]
    shots = []
    for _ in range(4):
        shots.append(xwd565(xwd_bytes()))
        time.sleep(1)
    a = shots[0]
    shots += list(also)
    probes = []
    x0, y0, x1, y1 = inside
    step = max(4, int(((x1 - x0) * (y1 - y0) / 150) ** 0.5))
    for y in range(y0 + step // 2, y1, step):
        for x in range(x0 + step // 2, x1, step):
            if any(ax0 <= x <= ax1 and ay0 <= y <= ay1 for ax0, ay0, ax1, ay1 in avoid):
                continue
            if all(s[y, x] == a[y, x] for s in shots):
                probes.append((x, y, int(a[y, x])))
    # Spread over the screen, preferring distinct values.
    probes.sort(key=lambda p: p[2])
    pick = probes[:: max(1, len(probes) // 12)][:12]
    screens = load_screens()
    screens[name] = {"probes": pick}
    write_screens(screens)
    print(f"{name}: {len(pick)} probes")


def write_screens(screens):
    lines = [
        "# The screens the capture driver recognises (E17 M2 T1, *provisional*; h3drive.py learn):",
        "# a screen shows when every probe [x, y, 5-6-5 value] matches. Numbers only (ADR-0009).",
        "# [button.<name>] (M2 T6, h3adv.py learn-button): a button found anywhere on the screen,",
        "# its probes relative to its top-left corner, `size` its width and height.",
    ]
    for name in sorted(screens):
        if name == "button":
            continue
        lines.append(f"\n[{name}]\nprobes = [")
        lines += [f"    [{x}, {y}, {v}]," for x, y, v in screens[name]["probes"]]
        lines.append("]")
    for name, button in sorted(screens.get("button", {}).items()):
        lines.append(f"\n[button.{name}]\nsize = {list(button['size'])}\nprobes = [")
        lines += [f"    [{x}, {y}, {v}]," for x, y, v in button["probes"]]
        lines.append("]")
    SCREENS.write_text("\n".join(lines) + "\n")


def capture(game, map_path, out, frames, key="Right", stacks=None, hot_seat=False, mouse=None):
    """The battle of the map: its opening, then a dump at each time in `frames` (ms after the
    opening, ascending); returns the dumps' paths. With `stacks` (a list; M3 T1b-2) the stacks
    the game's memory holds at each dump (h3mem: the fidgets) are appended to it. With
    `hot_seat` (M3 T3b) the map starts in hot seat, Red and Blue both humans, so that a faster
    stack of Blue's opens idle. With `mouse` (M3 T6c-1) the mouse moves there at the opening, the
    clock frozen, after the first turn started (its cursor, the status line's hint, a stack it
    marks: design §9.9)."""
    game.start(map_path)
    if hot_seat:
        import h3adv

        h3adv.to_hot_seat(game, ["Red", "Blue"])
    else:
        to_scenario(game)
    t0 = into_battle(game, key)
    if mouse is not None:
        game.move(*mouse)
        time.sleep(0.3)
    mem = None
    if stacks is not None:
        import h3mem

        mem = h3mem.Mem()
    now, paths = t0, []
    for ms in sorted(frames):
        # At most 100 ms a step (M3 T3a): the game runs one idle step for a longer one (its next
        # time moves to now, 0x496820), so a frame far on would show too few (the outline's
        # pulse, the flags).
        while ms > now - t0:
            now = game.clock(f"step {min(100, ms - (now - t0))}")
        game.settled()
        path = Path(out) / f"t{ms:05d}.xwd"
        game.dump(path)
        paths.append(path)
        if mem is not None:
            stacks.append(mem.stacks())
    return paths


def run_set(rest):
    import hashlib

    import h3cap

    opts = {"set": str(ROOT / "clients" / "situations" / "set.toml")}
    if rest and not rest[0].startswith("--"):
        opts["set"], rest = rest[0], rest[1:]
    opts.update(zip(rest[::2], rest[1::2]))
    set_path = Path(opts["set"])
    out = Path(opts.get("--out", ROOT / "clients" / "out" / "set"))
    situations = tomllib.loads(set_path.read_text())["situation"]
    if "--only" in opts:
        situations = [s for s in situations if s["id"] == opts["--only"]]
    h3view = str(h3cap.H3VIEW)
    failed, digests = 0, []
    for sit in situations:
        sid, scenario = sit["id"], set_path.parent / sit["scenario"]
        here = out / sid
        here.mkdir(parents=True, exist_ok=True)
        map_path = here / f"{sid}.h3m"
        tile = ["--tile", ",".join(map(str, sit["tile"]))] if "tile" in sit else []
        hot_seat = ["--hot-seat"] if sit.get("hot_seat") else []
        subprocess.run([h3view, "situation", str(scenario), "--out", str(map_path), *tile,
                        *hot_seat], check=True, capture_output=True)
        frames = {f["at"]: f for f in sit.get("frame", [])} or {0: {"at": 0}}
        # Where the mouse rests (M3 T6c-1): the situation's point, else the parked one; the
        # viewer draws its cursor and hint there.
        mouse = tuple(sit.get("mouse", PARK))
        game, stacks = Game(here), []
        try:
            paths = capture(game, map_path, here, list(frames), stacks=stacks,
                            hot_seat=bool(hot_seat), mouse=mouse)
        finally:
            game.stop()
        # Our frame at each time (M3 T1b-2): the battle's idle steps (the outline's pulse) on the
        # original's clock, the original's fidgets taken from its memory, as the frame check does.
        kept = [{"t": at, "frame": p.name, "stacks": st}
                for at, p, st in zip(sorted(frames), paths, stacks)]
        (here / "frames.toml").write_text(frames_toml(kept, mouse))
        keep(set_path, "set", sid, here / "frames.toml")
        ours = here / "ours.png"  # its record: ours.jsonl (the count boxes' masks)
        subprocess.run([h3view, "render", str(scenario), "--out", str(ours), "--frames",
                        str(here)], check=True, capture_output=True)
        for (at, frame), path in zip(sorted(frames.items()), paths):
            # The pixels' digest: an XWD file's colour map holds bytes the X server leaves unset.
            pixels = xwd565(path.read_bytes()).tobytes()
            digests.append(f"{hashlib.sha1(pixels).hexdigest()}  {sid}/{path.name}")
            n = h3cap.diff(path, here / "ours" / f"{path.stem}.png", ours.with_suffix(".jsonl"),
                           sit.get("mask", []), here / f"diff-t{at:05d}.png", quiet=True)
            expected = frame.get("differing")
            verdict = "ok" if n == expected else "CHANGED" if expected is not None else "new"
            failed += verdict != "ok"
            print(f"{sid} t={at}: {n} differing (recorded {expected}) {verdict}", flush=True)
    (out / "frames.sha1").write_text("\n".join(digests) + "\n")
    print(f"set: {len(situations)} situations, {failed} not as recorded; digests {out / 'frames.sha1'}")
    return 1 if failed else 0


def battle(map_path, out, rest):
    frames, key, keep = [0], "Right", False
    i = 0
    while i < len(rest):
        if rest[i] == "--frames":
            frames = [int(t) for t in rest[i + 1].split(",")]
            i += 2
        elif rest[i] == "--key":
            key = rest[i + 1]
            i += 2
        elif rest[i] == "--keep":
            keep, i = True, i + 1
        else:
            sys.exit(__doc__)
    game = Game(out)
    try:
        capture(game, map_path, out, frames, key)
        print(f"battle: frames {sorted(frames)} after the opening in {out}")
    finally:
        if not keep:
            game.stop()


def book_place(k):
    """The screen's centre of the spell book's place k (0 to 11; M3 T7a-2a, design §7.16): row by
    row, the left page first, 78 x 65 each from the dialog's (117, 89) and (333, 89) at (90, 2)."""
    return (246 + 86 * (k % 2) + 216 * (k // 6), 123 + 96 * (k % 6 // 2))


def act(game, action):
    """One action at the frozen clock: hex:<n>, click:x,y, move:x,y, key:<name>, step:<ms>,
    cast:<k>[:<hex>] (M3 T7a-2a: the hero's spell book by its key C, its place k, then the target
    hex if the spell takes one)."""
    kind, _, arg = action.partition(":")
    if kind == "cast":
        place, _, target = arg.partition(":")
        if not 0 <= int(place) < 12:
            raise ValueError(f"`{action}`: the book has the places 0 to 11")
        game.key("c", 0.5)
        game.move(*book_place(int(place)))
        time.sleep(0.3)
        game.click(*book_place(int(place)), settle=0.5)
        if target:
            game.move(*hex_point(int(target)))
            time.sleep(0.3)
            game.click(*hex_point(int(target)))
        game.move(*PARK)
        time.sleep(0.3)
    elif kind == "hex":
        game.move(*hex_point(int(arg)))
        time.sleep(0.3)
        game.click(*hex_point(int(arg)))
        game.move(*PARK)
        time.sleep(0.3)
    elif kind in ("click", "move"):
        x, y = map(int, arg.split(","))
        (game.click if kind == "click" else game.move)(x, y)
        time.sleep(0.3)
        if kind == "click":
            game.move(*PARK)
            time.sleep(0.3)
    elif kind == "key":
        game.key(arg, 0.3)
    elif kind == "step":
        game.clock(f"step {int(arg)}")
    else:
        raise ValueError(f"unknown action `{action}`")


def union(a, b):
    """The smallest box `[x0, y0, x1, y1]` (x1, y1 exclusive) holding boxes a and b (None: none)."""
    if a is None or b is None:
        return a or b
    return [min(a[0], b[0]), min(a[1], b[1]), max(a[2], b[2]), max(a[3], b[3])]


def changed_box(before, after):
    """The box of the pixels that differ between two screens (None: none)."""
    ys, xs = np.nonzero(before != after)
    if len(xs) == 0:
        return None
    return [int(xs.min()), int(ys.min()), int(xs.max()) + 1, int(ys.max()) + 1]


def timeline(states, heroes=False):
    """The changes of the stacks' group, frame and hex: lines `<ms> <side>.<record> g<group>
    f<frame> h<hex>`, the first state of each stack at the sequence's start; after a time's
    stacks, a change of the magic animation drawn (M3 T3b, design §7.15): `<ms> magic <number>
    f<frame> <side>.<record>...` over the stacks that show it, `<ms> magic -` once none does.
    With `heroes` (M3 T7a-2a: a hero's cast, design §7.16) before the magic line each change of
    a hero's group or frame, `<ms> hero <side> g<group> f<frame>`. A state read with the screen's
    changes (`screens`, M3 T7a-2b) ends its time with `<ms> screen <x0>,<y0>,<x1>,<y1>`, the box
    of the pixels that differ from the step before (x1, y1 exclusive): what memory does not show
    (the at-hex and Armageddon players, the projectiles, design §7.16), numbers only."""
    lines, last, magic, box = [], {}, None, None
    for i, state in enumerate(states):
        box = union(box, state.get("changed"))
        if i + 1 < len(states) and states[i + 1]["t"] == state["t"]:
            continue  # a later state at the same time (actions between them) shows instead
        for st in state["stacks"]:
            key = (st["side"], st["record"])
            now = (st["group"], st["frame"], st["hex"])
            if last.get(key) != now:
                lines.append(f"{state['t']} {key[0]}.{key[1]} g{now[0]} f{now[1]} h{now[2]}")
                last[key] = now
        for hero in state["heroes"] if heroes else ():
            key, now = ("hero", hero["side"]), (hero["group"], hero["frame"])
            if last.get(key) != now:
                lines.append(f"{state['t']} hero {hero['side']} g{now[0]} f{now[1]}")
                last[key] = now
        over = [f"{st['side']}.{st['record']}" for st in state["stacks"] if st.get("magic")]
        drawn = state["clocks"].get("magic")
        if over and drawn:
            line = f"magic {drawn[0]} f{drawn[1]} " + " ".join(over)
        else:
            line = "magic -" if magic else None  # erased, or none yet
        if line != magic:
            lines.append(f"{state['t']} {line}")
            magic = line
        if box is not None:
            lines.append(f"{state['t']} screen " + ",".join(map(str, box)))
            box = None
    return lines


def sequence(game, map_path, out, pre=0, actions=(), ms=1000, screen=False, frames=False,
             hot_seat=False, heroes=False, screens=False):
    """The battle of the map: its opening, `pre` ms on, the actions, then `ms` steps of 1 ms,
    each followed by the game's state (h3mem); returns the states (times in ms after the
    opening, the clocks relative to it). An action `at:<ms>` (M3 T1a) steps 1 ms at a time, each
    step read, until `<ms>` after the opening, and the actions after it act then: a battle in
    progress; an action `turn:<side>.<record>` steps so until that stack's turn starts (the game
    names it the active stack), the times written to `<out>/turns.txt`. With `hot_seat` the map starts in hot seat, Red and Blue both humans, so the
    actions play both sides. With `frames` (M2 T4b-1) every distinct screen the steps show is
    kept: `<out>/frames/t<ms>.npz` (the whole screen, 5-6-5), its first time and its state in
    `<out>/frames.jsonl`. A screen is distinct by its battlefield and, since M3 T6b, its panel
    (the log's lines and the buttons change there alone), since M3 T6c-2 its cursor too (the
    viewer draws it). With `heroes` (M3 T7a-2a) the timeline has the heroes' changes too; with
    `screens` (M3 T7a-2b) each state the box where the screen changed since the step before
    (`changed`), which the timeline lists."""
    import hashlib
    import json

    import h3mem

    game.start(map_path)
    if hot_seat:
        import h3adv

        h3adv.to_hot_seat(game, ["Red", "Blue"])
    else:
        to_scenario(game)
    t0 = into_battle(game)
    mem = h3mem.Mem()
    # The mouse poll's grid from the opening (M3 T6c-2): the cursor moves at the same times in
    # every run.
    mem.align_mouse_poll(t0)
    now = game.clock(f"step {pre}") if pre else t0
    kept = Path(out) / "frames"
    if frames:
        shutil.rmtree(kept, ignore_errors=True)
        kept.mkdir(parents=True)
    last, before = [None], [None]

    def state(t):
        clocks = mem.globals()
        for k in ("frame_deadline", "idle_step_time"):
            clocks[k] -= t0
        s = {"t": t - t0, "clocks": clocks, "stacks": mem.stacks(), "heroes": mem.battle_heroes()}
        # The log's item and entries (M3 T6b): what the frame check compares the viewer's with.
        s["log"] = mem.battle_log()
        if s["log"] is not None:
            s["log"]["at"] -= t0
        if screen or frames or screens:
            pix = game.settled()
            s["screen"] = hashlib.sha1(pix.tobytes()).hexdigest()[:12]
            if screens:
                s["changed"] = None if before[0] is None else changed_box(before[0], pix)
                before[0] = pix
            if frames and s["screen"] != last[0]:
                name = f"t{t - t0:05d}.npz"
                if (kept / name).exists():  # the actions changed it at the same time
                    name = f"t{t - t0:05d}b.npz"
                np.savez_compressed(kept / name, screen=pix)
                s["kept"] = name
            last[0] = s["screen"]
        return s

    def step():
        # 500 reads, not 20: under load the computer's turn can start after 20 reads at the new
        # time (its draw read a step late, the runbook's "Traps"); 500 cost about 3 ms a step.
        return game.clock("step 1", settle_reads=500)

    states, turns, stalled = [state(now)], [], None
    for a in actions:
        kind, _, arg = a.partition(":")
        if kind in ("at", "turn"):
            states.append(state(now))  # the state after the actions so far, at their time
            # A turn: the stack named is the active one. (The game names a striker again while
            # it turns back after its attack, before the next stack's turn: M3 T1a.)
            whose = [int(n) for n in arg.split(".")] if kind == "turn" else None
            since = now
            while (states[-1]["clocks"]["active"] != whose if whose
                   else now - t0 < int(arg)):
                if whose and now - since > TURN_LIMIT:  # M3 T7a-2a: another stack waits
                    stalled = (f"`{a}`: no such turn in {TURN_LIMIT} ms; the active stack "
                               f"{states[-1]['clocks']['active']}")
                    break
                now = step()
                states.append(state(now))
            if stalled:
                break
            turns.append(now - t0)
        else:
            act(game, a)
    if not stalled:
        states.append(state(now))
        for _ in range(ms):
            now = step()
            states.append(state(now))
    if frames:
        with open(Path(out) / "frames.jsonl", "w") as f:
            for s in states:
                if "kept" in s:
                    f.write(json.dumps({"frame": s.pop("kept"), **s}) + "\n")
    out = Path(out)
    with open(out / "states.jsonl", "w") as f:
        for s in states:
            f.write(json.dumps(s) + "\n")
    (out / "timeline.txt").write_text("\n".join(timeline(states[1:], heroes)) + "\n")
    (out / "turns.txt").write_text("".join(f"{t}\n" for t in turns))
    if stalled:  # M3 T7a-2b: the files above show how far it came
        raise RuntimeError(f"{stalled} (the states so far: {out / 'states.jsonl'})")
    return states


def sequence_command(map_path, out, rest):
    opts = {"--pre": "0", "--ms": "1000"}
    actions, screen, frames, hot_seat, heroes, screens, i = [], False, False, False, False, False, 0
    while i < len(rest):
        if rest[i] == "--action":
            actions.append(rest[i + 1])
            i += 2
        elif rest[i] == "--screen":
            screen, i = True, i + 1
        elif rest[i] == "--hot-seat":
            hot_seat, i = True, i + 1
        elif rest[i] == "--heroes":
            heroes, i = True, i + 1
        elif rest[i] == "--screens":
            screens, i = True, i + 1
        elif rest[i] == "--frames":
            frames, i = True, i + 1
        elif rest[i] in opts:
            opts[rest[i]] = rest[i + 1]
            i += 2
        else:
            sys.exit(__doc__)
    Path(out).mkdir(parents=True, exist_ok=True)
    game = Game(out)
    try:
        sequence(game, map_path, out, int(opts["--pre"]), actions, int(opts["--ms"]), screen,
                 frames, hot_seat, heroes, screens)
    finally:
        game.stop()
    print(f"sequence: {out}/timeline.txt")


def situation_args(h3view, set_path, entry, map_path):
    """`h3-view situation`'s command line for a sequence's map (Blue a human too in hot seat)."""
    args = [h3view, "situation", str(set_path.parent / entry["scenario"]), "--out", str(map_path)]
    return args + (["--hot-seat"] if entry.get("hot_seat") else [])


def run_sequences(rest):
    opts = {"set": str(ROOT / "clients" / "situations" / "sequences.toml")}
    if rest and not rest[0].startswith("--"):
        opts["set"], rest = rest[0], rest[1:]
    opts.update(zip(rest[::2], rest[1::2]))
    set_path = Path(opts["set"])
    out = Path(opts.get("--out", ROOT / "clients" / "out" / "sequences"))
    entries = tomllib.loads(set_path.read_text())["sequence"]
    if "--only" in opts:
        entries = [e for e in entries if e["id"] == opts["--only"]]
    h3view = str(ROOT / "clients" / "target" / "release" / "h3-view")
    failed = 0
    for e in entries:
        here = out / e["id"]
        here.mkdir(parents=True, exist_ok=True)
        map_path = here / f"{e['id']}.h3m"
        subprocess.run(situation_args(h3view, set_path, e, map_path), check=True,
                       capture_output=True)
        game = Game(here)
        try:
            sequence(game, map_path, here, e.get("pre", 0), e.get("actions", []), e["ms"],
                     hot_seat=e.get("hot_seat", False), heroes=e.get("heroes", False),
                     screens=e.get("screens", False))
        finally:
            game.stop()
        recorded = set_path.parent / e["timeline"]
        got = (here / "timeline.txt").read_text()
        if not recorded.exists():
            verdict = "new"
        else:
            verdict = "ok" if got == recorded.read_text() else "CHANGED"
        failed += verdict != "ok"
        print(f"{e['id']}: {len(got.splitlines())} changes in {e['ms']} ms {verdict}", flush=True)
    print(f"sequences: {len(entries)}, {failed} not as recorded; timelines under {out}")
    return 1 if failed else 0


def frames_toml(kept, mouse=None):
    """The kept frames for `h3-view sequences --frames`: each one's time, file and the stacks the
    original shows fidgeting (group 1: the viewer draws none of its own in a check), and, where
    the original's memory was read for them (the frame check, M3 T3a), its heroes' groups and
    frames (their defeats and victories are drawn from its generator; the flags are the
    viewer's); a situation's mouse first (M3 T6c-1: `h3-view render --frames` draws its cursor,
    hint and marked stack); a sequence's frame the point the original's cursor is drawn at (M3
    T6c-2: its memory, `cursor_at`; the game reads the mouse on a 33 ms grid of its own, design
    §9.10)."""
    out = ["# The original's kept frames (h3drive.py frames): numbers only.", ""]
    if mouse is not None:
        out += [f"mouse = [{mouse[0]}, {mouse[1]}]", ""]
    for k in kept:
        fidgets = ", ".join(f"[{st['side']}, {st['record']}, {st['group']}, {st['frame']}]"
                            for st in k["stacks"] if st["group"] == 1 and st["count"] > 0)
        out += ["[[frame]]", f"t = {k['t']}", f'file = "{k["frame"]}"', f"fidgets = [{fidgets}]"]
        if "heroes" in k:
            heroes = ", ".join(f"[{h['side']}, {h['group']}, {h['frame']}]" for h in k["heroes"])
            out.append(f"heroes = [{heroes}]")
        at = (k.get("clocks") or {}).get("cursor_at")
        if at is not None:
            out.append(f"cursor = [{at[0]}, {at[1]}]")
        out.append("")
    return "\n".join(out)


def run_frames(rest):
    """The frame check (M2 T4b-1, design §7.9): for each recorded sequence, every distinct
    battlefield the original shows (its frames kept by `sequence`), the viewer's frame at the
    same time (`h3-view sequences --frames`) and the masked diff of each pair against the counts
    recorded in the sequence's `frames` file (a line `<ms> <differing> <redraw artifacts>`, then
    ` n<side>.<record>` for each count box masked as its count is not the original's: T5b);
    exits 1 when one changed."""
    import json

    import h3cap

    opts = {"set": str(ROOT / "clients" / "situations" / "sequences.toml")}
    if rest and not rest[0].startswith("--"):
        opts["set"], rest = rest[0], rest[1:]
    reuse = "--reuse" in rest
    rest = [r for r in rest if r != "--reuse"]
    opts.update(zip(rest[::2], rest[1::2]))
    set_path = Path(opts["set"])
    out = Path(opts.get("--out", ROOT / "clients" / "out" / "frames"))
    entries = tomllib.loads(set_path.read_text())["sequence"]
    if "--only" in opts:
        entries = [e for e in entries if e["id"] == opts["--only"]]
    h3view = str(ROOT / "clients" / "target" / "release" / "h3-view")
    failed = 0
    for e in entries:
        # A sequence without engine commands (`hover`) is the opening's battle with the mouse
        # over it (M3 T6c-2: the viewer plays it too).
        here = out / e["id"]
        here.mkdir(parents=True, exist_ok=True)
        if not (reuse and (here / "frames.jsonl").exists()):
            map_path = here / f"{e['id']}.h3m"
            subprocess.run(situation_args(h3view, set_path, e, map_path), check=True,
                           capture_output=True)
            game = Game(here)
            try:
                sequence(game, map_path, here, e.get("pre", 0), e.get("actions", []), e["ms"],
                         frames=True, hot_seat=e.get("hot_seat", False))
            finally:
                game.stop()
        kept = [json.loads(line) for line in (here / "frames.jsonl").read_text().splitlines()]
        # A screen the actions replaced within its millisecond (a wait or a defend passes the
        # turn at once: `t<ms>b`) is compared, not the one before it: the viewer draws one
        # moment per millisecond (M3 T1a).
        kept = [k for i, k in enumerate(kept) if i + 1 == len(kept) or kept[i + 1]["t"] != k["t"]]
        (here / "frames.toml").write_text(frames_toml(kept))
        keep(set_path, "sequences", e["id"], here / "frames.toml")
        subprocess.run([h3view, "sequences", e["id"], "--frames", str(out), "--out", str(here)],
                       check=True, capture_output=True)
        observation = h3cap.snapshot(here / f"{e['id']}.jsonl")
        # The poses the viewer drew (its `ours.txt`): the count boxes are masked where they show.
        drawn, boxed = {}, {}
        for line in (here / "ours.txt").read_text().splitlines():
            poses = re.findall(r"(\d+):g(\d+)f\d+h(\d+)(m?)", line)
            drawn[int(line.split()[0])] = {int(i): (int(g), int(h), m == "m")
                                           for i, g, h, m in poses}
            # The count boxes it drew (E17 M3 T5a: hidden through walks and flights, placed by the
            # stacks holding their hexes then), each (x0, y0, x1, y1), its count and its stack's
            # `<side>.<record>` (T5b).
            boxed[int(line.split()[0])] = [
                ((int(x), int(y), int(x) + h3cap.COUNT_BOX[0], int(y) + h3cap.COUNT_BOX[1]),
                 int(n), record)
                for x, y, n, record in re.findall(r" box(-?\d+),(-?\d+)n(\d+)@?(\d+\.\d+)?",
                                                  line)]
        # The log item's text the viewer drew into each frame (E17 M3 T6b), by the frame's name.
        ours_log = here / "ours-log.json"
        texts = json.loads(ours_log.read_text()) if ours_log.exists() else {}
        shutil.rmtree(here / "diff", ignore_errors=True)
        (here / "diff").mkdir()
        # Each line `<ms> <differing> <redraw artifacts among them>` (M3 T1c, h3cap.redraws).
        lines, differing, only_redraws, earlier = [], 0, 0, []
        for k in kept:
            stem = Path(k["frame"]).stem
            cap = np.load(here / "frames" / k["frame"])["screen"]
            ours = h3cap.png565(here / "ours" / f"{stem}.png")
            masked = h3cap.mask(compared_masks("sequences", e, k["t"]), shape=cap.shape)
            # The count boxes masked where the drawn poses show them and where the viewer drew
            # them, unless the entry compares them (`boxes`, E17 M3 T5a): then only those whose
            # count is not the one the original shows (its memory: +0x50 while it is not -1,
            # else +0x4C; design §4.3), the engine's damage being rolled otherwise; listed
            # (` n<side>.<record>`, T5b).
            if e.get("boxes"):
                theirs = {f"{st['side']}.{st['record']}":
                          st["count"] if st.get("shown", -1) == -1 else st["shown"]
                          for st in k["stacks"]}
                other = [(box, record) for box, n, record in boxed[k["t"]]
                         if theirs.get(record) != n]
                boxes, listed = [box for box, _ in other], [record for _, record in other]
            else:
                boxes = (h3cap.boxes_at(observation, drawn[k["t"]])
                         + [box for box, _, _ in boxed[k["t"]]])
                listed = []
            # The log's item (E17 M3 T6b): compared where the original's shows the text the
            # viewer drew (its memory: the item's string), else, where its pixels differ, masked
            # and listed (` log`): a status line's hint (T6c) or a line whose numbers the engine
            # rolled otherwise. (The memory can lag the screen by a step as a turn starts: the
            # pixels then agree, and nothing is listed.)
            theirs_text = (k.get("log") or {}).get("text")
            x, y, w, h = h3cap.log_item()
            log_listed = ((texts.get(stem) is None or texts.get(stem) != theirs_text)
                          and bool((cap[y:y + h, x:x + w] != ours[y:y + h, x:x + w]).any()))
            if log_listed:
                boxes = boxes + [(x, y, x + w, y + h)]
            for x0, y0, x1, y1 in boxes:
                masked[max(y0, 0):max(y1, 0), max(x0, 0):max(x1, 0)] = True
            differ = (cap != ours) & ~masked
            redrawn = h3cap.redraws(cap, ours, differ, earlier)
            n, a = int(differ.sum()), int(redrawn.sum())
            if n:
                h3cap.compare(cap, ours, masked, here / "diff" / f"{stem}.png", redrawn)
                differing += 1
                only_redraws += a == n
            lines.append(f"{k['t']} {n} {a}" + "".join(f" n{r}" for r in listed)
                         + (" log" if log_listed else ""))
            earlier.append(ours)
        got = "\n".join(lines) + "\n"
        (here / "frames.txt").write_text(got)
        recorded = set_path.parent / e.get("frames", f"timelines/{e['id']}-frames.txt")
        if not recorded.exists():
            verdict = "new"
        else:
            verdict = "ok" if got == recorded.read_text() else "CHANGED"
        failed += verdict != "ok"
        print(f"{e['id']}: {len(kept)} frames, {differing} differ outside the masks "
              f"({only_redraws} only by redraw artifacts) {verdict}", flush=True)
    print(f"frames: {len(entries)} sequences, {failed} not as recorded; under {out}")
    return 1 if failed else 0


def compared_masks(kind, entry, t):
    """The fixed masks a comparison applies to a frame at `t` ms: a sequence's none (its panel
    compared since E17 M3 T6b, its cursor since T6c-2), a situation's its `mask` list. (A
    sequence's count boxes and its log's item are masked besides, by the original's counts and
    text: not fixed.)"""
    if kind == "sequences":
        return []
    return entry.get("mask", [])


def png_digest(path, boxes):
    """The digest of a viewer's PNG (8-bit RGB, unfiltered rows: h3_render::png) with the pixels in
    `boxes` (each x0, y0, x1, y1) blanked: what a comparison with those masks sees (E17 M3 T6a)."""
    import hashlib
    import zlib

    data = Path(path).read_bytes()
    at, idat, width, height = 8, b"", 0, 0
    while at < len(data):
        length, kind = struct.unpack(">I4s", data[at:at + 8])
        body = data[at + 8:at + 8 + length]
        if kind == b"IHDR":
            width, height = struct.unpack(">II", body[:8])
        elif kind == b"IDAT":
            idat += body
        at += 12 + length
    raw = bytearray(zlib.decompress(idat))
    stride = 1 + 3 * width
    if any(raw[y * stride] for y in range(height)):
        sys.exit(f"{path}: a filtered row (not h3_render::png's)")
    for x0, y0, x1, y1 in boxes:
        x0, x1 = max(x0, 0), min(x1, width)
        for y in range(max(y0, 0), min(y1, height)):
            if x1 > x0:
                raw[y * stride + 1 + 3 * x0:y * stride + 1 + 3 * x1] = bytes(3 * (x1 - x0))
    return hashlib.sha1(bytes(raw)).hexdigest()[:16]


def keep(set_path, kind, ident, frames):
    """Keeps a capture's `frames.toml` (its frames' times, the fidgets and heroes the original's
    memory showed: numbers only, ADR-0009) in `situations/kept/<kind>/<id>.toml`, beside the set
    it ran from, so that `drawn` can draw the viewer's frames without the original."""
    kept = set_path.parent / "kept" / kind / f"{ident}.toml"
    kept.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy(frames, kept)


def run_drawn(rest):
    """The viewer's frames without the original (runbook, "Which captures a change needs"): for
    each recorded sequence and each situation of the capture set, the
    viewer draws its frames at the times the harness last kept frames of the original
    (`situations/kept/`, written by `frames` and `set`); each frame's digest (of its pixels
    outside the fixed masks its comparison applies, `compared_masks`: a change only under them
    leaves the recorded count true; E17 M3 T6a) is compared with `situations/kept/drawn.txt`. One
    line per sequence or situation (`same`, `CHANGED (<n> of <m> frames, the first at <ms>)`,
    `new`, or the viewer's own timeline check failing); exits 1 unless every one is the same.
    With `--record` it writes the digests instead (those not run kept). Needs the release
    `h3-view` and the user's original files only."""
    import h3cap

    situations = ROOT / "clients" / "situations"
    kept_dir = situations / "kept"
    record = "--record" in rest
    rest = [r for r in rest if r != "--record"]
    opts = dict(zip(rest[::2], rest[1::2]))
    out = Path(opts.get("--out", ROOT / "clients" / "out" / "drawn"))
    h3view = str(ROOT / "clients" / "target" / "release" / "h3-view")
    items = [("sequences", e["id"], e)
             for e in tomllib.loads((situations / "sequences.toml").read_text())["sequence"]]
    items += [("set", s["id"], s)
              for s in tomllib.loads((situations / "set.toml").read_text())["situation"]]
    if "--only" in opts:
        items = [i for i in items if i[1] == opts["--only"]]
    digests_path = kept_dir / "drawn.txt"
    recorded = {}
    if digests_path.exists():
        for line in digests_path.read_text().splitlines():
            if line and not line.startswith("#"):
                name, stem, digest = line.split()
                recorded.setdefault(name, {})[stem] = digest
    got, failed = {}, 0
    for kind, ident, entry in items:
        name = f"{kind}/{ident}"
        kept = kept_dir / kind / f"{ident}.toml"
        if not kept.exists():
            command = "frames" if kind == "sequences" else "set"
            print(f"{name}: no kept frames (capture it: `{command} --only {ident}`)", flush=True)
            failed += 1
            continue
        here = out / kind / ident
        shutil.rmtree(here, ignore_errors=True)
        here.mkdir(parents=True)
        shutil.copy(kept, here / "frames.toml")
        if kind == "sequences":
            args = [h3view, "sequences", ident, "--frames", str(out / kind), "--out", str(here)]
        else:
            args = [h3view, "render", str(situations / entry["scenario"]), "--out",
                    str(here / "ours.png"), "--frames", str(here)]
        run_result = subprocess.run(args, capture_output=True, text=True)
        if run_result.returncode != 0:
            last = (run_result.stdout + run_result.stderr).strip().splitlines()[-1:]
            print(f"{name}: the viewer failed ({' '.join(last)})", flush=True)
            failed += 1
            continue
        listed = tomllib.loads(kept.read_text())["frame"]
        frames = [Path(f["file"]).stem for f in listed]
        got[name] = {Path(f["file"]).stem: png_digest(
            here / "ours" / f"{Path(f['file']).stem}.png",
            h3cap.mask_boxes(compared_masks(kind, entry, f["t"])))
            for f in listed}
        before = recorded.get(name)
        if before is None:
            verdict = "new"
        elif before == got[name]:
            verdict = "same"
        else:
            differ = [stem for stem in frames if before.get(stem) != got[name][stem]]
            gone = [stem for stem in before if stem not in got[name]]
            first = (differ or gone)[0].lstrip("t").rstrip("b").lstrip("0") or "0"
            verdict = (f"CHANGED ({len(differ) + len(gone)} of {len(frames)} frames, the first "
                       f"at {first})")
        failed += verdict != "same"
        print(f"{name}: {len(frames)} frames {verdict}", flush=True)
    if record:
        recorded.update(got)
        lines = ["# The viewer's frames at the times last kept (h3drive.py drawn --record): each",
                 "# frame's digest outside its comparison's fixed masks (h3drive.py png_digest),",
                 "# `<kind>/<id> <frame> <sha1, 16 digits>`."]
        lines += [f"{name} {stem} {digest}" for name in sorted(recorded)
                  for stem, digest in recorded[name].items()]
        digests_path.write_text("\n".join(lines) + "\n")
        print(f"drawn: {len(got)} recorded in {digests_path}")
        return 0
    print(f"drawn: {len(items)} sequences and situations, {failed} not the same; under {out}")
    return 1 if failed else 0


if __name__ == "__main__":
    main(sys.argv[1:])
