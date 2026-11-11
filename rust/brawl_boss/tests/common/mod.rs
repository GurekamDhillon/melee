//! Test helpers: a minimal JSON reader, the locator for the player's local files, and a synthetic file builder.
#![allow(dead_code)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------- JSON

#[derive(Clone, Debug, PartialEq)]
pub enum J {
    Null,
    Bool(bool),
    Num(f64),
    Int(i64),
    Str(String),
    Arr(Vec<J>),
    Obj(BTreeMap<String, J>),
}

impl J {
    pub fn get(&self, k: &str) -> &J {
        match self {
            J::Obj(m) => m.get(k).unwrap_or(&J::Null),
            _ => &J::Null,
        }
    }
    pub fn at(&self, i: usize) -> &J {
        match self {
            J::Arr(a) => a.get(i).unwrap_or(&J::Null),
            _ => &J::Null,
        }
    }
    pub fn arr(&self) -> &[J] {
        match self {
            J::Arr(a) => a,
            _ => &[],
        }
    }
    pub fn obj(&self) -> Option<&BTreeMap<String, J>> {
        match self {
            J::Obj(m) => Some(m),
            _ => None,
        }
    }
    pub fn f(&self) -> f64 {
        match self {
            J::Num(n) => *n,
            J::Int(i) => *i as f64,
            J::Bool(b) => *b as i32 as f64,
            _ => f64::NAN,
        }
    }
    pub fn i(&self) -> i64 {
        match self {
            J::Int(i) => *i,
            _ => self.f() as i64,
        }
    }
    pub fn is_int(&self) -> bool {
        matches!(self, J::Int(_))
    }
    pub fn s(&self) -> &str {
        match self {
            J::Str(s) => s,
            _ => "",
        }
    }
    pub fn is_null(&self) -> bool {
        matches!(self, J::Null)
    }
}

pub fn parse_json(s: &str) -> J {
    let b = s.as_bytes();
    let mut p = 0;
    let v = val(b, &mut p);
    v
}

fn ws(b: &[u8], p: &mut usize) {
    while *p < b.len() && (b[*p] as char).is_ascii_whitespace() {
        *p += 1;
    }
}

fn val(b: &[u8], p: &mut usize) -> J {
    ws(b, p);
    match b[*p] {
        b'{' => {
            *p += 1;
            let mut m = BTreeMap::new();
            loop {
                ws(b, p);
                if b[*p] == b'}' {
                    *p += 1;
                    return J::Obj(m);
                }
                let k = match val(b, p) {
                    J::Str(s) => s,
                    _ => panic!("key"),
                };
                ws(b, p);
                assert_eq!(b[*p], b':');
                *p += 1;
                let v = val(b, p);
                m.insert(k, v);
                ws(b, p);
                if b[*p] == b',' {
                    *p += 1;
                }
            }
        }
        b'[' => {
            *p += 1;
            let mut a = Vec::new();
            loop {
                ws(b, p);
                if b[*p] == b']' {
                    *p += 1;
                    return J::Arr(a);
                }
                a.push(val(b, p));
                ws(b, p);
                if b[*p] == b',' {
                    *p += 1;
                }
            }
        }
        b'"' => {
            *p += 1;
            let mut s = String::new();
            while b[*p] != b'"' {
                if b[*p] == b'\\' {
                    *p += 1;
                    match b[*p] {
                        b'n' => s.push('\n'),
                        b't' => s.push('\t'),
                        b'u' => {
                            let h = std::str::from_utf8(&b[*p + 1..*p + 5]).unwrap();
                            s.push(char::from_u32(u32::from_str_radix(h, 16).unwrap()).unwrap_or('?'));
                            *p += 4;
                        }
                        c => s.push(c as char),
                    }
                } else {
                    s.push(b[*p] as char);
                }
                *p += 1;
            }
            *p += 1;
            J::Str(s)
        }
        b't' => {
            *p += 4;
            J::Bool(true)
        }
        b'f' => {
            *p += 5;
            J::Bool(false)
        }
        b'n' => {
            *p += 4;
            J::Null
        }
        _ => {
            let st = *p;
            while *p < b.len() && matches!(b[*p], b'-' | b'+' | b'.' | b'e' | b'E' | b'0'..=b'9') {
                *p += 1;
            }
            let t = std::str::from_utf8(&b[st..*p]).unwrap();
            if t.contains(['.', 'e', 'E']) {
                J::Num(t.parse().unwrap())
            } else {
                J::Int(t.parse().unwrap())
            }
        }
    }
}

// ---------------------------------------------------------------- local files (never printed)

fn walk_up(rel: &[&str]) -> Option<PathBuf> {
    let mut starts = vec![PathBuf::from(env!("CARGO_MANIFEST_DIR"))];
    if let Ok(c) = std::env::current_dir() {
        starts.push(c);
    }
    for s in starts {
        let mut d: Option<&Path> = Some(&s);
        while let Some(p) = d {
            let mut c = p.to_path_buf();
            for r in rel {
                c.push(r);
            }
            if c.is_file() {
                return Some(c);
            }
            d = p.parent();
        }
    }
    None
}

/// The reader's decoded dump (`BRAWL_BOSS_JSON`, or `_build/petey-spec/decoded/bosspackun.json` above the crate).
pub fn spec_json() -> Option<J> {
    let p = std::env::var_os("BRAWL_BOSS_JSON")
        .map(PathBuf::from)
        .filter(|p| p.is_file())
        .or_else(|| walk_up(&["_build", "petey-spec", "decoded", "bosspackun.json"]))?;
    Some(parse_json(&std::fs::read_to_string(p).ok()?))
}

/// The boss stage archive from the player's local Brawl folder (`MELEE_BRAWL_LOCAL`, or `_build/brawl` above the
/// crate), the same place the game looks.
pub fn stage_archive() -> Option<Vec<u8>> {
    let rel = ["disc", "files", "stage", "adventure", "030101.pac"];
    let p = match std::env::var_os("MELEE_BRAWL_LOCAL") {
        Some(r) => {
            let mut p = PathBuf::from(r);
            rel.iter().for_each(|x| p.push(x));
            Some(p).filter(|p| p.is_file())
        }
        None => walk_up(&["_build", "brawl", "disc", "files", "stage", "adventure", "030101.pac"]),
    }?;
    std::fs::read(p).ok()
}

// ---------------------------------------------------------------- synthetic files

/// Builds a boss file in memory: a data section with tracked pointer fields (relocations), roots, imports.
#[derive(Default)]
pub struct Synth {
    pub data: Vec<u8>,
    pub relocs: Vec<u32>,
    pub roots: Vec<(String, u32)>,
    /// name -> fields
    pub imports: Vec<(String, Vec<u32>)>,
}

impl Synth {
    pub fn here(&self) -> u32 {
        self.data.len() as u32
    }
    pub fn align(&mut self) {
        while self.data.len() % 4 != 0 {
            self.data.push(0);
        }
    }
    pub fn u32(&mut self, v: u32) -> u32 {
        let at = self.here();
        self.data.extend_from_slice(&v.to_be_bytes());
        at
    }
    pub fn f32(&mut self, v: f32) -> u32 {
        self.u32(v.to_bits())
    }
    pub fn zeros(&mut self, n: usize) -> u32 {
        let at = self.here();
        self.data.resize(self.data.len() + n, 0);
        at
    }
    pub fn set(&mut self, at: u32, v: u32) {
        self.data[at as usize..at as usize + 4].copy_from_slice(&v.to_be_bytes());
    }
    pub fn setf(&mut self, at: u32, v: f32) {
        self.set(at, v.to_bits());
    }
    /// A pointer field at `at` to `to`.
    pub fn ptr_at(&mut self, at: u32, to: u32) {
        self.set(at, to);
        self.relocs.push(at);
    }
    pub fn ptr(&mut self, to: u32) -> u32 {
        let at = self.u32(to);
        self.relocs.push(at);
        at
    }
    /// An import reference field at `at` (chained at build time).
    pub fn import_at(&mut self, at: u32, name: &str) {
        if let Some(e) = self.imports.iter_mut().find(|e| e.0 == name) {
            e.1.push(at);
        } else {
            self.imports.push((name.into(), vec![at]));
        }
    }
    pub fn cstr(&mut self, s: &str) -> u32 {
        let at = self.here();
        self.data.extend_from_slice(s.as_bytes());
        self.data.push(0);
        self.align();
        at
    }
    /// A script: each command is (id, args) with args as (type, value); type 2 values are pointer targets
    /// (relocated) unless `IMPORT:<name>` is given through `ScriptArg::Import`.
    pub fn script(&mut self, cmds: &[(u16, Vec<SArg>)]) -> u32 {
        // argument arrays first
        let mut arg_at = Vec::new();
        for (_, args) in cmds {
            if args.is_empty() {
                arg_at.push(None);
                continue;
            }
            let base = self.here();
            for a in args {
                match a {
                    SArg::T(t, v) => {
                        self.u32(*t);
                        self.u32(*v);
                    }
                    SArg::Ptr(to) => {
                        self.u32(2);
                        self.ptr(*to);
                    }
                    SArg::Import(n) => {
                        self.u32(2);
                        let f = self.u32(0);
                        self.import_at(f, n);
                    }
                }
            }
            arg_at.push(Some(base));
        }
        let at = self.here();
        for ((id, args), aa) in cmds.iter().zip(arg_at) {
            self.u32((*id as u32) << 16 | (args.len() as u32) << 8);
            match aa {
                Some(p) => {
                    self.ptr(p);
                }
                None => {
                    self.u32(0);
                }
            }
        }
        self.u32(0);
        self.u32(0);
        at
    }

    pub fn build(mut self) -> Vec<u8> {
        self.align();
        // chain the imports: each field holds the next field, the last 0xFFFFFFFF
        for (_, f) in &self.imports {
            for (k, &at) in f.iter().enumerate() {
                let next = f.get(k + 1).copied().unwrap_or(0xffff_ffff);
                self.data[at as usize..at as usize + 4].copy_from_slice(&next.to_be_bytes());
            }
        }
        let mut strings = Vec::new();
        let mut recs = Vec::new();
        for (n, at) in &self.roots {
            recs.push((*at, strings.len() as u32));
            strings.extend_from_slice(n.as_bytes());
            strings.push(0);
        }
        for (n, f) in &self.imports {
            recs.push((f[0], strings.len() as u32));
            strings.extend_from_slice(n.as_bytes());
            strings.push(0);
        }
        let mut o = Vec::new();
        let total = 0x20 + self.data.len() + 4 * self.relocs.len() + 8 * recs.len() + strings.len();
        o.extend_from_slice(&(total as u32).to_be_bytes());
        o.extend_from_slice(&(self.data.len() as u32).to_be_bytes());
        o.extend_from_slice(&(self.relocs.len() as u32).to_be_bytes());
        o.extend_from_slice(&(self.roots.len() as u32).to_be_bytes());
        o.extend_from_slice(&(self.imports.len() as u32).to_be_bytes());
        o.resize(0x20, 0);
        o.extend_from_slice(&self.data);
        for r in &self.relocs {
            o.extend_from_slice(&r.to_be_bytes());
        }
        for (a, s) in &recs {
            o.extend_from_slice(&a.to_be_bytes());
            o.extend_from_slice(&s.to_be_bytes());
        }
        o.extend_from_slice(&strings);
        o
    }
}

#[derive(Clone, Debug)]
pub enum SArg {
    T(u32, u32),
    Ptr(u32),
    Import(String),
}
pub fn int(v: i32) -> SArg {
    SArg::T(0, v as u32)
}
pub fn sc(v: f64) -> SArg {
    SArg::T(1, (v * 60000.0).round() as i32 as u32)
}
pub fn var(mem: u32, kind: u32, id: u32) -> SArg {
    SArg::T(5, mem << 28 | kind << 24 | id)
}
pub fn rq(id: u32, not: bool) -> SArg {
    SArg::T(6, (not as u32) << 31 | id)
}
pub fn boolean(b: bool) -> SArg {
    SArg::T(3, b as u32)
}
pub fn imp(n: &str) -> SArg {
    SArg::Import(n.into())
}
