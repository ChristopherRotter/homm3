//! Inflating a map file: the game stores a map as one gzip member (RFC 1952). The header's
//! optional fields are skipped, the deflate data is inflated within a size limit, and the
//! trailer's CRC-32 and length must match what was inflated.

use miniz_oxide::inflate::{TINFLStatus, decompress_to_vec_with_limit};

use crate::ReadError;

/// gzip flag bits (RFC 1952 §2.3.1).
const FHCRC: u8 = 1 << 1;
const FEXTRA: u8 = 1 << 2;
const FNAME: u8 = 1 << 3;
const FCOMMENT: u8 = 1 << 4;
/// Bits 5–7 are reserved and must be zero.
const RESERVED: u8 = 0b1110_0000;
/// Length of the fixed header and of the trailer.
const HEADER: usize = 10;
const TRAILER: usize = 8;

/// Inflates `file`, a single gzip member, to at most `max` bytes.
///
/// # Errors
///
/// [`ReadError::NotGzip`] for anything but a well-formed deflate member, [`ReadError::MapTooLarge`]
/// when it inflates beyond `max`, [`ReadError::ChecksumMismatch`] when the trailer disagrees.
pub fn gunzip(file: &[u8], max: usize) -> Result<Vec<u8>, ReadError> {
    let start = data_start(file).ok_or(ReadError::NotGzip)?;
    let trailer_at = file.len().checked_sub(TRAILER).ok_or(ReadError::NotGzip)?;
    let deflate = file.get(start..trailer_at).ok_or(ReadError::NotGzip)?;
    let data = decompress_to_vec_with_limit(deflate, max).map_err(|e| match e.status {
        TINFLStatus::HasMoreOutput => ReadError::MapTooLarge { max },
        _ => ReadError::NotGzip,
    })?;
    let trailer = &file[trailer_at..];
    let crc = u32::from_le_bytes([trailer[0], trailer[1], trailer[2], trailer[3]]);
    let size = u32::from_le_bytes([trailer[4], trailer[5], trailer[6], trailer[7]]);
    // ISIZE is the length modulo 2^32 (RFC 1952 §2.3.1), so the cast truncates on purpose.
    let inflated = data.len() as u32;
    if crc32(&data) == crc && inflated == size {
        Ok(data)
    } else {
        Err(ReadError::ChecksumMismatch)
    }
}

/// Offset of the deflate data, after the header and its optional fields.
fn data_start(file: &[u8]) -> Option<usize> {
    let header = file.get(..HEADER)?;
    if header[..3] != [0x1f, 0x8b, 8] || header[3] & RESERVED != 0 {
        return None;
    }
    let flags = header[3];
    let mut at = HEADER;
    if flags & FEXTRA != 0 {
        let len = file.get(at..at + 2)?;
        at += 2 + usize::from(u16::from_le_bytes([len[0], len[1]]));
    }
    for flag in [FNAME, FCOMMENT] {
        if flags & flag != 0 {
            at += file.get(at..)?.iter().position(|&b| b == 0)? + 1;
        }
    }
    if flags & FHCRC != 0 {
        at += 2;
    }
    Some(at)
}

/// The CRC-32 of RFC 1952 (reflected polynomial `0xEDB88320`).
pub fn crc32(data: &[u8]) -> u32 {
    !data.iter().fold(!0, |crc, &b| {
        CRC_TABLE[usize::from((crc as u8) ^ b)] ^ (crc >> 8)
    })
}

/// The byte-wise table of [`crc32`].
const CRC_TABLE: [u32; 256] = {
    let mut table = [0u32; 256];
    let mut i = 0;
    while i < 256 {
        let mut c = i as u32;
        let mut k = 0;
        while k < 8 {
            c = if c & 1 == 0 {
                c >> 1
            } else {
                0xEDB8_8320 ^ (c >> 1)
            };
            k += 1;
        }
        table[i] = c;
        i += 1;
    }
    table
};
