#!/usr/bin/env python3
"""Generate rust/melee_chara/src/tu/ftkirbydata.rs from src/melee/ft/kinds/ftKirby/ftkirbydata.c.

  python rust/melee_chara/gen/kirby/gen_data.py        (any directory)

Computed from the C source only (no disc data). The hat callback tables of ftkirby.rs were written once
from ftkirby.c by hand-checked translation and are maintained in the Rust file.
"""
import re, sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
KB = ROOT / "src/melee/ft/kinds/ftKirby"
OUT = ROOT / "rust/melee_chara/src/tu"

NMAX = 127

src = (KB / "ftkirbydata.c").read_text(encoding="utf8")

# strip the TARGET_PC function (hand-ported)
data_part = src.split("#if defined(TARGET_PC)")[0]

lines = []
w = lines.append

w("""//! src/melee/ft/kinds/ftKirby/ftkirbydata.c: the file-name / symbol-name tables of Kirby's copy
//! abilities and the m-ex hook `ftKb_MexCopyKindData`. No fused sites (tools/rust-port/fma_sites.py).
//! The tables are generated from the C by `gen/kirby/gen_data.py` (strings are NUL-terminated byte arrays,
//! C `NULL` is a null pointer); pointers are native. The table rows are `static mut` because
//! `ftKb_MexCopyKindData` rewrites them at m-ex load, like the C globals.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use melee_types::*;
use core::ptr::null_mut;

/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics.
pub mod ext {
    macro_rules! hook {
        ($hook:ident, $name:ident, ($($a:ident: $t:ty),*) $(-> $r:ty)?) => {
            pub static mut $hook: Option<fn($($t),*) $(-> $r)?> = None;
            #[allow(non_snake_case)]
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $hook } {
                    Some(f) => f($($a),*),
                    None => panic!(concat!("TODO: ", stringify!($name), " is not ported")),
                }
            }
        };
    }
    hook!(MEX_KIRBYCAPFILE, Mex_KirbyCapFile, (internal: i32) -> *mut i8);
    hook!(MEX_KIRBYCAPSYMBOL, Mex_KirbyCapSymbol, (internal: i32) -> *mut i8);
    hook!(MEX_KIRBYEFFECTID, Mex_KirbyEffectId, (internal: i32) -> i32);
}

/// `Ft_Kind_Max` table length.
pub const KIND_MAX: usize = %d;
""" % NMAX)

# ---- char NAME[] = "..." ----
strs = {}
for m in re.finditer(r'^char (\w+)\[\] = ("(?:[^"\\]|\\.)*");', data_part, re.M):
    name, lit = m.group(1), m.group(2)
    body = lit[1:-1]
    n = len(bytes(body, "utf8").decode("unicode_escape").encode("latin1")) + 1
    strs[name] = n
    w(f'pub static {name}: [u8; {n}] = *b"{body}\\0";')
w("")

def cstr(name):
    return f"{name}.as_ptr() as *mut i8"

# ---- char* NAME[] = {"...", NULL, ...} ----
for m in re.finditer(r'^char\* (\w+)\[\] = \{(.*?)\n\};', data_part, re.M | re.S):
    name, body = m.group(1), m.group(2)
    items = [x.strip() for x in re.split(r',\s*\n', body.strip().rstrip(',')) if x.strip()]
    out = []
    for it in items:
        if it == "NULL":
            out.append("null_mut()")
        else:
            out.append(f'concat!({it}, "\\0").as_ptr() as *mut i8')
    w(f"pub static mut {name}: [*mut i8; {len(items)}] = [")
    for o in out:
        w(f"    {o},")
    w("];")
    w("")

# ---- Fighter_DemoStrings ----
m = re.search(r'^Fighter_DemoStrings (\w+) = \{(.*?)\n\};', data_part, re.M | re.S)
items = re.findall(r'"([^"]*)"', m.group(2))
w(f"pub static mut {m.group(1)}: Fighter_DemoStrings = Fighter_DemoStrings {{")
for fld, it in zip(["result_filename", "intro_filename", "ending_filename", "vi_wait_filename"], items):
    w(f'    {fld}: concat!("{it}", "\\0").as_ptr() as *mut i8,')
w("};")
w("")

# ---- Fighter_CostumeStrings NAME[] = { {a, b, c}, ... } ----
for m in re.finditer(r'^Fighter_CostumeStrings (\w+)\[\] = \{(.*?)\n\};', data_part, re.M | re.S):
    name, body = m.group(1), m.group(2)
    rows = re.findall(r'\{\s*(\w+),\s*(\w+),\s*(\w+)\s*\}', body)
    w(f"pub static mut {name}: [Fighter_CostumeStrings; {len(rows)}] = [")
    for a, b, c in rows:
        w(f"    Fighter_CostumeStrings {{ dat_filename: {cstr(a)}, joint_name: {cstr(b)}, matanim_joint_name: {cstr(c)} }},")
    w("];")
    w("")

# ---- ftKirby_CopyName NAME[Ft_Kind_Max] = {...} ----
m = re.search(r'^ftKirby_CopyName (\w+)\[Ft_Kind_Max\] = \{(.*?)\n\};', data_part, re.M | re.S)
name, body = m.group(1), m.group(2)
ents = []
for r in re.finditer(r'\{\s*(NULL|"[^"]*"),\s*(NULL|"[^"]*")\s*\}|^\s*(NULL),\s*$', body, re.M):
    if r.group(1) is not None:
        def p(x):
            return "null_mut()" if x == "NULL" else f'concat!({x}, "\\0").as_ptr() as *mut i8'
        ents.append(f"ftKirby_CopyName {{ filename: {p(r.group(1))}, name: {p(r.group(2))} }}")
    else:
        ents.append("ftKirby_CopyName { filename: null_mut(), name: null_mut() }")
assert len(ents) <= NMAX, len(ents)
while len(ents) < NMAX:
    ents.append("ftKirby_CopyName { filename: null_mut(), name: null_mut() }")
w(f"/// `{name}`: per-kind Kirby hat archive file / symbol names.")
w(f"pub static mut {name}: [ftKirby_CopyName; {NMAX}] = [")
for e in ents:
    w(f"    {e},")
w("];")
w("")

# ---- Fighter_CostumeStrings* NAME[Ft_Kind_Max] = {...} ----
m = re.search(r'^Fighter_CostumeStrings\* (\w+)\[Ft_Kind_Max\] = \{(.*?)\n\};', data_part, re.M | re.S)
name, body = m.group(1), m.group(2)
items = [x.strip() for x in body.replace("\n", " ").split(",") if x.strip()]
assert len(items) <= NMAX
items += ["NULL"] * (NMAX - len(items))
w(f"/// `{name}`: per-kind Kirby hat COSTUME model name rows (m-ex MexData.kirby_data.costumes).")
w(f"pub static mut {name}: [*mut Fighter_CostumeStrings; {NMAX}] = [")
for it in items:
    w("    null_mut()," if it == "NULL" else f"    (&raw mut {it}) as *mut Fighter_CostumeStrings,")
w("];")
w("")

# ---- u8 NAME[Ft_Kind_Max] = {...} ----
m = re.search(r'^u8 (\w+)\[Ft_Kind_Max\] = \{(.*?)\n\};', data_part, re.M | re.S)
name, body = m.group(1), m.group(2)
vals = [int(x) & 0xFF for x in re.findall(r'-?\d+', body)]
vals += [0] * (NMAX - len(vals))
w(f"/// `{name}`: per-kind Kirby hat effect bank id (0xFF = none).")
w(f"pub static mut {name}: [u8; {NMAX}] = [{', '.join(map(str, vals))}];")
w("")

w("""/// `ftKb_MexCopyKindData(dst, src, internal)` (TARGET_PC): fill fighter kind `dst`'s Kirby hat archive
/// and effect bank from MxDt.dat's MexData.kirby_data, for m-ex INTERNAL kind `internal`. `src` is the
/// clone base, used only when the disc has no row. The effect id is stored as 0xFF when m-ex says
/// "no effect bank" (255, or a negative id), which `ftKb_SpecialN_800EEC34`'s existing guard covers.
pub fn ftKb_MexCopyKindData(dst: i32, src: i32, internal: i32) {
    let file = ext::Mex_KirbyCapFile(internal);
    let symbol = ext::Mex_KirbyCapSymbol(internal);
    let effect = ext::Mex_KirbyEffectId(internal);
    unsafe {
        if !file.is_null() && !symbol.is_null() {
            ftKb_Init_803CA9D0[dst as usize].filename = file;
            ftKb_Init_803CA9D0[dst as usize].name = symbol;
        } else {
            ftKb_Init_803CA9D0[dst as usize] = ftKb_Init_803CA9D0[src as usize];
        }
        ftKb_Init_803CB46C[dst as usize] = (if effect < 0 { 0xFF } else { effect }) as u8;
    }
    // One line per added fighter (OSReport in the C); it prints what m-ex supplied, never a read-back
    // of the clone base's pointers.
    let cs = |p: *mut i8| -> String {
        if p.is_null() {
            "(clone base)".to_string()
        } else {
            unsafe { core::ffi::CStr::from_ptr(p as *const core::ffi::c_char) }.to_string_lossy().into_owned()
        }
    };
    eprintln!("gw: kind {} Kirby hat: {} / {}, effect {}", dst, cs(file), cs(symbol), effect);
}
""")

(OUT / "ftkirbydata.rs").write_text("\n".join(lines) + "\n", encoding="utf8")
print("ftkirbydata.rs:", len(lines), "lines")
