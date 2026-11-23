//! The primitives mu_net is built on: Ed25519 (identity, signatures), X25519 (per-session ephemeral keys),
//! XChaCha20-Poly1305 (every packet after the HELLO), BLAKE2b (`blake2b.rs`), and a system random source.
//! Ed25519, X25519 and the IETF ChaCha20-Poly1305 AEAD come from `ring` (already in the workspace through rustls);
//! HChaCha20, which turns that AEAD into XChaCha20-Poly1305, is ours. Vectors in the tests were made with libsodium.

use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, CHACHA20_POLY1305};
use ring::agreement;
use ring::rand::{SecureRandom, SystemRandom};
use ring::signature::{self, Ed25519KeyPair, KeyPair};

pub type Bytes32 = [u8; 32];
pub type Bytes64 = [u8; 64];

/// Fills `out` from the system's random source.
pub fn random(out: &mut [u8]) -> bool {
    SystemRandom::new().fill(out).is_ok()
}

// ------------------------------------------------------------------------------------------------ Ed25519

/// A persistent peer identity: the Ed25519 key pair of a 32-byte seed.
pub struct Identity {
    seed: Bytes32,
    pair: Ed25519KeyPair,
}

impl Identity {
    pub fn from_seed(seed: &Bytes32) -> Identity {
        // any 32 bytes are a valid seed
        let pair = Ed25519KeyPair::from_seed_unchecked(seed).expect("ed25519 seed");
        Identity { seed: *seed, pair }
    }
    pub fn public_key(&self) -> Bytes32 {
        self.pair.public_key().as_ref().try_into().unwrap()
    }
    pub fn seed(&self) -> &Bytes32 {
        &self.seed
    }
    /// Signs `parts` concatenated.
    pub fn sign(&self, parts: &[&[u8]]) -> Bytes64 {
        let msg = parts.concat();
        self.pair.sign(&msg).as_ref().try_into().unwrap()
    }
}

impl Drop for Identity {
    fn drop(&mut self) {
        self.seed.fill(0);
    }
}

/// Ed25519 verify of `sig` by `public` over `parts` concatenated.
pub fn verify(public: &Bytes32, parts: &[&[u8]], sig: &Bytes64) -> bool {
    let msg = parts.concat();
    signature::UnparsedPublicKey::new(&signature::ED25519, public).verify(&msg, sig).is_ok()
}

// ------------------------------------------------------------------------------------------------ X25519

/// A per-session X25519 key: the secret is used once (`agree`) and then gone.
pub struct Ephemeral {
    secret: Option<agreement::EphemeralPrivateKey>,
    public: Bytes32,
}

impl Ephemeral {
    pub fn generate() -> Option<Ephemeral> {
        let secret = agreement::EphemeralPrivateKey::generate(&agreement::X25519, &SystemRandom::new()).ok()?;
        let public = secret.compute_public_key().ok()?.as_ref().try_into().ok()?;
        Some(Ephemeral { secret: Some(secret), public })
    }
    pub fn public(&self) -> &Bytes32 {
        &self.public
    }
    /// The shared secret with `peer`, handed to `f` (and wiped by ring afterwards). None when the secret was already
    /// used or the result is all zero (a low-order point: ring refuses it, which is the protocol's own rule).
    pub fn agree<R>(&mut self, peer: &Bytes32, f: impl FnOnce(&[u8]) -> R) -> Option<R> {
        let secret = self.secret.take()?;
        agreement::agree_ephemeral(secret, &agreement::UnparsedPublicKey::new(&agreement::X25519, peer), f).ok()
    }
}

// ------------------------------------------------------------------------------------------------ XChaCha20-Poly1305

fn quarter(s: &mut [u32; 16], a: usize, b: usize, c: usize, d: usize) {
    s[a] = s[a].wrapping_add(s[b]);
    s[d] = (s[d] ^ s[a]).rotate_left(16);
    s[c] = s[c].wrapping_add(s[d]);
    s[b] = (s[b] ^ s[c]).rotate_left(12);
    s[a] = s[a].wrapping_add(s[b]);
    s[d] = (s[d] ^ s[a]).rotate_left(8);
    s[c] = s[c].wrapping_add(s[d]);
    s[b] = (s[b] ^ s[c]).rotate_left(7);
}

/// HChaCha20 (draft-irtf-cfrg-xchacha section 2.2): the ChaCha20 core over key and a 16-byte input, without the
/// final addition, words 0..4 and 12..16 out.
pub fn hchacha20(key: &Bytes32, input: &[u8; 16]) -> Bytes32 {
    let w = |b: &[u8], i: usize| u32::from_le_bytes(b[i * 4..i * 4 + 4].try_into().unwrap());
    let mut s = [0u32; 16];
    s[..4].copy_from_slice(&[0x6170_7865, 0x3320_646e, 0x7962_2d32, 0x6b20_6574]);
    for i in 0..8 {
        s[4 + i] = w(key, i);
    }
    for i in 0..4 {
        s[12 + i] = w(input, i);
    }
    for _ in 0..10 {
        quarter(&mut s, 0, 4, 8, 12);
        quarter(&mut s, 1, 5, 9, 13);
        quarter(&mut s, 2, 6, 10, 14);
        quarter(&mut s, 3, 7, 11, 15);
        quarter(&mut s, 0, 5, 10, 15);
        quarter(&mut s, 1, 6, 11, 12);
        quarter(&mut s, 2, 7, 8, 13);
        quarter(&mut s, 3, 4, 9, 14);
    }
    let mut out = [0u8; 32];
    for (i, &k) in [0usize, 1, 2, 3, 12, 13, 14, 15].iter().enumerate() {
        out[i * 4..i * 4 + 4].copy_from_slice(&s[k].to_le_bytes());
    }
    out
}

fn xkey(key: &Bytes32, nonce: &[u8; 24]) -> (LessSafeKey, Nonce) {
    let sub = hchacha20(key, nonce[..16].try_into().unwrap());
    let mut n = [0u8; 12];
    n[4..].copy_from_slice(&nonce[16..]);
    (LessSafeKey::new(UnboundKey::new(&CHACHA20_POLY1305, &sub).unwrap()), Nonce::assume_unique_for_key(n))
}

/// XChaCha20-Poly1305 encrypt: ciphertext followed by the 16-byte tag, appended to `out`.
pub fn xseal(key: &Bytes32, nonce: &[u8; 24], ad: &[u8], plain: &[u8], out: &mut Vec<u8>) {
    let (k, n) = xkey(key, nonce);
    let at = out.len();
    out.extend_from_slice(plain);
    let tag = k.seal_in_place_separate_tag(n, Aad::from(ad), &mut out[at..]).unwrap();
    out.extend_from_slice(tag.as_ref());
}

/// XChaCha20-Poly1305 decrypt of ciphertext + tag. None when the tag does not verify.
pub fn xopen(key: &Bytes32, nonce: &[u8; 24], ad: &[u8], sealed: &[u8]) -> Option<Vec<u8>> {
    if sealed.len() < 16 {
        return None;
    }
    let (k, n) = xkey(key, nonce);
    let mut buf = sealed.to_vec();
    let len = k.open_in_place(n, Aad::from(ad), &mut buf).ok()?.len();
    buf.truncate(len);
    Some(buf)
}

pub fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

pub fn unhex(s: &str, out: &mut [u8]) -> bool {
    if s.len() != out.len() * 2 || !s.is_ascii() {
        return false;
    }
    for (i, o) in out.iter_mut().enumerate() {
        match u8::from_str_radix(&s[i * 2..i * 2 + 2], 16) {
            Ok(v) => *o = v,
            Err(_) => return false,
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mu_net::blake2b::{hash32, keyed32, Blake2b};

    #[test]
    fn blake2b_matches_hashlib() {
        let r768: Vec<u8> = (0..768).map(|i| i as u8).collect();
        let r128: Vec<u8> = (0..128).map(|i| i as u8).collect();
        let r129: Vec<u8> = (0..129).map(|i| i as u8).collect();
        let r200: Vec<u8> = (0..200).map(|i| i as u8).collect();
        let key: Vec<u8> = (0..32).collect();
        assert_eq!(hex(&hash32(&[])), "0e5751c026e543b2e8ab2eb06099daa1d1e5df47778f7787faab45cdf12fe3a8");
        assert_eq!(hex(&hash32(&[b"a", b"bc"])), "bddd813c634239723171ef3fee98579b94964e3bb1cb3e427262c8c068d52319");
        assert_eq!(hex(&hash32(&[&r768])), "b8007121274217790e2923e0ad7027986e5a99d5531ef6ae7d294140fc81615d");
        assert_eq!(hex(&hash32(&[&r128])), "c3582f71ebb2be66fa5dd750f80baae97554f3b015663c8be377cfcb2488c1d1");
        assert_eq!(hex(&hash32(&[&r129[..100], &r129[100..]])), "f7f3c46ba2564ff4c4c162da1f5b605f9f1c4aa6a20652a9f9a337c1a2f5b9c9");
        assert_eq!(hex(&keyed32(&key, &[])), "4e51e7a913fc80137da52880fecca175bf81e117d5c68126dc2774033517ea0d");
        assert_eq!(hex(&keyed32(&key, &[b"abc"])), "d63a32d3e44738d7907f964316c241adaba0abfeabc32349677578a15a203f7f");
        assert_eq!(hex(&keyed32(&key, &[&r128])), "138893f1631ef3165629515d6ed800da3771b7926dced294205c7507351deebc");
        assert_eq!(hex(&keyed32(&key, &[&r200])), "a39ff6e7e838226fd50e24a55375ff3a39419fd93e32cc463f3f74323291c425");
        let mut s = Blake2b::new(64);
        s.update(b"abc");
        assert_eq!(
            hex(&s.finalize()),
            "ba80a53f981c4d0d6a2797b69f12f6e94c212f14685ac4b74b12bb6fdbffa2d17d87c5392aab792dc252d5de4533cc9518d38aa8dbf1925ab92386edd4009923"
        );
    }

    #[test]
    fn hchacha_and_xaead_match_libsodium() {
        let key: Bytes32 = core::array::from_fn(|i| i as u8);
        let inp: [u8; 16] = core::array::from_fn(|i| 0x40 + i as u8);
        assert_eq!(hex(&hchacha20(&key, &inp)), "001b38f1bc654a0470f0172049103eccb67d8bb16b11d2a468db66a2dd53d47d");
        let mut nonce = [0u8; 24];
        nonce[0] = 7;
        nonce[8] = 1;
        let ad = b"MUN1\x01\x01";
        let mut out = Vec::new();
        xseal(&key, &nonce, ad, b"hello mu_net commit", &mut out);
        assert_eq!(hex(&out), "527e66a55077c42306ec6a570bb76c51f6c064098de7a47c0f9fae06e8ea1faef92abb");
        assert_eq!(xopen(&key, &nonce, ad, &out).as_deref(), Some(&b"hello mu_net commit"[..]));
        let mut bad = out.clone();
        bad[3] ^= 1;
        assert!(xopen(&key, &nonce, ad, &bad).is_none());
        assert!(xopen(&key, &nonce, b"MUN1\x01\x00", &out).is_none());
    }

    #[test]
    fn ed25519_matches_libsodium() {
        let id = Identity::from_seed(&[0x42; 32]);
        assert_eq!(hex(&id.public_key()), "2152f8d19b791d24453242e15f2eab6cb7cffa7b6a5ed30097960e069881db12");
        let sig = id.sign(&[b"MeleeUnlockedNet1 ", b"hello"]);
        assert_eq!(
            hex(&sig),
            "a84473f8d9216cd9ac04894eeb992ed9deb517d78d41f24120c54bc1c89bfb48f89667b22ec84b95d366b601efe543e8a0c37738f99a1d21f57154638bb7940f"
        );
        assert!(verify(&id.public_key(), &[b"MeleeUnlockedNet1 hello"], &sig));
        assert!(!verify(&id.public_key(), &[b"MeleeUnlockedNet1 hellO"], &sig));
        let mut k = [0u8; 2];
        assert!(unhex("0aFf", &mut k) && k == [0x0a, 0xff]);
        assert!(!unhex("0g00", &mut k));
    }

    #[test]
    fn x25519_agrees_and_refuses_zero() {
        let mut a = Ephemeral::generate().unwrap();
        let mut b = Ephemeral::generate().unwrap();
        let (pa, pb) = (*a.public(), *b.public());
        let sa = a.agree(&pb, |s| s.to_vec()).unwrap();
        let sb = b.agree(&pa, |s| s.to_vec()).unwrap();
        assert_eq!(sa, sb);
        assert!(a.agree(&pb, |_| ()).is_none(), "a secret is used once");
        let mut c = Ephemeral::generate().unwrap();
        assert!(c.agree(&[0u8; 32], |_| ()).is_none(), "a low-order point gives an all-zero secret");
    }
}
