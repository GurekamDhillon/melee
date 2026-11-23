//! BLAKE2b (RFC 7693), plain and keyed, any output length 1..=64. mu_net uses it with a 32-byte output for every
//! hash and key derivation. Our own implementation from the RFC; tested against vectors from Python's hashlib.

const IV: [u64; 8] = [
    0x6a09_e667_f3bc_c908,
    0xbb67_ae85_84ca_a73b,
    0x3c6e_f372_fe94_f82b,
    0xa54f_f53a_5f1d_36f1,
    0x510e_527f_ade6_82d1,
    0x9b05_688c_2b3e_6c1f,
    0x1f83_d9ab_fb41_bd6b,
    0x5be0_cd19_137e_2179,
];

const SIGMA: [[usize; 16]; 10] = [
    [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
    [14, 10, 4, 8, 9, 15, 13, 6, 1, 12, 0, 2, 11, 7, 5, 3],
    [11, 8, 12, 0, 5, 2, 15, 13, 10, 14, 3, 6, 7, 1, 9, 4],
    [7, 9, 3, 1, 13, 12, 11, 14, 2, 6, 5, 10, 4, 0, 15, 8],
    [9, 0, 5, 7, 2, 4, 10, 15, 14, 1, 11, 12, 6, 8, 3, 13],
    [2, 12, 6, 10, 0, 11, 8, 3, 4, 13, 7, 5, 15, 14, 1, 9],
    [12, 5, 1, 15, 14, 13, 4, 10, 0, 7, 6, 3, 9, 2, 8, 11],
    [13, 11, 7, 14, 12, 1, 3, 9, 5, 0, 15, 4, 8, 6, 2, 10],
    [6, 15, 14, 9, 11, 3, 0, 8, 12, 2, 13, 7, 1, 4, 10, 5],
    [10, 2, 8, 4, 7, 6, 1, 5, 15, 11, 9, 14, 3, 12, 13, 0],
];

const BLOCK: usize = 128;

/// A running BLAKE2b state (`update` any number of times, then `finalize`).
#[derive(Clone)]
pub struct Blake2b {
    h: [u64; 8],
    t: u128,
    buf: [u8; BLOCK],
    len: usize,
    out_len: usize,
}

impl Blake2b {
    /// Unkeyed, `out_len` bytes of output (1..=64).
    pub fn new(out_len: usize) -> Self {
        Self::keyed(out_len, &[])
    }

    /// Keyed with `key` (0..=64 bytes; empty = unkeyed).
    pub fn keyed(out_len: usize, key: &[u8]) -> Self {
        assert!((1..=64).contains(&out_len) && key.len() <= 64);
        let mut h = IV;
        h[0] ^= 0x0101_0000 ^ ((key.len() as u64) << 8) ^ out_len as u64;
        let mut s = Blake2b { h, t: 0, buf: [0; BLOCK], len: 0, out_len };
        if !key.is_empty() {
            s.buf[..key.len()].copy_from_slice(key);
            s.len = BLOCK;
        }
        s
    }

    pub fn update(&mut self, mut data: &[u8]) -> &mut Self {
        while !data.is_empty() {
            if self.len == BLOCK {
                // a full buffer is only compressed once more input shows it is not the last block
                self.t += BLOCK as u128;
                let b = self.buf;
                self.compress(&b, false);
                self.len = 0;
            }
            let n = (BLOCK - self.len).min(data.len());
            self.buf[self.len..self.len + n].copy_from_slice(&data[..n]);
            self.len += n;
            data = &data[n..];
        }
        self
    }

    pub fn finalize(mut self) -> Vec<u8> {
        self.t += self.len as u128;
        let mut b = self.buf;
        b[self.len..].fill(0);
        self.compress(&b, true);
        let mut out = Vec::with_capacity(64);
        for w in self.h {
            out.extend_from_slice(&w.to_le_bytes());
        }
        out.truncate(self.out_len);
        out
    }

    fn compress(&mut self, block: &[u8; BLOCK], last: bool) {
        let mut m = [0u64; 16];
        for (i, w) in m.iter_mut().enumerate() {
            *w = u64::from_le_bytes(block[i * 8..i * 8 + 8].try_into().unwrap());
        }
        let mut v = [0u64; 16];
        v[..8].copy_from_slice(&self.h);
        v[8..].copy_from_slice(&IV);
        v[12] ^= self.t as u64;
        v[13] ^= (self.t >> 64) as u64;
        if last {
            v[14] = !v[14];
        }
        for r in 0..12 {
            let s = &SIGMA[r % 10];
            g(&mut v, 0, 4, 8, 12, m[s[0]], m[s[1]]);
            g(&mut v, 1, 5, 9, 13, m[s[2]], m[s[3]]);
            g(&mut v, 2, 6, 10, 14, m[s[4]], m[s[5]]);
            g(&mut v, 3, 7, 11, 15, m[s[6]], m[s[7]]);
            g(&mut v, 0, 5, 10, 15, m[s[8]], m[s[9]]);
            g(&mut v, 1, 6, 11, 12, m[s[10]], m[s[11]]);
            g(&mut v, 2, 7, 8, 13, m[s[12]], m[s[13]]);
            g(&mut v, 3, 4, 9, 14, m[s[14]], m[s[15]]);
        }
        for i in 0..8 {
            self.h[i] ^= v[i] ^ v[i + 8];
        }
    }
}

#[inline(always)]
fn g(v: &mut [u64; 16], a: usize, b: usize, c: usize, d: usize, x: u64, y: u64) {
    v[a] = v[a].wrapping_add(v[b]).wrapping_add(x);
    v[d] = (v[d] ^ v[a]).rotate_right(32);
    v[c] = v[c].wrapping_add(v[d]);
    v[b] = (v[b] ^ v[c]).rotate_right(24);
    v[a] = v[a].wrapping_add(v[b]).wrapping_add(y);
    v[d] = (v[d] ^ v[a]).rotate_right(16);
    v[c] = v[c].wrapping_add(v[d]);
    v[b] = (v[b] ^ v[c]).rotate_right(63);
}

/// BLAKE2b-256 over the concatenation of `parts`.
pub fn hash32(parts: &[&[u8]]) -> [u8; 32] {
    keyed32(&[], parts)
}

/// Keyed BLAKE2b-256 over the concatenation of `parts`.
pub fn keyed32(key: &[u8], parts: &[&[u8]]) -> [u8; 32] {
    let mut s = Blake2b::keyed(32, key);
    for p in parts {
        s.update(p);
    }
    s.finalize().try_into().unwrap()
}
