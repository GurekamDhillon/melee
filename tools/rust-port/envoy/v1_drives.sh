#!/bin/bash
# Envoy V1 (the drive slice, envoy-core-v2.md 6): drives charge, fire, drop and level under rollback, deterministically.
#   0. envoy_data's lint and its tests (cargo test on the host target);
#   1. a pad run (tools/rust-port/envoy/v1_pad.txt) from fight 2 with test drives (MELEE_ENVOY_TEST_DRIVES; Echo
#      Oath given, so no keystone pick): A takes the first card, A its fighter; tags, jumps and jabs. synctest depth
#      DEPTH (default 4) over 3600 frames: 0 mismatches, the time drives (Bolt, Nova), Relay's volley, Comet (landings)
#      and an echo fired, a drive was gained from the card, a tag (Relay keys on it);
#   2. a CPU run (MELEE_ENVOY_TEST_P1CPU=1: the player's fighter is a level-9 CPU, so hits and KOs happen) from fight 2
#      with Plague Bearer and the hit drives (Ember, Frost, Shatter, Rush): 0 mismatches, a hit-charged drive fired,
#      and the first KO dropped a drive that was taken (walked over or swept up at the fight's end);
#   3. the shared knob (MELEE_ENVOY_SHARED_DRIVES=1) on the CPU run: 0 mismatches and the run logs the shared set.
#   Every run prints its fire count and the fires per minute of play (the loud-start band, envoy-core-v2.md 3.3).
#   GW_ISO_VANILLA=... bash tools/rust-port/envoy/v1_drives.sh   (OUT=dir for the logs; DEPTH=4)
set -u
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"; cd "$repo"
ws="${GW_WORKSPACE:-E:/Projects/Melee Workspace}"
if [ -z "${GW_ISO_VANILLA:-}" ] && [ -f "$ws/.env" ]; then set -a; . "$ws/.env"; set +a; fi
[ -n "${GW_ISO_VANILLA:-}" ] || { echo "v1_drives: GW_ISO_VANILLA not set"; exit 2; }
exe="$repo/build/cargo/release/melee"; [ -e "$exe" ] || exe="$exe.exe"
out="${OUT:-${TEMP:-/tmp}/v1_drives}"; mkdir -p "$out"
depth="${DEPTH:-4}"
fail=0
host=x86_64-pc-windows-msvc; case "$(uname -s)" in Linux*) host=x86_64-unknown-linux-gnu ;; esac
if (cd rust/mods/envoy_data && CARGO_TARGET_DIR="$repo/build/cargo-mods-host" RUST_TEST_THREADS=1 cargo test -j 4 -q --target "$host" >"$out/lint.log" 2>&1); then
  echo "ok   0. envoy_data lint + tests ($(grep -o '[0-9]* passed' "$out/lint.log" | head -1))"
else echo "FAIL 0. envoy_data (see $out/lint.log)"; fail=1; fi
mods="$repo/build/mods-v1test"; rm -rf "$mods"; mkdir -p "$mods"
cp "$repo/build/mods/envoy_sim.wasm" "$mods/" || { echo "v1_drives: build the mods first (tools/rust-port/modhost/build_mods.sh)"; exit 2; }
export MELEE_VOLUME=3 MELEE_WASM_MODS="$mods" MELEE_ENVOY_ARM=1 MELEE_ENVOY_START_FIGHT=2 MELEE_PAD_SCRIPT="$here/v1_pad.txt"
run() { # name drives [env...]
  local name=$1 drives=$2; shift 2
  env MELEE_WINDOW_TITLE="v1_drives $name" MELEE_ENVOY_TEST_DRIVES="$drives" "$@" "$exe" synctest --engine tu --iso "$GW_ISO_VANILLA" \
    --scenario pad_falcon_fox_fd --depth "$depth" --frames 3600 --timeout 1500 >"$out/$name.log" 2>&1
  grep 'engine synctest' "$out/$name.log"
}
fired() { grep -q "envoy: drive fire $2" "$out/$1.log"; }
# fires per minute of play: the distinct fire lines (a resimulated frame logs again; the sort keeps one)
rate() { local n; n=$(grep -c "envoy: drive \(fire\|echo\)" "$out/$1.log"); echo "$n fire lines"; }
line=$(run pad "bolt,nova,comet/relay,rush,ember:2|echo_oath")
miss=""; for k in bolt nova comet relay; do fired pad $k || miss="$miss $k"; done
grep -q "envoy: drive echo" "$out/pad.log" || miss="$miss (no echo)"
grep -q "envoy: drive \(gained\|level up\)" "$out/pad.log" || miss="$miss (no card taken)"
tags=$(grep "envoy: tag " "$out/pad.log" | sort -u | wc -l)
case "$line" in *" 0 MISMATCHES"*) [ -z "$miss" ] && [ "$tags" -ge 1 ] && echo "ok   1. ${line#melee: }; $tags tags; $(rate pad); fired:$(grep -o 'drive fire [a-z_]*' "$out/pad.log" | sort -u | cut -c12- | tr '\n' ' ')" \
  || { echo "FAIL 1. missing:$miss tags $tags (see $out/pad.log)"; fail=1; } ;; *) echo "FAIL 1. ${line:-no result} (see $out/pad.log)"; fail=1 ;; esac
line=$(run cpu "ember,frost,shatter/rush,bulwark,nova|plague_bearer" MELEE_ENVOY_TEST_P1CPU=1)
hit=0; for k in ember frost shatter; do fired cpu $k && hit=1; done
drop=0; grep -q "envoy: drop " "$out/cpu.log" && grep -q "envoy: drive \(gained\|level up\)" "$out/cpu.log" && drop=1
case "$line" in *" 0 MISMATCHES"*) [ $hit = 1 ] && [ $drop = 1 ] && echo "ok   2. ${line#melee: }; $(rate cpu); fired:$(grep -o 'drive fire [a-z_]*' "$out/cpu.log" | sort -u | cut -c12- | tr '\n' ' ')" \
  || { echo "FAIL 2. hit-drive fired $hit, drop taken $drop (see $out/cpu.log)"; fail=1; } ;; *) echo "FAIL 2. ${line:-no result} (see $out/cpu.log)"; fail=1 ;; esac
line=$(run shared "ember,frost,nova|overclock" MELEE_ENVOY_TEST_P1CPU=1 MELEE_ENVOY_SHARED_DRIVES=1)
case "$line" in *" 0 MISMATCHES"*) grep -q "envoy: drives shared by the pair" "$out/shared.log" && echo "ok   3. ${line#melee: }; shared set; $(rate shared)" \
  || { echo "FAIL 3. the shared knob was not seen (see $out/shared.log)"; fail=1; } ;; *) echo "FAIL 3. ${line:-no result} (see $out/shared.log)"; fail=1 ;; esac
rm -rf "$mods"
[ $fail -eq 0 ] && echo "V1 DRIVES OK" || echo "V1 DRIVES FAILED (logs: $out)"
exit $fail
