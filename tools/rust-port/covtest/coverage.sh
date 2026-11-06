#!/bin/bash
# SPDX-License-Identifier: MIT
# Game-code branch coverage of a corpus run (notes/coverage-testing.md):
#   bash tools/rust-port/covtest/coverage.sh CORPUS_DIR [EXE]
# Merges CORPUS_DIR/prof/*.profdata (and any loose *.profraw under lanes/), exports with llvm-cov, and writes
# CORPUS_DIR/coverage/{summary.txt,functions.tsv,files.tsv} (report.py). llvm-profdata/llvm-cov come from
# rustup's llvm-tools (`rustup component add llvm-tools`): they must match rustc's LLVM (raw profile version).
set -eu
C=$1
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ws="${GW_WORKSPACE:-E:/Projects/Melee Workspace}"
EXE=${2:-$ws/_build/cargo-agents/covtest/cov/melee.exe}
L="$(rustc --print sysroot)/lib/rustlib/x86_64-pc-windows-msvc/bin"

shopt -s nullglob
# PROF_GLOB: which lane profiles (default all: prof/*.profdata); e.g. "l*" the CPU matches, "soak_*" the move soak
mkdir -p "$C/${COV_OUT:-coverage}"
in=("$C"/prof/${PROF_GLOB:-*}.profdata)
OUTD=${COV_OUT:-coverage}
"$L/llvm-profdata" merge -sparse "${in[@]}" -o "$C/$OUTD/all.profdata"
"$L/llvm-cov" export "$EXE" -instr-profile="$C/$OUTD/all.profdata" -skip-expansions > "$C/$OUTD/cov.json"
python -I "$here/report.py" "$C/$OUTD/cov.json" "$C/$OUTD"
rm -f "$C/$OUTD/cov.json"
