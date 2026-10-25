#!/usr/bin/env python3
"""Turn a MELEE_CALLTRACE table (pc/platform/gw_calltrace.c) into the call-trace CSV.

  python tools/rust-port/trace/calltrace.py --trace calltrace.raw --map melee-pc.map \
      --tus <workspace>/_build/masstest/files.txt --out docs/rust-port/trace/pad_falcon_fox_fd.csv \
      [--last-frame 1800] [--summary]

Columns: c_function, tu, first_frame, call_count, rust_status, rust_crate
  first_frame  match frame whose logic first called it (same numbering as the golden CSV's `frame`);
               -1 = before the match (boot, menus, loading).
  rust_status  ported+verified    a real-bodied Rust fn of that name exists and a test under
                                  rust/*/tests references it (the *_vs_c / *_ref oracles)
               ported-unverified  a real-bodied Rust fn exists, no test references it
               stub               only a panicking stub / hook declaration exists (docs/rust-port/missing.md)
               missing            nothing of that name in any crate
  rust_crate   crate holding the port; for stub/missing the crate the TU is planned for
               (tools/rust-port/transpile.sh crate_for)
No disc data is involved: function names, source paths and counters only.
"""
import argparse
import csv
import re
import sys
from collections import Counter
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
CRATE_ORDER = ["melee_core", "melee_types", "melee_runtime", "melee_world", "melee_items",
               "melee_fighter", "melee_chara", "melee_modes", "melee_audio", "melee_net",
               "melee_gx", "melee_platform", "melee"]


def crate_for(tu):
    p = tu.replace("\\", "/")
    if p.startswith("src/melee/ft/chara/ftCommon/"):
        return "melee_fighter"
    if p.startswith("src/melee/ft/chara/"):
        return "melee_chara"
    if p.startswith(("src/melee/ft/", "src/melee/pl/")):
        return "melee_fighter"
    if p.startswith(("src/melee/mp/", "src/melee/cm/", "src/melee/ef/", "src/melee/gr/", "src/melee/if/")):
        return "melee_world"
    if p.startswith(("src/melee/it/", "src/melee/ty/", "src/melee/vi/")):
        return "melee_items"
    if p.startswith(("src/melee/gm/", "src/melee/mn/", "pc/geno/", "pc/tests/")) or p == "pc/gameworld/script_game.c":
        return "melee_modes"
    if p.startswith(("src/sysdolphin/", "src/melee/lb/", "src/melee/db/", "src/melee/sfx/", "src/MSL/", "libs/",
                     "extern/")) or p in ("pc/gameworld/gekko_fp.c", "pc/gameworld/mtx_pc.c"):
        return "melee_runtime"
    return "melee_runtime"


def strip_rust(src):
    """Blank comments, strings and char literals (keeps length and newlines)."""
    out = []
    i, n = 0, len(src)
    while i < n:
        c = src[i]
        if src.startswith("//", i):
            j = src.find("\n", i)
            j = n if j < 0 else j
            out.append(" " * (j - i)); i = j
        elif src.startswith("/*", i):
            depth, j = 1, i + 2
            while j < n and depth:
                if src.startswith("/*", j): depth += 1; j += 2
                elif src.startswith("*/", j): depth -= 1; j += 2
                else: j += 1
            out.append(re.sub(r"[^\n]", " ", src[i:j])); i = j
        elif c == '"':
            j = i + 1
            while j < n and src[j] != '"':
                j += 2 if src[j] == "\\" else 1
            j = min(j + 1, n)
            out.append('"' + re.sub(r"[^\n]", " ", src[i + 1:j - 1]) + '"'); i = j
        elif c == "r" and re.match(r'r#*"', src[i:i + 8]):
            m = re.match(r'r(#*)"', src[i:])
            end = src.find('"' + m.group(1), i + m.end())
            end = n if end < 0 else end + 1 + len(m.group(1))
            out.append(re.sub(r"[^\n]", " ", src[i:end])); i = end
        else:
            out.append(c); i += 1
    return "".join(out)


FN_RE = re.compile(r"\bfn\s+([A-Za-z_]\w*)")


def index_rust(root):
    """name.lower() -> list of (crate, path, real_body)"""
    idx = {}
    for crate_dir in sorted((root / "rust").iterdir()):
        src = crate_dir / "src"
        if not src.is_dir():
            continue
        for path in src.rglob("*.rs"):
            parts = path.relative_to(src).parts
            if parts[0] in ("raw", "c2r"):
                continue
            text = strip_rust(path.read_text(encoding="utf-8", errors="replace"))
            for m in FN_RE.finditer(text):
                j, depth = m.end(), 0
                # skip generics and parameters to the body or the terminating ';'
                while j < len(text):
                    ch = text[j]
                    if ch in "([<":
                        depth += 1
                    elif ch in ")]>" and not (ch == ">" and text[j - 1] == "-"):
                        depth -= 1
                    elif depth <= 0 and ch in "{;":
                        break
                    j += 1
                if j >= len(text) or text[j] == ";":
                    continue
                k, d = j, 0
                while k < len(text):
                    if text[k] == "{": d += 1
                    elif text[k] == "}":
                        d -= 1
                        if d == 0: break
                    k += 1
                body = text[j + 1:k]
                panicky = re.search(r"\b(todo|unimplemented|panic)!", body) is not None
                real = not (panicky and len(body.strip()) < 220)
                idx.setdefault(m.group(1).lower(), []).append((crate_dir.name, path, real, len(body)))
    return idx


def tested_names(root):
    words = set()
    for t in (root / "rust").glob("*/tests/**/*.rs"):
        # generated recording stubs / field fill+hash only name the callees and fields of an oracle; they verify nothing
        if any(part.endswith(("_stubs", "_state")) for part in t.parts):
            continue
        words.update(w.lower() for w in re.findall(r"[A-Za-z_]\w*", t.read_text(encoding="utf-8", errors="replace")))
    return words


def missing_names(root):
    names = {}
    cur = None
    for line in (root / "docs/rust-port/missing.md").read_text(encoding="utf-8").splitlines():
        m = re.match(r"^## (melee\w*)", line)
        if m:
            cur = m.group(1)
        m = re.match(r"^- `([^`]+)` \[", line)
        if m and cur:
            names.setdefault(m.group(1).lower(), cur)
    return names


def read_map(path):
    text = Path(path).read_text(errors="replace")
    pref = int(re.search(r"Preferred load address is ([0-9a-fA-F]+)", text).group(1), 16)
    syms = {}
    rx = re.compile(r"^\s*[0-9a-f]{4}:[0-9a-f]{8}\s+(\S+)\s+([0-9a-f]{8,16})\s+f(?:\s+i)?\s+(\S+)\s*$")
    for line in text.splitlines():
        m = rx.match(line)
        if m:
            syms[int(m.group(2), 16) - pref] = (m.group(1), m.group(3))
    return syms


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--trace", required=True)
    ap.add_argument("--map", required=True)
    ap.add_argument("--tus", required=True, help="game TU list (files.txt)")
    ap.add_argument("--out", required=True)
    ap.add_argument("--root", default=str(REPO))
    ap.add_argument("--last-frame", type=int, default=None, help="drop functions first called after this frame")
    ap.add_argument("--summary", action="store_true")
    a = ap.parse_args()
    root = Path(a.root)

    tus = [l.strip() for l in Path(a.tus).read_text().splitlines() if l.strip()]
    obj_to_tu = {t.replace("/", "_") + ".obj": t for t in tus}
    syms = read_map(a.map)

    k0 = f0 = None
    rows = {}
    for line in Path(a.trace).read_text().splitlines():
        if line.startswith("#calltrace"):
            kv = dict(p.split("=") for p in line.split(",")[2:])
            k0, f0 = int(kv["k0"]), int(kv["f0"])
            continue
        rva, tick, cnt = line.split(",")
        rva, tick, cnt = int(rva, 16), int(tick), int(cnt)
        name, obj = syms.get(rva, (None, None))
        tu = obj_to_tu.get(obj)
        if name is None or tu is None:
            continue  # not a game TU (native shims have no hook anyway)
        frame = -1 if (k0 is None or k0 < 0 or tick < k0) else tick - k0 + f0
        fn = re.sub(r"^_?gw_", "", name) if name.startswith(("_gw_", "gw_")) else name.lstrip("_") or name
        key = (fn, tu)
        if key in rows:  # same symbol reported twice (alias): keep earliest
            r = rows[key]
            r[0] = min(r[0], frame) if frame >= 0 and r[0] >= 0 else min(r[0], frame)
            r[1] += cnt
        else:
            rows[key] = [frame, cnt]

    idx = index_rust(root)
    tested = tested_names(root)
    miss = missing_names(root)

    out = []
    for (fn, tu), (frame, cnt) in rows.items():
        if a.last_frame is not None and frame > a.last_frame:
            continue
        key = fn.lower()
        real = [d for d in idx.get(key, []) if d[2]]
        if real:
            real.sort(key=lambda d: (-d[3], CRATE_ORDER.index(d[0]) if d[0] in CRATE_ORDER else 99))
            crate = real[0][0]
            status = "ported+verified" if key in tested else "ported-unverified"
        elif key in idx or key in miss:
            status, crate = "stub", crate_for(tu)
        else:
            status, crate = "missing", crate_for(tu)
        out.append([fn, tu, frame, cnt, status, crate])
    out.sort(key=lambda r: (r[2], r[1], r[0]))
    Path(a.out).parent.mkdir(parents=True, exist_ok=True)
    with open(a.out, "w", newline="") as f:
        w = csv.writer(f, lineterminator="\n")
        w.writerow(["c_function", "tu", "first_frame", "call_count", "rust_status", "rust_crate"])
        w.writerows(out)
    print(f"{len(out)} functions -> {a.out}")
    if a.summary:
        print("by status:", dict(Counter(r[4] for r in out)))
        print("by crate:", dict(Counter(r[5] for r in out)))
        print("by crate+status:")
        for k, v in sorted(Counter((r[5], r[4]) for r in out).items()):
            print("  ", k, v)
        print("earliest unported:")
        for r in [r for r in out if r[4] in ("stub", "missing")][:40]:
            print("  ", r)


if __name__ == "__main__":
    sys.exit(main())
