#!/usr/bin/env python3
"""Reference captures against our renders (E17 M1 T8, M2 T1; *provisional*;
clients/capture/README.md). Every comparison is in the original's 5-6-5 screen format: a capture is
an XWD dump of the 16-bit X display (raw 5-6-5), a render is h3-view's PNG, whose channels widen
5-6-5 by repeating bits, so truncating them gives the 5-6-5 values back exactly.

  h3cap.py diff <capture.xwd> <render.png> [--record <render.jsonl>] [--mask <name,...>]
                [--out <diff.png>]
      counts the pixels that differ outside the masks and writes the capture dimmed with them red
      (the masked parts blue). Masks: `panel` (the bottom bar), `heroes` (the two corners where
      the heroes and their flags stand), `count_boxes` (each living stack's box, from the record's
      first snapshot: design §4.3).
  h3cap.py find <capture.xwd> <scenario.toml> [--out <dir>]
      finds the original's obstacles in a capture: renders the scenario without obstacles and
      with each obstacle type of its terrain alone (a usual one anchored at row 6, column 8; an
      absolute one where it stands), matches each against the capture at every anchor, and prints
      the scenario's `obstacles` lines for the matches (the diff then confirms them). On a special
      battlefield with obstacles of its own (the shore, a special ground's field: E17 M3 T2) the
      types are the field's, as the original's generator takes them.

Needs numpy, ImageMagick's `convert` and a release build of h3-view (`cargo build --release -p
h3-view` in clients/). Captures and renders of the original stay local (ADR-0009): never commit
them.
"""

import json
import re
import struct
import subprocess
import sys
import tempfile
import tomllib
from pathlib import Path

import numpy as np

CLIENTS = Path(__file__).resolve().parent.parent
ROOT = CLIENTS.parent
LAYOUT = CLIENTS / "layouts" / "battle.toml"
H3VIEW = CLIENTS / "target" / "release" / "h3-view"
FIELD_BOTTOM = 556  # the panel below (design §4)
WIDTH = 17  # hexes per row
# The heroes (design §4.5): a 150 × 175 frame at (-43, -19), mirrored at (693, -19) on the right;
# the flags at (29, 20) and (755, 20) lie inside these boxes.
HERO_BOXES = [(0, 0, 107, 156), (693, 0, 800, 156)]
COUNT_BOX = (30, 11)


def xwd565(path):
    """The pixels of an XWD dump of a 16-bit TrueColor display, as 5-6-5 values."""
    data = Path(path).read_bytes()
    head = struct.unpack(">25I", data[:100])
    header_size, depth, width, height = head[0], head[3], head[4], head[5]
    byte_order, bytes_per_line, colours = head[7], head[12], head[19]
    if depth != 16 or byte_order != 0 or head[14:17] != (0xF800, 0x7E0, 0x1F):
        sys.exit(f"{path}: not a little-endian 5-6-5 dump")
    start = header_size + 12 * colours
    rows = np.frombuffer(data, np.uint8, height * bytes_per_line, start)
    rows = rows.reshape(height, bytes_per_line)[:, : 2 * width]
    return rows.copy().view("<u2").astype(np.uint16)


def png565(path):
    """A render's pixels, truncated back to 5-6-5."""
    out = subprocess.run(
        ["convert", str(path), "-depth", "8", "rgb:-"], check=True, capture_output=True
    ).stdout
    rgb = np.frombuffer(out, np.uint8).reshape(-1, 800, 3).astype(np.uint16)
    return ((rgb[..., 0] >> 3) << 11) | ((rgb[..., 1] >> 2) << 5) | (rgb[..., 2] >> 3)


def layout():
    return tomllib.loads(LAYOUT.read_text())


def hex_box(row, column, lay=None):
    """A hex's box's top-left (design §4.1)."""
    lay = lay or layout()
    (ox, oy), (sx, sy), shift = lay["hex_origin"], lay["hex_step"], lay["even_row_shift"]
    return ox + sx * column + (shift if row % 2 == 0 else 0), oy + sy * row


def snapshot(record):
    """The first snapshot's observation of a record (JSON lines)."""
    for line in Path(record).read_text().splitlines():
        message = json.loads(line)
        if isinstance(message, dict) and "Snapshot" in message:
            return message["Snapshot"]["observation"]
    sys.exit(f"{record}: no snapshot")


def count_boxes(observation, showing=None):
    """Each living stack's count box (design §4.3, 0x43E140 part 1): (x0, y0, x1, y1); with
    `showing`, only the boxes of those stack ids (every living stack still takes its hexes). A
    stack faces as its side does unless it carries `right` (a turned stack, M3 T1a)."""
    lay = layout()
    ax, ay = lay["anchor"]
    stacks = [s for s in observation["stacks"] if s["alive"] and not s["war_machine"]]
    facing = {s["id"]: s.get("right", s["side"] == "Attacker") for s in stacks}
    taken = {h for s in stacks
             for h in (s["hex"], s["hex"] + (1 if facing[s["id"]] else -1) * s["two_hex"])}
    taken |= set(observation.get("obstacles", []))
    boxes = []
    for s in stacks:
        if showing is not None and s["id"] not in showing:
            continue
        row, column = divmod(s["hex"], WIDTH)
        x, y = hex_box(row, column, lay)
        x, y = x + ax, y + ay
        w = 1 if s["two_hex"] else 0
        right = facing[s["id"]]
        ahead = s["hex"] + (1 + w if right else -1 - w)
        if ahead in taken:
            dx, dy = (-15 + 44 * w, -15) if right else (-15 - 44 * w, -15)
        else:
            dx, dy = (22 + 44 * w, -15) if right else (-52 - 44 * w, -30)
        boxes.append((x + dx, y + dy, x + dx + COUNT_BOX[0], y + dy + COUNT_BOX[1]))
    return boxes


def boxes_at(observation, poses):
    """The count boxes shown with the stacks' poses `poses` ({id: (group, hex, faces left)}): a
    living stack standing or fidgeting (group 2 or 1) shows its box at its hex by its facing
    (design §4.3: 0x43E140 skips the others); every one takes its hexes; `observation` gives each
    one's side and size and the obstacles."""
    moved = [dict(s, hex=poses[s["id"]][1], right=not poses[s["id"]][2])
             for s in observation["stacks"] if s["alive"] and s["id"] in poses]
    showing = {i for i, (group, _, _) in poses.items() if group in (1, 2)}
    return count_boxes({**observation, "stacks": moved}, showing)


def mask(names, record=None, shape=(600, 800)):
    """The masked pixels: what we do not draw yet."""
    m = np.zeros(shape, bool)
    boxes = []
    for name in names:
        if name == "panel":
            boxes.append((0, FIELD_BOTTOM, shape[1], shape[0]))
        elif name == "heroes":
            boxes += HERO_BOXES
        elif name == "count_boxes":
            if record is None:
                sys.exit("the mask `count_boxes` needs --record")
            boxes += count_boxes(snapshot(record))
        elif name:
            sys.exit(f"unknown mask `{name}`")
    for x0, y0, x1, y1 in boxes:
        m[max(y0, 0) : max(y1, 0), max(x0, 0) : max(x1, 0)] = True
    return m


def diff(capture, render, record=None, masks=(), out=None, quiet=False):
    """The pixels that differ outside the masks; prints the count and where they lie."""
    cap, ours = xwd565(capture), png565(render)
    n, where = compare(cap, ours, mask(masks, record, cap.shape), out)
    if not quiet:
        print(f"differing: {n} of {int((~mask(masks, record, cap.shape)).sum())} unmasked pixels{where}")
    return n


def compare(cap, ours, masked, out=None, redrawn=None):
    """The count of pixels of `cap` and `ours` (5-6-5 arrays) that differ outside `masked`, and
    where they lie (text); with `out`, writes the capture dimmed with them red, the masked parts
    blue and the differing pixels `redrawn` marks (`redraws`) yellow."""
    differ = (cap != ours) & ~masked
    n = int(differ.sum())
    where = ""
    if n:
        ys, xs = np.nonzero(differ)
        where = f" in x {xs.min()}..{xs.max()}, y {ys.min()}..{ys.max()}"
    if out:
        rgb = np.stack(
            [(cap >> 11) << 3, ((cap >> 5) & 63) << 2, (cap & 31) << 3], axis=-1
        ) // 3
        rgb[masked] = rgb[masked] // 2 + (0, 0, 96)
        rgb[differ] = (255, 0, 0)
        if redrawn is not None:
            rgb[differ & redrawn] = (255, 255, 0)
        subprocess.run(
            ["convert", "-size", f"{cap.shape[1]}x{cap.shape[0]}", "-depth", "8", "rgb:-", out],
            input=rgb.astype(np.uint8).tobytes(),
            check=True,
        )
    return n, where


# -- redraw artifacts (M3 T1c) ------------------------------------------------------------------
# The original redraws by dirty rectangles and draws each item that meets one whole, over pixels
# it did not restore, so a shadow already there is darkened again (design §7.11). The viewer draws
# every moment whole and does not copy this (the user, 2026-10-14); the frame check counts such
# pixels apart instead of failing on them.


def shadow_body(d):
    """A shadow's body on a 5-6-5 pixel (0x47C9E0, 0x47C460's 5-6-5 masks; design §3.3)."""
    return (d >> 1) & 0x7BEF


def shadow_border(d):
    """A shadow's border on a 5-6-5 pixel (design §3.3)."""
    return ((d >> 2) & 0x39E7) + ((d >> 1) & 0x7BEF)


# *provisional*: M3 T1c's `melee` and `behind` needed four; since the heroes are drawn (M3 T3a)
# the left hero's shadow under a walker's redrawn rectangles takes five (`turn` and `climb`).
MAX_SHADOWS = 5


def shadowed(cap, base):
    """Where `cap` is `base` with one to MAX_SHADOWS more shadows, body or border in any order
    (arrays of 5-6-5 values of one shape)."""
    hit = np.zeros(cap.shape, bool)
    level = [base.astype(np.uint16)]
    for _ in range(MAX_SHADOWS):
        level = [f(d) for d in level for f in (shadow_body, shadow_border)]
        for d in level:
            hit |= cap == d
    return hit


def redraws(cap, ours, differ, earlier=()):
    """Which of the `differ` pixels are the original's redraw artifacts: its pixel is the viewer's
    at the same moment with one to MAX_SHADOWS more shadows, or one the viewer drew there at an
    `earlier` moment of the same sequence with at least one more (a rectangle the original left
    unrestored; a timing slip of the viewer's shows no more shadows, so it still fails). Any
    other difference is no artifact."""
    ys, xs = np.nonzero(differ)
    theirs = cap[ys, xs]
    hit = shadowed(theirs, ours[ys, xs])
    for before in earlier:
        if hit.all():
            break
        hit |= shadowed(theirs, before[ys, xs])
    out = np.zeros(differ.shape, bool)
    out[ys[hit], xs[hit]] = True
    return out


# -- finding the obstacles ----------------------------------------------------------------------
PACKS = ("base", "ab", "sod")


def pack_obstacles(terrain, battlefield=None):
    """The obstacle types of `terrain` (the base pack's), or of the special battlefield
    `battlefield` (`pack:id`, any pack's types listing it; E17 M3 T2), each with its pack's id in
    `pack`."""
    found = []
    for pack in PACKS:
        path = ROOT / "packs" / pack / "battle.toml"
        if not path.exists():
            continue
        for o in tomllib.loads(path.read_text()).get("obstacle", []):
            if battlefield:
                field_pack, field = battlefield.split(":")
                names = {f if ":" in f else f"{pack}:{f}" for f in o.get("battlefields", [])}
                hit = f"{field_pack}:{field}" in names
            else:
                hit = pack == "base" and terrain in o.get("terrains", [])
            if hit:
                found.append({**o, "pack": pack})
    return found


def battlefields(key):
    """The special battlefield `key` (`pack:id`) as its pack defines it, in a list (none if no
    pack does)."""
    pack, field = key.split(":")
    path = ROOT / "packs" / pack / "battle.toml"
    if not path.exists():
        return []
    return [b for b in tomllib.loads(path.read_text()).get("battlefield", []) if b["id"] == field]


def usual_hexes(o, anchor):
    """The hexes a usual obstacle blocks at `anchor` (BAT-GEO-004, h3-battle's `placement`)."""
    y, x = divmod(anchor, WIDTH)
    hexes = []
    for dx, dy in o["blocked"]:
        ty = y + dy
        tx = x + dx - (1 if y % 2 == 1 and ty % 2 == 0 else 0)
        hexes.append(ty * WIDTH + tx)
    return hexes


def entry(o, anchor=None):
    """A scenario's obstacle entry."""
    kind = f'{o.get("pack", "base")}:{o["id"]}'
    if o.get("absolute"):
        hexes = [y * WIDTH + x for x, y in o["blocked"]]
        return f'{{ hexes = {hexes}, kind = "{kind}" }}'
    return f'{{ hexes = {usual_hexes(o, anchor)}, anchor = {anchor}, kind = "{kind}" }}'


def with_obstacles(text, entries):
    """The scenario `text` with its obstacles replaced by `entries`."""
    text = re.sub(r"(?ms)^obstacles = \[.*?^\]\n|^obstacles = \[.*?\]\n|^random_obstacles = .*\n", "", text)
    return text.replace("[scenario]\n", "[scenario]\nobstacles = [" + ", ".join(entries) + "]\n", 1)


def render(text, out):
    scenario = out.with_suffix(".toml")
    scenario.write_text(text)
    subprocess.run(
        [str(H3VIEW), "render", str(scenario), "--out", str(out)],
        check=True,
        capture_output=True,
    )
    return png565(out)


def find(capture, scenario, out_dir=None):
    text = Path(scenario).read_text()
    head = tomllib.loads(text)["scenario"]
    terrain = head["terrain"].split(":")[-1]
    field = head.get("battlefield")
    own = field and any(f.get("own_obstacles") for f in battlefields(field))
    out = Path(out_dir or tempfile.mkdtemp(prefix="h3cap-"))
    out.mkdir(parents=True, exist_ok=True)
    cap = xwd565(capture)
    plain = render(with_obstacles(text, []), out / "plain.png")
    lay = layout()
    centre = 6 * WIDTH + 8
    x0, y0 = hex_box(6, 8, lay)
    found = []
    for o in pack_obstacles(terrain, field if own else None):
        anchor = None if o.get("absolute") else centre
        img = render(with_obstacles(text, [entry(o, anchor)]), out / f"{o['id']}.png")
        ys, xs = np.nonzero(img != plain)
        if len(ys) == 0:
            continue
        if anchor is None:
            score = float((cap[ys, xs] == img[ys, xs]).mean())
            found.append((score, o, None))
            continue
        best = (0.0, None)
        for row in range(1, 10):
            for column in range(1, 16):
                hx, hy = hex_box(row, column, lay)
                yy, xx = ys + hy - y0, xs + hx - x0
                inside = (yy >= 0) & (yy < FIELD_BOTTOM) & (xx >= 0) & (xx < cap.shape[1])
                if inside.sum() < 0.9 * len(ys):
                    continue
                same = float((cap[yy[inside], xx[inside]] == img[ys[inside], xs[inside]]).mean())
                if same > best[0]:
                    best = (same, row * WIDTH + column)
        found.append((best[0], o, best[1]))
    found.sort(key=lambda f: -f[0])
    for score, o, anchor in found[:8]:
        where = "absolute" if anchor is None else f"anchor {anchor}"
        print(f"# {o['id']}: {score:.4f} at {where}")
    # An obstacle partly covered by another still scores above 0.9 (the dirt situation's 0.95); a
    # look-alike at another's place scores about 0.5.
    matches = [entry(o, a) for score, o, a in found if score >= 0.9]
    print("obstacles = [\n" + "".join(f"    {e},\n" for e in matches) + "]")
    return matches


def main(args):
    if len(args) >= 3 and args[0] == "diff":
        opts = dict(zip(args[3::2], args[4::2]))
        masks = [m for m in opts.get("--mask", "").split(",") if m]
        diff(args[1], args[2], opts.get("--record"), masks, opts.get("--out"))
    elif len(args) >= 3 and args[0] == "find":
        opts = dict(zip(args[3::2], args[4::2]))
        find(args[1], args[2], opts.get("--out"))
    else:
        sys.exit(__doc__)


if __name__ == "__main__":
    main(sys.argv[1:])
