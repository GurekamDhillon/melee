#!/bin/bash
# gwtool wrapper that turns on LLVM entry instrumentation (-finstrument-functions semantics) for a game TU,
# then runs the real gwtool (whose O2 pipeline inserts the calls). Used ONLY for the call-trace build (see tools/rust-port/trace/README.md):
#   export GW_GWTOOL=<this script>      (build_objects.py / pipe_win.sh honour GW_GWTOOL)
# The normal build never sees it. Every function DEFINED in the TU gets, at entry (before inlining),
#   call void @__cyg_profile_func_enter(ptr @fn, ptr returnaddress)
# gwtool adds the call after symbol renaming, so it is the plain C symbol: the native shim pc/platform/gw_calltrace.c
# provides it. Inputs: the same argv pipe_win.sh gives gwtool ("<in>.bc -o <obj> --imports=<f>").
set -euo pipefail
tools="${GW_LLVM_BIN:-${GW_ROOT:?}/_toolchains/llvm/bin}"
real="${GW_GWTOOL_REAL:-$GW_ROOT/_build/gwtool/gwtool.exe}"
args=("$@")
in=""
for ((i = 0; i < ${#args[@]}; i++)); do
    case "${args[$i]}" in
    -o|--target|--prefix|--emit|--mcpu|--mattr|--opt) i=$((i + 1)) ;;
    -*) ;;
    *) in="${args[$i]}"; idx=$i ;;
    esac
done
[ -n "$in" ] || { echo "gwtool_trace: no input" >&2; exit 2; }
tmp="$in.trace.$$"
trap 'rm -f "$tmp.ll" "$tmp.bc"' EXIT
"$tools/llvm-dis.exe" "$in" -o "$tmp.ll"
# Mark every attribute group; the pass only instruments functions that have a body.
sed -i -E 's/^(attributes #[0-9]+ = \{)/\1 "instrument-function-entry-inlined"="__cyg_profile_func_enter"/' "$tmp.ll"
"$tools/llvm-as.exe" "$tmp.ll" -o "$tmp.bc"
args[$idx]="$tmp.bc"
"$real" "${args[@]}"
