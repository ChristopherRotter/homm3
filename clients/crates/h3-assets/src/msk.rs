//! MSK files (E17 M1 T3): 14 bytes that accompany most adventure map object DEFs of the sprite
//! archives (same name): the object's width and height in tiles (at most 8 × 6 in the files),
//! then two masks of 6 bytes, each a grid of 8 × 6 tiles, one bit a tile. Which mask means what
//! (the tiles the object covers, its shadow?) and the bit order are not settled: the adventure
//! map's code reading decides (research item 9). The decoder keeps the bytes as they are.

use crate::Error;
use crate::read::array;

/// The size of an MSK file.
pub const MSK_BYTES: usize = 14;

/// A decoded MSK.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Mask {
    /// The object's width in tiles.
    pub width: u8,
    /// The object's height in tiles.
    pub height: u8,
    /// The two masks in file order.
    pub masks: [[u8; 6]; 2],
}

/// Decodes an MSK: exactly [`MSK_BYTES`] bytes.
pub fn decode_msk(bytes: &[u8]) -> Result<Mask, Error> {
    let [
        width,
        height,
        a0,
        a1,
        a2,
        a3,
        a4,
        a5,
        b0,
        b1,
        b2,
        b3,
        b4,
        b5,
    ] = array::<MSK_BYTES>(bytes, 0).ok_or(Error::Truncated)?;
    if bytes.len() > MSK_BYTES {
        return Err(Error::TrailingBytes);
    }
    Ok(Mask {
        width,
        height,
        masks: [[a0, a1, a2, a3, a4, a5], [b0, b1, b2, b3, b4, b5]],
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build;
    use proptest::prelude::*;

    #[test]
    fn a_mask_round_trips_and_its_length_is_exact() {
        let mask = Mask {
            width: 3,
            height: 2,
            masks: [[0, 0, 0, 0, 0xE0, 0xE0], [1, 2, 3, 4, 5, 6]],
        };
        let bytes = build::msk(&mask);
        assert_eq!(bytes.len(), MSK_BYTES);
        assert_eq!(decode_msk(&bytes), Ok(mask));
        assert_eq!(decode_msk(&bytes[..13]), Err(Error::Truncated));
        let mut long = bytes;
        long.push(0);
        assert_eq!(decode_msk(&long), Err(Error::TrailingBytes));
    }

    proptest! {
        #[test]
        fn decode_msk_never_panics(bytes in proptest::collection::vec(any::<u8>(), 0..20)) {
            let _ = decode_msk(&bytes);
        }
    }
}
