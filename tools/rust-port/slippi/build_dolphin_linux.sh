#!/bin/bash
# SPDX-License-Identifier: MIT
# Build the local-test Slippi Dolphin (dolphin-emu-nogui) on Linux, for the loopback engine-vs-Dolphin harness
# (np_dolphin_pair.sh). The Linux counterpart of tools/rust-port/dolphin/build_dolphin.bat plus the Windows
# _build/dolphin-netplay link.
#   build_dolphin_linux.sh DIR      (DIR/dolphin-src, DIR/dolphin-build; the exe lands in DIR/dolphin-build/Binaries)
# Sources: project-slippi/dolphin mainline 0653df8 (the commit the repo's patches target), with
#   tools/rust-port/dolphin/slippi-dolphin-nhv1.patch   (state log, SLIPPI_LOCAL_TEST loopback binding, no STUN,
#                                                        SLIPPI_CHECKSUM_LOG, SLIPPI_PIPE_PREFIX)
#   tools/rust-port/slippi/dolphin_enet_mm.patch          (SLIPPI_MM_ENET; unused here, applied to match Windows)
#   tools/rust-port/slippi/dolphin_localtest_rust.patch   (Rust side of SLIPPI_LOCAL_TEST: every web request refused
#                                                        before DNS, no browser pages, matchmaking only to ws://127.0.0.1)
#   tools/rust-port/slippi/dolphin_linux_fixups.patch     (what MSVC's precompiled header hid)
# Needs cmake, ninja, gcc and cargo (any recent stable: the pinned 1.88.0 is overridden with RUSTUP_TOOLCHAIN or the
# system cargo). All C/C++ libraries are the bundled ones (USE_SYSTEM_LIBS=OFF: Arch's fmt 12 does not build 0653df8);
# GCC 16 defaults to C23, which the bundled libiconv 1.14 and SDL do not compile as: C is built as gnu17, SDL is off.
# CMake 4 needs CMAKE_POLICY_VERSION_MINIMUM=3.5 for the old Externals. Builds go through the laptop's build slots
# when ~/melee_out/locks/build.sh exists. Never commit the sources or the build.
set -e
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"; rp="$(cd "$here/.." && pwd)"
D=${1:?usage: build_dolphin_linux.sh DIR}; mkdir -p "$D"; D="$(cd "$D" && pwd)"
S=$D/dolphin-src; B=$D/dolphin-build
lock=(); [ -x "$HOME/melee_out/locks/build.sh" ] && lock=("$HOME/melee_out/locks/build.sh")
if [ ! -d "$S/.git" ]; then
  git clone https://github.com/project-slippi/dolphin.git "$S"
  git -C "$S" checkout -q 0653df8194eac2392dea9fe26276b70f3c87fd89
  git -C "$S" submodule update --init --recursive --depth 1 -j 8
fi
if [ ! -f "$S/Source/Core/Core/Slippi/NHV1Oracle.cpp" ]; then
  git -C "$S" apply "$rp/dolphin/slippi-dolphin-nhv1.patch" 2>&1 | grep -v "unable to rmdir" || true
  git -C "$S" apply --ignore-whitespace "$here/dolphin_enet_mm.patch"
  git -C "$S/Externals/SlippiRustExtensions" apply "$here/dolphin_localtest_rust.patch"
  git -C "$S" apply "$here/dolphin_linux_fixups.patch"
fi
if command -v rustup >/dev/null && [ -z "$RUSTUP_TOOLCHAIN" ]; then
  export RUSTUP_TOOLCHAIN=$(rustup toolchain list | grep -v nightly | head -1 | cut -d' ' -f1)
fi
mkdir -p "$B"; cd "$B"
[ -f build.ninja ] || "${lock[@]}" cmake -G Ninja -DCMAKE_POLICY_VERSION_MINIMUM=3.5 -DUSE_SYSTEM_LIBS=OFF \
  -DCMAKE_BUILD_TYPE=Release -DENABLE_QT=OFF -DENABLE_NOGUI=ON -DENABLE_TESTS=OFF -DENABLE_AUTOUPDATE=OFF \
  -DUSE_DISCORD_PRESENCE=OFF -DENABLE_CLI_TOOL=OFF -DENABLE_ANALYTICS=OFF -DLINUX_LOCAL_DEV=ON -DENABLE_SDL=OFF -DCMAKE_C_FLAGS=-std=gnu17 "$S"
"${lock[@]}" ninja -j "${JOBS:-8}" dolphin-nogui
# runtime files next to the exe (LINUX_LOCAL_DEV): the Slippi Sys with its online Gecko codeset
[ -d Binaries/Sys ] || cp -r "$S/Data/Sys" Binaries/Sys
echo "dolphin built: $B/Binaries/dolphin-emu-nogui"
