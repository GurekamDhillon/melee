//! The Envoy drive pool (envoy-core-v2.md 2, 4): `pool.toml` linted and generated at build time into `POOL` (one
//! record per drive or keystone), `POOL_DIGEST` (FNV-1a over every field: the sim hash, the run record) and the
//! vocabulary in `codes`. One source, two consumers (envoy_sim, envoy_ui), no runtime parsing.
#![cfg_attr(not(test), no_std)]

mod codes;
pub use codes::*;

/// One drive (one rule, one charge source, one effect, one number) or one keystone (one rule, one price).
#[derive(Clone, Copy)]
pub struct Drive {
    pub kind: u8,
    pub key: &'static str,
    pub name: &'static str,
    pub rule: &'static str,
    pub family: u8,
    pub charge: u8,
    /// Events to fill the charge, or milliseconds for `time`.
    pub need: i32,
    pub effect: u8,
    /// x1000: a percent (6000 = 6%) or a time (4000 = 4 s); keystones: their own unit.
    pub number: i32,
    pub gives: u8,
    pub reads: u8,
    pub price: u8,
    /// The presentation's sound key (never read by the sim).
    pub sound: &'static str,
}

include!(concat!(env!("OUT_DIR"), "/pool_gen.rs"));

/// The index of the record with `key` (tests, knobs).
pub fn by_key(key: &str) -> Option<usize> {
    POOL.iter().position(|c| c.key == key)
}

/// A drive's number at `level` (1-3).
pub fn scaled(number: i32, level: i32) -> i32 {
    number * LEVEL_SCALE[(level.clamp(1, MAX_LEVEL) - 1) as usize] / 1000
}

#[cfg(test)]
#[path = "spec.rs"]
mod spec;

#[cfg(test)]
mod tests {
    use super::*;

    fn rec(key: &str, fam: u8, rule: &str) -> spec::Rec {
        spec::Rec { kind: K_DRIVE, key: key.into(), name: key.into(), rule: rule.into(), family: fam, charge: C_HIT, need: 4, number: 1, sound: "s".into(), ..Default::default() }
    }

    #[test]
    fn codes_agree() {
        assert_eq!(FAMILIES[F_WHITE as usize], "WHITE");
        assert_eq!(CHARGES[C_TAG_IN as usize], "tag_in");
        assert_eq!(EFFECTS[E_OVERCLOCK as usize], "overclock");
        assert_eq!(EFFECTS[E_KEYSTONE0 as usize], "plague");
        assert_eq!(WORDS[W_HASTE as usize], "Haste");
        assert_eq!(PRICES[P_BLEED as usize], "Bleed");
        assert_eq!(PRICE_TEXT.len(), PRICES.len());
    }

    #[test]
    fn shipped_pool_is_clean() {
        let p = spec::parse(include_str!("../pool.toml")).unwrap();
        assert_eq!(spec::lint(&p), Vec::<String>::new());
        assert_eq!(p.len(), POOL_LEN);
        assert_eq!(spec::digest(&p), POOL_DIGEST);
        // the V1 slice: 9 drives, 3 keystones
        assert_eq!(POOL.iter().filter(|d| d.kind == K_DRIVE).count(), 9);
        assert_eq!(POOL.iter().filter(|d| d.kind == K_KEYSTONE).count(), 3);
        for k in ["ember", "nova", "bolt", "rush", "bulwark", "comet", "frost", "shatter", "relay", "plague_bearer", "echo_oath", "overclock"] {
            assert!(by_key(k).is_some(), "{k}");
        }
        assert_eq!(scaled(6000, 1), 6000);
        assert_eq!(scaled(6000, 3), 14400);
    }

    #[test]
    fn lint_catches() {
        let base: Vec<spec::Rec> = (0..6).map(|f| rec(&format!("d_{}", FAMILIES[f].to_lowercase()), f as u8, &format!("rule {f}"))).collect();
        assert_eq!(spec::lint(&base), Vec::<String>::new());
        let mut long = base.clone();
        long[0].rule = "one two three four five six seven eight nine ten eleven twelve thirteen".into();
        assert!(spec::lint(&long).iter().any(|e| e.contains("words")));
        let mut nums = base.clone();
        nums[0].rule = "deal 1 2 3".into();
        assert!(spec::lint(&nums).iter().any(|e| e.contains("numbers")));
        let mut dup = base.clone();
        dup[1].rule = dup[0].rule.clone();
        assert!(spec::lint(&dup).iter().any(|e| e.contains("duplicate rule")));
        let mut fam = base.clone();
        fam.pop();
        assert!(spec::lint(&fam).iter().any(|e| e.contains("family WHITE")));
        let mut word = base.clone();
        word[4].reads = W_CHILLED;
        word[4].rule = "reads Chilled".into();
        assert!(spec::lint(&word).iter().any(|e| e.contains("word Chilled")));
        word[5].gives = W_CHILLED;
        word[5].rule = "gives Chilled".into();
        assert_eq!(spec::lint(&word), Vec::<String>::new());
        let mut said = base.clone();
        said[0].rule = "says Burning".into();
        assert!(spec::lint(&said).iter().any(|e| e.contains("neither gives nor reads")));
        let mut slow = base.clone();
        slow[0].need = 40;
        assert!(spec::lint(&slow).iter().any(|e| e.contains("fire period")));
        let mut ks = base.clone();
        ks.push(spec::Rec { kind: K_KEYSTONE, key: "k".into(), name: "K".into(), rule: "k rule".into(), effect: E_ECHO, number: 1, sound: "s".into(), ..Default::default() });
        assert!(spec::lint(&ks).iter().any(|e| e.contains("price")));
        ks.last_mut().unwrap().price = P_WEAK;
        assert_eq!(spec::lint(&ks), Vec::<String>::new());
        assert!(spec::parse("[[drive]]\nfamily = \"NOPE\"\n").is_err());
    }
}
