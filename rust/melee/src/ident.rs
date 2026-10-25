//! `gs_identity` for this binary (pc/platform/gw_script.c): what a `.gdlp` must match to play here.
//!
//! id[0] the executable (so a recording made by the C build is refused: it is another build, as
//! the C refuses a recording from another exe), id[1] the disc (first 4 MiB, folded with the size),
//! id[2] the mod set (`gw_Mods_Describe`), id[3] the Geno data (the native sim carries no Geno
//! profiles: the "geno" seed alone).

use melee_data::gdlp::Identity;
use melee_net::hash::hash_file;
use std::path::Path;

pub const GENO_VERSION: u32 = 11;

fn fnv(mut h: u64, d: &[u8]) -> u64 {
    for &b in d {
        h ^= b as u64;
        h = h.wrapping_mul(1099511628211);
    }
    h
}

pub fn identity(exe: &Path, iso: &Path, mods_describe: &str) -> Identity {
    let e = hash_file(exe, 0);
    let mut disc = hash_file(iso, 4 << 20);
    if let Ok(m) = std::fs::metadata(iso) {
        disc ^= m.len().wrapping_mul(1099511628211);
    }
    Identity {
        id: [
            e,
            disc,
            fnv(14695981039346656037, mods_describe.as_bytes()),
            fnv(14695981039346656037, b"geno"),
        ],
        geno_version: GENO_VERSION,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use melee_data::gdlp::check_ident;

    #[test]
    fn identity_changes_with_each_input() {
        let d = std::env::temp_dir().join(format!("ident_{}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        let (a, b) = (d.join("a"), d.join("b"));
        std::fs::write(&a, b"exe").unwrap();
        std::fs::write(&b, b"disc").unwrap();
        let i = identity(&a, &b, "m#0001");
        assert_eq!(i.id, identity(&a, &b, "m#0001").id);
        assert_ne!(i.id[2], identity(&a, &b, "").id[2]);
        std::fs::write(&b, b"disc2").unwrap();
        assert_ne!(i.id[1], identity(&a, &b, "m#0001").id[1]);
        let _ = std::fs::remove_dir_all(&d);
        let _ = check_ident; // refusal texts are melee_data's tests
    }
}
