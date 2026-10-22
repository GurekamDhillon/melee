#!/bin/bash
# Run the state oracle one function per process (a crash in one function does not hide the rest).
#   TOU_CASES=200 bash rust/melee_modes/oracle/tou/run_all.sh [name ...]
# env: TOU_CASES (default 200), DETAIL (lines of failure detail), TOU_TRACE=1 (call traces),
#      TOU_SIDE=rust|c (run only one implementation, to see which one crashes)
HERE="$(cd "$(dirname "$0")" && pwd)"
WT="$(cd "$HERE/../../../.." && pwd)"
cd "$WT/rust"
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
BOUT=$(cargo test -p melee_modes --features c_oracle --test tou --release --no-run 2>&1)
echo "$BOUT" | grep -E "^error" -A8 | head -30
EXE=$(echo "$BOUT" | grep -o 'Executable.*(.*)' | sed 's/.*(\(.*\))/\1/' | head -1)
if [ -z "$EXE" ] || [ ! -f "$EXE" ]; then EXE=$(ls -t "$WT"/build/cargo/release/deps/tou-*.exe 2>/dev/null | head -1 | sed 's/\*$//'); fi
NAMES=("$@")
[ ${#NAMES[@]} -eq 0 ] && NAMES=($(grep -o '^    ("[a-z_0-9]*", "[A-Za-z_0-9]*", [0-9-]*, &\[' "$HERE/../../tests/tou/gen_tables.rs" | sed 's/^    ("[a-z_0-9]*", "\([A-Za-z_0-9]*\)".*/\1/'))
pass=0; fail=0
for n in "${NAMES[@]}"; do
  out=$(RUST_BACKTRACE=0 TOU_ONLY=$n timeout 300 "$EXE" state_oracle --nocapture 2>&1); rc=$?
  if [ $rc -eq 0 ]; then
    if echo "$out" | grep -q "no recipe: \[\]"; then echo "PASS $n"; pass=$((pass+1)); else echo "NORECIPE $n"; fi
  else
    echo "FAIL($rc) $n"; fail=$((fail+1))
    echo "$out" | grep -a -A8 "^FAIL\|panicked\|ASSERT\|CRASH" | cut -c1-300 | head -${DETAIL:-14}
  fi
done
echo "pass $pass fail $fail"
