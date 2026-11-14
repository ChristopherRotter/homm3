//! Bink's sound (E27 M2 T2; CLT-Q104, decision 21): a clip's audio track decoded as the game's
//! `BINKW32.DLL` decodes it (`BinkAudioDecompressOpen` 0x1000F340, `BinkAudioDecompress`
//! 0x1000F480 and its block decoder 0x1000F580, as `BinkDoFrame` calls them at 0x10005A90):
//! samples the DLL's own, the FFT's floating point emulated in integers ([`crate::x87`]).
//!
//! **Origin.** The format and its reading follow FFmpeg 6.1.1's `libavcodec/binkaudio.c`
//! (Copyright (c) 2007-2011 Peter Ross, (c) 2009 Daniel Verkamp; its revision-`b` RDFT path:
//! `decode_init`, `decode_block`, `binkaudio_receive_frame`; the bands' frequencies
//! `libavcodec/wma_freqs.c`'s `ff_wma_critical_freqs`), under the LGPL 2.1 or later and
//! distributed under the GPL version 2 or later (ADR-0009, CLT-Q29, as `h3-bink`'s video). The
//! arithmetic and the loop are the DLL's instead (read in its code, design §8), where it differs:
//!
//! - **The block:** the first two words binary32 (the spectrum's ends), then a quantizer byte per
//!   band (no cap at 95: the DLL takes every byte), then the coefficients in runs of 16, each run
//!   a 4-bit width and its values, a value not 0 followed by its sign; every block starts on a
//!   word (FFmpeg's `get_bits_align32`).
//! - **The scales:** a band's is `10^(f32(q × 0.664f) × 0.1f)` rounded to binary32 (FFmpeg's
//!   `expf(q × 0.15289…)`); a coefficient is `f32(c × scale)`.
//! - **The transform:** the spectrum mirrored into a whole complex one and Ooura's complex inverse
//!   FFT run on it ([`crate::fft`]), where FFmpeg runs a real transform; the samples the real
//!   parts times `1/√frame` (binary32), rounded half away from zero and held to 16 bits
//!   ([`X::sample`]).
//! - **The overlap:** in integers on the 16-bit samples (each of the two terms divided on its
//!   own, `div` unsigned on the signed products: a floor, the length being a power of two), not
//!   in floating point before the rounding.
//! - **The packet:** blocks are decoded until its decoded size (its first word) is given, a
//!   block's samples past it dropped (FFmpeg decodes until its bits end); bits past the packet
//!   read as 0 (the DLL reads the memory after it). The size is held to the track's largest
//!   (the DLL's ring, sized by it, takes no more: 0x10005AC8).
//!
//! The DLL knows the revision-`b` RDFT coding alone (no DCT, `BinkAudioDecompressOpen`'s third
//! argument 1 always): other tracks are refused. A binary32 end that is not finite reads as its
//! bits' finite value, a subnormal as 0 (`X::from_bits32`; the game's clips have neither).

use alloc::vec;
use alloc::vec::Vec;

use crate::Error;
use crate::fft::Fft;
use crate::trig;
use crate::x87::X;

/// The bands' starting frequencies (the DLL's table 0x1001A614: `ff_wma_critical_freqs` after a
/// 0).
const FREQUENCIES: [u32; 25] = [
    0, 100, 200, 300, 400, 510, 630, 770, 920, 1080, 1270, 1480, 1720, 2000, 2320, 2700, 3150,
    3700, 4400, 5300, 6400, 7700, 9500, 12000, 15500,
];
/// The band scales' factors (0x100183AC and 0x100183E8): 0.664f and 0.1f.
const FACTOR: u32 = 0x3F29_FBE7;
const TENTH: u32 = 0x3DCC_CCCD;
/// `1/√frame` as binary32 (0x1000F3DA: `fsqrt`, `fdivr` at 53 bits, `fstp`), for 512, 1024 and
/// 2048.
const SCALES: [u32; 3] = [0x3D35_04F3, 0x3D00_0000, 0x3CB5_04F3];
/// A track's flags (the header's high half of the word): stereo, the DCT coding.
const STEREO: u16 = 0x2000;
const DCT: u16 = 0x1000;

/// An audio track as the header describes it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Track {
    /// Its largest decoded packet in bytes.
    pub largest: u32,
    /// Samples a second.
    pub rate: u16,
    /// Its flags (0x2000 stereo, 0x4000 16-bit, 0x1000 the DCT coding).
    pub flags: u16,
}

impl Track {
    /// Its channels.
    #[must_use]
    pub fn channels(self) -> u32 {
        if self.flags & STEREO != 0 { 2 } else { 1 }
    }
}

/// The band scales of the 256 quantizer bytes (`0x1000F62D`: `fild`, `fmul` 0.664f, `fstp`;
/// 0x1000F310: `fmul` 0.1f and the CRT's `pow`; `fstp`).
pub(crate) fn band_scales() -> [X; 256] {
    let mut out = [X::ZERO; 256];
    for (q, scale) in out.iter_mut().enumerate() {
        let x = X::int(q as i32).mul(X::from_bits32(FACTOR)).single();
        *scale = trig::pow10(x.mul(X::from_bits32(TENTH)));
    }
    out
}

/// A track's decoder: its tables and the overlap's state between blocks and packets.
#[derive(Debug, Clone)]
pub struct Audio {
    frame: usize,
    overlap: usize,
    /// The bands' first complex coefficients, the last the end (`frame / 2`).
    bands: Vec<usize>,
    scales: [X; 256],
    scale: X,
    fft: Fft,
    /// The last block's last `overlap` samples.
    previous: Vec<i16>,
    first: bool,
    largest: u32,
}

/// Bits read lowest first from 32-bit little-endian words, 0 past the end (the DLL's reader).
struct Bits<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl Bits<'_> {
    fn bits(&mut self, n: u32) -> u32 {
        let mut v = 0;
        for k in 0..n {
            let byte = self.bytes.get(self.at >> 3).copied().unwrap_or(0);
            v |= u32::from((byte >> (self.at & 7)) & 1) << k;
            self.at += 1;
        }
        v
    }

    /// The bytes read, to the next word (the block's end: 0x1000F923).
    fn words(&self) -> usize {
        self.at.div_ceil(32) * 4
    }
}

impl Audio {
    /// The decoder for `track` (`BinkAudioDecompressOpen(rate, channels, 1)`, as `BinkOpen`
    /// calls it at 0x10005195).
    pub fn new(track: Track) -> Result<Self, Error> {
        if track.rate == 0 {
            return Err(Error::Field("audio rate"));
        }
        if track.flags & DCT != 0 {
            return Err(Error::Field("audio coding"));
        }
        let rate = u32::from(track.rate);
        let (frame, scale) = if rate >= 44_100 {
            (2048, SCALES[2])
        } else if rate >= 22_050 {
            (1024, SCALES[1])
        } else {
            (512, SCALES[0])
        };
        let half = (rate * track.channels() + 1) >> 1;
        let mut bands: Vec<usize> = FREQUENCIES
            .iter()
            .take_while(|&&f| f < half)
            .map(|&f| (f * (frame as u32 / 2) / half) as usize)
            .collect();
        bands.push(frame / 2);
        Ok(Self {
            frame,
            overlap: frame / 16,
            bands,
            scales: band_scales(),
            scale: X::from_bits32(scale),
            fft: Fft::new(2 * frame),
            previous: vec![0; frame / 16],
            first: true,
            largest: track.largest,
        })
    }

    /// Decodes one of the track's packets (its bytes after its size word): its samples as the
    /// DLL hands them on, 16-bit little-endian, the channels interleaved.
    pub fn decode(&mut self, packet: &[u8]) -> Vec<u8> {
        let size = packet
            .first_chunk::<4>()
            .map_or(0, |w| u32::from_le_bytes(*w))
            .min(self.largest) as usize;
        let mut out = Vec::with_capacity(size);
        let mut at = 4;
        while out.len() < size {
            let block = packet.get(at..).unwrap_or_default();
            let (samples, used) = self.block(block);
            at += used;
            let left = size - out.len();
            out.extend(samples.iter().flat_map(|s| s.to_le_bytes()).take(left));
        }
        out
    }

    /// One block: its `frame − overlap` samples and the bytes it used.
    fn block(&mut self, bytes: &[u8]) -> (Vec<i16>, usize) {
        let word = |k: usize| {
            bytes
                .get(4 * k..4 * k + 4)
                .map_or(0, |w| u32::from_le_bytes([w[0], w[1], w[2], w[3]]))
        };
        let mut bits = Bits {
            bytes: bytes.get(8..).unwrap_or_default(),
            at: 0,
        };
        let frame = self.frame;
        // The spectrum: `frame` complex values, the second half the first's mirror.
        let mut s = vec![X::ZERO; 2 * frame];
        s[0] = X::from_bits32(word(0));
        s[frame] = X::from_bits32(word(1));
        let quantizers: Vec<X> = (1..self.bands.len())
            .map(|_| self.scales[bits.bits(8) as usize])
            .collect();
        let mut coefficients = vec![0i32; frame];
        let mut i = 2;
        while i < frame {
            let end = (i + 16).min(frame);
            let width = bits.bits(4);
            while i < end {
                if width != 0 {
                    let v = bits.bits(width) as i32;
                    coefficients[i] = if v != 0 && bits.bits(1) == 1 { -v } else { v };
                }
                i += 1;
            }
        }
        // Each band's pairs (the first, 0, holds the ends) times its scale.
        let mut pair = 1;
        for (k, q) in quantizers.iter().enumerate() {
            for p in self.bands[k]..self.bands[k + 1] {
                if p == 0 {
                    continue;
                }
                for part in 0..2 {
                    let c = coefficients[2 * pair + part];
                    s[2 * pair + part] = X::int(c).mul(*q).single();
                }
                pair += 1;
            }
        }
        for p in 1..=frame / 2 {
            s[2 * frame - 2 * p] = s[2 * p];
            s[2 * frame - 2 * p + 1] = s[2 * p + 1].neg();
        }
        self.fft.inverse(&mut s);
        let mut samples: Vec<i16> = (0..frame)
            .map(|k| self.scale.mul(s[2 * k]).sample())
            .collect();
        let n = self.overlap as u32;
        if !self.first {
            for (i, (sample, &before)) in samples.iter_mut().zip(&self.previous).enumerate() {
                let i = i as u32;
                let old = ((n - i) as i32).wrapping_mul(i32::from(before)) as u32 / n;
                let new = (i as i32).wrapping_mul(i32::from(*sample)) as u32 / n;
                *sample = old.wrapping_add(new) as u16 as i16;
            }
        }
        self.first = false;
        self.previous
            .copy_from_slice(&samples[frame - self.overlap..frame]);
        samples.truncate(frame - self.overlap);
        (samples, 8 + bits.words())
    }
}
