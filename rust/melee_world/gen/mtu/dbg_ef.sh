#!/bin/bash
# first differing trace / state words of one ef oracle case:  bash rust/melee_world/gen/mtu/dbg_ef.sh <op>:<case>
set -e
export MTU_DBG=$1
python rust/melee_world/gen/mtu/mk_ef.py run >/dev/null
( cd rust && cargo test -p melee_world --test tu_ef -- --nocapture 2>/dev/null | grep -E "^(T|H|B|N) " > target/mtu/dbg_ef_rs.txt || true )
python - <<'PY'
from pathlib import Path
c = [l.split()[:2] for l in Path("rust/target/mtu/dbg_ef.txt").read_text().splitlines() if l[:2] in ("T ", "H ", "B ", "N ")]
r = [l.split()[:2] for l in Path("rust/target/mtu/dbg_ef_rs.txt").read_text().splitlines()]
n = 0
for i in range(max(len(c), len(r))):
    a = c[i] if i < len(c) else None
    b = r[i] if i < len(r) else None
    if a != b:
        print(i, "C", a, "RS", b)
        n += 1
        if n > 8:
            break
print("lens", len(c), len(r))
PY
