#!/usr/bin/env bash
# One-time setup of c2rust inside WSL (Debian). No sudo needed; everything lives in ~/opt.
#   wsl -d Debian -- bash "/mnt/e/Projects/Melee Workspace/worktrees/rust-port/tools/rust-port/setup_wsl.sh"
# Installs: LLVM/clang 18.1.8 prebuilt (c2rust 0.22 builds against it), libtinfo.so.5 from the
# Debian 11 package (the LLVM 18 tarball links it), and `cargo install --locked c2rust` (0.22.1).
# Writes ~/opt/c2rust-env.sh, which transpile.sh sources.
set -euo pipefail
OPT=$HOME/opt; mkdir -p "$OPT/compat"; cd "$OPT"
LLVM=$OPT/clang+llvm-18.1.8-x86_64-linux-gnu-ubuntu-18.04
if [ ! -x "$LLVM/bin/clang" ]; then
  curl -fsSL -o llvm18.tar.xz https://github.com/llvm/llvm-project/releases/download/llvmorg-18.1.8/clang+llvm-18.1.8-x86_64-linux-gnu-ubuntu-18.04.tar.xz
  tar xf llvm18.tar.xz && rm llvm18.tar.xz
fi
if [ ! -e "$OPT/compat/libtinfo.so.5" ]; then
  curl -fsSL -o t5.deb http://deb.debian.org/debian/pool/main/n/ncurses/libtinfo5_6.2+20201114-2+deb11u2_amd64.deb
  dpkg-deb -x t5.deb t5 && cp -a t5/lib/x86_64-linux-gnu/libtinfo.so.5* compat/ && rm -rf t5 t5.deb
fi
ln -sf /lib/x86_64-linux-gnu/libtinfo.so.6 "$OPT/compat/libtinfo.so"   # for cmake's FindTerminfo
cat > "$OPT/c2rust-env.sh" <<ENV
export LLVM=$LLVM
export LD_LIBRARY_PATH=$OPT/compat:$LLVM/lib
export LLVM_CONFIG_PATH=$LLVM/bin/llvm-config
export LIBCLANG_PATH=$LLVM/lib
export CMAKE_LIBRARY_PATH=$OPT/compat
export PATH=$LLVM/bin:\$HOME/.cargo/bin:\$PATH
ENV
. "$OPT/c2rust-env.sh"
command -v c2rust >/dev/null || RUSTFLAGS="-L $OPT/compat -C link-arg=-Wl,-rpath,$OPT/compat:$LLVM/lib" \
  cargo install --locked c2rust
c2rust --version
