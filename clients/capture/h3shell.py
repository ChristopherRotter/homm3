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
  h3shell.py drawn [--group <g>[,<g>...]] [--only <id>[,<id>...]] [--record] [--out <dir>]
      The viewer's side, without the original (no Wine; E27 M2 T4): each recipe's steps played
      by `h3-view shell --steps` on the shell's own clock (the release build of clients/; a
      recipe's `Show Intro` and `Bink Video` as `--no-intro` and `--video-quality`), each
      shot and clip of the viewer against the original's kept line (`exact`: the same pixels at
      the same time; else what differs, by pixel count and box when the original's frames of a
      `run` are in <out>'s parent, clients/out/shell) and against the viewer's own record,
      shell/drawn.txt (`same`, `CHANGED`, `new`, `GONE`); a recipe the viewer cannot finish says
      at which step and why (a screen not built yet names its task). Exits 1 unless every
      recipe's shots are as recorded; --record writes them. The frames stay in <out> (default
      clients/out/shell-drawn): <id>/frames/<shot>.565 (5-6-5, as the harness's dumps).

A recipe (a `[[recipe]]` of shell/<group>.toml): `id`, `what` (the catalogue's section), and
  registry  dwords of the settings key for this run, over setup.sh's (`Show Intro`, …)
  maps      the run's Maps/: "all" (the default: original-data/install/Maps), or a list of names
            (the viewer's too since E27 M3 T1b1: `drawn` gives them as `--maps`)
  made      maps of the harness's own (E27 M3 T2c; `[[recipe.made]]` tables): `file` (its name in
            the run's Maps/, listed in `maps`), `from` (the shipped map it copies), `name` and
            `description` (its header's, ours, Windows-1252), written at each run's start
            (`made_map`) and given to the viewer by path; never kept, never committed (ADR-0009)
  games     the run's Games/: files of original-data/games (default none)
  makes     saved games the run leaves in its Games/ (E27 M4 T4a; names, no spaces): each copied
            at the run's end into <out>/games/ and clients/out/shell/games/<id>/ (local, never
            in git) with the recipe's digest (`made.txt`): a maker recipe, the original making
            saves of shipped maps that a client's save can name too. A save's bytes are not the
            same from run to run (the original writes a few heap pointers it never set), so no
            digest of them is kept
  made_games  the run's Games/ too (E27 M4 T4a; `[[recipe.made_games]]` tables): `by` (the maker
            recipe) and `file` (one of its `makes`), taken from clients/out/shell/games/<by>/;
            the maker is played first when the file is not there or its recipe changed since.
            `drawn` gives the viewer the client's saves of the same names (E27 M4 T4b1,
            `h3-view save`), made on the maker's first map with its `viewer` table
  viewer    a maker's save as the client makes it (E27 M4 T4b1): `difficulty` (0 to 4) and `day`
            (the day it is made on, default 1); left out of the maker's digest
  tz        the run's time zone (E27 M4 T4a; Wine's local time is the TZ variable's, the list's
            dates by it: design §38), default "UTC"; a zone without daylight saving time, since
            the original converts a file's time by the present offset
  frozen    false: the clock runs in real time until a step freezes it (default true)
  sound     true: the game plays its sounds (h3drive's `sound`: an audio driver, the effects at
            volume 5, the samples on the virtual clock), each sample started kept as a line
            `<id> sound<k> t<ms> ch<channel> <length>ms x<loops>` (E27 M2 T4; default false;
            compared by `drawn` with the viewer's since E27 M2 T6a),
            and (E27 M2 T5b2) each clip's sound, its track on the virtual clock too (xdd.dll's
            7.), as `<id> voice<k> <smacker|bink> t<start>-<end> v<volume>` (`clip_voices`),
            compared by `drawn` with the viewer's
  backgrounds  the scenario screen's background at each opening, in order, as the original drew
            it under the harness (0 `gamselb0.pcx`, 1 `gamselb1.pcx`; E27 M3 T1a): the viewer's
            (`drawn`) draws, never the game's (default 0 at every opening)
  steps     the steps, in order (`STEPS`)
Every run starts from setup.sh's registry (the settings key removed and written again: what a
run saved never reaches the next), no HiScore.dat, the listed maps and saves.
"""

import gzip
import hashlib
import math
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

import h3drive

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent.parent
SHELL = HERE / "shell"
KEPT = SHELL / "kept"
OUT = ROOT / "clients" / "out" / "shell"
# The viewer's frames of `drawn` and its record (E27 M2 T4).
DRAWN_OUT = ROOT / "clients" / "out" / "shell-drawn"
DRAWN = SHELL / "drawn.txt"
H3VIEW = ROOT / "clients" / "target" / "release" / "h3-view"
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
# The display's auto-repeat while a `key-hold` holds a key (E27 M4 T4a): Xvfb's defaults, a
# 660 ms delay, then 25 repeats a second.
REPEAT_DELAY = 660
REPEAT_RATE = 25
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
  key-hold:<name>:<s>    a key held <s> seconds of real time with the display's auto-repeat on
                         (660 ms, then 25 a second), then let go: a held key's repeats
  type:<text>            the text typed
  press:<button>[:<k>] | point:<button>[:<k>]   a dialog's button found by its probes ([button.
                         <name>], screens.toml: ok, cancel, …; the k-th from the top left)
                         clicked, or only pointed at
  until-button:<button>[:<limit ms>]   step the clock until the button shows
  games-time:<s>[:<file>]   every file of the run's Games/ (or only <file>) modified at <s>
                         (seconds since 1970)
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
  mask:<x0>,<y0>,<x1>,<y1>   from here on the box (x1 and y1 excluded) is left out of the
                         shots' and clips' digests, the original's and the viewer's: what the
                         machine draws there (TCP/IP's address); `unmask` ends every mask
  unmask                 no box left out from here on
After each input the clock moves on by the recipe's `input_step` (default 100 ms: the game
draws its cursor at the pointer only on a later tick) and the game is idle before the next step."""


def digest(pix, masks=()):
    """The pixels' digest, each mask's box (x0, y0, x1, y1; the last two excluded) set to 0 first
    (E27 M2 T4: what the machine draws, TCP/IP's address)."""
    if masks:
        pix = np.array(pix, copy=True)
        for x0, y0, x1, y1 in masks:
            pix[y0:y1, x0:x1] = 0
    return hashlib.sha256(np.ascontiguousarray(pix, dtype="<u2").tobytes()).hexdigest()[:16]


def mask_box(arg):
    """A `mask` step's box."""
    box = tuple(int(v) for v in arg.split(","))
    if len(box) != 4 or box[0] >= box[2] or box[1] >= box[3]:
        raise ValueError(f"mask:{arg}: not <x0>,<y0>,<x1>,<y1> with x0 < x1 and y0 < y1")
    return box


def shot_masks(steps):
    """Each shot's and clip's masks, by its name, as the `mask` and `unmask` steps before it
    leave them (E27 M2 T4)."""
    masks, out = [], {}
    for step in steps:
        verb, _, arg = step.partition(":")
        if verb == "mask":
            masks = masks + [mask_box(arg)]
        elif verb == "unmask":
            masks = []
        elif verb in ("shot", "clip", "clip-until"):
            out[arg.split(":")[0]] = list(masks)
    return out


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


SHIPPED_MAPS = DATA / "install" / "Maps"
# The saves maker recipes made (E27 M4 T4a, `makes`): <maker id>/<file>, local, never in git.
MADE_GAMES = OUT / "games"
# A run's time zone unless its recipe gives one (E27 M4 T4a, `tz`).
DEFAULT_TZ = "UTC"
# A map file's formats whose header starts as `made_map` reads it (RoE, AB, SoD: h3m.md).
MADE_FORMATS = (0x0E, 0x15, 0x1C)


def made_map(made):
    """A map of the harness's own (E27 M3 T2c): the shipped map `from` with its header's name and
    description replaced (each a length and Windows-1252 bytes after the format, the players'
    flag, the size and the levels: 10 bytes), compressed again (no file time: the same bytes at
    every run)."""
    raw = gzip.decompress((SHIPPED_MAPS / made["from"]).read_bytes())
    if struct.unpack_from("<I", raw, 0)[0] not in MADE_FORMATS:
        raise ValueError(f"{made['from']}: not a RoE, AB or SoD map")

    def field(at):
        return at + 4 + struct.unpack_from("<I", raw, at)[0]

    name_end = field(10)
    description_end = field(name_end)
    name = made.get("name")
    name = raw[14:name_end] if name is None else name.encode("cp1252")
    description = made["description"].encode("cp1252")
    header = b"".join(struct.pack("<I", len(s)) + s for s in (name, description))
    return gzip.compress(raw[:10] + header + raw[description_end:], mtime=0)


def write_made(recipe, folder):
    """The recipe's own maps (`made`) written into `folder`; their paths."""
    folder.mkdir(parents=True, exist_ok=True)
    paths = []
    for made in recipe.get("made", []):
        path = folder / made["file"]
        path.write_bytes(made_map(made))
        paths.append(path)
    return paths


def recipe_digest(recipe):
    """A recipe's digest (E27 M4 T4a: what a maker's saves were made by): its table as read,
    the group and the viewer's save (`viewer`, E27 M4 T4b1: not the original's) left out."""
    table = {k: v for k, v in recipe.items() if k not in ("group", "viewer")}
    return hashlib.sha256(repr(sorted(table.items())).encode()).hexdigest()[:16]


def recipe_by_id(ident):
    recipe = next((r for r in all_recipes() if r["id"] == ident), None)
    if recipe is None:
        raise RuntimeError(f"no recipe `{ident}`")
    return recipe


def client_save(made, folder):
    """The client's save matching a maker's (E27 M4 T4b1, `made_games` for `drawn`): `h3-view
    save` on the maker's map with its `viewer` table's `difficulty` and `day` (default 1), named
    and written as the maker's file in `folder`."""
    by, name = made["by"], made["file"]
    maker = recipe_by_id(by)
    viewer = maker.get("viewer")
    if viewer is None or not maker.get("maps"):
        raise RuntimeError(f"`{by}` has no `viewer` table or map: no client's save of {name}")
    done = subprocess.run(
        [str(H3VIEW), "save", "--map", maker["maps"][0], "--difficulty", str(viewer["difficulty"]),
         "--day", str(viewer.get("day", 1)), "--name", Path(name).stem, "--out",
         str(folder / name)],
        capture_output=True, text=True)
    if done.returncode != 0:
        raise RuntimeError(f"{by}: the client's {name}: {(done.stdout + done.stderr).strip()}")
    return folder / name


def made_game(made):
    """A save a maker recipe made (E27 M4 T4a, `made_games`): its file under MADE_GAMES, the
    maker played first when the file is missing or was made by another version of the recipe."""
    by, name = made["by"], made["file"]
    maker = recipe_by_id(by)
    if name not in maker.get("makes", []):
        raise RuntimeError(f"`{by}` makes no {name}")
    folder = MADE_GAMES / by
    stamp = folder / "made.txt"
    current = (folder / name).is_file() and stamp.is_file() and \
        stamp.read_text().strip() == recipe_digest(maker)
    if not current:
        print(f"{by}: making {', '.join(maker['makes'])}", flush=True)
        Run(maker, OUT / by / "make").play()
    return folder / name


def prepare(recipe):
    game = h3drive.GAME_DIR
    # E27 M4 T4a: the made saves first (a maker's run is a run of its own).
    made_saves = [made_game(m) for m in recipe.get("made_games", [])]
    # E27 M4 T4a: the run's time zone pinned (Wine's local time follows TZ; the list shows a
    # save's time in it, design §38): every Wine command from here takes it (`h3drive.env`).
    os.environ["TZ"] = recipe.get("tz", DEFAULT_TZ)
    maps = recipe.get("maps", "all")
    shipped = SHIPPED_MAPS
    made = {m["file"] for m in recipe.get("made", [])}
    if maps == "all":
        chosen = sorted(p for p in shipped.iterdir() if p.is_file())
    else:
        chosen = [shipped / m for m in maps if m not in made]
    missing = [str(p) for p in chosen if not p.is_file()]
    if missing:
        raise RuntimeError(f"{recipe['id']}: maps not staged: {missing}")
    fill(game / "Maps", chosen)
    write_made(recipe, game / "Maps")
    saves = [DATA / "games" / g for g in recipe.get("games", [])]
    missing = [str(p) for p in saves if not p.is_file()]
    if missing:
        raise RuntimeError(f"{recipe['id']}: saved games not staged: {missing}")
    fill(game / "Games", saves + made_saves)
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
    # E27 M4 T4a: the display's auto-repeat off (h3drive's and h3map.py's runs turn it off too,
    # Xvfb starts with it on): a key held across steps is one key down; `key-hold` turns it on.
    h3drive.run("xset", "r", "off")


# -- the steps ---------------------------------------------------------------------------------
class Run:
    def __init__(self, recipe, out):
        self.recipe = recipe
        self.out = Path(out)
        if self.out.exists():
            shutil.rmtree(self.out)
        self.out.mkdir(parents=True)
        frozen = FROZEN_AT if recipe.get("frozen", True) else None
        # E27 M2 T4: a recipe with `sound` plays the game's sounds (h3drive's `sound`).
        self.game = h3drive.Game(self.out, sound=bool(recipe.get("sound", False)), frozen=frozen)
        self.game.screens = screens()
        self.kept = []  # (name, kind, time, digest or (count, first, last, digest))
        self.input_step = int(recipe.get("input_step", INPUT_STEP))
        # The next input's settle in seconds (`settle`; INPUT_SETTLE when None).
        self.settle = None
        # The boxes left out of the digests from here on (`mask`; E27 M2 T4).
        self.masks = []

    def now(self):
        a = self.game.ack(need=True)
        return a[1] - FROZEN_AT

    def reads(self):
        """The clock's reads since the last command: the game's and the clips' (xdd.dll's 5.)."""
        a = self.game.ack(need=True)
        return a[2] + a[11]

    def check_clip_sound(self):
        """A sound run's clips played on xdd.dll's devices (E27 M2 T5b2): its 7.'s three Smacker
        hooks and Bink's 11 Miles functions in place, no call the hooks could not follow."""
        a = self.game.ack(need=True)
        if a[13] != 3:
            raise RuntimeError(f"xdd.dll put {a[13]} of the clips' 3 Smacker sound hooks in place")
        if a[14] != len(BINK_CALLS):
            raise RuntimeError(f"xdd.dll answered {a[14]} of Bink's {len(BINK_CALLS)} Miles "
                               "functions (the sound never started?)")
        if a[9]:
            raise RuntimeError(f"xdd.dll could not follow {a[9]} sample calls")

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
        self.kept.append((name, "shot", self.now(), digest(pix, self.masks)))

    def clip(self, name, ms=None, screen=None, step=UNTIL_STEP, limit=UNTIL_LIMIT):
        here = self.out / name
        here.mkdir()
        self.idle()
        last = self.game.settled()
        start = self.now()
        lines = [(start, digest(last, self.masks))]
        np.savez_compressed(here / f"t{start}.npz", pix=last)
        end = start + (ms if ms is not None else limit)
        while self.now() < end:
            self.tick(f"step {step}")
            pix = self.game.settled()
            if not np.array_equal(pix, last):
                t = self.now()
                lines.append((t, digest(pix, self.masks)))
                np.savez_compressed(here / f"t{t}.npz", pix=pix)
                last = pix
            if screen and any(self.game.matches(n, pix) for n in screen.split("|")):
                break
        else:
            if screen:
                raise TimeoutError(f"{name}: the screen `{screen}` within {limit} ms")
        (here.parent / f"{name}.txt").write_text("".join(f"{t} {d}\n" for t, d in lines))
        self.kept.append((name, "clip", start, clip_entry(lines)))

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
        elif verb == "key-hold":
            # E27 M4 T4a: a key held for <s> seconds of real time with the display's auto-repeat
            # on (Xvfb's defaults: a 660 ms delay, then 25 a second), then auto-repeat off and
            # the key let go. Wine takes X's repeats as Windows' (a WM_KEYDOWN each, no WM_KEYUP
            # between: XkbSetDetectableAutoRepeat), and the game queues every key down
            # (input_key_message 0x4EC0E0, its repeat flag not looked at). A second `xdotool
            # keydown` of a held key gives no event (found 2026-11-20), so a repeat comes only
            # this way; its count follows real time, so a recipe holds a key long enough that the
            # outcome no longer depends on it (the list's end reached).
            name, _, seconds = arg.partition(":")
            h3drive.run("xset", "r", "rate", str(REPEAT_DELAY), str(REPEAT_RATE))
            h3drive.run("xset", "r", "on")
            h3drive.run("xdotool", "keydown", name)
            g.keys_held.add(name)
            time.sleep(float(seconds))
            h3drive.run("xset", "r", "off")
            h3drive.run("xdotool", "keyup", name)
            g.keys_held.discard(name)
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
            # E27 M4 T4a: `<s>:<file>` gives one file its time (each save's own date).
            when, _, only = arg.partition(":")
            games = h3drive.GAME_DIR / "Games"
            paths = [games / only] if only else sorted(games.iterdir())
            for path in paths:
                if not path.is_file():
                    raise RuntimeError(f"games-time:{arg}: no {path.name} in the run's Games/")
                os.utime(path, (int(when), int(when)))
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
        elif verb == "mask":
            self.masks = self.masks + [mask_box(arg)]
        elif verb == "unmask":
            self.masks = []
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

    def made(self):
        """The saves the run left in its Games/ (E27 M4 T4a, `makes`), copied into the run's
        out/games/ and the maker's MADE_GAMES folder with the recipe's digest (`made.txt`)."""
        names = self.recipe.get("makes", [])
        if not names:
            return
        games = h3drive.GAME_DIR / "Games"
        missing = [n for n in names if not (games / n).is_file()]
        if missing:
            listed = sorted(p.name for p in games.iterdir())
            raise RuntimeError(f"the run left no {missing} in its Games/ (there: {listed})")
        made = MADE_GAMES / self.recipe["id"]
        for folder in (self.out / "games", made):
            folder.mkdir(parents=True, exist_ok=True)
            for name in names:
                shutil.copy(games / name, folder / name)
        (made / "made.txt").write_text(recipe_digest(self.recipe) + "\n")

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
            self.made()
            if self.game.sound:
                # Each sample started (xdd.dll's `sounds`), its time from the frozen start.
                for k, (t, channel, length, loops) in enumerate(h3drive.sounds(self.game), 1):
                    self.kept.append((f"sound{k}", "sound", t - FROZEN_AT,
                                      (channel, length, loops)))
                # E27 M2 T5b2: each clip's sound (xdd.dll's 7.), its device on the clock.
                self.check_clip_sound()
                for k, (codec, start, end, volume) in enumerate(
                        clip_voices(self.game.clock_dir / "clipsound"), 1):
                    self.kept.append((f"voice{k}", "voice", start,
                                      (codec, end, volume)))
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
def clip_entry(lines):
    """A clip's kept entry from its (time, digest) lines: their count, first and last times and
    the digest of the lines."""
    whole = hashlib.sha256("".join(f"{t} {d}\n" for t, d in lines).encode()).hexdigest()[:16]
    return (len(lines), lines[0][0], lines[-1][0], whole)


# The Miles functions xdd.dll's 7. answers for BINKW32.DLL (its `bink_calls`).
BINK_CALLS = ("minimum_sample_buffer_size", "allocate_sample_handle", "init_sample",
              "set_sample_type", "set_sample_playback_rate", "set_sample_volume", "stop_sample",
              "resume_sample", "sample_buffer_ready", "load_sample_buffer",
              "release_sample_handle")


def volume_index(hundredths):
    """The index i of SMACKW32's volume table (0x100148E8: 2000 log10(i / 127), truncated; 0 is
    -10000) holding a DirectSound volume (as clients/scripts/smacker-dll.py's)."""
    for i in range(128):
        if (-10000 if i == 0 else int(2000 * math.log10(i / 127))) == hundredths:
            return i
    return f"unknown{hundredths}"


def clip_voices(path):
    """The clips' sound as xdd.dll's 7. logs it (E27 M2 T5b2; `<time> <call> <a> <b>` lines):
    (codec, start, end, volume) per voice in the order they began, the times from the frozen
    start, `end` None while it still played at the run's end. A Smacker clip's voice is its
    DirectSound buffer (`create` at SmackOpen): from its first `play` (the DLL's sound service)
    to its `stop` (SmackClose stops and releases it), its volume as the index of SMACKW32's
    table (0 to 127, `h3_view::clip_sound::volume`'s); a Bink clip's its Miles sample
    (`bink-allocate` at BinkOpen): from its first load (`bink-play`) to its pause (`bink-stop`,
    a skip) or its release (BinkClose), its volume Miles' 0 to 127. A buffer or sample closed
    before it played is no voice."""
    try:
        text = path.read_text()
    except OSError:
        return []
    voices, current = [], {}
    for line in text.splitlines():
        parts = line.split()
        if len(parts) != 4:
            continue
        t, what = int(parts[0]) - FROZEN_AT, parts[1]
        a = int(parts[2])
        codec = "bink" if what.startswith("bink-") else "smacker"
        what = what.removeprefix("bink-")
        if what in ("create", "allocate"):
            current[codec] = {"codec": codec, "volume": None, "start": None, "end": None}
            continue
        voice = current.get(codec)
        if voice is None:
            continue
        if what == "volume":
            voice["volume"] = volume_index(a) if codec == "smacker" else a
        elif what == "play" and voice["start"] is None:
            voice["start"] = t
            voices.append(voice)
        elif what in ("stop", "release") and voice["start"] is not None and voice["end"] is None:
            voice["end"] = t
        if what == "release":
            current.pop(codec)
    return [(v["codec"], v["start"], v["end"], v["volume"]) for v in voices]


def kept_line(ident, entry, other=None):
    name, kind, t, d = entry
    if kind == "shot":
        line = f"{ident} {name} t{t} {d}"
    elif kind == "sound":
        channel, length, loops = d
        line = f"{ident} {name} t{t} ch{channel} {length}ms x{loops}"
    elif kind == "voice":
        codec, end, volume = d
        line = f"{ident} {name} {codec} t{t}-{'' if end is None else end} v{volume}"
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


# -- the viewer's side (E27 M2 T4) ---------------------------------------------------------------
def load_565(path):
    """A frame the viewer wrote (`h3-view shell --frames`): 800 × 600 pixels of 5-6-5."""
    return np.fromfile(path, dtype="<u2").reshape(h3drive.HEIGHT, h3drive.WIDTH)


def read_drawn():
    """The viewer's record (shell/drawn.txt), by (id, name): its lines."""
    found = {}
    if DRAWN.exists():
        for line in DRAWN.read_text().splitlines():
            if line and not line.startswith("#"):
                ident, name, _ = line.split(" ", 2)
                found[(ident, name)] = line
    return found


def write_drawn(lines_by_id):
    """The viewer's record, the recipes run replaced, in the recipes' order."""
    old = read_drawn()
    by_id = {}
    for (ident, _), line in old.items():
        by_id.setdefault(ident, []).append(line)
    by_id.update(lines_by_id)
    order = [r["id"] for r in all_recipes()]
    head = ("# E27 M2 T4: the viewer's shots and clips of the shell's recipes (h3shell.py drawn "
            "--record): per shot\n# `<id> <shot> t<ms> <digest>`, per clip `<id> <clip> clip "
            "<frames>@<first>-<last> <digest>`, as shell/kept/; numbers only.\n")
    DRAWN.write_text(head + "".join(line + "\n" for ident in order
                                    for line in by_id.get(ident, [])))


def originals(line):
    """The original's kept line as the forms a viewer's line can match: the first run's and, for
    a shot or clip whose runs differ, the second's."""
    ident, name, rest = line.split(" ", 2)
    first, _, second = rest.partition(" differs ")
    return [f"{ident} {name} {first}"] + ([f"{ident} {name} {second}"] if second else [])


def drawn_recipe(recipe, out):
    """One recipe through the viewer: its kept-form lines by name, the stop's line (or None), or
    the viewer's failure as a string."""
    ident = recipe["id"]
    here = out / ident
    shutil.rmtree(here, ignore_errors=True)
    frames = here / "frames"
    frames.mkdir(parents=True)
    # The masks are the digests' (Python's), not the viewer's steps: left out, their places kept.
    index = [k for k, s in enumerate(recipe["steps"]) if s.split(":")[0] not in ("mask", "unmask")]
    (here / "steps.txt").write_text("".join(recipe["steps"][k] + "\n" for k in index))
    args = [str(H3VIEW), "shell", "--steps", str(here / "steps.txt"), "--frames", str(frames)]
    if int(recipe.get("registry", {}).get("Show Intro", 0)) == 0:
        args.append("--no-intro")
    # E27 M2 T5a: Bink Video 0 (Video Quality Low) plays the start's Smacker copies.
    if int(recipe.get("registry", {}).get("Bink Video", 1)) == 0:
        args += ["--video-quality", "low"]
    if "input_step" in recipe:
        args += ["--input-step", str(recipe["input_step"])]
    # E27 M3 T1a: the scenario screen's backgrounds as the original drew them, one an opening.
    if recipe.get("backgrounds"):
        args += ["--backgrounds", ",".join(str(int(b)) for b in recipe["backgrounds"])]
    # E27 M3 T1b1: a recipe's own maps (its run's Maps/), a name a line; "all" gives every map.
    # E27 M3 T2c: a map of the harness's own by its path (written here).
    maps = recipe.get("maps", "all")
    made = {p.name: p for p in write_made(recipe, here / "made")}
    if maps == "all":
        maps = sorted(p.name for p in SHIPPED_MAPS.iterdir() if p.is_file()) if made else None
    if maps is not None:
        lines = [str(made[m].resolve()) if m in made else m for m in maps]
        lines += [str(p.resolve()) for name, p in made.items() if name not in maps]
        (here / "maps.txt").write_text("".join(m + "\n" for m in lines))
        args += ["--maps", str(here / "maps.txt")]
    # E27 M2 T5b1: a sound recipe's effects (h3drive's volume 5): the clips with their sound, its
    # starts and stops listed (`sound`, `quiet`; compared from T5b2 on).
    if recipe.get("sound", False):
        args += ["--effects", str(h3drive.SOUND_VOLUME)]
    # E27 M4 T4b1: the viewer's saves' folder, the run's own (empty but for the client's saves
    # matching the makers', `made_games`): never the working directory's.
    games = here / "games"
    games.mkdir(parents=True, exist_ok=True)
    for made in recipe.get("made_games", []):
        client_save(made, games)
    args += ["--games", str(games)]
    # E27 M4 T4a: the recipe's time zone, the original's (a save's local time on the list).
    env = dict(os.environ, TZ=recipe.get("tz", DEFAULT_TZ))
    done = subprocess.run(args, capture_output=True, text=True, env=env)
    if done.returncode != 0:
        return None, None, ((done.stdout + done.stderr).strip().splitlines() or ["no output"])[-1]
    masks = shot_masks(recipe["steps"])
    lines, clips, stop, voices, samples = {}, {}, None, [], []
    for line in (frames / "replay.txt").read_text().splitlines():
        kind, name, rest = line.split(" ", 2)
        if kind == "sample":
            # E27 M2 T6a: `sample <file> t<ms> ch<k> <length>ms x<loops>`, a press's sample as
            # xdd.dll logs the original's (`sounds`): its `sound<k>` lines.
            t, channel, length, loops = rest.split()
            samples.append((int(t.lstrip("t")), (int(channel.removeprefix("ch")),
                                                 int(length.removesuffix("ms")),
                                                 int(loops.lstrip("x")))))
        elif kind == "sound":
            # E27 M2 T5b2: `sound <file> t<ms> v<volume>`, ended by its `quiet` (a skip) or
            # `closed` (the last frame) line: the original's voices (`clip_voices`).
            t, volume = rest.split()
            codec = "bink" if name.upper().endswith(".BIK") else "smacker"
            voices.append({"file": name, "codec": codec, "start": int(t.lstrip("t")),
                           "end": None, "volume": int(volume.lstrip("v"))})
        elif kind in ("quiet", "closed"):
            ended = next((v for v in voices if v["file"] == name and v["end"] is None), None)
            if ended is not None:
                ended["end"] = int(rest.lstrip("t"))
        elif kind == "shot":
            t = int(rest.lstrip("t"))
            pix = load_565(frames / f"{name}.565")
            lines[name] = kept_line(ident, (name, "shot", t, digest(pix, masks.get(name, []))))
        elif kind == "clip":
            t = int(rest.lstrip("t"))
            pix = load_565(frames / name / f"t{t}.565")
            clips.setdefault(name, []).append((t, digest(pix, masks.get(name, []))))
        elif kind == "stop":
            # `stop <k> <why>`: k counts the viewer's steps, the masks left out.
            k = index[int(name)]
            stop = f"step {k + 1} `{recipe['steps'][k]}`: {rest}"
    for name, frames_of in clips.items():
        lines[name] = kept_line(ident, (name, "clip", frames_of[0][0], clip_entry(frames_of)))
    order = list(masks)
    for k, (t, d) in enumerate(samples, 1):
        order.append(f"sound{k}")
        lines[f"sound{k}"] = kept_line(ident, (f"sound{k}", "sound", t, d))
    for k, v in enumerate(voices, 1):
        order.append(f"voice{k}")
        lines[f"voice{k}"] = kept_line(ident, (f"voice{k}", "voice", v["start"],
                                               (v["codec"], v["end"], v["volume"])))
    return dict(sorted(lines.items(), key=lambda kv: order.index(kv[0]))), stop, None


def compared(ident, name, ours, original):
    """How the viewer's line `ours` (None: not drawn) stands to the original's kept `original`."""
    if ours is None:
        return "not drawn"
    forms = originals(original)
    if ours in forms:
        return "exact"
    mine = ours.split(" ", 2)[2].split()
    theirs = forms[0].split(" ", 2)[2].split()
    if name.startswith(("voice", "sound")):
        return f"not the original's ({' '.join(theirs)}: the viewer's {' '.join(mine)})"
    if mine[-1] == theirs[-1]:
        return f"the original's pixels at {mine[0]}, not {theirs[0]}"
    note = ""
    shot = OUT / ident / "run1" / f"{name}.npz"
    ours_file = DRAWN_OUT / ident / "frames" / f"{name}.565"
    if mine[0].startswith("t") and shot.exists() and ours_file.exists():
        a, b = np.load(shot)["pix"], load_565(ours_file)
        note = f": {int((a != b).sum())} pixels in {h3drive.changed_box(a, b)}"
    return "not the original's" + note


def drawn_command(rest):
    only = group_names = None
    record = False
    out = DRAWN_OUT
    i = 0
    while i < len(rest):
        if rest[i] in ("--only", "--group", "--out") and i + 1 < len(rest):
            if rest[i] == "--only":
                only = set(rest[i + 1].split(","))
            elif rest[i] == "--group":
                group_names = rest[i + 1].split(",")
            else:
                out = Path(rest[i + 1])
            i += 2
        elif rest[i] == "--record":
            record = True
            i += 1
        else:
            sys.exit(__doc__)
    if not H3VIEW.exists():
        sys.exit(f"no {H3VIEW}: `cargo build --release -p h3-battle-client` in clients/ first")
    recipes = [r for r in all_recipes()
               if (only is None or r["id"] in only)
               and (group_names is None or r["group"] in group_names)]
    if not recipes:
        sys.exit("no recipe chosen")
    recorded = read_drawn()
    got, failed, shots, drawn, exact = {}, 0, 0, 0, 0
    for recipe in recipes:
        ident = recipe["id"]
        lines, stop, broken = drawn_recipe(recipe, out)
        if broken is not None:
            print(f"{ident}: the viewer failed ({broken})", flush=True)
            failed += 1
            continue
        got[ident] = list(lines.values())
        kept = {name: line for (i, name), line in read_kept(recipe["group"]).items() if i == ident}
        verdicts, changed = [], []
        for name, original in kept.items():
            shots += 1
            ours = lines.get(name)
            verdict = compared(ident, name, ours, original)
            drawn += ours is not None
            exact += verdict == "exact"
            verdicts.append(f"{name}: {verdict}")
        for name in sorted(set(lines) | {n for (i, n) in recorded if i == ident}):
            before, now = recorded.get((ident, name)), lines.get(name)
            if before != now:
                changed.append(f"{name} {'GONE' if now is None else 'new' if before is None else 'CHANGED'}")
        (out / ident / "verdicts.txt").write_text("".join(v + "\n" for v in verdicts))
        n_exact = sum(v.endswith(": exact") for v in verdicts)
        summary = (f"{ident}: {len(lines)} of {len(verdicts)} drawn, {n_exact} exact"
                   + (f"; stopped at {stop}" if stop else ""))
        if not changed:
            state = "as recorded"
        elif not any(i == ident for (i, _) in recorded):
            state = "new"
        else:
            state = (f"not as recorded ({len(changed)}): " + ", ".join(changed[:6])
                     + (", …" if len(changed) > 6 else ""))
        failed += bool(changed)
        print(f"{summary}; {state}", flush=True)
    if record:
        write_drawn(got)
        print(f"drawn: {len(got)} recipes recorded in {DRAWN}")
        return 1 if len(got) < len(recipes) else 0
    print(f"drawn: {len(recipes)} recipes, {drawn} of {shots} shots drawn, {exact} exact; "
          f"{failed} not as recorded; under {out}")
    return 1 if failed else 0


def main(args):
    if not args:
        sys.exit(__doc__)
    if args[0] == "run":
        sys.exit(run_command(args[1:]))
    elif args[0] == "drawn":
        sys.exit(drawn_command(args[1:]))
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
