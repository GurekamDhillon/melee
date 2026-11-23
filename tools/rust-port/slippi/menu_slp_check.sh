#!/bin/bash
# Online .slp proof on Linux, through the online select in one window per side (slippi_link::inprocess). Loopback only,
# both peers hidden, test identities only (SLPA#001 / SLPB#002: no account, no user.json, no real server).
# Each side is launched as Hero's launcher launches a game with --replay-dir: `--record <file> --slp auto` and
# MELEE_SLP_DIR, so every online match must leave its own .slp there (slp_export's live recording).
#   rematch: Unranked select -> game 1 -> back to the select on the same link -> game 2 (menu_inprocess_loop.sh's run,
#            1 stock); each side must write one .slp per game, and the two sides' files of a game must agree
#            (slp_agree.py: Game Start, every Pre-/Post-Frame event of the common frames)
#   quit:    one Unranked game; B quits mid-match (MELEE_SLIPPI_NP_QUIT_AT=600); A's .slp must still be written, with a
#            Game End, and play in `melee slpcheck` (parsed, not refused)
# usage: menu_slp_check.sh OUTDIR rematch|quit [--mm-port P] [--lag-ms N] [--jitter-ms N] [--loss-pct N]   (EXE=<melee binary>;
#   GW_ISO_VANILLA set by the caller; SLPCHECK_TITLE=<word> prefixes both window titles). --lag-ms / --jitter-ms / --loss-pct go to the mock's relay: many rollbacks (the stress run).
set -u
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"
B="${EXE:-$repo/rust/target/release/melee}"; O=$1; mode=$2; shift 2
mmp=43470; relay=()
while [ $# -gt 0 ]; do case "$1" in --mm-port) mmp=$2; shift ;; --lag-ms|--jitter-ms|--loss-pct) relay+=("$1" "$2"); shift ;; esac; shift; done
[ -n "${GW_ISO_VANILLA:-}" ] || { echo "GW_ISO_VANILLA is not set"; exit 2; }
export MELEE_INPUT=n MELEE_VOLUME=3 MELEE_WINDOW_HIDDEN=1 MELEE_PAD_IGNORE_ADAPTER=1 MELEE_WINDOW_UNFOCUSED=1 MELEE_KEYBOARD=0
M="$repo/tools/rust-port/menus"
rm -rf "$O"; mkdir -p "$O/slp-a" "$O/slp-b"
# the mock is backgrounded on its own (its $! is the mock itself) and never outlives this script
python3 -I "$here/mock_mm.py" --port $mmp "${relay[@]}" > "$O/mm.log" 2>&1 < /dev/null & mm=$!
ours() { for p in $(pgrep -x melee); do tr '\0' '\n' < /proc/$p/environ 2>/dev/null | grep -qE "^MELEE_SLIPPI_TEST_CODE=SLP[AB]#00[12]$" && echo $p; done; }
cleanup() { for p in $(ours); do kill $p 2>/dev/null; done; kill $mm 2>/dev/null; }
trap cleanup EXIT
sleep 2
side() { # side NAME CODE NPPORT SCRIPT G2SCRIPT G3SCRIPT [ENV=VALUE ...]
  local s; s=$(echo $1 | tr AB ab)
  local name=$1 code=$2 port=$3 script=$4 g2=$5 g3=$6; shift 6
  ( cd "$repo/rust" && env "$@" MELEE_WINDOW_TITLE="${SLPCHECK_TITLE:+$SLPCHECK_TITLE:}slpcheck:$name:$mode" MELEE_RETAIL_FIRST=1 MELEE_CARD=0 MELEE_PAD_SCRIPT="$script" \
    MELEE_SLIPPI_SET_PAD_SCRIPT_G2="$g2" MELEE_SLIPPI_SET_PAD_SCRIPT_G3="$g3" MELEE_SLIPPI_SET_MAX_FRAMES=1500 \
    MELEE_SLIPPI_MM_URL="ws://127.0.0.1:$mmp/v1/queue" MELEE_SLIPPI_NP_PORT=$port MELEE_SLIPPI_TEST_CODE="$code" \
    MELEE_SLIPPI_NP_OUT="$O/$s.csv" MELEE_SLIPPI_NP_MAX_SECONDS=420 MELEE_SLIPPI_NP_PADGEN=7 MELEE_SLP_DIR="$O/slp-$s" \
    timeout 420 "$B" play --iso "$GW_ISO_VANILLA" --skip-intro --max-frames 9000 --slippi-menus on \
      --record "$O/$s-menu.mrpl" --slp auto > "$O/$s-menu.log" 2>&1 < /dev/null )
}
if [ "$mode" = rematch ]; then
  side A "SLPA#001" $((mmp + 100)) "$M/slippi_online_unranked_css_1start.txt" "$M/slippi_back_a.txt" "$M/slippi_rematch_leave_a.txt" MELEE_SLIPPI_TEST_STOCKS=1 & pa=$!
  sleep 1
  side B "SLPB#002" $((mmp + 101)) "$M/slippi_online_b_unranked_css_1start.txt" "$M/slippi_back_b.txt" "" MELEE_SLIPPI_TEST_STOCKS=1 & pb=$!
else
  side A "SLPA#001" $((mmp + 100)) "$M/slippi_online_unranked_css_1start.txt" "" "" MELEE_SLIPPI_TEST_STOCKS=4 & pa=$!
  sleep 1
  side B "SLPB#002" $((mmp + 101)) "$M/slippi_online_b_unranked_css_1start.txt" "" "" MELEE_SLIPPI_TEST_STOCKS=4 MELEE_SLIPPI_NP_QUIT_AT=600 & pb=$!
fi
# the survivor (quit) or both (rematch) may stay on the select after the last match: the files are written at each
# match's end, so stop once the expected .slp are there (bounded)
want=$([ "$mode" = rematch ] && echo 2 || echo 1)
for _ in $(seq 1 140); do
  na=$(ls "$O"/slp-a/*.slp 2>/dev/null | wc -l); nb=$(ls "$O"/slp-b/*.slp 2>/dev/null | wc -l)
  [ "$mode" = rematch ] && [ "$na" -ge 2 ] && [ "$nb" -ge 2 ] && break
  [ "$mode" = quit ] && [ "$na" -ge 1 ] && ! kill -0 $pb 2>/dev/null && break
  sleep 3
done
sleep 3; cleanup; wait $pa $pb 2>/dev/null
echo "== mock"; grep -hE "join from|pairing" "$O/mm.log" | cut -c1-60
for s in a b; do echo "== $s"; grep -hE "gw: slp: [0-9]+ frames|Slippi: [0-9]|match over|QUIT_AT|disconnected|rematch|link taken over" "$O/$s-menu.log" | sed -E "s|$O|<out>|g" | cut -c1-150; done
fail=0; bad() { echo "FAIL: $*"; fail=1; }
for s in a b; do
  grep -aqE "double free|panicked|crash report written|SIGABRT|SIGSEGV" "$O/$s-menu.log" && bad "$s: a crash or panic (see $s-menu.log)"
  grep -ahoE "rollbacks [0-9]+ \(max depth [0-9]+" "$O/$s-menu.log" | sed "s/^/  $s: /"
done
mapfile -t fa < <(ls "$O"/slp-a/*.slp 2>/dev/null | sort); mapfile -t fb < <(ls "$O"/slp-b/*.slp 2>/dev/null | sort)
if [ "$mode" = rematch ]; then
  [ ${#fa[@]} -eq 2 ] || bad "A wrote ${#fa[@]} .slp (want 2)"; [ ${#fb[@]} -eq 2 ] || bad "B wrote ${#fb[@]} .slp (want 2)"
  for g in 0 1; do
    [ -n "${fa[$g]:-}" ] && [ -n "${fb[$g]:-}" ] || continue
    echo "== game $((g + 1))"; python3 -I "$here/slp_agree.py" "${fa[$g]}" "${fb[$g]}" || bad "game $((g + 1)): the two peers' files differ"
  done
  n=$(grep -c "join from SLPA#001" "$O/mm.log"); [ "$n" -eq 1 ] || bad "A: $n joins (want 1: both games on one link)"
else
  [ ${#fa[@]} -eq 1 ] || bad "A (the survivor) wrote ${#fa[@]} .slp (want 1)"
  if [ -n "${fa[0]:-}" ]; then
    python3 -I "$here/slp_agree.py" "${fa[0]}" "${fa[0]}" | head -2
    r=$("$B" slpcheck --iso "$GW_ISO_VANILLA" --slp "${fa[0]}" 2>/dev/null | grep -E "^SLPCHECK" | cut -c1-140); echo "$r"
    echo "$r" | grep -qE " (match|diverged) " || bad "slpcheck did not play the survivor's file: $r"
  fi
fi
[ $fail = 0 ] && echo "PASS: $mode"
exit $fail
