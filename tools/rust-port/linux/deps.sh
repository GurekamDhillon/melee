#!/bin/bash
# SPDX-License-Identifier: MIT
# Linux build dependencies of the native engine (docs/rust-port/notes/linux.md).
#   bash tools/rust-port/linux/deps.sh          # with sudo: apt-get install the packages
#   NOSUDO=1 bash tools/rust-port/linux/deps.sh # without sudo: unpack the two -dev packages into ~/opt/linuxdeps
#                                               # and print the PKG_CONFIG_PATH to export
# Only build-time headers/.pc files are missing on a stock Debian/Ubuntu desktop: ALSA (cpal) and libudev (gilrs).
set -euo pipefail
PKGS="build-essential pkg-config libasound2-dev libudev-dev libvulkan1 mesa-vulkan-drivers libx11-dev libxkbcommon-x11-0 libwayland-dev"
if [ -z "${NOSUDO:-}" ]; then
  sudo apt-get update && sudo apt-get install -y $PKGS
  command -v cargo >/dev/null || echo "deps: install Rust with rustup (https://rustup.rs), stable toolchain"
  exit 0
fi
D=$HOME/opt/linuxdeps; mkdir -p "$D"; cd "$D"
for p in libasound2-dev libudev-dev; do ls ${p}_*.deb >/dev/null 2>&1 || apt-get download $p; dpkg-deb -x ${p}_*.deb root; done
R=$D/root; L=$R/usr/lib/x86_64-linux-gnu
for pc in alsa libudev; do
  sed -i "s#^prefix=.*#prefix=$R/usr#; s#^libdir=.*#libdir=$L#; s#^includedir=.*#includedir=$R/usr/include#" $L/pkgconfig/$pc.pc
done
ln -sf /usr/lib/x86_64-linux-gnu/libasound.so.2 $L/libasound.so
ln -sf /usr/lib/x86_64-linux-gnu/libudev.so.1 $L/libudev.so
echo "export PKG_CONFIG_PATH=$L/pkgconfig"
