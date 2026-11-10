#!/usr/bin/env python3
"""melee_items front end to melee_world's gen_tu.py (the per-TU C-oracle generator).

  python rust/melee_items/gen/mtu_it2/gen_it.py <stem> [...]      (from the worktree root)

Same generator, retargeted: the specs come from specs_it.py, the generated stubs / fill / hash go to
gen/mtu_it2/tu/<stem>_gen.h and rust/melee_items/tests/mtu/<stem>_gen.rs, and the ext block is
spliced into rust/melee_items/src/tu/<stem>.rs. One change to the walker: a `UnkFlagStruct`
(`union { u8 byte; struct { u8 b0:1 ... b7:1; }; }`) is filled and hashed bit by bit BY VALUE, because
melee_types keeps bitfields in GameCube bit order (MSB first) and native C is LSB first: the same raw
byte would be two different flag words.
"""
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
sys.path.insert(0, str(ROOT / "rust/melee_world/gen/mtu"))
sys.path.insert(0, str(HERE))
import gen_tu  # noqa: E402  (chdirs to ROOT)

gen_tu.HERE = HERE
_orig_one = gen_tu.Walk.one


def _one(self, t, f, cexpr, rexpr, cf, ch, rf, rh, depth):
    c = gen_tu.canon(t)
    if c.kind == gen_tu.T.RECORD and c.get_declaration().spelling == "UnkFlagStruct":
        for b in range(8):
            cf.append(f"{cexpr}.x0.b{b} = rnd() & 1;")
            ch.append(f"HM((u32) {cexpr}.x0.b{b});")
            rf.append(f"unsafe {{ {rexpr}.anon_0.set_b{b}((rnd() & 1) as u8); }}")
            rh.append(f"unsafe {{ hm({rexpr}.anon_0.b{b}() as u32); }}")
        return
    return _orig_one(self, t, f, cexpr, rexpr, cf, ch, rf, rh, depth)


gen_tu.Walk.one = _one


def build(stem):
    from specs_it import SPECS
    return gen_tu.build(stem, SPECS[stem], False)


if __name__ == "__main__":
    for s in sys.argv[1:]:
        build(s)
