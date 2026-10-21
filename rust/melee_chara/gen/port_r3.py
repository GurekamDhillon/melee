#!/usr/bin/env python3
"""Round-3 porting aid: port_r2.py's mechanical C -> Rust translation, with a wider subset.

  python rust/melee_chara/gen/port_r3.py      (from the worktree root; Windows, LLVM clang, cargo)

Adds to the round-2 subset (see port_r2.py):
  - calls to round-1 ports that take only the fighter (`f(gobj)` in C, `f(&mut Fighter)` in Rust),
    to round-2 ports and to functions accepted earlier in this round (fixpoint over the scan);
  - `fp->a.b = <constant>;` and `fp->a.b = fp->c.d;` (scalar fields of the same C type), and
    `fp->a.b += / -= <constant>;` (integer fields wrap like the C under -fwrapv; float fields only
    when the constant itself is `float`, so no double promotion is involved);
  - functions made only of such statements (no call needed).
Every field's C type and every constant (cast to the field type) is evaluated by clang from the
TU itself; a field clang cannot take sizeof of (bitfields) or that is not a scalar is skipped.
The emitted Rust is then built; a function rustc rejects (Rust field name differs, e.g. anonymous
members, or a different field type) is dropped and the scan repeated, so what remains compiles
as written. `fp` is a raw `*mut Fighter` here (the stubs see the same fighter through the gobj).
Output: src/chars/r3/<Char>.rs and gen/ported_r3.txt; verified by gen/chref.py like rounds 1-2.
Computed from C source only.
"""
import re, struct, subprocess, sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import chref  # noqa: E402
import port_r2 as r2  # noqa: E402
from port_r2 import Skip, split_args, strip  # noqa: E402

ROOT, CRATE = r2.ROOT, r2.CRATE
BUILD = ROOT / "rust/target/port_r3"
OUT = CRATE / "src/chars/r3"

# C type code (probe) -> Rust scalar type
KINDS = {1: "f32", 3: "i8", 4: "u8", 5: "i16", 6: "u16", 7: "i32", 8: "u32", 9: "i64", 10: "u64"}
GENERIC = ("_Generic(X, float:1, double:2, signed char:3, char:3, unsigned char:4, short:5, "
           "unsigned short:6, int:7, unsigned int:8, long long:9, unsigned long long:10, default:0)")
FIELD = r"fp->(\w+(?:(?:\.|->)\w+)*)"


def read_list(name):
    out = []
    for line in (HERE / name).read_text().split("\n"):
        if line.strip() and not line.startswith("#"):
            out.append(line.split())
    return out


def known_calls(extra):
    """name -> (rust expr template with {g}/{f}, ret kind)"""
    calls = {}
    for tu, name in read_list("ported.txt"):
        src = (ROOT / tu).read_text(encoding="utf-8")
        try:
            rk, kinds = chref.signature(src, name)
        except Exception:
            continue
        if kinds != ["G"] or rk == "f":
            continue
        d, stem = Path(tu).parent.name, Path(tu).stem
        if rk == "i":  # round-1 returns bool / i32; normalise to i32
            calls[name] = (f"(crate::chars::{d}::{stem}::{name}(&mut *fp) as i32)", "i", True)
        else:
            calls[name] = (f"crate::chars::{d}::{stem}::{name}(&mut *fp)", "v", True)
    for tu, name in read_list("ported_r2.txt"):
        d, stem = Path(tu).parent.name, Path(tu).stem
        calls[name] = (f"crate::chars::r2::{d}::{stem}::{name}(gobj)", "v", False)
    for (tu, name) in extra:
        d, stem = Path(tu).parent.name, Path(tu).stem
        calls[(tu, name)] = (f"crate::chars::r3::{d}::{stem}::{name}(gobj)", "v", False)
    return calls


class TU3(r2.TU):
    def __init__(self, path):
        super().__init__(path)
        self.assigns = []  # (lhs, op, rhs, rhs_is_field)
        self.src = (ROOT / path).read_text(encoding="utf-8", errors="replace")

    def src_has_static(self, name):
        return re.search(r"^static[^;{]*" + name + r"\s*\(", self.src, re.M) is not None

    def assign(self, lhs, op, rhs, is_field):
        key = (lhs, op, rhs, is_field)
        if key not in self.assigns:
            self.assigns.append(key)
        return "@A%d@" % self.assigns.index(key)


class Fn3(r2.Fn):
    CALLS3 = {}

    def need_fp(self):
        if not self.has_fp:
            raise Skip("fp used before GET_FIGHTER")

    def arg(self, a, k):
        a = a.strip()
        if k == "F":
            if a != "fp":
                raise Skip("fp arg")
            self.need_fp()
            return "&mut *fp"
        if re.fullmatch(FIELD, a) and k in "fiu":
            self.need_fp()
            m = re.fullmatch(FIELD, a)
            p = m.group(1).replace("->", ".")
            if "->" in m.group(1):
                raise Skip("deref arg")
            cast = {"f": "f32", "i": "i32", "u": "u32"}[k]
            return f"((*fp).{p} as {cast})" if k != "f" else f"(*fp).{p}"
        return super().arg(a, k)

    def call(self, text):
        m = re.fullmatch(r"(\w+)\s*\((.*)\)", text.strip(), re.S)
        key = None
        if m and m.group(1) not in r2.CALLS:
            # a static of this TU wins over a same-named function of another TU
            if (self.tu.path, m.group(1)) in self.CALLS3:
                key = (self.tu.path, m.group(1))
            elif m.group(1) in self.CALLS3:
                key = m.group(1)
            else:
                others = [k for k in self.CALLS3 if isinstance(k, tuple) and k[1] == m.group(1)]
                if len(others) == 1 and not self.tu.src_has_static(m.group(1)):
                    key = others[0]
        if key:
            tmpl, ret, needs_fp = self.CALLS3[key]
            if split_args(m.group(2)) != ["gobj"]:
                raise Skip("r1/r2 call args")
            if needs_fp:
                self.need_fp()
            self.used_port = True
            return tmpl, ret
        return super().call(text)

    def block(self, s, ind):
        s = s.strip()
        while s:
            if s.startswith("}"):
                return s
            m = re.match(r"(Fighter\s*\*\s*fp\s*=\s*(GET_FIGHTER\(\s*gobj\s*\)|gobj->user_data|\(Fighter\s*\*\)\s*gobj->user_data)\s*;)", s)
            if m:
                if not self.has_fp:
                    self.has_fp = True
                    self.lines.append(ind + "let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;")
                s = s[m.end():].strip(); continue
            m = re.match(FIELD + r"\s*(=|\+=|-=)\s*([^;=]+);", s)
            if m:
                self.need_fp()
                lhs, op, rhs = m.group(1), m.group(2), m.group(3).strip()
                if "->" in lhs:
                    raise Skip("deref lhs")
                fm = re.fullmatch(FIELD, rhs)
                if fm:
                    if op != "=" or "->" in fm.group(1):
                        raise Skip("field rhs")
                    tok = self.tu.assign(lhs, op, fm.group(1), True)
                else:
                    if re.search(r"(fp|gobj)|->|\[", rhs) or re.search(r"(?!sizeof)\w+\s*\(", rhs):
                        raise Skip("non-constant rhs")
                    tok = self.tu.assign(lhs, op, rhs, False)
                self.lines.append(ind + tok)
                self.assigned = True
                s = s[m.end():].strip(); continue
            m = re.match(r"fp\s*=\s*GET_FIGHTER\(\s*gobj\s*\)\s*;", s)
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


def scan(drop, extra):
    ported = {n for _, n in read_list("ported.txt")} | {n for _, n in read_list("ported_r2.txt")}
    Fn3.CALLS3 = known_calls(extra)
    tus = {}
    for f in sorted((ROOT / "src/melee/ft/kinds").glob("*/*.c")):
        if f.parent.name == "ftCommon":
            continue
        src = f.read_text(encoding="utf-8", errors="replace")
        rel = f.relative_to(ROOT).as_posix()
        tu = TU3(rel)
        fns = []
        for m in re.finditer(r"^(static\s+)?(?:inline\s+)?void\s+(\w+)\s*\(\s*(?:HSD_GObj|Fighter_GObj)\s*\*\s*gobj\s*\)\s*\{", src, re.M):
            name = m.group(2)
            if name in ported or name in drop:
                continue
            i, d = m.end(), 1
            while d and i < len(src):
                d += src[i] == "{"; d -= src[i] == "}"; i += 1
            try:
                body = strip(src[m.end():i - 1])
                fn = Fn3(tu, name, body)
                fn.static = bool(m.group(1))
                fn.used_port = fn.assigned = False
                rest = fn.block(body, "    ")
                if rest:
                    raise Skip("trailing")
                if not (fn.used_port or fn.assigned):
                    raise Skip("nothing new for round 3")
                if not any(l.strip() not in ("return;",) and "let fp" not in l for l in fn.lines):
                    raise Skip("empty")
                fns.append(fn)
            except Skip:
                continue
        if fns:
            tus[rel] = (tu, fns)
    return tus


def probe(tus):
    """clang: type code / size of each lhs (and rhs field), constant bits cast to the lhs type."""
    BUILD.mkdir(parents=True, exist_ok=True)
    for ti, (rel, (tu, fns)) in enumerate(sorted(tus.items())):
        tu.ainfo = {}
        bad = set()
        while True:
            c = [f'#include "{(ROOT / rel).as_posix()}"', "int printf(const char*, ...);",
                 "#define FP ((Fighter*) 0)"]
            lines = {}
            for i, (lhs, op, rhs, isf) in enumerate(tu.assigns):
                if i in bad or isf:
                    continue
                L = f"FP->{lhs}"
                # file-scope static: clang rejects anything that is not a constant initializer
                lines[len(c) + 1] = i
                c.append(f"static __typeof__({L}) v_{i} = (__typeof__({L})) ({rhs});")
            c.append("int main(void) {")
            for i, (lhs, op, rhs, isf) in enumerate(tu.assigns):
                if i in bad:
                    continue
                L = f"FP->{lhs}"
                g = GENERIC.replace("X", f"(*(__typeof__({L})*) 0)")
                if isf:
                    R = f"FP->{rhs}"
                    gr = GENERIC.replace("X", f"(*(__typeof__({R})*) 0)")
                    stmt = (f'{{ (void) sizeof({L}); (void) sizeof({R}); printf("{i} %d %d %d 0\\n", '
                            f"{g}, (int) sizeof({L}), {gr}); }}")
                else:
                    gr = GENERIC.replace("X", f"({rhs})")
                    stmt = (f"{{ __typeof__({L}) v = v_{i}; unsigned long long b = 0; "
                            f'__builtin_memcpy(&b, &v, sizeof v); printf("{i} %d %d %d %llX\\n", {g}, '
                            f"(int) sizeof({L}), {gr}, b); }}")
                    stmt = stmt.replace(f"printf(\"{i} %d", f"printf(\"{i} %d", 1).replace(
                        f", {g},", f", __builtin_classify_type(v) == 5 ? 11 : {g},", 1)
                lines[len(c) + 1] = i
                c.append("    " + stmt)
            c.append("    return 0;\n}")
            src = BUILD / f"p{ti}.c"
            src.write_text("\n".join(c) + "\n", encoding="utf-8")
            exe = BUILD / f"p{ti}.exe"
            cmd = [chref.CLANG, *chref.COMMON, "-O0", str(src), "-o", str(exe), "-fuse-ld=lld",
                   "-Wl,/FORCE:UNRESOLVED", "-Wl,/IGNORE:4006", "-Wl,/IGNORE:4088",
                   "-Xclang", "-fno-caret-diagnostics", "-ferror-limit=0"]
            r = subprocess.run(cmd, cwd=ROOT, capture_output=True, text=True)
            if r.returncode:
                errs = {int(m.group(1)) for m in re.finditer(r"p%d\.c:(\d+):\d+: error" % ti, r.stderr)}
                newbad = {lines[e] for e in errs if e in lines}
                if not newbad:
                    sys.stderr.write(r.stderr[-3000:])
                    raise SystemExit(f"probe failed for {rel}")
                bad |= newbad
                continue
            break
        out = subprocess.run([str(exe)], capture_output=True, text=True, check=True).stdout
        for line in out.split("\n"):
            if line.strip():
                i, k, sz, rk, bits = line.split()
                tu.ainfo[int(i)] = (int(k), int(sz), int(rk), int(bits, 16))


BITFIELDS = set()
CALLBACKS = set()  # (setter tu, defining tu or "ext", function name)
# Fighter's top-level HSD_GObjEvent fields (the test maps their values back to callback names)
CB_FIELDS = ["grab_cb", "input_cb", "anim_cb", "phys_cb", "coll_cb", "cam_cb", "accessory1_cb",
             "accessory2_cb", "accessory3_cb", "accessory4_cb", "deal_dmg_cb", "shield_hit_cb",
             "reflect_hit_cb", "hitlag_cb", "pre_hitlag_cb", "post_hitlag_cb", "take_dmg_cb",
             "death1_cb", "death2_cb", "death3_cb", "take_dmg_2_cb", "hurtbox_detect_cb"]


def cb_name(key, ident):
    return f"{'ext' if key == 'ext' else Path(key).stem}__{ident}"


_DEF = {}


def def_tu(setter, ident):
    """the TU defining a callback (its key): the setter's own TU (statics) first, then any
    character TU with a global definition, else "ext" (defined outside ft/kinds)."""
    k = (setter, ident)
    if k not in _DEF:
        if defines(setter, ident):
            _DEF[k] = setter
        else:
            hits = [f.relative_to(ROOT).as_posix() for f in sorted((ROOT / "src/melee/ft/kinds").glob("*/*.c"))
                    if defines(f.relative_to(ROOT).as_posix(), ident, static=False)]
            _DEF[k] = hits[0] if len(hits) == 1 else "ext"
    return _DEF[k]


def defines(tu, ident, static=True):
    src = (ROOT / tu).read_text(encoding="utf-8", errors="replace")
    return re.search(r"^" + (r"(?:static\s+)?" if static else "") + r"(?:inline\s+)?void\s+" + ident + r"\s*\([^;]*\)\s*\{", src, re.M) is not None


def write_cb(r3list):
    """src/chars/cb.rs: one `extern "C"` entry per (setter TU, callback) the ports store, calling
    the port of the target when there is one (same TU first: statics), else TODO(group4)."""
    ported = {}
    for tu, name in read_list("ported.txt"):
        src = (ROOT / tu).read_text(encoding="utf-8")
        try:
            rk, kinds = chref.signature(src, name)
        except Exception:
            continue
        if kinds == ["G"]:
            d, stem = Path(tu).parent.name, Path(tu).stem
            ported.setdefault(name, {})[tu] = f"let _ = crate::chars::{d}::{stem}::{name}(&mut *((*g).user_data as *mut Fighter));"
    for lst, mod in ((read_list("ported_r2.txt"), "r2"), (r3list, "r3")):
        for tu, name in lst:
            d, stem = Path(tu).parent.name, Path(tu).stem
            ported.setdefault(name, {})[tu] = f"crate::chars::{mod}::{d}::{stem}::{name}(g);"
    rs = ["//! @generated by gen/port_r3.py: the callbacks the ported character functions store in",
          "//! Fighter's HSD_GObjEvent fields, one `extern \"C\"` entry per (setter TU, C function).",
          "//! Each calls the Rust port of its target, or panics TODO(group4) until that is ported.",
          "#![allow(non_snake_case, unused_unsafe, unreachable_code, unused_variables, unused_imports)]",
          "use melee_types::{Fighter, HSD_GObj};", ""]
    table = []
    out = []
    for tu, ident in sorted({(k, i) for _, k, i in CALLBACKS}):
        cands = ported.get(ident, {})
        if tu in cands:
            body = cands[tu]
        elif tu == "ext":
            body = f'panic!("TODO(group3): {ident} (outside ft/kinds) is not ported");'
        else:
            body = f'panic!("TODO(group4): {ident} ({tu}) is not ported");'
        rs += [f"/// `{ident}` (defined in {tu}).",
               f"pub unsafe extern \"C\" fn {cb_name(tu, ident)}(g: *mut HSD_GObj) {{",
               f"    {body}", "}", ""]
        table.append(f'    ({cb_name(tu, ident)}, "{tu}:{ident}"),')
    for setter, tu, ident in sorted(CALLBACKS):
        out.append(f"{setter} {tu} {ident}")
    rs += ["/// Every entry with its `<setter tu>:<C name>` key (tests/chars_ref.rs maps stored pointers back).",
           "pub const CALLBACKS: &[(unsafe extern \"C\" fn(*mut HSD_GObj), &str)] = &[", *table, "];", ""]
    (CRATE / "src/chars/cb.rs").write_text(chr(10).join(rs), encoding="utf-8", newline=chr(10))
    (HERE / "callbacks_r3.txt").write_text("# <setter tu> <defining tu | ext> <callback> stored by round-3 ports: gen/port_r3.py" + chr(10)
                                           + chr(10).join(out) + chr(10), encoding="utf-8", newline=chr(10))


def rust_assign(tu, idx):
    lhs, op, rhs, isf = tu.assigns[idx]
    if idx not in tu.ainfo:
        # clang could not take sizeof: a bitfield. Only Fighter-level one-bit flags set to a
        # boolean constant (`fp->x2223_b4 = true;`); the test compares them by value (BITFIELDS).
        v = {"true": 1, "false": 0, "1": 1, "0": 0}.get(rhs.strip())
        if isf or op != "=" or v is None or not re.fullmatch(r"\w+", lhs):
            raise Skip("probe")
        BITFIELDS.add(lhs)
        return f"(*fp).set_{lhs}({v});"
    k, sz, rk, bits = tu.ainfo[idx]
    if k == 11 and not isf and op == "=" and re.fullmatch(r"[A-Za-z_]\w*", rhs.strip()) and rhs.strip() != "NULL":
        # a callback: `fp->accessory4_cb = ftFx_SpecialHi_CreateLaunchGFX;` -> crate::chars::cb
        if lhs not in CB_FIELDS:
            raise Skip("non-GObjEvent pointer")
        key = def_tu(tu.path, rhs.strip())
        CALLBACKS.add((tu.path, key, rhs.strip()))
        return f"(*fp).{lhs} = Some(crate::chars::cb::{cb_name(key, rhs.strip())});"
    if k == 11 and not isf and op == "=" and bits == 0:
        return f"(*fp).{lhs} = core::mem::zeroed(); /* {' '.join(rhs.split())} */"
    if k not in KINDS:
        raise Skip("type")
    ty = KINDS[k]
    L = "(*fp)." + lhs
    sp = " ".join(rhs.split())
    if isf:
        if rk != k:
            raise Skip("type mismatch")
        return f"{L} = (*fp).{rhs};"
    if ty == "f32":
        if op != "=" and rk != 1:
            raise Skip("double promotion")
        val = f"f32::from_bits(0x{bits & 0xFFFFFFFF:08X}) /* {sp} */"
        return {"=": f"{L} = {val};", "+=": f"{L} += {val};", "-=": f"{L} -= {val};"}[op]
    width = sz * 8
    bits &= (1 << width) - 1
    if ty.startswith("i") and bits >> (width - 1):
        v = bits - (1 << width)
    else:
        v = bits
    val = f"{v}_{ty}" if re.fullmatch(r"-?\d+", sp) else f"({v}_{ty} /* {sp} */)"
    if op == "=":
        return f"{L} = {val};"
    return f"{L} = {L}.{'wrapping_add' if op == '+=' else 'wrapping_sub'}({val});"


def emit(tus):
    by_char = {}
    for rel, (tu, fns) in sorted(tus.items()):
        by_char.setdefault(Path(rel).parent.name, []).append((rel, tu, fns))
    if OUT.exists():
        for p in OUT.glob("*.rs"):
            p.unlink()
    OUT.mkdir(parents=True, exist_ok=True)
    listing = ["# Round-3 character functions (gen/port_r3.py output): <tu> <function>, checked by gen/chref.py"]
    linemap = {}  # (file, line) -> fn name
    BITFIELDS.clear()
    CALLBACKS.clear()
    dropped = set()
    for ch, items in sorted(by_char.items()):
        rs = [f"//! @generated by rust/melee_chara/gen/port_r3.py: round-3 callbacks of {ch}.",
              "//! Do not edit by hand: change the translator or move a function to a hand-written module.",
              "#![allow(non_snake_case, unused_unsafe, unused_variables, unused_parens, clippy::all)]", ""]
        for rel, tu, fns in items:
            body = []
            for fn in fns:
                try:
                    out = []
                    for l in fn.lines:
                        l = re.sub(r"@C(\d+)@", lambda m: r2.rust_const(tu, int(m.group(1))), l)
                        m = re.search(r"@A(\d+)@", l)
                        if m:
                            l = l[:m.start()] + rust_assign(tu, int(m.group(1)))
                        l = l.replace("st::ftCommon_GroundToAirStateChange(gobj, fp,", "st::ftCommon_GroundToAirStateChange(gobj, &mut *fp,")
                        l = l.replace("st::ftCommon_AirToGroundStateChange(gobj, fp,", "st::ftCommon_AirToGroundStateChange(gobj, &mut *fp,")
                        out.append(l)
                except Skip:
                    dropped.add(fn.name)
                    continue
                body.append((fn, out))
            if not body:
                continue
            stem = Path(rel).stem
            rs.append(f"pub mod {stem} {{")
            rs += ["    #[allow(unused_imports)]", "    use crate::stubs as st;", "    #[allow(unused_imports)]", "    use melee_types::{Fighter, HSD_GObj};", ""]
            for fn, out in body:
                rs.append(f"    /// `{fn.name}`{' (static)' if fn.static else ''} ({rel}).")
                rs.append(f"    pub fn {fn.name}(gobj: *mut HSD_GObj) {{")
                rs.append("        unsafe {")
                for l in out:
                    rs.append("        " + l)
                    linemap[(ch, len(rs))] = fn.name
                rs.append("        }")
                rs += ["    }", ""]
                listing.append(f"{rel} {fn.name}")
            rs += ["}", ""]
        (OUT / f"{ch}.rs").write_text("\n".join(rs), encoding="utf-8", newline="\n")
    mods = ["//! @generated by gen/port_r3.py: round-3 translated character callbacks, one file per character.",
            *[f"pub mod {p.stem};" for p in sorted(OUT.glob("*.rs")) if p.stem != "mod"], ""]
    (OUT / "mod.rs").write_text("\n".join(mods), encoding="utf-8", newline="\n")
    write_cb([l.split() for l in listing[1:]])
    (HERE / "ported_r3.txt").write_text("\n".join(listing) + "\n", encoding="utf-8", newline="\n")
    return linemap, dropped, len(listing) - 1


def cargo_errors(linemap):
    r = subprocess.run(["cargo", "build", "-p", "melee_chara", "--message-format=short"],
                       cwd=ROOT / "rust", capture_output=True, text=True)
    bad = set()
    for m in re.finditer(r"src[\\/]chars[\\/]r3[\\/](\w+)\.rs:(\d+):\d+: error", r.stderr):
        ln = int(m.group(2))
        name = linemap.get((m.group(1), ln))
        if name is None:  # error on the fn line or brace: nearest following body line
            cands = [n for (c, l), n in linemap.items() if c == m.group(1) and l >= ln]
            name = cands[0] if cands else None
        if name:
            bad.add(name)
    if r.returncode and not bad:
        sys.stderr.write(r.stderr[-4000:])
        raise SystemExit("cargo build failed outside round-3 functions")
    return bad


def main():
    drop, extra = set(), set()
    while True:
        tus = scan(drop, extra)
        names = {(rel, fn.name) for rel, (tu, fns) in tus.items() for fn in fns}
        if names - extra:  # new callers became possible: rescan with them callable
            extra |= names
            continue
        r2.eval_consts(tus)
        probe(tus)
        linemap, dropped, n = emit(tus)
        bad = dropped | cargo_errors(linemap)
        if not bad:
            break
        print(f"port_r3: dropping {len(bad)}: {sorted(bad)[:8]}...", flush=True)
        drop |= bad
        extra = {k for k in extra if k[1] not in drop}
    print(f"port_r3: {n} functions in {len(tus)} TUs")
    print("port_r3: bitfields written (add to the by-value list in chref_main.c / chars_ref.rs):",
          " ".join(sorted(BITFIELDS)))


if __name__ == "__main__":
    main()
