#!/bin/bash
# Petey as a Geno actor (docs/specs/geno-v5.2-boss.md, docs/rust-port/notes/geno-boss.md). Needs the owner's local
# boss IR (MELEE_BOSS_IR_DIR / MELEE_BRAWL_LOCAL/boss-ir / _build/boss-ir) and Brawl files; headless, scripted hits.
#
#   EXE=<melee exe> MODS=<dir with envoy_sim.wasm> [OUT=dir] [PLAYER=kirby] \
#     bash tools/rust-port/envoy/geno_boss_check.sh [idle death break bodybreak sync]
#
# Rows:
#   idle       no hits: the IR's attack loop (2 -> 4 -> 11/12 -> 3 -> 5 -> ...), wait lengths in range at Normal
#   death      body hits past the phase-1 threshold (Angry at the next wait evaluation, not at the wait's end), phase 2,
#              the death threshold, then one more hit: status 15 (threshold-then-hit), stage events 1 and 1000/1001, no Down, KO
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
# expected values: the owner's local file (never committed; from his Brawl files / IR)
expect=${GENO_EXPECT:-}
for d in "${MELEE_BOSS_IR_DIR:-}" "${MELEE_BRAWL_LOCAL:+$MELEE_BRAWL_LOCAL/boss-ir}" "$repo/../_build/boss-ir" "$repo/../../_build/boss-ir"; do
  [ -z "$expect" ] && [ -n "$d" ] && [ -f "$d/checks/geno-checks.env" ] && expect="$d/checks/geno-checks.env"
done
[ -n "$expect" ] || { echo "$(basename "$0"): no local expected values (boss-ir/checks/geno-checks.env)"; exit 2; }
. "$expect"
# the death schedule: hits past each phase threshold, then one under the death threshold
DEATH="$PETEY_DEATH"
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
      need idle "fight begins, cages $PETEY_CAGES difficulty 5 L $PETEY_L0" "cages from the IR's HP formula at Normal"
      order idle "attack loop" 'status 2 -> 4 ' 'status 4 -> 1[12] ' 'status 1[12] -> 3 ' 'status 3 -> 5 '
      python - "$(L idle)" $PETEY_WAITA $PETEY_WAITB <<'PY' || fail=1
import re, sys
ent = [(int(m.group(1)), int(m.group(2)), int(m.group(3))) for m in re.finditer(r'petey: f(\d+) status (-?\d+) -> (\d+)', open(sys.argv[1], errors='replace').read())]
wa = [f1 - f0 for (f0, a, b), (f1, c, d) in zip(ent, ent[1:]) if b == 2 and d == 4]
wb = [f1 - f0 for (f0, a, b), (f1, c, d) in zip(ent, ent[1:]) if b == 3 and d == 5]
a0, a1, b0, b1 = map(int, sys.argv[2:6])
bad = not wa or min(wa) < a0 or max(wa) > a1 or not wb or min(wb) < b0 or max(wb) > b1
print(('FAIL' if bad else 'ok  '), 'idle: Wait A', sorted(set(wa)), f'in {a0}..{a1}; Wait B', sorted(set(wb)), f'in {b0}..{b1}')
sys.exit(1 if bad else 0)
PY
      ;;
    death)
      play death 2400 MELEE_PETEY_TEST_HITS="$DEATH"
      order death "phase 1: Angry from the wait, its L" "test hit b $PETEY_HIT1" "phase 1 \\(L $PETEY_L1\\)" 'status 6 -> [45] '
      python - "$(L death)" "$PETEY_HIT1" <<'PY' || fail=1
import re, sys
t = open(sys.argv[1], errors='replace').read()
# crosscheck 2: rule 28 fires at the first evaluation in status 2/3 after 0x1400, not when the wait times out
m = re.search(r'petey: f(\d+) test hit b ' + sys.argv[2], t)
w = [x for x in re.finditer(r'petey: f(\d+) status \d+ -> ([23]) ', t) if m and x.start() > m.start()]
a = re.search(r'petey: f(\d+) status [23] -> 6 ', t)
d = int(a.group(1)) - int(w[0].group(1)) if w and a else None
ok = d is not None and 0 <= d <= 2
print(('ok  ' if ok else 'FAIL'), 'death: roar', d, 'frames after the first wait entered after the phase-1 hit (interrupts the wait)')
sys.exit(0 if ok else 1)
PY
      order death "phase 2, its L" "test hit b $PETEY_HIT2" "phase 2 \\(L $PETEY_L2\\)"
      order death "under the death threshold, then a hit: status 15, stage 1, lower cage freed, KO" "test hit b $PETEY_HIT3" "test hit b $PETEY_HIT4" 'status [0-9]+ -> 15 ' 'stage event 1$' "sound $PETEY_DEATH_SOUND" 'stage event 100[01]' 'defeated \(cause 1'
      never death 'status 15 -> 1 ' "no Down after death (crosscheck 4)"
      ;;
    break)
      play break 1400 MELEE_PETEY_TEST_HITS="300:r:$PETEY_BREAK_HIT"
      order break "cage hit to 0 -> 0x13FF -> stage 1001 -> Down (rule 10) -> KO" "test hit r $PETEY_BREAK_HIT" 'stage event 1001' 'status [0-9]+ -> 1 ' 'defeated \(cause 2'
      never break 'status [0-9]+ -> 15 ' "no threshold death (rules 6/7 are off: crosscheck 6)"
      ;;
    bodybreak)
      play bodybreak 1400 MELEE_PETEY_TEST_HITS="300:b:$PETEY_BREAK_HIT"
      order bodybreak "body hit to 0 -> forwarded -> stage 1000 -> Down -> KO" "test hit b $PETEY_BREAK_HIT" 'stage event 1000' 'status [0-9]+ -> 1 ' 'defeated \(cause 2'
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
