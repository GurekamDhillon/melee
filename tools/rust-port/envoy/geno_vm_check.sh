#!/bin/bash
# Geno VM bosses (geno_vm.rs, petey.rs vm_frame): `MELEE_VS_BOSS=<boss>` (default rayquaza) runs the boss's own enemy
# scripts, compiled from the player's local Brawl files, on Fox's slot of pad_falcon_fox_fd; Falcon walks to him and
# jabs (P1 pad script, Fox idle). Headless, hidden window, volume 3.
#
#   EXE=<melee exe> [BOSS=rayquaza] [OUT=dir] bash tools/rust-port/envoy/geno_vm_check.sh [fight sync nhv1 perf]
#
# Rows:
#   fight  the scripts compiled, the actor began, its statuses change, the jabs reach it (VM damage), its hitboxes arm
#   weapons  projectiles (650E): spawned, ending, reaching a player (BOSS=galleom / duon)
#   root   Ridley and Rayquaza: their models load and the clips' root translation moves them
#   crawl  Glunder on the VM: its surface queries let it crawl along FD's floor
#   sync   synctest depth 4 and 7, 3000 frames, the fight schedule: 0 mismatches, the VM inside the run
#   nhv1   the fight schedule twice through `golden`: identical NHV1 rows
#   perf   real-time play run (Falcon vs a level-1 CPU Fox carrier, FD), MELEE_PROFILE=1: zero_hitch_check
set -u
here=$(cd "$(dirname "$0")" && pwd)
repo=$(cd "$here/../../.." && pwd)
. "$here/exe_check.sh"; exe_pick geno_vm_check
out=${OUT:-$repo/build/geno-vm-check}
boss=${BOSS:-rayquaza}
iso=${GW_ISO_VANILLA:-}
[ -n "$iso" ] || { echo "geno_vm_check: GW_ISO_VANILLA is not set"; exit 2; }
rows=("$@"); [ ${#rows[@]} -gt 0 ] || rows=(fight crawl sync nhv1 perf)
mkdir -p "$out"
{ printf '420 0000 %s 0\n' "${WALK_X:-60}"; for i in $(seq 1 400); do printf '2 0100 0 0\n10 0000 0 0\n'; done; } > "$out/fight.txt"
fail=0
base() { # name pad cmd...
  local name=$1 pad=$2; shift 2
  ( cd "$repo" && env MELEE_VOLUME=3 MELEE_WINDOW_HIDDEN=1 MELEE_FULLSCREEN=off MELEE_WINDOW_UNFOCUSED=1 MELEE_PAD_IGNORE_ADAPTER=1 \
      MELEE_VS_BOSS="$boss" MELEE_VS_BOSS_SLOT=1 MELEE_PETEY_LOG=1 MELEE_PAD_SCRIPT="$out/$pad" \
      MELEE_WINDOW_TITLE="genovm geno_vm_check $boss $name" "$@" ) > "$out/$name.log" 2>&1
}
golden() { base "$1" "$2" timeout 900 "$exe" golden --engine tu --iso "$iso" --scenario pad_falcon_fox_fd --frames "${FRAMES:-3000}" --out "$out/$1.csv"; }
L() { echo "$out/$1.log"; }
need() { if grep -q -E "$2" "$(L "$1")"; then echo "ok   $1: $3"; else echo "FAIL $1: $3 (no /$2/)"; fail=1; fi; }
for r in "${rows[@]}"; do
  exe_banner "geno_vm_check $r"
  case "$r" in
    fight)
      golden fight fight.txt
      need fight "geno_vm: $boss: scripts compiled" "the scripts compiled from the player's files"
      need fight "petey: vm actor $boss" "the VM actor began"
      need fight 'petey: f[0-9]+ status -?[0-9]+ -> [0-9]+ px' "its statuses change"
      n=$(grep -c -E 'petey: f[0-9]+ status' "$(L fight)"); echo "     $n status changes"
      need fight 'hp [0-9.]+$' "status lines carry the VM's HP"
      grep -o -E 'hp [0-9.]+$' "$(L fight)" | sort -u | head -3 | sed 's/^/     /'
      need fight 'vm hitbox [0-9]+ on' "the boss's forwarded hitbox commands arm attack boxes"
      if grep -q "vm_boss_model: $boss: model loaded" "$(L fight)"; then
        python - "$(L fight)" <<'PY' || fail=1
import re, sys
v = [(float(a), float(b), float(c), float(d)) for a, b, c, d in re.findall(r'on bone -?\d+ at (-?[0-9.]+) (-?[0-9.]+) \(actor (-?[0-9.]+) (-?[0-9.]+)\)', open(sys.argv[1], errors='replace').read())]
off = [abs(x - ax) + abs(y - ay) for x, y, ax, ay in v]
ok = len(v) > 0 and max(off) > 1
print(('ok  ' if ok else 'FAIL'), f'fight: {len(v)} attack boxes placed on their bones (max {max(off) if off else 0:.1f} from the actor)')
sys.exit(0 if ok else 1)
PY
      fi
      ;;
    root)
      # root motion: the boss's model loads and its clips' root translation moves it (Input::root_move)
      for b in ${ROOT_BOSSES:-ridley rayquaza}; do
        BOSS_SAVE=$boss; boss=$b; golden "root-$b" fight.txt; boss=$BOSS_SAVE
        need "root-$b" "vm_boss_model: $b: model loaded" "$b: model and clips loaded"
        python - "$(L root-$b)" "$b" <<'PY' || fail=1
import re, sys
t = open(sys.argv[1], errors='replace').read()
tr = [float(m.group(1)) for m in re.finditer(r'vm root travel ([0-9.]+)', t)]
xs = [float(m.group(1)) for m in re.finditer(r'status -?\d+ -> \d+ px (-?[0-9.]+)', t)]
ok = len(tr) >= 2 and max(tr) > 5 and xs and max(xs) - min(xs) > 10
print(('ok  ' if ok else 'FAIL'), f'root-{sys.argv[2]}: {len(tr)} statuses moved by root motion (max {max(tr) if tr else 0:.1f}), x range {min(xs) if xs else 0:.1f}..{max(xs) if xs else 0:.1f}')
sys.exit(0 if ok else 1)
PY
      done
      ;;
    weapons)
      # projectiles (650E weapons, geno_boss_spawn via geno_vm_host): the fight schedule run long enough for the
      # boss's shooting attacks; spawned, a player touched or a hit landing, and every projectile removed by its owner
      golden weapons fight.txt
      need weapons "geno_boss: $boss: [1-9][0-9]* weapon records" "the boss's weapon records loaded (local data)"
      need weapons 'vm weapon [0-9]+ joint -?[0-9]+ -> slot Some' "650E weapons spawned into the pool"
      need weapons 'spawn slot [0-9]+ (removed|explodes)' "projectiles end (explode / removed)"
      need weapons 'vm weapon [0-9]+ joint -?[0-9]+ -> slot Some\([0-9]+\) at .* on its bone' "projectiles start on the script's bone (model loaded)"
      need weapons 'vm weapon slot [0-9]+ touches player|hit lands' "a projectile or the boss reached a player"
      s=$(grep -c 'vm weapon [0-9]* joint' "$(L weapons)"); e=$(grep -c 'spawn slot [0-9]* removed' "$(L weapons)")
      echo "     $s spawned, $e removed, $(grep -c 'touches player' "$(L weapons)") touches, $(grep -c 'hit lands' "$(L weapons)") hits landed"
      ;;
    crawl)
      # Glunder (an enemy run by the VM): the surface queries let it crawl its floor and turn at the end
      # an enemy's entry status is its stage placement's (not in the IR): Fall, so it lands and starts its loop
      BOSS_SAVE=$boss; boss=glunder; MELEE_VS_BOSS_STATUS=0 golden crawl idle.txt; boss=$BOSS_SAVE
      need crawl 'geno_vm: glunder: scripts compiled' "Glunder's scripts compiled"
      python - "$(L crawl)" <<'PY' || fail=1
import re, sys
xs = [float(m.group(1)) for m in re.finditer(r'petey: f\d+ status -?\d+ -> \d+ px (-?[0-9.]+)', open(sys.argv[1], errors='replace').read())]
ok = len(xs) >= 3 and max(xs) - min(xs) > 10
print(('ok  ' if ok else 'FAIL'), f'crawl: {len(xs)} status changes, x range {min(xs) if xs else 0:.1f}..{max(xs) if xs else 0:.1f} (it moves along its floor)')
sys.exit(0 if ok else 1)
PY
      ;;
    sync)
      for depth in 4 7; do
        base "sync$depth" fight.txt timeout 3000 "$exe" synctest --engine tu --iso "$iso" --scenario pad_falcon_fox_fd \
          --depth "$depth" --frames "${FRAMES:-3000}" --timeout 2800
        line=$(grep 'engine synctest' "$(L sync$depth)" | tail -1)
        case "$line" in *" 0 MISMATCHES"*) echo "ok   sync depth $depth: ${line#melee: }" ;; *) echo "FAIL sync depth $depth: ${line:-no result line}"; fail=1 ;; esac
        need "sync$depth" "petey: vm actor $boss" "the VM inside the synctest"
      done
      ;;
    nhv1)
      golden nhv1a fight.txt; golden nhv1b fight.txt
      if cmp -s "$out/nhv1a.csv" "$out/nhv1b.csv" && [ -s "$out/nhv1a.csv" ]; then
        echo "ok   nhv1: identical over two runs ($(wc -l < "$out/nhv1a.csv") rows, last $(tail -1 "$out/nhv1a.csv" | cut -d, -f1-3))"
      else echo "FAIL nhv1: the two runs differ"; fail=1; fi
      ;;
    perf)
      rm -rf "$out/perf"; mkdir -p "$out/perf"; cp "$exe" "$out/perf/melee.exe"
      ( cd "$out/perf" && env MELEE_VOLUME=3 MELEE_WINDOW_HIDDEN=1 MELEE_FULLSCREEN=off MELEE_WINDOW_UNFOCUSED=1 MELEE_PAD_IGNORE_ADAPTER=1 \
          MELEE_VS_BOSS="$boss" MELEE_PETEY_LOG=1 MELEE_PAD_SCRIPT="$out/fight.txt" MELEE_PROFILE=1 MELEE_WINDOW_TITLE="genovm geno_vm_check $boss perf" \
          timeout 600 ./melee.exe play --iso "$iso" --max-frames "${FRAMES:-3000}" --match "falcon,fox/cpu1,FINAL DESTINATION" --time-secs 600 ) > "$out/perf.log" 2>&1
      cp "$out/perf.log" "$out/perfrun.log"; need perfrun "petey: vm actor $boss" "the VM ran in the real-time run"
      p=$(ls -t "$out/perf/profiles/"profile-*.csv 2>/dev/null | head -1)
      if [ -n "$p" ]; then python "$repo/tools/rust-port/perf/zero_hitch_check.py" "$p" --label geno-vm > "$out/hitch.txt"; [ $? = 0 ] || fail=1; head -12 "$out/hitch.txt"
      else echo "FAIL perf: no profile written"; fail=1; fi
      rm -f "$out/perf/melee.exe"
      ;;
    *) echo "geno_vm_check: unknown row $r"; fail=1 ;;
  esac
done
[ $fail = 0 ] && echo "GENO VM CHECK OK" || echo "GENO VM CHECK FAILED"
exit $fail
