#!/bin/bash
# UNTESTED (written on Linux without his host; first run on the PC). The core v1 shot gate of
# docs/rust-port/notes/hero-renderer-plan.md section 5: the same scenes through his renderer (our engine as melee_game.dll
# in his Source Port host, `backend vulkan` by default) and through ours (melee.exe, wgpu), registered and scored by
# compare_shots.py, one table row per shot.
#   HOST_EXE=<melee_source.exe> HOST_DIR=<his checkout root> bash tools/rust-port/corev1/gate_shots.sh <out> [set ...]
# Sets: stages (the view-golden stages FD BF YS GG FoD DL64, a CPU match on each) and menus (the retail_full.txt walk
# through the title, menus, CSS and a match at fixed ticks); default both.
# Environment: HOST_BACKEND (default vulkan; d3d12, d3d11 for comparison rows), STAGE_SHOTS (default 120,300,600: match
# ticks), MENU_SHOTS (default 150,400,700,1000,1300,1600,1900: retail_full ticks), HOST_LAG (default 4: his XFB-copy count
# sits a few below our tick, notes/core-v1.md), plus everything run.sh reads (GW_ISO_VANILLA, CARGO_TARGET_DIR, PROFILE).
# Bar (section 5): mean abs <= 2.5/255 and >= 97% of pixels within 16 per shot. Writes <out>/table.md.
set -u
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
out=$1; shift
sets=("$@"); [ ${#sets[@]} -eq 0 ] && sets=(stages menus)
: "${HOST_EXE:?set HOST_EXE to his melee_source.exe}" "${HOST_DIR:?set HOST_DIR to his checkout root}"
backend="${HOST_BACKEND:-vulkan}"
lag="${HOST_LAG:-4}"
mkdir -p "$out"
table="$out/table.md"
echo "| set | scene | shot tick | his ($backend) vs ours | mean abs /255 | PSNR dB | % px <= 16 | % px > 24 | bar |" >"$table"
echo "|---|---|---|---|---|---|---|---|---|" >>"$table"

# ref ticks for one shot: compare_shots.py picks the closest of our ticks within 15 of his
ranges() { local r=() n; for n in ${1//,/ }; do r+=("$((n - 15 < 1 ? 1 : n - 15))-$((n + 15))"); done; local IFS=,; echo "${r[*]}"; }

# one scene: our shots, then one host run per shot, then the scores. $1 set, $2 scene name, $3 shot ticks, then the
# environment for both runs (MATCH=..., MELEE_PAD_SCRIPT=...) as NAME=VALUE words
scene() {
  local set=$1 name=$2 shots=$3; shift 3
  local d="$out/$set/$name" last=0 n
  for n in ${shots//,/ }; do [ "$n" -gt "$last" ] && last=$n; done
  local ticks=$((last + 60))
  mkdir -p "$d/runs"
  echo "== $set/$name: ours, ticks $(ranges "$shots")"
  env "$@" bash "$here/run.sh" ref "$d/ref" "$ticks" "$(ranges "$shots")" | tail -1
  for n in ${shots//,/ }; do
    echo "== $set/$name: his $backend, sim frame $((n - lag)) (our tick $n)"
    env "$@" HOST_SETTINGS="backend $backend" bash "$here/run.sh" host "$d/runs/h$n" "$ticks" \
      --capture "$d/runs/h$n/cap/f.ppm" --capture-sim-frame "$((n - lag))" | tail -1
  done
  python "$here/compare_shots.py" "$d/runs" "$d/ref" | tee "$d/scores.txt"
  # seq N -> ref tick T: crop [...], mean abs M/255, PSNR P dB, B% px > 24, W% px <= 16
  sed -nE 's/^seq ([0-9]+) -> ref tick ([0-9]+): .*mean abs ([0-9.]+)\/255, PSNR ([0-9.inf-]+) dB, ([0-9.]+)% px > 24, ([0-9.]+)% px <= 16$/\1 \2 \3 \4 \5 \6/p' \
    "$d/scores.txt" | while read -r seq t m p big near; do
      ok=$(awk -v m="$m" -v w="$near" 'BEGIN { print (m <= 2.5 && w >= 97) ? "pass" : "fail" }')
      echo "| $set | $name | $seq | tick $t | $m | $p | $near | $big | $ok |" >>"$table"
    done
}

for s in "${sets[@]}"; do
  case $s in
    stages)
      # the view-golden stages (notes/render.md: pad_falcon_fox_{fd,bf,ys,gr,fod,dl64}) by external stage id
      # (docs/rust-port/golden/coverage/scenarios.tsv), Falcon v Fox as CPUs: no pad script under either renderer
      for st in fd:32 bf:31 ys:8 gg:17 fod:2 dl64:28; do
        scene stages "${st%%:*}" "${STAGE_SHOTS:-120,300,600}" MATCH="falcon/cpu9,fox/cpu9,${st##*:}" MELEE_PAD_SCRIPT=
      done ;;
    menus)
      # run.sh's default script (tools/rust-port/play/retail_full.txt): title, main menu, VS, CSS, SSS, a match
      scene menus retail_full "${MENU_SHOTS:-150,400,700,1000,1300,1600,1900}" ;;
    *) echo "gate_shots.sh: unknown set $s (stages | menus)" >&2; exit 2 ;;
  esac
done
echo; cat "$table"
fails=$(grep -c '| fail |$' "$table"); rows=$(($(wc -l <"$table") - 2))
echo "gate_shots: $rows shots, $fails below the bar ($table)"
[ "$rows" -gt 0 ] && [ "$fails" -eq 0 ]
