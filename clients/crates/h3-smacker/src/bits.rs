//! FFmpeg's little-endian bit reader (`get_bits.h` with `BITSTREAM_READER_LE`), as
//! `libavcodec/smacker.c` builds it: the unchecked reader (`UNCHECKED_BITSTREAM_READER`), whose
//! position runs on past the end. Each byte's bits are taken from its lowest up; past the end
//! every bit reads as 0 (FFmpeg's zeroed padding) and the bits left go negative.

/// Bytes read bit by bit.
#[derive(Debug, Clone)]
pub(crate) struct Bits<'a> {
    bytes: &'a [u8],
    /// The next bit's position.
    index: usize,
}

impl<'a> Bits<'a> {
    pub(crate) fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, index: 0 }
    }

    /// The bits left (`get_bits_left`): negative once the reads ran past the end.
    pub(crate) fn left(&self) -> i64 {
        let size = i64::try_from(self.bytes.len()).unwrap_or(i64::MAX / 16) * 8;
        size - i64::try_from(self.index).unwrap_or(i64::MAX / 2)
    }

    /// One bit (`get_bits1`).
    pub(crate) fn bit(&mut self) -> u32 {
        let byte = self.bytes.get(self.index >> 3).copied().unwrap_or(0);
        let bit = (byte >> (self.index & 7)) & 1;
        self.index = self.index.saturating_add(1);
        u32::from(bit)
    }

    /// The next `n` bits, `n` at most 32, the first read in the lowest bit (`get_bits`).
    pub(crate) fn bits(&mut self, n: u32) -> u32 {
        let mut value = 0;
        for k in 0..n {
            value |= self.bit() << k;
        }
        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bits_come_lowest_first_and_zeros_past_the_end() {
        let mut bits = Bits::new(&[0b1010_0110, 0xFF]);
        assert_eq!(bits.left(), 16);
        assert_eq!(bits.bit(), 0);
        assert_eq!(bits.bits(3), 0b011);
        assert_eq!(bits.bits(8), 0b1111_1010);
        assert_eq!(bits.left(), 4);
        assert_eq!(bits.bits(6), 0b00_1111);
        assert_eq!(bits.left(), -2);
        assert_eq!(bits.bits(32), 0);
        assert_eq!(bits.left(), -34);
    }
}
