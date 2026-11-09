#!/bin/bash
# Stage-switch mod check (rust/mods/stage_switch, notes/modhost.md "Stage switching"): pad_falcon_fox_fd with the mod
# loops FD -> BF -> Yoshi's Story -> Dream Land -> Fountain -> FD (a switch every 600 frames, inside the sim).
#   1. two golden runs with the mod: byte-identical per-frame hash CSVs, 5 switches logged;
#   2. synctest depth 4 over all 3601 frames with MELEE_SNAP_CHECK=1: 0 mismatches (every switch is rolled back and resimulated).
#   [PROFILE=release] bash tools/rust-port/modhost/stage_switch.sh   (needs build/mods/stage_switch.wasm: build_mods.sh)
set -u
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"; cd "$repo"
ws="${GW_WORKSPACE:-E:/Projects/Melee Workspace}"
if [ -z "${GW_ISO_VANILLA:-}" ] && [ -f "$ws/.env" ]; then set -a; . "$ws/.env"; set +a; fi
[ -n "${GW_ISO_VANILLA:-}" ] || { echo "stage_switch: GW_ISO_VANILLA not set" >&2; exit 2; }
exe="$repo/build/cargo/${PROFILE:-release}/melee.exe"
mods="${MODS:-$repo/build/mods}"; d="$(mktemp -d)"; cp "$mods/stage_switch.wasm" "$d/" || exit 2
export MELEE_WINDOW_UNFOCUSED=1 MELEE_PAD_IGNORE_ADAPTER=1 MELEE_VOLUME=3 MELEE_SCRIPTS=0 MELEE_WASM_MODS="$d" MELEE_ENGINE_PRE_DRAWS=0
fail=0
for r in 1 2; do MELEE_WINDOW_TITLE="stagesw golden run$r" "$exe" golden --engine tu --iso "$GW_ISO_VANILLA" --scenario pad_falcon_fox_fd --frames 3601 --out "$d/r$r.csv" >"$d/r$r.log" 2>&1; done
n=$(grep -c "^stage switch:" "$d/r1.log")
if cmp -s "$d/r1.csv" "$d/r2.csv" && [ "$n" -eq 5 ] && [ "$(wc -l <"$d/r1.csv")" -eq 3602 ]; then echo "1. golden with the mod: 2 runs identical, $n switches"; else echo "1. FAIL: runs differ or $n switches"; fail=1; fi
MELEE_SNAP_CHECK=1 MELEE_WINDOW_TITLE="stagesw synctest" "$exe" synctest --engine tu --depth 4 --scenario pad_falcon_fox_fd --frames 3601 --timeout 1600 >"$d/sync.log" 2>&1 || fail=1
grep "engine synctest" "$d/sync.log"; grep -q " 0 MISMATCHES" "$d/sync.log" && ! grep -q "MELEE_SNAP_CHECK:" "$d/sync.log" || { echo "2. FAIL: synctest"; fail=1; }
echo "logs: $d"; [ $fail = 0 ] && echo "STAGE SWITCH OK" || { echo "STAGE SWITCH FAIL"; exit 1; }
