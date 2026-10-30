#!/bin/bash
# Regenerate the ftanim.c state oracle (run from anywhere).
set -e
cd "$(dirname "$0")/../../../.."
python rust/melee_fighter/gen/mtu/fill.py ftanim src/melee/ft/ftanim.c HSD_GObj,HSD_JObj,Fighter,Item,HSD_DObj Fighter,HSD_JObj,HSD_GObj,FighterBone
python rust/melee_fighter/gen/mtu/stubs.py ftanim src/melee/ft/ftanim.c rust/melee_fighter/src/tu/ftanim.rs \
  --ret "Mex_CostumeVisIdx=(int) (fv() % 4)|(fv() % 4) as i32" --ret "ftParts_80075240=&TOBJ|&raw mut (*crate::TOBJ)" --ret "ftPartsRemap=({ u32 q_ = fv(); q_ % 5 == 0 ? 0xFFu : (q_ >> 3) % 16; })|{ let q = fv(); if q % 5 == 0 { 0xFF } else { ((q >> 3) % 16) as i32 } }" --ret "ftParts_GetBoneIndex=(int) (fv() % 16)|(fv() % 16) as i32"
python rust/melee_fighter/gen/cref/tu/run_tu.py ftanim_ops
