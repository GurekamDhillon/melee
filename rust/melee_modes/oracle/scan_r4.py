#!/usr/bin/env python3
"""Group 6 round 4 scan: functions in gm/mn/if/ty/lb TUs whose every callee is already ported
(in melee_modes, or any other crate's pub fn of the same name), with no non-const file-scope
variable, no bitfield, and a non-pointer return. Iterates to a fixed point.
  python rust/melee_modes/oracle/scan_r4.py   (from the worktree root)"""
import os, re, sys, glob
from pathlib import Path
ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / "rust/melee_types/gen"))
os.chdir(ROOT)
import gen_types as g
ci, K, T = g.ci, g.K, g.T
NAT = [a.replace("x86_64-pc-windows-msvc", "x86_64-pc-windows-gnu") for a in g.NAT] + ["-mno-ms-bitfields"]
mine = set()
for f in glob.glob("rust/melee_modes/src/*.rs"):
    mine |= set(re.findall(r"pub fn ([A-Za-z0-9_]+)", open(f, encoding="utf-8").read()))
other = set()
for f in glob.glob("rust/*/src/**/*.rs", recursive=True):
    if "/raw/" in f.replace("\\", "/") or "melee_modes" in f:
        continue
    other |= set(re.findall(r"pub fn ([A-Za-z0-9_]+)", open(f, encoding="utf-8", errors="replace").read()))
tus = [l.strip() for l in open("docs/rust-port/game_tus.txt") if re.search(r"melee/(gm|mn|if|ty|lb)/", l)]
idx = ci.Index.create()
info = {}
for tp in tus:
    tu = idx.parse(tp, args=NAT)
    here = Path(tp).resolve()
    for c in tu.cursor.get_children():
        if c.kind != K.FUNCTION_DECL or not c.is_definition() or not c.location.file:
            continue
        if Path(str(c.location.file)).resolve() != here:
            continue
        calls, bad = set(), []
        for d in c.walk_preorder():
            if d.kind == K.CALL_EXPR:
                r = d.referenced
                if r is None or r.kind != K.FUNCTION_DECL:
                    bad.append("indirect call")
                else:
                    calls.add(r.spelling)
            elif d.kind == K.DECL_REF_EXPR and d.referenced is not None:
                r = d.referenced
                if r.kind == K.VAR_DECL and r.semantic_parent.kind == K.TRANSLATION_UNIT:
                    if not r.type.is_const_qualified() and not (r.type.get_canonical().kind in (T.CONSTANTARRAY, T.INCOMPLETEARRAY) and r.type.get_canonical().element_type.is_const_qualified()):
                        bad.append("global " + r.spelling)
                elif r.kind == K.FUNCTION_DECL and d.semantic_parent is not None:
                    pass
            elif d.kind == K.MEMBER_REF_EXPR and d.referenced is not None and d.referenced.kind == K.FIELD_DECL and d.referenced.is_bitfield():
                bad.append("bitfield")
            elif d.kind == K.ASM_STMT:
                bad.append("asm")
        rt = c.result_type.get_canonical().kind
        if rt == T.POINTER:
            bad.append("ptr return")
        calls.discard(c.spelling)
        info.setdefault(c.spelling, (tp, calls, bad, len(list(c.get_tokens()))))
done = set(mine)
rounds = []
while True:
    new = [n for n, (tp, calls, bad, nt) in info.items() if n not in done and not bad and calls
           and all(x in done or x in other for x in calls)]
    if not new:
        break
    rounds.append(new); done |= set(new)
for i, r in enumerate(rounds):
    for n in sorted(r):
        tp, calls, bad, nt = info[n]
        print(i, tp, n, nt, ",".join(sorted(x + ("*" if x not in mine and x in other else "") for x in calls)))
# near misses: one unported callee
print("---- near (bad-free, unported callees)")
for n, (tp, calls, bad, nt) in sorted(info.items()):
    if n in done or bad: continue
    miss = [x for x in calls if x not in done and x not in other]
    if len(miss) <= 1:
        print(tp, n, nt, "missing", miss)
