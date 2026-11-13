#!/bin/bash
# Petey boss checks (docs/rust-port/notes/petey-behavior.md, "port/envoy-petey2"). Scripted and headless: the pad
# script is idle and the player hits come from MELEE_PETEY_TEST_HITS, so every row is the same run on every machine.
#
#   EXE=<melee exe> MODS=<dir with envoy_sim.wasm> [OUT=dir] [DISC=vanilla|ace] [PLAYER=kirby] \
#     bash tools/rust-port/envoy/petey_check.sh [idle hits break bodybreak sync ...]
#
# Rows:
#   idle   no hits for 2600 frames: the attack loop (Wait A -> basket -> Wait B -> menu), wait lengths inside the
#          parameter ranges at L 2.5, the baskets' shake at motion frame 70 and voice at 17, the jump chain's sounds
#          and shakes, the landing's large shake at frame 2
#   hits   a cage hit in a wait (no flinch), body hits to 50% (Angry when the wait ends, L 3.3), to 25% (L 4.1), then under 10% (death:
#          the lower cage's princess freed, Down with its sounds and shakes, the carrier KO DOWN_END frames later)
#   break  one cage taken to 0 by a hit on that cage: the same-frame death check (rules 6/7) fires first, so it is a
#          death (status 15) with that side freed and no cage-break Down (sora_enemy handler, code-spec.md 3)
#   bodybreak  both cages taken to 0 by a body hit (no cage hit flag): the cage-break path, Down by rule 10
#   sync   synctest depth 4 over 3000 frames with the `hits` schedule: 0 mismatches
# The disc comes from GW_ISO_VANILLA / GW_ISO_ACE (never printed). Takes no lock itself: wrap it
# (`runlight.sh <who> bash ...` per row; `sync` is a heavier run).
set -u
here=$(cd "$(dirname "$0")" && pwd)
repo=$(cd "$here/../../.." && pwd)
. "$here/exe_check.sh"; exe_pick petey_check
mods=${MODS:-$repo/build/mods}
out=${OUT:-$repo/build/petey-check}
disc=${DISC:-vanilla}
player=${PLAYER:-kirby}
case "$disc" in ace) iso=${GW_ISO_ACE:-} ;; *) iso=${GW_ISO_VANILLA:-} ;; esac
[ -n "$iso" ] || { echo "petey_check: the $disc disc variable is not set"; exit 2; }
[ -f "$mods/envoy_sim.wasm" ] || { echo "petey_check: no envoy_sim.wasm in MODS"; exit 2; }
rows=("$@"); [ ${#rows[@]} -gt 0 ] || rows=(idle hits break bodybreak sync)
mkdir -p "$out"
pad="$out/idle_pad.txt"; echo "99999 0000 0 0" > "$pad"
HITS="50:l:16,400:b:66,900:b:39,1500:b:22"  # cages 157: L 141, 75 (<=50%), 36 (<=25%), 14 (<10%)
BREAK="300:r:200"
BODYBREAK="300:b:200"
fail=0
base_env() {
  env MELEE_VOLUME=3 MELEE_FULLSCREEN=off MELEE_WASM_MODS="$mods" MELEE_ENVOY_BOSS=petey MELEE_ENVOY_START_FIGHT=5 \
    MELEE_ENVOY_ARM=1 MELEE_PETEY_LOG=1 MELEE_WINDOW_UNFOCUSED=1 MELEE_PAD_IGNORE_ADAPTER=1 MELEE_PAD_SCRIPT="$pad" \
    MELEE_ENVOY_DIR="$out/save-$1" MELEE_WINDOW_TITLE="petey2 petey_check $1 ($disc)" "${@:2}"
}
play() { # name frames env...
  local name=$1 frames=$2; shift 2
  mkdir -p "$out/save-$name"
  exe_banner "petey_check $name ($disc, $player)"
  ( cd "$out" && base_env "$name" "$@" timeout 600 "$exe" play --iso "$iso" --match "$player,luigi/cpu9,32" --headless \
      --max-frames "$frames" > "$out/$name.$disc.log" 2>&1 )
  echo "   exit $? ($(grep -c '^petey:' "$out/$name.$disc.log") petey lines)"
}
L() { echo "$out/$1.$disc.log"; }
need() { # row pattern what
  if grep -q -E "$2" "$(L "$1")"; then echo "ok   $1: $3"; else echo "FAIL $1: $3 (no /$2/ in $(L "$1"))"; fail=1; fi
}
order() { # row what pattern...
  local row=$1 what=$2 last=0 n; shift 2
  for p in "$@"; do
    n=$(grep -n -E "$p" "$(L "$row")" | awk -F: -v l="$last" '$1 > l {print $1; exit}')
    [ -n "$n" ] || { echo "FAIL $row: $what (/$p/ not found after line $last)"; fail=1; return; }
    last=$n
  done
  echo "ok   $row: $what"
}
for r in "${rows[@]}"; do
  case "$r" in
    idle)
      play idle 2600
      need idle 'fight begins, cages 157 difficulty 5 L 2.5' "fight begins: cages 157, difficulty 5, L 2.5"
      order idle "attack loop: Wait A -> choose -> basket -> Wait B -> menu" 'status 2 -> 4 ' 'status 4 -> 1[12] ' 'status 1[12] -> 3 ' 'status 3 -> 5 '
      need idle 'shake 0 \(motion [12] frame 70\)' "basket shake at motion frame 70"
      need idle 'sound 9593 \(motion [12] frame 17\)' "basket voice at motion frame 17"
      need idle 'sound 9601 \(motion 9 frame 14\)' "JumpStart sound at frame 14"
      need idle 'shake 2 \(motion 7 frame 2\)' "landing: large shake at frame 2"
      need idle 'sound 9598 \(motion 7 frame 0\)' "landing sound at frame 0"
      python - "$(L idle)" <<'PY' || fail=1
import re, sys
ent = [(int(m.group(1)), int(m.group(2)), int(m.group(3))) for m in re.finditer(r'petey: f(\d+) status (-?\d+) -> (\d+)', open(sys.argv[1], errors='replace').read())]
bad = 0; wa = []; wb = []
for (f0, a, b), (f1, c, d) in zip(ent, ent[1:]):
    if b == 2 and d == 4: wa.append(f1 - f0)
    if b == 3 and d == 5: wb.append(f1 - f0)
# L 2.5: Wait A in [100 - 22.5, 150 - 30] = [77.5, 120]; Wait B in [1.55, 15.625] (the status change lands a frame late)
if not wa or min(wa) < 77 or max(wa) > 122: bad = 1
if not wb or min(wb) < 1 or max(wb) > 18: bad = 1
print(('FAIL' if bad else 'ok  '), 'idle: Wait A frames', sorted(set(wa)), 'in 77..122; Wait B frames', sorted(set(wb)), 'in 1..18')
sys.exit(bad)
PY
      python - "$(L idle)" <<'PY' || fail=1
import re, sys
# no KO under 100%: the victim's position stays within the blast zones the log reports
t = open(sys.argv[1], errors='replace').read()
z = re.search(r'petey: arena blast zones (-?\d+) (-?\d+) (-?\d+) (-?\d+), knockback x(\d+)', t)
if not z:
    print('FAIL idle: no arena blast zones in the log (needs the local Brawl files)'); sys.exit(1)
zl, zr, zt, zb = (float(z.group(i)) for i in range(1, 5))
v = [(float(a), float(b), float(c)) for a, b, c in re.findall(r'petey: f\d+ victim 0 pos (-?[\d.]+) (-?[\d.]+) pct ([\d.]+)', t)]
lo = [p for p in v if p[2] < 100]
out = [p for p in lo if p[0] < zl or p[0] > zr or p[1] > zt or p[1] < zb]
far = max((abs(p[0]) for p in lo), default=0); high = max((p[1] for p in lo), default=0)
print(('FAIL' if out or not lo else 'ok  '), 'idle: knockback x%s, launches under 100%% stay inside the arena blast zones: %d samples, max |x| %.0f, max y %.0f' % (z.group(5), len(lo), far, high))
sys.exit(1 if out or not lo else 0)
PY
      ;;
    hits)
      play hits 2600 MELEE_PETEY_TEST_HITS="$HITS"
      python - "$(L hits)" <<'PY' || fail=1
import re, sys
t = open(sys.argv[1], errors='replace').read()
fl = re.findall(r'status \d+ -> 1[34] ', t)
print(('FAIL' if fl else 'ok  '), 'hits: no flinch (DamageLeft/Right) on a cage hit in a wait (observed Brawl: 0 of 5)', fl[:2])
sys.exit(1 if fl else 0)
PY
      order hits "50% -> phase 1 (L 3.3) -> Angry with roar and shakes" 'test hit b 66' 'phase 1 \(L 3.3\)' 'status [23] -> 6 ' 'shake 1 \(motion 0 frame 4\)' 'sound 9592 \(motion 0 frame 9\)' 'status 6 -> [45] '
      order hits "25% -> phase 2 (L 4.1), no status change" 'test hit b 39' 'phase 2 \(L 4.1\)'
      order hits "under 10% + a hit -> dying, lower (left) cage freed, Down, KO" 'test hit b 22' 'sound 8837' 'status 15 -> 1 ' 'dying \(lower cage 0' 'sound 9612 \(motion 5 frame 0\)' 'shake 2 \(motion 5 frame 1\)' 'sound 9596 \(motion 5 frame 152\)' 'defeated \(cause 1, freed side 0\)'
      python - "$(L hits)" <<'PY' || fail=1
import re, sys
t = open(sys.argv[1], errors='replace').read()
a = re.search(r'petey: f(\d+) status 15 -> 1 ', t); b = re.search(r'petey: f(\d+) defeated', t)
ok = a and b and 179 <= int(b.group(1)) - int(a.group(1)) <= 181
print(('ok  ' if ok else 'FAIL'), 'hits: carrier KO', (int(b.group(1)) - int(a.group(1))) if a and b else None, 'frames after the death hit (want 180)')
sys.exit(0 if ok else 1)
PY
      ;;
    break)
      play break 1200 MELEE_PETEY_TEST_HITS="$BREAK"
      order break "a cage hit to 0 -> same-frame death check, right freed, no break" 'test hit r 200' 'status [0-9]+ -> 15 ' 'status 15 -> 1 ' 'dying \(cage 1 hit to 0.0%, same-frame check\)' 'defeated \(cause 1, freed side 1\)'
      if grep -q 'cage 1 broken' "$(L break)"; then echo "FAIL break: the cage-break path ran after the death check"; fail=1; else echo "ok   break: no cage-break path"; fi
      ;;
    bodybreak)
      play bodybreak 1200 MELEE_PETEY_TEST_HITS="$BODYBREAK"
      order bodybreak "a body hit to 0 -> broken, Down by rule 10" 'test hit b 200' 'cage 0 broken' 'status [0-9]+ -> 1 ' 'defeated \(cause 2, freed side 0\)'
      ;;
    sync)
      exe_banner "petey_check sync ($disc, $player)"
      mkdir -p "$out/save-sync"
      ( cd "$out" && base_env sync MELEE_PETEY_TEST_HITS="$HITS" "$exe" synctest --engine tu --iso "$iso" \
          --scenario pad_falcon_fox_fd --depth "${DEPTH:-4}" --frames "${FRAMES:-3000}" --timeout 1500 > "$out/sync.$disc.log" 2>&1 )
      line=$(grep 'engine synctest' "$out/sync.$disc.log" | tail -1)
      case "$line" in *" 0 MISMATCHES"*) echo "ok   sync: ${line#melee: }" ;; *) echo "FAIL sync: ${line:-no result line} (see $out/sync.$disc.log)"; fail=1 ;; esac
      grep -q 'petey: fight begins' "$out/sync.$disc.log" && echo "ok   sync: the Petey fight ran" || { echo "FAIL sync: no Petey fight in the run"; fail=1; }
      ;;
    *) echo "petey_check: unknown row $r"; fail=1 ;;
  esac
done
[ $fail = 0 ] && echo "PETEY CHECK OK ($disc)" || echo "PETEY CHECK FAILED ($disc)"
exit $fail
