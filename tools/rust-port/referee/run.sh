#!/bin/bash
# Run a Gekko probe on Dolphin's interpreter, headless, and print its REF lines.
#   bash tools/rust-port/referee/run.sh nanprobe|intprobe [OUT]
# Needs: clang + ld.lld (any recent LLVM with the PowerPC target), dolphin-emu, xvfb-run (Xvfb). No disc, no game data.
set -eu
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
name=${1:?probe name (nanprobe, intprobe)}; out=${2:-${TMPDIR:-/tmp}/gekko-referee}
mkdir -p "$out/user/Config" "$out/user/Logs"
cp "$here/Logger.ini" "$here/Dolphin.ini" "$out/user/Config/"
clang --target=powerpc-unknown-eabi -mcpu=750 -O1 -ffreestanding -nostdlib -fno-builtin -c "$here/$name.c" -o "$out/$name.o"
ld.lld -m elf32ppc -e _start -Ttext=0x80003100 --section-start=.data=0x80100000 -o "$out/$name.elf" "$out/$name.o"
: > "$out/user/Logs/dolphin.log"
# Dolphin never exits on its own here (the probe ends in a loop): a hard timeout, and a kill of what this run left behind
timeout -s INT -k 5 25 xvfb-run -a dolphin-emu -u "$out/user" -b -v Null -e "$out/$name.elf" > "$out/$name.run.txt" 2>&1 || true
pkill -9 -f -- "-e $out/$name.elf" 2>/dev/null || true
grep -h 'REF' "$out/user/Logs/dolphin.log" | sed 's/.*REF/REF/'
