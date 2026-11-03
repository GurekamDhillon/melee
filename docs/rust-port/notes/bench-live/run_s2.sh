#!/bin/bash
# usage: run_u2.sh OUTDIR SCRIPT FRAMES [extra]  (benchmark configuration of melee-unlocked melee_source.exe)
set -a; source "/e/Projects/Melee Workspace/.env"; set +a
cd /e/Projects/bench-live-scratch
OUT=$1; SCRIPT=$2; FRAMES=$3; shift 3
U="/e/Projects/Melee Workspace/scratch/ext/melee-unlocked"
mkdir -p "$OUT/cards" "$OUT/User"; cp u_settings.template "$OUT/port-settings.ini"
W=$(cygpath -w "$PWD/$OUT")
python bench.py --out $OUT --cwd "$(cygpath -w "$U")" --env MELEE_PRESENT_UNCAPPED=1 --env MELEE_SIM_TIMES="$W/sim.csv" -- \
  "$(cygpath -w "$U/build-review/port/Release/melee_source.exe")" --iso "$(cygpath -w "$GW_ISO_VANILLA")" --volume 0 --no-music --hidden \
  --threaded-renderer --fps unlocked --frame-mode interpolate --time-base 1 --window 1280x960 --scale 2 --dlss off --ssaa 1 --sharpness 0 \
  --frames "$FRAMES" --script "$(cygpath -w "$PWD/$SCRIPT")" --frame-times "$W/frames.csv" --log-file "$W/port.log" \
  --settings-path "$W/port-settings.ini" --load-settings --card-dir "$W/cards" --user-dir "$W/User" --replay-dir "$W/replays" "$@"
