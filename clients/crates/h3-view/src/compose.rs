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
use h3_render::Bugs;
use h3_render::blit::{Drawer, Placement, draw_frame, draw_image, draw_image_keyed};
use h3_render::palette::{ScreenPalette, with_player_colours};
use h3_render::screen::{PixelFormat, Screen};
use h3_render::text::{TextBox, TextStyle, draw_text};
use h3_render::tint::{Tint, adjust_hue_saturation};

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

/// Draws `list` in order onto a fresh frame of `layout`'s size (black where nothing is drawn),
/// with the original's drawing bugs fixed or, for the comparisons with its captures, reproduced
/// (`bugs`; the user, 2026-10-15).
///
/// # Errors
/// A frame too large, or anything the list names missing from `batch`.
pub fn compose(
    list: &[Draw],
    batch: &Batch,
    layout: &BattleLayout,
    bugs: Bugs,
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
            Draw::Magic {
                def,
                frame,
                x,
                y,
                mirrored,
                translucent,
            } => {
                // 0x4953B0 → 0x47EF60: with its flag the translucent drawer, else the plain one
                // with runs skipped (E17 M3 T3b, §7.15).
                let drawer = if *translucent {
                    Drawer::Translucent
                } else {
                    PLAIN
                };
                let at = Placement {
                    x: *x,
                    y: *y,
                    mirrored: *mirrored,
                };
                drawn(&mut screen, batch, def, (0, *frame), at, drawer)?;
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
            Draw::Flag {
                def,
                frame,
                x,
                y,
                palette,
                player,
            } => {
                // 0x4952B0 as a hero's, the DEF's entries 224-255 its player's 32 of the players'
                // palette (0x4634D5, 0x5FFE20: from entry 32 x the player's colour).
                let (sprite, frame) = frame_of(batch, def, 0, *frame)?;
                let players = batch
                    .palette(palette)
                    .ok_or_else(|| ComposeError::Missing(palette.clone()))?;
                let colours: Vec<Rgb> = players
                    .0
                    .iter()
                    .skip(32 * usize::from(*player))
                    .take(32)
                    .copied()
                    .collect();
                let palette =
                    ScreenPalette::new(&with_player_colours(&sprite.def.palette, &colours), FORMAT);
                let at = Placement {
                    x: *x,
                    y: *y,
                    mirrored: false,
                };
                draw_frame(
                    &mut screen,
                    frame,
                    &palette,
                    at,
                    Drawer::Shadowed { colour: None },
                );
            }
            Draw::Hero {
                def,
                group,
                frame,
                x,
                y,
                mirrored,
            } => {
                // `battle_draw_hero_def` 0x4952B0: the shadowed drawer without an outline, the
                // canvas at (x, y), mirrored in it (§4.5).
                let (sprite, frame) = frame_of(batch, def, *group, *frame)?;
                let palette = ScreenPalette::new(&sprite.def.palette, FORMAT);
                let at = Placement {
                    x: *x,
                    y: *y,
                    mirrored: *mirrored,
                };
                draw_frame(
                    &mut screen,
                    frame,
                    &palette,
                    at,
                    Drawer::Shadowed { colour: None },
                );
            }
            Draw::CountBox { x, y, count, tint } => {
                // 0x43E568: the picture, index 0 transparent (0x4958E0 → 0x44FDB0), its inside
                // tinted (0x44E940), the count's `%d` in the font (0x4B5490), centred.
                let b = &layout.count_box;
                draw_image_keyed(&mut screen, picture_of(batch, &b.picture)?, *x, *y);
                let [[dx, dy], size] = b.tint_rect;
                let [hue, saturation] = *tint;
                adjust_hue_saturation(
                    &mut screen,
                    [x.saturating_add(dx), y.saturating_add(dy)],
                    size,
                    Tint { hue, saturation },
                );
                let font = batch
                    .font(&b.font)
                    .ok_or_else(|| ComposeError::Missing(b.font.clone()))?;
                let palette = batch
                    .palette(&b.font_palette)
                    .ok_or_else(|| ComposeError::Missing(b.font_palette.clone()))?;
                let area = TextBox {
                    x: *x,
                    y: *y,
                    width: b.text_size[0],
                    height: b.text_size[1],
                };
                let text = format!("{count}");
                let colours = ScreenPalette::new(palette, FORMAT);
                let style = TextStyle {
                    font,
                    colours: &colours,
                    colour: b.text_colour,
                    alignment: b.text_align,
                    bugs,
                };
                draw_text(&mut screen, text.as_bytes(), area, style);
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

/// The plain drawer with runs skipped (`def_draw_plain` 0x47BCF0 → 0x47C570).
const PLAIN: Drawer = Drawer::Plain {
    skip_runs: true,
    colour: None,
};

/// A frame drawn plain with runs skipped, its canvas's top-left at (`x`, `y`).
fn plain(
    screen: &mut Screen,
    batch: &Batch,
    def: &str,
    at: (u32, u32),
    x: i32,
    y: i32,
    mirrored: bool,
) -> Result<(), ComposeError> {
    drawn(screen, batch, def, at, Placement { x, y, mirrored }, PLAIN)
}

/// A frame of `def` drawn with its own palette by `drawer`, placed by `at`.
fn drawn(
    screen: &mut Screen,
    batch: &Batch,
    def: &str,
    (group, index): (u32, u32),
    at: Placement,
    drawer: Drawer,
) -> Result<(), ComposeError> {
    let (sprite, frame) = frame_of(batch, def, group, index)?;
    let palette = ScreenPalette::new(&sprite.def.palette, FORMAT);
    draw_frame(screen, frame, &palette, at, drawer);
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
