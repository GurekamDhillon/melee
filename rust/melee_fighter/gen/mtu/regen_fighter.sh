#!/bin/bash
# Regenerate the fighter.c state oracle (run from anywhere): field fill/hash, recording stubs, C expected output.
set -e
cd "$(dirname "$0")/../../../.."
python rust/melee_fighter/gen/mtu/fill.py fighter src/melee/ft/fighter.c HSD_GObj,HSD_JObj,Fighter,Item,HSD_DObj Fighter,HSD_JObj,HSD_GObj,plAllocInfo
KIND='(int) (fv() % 70) - 20|((fv() % 70) as i32) - 20'
python rust/melee_fighter/gen/mtu/stubs.py fighter src/melee/ft/fighter.c rust/melee_fighter/src/tu/fighter.rs \
  --skip atan2f --skip ftParts_SetupParts --notrace Snap_CuratedRegion:1 \
  --outvec Player_LoadPlayerCoords:1 --outvec lb_8000B1CC:2 \
  --ret "Geno_DefineBaseKind=$KIND" --ret "Mex_InternalForPortKind=$KIND" --ret "Mex_FtBaseKind=$KIND" \
  --ret "Geno_DefineAiLike=$KIND" --ret "gm_MexVanillaKind=$KIND" \
  --ret "ftParts_GetBoneIndex=(int) (fv() % 16)|(fv() % 16) as i32" \
  --ret "GObj_Create=&GO[0]|&raw mut (*crate::GO)[0]" \
  --ret "HSD_ObjAlloc=(a0 == &fighter_alloc_data ? (void*) &FT[0] : (void*) PADB[95])|if a0 == &raw mut m::fighter_alloc_data { &raw mut (*crate::FT)[0] as *mut core::ffi::c_void } else { (*crate::PADB)[95].as_mut_ptr() as *mut core::ffi::c_void }" \
  --ret "Player_GetEntityAtIndex=pick_HSD_GObj()|crate::pick_HSD_GObj()"
python rust/melee_fighter/gen/cref/tu/run_tu.py fighter_ops
