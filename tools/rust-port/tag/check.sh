#!/bin/bash
export MELEE_WINDOW_HIDDEN=${MELEE_WINDOW_HIDDEN:-1}  # agents' game windows are invisible (MELEE_WINDOW_HIDDEN=0 shows them)
# Tag fighter checks (rust/mods/tag_fighter): a scripted 1v1 with three tags (one skipped).
#   1. golden run with the mods: the tags happen (log), two runs give identical NHV1 CSVs;
#   2. synctest (rollback + resimulate every frame, depth 4) across the tags: 0 mismatches;
#   3. synctest with MELEE_SNAP_CHECK=1: every snapshot save / load checked page by page across the tags.
#   EXE=<melee.exe> bash tools/rust-port/tag/check.sh   (build the mods first: tools/rust-port/modhost/build_mods.sh)
set -u
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"; cd "$repo"
ws="${GW_WORKSPACE:-E:/Projects/Melee Workspace}"
if [ -z "${GW_ISO_VANILLA:-}" ] && [ -f "$ws/.env" ]; then set -a; . "$ws/.env"; set +a; fi
[ -n "${GW_ISO_VANILLA:-}" ] || { echo "tag: GW_ISO_VANILLA not set" >&2; exit 2; }
exe="${EXE:-$repo/build/cargo/release/melee.exe}"
export MELEE_WINDOW_UNFOCUSED=1 MELEE_PAD_IGNORE_ADAPTER=1 MELEE_VOLUME=3 MELEE_SCRIPTS=0
export MELEE_WASM_MODS="$repo/build/mods-tag" MELEE_TAG_PARTNER=marth MELEE_PAD_SCRIPT="$here/tag_p1.txt" MELEE_PAD_CHANNELS=0
mkdir -p "$MELEE_WASM_MODS"; cp "$repo/build/mods/tag_fighter.wasm" "$MELEE_WASM_MODS/"
frames="${FRAMES:-900}"; s=pad_falcon_fox_fd
tmp="${TEMP:-/tmp}/tagcheck.$$"; mkdir -p "$tmp"; fail=0
run() { MELEE_WINDOW_TITLE="tagfight release $s:$1" "$exe" golden --engine tu --iso "$GW_ISO_VANILLA" --scenario $s --frames "$frames" --out "$tmp/$1.csv" >"$tmp/$1.log" 2>&1; }
run a; run b
n=$(grep -c "^tag: slot" "$tmp/a.log"); grep -E "^tag: slot|modhost: loaded|tag_fighter" "$tmp/a.log" | head
[ "$n" -ge 3 ] && echo "1a. tags applied: $n" || { echo "1a. FAIL: $n tags (want 3)"; fail=1; }
cmp -s "$tmp/a.csv" "$tmp/b.csv" && [ -s "$tmp/a.csv" ] && echo "1b. two runs identical ($(wc -l <"$tmp/a.csv") rows)" || { echo "1b. FAIL: runs differ"; fail=1; }
MELEE_WINDOW_TITLE="tagfight release $s:synctest" "$exe" synctest --scenario $s --frames "$frames" --iso "$GW_ISO_VANILLA" >"$tmp/sync.log" 2>&1; rc=$?
grep -c "^tag: slot" "$tmp/sync.log" | sed 's/^/2. tag swaps seen in synctest (incl. resimulation): /'; tail -2 "$tmp/sync.log"
[ $rc = 0 ] && echo "2. synctest exit 0" || { echo "2. FAIL: synctest exit $rc"; fail=1; }
MELEE_SNAP_CHECK=1 MELEE_SNAP_STATS=1 MELEE_WINDOW_TITLE="tagfight release $s:snapcheck" "$exe" synctest --scenario $s --frames "$frames" --iso "$GW_ISO_VANILLA" >"$tmp/snap.log" 2>&1; rc=$?
grep -iE "checks|mismatch" "$tmp/snap.log" | tail -3  # a page difference asserts (non-zero exit)
[ $rc = 0 ] && echo "3. snap check exit 0" || { echo "3. FAIL: snap check exit $rc"; fail=1; }
echo "logs: $tmp"; [ $fail = 0 ] && echo "TAG OK" || { echo "TAG FAIL"; exit 1; }
