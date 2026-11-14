#!/bin/bash
export MELEE_WINDOW_HIDDEN=${MELEE_WINDOW_HIDDEN:-1}  # agents' game windows are invisible (MELEE_WINDOW_HIDDEN=0 shows them)
# Envoy items, the first slice (docs/rust-port/notes/envoy-items-design.md 8): the items fire off their triggers, the
# item room / shop / boss drop / campfire hand out items and stocks, all deterministic under rollback.
#   0. envoy_data's tests (the item table's lint runs at compile time; the drive pool's lint still runs);
#   then, each as a synctest at every depth of DEPTHS (default "4 7") over FRAMES (default 3600) frames, 0 mismatches:
#   1. pad: items_pad.txt in fight 1 with all twelve items: short hop (Hop Shot), wavedash (Glide Wake), roll (Roll
#      Mines), a whiff (Whiff Echo), a tag to Fox and a shine (Shine Bell) fired off their own triggers; Moon Drops
#      orbit (published);
#   2. cpu: the player a level-9 CPU (MELEE_ENVOY_TEST_P1CPU) with all twelve: a hit item fired (Spark Ring on a nair
#      hit or Smash Quake on a smash hit), a KO paid Coin Magnet coins, Fire Lace burned;
#   3. room: fight 2 (the pack that ends the first session): the item room opens, its pedestal item is taken (walked
#      into or flown in), the shop opens and (knob `buystock`) a shop item is bought;
#   4. boss: fight 4 (the boss) with the stock limit 6 and 2 stocks (`lim:6,pool:2`): the boss drop's choice of two,
#      one taken, the campfire refills to 6 (`campfire, stocks refilled to 6`), the shop opens;
#   5. coop: MELEE_ENVOY_COOP=1, both players level-9 CPUs, fight 2: two item-room pedestals, both players take one;
#   6. petey: MELEE_ENVOY_BOSS=petey at fight 4: items hit Petey; the boss drop follows his defeat.
#   Fights 2 (envoy-items-design.md section 10):
#   7. gauntlet: fight 1 (the opening gauntlet), the player a level-9 CPU: the next duel comes in after the KO with a
#      new character (two different faces, no repeat), no results screen between;
#   8. live: the torture knob's stage switches as live transitions: the rift runs, the hand-off lands, the fight never
#      freezes, no KO caused by a switch;
#   9. crowd: fight 2 (the 1v3 crowd): every foe hit carries the crowd knockback (x1350 at three foes), no player hit.
#   GW_ISO_VANILLA=... bash tools/rust-port/envoy/items_check.sh   (EXE=path; ONLY="lint pad cpu room boss coop petey gauntlet live crowd";
#   OUT=dir for the logs; DEPTHS="4 7"; FRAMES=3600)
set -u
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"; cd "$repo"
ws="${GW_WORKSPACE:-E:/Projects/Melee Workspace}"
if [ -z "${GW_ISO_VANILLA:-}" ] && [ -f "$ws/.env" ]; then set -a; . "$ws/.env"; set +a; fi
[ -n "${GW_ISO_VANILLA:-}" ] || { echo "items_check: GW_ISO_VANILLA not set"; exit 2; }
. "$here/exe_check.sh"; exe_pick items_check
out="${OUT:-${TEMP:-/tmp}/items_check}"; mkdir -p "$out"
depths="${DEPTHS:-4 7}"; frames="${FRAMES:-3600}"; only="${ONLY:-lint pad cpu room boss coop petey gauntlet live crowd}"
fail=0
on() { case " $only " in *" $1 "*) return 0 ;; esac; return 1; }
cargo_locked() {
  if [ "${BUILDLOCK_HELD:-0}" = 1 ]; then cargo "$@"; return; fi
  bash "${BUILDLOCK:-E:/Projects/Melee Workspace/scratch/buildlock.sh}" run items1 cargo "$@"
}
host=x86_64-pc-windows-msvc; case "$(uname -s)" in Linux*) host=x86_64-unknown-linux-gnu ;; esac
if ! on lint; then :; elif (cd rust/mods/envoy_data && CARGO_TARGET_DIR="${HOST_TARGET:-$repo/build/cargo-mods-host}" RUST_TEST_THREADS=1 cargo_locked test -j 3 -q --target "$host" >"$out/lint.log" 2>&1); then
  echo "ok   0. envoy_data lint + tests ($(grep -o '[0-9]* passed' "$out/lint.log" | head -1))"
else echo "FAIL 0. envoy_data (see $out/lint.log)"; fail=1; fi
mods="${MODS_SRC:-$repo/build/mods}"
tmods="$out/mods"; rm -rf "$tmods"; mkdir -p "$tmods"
cp "$mods/envoy_sim.wasm" "$tmods/" || { echo "items_check: build the mods first (tools/rust-port/modhost/build_mods.sh)"; exit 2; }
ALL="spark_ring,glide_wake,shine_bell,hop_shot,ledge_spikes,whiff_echo,roll_mines,smash_quake,moon_drops,coin_magnet,homing_ink,fire_lace"
export MELEE_VOLUME=3 MELEE_WASM_MODS="$tmods" MELEE_ENVOY_ARM=1 MELEE_ENVOY_START_FIGHT=1 MELEE_PAD_SCRIPT="$here/items_pad.txt"
run() { # name depth knobs [env...]
  local name=$1 depth=$2 knobs=$3; shift 3
  exe_banner "items_check $name d$depth" >&2
  env MELEE_WINDOW_TITLE="${CHECK_TAG:-items1} agent: items_check $name d$depth (${CHECK_BRANCH:-port/envoy-items1})" MELEE_ENVOY_TEST_DRIVES="$knobs" "$@" timeout 2400 "$exe" synctest --engine tu --iso "$GW_ISO_VANILLA" \
    --scenario pad_falcon_fox_fd --depth "$depth" --frames "$frames" --timeout 2000 >"$out/$name.d$depth.log" 2>&1
  grep 'engine synctest' "$out/$name.d$depth.log"
}
L() { echo "$out/$1.d$2.log"; }
seen() { grep -aq "$3" "$(L "$1" "$2")"; }
fires() { grep -ao "envoy: item fires [a-z_]*" "$(L "$1" "$2")" | sort | uniq -c | awk '{printf "%s %s, ", $5, $1}'; }
check() { # n name depth line missing
  case "$4" in *" 0 MISMATCHES"*) [ -z "$5" ] && echo "ok   $1. $2 d$3: ${4#melee: }; fired: $(fires "$2" "$3")" \
    || { echo "FAIL $1. $2 d$3: missing:$5 (see $(L "$2" "$3"))"; fail=1; } ;; *) echo "FAIL $1. $2 d$3: ${4:-NO RESULT LINE (the run never finished)} (see $(L "$2" "$3"))"; fail=1 ;; esac
}
for d in $depths; do
  if on pad; then
    line=$(run pad "$d" "$ALL,noshop")
    miss=""
    for k in hop_shot glide_wake roll_mines whiff_echo shine_bell; do seen pad "$d" "envoy: item fires $k" || miss="$miss $k"; done
    seen pad "$d" "envoy: technique short hop apex" || miss="$miss (no short hop)"
    seen pad "$d" "envoy: quick tag" || miss="$miss (no tag)"
    check 1 pad "$d" "$line" "$miss"
  fi
  if on cpu; then
    line=$(run cpu "$d" "$ALL,noshop" MELEE_ENVOY_TEST_P1CPU=1)
    miss=""
    { seen cpu "$d" "envoy: item fires spark_ring" || seen cpu "$d" "envoy: item fires smash_quake"; } || miss="$miss (no hit item)"
    seen cpu "$d" "envoy: item coins coin_magnet" || miss="$miss (no Coin Magnet KO)"
    seen cpu "$d" "envoy: item nick moon_drops" || miss="$miss (no moon nick)"
    check 2 cpu "$d" "$line" "$miss"
  fi
  if on room; then
    line=$(frames=$(( frames < 9000 ? 9000 : frames )) run room "$d" "buystock,*4000" MELEE_ENVOY_TEST_P1CPU=1 MELEE_ENVOY_START_FIGHT=2)
    miss=""
    seen room "$d" "envoy: item room open" || miss="$miss (no item room)"
    seen room "$d" "envoy: item taken" || miss="$miss (no item taken)"
    seen room "$d" "envoy: shop open" || miss="$miss (no shop)"
    seen room "$d" "envoy: shop bought" || miss="$miss (no shop item bought)"
    check 3 room "$d" "$line" "$miss"
  fi
  if on boss; then
    line=$(frames=$(( frames < 6000 ? 6000 : frames )) run boss "$d" "lim:6,pool:2,*4000" MELEE_ENVOY_TEST_P1CPU=1 MELEE_ENVOY_START_FIGHT=4)
    miss=""
    seen boss "$d" "envoy: boss drop, choose one" || miss="$miss (no boss drop)"
    seen boss "$d" "envoy: item taken" || miss="$miss (no item taken)"
    seen boss "$d" "envoy: campfire, stocks refilled to.* 6$" || miss="$miss (campfire not to 6)"
    seen boss "$d" "envoy: shop open" || miss="$miss (no shop)"
    check 4 boss "$d" "$line" "$miss"
  fi
  if on coop; then
    line=$(frames=$(( frames < 9000 ? 9000 : frames )) run coop "$d" "*4000;*4000" MELEE_ENVOY_COOP=1 MELEE_PAD_CHANNELS=3 MELEE_ENVOY_TEST_P1CPU=1 MELEE_ENVOY_START_FIGHT=2)
    miss=""; seen coop "$d" "envoy: run start, players 2" || miss="$miss (not co-op)"
    grep -aqE "envoy: item taken [a-z_]+ 0?[0-9]$" "$(L coop "$d")" || miss="$miss (player 1 took no item)"
    grep -aqE "envoy: item taken [a-z_]+ 1[0-9][0-9]$" "$(L coop "$d")" || miss="$miss (player 2 took no item)"
    check 5 coop "$d" "$line" "$miss"
  fi
  if on petey; then
    line=$(frames=$(( frames < 6000 ? 6000 : frames )) run petey "$d" "$ALL,*4000" MELEE_ENVOY_TEST_P1CPU=1 MELEE_ENVOY_BOSS=petey MELEE_ENVOY_START_FIGHT=4)
    miss=""; seen petey "$d" "envoy: item fires" || miss="$miss (no item fired on Petey)"
    check 6 petey "$d" "$line" "$miss"
  fi
  if on gauntlet; then
    line=$(frames=$(( frames < 9000 ? 9000 : frames )) run gauntlet "$d" "*4000" MELEE_ENVOY_TEST_P1CPU=1 MELEE_ENVOY_START_FIGHT=1)
    miss=""; seen gauntlet "$d" "envoy: gauntlet, duels" || miss="$miss (not a gauntlet)"
    seen gauntlet "$d" "envoy: gauntlet duel, wave" || miss="$miss (no second duel)"
    seen gauntlet "$d" "envoy: wave new character" || miss="$miss (no new character)"
    seen gauntlet "$d" "refused; it comes back as itself" && miss="$miss (a repeat face)"
    # every duel's face differs from wave 1's and from each other (a rollback replays the same respawn: count distinct)
    w1=$(grep -aoE "room faces, wave 1 [0-9]+" "$(L gauntlet "$d")" | head -1 | grep -oE "[0-9]+$")
    duels=$(grep -aoE "gauntlet duel, wave\*100 \+ character [0-9]+" "$(L gauntlet "$d")" | grep -oE "[0-9]+$" | sort -u)
    faces=$( { [ -n "$w1" ] && echo $(( w1 / 10000 - 1 )); for x in $duels; do echo $(( x % 100 )); done; } | sort | uniq -d)
    [ "$(echo "$duels" | grep -c .)" = "$(echo "$duels" | awk '{print int($1/100)}' | sort -u | grep -c .)" ] || faces="$faces (a wave with two faces)"
    [ -n "$faces" ] && miss="$miss (repeated:$faces)"
    check 7 gauntlet "$d" "$line" "$miss"
  fi
  if on live; then
    line=$(run live "$d" "torture,*4000" MELEE_ENVOY_TEST_P1CPU=1 MELEE_ENVOY_START_FIGHT=1)
    miss=""; seen live "$d" "envoy: live stage transition begins" || miss="$miss (no live transition)"
    seen live "$d" "envoy: live stage hand-off, frame" || miss="$miss (no hand-off)"
    seen live "$d" "stage switch: live keep\|stage switch: safe place" || miss="$miss (no carry)"
    seen live "$d" "envoy: KO right after a stage switch" && miss="$miss (a switch caused a KO)"
    seen live "$d" "freeze kind 2" && miss="$miss (a freeze switch ran)"
    check 8 live "$d" "$line" "$miss"
  fi
  if on crowd; then
    line=$(frames=$(( frames < 6000 ? 6000 : frames )) run crowd "$d" "*4000" MELEE_ENVOY_TEST_P1CPU=1 MELEE_ENVOY_START_FIGHT=2)
    miss=""; seen crowd "$d" "envoy: crowd fight, foes\*10000 + knockback x1000 31350" || miss="$miss (not a 1v3 crowd at x1350)"
    seen crowd "$d" "envoy: crowd knockback, slot" || miss="$miss (no crowd knockback hit)"
    grep -aqE "envoy: crowd knockback, slot\*10000 \+ x1000 0?[0-9]{4}$" "$(L crowd "$d")" && miss="$miss (a player got it)"
    check 9 crowd "$d" "$line" "$miss"
  fi
done
rm -rf "$tmods"
[ $fail -eq 0 ] && echo "ITEMS OK" || echo "ITEMS FAILED (logs: $out)"
exit $fail
