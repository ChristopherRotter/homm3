//! Dense, typed handles into immutable tables (ADR-0006: "dense integer handles").
//!
//! A `Handle<T>` is a `u16` index tagged with a zero-sized marker type, so a creature handle can
//! never be used where a profile handle is expected. Handles are created by the component that
//! owns the table (e.g. the `Ruleset` compiler), which guarantees they are in range.

use core::cmp::Ordering;
use core::fmt;
use core::marker::PhantomData;

use crate::hash::{CanonicalHash, StateHasher};

/// A dense index into a table of `T`.
pub struct Handle<T> {
    raw: u16,
    marker: PhantomData<fn() -> T>,
}

impl<T> Handle<T> {
    /// Creates a handle from its raw index. Only table owners should call this.
    pub const fn from_raw(raw: u16) -> Self {
        Self {
            raw,
            marker: PhantomData,
        }
    }

    /// Creates a handle for a table position, if it fits in the handle's range.
    pub fn from_index(index: usize) -> Option<Self> {
        u16::try_from(index).ok().map(Self::from_raw)
    }

    /// The raw index.
    pub const fn raw(self) -> u16 {
        self.raw
    }

    /// The raw index as a `usize`, for table access.
    pub const fn index(self) -> usize {
        self.raw as usize
    }
}

// Manual impls: derives would require `T: Clone` etc., but `T` is only a marker.
impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Copy for Handle<T> {}
impl<T> Default for Handle<T> {
    /// Handle 0. Only meaningful as a placeholder in fixed-capacity storage.
    fn default() -> Self {
        Self::from_raw(0)
    }
}
impl<T> PartialEq for Handle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.raw == other.raw
    }
}
impl<T> Eq for Handle<T> {}
impl<T> PartialOrd for Handle<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl<T> Ord for Handle<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.raw.cmp(&other.raw)
    }
}
impl<T> fmt::Debug for Handle<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{}", self.raw)
    }
}
impl<T> CanonicalHash for Handle<T> {
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_u16(self.raw);
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use std::format;

    enum Tag {}

    #[test]
    fn handle_basics() {
        let a: Handle<Tag> = Handle::from_raw(3);
        let b = a;
        assert_eq!(a, b.clone());
        assert_eq!(Handle::<Tag>::default().raw(), 0);
        assert_eq!(a.raw(), 3);
        assert_eq!(a.index(), 3);
        assert!(Handle::<Tag>::from_raw(1) < a);
        assert_eq!(a.partial_cmp(&b), Some(Ordering::Equal));
        assert_eq!(format!("{a:?}"), "#3");
        assert_eq!(Handle::<Tag>::from_index(70_000), None);
        assert_eq!(Handle::<Tag>::from_index(7).map(Handle::raw), Some(7));
        let mut h = StateHasher::new();
        a.hash_into(&mut h);
        let mut g = StateHasher::new();
        g.write_u16(3);
        assert_eq!(h.finish(), g.finish());
    }
}
