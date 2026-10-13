#!/usr/bin/env bash
# The reference-capture setup (E17 M1 T8, M2 T1; *provisional*; clients/capture/README.md).
# Runs a copy of the original game under Wine on a virtual X display at 16 bits, so a capture is
# the original's own 5-6-5 screen. Linux only (a cloud sandbox); never touches the installation.
#
#   clients/capture/setup.sh [INSTALL_COPY]
#
# INSTALL_COPY (default original-data/install) must hold Heroes3.exe, its five DLLs (BINKW32.DLL,
# IFC20.dll, MP3DEC.ASI, MSS32.DLL, SMACKW32.DLL, copied from the installation) and Data/ with the
# four LODs, Heroes3.snd, H3ab_ahd.snd and H3ab_ahd.vid. VIDEO.VID (intro videos only) is replaced
# by an empty archive. The run directory is original-data/wine/game (gitignored), the Wine prefix
# $WINEPREFIX (default ~/.wine-h3).
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd)"
install="${1:-$root/original-data/install}"
run="$root/original-data/wine"
export WINEPREFIX="${WINEPREFIX:-$HOME/.wine-h3}" WINEARCH=win32 WINEDEBUG=-all
export WINEDLLOVERRIDES="mscoree,mshtml="

# 1. Packages: 32-bit Wine, the virtual display and input, mingw-w64 for the forwarder.
if ! command -v wine > /dev/null || ! command -v Xvfb > /dev/null; then
    dpkg --add-architecture i386
    apt-get update -qq
    DEBIAN_FRONTEND=noninteractive apt-get install -y -qq --no-install-recommends \
        wine wine32:i386 wine64 xvfb xdotool x11-apps imagemagick python3-numpy \
        gcc-mingw-w64-i686 binutils-mingw-w64-i686 > /dev/null
fi

# 2. The run directory: a copy, so the game may write its saves and settings there.
game="$run/game"
mkdir -p "$game/Data" "$game/Maps" "$game/Games" "$game/MP3"
cp "$install/Heroes3.exe" "$install/BINKW32.DLL" "$install/IFC20.dll" "$install/MP3DEC.ASI" \
    "$install/MSS32.DLL" "$install/SMACKW32.DLL" "$game/"
for f in H3bitmap.lod H3sprite.lod H3ab_bmp.lod H3ab_spr.lod Heroes3.snd H3ab_ahd.snd \
    H3ab_ahd.vid; do
    cp "$install/Data/$f" "$game/Data/"
done
printf '\0\0\0\0' > "$game/Data/VIDEO.VID"
chmod -R u+w "$game"

# 3. xdd.dll: DirectDrawCreate forwarded to Wine's ddraw, and the virtual clock (xdd.c).
i686-w64-mingw32-gcc -O2 -Wall -shared -o "$game/xdd.dll" "$root/clients/capture/xdd.c" \
    "$root/clients/capture/xdd.def" -Wl,--enable-stdcall-fixup

# 4. The prefix, drive D: = the run directory, and the game's settings (the value names are the
#    executable's): no intro, no combat grid, hex shade or movement shade, full screen, no sound.
if [ ! -f "$WINEPREFIX/system.reg" ]; then
    Xvfb :98 -screen 0 800x600x16 -nolisten tcp & xvfb=$!
    DISPLAY=:98 wineboot -i > /dev/null 2>&1
    kill "$xvfb"
fi
ln -sfn "$game" "$WINEPREFIX/dosdevices/d:"
# The key's name holds the registered sign in Windows-1252 (0xAE).
key="HKEY_LOCAL_MACHINE\\SOFTWARE\\New World Computing\\Heroes of Might and Magic$(printf '\xae') III\\1.0"
{
    printf 'REGEDIT4\r\n\r\n[%s]\r\n"AppPath"="D:\\\\"\r\n"CDDrive"="D:\\\\"\r\n' "$key"
    for v in "Show Intro" "Show Combat Grid" "Show Combat Mouse Hex" "Combat Shade Level" \
        "Quick Combat" "Move Reminder" "Autosave" "Music Volume" "Sound Volume" \
        "Last Music Volume" "Last Sound Volume" "Video Subtitles"; do
        printf '"%s"=dword:00000000\r\n' "$v"
    done
    # The adventure map shows a hero's path on the first click and walks on the second, as the
    # experiments' procedures say (left at 0, the first click walks at once).
    printf '"Show Route"=dword:00000001\r\n'
    printf '"Main Game Full Screen"=dword:00000001\r\n\r\n'
    printf '[HKEY_CURRENT_USER\\Software\\Wine\\Drivers]\r\n"Audio"=""\r\n'
} > "$run/h3.reg"
wine regedit "$run/h3.reg" > /dev/null 2>&1
wineserver -w
echo "capture setup: $game, prefix $WINEPREFIX"
