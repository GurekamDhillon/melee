//! Port of src/melee/gm/gm_17DB.c, split from gm_17c0.c upstream (doldecomp 598987cf3): fn_8017DD7C, fn_8017DE54, fn_8017DEC8, fn_8017DF28, fn_8017DF90, fn_8017E0E4, fn_8017E160, fn_8017E21C, fn_8017E318, fn_8017E3C8, gm_8017DB58, gm_8017DB6C, gm_8017DB78, gm_8017DFF4, gm_8017E068, gm_8017E280.
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, clippy::missing_safety_doc)]

use crate::lb_r6::memzero as memzero_gm_r7;
use melee_types as mt;
use crate::gm_ptr::{CHKIND_NONE};
use crate::gm::{FT_KIND_CREZYH, FT_KIND_MASTERH};
use super::ext_hook::memzero;
use crate::ext_fns;
use core::ffi::c_void;
use melee_types::*;
use super::gm_17c0::*;
use super::gm_17c0::ext;

pub unsafe fn gm_8017DB58(arg0: *mut Unk1PData_x24) {
    melee_core::ct!("gm_8017DB58");
    for i in 0..3 {
        (*arg0.add(i)).ckind = ChKind_None as _;
    }
}

pub unsafe fn gm_8017DB6C(arg0: *mut gm_8017DB6C_arg0_t, index: i32) -> u8 {
    melee_core::ct!("gm_8017DB6C");
    (*arg0.offset(index as isize)).x0
}

pub unsafe fn gm_8017DB78(arg0: *mut gm_8017DB6C_arg0_t, index: i32) -> u8 {
    melee_core::ct!("gm_8017DB78");
    (*arg0.offset(index as isize)).x1
}

pub unsafe fn fn_8017DD7C(arg0: *mut PlayerInitData, arg1: *mut Unk1PData_x24, _arg2: u8) -> i32 {
    melee_core::ct!("fn_8017DD7C");
    let mut index: usize = 1;
    for i in 0..3 {
        if (*arg1.add(i)).ckind as i32 != ChKind_None as i32 {
            ext::gm_SetupPlayerDefaults(arg0.add(index));
            let p = arg0.add(index);
            (*p).ckind = (*arg1.add(i)).ckind as _;
            (*p).slot_type = 1;
            (*p).stocks = 1;
            (*p).team = (*arg0).team;
            (*p).color = (*arg1.add(i)).color;
            (*p).cpu_kind = (*arg1.add(i)).cpu_kind;
            (*p).cpu_level = (*arg1.add(i)).cpu_level;
            (*p).attack_ratio = (*arg1.add(i)).attack_ratio;
            (*p).defense_ratio = (*arg1.add(i)).defense_ratio;
            (*p).set_xD_b1(1);
            if (*p).ckind as i32 == CKind_GKoops as i32 {
                (*p).set_xC_b1(0);
            }
            index += 1;
        }
    }
    (index - 1) as i32
}

pub unsafe fn fn_8017DE54(arg0: u8, arg1: *mut u8) -> i32 {
    melee_core::ct!("fn_8017DE54");
    if arg0 & 0x20 != 0 {
        return 0;
    }
    if arg0 & 0x10 != 0 {
        let mut count = 0;
        // m-ex: the external-id null literal 0x21 is ChKind_None
        if *arg1 as i32 != ChKind_None as i32 {
            count = 1;
        }
        let p = arg1.add(1);
        if *p as i32 != ChKind_None as i32 {
            count += 1;
        }
        if *p.add(1) as i32 != ChKind_None as i32 {
            count += 1;
        }
        return 3 - count;
    }
    if arg0 & 2 != 0 {
        return 2;
    }
    0
}

pub unsafe fn fn_8017DEC8(arg0: i32) -> *mut Unk1PData {
    melee_core::ct!("fn_8017DEC8");
    match arg0 {
        0 => core::ptr::addr_of_mut!((*ext::gm_GetAdventureData()).x0),
        1 => core::ptr::addr_of_mut!((*ext::gm_GetAllStarData()).x0),
        2 => core::ptr::addr_of_mut!(ext::gm_80473A18.x0),
        _ => core::ptr::null_mut(),
    }
}

pub unsafe fn fn_8017DF28() -> *mut Unk1PData {
    melee_core::ct!("fn_8017DF28");
    match ext::gm_GetCurrentGameMode() as u32 {
        GM_CLASSIC => core::ptr::addr_of_mut!((*ext::gm_GetAllStarData()).x0),
        GM_ADVENTURE => core::ptr::addr_of_mut!((*ext::gm_GetAdventureData()).x0),
        GM_ALLSTAR => core::ptr::addr_of_mut!(ext::gm_80473A18.x0),
        _ => core::ptr::null_mut(),
    }
}

pub unsafe fn fn_8017DF90() -> u8 {
    melee_core::ct!("fn_8017DF90");
    match ext::gm_GetCurrentGameMode() as u32 {
        GM_CLASSIC => 1,
        GM_ADVENTURE => 0,
        GM_ALLSTAR => 2,
        _ => 3,
    }
}

pub unsafe fn gm_8017DFF4(arg0: i32) -> i32 {
    melee_core::ct!("gm_8017DFF4");
    let var_r3 = fn_8017DEC8(arg0);
    if !var_r3.is_null() {
        return (*var_r3).x0.cpu_level as i32;
    }
    -1
}

pub unsafe fn gm_8017E068() -> i32 {
    melee_core::ct!("gm_8017E068");
    let var_r3 = fn_8017DF28();
    if !var_r3.is_null() {
        return (*var_r3).x0.cpu_level as i32;
    }
    -1
}

pub unsafe fn fn_8017E0E4() -> i32 {
    melee_core::ct!("fn_8017E0E4");
    let var_r3 = fn_8017DF28();
    if !var_r3.is_null() {
        return (*var_r3).xC.xC as i32;
    }
    -1
}

pub unsafe fn fn_8017E160() -> i32 {
    melee_core::ct!("fn_8017E160");
    let var_r3 = fn_8017DF28();
    if !var_r3.is_null() {
        for i in 0..3 {
            if (*var_r3).xC.x24[i].ckind as i32 != ChKind_None as i32 {
                return 1;
            }
        }
        return 0;
    }
    0
}

pub unsafe fn fn_8017E21C() {
    melee_core::ct!("fn_8017E21C");
    for i in 0..12 {
        lbl_804706D8[i].x0 = -1;
    }
}

pub unsafe fn gm_8017E280(arg0: u16, arg1: u32) -> i8 {
    melee_core::ct!("gm_8017E280");
    let var_r7 = lbl_804706D8.as_mut_ptr();
    for index in 0..12 {
        if arg0 as i32 == (*var_r7.add(index)).x0 as i32 {
            (*var_r7.add(index)).x2 = (*var_r7.add(index)).x2.wrapping_add(arg1 as u8);
            return (*var_r7.add(index)).x2 as i8;
        }
    }
    for index in 0..12 {
        if (*var_r7.add(index)).x0 == -1 {
            (*var_r7.add(index)).x0 = arg0 as i16;
            (*var_r7.add(index)).x2 = arg1 as u8;
            return arg1 as u8 as i8;
        }
    }
    -1
}

pub unsafe fn fn_8017E318() -> i32 {
    melee_core::ct!("fn_8017E318");
    let mut sum: i32 = 0;
    for i in 0..12 {
        if lbl_804706D8[i].x0 != -1 {
            sum += lbl_804706D8[i].x2 as i32;
        }
    }
    sum
}

pub unsafe fn fn_8017E3C8() {
    melee_core::ct!("fn_8017E3C8");
    for i in 0..12 {
        if lbl_804706D8[i].x0 != -1 {
            ext::Toy_SetUnlockState(lbl_804706D8[i].x0 as enum_t, lbl_804706D8[i].x2 as i32);
        }
    }
}
