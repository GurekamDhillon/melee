#!/bin/bash
# Run `melee slpcheck` over a directory of .slp files, one process per file (a crash only loses that file).
# usage: run_all.sh REPLAY_DIR OUT_DIR [extra slpcheck args...]   (env: JOBS=4, TIMEOUT=900, MELEE_EXE)
# OUT_DIR gets one <file>.csv (summary row) and <file>.log per replay; all.csv concatenates the rows.
# Headless: no window is opened by the engine; nothing here sends OS input.
set -u
DIR=$1; OUT=$2; shift 2
EXE=${MELEE_EXE:-build/cargo/release/melee.exe}
[ -e "$EXE" ] || [ ! -e "${EXE%.exe}" ] || EXE="${EXE%.exe}"
JOBS=${JOBS:-4}; TIMEOUT=${TIMEOUT:-900}
mkdir -p "$OUT"
export MELEE_WINDOW_UNFOCUSED=1
set -a; [ -f "../../.env" ] && . ../../.env; [ -f ".env" ] && . ./.env; set +a
one() {
  f=$1; shift; b=$(basename "$f" .slp)
  rm -f "$OUT/$b.csv"
  timeout "$TIMEOUT" "$EXE" slpcheck --slp "$f" --csv "$OUT/$b.csv" "$@" > "$OUT/$b.log" 2>&1
  rc=$?
  if [ ! -s "$OUT/$b.csv" ]; then
    # no summary row: the process died (panic outside the guard, fault, timeout)
    echo "$b.slp,crash-rc$rc,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,," > "$OUT/$b.csv"
  fi
}
export -f one; export OUT EXE TIMEOUT
ls "$DIR"/*.slp | xargs -P "$JOBS" -I{} bash -c 'one "$@"' _ {} "$@"
first=1
: > "$OUT/all.csv"
for c in "$OUT"/*.csv; do
  [ "$(basename "$c")" = all.csv ] && continue
  if [ $first = 1 ]; then head -1 "$c" | grep -q '^file,' && head -1 "$c" > "$OUT/all.csv"; first=0; fi
  grep -v '^file,' "$c" >> "$OUT/all.csv"
done
echo "done: $(grep -vc '^file,' "$OUT/all.csv") rows in $OUT/all.csv"
