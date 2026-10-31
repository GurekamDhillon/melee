//! src/melee/ft/kinds/ftSamus/ftsamus.c: the take-damage / death callback `ftSs_Init_80128428`.
#![allow(non_snake_case, unused_unsafe)]

use melee_types::*;

type G = *mut HSD_GObj;

/// Calls into other TUs (settable hooks; unset they run the port).
pub mod ext {
    use melee_types::*;
    crate::port_hook!(FTSS_SPECIALN_80129258, ftSs_SpecialN_80129258, crate::tu::ftsamusspecialn::ftSs_SpecialN_80129258, (gobj: *mut HSD_GObj));
    crate::port_hook!(FTSS_SPECIALS_8012A640, ftSs_SpecialS_8012A640, crate::chars::r6::ftSamus::ftsamusspecials::ftSs_SpecialS_8012A640, (gobj: *mut HSD_GObj));
    crate::port_hook!(FTCO_800D9C98, ftCo_800D9C98, melee_fighter::tu::ftco_0d95::ftco_800d9c98, (gobj: *mut HSD_GObj));
}

/// `ftSs_Init_80128428`: the take-damage / death callback.
pub fn ftSs_Init_80128428(gobj: G) {
    melee_core::ct!("ftSs_Init_80128428");
    ext::ftSs_SpecialN_80129258(gobj);
    ext::ftSs_SpecialS_8012A640(gobj);
    ext::ftCo_800D9C98(gobj);
}

/// `ftSs_CostumeList`: zero-initialised BSS array of the C (`UnkCostumeStruct ftSs_CostumeList[5]`), filled when the costumes load.
pub static mut ftSs_CostumeList: [melee_types::UnkCostumeStruct; 5] = unsafe { core::mem::zeroed() };
