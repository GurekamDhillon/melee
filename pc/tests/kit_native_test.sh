#!/bin/bash
# Run from this worktree. Toolchain setup is read-only; all outputs stay here.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/../.."
export GW_MELEE="$(pwd -W)"
export GW_BUILD_ROOT="$GW_MELEE/build/texupdate"
. ../../tools/port/portlib.sh
test_root="$GW_BUILD_ROOT/native-tests"
mkdir -p "$test_root"
"$GW_CLANG" --target=i686-pc-windows-msvc -std=c11 -D_CRT_SECURE_NO_WARNINGS -ffunction-sections \
    -I "$GW_MELEE/pc/platform" -c "$GW_MELEE/pc/tests/kit_native_test.c" -o "$test_root/kit_native.obj"
"$GW_CLANG" --target=i686-pc-windows-msvc -std=c++17 -ffunction-sections \
    -I "$GW_MELEE/pc/platform" "$GW_MELEE/pc/tests/kit_texture_test.cpp" "$test_root/kit_native.obj" \
    -o "$test_root/kit_texture.next.exe" -Xlinker /OPT:REF
mv -f "$test_root/kit_texture.next.exe" "$test_root/kit_texture.exe"
"$test_root/kit_texture.exe"
