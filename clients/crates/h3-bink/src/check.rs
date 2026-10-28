//! What the checks compare (E17 M5 T4b2a): a clip's every frame decoded and the frames' hash,
//! for the decoder's tests and `h3-view video` (against FFmpeg's decoding of the same file).

use alloc::vec::Vec;

use crate::{Clip, Decoder, Error, Frame};

/// Every frame of the Bink file `bytes` decoded in order, those that fail left out (FFmpeg
/// outputs none for them; the next decodes over what they left).
pub fn frames(bytes: &[u8]) -> Result<Vec<Frame>, Error> {
    let clip = Clip::parse(bytes)?;
    let mut decoder = Decoder::new(&clip)?;
    let mut frames = Vec::new();
    for n in 0..clip.frames().len() {
        if let Ok(frame) = decoder.decode(clip.video(n)?) {
            frames.push(frame.clone());
        }
    }
    Ok(frames)
}

/// The FNV-1a hash of the frames' pictures, plane by plane, row by row (the bytes FFmpeg's
/// `rawvideo` output of `yuv420p` holds).
#[must_use]
pub fn hash(frames: &[Frame]) -> u64 {
    let mut hash: u64 = 0xCBF2_9CE4_8422_2325;
    for frame in frames {
        for plane in frame.planes() {
            for y in 0..plane.height() {
                for &byte in plane.row(y) {
                    hash = (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01B3);
                }
            }
        }
    }
    hash
}
