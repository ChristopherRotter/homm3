//! The dialog items' drawers (E24 M3 T2a): each [`ui::Draw`] drawn as the executable draws its
//! item, from decoded assets a viewer's batch gives through [`Assets`]. Ported from `h3-view`'s
//! compositor (E17 M3 T6a to M5 T4b1, `docs/epics/E17-design.md` §9.7), which keeps its own copy
//! until the client lane takes this one (its inbox, E24 M3 T2a); the adventure screen's dialog
//! (`docs/epics/E24-design.md` §4.1) is drawn with it.

use alloc::string::String;
use alloc::vec::Vec;

use h3_assets::def::Frame;
use h3_assets::fnt::Font;
use h3_assets::palette::{Palette, Rgb};
use h3_assets::pcx::Pcx;
use h3_render::Bugs;
use h3_render::blit::{
    Drawer, Placement, darken, draw_frame, draw_image_clipped, draw_image_keyed_clipped,
    draw_image_keyed_in_clipped,
};
use h3_render::palette::{ScreenPalette, with_player_colours};
use h3_render::screen::Screen;
use h3_render::text::{TextBox, TextStyle, draw_text};

use crate::sprite::Sprite;
use crate::text::encode_1252;
use crate::ui::{Draw, PlayerColours};

/// The decoded assets a draw names, by name as the draw spells it (a viewer's batch).
pub trait Assets {
    /// The DEF `name`.
    fn sprite(&self, name: &str) -> Option<&Sprite>;
    /// The picture `name`.
    fn picture(&self, name: &str) -> Option<&Pcx>;
    /// The palette `name`.
    fn palette(&self, name: &str) -> Option<&Palette>;
    /// The font `name`.
    fn font(&self, name: &str) -> Option<&Font>;
}

/// The plain drawer with runs skipped (0x47C570 from 0x47BF00).
const PLAIN: Drawer = Drawer::Plain {
    skip_runs: true,
    colour: None,
};

/// Every asset `draw` names.
#[must_use]
pub fn names(draw: &Draw) -> Vec<&str> {
    fn player(p: &Option<PlayerColours>) -> Option<&str> {
        p.as_ref().map(|p| p.palette.as_str())
    }
    match draw {
        Draw::Picture {
            picture, player: p, ..
        } => core::iter::once(picture.as_str())
            .chain(player(p))
            .collect(),
        Draw::Button { def, player: p, .. } => {
            core::iter::once(def.as_str()).chain(player(p)).collect()
        }
        Draw::Text { item, .. } => alloc::vec![
            item.picture.as_str(),
            item.font.as_str(),
            item.font_palette.as_str(),
        ],
        Draw::Label { item, .. } => alloc::vec![item.font.as_str(), item.font_palette.as_str()],
        Draw::Darken { .. } => Vec::new(),
    }
}

/// Draws `draw` onto `screen` (in the screen's format) as the executable draws its item.
///
/// # Errors
/// The name of an asset, or a DEF's frame, that `assets` lacks.
pub fn draw(
    screen: &mut Screen,
    draw: &Draw,
    assets: &impl Assets,
    bugs: Bugs,
) -> Result<(), String> {
    match draw {
        Draw::Picture {
            picture,
            x,
            y,
            player,
            size,
        } => {
            // `H3DlgPcx` (0x450450 → 0x44FDB0): index 0 transparent, the picture's entries
            // 224-255 in the player's colours (0x450520) when it has one, at most the item's size
            // of it from its corner (E17 M5 T4a).
            let image = picture_of(assets, picture)?;
            let clip = size.map_or([i32::MIN, i32::MIN, i32::MAX, i32::MAX], |[w, h]| {
                [*x, *y, x.saturating_add(w), y.saturating_add(h)]
            });
            match (player, image) {
                (Some(colours), Pcx::Indexed(indexed)) => {
                    let of = colours_of(assets, colours)?;
                    let palette = with_player_colours(&indexed.palette, &of);
                    draw_image_keyed_in_clipped(screen, image, &palette, (*x, *y), clip);
                }
                _ => draw_image_keyed_clipped(screen, image, *x, *y, clip),
            }
        }
        Draw::Button {
            def,
            frame,
            x,
            y,
            mirrored,
            player,
        } => {
            // 0x456940: group 0's frame, frame 0 when the group has no such frame; the plain
            // drawer with runs skipped (0x47BF00 → 0x47C570); the DEF's entries 224-255 in a
            // player's colours when it has one (a DEF item's 0x4EAA4A, E17 M5 T4a; an adventure
            // button's 0x4569E0, E24 M3 T2a), mirrored as a DEF item's +0x3C says (E17 M5 T4b1).
            let (sprite, frame) = button_frame(assets, def, *frame)?;
            let palette = match player {
                Some(colours) => {
                    with_player_colours(&sprite.def.palette, &colours_of(assets, colours)?)
                }
                None => sprite.def.palette.clone(),
            };
            let at = Placement {
                x: *x,
                y: *y,
                mirrored: *mirrored,
            };
            let format = screen.format();
            draw_frame(
                screen,
                frame,
                &ScreenPalette::new(&palette, format),
                at,
                PLAIN,
            );
        }
        Draw::Darken { rect } => darken(screen, *rect),
        Draw::Text { item, x, y, text } => {
            // 0x5BC7F0: the item's picture, opaque (0x44FDB0), at most the item's size of it
            // (E17 M5 T3a: `spelroll.pcx` is a column wider), then its text (0x5BC5F0 →
            // 0x4B5490) in the item's box, the colour and alignment it was made with.
            let clip = [
                *x,
                *y,
                x.saturating_add(item.size[0]),
                y.saturating_add(item.size[1]),
            ];
            draw_image_clipped(screen, picture_of(assets, &item.picture)?, *x, *y, clip);
            let area = TextBox {
                x: *x,
                y: *y,
                width: item.size[0],
                height: item.size[1],
            };
            let font = (&item.font, &item.font_palette);
            text_in(
                screen,
                assets,
                font,
                (item.colour, item.align),
                area,
                text,
                bugs,
            )?;
        }
        Draw::Label {
            item,
            x,
            y,
            text,
            disabled,
        } => {
            // 0x5BC5F0 (E17 M5 T3a): the text in the item's box, its colour 2 more with the
            // state's bit 8 (0x5BC641); the background colour, 0 in every one built, fills
            // nothing.
            let area = TextBox {
                x: *x,
                y: *y,
                width: item.size[0],
                height: item.size[1],
            };
            let colour = item.colour.saturating_add(if *disabled { 2 } else { 0 });
            let font = (&item.font, &item.font_palette);
            text_in(screen, assets, font, (colour, item.align), area, text, bugs)?;
        }
    }
    Ok(())
}

/// The rectangle of the screen `draw` can change (left, top, right, bottom; the last two
/// excluded), as its drawer reaches (E24 M3 T4c, *provisional*): a picture's at most its item's
/// size of it, a button's its frame's canvas, a darkened rectangle itself; a text's or a label's
/// its box widened by its font's overhangs (E24 M3 T5a, [`text_reach`]); `None` when unknown: an
/// asset `assets` lacks (drawing it fails).
#[must_use]
pub fn reach(draw: &Draw, assets: &impl Assets) -> Option<[i32; 4]> {
    let side = |v: u32| i32::try_from(v).unwrap_or(i32::MAX);
    let rect = |x: i32, y: i32, [w, h]: [i32; 2]| [x, y, x.saturating_add(w), y.saturating_add(h)];
    match draw {
        Draw::Picture {
            picture,
            x,
            y,
            size,
            ..
        } => {
            let image = assets.picture(picture)?;
            let [w, h] = [side(image.width()), side(image.height())];
            let [w, h] = size.map_or([w, h], |[sw, sh]| [w.min(sw), h.min(sh)]);
            Some(rect(*x, *y, [w, h]))
        }
        Draw::Button {
            def, frame, x, y, ..
        } => {
            let (_, frame) = button_frame(assets, def, *frame).ok()?;
            Some(rect(
                *x,
                *y,
                [side(frame.full_width), side(frame.full_height)],
            ))
        }
        Draw::Darken { rect: [x, y, w, h] } => Some(rect(*x, *y, [*w, *h])),
        Draw::Text { item, x, y, .. } => {
            // The picture opaque inside the box (0x5BC7F0), the text over it.
            assets.picture(&item.picture)?;
            assets.palette(&item.font_palette)?;
            Some(text_reach(assets.font(&item.font)?, *x, *y, item.size))
        }
        Draw::Label { item, x, y, .. } => {
            assets.palette(&item.font_palette)?;
            Some(text_reach(assets.font(&item.font)?, *x, *y, item.size))
        }
    }
}

/// The rectangle the text drawer (0x4B5490, `h3_render::text`) can change for a text in the box
/// at (`x`, `y`) of `size` in `font` (E24 M3 T5a, design §4.7; *provisional*): the box's rows, its
/// columns widened by the font's widest overhang on each side. Its line drawer (0x4B5260) draws a
/// line only when it lies inside the box's height, skips the glyphs that would start left of the
/// box and ends before the first whose advance passes its right edge; so a glyph's pixels pass the
/// box only by a negative right space (its last pixels past its advance) or, after the first, a
/// negative left one (`smalfont.fnt`'s are 1 at most).
#[must_use]
pub fn text_reach(font: &Font, x: i32, y: i32, [w, h]: [i32; 2]) -> [i32; 4] {
    let over = |space: i32| space.saturating_neg().max(0);
    let left = font.glyphs.iter().map(|g| over(g.left)).max().unwrap_or(0);
    let right = font.glyphs.iter().map(|g| over(g.right)).max().unwrap_or(0);
    [
        x.saturating_sub(left),
        y,
        x.saturating_add(w).saturating_add(right),
        y.saturating_add(h),
    ]
}

/// A button's frame (0x456940): frame `index` of `def`'s group 0, frame 0 when the group has no
/// such frame.
fn button_frame<'a>(
    assets: &'a impl Assets,
    def: &str,
    index: u32,
) -> Result<(&'a Sprite, &'a Frame), String> {
    let count = assets
        .sprite(def)
        .and_then(|s| s.def.groups.iter().find(|g| g.id == 0))
        .map_or(0, |g| g.frames.len());
    let index = if usize::try_from(index).unwrap_or(usize::MAX) < count {
        index
    } else {
        0
    };
    frame_of(assets, def, index)
}

/// `text` in `area` by the text drawer (0x4B5490), in a font and the palette its colours come
/// from, with a colour argument and an alignment.
fn text_in(
    screen: &mut Screen,
    assets: &impl Assets,
    (font, palette): (&String, &String),
    (colour, alignment): (u32, u32),
    area: TextBox,
    text: &str,
    bugs: Bugs,
) -> Result<(), String> {
    let font = assets.font(font).ok_or_else(|| font.clone())?;
    let palette = assets.palette(palette).ok_or_else(|| palette.clone())?;
    let colours = ScreenPalette::new(palette, screen.format());
    let style = TextStyle {
        font,
        colours: &colours,
        colour,
        alignment,
        bugs,
    };
    draw_text(screen, &encode_1252(text).0, area, style);
    Ok(())
}

/// A player's 32 colours of the players' palette (from entry 32 × the player's colour).
#[must_use]
pub fn player_colours(players: &Palette, player: u8) -> Vec<Rgb> {
    players
        .0
        .iter()
        .skip(32 * usize::from(player))
        .take(32)
        .copied()
        .collect()
}

/// The 32 colours `colours` names.
fn colours_of(assets: &impl Assets, colours: &PlayerColours) -> Result<Vec<Rgb>, String> {
    let players = assets
        .palette(&colours.palette)
        .ok_or_else(|| colours.palette.clone())?;
    Ok(player_colours(players, colours.player))
}

fn picture_of<'a>(assets: &'a impl Assets, name: &str) -> Result<&'a Pcx, String> {
    assets.picture(name).ok_or_else(|| name.into())
}

/// Frame `index` of `def`'s group 0.
fn frame_of<'a>(
    assets: &'a impl Assets,
    def: &str,
    index: u32,
) -> Result<(&'a Sprite, &'a Frame), String> {
    let sprite = assets.sprite(def).ok_or_else(|| String::from(def))?;
    let frame = sprite
        .frame(0, index)
        .ok_or_else(|| alloc::format!("{def} group 0 frame {index}"))?;
    Ok((sprite, frame))
}

#[cfg(test)]
mod tests;
