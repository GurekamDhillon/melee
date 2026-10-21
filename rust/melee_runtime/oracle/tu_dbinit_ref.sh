#!/usr/bin/env bash
# C reference for the pure pieces of the db TU ports (tu/dbinit.rs db_filter_dpad, tu/dbsound.rs
# db_peak_hold). Both bodies are cut verbatim (by line range, checked below) from the game C and
# compiled natively; integer-only, so contraction does not matter.
# Regenerates tests/data/tu_db_ref.txt. Usage: bash rust/melee_runtime/oracle/tu_db_ref.sh
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../../.." && pwd)"
out="${TMPDIR:-${TEMP:-/tmp}}/melee_runtime_tu_db_ref"
mkdir -p "$out"
sed -n '179,194p' "$root/src/melee/db/dbinit.c" > "$out/dpad.inc"
sed -n '64,72p' "$root/src/melee/db/dbsound.c" > "$out/peak.inc"
grep -q 'if (button & HSD_PAD_DPADUP)' "$out/dpad.inc" || { echo "dbinit.c moved" >&2; exit 1; }
grep -q 'db_SoundRelated_6B6C = 0;' "$out/peak.inc" || { echo "dbsound.c moved" >&2; exit 1; }
cat > "$out/drv.c" <<'C'
#include <stdio.h>
#define HSD_PAD_DPADLEFT 1
#define HSD_PAD_DPADRIGHT 2
#define HSD_PAD_DPADDOWN 4
#define HSD_PAD_DPADUP 8
static int filt(int button) {
#include "dpad.inc"
    return button;
}
static int db_SoundRelated_6B68, db_SoundRelated_6B6C;
static void peak(int x) {
#include "peak.inc"
}
int main(void) {
    unsigned s = 12345u;
    int i;
    for (i = 0; i < 64; i++) printf("dpad %d %d\n", i, filt(i));
    for (i = 0; i < 200; i++) { s = s * 1103515245u + 12345u; printf("dpad %d %d\n", (int) s, filt((int) s)); }
    for (i = 0; i < 600; i++) {
        s = s * 1103515245u + 12345u;
        int x = (int) ((s >> 16) % 64);
        if (i % 97 == 0) x = 63;
        peak(x);
        printf("peak %d %d %d\n", x, db_SoundRelated_6B68, db_SoundRelated_6B6C);
    }
    return 0;
}
C
CL="${CLANG:-/c/Program Files/LLVM/bin/clang.exe}"
"$CL" -O2 -w "$out/drv.c" -I"$out" -o "$out/drv.exe"
"$out/drv.exe" | tr -d '\r' > "$here/../tests/data/tu_db_ref.txt"
wc -l "$here/../tests/data/tu_db_ref.txt"
