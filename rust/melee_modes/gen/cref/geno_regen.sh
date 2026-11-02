#!/usr/bin/env bash
# Regenerate rust/melee_modes/src/tu/{geno_game,geno_tests}.rs from the current pc/geno C.
#   rust/melee_modes/gen/cref/geno_regen.sh      (worktree root; WSL c2rust per tools/rust-port/setup_wsl.sh)
# raw c2rust -> geno_adapt.py (melee_types layouts) -> geno_finish.py (fma sites, statics)
# -> insert_ct.py markers -> geno_keep_prelude.py (keeps the committed ext adapters).
# Run rust/melee_types/gen/gen_types.py first when a Geno header changed. Afterwards splice only the
# geno_game/geno_tests hook lines of a fresh tools/rust-port/link/gen_link.py run into the committed
# rust/melee_link/src/gen_table.rs: a whole-table regeneration currently drops unrelated hooks.
set -euo pipefail  # a failed step leaves the TU files half-regenerated: restore them from git before rerunning
cd "$(dirname "$0")/../../../.."
tmp=$(mktemp -d); trap 'rm -rf "$tmp"' EXIT
for stem in geno_game geno_tests; do cp "rust/melee_modes/src/tu/$stem.rs" "$tmp/$stem.rs"; done
bash tools/rust-port/transpile.sh pc/geno/geno_game.c pc/geno/geno_tests.c
(cd rust/melee_modes/gen/cref && python geno_adapt.py geno_game geno_tests && python geno_finish.py geno_game geno_tests)
for stem in geno_game geno_tests; do
  python tools/rust-port/trace/insert_ct.py "rust/melee_modes/src/tu/$stem.rs"
  python rust/melee_modes/gen/cref/geno_keep_prelude.py "$tmp/$stem.rs" "rust/melee_modes/src/tu/$stem.rs"
done
