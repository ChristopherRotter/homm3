//! Canonical, versioned, platform-independent state hashing (ADR-0003 §11).
//!
//! State is hashed by feeding a canonical little-endian encoding of its fields, in a documented
//! order, into 64-bit FNV-1a. The encoding never includes `usize`, padding, pointers or iteration
//! order of unordered containers, so the hash is identical on every supported target.
//!
//! The hash detects desyncs and verifies replays; it is not a cryptographic commitment and must
//! not be exposed to untrusted clients in server mode (it is computed over hidden state such as
//! the RNG, docs/06-security.md "Side channels").

// rule: KER-HSH-001

/// Algorithm identifier recorded in replay headers.
pub const HASH_ALGORITHM: &str = "fnv1a64-le";
/// Version of the hashing scheme (algorithm + encoding conventions of this module).
pub const HASH_VERSION: u32 = 1;

const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

/// Streaming 64-bit FNV-1a over canonical little-endian encodings.
#[derive(Debug, Clone)]
pub struct StateHasher {
    state: u64,
}

impl Default for StateHasher {
    fn default() -> Self {
        Self::new()
    }
}

impl StateHasher {
    /// A hasher in its initial state.
    pub const fn new() -> Self {
        Self {
            state: FNV_OFFSET_BASIS,
        }
    }

    /// Feeds raw bytes.
    pub fn write_bytes(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.state ^= u64::from(b);
            self.state = self.state.wrapping_mul(FNV_PRIME);
        }
    }

    /// Feeds a `u8`.
    pub fn write_u8(&mut self, v: u8) {
        self.write_bytes(&[v]);
    }

    /// Feeds a `u16` (little-endian).
    pub fn write_u16(&mut self, v: u16) {
        self.write_bytes(&v.to_le_bytes());
    }

    /// Feeds a `u32` (little-endian).
    pub fn write_u32(&mut self, v: u32) {
        self.write_bytes(&v.to_le_bytes());
    }

    /// Feeds a `u64` (little-endian).
    pub fn write_u64(&mut self, v: u64) {
        self.write_bytes(&v.to_le_bytes());
    }

    /// Feeds an `i32` (two's complement, little-endian).
    pub fn write_i32(&mut self, v: i32) {
        self.write_bytes(&v.to_le_bytes());
    }

    /// Feeds a `bool` as one byte (0 or 1).
    pub fn write_bool(&mut self, v: bool) {
        self.write_u8(u8::from(v));
    }

    /// Feeds a length prefix (as `u32`), saturating for absurdly large lengths.
    pub fn write_len(&mut self, len: usize) {
        self.write_u32(u32::try_from(len).unwrap_or(u32::MAX));
    }

    /// The hash of everything fed so far.
    pub const fn finish(&self) -> u64 {
        self.state
    }
}

/// Types with a canonical hash encoding. Implementations must feed every field that affects
/// behavior, in a fixed order, and document additions by bumping the owner's hash version.
pub trait CanonicalHash {
    /// Feeds the canonical encoding of `self` into `h`.
    fn hash_into(&self, h: &mut StateHasher);
}

/// The canonical hash of a value.
pub fn hash_of<T: CanonicalHash + ?Sized>(value: &T) -> u64 {
    let mut h = StateHasher::new();
    value.hash_into(&mut h);
    h.finish()
}

macro_rules! impl_int {
    ($($t:ty => $m:ident),*) => {
        $(impl CanonicalHash for $t {
            fn hash_into(&self, h: &mut StateHasher) {
                h.$m(*self);
            }
        })*
    };
}
impl_int!(u8 => write_u8, u16 => write_u16, u32 => write_u32, u64 => write_u64, i32 => write_i32, bool => write_bool);

impl<T: CanonicalHash> CanonicalHash for Option<T> {
    fn hash_into(&self, h: &mut StateHasher) {
        match self {
            None => h.write_u8(0),
            Some(v) => {
                h.write_u8(1);
                v.hash_into(h);
            }
        }
    }
}

impl<T: CanonicalHash> CanonicalHash for [T] {
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_len(self.len());
        for v in self {
            v.hash_into(h);
        }
    }
}

impl CanonicalHash for str {
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_len(self.len());
        h.write_bytes(self.as_bytes());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fnv(bytes: &[u8]) -> u64 {
        let mut h = StateHasher::new();
        h.write_bytes(bytes);
        h.finish()
    }

    // covers: KER-HSH-001
    #[test]
    fn fnv1a64_reference_vectors() {
        // Published FNV-1a 64-bit test vectors (Fowler/Noll/Vo reference test suite).
        assert_eq!(fnv(b""), 0xcbf2_9ce4_8422_2325);
        assert_eq!(fnv(b"a"), 0xaf63_dc4c_8601_ec8c);
        assert_eq!(fnv(b"foobar"), 0x8594_4171_f739_67e8);
        assert_eq!(StateHasher::default().finish(), fnv(b""));
    }

    // covers: KER-HSH-001
    #[test]
    fn integers_are_little_endian() {
        let mut a = StateHasher::new();
        a.write_u32(0x0403_0201);
        assert_eq!(a.finish(), fnv(&[1, 2, 3, 4]));
        let mut b = StateHasher::new();
        b.write_u64(0x0807_0605_0403_0201);
        assert_eq!(b.finish(), fnv(&[1, 2, 3, 4, 5, 6, 7, 8]));
        let mut c = StateHasher::new();
        c.write_i32(-1);
        assert_eq!(c.finish(), fnv(&[0xff; 4]));
        let mut d = StateHasher::new();
        d.write_u16(0x0201);
        d.write_bool(true);
        d.write_u8(9);
        assert_eq!(d.finish(), fnv(&[1, 2, 1, 9]));
        let mut e = StateHasher::new();
        e.write_len(usize::MAX);
        assert_eq!(e.finish(), fnv(&[0xff; 4]));
    }

    // covers: KER-HSH-001
    #[test]
    fn composite_encodings() {
        assert_eq!(hash_of(&None::<u8>), fnv(&[0]));
        assert_eq!(hash_of(&Some(7u8)), fnv(&[1, 7]));
        assert_eq!(hash_of(&[5u8, 6][..]), fnv(&[2, 0, 0, 0, 5, 6]));
        assert_eq!(hash_of("ab"), fnv(&[2, 0, 0, 0, b'a', b'b']));
        assert_eq!(hash_of(&3u16), fnv(&[3, 0]));
        assert_eq!(hash_of(&3u32), fnv(&[3, 0, 0, 0]));
        assert_eq!(hash_of(&3u64), fnv(&[3, 0, 0, 0, 0, 0, 0, 0]));
        assert_eq!(hash_of(&-2i32), fnv(&[0xfe, 0xff, 0xff, 0xff]));
        assert_eq!(hash_of(&false), fnv(&[0]));
        assert_eq!(HASH_ALGORITHM, "fnv1a64-le");
        assert_eq!(HASH_VERSION, 1);
    }
}
