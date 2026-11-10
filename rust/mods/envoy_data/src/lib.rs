//! The Envoy crest pool (full-game-plan.md 4.2, envoy-redesign.md 2-3): `pool.toml` linted and generated at build
//! time into `POOL` (one record per crest), `POOL_DIGEST` (FNV-1a over every field: the sim hash, the run record) and
//! the vocabulary in `codes`. One source, two consumers (envoy_sim, envoy_ui), no runtime parsing.
#![cfg_attr(not(test), no_std)]

mod codes;
pub use codes::*;

/// One crest: one rule, one number, worn by one fighter, firing at a moment.
#[derive(Clone, Copy)]
pub struct Crest {
    pub key: &'static str,
    pub name: &'static str,
    pub rule: &'static str,
    pub family: u8,
    pub moment: u8,
    pub effect: u8,
    /// x1000: a ratio step (attack +200 = +20%), a percent (12000 = 12%) or a time (6000 = 6 s).
    pub number: i32,
    pub weight: u8,
    pub gives: u8,
    pub reads: u8,
}

include!(concat!(env!("OUT_DIR"), "/pool_gen.rs"));

/// The index of the crest with `key` (tests, knobs).
pub fn by_key(key: &str) -> Option<usize> {
    POOL.iter().position(|c| c.key == key)
}

#[cfg(test)]
#[path = "spec.rs"]
mod spec;

#[cfg(test)]
mod tests {
    use super::*;

    fn rec(key: &str, fam: u8, mom: u8, rule: &str) -> spec::Rec {
        spec::Rec { key: key.into(), name: key.into(), rule: rule.into(), family: fam, moment: mom, number: 1, weight: 1, ..Default::default() }
    }

    #[test]
    fn codes_agree() {
        assert_eq!(FAMILIES[F_OPENER as usize], "OPENER");
        assert_eq!(MOMENTS[M_BOSS as usize], "boss");
        assert_eq!(EFFECTS[E_FIRST_SHIFT as usize], "first_shift");
        assert_eq!(WORDS[W_BURNING as usize], "Burning");
    }

    #[test]
    fn shipped_pool_is_clean() {
        let p = spec::parse(include_str!("../pool.toml")).unwrap();
        assert_eq!(spec::lint(&p), Vec::<String>::new());
        assert_eq!(p.len(), POOL_LEN);
        assert_eq!(spec::digest(&p), POOL_DIGEST);
        assert!(by_key("power").is_some());
    }

    #[test]
    fn lint_catches() {
        let mut base: Vec<spec::Rec> = Vec::new();
        for (f, m) in [(F_ALWAYS, M_ALWAYS), (F_RELAY, M_TAG_IN), (F_BENCH, M_TAG_OUT), (F_SHIFT, M_SHIFT), (F_POOL, M_KO_DEALT), (F_OPENER, M_OPENER)] {
            for i in 0..4 {
                base.push(rec(&format!("{}_{}", FAMILIES[f as usize].to_lowercase(), ["a", "b", "c", "d"][i]), f, m, &format!("rule {} {}", f, ["a", "b", "c", "d"][i])));
            }
        }
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
        let mut mom = base.clone();
        mom[0].moment = M_SHIFT;
        assert!(spec::lint(&mom).iter().any(|e| e.contains("not one family")));
        let mut fam = base.clone();
        fam.pop();
        assert!(spec::lint(&fam).iter().any(|e| e.contains("family OPENER")));
        let mut word = base.clone();
        word[4].reads = W_MARKED;
        word[4].rule = "reads Marked".into();
        assert!(spec::lint(&word).iter().any(|e| e.contains("word Marked")));
        word[5].gives = W_MARKED;
        word[5].rule = "gives Marked".into();
        word[6].gives = W_MARKED;
        word[6].rule = "also gives Marked".into();
        assert_eq!(spec::lint(&word), Vec::<String>::new());
        let mut said = base.clone();
        said[0].rule = "says Burning".into();
        assert!(spec::lint(&said).iter().any(|e| e.contains("neither gives nor reads")));
        assert!(spec::parse("[[crest]]\nfamily = \"NOPE\"\n").is_err());
    }
}
