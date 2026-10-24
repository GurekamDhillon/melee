//! Script manifests (`gs_json_field`, `gs_header_field`, `gs_truthy` in `pc/platform/gw_script.c`).
//!
//! mod.json is read by substring search, not by a JSON parser: the first `"key"` anywhere in the text,
//! a colon, then a string (a backslash drops itself and keeps the next byte) or a bare value that ends
//! at `,` `}` CR LF with trailing spaces trimmed. The behaviour is reproduced exactly, quirks included,
//! because a manifest that loads in the C build must load the same here.

fn is_ws(b: u8) -> bool {
    matches!(b, b' ' | b'\t' | b'\r' | b'\n')
}

/// `gs_json_field(json, key, out, cap)`: `cap` is the C buffer size (the value is cut to `cap - 1` bytes).
pub fn json_field(json: &str, key: &str, cap: usize) -> Option<String> {
    let pat = format!("\"{key}\"");
    let b = json.as_bytes();
    let start = json.find(&pat)? + pat.len();
    let mut p = start;
    while p < b.len() && is_ws(b[p]) {
        p += 1;
    }
    if b.get(p) != Some(&b':') {
        return None;
    }
    p += 1;
    while p < b.len() && is_ws(b[p]) {
        p += 1;
    }
    let mut out = Vec::new();
    let lim = cap.saturating_sub(1);
    if b.get(p) == Some(&b'"') {
        p += 1;
        while p < b.len() && b[p] != b'"' && out.len() < lim {
            if b[p] == 0x5C && p + 1 < b.len() {
                p += 1;
            }
            out.push(b[p]);
            p += 1;
        }
    } else {
        while p < b.len() && !matches!(b[p], b',' | b'}' | b'\r' | b'\n') && out.len() < lim {
            out.push(b[p]);
            p += 1;
        }
        while out.last() == Some(&b' ') {
            out.pop();
        }
    }
    Some(String::from_utf8_lossy(&out).into_owned())
}

/// `gs_truthy`: `true`, `yes` (any case) or `1`.
pub fn truthy(v: &str) -> bool {
    v.eq_ignore_ascii_case("true") || v.eq_ignore_ascii_case("yes") || v == "1"
}

/// `gs_header_field`: a single-file script's manifest lives in its leading `-- @key: value` comments.
/// Scanning stops at the first line that is not a `--` comment.
pub fn header_field(src: &str, key: &str, cap: usize) -> Option<String> {
    let b = src.as_bytes();
    let mut p = 0usize;
    while p + 1 < b.len() && b[p] == b'-' && b[p + 1] == b'-' {
        let e = b[p..].iter().position(|&c| c == b'\n').map(|i| p + i);
        let mut q = p + 2;
        while q < b.len() && (b[q] == b' ' || b[q] == b'\t') {
            q += 1;
        }
        if b.get(q) == Some(&b'@') {
            let k = key.as_bytes();
            let name = b.get(q + 1..q + 1 + k.len());
            if name.is_some_and(|n| n.eq_ignore_ascii_case(k)) && b.get(q + 1 + k.len()) == Some(&b':') {
                q += 2 + k.len();
                while q < b.len() && (b[q] == b' ' || b[q] == b'\t') {
                    q += 1;
                }
                let mut out = Vec::new();
                while q < b.len() && !matches!(b[q], b'\r' | b'\n') && out.len() < cap.saturating_sub(1) {
                    out.push(b[q]);
                    q += 1;
                }
                while out.last() == Some(&b' ') {
                    out.pop();
                }
                return Some(String::from_utf8_lossy(&out).into_owned());
            }
        }
        let Some(e) = e else { break };
        p = e + 1;
        while p < b.len() && (b[p] == b'\r' || b[p] == b'\n') {
            p += 1;
        }
    }
    None
}

/// `atoi`: optional whitespace and sign, then digits; anything else is 0.
pub fn atoi(s: &str) -> i32 {
    let t = s.trim_start_matches([' ', '\t', '\n', '\r', '\x0b', '\x0c']);
    let (neg, t) = match t.as_bytes().first() {
        Some(b'-') => (true, &t[1..]),
        Some(b'+') => (false, &t[1..]),
        _ => (false, t),
    };
    let mut v: i64 = 0;
    for c in t.bytes().take_while(u8::is_ascii_digit) {
        v = v.wrapping_mul(10).wrapping_add((c - b'0') as i64);
    }
    (if neg { -v } else { v }) as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_strings_numbers_bools() {
        let j = "{\n \"id\": \"geno-lab\",\n \"api_version\": 2 ,\n \"gameplay\": true}\n";
        assert_eq!(json_field(j, "id", 64).as_deref(), Some("geno-lab"));
        assert_eq!(json_field(j, "api_version", 64).as_deref(), Some("2"));
        assert_eq!(json_field(j, "gameplay", 64).as_deref(), Some("true"));
        assert_eq!(json_field(j, "missing", 64), None);
    }

    #[test]
    fn json_backslash_and_cap() {
        assert_eq!(json_field(r#"{"n": "a\"b\\c"}"#, "n", 64).as_deref(), Some("a\"b\\c"));
        assert_eq!(json_field(r#"{"n": "abcdefgh"}"#, "n", 4).as_deref(), Some("abc"));
        // only the FIRST occurrence is looked at (strstr): a value equal to the key hides the real key
        assert_eq!(json_field(r#"{"x": "id", "id" : 3}"#, "id", 8), None);
    }

    #[test]
    fn headers() {
        let s = "-- Demo\n-- @name: My Mod \n--@Gameplay:   yes\n-- @rollback_safe: no\nlocal x = 1\n-- @version: 9\n";
        assert_eq!(header_field(s, "name", 64).as_deref(), Some("My Mod"));
        assert_eq!(header_field(s, "gameplay", 64).as_deref(), Some("yes"));
        assert!(truthy("yes") && truthy("TRUE") && truthy("1") && !truthy("no"));
        assert_eq!(header_field(s, "version", 64), None, "scanning stops at the first code line");
    }

    #[test]
    fn atoi_like_c() {
        assert_eq!((atoi("2"), atoi(" -7x"), atoi("zz"), atoi("+3")), (2, -7, 0, 3));
    }
}
