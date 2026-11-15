#!/usr/bin/env bash
# A/B frame-time comparison of two revisions (docs/rust-port/notes/profiling.md).
#
#   bash tools/rust-port/perf-ab.sh <base-rev> <head-rev> <scenario> [N]
#
# Builds both revisions with RELEASE=1 (cargo build --release -p melee) in private worktrees and private target
# dirs under $PERF_AB_DIR (default <workspace>/_build/perf-ab), checks each exe's banner id against its revision,
# then runs the scenario N times (default 5) alternating base / head with the frame profiler (MELEE_PROFILE=1) and
# prints per-phase medians and p99, frames over 16.7 ms split clean / contended, and the head - base difference with
# its noise range (perf_ab.py).
#
# Scenarios:
#   fd    real-menu walk to a 2-CPU Final Destination match (walk_vs_fd_2cpu.txt), 2400 frames. Every revision.
#   sse   the SSE match-start walks of scratch/sse4/quiet.sh: Lake (explore doors) and Jungle (walk_vs_sse_2cpu.txt,
#         walk_vs_jungle_2cpu.txt) + FD, 3200/3200/2400 frames. Revisions with the SSE stages (port/sse-stage2).
#   boss  a boss fight like the geno VM check's perf row: Falcon vs a level-1 CPU Fox carrier on FD with
#         MELEE_VS_BOSS=${BOSS:-rayquaza}, the jab schedule, 3000 frames. Revisions with MELEE_VS_BOSS.
#
# PERF_AB_SHARED=1: build in the lanes' shared target dir <workspace>/_build/cargo-shared (cargo locks it while in
# use; incremental instead of from scratch). Each exe is copied out right after its build and its banner id checked.
# Env: N (runs per side, default 5), FEATURES (cargo features for both builds, e.g. profile-tracy), KEEP=1 (keep the
# worktrees and target dirs; default: deleted at the end, exes and profiles stay in the output dir), OUT (output dir),
# TMO (seconds per run, default 900), BOSS. Needs GW_ISO_VANILLA (the workspace .env). Pad scripts come from the head
# revision's tree. Game runs: hidden window, volume 3, titled "perf-ab <side> <scenario> <i>", pad scripts only.
# Waits while the owner-play lock is held. Builds: buildlock lane "perf-ab", -j ${JOBS:-3}, waits for 8 GB free.
set -u
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo="$(cd "$here/../.." && pwd)"
ws="${GW_WORKSPACE:-E:/Projects/Melee Workspace}"
S="$ws/scratch"
command -v python >/dev/null 2>&1 || python() { python3 "$@"; }
[ $# -ge 3 ] || { sed -n 2,26p "$0"; exit 2; }
base_rev=$1; head_rev=$2; scen=$3; N=${4:-${N:-5}}
case "$scen" in fd|sse|boss) ;; *) echo "perf-ab: unknown scenario $scen (fd|sse|boss)" >&2; exit 2 ;; esac
if [ -z "${GW_ISO_VANILLA:-}" ] && [ -f "$ws/.env" ]; then set -a; eval "$(grep -E '^(export )?GW_ISO_VANILLA=' "$ws/.env" | sed 's/^export //')"; set +a; fi
[ -n "${GW_ISO_VANILLA:-}" ] || { echo "perf-ab: GW_ISO_VANILLA not set" >&2; exit 2; }
root="${PERF_AB_DIR:-$ws/_build/perf-ab}"
bs=$(git -C "$repo" rev-parse --short=9 "$base_rev^{commit}") || exit 2
hs=$(git -C "$repo" rev-parse --short=9 "$head_rev^{commit}") || exit 2
out="${OUT:-$root/out-$bs-$hs-$scen}"
mkdir -p "$out"
echo "perf-ab: base $base_rev = $bs, head $head_rev = $hs, scenario $scen, $N runs each; output $out"

owner_play() { local d; for d in "$S/.cargo-build.lock" "$S/.cargo-build.lock".[2-4] "$S/.light-run.lock"; do [ "$(cat "$d/owner" 2>/dev/null)" = owner-play ] && return 0; done; return 1; }
wait_quiet() { while owner_play; do echo "perf-ab: owner is playing; waiting"; sleep 60; done; }
free_kb() { awk '/^MemFree:/{print $2}' /proc/meminfo 2>/dev/null || echo 99999999; }
disk_ok() {
  local c e; c=$(df -k /c 2>/dev/null | awk 'NR==2{print $4}'); e=$(df -k /e 2>/dev/null | awk 'NR==2{print $4}')
  [ "${c:-99999999}" -ge $((15 * 1024 * 1024)) ] && [ "${e:-99999999}" -ge $((20 * 1024 * 1024)) ]
}

# build <short-sha> -> $out/melee-<sha>.exe
build() {
  local sha=$1 wt="$root/wt-$1" exe="$out/melee-$1.exe" tgt="$root/wt-$1/build/cargo-ab"
  [ -n "${PERF_AB_SHARED:-}" ] && tgt="$ws/_build/cargo-shared"
  if [ -f "$exe" ]; then echo "perf-ab: $exe exists (reused)"; else
    disk_ok || { echo "perf-ab: FAIL low disk (C: < 15 GB or E: < 20 GB free)" >&2; exit 3; }
    [ -d "$wt" ] || git -C "$repo" worktree add -q --detach "$wt" "$sha" || exit 3
    while [ "$(free_kb)" -lt $((8 * 1024 * 1024)) ]; do echo "perf-ab: waiting for 8 GB free memory"; sleep 30; done
    wait_quiet
    local feat=(); [ -n "${FEATURES:-}" ] && feat=(--features "$FEATURES")
    echo "perf-ab: building $sha (release${FEATURES:+, features $FEATURES})"
    ( cd "$wt/rust" && CARGO_TARGET_DIR="$tgt" bash "$S/buildlock.sh" run perf-ab \
        cargo build --release -j "${JOBS:-3}" -p melee "${feat[@]}" ) > "$out/build-$sha.log" 2>&1 \
      || { tail -20 "$out/build-$sha.log"; echo "perf-ab: FAIL build $sha" >&2; exit 3; }
    local b="$tgt/release/melee"; [ -f "$b" ] || b="$b.exe"
    cp "$b" "$exe"   # copied out at once: a shared target dir's exe is replaced by the next lane's build
  fi
  local id; id=$(grep -a -o 'melee \[[A-Za-z0-9_./-]* [0-9a-f]\{7,12\}' "$exe" | head -1 | awk '{print $NF}')
  case "$sha" in "$id"*) echo "perf-ab: banner ok: $sha -> ${id}" ;;
    *) case "$id" in "${sha:0:7}"*) echo "perf-ab: banner ok: $sha -> ${id}" ;; *) echo "perf-ab: FAIL banner of $exe is '${id}', want $sha" >&2; exit 3 ;; esac ;;
  esac
}
build "$bs"; build "$hs"

# the pad scripts and fixtures from the head revision
pads="$out/pads"; mkdir -p "$pads"
for f in walk_vs_fd_2cpu.txt walk_vs_sse_2cpu.txt walk_vs_jungle_2cpu.txt; do
  git -C "$repo" show "$hs:tools/rust-port/play/$f" > "$pads/$f" 2>/dev/null || rm -f "$pads/$f"
done
{ printf '420 0000 %s 0\n' "${WALK_X:-60}"; for i in $(seq 1 400); do printf '2 0100 0 0\n10 0000 0 0\n'; done; } > "$pads/fight.txt"
card_src="${PERF_AB_CARD:-$S/test-card-master}"

# one run: side sha i label pad frames [env...]
run1() {
  local side=$1 sha=$2 i=$3 lab=$4 pad=$5 frames=$6; shift 6
  local d="$out/runs/$side-$lab-$i"; rm -rf "$d"; mkdir -p "$d/card/USA/Card A" "$d/save"
  [ -d "$card_src" ] && cp -r "$card_src/." "$d/card/USA/Card A/"
  [ -f "$pads/$pad" ] || { echo "perf-ab: FAIL pad $pad not in $hs" >&2; exit 3; }
  # the profiler writes profiles/ beside the exe: each run gets its own copy
  cp "$out/melee-$sha.exe" "$d/melee.exe"
  wait_quiet
  local args=(play --iso "$GW_ISO_VANILLA" --widescreen --windowed --window 1280x720 --max-frames "$frames")
  [ "$scen" = boss ] && args=(play --iso "$GW_ISO_VANILLA" --max-frames "$frames" --match "falcon,fox/cpu1,FINAL DESTINATION" --time-secs 600)
  ( cd "$d" && env MELEE_WINDOW_HIDDEN=1 MELEE_VOLUME=3 MELEE_WINDOW_TITLE="perf-ab $side $scen $lab $i" MELEE_FULLSCREEN=off \
      MELEE_WINDOW_UNFOCUSED=1 MELEE_PAD_IGNORE_ADAPTER=1 MELEE_PAD_NO_PAUSE=1 MELEE_PROFILE=1 MELEE_CARD_PATH="$d/card" \
      MELEE_ENVOY_DIR="$d/save" MELEE_BRAWL_LOCAL="${MELEE_BRAWL_LOCAL:-$ws/_build/brawl}" MELEE_PAD_SCRIPT="$pads/$pad" "$@" \
      timeout "${TMO:-900}" "$d/melee.exe" "${args[@]}" ) > "$d/run.log" 2>&1
  local rc=$? p; p=$(ls -t "$d"/profiles/profile-*.csv 2>/dev/null | head -1)
  [ -n "$p" ] && cp "$p" "$out/runs/$side-$lab-$i.csv" && cp "${p%.csv}.jsonl" "$out/runs/$side-$lab-$i.jsonl" 2>/dev/null
  rm -f "$d/melee.exe"
  echo "  $side $lab $i: exit $rc, $( [ -n "$p" ] && echo "$(($(wc -l < "$p") - 1)) frames" || echo "NO PROFILE")"
}

L="MELEE_EXPLORE_GOTO=3@300,4@450,15@600,9@750,10@900,16@1050,14@1200"; J="MELEE_EXPLORE_GOTO=1@300,3@450,4@600,5@750,7@900"
G="MELEE_GPUCACHE=$out/gpucache"
scenario() { # side sha i
  case "$scen" in
    fd) run1 "$1" "$2" "$3" fd walk_vs_fd_2cpu.txt 2400 "$G" ;;
    sse)
      run1 "$1" "$2" "$3" lake walk_vs_sse_2cpu.txt 3200 "$G" "$L"
      run1 "$1" "$2" "$3" jungle walk_vs_jungle_2cpu.txt 3200 "$G" "$J"
      run1 "$1" "$2" "$3" fd walk_vs_fd_2cpu.txt 2400 "$G" ;;
    boss) run1 "$1" "$2" "$3" boss fight.txt 3000 MELEE_VS_BOSS="${BOSS:-rayquaza}" MELEE_PETEY_LOG=1 ;;
  esac
}
rm -rf "$out/runs"; mkdir -p "$out/runs"
# one warm-up run per side (shader cache, disc cache), not counted
echo "perf-ab: warm-up"; scenario base "$bs" 0; scenario head "$hs" 0
rm -f "$out/runs/"*-0.csv "$out/runs/"*-0.jsonl
for i in $(seq 1 "$N"); do
  if [ $((i % 2)) = 1 ]; then scenario base "$bs" "$i"; scenario head "$hs" "$i"; else scenario head "$hs" "$i"; scenario base "$bs" "$i"; fi
done
python "$here/perf/perf_ab.py" "$out/runs" --base "base ($bs)" --head "head ($hs)" | tee "$out/summary.txt"
rc=${PIPESTATUS[0]}
if [ -z "${KEEP:-}" ]; then
  for s in "$bs" "$hs"; do
    [ -d "$root/wt-$s" ] && { rm -rf "$root/wt-$s/build"; git -C "$repo" worktree remove --force "$root/wt-$s"; }
  done
  rm -rf "$out/runs/"*/ "$out/gpucache"
fi
exit "$rc"
