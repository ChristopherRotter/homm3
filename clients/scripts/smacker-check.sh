#!/usr/bin/env bash
# Checks the Smacker decoder `h3-smacker` against the game's own SMACKW32.DLL (E27 M2 T1): decodes
# every Smacker clip of the shell's video archive (default the repository's
# original-data/shell-videos.vid), AB's Credits.smk (original-data/install/Data/H3ab_ahd.vid,
# when staged) and the synthetic samples of eight seeds with `h3-view video --smacker`, decodes the
# same files with the DLL under Wine (scripts/smacker-dll.py: smackprobe.exe, 8-bit and 5-6-5),
# and compares them frame by frame: the colour indices and palettes, and the original's 5-6-5 copy
# (exit 1 on a difference). About 5 minutes after the release build, most of it H3INTRO.SMK
# (2 426 frames, decoded by the DLL twice); about 5 GB written (H3INTRO's 16-bit frames alone 1.1 GB
# each side): delete clients/out/smacker after a run in a small sandbox.
#
#   clients/scripts/smacker-check.sh [--vid <file>] [<name>...]
#
# Needs the capture harness's run directory and Wine prefix (clients/capture/setup.sh, the
# original data staged: SMACKW32.DLL, smackprobe.exe). Writes into clients/out/smacker
# (gitignored): the files, ours (<stem>.pal8, <stem>.565) and the DLL's (dll/<stem>.*).
set -euo pipefail
cd "$(dirname "$0")/.."

out=out/smacker
samples=(--sample 1 --sample 2 --sample 3 --sample 4 --sample 5 --sample 6 --sample 7 --sample 8)
ab=../original-data/install/Data/H3ab_ahd.vid
cargo build --release --locked --quiet -p h3-battle-client
view=target/release/h3-view
rm -rf "$out"
"$view" video --smacker --out "$out" "$@" > /dev/null
if [ $# -eq 0 ] && [ -f "$ab" ]; then
    "$view" video --smacker --out "$out" --vid "$ab" > /dev/null
fi
"$view" video --smacker --out "$out" "${samples[@]}" > /dev/null
status=0
for file in "$out"/*.smk "$out"/*.SMK; do
    [ -e "$file" ] || continue
    python3 -I scripts/smacker-dll.py "$file" "$out/dll" || status=1
done
"$view" video --smacker --out "$out" --reference "$out/dll" "$@" || status=1
if [ $# -eq 0 ] && [ -f "$ab" ]; then
    "$view" video --smacker --out "$out" --reference "$out/dll" --vid "$ab" || status=1
fi
"$view" video --smacker --out "$out" --reference "$out/dll" "${samples[@]}" || status=1
exit $status
