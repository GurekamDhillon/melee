//! Wire encoding of gw_net.c: little-endian, field by field (never a struct copy).
//!
//! Every packet starts with an 18-byte header:
//! `+0 u16 magic 0x4E47 | +2 u8 type | +3 u8 flags (bit0: t_echo valid) | +4 u32 session |
//!  +8 u32 t_send | +12 u32 t_echo | +16 u16 echo_delay_ms`, then a type-specific body.

pub const MAGIC: u32 = 0x4E47;
pub const HDR_LEN: usize = 18;
pub const MAX_PACKET: usize = 1200;

pub const T_HELLO: u8 = 1;
pub const T_ACCEPT: u8 = 2;
pub const T_REFUSE: u8 = 3;
pub const T_READY: u8 = 4;
pub const T_START: u8 = 5;
pub const T_START_ACK: u8 = 6;
pub const T_INPUT: u8 = 7;
pub const T_QUIT: u8 = 8;
pub const T_LOBBY: u8 = 9;

/// Appends little-endian fields (`put16`/`put32`/`put64`).
#[derive(Default)]
pub struct Writer {
    pub buf: Vec<u8>,
}

impl Writer {
    pub fn new() -> Self {
        Writer { buf: Vec::with_capacity(MAX_PACKET + 64) }
    }
    pub fn len(&self) -> usize {
        self.buf.len()
    }
    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }
    pub fn u8(&mut self, v: u8) {
        self.buf.push(v);
    }
    pub fn u16(&mut self, v: u32) {
        self.buf.extend_from_slice(&(v as u16).to_le_bytes());
    }
    pub fn u32(&mut self, v: u32) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }
    pub fn u64(&mut self, v: u64) {
        self.u32(v as u32);
        self.u32((v >> 32) as u32);
    }
    pub fn bytes(&mut self, b: &[u8]) {
        self.buf.extend_from_slice(b);
    }
}

/// Reads little-endian fields; callers check `left()` first, as the C does with `end - p`.
pub struct Reader<'a> {
    pub buf: &'a [u8],
    pub pos: usize,
}

impl<'a> Reader<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        Reader { buf, pos: 0 }
    }
    pub fn left(&self) -> usize {
        self.buf.len() - self.pos
    }
    pub fn u8(&mut self) -> u8 {
        let v = self.buf[self.pos];
        self.pos += 1;
        v
    }
    pub fn u16(&mut self) -> u32 {
        let v = u16::from_le_bytes([self.buf[self.pos], self.buf[self.pos + 1]]);
        self.pos += 2;
        v as u32
    }
    pub fn u32(&mut self) -> u32 {
        let b = &self.buf[self.pos..self.pos + 4];
        self.pos += 4;
        u32::from_le_bytes([b[0], b[1], b[2], b[3]])
    }
    pub fn u64(&mut self) -> u64 {
        let lo = self.u32() as u64;
        let hi = self.u32() as u64;
        lo | (hi << 32)
    }
    pub fn take(&mut self, n: usize) -> &'a [u8] {
        let s = &self.buf[self.pos..self.pos + n];
        self.pos += n;
        s
    }
    pub fn skip(&mut self, n: usize) {
        self.pos += n;
    }
}
