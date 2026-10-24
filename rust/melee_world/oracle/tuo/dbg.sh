#!/bin/bash
# dbg.sh <stem> <fn> <idx>: compare the full C log and the full Rust log of one case
# (PRE=1 also logs the state before the call; the Rust side runs ALL cases, like the real test)
HERE="$(cd "$(dirname "$0")" && pwd)"
WT="$(cd "$HERE/../../../.." && pwd)"
D="$WT/rust/target/tuo/$1"
PREV=""; [ -n "$PRE" ] && PREV="TUO_PRE=1"
env $PREV "$D/oracle.exe" "$D/tuo.cfg" --full "$2" "$3" | sed '$d' > "$D/c.log"
cd "$WT/rust"
env $PREV TUO_FULL="$2:$3" CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 cargo test -p melee_world --test "tuo_$1" -- --nocapture 2>/dev/null | sed -n "/^case $2 $3\$/,\$p" | sed '/^thread /,$d' > "$D/rs.log"
python "$HERE/difflog.py" "$D/c.log" "$D/rs.log"
