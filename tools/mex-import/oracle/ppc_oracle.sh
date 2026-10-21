#!/usr/bin/env bash
# The gw_ppc.c oracle: cuts the interpreter's execute core (gw_ppc_execute, _x, _cr, _fp, the CR /
# XER / condition / rotate helpers, gw_ppc_bridge_tail, gw_ppc_sqrt and the gw_ppc_machine type)
# out of pc/platform/gw_ppc.c at build time (no copy edited by hand), compiles it the way shipping
# shims are built (clang --target=i686-pc-windows-msvc -O2) against a flat test memory and a
# recording bridge (ppc_oracle.c), runs random instruction programs and writes
# tests/data/ppc_oracle.txt, compared by tests/ppc_oracle.rs.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../../.." && pwd)"
out="$root/rust/target/ppc-oracle"
CLANG="${CLANG:-clang}"
command -v "$CLANG" >/dev/null 2>&1 || CLANG="/c/Program Files/LLVM/bin/clang.exe"
mkdir -p "$out"
src="$root/pc/platform/gw_ppc.c"
cut() { # the definition starting at the line matching $1, through its closing brace at column 0
  awk -v pat="$1" 'index($0, pat) == 1 && !/;$/ {on=1} on {print} on && /^}/ {exit}' "$src"
}
line() { # a one-line definition
  awk -v pat="$1" 'index($0, pat) == 1 {print; exit}' "$src"
}
cut 'typedef struct gw_ppc_machine {' > "$out/ppc_cut_types.c"
{
  for f in \
    'static uint32_t gw_ppc_cr_bit(const gw_ppc_ctx *c, uint32_t bi) {' \
    'static void gw_ppc_cr_set_bit(gw_ppc_ctx *c, uint32_t bi, uint32_t v) {' \
    'static void gw_ppc_cr_set_field(gw_ppc_ctx *c, uint32_t crfd, uint32_t field) {' \
    'static uint32_t gw_ppc_cr_get_field(const gw_ppc_ctx *c, uint32_t crfd) {' \
    'static uint32_t gw_ppc_xer_ca(const gw_ppc_ctx *c) {' \
    'static void gw_ppc_xer_set_ca(gw_ppc_ctx *c, uint32_t v) {' \
    'static void gw_ppc_cr0_cmp(gw_ppc_ctx *c, uint32_t res) {' \
    'static void gw_ppc_cmp(gw_ppc_ctx *c, uint32_t crfd, uint32_t a, uint32_t b, int is_unsigned) {' \
    'static int gw_ppc_cond(gw_ppc_machine *m, uint32_t bo, uint32_t bi) {' \
    'static uint32_t gw_ppc_rotl(uint32_t v, uint32_t sh) {' \
    'static uint32_t gw_ppc_mask(uint32_t mb, uint32_t me) {' \
    'static int gw_ppc_bridge_tail(gw_ppc_machine *m, uint32_t target) {' \
    'static int gw_ppc_execute(gw_ppc_machine *m, uint32_t insn) {' \
    'static int gw_ppc_execute_x(gw_ppc_machine *m, uint32_t insn) {' \
    'static int gw_ppc_execute_cr(gw_ppc_machine *m, uint32_t insn) {' \
    'static int gw_ppc_execute_fp(gw_ppc_machine *m, uint32_t insn, int single) {'; do
    n=$(cut "$f" | wc -l)
    [ "$n" -gt 1 ] || { echo "cut failed: $f" >&2; exit 1; }
    cut "$f"
  done
  line 'static double gw_ppc_sqrt(double x) {'
} > "$out/ppc_cut_funcs.c"
"$CLANG" --target=i686-pc-windows-msvc -O2 -w -D_CRT_SECURE_NO_WARNINGS -I"$out" -I"$here" \
  -I"$root/pc/platform" "$here/ppc_oracle.c" -o "$out/ppc_oracle.exe"
"$out/ppc_oracle.exe" "${1:-6000}" | tr -d '\r' > "$here/../tests/data/ppc_oracle.txt"
echo "wrote tests/data/ppc_oracle.txt ($(wc -l < "$here/../tests/data/ppc_oracle.txt") lines)"
