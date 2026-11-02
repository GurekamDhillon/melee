#!/bin/bash
# usage: run_r.sh OUTDIR MATCHSPEC MAXFRAMES [extra play args]
set -a; source "/e/Projects/Melee Workspace/.env"; set +a
cd /e/Projects/bench-live-scratch
OUT=$1; SPEC=$2; MF=$3; shift 3
R=$(cygpath -w "/e/Projects/Melee Workspace/worktrees/bench2/rust/target/release/melee.exe")
W=$(cygpath -w $PWD)
mkdir -p $OUT
python bench.py --out $OUT --env MELEE_PAD_SCRIPT="$W/rust_idle.txt" --env MELEE_PAD_IGNORE_ADAPTER=1 --env MELEE_WINDOW_UNFOCUSED=1 \
  --env MELEE_VSYNC=off --env MELEE_INTERP=1 --env MELEE_RENDER_SCALE=2 --env MELEE_FRAME_TIMES="$W/$OUT/ft.csv" --env MELEE_SHOW_FPS=1 \
  -- "$R" play --iso "$(cygpath -w "$GW_ISO_VANILLA")" --match "$SPEC" --items OFF --stocks 4 --time-secs 480 --max-frames $MF "$@"
