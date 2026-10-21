#!/usr/bin/env python3
"""Turn group 3's raw c2rust output into compilable stable Rust modules.

  python rust/melee_fighter/gen/fix_raw.py            (from the worktree root; after transpile.sh)

Input:  rust/melee_fighter/src/raw/melee/{ft/*.rs, ft/kinds/ftCommon/*.rs, pl/*.rs}  (gitignored)
Output: rust/melee_fighter/src/c2r/{ft,ftcommon,pl}/*.rs + mod.rs files                (committed)

The raw code is already native (x86_64 little-endian, 8-byte pointers, no guest memory: c2rust
read the C with -DTARGET_PC for a native target). What this script fixes is only what stops it
compiling on stable:
  * `extern { pub type X; }` (nightly extern types)  -> `#[repr(C)] pub struct X { _opaque: [u8; 0] }`
  * `pub _: T` / `let mut _: T` (C names `_`)          -> `_unnamed_N`
  * `use ::c2rust_bitfields;`                          -> import the BitfieldStruct derive
  * `#[no_mangle] #[linkage = "external"]` (gnu89 header inline) -> a module-local fn
  * `::f128::f128` (long double literals)              -> f64 (long double is f64 on the GameCube)
  * hook calls c2rust left undeclared (a block-scope `extern` repeated in one TU becomes
    `Name_0` with no declaration)                     -> `#[link_name = "Name"] fn Name_0(...)`
  * HitCapsule's `owner` / `hit_grabbed_victim_only:1` union (c2rust makes the bit a whole u8)
                                                       -> accessors over the GC MSB (crate::compat)
  * the "RAW / Not compiled" banner                    -> a generated-file banner
Each module keeps its own copy of the C types (c2rust repeats headers per TU); layouts agree across
modules because every TU sees the same headers, and modules only meet through `extern "C"` symbols.
Unifying them onto `crate::types` (the GC-offset-annotated set) is later, mechanical work.
"""
import re, sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
RAW = ROOT / "rust/melee_fighter/src/raw/melee"
OUT = ROOT / "rust/melee_fighter/src/c2r"
GROUPS = {"ft": RAW / "ft", "ftcommon": RAW / "ft/kinds/ftCommon", "pl": RAW / "pl"}

BLOCK_EXTERN = re.compile(r"(?:^\s+|[{;]\s*)extern\s+([^=;(]*?)\b(\w+)\s*\(([^)]*)\)\s*;")
C2R = {"int": "::core::ffi::c_int", "unsigned": "::core::ffi::c_uint", "unsigned int": "::core::ffi::c_uint",
       "u32": "u32", "s32": "i32", "float": "f32", "f32": "f32", "void": None}


def c_to_rust(t):
    """Rust spelling of the simple C types used by the port's block-scope hook declarations."""
    t = t.strip()
    const = t.startswith("const ")
    t = t.replace("const ", "").strip()
    stars = t.count("*")
    base = t.replace("*", "").strip()
    if base == "char" and stars:
        r = "::core::ffi::c_char"
    elif base == "void" and stars:
        r = "::core::ffi::c_void"
    elif base in C2R:
        r = C2R[base]
    else:
        r = base  # a struct typedef the module defines (Fighter, HSD_GObj, Item...)
    for _ in range(stars):
        r = ("*const " if const else "*mut ") + r
    return r


def block_externs(c_text):
    """name -> (ret, [param types]) for function declarations at block scope in a C TU."""
    out = {}
    c_text = re.sub(r",\s*\n\s*", ", ", c_text)  # declarations wrapped over several lines
    for ln in c_text.splitlines():
        for m in BLOCK_EXTERN.finditer(ln):
          ret, name, params = m.group(1).strip(), m.group(2), m.group(3).strip()
          ps = []
          if params and params != "void":
              for prm in params.split(","):
                  prm = prm.strip()
                  mm = re.match(r"^(.*?[\s*])(\w+)$", prm)
                  ty = mm.group(1) if mm and mm.group(1).strip() not in ("", "const", "unsigned") and mm.group(2) not in ("int", "float", "char", "void", "unsigned") else prm
                  ps.append(c_to_rust(ty))
          out[name] = (None if ret == "void" else c_to_rust(ret), ps)
    return out


def missing_decls(body, c_text):
    """Extern decls for hook calls c2rust emitted without a declaration (repeated block externs)."""
    decls = []
    for name, (ret, ps) in sorted(block_externs(c_text).items()):
        for called in sorted(set(re.findall(r"\b(" + name + r"(?:_\d+)?)\(", body))):
            # c2rust puts a block-scope extern inside that block only; other blocks that call it
            # need a module-level declaration (4-space indent inside a top-level extern block).
            if re.search(r"^    (pub )?fn " + called + r"\(", body, re.M):
                continue
            r = f" -> {ret}" if ret else ""
            args = ", ".join(f"_: {t}" for t in ps)
            decls.append(f'    #[link_name = "{name}"]\n    fn {called}({args}){r};')
    return ('extern "C" {\n' + "\n".join(decls) + "\n}\n") if decls else ""


EXTERN_TYPE = re.compile(r"^\s*pub type (\w+);\s*$")


def fix(text, src, c_text=""):
    lines = text.splitlines()
    out, opaque, n_unnamed = [], [], 0
    in_extern = False
    pending_union_impl, union_name = False, None
    for ln in lines:
        if ln.startswith("// RAW c2rust output") or ln.startswith("// Not compiled."):
            continue
        if re.match(r'^(pub )?(unsafe )?extern "C" \{\s*$', ln):
            in_extern = True
        elif in_extern and ln.startswith("}"):
            in_extern = False
        if in_extern:
            m = EXTERN_TYPE.match(ln)
            if m:
                opaque.append(m.group(1))
                continue
        # a C member named `_` (padding / first union member): `_pad` in the definition and in
        # struct literals; at most one per record in this codebase (rustc rejects duplicates).
        # (Extern fn declarations keep `_` parameter names, which are legal there.)
        if not in_extern:
            ln = re.sub(r"^(\s*)pub _: ", lambda m: m.group(1) + "pub _pad: ", ln)
            ln = re.sub(r"^(\s*)_: ", lambda m: m.group(1) + "_pad: ", ln)
            if "let " not in ln and "fn " not in ln:
                ln = re.sub(r"([{,]\s*)_: ", lambda m: m.group(1) + "_pad: ", ln)
        if ln.strip() == '#[linkage = "external"]':
            # gnu89 `inline` in a header (math_ppc.h sqrtf...): every TU had its own copy, so keep
            # it module-local instead of exporting one clashing #[no_mangle] symbol per module.
            for k in range(len(out) - 1, max(len(out) - 4, -1), -1):
                if out[k].strip() == "#[no_mangle]":
                    del out[k]
                    break
            continue
        if ln.strip() == "pub hit_grabbed_victim_only: u8_0," and out and out[-1].strip() == "pub owner: *mut HSD_GObj,":
            # c2rust drops the `:1` in a union; the real field is the owner's GC MSB (crate::compat)
            ln = ln.replace("pub hit_grabbed_victim_only: u8_0,", "pub _owner_word: u64,")
            pending_union_impl = True
        if ln.startswith("pub union ") and ln.rstrip().endswith("{"):
            union_name = ln.split()[2]
        if ln == "}" and pending_union_impl:
            out.append(ln)
            out.append(f"impl {union_name} {{")
            out.append("    #[inline] pub fn hit_grabbed_victim_only(&self) -> u8_0 { crate::compat::gc_ptr_msb_flag(unsafe { self._owner_word }) }")
            out.append("    #[inline] pub fn set_hit_grabbed_victim_only(&mut self, v: u8_0) { crate::compat::gc_ptr_msb_flag_set(unsafe { &mut self._owner_word }, v) }")
            out.append("}")
            pending_union_impl = False
            continue
        if ln.strip() == "use ::c2rust_bitfields;":
            ln = "use ::c2rust_bitfields::BitfieldStruct;"
        while re.search(r"\bmut _:", ln):
            ln = re.sub(r"\bmut _:", f"mut _unnamed_{n_unnamed}:", ln, count=1)
            n_unnamed += 1
        ln = re.sub(r"\._(?![A-Za-z0-9_])", "._pad", ln)
        ln = ln.replace("::f128::f128::new(", "(").replace("::f128::f128", "f64")
        out.append(ln)
    hdr = [f"// @generated by rust/melee_fighter/gen/fix_raw.py from c2rust 0.22.1 output of {src}",
           "// (native x86_64, -DTARGET_PC). Mechanical translation: do not hand-edit; port by hand",
           "// into crate modules instead and drop the TU from c2r when it is replaced."]
    body = "\n".join(out)
    # an empty extern block left behind is legal; keep it
    opq = "".join(f"#[repr(C)]\npub struct {t} {{\n    _opaque: [u8; 0],\n}}\n" for t in opaque)
    return "\n".join(hdr) + "\n" + body + "\n" + opq + missing_decls(body, c_text)


def modname(stem):
    m = stem
    if m[0].isdigit():
        m = "_" + m
    return m


def main():
    OUT.mkdir(exist_ok=True)
    top = []
    for g, d in GROUPS.items():
        files = sorted(p for p in d.glob("*.rs"))
        if not files:
            raise SystemExit(f"fix_raw: no raw output in {d}; run tools/rust-port/transpile.sh first")
        gd = OUT / g
        gd.mkdir(exist_ok=True)
        mods = []
        for p in files:
            src = "src/melee/" + p.relative_to(RAW).with_suffix(".c").as_posix()
            (gd / (modname(p.stem) + ".rs")).write_text(fix(p.read_text(encoding="utf-8"), src, (ROOT / src).read_text(encoding="utf-8", errors="replace")), encoding="utf-8", newline="\n")
            mods.append(modname(p.stem))
        (gd / "mod.rs").write_text("// @generated by gen/fix_raw.py\n" + "".join(f"pub mod {m};\n" for m in mods),
                                   encoding="utf-8", newline="\n")
        top.append(g)
        print(f"fix_raw: {g}: {len(mods)} modules")
    (OUT / "mod.rs").write_text(
        "//! Mechanical (c2rust) native translation of every group-3 TU. See gen/fix_raw.py.\n"
        "// @generated by gen/fix_raw.py\n"
        "#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals,\n"
        "         unused_assignments, unused_mut, unused_variables, unused_parens, unused_unsafe, unused_imports,\n"
        "         unused_labels, unused_must_use, unreachable_code, path_statements, improper_ctypes, improper_ctypes_definitions,\n"
        "         clashing_extern_declarations, static_mut_refs, unpredictable_function_pointer_comparisons,\n"
        "         clippy::all)]\n"
        + "".join(f"pub mod {g};\n" for g in top), encoding="utf-8", newline="\n")


if __name__ == "__main__":
    main()
