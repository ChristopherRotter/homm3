#!/usr/bin/env python3
"""The game's BINKW32.DLL decoding of a Bink file as raw YUV 4:2:0 (E17 M6 T6c1, CLT-Q30): the
reference `h3-view video --reference` compares `h3-bink` with (`clients/scripts/bink-check.sh`).

Runs `binkprobe.exe frames` (clients/capture/binkprobe.c: the DLL as a black box, at the game's
x87 control word 0x27F) under Wine in the capture harness's run directory, on a one-entry video
archive holding the file (the probe opens clips as the game does, from an archive), with the
surface type 7 (YUY2: the DLL's own planes, no colour conversion). YUY2 gives each row its
chroma pair by pair; the 4:2:0 planes take the even rows' (each odd row's is checked equal to
the row above: the DLL copies a chroma row into both of its rows). The picture's rows only, as
FFmpeg's reference had them.

    python3 -I clients/scripts/bink-dll.py <in.bik> <out.yuv>

The run directory is $H3_RUN/game (default original-data/wine/game), the Wine prefix
$WINEPREFIX (default ~/.wine-h3): `clients/capture/setup.sh` makes both.
"""

import os
import struct
import subprocess
import sys
import tempfile


def archive(name, data):
    """A video archive of one entry: its count, a 40-byte name and the offset, then the file."""
    entry = name.encode("ascii")[:39].ljust(40, b"\0")
    return struct.pack("<I", 1) + entry + struct.pack("<I", 4 + 44) + data


def to_yuv420(yuy2, width, height):
    """The YUY2 frames' planes (Y, then U and V of the even rows); fails where an odd row's
    chroma differs from the row above's."""
    frame = width * height * 2
    if len(yuy2) % frame:
        sys.exit(f"bink-dll: {len(yuy2)} bytes are not whole {width} x {height} frames")
    out = bytearray()
    for f in range(len(yuy2) // frame):
        rows = [yuy2[f * frame + y * width * 2 : f * frame + (y + 1) * width * 2] for y in range(height)]
        out += b"".join(row[0::2] for row in rows)
        for k in (1, 3):
            chroma = [row[k::4] for row in rows]
            for y in range(1, height, 2):
                if chroma[y] != chroma[y - 1]:
                    sys.exit(f"bink-dll: frame {f}: row {y}'s chroma is not row {y - 1}'s")
            out += b"".join(chroma[0::2])
    return bytes(out)


def main():
    if len(sys.argv) != 3:
        sys.exit("usage: bink-dll.py <in.bik> <out.yuv>")
    source, target = sys.argv[1:]
    root = os.path.realpath(os.path.join(os.path.dirname(__file__), "..", ".."))
    run = os.path.realpath(os.environ.get("H3_RUN", os.path.join(root, "original-data", "wine")))
    game = os.path.join(run, "game")
    if not os.path.exists(os.path.join(game, "binkprobe.exe")):
        sys.exit(f"bink-dll: no binkprobe.exe in {game}: run clients/capture/setup.sh")
    env = dict(os.environ)
    env.setdefault("WINEPREFIX", os.path.expanduser("~/.wine-h3"))
    env.update(WINEARCH="win32", WINEDEBUG="-all")
    data = open(source, "rb").read()
    name = os.path.basename(source)
    with tempfile.TemporaryDirectory(dir=game) as tmp:
        vid = os.path.join(tmp, "clip.vid")
        out = os.path.join(tmp, "frames.yuy2")
        with open(vid, "wb") as f:
            f.write(archive(name, data))
        rel = os.path.relpath(tmp, game)
        # Once in about a hundred runs Wine failed to start the probe (no output, no message;
        # the same file then ran): a second try (E17 M6 T6c1).
        for attempt in range(2):
            done = subprocess.run(
                ["wine", "binkprobe.exe", "frames", f"{rel}\\clip.vid", name, "7",
                 f"{rel}\\frames.yuy2"],
                cwd=game, env=env, capture_output=True, text=True,
            )
            if done.returncode == 0 and os.path.exists(out):
                break
        else:
            sys.exit(f"bink-dll: binkprobe failed on {name} (exit {done.returncode}): "
                     f"{done.stderr.strip()}")
        width, height = struct.unpack("<II", data[20:28])
        yuv = to_yuv420(open(out, "rb").read(), width, height)
    with open(target, "wb") as f:
        f.write(yuv)


if __name__ == "__main__":
    main()
