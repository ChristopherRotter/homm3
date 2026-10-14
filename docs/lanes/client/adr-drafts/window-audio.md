# ADR-NNNN: The client's window, input and audio: SDL 3

- Status: Proposed (the library chosen by the user, 2026-10-14, E17 M2 T2; the text awaits
  acceptance and a number)
- Date: 2026-10-14
- Deciders: user; client lane session (E17 M2 T2)
- Fills ADR-0020 point 4 (chosen at M2 by a measured evaluation). Evaluation and measurements:
  `docs/epics/E17-design.md` §8; the spikes: branch `client/e17-m2-t2-spikes` (never merged).

<!-- Decisions only. Measurements and findings: E17-design.md §8. -->

## Context

ADR-0020 keeps the client's drawing in software and leaves the window, input and audio to one thin
adapter, chosen at M2 on technical grounds; Apache-2.0-only crates (`winit`, `cpal`) would need the
client under GPLv3. Three spikes showed the same battle frames, cursor, effect and music on the
user's Windows machine and in the sandbox: SDL 3, `minifb` + `tinyaudio`, `miniquad` + `tinyaudio`.
All three draw the frame exactly; SDL 3 alone played the sound cleanly (the `tinyaudio` spikes
crackled: DirectSound underruns) and has a hardware colour cursor, fullscreen, any integer scale and
DPI awareness.

## Decision

1. **SDL 3 for the window, input, cursor and audio output**, through `sdl3-sys` (Zlib; raw
   bindings that follow SDL's releases), SDL built from its sources by the `sdl3-src` crate and
   linked statically (feature `build-from-source-static`; the subsystems the client does not use
   left out). One executable, no SDL DLL. The thin `sdl3` wrapper crate is not used.
2. **One adapter crate** in `clients/` holds every SDL call (its `unsafe` included); the pure crates
   (`h3-assets`, `h3-render`, `h3-view`'s library) never see SDL. Headless rendering, the gates
   and the reference checks stay without a window.
3. **Presentation:** the screen goes up as a texture each time it changes (the 16-bit format or
   widened on the CPU: both read back exact) and is shown at the largest integer scale that fits,
   nearest sampling, centred, black around it (`SDL_SetRenderLogicalPresentation`, integer mode);
   the window is high-pixel-density aware. The loop waits for input or the next frame's time
   (`SDL_WaitEventTimeout`) and presents only when the frame changed.
4. **Settings, defaults *provisional* until M2's review:** vsync **off** (a frame shows within
   about 1.5 ms of the original's time; on: within one refresh); the game's cursor as the
   system's **hardware** colour cursor, scaled with the frame (drawn into the frame: an option).
5. **Audio:** one SDL logical device, one SDL audio stream per source (music, each playing effect),
   SDL converting, resampling and mixing. The game's IMA ADPCM WAVs are decoded by `h3-assets`
   (pure, ADR-0020 point 2), MP3 by `nanomp3` (MIT or Apache-2.0, pure Rust; minimp3's samples);
   the adapter only plays PCM.
6. **Licence:** SDL (Zlib), `sdl3-sys` (Zlib) and `nanomp3` (MIT or Apache-2.0) fit the engine's
   policy: the client stays GPL-2.0-or-later with the engine's `deny.toml` (ADR-0020 point 4's
   GPLv3 step is not needed).

## Consequences

- Building the client needs CMake and a C compiler (first build about 3 minutes, cached); on Linux
  the X11, Wayland, ALSA and Pulse headers decide which SDL backends exist. CI builds the adapter
  but opens no window.
- Upgrading SDL is a version bump of `sdl3-sys`/`sdl3-src`; their `unsafe` surface is contained in
  one crate.
- The executable grows by about 5 MB.
- A browser client (WASM) would need another adapter; SDL's Emscripten port or a web one, later.
- `winit`, `cpal`, Bevy, `rodio` and the like stay excluded (ADR-0009).

## Alternatives considered

- **`minifb` + `tinyaudio`:** pure Rust and small, but no vsync, no fullscreen, scales 1, 2, 4 and
  8 only, no DPI awareness on Windows, no custom cursor; `tinyaudio`'s DirectSound backend
  crackled on the user's machine; mixing and resampling would be ours.
- **`miniquad` + `tinyaudio`:** pure Rust, OpenGL; draws every refresh or spins without vsync, no
  timers, no custom cursor image; the same audio problem.
- **`sdl2`:** SDL 2 is in maintenance; SDL 3 is its successor.
- **`winit` + `softbuffer` + `cpal`:** Apache-2.0-only parts; would move the client to GPLv3 for
  no gain over SDL 3.
