//! src/melee/mn/mnsound.c: the sound-options menu (Settings > Sound): the stereo / mono channel
//! selector, the sound/music mix slider, its per-frame cursor animation and the scene builder.
//! Callees in TUs that are not ported yet are `ext` stubs named like the C function.
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, unused_mut, unused_variables, unused_assignments, clippy::missing_safety_doc)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types as mt;
#[allow(unused_imports)]
use mt::Vec as Vec3_mn_ptr;
use crate::mn_inl::*;
use core::ffi::{c_char, c_void};
use core::ptr::null_mut;
use melee_core::fp::fmadds;
use melee_types::*;

pub mod ext {
    use super::*;
    use crate::ext_fns;
    ext_fns! {
        fn GObj_Create(classifier: u16, p_link: u8, priority: u8) -> *mut HSD_GObj;
        extern "C" fn HSD_GObj_JObjCallback(gobj: *mut HSD_GObj, arg1: isize) = crate::tu::adapt::HSD_GObj_JObjCallback__9cd907;
        fn lbAudioAx_80024BD0() -> i32 = crate::tu::adapt::lbAudioAx_80024BD0__eb4f5a;
        fn lbAudioAx_80024C08(a0: enum_t) -> i32 = crate::tu::adapt::lbAudioAx_80024C08__87c4fa;
        fn mn_80229894(a0: i32, a1: u16, a2: i32) = crate::tu::adapt::mn_80229894__f7df64;
        fn mn_8022EC18(a0: *mut HSD_JObj, a1: *mut AnimLoopSettings, a2: HSD_TypeMask) -> f32 = crate::tu::adapt::mn_8022EC18__bee2c1;
        fn mn_8022ED6C(a0: *mut HSD_JObj, a1: *mut AnimLoopSettings) -> f32 = crate::tu::adapt::mn_8022ED6C__f73cae;
        fn mn_8022F298(a0: *mut HSD_JObj) -> f32 = crate::tu::adapt::mn_8022F298__0e9b78;
        fn mn_8022F3D8(a0: *mut HSD_JObj, a1: u8, a2: HSD_TypeMask) = crate::tu::adapt::mn_8022F3D8__9f7fac;
        fn HSD_JObjReqAnimAll(a0: *mut HSD_JObj, a1: f32);
        fn HSD_JObjLoadJoint(a0: *mut HSD_Joint) -> *mut HSD_JObj = crate::tu::adapt::HSD_JObjLoadJoint__3efee1;
        fn HSD_JObjAddAnimAll(a0: *mut HSD_JObj, a1: *mut HSD_AnimJoint, a2: *mut HSD_MatAnimJoint, a3: *mut HSD_ShapeAnimJoint);
        fn HSD_JObjAnimAll(a0: *mut HSD_JObj);
        fn HSD_JObjSetFlagsAll(a0: *mut HSD_JObj, flags: u32);
        fn HSD_JObjClearFlagsAll(a0: *mut HSD_JObj, flags: u32);
        fn HSD_SisLib_803A5CC4(a0: *mut HSD_Text) = crate::tu::adapt::HSD_SisLib_803A5CC4__6c7e38;
        fn gm_801602C0(a0: i8) = crate::tu::gm_1601::gm_801602C0;
        fn gmMainLib_GetGameRules() -> *mut GameRules = crate::tu::gmmain_lib::gmMainLib_GetGameRules;
        fn gmMainLib_8015ED74() -> u8 = crate::tu::gmmain_lib::gmMainLib_8015ED74;
        fn gmMainLib_8015ED80(a0: i8) = crate::tu::gmmain_lib::gmMainLib_8015ED80;
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
}
use ext::*;

static mut mnSound_803EEED8: [AnimLoopSettings; 5] = [
    AnimLoopSettings {
        start_frame: 0.0_f32,
        end_frame: 199.0_f32,
        loop_frame: 0.0_f32,
    },
    AnimLoopSettings {
        start_frame: 0.0_f32,
        end_frame: 29.0_f32,
        loop_frame: 0.0_f32,
    },
    AnimLoopSettings {
        start_frame: 30.0_f32,
        end_frame: 59.0_f32,
        loop_frame: 30.0_f32,
    },
    AnimLoopSettings {
        start_frame: 10.0_f32,
        end_frame: 15.0_f32,
        loop_frame: -0.1_f32,
    },
    AnimLoopSettings {
        start_frame: 0.0_f32,
        end_frame: 5.0_f32,
        loop_frame: -0.1_f32,
    },
];
static mut volume_left: AnimLoopSettings = AnimLoopSettings {
    start_frame: 0.0_f32,
    end_frame: 3.0_f32,
    loop_frame: -0.1_f32,
};
static mut volume_right: AnimLoopSettings = AnimLoopSettings {
    start_frame: 4.0_f32,
    end_frame: 7.0_f32,
    loop_frame: -0.1_f32,
};
static mut mnSound_804A08A8: StaticModelDesc = StaticModelDesc {
    joint: null_mut(),
    animjoint: null_mut(),
    matanim_joint: null_mut(),
    shapeanim_joint: null_mut(),
};
static mut mnSound_804D6C30: *mut HSD_GObj = null_mut();
unsafe extern "C" fn getVolumePosition(sound_music_mix: u8,
    pos_0: *const Vec3,
    pos_1: *const Vec3,
) -> f32 {
    // (mix + 100) / 200 * (p1.x - p0.x) + p0.x: the multiply-add is contracted (fmadds).
    fmadds(
        (sound_music_mix as i8 as i32 + 100) as f32 / 200.0_f32,
        (*pos_1).x - (*pos_0).x,
        (*pos_0).x,
    )
}
unsafe extern "C" fn updateVolumeAnim(jobj: *mut HSD_JObj,
    sound_music_mix: u8,
    direction: i32,
    pos_0: *mut Vec3,
    pos_1: *mut Vec3,
    jobj_anim_0: *mut *mut HSD_JObj,
    jobj_anim_1: *mut *mut HSD_JObj,
    jobj_anim_2: *mut *mut HSD_JObj,
) {
    lb_80011E24(jobj as *mut HSD_JObj, jobj_anim_0 as *mut *mut HSD_JObj, &[6, -1]);
    lb_80011E24(jobj as *mut HSD_JObj, jobj_anim_1 as *mut *mut HSD_JObj, &[3, -1]);
    lb_80011E24(jobj as *mut HSD_JObj, jobj_anim_2 as *mut *mut HSD_JObj, &[4, -1]);
    HSD_JObjGetTranslation(*jobj_anim_1, pos_0);
    HSD_JObjGetTranslation(*jobj_anim_2, pos_1);
    HSD_JObjSetTranslateX(
        *jobj_anim_0,
        getVolumePosition(sound_music_mix, pos_0, pos_1),
    );
    if direction != 0 {
        HSD_JObjReqAnimAll(
            *jobj_anim_0,
            if direction < 0 {
                volume_left.start_frame
            } else {
                volume_right.start_frame
            },
        );
    }
    gm_801602C0(sound_music_mix as i8);
}
unsafe extern "C" fn updateChannelAnim(jobj: *mut HSD_JObj,
    channel: i32,
    jobj_anim_0: *mut *mut HSD_JObj,
    jobj_anim_1: *mut *mut HSD_JObj,
    jobj_anim_2: *mut *mut HSD_JObj,
) {
    let mut right_frame: f32 = 0.;
    let mut left_frame: f32 = 0.;
    lb_80011E24(jobj as *mut HSD_JObj, jobj_anim_0 as *mut *mut HSD_JObj, &[8, -1]);
    lb_80011E24(jobj as *mut HSD_JObj, jobj_anim_1 as *mut *mut HSD_JObj, &[10, -1]);
    lb_80011E24(jobj as *mut HSD_JObj, jobj_anim_2 as *mut *mut HSD_JObj, &[9, -1]);
    left_frame = mn_8022F298(*jobj_anim_1);
    right_frame = mn_8022F298(*jobj_anim_2);
    HSD_JObjReqAnimAll(*jobj_anim_0, channel as f32);
    HSD_JObjAnimAll(*jobj_anim_0);
    HSD_JObjReqAnimAll(*jobj_anim_1, left_frame);
    mn_8022F3D8(*jobj_anim_1, 0xff, MOBJ_MASK);
    HSD_JObjAnimAll(*jobj_anim_1);
    HSD_JObjReqAnimAll(*jobj_anim_2, right_frame);
    mn_8022F3D8(*jobj_anim_2, 0xff, MOBJ_MASK);
    HSD_JObjAnimAll(*jobj_anim_2);
}
unsafe extern "C" fn updateCenterText() {
    let mut menu: *mut Menu = (*mnSound_804D6C30).user_data as *mut Menu;
    let mut text_id: i32 = 0;
    if !(*menu).text.is_null() {
        HSD_SisLib_803A5CC4((*menu).text);
    }
    if (*menu).unk2 as i32 == 0 {
        text_id = 0xbb as i32;
    } else {
        text_id = 0xbc as i32;
    }
    Menu_InitCenterText(menu, text_id);
}
pub unsafe extern "C" fn mnSound_802492CC(gobj: *mut HSD_GObj) {
    melee_core::ct!("mnSound_802492CC");
    let mut events: u64 = 0;
    let mut mix: u8 = 0;
    let mut menu: *mut Menu = (*mnSound_804D6C30).user_data as *mut Menu;
    if mn_804D6BC8.cooldown as i32 != 0 {
        Menu_DecrementAnimTimer();
        return;
    }
    events = Menu_GetAllInputs();
    if events & MenuInput_Back as i32 as u64 != 0 {
        sfxBack();
        mn_804A04F0.entering_menu = 0;
        gmMainLib_8015ED80((*((*mnSound_804D6C30).user_data as *mut Menu)).unk3 as i8);
        lbCardGame_SaveChanges();
        mn_80229894(4, 1, 3);
        return;
    }
    if events
        & (MenuInput_Up as i32 | MenuInput_Down as i32)
            as u64 != 0
    {
        sfxMove();
        if (*menu).unk2 as i32 == 0 {
            (*menu).unk2 = 1;
        } else {
            (*menu).unk2 = 0;
        }
        updateCenterText();
        return;
    }
    if events & MenuInput_Left as i32 as u64 != 0 {
        if (*menu).unk2 as i32 == 0 {
            if (*menu).unk1 as i32 == 1 {
                sfxMove();
                (*menu).unk1 = 0;
                let mut jobj_anim_2: *mut HSD_JObj = null_mut();
                let mut jobj_anim_1: *mut HSD_JObj = null_mut();
                let mut jobj_anim_0: *mut HSD_JObj = null_mut();
                updateChannelAnim(
                    HSD_GObjGetHSDObj(mnSound_804D6C30) as *mut HSD_JObj,
                    (*menu).unk1 as i32,
                    &raw mut jobj_anim_0,
                    &raw mut jobj_anim_1,
                    &raw mut jobj_anim_2,
                );
                lbAudioAx_80024C08((*menu).unk1 as enum_t);
            }
        } else if (*menu).unk3 as i8 as i32 > -100
        {
            sfxMove();
            (*menu).unk3 = ((*menu).unk3 as i32 - 5)
                as u8;
            mix = (*menu).unk3;
            let mut pos_1: Vec3 = Vec3 { x: 0., y: 0., z: 0. };
            let mut pos_0: Vec3 = Vec3 { x: 0., y: 0., z: 0. };
            let mut jobj_anim_2_0: *mut HSD_JObj = null_mut();
            let mut jobj_anim_1_0: *mut HSD_JObj = null_mut();
            let mut jobj_anim_0_0: *mut HSD_JObj = null_mut();
            updateVolumeAnim(
                HSD_GObjGetHSDObj(mnSound_804D6C30) as *mut HSD_JObj,
                mix,
                -1,
                &raw mut pos_0,
                &raw mut pos_1,
                &raw mut jobj_anim_0_0,
                &raw mut jobj_anim_1_0,
                &raw mut jobj_anim_2_0,
            );
            gmMainLib_8015ED80(
                (*((*mnSound_804D6C30).user_data as *mut Menu)).unk3 as i8,
            );
        }
    } else if events & MenuInput_Right as i32 as u64 != 0 {
        if (*menu).unk2 as i32 == 0 {
            if (*menu).unk1 as i32 == 0 {
                sfxMove();
                (*menu).unk1 = 1;
                let mut jobj_anim_2_1: *mut HSD_JObj = core::ptr::null_mut::<
                    HSD_JObj,
                >();
                let mut jobj_anim_1_1: *mut HSD_JObj = core::ptr::null_mut::<
                    HSD_JObj,
                >();
                let mut jobj_anim_0_1: *mut HSD_JObj = core::ptr::null_mut::<
                    HSD_JObj,
                >();
                updateChannelAnim(
                    HSD_GObjGetHSDObj(mnSound_804D6C30) as *mut HSD_JObj,
                    (*menu).unk1 as i32,
                    &raw mut jobj_anim_0_1,
                    &raw mut jobj_anim_1_1,
                    &raw mut jobj_anim_2_1,
                );
                lbAudioAx_80024C08((*menu).unk1 as enum_t);
            }
        } else if ((*menu).unk3 as i8 as i32) < 100
        {
            sfxMove();
            (*menu).unk3 = ((*menu).unk3 as i32 + 5)
                as u8;
            mix = (*menu).unk3;
            let mut pos_1_0: Vec3 = Vec3 { x: 0., y: 0., z: 0. };
            let mut pos_0_0: Vec3 = Vec3 { x: 0., y: 0., z: 0. };
            let mut jobj_anim_2_2: *mut HSD_JObj = null_mut();
            let mut jobj_anim_1_2: *mut HSD_JObj = null_mut();
            let mut jobj_anim_0_2: *mut HSD_JObj = null_mut();
            updateVolumeAnim(
                HSD_GObjGetHSDObj(mnSound_804D6C30) as *mut HSD_JObj,
                mix,
                1,
                &raw mut pos_0_0,
                &raw mut pos_1_0,
                &raw mut jobj_anim_0_2,
                &raw mut jobj_anim_1_2,
                &raw mut jobj_anim_2_2,
            );
            gmMainLib_8015ED80(
                (*((*mnSound_804D6C30).user_data as *mut Menu)).unk3 as i8,
            );
        }
    }
}
unsafe extern "C" fn animateSelectedChannel(gobj: *mut HSD_GObj,
    jobj_out: *mut *mut HSD_JObj,
) {
    let mut anim: *mut AnimLoopSettings = &raw mut mnSound_803EEED8
        as *mut AnimLoopSettings;
    let mut menu: *mut Menu = HSD_GObjGetUserData(gobj) as *mut Menu;
    let mut jobj: *mut HSD_JObj = HSD_GObjGetHSDObj(gobj) as *mut HSD_JObj;
    if (*menu).unk2 as i32 == 0 {
        if (*menu).unk1 as i32 == 0 {
            lb_80011E24(jobj as *mut HSD_JObj, jobj_out as *mut *mut HSD_JObj, &[0xa, -1]);
        } else {
            lb_80011E24(jobj as *mut HSD_JObj, jobj_out as *mut *mut HSD_JObj, &[9, -1]);
        }
        mn_8022EC18(*jobj_out, anim, MOBJ_MASK);
    }
}
unsafe extern "C" fn chooseVolumeAnim(jobj: *mut HSD_JObj,
    settings: *mut AnimLoopSettings,
) {
    let mut cursor_frame: f32 = mn_8022F298(jobj as *mut HSD_JObj);
    if volume_left.start_frame <= cursor_frame && cursor_frame <= volume_left.end_frame {
        *settings = volume_left;
    } else {
        *settings = volume_right;
    };
}
pub unsafe extern "C" fn fn_80249A1C(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_80249A1C");
    let mut anim: *mut AnimLoopSettings = &raw mut mnSound_803EEED8
        as *mut AnimLoopSettings;
    let mut menu: *mut Menu = HSD_GObjGetUserData(gobj) as *mut Menu;
    let mut jobj: *mut HSD_JObj = (*gobj).hsd_obj as *mut HSD_JObj;
    let mut sound_selection_jobj: *mut HSD_JObj = null_mut();
    let mut channel_selection_jobj: *mut HSD_JObj = null_mut();
    let mut temp: *mut AnimLoopSettings = null_mut();
    let mut selected_channel_jobj: *mut HSD_JObj = null_mut();
    let mut volume_anim: AnimLoopSettings = AnimLoopSettings {
        start_frame: 0.,
        end_frame: 0.,
        loop_frame: 0.,
    };
    let mut volume_cursor_jobj: *mut HSD_JObj = null_mut();
    let mut cursor_timer: u8 = 0;
    if mn_804A04F0.cur_menu as i32
        != MENU_KIND_SETTINGS_SOUND as i32
    {
        HSD_GObjFree(gobj);
        HSD_SisLib_803A5CC4((*menu).text);
        return;
    }
    cursor_timer = (*menu).cursor;
    if cursor_timer as i32 != 0 {
        (*menu).cursor = (cursor_timer as i32 - 1)
            as u8;
        if (*menu).cursor as i32 != 0 {
            HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
            return;
        }
        HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);
        return;
    }
    animateSelectedChannel(gobj, &raw mut selected_channel_jobj);
    lb_80011E24((*gobj).hsd_obj as *mut HSD_JObj, &raw mut volume_cursor_jobj, &[6, -1]);
    chooseVolumeAnim(volume_cursor_jobj, &raw mut volume_anim);
    mn_8022ED6C(volume_cursor_jobj as *mut HSD_JObj, &raw mut volume_anim);
    lb_80011E24(jobj as *mut HSD_JObj, &raw mut sound_selection_jobj, &[0xe, -1]);
    temp = anim.offset((*menu).unk2 as isize) as *mut AnimLoopSettings;
    mn_8022ED6C(
        sound_selection_jobj as *mut HSD_JObj,
        temp.offset(1 as i32 as isize),
    );
    lb_80011E24(jobj as *mut HSD_JObj, &raw mut channel_selection_jobj, &[0xb, -1]);
    let mut temp_0: *mut AnimLoopSettings = anim.offset((*menu).unk1 as isize)
        as *mut AnimLoopSettings;
    mn_8022ED6C(
        channel_selection_jobj as *mut HSD_JObj,
        temp_0.offset(3 as i32 as isize),
    );
}
unsafe extern "C" fn initUserData(user_data: *mut Menu, gobj: *mut HSD_GObj) {
    let mut proc_0: *mut HSD_GObjProc = null_mut();
    (*user_data).unk3 = gmMainLib_8015ED74();
    (*user_data).text = null_mut();
    GObj_InitUserData(
        gobj,
        0,
        Some(HSD_Free),
        user_data as *mut c_void,
    );
    proc_0 = HSD_GObj_SetupProc(
        gobj,
        Some(fn_80249A1C),
        0,
    );
    (*proc_0).set_flags_3(HSD_GObj_804D783C as u8);
}
pub unsafe extern "C" fn mnSound_80249C08(unused: i32) {
    melee_core::ct!("mnSound_80249C08");
    let mut model: *mut StaticModelDesc = &raw mut mnSound_804A08A8;
    let mut anims: *mut AnimLoopSettings = &raw mut mnSound_803EEED8
        as *mut AnimLoopSettings;
    let mut sound_selection_jobj: *mut HSD_JObj = null_mut();
    let mut channel_selection_jobj: *mut HSD_JObj = null_mut();
    let mut volume_cursor_jobj: *mut HSD_JObj = null_mut();
    let mut jobj: *mut HSD_JObj = null_mut();
    let mut user_data: *mut Menu = null_mut();
    let mut proc_0: *mut HSD_GObjProc = null_mut();
    let mut gobj: *mut HSD_GObj = GObj_Create(
        HSD_GOBJ_CLASS_ITEM as u16,
        7,
        0x80,
    );
    mnSound_804D6C30 = gobj;
    jobj = HSD_JObjLoadJoint((*model).joint as *mut HSD_Joint);
    HSD_GObjObject_80390A70(gobj, HSD_GObj_JObjKind, jobj as *mut c_void);
    GObj_SetupGXLink(
        gobj,
        Some(
            HSD_GObj_JObjCallback
               ,
        ),
        4,
        0x80,
    );
    HSD_JObjAddAnimAll(
        jobj,
        (*model).animjoint,
        (*model).matanim_joint,
        (*model).shapeanim_joint,
    );
    HSD_JObjReqAnimAll(jobj, 0.0_f32);
    user_data = HSD_MemAlloc(core::mem::size_of::<Menu>() as isize) as *mut Menu;
    if user_data.is_null() {
        OSReport(
            c"Can't get user_data.\n".as_ptr()
                as *mut c_char,
        );
        __assert("src/melee/mn/mnsound.c", 310, "user_data");
    };
    gmMainLib_GetGameRules();
    (*user_data).cursor = 0x14;
    (*user_data).unk1 = lbAudioAx_80024BD0() as u8;
    (*user_data).unk2 = 0;
    initUserData(user_data, gobj);
    let mut menu: *mut Menu = HSD_GObjGetUserData(mnSound_804D6C30) as *mut Menu;
    if !(*menu).text.is_null() {
        HSD_SisLib_803A5CC4((*menu).text);
    }
    Menu_InitCenterText(
        menu,
        if (*menu).unk2 as i32 == 0 {
            0xbb
        } else {
            0xbc
        },
    );
    let mut channel_anim_2: *mut HSD_JObj = null_mut();
    let mut channel_anim_1: *mut HSD_JObj = null_mut();
    let mut channel_anim_0: *mut HSD_JObj = null_mut();
    updateChannelAnim(
        HSD_GObjGetHSDObj(gobj) as *mut HSD_JObj,
        (*user_data).unk1 as i32,
        &raw mut channel_anim_0,
        &raw mut channel_anim_1,
        &raw mut channel_anim_2,
    );
    lb_80011E24(jobj as *mut HSD_JObj, &raw mut volume_cursor_jobj, &[6, -1]);
    HSD_JObjReqAnimAll(volume_cursor_jobj, volume_left.end_frame);
    HSD_JObjAnimAll(volume_cursor_jobj);
    let mut pos_1: Vec3 = Vec3 { x: 0., y: 0., z: 0. };
    let mut pos_0: Vec3 = Vec3 { x: 0., y: 0., z: 0. };
    let mut jobj_anim_2: *mut HSD_JObj = null_mut();
    let mut jobj_anim_1: *mut HSD_JObj = null_mut();
    let mut jobj_anim_0: *mut HSD_JObj = null_mut();
    updateVolumeAnim(
        HSD_GObjGetHSDObj(gobj) as *mut HSD_JObj,
        (*user_data).unk3,
        0,
        &raw mut pos_0,
        &raw mut pos_1,
        &raw mut jobj_anim_0,
        &raw mut jobj_anim_1,
        &raw mut jobj_anim_2,
    );
    lb_80011E24(jobj as *mut HSD_JObj, &raw mut sound_selection_jobj, &[0xe, -1]);
    let mut anim: *mut AnimLoopSettings = anims.offset((*user_data).unk2 as isize)
        as *mut AnimLoopSettings;
    mn_8022ED6C(
        sound_selection_jobj as *mut HSD_JObj,
        anim.offset(1 as i32 as isize),
    );
    lb_80011E24(jobj as *mut HSD_JObj, &raw mut channel_selection_jobj, &[0xb, -1]);
    HSD_JObjReqAnimAll(
        channel_selection_jobj,
        (*anims
            .offset(
                ((*user_data).unk1 as i32 + 3)
                    as isize,
            ))
            .end_frame,
    );
    HSD_JObjAnimAll(channel_selection_jobj);
    HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
}
pub unsafe extern "C" fn mnSound_8024A09C(arg0: i32) {
    melee_core::ct!("mnSound_8024A09C");
    let mut proc_0: *mut HSD_GObjProc = null_mut();
    let mut archive: *mut HSD_Archive = null_mut();
    mn_804D6BC8.cooldown = 5;
    mn_804A04F0.prev_menu = mn_804A04F0.cur_menu;
    mn_804A04F0.cur_menu = MENU_KIND_SETTINGS_SOUND as i32 as MenuKind8;
    mn_804A04F0.hovered_selection = 0;
    archive = mn_804D6BB8;
    lbArchive_LoadSections(archive, &[(&raw mut mnSound_804A08A8.joint as *mut *mut c_void, c"MenMainConSo_Top_joint".as_ptr()), (&raw mut mnSound_804A08A8.animjoint as *mut *mut c_void, c"MenMainConSo_Top_animjoint".as_ptr()), (&raw mut mnSound_804A08A8.matanim_joint as *mut *mut c_void, c"MenMainConSo_Top_matanim_joint".as_ptr()), (&raw mut mnSound_804A08A8.shapeanim_joint as *mut *mut c_void, c"MenMainConSo_Top_shapeanim_joint".as_ptr())]);
    mnSound_80249C08(arg0);
    proc_0 = HSD_GObj_SetupProc(
        GObj_Create(0, 1, 0x80),
        Some(mnSound_802492CC),
        0,
    );
    (*proc_0).set_flags_3(HSD_GObj_804D783C as u8);
}

// ---- folded from mn_ptr.rs (consolidation steps 21/22) ----
/// mnsound.c `getVolumePosition_fp`: the slider position for a sound/music mix of -100..100
/// (one fused site).
pub fn getVolumePosition_fp(sound_music_mix: u8, pos_0: &Vec3_mn_ptr, pos_1: &Vec3_mn_ptr) -> f32 {
    melee_core::ct!("getVolumePosition_fp");
    fmadds((sound_music_mix as i8 as i32 + 100) as f32 / 200.0, pos_1.x - pos_0.x, pos_0.x)
}
