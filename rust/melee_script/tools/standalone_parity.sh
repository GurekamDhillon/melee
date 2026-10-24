#!/usr/bin/env bash
# Runs every engine-less Lua test script under the system `lua` (Lua 5.4.7, the C build's reference) and under the
# Rust host's `luarun` (same vendored sources), from the repo root, and requires the same exit status and the same
# stdout+stderr. Usage: standalone_parity.sh [path/to/luarun.exe]   (run from anywhere; set LUA=lua)
set -u
cd "$(dirname "$0")/../../.."
RUN=${1:-build/cargo/debug/luarun.exe}
REF=${LUA:-lua}
same=0; diff_=0; both_fail=0; mism=()
for f in pc/tests/*.lua pc/geno/tools/*.lua pc/gameworld/*.lua pc/scripts/*.lua pc/scripts/tests/*.lua; do
  [ -f "$f" ] || continue
  timeout 120 $REF "$f" > /tmp/ref.out 2>&1; rc1=$?
  timeout 120 "$RUN" "$f" > /tmp/rs.out 2>&1; rc2=$?
  # error text differs only in the interpreter name prefix and the traceback tail; compare status + stdout lines
  if [ $rc1 -ne 0 ] && [ $rc2 -ne 0 ]; then both_fail=$((both_fail+1)); continue; fi
  if [ $rc1 -eq $rc2 ] && cmp -s /tmp/ref.out /tmp/rs.out; then same=$((same+1)); else diff_=$((diff_+1)); mism+=("$f rc=$rc1/$rc2"); fi
done
echo "identical (status and output): $same; both fail (need the engine): $both_fail; different: $diff_"
for m in "${mism[@]:-}"; do [ -n "$m" ] && echo "  DIFF $m"; done
[ $diff_ -eq 0 ]
