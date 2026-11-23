#!/usr/bin/env python3
"""The original's random map generator as an oracle (E14 T1b1, *provisional*;
docs/lanes/rmg/ORACLE.md).

Runs the original under Wine as the shell's recipes run it (h3shell.py: a fresh start, the clock
frozen from the game's first read), with its date fixed (xdd.dll's H3TIME, xdd.c's 9.: the
generator's seed is time() in seconds), sets the scenario screen's Random Map tab to a case's
choices (none Random: a Random choice draws from the game's earlier state, survey §1.1), presses
Begin and keeps the map the generator wrote into the run's random_maps/ (gzip). The map is the
original's output: it stays in <out> (default clients/out/rmg; local, never in git, ADR-0009);
only its digests and the choices that made it are kept (rmg/kept.txt).

  h3rmg.py one --seed <s> [--size S|M|L|XL] [--levels 1|2] [--players <n>] [--teams <t>]
               [--computers <c>] [--computer-teams <t>] [--water none|normal|islands]
               [--monsters weak|normal|strong] [--before "<step>;..."] [--again] [--out <dir>]
      One case: the map's digests, its seed and template as its description names them.
      --before plays h3shell steps on the main menu first (another history before the
      generation, for the memory-dependent draws; survey §2.6); --again generates a second map
      in the same game (the system options' Main Menu, then the same choices again).
  h3rmg.py corpus [--only <id>,...] [--twice] [--again] [--record] [--out <dir>]
      Each case of rmg/corpus.toml, against rmg/kept.txt: `ok` (as kept), `CHANGED`, `new`;
      --twice runs each case again from a fresh start, --again makes a second map in the same
      game, and either fails on a difference; --record writes the kept lines (the first map's).
  h3rmg.py read [--out <dir>]
      Each case's first map in <out> read by h3_map's reader, imported and validated in a game
      of its players (h3-map's original-data check, `cargo run -p h3-map --example
      original_data`, over a folder of them): how many were read, each one refused and why, the
      import by outcome. Its counts against the shipped maps' surveys do not apply and are left
      out. Exits 1 unless every map was read.
  h3rmg.py templates
      The template each case's seed draws, predicted from either archive's rmg.txt (the staged
      original-data/extracted/<archive>/Rmg.txt) by the survey's reading (§1.4, §1.5, §2.1: the
      templates kept for the size index and the players, then the first rand() after
      srand(seed) modulo their count; none of the corpus's water is Random), against the one
      kept.txt names. Exits 1 unless SoD's table (H3bitmap.lod's, the one the game reads:
      ORACLE.md) predicts every kept case.

Exits 1 when a case failed, changed or differed between its runs.
"""
import argparse
import csv
import gzip
import hashlib
import io
import json
import os
import re
import shutil
import struct
import subprocess
import sys
import time
import tomllib
from pathlib import Path

import h3drive
import h3shell

HERE = Path(__file__).resolve().parent
RMG = HERE / "rmg"
CORPUS = RMG / "corpus.toml"
KEPT = RMG / "kept.txt"
OUT = h3shell.ROOT / "clients" / "out" / "rmg"
# A map of the scenario list (the screen needs one; the random map is chosen over it).
LIST_MAP = "Arrogance.h3m"

# The tab's buttons at 800 x 600 (E27 design §3.16; the shell's `adv-random` recipe): a choice's
# point by its value.
SIZES = {"S": 183, "M": 230, "L": 277, "XL": 324}
SIZE_Y = 97
LEVELS = (371, 97)  # toggles one level and two
WATER = {"none": 111, "normal": 196, "islands": 281}
WATER_Y = 435
MONSTERS = {"weak": 111, "normal": 196, "strong": 281}
MONSTERS_Y = 501
BEGIN = (497, 555)
TAB = (514, 115)
PARK = "move:790,300"
# The levels the tab shows at its first opening in a fresh start (observed, ORACLE.md).
DEFAULT_LEVELS = 2
# How long a generation may take in real time (2 to 3 s whatever the size, ORACLE.md; a wide
# bound).
GENERATION_LIMIT = 120.0
# After this long without a file, an OK box means the original refused the choices (GENRLTXT 750
# to 752, survey §1.2), looked for every REFUSAL_EVERY seconds.
REFUSAL_AFTER = 5.0
REFUSAL_EVERY = 5.0
# The path from the adventure map back to the main menu (--again; seen, ORACLE.md): the panel's
# System Options button, their Main Menu, the question's OK.
TO_MAIN_MENU = ["click:727,309", "click:565,437", "press:ok"]
# The game's clock moved on after a map's start, before the first input there (in 100 ms steps).
SETTLE_STEPS = 30
# The description's words for the choices (survey §1.6): the size in tiles, the monster strength
# 2 to 4 (the dialog's 0 to 2 plus 2, §1.2).
SIZE_TILES = {"S": 36, "M": 72, "L": 108, "XL": 144}
STRENGTH = {"weak": 2, "normal": 3, "strong": 4}

CHOICES = ("size", "levels", "players", "teams", "computers", "computer_teams", "water",
           "monsters")
DEFAULTS = {"size": "M", "levels": 1, "players": 2, "teams": 0, "computers": 0,
            "computer_teams": 0, "water": "normal", "monsters": "normal"}


def validate(case):
    """The case's choices, none Random, each within the tab's caps (a disabled button takes no
    press, so a choice past its cap would be the tab's earlier value)."""
    c = {**DEFAULTS, **{k: case[k] for k in CHOICES if k in case}}
    if c["size"] not in SIZES or c["levels"] not in (1, 2):
        raise ValueError(f"size {c['size']}, levels {c['levels']}")
    if not 1 <= c["players"] <= 8 or not 0 <= c["teams"] < max(c["players"], 1):
        raise ValueError(f"players {c['players']}, teams {c['teams']}")
    if not 0 <= c["computers"] <= 8 - c["players"]:
        raise ValueError(f"computers {c['computers']} with {c['players']} players")
    if c["computers"] and not 0 <= c["computer_teams"] < c["computers"]:
        raise ValueError(f"computer teams {c['computer_teams']} of {c['computers']}")
    if c["players"] + c["computers"] < 2:
        raise ValueError("fewer than two players in all (the generator makes them 1 and 1)")
    if c["water"] not in WATER or c["monsters"] not in MONSTERS:
        raise ValueError(f"water {c['water']}, monsters {c['monsters']}")
    return c


def tab_steps(c, levels=DEFAULT_LEVELS):
    """h3shell steps from the scenario list to the tab set to the choices `c`, the tab showing
    `levels` before (its toggle)."""
    steps = [f"click:{TAB[0]},{TAB[1]}", PARK,
             f"click:{SIZES[c['size']]},{SIZE_Y}", PARK]
    if c["levels"] != levels:
        steps += [f"click:{LEVELS[0]},{LEVELS[1]}", PARK]
    steps += [f"click:{85 + 32 * (c['players'] - 1)},169", PARK,
              f"click:{85 + 32 * c['teams']},235", PARK,
              f"click:{85 + 32 * c['computers']},301", PARK]
    if c["computers"]:
        steps += [f"click:{85 + 32 * c['computer_teams']},367", PARK]
    steps += [f"click:{WATER[c['water']]},{WATER_Y}", PARK,
              f"click:{MONSTERS[c['monsters']]},{MONSTERS_Y}", PARK, "shot:tab"]
    return steps


TO_LIST = ["click:643,70", "check:new_game_menu", "click:649,65", PARK, "until:scenario_list"]


def header(raw):
    """The map's format, size, levels, name and description (docs/formats/h3m.md, the header)."""
    fmt, _, size, levels = struct.unpack_from("<IBIB", raw, 0)
    at = 10
    texts = []
    for _ in range(2):
        (n,) = struct.unpack_from("<I", raw, at)
        texts.append(raw[at + 4:at + 4 + n].decode("latin-1"))
        at += 4 + n
    return fmt, size, levels + 1, texts[0], texts[1]


def described(description):
    """The template and seed a generated map's description names (survey §1.6: "Template was
    <name>, Random seed was <seed>, ...")."""
    m = re.search(r"Template was (.*?), Random seed was (\d+)", description, re.DOTALL)
    if not m:
        raise RuntimeError("the map's description names no template and seed")
    return m.group(1), int(m.group(2))


class Generation(h3shell.Run):
    """A run whose steps end on the tab set; then Begin, the generator's file awaited and kept
    (and with `again` a second generation in the same game)."""

    def __init__(self, recipe, out, choices, again=False):
        super().__init__(recipe, out)
        self.choices = choices
        self.again = again
        self.maps = []

    def made(self):
        folder = h3drive.GAME_DIR / "random_maps"
        self.maps.append(self.begin(folder, 1))
        if self.again:
            # The map's start reached, back to the main menu by the system options.
            self.idle()
            for _ in range(SETTLE_STEPS):
                self.tick("step 100")
            # The tab shows its first opening's levels again (seen: ORACLE.md); every choice is
            # pressed anew, and `facts` checks them against the description.
            for step in [*TO_MAIN_MENU, "until:main_menu", *TO_LIST, *tab_steps(self.choices)]:
                self.do(step)
            self.maps.append(self.begin(folder, 2))

    def begin(self, folder, count):
        """Begin pressed; waits for a new file in random_maps/ that stopped growing and is a whole
        gzip stream, copied to the run's out/ as map<count>.h3m."""
        g = self.game
        hooked = g.ack(need=True)[16]
        if hooked != 2:
            raise RuntimeError(f"xdd.dll hooked {hooked} of the 2 date imports (H3TIME, xdd.c's "
                               "9.)")
        before = {p.name for p in folder.iterdir()}
        g.click(*BEGIN, settle=0.1)
        start = time.monotonic()
        end = start + GENERATION_LIMIT
        last = None
        looked = start
        while time.monotonic() < end:
            time.sleep(1.0)
            new = sorted(p for p in folder.iterdir() if p.name not in before)
            if not new:
                if g.proc and g.proc.poll() is not None:
                    raise RuntimeError(f"the game ended before a map was written{g.crash_note()}")
                now = time.monotonic()
                if now - start >= REFUSAL_AFTER and now - looked >= REFUSAL_EVERY:
                    looked = now
                    if self.buttons("ok"):
                        g.dump(self.out / "refused.xwd")
                        raise RuntimeError("the original refused the choices: an OK box and no "
                                           "map (GENRLTXT 750 to 752; refused.xwd)")
                continue
            path = new[-1]
            size = path.stat().st_size
            if size and size == last:
                try:
                    gzip.decompress(path.read_bytes())
                except (EOFError, OSError, gzip.BadGzipFile):
                    continue
                kept = self.out / f"map{count}.h3m"
                shutil.copy(path, kept)
                return kept
            last = size
        raise TimeoutError(f"no whole map in random_maps/ within {GENERATION_LIMIT} s")


def recipe_for(ident, choices, before=()):
    steps = ["move:400,300", "until:main_menu", *before, *TO_LIST, *tab_steps(choices)]
    return {"id": ident, "what": "E14 T1b1: a random map generated", "maps": [LIST_MAP],
            "steps": steps}


def generate(ident, case, out, again=False):
    """One case generated: a list of (path, facts) per map (with `again` two)."""
    choices = validate(case)
    seed = int(case["seed"])
    folder = h3drive.GAME_DIR / "random_maps"
    if folder.exists():
        shutil.rmtree(folder)
    folder.mkdir()
    os.environ["H3TIME"] = str(seed)
    run = Generation(recipe_for(ident, choices, case.get("before", [])), out, choices, again)
    try:
        run.play()
    finally:
        os.environ.pop("H3TIME", None)
    return [(path, facts(path, seed, choices)) for path in run.maps]


def facts(path, seed, choices):
    gz = path.read_bytes()
    raw = gzip.decompress(gz)
    fmt, size, levels, _name, description = header(raw)
    template, said = described(description)
    if said != seed:
        raise RuntimeError(f"{path}: the description's seed is {said}, not {seed} (the date "
                           "not fixed? xdd.dll's H3TIME)")
    m = re.search(r"size (\d+), levels (\d+), humans (\d+), computers (\d+), water (\w+), "
                  r"monsters (\d+)", description)
    told = m and (int(m.group(1)), int(m.group(2)), int(m.group(3)), int(m.group(4)),
                  m.group(5).lower(), int(m.group(6)))
    wanted = (SIZE_TILES[choices["size"]], choices["levels"], choices["players"],
              choices["computers"], choices["water"], STRENGTH[choices["monsters"]])
    if told != wanted:
        raise RuntimeError(f"{path}: the description gives {told}, the case {wanted}")
    return {"template": template, "seed": said, "format": fmt, "size": size, "levels": levels,
            "bytes": len(raw), "h3m": hashlib.sha256(raw).hexdigest()[:16],
            "gz": hashlib.sha256(gz).hexdigest()[:16]}


def kept_line(ident, f):
    # The template's name quoted as JSON: names hold spaces and, as rmg.txt gives one, a line
    # break ("Meeting \nin Muzgob").
    return (f"{ident} seed={f['seed']} template={json.dumps(f['template'])} "
            f"format=0x{f['format']:X} size={f['size']} levels={f['levels']} bytes={f['bytes']} "
            f"h3m={f['h3m']} gz={f['gz']}")


def read_kept():
    lines = {}
    if KEPT.exists():
        for line in KEPT.read_text().splitlines():
            if line and not line.startswith("#"):
                lines[line.split()[0]] = line
    return lines


def one_command(args):
    case = {k: getattr(args, k) for k in CHOICES if getattr(args, k) is not None}
    case["seed"] = args.seed
    if args.before:
        case["before"] = args.before.split(";")
    out = Path(args.out or OUT / "one")
    for k, (path, f) in enumerate(generate("one", case, out, again=args.again), 1):
        print(kept_line(f"one-{k}", f), path)
    return 0


def corpus_command(args):
    cases = tomllib.loads(CORPUS.read_text())["case"]
    only = set(args.only.split(",")) if args.only else None
    kept = read_kept()
    lines = dict(kept)
    failed = 0
    for case in cases:
        ident = case["id"]
        if only and ident not in only:
            continue
        out = Path(args.out or OUT) / ident
        try:
            # (what, facts): the first map, the same game's second, a fresh start's.
            runs = [(f"{'again' if k else 'first'}", f)
                    for k, (_, f) in enumerate(generate(ident, case, out / "run1", args.again))]
            if args.twice:
                runs.append(("twice", generate(ident, case, out / "run2")[0][1]))
        except Exception as e:  # noqa: BLE001 (a case's failure is reported, the rest go on)
            print(f"{ident}: FAILED: {e}")
            failed += 1
            continue
        line = kept_line(ident, runs[0][1])
        verdict = "ok" if kept.get(ident) == line else ("CHANGED" if ident in kept else "new")
        for what, f in runs[1:]:
            verdict += f", {what} " + ("same" if f == runs[0][1] else
                                       f"DIFFERS ({kept_line(ident, f)})")
        if not verdict.startswith(("ok", "new")) or "DIFFERS" in verdict:
            failed += 1
        print(f"{line}  {verdict}")
        lines[ident] = line
    if args.record:
        order = [c["id"] for c in cases]
        body = [lines[i] for i in order if i in lines]
        KEPT.write_text("# The oracle's maps (h3rmg.py corpus --record; ORACLE.md): digests only, "
                        "never the maps.\n" + "\n".join(body) + "\n")
    return 1 if failed else 0


def read_command(args):
    out = Path(args.out or OUT)
    maps = sorted(out.glob("*/run1/map1.h3m"))
    folder = out / "read"
    if folder.exists():
        shutil.rmtree(folder)
    (folder / "Maps").mkdir(parents=True)
    for path in maps:
        shutil.copy(path, folder / "Maps" / f"{path.parent.parent.name}.h3m")
    r = subprocess.run(["cargo", "run", "-q", "-p", "h3-map", "--example", "original_data", "--",
                        str(folder)], cwd=h3shell.ROOT, capture_output=True, text=True, check=False)
    lines = r.stdout.splitlines()
    read = None
    for line in lines:
        m = re.match(r"h3-map: (\d+) of (\d+) maps read", line)
        if m:
            read = int(m.group(1))
            print(line)
        elif re.match(r"h3-map: [^ ]+\.h3m: ", line):
            print(line)
    at = next((k for k, line in enumerate(lines) if "the import by outcome" in line), None)
    if at is None:
        print(r.stderr[-2000:])
        return 1
    print("h3-map: the import by outcome:")
    print("\n".join(lines[at + 1:]))
    return 0 if read == len(maps) and maps else 1


# The two template tables (survey §1.4), extracted from the archives (installation.md).
TABLES = {"ab": h3shell.DATA / "extracted" / "H3ab_bmp" / "Rmg.txt",
          "sod": h3shell.DATA / "extracted" / "H3bitmap" / "Rmg.txt"}


def templates_of(path):
    """rmg.txt's templates in row order (survey §2.1, "rmg.txt's columns"): name, least and
    greatest size index, and per zone (type, least and most human positions, least and most
    total). Tab-separated cells, a quoted one holding line breaks as the game reads it."""
    def is_set(row, k):
        return k < len(row) and row[k] != "" and not row[k].startswith(" ")

    def number(row, k):
        return int(row[k]) if is_set(row, k) else 0

    rows = list(csv.reader(io.StringIO(path.read_text(encoding="latin-1")), delimiter="\t"))
    found = []
    for row in rows[3:]:
        if is_set(row, 0) and len(row) >= 2:
            found.append((row[0], number(row, 1), number(row, 2), []))
        if found and len(row) >= 76 and is_set(row, 3):
            kind = 2  # none set: a treasure zone; the last set wins
            for k, t in ((4, 0), (5, 1), (6, 2), (7, 3)):
                if is_set(row, k):
                    kind = t
            found[-1][3].append((kind, *(number(row, k) for k in (9, 10, 11, 12))))
    return found


def drawn_template(templates, c, seed):
    """The template the seed draws for the choices `c` (no Random one), with the count kept."""
    side = SIZE_TILES[c["size"]]
    index = side * side * c["levels"] // 1296
    if c["water"] == "islands":
        index = max(index // 2, 1)
    players, total = c["players"], c["players"] + c["computers"]
    kept = []
    for name, least, most, zones in templates:
        if not least <= index <= most:
            continue
        fit = [z for z in zones if z[1] <= players <= z[2] and z[3] <= total <= z[4]]
        if (sum(1 for z in fit if z[0] == 0) >= players
                and sum(1 for z in fit if z[0] in (0, 1)) >= total):
            kept.append(name)
    state = (seed * 214013 + 2531011) & 0xFFFFFFFF  # the C library's rand() after srand(seed)
    return (kept[((state >> 16) & 0x7FFF) % len(kept)] if kept else None), len(kept)


def templates_command(_args):
    tables = {k: templates_of(p) for k, p in TABLES.items()}
    kept = read_kept()
    misses = 0
    for case in tomllib.loads(CORPUS.read_text())["case"]:
        ident = case["id"]
        m = re.search(r'template=("(?:[^"\\]|\\.)*")', kept.get(ident, ""))
        told = json.loads(m.group(1)) if m else None
        line = [f"{ident}: kept {json.dumps(told)}"]
        for k, templates in tables.items():
            name, count = drawn_template(templates, validate(case), int(case["seed"]))
            same = "=" if name == told else "differs"
            line.append(f"{k} {json.dumps(name)} of {count} {same}")
            if k == "sod" and name != told:
                misses += 1
        print("; ".join(line))
    print(f"SoD's table predicts {'every' if not misses else 'not every'} kept case"
          + (f" ({misses} missed)" if misses else ""))
    return 1 if misses else 0


def main(argv):
    p = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    sub = p.add_subparsers(dest="command", required=True)
    one = sub.add_parser("one")
    one.add_argument("--seed", type=int, required=True)
    one.add_argument("--size")
    one.add_argument("--levels", type=int)
    one.add_argument("--players", type=int)
    one.add_argument("--teams", type=int)
    one.add_argument("--computers", type=int)
    one.add_argument("--computer-teams", dest="computer_teams", type=int)
    one.add_argument("--water")
    one.add_argument("--monsters")
    one.add_argument("--before")
    one.add_argument("--again", action="store_true")
    one.add_argument("--out")
    corpus = sub.add_parser("corpus")
    corpus.add_argument("--only")
    corpus.add_argument("--twice", action="store_true")
    corpus.add_argument("--again", action="store_true")
    corpus.add_argument("--record", action="store_true")
    corpus.add_argument("--out")
    read = sub.add_parser("read")
    read.add_argument("--out")
    sub.add_parser("templates")
    args = p.parse_args(argv)
    return {"one": one_command, "corpus": corpus_command, "read": read_command,
            "templates": templates_command}[args.command](args)


if __name__ == "__main__":
    os.chdir(HERE)
    sys.exit(main(sys.argv[1:]))
