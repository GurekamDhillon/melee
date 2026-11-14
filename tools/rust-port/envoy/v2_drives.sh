#!/bin/bash
export MELEE_WINDOW_HIDDEN=${MELEE_WINDOW_HIDDEN:-1}  # agents' game windows are invisible (MELEE_WINDOW_HIDDEN=0 shows them)
# Envoy V2 (the full drive core, envoy-core-v2.md 6): drives charge, fire, launch, drop and level under rollback,
# deterministically, solo and in local co-op.
#   0. envoy_data's lint (the full-pool rules on) and its tests; the balancing harness's bands (sim.py);
#   then, each as a synctest at every depth of DEPTHS (default "4 7") over FRAMES (default 3600) frames, 0 mismatches:
#   1. pad: tools/rust-port/envoy/drives_pad.txt in fight 1 (a duel: the level-3 CPU leaves the pad player free to
#      tag) with time, landing, jump and tag drives (MELEE_ENVOY_TEST_DRIVES; Echo Oath given, so no keystone pick):
#      Bolt, Nova, Comet, Relay, Slipstream (tag-in) fired, a blast armed, an echo, a tag. The pad script tags with
#      D-pad down, so the row sets MELEE_ENVOY_TAG_BUTTON=dpad (Quick Tag made Y the default; without it no tag fires);
#   2. cpu: MELEE_ENVOY_TEST_P1CPU=1 (the player is a level-9 CPU, so hits and KOs happen) with the hit drives and
#      Plague Bearer + Winter Oath: a hit drive fired, Shatter on a Chilled foe, a drop taken;
#   3. twin: the CPU run with Twin Soul + Glass Core and Spire, Wildfire, Echo, Updraft / Tailwind, Thorns, Leech, Hex:
#      both sets of the pair fired (Twin Soul);
#   4. shared: MELEE_ENVOY_SHARED_DRIVES=1 on the CPU run: the run logs the shared set;
#   5. coop: MELEE_ENVOY_COOP=1, both players level-9 CPUs (MELEE_ENVOY_TEST_P1CPU) in fight 3 (the pack that ends
#      the first session, so the stop opens): two players, each with their own drives firing (owners 0-1 and 2-3),
#      each took a card at the stop (picks are banked to the stop since sessions; fight 2 has none). Foe drives are
#      checked by v3_slice (loop3, roadstop). The row runs at least 9000 frames (the pack took about 5300 frames; 7300 since wave two brings new faces);
#   6. petey: MELEE_ENVOY_BOSS=petey at fight 5 on the CPU run: drives fire on Petey (damage(amount)).
#   Every run prints its fire count (the loud-start band, envoy-core-v2.md 3.3).
#   V3 (port/v3sim): the same six on the Heat rules: charge is by damage (the pad run no longer needs Bolt, which
#   charges on 30% dealt), every test row carries `noshop`, the twin row's Echo is the V3 neighbour drive.
#   GW_ISO_VANILLA=... bash tools/rust-port/envoy/v2_drives.sh   (EXE=path to the build under test; ONLY="lint sim pad cpu twin shared coop petey" selects rows; OUT=dir for the logs; DEPTHS="4 7"; FRAMES=3600)
set -u
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"; cd "$repo"
ws="${GW_WORKSPACE:-E:/Projects/Melee Workspace}"
if [ -z "${GW_ISO_VANILLA:-}" ] && [ -f "$ws/.env" ]; then set -a; . "$ws/.env"; set +a; fi
[ -n "${GW_ISO_VANILLA:-}" ] || { echo "v2_drives: GW_ISO_VANILLA not set"; exit 2; }
command -v python >/dev/null 2>&1 || python() { python3 "$@"; }
. "$here/exe_check.sh"; exe_pick v2_drives   # EXE=path overrides the default build/cargo/release/melee
out="${OUT:-${TEMP:-/tmp}/v2_drives}"; mkdir -p "$out"
depths="${DEPTHS:-4 7}"; frames="${FRAMES:-3600}"; only="${ONLY:-lint sim pad cpu twin shared coop petey}"
fail=0
on() { case " $only " in *" $1 "*) return 0 ;; esac; return 1; }
# the machine-wide build lock, through the shared script (never by hand); BUILDLOCK_HELD=1: the caller holds it
cargo_locked() {
  local me; me="$(git rev-parse --abbrev-ref HEAD 2>/dev/null)"
  if [ "${BUILDLOCK_HELD:-0}" = 1 ]; then cargo "$@"; return; fi
  bash "${BUILDLOCK:-E:/Projects/Melee Workspace/scratch/buildlock.sh}" run "$me" cargo "$@"
}
host=x86_64-pc-windows-msvc; case "$(uname -s)" in Linux*) host=x86_64-unknown-linux-gnu ;; esac
if ! on lint; then :; elif (cd rust/mods/envoy_data && CARGO_TARGET_DIR="$repo/build/cargo-mods-host" RUST_TEST_THREADS=1 cargo_locked test -j 4 -q --target "$host" >"$out/lint.log" 2>&1); then
  echo "ok   0. envoy_data lint + tests ($(grep -o '[0-9]* passed' "$out/lint.log" | head -1))"
else echo "FAIL 0. envoy_data (see $out/lint.log)"; fail=1; fi
if ! on sim; then :; elif python "$here/sim.py" --seeds 120 --no-fail >"$out/sim.log" 2>&1; then
  echo "ok   0. sim.py bands ($(grep '^brawler' "$out/sim.log" | tr -s ' '))"
else echo "FAIL 0. sim.py bands (see $out/sim.log)"; fail=1; fi
mods="$repo/build/mods-v2test"; rm -rf "$mods"; mkdir -p "$mods"
cp "$repo/build/mods/envoy_sim.wasm" "$mods/" || { echo "v2_drives: build the mods first (tools/rust-port/modhost/build_mods.sh)"; exit 2; }
export MELEE_VOLUME=3 MELEE_WASM_MODS="$mods" MELEE_ENVOY_ARM=1 MELEE_ENVOY_START_FIGHT=2 MELEE_PAD_SCRIPT="$here/drives_pad.txt"
run() { # name depth drives [env...]
  local name=$1 depth=$2 drives=$3; shift 3
  exe_banner "v2_drives $name d$depth" >&2
  env MELEE_WINDOW_TITLE="v3sim agent: v2_drives $name d$depth (port/v3sim)" MELEE_ENVOY_TEST_DRIVES="$drives" "$@" "$exe" synctest --engine tu --iso "$GW_ISO_VANILLA" \
    --scenario pad_falcon_fox_fd --depth "$depth" --frames "$frames" --timeout 1800 >"$out/$name.d$depth.log" 2>&1
  grep 'engine synctest' "$out/$name.d$depth.log"
}
L() { echo "$out/$1.d$2.log"; }
fired() { grep -q "envoy: drive fire $3 " "$(L "$1" "$2")"; }
seen() { grep -q "$3" "$(L "$1" "$2")"; }
rate() { echo "$(grep -c "envoy: drive \(fire\|echo\)" "$(L "$1" "$2")") fire lines"; }
list() { grep -o 'drive fire [a-z_]*' "$(L "$1" "$2")" | sort -u | cut -c12- | tr '\n' ' '; }
check() { # n name depth line missing
  case "$4" in *" 0 MISMATCHES"*) [ -z "$5" ] && echo "ok   $1. $2 d$3: ${4#melee: }; $(rate "$2" "$3"); fired: $(list "$2" "$3")" \
    || { echo "FAIL $1. $2 d$3: missing:$5 (see $(L "$2" "$3"))"; fail=1; } ;; *) echo "FAIL $1. $2 d$3: ${4:-NO RESULT LINE (the run never finished)} (see $(L "$2" "$3"))"; fail=1 ;; esac
}
# a fire's log value is owner * 10 + level: owner 0-1 player 1's sets, 2-3 player 2's, 5-7 a foe
owner_fired() { grep -Eq "envoy: drive fire [a-z_]+ $3[1-3]$" "$(L "$1" "$2")" || { [ "$3" = 0 ] && grep -Eq "envoy: drive fire [a-z_]+ [1-3]$" "$(L "$1" "$2")"; }; }
for d in $depths; do
  if on pad; then
    line=$(run pad "$d" "bolt,nova,comet,noshop/relay,slipstream,feather_storm|echo_oath" MELEE_ENVOY_START_FIGHT=1 MELEE_ENVOY_TAG_BUTTON=dpad)
    miss=""; for k in nova comet relay slipstream; do fired pad "$d" $k || miss="$miss $k"; done
    seen pad "$d" "envoy: drive echo" || miss="$miss (no echo)"
    seen pad "$d" "envoy: blast armed" || miss="$miss (no blast)"
    seen pad "$d" "envoy: tag, " || miss="$miss (no tag)"
    check 1 pad "$d" "$line" "$miss"
  fi
  if on cpu; then
    line=$(run cpu "$d" "ember,frost,shatter,noshop/rush,bulwark,nova|plague_bearer,winter_oath" MELEE_ENVOY_TEST_P1CPU=1)
    miss=""; hit=0; for k in ember frost; do fired cpu "$d" $k && hit=1; done; [ $hit = 1 ] || miss="$miss (no hit drive)"
    fired cpu "$d" shatter || miss="$miss shatter"
    { seen cpu "$d" "envoy: drop " && seen cpu "$d" "envoy: drive \(gained\|level up\)"; } || miss="$miss (no drop taken)"
    check 2 cpu "$d" "$line" "$miss"
  fi
  if on twin; then
    line=$(run twin "$d" "spire,wildfire,echo,updraft,noshop/tailwind,thorns,leech,hex|twin_soul,glass_core" MELEE_ENVOY_TEST_P1CPU=1)
    miss=""; owner_fired twin "$d" 0 || miss="$miss (main set never fired)"; owner_fired twin "$d" 1 || miss="$miss (bench set never fired: Twin Soul)"
    check 3 twin "$d" "$line" "$miss"
  fi
  if on shared; then
    line=$(run shared "$d" "ember,frost,nova,noshop|overclock" MELEE_ENVOY_TEST_P1CPU=1 MELEE_ENVOY_SHARED_DRIVES=1)
    miss=""; seen shared "$d" "envoy: drives shared by the pair" || miss="$miss (shared knob not seen)"
    check 4 shared "$d" "$line" "$miss"
  fi
  if on coop; then
    line=$(frames=$(( frames < 9000 ? 9000 : frames )) run coop "$d" "bolt,nova,comet,noshop/relay,slipstream|echo_oath;bolt,comet,feather_storm/surge,relay|overclock" MELEE_ENVOY_COOP=1 MELEE_PAD_CHANNELS=3 MELEE_ENVOY_TEST_P1CPU=1 MELEE_ENVOY_START_FIGHT=2)
    miss=""; seen coop "$d" "envoy: run start, players 2" || miss="$miss (not co-op)"
    { owner_fired coop "$d" 0 || owner_fired coop "$d" 1; } || miss="$miss (player 1 never fired)"
    { owner_fired coop "$d" 2 || owner_fired coop "$d" 3; } || miss="$miss (player 2 never fired)"
    { seen coop "$d" "envoy: card taken, player 0" && seen coop "$d" "envoy: card taken, player 1"; } || miss="$miss (both players did not take a card)"
    check 5 coop "$d" "$line" "$miss"
  fi
  if on petey; then
    line=$(run petey "$d" "ember,bolt,nova,noshop/rush|plague_bearer" MELEE_ENVOY_TEST_P1CPU=1 MELEE_ENVOY_BOSS=petey MELEE_ENVOY_START_FIGHT=4)
    miss=""; seen petey "$d" "envoy: drive fire" || miss="$miss (no fire on Petey)"
    check 6 petey "$d" "$line" "$miss"
  fi
done
rm -rf "$mods"
[ $fail -eq 0 ] && echo "V2 DRIVES OK" || echo "V2 DRIVES FAILED (logs: $out)"
exit $fail
