#!/bin/bash
# Assemble the local m-ex fighter mod sets the `mex` coverage group mounts (scenarios.json env MELEE_MODS_DIR={ws}/_build/mex-mods/<set>).
# Every file is generated from the user's own sources (ports/README.md) and stays local: nothing here is committed.
#   bash tools/rust-port/golden/coverage/mex_mods.sh            (copies the current installs; override a source with SRC_<SET>=dir)
# Sets: none (empty: the ACE disc alone), halberd (metaknight-slot), ukirby (ultimate-kirby-slot), sora (ultimate-trail-slot), sorapal (ultimate-trail-slot built with --pc-palette 64: engine pobj_palette 1, Marth host), goku (goku-xv2), courier (vanilla-courier: pc/geno/mods/vanilla-courier with the original-art assets tools/geno/build_courier.sh writes, SRC_COURIER_FILES).
set -eu
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ws="${GW_WORKSPACE:-E:/Projects/Melee Workspace}"; out="$ws/_build/mex-mods"
declare -A src=(
  [halberd]="${SRC_HALBERD:-$ws/ports/halberd/mods-slot/metaknight-slot}"
  [ukirby]="${SRC_UKIRBY:-$ws/_build/tmp/ultimate-mods/ultimate-kirby-slot}"
  [sora]="${SRC_SORA:-$ws/_build/timing-audit/mods-sora/ultimate-trail-slot}"
  [sorapal]="${SRC_SORAPAL:-$ws/_build/tmp/ultimate-mods/ultimate-trail-slot-v9}"
  [goku]="${SRC_GOKU:-$ws/_build/animreload-live/mods/goku-xv2}"
)
mkdir -p "$out/none"
for s in "${!src[@]}"; do
  d="${src[$s]}"; [ -f "$d/mod.json" ] || { echo "$s: no install at $d (build it: ports/README.md)"; continue; }
  rm -rf "$out/$s"; mkdir -p "$out/$s"; cp -r "$d" "$out/$s/"; rm -rf "$out/$s"/*/_work
  echo "$s: $(basename "$d") from $d"
done
# courier: the text fixture of this checkout plus its built assets (original art, never committed)
cf="${SRC_COURIER_FILES:-$ws/_build/geno-slice4/courier}"
if [ -f "$cf/plan.json" ]; then
  rm -rf "$out/courier"; mkdir -p "$out/courier"; cp -r "$here/../../../../pc/geno/mods/vanilla-courier" "$out/courier/"
  cp "$cf"/GnCourier*.dat "$cf/plan.json" "$out/courier/vanilla-courier/files/"
  echo "courier: vanilla-courier with assets from $cf"
else
  echo "courier: no built assets at $cf (tools/geno/build_courier.sh --install)"
fi
