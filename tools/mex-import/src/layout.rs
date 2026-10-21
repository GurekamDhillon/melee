//! "GameCube offset -> native field" over melee_types' generated layout tables
//! (`melee_types::GC_RECORDS`, one entry per record with its `GcLayout` fields).
//!
//! `resolve(record, gc_offset)` walks nested records and arrays down to the innermost field
//! covering the byte and returns both spellings: the C path (`Fighter.x914[2].x0`) and the
//! native Rust path (`Fighter.x914[2].x0`; `(anon)` members become `anon_<hex>` and bitfield
//! storage units `bf_<hex>`, as gen_types.py names them).

use melee_core::gc::GcField;
use std::collections::HashMap;

mod aliases {
    include!(concat!(env!("OUT_DIR"), "/aliases.rs"));
}
mod returns {
    include!(concat!(env!("OUT_DIR"), "/returns.rs"));
}

/// The symbols.txt name of the vanilla function at `addr` (None when not a function start).
pub fn vanilla_name(addr: u32) -> Option<&'static str> {
    returns::NAMES.binary_search_by_key(&addr, |e| e.0).ok().map(|i| returns::NAMES[i].1)
}

pub struct Registry {
    pub records: Vec<(&'static str, u32, &'static [GcField])>,
    by_name: HashMap<&'static str, usize>,
    /// typedef name -> (record, is pointer)
    alias: HashMap<&'static str, (usize, bool)>,
    /// Vanilla GC function address -> record its `T *` return points to (build.rs, from the
    /// decomp headers + symbols.txt).
    pub returns: HashMap<u32, (usize, &'static str)>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CType {
    Record(usize),
    /// Pointer to a known record.
    PtrTo(usize),
    /// Anything else (scalars, void*, function pointers, unknown typedefs, unnamed records).
    Leaf,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mapping {
    pub c_path: String,
    pub native_path: String,
    /// The innermost field's C type and its GC offset/size inside the outermost record.
    pub leaf_type: &'static str,
    pub leaf_off: u32,
    pub leaf_size: u32,
    pub bitfield: bool,
    /// The access starts at the leaf's first byte.
    pub at_start: bool,
    /// Pointer type of the leaf, if it points at a known record.
    pub leaf_ptr: Option<usize>,
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}

impl Registry {
    pub fn new() -> Registry {
        let records: Vec<_> = melee_types::GC_RECORDS.iter().map(|(n, s, _, f)| (*n, *s, *f)).collect();
        let mut by_name = HashMap::new();
        for (i, (n, _, _)) in records.iter().enumerate() {
            by_name.entry(*n).or_insert(i);
        }
        let mut alias = HashMap::new();
        for &(n, t, ptr) in aliases::ALIASES {
            // follow alias chains a few levels (Vec3 = Vec; a = Vec3)
            let (mut t, mut ptr) = (t, ptr);
            for _ in 0..4 {
                if by_name.contains_key(t) {
                    break;
                }
                match aliases::ALIASES.iter().find(|a| a.0 == t) {
                    Some(&(_, t2, p2)) => {
                        t = t2;
                        ptr |= p2;
                    }
                    None => break,
                }
            }
            if let Some(&r) = by_name.get(t) {
                if !by_name.contains_key(n) {
                    alias.insert(n, (r, ptr));
                }
            }
        }
        let mut reg = Registry { records, by_name, alias, returns: HashMap::new() };
        for &(a, name, ty) in returns::RETURNS {
            if let Some(r) = reg.find(ty) {
                reg.returns.insert(a, (r, name));
            }
        }
        reg
    }

    pub fn find(&self, name: &str) -> Option<usize> {
        self.by_name.get(name).copied().or_else(|| self.alias.get(name).filter(|a| !a.1).map(|a| a.0))
    }
    pub fn name(&self, r: usize) -> &'static str {
        self.records[r].0
    }
    pub fn size(&self, r: usize) -> u32 {
        self.records[r].1
    }

    fn strip(t: &str) -> &str {
        let mut t = t.trim();
        for p in ["const ", "volatile ", "struct ", "union ", "enum "] {
            if let Some(r) = t.strip_prefix(p) {
                t = r.trim();
            }
        }
        t
    }

    /// (element type text, element count) for `T[N]...`; count 1 when not an array.
    fn array(t: &str) -> (&str, u32) {
        if t.contains('(') {
            return (t, 1); // function pointers, pointer-to-array: leaves
        }
        let Some(i) = t.find('[') else { return (t, 1) };
        let mut n: u32 = 1;
        for part in t[i..].split('[').skip(1) {
            let d = part.trim_end_matches(']').trim();
            n = n.saturating_mul(d.parse().unwrap_or(1));
        }
        (t[..i].trim(), n)
    }

    pub fn classify(&self, c_type: &str) -> CType {
        let t = Self::strip(c_type);
        if t.contains('(') {
            return CType::Leaf;
        }
        if let Some(p) = t.strip_suffix('*') {
            let p = Self::strip(p);
            if p.contains('*') {
                return CType::Leaf;
            }
            return self.find(p).map(CType::PtrTo).unwrap_or(CType::Leaf);
        }
        if let Some(&(r, true)) = self.alias.get(t) {
            return CType::PtrTo(r);
        }
        self.find(t).map(CType::Record).unwrap_or(CType::Leaf)
    }

    fn native_name(f: &GcField) -> String {
        if f.bits.is_some() {
            format!("bf_{:x}", f.offset)
        } else if f.name == "(anon)" {
            format!("anon_{:x}", f.offset)
        } else {
            f.name.to_string()
        }
    }

    /// The innermost field of record `r` covering GC byte `off`, for an access of `size` bytes.
    pub fn resolve(&self, r: usize, off: u32, size: u32) -> Option<Mapping> {
        let root = self.name(r);
        let mut m = Mapping {
            c_path: root.to_string(),
            native_path: root.to_string(),
            leaf_type: "",
            leaf_off: 0,
            leaf_size: 0,
            bitfield: false,
            at_start: false,
            leaf_ptr: None,
        };
        let (mut rec, mut rel, mut base) = (r, off, 0u32);
        let mut found = false;
        for _ in 0..12 {
            let fields = self.records[rec].2;
            // Prefer the first field covering the byte (a union's first member, a bitfield
            // unit's first bitfield), as GcLayout::gc_field_at does.
            let Some(f) = fields.iter().find(|f| rel >= f.offset && rel < f.offset + f.size.max(1)) else { break };
            found = true;
            m.c_path.push('.');
            m.c_path.push_str(f.name);
            m.native_path.push('.');
            m.native_path.push_str(&Self::native_name(f));
            m.leaf_type = f.c_type;
            m.leaf_off = base + f.offset;
            m.leaf_size = f.size;
            m.bitfield = f.bits.is_some();
            m.leaf_ptr = None;
            let inner = rel - f.offset;
            let (elem, n) = Self::array(f.c_type);
            let esize = if n > 1 { f.size / n } else { f.size };
            let (idx, inner) = if n > 1 && esize > 0 { (Some(inner / esize), inner % esize) } else { (None, inner) };
            if let Some(i) = idx {
                let s = format!("[{i}]");
                m.c_path.push_str(&s);
                m.native_path.push_str(&s);
                m.leaf_off += i * esize;
                m.leaf_size = esize;
                m.leaf_type = f.c_type;
            }
            if m.bitfield {
                break;
            }
            match self.classify(elem) {
                CType::Record(sub) if self.size(sub) > 0 => {
                    base = m.leaf_off;
                    rec = sub;
                    rel = inner;
                    continue;
                }
                CType::PtrTo(p) if n == 1 => m.leaf_ptr = Some(p),
                _ => {}
            }
            break;
        }
        if !found {
            return None;
        }
        m.at_start = off == m.leaf_off && (m.bitfield || size == m.leaf_size || size == 0);
        Some(m)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fighter_fields() {
        let r = Registry::new();
        let ft = r.find("Fighter").unwrap();
        let m = r.resolve(ft, 0x84, 4).unwrap();
        assert_eq!(m.c_path, "Fighter.self_vel.y");
        assert!(m.at_start);
        let m = r.resolve(ft, 0x2C, 4).unwrap();
        assert_eq!(m.c_path, "Fighter.facing_dir");
        assert_eq!(m.leaf_type, "float");
        let m = r.resolve(ft, 0x10C, 4).unwrap();
        assert_eq!(m.leaf_ptr, r.find("ftData"));
        let m = r.resolve(ft, 0x594, 4).unwrap();
        assert!(m.native_path.starts_with("Fighter.anon_594"), "{}", m.native_path);
        // an array of records: HitCapsule[4] at 0x914, element 2
        let hc = r.size(r.find("HitCapsule").unwrap());
        let m = r.resolve(ft, 0x914 + 2 * hc, 4).unwrap();
        assert!(m.c_path.starts_with("Fighter.x914[2]."), "{}", m.c_path);
        let g = r.find("HSD_GObj").unwrap();
        assert_eq!(r.resolve(g, 0x2C, 4).unwrap().c_path, "HSD_GObj.user_data");
        // a bitfield unit maps to its native bf_ storage
        let m = r.resolve(ft, 0x2101, 1).unwrap();
        assert!(m.bitfield && m.native_path == "Fighter.bf_2101", "{m:?}");
    }
}
