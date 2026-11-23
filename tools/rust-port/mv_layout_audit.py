"""Audit: where a C union's views alias differently on the native layout than on the GameCube (docs/rust-port/notes/fn-identity.md
has the sibling problem for code addresses; this one is data).

A union view that holds a pointer is 8 bytes natively, 4 on the GameCube, so every leaf after it moves: two leaves of different
views that are one GameCube word can be different native bytes (pk.specialhi.x4 / pk.speciallw.x4), and two that are different
GameCube words can share native bytes (pk.speciallw.x4 / pk.specialhi.x8). C that writes one view and reads the other is then
wrong on a native build, and so is a port of it unless it keeps the views in step (the "same GC word" dual writes, which in turn
clobber the native neighbour).

For every union reachable from the game's types (ft, it, gr, lb, gm types.h) this lists the drifting leaf pairs, then scans the
Rust port for accesses through the field that holds the union (`.mv.pk.specialhi.x4`) and reports, per pair, whether the port
writes one side and reads the other (a cross-view flow, broken natively) and where the port writes both (a dual write) together
with the native bytes such a write clobbers. Read only: it changes no file.
    python tools/rust-port/mv_layout_audit.py [--libclang PATH] [--tsv OUT]
Needs libclang (LIBCLANG, else the LLVM install next to CLANG as gen/chref.py) and GALE_INCLUDE like chref.py.
"""
import os
import re
import sys
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "rust/melee_chara/gen"))
import chref  # noqa: E402  (its COMMON flags: the oracle's native build)

HEADERS = ["ft/types.h", "it/types.h", "gr/types.h", "lb/types.h", "gm/types.h"]
# One entity is one kind, so two views of these never hold data for the same object (a fighter is one character, an
# item one kind, a ground one stage piece): only a pair with a SHARED view can carry a value across. mv's shared view
# is `co`, the common motion states'; the per-kind unions have none. Every other union is the per-state views of one
# entity (ftCommon_MotionVars, ftPikachu_MotionVars...): all of its pairs count. `_` (the pad arrays) never does.
PER_KIND = {"union Fighter_MotionVars": {"co"}, "union Fighter_FighterVars": set(), "union Item_ItemVars": set(),
            "union GroundVars": set()}


def relevant(u, a, b):
    if a[0] == "_" or b[0] == "_":
        return False
    if u in PER_KIND:
        return bool({a[0], b[0]} & PER_KIND[u])
    return True


def parse(target):
    import clang.cindex as ci
    flags, skip = [], False
    for f in chref.COMMON:
        if skip or f == "-include":
            skip = not skip
            continue
        if not f.startswith("--target="):
            flags.append(f)
    tgt = target or next(f for f in chref.COMMON if f.startswith("--target=")).split("=", 1)[1]
    src = "".join(f'#include "{h}"\n' for h in HEADERS)
    return ci.Index.create().parse("audit.c", args=["-fsyntax-only", f"--target={tgt}", *flags], unsaved_files=[("audit.c", src)])


def union_views(tu):
    """{union name: {view: {leaf path: (byte offset, size, kind)}}} and [(holder struct, field, union name)]."""
    import clang.cindex as ci
    unions, holders = {}, []

    def leaves(t, path, off, out):
        t = t.get_canonical()
        if t.kind == ci.TypeKind.RECORD:
            for f in t.get_fields():
                fo = t.get_offset(f.spelling)
                if f.is_bitfield():
                    out[f"{path}.{f.spelling}"] = (off + fo // 8, 0, "bf")
                    continue
                leaves(f.type, f"{path}.{f.spelling}", off + fo // 8, out)
        elif t.kind == ci.TypeKind.CONSTANTARRAY:
            es = t.element_type.get_size()
            for i in range(t.element_count):
                leaves(t.element_type, f"{path}[{i}]", off + i * es, out)
        else:
            out[path] = (off, t.get_size(), "p" if t.kind == ci.TypeKind.POINTER else "s")

    seen = set()
    for c in tu.cursor.walk_preorder():
        if c.kind in (ci.CursorKind.STRUCT_DECL, ci.CursorKind.UNION_DECL) and c.is_definition() and c.location.file \
                and "src/melee" in str(c.location.file).replace("\\", "/"):
            for f in c.type.get_fields():
                ft = f.type.get_canonical()
                while ft.kind == ci.TypeKind.CONSTANTARRAY:
                    ft = ft.element_type.get_canonical()
                d = ft.get_declaration()
                if d.kind == ci.CursorKind.UNION_DECL and d.is_definition():
                    holders.append((c.type.spelling, f.spelling, d.type.get_canonical().spelling))
                    if d.type.get_canonical().spelling not in seen:
                        seen.add(d.type.get_canonical().spelling)
                        views = {}
                        for v in d.type.get_fields():
                            out = {}
                            leaves(v.type, "", 0, out)
                            views[v.spelling] = out
                        unions[d.type.get_canonical().spelling] = views
    return unions, holders


def drift(nat, gc):
    """Pairs of leaves in different views whose overlap differs between the layouts:
    'gc-only' (one GameCube word, apart natively) and 'native-only' (apart on the GameCube, overlapping natively)."""
    out = []
    flat = [(v, p, nat[v][p], gc[v][p]) for v in nat for p in nat[v] if p in gc.get(v, {})]
    flat.sort(key=lambda x: x[3][0])

    def ov(a, b):
        return a[1] and b[1] and a[0] < b[0] + b[1] and b[0] < a[0] + a[1]
    by_native = sorted(flat, key=lambda x: x[2][0])
    pairs = set()
    for lst, idx in ((flat, 3), (by_native, 2)):
        for i, a in enumerate(lst):
            for b in lst[i + 1:]:
                if b[idx][0] >= a[idx][0] + max(a[idx][1], 1):
                    break
                if a[0] != b[0]:
                    pairs.add((a[:2], b[:2]) if a[:2] < b[:2] else (b[:2], a[:2]))
    lut = {(v, p): (n, g) for v, p, n, g in flat}
    for a, b in sorted(pairs):
        (na, ga), (nb, gb) = lut[a], lut[b]
        g, n = ov(ga, gb), ov(na, nb)
        if g == n:
            continue
        if "p" in (ga[2], gb[2]) or "bf" in (ga[2], gb[2]):
            continue  # a pointer's bytes or a bitfield: not a value a view could share
        out.append(("gc-only" if g else "native-only", a, b, ga, gb, na, nb))
    return out


def port_accesses(holders):
    """{(holder field, view, leaf path): {"w": set(fn), "r": set(fn)}} over rust/*/src."""
    acc = defaultdict(lambda: {"w": set(), "r": set()})
    fields = {f for _, f, _ in holders if re.fullmatch(r"\w+", f)}  # anonymous members have no Rust path
    # a lookahead, so `.mv.pk.specialhi.x4` is seen as mv's pk view and as pk's specialhi view
    pat = re.compile(r"(?=\.(%s)\.(\w+)((?:\.\w+|\[\d+\])+)(\s*(?:[-+*/|&^]?=(?!=)))?)" % "|".join(map(re.escape, sorted(fields))))
    for f in sorted((ROOT / "rust").glob("*/src/**/*.rs")):
        cur = "?"
        for line in f.read_text(encoding="utf-8", errors="replace").splitlines():
            m = re.search(r"\bfn (\w+)", line)
            if m:
                cur = f"{f.relative_to(ROOT / 'rust').parts[0]}::{m.group(1)}"
            for m in pat.finditer(line):
                field, view, path, asg = m.groups()
                path = re.sub(r"\(\)$", "", path)
                acc[(field, view, path)]["w" if asg else "r"].add(cur)
    return acc


def main():
    import clang.cindex as ci
    lib = os.environ.get("LIBCLANG") or (sys.argv[sys.argv.index("--libclang") + 1] if "--libclang" in sys.argv else
                                         str(Path(chref.CLANG).parent / "libclang.dll"))
    ci.Config.set_library_file(lib)
    nat_u, holders = union_views(parse(None))
    gc_u, _ = union_views(parse("powerpc-unknown-eabi"))
    acc = port_accesses(holders)
    rows = []
    for u in sorted(nat_u):
        if u not in gc_u:
            continue
        for kind, a, b, ga, gb, na, nb in drift(nat_u[u], gc_u[u]):
            if not relevant(u, a, b):
                continue
            for field in sorted({f for _, f, uu in holders if uu == u}):  # one row per holding field name
                xa, xb = acc.get((field, a[0], a[1]), {"w": set(), "r": set()}), acc.get((field, b[0], b[1]), {"w": set(), "r": set()})
                cross = sorted({f"{w} -> {r}" for w in xa["w"] for r in xb["r"]} | {f"{w} -> {r}" for w in xb["w"] for r in xa["r"]})
                dual = sorted(xa["w"] & xb["w"])
                rows.append((u, field, kind, f"{a[0]}{a[1]}", f"{b[0]}{b[1]}", ga[0], gb[0], na[0], nb[0],
                             len(xa["w"]) + len(xa["r"]) + len(xb["w"]) + len(xb["r"]), cross, dual))
    out = sys.argv[sys.argv.index("--tsv") + 1] if "--tsv" in sys.argv else None
    lines = ["union\tfield\tkind\tleaf_a\tleaf_b\tgc_a\tgc_b\tnative_a\tnative_b\tport_accesses\tcross_view_flows\tdual_writes"]
    for r in rows:
        lines.append("\t".join(map(str, r[:10])) + "\t" + "; ".join(r[10]) + "\t" + "; ".join(r[11]))
    if out:
        Path(out).write_text("\n".join(lines) + "\n", encoding="utf-8")
    by_u = defaultdict(lambda: [0, 0, 0, 0])
    for r in rows:
        s = by_u[(r[0], r[1])]
        s[0] += 1
        s[1] += r[9] > 0
        s[2] += bool(r[10])
        s[3] += bool(r[11])
    print("union / field: drifting pairs, pairs the port touches, with a cross-view flow, with a dual write")
    for (u, f), s in sorted(by_u.items(), key=lambda kv: -kv[1][0]):
        print(f"  {u} .{f}: {s[0]}, {s[1]}, {s[2]}, {s[3]}")
    for r in rows:
        if r[10] or r[11]:
            print(f"{r[0]} .{r[1]} {r[2]}: {r[3]} (gc {r[5]}, native {r[7]}) / {r[4]} (gc {r[6]}, native {r[8]})")
            for c in r[10][:6]:
                print(f"    cross-view: {c}")
            for d in r[11][:6]:
                print(f"    dual write: {d}")


if __name__ == "__main__":
    main()
