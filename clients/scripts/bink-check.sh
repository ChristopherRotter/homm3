#!/usr/bin/env bash
# Checks the Bink decoder `h3-bink` against the game's own BINKW32.DLL (E17 M6 T6c1, CLT-Q30;
# against FFmpeg from E17 M5 T4b2a until then): decodes every Bink clip of a video archive
# (default the repository's original-data/battle-videos.vid) and the synthetic samples of eight
# seeds with `h3-view video --original` (read as the DLL reads, CLT-Q68), decodes the same files
# with the DLL under Wine (scripts/bink-dll.py: binkprobe.exe at the game's x87 control word,
# YUY2), and compares them frame by frame (exit 1 on a difference). Since E27 M2 T2 also the
# shell's four Bink clips (original-data/shell-videos.vid: NWCLOGO, H3INTRO, LOSEGAME, H3X1intr)
# when no names are given, every clip's sound (its first track, `binkprobe sound`: the DLL's
# decoder on each frame's packet) and the sound decoder's tables (`binkprobe sound-tables`). Since
# E27 M2 T2b every frame as the DLL's BinkCopyToBuffer copies it for the game (`h3_bink::Target`:
# the cells its decoding marked changed, CLT-Q122), which the shell's clips need to match.
# About 10 minutes after the release build, most of it H3INTRO's 2 426 frames (about 2.5 GB
# written into clients/out/video: delete it after a run in a small sandbox).
#
#   clients/scripts/bink-check.sh [--vid <file>] [<name>...]
#
# Needs the capture harness's run directory and Wine prefix (clients/capture/setup.sh, the
# original data staged: BINKW32.DLL, binkprobe.exe). Writes into clients/out/video (gitignored):
# the files, ours (<stem>.yuv, <stem>.pcm) and the DLL's (dll/<stem>.yuv, dll/<stem>.pcm).
set -euo pipefail
cd "$(dirname "$0")/.."

out=out/video
samples=(--sample 1 --sample 2 --sample 3 --sample 4 --sample 5 --sample 6 --sample 7 --sample 8)
shell=(--vid ../original-data/shell-videos.vid NWCLOGO H3INTRO LOSEGAME H3X1intr)
cargo build --release --locked --quiet -p h3-battle-client
view=target/release/h3-view
rm -rf "$out"
"$view" video --original --out "$out" "$@" > /dev/null
if [ $# -eq 0 ] && [ -f ../original-data/shell-videos.vid ]; then
    "$view" video --original --out "$out" "${shell[@]}" > /dev/null
fi
"$view" video --original --out "$out" "${samples[@]}" > /dev/null
mkdir -p "$out/dll"
status=0
"$view" video --sound-tables "$out/tables"
python3 -I scripts/bink-dll.py --sound-tables "$out/dll/tables" || status=1
if cmp -s "$out/tables" "$out/dll/tables"; then
    echo "sound tables: as the reference"
else
    echo "sound tables: differ from the reference"
    status=1
fi
for file in "$out"/*.bik "$out"/*.BIK; do
    [ -e "$file" ] || continue
    stem=$(basename "${file%.*}")
    python3 -I scripts/bink-dll.py "$file" "$out/dll/$stem.yuv" || status=1
done
"$view" video --original --out "$out" --reference "$out/dll" "$@" || status=1
if [ $# -eq 0 ] && [ -f ../original-data/shell-videos.vid ]; then
    "$view" video --original --out "$out" --reference "$out/dll" "${shell[@]}" || status=1
fi
"$view" video --original --out "$out" --reference "$out/dll" "${samples[@]}" || status=1
exit $status
