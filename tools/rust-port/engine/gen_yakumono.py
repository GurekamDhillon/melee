#!/usr/bin/env python3
"""Generate rust/melee_engine/src/yakumono_layouts.rs: the byte-swap unit of every field of every stage's `yakumono_param` record.

The stage DAT holds `yakumono_param` as big-endian GameCube bytes; the stage TUs (rust/melee_world/src/tu/gr*.rs) read it as a native
`#[repr(C)]` struct of ints, floats, s16s and u8s with the GameCube offsets. A blind 32-bit word swap is right for int / float fields but
swaps the halves of every s16 pair (and the bytes of u8 groups). This script reads the structs (`pub struct <Name>Yakumono<..>`) and writes
(size, [(offset, width)]) so the engine swaps each field by its own width.

  python tools/rust-port/engine/gen_yakumono.py            # write the table
  python tools/rust-port/engine/gen_yakumono.py --check    # fail when stale
"""
import re, sys, pathlib

ROOT = pathlib.Path(__file__).resolve().parents[3]
SRC = ROOT / "rust/melee_world/src/tu"
OUT = ROOT / "rust/melee_engine/src/yakumono_layouts.rs"
W = {"s8": 1, "s16": 2, "s32": 4, "i8": 1, "u8": 1, "i16": 2, "u16": 2, "i32": 4, "u32": 4, "f32": 4}
# pointer fields: the GameCube record holds a 4-byte relocated pointer, the native struct an 8-byte one. The kind says how the
# engine builds the native pointer: 1 = address in the raw big-endian image (colour-overlay / command streams the lb code reads
# as bytes), 2 = decoded DynamicsDesc, 3 = decoded s16 array, 4 = decoded lbColl hit description record.
PTR_KIND = {"c_void": 1, "DynamicsDesc": 2, "i16": 3, "lbColl_80008D30_arg1": 4}
STRUCTS = {}
STRUCT_RE = r"#\[repr\(C[^\]]*\)\]\s*(?:#\[derive[^\]]*\]\s*)?pub struct (\w+)\s*\{(.*?)\n\}"


def load_structs():
    STRUCTS["Vec3"] = "pub x: f32, pub y: f32, pub z: f32,"
    STRUCTS["Vec2"] = "pub x: f32, pub y: f32,"
    for p in sorted(SRC.glob("*.rs")) + [ROOT / "rust/melee_types/src/gen.rs"]:
        t = p.read_text(encoding="utf-8", errors="replace")
        for m in re.finditer(STRUCT_RE, t, re.S):
            STRUCTS.setdefault(m.group(1), m.group(2))


def parse(body, depth=0):
    """(c_size, rust_size, [(c_off, rust_off, width, ptr_kind)]); None when the struct holds anything else than scalars, scalar
    arrays, pointers of a known kind and nested structs of those. ptr_kind > 0 is a pointer (4 bytes GameCube, 8 native)."""
    coff, roff, fields, cmax, rmax = 0, 0, [], 1, 1
    for m in re.finditer(r"pub\s+(\w+)\s*:\s*([^,\n]+),", body):
        ty = m.group(2).strip()
        am = re.fullmatch(r"\[\s*(\w+)\s*;\s*([0-9a-fA-Fx +*-]+)\s*\]", ty)
        if am:
            ety, n = am.group(1), int(eval(am.group(2), {"__builtins__": {}}))
        elif re.match(r"\[", ty):
            return None
        else:
            ety, n = ty, 1
        pm = re.fullmatch(r"\*(?:mut|const)\s+(\w+)", ety)
        if pm:
            k = PTR_KIND.get(pm.group(1))
            if k is None:
                return None
            for _ in range(n):
                coff = (coff + 3) // 4 * 4
                roff = (roff + 7) // 8 * 8
                fields.append((coff, roff, 4, k))
                coff += 4
                roff += 8
            cmax, rmax = max(cmax, 4), max(rmax, 8)
        elif ety in W:
            w = W[ety]
            cmax, rmax = max(cmax, w), max(rmax, w)
            for _ in range(n):
                coff = (coff + w - 1) // w * w
                roff = (roff + w - 1) // w * w
                fields.append((coff, roff, w, 0))
                coff += w
                roff += w
        elif ety in STRUCTS and depth < 4:
            sub = parse(STRUCTS[ety], depth + 1)
            if sub is None:
                return None
            scs, srs, sf = sub
            hasp = any(k for *_, k in sf)
            sca = max([w for _, _, w, k in sf if k == 0] + [4 if hasp else 1])
            sra = max([w for _, _, w, k in sf if k == 0] + [8 if hasp else 1])
            for _ in range(n):
                coff = (coff + sca - 1) // sca * sca
                roff = (roff + sra - 1) // sra * sra
                fields.extend((coff + a, roff + b, w, k) for a, b, w, k in sf)
                coff += scs
                roff += srs
            cmax, rmax = max(cmax, sca), max(rmax, sra)
        else:
            return None
    return (coff + cmax - 1) // cmax * cmax, (roff + rmax - 1) // rmax * rmax, fields


def collect():
    load_structs()
    rows = []
    for p in sorted(SRC.glob("gr*.rs")):
        t = p.read_text(encoding="utf-8", errors="replace")
        # records named otherwise (grGb_StageAttr) are found by the cast of Ground_GetYakumonoParam()
        cast = set(re.findall(r"Ground_GetYakumonoParam\(\)\s+as\s+\*(?:mut|const)\s+(\w+)", t))
        for m in re.finditer(STRUCT_RE, t, re.S):
            if "Yakumono" not in m.group(1) and m.group(1) not in cast:
                continue
            r = parse(m.group(2))
            if r is None:
                continue
            csize, rsize, fields = r
            cf = ROOT / "src/melee/gr" / (p.stem + ".c")
            km = re.search(r"StageData\s+\w+\s*=\s*\{\s*(Gr_Kind_\w+)", cf.read_text(encoding="utf-8", errors="replace")) if cf.exists() else None
            if km and any(w != 4 or k or a != b for a, b, w, k in fields):
                rows.append((km.group(1), m.group(1), csize, rsize, fields))
    return rows


def render(rows):
    o = ["//! GENERATED by tools/rust-port/engine/gen_yakumono.py from the stage TUs' `*Yakumono*` structs. Do not edit.",
         "//! Layout of the stage `yakumono_param` records that hold s16 / u8 fields or pointers (all-word records use the plain word swap).",
         "/// (GrKind, struct, GameCube size, native size, [(GameCube offset, native offset, width, pointer kind)])",
         "pub static YAKUMONO_LAYOUTS: &[(i32, &str, u32, u32, &[(u32, u32, u8, u8)])] = &["]
    for kind, name, csize, rsize, fields in rows:
        f = ", ".join(f"({a:#x}, {b:#x}, {w}, {k})" for a, b, w, k in fields)
        o.append(f'    (melee_types::{kind} as i32, "{name}", {csize:#x}, {rsize:#x}, &[{f}]),')
    o.append("];")
    return "\n".join(o) + "\n"


if __name__ == "__main__":
    text = render(collect())
    if "--check" in sys.argv:
        sys.exit(0 if OUT.exists() and OUT.read_text(encoding="utf-8").replace("\r\n", "\n") == text else 1)
    OUT.write_text(text, encoding="utf-8", newline="\n")
    print(f"wrote {OUT}: {len(collect())} layouts")
