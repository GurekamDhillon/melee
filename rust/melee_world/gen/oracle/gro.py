#!/usr/bin/env python3
"""gr_oracle.py with one fix: a stub whose C return type is a pointer to a small record
(oracle.kind_of -> 'ps:<tag>:<size>') got no return statement from oracle.stub_c (it fell off the
end, so the call log lacked the returned pointer and the Rust hook, which logs it, differed).

  python rust/melee_world/gen/oracle/gro.py <stem> [...] [-v FUNC]
"""
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import gr_oracle as G  # noqa: E402

O = G.O
_stub_c = O.stub_c


def stub_c(name, fn):
    txt, kinds, rk, ret = _stub_c(name, fn)
    if rk.startswith("ps:") and ret.get_canonical().kind == O.T.POINTER:
        body = txt.rsplit("}", 1)[0]
        body += f"    {{ {ret.spelling} r = ({ret.spelling}) gen_p(); lg_retp((const void*) r); lg_c(';'); return r; }}\n}}"
        txt = body
    return txt, kinds, rk, ret


O.stub_c = stub_c

# An `Mtx` parameter (float[3][4], decays to float(*)[4]) is an ordinary pointer to the stubs.
_kind_of = O.kind_of


def kind_of(t):
    if t.get_canonical().kind == O.T.CONSTANTARRAY:
        return "p"
    return _kind_of(t)


O.kind_of = kind_of

# spec key `patch=[(old, new), ...]`: text edits applied to the game's .c before the oracle compiles
# it (each `old` must occur exactly once). Used where the C reaches a field through a GameCube byte
# offset (`(u8*) p + 0xC4`, `char pad[0xDD8]`) that does not exist in the native layout: the patch
# names the field the offset means on the GC.
_load_spec = G.load_spec
PATCHED = {}


def load_spec(stem):
    spec = _load_spec(stem)
    if spec.get("patch"):
        src = (G.ROOT / spec["src"]).read_text(encoding="utf-8")
        for old, new in spec["patch"]:
            if src.count(old) != 1:
                raise SystemExit(f"gro: patch for {stem}: {old[:60]!r} occurs {src.count(old)} times")
            src = src.replace(old, new)
        O.BUILD.mkdir(parents=True, exist_ok=True)
        dst = O.BUILD / f"gro_{stem}_patched.c"
        dst.write_text(src, encoding="utf-8")
        O.COMMON.extend(["-I", str((G.ROOT / spec["src"]).parent)])
        PATCHED[stem] = (dst.as_posix(), spec["src"])
        spec = dict(spec, src=dst.as_posix())
    return spec


G.load_spec = load_spec

# A `u` set naming "nobf" compares the union members' plain fields but not their bitfields. The
# bytes of a union bitfield may hold another member's data (a float written by a sibling callback);
# the C packs bitfields LSB-first and the Rust records MSB-first like the PPC, so the two disagree on
# such bytes although both are right.
_bf_paths = G.bf_paths


def bf_paths(pr, tname, rs_tname=None, keep=None):
    out = _bf_paths(pr, tname, rs_tname, keep)
    u = keep.__defaults__[0] if keep is not None and keep.__defaults__ else ()
    if "nobf" in u:
        out = [e for e in out if not e[0].startswith("u.")]
    return out


G.bf_paths = bf_paths

def main():
    G.main()
    for stem, (dst, orig) in PATCHED.items():
        p = G.CRATE / f"tests/gro_{stem}.rs"
        t = p.read_text(encoding="utf-8")
        p.write_text(t.replace(dst, orig + " (GC-offset overlays patched: spec_" + stem + ".py)"), encoding="utf-8", newline="\n")


if __name__ == "__main__":
    main()
