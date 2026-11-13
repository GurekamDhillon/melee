#!/bin/bash
# Subspace enemies (docs/rust-port/notes/envoy-subspace.md): one scripted check per enemy kind.
# Each row is a synctest of an enemy room (the sim's `~N` test fight: three of one kind, no foe fighters) with a
# scripted player (down smashes), and passes when: three spawned, an enemy's attack hit the player, the player's hits
# landed, every enemy was beaten, the room was won, and the synctest reports 0 mismatches.
#   bash tools/rust-port/envoy/subspace_check.sh            (EXE, MODS, OUT; DISC=vanilla|ace; KINDS; DEPTHS; FRAMES)
# DISC=ace uses GW_ISO_ACE and an m-ex fighter (character kind MAIN, default 34: the first added slot) as the player.
# No disc path is ever printed.
set -u
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"; cd "$repo"
ws="${GW_WORKSPACE:-E:/Projects/Melee Workspace}"
if [ -z "${GW_ISO_VANILLA:-}" ] && [ -f "$ws/.env" ]; then set -a; . "$ws/.env"; set +a; fi
disc="${DISC:-vanilla}"
if [ "$disc" = ace ]; then iso="${GW_ISO_ACE:-}"; else iso="${GW_ISO_VANILLA:-}"; fi
[ -n "$iso" ] || { echo "subspace_check: the $disc disc is not configured"; exit 2; }
. "$here/exe_check.sh"; exe_pick subspace_check
out="${OUT:-${TEMP:-/tmp}/subspace_check}/$disc"; mkdir -p "$out"
srcmods="${MODS:-$repo/build/mods}"
mods="$out/mods"; rm -rf "$mods"; mkdir -p "$mods"
cp "$srcmods/envoy_sim.wasm" "$mods/" || { echo "subspace_check: envoy_sim.wasm not found in $srcmods (build the mods first)"; exit 2; }
depths="${DEPTHS:-4}"; frames="${FRAMES:-3000}"
names=(goomba primid bill giant)
kinds="${KINDS:-goomba primid bill giant}"
fail=0
export MELEE_VOLUME=3 MELEE_FULLSCREEN=off MELEE_WASM_MODS="$mods" MELEE_ENVOY_ARM=1 MELEE_ENVOY_START_FIGHT=1 MELEE_SUBSPACE_LOG=1
export MELEE_PAD_SCRIPT="${PAD:-$here/subspace_pad.txt}"
[ "$disc" = ace ] && export MELEE_ENVOY_TEST_MAIN="${MAIN:-34}"
exe_banner "subspace_check ($disc)"
for d in $depths; do
  for k in 0 1 2 3; do
    name=${names[$k]}
    case " $kinds " in *" $name "*) ;; *) continue ;; esac
    log="$out/$name.d$d.log"
    MELEE_WINDOW_TITLE="subspace1 check $name $disc d$d" MELEE_ENVOY_TEST_DRIVES="~$((100 + k + 32 * ${TIER:-0})),noshop" \
      timeout 2400 "$exe" synctest --engine tu --iso "$iso" --scenario pad_falcon_fox_fd --depth "$d" --frames "$frames" --timeout 1800 >"$log" 2>&1
    line=$(grep -a 'engine synctest' "$log" | tail -1)
    c() { grep -a -c "$1" "$log"; }
    spawned=$(c 'subspace: .* spawned'); atk=$(c 'subspace: .* attack [0-9]* hits slot'); got=$(c 'subspace: .* hit by slot\|subspace: .* stomped by')
    gone=$(c 'envoy: enemy gone, status'); won=$(c 'envoy: fight won'); main=$(grep -a -o 'rewritten: main [0-9]*' "$log" | head -1)
    miss=""
    [ "$spawned" -ge 3 ] || miss="$miss (spawned $spawned of 3)"
    [ "$atk" -ge 1 ] || miss="$miss (no enemy attack landed)"
    # Bullet Bills reach the player before the Ready prompt lets them act and are spent on the hit (or fly off), so
    # a player hit is not required for them; every other kind must be hit.
    [ "$got" -ge 1 ] || [ "$name" = bill ] || miss="$miss (no player hit landed)"
    [ "$gone" -ge 3 ] || miss="$miss (beaten $gone of 3)"
    [ "$won" -ge 1 ] || miss="$miss (room not won)"
    grep -a -q 'panicked\|ARENA MISUSE\|crash' "$log" && miss="$miss (panic / crash line in the log)"
    [ "$disc" = ace ] && [ "$main" != "rewritten: main ${MELEE_ENVOY_TEST_MAIN}" ] && miss="$miss (player is not the m-ex fighter: $main)"
    case "$line" in
      *" 0 MISMATCHES"*) if [ -z "$miss" ]; then echo "ok   $name $disc d$d: ${line#melee: }; spawned $spawned, enemy hits $atk, player hits $got, beaten $gone, $main"
                         else echo "FAIL $name $disc d$d: missing:$miss (see $log)"; fail=1; fi ;;
      *) echo "FAIL $name $disc d$d: ${line:-NO RESULT LINE (the run never finished)}$miss (see $log)"; fail=1 ;;
    esac
  done
done
[ $fail = 0 ] && echo "SUBSPACE CHECK OK ($disc)" || echo "SUBSPACE CHECK FAILED ($disc)"
exit $fail
