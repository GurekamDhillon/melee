//! src/melee/mn/mninfobonus.c: native port (c2rust-derived body; callees of unported TUs are `ext` TODO hooks).
#![allow(non_snake_case, non_upper_case_globals, non_camel_case_types, unused_unsafe, unused_mut, unused_assignments, unused_variables, unused_parens, dead_code, static_mut_refs, clippy::all)]

use melee_types::*;
#[allow(unused_imports)] use self::ext::*;
#[allow(unused_imports)] use super::mnmain::{mn_80229624, mn_80229894, mn_804A04F0, mn_804D6BB8, mn_804D6BC8};

pub type u8_0 = u8;
pub type u16_0 = u16;
pub type u32_0 = u32;
pub type u64_0 = u64;
pub type f32_0 = f32;
pub type f64_0 = f64;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct mnInfoBonus_804A09B0_t {
    pub x0: ::core::ffi::c_int,
    pub x4: [*mut HSD_Text; 5],
    pub x18: [*mut HSD_Text; 5],
    pub x2C: [*mut HSD_Text; 5],
    pub x40: *mut HSD_Text,
    pub x44: ::core::ffi::c_int,
    pub x48: ::core::ffi::c_int,
    pub x4C: *mut HSD_GObj,
    pub x50: StaticModelDesc,
}
melee_core::gc_layout!(mnInfoBonus_804A09B0_t, "mnInfoBonus_804A09B0_t", size 0x60, align 4, [
    x0 @ 0x0: 4 "c_int",
    x4 @ 0x4: 20 "[*mut HSD_Text; 5]",
    x18 @ 0x18: 20 "[*mut HSD_Text; 5]",
    x2C @ 0x2C: 20 "[*mut HSD_Text; 5]",
    x40 @ 0x40: 4 "*mut HSD_Text",
    x44 @ 0x44: 4 "c_int",
    x48 @ 0x48: 4 "c_int",
    x4C @ 0x4C: 4 "*mut HSD_GObj",
    x50 @ 0x50: 16 "StaticModelDesc",
]);

#[inline]
pub unsafe extern "C" fn HSD_GObjGetHSDObj(
    mut gobj: *mut HSD_GObj,
) -> *mut ::core::ffi::c_void {
    melee_core::ct!("HSD_GObjGetHSDObj");
    return (*gobj).hsd_obj;
}
pub const JOBJ_HIDDEN: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int) << 4 as ::core::ffi::c_int;
#[inline]
pub unsafe extern "C" fn HSD_JObjGetChild(
    mut jobj: *mut HSD_JObj,
) -> *mut HSD_JObj {
    melee_core::ct!("HSD_JObjGetChild");
    if jobj.is_null() {
        return ::core::ptr::null_mut::<HSD_JObj>();
    } else {
        return (*jobj).child as *mut HSD_JObj;
    };
}
#[inline]
pub unsafe extern "C" fn HSD_JObjGetNext(
    mut jobj: *mut HSD_JObj,
) -> *mut HSD_JObj {
    melee_core::ct!("HSD_JObjGetNext");
    if jobj.is_null() {
        return ::core::ptr::null_mut::<HSD_JObj>();
    } else {
        return (*jobj).next as *mut HSD_JObj;
    };
}
#[inline]
pub unsafe extern "C" fn Menu_GetAllInputs() -> u64_0 {
    melee_core::ct!("Menu_GetAllInputs");
    mn_804A04F0.buttons = mn_80229624(4 as u32_0) as u64_0;
    return mn_804A04F0.buttons;
}
#[inline]
pub unsafe extern "C" fn sfxMove() {
    melee_core::ct!("sfxMove");
    lbAudioAx_80024030(2 as ::core::ffi::c_int);
}
pub static mut mnInfoBonus_803EFCE8: [::core::ffi::c_int; 257] = [
    0 as ::core::ffi::c_int,
    0x1 as ::core::ffi::c_int,
    0x2 as ::core::ffi::c_int,
    0x3 as ::core::ffi::c_int,
    0x4 as ::core::ffi::c_int,
    0x5 as ::core::ffi::c_int,
    0x6 as ::core::ffi::c_int,
    0x7 as ::core::ffi::c_int,
    0x8 as ::core::ffi::c_int,
    0x9 as ::core::ffi::c_int,
    0xa as ::core::ffi::c_int,
    0xb as ::core::ffi::c_int,
    0xc as ::core::ffi::c_int,
    0xd as ::core::ffi::c_int,
    0xe as ::core::ffi::c_int,
    0xf as ::core::ffi::c_int,
    0x10 as ::core::ffi::c_int,
    0x11 as ::core::ffi::c_int,
    0x12 as ::core::ffi::c_int,
    0x13 as ::core::ffi::c_int,
    0x14 as ::core::ffi::c_int,
    0x15 as ::core::ffi::c_int,
    0x16 as ::core::ffi::c_int,
    0x17 as ::core::ffi::c_int,
    0x18 as ::core::ffi::c_int,
    0x19 as ::core::ffi::c_int,
    0x1a as ::core::ffi::c_int,
    0x1b as ::core::ffi::c_int,
    0x1c as ::core::ffi::c_int,
    0x1d as ::core::ffi::c_int,
    0x1e as ::core::ffi::c_int,
    0x1f as ::core::ffi::c_int,
    0x20 as ::core::ffi::c_int,
    0x21 as ::core::ffi::c_int,
    0x22 as ::core::ffi::c_int,
    0x23 as ::core::ffi::c_int,
    0x24 as ::core::ffi::c_int,
    0x25 as ::core::ffi::c_int,
    0x26 as ::core::ffi::c_int,
    0x27 as ::core::ffi::c_int,
    0x28 as ::core::ffi::c_int,
    0x29 as ::core::ffi::c_int,
    0x2a as ::core::ffi::c_int,
    0x2b as ::core::ffi::c_int,
    0x2c as ::core::ffi::c_int,
    0x2d as ::core::ffi::c_int,
    0x2e as ::core::ffi::c_int,
    0x2f as ::core::ffi::c_int,
    0x30 as ::core::ffi::c_int,
    0x31 as ::core::ffi::c_int,
    0x32 as ::core::ffi::c_int,
    0x33 as ::core::ffi::c_int,
    0x34 as ::core::ffi::c_int,
    0x35 as ::core::ffi::c_int,
    0x36 as ::core::ffi::c_int,
    0x37 as ::core::ffi::c_int,
    0x38 as ::core::ffi::c_int,
    0x39 as ::core::ffi::c_int,
    0x3a as ::core::ffi::c_int,
    0x3b as ::core::ffi::c_int,
    0x3c as ::core::ffi::c_int,
    0x3d as ::core::ffi::c_int,
    0x3e as ::core::ffi::c_int,
    0x3f as ::core::ffi::c_int,
    0x40 as ::core::ffi::c_int,
    0x41 as ::core::ffi::c_int,
    0x42 as ::core::ffi::c_int,
    0x43 as ::core::ffi::c_int,
    0x44 as ::core::ffi::c_int,
    0x45 as ::core::ffi::c_int,
    0x46 as ::core::ffi::c_int,
    0x47 as ::core::ffi::c_int,
    0x48 as ::core::ffi::c_int,
    0x49 as ::core::ffi::c_int,
    0x4a as ::core::ffi::c_int,
    0x4b as ::core::ffi::c_int,
    0x4c as ::core::ffi::c_int,
    0x4d as ::core::ffi::c_int,
    0xf9 as ::core::ffi::c_int,
    0x4e as ::core::ffi::c_int,
    0x4f as ::core::ffi::c_int,
    0x50 as ::core::ffi::c_int,
    0x51 as ::core::ffi::c_int,
    0x52 as ::core::ffi::c_int,
    0x53 as ::core::ffi::c_int,
    0x54 as ::core::ffi::c_int,
    0x55 as ::core::ffi::c_int,
    0x56 as ::core::ffi::c_int,
    0x57 as ::core::ffi::c_int,
    0x58 as ::core::ffi::c_int,
    0x59 as ::core::ffi::c_int,
    0x5a as ::core::ffi::c_int,
    0x5b as ::core::ffi::c_int,
    0x5c as ::core::ffi::c_int,
    0x5d as ::core::ffi::c_int,
    0x5e as ::core::ffi::c_int,
    0x5f as ::core::ffi::c_int,
    0x60 as ::core::ffi::c_int,
    0xfa as ::core::ffi::c_int,
    0x61 as ::core::ffi::c_int,
    0x62 as ::core::ffi::c_int,
    0x63 as ::core::ffi::c_int,
    0x64 as ::core::ffi::c_int,
    0x65 as ::core::ffi::c_int,
    0xfb as ::core::ffi::c_int,
    0x66 as ::core::ffi::c_int,
    0x67 as ::core::ffi::c_int,
    0x68 as ::core::ffi::c_int,
    0x69 as ::core::ffi::c_int,
    0x6a as ::core::ffi::c_int,
    0x6b as ::core::ffi::c_int,
    0x6c as ::core::ffi::c_int,
    0xfc as ::core::ffi::c_int,
    0x6d as ::core::ffi::c_int,
    0xfd as ::core::ffi::c_int,
    0x6e as ::core::ffi::c_int,
    0x6f as ::core::ffi::c_int,
    0x70 as ::core::ffi::c_int,
    0x71 as ::core::ffi::c_int,
    0x72 as ::core::ffi::c_int,
    0x73 as ::core::ffi::c_int,
    0x74 as ::core::ffi::c_int,
    0x75 as ::core::ffi::c_int,
    0x76 as ::core::ffi::c_int,
    0x77 as ::core::ffi::c_int,
    0x78 as ::core::ffi::c_int,
    0x79 as ::core::ffi::c_int,
    0x7a as ::core::ffi::c_int,
    0x7b as ::core::ffi::c_int,
    0x7c as ::core::ffi::c_int,
    0x7d as ::core::ffi::c_int,
    0x7e as ::core::ffi::c_int,
    0x7f as ::core::ffi::c_int,
    0x80 as ::core::ffi::c_int,
    0x81 as ::core::ffi::c_int,
    0x82 as ::core::ffi::c_int,
    0x83 as ::core::ffi::c_int,
    0x84 as ::core::ffi::c_int,
    0x85 as ::core::ffi::c_int,
    0x86 as ::core::ffi::c_int,
    0x87 as ::core::ffi::c_int,
    0x88 as ::core::ffi::c_int,
    0x89 as ::core::ffi::c_int,
    0x8a as ::core::ffi::c_int,
    0x8b as ::core::ffi::c_int,
    0x8c as ::core::ffi::c_int,
    0x8d as ::core::ffi::c_int,
    0x8e as ::core::ffi::c_int,
    0x8f as ::core::ffi::c_int,
    0x90 as ::core::ffi::c_int,
    0x91 as ::core::ffi::c_int,
    0x92 as ::core::ffi::c_int,
    0x93 as ::core::ffi::c_int,
    0x94 as ::core::ffi::c_int,
    0x95 as ::core::ffi::c_int,
    0x96 as ::core::ffi::c_int,
    0x97 as ::core::ffi::c_int,
    0x98 as ::core::ffi::c_int,
    0x99 as ::core::ffi::c_int,
    0x9a as ::core::ffi::c_int,
    0x9b as ::core::ffi::c_int,
    0x9c as ::core::ffi::c_int,
    0x9d as ::core::ffi::c_int,
    0xfe as ::core::ffi::c_int,
    0xff as ::core::ffi::c_int,
    0x9e as ::core::ffi::c_int,
    0x9f as ::core::ffi::c_int,
    0xa0 as ::core::ffi::c_int,
    0xa1 as ::core::ffi::c_int,
    0xa2 as ::core::ffi::c_int,
    0xa3 as ::core::ffi::c_int,
    0xa4 as ::core::ffi::c_int,
    0xa5 as ::core::ffi::c_int,
    0xa6 as ::core::ffi::c_int,
    0xa7 as ::core::ffi::c_int,
    0xa8 as ::core::ffi::c_int,
    0xa9 as ::core::ffi::c_int,
    0xaa as ::core::ffi::c_int,
    0xab as ::core::ffi::c_int,
    0xac as ::core::ffi::c_int,
    0xad as ::core::ffi::c_int,
    0xae as ::core::ffi::c_int,
    0xaf as ::core::ffi::c_int,
    0xb0 as ::core::ffi::c_int,
    0xb1 as ::core::ffi::c_int,
    0xb2 as ::core::ffi::c_int,
    0xb3 as ::core::ffi::c_int,
    0xb4 as ::core::ffi::c_int,
    0xb5 as ::core::ffi::c_int,
    0xb6 as ::core::ffi::c_int,
    0xb7 as ::core::ffi::c_int,
    0xb8 as ::core::ffi::c_int,
    0xb9 as ::core::ffi::c_int,
    0xba as ::core::ffi::c_int,
    0xbb as ::core::ffi::c_int,
    0xbc as ::core::ffi::c_int,
    0xbd as ::core::ffi::c_int,
    0xbe as ::core::ffi::c_int,
    0xbf as ::core::ffi::c_int,
    0xc0 as ::core::ffi::c_int,
    0xc1 as ::core::ffi::c_int,
    0xc2 as ::core::ffi::c_int,
    0xc3 as ::core::ffi::c_int,
    0xc4 as ::core::ffi::c_int,
    0xc5 as ::core::ffi::c_int,
    0xc6 as ::core::ffi::c_int,
    0xc7 as ::core::ffi::c_int,
    0xc8 as ::core::ffi::c_int,
    0xc9 as ::core::ffi::c_int,
    0xca as ::core::ffi::c_int,
    0xcb as ::core::ffi::c_int,
    0xcc as ::core::ffi::c_int,
    0xcd as ::core::ffi::c_int,
    0xce as ::core::ffi::c_int,
    0xcf as ::core::ffi::c_int,
    0xd0 as ::core::ffi::c_int,
    0xd1 as ::core::ffi::c_int,
    0xd2 as ::core::ffi::c_int,
    0xd3 as ::core::ffi::c_int,
    0xd4 as ::core::ffi::c_int,
    0xd5 as ::core::ffi::c_int,
    0xd6 as ::core::ffi::c_int,
    0xd7 as ::core::ffi::c_int,
    0xd8 as ::core::ffi::c_int,
    0xd9 as ::core::ffi::c_int,
    0xda as ::core::ffi::c_int,
    0xdb as ::core::ffi::c_int,
    0xdc as ::core::ffi::c_int,
    0xdd as ::core::ffi::c_int,
    0xde as ::core::ffi::c_int,
    0xdf as ::core::ffi::c_int,
    0xe0 as ::core::ffi::c_int,
    0xe1 as ::core::ffi::c_int,
    0xe2 as ::core::ffi::c_int,
    0xe3 as ::core::ffi::c_int,
    0xe4 as ::core::ffi::c_int,
    0xe5 as ::core::ffi::c_int,
    0xf6 as ::core::ffi::c_int,
    0xf7 as ::core::ffi::c_int,
    0xf8 as ::core::ffi::c_int,
    0xe6 as ::core::ffi::c_int,
    0xe7 as ::core::ffi::c_int,
    0xe8 as ::core::ffi::c_int,
    0xe9 as ::core::ffi::c_int,
    0xea as ::core::ffi::c_int,
    0xeb as ::core::ffi::c_int,
    0xec as ::core::ffi::c_int,
    0xed as ::core::ffi::c_int,
    0xee as ::core::ffi::c_int,
    0xef as ::core::ffi::c_int,
    0xf0 as ::core::ffi::c_int,
    0xf1 as ::core::ffi::c_int,
    0xf2 as ::core::ffi::c_int,
    0xf3 as ::core::ffi::c_int,
    0xf4 as ::core::ffi::c_int,
    0xf5 as ::core::ffi::c_int,
    0x100 as ::core::ffi::c_int,
];
pub static mut mnInfoBonus_804A09B0: mnInfoBonus_804A09B0_t =
    mnInfoBonus_804A09B0_t {
        x0: 0,
        x4: [::core::ptr::null::<HSD_Text>() as *mut HSD_Text; 5],
        x18: [::core::ptr::null::<HSD_Text>() as *mut HSD_Text; 5],
        x2C: [::core::ptr::null::<HSD_Text>() as *mut HSD_Text; 5],
        x40: ::core::ptr::null::<HSD_Text>() as *mut HSD_Text,
        x44: 0,
        x48: 0,
        x4C: ::core::ptr::null::<HSD_GObj>() as *mut HSD_GObj,
        x50: StaticModelDesc {
            joint: ::core::ptr::null::<HSD_Joint>() as *mut HSD_Joint,
            animjoint: ::core::ptr::null::<HSD_AnimJoint>()
                as *mut HSD_AnimJoint,
            matanim_joint: ::core::ptr::null::<HSD_MatAnimJoint>()
                as *mut HSD_MatAnimJoint,
            shapeanim_joint: ::core::ptr::null::<HSD_ShapeAnimJoint>()
                as *mut HSD_ShapeAnimJoint,
        },
    };
pub static mut mnInfoBonus_804D6C80: [u8_0; 8] = [0; 8];
pub const HSD_PAD_R: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int) << 5 as ::core::ffi::c_int;
pub const HSD_PAD_L: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int) << 6 as ::core::ffi::c_int;
pub const HSD_PAD_A: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int;
#[inline]
pub unsafe extern "C" fn mnInfoBonus_802528F8_inline(
    mut j: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    melee_core::ct!("mnInfoBonus_802528F8_inline");
    if *(&raw mut mnInfoBonus_804D6C80 as *mut u8_0) != 0 {
        return true_0;
    }
    return gm_8016F120(j);
}
#[inline]
pub unsafe extern "C" fn textSize(mut text: *mut HSD_Text) {
    melee_core::ct!("textSize");
    (*text).font_size.x = 0.0521f32 as f32_0;
    (*text).font_size.y = 0.0521f32 as f32_0;
}
pub unsafe extern "C" fn textSetup(mut o: *mut mnInfoBonus_804A09B0_t) {
    melee_core::ct!("textSetup");
    (*o).x40 = HSD_SisLib_803A5ACC(
        0 as ::core::ffi::c_int,
        1 as s32,
        -9.5f32,
        9.1f32,
        17.0f32,
        364.68332f32,
        38.38772f32,
    );
    textSize((*o).x40);
    HSD_SisLib_803A6368((*o).x40, 0xa5 as s32);
}
pub unsafe extern "C" fn mnInfoBonus_PcKind(
    mut i: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    melee_core::ct!("mnInfoBonus_PcKind");
    return if i >= 0 as ::core::ffi::c_int && i < 0x100 as ::core::ffi::c_int {
        mnInfoBonus_803EFCE8[i as usize]
    } else {
        0x100 as ::core::ffi::c_int
    };
}
pub unsafe extern "C" fn mnInfoBonus_802528F8() -> ::core::ffi::c_int {
    melee_core::ct!("mnInfoBonus_802528F8");
    let mut unused: ::core::ffi::c_int = 0;
    let mut var_r30: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut var_r29: ::core::ffi::c_int = mnInfoBonus_804A09B0.x0;
    let mut i: ::core::ffi::c_int = 0;
    i = 0 as ::core::ffi::c_int;
    loop {
        let mut j: ::core::ffi::c_int = mnInfoBonus_803EFCE8[i as usize];
        if j == 0x100 as ::core::ffi::c_int {
            break;
        }
        if var_r29 != 0 as ::core::ffi::c_int {
            if mnInfoBonus_802528F8_inline(j) != 0 {
                var_r29 -= 1;
            }
        } else if mnInfoBonus_802528F8_inline(j) != 0 {
            var_r30 += 1;
        }
        i += 1;
    }
    return var_r30;
}
#[inline]
pub unsafe extern "C" fn mnInfoBonus_802528F8_wrapper() -> ::core::ffi::c_int {
    melee_core::ct!("mnInfoBonus_802528F8_wrapper");
    return mnInfoBonus_802528F8();
}
#[inline]
pub unsafe extern "C" fn mnInfoBonus_802529B4_inline0(
    mut i: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    melee_core::ct!("mnInfoBonus_802529B4_inline0");
    if *(&raw mut mnInfoBonus_804D6C80 as *mut u8_0) as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        return 1 as ::core::ffi::c_int;
    }
    return gm_8016F120(i);
}
#[inline]
pub unsafe extern "C" fn mnInfoBonus_802529B4_inline1(
    mut i: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    melee_core::ct!("mnInfoBonus_802529B4_inline1");
    return (gm_8016F208(i) as u16_0 as ::core::ffi::c_int
        - 2 as ::core::ffi::c_int)
        * 3 as ::core::ffi::c_int;
}
pub unsafe extern "C" fn mnInfoBonus_802529B4() {
    melee_core::ct!("mnInfoBonus_802529B4");
    let mut o: *mut mnInfoBonus_804A09B0_t = &raw mut mnInfoBonus_804A09B0;
    let mut text_idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut bonus_offset: ::core::ffi::c_int = (*o).x0;
    let mut var_r28: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
    let mut i: ::core::ffi::c_int = 0;
    i = 0 as ::core::ffi::c_int;
    loop {
        let mut temp_1: ::core::ffi::c_int = mnInfoBonus_803EFCE8[i as usize];
        if temp_1 == 0x100 as ::core::ffi::c_int {
            break;
        }
        if bonus_offset != 0 as ::core::ffi::c_int {
            if mnInfoBonus_802529B4_inline0(temp_1) != 0 as ::core::ffi::c_int
            {
                bonus_offset -= 1;
            }
        } else if var_r28 != 0 as ::core::ffi::c_int {
            if mnInfoBonus_802529B4_inline0(temp_1) != 0 as ::core::ffi::c_int
            {
                HSD_SisLib_803A6368(
                    (*o).x4[text_idx as usize],
                    mnInfoBonus_802529B4_inline1(temp_1) as s32 + 0x1ba as s32,
                );
                HSD_SisLib_803A6368(
                    (*o).x18[text_idx as usize],
                    mnInfoBonus_802529B4_inline1(temp_1) as s32 + 0x1bb as s32,
                );
                HSD_SisLib_803A6368(
                    (*o).x2C[text_idx as usize],
                    mnInfoBonus_802529B4_inline1(temp_1) as s32 + 0x1bc as s32,
                );
                text_idx += 1;
                var_r28 -= 1;
            }
        }
        i += 1;
    }
}
#[inline]
pub unsafe extern "C" fn mnInfoBonus_80252ADC_inline(mut text: *mut HSD_Text) {
    melee_core::ct!("mnInfoBonus_80252ADC_inline");
    (*text).x34.x = 0.034f32 as f32_0;
    (*text).x34.y = 0.034f32 as f32_0;
}
pub unsafe extern "C" fn mnInfoBonus_80252ADC() {
    melee_core::ct!("mnInfoBonus_80252ADC");
    let mut o: *mut mnInfoBonus_804A09B0_t = &raw mut mnInfoBonus_804A09B0;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut pad_0: u64_0 = 0 as u64_0;
    while i < 5 as ::core::ffi::c_int {
        (*o).x4[i as usize] = HSD_SisLib_803A5ACC(
            0 as ::core::ffi::c_int,
            1 as s32,
            -14.0f32,
            melee_core::fp::fmadds(2.77f32, i as f32_0, -6.35f32),
            17.0f32,
            8.9f32,
            1.25f32,
        );
        mnInfoBonus_80252ADC_inline((*o).x4[i as usize]);
        (*(*o).x4[i as usize]).default_alignment = 2 as u8_0;
        (*o).x2C[i as usize] = HSD_SisLib_803A5ACC(
            0 as ::core::ffi::c_int,
            1 as s32,
            -14.0f32,
            melee_core::fp::fmadds(2.77f32, i as f32_0, -5.0f32),
            17.0f32,
            8.9f32,
            1.25f32,
        );
        mnInfoBonus_80252ADC_inline((*o).x2C[i as usize]);
        (*(*o).x2C[i as usize]).default_alignment = 2 as u8_0;
        (*o).x18[i as usize] = HSD_SisLib_803A5ACC(
            0 as ::core::ffi::c_int,
            1 as s32,
            -4.7500005f32,
            melee_core::fp::fmadds(2.77f32, i as f32_0, -6.25f32),
            17.0f32,
            18.5f32,
            2.5f32,
        );
        mnInfoBonus_80252ADC_inline((*o).x18[i as usize]);
        if i == 4 as ::core::ffi::c_int {
            mnInfoBonus_802529B4();
        }
        i += 1;
    }
}
pub unsafe extern "C" fn fn_80252C50(mut gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_80252C50");
    let mut o: *mut mnInfoBonus_804A09B0_t = &raw mut mnInfoBonus_804A09B0;
    let mut i: ::core::ffi::c_int = 0;
    let mut temp_r3: u64_0 = 0;
    let mut pad_0: u32_0 = 0;
    if mn_804D6BC8.cooldown as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        mn_804D6BC8.cooldown = mn_804D6BC8.cooldown.wrapping_sub(1);
        mn_804D6BC8.x2 = 0 as u16_0;
        mn_804D6BC8.x4 = 0 as ::core::ffi::c_int as s32;
        return;
    }
    if (*o).x44 != 0 as ::core::ffi::c_int {
        if (*o).x44 == 1 as ::core::ffi::c_int {
            mnInfoBonus_80252ADC();
        }
        (*o).x44 -= 1;
        return;
    }
    i = 0 as ::core::ffi::c_int;
    temp_r3 = Menu_GetAllInputs();
    if temp_r3 & 0x20 as u64_0 != 0 as u64_0 {
        lbAudioAx_80024030(i);
        mn_804A04F0.entering_menu = i as u8_0;
        mn_80229894(0x1c as s32, 1 as u16_0, 3 as s32);
        i = 0 as ::core::ffi::c_int;
        while i < 5 as ::core::ffi::c_int {
            HSD_SisLib_803A5CC4((*o).x4[i as usize]);
            HSD_SisLib_803A5CC4((*o).x2C[i as usize]);
            HSD_SisLib_803A5CC4((*o).x18[i as usize]);
            i += 1;
        }
        HSD_SisLib_803A5CC4((*o).x40);
        HSD_GObjFree((*o).x4C);
        (*o).x4C = ::core::ptr::null_mut::<HSD_GObj>();
        return;
    }
    if DbLevel as ::core::ffi::c_int >= DbLKind_DebugRom as ::core::ffi::c_int
        && (*(&raw mut HSD_PadCopyStatus as *mut HSD_PadStatus)).button
            & HSD_PAD_L as u32_0
            != 0
        && (*(&raw mut HSD_PadCopyStatus as *mut HSD_PadStatus)).button
            & HSD_PAD_R as u32_0
            != 0
        && (*(&raw mut HSD_PadCopyStatus as *mut HSD_PadStatus)).button
            & HSD_PAD_A as u32_0
            != 0
    {
        (*o).x0 = 0 as ::core::ffi::c_int;
        *(&raw mut mnInfoBonus_804D6C80 as *mut u8_0) = 1 as u8_0;
        mnInfoBonus_802529B4();
    }
    if temp_r3 & 1 as u64_0 != 0 as u64_0 {
        if (*o).x0 > 0 as ::core::ffi::c_int {
            sfxMove();
            (*o).x0 -= 1;
            mnInfoBonus_802529B4();
        }
    } else if temp_r3 & 2 as u64_0 != 0 as u64_0
        && mnInfoBonus_802528F8_wrapper() > 5 as ::core::ffi::c_int
    {
        sfxMove();
        (*o).x0 += 1;
        mnInfoBonus_802529B4();
    }
}
pub unsafe extern "C" fn fn_80252E4C(mut arg0: *mut HSD_GObj) {
    melee_core::ct!("fn_80252E4C");
    let mut temp_r30: *mut HSD_JObj = HSD_GObjGetHSDObj(arg0) as *mut HSD_JObj;
    let mut o: *mut mnInfoBonus_804A09B0_t = &raw mut mnInfoBonus_804A09B0;
    if (*o).x48 < 0xc8 as ::core::ffi::c_int {
        (*o).x48 += 1;
    } else {
        (*o).x48 = 0 as ::core::ffi::c_int;
    }
    if (*o).x0 > 0 as ::core::ffi::c_int {
        HSD_JObjClearFlags(
            HSD_JObjGetNext(HSD_JObjGetChild(temp_r30)),
            JOBJ_HIDDEN as u32_0,
        );
    } else {
        HSD_JObjSetFlags(
            HSD_JObjGetNext(HSD_JObjGetChild(temp_r30)),
            JOBJ_HIDDEN as u32_0,
        );
    }
    if mnInfoBonus_802528F8_wrapper() > 5 as ::core::ffi::c_int {
        HSD_JObjClearFlags(HSD_JObjGetChild(temp_r30), JOBJ_HIDDEN as u32_0);
    } else {
        HSD_JObjSetFlags(HSD_JObjGetChild(temp_r30), JOBJ_HIDDEN as u32_0);
    }
    HSD_JObjReqAnimAll(temp_r30, (*o).x48 as f32_0);
    HSD_JObjAnimAll(temp_r30);
}
#[inline]
pub unsafe extern "C" fn mnInfoBonus_inline_SetGObjFlag(
    mut gobjproc: *mut HSD_GObjProc,
) {
    melee_core::ct!("mnInfoBonus_inline_SetGObjFlag");
    (*gobjproc).set_flags_3(HSD_GObj_804D783C as u8_0 as u8_0);
}
#[inline]
pub unsafe extern "C" fn mnInfoBonus_80252F8C_inline0(
    mut o: *mut mnInfoBonus_804A09B0_t,
) {
    melee_core::ct!("mnInfoBonus_80252F8C_inline0");
    let mut gobj: *mut HSD_GObj = ::core::ptr::null_mut::<HSD_GObj>();
    let mut jobj: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut model_desc: *mut StaticModelDesc =
        ::core::ptr::null_mut::<StaticModelDesc>();
    model_desc = &raw mut (*o).x50;
    gobj = GObj_Create(6 as u16_0, 7 as u8_0, 0x80 as u8_0);
    (*o).x4C = gobj;
    jobj = HSD_JObjLoadJoint((*model_desc).joint as *mut HSD_Joint);
    HSD_GObjObject_80390A70(
        gobj,
        HSD_GObj_JObjKind,
        jobj as *mut ::core::ffi::c_void,
    );
    GObj_SetupGXLink(
        gobj,
        Some(
            HSD_GObj_JObjCallback
                as unsafe extern "C" fn(
                    *mut HSD_GObj,
                    ::core::ffi::c_int,
                ) -> (),
        ),
        4 as u8_0,
        0x80 as u32_0,
    );
    HSD_JObjAddAnimAll(
        jobj,
        (*model_desc).animjoint,
        (*model_desc).matanim_joint,
        (*model_desc).shapeanim_joint,
    );
    HSD_JObjReqAnimAll(jobj, 0.0f32);
    HSD_JObjAnimAll(jobj);
    HSD_JObjSetFlags(
        HSD_JObjGetNext(HSD_JObjGetChild(jobj)),
        JOBJ_HIDDEN as u32_0,
    );
    HSD_JObjSetFlags(HSD_JObjGetChild(jobj), JOBJ_HIDDEN as u32_0);
    mnInfoBonus_inline_SetGObjFlag(HSD_GObj_SetupProc(
        gobj,
        Some(fn_80252E4C as unsafe extern "C" fn(*mut HSD_GObj) -> ()),
        0 as u8_0,
    ));
}
pub unsafe extern "C" fn mnInfoBonus_80252F8C() {
    melee_core::ct!("mnInfoBonus_80252F8C");
    let mut o: *mut mnInfoBonus_804A09B0_t = &raw mut mnInfoBonus_804A09B0;
    let mut archive: *mut HSD_Archive = ::core::ptr::null_mut::<HSD_Archive>();
    let mut pad0: [u8_0; 8] = [0; 8];
    mn_804D6BC8.cooldown = 5 as u16_0;
    mn_804A04F0.prev_menu = mn_804A04F0.cur_menu;
    mn_804A04F0.cur_menu = 0x1f as MenuKind8;
    mn_804A04F0.hovered_selection = 0 as u16_0;
    memzero(
        o as *mut ::core::ffi::c_void,
        ::core::mem::size_of::<mnInfoBonus_804A09B0_t>() as ssize_t,
    );
    (*o).x44 = 8 as ::core::ffi::c_int;
    *(&raw mut mnInfoBonus_804D6C80 as *mut u8_0) = 0 as u8_0;
    archive = mn_804D6BB8;
    lbArchive_LoadSections(archive, &raw mut (*o).x50.joint as *mut *mut ::core::ffi::c_void, &[(b"MenMainConBo_Top_joint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut (*o).x50.animjoint) as usize, (b"MenMainConBo_Top_animjoint\0" as *const u8
            as *const ::core::ffi::c_char) as usize, (&raw mut (*o).x50.matanim_joint) as usize, (b"MenMainConBo_Top_matanim_joint\0" as *const u8
            as *const ::core::ffi::c_char) as usize, (&raw mut (*o).x50.shapeanim_joint) as usize, (b"MenMainConBo_Top_shapeanim_joint\0" as *const u8
            as *const ::core::ffi::c_char) as usize, (0 as ::core::ffi::c_int) as usize]);
    mnInfoBonus_inline_SetGObjFlag(HSD_GObj_SetupProc(
        GObj_Create(0 as u16_0, 1 as u8_0, 0x80 as u8_0),
        Some(fn_80252C50 as unsafe extern "C" fn(*mut HSD_GObj) -> ()),
        0 as u8_0,
    ));
    mnInfoBonus_80252F8C_inline0(o);
    textSetup(o);
}
pub const NULL: ::core::ffi::c_long = 0 as ::core::ffi::c_long;

/// Callees in unported TUs: settable hooks named like the C function; unset they panic with a TODO.
pub mod ext {
    #![allow(non_snake_case, non_upper_case_globals, dead_code, unused_imports)]
    use melee_types::*;
    use super::*;
    macro_rules! hook {
        ($h:ident, $name:ident, ($($a:ident: $t:ty),*) $(-> $r:ty)?, $port:path) => {
            pub static mut $h: Option<unsafe extern "C" fn($($t),*) $(-> $r)?> = None;
            pub unsafe extern "C" fn $name($($a: $t),*) $(-> $r)? {
                match $h {
                    Some(f) => f($($a),*),
                    None => $port($($a),*),
                }
            }
        };
        ($h:ident, $name:ident, ($($a:ident: $t:ty),*) $(-> $r:ty)?) => {
            pub static mut $h: Option<unsafe extern "C" fn($($t),*) $(-> $r)?> = None;
            pub unsafe extern "C" fn $name($($a: $t),*) $(-> $r)? {
                match $h {
                    Some(f) => f($($a),*),
                    None => panic!(concat!("TODO: ", stringify!($name), " is not ported")),
                }
            }
        };
    }
    hook!(H_GObj_Create, GObj_Create, (a0: u16_0, a1: u8_0, a2: u8_0) -> *mut HSD_GObj);
    pub static mut H_HSD_GObj_JObjCallback: Option<unsafe extern "C" fn(*mut HSD_GObj, ::core::ffi::c_int) > = None;
            pub unsafe extern "C" fn HSD_GObj_JObjCallback(a0: *mut HSD_GObj, a1: ::core::ffi::c_int)  {
                match H_HSD_GObj_JObjCallback {
                    Some(f) => f(a0, a1),
                    None => { unsafe { melee_runtime::tu::gobj::HSD_GObj_JObjCallback(a0 as _, a1 as _); } return; },
                }
            }
    pub static mut H_lbAudioAx_80024030: Option<unsafe extern "C" fn(::core::ffi::c_int) > = None;
            pub unsafe extern "C" fn lbAudioAx_80024030(a0: ::core::ffi::c_int)  {
                match H_lbAudioAx_80024030 {
                    Some(f) => f(a0),
                    None => { unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80024030(a0 as _); } return; },
                }
            }
    hook!(H_HSD_JObjReqAnimAll, HSD_JObjReqAnimAll, (a0: *mut HSD_JObj, a1: f32_0));
    pub static mut H_HSD_JObjLoadJoint: Option<unsafe extern "C" fn(*mut HSD_Joint) -> *mut HSD_JObj> = None;
            pub unsafe extern "C" fn HSD_JObjLoadJoint(a0: *mut HSD_Joint) -> *mut HSD_JObj {
                match H_HSD_JObjLoadJoint {
                    Some(f) => f(a0),
                    None => { return unsafe { melee_runtime::tu::jobj::HSD_JObjLoadJoint(a0 as _) as _ }; },
                }
            }
    hook!(H_HSD_JObjAddAnimAll, HSD_JObjAddAnimAll, (a0: *mut HSD_JObj, a1: *mut HSD_AnimJoint, a2: *mut HSD_MatAnimJoint, a3: *mut HSD_ShapeAnimJoint));
    hook!(H_HSD_JObjAnimAll, HSD_JObjAnimAll, (a0: *mut HSD_JObj));
    hook!(H_HSD_JObjSetFlags, HSD_JObjSetFlags, (a0: *mut HSD_JObj, a1: u32_0));
    hook!(H_HSD_JObjClearFlags, HSD_JObjClearFlags, (a0: *mut HSD_JObj, a1: u32_0));
    pub static mut H_HSD_SisLib_803A6368: Option<unsafe extern "C" fn(*mut HSD_Text, s32) > = None;
            pub unsafe extern "C" fn HSD_SisLib_803A6368(a0: *mut HSD_Text, a1: s32)  {
                match H_HSD_SisLib_803A6368 {
                    Some(f) => f(a0, a1),
                    None => { unsafe { melee_runtime::tu::sislib::HSD_SisLib_803A6368(a0 as _, a1 as _); } return; },
                }
            }
    pub static mut H_HSD_SisLib_803A5ACC: Option<unsafe extern "C" fn(::core::ffi::c_int, s32, f32_0, f32_0, f32_0, f32_0, f32_0) -> *mut HSD_Text> = None;
            pub unsafe extern "C" fn HSD_SisLib_803A5ACC(a0: ::core::ffi::c_int, a1: s32, a2: f32_0, a3: f32_0, a4: f32_0, a5: f32_0, a6: f32_0) -> *mut HSD_Text {
                match H_HSD_SisLib_803A5ACC {
                    Some(f) => f(a0, a1, a2, a3, a4, a5, a6),
                    None => { return unsafe { melee_runtime::tu::sislib::HSD_SisLib_803A5ACC(a0 as _, a1 as _, a2 as _, a3 as _, a4 as _, a5 as _, a6 as _) as _ }; },
                }
            }
    pub static mut H_HSD_SisLib_803A5CC4: Option<unsafe extern "C" fn(*mut HSD_Text) > = None;
            pub unsafe extern "C" fn HSD_SisLib_803A5CC4(a0: *mut HSD_Text)  {
                match H_HSD_SisLib_803A5CC4 {
                    Some(f) => f(a0),
                    None => { unsafe { melee_runtime::tu::sislib::HSD_SisLib_803A5CC4(a0 as _); } return; },
                }
            }
    pub static mut H_gm_8016F208: Option<unsafe extern "C" fn(::core::ffi::c_int) -> ::core::ffi::c_int> = None;
            pub unsafe extern "C" fn gm_8016F208(a0: ::core::ffi::c_int) -> ::core::ffi::c_int {
                match H_gm_8016F208 {
                    Some(f) => f(a0),
                    None => { return unsafe { crate::tu::gm_16f1::gm_8016F208(a0 as _) as _ }; },
                }
            }
    pub static mut H_gm_8016F120: Option<unsafe extern "C" fn(::core::ffi::c_int) -> ::core::ffi::c_int> = None;
            pub unsafe extern "C" fn gm_8016F120(a0: ::core::ffi::c_int) -> ::core::ffi::c_int {
                match H_gm_8016F120 {
                    Some(f) => f(a0),
                    None => { return unsafe { crate::tu::gmvs::gm_8016F120(a0 as _) as _ }; },
                }
            }
    pub static mut H_memzero: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void, ssize_t) > = None;
            pub unsafe extern "C" fn memzero(a0: *mut ::core::ffi::c_void, a1: ssize_t)  {
                match H_memzero {
                    Some(f) => f(a0, a1),
                    None => { unsafe { crate::tu::ext_hook::memzero(a0 as _, a1 as _); } return; },
                }
            }
    hook!(H_lbArchive_LoadSections, lbArchive_LoadSections, (a0: *mut HSD_Archive, a1: *mut *mut ::core::ffi::c_void, va: &[usize]));
    hook!(H_GObj_SetupGXLink, GObj_SetupGXLink, (a0: *mut HSD_GObj, a1: GObj_RenderFunc, a2: u8_0, a3: u32_0));
    hook!(H_HSD_GObjObject_80390A70, HSD_GObjObject_80390A70, (a0: *mut HSD_GObj, a1: u8_0, a2: *mut ::core::ffi::c_void));
    hook!(H_HSD_GObjFree, HSD_GObjFree, (a0: *mut HSD_GObj));
    hook!(H_HSD_GObj_SetupProc, HSD_GObj_SetupProc, (a0: *mut HSD_GObj, a1: HSD_GObjEvent, a2: u8_0) -> *mut HSD_GObjProc);
    pub static mut HSD_GObj_804D783C: s32 = unsafe { core::mem::zeroed() };
    pub static mut HSD_GObj_JObjKind: u8_0 = unsafe { core::mem::zeroed() };
    pub use melee_runtime::tu::controller::HSD_PadCopyStatus;
    pub use crate::tu::globals::DbLevel;
}
