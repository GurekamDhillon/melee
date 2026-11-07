#!/usr/bin/env python3
"""Integer-semantics audit: where can the Rust port's integer behaviour differ from Gekko's?

Three scanners and a join (docs/rust-port/notes/int-audit.md has the method and the site table):

  python tools/rust-port/int_audit.py dol  [--dol main.dol] -o dol.csv
      Disassembles every .text function of the retail DOL (symbols.txt + splits.txt, capstone) and lists each
      integer-semantics site: cmpw/cmplw feeding a relational branch (cmp-s / cmp-u), slw/srw/sraw (shift),
      divw/divwu (div), fctiwz/fctiw (fptoi), extsb/extsh/lha (sext8/sext16), lbz/lhz (zext8/zext16).
      Prints only addresses, function names and mnemonics (no disc bytes). The DOL path defaults to
      <workspace>/melee/orig/GALE01/sys/main.dol; it is never printed.
  python tools/rust-port/int_audit.py cir  -o cir.csv [-j N]
      Compiles every C TU of the shipping build to LLVM IR (the gwtool input: ppc32, -O2 -disable-llvm-passes,
      line tables) and lists the same classes with their C line: sdiv/udiv/srem/urem with a non-constant
      divisor, shl/lshr/ashr with a non-constant amount, fptosi/fptoui, relational icmp on i8/i16/i32,
      sext/zext from i8/i16, icmp against 1 (bool).
  python tools/rust-port/int_audit.py rmir MIR_DIR -o rmir.csv
      Reads the Rust crates' MIR (RUSTC_BOOTSTRAP=1 cargo rustc -p CRATE --lib -- -Zunpretty=mir -Zmir-include-spans=on
      > MIR_DIR/CRATE.mir; the MSVC target drops columns from debug info, MIR keeps exact spans) and lists every site
      that does not follow Gekko semantics unless it goes through melee_core::ppc: Div/Rem with a non-constant (or 0 / -1)
      divisor (Rust panics), Shl/Shr with a non-constant amount (Rust masks it), and FloatToInt casts other than to
      u32/u64/usize (Rust gives 0 for NaN, and saturates narrow targets instead of fctiwz + truncate), including
      `(f as u32) as u8`. scope=game for files that port game C (GAME below), host otherwise.
  python tools/rust-port/int_audit.py fix rmir.csv [--dry]
      Rewrites the game-scope sites in place to melee_core::ppc calls (`<i32 as Ppc>::pdiv(a, b)`, `fctiwz_f32(x)`);
      prints the ones that need a hand edit.
  python tools/rust-port/int_audit.py cmp cir.csv MIR_DIR -o cmp.csv
      Per function, flags where the relational-compare signedness (32/64-bit), sign/zero extension or `== 1` tests of
      the Rust MIR disagree with the C IR. Every flag is then checked by hand (notes/int-audit.md).
  python tools/rust-port/int_audit.py report dol.csv cir.csv rmir.csv > table.md
      Per function: DOL site counts per class, the C lines, the Rust locations still open.
"""
import argparse, collections, concurrent.futures as cf, csv, os, re, subprocess, sys, tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
WS = ROOT.parents[1]


# ----------------------------------------------------------------------------------------------- DOL
def load_dol(path):
    b = Path(path).read_bytes()
    be = lambda o: int.from_bytes(b[o:o + 4], "big")
    secs = []
    for i in range(18):
        off, addr, size = be(i * 4), be(0x48 + i * 4), be(0x90 + i * 4)
        if size:
            secs.append((addr, size, off, i < 7))
    def read(addr, n):
        for a, s, o, _ in secs:
            if a <= addr and addr + n <= a + s:
                return b[o + addr - a:o + addr - a + n]
        return None
    return read


def load_symbols():
    out = []
    for line in (ROOT / "config/GALE01/symbols.txt").read_text().splitlines():
        m = re.match(r"(\S+) = (\.\w+):0x([0-9A-Fa-f]+); // type:function size:0x([0-9A-Fa-f]+)", line)
        if m and m.group(2) in (".text", ".init"):
            out.append((int(m.group(3), 16), int(m.group(4), 16), m.group(1)))
    return sorted(out)


def load_splits():
    tus, cur = [], None
    for line in (ROOT / "config/GALE01/splits.txt").read_text().splitlines():
        if line and not line[0].isspace() and line.endswith(":"):
            cur = line[:-1]
        m = re.match(r"\s+\.text\s+start:0x([0-9A-F]+) end:0x([0-9A-F]+)", line)
        if m and cur:
            tus.append((int(m.group(1), 16), int(m.group(2), 16), cur))
    return tus


def tu_of(tus, a):
    for s, e, n in tus:
        if s <= a < e:
            return n
    return ""


REL = {"blt", "bgt", "ble", "bge", "bnl", "bng", "bltlr", "bgtlr", "blelr", "bgelr", "bnllr", "bnglr"}


def crfield(op):
    m = re.match(r"cr(\d)", op.strip())
    return int(m.group(1)) if m else 0


def scan_dol(args):
    import capstone
    md = capstone.Cs(capstone.CS_ARCH_PPC, capstone.CS_MODE_32 | capstone.CS_MODE_BIG_ENDIAN)
    md.skipdata = True  # jump tables inside .text must not stop the walk (evalorder_audit found it)
    dolp = args.dol or os.environ.get("INT_AUDIT_DOL") or str(WS / "melee/orig/GALE01/sys/main.dol")
    if not Path(dolp).exists():
        sys.exit("int_audit: DOL not found (pass --dol or INT_AUDIT_DOL)")
    read = load_dol(dolp)
    tus = load_splits()
    rows = []
    for addr, size, name in load_symbols():
        code = read(addr, size)
        if code is None:
            continue
        ins = []
        for k in range(0, size, 4):
            w = code[k:k + 4]
            d = list(md.disasm_lite(w, addr + k))
            ins.append((addr + k, d[0][2], d[0][3]) if d else (addr + k, ".word", ""))
        tu = tu_of(tus, addr)
        for j, (a, mn, op) in enumerate(ins):
            cls = None
            if mn in ("cmpw", "cmpwi", "cmplw", "cmplwi"):
                cr = crfield(op) if op.startswith("cr") else 0
                for a2, mn2, op2 in ins[j + 1:j + 16]:
                    base = mn2.rstrip("+-")
                    if base.startswith("b") and base not in ("b", "bl", "blr", "bctr", "bctrl"):
                        if (crfield(op2) if op2.startswith("cr") else 0) == cr:
                            if base in REL:
                                cls = "cmp-u" if mn.startswith("cmpl") else "cmp-s"
                            break
                    if mn2.startswith(("cmp", "fcmp", "mtcr", "mcrf")) and (crfield(op2) if op2.startswith("cr") else 0) == cr:
                        break
            elif mn in ("slw", "slw.", "srw", "srw.", "sraw", "sraw."):
                cls = "shift"
            elif mn in ("divw", "divw.", "divwu", "divwu.", "divwo", "divwuo"):
                cls = "div"
            elif mn in ("fctiwz", "fctiw", "fctiwz.", "fctiw."):
                cls = "fptoi"
            elif mn in ("extsb", "extsb."):
                cls = "sext8"
            elif mn in ("extsh", "extsh.", "lha", "lhax", "lhau", "lhaux", "psq_l"):
                cls = "sext16" if mn != "psq_l" else None
            elif mn in ("lbz", "lbzx", "lbzu", "lbzux"):
                cls = "zext8"
            elif mn in ("lhz", "lhzx", "lhzu", "lhzux", "lhbrx"):
                cls = "zext16"
            if cls:
                rows.append((f"{a:08X}", name, tu, cls, mn))
    with open(args.o, "w", newline="") as f:
        w = csv.writer(f)
        w.writerow(["addr", "func", "tu", "class", "mnemonic"])
        w.writerows(rows)
    c = collections.Counter(r[3] for r in rows)
    print("dol sites:", dict(sorted(c.items())))


# ----------------------------------------------------------------------------------------------- C IR
def c_cmd(src, out):
    clang = os.environ.get("GW_CLANG") or str(WS / "_toolchains/llvm/bin/clang.exe")
    gale = os.environ.get("GALE_INCLUDE") or str((WS / "melee/build/GALE01/include").resolve())
    return [clang, "--target=ppc32-none-eabi", "-std=c99", "-nostdinc", "-fno-builtin", "-DLINT", "-DTARGET_PC",
            "-fno-short-enums", "-fsigned-char", "-mlong-double-64", "-fno-strict-aliasing", "-fwrapv", "-fcommon",
            "-fgnu89-inline", "-ftrivial-auto-var-init=zero", "-O2", "-Xclang", "-disable-llvm-passes",
            "-gline-tables-only", "-emit-llvm", "-S", "-w", "-Isrc", "-Ipc", "-Ipc/gameworld", "-isystem", "src/MSL",
            "-isystem", "libs/dolphin/include", "-isystem", "libs/dolphin/src", "-isystem", gale,
            "-include", "src/MSL/math_ppc.h", src, "-o", out]


NUM = r"(?:-?\d+|true|false)"


def ir_sites(text, rust):
    """(func, line, file, class, detail) for every site in one .ll text."""
    dbgfile, dbgline, scopes = {}, {}, {}
    for m in re.finditer(r"^!(\d+) = !DIFile\(filename: \"([^\"]*)\"(?:, directory: \"([^\"]*)\")?", text, re.M):
        dbgfile[m.group(1)] = m.group(2).replace("\\\\", "/")
    for m in re.finditer(r"^!(\d+) = (?:distinct )?!DI(?:Subprogram|LexicalBlock(?:File)?)\((.*)\)$", text, re.M):
        fm = re.search(r"file: !(\d+)", m.group(2))
        scopes[m.group(1)] = fm.group(1) if fm else None
    for m in re.finditer(r"^!(\d+) = !DILocation\(line: (\d+)(?:, column: (\d+))?, scope: !(\d+)", text, re.M):
        dbgline[m.group(1)] = (int(m.group(2)), int(m.group(3) or 0), dbgfile.get(scopes.get(m.group(4)), "?"))
    out, fn = [], None
    for line in text.splitlines():
        if line.startswith("define "):
            m = re.search(r'@("?)([^"(]+)\1\(', line)
            fn = m.group(2) if m else "?"
            narrow = set()
            continue
        if line.startswith("}"):
            fn = None
            continue
        if not fn or not line.startswith("  "):
            continue
        cls = det = None
        m = re.match(r"\s+%\S+ = (sdiv|udiv|srem|urem)(?: exact)? (i\d+) (\S+), (\S+?)(?:, !dbg|$)", line)
        if m:
            d = m.group(4)
            if not re.fullmatch(NUM, d) or d in ("0", "-1"):
                cls, det = "div", f"{m.group(1)}.{m.group(2)}"
        m = m or None
        if not cls:
            m = re.match(r"\s+%\S+ = (shl|lshr|ashr)(?: nuw| nsw| exact)* (i\d+) (\S+), (\S+?)(?:, !dbg|$)", line)
            if m and not re.fullmatch(NUM, m.group(4)):
                cls, det = "shift", f"{m.group(1)}.{m.group(2)}"
        if not cls:
            m = re.search(r"= (fptosi|fptoui) (float|double) \S+ to (i\d+)", line)
            if m:
                cls, det = "fptoi", f"{m.group(1)}.{m.group(2)}.{m.group(3)}"
            m = re.search(r"call (i\d+) @llvm\.(fptosi|fptoui)\.sat\.i\d+\.(f32|f64)", line)
            if m:
                cls, det = "fptoi", f"{m.group(2)}.sat.{m.group(3)}.{m.group(1)}"
        if not cls and not rust:
            m = re.search(r"= icmp (slt|sgt|sle|sge|ult|ugt|ule|uge) (i8|i16|i32) (\S+), (\S+?)(?:,|$)", line)
            if m:
                ops = [o for o in (m.group(3), m.group(4)) if not re.fullmatch(NUM, o)]
                if m.group(2) != "i32" or (ops and all(o in narrow for o in ops)):
                    cls, det = "cmp-n", m.group(1) + "." + m.group(2)  # promoted u8/s8/u16/s16: either signedness
                else:
                    cls, det = ("cmp-s" if m.group(1)[0] == "s" else "cmp-u"), m.group(1) + "." + m.group(2)
            if re.search(r"= icmp (eq|ne) i(8|16|32) %\S+, 1(?:,|$)", line):
                cls, det = "bool", "icmp-1"
            m2 = re.search(r"(%\S+) = (sext|zext) (i8|i16) ", line)
            if m2:
                narrow.add(m2.group(1))
            m2 = re.search(r"= (sext|zext) (i8|i16) ", line)
            if m2:
                cls, det = ("sext" if m2.group(1) == "sext" else "zext") + m2.group(2)[1:], m2.group(0)
        if not cls:
            continue
        dm = re.search(r"!dbg !(\d+)", line)
        ln, col, f = dbgline.get(dm.group(1), (0, 0, "?")) if dm else (0, 0, "?")
        out.append((fn, ln, col, f, cls, det))
    return out


def scan_cir(args):
    tus = sorted({n for _, _, n in load_splits()})
    srcs = [f"src/{n}" for n in tus if (ROOT / "src" / n).exists() and n.endswith(".c")]
    rows = []
    with tempfile.TemporaryDirectory(dir=args.tmp) as td:
        def one(s):
            o = Path(td) / (s.replace("/", "_") + ".ll")
            r = subprocess.run(c_cmd(s, str(o)), cwd=ROOT, capture_output=True)
            if r.returncode:
                return s, None
            t = o.read_text(errors="replace")
            o.unlink()
            return s, [(fn, ln, f, c, d) for fn, ln, col, f, c, d in ir_sites(t, False)]
        with cf.ThreadPoolExecutor(args.j) as ex:
            for s, res in ex.map(one, srcs):
                if res is None:
                    print("compile failed:", s, file=sys.stderr)
                    continue
                rows += [(s, *r) for r in res]
    with open(args.o, "w", newline="") as f:
        w = csv.writer(f)
        w.writerow(["tu", "func", "line", "file", "class", "detail"])
        w.writerows(rows)
    print("c sites:", dict(sorted(collections.Counter(r[4] for r in rows).items())))


# ----------------------------------------------------------------------------------------------- Rust MIR
INTS = {"i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128", "usize"}
SPAN = re.compile(r"// scope \d+ at (\S+?):(\d+):(\d+): (\d+):(\d+)")


def operand_type(op, locals_):
    op = op.strip()
    m = re.match(r"(?:copy|move) \(.*: ([\w:<>\[\]; &*]+)\)$", op)
    if m:
        return m.group(1).strip(), False
    m = re.match(r"(?:copy|move) (_\d+)$", op)
    if m:
        return locals_.get(m.group(1), "?"), False
    m = re.match(r"const (-?\d+)_(\w+)$", op)
    if m:
        return m.group(2), m.group(1)
    if op.startswith("const "):
        return "?", "named"
    return "?", False


def split_args(s):
    d, out, cur = 0, [], ""
    for ch in s:
        if ch in "([{<":
            d += 1
        elif ch in ")]}>":
            d -= 1
        if ch == "," and d == 0:
            out.append(cur); cur = ""
        else:
            cur += ch
    out.append(cur)
    return out


def mir_sites(text):
    """(fn, file, l1, c1, l2, c2, class, detail) for each Rust site that needs a melee_core::ppc helper."""
    out, fn, locals_, u32loc = [], None, {}, {}
    for line in text.splitlines():
        if line.startswith("fn "):
            fn = line[3:].split("(")[0]
            locals_, u32loc = {}, {}
            if "(" in line:
                for m in re.finditer(r"(_\d+): ([^,)]+)", line[line.index("("):]):
                    locals_[m.group(1)] = m.group(2).strip()
            continue
        m = re.match(r"\s+let (?:mut )?(_\d+): ([^;]+);", line)
        if m:
            locals_[m.group(1)] = m.group(2).strip()
            continue
        sm = SPAN.search(line)
        if not sm or fn is None:
            continue
        f = sm.group(1).replace("\\", "/")
        l1, c1, l2, c2 = map(int, sm.groups()[1:])
        stmt = line.split("//")[0]
        m = re.match(r"\s+(_\d+|\(.*\)) = (Div|Rem|Shl|Shr)\((.*)\);\s*$", stmt)
        if m:
            a, b = split_args(m.group(3))
            ta, _ = operand_type(a, locals_)
            tb, cb = operand_type(b, locals_)
            if ta not in INTS:
                continue
            if m.group(2) in ("Div", "Rem"):
                if cb == "named" or (cb is not False and cb not in ("0", "-1")):
                    continue
            elif cb is not False:
                continue
            out.append((fn, f, l1, c1, l2, c2, m.group(2).lower(), f"{ta}/{tb}"))
            continue
        m = re.match(r"\s+(_\d+|\(.*\)) = (.*) as (\w+) \(FloatToInt\);\s*$", stmt)
        if m:
            ts, _ = operand_type(m.group(2), locals_)
            if m.group(3) in ("u32", "u64", "usize"):
                # (u32) f is __cvt_fp2unsigned: saturating, NaN -> 0, same as Rust `as`. But `(f as u32) as u8`
                # (a port's spelling of C `(u8) f`) is not fctiwz + truncate: remember it.
                u32loc[m.group(1)] = (ts, m.group(3))
                continue
            out.append((fn, f, l1, c1, l2, c2, "fptoi", f"{ts}->{m.group(3)}"))
            continue
        m = re.match(r"\s+(_\d+|\(.*\)) = (?:move|copy) (_\d+) as (u8|u16|i8|i16) \(IntToInt\);\s*$", stmt)
        if m and m.group(2) in u32loc:
            ts, mid = u32loc[m.group(2)]
            out.append((fn, f, l1, c1, l2, c2, "fptoi", f"{ts}->{mid}->{m.group(3)}"))
    return out


# Rust files that port game C (compiled through gwtool in the C build, so PPC semantics apply). Everything else is
# native host code (renderer, netplay, scripting, launcher...) whose C counterpart, if any, is a native shim.
GAME = re.compile(r"(rust/)?(melee\w*/src/(.*/)?tu/|melee_runtime/src/(aobj|cabi|dat|fobj|fog|gekko|lobj|mobj|particle|"
                  r"random|tobj)\.rs|melee_fighter/src/|melee_chara/src/|melee_items/src/|melee_world/src/gr/|"
                  r"melee_audio/src/(synth|sound|lbvol|posobj|lbaudio)\.rs|melee_modes/src/tu_util\.rs)")


def scan_rmir(args):
    rows = []
    for d in args.dirs:
        for p in sorted(Path(d).glob("*.mir")):
            for r in mir_sites(p.read_text(errors="replace")):
                f = r[1]
                if "/raw/" in f or not re.match(r"(rust/)?melee\w*/src/", f) or "melee_core/src/ppc.rs" in f:
                    continue
                rows.append((p.stem, *r, "game" if GAME.match(f) else "host"))
    rows = sorted(set(rows), key=lambda r: (r[2], r[3], r[4]))
    with open(args.o, "w", newline="") as fo:
        w = csv.writer(fo)
        w.writerow(["crate", "func", "file", "l1", "c1", "l2", "c2", "class", "detail", "scope"])
        w.writerows(rows)
    print("rust sites:", dict(sorted(collections.Counter((r[7], r[9]) for r in rows).items())))


def top_level(s, pats):
    """(offset, token) of each top-level occurrence of any of pats in expression text s."""
    d, i, hits = 0, 0, []
    while i < len(s):
        ch = s[i]
        if ch == '"':
            i += 1
            while i < len(s) and s[i] != '"':
                i += 2 if s[i] == "\\" else 1
        elif ch in "([{":
            d += 1
        elif ch in ")]}":
            d -= 1
        elif d == 0:
            for p in pats:
                if s.startswith(p, i):
                    hits.append((i, p))
                    i += len(p) - 1
                    break
        i += 1
    return hits


def wrapped(s):
    """True when s is one parenthesised group: its first '(' closes at the last character."""
    d = 0
    for i, ch in enumerate(s):
        d += ch in "([{"
        d -= ch in ")]}"
        if d == 0:
            return i == len(s) - 1
    return False


ALIAS = {"s32": "i32", "s16": "i16", "s8": "i8", "c_int": "i32", "::core::ffi::c_int": "i32", "core::ffi::c_int": "i32", "libc::c_int": "i32", "::core::ffi::c_uint": "u32", "core::ffi::c_uint": "u32", "c_uint": "u32",
         "::core::ffi::c_float": "f32"}


def rewrite(span, cls, detail):
    """New text for one site's span, or None when it needs a hand edit. MIR gives the statement's span when the
    value is stored straight into a place (`p = x as i32`) and keeps the source's parentheses."""
    eq = [i for i, _ in top_level(span, ["="]) if (i == 0 or span[i - 1] not in "=<>!+-*/%&|^") and span[i + 1:i + 2] not in ("=", ">")]
    if eq and not span.lstrip().startswith("let "):
        i = eq[0] + 1
        r = rewrite_expr(span[i:], cls, detail)
        return None if r is None else span[:i] + (" " if span[i:i + 1] == " " else "") + r.lstrip()
    return rewrite_expr(span, cls, detail)


def rewrite_expr(span, cls, detail):
    s = span.strip()
    if s.startswith("(") and wrapped(s):
        r = rewrite_expr(s[1:-1], cls, detail)
        return None if r is None else "(" + r + ")"
    return rewrite_core(s, cls, detail)


def rewrite_core(span, cls, detail):
    if cls == "fptoi" and detail.count("->") == 2:
        src, mid, dst = detail.split("->")
        hits = top_level(span, [" as "])
        if not hits or src not in ("f32", "f64") or ALIAS.get(span[hits[-1][0] + 4:].strip(), span[hits[-1][0] + 4:].strip()) != dst:
            return None
        inner = span[:hits[-1][0]].strip()
        if inner.startswith("(") and wrapped(inner):
            inner = inner[1:-1].strip()
        h2 = top_level(inner, [" as "])
        if not h2 or ALIAS.get(inner[h2[-1][0] + 4:].strip(), inner[h2[-1][0] + 4:].strip()) != mid:
            return None
        expr = inner[:h2[-1][0]].strip()
        return f"melee_core::ppc::fctiwz{'_f32' if src == 'f32' else ''}({expr}) as {dst}"
    if cls == "fptoi":
        src, dst = detail.split("->")
        hits = top_level(span, [" as "])
        if not hits or src not in ("f32", "f64"):
            return None
        i = hits[-1][0]
        expr, ty = span[:i].strip(), span[i + 4:].strip()
        ty = ALIAS.get(ty, ty)
        if ty == dst and dst in ("u32", "u64", "i64", "usize"):  # semantics_lint I3: explicit helpers, same values
            return f"melee_core::ppc::fcvt_{dst}({expr})"
        if ty != dst or dst not in ("i32", "i16", "i8", "u16", "u8"):
            return None
        call = f"melee_core::ppc::fctiwz{'_f32' if src == 'f32' else ''}({expr})"
        return call if dst == "i32" else f"{call} as {dst}"
    op = {"div": "/", "rem": "%", "shl": "<<", "shr": ">>"}[cls]
    name = {"div": "pdiv", "rem": "prem", "shl": "pshl", "shr": "pshr"}[cls]
    ta = detail.split("/")[0]
    hits = [h for h in top_level(span, ["<<=", ">>=", "<<", ">>", "/=", "%=", "//", "/", "%"]) if h[1].rstrip("=") == op]
    if not hits:
        return None
    i, p = hits[-1]
    lhs, rhs = span[:i].strip(), span[i + len(p):].strip()
    if not lhs or not rhs or ta not in INTS:
        return None
    if ta in ("u128", "i128"):
        return span  # host-side packed-bitfield helpers (tu/support.rs): no C operator behind them
    if re.fullmatch(r"\(?\s*(0x[0-9A-Fa-f_]+|\d[\d_]*)(_?[iu]\d+)?(\s+as\s+[\w:]+)*\s*\)?", rhs):
        lit = int(re.match(r"\(?\s*(0x[0-9A-Fa-f_]+|\d[\d_]*)", rhs).group(1).replace("_", ""), 0)
        if (cls in ("div", "rem") and lit != 0) or (cls in ("shl", "shr") and lit < int(re.sub(r"\D", "", ta) or 64)):
            return span  # a literal operand MIR did not fold (`4 as i32`): the plain operator is already exact
    r = f"({rhs}) as i64" if cls in ("shl", "shr") else rhs
    call = f"<{ta} as melee_core::ppc::Ppc>::{name}({lhs}, {r})"
    return f"{lhs} = {call}" if p.endswith("=") else call


def fix(args):
    rows = [r for r in csv.DictReader(open(args.csv)) if r["scope"] == "game"]
    byfile = collections.defaultdict(list)
    for r in rows:
        byfile[r["file"]].append(r)
    manual, total = [], 0
    for f, rs in sorted(byfile.items()):
        path = ROOT / f if f.startswith("rust/") else ROOT / "rust" / f
        text = path.read_text(encoding="utf-8")
        lines = text.split("\n")
        starts = [0]
        for L in lines:
            starts.append(starts[-1] + len(L) + 1)
        uniq = {}
        for r in rs:
            a = starts[int(r["l1"]) - 1] + int(r["c1"]) - 1
            b = starts[int(r["l2"]) - 1] + int(r["c2"]) - 1
            uniq.setdefault((a, b), (r["class"], r["detail"]))
        # innermost first; each edit shifts the spans after it and grows the spans around it
        todo = sorted(([a, b, *uniq[(a, b)]] for a, b in uniq), key=lambda k: k[1] - k[0])
        edits = 0
        for n, (a, b, c, d) in enumerate(todo):
            new = rewrite(text[a:b], c, d)
            if new is None:
                manual.append(f"{f}:{text[:a].count(chr(10)) + 1} {c} {d} | {text[a:b][:120]!r}")
                continue
            delta = len(new) - (b - a)
            text = text[:a] + new + text[b:]
            edits += 1
            for k in todo[n + 1:]:
                if k[0] <= a and b <= k[1]:
                    k[1] += delta
                elif k[0] >= b:
                    k[0] += delta; k[1] += delta
        if edits and not args.dry:
            path.write_text(text, encoding="utf-8", newline="")
        total += edits
    print(f"rewritten {total}; manual {len(manual)}")
    for m in manual:
        print("  " + m)


# ----------------------------------------------------------------------------------------------- compare / extension cross-check
SIGNED = {"i32", "i64", "s32"}       # 8/16-bit compares are promoted in C: either signedness gives the same answer
UNSIGNED = {"u32", "u64"}


def mir_cmp(text):
    """fn -> Counter of cmp-s / cmp-u (relational compares on fixed-width ints) and sext / zext (i8/i16/u8/u16 widened),
    with the source file of the function's first span."""
    out, fn, locals_ = {}, None, {}
    for line in text.splitlines():
        if line.startswith("fn "):
            fn = line[3:].split("(")[0]
            locals_ = {}
            if "(" in line:
                for m in re.finditer(r"(_\d+): ([^,)]+)", line[line.index("("):]):
                    locals_[m.group(1)] = m.group(2).strip()
            out[fn] = [collections.Counter(), None]
            continue
        m = re.match(r"\s+let (?:mut )?(_\d+): ([^;]+);", line)
        if m:
            locals_[m.group(1)] = m.group(2).strip()
            continue
        if fn is None:
            continue
        sm = SPAN.search(line)
        if sm and out[fn][1] is None:
            out[fn][1] = sm.group(1).replace("\\", "/")
        stmt = line.split("//")[0]
        m = re.match(r"\s+\S+ = (Lt|Le|Gt|Ge)\((.*)\);\s*$", stmt)
        if m:
            a, b = split_args(m.group(2))
            t = operand_type(a, locals_)[0]
            if t == "?":
                t = operand_type(b, locals_)[0]
            if t in SIGNED:
                out[fn][0]["cmp-s"] += 1
            elif t in UNSIGNED:
                out[fn][0]["cmp-u"] += 1
            continue
        if re.match(r"\s+\S+ = (Eq|Ne)\((?:copy|move) .*, const 1_[iu](32|8|16)\);\s*$", stmt):
            out[fn][0]["eq1"] += 1
            continue
        m = re.match(r"\s+\S+ = (.*) as (\w+) \(IntToInt\);\s*$", stmt)
        if m:
            src = operand_type(m.group(1), locals_)[0]
            if src in ("i8", "i16") and m.group(2) not in ("i8", "u8", "i16", "u16"):
                out[fn][0]["sext"] += 1
            elif src in ("u8", "u16") and m.group(2) not in ("i8", "u8", "i16", "u16"):
                out[fn][0]["zext"] += 1
    return out


def scan_cmp(args):
    """Functions whose relational-compare signedness disagrees between the C (shipping IR) and the Rust (MIR): the
    Rust has unsigned compares where the C has none, or signed where the C has none. Each is checked by hand."""
    c = collections.defaultdict(collections.Counter)
    for r in csv.DictReader(open(args.cir)):
        c[r["func"]][r["class"]] += 1
    rows = []
    for p in sorted(Path(args.mir).glob("*.mir")):
        for fn, (cnt, f) in mir_cmp(p.read_text(errors="replace")).items():
            if not f or "/raw/" in f or not re.match(r"(rust/)?melee\w*/src/", f):
                continue
            name = fn.split("::")[-1].split("<")[0].strip()
            if name not in c:
                continue
            cc = c[name]
            flag = []
            if cnt["cmp-u"] and not cc["cmp-u"]:
                flag.append("rust-unsigned-only")
            if cnt["cmp-s"] and not cc["cmp-s"]:
                flag.append("rust-signed-only")
            if cc["cmp-u"] and not cnt["cmp-u"] and cnt["cmp-s"]:
                flag.append("c-unsigned-missing")
            if cc["sext8"] + cc["sext16"] and cnt["zext"] and not cnt["sext"] and not (cc["zext8"] + cc["zext16"]):
                flag.append("ext-swapped")
            if cnt["eq1"] > cc["bool"]:
                flag.append("rust-eq1-extra")
            if flag:
                rows.append((p.stem, name, f, cnt["cmp-s"], cnt["cmp-u"], cc["cmp-s"], cc["cmp-u"], " ".join(flag)))
    with open(args.o, "w", newline="") as fo:
        w = csv.writer(fo)
        w.writerow(["crate", "func", "file", "rust_cmp_s", "rust_cmp_u", "c_cmp_s", "c_cmp_u", "flag"])
        w.writerows(sorted(set(rows)))
    print("flagged functions:", len(set(rows)), dict(collections.Counter(x for r in rows for x in r[7].split())))


def main():
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="cmd", required=True)
    a = sub.add_parser("dol"); a.add_argument("--dol"); a.add_argument("-o", required=True)
    a = sub.add_parser("cir"); a.add_argument("-o", required=True); a.add_argument("-j", type=int, default=8)
    a.add_argument("--tmp", default=None)
    a = sub.add_parser("rmir"); a.add_argument("dirs", nargs="+"); a.add_argument("-o", required=True)
    a = sub.add_parser("fix"); a.add_argument("csv"); a.add_argument("--dry", action="store_true")
    a = sub.add_parser("cmp"); a.add_argument("cir"); a.add_argument("mir"); a.add_argument("-o", required=True)
    a = sub.add_parser("report"); a.add_argument("dol"); a.add_argument("cir"); a.add_argument("rmir")
    args = ap.parse_args()
    {"dol": scan_dol, "cir": scan_cir, "rmir": scan_rmir, "fix": fix, "cmp": scan_cmp, "report": report}[args.cmd](args)


def report(args):
    """Markdown site table. Every DOL div/shift/fptoi site gets a row; compares and extensions are summarised per
    class, with the functions the cmp cross-check flagged listed with their verdicts (--verdicts CSV: func,verdict)."""
    fnidx = {}
    for p in sorted((ROOT / "rust").glob("melee*/src/**/*.rs")):
        rel = p.relative_to(ROOT / "rust").as_posix()
        if "/raw/" in rel:
            continue
        for i, line in enumerate(p.read_text(encoding="utf-8", errors="replace").splitlines(), 1):
            m = re.search(r"\bfn ([A-Za-z_]\w*)\s*[(<]", line)
            if m:
                fnidx.setdefault(m.group(1), f"{rel}:{i}")
    csites = collections.defaultdict(list)
    for r in csv.DictReader(open(args.cir)):
        csites[(r["func"], r["class"])].append(int(r["line"]))
    rsites = collections.defaultdict(list)
    cmap = {"div": "div", "rem": "div", "shl": "shift", "shr": "shift", "fptoi": "fptoi"}
    for r in csv.DictReader(open(args.rmir)):
        if r.get("scope", "game") != "game":
            continue
        name = r["func"].split("::")[-1].split("<")[0].split("{")[0].strip()
        rsites[(name, cmap[r["class"]])].append(f"{r['file']}:{r['l1']}")
    rows, counts = [], collections.Counter()
    for r in csv.DictReader(open(args.dol)):
        cls = r["class"]
        if cls not in ("div", "shift", "fptoi"):
            counts[(cls, "-")] += 1
            continue
        fn = r["func"]
        k = sum(1 for x in rows if x[1] == fn and x[3] == cls)
        cl = sorted(csites.get((fn, cls), []))
        cline = f"{r['tu']}:{cl[k]}" if k < len(cl) else (f"{r['tu']}" if r["tu"] else "")
        rs = sorted(set(rsites.get((fn, cls), [])))
        if rs:
            loc, st = rs[min(k, len(rs) - 1)], "fixed"
        elif fn in fnidx:
            loc = fnidx[fn]
            st = "ok" if cl or cls == "fptoi" else "ok-const"
        else:
            loc, st = "", "no-port"
        rows.append((r["addr"], fn, cline, cls, r["mnemonic"], loc, st))
        counts[(cls, st)] += 1
    print("| class | status | sites |\n|---|---|---|")
    for (c, s), n in sorted(counts.items()):
        print(f"| {c} | {s} | {n} |")
    print()
    print("| address | function | C line | class | insn | Rust location | status |\n|---|---|---|---|---|---|---|")
    for x in rows:
        print("| " + " | ".join(x) + " |")


if __name__ == "__main__":
    main()
