#!/bin/bash
export MELEE_WINDOW_HIDDEN=${MELEE_WINDOW_HIDDEN:-1}  # agents' game windows are invisible (MELEE_WINDOW_HIDDEN=0 shows them)
# Mod host determinism check (docs/rust-port/notes/modhost.md):
#   1. mods off: the golden scenario still matches its committed CSV (no hook installed, hashes unchanged);
#   2. hello_mod loaded: two runs of the same scripted replay give byte-identical NHV1 CSVs;
#   3. the mod's effect is visible (the modded CSV differs from the vanilla one; first differing frame printed);
#   4. rollback: synctest (rollback + resimulate every frame) with the mod loaded reports no mismatch.
#   [PROFILE=release] [FRAMES=3601] [SCENARIO=cpu_fox_falco_dl] bash tools/rust-port/modhost/determinism.sh [--skip-build]
set -u
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"; cd "$repo"
ws="${GW_WORKSPACE:-E:/Projects/Melee Workspace}"
if [ -z "${GW_ISO_VANILLA:-}" ] && [ -f "$ws/.env" ]; then set -a; . "$ws/.env"; set +a; fi
[ -n "${GW_ISO_VANILLA:-}" ] || { echo "modhost: GW_ISO_VANILLA not set" >&2; exit 2; }
export MELEE_WINDOW_UNFOCUSED=1 MELEE_PAD_IGNORE_ADAPTER=1 MELEE_VOLUME=3 MELEE_SCRIPTS=0
profile="${PROFILE:-release}"; frames="${FRAMES:-3601}"; s="${SCENARIO:-cpu_fox_falco_dl}"; idle=""; case $s in cpu_*) idle="--idle-pads" ;; esac
exe="$repo/build/cargo/$profile/melee.exe"
if [ "${1:-}" != --skip-build ]; then
  bash "$here/build_mods.sh" || exit 1
  (cd rust && cargo build --quiet -p melee --profile "$profile") || exit 1
fi
tmp="${TEMP:-/tmp}/modhost.$$"; mkdir -p "$tmp"; fail=0
run() { MELEE_WINDOW_TITLE="modhost:$1" "$exe" golden --engine tu --iso "$GW_ISO_VANILLA" --scenario $s --frames "$frames" $idle --out "$tmp/$1.csv" >"$tmp/$1.log" 2>&1; }
MELEE_WASM_MODS= run off
n=$(python tools/rust-port/compare_golden.py "docs/rust-port/golden/$s.csv" "$tmp/off.csv" --allow-short --print-count 2>/dev/null | awk '/^COUNT/{print $2}')
echo "1. mods off: $s matches the golden for ${n:-0} frames"; [ "${n:-0}" -ge "$((frames-2))" ] || fail=1
MELEE_WASM_MODS="$repo/build/mods" run on1; MELEE_WASM_MODS="$repo/build/mods" run on2
grep modhost "$tmp/on1.log" | head -3
if cmp -s "$tmp/on1.csv" "$tmp/on2.csv" && [ -s "$tmp/on1.csv" ]; then echo "2. mod loaded: two runs identical ($(wc -l <"$tmp/on1.csv") rows, sha1 $(sha1sum "$tmp/on1.csv" | tr -d "\\\\" | cut -c1-12))"; else echo "2. FAIL: modded runs differ"; fail=1; fi
d=$(python - "$tmp/off.csv" "$tmp/on1.csv" <<'PY'
import sys
a=open(sys.argv[1]).read().splitlines(); b=open(sys.argv[2]).read().splitlines()
print(next((i for i,(x,y) in enumerate(zip(a,b)) if x!=y), -1))
PY
)
if [ "$d" -ge 0 ]; then echo "3. mod effect: first differing CSV row $d"; else echo "3. FAIL: the mod changed nothing"; fail=1; fi
MELEE_WASM_MODS="$repo/build/mods" MELEE_WINDOW_TITLE="modhost:synctest" "$exe" synctest --scenario $s --frames "${SYNC_FRAMES:-600}" --iso "$GW_ISO_VANILLA" >"$tmp/sync.log" 2>&1; rc=$?
tail -2 "$tmp/sync.log"; [ $rc = 0 ] && echo "4. synctest with the mod: exit 0" || { echo "4. FAIL: synctest exit $rc"; fail=1; }
echo "logs: $tmp"; [ $fail = 0 ] && echo "MODHOST OK" || { echo "MODHOST FAIL"; exit 1; }
