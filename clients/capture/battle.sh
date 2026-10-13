#!/usr/bin/env bash
# Captures a battle's opening in the original (E17 M1 T8, *provisional*; clients/README.md,
# "Reference captures"), after setup.sh. The map's first player has a hero selected that stands
# just west of an enemy hero (the battle lane's smoke map does:
# `cargo run -p h3-map --example smoke_map -- original-data/extracted/H3bitmap/Objects.txt <map>`).
# The script starts a single scenario on it, steps the hero east into the enemy (the arrow keys move
# the selected hero) and writes a burst of frames as XWD dumps: the X display runs at 16 bits, so
# each is the original's own 5-6-5 screen. `h3cap.py first` picks the opening frame.
#
#   clients/capture/battle.sh <map.h3m> <out-dir> [frames, default 40]
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd)"
map="$1" out="$2" frames="${3:-40}"
game="$root/original-data/wine/game"
export WINEPREFIX="${WINEPREFIX:-$HOME/.wine-h3}" WINEDEBUG=-all DISPLAY=:99
if ! pgrep -f "Xvfb :99" > /dev/null; then
    Xvfb :99 -screen 0 800x600x16 -nolisten tcp > /dev/null 2>&1 &
    sleep 1
fi
# The only map, so the scenario list selects it.
find "$game/Maps" -maxdepth 1 -type f -delete
cp "$map" "$game/Maps/"
mkdir -p "$out"
(cd "$game" && wine Heroes3.exe > "$out/wine.log" 2>&1 &)
click() {
    xdotool mousemove "$1" "$2"
    sleep 0.2
    xdotool click 1
    sleep "$3"
}
sleep "${H3_START_WAIT:-15}" # the main menu
click 640 75 2               # New Game
click 640 75 4               # Single Scenario
click 497 553 7              # Begin
# Away from the screen's edges: a cursor there scrolls the adventure map.
xdotool mousemove 700 300
sleep 0.5
xdotool key Right          # into the enemy: the battle starts
xdotool mousemove 420 580  # over the panel, off the battlefield
for i in $(seq -w 1 "$frames"); do
    xwd -root -silent > "$out/f$i.xwd"
    sleep 0.1
done
wineserver -k
echo "battle: $frames frames in $out"
