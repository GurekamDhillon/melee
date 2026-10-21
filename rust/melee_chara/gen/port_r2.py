#!/usr/bin/env python3
"""Round-2 porting aid: mechanical C -> Rust translation of the thin character callbacks.

  python rust/melee_chara/gen/port_r2.py      (from the worktree root; Windows, LLVM clang)

Scope (anything outside it is left for hand porting): a `void f(HSD_GObj* gobj)` function in
src/melee/ft/kinds/<Char>/*.c (not ftCommon) whose body is built only from
  - `Fighter* fp = GET_FIGHTER(gobj);` / `= gobj->user_data;`, `PAD_STACK(n);`, `u8 _[n];`
  - calls to the group-3 functions this crate stubs (`crate::stubs`, TODO(group3)) or to the
    ported ftCommon helpers (`ftCommon_8007D5D4/8007D60C`, the two state-change inlines of
    ftCommon/inlines.h), with arguments that are `gobj`, `fp`, `NULL`, a compile-time constant
    or a plain `fp->field` read;
  - `if (c) { ... } [else { ... }]` where c is `[!]call`, `call ==/!= const`, and `return;`.
Every constant argument is evaluated by clang from the TU itself (the TU is `#include`d into a
small program that prints the value bits), so Rust carries the exact value with the C spelling
in a comment. Output: src/chars/r2/<Char>.rs (one `pub mod <tu>` per C file, re-exported from
the round-1 TU module) and gen/ported_r2.txt. Every function is then checked against the C by
gen/chref.py / tests/chars_ref.rs like the hand ports. Computed from C source only.
"""
import os, re, struct, subprocess, sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
HERE = Path(__file__).resolve().parent
CRATE = HERE.parent
sys.path.insert(0, str(HERE))
import chref  # noqa: E402  (shares the compiler flags)

BUILD = ROOT / "rust/target/port_r2"

# stub name -> (rust fn in crate::stubs, [param kinds], return kind)  g=gobj f=f32 i=i32 u=u32 p=gobj-or-null
STUBS = {
    "ft_80084F3C": ("ft_80084F3C", "g", "v"), "ft_80084EEC": ("ft_80084EEC", "g", "v"),
    "ft_80085134": ("ft_80085134", "g", "v"), "ft_80084DB0": ("ft_80084DB0", "g", "v"),
    "ft_80084FA8": ("ft_80084FA8", "g", "v"), "ft_80083F88": ("ft_80083F88", "g", "v"),
    "ftAnim_IsFramesRemaining": ("ftAnim_IsFramesRemaining", "g", "i"),
    "ftAnim_8006EBA4": ("ftAnim_8006EBA4", "g", "v"),
    "Fighter_ChangeMotionState": ("Fighter_ChangeMotionState", "giufffp", "v"),
    "ft_8008A2BC": ("ft_8008A2BC", "g", "v"), "ftCo_Fall_Enter": ("ftCo_Fall_Enter", "g", "v"),
    "ft_80081D0C": ("ft_80081D0C", "g", "i"), "ft_80082708": ("ft_80082708", "g", "i"),
    "ft_800827A0": ("ft_800827A0", "g", "i"),
    "ftCommon_8007D7FC": ("ftCommon_8007D7FC", "F", "v"),
    "ftCo_80096900": ("ftCo_80096900", "giiiff", "v"),
}
# ported group-3 / inline helpers (crate::stubs re-exports or wraps them)
HELPERS = {
    "ftCommon_8007D5D4": ("ftCommon_8007D5D4", "F", "v"),
    "ftCommon_8007D60C": ("ftCommon_8007D60C", "F", "v"),
    "ftCommon_GroundToAirStateChange": ("ftCommon_GroundToAirStateChange", "gFiu", "v"),
    "ftCommon_AirToGroundStateChange": ("ftCommon_AirToGroundStateChange", "gFiu", "v"),
}
CALLS = {**STUBS, **HELPERS}


class Skip(Exception):
    pass


def strip(body):
    body = re.sub(r"/\*.*?\*/", " ", body, flags=re.S)
    body = re.sub(r"//[^\n]*", " ", body)
    if "#" in body:
        raise Skip("preprocessor")
    return body


def split_args(s):
    out, d, cur = [], 0, ""
    for c in s:
        if c == "," and d == 0:
            out.append(cur.strip()); cur = ""; continue
        d += c in "([{"; d -= c in ")]}"
        cur += c
    if cur.strip():
        out.append(cur.strip())
    return out


class TU:
    def __init__(self, path):
        self.path = path
        self.consts = []  # (expr, kind)

    def const(self, expr, kind):
        if re.search(r"\b(fp|gobj)\b|->|\(\s*\w+\s*\(", expr):
            raise Skip(f"non-constant {expr!r}")
        if re.search(r"\b[a-z_]\w*\s*\(", expr) and not re.search(r"\b(sizeof)\s*\(", expr):
            raise Skip(f"call in constant {expr!r}")
        key = (expr, kind)
        if key not in self.consts:
            self.consts.append(key)
        return "@C%d@" % self.consts.index(key)


class Fn:
    def __init__(self, tu, name, body):
        self.tu, self.name, self.has_fp, self.lines = tu, name, False, []

    def arg(self, a, k):
        a = a.strip()
        if k == "g":
            if a != "gobj":
                raise Skip(f"gobj arg {a!r}")
            return "gobj"
        if k == "F":
            if a != "fp":
                raise Skip(f"fp arg {a!r}")
            self.need_fp()
            return "fp"
        if k == "p":
            if a in ("NULL", "0", "nullptr"):
                return "core::ptr::null_mut()"
            raise Skip(f"ptr arg {a!r}")
        m = re.fullmatch(r"fp->(\w+(?:\.\w+)*)", a)
        if m:
            self.need_fp()
            cast = {"f": "f32", "i": "i32", "u": "u32"}[k]
            return f"(fp.{m.group(1)} as {cast})" if k != "f" else f"fp.{m.group(1)}"
        return self.tu.const(a, k)

    def need_fp(self):
        if not self.has_fp:
            raise Skip("fp used before GET_FIGHTER")

    def call(self, text):
        m = re.fullmatch(r"(\w+)\s*\((.*)\)", text.strip(), re.S)
        if not m or m.group(1) not in CALLS:
            raise Skip(f"call {text.strip()[:40]!r}")
        rs, kinds, ret = CALLS[m.group(1)]
        args = split_args(m.group(2))
        if len(args) != len(kinds):
            raise Skip("arity")
        return f"st::{rs}({', '.join(self.arg(a, k) for a, k in zip(args, kinds))})", ret

    def cond(self, c):
        c = c.strip()
        neg = False
        m = re.fullmatch(r"(.*?)\s*(==|!=)\s*([^=!()]+)", c, re.S)
        if m and m.group(1).count("(") == m.group(1).count(")"):
            e, r = self.call(m.group(1))
            if r == "v":
                raise Skip("void in cond")
            return f"{e} {m.group(2)} {self.tu.const(m.group(3), 'i')}"
        if c.startswith("!"):
            neg, c = True, c[1:].strip()
        e, r = self.call(c)
        if r == "v":
            raise Skip("void in cond")
        return f"{e} {'==' if neg else '!='} 0"

    def block(self, s, ind):
        """translate statements in s; returns remaining text"""
        s = s.strip()
        while s:
            if s.startswith("}"):
                return s
            m = re.match(r"(Fighter\s*\*\s*fp\s*=\s*(GET_FIGHTER\(\s*gobj\s*\)|gobj->user_data|\(Fighter\s*\*\)\s*gobj->user_data)\s*;)", s)
            if m:
                if not self.has_fp:
                    self.has_fp = True
                    self.lines.append(ind + "let fp = unsafe { &mut *((*gobj).user_data as *mut Fighter) };")
                s = s[m.end():].strip(); continue
            m = re.match(r"(fp\s*=\s*GET_FIGHTER\(\s*gobj\s*\)\s*;)", s)
            if m and self.has_fp:
                s = s[m.end():].strip(); continue
            m = re.match(r"(PAD_STACK\(\s*\w+\s*\)\s*;|u8\s+_\s*\[\s*\w+\s*\]\s*;|;)", s)
            if m:
                s = s[m.end():].strip(); continue
            m = re.match(r"return\s*;", s)
            if m:
                self.lines.append(ind + "return;")
                s = s[m.end():].strip(); continue
            if re.match(r"if\s*\(", s):
                i, d = s.index("(") + 1, 1
                j = i
                while d:
                    d += s[j] == "("; d -= s[j] == ")"; j += 1
                c = self.cond(s[i:j - 1])
                s = s[j:].strip()
                self.lines.append(ind + f"if {c} {{")
                s = self.sub(s, ind)
                if re.match(r"else\b", s):
                    s = s[4:].strip()
                    self.lines.append(ind + "} else {")
                    if re.match(r"if\b", s):
                        raise Skip("else if")
                    s = self.sub(s, ind)
                self.lines.append(ind + "}")
                continue
            m = re.match(r"([^;{}]*);", s)
            if m:
                e, r = self.call(m.group(1))
                self.lines.append(ind + (f"let _ = {e};" if r != "v" else f"{e};"))
                s = s[m.end():].strip(); continue
            raise Skip(f"stmt {s[:40]!r}")
        return s

    def sub(self, s, ind):
        if s.startswith("{"):
            s = self.block(s[1:], ind + "    ")
            if not s.startswith("}"):
                raise Skip("brace")
            return s[1:].strip()
        m = re.match(r"([^;{}]*;)", s)
        if not m:
            raise Skip("if body")
        rest = self.block(m.group(1), ind + "    ")
        if rest:
            raise Skip("if body")
        return s[m.end():].strip()


def scan():
    ported = set()
    for line in (HERE / "ported.txt").read_text().split("\n"):
        if line.strip() and not line.startswith("#"):
            ported.add(line.split()[1])
    tus = {}
    for f in sorted((ROOT / "src/melee/ft/kinds").glob("*/*.c")):
        if f.parent.name == "ftCommon":
            continue
        src = f.read_text(encoding="utf-8", errors="replace")
        rel = f.relative_to(ROOT).as_posix()
        tu = TU(rel)
        fns = []
        for m in re.finditer(r"^(static\s+)?(?:inline\s+)?void\s+(\w+)\s*\(\s*(?:HSD_GObj|Fighter_GObj)\s*\*\s*gobj\s*\)\s*\{", src, re.M):
            name = m.group(2)
            if name in ported:
                continue
            i, d = m.end(), 1
            while d and i < len(src):
                d += src[i] == "{"; d -= src[i] == "}"; i += 1
            try:
                body = strip(src[m.end():i - 1])
                fn = Fn(tu, name, body)
                fn.static = bool(m.group(1))
                rest = fn.block(body, "    ")
                if rest:
                    raise Skip("trailing")
                if not any("st::" in l for l in fn.lines):
                    raise Skip("no call")
                fns.append(fn)
            except Skip:
                continue
        if fns:
            tus[rel] = (tu, fns)
    return tus


def eval_consts(tus):
    BUILD.mkdir(parents=True, exist_ok=True)
    for ti, (rel, (tu, fns)) in enumerate(sorted(tus.items())):
        if not tu.consts:
            tu.values = []
            continue
        c = [f'#include "{(ROOT / rel).as_posix()}"', "int printf(const char*, ...);",
             "int main(void) {"]
        for e, k in tu.consts:
            if k == "f":
                c.append(f"    {{ float v = (float) ({e}); printf(\"%08X\\n\", *(unsigned*) &v); }}")
            else:
                c.append(f"    printf(\"%08X\\n\", (unsigned) ({e}));")
        c.append("    return 0;\n}")
        src = BUILD / f"c{ti}.c"
        src.write_text("\n".join(c) + "\n", encoding="utf-8")
        exe = BUILD / f"c{ti}.exe"
        cmd = [chref.CLANG, *[x if x != str(chref.BUILD) else str(chref.BUILD) for x in chref.COMMON], "-O0",
               str(src), "-o", str(exe), "-fuse-ld=lld", "-Wl,/FORCE:UNRESOLVED", "-Wl,/IGNORE:4006",
               "-Wl,/IGNORE:4088", str(chref.BUILD / "gekko_fp.obj")]
        r = subprocess.run(cmd, cwd=ROOT, capture_output=True, text=True)
        if r.returncode:
            sys.stderr.write(r.stdout + r.stderr)
            raise SystemExit(f"const eval failed for {rel}")
        out = subprocess.run([str(exe)], capture_output=True, text=True, check=True).stdout.split()
        tu.values = [int(x, 16) for x in out]


def rust_const(tu, idx):
    e, k = tu.consts[idx]
    v = tu.values[idx]
    sp = " ".join(e.split())
    if k == "f":
        lit = re.fullmatch(r"(-?\d+(?:\.\d*)?)[fF]?", sp)
        if lit and struct.unpack("<I", struct.pack("<f", float(lit.group(1))))[0] == v:
            x = struct.unpack("<f", struct.pack("<I", v))[0]
            return f"{x!r}_f32"
        return f"f32::from_bits(0x{v:08X}) /* {sp} */"
    if k == "u":
        return f"0x{v:X}u32 /* {sp} */" if not sp.isdigit() else f"{v}u32"
    s = v - (1 << 32) if v & 0x80000000 else v
    return f"{s} /* {sp} */" if not re.fullmatch(r"-?\d+", sp) else f"{s}"


def emit(tus):
    by_char = {}
    for rel, (tu, fns) in sorted(tus.items()):
        by_char.setdefault(Path(rel).parent.name, []).append((rel, tu, fns))
    outdir = CRATE / "src/chars/r2"
    outdir.mkdir(parents=True, exist_ok=True)
    listing = ["# Round-2 character functions (gen/port_r2.py output): <tu> <function>, checked by gen/chref.py"]
    for ch, items in sorted(by_char.items()):
        rs = [f"//! @generated by rust/melee_chara/gen/port_r2.py: round-2 thin callbacks of {ch}.",
              "//! Re-exported from the TU modules in `chars::" + ch + "`. Do not edit by hand: change",
              "//! the translator or move a function to the hand-written module.",
              "#![allow(non_snake_case, unused_unsafe, unused_variables, clippy::all)]", ""]
        for rel, tu, fns in items:
            stem = Path(rel).stem
            rs.append(f"pub mod {stem} {{")
            rs.append("    #[allow(unused_imports)]\n    use crate::stubs as st;\n    #[allow(unused_imports)]\n    use melee_types::{Fighter, HSD_GObj};\n")
            for fn in fns:
                rs.append(f"    /// `{fn.name}`{' (static)' if fn.static else ''} ({rel}).")
                rs.append(f"    pub fn {fn.name}(gobj: *mut HSD_GObj) {{")
                for l in fn.lines:
                    l = re.sub(r"@C(\d+)@", lambda m: rust_const(tu, int(m.group(1))), l)
                    rs.append("    " + l)
                rs.append("    }\n")
                listing.append(f"{rel} {fn.name}")
            rs.append("}\n")
        (outdir / f"{ch}.rs").write_text("\n".join(rs), encoding="utf-8", newline="\n")
    mods = ["//! @generated by gen/port_r2.py: round-2 translated character callbacks, one file per character.",
            *[f"pub mod {ch};" for ch in sorted(by_char)], ""]
    (outdir / "mod.rs").write_text("\n".join(mods), encoding="utf-8", newline="\n")
    (HERE / "ported_r2.txt").write_text("\n".join(listing) + "\n", encoding="utf-8", newline="\n")
    return by_char


def main():
    tus = scan()
    eval_consts(tus)
    by_char = emit(tus)
    n = sum(len(f) for _, f in tus.values())
    print(f"port_r2: {n} functions in {len(tus)} TUs of {len(by_char)} characters")


if __name__ == "__main__":
    main()
