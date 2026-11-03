#!/bin/bash
# Lay out the Rust engine for Melee Unlocked's launcher: a folder to extract over his release folder, so the patched
# launcher (ours/rust-engine) finds <game folder>\rust\melee.exe and shows the Rust Port on its GAME BUILD row.
#   bash tools/rust-port/launcher/package.sh <out dir on E:> [patched MeleeUnlockedLauncher.exe]
# Builds melee.exe (release) unless MELEE_EXE names one. Ships no disc data: the folder is checked against the list of
# files it may hold, and any .iso/.dat/.usd/.ssm/.hps/.thp/.dol/.gci in it fails the run. Writes <out>.zip beside it.
set -euo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"
OUT="${1:?usage: package.sh <out dir> [launcher exe]}"; LAUNCHER="${2:-}"
exe="${MELEE_EXE:-}"
if [ -z "$exe" ]; then
  export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$repo/build/cargo}"
  (cd "$repo/rust" && cargo build --release -p melee --bin melee)
  exe="$CARGO_TARGET_DIR/release/melee.exe"
fi
[ -f "$exe" ] || { echo "no melee.exe at $exe"; exit 1; }
rm -rf "$OUT"; mkdir -p "$OUT/rust"
cp "$exe" "$OUT/rust/melee.exe"
hash="$(git -C "$repo" rev-parse --short HEAD)"; ver="$(sed -n 's/^version = "\(.*\)"/\1/p' "$repo/rust/Cargo.toml" | head -1)"
echo "melee $ver ($hash), Rust port" > "$OUT/rust/version.txt"
[ -f "$repo/LICENSE" ] && cp "$repo/LICENSE" "$OUT/rust/LICENSE.txt"
cat > "$OUT/rust/README.txt" <<EOF
Melee, native Rust port ($ver, $hash): a third engine for the Melee Unlocked launcher.

Extract this folder over the launcher's folder (the one with MeleeUnlockedLauncher.exe), so this file ends up in
<launcher folder>\\rust\\. The launcher (with the Rust engine patch) then shows "Rust Port" on its GAME BUILD row.
Your Melee NTSC 1.02 disc is the one the launcher already knows; nothing from it is in this folder.

Reads the launcher's port-settings.ini (display, volume, keyboard and controller bindings, background input).
Saves go to User\\GC\\RustPort, replays (.mrpl) to <replay folder>\\RustPort, the log to melee_port.log.
Not supported yet: online lobbies, mods that change the game's code, .slp replays. Those are refused with exit code 3.
EOF
if [ -n "$LAUNCHER" ]; then cp "$LAUNCHER" "$OUT/MeleeUnlockedLauncher.exe"; fi
# nothing but the expected files, and nothing disc-shaped
bad=$(cd "$OUT" && find . -type f | grep -viE '^\./(rust/(melee\.exe|version\.txt|README\.txt|LICENSE\.txt)|MeleeUnlockedLauncher\.exe)$' || true)
disc=$(cd "$OUT" && find . -type f -iregex '.*\.\(iso\|gcm\|dat\|usd\|ssm\|hps\|thp\|dol\|gci\|sem\)$' || true)
[ -z "$bad$disc" ] || { echo "unexpected files in the package: $bad $disc"; exit 1; }
rm -f "$OUT.zip"
(cd "$OUT" && powershell.exe -NoProfile -Command "Compress-Archive -Path * -DestinationPath '$(cygpath -w "$OUT.zip")'")
echo "PACKAGE OK: $(cygpath -w "$OUT") ($(cd "$OUT" && find . -type f | wc -l) files), $(cygpath -w "$OUT.zip")"
