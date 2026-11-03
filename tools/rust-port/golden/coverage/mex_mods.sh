#!/bin/bash
# Assemble the local m-ex fighter mod sets the `mex` coverage group mounts (scenarios.json env MELEE_MODS_DIR={ws}/_build/mex-mods/<set>).
# Every file is generated from the user's own sources (ports/README.md) and stays local: nothing here is committed.
#   bash tools/rust-port/golden/coverage/mex_mods.sh            (copies the current installs; override a source with SRC_<SET>=dir)
# Sets: none (empty: the ACE disc alone), halberd (metaknight-slot), ukirby (ultimate-kirby-slot), sora (ultimate-trail-slot), goku (goku-xv2).
set -eu
ws="${GW_WORKSPACE:-E:/Projects/Melee Workspace}"; out="$ws/_build/mex-mods"
declare -A src=(
  [halberd]="${SRC_HALBERD:-$ws/ports/halberd/mods-slot/metaknight-slot}"
  [ukirby]="${SRC_UKIRBY:-$ws/_build/tmp/ultimate-mods/ultimate-kirby-slot}"
  [sora]="${SRC_SORA:-$ws/_build/timing-audit/mods-sora/ultimate-trail-slot}"
  [goku]="${SRC_GOKU:-$ws/_build/animreload-live/mods/goku-xv2}"
)
mkdir -p "$out/none"
for s in "${!src[@]}"; do
  d="${src[$s]}"; [ -f "$d/mod.json" ] || { echo "$s: no install at $d (build it: ports/README.md)"; continue; }
  rm -rf "$out/$s"; mkdir -p "$out/$s"; cp -r "$d" "$out/$s/"; rm -rf "$out/$s"/*/_work
  echo "$s: $(basename "$d") from $d"
done
