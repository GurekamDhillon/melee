#!/bin/bash
# The 1P records on the memory card, C build vs Rust engine: run one 1P scene launch with `skipmemcard=0` (the boot
# scene creates the save on a fresh card folder, the mode's saves reach it) in both, compare the NHV1 rows, then load
# each card in the Rust engine (MELEE_CARD_DUMP) and compare the decoded GameCube images of GmSaveData and the
# name-tag banks byte for byte (the .gci files are encrypted, so they differ wholesale on one changed byte).
#   bash tools/rust-port/golden/coverage/card_parity.sh OUTDIR NAME "SCENE" PAD     e.g.
#   bash tools/rust-port/golden/coverage/card_parity.sh "$TEMP/cp" tt_samus "mode=tt;p1=ck:16" randabs_s1_3600.lua
# C binary: C_BIN (default _build/agents/sfxparity), copied into _build/agents/${COVERAGE_AGENT:-cardparity}.
# Headless, no OS input, MELEE_VOLUME=3. Known difference: x1A54 (power-on seconds, image byte 0x1EC..0x1EF) comes
# from OSGetTime, the C turbo run's virtual clock vs the engine's host clock.
set -u
out="$1"; name="$2"; scene="$3"; pad="$4"
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"; wt="$(cd "$here/../../../.." && pwd)"
ws="${GW_WORKSPACE:-E:/Projects/Melee Workspace}"
[ -z "${GW_ISO_VANILLA:-}" ] && [ -f "$ws/.env" ] && { set -a; . "$ws/.env"; set +a; }
exe="${MELEE_EXE:-$wt/build/cargo/${PROFILE:-release}/melee.exe}"
cbin="${C_BIN:-$ws/_build/agents/sfxparity}"
padroot="$(cd "$wt/tools/rust-port/golden/coverage/pads" && pwd -W)"
export MELEE_VOLUME=3 MELEE_PAD_IGNORE_ADAPTER=1 MELEE_WINDOW_UNFOCUSED=1 MELEE_TEST_SEED=12345 MELEE_CARD_DIAG=1
mkdir -p "$out"; out="$(cd "$out" && pwd -W)"
run() { # run c|rust DIR
  rm -rf "$2"; mkdir -p "$2/card"; export MELEE_CARD_PATH="$2/card" MELEE_WINDOW_TITLE="cardparity:$1:$name"
  if [ "$1" = c ]; then
    root="$ws/_build/agents/${COVERAGE_AGENT:-cardparity}"; mkdir -p "$root/runs"
    for f in melee-pc.exe melee-pc.map SDL3.dll webgpu_dawn.dll; do cmp -s "$cbin/$f" "$root/$f" 2>/dev/null || cp -f "$cbin/$f" "$root/$f"; done
    GW_BUILD_ROOT="$root" GW_MELEE="$wt" MELEE_UNATTENDED=1 MELEE_WINDOW_X=30000 MELEE_WINDOW_Y=30000 MELEE_SCRIPTS=0 MELEE_TURBO=1 \
      SDL_WINDOW_ACTIVATE_WHEN_SHOWN=0 MELEE_PAD_SCRIPT="$padroot/$pad" MELEE_SCENE="$scene;skipmemcard=0" MELEE_NEUTRAL_HASHLOG="$2/hash.csv" \
      bash "$ws/tools/port/run.sh" --max-seconds 180 "cardparity-$name" --iso "$GW_ISO_VANILLA" > "$2/log.txt" 2>&1
  else
    printf '%s\t1p\t3600\t%s\t%s;skipmemcard=0\n' "$name" "$pad" "$scene" > "$2/scn.tsv"
    (cd "$wt" && MELEE_SCENARIO_TSV="$2/scn.tsv" "$exe" golden --engine tu --iso "$GW_ISO_VANILLA" --scenario "$name" --frames 1000000 --out "$2/hash.csv" > "$2/log.txt" 2>&1)
  fi
}
dump() { # load DIR/card in the engine, write DIR/img.*.bin
  rm -rf "$1/ld"; cp -r "$1/card" "$1/ld"
  printf 'ld\t1p\t3600\trandabs_s1_1800.lua\tmode=tt;p1=ck:8;skipmemcard=0\n' > "$1/ld.tsv"
  (cd "$wt" && MELEE_WINDOW_TITLE="cardparity:dump:$name" MELEE_CARD_PATH="$1/ld" MELEE_CARD_DUMP="$1/img" MELEE_SCENARIO_TSV="$1/ld.tsv" \
    "$exe" golden --engine tu --iso "$GW_ISO_VANILLA" --scenario ld --frames 1 --out "$1/ld.csv" > "$1/ld.log" 2>&1)
}
run c "$out/c" & run rust "$out/rust"; wait
python "$wt/tools/rust-port/compare_golden.py" "$out/c/hash.csv" "$out/rust/hash.csv" | tail -1
dump "$out/c"; dump "$out/rust"
grep -h "save image -> native: events" "$out/c/ld.log" "$out/rust/ld.log" | sed 's/^/  /'
rc=0
for f in "$out"/c/img.*.bin; do
  b="$(basename "$f")"; n=$(cmp -l "$f" "$out/rust/$b" | wc -l); d=$n
  # cmp counts from 1: x1A54 is image bytes 493..496
  if [ "$b" = img.save.bin ]; then d=$(cmp -l "$f" "$out/rust/$b" | awk '$1 < 493 || $1 > 496' | wc -l); fi
  echo "$b: $n byte(s) differ, $d outside the power-on clock"; [ "$d" = 0 ] || rc=1
done
[ $rc = 0 ] && echo "CARD PARITY OK" || echo "CARD PARITY FAIL"; exit $rc
