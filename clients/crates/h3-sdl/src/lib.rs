//! The client's platform adapter (E17 M2 T5a, ADR-0024): the window, its input and the cursor
//! through SDL 3 (`sdl3-sys`, SDL built from its sources and linked statically), and the audio
//! output (T5b, [`Audio`]; [`load_wav`], SDL's own WAV loader, for the gate). Every SDL call and all of the client's `unsafe` code live here: the pure crates
//! (`h3-assets`, `h3-render`, `h3-view`'s library) never see SDL, and headless rendering, the
//! gates and the reference checks run without it.
//!
//! What the window does (ADR-0024 point 3): it shows a frame of the original's size, widened to
//! 8-bit channels by the caller (`h3_render::screen::Widen`), at the largest integer scale that
//! fits, nearest sampling, centred, black around it; it is high-pixel-density aware. The caller's
//! loop waits for input or its next frame's time ([`Window::wait`]) and presents only when the
//! frame changed. Defaults (*provisional* until M2's review, point 4): vsync off, the game's
//! cursor as the system's hardware colour cursor, scaled with the frame.
//!
//! Not covered by the coverage gate (`clients/scripts/check.sh`: an adapter, as `src/bin/`); the
//! viewer's `h3-view window --check` exercises it.

use std::ffi::{CStr, CString, c_int};
use std::ptr::{null, null_mut};

use sdl3_sys::everything::*;

/// What the window shows when it opens.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    /// The window's title.
    pub title: String,
    /// The frame's width in pixels (the original's screen: 800).
    pub width: u32,
    /// The frame's height in pixels (600).
    pub height: u32,
    /// The window's integer scale; `None`: the largest that fits the display's usable area.
    pub scale: Option<u32>,
    /// Whether presenting waits for the display's refresh.
    pub vsync: bool,
    /// Whether the window starts fullscreen (the desktop's mode, the frame centred).
    pub fullscreen: bool,
}

/// A cursor's image: straight RGBA, 8 bits per channel, row by row, from the top; the hot spot in
/// its pixels.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CursorImage {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// The hot spot's column.
    pub hot_x: u32,
    /// The hot spot's row.
    pub hot_y: u32,
    /// `width × height × 4` bytes.
    pub rgba: Vec<u8>,
}

/// The keys the viewer answers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    /// Escape.
    Escape,
    /// The space bar.
    Space,
    /// The right arrow.
    Right,
    /// R.
    R,
    /// F.
    F,
    /// V.
    V,
    /// C.
    C,
    /// M.
    M,
    /// E.
    E,
}

/// What happened, as the viewer needs it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    /// The window was closed.
    Quit,
    /// A key went down (repeats left out).
    Key(Key),
    /// The pointer moved, in the frame's pixels (outside the frame too); `None`: it left the
    /// window.
    Pointer(Option<(i32, i32)>),
    /// The window needs the frame again (exposed, resized, its scale changed).
    Redraw,
}

/// What the window shows, read back before it is presented: the whole output, the frame's place
/// in it and its scale.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadBack {
    /// The output's width in pixels.
    pub width: u32,
    /// The output's height in pixels.
    pub height: u32,
    /// `width × height` pixels, `0x00RRGGBB`, row by row.
    pub pixels: Vec<u32>,
    /// The frame's left edge in the output.
    pub left: u32,
    /// The frame's top edge in the output.
    pub top: u32,
    /// The frame's integer scale.
    pub scale: u32,
}

/// The hardware cursor in use: SDL's cursor and the scaled image it was made from (kept alive
/// while the cursor is).
#[derive(Debug)]
struct Hardware {
    cursor: *mut SDL_Cursor,
    surface: *mut SDL_Surface,
    _pixels: Vec<u8>,
    scale: u32,
}

/// The window, its renderer and the frame's texture. Not `Send`: SDL's video lives on the thread
/// that opened it.
#[derive(Debug)]
pub struct Window {
    window: *mut SDL_Window,
    renderer: *mut SDL_Renderer,
    texture: *mut SDL_Texture,
    width: u32,
    height: u32,
    vsync: bool,
    fullscreen: bool,
    /// The game's cursor as the hardware cursor, if set; `None`: the system's cursor is hidden
    /// over the window (the frame draws the cursor itself).
    cursor: Option<CursorImage>,
    hardware: Option<Hardware>,
}

/// The room left for the window's title bar and frame when it is sized to fit, in points.
const DECORATION: i64 = 40;

/// SDL's last error.
fn sdl_error() -> String {
    // SAFETY: SDL_GetError returns a valid, NUL-terminated string (empty if none).
    unsafe { CStr::from_ptr(SDL_GetError()) }
        .to_string_lossy()
        .into_owned()
}

/// `what` failed, with SDL's error.
fn failed(what: &str) -> String {
    format!("{what}: {}", sdl_error())
}

/// A C string from SDL, or `none`.
fn text(p: *const std::ffi::c_char) -> String {
    if p.is_null() {
        "none".into()
    } else {
        // SAFETY: SDL returns NUL-terminated strings it keeps alive.
        unsafe { CStr::from_ptr(p) }.to_string_lossy().into_owned()
    }
}

/// Shuts `subsystem` down (SDL counts each subsystem's starts), and SDL itself once no subsystem
/// is left: the window and the audio start and stop theirs independently.
///
/// # Safety
/// Balances one successful `SDL_InitSubSystem(subsystem)`.
unsafe fn quit(subsystem: SDL_InitFlags) {
    // SAFETY: the caller's.
    unsafe {
        SDL_QuitSubSystem(subsystem);
        if SDL_WasInit(SDL_InitFlags(0)) == SDL_InitFlags(0) {
            SDL_Quit();
        }
    }
}

fn int(v: u32) -> c_int {
    c_int::try_from(v).unwrap_or(c_int::MAX)
}

/// The largest integer scale at which `w × h` fits `room_w × room_h`, at least 1.
fn fit(room_w: i64, room_h: i64, w: u32, h: u32) -> u32 {
    let (w, h) = (i64::from(w.max(1)), i64::from(h.max(1)));
    u32::try_from((room_w / w).min(room_h / h).max(1)).unwrap_or(1)
}

impl Window {
    /// Opens the window (SDL's video too), sized to the frame at `config.scale` and centred.
    ///
    /// # Errors
    /// SDL's error when the video, the window, the renderer or the texture cannot be made.
    pub fn open(config: &Config) -> Result<Self, String> {
        let title = CString::new(config.title.replace('\0', " ")).unwrap_or_default();
        let (w, h) = (int(config.width), int(config.height));
        // SAFETY: plain SDL calls; every pointer is checked before use and owned by `Window`.
        unsafe {
            if !SDL_InitSubSystem(SDL_INIT_VIDEO) {
                return Err(failed("SDL_InitSubSystem"));
            }
            let mut flags =
                SDL_WINDOW_HIDDEN | SDL_WINDOW_HIGH_PIXEL_DENSITY | SDL_WINDOW_RESIZABLE;
            if config.fullscreen {
                flags |= SDL_WINDOW_FULLSCREEN;
            }
            let window = SDL_CreateWindow(title.as_ptr(), w, h, flags);
            if window.is_null() {
                let e = failed("SDL_CreateWindow");
                quit(SDL_INIT_VIDEO);
                return Err(e);
            }
            let mut opened = Self {
                window,
                renderer: null_mut(),
                texture: null_mut(),
                width: config.width,
                height: config.height,
                vsync: config.vsync,
                fullscreen: config.fullscreen,
                cursor: None,
                hardware: None,
            };
            // The window's pixels per point, from its size at one point per frame pixel: the
            // display's usable area in pixels decides the scale (integers only).
            let (mut pw, mut ph) = (0, 0);
            SDL_GetWindowSizeInPixels(window, &mut pw, &mut ph);
            let (pw, ph) = (i64::from(pw.max(1)), i64::from(ph.max(1)));
            let (wi, hi) = (i64::from(w.max(1)), i64::from(h.max(1)));
            let scale = config.scale.unwrap_or_else(|| {
                let mut usable = SDL_Rect {
                    x: 0,
                    y: 0,
                    w: 0,
                    h: 0,
                };
                SDL_GetDisplayUsableBounds(SDL_GetDisplayForWindow(window), &mut usable);
                let room_w = i64::from(usable.w) * pw / wi;
                let room_h = (i64::from(usable.h) - DECORATION) * ph / hi;
                fit(room_w, room_h, config.width, config.height)
            });
            let points = |frame: c_int, pixels: i64, per: i64| {
                c_int::try_from(i64::from(frame) * i64::from(scale) * per / pixels).unwrap_or(frame)
            };
            SDL_SetWindowSize(window, points(w, pw, wi), points(h, ph, hi));
            SDL_SetWindowMinimumSize(window, 1, 1);
            SDL_SetWindowPosition(window, SDL_WINDOWPOS_CENTERED, SDL_WINDOWPOS_CENTERED);
            SDL_HideCursor();
            SDL_ShowWindow(window);
            // Window changes apply asynchronously (X11, Wayland: a window manager's fullscreen):
            // wait for them and take their events, so that the renderer starts at the final size.
            SDL_SyncWindow(window);
            SDL_PumpEvents();

            opened.renderer = SDL_CreateRenderer(window, null());
            if opened.renderer.is_null() {
                return Err(failed("SDL_CreateRenderer"));
            }
            SDL_SetRenderVSync(opened.renderer, c_int::from(config.vsync));
            if !SDL_SetRenderLogicalPresentation(
                opened.renderer,
                w,
                h,
                SDL_LOGICAL_PRESENTATION_INTEGER_SCALE,
            ) {
                return Err(failed("SDL_SetRenderLogicalPresentation"));
            }
            opened.texture = SDL_CreateTexture(
                opened.renderer,
                SDL_PIXELFORMAT_XRGB8888,
                SDL_TEXTUREACCESS_STREAMING,
                w,
                h,
            );
            if opened.texture.is_null() {
                return Err(failed("SDL_CreateTexture"));
            }
            SDL_SetTextureScaleMode(opened.texture, SDL_SCALEMODE_NEAREST);
            Ok(opened)
        }
    }

    /// SDL's clock: nanoseconds since SDL started.
    #[must_use]
    pub fn ticks_ns(&self) -> u64 {
        // SAFETY: a plain SDL call.
        unsafe { SDL_GetTicksNS() }
    }

    /// Waits up to `timeout_ms` for the first event, then takes every event pending; the ones
    /// the viewer answers, in order (empty after a timeout).
    pub fn wait(&mut self, timeout_ms: u32) -> Vec<Event> {
        let mut events = Vec::new();
        // SAFETY: SDL_Event is plain data; SDL fills it.
        let mut ev: SDL_Event = unsafe { std::mem::zeroed() };
        // SAFETY: as above.
        let mut got = unsafe { SDL_WaitEventTimeout(&mut ev, int(timeout_ms)) };
        while got {
            if let Some(e) = self.translate(&mut ev) {
                events.push(e);
            }
            // SAFETY: as above.
            got = unsafe { SDL_PollEvent(&mut ev) };
        }
        events
    }

    /// An SDL event as the viewer's, if it answers it.
    fn translate(&mut self, ev: &mut SDL_Event) -> Option<Event> {
        // SAFETY: the union's member is the one its type names.
        unsafe {
            let ty = SDL_EventType(ev.r#type);
            match ty {
                SDL_EVENT_QUIT | SDL_EVENT_WINDOW_CLOSE_REQUESTED => Some(Event::Quit),
                SDL_EVENT_KEY_DOWN if !ev.key.repeat => {
                    let key = match ev.key.key {
                        SDLK_ESCAPE => Key::Escape,
                        SDLK_SPACE => Key::Space,
                        SDLK_RIGHT => Key::Right,
                        SDLK_R => Key::R,
                        SDLK_F => Key::F,
                        SDLK_V => Key::V,
                        SDLK_C => Key::C,
                        SDLK_M => Key::M,
                        SDLK_E => Key::E,
                        _ => return None,
                    };
                    Some(Event::Key(key))
                }
                SDL_EVENT_MOUSE_MOTION => {
                    SDL_ConvertEventToRenderCoordinates(self.renderer, ev);
                    // Frame pixels: the converted position's integer part (`floor`, so that
                    // the pixel left of the frame is -1).
                    let (x, y) = (ev.motion.x.floor(), ev.motion.y.floor());
                    #[allow(clippy::cast_possible_truncation)]
                    Some(Event::Pointer(Some((x as i32, y as i32))))
                }
                SDL_EVENT_WINDOW_MOUSE_LEAVE => Some(Event::Pointer(None)),
                SDL_EVENT_WINDOW_EXPOSED
                | SDL_EVENT_WINDOW_PIXEL_SIZE_CHANGED
                | SDL_EVENT_WINDOW_DISPLAY_SCALE_CHANGED
                | SDL_EVENT_WINDOW_ENTER_FULLSCREEN
                | SDL_EVENT_WINDOW_LEAVE_FULLSCREEN => {
                    self.refresh_cursor();
                    Some(Event::Redraw)
                }
                _ => None,
            }
        }
    }

    /// The frame's integer scale now: the largest at which it fits the window's output.
    #[must_use]
    pub fn scale(&self) -> u32 {
        let (mut ow, mut oh) = (0, 0);
        // SAFETY: a plain SDL call on the window's renderer.
        unsafe { SDL_GetRenderOutputSize(self.renderer, &mut ow, &mut oh) };
        fit(i64::from(ow), i64::from(oh), self.width, self.height)
    }

    /// Draws `frame` (`width × height` pixels, `0x00RRGGBB`) into the back buffer.
    fn draw(&mut self, frame: &[u32]) -> Result<(), String> {
        let pixels = usize::try_from(u64::from(self.width) * u64::from(self.height)).unwrap_or(0);
        if frame.len() != pixels {
            return Err(format!(
                "a frame of {} pixels, not {} × {}",
                frame.len(),
                self.width,
                self.height
            ));
        }
        // SAFETY: the texture is `width × height` XRGB8888; `frame` holds that many pixels.
        unsafe {
            if !SDL_UpdateTexture(
                self.texture,
                null(),
                frame.as_ptr().cast(),
                int(self.width.saturating_mul(4)),
            ) {
                return Err(failed("SDL_UpdateTexture"));
            }
            SDL_SetRenderDrawColor(self.renderer, 0, 0, 0, 255);
            SDL_RenderClear(self.renderer);
            if !SDL_RenderTexture(self.renderer, self.texture, null(), null()) {
                return Err(failed("SDL_RenderTexture"));
            }
        }
        Ok(())
    }

    /// Shows `frame` (`width × height` pixels, `0x00RRGGBB`).
    ///
    /// # Errors
    /// A frame of another size, or SDL's error.
    pub fn present(&mut self, frame: &[u32]) -> Result<(), String> {
        self.draw(frame)?;
        // SAFETY: a plain SDL call on the window's renderer.
        if unsafe { SDL_RenderPresent(self.renderer) } {
            Ok(())
        } else {
            Err(failed("SDL_RenderPresent"))
        }
    }

    /// As [`Window::present`], reading the whole output back first (the window's check).
    ///
    /// # Errors
    /// As [`Window::present`], or when the output cannot be read.
    pub fn present_read(&mut self, frame: &[u32]) -> Result<ReadBack, String> {
        self.draw(frame)?;
        let back = self.read_back();
        self.present_drawn()?;
        back
    }

    fn present_drawn(&mut self) -> Result<(), String> {
        // SAFETY: a plain SDL call on the window's renderer.
        if unsafe { SDL_RenderPresent(self.renderer) } {
            Ok(())
        } else {
            Err(failed("SDL_RenderPresent"))
        }
    }

    /// The back buffer as drawn, the whole output (the logical presentation lifted while it is
    /// read, as the T2 spike did).
    fn read_back(&mut self) -> Result<ReadBack, String> {
        let (w, h) = (int(self.width), int(self.height));
        // SAFETY: SDL allocates the surfaces; they are read within their bounds and destroyed.
        unsafe {
            let mut rect = SDL_FRect {
                x: 0.0,
                y: 0.0,
                w: 0.0,
                h: 0.0,
            };
            SDL_GetRenderLogicalPresentationRect(self.renderer, &mut rect);
            SDL_SetRenderLogicalPresentation(
                self.renderer,
                0,
                0,
                SDL_LOGICAL_PRESENTATION_DISABLED,
            );
            let surface = SDL_RenderReadPixels(self.renderer, null());
            SDL_SetRenderLogicalPresentation(
                self.renderer,
                w,
                h,
                SDL_LOGICAL_PRESENTATION_INTEGER_SCALE,
            );
            if surface.is_null() {
                return Err(failed("SDL_RenderReadPixels"));
            }
            let converted = SDL_ConvertSurface(surface, SDL_PIXELFORMAT_XRGB8888);
            SDL_DestroySurface(surface);
            if converted.is_null() {
                return Err(failed("SDL_ConvertSurface"));
            }
            let (cw, ch) = ((*converted).w.max(0), (*converted).h.max(0));
            let pitch = usize::try_from((*converted).pitch.max(0)).unwrap_or(0);
            let (cw, ch) = (
                usize::try_from(cw).unwrap_or(0),
                usize::try_from(ch).unwrap_or(0),
            );
            let base = (*converted).pixels.cast::<u8>().cast_const();
            let mut pixels = Vec::with_capacity(cw * ch);
            for y in 0..ch {
                let row = std::slice::from_raw_parts(base.add(y * pitch), cw * 4);
                pixels.extend(
                    row.chunks_exact(4)
                        .map(|p| u32::from_le_bytes([p[0], p[1], p[2], 0])),
                );
            }
            SDL_DestroySurface(converted);
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let (left, top, width) = (rect.x as u32, rect.y as u32, rect.w as u32);
            Ok(ReadBack {
                width: u32::try_from(cw).unwrap_or(0),
                height: u32::try_from(ch).unwrap_or(0),
                pixels,
                left,
                top,
                scale: width / self.width.max(1),
            })
        }
    }

    /// Whether presenting waits for the display's refresh.
    #[must_use]
    pub fn vsync(&self) -> bool {
        self.vsync
    }

    /// Turns vsync on or off; whether the renderer took it.
    pub fn set_vsync(&mut self, on: bool) -> bool {
        // SAFETY: a plain SDL call on the window's renderer.
        let taken = unsafe { SDL_SetRenderVSync(self.renderer, c_int::from(on)) };
        if taken {
            self.vsync = on;
        }
        taken
    }

    /// Whether the window is fullscreen.
    #[must_use]
    pub fn fullscreen(&self) -> bool {
        self.fullscreen
    }

    /// Fullscreen (the desktop's mode, the frame centred at its integer scale) or windowed.
    pub fn set_fullscreen(&mut self, on: bool) {
        // SAFETY: plain SDL calls on the window.
        if unsafe { SDL_SetWindowFullscreen(self.window, on) } {
            self.fullscreen = on;
            // SAFETY: as above.
            unsafe { SDL_SyncWindow(self.window) };
        }
    }

    /// The game's cursor as the hardware cursor (`Some`), or the system's cursor hidden over the
    /// window (`None`: the frame draws the cursor).
    ///
    /// # Errors
    /// SDL's error when the cursor cannot be made; the system's cursor is hidden then.
    pub fn set_cursor(&mut self, image: Option<CursorImage>) -> Result<(), String> {
        self.cursor = image;
        self.drop_hardware();
        self.make_cursor()
    }

    /// Whether the hardware cursor is in use.
    #[must_use]
    pub fn hardware_cursor(&self) -> bool {
        self.hardware.is_some()
    }

    /// The hardware cursor again when the frame's scale changed.
    fn refresh_cursor(&mut self) {
        if self.cursor.is_some()
            && self
                .hardware
                .as_ref()
                .is_none_or(|h| h.scale != self.scale())
        {
            self.drop_hardware();
            let _ = self.make_cursor();
        }
    }

    /// The hardware cursor from the image at the frame's scale, or the system's cursor hidden.
    fn make_cursor(&mut self) -> Result<(), String> {
        let Some(image) = &self.cursor else {
            // SAFETY: a plain SDL call.
            unsafe { SDL_HideCursor() };
            return Ok(());
        };
        let scale = self.scale();
        let s = usize::try_from(scale).unwrap_or(1);
        let (w, h) = (
            usize::try_from(image.width).unwrap_or(0),
            usize::try_from(image.height).unwrap_or(0),
        );
        if image.rgba.len() != w * h * 4 || w == 0 || h == 0 {
            return Err(format!(
                "a cursor of {w} × {h} with {} bytes",
                image.rgba.len()
            ));
        }
        let mut pixels = vec![0u8; w * s * h * s * 4];
        for (y, row) in pixels.chunks_exact_mut(w * s * 4).enumerate() {
            for (x, px) in row.chunks_exact_mut(4).enumerate() {
                let src = ((y / s) * w + x / s) * 4;
                px.copy_from_slice(&image.rgba[src..src + 4]);
            }
        }
        let (hot_x, hot_y) = (
            int(image.hot_x.saturating_mul(scale)),
            int(image.hot_y.saturating_mul(scale)),
        );
        // SAFETY: the surface borrows `pixels`, kept alive with the cursor in `Hardware`.
        unsafe {
            let surface = SDL_CreateSurfaceFrom(
                c_int::try_from(w * s).unwrap_or(0),
                c_int::try_from(h * s).unwrap_or(0),
                SDL_PIXELFORMAT_RGBA32,
                pixels.as_mut_ptr().cast(),
                c_int::try_from(w * s * 4).unwrap_or(0),
            );
            if surface.is_null() {
                SDL_HideCursor();
                return Err(failed("SDL_CreateSurfaceFrom"));
            }
            let cursor = SDL_CreateColorCursor(surface, hot_x, hot_y);
            if cursor.is_null() {
                SDL_DestroySurface(surface);
                SDL_HideCursor();
                return Err(failed("SDL_CreateColorCursor"));
            }
            SDL_SetCursor(cursor);
            SDL_ShowCursor();
            self.hardware = Some(Hardware {
                cursor,
                surface,
                _pixels: pixels,
                scale,
            });
        }
        Ok(())
    }

    /// Lets go of the hardware cursor (the system's default first, so that SDL never holds a
    /// destroyed cursor).
    fn drop_hardware(&mut self) {
        if let Some(h) = self.hardware.take() {
            // SAFETY: the cursor and surface were made by SDL and are freed once.
            unsafe {
                SDL_SetCursor(SDL_GetDefaultCursor());
                SDL_DestroyCursor(h.cursor);
                SDL_DestroySurface(h.surface);
            }
        }
    }

    /// One line on what SDL chose: its version, the video driver, the renderer, the window's
    /// output and the frame's scale, vsync, the cursor.
    #[must_use]
    pub fn describe(&self) -> String {
        // SAFETY: plain SDL calls; the strings are SDL's.
        let (version, driver, renderer) = unsafe {
            (
                SDL_GetVersion(),
                text(SDL_GetCurrentVideoDriver()),
                text(SDL_GetRendererName(self.renderer)),
            )
        };
        let (mut ow, mut oh) = (0, 0);
        // SAFETY: a plain SDL call on the window's renderer.
        unsafe { SDL_GetRenderOutputSize(self.renderer, &mut ow, &mut oh) };
        format!(
            "SDL {}.{}.{}, video {driver}, renderer {renderer}, output {ow} × {oh}, scale {}, \
             vsync {}, {} cursor{}",
            version / 1_000_000,
            version / 1000 % 1000,
            version % 1000,
            self.scale(),
            if self.vsync { "on" } else { "off" },
            if self.hardware.is_some() {
                "hardware"
            } else {
                "drawn"
            },
            if self.fullscreen { ", fullscreen" } else { "" },
        )
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        self.drop_hardware();
        // SAFETY: each pointer was made by SDL, is freed once, the texture and the renderer
        // before the window; `quit` balances SDL_InitSubSystem.
        unsafe {
            if !self.texture.is_null() {
                SDL_DestroyTexture(self.texture);
            }
            if !self.renderer.is_null() {
                SDL_DestroyRenderer(self.renderer);
            }
            SDL_DestroyWindow(self.window);
            quit(SDL_INIT_VIDEO);
        }
    }
}

/// The audio output (E17 M2 T5b, ADR-0024 point 5): one SDL logical device on the system's
/// default playback device and one SDL audio stream per source, each converting and resampling
/// its 16-bit samples to the device's format; SDL mixes them. A stream's index is its handle. Not
/// `Send`, as [`Window`].
pub struct Audio {
    device: SDL_AudioDeviceID,
    streams: Vec<*mut SDL_AudioStream>,
}

impl std::fmt::Debug for Audio {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Audio")
            .field("device", &self.device.0)
            .field("streams", &self.streams.len())
            .finish()
    }
}

/// SDL's description of 16-bit samples, `channels` × `rate`.
fn spec(rate: u32, channels: u16) -> SDL_AudioSpec {
    SDL_AudioSpec {
        format: SDL_AUDIO_S16,
        channels: c_int::from(channels),
        freq: int(rate),
    }
}

impl Audio {
    /// Opens the default playback device (SDL's audio too).
    ///
    /// # Errors
    /// SDL's error when the audio or the device cannot be opened (no sound device, say).
    pub fn open() -> Result<Self, String> {
        // SAFETY: plain SDL calls; the device is checked and owned by `Audio`.
        unsafe {
            if !SDL_InitSubSystem(SDL_INIT_AUDIO) {
                return Err(failed("SDL_InitSubSystem(audio)"));
            }
            let device = SDL_OpenAudioDevice(SDL_AUDIO_DEVICE_DEFAULT_PLAYBACK, null());
            if device == SDL_AudioDeviceID(0) {
                let e = failed("SDL_OpenAudioDevice");
                quit(SDL_INIT_AUDIO);
                return Err(e);
            }
            Ok(Self {
                device,
                streams: Vec::new(),
            })
        }
    }

    /// Adds a stream for 16-bit samples of `channels` × `rate`, bound to the device; its index.
    ///
    /// # Errors
    /// SDL's error when the stream cannot be made or bound.
    pub fn add_stream(&mut self, rate: u32, channels: u16) -> Result<usize, String> {
        let spec = spec(rate, channels);
        // SAFETY: plain SDL calls; the stream is checked and owned by `Audio`.
        unsafe {
            let stream = SDL_CreateAudioStream(&raw const spec, null());
            if stream.is_null() {
                return Err(failed("SDL_CreateAudioStream"));
            }
            if !SDL_BindAudioStream(self.device, stream) {
                let e = failed("SDL_BindAudioStream");
                SDL_DestroyAudioStream(stream);
                return Err(e);
            }
            self.streams.push(stream);
        }
        Ok(self.streams.len() - 1)
    }

    /// The stream at `index` (null for an index that is none: SDL then fails the call).
    fn stream(&self, index: usize) -> *mut SDL_AudioStream {
        self.streams.get(index).copied().unwrap_or(null_mut())
    }

    /// Changes the samples a stream takes from now on (`channels` × `rate`).
    pub fn set_format(&mut self, index: usize, rate: u32, channels: u16) {
        let spec = spec(rate, channels);
        // SAFETY: a stream of ours; SDL copies the spec.
        unsafe { SDL_SetAudioStreamFormat(self.stream(index), &raw const spec, null()) };
    }

    /// Queues `samples` (interleaved) on a stream; with `flush`, the stream's last samples are
    /// made playable at once (a one-shot clip: else the resampler holds them back for what
    /// follows and the stream never empties; design §8).
    ///
    /// # Errors
    /// SDL's error.
    pub fn put(&mut self, index: usize, samples: &[i16], flush: bool) -> Result<(), String> {
        let bytes = c_int::try_from(size_of_val(samples)).unwrap_or(c_int::MAX);
        let stream = self.stream(index);
        // SAFETY: SDL copies `bytes` bytes of the slice before returning.
        unsafe {
            if !SDL_PutAudioStreamData(stream, samples.as_ptr().cast(), bytes) {
                return Err(failed("SDL_PutAudioStreamData"));
            }
            if flush && !SDL_FlushAudioStream(stream) {
                return Err(failed("SDL_FlushAudioStream"));
            }
        }
        Ok(())
    }

    /// Pauses the device (every stream holds where it is) or resumes it.
    pub fn pause(&mut self, on: bool) {
        // SAFETY: our device.
        unsafe {
            if on {
                SDL_PauseAudioDevice(self.device);
            } else {
                SDL_ResumeAudioDevice(self.device);
            }
        }
    }

    /// Drops what a stream has queued: it falls silent.
    pub fn clear(&mut self, index: usize) {
        // SAFETY: a stream of ours.
        unsafe { SDL_ClearAudioStream(self.stream(index)) };
    }

    /// The samples a stream still has queued (its input's, all channels counted).
    #[must_use]
    pub fn queued(&self, index: usize) -> usize {
        // SAFETY: a stream of ours.
        let bytes = unsafe { SDL_GetAudioStreamQueued(self.stream(index)) };
        usize::try_from(bytes).unwrap_or(0) / 2
    }

    /// A stream's gain, applied as SDL takes its samples (1: as they are).
    pub fn set_gain(&mut self, index: usize, gain: f32) {
        // SAFETY: a stream of ours.
        unsafe { SDL_SetAudioStreamGain(self.stream(index), gain) };
    }

    /// The driver and the device's format, for the viewer's report.
    #[must_use]
    pub fn describe(&self) -> String {
        let mut device = SDL_AudioSpec {
            format: SDL_AUDIO_S16,
            channels: 0,
            freq: 0,
        };
        let mut frames: c_int = 0;
        // SAFETY: SDL fills the spec and the frame count.
        let known =
            unsafe { SDL_GetAudioDeviceFormat(self.device, &raw mut device, &raw mut frames) };
        // SAFETY: a plain SDL call.
        let driver = text(unsafe { SDL_GetCurrentAudioDriver() });
        if known {
            format!(
                "audio {driver}: {} Hz, {} channels, {} frames per buffer",
                device.freq, device.channels, frames
            )
        } else {
            format!("audio {driver}: format unknown")
        }
    }
}

impl Drop for Audio {
    fn drop(&mut self) {
        // SAFETY: each stream and the device were made by SDL and are freed once, the streams
        // first; `quit` balances SDL_InitSubSystem.
        unsafe {
            for &stream in &self.streams {
                SDL_DestroyAudioStream(stream);
            }
            SDL_CloseAudioDevice(self.device);
            quit(SDL_INIT_AUDIO);
        }
    }
}

/// A WAV as SDL's own loader decodes it (`SDL_LoadWAV_IO`; E17 M2 T5b's reference for the
/// pure decoder, `h3_assets::wav`): the rate, the channels and the samples as 16 bits (SDL's
/// 8-bit PCM widened as SDL converts it). `whole` sets the hints `SDL_WAVE_TRUNCATION=dropframe`
/// and `SDL_WAVE_FACT_CHUNK=truncate` (every complete frame, up to the `fact` chunk's count);
/// else SDL's defaults (the last cut block dropped, the `fact` chunk ignored).
///
/// # Errors
/// SDL's error for a file it rejects.
pub fn load_wav(bytes: &[u8], whole: bool) -> Result<(u32, u16, Vec<i16>), String> {
    let hints = [
        (SDL_HINT_WAVE_TRUNCATION, c"dropframe"),
        (SDL_HINT_WAVE_FACT_CHUNK, c"truncate"),
    ];
    let mut spec = SDL_AudioSpec {
        format: SDL_AUDIO_UNKNOWN,
        channels: 0,
        freq: 0,
    };
    let mut buffer: *mut u8 = null_mut();
    let mut length: u32 = 0;
    // SAFETY: SDL reads the slice through a constant memory stream it closes itself, and hands
    // back a buffer of `length` bytes, freed with SDL_free below.
    unsafe {
        for (name, value) in hints {
            if whole {
                SDL_SetHint(name, value.as_ptr());
            } else {
                SDL_ResetHint(name);
            }
        }
        let io = SDL_IOFromConstMem(bytes.as_ptr().cast(), bytes.len());
        if io.is_null() {
            return Err(failed("SDL_IOFromConstMem"));
        }
        if !SDL_LoadWAV_IO(io, true, &raw mut spec, &raw mut buffer, &raw mut length) {
            return Err(failed("SDL_LoadWAV_IO"));
        }
        let raw: &[u8] = if buffer.is_null() {
            &[]
        } else {
            std::slice::from_raw_parts(buffer, usize::try_from(length).unwrap_or(0))
        };
        let samples = if spec.format == SDL_AUDIO_U8 {
            raw.iter().map(|&b| (i16::from(b) - 128) << 8).collect()
        } else {
            raw.as_chunks::<2>()
                .0
                .iter()
                .map(|&pair| i16::from_ne_bytes(pair))
                .collect()
        };
        let format = spec.format;
        SDL_free(buffer.cast());
        if format != SDL_AUDIO_S16 && format != SDL_AUDIO_U8 {
            return Err(format!("SDL_LoadWAV_IO: sample format {:#x}", format.0));
        }
        let rate = u32::try_from(spec.freq).unwrap_or(0);
        let channels = u16::try_from(spec.channels).unwrap_or(0);
        Ok((rate, channels, samples))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_scale_fits() {
        // The original's 800 × 600 on 2560 × 1440 (less a taskbar and the title), 1920 × 1080
        // and 3840 × 2160; at least 1.
        assert_eq!(fit(2560, 1352, 800, 600), 2);
        assert_eq!(fit(1920, 992, 800, 600), 1);
        assert_eq!(fit(3840, 2072, 800, 600), 3);
        assert_eq!(fit(600, 400, 800, 600), 1);
        assert_eq!(fit(0, 0, 0, 0), 1);
    }
}
