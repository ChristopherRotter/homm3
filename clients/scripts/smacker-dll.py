#!/usr/bin/env python3
"""The game's SMACKW32.DLL decoding of a Smacker file (E27 M2 T1): the reference
`h3-view video --smacker --reference` compares `h3-smacker` with
(`clients/scripts/smacker-check.sh`).

Runs `smackprobe.exe frames` (clients/capture/smackprobe.c: the DLL as a black box, opened as
the game opens a clip) under Wine in the capture harness's run directory, on a one-entry video
archive holding the file (the probe opens clips as the game does, from an archive), twice: with
the buffer type 0 (8 bits: the frames' colour indices) into <out>/<stem>.pal8, and with the
game's 5-6-5 type (0xC0000000) into <out>/<stem>.565; each frame's buffer followed by the
palette the DLL holds (768 bytes), the formats `h3_smacker::check::raw` and `original_16` write.
Then (E27 M2 T2) `smackprobe.exe audio` with the stand-in Miles (clients/capture/mss32.c): the
samples the DLL hands Miles for each track the header describes, into <out>/<stem>.t<k>.pcm
(`h3_smacker::check::sound`'s format); the DLL takes a sample handle per such track in their
order, so the n-th handle's file is the n-th described track's.

    python3 -I clients/scripts/smacker-dll.py <in.smk> <out dir>

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


def run(command, game, env, out):
    """Runs the probe until it leaves `out`; a second try where Wine failed to start it, as
    bink-dll.py (E17 M6 T6c1)."""
    for attempt in range(2):
        done = subprocess.run(command, cwd=game, env=env, capture_output=True, text=True)
        if done.returncode == 0 and os.path.exists(out):
            return
    sys.exit(f"smacker-dll: {' '.join(command[1:3])} failed on {command[4]} "
             f"(exit {done.returncode}): {done.stderr.strip()}")


def main():
    if len(sys.argv) != 3:
        sys.exit("usage: smacker-dll.py <in.smk> <out dir>")
    source, target = sys.argv[1:]
    root = os.path.realpath(os.path.join(os.path.dirname(__file__), "..", ".."))
    run_dir = os.path.realpath(os.environ.get("H3_RUN", os.path.join(root, "original-data", "wine")))
    game = os.path.join(run_dir, "game")
    if not os.path.exists(os.path.join(game, "smackprobe.exe")):
        sys.exit(f"smacker-dll: no smackprobe.exe in {game}: run clients/capture/setup.sh")
    env = dict(os.environ)
    env.setdefault("WINEPREFIX", os.path.expanduser("~/.wine-h3"))
    env.update(WINEARCH="win32", WINEDEBUG="-all")
    data = open(source, "rb").read()
    name = os.path.basename(source)
    stem = os.path.splitext(name)[0]
    os.makedirs(target, exist_ok=True)
    with tempfile.TemporaryDirectory(dir=game) as tmp:
        with open(os.path.join(tmp, "clip.vid"), "wb") as f:
            f.write(archive(name, data))
        rel = os.path.relpath(tmp, game)
        for kind, extension in (("0", "pal8"), ("C0000000", "565")):
            out = os.path.join(tmp, f"frames.{extension}")
            run(["wine", "smackprobe.exe", "frames", f"{rel}\\clip.vid", name, kind,
                 f"{rel}\\frames.{extension}"], game, env, out)
            with open(out, "rb") as f, open(os.path.join(target, f"{stem}.{extension}"), "wb") as g:
                g.write(f.read())
        tracks = [k for k in range(7) if len(data) >= 76 + 4 * k
                  and struct.unpack_from("<I", data, 72 + 4 * k)[0] & 0xFFFFFF]
        if tracks:
            run(["wine", "smackprobe.exe", "audio", f"{rel}\\clip.vid", name,
                        f"{rel}\\sound"], game, env, os.path.join(tmp, "sound.log"))
            for n, k in enumerate(tracks):
                pcm = os.path.join(tmp, f"sound.{n}.pcm")
                with open(os.path.join(target, f"{stem}.t{k}.pcm"), "wb") as g:
                    if os.path.exists(pcm):
                        with open(pcm, "rb") as f:
                            g.write(f.read())


if __name__ == "__main__":
    main()
