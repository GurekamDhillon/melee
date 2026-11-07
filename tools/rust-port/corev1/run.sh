#!/bin/bash
# The engine as a core v1 game library (docs/rust-port/notes/core-v1.md), one scripted run in a core v1 host, or the
# same run in our own renderer for comparison. No OS input: MELEE_PAD_SCRIPT drives port 1.
#   HOST_EXE=<melee_source.exe> HOST_DIR=<its checkout root> bash tools/rust-port/corev1/run.sh host <out> <ticks> [host args]
#   bash tools/rust-port/corev1/run.sh ref <out> <ticks> <shot ticks, e.g. 590-615,1690-1715>
# Host captures: --capture <out>/cap/f.ppm --capture-sim-frame N (N = the host's XFB-copy count, a few below our tick).
# Then: python tools/rust-port/corev1/compare_shots.py <runs dir holding h<N>/ folders> <ref out>.
set -u
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"
ws="${GW_WORKSPACE:-E:/Projects/Melee Workspace}"
if [ -z "${GW_ISO_VANILLA:-}" ] && [ -f "$ws/.env" ]; then set -a; . "$ws/.env"; set +a; fi
mode=$1; out=$2; ticks=$3; shift 3
tgt="${CARGO_TARGET_DIR:-$ws/_build/cargo-shared/fast}/${PROFILE:-fast}"
rm -rf "$out"; mkdir -p "$out/cap" "$out/shots"
export MELEE_PAD_SCRIPT="${MELEE_PAD_SCRIPT:-$repo/tools/rust-port/play/retail_full.txt}"
export MELEE_VOLUME=3 MELEE_CARD=0 MELEE_NO_GC_ADAPTER=1 MELEE_PAD_IGNORE_ADAPTER=1 MELEE_WINDOW_UNFOCUSED=1
export MELEE_HASH_LOG="$out/hash.txt"
args="--skip-intro --time-secs 30 --quit-after-matches 1 --max-frames $ticks"
case $mode in
  host)
    export MELEE_WINDOW_TITLE="${MELEE_WINDOW_TITLE:-corev1: rust engine in a core v1 host: $(basename "$out")}"
    export MELEE_GAME_DLL="$tgt/melee_game.dll" MELEE_COREV1_ARGS="$args"
    (cd "$HOST_DIR" && "$HOST_EXE" --iso "$GW_ISO_VANILLA" --volume 3 --log-file "$out/host.log" "$@" >"$out/stdout.txt" 2>"$out/stderr.txt") ;;
  ref)
    export MELEE_WINDOW_TITLE="${MELEE_WINDOW_TITLE:-corev1: rust engine, own renderer: $(basename "$out")}"
    cp "$tgt/melee.exe" "$out/melee.exe"
    (cd "$out" && MELEE_RETAIL_SHOT_TICKS="$1" ./melee.exe play --iso "$GW_ISO_VANILLA" --no-audio $args --shot-dir "$out/shots" >"$out/stdout.txt" 2>"$out/stderr.txt") ;;
  *) echo "run.sh host|ref ..." >&2; exit 2 ;;
esac
echo "exit $?"
