//! A little byte codec for handing a live link to another process (`enet::Host::export`, `slippi::Client::export`):
//! the opponent keeps one ENet session from the online select into the match, as with Slippi Dolphin's and Hero's
//! single in-process client.

#[derive(Default)]
pub struct W(pub Vec<u8>);
impl W {
    pub fn u8(&mut self, v: u8) {
        self.0.push(v);
    }
    pub fn u16(&mut self, v: u16) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    pub fn u32(&mut self, v: u32) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    pub fn u64(&mut self, v: u64) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    pub fn i64(&mut self, v: i64) {
        self.u64(v as u64);
    }
    pub fn u128(&mut self, v: u128) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    pub fn f32(&mut self, v: f32) {
        self.u32(v.to_bits());
    }
    pub fn bytes(&mut self, b: &[u8]) {
        self.u32(b.len() as u32);
        self.0.extend_from_slice(b);
    }
    pub fn addr(&mut self, a: crate::transport::Addr) {
        self.u32(a.ip);
        self.u16(a.port);
        self.u128(a.ip6);
    }
}

pub struct R<'a> {
    b: &'a [u8],
    o: usize,
}
impl<'a> R<'a> {
    pub fn new(b: &'a [u8]) -> Self {
        R { b, o: 0 }
    }
    fn take<const N: usize>(&mut self) -> Option<[u8; N]> {
        let s = self.b.get(self.o..self.o + N)?;
        self.o += N;
        s.try_into().ok()
    }
    pub fn u8(&mut self) -> Option<u8> {
        self.take::<1>().map(|s| s[0])
    }
    pub fn u16(&mut self) -> Option<u16> {
        self.take().map(u16::from_le_bytes)
    }
    pub fn u32(&mut self) -> Option<u32> {
        self.take().map(u32::from_le_bytes)
    }
    pub fn u64(&mut self) -> Option<u64> {
        self.take().map(u64::from_le_bytes)
    }
    pub fn i64(&mut self) -> Option<i64> {
        self.u64().map(|v| v as i64)
    }
    pub fn u128(&mut self) -> Option<u128> {
        self.take().map(u128::from_le_bytes)
    }
    pub fn f32(&mut self) -> Option<f32> {
        self.u32().map(f32::from_bits)
    }
    pub fn bytes(&mut self) -> Option<Vec<u8>> {
        let n = self.u32()? as usize;
        let s = self.b.get(self.o..self.o + n)?.to_vec();
        self.o += n;
        Some(s)
    }
    pub fn addr(&mut self) -> Option<crate::transport::Addr> {
        Some(crate::transport::Addr { ip: self.u32()?, port: self.u16()?, ip6: self.u128()? })
    }
    pub fn done(&self) -> bool {
        self.o == self.b.len()
    }
}
