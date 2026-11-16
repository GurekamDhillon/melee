#!/bin/bash
# The bracket client end to end on loopback: tools/rust-port/bracket/test_server.py on 127.0.0.1, then a two-peer
# netplay pair (tools/rust-port/netplay/play_pair.sh) with auto-upload on for both peers, each with its own key folder
# and replay folder. Each peer's run ends by exporting its .slp (headless playback), queueing it and uploading it,
# signed, to the test server. Never the real server.
#   bash tools/rust-port/bracket/pair_upload.sh NAME     (PROFILE=release by default; NP_SCRATCH for the outputs)
# Exit 0: the pair passed and the server stored one upload from each peer (two different keys).
set -u
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"
name=${1:-bracketpair}
out="${NP_SCRATCH:-${TEMP:-/tmp}/np}/$name"; rm -rf "$out"; mkdir -p "$out"
port=$((52100 + RANDOM % 300))
python "$here/test_server.py" "$port" "$out/server" 7 3 1 >"$out/server.log" 2>&1 &
srv=$!
trap 'kill $srv 2>/dev/null' EXIT
sleep 1
for p in host guest; do mkdir -p "$out/$p-bracket" "$out/$p-replays"; done
q() { printf %q "$1"; }
common="MELEE_BRACKET_UPLOAD=1 MELEE_BRACKET_EVENT=7 MELEE_BRACKET_SERVER=http://127.0.0.1:$port MELEE_BRACKET_FLUSH_SECS=30"
export NP_HOST_ENV="$common MELEE_BRACKET_DIR=$(q "$out/host-bracket") MELEE_SLP_DIR=$(q "$out/host-replays")"
export NP_GUEST_ENV="$common MELEE_BRACKET_DIR=$(q "$out/guest-bracket") MELEE_SLP_DIR=$(q "$out/guest-replays")"
export NP_SCRATCH="$out" NP_TIME_SECS=${NP_TIME_SECS:-30}
bash "$repo/tools/rust-port/netplay/play_pair.sh" "$name" 0 0 --headless --no-audio; pr=$?
echo "--- client logs"
grep -h "^bracket:" "$out/$name-host.log" "$out/$name-guest.log"
echo "--- server uploads"
cat "$out/server/uploads.log" 2>/dev/null
n=$(wc -l <"$out/server/uploads.log" 2>/dev/null || echo 0)
keys=$(cut -d' ' -f1 "$out/server/uploads.log" 2>/dev/null | sort -u | wc -l)
for p in host guest; do echo "$p queue:"; cat "$out/$p-bracket/queue.tsv" 2>/dev/null; done
[ $pr -eq 0 ] && [ "$n" -ge 2 ] && [ "$keys" -eq 2 ] && { echo "OK $name: pair passed, $n uploads from $keys keys"; exit 0; }
echo "FAILED $name: pair exit $pr, $n uploads, $keys keys"; exit 1
