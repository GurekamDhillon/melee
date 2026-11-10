#!/bin/bash
# Envoy milestone 1 (crests core): crests fire at their moments under rollback, deterministically.
#   0. envoy_data's lint and its tests (cargo test on the host target);
#   1. a pad run (tools/rust-port/envoy/m1_pad.txt) from fight 2 with test crests worn
#      (MELEE_ENVOY_TEST_CRESTS): the offer is taken with A, tags every few seconds, the Early Shift opener brings the
#      first stage shift to 15 s. synctest depth DEPTH (default 4) over 3600 frames: 0 mismatches, and every crest
#      named below fired (opener, tag in, tag out, shift) and an offered crest was taken;
#   2. a CPU run (MELEE_ENVOY_TEST_P1CPU=1: the player's fighter is a level-9 CPU, so KOs happen both ways) from fight 2:
#      0 mismatches, Softened fired at the opener and a KO-dealt or stock-lost crest fired.
#   GW_ISO_VANILLA=... bash tools/rust-port/envoy/m1_crests.sh   (OUT=dir for the logs; DEPTH=4)
set -u
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"; cd "$repo"
ws="${GW_WORKSPACE:-E:/Projects/Melee Workspace}"
if [ -z "${GW_ISO_VANILLA:-}" ] && [ -f "$ws/.env" ]; then set -a; . "$ws/.env"; set +a; fi
[ -n "${GW_ISO_VANILLA:-}" ] || { echo "m1_crests: GW_ISO_VANILLA not set"; exit 2; }
exe="$repo/build/cargo/release/melee"; [ -e "$exe" ] || exe="$exe.exe"
out="${OUT:-${TEMP:-/tmp}/m1_crests}"; mkdir -p "$out"
depth="${DEPTH:-4}"
fail=0
host=x86_64-pc-windows-msvc; case "$(uname -s)" in Linux*) host=x86_64-unknown-linux-gnu ;; esac
if (cd rust/mods/envoy_data && CARGO_TARGET_DIR="$repo/build/cargo-mods-host" RUST_TEST_THREADS=1 cargo test -j 4 -q --target "$host" >"$out/lint.log" 2>&1); then
  echo "ok   0. envoy_data lint + tests ($(grep -o '[0-9]* passed' "$out/lint.log" | head -1))"
else echo "FAIL 0. envoy_data (see $out/lint.log)"; fail=1; fi
mods="$repo/build/mods-m1test"; rm -rf "$mods"; mkdir -p "$mods"
cp "$repo/build/mods/envoy_sim.wasm" "$mods/" || { echo "m1_crests: build the mods first (tools/rust-port/modhost/build_mods.sh)"; exit 2; }
export MELEE_VOLUME=3 MELEE_WASM_MODS="$mods" MELEE_ENVOY_ARM=1 MELEE_ENVOY_START_FIGHT=2 MELEE_PAD_SCRIPT="$here/m1_pad.txt"
run() { # name crests [env...]
  local name=$1 crests=$2; shift 2
  env MELEE_WINDOW_TITLE="m1_crests $name" MELEE_ENVOY_TEST_CRESTS="$crests" "$@" "$exe" synctest --engine tu --iso "$GW_ISO_VANILLA" \
    --scenario pad_falcon_fox_fd --depth "$depth" --frames 3600 --timeout 1500 >"$out/$name.log" 2>&1
  grep 'engine synctest' "$out/$name.log"
}
fired() { grep -q "envoy: crest fire $2" "$out/$1.log"; }
line=$(run pad "hunter,early_shift,exposure/mend,ember")
miss=""; for k in early_shift mend hunter ember exposure; do fired pad $k || miss="$miss $k"; done
grep -q "envoy: crest taken" "$out/pad.log" || miss="$miss (no crest taken)"
tags=$(grep "envoy: tag " "$out/pad.log" | sort -u | wc -l); shifts=$(grep "envoy: moment shift" "$out/pad.log" | sort -u | wc -l)
case "$line" in *" 0 MISMATCHES"*) [ -z "$miss" ] && [ "$tags" -ge 2 ] && echo "ok   1. ${line#melee: }; $tags tags, $shifts shifts, $(grep 'crest fire' "$out/pad.log" | sort -u | wc -l) distinct fires" \
  || { echo "FAIL 1. missing:$miss tags $tags"; fail=1; } ;; *) echo "FAIL 1. ${line:-no result} (see $out/pad.log)"; fail=1 ;; esac
line=$(run cpu "trophy,spite,softened/rebirth,wildfire,last_stand" MELEE_ENVOY_TEST_P1CPU=1)
ko=0; for k in trophy spite rebirth; do fired cpu $k && ko=1; done
case "$line" in *" 0 MISMATCHES"*) fired cpu softened && [ $ko = 1 ] && echo "ok   2. ${line#melee: }; fired:$(grep -o 'crest fire [a-z_]*' "$out/cpu.log" | sort -u | cut -c12- | tr '\n' ' ')" \
  || { echo "FAIL 2. softened or a KO crest did not fire (see $out/cpu.log)"; fail=1; } ;; *) echo "FAIL 2. ${line:-no result} (see $out/cpu.log)"; fail=1 ;; esac
rm -rf "$mods"
[ $fail -eq 0 ] && echo "M1 CRESTS OK" || echo "M1 CRESTS FAILED (logs: $out)"
exit $fail
