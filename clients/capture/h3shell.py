#!/usr/bin/env python3
"""The game shell's reference captures (E27 M1 T2, *provisional*; the runbook, "The shell's
screens"): the original's screens outside play, reached by recipes on a clock frozen from the
game's first read, each recipe recorded twice and the two compared.

  h3shell.py run [--group <g>[,<g>...]] [--only <id>[,<id>...]] [--once] [--record] [--out <dir>]
      Plays each recipe of shell/<group>.toml (every group without --group) twice from a fresh
      start, compares each shot and clip of the second run with the first's (pixels and times)
      and both with shell/kept/<group>.txt: `ok` (as kept), `CHANGED`, `new`; a shot whose two
      runs differ says where (`differs`, its box and pixel count). With --record the kept lines
      of the recipes run are written (a shot that differs keeps both digests, `differs`). --once
      plays one run (no comparison of runs; never --record). The frames stay in <out>
      (default clients/out/shell; local, never in git): <id>/<run>/<shot>.npz, a clip's
      <name>/t<ms>.npz and <name>.txt.
  h3shell.py play <id> [--keep] [--out <dir>]
      One run of a recipe, its shots dumped; with --keep the game is left running at the end
      (to learn a screen's probes, `learn`; `wineserver -k` ends it).
  h3shell.py learn <group> <screen> [--in x0,y0,x1,y1] [--avoid x0,y0,x1,y1 ...]
      The display's probes for a screen, into shell/<group>.screens.toml (h3drive.py learn's
      grid; the clock may be frozen: the probes are the points that kept their value).
  h3shell.py list    The recipes and their shots.

A recipe (a `[[recipe]]` of shell/<group>.toml): `id`, `what` (the catalogue's section), and
  registry  dwords of the settings key for this run, over setup.sh's (`Show Intro`, …)
  maps      the run's Maps/: "all" (the default: original-data/install/Maps), or a list of names
  games     the run's Games/: files of original-data/games (default none)
  frozen    false: the clock runs in real time until a step freezes it (default true)
  steps     the steps, in order (`STEPS`)
Every run starts from setup.sh's registry (the settings key removed and written again: what a
run saved never reaches the next), no HiScore.dat, the listed maps and saves.
"""

import hashlib
import os
import re
import shutil
import subprocess
import sys
import time
import tomllib
from pathlib import Path

import numpy as np

import h3drive

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent.parent
SHELL = HERE / "shell"
KEPT = SHELL / "kept"
OUT = ROOT / "clients" / "out" / "shell"
DATA = ROOT / "original-data"
# The clock's time at the game's first read (xdd.dll's H3FROZEN): any value; times are counted
# from it.
FROZEN_AT = int(os.environ.get("H3SHELL_FROZEN", 1_000_000))
# The shell's settings over setup.sh's, before a recipe's own: not a first run (no `Testing`
# clip, which the harness's VIDEO.VID lacks: its counts would set `Bink Video` 0), and the Bink
# copies of the start's clips, as a computer that passed the first run's test plays them (design
# §2.1; a recipe sets 0 for the Smacker copies, the system options' Low quality, CLT-Q96).
BASE_SETTINGS = {"First Time": 0, "Bink Video": 1}
# The pointer's place at a run's start (the display keeps the last run's otherwise).
PARK = (400, 300)
# The game waiting on the clock: IDLE_READS reads since a command and more IDLE_WINDOW seconds
# later (`Run.busy_wait`).
IDLE_READS = 20
IDLE_WINDOW = 0.02
# The clock's step after each input (a recipe's `input_step`), so the cursor is drawn where the
# pointer is (`Run.after_input`).
INPUT_STEP = 100
# How long an input is given to reach the game (Wine's X events come in its message loop) before
# the game is waited for (`Run.after_input`).
INPUT_SETTLE = 0.15
# A step of the frozen clock while waiting for a screen (`until`) and a clip's default step.
UNTIL_STEP = 10
UNTIL_LIMIT = 60_000
STEPS = """The steps (strings `<verb>[:<arguments>]`):
  until:<screen>[|<screen>...][:<limit ms>]   step the clock 10 ms at a time until a screen shows
  check:<screen>         fail unless the screen shows now
  step:<ms>[:<by>]       the clock moves on by <ms>, <by> ms at a time when given (a clip
                         shows at most one frame a step: its frames are due from the last one)
  freeze                 a real-time run's clock stops (`frozen = false`)
  move:<x>,<y>           the pointer moved (no press)
  click:<x>,<y>          a left press and release there
  down:<x>,<y> | up      the left button pressed there and held | released
  right:<x>,<y> | right-up   the right button pressed there and held | released
  key:<name> | key-down:<name> | key-up:<name>   a key (xdotool's names) pressed, held, let go
  type:<text>            the text typed
  press:<button>[:<k>] | point:<button>[:<k>]   a dialog's button found by its probes ([button.
                         <name>], screens.toml: ok, cancel, …; the k-th from the top left)
                         clicked, or only pointed at
  until-button:<button>[:<limit ms>]   step the clock until the button shows
  games-time:<s>         every file of the run's Games/ modified at <s> (seconds since 1970)
  click-wait:<x>,<y>:<ms>   a click, then <ms> of real time before the input step (a load)
  wait:<s>               <s> seconds of real time, the clock where it is, then idle (a load)
  settle:<s>             the next input's settle: <s> seconds of real time (not 0.15) before the
                         game is waited for and the clock steps (an input that starts a load)
  poke:<address>:<hex>   bytes written into the running game's memory (h3mem), the address a
                         sum of `game` (the game record) and numbers: only to bring about what
                         the original then shows itself (an end's condition met, a setting)
  shot:<name>            the screen kept once the game is idle and the display settled
  clip:<name>:<ms>[:<step>]   every changed screen kept for <ms>, the clock stepped by <step>
                         (default 10): `<name>/t<ms>.npz`, each with its time
  clip-until:<name>:<screen>[:<step>][:<limit ms>]   the same until the screen shows (kept)
After each input the clock moves on by the recipe's `input_step` (default 100 ms: the game
draws its cursor at the pointer only on a later tick) and the game is idle before the next step."""


def digest(pix):
    return hashlib.sha256(np.ascontiguousarray(pix, dtype="<u2").tobytes()).hexdigest()[:16]


def groups():
    return sorted(p.stem for p in SHELL.glob("*.toml") if not p.stem.endswith(".screens"))


def load_group(group):
    data = tomllib.loads((SHELL / f"{group}.toml").read_text())
    for recipe in data.get("recipe", []):
        recipe["group"] = group
    return data.get("recipe", [])


def all_recipes():
    recipes = [r for g in groups() for r in load_group(g)]
    seen = set()
    for r in recipes:
        if r["id"] in seen:
            raise SystemExit(f"the recipe id `{r['id']}` twice")
        seen.add(r["id"])
    return recipes


def screens():
    """h3drive's screens and every group's own (shell/<group>.screens.toml); a name twice is
    refused."""
    found = h3drive.load_screens()
    for path in sorted(SHELL.glob("*.screens.toml")):
        for name, entry in tomllib.loads(path.read_text()).items():
            if name in found and name != "button":
                raise SystemExit(f"the screen `{name}` of {path.name} is h3drive's or another's")
            if name == "button":
                found.setdefault("button", {}).update(entry)
            else:
                found[name] = entry
    return found


# -- a run's start ---------------------------------------------------------------------------
def fill(folder, sources):
    folder.mkdir(parents=True, exist_ok=True)
    for old in folder.iterdir():
        if old.is_file():
            old.unlink()
    for src in sources:
        shutil.copy(src, folder / src.name)


def registry_file(values):
    """setup.sh's settings (its h3.reg) with the settings key removed first, then the recipe's
    dwords: a run starts from setup.sh's settings whatever an earlier run saved."""
    base = (h3drive.GAME_DIR.parent / "h3.reg").read_bytes().decode("cp1252")
    key = h3drive.SETTINGS_KEY
    head, rest = base.split("\r\n\r\n", 1)
    lines = [f"[-{key}]", ""]
    values = {**BASE_SETTINGS, **values}
    extra = "".join(f'"{n}"=dword:{int(v):08x}\r\n' for n, v in values.items())
    # The recipe's values last in the key's section, after setup.sh's (the later line wins).
    at = rest.index(f"[{key}]\r\n")
    end = rest.index("\r\n\r\n", at) + 2
    rest = rest[:end] + extra + rest[end:]
    return (head + "\r\n\r\n" + "\r\n".join(lines) + "\r\n" + rest).encode("cp1252")


def prepare(recipe):
    game = h3drive.GAME_DIR
    maps = recipe.get("maps", "all")
    shipped = DATA / "install" / "Maps"
    if maps == "all":
        chosen = sorted(p for p in shipped.iterdir() if p.is_file())
    else:
        chosen = [shipped / m for m in maps]
    missing = [str(p) for p in chosen if not p.is_file()]
    if missing:
        raise RuntimeError(f"{recipe['id']}: maps not staged: {missing}")
    fill(game / "Maps", chosen)
    saves = [DATA / "games" / g for g in recipe.get("games", [])]
    missing = [str(p) for p in saves if not p.is_file()]
    if missing:
        raise RuntimeError(f"{recipe['id']}: saved games not staged: {missing}")
    fill(game / "Games", saves)
    for old in (game / "HiScore.dat", game / "Data" / "HiScore.dat"):
        if old.exists():
            old.unlink()
    reg = game.parent / "shell.reg"
    reg.write_bytes(registry_file(recipe.get("registry", {})))
    subprocess.run(["wineserver", "-k"], env=h3drive.env(), check=False)
    subprocess.run(["wine", "regedit", str(reg)], env=h3drive.env(), check=True,
                   stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    try:
        subprocess.run(["wineserver", "-w"], env=h3drive.env(), check=False,
                       timeout=h3drive.WINESERVER_WAIT)
    except subprocess.TimeoutExpired:
        subprocess.run(["wineserver", "-k"], env=h3drive.env(), check=False)
    for name, value in recipe.get("registry", {}).items():
        held = h3drive.setting_dword(name)
        if held != int(value):
            raise RuntimeError(f"{recipe['id']}: the registry holds {name} = {held}, not {value}")


def display():
    """The instance's Xvfb started (as `Game.start` does), so the pointer can be parked before
    the game starts."""
    xvfb = f"(^|/)Xvfb {re.escape(h3drive.DISPLAY)}( |$)"
    if subprocess.run(["pgrep", "-f", xvfb], capture_output=True).returncode:
        subprocess.Popen(["Xvfb", h3drive.DISPLAY, "-screen", "0",
                          f"{h3drive.WIDTH}x{h3drive.HEIGHT}x16", "-nolisten", "tcp"],
                         stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        time.sleep(1)


# -- the steps ---------------------------------------------------------------------------------
class Run:
    def __init__(self, recipe, out):
        self.recipe = recipe
        self.out = Path(out)
        if self.out.exists():
            shutil.rmtree(self.out)
        self.out.mkdir(parents=True)
        frozen = FROZEN_AT if recipe.get("frozen", True) else None
        self.game = h3drive.Game(self.out, frozen=frozen)
        self.game.screens = screens()
        self.kept = []  # (name, kind, time, digest or (count, first, last, digest))
        self.input_step = int(recipe.get("input_step", INPUT_STEP))
        # The next input's settle in seconds (`settle`; INPUT_SETTLE when None).
        self.settle = None

    def now(self):
        a = self.game.ack(need=True)
        return a[1] - FROZEN_AT

    def reads(self):
        """The clock's reads since the last command: the game's and the clips' (xdd.dll's 5.)."""
        a = self.game.ack(need=True)
        return a[2] + a[11]

    def busy_wait(self, timeout=60.0):
        """Waits until the game waits on the clock: IDLE_READS reads since the last command (the
        game's or, in a full-screen Bink clip, BinkWait's: the game's loop reads none there), and
        more of them IDLE_WINDOW later. Loading reads the clock a few times a second, a waiting
        loop hundreds to some 200 000 times: a dump taken while the game is still busy shows an
        unchanged screen too, and a step given then moved the time a screen shows by a step (the
        main menu at 10 ms in one run, 20 in the next)."""
        end = time.monotonic() + timeout
        while self.game.ack() is None:
            if time.monotonic() > end:
                raise TimeoutError(f"no acknowledgement from xdd.dll{self.game.crash_note()}")
            time.sleep(0.01)
        while time.monotonic() < end:
            seen = self.reads()
            if seen >= IDLE_READS:
                time.sleep(IDLE_WINDOW)
                if self.reads() > seen:
                    return
            else:
                time.sleep(0.002)
        raise TimeoutError(f"the game did not wait on the clock within {timeout} s (ack "
                           f"{self.game.ack()}){self.game.crash_note()}")

    def start_drawn(self, quiet=1.0, timeout=30.0):
        """The first screen drawn at the frozen time before the recipe's first step: the display
        unchanged for `quiet` seconds of real time (the main menu shows at 0 ms; a first step
        given before it was drawn showed it at 10 ms in one run of two, `menu-tutorial`)."""
        end = time.monotonic() + timeout
        last = self.game.settled()
        since = time.monotonic()
        while time.monotonic() < end:
            time.sleep(0.1)
            pix = self.game.settled()
            if not np.array_equal(pix, last):
                last, since = pix, time.monotonic()
            elif time.monotonic() - since >= quiet:
                return
        raise TimeoutError(f"the first screen kept changing for {timeout} s")

    def tick(self, command, timeout=30.0):
        """A clock command applied, then the game waiting on the clock again (`busy_wait`)."""
        g = self.game
        g.seq += 1
        tmp = g.clock_dir / "ctl.tmp"
        tmp.write_text(f"{g.seq} {command}\n")
        tmp.replace(g.clock_dir / "ctl")
        end = time.monotonic() + timeout
        while (g.ack() or (None,))[0] != g.seq:
            if time.monotonic() > end:
                raise TimeoutError(f"xdd.dll did not take `{command}`")
            time.sleep(0.002)
        self.busy_wait(timeout)

    def idle(self):
        """The game idle at the clock's time."""
        self.tick("step 0")

    def after_input(self):
        """The clock moved on by the recipe's input step once an input was given: the game draws
        its cursor at the pointer only on a later tick (a move shows no cursor until the clock
        moves on some 50 ms; the buttons' hover at once). Before the step the game takes the
        input and is idle again (INPUT_SETTLE, then `idle`): a step given while the game had not
        yet taken a release that starts a load moved the load's frames by 10 ms in one run of
        two (`menu-tutorial`, the machine busy). A `settle` step sets the next input's settle
        (E27 M1 T2: a map's load started by the input)."""
        time.sleep(INPUT_SETTLE if self.settle is None else self.settle)
        self.settle = None
        self.idle()
        self.tick(f"step {self.input_step}")

    def buttons(self, name):
        """The top-left corners of every place the button `name` shows, top to bottom, left to
        right (h3adv.find_button's search)."""
        probes = self.game.screens["button"][name]["probes"]
        pix = self.game.settled()
        dx0, dy0, v0 = probes[0]
        ys, xs = np.nonzero(pix == v0)
        h, w = pix.shape
        found = []
        for y, x in zip(ys - dy0, xs - dx0):
            if all(0 <= y + dy < h and 0 <= x + dx < w and pix[y + dy, x + dx] == v
                   for dx, dy, v in probes):
                found.append((int(y), int(x)))
        return [(x, y) for y, x in sorted(found)]

    def button(self, name, k):
        found = self.buttons(name)
        if len(found) <= k:
            raise RuntimeError(f"the button `{name}` shows {len(found)} times, not {k + 1}")
        w, h = self.game.screens["button"][name]["size"]
        x, y = found[k]
        return x + w // 2, y + h // 2

    def until(self, names, limit):
        names = tuple(names.split("|"))
        for n in names:
            if n not in self.game.screens:
                raise RuntimeError(f"no probes for the screen `{n}`")
        for _ in range(limit // UNTIL_STEP + 1):
            pix = self.game.settled()
            if any(self.game.matches(n, pix) for n in names):
                return
            self.tick(f"step {UNTIL_STEP}")
        self.game.dump(self.out / f"not-{names[0]}.xwd")
        raise TimeoutError(f"the screen `{'|'.join(names)}` did not show within {limit} ms (the "
                           f"display: not-{names[0]}.xwd){self.game.crash_note()}")

    def shot(self, name):
        self.idle()
        pix = self.game.settled()
        np.savez_compressed(self.out / f"{name}.npz", pix=pix)
        self.kept.append((name, "shot", self.now(), digest(pix)))

    def clip(self, name, ms=None, screen=None, step=UNTIL_STEP, limit=UNTIL_LIMIT):
        here = self.out / name
        here.mkdir()
        self.idle()
        last = self.game.settled()
        start = self.now()
        lines = [(start, digest(last))]
        np.savez_compressed(here / f"t{start}.npz", pix=last)
        end = start + (ms if ms is not None else limit)
        while self.now() < end:
            self.tick(f"step {step}")
            pix = self.game.settled()
            if not np.array_equal(pix, last):
                t = self.now()
                lines.append((t, digest(pix)))
                np.savez_compressed(here / f"t{t}.npz", pix=pix)
                last = pix
            if screen and any(self.game.matches(n, pix) for n in screen.split("|")):
                break
        else:
            if screen:
                raise TimeoutError(f"{name}: the screen `{screen}` within {limit} ms")
        (here.parent / f"{name}.txt").write_text("".join(f"{t} {d}\n" for t, d in lines))
        whole = hashlib.sha256("".join(f"{t} {d}\n" for t, d in lines).encode()).hexdigest()[:16]
        self.kept.append((name, "clip", start, (len(lines), lines[0][0], lines[-1][0], whole)))

    def do(self, step):
        verb, _, arg = step.partition(":")
        g = self.game
        if verb == "until":
            names, _, limit = arg.partition(":")
            self.until(names, int(limit or UNTIL_LIMIT))
        elif verb == "check":
            if not g.matches(arg, g.settled()):
                g.dump(self.out / f"not-{arg}.xwd")
                raise RuntimeError(f"not the screen `{arg}` (the display: not-{arg}.xwd)")
        elif verb == "step":
            # By <by> ms at a time when given: a clip shows at most one frame a step (its frames
            # are due from the last one shown), so a long jump would leave it at its start.
            ms, _, by = arg.partition(":")
            ms, by = int(ms), int(by or ms)
            while ms > 0:
                self.tick(f"step {min(by, ms)}")
                ms -= by
        elif verb == "freeze":
            self.tick("freeze")
        elif verb == "move":
            g.move(*map(int, arg.split(",")))
            self.after_input()
        elif verb == "click":
            g.click(*map(int, arg.split(",")), settle=0.1)
            self.after_input()
        elif verb == "click-wait":
            # A click whose work needs no clock (a saved game's load: its bar fills in real time,
            # the clock frozen), waited for <ms> in real time before the input step: busy, the
            # load's reads of the clock passed for the idle loop and the step came amid it (E27
            # M1 T2, `scn-load`).
            point, _, ms = arg.partition(":")
            g.click(*map(int, point.split(",")), settle=0.1)
            time.sleep(int(ms) / 1000)
            self.after_input()
        elif verb == "down":
            h3drive.run("xdotool", "mousemove", *arg.split(","))
            h3drive.run("xdotool", "mousedown", "1")
            g.left_held = True
            self.after_input()
        elif verb == "up":
            h3drive.run("xdotool", "mouseup", "1")
            g.left_held = False
            self.after_input()
        elif verb == "right":
            h3drive.run("xdotool", "mousemove", *arg.split(","))
            h3drive.run("xdotool", "mousedown", "3")
            g.right_held = True
            self.after_input()
        elif verb == "right-up":
            h3drive.run("xdotool", "mouseup", "3")
            g.right_held = False
            self.after_input()
        elif verb == "key":
            g.key(arg, settle=0.1)
            self.after_input()
        elif verb == "key-down":
            h3drive.run("xdotool", "keydown", arg)
            g.keys_held.add(arg)
            self.after_input()
        elif verb == "key-up":
            h3drive.run("xdotool", "keyup", arg)
            g.keys_held.discard(arg)
            self.after_input()
        elif verb == "type":
            h3drive.run("xdotool", "type", "--delay", "50", arg)
            self.after_input()
        elif verb in ("press", "point"):
            # A dialog's button found by its probes ([button.<name>]; the k-th from the top left,
            # default the first): pressed (a click on its middle), or only pointed at.
            name, _, k = arg.partition(":")
            x, y = self.button(name, int(k or 0))
            if verb == "press":
                g.click(x, y, settle=0.1)
            else:
                g.move(x, y)
            self.after_input()
        elif verb == "until-button":
            name, _, limit = arg.partition(":")
            for _ in range(int(limit or UNTIL_LIMIT) // UNTIL_STEP):
                if self.buttons(name):
                    break
                self.tick(f"step {UNTIL_STEP}")
            else:
                raise TimeoutError(f"the button `{name}` did not show")
        elif verb == "games-time":
            # Every file of the run's Games/ given one modification time (seconds since 1970):
            # the scenario screen's load and save modes show a save's time (design §3.5), and
            # `fill`'s copies carry the real time of the run (E27 M1 T2, the scenario group).
            for path in (h3drive.GAME_DIR / "Games").iterdir():
                if path.is_file():
                    os.utime(path, (int(arg), int(arg)))
        elif verb == "settle":
            # The next input's settle (E27 M1 T2): an input that starts a map's load (Begin) is
            # taken while the game loads; the input's own idle test and step could come during
            # the load on a busy machine (`wait`'s case), so the load is given time to end first.
            self.settle = float(arg)
        elif verb == "wait":
            # Real time passes with the clock where it is, then the game idle (E27 M1 T2): a map's
            # load reads the clock only now and then, so on a busy machine `busy_wait` can take
            # it for idle while it still loads and `until` steps the clock meanwhile (the map then
            # shows 10 to 100 ms later in one run of two: the options' and the end's recipes); a
            # wait after the press that starts the load lets it end first.
            time.sleep(float(arg))
            self.idle()
        elif verb == "poke":
            # Bytes written into the running game (h3mem's write_bytes; E27 M1 T2, the end
            # group): `<address>:<hex bytes>`, the address a sum of terms, `game` the game record
            # (the pointer at h3mem.GAME_POINTER), the others numbers (0x… for hex). Only to
            # bring about a state whose screens the original then shows by its own paths (a
            # scenario's condition met, a player's towns gone, a setting the harness overrides),
            # never to draw a screen; the game idle again after it.
            import h3mem
            where, _, data = arg.partition(":")
            mem = h3mem.Mem()
            try:
                address = sum(mem.game() if t == "game" else int(t, 0) for t in where.split("+"))
                mem.write_bytes(address, bytes.fromhex(data))
            finally:
                mem.file.close()
            self.idle()
        elif verb == "shot":
            self.shot(arg)
        elif verb == "clip":
            parts = arg.split(":")
            self.clip(parts[0], ms=int(parts[1]), step=int(parts[2]) if len(parts) > 2 else 10)
        elif verb == "clip-until":
            parts = arg.split(":")
            self.clip(parts[0], screen=parts[1],
                      step=int(parts[2]) if len(parts) > 2 and parts[2] else UNTIL_STEP,
                      limit=int(parts[3]) if len(parts) > 3 else UNTIL_LIMIT)
        else:
            raise ValueError(f"{self.recipe['id']}: an unknown step `{step}`\n{STEPS}")

    def play(self, keep=False):
        prepare(self.recipe)
        display()
        h3drive.run("xdotool", "mousemove", *map(str, PARK))
        self.game.start(None)
        try:
            self.busy_wait()
            self.start_drawn()
            for step in self.recipe["steps"]:
                self.do(step)
        except Exception:
            try:
                self.game.dump(self.out / "failed.xwd")
            except Exception:  # noqa: BLE001 (the display may be gone)
                pass
            raise
        finally:
            if not keep:
                self.game.stop()
        return self.kept


# -- comparing and keeping ----------------------------------------------------------------------
def kept_line(ident, entry, other=None):
    name, kind, t, d = entry
    if kind == "shot":
        line = f"{ident} {name} t{t} {d}"
    else:
        n, first, last, whole = d
        line = f"{ident} {name} clip {n}@{first}-{last} {whole}"
    if other is not None and other != entry:
        line += " differs " + kept_line(ident, other).split(" ", 2)[2]
    return line


def read_kept(group):
    path = KEPT / f"{group}.txt"
    kept = {}
    if path.exists():
        for line in path.read_text().splitlines():
            if line and not line.startswith("#"):
                ident, name, _ = line.split(" ", 2)
                kept[(ident, name)] = line
    return kept


def write_kept(group, lines_by_id):
    KEPT.mkdir(exist_ok=True)
    path = KEPT / f"{group}.txt"
    old = read_kept(group)
    order = [r["id"] for r in load_group(group)]
    by_id = {}
    for (ident, _), line in old.items():
        by_id.setdefault(ident, []).append(line)
    by_id.update(lines_by_id)
    head = (f"# E27 M1 T2: the original's shell captures of shell/{group}.toml (h3shell.py run "
            "--record): per shot\n# `<id> <shot> t<ms> <digest>`, per clip `<id> <clip> clip "
            "<frames>@<first>-<last> <digest>`; numbers only.\n")
    body = "".join(line + "\n" for ident in order for line in by_id.get(ident, []))
    path.write_text(head + body)


def differences(a_dir, b_dir, a, b):
    """What differs between two runs' shots and clips, one line each (empty: none)."""
    out = []
    for ea, eb in zip(a, b):
        if ea == eb:
            continue
        name, kind = ea[0], ea[1]
        if kind == "shot" and eb[1] == "shot":
            pa = np.load(a_dir / f"{name}.npz")["pix"]
            pb = np.load(b_dir / f"{name}.npz")["pix"]
            box = h3drive.changed_box(pa, pb)
            count = int((pa != pb).sum())
            out.append(f"{name}: t{ea[2]} against t{eb[2]}, {count} pixels in {box}")
        else:
            out.append(f"{name}: {kept_line('', ea).strip()} against {kept_line('', eb).strip()}")
    if len(a) != len(b):
        out.append(f"{len(a)} shots against {len(b)}")
    return out


def run_command(rest):
    only = group_names = None
    once = record = False
    out = OUT
    i = 0
    while i < len(rest):
        if rest[i] == "--only":
            only = set(rest[i + 1].split(","))
            i += 2
        elif rest[i] == "--group":
            group_names = rest[i + 1].split(",")
            i += 2
        elif rest[i] == "--out":
            out = Path(rest[i + 1])
            i += 2
        elif rest[i] == "--once":
            once = True
            i += 1
        elif rest[i] == "--record":
            record = True
            i += 1
        else:
            sys.exit(__doc__)
    if once and record:
        sys.exit("--record needs two runs (not --once)")
    recipes = [r for r in all_recipes()
               if (only is None or r["id"] in only)
               and (group_names is None or r["group"] in group_names)]
    if not recipes:
        sys.exit("no recipe chosen")
    failed = 0
    for recipe in recipes:
        ident, group = recipe["id"], recipe["group"]
        kept = read_kept(group)
        started = time.monotonic()
        runs = []
        try:
            for k in range(1 if once else 2):
                here = out / ident / f"run{k + 1}"
                runs.append((here, Run(recipe, here).play()))
        except Exception as err:  # noqa: BLE001 (each recipe reports, the next runs)
            failed += 1
            print(f"{ident}: FAILED {err}")
            continue
        (a_dir, a), (b_dir, b) = runs[0], runs[-1]
        diffs = [] if once else differences(a_dir, b_dir, a, b)
        lines = [kept_line(ident, ea, eb) for ea, eb in zip(a, b)]
        for line in lines:
            name = line.split(" ", 2)[1]
            was = kept.get((ident, name))
            state = "new" if was is None else ("ok" if was == line else "CHANGED")
            print(f"{ident} {name}: {state}")
        for d in diffs:
            print(f"{ident}: differs: {d}")
        if diffs:
            (out / ident / "differences.txt").write_text("".join(d + "\n" for d in diffs))
        if record:
            # Each recipe's lines as it ends (a VM restart loses only the recipe in progress).
            write_kept(group, {ident: lines})
        print(f"{ident}: {len(a)} kept, {'same' if not diffs else f'{len(diffs)} differ'} "
              f"({time.monotonic() - started:.0f} s)")
    print(f"{len(recipes) - failed} of {len(recipes)} recipes played" +
          (f", {failed} FAILED" if failed else ""))
    return 1 if failed else 0


def main(args):
    if not args:
        sys.exit(__doc__)
    if args[0] == "run":
        sys.exit(run_command(args[1:]))
    elif args[0] == "play" and len(args) >= 2:
        recipe = next((r for r in all_recipes() if r["id"] == args[1]), None)
        if recipe is None:
            sys.exit(f"no recipe `{args[1]}`")
        out = OUT / args[1] / "play"
        if "--out" in args:
            out = Path(args[args.index("--out") + 1])
        for entry in Run(recipe, out).play(keep="--keep" in args):
            print(kept_line(args[1], entry))
    elif args[0] == "learn" and len(args) >= 3:
        group, name = args[1], args[2]
        if not (SHELL / f"{group}.toml").exists():
            sys.exit(f"no group `{group}` (shell/{group}.toml)")
        inside, avoid, i = (0, 0, h3drive.WIDTH - 1, h3drive.HEIGHT - 1), [], 3
        while i + 1 < len(args):
            box = tuple(map(int, args[i + 1].split(",")))
            if args[i] == "--in":
                inside = box
            elif args[i] == "--avoid":
                avoid.append(box)
            else:
                sys.exit(__doc__)
            i += 2
        learn(group, name, inside, avoid)
    elif args[0] == "list":
        for r in all_recipes():
            shots = [s.split(":")[1] for s in r["steps"] if s.split(":")[0] in
                     ("shot", "clip", "clip-until")]
            print(f"{r['group']}/{r['id']}: {', '.join(shots)}")
    else:
        sys.exit(__doc__)


def learn(group, name, inside, avoid):
    """h3drive.learn's grid of probes, written into the group's own screens file; a screen of
    h3drive's or another group's is refused."""
    path = SHELL / f"{group}.screens.toml"
    own = tomllib.loads(path.read_text()) if path.exists() else {}
    if name in set(screens()) - set(own):
        sys.exit(f"the screen `{name}` is h3drive's or another group's")
    old = h3drive.SCREENS
    h3drive.SCREENS = path
    try:
        h3drive.learn(name, inside, avoid)
    finally:
        h3drive.SCREENS = old


if __name__ == "__main__":
    os.chdir(HERE)
    main(sys.argv[1:])
