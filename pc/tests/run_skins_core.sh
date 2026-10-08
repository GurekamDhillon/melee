#!/bin/bash
# Native test of the skin registry's pure logic (pc/platform/gw_skins_core.h).
set -euo pipefail
cd "$(dirname "$0")/../../../.."
source tools/port/portlib.sh
mkdir -p "$GW_BUILD_ROOT/native-tests"
"$GW_CLANG" --target=i686-pc-windows-msvc -std=c11 -Wall "$GW_MELEE/pc/tests/skins_core_test.c" -o "$GW_BUILD_ROOT/native-tests/skins-core.exe"
"$GW_BUILD_ROOT/native-tests/skins-core.exe"
