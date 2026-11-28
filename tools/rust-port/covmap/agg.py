#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Coverage map: which retail main.dol functions run in real Slippi games, and which the live referee checks
(docs/rust-port/notes/coverage-map.md).
   python -I agg.py RUN_DIR SUBSET.tsv OUT_DIR
RUN_DIR/count/*.tsv  fncounts.py per replay (calls per retail-named Rust function, cov build)
RUN_DIR/ref/*.txt    live referee reports (MELEE_REFEREE_NATIVE) of the same replays
SUBSET.tsv           pick.py output (per-replay content: stage, internal characters, item kinds)
Writes OUT_DIR/functions.csv (one row per executed retail function), OUT_DIR/summary.txt, OUT_DIR/hot_unchecked.md."""
import bisect, collections, csv, glob, os, re, sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
sys.path.insert(0, HERE)
import names  # noqa: E402

RENDER_TU = r"^(sysdolphin/baselib/(dobj|tobj|state|tev|mobj|lobj|cobj|pobj|displayfunc|fog|shadow|texp|texpdag|psdisp|psdisptev|video|wobj)\.c|dolphin/(gx|vi)/|melee/ft/ftdrawcommon|melee/ft/ftmaterial|melee/ft/ftafterimage|melee/it/itdraw|melee/it/itmaterial|melee/gr/grdisplay|melee/gr/grmaterial|melee/lb/lbgx|melee/lb/lbshadow|melee/lb/lbrefract|melee/lb/lbspdisplay|melee/ty/tydisplay)"
FAMILIES = [
    ("render (no sim state)", RENDER_TU),
    ("hit/damage", r"^(melee/ft/ftcoll|melee/lb/lbcollision|melee/it/ithitbox|melee/ft/kinds/ftCommon/ftCo_Damage)"),
    ("camera", r"^(melee/cm/|melee/ft/ftcamera)"),
    ("collision/mp", r"^(melee/mp/|melee/ft/ftcliffcommon|melee/it/itgroundcoll|melee/it/itmaplib|melee/it/itcoll)"),
    ("common fighter states", r"^melee/ft/kinds/ftCommon/"),
    ("fighter core", r"^melee/ft/[^/]+\.c"),
    ("items: per-kind (it/kinds)", r"^melee/it/kinds/"),
    ("items: core", r"^melee/it/"),
    ("stages", r"^melee/gr/"),
    ("effects/particles", r"^(melee/ef/|sysdolphin/baselib/(particle|generator|psappsrt)\.c)"),
    ("HUD (if)", r"^melee/if/"),
    ("HSD scene graph + animation", r"^sysdolphin/baselib/(jobj|aobj|fobj|robj|spline|mtx|quatlib|class|object|objalloc|bytecode|archive)\.c"),
    ("gobj/proc", r"^sysdolphin/baselib/gobj"),
    ("RNG", r"^sysdolphin/baselib/random"),
    ("audio", r"^(sysdolphin/baselib/(synth|axdriver)|dolphin/(ax|axfx|ai|dsp|ar)/|melee/lb/lbaudio_ax|melee/sfx/)"),
    ("math (MSL, lbvector, PSMTX)", r"^(MSL/(math|trigf|float)|melee/lb/(lbvector|lbtrigf)|dolphin/mtx/|Runtime/)"),
    ("player (pl)", r"^melee/pl/"),
    ("lb misc", r"^melee/lb/"),
    ("modes (gm/mn)", r"^melee/(gm|mn)/"),
    ("HSD misc", r"^sysdolphin/"),
    ("SDK misc", r"^(dolphin/|MSL/)"),
]


def family(tu):
    if tu is None:
        return "unknown TU"
    m = re.match(r"melee/ft/kinds/ft([A-Za-z]+)/", tu)
    if m and m.group(1) != "Common":
        return f"character: {m.group(1)}"
    for fam, rx in FAMILIES:
        if re.match(rx, tu):
            return fam
    return "other"


def splits():
    """retail .text ranges of every TU (config/GALE01/splits.txt): [(start, end, tu)]"""
    out, tu = [], None
    for l in open(os.path.join(ROOT, "config/GALE01/splits.txt"), encoding="utf-8"):
        if l and not l[0].isspace() and l.rstrip().endswith(":"):
            tu = l.rstrip()[:-1]
        m = re.match(r"\s+\.text\s+start:0x([0-9A-F]+) end:0x([0-9A-F]+)", l)
        if m and tu:
            out.append((int(m.group(1), 16), int(m.group(2), 16), tu))
    return sorted(out)


def symbols():
    out = {}
    for l in open(os.path.join(ROOT, "config/GALE01/symbols.txt"), encoding="utf-8"):
        m = re.match(r"(\S+) = (\S+):0x([0-9A-F]+); // type:function size:0x([0-9A-F]+)", l)
        if m:
            out[m.group(1)] = (int(m.group(3), 16), int(m.group(4), 16))
    return out


def hooked_in_source():
    nat, cap = set(), set()
    rx_n = re.compile(r"referee_native!\(\s*([A-Za-z0-9_]+)")
    rx_c = re.compile(r"referee_hook!\(\s*([A-Za-z0-9_]+)")
    for p in glob.glob(os.path.join(ROOT, "rust", "**", "*.rs"), recursive=True):
        if os.sep + "melee_core" + os.sep in p:
            continue
        t = open(p, encoding="utf-8", errors="replace").read()
        nat.update(rx_n.findall(t))
        cap.update(rx_c.findall(t))
    return nat, cap


def main():
    run, sub, out = sys.argv[1], sys.argv[2], sys.argv[3]
    os.makedirs(out, exist_ok=True)
    syms = symbols()
    sp = splits()
    sp_starts = [x[0] for x in sp]

    def tu_of(addr):
        i = bisect.bisect_right(sp_starts, addr) - 1
        return sp[i][2] if i >= 0 and sp[i][0] <= addr < sp[i][1] else None
    items = names.items(ROOT)
    nat, cap = hooked_in_source()
    content = {}
    for l in list(open(sub, encoding="utf-8"))[1:]:
        r = l.rstrip("\n").split("\t")
        b = (r[0] + "_" + r[1][:-4]).replace("/", "_").replace(" ", "_")
        el = {f"stage:{names.STAGES.get(int(r[3]), r[3])}"}
        el |= {f"char:{names.INTERNAL[int(c)] if int(c) < 27 else c}" for c in r[4].split(",") if c}
        el |= {f"item:{items.get(int(c), 'kind ' + c)}" for c in r[5].split(",") if c}
        content[b] = el
    calls = collections.Counter()
    reps = collections.defaultdict(set)
    where = {}
    nrep = 0
    statuses = collections.Counter()
    for p in sorted(glob.glob(os.path.join(run, "count", "*.tsv"))):
        b = os.path.basename(p)[:-4]
        st = open(p[:-4] + ".status", errors="replace").read().split()
        statuses[st[2] if len(st) > 2 else "crashed (dropped)"] += 1
        if len(st) <= 2:  # the cov build crashed: no exit, so no complete profile
            continue
        nrep += 1
        for l in list(open(p, encoding="utf-8"))[1:]:
            nm, crate, fi, ln, n = l.rstrip("\n").split("\t")
            n = int(n)
            if nm not in syms:
                continue
            where.setdefault(nm, (crate, fi))
            if n:
                calls[nm] += n
                reps[nm].add(b)
    # referee
    S = collections.defaultdict(lambda: collections.Counter())
    reasons = collections.defaultdict(collections.Counter)
    mism = collections.defaultdict(list)
    interp = set()
    by_addr = sorted((a, a + sz, nm) for nm, (a, sz) in syms.items())
    starts = [x[0] for x in by_addr]
    caddr = set()
    nref = 0
    for p in sorted(glob.glob(os.path.join(run, "ref", "*.txt"))):
        nref += 1
        for l in open(p, encoding="utf-8", errors="replace"):
            f = l.rstrip("\n").split("\t")
            if f[0] == "S":
                for kv in f[2:]:
                    k, _, v = kv.partition(" ")
                    if k in ("calls", "compared", "match", "mismatch", "notcheck"):
                        S[f[1]][k] += int(v)
            elif f[0] == "N" and len(f) >= 5:
                reasons[f[1]][re.sub(r"[0-9A-F]{8}", "X", f[4])[:90]] += int(f[2])
            elif f[0] == "M" and len(mism[f[1]]) < 3:
                mism[f[1]].append(os.path.basename(p)[:-4] + ": " + f[2][:160])
            elif f[0] == "C":
                caddr.add(int(f[1], 16))
    for a in caddr:
        i = bisect.bisect_right(starts, a) - 1
        if i >= 0 and by_addr[i][0] <= a < by_addr[i][1]:
            interp.add(by_addr[i][2])
    hooked = nat | set(S)

    def verdict(nm):
        if nm in hooked:
            s = S.get(nm)
            if not s or not s["calls"]:
                return "hooked, not reached"
            if s["mismatch"]:
                return "mismatch"
            if s["compared"]:
                return "match"
            return "not checkable"
        if nm in cap:
            return "capture hook only"
        return "unchecked"

    rows = []
    for nm in calls:
        crate, fi = where[nm]
        rs = reps[nm]
        trig = set.intersection(*(content.get(b, set()) for b in rs)) if rs else set()
        if len(rs) == nrep:
            trig = set()
        s = S.get(nm, collections.Counter())
        rows.append({"function": nm, "address": f"{syms[nm][0]:08X}", "size": syms[nm][1], "family": family(tu_of(syms[nm][0])), "tu": tu_of(syms[nm][0]),
                     "rust_file": f"{crate}/src/{fi}", "calls": calls[nm], "replays": len(rs),
                     "trigger": ";".join(sorted(trig))[:200] if trig else ("all" if len(rs) == nrep else "mixed"),
                     "verdict": verdict(nm), "ref_calls": s["calls"], "ref_compared": s["compared"], "ref_match": s["match"],
                     "ref_mismatch": s["mismatch"], "ref_notcheck": s["notcheck"],
                     "interpreted_callee": int(nm in interp and nm not in hooked),
                     "top_notcheck_reason": (reasons[nm].most_common(1)[0][0] if reasons.get(nm) else "")})
    rows.sort(key=lambda r: -r["calls"])
    with open(os.path.join(out, "functions.csv"), "w", newline="") as fo:
        w = csv.DictWriter(fo, fieldnames=list(rows[0]))
        w.writeheader()
        w.writerows(rows)
    # summary
    V = collections.Counter(r["verdict"] for r in rows)
    VC = collections.Counter()
    for r in rows:
        VC[r["verdict"]] += r["calls"]
    tot, totc = len(rows), sum(r["calls"] for r in rows)
    checked = [r for r in rows if r["verdict"] in ("match", "mismatch", "not checkable")]
    L = [f"replays: count pass {nrep} ({dict(statuses)}), referee pass {nref}",
         f"retail functions executed: {tot}; calls {totc:,}",
         f"hooked in source (referee_native!): {len(nat)}; capture-mode hooks (referee_hook!): {len(cap)}"]
    for k in ("match", "mismatch", "not checkable", "hooked, not reached", "capture hook only", "unchecked"):
        L.append(f"  {k:<22} functions {V[k]:>5} ({100 * V[k] / tot:5.2f}%)   calls {VC[k]:>14,} ({100 * VC[k] / totc:6.2f}%)")
    cm = V["match"] + V["mismatch"]
    L.append(f"checked (live, compared at least once): {cm} functions = {100 * cm / tot:.2f}% unweighted, "
             f"{100 * (VC['match'] + VC['mismatch']) / totc:.2f}% call-weighted")
    L.append(f"hooked and reached (incl. not checkable): {len(checked)} = {100 * len(checked) / tot:.2f}% unweighted, "
             f"{100 * sum(r['calls'] for r in checked) / totc:.2f}% call-weighted")
    ic = [r for r in rows if r["interpreted_callee"]]
    L.append(f"unhooked functions whose retail code ran as an interpreted callee of a checked call: {len(ic)} "
             f"({100 * sum(r['calls'] for r in ic) / totc:.2f}% of calls)")
    fam = collections.defaultdict(lambda: [0, 0, 0, 0])
    for r in rows:
        a = fam[r["family"]]
        a[0] += 1; a[1] += r["calls"]
        if r["verdict"] in ("match", "mismatch"):
            a[2] += 1; a[3] += r["calls"]
    L.append("")
    L.append("family | executed | calls | checked | checked calls")
    for k, a in sorted(fam.items(), key=lambda x: -x[1][1]):
        L.append(f"{k} | {a[0]} | {a[1]:,} | {a[2]} | {100 * a[3] / max(a[1], 1):.1f}%")
    L.append("")
    L.append("hooked functions: verdicts")
    for r in sorted((r for r in rows if r["verdict"] not in ("unchecked", "capture hook only")), key=lambda r: r["function"]):
        L.append(f"  {r['function']:<36} {r['verdict']:<20} calls {r['ref_calls']:>6} compared {r['ref_compared']:>6} "
                 f"mismatch {r['ref_mismatch']:>4} notcheck {r['ref_notcheck']:>6}  {r['top_notcheck_reason'][:70]}")
    for nm in sorted(hooked - set(calls)):
        L.append(f"  {nm:<36} hooked, never executed in these replays")
    L.append("")
    L.append("mismatches (first 3 per function):")
    for nm, ms in sorted(mism.items()):
        for m in ms:
            L.append(f"  {nm}: {m}")
    open(os.path.join(out, "summary.txt"), "w").write("\n".join(L) + "\n")
    # gameplay view: the render family writes no sim state
    NONSIM = ("render (no sim state)", "audio")
    sim = [r for r in rows if r["family"] not in NONSIM]
    sc = sum(r["calls"] for r in sim)
    sm = [r for r in sim if r["verdict"] in ("match", "mismatch")]
    L.insert(9, f"sim view (render and audio families dropped): {len(sim)} functions, {sc:,} calls; checked {len(sm)} = "
                f"{100 * len(sm) / len(sim):.2f}% unweighted, {100 * sum(r['calls'] for r in sm) / sc:.2f}% call-weighted")
    # bulk-hook reach: executed, unchecked functions that one table-driven hook would cover
    rs_ms = open(os.path.join(ROOT, "rust/melee_link/src/retail_ms.rs"), encoding="utf-8").read()
    ms = set(re.findall(r"tu::[A-Za-z0-9_]+::([A-Za-z0-9_]+)\(cast", rs_ms))
    rs_it = open(os.path.join(ROOT, "rust/melee_link/src/retail_it.rs"), encoding="utf-8").read()
    itl = set(re.findall(r"tu::[A-Za-z0-9_]+::([A-Za-z0-9_]+)\(", rs_it))
    ist = set()
    for p in glob.glob(os.path.join(ROOT, "rust/melee_items/src/tu/*.rs")):
        for l in open(p, encoding="utf-8", errors="replace"):
            if "st_row(" in l:
                ist.update(x for x in re.findall(r"\b([A-Za-z][A-Za-z0-9_]+)\b", l) if x in syms)
    unc = [r for r in rows if r["verdict"] in ("unchecked", "capture hook only")]
    uc = sum(r["calls"] for r in unc)
    L.append("")
    L.append(f"bulk-hook reach (executed, unchecked: {len(unc)} functions, {uc:,} calls; sim only "
             f"{sum(1 for r in unc if r['family'] not in NONSIM)})")
    for label, S_ in (("MotionState table callbacks (retail_ms.rs, gen_retail_ms.py)", ms),
                      ("item ItemStateTable rows (st_row in melee_items tu/)", ist),
                      ("item logic tables (retail_it.rs, gen_retail_it.py)", itl)):
        hit = [r for r in unc if r["function"] in S_]
        L.append(f"  {label}: {len(S_)} names in the tables; executed+unchecked {len(hit)}, calls {sum(r['calls'] for r in hit):,}")
    open(os.path.join(out, "summary.txt"), "w").write("\n".join(L) + "\n")
    # hot unchecked, grouped: the 200 with most calls among sim functions, then the top 10 of every family
    hot = [r for r in unc if r["family"] not in NONSIM][:200]

    def table(rs, H):
        H.append("| function | calls | replays | trigger |")
        H.append("|---|---:|---:|---|")
        for r in rs:
            H.append(f"| {r['function']} | {r['calls']:,} | {r['replays']} | {r['trigger'][:80]} |")
    g = collections.defaultdict(list)
    for r in hot:
        g[r["family"]].append(r)
    H = []
    for k, rs in sorted(g.items(), key=lambda x: -sum(r["calls"] for r in x[1])):
        H.append(f"\n**{k}** ({len(rs)} of the 200, {sum(r['calls'] for r in rs):,} calls)\n")
        table(rs, H)
    open(os.path.join(out, "hot_unchecked.md"), "w").write("\n".join(H) + "\n")
    g = collections.defaultdict(list)
    for r in unc:
        g[r["family"]].append(r)
    H = []
    for k, rs in sorted(g.items(), key=lambda x: -sum(r["calls"] for r in x[1])):
        H.append(f"\n**{k}** ({len(rs)} unchecked, {sum(r['calls'] for r in rs):,} calls; top 10)\n")
        table(rs[:10], H)
    open(os.path.join(out, "family_top10.md"), "w").write("\n".join(H) + "\n")
    print("\n".join(L[:20]))


if __name__ == "__main__":
    main()
