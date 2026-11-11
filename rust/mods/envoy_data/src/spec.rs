// pool.toml -> records, the lint (envoy_lint), and pool_gen.rs. std only: used by build.rs and the crate's tests.
// The parser reads the subset pool.toml uses: `[[drive]]` / `[[keystone]]` tables of `key = "string"` /
// `key = integer` lines and `#` comments.

use super::codes::*;

#[derive(Clone, Debug, Default)]
pub struct Rec {
    pub kind: u8,
    pub key: String,
    pub name: String,
    pub rule: String,
    pub family: u8,
    pub charge: u8,
    pub need: i32,
    pub effect: u8,
    pub number: i32,
    pub gives: u8,
    pub reads: u8,
    pub price: u8,
    /// Milliseconds after a fire before it charges again (0 none).
    pub cooldown: i32,
    /// The effect shape (LOOKS index + 1; 0 none).
    pub look: u8,
    pub sound: String,
    pub line: usize,
    /// V3 (12.7): trigger | shaper | heat_add | gain_mul | heat_mul | neighbour | scaler | economy (ROLES).
    pub role: u8,
    /// V3: common | rare | epic (TIERS).
    pub tier: u8,
    /// V3: the chain coefficient x1000 (triggers).
    pub coef: i32,
    /// V3: the condition (WHENS).
    pub when: u8,
    /// V3: 1 = the fire launches (a timed drive only).
    pub launch: u8,
}

/// The full pool rule (two givers per read word) is off for the V3 slice (12.6: the slice's lint asks for one);
/// three drives per family still holds.
pub const FULL_POOL: bool = false;

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
        if line == "[[drive]]" || line == "[[keystone]]" {
            let kind = if line == "[[drive]]" { K_DRIVE } else { K_KEYSTONE };
            // a drive with no `charge` is not a trigger: it never fills (passive)
            out.push(Rec { line: ln, kind, charge: C_PASSIVE, ..Default::default() });
            seen.clear();
            continue;
        }
        let Some(r) = out.last_mut() else { return Err(format!("line {ln}: a field outside a [[drive]] / [[keystone]] table")) };
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
            "sound" => r.sound = s()?,
            "family" => r.family = find(&FAMILIES, &s()?).ok_or(format!("line {ln}: unknown family"))?,
            "charge" => r.charge = find(&CHARGES, &s()?).ok_or(format!("line {ln}: unknown charge source"))?,
            "effect" => r.effect = find(&EFFECTS, &s()?).ok_or(format!("line {ln}: unknown effect"))?,
            "price" => r.price = find(&PRICES[1..], &s()?).map(|w| w + 1).ok_or(format!("line {ln}: price: not one of the seven (or own)"))?,
            "gives" => r.gives = find(&WORDS[1..], &s()?).map(|w| w + 1).ok_or(format!("line {ln}: gives: not one of the four words"))?,
            "reads" => r.reads = find(&WORDS[1..], &s()?).map(|w| w + 1).ok_or(format!("line {ln}: reads: not one of the four words"))?,
            "look" => r.look = find(&LOOKS, &s()?).map(|w| w + 1).ok_or(format!("line {ln}: look: not one of LOOKS"))?,
            "cooldown" => r.cooldown = i()?,
            "kind" => r.role = find(&ROLES, &s()?).ok_or(format!("line {ln}: kind: not one of ROLES"))?,
            "tier" => r.tier = find(&TIERS, &s()?).ok_or(format!("line {ln}: tier: common | rare | epic"))?,
            "coef" => r.coef = i()?,
            "when" => r.when = find(&WHENS[1..], &s()?).map(|w| w + 1).ok_or(format!("line {ln}: when: not one of WHENS"))?,
            "launch" => r.launch = (v == "true") as u8,
            "need" => r.need = i()?,
            "number" => r.number = i()?,
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

/// Estimated seconds between fires of a level-1 drive at a CPU-proxy pace (envoy-core-v2.md 3.3: the harness band):
/// a hit landed a second, a hit taken every 3 s, a landing every 2 s, a jump every 1.5 s, a tag every 15 s.
pub fn est_period_ms(r: &Rec) -> i32 {
    // V3: damage sources at a CPU-proxy pace of 8% dealt a second (an aerial's share a third of it; damage taken 3%
    // a second); `need` is percent x1000
    let pct_ms = |pace_milli_per_s: i64| -> i32 { ((r.need as i64) * 1000 / pace_milli_per_s).min(i32::MAX as i64) as i32 };
    match r.charge {
        C_DEALT => return pct_ms(8000),
        C_AERIAL_DEALT => return pct_ms(2700),
        C_TAKEN => return pct_ms(3000),
        C_CHILLED_DEALT | C_OFFSTAGE_DEALT | C_CASHOUT => return 0, // gated on a word or a position: no pace
        _ => {}
    }
    let per_event_ms = match r.charge {
        C_HIT => 1000,
        C_LANDING => 2000,
        C_JUMP => 1500,
        C_TIME => return r.need,
        C_AERIAL_HIT => 2500,
        C_HIT_100 => 6000,
        C_FIRE => 3000,
        C_TAG_IN => 15000,
        _ => return 0, // KO, shift, passive: events, not a pace (exempt from the band)
    };
    // reading a word: only some hits qualify (about one in three)
    let k = if r.reads != W_NONE && r.charge == C_HIT { 3 } else { 1 };
    r.need.saturating_mul(per_event_ms).saturating_mul(k)
}
/// The band one drive's estimated period must sit in (ms): never a pile, never a rarity.
pub const PERIOD_BAND: (i32, i32) = (2000, 16000);

/// envoy_lint: the readability limits and the slice's rules. Every problem found, one per line.
pub fn lint(p: &[Rec]) -> Vec<String> {
    let mut e = Vec::new();
    for (i, r) in p.iter().enumerate() {
        let at = format!("{} `{}` (line {})", KINDS[r.kind as usize], r.key, r.line);
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
        if r.number <= 0 {
            e.push(format!("{at}: number must be > 0"));
        }
        if r.sound.is_empty() {
            e.push(format!("{at}: no sound"));
        }
        if r.kind == K_DRIVE {
            if is_key_effect(r.effect) {
                e.push(format!("{at}: effect `{}` is a keystone rule", EFFECTS[r.effect as usize]));
            }
            let trig = r.role == R_TRIGGER;
            let event = matches!(r.charge, C_TAG_IN | C_SHIFT | C_CASHOUT);
            if trig && r.need <= 0 && !event {
                e.push(format!("{at}: need must be > 0"));
            }
            if trig && (r.charge == C_PASSIVE || COUNT_SOURCES.contains(&r.charge)) {
                e.push(format!("{at}: a trigger's charge `{}` counts hits, landings or jumps, or is none (V3: by damage)", CHARGES[r.charge as usize]));
            }
            if !trig && r.charge != C_PASSIVE {
                e.push(format!("{at}: only a trigger has a charge source"));
            }
            if trig && !(0..=1000).contains(&r.coef) {
                e.push(format!("{at}: a trigger's coef is 0-1000"));
            }
            if r.coef == 1000 && r.launch == 0 {
                e.push(format!("{at}: coef 1000 only on a launching timed drive"));
            }
            if r.launch != 0 && r.charge != C_TIME {
                e.push(format!("{at}: only a timed drive launches"));
            }
            if LAUNCHES.contains(&r.effect) && r.launch == 0 {
                e.push(format!("{at}: effect `{}` launches: set launch = true", EFFECTS[r.effect as usize]));
            }
            if r.role == R_GAIN_MUL && r.tier == T_COMMON {
                e.push(format!("{at}: a gain_mul drive is Rare or Epic"));
            }
            if r.role == R_HEAT_MUL && r.tier != T_EPIC {
                e.push(format!("{at}: a heat_mul drive is Epic"));
            }
            if TECH_WHENS.contains(&r.when) && !p.iter().any(|q| q.kind == K_DRIVE && q.family == r.family && !TECH_WHENS.contains(&q.when) && q.key != r.key) {
                e.push(format!("{at}: a technique drive needs a same-family drive with no technique condition"));
            }
            // 4.3: a fire returns less charge than it cost (damage-charged triggers): coef x damage / need
            if trig && DAMAGE_SOURCES.contains(&r.charge) && r.charge != C_TAKEN && fire_damage_m(r) > 0 {
                let lo = r.coef as i64 * fire_damage_m(r) as i64 / r.need.max(1) as i64; // x1000
                let hi = lo * 2400 / 1000 * 2000 / 1000;
                if lo > 100 || hi > 500 {
                    e.push(format!("{at}: a fire returns {lo}/1000 (level 1 Cold) and {hi}/1000 (level 3 Blazing) of its need: over 100 / 500 (4.3)"));
                }
            }
            if r.price != P_NONE {
                e.push(format!("{at}: a drive has no price"));
            }
            let ms = est_period_ms(r);
            let ms = if ms == 0 { 0 } else { ms.max(r.cooldown) };
            // a drive gated on a word (Shatter, Wildfire) or an event (a KO, a shift, always on) has no pace of its own
            if r.reads == W_NONE && ms != 0 && !(PERIOD_BAND.0..=PERIOD_BAND.1).contains(&ms) {
                e.push(format!("{at}: estimated fire period {ms} ms is outside the band {:?}", PERIOD_BAND));
            }
            if r.look == 0 {
                e.push(format!("{at}: no look"));
            }
            if r.cooldown < 0 {
                e.push(format!("{at}: cooldown must be >= 0"));
            }

        } else {
            if !is_key_effect(r.effect) {
                e.push(format!("{at}: effect `{}` is a drive effect", EFFECTS[r.effect as usize]));
            }
            if r.price == P_NONE {
                e.push(format!("{at}: a keystone needs exactly one price (or own)"));
            }
            if r.look != 0 || r.cooldown != 0 {
                e.push(format!("{at}: a keystone has no look or cooldown (its look is the halo)"));
            }
        }
        for (what, w) in [("gives", r.gives), ("reads", r.reads)] {
            if w != W_NONE && !r.rule.contains(WORDS[w as usize]) {
                e.push(format!("{at}: {what} {} but the rule does not say it", WORDS[w as usize]));
            }
        }
        if r.gives != W_NONE && r.gives == r.reads {
            e.push(format!("{at}: gives and reads the same word"));
        }
        // a word the rule says is a word it gives or reads (one meaning per word)
        for (k, w) in WORDS.iter().enumerate().skip(1) {
            if r.rule.contains(w) && r.gives != k as u8 && r.reads != k as u8 {
                e.push(format!("{at}: the rule says {w} but neither gives nor reads it"));
            }
        }
    }
    let need_givers = if FULL_POOL { 2 } else { 1 };
    for (k, w) in WORDS.iter().enumerate().skip(1) {
        let k = k as u8;
        let readers = p.iter().filter(|r| r.reads == k).count();
        let givers = p.iter().filter(|r| r.gives == k).count();
        if readers > 0 && givers < need_givers {
            e.push(format!("word {w}: {readers} record(s) read it but only {givers} give it (at least {need_givers})"));
        }
    }
    let need_members = 3;
    for (f, name) in FAMILIES.iter().enumerate() {
        let n = p.iter().filter(|r| r.kind == K_DRIVE && r.family == f as u8).count();
        if n < need_members {
            e.push(format!("family {name}: {n} drives (at least {need_members})"));
        }
    }
    if p.len() > 250 {
        e.push("more than 250 records (ids are u8)".into());
    }
    e
}

/// FNV-1a over every field of every record, in pool order: the pool's identity (sim hash, run record, handshake).
/// The damage (percent x1000) one level-1 Cold fire of a trigger deals to its target (0: no damage).
pub fn fire_damage_m(r: &Rec) -> i32 {
    match r.effect {
        E_BURST | E_BOLT | E_CHILL | E_SHATTER | E_MARK | E_WILDFIRE | E_THORNS | E_NOVA | E_SHOCKWAVE | E_SPIRE => r.number,
        E_FEATHERS => r.number * 3,
        _ => 0,
    }
}

pub fn digest(p: &[Rec]) -> u32 {
    let mut h: u32 = 0x811C_9DC5;
    let mut eat = |b: &[u8]| {
        for &x in b {
            h ^= x as u32;
            h = h.wrapping_mul(0x0100_0193);
        }
    };
    for r in p {
        for s in [&r.key, &r.name, &r.rule, &r.sound] {
            eat(s.as_bytes());
            eat(&[0]);
        }
        eat(&[r.kind, r.family, r.charge, r.effect, r.gives, r.reads, r.price, r.look]);
        eat(&r.cooldown.to_le_bytes());
        eat(&r.need.to_le_bytes());
        eat(&r.number.to_le_bytes());
        eat(&[r.role, r.tier, r.when, r.launch]);
        eat(&r.coef.to_le_bytes());
    }
    for c in HEAT_CONSTS {
        eat(&c.to_le_bytes());
    }
    h
}

#[allow(dead_code)]
pub fn generate(p: &[Rec]) -> String {
    let mut s = String::new();
    s.push_str("// @generated by envoy_data/build.rs from pool.toml. Do not edit.\n");
    s.push_str(&format!("pub const POOL_LEN: usize = {};\n", p.len()));
    s.push_str(&format!("pub const POOL_DIGEST: u32 = 0x{:08X};\n", digest(p)));
    s.push_str("pub static POOL: [Drive; POOL_LEN] = [\n");
    for r in p {
        s.push_str(&format!(
            "    Drive {{ kind: {}, key: {:?}, name: {:?}, rule: {:?}, family: {}, charge: {}, need: {}, effect: {}, number: {}, gives: {}, reads: {}, price: {}, cooldown: {}, look: {}, sound: {:?}, role: {}, tier: {}, coef: {}, when: {}, launch: {} }},\n",
            r.kind, r.key, r.name, r.rule, r.family, r.charge, r.need, r.effect, r.number, r.gives, r.reads, r.price, r.cooldown, r.look, r.sound, r.role, r.tier, r.coef, r.when, r.launch
        ));
    }
    s.push_str("];\n");
    s
}
