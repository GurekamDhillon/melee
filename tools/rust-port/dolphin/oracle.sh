#!/bin/bash
# SPDX-License-Identifier: MIT
# Dolphin oracle: one .slp through the patched Slippi Dolphin AND the Rust engine, compared frame by frame.
# docs/rust-port/notes/dolphin-oracle.md has the build, the hook and the record format.
#
#   bash tools/rust-port/dolphin/oracle.sh REPLAY.slp OUT_DIR [--wide] [--window N] [--at SLIPPI_FRAME] [--zero-eq]
#   --at F: dump and diff the window ending at slippi frame F instead of the first NHV1 divergence (e.g. slpcheck's
#   first divergent frame); --zero-eq: wide_diff treats f32 +0/-0 as equal
#
#   1. Dolphin (run_dolphin.ps1): headless, Null video, no audio, no window, no OS input; plays the replay in Slippi
#      playback mode and dumps GameCube memory after HSD_GObj_RunProcs every frame -> OUT_DIR/dolphin.nhraw
#   2. raw2csv.py -> OUT_DIR/dolphin.csv (the NHV1 words and hash of docs/specs/tools/nhv1_ref.py)
#   3. melee slpcheck --full --nhv1-out -> OUT_DIR/rust.csv
#   4. compare_golden.py rust.csv dolphin.csv --ignore-seed --allow-short (nhv1.md 8: a replay forces the seed)
#   5. on a mismatch at CSV frame F: by-name dumps of slippi frames F-123-N .. F-123 from both sides
#      (melee dolphin-wide / slpcheck --wide-out; --wide adds game_camera and stage_info) and wide_diff.py, which names
#      the fields that first differ.
# Last line: `ORACLE <file> match (<n> frames)` or `ORACLE <file> diverged at frame F (slippi F-123): <columns>; first
# field(s): ...`, plus "; the Dolphin runs disagree from frame F" when the two Dolphin runs differ. Env: MELEE_EXE (default build/bin/melee.exe, then the shared fast target), KEEP=1 keeps the raw dump.
set -u
SLP=$1; OUT=$2; shift 2
WIDE=0; WINDOW=3; AT=""; ZEQ=""
while [ $# -gt 0 ]; do
  case $1 in
    --wide) WIDE=1 ;;
    --window) WINDOW=$2; shift ;;
    --at) AT=$2; shift ;;
    --zero-eq) ZEQ=--zero-eq ;;
  esac
  shift
done
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
root="$(cd "$here/../../.." && pwd)"
ws="${GW_WORKSPACE:-E:/Projects/Melee Workspace}"
if [ -n "${MELEE_EXE:-}" ] && [ ! -x "$MELEE_EXE" ]; then echo "ORACLE error: MELEE_EXE is set but not an executable: $MELEE_EXE" >&2; exit 2; fi
EXE=${MELEE_EXE:-$root/build/bin/melee.exe}
[ -x "$EXE" ] || EXE="$ws/_build/cargo-shared/fast/fast/melee.exe"
set -a; [ -f "$ws/.env" ] && . "$ws/.env" >/dev/null 2>&1; set +a
mkdir -p "$OUT"
OUTW=$(cygpath -w "$OUT" 2>/dev/null || echo "$OUT")
SLPW=$(cygpath -w "$SLP" 2>/dev/null || echo "$SLP")
name=$(basename "$SLP")
# the disc both sides play: an Akaneia replay with one of the disc's m-ex fighters runs on GW_ISO_AKANEIA (slpcheck's own
# rule, replay_codes.py --disc; DOLPHIN_DISC=vanilla|akaneia|ace overrides), every other replay on GW_ISO_VANILLA
DISC=${DOLPHIN_DISC:-$(python -I "$here/replay_codes.py" --disc "$SLP")}
export DOLPHIN_DISC=$DISC
ISO=$GW_ISO_VANILLA; [ "$DISC" = akaneia ] && ISO=$GW_ISO_AKANEIA; [ "$DISC" = ace ] && ISO=$GW_ISO_ACE
log() { echo "oracle: $*"; }

wflag=""; [ $WIDE = 1 ] && wflag="-Wide"
# Dolphin runs twice, concurrently (DOLPHIN_RUNS=1 for once): the playback is occasionally nondeterministic
# (notes/dolphin-oracle.md, "Nondeterminism"), and a frame where the two runs disagree is no evidence against Rust.
RUNS=${DOLPHIN_RUNS:-2}
powershell -NoProfile -ExecutionPolicy Bypass -File "$(cygpath -w "$here/run_dolphin.ps1")" -Slp "$SLPW" -Out "$OUTW" $wflag \
  | sed 's/^/oracle: /' &
if [ "$RUNS" -ge 2 ]; then
  mkdir -p "$OUT/run2"
  powershell -NoProfile -ExecutionPolicy Bypass -File "$(cygpath -w "$here/run_dolphin.ps1")" -Slp "$SLPW" \
    -Out "$(cygpath -w "$OUT/run2" 2>/dev/null || echo "$OUT/run2")" > /dev/null 2>&1 &
fi
wait
[ -s "$OUT/dolphin.nhraw" ] || { echo "ORACLE $name error: Dolphin wrote no records (see $OUT/dolphin.stderr.txt)"; exit 2; }
python -I "$here/raw2csv.py" "$OUT/dolphin.nhraw" "$OUT/dolphin.csv" || { echo "ORACLE $name error: raw2csv"; exit 2; }
NONDET=""
if [ "$RUNS" -ge 2 ]; then
  if python -I "$here/raw2csv.py" "$OUT/run2/dolphin.nhraw" "$OUT/run2/dolphin.csv" > /dev/null 2>&1; then
    rm -f "$OUT/run2/dolphin.nhraw"
    if ! python "$root/tools/rust-port/compare_golden.py" "$OUT/run2/dolphin.csv" "$OUT/dolphin.csv" --allow-short --max-report 1 \
        > "$OUT/run2/compare_runs.txt" 2>&1; then
      NONDET=$(grep -m1 -o 'first divergence at frame -\?[0-9]*' "$OUT/run2/compare_runs.txt" | grep -o -- '-\?[0-9]*$')
      NONDET=${NONDET:-?}
      log "DOLPHIN NONDETERMINISTIC: the two Dolphin runs disagree from frame $NONDET (run2/compare_runs.txt)"
    else
      log "the two Dolphin runs agree"
    fi
  else
    log "the second Dolphin run wrote no records; one run only"
  fi
fi

MELEE_WINDOW_UNFOCUSED=1 MELEE_VOLUME=3 "$EXE" slpcheck --slp "$SLP" --full --iso "$ISO" \
  --nhv1-out "$OUT/rust.csv" --csv "$OUT/slpcheck.csv" > "$OUT/rust.log" 2>&1
grep -a '^SLPCHECK' "$OUT/rust.log" | sed 's/^/oracle: /'
[ -s "$OUT/rust.csv" ] || { echo "ORACLE $name error: the Rust run wrote no rows (see $OUT/rust.log)"; exit 2; }

python "$root/tools/rust-port/compare_golden.py" "$OUT/rust.csv" "$OUT/dolphin.csv" --ignore-seed --allow-short --max-report 1 \
  > "$OUT/compare.txt" 2>&1
rc=$?
python "$root/tools/rust-port/compare_golden.py" "$OUT/rust.csv" "$OUT/dolphin.csv" --allow-short --print-count --max-report 0 \
  > "$OUT/compare_seed.txt" 2>&1
sed 's/^/oracle: /' "$OUT/compare.txt"
log "with seed/hash: $(tail -1 "$OUT/compare_seed.txt")"
nrows=$(($(wc -l < "$OUT/rust.csv") - 1))
if [ $rc = 0 ]; then
  echo "ORACLE $name match ($nrows Rust frames equal to Dolphin, seed ignored)${NONDET:+; the Dolphin runs disagree from frame $NONDET}"
else
  F=$(grep -m1 -o 'first divergence at frame -\?[0-9]*' "$OUT/compare.txt" | grep -o -- '-\?[0-9]*$')
  cols=$(grep -m1 '^frame .*differs in' "$OUT/compare.txt" | sed 's/.*differs in //')
  if [ -z "$F" ]; then
    echo "ORACLE $name error: $(tail -1 "$OUT/compare.txt")"
    exit 1
  fi
  S=$((F - 123)); [ -n "$AT" ] && S=$AT
  A=$((S - WINDOW)); [ $A -lt -123 ] && A=-123
  # no earlier frame to take the baseline from: list every difference
  [ $A = $S ] && ZEQ="$ZEQ --persistent"
  ww=""; [ $WIDE = 1 ] && ww="--wide"
  "$EXE" dolphin-wide --raw "$OUT/dolphin.nhraw" --out "$OUT/dolphin_wide.txt" --from $A --to $S 2>/dev/null
  MELEE_WINDOW_UNFOCUSED=1 MELEE_VOLUME=3 "$EXE" slpcheck --slp "$SLP" --full --iso "$ISO" $ww \
    --wide-out "$OUT/rust_wide.txt" --wide-from $A --wide-to $S > /dev/null 2>&1
  python "$here/wide_diff.py" "$OUT/rust_wide.txt" "$OUT/dolphin_wide.txt" --all-frames --max 12 $ZEQ > "$OUT/wide_diff.txt"
  sed 's/^/oracle: /' "$OUT/wide_diff.txt"
  # the new differences on the divergent frame itself (wide_diff lists earlier frames' new differences above it:
  # fields outside NHV1 that drifted first, e.g. a 1-ULP ECB value)
  at=$(awk -v f="frame $S " 'index($0,f)==1{on=1;next} /^frame /{on=0} on && /^    [a-z]/{print $1"."$2}' "$OUT/wide_diff.txt" | head -4 | tr '\n' ' ')
  early=$(grep -m1 'new difference' "$OUT/wide_diff.txt" | awk '{print $2}')
  echo "ORACLE $name diverged at frame $F (slippi $((F - 123))): $cols; window ending at slippi $S, new at that frame: ${at:-none (see wide_diff.txt)}; earliest new difference in the window: slippi ${early:-none}${NONDET:+; the Dolphin runs disagree from frame $NONDET}"
fi
[ "${KEEP:-0}" = 1 ] || rm -f "$OUT/dolphin.nhraw"
exit $rc
