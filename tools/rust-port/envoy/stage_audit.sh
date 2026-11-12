#!/bin/bash
# Envoy stage audit (notes/envoy-stage-audit.md): can a stage be started on and shifted into / out of during an Envoy fight?
#   GW_ISO_VANILLA=... bash tools/rust-port/envoy/stage_audit.sh GRKIND [EXE] [FRAMES]
# GRKIND is the ground kind of the stage under test (Kongo Jungle 64 = 30, Yoshi's Island 64 = 29, Pokemon Stadium = 16).
# Builds a private envoy_sim (STAGES = BF, X, BF, X, BF; a shift every 600 frames instead of 1800), then runs the headless
# golden twice with the Envoy test knobs (MELEE_ENVOY_ARM, _TEST_P1CPU, _START_FIGHT=3) and MELEE_STAGE_HOP_EXTRA=GRKIND:
#   * the two per-frame hash CSVs must be identical (determinism);
#   * each stage shift must freeze the hash for 90 frames (the wipe: switch at the midpoint);
#   * "heap0 free" after every shift is printed (the leak per shift).
# Needs build/mods deps like build_mods.sh (wasm32-unknown-unknown). Writes to $OUT (default: a temp dir).
set -u
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"; cd "$repo"
kind="${1:?usage: stage_audit.sh GRKIND [EXE] [FRAMES]}"; exe="${2:-$repo/build/cargo/release/melee}"; [ -e "$exe" ] || exe="$exe.exe"
frames="${3:-7200}"
[ -n "${GW_ISO_VANILLA:-}" ] || { echo "stage_audit: GW_ISO_VANILLA not set"; exit 2; }
out="${OUT:-$(mktemp -d)}"; mkdir -p "$out/mod" "$out/mods"
cp -r rust/mods/envoy_sim/Cargo.toml rust/mods/envoy_sim/src "$out/mod/"
sed -i "s/^const STAGES: \[i32; 5\] = .*/const STAGES: [i32; 5] = [36, $kind, 36, $kind, 36];/; s/^const STAGE_PERIOD: i32 = 1800;/const STAGE_PERIOD: i32 = 600;/" "$out/mod/src/lib.rs"
(cd "$out/mod" && RUSTFLAGS="-C target-feature=-simd128 -C link-arg=--initial-memory=1048576 -C link-arg=--max-memory=1048576 -C link-arg=-zstack-size=16384 -C link-arg=--export-table" \
  cargo build -j 4 --quiet --release --target wasm32-unknown-unknown --target-dir "$out/target") || exit 1
cp "$out/target/wasm32-unknown-unknown/release/envoy_sim.wasm" "$out/mods/"
printf 'envoy_audit\tvs\t%s\tidle_x\tmode=vs;at=match;p1=ck:2/human;p2=ck:9/human;p3=ck:20/cpu9;p4=none;stage=bf;match=time;minutes=0;items=off\n' "$frames" >"$out/scen.tsv"
export MELEE_VOLUME=3 MELEE_WASM_MODS="$out/mods" MELEE_ENGINE_PRE_DRAWS=0 MELEE_SCENARIO_TSV="$out/scen.tsv"
export MELEE_ENVOY_ARM=1 MELEE_ENVOY_TEST_P1CPU=1 MELEE_ENVOY_START_FIGHT=3 MELEE_STAGE_HOP_EXTRA="$kind"
for r in 1 2; do
  MELEE_WINDOW_TITLE="stage audit $kind run$r" "$exe" golden --engine tu --iso "$GW_ISO_VANILLA" --scenario envoy_audit --frames "$frames" --out "$out/r$r.csv" >"$out/r$r.log" 2>&1
done
if cmp -s "$out/r1.csv" "$out/r2.csv"; then echo "ok   determinism: $(wc -l <"$out/r1.csv") rows identical"; else echo "FAIL determinism"; fi
grep -c "panicked" "$out/r1.log" | sed 's/^/panics: /'
grep "^stage switch:" "$out/r1.log" | sed 's/ (.*heap0 free / heap0 free /'
python - "$out/r1.csv" <<'PY'
import sys, csv
rows = list(csv.reader(open(sys.argv[1])))[1:]
prev = None; start = 0; runs = []
for r in rows:
    f, h = int(r[0]), r[1]
    if h != prev:
        if prev is not None and f - start >= 20: runs.append((start, f - 1))
        start, prev = f, h
print("frozen frame runs (first = the reward screen; two halves of 44-45 = one 90-frame wipe):", runs)
PY
echo "logs: $out"
