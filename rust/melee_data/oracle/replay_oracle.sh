#!/usr/bin/env bash
# Build the gw_replay.c .slp reader oracle (rp struct .. rp_load cut out of the shipping file) and run
# it over the synthetic corpus (tests/data/slp, from gen_slp.py) and a sample of real replays
# (REAL_SLP_DIR, default the workspace Gang-Steals folder, every 17th file).
# -> tests/data/replay_oracle.txt (synthetic), tests/data/replay_real_oracle.txt (real)
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../../.." && pwd)"
out="$root/rust/target/replay-oracle"
CLANG="${CLANG:-clang}"
command -v "$CLANG" >/dev/null 2>&1 || CLANG="/c/Program Files/LLVM/bin/clang.exe"
mkdir -p "$out"
awk '/^#define GW_RP_GAME_INFO/{p=1} /^\/\* LIVE MODE/{p=0} p' "$root/pc/platform/gw_replay.c" > "$out/replay_cut.inc"
"$CLANG" -O1 -w -D_CRT_SECURE_NO_WARNINGS -I"$out" -I"$root/pc/platform" "$here/replay_oracle.c" -o "$out/replay_oracle.exe"
rm -rf "$here/../tests/data/slp"
python "$here/gen_slp.py" "$here/../tests/data/slp"
(cd "$here/../tests/data/slp" && ls *.slp | LC_ALL=C sort | xargs "$out/replay_oracle.exe") | tr -d '\r' > "$here/../tests/data/replay_oracle.txt"
echo "synthetic: $(wc -l < "$here/../tests/data/replay_oracle.txt") files"
REAL="${REAL_SLP_DIR:-$root/../../Gang-Steals}"
if [ -d "$REAL" ]; then
  (cd "$REAL" && find . -name '*.slp' | sed 's|^./||' | LC_ALL=C sort | awk 'NR%17==0') > "$out/real_list.txt"
  tr '\n' '\0' < "$out/real_list.txt" | (cd "$REAL" && xargs -0 "$out/replay_oracle.exe") | tr -d '\r' > "$here/../tests/data/replay_real_oracle.txt"
  echo "real: $(wc -l < "$here/../tests/data/replay_real_oracle.txt") files"
fi
