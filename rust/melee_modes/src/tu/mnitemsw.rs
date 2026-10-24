//! src/melee/mn/mnitemsw.c: native port (c2rust-derived body; callees of unported TUs are `ext` TODO hooks).
#![allow(non_snake_case, non_upper_case_globals, non_camel_case_types, unused_unsafe, unused_mut, unused_assignments, unused_variables, unused_parens, dead_code, static_mut_refs, clippy::all)]

use melee_types::*;
#[allow(unused_imports)] use self::ext::*;
#[allow(unused_imports)] use super::mnmain::{MenMainConIs_Top, MenMainCursorIs_Top, mn_80229624, mn_80229860, mn_8022E978, mn_804A04F0, mn_804D6BB5, mn_804D6BC8};
#[allow(unused_imports)] use super::mnmainrule::{mn_8023164C};

pub type u8_0 = u8;
pub type u16_0 = u16;
pub type u32_0 = u32;
pub type u64_0 = u64;
pub type f32_0 = f32;
pub type f64_0 = f64;
pub const MenuInput_StartButton: _MenuInput = 256;
pub const MenuInput_AButton: _MenuInput = 512;
pub const MenuInput_Back: _MenuInput = 32;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub type bool_0 = ::core::ffi::c_int;
pub type _MenuInput = ::core::ffi::c_int;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct MnItemSwData {
    pub menu_kind: u8_0,
    pub cursor: u8_0,
    pub items: [u8_0; 31],
    pub x21: u8_0,
    pub pad: u8_0,
    pub x23: u8_0,
    pub jobjs: [*mut HSD_JObj; 9],
}
melee_core::gc_layout!(MnItemSwData, "MnItemSwData", size 0x48, align 4, [
    menu_kind @ 0x0: 1 "u8_0",
    cursor @ 0x1: 1 "u8_0",
    items @ 0x2: 31 "[u8_0; 31]",
    x21 @ 0x21: 1 "u8_0",
    pad @ 0x22: 1 "u8_0",
    x23 @ 0x23: 1 "u8_0",
    jobjs @ 0x24: 36 "[*mut HSD_JObj; 9]",
]);

#[derive(Copy, Clone)]
#[repr(C)]
pub struct MnItemSwTable {
    pub x00: [[f32_0; 3]; 4],
    pub x30: [f32_0; 18],
    pub items: [f32_0; 32],
    pub item_order: [u8_0; 32],
}
melee_core::gc_layout!(MnItemSwTable, "MnItemSwTable", size 0x118, align 4, [
    x00 @ 0x0: 48 "[[f32_0; 3]; 4]",
    x30 @ 0x30: 72 "[f32_0; 18]",
    items @ 0x78: 128 "[f32_0; 32]",
    item_order @ 0xF8: 32 "[u8_0; 32]",
]);

#[inline]
pub unsafe extern "C" fn HSD_JObjGetChild(mut jobj: *mut HSD_JObj) -> *mut HSD_JObj {
    if jobj.is_null() {
        return ::core::ptr::null_mut::<HSD_JObj>()
    } else {
        return (*jobj).child as *mut HSD_JObj
    };
}
#[inline]
pub unsafe extern "C" fn HSD_JObjGetNext(mut jobj: *mut HSD_JObj) -> *mut HSD_JObj {
    if jobj.is_null() {
        return ::core::ptr::null_mut::<HSD_JObj>()
    } else {
        return (*jobj).next as *mut HSD_JObj
    };
}
#[inline]
pub unsafe extern "C" fn HSD_JObjMtxIsDirty(mut jobj: *mut HSD_JObj) -> bool_0 {
    let mut result: bool_0 = 0;
    if !jobj.is_null() {} else {
        __assert(
            b"src/sysdolphin/baselib/jobj.h\0" as *const u8
                as *const ::core::ffi::c_char,
            228 as u32_0,
            b"jobj\0" as *const u8 as *const ::core::ffi::c_char,
        );
    };
    result = false_0 as bool_0;
    if (*jobj).flags & JOBJ_USER_DEF_MTX as u32_0 == 0
        && (*jobj).flags & JOBJ_MTX_DIRTY as u32_0 != 0
    {
        result = true_0 as bool_0;
    }
    return result;
}
#[inline]
pub unsafe extern "C" fn HSD_JObjSetTranslateX(mut jobj: *mut HSD_JObj, mut x: f32_0) {
    if !jobj.is_null() {} else {
        __assert(
            b"src/sysdolphin/baselib/jobj.h\0" as *const u8
                as *const ::core::ffi::c_char,
            513 as u32_0,
            b"jobj\0" as *const u8 as *const ::core::ffi::c_char,
        );
    };
    (*jobj).translate.x = x;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32_0 == 0 {
        if !jobj.is_null() && HSD_JObjMtxIsDirty(jobj) == 0 {
            HSD_JObjSetMtxDirtySub(jobj);
        }
    }
}
#[inline]
pub unsafe extern "C" fn HSD_JObjSetTranslateY(mut jobj: *mut HSD_JObj, mut y: f32_0) {
    if !jobj.is_null() {} else {
        __assert(
            b"src/sysdolphin/baselib/jobj.h\0" as *const u8
                as *const ::core::ffi::c_char,
            531 as u32_0,
            b"jobj\0" as *const u8 as *const ::core::ffi::c_char,
        );
    };
    (*jobj).translate.y = y;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32_0 == 0 {
        if !jobj.is_null() && HSD_JObjMtxIsDirty(jobj) == 0 {
            HSD_JObjSetMtxDirtySub(jobj);
        }
    }
}
#[inline]
pub unsafe extern "C" fn HSD_JObjGetTranslationX(mut jobj: *mut HSD_JObj) -> f32_0 {
    if !jobj.is_null() {} else {
        __assert(
            b"src/sysdolphin/baselib/jobj.h\0" as *const u8
                as *const ::core::ffi::c_char,
            582 as u32_0,
            b"jobj\0" as *const u8 as *const ::core::ffi::c_char,
        );
    };
    return (*jobj).translate.x;
}
#[inline]
pub unsafe extern "C" fn HSD_JObjGetTranslationY(mut jobj: *mut HSD_JObj) -> f32_0 {
    if !jobj.is_null() {} else {
        __assert(
            b"src/sysdolphin/baselib/jobj.h\0" as *const u8
                as *const ::core::ffi::c_char,
            588 as u32_0,
            b"jobj\0" as *const u8 as *const ::core::ffi::c_char,
        );
    };
    return (*jobj).translate.y;
}
#[inline]
pub unsafe extern "C" fn HSD_JObjAddTranslationY(
    mut jobj: *mut HSD_JObj,
    mut y: ::core::ffi::c_float,
) {
    if !jobj.is_null() {} else {
        __assert(
            b"src/sysdolphin/baselib/jobj.h\0" as *const u8
                as *const ::core::ffi::c_char,
            672 as u32_0,
            b"jobj\0" as *const u8 as *const ::core::ffi::c_char,
        );
    };
    (*jobj).translate.y += y;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32_0 == 0 {
        if !jobj.is_null() && HSD_JObjMtxIsDirty(jobj) == 0 {
            HSD_JObjSetMtxDirtySub(jobj);
        }
    }
}
pub const U32_TO_F32: ::core::ffi::c_double = 4503599627370496.0f64;
#[inline]
pub unsafe extern "C" fn sfxBack() {
    lbAudioAx_80024030(0 as ::core::ffi::c_int);
}
#[inline]
pub unsafe extern "C" fn sfxForward() {
    lbAudioAx_80024030(1 as ::core::ffi::c_int);
}
#[inline]
pub unsafe extern "C" fn sfxMove() {
    lbAudioAx_80024030(2 as ::core::ffi::c_int);
}
pub static mut mnItemSw_804D6BEC: u8_0 = 0;
pub static mut mnItemSw_804D6BE8: *mut HSD_GObj = ::core::ptr::null::<HSD_GObj>()
    as *mut HSD_GObj;
pub static mut mnItemSw_Table: MnItemSwTable = MnItemSwTable {
    x00: [
        [0.0f32, 9.0f32, -0.1f32],
        [0.0f32, 0.0f32, -0.1f32],
        [0.0f32, 0.0f32, -0.1f32],
        [0.0f32, 0.0f32, -0.1f32],
    ],
    x30: [
        0.0f32,
        10.0f32,
        -0.1f32,
        0.0f32,
        199.0f32,
        0.0f32,
        10.0f32,
        11.0f32,
        8.0f32,
        9.0f32,
        6.0f32,
        7.0f32,
        4.0f32,
        5.0f32,
        2.0f32,
        3.0f32,
        0.0f32,
        1.0f32,
    ],
    items: [
        0.0f32,
        1.0f32,
        2.0f32,
        3.0f32,
        4.0f32,
        5.0f32,
        6.0f32,
        7.0f32,
        8.0f32,
        9.0f32,
        10.0f32,
        11.0f32,
        12.0f32,
        13.0f32,
        14.0f32,
        15.0f32,
        16.0f32,
        17.0f32,
        18.0f32,
        19.0f32,
        20.0f32,
        21.0f32,
        22.0f32,
        23.0f32,
        25.0f32,
        24.0f32,
        26.0f32,
        27.0f32,
        29.0f32,
        28.0f32,
        30.0f32,
        31.0f32,
    ],
    item_order: [
        0x5 as ::core::ffi::c_int as u8_0,
        0x12 as ::core::ffi::c_int as u8_0,
        0xa as ::core::ffi::c_int as u8_0,
        0x1e as ::core::ffi::c_int as u8_0,
        0xd as ::core::ffi::c_int as u8_0,
        0x18 as ::core::ffi::c_int as u8_0,
        0x3 as ::core::ffi::c_int as u8_0,
        0xe as ::core::ffi::c_int as u8_0,
        0x17 as ::core::ffi::c_int as u8_0,
        0x1b as ::core::ffi::c_int as u8_0,
        0x1 as ::core::ffi::c_int as u8_0,
        0x9 as ::core::ffi::c_int as u8_0,
        0x8 as ::core::ffi::c_int as u8_0,
        0x7 as ::core::ffi::c_int as u8_0,
        0x15 as ::core::ffi::c_int as u8_0,
        0x4 as ::core::ffi::c_int as u8_0,
        0x6 as ::core::ffi::c_int as u8_0,
        0x2 as ::core::ffi::c_int as u8_0,
        0xf as ::core::ffi::c_int as u8_0,
        0 as ::core::ffi::c_int as u8_0,
        0x11 as ::core::ffi::c_int as u8_0,
        0xb as ::core::ffi::c_int as u8_0,
        0x1f as ::core::ffi::c_int as u8_0,
        0x1a as ::core::ffi::c_int as u8_0,
        0x14 as ::core::ffi::c_int as u8_0,
        0x19 as ::core::ffi::c_int as u8_0,
        0x10 as ::core::ffi::c_int as u8_0,
        0x16 as ::core::ffi::c_int as u8_0,
        0x13 as ::core::ffi::c_int as u8_0,
        0x1d as ::core::ffi::c_int as u8_0,
        0xc as ::core::ffi::c_int as u8_0,
        0 as ::core::ffi::c_int as u8_0,
    ],
};
pub static mut mnItemSw_804D4BA0: [f32_0; 2] = [0.0f32, 1.0f32];
#[inline]
pub unsafe extern "C" fn mnItemSw_GetTable() -> *mut MnItemSwTable {
    return &raw mut mnItemSw_Table;
}
pub unsafe extern "C" fn mnItemSw_80233A98(mut arg0: s32) -> s32 {
    match arg0 {
        19 => {
            if lbLang_IsSettingJP() != 0 {
                return mnItemSw_Table.items[arg0 as usize] as s32;
            }
            return 0x27 as s32;
        }
        17 => {
            if lbLang_IsSettingJP() != 0 {
                return mnItemSw_Table.items[arg0 as usize] as s32;
            }
            return 0x28 as s32;
        }
        18 | _ => return mnItemSw_Table.items[arg0 as usize] as s32,
    };
}
pub unsafe extern "C" fn mnItemSw_80233B68(
    mut arg0: *mut MnItemSwData,
    mut arg1: u32_0,
) {
    let mut temp_r0: s32 = 0;
    let mut var_r0: u8_0 = 0;
    let mut var_r0_2: u8_0 = 0;
    let mut hovered: *mut u16_0 = &raw mut mn_804A04F0.hovered_selection;
    let mut sel: u8_0 = *hovered as u8_0;
    if arg1 & 1 as u32_0 != 0 {
        match sel as s32 {
            0 => {
                *hovered = 0x1f as u16_0;
            }
            16 => {
                *hovered = 0x20 as u16_0;
            }
            31 => {
                *hovered = 0xf as u16_0;
            }
            32 => {
                *hovered = 0x1e as u16_0;
            }
            _ => {
                *hovered = (sel as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    as u16_0;
            }
        }
        if *hovered as u8_0 as ::core::ffi::c_int == 0x1f as ::core::ffi::c_int
            || *hovered as u8_0 as ::core::ffi::c_int == 0x20 as ::core::ffi::c_int
        {
            mn_804A04F0.confirmed_selection = (*arg0).x21;
            return;
        }
        mn_804A04F0.confirmed_selection = (*arg0).items[*hovered as u8_0 as usize];
        return;
    }
    if arg1 & 2 as u32_0 != 0 {
        match sel as s32 {
            15 => {
                *hovered = 0x1f as u16_0;
            }
            30 => {
                *hovered = 0x20 as u16_0;
            }
            31 => {
                *hovered = 0 as u16_0;
            }
            32 => {
                *hovered = 0x10 as u16_0;
            }
            _ => {
                *hovered = (sel as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    as u16_0;
            }
        }
        if *hovered as u8_0 as ::core::ffi::c_int == 0x1f as ::core::ffi::c_int
            || *hovered as u8_0 as ::core::ffi::c_int == 0x20 as ::core::ffi::c_int
        {
            mn_804A04F0.confirmed_selection = (*arg0).x21;
            return;
        }
        mn_804A04F0.confirmed_selection = (*arg0).items[*hovered as u8_0 as usize];
        return;
    }
    if arg1 & 4 as u32_0 != 0 {
        if (sel as ::core::ffi::c_int - 0x1f as ::core::ffi::c_int) as u8_0
            as ::core::ffi::c_uint <= 1 as ::core::ffi::c_uint
        {
            if mn_804A04F0.confirmed_selection as ::core::ffi::c_int
                == 5 as ::core::ffi::c_int
            {
                var_r0 = 0 as u8_0;
            } else {
                var_r0 = (mn_804A04F0.confirmed_selection as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int) as u8_0;
            }
            mn_804A04F0.confirmed_selection = var_r0;
            return;
        }
        if sel as ::core::ffi::c_uint >= 0x10 as ::core::ffi::c_uint
            && (sel as ::core::ffi::c_uint) < 0x1f as ::core::ffi::c_uint
        {
            *hovered = (sel as ::core::ffi::c_int - 0x10 as ::core::ffi::c_int) as u16_0;
            if *hovered as u8_0 as ::core::ffi::c_int == 0x1f as ::core::ffi::c_int
                || *hovered as u8_0 as ::core::ffi::c_int == 0x20 as ::core::ffi::c_int
            {
                mn_804A04F0.confirmed_selection = (*arg0).x21;
                return;
            }
            mn_804A04F0.confirmed_selection = (*arg0).items[*hovered as u8_0 as usize];
        }
    } else if arg1 & 8 as u32_0 != 0 {
        if (sel as ::core::ffi::c_int - 0x1f as ::core::ffi::c_int) as u8_0
            as ::core::ffi::c_uint <= 1 as ::core::ffi::c_uint
        {
            if mn_804A04F0.confirmed_selection as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
            {
                var_r0_2 = 5 as u8_0;
            } else {
                var_r0_2 = (mn_804A04F0.confirmed_selection as ::core::ffi::c_int
                    - 1 as ::core::ffi::c_int) as u8_0;
            }
            mn_804A04F0.confirmed_selection = var_r0_2;
            return;
        }
        if (sel as ::core::ffi::c_uint) < 0x10 as ::core::ffi::c_uint {
            temp_r0 = (sel as ::core::ffi::c_int + 0x10 as ::core::ffi::c_int) as s32;
            if temp_r0 >= 0x1f as ::core::ffi::c_int {
                *hovered = 0x1e as u16_0;
            } else {
                *hovered = temp_r0 as u16_0;
            }
            if *hovered as u8_0 as ::core::ffi::c_int == 0x1f as ::core::ffi::c_int
                || *hovered as u8_0 as ::core::ffi::c_int == 0x20 as ::core::ffi::c_int
            {
                mn_804A04F0.confirmed_selection = (*arg0).x21;
                return;
            }
            mn_804A04F0.confirmed_selection = (*arg0).items[*hovered as u8_0 as usize];
        }
    }
}
#[inline]
pub unsafe extern "C" fn mnItemSw_CommitItems(
    mut data: *mut MnItemSwData,
    mut i: s32,
    mut order: *mut u8_0,
) {
    while i < 0x1f as ::core::ffi::c_int {
        mn_8022E978(*order, (*data).items[i as usize]);
        i += 1;
        order = order.offset(1);
    }
    (*gmMainLib_GetGamePrefs()).item_freq = ((*data).x21 as ::core::ffi::c_int
        - 1 as ::core::ffi::c_int) as u8_0;
}
pub unsafe extern "C" fn fn_80233E10(mut gobj: *mut HSD_GObj) {
    let mut data: *mut MnItemSwData = ::core::ptr::null_mut::<MnItemSwData>();
    let mut order: *mut u8_0 = ::core::ptr::null_mut::<u8_0>();
    let mut buttons: u32_0 = 0;
    let mut i: s32 = 0;
    let mut _pad: [::core::ffi::c_uchar; 24] = [0; 24];
    data = (*mnItemSw_804D6BE8).user_data as *mut MnItemSwData;
    mn_804A04F0.buttons = mn_80229624(4 as u32_0) as u64_0;
    buttons = mn_804A04F0.buttons as u32_0;
    i = 0 as ::core::ffi::c_int as s32;
    if buttons & MenuInput_Back as ::core::ffi::c_int as u32_0 != 0 {
        sfxBack();
        mn_804A04F0.entering_menu = 0 as u8_0;
        data = (*mnItemSw_804D6BE8).user_data as *mut MnItemSwData;
        mnItemSw_CommitItems(
            data,
            i,
            &raw mut (*(mnItemSw_GetTable
                as unsafe extern "C" fn() -> *mut MnItemSwTable)())
                .item_order as *mut u8_0,
        );
        lbCardGame_SaveChanges();
        mn_804D6BC8.cooldown = 5 as u16_0;
        mn_8023164C();
        HSD_GObjFree(gobj);
        return;
    }
    if mnItemSw_804D6BEC as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        return;
    }
    if buttons & MenuInput_AButton as ::core::ffi::c_int as u32_0 != 0 {
        if (mn_804A04F0.hovered_selection as ::core::ffi::c_uint)
            < 0x1f as ::core::ffi::c_uint
        {
            sfxMove();
            mn_804A04F0.confirmed_selection = (if mn_804A04F0.confirmed_selection
                as ::core::ffi::c_uint > 0 as ::core::ffi::c_uint
            {
                i as u8_0 as ::core::ffi::c_int
            } else {
                1 as ::core::ffi::c_int
            }) as u8_0;
            let mut j: s32 = 0 as s32;
            let mut order2: *mut u8_0 = &raw mut (*(mnItemSw_GetTable
                as unsafe extern "C" fn() -> *mut MnItemSwTable)())
                .item_order as *mut u8_0;
            let mut data2: *mut MnItemSwData = (*mnItemSw_804D6BE8).user_data
                as *mut MnItemSwData;
            while j < 0x1f as ::core::ffi::c_int {
                mn_8022E978(*order2, (*data2).items[j as usize]);
                j += 1;
                order2 = order2.offset(1);
            }
            (*gmMainLib_GetGamePrefs()).item_freq = ((*data2).x21 as ::core::ffi::c_int
                - 1 as ::core::ffi::c_int) as u8_0;
            return;
        }
    } else if buttons & MenuInput_StartButton as ::core::ffi::c_int as u32_0 != 0 {
        sfxForward();
        match gm_GetCurrentGameMode() as ::core::ffi::c_int {
            1 => {
                let mut data2_0: *mut MnItemSwData = (*mnItemSw_804D6BE8).user_data
                    as *mut MnItemSwData;
                mnItemSw_CommitItems(
                    data2_0,
                    i,
                    &raw mut (*(mnItemSw_GetTable
                        as unsafe extern "C" fn() -> *mut MnItemSwTable)())
                        .item_order as *mut u8_0,
                );
                lbCardGame_SaveChanges();
                mn_80229860(GM_VS as ::core::ffi::c_int as s8);
                return;
            }
            _ => {
                let mut data2_1: *mut MnItemSwData = (*mnItemSw_804D6BE8).user_data
                    as *mut MnItemSwData;
                mnItemSw_CommitItems(
                    data2_1,
                    i,
                    &raw mut (*(mnItemSw_GetTable
                        as unsafe extern "C" fn() -> *mut MnItemSwTable)())
                        .item_order as *mut u8_0,
                );
                lbCardGame_SaveChanges();
                mn_8022F4CC();
                return;
            }
        }
    }
    if buttons & 0xf as u32_0 != 0 {
        let mut old_hovered: s32 = mn_804A04F0.hovered_selection as s32;
        let mut old_confirmed: s32 = mn_804A04F0.confirmed_selection as s32;
        mnItemSw_80233B68(data, buttons);
        if old_hovered != mn_804A04F0.hovered_selection as s32
            || old_confirmed != mn_804A04F0.confirmed_selection as s32
        {
            sfxMove();
        }
    }
}
pub unsafe extern "C" fn mnItemSw_8023405C(
    mut data: *mut MnItemSwData,
    mut idx: u8_0,
) -> *mut HSD_JObj {
    let mut cur: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut i: u8_0 = 0;
    if idx as ::core::ffi::c_int >= 16 as ::core::ffi::c_int {
        cur = HSD_JObjGetChild((*data).jobjs[6 as ::core::ffi::c_int as usize]);
        i = 16 as u8_0;
        while (i as ::core::ffi::c_int) < idx as ::core::ffi::c_int {
            cur = HSD_JObjGetNext(cur);
            i = i.wrapping_add(1);
        }
    } else {
        if (*data).jobjs[4 as ::core::ffi::c_int as usize].is_null() {
            cur = ::core::ptr::null_mut::<HSD_JObj>();
        } else {
            cur = (*(*data).jobjs[4 as ::core::ffi::c_int as usize]).child
                as *mut HSD_JObj;
        }
        i = 0 as u8_0;
        while (i as ::core::ffi::c_int) < idx as ::core::ffi::c_int {
            cur = HSD_JObjGetNext(cur);
            i = i.wrapping_add(1);
        }
    }
    return cur;
}
#[inline]
pub unsafe extern "C" fn mnItemSw_GetItemAnim(mut i: s32) -> s32 {
    return mnItemSw_80233A98(mnItemSw_Table.item_order[i as usize] as s32);
}
#[inline]
pub unsafe extern "C" fn mnItemSw_SetCursorPosition(mut data: *mut MnItemSwData) {
    let mut cursor: u8_0 = 0;
    let mut cjobj: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut y_spacing: f32_0 = 0.;
    cursor = (*data).cursor;
    cjobj = (*data).jobjs[2 as ::core::ffi::c_int as usize];
    if cursor as ::core::ffi::c_int == 0x1f as ::core::ffi::c_int
        || cursor as ::core::ffi::c_int == 0x20 as ::core::ffi::c_int
    {
        HSD_JObjSetFlagsAll(cjobj, JOBJ_HIDDEN as u32_0);
    } else {
        HSD_JObjClearFlagsAll(cjobj, JOBJ_HIDDEN as u32_0);
        y_spacing = HSD_JObjGetTranslationY(
            (*data).jobjs[5 as ::core::ffi::c_int as usize],
        ) - HSD_JObjGetTranslationY((*data).jobjs[4 as ::core::ffi::c_int as usize]);
        if (cursor as ::core::ffi::c_int) < 0x10 as ::core::ffi::c_int {
            HSD_JObjSetTranslateX(
                cjobj,
                HSD_JObjGetTranslationX((*data).jobjs[4 as ::core::ffi::c_int as usize]),
            );
            HSD_JObjSetTranslateY(
                cjobj,
                melee_core::fp::fmadds(y_spacing, cursor as f32_0, HSD_JObjGetTranslationY(
                        (*data).jobjs[4 as ::core::ffi::c_int as usize],
                    )),
            );
        } else {
            HSD_JObjSetTranslateX(
                cjobj,
                HSD_JObjGetTranslationX((*data).jobjs[6 as ::core::ffi::c_int as usize]),
            );
            HSD_JObjSetTranslateY(
                cjobj,
                melee_core::fp::fmadds(y_spacing, (cursor as ::core::ffi::c_int - 0x10 as ::core::ffi::c_int) as f32_0, HSD_JObjGetTranslationY(
                        (*data).jobjs[4 as ::core::ffi::c_int as usize],
                    )),
            );
        }
    }
    HSD_JObjClearFlagsAll(
        (*data).jobjs[3 as ::core::ffi::c_int as usize],
        JOBJ_HIDDEN as u32_0,
    );
}
pub unsafe extern "C" fn mnItemSw_80234104(mut gobj: *mut HSD_GObj) {
    let mut i: s32 = 0;
    let mut item_anim: s32 = 0;
    let mut item_jobj: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut tmp: *mut MnItemSwData = (*gobj).user_data as *mut MnItemSwData;
    let mut data: *mut MnItemSwData = tmp;
    -14.9527f32;
    -4.6369f32;
    -0.52099997f32;
    HSD_JObjClearFlagsAll(
        (*data).jobjs[4 as ::core::ffi::c_int as usize],
        JOBJ_HIDDEN as u32_0,
    );
    HSD_JObjClearFlagsAll(
        (*data).jobjs[6 as ::core::ffi::c_int as usize],
        JOBJ_HIDDEN as u32_0,
    );
    i = 0 as ::core::ffi::c_int as s32;
    while i < 0x1f as ::core::ffi::c_int {
        let mut jobj: *mut HSD_JObj = mnItemSw_8023405C(data, i as u8_0);
        if i != (*data).cursor as s32 {
            lb_80011E24(jobj, &raw mut item_jobj, &[(8 as ::core::ffi::c_int) as usize, (-(1 as ::core::ffi::c_int)) as usize]);
            HSD_JObjSetFlagsAll(item_jobj, JOBJ_HIDDEN as u32_0);
        }
        lb_80011E24(jobj, &raw mut item_jobj, &[(3 as ::core::ffi::c_int) as usize, (-(1 as ::core::ffi::c_int)) as usize]);
        item_anim = mnItemSw_GetItemAnim(i);
        HSD_JObjReqAnimAll(item_jobj, item_anim as f32_0);
        HSD_JObjAnimAll(item_jobj);
        HSD_JObjReqAnimAll(
            item_jobj,
            mnItemSw_Table.x30[0 as ::core::ffi::c_int as usize],
        );
        mn_8022F3D8(item_jobj, 1 as u8_0, TOBJ_MASK);
        HSD_JObjAnimAll(item_jobj);
        i += 1;
    }
    mnItemSw_SetCursorPosition(data);
}
#[inline]
pub unsafe extern "C" fn mnItemSw_ReqFreqAnim(
    mut jobj: *mut HSD_JObj,
    mut tbl: *mut MnItemSwTable,
    mut freq: u8_0,
    mut changed: u8_0,
) -> u8_0 {
    if mn_804A04F0.hovered_selection as u8_0 as ::core::ffi::c_int
        == 0x1f as ::core::ffi::c_int
        || mn_804A04F0.hovered_selection as u8_0 as ::core::ffi::c_int
            == 0x20 as ::core::ffi::c_int
    {
        HSD_JObjReqAnimAll(
            jobj,
            (*tbl)
                .x30[(7 as ::core::ffi::c_int
                + freq as ::core::ffi::c_int * 2 as ::core::ffi::c_int) as usize],
        );
    } else {
        HSD_JObjReqAnimAll(
            jobj,
            (*tbl)
                .x30[(6 as ::core::ffi::c_int
                + freq as ::core::ffi::c_int * 2 as ::core::ffi::c_int) as usize],
        );
    }
    HSD_JObjAnimAll(jobj);
    return changed;
}
#[inline]
pub unsafe extern "C" fn mnItemSw_UpdateConfirmed(
    mut user_data: *mut MnItemSwData,
    mut tbl: *mut MnItemSwTable,
    mut changed: u8_0,
) -> u8_0 {
    let mut item_jobj: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut animated_jobj: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    if mn_804A04F0.hovered_selection as ::core::ffi::c_int == 0x1f as ::core::ffi::c_int
        || mn_804A04F0.hovered_selection as ::core::ffi::c_int
            == 0x20 as ::core::ffi::c_int
    {
        changed = mnItemSw_ReqFreqAnim(
            (*user_data).jobjs[3 as ::core::ffi::c_int as usize],
            tbl,
            mn_804A04F0.confirmed_selection,
            changed,
        );
    } else {
        let mut confirmed_jobj: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
        let mut confirmed: u8_0 = mn_804A04F0.confirmed_selection;
        let mut jobj: *mut HSD_JObj = mnItemSw_8023405C(
            user_data,
            mn_804A04F0.hovered_selection as u8_0,
        );
        lb_80011E24(jobj, &raw mut confirmed_jobj, &[(2 as ::core::ffi::c_int) as usize, (-(1 as ::core::ffi::c_int)) as usize]);
        item_jobj = confirmed_jobj;
        HSD_JObjReqAnimAll(item_jobj, mnItemSw_804D4BA0[confirmed as usize]);
        animated_jobj = confirmed_jobj;
        HSD_JObjAnimAll(animated_jobj);
    }
    return changed;
}
pub unsafe extern "C" fn mnItemSw_8023453C(
    mut gobj: *mut HSD_GObj,
    mut arg1: u8_0,
    mut arg2: u8_0,
) {
    let mut hidden_jobj: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut frame_jobj: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut item_jobj: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut animated_jobj: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut lookup_data: *mut MnItemSwData = ::core::ptr::null_mut::<MnItemSwData>();
    let mut column_x: f32_0 = 0.;
    let mut sp44: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut cjobj: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut data: *mut MnItemSwData = (*gobj).user_data as *mut MnItemSwData;
    let mut cursor: u8_0 = 0;
    let mut tbl: *mut MnItemSwTable = mnItemSw_GetTable();
    let mut arg1_: u8_0 = arg1;
    let mut x: f32_0 = 0.;
    let mut arg2_: u8_0 = arg2;
    if arg1_ as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        let mut y_spacing: f32_0 = 0.;
        let mut anim_val: f32_0 = 0.;
        let mut old_cursor: u8_0 = (*data).cursor;
        if old_cursor as ::core::ffi::c_int == 0x1f as ::core::ffi::c_int
            || old_cursor as ::core::ffi::c_int == 0x20 as ::core::ffi::c_int
        {
            mnItemSw_ReqFreqAnim(
                (*data).jobjs[3 as ::core::ffi::c_int as usize],
                tbl,
                (*data).x21,
                arg1_,
            );
        } else {
            lookup_data = data;
            let mut jobj: *mut HSD_JObj = mnItemSw_8023405C(lookup_data, old_cursor);
            lb_80011E24(jobj, &raw mut sp44, &[(8 as ::core::ffi::c_int) as usize, (-(1 as ::core::ffi::c_int)) as usize]);
            hidden_jobj = sp44;
            HSD_JObjSetFlagsAll(hidden_jobj, JOBJ_HIDDEN as u32_0);
            frame_jobj = sp44;
            anim_val = mn_8022F298(frame_jobj);
            lb_80011E24(jobj, &raw mut sp44, &[(3 as ::core::ffi::c_int) as usize, (-(1 as ::core::ffi::c_int)) as usize]);
            item_jobj = sp44;
            HSD_JObjReqAnimAll(
                item_jobj,
                mnItemSw_80233A98((*tbl).item_order[old_cursor as usize] as s32) as f32_0,
            );
            animated_jobj = sp44;
            HSD_JObjAnimAll(animated_jobj);
            HSD_JObjReqAnimAll(sp44, (*tbl).x30[0 as ::core::ffi::c_int as usize]);
            mn_8022F3D8(sp44, 1 as u8_0, TOBJ_MASK);
            HSD_JObjAnimAll(sp44);
        }
        cursor = mn_804A04F0.hovered_selection as u8_0;
        if cursor as ::core::ffi::c_int == 0x1f as ::core::ffi::c_int
            || cursor as ::core::ffi::c_int == 0x20 as ::core::ffi::c_int
        {
            arg1_ = mnItemSw_ReqFreqAnim(
                (*data).jobjs[3 as ::core::ffi::c_int as usize],
                tbl,
                mn_804A04F0.confirmed_selection,
                arg1_,
            );
        } else {
            let mut jobj_0: *mut HSD_JObj = mnItemSw_8023405C(
                data,
                mn_804A04F0.hovered_selection as u8_0,
            );
            lb_80011E24(jobj_0, &raw mut sp44, &[(8 as ::core::ffi::c_int) as usize, (-(1 as ::core::ffi::c_int)) as usize]);
            HSD_JObjClearFlagsAll(sp44, JOBJ_HIDDEN as u32_0);
            HSD_JObjReqAnimAll(sp44, anim_val);
            HSD_JObjAnimAll(sp44);
            lb_80011E24(jobj_0, &raw mut sp44, &[(3 as ::core::ffi::c_int) as usize, (-(1 as ::core::ffi::c_int)) as usize]);
            HSD_JObjReqAnimAll(
                sp44,
                mnItemSw_80233A98((*tbl).item_order[cursor as usize] as s32) as f32_0,
            );
            HSD_JObjAnimAll(sp44);
            HSD_JObjReqAnimAll(sp44, (*tbl).x30[0 as ::core::ffi::c_int as usize]);
            mn_8022F3D8(sp44, 1 as u8_0, TOBJ_MASK);
            HSD_JObjAnimAll(sp44);
        }
        cjobj = (*data).jobjs[2 as ::core::ffi::c_int as usize];
        if (cursor as ::core::ffi::c_int - 0x1f as ::core::ffi::c_int) as u8_0
            as ::core::ffi::c_uint <= 1 as ::core::ffi::c_uint
        {
            HSD_JObjSetFlagsAll(cjobj, JOBJ_HIDDEN as u32_0);
        } else {
            HSD_JObjClearFlagsAll(cjobj, JOBJ_HIDDEN as u32_0);
            y_spacing = HSD_JObjGetTranslationY(
                (*data).jobjs[5 as ::core::ffi::c_int as usize],
            ) - HSD_JObjGetTranslationY((*data).jobjs[4 as ::core::ffi::c_int as usize]);
            if (cursor as ::core::ffi::c_int) < 0x10 as ::core::ffi::c_int {
                x = HSD_JObjGetTranslationX(
                    (*data).jobjs[4 as ::core::ffi::c_int as usize],
                );
                HSD_JObjSetTranslateX(cjobj, x);
                HSD_JObjSetTranslateY(
                    cjobj,
                    melee_core::fp::fmadds(y_spacing, cursor as f32_0, HSD_JObjGetTranslationY(
                            (*data).jobjs[4 as ::core::ffi::c_int as usize],
                        )),
                );
            } else {
                column_x = HSD_JObjGetTranslationX(
                    (*data).jobjs[6 as ::core::ffi::c_int as usize],
                );
                x = column_x;
                HSD_JObjSetTranslateX(cjobj, x);
                HSD_JObjSetTranslateY(
                    cjobj,
                    melee_core::fp::fmadds(y_spacing, (cursor as ::core::ffi::c_int - 0x10 as ::core::ffi::c_int) as f32_0, HSD_JObjGetTranslationY(
                            (*data).jobjs[4 as ::core::ffi::c_int as usize],
                        )),
                );
            }
        }
    }
    if arg2_ as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        arg1_ = mnItemSw_UpdateConfirmed(data, tbl, arg1_);
    }
    let mut sel: u16_0 = 0;
    if arg1 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        sel = mn_804A04F0.hovered_selection;
    } else {
        sel = (*data).cursor as u16_0;
    }
    if sel as u8_0 as ::core::ffi::c_int != 0x1f as ::core::ffi::c_int
        && sel as u8_0 as ::core::ffi::c_int != 0x20 as ::core::ffi::c_int
    {
        let mut jobj_1: *mut HSD_JObj = mnItemSw_8023405C(data, sel as u8_0);
        lb_80011E24(jobj_1, &raw mut sp44, &[(8 as ::core::ffi::c_int) as usize, (-(1 as ::core::ffi::c_int)) as usize]);
        mn_8022ED6C(
            sp44,
            (&raw mut (*tbl).x30 as *mut f32_0).offset(3 as ::core::ffi::c_int as isize)
                as *mut f32_0 as *mut AnimLoopSettings,
        );
        lb_80011E24(jobj_1, &raw mut sp44, &[(3 as ::core::ffi::c_int) as usize, (-(1 as ::core::ffi::c_int)) as usize]);
        mn_8022ED6C(
            sp44,
            (&raw mut (*tbl).x30 as *mut f32_0).offset(0 as ::core::ffi::c_int as isize)
                as *mut f32_0 as *mut AnimLoopSettings,
        );
    }
}
#[inline]
pub unsafe extern "C" fn mnItemSw_SaveSettings(mut gobj: *mut HSD_GObj) {
    let mut order: *mut u8_0 = &raw mut (*(mnItemSw_GetTable
        as unsafe extern "C" fn() -> *mut MnItemSwTable)())
        .item_order as *mut u8_0;
    let mut i: s32 = 0 as s32;
    let mut data: *mut MnItemSwData = (*gobj).user_data as *mut MnItemSwData;
    while i < 0x1f as ::core::ffi::c_int {
        mn_8022E978(*order, (*data).items[i as usize]);
        i += 1;
        order = order.offset(1);
    }
    (*gmMainLib_GetGamePrefs()).item_freq = ((*data).x21 as ::core::ffi::c_int
        - 1 as ::core::ffi::c_int) as u8_0;
}
pub unsafe extern "C" fn fn_80234C24(mut gobj: *mut HSD_GObj) {
    let mut data: *mut MnItemSwData = ::core::ptr::null_mut::<MnItemSwData>();
    let mut anims: *mut f32_0 = ::core::ptr::null_mut::<f32_0>();
    let mut jobj: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut text: *mut HSD_Text = ::core::ptr::null_mut::<HSD_Text>();
    let mut menu_changed: s32 = 0;
    let mut hovered_changed: s32 = 0;
    let mut confirmed_changed: s32 = 0;
    let mut _pad: [::core::ffi::c_uchar; 24] = [0; 24];
    menu_changed = 0 as ::core::ffi::c_int as s32;
    hovered_changed = 0 as ::core::ffi::c_int as s32;
    confirmed_changed = 0 as ::core::ffi::c_int as s32;
    data = (*gobj).user_data as *mut MnItemSwData;
    let mut state: u8_0 = (*data).x23;
    if (state as ::core::ffi::c_int == 0 as ::core::ffi::c_int
        || state as ::core::ffi::c_int == 1 as ::core::ffi::c_int
        || state as ::core::ffi::c_int == 3 as ::core::ffi::c_int)
        && (*data).menu_kind as ::core::ffi::c_int
            != mn_804A04F0.cur_menu as ::core::ffi::c_int
    {
        if mn_804A04F0.entering_menu as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
            (*data).x23 = 4 as u8_0;
        } else if mn_804A04F0.buttons & 0x20 as u64_0 != 0 as u64_0 {
            (*data).x23 = 2 as u8_0;
        }
        state = (*data).x23;
        jobj = (*data).jobjs[1 as ::core::ffi::c_int as usize];
        match state as s32 {
            1 => {
                anims = &raw mut *(&raw mut mnItemSw_Table.x00 as *mut [f32_0; 3])
                    .offset(0 as ::core::ffi::c_int as isize) as *mut f32_0;
            }
            2 => {
                anims = &raw mut *(&raw mut mnItemSw_Table.x00 as *mut [f32_0; 3])
                    .offset(2 as ::core::ffi::c_int as isize) as *mut f32_0;
            }
            3 => {
                anims = &raw mut *(&raw mut mnItemSw_Table.x00 as *mut [f32_0; 3])
                    .offset(1 as ::core::ffi::c_int as isize) as *mut f32_0;
            }
            4 => {
                anims = &raw mut *(&raw mut mnItemSw_Table.x00 as *mut [f32_0; 3])
                    .offset(3 as ::core::ffi::c_int as isize) as *mut f32_0;
            }
            _ => {}
        }
        HSD_JObjReqAnim(jobj, *anims.offset(0 as ::core::ffi::c_int as isize));
        HSD_JObjAnim(jobj);
        state = (*data).x23;
        if state as ::core::ffi::c_int == 0 as ::core::ffi::c_int
            || state as ::core::ffi::c_int == 1 as ::core::ffi::c_int
            || state as ::core::ffi::c_int == 3 as ::core::ffi::c_int
        {
            menu_changed = 1 as ::core::ffi::c_int as s32;
            hovered_changed = 1 as ::core::ffi::c_int as s32;
            confirmed_changed = 1 as ::core::ffi::c_int as s32;
        }
    }
    let mut state_0: u8_0 = (*data).x23;
    if state_0 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        jobj = (*data).jobjs[1 as ::core::ffi::c_int as usize];
        match state_0 as s32 {
            1 => {
                anims = &raw mut *(&raw mut mnItemSw_Table.x00 as *mut [f32_0; 3])
                    .offset(0 as ::core::ffi::c_int as isize) as *mut f32_0;
            }
            2 => {
                anims = &raw mut *(&raw mut mnItemSw_Table.x00 as *mut [f32_0; 3])
                    .offset(2 as ::core::ffi::c_int as isize) as *mut f32_0;
            }
            3 => {
                anims = &raw mut *(&raw mut mnItemSw_Table.x00 as *mut [f32_0; 3])
                    .offset(1 as ::core::ffi::c_int as isize) as *mut f32_0;
            }
            4 => {
                anims = &raw mut *(&raw mut mnItemSw_Table.x00 as *mut [f32_0; 3])
                    .offset(3 as ::core::ffi::c_int as isize) as *mut f32_0;
            }
            _ => {}
        }
        if mn_8022F298(jobj) >= *anims.offset(1 as ::core::ffi::c_int as isize) {
            state_0 = (*data).x23;
            match state_0 as s32 {
                1 | 3 => {
                    (*data).x23 = 0 as u8_0;
                    text = HSD_SisLib_803A5ACC(
                        0 as ::core::ffi::c_int,
                        mn_804D6BB5 as s32,
                        -14.9527f32,
                        -4.6369f32,
                        17.5f32,
                        200.0f32,
                        300.0f32,
                    );
                    (*data).jobjs[7 as ::core::ffi::c_int as usize] = text
                        as *mut HSD_JObj;
                    (*text).font_size.x = 0.0521f32 as f32_0;
                    (*text).font_size.y = 0.0521f32 as f32_0;
                    HSD_SisLib_803A6368(text, 2 as s32);
                    text = HSD_SisLib_803A5ACC(
                        0 as ::core::ffi::c_int,
                        mn_804D6BB5 as s32,
                        -0.52099997f32,
                        -4.6369f32,
                        17.5f32,
                        200.0f32,
                        300.0f32,
                    );
                    (*data).jobjs[8 as ::core::ffi::c_int as usize] = text
                        as *mut HSD_JObj;
                    (*text).font_size.x = 0.0521f32 as f32_0;
                    (*text).font_size.y = 0.0521f32 as f32_0;
                    if lbLang_IsSettingJP() != 0 as ::core::ffi::c_int
                        && lbLang_IsSavedLanguageUS() != 0 as ::core::ffi::c_int
                    {
                        HSD_SisLib_803A6368(text, 5 as s32);
                    } else {
                        HSD_SisLib_803A6368(text, 3 as s32);
                    }
                    mnItemSw_80234104(gobj);
                    mnItemSw_804D6BEC = 0 as u8_0;
                    return;
                }
                2 | 4 => {
                    HSD_GObjFree(gobj);
                    HSD_SisLib_803A5CC4(
                        (*data).jobjs[7 as ::core::ffi::c_int as usize] as *mut HSD_Text,
                    );
                    HSD_SisLib_803A5CC4(
                        (*data).jobjs[8 as ::core::ffi::c_int as usize] as *mut HSD_Text,
                    );
                    return;
                }
                _ => {}
            }
        }
        HSD_JObjAnim(jobj);
    }
    let mut state_1: u8_0 = (*data).x23;
    if state_1 as ::core::ffi::c_int == 0 as ::core::ffi::c_int
        || state_1 as ::core::ffi::c_int == 1 as ::core::ffi::c_int
        || state_1 as ::core::ffi::c_int == 3 as ::core::ffi::c_int
    {
        if (*data).cursor as s32 != mn_804A04F0.hovered_selection as s32 {
            hovered_changed = 1 as ::core::ffi::c_int as s32;
        }
        if (mn_804A04F0.hovered_selection as ::core::ffi::c_int
            - 0x1f as ::core::ffi::c_int) as u16_0 as ::core::ffi::c_uint
            <= 1 as ::core::ffi::c_uint
        {
            if (*data).x21 as ::core::ffi::c_int
                != mn_804A04F0.confirmed_selection as ::core::ffi::c_int
            {
                confirmed_changed = 1 as ::core::ffi::c_int as s32;
            }
        } else if (*data).items[mn_804A04F0.hovered_selection as usize]
            as ::core::ffi::c_int
            != mn_804A04F0.confirmed_selection as ::core::ffi::c_int
        {
            confirmed_changed = 1 as ::core::ffi::c_int as s32;
        }
    }
    mnItemSw_8023453C(gobj, hovered_changed as u8_0, confirmed_changed as u8_0);
    if menu_changed != 0 as ::core::ffi::c_int {
        (*data).menu_kind = mn_804A04F0.cur_menu;
    }
    if hovered_changed != 0 as ::core::ffi::c_int {
        (*data).cursor = mn_804A04F0.hovered_selection as u8_0;
    }
    if confirmed_changed != 0 as ::core::ffi::c_int {
        if mn_804A04F0.hovered_selection as ::core::ffi::c_int
            == 0x1f as ::core::ffi::c_int
            || mn_804A04F0.hovered_selection as ::core::ffi::c_int
                == 0x20 as ::core::ffi::c_int
        {
            (*data).x21 = mn_804A04F0.confirmed_selection;
            mnItemSw_SaveSettings(gobj);
            return;
        }
        (*data).items[mn_804A04F0.hovered_selection as usize] = mn_804A04F0
            .confirmed_selection;
    }
}
pub unsafe extern "C" fn mnItemSw_80235020(
    mut arg0: u8_0,
    mut arg1: *mut MnItemSwData,
) -> *mut HSD_JObj {
    let mut sp14: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut sp10: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut item_val: u8_0 = 0;
    let mut jobj: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut hovered: u8_0 = 0;
    let mut tbl: *mut MnItemSwTable = mnItemSw_GetTable();
    hovered = mn_804A04F0.hovered_selection as u8_0;
    jobj = HSD_JObjLoadJoint(MenMainCursorIs_Top.joint);
    HSD_JObjAddAnimAll(
        jobj,
        MenMainCursorIs_Top.animjoint,
        MenMainCursorIs_Top.matanim_joint,
        MenMainCursorIs_Top.shapeanim_joint,
    );
    lb_80011E24(jobj, &raw mut sp14, &[(3 as ::core::ffi::c_int) as usize, (-(1 as ::core::ffi::c_int)) as usize]);
    item_val = arg0;
    HSD_JObjReqAnimAll(
        sp14,
        mnItemSw_80233A98((*tbl).item_order[item_val as usize] as s32) as f32_0,
    );
    HSD_JObjAnimAll(sp14);
    if arg0 as ::core::ffi::c_int == hovered as ::core::ffi::c_int {
        HSD_JObjReqAnimAll(sp14, (*tbl).x30[1 as ::core::ffi::c_int as usize]);
    } else {
        HSD_JObjReqAnimAll(sp14, (*tbl).x30[0 as ::core::ffi::c_int as usize]);
    }
    mn_8022F3D8(sp14, 1 as u8_0, TOBJ_MASK);
    HSD_JObjAnimAll(sp14);
    item_val = (*arg1).items[arg0 as usize];
    lb_80011E24(jobj, &raw mut sp10, &[(2 as ::core::ffi::c_int) as usize, (-(1 as ::core::ffi::c_int)) as usize]);
    HSD_JObjReqAnimAll(sp10, mnItemSw_804D4BA0[item_val as usize]);
    HSD_JObjAnimAll(sp10);
    lb_80011E24(jobj, &raw mut sp14, &[(8 as ::core::ffi::c_int) as usize, (-(1 as ::core::ffi::c_int)) as usize]);
    if arg0 as ::core::ffi::c_int == hovered as ::core::ffi::c_int {
        HSD_JObjReqAnimAll(sp14, (*tbl).x30[3 as ::core::ffi::c_int as usize]);
        HSD_JObjAnimAll(sp14);
    } else {
        HSD_JObjSetFlagsAll(sp14, JOBJ_HIDDEN as u32_0);
    }
    return jobj;
}
#[inline]
pub unsafe extern "C" fn setInitialCursorPosition(
    mut user_data: *mut MnItemSwData,
    mut item_jobj: *mut HSD_JObj,
) {
    let mut cursor: u8_0 = 0;
    let mut y_spacing: f32_0 = 0.;
    let mut cjobj: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    HSD_JObjSetFlagsAll(
        (*user_data).jobjs[4 as ::core::ffi::c_int as usize],
        JOBJ_HIDDEN as u32_0,
    );
    HSD_JObjSetFlagsAll(
        (*user_data).jobjs[6 as ::core::ffi::c_int as usize],
        JOBJ_HIDDEN as u32_0,
    );
    cursor = (*user_data).cursor;
    cjobj = (*user_data).jobjs[2 as ::core::ffi::c_int as usize];
    if cursor as ::core::ffi::c_int == 0x1f as ::core::ffi::c_int
        || cursor as ::core::ffi::c_int == 0x20 as ::core::ffi::c_int
    {
        HSD_JObjSetFlagsAll(cjobj, JOBJ_HIDDEN as u32_0);
    } else {
        HSD_JObjClearFlagsAll(cjobj, JOBJ_HIDDEN as u32_0);
        y_spacing = HSD_JObjGetTranslationY(
            (*user_data).jobjs[5 as ::core::ffi::c_int as usize],
        )
            - HSD_JObjGetTranslationY(
                (*user_data).jobjs[4 as ::core::ffi::c_int as usize],
            );
        if (cursor as ::core::ffi::c_int) < 0x10 as ::core::ffi::c_int {
            HSD_JObjSetTranslateX(
                cjobj,
                HSD_JObjGetTranslationX(
                    (*user_data).jobjs[4 as ::core::ffi::c_int as usize],
                ),
            );
            HSD_JObjSetTranslateY(
                cjobj,
                melee_core::fp::fmadds(y_spacing, cursor as f32_0, HSD_JObjGetTranslationY(
                        (*user_data).jobjs[4 as ::core::ffi::c_int as usize],
                    )),
            );
        } else {
            HSD_JObjSetTranslateX(
                cjobj,
                HSD_JObjGetTranslationX(
                    (*user_data).jobjs[6 as ::core::ffi::c_int as usize],
                ),
            );
            HSD_JObjSetTranslateY(
                cjobj,
                melee_core::fp::fmadds(y_spacing, (cursor as ::core::ffi::c_int - 0x10 as ::core::ffi::c_int) as f32_0, HSD_JObjGetTranslationY(
                        (*user_data).jobjs[4 as ::core::ffi::c_int as usize],
                    )),
            );
        }
    }
    HSD_JObjSetFlagsAll(
        (*user_data).jobjs[2 as ::core::ffi::c_int as usize],
        JOBJ_HIDDEN as u32_0,
    );
}
#[inline]
pub unsafe extern "C" fn initUserData(
    mut user_data: *mut MnItemSwData,
    mut arg0: s32,
    mut tbl: *mut MnItemSwTable,
) {
    let mut order: *mut u8_0 = ::core::ptr::null_mut::<u8_0>();
    let mut i: s32 = 0;
    (*user_data).menu_kind = mn_804A04F0.cur_menu as u8_0;
    (*user_data).cursor = mn_804A04F0.hovered_selection as u8_0;
    order = &raw mut (*tbl).item_order as *mut u8_0;
    i = 0 as ::core::ffi::c_int as s32;
    while (i as u8_0 as ::core::ffi::c_int) < 0x1f as ::core::ffi::c_int {
        (*user_data).items[i as u8_0 as usize] = gm_8016403C(*order) as u8_0;
        order = order.offset(1);
        i += 1;
    }
    (*user_data).x21 = ((*gmMainLib_GetGamePrefs()).item_freq as ::core::ffi::c_int
        + 1 as ::core::ffi::c_int) as u8_0;
    (*user_data).x23 = arg0 as u8_0;
}
pub unsafe extern "C" fn mnItemSw_802351A0(mut arg0: s32) -> *mut HSD_GObj {
    let mut jobj: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut item_jobj: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut user_data: *mut MnItemSwData = ::core::ptr::null_mut::<MnItemSwData>();
    let mut mdl: *mut StaticModelDesc = &raw mut MenMainConIs_Top;
    let mut tbl: *mut MnItemSwTable = mnItemSw_GetTable();
    let mut y_spacing: f32_0 = 0.;
    let mut i: s32 = 0;
    let mut gobj: *mut HSD_GObj = GObj_Create(6 as u16_0, 7 as u8_0, 0x80 as u8_0);
    mnItemSw_804D6BE8 = gobj;
    jobj = HSD_JObjLoadJoint((*mdl).joint);
    HSD_GObjObject_80390A70(gobj, HSD_GObj_JObjKind, jobj as *mut ::core::ffi::c_void);
    GObj_SetupGXLink(
        gobj,
        Some(
            HSD_GObj_JObjCallback
                as unsafe extern "C" fn(*mut HSD_GObj, ::core::ffi::c_int) -> (),
        ),
        6 as u8_0,
        0x80 as u32_0,
    );
    HSD_GObj_SetupProc(
        gobj,
        Some(fn_80234C24 as unsafe extern "C" fn(*mut HSD_GObj) -> ()),
        0 as u8_0,
    );
    HSD_JObjAddAnimAll(
        jobj,
        (*mdl).animjoint,
        (*mdl).matanim_joint,
        (*mdl).shapeanim_joint,
    );
    HSD_JObjReqAnimAll(jobj, 0.0f32);
    HSD_JObjAnimAll(jobj);
    user_data = HSD_MemAlloc(::core::mem::size_of::<MnItemSwData>() as ssize_t)
        as *mut MnItemSwData;
    if !user_data.is_null() {} else {
        OSReport(b"Can't get user_data.\n\0" as *const u8 as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char, &[]);
        __assert(
            b"src/melee/mn/mnitemsw.c\0" as *const u8 as *const ::core::ffi::c_char,
            925 as u32_0,
            b"user_data\0" as *const u8 as *const ::core::ffi::c_char,
        );
    };
    GObj_InitUserData(
        gobj,
        0 as u8_0,
        Some(HSD_Free as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()),
        user_data as *mut ::core::ffi::c_void,
    );
    initUserData(user_data, arg0, tbl);
    i = 0 as ::core::ffi::c_int as s32;
    while i < 7 as ::core::ffi::c_int {
        lb_80011E24(jobj, (&raw mut (*user_data).jobjs as *mut *mut HSD_JObj).offset(i as isize)
                as *mut *mut HSD_JObj, &[(i) as usize, (-(1 as ::core::ffi::c_int)) as usize]);
        i += 1;
    }
    if mn_804A04F0.hovered_selection as ::core::ffi::c_int == 0x1f as ::core::ffi::c_int
        || mn_804A04F0.hovered_selection as ::core::ffi::c_int
            == 0x20 as ::core::ffi::c_int
    {
        mn_804A04F0.confirmed_selection = (*user_data).x21;
    } else {
        mn_804A04F0.confirmed_selection = (*user_data)
            .items[(*user_data).cursor as usize];
    }
    y_spacing = HSD_JObjGetTranslationY(
        (*user_data).jobjs[5 as ::core::ffi::c_int as usize],
    ) - HSD_JObjGetTranslationY((*user_data).jobjs[4 as ::core::ffi::c_int as usize]);
    i = 0 as ::core::ffi::c_int as s32;
    while i < 0x1f as ::core::ffi::c_int {
        item_jobj = mnItemSw_80235020(i as u8_0, user_data);
        if i < 0x10 as ::core::ffi::c_int {
            HSD_JObjAddChild(
                (*user_data).jobjs[4 as ::core::ffi::c_int as usize],
                item_jobj,
            );
            HSD_JObjAddTranslationY(
                item_jobj,
                y_spacing as ::core::ffi::c_float * i as ::core::ffi::c_float,
            );
        } else {
            HSD_JObjAddChild(
                (*user_data).jobjs[6 as ::core::ffi::c_int as usize],
                item_jobj,
            );
            HSD_JObjAddTranslationY(
                item_jobj,
                y_spacing as ::core::ffi::c_float
                    * (i as ::core::ffi::c_int - 0x10 as ::core::ffi::c_int)
                        as ::core::ffi::c_float,
            );
        }
        i += 1;
    }
    setInitialCursorPosition(user_data, item_jobj);
    let mut hov: u8_0 = mn_804A04F0.hovered_selection as u8_0;
    let mut x21: u8_0 = (*user_data).x21;
    item_jobj = (*user_data).jobjs[3 as ::core::ffi::c_int as usize];
    if hov as ::core::ffi::c_int == 0x1f as ::core::ffi::c_int
        || hov as ::core::ffi::c_int == 0x20 as ::core::ffi::c_int
    {
        HSD_JObjReqAnimAll(
            item_jobj,
            (*tbl)
                .x30[(7 as ::core::ffi::c_int
                + x21 as ::core::ffi::c_int * 2 as ::core::ffi::c_int) as usize],
        );
    } else {
        HSD_JObjReqAnimAll(
            item_jobj,
            (*tbl)
                .x30[(6 as ::core::ffi::c_int
                + x21 as ::core::ffi::c_int * 2 as ::core::ffi::c_int) as usize],
        );
    }
    HSD_JObjAnimAll(item_jobj);
    HSD_JObjSetFlagsAll(
        (*user_data).jobjs[3 as ::core::ffi::c_int as usize],
        JOBJ_HIDDEN as u32_0,
    );
    return gobj;
}
pub unsafe extern "C" fn mnItemSw_802358C0() {
    let mut proc_0: *mut HSD_GObjProc = ::core::ptr::null_mut::<HSD_GObjProc>();
    mn_804A04F0.prev_menu = mn_804A04F0.cur_menu;
    mn_804A04F0.cur_menu = 0x10 as MenuKind8;
    mn_804A04F0.hovered_selection = 0 as u16_0;
    mnItemSw_804D6BEC = 1 as u8_0;
    mnItemSw_802351A0(1 as s32);
    proc_0 = HSD_GObj_SetupProc(
        GObj_Create(0 as u16_0, 1 as u8_0, 0x80 as u8_0),
        Some(fn_80233E10 as unsafe extern "C" fn(*mut HSD_GObj) -> ()),
        0 as u8_0,
    );
    (*proc_0).set_flags_3(HSD_GObj_804D783C as u8_0 as u8_0);
}
pub const NULL: ::core::ffi::c_long = 0 as ::core::ffi::c_long;

/// Callees in unported TUs: settable hooks named like the C function; unset they panic with a TODO.
pub mod ext {
    #![allow(non_snake_case, non_upper_case_globals, dead_code, unused_imports)]
    use melee_types::*;
    use super::*;
    macro_rules! hook {
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
    hook!(H_OSReport, OSReport, (a0: *mut ::core::ffi::c_char, va: &[usize]));
    hook!(H___assert, __assert, (a0: *const ::core::ffi::c_char, a1: u32_0, a2: *const ::core::ffi::c_char) -> !);
    hook!(H_GObj_Create, GObj_Create, (a0: u16_0, a1: u8_0, a2: u8_0) -> *mut HSD_GObj);
    hook!(H_HSD_GObj_JObjCallback, HSD_GObj_JObjCallback, (a0: *mut HSD_GObj, a1: ::core::ffi::c_int));
    hook!(H_HSD_JObjReqAnimAll, HSD_JObjReqAnimAll, (a0: *mut HSD_JObj, a1: f32_0));
    hook!(H_HSD_JObjSetMtxDirtySub, HSD_JObjSetMtxDirtySub, (a0: *mut HSD_JObj));
    hook!(H_HSD_JObjLoadJoint, HSD_JObjLoadJoint, (a0: *mut HSD_Joint) -> *mut HSD_JObj);
    hook!(H_HSD_JObjAddAnimAll, HSD_JObjAddAnimAll, (a0: *mut HSD_JObj, a1: *mut HSD_AnimJoint, a2: *mut HSD_MatAnimJoint, a3: *mut HSD_ShapeAnimJoint));
    hook!(H_HSD_JObjAnimAll, HSD_JObjAnimAll, (a0: *mut HSD_JObj));
    hook!(H_HSD_JObjSetFlagsAll, HSD_JObjSetFlagsAll, (a0: *mut HSD_JObj, a1: u32_0));
    hook!(H_HSD_JObjClearFlagsAll, HSD_JObjClearFlagsAll, (a0: *mut HSD_JObj, a1: u32_0));
    hook!(H_HSD_JObjAddChild, HSD_JObjAddChild, (a0: *mut HSD_JObj, a1: *mut HSD_JObj));
    hook!(H_HSD_JObjReqAnim, HSD_JObjReqAnim, (a0: *mut HSD_JObj, a1: f32_0));
    hook!(H_HSD_JObjAnim, HSD_JObjAnim, (a0: *mut HSD_JObj));
    hook!(H_lbAudioAx_80024030, lbAudioAx_80024030, (a0: ::core::ffi::c_int));
    hook!(H_lb_80011E24, lb_80011E24, (a0: *mut HSD_JObj, a1: *mut *mut HSD_JObj, va: &[usize]) -> ::core::ffi::c_int);
    hook!(H_mn_8022ED6C, mn_8022ED6C, (a0: *mut HSD_JObj, a1: *mut AnimLoopSettings) -> ::core::ffi::c_float);
    hook!(H_mn_8022F298, mn_8022F298, (a0: *mut HSD_JObj) -> f32_0);
    hook!(H_mn_8022F3D8, mn_8022F3D8, (a0: *mut HSD_JObj, a1: u8_0, a2: HSD_TypeMask));
    hook!(H_mn_8022F4CC, mn_8022F4CC, ());
    hook!(H_HSD_SisLib_803A6368, HSD_SisLib_803A6368, (a0: *mut HSD_Text, a1: s32));
    hook!(H_HSD_SisLib_803A5ACC, HSD_SisLib_803A5ACC, (a0: ::core::ffi::c_int, a1: s32, a2: f32_0, a3: f32_0, a4: f32_0, a5: f32_0, a6: f32_0) -> *mut HSD_Text);
    hook!(H_HSD_SisLib_803A5CC4, HSD_SisLib_803A5CC4, (a0: *mut HSD_Text));
    hook!(H_gm_8016403C, gm_8016403C, (a0: u8_0) -> bool_0);
    hook!(H_gm_GetCurrentGameMode, gm_GetCurrentGameMode, () -> u8_0);
    hook!(H_gmMainLib_GetGamePrefs, gmMainLib_GetGamePrefs, () -> *mut GamePrefs);
    hook!(H_lbCardGame_SaveChanges, lbCardGame_SaveChanges, ());
    hook!(H_lbLang_IsSettingJP, lbLang_IsSettingJP, () -> bool_0);
    hook!(H_lbLang_IsSavedLanguageUS, lbLang_IsSavedLanguageUS, () -> bool_0);
    hook!(H_GObj_SetupGXLink, GObj_SetupGXLink, (a0: *mut HSD_GObj, a1: GObj_RenderFunc, a2: u8_0, a3: u32_0));
    hook!(H_HSD_GObjObject_80390A70, HSD_GObjObject_80390A70, (a0: *mut HSD_GObj, a1: u8_0, a2: *mut ::core::ffi::c_void));
    hook!(H_HSD_GObjFree, HSD_GObjFree, (a0: *mut HSD_GObj));
    hook!(H_HSD_GObj_SetupProc, HSD_GObj_SetupProc, (a0: *mut HSD_GObj, a1: HSD_GObjEvent, a2: u8_0) -> *mut HSD_GObjProc);
    hook!(H_GObj_InitUserData, GObj_InitUserData, (a0: *mut HSD_GObj, a1: u8_0, a2: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()>, a3: *mut ::core::ffi::c_void));
    hook!(H_HSD_Free, HSD_Free, (a0: *mut ::core::ffi::c_void));
    hook!(H_HSD_MemAlloc, HSD_MemAlloc, (a0: ssize_t) -> *mut ::core::ffi::c_void);
    pub static mut HSD_GObj_804D783C: s32 = unsafe { core::mem::zeroed() };
    pub static mut HSD_GObj_JObjKind: u8_0 = unsafe { core::mem::zeroed() };
}
