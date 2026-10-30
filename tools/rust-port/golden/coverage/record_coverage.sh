#!/bin/bash
# Record coverage goldens from the C build: each scenario of docs/rust-port/golden/coverage/scenarios.json
# runs TWICE with MELEE_NEUTRAL_HASHLOG; the CSVs must be identical before the first is copied to
# docs/rust-port/golden/coverage/<name>.csv (hash CSVs only: no disc data). Details: docs/rust-port/notes/coverage.md.
#
#   export GW_MELEE=<your worktree>          (default: this worktree)
#   export COVERAGE_AGENT=<unique name>      (your private run root: _build/agents/<name>; default "cov-$$")
#   bash tools/rust-port/golden/coverage/record_coverage.sh [--bin DIR] [--force] NAME...
#   bash tools/rust-port/golden/coverage/record_coverage.sh [--bin DIR] --batch N      (batches in scenarios.json)
#   bash tools/rust-port/golden/coverage/record_coverage.sh [--bin DIR] --group G | --all
#
# --bin DIR  the shared read-only binary (default _build/agents/coverage-bin): melee-pc.exe, melee-pc.map, SDL3.dll,
#            webgpu_dawn.dll. It is copied into your private root once; nothing is ever written to DIR.
# --force    re-record a scenario that already has a CSV (default: skip it, so a batch can be resumed).
# No OS input is ever sent: input is MELEE_PAD_SCRIPT (Lua/text) and the scene string; runs are headless/turbo and
# end themselves (the pad script quits at a fixed frame; --max-seconds is only a hang guard).
# Exit code 1 if any scenario is not deterministic or failed to run.
set -u
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ws="${GW_WORKSPACE:-E:/Projects/Melee Workspace}"
wt="${GW_MELEE:-$(cd "$here/../../../.." && pwd)}"
bin="$ws/_build/agents/coverage-bin"
force=0; names=()
while [ $# -gt 0 ]; do
    case "$1" in
        --bin) bin="$2"; shift ;;
        --force) force=1 ;;
        --batch) mapfile -t b < <(python "$here/scn.py" batch "$2"); names+=("${b[@]}"); shift ;;
        --group) mapfile -t b < <(python "$here/scn.py" names --group "$2" --status ready); names+=("${b[@]}"); shift ;;
        --all) mapfile -t b < <(python "$here/scn.py" names --status ready); names+=("${b[@]}") ;;
        *) names+=("$1") ;;
    esac
    shift
done
[ ${#names[@]} -gt 0 ] || { echo "no scenarios given"; exit 2; }
agent="${COVERAGE_AGENT:-cov-$$}"
root="$ws/_build/agents/$agent"
export GW_BUILD_ROOT="$root" GW_MELEE="$wt"
mkdir -p "$root/runs" "$root/covout"
for f in melee-pc.exe melee-pc.map SDL3.dll webgpu_dawn.dll; do
    [ -f "$bin/$f" ] || { echo "missing $bin/$f"; exit 2; }
    cmp -s "$bin/$f" "$root/$f" 2>/dev/null || cp -f "$bin/$f" "$root/$f"
done
set -a; source "$ws/.env"; set +a
export MELEE_UNATTENDED=1 MELEE_WINDOW_X=30000 MELEE_WINDOW_Y=30000 MELEE_SCRIPTS=0 MELEE_TURBO=1
export MELEE_PAD_IGNORE_ADAPTER=1 SDL_WINDOW_ACTIVATE_WHEN_SHOWN=0
golden="$wt/docs/rust-port/golden/coverage"
padroot="$(cd "$wt/tools/rust-port/golden" && pwd -W)"
out="$root/covout"; outw="$(cd "$out" && pwd -W)"
mkdir -p "$golden"
rc=0
for name in "${names[@]}"; do
    if [ "$force" = 0 ] && [ -s "$golden/$name.csv" ]; then echo "$name: already recorded (use --force)"; continue; fi
    IFS=$'\t' read -r _ pad scene seed secs envs < <(python "$here/scn.py" tsv "$name") || { echo "$name: unknown scenario"; rc=1; continue; }
    extra=()
    IFS=',' read -ra kv <<< "$envs"; for e in "${kv[@]}"; do [ -n "$e" ] && extra+=("$e"); done
    for pass in a b; do
        rm -f "$out/$name.$pass.csv"
        env MELEE_TEST_SEED="$seed" MELEE_PAD_SCRIPT="$padroot/$pad" ${scene:+"MELEE_SCENE=$scene"} \
            MELEE_NEUTRAL_HASHLOG="$outw/$name.$pass.csv" "${extra[@]}" \
            bash "$ws/tools/port/run.sh" --max-seconds "$secs" "cov-$name-$pass" --iso "$GW_ISO_VANILLA" \
            > "$out/$name.$pass.out" 2>&1
        echo "$name pass $pass: run rc=$? rows=$(($(wc -l < "$out/$name.$pass.csv" 2>/dev/null || echo 1) - 1))"
    done
    rows=$(($(wc -l < "$out/$name.a.csv" 2>/dev/null || echo 1) - 1))
    if [ "$rows" -lt 1 ]; then echo "$name: NO hash rows (scene did not reach a match?), not written"; rc=1; continue; fi
    if python "$here/../../compare_golden.py" "$out/$name.a.csv" "$out/$name.b.csv"; then
        cp -f "$out/$name.a.csv" "$golden/$name.csv"
        echo "$name: deterministic, golden written ($rows rows)"
    else
        echo "$name: NOT deterministic, golden not written"; rc=1
    fi
done
exit $rc
