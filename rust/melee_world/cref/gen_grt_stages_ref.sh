#!/usr/bin/env bash
# Build the C state-oracle references for the gr/grt*.c per-stage TU ports (shard 593-600).
# Generates the Rust ports + test + C side with gen_grt_stages.py, compiles every UNMODIFIED
# src/melee/gr/<stem>.c natively (game flags, contraction ON, then lower.py), runs each driver and
# writes tests/data/tu_<stem>_ref.txt.
# Usage (Git Bash): bash rust/melee_world/cref/gen_grt_stages_ref.sh
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../../.." && pwd)"
out="${TMPDIR:-${TEMP:-/tmp}}/melee_world_cref_grt"
rm -rf "$out"; mkdir -p "$out"
python "$here/gen_grt_stages.py" "$root" "$out"
CL="${CLANG:-/c/Program Files/LLVM/bin/clang.exe}"
F=(--target=x86_64-pc-windows-msvc -std=gnu99 -fno-builtin -DLINT -DTARGET_PC -fno-short-enums
   -fsigned-char -mlong-double-64 -fno-strict-aliasing -fwrapv -fcommon -fgnu89-inline -ftrivial-auto-var-init=zero -O2
   -Xclang -disable-llvm-passes -S -emit-llvm -w -I"$out" -I"$out/inc")
cd "$out"
if [ "${GRT_SET:-a}" = a ]; then STEMS="grtdrmario grtemblem grtest grtfalco grtfox grtgamewatch grtganon grticeclimber"; else STEMS="grtkirby grtkoopa grtlink grtluigi grtmario grtmars grtmewtwo grtness"; fi
for stem in $STEMS; do
  "$CL" "${F[@]}" "${stem}_drv.c" -o "$stem.ll"
  python "$root/rust/melee_runtime/oracle/lower.py" "$stem.ll" "$stem.lo.ll" >/dev/null
  "$CL" -O2 -fno-builtin -ffp-contract=off -w "$stem.lo.ll" -o "$stem.exe"
  "./$stem.exe" > "$here/../tests/data/tu_${stem}_ref.txt"
  wc -l "$here/../tests/data/tu_${stem}_ref.txt"
done
