"""Parse rust/melee_types/src/gen.rs into the tables the layout checker needs.

gen.rs is mechanical (gen_types.py), so its shapes are regular:
  /// C `NAME` (FILE:LINE); GameCube size 0xS, align A.
  #[repr(C, ...)] ... pub struct|union RUST { pub f: T, ... }
  impl ... { fn gc_decode...(d, at) { v.f = <T as GcDecode>::gc_decode(d, at + 0xOFF); ... } }
  impl GcLayout for RUST { GC_NAME, GC_SIZE, GC_ALIGN, GC_FIELDS = &[GcField::new|bitfield(...)] }
"""
import re
from pathlib import Path

DOC = re.compile(r"^/// C `(.*)` \((.*?)\); GameCube size 0x([0-9A-F]+), align (\d+)\.$")
DEF = re.compile(r"^pub (struct|union) (\w+) \{$")
FIELD = re.compile(r"^    pub (r#)?(\w+): (.*),$")
DECODE = re.compile(r"v\.((?:r#)?\w+) = .*?at \+ 0x([0-9A-F]+)\b")
LAYOUT = re.compile(r"^impl GcLayout for (\w+) \{$")
GCF = re.compile(r'GcField::(new|bitfield)\("([^"]*)", 0x([0-9A-F]+), (\d+)(?:, (\d+), (\d+))?, "(.*)"\),$')
ALIAS = re.compile(r"^pub type (\w+) = (.*);")
DECODE_HEAD = re.compile(r"^impl (?:GcDecode for )?(\w+) \{")


def parse(path):
    lines = Path(path).read_text(encoding="utf-8").split("\n")
    recs, aliases = {}, {}
    doc = None
    cur = None          # record whose struct body is being read
    dec = None          # record whose decode body is being read
    lay = None
    for ln in lines:
        m = DOC.match(ln)
        if m:
            doc = m.groups()
            continue
        m = DEF.match(ln)
        if m:
            kind, rn = m.groups()
            cur = recs.setdefault(rn, {"rust": rn, "kind": kind, "fields": [], "off": {}, "gc": None})
            if doc:
                cur["c_name"], cur["c_loc"], cur["doc_size"], cur["doc_align"] = doc[0], doc[1], int(doc[2], 16), int(doc[3])
            doc = None
            continue
        if cur is not None:
            if ln == "}":
                cur = None
                continue
            m = FIELD.match(ln)
            if m:
                cur["fields"].append((m.group(2), m.group(3).strip()))
            continue
        m = ALIAS.match(ln)
        if m:
            aliases[m.group(1)] = m.group(2).strip()
            continue
        m = DECODE_HEAD.match(ln)
        if m and m.group(1) in recs and ("gc_decode" in ln or ln.startswith("impl GcDecode") or "#[allow" in ln):
            dec = recs[m.group(1)]
            continue
        if ln.startswith("impl ") and dec is not None and not ln.startswith("impl GcLayout"):
            dec = None
        if dec is not None:
            for f, o in DECODE.findall(ln):
                f = f[2:] if f.startswith("r#") else f
                dec["off"].setdefault(f, int(o, 16))
            if ln.startswith("}"):
                dec = None
        m = LAYOUT.match(ln)
        if m:
            lay = recs.setdefault(m.group(1), {"rust": m.group(1), "kind": "?", "fields": [], "off": {}, "gc": None})
            lay["gc"] = {"fields": []}
            continue
        if lay is not None:
            s = ln.strip()
            if s.startswith("const GC_NAME"):
                lay["gc"]["name"] = s.split('"', 1)[1].rsplit('"', 1)[0]
            elif s.startswith("const GC_SIZE"):
                lay["gc"]["size"] = int(s.split("=")[1].strip(" ;"), 16)
            elif s.startswith("const GC_ALIGN"):
                lay["gc"]["align"] = int(s.split("=")[1].strip(" ;"))
            else:
                m = GCF.search(s)
                if m:
                    k, n, o, sz, b, w, ct = m.groups()
                    lay["gc"]["fields"].append({"name": n, "off": int(o, 16), "size": int(sz),
                                                "bits": (int(b), int(w)) if k == "bitfield" else None, "ctype": ct})
            if ln == "}":
                lay = None
    return recs, aliases


PRIMS = {"u8": (1, "u"), "i8": (1, "i"), "u16": (2, "u"), "i16": (2, "i"), "u32": (4, "u"), "i32": (4, "i"),
         "u64": (8, "u"), "i64": (8, "i"), "f32": (4, "f"), "f64": (8, "f"), "bool": (1, "b"),
         "usize": (4, "u"), "isize": (4, "i")}


def resolve(t, aliases, depth=0):
    """Follow `pub type` aliases (enums are int aliases)."""
    while t in aliases and depth < 20:
        t = aliases[t]
        depth += 1
    return t


def split_array(t):
    """`[T; N]` -> (T, N) else None. Handles nesting."""
    if not (t.startswith("[") and t.endswith("]")):
        return None
    body = t[1:-1]
    depth = 0
    for i in range(len(body) - 1, -1, -1):
        c = body[i]
        if c in "])>":
            depth += 1
        elif c in "[(<":
            depth -= 1
        elif c == ";" and depth == 0:
            return body[:i].strip(), body[i + 1:].strip()
    return None
