//! Port of src/melee/if/if_3004.c, split from soundtest.c upstream (doldecomp 598987cf3): fn_80300CC8, fn_80300DE0, fn_80300ED0, fn_8030110C, fn_803011EC, gw_dev_save_profile, gw_dev_unlock_all_chars, gw_dev_unlock_all_stages, un_80300758, un_80300790, un_803009E0, un_80300A88, un_80300AF4, un_80300B58, un_80300BBC, un_80300BF8, un_80300C3C, un_80300C80, un_80300D78, un_80300E74, un_80300F3C, un_80300F98, un_80301028, un_80301074, un_803010C0, un_80301280, un_803012D4, un_80301328, un_8030136C, un_80301634, un_803016D8, un_80301734, un_8030178C, un_80301800, un_80301840, un_803018BC, un_8030191C, un_80301964, un_80301A64, un_80301AD4, un_80301B48, un_80301BA8, un_80301C64, un_80301C80, un_80301CE0, un_80301D7C.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, clippy::all)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use super::soundtest::*;
use super::soundtest::ext;

pub unsafe extern "C" fn un_80300758(arg0: soundtest_callback_arg0) -> i32 {
    melee_core::ct!("un_80300758");
    if arg0 == 1 {
        un_802FFCD0(4, (&raw mut un_803FA258 as *mut u8).add(0x10) as *mut c_void);
    }
    0
}

pub unsafe extern "C" fn un_80300790(arg0: soundtest_callback_arg0) -> i32 {
    melee_core::ct!("un_80300790");
    if arg0 == 1 {
        un_802FFCD0(4, (&raw mut un_803FA258 as *mut u8).add(0x24) as *mut c_void);
    }
    0
}

pub unsafe extern "C" fn un_803009E0(_arg0: soundtest_callback_arg0) -> i32 {
    melee_core::ct!("un_803009E0");
    match un_804D6DD8 {
        2 => {
            let new_var = ext::gmMainLib_8015D06C(un_804D6DC8 as u8);
            let val = *new_var as u32;
            un_804D6DD0 = (val / 100) as f32;
        }
        3 => {
            let result = ext::gmMainLib_8015D48C(un_804D6DC8 as u8);
            ext::gmMainLib_8015D4E8(un_804D6DC8 as u8, result);
            un_804D6DCC = result;
        }
        _ => {}
    }
    0
}

pub unsafe extern "C" fn un_80300A88(_arg0: soundtest_callback_arg0) -> i32 {
    melee_core::ct!("un_80300A88");
    match un_804D6DD8 {
        3 => {
            if un_804D6DCC >= 1 {
                un_804D6DCC = 1;
            }
        }
        _ => {}
    }
    0
}

pub unsafe extern "C" fn un_80300AF4(arg0: soundtest_callback_arg0) -> i32 {
    melee_core::ct!("un_80300AF4");
    if arg0 == 1 {
        sfx_forward();
        let data = &raw mut un_803FA258;
        (*data).x4[1] = 0x3F;
        (*data).x4[3] = 0xE;
        (*data).x24[1] = 3;
        (*data).x24[2] = 3;
        (*data).x24[3] = 3;
        ext::gm_SetNextGameModeStateId(4);
        ext::gm_801A4B60();
    }
    0
}

pub unsafe extern "C" fn un_80300B58(arg0: soundtest_callback_arg0) -> i32 {
    melee_core::ct!("un_80300B58");
    if arg0 == 1 {
        sfx_forward();
        let data = &raw mut un_803FA258;
        (*data).x4[1] = 0x3B;
        (*data).x4[3] = 0x2;
        (*data).x24[1] = 3;
        (*data).x24[2] = 3;
        (*data).x24[3] = 3;
        ext::gm_SetNextGameModeStateId(4);
        ext::gm_801A4B60();
    }
    0
}

pub unsafe extern "C" fn un_80300BBC(update_scene: soundtest_callback_arg0) -> i32 {
    melee_core::ct!("un_80300BBC");
    if update_scene == 1 {
        ext::Toy_803124BC();
        ext::Toy_80311680();
        ext::gm_ChangeGameModeAfterCurrentScene(GM_TOY_GALLERY as i32);
        ext::gm_801A4B60();
    }
    0
}

pub unsafe extern "C" fn un_80300BF8(update_scene: soundtest_callback_arg0) -> i32 {
    melee_core::ct!("un_80300BF8");
    if update_scene == 1 {
        ext::Toy_803124BC();
        ext::Toy_SetUnlockState(0xE6, 1);
        ext::gm_ChangeGameModeAfterCurrentScene(GM_MENU as i32);
        ext::gm_801A4B60();
    }
    0
}

pub unsafe extern "C" fn un_80300C3C(update_scene: soundtest_callback_arg0) -> i32 {
    melee_core::ct!("un_80300C3C");
    if update_scene == 1 {
        ext::Toy_803124BC();
        ext::Toy_SetUnlockState(0xC9, 1);
        ext::gm_ChangeGameModeAfterCurrentScene(GM_MENU as i32);
        ext::gm_801A4B60();
    }
    0
}

pub unsafe extern "C" fn un_80300C80(arg0: soundtest_callback_arg0) -> i32 {
    melee_core::ct!("un_80300C80");
    if arg0 == 1 {
        let ptr = ext::gmMainLib_8015CCF0();
        *ptr = un_804D6DDC.wrapping_mul(10) as u32;
        ext::Toy_803124BC();
        ext::gm_ChangeGameModeAfterCurrentScene(GM_MENU as i32);
        ext::gm_801A4B60();
    }
    0
}

pub unsafe extern "C" fn fn_80300CC8(arg0: soundtest_callback_arg0) -> i32 {
    melee_core::ct!("fn_80300CC8");
    match arg0 {
        0 => {
            sfx_back();
            ext::un_80304334(ext::un_80302DF0());
        }
        6 => {
            sfx_forward();
            let r29 = ext::gmMainLib_GetTimeMatchTotal();
            let r30 = ext::gmMainLib_GetStockMatchTotal();
            let r31 = ext::gmMainLib_GetCoinMatchTotal();
            let ptr = ext::gmMainLib_GetBonusMatchTotal();
            *r29 = un_804D6DE0 as u32;
            *r30 = un_804D6DE4 as u32;
            *r31 = un_804D6DE8 as u32;
            *ptr = un_804D6DEC as u32;
            ext::gm_ChangeGameModeAfterCurrentScene(GM_MENU as i32);
            ext::gm_801A4B60();
        }
        _ => {}
    }
    0
}

pub unsafe extern "C" fn un_80300D78(arg0: soundtest_callback_arg0) -> i32 {
    melee_core::ct!("un_80300D78");
    if arg0 == 1 {
        ext::lbAudioAx_80024030(1);
        un_804D6DE0 = un_804D6DF0;
        un_804D6DE4 = un_804D6DF0;
        un_804D6DE8 = un_804D6DF0;
        un_804D6DEC = un_804D6DF0;
        un_802FFD94(arg0 as i32, &raw mut un_803FC70C as *mut c_void, Some(fn_80300CC8));
    }
    0
}

pub unsafe extern "C" fn fn_80300DE0(arg0: soundtest_callback_arg0) -> i32 {
    melee_core::ct!("fn_80300DE0");
    let ptr = ext::gmMainLib_8015D06C(un_804D6DC8 as u8);
    match arg0 {
        0 => {
            sfx_back();
            ext::un_80304334(ext::un_80302DF0());
        }
        6 => {
            sfx_forward();
            *ptr = melee_core::ppc::fcvt_u32((100.0f32 * un_804D6DD0)) as i32;
            ext::gm_ChangeGameModeAfterCurrentScene(GM_MENU as i32);
            ext::gm_801A4B60();
        }
        _ => {}
    }
    0
}

pub unsafe extern "C" fn un_80300E74(arg0: soundtest_callback_arg0) -> i32 {
    melee_core::ct!("un_80300E74");
    if arg0 == 1 {
        ext::lbAudioAx_80024030(1);
        un_804D6DD8 = 2;
        un_802FFD94(arg0 as i32, &raw mut un_803FC8C8 as *mut c_void, Some(fn_80300DE0));
    }
    0
}

pub unsafe extern "C" fn fn_80300ED0(arg0: soundtest_callback_arg0) -> i32 {
    melee_core::ct!("fn_80300ED0");
    match arg0 {
        0 => {
            sfx_back();
            ext::un_80304334(ext::un_80302DF0());
        }
        6 => {
            sfx_forward();
            ext::gmMainLib_8015D4E8(un_804D6DC8 as u8, un_804D6DCC);
            ext::gm_ChangeGameModeAfterCurrentScene(GM_MENU as i32);
            ext::gm_801A4B60();
        }
        _ => {}
    }
    0
}

pub unsafe extern "C" fn un_80300F3C(arg0: soundtest_callback_arg0) -> i32 {
    melee_core::ct!("un_80300F3C");
    if arg0 == 1 {
        ext::lbAudioAx_80024030(1);
        un_804D6DD8 = 3;
        un_802FFD94(arg0 as i32, &raw mut un_803FC828 as *mut c_void, Some(fn_80300ED0));
    }
    0
}

pub unsafe extern "C" fn un_80300F98(arg0: soundtest_callback_arg0) -> i32 {
    melee_core::ct!("un_80300F98");
    if arg0 == 1 {
        let temp_r31 = ext::gmMainLib_GetKOTotal();
        sfx_forward();
        *temp_r31 = un_804D6DF4;
        ext::gm_ChangeGameModeAfterCurrentScene(GM_MENU as i32);
        ext::gm_801A4B60();
    }
    0
}

pub unsafe extern "C" fn un_80301028(arg0: soundtest_callback_arg0) -> i32 {
    melee_core::ct!("un_80301028");
    if arg0 == 1 {
        let ptr = ext::gmMainLib_8015CDD4();
        (*ptr).x5 = un_804D5908.wrapping_sub(1) as u8;
        sfx_forward();
        ext::gm_ChangeGameModeAfterCurrentScene(GM_ADVENTURE as i32);
        ext::gm_801A4B60();
    }
    0
}

pub unsafe extern "C" fn un_80301074(arg0: soundtest_callback_arg0) -> i32 {
    melee_core::ct!("un_80301074");
    if arg0 == 1 {
        let ptr = ext::gmMainLib_8015CDC8();
        (*ptr).x5 = un_804D590C.wrapping_sub(1) as u8;
        sfx_forward();
        ext::gm_ChangeGameModeAfterCurrentScene(GM_CLASSIC as i32);
        ext::gm_801A4B60();
    }
    0
}

pub unsafe extern "C" fn un_803010C0(arg0: soundtest_callback_arg0) -> i32 {
    melee_core::ct!("un_803010C0");
    if arg0 == 1 {
        let ptr = ext::gmMainLib_8015CDE0();
        (*ptr).x5 = un_804D5910.wrapping_sub(1) as u8;
        sfx_forward();
        ext::gm_ChangeGameModeAfterCurrentScene(GM_ALLSTAR as i32);
        ext::gm_801A4B60();
    }
    0
}

pub unsafe extern "C" fn fn_8030110C(arg0: soundtest_callback_arg0) -> i32 {
    melee_core::ct!("fn_8030110C");
    match arg0 {
        0 => {
            sfx_back();
            ext::un_80304334(ext::un_80302DF0());
        }
        6 => {
            sfx_forward();
            ext::gm_801BEFA4(un_804D6DF8);
            ext::gm_801BEFC0(un_804D6DFC);
            let ptr = ext::gmMainLib_8015CDC8();
            (*ptr).c_kind = un_804D6DF8 as i8;
            (*ptr).color = un_804D6DFC as u8;
            match un_804D6E00 {
                0 => {
                    ext::gm_ChangeGameModeAfterCurrentScene(GM_DEBUG_GOVER as i32);
                    ext::gm_801BF040(0x15);
                }
                1 => {
                    ext::gm_ChangeGameModeAfterCurrentScene(GM_DEBUG_GOVER as i32);
                    ext::gm_801BF040(0x16);
                }
                2 => {
                    ext::gm_ChangeGameModeAfterCurrentScene(GM_DEBUG_GOVER as i32);
                    ext::gm_801BF040(0x17);
                }
                _ => {}
            }
            ext::gm_801A4B60();
        }
        _ => {}
    }
    0
}

pub unsafe extern "C" fn fn_803011EC(arg0: soundtest_callback_arg0) -> i32 {
    melee_core::ct!("fn_803011EC");
    match arg0 {
        0 => {
            sfx_back();
            ext::un_80304334(ext::un_80302DF0());
        }
        6 => {
            sfx_forward();
            ext::gm_801BEFA4(un_804D6DF8);
            ext::gm_801BEFC0(un_804D6DFC);
            let c_kind = un_804D6DF8 as i8;
            (*ext::gm_GetAllStarData()).x0.x0.ckind = c_kind;
            let color = un_804D6DFC as u8;
            (*ext::gm_GetAllStarData()).x0.x0.color = color;
            ext::gm_ChangeGameModeAfterCurrentScene(GM_CLASSIC_GOVER as i32);
            ext::gm_801A4B60();
        }
        _ => {}
    }
    0
}

pub unsafe extern "C" fn un_80301280(arg0: soundtest_callback_arg0) -> i32 {
    melee_core::ct!("un_80301280");
    if arg0 == 1 {
        ext::lbAudioAx_80024030(1);
        un_802FFD94(arg0 as i32, &raw mut un_803FCD34 as *mut c_void, Some(fn_803011EC));
    }
    0
}

pub unsafe extern "C" fn un_803012D4(arg0: soundtest_callback_arg0) -> i32 {
    melee_core::ct!("un_803012D4");
    if arg0 == 1 {
        ext::lbAudioAx_80024030(1);
        un_802FFD94(arg0 as i32, &raw mut un_803FCD34 as *mut c_void, Some(fn_8030110C));
    }
    0
}

pub unsafe extern "C" fn un_80301328(update_scene: soundtest_callback_arg0) -> i32 {
    melee_core::ct!("un_80301328");
    if update_scene == 1 {
        sfx_forward();
        ext::gm_801BF708(0x0);
        ext::gm_ChangeGameModeAfterCurrentScene(GM_OPENING_MV as i32);
        ext::gm_801A4B60();
    }
    0
}

pub unsafe extern "C" fn un_8030136C(update_scene: soundtest_callback_arg0) -> i32 {
    melee_core::ct!("un_8030136C");
    if update_scene == 1 {
        sfx_forward();
        ext::gm_801BF708(0x5);
        ext::gm_ChangeGameModeAfterCurrentScene(GM_OPENING_MV as i32);
        ext::gm_801A4B60();
    }
    0
}

pub unsafe extern "C" fn un_80301634(_arg0: soundtest_callback_arg0) -> i32 {
    melee_core::ct!("un_80301634");
    ext::lbCardNew_AllocWorkArea();
    ext::lbCardGame_LoadArchive(0);
    let r31 = ext::HSD_MemAlloc(ext::lbSnap_8001E210());
    let r3 = ext::HSD_MemAlloc(ext::lbSnap_8001E204());
    ext::lbSnap_8001E218(r3, r31);
    un_804D6E04 = ext::HSD_MemAlloc(0x96000);
    let gobj = ext::GObj_Create(0x13, 0x14, 0);
    ext::HSD_SObjLib_803A55DC(gobj, 0x280, 0x1E0, 0xC);
    (*gobj).gxlink_prios = 0x40000;
    un_804D6E08 = null_mut();
    0
}

pub unsafe extern "C" fn un_803016D8(arg0: soundtest_callback_arg0) -> i32 {
    melee_core::ct!("un_803016D8");
    if arg0 != 1 {
        return 0;
    }
    let _result = ext::lb_8001B8C8(0);
    0
}

pub unsafe extern "C" fn un_80301734(arg0: soundtest_callback_arg0) -> i32 {
    melee_core::ct!("un_80301734");
    if arg0 != 1 {
        return 0;
    }
    let _result = ext::lb_8001C87C();
    0
}

pub unsafe extern "C" fn un_8030178C(arg0: soundtest_callback_arg0) -> i32 {
    melee_core::ct!("un_8030178C");
    if arg0 != 1 {
        return 0;
    }
    let result = ext::lb_8001C8BC();
    if result == 0 {
        ext::lbCardGame_SetCardStatus(0);
    }
    0
}

pub unsafe extern "C" fn un_80301800(arg0: soundtest_callback_arg0) -> i32 {
    melee_core::ct!("un_80301800");
    if arg0 != 1 {
        return 0;
    }
    ext::lbCardGame_SaveChanges();
    0
}

pub unsafe extern "C" fn un_80301840(arg0: soundtest_callback_arg0) -> i32 {
    melee_core::ct!("un_80301840");
    if arg0 != 1 {
        return 0;
    }
    let result = ext::lb_8001CBBC() as i32;
    ext::gmMainLib_8015FA34(result);
    if result == 0 || result == 2 {
        ext::lbCardGame_SetCardStatus(0);
    }
    0
}

pub unsafe fn gw_dev_save_profile() {
    melee_core::ct!("gw_dev_save_profile");
    ext::lbCardNew_AllocWorkArea();
    ext::lb_8001C87C();
}

pub unsafe extern "C" fn gw_dev_unlock_all_chars(arg0: soundtest_callback_arg0) -> i32 {
    melee_core::ct!("gw_dev_unlock_all_chars");
    if arg0 != 1 {
        return 0;
    }
    ext::gm_80164F18();
    gw_dev_save_profile();
    0
}

pub unsafe extern "C" fn gw_dev_unlock_all_stages(arg0: soundtest_callback_arg0) -> i32 {
    melee_core::ct!("gw_dev_unlock_all_stages");
    if arg0 != 1 {
        return 0;
    }
    ext::gm_8016468C();
    gw_dev_save_profile();
    0
}

pub unsafe extern "C" fn un_803018BC(arg0: soundtest_callback_arg0) -> i32 {
    melee_core::ct!("un_803018BC");
    if arg0 != 1 {
        return 0;
    }
    let _result = ext::lb_8001CC4C();
    ext::lbCardGame_SetCardStatus(4);
    0
}

pub unsafe extern "C" fn un_8030191C(arg0: soundtest_callback_arg0) -> i32 {
    melee_core::ct!("un_8030191C");
    if arg0 != 1 {
        return 0;
    }
    ext::lbSnap_8001D40C(0);
    0
}

pub unsafe extern "C" fn un_80301964(arg0: soundtest_callback_arg0) -> i32 {
    melee_core::ct!("un_80301964");
    if arg0 != 1 {
        return 0;
    }
    if ext::lbSnap_8001E058(0, un_804D6E0C) == 0xB {
        let mut result;
        loop {
            result = ext::lbCardNew_CompleteNextTask();
            if result != 0xB {
                break;
            }
        }
        if result == 0 && ext::lbSnap_8001DE8C(un_804D6E04) != 0 {
            if !un_804D6E08.is_null() {
                ext::HSD_GObjFree(un_804D6E08);
            }
            let gobj = ext::GObj_Create(0xE, 0xF, 0);
            ext::GObj_SetupGXLink(gobj, Some(ext::cb::HSD_SObjLib_803A49E0), 0x12, 0);
            un_803FD274.image_ptr = un_804D6E04 as *mut u8;
            let sobj = ext::HSD_SObjLib_803A477C(gobj, &raw mut un_804D5990, 0, 0, 0x80, 0);
            (*sobj).x10 = 320.0;
            (*sobj).x14 = 240.0;
            (*sobj).x40 |= 2;
            un_804D6E08 = gobj;
        }
    }
    0
}

pub unsafe extern "C" fn un_80301A64(arg0: soundtest_callback_arg0) -> i32 {
    melee_core::ct!("un_80301A64");
    if arg0 != 1 {
        return 0;
    }
    let mut result = ext::lbSnap_8001D5FC(0, un_804D6E10);
    if result == 0xB {
        loop {
            result = ext::lbCardNew_CompleteNextTask();
            if result != 0xB {
                break;
            }
        }
        if result == 0 {
            ext::lbSnap_8001D40C(0);
        }
    }
    0
}

pub unsafe extern "C" fn un_80301AD4(arg0: soundtest_callback_arg0) -> i32 {
    melee_core::ct!("un_80301AD4");
    if arg0 != 1 {
        return 0;
    }
    let mut result = ext::lbSnap_8001D7B0(0, un_804D6E14, un_804D6E10);
    if result == 0xB {
        loop {
            result = ext::lbCardNew_CompleteNextTask();
            if result != 0xB {
                break;
            }
        }
        if result == 0 {
            ext::lbSnap_8001D40C(0);
        }
    }
    0
}

pub unsafe extern "C" fn un_80301B48(arg0: soundtest_callback_arg0) -> i32 {
    melee_core::ct!("un_80301B48");
    match arg0 {
        0 => {
            sfx_back();
            ext::un_80304334(ext::un_80302DF0());
        }
        6 => {
            sfx_forward();
            ext::gm_SetNextGameModeStateId(6);
            ext::gm_801A4B60();
        }
        _ => {}
    }
    0
}

/// `void un_80301BA8(struct UnkSoundTestData1* out)`
pub unsafe extern "C" fn un_80301BA8(out: *mut UnkSoundTestData1) {
    melee_core::ct!("un_80301BA8");
    let d = &raw mut un_803FA258;
    (*out).unk_0 = (*d).x104;
    (*out).unk_4 = (*d).x108;
    (*out).unk_A = (*d).x100 as u8;
    (*out).unk_B = (*d).x10C as u8;
    (*out).unk_C = (*d).x11C as u8;
    (*out).unk_8 = 0;
    (*out).unk_9 = 0x78;
    (*out).unk_D = (*d).x110 as u8;
    (*out).unk_19 = (*d).xF8 as u8;
    (*out).unk_E = (*d).x114 as u8;
    (*out).unk_1A = (*d).xF8 as u8;
    (*out).unk_F = (*d).x118 as u8;
    (*out).unk_1B = (*d).xF8 as u8;
    (*out).unk_10 = (*d).x120 as u8;
    (*out).unk_1C = (*d).xFC as u8;
    (*out).unk_11 = (*d).x124 as u8;
    (*out).unk_1D = (*d).xFC as u8;
    (*out).unk_12 = (*d).x128 as u8;
    (*out).unk_1E = (*d).xFC as u8;
    (*out).unk_13 = 0;
    (*out).unk_14 = 0;
    (*out).unk_15 = 0;
    (*out).unk_16 = 0;
    (*out).unk_17 = 0;
    (*out).unk_18 = 0;
}

/// `void un_80301C64(un_80301C64_t*)`
pub unsafe extern "C" fn un_80301C64(arg0: *mut un_80301C64_t) {
    melee_core::ct!("un_80301C64");
    (*arg0).x0 = un_803FA258.x138;
    (*arg0).x4 = 0;
}

pub unsafe extern "C" fn un_80301C80(arg0: soundtest_callback_arg0) -> i32 {
    melee_core::ct!("un_80301C80");
    match arg0 {
        0 => {
            sfx_back();
            ext::un_80304334(ext::un_80302DF0());
        }
        6 => {
            sfx_forward();
            ext::gm_SetNextGameModeStateId(0xB);
            ext::gm_801A4B60();
        }
        _ => {}
    }
    0
}

pub unsafe extern "C" fn un_80301CE0(arg0: soundtest_callback_arg0) -> i32 {
    melee_core::ct!("un_80301CE0");
    match arg0 {
        6 => {
            sfx_forward();
            ext::gm_SetNextGameModeStateId(0xB);
            ext::gm_801A4B60();
        }
        1 => {
            un_802FFCD0(4, un_803FA258.x144.as_mut_ptr() as *mut c_void);
        }
        _ => {}
    }
    0
}

pub unsafe extern "C" fn un_80301D7C(arg0: soundtest_callback_arg0) -> i32 {
    melee_core::ct!("un_80301D7C");
    if arg0 == 1 {
        sfx_forward();
        ext::gm_801BEB74(un_803FA258.x188.wrapping_sub(1) as u8);
        ext::gm_ChangeGameModeAfterCurrentScene(GM_EVENT as i32);
        ext::gm_801A4B60();
    }
    0
}
