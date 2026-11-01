//! Port of src/melee/mn/mn_22EC.c: menu helpers: looping JObj animation updaters, PLink GObj
//! teardown, the current-frame probe (`mn_8022F298`), the `start`-pressed check and the scalar
//! step-toward-target helpers. No fused sites (tools/rust-port/fma_sites.py: 0 fmuladd).
//! Callees of unported TUs are `ext` hooks named like the C (unset: panics "TODO").
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, dead_code)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types as mt;
use core::ffi::c_void;
use core::ptr::null_mut;

use melee_types::*;

pub mod ext {
    #![allow(non_upper_case_globals, non_snake_case, unused_imports, dead_code)]
    use super::*;
    macro_rules! ext_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)? $(= $def:path)?;)*) => {
            pub mod hook {
                use super::*;
                $( pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } {
                    Some(f) => f($($a),*),
                    None => { $crate::ext_dflt!([$($def)?] ($($a),*) $name) }
                }
            } )*
        };
    }
    ext_fns! {
    fn HSD_FObjStopAnim(a0: *mut HSD_FObj, a1: *mut c_void, a2: HSD_ObjUpdateFunc, a3: f32);
    fn HSD_ForeachAnim(a0: *mut c_void, a1: u32, a2: u32, a3: unsafe extern "C" fn(*mut HSD_AObj, *mut c_void, u32), a4: u32, a5: i32);
    fn HSD_GObjFree(a0: *mut HSD_GObj);
    fn HSD_GObj_80390CD4(a0: *mut HSD_GObj);
    fn HSD_JObjAnim(a0: *mut HSD_JObj);
    fn HSD_JObjAnimAll(a0: *mut HSD_JObj);
    fn HSD_JObjReqAnim(a0: *mut HSD_JObj, a1: f32);
    fn HSD_JObjReqAnimAll(a0: *mut HSD_JObj, a1: f32);
    fn HSD_SisLib_803A5E70() = crate::tu::adapt::HSD_SisLib_803A5E70__5f9c7d;
    fn gmMainLib_GetGameRules() -> *mut GameRules = crate::tu::adapt::gmMainLib_GetGameRules__34f6cb;
    fn gm_80190EA4() = crate::tu::adapt::gm_80190EA4__7bb941;
    fn gm_GetButtonsTriggered(a0: u8) -> u64 = crate::tu::adapt::gm_GetButtonsTriggered__dca604;
    fn gm_GetCurrentGameMode() -> u8 = crate::tu::adapt::gm_GetCurrentGameMode__f56b36;
    fn mnCharSel_802640A0() -> i32 = crate::tu::adapt::mnCharSel_802640A0__c8d517;
    fn mn_80229894(a0: i32, a1: u16, a2: i32) = crate::tu::adapt::mn_80229894__f7df64;
    fn mn_8022EBDC() = crate::tu::adapt::mn_8022EBDC__a19b61;
    }
    /// `HSD_GObjPLinkHead` (baselib/gobjplink.c data): the per-link GObj list heads.
    pub use melee_runtime::tu::gobj_native::HSD_GObjPLinkHead;
}

/// `PAD_LR_START` (dolphin/pad.h)
const PAD_LR_START: u64 = 1u64 << 34;
/// `PAD_MAX_CONTROLLERS`
const PAD_MAX_CONTROLLERS: u8 = 4;

/// `mn_8022EC18`
pub fn mn_8022EC18(arg0: *mut HSD_JObj, arg1: *mut AnimLoopSettings, arg2: HSD_TypeMask) -> f32 {
    melee_core::ct!("mn_8022EC18");
    unsafe {
        let var_f31: f32;

        let temp_f1 = mn_8022F298(arg0);
        if !((*arg1).start_frame <= temp_f1) || !(temp_f1 <= (*arg1).end_frame) {
            ext::HSD_JObjReqAnimAll(arg0, core::ptr::read_volatile(&(*arg1).start_frame));
            mn_8022F3D8(arg0, 0xFF, arg2);
        }
        if (*arg1).loop_frame == -0.1f32 {
            let mut v = mn_8022F298(arg0);
            if v < (*arg1).end_frame {
                ext::HSD_JObjAnimAll(arg0);
                v = mn_8022F298(arg0);
                if v > (*arg1).end_frame {
                    v = (*arg1).end_frame;
                    ext::HSD_JObjReqAnimAll(arg0, v);
                    mn_8022F3D8(arg0, 0xFF, arg2);
                    ext::HSD_JObjAnimAll(arg0);
                }
            }
            var_f31 = v;
        } else {
            ext::HSD_JObjAnimAll(arg0);
            let mut v = mn_8022F298(arg0);
            if v >= (*arg1).end_frame {
                v = (*arg1).loop_frame + (v - (*arg1).end_frame);
                ext::HSD_JObjReqAnimAll(arg0, v);
                mn_8022F3D8(arg0, 0xFF, arg2);
                ext::HSD_JObjAnimAll(arg0);
            }
            var_f31 = v;
        }
        var_f31
    }
}

/// `mn_8022ED6C`: updates a jobj animation in accordance to the passed in anim loop setting;
/// returns the current frame of the jobj animation
pub fn mn_8022ED6C(jobj: *mut HSD_JObj, anim_loop: *mut AnimLoopSettings) -> f32 {
    melee_core::ct!("mn_8022ED6C");
    unsafe {
        let mut cur_frame = mn_8022F298(jobj);
        if !((*anim_loop).start_frame <= cur_frame && cur_frame <= (*anim_loop).end_frame) {
            ext::HSD_JObjReqAnimAll(jobj, core::ptr::read_volatile(&(*anim_loop).start_frame));
        }
        if (*anim_loop).loop_frame == -0.1f32 {
            cur_frame = mn_8022F298(jobj);
            let mut end_frame = (*anim_loop).end_frame;
            if cur_frame < end_frame {
                ext::HSD_JObjAnimAll(jobj);
                cur_frame = mn_8022F298(jobj);
                end_frame = (*anim_loop).end_frame;
                if cur_frame > end_frame {
                    cur_frame = end_frame;
                    ext::HSD_JObjReqAnimAll(jobj, cur_frame);
                    ext::HSD_JObjAnimAll(jobj);
                }
            }
        } else {
            ext::HSD_JObjAnimAll(jobj);
            cur_frame = mn_8022F298(jobj);
            let end_frame = (*anim_loop).end_frame;
            if cur_frame >= end_frame {
                cur_frame = (*anim_loop).loop_frame + (cur_frame - end_frame);
                ext::HSD_JObjReqAnimAll(jobj, cur_frame);
                ext::HSD_JObjAnimAll(jobj);
            }
        }
        cur_frame
    }
}

/// `mn_8022EE84`
pub fn mn_8022EE84(arg0: *mut HSD_JObj, arg1: *mut AnimLoopSettings, arg2: HSD_TypeMask) -> f32 {
    melee_core::ct!("mn_8022EE84");
    unsafe {
        let var_f31: f32;

        let temp_f1 = mn_8022F298(arg0);
        if !((*arg1).start_frame <= temp_f1) || !(temp_f1 <= (*arg1).end_frame) {
            ext::HSD_JObjReqAnim(arg0, core::ptr::read_volatile(&(*arg1).start_frame));
            mn_8022F3D8(arg0, 0xFF, arg2);
        }
        if (*arg1).loop_frame == -0.1f32 {
            let mut v = mn_8022F298(arg0);
            if (*arg1).end_frame != v {
                ext::HSD_JObjAnim(arg0);
                v = mn_8022F298(arg0);
                let temp_f0 = (*arg1).end_frame;
                if v > temp_f0 {
                    v = temp_f0;
                    ext::HSD_JObjReqAnim(arg0, v);
                    mn_8022F3D8(arg0, 0xFF, arg2);
                    ext::HSD_JObjAnim(arg0);
                }
            }
            var_f31 = v;
        } else {
            ext::HSD_JObjAnim(arg0);
            let mut v = mn_8022F298(arg0);
            let temp_f0_2 = (*arg1).end_frame;
            if v >= temp_f0_2 {
                v = (*arg1).loop_frame + (v - temp_f0_2);
                ext::HSD_JObjReqAnim(arg0, v);
                mn_8022F3D8(arg0, 0xFF, arg2);
                ext::HSD_JObjAnim(arg0);
            }
            var_f31 = v;
        }
        var_f31
    }
}

/// `mn_8022EFD8`: updates a jobj looping animation. If the end frame is -0.1f, the animation
/// will stop after reaching the end frame.
pub fn mn_8022EFD8(arg0: *mut HSD_JObj, arg1: *mut AnimLoopSettings) -> f32 {
    melee_core::ct!("mn_8022EFD8");
    unsafe {
        let var_f31: f32;

        let temp_f1 = mn_8022F298(arg0);
        if !((*arg1).start_frame <= temp_f1) || !(temp_f1 <= (*arg1).end_frame) {
            ext::HSD_JObjReqAnim(arg0, core::ptr::read_volatile(&(*arg1).start_frame));
        }
        if (*arg1).loop_frame == -0.1f32 {
            let mut v = mn_8022F298(arg0);
            if (*arg1).end_frame != v {
                ext::HSD_JObjAnim(arg0);
                v = mn_8022F298(arg0);
                if v > (*arg1).end_frame {
                    v = (*arg1).end_frame;
                    ext::HSD_JObjReqAnim(arg0, v);
                    ext::HSD_JObjAnim(arg0);
                }
            }
            var_f31 = v;
        } else {
            ext::HSD_JObjAnim(arg0);
            let mut v = mn_8022F298(arg0);
            let temp_f0_2 = (*arg1).end_frame;
            if v >= temp_f0_2 {
                v = (*arg1).loop_frame + (v - temp_f0_2);
                ext::HSD_JObjReqAnim(arg0, v);
                ext::HSD_JObjAnim(arg0);
            }
            var_f31 = v;
        }
        var_f31
    }
}

/// `mn_8022F0F0`: frees every GObj in PLink `arg0 & 0xFF`
pub fn mn_8022F0F0(arg0: i32) {
    melee_core::ct!("mn_8022F0F0");
    unsafe {
        let mut curr = *ext::HSD_GObjPLinkHead.add(arg0 as u8 as usize);
        while !curr.is_null() {
            let next = (*curr).next;
            ext::HSD_GObjFree(curr);
            curr = next;
        }
    }
}

/// `mn_8022F138`
pub fn mn_8022F138(arg0: u16, arg1: u16) {
    melee_core::ct!("mn_8022F138");
    let mut i = arg0 as i32;
    while i <= arg1 as i32 {
        mn_8022F0F0(i);
        i += 1;
    }
}

/// `mn_8022F1A8_inline` (static inline)
fn mn_8022F1A8_inline(arg0: u8) {
    unsafe {
        let mut curr = *ext::HSD_GObjPLinkHead.add(arg0 as usize);
        while !curr.is_null() {
            let next = (*curr).next;
            ext::HSD_GObj_80390CD4(curr);
            curr = next;
        }
    }
}

/// `mn_8022F1A8`
pub fn mn_8022F1A8(arg0: u16, arg1: u16) {
    melee_core::ct!("mn_8022F1A8");
    let mut i = arg0 as i32;
    while i <= arg1 as i32 {
        mn_8022F1A8_inline(i as u8);
        i += 1;
    }
}

/// `mn_8022F218`
pub fn mn_8022F218() -> bool {
    melee_core::ct!("mn_8022F218");
    if ext::gm_GetButtonsTriggered(PAD_MAX_CONTROLLERS) & PAD_LR_START != 0 {
        return true;
    }
    false
}

/// `mn_8022F268`
pub fn mn_8022F268() {
    melee_core::ct!("mn_8022F268");
    unsafe { (*ext::gmMainLib_GetGameRules()).force_main_menu = 1 };
}

/// `mn_8022F298`: returns the current frame of a jobj animation (-1.0 when it has none)
pub fn mn_8022F298(jobj: *mut HSD_JObj) -> f32 {
    melee_core::ct!("mn_8022F298");
    unsafe {
        let mut jobj = jobj;
        let mut aobj = (*jobj).aobj;
        if !aobj.is_null() {
            return (*aobj).curr_frame;
        }
        let dobj = (*jobj).u.dobj;
        if !dobj.is_null() {
            let mobj = (*dobj).mobj;
            if !mobj.is_null() {
                aobj = (*mobj).aobj;
                if !aobj.is_null() {
                    return (*aobj).curr_frame;
                }
                let tobj = (*mobj).tobj;
                if !tobj.is_null() {
                    aobj = (*tobj).aobj;
                    if !aobj.is_null() {
                        return (*aobj).curr_frame;
                    }
                }
            }
        }
        if (*jobj).flags & 0x1000 == 0 {
            jobj = (*jobj).child;
            while !jobj.is_null() {
                let frame = mn_8022F298(jobj);
                if frame != -1.0f32 {
                    return frame;
                }
                jobj = (*jobj).next;
            }
        }
        -1.0f32
    }
}

/// `mn_8022F360`: the `HSD_ForeachAnim` callback of `mn_8022F3D8`
pub unsafe extern "C" fn mn_8022F360(aobj: *mut HSD_AObj, obj: *mut c_void, arg2: u32) {
    melee_core::ct!("mn_8022F360");
    let mut fobj = (*aobj).fobj;
    while !fobj.is_null() {
        if (*fobj).obj_type as u32 == arg2 || arg2 == 0xFF {
            ext::HSD_FObjStopAnim(fobj, obj, None, 0.0);
        }
        fobj = (*fobj).next;
    }
}

/// `mn_8022F3D8`
pub fn mn_8022F3D8(jobj: *mut HSD_JObj, arg1: u8, mask: HSD_TypeMask) {
    melee_core::ct!("mn_8022F3D8");
    ext::HSD_ForeachAnim(jobj as *mut c_void, JOBJ_TYPE as u32, mask as u32, mn_8022F360, AOBJ_ARG_AOU as u32, arg1 as i32);
}

/// `mn_8022F410`: moves `*x` toward `*target` in increments of `dx`. Returns +1 if `*x` was
/// increased toward the target, -1 if it was decreased, 0 if it was already at the target.
pub fn mn_8022F410(x: *mut f32, target: *const f32, dx: f32) -> i32 {
    melee_core::ct!("mn_8022F410");
    unsafe {
        if *x == *target {
            return 0;
        }
        if *x < *target {
            let a = *x + dx;
            *x = if a > *target { *target } else { a };
            1
        } else {
            let a = *x - dx;
            *x = if a < *target { *target } else { a };
            -1
        }
    }
}

/// `mn_8022F470`: same as `mn_8022F410`, but for int values.
pub fn mn_8022F470(x: *mut i32, target: *const i32, dx: i32) -> i32 {
    melee_core::ct!("mn_8022F470");
    unsafe {
        if *x == *target {
            return 0;
        }
        if *x < *target {
            let a = (*x).wrapping_add(dx);
            *x = if a > *target { *target } else { a };
            1
        } else {
            let a = (*x).wrapping_sub(dx);
            *x = if a < *target { *target } else { a };
            -1
        }
    }
}

/// `mn_8022F4CC`
pub fn mn_8022F4CC() {
    melee_core::ct!("mn_8022F4CC");
    if ext::gm_GetCurrentGameMode() as u32 == GM_TOURNAMENT as u32 {
        ext::HSD_SisLib_803A5E70();
        ext::mn_8022EBDC();
        ext::gm_80190EA4();
        return;
    }
    if ext::gm_GetCurrentGameMode() as u32 == GM_MENU as u32 {
        ext::mn_80229894(2, 3, 3);
        return;
    }
    ext::HSD_SisLib_803A5E70();
    ext::mn_8022EBDC();
    ext::mnCharSel_802640A0();
}

// ---- folded from mn_ptr.rs (consolidation steps 21/22) ----
/// mn_22EC.c `mn_8022F410_fp`: step `*x` toward `*target` by `dx`; returns the direction.
pub fn mn_8022F410_fp(x: &mut f32, target: &f32, dx: f32) -> i32 {
    melee_core::ct!("mn_8022F410_fp");
    let t = *target;
    if *x == t {
        return 0;
    }
    if *x < t {
        let a = *x + dx;
        *x = if a > t { t } else { a };
        1
    } else {
        let a = *x - dx;
        *x = if a < t { t } else { a };
        -1
    }
}

/// mn_22EC.c `mn_8022F470_fp`: the integer version (`-fwrapv`).
pub fn mn_8022F470_fp(x: &mut i32, target: &i32, dx: i32) -> i32 {
    melee_core::ct!("mn_8022F470_fp");
    let t = *target;
    if *x == t {
        return 0;
    }
    if *x < t {
        let a = x.wrapping_add(dx);
        *x = if a > t { t } else { a };
        1
    } else {
        let a = x.wrapping_sub(dx);
        *x = if a < t { t } else { a };
        -1
    }
}
