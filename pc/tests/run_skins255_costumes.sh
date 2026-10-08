#!/bin/bash
set -euo pipefail
cd "$(dirname "$0")/../../../.."
source tools/port/portlib.sh
mkdir -p "$GW_BUILD_ROOT/native-tests"
"$GW_CLANG" --target=i686-pc-windows-msvc -std=c11 "$GW_MELEE/pc/tests/skins255_costumes_test.c" -o "$GW_BUILD_ROOT/native-tests/skins255-costumes.exe"
"$GW_BUILD_ROOT/native-tests/skins255-costumes.exe"
