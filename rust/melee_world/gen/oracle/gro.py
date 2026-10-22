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

if __name__ == "__main__":
    G.main()
