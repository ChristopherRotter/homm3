//! A scene composed into one frame (E24 M1 T3b1, *provisional*): each [`Draw`] with the drawer
//! the executable uses for it, from a [`Batch`], into a screen in the original's 16-bit format
//! (`docs/epics/E17-design.md` §3.2). The one place that decides how a named thing is drawn, as
//! `h3-view`'s compose is for the battle: the frame's picture opaque in the viewer's colours
//! (design §1.9), the view's tiles and the path's arrows through `draw_frame_raw`
//! ([`crate::draw::raw`]), the unexplored tiles' second time and the arrows' shadows through
//! 0x47E820 ([`crate::draw::shadows`]; E24 M1 T5b2, §2.2), the objects' bodies
//! and flat objects and the heroes' and boats' pictures through the flagged drawer
//! ([`crate::draw::flagged`]; an owned object's flag pixels in its entry of the flag palette,
//! E24 M1 T5b1, design §2.9), their shadows through
//! the shadow pass ([`crate::draw::shadow_pass`], §2.2), the gems through the plain drawer with
//! runs skipped (`h3_render::blit`), the dialog's items by `h3_ui::paint` (E24 M3 T2a). A DEF's
//! palette is cycled once per scene (design §1.6).

use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use h3_assets::def::Frame;
use h3_assets::palette::{Palette, Rgb};
use h3_assets::pcx::Pcx;
use h3_render::blit::{Drawer, Placement, draw_frame, draw_image};
use h3_render::palette::{ScreenPalette, with_player_colours};
use h3_render::screen::{PixelFormat, Screen};

use h3_ui::sprite::Sprite;

use crate::batch::Batch;
use crate::draw::{self, Place};
use crate::scene::{Cycle, Draw, Pass, Scene};

/// The screen's pixel format (E17 design §3.2: the executable's 16-bit 5-6-5 screen).
pub const FORMAT: PixelFormat = PixelFormat::RGB565;

/// Why a scene does not compose.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComposeError {
    /// The frame is larger than a screen may be.
    Frame([u32; 2]),
    /// An asset or a frame the scene names is not in the batch.
    Missing(String),
}

impl fmt::Display for ComposeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Frame([w, h]) => write!(f, "a {w} x {h} frame is too large"),
            Self::Missing(what) => write!(f, "not in the batch: {what}"),
        }
    }
}

/// Draws `scene` in order onto a fresh `frame`-sized screen (black where nothing is drawn).
///
/// # Errors
/// A frame too large, or anything the scene names missing from `batch` (a picture past its DEF's
/// frames among them: the viewer refuses such a map, design §1.3).
pub fn compose(scene: &Scene, batch: &Batch, frame: [u32; 2]) -> Result<Screen, ComposeError> {
    let [width, height] = frame;
    let mut screen = Screen::new(width, height, FORMAT).ok_or(ComposeError::Frame(frame))?;
    // Each DEF's palette cycled once, by its name as the scene spells it (one name in two
    // spellings only cycles twice), and the flag palette's colours once (E24 M2 T4b3).
    let mut palettes: BTreeMap<&str, ScreenPalette> = BTreeMap::new();
    let mut flag_colours = None;
    for item in &scene.draws {
        match item {
            Draw::Frame {
                picture,
                players,
                viewer,
            } => {
                let image = batch
                    .picture(picture)
                    .ok_or_else(|| ComposeError::Missing(picture.clone()))?;
                let players = batch
                    .palette(players)
                    .ok_or_else(|| ComposeError::Missing(players.clone()))?;
                frame_picture(&mut screen, image, players, *viewer);
            }
            Draw::Piece {
                pass,
                def,
                group,
                frame,
                x,
                y,
                mirror_x,
                mirror_y,
                clip,
                flag,
            } => {
                let (sprite, f) = frame_of(batch, def, *group, *frame)?;
                let palette = palettes
                    .entry(def.as_str())
                    .or_insert_with(|| cycled(sprite, def, &scene.cycles));
                let at = Place {
                    x: *x,
                    y: *y,
                    mirror_x: *mirror_x,
                    mirror_y: *mirror_y,
                };
                match pass {
                    Pass::Flat | Pass::Body => {
                        let colour = flag_colour(batch, &scene.flags, *flag, &mut flag_colours)?;
                        draw::flagged(&mut screen, f, palette, at, *clip, colour);
                    }
                    Pass::Shadow => draw::shadow_pass(&mut screen, f, at, *clip),
                    Pass::PathShadow => draw::shadows(&mut screen, f, at, *clip),
                    Pass::Blend => draw::blend(&mut screen, f, palette, at, *clip),
                    _ => draw::raw(&mut screen, f, palette, at, *clip),
                }
                if *pass == Pass::Shroud {
                    draw::shadows(&mut screen, f, at, *clip);
                }
            }
            Draw::Gem { def, frame, x, y } => {
                let (sprite, f) = frame_of(batch, def, 0, *frame)?;
                let palette = ScreenPalette::new(&sprite.def.palette, FORMAT);
                let at = Placement {
                    x: *x,
                    y: *y,
                    mirrored: false,
                };
                let plain = Drawer::Plain {
                    skip_runs: true,
                    colour: None,
                };
                draw_frame(&mut screen, f, &palette, at, plain);
            }
            Draw::Dialog(draw) => {
                // The dialog's items by their own drawers (E24 M3 T2a, design §4.1).
                h3_ui::paint::draw(&mut screen, draw, batch, scene.bugs)
                    .map_err(ComposeError::Missing)?;
            }
        }
    }
    Ok(screen)
}

/// The screen colour of entry `flag` of the flag palette `flags`, `None` without an entry or for
/// a colour of 0, which the flagged drawer takes as none (0x47D0A0; design §2.9); the palette's
/// colours worked out once a scene, in `colours`.
fn flag_colour(
    batch: &Batch,
    flags: &str,
    flag: Option<u8>,
    colours: &mut Option<ScreenPalette>,
) -> Result<Option<u16>, ComposeError> {
    let Some(entry) = flag else {
        return Ok(None);
    };
    let palette = match colours.take() {
        Some(palette) => palette,
        None => {
            let palette = batch
                .palette(flags)
                .ok_or_else(|| ComposeError::Missing(String::from(flags)))?;
            ScreenPalette::new(palette, FORMAT)
        }
    };
    let colour = colours.insert(palette).get(entry);
    Ok((colour != 0).then_some(colour))
}

/// The frame's picture drawn opaque at the screen's top-left corner in a viewer's colours: an
/// 8-bit one's entries 224–255 its 32 colours of the players' palette (from entry 32 × the
/// colour), row by row with no copy of it; a 24-bit one has no palette and is drawn as it is.
fn frame_picture(screen: &mut Screen, image: &Pcx, players: &Palette, viewer: u8) {
    match image {
        Pcx::Indexed(indexed) => {
            let colours: Vec<Rgb> = players
                .0
                .iter()
                .skip(32 * usize::from(viewer))
                .take(32)
                .copied()
                .collect();
            let palette =
                ScreenPalette::new(&with_player_colours(&indexed.palette, &colours), FORMAT);
            let width = usize::try_from(screen.width()).unwrap_or(0).max(1);
            let from = usize::try_from(indexed.width).unwrap_or(0).max(1);
            for (row, line) in screen
                .pixels_mut()
                .chunks_mut(width)
                .zip(indexed.pixels.chunks(from))
            {
                for (p, &index) in row.iter_mut().zip(line) {
                    *p = palette.get(index);
                }
            }
        }
        Pcx::Rgb(_) => draw_image(screen, image, 0, 0),
    }
}

/// `sprite`'s palette in the screen's format, the ranges of `def`'s cycles (by name, ignoring
/// case) rotated their steps: each step gives every entry of a range its lower neighbour's colour
/// and the first the last's (`palette16_rotate` with −1, design §1.6).
fn cycled(sprite: &Sprite, def: &str, cycles: &[Cycle]) -> ScreenPalette {
    let mut palette = sprite.def.palette.clone();
    for cycle in cycles.iter().filter(|c| c.def.eq_ignore_ascii_case(def)) {
        for &[first, last] in &cycle.ranges {
            rotate(&mut palette, first, last, cycle.steps);
        }
    }
    ScreenPalette::new(&palette, FORMAT)
}

/// Rotates entries `first` to `last` of `palette` up by `steps` (an empty range is left).
fn rotate(palette: &mut Palette, first: u8, last: u8, steps: u32) {
    let (first, last) = (usize::from(first), usize::from(last));
    let Some(range) = palette.0.get_mut(first..=last) else {
        return;
    };
    let n = u32::try_from(range.len()).unwrap_or(1).max(1);
    let k = usize::try_from(steps % n).unwrap_or(0);
    range.rotate_right(k);
}

fn frame_of<'b>(
    batch: &'b Batch,
    def: &str,
    group: u32,
    index: u32,
) -> Result<(&'b Sprite, &'b Frame), ComposeError> {
    let sprite = batch
        .sprite(def)
        .ok_or_else(|| ComposeError::Missing(def.into()))?;
    let frame = sprite
        .frame(group, index)
        .ok_or_else(|| ComposeError::Missing(format!("{def} group {group} frame {index}")))?;
    Ok((sprite, frame))
}

#[cfg(test)]
mod tests;
