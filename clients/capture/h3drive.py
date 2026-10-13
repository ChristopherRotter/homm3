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

Library use: `Game` (start, wait, click, key, clock, settled, step_until, dump, stop), the
recipes (`to_scenario`, `into_battle`) and `capture`. Needs numpy, xdotool, xwd, Xvfb and Wine.
"""

import os
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
    return run("xwd", "-root", "-silent", capture_output=True).stdout


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
        """Steps the frozen clock by `step_ms` until the screen `name` shows; returns its time."""
        t = None
        for _ in range(limit_ms // step_ms):
            t = self.clock(f"step {step_ms}")
            if self.matches(name, self.settled()):
                return t
        bad = self.out / f"not-{name}.xwd"
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


def into_battle(game, key="Right"):
    """From the adventure map: freezes the clock, moves the selected hero with `key` into the
    enemy, and steps the clock to the battle screen. Returns the clock's time at the opening."""
    game.clock("freeze")
    game.key(key)
    game.move(420, 580)  # over the panel, off the battlefield
    return game.step_until("battle", 10)


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
    ]
    for name in sorted(screens):
        lines.append(f"\n[{name}]\nprobes = [")
        lines += [f"    [{x}, {y}, {v}]," for x, y, v in screens[name]["probes"]]
        lines.append("]")
    SCREENS.write_text("\n".join(lines) + "\n")


def capture(game, map_path, out, frames, key="Right"):
    """The battle of the map: its opening, then a dump at each time in `frames` (ms after the
    opening, ascending); returns the dumps' paths."""
    game.start(map_path)
    to_scenario(game)
    t0 = into_battle(game, key)
    now, paths = t0, []
    for ms in sorted(frames):
        if ms > now - t0:
            now = game.clock(f"step {ms - (now - t0)}")
        game.settled()
        path = Path(out) / f"t{ms:05d}.xwd"
        game.dump(path)
        paths.append(path)
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
        subprocess.run([h3view, "situation", str(scenario), "--out", str(map_path), *tile],
                       check=True, capture_output=True)
        ours = here / "ours.png"
        subprocess.run([h3view, "render", str(scenario), "--out", str(ours)], check=True,
                       capture_output=True)
        frames = {f["at"]: f for f in sit.get("frame", [])} or {0: {"at": 0}}
        game = Game(here)
        try:
            paths = capture(game, map_path, here, list(frames))
        finally:
            game.stop()
        for (at, frame), path in zip(sorted(frames.items()), paths):
            # The pixels' digest: an XWD file's colour map holds bytes the X server leaves unset.
            pixels = xwd565(path.read_bytes()).tobytes()
            digests.append(f"{hashlib.sha1(pixels).hexdigest()}  {sid}/{path.name}")
            n = h3cap.diff(path, ours, ours.with_suffix(".jsonl"), sit.get("mask", []),
                           here / f"diff-t{at:05d}.png", quiet=True)
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


if __name__ == "__main__":
    main(sys.argv[1:])
