#!/bin/bash
export MELEE_WINDOW_HIDDEN=${MELEE_WINDOW_HIDDEN:-1}  # agents' game windows are invisible (MELEE_WINDOW_HIDDEN=0 shows them)
# Two engine peers over the Slippi protocol (ENet, loopback), headless. usage: np_pair.sh BASEPORT OUTDIR [extra args...]
# Extra args go to both peers (--frames N, --delay D, --lag-ms N, --loss-pct P). Checksum CSVs must be identical.
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"
ws="${GW_WORKSPACE:-E:/Projects/Melee Workspace}"; set -a; . "$ws/.env"; set +a
B="${EXE:-$repo/build/cargo/release/melee.exe}"; P=$1; O=$2; shift 2; mkdir -p "$O"
export MELEE_VOLUME=3 MELEE_WINDOW_UNFOCUSED=1 MELEE_PAD_IGNORE_ADAPTER=1
MELEE_WINDOW_TITLE="slippi2:np:host" "$B" slippi-netplay --host --port $P --peer 127.0.0.1:$((P+1)) --out "$O/h.csv" "$@" > "$O/h.log" 2>&1 &
MELEE_WINDOW_TITLE="slippi2:np:guest" "$B" slippi-netplay --join --port $((P+1)) --peer 127.0.0.1:$P --out "$O/g.csv" "$@" > "$O/g.log" 2>&1; g=$?; wait $!; h=$?
tail -n 1 "$O/h.log"; tail -n 1 "$O/g.log"; echo "host rc=$h guest rc=$g"
python -I -c "import sys
a=open(sys.argv[1]).read().split()[1:]; b=open(sys.argv[2]).read().split()[1:]; n=min(len(a),len(b))
d=[x for x,y in zip(a,b) if x!=y]; u=len(set(r.split(',')[1] for r in a))
print(('CSV-EQUAL' if not d else 'CSV-DIFFER %d rows, first %s' % (len(d), d[0])), n, 'common rows,', u, 'distinct checksums')" "$O/h.csv" "$O/g.csv"
