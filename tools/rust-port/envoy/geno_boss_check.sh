#!/bin/bash
# Petey as a Geno actor (docs/specs/geno-v5.2-boss.md, docs/rust-port/notes/geno-boss.md). Needs the owner's local
# boss IR (MELEE_BOSS_IR_DIR / MELEE_BRAWL_LOCAL/boss-ir / _build/boss-ir) and Brawl files; headless, scripted hits.
#
#   EXE=<melee exe> MODS=<dir with envoy_sim.wasm> [OUT=dir] [PLAYER=kirby] \
#     bash tools/rust-port/envoy/geno_boss_check.sh [idle death break bodybreak sync]
#
# Rows:
#   idle       no hits: the IR's attack loop (2 -> 4 -> 11/12 -> 3 -> 5 -> ...), wait lengths in range at L 2.5
#   death      body hits to 50% (phase 1: Angry at the next wait evaluation, not at the wait's end), 25% (phase 2), under
#              10%, then one more hit: status 15 (threshold-then-hit), stage events 1 and 1000/1001, no Down, KO
#   break      a hit on the right cage to 0: cage-break path (stage event 1001, Down by rule 10, event 9 not before)
#   bodybreak  a body hit to 0 on both cages: the same path for the lower side
#   sync       synctest at DEPTH (default 4) over FRAMES (3000) with the death schedule: 0 mismatches
set -u
here=$(cd "$(dirname "$0")" && pwd)
repo=$(cd "$here/../../.." && pwd)
. "$here/exe_check.sh"; exe_pick geno_boss_check
mods=${MODS:-$repo/build/mods}
out=${OUT:-$repo/build/geno-boss-check}
player=${PLAYER:-kirby}
iso=${GW_ISO_VANILLA:-}
[ -n "$iso" ] || { echo "geno_boss_check: GW_ISO_VANILLA is not set"; exit 2; }
rows=("$@"); [ ${#rows[@]} -gt 0 ] || rows=(idle death break bodybreak sync)
mkdir -p "$out"
pad="$out/idle_pad.txt"; echo "99999 0000 0 0" > "$pad"
# cages trunc(120 + 15 L) = 157 at L 2.5: 141 (90%), 75 (48%), 36 (23%), 14 (8.9%), then any hit at <= 10%
DEATH="50:l:16,400:b:66,900:b:39,1500:b:22,1700:b:3"
fail=0
base_env() {
  env MELEE_VOLUME=3 MELEE_WINDOW_HIDDEN=1 MELEE_FULLSCREEN=off MELEE_WASM_MODS="$mods" MELEE_ENVOY_BOSS=petey \
    MELEE_ENVOY_START_FIGHT=5 MELEE_ENVOY_ARM=1 MELEE_PETEY_LOG=1 MELEE_WINDOW_UNFOCUSED=1 MELEE_PAD_IGNORE_ADAPTER=1 \
    MELEE_PAD_SCRIPT="$pad" MELEE_ENVOY_DIR="$out/save-$1" MELEE_WINDOW_TITLE="genoboss geno_boss_check $1" "${@:2}"
}
play() { # name frames env...
  local name=$1 frames=$2; shift 2
  mkdir -p "$out/save-$name"
  exe_banner "geno_boss_check $name"
  ( cd "$out" && base_env "$name" "$@" timeout 600 "$exe" play --iso "$iso" --match "$player,luigi/cpu9,32" --headless \
      --max-frames "$frames" > "$out/$name.log" 2>&1 )
  echo "   exit $? ($(grep -c '^petey:\|^geno_boss:' "$out/$name.log") boss lines)"
  grep -q 'petey: geno actor' "$out/$name.log" || { echo "FAIL $name: not the Geno actor ($(grep -m1 'petey: no Geno define\|hand-coded' "$out/$name.log"))"; fail=1; }
}
L() { echo "$out/$1.log"; }
need() { if grep -q -E "$2" "$(L "$1")"; then echo "ok   $1: $3"; else echo "FAIL $1: $3 (no /$2/)"; fail=1; fi; }
never() { if grep -q -E "$2" "$(L "$1")"; then echo "FAIL $1: $3 (/$2/ seen)"; fail=1; else echo "ok   $1: $3"; fi; }
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
      need idle 'fight begins, cages 157 difficulty 5 L 2.5' "cages trunc(120 + 15 L) = 157 at Normal"
      order idle "attack loop" 'status 2 -> 4 ' 'status 4 -> 1[12] ' 'status 1[12] -> 3 ' 'status 3 -> 5 '
      python - "$(L idle)" <<'PY' || fail=1
import re, sys
ent = [(int(m.group(1)), int(m.group(2)), int(m.group(3))) for m in re.finditer(r'petey: f(\d+) status (-?\d+) -> (\d+)', open(sys.argv[1], errors='replace').read())]
wa = [f1 - f0 for (f0, a, b), (f1, c, d) in zip(ent, ent[1:]) if b == 2 and d == 4]
wb = [f1 - f0 for (f0, a, b), (f1, c, d) in zip(ent, ent[1:]) if b == 3 and d == 5]
bad = not wa or min(wa) < 77 or max(wa) > 122 or not wb or min(wb) < 1 or max(wb) > 18
print(('FAIL' if bad else 'ok  '), 'idle: Wait A', sorted(set(wa)), 'in 77..122; Wait B', sorted(set(wb)), 'in 1..18')
sys.exit(1 if bad else 0)
PY
      ;;
    death)
      play death 2400 MELEE_PETEY_TEST_HITS="$DEATH"
      order death "phase 1 at <= 50%: Angry from the wait, L 3.3" 'test hit b 66' 'phase 1 \(L 3.3\)' 'status 6 -> [45] '
      python - "$(L death)" <<'PY' || fail=1
import re, sys
t = open(sys.argv[1], errors='replace').read()
# crosscheck 2: rule 28 fires at the first evaluation in status 2/3 after 0x1400, not when the wait times out
m = re.search(r'petey: f(\d+) test hit b 66', t)
w = [x for x in re.finditer(r'petey: f(\d+) status \d+ -> ([23]) ', t) if m and x.start() > m.start()]
a = re.search(r'petey: f(\d+) status [23] -> 6 ', t)
d = int(a.group(1)) - int(w[0].group(1)) if w and a else None
ok = d is not None and 0 <= d <= 2
print(('ok  ' if ok else 'FAIL'), 'death: roar', d, 'frames after the first wait entered after the 50% hit (interrupts the wait)')
sys.exit(0 if ok else 1)
PY
      order death "25% -> phase 2 (L 4.1)" 'test hit b 39' 'phase 2 \(L 4.1\)'
      order death "under 10%, then a hit: status 15, stage 1, lower cage freed, KO" 'test hit b 22' 'test hit b 3' 'status [0-9]+ -> 15 ' 'stage event 1$' 'sound 8837' 'stage event 100[01]' 'defeated \(cause 1'
      never death 'status 15 -> 1 ' "no Down after death (crosscheck 4)"
      ;;
    break)
      play break 1400 MELEE_PETEY_TEST_HITS="300:r:200"
      order break "cage hit to 0 -> 0x13FF -> stage 1001 -> Down (rule 10) -> KO" 'test hit r 200' 'stage event 1001' 'status [0-9]+ -> 1 ' 'defeated \(cause 2'
      never break 'status [0-9]+ -> 15 ' "no threshold death (rules 6/7 are off: crosscheck 6)"
      ;;
    bodybreak)
      play bodybreak 1400 MELEE_PETEY_TEST_HITS="300:b:200"
      order bodybreak "body hit to 0 -> forwarded -> stage 1000 -> Down -> KO" 'test hit b 200' 'stage event 1000' 'status [0-9]+ -> 1 ' 'defeated \(cause 2'
      ;;
    sync)
      exe_banner "geno_boss_check sync"
      mkdir -p "$out/save-sync"
      ( cd "$out" && base_env sync MELEE_PETEY_TEST_HITS="$DEATH" timeout 3000 "$exe" synctest --engine tu --iso "$iso" \
          --scenario pad_falcon_fox_fd --depth "${DEPTH:-4}" --frames "${FRAMES:-3000}" --timeout 2800 > "$out/sync-${DEPTH:-4}.log" 2>&1 )
      line=$(grep 'engine synctest' "$out/sync-${DEPTH:-4}.log" | tail -1)
      case "$line" in *" 0 MISMATCHES"*) echo "ok   sync depth ${DEPTH:-4}: ${line#melee: }" ;; *) echo "FAIL sync depth ${DEPTH:-4}: ${line:-no result line}"; fail=1 ;; esac
      grep -q 'petey: geno actor' "$out/sync-${DEPTH:-4}.log" && echo "ok   sync: the Geno Petey fight ran" || { echo "FAIL sync: no Geno Petey in the run"; fail=1; }
      ;;
    *) echo "geno_boss_check: unknown row $r"; fail=1 ;;
  esac
done
[ $fail = 0 ] && echo "GENO BOSS CHECK OK" || echo "GENO BOSS CHECK FAILED"
exit $fail
