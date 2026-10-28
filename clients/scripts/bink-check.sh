#!/usr/bin/env bash
# Checks the Bink decoder `h3-bink` against FFmpeg (E17 M5 T4b2a, CLT-Q29): decodes every Bink
# clip of a video archive (default the repository's original-data/battle-videos.vid) and the
# synthetic samples of eight seeds with `h3-view video`, decodes the same files with FFmpeg into
# raw YUV 4:2:0 (scripts/bink-reference.py: each frame released before the next, why there),
# and compares them frame by frame (exit 1 on a difference). About 5 minutes, most of it the
# release build.
#
#   clients/scripts/bink-check.sh [--vid <file>] [<name>...]
#
# Needs PyAV (`pip install --break-system-packages av`: FFmpeg's libraries). Writes into
# clients/out/video (gitignored): the files, ours (<stem>.yuv) and FFmpeg's (ffmpeg/<stem>.yuv).
set -euo pipefail
cd "$(dirname "$0")/.."

out=out/video
samples=(--sample 1 --sample 2 --sample 3 --sample 4 --sample 5 --sample 6 --sample 7 --sample 8)
cargo build --release --locked --quiet -p h3-view
view=target/release/h3-view
rm -rf "$out"
"$view" video --out "$out" "$@" > /dev/null
"$view" video --out "$out" "${samples[@]}" > /dev/null
mkdir -p "$out/ffmpeg"
for file in "$out"/*.bik "$out"/*.BIK; do
    [ -e "$file" ] || continue
    stem=$(basename "${file%.*}")
    python3 -I scripts/bink-reference.py "$file" "$out/ffmpeg/$stem.yuv"
done
status=0
"$view" video --out "$out" --reference "$out/ffmpeg" "$@" || status=1
"$view" video --out "$out" --reference "$out/ffmpeg" "${samples[@]}" || status=1
exit $status
