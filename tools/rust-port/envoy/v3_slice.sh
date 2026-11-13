#!/bin/bash
# Envoy V3 (Heat, envoy-synergy-v3.md 12.10; docs/rust-port/notes/envoy-v3-rules.md): the slice's checks.
#   A. envoy_data's lint and tests (the split test: one record against 2 / 7 / 30, bit-identical);
#   B. sim.py's V3 measures (--v3; exit 1 on a missed band);
#   C. synctests at every depth of DEPTHS (default "4 7") over FRAMES (default 3600), 0 mismatches each:
#      1 chain: a chain of links with Echo (CPU player)        2 cash: a cash-out, the flare, Heat halved (a pack)
#      3 states: Heat states up and down, a stock lost          4 tech: technique marks, offstage damage, +Heat drives
#      5 volley: Volley on D-pad right (pad)                     6 shop: a full-row pick, a buy, a level, a reroll, a
#                                                                  reorder, a full row replaced (pad, fight 2)
#      7 finale: Grand Finale A/B launch ratio               8 coop: two meters, cash-outs, one purse
#      9 petey: the Petey boss with breaks          10 loop3: a run begun in loop 3 (endless loops)
#     11 loopboss: the boss falls in loop 2, the run goes on into loop 3
#   then V2's six (v2_drives.sh) on the new rules.
#   GW_ISO_VANILLA=... bash tools/rust-port/envoy/v3_slice.sh   (OUT=dir; DEPTHS; FRAMES; ONLY="chain cash ...")
set -u
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"; cd "$repo"
ws="${GW_WORKSPACE:-E:/Projects/Melee Workspace}"
if [ -z "${GW_ISO_VANILLA:-}" ] && [ -f "$ws/.env" ]; then set -a; . "$ws/.env"; set +a; fi
[ -n "${GW_ISO_VANILLA:-}" ] || { echo "v3_slice: GW_ISO_VANILLA not set"; exit 2; }
command -v python >/dev/null 2>&1 || python() { python3 "$@"; }
. "$here/exe_check.sh"; exe_pick v3_slice   # EXE=path overrides the default build/cargo/release/melee
out="${OUT:-${TEMP:-/tmp}/v3_slice}"; mkdir -p "$out"
depths="${DEPTHS:-4 7}"; frames="${FRAMES:-3600}"; only="${ONLY:-lint sim chain cash states tech volley shop finale coop petey loop3 loopboss}"
fail=0
# the machine-wide build lock, through the shared script (never by hand); BUILDLOCK_HELD=1: the caller holds it
cargo_locked() {
  local me; me="$(git rev-parse --abbrev-ref HEAD 2>/dev/null)"
  if [ "${BUILDLOCK_HELD:-0}" = 1 ]; then cargo "$@"; return; fi
  bash "${BUILDLOCK:-E:/Projects/Melee Workspace/scratch/buildlock.sh}" run "$me" cargo "$@"
}
on() { case " $only " in *" $1 "*) return 0 ;; esac; return 1; }
host=x86_64-pc-windows-msvc; case "$(uname -s)" in Linux*) host=x86_64-unknown-linux-gnu ;; esac
if on lint; then
  if (cd rust/mods/envoy_data && CARGO_TARGET_DIR="$repo/build/cargo-mods-host" RUST_TEST_THREADS=1 cargo_locked test -j 4 -q --target "$host" >"$out/lint.log" 2>&1); then
    echo "ok   A. envoy_data lint + tests, the split test ($(grep -o '[0-9]* passed' "$out/lint.log" | head -1))"
  else echo "FAIL A. envoy_data (see $out/lint.log)"; fail=1; fi
fi
if on sim; then
  if python "$here/sim.py" --v3 --seeds 120 --no-fail >"$out/sim.log" 2>&1; then
    echo "ok   B. sim.py V3 bands"; grep '^  ' "$out/sim.log" | head -12
  else echo "FAIL B. sim.py V3 bands (see $out/sim.log)"; grep '^  ' "$out/sim.log" | head -12; fail=1; fi
fi
mods="$repo/build/mods-v3test"; rm -rf "$mods"; mkdir -p "$mods"
cp "$repo/build/mods/envoy_sim.wasm" "$mods/" || { echo "v3_slice: build the mods first"; exit 2; }
export MELEE_VOLUME=3 MELEE_WASM_MODS="$mods" MELEE_ENVOY_ARM=1 MELEE_ENVOY_START_FIGHT=2 MELEE_PAD_SCRIPT="$here/drives_pad.txt"
run() { # name depth drives [env...]
  local name=$1 depth=$2 drives=$3; shift 3
  exe_banner "v3_slice $name d$depth" >&2
  env MELEE_WINDOW_TITLE="v3g agent: v3_slice $name d$depth (port/v3g)" MELEE_ENVOY_TEST_DRIVES="$drives" "$@" "$exe" synctest --engine tu --iso "$GW_ISO_VANILLA" \
    --scenario pad_falcon_fox_fd --depth "$depth" --frames "$frames" --timeout 1800 >"$out/$name.d$depth.log" 2>&1
  grep 'engine synctest' "$out/$name.d$depth.log"
}
L() { echo "$out/$1.d$2.log"; }
seen() { grep -q "$3" "$(L "$1" "$2")"; }
cnt() { grep -c "$3" "$(L "$1" "$2")"; }
check() { # n name depth line missing
  case "$4" in *" 0 MISMATCHES"*) [ -z "$5" ] && echo "ok   C$1. $2 d$3: ${4#melee: }; $(cnt "$2" "$3" 'drive fire') fires, $(cnt "$2" "$3" 'chain link') links, $(cnt "$2" "$3" 'cash-out, heat') cash-outs" \
    || { echo "FAIL C$1. $2 d$3: missing:$5 (see $(L "$2" "$3"))"; fail=1; } ;; *) echo "FAIL C$1. $2 d$3: ${4:-NO RESULT LINE (the run never finished)} (see $(L "$2" "$3"))"; fail=1 ;; esac
}
for d in $depths; do
  if on chain; then
    line=$(run chain "$d" "echo,ember,bolt,frost,shatter,noshop|winter_oath" MELEE_ENVOY_TEST_P1CPU=1)
    miss=""; seen chain "$d" "chain link, set\*100 + link 2" || grep -Eq "chain link, set\*100 \+ link[^0-9]*[0-9]*[2-9]$" "$(L chain "$d")" || miss="$miss (no second link)"
    seen chain "$d" "drive echo" || miss="$miss (no echo)"
    check 1 chain "$d" "$line" "$miss"
  fi
  if on cash; then
    line=$(run cash "$d" "ember,bolt,nova,noshop|plague_bearer" MELEE_ENVOY_TEST_P1CPU=1 MELEE_ENVOY_START_FIGHT=2)
    miss=""; seen cash "$d" "cash-out, heat" || miss="$miss (no cash-out)"; seen cash "$d" "envoy: flare" || miss="$miss (no flare)"
    check 2 cash "$d" "$line" "$miss"
  fi
  if on states; then
    line=$(run states "$d" "ember,bolt,metronome,noshop|glass_core" MELEE_ENVOY_TEST_P1CPU=1)
    miss=""; grep -Eq "heat state, player\*10 \+ state[^0-9]*[1-3]$" "$(L states "$d")" || miss="$miss (no state up)"
    check 3 states "$d" "$line" "$miss"
  fi
  if on tech; then
    line=$(run tech "$d" "riposte,clean_landing,perfect_form,metronome,ember,noshop|echo_oath" MELEE_ENVOY_START_FIGHT=1 MELEE_PAD_SCRIPT="$here/tech_pad.txt")
    miss=""
    for t in l-cancel wavedash; do seen tech "$d" "technique $t" || miss="$miss (no $t)"; done
    echo "     tech d$d: $(grep -c 'technique l-cancel' "$(L tech "$d")") l-cancel, $(grep -c 'technique wavedash' "$(L tech "$d")") wavedash, $(grep -c 'technique tech' "$(L tech "$d")") tech (informational: the pad cannot get Falcon knocked down), $(grep -c 'heat add' "$(L tech "$d")") +Heat paid"
    check 4 tech "$d" "$line" "$miss"
  fi
  if on volley; then
    line=$(run volley "$d" "nova,ember,bolt,noshop|echo_oath" MELEE_ENVOY_START_FIGHT=1 MELEE_PAD_SCRIPT="$here/volley_pad.txt")
    miss=""; seen volley "$d" "envoy: volley" || miss="$miss (no volley)"
    check 5 volley "$d" "$line" "$miss"
  fi
  if on shop; then
    line=$(run shop "$d" "ember,bolt,frost,\$400|echo_oath" MELEE_PAD_SCRIPT="$here/shop_pad.txt" MELEE_ENVOY_START_FIGHT=3)
    miss=""; seen shop "$d" "shop open" || miss="$miss (no shop)"
    seen shop "$d" "drive replaced" || miss="$miss (no replace)"
    { seen shop "$d" "shop bought" || seen shop "$d" "drive replaced"; } || miss="$miss (no buy)"
    check 6 shop "$d" "$line" "$miss"
    echo "     shop d$d: $(grep -c 'shop reroll' "$(L shop "$d")") rerolls, $(grep -c 'shop reorder' "$(L shop "$d")") moves, $(grep -c 'drive replaced' "$(L shop "$d")") replaced, $(grep -c 'drive level up' "$(L shop "$d")") level-ups"
  fi
  if on finale; then
    # A/B at one Heat (^300: +30%), every player hit treated as a smash (finaleall); the runs are identical up to the
    # first hit, so its launch with the write on vs off is the measure: speed ratio ~1.30
    line=$(run finale "$d" "ember,nova,noshop,^300,finaleall|grand_finale" MELEE_ENVOY_TEST_P1CPU=1)
    run finaleoff "$d" "ember,nova,noshop,^300,finaleall,finaleoff|grand_finale" MELEE_ENVOY_TEST_P1CPU=1 >/dev/null
    on_=$(grep -m1 -o 'finale probe (write on).* [0-9]*' "$(L finale "$d")" | grep -o '[0-9]*$'); off_=$(grep -m1 -o 'finale probe (write off).* [0-9]*' "$(L finaleoff "$d")" | grep -o '[0-9]*$')
    ratio=$(python -c "import math;print(round(math.sqrt(${on_:-0}/max(1,${off_:-0})),3))")
    miss=""; python -c "import sys;sys.exit(not 1.25<=$ratio<=1.35)" || miss="$miss (first-hit launch ratio $ratio, want ~1.30)"
    echo "     finale d$d: $(grep -c 'finale knockback' "$(L finale "$d")") knockback writes; first hit launch speed on/off = $ratio"
    check 7 finale "$d" "$line" "$miss"
  fi
  if on coop; then
    line=$(run coop "$d" "nova,comet,ember,noshop/relay|echo_oath;nova,comet,bolt/surge|overclock" MELEE_ENVOY_COOP=1 MELEE_PAD_CHANNELS=3 MELEE_ENVOY_START_FIGHT=1)
    miss=""; seen coop "$d" "envoy: run start, players 2" || miss="$miss (not co-op)"
    check 8 coop "$d" "$line" "$miss"
  fi
  if on petey; then
    # *6000: the CPU player hits x6 so the cages break inside the test (breaks, their cash-outs, RUN CLEAR)
    line=$(run petey "$d" "ember:3,bolt:3,nova:3,noshop,^400,*6000/rush|plague_bearer,grand_finale" MELEE_ENVOY_TEST_P1CPU=1 MELEE_ENVOY_BOSS=petey MELEE_ENVOY_START_FIGHT=4)
    miss=""; seen petey "$d" "envoy: boss break 4" || miss="$miss (no 4th break)"
    seen petey "$d" "RUN CLEAR" || miss="$miss (no RUN CLEAR)"; seen petey "$d" "cash-out, coins" || miss="$miss (no break cash-out)"
    check 9 petey "$d" "$line" "$miss"
    echo "     petey d$d: $(grep -c 'boss break' "$(L petey "$d")") breaks"
  fi
  if on loop3; then
    # endless loops: a run begun in loop 3 (@3) at the pack fight: CPU level 9, 3 foes, the elite modifier, foe drives
    line=$(run loop3 "$d" "ember:5,bolt:4,nova:3,noshop,@3|plague_bearer" MELEE_ENVOY_TEST_P1CPU=1 MELEE_ENVOY_START_FIGHT=2)
    miss=""; seen loop3 "$d" "foe carries" || miss="$miss (no foe drives)"
    check 10 loop3 "$d" "$line" "$miss"
    echo "     loop3 d$d: $(grep -c 'foe carries' "$(L loop3 "$d")") foe drives, $(grep -c 'drive fire' "$(L loop3 "$d")") fires"
  fi
  if on dropend; then
    # nothing a fight dropped is lost: the pack fight (2 foes) won by the CPU pilot; drops still on the floor at the
    # last KO fly to their owner (held match end), are worn or banked, then the match ends and the next fight runs
    line=$(run dropend "$d" "ember,bolt,nova,noshop,dropall" MELEE_ENVOY_TEST_P1CPU=1 MELEE_ENVOY_START_FIGHT=2)
    miss=""; seen dropend "$d" "fight won" || miss="$miss (no win)"
    seen dropend "$d" "RUN OVER" && miss="$miss (RUN OVER)"
    seen dropend "$d" "drop flies in" || miss="$miss (no fly-in)"
    check 13 dropend "$d" "$line" "$miss"
    echo "     dropend d$d: $(cnt dropend "$d" 'envoy: drop [a-z_]* -*[0-9]') drops, $(cnt dropend "$d" 'drop flies in') flown in, $(cnt dropend "$d" 'fight won') won"
  fi
  if on roadstop; then
    # the forked road's Steep session (fight 3, its last): every foe one more drive; the stop opens a pick and the
    # road's level-up token; drive protection at the shop (test knob protect: pin, a refused scrap and replace of
    # the pinned drive, a scrap, a move to the partner)
    line=$(run roadstop "$d" "ember:2,bolt,nova,*8000,steep,protect" MELEE_ENVOY_TEST_P1CPU=1 MELEE_ENVOY_START_FIGHT=2)
    miss=""; seen roadstop "$d" "foe carries" || miss="$miss (no foe drive)"
    seen roadstop "$d" "the stop, picks banked" || miss="$miss (no stop)"
    seen roadstop "$d" "envoy: scrapped" || miss="$miss (no scrap)"
    seen roadstop "$d" "test replace pinned refused 1" || miss="$miss (pinned replace not refused)"
    check 14 roadstop "$d" "$line" "$miss"
    echo "     roadstop d$d: $(cnt roadstop "$d" 'foe carries') foe drives, $(cnt roadstop "$d" 'drive level up') level-ups, $(cnt roadstop "$d" 'moved across') moved, $(cnt roadstop "$d" 'envoy: scrapped') scrapped"
  fi
  if on qtag; then
    # Quick Tag: Y tags with no freeze; a jab then Y inside the 20-frame window is a tag-cancel and a Handoff
    line=$(run qtag "$d" "ember,bolt,nova,noshop" MELEE_PAD_SCRIPT="$here/qtag_pad.txt" MELEE_ENVOY_START_FIGHT=1)
    miss=""; seen qtag "$d" "quick tag" || miss="$miss (no quick tag)"
    check 15 qtag "$d" "$line" "$miss"
    echo "     qtag d$d: $(cnt qtag "$d" 'quick tag') tags, $(cnt qtag "$d" 'envoy: handoff') handoffs, $(cnt qtag "$d" 'tag refused') refused"
  fi
  if on qtagcoop; then
    # Quick Tag in co-op: both players tag; no freeze (one player's tag never stops the other)
    line=$(run qtagcoop "$d" "nova,ember,noshop,tagauto;bolt,ember" MELEE_ENVOY_COOP=1 MELEE_PAD_CHANNELS=3 MELEE_ENVOY_START_FIGHT=1)
    miss=""; seen qtagcoop "$d" "quick tag, player.*count 1[0-9][0-9]" || miss="$miss (player 2 never tagged)"
    check 16 qtagcoop "$d" "$line" "$miss"
    echo "     qtagcoop d$d: $(cnt qtagcoop "$d" 'quick tag') tags, $(cnt qtagcoop "$d" 'envoy: handoff') handoffs"
  fi
  if on waves; then
    # enemy waves (first step): three of script kind WAVE_KIND (0 Goomba, 2 ReDead, 4 Octorok) on FD, no foe fighters
    line=$(run waves "$d" "ember,bolt,nova,noshop,~${WAVE_KIND:-0}" MELEE_ENVOY_ENEMY_LOG=1)
    miss=""; [ "$(cnt waves "$d" 'enemy spawned, handle (-1: refused) [0-9]')" -ge 3 ] || miss="$miss (not 3 spawned)"
    seen waves "$d" "envoy: enemy hit" || miss="$miss (no enemy damage record)"
    check 12 waves "$d" "$line" "$miss"
    echo "     waves d$d: $(cnt waves "$d" 'envoy: enemy hit') hits recorded, $(cnt waves "$d" 'enemy gone') gone, $(cnt waves "$d" 'fight won') won"
  fi
  if on loopboss; then
    # endless loops: the boss falls in loop 2 and the run goes on into loop 3 (no RUN CLEAR state, the loop moves)
    line=$(run loopboss "$d" "ember:4,bolt:4,nova:4,noshop,^400,*9000,@2/rush|plague_bearer,grand_finale" MELEE_ENVOY_TEST_P1CPU=1 MELEE_ENVOY_BOSS=petey MELEE_ENVOY_START_FIGHT=4)
    miss=""; seen loopboss "$d" "envoy: loop now" || miss="$miss (no loop step)"
    check 11 loopboss "$d" "$line" "$miss"
  fi
done
rm -rf "$mods"
[ $fail -eq 0 ] && echo "V3 SLICE OK" || echo "V3 SLICE FAILED (logs: $out)"
exit $fail
