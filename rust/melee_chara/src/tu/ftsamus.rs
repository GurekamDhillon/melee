//! src/melee/ft/kinds/ftSamus/ftsamus.c: the take-damage / death callback `ftSs_Init_80128428`.
#![allow(non_snake_case, unused_unsafe)]

use melee_types::*;

type G = *mut HSD_GObj;

/// `ftSs_Init_80128428`: the take-damage / death callback.
pub fn ftSs_Init_80128428(gobj: G) {
    crate::tu::ftsamusspecialn::ftSs_SpecialN_80129258(gobj);
    crate::chars::r6::ftSamus::ftsamusspecials::ftSs_SpecialS_8012A640(gobj);
    melee_fighter::tu::ftco_0d95::ftco_800d9c98(gobj);
}

