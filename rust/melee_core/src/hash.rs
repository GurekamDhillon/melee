//! Byte-order-neutral state hashing.
//!
//! [`StateHash`] feeds a value's *canonical* bytes to a [`StateHasher`], never its in-memory
//! representation: each scalar is written in GameCube (big-endian) order, pointers are never
//! hashed (hash what they identify: an index, an id, or the pointee), and padding is skipped.
//! Big-endian was chosen so a C-side walker over the existing big-endian guest memory can produce
//! the same stream by hashing each field's raw bytes; that lets the C build's hash oracle
//! (MELEE_TURBO_HASHLOG / MELEE_RB_HASH_DETAIL) be compared field-for-field with Rust.
//!
//! The hash is FNV-1a 64: for equality between builds, not security.

/// FNV-1a 64-bit hasher over the canonical byte stream.
#[derive(Clone, Debug)]
pub struct StateHasher {
    state: u64,
}

impl Default for StateHasher {
    fn default() -> Self {
        Self::new()
    }
}

impl StateHasher {
    pub const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    pub const PRIME: u64 = 0x0000_0100_0000_01b3;

    pub const fn new() -> Self {
        Self { state: Self::OFFSET }
    }
    #[inline]
    pub fn write(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.state = (self.state ^ b as u64).wrapping_mul(Self::PRIME);
        }
    }
    pub fn finish(&self) -> u64 {
        self.state
    }
}

/// A value that contributes deterministic, endian-independent bytes to a state hash.
pub trait StateHash {
    fn state_hash(&self, h: &mut StateHasher);
}

macro_rules! scalar {
    ($($t:ty),*) => {$(
        impl StateHash for $t {
            #[inline]
            fn state_hash(&self, h: &mut StateHasher) { h.write(&self.to_be_bytes()); }
        }
    )*};
}
scalar!(u8, i8, u16, i16, u32, i32, u64, i64);

/// Floats hash their exact bit pattern (-0.0 != 0.0, NaN payloads count), like a memory hash.
impl StateHash for f32 {
    #[inline]
    fn state_hash(&self, h: &mut StateHasher) {
        h.write(&self.to_bits().to_be_bytes());
    }
}
impl StateHash for f64 {
    #[inline]
    fn state_hash(&self, h: &mut StateHasher) {
        h.write(&self.to_bits().to_be_bytes());
    }
}
/// One byte, as C `bool`/`u8` flags are on the GameCube.
impl StateHash for bool {
    #[inline]
    fn state_hash(&self, h: &mut StateHasher) {
        h.write(&[*self as u8]);
    }
}
impl<T: StateHash, const N: usize> StateHash for [T; N] {
    fn state_hash(&self, h: &mut StateHasher) {
        self.as_slice().state_hash(h)
    }
}
impl<T: StateHash> StateHash for [T] {
    fn state_hash(&self, h: &mut StateHasher) {
        for v in self {
            v.state_hash(h);
        }
    }
}

/// Hash one value from a fresh hasher.
pub fn hash_of<T: StateHash + ?Sized>(v: &T) -> u64 {
    let mut h = StateHasher::new();
    v.state_hash(&mut h);
    h.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scalars_hash_big_endian_bytes() {
        let mut b = StateHasher::new();
        b.write(&[0x12, 0x34, 0x56, 0x78]);
        assert_eq!(hash_of(&0x1234_5678u32), b.finish());
    }

    #[test]
    fn fnv1a_reference_vector() {
        let mut h = StateHasher::new();
        h.write(b"a");
        assert_eq!(h.finish(), 0xaf63_dc4c_8601_ec8c);
    }

    #[test]
    fn floats_hash_bits() {
        assert_ne!(hash_of(&0.0f32), hash_of(&-0.0f32));
        assert_eq!(hash_of(&1.0f32), hash_of(&0x3f80_0000u32));
    }
}
