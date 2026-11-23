//! The handshake and packet protection (plan sections 2.3 to 2.5): the signed HELLO, the key schedule, and the
//! XChaCha20-Poly1305 packets with their replay rules.

use super::blake2b::{hash32, keyed32};
use super::crypto::{random, verify, xopen, xseal, Bytes32, Bytes64, Ephemeral, Identity};
use super::desc::PROTOCOL_VERSION;

pub const HELLO_SIZE: usize = 160;
pub const PACKET_VERSION: u8 = 1;
pub const PACKET_HEADER: usize = 10;
pub const TAG: usize = 16;
pub const MAX_PACKET: usize = PACKET_HEADER + super::wire::MAX_PLAIN + TAG;
const HELLO_MAGIC: &[u8; 4] = b"MUNH";
const D_HELLO: &[u8] = b"MeleeUnlockedNet1 hello";
const D_SESSION: &[u8] = b"MeleeUnlockedNet1 session key";
const D_TRANSCRIPT: &[u8] = b"MeleeUnlockedNet1 transcript";
const D_DIRECTION: &[u8] = b"MeleeUnlockedNet1 direction";
const REPLAY_WINDOW: u64 = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SecError {
    Malformed,
    /// the peer runs another protocol version
    Version,
    /// a second, different HELLO
    Changed,
    /// our own HELLO came back
    Reflected,
    /// the identity is not the one the lobby named
    Unexpected,
    Signature,
    /// the X25519 result was all zero (or the secret was already used)
    KeyAgreement,
    NoHandshake,
    Replay,
    Tag,
}

impl SecError {
    pub fn text(self) -> &'static str {
        match self {
            SecError::Malformed => "a malformed hello or packet",
            SecError::Version => "the other player runs a different network version",
            SecError::Changed => "the other side sent a second, different hello",
            SecError::Reflected => "our own hello came back",
            SecError::Unexpected => "the other side is not the player the lobby named",
            SecError::Signature => "the hello's signature does not verify",
            SecError::KeyAgreement => "the key agreement failed",
            SecError::NoHandshake => "a packet before the handshake",
            SecError::Replay => "a replayed or stale packet",
            SecError::Tag => "a packet that does not decrypt",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Accepted {
    /// the first valid HELLO: the keys exist now
    New,
    /// byte-for-byte the HELLO already accepted (it came over a second connection)
    Same,
}

/// One side of a session's security: its HELLO, then the two direction keys.
pub struct Security {
    identity: Identity,
    expected: Option<Bytes32>,
    eph: Ephemeral,
    hello: [u8; HELLO_SIZE],
    peer_hello: Option<[u8; HELLO_SIZE]>,
    send_key: Bytes32,
    recv_key: Bytes32,
    decider: bool,
    send_seq: u64,
    last_ctl: Option<u64>,
    /// channel 1: the highest accepted sequence and a bitmap of the 64 below it (bit 0 = the highest itself)
    win_top: Option<u64>,
    win_bits: u64,
}

impl Security {
    /// Makes this session's ephemeral key and HELLO. `expected`: the only identity accepted from the peer.
    pub fn begin(identity: Identity, expected: Option<Bytes32>) -> Option<Security> {
        let eph = Ephemeral::generate()?;
        let mut hello = [0u8; HELLO_SIZE];
        hello[..4].copy_from_slice(HELLO_MAGIC);
        hello[4..6].copy_from_slice(&PROTOCOL_VERSION.to_le_bytes());
        hello[8..40].copy_from_slice(&identity.public_key());
        hello[40..72].copy_from_slice(eph.public());
        if !random(&mut hello[72..96]) {
            return None;
        }
        let sig = identity.sign(&[D_HELLO, &hello[..96]]);
        hello[96..].copy_from_slice(&sig);
        Some(Security {
            identity,
            expected,
            eph,
            hello,
            peer_hello: None,
            send_key: [0; 32],
            recv_key: [0; 32],
            decider: false,
            send_seq: 0,
            last_ctl: None,
            win_top: None,
            win_bits: 0,
        })
    }

    pub fn hello(&self) -> &[u8; HELLO_SIZE] {
        &self.hello
    }
    pub fn identity(&self) -> &Identity {
        &self.identity
    }
    pub fn own_ephemeral(&self) -> &Bytes32 {
        self.eph.public()
    }
    pub fn ready(&self) -> bool {
        self.peer_hello.is_some()
    }
    /// The side whose ephemeral key sorts first chooses the connection and sends VERIFY first.
    pub fn decider(&self) -> bool {
        self.decider
    }
    pub fn peer_identity(&self) -> Option<Bytes32> {
        self.peer_hello.map(|h| h[8..40].try_into().unwrap())
    }
    pub fn peer_ephemeral(&self) -> Option<Bytes32> {
        self.peer_hello.map(|h| h[40..72].try_into().unwrap())
    }

    /// The receiver's checks of a HELLO, in the protocol's order.
    pub fn accept_hello(&mut self, d: &[u8]) -> Result<Accepted, SecError> {
        if d.len() < 6 || &d[..4] != HELLO_MAGIC {
            return Err(SecError::Malformed);
        }
        if u16::from_le_bytes([d[4], d[5]]) != PROTOCOL_VERSION {
            return Err(SecError::Version);
        }
        if d.len() != HELLO_SIZE || d[6] != 0 || d[7] != 0 {
            return Err(SecError::Malformed);
        }
        if let Some(h) = &self.peer_hello {
            return if h[..] == d[..] { Ok(Accepted::Same) } else { Err(SecError::Changed) };
        }
        let identity: Bytes32 = d[8..40].try_into().unwrap();
        let eph: Bytes32 = d[40..72].try_into().unwrap();
        if &eph == self.eph.public() {
            return Err(SecError::Reflected);
        }
        if self.expected.is_some_and(|e| e != identity) {
            return Err(SecError::Unexpected);
        }
        let sig: Bytes64 = d[96..].try_into().unwrap();
        if !verify(&identity, &[D_HELLO, &d[..96]], &sig) {
            return Err(SecError::Signature);
        }
        let own = *self.eph.public();
        let (e1, e2, h1, h2) = if own < eph { (own, eph, &self.hello[..], d) } else { (eph, own, d, &self.hello[..]) };
        let master = self.eph.agree(&eph, |shared| hash32(&[D_SESSION, shared, &e1, &e2])).ok_or(SecError::KeyAgreement)?;
        let transcript = hash32(&[D_TRANSCRIPT, h1, h2]);
        self.send_key = keyed32(&master, &[D_DIRECTION, &transcript, &own]);
        self.recv_key = keyed32(&master, &[D_DIRECTION, &transcript, &eph]);
        self.decider = own < eph;
        self.peer_hello = Some(d.try_into().unwrap());
        Ok(Accepted::New)
    }

    fn nonce(seq: u64, channel: u8) -> [u8; 24] {
        let mut n = [0u8; 24];
        n[..8].copy_from_slice(&seq.to_le_bytes());
        n[8] = channel;
        n
    }

    /// A protected packet carrying `plain` on `channel`. None before the handshake or for an oversized message.
    pub fn seal(&mut self, channel: u8, plain: &[u8]) -> Option<Vec<u8>> {
        if !self.ready() || plain.len() > super::wire::MAX_PLAIN || channel > 1 {
            return None;
        }
        let seq = self.send_seq;
        self.send_seq += 1;
        let mut out = Vec::with_capacity(PACKET_HEADER + plain.len() + TAG);
        out.push(PACKET_VERSION);
        out.push(channel);
        out.extend_from_slice(&seq.to_le_bytes());
        let ad = [b'M', b'U', b'N', b'1', PACKET_VERSION, channel];
        xseal(&self.send_key, &Self::nonce(seq, channel), &ad, plain, &mut out);
        Some(out)
    }

    /// The receiver's checks of a protected packet that arrived on `channel`, in the protocol's order; the replay
    /// state moves only once the tag verified.
    pub fn open(&mut self, channel: u8, p: &[u8]) -> Result<Vec<u8>, SecError> {
        if p.len() < PACKET_HEADER + TAG || p.len() > MAX_PACKET || p[0] != PACKET_VERSION || p[1] != channel {
            return Err(SecError::Malformed);
        }
        if !self.ready() {
            return Err(SecError::NoHandshake);
        }
        let seq = u64::from_le_bytes(p[2..10].try_into().unwrap());
        if !self.replay_ok(channel, seq) {
            return Err(SecError::Replay);
        }
        let ad = [b'M', b'U', b'N', b'1', PACKET_VERSION, channel];
        let plain = xopen(&self.recv_key, &Self::nonce(seq, channel), &ad, &p[PACKET_HEADER..]).ok_or(SecError::Tag)?;
        self.replay_note(channel, seq);
        Ok(plain)
    }

    fn replay_ok(&self, channel: u8, seq: u64) -> bool {
        if channel == 0 {
            return self.last_ctl.is_none_or(|l| seq > l);
        }
        match self.win_top {
            None => true,
            Some(top) if seq > top => true,
            Some(top) => top - seq < REPLAY_WINDOW && self.win_bits & (1u64 << (top - seq)) == 0,
        }
    }

    fn replay_note(&mut self, channel: u8, seq: u64) {
        if channel == 0 {
            self.last_ctl = Some(seq);
            return;
        }
        match self.win_top {
            Some(top) if seq <= top => self.win_bits |= 1u64 << (top - seq),
            Some(top) => {
                let shift = seq - top;
                self.win_bits = if shift >= 64 { 0 } else { self.win_bits << shift };
                self.win_bits |= 1;
                self.win_top = Some(seq);
            }
            None => {
                self.win_top = Some(seq);
                self.win_bits = 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pair() -> (Security, Security) {
        let a = Identity::from_seed(&[1; 32]);
        let b = Identity::from_seed(&[2; 32]);
        let (pa, pb) = (a.public_key(), b.public_key());
        (Security::begin(a, Some(pb)).unwrap(), Security::begin(b, Some(pa)).unwrap())
    }

    #[test]
    fn handshake_agrees_on_keys_and_one_decider() {
        let (mut a, mut b) = pair();
        let (ha, hb) = (*a.hello(), *b.hello());
        assert_eq!(&ha[..8], b"MUNH\x01\x00\x00\x00");
        assert_eq!(a.accept_hello(&hb), Ok(Accepted::New));
        assert_eq!(b.accept_hello(&ha), Ok(Accepted::New));
        assert_ne!(a.decider(), b.decider());
        assert_eq!(a.accept_hello(&hb), Ok(Accepted::Same));
        let p = a.seal(1, b"commit").unwrap();
        assert_eq!(p.len(), 10 + 6 + 16);
        assert_eq!(&p[..10], &[1, 1, 0, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(b.open(1, &p).unwrap(), b"commit");
        let q = b.seal(0, b"verify").unwrap();
        assert_eq!(a.open(0, &q).unwrap(), b"verify");
        // the two directions use different keys: a packet does not open on the side that sealed it
        let r = a.seal(0, b"x").unwrap();
        assert_eq!(a.open(0, &r), Err(SecError::Tag));
        assert_eq!(b.open(1, &r), Err(SecError::Malformed), "the channel byte must match the arrival channel");
    }

    #[test]
    fn hello_refusals_in_order() {
        let (mut a, b) = pair();
        let hb = *b.hello();
        let mut v = hb;
        v[4] = 2;
        assert_eq!(a.accept_hello(&v), Err(SecError::Version));
        let mut f = hb;
        f[6] = 1;
        assert_eq!(a.accept_hello(&f), Err(SecError::Malformed));
        assert_eq!(a.accept_hello(&hb[..159]), Err(SecError::Malformed));
        let own = *a.hello();
        assert_eq!(a.accept_hello(&own), Err(SecError::Reflected));
        let mut s = hb;
        s[80] ^= 1;
        assert_eq!(a.accept_hello(&s), Err(SecError::Signature));
        let c = Security::begin(Identity::from_seed(&[3; 32]), None).unwrap();
        assert_eq!(a.accept_hello(c.hello()), Err(SecError::Unexpected));
        assert_eq!(a.accept_hello(&hb), Ok(Accepted::New));
        assert_eq!(a.accept_hello(c.hello()), Err(SecError::Changed));
    }

    #[test]
    fn replay_rules() {
        let (mut a, mut b) = pair();
        let (ha, hb) = (*a.hello(), *b.hello());
        a.accept_hello(&hb).unwrap();
        b.accept_hello(&ha).unwrap();
        // control: strictly increasing
        let c0 = a.seal(0, b"0").unwrap();
        let c1 = a.seal(0, b"1").unwrap();
        assert!(b.open(0, &c1).is_ok());
        assert_eq!(b.open(0, &c0), Err(SecError::Replay));
        // input: a window of 64, each sequence once; the sequence counter is shared with channel 0 (now 2)
        let ins: Vec<Vec<u8>> = (0..80).map(|_| a.seal(1, b"i").unwrap()).collect();
        assert!(b.open(1, &ins[70]).is_ok());
        assert!(b.open(1, &ins[10]).is_ok(), "within 63 of the highest");
        assert_eq!(b.open(1, &ins[10]), Err(SecError::Replay), "only once");
        assert_eq!(b.open(1, &ins[6]), Err(SecError::Replay), "older than the window");
        assert!(b.open(1, &ins[7]).is_ok(), "seq 9 is 63 below seq 72");
        assert!(b.open(1, &ins[79]).is_ok());
        // a forged packet does not move the window
        let mut bad = ins[75].clone();
        bad[20] ^= 1;
        assert_eq!(b.open(1, &bad), Err(SecError::Tag));
        assert!(b.open(1, &ins[75]).is_ok());
    }
}
