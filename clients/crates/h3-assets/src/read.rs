//! Bounded little-endian reads over untrusted bytes: each read checks its range and answers `None`
//! past the end, so no decoder indexes out of bounds.

/// `N` bytes at `at`.
pub(crate) fn array<const N: usize>(bytes: &[u8], at: usize) -> Option<[u8; N]> {
    bytes.get(at..)?.first_chunk::<N>().copied()
}

/// The `len` bytes at `at`.
pub(crate) fn slice(bytes: &[u8], at: usize, len: usize) -> Option<&[u8]> {
    bytes.get(at..)?.get(..len)
}

pub(crate) fn u8_at(bytes: &[u8], at: usize) -> Option<u8> {
    bytes.get(at).copied()
}

pub(crate) fn u16_at(bytes: &[u8], at: usize) -> Option<u16> {
    array(bytes, at).map(u16::from_le_bytes)
}

pub(crate) fn u32_at(bytes: &[u8], at: usize) -> Option<u32> {
    array(bytes, at).map(u32::from_le_bytes)
}

/// `N` consecutive little-endian `u32`s at `at`.
pub(crate) fn u32s<const N: usize>(bytes: &[u8], at: usize) -> Option<[u32; N]> {
    let bytes = bytes.get(at..)?.get(..N * 4)?;
    let mut words = [0; N];
    for (word, &chunk) in words.iter_mut().zip(bytes.as_chunks::<4>().0) {
        *word = u32::from_le_bytes(chunk);
    }
    Some(words)
}

/// A file's 32-bit size or offset as an index; saturates where `usize` is narrower, so that the
/// read that follows fails as out of range.
pub(crate) fn index(value: u32) -> usize {
    usize::try_from(value).unwrap_or(usize::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_are_little_endian_and_bounded() {
        let b = [1, 2, 3, 4, 0xFF];
        assert_eq!(u8_at(&b, 4), Some(0xFF));
        assert_eq!(u8_at(&b, 5), None);
        assert_eq!(u16_at(&b, 0), Some(0x0201));
        assert_eq!(u16_at(&b, 4), None);
        assert_eq!(u32_at(&b, 0), Some(0x0403_0201));
        assert_eq!(u32_at(&b, 2), None);
        assert_eq!(array::<2>(&b, 9), None);
        assert_eq!(slice(&b, 1, 2), Some(&b[1..3]));
        assert_eq!(slice(&b, 4, 2), None);
        assert_eq!(slice(&b, 6, 0), None);
        assert_eq!(index(7), 7);
        assert_eq!(u32s::<1>(&b, 1), Some([0xFF04_0302]));
        assert_eq!(u32s::<2>(&b, 0), None);
        assert_eq!(u32s::<1>(&b, 6), None);
    }
}
