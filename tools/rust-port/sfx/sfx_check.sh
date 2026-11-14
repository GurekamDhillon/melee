#!/bin/bash
# SFX parity check (port/sfxfix): every sound the game asks for in the live play path keys a voice with sample data.
# Rows (hidden, headless retail flow, MELEE_SFX_LOG request log, see melee_audio sound.rs):
#   vs4    Fox / Falco / Samus / Link CPUs, items very high, Battlefield (projectiles + items)
#   envoy  an Envoy fight with Quick Tag every 150 frames (partner, foes, waves; needs envoy_sim.wasm in MODS)
# FAIL when any request is refused (badid / nofree / paused / nosem) or any key-on finds no sample data (`nodata`: the
# sound's SSM bank was never loaded, the Nov-13 bug: tag partners and Envoy wave characters). `novoice` (all 64
# voices busy) is reported, not failed. Envoy row: the tag partner's character must be in the match bank load.
#   EXE=... MODS=dir-with-envoy_sim.wasm [OUT=dir] bash tools/rust-port/sfx/sfx_check.sh
set -u
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"
ws="${GW_WORKSPACE:-E:/Projects/Melee Workspace}"
if [ -z "${GW_ISO_VANILLA:-}" ] && [ -f "$ws/.env" ]; then set -a; . "$ws/.env"; set +a; fi
[ -n "${GW_ISO_VANILLA:-}" ] || { echo "sfx_check: GW_ISO_VANILLA not set"; exit 2; }
exe="${EXE:-$repo/build/cargo/release/melee.exe}"
out="${OUT:-${TEMP:-/tmp}/sfx_check}"; rm -rf "$out"; mkdir -p "$out"
w() { if command -v cygpath >/dev/null; then cygpath -w "$1"; else echo "$1"; fi; }
fail=0
run() { # name frames match [env...]
  local name=$1 frames=$2 match=$3; shift 3
  env MELEE_WINDOW_HIDDEN=1 MELEE_VOLUME=3 MELEE_WINDOW_TITLE="sfx_check $name" MELEE_SFX_LOG="$(w "$out/$name.sfx")" MELEE_AUDIO_DBG=1 "$@" \
    timeout 900 "$exe" play --iso "$GW_ISO_VANILLA" --match "$match" --items "very high" --headless --max-frames "$frames" >"$out/$name.log" 2>&1
  local s; s=$(python "$here/sfxlog_summary.py" "$out/$name.sfx" 2>&1)
  local starts; starts=$(echo "$s" | head -1)
  if [ ! -s "$out/$name.sfx" ]; then echo "FAIL sfx $name: no request log (see $out/$name.log)"; fail=1; return 1; fi
  if echo "$starts" | grep -q "'nodata'\|'badid'\|'nofree'\|'paused'\|'nosem'"; then
    echo "FAIL sfx $name: $starts"; echo "$s" | sed -n '3,12p'; fail=1; return 1
  fi
  echo "ok   sfx $name: $starts"
}
run vs4 3000 fox/cpu9,falco/cpu9,samus/cpu9,link/cpu9,31
mods="${MODS:-$repo/build/mods}"
if [ -f "$mods/envoy_sim.wasm" ]; then
  m="$out/mods"; mkdir -p "$m"; cp "$mods/envoy_sim.wasm" "$m/"
  if run envoy 3000 fox,falco/cpu9,32 MELEE_WASM_MODS="$(w "$m")" MELEE_ENVOY_ARM=1 MELEE_ENVOY_START_FIGHT=2 MELEE_ENVOY_TEST_DRIVES=tagauto \
      MELEE_PAD_SCRIPT="$(w "$repo/tools/rust-port/envoy/drives_pad.txt")"; then
    banks=$(grep -a -m1 'match banks ckinds' "$out/envoy.log")
    tags=$(grep -a -c 'quick tag' "$out/envoy.log")
    case "$banks" in *" 20"*) echo "ok   sfx envoy: partner bank loaded ($banks), $tags tags" ;;
      *) echo "FAIL sfx envoy: the partner (Falco, 20) is not in the bank load: $banks"; fail=1 ;; esac
  fi
else echo "skip sfx envoy: no envoy_sim.wasm in $mods"; fi
[ $fail = 0 ] && echo "SFX CHECK OK" || echo "SFX CHECK FAILED"
exit $fail
