#!/bin/bash
# Gameplay-code scenario checks (docs/rust-port/notes/gameplay-codes.md): each row runs one scenario twice headless
# through `melee golden` (off = no --codes, on = --codes LIST) and compares the per-frame hash CSVs.
#   expect "diff": on != off (the code changes the match);  "same": on == off (the code must not touch this match).
# Needs GW_ISO_VANILLA (the workspace .env, never printed). No OS input: CPUs / idle pads only, unfocused, volume 3.
#   bash tools/rust-port/codes/run_codes.sh [NAME_FILTER]
set -u
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo="$(cd "$here/../../.." && pwd)"
cd "$repo"
ws="${GW_WORKSPACE:-E:/Projects/Melee Workspace}"
if [ -z "${GW_ISO_VANILLA:-}" ] && [ -f "$ws/.env" ]; then set -a; . "$ws/.env"; set +a; fi
export MELEE_WINDOW_UNFOCUSED=1 MELEE_PAD_IGNORE_ADAPTER=1 MELEE_VOLUME=3 MELEE_SCENARIO_TSV="$here/codes.tsv"
exe="${MELEE_EXE:-$repo/build/cargo/${PROFILE:-release}/melee.exe}"
tmp="${TEMP:-/tmp}/codes.$$"; mkdir -p "$tmp"
fail=0
# scenario  codes  expect
cases="
codes_stadium   frozen-ps       diff
codes_stadium   frozen-stages   diff
codes_story     frozen-stages   diff
codes_story     frozen-ps       same
codes_dl64      frozen-stages   diff
codes_fd        frozen-stages   diff
codes_fd        frozen-ps       same
codes_bf_p1p3   neutral-spawn   diff
codes_bf_p1p3   tournament      diff
codes_bf_1min   handwarmers     diff
codes_bf_2min   handwarmers     same
codes_ic_fd     no-wobble       any
codes_bf_2min   lgl:0           same
codes_bf_2min   skip-results    same
codes_bf_2min   frozen-stages,no-wobble,lgl,skip-results,freeze-fix,ucf   same
codes_bf_2min   stealth         diff
codes_bf_1min   handwarmers+    same-as-on
pad_falcon_fox_fd ucf           same
codes_ucf_fd    ucf             same
codes_ucf_fd    none            diff
"
run() { # name codes(or -) out
  local extra=(); [ "$2" != - ] && extra=(--codes "$2")
  local frames; frames=$(awk -F'	' -v s="$1" '$1==s{print $3+1}' "$MELEE_SCENARIO_TSV")
  MELEE_WINDOW_TITLE="slpcodes:$1:$2" "$exe" golden --engine tu --iso "$GW_ISO_VANILLA" --scenario "$1" --frames "$frames" --out "$3" "${extra[@]}" >"$3.log" 2>&1 || true
}
while read -r name codes expect; do
  [ -z "${name:-}" ] && continue
  [ -n "${1:-}" ] && [[ "$name" != *"$1"* ]] && continue
  off="$tmp/$name.off.csv"; on="$tmp/$name.$(echo "$codes" | tr ',:' '__').csv"
  [ -f "$off" ] || run "$name" - "$off"
  # same-as-on: determinism, the code run again (`X+` parses as X) must equal the first on-run
  [ "$expect" = same-as-on ] && { off="$tmp/$name.${codes%+}.csv"; expect=same; }
  run "$name" "$codes" "$on"
  r=$(python - "$off" "$on" <<'EOF'
import sys
a=open(sys.argv[1]).read().splitlines(); b=open(sys.argv[2]).read().splitlines()
n=min(len(a),len(b))
d=next((i for i in range(n) if a[i]!=b[i]), None)
print(f"rows {len(a)}/{len(b)} " + ("equal" if d is None and len(a)==len(b) else f"first-diff-row {d if d is not None else n}"))
EOF
)
  got=same; [[ "$r" == *first-diff* ]] && got=diff
  [[ "$r" == "rows 0/"* || "$r" == *"/0 "* ]] && got=empty
  ok=PASS
  if [ "$got" = empty ] || { [ "$expect" != any ] && [ "$got" != "$expect" ]; }; then ok=FAIL; fail=1; fi
  printf '%s %-14s %-22s expect %-4s got %-5s %s\n' "$ok" "$name" "$codes" "$expect" "$got" "$r"
done <<< "$cases"
[ $fail = 0 ] && echo "CODES OK" || echo "CODES FAIL (logs in $tmp)"
exit $fail
