#!/usr/bin/env bash
# Layout check part B: walk every archive on the vanilla disc (GW_ISO_VANILLA from the workspace .env; never printed).
#   bash tools/rust-port/layout/run_walk.sh [OUT.txt] [file-substring ...]
# Generates the walker's type table from melee_types gen.rs into the build dir, builds the example on the shared
# target dir, runs it. The output names disc files and field values: keep it out of git (default: build/layout/).
set -euo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
wt="$(cd "$here/../../.." && pwd)"
ws="$(cd "$wt/../.." && pwd)"
out="${1:-$wt/build/layout/walk.txt}"; shift || true
mkdir -p "$wt/build/layout" "$(dirname "$out")"
python "$here/gen_walk_table.py" "$wt/build/layout/walk_table.rs"
if [ -z "${GW_ISO_VANILLA:-}" ]; then
  GW_ISO_VANILLA="$(grep -E '^(export )?GW_ISO_VANILLA=' "$ws/.env" | head -1 | cut -d= -f2- | tr -d "\"'")"
fi
export GW_ISO_VANILLA
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$ws/_build/cargo-shared/release}"
export LAYOUT_WALK_TABLE="$(cd "$wt/build/layout" && pwd -W 2>/dev/null || pwd)/walk_table.rs"
cd "$wt/rust"
cargo build --release -p melee_engine --example layout_walk --features layout-walk
"$CARGO_TARGET_DIR/release/examples/layout_walk" "$@" > "$out" 2> "${out%.txt}.err" || { echo "walker failed: see ${out%.txt}.err"; tail -5 "${out%.txt}.err"; exit 1; }
grep -E "^(TOTAL|SKIPPED)" "$out"; echo "records compared: $(grep -c ^VISITED "$out")"
