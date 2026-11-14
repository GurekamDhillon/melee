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
#      freezes, no KO caused by a switch; every fighter mapped (no spawn-point reset, no push past the mapped point beyond a
#      12-unit step up, or an 80-unit slide out of the new body), with the probe knob forcing a ledge-hang and an under-stage case;
#   9. crowd: fight 2 (the 1v3 crowd): every foe hit carries the crowd knockback (x1350 at three foes), no player hit;
#  22. shiftko: the torture knob's freeze switches on the gauntlet: no KO and no stock lost because of a switch.
#   Slice 2 (envoy-items-design.md section 11; every run carries the test knob `autotake`: the choices have no timer
#   for players, so the CPU-driven checks take the cursor's choice after a wait):
#   10. shop: the map's room choice steered to a shop (`forkshop`): the shop opens, an item and a stock are bought;
#   11. devil: the boss with the red door forced (`door:devil`, `devilbuy`): a deal taken, its stock slots paid;
#   12. angel: the boss with the white door forced: the angel room's item taken;
#   13. secret: the cracked wall in fight 1 beside player 1 (`secret:1`, `wallnear`): hit, broken, the secret room
#       after the fight, its item taken;
#   14. whiff: player 1 a CPU with Whiff Echo: a true whiff fires it, and an attack that ends on a shield is not one;
#   15. triggers: the new triggers seen (crouch cancel, powershield, a laser hit) with their items;
#   16. coopstock: co-op, the devil door: player 1 buys, only player 1's stock limit drops;
#   17. map: the act map: a fight node taken, the room choice opened and taken (the item room it names opens), the
#       next fight on the path; and knob `mapcheck` (rooms.rs map_self_check): every map of 24 seeds in loops 1-3
#       and every way through each, through the sim's own code: the graph (no dead end, no crossing, edges only to
#       neighbours on the next row, every node reachable, kinds by row), the stick walking only reachable lanes,
#       the view the UI draws (the cursor on the row being chosen, that row never marked skipped, the planned path
#       along real edges, the published edges), the node / room taken the cursor's, the fight's kind the node's,
#       save / continue keeping the path, no later view word overwriting the map;
#   25. mapcoop: the same in a co-op run (two players);
#   18. campfire: the room choice steered to a campfire: each player's stocks refilled to their own limit;
#   19. orbs: Moon Drops and the four stance orbs: orb hits, and an orb shattered by a foe's attack.
#   Synergy lane (envoy-synergy.md):
#   20. synergy: player 1 a CPU with the proc items, their synergy partners and two evolutions' recipes: procs fire,
#       a chain two or more deep, no proc deeper than 4, a synergy's first-trigger banner, the evolutions formed; the
#       proc sequence is the same at both depths (same seed, same result);
#   21. coopsyn: co-op, both CPUs, player 1 with the three co-op items: Kin Tether, Kindred Spark fire.
#   Pickup (11.11):
#   23. pickup: knob `pedfloor` puts an item-room pedestal on the floor; pickup_pad.txt walks player 1 past it with no
#       button (in range, then out of range, nothing taken), then back with A presses: taken by the button, and the
#       fighter in Melee's pickup motion (LightGet) instead of a jab;
#   24. air: pickup_air_z_pad.txt / pickup_air_l_pad.txt: knob `pedair` puts the pedestal under player 1 as a hop leaves the floor; Z / L taps in the air: taken
#       by the button from an airborne state (jumps, falls, EscapeAir).
#   GW_ISO_VANILLA=... bash tools/rust-port/envoy/items_check.sh   (EXE=path; ONLY="lint pad cpu room boss coop petey gauntlet live crowd
#   shop devil angel secret whiff triggers coopstock map mapcoop campfire orbs synergy coopsyn shiftko pickup air";
#   OUT=dir for the logs; DEPTHS="4 7"; FRAMES=3600)
set -u
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"; cd "$repo"
ws="${GW_WORKSPACE:-E:/Projects/Melee Workspace}"
if [ -z "${GW_ISO_VANILLA:-}" ] && [ -f "$ws/.env" ]; then set -a; . "$ws/.env"; set +a; fi
[ -n "${GW_ISO_VANILLA:-}" ] || { echo "items_check: GW_ISO_VANILLA not set"; exit 2; }
. "$here/exe_check.sh"; exe_pick items_check
out="${OUT:-${TEMP:-/tmp}/items_check}"; mkdir -p "$out"
depths="${DEPTHS:-4 7}"; frames="${FRAMES:-3600}"; only="${ONLY:-lint pad cpu room boss coop petey gauntlet live crowd shop devil angel secret whiff triggers coopstock map mapcoop campfire orbs synergy coopsyn shiftko pickup air}"
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
ORBS="moon_drops,dodge_orb,nair_ring,dash_orb,shield_orbs"
SYN="brimstone_jab,ember_seed:3,static_coil:3,frost_bud:3,fire_lace,spark_ring:3,chain_spark,roll_mines,smash_quake,shine_bell,hop_shot,homing_ink,big_bang,moon_drops,crimson_count"
COOPSYN="kin_tether:2,kindred_spark,rally_banner,spark_ring,smash_quake,roll_mines,*4000;ember_seed:2,static_coil,frost_bud,*4000"
export MELEE_VOLUME=3 MELEE_WASM_MODS="$tmods" MELEE_ENVOY_ARM=1 MELEE_ENVOY_START_FIGHT=1 MELEE_PAD_SCRIPT="$here/items_pad.txt"
run() { # name depth knobs [env...]
  local name=$1 depth=$2 knobs=$3; shift 3
  exe_banner "items_check $name d$depth" >&2
  knobs="$knobs,autotake"
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
    seen cpu "$d" "envoy: orb hit moon_drops" || miss="$miss (no moon hit)"
    check 2 cpu "$d" "$line" "$miss"
  fi
  if on room; then
    line=$(frames=$(( frames < 9000 ? 9000 : frames )) run room "$d" "forkitem,*4000" MELEE_ENVOY_TEST_P1CPU=1 MELEE_ENVOY_START_FIGHT=2)
    miss=""
    seen room "$d" "envoy: map room choice open" || miss="$miss (no room choice)"
    seen room "$d" "envoy: item room open" || miss="$miss (no item room)"
    seen room "$d" "envoy: item taken" || miss="$miss (no item taken)"
    check 3 room "$d" "$line" "$miss"
  fi
  if on boss; then
    line=$(frames=$(( frames < 6000 ? 6000 : frames )) run boss "$d" "nodoor,*4000" MELEE_ENVOY_TEST_P1CPU=1 MELEE_ENVOY_START_FIGHT=4)
    miss=""
    seen boss "$d" "envoy: boss drop, choose one" || miss="$miss (no boss drop)"
    seen boss "$d" "envoy: item taken" || miss="$miss (no item taken)"
    seen boss "$d" "envoy: loop now" || miss="$miss (no next loop)"
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
    line=$(frames=$(( frames < 9000 ? 9000 : frames )) run gauntlet "$d" "duelnode,*4000" MELEE_ENVOY_TEST_P1CPU=1 MELEE_ENVOY_START_FIGHT=1)
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
    line=$(run live "$d" "duelnode,torture,*4000" MELEE_ENVOY_TEST_P1CPU=1 MELEE_ENVOY_START_FIGHT=1 MELEE_ENVOY_LIVE_PROBE=1)
    miss=""; seen live "$d" "envoy: live stage transition begins" || miss="$miss (no live transition)"
    seen live "$d" "envoy: live stage hand-off, frame" || miss="$miss (no hand-off)"
    seen live "$d" "stage switch: live map" || miss="$miss (no mapped carry)"
    # nobody is reset to a spawn point; nobody moves more than LIVE_PUSH (40) past the mapped point; both fallbacks ran
    seen live "$d" "stage switch: safe place" && miss="$miss (a spawn-point reset)"
    # a mapped point over the floor moves at most a step up (12); one inside the new body slides outward at most 80
    big=$(grep -aoE "stage switch: live map .* push [0-9.]+" "$(L live "$d")" | awk '{ lim = ($0 ~ /case (under|hang|ledge)/) ? 80 : 12; if ($NF + 0 > lim) n++ } END { print n + 0 }')
    [ "$big" = 0 ] || miss="$miss ($big pushes past the limit beyond the mapped point)"
    seen live "$d" "case scale\|case ledge" || miss="$miss (no plain mapping)"
    seen live "$d" "case hang" || miss="$miss (no ledge-hang mapping)"
    seen live "$d" "case under" || miss="$miss (no under-stage mapping)"
    seen live "$d" "envoy: KO right after a stage switch" && miss="$miss (a switch caused a KO)"
    seen live "$d" "stock lost soon after a stage switch" && miss="$miss (a stock lost to a switch)"
    seen live "$d" "freeze kind 2" && miss="$miss (a freeze switch ran)"
    check 8 live "$d" "$line" "$miss"
  fi
  if on crowd; then
    line=$(frames=$(( frames < 6000 ? 6000 : frames )) run crowd "$d" "crowdnode,*4000" MELEE_ENVOY_TEST_P1CPU=1 MELEE_ENVOY_START_FIGHT=2)
    miss=""; seen crowd "$d" "envoy: crowd fight, foes\*10000 + knockback x1000 31350" || miss="$miss (not a 1v3 crowd at x1350)"
    seen crowd "$d" "envoy: crowd knockback, slot" || miss="$miss (no crowd knockback hit)"
    grep -aqE "envoy: crowd knockback, slot\*10000 \+ x1000 0?[0-9]{4}$" "$(L crowd "$d")" && miss="$miss (a player got it)"
    check 9 crowd "$d" "$line" "$miss"
  fi
  if on shiftko; then
    # regression (candidate9 torture): the freeze switch on the gauntlet re-placed a CPU whose AI still chased its old
    # target spot and it flew off; the CPU now forgets its target at every re-place, so no switch costs a stock
    line=$(frames=$(( frames < 6000 ? 6000 : frames )) run shiftko "$d" "torture,freezeshift,*4000" MELEE_ENVOY_TEST_P1CPU=1 MELEE_ENVOY_START_FIGHT=1)
    miss=""; seen shiftko "$d" "stage switch: safe place" || miss="$miss (no freeze switch ran)"
    seen shiftko "$d" "envoy: KO right after a stage switch" && miss="$miss (a switch caused a KO)"
    seen shiftko "$d" "stock lost soon after a stage switch" && miss="$miss (a stock lost to a switch)"
    check 22 shiftko "$d" "$line" "$miss"
  fi
  if on shop; then
    line=$(frames=$(( frames < 9000 ? 9000 : frames )) run shop "$d" "forkshop,buystock,slotsale,*4000" MELEE_ENVOY_TEST_P1CPU=1 MELEE_ENVOY_START_FIGHT=2)
    miss=""; seen shop "$d" "envoy: map room kind (MN) 7$" || miss="$miss (the shop not taken on the map)"
    seen shop "$d" "envoy: shop open" || miss="$miss (no shop)"
    seen shop "$d" "envoy: shop bought" || miss="$miss (no shop item bought)"
    seen shop "$d" "envoy: shop has a stock slot" || miss="$miss (no stock slot on sale)"
    check 10 shop "$d" "$line" "$miss"
  fi
  if on devil; then
    line=$(frames=$(( frames < 6000 ? 6000 : frames )) run devil "$d" "door:devil,devilbuy,lim:5,pool:5,*4000" MELEE_ENVOY_TEST_P1CPU=1 MELEE_ENVOY_START_FIGHT=4)
    miss=""; seen devil "$d" "envoy: the red door opens" || miss="$miss (no red door)"
    seen devil "$d" "envoy: devil deal open" || miss="$miss (no deal)"
    seen devil "$d" "envoy: devil deal taken" || miss="$miss (no deal taken)"
    grep -aqE "envoy: stock slots paid, player\*100 \+ limit now [34]$" "$(L devil "$d")" || miss="$miss (the limit did not drop from 5)"
    check 11 devil "$d" "$line" "$miss"
  fi
  if on angel; then
    line=$(frames=$(( frames < 6000 ? 6000 : frames )) run angel "$d" "door:angel,*4000" MELEE_ENVOY_TEST_P1CPU=1 MELEE_ENVOY_START_FIGHT=4)
    miss=""; seen angel "$d" "envoy: the white door opens" || miss="$miss (no white door)"
    seen angel "$d" "envoy: angel room open" || miss="$miss (no angel room)"
    seen angel "$d" "envoy: angel room pedestal" || miss="$miss (no angel item)"
    [ "$(grep -ac 'envoy: item taken' "$(L angel "$d")")" -ge 2 ] || miss="$miss (the angel item was not taken)"
    check 12 angel "$d" "$line" "$miss"
  fi
  if on secret; then
    line=$(frames=$(( frames < 9000 ? 9000 : frames )) run secret "$d" "secret:1,wallnear,*4000" MELEE_ENVOY_TEST_P1CPU=1 MELEE_ENVOY_START_FIGHT=1)
    miss=""; seen secret "$d" "envoy: cracked wall stands" || miss="$miss (no wall)"
    seen secret "$d" "envoy: cracked wall hit" || miss="$miss (the wall never hit)"
    seen secret "$d" "envoy: secret wall broken" || miss="$miss (the wall never broke)"
    seen secret "$d" "envoy: secret room open" || miss="$miss (no secret room)"
    seen secret "$d" "envoy: secret room pedestal" || miss="$miss (no secret item)"
    check 13 secret "$d" "$line" "$miss"
  fi
  if on whiff; then
    line=$(frames=$(( frames < 6000 ? 6000 : frames )) run whiff "$d" "whiff_echo,*4000" MELEE_ENVOY_TEST_P1CPU=1)
    miss=""; seen whiff "$d" "envoy: technique whiff" || miss="$miss (no whiff)"
    seen whiff "$d" "envoy: item fires whiff_echo" || miss="$miss (Whiff Echo never fired)"
    seen whiff "$d" "envoy: attack ended on a shield, not a whiff" || miss="$miss (no shielded attack told apart)"
    check 14 whiff "$d" "$line" "$miss"
  fi
  if on triggers; then
    line=$(frames=$(( frames < 9000 ? 9000 : frames )) run triggers "$d" "laser_prism,crouch_thorns,perfect_guard,brimstone_jab,bloodlust,*4000" MELEE_ENVOY_TEST_P1CPU=1)
    miss=""
    # crouch cancel, powershield, jab and laser hits depend on what the CPUs do: reported, not required (a pad script
    # that forces each is a follow-up); Bloodlust (taking a hit) is required
    for k in "crouch cancel" "powershield" "jab hit" "laser hit"; do seen triggers "$d" "envoy: technique $k" && echo "     15. triggers d$d: seen $k" || echo "     15. triggers d$d: not seen $k (CPU-dependent)"; done
    seen triggers "$d" "envoy: item fires bloodlust" || miss="$miss (Bloodlust never fired)"
    check 15 triggers "$d" "$line" "$miss"
  fi
  if on coopstock; then
    line=$(frames=$(( frames < 6000 ? 6000 : frames )) run coopstock "$d" "door:devil,devilbuy,*4000;*4000" MELEE_ENVOY_COOP=1 MELEE_PAD_CHANNELS=3 MELEE_ENVOY_TEST_P1CPU=1 MELEE_ENVOY_START_FIGHT=4)
    miss=""; seen coopstock "$d" "envoy: run start, players 2" || miss="$miss (not co-op)"
    seen coopstock "$d" "envoy: devil deal taken" || miss="$miss (no deal)"
    grep -aqE "envoy: stock limits now, p1\*100 \+ p2 [23]04$" "$(L coopstock "$d")" || miss="$miss (the deal did not cost only player 1)"
    check 16 coopstock "$d" "$line" "$miss"
  fi
  if on map; then
    line=$(frames=$(( frames < 9000 ? 9000 : frames )) run map "$d" "mapcheck,*4000" MELEE_ENVOY_TEST_P1CPU=1 MELEE_ENVOY_START_FIGHT=1)
    miss=""; seen map "$d" "envoy: map_check OK" || miss="$miss (map_check: $(grep -aE 'map_check (FAIL|first)' "$(L map "$d")" | head -3 | tr '
' ';'))"
    seen map "$d" "envoy: map node taken, row\*10 + lane [0-2]$" || miss="$miss (no fight node taken)"
    seen map "$d" "envoy: map room choice open, row 1$" || miss="$miss (no room choice)"
    seen map "$d" "envoy: map room taken, row\*10 + lane 1[0-2]$" || miss="$miss (no room taken)"
    seen map "$d" "envoy: item room open" || miss="$miss (the item room taken on the map never opened)"
    check 17 map "$d" "$line" "$miss"
  fi
  if on mapcoop; then
    # the act map in co-op (two players, P2's wish flag): the same map_check, both players CPUs
    line=$(frames=$(( frames < 9000 ? 9000 : frames )) run mapcoop "$d" "mapcheck,*4000;*4000" MELEE_ENVOY_COOP=1 MELEE_PAD_CHANNELS=3 MELEE_ENVOY_TEST_P1CPU=1 MELEE_ENVOY_START_FIGHT=1)
    miss=""; seen mapcoop "$d" "envoy: map_check OK" || miss="$miss (map_check: $(grep -aE 'map_check (FAIL|first)' "$(L mapcoop "$d")" | head -3 | tr '
' ';'))"
    seen mapcoop "$d" "envoy: run start, players 2" || miss="$miss (not a co-op run)"
    seen mapcoop "$d" "envoy: map room taken, row\*10 + lane 1[0-2]$" || miss="$miss (no room taken)"
    check 25 mapcoop "$d" "$line" "$miss"
  fi
  if on campfire; then
    line=$(frames=$(( frames < 9000 ? 9000 : frames )) run campfire "$d" "campfire,lim:6,pool:2,*4000" MELEE_ENVOY_TEST_P1CPU=1 MELEE_ENVOY_START_FIGHT=2)
    miss=""; seen campfire "$d" "envoy: map room kind (MN) 8$" || miss="$miss (no campfire taken)"
    seen campfire "$d" "envoy: campfire, player\*100 + stocks refilled to 6$" || miss="$miss (not refilled to 6)"
    check 18 campfire "$d" "$line" "$miss"
  fi
  if on orbs; then
    line=$(frames=$(( frames < 6000 ? 6000 : frames )) run orbs "$d" "$ORBS,*4000" MELEE_ENVOY_TEST_P1CPU=1)
    miss=""; seen orbs "$d" "envoy: orb hit moon_drops" || miss="$miss (no moon hit)"
    seen orbs "$d" "envoy: orb shattered by an attack" || miss="$miss (no shatter)"
    stance=$(grep -aoE "envoy: orb hit (dodge_orb|nair_ring|dash_orb|shield_orbs)" "$(L orbs "$d")" | sort -u | awk '{print $4}' | tr '\n' ' ')
    blocks=$(grep -ac "envoy: orb blocked a projectile" "$(L orbs "$d")")
    check 19 orbs "$d" "$line" "$miss"
    echo "     orbs d$d: stance orbs that hit: ${stance:-none}; projectiles blocked: $blocks"
  fi
  if on synergy; then
    line=$(frames=$(( frames < 6000 ? 6000 : frames )) run synergy "$d" "$SYN,*4000" MELEE_ENVOY_TEST_P1CPU=1 MELEE_ENVOY_START_FIGHT=2)
    miss=""; seen synergy "$d" "envoy: item procs" || miss="$miss (no proc)"
    seen synergy "$d" "envoy: proc chain" || miss="$miss (no chain past depth 1)"
    grep -aqE "envoy: item procs [a-z_]+ -?[0-9]*[5-9][0-9]{3}$" "$(L synergy "$d")" && miss="$miss (a proc deeper than 4)"
    seen synergy "$d" "envoy: synergy first" || miss="$miss (no synergy banner)"
    seen synergy "$d" "envoy: evolution formed storm_crown" || miss="$miss (Storm Crown not formed)"
    seen synergy "$d" "envoy: evolution formed inferno_seed" || miss="$miss (Inferno Seed not formed)"
    grep -ao "envoy: proc at .*" "$(L synergy "$d")" | sort -u > "$out/synergy.procs.d$d"
    check 20 synergy "$d" "$line" "$miss"
    echo "     synergy d$d: procs $(wc -l < "$out/synergy.procs.d$d"), chains $(grep -ac 'envoy: proc chain' "$(L synergy "$d")"), synergies first: $(grep -aoE 'envoy: synergy first [a-z_]+' "$(L synergy "$d")" | awk '{print $4}' | sort -u | tr '
' ' ')"
  fi
  if on coopsyn; then
    line=$(frames=$(( frames < 9000 ? 9000 : frames )) run coopsyn "$d" "$COOPSYN" MELEE_ENVOY_COOP=1 MELEE_PAD_CHANNELS=3 MELEE_ENVOY_TEST_P1CPU=1 MELEE_ENVOY_START_FIGHT=2)
    miss=""; seen coopsyn "$d" "envoy: run start, players 2" || miss="$miss (not co-op)"
    seen coopsyn "$d" "envoy: item fires kin_tether\|envoy: item fires kindred_spark" || miss="$miss (no co-op item fired)"
    check 21 coopsyn "$d" "$line" "$miss"
    echo "     coopsyn d$d: tether $(grep -ac 'item fires kin_tether' "$(L coopsyn "$d")"), kindred $(grep -ac 'item fires kindred_spark' "$(L coopsyn "$d")"), rally $(grep -ac 'item fires rally_banner' "$(L coopsyn "$d")")"
  fi
  if on pickup; then
    line=$(run pickup "$d" "pedfloor,noshop" MELEE_PAD_SCRIPT="$here/pickup_pad.txt")
    f="$(L pickup "$d")"; miss=""
    seen pickup "$d" "envoy: item room open" || miss="$miss (no pedestal)"
    # first in range, then out again with the item still there (walking past takes nothing), then the A press takes it
    a=$(grep -an "envoy: pickup range entered" "$f" | head -1 | cut -d: -f1)
    b=$(grep -an "envoy: pickup range left, item untaken" "$f" | head -1 | cut -d: -f1)
    c=$(grep -an "envoy: pickup by button" "$f" | head -1 | cut -d: -f1)
    [ -n "$a" ] || miss="$miss (never in range)"
    [ -n "$b" ] && [ -n "$a" ] && [ "$b" -gt "$a" ] || miss="$miss (walking past never left the item behind)"
    [ -n "$c" ] && [ -n "$b" ] && [ "$c" -gt "$b" ] || miss="$miss (no take by the button after the walk past)"
    seen pickup "$d" "envoy: item taken" || miss="$miss (no item taken)"
    seen pickup "$d" "envoy: pickup motion (LightGet)" || miss="$miss (no pickup motion: the A press jabbed)"
    check 23 pickup "$d" "$line" "$miss"
  fi
  if on air; then
    for b in z l; do
      line=$(run "air$b" "$d" "pedair,noshop" MELEE_PAD_SCRIPT="$here/pickup_air_${b}_pad.txt")
      miss=""
      grep -aqE "envoy: pickup by button, state (2[5-9]|3[0-4]|236)$" "$(L "air$b" "$d")" || miss="$miss (no airborne take with ${b})"
      seen "air$b" "$d" "envoy: item taken" || miss="$miss (no item taken)"
      check 24 "air$b" "$d" "$line" "$miss"
    done
  fi
done
if on synergy; then
  set -- $depths
  if [ $# -ge 2 ] && [ -s "$out/synergy.procs.d$1" ]; then
    if cmp -s "$out/synergy.procs.d$1" "$out/synergy.procs.d$2"; then echo "ok   20b. synergy: the frame-tagged proc set is identical at d$1 and d$2 ($(wc -l < "$out/synergy.procs.d$1") procs)"
    else echo "FAIL 20b. synergy: the frame-tagged proc set differs between d$1 and d$2"; fail=1; fi
  fi
fi
rm -rf "$tmods"
[ $fail -eq 0 ] && echo "ITEMS OK" || echo "ITEMS FAILED (logs: $out)"
exit $fail
