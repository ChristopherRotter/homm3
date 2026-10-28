//! The adventure screen's layout as data (E24 M1 T2; ADR-0020 point 3: the frame size and every
//! layout are data, never literals in code): `clients/map/layouts/adventure.toml` at the native
//! resolution (`docs/epics/E24-design.md` §1.1, §1.2, §1.9). An HD layout is another file of the
//! same format: the view's place, its clip and its loops are the immediates an HD layout
//! changes (§1.2). The panels come with M3: the dialog and the resource bar since T2a (§4.1, in
//! `h3_ui::ui`'s format), the minimap, the lists' contents and the status window later. The format
//! is *provisional* until the M3 review.

use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use h3_ui::ui::{Dialog, Item};
use serde::Deserialize;

/// The adventure screen's layout.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdventureLayout {
    /// The frame's width and height (800 × 600).
    pub frame: [u32; 2],
    /// The frame's picture (`AdvMap.pcx`), drawn opaque at (0, 0) before the view, its palette's
    /// entries 224–255 the viewing player's colours (adventure_full_update 0x417420, §1.9).
    pub picture: String,
    /// The palette of the players' colours (`PLAYERS.PAL`): the picture takes the viewing
    /// player's 32 entries from 32 × its colour (0x5FFE20).
    pub player_palette: String,
    /// The map's view.
    pub view: View,
    /// The view's scroll (E24 M2 T2, design §3.4).
    pub scroll: Scroll,
    /// The adventure cursor (E24 M2 T2, design §3.4).
    pub cursor: CursorArt,
    /// The adventure screen's dialog (E24 M3 T2a, design §4.1: `adventure_dialog_create`
    /// 0x401510), its items in the executable's order.
    pub dialog: Dialog,
    /// The resource bar's panel (`resource_bar_create` 0x558BA0).
    pub resource_bar: Dialog,
    /// What the panels' logic needs of the dialog (E24 M3 T2a).
    pub panels: Panels,
}

/// What the panels' logic needs of the dialog's items (E24 M3 T2a, design §4.1).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Panels {
    /// The items drawn in the viewer's colours (0x403F60: the buttons 3 to 12; the resource
    /// bar's picture, 0x558C99, is the resource bar's own).
    pub coloured: Vec<u16>,
    /// The underground button's DEF on the surface and underground (0x65F22C).
    pub underground: [String; 2],
    /// Sleep's DEF while the hero is awake and asleep (0x65F238).
    pub sleep: [String; 2],
}

/// The view's scroll (`view_scroll` 0x4195C0, `edge_scroll` 0x419820; design §3.4).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scroll {
    /// The view's tile the map's first and last column and row stop at (9, 8, the view's middle
    /// of 19 by 17): the view's corner stays within −9 to the map's width less 10 and −8 to its
    /// height less 9.
    pub middle: [i32; 2],
    /// The edge zones: the view scrolls while the pointer lies within this many pixels of a side
    /// of the frame (16, MCL-Q6's answer; the original's are 16, 14, 16 and 15 pixels wide,
    /// reproduced under `Bugs::Original` only).
    pub edge: i32,
}

/// The adventure cursor (the mouse manager's type 1: `cursor_defs` 0x67FF38; design §3.4).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CursorArt {
    /// Its DEF (`cradvntr.def`).
    pub def: String,
    /// The pointer's frame (0).
    pub arrow: u32,
    /// The first edge cursor's frame (32): the scroll's eight directions from north clockwise.
    pub edges: u32,
    /// Its frames' hot spots (`cursor_hot_spots` 0x67FF50 + (144 + frame) × 8): the canvas's
    /// corner is the mouse less its frame's.
    pub hot: Vec<[i32; 2]>,
}

/// The scroll's directions, one edge cursor each.
pub const DIRECTIONS: u32 = 8;

/// The map's view (§1.1, §1.2).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct View {
    /// Where the view's tile (vx, vy) has its top-left corner (0, 8): the tile (vx + i, vy + j)
    /// has it at this plus (tile i, tile j).
    pub origin: [i32; 2],
    /// A tile's side (32).
    pub tile: i32,
    /// The columns i the passes draw, first and last (−1, 19).
    pub columns: [i32; 2],
    /// The rows j the passes draw, first and last (−1, 17).
    pub rows: [i32; 2],
    /// Every pass clips its pieces to this rectangle: left, top, right, bottom, the last two
    /// exclusive (8, 8, 600, 552).
    pub clip: [i32; 4],
    /// The corner gems, over everything else (0x40F2F0): group 0's frame of the viewing player,
    /// runs skipped, not mirrored.
    pub gems: Vec<Gem>,
}

/// One corner gem.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Gem {
    /// Its DEF.
    pub def: String,
    /// Its canvas's top-left corner.
    pub at: [i32; 2],
}

/// The frames of a gem's DEF the view draws: one per player colour.
pub const GEM_FRAMES: usize = 8;

impl View {
    /// The top-left corner of the view's tile (vx + i, vy + j).
    #[must_use]
    pub fn place(&self, i: i32, j: i32) -> [i32; 2] {
        [
            self.origin[0].saturating_add(self.tile.saturating_mul(i)),
            self.origin[1].saturating_add(self.tile.saturating_mul(j)),
        ]
    }
}

impl AdventureLayout {
    /// Parses and checks a layout: the names with their extensions, the loops and the clip in
    /// order, the clip inside the frame, a tile of at least one pixel.
    ///
    /// # Errors
    /// The parser's message, or the rule the layout breaks.
    pub fn parse(text: &str) -> Result<Self, String> {
        let mut layout: Self = toml::from_str(text).map_err(|e| e.to_string())?;
        layout.dialog = layout.dialog.expanded();
        layout.resource_bar = layout.resource_bar.expanded();
        layout.check()?;
        Ok(layout)
    }

    fn check(&self) -> Result<(), String> {
        name(&self.picture, ".PCX")?;
        name(&self.player_palette, ".PAL")?;
        for gem in &self.view.gems {
            name(&gem.def, ".DEF")?;
        }
        let view = &self.view;
        let [left, top, right, bottom] = view.clip;
        let [width, height] = self.frame.map(i64::from);
        let inside = 0 <= left
            && left < right
            && i64::from(right) <= width
            && 0 <= top
            && top < bottom
            && i64::from(bottom) <= height;
        if !inside {
            return Err(format!(
                "the view's clip {:?} is not inside the frame",
                view.clip
            ));
        }
        if view.columns[0] > view.columns[1] || view.rows[0] > view.rows[1] {
            return Err("the view's columns or rows run backwards".to_string());
        }
        if view.tile < 1 {
            return Err(format!("a tile of {} pixels", view.tile));
        }
        name(&self.cursor.def, ".DEF")?;
        let frames = u32::try_from(self.cursor.hot.len()).unwrap_or(u32::MAX);
        let edges = self.cursor.edges.saturating_add(DIRECTIONS);
        if self.cursor.arrow >= frames || edges > frames {
            return Err(format!(
                "the cursor's hot spots end at frame {frames}: the arrow {} or the edges {} to {} \
                 lie past them",
                self.cursor.arrow,
                self.cursor.edges,
                edges.saturating_sub(1)
            ));
        }
        let edge = i64::from(self.scroll.edge);
        if edge < 1 || 2 * edge > width.min(height) {
            return Err(format!(
                "edge zones of {} pixels in a frame of {:?}",
                self.scroll.edge, self.frame
            ));
        }
        for (named, ext) in self.dialog_names() {
            name(named, ext)?;
        }
        Ok(())
    }

    /// Every name the dialog, the resource bar and the panels' data mention, with the extension
    /// each must have.
    fn dialog_names(&self) -> Vec<(&str, &'static str)> {
        let items = self.dialog.items.iter().chain(&self.resource_bar.items);
        let mut out = Vec::new();
        for item in items {
            match item {
                Item::Picture(p) => out.push((p.picture.as_str(), ".PCX")),
                Item::Button(b) => out.push((b.def.as_str(), ".DEF")),
                Item::Text(t) => {
                    out.extend([(t.picture.as_str(), ".PCX"), (t.font.as_str(), ".FNT")]);
                    out.push((t.font_palette.as_str(), ".PAL"));
                }
                Item::Def(d) => out.extend(d.def.as_deref().map(|d| (d, ".DEF"))),
                Item::Label(l) => {
                    out.extend([(l.font.as_str(), ".FNT"), (l.font_palette.as_str(), ".PAL")]);
                }
                Item::Area(_) => {}
            }
        }
        let panels = &self.panels;
        out.extend(
            panels
                .underground
                .iter()
                .chain(&panels.sleep)
                .map(|d| (d.as_str(), ".DEF")),
        );
        out
    }

    /// Every archive entry name the layout mentions, upper-cased, each once.
    #[must_use]
    pub fn names(&self) -> BTreeSet<String> {
        core::iter::once(self.picture.as_str())
            .chain([self.player_palette.as_str(), self.cursor.def.as_str()])
            .chain(self.view.gems.iter().map(|g| g.def.as_str()))
            .chain(self.dialog_names().into_iter().map(|(n, _)| n))
            .map(str::to_ascii_uppercase)
            .collect()
    }
}

/// A name with its extension (`ext` upper case, with the dot) and something before it.
fn name(name: &str, ext: &str) -> Result<(), String> {
    let upper = name.to_ascii_uppercase();
    if upper.len() <= ext.len() || !upper.ends_with(ext) {
        return Err(format!("`{name}` is not a {ext} name"));
    }
    Ok(())
}

#[cfg(test)]
mod tests;
