#!/bin/bash
# Prove snapshot resume: for each dumped frame N, `melee golden --from-snapshot` must match the golden CSV
# from N on at least as far as a from-0 run does.
#
#   bash tools/rust-port/snap/prove.sh [scenario] [frame ...]       (default: pad_falcon_fox_fd 0 300)
#
# Needs the dumps (tools/rust-port/snap/dump.sh) in $GW_BUILD_ROOT/snaps and a built melee
# (cd rust && cargo build -p melee). Prints the first diverging frame of each run; exit 0 = proven.
set -u
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
root="$(cd "$here/../../.." && pwd)"
ws="${GW_WORKSPACE:-E:/Projects/Melee Workspace}"
set -a; source "$ws/.env"; set +a
scenario="${1:-pad_falcon_fox_fd}"; shift || true
frames=("$@"); [ ${#frames[@]} -gt 0 ] || frames=(0 300)
exe="${MELEE_EXE:-$root/build/cargo/debug/melee.exe}"
snaps="${GW_BUILD_ROOT:?GW_BUILD_ROOT}/snaps"
out="$GW_BUILD_ROOT/snaps/prove"; mkdir -p "$out"
golden="$root/docs/rust-port/golden/$scenario.csv"
first() { python "$root/tools/rust-port/compare_golden.py" "$golden" "$1" --allow-short "${@:2}" 2>&1 | sed -n 's/.*first divergence at frame \([0-9]*\).*/\1/p'; }
"$exe" golden --iso "$GW_ISO_VANILLA" --scenario "$scenario" --out "$out/from0.csv" >/dev/null 2>&1
base="$(first "$out/from0.csv")"; base="${base:-none}"
echo "from-0 run: first divergence at frame $base"
rc=0
for n in "${frames[@]}"; do
    "$exe" golden --iso "$GW_ISO_VANILLA" --scenario "$scenario" --out "$out/res$n.csv" --from-snapshot "$snaps/$scenario.f$n.snap" --start "$n" >/dev/null 2>&1
    d="$(first "$out/res$n.csv" --from "$n")"; d="${d:-none}"
    # none (no divergence) beats any frame
    if [ "$d" = none ] || { [ "$base" != none ] && [ "$d" -ge "$base" ]; }; then v=OK; else v=FAIL; rc=1; fi
    echo "resume from frame $n: first divergence at frame $d  [$v]"
done
exit $rc
