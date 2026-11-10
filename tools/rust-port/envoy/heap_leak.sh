#!/bin/bash
# Envoy milestone 0, the stage-switch heap leak check (full-game plan R2): SHIFTS stage switches (default 100) through BF, Yoshi's
# Story, Kongo Jungle 64, Dream Land, Yoshi's Island 64, Fountain, frozen Pokemon Stadium and FD, PERIOD logic frames apart (default
# 60), with the stage_switch mod's C twin (ROTATION_EXT). Prints heap0 free after each switch and fails when, after the first full
# lap (every stage's one-time pools warmed), heap0 free at a stage is lower than it was on that stage's first visit after the lap.
#   GW_ISO_VANILLA=... bash tools/rust-port/envoy/heap_leak.sh [EXE] [SHIFTS] [PERIOD]
set -u
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"; cd "$repo"
exe="${1:-$repo/build/cargo/release/melee}"; [ -e "$exe" ] || exe="$exe.exe"
shifts="${2:-100}"; period="${3:-60}"
[ -n "${GW_ISO_VANILLA:-}" ] || { echo "heap_leak: GW_ISO_VANILLA not set"; exit 2; }
out="${OUT:-$(mktemp -d)}"; mkdir -p "$out/mods"
PERIOD="$period" ROTATION_EXT=1 bash rust/mods/stage_switch/c/build.sh "$out/mods" >/dev/null || exit 1
frames=$(( (shifts + 2) * (period + 10) + 400 ))
printf 'heap_leak\tvs\t%s\tidle_x\tmode=vs;at=match;p1=ck:2/cpu9;p2=ck:9/cpu9;p3=none;p4=none;stage=bf;match=time;minutes=0;items=off\n' "$frames" >"$out/scen.tsv"
export MELEE_VOLUME=3 MELEE_WASM_MODS="$out/mods" MELEE_ENGINE_PRE_DRAWS=0 MELEE_SCENARIO_TSV="$out/scen.tsv"
export MELEE_STAGE_HOP_EXTRA=30,29,16 MELEE_SLIPPI_FROZEN_PS=1
MELEE_WINDOW_TITLE="m0 heap leak ${shifts}x" "$exe" golden --engine tu --iso "$GW_ISO_VANILLA" --scenario heap_leak --frames "$frames" --out "$out/r.csv" >"$out/r.log" 2>&1
python - "$out/r.log" "$shifts" <<'PY'
import re, sys
rows = [(int(a), int(b), int(c)) for a, b, c in re.findall(r"^stage switch: (-?\d+) -> (\d+) .*heap0 free (\d+)\)", open(sys.argv[1]).read(), re.M)]
want = int(sys.argv[2])
print(f"{len(rows)} switches; heap0 free first {rows[0][2] if rows else '-'} last {rows[-1][2] if rows else '-'}")
lap = 8
base = {}; worst = 0
for i, (_, k, free) in enumerate(rows):
    if i < lap: continue
    if k not in base: base[k] = free; continue
    worst = max(worst, base[k] - free)
for k in sorted(base):
    seen = [f for (_, kk, f) in rows[lap:] if kk == k]
    print(f"  grkind {k:2}: {len(seen)} visits, heap0 free {seen[0]} .. {seen[-1]} (min {min(seen)})")
ok = len(rows) >= want and worst <= 1024 * want  # at most 1 KB per switch (the PS pool residue is ~0.8 KB)
print(("ok  " if ok else "FAIL") + f" heap0 free is flat after the first lap (worst drop vs a stage's first visit {worst} bytes)")
sys.exit(0 if ok else 1)
PY
rc=$?
[ $rc -eq 0 ] || echo "logs: $out"
exit $rc
