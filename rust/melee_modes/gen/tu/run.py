#!/usr/bin/env python3
"""Build the C side of a TU state oracle and run it on the Rust-exported cases.

  ORACLE_EXPORT=rust/target/cref cargo test -p melee_modes --test tu_<stem>     (writes tu_<stem>.cases)
  python rust/melee_modes/gen/tu/run.py <stem> [src/melee/gm/<stem>.c] [-- func:idx]   (worktree root)

The real game TU is `#include`d into a generated driver (so its statics are reachable) and compiled
exactly like group 3's cref (rust/melee_fighter/gen/cref/run.py): x86_64 msvc target, the same
masks, the shipping float contraction (-ffp-contract=on, -O2 -disable-llvm-passes) and gwtool's
float/shift lowering (rust/melee_runtime/oracle/lower.py). Every function the TU calls but does not
define is replaced by a recording stub generated from the IR `declare`s (gen/tu/rt.c has the
recorder). Output: rust/melee_modes/tests/data/tu_<stem>.txt (one `R` row per case). Data comes
from C source and the Rust-built synthetic state only: no disc data.
"""
import os, re, subprocess, sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(ROOT / "rust/melee_fighter/gen/cref"))
import run as cref  # noqa: E402  (FLAGS, CLANG, LOWER)

BUILD = ROOT / "rust/target/cref"
DATA = ROOT / "rust/melee_modes/tests/data"
CLANG = cref.CLANG

# libc / compiler-support callees that keep their real implementation
REAL = set("""sprintf snprintf vsprintf vsnprintf memcpy memset memmove memcmp strlen strcpy strncpy strcmp strncmp
strcat strncat strchr strrchr strstr atoi sscanf toupper tolower isdigit isalpha isspace isupper islower
strtol strtoul qsort bsearch abs labs""".split())

# The shipping build is clang for powerpc (Itanium-style ABI): call arguments are evaluated left to
# right. clang's msvc triple evaluates them right to left, so the front end runs with the gnu triple
# (same LLP64 data model); the IR is retargeted to msvc for linking.
GNU_FLAGS = [("--target=x86_64-w64-windows-gnu" if f.startswith("--target=") else f) for f in cref.FLAGS]

# printf-style variadic callees: the fixed-argument index of the format string. Their varargs are
# logged by rt_va (count and kinds from the format); other variadics log the fixed arguments only.
VA_FMT = {"OSReport": 0, "HSD_SisLib_803A6B98": 3, "HSD_SisLib_803A70A0": 2, "OSPanic": 2}
# variadic callees whose varargs have fixed kinds (see rt_va_kinds)
VA_KINDS = {"HSD_ForeachAnim": "d", "lb_80011E24": "ii"}

SIZES = {"i1": 1, "i8": 1, "i16": 2, "i32": 4, "i64": 8, "ptr": 8, "float": 4, "double": 8}


def sh(cmd, what):
    r = subprocess.run(cmd, cwd=ROOT, capture_output=True, text=True)
    if r.returncode:
        sys.stderr.write(r.stdout + r.stderr)
        raise SystemExit(f"{what} failed")
    return r


def parse_cases_header(path):
    globals_, fntab, funcs = [], [], []
    with open(path, encoding="utf-8") as f:
        for line in f:
            t = line.split()
            if not t:
                continue
            if t[0] == "GLOBAL":
                globals_.append((t[1], int(t[3], 16), t[4]))
            elif t[0] == "FNTAB":
                fntab.append(t[1])
            elif t[0] == "FUNC":
                funcs.append((int(t[1]), t[2], "" if t[3] == "-" else t[3], t[4]))
            elif t[0] == "CASE":
                break
    return globals_, fntab, funcs


def split_params(s):
    out, depth, cur = [], 0, ""
    for ch in s:
        if ch in "({[<":
            depth += 1
        elif ch in ")}]>":
            depth -= 1
        if ch == "," and depth == 0:
            out.append(cur.strip())
            cur = ""
        else:
            cur += ch
    if cur.strip():
        out.append(cur.strip())
    return out


DECL = re.compile(r"^declare\s+(?P<pre>[^@]*?)@(?P<name>[\w.$]+)\((?P<params>.*)\)(?P<post>[^)]*)$")


def make_stubs(ll_text, defined, extra_skip, ext_globals):
    """IR for the recording stubs + definitions of the external data."""
    out = []
    strs = []
    manual = []
    kc = []
    for line in ll_text.splitlines():
        if not line.startswith("declare "):
            continue
        m = DECL.match(line)
        if not m:
            manual.append(("unparsed", line))
            continue
        name = m["name"]
        if name.startswith("llvm.") or name in REAL or name in defined or name in extra_skip or name.startswith("rt_"):
            continue
        pre = m["pre"].split()
        ret = [w for w in pre if w in ("void", "ptr", "float", "double") or re.fullmatch(r"i\d+", w) or w.startswith(("{", "<", "%"))]
        if len(ret) != 1:
            manual.append((name, line))
            continue
        ret = ret[0]
        if ret.startswith(("{", "<", "%")):
            manual.append((name, line))
            continue
        params = split_params(m["params"])
        ps = []
        body = []
        ok = True
        n = 0
        va = False
        for p in params:
            if p == "...":
                va = True
                continue
            ty = p.split()[0]
            if "sret" in p or "byval" in p or ty not in SIZES:
                ok = False
                break
            ps.append(f"{ty} %a{n}")
            if ty == "ptr":
                body.append(f"  call void @rt_arg_p(ptr %a{n})")
            elif ty == "float":
                body.append(f"  call void @rt_arg_f(float %a{n})")
            elif ty == "double":
                body.append(f"  call void @rt_arg_d(double %a{n})")
            else:
                if ty == "i64":
                    body.append(f"  call void @rt_arg_i(i64 %a{n}, i32 64)")
                else:
                    body.append(f"  %z{n} = zext {ty} %a{n} to i64\n  call void @rt_arg_i(i64 %z{n}, i32 {SIZES[ty] * 8 if ty != 'i1' else 1})")
            n += 1
        if not ok:
            manual.append((name, line))
            continue
        if va:
            ps.append("...")
            if name in VA_FMT:
                body += ["  %ap = alloca ptr, align 8", "  call void @llvm.va_start.p0(ptr %ap)",
                         f"  call void @rt_va(ptr %ap, i32 {VA_FMT[name]})", "  call void @llvm.va_end.p0(ptr %ap)"]
            elif name in VA_KINDS:
                kb = VA_KINDS[name].encode() + b"\0"
                kc.append(f'@.vk.{len(kc)} = private unnamed_addr constant [{len(kb)} x i8] c"{VA_KINDS[name]}\\00"')
                body += ["  %ap = alloca ptr, align 8", "  call void @llvm.va_start.p0(ptr %ap)",
                         f"  call void @rt_va_kinds(ptr %ap, ptr @.vk.{len(kc) - 1})", "  call void @llvm.va_end.p0(ptr %ap)"]
        idx = len(strs)
        strs.append(name)
        head = [f"define dso_local {ret} @{name}({', '.join(ps)}) {{",
                f"  call void @rt_begin(ptr @.stub.{idx})", *body, "  call void @rt_args_end()"]
        if ret == "void":
            head += ["  call void @rt_ret_v()", "  ret void"]
        elif ret == "ptr":
            head += ["  %r = call ptr @rt_ret_p()", "  ret ptr %r"]
        elif ret == "float":
            head += ["  %r = call float @rt_ret_f()", "  ret float %r"]
        elif ret == "double":
            head += ["  %r = call double @rt_ret_d()", "  ret double %r"]
        else:
            bits = int(ret[1:])
            head += [f"  %r = call i64 @rt_ret_i(i32 {bits})"]
            if bits == 64:
                head += ["  ret i64 %r"]
            else:
                head += [f"  %t = trunc i64 %r to {ret}", f"  ret {ret} %t"]
        head.append("}")
        out.append("\n".join(head))
    consts = []
    for i, n in enumerate(strs):
        b = n.encode() + b"\0"
        consts.append(f'@.stub.{i} = private unnamed_addr constant [{len(b)} x i8] c"{n}\\00"')
    consts += kc
    data = []
    for name, size in ext_globals:
        data.append(f"@{name} = dso_local global [{size} x i8] zeroinitializer, align 16")
    decls = """declare void @rt_begin(ptr)
declare void @rt_arg_i(i64, i32)
declare void @rt_arg_p(ptr)
declare void @rt_arg_f(float)
declare void @rt_arg_d(double)
declare void @rt_args_end()
declare i64 @rt_ret_i(i32)
declare ptr @rt_ret_p()
declare float @rt_ret_f()
declare double @rt_ret_d()
declare void @rt_ret_v()
declare void @rt_va(ptr, i32)
declare void @rt_va_kinds(ptr, ptr)
declare void @llvm.va_start.p0(ptr)
declare void @llvm.va_end.p0(ptr)
"""
    return decls + "\n".join(consts) + "\n" + "\n".join(data) + "\n" + "\n\n".join(out) + "\n", manual


def ir_global_size(line):
    m = re.search(r"global (\[(\d+) x (i\d+|ptr|float|double)\]|(i\d+|ptr|float|double))\b", line)
    if not m:
        return None
    if m.group(2):
        return int(m.group(2)) * SIZES[m.group(3)]
    return SIZES[m.group(4)]


def main():
    args = sys.argv[1:]
    trace = None
    if "--" in args:
        i = args.index("--")
        trace = args[i + 1]
        args = args[:i]
    stem = args[0]
    src = args[1] if len(args) > 1 else f"src/melee/gm/{stem}.c"
    BUILD.mkdir(parents=True, exist_ok=True)
    DATA.mkdir(parents=True, exist_ok=True)
    cases = BUILD / f"tu_{stem}.cases"
    globals_, fntab, funcs = parse_cases_header(cases)

    # ---- driver
    d = []
    # Some TUs rely on PPC layout (4-byte pointers) to alias two struct views of one object; native
    # 64-bit C would not alias them, so the oracle compiles a mechanically patched copy
    # (gen/tu/patches/<stem>.py: patch(text) -> text). The Rust port implements the PPC semantics.
    srcpath = ROOT / src
    pp = HERE / "patches" / f"{stem}.py"
    if pp.exists():
        import importlib.util
        spec = importlib.util.spec_from_file_location(f"patch_{stem}", pp)
        mod = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(mod)
        patched = BUILD / f"{stem}_patched.c"
        patched.write_text(mod.patch((ROOT / src).read_text(encoding="utf-8")), encoding="utf-8", newline="\n")
        srcpath = patched
    d.append(f'#include "{srcpath.as_posix()}"')
    d.append("typedef unsigned long long rt_u64;")
    d.append("typedef struct { const char* name; void* addr; rt_u64 size; } RtGlobal;")
    d.append("typedef struct { const char* name; void* addr; } RtFn;")
    d.append("typedef struct { const char* name; rt_u64 (*thunk)(const rt_u64*); } RtFunc;")
    d.append("extern int rt_main(int, char**, const RtGlobal*, int, const RtFn*, int, const RtFunc*, int);")
    d.append("extern float rt_f32(rt_u64); extern double rt_f64(rt_u64); extern rt_u64 rt_f32bits(float); extern rt_u64 rt_f64bits(double); extern rt_u64 rt_normptr(rt_u64);")
    for i, name, argk, retk in funcs:
        a = []
        pre = ""
        for k, c in enumerate(argk):
            if c == "o":  # pointer to a local void* (zeroed): a cursor out-parameter
                pre += f"void* o{k} = 0; "
            a.append({"i": f"a[{k}]", "p": f"(void*)(__UINTPTR_TYPE__)a[{k}]", "f": f"rt_f32(a[{k}])", "d": f"rt_f64(a[{k}])", "o": f"&o{k}"}[c])
        call = f"{name}({', '.join(a)})"
        body = {
            "v": f"{call}; return 0;",
            "b": f"return (rt_u64)!!({call});",
            "i8": f"return (rt_u64)({call}) & 0xFFULL;",
            "i16": f"return (rt_u64)({call}) & 0xFFFFULL;",
            "i32": f"return (rt_u64)({call}) & 0xFFFFFFFFULL;",
            "i64": f"return (rt_u64)({call});",
            "p": f"return rt_normptr((rt_u64)(__UINTPTR_TYPE__)({call}));",
            "f": f"return rt_f32bits({call});",
            "d": f"return rt_f64bits({call});",
            # a 4-byte struct returned by value: its raw bytes
            "g4": f"{{ __typeof__({call}) t_ = {call}; unsigned int v_; __builtin_memcpy(&v_, &t_, 4); return v_; }}",
        }[retk]
        d.append(f"static rt_u64 th_{i}(const rt_u64* a) {{ (void)a; {pre}{body} }}")
    d.append("int main(int argc, char** argv) {")
    d.append("  static const RtGlobal gs[] = {")
    for n, size, mode in globals_:
        d.append(f'    {{ "{n}", (void*)&{n}, {size} }},')
    d.append("    { 0, 0, 0 } };")
    d.append("  static const RtFn fs[] = {")
    for n in fntab:
        d.append(f'    {{ "{n}", (void*){n} }},')
    d.append("    { 0, 0 } };")
    d.append("  static const RtFunc fn[] = {")
    for i, name, argk, retk in funcs:
        d.append(f'    {{ "{name}", th_{i} }},')
    d.append("    { 0, 0 } };")
    d.append(f"  return rt_main(argc, argv, gs, {len(globals_)}, fs, {len(fntab)}, fn, {len(funcs)});")
    d.append("}")
    drv = BUILD / f"{stem}_driver.c"
    drv.write_text("\n".join(d) + "\n", encoding="utf-8")

    # ---- compile + lower the driver (TU included) and gekko_fp
    ll = BUILD / f"{stem}_driver.ll"
    lo = BUILD / f"{stem}_driver.lo.ll"
    sh([CLANG, *GNU_FLAGS, "-I", str(ROOT / Path(src).parent), str(drv), "-o", str(ll)], "driver compile")
    # linked as msvc (libc, VirtualAlloc); only the front end's evaluation order comes from the gnu triple
    t = ll.read_text(encoding="utf-8", errors="replace")
    t = re.sub(r'target triple = "[^"]*"', 'target triple = "x86_64-pc-windows-msvc"', t)
    ll.write_text(t, encoding="utf-8")
    sh([sys.executable, str(cref.LOWER), str(ll), str(lo)], "lower")
    gk = BUILD / "gekko_fp.ll"
    gklo = BUILD / "gekko_fp.lo.ll"
    if not gklo.exists():
        sh([CLANG, *cref.FLAGS, str(ROOT / "pc/gameworld/gekko_fp.c"), "-o", str(gk)], "gekko compile")
        sh([sys.executable, str(cref.LOWER), str(gk), str(gklo)], "gekko lower")

    text = lo.read_text(encoding="utf-8", errors="replace")
    defined = set(re.findall(r"^define [^@]*@([\w.$]+)\(", text, re.M))
    defined |= set(re.findall(r"^define [^@]*@([\w.$]+)\(", gklo.read_text(encoding="utf-8", errors="replace"), re.M))
    reg = {n: s for n, s, m in globals_}
    ext_globals = []
    for m in re.finditer(r"^@([\w.$]+) = external[^\n]*", text, re.M):
        name = m.group(1)
        sz = reg.get(name) or ir_global_size(m.group(0)) or 4096
        ext_globals.append((name, max(sz, 16)))
    stub_ir, manual = make_stubs(text, defined, set(), ext_globals)
    # the external declarations in the driver IR would clash with the definitions: drop them
    stub_path = BUILD / f"{stem}_stubs.ll"
    hdr = "\n".join(l for l in text.splitlines() if l.startswith(("target datalayout", "target triple")))
    stub_path.write_text(hdr + "\n" + stub_ir, encoding="utf-8")
    if manual:
        for n, l in manual:
            print(f"MANUAL STUB NEEDED: {n}: {l}", file=sys.stderr)
        raise SystemExit("callees with struct/sret/byval signatures need hand-written stubs")
    exe = BUILD / f"{stem}_oracle.exe"
    sh([CLANG, "--target=x86_64-pc-windows-msvc", "-O0", "-w", "-ffp-contract=off", str(lo), str(gklo), str(stub_path),
        str(HERE / "rt.c"), "-o", str(exe)], "link")
    cmd = [str(exe), str(cases)] + ([trace] if trace else [])
    r = subprocess.run(cmd, cwd=ROOT, capture_output=True, text=True)
    if r.returncode:
        sys.stderr.write(r.stdout[-400:] + r.stderr[-1500:])
        raise SystemExit(f"oracle exe exited {r.returncode}")
    if trace:
        sys.stdout.write(r.stdout.replace("\r", ""))
        return
    out = DATA / f"tu_{stem}.txt"
    out.write_text(r.stdout.replace("\r", ""), encoding="utf-8", newline="\n")
    print(f"{stem}: {len([l for l in r.stdout.splitlines() if l.startswith('R ')])} rows -> {out}")


if __name__ == "__main__":
    main()
