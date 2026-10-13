#!/usr/bin/env python3
"""Reference captures against our renders (E17 M1 T8, *provisional*; clients/README.md,
"Reference captures"). Every comparison is in the original's 5-6-5 screen format: a capture is an
XWD dump of the 16-bit X display (raw 5-6-5), a render is h3-view's PNG, whose channels widen 5-6-5
by repeating bits, so truncating them gives the 5-6-5 values back exactly.

  h3cap.py first <burst-dir>
      the opening frame of a battle.sh burst: the first frame after the largest change whose next
      frame differs in under 2 % of the pixels (the screen settled).
  h3cap.py diff <capture.xwd> <render.png> [--out diff.png]
      counts the differing pixels (all, the battlefield above y 556) and writes the capture dimmed
      with the differing pixels red.
  h3cap.py obstacles <capture.xwd> <base.png> <candidate.png>...
      for each candidate (a render of the base scenario plus one usual obstacle anchored at row 6,
      column 8, or one absolute obstacle), the hexes where its pixels match the capture best.

Needs numpy and ImageMagick's `convert`. Captures and renders of the original stay local
(ADR-0009): never commit them.
"""

import struct
import subprocess
import sys
import tomllib
from pathlib import Path

import numpy as np

LAYOUT = Path(__file__).resolve().parent.parent / "layouts" / "battle.toml"
FIELD_BOTTOM = 556  # the panel below (design §4)


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


def first(burst):
    frames = sorted(Path(burst).glob("f*.xwd"))
    pix = [xwd565(f) for f in frames]
    change = [float((a != b).mean()) for a, b in zip(pix, pix[1:])]
    jump = int(np.argmax(change)) + 1
    for i in range(jump, len(change)):
        if change[i] < 0.02:
            return frames[i]
    sys.exit(f"{burst}: the screen never settles")


def diff(capture, render, out=None):
    cap, ours = xwd565(capture), png565(render)
    differ = cap != ours
    print(
        f"differing: {int(differ.sum())} of {differ.size}; "
        f"battlefield (y < {FIELD_BOTTOM}): {int(differ[:FIELD_BOTTOM].sum())}"
    )
    if out:
        rgb = np.stack(
            [(cap >> 11) << 3, ((cap >> 5) & 63) << 2, (cap & 31) << 3], axis=-1
        ) // 3
        rgb[differ] = (255, 0, 0)
        subprocess.run(
            ["convert", "-size", f"{cap.shape[1]}x{cap.shape[0]}", "-depth", "8", "rgb:-", out],
            input=rgb.astype(np.uint8).tobytes(),
            check=True,
        )
    return differ


def obstacles(capture, base, candidates):
    layout = tomllib.loads(LAYOUT.read_text())
    (ox, oy), (sx, sy), shift = layout["hex_origin"], layout["hex_step"], layout["even_row_shift"]

    def hex_box(row, column):
        return ox + sx * column + (shift if row % 2 == 0 else 0), oy + sy * row

    cap, plain = xwd565(capture), png565(base)
    x0, y0 = hex_box(6, 8)
    for candidate in candidates:
        img = png565(candidate)
        ys, xs = np.nonzero(img != plain)
        if len(ys) == 0:
            print(f"{candidate}: draws nothing")
            continue
        here = float((cap[ys, xs] == img[ys, xs]).mean())
        scores = [(here, "as rendered")]
        for row in range(11):
            for column in range(17):
                hx, hy = hex_box(row, column)
                yy, xx = ys + hy - y0, xs + hx - x0
                inside = (yy >= 0) & (yy < FIELD_BOTTOM) & (xx >= 0) & (xx < cap.shape[1])
                if inside.sum() < 0.9 * len(ys):
                    continue
                same = cap[yy[inside], xx[inside]] == img[ys[inside], xs[inside]]
                scores.append((float(same.mean()), f"row {row} column {column}"))
        scores.sort(reverse=True)
        best = ", ".join(f"{where} {score:.4f}" for score, where in scores[:2])
        print(f"{candidate}: {len(ys)} pixels; best {best}")


def main(args):
    if len(args) >= 2 and args[0] == "first":
        print(first(args[1]))
    elif len(args) >= 3 and args[0] == "diff":
        out = args[4] if len(args) == 5 and args[3] == "--out" else None
        diff(args[1], args[2], out)
    elif len(args) >= 4 and args[0] == "obstacles":
        obstacles(args[1], args[2], args[3:])
    else:
        sys.exit(__doc__)


if __name__ == "__main__":
    main(sys.argv[1:])
