#!/bin/bash
# Quick-swap cast checks (docs/rust-port/notes/quick-swap.md; rust/mods/quick_swap, MELEE_TAG_INPLACE=1).
#   1. real-time play run, windowed, MELEE_PROFILE=1: P1 swaps on D-pad down every 40 frames (swap_p1.txt) against a
#      level-1 CPU on FD; prints the swaps, the per-swap spawn/despawn cost, the worst tick, heap0 free before / after,
#      and runs zero_hitch_check.py on the profile;
#   2. synctest (rollback + resimulate, depth 7) across the swaps: 0 mismatches.
#   ISO=VANILLA|ACE  EXE=<melee.exe>  MODS=<dir with quick_swap.wasm>  bash tools/rust-port/quickswap/check.sh
# The disc path comes from the workspace .env (GW_ISO_<ISO>) and is never printed.
set -u
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"
ws="${GW_WORKSPACE:-E:/Projects/Melee Workspace}"
[ -f "$ws/.env" ] && { set -a; . "$ws/.env"; set +a; }
which="${ISO:-VANILLA}"; var="GW_ISO_$which"; iso="${!var:-}"
[ -n "$iso" ] || { echo "quickswap: $var not set"; exit 2; }
exe="${EXE:-$repo/build/cargo/release/melee.exe}"
mods="${MODS:-$repo/build/mods}"
p1="${P1:-falcon}"; partner="${PARTNER:-marth}"; stage="${STAGE:-FINAL DESTINATION}"
frames="${FRAMES:-9000}"; build=$(git -C "$repo" rev-parse --short HEAD)
out="${OUT:-$repo/build/quickswap/$which-$p1-$partner}"; rm -rf "$out"; mkdir -p "$out/bin" "$out/mods"
cp "$exe" "$out/bin/melee.exe"; cp "$mods/quick_swap.wasm" "$out/mods/"
export MELEE_VOLUME=3 MELEE_FULLSCREEN=off MELEE_WINDOW_UNFOCUSED=1 MELEE_PAD_IGNORE_ADAPTER=1 MELEE_SCRIPTS=0
export MELEE_WASM_MODS="$out/mods" MELEE_TAG_INPLACE=1 MELEE_TAG_PARTNER="$partner" MELEE_PAD_CHANNELS=0
export MELEE_PAD_SCRIPT="${PAD:-$here/swap_p1.txt}"
fail=0
if [ -z "${SKIP_PLAY:-}" ]; then
  ( cd "$out/bin" && MELEE_PROFILE=1 MELEE_WINDOW_TITLE="quickswap $build $which play $p1+$partner" \
      timeout "${TIMEOUT:-400}" ./melee.exe play --iso "$iso" --max-frames "$frames" \
      --match "$p1,fox/cpu1,$stage" --time-secs 600 >"$out/play.log" 2>&1 )
  echo "1. play exit $?"
  python - "$out/play.log" <<'PY'
import re, sys
t = open(sys.argv[1], errors="replace").read()
rows = re.findall(r"^tag: slot \d+ (-?\d+) -> (-?\d+) at frame (\d+) \(in place: spawn (\d+) us, despawn (\d+) us, heap0 free (\d+), heap1 free (\d+)\)", t, re.M)
sw = [r[:5] for r in rows]
print(f"   swaps: {len(sw)}; refused presses: {len(re.findall('quick_swap: refused', t))}")
if sw:
    tot = [int(a) + int(b) for *_, a, b in sw]
    print(f"   swap cost (spawn+despawn) first {tot[0]} us, worst {max(tot)} us, worst after the first {max(tot[1:] or [0])} us, median {sorted(tot)[len(tot)//2]} us")
    print(f"   worst over the first 50: {max(tot[:50])} us")
    h0 = [int(r[5]) for r in rows]; h1 = [int(r[6]) for r in rows]
    print(f"   heap0 free after swap 2: {h0[min(1,len(h0)-1)]}, after swap {len(h0)}: {h0[-1]} (min {min(h0)}); heap1 free {h1[min(1,len(h1)-1)]} -> {h1[-1]}")
    ev = [h0[i] for i in range(1, len(h0), 2)]  # same character active after every second swap
    print(f"   heap0 drift on same-side swaps: {ev[0] - ev[-1] if ev else 0} bytes over {len(ev)} pairs")
PY
  p=$(ls -t "$out/bin/profiles/"profile-*.csv 2>/dev/null | head -1)
  if [ -n "$p" ]; then python "$repo/tools/rust-port/perf/zero_hitch_check.py" "$p" --label "quickswap-$which" >"$out/hitch.txt"; [ $? = 0 ] || fail=1; head -25 "$out/hitch.txt"
  else echo "   no profile written"; fail=1; fi
fi
if [ -z "${SKIP_SYNC:-}" ]; then
  MELEE_WINDOW_TITLE="quickswap $build $which synctest" timeout "${TIMEOUT:-900}" "$exe" synctest --engine tu --iso "$iso" \
    --scenario pad_falcon_fox_fd --depth 7 --frames "${SYNC_FRAMES:-2400}" >"$out/sync.log" 2>&1; rc=$?
  echo "2. synctest exit $rc; swaps applied incl. resimulation: $(grep -c '^tag: slot' "$out/sync.log")"
  grep 'engine synctest' "$out/sync.log" | tail -1
  [ $rc = 0 ] || fail=1
fi
echo "logs: $out"; [ $fail = 0 ] && echo "QUICKSWAP OK" || { echo "QUICKSWAP FAIL"; exit 1; }
