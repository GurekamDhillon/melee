//! mu_net message codecs (docs/rust-port/notes/p2p-direct-plan.md section 2.8 / 2.9): little-endian, field by
//! field. Control messages travel on channel 0, input messages on channel 1, both inside protected packets.

use super::crypto::Bytes32;

/// Little-endian writer.
#[derive(Default)]
pub struct W(pub Vec<u8>);
impl W {
    pub fn u8(&mut self, v: u8) -> &mut Self {
        self.0.push(v);
        self
    }
    pub fn u16(&mut self, v: u16) -> &mut Self {
        self.0.extend_from_slice(&v.to_le_bytes());
        self
    }
    pub fn u32(&mut self, v: u32) -> &mut Self {
        self.0.extend_from_slice(&v.to_le_bytes());
        self
    }
    pub fn i32(&mut self, v: i32) -> &mut Self {
        self.u32(v as u32)
    }
    pub fn u64(&mut self, v: u64) -> &mut Self {
        self.0.extend_from_slice(&v.to_le_bytes());
        self
    }
    pub fn bytes(&mut self, b: &[u8]) -> &mut Self {
        self.0.extend_from_slice(b);
        self
    }
    /// `str8(max)`: u8 length then the bytes. The caller has checked the length.
    pub fn str8(&mut self, s: &str, max: usize) -> &mut Self {
        let b = &s.as_bytes()[..s.len().min(max)];
        self.u8(b.len() as u8).bytes(b)
    }
}

/// Little-endian reader; any read past the end sets `bad` and yields zeros.
pub struct R<'a> {
    d: &'a [u8],
    at: usize,
    pub bad: bool,
}
impl<'a> R<'a> {
    pub fn new(d: &'a [u8]) -> Self {
        R { d, at: 0, bad: false }
    }
    pub fn take(&mut self, n: usize) -> &'a [u8] {
        if self.bad || self.d.len() - self.at < n {
            self.bad = true;
            return &[];
        }
        let s = &self.d[self.at..self.at + n];
        self.at += n;
        s
    }
    pub fn arr<const N: usize>(&mut self) -> [u8; N] {
        let s = self.take(N);
        if s.len() == N {
            s.try_into().unwrap()
        } else {
            [0; N]
        }
    }
    pub fn u8(&mut self) -> u8 {
        self.arr::<1>()[0]
    }
    pub fn u16(&mut self) -> u16 {
        u16::from_le_bytes(self.arr())
    }
    pub fn u32(&mut self) -> u32 {
        u32::from_le_bytes(self.arr())
    }
    pub fn i32(&mut self) -> i32 {
        i32::from_le_bytes(self.arr())
    }
    pub fn u64(&mut self) -> u64 {
        u64::from_le_bytes(self.arr())
    }
    pub fn str8(&mut self, max: usize) -> String {
        let n = self.u8() as usize;
        if n > max {
            self.bad = true;
            return String::new();
        }
        match std::str::from_utf8(self.take(n)) {
            Ok(s) => s.to_string(),
            Err(_) => {
                self.bad = true;
                String::new()
            }
        }
    }
    pub fn done(&self) -> bool {
        !self.bad && self.at == self.d.len()
    }
}

pub const CH_CONTROL: u8 = 0;
pub const CH_INPUT: u8 = 1;

pub const T_VERIFY: u8 = 0x01;
pub const T_BYE: u8 = 0x02;
pub const T_RESULT: u8 = 0x03;
pub const T_INFO: u8 = 0x04;
pub const T_NEXT: u8 = 0x05;
pub const T_COMMIT: u8 = 0x10;
pub const T_ACK: u8 = 0x11;
pub const T_PING: u8 = 0x12;
pub const T_PONG: u8 = 0x13;
pub const T_SAMPLE: u8 = 0x14;

pub const BYE_LEFT: u8 = 0;
pub const BYE_MISMATCH: u8 = 1;

pub const PAD: usize = 8;
pub const COMMIT_HEADER: usize = 22;
pub const MAX_COMMIT_FRAMES: usize = 128;
pub const SAMPLE_SIZE: usize = 21;
pub const SAMPLE_VERSION: u8 = 1;
/// Largest plain message.
pub const MAX_PLAIN: usize = 1200;

pub type Pad = [u8; PAD];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Commit {
    /// newest committed frame carried (wire number, with the game's base)
    pub frame: i32,
    pub checksum_frame: i32,
    pub checksum: u32,
    pub ack: i32,
    pub stamp: u32,
    /// pads of `frame`, `frame - 1`, ... (newest first), 1..=128
    pub pads: Vec<Pad>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Msg {
    Verify { digest: Bytes32, slot: u8 },
    Bye { reason: u8 },
    Result { frames: i32, digest: Bytes32 },
    Info { revision: u8, games: u16 },
    Next { game: u16, digest: Bytes32 },
    Commit(Commit),
    Ack { ack: i32, echo: u32 },
    Ping { stamp: u32 },
    Pong { stamp: u32 },
    Sample { version: u8, target: i32, slot: u8, pad: Pad },
    /// a type this side does not know, or a known type of the wrong length: counted and dropped
    Unknown { kind: u8, len: usize },
}

impl Msg {
    /// The channel this message travels on.
    pub fn channel(&self) -> u8 {
        match self {
            Msg::Verify { .. } | Msg::Bye { .. } | Msg::Result { .. } | Msg::Info { .. } | Msg::Next { .. } => CH_CONTROL,
            Msg::Unknown { kind, .. } if *kind < 0x10 => CH_CONTROL,
            _ => CH_INPUT,
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut w = W::default();
        match self {
            Msg::Verify { digest, slot } => w.u8(T_VERIFY).bytes(digest).u8(*slot),
            Msg::Bye { reason } => w.u8(T_BYE).u8(*reason),
            Msg::Result { frames, digest } => w.u8(T_RESULT).i32(*frames).bytes(digest),
            Msg::Info { revision, games } => w.u8(T_INFO).u8(*revision).u16(*games),
            Msg::Next { game, digest } => w.u8(T_NEXT).u16(*game).bytes(digest),
            Msg::Commit(c) => {
                w.u8(T_COMMIT).i32(c.frame).u8(c.pads.len() as u8).i32(c.checksum_frame).u32(c.checksum).i32(c.ack).u32(c.stamp);
                for p in &c.pads {
                    w.bytes(p);
                }
                &mut w
            }
            Msg::Ack { ack, echo } => w.u8(T_ACK).i32(*ack).u32(*echo),
            Msg::Ping { stamp } => w.u8(T_PING).u32(*stamp),
            Msg::Pong { stamp } => w.u8(T_PONG).u32(*stamp),
            Msg::Sample { version, target, slot, pad } => w.u8(T_SAMPLE).u8(*version).i32(*target).u8(*slot).bytes(pad).bytes(&[0; 6]),
            Msg::Unknown { kind, .. } => w.u8(*kind),
        };
        w.0
    }

    /// Parses one message. Exact lengths only; a COMMIT's count must be 1..=128 and match the length. The remaining
    /// COMMIT rules (frame >= count, non-negative checksum frame and ack, ack within what was sent) are the input
    /// layer's, which knows the game's frame base.
    pub fn decode(d: &[u8]) -> Option<Msg> {
        if d.is_empty() || d.len() > MAX_PLAIN {
            return None;
        }
        let kind = d[0];
        let mut r = R::new(&d[1..]);
        let unknown = Msg::Unknown { kind, len: d.len() };
        let m = match (kind, d.len()) {
            (T_VERIFY, 34) => Msg::Verify { digest: r.arr(), slot: r.u8() },
            (T_BYE, 2) => Msg::Bye { reason: r.u8() },
            (T_RESULT, 37) => Msg::Result { frames: r.i32(), digest: r.arr() },
            (T_INFO, 4) => Msg::Info { revision: r.u8(), games: r.u16() },
            (T_NEXT, 35) => Msg::Next { game: r.u16(), digest: r.arr() },
            (T_COMMIT, n) if n >= COMMIT_HEADER => {
                let frame = r.i32();
                let count = r.u8() as usize;
                let (checksum_frame, checksum, ack, stamp) = (r.i32(), r.u32(), r.i32(), r.u32());
                if count == 0 || count > MAX_COMMIT_FRAMES || n != COMMIT_HEADER + PAD * count {
                    return Some(unknown);
                }
                let pads = (0..count).map(|_| r.arr::<PAD>()).collect();
                Msg::Commit(Commit { frame, checksum_frame, checksum, ack, stamp, pads })
            }
            (T_ACK, 9) => Msg::Ack { ack: r.i32(), echo: r.u32() },
            (T_PING, 5) => Msg::Ping { stamp: r.u32() },
            (T_PONG, 5) => Msg::Pong { stamp: r.u32() },
            (T_SAMPLE, SAMPLE_SIZE) => {
                let m = Msg::Sample { version: r.u8(), target: r.i32(), slot: r.u8(), pad: r.arr() };
                r.take(6);
                m
            }
            _ => return Some(unknown),
        };
        if r.done() {
            Some(m)
        } else {
            Some(unknown)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes_and_round_trips() {
        let c = Commit { frame: 9, checksum_frame: 3, checksum: 0xdead_beef, ack: 4, stamp: 77, pads: vec![[1; 8], [2; 8]] };
        let cases = [
            (Msg::Verify { digest: [5; 32], slot: 1 }, 34),
            (Msg::Bye { reason: BYE_MISMATCH }, 2),
            (Msg::Result { frames: 600, digest: [6; 32] }, 37),
            (Msg::Info { revision: 2, games: 0 }, 4),
            (Msg::Next { game: 2, digest: [7; 32] }, 35),
            (Msg::Commit(c), 22 + 16),
            (Msg::Ack { ack: 12, echo: 99 }, 9),
            (Msg::Ping { stamp: 1 }, 5),
            (Msg::Pong { stamp: 1 }, 5),
            (Msg::Sample { version: 1, target: 40, slot: 0, pad: [3; 8] }, 21),
        ];
        for (m, n) in cases {
            let b = m.encode();
            assert_eq!(b.len(), n, "{m:?}");
            assert_eq!(Msg::decode(&b), Some(m));
        }
    }

    #[test]
    fn layout_is_little_endian_in_field_order() {
        let c = Commit { frame: 0x0102_0304, checksum_frame: 0x11, checksum: 0x2233_4455, ack: 0x66, stamp: 0x7788_99aa, pads: vec![[0xee; 8]] };
        let b = Msg::Commit(c).encode();
        assert_eq!(&b[..22], &[0x10, 4, 3, 2, 1, 1, 0x11, 0, 0, 0, 0x55, 0x44, 0x33, 0x22, 0x66, 0, 0, 0, 0xaa, 0x99, 0x88, 0x77]);
        let s = Msg::Sample { version: 1, target: 0x0a0b, slot: 1, pad: [9; 8] }.encode();
        assert_eq!(&s[..7], &[0x14, 1, 0x0b, 0x0a, 0, 0, 1]);
        assert_eq!(&s[15..], &[0; 6]);
    }

    #[test]
    fn wrong_lengths_are_unknown() {
        let mut b = Msg::Commit(Commit { frame: 2, checksum_frame: 0, checksum: 0, ack: 0, stamp: 0, pads: vec![[0; 8]; 2] }).encode();
        b.push(0);
        assert!(matches!(Msg::decode(&b), Some(Msg::Unknown { kind: T_COMMIT, .. })));
        b.truncate(22);
        b[5] = 0; // count 0
        assert!(matches!(Msg::decode(&b), Some(Msg::Unknown { .. })));
        assert!(matches!(Msg::decode(&[T_VERIFY, 0]), Some(Msg::Unknown { kind: T_VERIFY, len: 2 })));
        assert!(matches!(Msg::decode(&[0x7f]), Some(Msg::Unknown { kind: 0x7f, .. })));
        assert_eq!(Msg::decode(&[]), None);
        assert_eq!(Msg::decode(&vec![T_PING; 1201]), None);
    }
}
