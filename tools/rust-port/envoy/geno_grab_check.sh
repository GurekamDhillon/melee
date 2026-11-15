#!/bin/bash
# Geno boss grabs (geno_boss_grab.rs, notes geno-boss.md "Grabs"): a minimal Tabuu actor and Bucculus's chew from the
# owner's local IR (MELEE_BOSS_IR_DIR / MELEE_BRAWL_LOCAL/boss-ir / _build/boss-ir) on Fox's slot of pad_falcon_fox_fd;
# Falcon is P1's pad script, Fox idle. Headless, hidden window, volume 3.
#
#   EXE=<melee exe> [OUT=dir] bash tools/rust-port/envoy/geno_grab_check.sh [hold mash chase chew chewmash sync nhv1 perf]
#
# Rows:
#   hold      Tabuu; Falcon holds right (no mash): Chain catch -> ChainThrow release (throw); Dive catch ->
#             DiveTeleport -> DiveSlam throw
#   mash      Tabuu; Falcon mashes R every other frame once next to him: caught, then escapes (CaptureCut)
#   chase     Tabuu; Falcon stands still: the approach gates (6604 / 6605 / 660E, slot-5 kinetic) walk Tabuu to him
#             in ChainStart, the catch follows
#   chew      Bucculus; Falcon walks in: Bites1 catch -> Absorption bites every Bites2 cadence -> spat
#             out (RollingJump) after the IR's chew limit
#   chewmash  Bucculus; Falcon mashes: caught, escapes, Bucculus leaves the chew
#   sync      synctest depth 4 and 7, 3000 frames, for both actors: 0 mismatches, grabs inside every run
#   nhv1      `golden` twice per actor: identical NHV1 rows
#   perf      real-time play runs (Falcon vs a level-1 CPU Fox carrier, FD), MELEE_PROFILE=1, both actors: zero_hitch_check
set -u
here=$(cd "$(dirname "$0")" && pwd)
repo=$(cd "$here/../../.." && pwd)
. "$here/exe_check.sh"; exe_pick geno_grab_check
out=${OUT:-$repo/build/geno-grab-check}
iso=${GW_ISO_VANILLA:-}
[ -n "$iso" ] || { echo "geno_grab_check: GW_ISO_VANILLA is not set"; exit 2; }
rows=("$@"); [ ${#rows[@]} -gt 0 ] || rows=(hold mash chase chew chewmash sync nhv1 perf)
mkdir -p "$out"
printf '99999 0000 60 0\n' > "$out/hold.txt"
printf '99999 0000 0 0\n' > "$out/idle.txt"
{ printf '400 0000 60 0\n'; for i in $(seq 1 2000); do printf '2 0020 0 0\n2 0000 0 0\n'; done; } > "$out/mash.txt"
# expected values: the owner's local file (never committed; from his Brawl files / IR)
expect=${GENO_EXPECT:-}
for d in "${MELEE_BOSS_IR_DIR:-}" "${MELEE_BRAWL_LOCAL:+$MELEE_BRAWL_LOCAL/boss-ir}" "$repo/../_build/boss-ir" "$repo/../../_build/boss-ir"; do
  [ -z "$expect" ] && [ -n "$d" ] && [ -f "$d/checks/geno-checks.env" ] && expect="$d/checks/geno-checks.env"
done
[ -n "$expect" ] || { echo "$(basename "$0"): no local expected values (boss-ir/checks/geno-checks.env)"; exit 2; }
. "$expect"
fail=0
base() { # boss name pad cmd...
  local boss=$1 name=$2 pad=$3; shift 3
  ( cd "$repo" && env MELEE_VOLUME=3 MELEE_WINDOW_HIDDEN=1 MELEE_FULLSCREEN=off MELEE_WINDOW_UNFOCUSED=1 MELEE_PAD_IGNORE_ADAPTER=1 \
      MELEE_TABUU_DEFINE=1 MELEE_VS_BOSS=$boss MELEE_VS_BOSS_SLOT=1 MELEE_GRAB_LOG=1 MELEE_PETEY_LOG=1 MELEE_PAD_SCRIPT="$out/$pad" \
      MELEE_WINDOW_TITLE="genograb geno_grab_check $name" "$@" ) > "$out/$name.log" 2>&1
}
golden() { base "$1" "$2" "$3" timeout 900 "$exe" golden --engine tu --iso "$iso" --scenario pad_falcon_fox_fd --frames "${FRAMES:-3000}" --out "$out/$2.csv"; }
L() { echo "$out/$1.log"; }
need() { if grep -q -E "$2" "$(L "$1")"; then echo "ok   $1: $3"; else echo "FAIL $1: $3 (no /$2/)"; fail=1; fi; }
never() { if grep -q -E "$2" "$(L "$1")"; then echo "FAIL $1: $3 (/$2/ seen)"; fail=1; else echo "ok   $1: $3"; fi; }
for r in "${rows[@]}"; do
  exe_banner "geno_grab_check $r"
  case "$r" in
    hold)
      golden tabuu hold hold.txt
      need hold 'tabuu actor' "the Tabuu grab define lowered from the IR"
      need hold "catch: slot 0 held \\(status 36 motion frame $TABUU_CHAIN_F" "Chain catch at the start of ChainStart's 060A window"
      need hold 'status 36 -> 37 \(ChainThrow\)' "held through ChainStart into ChainThrow (hold mask 36|37)"
      need hold "throw: slot 0 with ChainAttack \\($TABUU_CHAIN_DMG dmg, angle $TABUU_CHAIN_ANGLE" "ChainAttack release with the IR's box"
      need hold "catch: slot 0 held \\(status 19 motion frame $TABUU_DIVE_F" "Dive catch on DiveStart2's catch frame"
      need hold 'status 20 -> 21 \(DiveSlam\)' "DiveTeleport -> DiveSlam while held"
      need hold "throw: slot 0 with grab:dive throw \\($TABUU_DIVE_DMG dmg, angle $TABUU_DIVE_ANGLE, kbg $TABUU_DIVE_KBG" "Dive throw with the IR's throw block"
      never hold 'escape:' "no escape without a mash"
      ;;
    mash)
      golden tabuu mash mash.txt
      need mash 'catch: slot 0 held' "caught"
      need mash 'escape: slot 0 left the hold \((229|230)\)' "mashed out (CaptureCut / CaptureJump)"
      ;;
    chase)
      MELEE_TABUU_GRAB=chain golden tabuu chase idle.txt
      need chase 'approach: x' "Tabuu walks toward the standing target (slot-5 kinetic on)"
      need chase 'catch: slot 0 held \(status 36' "the Chain catch after the approach"
      ;;
    chew)
      golden bucculus chew hold.txt
      need chew 'bucculus actor' "the Bucculus chew define lowered from the IR"
      need chew 'catch: slot 0 held \(status 11' "Bites1 catch (060A)"
      need chew "chew: slot 0 with Absorption bite \\($BUCC_BITE_DMG dmg" "Absorption bite (the IR's box) on the held player"
      python - "$(L chew)" $BUCC_LIMIT $BUCC_SLACK $BUCC_GAP_MIN $BUCC_GAP_MAX <<'PY' || fail=1
import re, sys
t = open(sys.argv[1], errors='replace').read()
c = re.search(r'f(\d+) catch: slot 0 held \(status 11', t)
bites = [int(m.group(1)) for m in re.finditer(r'f(\d+) chew: slot 0', t)]
spit = re.search(r'f(\d+) throw: slot 0 with Spit-out roll', t)
if not (c and spit):
    print('FAIL chew: no catch / spit'); sys.exit(1)
c, s = int(c.group(1)), int(spit.group(1))
b = [x for x in bites if c < x < s]
gaps = sorted(set(y - x for x, y in zip(b, b[1:])))
lim, slack, g0, g1 = map(int, sys.argv[2:6])
# R23 is live in Bites2 only: the spit comes at the first Bites2 frame once the limit has passed (<= one Absorption later)
ok = len(b) >= 3 and lim <= s - c <= lim + slack and gaps and all(g0 <= g <= g1 for g in gaps)
print(('ok  ' if ok else 'FAIL'), f'chew: {len(b)} bites, gaps {gaps} (Absorption + Bites2 cadence), spat out {s - c} frames after the catch (the IR limit, checked in Bites2)')
sys.exit(0 if ok else 1)
PY
      ;;
    chewmash)
      golden bucculus chewmash mash.txt
      need chewmash 'catch: slot 0 held \(status 11' "caught"
      need chewmash 'escape: slot 0 left the hold' "mashed out of the chew"
      need chewmash 'status (10|12) -> 7 ' "Bucculus leaves the chew when the hold ends"
      ;;
    sync)
      for boss in tabuu bucculus; do
        for depth in 4 7; do
          n="sync-$boss-$depth"
          base "$boss" "$n" hold.txt timeout 3000 "$exe" synctest --engine tu --iso "$iso" --scenario pad_falcon_fox_fd \
            --depth "$depth" --frames "${FRAMES:-3000}" --timeout 2800
          line=$(grep 'engine synctest' "$(L $n)" | tail -1)
          case "$line" in *" 0 MISMATCHES"*) echo "ok   $n: ${line#melee: }" ;; *) echo "FAIL $n: ${line:-no result line}"; fail=1 ;; esac
          need "$n" 'throw: slot 0' "a grab and release inside the synctest"
        done
      done
      ;;
    nhv1)
      for boss in tabuu bucculus; do
        golden "$boss" "nhv1-$boss-a" hold.txt; golden "$boss" "nhv1-$boss-b" hold.txt
        a="$out/nhv1-$boss-a.csv"; b="$out/nhv1-$boss-b.csv"
        if cmp -s "$a" "$b" && [ -s "$a" ]; then
          echo "ok   nhv1 $boss: identical over two runs ($(wc -l < "$a") rows, last $(tail -1 "$a" | cut -d, -f1-3))"
        else echo "FAIL nhv1 $boss: the two runs differ"; fail=1; fi
      done
      ;;
    perf)
      for boss in tabuu bucculus; do
        d="$out/perf-$boss"; rm -rf "$d"; mkdir -p "$d"; cp "$exe" "$d/melee.exe"
        ( cd "$d" && env MELEE_VOLUME=3 MELEE_WINDOW_HIDDEN=1 MELEE_FULLSCREEN=off MELEE_WINDOW_UNFOCUSED=1 MELEE_PAD_IGNORE_ADAPTER=1 \
            MELEE_TABUU_DEFINE=1 MELEE_VS_BOSS=$boss MELEE_GRAB_LOG=1 MELEE_PETEY_LOG=1 MELEE_PAD_SCRIPT="$out/hold.txt" MELEE_PROFILE=1 MELEE_WINDOW_TITLE="genograb geno_grab_check perf $boss" \
            timeout 600 ./melee.exe play --iso "$iso" --max-frames "${FRAMES:-3000}" --match "falcon,fox/cpu1,FINAL DESTINATION" --time-secs 600 ) > "$out/perf-$boss.log" 2>&1
        need "perf-$boss" 'catch: slot' "grabs in the real-time run"
        p=$(ls -t "$d/profiles/"profile-*.csv 2>/dev/null | head -1)
        if [ -n "$p" ]; then python "$repo/tools/rust-port/perf/zero_hitch_check.py" "$p" --label "geno-grab-$boss" > "$out/hitch-$boss.txt"; [ $? = 0 ] || fail=1; head -2 "$out/hitch-$boss.txt"
        else echo "FAIL perf $boss: no profile written"; fail=1; fi
        rm -f "$d/melee.exe"
      done
      ;;
    *) echo "geno_grab_check: unknown row $r"; fail=1 ;;
  esac
done
[ $fail = 0 ] && echo "GENO GRAB CHECK OK" || echo "GENO GRAB CHECK FAILED"
exit $fail
