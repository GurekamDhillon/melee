#!/bin/bash
# Geno boss grabs (geno_boss_grab.rs, notes geno-boss.md "Grabs"): a minimal Tabuu actor from the owner's local boss IR
# (MELEE_BOSS_IR_DIR / MELEE_BRAWL_LOCAL/boss-ir / _build/boss-ir) on Fox's slot of pad_falcon_fox_fd; Falcon walks
# into him (P1 pad script, Fox idle). Headless, hidden window, volume 3.
#
#   EXE=<melee exe> [OUT=dir] bash tools/rust-port/envoy/geno_grab_check.sh [hold mash sync nhv1 perf]
#
# Rows:
#   hold   Falcon holds right (no mash): Chain catch -> held through ChainStart -> ChainThrow release at f135 (throw);
#          Dive catch at f119 -> DiveTeleport -> DiveSlam throw at f5
#   mash   Falcon mashes R every other frame once next to him: caught, then escapes (CaptureCut), never thrown
#   sync   synctest depth 4 and 7, 3000 frames, the hold schedule: 0 mismatches, grabs inside the run
#   nhv1   the hold schedule twice through `golden`: identical NHV1 rows
#   perf   real-time play run (Falcon vs a level-1 CPU Fox carrier, FD), MELEE_PROFILE=1: zero_hitch_check
set -u
here=$(cd "$(dirname "$0")" && pwd)
repo=$(cd "$here/../../.." && pwd)
. "$here/exe_check.sh"; exe_pick geno_grab_check
out=${OUT:-$repo/build/geno-grab-check}
iso=${GW_ISO_VANILLA:-}
[ -n "$iso" ] || { echo "geno_grab_check: GW_ISO_VANILLA is not set"; exit 2; }
rows=("$@"); [ ${#rows[@]} -gt 0 ] || rows=(hold mash sync nhv1 perf)
mkdir -p "$out"
printf '99999 0000 60 0\n' > "$out/hold.txt"
{ printf '400 0000 60 0\n'; for i in $(seq 1 2000); do printf '2 0020 0 0\n2 0000 0 0\n'; done; } > "$out/mash.txt"
fail=0
base() { # name pad cmd...
  local name=$1 pad=$2; shift 2
  ( cd "$repo" && env MELEE_VOLUME=3 MELEE_WINDOW_HIDDEN=1 MELEE_FULLSCREEN=off MELEE_WINDOW_UNFOCUSED=1 MELEE_PAD_IGNORE_ADAPTER=1 \
      MELEE_VS_BOSS=tabuu MELEE_VS_BOSS_SLOT=1 MELEE_GRAB_LOG=1 MELEE_PAD_SCRIPT="$out/$pad" \
      MELEE_WINDOW_TITLE="genograb geno_grab_check $name" "$@" ) > "$out/$name.log" 2>&1
}
golden() { base "$1" "$2" timeout 900 "$exe" golden --engine tu --iso "$iso" --scenario pad_falcon_fox_fd --frames "${FRAMES:-3000}" --out "$out/$1.csv"; }
L() { echo "$out/$1.log"; }
need() { if grep -q -E "$2" "$(L "$1")"; then echo "ok   $1: $3"; else echo "FAIL $1: $3 (no /$2/)"; fail=1; fi; }
never() { if grep -q -E "$2" "$(L "$1")"; then echo "FAIL $1: $3 (/$2/ seen)"; fail=1; else echo "ok   $1: $3"; fi; }
for r in "${rows[@]}"; do
  exe_banner "geno_grab_check $r"
  case "$r" in
    hold)
      golden hold hold.txt
      need hold 'tabuu actor' "the Tabuu grab define lowered from the IR"
      need hold 'catch: slot 0 held \(status 36 motion frame 89' "Chain catch in ChainStart's 060A window (89-106)"
      need hold 'status 36 -> 37 \(ChainThrow\)' "held through ChainStart into ChainThrow (hold mask 36|37)"
      need hold 'throw: slot 0 with ChainAttack \(2[0-9.]+ dmg, angle 361' "ChainAttack release f135: 20 + 2.5 L at 361"
      need hold 'catch: slot 0 held \(status 19 motion frame 119' "Dive catch on DiveStart2 f119"
      need hold 'status 20 -> 21 \(DiveSlam\)' "DiveTeleport -> DiveSlam while held"
      need hold 'throw: slot 0 with grab:dive throw \(3[0-9.]+ dmg, angle 22, kbg 75' "Dive throw 30 + 2.8 L at 22, growth 75"
      never hold 'escape:' "no escape without a mash"
      ;;
    mash)
      golden mash mash.txt
      need mash 'catch: slot 0 held' "caught"
      need mash 'escape: slot 0 left the hold \((229|230)\)' "mashed out (CaptureCut / CaptureJump)"
      ;;
    sync)
      for depth in 4 7; do
        base "sync$depth" hold.txt timeout 3000 "$exe" synctest --engine tu --iso "$iso" --scenario pad_falcon_fox_fd \
          --depth "$depth" --frames "${FRAMES:-3000}" --timeout 2800
        line=$(grep 'engine synctest' "$(L sync$depth)" | tail -1)
        case "$line" in *" 0 MISMATCHES"*) echo "ok   sync depth $depth: ${line#melee: }" ;; *) echo "FAIL sync depth $depth: ${line:-no result line}"; fail=1 ;; esac
        need "sync$depth" 'throw: slot 0' "a grab and throw inside the synctest"
      done
      ;;
    nhv1)
      golden nhv1a hold.txt; golden nhv1b hold.txt
      if cmp -s "$out/nhv1a.csv" "$out/nhv1b.csv" && [ -s "$out/nhv1a.csv" ]; then
        echo "ok   nhv1: identical over two runs ($(wc -l < "$out/nhv1a.csv") rows, last $(tail -1 "$out/nhv1a.csv"))"
      else echo "FAIL nhv1: the two runs differ"; fail=1; fi
      ;;
    perf)
      rm -rf "$out/perf"; mkdir -p "$out/perf"; cp "$exe" "$out/perf/melee.exe"
      ( cd "$out/perf" && env MELEE_VOLUME=3 MELEE_WINDOW_HIDDEN=1 MELEE_FULLSCREEN=off MELEE_WINDOW_UNFOCUSED=1 MELEE_PAD_IGNORE_ADAPTER=1 \
          MELEE_VS_BOSS=tabuu MELEE_GRAB_LOG=1 MELEE_PAD_SCRIPT="$out/hold.txt" MELEE_PROFILE=1 MELEE_WINDOW_TITLE="genograb geno_grab_check perf" \
          timeout 600 ./melee.exe play --iso "$iso" --max-frames "${FRAMES:-3000}" --match "falcon,fox/cpu1,FINAL DESTINATION" --time-secs 600 ) > "$out/perf.log" 2>&1
      need perf 'throw: slot' "grabs in the real-time run"
      p=$(ls -t "$out/perf/profiles/"profile-*.csv 2>/dev/null | head -1)
      if [ -n "$p" ]; then python "$repo/tools/rust-port/perf/zero_hitch_check.py" "$p" --label geno-grab > "$out/hitch.txt"; [ $? = 0 ] || fail=1; head -12 "$out/hitch.txt"
      else echo "FAIL perf: no profile written"; fail=1; fi
      ;;
    *) echo "geno_grab_check: unknown row $r"; fail=1 ;;
  esac
done
[ $fail = 0 ] && echo "GENO GRAB CHECK OK" || echo "GENO GRAB CHECK FAILED"
exit $fail
