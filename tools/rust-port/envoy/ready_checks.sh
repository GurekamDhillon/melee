#!/bin/bash
# Envoy readiness checks (docs/rust-port/notes/envoy-readiness.md): one scripted check per fix of the readiness
# audit. Each row FAILS on the build before its fix and passes after it. Headless, turbo, in-engine pads only.
#
#   EXE=build/cargo/release/melee.exe MODS=build/mods OUT=... [CARD=...] bash tools/rust-port/envoy/ready_checks.sh [row...]
# Rows (default: all that can run):
#   quit    pause + L+R+A+Start in room 1 keeps the run (no RUN OVER); the next boot continues it at that room
#   seed    MELEE_ENVOY_SEED=1 / 2 / 1 from the camp: runs 1 and 3 roll the same, run 2 differs
#   fresh   two New runs with no pinned seed (MELEE_ENVOY_SALT_LIVE=1): different salts and different room-1 foes
#   acecss  the ACE disc (GW_ISO_ACE): the retail VS character select opens and a match starts (no panic)
# GW_ISO_VANILLA must be set (never printed); acecss is skipped without GW_ISO_ACE. CARD: a memory card folder with
# a save (default: the exe's card/). MODS: a folder with envoy_sim.wasm, envoy_ui.wasm and envoy_ui/ (the art).
# The Brawl files must be findable (MELEE_BRAWL_LOCAL or _build/brawl above the working directory).
set -u
here=$(cd "$(dirname "$0")" && pwd)
repo=$(cd "$here/../../.." && pwd)
. "$here/exe_check.sh"; exe_pick ready_checks "$repo/build/cargo/release/melee"
mods=${MODS:-$repo/build/mods}
out=${OUT:-$repo/build/envoy-ready}
card=${CARD:-$(dirname "$exe")/card}
[ -n "${GW_ISO_VANILLA:-}" ] || { echo "ready_checks: GW_ISO_VANILLA is not set"; exit 2; }
[ -d "$card/USA" ] || { echo "ready_checks: no memory card at CARD (a folder with USA/Card A)"; exit 2; }
[ -f "$mods/envoy_sim.wasm" ] || { echo "ready_checks: no envoy_sim.wasm in MODS"; exit 2; }
command -v python >/dev/null 2>&1 || python() { python3 "$@"; }
rows=("$@"); [ ${#rows[@]} -gt 0 ] || rows=(quit seed fresh acecss)
mkdir -p "$out"
exe_banner ready_checks
fail=0
ok() { echo "ok   $*"; }
bad() { echo "FAIL $*"; fail=1; }
# play NAME FRAMES PAD ISO KEEP [ENV...] [-- ARGS...]: one headless boot in $out/NAME (KEEP=1 keeps its save)
play() {
  local name=$1 frames=$2 pad=$3 iso=$4 keep=$5; shift 5
  local envs=(); while [ $# -gt 0 ] && [ "$1" != "--" ]; do envs+=("$1"); shift; done; [ $# -gt 0 ] && shift
  local d="$out/$name"
  [ "$keep" = 1 ] || rm -rf "$d"
  mkdir -p "$d/save"; [ -d "$d/card" ] || cp -r "$card" "$d/card"
  (cd "$d" && env -u MELEE_ENVOY_ARM -u MELEE_ENVOY_START_FIGHT -u MELEE_ENVOY_COOP -u MELEE_ENVOY_SEED MELEE_ENVOY_TEST_DRIVES=autotake \
    MELEE_VOLUME=3 MELEE_FULLSCREEN=off MELEE_WINDOW_TITLE="ready_checks $name (${BRANCH_TAG:-envoy})" MELEE_WASM_MODS="$mods" \
    MELEE_ENVOY_DIR="$d/save" MELEE_CARD_PATH="$d/card" MELEE_PAD_SCRIPT="$pad" MELEE_PAD_CHANNELS=03 MELEE_TURBO=1 \
    MELEE_CRASH_STACK=200 MELEE_RECLAIM_LEAK=1 MELEE_HANG_DUMP=60 MELEE_PAD_IGNORE_ADAPTER=1 "${envs[@]}" \
    timeout 900 "$exe" play --iso "$iso" --headless --no-audio --max-frames "$frames" "$@" >"$d/run.log" 2>&1)
  echo "     $name: exit $?"
}
pads="$out/pads"; mkdir -p "$pads"
for r in "${rows[@]}"; do
  case "$r" in
  quit)
    # A through the keystone pick, then Start (pause) and L+R+A+Start (the retail quit) in room 1, then Start / A
    mid=$(python -c "print(';'.join(['3 0100 0 0;40 0000 0 0']*30)+';3 1000 0 0;30 0000 0 0;10 1160 0 0;300 0000 0 0')")
    python "$here/ready_pad.py" "$pads/quit.txt" 10 21 --nopick --mid "$mid" --segs 600
    python "$here/ready_pad.py" "$pads/def.txt" 10 21 --nopick
    play quit 3600 "$pads/quit.txt" "$GW_ISO_VANILLA" 0
    l="$out/quit/run.log"
    grep -q "game pause by player 0" "$l" && ok "quit: the fight was paused" || bad "quit: the pad never paused the fight (see $l)"
    grep -q "envoy: fight left from the pause" "$l" && ok "quit: the sim took the quit as a quit" || bad "quit: no 'fight left from the pause' (see $l)"
    grep -q "envoy: RUN OVER" "$l" && bad "quit: the quit ended the run (RUN OVER)" || ok "quit: no RUN OVER"
    [ -f "$out/quit/save/run.bin" ] && ok "quit: the run is still saved" || bad "quit: no saved run after the quit"
    cp "$l" "$out/quit/run-1.log"
    play quit 1500 "$pads/def.txt" "$GW_ISO_VANILLA" 1 MELEE_ENVOY_TEST_P1CPU=1 MELEE_PAD_NO_PAUSE=1
    grep -q "envoy: armed, run continued at loop\*10 + fight.* 11" "$l" && ok "quit: the next boot continues the run at room 1" || bad "quit: the next boot did not continue at room 1 (see $l)"
    grep -q "envoy: match 1 rewritten" "$l" && ok "quit: the continued room starts" || bad "quit: the continued room never started"
    ;;
  seed)
    python "$here/ready_pad.py" "$pads/def.txt" 10 21 --nopick
    sig() { grep -a -oE "envoy: (drop [a-z_]+|keystone taken [a-z_]+|drive gained [a-z_]+|wave new character, slot [0-9] as [0-9]+)|rewritten: .* foes .*" "$out/$1/run.log" | head -40 | cksum | cut -d' ' -f1; }
    for k in a:1 b:2 c:1; do
      play "seed-${k%%:*}" 14000 "$pads/def.txt" "$GW_ISO_VANILLA" 0 MELEE_ENVOY_TEST_P1CPU=1 MELEE_PAD_NO_PAUSE=1 MELEE_ENVOY_SEED="${k##*:}"
    done
    for k in a b c; do grep -q "envoy: fight won, next 2" "$out/seed-$k/run.log" || bad "seed: run $k never cleared room 1 (see $out/seed-$k/run.log)"; done
    a=$(sig seed-a); b=$(sig seed-b); c=$(sig seed-c)
    [ "$a" = "$c" ] && ok "seed: the same seed rolls the same run ($a)" || bad "seed: seed 1 twice rolled differently ($a vs $c)"
    [ "$a" != "$b" ] && ok "seed: another seed rolls another run ($a vs $b)" || bad "seed: seeds 1 and 2 rolled the same run ($a): the run seed is fixed"
    ;;
  fresh)
    # two consecutive New runs with no pinned seed (the clock salt, as a player gets it): different salts, different
    # room-1 foes (the owner saw every run identical before the salt)
    python "$here/ready_pad.py" "$pads/def.txt" 10 21 --nopick
    for k in a b; do play "fresh-$k" 3000 "$pads/def.txt" "$GW_ISO_VANILLA" 0 MELEE_ENVOY_TEST_P1CPU=1 MELEE_PAD_NO_PAUSE=1 MELEE_ENVOY_SALT_LIVE=1; done
    sa=$(grep -aoE "envoy: new run salt [0-9]+" "$out/fresh-a/run.log" | head -1 | grep -oE "[0-9]+$"); sb=$(grep -aoE "envoy: new run salt [0-9]+" "$out/fresh-b/run.log" | head -1 | grep -oE "[0-9]+$")
    fa=$(grep -aoE "rewritten: .* foes \[[0-9, ]*\]" "$out/fresh-a/run.log" | head -1 | grep -oE "foes .*"); fb=$(grep -aoE "rewritten: .* foes \[[0-9, ]*\]" "$out/fresh-b/run.log" | head -1 | grep -oE "foes .*")
    [ -n "$sa" ] && [ -n "$sb" ] && [ "$sa" != "$sb" ] && [ "$sa" != 0 ] && ok "fresh: two New runs rolled different salts ($sa, $sb)" || bad "fresh: salts '$sa' and '$sb' (want two different non-zero)"
    [ -n "$fa" ] && [ "$fa" != "$fb" ] && ok "fresh: room 1 foes differ ($fa / $fb)" || bad "fresh: room 1 foes '$fa' and '$fb' (want different)"
    ;;
  acecss)
    if [ -z "${GW_ISO_ACE:-}" ]; then echo "skip acecss: GW_ISO_ACE is not set"; continue; fi
    play acecss 4000 "$repo/tools/rust-port/play/retail_full.txt" "$GW_ISO_ACE" 0 -- --skip-intro --time-secs 30
    l="$out/acecss/run.log"
    grep -q "panicked\|crash report" "$l" && bad "acecss: the game stopped ($(grep -m1 -o 'archive symbol .[A-Za-z_]*. has no type\|panicked at [^:]*' "$l"))" || ok "acecss: no panic"
    grep -q "scene vs" "$l" && ok "acecss: a VS match started from the character select" || bad "acecss: no VS match (see $l)"
    ;;
  *) echo "ready_checks: unknown row $r"; fail=1;;
  esac
done
[ $fail = 0 ] && echo "READY CHECKS OK" || echo "READY CHECKS FAILED (logs: $out)"
exit $fail
