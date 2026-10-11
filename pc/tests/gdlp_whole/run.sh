#!/bin/bash
# Whole-match .gdlp acceptance (R3/R4): records a scripted Falcon v Fox match with MELEE_GDLP_RECORD,
# plays it back with MELEE_GDLP (state trace compared with its own recorded post-frames by replay_compare; MELEE_SLP_PARITY also checks the RNG seed per frame, which a port recording does not satisfy: same as MELEE_SLP_RECORD),
# exports it to .slp (the game's lab rec export-slp rule, via pc/tools/gdlp_export.py), plays the .slp with
# MELEE_SLP and checks it parses with the workspace's tools/replay/slp.py.
# Needs GW_MELEE, GW_BUILD_ROOT exported and the vanilla disc in .env. Usage: run.sh [tag]
set -u
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ws="${GW_WORKSPACE:-E:/Projects/Melee Workspace}"
tag="${1:-gdlpw}"
out="$GW_BUILD_ROOT/gdlp_whole/$tag"
rm -rf "$out"; mkdir -p "$out"
set -a; source "$ws/.env"; set +a
export MELEE_UNATTENDED=1 MELEE_WINDOW_X=30000 MELEE_WINDOW_Y=30000 MELEE_NETPLAY_BIND=127.0.0.1 MELEE_SCRIPTS=0
pad="$(cd "$here" && pwd -W)/pad.lua"
run() { # name, max seconds, extra env as VAR=value...
    local name="$1" secs="$2"; shift 2
    env "$@" MELEE_TURBO=1 MELEE_PAD_SCRIPT="$pad" bash "$ws/tools/port/run.sh" --max-seconds "$secs" "$tag-$name" --iso "$GW_ISO_VANILLA" > "$out/$name.out" 2>&1
    echo "run $name rc=$?"
}
scene='mode=vs;at=match;p1=falcon/human;p2=fox/human;p3=none;p4=none;stage=fd;match=time;minutes=0;items=off'
gdlp="$(cd "$out" && pwd -W)/a.gdlp"
run rec 240 "MELEE_SCENE=$scene" "MELEE_GDLP_RECORD=$gdlp"
log="$GW_BUILD_ROOT/runs/$tag-rec/melee-pc.log"
grep -a "GDLPTEST\|replay: recording\|replay: recording closed" "$log" | head -5
ls -l "$out/a.gdlp" 2>&1 | head -1
python "$GW_MELEE/pc/tools/gdlp_export.py" --info "$out/a.gdlp"
python "$GW_MELEE/pc/tools/gdlp_export.py" "$out/a.gdlp" "$out/a.slp"
python - "$out/a.slp" "$ws" <<'PY'
import sys
sys.path.insert(0, sys.argv[2] + "/tools/replay")
import slp
r = slp.parse(sys.argv[1])
print("slp.py parsed: frames %d..%d, %d post-frame frames" % (r.first_frame, r.last_frame, len(r.post)))
PY
gdlpw="$(cd "$out" && pwd -W)"
run play 120 "MELEE_GDLP=$gdlp" "MELEE_STATE_TRACE=$gdlpw/b.csv"
grep -a "MELEE_GDLP\|past the replay" "$GW_BUILD_ROOT/runs/$tag-play/melee-pc.log" | tail -4
run slp 120 "MELEE_SLP=$gdlpw/a.slp" "MELEE_STATE_TRACE=$gdlpw/c.csv"
grep -a "past the replay" "$GW_BUILD_ROOT/runs/$tag-slp/melee-pc.log" | tail -2
echo "--- gdlp playback vs the recorded post-frames"; python "$ws/tools/replay/replay_compare.py" "$out/a.slp" "$out/b.csv" | tail -3
echo "--- exported .slp playback vs the recorded post-frames"; python "$ws/tools/replay/replay_compare.py" "$out/a.slp" "$out/c.csv" | tail -3
