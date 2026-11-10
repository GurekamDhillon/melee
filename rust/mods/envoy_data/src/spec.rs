// pool.toml -> records, the lint (envoy_lint), and pool_gen.rs. std only: used by build.rs and the crate's tests.
// The parser reads the subset pool.toml uses: `[[crest]]` tables of `key = "string"` / `key = integer` lines and
// `#` comments.

use super::codes::*;

#[derive(Clone, Debug, Default)]
pub struct Rec {
    pub key: String,
    pub name: String,
    pub rule: String,
    pub family: u8,
    pub moment: u8,
    pub effect: u8,
    pub number: i32,
    pub weight: u8,
    pub gives: u8,
    pub reads: u8,
    pub line: usize,
}

fn find(t: &[&str], v: &str) -> Option<u8> {
    t.iter().position(|x| *x == v).map(|i| i as u8)
}

pub fn parse(src: &str) -> Result<Vec<Rec>, String> {
    let mut out: Vec<Rec> = Vec::new();
    let mut seen: Vec<String> = Vec::new();
    for (n, raw) in src.lines().enumerate() {
        let ln = n + 1;
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line == "[[crest]]" {
            out.push(Rec { line: ln, weight: 1, ..Default::default() });
            seen.clear();
            continue;
        }
        let Some(r) = out.last_mut() else { return Err(format!("line {ln}: a field outside a [[crest]] table")) };
        let Some((k, v)) = line.split_once('=') else { return Err(format!("line {ln}: expected `key = value`")) };
        let (k, v) = (k.trim(), v.trim());
        if seen.iter().any(|s| s == k) {
            return Err(format!("line {ln}: field `{k}` given twice"));
        }
        seen.push(k.to_string());
        let s = || -> Result<String, String> {
            let b = v.strip_prefix('"').and_then(|x| x.strip_suffix('"')).ok_or(format!("line {ln}: `{k}` must be a \"string\""))?;
            if b.contains('"') || b.contains('\\') {
                return Err(format!("line {ln}: `{k}`: no quotes or escapes inside strings"));
            }
            Ok(b.to_string())
        };
        let i = || -> Result<i32, String> { v.parse::<i32>().map_err(|_| format!("line {ln}: `{k}` must be an integer")) };
        match k {
            "key" => r.key = s()?,
            "name" => r.name = s()?,
            "rule" => r.rule = s()?,
            "family" => r.family = find(&FAMILIES, &s()?).ok_or(format!("line {ln}: unknown family"))?,
            "moment" => r.moment = find(&MOMENTS, &s()?).ok_or(format!("line {ln}: unknown moment"))?,
            "effect" => r.effect = find(&EFFECTS, &s()?).ok_or(format!("line {ln}: unknown effect"))?,
            "gives" => r.gives = find(&WORDS[1..], &s()?).map(|w| w + 1).ok_or(format!("line {ln}: gives: not one of the four words"))?,
            "reads" => r.reads = find(&WORDS[1..], &s()?).map(|w| w + 1).ok_or(format!("line {ln}: reads: not one of the four words"))?,
            "number" => r.number = i()?,
            "weight" => r.weight = i()?.clamp(0, 255) as u8,
            _ => return Err(format!("line {ln}: unknown field `{k}`")),
        }
    }
    Ok(out)
}

fn count_numbers(s: &str) -> usize {
    let b = s.as_bytes();
    let mut n = 0;
    let mut i = 0;
    while i < b.len() {
        if b[i].is_ascii_digit() {
            n += 1;
            while i < b.len() && (b[i].is_ascii_digit() || (b[i] == b'.' && i + 1 < b.len() && b[i + 1].is_ascii_digit())) {
                i += 1;
            }
        } else {
            i += 1;
        }
    }
    n
}

/// envoy_lint: the readability limits (envoy-redesign.md 3.1). Every problem found, one per line.
pub fn lint(p: &[Rec]) -> Vec<String> {
    let mut e = Vec::new();
    for (i, r) in p.iter().enumerate() {
        let at = format!("crest `{}` (line {})", r.key, r.line);
        if r.key.is_empty() || !r.key.bytes().all(|c| c.is_ascii_lowercase() || c == b'_') {
            e.push(format!("{at}: key must be lower_snake_case"));
        }
        if r.name.is_empty() || r.name.len() > 16 {
            e.push(format!("{at}: name must be 1-16 characters"));
        }
        if p[..i].iter().any(|q| q.key == r.key) {
            e.push(format!("{at}: duplicate key"));
        }
        if p[..i].iter().any(|q| q.name == r.name) {
            e.push(format!("{at}: duplicate name"));
        }
        if p[..i].iter().any(|q| q.rule == r.rule) {
            e.push(format!("{at}: duplicate rule text"));
        }
        let words = r.rule.split_whitespace().count();
        if words == 0 || words > 12 {
            e.push(format!("{at}: rule has {words} words (1-12)"));
        }
        let nums = count_numbers(&r.rule);
        if nums > 2 {
            e.push(format!("{at}: rule has {nums} numbers (at most 2)"));
        }
        if !FAMILY_MOMENTS[r.family as usize].contains(&r.moment) {
            e.push(format!("{at}: moment `{}` is not one family {} keys on", MOMENTS[r.moment as usize], FAMILIES[r.family as usize]));
        }
        if r.family == F_HEAVY {
            e.push(format!("{at}: HEAVY crests need a price (milestone 3)"));
        }
        if r.number <= 0 {
            e.push(format!("{at}: number must be > 0"));
        }
        if !(1..=3).contains(&r.weight) {
            e.push(format!("{at}: weight must be 1-3"));
        }
        for (what, w) in [("gives", r.gives), ("reads", r.reads)] {
            if w != W_NONE && !r.rule.contains(WORDS[w as usize]) {
                e.push(format!("{at}: {what} {} but the rule does not say it", WORDS[w as usize]));
            }
        }
        if r.gives != W_NONE && r.gives == r.reads {
            e.push(format!("{at}: gives and reads the same word"));
        }
        // a word the rule says is a word it gives or reads (one meaning per word, R11)
        for (k, w) in WORDS.iter().enumerate().skip(1) {
            if r.rule.contains(w) && r.gives != k as u8 && r.reads != k as u8 {
                e.push(format!("{at}: the rule says {w} but neither gives nor reads it"));
            }
        }
    }
    for (k, w) in WORDS.iter().enumerate().skip(1) {
        let k = k as u8;
        let readers = p.iter().filter(|r| r.reads == k).count();
        let givers = p.iter().filter(|r| r.gives == k).count();
        if readers > 0 && givers < 2 && !INHERENT.contains(&k) {
            e.push(format!("word {w}: {readers} crest(s) read it but only {givers} give it (at least 2)"));
        }
    }
    for (f, name) in FAMILIES.iter().enumerate() {
        let n = p.iter().filter(|r| r.family == f as u8).count();
        if f as u8 != F_HEAVY && n < 4 {
            e.push(format!("family {name}: {n} crests (at least 4)"));
        }
    }
    if p.len() > 250 {
        e.push("more than 250 crests (ids are u8)".into());
    }
    e
}

/// FNV-1a over every field of every record, in pool order: the pool's identity (sim hash, run record, handshake).
pub fn digest(p: &[Rec]) -> u32 {
    let mut h: u32 = 0x811C_9DC5;
    let mut eat = |b: &[u8]| {
        for &x in b {
            h ^= x as u32;
            h = h.wrapping_mul(0x0100_0193);
        }
    };
    for r in p {
        for s in [&r.key, &r.name, &r.rule] {
            eat(s.as_bytes());
            eat(&[0]);
        }
        eat(&[r.family, r.moment, r.effect, r.weight, r.gives, r.reads]);
        eat(&r.number.to_le_bytes());
    }
    h
}

#[allow(dead_code)]
pub fn generate(p: &[Rec]) -> String {
    let mut s = String::new();
    s.push_str("// @generated by envoy_data/build.rs from pool.toml. Do not edit.\n");
    s.push_str(&format!("pub const POOL_LEN: usize = {};\n", p.len()));
    s.push_str(&format!("pub const POOL_DIGEST: u32 = 0x{:08X};\n", digest(p)));
    s.push_str("pub static POOL: [Crest; POOL_LEN] = [\n");
    for r in p {
        s.push_str(&format!(
            "    Crest {{ key: {:?}, name: {:?}, rule: {:?}, family: {}, moment: {}, effect: {}, number: {}, weight: {}, gives: {}, reads: {} }},\n",
            r.key, r.name, r.rule, r.family, r.moment, r.effect, r.number, r.weight, r.gives, r.reads
        ));
    }
    s.push_str("];\n");
    s
}
