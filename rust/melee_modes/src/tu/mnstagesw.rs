//! src/melee/mn/mnstagesw.c: the stage-switch menu (Rules > Stage Switch): the per-stage
//! on/off toggles, the 29-stage icon grid, its cursor and the scene builder.
//! Callees in TUs that are not ported yet are `ext` stubs named like the C function.
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, unused_mut, unused_variables, unused_assignments, clippy::missing_safety_doc)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types as mt;
use crate::mn_inl::*;
use core::ffi::{c_char, c_void};
use core::ptr::null_mut;
use melee_core::fp::fmadds;
use melee_types::*;

/// `MnStageSwData` (mnstagesw.c): the menu's user data (0xB4 bytes on the GameCube).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct MnStageSwData {
    pub x0: u8,
    pub x1: u8,
    pub x2: [u8; 29],
    pub x1F: u8,
    pub x20: *mut HSD_JObj,
    pub x24: *mut HSD_JObj,
    pub x28: *mut HSD_JObj,
    pub x2C: *mut HSD_JObj,
    pub x30: *mut HSD_JObj,
    pub x34: *mut HSD_JObj,
    pub pad_38: [u8; 8],
    pub x40: [*mut HSD_Text; 29],
}

melee_core::gc_layout!(MnStageSwData, "MnStageSwData", size 0xB4, align 4, [
    x0 @ 0x0: 1 "u8",
    x1 @ 0x1: 1 "u8",
    x2 @ 0x2: 29 "u8[29]",
    x1F @ 0x1F: 1 "u8",
    x20 @ 0x20: 4 "HSD_JObj*",
    x24 @ 0x24: 4 "HSD_JObj*",
    x28 @ 0x28: 4 "HSD_JObj*",
    x2C @ 0x2C: 4 "HSD_JObj*",
    x30 @ 0x30: 4 "HSD_JObj*",
    x34 @ 0x34: 4 "HSD_JObj*",
    pad_38 @ 0x38: 8 "u8[8]",
    x40 @ 0x40: 116 "HSD_Text*[29]",
]);

pub mod ext {
    use super::*;
    use crate::ext_fns;
    ext_fns! {
        fn GObj_Create(classifier: u16, p_link: u8, priority: u8) -> *mut HSD_GObj;
        extern "C" fn HSD_GObj_JObjCallback(gobj: *mut HSD_GObj, arg1: i32) = crate::tu::adapt::HSD_GObj_JObjCallback__9cd907;
        fn HSD_GObj_80390CD4(gobj: *mut HSD_GObj);
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
        fn mn_802339FC() = crate::tu::mnruleplus::mn_802339FC;
        fn gm_801641CC(a0: u8) -> u16 = crate::gm::gm_801641CC;
        fn gm_801641E4(stage: u8, enable: u8) = crate::tu::gm_1601::gm_801641E4;
        fn gm_IsStageUnlocked(stkind: u16) -> i32 = crate::mn_inl::gm_IsStageUnlocked_i32;
        fn gm_80164430(a0: u16) -> i32 = crate::mn_inl::gm_80164430_i32;
        fn gm_GetCurrentGameMode() -> u8 = crate::tu::adapt::gm_GetCurrentGameMode__f56b36;
        fn lbCardGame_SaveChanges() = crate::tu::adapt::lbCardGame_SaveChanges__0fc9e6;
        fn GObj_SetupGXLink(gobj: *mut HSD_GObj, render_cb: GObj_RenderFunc, gx_link: u8, priority: u32);
        fn HSD_GObjObject_80390A70(gobj: *mut HSD_GObj, kind: u8, obj: *mut c_void);
        fn HSD_GObjFree(a0: *mut HSD_GObj);
        fn HSD_GObj_SetupProc(a0: *mut HSD_GObj, a1: HSD_GObjEvent, a2: u8) -> *mut HSD_GObjProc;
        fn GObj_InitUserData(gobj: *mut HSD_GObj, kind: u8, remove_func: Option<unsafe extern "C" fn(*mut c_void) -> ()>, data: *mut c_void, );
        extern "C" fn HSD_Free(ptr: *mut c_void) = crate::tu::adapt::HSD_Free__3ee311;
        fn HSD_MemAlloc(size: isize) -> *mut c_void = crate::tu::adapt::HSD_MemAlloc__849f1b;
    }
    pub use melee_runtime::tu::gobj_native::HSD_GObj_804D783C;
    pub use melee_runtime::tu::gobj_native::HSD_GObj_JObjKind;
    pub use crate::tu::mnmain::MenMainConSs_Top;
    pub use crate::tu::mnmain::MenMainCursorSs_Top;
}
use ext::*;

pub const NUM_STAGES: i32 = 29;
pub static mut mnStageSw_803ED488: [AnimLoopSettings; 5] = [
    AnimLoopSettings {
        start_frame: 0.0_f32,
        end_frame: 199.0_f32,
        loop_frame: 0.0_f32,
    },
    AnimLoopSettings {
        start_frame: 0.0_f32,
        end_frame: 9.0_f32,
        loop_frame: -0.1_f32,
    },
    AnimLoopSettings {
        start_frame: 0.0_f32,
        end_frame: 0.0_f32,
        loop_frame: -0.1_f32,
    },
    AnimLoopSettings {
        start_frame: 0.0_f32,
        end_frame: 0.0_f32,
        loop_frame: -0.1_f32,
    },
    AnimLoopSettings {
        start_frame: 0.0_f32,
        end_frame: 0.0_f32,
        loop_frame: -0.1_f32,
    },
];
static mut mnStageSw_803ED4C4: [u8; 29] = [
    0 as i32 as u8,
    0x1 as i32 as u8,
    0x2 as i32 as u8,
    0x3 as i32 as u8,
    0x4 as i32 as u8,
    0x5 as i32 as u8,
    0x6 as i32 as u8,
    0x7 as i32 as u8,
    0x8 as i32 as u8,
    0x9 as i32 as u8,
    0xb as i32 as u8,
    0xa as i32 as u8,
    0x18 as i32 as u8,
    0x1a as i32 as u8,
    0x1c as i32 as u8,
    0xc as i32 as u8,
    0xd as i32 as u8,
    0xe as i32 as u8,
    0xf as i32 as u8,
    0x10 as i32 as u8,
    0x11 as i32 as u8,
    0x12 as i32 as u8,
    0x13 as i32 as u8,
    0x14 as i32 as u8,
    0x15 as i32 as u8,
    0x16 as i32 as u8,
    0x17 as i32 as u8,
    0x19 as i32 as u8,
    0x1b as i32 as u8,
];
static mut mnStageSw_stageIcons: [u8; 29] = [
    0x8 as i32 as u8,
    0x9 as i32 as u8,
    0x11 as i32 as u8,
    0xa as i32 as u8,
    0xc as i32 as u8,
    0x6 as i32 as u8,
    0xb as i32 as u8,
    0x7 as i32 as u8,
    0xe as i32 as u8,
    0xd as i32 as u8,
    0x1d as i32 as u8,
    0x17 as i32 as u8,
    0xf as i32 as u8,
    0x10 as i32 as u8,
    0x12 as i32 as u8,
    0x13 as i32 as u8,
    0x14 as i32 as u8,
    0x15 as i32 as u8,
    0x1a as i32 as u8,
    0x1b as i32 as u8,
    0x1c as i32 as u8,
    0x16 as i32 as u8,
    0x18 as i32 as u8,
    0x1f as i32 as u8,
    0x20 as i32 as u8,
    0x21 as i32 as u8,
    0x22 as i32 as u8,
    0x23 as i32 as u8,
    0x24 as i32 as u8,
];
static mut mnStageSw_804D4BB8: [f32; 2] = [0.0_f32, 1.0_f32];
static mut mnStageSw_804D6BF0: *mut HSD_GObj = null_mut();
static mut mnStageSw_804D6BF4: i8 = 0;
unsafe extern "C" fn mnStageSw_8023593C(mut gobj: *mut HSD_GObj) {
    let mut i: i32 = 0;
    let mut user_data: *mut u8 = (*gobj).user_data as *mut u8;
    let mut p: *mut u8 = null_mut();
    i = 0 as i32;
    while i < NUM_STAGES {
        p = (&raw mut mnStageSw_803ED4C4 as *mut u8)
            .offset(i as u8 as i32 as isize);
        if gm_80164430(gm_801641CC(*p)) != 0 {
            gm_801641E4(
                mnStageSw_803ED4C4[i as usize],
                *user_data
                    .offset((i as i32 + 2) as isize),
            );
        }
        i += 1;
    }
}
unsafe extern "C" fn mnStageSw_802359C8(mut data: *mut MnStageSwData) {
    let mut start_y: f32 = 0.;
    let mut text: *mut HSD_Text = null_mut();
    let mut delta_y: f32 = 0.;
    let mut step_y: f32 = 0.;
    let mut i: i32 = 0;
    start_y = -1.6_f32 + HSD_JObjGetTranslationY((*data).x2C);
    step_y = HSD_JObjGetTranslationY((*data).x2C);
    delta_y = HSD_JObjGetTranslationY((*data).x30);
    delta_y -= step_y;
    i = 0 as i32;
    while i < 15 {
        text = HSD_SisLib_803A5ACC(
            0,
            mn_804D6BB5 as i32,
            1.0_f32 + HSD_JObjGetTranslationX((*data).x2C),
            -fmadds(delta_y, i as f32, start_y),
            17.5_f32,
            160.0_f32,
            300.0_f32,
        );
        (*data).x40[i as usize] = text;
        (*text).font_size.x = 0.0521_f32 as f32;
        (*text).font_size.y = 0.0521_f32 as f32;
        (*text).default_alignment = 2;
        (*text).default_fitting = 1;
        if gm_80164430(gm_801641CC(mnStageSw_803ED4C4[i as u8 as usize]))
            != 0
        {
            HSD_SisLib_803A6368(
                text,
                mnStageSw_stageIcons[mnStageSw_803ED4C4[i as usize] as usize] as i32,
            );
        } else {
            HSD_SisLib_803A6368(text, 0x25);
        }
        i += 1;
    }
    i = 15 as i32;
    while i < NUM_STAGES {
        text = HSD_SisLib_803A5ACC(
            0,
            mn_804D6BB5 as i32,
            1.0_f32 + HSD_JObjGetTranslationX((*data).x34),
            -fmadds(delta_y, (i as i32 - 15) as f32, start_y),
            17.5_f32,
            160.0_f32,
            300.0_f32,
        );
        (*data).x40[i as usize] = text;
        (*text).font_size.x = 0.0521_f32 as f32;
        (*text).font_size.y = 0.0521_f32 as f32;
        (*text).default_alignment = 2;
        (*text).default_fitting = 1;
        if gm_80164430(gm_801641CC(mnStageSw_803ED4C4[i as u8 as usize]))
            != 0
        {
            HSD_SisLib_803A6368(
                text,
                mnStageSw_stageIcons[mnStageSw_803ED4C4[i as usize] as usize] as i32,
            );
        } else {
            HSD_SisLib_803A6368(text, 0x25);
        }
        i += 1;
    }
}
unsafe extern "C" fn mnStageSw_80235C58(mut arg0: u8) -> i32 {
    let mut current_block: u64;
    let mut next: i32 = 0;
    let mut found: i32 = 0;
    let mut temp: i32 = 0;
    let mut curr: i32 = 0;
    let mut low_bound: i32 = 0;
    let mut high_bound: i32 = 0;
    let mut idx: i32 = 0;
    let mut low: u8 = 0;
    let mut end: u8 = 0;
    let mut start: u8 = 0;
    let mut high: u8 = 0;
    let mut prev: i32 = 0;
    let mut i: i32 = 0;
    if (arg0 as i32) < 15 {
        low = 0;
        high = 14;
    } else {
        low = 15;
        high = 28;
    }
    if (arg0 as i32) < 15 {
        start = 0;
        end = 14;
    } else {
        start = 15;
        end = 28;
    }
    curr = start as i32;
    next = end as i32;
    loop {
        if !(curr <= next) {
            current_block = 12147880666119273379;
            break;
        }
        if gm_80164430(gm_801641CC(mnStageSw_803ED4C4[curr as u8 as usize]))
            != 0
        {
            found = 0 as i32;
            current_block = 4502130264477864322;
            break;
        } else {
            curr += 1;
        }
    }
    match current_block {
        12147880666119273379 => {
            found = 1 as i32;
        }
        _ => {}
    }
    if found != 0 {
        return -1;
    }
    if (arg0 as i32) < NUM_STAGES
        && gm_80164430(gm_801641CC(mnStageSw_803ED4C4[arg0 as usize]))
            != 0
    {
        return arg0 as i32;
    }
    idx = arg0 as i32;
    next = (idx as i32 + 1) as i32;
    found = 1 as i32;
    curr = idx + found;
    low_bound = low as i32;
    high_bound = high as i32;
    i = 1 as i32;
    while found != 0 {
        temp = idx - i;
        prev = temp;
        if low_bound <= prev
            && gm_80164430(gm_801641CC(mnStageSw_803ED4C4[prev as u8 as usize]))
                != 0
        {
            return prev;
        }
        if next <= high_bound
            && gm_80164430(gm_801641CC(mnStageSw_803ED4C4[curr as u8 as usize]))
                != 0
        {
            return idx + i;
        }
        next += 1;
        curr += 1;
        i += 1;
    }
    panic!("Reached end of non-void function without returning");
}
unsafe extern "C" fn mnStageSw_80235DC8(mut user_data: *const u8, mut buttons: i32) {
    let mut selection: i32 = 0;
    let mut idx: u8 = 0;
    idx = mn_804A04F0.hovered_selection as u8;
    if buttons as i32 & 1 != 0 {
        loop {
            match idx as i32 {
                0 => {
                    mn_804A04F0.hovered_selection = 14;
                }
                15 => {
                    mn_804A04F0.hovered_selection = 28;
                }
                _ => {
                    mn_804A04F0.hovered_selection = (idx as i32
                        - 1) as u16;
                }
            }
            idx = mn_804A04F0.hovered_selection as u8;
            if !(gm_80164430(gm_801641CC(mnStageSw_803ED4C4[idx as usize]))
                == 0)
            {
                break;
            }
        }
        mn_804A04F0.confirmed_selection = *user_data
            .offset((idx as i32 + 2) as isize);
        return;
    }
    if buttons as i32 & 2 != 0 {
        loop {
            match idx as i32 {
                14 => {
                    mn_804A04F0.hovered_selection = 0;
                }
                28 => {
                    mn_804A04F0.hovered_selection = 15;
                }
                _ => {
                    mn_804A04F0.hovered_selection = (idx as i32
                        + 1) as u16;
                }
            }
            idx = mn_804A04F0.hovered_selection as u8;
            if !(gm_80164430(gm_801641CC(mnStageSw_803ED4C4[idx as usize]))
                == 0)
            {
                break;
            }
        }
        mn_804A04F0.confirmed_selection = *user_data
            .offset((idx as i32 + 2) as isize);
        return;
    }
    if buttons as i32 & 4 != 0 {
        if idx as i32 >= 15
            && (idx as i32) < NUM_STAGES
        {
            selection = mnStageSw_80235C58(
                (idx as i32 - 15) as u8,
            );
            if selection != -1 {
                mn_804A04F0.hovered_selection = selection as u8 as u16;
                mn_804A04F0.confirmed_selection = *user_data
                    .offset(
                        (selection as u8 as i32
                            + 2) as isize,
                    );
            }
        }
    } else if buttons as i32 & 8 != 0
        && (idx as i32) < 15
    {
        selection = mnStageSw_80235C58(
            (idx as i32 + 15) as u8,
        );
        if selection != -1 {
            mn_804A04F0.hovered_selection = selection as u8 as u16;
            mn_804A04F0.confirmed_selection = *user_data
                .offset(
                    (selection as u8 as i32 + 2)
                        as isize,
                );
        }
    }
}
unsafe extern "C" fn mnStageSw_CountEnabled(mut user_data: *const u8) -> i32 {
    let mut i: i32 = 0;
    let mut count: i32 = 0;
    i = 0 as i32;
    while i < NUM_STAGES {
        if *user_data
            .offset((i as i32 + 2) as isize)
            as i32 != 0
        {
            count += 1;
        }
        i += 1;
    }
    return count;
}
unsafe extern "C" fn fn_80235F80(mut gobj: *mut HSD_GObj) {
    let mut i: i32 = 0;
    let mut stage_ids: *mut u8 = null_mut();
    let mut enabled: i32 = 0;
    let mut result: i32 = 0;
    let mut buttons: u32 = 0;
    let mut count: i32 = 0;
    let mut user_data: *mut u8 = null_mut();
    let mut confirmed: *mut u8 = null_mut();
    user_data = (*mnStageSw_804D6BF0).user_data as *mut u8;
    mn_804A04F0.buttons = mn_80229624(4) as u64;
    buttons = mn_804A04F0.buttons as u32;
    if buttons & 0x20 != 0 {
        sfxBack();
        mn_804A04F0.entering_menu = 0;
        mnStageSw_8023593C(mnStageSw_804D6BF0);
        lbCardGame_SaveChanges();
        mn_804D6BC8.cooldown = 5;
        mn_802339FC();
        HSD_GObjFree(gobj);
        return;
    }
    if mnStageSw_804D6BF4 as u8 as i32 == 0 {
        if buttons & 0x200 != 0 {
            if (mn_804A04F0.hovered_selection as i32) < NUM_STAGES {
                confirmed = &raw mut mn_804A04F0.confirmed_selection;
                if *confirmed as i32 != 0 {
                    user_data = (*mnStageSw_804D6BF0).user_data as *mut u8;
                    count = mnStageSw_CountEnabled(user_data);
                    if count > 1 {
                        enabled = 0 as i32;
                    } else {
                        enabled = 1 as i32;
                    }
                    if enabled != 0 {
                        lbAudioAx_80024030(3);
                    } else {
                        sfxMove();
                        *confirmed = 0;
                    }
                } else {
                    sfxMove();
                    *confirmed = 1;
                }
                user_data = (*mnStageSw_804D6BF0).user_data as *mut u8;
                stage_ids = &raw mut mnStageSw_803ED4C4 as *mut u8;
                i = 0 as i32;
                while i < NUM_STAGES {
                    if gm_80164430(gm_801641CC(mnStageSw_803ED4C4[i as u8 as usize]))
                        != 0
                    {
                        gm_801641E4(
                            *stage_ids,
                            *user_data
                                .offset(
                                    (i as i32 + 2) as isize,
                                ),
                        );
                    }
                    i += 1;
                    stage_ids = stage_ids.offset(1);
                }
                return;
            }
        } else if buttons & 0x100 != 0 {
            sfxForward();
            result = gm_GetCurrentGameMode() as i32;
            match result {
                1 => {
                    mnStageSw_8023593C(mnStageSw_804D6BF0);
                    lbCardGame_SaveChanges();
                    mn_80229860(GM_VS as i32 as i8);
                    return;
                }
                _ => {
                    mnStageSw_8023593C(mnStageSw_804D6BF0);
                    lbCardGame_SaveChanges();
                    mn_8022F4CC();
                    return;
                }
            }
        }
        if buttons & 0xf != 0 {
            sfxMove();
            mnStageSw_80235DC8(user_data, buttons as i32);
        }
    }
}
unsafe extern "C" fn mnStageSw_80236178(mut data: *mut MnStageSwData, mut idx: u8) {
    let mut jobj: *mut HSD_JObj = null_mut();
    let mut ref_jobj: *mut HSD_JObj = null_mut();
    let mut delta: f32 = 0.;
    jobj = (*data).x28;
    HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);
    delta = HSD_JObjGetTranslationY((*data).x30) - HSD_JObjGetTranslationY((*data).x2C);
    if (idx as i32) < 15 {
        ref_jobj = (*data).x2C;
        HSD_JObjSetTranslateX(jobj, HSD_JObjGetTranslationX(ref_jobj));
        HSD_JObjSetTranslateY(
            jobj,
            fmadds(delta, idx as f32, HSD_JObjGetTranslationY((*data).x2C)),
        );
    } else {
        ref_jobj = (*data).x34;
        HSD_JObjSetTranslateX(jobj, HSD_JObjGetTranslationX(ref_jobj));
        HSD_JObjSetTranslateY(
            jobj,
            fmadds(delta, (idx as i32 - 15) as f32, HSD_JObjGetTranslationY((*data).x2C)),
        );
    };
}
unsafe extern "C" fn mnStageSw_802364A0_noinline(
    mut data: *mut MnStageSwData,
    mut idx: u8,
) -> *mut HSD_JObj {
    return mnStageSw_802364A0(data, idx);
}
unsafe extern "C" fn mnStageSw_802364A0(
    mut data: *mut MnStageSwData,
    mut idx: u8,
) -> *mut HSD_JObj {
    let mut jobj: *mut HSD_JObj = null_mut();
    let mut i: u8 = 0;
    if idx as i32 >= 15 {
        jobj = HSD_JObjGetChild((*data).x34);
        i = 15;
        while (i as i32) < idx as i32 {
            jobj = HSD_JObjGetNext(jobj);
            i = i.wrapping_add(1);
        }
        return jobj;
    }
    jobj = HSD_JObjGetChild((*data).x2C);
    i = 0;
    while (i as i32) < idx as i32 {
        jobj = HSD_JObjGetNext(jobj);
        i = i.wrapping_add(1);
    }
    return jobj;
}
unsafe extern "C" fn mnStageSw_80236548(
    mut gobj: *mut HSD_GObj,
    mut arg1: u8,
    mut arg2: u8,
) {
    let mut hover_anim_jobj: *mut HSD_JObj = null_mut();
    let mut cursor_jobj: *mut HSD_JObj = null_mut();
    let mut jobj: *mut HSD_JObj = null_mut();
    let mut data: *mut MnStageSwData = null_mut();
    let mut delta_y: f32 = 0.;
    let mut frame: f32 = 0.;
    let mut x: f32 = 0.;
    let mut hovered: u16 = 0;
    let mut sel: u8 = 0;
    data = (*gobj).user_data as *mut MnStageSwData;
    if arg1 as i32 != 0 {
        jobj = mnStageSw_802364A0_noinline(data, (*data).x1);
        lb_80011E24(jobj as *mut HSD_JObj, &raw mut hover_anim_jobj, &[3, -1]);
        HSD_JObjSetFlagsAll(hover_anim_jobj, JOBJ_HIDDEN as u32);
        frame = mn_8022F298(hover_anim_jobj as *mut HSD_JObj);
        sel = mn_804A04F0.hovered_selection as u8;
        jobj = mnStageSw_802364A0_noinline(data, sel);
        lb_80011E24(jobj as *mut HSD_JObj, &raw mut hover_anim_jobj, &[3, -1]);
        HSD_JObjClearFlagsAll(hover_anim_jobj, JOBJ_HIDDEN as u32);
        HSD_JObjReqAnimAll(hover_anim_jobj, frame);
        HSD_JObjAnimAll(hover_anim_jobj);
        cursor_jobj = (*data).x28;
        HSD_JObjClearFlagsAll(cursor_jobj, JOBJ_HIDDEN as u32);
        delta_y = HSD_JObjGetTranslationY((*data).x30)
            - HSD_JObjGetTranslationY((*data).x2C);
        if (sel as i32) < 15 {
            x = HSD_JObjGetTranslationX((*data).x2C);
            HSD_JObjSetTranslateX(cursor_jobj, x);
            HSD_JObjSetTranslateY(
                cursor_jobj,
                fmadds(delta_y, sel as f32, HSD_JObjGetTranslationY((*data).x2C)),
            );
        } else {
            x = HSD_JObjGetTranslationX((*data).x34);
            HSD_JObjSetTranslateX(cursor_jobj, x);
            HSD_JObjSetTranslateY(
                cursor_jobj,
                fmadds(delta_y, (sel as i32 - 15) as f32, HSD_JObjGetTranslationY((*data).x2C)),
            );
        }
    }
    if arg2 as i32 != 0 {
        let mut sp3C: *mut HSD_JObj = null_mut();
        arg2 = mn_804A04F0.confirmed_selection;
        jobj = mnStageSw_802364A0_noinline(data, mn_804A04F0.hovered_selection as u8);
        lb_80011E24(jobj as *mut HSD_JObj, &raw mut sp3C, &[2, -1]);
        HSD_JObjReqAnimAll(sp3C, mnStageSw_804D4BB8[arg2 as usize]);
        HSD_JObjAnimAll(sp3C);
    }
    if arg1 as i32 != 0 {
        hovered = mn_804A04F0.hovered_selection;
    } else {
        hovered = (*data).x1 as u16;
    }
    sel = hovered as u8;
    lb_80011E24(mnStageSw_802364A0_noinline(data, sel) as *mut HSD_JObj, &raw mut hover_anim_jobj, &[3, -1]);
    mn_8022ED6C(
        hover_anim_jobj as *mut HSD_JObj,
        &raw mut mnStageSw_803ED488 as *mut AnimLoopSettings,
    );
}
unsafe extern "C" fn mnStageSw_FinishEnter(
    mut menu_gobj: *mut HSD_GObj,
    mut data: *mut MnStageSwData,
    mut child: *mut *mut HSD_JObj,
) {
    let mut jobj: *mut HSD_JObj = null_mut();
    let mut i: i32 = 0;
    i = 0 as i32;
    (*data).x1F = i as u8;
    mnStageSw_802359C8(data);
    menu_gobj = (*menu_gobj).user_data as *mut HSD_GObj;
    HSD_JObjClearFlagsAll(
        (*(menu_gobj as *mut MnStageSwData)).x2C,
        JOBJ_HIDDEN as u32,
    );
    HSD_JObjClearFlagsAll(
        (*(menu_gobj as *mut MnStageSwData)).x34,
        JOBJ_HIDDEN as u32,
    );
    while i < NUM_STAGES {
        jobj = mnStageSw_802364A0(menu_gobj as *mut MnStageSwData, i as u8);
        if i != (*(menu_gobj as *mut MnStageSwData)).x1 as i32 {
            lb_80011E24(jobj as *mut HSD_JObj, child as *mut *mut HSD_JObj, &[3, -1]);
            HSD_JObjSetFlagsAll(*child, JOBJ_HIDDEN as u32);
        }
        i += 1;
    }
    mnStageSw_80236178(
        menu_gobj as *mut MnStageSwData,
        (*(menu_gobj as *mut MnStageSwData)).x1,
    );
    mnStageSw_804D6BF4 = 0;
}
unsafe extern "C" fn mnStageSw_FreeTexts(mut data: *mut MnStageSwData) {
    let mut i: i32 = 0;
    i = 0 as i32;
    while i < NUM_STAGES {
        HSD_SisLib_803A5CC4((*data).x40[i as usize]);
        i += 1;
    }
}
unsafe extern "C" fn fn_80236998(mut gobj: *mut HSD_GObj) {
    let mut jobj: *mut HSD_JObj = null_mut();
    let mut current_data: *mut MnStageSwData = (*gobj).user_data as *mut MnStageSwData;
    let mut data: *mut MnStageSwData = current_data;
    let mut anims: *mut AnimLoopSettings = null_mut();
    let mut changed_menu: i32 = 0;
    let mut changed_hovered: i32 = 0;
    let mut pad3: u64 = 0;
    let mut child: *mut HSD_JObj = null_mut();
    let mut changed_confirmed: i32 = 0;
    let mut state: u8 = 0;
    changed_menu = 0 as i32;
    changed_hovered = 0 as i32;
    changed_confirmed = 0 as i32;
    state = (*data).x1F;
    if (state as i32 == 0
        || state as i32 == 1
        || state as i32 == 3)
        && (*data).x0 as i32 != mn_804A04F0.cur_menu as i32
    {
        if mn_804A04F0.entering_menu as i32 != 0 {
            (*data).x1F = 4;
        } else {
            (*data).x1F = 2;
        }
        match (*data).x1F as i32 {
            1 => {
                anims = (&raw mut mnStageSw_803ED488 as *mut AnimLoopSettings)
                    .offset(1 as i32 as isize) as *mut AnimLoopSettings;
            }
            2 => {
                anims = (&raw mut mnStageSw_803ED488 as *mut AnimLoopSettings)
                    .offset(3 as i32 as isize) as *mut AnimLoopSettings;
            }
            3 => {
                anims = (&raw mut mnStageSw_803ED488 as *mut AnimLoopSettings)
                    .offset(2 as i32 as isize) as *mut AnimLoopSettings;
            }
            4 => {
                anims = (&raw mut mnStageSw_803ED488 as *mut AnimLoopSettings)
                    .offset(4 as i32 as isize) as *mut AnimLoopSettings;
            }
            _ => {}
        }
        jobj = (*data).x24;
        HSD_JObjReqAnim(jobj, (*anims).start_frame);
        HSD_JObjAnim(jobj);
        state = (*data).x1F;
        if state as i32 == 0
            || state as i32 == 1
            || state as i32 == 3
        {
            changed_menu = 1 as i32;
            changed_hovered = 1 as i32;
            changed_confirmed = 1 as i32;
        }
    }
    state = (*data).x1F;
    if state as i32 != 0 {
        jobj = (*data).x24;
        match state as i32 {
            1 => {
                anims = (&raw mut mnStageSw_803ED488 as *mut AnimLoopSettings)
                    .offset(1 as i32 as isize) as *mut AnimLoopSettings;
            }
            2 => {
                anims = (&raw mut mnStageSw_803ED488 as *mut AnimLoopSettings)
                    .offset(3 as i32 as isize) as *mut AnimLoopSettings;
            }
            3 => {
                anims = (&raw mut mnStageSw_803ED488 as *mut AnimLoopSettings)
                    .offset(2 as i32 as isize) as *mut AnimLoopSettings;
            }
            4 => {
                anims = (&raw mut mnStageSw_803ED488 as *mut AnimLoopSettings)
                    .offset(4 as i32 as isize) as *mut AnimLoopSettings;
            }
            _ => {}
        }
        if mn_8022F298(jobj as *mut HSD_JObj) >= (*anims).end_frame {
            match (*data).x1F as i32 {
                1 | 3 => {
                    mnStageSw_FinishEnter(gobj, data, &raw mut child);
                    return;
                }
                2 | 4 => {
                    mnStageSw_FreeTexts(data);
                    HSD_GObjFree(gobj);
                    return;
                }
                _ => {}
            }
        }
        HSD_JObjAnim(jobj);
    }
    state = (*data).x1F;
    if state as i32 == 0
        || state as i32 == 1
        || state as i32 == 3
    {
        if (*data).x1 as i32 != mn_804A04F0.hovered_selection as i32 {
            changed_hovered = 1 as i32;
        }
        if (*data).x2[mn_804A04F0.hovered_selection as usize] as i32
            != mn_804A04F0.confirmed_selection as i32
        {
            changed_confirmed = 1 as i32;
        }
    }
    mnStageSw_80236548(gobj, changed_hovered as u8, changed_confirmed as u8);
    if changed_menu != 0 {
        (*data).x0 = mn_804A04F0.cur_menu as u8;
    }
    if changed_hovered != 0 {
        (*data).x1 = mn_804A04F0.hovered_selection as u8;
    }
    if changed_confirmed != 0 {
        (*data).x2[mn_804A04F0.hovered_selection as usize] = mn_804A04F0
            .confirmed_selection;
    }
}
unsafe extern "C" fn mnStageSw_SetCursorPosition(mut user_data: *mut MnStageSwData) {
    let mut hovered: u8 = 0;
    let mut y_spacing: f32 = 0.;
    let mut cursor: *mut HSD_JObj = null_mut();
    HSD_JObjSetFlagsAll((*user_data).x34, JOBJ_HIDDEN as u32);
    cursor = (*user_data).x28;
    hovered = (*user_data).x1;
    HSD_JObjClearFlagsAll(cursor, JOBJ_HIDDEN as u32);
    y_spacing = HSD_JObjGetTranslationY((*user_data).x30)
        - HSD_JObjGetTranslationY((*user_data).x2C);
    if (hovered as i32) < 15 {
        HSD_JObjSetTranslateX(cursor, HSD_JObjGetTranslationX((*user_data).x2C));
        HSD_JObjSetTranslateY(
            cursor,
            fmadds(y_spacing, hovered as f32, HSD_JObjGetTranslationY((*user_data).x2C)),
        );
    } else {
        HSD_JObjSetTranslateX(cursor, HSD_JObjGetTranslationX((*user_data).x34));
        HSD_JObjSetTranslateY(
            cursor,
            fmadds(y_spacing, (hovered as i32 - 15) as f32, HSD_JObjGetTranslationY((*user_data).x2C)),
        );
    }
    HSD_JObjSetFlagsAll((*user_data).x28, JOBJ_HIDDEN as u32);
}
unsafe extern "C" fn mnStageSw_InitUserData(
    mut user_data: *mut MnStageSwData,
    mut state: i8,
) {
    let mut i: i32 = 0;
    (*user_data).x0 = mn_804A04F0.cur_menu as u8;
    (*user_data).x1 = mn_804A04F0.hovered_selection as u8;
    (*user_data).x1F = state as u8;
    i = 0 as i32;
    while (i as u8 as i32) < NUM_STAGES {
        if gm_80164430(gm_801641CC(mnStageSw_803ED4C4[i as u8 as usize]))
            != 0
        {
            (*user_data).x2[i as u8 as usize] = gm_IsStageUnlocked(
                mnStageSw_803ED4C4[i as usize] as u16,
            ) as u8;
        } else {
            (*user_data).x2[i as u8 as usize] = 0;
        }
        i += 1;
    }
}
unsafe extern "C" fn mnStageSw_SetCursorAnim(
    mut index: i32,
    mut user_data: *mut MnStageSwData,
    mut cursor_jobj: *mut HSD_JObj,
    mut cursor_index: *mut u32,
    mut cursor_anim_jobj: *mut *mut HSD_JObj,
) {
    let mut enabled: u8 = 0;
    *cursor_index = index as u8 as u32;
    enabled = (*user_data).x2[*cursor_index as usize];
    lb_80011E24(cursor_jobj as *mut HSD_JObj, cursor_anim_jobj as *mut *mut HSD_JObj, &[2, -1]);
    HSD_JObjReqAnimAll(*cursor_anim_jobj, mnStageSw_804D4BB8[enabled as usize]);
}
unsafe extern "C" fn mnStageSw_CreateCursor(
    mut user_data: *mut MnStageSwData,
    mut index: i32,
    mut anims: *mut AnimLoopSettings,
) -> *mut HSD_JObj {
    let mut cursor_jobj: *mut HSD_JObj = null_mut();
    let mut cursor_anim_jobj: *mut HSD_JObj = null_mut();
    let mut hover_anim_jobj: *mut HSD_JObj = null_mut();
    let mut hovered: u8 = 0;
    let mut idx: u32 = 0;
    hovered = mn_804A04F0.hovered_selection as u8;
    cursor_jobj = HSD_JObjLoadJoint(MenMainCursorSs_Top.joint as *mut HSD_Joint);
    HSD_JObjAddAnimAll(
        cursor_jobj,
        MenMainCursorSs_Top.animjoint,
        MenMainCursorSs_Top.matanim_joint,
        MenMainCursorSs_Top.shapeanim_joint,
    );
    mnStageSw_SetCursorAnim(
        index,
        user_data,
        cursor_jobj,
        &raw mut idx,
        &raw mut cursor_anim_jobj,
    );
    HSD_JObjAnimAll(cursor_anim_jobj);
    lb_80011E24(cursor_jobj as *mut HSD_JObj, &raw mut hover_anim_jobj, &[3, -1]);
    if idx == hovered as u32 {
        HSD_JObjReqAnimAll(
            hover_anim_jobj,
            (*anims.offset(0 as i32 as isize)).start_frame,
        );
        HSD_JObjAnimAll(hover_anim_jobj);
    } else {
        HSD_JObjSetFlagsAll(hover_anim_jobj, JOBJ_HIDDEN as u32);
    }
    if gm_80164430(gm_801641CC(mnStageSw_803ED4C4[idx as usize]))
        == 0
    {
        HSD_JObjSetFlagsAll(cursor_jobj, JOBJ_HIDDEN as u32);
    }
    return cursor_jobj;
}
#[cfg_attr(fptrace, melee_fptrace::trace)]
unsafe extern "C" fn mnStageSw_80236CBC(mut arg0: i8) -> *mut HSD_GObj {
    let mut gobj: *mut HSD_GObj = null_mut();
    let mut jobj: *mut HSD_JObj = null_mut();
    let mut user_data: *mut MnStageSwData = null_mut();
    let mut anims: *mut AnimLoopSettings = &raw mut mnStageSw_803ED488
        as *mut AnimLoopSettings;
    let mut mdl: *mut StaticModelDesc = &raw mut MenMainConSs_Top;
    let mut y_spacing: f32 = 0.;
    let mut i: i32 = 0;
    gobj = GObj_Create(6, 7, 0x80);
    mnStageSw_804D6BF0 = gobj;
    jobj = HSD_JObjLoadJoint((*mdl).joint as *mut HSD_Joint);
    HSD_GObjObject_80390A70(gobj, HSD_GObj_JObjKind, jobj as *mut c_void);
    GObj_SetupGXLink(
        gobj,
        Some(
            HSD_GObj_JObjCallback
               ,
        ),
        6,
        0x80,
    );
    HSD_GObj_SetupProc(
        gobj,
        Some(fn_80236998),
        0,
    );
    HSD_JObjAddAnimAll(
        jobj,
        (*mdl).animjoint,
        (*mdl).matanim_joint,
        (*mdl).shapeanim_joint,
    );
    HSD_JObjReqAnimAll(jobj, 0.0_f32);
    HSD_JObjAnimAll(jobj);
    user_data = HSD_MemAlloc(core::mem::size_of::<MnStageSwData>() as isize)
        as *mut MnStageSwData;
    if user_data.is_null() {
        OSReport(
            c"Can't get user_data.\n".as_ptr()
                as *mut c_char,
        );
        __assert("src/melee/mn/mnstagesw.c", 758, "user_data");
    };
    GObj_InitUserData(
        gobj,
        0,
        Some(HSD_Free),
        user_data as *mut c_void,
    );
    mnStageSw_InitUserData(user_data, arg0);
    i = 0 as i32;
    while i < 6 {
        lb_80011E24(jobj as *mut HSD_JObj, (&raw mut (*user_data).x20).offset(i as isize), &[i, -1]);
        i += 1;
    }
    mn_804A04F0.confirmed_selection = (*user_data).x2[(*user_data).x1 as usize];
    y_spacing = HSD_JObjGetTranslationY((*user_data).x30)
        - HSD_JObjGetTranslationY((*user_data).x2C);
    i = 0 as i32;
    while i < NUM_STAGES {
        let mut cursor_jobj: *mut HSD_JObj = null_mut();
        cursor_jobj = mnStageSw_CreateCursor(user_data, i, anims);
        if i < 15 {
            HSD_JObjAddChild((*user_data).x2C, cursor_jobj);
            HSD_JObjAddTranslationY(
                cursor_jobj,
                y_spacing as f32 * i as f32,
            );
        } else {
            let mut parent: *mut HSD_JObj = (*user_data).x34;
            HSD_JObjAddChild(parent, cursor_jobj);
            HSD_JObjAddTranslationY(
                cursor_jobj,
                y_spacing as f32
                    * (i as i32 - 15)
                        as f32,
            );
        }
        i += 1;
    }
    HSD_JObjSetFlagsAll((*user_data).x2C, JOBJ_HIDDEN as u32);
    mnStageSw_SetCursorPosition(user_data);
    return gobj;
}
pub unsafe extern "C" fn mnStageSw_80237410() {
    melee_core::ct!("mnStageSw_80237410");
    let mut gobj: *mut HSD_GObj = null_mut();
    let mut proc_0: *mut HSD_GObjProc = null_mut();
    mn_804A04F0.prev_menu = mn_804A04F0.cur_menu;
    mn_804A04F0.cur_menu = 0x11 as MenuKind8;
    mn_804A04F0.hovered_selection = mnStageSw_80235C58(0) as u16;
    mnStageSw_804D6BF4 = 1;
    gobj = mnStageSw_80236CBC(1);
    HSD_GObj_80390CD4(gobj);
    gobj = GObj_Create(0, 1, 0x80);
    proc_0 = HSD_GObj_SetupProc(
        gobj,
        Some(fn_80235F80),
        0,
    );
    (*proc_0).set_flags_3(HSD_GObj_804D783C as u8);
}
pub const NULL: i32 = 0;

// ---- folded from mn_ptr.rs (consolidation steps 21/22) ----
/// mnstagesw.c `mnStageSw_CountEnabled_fp` (NUM_STAGES = 29 flags from byte 2).
pub fn mnStageSw_CountEnabled_fp(user_data: &[u8]) -> i32 {
    melee_core::ct!("mnStageSw_CountEnabled_fp");
    user_data[2..2 + 29].iter().filter(|&&b| b != 0).count() as i32
}
