#!/bin/bash
# Play the committed C-recorded replays in the Rust sim (`melee golden --slp`) and compare with the C
# build's own MELEE_SLP playback of the same files (docs/rust-port/golden/slp/*.play.csv; recorded by
# record_slp.sh). The post-frame RNG seed column is skipped (replays force the seed every frame).
#   bash tools/rust-port/golden/check_slp.sh [exe]   (default: rust/../build/cargo/release/melee.exe)
set -u
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
root="$here/../../.."
exe="${1:-$root/build/cargo/release/melee.exe}"
ws="${GW_WORKSPACE:-E:/Projects/Melee Workspace}"
set -a; . "$ws/.env"; set +a
out="$(mktemp -d)"
rc=0
for n in pad_falcon_fox_fd; do
    "$exe" golden --iso "$GW_ISO_VANILLA" --slp "$root/docs/rust-port/golden/slp/$n.slp" --out "$out/$n.csv" 2>&1 | tail -2
    python "$here/../compare_golden.py" "$root/docs/rust-port/golden/slp/$n.play.csv" "$out/$n.csv" --allow-short --ignore-seed || rc=1
done
rm -rf "$out"
exit $rc
