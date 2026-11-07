#!/bin/bash
# SPDX-License-Identifier: MIT
# Name the fields behind one checked match's first NHV1 difference (notes/coverage-testing.md):
#   bash tools/rust-port/covtest/classify.sh CORPUS ID [inputs|ai]
# Plays the .slp in Dolphin once more keeping the raw dump, then `melee dolphin-wide` and `slpcheck --wide-out` over
# the 3 frames before the first difference, and wide_diff.py (notes/dolphin-oracle.md). Output: checks/ID/<mode>/wide_diff.txt
set -u
C=$1; ID=$2; MODE=${3:-inputs}
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ws="${GW_WORKSPACE:-E:/Projects/Melee Workspace}"
EXE=${MELEE_EXE:-$ws/_build/cargo-agents/covtest/cov/melee.exe}
set -a; [ -f "$ws/.env" ] && . "$ws/.env" >/dev/null 2>&1; set +a
chk="$C/checks/$ID"; slp="$C/slp/$ID.slp"; w="$chk/$MODE/wide"; mkdir -p "$w"
F=$(grep -m1 -o 'first divergence at frame -\?[0-9]*' "$chk/$MODE/compare.txt" | grep -o -- '-\?[0-9]*$')
[ -n "$F" ] || { echo "CLASSIFY $ID: no difference"; exit 0; }
S=$((F - 123)); A=$((S - 3)); [ $A -lt -123 ] && A=-123
flag=--cpu-slots; [ "$MODE" = ai ] && flag=--cpu-ai
DOLPHIN_USER_DIR="$(cygpath -w "$C/lanes/classify_user")" powershell -NoProfile -ExecutionPolicy Bypass -File "$(cygpath -w "$here/../dolphin/run_dolphin.ps1")" \
  -Slp "$(cygpath -w "$slp")" -Out "$(cygpath -w "$w")" > /dev/null 2>&1
"$EXE" dolphin-wide --raw "$w/dolphin.nhraw" --out "$w/dolphin_wide.txt" --from $A --to $S 2>/dev/null
LLVM_PROFILE_FILE="$w/x.profraw" MELEE_WINDOW_UNFOCUSED=1 MELEE_VOLUME=3 "$EXE" slpcheck --slp "$slp" --full --iso "$GW_ISO_VANILLA" $flag \
  --wide-out "$w/rust_wide.txt" --wide-from $A --wide-to $S > /dev/null 2>&1
p=""; [ $A = $S ] && p=--persistent
python "$here/../dolphin/wide_diff.py" "$w/rust_wide.txt" "$w/dolphin_wide.txt" --all-frames --max 12 $p > "$chk/$MODE/wide_diff.txt"
rm -f "$w/dolphin.nhraw" "$w"/*.profraw "$w/rust_wide.txt" "$w/dolphin_wide.txt"
echo "CLASSIFY $ID frame $F: $(grep '^    ' "$chk/$MODE/wide_diff.txt" | awk '{print $1"."$2}' | sort -u | head -6 | tr '\n' ' ')"
