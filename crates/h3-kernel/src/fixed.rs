//! Fixed-capacity, inline storage for hot-path state (02-architecture §4: "fixed-capacity
//! structures in battle; no allocation in the battle hot path"). Cloning a `FixedVec` is a plain
//! memory copy; it never allocates.

use core::fmt;

use crate::hash::{CanonicalHash, StateHasher};

/// Error returned when a [`FixedVec`] is full.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CapacityError {
    /// The capacity that was exceeded.
    pub capacity: usize,
}

impl fmt::Display for CapacityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "capacity of {} exceeded", self.capacity)
    }
}

/// A vector with inline storage for at most `N` elements.
#[derive(Clone, Copy)]
pub struct FixedVec<T: Copy + Default, const N: usize> {
    items: [T; N],
    len: usize,
}

impl<T: Copy + Default, const N: usize> Default for FixedVec<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Copy + Default, const N: usize> FixedVec<T, N> {
    /// An empty vector.
    pub fn new() -> Self {
        Self {
            items: [T::default(); N],
            len: 0,
        }
    }

    /// A vector made of the first `len` elements of `items` (`len` is clamped to `N`). Lets
    /// callers that know their bound build a vector without a fallible `push`.
    pub fn from_array_prefix(items: [T; N], len: usize) -> Self {
        Self {
            items,
            len: len.min(N),
        }
    }

    /// A vector holding a copy of `values`.
    pub fn from_slice(values: &[T]) -> Result<Self, CapacityError> {
        let mut v = Self::new();
        for &x in values {
            v.push(x)?;
        }
        Ok(v)
    }

    /// Appends an element.
    pub fn push(&mut self, value: T) -> Result<(), CapacityError> {
        match self.items.get_mut(self.len) {
            Some(slot) => {
                *slot = value;
                self.len += 1;
                Ok(())
            }
            None => Err(CapacityError { capacity: N }),
        }
    }

    /// Removes all elements.
    pub fn clear(&mut self) {
        self.len = 0;
    }

    /// Number of elements.
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Whether there are no elements.
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Maximum number of elements.
    pub const fn capacity(&self) -> usize {
        N
    }

    /// The elements as a slice.
    pub fn as_slice(&self) -> &[T] {
        self.items.get(..self.len).unwrap_or(&[])
    }

    /// The elements as a mutable slice.
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        self.items.get_mut(..self.len).unwrap_or(&mut [])
    }

    /// The element at `index`, if any.
    pub fn get(&self, index: usize) -> Option<&T> {
        self.as_slice().get(index)
    }

    /// The element at `index`, mutably, if any.
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.as_mut_slice().get_mut(index)
    }

    /// Iterates over the elements.
    pub fn iter(&self) -> core::slice::Iter<'_, T> {
        self.as_slice().iter()
    }
}

impl<T: Copy + Default + PartialEq, const N: usize> PartialEq for FixedVec<T, N> {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl<T: Copy + Default + Eq, const N: usize> Eq for FixedVec<T, N> {}

impl<T: Copy + Default + fmt::Debug, const N: usize> fmt::Debug for FixedVec<T, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.as_slice()).finish()
    }
}

impl<T: Copy + Default + CanonicalHash, const N: usize> CanonicalHash for FixedVec<T, N> {
    fn hash_into(&self, h: &mut StateHasher) {
        self.as_slice().hash_into(h);
    }
}

impl<'a, T: Copy + Default, const N: usize> IntoIterator for &'a FixedVec<T, N> {
    type Item = &'a T;
    type IntoIter = core::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use crate::hash::hash_of;
    use std::format;
    use std::string::ToString;

    #[test]
    fn push_until_full() {
        let mut v: FixedVec<u8, 2> = FixedVec::default();
        assert!(v.is_empty());
        assert_eq!(v.capacity(), 2);
        v.push(1).unwrap();
        v.push(2).unwrap();
        assert_eq!(v.push(3), Err(CapacityError { capacity: 2 }));
        assert_eq!(
            CapacityError { capacity: 2 }.to_string(),
            "capacity of 2 exceeded"
        );
        assert_eq!(v.len(), 2);
        assert_eq!(v.as_slice(), &[1, 2]);
        assert_eq!(v.get(1), Some(&2));
        assert_eq!(v.get(2), None);
        *v.get_mut(0).unwrap() = 9;
        v.as_mut_slice()[1] = 8;
        assert_eq!(v.iter().copied().collect::<std::vec::Vec<_>>(), [9, 8]);
        assert_eq!((&v).into_iter().count(), 2);
        v.clear();
        assert!(v.is_empty());
        assert_eq!(v.get_mut(0), None);
    }

    #[test]
    fn equality_debug_and_hash_ignore_unused_slots() {
        let mut a: FixedVec<u8, 4> = FixedVec::from_slice(&[1, 2, 3]).unwrap();
        a.clear();
        a.push(7).unwrap();
        let b: FixedVec<u8, 4> = FixedVec::from_slice(&[7]).unwrap();
        assert_eq!(a, b);
        assert_eq!(format!("{a:?}"), "[7]");
        assert_eq!(hash_of(&a), hash_of(&b));
        assert_eq!(
            FixedVec::<u8, 1>::from_slice(&[1, 2]),
            Err(CapacityError { capacity: 1 })
        );
        assert_eq!(
            FixedVec::from_array_prefix([1u8, 2, 3], 2).as_slice(),
            &[1, 2]
        );
        assert_eq!(FixedVec::from_array_prefix([1u8, 2, 3], 9).len(), 3);
    }
}
