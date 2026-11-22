#!/usr/bin/env python3
"""Rollback-statics lint: a heap-owning static the rollback snapshot can rewind is a double free waiting to happen.

A rollback load restores every writable static of the executable (and the game arena). A static whose value owns
system-heap memory (Vec, String, Box, HashMap, ... behind a Mutex, RwLock or `static mut`) and is changed while a
rollback session runs gets an old header back, pointing at a buffer since freed: `free(): double free`
(retail_flow::transition_time, 2026-11-22). Such state belongs in `melee_engine::arena::SysStatic`, a thread_local,
or the game arena by design.

This lint lists every non-thread_local `static` in the host crates whose type names a heap-owning type and is not a
`SysStatic`, and fails on any that is not in rbstatics_allow.tsv (path, name, reason: why a rewind cannot hurt it).
Generated game translation units (`/tu/`) are game state and skipped.

usage: rbstatics_lint.py [RUST_DIR]   (default: ../../rust from this file)
"""
import os
import re
import sys

CRATES = ["melee/src", "melee_engine/src", "melee_core/src", "melee_platform/src", "melee_net/src"]
HEAP = re.compile(r"\b(String|Vec<|Box<|HashMap|BTreeMap|VecDeque|HashSet|BTreeSet|PathBuf)")
STATIC = re.compile(r"^\s*(?:pub(?:\([a-z]+\))?\s+)?static\s+(mut\s+)?([A-Z_][A-Z_0-9]*)\s*:\s*(.*)$")

here = os.path.dirname(os.path.abspath(__file__))
root = sys.argv[1] if len(sys.argv) > 1 else os.path.normpath(os.path.join(here, "..", "..", "rust"))
allow = set()
with open(os.path.join(here, "rbstatics_allow.tsv")) as f:
    for line in f:
        if line.strip() and not line.startswith("#"):
            p, name = line.split("\t")[:2]
            allow.add((p, name))

found, bad = [], []
for crate in CRATES:
    for dp, _, files in os.walk(os.path.join(root, crate)):
        if "/tu" in dp.replace(os.sep, "/"):
            continue
        for fn in files:
            if not fn.endswith(".rs"):
                continue
            path = os.path.join(dp, fn)
            rel = os.path.relpath(path, root).replace(os.sep, "/")
            lines = open(path, encoding="utf-8", errors="replace").read().split("\n")
            tl = 0  # inside a thread_local! { ... } block: TLS is not in a snapshot
            for i, l in enumerate(lines):
                if "thread_local!" in l:
                    tl = 1
                    continue
                if tl:
                    tl += l.count("{") - l.count("}")
                    if tl <= 0:
                        tl = 0
                    continue
                m = STATIC.match(l)
                if not m or "SysStatic" in l:
                    continue
                ty = m.group(3)
                if not HEAP.search(ty):
                    continue
                found.append((rel, m.group(2)))
                if (rel, m.group(2)) not in allow:
                    bad.append(f"{rel}:{i + 1}: static {m.group(2)}: {ty.strip()[:90]}")

print(f"rbstatics: {len(found)} heap-owning statics outside SysStatic/thread_local, {len(found) - len(bad)} reviewed")
for b in bad:
    print("UNREVIEWED " + b)
if bad:
    print("Put each in melee_engine::arena::SysStatic (or a thread_local), or add it to tools/rust-port/rbstatics_allow.tsv "
          "with the reason a rollback load cannot rewind it into a double free.")
sys.exit(1 if bad else 0)
