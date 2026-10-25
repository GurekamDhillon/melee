//! geno_registry.c's "tiny JSON reader": a DOM over a fixed node pool. Same quirks as the C one (comments,
//! trailing commas, a UTF-8 BOM, `\u` escapes become `?`, strtod number syntax, node/arena/depth limits), because
//! the stable ids hash what this reader accepts.
#![allow(clippy::needless_range_loop)]

pub const JDOC_NODES: usize = 32768;
pub const JDOC_ARENA: usize = 524288;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Ty {
    Null,
    Bool,
    Num,
    Str,
    Arr,
    Obj,
}

#[derive(Clone, Debug)]
pub struct Node {
    pub ty: Ty,
    pub num: f64,
    pub str: Vec<u8>,
    pub key: Vec<u8>,
    pub first: i32,
    pub next: i32,
    pub count: i32,
}

pub struct Doc {
    pub n: Vec<Node>,
    na: usize,
    pub err: &'static str,
    s: Vec<u8>,
    pub p: usize,
}

impl Doc {
    fn ch(&self, i: usize) -> u8 {
        self.s.get(i).copied().unwrap_or(0)
    }

    fn new_node(&mut self, ty: Ty) -> i32 {
        if self.n.len() >= JDOC_NODES {
            self.err = "too many values";
            return -1;
        }
        self.n.push(Node { ty, num: 0.0, str: Vec::new(), key: Vec::new(), first: -1, next: -1, count: 0 });
        (self.n.len() - 1) as i32
    }

    fn ws(&mut self) {
        loop {
            while matches!(self.ch(self.p), b' ' | b'\t' | b'\r' | b'\n') {
                self.p += 1;
            }
            if self.ch(self.p) == b'/' && self.ch(self.p + 1) == b'/' {
                while self.ch(self.p) != 0 && self.ch(self.p) != b'\n' {
                    self.p += 1;
                }
                continue;
            }
            return;
        }
    }

    fn string(&mut self) -> Option<Vec<u8>> {
        if self.ch(self.p) != b'"' {
            self.err = "expected a string";
            return None;
        }
        self.p += 1;
        let mut out = Vec::new();
        while self.ch(self.p) != 0 && self.ch(self.p) != b'"' {
            let mut c = self.ch(self.p);
            self.p += 1;
            if c == b'\\' {
                let e = self.ch(self.p);
                if e == 0 {
                    break; // the C reads past the terminator here; an unterminated string is the honest answer
                }
                self.p += 1;
                c = match e {
                    b'n' => b'\n',
                    b't' => b'\t',
                    b'r' => b'\r',
                    b'u' => b'?',
                    o => o,
                };
                if e == b'u' {
                    for _ in 0..4 {
                        if self.ch(self.p) != 0 {
                            self.p += 1;
                        }
                    }
                }
            }
            if self.na >= JDOC_ARENA - 1 {
                self.err = "strings too long";
                return None;
            }
            out.push(c);
            self.na += 1;
        }
        if self.ch(self.p) != b'"' {
            self.err = "unterminated string";
            return None;
        }
        self.p += 1;
        self.na += 1;
        Some(out)
    }

    fn container(&mut self, depth: i32, is_obj: bool) -> i32 {
        let me = self.new_node(if is_obj { Ty::Obj } else { Ty::Arr });
        let close = if is_obj { b'}' } else { b']' };
        let mut last = -1i32;
        if me < 0 {
            return -1;
        }
        self.p += 1;
        self.ws();
        if self.ch(self.p) == close {
            self.p += 1;
            return me;
        }
        loop {
            let mut key = Vec::new();
            self.ws();
            if is_obj {
                match self.string() {
                    Some(k) => key = k,
                    None => return -1,
                }
                self.ws();
                if self.ch(self.p) != b':' {
                    self.err = "expected ':'";
                    return -1;
                }
                self.p += 1;
            }
            let c = self.value(depth + 1);
            if c < 0 {
                return -1;
            }
            self.n[c as usize].key = key;
            if last < 0 {
                self.n[me as usize].first = c;
            } else {
                self.n[last as usize].next = c;
            }
            last = c;
            self.n[me as usize].count += 1;
            self.ws();
            if self.ch(self.p) == b',' {
                self.p += 1;
                self.ws();
                if self.ch(self.p) == close {
                    self.p += 1;
                    return me;
                }
                continue;
            }
            if self.ch(self.p) == close {
                self.p += 1;
                return me;
            }
            self.err = if is_obj { "expected ',' or '}'" } else { "expected ',' or ']'" };
            return -1;
        }
    }

    fn starts(&self, lit: &[u8]) -> bool {
        self.s.get(self.p..).is_some_and(|r| r.starts_with(lit))
    }

    fn value(&mut self, depth: i32) -> i32 {
        if depth > 32 {
            self.err = "nested too deeply";
            return -1;
        }
        self.ws();
        match self.ch(self.p) {
            b'{' => return self.container(depth, true),
            b'[' => return self.container(depth, false),
            b'"' => {
                let me = self.new_node(Ty::Str);
                if me < 0 {
                    return -1;
                }
                return match self.string() {
                    Some(s) => {
                        self.n[me as usize].str = s;
                        me
                    }
                    None => -1,
                };
            }
            b't' | b'f' | b'n' => {
                if self.starts(b"true") || self.starts(b"false") {
                    let me = self.new_node(Ty::Bool);
                    if me < 0 {
                        return -1;
                    }
                    let t = self.ch(self.p) == b't';
                    self.n[me as usize].num = if t { 1.0 } else { 0.0 };
                    self.p += if t { 4 } else { 5 };
                    return me;
                }
                if self.starts(b"null") {
                    self.p += 4;
                    return self.new_node(Ty::Null);
                }
            }
            c => {
                if c == b'-' || c.is_ascii_digit() {
                    let me = self.new_node(Ty::Num);
                    if me < 0 {
                        return -1;
                    }
                    if let Some((v, used)) = strtod(&self.s[self.p..]) {
                        self.n[me as usize].num = v;
                        self.p += used;
                        return me;
                    }
                }
            }
        }
        self.err = "unexpected character";
        -1
    }

    /// Parse `text`; the root node or -1 (`err` says why).
    pub fn parse(text: &[u8]) -> (Doc, i32) {
        let end = text.iter().position(|&b| b == 0).unwrap_or(text.len());
        let mut d = Doc { n: Vec::new(), na: 0, err: "", s: text[..end].to_vec(), p: 0 };
        if d.s.starts_with(&[0xEF, 0xBB, 0xBF]) {
            d.p = 3;
        }
        let root = d.value(0);
        if root < 0 {
            return (d, -1);
        }
        d.ws();
        if d.ch(d.p) != 0 {
            d.err = "text after the top-level value";
            return (d, -1);
        }
        (d, root)
    }

    pub fn get(&self, obj: i32, key: &str) -> i32 {
        if obj < 0 || self.n[obj as usize].ty != Ty::Obj {
            return -1;
        }
        let mut c = self.n[obj as usize].first;
        while c >= 0 {
            if self.n[c as usize].key == key.as_bytes() {
                return c;
            }
            c = self.n[c as usize].next;
        }
        -1
    }

    /// The children of an array/object node, in file order.
    pub fn kids(&self, x: i32) -> Vec<i32> {
        let mut v = Vec::new();
        let mut c = if x >= 0 { self.n[x as usize].first } else { -1 };
        while c >= 0 {
            v.push(c);
            c = self.n[c as usize].next;
        }
        v
    }

    pub fn ty(&self, x: i32) -> Option<Ty> {
        if x < 0 {
            None
        } else {
            Some(self.n[x as usize].ty)
        }
    }

    /// The string value as text (lossy for invalid UTF-8).
    pub fn text(&self, x: i32) -> String {
        String::from_utf8_lossy(&self.n[x as usize].str).into_owned()
    }

    pub fn is(&self, x: i32, ty: Ty) -> bool {
        x >= 0 && self.n[x as usize].ty == ty
    }

    /// "near" context for error messages: up to 20 bytes at the error position.
    pub fn near(&self) -> String {
        let e = (self.p + 20).min(self.s.len());
        String::from_utf8_lossy(self.s.get(self.p.min(self.s.len())..e).unwrap_or(&[])).into_owned()
    }
}

/// C `strtod` over a prefix that starts with '-' or a digit: decimal, hex integers, inf/infinity/nan.
/// None when nothing converts.
pub fn strtod(s: &[u8]) -> Option<(f64, usize)> {
    let mut i = 0;
    let neg = s.first() == Some(&b'-');
    if neg || s.first() == Some(&b'+') {
        i += 1;
    }
    let rest = &s[i..];
    let lower: Vec<u8> = rest.iter().take(8).map(|b| b.to_ascii_lowercase()).collect();
    if lower.starts_with(b"infinity") {
        return Some((if neg { f64::NEG_INFINITY } else { f64::INFINITY }, i + 8));
    }
    if lower.starts_with(b"inf") {
        return Some((if neg { f64::NEG_INFINITY } else { f64::INFINITY }, i + 3));
    }
    if lower.starts_with(b"nan") {
        return Some((f64::NAN, i + 3));
    }
    if rest.len() > 2 && rest[0] == b'0' && (rest[1] | 0x20) == b'x' && rest[2].is_ascii_hexdigit() {
        let mut j = 2;
        let mut v = 0f64;
        while j < rest.len() && rest[j].is_ascii_hexdigit() {
            v = v * 16.0 + (rest[j] as char).to_digit(16).unwrap() as f64;
            j += 1;
        }
        return Some((if neg { -v } else { v }, i + j));
    }
    let mut j = 0;
    let mut digits = 0;
    while j < rest.len() && rest[j].is_ascii_digit() {
        j += 1;
        digits += 1;
    }
    if j < rest.len() && rest[j] == b'.' {
        let mut k = j + 1;
        let mut frac = 0;
        while k < rest.len() && rest[k].is_ascii_digit() {
            k += 1;
            frac += 1;
        }
        if digits + frac > 0 {
            digits += frac;
            j = k;
        }
    }
    if digits == 0 {
        return None;
    }
    if j < rest.len() && (rest[j] | 0x20) == b'e' {
        let mut k = j + 1;
        if k < rest.len() && (rest[k] == b'+' || rest[k] == b'-') {
            k += 1;
        }
        let st = k;
        while k < rest.len() && rest[k].is_ascii_digit() {
            k += 1;
        }
        if k > st {
            j = k;
        }
    }
    let txt = std::str::from_utf8(&rest[..j]).ok()?;
    let txt = if txt.ends_with('.') { &txt[..txt.len() - 1] } else { txt };
    let v: f64 = txt.parse().ok()?;
    Some((if neg { -v } else { v }, i + j))
}

/// C `%.<prec>g` for finite values (the canonical number form the stable ids hash).
pub fn fmt_g(v: f64, prec: usize) -> String {
    if v.is_nan() {
        return "nan".into();
    }
    if v.is_infinite() {
        return if v < 0.0 { "-inf".into() } else { "inf".into() };
    }
    if v == 0.0 {
        return if v.is_sign_negative() { "-0".into() } else { "0".into() };
    }
    let prec = prec.max(1);
    let e = format!("{:.*e}", prec - 1, v);
    let (mant, exp) = e.split_once('e').unwrap();
    let exp: i32 = exp.parse().unwrap();
    let strip = |s: &str| -> String {
        if s.contains('.') {
            s.trim_end_matches('0').trim_end_matches('.').to_string()
        } else {
            s.to_string()
        }
    };
    if exp < -4 || exp >= prec as i32 {
        format!("{}e{}{:02}", strip(mant), if exp < 0 { '-' } else { '+' }, exp.abs())
    } else {
        strip(&format!("{:.*}", (prec as i32 - 1 - exp) as usize, v))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn g_format() {
        assert_eq!(fmt_g(9.0, 9), "9");
        assert_eq!(fmt_g(0.08, 9), "0.08");
        assert_eq!(fmt_g(1e-5, 9), "1e-05");
        assert_eq!(fmt_g(123456789012.0, 9), "1.23456789e+11");
        assert_eq!(fmt_g(0.1f32 as f64, 9), "0.100000001");
        assert_eq!(fmt_g(-2.5, 9), "-2.5");
        assert_eq!(fmt_g(100000000.0, 9), "100000000");
        assert_eq!(fmt_g(1000000000.0, 9), "1e+09");
    }
    #[test]
    fn strtod_prefixes() {
        assert_eq!(strtod(b"12abc"), Some((12.0, 2)));
        assert_eq!(strtod(b"1e"), Some((1.0, 1)));
        assert_eq!(strtod(b"1.5e+2,"), Some((150.0, 6)));
        assert_eq!(strtod(b"-"), None);
        assert_eq!(strtod(b"0x1F"), Some((31.0, 4)));
        assert_eq!(strtod(b"1."), Some((1.0, 2)));
    }
    #[test]
    fn lax_json() {
        let (d, r) = Doc::parse(b"{\"geno\":1, // c\n\"a\":[1,2,],}");
        assert!(r >= 0, "{}", d.err);
        assert_eq!(d.n[d.get(r, "a") as usize].count, 2);
        let (d, r) = Doc::parse(b"{\"a\":1");
        assert!(r < 0);
        assert_eq!(d.err, "expected ',' or '}'");
    }
}
