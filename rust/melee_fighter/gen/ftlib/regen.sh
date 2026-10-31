#!/bin/bash
# Regenerate rust/melee_fighter/src/tu/ftlib.rs from the game's C (libclang, see c2rs.py). Run from the worktree root.
set -e
python rust/melee_world/gen/mtu/c2rs.py src/melee/ft/ftlib.c rust/melee_fighter/src/tu/ftlib.rs \
  --head rust/melee_fighter/gen/ftlib/head.rs --pub-all --all-extern --check
# one shared static per global (traps.md): the ext copies become re-exports of the canonical definitions
OUT=rust/melee_fighter/src/tu/ftlib.rs
sed -i \
  -e 's|^    pub static mut CostumeListsForeachCharacter: .*|    pub use crate::tu::ftdata::CostumeListsForeachCharacter;|' \
  -e 's|^    pub static mut HSD_GObjPLinkHead: .*|    pub use melee_runtime::tu::gobj_native::HSD_GObjPLinkHead;|' \
  -e 's|^    pub static mut ftData_UnkBytePerCharacter: .*|    pub use melee_runtime::shared::ftData_UnkBytePerCharacter;|' \
  -e 's|^    pub static mut p_ftCommonData: .*|    pub use melee_runtime::shared::p_ftCommonData;|' $OUT
