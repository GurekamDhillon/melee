#!/bin/bash
# The launcher's lobby Direct button on the Rust engine (launcher_compat.rs, lobby_direct_args), loopback only.
# Two peers started with Melee Unlocked's lobby command line (`--iso .. --lobby-direct CODE --lobby-character N
# --lobby-status-file F`), each searching for the other's code through the local matchmaking mock (mock_mm.py), play
# one full match (1 stock, generated pads) and must exit 0 with 0 HARD desyncs and equal per-frame checksums.
# Then a failed connect: one peer searching for a code nobody plays must exit non-zero (the launcher's "could not
# connect" box) with the reason in its log.
# Test identities only (AAAA#001 / BBBB#002): no Slippi account, no user.json, no real server; windows hidden.
#   bash tools/rust-port/launcher/lobby_direct_pair.sh OUTDIR <disc>
set -u
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"
B="${EXE:-$repo/build/cargo/${PROFILE:-fast}/melee.exe}"; O=$1; DISC=$2; mkdir -p "$O"; rm -f "$O"/*
mmp=${MMPORT:-43370}
export MELEE_INPUT=n MELEE_VOLUME=3 MELEE_WINDOW_HIDDEN=1 MELEE_PAD_IGNORE_ADAPTER=1 MELEE_WINDOW_UNFOCUSED=1 MELEE_KEYBOARD=0 MELEE_CARD=0
export MELEE_SLIPPI_MM_URL="ws://127.0.0.1:$mmp/v1/queue" MELEE_SLIPPI_TEST_STOCKS=1
python -I "$repo/tools/rust-port/slippi/mock_mm.py" --port $mmp > "$O/mm.log" 2>&1 < /dev/null & mm=$!
sleep 2
peer() { # peer NAME MYCODE THEIRCODE CHAR NPPORT MAXS
  local s; s=$(echo $1 | tr AB ab)
  ( cd "$O" && MELEE_WINDOW_TITLE="lobbydirect:$1:lobby-direct" MELEE_SLIPPI_TEST_CODE="$2" MELEE_SLIPPI_NP_PORT=$5 MELEE_SLIPPI_NP_MAX_SECONDS=$6 \
    MELEE_LOG_FILE="$(cygpath -w "$O/$s.log")" \
    timeout 400 "$B" --iso "$DISC" --threaded-renderer --lobby-status-file "$(cygpath -w "$O/$s-status.json")" \
      --lobby-direct "$3" --lobby-character "$4" --slippi-padgen 7 --slippi-out "$(cygpath -w "$O/$s.csv")" --max-frames 20000 \
      > "$O/$s.out" 2>&1 < /dev/null; echo $? > "$O/$s.rc" )
}
# status sampler: the lobby status file's sequence on A
( for _ in $(seq 1 400); do [ -f "$O/a-status.json" ] && { cat "$O/a-status.json"; echo; } >> "$O/a-status-raw.txt"; sleep 1; done ) & sp=$!
peer A "AAAA#001" "BBBB#002" 2 $((mmp + 100)) 300 & pa=$!
sleep 1
peer B "BBBB#002" "AAAA#001" 9 $((mmp + 101)) 300 & pb=$!
wait $pa $pb; kill $sp 2>/dev/null; uniq "$O/a-status-raw.txt" > "$O/a-status-seq.txt" 2>/dev/null
fail=0; bad() { echo "FAIL: $*"; fail=1; }
mask() { sed -E 's/[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+/<ip>/g; s/(AAAA|BBBB)#00[12]/<code>/g'; }
for s in a b; do
  echo "== $s exit $(cat $O/$s.rc)"
  grep -hE "launcher-compat|match over|HARD desyncs|final hash" "$O/$s.log" "$O/$s.out" 2>/dev/null | mask | sed 's/^/  /' | head -12
  [ "$(cat $O/$s.rc)" = 0 ] || bad "$s exit $(cat $O/$s.rc)"
  grep -q "match over at game frame" "$O/$s.log" || bad "$s: no full match"
  grep -hE "HARD desyncs" "$O/$s.log" | grep -qvE " 0 HARD desyncs" && bad "$s: desync"
done
python -I -c "import sys
a=open(sys.argv[1]).read().split()[1:]; b=open(sys.argv[2]).read().split()[1:]; n=min(len(a),len(b))
d=sum(1 for x,y in zip(a,b) if x!=y); print('checksum rows compared', n, 'differing', d); sys.exit(1 if d or n<300 else 0)" "$O/a.csv" "$O/b.csv" || bad "checksums"
echo "--- A's lobby status sequence:"; sed 's/^/  /' "$O/a-status-seq.txt" | head -12
# a failed connect: nobody plays CCCC#003
rm -f "$O/a.log" "$O/a.out"
peer A "AAAA#001" "CCCC#003" 2 $((mmp + 102)) 20
rc=$(cat "$O/a.rc"); echo "== failed connect: exit $rc"
grep -hE "could not start|launcher-compat" "$O/a.log" "$O/a.out" | mask | sed 's/^/  /' | head -5
[ "$rc" != 0 ] || bad "failed connect exited 0"
grep -q "the online match could not start" "$O/a.log" "$O/a.out" || bad "no reason in the log"
kill $mm 2>/dev/null; wait $mm 2>/dev/null
[ $fail = 0 ] && echo "PASS" || { echo "FAIL"; exit 1; }
