//! Two tables generated from this checkout's sources (never from disc data):
//!   aliases.rs: melee_types typedef aliases (`pub type Vec3 = Vec;`, `pub type Vec2Ptr = *mut
//!     Vec2;`), so a field typed `Vec3` resolves to the record GC_RECORDS names `Vec`.
//!   returns.rs: vanilla functions declared `T *name(` in the decomp headers (src/**/*.h), keyed
//!     by their GC address from config/GALE01/symbols.txt: a `bl` to one leaves r3 = T*.
use std::collections::{BTreeMap, HashMap};
use std::fmt::Write;
use std::path::{Path, PathBuf};

fn main() {
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    let od = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    std::fs::write(od.join("aliases.rs"), aliases(&here.join("../../rust/melee_types/src/gen.rs"))).unwrap();
    std::fs::write(od.join("returns.rs"), returns(&here.join("../.."))).unwrap();
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
    o
}
