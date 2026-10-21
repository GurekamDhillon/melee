//! Two tables generated from this checkout's sources (never from disc data):
//!   aliases.rs: melee_types typedef aliases (`pub type Vec3 = Vec;`, `pub type Vec2Ptr = *mut
//!     Vec2;`), so a field typed `Vec3` resolves to the record GC_RECORDS names `Vec`.
//!   returns.rs: vanilla functions declared `T *name(` in the decomp headers (src/**/*.h), keyed
//!     by their GC address from config/GALE01/symbols.txt: a `bl` to one leaves r3 = T*; and
//!     NAMES, every symbols.txt function by address (bridge logs).
use std::collections::{BTreeMap, HashMap};
use std::fmt::Write;
use std::path::{Path, PathBuf};

fn main() {
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    let od = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    std::fs::write(od.join("aliases.rs"), aliases(&here.join("../../rust/melee_types/src/gen.rs"))).unwrap();
    std::fs::write(od.join("returns.rs"), returns(&here.join("../.."))).unwrap();
    std::fs::write(od.join("native.rs"), native(&here.join("../../rust/melee_types/src/gen.rs"))).unwrap();
}

/// native.rs: for every melee_types record, each field's GameCube offset next to its NATIVE
/// offset (`offset_of!`) and a descriptor of its native type (GC/native sizes from
/// `GcDecode::GC_BYTES` / `size_of`, and whether it is a scalar, a pointer, a nested record or an
/// array). Read from the generated `GcDecode` impls (`v.f = <T as GcDecode>::gc_decode(d, at +
/// 0xOFF);`), which name the native field and type for each GC offset. A union contributes its
/// first decoded member. This is what `remap` rewrites guest field accesses with.
fn native(gen: &Path) -> String {
    let src = std::fs::read_to_string(gen).expect("melee_types/src/gen.rs");
    let mut gc_name: HashMap<String, String> = HashMap::new();
    let mut recs: Vec<(String, Vec<(String, String, u32)>)> = vec![];
    let mut cur: Option<usize> = None;
    let mut layout_for: Option<String> = None;
    for l in src.lines() {
        if let Some(r) = l.strip_prefix("impl GcDecode for ") {
            let id = r.trim_end_matches(" {").trim();
            cur = if ident(id) { recs.push((id.to_string(), vec![])); Some(recs.len() - 1) } else { None };
            continue;
        }
        if let Some(r) = l.strip_prefix("impl GcLayout for ") {
            layout_for = Some(r.trim_end_matches(" {").trim().to_string());
            continue;
        }
        if let (Some(id), Some(r)) = (&layout_for, l.trim().strip_prefix("const GC_NAME: &'static str = \"")) {
            gc_name.insert(id.clone(), r.trim_end_matches("\";").to_string());
            layout_for = None;
            continue;
        }
        if l.starts_with('}') {
            cur = None;
        }
        let Some(ci) = cur else { continue };
        // A union with a pointer member decodes it only when the word is relocated; the rewrite
        // uses the pointer-free member (`else { v.f = ... }`), as an unrelocated decode does.
        if l.trim_start().starts_with("if d.is_reloc(") {
            continue;
        }
        let Some(vi) = l.find("v.") else { continue };
        let rest = &l[vi + 2..];
        let Some((field, rest)) = rest.split_once(" = <") else { continue };
        let Some((ty, rest)) = rest.split_once(" as GcDecode>::gc_decode(d, at + 0x") else { continue };
        let Some(off) = rest.split(')').next().and_then(|h| u32::from_str_radix(h, 16).ok()) else { continue };
        let fields = &mut recs[ci].1;
        if !fields.iter().any(|f| f.0 == field) {
            fields.push((field.to_string(), ty.to_string(), off));
        }
    }
    let names: std::collections::HashSet<String> = recs.iter().map(|r| r.0.clone()).collect();
    // `pub type Mtx = [[f32; 4]; 3];`: a field typed by an alias is classified by what the alias
    // names (else an array alias would read as one big scalar and be byte-reversed whole).
    let mut alias: HashMap<String, String> = HashMap::new();
    for l in src.lines() {
        if let Some((n, t)) = l.strip_prefix("pub type ").and_then(|r| r.trim_end_matches(';').split_once(" = ")) {
            alias.insert(n.trim().to_string(), t.trim().to_string());
        }
    }
    let mut tys: Vec<String> = vec![];
    let mut ty_ix: HashMap<String, usize> = HashMap::new();
    fn intern(t: &str, tys: &mut Vec<String>, ix: &mut HashMap<String, usize>, names: &std::collections::HashSet<String>, gcn: &HashMap<String, String>, al: &HashMap<String, String>, out: &mut String) -> usize {
        if let Some(&i) = ix.get(t) {
            return i;
        }
        let mut rt = t;
        for _ in 0..8 {
            match al.get(rt) {
                Some(n) if !names.contains(rt) => rt = n,
                _ => break,
            }
        }
        let t_decl = t;
        let t = rt;
        let pointee = t.strip_prefix("*mut ").or_else(|| t.strip_prefix("*const "));
        let kind = if let Some(p) = pointee.filter(|p| names.contains(*p)) {
            format!("Kind::PtrTo({:?})", gcn.get(p).cloned().unwrap_or_else(|| p.to_string()))
        } else if t.starts_with('*') || t.starts_with("Option<") {
            "Kind::Ptr".to_string()
        } else if t == "bool" {
            "Kind::Bool".to_string()
        } else if let Some(inner) = t.strip_prefix('[').and_then(|x| x.strip_suffix(']')) {
            let (e, n) = inner.rsplit_once("; ").unwrap();
            let ei = intern(e.trim(), tys, ix, names, gcn, al, out);
            format!("Kind::Arr(&T{ei}, {n})")
        } else if names.contains(t) {
            format!("Kind::Rec({:?})", gcn.get(t).cloned().unwrap_or_else(|| t.to_string()))
        } else {
            "Kind::Scalar".to_string()
        };
        let i = tys.len();
        tys.push(t_decl.to_string());
        ix.insert(t_decl.to_string(), i);
        writeln!(out, "static T{i}: Ty = Ty {{ gc: <{t_decl} as GcDecode>::GC_BYTES, nat: size_of::<{t_decl}>(), kind: {kind} }};").unwrap();
        i
    }
    let mut o = String::from("#[allow(unused_imports)] use melee_types::*;
#[allow(unused_imports)] use core::ffi::c_void;
use melee_core::gcdecode::GcDecode;
use core::mem::{offset_of, size_of};
");
    let mut table = String::from("pub static NATIVE: &[(&str, usize, &[NField])] = &[
");
    for (id, fields) in &recs {
        let mut fl = String::new();
        for (f, t, off) in fields {
            let ti = intern(t, &mut tys, &mut ty_ix, &names, &gc_name, &alias, &mut o);
            write!(fl, "NField {{ gc: {off:#x}, nat: offset_of!({id}, {f}), ty: &T{ti} }}, ").unwrap();
        }
        let gn = gc_name.get(id).cloned().unwrap_or_else(|| id.clone());
        writeln!(table, "    ({gn:?}, size_of::<{id}>(), &[{fl}]),").unwrap();
    }
    table.push_str("];
");
    o.push_str(&table);
    o
}

fn aliases(gen: &Path) -> String {
    println!("cargo:rerun-if-changed={}", gen.display());
    let src = std::fs::read_to_string(gen).expect("melee_types/src/gen.rs");
    let mut out = String::from("pub static ALIASES: &[(&str, &str, bool)] = &[\n");
    for l in src.lines() {
        let Some(rest) = l.strip_prefix("pub type ") else { continue };
        let Some((name, target)) = rest.trim_end_matches(';').split_once(" = ") else { continue };
        let (target, ptr) = match target.strip_prefix("*mut ").or_else(|| target.strip_prefix("*const ")) {
            Some(t) => (t, true),
            None => (target, false),
        };
        if target.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            writeln!(out, "    ({name:?}, {target:?}, {ptr}),").unwrap();
        }
    }
    out.push_str("];\n");
    out
}

fn ident(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') && !s.starts_with(|c: char| c.is_ascii_digit())
}

fn returns(root: &Path) -> String {
    let syms = root.join("config/GALE01/symbols.txt");
    println!("cargo:rerun-if-changed={}", syms.display());
    println!("cargo:rerun-if-changed={}", root.join("src").display());
    let mut addr = HashMap::new();
    for l in std::fs::read_to_string(&syms).unwrap_or_default().lines() {
        if !l.contains("type:function") {
            continue;
        }
        let Some((name, rest)) = l.split_once(" = ") else { continue };
        let Some(hex) = rest.split(':').nth(1).and_then(|h| h.split(';').next()) else { continue };
        if let Ok(a) = u32::from_str_radix(hex.trim().trim_start_matches("0x"), 16) {
            addr.insert(name.trim().to_string(), a);
        }
    }
    let mut found = BTreeMap::new();
    let mut stack = vec![root.join("src")];
    while let Some(d) = stack.pop() {
        let Ok(rd) = std::fs::read_dir(&d) else { continue };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
                continue;
            }
            if p.extension().map(|x| x != "h").unwrap_or(true) {
                continue;
            }
            for l in std::fs::read_to_string(&p).unwrap_or_default().lines() {
                let mut l = l.trim();
                if let Some(r) = l.strip_prefix("/*") {
                    match r.find("*/") {
                        Some(i) => l = r[i + 2..].trim(),
                        None => continue,
                    }
                }
                for pre in ["extern ", "struct ", "union "] {
                    l = l.strip_prefix(pre).unwrap_or(l).trim_start();
                }
                let Some(star) = l.find('*') else { continue };
                let ty = l[..star].trim();
                let rest = l[star + 1..].trim_start();
                let Some(paren) = rest.find('(') else { continue };
                let name = rest[..paren].trim();
                if !ident(ty) || !ident(name) || matches!(ty, "void" | "char" | "return" | "const") {
                    continue;
                }
                if let Some(a) = addr.get(name) {
                    found.insert(*a, (name.to_string(), ty.to_string()));
                }
            }
        }
    }
    let mut o = String::from("pub static RETURNS: &[(u32, &str, &str)] = &[\n");
    for (a, (n, t)) in found {
        writeln!(o, "    ({a:#x}, {n:?}, {t:?}),").unwrap();
    }
    o.push_str("];\n");
    // Every vanilla function's name by address (symbols.txt), for bridge logs.
    let mut by_addr: Vec<(u32, &String)> = addr.iter().map(|(n, a)| (*a, n)).collect();
    by_addr.sort();
    by_addr.dedup_by_key(|e| e.0);
    o.push_str("pub static NAMES: &[(u32, &str)] = &[\n");
    for (a, n) in by_addr {
        writeln!(o, "    ({a:#x}, {n:?}),").unwrap();
    }
    o.push_str("];\n");
    o
}
