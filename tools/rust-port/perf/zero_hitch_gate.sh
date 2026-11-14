#!/bin/bash
export MELEE_WINDOW_HIDDEN=${MELEE_WINDOW_HIDDEN:-1}  # agents' game windows are invisible (MELEE_WINDOW_HIDDEN=0 shows them)
# Zero-hitch gate (docs/rust-port/notes/zero-hitch.md): fixed scripted scenarios in REAL TIME (no turbo, windowed,
# vsync on) with the frame profiler on; FAILS if any frame interval or tick is over 16.7 ms, and prints each offender
# with its cause (zero_hitch_check.py).
#
#   EXE=build/cargo/release/melee.exe MODS=build/mods OUT=build/zerohitch bash tools/rust-port/perf/zero_hitch_gate.sh [scenario...]
#
# Scenarios (default: all): vanilla_bf (title -> menus -> VS, 4 CPUs on Battlefield, results -> CSS -> second match),
# vanilla_fod (a CPU match on Fountain of Dreams through --match, results), envoy (camp -> picker -> fights 1-5 with
# tagauto, wave2, a Shifting road, drops, picks, shop; the boss if reached), and the _nointerp / _60 variants:
# INTERP=off, FPS=60 apply to every scenario of the run (e.g. INTERP=off bash ... envoy).
# The gate does NOT take buildlock/runlight itself: wrap it (heavy lock AND the light slot, machine quiet):
#   buildlock.sh run port/zerolag runlight.sh zerolag bash tools/rust-port/perf/zero_hitch_gate.sh
# Each scenario is killed after GATE_TIMEOUT seconds (default 540) so a slow run cannot hold the locks; run the
# scenarios as separate short holds (one scenario per lock hold) on a shared machine.
# GW_ISO_VANILLA must be set (never printed). CARD: a memory card folder with a save (default: the exe's card/).
set -u
here=$(cd "$(dirname "$0")" && pwd)
repo=$(cd "$here/../../.." && pwd)
exe=${EXE:-$repo/build/cargo/release/melee.exe}
mods_src=${MODS:-$repo/build/mods}
out=${OUT:-$repo/build/zerohitch}
build=${BUILD_LABEL:-$(git -C "$repo" rev-parse --short HEAD 2>/dev/null)}
[ -n "${GW_ISO_VANILLA:-}" ] || { echo "gate: GW_ISO_VANILLA is not set"; exit 2; }
card=${CARD:-$(dirname "$exe")/card}
[ -d "$card/USA" ] || { echo "gate: no memory card at CARD (a folder with USA/Card A)"; exit 2; }
scen=("$@"); [ ${#scen[@]} -gt 0 ] || scen=(vanilla_bf vanilla_fod envoy)
mkdir -p "$out"
# nothing else may run: other melee/cargo/rustc/link processes make the numbers meaningless
busy=$(powershell.exe -NoProfile -Command "(Get-Process melee*,cargo,rustc,link -ErrorAction SilentlyContinue | Measure-Object).Count" | tr -d '\r')
[ "${busy:-0}" = 0 ] || { echo "gate: $busy melee/cargo/rustc/link process(es) running: the machine is not quiet"; [ -n "${GATE_ALLOW_BUSY:-}" ] || exit 3; }

fail=0
run() {  # name, frames, env..., -- , args...
  local name=$1 frames=$2; shift 2
  local envs=() ; while [ "$1" != "--" ]; do envs+=("$1"); shift; done; shift
  local d="$out/$name"; rm -rf "$d"; mkdir -p "$d/bin" "$d/save"
  cp "$exe" "$d/bin/melee.exe"; cp -r "$card" "$d/card"
  local m="$d/mods"; mkdir -p "$m"
  [ -n "${NEED_MODS:-}" ] && cp "$mods_src/envoy_sim.wasm" "$mods_src/envoy_ui.wasm" "$m/" && cp -r "$repo/rust/mods/envoy_ui/art" "$m/envoy_ui"
  echo "-- $name (${frames} ticks, real time)"
  ( cd "$d/bin" && env MELEE_VOLUME=3 MELEE_PROFILE=1 MELEE_FULLSCREEN=off MELEE_RECLAIM_LEAK=1 \
      MELEE_WINDOW_TITLE="zerolag agent: gate $name ($build)" MELEE_CARD_PATH="$d/card" MELEE_ENVOY_DIR="$d/save" \
      MELEE_WASM_MODS="$m" ${INTERP:+MELEE_INTERP=$INTERP} ${FPS:+MELEE_FPS_CAP=$FPS} "${envs[@]}" \
      timeout "${GATE_TIMEOUT:-540}" ./melee.exe play --iso "$GW_ISO_VANILLA" --max-frames "$frames" "$@" >"$d/run.log" 2>&1 )
  local p; p=$(ls -t "$d/bin/profiles/"profile-*.csv 2>/dev/null | head -1)
  [ -n "$p" ] || { echo "FAIL $name: no profile written (see $d/run.log)"; fail=1; return; }
  python "$here/zero_hitch_check.py" "$p" --label "$name" || fail=1
}

for s in "${scen[@]}"; do
  case "$s" in
    vanilla_bf)  NEED_MODS= run vanilla_bf 9000 MELEE_PAD_SCRIPT="$repo/tools/rust-port/play/retail_full.txt" -- --skip-intro --time-secs 30;;
    vanilla_fod) NEED_MODS= run vanilla_fod 3000 -- --match "fox/cpu9,falco/cpu9,FOUNTAIN OF DREAMS" --time-secs 30 --quit-at-results;;
    envoy)       NEED_MODS=1 run envoy ${ENVOY_TICKS:-9000} MELEE_PAD_SCRIPT="$repo/tools/rust-port/envoy/camp_pad.txt" MELEE_PAD_CHANNELS=03 \
                   MELEE_ENVOY_TEST_P1CPU=1 MELEE_ENVOY_BOSS=petey "MELEE_ENVOY_TEST_DRIVES=ember,bolt,tagauto,wave2,shift,dropall/ember,bolt" --;;
    *) echo "gate: unknown scenario $s"; fail=1;;
  esac
done
[ $fail = 0 ] && echo "ZERO-HITCH GATE OK" || echo "ZERO-HITCH GATE FAILED"
exit $fail
