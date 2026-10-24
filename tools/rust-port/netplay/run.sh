#!/bin/bash
# usage: run.sh port "extra args"
cd "E:/Projects/Melee Workspace/worktrees/plat-net"; set -a; . "E:/Projects/Melee Workspace/.env"; set +a
B="$PWD/build/cargo/release/melee.exe"; G=docs/rust-port/golden/pad_falcon_fox_fd.csv; O="${NP_SCRATCH:-$TEMP/np}"; P=$1; shift
"$B" netplay --host --port $P --golden $G --out "$O/h.csv" "$@" > "$O/h.log" 2>&1 &
sleep 3
"$B" netplay --join 127.0.0.1:$P --golden $G --out "$O/g.csv" "$@" > "$O/g.log" 2>&1; echo guest rc=$?; wait
for f in h g; do grep -v "tick " "$O/$f.log" | cut -c1-450 | tail -n 5; done
cmp "$O/h.csv" "$O/g.csv" && echo CSV-IDENTICAL
