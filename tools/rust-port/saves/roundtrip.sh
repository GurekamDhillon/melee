#!/bin/bash
export MELEE_WINDOW_HIDDEN=${MELEE_WINDOW_HIDDEN:-1}  # agents' game windows are invisible (MELEE_WINDOW_HIDDEN=0 shows them)
# Cross-build memory-card round trip (no OS input: MELEE_PAD_SCRIPT drives port 1, windows never take focus).
#   C_EXE=<a melee-pc.exe build root with the card shim> bash tools/rust-port/saves/roundtrip.sh [OUT_DIR]
# Every card is a scratch folder under OUT_DIR (default $TEMP/saves-roundtrip.$$), never the user's.
# The Rust runs are windowed (never focused), as the runs this was checked with.
#   1. Rust, fresh card R: "no save data" -> A creates the save; restart; Options > Rumble > P1 OFF.
#   2. C boots R: it must load it with no prompt (decision 0 -> 20) and, at the main menu, write it back.
#   3. Rust boots R again: load result 0, no file errors, rumble still [0, 1, 1, 1] (C read the Rust image and rewrote it).
#   4. C, fresh card K: creates its own save. 5. Rust boots K: load result 0, no file errors.
# Exit 0 when every step held. Needs GW_ISO_VANILLA (the workspace .env).
set -u
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"
ws="${GW_WORKSPACE:-E:/Projects/Melee Workspace}"
if [ -z "${GW_ISO_VANILLA:-}" ] && [ -f "$ws/.env" ]; then set -a; . "$ws/.env"; set +a; fi
rexe="$repo/build/cargo/${PROFILE:-release}/melee.exe"
cexe="${C_EXE:?set C_EXE to a melee-pc.exe built with the card shim}"
out="${1:-${TEMP:-/tmp}/saves-roundtrip.$$}"; mkdir -p "$out"; out="$(cd "$out" && pwd -W 2>/dev/null || pwd)"
export MELEE_VOLUME=3 MELEE_PAD_IGNORE_ADAPTER=1 MELEE_WINDOW_UNFOCUSED=1 MELEE_CARD_DIAG=1
fail=0
pad() { # name, lines...
  local f="$out/$1.txt"; shift; printf "%s\n" "$@" > "$f"; echo "$f"
}
create=$(pad create "120 0000 0 0" "3 0100 0 0" "200 0000 0 0" "3 0100 0 0" "400 0000 0 0")
idle=$(pad idle "400 0000 0 0")
# title -> main menu -> Options -> Rumble -> A toggles P1 -> B, B (saved on the way out)
rumble=$(pad rumble "60 0000 0 0" "3 1000 0 0" "150 0000 0 0" "4 0000 0 -100" "20 0000 0 0" "4 0000 0 -100" "20 0000 0 0" \
  "4 0000 0 -100" "20 0000 0 0" "3 0100 0 0" "80 0000 0 0" "3 0100 0 0" "80 0000 0 0" "3 0100 0 0" "40 0000 0 0" "3 0200 0 0" \
  "60 0000 0 0" "3 0200 0 0" "300 0000 0 0")
# C: past the special messages (unlock-all) to the main menu, where mnMain's entry saves
msgs=(); for i in 1 2 3 4 5 6 7 8; do msgs+=("27 0000 0 0" "3 0100 0 0"); done
cmenu=$(pad cmenu "60 0000 0 0" "3 1000 0 0" "${msgs[@]}" "600 0000 0 0")
rust() { # scenario card pad log frames
  MELEE_WINDOW_TITLE="saves: $1" MELEE_CARD_PATH="$2" MELEE_PAD_SCRIPT="$3" nice -n 19 "$rexe" play --no-audio \
    --iso "$GW_ISO_VANILLA" --skip-intro --max-frames "$5" >"$4" 2>&1
}
cgame() { # scenario card pad seconds
  ( GW_BUILD_ROOT="$(dirname "$cexe")" MELEE_WINDOW_TITLE="saves: $1" MELEE_CARD_PATH="$2" MELEE_PAD_SCRIPT="$3" MELEE_SKIP_INTRO=1 \
    MELEE_TURBO=0 MELEE_NO_ONBOARD=1 MELEE_MAX_SECONDS="$4" nice -n 19 bash "$ws/tools/port/run.sh" --realtime --max-seconds "$4" \
    "saves-rt-$1" --iso "$GW_ISO_VANILLA" >/dev/null 2>&1 )
  cp -f "$(dirname "$cexe")/runs/saves-rt-$1/melee-pc.log" "$out/c_$1.log" 2>/dev/null
}
check() { # what, ok?
  if [ "$2" = 1 ]; then echo "ok   $1"; else echo "FAIL $1"; fail=1; fi
}
R="$out/R"; K="$out/K"; mkdir -p "$R" "$K"
rust r-create "$R" "$create" "$out/r1.log" 700
rust r-msgs "$R" "$cmenu" "$out/r2m.log" 900  # the special messages are shown once, then saved as seen
rust r-rumble "$R" "$rumble" "$out/r2.log" 900
grep "save image -> native\|native -> save" "$out/r2.log" | tail -1
gci="$R/USA/Card A/01-GALE-SuperSmashBros0110290334.gci"
check "Rust created the save ($(stat -c %s "$gci" 2>/dev/null) bytes)" $([ -f "$gci" ] && echo 1 || echo 0)
cp -f "$gci" "$out/rust_written.gci"
cgame c-load-rust "$R" "$cmenu" 30
check "C loaded the Rust save without a prompt" $(grep -q "memcard decision=20" "$out/c_c-load-rust.log" && ! grep -q "memcard decision=\(2\|3\|5\|9\|1[0-7]\) " "$out/c_c-load-rust.log" && echo 1 || echo 0)
check "C wrote the save back" $(cmp -s "$gci" "$out/rust_written.gci" && echo 0 || echo 1)
rust r-load-c "$R" "$idle" "$out/r3.log" 120
grep "load result\|save image -> native" "$out/r3.log" | tail -2
check "Rust loaded the C-rewritten save cleanly" $(grep -q "load result 0 (per-file errors, save + 7 name-tag banks: \[0, 0, 0, 0, 0, 0, 0, 0\])" "$out/r3.log" && echo 1 || echo 0)
check "rumble P1 OFF survived Rust -> C -> Rust" $(grep "save image -> native" "$out/r3.log" | tail -1 | grep -q "rumble=\[0, 1, 1, 1\]" && echo 1 || echo 0)
cgame c-create "$K" "$create" 20
kg="$K/USA/Card A/01-GALE-SuperSmashBros0110290334.gci"
check "C created a save ($(stat -c %s "$kg" 2>/dev/null) bytes)" $([ -f "$kg" ] && echo 1 || echo 0)
rust r-load-cnew "$K" "$idle" "$out/r4.log" 120
grep "load result" "$out/r4.log" | tail -1
check "Rust loaded the C-created save cleanly" $(grep -q "load result 0 (per-file errors, save + 7 name-tag banks: \[0, 0, 0, 0, 0, 0, 0, 0\])" "$out/r4.log" && echo 1 || echo 0)
echo "cards and logs: $out"
exit $fail
