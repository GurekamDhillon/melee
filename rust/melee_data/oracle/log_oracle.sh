#!/usr/bin/env bash
# Build the gw_log.c logger oracle (the gl_* logger cut out of the shipping file, sink replaced) and run
# log_script.txt through it -> ../tests/data/log_oracle.txt
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../../.." && pwd)"
out="$root/rust/target/log-oracle"
CLANG="${CLANG:-clang}"
command -v "$CLANG" >/dev/null 2>&1 || CLANG="/c/Program Files/LLVM/bin/clang.exe"
mkdir -p "$out"
awk 'NR>=80 && NR<=619 { if (NR==341) print "  gl_emit(s);"; if (NR>=341 && NR<=364) next; print }' "$root/pc/platform/gw_log.c" > "$out/../log-oracle/log_cut.inc"
python "$here/log_script.py" > "$here/../tests/data/log_script.txt"
"$CLANG" -O1 -w -D_CRT_SECURE_NO_WARNINGS -I"$out" "$here/log_oracle.c" -o "$out/log_oracle.exe"
"$out/log_oracle.exe" "$here/../tests/data/log_script.txt" | tr -d '\r' > "$here/../tests/data/log_oracle.txt"
echo "wrote tests/data/log_oracle.txt ($(wc -l < "$here/../tests/data/log_oracle.txt") lines)"
