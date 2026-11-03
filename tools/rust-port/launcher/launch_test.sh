#!/bin/bash
# Start Melee Unlocked's launcher (patched with the Rust engine, ours/rust-engine in the scratch clone) hidden, let it
# press PLAY itself (MELEE_LAUNCHER_TEST_AUTOPLAY), and check that the Rust engine played one VS match and exited 0.
# No OS input: the launcher's test mode keeps its window off the desktop; MELEE_PAD_SCRIPT drives port 1.
#   bash tools/rust-port/launcher/launch_test.sh <test folder with MeleeUnlockedLauncher.exe, launcher.ini engine=2>
# A folder made from package.sh (rust/melee.exe inside it) runs that exe, else this checkout's build. SEGMENT=3 picks
# the Rust Port on the GAME BUILD row first (as a click does; launcher.ini may then say engine=1). BUILD=1 runs the
# Build tab's flow (disc check, then the engine's hidden --frames 30 run) instead of a match: "BUILD OK".
set -u
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"
T="$1"
export MELEE_LAUNCHER_TEST=1 MELEE_LAUNCHER_TEST_AUTOPLAY=1 LOCALAPPDATA="$(cygpath -w "$T/localappdata")"
export MELEE_LAUNCHER_TEST_LAUNCH_LOG="$(cygpath -w "$T/launch-cmd.txt")"
export MELEE_LAUNCHER_TEST_GAME_ARGS="--skip-intro --time-secs 30 --quit-after-matches 1 --max-frames 40000"
[ -f "$T/rust/melee.exe" ] || export MELEE_RUST_EXE="$(cygpath -w "$repo/build/cargo/${PROFILE:-release}/melee.exe")"
[ -n "${SEGMENT:-}" ] && export MELEE_LAUNCHER_TEST_SEGMENT="$SEGMENT"
export MELEE_PAD_SCRIPT="$(cygpath -w "$repo/tools/rust-port/play/retail_full.txt")"
export MELEE_VOLUME=3 MELEE_PAD_IGNORE_ADAPTER=1 MELEE_WINDOW_UNFOCUSED=1 MELEE_WINDOW_TITLE="launcher: vs match via MU launcher" MELEE_CARD=0
rm -f "$T/melee_port.log" "$T/launch-cmd.txt" "$T/launcher_build.log"
if [ -n "${BUILD:-}" ]; then
  export MELEE_LAUNCHER_TEST_AUTOBUILD=1
  "$T/MeleeUnlockedLauncher.exe"; rc=$?
  echo "launcher exit: $rc"; grep -E "^launcher-compat|headless" "$T/launcher_build.log" | sed 's/--iso .*.[iI][sS][oO]/--iso <disc>/' | head -20
  done=$(sed -n 's/^build done //p' "$T/launch-cmd.txt" 2>/dev/null)
  echo "build result seen by the launcher: ${done:-none}"
  [ "$done" = 0 ] && grep -q -- "--headless --max-frames 30" "$T/launcher_build.log" && echo "BUILD OK" || { echo "BUILD FAIL"; exit 1; }
  exit 0
fi
"$T/MeleeUnlockedLauncher.exe"; rc=$?
echo "launcher exit: $rc"
grep -q "^engine=2" "$T/launcher.ini" && echo "launcher.ini: engine=2"
echo "--- command the launcher ran:"; grep -v "^game exit" "$T/launch-cmd.txt" | sed 's/--iso "[^"]*"/--iso "<disc>"/' 
echo "--- game log:"; grep -E "^launcher-compat|^play: (display|tick .*scene (vs|results)|quit|closed)" "$T/melee_port.log" | sed 's/--iso .*.[iI][sS][oO]/--iso <disc>/'
# the retail flow logs each scene; a match is the results scene after vs/suddendeath, then the planned quit
n=$(grep -c "scene results" "$T/melee_port.log"); quit=$(grep -c "^play: quit after 1 matches" "$T/melee_port.log")
game=$(sed -n 's/^game exit //p' "$T/launch-cmd.txt")
echo "results scenes: $n, quit line: $quit, game exit code seen by the launcher: ${game:-none}"
[ "$game" = 0 ] && [ "$n" -ge 1 ] && [ "$quit" = 1 ] && echo "LAUNCH OK" || { echo "LAUNCH FAIL"; exit 1; }
