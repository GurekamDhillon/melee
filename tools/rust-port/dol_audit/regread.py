"""Audit 1: registers (and stack slots) read before any path in the function wrote them.

For every function of the retail main.dol: a basic-block CFG over [addr, addr+size) (jump tables
resolved from the lis/addi/lwzx/mtctr/bctr idiom), and a forward dataflow where every register
carries the set of "undefined origins" that may reach it on some path (empty = defined on every path).
Origins: callee-saved caller value at entry, argument/scratch register unset at entry (prototype from
the decomp C), caller-saved register clobbered by a call, r3/f1 after a call to a void function, and
a stack slot loaded before any store. Taint propagates through computation; a tainted value reaching
a store, a load address, a conditional branch, a call argument, an indirect jump or the return value
is a "sink" (observable candidate); one that never reaches a sink is dead (benign).

Prints counts, function names and addresses only (no instruction bytes).
Usage: python -I tools/rust-port/dol_audit/regread.py [--dol P] [--symbols P] [--splits P]
          [--area NAME] [--func NAME] [--all] [--tsv OUT]
"""
import argparse
import os
import re
import sys
from collections import Counter, defaultdict

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import dol as D  # noqa: E402

F, CR, CTR, CA, NREG = D.F, D.CR, D.CTR, D.CA, D.NREG
SIDX = 0x8000  # stack key bias (frames are far below 32 KB)

KIND_TEXT = {
    "callee": "callee-saved caller value",
    "arg": "argument register beyond the prototype",
    "scratch": "scratch register unset at entry",
    "clobber": "caller-saved garbage after a call",
    "voidret": "result register of a void callee",
    "stack": "stack slot read before any store",
}
SINK_ORDER = ("return", "call-arg", "branch", "store", "load-addr", "jump", "other")


class FnResult:
    def __init__(self, f, proto):
        self.f = f
        self.proto = proto
        self.origins = []          # labels: (kind, reg, addr)
        self.reads = defaultdict(int)   # (addr, reg) -> origin mask (direct reads)
        self.sinks = defaultdict(int)   # (addr, kind) -> origin mask
        self.unresolved_bctr = []
        self.r3_unwritten_ret = []       # blr addresses where r3 (or f1) may still be the entry value
        self.notes = []


def resolve_jump_table(dol, ins, bi, f):
    """Targets of the bctr at ins[bi], or None. MWCC: lis rT,hi; addi rT,rT,lo; lwzx rX,rT,rI; mtctr rX; bctr,
    bounded by a preceding cmplwi rI0,N."""
    mt = None
    for j in range(bi - 1, max(-1, bi - 12), -1):
        if ins[j].mn == "mtctr":
            mt = j
            break
    if mt is None:
        return None
    src = (ins[mt].w >> 21) & 31
    tbl_reg = None
    for j in range(mt - 1, max(-1, mt - 12), -1):
        w = ins[j].w
        if ins[j].mn == "lwzx" and ((w >> 21) & 31) == src:
            tbl_reg = (w >> 16) & 31
            idx_reg = (w >> 11) & 31
            lw = j
            break
    if tbl_reg is None:
        return None
    lo = hi = None
    for j in range(lw - 1, max(-1, lw - 16), -1):
        w = ins[j].w
        op = w >> 26
        if lo is None and op == 14 and ((w >> 21) & 31) == tbl_reg and ((w >> 16) & 31) == tbl_reg:
            lo = D._s16(w & 0xFFFF)
        elif op == 15 and ((w >> 21) & 31) == tbl_reg and ((w >> 16) & 31) == 0:
            hi = (w & 0xFFFF) << 16
            break
    if hi is None or lo is None:
        return None
    tbl = (hi + lo) & 0xFFFFFFFF
    n = None
    for j in range(lw - 1, max(-1, lw - 24), -1):
        if ins[j].mn == "cmpli":
            n = ins[j].stval[0] + 1
            break
    out = []
    k = 0
    while k < (n if n else 1024):
        t = dol.word(tbl + 4 * k)
        if t is None or not (f.addr <= t < f.end):
            if n:
                return None
            break
        out.append(t)
        k += 1
    return out or None


def analyse(dol, f, protos, by_addr, defs, decls):
    proto = D.proto_for(f, defs, decls)
    res = FnResult(f, proto)
    ins = D.decode_function(dol, f)
    n = len(ins)
    if n == 0:
        return res
    a2i = {f.addr + 4 * k: k for k in range(n)}

    # ---- blocks
    leaders = {0}
    succ_of = {}
    jt = {}
    for k, i in enumerate(ins):
        if i.kind == "br":
            if f.addr <= i.target < f.end:
                leaders.add(a2i[i.target])
            if k + 1 < n:
                leaders.add(k + 1)
        elif i.kind in ("ret", "bctr"):
            if k + 1 < n:
                leaders.add(k + 1)
            if i.kind == "bctr" and i.cond is None and not i.ctr_dec:
                t = resolve_jump_table(dol, ins, k, f)
                jt[k] = t
                if t:
                    for a in t:
                        leaders.add(a2i[a])
        elif i.kind == "call" and i.target is not None and i.mn == "bcl":
            pass
    lead = sorted(leaders)
    blk_of = {}
    blocks = []
    for bi, s in enumerate(lead):
        e = lead[bi + 1] if bi + 1 < len(lead) else n
        blocks.append((s, e))
        blk_of[s] = bi

    # ---- origins
    olabel = {}
    own = [0] * NREG

    def origin(kind, reg, addr):
        key = (kind, reg, addr)
        o = olabel.get(key)
        if o is None:
            o = len(res.origins)
            olabel[key] = o
            res.origins.append(key)
            if reg is not None and reg >= 0:
                own[reg] |= 1 << o
        return 1 << o

    # ---- entry state
    regs = [0] * NREG
    if proto is None or proto.varargs:
        argg = list(range(3, 11))
        argf = [F + k for k in range(1, 9)]
    else:
        argg, argf = proto.gpr_args()
    defined = {1, 2, 13} | set(argg) | set(argf)
    if proto is not None and proto.varargs:
        defined |= set(D.crf(1))
    for r in range(NREG):
        if r in defined:
            continue
        if r in D.CALLEE_SAVED:
            regs[r] = origin("callee", r, f.addr)
        elif (3 <= r <= 10) or (F + 1 <= r <= F + 8):
            regs[r] = origin("arg", r, f.addr)
        else:
            regs[r] = origin("scratch", r, f.addr)
    raw = list(regs)  # origins whose register still holds the untouched undefined value
    entry = (regs, 0, 0, {}, 0, raw)  # regs, stack-defined mask, sp delta, stack taint {key: mask}, written bits, raw

    ret_cls = proto.ret if proto is not None else None
    callee_saved_set = set(D.CALLEE_SAVED)

    def callee_proto(addr):
        g = by_addr.get(addr)
        if g is None:
            return None
        return protos.get(g.name)

    states = {0: entry}
    work = [0]
    inq = {0}

    def merge(bi, st):
        old = states.get(bi)
        if old is None:
            states[bi] = (list(st[0]), st[1], st[2], dict(st[3]), st[4], list(st[5]))
            if bi not in inq:
                work.append(bi)
                inq.add(bi)
            return
        r0, m0, d0, t0, w0, q0 = old
        r1, m1, d1, t1, w1, q1 = st
        ch = False
        for k in range(NREG):
            v = r0[k] | r1[k]
            if v != r0[k]:
                r0[k] = v
                ch = True
            v = q0[k] | q1[k]
            if v != q0[k]:
                q0[k] = v
                ch = True
        m = m0 & m1
        d = d0 if d0 == d1 else None
        if d0 is not None and d is None:
            ch = True
        w = w0 & w1
        for key, v in t1.items():
            nv = t0.get(key, 0) | v
            if nv != t0.get(key, 0):
                t0[key] = nv
                ch = True
        if m != m0 or w != w0:
            ch = True
        if ch:
            states[bi] = (r0, m, d, t0, w, q0)
            if bi not in inq:
                work.append(bi)
                inq.add(bi)

    def rng(key, nb):
        return ((1 << nb) - 1) << (key + SIDX)

    iters = 0
    while work:
        iters += 1
        if iters > 20000:
            res.notes.append("dataflow iteration cap")
            break
        bi = work.pop()
        inq.discard(bi)
        r, sm, sd, stt, wr, q = states[bi]
        r = list(r)
        q = list(q)
        stt = dict(stt)
        s, e = blocks[bi]
        fall = True
        for k in range(s, e):
            i = ins[k]
            mn = i.mn
            # --- uses
            t_in = 0
            uses = i.uses
            if i.kind == "store" and i.mem and i.mem[0] == 1 and i.mem[1] is None:
                # saves of callee-saved caller values to the frame are not reads
                vals = [v for v in i.stval if not (v in callee_saved_set and r[v] and (r[v] & ~own[v]) == 0)]
                uses = [u for u in uses if u not in i.stval or u in vals]
                if mn == "stmw" and len(vals) != len(i.stval):
                    uses = [u for u in uses if u not in i.stval]
            for u in uses:
                tu = r[u]
                if tu:
                    dr = q[u]
                    if dr:
                        res.reads[(i.addr, u)] |= dr
                    t_in |= tu
            # --- stack load / store
            ld_taint = 0
            if i.mem is not None and i.mem[0] == 1 and i.mem[1] is None and sd is not None:
                key = sd + i.mem[2]
                nb = i.mem[3]
                if key < 0 and key + SIDX >= 0:
                    if i.kind == "store":
                        sm |= rng(key, nb)
                        vt = 0
                        for v in uses:
                            if v in i.stval:
                                vt |= r[v]
                        if vt:
                            stt[key] = vt
                        else:
                            stt.pop(key, None)
                    elif i.kind == "load":
                        want = rng(key, min(nb, max(0, -key)))
                        if (sm & want) != want:
                            ld_taint |= origin("stack", -1, i.addr)
                        ld_taint |= stt.get(key, 0)
            # --- sinks
            if t_in:
                kd = i.kind
                if kd == "store":
                    addr_t = 0
                    for u in uses:
                        if u not in i.stval:
                            addr_t |= r[u]
                    val_t = t_in & ~addr_t if addr_t == 0 else t_in
                    if i.mem[0] == 1 and addr_t == 0:
                        pass  # store to the frame: tracked through the stack-slot taint
                    else:
                        res.sinks[(i.addr, "store")] |= t_in
                elif kd == "load":
                    res.sinks[(i.addr, "load-addr")] |= t_in
                elif kd == "br" or kd == "ret" or kd == "bctr" or kd == "call" or kd == "callind":
                    if i.cond is not None and r[i.cond]:
                        res.sinks[(i.addr, "branch")] |= r[i.cond]
                    if kd in ("bctr", "callind") and r[CTR]:
                        res.sinks[(i.addr, "jump")] |= r[CTR]
                elif kd == "other" and mn not in ("twi", "tw", "dcache"):
                    res.sinks[(i.addr, "other")] |= t_in
            # --- defs
            out_t = t_in | ld_taint
            if i.kind == "store":
                out_t = r[i.mem[0]] if (i.mem and i.mem[0] is not None) else 0  # update form: address only
            for dreg in i.defs:
                r[dreg] = out_t
                q[dreg] = 0
            if i.kind in ("load",) and ld_taint == 0 and i.mem and i.mem[0] == 1 and mn == "lmw":
                pass
            # mfcr / mflr define cleanly (CR save; LR is not tracked)
            if mn in ("mfcr", "mflr"):
                r[i.defs[0]] = 0
            if mn in ("mtcrf",):
                for dreg in i.defs:
                    r[dreg] = t_in
            # crclr/crset idiom
            if mn in ("crxor", "creqv") and ((i.w >> 16) & 31) == ((i.w >> 11) & 31):
                r[i.defs[0]] = 0
            # --- written bits for r3/f1/r4
            for dreg in i.defs:
                if dreg == 3:
                    wr |= 1
                elif dreg == F + 1:
                    wr |= 2
                elif dreg == 4:
                    wr |= 4
            # --- stack pointer
            if 1 in i.defs:
                if mn == "stwu" and i.mem[0] == 1:
                    if sd is not None:
                        sd += i.mem[2]
                        sm |= rng(sd, 4) if sd + SIDX >= 0 else 0
                elif mn == "addi" and ((i.w >> 16) & 31) == 1:
                    if sd is not None:
                        sd += D._s16(i.w & 0xFFFF)
                elif mn == "lwz" and i.mem[0] == 1 and i.mem[2] == 0:
                    sd = 0
                else:
                    sd = None
                r[1] = 0
            # address of a local escapes: whatever it points at may be written by anyone
            if sd is not None and ((mn == "addi" and ((i.w >> 16) & 31) == 1 and 1 not in i.defs)
                                   or (mn == "mr" and ((i.w >> 21) & 31) == 1 and 1 not in i.defs)):
                k0 = sd + (D._s16(i.w & 0xFFFF) if mn == "addi" else 0)
                if k0 < 0:
                    sm |= rng(k0, -k0)
                    r[i.defs[0]] = 0
            if i.mem is not None and i.mem[0] == 1 and i.mem[1] is not None and i.kind == "store" and sd is not None:
                sm |= rng(sd, -sd) if sd < 0 else 0

            # --- control flow
            kd = i.kind
            if kd in ("call", "callind") or (kd == "br" and not (f.addr <= (i.target or 0) < f.end)):
                tail = kd == "br"
                cp = callee_proto(i.target) if kd != "callind" else None
                if cp is not None and not cp.is_asm:
                    g, fl = cp.gpr_args()
                    for a in g + fl:
                        if r[a]:
                            dr = q[a]
                            if dr:
                                res.reads[(i.addr, a)] |= dr
                            res.sinks[(i.addr, "call-arg")] |= r[a]
                if tail:
                    if i.cond is None:
                        fall = False
                        break
                    continue
                for c in D.CALLER_SAVED:
                    r[c] = q[c] = origin("clobber", c, i.addr)
                if cp is None or cp.is_asm:
                    for c in (3, 4, F + 1):
                        r[c] = q[c] = 0
                else:
                    rc = cp.ret
                    if rc in ("int", "struct"):
                        r[3] = q[3] = 0
                    elif rc == "int64":
                        r[3] = r[4] = q[3] = q[4] = 0
                    elif rc == "float":
                        r[F + 1] = q[F + 1] = 0
                    if rc == "void":
                        r[F + 1] = q[F + 1] = origin("voidret", F + 1, i.addr)
                    if rc in ("void", "float"):
                        r[3] = q[3] = origin("voidret", 3, i.addr)
                wr |= 7
                if sd is not None:
                    pass
                continue
            if kd == "ret":
                if ret_cls in ("int", "struct", "int64", "float"):
                    rr = [F + 1] if ret_cls == "float" else ([3, 4] if ret_cls == "int64" else [3])
                    for a in rr:
                        if r[a]:
                            dr = q[a]
                            if dr:
                                res.reads[(i.addr, a)] |= dr
                            res.sinks[(i.addr, "return")] |= r[a]
                    bit = 2 if ret_cls == "float" else 1
                    if not (wr & bit) and not r[rr[0]]:
                        res.r3_unwritten_ret.append(i.addr)
                if i.cond is None:
                    fall = False
                    break
                continue
            if kd == "bctr":
                if i.cond is None and not i.ctr_dec:
                    t = jt.get(k)
                    if t:
                        st = (r, sm, sd, stt, wr, q)
                        for a in t:
                            merge(blk_of[a2i[a]], st)
                    else:
                        res.unresolved_bctr.append(i.addr)
                    fall = False
                    break
                continue
            if kd == "br":
                tk = a2i[i.target]
                st = (r, sm, sd, stt, wr, q)
                merge(blk_of[tk], st)
                if i.cond is None and not i.ctr_dec:
                    fall = False
                    break
        if fall and e < n:
            merge(blk_of[e], (r, sm, sd, stt, wr, q))
    return res


def summarize(res):
    """Per origin: (label, direct read addrs, sink kinds/addrs)."""
    out = []
    reads_by_o = defaultdict(list)
    for (addr, reg), m in res.reads.items():
        o = 0
        while m:
            if m & 1:
                reads_by_o[o].append((addr, reg))
            m >>= 1
            o += 1
    sinks_by_o = defaultdict(list)
    for (addr, kind), m in res.sinks.items():
        o = 0
        while m:
            if m & 1:
                sinks_by_o[o].append((addr, kind))
            m >>= 1
            o += 1
    for o, lab in enumerate(res.origins):
        kind = lab[0]
        rd = sorted(reads_by_o.get(o, []))
        sk = sorted(sinks_by_o.get(o, []))
        if kind == "stack":
            rd = [(lab[2], -1)]
            if not sk:
                # a stack-slot origin is attached to its load; its own reads are its sinks
                pass
        elif not rd:
            continue
        out.append((lab, rd, sk))
    return out


def hit_rows(res):
    """Flattened report rows: one per (function, origin)."""
    rows = []
    p = res.proto
    seen_ret = set()
    for lab, rd, sk in summarize(res):
        kind, reg, oaddr = lab
        sink_kinds = sorted({k for _, k in sk}, key=SINK_ORDER.index)
        if "return" in sink_kinds and kind != "stack":
            cls = "missing return value" if reg in (3, F + 1, 4) and kind in ("voidret", "clobber", "scratch", "arg", "callee") and all(k == "return" for _, k in sk) else KIND_TEXT[kind]
        else:
            cls = KIND_TEXT[kind]
        if cls == "missing return value":
            if reg in seen_ret:
                continue
            seen_ret.add(reg)
            kind = "missingret"
        rows.append({
            "func": res.f.name, "addr": res.f.addr, "area": res.f.area, "tu": res.f.tu, "f": res.f,
            "kind": kind, "class": cls, "reg": "stack" if kind == "stack" else D.regname(reg),
            "origin_at": oaddr, "first_read": rd[0][0] if rd else oaddr, "nreads": len(rd),
            "sinks": sink_kinds, "first_sink": sk[0][0] if sk else None,
            "observable": bool(sk), "asm": bool(p and p.is_asm), "proto": bool(p),
        })
    for a in res.r3_unwritten_ret:
        if p is not None and p.body is not None and not re.search(r"\breturn\b\s*[^;\s]", p.body):
            rows.append({
                "func": res.f.name, "addr": res.f.addr, "area": res.f.area, "tu": res.f.tu, "f": res.f,
                "kind": "noreturn", "class": "missing return value (no return statement; r3/f1 = entry value)",
                "reg": "f1" if p.ret == "float" else "r3", "origin_at": res.f.addr, "first_read": a, "nreads": 1,
                "sinks": ["return"], "first_sink": a, "observable": True, "asm": p.is_asm, "proto": True,
            })
            break
    return rows


def run(args):
    dol = D.Dol(args.dol)
    funcs = D.load_functions(args.symbols, args.splits)
    defs, decls = D.load_prototypes()
    by_addr = {f.addr: f for f in funcs}
    protos = {}
    for f in funcs:
        p = D.proto_for(f, defs, decls)
        if p is not None:
            protos[f.name] = p
    sel = funcs
    if args.func:
        sel = [f for f in funcs if re.fullmatch(args.func, f.name)]
    if args.area:
        sel = [f for f in sel if f.area == args.area]
    rows = []
    stats = Counter()
    unresolved = []
    for f in sel:
        res = analyse(dol, f, protos, by_addr, defs, decls)
        stats["functions"] += 1
        if res.proto is None:
            stats["no prototype"] += 1
        if res.unresolved_bctr:
            unresolved.append((f, res.unresolved_bctr))
        rows += hit_rows(res)
    caller_use(dol, funcs, protos, rows)
    port_status(rows)
    for r in rows:
        if not r["observable"]:
            r["verdict"] = "benign: value dead"
        elif r["kind"] in ("missingret", "noreturn") and r.get("callers_use") == 0:
            r["verdict"] = "benign: no caller reads the result"
        elif r["asm"]:
            r["verdict"] = "asm function (hand-written)"
        else:
            r["verdict"] = "observable candidate"
    return funcs, rows, stats, unresolved


def _uses_after(ins_by_addr, start, reg, func, depth=0, seen=None):
    """Does any path from `start` (inside `func`) read `reg` before writing it? 1 yes, 0 no, None unknown."""
    if seen is None:
        seen = set()
    a = start
    steps = 0
    while func.addr <= a < func.end and steps < 80:
        if a in seen:
            return 0
        seen.add(a)
        i = ins_by_addr(a)
        if reg in i.uses and not (i.kind == "store" and i.mem and i.mem[0] == 1 and reg in i.stval and False):
            return 1
        if i.kind in ("call", "callind"):
            return 0
        if reg in i.defs:
            return 0
        if i.kind == "ret":
            return None if i.cond is None else _uses_after(ins_by_addr, a + 4, reg, func, depth, seen)
        if i.kind == "bctr":
            return None
        if i.kind == "br":
            if not (func.addr <= i.target < func.end):
                return None
            if i.cond is None and not i.ctr_dec:
                a = i.target
                steps += 1
                continue
            if depth > 6:
                return None
            x = _uses_after(ins_by_addr, i.target, reg, func, depth + 1, seen)
            if x:
                return x
        a += 4
        steps += 1
    return None


def caller_use(dol, funcs, protos, rows):
    want = {}
    for r in rows:
        if r["kind"] in ("missingret", "noreturn"):
            want[r["addr"]] = D.F + 1 if r["reg"] == "f1" else 3
    if not want:
        return
    cache = {}

    def ins_at(a):
        i = cache.get(a)
        if i is None:
            i = D.decode(a, dol.word(a) or 0)
            cache[a] = i
        return i
    sites = defaultdict(list)
    for f in funcs:
        for a in range(f.addr, f.end, 4):
            w = dol.word(a)
            if w is not None and (w >> 26) == 18 and (w & 1):
                i = ins_at(a)
                if i.target in want:
                    sites[i.target].append((f, a))
    for r in rows:
        if r["addr"] in want:
            reg = want[r["addr"]]
            n_use = n_unk = 0
            for f, a in sites.get(r["addr"], []):
                u = _uses_after(ins_at, a + 4, reg, f)
                if u == 1:
                    n_use += 1
                elif u is None:
                    n_unk += 1
            r["callers"] = len(sites.get(r["addr"], []))
            r["callers_use"] = n_use + n_unk if n_use == 0 and n_unk else n_use
            r["callers_unknown"] = n_unk


PORT_NOTE = re.compile(r"uninit|garbage|caller'?s\s+[rf]\d+|leftover|stale|undefined|unset|never set|never written|"
                       r"no return|missing return|returns whatever|zero-init|auto-var-init|zeroes locals|"
                       r"console (?:value|reads|returns|passes)|\b[rf]\d{1,2}\b[^.]*(?:caller|uninit|stale|garbage)", re.I)
ZERO_WORDS = re.compile(r"zero|auto-var-init|-ftrivial", re.I)
CONSOLE_WORDS = re.compile(r"console|caller'?s|stale|reproduc|keeps? the|same as the console", re.I)


def port_status(rows):
    fns = D.load_rust_functions()
    for r in rows:
        rf = D.rust_port_of(fns, r["f"])
        if rf is None:
            r["port"] = "-"
            r["port_note"] = ""
            r["port_value"] = "no Rust function"
            continue
        r["port"] = "%s:%d" % (rf.path, rf.line)
        cm = [c for c in re.findall(r"//[^\n]*|/\*.*?\*/", rf.raw, re.S) if PORT_NOTE.search(c)]
        r["port_note"] = (" | ".join(" ".join(c.split())[:160] for c in cm[:2])) if cm else ""
        txt = " ".join(cm)
        if not cm:
            r["port_value"] = "no comment (not checked)"
        elif CONSOLE_WORDS.search(txt) and not ZERO_WORDS.search(txt):
            r["port_value"] = "console value (per comment)"
        elif ZERO_WORDS.search(txt) and not CONSOLE_WORDS.search(txt):
            r["port_value"] = "0 (per comment)"
        else:
            r["port_value"] = "see comment"


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--dol", default=D.DOL_PATH)
    ap.add_argument("--symbols", default=D.SYM_PATH)
    ap.add_argument("--splits", default=D.SPLIT_PATH)
    ap.add_argument("--area")
    ap.add_argument("--func")
    ap.add_argument("--all", action="store_true", help="list every hit, all areas, dead ones too")
    ap.add_argument("--tsv", help="write every row to this TSV (names/addresses only)")
    args = ap.parse_args()
    funcs, rows, stats, unresolved = run(args)

    print("functions analysed %d, without a C prototype %d, bctr not resolved to a table %d (in %d functions)" % (
        stats["functions"], stats["no prototype"], sum(len(u[1]) for u in unresolved), len(unresolved)))
    tot = Counter((r["class"], r["observable"]) for r in rows)
    print("\nhits (one per function x origin):", len(rows), " observable-candidate:", sum(r["observable"] for r in rows))
    for (c, o), k in sorted(tot.items()):
        print("  %-62s %-10s %6d" % (c, "sink" if o else "dead", k))
    print("\nby area (all / observable-candidate / in asm functions):")
    for a in D.AREA_ORDER:
        rr = [r for r in rows if r["area"] == a]
        if rr:
            print("  %-20s %6d %6d %6d" % (a, len(rr), sum(r["observable"] for r in rr), sum(r["asm"] for r in rr)))
    show = rows if args.all else [r for r in rows if r["area"] in D.ONLINE_AREAS]
    print("\nhits%s:" % ("" if args.all else " in the online-relevant areas"))
    for a in D.AREA_ORDER:
        rr = [r for r in show if r["area"] == a]
        if not rr:
            continue
        print("\n[%s] %d" % (a, len(rr)))
        for r in sorted(rr, key=lambda r: (not r["observable"], r["addr"], r["first_read"])):
            print("  %08X %-34s %-6s %-40s read@%08X %-26s %-22s port: %s" % (
                r["addr"], r["func"], r["reg"], r["class"][:40], r["first_read"],
                ("sinks=" + ",".join(r["sinks"])) if r["sinks"] else "dead",
                {"observable candidate": "OBSERVABLE?", "benign: value dead": "benign(dead)",
                 "benign: no caller reads the result": "benign(result unused)"}.get(r["verdict"], r["verdict"]),
                r.get("port_value", "")))
    if unresolved:
        print("\nunresolved bctr (treated as an exit):")
        for f, l in unresolved:
            if args.all or f.area in D.ONLINE_AREAS:
                print("  %08X %-34s %s" % (f.addr, f.name, " ".join("%08X" % a for a in l)))
    if args.tsv:
        with open(args.tsv, "w", encoding="utf-8") as o:
            keys = ["func", "addr", "area", "tu", "kind", "class", "reg", "origin_at", "first_read", "nreads", "sinks",
                    "first_sink", "observable", "asm", "proto", "verdict", "callers", "callers_use", "port", "port_value", "port_note"]
            o.write("\t".join(keys) + "\n")
            for r in rows:
                o.write("\t".join(("%08X" % r[k]) if k in ("addr", "origin_at", "first_read", "first_sink") and r.get(k) is not None
                                  else (",".join(r[k]) if isinstance(r.get(k), list) else str(r.get(k, ""))) for k in keys) + "\n")


if __name__ == "__main__":
    main()
