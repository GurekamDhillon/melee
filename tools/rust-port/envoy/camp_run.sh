#!/bin/bash
export MELEE_WINDOW_HIDDEN=${MELEE_WINDOW_HIDDEN:-1}  # agents' game windows are invisible (MELEE_WINDOW_HIDDEN=0 shows them)
# Envoy from the camp door, fights 1 -> 3 (the run must never fall out to plain VS between fights).
#
# Boots the game with no MELEE_ENVOY_ARM (the camp arms the run), walks title > main menu > the Envoy row (V3: the
# main menu's sixth row, Down twice) > A > the camp's Start (Start run) with a pad script
# (camp_pad.txt; MELEE_ENVOY_TEST_P1CPU makes the player's fighters level-9 CPUs so the fights are won headless),
# and checks the log: every VS match is a rewritten Envoy match, fights 1, 2 and 3 are fought and won, the run is
# never disarmed or over before fight 3.
#
#   EXE=build/cargo/release/melee.exe MODS=build/mods-camp OUT=... bash tools/rust-port/envoy/camp_run.sh
# PAD: the pad script (default camp_pad.txt; camp_pad_ace.txt with GW_ISO_VANILLA pointed at the ACE disc).
# MAX_FIGHT (default 3): the fights that must be fought and won (the last one only fought).
# CARD: a memory card folder with a save (copied). GW_ISO_VANILLA must be set (never printed). Needs envoy_sim.wasm
# and envoy_ui.wasm (tools/rust-port/modhost/build_mods.sh) and the Brawl files (_build/brawl above the exe or the
# working directory, or MELEE_BRAWL_LOCAL): without them the camp refuses to arm.
set -u
here=$(cd "$(dirname "$0")" && pwd)
repo=$(cd "$here/../../.." && pwd)
exe=${EXE:-$repo/build/cargo/release/melee.exe}
out=${OUT:-$repo/build/envoy-camp}
frames=${FRAMES:-45000}
mods=${MODS:-}
if [ -z "$mods" ]; then
  mods="$repo/build/mods-camp"; rm -rf "$mods"; mkdir -p "$mods"
  cp "$repo/build/mods/envoy_sim.wasm" "$repo/build/mods/envoy_ui.wasm" "$mods/" || { echo "camp_run: build the mods first"; exit 2; }
  # the UI's art: <mods>/envoy_ui/ (the host looks for a mod's textures in <mods>/<mod name>/)
  cp -r "$repo/rust/mods/envoy_ui/art" "$mods/envoy_ui"
fi
[ -n "${GW_ISO_VANILLA:-}" ] || { echo "camp_run: GW_ISO_VANILLA is not set"; exit 2; }
rm -rf "$out"; mkdir -p "$out/save"
# a copy of a memory card with a save on it (the menus start at the title, not the memcard prompts): CARD, else the
# exe's own card folder; the copy keeps the run from writing the real card
card=${CARD:-$(dirname "$exe")/card}
[ -d "$card/USA" ] || { echo "camp_run: no memory card at CARD (a folder with USA/Card A)"; exit 2; }
cp -r "$card" "$out/card"
log="$out/camp.log"
# ACE=1: the m-ex disc (GW_ISO_ACE) boots straight to the title, so the pad script loses its second boot Start
iso=$GW_ISO_VANILLA; pad="$here/camp_pad.txt"
if [ "${ACE:-0}" = 1 ]; then
  [ -n "${GW_ISO_ACE:-}" ] || { echo "camp_run: ACE=1 needs GW_ISO_ACE"; exit 2; }
  . "$here/exe_check.sh"; iso=$GW_ISO_ACE; pad=$(pad_for_disc "$pad" "$out")
fi
unset MELEE_ENVOY_ARM MELEE_ENVOY_START_FIGHT MELEE_ENVOY_COOP
# slice 2: rooms wait for the player (no timers); the pad script does not walk to pedestals or pick rooms, so the
# test knob takes the cursor's choice after a wait, and the first room choice goes to the shop
export MELEE_ENVOY_TEST_DRIVES=autotake,forkshop
env MELEE_VOLUME=3 MELEE_WINDOW_TITLE="${CAMP_TITLE:-v3sim agent: camp_run, Envoy camp door, fights 1-3 (port/v3sim)}" MELEE_WASM_MODS="$mods" \
  MELEE_ENVOY_DIR="$out/save" MELEE_CARD_PATH="$out/card" MELEE_PAD_SCRIPT="${PAD:-$pad}" MELEE_PAD_CHANNELS=03 MELEE_ENVOY_TEST_P1CPU=1 \
  MELEE_TURBO=1 ${CAMP_ENV:-} \
  "$exe" play --iso "$iso" --headless --no-audio --max-frames "$frames" >"$log" 2>&1
fail=0
ok() { echo "ok   $*"; }
bad() { echo "FAIL $*"; fail=1; }
# the order of what happened: VS matches, rewrites, wins, disarms, run over
seq=$(grep -oE "scene vs \(|envoy: match [0-9]+ rewritten|envoy: fight won, next [0-9]+|envoy: disarmed|envoy: RUN OVER|envoy: armed" "$log" \
  | sed -E 's/scene vs \(/VS/; s/envoy: match ([0-9]+) rewritten/R\1/; s/envoy: fight won, next ([0-9]+)/W\1/; s/envoy: disarmed/D/; s/envoy: RUN OVER/X/; s/envoy: armed/A/' | tr '\n' ' ')
echo "     events: $seq"
grep -q "envoy: armed" "$log" && ok "the camp armed the run (no MELEE_ENVOY_ARM)" || bad "the camp never armed the run"
last=${MAX_FIGHT:-3}
for n in $(seq 1 "$last"); do
  grep -q "envoy: match $n rewritten" "$log" && ok "fight $n is an Envoy match" || bad "fight $n never started as an Envoy match"
done
for n in $(seq 2 "$last"); do
  # the boss (fight 4) wraps the loop: its win is the campfire that follows it, not a "next 5"
  if [ "$n" = 5 ]; then grep -q "envoy: campfire, stocks refilled" "$log" && ok "fight 4 (the boss) won" || bad "fight 4 (the boss) not won"; continue; fi
  grep -q "envoy: fight won, next $n" "$log" && ok "fight $((n - 1)) won" || bad "fight $((n - 1)) not won"
done
# items (envoy-items-design.md 8): the item room after the session's last fight (fight 2) gives an item, the shop opens;
# with MAX_FIGHT >= 5 the boss (fight 4) drops a choice of two and the campfire refills the stocks
if [ "$last" -ge 3 ]; then
  grep -q "envoy: item room open" "$log" && ok "the item room opened" || bad "no item room"
  grep -q "envoy: item taken" "$log" && ok "$(grep -c "envoy: item taken" "$log") item(s) taken" || bad "no item taken"
  grep -q "envoy: shop open" "$log" && ok "the shop opened" || bad "no shop"
fi
if [ "$last" -ge 5 ]; then
  grep -q "envoy: boss drop, choose one" "$log" && ok "the boss dropped a choice of two" || bad "no boss drop"
  grep -q "envoy: campfire, stocks refilled to" "$log" && ok "$(grep -m1 -o "campfire, stocks refilled to.*" "$log")" || bad "no campfire"
fi
# every VS match up to the third rewrite is a rewritten one (a plain VS match would be a VS with no R before it)
prefix=${seq%%R$last*}
if [ "$prefix" != "$seq" ]; then
  plain=$(echo "$prefix R$last VS" | grep -oE "R[0-9]+ VS|VS" | grep -c "^VS$")
  [ "$plain" = 0 ] && ok "no plain VS match before fight 3" || bad "$plain plain VS match(es) before fight 3"
  case "$prefix" in *D*|*X*) bad "the run was disarmed or over before fight 3";; *) ok "armed throughout";; esac
fi
[ $fail = 0 ] && echo "CAMP RUN OK" || echo "CAMP RUN FAILED (see $log)"
exit $fail
