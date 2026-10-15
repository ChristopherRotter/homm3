//! A draw list composed into one frame (E17 M1 T7): each [`Draw`] with the drawer the executable
//! uses for it (`docs/epics/E17-design.md` §3.3, §4), from a checkpoint's [`Batch`], into a screen
//! in the original's 16-bit format. The one place that decides how a named thing is drawn: a
//! graphics option (the user, 2026-10-13) changes this, not the scene.

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use h3_assets::def::Frame;
use h3_assets::palette::Rgb;
use h3_render::blit::{Drawer, Placement, draw_frame, draw_image, draw_image_keyed};
use h3_render::palette::ScreenPalette;
use h3_render::screen::{PixelFormat, Screen};

use crate::batch::{Batch, Sprite};
use crate::layout::BattleLayout;
use crate::scene::Draw;

/// The screen's pixel format (design §3.2: the executable's 16-bit 5-6-5 screen).
pub const FORMAT: PixelFormat = PixelFormat::RGB565;

/// Why a draw list does not compose.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComposeError {
    /// The layout's frame is larger than a screen may be.
    Frame([u32; 2]),
    /// An asset, a group or a frame the list names is not in the batch.
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

/// Draws `list` in order onto a fresh frame of `layout`'s size (black where nothing is drawn).
///
/// # Errors
/// A frame too large, or anything the list names missing from `batch`.
pub fn compose(
    list: &[Draw],
    batch: &Batch,
    layout: &BattleLayout,
) -> Result<Screen, ComposeError> {
    let [width, height] = layout.frame;
    let mut screen = Screen::new(width, height, FORMAT).ok_or(ComposeError::Frame(layout.frame))?;
    for item in list {
        match item {
            Draw::Background { picture, x, y } => {
                draw_image(&mut screen, picture_of(batch, picture)?, *x, *y);
            }
            Draw::AbsoluteObstacle { picture, x, y } => {
                draw_image_keyed(&mut screen, picture_of(batch, picture)?, *x, *y);
            }
            Draw::Obstacle {
                def,
                group,
                frame,
                x,
                y,
            } => {
                // 0x495500: the plain drawer with runs skipped, never mirrored (§4.4).
                plain(&mut screen, batch, def, (*group, *frame), *x, *y, false)?;
            }
            Draw::Projectile {
                def,
                frame,
                x,
                y,
                mirrored,
            } => {
                // 0x43F78A: the same drawer, group 0, mirrored toward a target on the left.
                plain(&mut screen, batch, def, (0, *frame), *x, *y, *mirrored)?;
            }
            Draw::Ray(ray) => {
                // 0x5A55BF: each pixel its colour entry scaled into the masks, written straight
                // into the screen.
                let colours: Vec<u16> = ray
                    .colours
                    .iter()
                    .map(|&[r, g, b]| FORMAT.scale(Rgb { r, g, b }))
                    .collect();
                // An entry beyond the colours takes the last; none: black (the executable's
                // colour 0).
                let last = colours.last().copied().unwrap_or(0);
                for stroke in &ray.strokes {
                    for ([x, y], entry) in stroke.pixels() {
                        let pixel = colours.get(entry).copied().unwrap_or(last);
                        screen.set(i64::from(x), i64::from(y), pixel);
                    }
                }
            }
            Draw::Creature {
                def,
                group,
                frame,
                anchor,
                mirrored,
                outline,
            } => {
                // 0x43E140: the shadowed drawer, the canvas placed by facing (§4.2).
                let (sprite, frame) = frame_of(batch, def, *group, *frame)?;
                let [dx, dy] = layout.stack_canvas;
                let width = i32::try_from(sprite.def.width).unwrap_or(i32::MAX);
                // Saturating: a hostile DEF width or a far anchor clips instead of overflowing.
                let x = if *mirrored {
                    anchor[0].saturating_sub(width).saturating_add(dx)
                } else {
                    anchor[0].saturating_sub(dx)
                };
                let at = Placement {
                    x,
                    y: anchor[1].saturating_sub(dy),
                    mirrored: *mirrored,
                };
                let colour = match outline {
                    Some(entry) => {
                        let name = &layout.outline_palette;
                        let palette = batch
                            .palette(name)
                            .ok_or_else(|| ComposeError::Missing(name.clone()))?;
                        Some(FORMAT.pack(palette.color(*entry)))
                    }
                    None => None,
                };
                let palette = ScreenPalette::new(&sprite.def.palette, FORMAT);
                draw_frame(
                    &mut screen,
                    frame,
                    &palette,
                    at,
                    Drawer::Shadowed { colour },
                );
            }
        }
    }
    Ok(screen)
}

/// A frame drawn plain with runs skipped (`def_draw_plain` 0x47BCF0 → 0x47C570), its canvas's
/// top-left at (`x`, `y`).
fn plain(
    screen: &mut Screen,
    batch: &Batch,
    def: &str,
    (group, index): (u32, u32),
    x: i32,
    y: i32,
    mirrored: bool,
) -> Result<(), ComposeError> {
    let (sprite, frame) = frame_of(batch, def, group, index)?;
    let drawer = Drawer::Plain {
        skip_runs: true,
        colour: None,
    };
    let palette = ScreenPalette::new(&sprite.def.palette, FORMAT);
    draw_frame(
        screen,
        frame,
        &palette,
        Placement { x, y, mirrored },
        drawer,
    );
    Ok(())
}

fn picture_of<'b>(batch: &'b Batch, name: &str) -> Result<&'b h3_assets::pcx::Pcx, ComposeError> {
    batch
        .picture(name)
        .ok_or_else(|| ComposeError::Missing(name.into()))
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
