#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Calls per retail-named Rust function from an `llvm-cov export` JSON of the cov build (notes/coverage-map.md).
   python -I fncounts.py COV.json OUT.tsv
A function's `count` in the export is its entry count (LLVM source-based coverage counts the entry region exactly,
inlined copies included). Functions in rust/<crate>/src/**.rs of the instrumented game crates are kept; a function is named by the `fn NAME`
on its first line (closures and macro bodies have none and are dropped: their calls are inside the parent's).
Generic instantiations of one source function are summed. OUT.tsv: name, crate, file, line, calls."""
import json, re, sys

FN = re.compile(r"\bfn\s+([A-Za-z_][A-Za-z0-9_]*)")
TU = re.compile(r"/rust/(melee_(?:fighter|items|world|runtime|modes|chara))/src/(.+\.rs)$")


def main():
    js, out = sys.argv[1], sys.argv[2]
    data = json.load(open(js))["data"][0]
    src, acc = {}, {}
    for f in data["functions"]:
        p = f["filenames"][0].replace("\\", "/")
        m = TU.search(p)
        if not m or not f["regions"]:
            continue
        line = min(r[0] for r in f["regions"])
        if p not in src:
            try:
                src[p] = open(p, encoding="utf-8", errors="replace").read().split("\n")
            except OSError:
                src[p] = []
        L = src[p]
        nm = None
        if 0 < line <= len(L):
            mm = FN.search(L[line - 1])
            nm = mm.group(1) if mm else None
        if nm is None:
            continue
        k = (m.group(1), m.group(2), line, nm)
        acc[k] = acc.get(k, 0) + f["count"]
    with open(out, "w") as fo:
        fo.write("name\tcrate\tfile\tline\tcalls\n")
        for (c, fi, ln, nm), n in sorted(acc.items()):
            fo.write(f"{nm}\t{c}\t{fi}\t{ln}\t{n}\n")


if __name__ == "__main__":
    main()
