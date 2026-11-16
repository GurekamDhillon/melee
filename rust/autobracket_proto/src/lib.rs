// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) the melee-autobracket authors.
// Vendored from github.com/GurekamDhillon/melee-autobracket crates/proto/src/sign.rs (277d8998). Changes: the nonce
// comes from getrandom (no `rand`), `sign_request_with` takes it as an argument (fixed-vector tests), hex is local;
// the verifying half is kept so tests can check a signature the way the server does.
//! Request signing for the bracket server (docs/api.md §2). Each request carries the headers `x-ab-key`, `x-ab-time`,
//! `x-ab-nonce` and `x-ab-sig`; the signature covers [`canonical`].

pub use ed25519_dalek::{SigningKey, VerifyingKey};
use ed25519_dalek::{Signature, Signer};
use sha2::{Digest, Sha256};

/// Largest accepted difference, in seconds, between the signed time and the verifier's clock.
pub const MAX_SKEW_SECS: i64 = 120;

/// Header values: `key` is the 32-byte verifying key and `sig` the 64-byte signature, `nonce` 16 random bytes, all
/// lowercase hex; `time` is unix seconds in decimal.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SignedHeaders {
    pub key: String,
    pub time: String,
    pub nonce: String,
    pub sig: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SignError {
    Malformed,
    BadSignature,
    Skew,
}

pub fn hex(b: &[u8]) -> String {
    const H: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(b.len() * 2);
    for &x in b {
        s.push(H[(x >> 4) as usize] as char);
        s.push(H[(x & 15) as usize] as char);
    }
    s
}

pub fn unhex<const N: usize>(s: &str) -> Option<[u8; N]> {
    let b = s.as_bytes();
    if b.len() != N * 2 {
        return None;
    }
    let d = |c: u8| (c as char).to_digit(16).map(|v| v as u8);
    let mut o = [0u8; N];
    for i in 0..N {
        o[i] = d(b[2 * i])? << 4 | d(b[2 * i + 1])?;
    }
    Some(o)
}

pub fn sha256(b: &[u8]) -> [u8; 32] {
    Sha256::digest(b).into()
}

pub fn sha256_hex(body: &[u8]) -> String {
    hex(&Sha256::digest(body))
}

/// The signed message: `"{METHOD}\n{path_and_query}\n{time}\n{nonce}\n{sha256_hex(body)}"`.
pub fn canonical(method: &str, path_and_query: &str, time: &str, nonce: &str, body: &[u8]) -> String {
    format!("{method}\n{path_and_query}\n{time}\n{nonce}\n{}", sha256_hex(body))
}

/// 16 fresh random bytes from the OS (a retry is a new request: a new nonce).
pub fn fresh_nonce() -> [u8; 16] {
    let mut n = [0u8; 16];
    getrandom::getrandom(&mut n).expect("the OS random source");
    n
}

pub fn sign_request(sk: &SigningKey, method: &str, path_and_query: &str, now_unix: i64, body: &[u8]) -> SignedHeaders {
    sign_request_with(sk, method, path_and_query, now_unix, &fresh_nonce(), body)
}

pub fn sign_request_with(sk: &SigningKey, method: &str, path_and_query: &str, now_unix: i64, nonce: &[u8; 16], body: &[u8]) -> SignedHeaders {
    let nonce = hex(nonce);
    let time = now_unix.to_string();
    let sig = sk.sign(canonical(method, path_and_query, &time, &nonce, body).as_bytes());
    SignedHeaders { key: hex(&sk.verifying_key().to_bytes()), time, nonce, sig: hex(&sig.to_bytes()) }
}

/// The server's check (`verify_strict`), for tests.
pub fn verify_request(h: &SignedHeaders, method: &str, path_and_query: &str, now_unix: i64, body: &[u8]) -> Result<String, SignError> {
    let kb: [u8; 32] = unhex(&h.key).ok_or(SignError::Malformed)?;
    let sb: [u8; 64] = unhex(&h.sig).ok_or(SignError::Malformed)?;
    let _: [u8; 16] = unhex(&h.nonce).ok_or(SignError::Malformed)?;
    let time: i64 = h.time.parse().map_err(|_| SignError::Malformed)?;
    let key = VerifyingKey::from_bytes(&kb).map_err(|_| SignError::Malformed)?;
    if now_unix.abs_diff(time) > MAX_SKEW_SECS as u64 {
        return Err(SignError::Skew);
    }
    let msg = canonical(method, path_and_query, &h.time, &h.nonce, body);
    key.verify_strict(msg.as_bytes(), &Signature::from_bytes(&sb)).map_err(|_| SignError::BadSignature)?;
    Ok(hex(&kb))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rfc_key() -> SigningKey {
        SigningKey::from_bytes(&unhex("9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60").unwrap())
    }
    const NONCE: [u8; 16] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];

    /// docs/api.md §2's worked example (checked against `proto` by the server repo's tests/api_doc.rs).
    #[test]
    fn api_doc_worked_example() {
        let body = br#"{"tag":"Mango","invite":null}"#;
        assert_eq!(sha256_hex(body), "75b3ccf624edbf047501905d78082bcf831e3d40a78c50112916d6d64db6a91c");
        let h = sign_request_with(&rfc_key(), "POST", "/v1/players", 1763200000, &NONCE, body);
        assert_eq!(h.key, "d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a");
        assert_eq!(h.nonce, "000102030405060708090a0b0c0d0e0f");
        assert_eq!(h.sig, "d5e64034507c78a9938b1fe15b48130f5e3716eadc4eb23f5562f9ea2bafa5e4dd0f4e7ab07296bc552409902e7f4c7debafee185cad08e016c1acb0c1ff5b02");
        assert_eq!(canonical("POST", "/v1/players", "1763200000", &h.nonce, body), "POST\n/v1/players\n1763200000\n000102030405060708090a0b0c0d0e0f\n75b3ccf624edbf047501905d78082bcf831e3d40a78c50112916d6d64db6a91c");
        assert!(verify_request(&h, "POST", "/v1/players", 1763200000, body).is_ok());
    }

    /// A replay upload and a GET with a query, signed by upstream `proto::sign` (crates/proto at 277d8998) with the
    /// same fixed key / time / nonce (the vectors were printed by a scratch program linking that crate).
    #[test]
    fn upstream_vectors() {
        let body: Vec<u8> = (0..=255u8).cycle().take(1000).collect();
        let h = sign_request_with(&rfc_key(), "POST", "/v1/sets/7/3/games/2/replay", 1763200000, &NONCE, &body);
        assert_eq!(h.sig, UPSTREAM_UPLOAD_SIG);
        let g = sign_request_with(&rfc_key(), "GET", "/v1/me/next?event=7", 1763200000, &NONCE, b"");
        assert_eq!(g.sig, UPSTREAM_NEXT_SIG);
        assert_eq!(sha256_hex(b""), "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
    }
    const UPSTREAM_UPLOAD_SIG: &str = "35ba579b81a0b5984fcff16c2917dba5ea5b836566e024d60cb5f0c6229773f7d08f9ce703b91c316e437c7f6f0ee49f229a2fbb8f15276ba2eb11172bc12609";
    const UPSTREAM_NEXT_SIG: &str = "6635926fe7120f208af809da7639d7d3aae0616e9c5c14ecb049c7db8af1dfeabe2e4de020d1595b8cbf21b627b2638251e39eef559e502c7f49d644a34c2708";

    #[test]
    fn tamper_fails_and_nonce_fresh() {
        let h = sign_request(&rfc_key(), "GET", "/v1/me/next?event=1", 1763200000, b"");
        assert_eq!(verify_request(&h, "GET", "/v1/me/next?event=2", 1763200000, b""), Err(SignError::BadSignature));
        assert_eq!(verify_request(&h, "GET", "/v1/me/next?event=1", 1763200000 + 500, b""), Err(SignError::Skew));
        assert_ne!(h.nonce, sign_request(&rfc_key(), "GET", "/", 0, b"").nonce);
    }
}
