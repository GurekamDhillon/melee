#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 GurekamDhillon
"""The Geno stable fighter-entry id (docs/specs/geno-v5.md section 5), pure Python.

    python docs/specs/tools/geno_id.py             self-test against the value pinned by our C test
    python docs/specs/tools/geno_id.py FILE        print the id (16 hex digits) of every fighters[] entry of a geno.json

Mirrors pc/platform/geno_registry.c: gn_mix, gn_hash_bytes, gn_hash_node, and the overlay-word fold in gn_add_fighter.
Words files named by `subactions[].file` are read relative to the geno.json's folder. An entry with a `lua` block also folds
the Lua module hash (out of scope here: not computed).
"""
import json
import sys
from pathlib import Path

M64 = (1 << 64) - 1
GENO_ID_VERSION = 1
TYPE = {type(None): 0, bool: 1, int: 2, float: 2, str: 3, list: 4, dict: 5}  # JN_NULL..JN_OBJ


def mix(h, v):
    h ^= v & M64
    h = (h * 0x9E3779B97F4A7C15) & M64
    h ^= h >> 31
    h = (h * 0xBF58476D1CE4E5B9) & M64
    h ^= h >> 29
    return h


def hash_bytes(h, b):
    h = mix(h, len(b))
    for c in b:
        h = mix(h, c)
    return h


def c_g9(x):
    """printf("%.9g") for a double."""
    return "%.9g" % x


def hash_node(x, h):
    t = TYPE[type(x)]
    h = mix(h, 0x100 + t)
    if t == 1:
        h = mix(h, 1 if x else 0)
    elif t == 2:
        h = hash_bytes(h, c_g9(float(x)).encode())
    elif t == 3:
        h = hash_bytes(h, x.encode("utf-8"))
    elif t == 4:
        h = mix(h, len(x))
        for c in x:
            h = hash_node(c, h)
    elif t == 5:  # pairs in file order (a list of (key, value))
        h = mix(h, len(x))
        for k, v in x:
            h = hash_bytes(h, k.encode("utf-8"))
            h = hash_node(v, h)
    return h


def load_ordered(text):
    """json.loads keeping object member order and duplicates: objects become a PairList."""
    class PairList(list):
        pass
    return json.loads(text, object_pairs_hook=PairList)


def to_nodes(x):
    if isinstance(x, list) and type(x).__name__ == "PairList":
        return _Obj([(k, to_nodes(v)) for k, v in x])
    if isinstance(x, list):
        return [to_nodes(v) for v in x]
    return x


class _Obj(list):
    pass


TYPE[_Obj] = 5


def entry_id(entry_pairs, words_for_overlays):
    """entry_pairs: the fighters[] entry as an _Obj; words_for_overlays: list of (anim_index, [words]) in `subactions` order."""
    h = mix(0x47454E4F00000000, GENO_ID_VERSION)  # "GENO"
    h = hash_node(entry_pairs, h)
    for anim, words in words_for_overlays:
        h = mix(h, 0x4F56000000000000 | anim)
        for w in words + [0]:  # the overlay is stored with a terminating End word, and the id folds the stored words
            h = mix(h, w & 0xFFFFFFFF)
    return h or 1


def parse_word(v):
    if isinstance(v, str):
        return int(v, 0) & 0xFFFFFFFF
    return int(v) & 0xFFFFFFFF


def words_file(path):
    out = []
    for line in path.read_text().splitlines():
        line = line.split("#", 1)[0].replace(",", " ")
        out += [int(t, 0) for t in line.split()]
    return out


def ids_of(path):
    doc = to_nodes(load_ordered(Path(path).read_text(encoding="utf-8-sig")))
    root = dict(doc)
    res = []
    for e in root.get("fighters", []):
        ed = dict(e)
        ov = []
        for s in ed.get("subactions", []):
            sd = dict(s)
            if "words" in sd:
                ws = [parse_word(w) for w in sd["words"]]
            else:
                ws = words_file(Path(path).parent / sd["file"])
            ov.append((int(sd["index"]), ws))
        res.append("%016x" % entry_id(e, ov))
    return res


def selftest():
    t1 = '{"geno":1,"fighters":[{"attach":"kirby","jumps":{"max":9}}]}'
    t2 = '{ "geno": 1, "fighters": [ {\n "attach" : "kirby" , "jumps" : { "max" : 9.0 } } ] }'
    t3 = '{"geno":1,"fighters":[{"attach":"kirby","jumps":{"max":8}}]}'
    def one(t):
        return "%016x" % entry_id(dict(to_nodes(load_ordered(t)))["fighters"][0], [])
    a, b, c = one(t1), one(t2), one(t3)
    ok = a == b and a != c and a == "39ebc1bd1de2fc58"
    print("t1 %s (pinned 39ebc1bd1de2fc58)  t2 %s (same)  t3 %s (differs)  -> %s" % (a, b, c, "OK" if ok else "MISMATCH"))
    return 0 if ok else 1


if __name__ == "__main__":
    if len(sys.argv) > 1:
        for i in ids_of(sys.argv[1]):
            print(i)
    else:
        sys.exit(selftest())
