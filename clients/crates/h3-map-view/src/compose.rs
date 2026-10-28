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
//! runs skipped (0x40F2F0: the raw drawer's literal pixels, E24 M3 T2b2), the dialog's items by
//! `h3_ui::paint` (E24 M3 T2a). A DEF's palette is cycled once per scene (design §1.6). A
//! [`Composer`] keeps a frame from scene to scene and draws only what changed (E24 M3 T2b1,
//! T2b2; design §4.2, §4.3).

use alloc::collections::BTreeMap;
use alloc::collections::btree_map::Entry;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use h3_assets::palette::{Palette, Rgb};
use h3_assets::pcx::Pcx;
use h3_render::Bugs;
use h3_render::blit::draw_image;
use h3_render::palette::{ScreenPalette, with_player_colours};
use h3_render::screen::{PixelFormat, Screen};

use h3_ui::sprite::Sprite;

use crate::batch::{Batch, Drawable};
use crate::draw::{self, FrameSpans, Place};
use crate::scene::{Cycle, Draw, Pass, Scene, ViewPlace};

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
    let mut composer = Composer::new(frame)?;
    composer.compose(scene, batch)?;
    Ok(composer.screen)
}

/// A frame kept from scene to scene (E24 M3 T2b1, T2b2, *provisional*; design §4.2, §4.3): the
/// scene's leading draws that do not move with the view (the frame's picture and the dialog's
/// items) are drawn on black once as the background while they stay the same; of the view only
/// what changed is drawn again. The view's 32 × 32 cells are compared with the last scene's: a
/// cell whose draws (each draw whose rectangle meets it, in order, with its DEF's palette steps)
/// are the last scene's is left, the others restored from the background and drawn again, each
/// draw cut to them. A view moved by some pixels (a scroll, a walk) moves the clip's pixels by as
/// many first and compares each cell with the last scene's draws moved, so only what it brings in
/// and what changed is drawn; a jump draws the whole view. What it draws is [`compose`]'s, pixel
/// for pixel: every drawer puts a pixel from the screen's pixel there and the draws over it.
#[derive(Debug, Clone)]
pub struct Composer {
    screen: Screen,
    background: Option<Background>,
    /// Each DEF's palette (by its name as the scenes spell it) for its cycles' steps.
    palettes: BTreeMap<String, (Vec<u32>, ScreenPalette)>,
    /// The last scene's draws over the background.
    last: Option<Last>,
    /// The rectangles the last scene changed.
    damage: Vec<[i32; 4]>,
    /// The cells' draws, this scene's and the last's moved (kept for their space).
    now: Buckets,
    was: Buckets,
}

/// The background: the leading draws that do not move and the bugs they were drawn with, what
/// they drew on black, and the view's clip when the background is one colour across it (only
/// then may the clip's pixels move: a moved pixel of the background is the background).
#[derive(Debug, Clone)]
struct Background {
    still: Vec<Draw>,
    bugs: Bugs,
    screen: Screen,
    uniform: Option<[i32; 4]>,
}

/// A scene's draws over the background as drawn (those of `draws` from `from` on): each with its
/// rectangle, the view's place, the palette cycles and the flag palette.
#[derive(Debug, Clone)]
struct Last {
    draws: Vec<Draw>,
    from: usize,
    rects: Vec<[i32; 4]>,
    view: Option<ViewPlace>,
    cycles: Vec<Cycle>,
    flags: String,
}

impl Composer {
    /// A black `frame`-sized screen.
    ///
    /// # Errors
    /// A frame too large.
    pub fn new(frame: [u32; 2]) -> Result<Self, ComposeError> {
        let [width, height] = frame;
        let screen = Screen::new(width, height, FORMAT).ok_or(ComposeError::Frame(frame))?;
        Ok(Self {
            screen,
            background: None,
            palettes: BTreeMap::new(),
            last: None,
            damage: Vec::new(),
            now: Buckets::default(),
            was: Buckets::default(),
        })
    }

    /// The frame as the last scene left it.
    #[must_use]
    pub fn screen(&self) -> &Screen {
        &self.screen
    }

    /// The rectangles of the frame the last scene changed (left, top, right, bottom; the last two
    /// exclusive), on the screen; they may overlap. Everything outside them is as the scene
    /// before left it.
    #[must_use]
    pub fn damage(&self) -> &[[i32; 4]] {
        &self.damage
    }

    /// Draws `scene` over the kept frame: the background again only when its draws changed, of
    /// the rest only what changed.
    ///
    /// # Errors
    /// As [`compose`]; the next scene then draws everything again.
    pub fn compose(&mut self, scene: &Scene, batch: &Batch) -> Result<&Screen, ComposeError> {
        let still = self.drawn(scene, batch)?;
        if let Some(last) = &mut self.last {
            last.draws = scene.draws.get(still..).unwrap_or_default().to_vec();
            last.from = 0;
        }
        Ok(&self.screen)
    }

    /// As [`Composer::compose`], keeping the scene's draws rather than a copy of them (E24 M3
    /// T2b2: the window's frames).
    ///
    /// # Errors
    /// As [`compose`]; the next scene then draws everything again.
    pub fn compose_owned(&mut self, scene: Scene, batch: &Batch) -> Result<&Screen, ComposeError> {
        let still = self.drawn(&scene, batch)?;
        if let Some(last) = &mut self.last {
            last.draws = scene.draws;
            last.from = still;
        }
        Ok(&self.screen)
    }

    /// Draws `scene`; how many of its leading draws are the background's. A scene that fails
    /// leaves the next to draw everything.
    fn drawn(&mut self, scene: &Scene, batch: &Batch) -> Result<usize, ComposeError> {
        self.damage.clear();
        let drawn = self.draw(scene, batch);
        if drawn.is_err() {
            self.background = None;
            self.last = None;
            self.damage.clear();
            self.damage.push(whole(&self.screen));
        }
        drawn
    }

    fn draw(&mut self, scene: &Scene, batch: &Batch) -> Result<usize, ComposeError> {
        let still = scene
            .draws
            .iter()
            .take_while(|d| matches!(d, Draw::Frame { .. } | Draw::Dialog(_)))
            .count();
        let (still, moving) = scene.draws.split_at(still);
        let all = whole(&self.screen);
        let rects = moving
            .iter()
            .map(|d| rect_of(d, batch, all))
            .collect::<Result<Vec<_>, _>>()?;
        let last = self.last.take();
        let Self {
            screen,
            background,
            palettes,
            damage,
            now,
            was,
            ..
        } = self;
        let mut pen = Pen::new(scene, batch, palettes);
        match background {
            Some(b) if b.still.as_slice() == still && b.bugs == scene.bugs => {
                let usable = last.as_ref().filter(|l| {
                    l.flags == scene.flags
                        && !moving
                            .iter()
                            .any(|d| matches!(d, Draw::Frame { .. } | Draw::Dialog(_)))
                });
                if let Some(last) = usable {
                    let view = Changed {
                        moving,
                        rects: &rects,
                        view: scene.view,
                        cycles: &scene.cycles,
                    };
                    view.draw(screen, b, last, &mut pen, [now, was], damage)?;
                } else {
                    let covered = rects.iter().copied().fold(None, |r, d| union(r, Some(d)));
                    let before = last.iter().flat_map(|l| l.rects.iter().copied());
                    if let Some(rect) = before.fold(covered, |r, d| union(r, Some(d))) {
                        let rect = meet(rect, all).unwrap_or_default();
                        screen.copy_rect(&b.screen, rect.map(i64::from));
                        damage.push(rect);
                    }
                    for item in moving {
                        pen.draw(screen, item, None)?;
                    }
                }
            }
            _ => {
                let mut drawn = screen.clone();
                drawn.fill(0);
                for item in still {
                    pen.draw(&mut drawn, item, None)?;
                }
                let uniform = scene
                    .view
                    .map(|v| v.clip)
                    .filter(|&clip| one_colour(&drawn, clip));
                screen.clone_from(&drawn);
                *background = Some(Background {
                    still: still.to_vec(),
                    bugs: scene.bugs,
                    screen: drawn,
                    uniform,
                });
                for item in moving {
                    pen.draw(screen, item, None)?;
                }
                damage.push(all);
            }
        }
        // The draws are the caller's to keep.
        self.last = Some(Last {
            draws: Vec::new(),
            from: 0,
            rects,
            view: scene.view,
            cycles: scene.cycles.clone(),
            flags: scene.flags.clone(),
        });
        Ok(still.len())
    }
}

impl Last {
    /// The draws over the background.
    fn moving(&self) -> &[Draw] {
        self.draws.get(self.from..).unwrap_or_default()
    }
}

/// A scene's draws over the background, compared with the last's.
struct Changed<'s> {
    moving: &'s [Draw],
    rects: &'s [[i32; 4]],
    view: Option<ViewPlace>,
    cycles: &'s [Cycle],
}

impl<'s> Changed<'s> {
    /// Draws what changed since `last` onto `screen`: the clip's pixels moved with the view, the
    /// cells whose draws differ restored from `background` and drawn again; each rectangle
    /// changed pushed to `damage`.
    fn draw(
        &self,
        screen: &mut Screen,
        background: &Background,
        last: &Last,
        pen: &mut Pen<'s, '_>,
        [now, was]: [&mut Buckets; 2],
        damage: &mut Vec<[i32; 4]>,
    ) -> Result<(), ComposeError> {
        let all = whole(screen);
        // The view's move since the last scene, if its pixels may move (the same clip and tile,
        // a background of one colour across the clip, a move smaller than the clip).
        let moved = match (last.view, self.view) {
            (Some(a), Some(b))
                if a.clip == b.clip && a.tile == b.tile && background.uniform == Some(b.clip) =>
            {
                let s = [
                    b.origin[0].saturating_sub(a.origin[0]),
                    b.origin[1].saturating_sub(a.origin[1]),
                ];
                let [l, t, r, bottom] = b.clip;
                let small = s[0].unsigned_abs() < r.saturating_sub(l).unsigned_abs()
                    && s[1].unsigned_abs() < bottom.saturating_sub(t).unsigned_abs();
                (s != [0, 0] && small).then_some((b.clip, s))
            }
            _ => None,
        };
        let shift = moved.map_or([0, 0], |(_, s)| s);
        // What the move keeps: the clip's pixels whose old place lay in the clip (none when
        // none lies on the screen).
        let kept = moved.map(|(clip, s)| {
            let to = move_pixels(screen, clip, s);
            damage.extend(to);
            to.unwrap_or_default()
        });
        let changed = changed_defs(&last.cycles, self.cycles);
        // The cells: every draw's rectangle now, the last's where they were and moved, the clip
        // when it moved.
        let rects = self
            .rects
            .iter()
            .copied()
            .chain(last.rects.iter().copied())
            .chain(last.rects.iter().map(|&r| moved_by(r, shift)))
            .chain(moved.map(|(clip, _)| clip));
        let Some(extent) = rects
            .fold(None, |r, d| union(r, Some(d)))
            .and_then(|r| meet(r, all))
        else {
            return Ok(());
        };
        let (align, tile) = self
            .view
            .map_or(([0, 0], DEFAULT_TILE), |v| (v.origin, v.tile.max(1)));
        let grid = Grid::new(extent, align, tile);
        grid.bucket(self.rects.iter().copied(), now);
        grid.bucket(last.rects.iter().map(|&r| moved_by(r, shift)), was);
        let before = last.moving();
        let same = |cell: usize| {
            let (a, b) = (now.of(cell), was.of(cell));
            a.len() == b.len()
                && a.iter().zip(b).all(|(&i, &j)| {
                    let (d, e) = (&self.moving[to_index(i)], &before[to_index(j)]);
                    same_moved(d, e, shift) && !uses(d, &changed)
                })
        };
        let dirty: Vec<bool> = (0..grid.cells())
            .map(|cell| {
                let rect = grid.rect(cell, extent);
                let stays = kept.is_none_or(|k| contains(k, rect));
                !(stays && same(cell))
            })
            .collect();
        let mut indices = Vec::new();
        for (rect, cells) in grid.merged(&dirty, extent) {
            screen.copy_rect(&background.screen, rect.map(i64::from));
            damage.push(rect);
            indices.clear();
            for cell in cells {
                indices.extend_from_slice(now.of(cell));
            }
            indices.sort_unstable();
            indices.dedup();
            for &i in &indices {
                let i = to_index(i);
                if meet(self.rects[i], rect).is_some() {
                    pen.draw(screen, &self.moving[i], Some(rect))?;
                }
            }
        }
        Ok(())
    }
}

/// A cell's side for a scene without a view's place.
const DEFAULT_TILE: i32 = 32;

/// The cells of a rectangle of the screen: squares of a tile's side, their corners at `align`
/// plus whole tiles.
#[derive(Debug, Clone, Copy)]
struct Grid {
    left: i64,
    top: i64,
    tile: i64,
    columns: usize,
    rows: usize,
}

impl Grid {
    /// The cells covering `extent`.
    fn new(extent: [i32; 4], align: [i32; 2], tile: i32) -> Self {
        let tile = i64::from(tile);
        let [l, t, r, b] = extent.map(i64::from);
        let left = l - (l - i64::from(align[0])).rem_euclid(tile);
        let top = t - (t - i64::from(align[1])).rem_euclid(tile);
        let count =
            |from: i64, to: i64| usize::try_from((to - from + tile - 1) / tile).unwrap_or(0);
        Self {
            left,
            top,
            tile,
            columns: count(left, r),
            rows: count(top, b),
        }
    }

    fn cells(&self) -> usize {
        self.columns * self.rows
    }

    /// The columns and rows of the cells `rect` meets (none for an empty one).
    fn span(&self, [l, t, r, b]: [i32; 4]) -> Option<([usize; 2], [usize; 2])> {
        let (l, t, r, b) = (i64::from(l), i64::from(t), i64::from(r), i64::from(b));
        if l >= r || t >= b {
            return None;
        }
        let cut = |v: i64, n: usize| usize::try_from(v.max(0)).unwrap_or(0).min(n);
        let columns = [
            cut((l - self.left).div_euclid(self.tile), self.columns),
            cut((r - 1 - self.left).div_euclid(self.tile) + 1, self.columns),
        ];
        let rows = [
            cut((t - self.top).div_euclid(self.tile), self.rows),
            cut((b - 1 - self.top).div_euclid(self.tile) + 1, self.rows),
        ];
        (columns[0] < columns[1] && rows[0] < rows[1]).then_some((columns, rows))
    }

    /// Cell `cell`'s rectangle, cut to `extent`.
    fn rect(&self, cell: usize, extent: [i32; 4]) -> [i32; 4] {
        let (c, r) = (cell % self.columns.max(1), cell / self.columns.max(1));
        self.cut([c, r], [c + 1, r + 1], extent)
    }

    /// The rectangle of columns `from[0]..to[0]` and rows `from[1]..to[1]`, cut to `extent`.
    fn cut(&self, from: [usize; 2], to: [usize; 2], extent: [i32; 4]) -> [i32; 4] {
        let at = |n: usize, start: i64| {
            i32::try_from(start + self.tile * i64::try_from(n).unwrap_or(0)).unwrap_or(i32::MAX)
        };
        let rect = [
            at(from[0], self.left),
            at(from[1], self.top),
            at(to[0], self.left),
            at(to[1], self.top),
        ];
        meet(rect, extent).unwrap_or_default()
    }

    /// Each rectangle's index listed in every cell it meets, in order.
    fn bucket(&self, rects: impl Iterator<Item = [i32; 4]>, out: &mut Buckets) {
        out.spans.clear();
        out.spans.extend(rects.map(|r| self.span(r)));
        out.starts.clear();
        out.starts.resize(self.cells() + 1, 0);
        for &([c0, c1], [r0, r1]) in out.spans.iter().flatten() {
            for r in r0..r1 {
                for count in &mut out.starts[r * self.columns + c0 + 1..r * self.columns + c1 + 1] {
                    *count += 1;
                }
            }
        }
        for i in 1..out.starts.len() {
            out.starts[i] += out.starts[i - 1];
        }
        out.items.clear();
        out.items
            .resize(to_index(*out.starts.last().unwrap_or(&0)), 0);
        out.next.clear();
        out.next.extend_from_slice(&out.starts);
        for (i, span) in out.spans.iter().enumerate() {
            let Some(([c0, c1], [r0, r1])) = *span else {
                continue;
            };
            let i = u32::try_from(i).unwrap_or(u32::MAX);
            for r in r0..r1 {
                for at in &mut out.next[r * self.columns + c0..r * self.columns + c1] {
                    out.items[to_index(*at)] = i;
                    *at += 1;
                }
            }
        }
    }

    /// The dirty cells as rectangles (each row's runs, a run below one of the same columns
    /// joined to it), cut to `extent`, with their cells.
    fn merged(&self, dirty: &[bool], extent: [i32; 4]) -> Vec<([i32; 4], Vec<usize>)> {
        // Each rectangle as its columns, first row and rows; those the last row reached.
        let mut runs: Vec<([usize; 2], usize, usize)> = Vec::new();
        let (mut above, mut reached) = (Vec::new(), Vec::new());
        for r in 0..self.rows {
            let row = dirty
                .get(r * self.columns..(r + 1) * self.columns)
                .unwrap_or_default();
            reached.clear();
            let mut c = 0;
            while c < row.len() {
                if !row[c] {
                    c += 1;
                    continue;
                }
                let from = c;
                while c < row.len() && row[c] {
                    c += 1;
                }
                let columns = [from, c];
                if let Some(&k) = above.iter().find(|&&k: &&usize| runs[k].0 == columns) {
                    runs[k].2 += 1;
                    reached.push(k);
                } else {
                    reached.push(runs.len());
                    runs.push((columns, r, 1));
                }
            }
            core::mem::swap(&mut above, &mut reached);
        }
        runs.into_iter()
            .map(|([c0, c1], r0, n)| {
                let rect = self.cut([c0, r0], [c1, r0 + n], extent);
                let cells = (r0..r0 + n)
                    .flat_map(|r| (c0..c1).map(move |c| r * self.columns + c))
                    .collect();
                (rect, cells)
            })
            .collect()
    }
}

/// Each cell's list of draws: cell `c`'s from `starts[c]` to `starts[c + 1]` of `items`.
#[derive(Debug, Clone, Default)]
struct Buckets {
    starts: Vec<u32>,
    items: Vec<u32>,
    /// Each rectangle's columns and rows of cells; the next free place of each cell's list.
    spans: Vec<Option<([usize; 2], [usize; 2])>>,
    next: Vec<u32>,
}

impl Buckets {
    fn of(&self, cell: usize) -> &[u32] {
        let at = |i: usize| to_index(self.starts.get(i).copied().unwrap_or(0));
        self.items.get(at(cell)..at(cell + 1)).unwrap_or_default()
    }
}

fn to_index(value: u32) -> usize {
    usize::try_from(value).unwrap_or(usize::MAX)
}

/// Whether `now` draws what `was` drew moved by `shift`.
fn same_moved(now: &Draw, was: &Draw, shift: [i32; 2]) -> bool {
    let at = |x: i32, from: i32, by: i32| x == from.saturating_add(by);
    match (now, was) {
        (
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
            },
            Draw::Piece {
                pass: pass_was,
                def: def_was,
                group: group_was,
                frame: frame_was,
                x: x_was,
                y: y_was,
                mirror_x: mirror_x_was,
                mirror_y: mirror_y_was,
                clip: clip_was,
                flag: flag_was,
            },
        ) => {
            pass == pass_was
                && group == group_was
                && frame == frame_was
                && mirror_x == mirror_x_was
                && mirror_y == mirror_y_was
                && flag == flag_was
                && at(*x, *x_was, shift[0])
                && at(*y, *y_was, shift[1])
                && *clip == moved_by(*clip_was, shift)
                && def == def_was
        }
        (
            Draw::Gem { def, frame, x, y },
            Draw::Gem {
                def: def_was,
                frame: frame_was,
                x: x_was,
                y: y_was,
            },
        ) => {
            frame == frame_was
                && at(*x, *x_was, shift[0])
                && at(*y, *y_was, shift[1])
                && def == def_was
        }
        _ => false,
    }
}

/// Whether `draw`'s DEF is one of `defs` (ignoring case).
fn uses(draw: &Draw, defs: &[String]) -> bool {
    let def = match draw {
        Draw::Piece { def, .. } | Draw::Gem { def, .. } => def,
        Draw::Frame { .. } | Draw::Dialog(_) => return true,
    };
    defs.iter().any(|d| d.eq_ignore_ascii_case(def))
}

/// The DEFs whose palettes cycle otherwise in `now` than in `was` (their ranges or steps).
fn changed_defs(was: &[Cycle], now: &[Cycle]) -> Vec<String> {
    let of = |list: &'_ [Cycle], def: &str| -> Vec<(Vec<[u8; 2]>, u32)> {
        list.iter()
            .filter(|c| c.def.eq_ignore_ascii_case(def))
            .map(|c| (c.ranges.clone(), c.steps))
            .collect()
    };
    let mut changed: Vec<String> = Vec::new();
    for cycle in was.iter().chain(now) {
        let def = &cycle.def;
        if !changed.iter().any(|d| d.eq_ignore_ascii_case(def)) && of(was, def) != of(now, def) {
            changed.push(def.clone());
        }
    }
    changed
}

/// Moves the screen's pixels inside `clip` by `shift`: those whose new place lies in the clip
/// and whose old one did too; the rectangle they now fill, if any.
fn move_pixels(screen: &mut Screen, clip: [i32; 4], [sx, sy]: [i32; 2]) -> Option<[i32; 4]> {
    let clip = meet(clip, whole(screen))?;
    let to = meet(clip, moved_by(clip, [sx, sy]))?;
    let width = i64::from(screen.width());
    let [l, t, r, b] = to.map(i64::from);
    let n = usize::try_from(r - l).unwrap_or(0);
    let (sx, sy) = (i64::from(sx), i64::from(sy));
    let pixels = screen.pixels_mut();
    let mut copy = |y: i64| {
        let from = usize::try_from((y - sy) * width + l - sx).unwrap_or(0);
        let at = usize::try_from(y * width + l).unwrap_or(0);
        pixels.copy_within(from..from + n, at);
    };
    // Rows moving down are copied from the bottom up, so that none is overwritten first.
    if sy > 0 {
        (t..b).rev().for_each(&mut copy);
    } else {
        (t..b).for_each(&mut copy);
    }
    Some(to)
}

/// Whether the screen is one colour across `clip` (cut to it).
fn one_colour(screen: &Screen, clip: [i32; 4]) -> bool {
    let Some([l, t, r, b]) = meet(clip, whole(screen)) else {
        return true;
    };
    let width = usize::try_from(screen.width()).unwrap_or(0);
    let at =
        |x: i32, y: i32| usize::try_from(y).unwrap_or(0) * width + usize::try_from(x).unwrap_or(0);
    let pixels = screen.pixels();
    let first = pixels.get(at(l, t)).copied();
    (t..b).all(|y| {
        pixels
            .get(at(l, y)..at(r, y))
            .unwrap_or_default()
            .iter()
            .all(|&p| Some(p) == first)
    })
}

/// The rectangle `draw` can change: a piece's clip, a gem's frame's canvas, `all` (the screen)
/// for a frame's picture or a dialog's item.
///
/// # Errors
/// A gem whose DEF or frame the batch lacks.
fn rect_of(draw: &Draw, batch: &Batch, all: [i32; 4]) -> Result<[i32; 4], ComposeError> {
    Ok(match draw {
        Draw::Piece { clip, .. } => *clip,
        Draw::Gem { def, frame, x, y } => {
            let ((f, _), _) = gem_frame(batch, def, *frame)?;
            let side = |v: u32| i32::try_from(v).unwrap_or(i32::MAX);
            [
                *x,
                *y,
                x.saturating_add(side(f.full_width)),
                y.saturating_add(side(f.full_height)),
            ]
        }
        Draw::Frame { .. } | Draw::Dialog(_) => all,
    })
}

/// The whole screen as a rectangle.
fn whole(screen: &Screen) -> [i32; 4] {
    let side = |v: u32| i32::try_from(v).unwrap_or(i32::MAX);
    [0, 0, side(screen.width()), side(screen.height())]
}

/// The smallest rectangle holding both.
fn union(a: Option<[i32; 4]>, b: Option<[i32; 4]>) -> Option<[i32; 4]> {
    match (a, b) {
        (Some(a), Some(b)) => Some([
            a[0].min(b[0]),
            a[1].min(b[1]),
            a[2].max(b[2]),
            a[3].max(b[3]),
        ]),
        (a, b) => a.or(b),
    }
}

/// Where two rectangles meet, if they do.
fn meet(a: [i32; 4], b: [i32; 4]) -> Option<[i32; 4]> {
    let r = [
        a[0].max(b[0]),
        a[1].max(b[1]),
        a[2].min(b[2]),
        a[3].min(b[3]),
    ];
    (r[0] < r[2] && r[1] < r[3]).then_some(r)
}

/// Whether `outer` holds all of `inner`.
fn contains(outer: [i32; 4], inner: [i32; 4]) -> bool {
    outer[0] <= inner[0] && outer[1] <= inner[1] && inner[2] <= outer[2] && inner[3] <= outer[3]
}

/// `rect` moved by `shift`.
fn moved_by(rect: [i32; 4], [sx, sy]: [i32; 2]) -> [i32; 4] {
    [
        rect[0].saturating_add(sx),
        rect[1].saturating_add(sy),
        rect[2].saturating_add(sx),
        rect[3].saturating_add(sy),
    ]
}

/// What drawing a scene looks up once (its DEFs' frames and palettes, the flag palette's
/// colours), each DEF's palette cycled from the kept ones (E24 M2 T4b3, M3 T2b1).
struct Pen<'s, 'k> {
    scene: &'s Scene,
    batch: &'s Batch,
    kept: &'k mut BTreeMap<String, (Vec<u32>, ScreenPalette)>,
    defs: BTreeMap<&'s str, (Drawable<'s>, ScreenPalette)>,
    flag_colours: Option<ScreenPalette>,
}

impl<'s, 'k> Pen<'s, 'k> {
    fn new(
        scene: &'s Scene,
        batch: &'s Batch,
        kept: &'k mut BTreeMap<String, (Vec<u32>, ScreenPalette)>,
    ) -> Self {
        Self {
            scene,
            batch,
            kept,
            defs: BTreeMap::new(),
            flag_colours: None,
        }
    }

    /// Draws `item` onto `screen`, cut to `cut` if one is given (a piece and a gem; the frame's
    /// picture and the dialog's items are drawn whole).
    fn draw(
        &mut self,
        screen: &mut Screen,
        item: &'s Draw,
        cut: Option<[i32; 4]>,
    ) -> Result<(), ComposeError> {
        let (batch, scene) = (self.batch, self.scene);
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
                frame_picture(screen, image, players, *viewer);
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
                let Some(clip) = cut.map_or(Some(*clip), |c| meet(*clip, c)) else {
                    return Ok(());
                };
                let Self {
                    kept,
                    defs,
                    flag_colours,
                    ..
                } = self;
                let (f, palette) = frame_in(defs, kept, batch, scene, def, *group, *frame)?;
                let at = Place {
                    x: *x,
                    y: *y,
                    mirror_x: *mirror_x,
                    mirror_y: *mirror_y,
                };
                match pass {
                    Pass::Flat | Pass::Body => {
                        let colour = flag_colour(batch, scene, flag_colours, *flag)?;
                        draw::flagged(screen, f, palette, at, clip, colour);
                    }
                    Pass::Shadow => draw::shadow_pass(screen, f, at, clip),
                    Pass::PathShadow => draw::shadows(screen, f, at, clip),
                    Pass::Blend => draw::blend(screen, f, palette, at, clip),
                    _ => draw::raw(screen, f, palette, at, clip),
                }
                if *pass == Pass::Shroud {
                    draw::shadows(screen, f, at, clip);
                }
            }
            Draw::Gem { def, frame, x, y } => {
                // The plain drawer with runs skipped, unmirrored, on the screen (0x40F2F0): the
                // raw drawer's literal pixels (design §1.1), in the DEF's own palette.
                let all = whole(screen);
                let Some(clip) = cut.map_or(Some(all), |c| meet(all, c)) else {
                    return Ok(());
                };
                let (f, sprite) = gem_frame(batch, def, *frame)?;
                let palette = ScreenPalette::new(&sprite.def.palette, FORMAT);
                let at = Place {
                    x: *x,
                    y: *y,
                    ..Place::default()
                };
                draw::raw(screen, f, &palette, at, clip);
            }
            Draw::Dialog(draw) => {
                // The dialog's items by their own drawers (E24 M3 T2a, design §4.1).
                h3_ui::paint::draw(screen, draw, batch, scene.bugs)
                    .map_err(ComposeError::Missing)?;
            }
        }
        Ok(())
    }
}

/// `def`'s frame with its spans and the DEF's palette cycled, looked up once a scene by the name as
/// the scene spells it (in `defs`).
fn frame_in<'s, 'd>(
    defs: &'d mut BTreeMap<&'s str, (Drawable<'s>, ScreenPalette)>,
    kept: &mut BTreeMap<String, (Vec<u32>, ScreenPalette)>,
    batch: &'s Batch,
    scene: &Scene,
    def: &'s str,
    group: u32,
    index: u32,
) -> Result<(FrameSpans<'s>, &'d ScreenPalette), ComposeError> {
    let (drawable, palette) = match defs.entry(def) {
        Entry::Occupied(known) => known.into_mut(),
        Entry::Vacant(new) => {
            let drawable = batch
                .drawable(def)
                .ok_or_else(|| ComposeError::Missing(def.into()))?;
            let palette = cycled_kept(kept, drawable.sprite, def, &scene.cycles);
            new.insert((drawable, palette))
        }
    };
    let f = drawable
        .frame(group, index)
        .ok_or_else(|| ComposeError::Missing(format!("{def} group {group} frame {index}")))?;
    Ok((f, palette))
}

/// A gem's frame (group 0) with its spans, and its DEF.
fn gem_frame<'b>(
    batch: &'b Batch,
    def: &str,
    index: u32,
) -> Result<(FrameSpans<'b>, &'b Sprite), ComposeError> {
    let drawable = batch
        .drawable(def)
        .ok_or_else(|| ComposeError::Missing(def.into()))?;
    let f = drawable
        .frame(0, index)
        .ok_or_else(|| ComposeError::Missing(format!("{def} group 0 frame {index}")))?;
    Ok((f, drawable.sprite))
}

/// [`cycled`], kept by the DEF's name for the steps of its cycles: made again only when they
/// change (E24 M3 T2b1).
fn cycled_kept(
    kept: &mut BTreeMap<String, (Vec<u32>, ScreenPalette)>,
    sprite: &Sprite,
    def: &str,
    cycles: &[Cycle],
) -> ScreenPalette {
    let steps = || {
        cycles
            .iter()
            .filter(|c| c.def.eq_ignore_ascii_case(def))
            .map(|c| c.steps)
    };
    if let Some((was, palette)) = kept.get(def)
        && was.iter().copied().eq(steps())
    {
        return palette.clone();
    }
    let palette = cycled(sprite, def, cycles);
    kept.insert(def.into(), (steps().collect(), palette.clone()));
    palette
}

/// The screen colour of entry `flag` of the scene's flag palette, `None` without an entry or for a
/// colour of 0, which the flagged drawer takes as none (0x47D0A0; design §2.9); the palette's
/// colours worked out once a scene, in `colours`.
fn flag_colour(
    batch: &Batch,
    scene: &Scene,
    colours: &mut Option<ScreenPalette>,
    flag: Option<u8>,
) -> Result<Option<u16>, ComposeError> {
    let Some(entry) = flag else {
        return Ok(None);
    };
    let palette = match colours.take() {
        Some(palette) => palette,
        None => {
            let flags = &scene.flags;
            let palette = batch
                .palette(flags)
                .ok_or_else(|| ComposeError::Missing(flags.clone()))?;
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

#[cfg(test)]
mod tests;
