//! src/melee/mn/mnruleplus.c: the Rules Plus menu (time limit, friendly fire, pause, score
//! display, sudden-death penalty, stage selection): the option panels, their number / toggle
//! animations, the per-option description text and the scene builder.
//! Callees in TUs that are not ported yet are `ext` stubs named like the C function.
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, unused_mut, unused_variables, unused_assignments, clippy::missing_safety_doc)]

use crate::mn_inl::*;
use core::ffi::{c_char, c_void};
use core::ptr::null_mut;
use melee_core::fp::fmadds;
use melee_types::*;

/// `JObjIndices` (mnruleplus.c): four joint indices. The C is a union with an `s32 packed`
/// initialiser (0x02030506 big-endian); the bytes are stored in that order here.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct JObjIndices {
    pub idx: [u8; 4],
}

melee_core::gc_layout!(JObjIndices, "JObjIndices", size 4, align 4, [
    idx @ 0x0: 4 "u8[4]",
    packed @ 0x0: 4 "s32",
]);

/// The anonymous struct of `mn_803ED2E8`: per-option (min, max) bounds and description ids.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct mn_803ED2E8_t {
    pub stat: [[u8; 2]; 6],
    pub desc: [[u8; 3]; 6],
    pub pad: [u8; 2],
}

melee_core::gc_layout!(mn_803ED2E8_t, "mn_803ED2E8_t", size 0x20, align 1, [
    stat @ 0x0: 12 "u8[6][2]",
    desc @ 0xC: 18 "u8[6][3]",
    pad @ 0x1E: 2 "u8[2]",
]);

/// `mn_803ED1D0_t` as mnruleplus.c defines it (0xA0 bytes). The copy in mnruleplus.h spells the
/// pad and the second table differently (`xE[6]`); the TU's own definition is the one it compiles.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct mn_803ED1D0_t {
    pub x0: [u16; 7],
    pub pad_E: u16,
    pub x10: [u16; 6],
    pub text_start_frames: [f32; 12],
    pub x4C: AnimLoopSettings,
    pub x58: AnimLoopSettings,
    pub x64: [AnimLoopSettings; 2],
    pub x7C: [AnimLoopSettings; 2],
    pub x94: AnimLoopSettings,
}

melee_core::gc_layout!(mn_803ED1D0_t, "mn_803ED1D0_t", size 0xA0, align 4, [
    x0 @ 0x0: 14 "u16[7]",
    pad_E @ 0xE: 2 "u16",
    x10 @ 0x10: 12 "u16[6]",
    text_start_frames @ 0x1C: 48 "float[12]",
    x4C @ 0x4C: 12 "AnimLoopSettings",
    x58 @ 0x58: 12 "AnimLoopSettings",
    x64 @ 0x64: 24 "AnimLoopSettings[2]",
    x7C @ 0x7C: 24 "AnimLoopSettings[2]",
    x94 @ 0x94: 12 "AnimLoopSettings",
]);

pub mod ext {
    use super::*;
    use crate::ext_fns;
    ext_fns! {
        fn GObj_Create(classifier: u16, p_link: u8, priority: u8) -> *mut HSD_GObj;
        extern "C" fn HSD_GObj_JObjCallback(gobj: *mut HSD_GObj, arg1: i32) = crate::tu::adapt::HSD_GObj_JObjCallback__9cd907;
        fn lb_8001204C(a0: *mut HSD_JObj, a1: *mut *mut HSD_JObj, a2: *mut u16, a3: i32) -> i32 = crate::tu::adapt::lb_8001204C__f0bab9;
        fn mn_80229860(a0: i8) = crate::tu::adapt::mn_80229860__033db5;
        fn mn_8022ED6C(a0: *mut HSD_JObj, a1: *mut AnimLoopSettings) -> f32 = crate::tu::adapt::mn_8022ED6C__f73cae;
        fn mn_8022F298(a0: *mut HSD_JObj) -> f32 = crate::tu::adapt::mn_8022F298__0e9b78;
        fn mn_8022F4CC() = crate::tu::adapt::mn_8022F4CC__d43938;
        fn HSD_JObjReqAnimAll(a0: *mut HSD_JObj, a1: f32);
        fn HSD_JObjLoadJoint(a0: *mut HSD_Joint) -> *mut HSD_JObj = crate::tu::adapt::HSD_JObjLoadJoint__3efee1;
        fn HSD_JObjAddAnimAll(a0: *mut HSD_JObj, a1: *mut HSD_AnimJoint, a2: *mut HSD_MatAnimJoint, a3: *mut HSD_ShapeAnimJoint);
        fn HSD_JObjAnimAll(a0: *mut HSD_JObj);
        fn HSD_JObjSetFlagsAll(a0: *mut HSD_JObj, flags: u32);
        fn HSD_JObjClearFlagsAll(a0: *mut HSD_JObj, flags: u32);
        fn HSD_JObjAddChild(jobj: *mut HSD_JObj, child: *mut HSD_JObj);
        fn HSD_JObjReqAnim(jobj: *mut HSD_JObj, frame: f32);
        fn HSD_JObjAnim(jobj: *mut HSD_JObj);
        fn HSD_SisLib_803A5CC4(a0: *mut HSD_Text) = crate::tu::adapt::HSD_SisLib_803A5CC4__6c7e38;
        fn mn_8023164C() = crate::tu::adapt::mn_8023164C__7a202c;
        fn mn_80231F80(a0: u8) -> i32;
        fn mnStageSw_80237410() = crate::tu::mnstagesw::mnStageSw_80237410;
        fn gm_GetCurrentGameMode() -> u8 = crate::tu::adapt::gm_GetCurrentGameMode__f56b36;
        fn gmMainLib_GetGameRules() -> *mut GameRules = crate::tu::gmmain_lib::gmMainLib_GetGameRules;
        fn gmMainLib_8015EE0C() -> i32 = crate::tu::gmmain_lib::gmMainLib_8015EE0C;
        fn gmMainLib_8015EE44() -> i32 = crate::tu::gmmain_lib::gmMainLib_8015EE44;
        fn GObj_SetupGXLink(gobj: *mut HSD_GObj, render_cb: GObj_RenderFunc, gx_link: u8, priority: u32);
        fn HSD_GObjObject_80390A70(gobj: *mut HSD_GObj, kind: u8, obj: *mut c_void);
        fn HSD_GObjFree(a0: *mut HSD_GObj);
        fn HSD_GObj_SetupProc(a0: *mut HSD_GObj, a1: HSD_GObjEvent, a2: u8) -> *mut HSD_GObjProc;
        fn GObj_InitUserData(gobj: *mut HSD_GObj, kind: u8, remove_func: Option<unsafe extern "C" fn(*mut c_void) -> ()>, data: *mut c_void, );
        extern "C" fn HSD_Free(ptr: *mut c_void) = crate::tu::adapt::HSD_Free__3ee311;
        fn HSD_MemAlloc(size: isize) -> *mut c_void = crate::tu::adapt::HSD_MemAlloc__849f1b;
    }
    pub static mut HSD_GObj_804D783C: i32 = unsafe { core::mem::zeroed() };
    pub static mut HSD_GObj_JObjKind: u8 = unsafe { core::mem::zeroed() };
    pub static mut MenMainConRl_Top: StaticModelDesc = unsafe { core::mem::zeroed() };
    pub static mut MenMainCursorRl_Top: StaticModelDesc = unsafe { core::mem::zeroed() };
    pub static mut MenMainCursorTr01_Top: StaticModelDesc = unsafe { core::mem::zeroed() };
    pub static mut MenMainCursorTr02_Top: StaticModelDesc = unsafe { core::mem::zeroed() };
    pub static mut MenMainCursorTr03_Top: StaticModelDesc = unsafe { core::mem::zeroed() };
    pub static mut MenMainCursorTr04_Top: StaticModelDesc = unsafe { core::mem::zeroed() };
    pub static mut MenMainNmRl_Top: StaticModelDesc = unsafe { core::mem::zeroed() };
    pub static mut mn_803EB6B0: [MenuKindData; 0] = unsafe { core::mem::zeroed() };
}
use ext::*;

pub static mut mn_804D6BE4: f32 = 0.;
pub static mut mn_804D6BE0: *mut HSD_GObj = null_mut();
pub static mut mn_803ED1D0: mn_803ED1D0_t = mn_803ED1D0_t {
    x0: [
        3 as i32 as u16,
        4 as i32 as u16,
        5 as i32 as u16,
        6 as i32 as u16,
        7 as i32 as u16,
        8 as i32 as u16,
        9 as i32 as u16,
    ],
    pad_E: 0,
    x10: [
        7 as i32 as u16,
        2 as i32 as u16,
        2 as i32 as u16,
        2 as i32 as u16,
        2 as i32 as u16,
        0 as i32 as u16,
    ],
    text_start_frames: [
        20.0_f32,
        21.0_f32,
        22.0_f32,
        23.0_f32,
        24.0_f32,
        25.0_f32,
        26.0_f32,
        27.0_f32,
        28.0_f32,
        29.0_f32,
        30.0_f32,
        31.0_f32,
    ],
    x4C: AnimLoopSettings {
        start_frame: 0.0_f32,
        end_frame: 99.0_f32,
        loop_frame: 0.0_f32,
    },
    x58: AnimLoopSettings {
        start_frame: 200.0_f32,
        end_frame: 399.0_f32,
        loop_frame: 200.0_f32,
    },
    x64: [
        AnimLoopSettings {
            start_frame: 100.0_f32,
            end_frame: 124.0_f32,
            loop_frame: -0.1_f32,
        },
        AnimLoopSettings {
            start_frame: 125.0_f32,
            end_frame: 149.0_f32,
            loop_frame: -0.1_f32,
        },
    ],
    x7C: [
        AnimLoopSettings {
            start_frame: 150.0_f32,
            end_frame: 174.0_f32,
            loop_frame: -0.1_f32,
        },
        AnimLoopSettings {
            start_frame: 175.0_f32,
            end_frame: 199.0_f32,
            loop_frame: -0.1_f32,
        },
    ],
    x94: AnimLoopSettings {
        start_frame: 0.0_f32,
        end_frame: 99.0_f32,
        loop_frame: 60.0_f32,
    },
};
pub static mut mn_803ED270: [AnimLoopSettings; 3] = [
    AnimLoopSettings {
        start_frame: 0.0_f32,
        end_frame: 3.0_f32,
        loop_frame: -0.1_f32,
    },
    AnimLoopSettings {
        start_frame: 20.0_f32,
        end_frame: 23.0_f32,
        loop_frame: -0.1_f32,
    },
    AnimLoopSettings {
        start_frame: 40.0_f32,
        end_frame: 43.0_f32,
        loop_frame: -0.1_f32,
    },
];
pub static mut mn_803ED294: [AnimLoopSettings; 7] = [
    AnimLoopSettings {
        start_frame: 100.0_f32,
        end_frame: 103.0_f32,
        loop_frame: -0.1_f32,
    },
    AnimLoopSettings {
        start_frame: 120.0_f32,
        end_frame: 123.0_f32,
        loop_frame: -0.1_f32,
    },
    AnimLoopSettings {
        start_frame: 140.0_f32,
        end_frame: 143.0_f32,
        loop_frame: -0.1_f32,
    },
    AnimLoopSettings {
        start_frame: 30.0_f32,
        end_frame: 49.0_f32,
        loop_frame: -0.1_f32,
    },
    AnimLoopSettings {
        start_frame: 70.0_f32,
        end_frame: 89.0_f32,
        loop_frame: -0.1_f32,
    },
    AnimLoopSettings {
        start_frame: 90.0_f32,
        end_frame: 109.0_f32,
        loop_frame: -0.1_f32,
    },
    AnimLoopSettings {
        start_frame: 50.0_f32,
        end_frame: 69.0_f32,
        loop_frame: -0.1_f32,
    },
];
pub static mut mn_803ED2E8: mn_803ED2E8_t = mn_803ED2E8_t {
    stat: [
        [0 as i32 as u8, 99 as i32 as u8],
        [0 as i32 as u8, 1 as i32 as u8],
        [0 as i32 as u8, 1 as i32 as u8],
        [0 as i32 as u8, 1 as i32 as u8],
        [0 as i32 as u8, 2 as i32 as u8],
        [0 as i32 as u8, 0 as i32 as u8],
    ],
    desc: [
        [
            0x37 as i32 as u8,
            0 as i32 as u8,
            0 as i32 as u8,
        ],
        [
            0x38 as i32 as u8,
            0x39 as i32 as u8,
            0 as i32 as u8,
        ],
        [
            0x3a as i32 as u8,
            0x3b as i32 as u8,
            0 as i32 as u8,
        ],
        [
            0x3d as i32 as u8,
            0x3c as i32 as u8,
            0 as i32 as u8,
        ],
        [
            0x3f as i32 as u8,
            0x3e as i32 as u8,
            0x40 as i32 as u8,
        ],
        [
            0x41 as i32 as u8,
            0 as i32 as u8,
            0 as i32 as u8,
        ],
    ],
    pad: [0 as i32 as u8, 0 as i32 as u8],
};
pub static mut mn_803ED308: [c_char; 24] = unsafe {
    core::mem::transmute::<
        [u8; 24],
        [c_char; 24],
    >(*b"Can't get user_data.\n\0\0\0")
};
pub static mut mn_803ED320: [c_char; 16] = unsafe {
    core::mem::transmute::<
        [u8; 16],
        [c_char; 16],
    >(*b"mnruleplus.c\0\0\0\0")
};
pub static mut mn_803ED330: [c_char; 16] = unsafe {
    core::mem::transmute::<
        [u8; 16],
        [c_char; 16],
    >(*b"user_data\0\0\0\0\0\0\0")
};
pub static mut mnRulePlus_TimeLimitOffFrame: f32 = 1.0_f32;
unsafe extern "C" fn SisLib_ClearText(mut text: *mut *mut HSD_Text) {
    if !(*text).is_null() {
        HSD_SisLib_803A5CC4(*text);
        *text = null_mut();
    }
}
unsafe extern "C" fn mnRulePlus_GetDescIdx(mut sel: u8, mut confirmed: u8) -> u8 {
    if sel as i32 == 0 || sel as i32 == 5 {
        return mn_803ED2E8.desc[sel as usize][0 as i32 as usize];
    }
    return mn_803ED2E8.desc[sel as usize][confirmed as usize];
}
unsafe extern "C" fn mnRulePlus_SaveRules() {
    let mut data: *mut MenuRulesPlusData = (*mn_804D6BE0).user_data
        as *mut MenuRulesPlusData;
    (*gmMainLib_GetGameRules()).stock_time_limit = (*data)
        .rule_values
        .anon_0
        .time_limit;
    (*gmMainLib_GetGameRules()).friendly_fire = (*data)
        .rule_values
        .anon_0
        .friendly_fire;
    (*gmMainLib_GetGameRules()).pause = (*data).rule_values.anon_0.pause;
    (*gmMainLib_GetGameRules()).score_display = (*data).rule_values.anon_0.score;
    (*gmMainLib_GetGameRules()).unk_xc = (*data).rule_values.anon_0.sd_penalty;
}
unsafe extern "C" fn mnRulePlus_IsOptionVisible(mut sel: u8) -> i32 {
    if gm_GetCurrentGameMode() as i32
        == GM_TOURNAMENT as i32
        && sel as i32 == 1
    {
        return 0
    } else if sel as i32 == 3 {
        if gmMainLib_8015EE0C() != 0 {
            return 1
        } else {
            return 0
        }
    } else if sel as i32 == 5 {
        if gmMainLib_8015EE44() != 0 {
            return 1
        } else {
            return 0
        }
    } else {
        return 1
    };
}
pub unsafe extern "C" fn fn_8023201C(mut gobj: *mut HSD_GObj) {
    let mut data: *mut MenuRulesPlusData = (*mn_804D6BE0).user_data
        as *mut MenuRulesPlusData;
    let mut buttons: u32 = 0;
    buttons = mn_80229624(4);
    mn_804A04F0.buttons = buttons as u64;
    if buttons & 0x200 != 0 {
        if mn_804A04F0.hovered_selection as i32 == 5
        {
            sfxForward();
            mn_804A04F0.entering_menu = 1;
            mnRulePlus_SaveRules();
            mn_804D6BC8.cooldown = 5;
            mnStageSw_80237410();
            HSD_GObjFree(gobj);
        }
    } else if buttons & 0x100 != 0 {
        sfxForward();
        match gm_GetCurrentGameMode() as i32 {
            1 => {
                mnRulePlus_SaveRules();
                mn_80229860(GM_VS as i32 as i8);
                return;
            }
            _ => {}
        }
        mnRulePlus_SaveRules();
        mn_8022F4CC();
        return;
    } else if buttons & 0x20 != 0 {
        sfxBack();
        mn_804A04F0.entering_menu = 0;
        mnRulePlus_SaveRules();
        mn_804D6BC8.cooldown = 5;
        mn_8023164C();
        HSD_GObjFree(gobj);
        return;
    } else if buttons & 1 != 0 {
        sfxMove();
        loop {
            if mn_804A04F0.hovered_selection as i32 == 0 {
                mn_804A04F0.hovered_selection = 5;
            } else {
                mn_804A04F0.hovered_selection = (mn_804A04F0.hovered_selection
                    as i32 - 1) as u16;
            }
            if !(mnRulePlus_IsOptionVisible(mn_804A04F0.hovered_selection as u8)
                == 0)
            {
                break;
            }
        }
        mn_804A04F0.confirmed_selection = (*data)
            .rule_values
            .values[mn_804A04F0.hovered_selection as usize];
        return;
    } else if buttons & 2 != 0 {
        sfxMove();
        loop {
            if mn_804A04F0.hovered_selection as i32 == 5 {
                mn_804A04F0.hovered_selection = 0;
            } else {
                mn_804A04F0.hovered_selection = (mn_804A04F0.hovered_selection
                    as i32 + 1) as u16;
            }
            if !(mnRulePlus_IsOptionVisible(mn_804A04F0.hovered_selection as u8)
                == 0)
            {
                break;
            }
        }
        mn_804A04F0.confirmed_selection = (*data)
            .rule_values
            .values[mn_804A04F0.hovered_selection as usize];
        return;
    } else if mn_804A04F0.hovered_selection as i32
        != 5
    {
        let mut bounds: *mut u8 = &raw mut *(&raw mut mn_803ED2E8.stat
            as *mut [u8; 2])
            .offset(mn_804A04F0.hovered_selection as isize) as *mut u8;
        if buttons & 4 != 0 {
            sfxMove();
            if mn_804A04F0.confirmed_selection as i32
                > *bounds.offset(0 as i32 as isize) as i32
            {
                mn_804A04F0.confirmed_selection = (mn_804A04F0.confirmed_selection
                    as i32 - 1) as u8;
                return;
            }
            mn_804A04F0.confirmed_selection = *bounds
                .offset(1 as i32 as isize);
            return;
        }
        if buttons & 8 != 0 {
            sfxMove();
            if (mn_804A04F0.confirmed_selection as i32)
                < *bounds.offset(1 as i32 as isize) as i32
            {
                mn_804A04F0.confirmed_selection = (mn_804A04F0.confirmed_selection
                    as i32 + 1) as u8;
                return;
            }
            mn_804A04F0.confirmed_selection = *bounds
                .offset(0 as i32 as isize);
        }
    }
}
pub unsafe extern "C" fn mn_80232458(
    mut option: u8,
    mut value: u8,
    mut direction: u8,
) -> *mut AnimLoopSettings {
    let mut count: u8 = 0;
    if option as i32 != 1
        && option as i32 != 2
        && option as i32 != 3
        && option as i32 != 4
    {
        return null_mut();
    }
    count = mn_803ED2E8.stat[option as usize][1 as i32 as usize];
    if direction as i32 != 0 {
        if value as i32 == 0 {
            return (&raw mut mn_803ED270 as *mut AnimLoopSettings).offset(count as isize)
                as *mut AnimLoopSettings;
        }
        return (&raw mut mn_803ED270 as *mut AnimLoopSettings)
            .offset((value as i32 - 1) as isize)
            as *mut AnimLoopSettings;
    }
    return (&raw mut mn_803ED294 as *mut AnimLoopSettings)
        .offset((count as i32 - value as i32) as isize)
        as *mut AnimLoopSettings;
}
unsafe extern "C" fn mnRulePlus_AnimTimeDigits(
    mut value: u8,
    mut jobjs: *mut *mut HSD_JObj,
) {
    let mut jobj: *mut HSD_JObj = null_mut();
    jobj = *jobjs.offset(2 as i32 as isize);
    HSD_JObjReqAnimAll(
        jobj,
        (value as i32 / 10) as u8 as f32,
    );
    HSD_JObjAnimAll(jobj);
    jobj = *jobjs.offset(3 as i32 as isize);
    HSD_JObjReqAnimAll(
        jobj,
        (value as i32 % 10) as u8 as f32,
    );
    HSD_JObjAnimAll(jobj);
}
pub static mut mn_804DBE40: JObjIndices = JObjIndices {
    idx: [2, 3, 5, 6],
};
unsafe extern "C" fn mnRulePlus_AnimZeros(mut jobjs: *mut *mut HSD_JObj) {
    let mut jobj1: *mut HSD_JObj = null_mut();
    let mut jobj2: *mut HSD_JObj = null_mut();
    jobj1 = *jobjs.offset(5 as i32 as isize);
    HSD_JObjReqAnimAll(jobj1, 0.0_f32);
    HSD_JObjAnimAll(jobj1);
    jobj2 = *jobjs.offset(6 as i32 as isize);
    HSD_JObjReqAnimAll(jobj2, 0.0_f32);
    HSD_JObjAnimAll(jobj2);
}
pub unsafe extern "C" fn mn_802324E4(
    mut time_limit: u8,
    mut data: *mut MenuRulesPlusData,
) {
    let mut i: i32 = 0;
    let mut local_indices: JObjIndices = JObjIndices { idx: [0; 4] };
    let mut jobjs: *mut *mut HSD_JObj = core::ptr::null_mut::<*mut HSD_JObj>();
    jobjs = &raw mut *(&raw mut (*data).x34 as *mut [*mut HSD_JObj; 7])
        .offset(0 as i32 as isize) as *mut *mut HSD_JObj;
    local_indices = mn_804DBE40;
    if time_limit as i32 == 0 {
        i = 0 as i32;
        loop {
            let mut idx: u8 = local_indices.idx[i as usize];
            HSD_JObjSetFlagsAll(*jobjs.offset(idx as isize), JOBJ_HIDDEN as u32);
            i += 1;
            if i >= 4 {
                break;
            }
        }
        HSD_JObjReqAnimAll(
            *jobjs.offset(4 as i32 as isize),
            mnRulePlus_TimeLimitOffFrame,
        );
        HSD_JObjAnimAll(*jobjs.offset(4 as i32 as isize));
        return;
    }
    i = 0 as i32;
    loop {
        let mut idx_0: u8 = local_indices.idx[i as usize];
        HSD_JObjClearFlagsAll(*jobjs.offset(idx_0 as isize), JOBJ_HIDDEN as u32);
        i += 1;
        if i >= 4 {
            break;
        }
    }
    HSD_JObjReqAnimAll(*jobjs.offset(4 as i32 as isize), mn_804D6BE4);
    HSD_JObjAnimAll(*jobjs.offset(4 as i32 as isize));
    mnRulePlus_AnimTimeDigits(time_limit, jobjs);
    mnRulePlus_AnimZeros(jobjs);
}
pub unsafe extern "C" fn mn_80232660(
    mut gobj: *mut HSD_GObj,
    mut jobj: *mut HSD_JObj,
    mut option: u8,
) {
    let mut frame: f32 = 0.;
    match option as i32 {
        1 | 2 | 3 | 4 => {
            frame = mn_8022F298(jobj as *mut HSD_JObj);
            let mut settings: *mut AnimLoopSettings = core::ptr::null_mut::<
                AnimLoopSettings,
            >();
            let mut i: i32 = 0;
            i = 0 as i32;
            while i < 3 {
                settings = (&raw mut mn_803ED294 as *mut AnimLoopSettings)
                    .offset(i as isize) as *mut AnimLoopSettings;
                if (*settings).start_frame <= frame && frame <= (*settings).end_frame {
                    break;
                }
                settings = (&raw mut mn_803ED270 as *mut AnimLoopSettings)
                    .offset(i as isize) as *mut AnimLoopSettings;
                if (*settings).start_frame <= frame && frame <= (*settings).end_frame {
                    break;
                }
                i += 1;
            }
            mn_8022ED6C(jobj as *mut HSD_JObj, settings);
        }
        0 => {}
        5 => return,
        _ => {
            return;
        }
    }
    return;
}
unsafe extern "C" fn mn_802327A4_InitOptionRoots(
    mut option_roots: *mut *mut HSD_JObj,
    mut rule_data: *mut mn_803ED1D0_t,
    mut data: *mut MenuRulesPlusData,
    mut num_options: u8,
    mut i: *mut i32,
) {
    let mut option_root: *mut *mut HSD_JObj = core::ptr::null_mut::<*mut HSD_JObj>();
    let mut j8: u8 = 0;
    let mut j: i32 = 0;
    let mut visible: i32 = 0;
    option_root = option_roots;
    *i = 0 as i32;
    while *i < num_options as i32 {
        let mut valid: i32 = 0;
        if gm_GetCurrentGameMode() as i32
            == GM_TOURNAMENT as i32
            && *i as u8 as i32 == 1
        {
            valid = 0 as i32;
        } else if *i as u8 as i32 == 3 {
            if gmMainLib_8015EE0C() != 0 {
                valid = 1 as i32;
            } else {
                valid = 0 as i32;
            }
        } else if *i as u8 as i32 == 5 {
            if gmMainLib_8015EE44() != 0 {
                valid = 1 as i32;
            } else {
                valid = 0 as i32;
            }
        } else {
            valid = 1 as i32;
        }
        if valid != 0 {
            let mut child: *mut HSD_JObj = null_mut();
            let mut root: *mut HSD_JObj = null_mut();
            visible = 0 as i32;
            j = visible;
            while j < *i as u8 as i32 {
                j8 = j as u8;
                if mn_80231F80(j8) != 0 {
                    visible += 1;
                }
                j += 1;
            }
            root = (*data).xC[(*rule_data).x0[visible as u8 as usize] as usize];
            if root.is_null() {
                child = null_mut();
            } else {
                child = (*root).child as *mut HSD_JObj;
            }
            *option_root = child;
        }
        option_root = option_root.offset(1);
        *i += 1;
    }
}
unsafe extern "C" fn mn_802327A4_UpdateOption(
    mut data: *mut MenuRulesPlusData,
    mut gobj: *mut HSD_GObj,
    mut arg2: u32,
    mut arg1: u32,
    mut selected: *mut u16,
) {
    if arg2 as i32 != 0 {
        let mut hovered: u16 = 0;
        let mut data2: *mut MenuRulesPlusData = (*gobj).user_data
            as *mut MenuRulesPlusData;
        hovered = mn_804A04F0.hovered_selection;
        let mut hovered_u8: u8 = hovered as u8;
        let mut confirmed: u8 = mn_804A04F0.confirmed_selection;
        let mut tree: *mut HSD_JObj = (*data)
            .x34[hovered as usize][0 as i32 as usize];
        match hovered_u8 as i32 {
            0 => {
                mn_802324E4(confirmed, data2);
            }
            1 | 2 | 3 | 4 => {
                if mn_804A04F0.buttons & 4 != 0 {
                    HSD_JObjReqAnimAll(
                        tree,
                        (*mn_80232458(hovered_u8, confirmed, 0)).start_frame,
                    );
                } else {
                    HSD_JObjReqAnimAll(
                        tree,
                        (*mn_80232458(hovered_u8, confirmed, 1)).start_frame,
                    );
                }
                HSD_JObjAnimAll(tree);
            }
            5 | _ => {}
        }
    }
    if arg1 as i32 != 0 {
        *selected = mn_804A04F0.hovered_selection;
    } else {
        *selected = (*data).hovered_selection as u16;
    };
}
pub unsafe extern "C" fn mn_802327A4(
    mut gobj: *mut HSD_GObj,
    mut arg1: u32,
    mut arg2: u32,
) {
    let mut option_roots: [*mut HSD_JObj; 6] = [null_mut(); 6];
    let mut jobj_parts: [*mut HSD_JObj; 17] = [null_mut(); 17];
    let mut jobj_map: [u16; 17] = [0; 17];
    let mut rule_data: *mut mn_803ED1D0_t = &raw mut mn_803ED1D0;
    let mut num_options: u8 = (*(&raw mut mn_803EB6B0 as *mut MenuKindData)
        .offset(15 as i32 as isize))
        .selection_count;
    let mut data: *mut MenuRulesPlusData = (*gobj).user_data as *mut MenuRulesPlusData;
    let mut part: *mut HSD_JObj = null_mut();
    let mut i: i32 = 0;
    let mut visible: i32 = 0;
    let mut selected: u16 = 0;
    i = 0 as i32;
    while 17 > i {
        jobj_map[i as usize] = i as u16;
        i += 1;
    }
    mn_802327A4_InitOptionRoots(
        &raw mut option_roots as *mut *mut HSD_JObj,
        rule_data,
        data,
        num_options,
        &raw mut i,
    );
    if arg1 as i32 != 0 {
        let mut new_sel: u8 = 0;
        selected = (*data).hovered_selection as u16;
        lb_8001204C(
            option_roots[selected as usize],
            &raw mut jobj_parts as *mut *mut HSD_JObj,
            &raw mut jobj_map as *mut u16,
            17,
        );
        HSD_JObjSetFlagsAll(
            jobj_parts[16 as i32 as usize],
            JOBJ_HIDDEN as u32,
        );
        HSD_JObjSetFlagsAll(
            jobj_parts[13 as i32 as usize],
            JOBJ_HIDDEN as u32,
        );
        part = jobj_parts[2 as i32 as usize];
        if selected as i32 == 5 {
            HSD_JObjReqAnimAll(
                part,
                (*rule_data).x7C[0 as i32 as usize].start_frame,
            );
        } else {
            HSD_JObjReqAnimAll(
                part,
                (*rule_data).x64[0 as i32 as usize].start_frame,
            );
        }
        HSD_JObjAnimAll(part);
        let mut part_0: *mut HSD_JObj = jobj_parts[7 as i32 as usize];
        HSD_JObjReqAnim(
            part_0,
            (*rule_data)
                .text_start_frames[(selected as i32
                * 2) as usize],
        );
        HSD_JObjAnim(part_0);
        HSD_JObjSetFlagsAll(
            jobj_parts[8 as i32 as usize],
            JOBJ_HIDDEN as u32,
        );
        new_sel = mn_804A04F0.hovered_selection as u8;
        lb_8001204C(
            option_roots[mn_804A04F0.hovered_selection as u8 as usize],
            &raw mut jobj_parts as *mut *mut HSD_JObj,
            &raw mut jobj_map as *mut u16,
            17,
        );
        let mut part_1: *mut HSD_JObj = jobj_parts[16 as i32 as usize];
        HSD_JObjClearFlagsAll(part_1, JOBJ_HIDDEN as u32);
        HSD_JObjReqAnimAll(part_1, (*rule_data).x4C.start_frame);
        if new_sel as i32 != 5 {
            let mut part_2: *mut HSD_JObj = jobj_parts[13
                as usize];
            let mut part2: *mut HSD_JObj = null_mut();
            HSD_JObjClearFlagsAll(part_2, JOBJ_HIDDEN as u32);
            HSD_JObjReqAnimAll(part_2, (*rule_data).x58.start_frame);
            part2 = jobj_parts[2 as i32 as usize];
            HSD_JObjReqAnimAll(
                part2,
                (*rule_data).x64[1 as i32 as usize].start_frame,
            );
            HSD_JObjAnimAll(part2);
        } else {
            let mut part_3: *mut HSD_JObj = jobj_parts[2 as i32 as usize];
            HSD_JObjReqAnimAll(
                part_3,
                (*rule_data).x7C[1 as i32 as usize].start_frame,
            );
            HSD_JObjAnimAll(part_3);
        }
        let mut part_4: *mut HSD_JObj = jobj_parts[7 as i32 as usize];
        HSD_JObjReqAnim(
            part_4,
            (*rule_data)
                .text_start_frames[(new_sel as i32
                * 2 + 1) as usize],
        );
        HSD_JObjAnim(part_4);
        if new_sel as i32 == 5 {
            let mut part_5: *mut HSD_JObj = jobj_parts[8 as i32 as usize];
            HSD_JObjClearFlagsAll(part_5, JOBJ_HIDDEN as u32);
            HSD_JObjReqAnimAll(part_5, (*rule_data).x94.start_frame);
            HSD_JObjAnimAll(part_5);
        }
    }
    mn_802327A4_UpdateOption(data, gobj, arg2, arg1, &raw mut selected);
    i = 0 as i32;
    while i < num_options as i32 {
        if gm_GetCurrentGameMode() as i32
            == GM_TOURNAMENT as i32
            && i as u8 as i32 == 1
        {
            visible = 0 as i32;
        } else if i as u8 as i32 == 3 {
            if gmMainLib_8015EE0C() != 0 {
                visible = 1 as i32;
            } else {
                visible = 0 as i32;
            }
        } else if i as u8 as i32 == 5 {
            if gmMainLib_8015EE44() != 0 {
                visible = 1 as i32;
            } else {
                visible = 0 as i32;
            }
        } else {
            visible = 1 as i32;
        }
        if visible != 0 {
            let mut als: *mut AnimLoopSettings = core::ptr::null_mut::<
                AnimLoopSettings,
            >();
            lb_8001204C(
                option_roots[i as usize],
                &raw mut jobj_parts as *mut *mut HSD_JObj,
                &raw mut jobj_map as *mut u16,
                17,
            );
            if i == selected as u8 as i32 {
                mn_8022ED6C(
                    jobj_parts[16 as i32 as usize],
                    &raw mut (*rule_data).x4C,
                );
            }
            if i == selected as u8 as i32 && i != 5 {
                mn_8022ED6C(
                    jobj_parts[13 as i32 as usize],
                    &raw mut (*rule_data).x58,
                );
            }
            if i == 5 {
                let mut tmp: *mut AnimLoopSettings = &raw mut (*rule_data).x7C
                    as *mut AnimLoopSettings;
                als = tmp
                    .offset(
                        (i == selected as u8 as i32) as i32 as isize,
                    ) as *mut AnimLoopSettings;
            } else {
                let mut tmp_0: *mut AnimLoopSettings = &raw mut (*rule_data).x64
                    as *mut AnimLoopSettings;
                als = tmp_0
                    .offset(
                        (i == selected as u8 as i32) as i32 as isize,
                    ) as *mut AnimLoopSettings;
            }
            mn_8022ED6C(jobj_parts[2 as i32 as usize], als);
            mn_8022ED6C(
                jobj_parts[8 as i32 as usize],
                &raw mut (*rule_data).x94,
            );
            mn_80232660(
                gobj,
                (*data).x34[i as usize][0 as i32 as usize],
                i as u8,
            );
        }
        i += 1;
    }
}
pub static mut mn_804DBE48: JObjIndices = JObjIndices {
    idx: [2, 3, 5, 6],
};
pub unsafe extern "C" fn mn_80232D4C(
    mut gobj: *mut HSD_GObj,
    mut arg1: u32,
    mut arg2: u32,
) {
    let mut data: *mut MenuRulesPlusData = (*gobj).user_data as *mut MenuRulesPlusData;
    let mut raw_selection: u32 = 0;
    let mut selection: u8 = 0;
    let mut confirmed: u8 = 0;
    let mut desc_idx: u8 = 0;
    let mut text: *mut HSD_Text = null_mut();
    if arg1 as i32 != 0 {
        raw_selection = mn_804A04F0.hovered_selection as u32;
    } else {
        raw_selection = (*data).hovered_selection as u32;
    }
    selection = raw_selection as u8;
    let mut current_block_32: u64;
    match (*data).state as i32 {
        2 | 4 => {
            SisLib_ClearText(&raw mut (*data).description);
            current_block_32 = 8156856794521447929;
        }
        5 => {
            current_block_32 = 8156856794521447929;
        }
        3 | 1 => {
            if (*data).description.is_null() {
                confirmed = mn_804A04F0.confirmed_selection;
                SisLib_ClearText(&raw mut (*data).description);
                desc_idx = mnRulePlus_GetDescIdx(selection, confirmed);
                text = HSD_SisLib_803A5ACC(
                    0,
                    1,
                    -9.5_f32,
                    8.0_f32,
                    17.0_f32,
                    364.68332_f32,
                    76.77544_f32,
                );
                (*data).description = text;
                (*text).font_size.x = 0.0521_f32 as f32;
                (*text).font_size.y = (*text).font_size.x;
                HSD_SisLib_803A6368(text, desc_idx as i32);
                return;
            }
            current_block_32 = 5689316957504528238;
        }
        0 => {
            if arg1 as i32 != 0
                || arg2 as i32 != 0
                    && selection as i32 != 0
                    && selection as i32 != 5
            {
                confirmed = mn_804A04F0.confirmed_selection;
                SisLib_ClearText(&raw mut (*data).description);
                if selection as i32 == 0
                    || selection as i32 == 5
                {
                    desc_idx = mn_803ED2E8
                        .desc[selection as usize][0 as i32 as usize];
                } else {
                    desc_idx = mn_803ED2E8.desc[selection as usize][confirmed as usize];
                }
                text = HSD_SisLib_803A5ACC(
                    0,
                    1,
                    -9.5_f32,
                    8.0_f32,
                    17.0_f32,
                    364.68332_f32,
                    76.77544_f32,
                );
                (*data).description = text;
                (*text).font_size.x = 0.0521_f32 as f32;
                (*text).font_size.y = (*text).font_size.x;
                HSD_SisLib_803A6368(text, desc_idx as i32);
            }
            current_block_32 = 5689316957504528238;
        }
        _ => {
            current_block_32 = 5689316957504528238;
        }
    }
    match current_block_32 {
        5689316957504528238 => {}
        _ => return,
    };
}
pub unsafe extern "C" fn fn_80232F44(mut gobj: *mut HSD_GObj) {
    let mut data: *mut MenuRulesPlusData = (*gobj).user_data as *mut MenuRulesPlusData;
    let mut anim_settings: *mut AnimLoopSettings = core::ptr::null_mut::<
        AnimLoopSettings,
    >();
    let mut menu_changed: i32 = 0;
    let mut selection_changed: u32 = 0;
    let mut value_changed: u32 = 0;
    let mut data2: *mut MenuRulesPlusData = null_mut();
    let mut state: u8 = 0;
    let mut jobj: *mut HSD_JObj = null_mut();
    let mut jobj2: *mut HSD_JObj = null_mut();
    state = (*data).state as u8;
    if (state as i32 == 0
        || state as i32 == 1
        || state as i32 == 3)
        && (*data).menu_kind as i32
            != mn_804A04F0.cur_menu as i32
    {
        if mn_804A04F0.entering_menu as i32 != 0 {
            (*data).state = 4 as MenuState8;
        } else {
            (*data).state = 2 as MenuState8;
        }
        state = (*data).state as u8;
        jobj = (*data).xC[2 as i32 as usize];
        jobj2 = jobj;
        match state as i32 {
            1 => {
                anim_settings = (&raw mut mn_803ED294 as *mut AnimLoopSettings)
                    .offset(3 as i32 as isize) as *mut AnimLoopSettings;
            }
            2 => {
                anim_settings = (&raw mut mn_803ED294 as *mut AnimLoopSettings)
                    .offset(5 as i32 as isize) as *mut AnimLoopSettings;
            }
            3 => {
                anim_settings = (&raw mut mn_803ED294 as *mut AnimLoopSettings)
                    .offset(4 as i32 as isize) as *mut AnimLoopSettings;
            }
            4 => {
                anim_settings = (&raw mut mn_803ED294 as *mut AnimLoopSettings)
                    .offset(6 as i32 as isize) as *mut AnimLoopSettings;
            }
            _ => {}
        }
        HSD_JObjReqAnim(jobj2, (*anim_settings).start_frame);
        HSD_JObjAnim(jobj2);
        state = (*data).state as u8;
        if state as i32 == 0
            || state as i32 == 1
            || state as i32 == 3
        {
            menu_changed = 1 as i32;
            selection_changed = 1;
            value_changed = 1;
        }
    }
    state = (*data).state as u8;
    if state as i32 != 0 {
        jobj = (*data).xC[2 as i32 as usize];
        match state as i32 {
            1 => {
                anim_settings = (&raw mut mn_803ED294 as *mut AnimLoopSettings)
                    .offset(3 as i32 as isize) as *mut AnimLoopSettings;
            }
            2 => {
                anim_settings = (&raw mut mn_803ED294 as *mut AnimLoopSettings)
                    .offset(5 as i32 as isize) as *mut AnimLoopSettings;
            }
            3 => {
                anim_settings = (&raw mut mn_803ED294 as *mut AnimLoopSettings)
                    .offset(4 as i32 as isize) as *mut AnimLoopSettings;
            }
            4 => {
                anim_settings = (&raw mut mn_803ED294 as *mut AnimLoopSettings)
                    .offset(6 as i32 as isize) as *mut AnimLoopSettings;
            }
            _ => {}
        }
        if mn_8022F298(jobj as *mut HSD_JObj) >= (*anim_settings).end_frame {
            state = (*data).state as u8;
            match state as i32 {
                3 | 1 => {
                    (*data).state = 0 as MenuState8;
                }
                2 | 4 => {
                    HSD_GObjFree(gobj);
                    return;
                }
                _ => {}
            }
        }
        HSD_JObjAnim(jobj);
    }
    state = (*data).state as u8;
    if state as i32 == 0
        || state as i32 == 1
        || state as i32 == 3
    {
        if (*data).hovered_selection as i32 != mn_804A04F0.hovered_selection as i32 {
            selection_changed = 1;
        }
        if (*data).rule_values.values[mn_804A04F0.hovered_selection as usize]
            as i32
            != mn_804A04F0.confirmed_selection as i32
        {
            value_changed = 1;
        }
    }
    mn_802327A4(gobj, selection_changed, value_changed);
    mn_80232D4C(gobj, selection_changed, value_changed);
    if menu_changed != 0 {
        (*data).menu_kind = mn_804A04F0.cur_menu as MenuKind8;
    }
    if selection_changed as i32 != 0 {
        (*data).hovered_selection = mn_804A04F0.hovered_selection as u8;
    }
    if value_changed as i32 != 0 {
        (*data).rule_values.values[(*data).hovered_selection as usize] = mn_804A04F0
            .confirmed_selection;
        data2 = (*gobj).user_data as *mut MenuRulesPlusData;
        (*gmMainLib_GetGameRules()).stock_time_limit = (*data2)
            .rule_values
            .anon_0
            .time_limit;
        (*gmMainLib_GetGameRules()).friendly_fire = (*data2)
            .rule_values
            .anon_0
            .friendly_fire;
        (*gmMainLib_GetGameRules()).pause = (*data2).rule_values.anon_0.pause;
        (*gmMainLib_GetGameRules()).score_display = (*data2)
            .rule_values
            .anon_0
            .score;
        (*gmMainLib_GetGameRules()).unk_xc = (*data2)
            .rule_values
            .anon_0
            .sd_penalty;
    }
}
unsafe extern "C" fn mnRulePlus_CountAllVisible(mut limit: u8, mut result: *mut i32) {
    let mut count: i32 = 0;
    let mut iter: i32 = count;
    while iter < limit as i32 {
        if mn_80231F80(iter as u8) != 0 {
            count += 1;
        }
        iter += 1;
    }
    *result = count;
}
unsafe extern "C" fn mnRulePlus_CountVisibleBefore(mut limit: u8) -> i32 {
    let mut count: i32 = 0;
    let mut iter: i32 = 0;
    iter = 0 as i32;
    while iter < limit as i32 {
        if mn_80231F80(iter as u8) != 0 {
            count += 1;
        }
        iter += 1;
    }
    return count;
}
pub unsafe extern "C" fn mn_80233218(mut state: MenuState) -> *mut HSD_GObj {
    let mut sub_count_ptr: *mut u16 = null_mut();
    let mut rules: *mut GameRules = null_mut();
    let mut gobj: *mut HSD_GObj = null_mut();
    let mut user_data: *mut MenuRulesPlusData = core::ptr::null_mut::<
        MenuRulesPlusData,
    >();
    let mut num_options: u8 = 0;
    let mut selected: u8 = 0;
    let mut confirmed: u8 = 0;
    let mut i: i32 = 0;
    let mut j: i32 = 0;
    let mut vis_before: i32 = 0;
    let mut vis_total: i32 = 0;
    let mut visible: i32 = 0;
    let mut als: *mut AnimLoopSettings = null_mut();
    let mut option_jobj: *mut HSD_JObj = null_mut();
    let mut cursor_jobj: *mut HSD_JObj = null_mut();
    let mut value_jobj: *mut HSD_JObj = null_mut();
    let mut desc: *mut StaticModelDesc = null_mut();
    let mut rule_data: *mut mn_803ED1D0_t = null_mut();
    let mut root_jobj: *mut HSD_JObj = null_mut();
    let mut jobj_parts: [*mut HSD_JObj; 17] = [null_mut(); 17];
    let mut jobj_map: [u16; 17] = [0; 17];
    rule_data = &raw mut mn_803ED1D0;
    selected = mn_804A04F0.hovered_selection as u8;
    num_options = (*(&raw mut mn_803EB6B0 as *mut MenuKindData)
        .offset(15 as i32 as isize))
        .selection_count;
    i = 0 as i32;
    while 17 > i {
        jobj_map[i as usize] = i as u16;
        i += 1;
    }
    desc = &raw mut MenMainConRl_Top;
    gobj = GObj_Create(6, 7, 0x80);
    mn_804D6BE0 = gobj;
    root_jobj = HSD_JObjLoadJoint((*desc).joint as *mut HSD_Joint);
    HSD_GObjObject_80390A70(
        gobj,
        HSD_GObj_JObjKind,
        root_jobj as *mut c_void,
    );
    GObj_SetupGXLink(
        gobj,
        Some(
            HSD_GObj_JObjCallback
               ,
        ),
        4,
        0x80,
    );
    HSD_GObj_SetupProc(
        gobj,
        Some(fn_80232F44),
        0,
    );
    HSD_JObjAddAnimAll(
        root_jobj,
        (*desc).animjoint,
        (*desc).matanim_joint,
        (*desc).shapeanim_joint,
    );
    HSD_JObjReqAnimAll(root_jobj, 0.0_f32);
    HSD_JObjAnimAll(root_jobj);
    user_data = HSD_MemAlloc(core::mem::size_of::<MenuRulesPlusData>() as isize)
        as *mut MenuRulesPlusData;
    if user_data.is_null() {
        OSReport(mn_803ED308.as_ptr());
        __assert("mnruleplus.c", 0x3df, "user_data");
    }
    GObj_InitUserData(
        gobj,
        0,
        Some(HSD_Free),
        user_data as *mut c_void,
    );
    (*user_data).menu_kind = mn_804A04F0.cur_menu;
    (*user_data).hovered_selection = mn_804A04F0.hovered_selection as u8;
    (*user_data).rule_values.anon_0.time_limit = (*gmMainLib_GetGameRules())
        .stock_time_limit;
    (*user_data).rule_values.anon_0.friendly_fire = (*gmMainLib_GetGameRules())
        .friendly_fire;
    (*user_data).rule_values.anon_0.pause = (*gmMainLib_GetGameRules()).pause;
    (*user_data).rule_values.anon_0.score = (*gmMainLib_GetGameRules())
        .score_display;
    rules = gmMainLib_GetGameRules();
    (*user_data).rule_values.anon_0.sd_penalty = (*rules).unk_xc;
    (*user_data).state = state as u8 as MenuState8;
    (*user_data).description = null_mut();
    i = 0 as i32;
    while i < 10 {
        lb_80011E24(root_jobj as *mut HSD_JObj, (&raw mut (*user_data).xC as *mut *mut HSD_JObj).offset(i as isize)
                as *mut *mut HSD_JObj, &[i, -1]);
        i += 1;
    }
    mn_804A04F0.confirmed_selection = (*user_data)
        .rule_values
        .values[(*user_data).hovered_selection as usize];
    if state as u8 as i32 != 0 {
        let mut anim_jobj: *mut HSD_JObj = (*user_data)
            .xC[2 as i32 as usize];
        match (*user_data).state as i32 {
            1 => {
                als = (&raw mut mn_803ED294 as *mut AnimLoopSettings)
                    .offset(3 as i32 as isize) as *mut AnimLoopSettings;
            }
            3 => {
                als = (&raw mut mn_803ED294 as *mut AnimLoopSettings)
                    .offset(4 as i32 as isize) as *mut AnimLoopSettings;
            }
            2 | _ => {}
        }
        HSD_JObjReqAnim(anim_jobj, (*als).start_frame);
        HSD_JObjAnim(anim_jobj);
    }
    sub_count_ptr = &raw mut mn_803ED1D0.x10 as *mut u16;
    i = 0 as i32;
    while i < num_options as i32 {
        vis_before = mnRulePlus_CountVisibleBefore(i as u8);
        option_jobj = (*user_data)
            .xC[mn_803ED1D0.x0[vis_before as u8 as usize] as usize];
        mnRulePlus_CountAllVisible(6, &raw mut vis_total);
        HSD_JObjReqAnim(option_jobj, vis_total as f32);
        HSD_JObjAnim(option_jobj);
        if gm_GetCurrentGameMode() as i32
            == GM_TOURNAMENT as i32
            && i as u8 as i32 == 1
        {
            visible = 0 as i32;
        } else if i as u8 as i32 == 3 {
            if gmMainLib_8015EE0C() != 0 {
                visible = 1 as i32;
            } else {
                visible = 0 as i32;
            }
        } else if i as u8 as i32 == 5 {
            if gmMainLib_8015EE44() != 0 {
                visible = 1 as i32;
            } else {
                visible = 0 as i32;
            }
        } else {
            visible = 1 as i32;
        }
        if visible != 0 {
            desc = &raw mut MenMainCursorRl_Top;
            cursor_jobj = HSD_JObjLoadJoint((*desc).joint as *mut HSD_Joint);
            HSD_JObjAddAnimAll(
                cursor_jobj,
                (*desc).animjoint,
                (*desc).matanim_joint,
                (*desc).shapeanim_joint,
            );
            HSD_JObjReqAnimAll(cursor_jobj, 0.0_f32);
            HSD_JObjAnimAll(cursor_jobj);
            lb_8001204C(
                cursor_jobj as *mut HSD_JObj,
                &raw mut jobj_parts as *mut *mut HSD_JObj,
                &raw mut jobj_map as *mut u16,
                17,
            );
            let mut text_jobj: *mut HSD_JObj = jobj_parts[7
                as usize];
            HSD_JObjReqAnim(
                text_jobj,
                (*(&raw mut mn_803ED1D0.text_start_frames as *mut f32
                    as *mut [f32; 2])
                    .offset(
                        i as isize,
                    ))[(selected as i32 == i) as i32
                    as usize],
            );
            HSD_JObjAnim(text_jobj);
            if selected as i32 != i {
                HSD_JObjSetFlagsAll(
                    jobj_parts[16 as i32 as usize],
                    JOBJ_HIDDEN as u32,
                );
            }
            if selected as i32 != i || i == 5 {
                HSD_JObjSetFlagsAll(
                    jobj_parts[13 as i32 as usize],
                    JOBJ_HIDDEN as u32,
                );
            } else {
                HSD_JObjReqAnimAll(
                    jobj_parts[13 as i32 as usize],
                    mn_803ED1D0.x58.start_frame,
                );
            }
            if selected as i32 != i || i != 5 {
                HSD_JObjSetFlagsAll(
                    jobj_parts[8 as i32 as usize],
                    JOBJ_HIDDEN as u32,
                );
            }
            if i == 5 {
                let mut tmp: *mut AnimLoopSettings = &raw mut (*rule_data).x7C
                    as *mut AnimLoopSettings;
                als = tmp
                    .offset(
                        (i == selected as i32) as i32
                            as isize,
                    ) as *mut AnimLoopSettings;
            } else {
                let mut tmp_0: *mut AnimLoopSettings = &raw mut (*rule_data).x64
                    as *mut AnimLoopSettings;
                als = tmp_0
                    .offset(
                        (i == selected as i32) as i32
                            as isize,
                    ) as *mut AnimLoopSettings;
            }
            HSD_JObjReqAnimAll(
                jobj_parts[2 as i32 as usize],
                (*als).start_frame,
            );
            HSD_JObjAnimAll(jobj_parts[2 as i32 as usize]);
            HSD_JObjAddChild(option_jobj, cursor_jobj);
            if i != 5 {
                match i {
                    0 => {
                        desc = &raw mut MenMainCursorTr01_Top;
                    }
                    1 => {
                        desc = &raw mut MenMainCursorTr02_Top;
                    }
                    2 | 3 => {
                        desc = &raw mut MenMainCursorTr03_Top;
                    }
                    4 => {
                        desc = &raw mut MenMainCursorTr04_Top;
                    }
                    _ => {}
                }
                value_jobj = HSD_JObjLoadJoint((*desc).joint as *mut HSD_Joint);
                HSD_JObjAddAnimAll(
                    value_jobj,
                    (*desc).animjoint,
                    (*desc).matanim_joint,
                    (*desc).shapeanim_joint,
                );
                HSD_JObjReqAnimAll(value_jobj, 0.0_f32);
                HSD_JObjAnimAll(value_jobj);
                j = 0 as i32;
                while j < *sub_count_ptr.offset(i as isize) as i32 {
                    lb_80011E24(value_jobj as *mut HSD_JObj, (&raw mut *(&raw mut (*user_data).x34 as *mut [*mut HSD_JObj; 7])
                            .offset(i as isize) as *mut *mut HSD_JObj)
                            .offset(j as isize) as *mut *mut HSD_JObj, &[j, -1]);
                    j += 1;
                }
                match i {
                    0 => {
                        let mut digit_indices: JObjIndices = mn_804DBE48;
                        let mut index_ptr: *mut u8 = null_mut();
                        index_ptr = &raw mut digit_indices.idx as *mut u8;
                        j = 0 as i32;
                        while j < 4 {
                            let mut num_jobj: *mut HSD_JObj = HSD_JObjLoadJoint(
                                MenMainNmRl_Top.joint as *mut HSD_Joint,
                            );
                            HSD_JObjAddAnimAll(
                                num_jobj,
                                MenMainNmRl_Top.animjoint,
                                MenMainNmRl_Top.matanim_joint,
                                MenMainNmRl_Top.shapeanim_joint,
                            );
                            HSD_JObjAddChild(
                                (*user_data)
                                    .x34[0 as i32 as usize][*index_ptr as usize],
                                num_jobj,
                            );
                            j += 1;
                            index_ptr = index_ptr.offset(1);
                        }
                        mn_802324E4(
                            (*user_data).rule_values.anon_0.time_limit,
                            user_data,
                        );
                    }
                    1 | 2 | 3 | 4 => {
                        let mut val: u8 = (*user_data).rule_values.values[i as usize];
                        let mut default_val: u8 = 0;
                        let mut val_als: *mut AnimLoopSettings = core::ptr::null_mut::<
                            AnimLoopSettings,
                        >();
                        if i as u8 as i32 != 1
                            && i as u8 as i32 != 2
                            && i as u8 as i32 != 3
                            && i as u8 as i32 != 4
                        {
                            val_als = null_mut();
                        } else {
                            default_val = mn_803ED2E8
                                .stat[i as u8 as usize][1 as i32 as usize];
                            if val as i32 == 0 {
                                val_als = (&raw mut mn_803ED270 as *mut AnimLoopSettings)
                                    .offset(default_val as isize) as *mut AnimLoopSettings;
                            } else {
                                val_als = (&raw mut mn_803ED270 as *mut AnimLoopSettings)
                                    .offset(
                                        (val as i32 - 1)
                                            as isize,
                                    ) as *mut AnimLoopSettings;
                            }
                        }
                        HSD_JObjReqAnimAll(value_jobj, (*val_als).end_frame);
                        HSD_JObjAnimAll(value_jobj);
                    }
                    _ => {}
                }
                HSD_JObjAddChild(option_jobj, value_jobj);
            }
        }
        i += 1;
    }
    let mut text: *mut HSD_Text = null_mut();
    let mut desc_idx: u8 = 0;
    confirmed = mn_804A04F0.confirmed_selection;
    SisLib_ClearText(&raw mut (*user_data).description);
    desc_idx = mnRulePlus_GetDescIdx(selected, confirmed);
    text = HSD_SisLib_803A5ACC(
        0,
        1,
        -9.5_f32,
        8.0_f32,
        17.0_f32,
        364.68332_f32,
        76.77544_f32,
    );
    (*user_data).description = text;
    (*text).font_size.x = 0.0521_f32 as f32;
    (*text).font_size.y = (*text).font_size.x;
    HSD_SisLib_803A6368(text, desc_idx as i32);
    return gobj;
}
pub unsafe extern "C" fn mn_802339FC() {
    let mut state: MenuState = MENU_STATE_IDLE;
    let mut think: *mut HSD_GObjProc = null_mut();
    mn_804A04F0.prev_menu = mn_804A04F0.cur_menu;
    match mn_804A04F0.prev_menu as i32 {
        17 => {
            mn_804A04F0.cur_menu = MENU_KIND_RULES_EXTRA as i32
                as MenuKind8;
            state = MENU_STATE_EXIT_TO;
            mn_804A04F0.hovered_selection = 5;
        }
        _ => {
            mn_804A04F0.cur_menu = MENU_KIND_RULES_EXTRA as i32
                as MenuKind8;
            state = MENU_STATE_ENTER_TO;
            mn_804A04F0.hovered_selection = 0;
        }
    }
    mn_80233218(state);
    think = HSD_GObj_SetupProc(
        GObj_Create(0, 1, 0x80),
        core::mem::transmute::<
            Option<unsafe extern "C" fn(*mut HSD_GObj) -> ()>,
            HSD_GObjEvent,
        >(Some(fn_8023201C)),
        0,
    );
    (*think).set_flags_3(HSD_GObj_804D783C as u8);
}
pub const NULL: i32 = 0;
