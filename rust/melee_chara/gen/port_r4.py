#!/usr/bin/env python3
"""Round-4 porting aid: port_r3.py's mechanical C -> Rust translation, with conditions on fields.

  python rust/melee_chara/gen/port_r4.py      (from the worktree root; Windows, LLVM clang, cargo)

Adds to the round-3 subset (see port_r3.py / port_r2.py):
  - `if` conditions on Fighter fields: `fp->a.b OP <constant>` (OP one of == != < > <= >=),
    `fp->a.b` and `!fp->a.b`, combined with `&&` / `||` (and parentheses) with each other and
    with the call conditions of round 2;
  - `else if` chains;
  - calls to round-3 ports and to functions accepted earlier in this round (fixpoint).
Each comparison follows C's usual arithmetic conversions, decided from the types clang reports
for the TU: int-or-narrower fields against an int constant compare as i32, anything involving
`unsigned int` as u32 (the field converted the C way, `as u32`), f32 fields against a float
constant as f32, against a double constant as f64 (the field widened), against an int constant
as f32 (the constant converted by clang). The constant's bits are evaluated by clang in its own
type. Bitfields are accepted only where the test compares them by value (KNOWN_BF, the list in
gen/chref_main.c / tests/chars_ref.rs). Stored callbacks get chars::cb entries as in round 3
(new keys in callbacks_r4.txt). Output: src/chars/r4/<Char>.rs and gen/ported_r4.txt; verified by
gen/chref.py like rounds 1-3. Computed from C source only.
"""
import re, struct, subprocess, sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import chref  # noqa: E402
import port_r2 as r2  # noqa: E402
import port_r3 as r3  # noqa: E402
from port_r2 import Skip, split_args, strip  # noqa: E402

ROOT, CRATE = r2.ROOT, r2.CRATE
BUILD = ROOT / "rust/target/port_r4"
OUT = CRATE / "src/chars/r4"
# round 3's field pattern plus constant array indices (`fp->cmd_vars[0]`); port_r3 reads its
# module global at call time, so the wider pattern applies to its statements too
FIELD = r"fp->(\w+(?:\[\d+\])?(?:(?:\.|->)\w+(?:\[\d+\])?)*)"
r3.FIELD = FIELD
# Fighter-level one-bit bitfields the oracle compares by value (gen/chref_main.c bf_get/bf_set)
KNOWN_BF = {"x2218_b4", "fall_fast", "x2219_b5", "x221F_b3", "x2223_b4", "is_sub_fighter", "x2227_b0",
            "allow_interrupt", "invisible", "reflecting"}
OPS = ("==", "!=", "<=", ">=", "<", ">")


def known_calls(extra):
    calls = r3.known_calls(set())
    for tu, name in r3.read_list("ported_r3.txt"):
        d, stem = Path(tu).parent.name, Path(tu).stem
        calls[(tu, name)] = (f"crate::chars::r3::{d}::{stem}::{name}(gobj)", "v", False)
    for (tu, name) in extra:
        d, stem = Path(tu).parent.name, Path(tu).stem
        calls[(tu, name)] = (f"crate::chars::r4::{d}::{stem}::{name}(gobj)", "v", False)
    return calls


class TU4(r3.TU3):
    def __init__(self, path):
        super().__init__(path)
        self.cmps = []  # (field, op, rhs)
        self.da_types = []  # dat_attrs struct types of `T* da = fp->dat_attrs;` locals

    def cmp(self, field, op, rhs):
        key = (field, op, rhs)
        if key not in self.cmps:
            self.cmps.append(key)
        return "@K%d@" % self.cmps.index(key)


def top_split(c, op):
    out, d, cur, i = [], 0, "", 0
    while i < len(c):
        ch = c[i]
        d += ch == "("; d -= ch == ")"
        if d == 0 and c.startswith(op, i):
            out.append(cur); cur = ""; i += len(op); continue
        cur += ch; i += 1
    out.append(cur)
    return [x.strip() for x in out]


def unparen(c):
    c = c.strip()
    while c.startswith("(") and c.endswith(")"):
        d = 0
        for i, ch in enumerate(c):
            d += ch == "("; d -= ch == ")"
            if d == 0 and i < len(c) - 1:
                return c
        c = c[1:-1].strip()
    return c


class Fn4(r3.Fn3):
    def cond(self, c):
        c = unparen(c)
        for op, rop in (("||", "||"), ("&&", "&&")):
            parts = top_split(c, op)
            if len(parts) > 1:
                return "(" + f" {rop} ".join(self.cond(p) for p in parts) + ")"
        m = re.fullmatch(FIELD + r"\s*(==|!=|<=|>=|<|>)\s*([^=!<>&|]+)", c)
        if m:
            return self.fcmp(m.group(1), m.group(2), m.group(3).strip())
        m = re.fullmatch(r"(!\s*)?" + FIELD, c)
        if m:
            return self.fcmp(m.group(2), "==" if m.group(1) else "!=", "0")
        if c.startswith("!") and c[1:].strip().startswith("("):
            return "!" + self.cond(c[1:])
        return "(" + super().cond(c) + ")"

    def fcmp(self, field, op, rhs):
        self.need_fp()
        if "->" in field:
            raise Skip("deref cond")
        if re.search(r"\b(fp|gobj)\b|->|\[", rhs) or re.search(r"(?!sizeof)\b\w+\s*\(", rhs):
            raise Skip("non-constant cmp")
        self.used_port = True
        return self.tu.cmp(field, op, rhs)

    def block(self, s, ind):
        # round 3's block, with `else if` handled by translating the nested if as its own block
        s = s.strip()
        while s:
            if s.startswith("}"):
                return s
            if re.match(r"if\s*\(", s):
                s = self.if_stmt(s, ind)
                continue
            # one non-if statement through round 3's translator
            m = re.match(r"(Fighter\s*\*\s*fp\s*=[^;]*;|[^;{}]*;)", s)
            if not m:
                raise Skip(f"stmt {s[:40]!r}")
            stmt = re.sub(r"(_cb\s*=\s*)&\s*(\w+\s*;)$", lambda q: q.group(1) + q.group(2), m.group(1).strip())
            if stmt != m.group(1).strip():
                s = stmt + s[m.end():]
                m = re.match(r"([^;{}]*;)", s)
            if re.match(r"\s*fp->_da\d+\.[^;=]*[-+*/|&]?=[^=]", m.group(1)):
                raise Skip("write to dat_attrs")
            rest = super().block(m.group(1), ind)
            if rest:
                raise Skip("stmt")
            s = s[m.end():].strip()
        return s

    def if_stmt(self, s, ind):
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
            if re.match(r"if\s*\(", s):
                s = self.if_stmt(s, ind + "    ")
            else:
                s = self.sub(s, ind)
        self.lines.append(ind + "}")
        return s


DA_DECL = (r"(?:const\s+)?(\w+)\s*\*\s*(\w+)\s*=\s*(?:\(\s*\w+\s*\*\s*\)\s*)?"
           r"(?:fp->dat_attrs|getFtSpecialAttrsD?\(\s*fp\s*\))\s*;")


def normalise(body, tu):
    """spellings of `Fighter* fp = GET_FIGHTER(gobj);` the translator does not know, and
    `T* da = fp->dat_attrs;` locals: `da->x` becomes the pseudo field `fp->_daK.x` (K indexes
    tu.da_types), mapped back to `((T*) 0)->x` for clang and `(*((*fp).dat_attrs as *const T)).x`
    in Rust. Only reads are accepted (Fn4.block)."""
    body = norm_fp(body)
    for m in list(re.finditer(DA_DECL, body)):
        ty, name = m.group(1), m.group(2)
        if ty not in tu.da_types:
            tu.da_types.append(ty)
        k = tu.da_types.index(ty)
        body = body.replace(m.group(0), ";", 1)
        body = re.sub(r"\b%s\s*->\s*" % name, f"fp->_da{k}.", body)
        if re.search(r"\b%s\b" % name, body):
            raise Skip("dat_attrs local used otherwise")
    return body


def da_c(tu, text):
    return re.sub(r"FP->_da(\d+)\.", lambda m: f"(({tu.da_types[int(m.group(1))]}*) 0)->", text)


def da_rs(tu, text):
    return re.sub(r"\(\*fp\)\._da(\d+)\.",
                  lambda m: f"(*((*fp).dat_attrs as *const melee_types::{tu.da_types[int(m.group(1))]})).", text)


def norm_fp(body):
    body = re.sub(r"\bgetFighter(?:Plus)?\s*\(\s*gobj\s*\)", "GET_FIGHTER(gobj)", body)
    m = re.search(r"Fighter\s*\*\s*(\w+)\s*=\s*GET_FIGHTER\(\s*gobj\s*\)\s*;", body)
    if m and m.group(1) != "fp" and not re.search(r"\bfp\b", body):
        body = re.sub(r"\b%s\b" % m.group(1), "fp", body)
    m = re.match(r"\s*Fighter\s*\*\s*fp\s*;", body)
    if m:
        rest = body[m.end():]
        a = re.search(r"\bfp\b", rest)
        if a and re.match(r"fp\s*=\s*GET_FIGHTER\(\s*gobj\s*\)\s*;", rest[a.start():]):
            body = rest[:a.start()] + "Fighter* " + rest[a.start():]
    return body


def scan(drop, extra):
    ported = ({n for _, n in r3.read_list("ported.txt")} | {n for _, n in r3.read_list("ported_r2.txt")}
              | {n for _, n in r3.read_list("ported_r3.txt")})
    Fn4.CALLS3 = known_calls(extra)
    tus = {}
    for f in sorted((ROOT / "src/melee/ft/kinds").glob("*/*.c")):
        if f.parent.name == "ftCommon":
            continue
        src = f.read_text(encoding="utf-8", errors="replace")
        rel = f.relative_to(ROOT).as_posix()
        tu = TU4(rel)
        fns = []
        for m in re.finditer(r"^(static\s+)?(?:inline\s+)?void\s+(\w+)\s*\(\s*(?:HSD_GObj|Fighter_GObj)\s*\*\s*gobj\s*\)\s*\{", src, re.M):
            name = m.group(2)
            if name in ported or name in drop:
                continue
            i, d = m.end(), 1
            while d and i < len(src):
                d += src[i] == "{"; d -= src[i] == "}"; i += 1
            try:
                body = normalise(strip(src[m.end():i - 1]), tu)
                fn = Fn4(tu, name, body)
                fn.static = bool(m.group(1))
                fn.used_port = fn.assigned = False
                rest = fn.block(body, "    ")
                if rest:
                    raise Skip("trailing")
                if not (fn.used_port or fn.assigned):
                    raise Skip("nothing new")
                fns.append(fn)
            except Skip:
                continue
        if fns:
            tus[rel] = (tu, fns)
    return tus


def probe_cmps(tus):
    """clang: field type code + size (fails for bitfields), constant type code + bits in its own type."""
    BUILD.mkdir(parents=True, exist_ok=True)
    for ti, (rel, (tu, fns)) in enumerate(sorted(tus.items())):
        tu.cinfo = {}
        if not tu.cmps:
            continue
        bad = set()  # ("f"|"r", i)
        while True:
            c = [f'#include "{(ROOT / rel).as_posix()}"', "int printf(const char*, ...);",
                 "#define FP ((Fighter*) 0)"]
            lines = {}
            for i, (fld, op, rhs) in enumerate(tu.cmps):
                if ("r", i) in bad:
                    continue
                lines[len(c) + 1] = ("r", i)
                c.append(f"static __typeof__({rhs}) r_{i} = ({rhs});")
            c.append("int main(void) {")
            for i, (fld, op, rhs) in enumerate(tu.cmps):
                L = f"FP->{fld}"
                if ("f", i) not in bad:
                    g = r3.GENERIC.replace("X", f"(*(__typeof__({L})*) 0)")
                    lines[len(c) + 1] = ("f", i)
                    c.append(f'    {{ (void) sizeof({L}); printf("f {i} %d %d\\n", {g}, (int) sizeof({L})); }}')
                if ("r", i) not in bad:
                    g = r3.GENERIC.replace("X", f"r_{i}")
                    lines[len(c) + 1] = ("r", i)
                    c.append(f'    {{ unsigned long long b = 0; __builtin_memcpy(&b, &r_{i}, sizeof r_{i}); '
                             f'printf("r {i} %d %d %llX\\n", {g}, (int) sizeof r_{i}, b); }}')
            c.append("    return 0;\n}")
            src = BUILD / f"k{ti}.c"
            src.write_text("\n".join(c) + "\n", encoding="utf-8")
            exe = BUILD / f"k{ti}.exe"
            cmd = [chref.CLANG, *chref.COMMON, "-O0", str(src), "-o", str(exe), "-fuse-ld=lld",
                   "-Wl,/FORCE:UNRESOLVED", "-Wl,/IGNORE:4006", "-Wl,/IGNORE:4088",
                   "-Xclang", "-fno-caret-diagnostics", "-ferror-limit=0"]
            r = subprocess.run(cmd, cwd=ROOT, capture_output=True, text=True)
            if r.returncode:
                errs = {int(m.group(1)) for m in re.finditer(r"k%d\.c:(\d+):\d+: error" % ti, r.stderr)}
                newbad = {lines[e] for e in errs if e in lines}
                if not newbad:
                    sys.stderr.write(r.stderr[-3000:])
                    raise SystemExit(f"cmp probe failed for {rel}")
                bad |= newbad
                continue
            break
        out = subprocess.run([str(exe)], capture_output=True, text=True, check=True).stdout
        for line in out.split("\n"):
            p = line.split()
            if not p:
                continue
            i = int(p[1])
            d = tu.cinfo.setdefault(i, {})
            if p[0] == "f":
                d["f"] = (int(p[2]), int(p[3]))
            else:
                d["r"] = (int(p[2]), int(p[3]), int(p[4], 16))


def probe_assigns(tus):
    """port_r3.probe with the dat_attrs pseudo fields mapped for clang"""
    orig = Path.write_text

    def wt(self, data, *a, **k):
        if self.parent == BUILD and self.name.startswith("p") and hasattr(wt, "tu"):
            data = da_c(wt.tu, data)
        return orig(self, data, *a, **k)
    Path.write_text = wt
    try:
        for rel in sorted(tus):
            wt.tu = tus[rel][0]
            r3.probe({rel: tus[rel]})
    finally:
        Path.write_text = orig


def sval(bits, size, signed):
    bits &= (1 << (8 * size)) - 1
    if signed and bits >> (8 * size - 1):
        return bits - (1 << (8 * size))
    return bits


def rust_cmp(tu, idx):
    fld, op, rhs = tu.cmps[idx]
    info = tu.cinfo.get(idx, {})
    if "r" not in info:
        raise Skip("cmp rhs")
    rk, rsz, rbits = info["r"]
    sp = " ".join(rhs.split()).replace("*/", "* /")
    rint = rk in (3, 4, 5, 6, 7, 8)
    if "f" not in info:  # bitfield: one-bit, promotes to int
        if fld not in KNOWN_BF or not rint:
            raise Skip("bitfield cmp")
        v = sval(rbits, rsz, rk in (3, 5, 7))
        if rk == 8:
            return f"(((*fp).{fld}() as u32) {op} {v & 0xFFFFFFFF}u32 /* {sp} */)"
        return f"(((*fp).{fld}() as i32) {op} {v}i32 /* {sp} */)"
    fk, fsz = info["f"]
    L = f"(*fp).{fld}"
    if fk == 1:
        if rk == 1:
            return f"({L} {op} f32::from_bits(0x{rbits & 0xFFFFFFFF:08X}) /* {sp} */)"
        if rk == 2:
            return f"(({L} as f64) {op} f64::from_bits(0x{rbits:016X}) /* {sp} */)"
        if rint:
            v = sval(rbits, rsz, rk in (3, 5, 7))
            if abs(v) > (1 << 24):
                raise Skip("int->float")
            fb = struct.unpack("<I", struct.pack("<f", float(v)))[0]
            return f"({L} {op} f32::from_bits(0x{fb:08X}) /* {sp} */)"
        raise Skip("cmp type")
    if fk == 0 and fsz == 8 and op in ("==", "!=") and rhs.strip() in ("NULL", "0"):
        return f"({'' if op == '==' else '!'}{L}.is_null())"
    if fk not in (3, 4, 5, 6, 7, 8) or not rint:
        raise Skip("cmp type")
    v = sval(rbits, rsz, rk in (3, 5, 7))
    if fk == 8 or rk == 8:
        return f"(({L} as u32) {op} {v & 0xFFFFFFFF}u32 /* {sp} */)"
    return f"(({L} as i32) {op} {v}i32 /* {sp} */)"


def read_callbacks(name):
    p = HERE / name
    if not p.exists():
        return set()
    return {tuple(l.split()) for l in p.read_text().split("\n") if l.strip() and not l.startswith("#")}


def rust_assign(tu, idx):
    lhs, op, rhs, isf = tu.assigns[idx]
    if idx not in tu.ainfo and lhs not in KNOWN_BF:
        raise Skip("bitfield not compared by value")
    if idx in tu.ainfo and isf and op == "=":
        k, sz, rk, _ = tu.ainfo[idx]
        if k == 0 and rk == 0:  # struct copy (Vec3 ...): rustc checks both sides have one type
            return f"(*fp).{lhs} = (*fp).{rhs};"
    return r3.rust_assign(tu, idx)


def write_cb(r4list, r4cbs):
    """chars/cb.rs over the callbacks of rounds 3 and 4: port_r3.write_cb, then entries whose
    target is a round-4 port call it. The round-4 set goes to callbacks_r4.txt; callbacks_r3.txt
    is kept as round 3 wrote it."""
    r3cbs = read_callbacks("callbacks_r3.txt")
    keep = (HERE / "callbacks_r3.txt").read_bytes()
    r3.CALLBACKS.clear(); r3.CALLBACKS.update(r3cbs | r4cbs)
    r3.write_cb([tuple(x) for x in r3.read_list("ported_r3.txt")])
    (HERE / "callbacks_r3.txt").write_bytes(keep)
    rs = (CRATE / "src/chars/cb.rs").read_text(encoding="utf-8")
    for tu, name in r4list:
        d, stem = Path(tu).parent.name, Path(tu).stem
        key = r3.cb_name(tu, name)
        rs = re.sub(r"(fn %s\(g: \*mut HSD_GObj\) \{\n    )panic!\([^\n]*\);" % re.escape(key),
                    lambda m: m.group(1) + f"crate::chars::r4::{d}::{stem}::{name}(g);", rs)
    rs = rs.replace("@generated by gen/port_r3.py", "@generated by gen/port_r3.py + gen/port_r4.py", 1)
    (CRATE / "src/chars/cb.rs").write_text(rs, encoding="utf-8", newline="\n")
    out = sorted(" ".join(c) for c in r4cbs - r3cbs)
    (HERE / "callbacks_r4.txt").write_text(
        "# <setter tu> <defining tu | ext> <callback> stored by round-4 ports (new ones only): gen/port_r4.py\n"
        + "".join(l + "\n" for l in out), encoding="utf-8", newline="\n")


def emit(tus):
    by_char = {}
    for rel, (tu, fns) in sorted(tus.items()):
        by_char.setdefault(Path(rel).parent.name, []).append((rel, tu, fns))
    if OUT.exists():
        for p in OUT.glob("*.rs"):
            p.unlink()
    OUT.mkdir(parents=True, exist_ok=True)
    listing = ["# Round-4 character functions (gen/port_r4.py output): <tu> <function>, checked by gen/chref.py"]
    linemap, dropped = {}, set()
    r3.CALLBACKS.clear()
    for ch, items in sorted(by_char.items()):
        rs = [f"//! @generated by rust/melee_chara/gen/port_r4.py: round-4 callbacks of {ch}.",
              "//! Do not edit by hand: change the translator or move a function to a hand-written module.",
              "#![allow(non_snake_case, unused_unsafe, unused_variables, unused_parens, unused_comparisons, clippy::all)]", ""]
        for rel, tu, fns in items:
            body = []
            for fn in fns:
                try:
                    out = []
                    for l in fn.lines:
                        l = re.sub(r"@C(\d+)@", lambda m: r2.rust_const(tu, int(m.group(1))), l)
                        l = re.sub(r"@K(\d+)@", lambda m: rust_cmp(tu, int(m.group(1))), l)
                        m = re.search(r"@A(\d+)@", l)
                        if m:
                            l = l[:m.start()] + rust_assign(tu, int(m.group(1)))
                        l = l.replace("st::ftCommon_GroundToAirStateChange(gobj, fp,", "st::ftCommon_GroundToAirStateChange(gobj, &mut *fp,")
                        l = l.replace("st::ftCommon_AirToGroundStateChange(gobj, fp,", "st::ftCommon_AirToGroundStateChange(gobj, &mut *fp,")
                        out.append(da_rs(tu, l))
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
    mods = ["//! @generated by gen/port_r4.py: round-4 translated character callbacks, one file per character.",
            *[f"pub mod {p.stem};" for p in sorted(OUT.glob("*.rs")) if p.stem != "mod"], ""]
    (OUT / "mod.rs").write_text("\n".join(mods), encoding="utf-8", newline="\n")
    (HERE / "ported_r4.txt").write_text("\n".join(listing) + "\n", encoding="utf-8", newline="\n")
    write_cb([l.split() for l in listing[1:]], set(r3.CALLBACKS))
    return linemap, dropped, len(listing) - 1


def cargo_errors(linemap):
    r = subprocess.run(["cargo", "build", "-p", "melee_chara", "--message-format=short"],
                       cwd=ROOT / "rust", capture_output=True, text=True)
    bad = set()
    for m in re.finditer(r"src[\\/]chars[\\/]r4[\\/](\w+)\.rs:(\d+):\d+: (?:error|warning: unreachable)", r.stderr):
        ln = int(m.group(2))
        name = linemap.get((m.group(1), ln))
        if name is None:
            cands = [n for (c, l), n in linemap.items() if c == m.group(1) and l >= ln]
            name = cands[0] if cands else None
        if name:
            bad.add(name)
    if r.returncode and not bad:
        sys.stderr.write(r.stderr[-4000:])
        raise SystemExit("cargo build failed outside round-4 functions")
    return bad


def main():
    drop, extra = set(), set()
    while True:
        tus = scan(drop, extra)
        names = {(rel, fn.name) for rel, (tu, fns) in tus.items() for fn in fns}
        if names - extra:
            extra |= names
            continue
        r2.BUILD = BUILD
        r3.BUILD = BUILD
        r2.eval_consts(tus)
        probe_assigns(tus)
        probe_cmps(tus)
        linemap, dropped, n = emit(tus)
        bad = dropped | cargo_errors(linemap)
        if not bad:
            break
        print(f"port_r4: dropping {len(bad)}: {sorted(bad)[:8]}...", flush=True)
        drop |= bad
        extra = {k for k in extra if k[1] not in drop}
    print(f"port_r4: {n} functions in {len(tus)} TUs")


if __name__ == "__main__":
    main()
