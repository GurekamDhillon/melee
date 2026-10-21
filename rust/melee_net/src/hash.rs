//! Identity hashes the game fills the config with (gw_net_hash64 / gw_net_hash_file).

pub const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
pub const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

/// FNV-1a 64. `seed` 0 means the standard offset basis (as in the C).
pub fn hash64(data: &[u8], seed: u64) -> u64 {
    let mut h = if seed != 0 { seed } else { FNV_OFFSET };
    for &b in data {
        h ^= b as u64;
        h = h.wrapping_mul(FNV_PRIME);
    }
    h
}

/// Hash of a file's first `max_bytes` (0 = whole file); 0 on error, never 0 otherwise.
/// Reads in 64 KiB chunks and chains them, exactly like the C (the seed trick means an
/// intermediate hash of 0 would restart the basis; kept for compatibility).
pub fn hash_file(path: &std::path::Path, max_bytes: usize) -> u64 {
    use std::io::Read;
    let mut f = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return 0,
    };
    let mut buf = vec![0u8; 65536];
    let mut h = FNV_OFFSET;
    let mut total = 0usize;
    loop {
        let mut n = match f.read(&mut buf) {
            Ok(0) | Err(_) => break,
            Ok(n) => n,
        };
        if max_bytes != 0 && total + n > max_bytes {
            n = max_bytes - total;
        }
        h = hash64(&buf[..n], h);
        total += n;
        if max_bytes != 0 && total >= max_bytes {
            break;
        }
    }
    if h == 0 {
        1
    } else {
        h
    }
}
