#!/usr/bin/env python3
"""Shadow headers for the x86-64 C oracles: the same widened slots as NATIVE_FIELD_TYPES.

gen_types.py widens a few GameCube 4-byte slots to native pointers/usize (the C trophy code keeps native
addresses in them). The oracles compile the game's own C, so their records must have the very layout the
Rust type has, or sizeof and every later offset differ. `build()` writes patched copies of the headers
that declare those records into `<root>/rust/target/native_inc/melee/...`; an oracle that puts
`-include`s the copies (force_include_flags) gets the widened records: the copies keep the originals' include
guards, so the later `#include`s of the originals, however spelled, are skipped (quote includes inside the
copies are rewritten to absolute paths). Idempotent; only these slots change.
"""
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
UP = "__UINTPTR_TYPE__"
# header -> {(record, field): replacement declaration}
PATCHES = {
    "src/melee/if/types.h": {
        ("un_804D6EF4_t", "x00"): f"{UP} x00;",
        ("un_804D6EF4_t", "x08"): f"{UP} x08;",
        ("un_804D6EF4_t", "x0C"): f"{UP} x0C;",
    },
    "src/melee/ty/types.h": {
        ("TyFiguponED4", "x0"): f"{UP} x0;",
        ("TyFiguponED4", "x4"): f"{UP} x4;",
        ("TyFiguponED4", "xC"): f"{UP} xC;",
        ("ToyED8Data", "x54"): f"{UP} x54;",
        ("TyDisplayData", "pad_144"): "HSD_Text* pad_144[4];",
        ("Toy6E68", "pad10"): "void* pad10[2];",
        ("TyFiguponData", "x10"): f"{UP} x10;",
    },
}
OUT = ROOT / "rust" / "target" / "native_inc"


def _patch(text, recs, hdr):
    for (rec, fld), decl in recs.items():
        m = re.search(r"struct %s\s*\{" % re.escape(rec), text)
        if not m:
            raise SystemExit(f"{hdr}: struct {rec} not found")
        end = text.index("\n};", m.end())
        body = text[m.end():end]
        pat = re.compile(r"^([ \t]*(?:/\*.*?\*/[ \t]*)?)[A-Za-z_][\w \t\*]*?\b%s\b(?:\[[^\]]*\])?;[ \t]*$" % re.escape(fld), re.M)
        nb, n = pat.subn(lambda mm: mm.group(1) + decl, body, count=1)
        if n != 1:
            raise SystemExit(f"{hdr}: field {rec}.{fld} not found")
        text = text[:m.end()] + nb + text[end:]
    return text


def build():
    """Write the shadow headers; return the include directory to pass as -I (first)."""
    for hdr, recs in PATCHES.items():
        src = ROOT / hdr
        text = src.read_text(encoding="utf-8")
        text = _patch(text, recs, hdr)
        text = re.sub(r'#include "([^"]+)"', lambda m: '#include "%s"' % (src.parent / m.group(1)).as_posix()
                      if (src.parent / m.group(1)).exists() else m.group(0), text)
        dst = OUT / hdr.removeprefix("src/")
        dst.parent.mkdir(parents=True, exist_ok=True)
        if not dst.exists() or dst.read_text(encoding="utf-8") != text:
            dst.write_text(text, encoding="utf-8", newline="\n")
    return OUT


def force_include_flags():
    """clang flags that preload the shadow copies (their include guards then skip the originals, however
    the originals are spelled in the including file)."""
    build()
    return ["-include", str(OUT / "melee/if/types.h"), "-include", str(OUT / "melee/ty/types.h")]


if __name__ == "__main__":
    print(build())
