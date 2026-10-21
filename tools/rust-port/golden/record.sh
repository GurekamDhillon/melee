#!/bin/bash
# Record the Rust-port golden corpus: per-frame byte-order-neutral gameplay hashes
# (MELEE_NEUTRAL_HASHLOG, docs/rust-port/neutral-hash.md) from the C build, the oracle.
# Each scenario runs TWICE; the two CSVs must be identical (determinism) before the first is
# copied to docs/rust-port/golden/<scenario>.csv. Only hash CSVs are written there: no disc data.
#
#   export GW_MELEE=<this worktree>  GW_BUILD_ROOT=<a private build root>   (tools/port/agent_new.sh)
#   bash tools/rust-port/golden/record.sh [scenario ...]      (default: all)
set -u
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ws="${GW_WORKSPACE:-E:/Projects/Melee Workspace}"
golden="$here/../../../docs/rust-port/golden"
out="$GW_BUILD_ROOT/rustgold"
mkdir -p "$out" "$golden"
set -a; source "$ws/.env"; set +a
export MELEE_UNATTENDED=1 MELEE_WINDOW_X=30000 MELEE_WINDOW_Y=30000 MELEE_SCRIPTS=0 MELEE_TEST_SEED=12345
padw="$(cd "$here" && pwd -W)"
outw="$(cd "$out" && pwd -W)"

# name | pad script | max seconds | scene
SCENARIOS='
pad_falcon_fox_fd|pad_fixed.lua|240|mode=vs;at=match;p1=falcon/human;p2=fox/human;p3=none;p4=none;stage=fd;match=time;minutes=0;items=off
pad_marth_sheik_bf|pad_fixed.lua|240|mode=vs;at=match;p1=marth/human;p2=sheik/human;p3=none;p4=none;stage=bf;match=time;minutes=0;items=off
cpu_fox_falco_dl|pad_idle.lua|300|mode=vs;at=match;p1=fox/cpu9;p2=falco/cpu9;p3=none;p4=none;stage=dreamland;match=time;minutes=0;items=off
cpu_peach_samus_ys_items|pad_idle.lua|300|mode=vs;at=match;p1=peach/cpu9;p2=samus/cpu9;p3=none;p4=none;stage=ys;match=time;minutes=0;items=4
'
want=" $* "
rc=0
while IFS='|' read -r name pad secs scene; do
    [ -n "$name" ] || continue
    [ $# -eq 0 ] || [[ "$want" == *" $name "* ]] || continue
    for pass in a b; do
        rm -f "$out/$name.$pass.csv"
        env MELEE_TURBO=1 MELEE_PAD_SCRIPT="$padw/$pad" "MELEE_SCENE=$scene" \
            MELEE_NEUTRAL_HASHLOG="$outw/$name.$pass.csv" \
            bash "$ws/tools/port/run.sh" --max-seconds "$secs" "rustgold-$name-$pass" --iso "$GW_ISO_VANILLA" \
            > "$out/$name.$pass.out" 2>&1
        echo "$name pass $pass: run rc=$? rows=$(($(wc -l < "$out/$name.$pass.csv" 2>/dev/null || echo 1) - 1))"
    done
    if python "$here/../compare_golden.py" "$out/$name.a.csv" "$out/$name.b.csv"; then
        cp -f "$out/$name.a.csv" "$golden/$name.csv"
        echo "$name: deterministic, golden written"
    else
        echo "$name: NOT deterministic, golden not written"; rc=1
    fi
done <<< "$SCENARIOS"
exit $rc
