//! A scene composed into one frame (E24 M1 T3b1, *provisional*): each [`Draw`] with the drawer
//! the executable uses for it, from a [`Batch`], into a screen in the original's 16-bit format
//! (`docs/epics/E17-design.md` §3.2). The one place that decides how a named thing is drawn, as
//! `h3-view`'s compose is for the battle: the frame's picture opaque in the viewer's colours
//! (design §1.9), the view's tiles through `draw_frame_raw` ([`crate::draw::raw`]) and the
//! unexplored tiles' second time through 0x47E820 ([`crate::draw::shadows`]), the objects' bodies
//! and flat objects through the flagged drawer ([`crate::draw::flagged`]), their shadows through
//! the shadow pass ([`crate::draw::shadow_pass`], §2.2), the gems through the plain drawer with
//! runs skipped (`h3_render::blit`). A DEF's palette is cycled once per scene
//! (design §1.6).

use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use h3_assets::def::Frame;
use h3_assets::palette::{Palette, Rgb};
use h3_assets::pcx::{IndexedImage, Pcx};
use h3_render::blit::{Drawer, Placement, draw_frame, draw_image};
use h3_render::palette::{ScreenPalette, with_player_colours};
use h3_render::screen::{PixelFormat, Screen};

use crate::batch::{Batch, Sprite};
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
    let mut palettes: BTreeMap<String, ScreenPalette> = BTreeMap::new();
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
                draw_image(&mut screen, &in_colours(image, players, *viewer), 0, 0);
            }
            Draw::Piece {
                pass,
                def,
                frame,
                x,
                y,
                mirror_x,
                mirror_y,
                clip,
            } => {
                let (sprite, f) = frame_of(batch, def, *frame)?;
                let palette = palettes
                    .entry(def.to_ascii_uppercase())
                    .or_insert_with(|| cycled(sprite, def, &scene.cycles));
                let at = Place {
                    x: *x,
                    y: *y,
                    mirror_x: *mirror_x,
                    mirror_y: *mirror_y,
                };
                match pass {
                    Pass::Flat | Pass::Body => {
                        draw::flagged(&mut screen, f, palette, at, *clip, None)
                    }
                    Pass::Shadow => draw::shadow_pass(&mut screen, f, at, *clip),
                    _ => draw::raw(&mut screen, f, palette, at, *clip),
                }
                if *pass == Pass::Shroud {
                    draw::shadows(&mut screen, f, at, *clip);
                }
            }
            Draw::Gem { def, frame, x, y } => {
                let (sprite, f) = frame_of(batch, def, *frame)?;
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
        }
    }
    Ok(screen)
}

/// A picture in a viewer's colours: an 8-bit one's entries 224–255 its 32 colours of the players'
/// palette (from entry 32 × the colour); a 24-bit one has no palette and is kept.
fn in_colours(image: &Pcx, players: &Palette, viewer: u8) -> Pcx {
    match image {
        Pcx::Indexed(indexed) => {
            let colours: Vec<Rgb> = players
                .0
                .iter()
                .skip(32 * usize::from(viewer))
                .take(32)
                .copied()
                .collect();
            Pcx::Indexed(Box::new(IndexedImage {
                palette: with_player_colours(&indexed.palette, &colours),
                ..(**indexed).clone()
            }))
        }
        Pcx::Rgb(_) => image.clone(),
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
    index: u32,
) -> Result<(&'b Sprite, &'b Frame), ComposeError> {
    let sprite = batch
        .sprite(def)
        .ok_or_else(|| ComposeError::Missing(def.into()))?;
    let frame = sprite
        .frame(0, index)
        .ok_or_else(|| ComposeError::Missing(format!("{def} group 0 frame {index}")))?;
    Ok((sprite, frame))
}

#[cfg(test)]
mod tests;
