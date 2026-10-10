//! A bounds-checked reader over the inflated map bytes: every read names its field, so a failure
//! says where the file broke off.

use crate::model::{Bits, Pos, Text};
use crate::{Limits, ReadError};

/// Little-endian reader over the inflated map.
#[derive(Debug)]
pub(crate) struct Cursor<'a> {
    bytes: &'a [u8],
    at: usize,
    /// The bounds the counts and strings are checked against.
    pub(crate) limits: &'a Limits,
}

impl<'a> Cursor<'a> {
    pub(crate) const fn new(bytes: &'a [u8], limits: &'a Limits) -> Self {
        Self {
            bytes,
            at: 0,
            limits,
        }
    }

    /// Offset of the next byte.
    pub(crate) const fn offset(&self) -> usize {
        self.at
    }

    /// Bytes not read yet.
    pub(crate) const fn remaining(&self) -> usize {
        self.bytes.len() - self.at
    }

    /// The next `n` bytes.
    pub(crate) fn take(&mut self, n: usize, field: &'static str) -> Result<&'a [u8], ReadError> {
        if n > self.remaining() {
            return Err(ReadError::Truncated {
                field,
                offset: self.at,
            });
        }
        let out = &self.bytes[self.at..self.at + n];
        self.at += n;
        Ok(out)
    }

    pub(crate) fn array<const N: usize>(
        &mut self,
        field: &'static str,
    ) -> Result<[u8; N], ReadError> {
        let mut out = [0; N];
        out.copy_from_slice(self.take(N, field)?);
        Ok(out)
    }

    pub(crate) fn skip(&mut self, n: usize, field: &'static str) -> Result<(), ReadError> {
        self.take(n, field).map(|_| ())
    }

    pub(crate) fn u8(&mut self, field: &'static str) -> Result<u8, ReadError> {
        self.array::<1>(field).map(|[b]| b)
    }

    pub(crate) fn i8(&mut self, field: &'static str) -> Result<i8, ReadError> {
        self.array::<1>(field).map(i8::from_le_bytes)
    }

    /// A boolean byte: any non-zero value is true, as the game reads it.
    pub(crate) fn bool(&mut self, field: &'static str) -> Result<bool, ReadError> {
        self.u8(field).map(|b| b != 0)
    }

    pub(crate) fn u16(&mut self, field: &'static str) -> Result<u16, ReadError> {
        self.array(field).map(u16::from_le_bytes)
    }

    pub(crate) fn u32(&mut self, field: &'static str) -> Result<u32, ReadError> {
        self.array(field).map(u32::from_le_bytes)
    }

    pub(crate) fn i32(&mut self, field: &'static str) -> Result<i32, ReadError> {
        self.array(field).map(i32::from_le_bytes)
    }

    /// A tile position: x, y, level, one byte each.
    pub(crate) fn pos(&mut self, field: &'static str) -> Result<Pos, ReadError> {
        self.array::<3>(field).map(|[x, y, z]| Pos { x, y, z })
    }

    /// A bit set of `len` bytes.
    pub(crate) fn bits(&mut self, len: usize, field: &'static str) -> Result<Bits, ReadError> {
        self.take(len, field).map(|b| Bits(b.to_vec()))
    }

    /// A string: a `u32` length, then that many bytes (Windows-1252 in the game's own maps).
    pub(crate) fn text(&mut self, field: &'static str) -> Result<Text, ReadError> {
        let len = self.count(field, self.limits.max_string, 1)?;
        self.take(len, field).map(|b| Text(b.to_vec()))
    }

    /// A `u32` count of records of at least `min_record` bytes each, checked against `max` and
    /// against the bytes left, so that no allocation exceeds what the file can hold.
    pub(crate) fn count(
        &mut self,
        field: &'static str,
        max: u32,
        min_record: usize,
    ) -> Result<usize, ReadError> {
        let offset = self.at;
        let n = self.u32(field)?;
        self.check_count(field, n, max, min_record, offset)
    }

    /// [`Self::count`] for a count that was read as a byte.
    pub(crate) fn count_u8(
        &mut self,
        field: &'static str,
        min_record: usize,
    ) -> Result<usize, ReadError> {
        let offset = self.at;
        let n = self.u8(field)?;
        self.check_count(field, u32::from(n), u32::from(u8::MAX), min_record, offset)
    }

    /// [`Self::count`] for a count that was read as a `u16`.
    pub(crate) fn count_u16(
        &mut self,
        field: &'static str,
        max: u32,
        min_record: usize,
    ) -> Result<usize, ReadError> {
        let offset = self.at;
        let n = self.u16(field)?;
        self.check_count(field, u32::from(n), max, min_record, offset)
    }

    fn check_count(
        &self,
        field: &'static str,
        n: u32,
        max: u32,
        min_record: usize,
        offset: usize,
    ) -> Result<usize, ReadError> {
        if n > max {
            return Err(ReadError::LimitExceeded {
                field,
                value: u64::from(n),
                max: u64::from(max),
                offset,
            });
        }
        // Saturates where a usize is narrower than 32 bits; the bytes-left check rejects it then.
        let n = usize::try_from(n).unwrap_or(usize::MAX);
        if n.saturating_mul(min_record) > self.remaining() {
            return Err(ReadError::Truncated { field, offset });
        }
        Ok(n)
    }
}
