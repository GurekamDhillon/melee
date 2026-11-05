//! Port of src/melee/if/if_2FFC.c, split from soundtest.c upstream (doldecomp 598987cf3): fn_802FFE0C, fn_802FFE6C, un_802FFCD0, un_802FFD94, un_802FFEA4, un_802FFEE0, un_802FFF2C, un_80300218, un_80300248, un_80300290, un_80300338, un_80300378, un_803003C4, un_80300410.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, clippy::all)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use super::soundtest::*;
use super::soundtest::ext;

/// `void un_802FFCD0(int count, void* data)`: fills `data[0..count]` with `data[count]`.
pub unsafe extern "C" fn un_802FFCD0(count: i32, data: *mut c_void) {
    melee_core::ct!("un_802FFCD0");
    let arr = data as *mut i32;
    let mut i = 0;
    while i < count {
        *arr.offset(i as isize) = *arr.offset(count as isize);
        i += 1;
    }
}

/// `void un_802FFD94(int arg0, void* arg1, soundtest_callback arg2)`: opens a sub-menu on confirm.
pub unsafe extern "C" fn un_802FFD94(arg0: i32, arg1: *mut c_void, arg2: soundtest_callback) {
    melee_core::ct!("un_802FFD94");
    if arg0 == 1 {
        let d = ext::un_80302DF0();
        sfx_forward();
        ext::un_80304210(d, arg1, 0, -60, 0);
        ext::un_80302DF8(ext::un_80302DF0(), arg2);
    }
}

pub unsafe extern "C" fn fn_802FFE0C(arg0: soundtest_callback_arg0) -> i32 {
    melee_core::ct!("fn_802FFE0C");
    match arg0 {
        0 => {
            sfx_back();
            ext::un_80304334(ext::un_80302DF0());
        }
        6 => {
            sfx_forward();
            ext::gm_SetNextGameModeStateId(4);
            ext::gm_801A4B60();
        }
        _ => {}
    }
    0
}

pub unsafe extern "C" fn fn_802FFE6C(update_scene: soundtest_callback_arg0) -> i32 {
    melee_core::ct!("fn_802FFE6C");
    let mut res = update_scene as i32;
    if update_scene == 0 {
        sfx_back();
        ext::un_80304334(ext::un_80302DF0());
        res = 0;
    }
    res
}

pub unsafe extern "C" fn un_802FFEA4(update_scene: soundtest_callback_arg0) -> i32 {
    melee_core::ct!("un_802FFEA4");
    if update_scene == 1 {
        sfx_forward();
        ext::gm_SetNextGameModeStateId(0x4);
        ext::gm_801A4B60();
    }
    0
}

/// `void un_802FFEE0(struct UnkSoundTestData0*)`
pub unsafe extern "C" fn un_802FFEE0(arg0: *mut UnkSoundTestData0) {
    melee_core::ct!("un_802FFEE0");
    let v = un_803FA258.x12C as u16;
    let val: u16 = if (v as i32) < 0x3E { v } else { 0x3E };
    (*arg0).x0 = val;
    (*arg0).x2 = un_803FA258.x130 as u16;
    (*arg0).x4 = 0x98967F;
    (*arg0).x8 = 0;
}

/// `void un_802FFF2C(StartMeleeData*)`: the settings block -> a match start description. The C reads the
/// block through the `un_803FA128_x130_t` overlay of `&un_803FA258`.
pub unsafe extern "C" fn un_802FFF2C(arg0: *mut StartMeleeData) {
    melee_core::ct!("un_802FFF2C");
    let r = &raw mut (*arg0).rules;
    let s = &raw mut un_803FA258 as *mut un_803FA128_x130_t;
    ext::gm_SetupRulesDefaults(r);
    (*r).set_x2_2(0);
    (*r).is_teams = (*s).xC as u8;
    match (*s).xC8 {
        0 | 2 => {
            (*r).set_match_kind(if (*s).xC8 == 0 { 0 } else { 2 });
            let timer = (*s).xCC[1].wrapping_add((*s).xCC[0].wrapping_mul(0x3C)) as u16;
            if timer != 0 {
                (*r).time_limit = timer as u32;
                (*r).set_timer_enabled(1);
            } else {
                (*r).set_timer_enabled(0);
            }
        }
        1 => {
            (*r).set_match_kind(1);
            (*r).set_timer_enabled(0);
        }
        _ => {
            (*r).set_match_kind(0);
            (*r).set_timer_enabled(0);
        }
    }
    (*r).stkind = (*s).x8 as u16;
    (*r).x20 = -1i64 as u64;
    (*r).item_freq = (*s).xCC[3].wrapping_sub(1) as i8;
    (*r).sd_penalty = -1;
    (*r).x30 = (*s).xEC;
    ext::gm_SetupAllPlayerDefaults((*arg0).players.as_mut_ptr());
    let sp = s;
    let mut i = 0usize;
    while i < 4 {
        let p = &mut (*arg0).players[i];
        p.ckind = (*sp).x10[i] as i8;
        p.slot_type = (*sp).x24[i] as u8;
        p.color = (*sp).x38[i] as u8;
        p.sub_color = (*sp).x48[i] as u8;
        p.team = (*sp).x58[i] as u8;
        p.set_rumble_enabled((*sp).xDC[i] as u8);
        p.damage1 = (*sp).x68[i] as u16;
        p.attack_ratio = (*sp).x78[i];
        p.defense_ratio = (*sp).x88[i];
        p.cpu_kind = (*sp).xA8[i] as u8;
        p.cpu_level = (*sp).xB8[i] as u8;
        p.stocks = (*s).xCC[2] as i8;
        p.set_xC_b1(0);
        p.model_scale = (*sp).x98[i];
        i += 1;
    }
}

pub unsafe extern "C" fn un_80300218(_arg0: soundtest_callback_arg0) -> i32 {
    melee_core::ct!("un_80300218");
    ext::lbLang_SetSavedLanguage(un_803FA258.x0);
    0
}

pub unsafe extern "C" fn un_80300248(arg0: soundtest_callback_arg0) -> i32 {
    melee_core::ct!("un_80300248");
    if un_803FA258.x4[0] != 0 && arg0 == 1 {
        sfx_forward();
        ext::gmMainLib_8015FB68();
    }
    0
}

pub unsafe extern "C" fn un_80300290(arg0: soundtest_callback_arg0) -> i32 {
    melee_core::ct!("un_80300290");
    if arg0 == 1 {
        let x = ext::un_80302DF0();
        sfx_forward();
        ext::un_80304210(x, &raw mut un_803FA658 as *mut c_void, 0, -60, 0);
        ext::un_80302DF8(ext::un_80302DF0(), Some(fn_802FFE6C));
    }
    0
}

pub unsafe extern "C" fn un_80300338(_arg0: soundtest_callback_arg0) -> i32 {
    melee_core::ct!("un_80300338");
    load_f4(save_base());
    0
}

pub unsafe extern "C" fn un_80300378(_arg0: soundtest_callback_arg0) -> i32 {
    melee_core::ct!("un_80300378");
    un_803FA258.xF0 &= 0xFFFE;
    load_f4(save_base());
    0
}

pub unsafe extern "C" fn un_803003C4(_arg0: soundtest_callback_arg0) -> i32 {
    melee_core::ct!("un_803003C4");
    un_803FA258.xF0 &= 0xFFFC;
    load_f4(save_base());
    0
}

pub unsafe extern "C" fn un_80300410(arg0: soundtest_callback_arg0) -> i32 {
    melee_core::ct!("un_80300410");
    if arg0 == 1 {
        sfx_forward();
        let dst = save_base();
        *dst.add(0x1868) = un_803FA258.xF4;
        *dst.add(0x1869) = un_803FA258.xF5;
        *dst.add(0x186A) = un_803FA258.xF6;
        *dst.add(0x186B) = un_803FA258.xF7;
    }
    0
}
