#!/bin/bash
export MELEE_WINDOW_HIDDEN=${MELEE_WINDOW_HIDDEN:-1}  # agents' game windows are invisible (MELEE_WINDOW_HIDDEN=0 shows them)
# The Envoy UI aspect check (docs/rust-port/notes/ui-remake.md): every Envoy UI surface is put on screen by a
# scripted run with MELEE_UI_ASPECT=1, and the host's audit (melee_modhost ui/cmd.rs `Audit`) says what was drawn
# out of proportion or on top of something else:
#   * a sprite whose drawn width / height is more than 2% off its art's;
#   * a round shape that is not round (sides 2% to 25% apart with fully round ends);
#   * a line cut at its width (the host never scales text sideways: a line that does not fit is cut, never squeezed);
#   * a line that runs past the side of the sticker it starts on, off the screen, or over another line;
#   * two sprites of one layer that cross (icons packed closer than they are wide).
# A run fails on any `ui_aspect: .. BAD` line that ui_aspect_allow.txt does not list, and when its surface never
# showed (the log lines under `need`). The screenshots of every run stay in OUT/<config>/<scenario>/shots.
# Scenarios (each a hidden `play` run; the fights start straight in the armed match, MELEE_ENVOY_ARM):
#   pause    the fight HUD, then the pause overlay: every page (Items with a card, Run, Fight, Options with the
#            abandon question, Words scrolled, Combos with an entry)
#   lab      the menu's Item Lab cell, the Lab setup, then every page of the Lab panel, empty and with every item
#   camp     the Envoy camp: its rows, the fighter picker, How to play, the road and the map before fight 1
#   room     fight 2 to its item room: the room choice, the pedestal banner, the new-item card
#   shop     the room choice steered to the shop: the counter, an item's card, the messages
#   devil / angel   the boss with the red / white door forced: the door callout, the deal list, the room banner
#   secret   the cracked wall broken in fight 1: the callout, the secret room
#   synergy  the proc items and their partners: the synergy and evolution banners
#   coop     two players: both corners, both stock rows, the second pedestal
#   pickup   the pedestal on the floor, taken with A: the prompt
#   run      the camp door to the end of a lost run: the road between fights, the results, the run-end summary
#   GW_ISO_VANILLA=... EXE=... MODS=<folder with envoy_sim.wasm, envoy_ui.wasm, envoy_ui/> CARD=<card files> OUT=...
#   bash ui_aspect.sh   [ONLY="pause lab ..."] [CONFIGS="ws720 ws1080 43-720 43-1080"] (default: ws720 43-720)
set -u
here=$(cd "$(dirname "$0")" && pwd)
repo=$(cd "$here/../../.." && pwd)
. "$here/exe_check.sh"; exe_pick ui_aspect
mods=${MODS:?MODS (a folder with envoy_sim.wasm, envoy_ui.wasm and envoy_ui/)}
out=${OUT:-$repo/build/ui-aspect}
only=${ONLY:-pause lab camp room shop devil angel secret synergy coop pickup run}
configs=${CONFIGS:-ws720 43-720}
[ -n "${GW_ISO_VANILLA:-}" ] || { echo "ui_aspect: GW_ISO_VANILLA is not set"; exit 2; }
allow="$here/ui_aspect_allow.txt"
fail=0
every() { python -c "print(','.join(str(x) for x in range($1,$2,$3)))"; }
SYN="brimstone_jab,ember_seed:3,static_coil:3,frost_bud:3,fire_lace,spark_ring:3,chain_spark,roll_mines,smash_quake,shine_bell,hop_shot,homing_ink,big_bang,moon_drops,crimson_count"
run() { # config scenario
  local c=$1 s=$2 size ws
  case "$c" in ws720) size=1280x720; ws=1 ;; ws1080) size=1920x1080; ws=1 ;; 43-720) size=960x720; ws=0 ;; 43-1080) size=1440x1080; ws=0 ;;
    *) echo "ui_aspect: unknown config $c"; exit 2 ;; esac
  local d="$out/$c/$s"
  rm -rf "$d"; mkdir -p "$d/shots" "$d/save" "$d/card/USA/Card A"
  cp -r "${CARD:?CARD (the test card files: copied to <card>/USA/Card A)}"/. "$d/card/USA/Card A/"
  local frames=4200 pad="$here/items_pad.txt" shots="" need=""
  local args=(--match falcon,fox,32)
  local envs=(MELEE_ENVOY_ARM=1 MELEE_ENVOY_START_FIGHT=1 MELEE_ENVOY_TEST_P1CPU=1 MELEE_TURBO=1 MELEE_PAD_NO_PAUSE=1)
  marks() { python "$here/ui_aspect_pad.py" "$1" "$d/pad.txt" "$d/marks.txt"; pad="$d/pad.txt"
    frames=$(( $(head -1 "$d/pad.txt" | grep -oE "[0-9]+ frames" | grep -oE "[0-9]+") + 80 ))
    shots=$(awk '{printf "%s%s",(NR>1?",":""),$2}' "$d/marks.txt"); }
  case "$s" in
  pause) marks pause; need="envoy pause: page Run|envoy pause: page Fight|envoy pause: page Options|envoy pause: abandon asked|envoy pause: page Words|envoy pause: page Combos"
    envs=(MELEE_ENVOY_ARM=1 MELEE_ENVOY_START_FIGHT=1 "MELEE_ENVOY_TEST_DRIVES=spark_ring,hop_shot,hop_shot,moon_drops,coin_magnet,fire_lace,ember_seed,static_coil,frost_bud,brimstone_jab,chain_spark,big_bang,noshop") ;;
  lab) marks lab; args=(); envs=(MELEE_PAD_CHANNELS=01)
    need="envoy lab: setup open|envoy lab: panel open|envoy lab: page Loadout|envoy lab: page Triggers|envoy lab: page Effects|envoy lab: page Sandbox|envoy lab: page Synergy" ;;
  camp) marks camp; args=(); envs=(MELEE_PAD_CHANNELS=01); need="envoy: armed" ;;
  room)    shots=$(every 150 4200 75); need="envoy: map room choice open|envoy: item room open|envoy: item taken"
    envs+=(MELEE_ENVOY_START_FIGHT=2 "MELEE_ENVOY_TEST_DRIVES=forkitem,autotake,*4000") ;;
  shop)    shots=$(every 150 4200 75); need="envoy: shop open|envoy: shop bought"
    envs+=(MELEE_ENVOY_START_FIGHT=2 "MELEE_ENVOY_TEST_DRIVES=forkshop,buystock,slotsale,autotake,spark_ring,hop_shot:2,*4000") ;;
  devil)   shots=$(every 150 4200 75); need="envoy: devil deal open|envoy: devil deal taken"
    envs+=(MELEE_ENVOY_START_FIGHT=4 "MELEE_ENVOY_TEST_DRIVES=door:devil,devilbuy,lim:5,pool:5,autotake,*4000") ;;
  angel)   shots=$(every 150 4200 75); need="envoy: angel room"
    envs+=(MELEE_ENVOY_START_FIGHT=4 "MELEE_ENVOY_TEST_DRIVES=door:angel,autotake,*4000") ;;
  secret)  shots=$(every 150 4200 75); need="envoy: secret"
    envs+=("MELEE_ENVOY_TEST_DRIVES=secret:1,wallnear,autotake,*4000") ;;
  synergy) frames=3000; shots=$(every 150 3000 60); need="envoy: evolution formed"
    envs+=(MELEE_ENVOY_START_FIGHT=2 "MELEE_ENVOY_TEST_DRIVES=$SYN,autotake,*4000") ;;
  coop)    shots=$(every 150 4200 75); need="envoy: item room open"
    envs+=(MELEE_ENVOY_COOP=1 MELEE_PAD_CHANNELS=3 MELEE_ENVOY_START_FIGHT=2 "MELEE_ENVOY_TEST_DRIVES=forkitem,autotake,spark_ring,hop_shot:2,*4000;moon_drops,*4000") ;;
  pickup)  frames=1500; shots=$(every 150 1500 30); pad="$here/pickup_pad.txt"; need="envoy: item taken"
    envs=(MELEE_ENVOY_ARM=1 MELEE_ENVOY_START_FIGHT=1 "MELEE_ENVOY_TEST_DRIVES=pedfloor,noshop") ;;
  run)     frames=${RUN_FRAMES:-9000}; args=(--shot-ages "20,60,150,260,420,700,1100"); pad="$here/camp_pad.txt"; need="envoy: RUN OVER"
    envs=(MELEE_PAD_CHANNELS=03 MELEE_TURBO=1 MELEE_PAD_NO_PAUSE=1 "MELEE_ENVOY_TEST_DRIVES=lim:1,pool:1,autotake") ;;
  *) echo "ui_aspect: unknown scenario $s"; exit 2 ;;
  esac
  (cd "$d" && env MELEE_VOLUME=3 MELEE_WINDOW_TITLE="${CHECK_TAG:-uiremake} agent: ui_aspect $s $c (${CHECK_BRANCH:-port/uiremake})" \
    MELEE_WASM_MODS="$mods" MELEE_ENVOY_DIR="$d/save" MELEE_CARD_PATH="$d/card" MELEE_PAD_SCRIPT="$pad" MELEE_PAD_IGNORE_ADAPTER=1 \
    MELEE_WIDESCREEN=$ws MELEE_FULLSCREEN=off MELEE_WINDOW="$size" MELEE_UI_LOG=1 MELEE_UI_ASPECT=1 MELEE_RECLAIM_LEAK=1 \
    ${shots:+MELEE_RETAIL_SHOT_TICKS="$shots"} "${envs[@]}" \
    timeout ${TMO:-900} "$exe" play --iso "$GW_ISO_VANILLA" --max-frames "$frames" --shots --shot-dir "$d/shots" "${args[@]}" >"$d/run.log" 2>&1)
  local rc=$? miss="" lines bad
  grep -a "ui_aspect:" "$d/run.log" | sed 's/.*ui_aspect: //' | sort -u > "$d/aspect.txt"
  lines=$(wc -l < "$d/aspect.txt")
  # the allow list: one pattern (a fixed string) per line, `#` comments
  grep " BAD$" "$d/aspect.txt" | { if [ -f "$allow" ]; then grep -v -F -f <(grep -v '^#' "$allow" | sed '/^$/d'); else cat; fi; } > "$d/bad.txt"
  bad=$(wc -l < "$d/bad.txt")
  local IFS='|'; for n in $need; do grep -aq -- "$n" "$d/run.log" || miss="$miss ($n)"; done; unset IFS
  [ "$lines" -ge 3 ] || miss="$miss (the audit saw nothing: no UI drawn?)"
  grep -aqiE "panicked|wasm trap|gd_ui_submit refused" "$d/run.log" && miss="$miss (a trap or a refused buffer in the log)"
  if [ "$bad" = 0 ] && [ -z "$miss" ]; then echo "ok   $c $s: $lines draws audited, in proportion ($(ls "$d/shots" | wc -l) shots, exit $rc)"
  else echo "FAIL $c $s: $bad out of proportion or overlapping${miss:+; missing:$miss} (see $d/bad.txt, $d/run.log)"; sed 's/^/       /' "$d/bad.txt" | head -12; fail=1; fi
}
exe_banner ui_aspect
for c in $configs; do for s in $only; do run "$c" "$s"; done; done
exit $fail
