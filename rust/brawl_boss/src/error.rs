use core::fmt;

/// Every way a player's file can fail to decode. Decoding never panics and never reads out of bounds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    /// A read at `at` of `len` bytes past the end of a buffer of `size` bytes.
    Truncated { at: usize, len: usize, size: usize },
    /// Wrong magic / signature.
    BadMagic,
    /// An LZ77 stream is malformed (back-reference before the start, or ran out of input).
    Lz77 { at: usize },
    /// A declared size is implausible (over the decoder's limits).
    TooLarge { what: &'static str, value: usize },
    /// The header is inconsistent (sizes do not add up).
    BadHeader(&'static str),
    /// A field that must be a pointer is not in the relocation list (data offset of the field).
    NotAPointer { field: u32 },
    /// A pointer, string or import chain points outside the data.
    BadOffset { what: &'static str, at: u32 },
    /// The boss file was not found inside the archive.
    NotFound,
    /// A script is too long or runs off the data.
    BadScript { at: u32 },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

pub type Result<T> = core::result::Result<T, Error>;

pub(crate) fn get(d: &[u8], at: usize, len: usize) -> Result<&[u8]> {
    at.checked_add(len).and_then(|e| d.get(at..e)).ok_or(Error::Truncated { at, len, size: d.len() })
}
pub(crate) fn be16(d: &[u8], at: usize) -> Result<u16> {
    let b = get(d, at, 2)?;
    Ok(u16::from_be_bytes([b[0], b[1]]))
}
pub(crate) fn be32(d: &[u8], at: usize) -> Result<u32> {
    let b = get(d, at, 4)?;
    Ok(u32::from_be_bytes([b[0], b[1], b[2], b[3]]))
}
