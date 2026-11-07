#!/usr/bin/env python3
"""Evaluation-order audit: C leaves operand and argument order unspecified; MWCC picked one, Rust always goes left to right.

  python tools/rust-port/evalorder_audit.py scan -o sites.csv [-j N]   C AST (clang -ast-dump=json) of every game TU
  python tools/rust-port/evalorder_audit.py dol sites.csv -o order.csv  the retail call order of each site's callees

scan: every full-expression in a game function (src/melee, src/sysdolphin; header inlines included when the TU uses them)
where two unsequenced subexpressions (arguments of one call; operands of a binary operator other than && || , ; the two
sides of an assignment) contain
  call-call   a call in each (RNG calls HSD_Rand* / fn names with Rand flagged `rng`),
  call-read   a call in one and a read of a global / through a pointer in the other (the call may change it).
Each row: tu, func, line, kind, rng, callees in source (left-to-right) order, text.
dol: per call-call row, the bl targets of the DOL function (symbols.txt) in address order; `order` = l2r when the
callees' first calls after the previous site's appear in source order, r2l when reversed, ? when a callee is inlined or
ambiguous. The C PC build (clang) and Rust both evaluate left to right; an r2l row is a site where the console differs.
No disc bytes are printed: only addresses, symbol names and mnemonics."""
import argparse, concurrent.futures as cf, csv, json, os, re, subprocess, sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
sys.path.insert(0, str(HERE))
import int_audit  # noqa: E402

UNSEQ_BIN = {"+", "-", "*", "/", "%", "<<", ">>", "<", ">", "<=", ">=", "==", "!=", "&", "|", "^", "=", "+=", "-=",
             "*=", "/=", "%=", "<<=", ">>=", "&=", "|=", "^="}
PURE = re.compile(r"^(__builtin_\w+|sqrtf?|fabsf?|sinf?|cosf?|tanf?|atan2?f?|asinf?|acosf?|powf?|expf?|logf?|floorf?|ceilf?|fmodf?|ABS|__fabs|__frsqrte|__fres|lbVector_\w+Len\w*|HSD_JObjGet\w+|"
                  r"GET_\w+|Player_Get\w+|ftLib_80086\w+|mpLib_GetLine\w*|PSVECMag|PSVECDotProduct|PSVECSquareMag)$")


def ast_cmd(src):
    c = int_audit.c_cmd(src, "x")
    i = c.index("-emit-llvm")
    c = c[:i] + ["-fsyntax-only", "-Xclang", "-ast-dump=json"] + c[i + 2:]
    c = [a for a in c if a not in ("-o", "x", "-gline-tables-only", "-S", "-DTARGET_PC")]
    return c[:1] + ["-UTARGET_PC"] + c[1:]  # the source as MWCC compiled it (no PC-only code, no GW_NOFUSE calls)


def walk_calls(n, out):
    """callee names of every CallExpr under n (unknown callee: '?')."""
    if not isinstance(n, dict):
        return
    if n.get("kind") == "CallExpr":
        out.append(callee(n))
    for ch in n.get("inner", []) or []:
        walk_calls(ch, out)


def callee(n):
    inner = n.get("inner") or []
    if not inner:
        return "?"
    f = inner[0]
    while isinstance(f, dict) and f.get("kind") in ("ImplicitCastExpr", "ParenExpr"):
        f = (f.get("inner") or [None])[0]
    if isinstance(f, dict) and f.get("kind") == "DeclRefExpr":
        return f.get("referencedDecl", {}).get("name", "?")
    return "<indirect>"


def reads_state(n):
    """a read of a global variable or through a pointer/member-of-pointer (not a local)."""
    if not isinstance(n, dict):
        return False
    k = n.get("kind")
    if k == "CallExpr":
        return False
    if k == "DeclRefExpr":
        d = n.get("referencedDecl", {})
        if d.get("kind") == "VarDecl" and n.get("_global"):
            return True
    if k == "MemberExpr" and n.get("isArrow"):
        return True
    if k == "UnaryOperator" and n.get("opcode") == "*":
        return True
    if k == "ArraySubscriptExpr":
        return True
    return any(reads_state(ch) for ch in n.get("inner", []) or [])


def refs(n, out=None):
    """names of the variables an expression refers to."""
    out = set() if out is None else out
    if isinstance(n, dict):
        if n.get("kind") == "DeclRefExpr" and n.get("referencedDecl", {}).get("kind") in ("VarDecl", "ParmVarDecl"):
            out.add(n["referencedDecl"].get("name", "?"))
        for ch in n.get("inner", []) or []:
            refs(ch, out)
    return out


def side_calls(n):
    out = []
    walk_calls(n, out)
    return [c for c in out if not PURE.match(c)]


class Scan:
    def __init__(self, tu, globals_):
        self.tu, self.globals = tu, globals_
        self.rows, self.fn, self.line, self.file = [], None, 0, ""
        self._last_file, self._last_line = "", 0

    def _upd(self, l):
        """clang prints a location's file / line only when they change from the previous PRINTED location (loc,
        range.begin, range.end; spellingLoc before expansionLoc), so every one is replayed in order."""
        if not l:
            return None
        if "spellingLoc" in l or "expansionLoc" in l:
            self._upd(l.get("spellingLoc"))
            return self._upd(l.get("expansionLoc"))
        if "file" in l:
            self._last_file = l["file"]
        if "line" in l:
            self._last_line = l["line"]
        return (self._last_file, self._last_line)

    def loc(self, n):
        a = self._upd(n.get("loc"))
        b = self._upd(n.get("range", {}).get("begin"))
        self._upd(n.get("range", {}).get("end"))
        fl = b or a
        if fl:
            self.file, self.line = fl
        return self.line

    def mark_globals(self, n):
        if isinstance(n, dict):
            if n.get("kind") == "DeclRefExpr" and n.get("referencedDecl", {}).get("id") in self.globals:
                n["_global"] = True
            for ch in n.get("inner", []) or []:
                self.mark_globals(ch)

    def check(self, n, parts, what):
        calls = [side_calls(p) for p in parts]
        with_calls = [i for i, c in enumerate(calls) if c]
        if len(with_calls) >= 2:
            seq = [c for cs in calls for c in cs]
            rng = any("Rand" in c for c in seq)
            self.rows.append((self.tu, self.fn, self.file, self.line, "call-call", int(rng), what, "|".join(seq)))
        elif len(with_calls) == 1 and not what.startswith("op ") or (len(with_calls) == 1 and what[3:] not in
                                                                        ("=", "+=", "-=", "*=", "/=", "%=", "<<=", ">>=",
                                                                         "&=", "|=", "^=")):
            others = [p for i, p in enumerate(parts) if i != with_calls[0]]
            if any(reads_state(p) for p in others):
                seq = calls[with_calls[0]]
                rng = any("Rand" in c for c in seq)
                # shared: the read goes through a variable the call also receives (the call can write what is read)
                shared = refs(parts[with_calls[0]]) & set().union(*(refs(p) for p in others))
                kind = "call-read-shared" if shared else "call-read"
                self.rows.append((self.tu, self.fn, self.file, self.line, kind, int(rng), what,
                                  "|".join(seq) + ("" if not shared else " @" + ",".join(sorted(shared)))))

    def visit(self, n):
        if not isinstance(n, dict):
            return
        k = n.get("kind")
        self.loc(n)
        if k == "FunctionDecl" and n.get("inner") and any(c.get("kind") == "CompoundStmt" for c in n["inner"]):
            old = self.fn
            self.fn = n.get("name")
            for ch in n["inner"]:
                self.visit(ch)
            self.fn = old
            return
        if self.fn:
            if k == "CallExpr":
                args = (n.get("inner") or [])[1:]
                if len(args) >= 2:
                    self.check(n, args, "args of " + callee(n))
            elif k in ("BinaryOperator", "CompoundAssignOperator") and n.get("opcode") in UNSEQ_BIN:
                self.check(n, n.get("inner") or [], "op " + n.get("opcode"))
        for ch in n.get("inner", []) or []:
            self.visit(ch)


def fn_effects(fn):
    """(writes memory other than its own locals, callees) for one FunctionDecl body."""
    calls, writes = set(), [False]

    def target_local(t):
        while isinstance(t, dict) and t.get("kind") in ("ParenExpr", "ImplicitCastExpr"):
            t = (t.get("inner") or [None])[0]
        return isinstance(t, dict) and t.get("kind") == "DeclRefExpr" and             t.get("referencedDecl", {}).get("kind") in ("VarDecl", "ParmVarDecl") and not t.get("_global")

    def w(n):
        if not isinstance(n, dict):
            return
        k = n.get("kind")
        if k == "CallExpr":
            calls.add(callee(n))
        elif k in ("BinaryOperator", "CompoundAssignOperator") and n.get("opcode", "").endswith("=") and                 n.get("opcode") not in ("==", "!=", "<=", ">="):
            if not target_local((n.get("inner") or [None])[0]):
                writes[0] = True
        elif k == "UnaryOperator" and n.get("opcode") in ("++", "--"):
            if not target_local((n.get("inner") or [None])[0]):
                writes[0] = True
        elif k == "VarDecl" and n.get("storageClass") == "static":
            writes[0] = True  # function-local static state
        for ch in n.get("inner", []) or []:
            w(ch)
    w(fn)
    return writes[0], calls


def scan_tu(src):
    p = subprocess.run(ast_cmd(src), cwd=ROOT, capture_output=True)
    if p.returncode:
        return src, None
    ast = json.loads(p.stdout)
    globals_ = set()
    for d in ast.get("inner", []):
        if d.get("kind") == "VarDecl":
            globals_.add(d.get("id"))
    s = Scan(src, globals_)
    for d in ast.get("inner", []):
        s.mark_globals(d)
    for d in ast.get("inner", []):
        s.visit(d)
    eff = {}
    for d in ast.get("inner", []):
        if d.get("kind") == "FunctionDecl" and any(c.get("kind") == "CompoundStmt" for c in d.get("inner", []) or []):
            wr, cs = fn_effects(d)
            eff[d.get("name")] = (wr, sorted(cs))
    # keep game files only (inline helpers from headers under src/ count; SDK/MSL headers do not)
    rows = [r for r in s.rows if r[2] and not re.search(r"(MSL|libs/dolphin|dolphin/)", r[2].replace("\\", "/"))]
    return src, (sorted(set(rows), key=lambda r: (r[2], r[3])), eff)


def cmd_scan(a):
    tus = sorted({n for _, _, n in int_audit.load_splits()})
    srcs = [f"src/{n}" for n in tus if (ROOT / "src" / n).exists() and n.endswith(".c")]
    rows, failed, eff = [], [], {}
    with cf.ProcessPoolExecutor(a.j) as ex:
        for src, r in ex.map(scan_tu, srcs):
            if r is None:
                failed.append(src)
            else:
                rows += r[0]
                for k, (wr, cs) in r[1].items():
                    o = eff.get(k, (False, set()))
                    eff[k] = (o[0] or wr, set(o[1]) | set(cs))
    # purity fixed point: impure = writes non-local memory, or calls an impure / unknown (no body: SDK, HSD_Rand, ...)
    impure = {k for k, (wr, _) in eff.items() if wr}
    changed = True
    while changed:
        changed = False
        for k, (_, cs) in eff.items():
            if k not in impure and any(c not in eff or c in impure for c in cs if not PURE.match(c)):
                impure.add(k); changed = True
    pure = set(eff) - impure
    def keep(r):
        cs = [c for c in r[7].split(" @")[0].split("|") if c]
        return any(c not in pure and not PURE.match(c) for c in cs)
    before = len(rows)
    rows = [r[:4] + (r[4] + ("" if keep(r) else "-pure"),) + r[5:] for r in rows]
    print(f"evalorder: {sum(1 for r in rows if r[4].endswith('-pure'))} of {before} sites only call pure functions")
    rows = sorted(set(rows), key=lambda r: (r[2], r[3], r[1]))
    with open(a.o, "w", newline="") as f:
        w = csv.writer(f)
        w.writerow(["tu", "func", "file", "line", "kind", "rng", "where", "callees"])
        w.writerows(rows)
    print(f"evalorder: {len(rows)} sites ({sum(1 for r in rows if r[4] == 'call-call')} call-call, "
          f"{sum(1 for r in rows if r[5])} rng) in {len(srcs) - len(failed)} TUs; {len(failed)} TUs failed")


def cmd_dol(a):
    import capstone
    md = capstone.Cs(capstone.CS_ARCH_PPC, capstone.CS_MODE_32 | capstone.CS_MODE_BIG_ENDIAN)
    md.skipdata = True  # jump tables inside .text must not stop the walk
    dolp = a.dol or os.environ.get("INT_AUDIT_DOL") or str(int_audit.WS / "melee/orig/GALE01/sys/main.dol")
    if not Path(dolp).exists():
        sys.exit("evalorder: DOL not found (pass --dol or INT_AUDIT_DOL)")
    read = int_audit.load_dol(dolp)
    syms = int_audit.load_symbols()
    by_addr = {addr: name for addr, _, name in syms}
    by_name = {name: (addr, size) for addr, size, name in syms}
    bl_cache = {}

    def bls(fn):
        if fn not in bl_cache:
            seq = []
            if fn in by_name:
                addr, size = by_name[fn]
                code = read(addr, size)
                if code:
                    for ins in md.disasm(code, addr):
                        if ins.mnemonic == "bl":
                            t = int(ins.op_str.split()[-1].lstrip("$"), 0)
                            seq.append((ins.address, by_addr.get(t, f"{t:08X}")))
            bl_cache[fn] = seq
        return bl_cache[fn]

    rows = list(csv.DictReader(open(a.sites)))
    cursor = {}
    out = []
    for r in rows:
        if r["kind"] != "call-call":
            out.append({**r, "order": "-", "dol": ""})
            continue
        fn = r["func"]
        seq = bls(fn)
        names = r["callees"].split("|")
        if not seq:
            out.append({**r, "order": "no-dol", "dol": ""}); continue
        start = cursor.get(fn, 0)
        pos = []
        for nm in dict.fromkeys(names):
            idx = [i for i, (_, t) in enumerate(seq) if t == nm and i >= start]
            pos.append(idx[0] if idx else None)
        known = [p for p in pos if p is not None]
        if len(known) < 2 or len(known) != len(pos):
            order = "?"
        elif known == sorted(known):
            order = "l2r"
        elif known == sorted(known, reverse=True):
            order = "r2l"
        else:
            order = "mixed"
        if known:
            cursor[fn] = min(known) + 1
        out.append({**r, "order": order,
                    "dol": " ".join(f"{seq[p][0]:08X}:{seq[p][1]}" for p in known)})
    with open(a.o, "w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=list(out[0].keys()))
        w.writeheader(); w.writerows(out)
    from collections import Counter
    print("evalorder dol:", dict(Counter((o["kind"], o["order"]) for o in out)))


def main():
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="cmd", required=True)
    s = sub.add_parser("scan"); s.add_argument("-o", required=True); s.add_argument("-j", type=int, default=os.cpu_count())
    d = sub.add_parser("dol"); d.add_argument("sites"); d.add_argument("-o", required=True); d.add_argument("--dol")
    a = ap.parse_args()
    {"scan": cmd_scan, "dol": cmd_dol}[a.cmd](a)


if __name__ == "__main__":
    main()
