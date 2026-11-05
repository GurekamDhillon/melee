#!/bin/bash
# C reference lane for a tree that has TUs the workspace's shared lists do not know yet (the doldecomp 598987cf3
# sync added ft_0763, gm_17DB, if_2FFC, if_3004, crowdsfx_1, hsd_3910). The shared `_build/masstest/files.txt` and
# `_build/melee_link_objects.rsp` belong to the workspace repo and serve every lane, so they are NOT edited: this
# script compiles the extra TUs into the lane and writes a lane-private link list instead.
#   GW_MELEE=<this worktree> GW_BUILD_ROOT=<lane> bash tools/rust-port/c_lane_newtus.sh
#   then: GW_LINK_OBJECTS=<lane>/melee_link_objects_newtus.rsp tools/port/build.sh   (the line is printed)
# Extra TUs = docs/rust-port/game_tus.txt minus the shared files.txt, restricted to sources this tree has.
# Re-run after editing one of them (build.sh only rebuilds the TUs in the shared list).
set -eu
ws="${GW_WORKSPACE:-E:/Projects/Melee Workspace}"
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
melee="${GW_MELEE:-$(cd "$here/../.." && pwd)}"
lane="${GW_BUILD_ROOT:?set GW_BUILD_ROOT to the lane (tools/port/agent_new.sh)}"
shared="$ws/_build/masstest/files.txt"
extra=()
while IFS= read -r f; do
  f="${f%$'\r'}"; [ -n "$f" ] && [ -f "$melee/$f" ] || continue
  grep -qxF "$f" <(tr -d '\r' <"$shared") || extra+=("$f")
done <"$melee/docs/rust-port/game_tus.txt"
echo "c_lane_newtus: ${#extra[@]} TU(s) beyond the shared list: ${extra[*]:-none}"
args=(); for f in "${extra[@]}"; do args+=(--tu "$f"); done
if [ ${#extra[@]} -gt 0 ]; then
  GW_MELEE="$melee" python "$ws/tools/port/build_objects.py" game --melee "$melee" --out "$lane/masstest/out" "${args[@]}"
fi
python "$ws/tools/port/link_response.py" --root "$ws" --build-root "$lane"
rsp="$lane/melee_link_objects_newtus.rsp"
cp "$lane/melee_link_objects.rsp" "$rsp"
for f in "${extra[@]}"; do
  obj="$lane/masstest/out/${f//\//_}.obj"
  [ -f "$obj" ] || { echo "c_lane_newtus: missing $obj" >&2; exit 1; }
  echo "\"$(cd "$(dirname "$obj")" && pwd -W 2>/dev/null || pwd)/$(basename "$obj")\"" >>"$rsp"
done
echo "c_lane_newtus: GW_LINK_OBJECTS=$rsp"
