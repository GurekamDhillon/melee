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
#      9 petey: the Petey boss with breaks
#   then V2's six (v2_drives.sh) on the new rules.
#   GW_ISO_VANILLA=... bash tools/rust-port/envoy/v3_slice.sh   (OUT=dir; DEPTHS; FRAMES; ONLY="chain cash ...")
set -u
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"; cd "$repo"
ws="${GW_WORKSPACE:-E:/Projects/Melee Workspace}"
if [ -z "${GW_ISO_VANILLA:-}" ] && [ -f "$ws/.env" ]; then set -a; . "$ws/.env"; set +a; fi
[ -n "${GW_ISO_VANILLA:-}" ] || { echo "v3_slice: GW_ISO_VANILLA not set"; exit 2; }
command -v python >/dev/null 2>&1 || python() { python3 "$@"; }
exe="${EXE:-$repo/build/cargo/release/melee}"; [ -e "$exe" ] || exe="$exe.exe"
out="${OUT:-${TEMP:-/tmp}/v3_slice}"; mkdir -p "$out"
depths="${DEPTHS:-4 7}"; frames="${FRAMES:-3600}"; only="${ONLY:-lint sim chain cash states tech volley shop finale coop petey}"
fail=0
# the machine-wide cargo lock (one build at a time): mkdir succeeds only when free
cargo_locked() {
  local L="${CARGO_LOCK:-E:/Projects/Melee Workspace/scratch/.cargo-build.lock}" i
  for i in $(seq 1 240); do mkdir "$L" 2>/dev/null && break; sleep 15; done
  echo "$(git rev-parse --abbrev-ref HEAD 2>/dev/null)" > "$L/owner"
  cargo "$@"; local rc=$?
  rm -rf "$L"; return $rc
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
  env MELEE_WINDOW_TITLE="v3sim agent: v3_slice $name d$depth (port/v3sim)" MELEE_ENVOY_TEST_DRIVES="$drives" "$@" "$exe" synctest --engine tu --iso "$GW_ISO_VANILLA" \
    --scenario pad_falcon_fox_fd --depth "$depth" --frames "$frames" --timeout 1800 >"$out/$name.d$depth.log" 2>&1
  grep 'engine synctest' "$out/$name.d$depth.log"
}
L() { echo "$out/$1.d$2.log"; }
seen() { grep -q "$3" "$(L "$1" "$2")"; }
cnt() { grep -c "$3" "$(L "$1" "$2")"; }
check() { # n name depth line missing
  case "$4" in *" 0 MISMATCHES"*) [ -z "$5" ] && echo "ok   C$1. $2 d$3: ${4#melee: }; $(cnt "$2" "$3" 'drive fire') fires, $(cnt "$2" "$3" 'chain link') links, $(cnt "$2" "$3" 'cash-out, heat') cash-outs" \
    || { echo "FAIL C$1. $2 d$3: missing:$5 (see $(L "$2" "$3"))"; fail=1; } ;; *) echo "FAIL C$1. $2 d$3: ${4:-no result} (see $(L "$2" "$3"))"; fail=1 ;; esac
}
for d in $depths; do
  if on chain; then
    line=$(run chain "$d" "echo,ember,bolt,frost,shatter,noshop|winter_oath" MELEE_ENVOY_TEST_P1CPU=1)
    miss=""; seen chain "$d" "chain link, set\*100 + link 2" || grep -Eq "chain link, set\*100 \+ link[^0-9]*[0-9]*[2-9]$" "$(L chain "$d")" || miss="$miss (no second link)"
    seen chain "$d" "drive echo" || miss="$miss (no echo)"
    check 1 chain "$d" "$line" "$miss"
  fi
  if on cash; then
    line=$(run cash "$d" "ember,bolt,nova,noshop|plague_bearer" MELEE_ENVOY_TEST_P1CPU=1 MELEE_ENVOY_START_FIGHT=3)
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
    line=$(run shop "$d" "ember,bolt,frost,\$400|echo_oath" MELEE_PAD_SCRIPT="$here/shop_pad.txt")
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
    line=$(run petey "$d" "ember:3,bolt:3,nova:3,noshop,^400,*6000/rush|plague_bearer,grand_finale" MELEE_ENVOY_TEST_P1CPU=1 MELEE_ENVOY_BOSS=petey MELEE_ENVOY_START_FIGHT=5)
    miss=""; seen petey "$d" "envoy: boss break 4" || miss="$miss (no 4th break)"
    seen petey "$d" "RUN CLEAR" || miss="$miss (no RUN CLEAR)"; seen petey "$d" "cash-out, coins" || miss="$miss (no break cash-out)"
    check 9 petey "$d" "$line" "$miss"
    echo "     petey d$d: $(grep -c 'boss break' "$(L petey "$d")") breaks"
  fi
done
rm -rf "$mods"
[ $fail -eq 0 ] && echo "V3 SLICE OK" || echo "V3 SLICE FAILED (logs: $out)"
exit $fail
