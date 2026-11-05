//! Native port of src/melee/sfx/crowdsfx.c: the match crowd's reactions (gasps and cheers) to
//! knockback, blast-zone proximity and recoveries, driven by `CrowdConfig` (PlCo `pData[21]`).
//!
//! Lives in melee_fighter rather than melee_runtime because nearly every callee is a fighter-lib
//! or player accessor (`ftLib_*`, `Player_8003248C`, `pl_8003FDA0`) and `gCrowdConfig` is owned by
//! the fighter loader (`crate::tu::fighter`). The globals defined here are `crowdsfx` and
//! `crowdsfx_ptr`. Calls out of the TU go through `ext`; ported callees forward to the port,
//! the rest panic with a TODO until their TU is ported.
//!
//! Fused sites (`tools/rust-port/fma_sites.py src/melee/sfx/crowdsfx.c`): none (float adds feed
//! compares only).
//!
//! The C falls off the end of `un_803224DC` / `un_80322598` after calling `un_8032201C`; those
//! return that call's result (what the PPC leaves in r3).
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, clippy::missing_safety_doc, unused_unsafe)]

use crate::types::*;

type G = *mut HSD_GObj;

/// Calls out of the TU. `hook::<name>` overrides; unset = the port, or a TODO panic.
pub mod ext {
    use super::*;
    macro_rules! ext_fns {
        ($($name:ident($($a:ident: $t:ty),*) $(-> $r:ty)? = $def:expr;)*) => {
            pub mod hook {
                #[allow(unused_imports)] use super::*;
                $( #[allow(non_upper_case_globals)] pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( #[allow(non_snake_case)] pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } { Some(f) => f($($a),*), None => ($def)($($a),*) }
            } )*
        };
    }
    ext_fns! {
        GObj_Create(class: u32, p_link: u32, prio: u32) -> G = |a0, a1, a2| unsafe { melee_runtime::tu::gobjplink::GObj_Create(a0 as u16, a1 as u8, a2 as u8) as _ };
        HSD_GObj_SetupProc(g: G, f: Option<unsafe extern "C" fn(G)>, prio: u32) = |a0, a1, a2| unsafe { let _ = melee_runtime::tu::gobjproc::HSD_GObj_SetupProc(a0 as _, core::mem::transmute(a1), a2 as u8); };
        HSD_GObjPLinkHead(link: i32) -> G = crate::hooks::plink_head;
        lbAudioAx_80023710(id: i32) -> i32 = |id: i32| -> i32 { unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80023710(id as _) as _ } };
        lbAudioAx_800236B8(id: i32) = |id: i32| { unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_800236B8(id as _); } };
        lbAudioAx_800240B4(sfx: i32) -> i32 = |sfx: i32| -> i32 { unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_800240B4(sfx as _) as _ } };
        lbAudioAx_8002411C(sfx: i32) -> i32 = |sfx: i32| -> i32 { unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_8002411C(sfx as _) as _ } };
        mpLib_80458868() -> *mut mpCollisionBox = || todo!("TODO(port) mpLib_80458868 (mp/mplib.c data)");
        ftLib_8008732C(g: G) -> i32 = |_| todo!("ftLib_8008732C: installed by melee_link");
        ftLib_8008731C(g: G) -> i32 = crate::ftlib::ftlib_8008731c;
        ftLib_80086644(g: G, pos: &mut Vec3) = crate::ftlib::ftlib_80086644;
        ftLib_80087460(g: G) -> u32 = crate::ftlib::ftlib_80087460;
        ftLib_8008741C(i: u32) -> G = crate::ftlib::ftlib_8008741c;
        ftLib_800874BC(g: G) -> i32 = crate::ftlib::ftlib_800874bc;
        ftLib_80086BE0(g: G) -> u8 = crate::ftlib::ftlib_80086be0;
        ftLib_80087120(g: G) -> i32 = crate::ftlib::ftlib_80087120;
        ftLib_8008746C(g: G) -> i32 = crate::ftlib::ftlib_8008746c;
        ftLib_80087454(g: G) -> f32 = crate::ftlib::ftlib_80087454;
        Player_8003248C(slot: u8, sub: i32) -> i32 = |a0, a1| crate::tu::player::Player_8003248C(a0 as i32, a1) as i32;
        pl_8003FDA0(slot: u8, sub: i32) = |a0, a1| crate::tu::plbonuslib::pl_8003FDA0(a0 as i32, a1);
        pl_8003FDC8(slot: u8, sub: i32) = |a0, a1| crate::tu::plbonuslib::pl_8003FDC8(a0 as i32, a1);
    }
}

/// `gCrowdConfig` (defined by fighter.c; the loader stores it as an untyped pointer).
#[inline(always)]
pub(crate) unsafe fn cfg() -> &'static CrowdConfig {
    &*(crate::tu::fighter::gCrowdConfig as *const CrowdConfig)
}

/* 4A2F08 */
pub static mut crowdsfx: CrowdSFX_UnkStruct = unsafe { core::mem::zeroed() };
/* 4D7050 */
pub static mut crowdsfx_ptr: *mut CrowdSFX_UnkStruct = core::ptr::null_mut();

#[inline(always)]
pub(crate) unsafe fn data() -> &'static mut CrowdSFX_UnkStruct {
    &mut *crowdsfx_ptr
}

pub unsafe extern "C" fn fn_803219AC(gobj: G) {
    melee_core::ct!("fn_803219AC");
    let d = data();
    if d.x4 < 0x10000 {
        d.x4 += 1;
    }
    un_80321A00(gobj);
    un_80321AF4(gobj);
}

pub unsafe fn un_80321A00(_gobj: G) {
    melee_core::ct!("un_80321A00");
    let d = data();
    let v = cfg();

    if d.x18 >= v.max_gasp_count {
        if d.x10 < v.cheer_limit {
            d.x10 += 1;
        }
        return;
    }

    if ext::lbAudioAx_80023710(d.x2C) != 0 {
        return;
    }
    d.x18 += 1;

    if d.x18 < cfg().max_gasp_count {
        if d.x1C != 0 {
            let zero = 0;
            d.x1C = zero;
            d.x10 = zero;
            d.x18 = cfg().max_gasp_count;
            un_80321C28();
            if d.x20 != 0 {
                un_80321CA4(0x144);
                d.x20 = zero;
            }
        } else {
            un_80321BF8(d.x14);
        }
    } else {
        d.x10 = 0;
        un_80321C28();
        un_80321CA4(0x140);
    }
}

pub unsafe fn un_80321AF4(_gobj: G) {
    melee_core::ct!("un_80321AF4");
    let bx = ext::mpLib_80458868();
    let d = data();
    let old_x24 = d.x24;
    let mut flag = 0;
    let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

    d.x24 = 0;
    let mut cur = ext::HSD_GObjPLinkHead(HSD_GOBJ_PLINK_FIGHTER);

    while !cur.is_null() {
        if ext::ftLib_8008732C(cur) == 0 && ext::ftLib_8008731C(cur) == 0 {
            ext::ftLib_80086644(cur, &mut pos);

            if pos.y < cfg().blastzone_y_offset + (*bx.add(1)).bottom {
                d.x24 += 1;
            } else if d.xC == ext::ftLib_80087460(cur) {
                flag = 1;
            }
        }
        cur = (*cur).next;
    }

    if old_x24 < cfg().fighters_near_blastzone && d.x24 >= cfg().fighters_near_blastzone {
        if flag != 0 {
            un_8032201C(d.xC, 3);
        } else {
            un_80322178(3);
        }
    }
}

pub unsafe fn un_80321BF8(arg0: i32) {
    melee_core::ct!("un_80321BF8");
    data().x2C = ext::lbAudioAx_800240B4(arg0);
}

pub unsafe fn un_80321C28() {
    melee_core::ct!("un_80321C28");
    let d = data();
    if ext::lbAudioAx_80023710(d.x2C) != 0 {
        ext::lbAudioAx_800236B8(d.x2C);
    }
    d.x2C = -1;
}

#[inline]
unsafe fn un_80321C70_inline() {
    let d = data();
    if d.x18 >= cfg().max_gasp_count || d.x18 < cfg().x24 {
        return;
    }
    d.x1C = 1;
}

pub unsafe fn un_80321C70() {
    melee_core::ct!("un_80321C70");
    let d = data();
    if d.x18 >= cfg().max_gasp_count || d.x18 < cfg().x24 {
        return;
    }
    d.x1C = 1;
}

pub unsafe fn un_80321CA4(arg: i32) {
    melee_core::ct!("un_80321CA4");
    let d = data();
    un_80321CE8();
    d.x28 = ext::lbAudioAx_8002411C(arg);
}

pub unsafe fn un_80321CE8() {
    melee_core::ct!("un_80321CE8");
    let d = data();
    if ext::lbAudioAx_80023710(d.x28) != 0 {
        ext::lbAudioAx_800236B8(d.x28);
    }
    d.x28 = -1;
}

unsafe fn un_80321CE8_caller(arg0: u32) {
    let d = data();
    if ext::lbAudioAx_80023710(d.x28) != 0 {
        ext::lbAudioAx_800236B8(d.x28);
    }
    d.x28 = -1;
    d.x28 = ext::lbAudioAx_8002411C(arg0 as i32);
}

pub unsafe fn un_80321D30(arg0: u32, arg1: f32) {
    melee_core::ct!("un_80321D30");
    let d = data();
    let cat = un_80322298(arg1);
    if cat >= 2 && un_80321EBC(arg0, arg1) {
        un_80321CE8();
        return;
    }

    match cat {
        3 => un_80321CE8_caller(0x140),
        2 => un_80321CE8_caller(0x141),
        1 => un_80321CE8_caller(0x142),
        _ => {}
    }

    if arg0 != 0 && d.xC == arg0 {
        un_80321C70_inline();
    }
}

pub unsafe fn un_80321EBC(arg0: u32, arg1: f32) -> bool {
    melee_core::ct!("un_80321EBC");
    let d = data();
    let gobj = ext::ftLib_8008741C(arg0);
    if gobj.is_null() {
        return false;
    }

    let port = ext::ftLib_800874BC(gobj);
    if ext::Player_8003248C(ext::ftLib_80086BE0(gobj), port) == 1 {
        return false;
    }

    if ext::ftLib_80087120(gobj) < cfg().x1C {
        return false;
    }
    if d.x10 < cfg().cheer_limit {
        return false;
    }
    if d.xC == arg0 {
        return false;
    }

    d.x14 = ext::ftLib_8008746C(gobj);
    if d.x14 == 0x83D60 {
        return false;
    }

    {
        let data2 = data();
        if ext::lbAudioAx_80023710(data2.x2C) != 0 {
            ext::lbAudioAx_800236B8(data2.x2C);
        }
        data2.x2C = -1;
        let sfx = if un_80322298(arg1) == 3 { 0x140 } else { 0x141 };
        let data2 = data();
        data2.x2C = ext::lbAudioAx_800240B4(sfx);
    }

    d.xC = arg0;
    d.x18 = 0;

    let port = ext::ftLib_800874BC(gobj);
    ext::pl_8003FDA0(ext::ftLib_80086BE0(gobj), port);

    true
}

/// Start the crowd sound for `cat` (3..1) on channel x28, replacing the old one.
unsafe fn restart_x28(sfx: i32) {
    let d = data();
    if ext::lbAudioAx_80023710(d.x28) != 0 {
        ext::lbAudioAx_800236B8(d.x28);
    }
    d.x28 = -1;
    d.x28 = ext::lbAudioAx_8002411C(sfx);
}

pub unsafe fn un_8032201C(arg0: u32, cat: i32) -> bool {
    melee_core::ct!("un_8032201C");
    let d = data();

    match cat {
        3 => restart_x28(0x13d),
        2 => restart_x28(0x13e),
        1 => restart_x28(0x13f),
        0 => return false,
        _ => {}
    }

    if arg0 != 0 {
        if d.xC == arg0 {
            un_80321C70_inline();
        }

        let gobj = ext::ftLib_8008741C(arg0);
        ext::pl_8003FDC8(ext::ftLib_80086BE0(gobj), ext::ftLib_800874BC(gobj));
    }
    true
}

pub unsafe fn un_80322178(arg: i32) {
    melee_core::ct!("un_80322178");
    match arg {
        3 => restart_x28(0x13d),
        2 => restart_x28(0x13e),
        1 => restart_x28(0x13f),
        _ => {}
    }
}

pub unsafe fn un_803224DC(spawn_id: i32, pos_x: f32, kb_mag: f32) -> bool {
    melee_core::ct!("un_803224DC");
    let v = cfg();

    let tmp_cat = if kb_mag >= v.kb_threshold_high {
        3
    } else if kb_mag >= v.kb_threshold_mid {
        2
    } else if kb_mag >= v.kb_threshold_low {
        1
    } else {
        0
    };

    let val2c = v.horiz_margin;
    let bx = ext::mpLib_80458868();
    let val18 = (*bx.add(1)).left;
    let cat = tmp_cat;

    let out_of_bounds = pos_x < val2c + val18 || pos_x > (*bx.add(1)).right - val2c;
    if out_of_bounds {
        // The C has no `return` on this path; r3 still holds un_8032201C's result.
        un_8032201C(spawn_id as u32, cat)
    } else {
        false
    }
}

// Split into src/melee/sfx/crowdsfx_1.c upstream (doldecomp 598987cf3); re-exported for existing callers.
pub use super::crowdsfx_1::{un_80322258, un_80322298, un_803222EC, un_80322314, un_8032233C, un_80322598};
