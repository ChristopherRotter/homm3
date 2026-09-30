//! Total-order sorting (ADR-0003 §9). Logic crates sort only through this module, and every key
//! must end in a unique tie-breaker (typically an entity id), so equal elements can never be
//! ordered differently by different standard-library versions or platforms.

/// Error: two elements had equal keys, i.e. the caller's key is not unique (a bug).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DuplicateKey;

/// Sorts `items` ascending by `key`, which must be unique across `items`.
///
/// Returns [`DuplicateKey`] if two keys compare equal; the slice is then sorted but the relative
/// order of the equal elements is unspecified, so callers must treat this as an invariant
/// violation.
pub fn sort_by_unique_key<T, K: Ord>(
    items: &mut [T],
    mut key: impl FnMut(&T) -> K,
) -> Result<(), DuplicateKey> {
    items.sort_unstable_by_key(|x| key(x));
    if is_strictly_increasing_by(items, key) {
        Ok(())
    } else {
        Err(DuplicateKey)
    }
}

/// Whether `items` are strictly increasing by `key` (sorted and without duplicate keys).
pub fn is_strictly_increasing_by<T, K: Ord>(items: &[T], mut key: impl FnMut(&T) -> K) -> bool {
    items.is_sorted_by(|a, b| key(a) < key(b))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sorts_with_unique_keys() {
        let mut v = [(3, 'c'), (1, 'a'), (2, 'b')];
        assert_eq!(sort_by_unique_key(&mut v, |x| x.0), Ok(()));
        assert_eq!(v, [(1, 'a'), (2, 'b'), (3, 'c')]);
        assert!(is_strictly_increasing_by(&v, |x| x.0));
    }

    #[test]
    fn detects_duplicate_keys() {
        let mut v = [(1, 'a'), (1, 'b'), (0, 'z')];
        assert_eq!(sort_by_unique_key(&mut v, |x| x.0), Err(DuplicateKey));
        // With the unique tie-breaker appended the order is total again.
        assert_eq!(sort_by_unique_key(&mut v, |x| (x.0, x.1)), Ok(()));
        assert_eq!(v, [(0, 'z'), (1, 'a'), (1, 'b')]);
        let empty: [u8; 0] = [];
        assert!(is_strictly_increasing_by(&empty, |x| *x));
    }
}
