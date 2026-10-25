#!/bin/bash
# Regenerate rust/melee_fighter/src/tu/ftCo_0A01_ai.rs from the game's C (libclang, see c2rs.py).
# Run from the worktree root. Hand-structured goto functions live in gen/ai/hand/.
set -e
G=rust/melee_fighter/gen/ai
SIB=ftcpuattack:src/melee/ft/ftcpuattack.c,ftcmdscript:src/melee/ft/ftcmdscript.c
TMP=src/melee/ft/kinds/ftCommon/ftCo_0A01_ai.c
python $G/degoto.py src/melee/ft/kinds/ftCommon/ftCo_0A01.c $TMP
python rust/melee_world/gen/mtu/c2rs.py $TMP rust/melee_fighter/src/tu/ftCo_0A01_ai.rs \
  --head $G/head.rs --hand $G/hand --sib $SIB --check
rm -f $TMP
# post-fixes: Rust `bool` returns of the hand-ported ftcpuattack fns; a local named like an enum constant.
OUT=rust/melee_fighter/src/tu/ftCo_0A01_ai.rs
sed -i -e 's/super::ftcpuattack::ftCo_800B9CBC(fp) != 0/super::ftcpuattack::ftCo_800B9CBC(fp)/' \
       -e 's/super::ftcpuattack::ftCo_800B8A9C(fp) != 0/super::ftcpuattack::ftCo_800B8A9C(fp)/' \
       -e 's/super::ftcpuattack::ftCo_800BA2E8(fp, temp_r0) == 0/!super::ftcpuattack::ftCo_800BA2E8(fp, temp_r0)/' \
       -e 's/\brunning\b/running_/g' $OUT
