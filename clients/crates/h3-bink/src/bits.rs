//! FFmpeg's little-endian bit reader (`get_bits.h` with `BITSTREAM_READER_LE`, the checked
//! reader): each byte's bits are taken from its lowest up; past the packet's end every bit reads
//! as 0 (FFmpeg's zeroed padding) and the position stops 8 bits past the end.

/// A packet read bit by bit.
#[derive(Debug)]
pub(crate) struct Bits<'a> {
    bytes: &'a [u8],
    /// The next bit's position.
    index: usize,
    /// The packet's size in bits.
    size: usize,
}

impl<'a> Bits<'a> {
    pub(crate) fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            index: 0,
            size: bytes.len().saturating_mul(8),
        }
    }

    /// The packet's size in bits.
    pub(crate) fn size(&self) -> usize {
        self.size
    }

    /// How many bits were read (`get_bits_count`).
    pub(crate) fn count(&self) -> usize {
        self.index
    }

    /// Whether at least `n` bits are left (`get_bits_left(gb) >= n`).
    pub(crate) fn has(&self, n: usize) -> bool {
        self.index.saturating_add(n) <= self.size
    }

    /// The next `n` bits, `n` at most 25, the first read in the lowest bit (`get_bits`).
    pub(crate) fn read(&mut self, n: u32) -> u32 {
        let first = self.index >> 3;
        let mut window = 0u64;
        for k in 0..5 {
            let byte = self.bytes.get(first + k).copied().unwrap_or(0);
            window |= u64::from(byte) << (8 * k);
        }
        let value = (window >> (self.index & 7)) & ((1 << n) - 1);
        self.skip(n as usize);
        value as u32
    }

    /// One bit, as a flag (`get_bits1`).
    pub(crate) fn flag(&mut self) -> bool {
        self.read(1) == 1
    }

    /// Skips `n` bits (`skip_bits_long`), stopping 8 bits past the end.
    pub(crate) fn skip(&mut self, n: usize) {
        self.index = self
            .index
            .saturating_add(n)
            .min(self.size.saturating_add(8));
    }

    /// Skips to the next multiple of 32 bits, where the next plane starts.
    pub(crate) fn align32(&mut self) {
        let rest = self.index & 31;
        if rest != 0 {
            self.skip(32 - rest);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bits_come_lowest_first_and_zero_past_the_end() {
        let mut bits = Bits::new(&[0b1011_0110, 0xFF]);
        assert_eq!(bits.size(), 16);
        assert!(!bits.flag());
        assert_eq!(bits.read(3), 0b011);
        assert_eq!(bits.read(8), 0b1111_1011);
        assert_eq!(bits.count(), 12);
        assert!(bits.has(4));
        assert!(!bits.has(5));
        // Four bits left, then zeros.
        assert_eq!(bits.read(13), 0b1111);
        assert_eq!(bits.count(), 24, "stops 8 bits past the end");
        assert_eq!(bits.read(25), 0);
        assert_eq!(bits.count(), 24);
    }

    #[test]
    fn reads_reach_25_bits_at_any_offset() {
        let bytes = [0xAB, 0xCD, 0xEF, 0x12, 0x34, 0x56];
        let whole = u64::from_le_bytes([0xAB, 0xCD, 0xEF, 0x12, 0x34, 0x56, 0, 0]);
        for at in 0..8 {
            let mut bits = Bits::new(&bytes);
            bits.skip(at);
            assert_eq!(u64::from(bits.read(25)), (whole >> at) & 0x1FF_FFFF);
        }
    }

    #[test]
    fn alignment_goes_to_the_next_32_bits() {
        let mut bits = Bits::new(&[0; 8]);
        bits.align32();
        assert_eq!(bits.count(), 0);
        bits.skip(1);
        bits.align32();
        assert_eq!(bits.count(), 32);
        bits.skip(33);
        bits.align32();
        assert_eq!(bits.count(), 72, "past the end it stops at the end and 8");
    }
}
