//! A DEF with every frame decoded (E17 M1, `h3-view`'s checkpoint batch; moved here at E24 M3 T1a
//! with the cursor that draws one): what a viewer's batch holds of a DEF.

use alloc::collections::BTreeMap;

use h3_assets::def::{Def, Frame};

/// A DEF with every frame decoded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sprite {
    /// The DEF's header, palette and groups.
    pub def: Def,
    /// Its frames, by their offset in the DEF.
    pub frames: BTreeMap<u32, Frame>,
}

impl Sprite {
    /// Frame `index` of the group of id `group`.
    #[must_use]
    pub fn frame(&self, group: u32, index: u32) -> Option<&Frame> {
        let group = self.def.groups.iter().find(|g| g.id == group)?;
        let at = group
            .frames
            .get(usize::try_from(index).unwrap_or(usize::MAX))?;
        self.frames.get(&at.offset)
    }
}
