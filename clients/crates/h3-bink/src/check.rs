//! What the checks compare (E17 M5 T4b2a): a clip's every frame decoded and the frames' hash,
//! for the decoder's tests and `h3-view video` (against `BINKW32.DLL`'s decoding of the same
//! file since E17 M6 T6c1, FFmpeg's before).

use alloc::vec::Vec;

use crate::audio::Audio;
use crate::fft::Fft;
use crate::x87::X;
use crate::{Clip, Decoder, Error, Frame, Reading};

/// Every frame of the Bink file `bytes` decoded in order ([`Reading::Fixed`]), those that fail
/// left out (FFmpeg outputs none for them; the next decodes over what they left).
pub fn frames(bytes: &[u8]) -> Result<Vec<Frame>, Error> {
    frames_read(bytes, Reading::Fixed)
}

/// [`frames`], read as `reading` says.
pub fn frames_read(bytes: &[u8], reading: Reading) -> Result<Vec<Frame>, Error> {
    let clip = Clip::parse(bytes)?;
    let mut decoder = Decoder::new(&clip)?.reading(reading);
    let mut frames = Vec::new();
    for n in 0..clip.frames().len() {
        if let Ok(frame) = decoder.decode(clip.video(n)?) {
            frames.push(frame.clone());
        }
    }
    Ok(frames)
}

/// The FNV-1a hash of the frames' pictures, plane by plane, row by row (the bytes of raw YUV
/// 4:2:0, FFmpeg's `rawvideo` output of `yuv420p`, `bink-dll.py`'s).
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

/// The first audio track's sound (E27 M2 T2): every frame's packet decoded in order by
/// [`Audio`], the bytes as `binkprobe sound` writes the DLL's; empty without a track.
pub fn sound(bytes: &[u8]) -> Result<Vec<u8>, Error> {
    let clip = Clip::parse(bytes)?;
    let Some(&track) = clip.tracks.first() else {
        return Ok(Vec::new());
    };
    let mut audio = Audio::new(track)?;
    let mut out = Vec::new();
    for n in 0..clip.frames().len() {
        if let Some(packet) = clip.audio(n, 0)? {
            out.extend(audio.decode(packet));
        }
    }
    Ok(out)
}

/// The sound decoder's tables as `binkprobe sound-tables` writes the DLL's (E27 M2 T2): the 256
/// band scales, then the twiddles of the transforms of 1024, 2048 and 4096 values, each a
/// binary32's little-endian bits.
#[must_use]
pub fn sound_tables() -> Vec<u8> {
    let bits = |x: &X| x.to_bits32().unwrap_or(0).to_le_bytes();
    let mut out: Vec<u8> = crate::audio::band_scales().iter().flat_map(bits).collect();
    for n in [1024, 2048, 4096] {
        out.extend(Fft::new(n).twiddles().iter().flat_map(bits));
    }
    out
}

/// The sound decoder's inverse FFT of `n` binary32 values (their bits) per run of `bits`, as
/// `binkprobe cdft` runs the DLL's (E27 M2 T2: the transform checked on any values; one table
/// for all runs, as a decoder keeps it).
#[must_use]
pub fn inverse_fft(n: usize, bits: &[u32]) -> Vec<u32> {
    let mut fft = Fft::new(n);
    let mut out = Vec::with_capacity(bits.len());
    for run in bits.chunks_exact(n) {
        let mut a: Vec<X> = run.iter().map(|&b| X::from_bits32(b)).collect();
        fft.inverse(&mut a);
        out.extend(a.iter().map(|x| x.single().to_bits32().unwrap_or(0)));
    }
    out
}
