#!/usr/bin/env bash
# First differing state word of one case:  bash rust/melee_items/gen/kinds/dbg.sh <stem> <op>:<case>
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../../../.." && pwd)"
cd "$root"
stem=$1; oc=$2
MTU_DBG=$oc python rust/melee_items/gen/kinds/run_tu.py "$stem" | tail -1
(cd rust && MTU_DBG=$oc cargo test -p melee_items --test kinds_$stem -- --nocapture 2>&1 | grep -E "^(H|B|T) " > target/mtu/dbg_rs.txt) || true
python - "$stem" <<'PY'
import sys
stem=sys.argv[1]
a=open(f'rust/target/mtu/dbg_{stem}.txt').read().split('\n')
b=open('rust/target/mtu/dbg_rs.txt').read().split('\n')
cur=None; start=0
for i,(x,y) in enumerate(zip(a,b)):
    if x.startswith('B'): cur=x; start=i
    if x!=y:
        print('first diff at',i,'in block',cur,'index',i-start-1,'C:',x,'Rust:',y); break
else:
    print('no diff in',len(a),len(b))
PY
