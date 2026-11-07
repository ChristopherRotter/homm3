//! A draw list composed into one frame (E17 M1 T7): each [`Draw`] with the drawer the executable
//! uses for it (`docs/epics/E17-design.md` §3.3, §4), from a checkpoint's [`Batch`], into a screen
//! in the original's 16-bit format. The one place that decides how a named thing is drawn: a
//! graphics option (the user, 2026-10-13) changes this, not the scene; a dialog's items by the
//! common drawers both viewers share (`h3_ui::paint`, since E17 M6 T6a).

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use h3_assets::def::Frame;
use h3_assets::palette::Rgb;
use h3_render::Bugs;
use h3_render::blit::{
    Drawer, Placement, darken_under, draw_frame, draw_frame_rows, draw_image, draw_image_keyed,
    draw_image_keyed_clipped,
};
use h3_render::palette::{ScreenPalette, with_player_colours};
use h3_render::recolour;
use h3_render::screen::{PixelFormat, Screen};
use h3_render::text::{TextBox, TextStyle, draw_text};
use h3_render::tint::{Tint, adjust_hue_saturation};

use h3_ui::sprite::Sprite;

use crate::batch::Batch;
use crate::clip;
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
            Draw::AbsoluteObstacle { picture, x, y } | Draw::Grid { picture, x, y } => {
                draw_image_keyed(&mut screen, picture_of(batch, picture)?, *x, *y);
            }
            Draw::Shade { picture, x, y } => {
                darken_under(&mut screen, picture_of(batch, picture)?, *x, *y);
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
            Draw::Tiled { def, frame, area } => {
                tiled(&mut screen, batch, def, *frame, *area)?;
            }
            Draw::Within { rect, list } => {
                // Its own frame, of which the rectangle is copied (E17 M4 T2b).
                let own = compose(list, batch, layout, bugs)?;
                copy_rect(&mut screen, &own, *rect);
            }
            Draw::Dissolve {
                step,
                rect,
                measured,
                list,
                saved,
            } => {
                // The new picture on its own frame, blended into the saved one (E17 M5 T5a).
                let rect = match rect {
                    Some(r) => Some(*r),
                    None => measure(measured, batch, layout)?,
                };
                if let Some(rect) = rect {
                    // The back buffer a clipped redraw left, the original's slip (E17 M6 T6b2,
                    // CLT-Q67): the saved picture inside the rectangle.
                    if let Some(saved) = saved {
                        let back = back_buffer(saved, batch, layout, bugs)?;
                        copy_rect(&mut screen, &back, rect);
                    }
                    let new = compose(list, batch, layout, bugs)?;
                    screen.dissolve_rect(&new, rect.map(i64::from), *step);
                }
            }
            Draw::Shaken {
                offsets,
                rect,
                list,
            } => {
                // Earthquake's shake (E17 M4 T2c-2, 0x5A7D38): the saved frame copied at each
                // offset in turn over the step before's, clipped to the frame; the copy's
                // rectangle shown over the saved frame (copied in every profile, CLT-Q19 (6)).
                let saved = compose(list, batch, layout, bugs)?;
                let mut shaken = saved.clone();
                for &[dx, dy] in offsets {
                    let [w, h] = [shaken.width(), shaken.height()].map(i64::from);
                    for y in 0..h {
                        for x in 0..w {
                            let from = (x - i64::from(dx), y - i64::from(dy));
                            if let Some(p) = saved.pixel(from.0, from.1) {
                                shaken.set(x, y, p);
                            }
                        }
                    }
                }
                copy_rect(&mut screen, &saved, [0, 0, i32::MAX, i32::MAX]);
                copy_rect(&mut screen, &shaken, *rect);
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
            Draw::Explosion {
                def,
                frame,
                x,
                y,
                rows,
            } => {
                // 0x446453: the plain drawer, group 0; the original's slip, only the canvas's top
                // rows, reproduced for the comparisons alone (CLT-Q22, as CLT-Q19).
                let rows = match bugs {
                    Bugs::Original => *rows,
                    Bugs::Fixed => u32::MAX,
                };
                let (sprite, frame) = frame_of(batch, def, 0, *frame)?;
                let palette = ScreenPalette::new(&sprite.def.palette, FORMAT);
                let at = Placement {
                    x: *x,
                    y: *y,
                    mirrored: false,
                };
                draw_frame_rows(&mut screen, frame, &palette, at, PLAIN, rows);
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
                let colours = h3_ui::paint::player_colours(players, *player);
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
            Draw::CountBox {
                x, y, count, tint, ..
            } => {
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
            // The dialog's items (E17 M6 T6a): the common drawers' (`h3_ui::paint`, ported from
            // this compositor at E24 M3 T2a; design §9.7).
            Draw::Dialog(dialog) => {
                h3_ui::paint::draw(&mut screen, dialog, batch, bugs)
                    .map_err(ComposeError::Missing)?;
            }
            Draw::Cursor {
                def,
                frame,
                x,
                y,
                hot,
            } => {
                // 0x50D8B0: group 0's frame into the cursor's surface by the plain drawer with runs
                // skipped (0x47BEB0 → 0x47C570), which the screen's overlay draws over all, its
                // canvas at the mouse less the hot spot. The original's bug: the mouse's x
                // rounded down to even first (0x50CE04, design §9.9; the user, 2026-10-20: fixed,
                // reproduced for the comparisons only).
                let x = match bugs {
                    Bugs::Original => *x & !1,
                    Bugs::Fixed => *x,
                };
                plain(
                    &mut screen,
                    batch,
                    def,
                    (0, *frame),
                    x - hot[0],
                    *y - hot[1],
                    false,
                )?;
            }
            Draw::Video {
                start,
                looped,
                since,
                x,
                y,
            } => {
                // `BinkCopyToBuffer` (0x44DB19): the frame's pixels as they are (E17 M5 T4b2b).
                let video = |name: &String| {
                    batch
                        .video(name)
                        .ok_or_else(|| ComposeError::Missing(name.clone()))
                };
                let first = video(start)?;
                let second = looped.as_ref().map(video).transpose()?;
                let timing = second.map(clip::Video::timing);
                let on = clip::shown(*since, first.timing(), timing, bugs);
                let (shown, frame) = on.map_or((first, usize::MAX), |(part, frame)| {
                    let shown = if part == clip::Part::Loop {
                        second.unwrap_or(first)
                    } else {
                        first
                    };
                    (shown, frame)
                });
                let pixels = shown.frames.get(frame).map_or(&[][..], Vec::as_slice);
                copy_pixels(&mut screen, pixels, shown.width, (*x, *y));
            }
            Draw::Wall {
                picture,
                x,
                y,
                clip,
            } => {
                // A siege's part (0x4957B0 → 0x44FDB0, index 0 transparent; E17 M4 T1b).
                let image = picture_of(batch, picture)?;
                match clip {
                    Some(clip) => draw_image_keyed_clipped(&mut screen, image, *x, *y, *clip),
                    None => draw_image_keyed(&mut screen, image, *x, *y),
                }
            }
            Draw::Creature {
                def,
                group,
                frame,
                anchor,
                mirrored,
                outline,
                ..
            }
            | Draw::Shooter {
                def,
                group,
                frame,
                anchor,
                mirrored,
                outline,
                ..
            } => {
                // 0x43E140: the shadowed drawer, the canvas placed by facing (§4.2); a siege's
                // shooter only its canvas's top rows (0x495090, E17 M4 T1b).
                let rows = match item {
                    Draw::Shooter { rows, .. } => *rows,
                    _ => u32::MAX,
                };
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
                let mut palette = ScreenPalette::new(&sprite.def.palette, FORMAT);
                // A stack's palette effect, for its sprite alone (0x43E280; E17 M5 T5b1).
                if let Draw::Creature {
                    effect: Some(effect),
                    ..
                } = item
                {
                    recolour::apply(&mut palette, FORMAT, *effect);
                }
                draw_frame_rows(
                    &mut screen,
                    frame,
                    &palette,
                    at,
                    Drawer::Shadowed { colour },
                    rows,
                );
            }
        }
    }
    Ok(screen)
}

/// The rectangle a dissolve's `measured` items cover (`battle_refresh_creatures` 0x495BF0; E17
/// M5 T5a, design §7.25), left, top, right and bottom, exclusive: each item's, cut to the
/// battlefield (0x694F18: the layout's mouse field, its bounds inclusive), widens the dirty
/// rectangle edge by edge from the empty one (799, 599, 0, 0; 0x6AACE8). A creature's is its
/// frame's content box as `battle_item_rect` 0x495F50 places it (mirrored in the DEF's width), a
/// count box's its picture's (`battle_draw_pcx` 0x4958E0), a magic animation's its DEF's whole
/// canvas at its corner (`battle_draw_magic_frame` 0x4953B0). None when nothing widened it.
fn measure(
    measured: &[Draw],
    batch: &Batch,
    layout: &BattleLayout,
) -> Result<Option<[i32; 4]>, ComposeError> {
    let mut dirty = [799, 599, 0, 0];
    for item in measured {
        let Some(cut) = item_box(item, batch, layout)? else {
            continue;
        };
        dirty = [
            dirty[0].min(cut[0]),
            dirty[1].min(cut[1]),
            dirty[2].max(cut[2]),
            dirty[3].max(cut[3]),
        ];
    }
    let [x0, y0, x1, y1] = dirty;
    Ok((x0 <= x1 && y0 <= y1).then_some([x0, y0, x1 + 1, y1 + 1]))
}

/// An item's rectangle as a measuring or clipped redraw takes it (left, top, right, bottom, all
/// inclusive), cut to the battlefield (0x694F18: the layout's mouse field, its bounds inclusive);
/// none for an item of a kind not measured. A creature's, a hero's and a flag's is its frame's
/// content box as `battle_item_rect` 0x495F50 places it (mirrored in the DEF's width; a
/// creature's canvas at its anchor less the layout's offset, `battle_draw_sprite` 0x4951B0, a
/// hero's or a flag's at its corner, `battle_draw_hero_def` 0x4952B0), a count box's its
/// picture's (`battle_draw_pcx` 0x4958E0), a magic animation's its DEF's whole canvas at its
/// corner (`battle_draw_magic_frame` 0x4953B0), an obstacle's too (`draw_battle_obstacle`
/// 0x495525, E17 M6 T6b2).
fn item_box(
    item: &Draw,
    batch: &Batch,
    layout: &BattleLayout,
) -> Result<Option<[i32; 4]>, ComposeError> {
    let [width, height] = layout.mouse.field;
    let [x0, y0, x1, y1] = if let Draw::CountBox { x, y, .. } = item {
        let picture = picture_of(batch, &layout.count_box.picture)?;
        let size = [picture.width(), picture.height()].map(|v| i32::try_from(v).unwrap_or(0));
        [*x, *y, x + size[0] - 1, y + size[1] - 1]
    } else {
        // Each sprite's DEF, group and frame, its canvas's corner (a creature's: its anchor),
        // whether that is a creature's anchor, mirrored, and whether its whole canvas counts.
        let (def, group, frame, at, stack, mirrored, whole) = match item {
            Draw::Creature {
                def,
                group,
                frame,
                anchor,
                mirrored,
                ..
            } => (def, *group, *frame, *anchor, true, *mirrored, false),
            Draw::Hero {
                def,
                group,
                frame,
                x,
                y,
                mirrored,
            } => (def, *group, *frame, [*x, *y], false, *mirrored, false),
            Draw::Flag {
                def, frame, x, y, ..
            } => (def, 0, *frame, [*x, *y], false, false, false),
            Draw::Magic {
                def, frame, x, y, ..
            } => (def, 0, *frame, [*x, *y], false, false, true),
            Draw::Obstacle {
                def,
                group,
                frame,
                x,
                y,
            } => (def, *group, *frame, [*x, *y], false, false, true),
            _ => return Ok(None),
        };
        let (sprite, frame) = frame_of(batch, def, group, frame)?;
        let canvas = [sprite.def.width, sprite.def.height].map(|v| i32::try_from(v).unwrap_or(0));
        let [dx, dy] = layout.stack_canvas;
        // A creature's canvas lies left of its anchor by the offset, a mirrored one by its width
        // less the offset.
        let corner = match (stack, mirrored) {
            (false, _) => at,
            (true, false) => [at[0].saturating_sub(dx), at[1].saturating_sub(dy)],
            (true, true) => [
                at[0].saturating_sub(canvas[0]).saturating_add(dx),
                at[1].saturating_sub(dy),
            ],
        };
        if whole {
            [
                corner[0],
                corner[1],
                corner[0] + canvas[0] - 1,
                corner[1] + canvas[1] - 1,
            ]
        } else {
            let size = [frame.width, frame.height].map(|v| i32::try_from(v).unwrap_or(0));
            let y = corner[1].saturating_add(frame.top);
            let (x0, x1) = if mirrored {
                // 0x495F80: the canvas's x plus its width less the margin and the width.
                let right = corner[0]
                    .saturating_add(canvas[0])
                    .saturating_sub(frame.left);
                (right.saturating_sub(size[0]), right.saturating_sub(1))
            } else {
                let x = corner[0].saturating_add(frame.left);
                (x, x.saturating_add(size[0]).saturating_sub(1))
            };
            [x0, y, x1, y.saturating_add(size[1]).saturating_sub(1)]
        }
    };
    Ok(Some([
        x0.max(0),
        y0.max(0),
        x1.min(width - 1),
        y1.min(height - 1),
    ]))
}

/// The back buffer a creature's clipped redraws left (E17 M6 T6b2, CLT-Q67, design §7.25;
/// [`crate::scene::Saved`]): the background composed over the whole frame
/// (`compose_battle_background` 0x493CF0), then each other item of the saved battle whose
/// rectangle meets the one the clip's items cover, whole, in order (`battle_draw_sprite` 0x495214
/// and the other drawers: an item outside it is skipped, never cut); an item of a kind not
/// measured (a siege's parts, a projectile) drawn as the screen shows it (*provisional*: a
/// creature's cast draws none).
fn back_buffer(
    saved: &crate::scene::Saved,
    batch: &Batch,
    layout: &BattleLayout,
    bugs: Bugs,
) -> Result<Screen, ComposeError> {
    let clip = measure(&saved.clip, batch, layout)?;
    let mut kept = Vec::new();
    for item in &saved.list {
        let drawn = match item {
            Draw::Background { .. }
            | Draw::AbsoluteObstacle { .. }
            | Draw::Shade { .. }
            | Draw::Grid { .. } => true,
            _ => match item_box(item, batch, layout)? {
                // Exclusive on the clip's right and bottom (`measure`).
                Some([x0, y0, x1, y1]) => clip.is_some_and(|[cx0, cy0, cx1, cy1]| {
                    x0 < cx1 && x1 >= cx0 && y0 < cy1 && y1 >= cy0
                }),
                None => true,
            },
        };
        if drawn {
            kept.push(item.clone());
        }
    }
    compose(&kept, batch, layout, bugs)
}

/// The plain drawer with runs skipped (`def_draw_plain` 0x47BCF0 → 0x47C570).
const PLAIN: Drawer = Drawer::Plain {
    skip_runs: true,
    colour: None,
};

/// The pixels of `from` in `rect` (left, top, right, bottom, exclusive; clipped to `from`)
/// copied into `screen` at the same places.
fn copy_rect(screen: &mut Screen, from: &Screen, rect: [i32; 4]) {
    let [w, h] = [from.width(), from.height()].map(i64::from);
    let [left, top, right, bottom] = rect.map(i64::from);
    for y in top.max(0)..bottom.min(h) {
        for x in left.max(0)..right.min(w) {
            // Inside `from`: a pixel every time.
            screen.set(x, y, from.pixel(x, y).unwrap_or_default());
        }
    }
}

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

/// The columns and rows a tiling counts from, whatever the area (`mass_damage_handler`'s 0x31F
/// and 0x257: ceil(800 / W), ceil(600 / H)).
const TILES_FROM: [i64; 2] = [800, 600];

/// Frame `frame` of `def`'s group 0 tiled from (0, 0) over `area`, plain (0x47C170 without its
/// translucent flag): each tile its canvas cut to the width and height left of the area
/// (Armageddon's, 0x5A4FEF to 0x5A50AD; E17 M3 T7c-2).
fn tiled(
    screen: &mut Screen,
    batch: &Batch,
    def: &str,
    frame: u32,
    area: [i32; 2],
) -> Result<(), ComposeError> {
    let (sprite, frame) = frame_of(batch, def, 0, frame)?;
    let palette = ScreenPalette::new(&sprite.def.palette, FORMAT);
    let (w, h) = (
        i64::from(frame.full_width.max(1)),
        i64::from(frame.full_height.max(1)),
    );
    let (columns, rows) = ((TILES_FROM[0] + w - 1) / w, (TILES_FROM[1] + h - 1) / h);
    for row in 0..rows {
        let height = h.min(i64::from(area[1]) - row * h);
        for column in 0..columns {
            let width = w.min(i64::from(area[0]) - column * w);
            let (Ok(width), Ok(height)) = (u32::try_from(width), u32::try_from(height)) else {
                continue;
            };
            let cut = Frame {
                full_width: width,
                full_height: height,
                ..frame.clone()
            };
            let at = Placement {
                x: i32::try_from(column * w).unwrap_or(i32::MAX),
                y: i32::try_from(row * h).unwrap_or(i32::MAX),
                mirrored: false,
            };
            draw_frame(screen, &cut, &palette, at, PLAIN);
        }
    }
    Ok(())
}

/// Rows of `width` 5-6-5 pixels copied onto the screen with their corner at `at`, clipped to it.
fn copy_pixels(screen: &mut Screen, pixels: &[u16], width: usize, at: (i32, i32)) {
    let screen_width = usize::try_from(screen.width()).unwrap_or(0);
    let screen_height = i64::from(screen.height());
    let out = screen.pixels_mut();
    for (row, line) in pixels.chunks(width.max(1)).enumerate() {
        let y = i64::from(at.1) + i64::try_from(row).unwrap_or(i64::MAX);
        if !(0..screen_height).contains(&y) {
            continue;
        }
        for (column, &pixel) in line.iter().enumerate() {
            let x = i64::from(at.0) + i64::try_from(column).unwrap_or(i64::MAX);
            let inside = usize::try_from(x).ok().filter(|&x| x < screen_width);
            let at = inside.and_then(|x| usize::try_from(y).ok().map(|y| y * screen_width + x));
            if let Some(slot) = at.and_then(|i| out.get_mut(i)) {
                *slot = pixel;
            }
        }
    }
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
