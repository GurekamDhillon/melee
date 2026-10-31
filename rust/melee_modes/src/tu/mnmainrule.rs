//! src/melee/mn/mnmainrule.c: native port (c2rust-derived body; callees of unported TUs are `ext` TODO hooks).
#![allow(non_snake_case, non_upper_case_globals, non_camel_case_types, unused_unsafe, unused_mut, unused_assignments, unused_variables, unused_parens, dead_code, static_mut_refs, clippy::all)]

use melee_types::*;
#[allow(unused_imports)] use self::ext::*;
#[allow(unused_imports)] use super::mnmain::{MenMainBack_Top, MenMainConIs_Top, MenMainConRl_Top, MenMainConSs_Top, MenMainCursorIs_Top, MenMainCursorRl01_Top, MenMainCursorRl02_Top, MenMainCursorRl03_Top, MenMainCursorRl04_Top, MenMainCursorRl05_Top, MenMainCursorRl_Top, MenMainCursorSs_Top, MenMainCursorTr01_Top, MenMainCursorTr02_Top, MenMainCursorTr03_Top, MenMainCursorTr04_Top, MenMainNmRl_Top, MenMainPanel_Top, MenMain_cam, MenMain_fog, MenMain_lights, mn_80229624, mn_80229860, mn_80229894, mn_80229B2C, mn_80229DC0, mn_8022BCF8, mn_8022BE34, mn_8022BEDC, mn_8022C304, mn_8022EBDC, mn_803EB6B0, mn_804A04F0, mn_804D6BC8};
#[allow(unused_imports)] use super::mnitemsw::{mnItemSw_802358C0};

pub type u8_0 = u8;
pub type u16_0 = u16;
pub type u32_0 = u32;
pub type u64_0 = u64;
pub type f32_0 = f32;
pub type f64_0 = f64;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub type bool_0 = ::core::ffi::c_int;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct mn_802307F8_t {
    pub x0: u8_0,
    pub x1: u8_0,
    pub x2: u8_0,
    pub x3: u8_0,
    pub x4: u8_0,
    pub x5: u8_0,
    pub x6: u8_0,
    pub x7_pad: [u8_0; 2],
    pub x9: u8_0,
    pub xA: u8_0,
    pub xB_pad: [u8_0; 1],
    pub c2rust_unnamed: C2RustUnnamed_1,
    pub text: *mut HSD_Text,
}
melee_core::gc_layout!(mn_802307F8_t, "mn_802307F8_t", size 0x134, align 4, [
    x0 @ 0x0: 1 "u8_0",
    x1 @ 0x1: 1 "u8_0",
    x2 @ 0x2: 1 "u8_0",
    x3 @ 0x3: 1 "u8_0",
    x4 @ 0x4: 1 "u8_0",
    x5 @ 0x5: 1 "u8_0",
    x6 @ 0x6: 1 "u8_0",
    x7_pad @ 0x7: 2 "[u8_0; 2]",
    x9 @ 0x9: 1 "u8_0",
    xA @ 0xA: 1 "u8_0",
    xB_pad @ 0xB: 1 "[u8_0; 1]",
    c2rust_unnamed @ 0xC: 292 "C2RustUnnamed_1",
    text @ 0x130: 4 "*mut HSD_Text",
]);

#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_1 {
    pub xC: [*mut HSD_JObj; 17],
    pub c2rust_unnamed: C2RustUnnamed_2,
}
melee_core::gc_layout!(C2RustUnnamed_1, "C2RustUnnamed_1", size 0x124, align 4, [
    xC @ 0x0: 68 "[*mut HSD_JObj; 17]",
    c2rust_unnamed @ 0x0: 292 "C2RustUnnamed_2",
]);

#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_2 {
    pub xC0: [*mut HSD_JObj; 10],
    pub x34: [C2RustUnnamed_3; 7],
}
melee_core::gc_layout!(C2RustUnnamed_2, "C2RustUnnamed_2", size 0x124, align 4, [
    xC0 @ 0x0: 40 "[*mut HSD_JObj; 10]",
    x34 @ 0x28: 252 "[C2RustUnnamed_3; 7]",
]);

#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_3 {
    pub joints: [*mut HSD_JObj; 9],
}
melee_core::gc_layout!(C2RustUnnamed_3, "C2RustUnnamed_3", size 0x24, align 4, [
    joints @ 0x0: 36 "[*mut HSD_JObj; 9]",
]);

#[derive(Copy, Clone)]
#[repr(C)]
pub struct mn_80231634_t {
    pub pad_0: [::core::ffi::c_char; 16],
    pub x10: ::core::ffi::c_int,
}
melee_core::gc_layout!(mn_80231634_t, "mn_80231634_t", size 0x14, align 4, [
    pad_0 @ 0x0: 16 "[c_char; 16]",
    x10 @ 0x10: 4 "c_int",
]);

#[derive(Copy, Clone)]
#[repr(C)]
pub union mn_802307F8_value_view {
    pub fields: mn_802307F8_t,
    pub indexed: C2RustUnnamed_4,
}
melee_core::gc_layout!(mn_802307F8_value_view, "mn_802307F8_value_view", size 0x134, align 4, [
    fields @ 0x0: 308 "mn_802307F8_t",
    indexed @ 0x0: 9 "C2RustUnnamed_4",
]);

#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_4 {
    pub x0: u8_0,
    pub x1: u8_0,
    pub values: [u8_0; 7],
}
melee_core::gc_layout!(C2RustUnnamed_4, "C2RustUnnamed_4", size 0x9, align 1, [
    x0 @ 0x0: 1 "u8_0",
    x1 @ 0x1: 1 "u8_0",
    values @ 0x2: 7 "[u8_0; 7]",
]);

#[derive(Copy, Clone)]
#[repr(C)]
pub struct JObjIndices5 {
    pub idx: [u8_0; 5],
}
melee_core::gc_layout!(JObjIndices5, "JObjIndices5", size 0x5, align 1, [
    idx @ 0x0: 5 "[u8_0; 5]",
]);

#[derive(Copy, Clone)]
#[repr(C)]
pub struct JObjIndices2 {
    pub idx: [u8_0; 2],
}
melee_core::gc_layout!(JObjIndices2, "JObjIndices2", size 0x2, align 1, [
    idx @ 0x0: 2 "[u8_0; 2]",
]);

#[derive(Copy, Clone)]
#[repr(C)]
pub struct mn_803EC818_t {
    pub x0: u8_0,
    pub pad_1: [u8_0; 4],
}
melee_core::gc_layout!(mn_803EC818_t, "mn_803EC818_t", size 0x5, align 1, [
    x0 @ 0x0: 1 "u8_0",
    pad_1 @ 0x1: 4 "[u8_0; 4]",
]);

#[derive(Copy, Clone)]
#[repr(C)]
pub struct JObjIndices6 {
    pub idx: [u8_0; 6],
}
melee_core::gc_layout!(JObjIndices6, "JObjIndices6", size 0x6, align 1, [
    idx @ 0x0: 6 "[u8_0; 6]",
]);

#[inline]
pub unsafe extern "C" fn HSD_GObjGetUserData(
    mut gobj: *mut HSD_GObj,
) -> *mut ::core::ffi::c_void {
    melee_core::ct!("HSD_GObjGetUserData");
    return (*gobj).user_data;
}
pub const JOBJ_HIDDEN: ::core::ffi::c_int = (1 as ::core::ffi::c_int)
    << 4 as ::core::ffi::c_int;
#[inline]
pub unsafe extern "C" fn HSD_JObjGetChild(mut jobj: *mut HSD_JObj) -> *mut HSD_JObj {
    melee_core::ct!("HSD_JObjGetChild");
    if jobj.is_null() {
        return ::core::ptr::null_mut::<HSD_JObj>()
    } else {
        return (*jobj).child as *mut HSD_JObj
    };
}
pub const U32_TO_F32: ::core::ffi::c_double = 4503599627370496.0f64;
#[inline]
pub unsafe extern "C" fn sfxBack() {
    melee_core::ct!("sfxBack");
    lbAudioAx_80024030(0 as ::core::ffi::c_int);
}
#[inline]
pub unsafe extern "C" fn sfxForward() {
    melee_core::ct!("sfxForward");
    lbAudioAx_80024030(1 as ::core::ffi::c_int);
}
#[inline]
pub unsafe extern "C" fn sfxMove() {
    melee_core::ct!("sfxMove");
    lbAudioAx_80024030(2 as ::core::ffi::c_int);
}
pub static mut mn_803EC600: [u16_0; 8] = [
    3 as ::core::ffi::c_int as u16_0,
    4 as ::core::ffi::c_int as u16_0,
    5 as ::core::ffi::c_int as u16_0,
    6 as ::core::ffi::c_int as u16_0,
    7 as ::core::ffi::c_int as u16_0,
    8 as ::core::ffi::c_int as u16_0,
    9 as ::core::ffi::c_int as u16_0,
    0 as ::core::ffi::c_int as u16_0,
];
pub static mut rule_label_frames: [[f32_0; 2]; 7] = [
    [0.0f32, 1.0f32],
    [2.0f32, 3.0f32],
    [6.0f32, 7.0f32],
    [8.0f32, 9.0f32],
    [10.0f32, 11.0f32],
    [12.0f32, 13.0f32],
    [14.0f32, 15.0f32],
];
pub static mut highlight_anim: AnimLoopSettings = AnimLoopSettings {
    start_frame: 0.0f32,
    end_frame: 99.0f32,
    loop_frame: 0.0f32,
};
pub static mut value_arrow_anim: AnimLoopSettings = AnimLoopSettings {
    start_frame: 0.0f32,
    end_frame: 199.0f32,
    loop_frame: 0.0f32,
};
pub static mut option_anims: [AnimLoopSettings; 2] = [
    AnimLoopSettings {
        start_frame: 0.0f32,
        end_frame: 24.0f32,
        loop_frame: -0.1f32,
    },
    AnimLoopSettings {
        start_frame: 25.0f32,
        end_frame: 49.0f32,
        loop_frame: -0.1f32,
    },
];
pub static mut submenu_anims: [AnimLoopSettings; 2] = [
    AnimLoopSettings {
        start_frame: 50.0f32,
        end_frame: 74.0f32,
        loop_frame: -0.1f32,
    },
    AnimLoopSettings {
        start_frame: 75.0f32,
        end_frame: 99.0f32,
        loop_frame: -0.1f32,
    },
];
pub static mut submenu_arrow_anim: AnimLoopSettings = AnimLoopSettings {
    start_frame: 0.0f32,
    end_frame: 99.0f32,
    loop_frame: 60.0f32,
};
pub static mut mn_803EC734: [AnimLoopSettings; 5] = [
    AnimLoopSettings {
        start_frame: 0.0f32,
        end_frame: 3.0f32,
        loop_frame: -0.1f32,
    },
    AnimLoopSettings {
        start_frame: 20.0f32,
        end_frame: 23.0f32,
        loop_frame: -0.1f32,
    },
    AnimLoopSettings {
        start_frame: 40.0f32,
        end_frame: 43.0f32,
        loop_frame: -0.1f32,
    },
    AnimLoopSettings {
        start_frame: 60.0f32,
        end_frame: 63.0f32,
        loop_frame: -0.1f32,
    },
    AnimLoopSettings {
        start_frame: 80.0f32,
        end_frame: 83.0f32,
        loop_frame: -0.1f32,
    },
];
pub static mut mn_803EC770: [AnimLoopSettings; 9] = [
    AnimLoopSettings {
        start_frame: 100.0f32,
        end_frame: 103.0f32,
        loop_frame: -0.1f32,
    },
    AnimLoopSettings {
        start_frame: 120.0f32,
        end_frame: 123.0f32,
        loop_frame: -0.1f32,
    },
    AnimLoopSettings {
        start_frame: 140.0f32,
        end_frame: 143.0f32,
        loop_frame: -0.1f32,
    },
    AnimLoopSettings {
        start_frame: 160.0f32,
        end_frame: 163.0f32,
        loop_frame: -0.1f32,
    },
    AnimLoopSettings {
        start_frame: 180.0f32,
        end_frame: 183.0f32,
        loop_frame: -0.1f32,
    },
    AnimLoopSettings {
        start_frame: 30.0f32,
        end_frame: 49.0f32,
        loop_frame: -0.1f32,
    },
    AnimLoopSettings {
        start_frame: 70.0f32,
        end_frame: 89.0f32,
        loop_frame: -0.1f32,
    },
    AnimLoopSettings {
        start_frame: 90.0f32,
        end_frame: 109.0f32,
        loop_frame: -0.1f32,
    },
    AnimLoopSettings {
        start_frame: 50.0f32,
        end_frame: 69.0f32,
        loop_frame: -0.1f32,
    },
];
pub static mut mn_803EC7DC: [[u8_0; 2]; 8] = [
    [0 as ::core::ffi::c_int as u8_0, 3 as ::core::ffi::c_int as u8_0],
    [0 as ::core::ffi::c_int as u8_0, 0x63 as ::core::ffi::c_int as u8_0],
    [0 as ::core::ffi::c_int as u8_0, 2 as ::core::ffi::c_int as u8_0],
    [5 as ::core::ffi::c_int as u8_0, 0x14 as ::core::ffi::c_int as u8_0],
    [0 as ::core::ffi::c_int as u8_0, 4 as ::core::ffi::c_int as u8_0],
    [0 as ::core::ffi::c_int as u8_0, 0 as ::core::ffi::c_int as u8_0],
    [0 as ::core::ffi::c_int as u8_0, 0 as ::core::ffi::c_int as u8_0],
    [0 as ::core::ffi::c_int as u8_0, 0 as ::core::ffi::c_int as u8_0],
];
pub static mut mn_803EC7EC: [*mut StaticModelDesc; 7] = unsafe {
    [
        &raw const MenMainCursorRl01_Top as *mut StaticModelDesc,
        &raw const MenMainCursorRl02_Top as *mut StaticModelDesc,
        &raw const MenMainCursorRl03_Top as *mut StaticModelDesc,
        &raw const MenMainCursorRl04_Top as *mut StaticModelDesc,
        &raw const MenMainCursorRl05_Top as *mut StaticModelDesc,
        ::core::ptr::null::<StaticModelDesc>() as *mut StaticModelDesc,
        ::core::ptr::null::<StaticModelDesc>() as *mut StaticModelDesc,
    ]
};
pub static mut mn_803EC808: [u16_0; 8] = [
    2 as ::core::ffi::c_int as u16_0,
    9 as ::core::ffi::c_int as u16_0,
    2 as ::core::ffi::c_int as u16_0,
    4 as ::core::ffi::c_int as u16_0,
    2 as ::core::ffi::c_int as u16_0,
    0 as ::core::ffi::c_int as u16_0,
    0 as ::core::ffi::c_int as u16_0,
    0 as ::core::ffi::c_int as u16_0,
];
pub static mut mn_803EC818: [mn_803EC818_t; 7] = [
    mn_803EC818_t {
        x0: 0x26 as u8_0,
        pad_1: [
            0x27 as ::core::ffi::c_int as u8_0,
            0x28 as ::core::ffi::c_int as u8_0,
            0x29 as ::core::ffi::c_int as u8_0,
            0 as ::core::ffi::c_int as u8_0,
        ],
    },
    mn_803EC818_t {
        x0: 0x2a as u8_0,
        pad_1: [
            0 as ::core::ffi::c_int as u8_0,
            0 as ::core::ffi::c_int as u8_0,
            0 as ::core::ffi::c_int as u8_0,
            0 as ::core::ffi::c_int as u8_0,
        ],
    },
    mn_803EC818_t {
        x0: 0x2d as u8_0,
        pad_1: [
            0x2e as ::core::ffi::c_int as u8_0,
            0x2c as ::core::ffi::c_int as u8_0,
            0 as ::core::ffi::c_int as u8_0,
            0 as ::core::ffi::c_int as u8_0,
        ],
    },
    mn_803EC818_t {
        x0: 0x2f as u8_0,
        pad_1: [
            0 as ::core::ffi::c_int as u8_0,
            0 as ::core::ffi::c_int as u8_0,
            0 as ::core::ffi::c_int as u8_0,
            0 as ::core::ffi::c_int as u8_0,
        ],
    },
    mn_803EC818_t {
        x0: 0x30 as u8_0,
        pad_1: [
            0x31 as ::core::ffi::c_int as u8_0,
            0x32 as ::core::ffi::c_int as u8_0,
            0x33 as ::core::ffi::c_int as u8_0,
            0x34 as ::core::ffi::c_int as u8_0,
        ],
    },
    mn_803EC818_t {
        x0: 0x35 as u8_0,
        pad_1: [
            0 as ::core::ffi::c_int as u8_0,
            0 as ::core::ffi::c_int as u8_0,
            0 as ::core::ffi::c_int as u8_0,
            0 as ::core::ffi::c_int as u8_0,
        ],
    },
    mn_803EC818_t {
        x0: 0x36 as u8_0,
        pad_1: [
            0 as ::core::ffi::c_int as u8_0,
            0 as ::core::ffi::c_int as u8_0,
            0 as ::core::ffi::c_int as u8_0,
            0 as ::core::ffi::c_int as u8_0,
        ],
    },
];
pub static mut mn_804D4B88: [f32_0; 2] = [4.0f32, 5.0f32];
pub static mut mn_804D4B90: f32_0 = 1.0f32;
pub static mut mn_StockCountLimits: [u8_0; 2] = [
    1 as ::core::ffi::c_int as u8_0,
    0x63 as ::core::ffi::c_int as u8_0,
];
pub static mut mn_StockCountTextId: u8_0 = 0x2b as u8_0;
pub static mut mn_804D6BD8: f32_0 = 0.;
pub static mut mn_804D6BD4: s32 = 0;
pub static mut mn_804D6BD0: *mut HSD_GObj = ::core::ptr::null::<HSD_GObj>()
    as *mut HSD_GObj;
#[inline]
pub unsafe extern "C" fn mn_8022F538_GetHoveredSelection() -> u8_0 {
    melee_core::ct!("mn_8022F538_GetHoveredSelection");
    return mn_804A04F0.hovered_selection as u8_0;
}
pub unsafe extern "C" fn fn_8022F538(mut arg0: *mut HSD_GObj) {
    melee_core::ct!("fn_8022F538");
    let mut data: *mut mn_802307F8_value_view = ::core::ptr::null_mut::<
        mn_802307F8_value_view,
    >();
    let mut rules: *mut GameRules = ::core::ptr::null_mut::<GameRules>();
    let mut limits: *mut u8_0 = ::core::ptr::null_mut::<u8_0>();
    let mut buttons: u32_0 = 0;
    let mut valid: s32 = 0;
    let mut hovered: u8_0 = 0;
    let mut _pad: [::core::ffi::c_uchar; 32] = [0; 32];
    data = HSD_GObjGetUserData(mn_804D6BD0) as *mut mn_802307F8_value_view;
    buttons = mn_80229624(4 as u32_0);
    mn_804A04F0.buttons = buttons as u64_0;
    if buttons & 0x200 as u32_0 != 0 as u32_0 {
        if mn_804A04F0.hovered_selection as ::core::ffi::c_int == 5 as ::core::ffi::c_int
            || mn_804A04F0.hovered_selection as ::core::ffi::c_int
                == 6 as ::core::ffi::c_int
        {
            sfxForward();
            mn_804A04F0.entering_menu = 1 as u8_0;
            mn_804D6BC8.cooldown = 5 as u16_0;
            match mn_804A04F0.hovered_selection as ::core::ffi::c_int {
                5 => {
                    mnItemSw_802358C0();
                    HSD_GObjFree(arg0);
                }
                6 => {
                    mn_802339FC();
                    HSD_GObjFree(arg0);
                }
                _ => {}
            }
            data = HSD_GObjGetUserData(mn_804D6BD0) as *mut mn_802307F8_value_view;
            rules = gmMainLib_GetGameRules();
            (*rules).mode = (*data).fields.x2;
            (*rules).time_limit = (*data).fields.x3;
            (*rules).handicap = (*data).fields.x4;
            (*rules).damage_ratio = (*data).fields.x5;
            (*rules).stage_sel = (*data).fields.x6;
            (*rules).stock_count = (*data).fields.x9;
            return;
        }
    } else if buttons & 0x100 as u32_0 != 0 as u32_0 {
        sfxForward();
        match gm_GetCurrentGameMode() as s32 {
            1 => {
                data = (*mn_804D6BD0).user_data as *mut mn_802307F8_value_view;
                rules = gmMainLib_GetGameRules();
                (*rules).mode = (*data).fields.x2;
                (*rules).time_limit = (*data).fields.x3;
                (*rules).handicap = (*data).fields.x4;
                (*rules).damage_ratio = (*data).fields.x5;
                (*rules).stage_sel = (*data).fields.x6;
                (*rules).stock_count = (*data).fields.x9;
                mn_80229860(GM_VS as ::core::ffi::c_int as s8);
                return;
            }
            _ => {
                data = (*mn_804D6BD0).user_data as *mut mn_802307F8_value_view;
                rules = gmMainLib_GetGameRules();
                (*rules).mode = (*data).fields.x2;
                (*rules).time_limit = (*data).fields.x3;
                (*rules).handicap = (*data).fields.x4;
                (*rules).damage_ratio = (*data).fields.x5;
                (*rules).stage_sel = (*data).fields.x6;
                (*rules).stock_count = (*data).fields.x9;
                if gm_GetCurrentGameMode() as ::core::ffi::c_int
                    == GM_TOURNAMENT as ::core::ffi::c_int
                {
                    HSD_SisLib_803A5E70();
                    mn_8022EBDC();
                    gm_80190EA4();
                    return;
                }
                if gm_GetCurrentGameMode() as ::core::ffi::c_int
                    == GM_MENU as ::core::ffi::c_int
                {
                    mn_80229894(2 as s32, 3 as u16_0, 3 as s32);
                    return;
                }
                HSD_SisLib_803A5E70();
                mn_8022EBDC();
                mnCharSel_802640A0();
                return;
            }
        }
    }
    if buttons & 0x20 as u32_0 != 0 as u32_0 {
        sfxBack();
        mn_804A04F0.entering_menu = 0 as u8_0;
        data = (*mn_804D6BD0).user_data as *mut mn_802307F8_value_view;
        rules = gmMainLib_GetGameRules();
        (*rules).mode = (*data).fields.x2;
        (*rules).time_limit = (*data).fields.x3;
        (*rules).handicap = (*data).fields.x4;
        (*rules).damage_ratio = (*data).fields.x5;
        (*rules).stage_sel = (*data).fields.x6;
        (*rules).stock_count = (*data).fields.x9;
        if gm_GetCurrentGameMode() as ::core::ffi::c_int
            == GM_TOURNAMENT as ::core::ffi::c_int
        {
            HSD_SisLib_803A5E70();
            mn_8022EBDC();
            gm_80190EA4();
            return;
        }
        if gm_GetCurrentGameMode() as ::core::ffi::c_int == GM_MENU as ::core::ffi::c_int
        {
            mn_80229894(2 as s32, 3 as u16_0, 3 as s32);
            return;
        }
        HSD_SisLib_803A5E70();
        mn_8022EBDC();
        mnCharSel_802640A0();
        return;
    }
    if buttons & 1 as u32_0 != 0 as u32_0 {
        sfxMove();
        loop {
            if mn_804A04F0.hovered_selection as s32 == 0 as ::core::ffi::c_int {
                mn_804A04F0.hovered_selection = 6 as u16_0;
            } else {
                mn_804A04F0.hovered_selection = mn_804A04F0
                    .hovered_selection
                    .wrapping_sub(1);
            }
            hovered = mn_8022F538_GetHoveredSelection();
            if gm_GetCurrentGameMode() as ::core::ffi::c_int
                == GM_TOURNAMENT as ::core::ffi::c_int
                && hovered as ::core::ffi::c_int == 4 as ::core::ffi::c_int
            {
                valid = 0 as ::core::ffi::c_int as s32;
            } else {
                valid = 1 as ::core::ffi::c_int as s32;
            }
            if !(valid == 0 as ::core::ffi::c_int) {
                break;
            }
        }
        if mn_804A04F0.hovered_selection as ::core::ffi::c_int == 1 as ::core::ffi::c_int
            && (*data).fields.x2 as ::core::ffi::c_int == 1 as ::core::ffi::c_int
        {
            mn_804A04F0.confirmed_selection = (*data).fields.x9;
            return;
        }
        mn_804A04F0.confirmed_selection = (*data)
            .indexed
            .values[mn_804A04F0.hovered_selection as usize];
        return;
    }
    if buttons & 2 as u32_0 != 0 as u32_0 {
        sfxMove();
        loop {
            if mn_804A04F0.hovered_selection as s32 == 6 as ::core::ffi::c_int {
                mn_804A04F0.hovered_selection = 0 as u16_0;
            } else {
                mn_804A04F0.hovered_selection = mn_804A04F0
                    .hovered_selection
                    .wrapping_add(1);
            }
            hovered = mn_8022F538_GetHoveredSelection();
            if gm_GetCurrentGameMode() as ::core::ffi::c_int
                == GM_TOURNAMENT as ::core::ffi::c_int
                && hovered as ::core::ffi::c_int == 4 as ::core::ffi::c_int
            {
                valid = 0 as ::core::ffi::c_int as s32;
            } else {
                valid = 1 as ::core::ffi::c_int as s32;
            }
            if !(valid == 0 as ::core::ffi::c_int) {
                break;
            }
        }
        if mn_804A04F0.hovered_selection as ::core::ffi::c_int == 1 as ::core::ffi::c_int
            && (*data).fields.x2 as ::core::ffi::c_int == 1 as ::core::ffi::c_int
        {
            mn_804A04F0.confirmed_selection = (*data).fields.x9;
            return;
        }
        mn_804A04F0.confirmed_selection = (*data)
            .indexed
            .values[mn_804A04F0.hovered_selection as usize];
        return;
    }
    if mn_804A04F0.hovered_selection as ::core::ffi::c_int == 5 as ::core::ffi::c_int {
        return;
    }
    if mn_804A04F0.hovered_selection as ::core::ffi::c_int == 6 as ::core::ffi::c_int {
        return;
    }
    if mn_804A04F0.hovered_selection as ::core::ffi::c_int == 1 as ::core::ffi::c_int
        && (*data).fields.x2 as ::core::ffi::c_int == 1 as ::core::ffi::c_int
    {
        limits = &raw mut mn_StockCountLimits as *mut u8_0;
    } else {
        limits = &raw mut *(&raw mut mn_803EC7DC as *mut [u8_0; 2])
            .offset(mn_804A04F0.hovered_selection as isize) as *mut u8_0;
    }
    if buttons & 4 as u32_0 != 0 as u32_0 {
        sfxMove();
        if mn_804A04F0.confirmed_selection as ::core::ffi::c_int
            > *limits.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        {
            mn_804A04F0.confirmed_selection = mn_804A04F0
                .confirmed_selection
                .wrapping_sub(1);
            if mn_804A04F0.confirmed_selection as ::core::ffi::c_int
                == *limits.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                && mn_804A04F0.hovered_selection as ::core::ffi::c_int
                    == 1 as ::core::ffi::c_int
            {
                gm_801A36E0(4 as s32, 0x19 as s32);
            }
        } else {
            mn_804A04F0.confirmed_selection = *limits
                .offset(1 as ::core::ffi::c_int as isize);
        }
        if mn_804D6BD4 == 0 as ::core::ffi::c_int
            && mn_804A04F0.hovered_selection as ::core::ffi::c_int
                == 2 as ::core::ffi::c_int
            && mn_804A04F0.confirmed_selection as ::core::ffi::c_int
                == 1 as ::core::ffi::c_int
        {
            mn_804A04F0.confirmed_selection = 0 as u8_0;
        }
        if gm_GetCurrentGameMode() as ::core::ffi::c_int
            == GM_TOURNAMENT as ::core::ffi::c_int
            && mn_804A04F0.hovered_selection as ::core::ffi::c_int
                == 1 as ::core::ffi::c_int
            && (*data).fields.x2 as ::core::ffi::c_int != 1 as ::core::ffi::c_int
            && mn_804A04F0.confirmed_selection as ::core::ffi::c_int
                == *limits.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        {
            mn_804A04F0.confirmed_selection = *limits
                .offset(1 as ::core::ffi::c_int as isize);
        }
        if mn_804A04F0.hovered_selection as ::core::ffi::c_int == 2 as ::core::ffi::c_int
        {
            gmMainLib_8015EA80();
            mnCharSel_80264070();
        }
        return;
    }
    if buttons & 8 as u32_0 != 0 as u32_0 {
        sfxMove();
        if (mn_804A04F0.confirmed_selection as ::core::ffi::c_int)
            < *limits.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        {
            mn_804A04F0.confirmed_selection = mn_804A04F0
                .confirmed_selection
                .wrapping_add(1);
            if mn_804A04F0.confirmed_selection as ::core::ffi::c_int
                == *limits.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                && mn_804A04F0.hovered_selection as ::core::ffi::c_int
                    == 1 as ::core::ffi::c_int
            {
                gm_801A36E0(4 as s32, 0x19 as s32);
            }
        } else {
            mn_804A04F0.confirmed_selection = *limits
                .offset(0 as ::core::ffi::c_int as isize);
        }
        if mn_804D6BD4 == 0 as ::core::ffi::c_int
            && mn_804A04F0.hovered_selection as ::core::ffi::c_int
                == 2 as ::core::ffi::c_int
            && mn_804A04F0.confirmed_selection as ::core::ffi::c_int
                == 1 as ::core::ffi::c_int
        {
            mn_804A04F0.confirmed_selection = 2 as u8_0;
        }
        if gm_GetCurrentGameMode() as ::core::ffi::c_int
            == GM_TOURNAMENT as ::core::ffi::c_int
            && mn_804A04F0.hovered_selection as ::core::ffi::c_int
                == 1 as ::core::ffi::c_int
            && (*data).fields.x2 as ::core::ffi::c_int != 1 as ::core::ffi::c_int
            && mn_804A04F0.confirmed_selection as ::core::ffi::c_int
                == *limits.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        {
            mn_804A04F0.confirmed_selection = (*limits
                .offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as u8_0;
        }
        if mn_804A04F0.hovered_selection as ::core::ffi::c_int == 2 as ::core::ffi::c_int
        {
            gmMainLib_8015EA80();
            mnCharSel_80264070();
        }
    }
}
pub static mut mn_804DBDF8: JObjIndices2 = JObjIndices2 {
    idx: [2 as ::core::ffi::c_int as u8_0, 3 as ::core::ffi::c_int as u8_0],
};
pub unsafe extern "C" fn mn_8022FB88(
    mut arg0: u8_0,
    mut arg1: *mut ::core::ffi::c_void,
) {
    melee_core::ct!("mn_8022FB88");
    let mut data: *mut mn_802307F8_t = arg1 as *mut mn_802307F8_t;
    let mut digit_indices: [u8_0; 4] = [
        2 as ::core::ffi::c_int as u8_0,
        3 as ::core::ffi::c_int as u8_0,
        5 as ::core::ffi::c_int as u8_0,
        6 as ::core::ffi::c_int as u8_0,
    ];
    let mut hide_digit: *mut u8_0 = ::core::ptr::null_mut::<u8_0>();
    let mut seconds_tens: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut seconds_ones: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut time: ::core::ffi::c_int = 0;
    let mut minutes_tens: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut enabled_clock: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut minutes_ones: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut show_count: s32 = 0;
    let mut show_digit: *mut u8_0 = ::core::ptr::null_mut::<u8_0>();
    let mut digits: *mut *mut HSD_JObj = ::core::ptr::null_mut::<*mut HSD_JObj>();
    let mut hide_count: s32 = 0;
    let mut _pad: [::core::ffi::c_uchar; 4] = [0; 4];
    if arg0 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        let mut disabled_clock: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
        hide_count = 0 as ::core::ffi::c_int as s32;
        hide_digit = &raw mut digit_indices as *mut u8_0;
        while hide_count < 4 as ::core::ffi::c_int {
            HSD_JObjSetFlagsAll(
                (*data)
                    .c2rust_unnamed
                    .c2rust_unnamed
                    .x34[1 as ::core::ffi::c_int as usize]
                    .joints[*hide_digit as usize],
                JOBJ_HIDDEN as u32_0,
            );
            hide_count += 1;
            hide_digit = hide_digit.offset(1);
        }
        disabled_clock = (*data)
            .c2rust_unnamed
            .c2rust_unnamed
            .x34[1 as ::core::ffi::c_int as usize]
            .joints[4 as ::core::ffi::c_int as usize] as *mut HSD_JObj;
        HSD_JObjReqAnimAll(disabled_clock, mn_804D4B90);
        HSD_JObjAnimAll(disabled_clock);
        return;
    }
    show_count = 0 as ::core::ffi::c_int as s32;
    show_digit = &raw mut digit_indices as *mut u8_0;
    while show_count < 4 as ::core::ffi::c_int {
        HSD_JObjClearFlagsAll(
            (*data)
                .c2rust_unnamed
                .c2rust_unnamed
                .x34[1 as ::core::ffi::c_int as usize]
                .joints[*show_digit as usize],
            JOBJ_HIDDEN as u32_0,
        );
        show_count += 1;
        show_digit = show_digit.offset(1);
    }
    enabled_clock = (*data)
        .c2rust_unnamed
        .c2rust_unnamed
        .x34[1 as ::core::ffi::c_int as usize]
        .joints[4 as ::core::ffi::c_int as usize] as *mut HSD_JObj;
    HSD_JObjReqAnimAll(enabled_clock, mn_804D6BD8);
    HSD_JObjAnimAll(enabled_clock);
    digits = &raw mut (*(&raw mut (*data).c2rust_unnamed.c2rust_unnamed.x34
        as *mut C2RustUnnamed_3)
        .offset(1 as ::core::ffi::c_int as isize))
        .joints as *mut *mut HSD_JObj as *mut *mut HSD_JObj;
    time = arg0 as ::core::ffi::c_int;
    minutes_tens = *digits.offset(2 as ::core::ffi::c_int as isize);
    HSD_JObjReqAnimAll(minutes_tens, (time / 10 as ::core::ffi::c_int) as u8_0 as f32_0);
    HSD_JObjAnimAll(minutes_tens);
    minutes_ones = *digits.offset(3 as ::core::ffi::c_int as isize);
    HSD_JObjReqAnimAll(minutes_ones, (time % 10 as ::core::ffi::c_int) as u8_0 as f32_0);
    HSD_JObjAnimAll(minutes_ones);
    seconds_tens = *digits.offset(5 as ::core::ffi::c_int as isize);
    HSD_JObjReqAnimAll(seconds_tens, 0.0f32);
    HSD_JObjAnimAll(seconds_tens);
    seconds_ones = *digits.offset(6 as ::core::ffi::c_int as isize);
    HSD_JObjReqAnimAll(seconds_ones, 0.0f32);
    HSD_JObjAnimAll(seconds_ones);
}
pub static mut mn_804DBE04: JObjIndices2 = JObjIndices2 {
    idx: [7 as ::core::ffi::c_int as u8_0, 8 as ::core::ffi::c_int as u8_0],
};
pub static mut mn_804DBE08: JObjIndices5 = JObjIndices5 {
    idx: [
        2 as ::core::ffi::c_int as u8_0,
        3 as ::core::ffi::c_int as u8_0,
        4 as ::core::ffi::c_int as u8_0,
        5 as ::core::ffi::c_int as u8_0,
        6 as ::core::ffi::c_int as u8_0,
    ],
};
pub unsafe extern "C" fn mn_8022FD18(mut arg0: u8_0) {
    melee_core::ct!("mn_8022FD18");
    let mut pad: [u8_0; 8] = [0; 8];
    let mut data: *mut mn_802307F8_t = (*mn_804D6BD0).user_data as *mut mn_802307F8_t;
    let mut stock_digits: JObjIndices2 = JObjIndices2 { idx: [0; 2] };
    let mut time_indices: JObjIndices5 = JObjIndices5 { idx: [0; 5] };
    let mut jobjs: *mut *mut HSD_JObj = ::core::ptr::null_mut::<*mut HSD_JObj>();
    let mut jobj: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut jobj2: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut ptr0: *mut u8_0 = ::core::ptr::null_mut::<u8_0>();
    let mut ptr1: *mut u8_0 = ::core::ptr::null_mut::<u8_0>();
    let mut ptr2: *mut u8_0 = ::core::ptr::null_mut::<u8_0>();
    let mut ptr3: *mut u8_0 = ::core::ptr::null_mut::<u8_0>();
    let mut i: s32 = 0;
    let mut data2: *mut mn_802307F8_t = ::core::ptr::null_mut::<mn_802307F8_t>();
    let mut val: u8_0 = 0;
    data2 = data;
    stock_digits = mn_804DBE04;
    time_indices = mn_804DBE08;
    if arg0 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        i = 0 as ::core::ffi::c_int as s32;
        ptr0 = &raw mut stock_digits.idx as *mut u8_0;
        while i < 2 as ::core::ffi::c_int {
            HSD_JObjSetFlagsAll(
                (*data)
                    .c2rust_unnamed
                    .c2rust_unnamed
                    .x34[1 as ::core::ffi::c_int as usize]
                    .joints[*ptr0 as usize],
                JOBJ_HIDDEN as u32_0,
            );
            i += 1;
            ptr0 = ptr0.offset(1);
        }
        i = 0 as ::core::ffi::c_int as s32;
        ptr1 = &raw mut time_indices.idx as *mut u8_0;
        while i < 5 as ::core::ffi::c_int {
            HSD_JObjClearFlagsAll(
                (*data)
                    .c2rust_unnamed
                    .c2rust_unnamed
                    .x34[1 as ::core::ffi::c_int as usize]
                    .joints[*ptr1 as usize],
                JOBJ_HIDDEN as u32_0,
            );
            i += 1;
            ptr1 = ptr1.offset(1);
        }
        mn_8022FB88((*data2).x3, data2 as *mut ::core::ffi::c_void);
        return;
    }
    i = 0 as ::core::ffi::c_int as s32;
    ptr2 = &raw mut stock_digits.idx as *mut u8_0;
    while i < 2 as ::core::ffi::c_int {
        HSD_JObjClearFlagsAll(
            (*data)
                .c2rust_unnamed
                .c2rust_unnamed
                .x34[1 as ::core::ffi::c_int as usize]
                .joints[*ptr2 as usize],
            JOBJ_HIDDEN as u32_0,
        );
        i += 1;
        ptr2 = ptr2.offset(1);
    }
    i = 0 as ::core::ffi::c_int as s32;
    ptr3 = &raw mut time_indices.idx as *mut u8_0;
    while i < 5 as ::core::ffi::c_int {
        HSD_JObjSetFlagsAll(
            (*data)
                .c2rust_unnamed
                .c2rust_unnamed
                .x34[1 as ::core::ffi::c_int as usize]
                .joints[*ptr3 as usize],
            JOBJ_HIDDEN as u32_0,
        );
        i += 1;
        ptr3 = ptr3.offset(1);
    }
    val = (*data).x9;
    jobjs = &raw mut (*(&raw mut (*data).c2rust_unnamed.c2rust_unnamed.x34
        as *mut C2RustUnnamed_3)
        .offset(1 as ::core::ffi::c_int as isize))
        .joints as *mut *mut HSD_JObj as *mut *mut HSD_JObj;
    jobj = *jobjs.offset(7 as ::core::ffi::c_int as isize);
    HSD_JObjReqAnimAll(
        jobj,
        ((*data).x9 as ::core::ffi::c_int / 10 as ::core::ffi::c_int) as u8_0 as f32_0,
    );
    HSD_JObjAnimAll(jobj);
    jobj2 = *jobjs.offset(8 as ::core::ffi::c_int as isize);
    HSD_JObjReqAnimAll(
        jobj2,
        (val as ::core::ffi::c_int % 10 as ::core::ffi::c_int) as u8_0 as f32_0,
    );
    HSD_JObjAnimAll(jobj2);
}
#[inline]
pub unsafe extern "C" fn mn_8022FEC8_AnimDigit(mut jobj: *mut HSD_JObj, mut digit: u8_0) {
    melee_core::ct!("mn_8022FEC8_AnimDigit");
    HSD_JObjReqAnimAll(jobj, digit as f32_0);
    HSD_JObjAnimAll(jobj);
}
#[inline]
pub unsafe extern "C" fn mn_8022FEC8_AnimDamageDigits(
    mut value: u8_0,
    mut data: *mut mn_802307F8_t,
) {
    melee_core::ct!("mn_8022FEC8_AnimDamageDigits");
    mn_8022FEC8_AnimDigit(
        HSD_JObjGetChild(
            (*data)
                .c2rust_unnamed
                .c2rust_unnamed
                .x34[3 as ::core::ffi::c_int as usize]
                .joints[2 as ::core::ffi::c_int as usize],
        ),
        (value as ::core::ffi::c_int / 10 as ::core::ffi::c_int) as u8_0,
    );
    mn_8022FEC8_AnimDigit(
        HSD_JObjGetChild(
            (*data)
                .c2rust_unnamed
                .c2rust_unnamed
                .x34[3 as ::core::ffi::c_int as usize]
                .joints[3 as ::core::ffi::c_int as usize],
        ),
        (value as ::core::ffi::c_int % 10 as ::core::ffi::c_int) as u8_0,
    );
}
#[inline]
pub unsafe extern "C" fn mn_8022FEC8_GetSettings(
    mut rule_kind: u8_0,
    mut rule_value: u8_0,
) -> *mut AnimLoopSettings {
    melee_core::ct!("mn_8022FEC8_GetSettings");
    let mut default_value: u8_0 = 0;
    if rule_kind as ::core::ffi::c_int != 0 as ::core::ffi::c_int
        && rule_kind as ::core::ffi::c_int != 2 as ::core::ffi::c_int
        && rule_kind as ::core::ffi::c_int != 4 as ::core::ffi::c_int
    {
        return ::core::ptr::null_mut::<AnimLoopSettings>();
    }
    default_value = mn_803EC7DC[rule_kind as usize][1 as ::core::ffi::c_int as usize];
    return (&raw mut mn_803EC770 as *mut AnimLoopSettings)
        .offset(
            (default_value as ::core::ffi::c_int - rule_value as ::core::ffi::c_int)
                as isize,
        ) as *mut AnimLoopSettings;
}
#[inline]
pub unsafe extern "C" fn mn_8022FEC8_AnimStockDigits(
    mut data: *mut mn_802307F8_t,
    mut value: ::core::ffi::c_int,
) {
    melee_core::ct!("mn_8022FEC8_AnimStockDigits");
    let mut digit_jobjs: *mut *mut HSD_JObj = ::core::ptr::null_mut::<*mut HSD_JObj>();
    let mut digit_jobj: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut digit_jobj2: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    digit_jobjs = &raw mut (*(&raw mut (*data).c2rust_unnamed.c2rust_unnamed.x34
        as *mut C2RustUnnamed_3)
        .offset(1 as ::core::ffi::c_int as isize))
        .joints as *mut *mut HSD_JObj as *mut *mut HSD_JObj;
    digit_jobj = *digit_jobjs.offset(7 as ::core::ffi::c_int as isize);
    mn_8022FEC8_AnimDigit(digit_jobj, (value / 10 as ::core::ffi::c_int) as u8_0);
    digit_jobj2 = *digit_jobjs.offset(8 as ::core::ffi::c_int as isize);
    mn_8022FEC8_AnimDigit(digit_jobj2, (value % 10 as ::core::ffi::c_int) as u8_0);
}
pub unsafe extern "C" fn mn_8022FEC8(
    mut arg0: *mut HSD_GObj,
    mut arg1: *mut HSD_JObj,
    mut arg2: u8_0,
    mut arg3: u8_0,
) {
    melee_core::ct!("mn_8022FEC8");
    let mut settings: *mut AnimLoopSettings = ::core::ptr::null_mut::<
        AnimLoopSettings,
    >();
    let mut data: *mut mn_802307F8_t = ::core::ptr::null_mut::<mn_802307F8_t>();
    let mut value: ::core::ffi::c_int = 0;
    let mut default_value: u8_0 = 0;
    let mut _pad: [::core::ffi::c_uchar; 16] = [0; 16];
    data = HSD_GObjGetUserData(arg0) as *mut mn_802307F8_t;
    match arg2 as s32 {
        1 => {
            if (*((*mn_804D6BD0).user_data as *mut mn_802307F8_t)).x2
                as ::core::ffi::c_int != 1 as ::core::ffi::c_int
            {
                mn_8022FB88(arg3, data as *mut ::core::ffi::c_void);
                return;
            }
            value = arg3 as ::core::ffi::c_int;
            mn_8022FEC8_AnimStockDigits(data, value);
            return;
        }
        3 => {
            mn_8022FEC8_AnimDamageDigits(arg3, data);
            return;
        }
        0 | 2 | 4 => {
            if mn_804A04F0.buttons & 4 as u64_0 != 0 as u64_0 {
                settings = mn_8022FEC8_GetSettings(arg2, arg3);
                HSD_JObjReqAnimAll(arg1, (*settings).start_frame);
            } else {
                if arg2 as ::core::ffi::c_int != 0 as ::core::ffi::c_int
                    && arg2 as ::core::ffi::c_int != 2 as ::core::ffi::c_int
                    && arg2 as ::core::ffi::c_int != 4 as ::core::ffi::c_int
                {
                    settings = ::core::ptr::null_mut::<AnimLoopSettings>();
                } else {
                    default_value = mn_803EC7DC[arg2
                        as usize][1 as ::core::ffi::c_int as usize];
                    if arg3 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
                        settings = (&raw mut mn_803EC734 as *mut AnimLoopSettings)
                            .offset(default_value as isize) as *mut AnimLoopSettings;
                    } else {
                        settings = (&raw mut mn_803EC734 as *mut AnimLoopSettings)
                            .offset(
                                (arg3 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                                    as isize,
                            ) as *mut AnimLoopSettings;
                    }
                }
                HSD_JObjReqAnimAll(arg1, (*settings).start_frame);
            }
            HSD_JObjAnimAll(arg1);
            return;
        }
        5 | 6 | _ => return,
    };
}
pub unsafe extern "C" fn mn_80230198(
    mut gobj: *mut HSD_GObj,
    mut jobj: *mut HSD_JObj,
    mut mode: u8_0,
) {
    melee_core::ct!("mn_80230198");
    let mut settings: *mut AnimLoopSettings = ::core::ptr::null_mut::<
        AnimLoopSettings,
    >();
    let mut frame: f32_0 = 0.;
    let mut i: ::core::ffi::c_int = 0;
    let mut _pad: [::core::ffi::c_uchar; 8] = [0; 8];
    match mode as ::core::ffi::c_int {
        0 | 2 | 4 => {
            frame = mn_8022F298(jobj as *mut HSD_JObj);
            i = 0 as ::core::ffi::c_int;
            while i < 5 as ::core::ffi::c_int {
                settings = (&raw mut mn_803EC770 as *mut AnimLoopSettings)
                    .offset(i as isize) as *mut AnimLoopSettings;
                if mn_803EC770[i as usize].start_frame <= frame
                    && frame <= mn_803EC770[i as usize].end_frame
                {
                    break;
                }
                settings = (&raw mut mn_803EC734 as *mut AnimLoopSettings)
                    .offset(i as isize) as *mut AnimLoopSettings;
                if mn_803EC734[i as usize].start_frame <= frame
                    && frame <= mn_803EC734[i as usize].end_frame
                {
                    break;
                }
                i += 1;
            }
            mn_8022ED6C(jobj as *mut HSD_JObj, settings);
        }
        1 | 3 | 5 | 6 | _ => {}
    };
}
#[inline]
pub unsafe extern "C" fn mn_80230274_InitOptionRoots(
    mut option_roots: *mut *mut HSD_JObj,
    mut base: *const u16_0,
    mut data: *mut mn_802307F8_t,
    mut count: u8_0,
    mut i: *mut s32,
) {
    melee_core::ct!("mn_80230274_InitOptionRoots");
    let mut option_root: *mut *mut HSD_JObj = ::core::ptr::null_mut::<*mut HSD_JObj>();
    let mut j8: u8_0 = 0;
    let mut j: s32 = 0;
    let mut visible: s32 = 0;
    option_root = option_roots;
    *i = 0 as ::core::ffi::c_int as s32;
    while *i < count as ::core::ffi::c_int {
        let mut valid: s32 = 0;
        if gm_GetCurrentGameMode() as ::core::ffi::c_int
            == GM_TOURNAMENT as ::core::ffi::c_int
            && *i as u8_0 as ::core::ffi::c_int == 4 as ::core::ffi::c_int
        {
            valid = 0 as ::core::ffi::c_int as s32;
        } else {
            valid = 1 as ::core::ffi::c_int as s32;
        }
        if valid != 0 {
            let mut v: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
            let mut p: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
            visible = 0 as ::core::ffi::c_int as s32;
            j = visible;
            while j < *i as u8_0 as s32 {
                let mut valid2: s32 = 0;
                j8 = j as u8_0;
                if gm_GetCurrentGameMode() as ::core::ffi::c_int
                    == GM_TOURNAMENT as ::core::ffi::c_int
                    && j8 as ::core::ffi::c_int == 4 as ::core::ffi::c_int
                {
                    valid2 = 0 as ::core::ffi::c_int as s32;
                } else {
                    valid2 = 1 as ::core::ffi::c_int as s32;
                }
                if valid2 != 0 {
                    visible += 1;
                }
                j += 1;
            }
            p = (*data)
                .c2rust_unnamed
                .xC[*base.offset(visible as u8_0 as isize) as usize] as *mut HSD_JObj;
            if p.is_null() {
                v = ::core::ptr::null_mut::<HSD_JObj>();
            } else {
                v = (*p).child as *mut HSD_JObj;
            }
            *option_root = v;
        }
        option_root = option_root.offset(1);
        *i += 1;
    }
}
pub unsafe extern "C" fn mn_80230274(
    mut arg0: *mut HSD_GObj,
    mut arg1: ::core::ffi::c_int,
    mut arg2: ::core::ffi::c_int,
) {
    melee_core::ct!("mn_80230274");
    let mut option_roots: [*mut HSD_JObj; 8] = [::core::ptr::null_mut::<HSD_JObj>(); 8];
    let mut roots: [*mut HSD_JObj; 17] = [::core::ptr::null_mut::<HSD_JObj>(); 17];
    let mut pad_: [u8_0; 4] = [0; 4];
    let mut indices: [u16_0; 17] = [0; 17];
    let mut pad_2: [u8_0; 4] = [0; 4];
    let mut settings: *mut AnimLoopSettings = ::core::ptr::null_mut::<
        AnimLoopSettings,
    >();
    let mut jobj: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut i: s32 = 0;
    let mut user_data: *mut mn_802307F8_t = ::core::ptr::null_mut::<mn_802307F8_t>();
    let mut focus: u8_0 = 0;
    let mut count: u8_0 = 0;
    let mut selected: u8_0 = 0;
    let mut base: *mut u16_0 = ::core::ptr::null_mut::<u16_0>();
    let mut tail_i: s32 = 0;
    let mut data: *mut mn_802307F8_t = ::core::ptr::null_mut::<mn_802307F8_t>();
    let mut _pad: [::core::ffi::c_uchar; 32] = [0; 32];
    user_data = (*arg0).user_data as *mut mn_802307F8_t;
    count = (*(&raw mut mn_803EB6B0 as *mut MenuKindData)
        .offset(13 as ::core::ffi::c_int as isize))
        .selection_count;
    base = &raw mut mn_803EC600 as *mut u16_0;
    data = user_data;
    i = 0 as ::core::ffi::c_int as s32;
    while 0x11 as ::core::ffi::c_int > i {
        indices[i as usize] = i as u16_0;
        i += 1;
    }
    mn_80230274_InitOptionRoots(
        &raw mut option_roots as *mut *mut HSD_JObj,
        base,
        data,
        count,
        &raw mut i,
    );
    if arg1 != 0 as ::core::ffi::c_int {
        let mut hovered: u8_0 = 0;
        selected = (*data).x1;
        lb_8001204C(
            option_roots[selected as usize],
            &raw mut roots as *mut *mut HSD_JObj,
            &raw mut indices as *mut u16_0,
            0x11 as ::core::ffi::c_int,
        );
        HSD_JObjSetFlagsAll(
            roots[16 as ::core::ffi::c_int as usize],
            JOBJ_HIDDEN as u32_0,
        );
        HSD_JObjSetFlagsAll(
            roots[13 as ::core::ffi::c_int as usize],
            JOBJ_HIDDEN as u32_0,
        );
        jobj = roots[2 as ::core::ffi::c_int as usize];
        if (selected as ::core::ffi::c_int - 5 as ::core::ffi::c_int) as u8_0
            as ::core::ffi::c_int <= 1 as ::core::ffi::c_int
        {
            HSD_JObjReqAnimAll(
                jobj,
                submenu_anims[0 as ::core::ffi::c_int as usize].start_frame,
            );
        } else {
            HSD_JObjReqAnimAll(
                jobj,
                option_anims[0 as ::core::ffi::c_int as usize].start_frame,
            );
        }
        HSD_JObjAnimAll(jobj);
        if selected as ::core::ffi::c_int == 1 as ::core::ffi::c_int
            && (*((*mn_804D6BD0).user_data as *mut mn_802307F8_t)).x2
                as ::core::ffi::c_int == 1 as ::core::ffi::c_int
        {
            HSD_JObjReqAnim(
                roots[7 as ::core::ffi::c_int as usize],
                mn_804D4B88[0 as ::core::ffi::c_int as usize],
            );
        } else {
            HSD_JObjReqAnim(
                roots[7 as ::core::ffi::c_int as usize],
                rule_label_frames[selected as usize][0 as ::core::ffi::c_int as usize],
            );
        }
        HSD_JObjAnim(roots[7 as ::core::ffi::c_int as usize]);
        HSD_JObjSetFlagsAll(
            roots[8 as ::core::ffi::c_int as usize],
            JOBJ_HIDDEN as u32_0,
        );
        hovered = mn_8022F538_GetHoveredSelection();
        lb_8001204C(
            option_roots[hovered as usize],
            &raw mut roots as *mut *mut HSD_JObj,
            &raw mut indices as *mut u16_0,
            0x11 as ::core::ffi::c_int,
        );
        jobj = roots[16 as ::core::ffi::c_int as usize];
        HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32_0);
        HSD_JObjReqAnimAll(jobj, highlight_anim.start_frame);
        if hovered as ::core::ffi::c_int != 5 as ::core::ffi::c_int
            && hovered as ::core::ffi::c_int != 6 as ::core::ffi::c_int
        {
            jobj = roots[13 as ::core::ffi::c_int as usize];
            HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32_0);
            HSD_JObjReqAnimAll(jobj, value_arrow_anim.start_frame);
            jobj = roots[2 as ::core::ffi::c_int as usize];
            HSD_JObjReqAnimAll(
                jobj,
                option_anims[1 as ::core::ffi::c_int as usize].start_frame,
            );
            HSD_JObjAnimAll(jobj);
        } else {
            jobj = roots[2 as ::core::ffi::c_int as usize];
            HSD_JObjReqAnimAll(
                jobj,
                submenu_anims[1 as ::core::ffi::c_int as usize].start_frame,
            );
            HSD_JObjAnimAll(jobj);
        }
        if hovered as ::core::ffi::c_int == 1 as ::core::ffi::c_int
            && (*((*mn_804D6BD0).user_data as *mut mn_802307F8_t)).x2
                as ::core::ffi::c_int == 1 as ::core::ffi::c_int
        {
            HSD_JObjReqAnim(
                roots[7 as ::core::ffi::c_int as usize],
                mn_804D4B88[1 as ::core::ffi::c_int as usize],
            );
        } else {
            HSD_JObjReqAnim(
                roots[7 as ::core::ffi::c_int as usize],
                rule_label_frames[hovered as usize][1 as ::core::ffi::c_int as usize],
            );
        }
        tail_i = (hovered as ::core::ffi::c_int - 5 as ::core::ffi::c_int) as s32;
        HSD_JObjAnim(roots[7 as ::core::ffi::c_int as usize]);
        if tail_i as u8_0 as ::core::ffi::c_int <= 1 as ::core::ffi::c_int {
            jobj = roots[8 as ::core::ffi::c_int as usize];
            HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32_0);
            HSD_JObjReqAnimAll(jobj, submenu_arrow_anim.start_frame);
            HSD_JObjAnimAll(jobj);
        }
    }
    if arg2 != 0 as ::core::ffi::c_int {
        mn_8022FEC8(
            arg0,
            (*data)
                .c2rust_unnamed
                .c2rust_unnamed
                .x34[mn_804A04F0.hovered_selection as usize]
                .joints[0 as ::core::ffi::c_int as usize],
            mn_804A04F0.hovered_selection as u8_0,
            mn_804A04F0.confirmed_selection,
        );
    }
    i = 0 as ::core::ffi::c_int as s32;
    while i < count as ::core::ffi::c_int {
        let mut valid: s32 = 0;
        if gm_GetCurrentGameMode() as ::core::ffi::c_int
            == GM_TOURNAMENT as ::core::ffi::c_int
            && i as u8_0 as ::core::ffi::c_int == 4 as ::core::ffi::c_int
        {
            valid = 0 as ::core::ffi::c_int as s32;
        } else {
            valid = 1 as ::core::ffi::c_int as s32;
        }
        if valid != 0 {
            lb_8001204C(
                option_roots[i as usize],
                &raw mut roots as *mut *mut HSD_JObj,
                &raw mut indices as *mut u16_0,
                0x11 as ::core::ffi::c_int,
            );
            focus = (if arg1 != 0 as ::core::ffi::c_int {
                mn_804A04F0.hovered_selection as ::core::ffi::c_int
            } else {
                (*data).x1 as ::core::ffi::c_int
            }) as u8_0;
            if i == focus as ::core::ffi::c_int {
                mn_8022ED6C(
                    roots[16 as ::core::ffi::c_int as usize],
                    &raw mut highlight_anim,
                );
            }
            if i == focus as ::core::ffi::c_int && i != 5 as ::core::ffi::c_int
                && i != 6 as ::core::ffi::c_int
            {
                mn_8022ED6C(
                    roots[13 as ::core::ffi::c_int as usize],
                    &raw mut value_arrow_anim,
                );
            }
            if (i as ::core::ffi::c_int - 5 as ::core::ffi::c_int) as u32_0 <= 1 as u32_0
            {
                settings = (&raw mut submenu_anims as *mut AnimLoopSettings)
                    .offset(
                        (i == focus as ::core::ffi::c_int) as ::core::ffi::c_int as isize,
                    ) as *mut AnimLoopSettings;
            } else {
                settings = (&raw mut option_anims as *mut AnimLoopSettings)
                    .offset(
                        (i == focus as ::core::ffi::c_int) as ::core::ffi::c_int as isize,
                    ) as *mut AnimLoopSettings;
            }
            mn_8022ED6C(roots[2 as ::core::ffi::c_int as usize], settings);
            mn_8022ED6C(
                roots[8 as ::core::ffi::c_int as usize],
                &raw mut submenu_arrow_anim,
            );
            mn_80230198(
                arg0,
                (*data)
                    .c2rust_unnamed
                    .c2rust_unnamed
                    .x34[i as usize]
                    .joints[0 as ::core::ffi::c_int as usize],
                i as u8_0,
            );
            if i == 1 as ::core::ffi::c_int && arg2 != 0 as ::core::ffi::c_int
                && focus as ::core::ffi::c_int == 0 as ::core::ffi::c_int
            {
                if mn_804A04F0.confirmed_selection as ::core::ffi::c_int
                    == 1 as ::core::ffi::c_int
                {
                    HSD_JObjReqAnim(
                        roots[7 as ::core::ffi::c_int as usize],
                        mn_804D4B88[0 as ::core::ffi::c_int as usize],
                    );
                } else {
                    HSD_JObjReqAnim(
                        roots[7 as ::core::ffi::c_int as usize],
                        rule_label_frames[1 as ::core::ffi::c_int
                            as usize][0 as ::core::ffi::c_int as usize],
                    );
                }
                HSD_JObjAnim(roots[7 as ::core::ffi::c_int as usize]);
                mn_8022FD18(
                    (mn_804A04F0.confirmed_selection as ::core::ffi::c_int
                        != 1 as ::core::ffi::c_int) as ::core::ffi::c_int as u8_0,
                );
            }
        }
        i += 1;
    }
}
pub static mut time_digit_indices: JObjIndices6 = JObjIndices6 {
    idx: [
        2 as ::core::ffi::c_int as u8_0,
        3 as ::core::ffi::c_int as u8_0,
        5 as ::core::ffi::c_int as u8_0,
        6 as ::core::ffi::c_int as u8_0,
        7 as ::core::ffi::c_int as u8_0,
        8 as ::core::ffi::c_int as u8_0,
    ],
};
pub unsafe extern "C" fn mn_802307F8(
    mut arg0: *mut mn_802307F8_t,
    mut arg1: s32,
    mut arg2: s32,
) {
    melee_core::ct!("mn_802307F8");
    let mut text: *mut HSD_Text = ::core::ptr::null_mut::<HSD_Text>();
    let mut text_id: u8_0 = 0;
    if !(*arg0).text.is_null() {
        HSD_SisLib_803A5CC4((*arg0).text);
        (*arg0).text = ::core::ptr::null_mut::<HSD_Text>();
    }
    if arg1 == 1 as ::core::ffi::c_int
        && (*arg0).x2 as ::core::ffi::c_int == 1 as ::core::ffi::c_int
    {
        text_id = mn_StockCountTextId;
    } else if arg1 == 1 as ::core::ffi::c_int || arg1 == 3 as ::core::ffi::c_int
        || (arg1 as ::core::ffi::c_int - 5 as ::core::ffi::c_int) as u32_0
            <= 1 as ::core::ffi::c_uint
    {
        text_id = mn_803EC818[arg1 as usize].x0;
    } else {
        text_id = *(&raw mut (*(&raw mut mn_803EC818 as *mut mn_803EC818_t)
            .offset(arg1 as isize))
            .x0)
            .offset(arg2 as isize);
    }
    text = HSD_SisLib_803A5ACC(
        0 as ::core::ffi::c_int,
        1 as s32,
        -9.5f32,
        8.0f32,
        17.0f32,
        364.68332f32,
        76.77544f32,
    );
    (*arg0).text = text;
    (*text).font_size.x = 0.0521f32 as f32_0;
    (*text).font_size.y = (*text).font_size.x;
    HSD_SisLib_803A6368(text, text_id as s32);
}
pub unsafe extern "C" fn mn_802308F0(
    mut gobj: *mut HSD_GObj,
    mut arg1: ::core::ffi::c_int,
    mut arg2: ::core::ffi::c_int,
) {
    melee_core::ct!("mn_802308F0");
    let mut data: *mut mn_802307F8_t = HSD_GObjGetUserData(gobj) as *mut mn_802307F8_t;
    let mut sel: u16_0 = 0;
    let mut kind: u8_0 = 0;
    if arg1 != 0 as ::core::ffi::c_int {
        sel = mn_804A04F0.hovered_selection;
    } else {
        sel = (*data).x1 as u16_0;
    }
    kind = sel as u8_0;
    match (*data).xA as ::core::ffi::c_int {
        2 | 4 => {
            if !(*data).text.is_null() {
                HSD_SisLib_803A5CC4((*data).text);
                (*data).text = ::core::ptr::null_mut::<HSD_Text>();
            }
        }
        1 | 3 => {
            if (*data).text.is_null() {
                mn_802307F8(data, kind as s32, mn_804A04F0.confirmed_selection as s32);
            }
        }
        0 => {
            if arg1 != 0 as ::core::ffi::c_int
                || arg2 != 0 as ::core::ffi::c_int
                    && (kind as ::core::ffi::c_int == 0 as ::core::ffi::c_int
                        || kind as ::core::ffi::c_int == 2 as ::core::ffi::c_int
                        || kind as ::core::ffi::c_int == 4 as ::core::ffi::c_int)
            {
                mn_802307F8(data, kind as s32, mn_804A04F0.confirmed_selection as s32);
            }
        }
        5 | _ => {}
    };
}
pub unsafe extern "C" fn fn_802309F0(mut arg0: *mut HSD_GObj) {
    melee_core::ct!("fn_802309F0");
    let mut rules: *mut GameRules = ::core::ptr::null_mut::<GameRules>();
    let mut jobj: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut anim: *mut AnimLoopSettings = ::core::ptr::null_mut::<AnimLoopSettings>();
    let mut ptr: *mut u8_0 = ::core::ptr::null_mut::<u8_0>();
    let mut update_menu: s32 = 0;
    let mut update_selection: s32 = 0;
    let mut update_value: s32 = 0;
    let mut state: u8_0 = 0;
    let mut data: *mut mn_802307F8_t = ::core::ptr::null_mut::<mn_802307F8_t>();
    let mut _pad: [::core::ffi::c_uchar; 24] = [0; 24];
    update_menu = 0 as ::core::ffi::c_int as s32;
    update_selection = 0 as ::core::ffi::c_int as s32;
    update_value = 0 as ::core::ffi::c_int as s32;
    data = (*arg0).user_data as *mut mn_802307F8_t;
    if ((*data).xA as ::core::ffi::c_int == 0 as ::core::ffi::c_int
        || (*data).xA as ::core::ffi::c_int == 1 as ::core::ffi::c_int
        || (*data).xA as ::core::ffi::c_int == 3 as ::core::ffi::c_int)
        && (*data).x0 as ::core::ffi::c_int != mn_804A04F0.cur_menu as ::core::ffi::c_int
    {
        if (mn_804A04F0.cur_menu as ::core::ffi::c_int - 0xf as ::core::ffi::c_int)
            as u8_0 as ::core::ffi::c_int <= 1 as ::core::ffi::c_int
        {
            (*data).xA = 4 as u8_0;
        } else {
            (*data).xA = 2 as u8_0;
        }
        state = (*data).xA;
        jobj = (*data).c2rust_unnamed.xC[2 as ::core::ffi::c_int as usize]
            as *mut HSD_JObj;
        match state as ::core::ffi::c_int {
            1 => {
                anim = (&raw mut mn_803EC770 as *mut AnimLoopSettings)
                    .offset(5 as ::core::ffi::c_int as isize) as *mut AnimLoopSettings;
            }
            2 => {
                anim = (&raw mut mn_803EC770 as *mut AnimLoopSettings)
                    .offset(7 as ::core::ffi::c_int as isize) as *mut AnimLoopSettings;
            }
            3 => {
                anim = (&raw mut mn_803EC770 as *mut AnimLoopSettings)
                    .offset(6 as ::core::ffi::c_int as isize) as *mut AnimLoopSettings;
            }
            4 => {
                anim = (&raw mut mn_803EC770 as *mut AnimLoopSettings)
                    .offset(8 as ::core::ffi::c_int as isize) as *mut AnimLoopSettings;
            }
            _ => {}
        }
        HSD_JObjReqAnim(jobj, (*anim).start_frame);
        HSD_JObjAnim(jobj);
        if (*data).xA as ::core::ffi::c_int == 0 as ::core::ffi::c_int
            || (*data).xA as ::core::ffi::c_int == 1 as ::core::ffi::c_int
            || (*data).xA as ::core::ffi::c_int == 3 as ::core::ffi::c_int
        {
            update_menu = 1 as ::core::ffi::c_int as s32;
            update_selection = 1 as ::core::ffi::c_int as s32;
            update_value = 1 as ::core::ffi::c_int as s32;
        }
    }
    state = (*data).xA;
    if state as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        jobj = (*data).c2rust_unnamed.xC[2 as ::core::ffi::c_int as usize]
            as *mut HSD_JObj;
        match state as ::core::ffi::c_int {
            1 => {
                anim = (&raw mut mn_803EC770 as *mut AnimLoopSettings)
                    .offset(5 as ::core::ffi::c_int as isize) as *mut AnimLoopSettings;
            }
            2 => {
                anim = (&raw mut mn_803EC770 as *mut AnimLoopSettings)
                    .offset(7 as ::core::ffi::c_int as isize) as *mut AnimLoopSettings;
            }
            3 => {
                anim = (&raw mut mn_803EC770 as *mut AnimLoopSettings)
                    .offset(6 as ::core::ffi::c_int as isize) as *mut AnimLoopSettings;
            }
            4 => {
                anim = (&raw mut mn_803EC770 as *mut AnimLoopSettings)
                    .offset(8 as ::core::ffi::c_int as isize) as *mut AnimLoopSettings;
            }
            _ => {}
        }
        if mn_8022F298(jobj as *mut HSD_JObj) >= (*anim).end_frame {
            match (*data).xA as ::core::ffi::c_int {
                1 | 3 => {
                    (*data).xA = 0 as u8_0;
                }
                2 | 4 => {
                    HSD_GObjFree(arg0);
                    return;
                }
                _ => {}
            }
        }
        HSD_JObjAnim(jobj);
    }
    if (*data).xA as ::core::ffi::c_int == 0 as ::core::ffi::c_int
        || (*data).xA as ::core::ffi::c_int == 1 as ::core::ffi::c_int
        || (*data).xA as ::core::ffi::c_int == 3 as ::core::ffi::c_int
    {
        update_selection = ((*data).x1 as ::core::ffi::c_int
            != mn_804A04F0.hovered_selection as ::core::ffi::c_int) as ::core::ffi::c_int
            as s32;
        if mn_804A04F0.hovered_selection as ::core::ffi::c_int == 1 as ::core::ffi::c_int
            && (*data).x2 as ::core::ffi::c_int == 1 as ::core::ffi::c_int
        {
            if (*data).x9 as ::core::ffi::c_int
                != mn_804A04F0.confirmed_selection as ::core::ffi::c_int
            {
                update_value = 1 as ::core::ffi::c_int as s32;
            }
        } else {
            ptr = data as *mut u8_0;
            ptr = ptr
                .offset(mn_804A04F0.hovered_selection as ::core::ffi::c_int as isize);
            if *ptr.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                != mn_804A04F0.confirmed_selection as ::core::ffi::c_int
            {
                update_value = 1 as ::core::ffi::c_int as s32;
            }
        }
    }
    mn_80230274(
        arg0,
        update_selection as ::core::ffi::c_int,
        update_value as ::core::ffi::c_int,
    );
    mn_802308F0(
        arg0,
        update_selection as ::core::ffi::c_int,
        update_value as ::core::ffi::c_int,
    );
    if update_menu != 0 as ::core::ffi::c_int {
        (*data).x0 = mn_804A04F0.cur_menu as u8_0;
    }
    if update_selection != 0 as ::core::ffi::c_int {
        (*data).x1 = mn_804A04F0.hovered_selection as u8_0;
    }
    if update_value != 0 as ::core::ffi::c_int {
        if (*data).x1 as ::core::ffi::c_int == 1 as ::core::ffi::c_int
            && (*data).x2 as ::core::ffi::c_int == 1 as ::core::ffi::c_int
        {
            (*data).x9 = mn_804A04F0.confirmed_selection;
        } else {
            state = mn_804A04F0.confirmed_selection;
            ptr = data as *mut u8_0;
            ptr = ptr.offset((*data).x1 as ::core::ffi::c_int as isize);
            *ptr.offset(2 as ::core::ffi::c_int as isize) = state;
        }
        data = (*arg0).user_data as *mut mn_802307F8_t;
        rules = gmMainLib_GetGameRules();
        (*rules).mode = (*data).x2;
        (*rules).time_limit = (*data).x3;
        (*rules).handicap = (*data).x4;
        (*rules).damage_ratio = (*data).x5;
        (*rules).stage_sel = (*data).x6;
        (*rules).stock_count = (*data).x9;
    }
}
pub unsafe extern "C" fn mn_80230D18(
    mut arg0: *mut mn_802307F8_t,
    mut arg1: *mut HSD_JObj,
    mut arg2: ::core::ffi::c_int,
) -> s32 {
    melee_core::ct!("mn_80230D18");
    let mut rules: *mut GameRules = ::core::ptr::null_mut::<GameRules>();
    let mut ret: s32 = 0;
    let mut i: s32 = 0;
    (*arg0).x0 = mn_804A04F0.cur_menu as u8_0;
    (*arg0).x1 = mn_804A04F0.hovered_selection as u8_0;
    rules = gmMainLib_GetGameRules();
    (*arg0).x4 = (*rules).handicap;
    if mn_804D6BD4 == 0 as ::core::ffi::c_int
        && (*arg0).x4 as ::core::ffi::c_int == 1 as ::core::ffi::c_int
    {
        (*arg0).x4 = 0 as u8_0;
    }
    (*arg0).x5 = (*gmMainLib_GetGameRules()).damage_ratio;
    (*arg0).x6 = (*gmMainLib_GetGameRules()).stage_sel;
    (*arg0).x2 = (*gmMainLib_GetGameRules()).mode;
    (*arg0).x9 = (*gmMainLib_GetGameRules()).stock_count;
    (*arg0).x3 = (*gmMainLib_GetGameRules()).time_limit;
    if gm_GetCurrentGameMode() as ::core::ffi::c_int
        == GM_TOURNAMENT as ::core::ffi::c_int
        && (*arg0).x3 as ::core::ffi::c_int == 0 as ::core::ffi::c_int
    {
        (*arg0).x3 = 0x63 as u8_0;
    }
    (*arg0).xA = arg2 as u8_0;
    (*arg0).text = ::core::ptr::null_mut::<HSD_Text>();
    ret = 0 as ::core::ffi::c_int as s32;
    i = 0 as ::core::ffi::c_int as s32;
    while i < 10 as ::core::ffi::c_int {
        ret = lb_80011E24(arg1 as *mut HSD_JObj, (&raw mut (*arg0).c2rust_unnamed.xC as *mut *mut HSD_JObj).offset(i as isize)
                as *mut *mut HSD_JObj, &[(i) as usize, (-(1 as ::core::ffi::c_int)) as usize]) as s32;
        i += 1;
    }
    return ret;
}
#[inline]
pub unsafe extern "C" fn mn_80230E38_CountVisible(mut limit: u8_0) -> s32 {
    melee_core::ct!("mn_80230E38_CountVisible");
    let mut option_index: u8_0 = 0;
    let mut visible: s32 = 0;
    let mut i: s32 = 0;
    let mut count: s32 = 0 as s32;
    i = 0 as ::core::ffi::c_int as s32;
    while i < limit as s32 {
        option_index = i as u8_0;
        if gm_GetCurrentGameMode() as ::core::ffi::c_int
            == GM_TOURNAMENT as ::core::ffi::c_int
            && option_index as ::core::ffi::c_int == 4 as ::core::ffi::c_int
        {
            visible = 0 as ::core::ffi::c_int as s32;
        } else {
            visible = 1 as ::core::ffi::c_int as s32;
        }
        if visible != 0 as ::core::ffi::c_int {
            count += 1;
        }
        i += 1;
    }
    return count;
}
pub unsafe extern "C" fn mn_80230E38(mut arg0: ::core::ffi::c_int) -> *mut HSD_GObj {
    melee_core::ct!("mn_80230E38");
    let mut operand_pad: [u8_0; 12] = [0; 12];
    let mut jobj_map: [u16_0; 17] = [0; 17];
    let mut jobj_parts: [*mut HSD_JObj; 17] = [::core::ptr::null_mut::<HSD_JObj>(); 17];
    let mut gobj: *mut HSD_GObj = ::core::ptr::null_mut::<HSD_GObj>();
    let mut user_data: *mut mn_802307F8_t = ::core::ptr::null_mut::<mn_802307F8_t>();
    let mut num_options: u8_0 = 0;
    let mut selected: u8_0 = 0;
    let mut i: s32 = 0;
    let mut j: s32 = 0;
    let mut vis_before: s32 = 0;
    let mut vis_total: s32 = 0;
    let mut visible: s32 = 0;
    let mut als: *mut AnimLoopSettings = ::core::ptr::null_mut::<AnimLoopSettings>();
    let mut option_jobj: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut cursor_jobj: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut value_jobj: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut desc: *mut StaticModelDesc = ::core::ptr::null_mut::<StaticModelDesc>();
    let mut root_jobj: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut sub_count_ptr: *mut u16_0 = ::core::ptr::null_mut::<u16_0>();
    let mut _pad: [::core::ffi::c_uchar; 4] = [0; 4];
    selected = mn_804A04F0.hovered_selection as u8_0;
    num_options = (*(&raw mut mn_803EB6B0 as *mut MenuKindData)
        .offset(13 as ::core::ffi::c_int as isize))
        .selection_count;
    i = 0 as ::core::ffi::c_int as s32;
    while 17 as ::core::ffi::c_int > i {
        jobj_map[i as usize] = i as u16_0;
        i += 1;
    }
    desc = &raw mut MenMainConRl_Top;
    gobj = GObj_Create(6 as u16_0, 7 as u8_0, 0x80 as u8_0);
    mn_804D6BD0 = gobj;
    root_jobj = HSD_JObjLoadJoint((*desc).joint as *mut HSD_Joint);
    HSD_GObjObject_80390A70(
        gobj,
        HSD_GObj_JObjKind,
        root_jobj as *mut ::core::ffi::c_void,
    );
    GObj_SetupGXLink(
        gobj,
        Some(
            HSD_GObj_JObjCallback
                as unsafe extern "C" fn(*mut HSD_GObj, ::core::ffi::c_int) -> (),
        ),
        4 as u8_0,
        0x80 as u32_0,
    );
    HSD_GObj_SetupProc(
        gobj,
        Some(fn_802309F0 as unsafe extern "C" fn(*mut HSD_GObj) -> ()),
        0 as u8_0,
    );
    HSD_JObjAddAnimAll(
        root_jobj,
        (*desc).animjoint,
        (*desc).matanim_joint,
        (*desc).shapeanim_joint,
    );
    HSD_JObjReqAnimAll(root_jobj, 0.0f32);
    HSD_JObjAnimAll(root_jobj);
    user_data = HSD_MemAlloc(::core::mem::size_of::<mn_802307F8_t>() as ssize_t)
        as *mut mn_802307F8_t;
    if !user_data.is_null() {} else {
        OSReport(b"Can't get user_data.\n\0" as *const u8 as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char, &[]);
        __assert(
            b"src/melee/mn/mnmainrule.c\0" as *const u8 as *const ::core::ffi::c_char,
            1087 as u32_0,
            b"user_data\0" as *const u8 as *const ::core::ffi::c_char,
        );
    };
    GObj_InitUserData(
        gobj,
        0 as u8_0,
        Some(HSD_Free as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()),
        user_data as *mut ::core::ffi::c_void,
    );
    mn_80230D18(user_data, root_jobj, arg0);
    if (*user_data).x1 as ::core::ffi::c_int == 1 as ::core::ffi::c_int
        && (*((*mn_804D6BD0).user_data as *mut mn_802307F8_t)).x2 as ::core::ffi::c_int
            == 1 as ::core::ffi::c_int
    {
        mn_804A04F0.confirmed_selection = (*user_data).x9;
    } else {
        mn_804A04F0.confirmed_selection = *(&raw mut (*user_data).x2)
            .offset((*user_data).x1 as ::core::ffi::c_int as isize);
    }
    if arg0 as u8_0 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        let mut anim_jobj: *mut HSD_JObj = (*user_data)
            .c2rust_unnamed
            .xC[2 as ::core::ffi::c_int as usize];
        match (*user_data).xA as s32 {
            1 => {
                als = (&raw mut mn_803EC770 as *mut AnimLoopSettings)
                    .offset(5 as ::core::ffi::c_int as isize) as *mut AnimLoopSettings;
            }
            3 => {
                als = (&raw mut mn_803EC770 as *mut AnimLoopSettings)
                    .offset(6 as ::core::ffi::c_int as isize) as *mut AnimLoopSettings;
            }
            2 | _ => {}
        }
        HSD_JObjReqAnim(anim_jobj, (*als).start_frame);
        HSD_JObjAnim(anim_jobj);
    }
    sub_count_ptr = &raw mut mn_803EC808 as *mut u16_0;
    i = 0 as ::core::ffi::c_int as s32;
    while i < num_options as s32 {
        vis_before = mn_80230E38_CountVisible(i as u8_0);
        option_jobj = (*user_data)
            .c2rust_unnamed
            .xC[mn_803EC600[vis_before as u8_0 as usize] as usize] as *mut HSD_JObj;
        vis_total = mn_80230E38_CountVisible(7 as u8_0);
        HSD_JObjReqAnim(option_jobj, vis_total as f32_0);
        HSD_JObjAnim(option_jobj);
        if gm_GetCurrentGameMode() as ::core::ffi::c_int
            == GM_TOURNAMENT as ::core::ffi::c_int
            && i as u8_0 as ::core::ffi::c_int == 4 as ::core::ffi::c_int
        {
            visible = 0 as ::core::ffi::c_int as s32;
        } else {
            visible = 1 as ::core::ffi::c_int as s32;
        }
        if visible != 0 as ::core::ffi::c_int {
            desc = &raw mut MenMainCursorRl_Top;
            cursor_jobj = HSD_JObjLoadJoint((*desc).joint as *mut HSD_Joint);
            HSD_JObjAddAnimAll(
                cursor_jobj,
                (*desc).animjoint,
                (*desc).matanim_joint,
                (*desc).shapeanim_joint,
            );
            HSD_JObjReqAnimAll(cursor_jobj, 0.0f32);
            HSD_JObjAnimAll(cursor_jobj);
            lb_8001204C(
                cursor_jobj as *mut HSD_JObj,
                &raw mut jobj_parts as *mut *mut HSD_JObj,
                &raw mut jobj_map as *mut u16_0,
                17 as ::core::ffi::c_int,
            );
            if i == 1 as ::core::ffi::c_int
                && (*((*mn_804D6BD0).user_data as *mut mn_802307F8_t)).x2
                    as ::core::ffi::c_int == 1 as ::core::ffi::c_int
            {
                HSD_JObjReqAnim(
                    jobj_parts[7 as ::core::ffi::c_int as usize],
                    mn_804D4B88[(selected as ::core::ffi::c_int == i)
                        as ::core::ffi::c_int as usize],
                );
            } else {
                HSD_JObjReqAnim(
                    jobj_parts[7 as ::core::ffi::c_int as usize],
                    rule_label_frames[i
                        as usize][(selected as ::core::ffi::c_int == i)
                        as ::core::ffi::c_int as usize],
                );
            }
            HSD_JObjAnim(jobj_parts[7 as ::core::ffi::c_int as usize]);
            if selected as s32 != i {
                HSD_JObjSetFlagsAll(
                    jobj_parts[16 as ::core::ffi::c_int as usize],
                    JOBJ_HIDDEN as u32_0,
                );
            }
            if selected as s32 != i
                || (i as ::core::ffi::c_int - 5 as ::core::ffi::c_int) as u32_0
                    <= 1 as ::core::ffi::c_uint
            {
                HSD_JObjSetFlagsAll(
                    jobj_parts[13 as ::core::ffi::c_int as usize],
                    JOBJ_HIDDEN as u32_0,
                );
            }
            if (i as ::core::ffi::c_int - 5 as ::core::ffi::c_int) as u32_0
                <= 1 as ::core::ffi::c_uint
            {
                als = (&raw mut submenu_anims as *mut AnimLoopSettings)
                    .offset(
                        (i == selected as ::core::ffi::c_int) as ::core::ffi::c_int
                            as isize,
                    ) as *mut AnimLoopSettings;
            } else {
                als = (&raw mut option_anims as *mut AnimLoopSettings)
                    .offset(
                        (i == selected as ::core::ffi::c_int) as ::core::ffi::c_int
                            as isize,
                    ) as *mut AnimLoopSettings;
            }
            HSD_JObjReqAnimAll(
                jobj_parts[2 as ::core::ffi::c_int as usize],
                (*als).start_frame,
            );
            HSD_JObjAnimAll(jobj_parts[2 as ::core::ffi::c_int as usize]);
            if selected as s32 != i
                || i != 5 as ::core::ffi::c_int && i != 6 as ::core::ffi::c_int
            {
                HSD_JObjSetFlagsAll(
                    jobj_parts[8 as ::core::ffi::c_int as usize],
                    JOBJ_HIDDEN as u32_0,
                );
            }
            HSD_JObjAddChild(option_jobj, cursor_jobj);
            if i != 5 as ::core::ffi::c_int && i != 6 as ::core::ffi::c_int {
                desc = mn_803EC7EC[i as usize];
                value_jobj = HSD_JObjLoadJoint((*desc).joint as *mut HSD_Joint);
                HSD_JObjAddAnimAll(
                    value_jobj,
                    (*desc).animjoint,
                    (*desc).matanim_joint,
                    (*desc).shapeanim_joint,
                );
                HSD_JObjReqAnimAll(value_jobj, 0.0f32);
                HSD_JObjAnimAll(value_jobj);
                j = 0 as ::core::ffi::c_int as s32;
                while j < *sub_count_ptr.offset(i as isize) as ::core::ffi::c_int {
                    lb_80011E24(value_jobj as *mut HSD_JObj, (&raw mut (*(&raw mut (*user_data)
                            .c2rust_unnamed
                            .c2rust_unnamed
                            .x34 as *mut C2RustUnnamed_3)
                            .offset(i as isize))
                            .joints as *mut *mut HSD_JObj)
                            .offset(j as isize) as *mut *mut HSD_JObj, &[(j) as usize, (-(1 as ::core::ffi::c_int)) as usize]);
                    j += 1;
                }
                match i {
                    1 => {
                        let mut time_indices: JObjIndices6 = JObjIndices6 {
                            idx: [0; 6],
                        };
                        let mut index_ptr: *mut u8_0 = ::core::ptr::null_mut::<u8_0>();
                        j = 0 as ::core::ffi::c_int as s32;
                        index_ptr = (&raw mut time_indices.idx as *mut u8_0)
                            .offset(j as isize) as *mut u8_0;
                        time_indices = time_digit_indices;
                        while j < 6 as ::core::ffi::c_int {
                            let mut text: *mut HSD_JObj = HSD_JObjLoadJoint(
                                MenMainNmRl_Top.joint as *mut HSD_Joint,
                            );
                            HSD_JObjAddAnimAll(
                                text,
                                MenMainNmRl_Top.animjoint,
                                MenMainNmRl_Top.matanim_joint,
                                MenMainNmRl_Top.shapeanim_joint,
                            );
                            HSD_JObjAddChild(
                                (*user_data)
                                    .c2rust_unnamed
                                    .c2rust_unnamed
                                    .x34[1 as ::core::ffi::c_int as usize]
                                    .joints[*index_ptr as usize],
                                text,
                            );
                            j += 1;
                            index_ptr = index_ptr.offset(1);
                        }
                        mn_8022FD18(
                            ((*((*mn_804D6BD0).user_data as *mut mn_802307F8_t)).x2
                                as ::core::ffi::c_int != 1 as ::core::ffi::c_int)
                                as ::core::ffi::c_int as u8_0,
                        );
                    }
                    3 => {
                        let mut damage_indices: JObjIndices2 = JObjIndices2 {
                            idx: [0; 2],
                        };
                        let mut index_ptr_0: *mut u8_0 = ::core::ptr::null_mut::<u8_0>();
                        let mut value: u8_0 = (*user_data).x5;
                        let mut digit_jobj: *mut HSD_JObj = ::core::ptr::null_mut::<
                            HSD_JObj,
                        >();
                        let mut __0: [::core::ffi::c_uchar; 36] = [0; 36];
                        damage_indices = mn_804DBDF8;
                        index_ptr_0 = &raw mut damage_indices.idx as *mut u8_0;
                        j = 0 as ::core::ffi::c_int as s32;
                        while j < 2 as ::core::ffi::c_int {
                            let mut text_0: *mut HSD_JObj = HSD_JObjLoadJoint(
                                MenMainNmRl_Top.joint as *mut HSD_Joint,
                            );
                            HSD_JObjAddAnimAll(
                                text_0,
                                MenMainNmRl_Top.animjoint,
                                MenMainNmRl_Top.matanim_joint,
                                MenMainNmRl_Top.shapeanim_joint,
                            );
                            HSD_JObjAddChild(
                                (*user_data)
                                    .c2rust_unnamed
                                    .c2rust_unnamed
                                    .x34[3 as ::core::ffi::c_int as usize]
                                    .joints[*index_ptr_0 as usize],
                                text_0,
                            );
                            j += 1;
                            index_ptr_0 = index_ptr_0.offset(1);
                        }
                        digit_jobj = mn_80231634(
                            (*user_data)
                                .c2rust_unnamed
                                .c2rust_unnamed
                                .x34[3 as ::core::ffi::c_int as usize]
                                .joints[2 as ::core::ffi::c_int as usize]
                                as *mut mn_80231634_t,
                        ) as *mut HSD_JObj;
                        HSD_JObjReqAnimAll(
                            digit_jobj,
                            (value as ::core::ffi::c_int / 10 as ::core::ffi::c_int)
                                as u8_0 as f32_0,
                        );
                        HSD_JObjAnimAll(digit_jobj);
                        digit_jobj = mn_80231634(
                            (*user_data)
                                .c2rust_unnamed
                                .c2rust_unnamed
                                .x34[3 as ::core::ffi::c_int as usize]
                                .joints[3 as ::core::ffi::c_int as usize]
                                as *mut mn_80231634_t,
                        ) as *mut HSD_JObj;
                        HSD_JObjReqAnimAll(
                            digit_jobj,
                            (value as ::core::ffi::c_int % 10 as ::core::ffi::c_int)
                                as u8_0 as f32_0,
                        );
                        HSD_JObjAnimAll(digit_jobj);
                    }
                    0 | 2 | 4 => {
                        let mut value_0: u8_0 = *(&raw mut (*user_data).x2)
                            .offset(i as isize);
                        let mut default_value: u8_0 = 0;
                        let mut value_als: *mut AnimLoopSettings = ::core::ptr::null_mut::<
                            AnimLoopSettings,
                        >();
                        if i as u8_0 as ::core::ffi::c_int != 0 as ::core::ffi::c_int
                            && i as u8_0 as ::core::ffi::c_int != 2 as ::core::ffi::c_int
                            && i as u8_0 as ::core::ffi::c_int != 4 as ::core::ffi::c_int
                        {
                            value_als = ::core::ptr::null_mut::<AnimLoopSettings>();
                        } else {
                            default_value = mn_803EC7DC[i as u8_0
                                as usize][1 as ::core::ffi::c_int as usize];
                            if value_0 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
                                value_als = (&raw mut mn_803EC734 as *mut AnimLoopSettings)
                                    .offset(default_value as isize) as *mut AnimLoopSettings;
                            } else {
                                value_als = (&raw mut mn_803EC734 as *mut AnimLoopSettings)
                                    .offset(
                                        (value_0 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                                            as isize,
                                    ) as *mut AnimLoopSettings;
                            }
                        }
                        HSD_JObjReqAnimAll(
                            (*user_data)
                                .c2rust_unnamed
                                .c2rust_unnamed
                                .x34[i as usize]
                                .joints[0 as ::core::ffi::c_int as usize],
                            (*value_als).end_frame,
                        );
                        HSD_JObjAnimAll(
                            (*user_data)
                                .c2rust_unnamed
                                .c2rust_unnamed
                                .x34[i as usize]
                                .joints[0 as ::core::ffi::c_int as usize],
                        );
                    }
                    _ => {}
                }
                HSD_JObjAddChild(option_jobj, value_jobj);
            }
        }
        i += 1;
    }
    mn_802307F8(
        user_data,
        (*user_data).x1 as s32,
        mn_804A04F0.confirmed_selection as s32,
    );
    return gobj as *mut HSD_GObj;
}
pub unsafe extern "C" fn mn_80231634(
    mut arg0: *mut mn_80231634_t,
) -> ::core::ffi::c_int {
    melee_core::ct!("mn_80231634");
    if arg0.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    return (*arg0).x10;
}
pub unsafe extern "C" fn mn_8023164C() {
    melee_core::ct!("mn_8023164C");
    let mut proc_0: *mut HSD_GObjProc = ::core::ptr::null_mut::<HSD_GObjProc>();
    let mut mode: s32 = 0;
    mn_804A04F0.prev_menu = mn_804A04F0.cur_menu;
    mn_804A04F0.cur_menu = 0xd as MenuKind8;
    match mn_804A04F0.prev_menu as ::core::ffi::c_int {
        15 => {
            mn_804A04F0.hovered_selection = 6 as u16_0;
            mode = 3 as ::core::ffi::c_int as s32;
        }
        16 => {
            mn_804A04F0.hovered_selection = 5 as u16_0;
            mode = 3 as ::core::ffi::c_int as s32;
        }
        _ => {
            mn_804A04F0.hovered_selection = 0 as u16_0;
            mode = 1 as ::core::ffi::c_int as s32;
        }
    }
    proc_0 = HSD_GObj_SetupProc(
        GObj_Create(0 as u16_0, 1 as u8_0, 0x80 as u8_0),
        Some(fn_8022F538 as unsafe extern "C" fn(*mut HSD_GObj) -> ()),
        0 as u8_0,
    );
    (*proc_0).set_flags_3(HSD_GObj_804D783C as u8_0 as u8_0);
    HSD_GObj_80390CD4(mn_80230E38(mode as ::core::ffi::c_int) as *mut HSD_GObj);
}
pub unsafe extern "C" fn mn_80231714() {
    melee_core::ct!("mn_80231714");
    let mut proc_0: *mut HSD_GObjProc = ::core::ptr::null_mut::<HSD_GObjProc>();
    let mut mode: s32 = 0;
    mn_804A04F0.prev_menu = mn_804A04F0.cur_menu;
    mn_804A04F0.cur_menu = 0xd as MenuKind8;
    match mn_804A04F0.prev_menu as ::core::ffi::c_int {
        15 => {
            mn_804A04F0.hovered_selection = 6 as u16_0;
            mode = 3 as ::core::ffi::c_int as s32;
        }
        16 => {
            mn_804A04F0.hovered_selection = 5 as u16_0;
            mode = 3 as ::core::ffi::c_int as s32;
        }
        _ => {
            mn_804A04F0.hovered_selection = 0 as u16_0;
            mode = 1 as ::core::ffi::c_int as s32;
        }
    }
    proc_0 = HSD_GObj_SetupProc(
        GObj_Create(0 as u16_0, 1 as u8_0, 0x80 as u8_0),
        Some(fn_8022F538 as unsafe extern "C" fn(*mut HSD_GObj) -> ()),
        0 as u8_0,
    );
    (*proc_0).set_flags_3(HSD_GObj_804D783C as u8_0 as u8_0);
    HSD_GObj_80390CD4(mn_80230E38(mode as ::core::ffi::c_int) as *mut HSD_GObj);
    mn_804D6BD4 = 1 as ::core::ffi::c_int as s32;
}
pub unsafe extern "C" fn mn_802317E4(
    mut archive: *mut HSD_Archive,
    mut arg1: ::core::ffi::c_int,
) {
    melee_core::ct!("mn_802317E4");
    mn_80231804(archive, arg1);
}
pub unsafe extern "C" fn mn_80231804(
    mut archive: *mut HSD_Archive,
    mut arg1: ::core::ffi::c_int,
) {
    melee_core::ct!("mn_80231804");
    let mut temp_r3: *mut HSD_GObjProc = ::core::ptr::null_mut::<HSD_GObjProc>();
    let mut _pad: [::core::ffi::c_uchar; 4] = [0; 4];
    mn_804D6BC8.cooldown = 0x14 as u16_0;
    mn_804D6BC8.x2 = 0 as u16_0;
    mn_804D6BC8.x4 = 0 as ::core::ffi::c_int as s32;
    mn_804A04F0.buttons = 0 as u64_0;
    mn_804A04F0.prev_menu = mn_804A04F0.cur_menu;
    mn_804A04F0.cur_menu = 0xd as MenuKind8;
    mn_804A04F0.hovered_selection = 0 as u16_0;
    HSD_SisLib_803A5E70();
    lbArchive_LoadSections(archive, &raw mut MenMainBack_Top.joint as *mut *mut ::core::ffi::c_void, &[(b"MenMainBack_Top_joint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut MenMainBack_Top.animjoint) as usize, (b"MenMainBack_Top_animjoint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut MenMainBack_Top.matanim_joint) as usize, (b"MenMainBack_Top_matanim_joint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut MenMainBack_Top.shapeanim_joint) as usize, (b"MenMainBack_Top_shapeanim_joint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut MenMain_cam) as usize, (b"ScMenMain_cam_int1_camera\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut MenMain_lights) as usize, (b"ScMenMain_scene_lights\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut MenMain_fog) as usize, (b"ScMenMain_fog\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut MenMainPanel_Top.joint) as usize, (b"MenMainPanel_Top_joint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut MenMainPanel_Top.animjoint) as usize, (b"MenMainPanel_Top_animjoint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut MenMainPanel_Top.matanim_joint) as usize, (b"MenMainPanel_Top_matanim_joint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut MenMainPanel_Top.shapeanim_joint) as usize, (b"MenMainPanel_Top_shapeanim_joint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut MenMainConRl_Top.joint) as usize, (b"MenMainConRl_Top_joint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut MenMainConRl_Top.animjoint) as usize, (b"MenMainConRl_Top_animjoint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut MenMainConRl_Top.matanim_joint) as usize, (b"MenMainConRl_Top_matanim_joint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut MenMainConRl_Top.shapeanim_joint) as usize, (b"MenMainConRl_Top_shapeanim_joint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorRl_Top.joint) as usize, (b"MenMainCursorRl_Top_joint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorRl_Top.animjoint) as usize, (b"MenMainCursorRl_Top_animjoint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorRl_Top.matanim_joint) as usize, (b"MenMainCursorRl_Top_matanim_joint\0" as *const u8
            as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorRl_Top.shapeanim_joint) as usize, (b"MenMainCursorRl_Top_shapeanim_joint\0" as *const u8
            as *const ::core::ffi::c_char) as usize, (&raw mut MenMainNmRl_Top.joint) as usize, (b"MenMainNmRl_Top_joint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut MenMainNmRl_Top.animjoint) as usize, (b"MenMainNmRl_Top_animjoint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut MenMainNmRl_Top.matanim_joint) as usize, (b"MenMainNmRl_Top_matanim_joint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut MenMainNmRl_Top.shapeanim_joint) as usize, (b"MenMainNmRl_Top_shapeanim_joint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorTr01_Top.joint) as usize, (b"MenMainCursorTr01_Top_joint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorTr01_Top.animjoint) as usize, (b"MenMainCursorTr01_Top_animjoint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorTr01_Top.matanim_joint) as usize, (b"MenMainCursorTr01_Top_matanim_joint\0" as *const u8
            as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorTr01_Top.shapeanim_joint) as usize, (b"MenMainCursorTr01_Top_shapeanim_joint\0" as *const u8
            as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorTr02_Top.joint) as usize, (b"MenMainCursorTr02_Top_joint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorTr02_Top.animjoint) as usize, (b"MenMainCursorTr02_Top_animjoint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorTr02_Top.matanim_joint) as usize, (b"MenMainCursorTr02_Top_matanim_joint\0" as *const u8
            as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorTr02_Top.shapeanim_joint) as usize, (b"MenMainCursorTr02_Top_shapeanim_joint\0" as *const u8
            as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorTr03_Top.joint) as usize, (b"MenMainCursorTr03_Top_joint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorTr03_Top.animjoint) as usize, (b"MenMainCursorTr03_Top_animjoint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorTr03_Top.matanim_joint) as usize, (b"MenMainCursorTr03_Top_matanim_joint\0" as *const u8
            as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorTr03_Top.shapeanim_joint) as usize, (b"MenMainCursorTr03_Top_shapeanim_joint\0" as *const u8
            as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorTr04_Top.joint) as usize, (b"MenMainCursorTr04_Top_joint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorTr04_Top.animjoint) as usize, (b"MenMainCursorTr04_Top_animjoint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorTr04_Top.matanim_joint) as usize, (b"MenMainCursorTr04_Top_matanim_joint\0" as *const u8
            as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorTr04_Top.shapeanim_joint) as usize, (b"MenMainCursorTr04_Top_shapeanim_joint\0" as *const u8
            as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorRl01_Top.joint) as usize, (b"MenMainCursorRl01_Top_joint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorRl01_Top.animjoint) as usize, (b"MenMainCursorRl01_Top_animjoint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorRl01_Top.matanim_joint) as usize, (b"MenMainCursorRl01_Top_matanim_joint\0" as *const u8
            as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorRl01_Top.shapeanim_joint) as usize, (b"MenMainCursorRl01_Top_shapeanim_joint\0" as *const u8
            as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorRl02_Top.joint) as usize, (b"MenMainCursorRl02_Top_joint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorRl02_Top.animjoint) as usize, (b"MenMainCursorRl02_Top_animjoint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorRl02_Top.matanim_joint) as usize, (b"MenMainCursorRl02_Top_matanim_joint\0" as *const u8
            as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorRl02_Top.shapeanim_joint) as usize, (b"MenMainCursorRl02_Top_shapeanim_joint\0" as *const u8
            as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorRl03_Top.joint) as usize, (b"MenMainCursorRl03_Top_joint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorRl03_Top.animjoint) as usize, (b"MenMainCursorRl03_Top_animjoint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorRl03_Top.matanim_joint) as usize, (b"MenMainCursorRl03_Top_matanim_joint\0" as *const u8
            as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorRl03_Top.shapeanim_joint) as usize, (b"MenMainCursorRl03_Top_shapeanim_joint\0" as *const u8
            as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorRl04_Top.joint) as usize, (b"MenMainCursorRl04_Top_joint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorRl04_Top.animjoint) as usize, (b"MenMainCursorRl04_Top_animjoint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorRl04_Top.matanim_joint) as usize, (b"MenMainCursorRl04_Top_matanim_joint\0" as *const u8
            as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorRl04_Top.shapeanim_joint) as usize, (b"MenMainCursorRl04_Top_shapeanim_joint\0" as *const u8
            as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorRl05_Top.joint) as usize, (b"MenMainCursorRl05_Top_joint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorRl05_Top.animjoint) as usize, (b"MenMainCursorRl05_Top_animjoint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorRl05_Top.matanim_joint) as usize, (b"MenMainCursorRl05_Top_matanim_joint\0" as *const u8
            as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorRl05_Top.shapeanim_joint) as usize, (b"MenMainCursorRl05_Top_shapeanim_joint\0" as *const u8
            as *const ::core::ffi::c_char) as usize, (&raw mut MenMainConIs_Top.joint) as usize, (b"MenMainConIs_Top_joint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut MenMainConIs_Top.animjoint) as usize, (b"MenMainConIs_Top_animjoint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut MenMainConIs_Top.matanim_joint) as usize, (b"MenMainConIs_Top_matanim_joint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut MenMainConIs_Top.shapeanim_joint) as usize, (b"MenMainConIs_Top_shapeanim_joint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorIs_Top.joint) as usize, (b"MenMainCursorIs_Top_joint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorIs_Top.animjoint) as usize, (b"MenMainCursorIs_Top_animjoint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorIs_Top.matanim_joint) as usize, (b"MenMainCursorIs_Top_matanim_joint\0" as *const u8
            as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorIs_Top.shapeanim_joint) as usize, (b"MenMainCursorIs_Top_shapeanim_joint\0" as *const u8
            as *const ::core::ffi::c_char) as usize, (&raw mut MenMainConSs_Top.joint) as usize, (b"MenMainConSs_Top_joint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut MenMainConSs_Top.animjoint) as usize, (b"MenMainConSs_Top_animjoint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut MenMainConSs_Top.matanim_joint) as usize, (b"MenMainConSs_Top_matanim_joint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut MenMainConSs_Top.shapeanim_joint) as usize, (b"MenMainConSs_Top_shapeanim_joint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorSs_Top.joint) as usize, (b"MenMainCursorSs_Top_joint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorSs_Top.animjoint) as usize, (b"MenMainCursorSs_Top_animjoint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorSs_Top.matanim_joint) as usize, (b"MenMainCursorSs_Top_matanim_joint\0" as *const u8
            as *const ::core::ffi::c_char) as usize, (&raw mut MenMainCursorSs_Top.shapeanim_joint) as usize, (b"MenMainCursorSs_Top_shapeanim_joint\0" as *const u8
            as *const ::core::ffi::c_char) as usize, (0 as ::core::ffi::c_int) as usize]);
    mn_804D6BD4 = arg1 as s32;
    mn_8022C304();
    mn_8022BCF8();
    mn_8022BEDC(mn_8022BE34());
    mn_80229B2C();
    mn_80229DC0();
    temp_r3 = HSD_GObj_SetupProc(
        GObj_Create(0 as u16_0, 1 as u8_0, 0x80 as u8_0),
        Some(fn_8022F538 as unsafe extern "C" fn(*mut HSD_GObj) -> ()),
        0 as u8_0,
    );
    (*temp_r3).set_flags_3(HSD_GObj_804D783C as u8_0 as u8_0);
    HSD_GObj_80390CD4(mn_80230E38(1 as ::core::ffi::c_int) as *mut HSD_GObj);
    lbAudioAx_80023F28(gmMainLib_8015ECB0());
}
pub unsafe extern "C" fn mn_80231F80(mut arg0: u8_0) -> bool_0 {
    melee_core::ct!("mn_80231F80");
    if gm_GetCurrentGameMode() as ::core::ffi::c_int
        == GM_TOURNAMENT as ::core::ffi::c_int
        && arg0 as ::core::ffi::c_int == 1 as ::core::ffi::c_int
    {
        return false_0;
    }
    if arg0 as ::core::ffi::c_int == 3 as ::core::ffi::c_int {
        if gmMainLib_8015EE0C() != 0 {
            return true_0;
        }
        return false_0;
    }
    if arg0 as ::core::ffi::c_int == 5 as ::core::ffi::c_int {
        if gmMainLib_8015EE44() != 0 {
            return true_0;
        }
        return false_0;
    }
    return true_0;
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
                    None => melee_core::unported!(concat!("TODO: ", stringify!($name), " is not ported")),
                }
            }
        };
    }
    hook!(H_OSReport, OSReport, (a0: *mut ::core::ffi::c_char, va: &[usize]));
    hook!(H___assert, __assert, (a0: *const ::core::ffi::c_char, a1: u32_0, a2: *const ::core::ffi::c_char) -> !);
    hook!(H_GObj_Create, GObj_Create, (a0: u16_0, a1: u8_0, a2: u8_0) -> *mut HSD_GObj);
    pub static mut H_HSD_GObj_JObjCallback: Option<unsafe extern "C" fn(*mut HSD_GObj, ::core::ffi::c_int) > = None;
            pub unsafe extern "C" fn HSD_GObj_JObjCallback(a0: *mut HSD_GObj, a1: ::core::ffi::c_int)  {
                match H_HSD_GObj_JObjCallback {
                    Some(f) => f(a0, a1),
                    None => { unsafe { melee_runtime::tu::gobj::HSD_GObj_JObjCallback(a0 as _, a1 as _); } return; },
                }
            }
    hook!(H_HSD_GObj_80390CD4, HSD_GObj_80390CD4, (a0: *mut HSD_GObj));
    pub static mut H_lbAudioAx_80023F28: Option<unsafe extern "C" fn(::core::ffi::c_int) -> ::core::ffi::c_int> = None;
            pub unsafe extern "C" fn lbAudioAx_80023F28(a0: ::core::ffi::c_int) -> ::core::ffi::c_int {
                match H_lbAudioAx_80023F28 {
                    Some(f) => f(a0),
                    None => { return unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80023F28(a0 as _) as _ }; },
                }
            }
    pub static mut H_lbAudioAx_80024030: Option<unsafe extern "C" fn(::core::ffi::c_int) > = None;
            pub unsafe extern "C" fn lbAudioAx_80024030(a0: ::core::ffi::c_int)  {
                match H_lbAudioAx_80024030 {
                    Some(f) => f(a0),
                    None => { unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80024030(a0 as _); } return; },
                }
            }
    hook!(H_lb_80011E24, lb_80011E24, (a0: *mut HSD_JObj, a1: *mut *mut HSD_JObj, va: &[usize]) -> ::core::ffi::c_int);
    pub static mut H_lb_8001204C: Option<unsafe extern "C" fn(*mut HSD_JObj, *mut *mut HSD_JObj, *mut u16_0, ::core::ffi::c_int) -> ::core::ffi::c_int> = None;
            pub unsafe extern "C" fn lb_8001204C(a0: *mut HSD_JObj, a1: *mut *mut HSD_JObj, a2: *mut u16_0, a3: ::core::ffi::c_int) -> ::core::ffi::c_int {
                match H_lb_8001204C {
                    Some(f) => f(a0, a1, a2, a3),
                    None => { return unsafe { melee_runtime::tu::lbspdisplay::lb_8001204C(a0 as _, a1 as _, a2 as _, a3 as _) as _ }; },
                }
            }
    pub static mut H_mn_8022ED6C: Option<unsafe extern "C" fn(*mut HSD_JObj, *mut AnimLoopSettings) -> ::core::ffi::c_float> = None;
            pub unsafe extern "C" fn mn_8022ED6C(a0: *mut HSD_JObj, a1: *mut AnimLoopSettings) -> ::core::ffi::c_float {
                match H_mn_8022ED6C {
                    Some(f) => f(a0, a1),
                    None => { return unsafe { crate::tu::mn_22EC::mn_8022ED6C(a0 as _, a1 as _) as _ }; },
                }
            }
    pub static mut H_mn_8022F298: Option<unsafe extern "C" fn(*mut HSD_JObj) -> f32_0> = None;
            pub unsafe extern "C" fn mn_8022F298(a0: *mut HSD_JObj) -> f32_0 {
                match H_mn_8022F298 {
                    Some(f) => f(a0),
                    None => { return unsafe { crate::tu::mn_22EC::mn_8022F298(a0 as _) as _ }; },
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
    hook!(H_HSD_JObjSetFlagsAll, HSD_JObjSetFlagsAll, (a0: *mut HSD_JObj, a1: u32_0));
    hook!(H_HSD_JObjClearFlagsAll, HSD_JObjClearFlagsAll, (a0: *mut HSD_JObj, a1: u32_0));
    hook!(H_HSD_JObjAddChild, HSD_JObjAddChild, (a0: *mut HSD_JObj, a1: *mut HSD_JObj));
    hook!(H_HSD_JObjReqAnim, HSD_JObjReqAnim, (a0: *mut HSD_JObj, a1: f32_0));
    hook!(H_HSD_JObjAnim, HSD_JObjAnim, (a0: *mut HSD_JObj));
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
    pub static mut H_HSD_SisLib_803A5E70: Option<unsafe extern "C" fn() > = None;
            pub unsafe extern "C" fn HSD_SisLib_803A5E70()  {
                match H_HSD_SisLib_803A5E70 {
                    Some(f) => f(),
                    None => { unsafe { melee_runtime::tu::sislib::HSD_SisLib_803A5E70(); } return; },
                }
            }
    pub static mut H_gm_801A36E0: Option<unsafe extern "C" fn(s32, s32) > = None;
            pub unsafe extern "C" fn gm_801A36E0(a0: s32, a1: s32)  {
                match H_gm_801A36E0 {
                    Some(f) => f(a0, a1),
                    None => { unsafe { crate::tu::gm_1A36::gm_801A36E0(a0 as _, a1 as _); } return; },
                }
            }
    pub static mut H_gm_GetCurrentGameMode: Option<unsafe extern "C" fn() -> u8_0> = None;
            pub unsafe extern "C" fn gm_GetCurrentGameMode() -> u8_0 {
                match H_gm_GetCurrentGameMode {
                    Some(f) => f(),
                    None => { return unsafe { crate::tu::gm_1A3F::gm_GetCurrentGameMode() as _ }; },
                }
            }
    pub static mut H_gmMainLib_GetGameRules: Option<unsafe extern "C" fn() -> *mut GameRules> = None;
            pub unsafe extern "C" fn gmMainLib_GetGameRules() -> *mut GameRules {
                match H_gmMainLib_GetGameRules {
                    Some(f) => f(),
                    None => { return unsafe { crate::tu::gmmain_lib::gmMainLib_GetGameRules() as _ }; },
                }
            }
    pub static mut H_gmMainLib_8015EA80: Option<unsafe extern "C" fn() > = None;
            pub unsafe extern "C" fn gmMainLib_8015EA80()  {
                match H_gmMainLib_8015EA80 {
                    Some(f) => f(),
                    None => { unsafe { crate::tu::gmmain_lib::gmMainLib_8015EA80(); } return; },
                }
            }
    pub static mut H_gmMainLib_8015ECB0: Option<unsafe extern "C" fn() -> ::core::ffi::c_int> = None;
            pub unsafe extern "C" fn gmMainLib_8015ECB0() -> ::core::ffi::c_int {
                match H_gmMainLib_8015ECB0 {
                    Some(f) => f(),
                    None => { return unsafe { crate::tu::gmmain_lib::gmMainLib_8015ECB0() as _ }; },
                }
            }
    pub static mut H_gmMainLib_8015EE0C: Option<unsafe extern "C" fn() -> s32> = None;
            pub unsafe extern "C" fn gmMainLib_8015EE0C() -> s32 {
                match H_gmMainLib_8015EE0C {
                    Some(f) => f(),
                    None => { return unsafe { crate::tu::gmmain_lib::gmMainLib_8015EE0C() as _ }; },
                }
            }
    pub static mut H_gmMainLib_8015EE44: Option<unsafe extern "C" fn() -> s32> = None;
            pub unsafe extern "C" fn gmMainLib_8015EE44() -> s32 {
                match H_gmMainLib_8015EE44 {
                    Some(f) => f(),
                    None => { return unsafe { crate::tu::gmmain_lib::gmMainLib_8015EE44() as _ }; },
                }
            }
    pub static mut H_gm_80190EA4: Option<unsafe extern "C" fn() > = None;
            pub unsafe extern "C" fn gm_80190EA4()  {
                match H_gm_80190EA4 {
                    Some(f) => f(),
                    None => { unsafe { crate::tu::gmtou_0::gm_80190EA4(); } return; },
                }
            }
    hook!(H_lbArchive_LoadSections, lbArchive_LoadSections, (a0: *mut HSD_Archive, a1: *mut *mut ::core::ffi::c_void, va: &[usize]));
    hook!(H_GObj_SetupGXLink, GObj_SetupGXLink, (a0: *mut HSD_GObj, a1: GObj_RenderFunc, a2: u8_0, a3: u32_0));
    hook!(H_HSD_GObjObject_80390A70, HSD_GObjObject_80390A70, (a0: *mut HSD_GObj, a1: u8_0, a2: *mut ::core::ffi::c_void));
    hook!(H_HSD_GObjFree, HSD_GObjFree, (a0: *mut HSD_GObj));
    hook!(H_HSD_GObj_SetupProc, HSD_GObj_SetupProc, (a0: *mut HSD_GObj, a1: HSD_GObjEvent, a2: u8_0) -> *mut HSD_GObjProc);
    hook!(H_GObj_InitUserData, GObj_InitUserData, (a0: *mut HSD_GObj, a1: u8_0, a2: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()>, a3: *mut ::core::ffi::c_void));
    pub static mut H_HSD_Free: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) > = None;
            pub unsafe extern "C" fn HSD_Free(a0: *mut ::core::ffi::c_void)  {
                match H_HSD_Free {
                    Some(f) => f(a0),
                    None => { unsafe { melee_runtime::tu::memory::HSD_Free(a0 as _); } return; },
                }
            }
    pub static mut H_HSD_MemAlloc: Option<unsafe extern "C" fn(ssize_t) -> *mut ::core::ffi::c_void> = None;
            pub unsafe extern "C" fn HSD_MemAlloc(a0: ssize_t) -> *mut ::core::ffi::c_void {
                match H_HSD_MemAlloc {
                    Some(f) => f(a0),
                    None => { return unsafe { melee_runtime::tu::memory::HSD_MemAlloc(a0 as _) as _ }; },
                }
            }
    pub static mut H_mn_802339FC: Option<unsafe extern "C" fn() > = None;
            pub unsafe extern "C" fn mn_802339FC()  {
                match H_mn_802339FC {
                    Some(f) => f(),
                    None => { unsafe { crate::tu::mnruleplus::mn_802339FC(); } return; },
                }
            }
    pub static mut H_mnCharSel_80264070: Option<unsafe extern "C" fn() > = None;
            pub unsafe extern "C" fn mnCharSel_80264070()  {
                match H_mnCharSel_80264070 {
                    Some(f) => f(),
                    None => { unsafe { crate::tu::mncharsel::mnCharSel_80264070(); } return; },
                }
            }
    pub static mut H_mnCharSel_802640A0: Option<unsafe extern "C" fn() > = None;
            pub unsafe extern "C" fn mnCharSel_802640A0()  {
                match H_mnCharSel_802640A0 {
                    Some(f) => f(),
                    None => { unsafe { crate::tu::mncharsel::mnCharSel_802640A0(); } return; },
                }
            }
    pub static mut HSD_GObj_804D783C: s32 = unsafe { core::mem::zeroed() };
    pub static mut HSD_GObj_JObjKind: u8_0 = unsafe { core::mem::zeroed() };
}
