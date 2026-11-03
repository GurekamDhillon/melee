#!/bin/bash
# Start Melee Unlocked's launcher (patched with the Rust engine, ours/rust-engine in the scratch clone) hidden, let it
# press PLAY itself (MELEE_LAUNCHER_TEST_AUTOPLAY), and check that the Rust engine played one VS match and exited 0.
# No OS input: the launcher's test mode keeps its window off the desktop; MELEE_PAD_SCRIPT drives port 1.
#   bash tools/rust-port/launcher/launch_test.sh <test folder with MeleeUnlockedLauncher.exe, launcher.ini engine=2>
set -u
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"
T="$1"
export MELEE_LAUNCHER_TEST=1 MELEE_LAUNCHER_TEST_AUTOPLAY=1 LOCALAPPDATA="$(cygpath -w "$T/localappdata")"
export MELEE_LAUNCHER_TEST_LAUNCH_LOG="$(cygpath -w "$T/launch-cmd.txt")"
export MELEE_LAUNCHER_TEST_GAME_ARGS="--skip-intro --time-secs 30 --quit-after-matches 1 --max-frames 40000"
export MELEE_RUST_EXE="$(cygpath -w "$repo/build/cargo/${PROFILE:-release}/melee.exe")"
export MELEE_PAD_SCRIPT="$(cygpath -w "$repo/tools/rust-port/play/retail_full.txt")"
export MELEE_VOLUME=3 MELEE_PAD_IGNORE_ADAPTER=1 MELEE_WINDOW_UNFOCUSED=1 MELEE_WINDOW_TITLE="launcher: vs match via MU launcher" MELEE_CARD=0
rm -f "$T/melee_port.log" "$T/launch-cmd.txt"
"$T/MeleeUnlockedLauncher.exe"; rc=$?
echo "launcher exit: $rc"
echo "--- command the launcher ran:"; grep -v "^game exit" "$T/launch-cmd.txt" | sed 's/--iso "[^"]*"/--iso "<disc>"/' 
echo "--- game log:"; grep -E "^launcher-compat|^play: (display|tick .*scene (vs|results)|quit|closed)" "$T/melee_port.log" | sed 's/--iso [^ ]*/--iso <disc>/'
# the retail flow logs each scene; a match is the results scene after vs/suddendeath, then the planned quit
n=$(grep -c "scene results" "$T/melee_port.log"); quit=$(grep -c "^play: quit after 1 matches" "$T/melee_port.log")
game=$(sed -n 's/^game exit //p' "$T/launch-cmd.txt")
echo "results scenes: $n, quit line: $quit, game exit code seen by the launcher: ${game:-none}"
[ "$game" = 0 ] && [ "$n" -ge 1 ] && [ "$quit" = 1 ] && echo "LAUNCH OK" || { echo "LAUNCH FAIL"; exit 1; }
