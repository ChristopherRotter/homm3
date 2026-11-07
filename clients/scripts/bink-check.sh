#!/usr/bin/env bash
# Checks the Bink decoder `h3-bink` against the game's own BINKW32.DLL (E17 M6 T6c1, CLT-Q30;
# against FFmpeg from E17 M5 T4b2a until then): decodes every Bink clip of a video archive
# (default the repository's original-data/battle-videos.vid) and the synthetic samples of eight
# seeds with `h3-view video --original` (read as the DLL reads, CLT-Q68), decodes the same files
# with the DLL under Wine (scripts/bink-dll.py: binkprobe.exe at the game's x87 control word,
# YUY2), and compares them frame by frame (exit 1 on a difference). About 5 minutes, most of it
# the release build; the DLL takes about 3 seconds a file.
#
#   clients/scripts/bink-check.sh [--vid <file>] [<name>...]
#
# Needs the capture harness's run directory and Wine prefix (clients/capture/setup.sh, the
# original data staged: BINKW32.DLL, binkprobe.exe). Writes into clients/out/video (gitignored):
# the files, ours (<stem>.yuv) and the DLL's (dll/<stem>.yuv).
set -euo pipefail
cd "$(dirname "$0")/.."

out=out/video
samples=(--sample 1 --sample 2 --sample 3 --sample 4 --sample 5 --sample 6 --sample 7 --sample 8)
cargo build --release --locked --quiet -p h3-view
view=target/release/h3-view
rm -rf "$out"
"$view" video --original --out "$out" "$@" > /dev/null
"$view" video --original --out "$out" "${samples[@]}" > /dev/null
mkdir -p "$out/dll"
status=0
for file in "$out"/*.bik "$out"/*.BIK; do
    [ -e "$file" ] || continue
    stem=$(basename "${file%.*}")
    python3 -I scripts/bink-dll.py "$file" "$out/dll/$stem.yuv" || status=1
done
"$view" video --original --out "$out" --reference "$out/dll" "$@" || status=1
"$view" video --original --out "$out" --reference "$out/dll" "${samples[@]}" || status=1
exit $status
