//! src/melee/gm/gm_1884.c: the Training Mode scene (HUD counters, the pause menu, CPU/item setup).
//!
//! The file's statics are `pub static mut` so the state oracle (tests/tu_gm_1884.rs against
//! gen/turef/tu_gm_1884_ref.c, the C TU compiled natively with shipping contraction) can seed and
//! compare them. `12.0f * (a + b) + 50.0f` is contracted to a fused multiply-add in the shipping
//! build, so it is `mul_add` here.
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types as mt;
#[allow(unused_imports)]
use mt::HSD_Text as HSD_Text_gm_ptr;
use crate::tu_util::*;
use core::ffi::{c_char, c_void};
use core::ptr::null_mut;
use melee_types::inline as mi;
use melee_types::*;

/// Callees outside this TU (hooks; unset they panic `TODO: <name> is not ported`).
pub mod ext {
    pub use crate::tu_ext::{
        fn_801650E8, fn_80165190, fn_801651FC, fn_801652B0, fn_8016B388, fn_8016EF98,
        gm_8016895C, gm_8016B328, gm_8016EDDC, gm_801A36C0, gm_801A36E0, gm_GetDbPauseFlag,
        ifAll_GetHUDGObj, ifStatus_802F6508, it_8026D258, lbArchive_80016DBC, lbAudioAx_80024030,
        lbLang_IsSavedLanguageUS, lbLang_IsSettingUS, lb_80011E24, lb_80019880, memzero,
        pl_80041300, pl_800412D0, pl_8004134C, GObj_Create, GObj_SetupGXLink, HSD_GObjObject_80390A70,
        HSD_GObj_JObjCallback, HSD_GObj_JObjKind, HSD_GObj_SetupProc, HSD_JObjAnimAll,
        HSD_JObjClearFlagsAll, HSD_JObjLoadJoint, HSD_JObjReqAnimAll, HSD_JObjSetFlags,
        HSD_JObjSetFlagsAll, HSD_SisLib_803A5ACC, HSD_SisLib_803A611C, HSD_SisLib_803A62A0,
        HSD_SisLib_803A6368, Player_800328D4, Player_GetEntity, Player_GetPlayerCharacter,
        Player_SetFacingDirection, Player_SetHUDDamage, Player_SetPlayerAndEntityCpuType,
        Stage_80224E64,
    };
}

/// `__OSBusClock`: the C reads guest memory `0x800000F8` (OSInit stores the 162 MHz bus clock
/// there; `OS_TIMER_CLOCK` is a quarter of it). Settable so the oracle and the integrating crate
/// can bind it to wherever the clock lives.
pub static mut OS_BUS_CLOCK: u32 = 162_000_000;

/// `TrainingItemEntry` (4 bytes on the GC).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct TrainingItemEntry {
    pub item_id: i16,
    pub text_id: i16,
}
melee_core::gc_layout!(TrainingItemEntry, "TrainingItemEntry", size 0x4, align 2, [
    item_id @ 0x0: 2 "s16",
    text_id @ 0x2: 2 "s16",
]);

/// `TrainingSpeedArray` (24 bytes).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct TrainingSpeedArray {
    pub values: [f32; 6],
}
melee_core::gc_layout!(TrainingSpeedArray, "TrainingSpeedArray", size 0x18, align 4, [
    values @ 0x0: 24 "f32[6]",
]);

const fn ie(item_id: ItemKind, text_id: i16) -> TrainingItemEntry {
    TrainingItemEntry { item_id: item_id as i16, text_id }
}
pub static gmTraining_ItemTable: [TrainingItemEntry; 31] = [
    ie(It_Kind_Foods, 21),
    ie(It_Kind_Tomato, 11),
    ie(It_Kind_Heart, 10),
    ie(It_Kind_L_Gun, 18),
    ie(It_Kind_F_Flower, 35),
    ie(It_Kind_S_Scope, 25),
    ie(It_Kind_StarRod, 26),
    ie(It_Kind_LipStick, 19),
    ie(It_Kind_Sword, 14),
    ie(It_Kind_Bat, 13),
    ie(It_Kind_Harisen, 2),
    ie(It_Kind_Hammer, 38),
    ie(It_Kind_G_Shell, 16),
    ie(It_Kind_R_Shell, 17),
    ie(It_Kind_Flipper, 24),
    ie(It_Kind_Freeze, 20),
    ie(It_Kind_Dosei, 9),
    ie(It_Kind_M_Ball, 27),
    ie(It_Kind_BombHei, 8),
    ie(It_Kind_MSBomb, 22),
    ie(It_Kind_Kinoko, 78),
    ie(It_Kind_DKinoko, 79),
    ie(It_Kind_Star, 12),
    ie(It_Kind_Parasol, 15),
    ie(It_Kind_WStar, 76),
    ie(It_Kind_MetalB, 81),
    ie(It_Kind_RabbitC, 77),
    ie(It_Kind_ScBall, 80),
    ie(It_Kind_TaruCann, 5),
    ie(It_Kind_Spycloak, 82),
    ie(It_Kind_None, 23),
];

pub static lbl_803B7C68: TrainingSpeedArray = TrainingSpeedArray { values: [2.0, 1.5, 1.0, 0.666, 0.5, 0.25] };

pub static mut lbl_804D662C: *mut *mut DynamicModelDesc = null_mut();
pub static mut lbl_804D6628: *mut HSD_Archive = null_mut();
pub static mut lbl_80473700: TrainingModeState = unsafe { core::mem::zeroed() };
pub static mut gm_80473814: CssSubStruct = unsafe { core::mem::zeroed() };

pub unsafe fn gm_80188454(idx: i32) -> i32 {
    melee_core::ct!("gm_80188454");
    // `char_data[idx]` (the C does not bound-check idx)
    *(&raw const lbl_80473700 as *const i32).offset(2 + idx as isize)
}

pub unsafe fn fn_8018846C() -> i32 {
    melee_core::ct!("fn_8018846C");
    let ptr = &raw mut lbl_80473700 as *mut i32;
    let result = ext::pl_8004134C(0);
    let p = ptr.offset(ext::Player_GetPlayerCharacter(0) as isize + 2);
    if *p < result {
        *p = result;
    }
    if result != 0 {
        *ptr.add(67) = result;
        *ptr.add(68) = 1;
    }
    if *ptr.add(68) != 0 {
        return *ptr.add(67);
    }
    result
}

pub unsafe fn fn_801884F8() -> i32 {
    melee_core::ct!("fn_801884F8");
    let state = &mut lbl_80473700;
    let mut result = ext::pl_80041300(0);
    if result != 0 {
        state.result_cache[0] = result;
        state.result_cache[1] = 1;
    }
    if state.result_cache[1] != 0 {
        result = state.result_cache[0];
    }
    result
}

pub unsafe fn fn_80188550(arg0: i32) {
    melee_core::ct!("fn_80188550");
    let state = &mut lbl_80473700;
    let current = state.count;

    if arg0 != current {
        if arg0 > state.count {
            let mut skip = state.count;
            let mut remaining = arg0 - current;

            for i in 0..4i32 {
                if i != 0 {
                    if skip == 0 {
                        // `if (i != 0) slot_type = 1; else slot_type = j;` (i != 0 here)
                        state.players[i as usize].slot_type = 1;
                        ext::gm_8016EDDC(i, &mut state.players[i as usize]);
                        remaining -= 1;
                        if remaining == 0 {
                            break;
                        }
                    } else {
                        skip -= 1;
                    }
                }
            }
        } else {
            let mut to_remove = current - arg0;
            let mut j = 3i32;
            while j >= 0 {
                if j != 0 && to_remove != 0 {
                    if current != 3 {
                        ext::fn_8016EF98(to_remove + 1);
                    } else {
                        ext::fn_8016EF98(j);
                    }
                    to_remove -= 1;
                    if to_remove == 0 {
                        break;
                    }
                }
                j -= 1;
            }
        }
        state.count = arg0;
    }
}

pub unsafe fn fn_80188644() {
    melee_core::ct!("fn_80188644");
    let mut sp10 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let state = &mut lbl_80473700;
    let mut saved_count = state.count;
    state.count = 1;
    ext::Player_SetFacingDirection(0, 1.0);
    ext::Player_SetHUDDamage(0, 0);
    ext::Stage_80224E64(0, &mut sp10);
    ext::Player_800328D4(0, &mut sp10);
    ext::ifStatus_802F6508(0);

    for i in 0..4i32 {
        if i != 0 && i <= saved_count {
            ext::fn_8016EF98(i);
            if i == 1 {
                state.players[i as usize].slot_type = 1;
                ext::gm_8016EDDC(i, &mut state.players[i as usize]);
            }
        }
    }

    let mv = &mut gm_80473814.menu_values;
    mv[0] = 0;
    mv[1] = 0;
    mv[2] = 0;
    mv[3] = 0;
    saved_count = 0;
    mv[4] = saved_count;
    mv[5] = saved_count;
    mv[6] = saved_count;
    mv[saved_count as usize] = 2;
}

/// Hundreds / tens / ones digit animations shared by the HUD counters: `jobjs` ordered
/// [hundreds, tens, ones], `val` already clamped to 0..=999.
unsafe fn show_digits(jobjs: [*mut HSD_JObj; 3], val: i32) {
    if (val / 100) != 0 {
        let temp = val;
        ext::HSD_JObjReqAnimAll(jobjs[0], (temp / 100) as f32);
    } else {
        ext::HSD_JObjReqAnimAll(jobjs[0], 10.0);
    }

    if (val / 10) != 0 {
        ext::HSD_JObjReqAnimAll(jobjs[1], ((val % 100) / 10) as f32);
    } else {
        ext::HSD_JObjReqAnimAll(jobjs[1], 10.0);
    }

    ext::HSD_JObjReqAnimAll(jobjs[2], (val % 10) as f32);

    for i in 0..3 {
        ext::HSD_JObjAnimAll(jobjs[i]);
    }
}

/// `jobjs[2] = arg0; jobjs[1] = jobjs[2]->next; jobjs[0] = jobjs[1]->next` (NULL-safe).
unsafe fn next_chain(arg0: *mut HSD_JObj) -> [*mut HSD_JObj; 3] {
    let j2 = arg0;
    let j1 = jobj_next(j2);
    let j0 = jobj_next(j1);
    [j0, j1, j2]
}

pub unsafe fn fn_80188738(arg0: *mut HSD_JObj) {
    melee_core::ct!("fn_80188738");
    let val = if ext::pl_800412D0(0) > 999 { 999 } else { ext::pl_800412D0(0) };
    show_digits(next_chain(arg0), val);
}

pub unsafe fn fn_80188910(arg0: *mut HSD_JObj) {
    melee_core::ct!("fn_80188910");
    let val = if fn_8018846C() > 999 { 999 } else { fn_8018846C() };
    show_digits(next_chain(arg0), val);
}

pub unsafe fn fn_80188B3C(arg0: *mut HSD_JObj) {
    melee_core::ct!("fn_80188B3C");
    let state = &mut lbl_80473700;
    let mut val = ext::pl_80041300(0);
    if val != 0 {
        state.result_cache[0] = val;
        state.result_cache[1] = 1;
    }
    if state.result_cache[1] != 0 {
        val = state.result_cache[0];
    }

    if val > 999 {
        val = 999;
    } else {
        val = fn_801884F8();
    }
    show_digits(next_chain(arg0), val);
}

pub unsafe fn fn_80188D3C(arg0: *mut HSD_JObj) {
    melee_core::ct!("fn_80188D3C");
    let val = gm_80473814.menu_values[4];

    let j2 = jobj_child(arg0);
    let j1 = jobj_next(j2);
    let j0 = jobj_next(j1);
    let jobjs = [j0, j1, j2];

    let hundreds = val / 100;
    if hundreds != 0 {
        ext::HSD_JObjReqAnimAll(jobjs[0], hundreds as f32);
    } else {
        ext::HSD_JObjReqAnimAll(jobjs[0], 10.0);
    }

    if (val / 10) != 0 {
        ext::HSD_JObjReqAnimAll(jobjs[1], ((val - (hundreds * 100)) / 10) as f32);
    } else {
        ext::HSD_JObjReqAnimAll(jobjs[1], 10.0);
    }

    ext::HSD_JObjReqAnimAll(jobjs[2], (val % 10) as f32);

    for i in 0..3 {
        ext::HSD_JObjAnimAll(jobjs[i]);
    }
}

/// `sub->text->pos_x = 12.0f * (9.798828f + tx) + 50.0f` (fused in the shipping build).
#[inline]
fn text_x(tx: f32) -> f32 {
    12.0f32.mul_add(9.798828f32 + tx, 50.0)
}

pub unsafe extern "C" fn fn_80188EE8(_gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_80188EE8");
    let state = &mut lbl_80473700;
    let sub = &raw mut gm_80473814;

    if ext::gm_GetDbPauseFlag(2) != 0 {
        ext::HSD_SisLib_803A6368((*sub).text, 0x1E);
        ext::HSD_JObjSetFlagsAll((*sub).jobjs[3], JOBJ_HIDDEN as u32);
    } else {
        ext::HSD_JObjClearFlagsAll((*sub).jobjs[3], JOBJ_HIDDEN as u32);
    }

    {
        let counter = (*sub).anim_frames[22] as u32;
        if counter < 10 || (counter >= 20 && counter < 30) {
            (*sub).anim_frames[22] = (*sub).anim_frames[22].wrapping_add(1);
            (*sub).anim_frames[1] = (*sub).anim_frames[22];
        }
    }

    ext::HSD_JObjReqAnimAll((*sub).jobjs[22], (*sub).anim_frames[22] as u32 as f32);
    ext::HSD_JObjAnimAll((*sub).jobjs[22]);
    ext::HSD_JObjReqAnimAll((*sub).jobjs[1], (*sub).anim_frames[1] as u32 as f32);
    ext::HSD_JObjAnimAll((*sub).jobjs[1]);

    (*(*sub).text).pos_x = text_x(mi::HSD_JObjGetTranslationX(&*(*sub).jobjs[1]));
    (*(*sub).text).pos_y = 150.0;

    fn_80188738((*sub).jobjs[9]);
    fn_80188910((*sub).jobjs[5]);
    fn_80188B3C((*sub).jobjs[19]);

    let mut jobj = (*sub).jobjs[12];
    ext::HSD_JObjReqAnimAll(jobj, (*sub).x03 as f32);
    ext::HSD_JObjAnimAll(jobj);

    jobj = (*sub).jobjs[13];
    ext::HSD_JObjReqAnimAll(jobj, (*sub).menu_values[0] as u32 as f32);
    ext::HSD_JObjAnimAll(jobj);

    jobj = (*sub).jobjs[23];
    ext::HSD_JObjReqAnimAll(jobj, (*sub).x00 as f32);
    ext::HSD_JObjAnimAll(jobj);

    jobj = (*sub).jobjs[34];
    ext::HSD_JObjReqAnimAll(jobj, (*sub).menu_values[0] as u32 as f32);
    ext::HSD_JObjAnimAll(jobj);

    let mut val = (*sub).menu_values[1];
    let text = (*sub).text;
    if ext::lbLang_IsSettingUS() != 0 && val == 0x13 {
        ext::HSD_SisLib_803A6368(text, 0x17);
    } else {
        ext::HSD_SisLib_803A6368(text, gmTraining_ItemTable[val as usize].text_id as i32);
    }

    jobj = (*sub).jobjs[32];
    ext::HSD_JObjReqAnimAll(jobj, (*sub).menu_values[2].wrapping_add(1) as u32 as f32);
    ext::HSD_JObjAnimAll(jobj);

    jobj = (*sub).jobjs[31];
    ext::HSD_JObjReqAnimAll(jobj, (*sub).menu_values[3] as u32 as f32);
    ext::HSD_JObjAnimAll(jobj);

    fn_80188D3C((*sub).jobjs[27]);

    ext::HSD_JObjSetFlags((*sub).jobjs[25], JOBJ_HIDDEN as u32);

    val = (*sub).menu_values[6];
    let cursor_jobj = (*sub).jobjs[26];
    if val == 2 && state.mode == 3 {
        val = 3;
    }
    ext::HSD_JObjReqAnimAll(cursor_jobj, val as f32);
    ext::HSD_JObjAnimAll(cursor_jobj);
}

/// `gm_801891F4_SetCpuType`
unsafe fn set_cpu_type(sub: &mut CssSubStruct, cpu_type: i32) {
    let mut count = lbl_80473700.count;
    sub.x03 = cpu_type as u8;
    for i in 0..4i32 {
        if i != 0 && count != 0 {
            ext::Player_SetPlayerAndEntityCpuType(i, cpu_type);
            count -= 1;
            if count == 0 {
                break;
            }
        }
    }
}

#[inline]
fn sfx_move() {
    ext::lbAudioAx_80024030(2);
}

/// `case 6:`'s `switch (menu_values[6])` (also used when the menu closes).
unsafe fn apply_pause_camera(v: i32) {
    match v {
        0 => ext::fn_801650E8(),
        1 => ext::fn_80165190(0, 5),
        2 => {
            if lbl_80473700.mode == 3 {
                ext::fn_801652B0(0, 0);
            } else {
                ext::fn_801652B0(0, 3);
            }
        }
        _ => {}
    }
}

/// A menu row that wraps between 0 and `max` with left/right (`cases 0, 2, 3`): returns after
/// the first handled direction like the C does.
unsafe fn wrap_row(sub: &mut CssSubStruct, buttons: u64, max: u32) {
    let k = sub.x00 as usize;
    if buttons & PAD_ANY_LEFT != 0 {
        sfx_move();
        if sub.menu_values[k] as u32 != 0 {
            sub.menu_values[k] = sub.menu_values[k].wrapping_sub(1);
            return;
        }
        sub.menu_values[k] = max as i32;
        return;
    }
    if buttons & PAD_ANY_RIGHT != 0 {
        sfx_move();
        if (sub.menu_values[k] as u32) < max {
            sub.menu_values[k] = sub.menu_values[k].wrapping_add(1);
            return;
        }
        sub.menu_values[k] = 0;
    }
}

pub unsafe fn fn_801891F4() {
    melee_core::ct!("fn_801891F4");
    let sub: &mut CssSubStruct = &mut *(&raw mut gm_80473814);
    let buttons = ext::gm_801A36C0(lbl_80473700.mode as u8);

    if ext::gm_GetDbPauseFlag(2) != 0 {
        if sub.x01 == 0 {
            ext::fn_801651FC(0, 0);
            set_cpu_type(sub, 0);
            sub.anim_frames[22] = 0;
        }
        sub.x01 = 1;

        if buttons & PAD_ANY_UP != 0 {
            sfx_move();
            if sub.x00 != 0 {
                sub.x00 -= 1;
                if sub.x00 == 5 {
                    sub.x00 = 4;
                }
            } else {
                sub.x00 = 8;
            }
        }

        if buttons & PAD_ANY_DOWN != 0 {
            sfx_move();
            if sub.x00 < 8 {
                sub.x00 += 1;
                if sub.x00 == 5 {
                    sub.x00 = 6;
                }
            } else {
                sub.x00 = 0;
            }
        }

        match sub.x00 {
            0 => wrap_row(sub, buttons, 5),
            1 => {
                let k = sub.x00 as usize;
                if buttons & PAD_ANY_LEFT != 0 {
                    sfx_move();
                    if sub.menu_values[k] as u32 != 0 {
                        sub.menu_values[k] = sub.menu_values[k].wrapping_sub(1);
                    } else {
                        sub.menu_values[k] = 0x1D;
                    }
                } else if buttons & PAD_ANY_RIGHT != 0 {
                    sfx_move();
                    if (sub.menu_values[k] as u32) < 0x1D {
                        sub.menu_values[k] = sub.menu_values[k].wrapping_add(1);
                    } else {
                        sub.menu_values[k] = 0;
                    }
                }
                if buttons & PAD_BUTTON_A as u64 != 0 {
                    ext::lbAudioAx_80024030(8);
                    let item = gmTraining_ItemTable[sub.menu_values[1] as usize].item_id;
                    let jobj = (*ext::Player_GetEntity(0)).hsd_obj as *mut HSD_JObj;
                    let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
                    mi::HSD_JObjGetTranslation2(&*jobj, &mut pos);
                    pos.y += 10.0;
                    ext::it_8026D258(&mut pos, item as i32);
                }
            }
            2 => wrap_row(sub, buttons, 2),
            3 => wrap_row(sub, buttons, 5),
            4 => {
                let k = sub.x00 as usize;
                if buttons & PAD_ANY_LEFT != 0 {
                    sfx_move();
                    if sub.menu_values[k] as u32 != 0 {
                        sub.menu_values[k] = sub.menu_values[k].wrapping_sub(1);
                        if sub.menu_values[k] as u32 == 0 {
                            ext::gm_801A36E0(0, 0x19);
                            return;
                        }
                    } else {
                        sub.menu_values[k] = 0x3E7;
                        return;
                    }
                } else if buttons & PAD_ANY_RIGHT != 0 {
                    sfx_move();
                    if (sub.menu_values[k] as u32) < 0x3E7 {
                        sub.menu_values[k] = sub.menu_values[k].wrapping_add(1);
                        if sub.menu_values[k] as u32 == 0x3E7 {
                            ext::gm_801A36E0(0, 0x19);
                            return;
                        }
                    } else {
                        sub.menu_values[k] = 0;
                        return;
                    }
                }
            }
            5 => {
                let k = sub.x00 as usize;
                if buttons & PAD_ANY_LEFT != 0 {
                    sfx_move();
                    sub.menu_values[k] = 0;
                    return;
                }
                if buttons & PAD_ANY_RIGHT != 0 {
                    sfx_move();
                    sub.menu_values[k] = 1;
                    return;
                }
            }
            6 => {
                let k = sub.x00 as usize;
                if buttons & PAD_ANY_LEFT != 0 {
                    sfx_move();
                    if sub.menu_values[k] as u32 != 0 {
                        sub.menu_values[k] = sub.menu_values[k].wrapping_sub(1);
                    } else {
                        sub.menu_values[k] = 2;
                    }
                    apply_pause_camera(sub.menu_values[k]);
                } else if buttons & PAD_ANY_RIGHT != 0 {
                    sfx_move();
                    if (sub.menu_values[k] as u32) < 2 {
                        sub.menu_values[k] = sub.menu_values[k].wrapping_add(1);
                    } else {
                        sub.menu_values[k] = 0;
                    }
                    apply_pause_camera(sub.menu_values[k]);
                }
            }
            7 => {
                if buttons & PAD_BUTTON_A as u64 != 0 {
                    ext::lbAudioAx_80024030(1); // sfxForward()
                    fn_80188644();
                }
            }
            8 => {
                if buttons & PAD_BUTTON_A as u64 != 0 {
                    ext::gm_8016B328();
                }
            }
            _ => {}
        }
    } else {
        if sub.x01 == 1 {
            let speeds = lbl_803B7C68;

            ext::lbAudioAx_80024030(0); // sfxBack()
            sub.anim_frames[22] = 0x14;
            {
                let selected_speed = speeds.values[sub.menu_values[0] as usize];
                let tick = (OS_BUS_CLOCK / 4) as f32;
                ext::lb_80019880((1.0f32 / 60.0f32 / selected_speed * tick) as u64);
            }

            fn_80188550(sub.menu_values[2] + 1);

            set_cpu_type(sub, sub.menu_values[3]);

            {
                let damage = sub.menu_values[4];
                let mut count = lbl_80473700.count;
                for i in 0..4i32 {
                    if i != 0 && count != 0 {
                        ext::Player_SetHUDDamage(i, damage);
                        count -= 1;
                        if count == 0 {
                            break;
                        }
                    }
                }
            }

            for i in 1..4i32 {
                ext::fn_8016B388(i, sub.menu_values[4]);
            }

            apply_pause_camera(sub.menu_values[6]);
        }
        sub.x01 = 0;
    }
}
/// `fn_801891F4` registered as a proc (the C casts a `void(void)` to `HSD_GObjEvent`).
pub unsafe extern "C" fn fn_801891F4_cb(_gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_801891F4_cb");
    fn_801891F4()
}
pub unsafe fn fn_80189B88() {
    melee_core::ct!("fn_80189B88");
    let sub = &mut gm_80473814;
    sub.x00 = 0;
    sub.x01 = 0;
    ext::HSD_GObj_SetupProc(ext::GObj_Create(0xE, 2, 0), Some(fn_801891F4_cb), 0x15);
    let gobj = ext::GObj_Create(0xE, 0xF, 0);
    let jobj = ext::HSD_JObjLoadJoint((**lbl_804D662C).joint);
    sub.gobj = gobj;
    ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_JObjKind, jobj as *mut c_void);
    ext::GObj_SetupGXLink(gobj, Some(ext::HSD_GObj_JObjCallback), 0xB, 0);
    ext::HSD_GObj_SetupProc(gobj, Some(fn_80188EE8), 0x11);
    ext::gm_8016895C(jobj, *lbl_804D662C, 0);
    ext::HSD_JObjReqAnimAll(jobj, 0.0);
    ext::HSD_JObjAnimAll(jobj);
    for i in 0..0x27usize {
        sub.anim_frames[i] = 0;
        ext::lb_80011E24(jobj, &mut sub.jobjs[i], i as i32, -1);
    }
    sub.anim_frames[22] = 0x1E;
    sub.anim_frames[1] = 0x1E;
    sub.menu_values[0] = 0;
    sub.menu_values[1] = 0;
    sub.menu_values[2] = 0;
    sub.menu_values[3] = 0;
    sub.menu_values[4] = 0;
    sub.menu_values[5] = 0;
    sub.menu_values[6] = 0;
    sub.menu_values[0] = 2;
}

pub unsafe fn gm_80189CDC(arg0: *mut StartMeleeData) {
    melee_core::ct!("gm_80189CDC");
    let arg0 = &mut *arg0;
    let state = &mut lbl_80473700;

    arg0.rules.set_match_kind(0);
    arg0.rules.is_teams = 1;
    arg0.rules.item_freq = -1;
    arg0.rules.x20 = 0xFFFF_FFFF_FFFF_FFFF;
    arg0.rules.set_x5_0(1);

    for i in 0..4usize {
        gm_80473814.saved_players[i] = arg0.players[i];
        state.players[i] = gm_80473814.saved_players[i];
    }

    state.mode = arg0.players[0].slot as i32 - 1;
    state.count = 1;

    for i in 0..0x1Busize {
        // `0x1B > i`: char_data holds 25 ints on the GC layout, the C writes 27 (into pad_6C)
        *(&raw mut lbl_80473700 as *mut i32).add(2 + i) = 0;
    }
}

pub unsafe fn fn_8018A000() -> *mut HSD_Text {
    melee_core::ct!("fn_8018A000");
    let state = &mut lbl_80473700;
    ext::memzero(state.result_cache.as_mut_ptr() as *mut c_void, core::mem::size_of_val(&state.result_cache) as i32);
    lbl_804D6628 = ext::lbArchive_80016DBC(
        c"GmTrain".as_ptr(),
        &[
            &raw mut lbl_804D662C as *mut c_void,
            c"ScGamTraining_scene_models".as_ptr() as *mut c_void,
        ],
    );
    fn_80189B88();
    ext::ifAll_GetHUDGObj();
    ext::HSD_SisLib_803A611C(0, null_mut(), 9, 0x14, 0, 0xE, 0, 0x12);
    if ext::lbLang_IsSavedLanguageUS() != 0 {
        ext::HSD_SisLib_803A62A0(0, c"SdTrain.usd".as_ptr() as *mut c_char, c"SIS_TrainingData".as_ptr() as *mut c_char);
    } else {
        ext::HSD_SisLib_803A62A0(0, c"SdTrain.dat".as_ptr() as *mut c_char, c"SIS_TrainingData".as_ptr() as *mut c_char);
    }

    let text_ptr = &raw mut gm_80473814.text;
    *text_ptr = ext::HSD_SisLib_803A5ACC(
        0,
        0,
        text_x(mi::HSD_JObjGetTranslationX(&*gm_80473814.jobjs[1])),
        150.0,
        0.1,
        167.0,
        16.0,
    );
    let text = *text_ptr;
    ext::lbLang_IsSettingUS();
    ext::HSD_SisLib_803A6368(text, gmTraining_ItemTable[0].text_id as i32);
    (**text_ptr).default_fitting = 1;
    // resetText
    (**text_ptr).x34.x = 0.7;
    (**text_ptr).x34.y = 0.6;
    let text = *text_ptr;
    (**text_ptr).default_alignment = 2;
    text
}

// ---- folded from gm_ptr.rs (consolidation steps 21/22) ----
/// gm_1884.c `resetText`.
pub fn resetText(text: &mut HSD_Text_gm_ptr) {
    melee_core::ct!("resetText");
    text.x34.x = 0.7;
    text.x34.y = 0.6;
}
