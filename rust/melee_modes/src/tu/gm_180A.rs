//! src/melee/gm/gm_180A.c: the Home-Run Contest scene (distance counter, best-record store).
//!
//! The file's statics are `pub static mut` so the state oracle (tests/tu_gm_180A.rs against
//! gen/turef/tu_gm_180A_ref.c, the C TU compiled natively with shipping contraction) can seed and
//! compare them. TARGET_PC aliases `STATE_EC8` / `STATE_X80` to `lbl_80472EC8`.
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs)]

use crate::tu_util::*;
use core::ptr::null_mut;
use melee_types::inline as mi;
use melee_types::*;

/// Callees outside this TU (hooks; unset they panic `TODO: <name> is not ported`).
pub mod ext {
    pub use crate::tu_ext::{
        lbArchive_80016DBC, lbAudioAx_80023870, lbAudioAx_800237A8, lbLang_IsSavedLanguageUS,
        GObj_Create, GObj_SetupGXLink, Ground_801C1DC0, Ground_801C57F0, HSD_GObjObject_80390A70,
        HSD_GObj_JObjCallback, HSD_GObj_SetupProc, HSD_JObjAnimAll, HSD_JObjClearFlagsAll,
        HSD_JObjLoadJoint, HSD_JObjReqAnimAll, HSD_JObjSetFlagsAll, Player_80031790,
        Player_GetEntity, Player_GetNametagSlotID, Player_GetPlayerId, gmMainLib_8015D06C,
        gm_80167858, gm_8016895C, gm_80168F88, gm_8016AEEC, gm_8016AEFC, gm_8016B328,
        gm_801A4624, gm_CKindToSelKind, ifTime_HideTimers, HSD_GObj_JObjKind, HSD_PadCopyStatus,
    };
}

/// `struct lbl_80472E48_t` (0x80): `b76:2, b54:2, b32:2, b10:2` PPC bitfields in `bf_0`.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct lbl_80472E48_t {
    pub bf_0: [u8; 1],
    pub pad_1: [u8; 3],
    pub unk_4: i32,
    pub unk_8: i32,
    pub xC: u32,
    pub x10: u8,
    pub pad_11: [u8; 3],
    pub x14: [i32; 0x1B],
}
melee_core::gc_layout!(lbl_80472E48_t, "lbl_80472E48_t", size 0x80, align 4, [
    bf_0 @ 0x0: 1 "u8",
    pad_1 @ 0x1: 3 "char[3]",
    unk_4 @ 0x4: 4 "s32",
    unk_8 @ 0x8: 4 "s32",
    xC @ 0xC: 4 "u32",
    x10 @ 0x10: 1 "u8",
    pad_11 @ 0x11: 3 "char[3]",
    x14 @ 0x14: 108 "s32[27]",
]);
const _: () = assert!(core::mem::size_of::<lbl_80472E48_t>() == 0x80);
impl lbl_80472E48_t {
    pub fn b76(&self) -> u8 { bits8_get(self.bf_0[0], 0, 2) }
    pub fn b54(&self) -> u8 { bits8_get(self.bf_0[0], 2, 2) }
    pub fn b32(&self) -> u8 { bits8_get(self.bf_0[0], 4, 2) }
    pub fn b10(&self) -> u8 { bits8_get(self.bf_0[0], 6, 2) }
    pub fn set_b76(&mut self, v: u8) { bits8_set(&mut self.bf_0[0], 0, 2, v) }
    pub fn set_b54(&mut self, v: u8) { bits8_set(&mut self.bf_0[0], 2, 2, v) }
    pub fn set_b32(&mut self, v: u8) { bits8_set(&mut self.bf_0[0], 4, 2, v) }
    pub fn set_b10(&mut self, v: u8) { bits8_set(&mut self.bf_0[0], 6, 2, v) }
}

pub static mut lbl_80472E48: lbl_80472E48_t = unsafe { core::mem::zeroed() };
pub static mut lbl_80472EC8: [i32; 4] = [0; 4];
pub static mut lbl_804D65C8: *mut HSD_Archive = null_mut();
pub static mut lbl_804D65CC: *mut *mut DynamicModelDesc = null_mut();
pub static mut lbl_804D65D0: *mut *mut DynamicModelDesc = null_mut();
pub static mut lbl_804D65D4: i32 = 0;
pub static mut lbl_804D65D8: i32 = 0;

pub fn gm_80180AE4() -> i32 {
    unsafe { lbl_80472EC8[0].wrapping_mul(0xA) }
}

pub fn gm_80180AF4() -> *mut Fighter_GObj {
    ext::Player_GetEntity(1)
}

pub fn gm_80180B18() {
    unsafe {
        for i in 0..0x1Bi32 {
            let idx = ext::gm_CKindToSelKind(i as u8);
            // clang evaluates the right-hand side (gmMainLib_8015D06C) before the index call
            let v = (*ext::gmMainLib_8015D06C(idx) as u32) / 10;
            let k = ext::gm_CKindToSelKind(i as u8);
            lbl_80472E48.x14[k as usize] = v as i32;
        }
    }
}

pub fn gm_80180BA0() {
    unsafe {
        for i in 0..0x1Bi32 {
            let idx = ext::gm_CKindToSelKind(i as u8);
            let p = ext::gmMainLib_8015D06C(ext::gm_CKindToSelKind(i as u8));
            *p = lbl_80472E48.x14[idx as usize].wrapping_mul(0xA);
        }
    }
}

pub unsafe extern "C" fn fn_80180C14(gobj: *mut HSD_GObj) {
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    if lbl_80472E48.b10() != 0 {
        ext::HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);
        ext::HSD_JObjAnimAll(jobj);
    }
}

/// `HSD_JObjGetNext^n(HSD_JObjGetChild(HSD_JObjGetChild(j)))`
unsafe fn digit_jobj(jobj: *mut HSD_JObj, nexts: u32) -> *mut HSD_JObj {
    let mut j = jobj_child(jobj_child(jobj));
    for _ in 0..nexts {
        j = jobj_next(j);
    }
    j
}

pub unsafe extern "C" fn fn_80180C60(gobj: *mut HSD_GObj) {
    let st = &mut lbl_80472E48;
    let ec8 = &mut lbl_80472EC8;

    let mut dist = (0.1f32 * ext::Ground_801C57F0(0)) as i32;
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    if dist < 0 {
        dist = 0;
    }

    ec8[0] = dist;
    let b76 = st.b76();

    if b76 != 0 && st.b54() != 0 {
        ext::ifTime_HideTimers();
        if ec8[0] == ec8[1] {
            ec8[3] = ec8[3].wrapping_add(1);
        } else {
            ec8[3] = 0;
        }
        if ec8[3] > 0x3C {
            st.set_b32(1);
            if dist == 0 && st.b10() == 0 {
                st.set_b10(1);
            }
        }
    } else {
        if b76 != 0 {
            ext::ifTime_HideTimers();
            if ec8[0] == ec8[1] {
                ec8[3] = ec8[3].wrapping_add(1);
            } else {
                ec8[3] = 0;
            }
            if ec8[3] > 0x78 {
                st.set_b32(1);
                if st.b10() == 0 {
                    st.set_b10(1);
                }
            }
        } else if ext::gm_8016AEEC() == 0 && ext::gm_8016AEFC() == 0x3B {
            st.set_b76(1);
            ext::ifTime_HideTimers();
            ext::Player_80031790(0);
        }
        if ext::Ground_801C1DC0() != 0 {
            if st.b76() == 0 {
                st.set_b76(1);
                ext::ifTime_HideTimers();
                ext::Player_80031790(0);
            }
            if st.b54() == 0 {
                st.set_b54(1);
                ext::Player_80031790(0);
            }
            ec8[3] = 0;
        }
    }

    if st.b32() == 0 {
        ext::HSD_JObjReqAnimAll(jobj, 0.0);
    } else if st.b10() != 0 {
        ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
    } else if dist > st.x14[ext::gm_CKindToSelKind(st.unk_4 as u8) as usize] {
        if lbl_804D65D4 == 0 {
            ext::lbAudioAx_800237A8(0x9C40, 0x7F, 0x40);
            ext::lbAudioAx_800237A8(0x144, 0x7F, 0x40);
            ext::gm_80167858(st.x10 as i8 as i32, ext::Player_GetNametagSlotID(0) as i32, 0xD, 0x5A);
            lbl_804D65D4 = 1;
        }
        ext::HSD_JObjClearFlagsAll(jobj_next(jobj_child(jobj_child(jobj))), JOBJ_HIDDEN as u32);
    }

    let d0 = if ext::lbLang_IsSavedLanguageUS() != 0 {
        ((dist as f32) as f64 / 0.304788) as i32
    } else {
        dist
    };
    let mut disp = d0;
    if d0 > 0x1869F {
        disp = 0x1869F;
    }

    // ones digit
    ext::HSD_JObjReqAnimAll(digit_jobj(jobj, 2), (disp % 10) as f32);
    // tens digit
    ext::HSD_JObjReqAnimAll(digit_jobj(jobj, 3), ((disp / 10) % 10) as f32);
    // hundreds digit
    let d = disp / 100;
    if d != 0 {
        ext::HSD_JObjReqAnimAll(digit_jobj(jobj, 4), (d % 10) as f32);
    } else {
        ext::HSD_JObjReqAnimAll(digit_jobj(jobj, 4), 10.0);
    }
    // thousands digit
    let d = disp / 1000;
    if d != 0 {
        ext::HSD_JObjReqAnimAll(digit_jobj(jobj, 5), (d % 10) as f32);
    } else {
        ext::HSD_JObjReqAnimAll(digit_jobj(jobj, 5), 10.0);
    }
    // ten-thousands digit
    let d = disp / 10000;
    if d != 0 {
        ext::HSD_JObjReqAnimAll(digit_jobj(jobj, 6), (d % 10) as f32);
    } else {
        ext::HSD_JObjReqAnimAll(digit_jobj(jobj, 6), 10.0);
    }

    ext::HSD_JObjAnimAll(jobj);
    ec8[1] = ec8[0];
    if ec8[0] > ec8[2].wrapping_add(0xA) {
        ec8[2] = ec8[0];
        ext::lbAudioAx_80023870(0xBB, 0x7F, 0x40, 0x8A);
    }
}

pub unsafe fn fn_80181598() {
    let st = &mut lbl_80472E48;
    if ext::gm_801A4624() != 0 {
        return;
    }

    let mut mode = st.b10() as u32;

    if mode != 0 {
        if mode == 1 {
            ext::lbAudioAx_800237A8(0xC0, 0x7F, 0x40);
            ext::lbAudioAx_800237A8(0x148, 0x7F, 0x40);
            st.set_b10(2);
        }
        lbl_804D65D8 = lbl_804D65D8.wrapping_add(1);
        if lbl_804D65D8 >= 0xF0
            || (lbl_804D65D8 > 0x3C
                && (ext::HSD_PadCopyStatus[st.x10 as usize].trigger & HSD_PAD_A as u32) != 0)
        {
            ext::gm_8016B328();
            return;
        }
    }

    if st.b32() != 0 && {
        mode = st.b10() as u32;
        mode == 0 || mode == 3
    } {
        st.xC = st.xC.wrapping_add(1);
        if st.xC > 0x3C
            && (st.xC >= 0xF0
                || (ext::HSD_PadCopyStatus[st.x10 as usize].trigger & HSD_PAD_A as u32) != 0)
        {
            let idx = ext::gm_CKindToSelKind(st.unk_4 as u8);
            let val = lbl_80472EC8[0];
            let byte_off = ((idx as u32) << 2) as usize;
            let base = st.x14.as_mut_ptr() as *mut u8;
            if val > *(base.add(byte_off) as *const i32) {
                let k = (ext::gm_CKindToSelKind(st.unk_4 as u8) as usize) << 2;
                *(base.add(k) as *mut i32) = val;
            }
            ext::gm_8016B328();
        }
    }
}
/// `fn_80181598` registered as a proc (the C casts a `void(void)` to `HSD_GObjEvent`).
pub unsafe extern "C" fn fn_80181598_cb(_gobj: *mut HSD_GObj) {
    fn_80181598()
}

pub fn fn_80181708() {
    unsafe {
        lbl_80472EC8 = [0; 4];
        let st = &mut lbl_80472E48;
        st.set_b76(0);
        st.set_b54(0);
        st.set_b32(0);
        st.set_b10(0);
        st.xC = 0;
        st.x10 = ext::Player_GetPlayerId(0) as i8 as u8;
        lbl_804D65D4 = 0;
        lbl_804D65D8 = 0;

        ext::HSD_GObj_SetupProc(ext::GObj_Create(0xE, 0x11, 0), Some(fn_80181598_cb), 0x15);

        let gobj = ext::GObj_Create(0xE, 0xF, 0);
        let jobj = ext::HSD_JObjLoadJoint((**lbl_804D65CC).joint);
        ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_JObjKind, jobj as *mut core::ffi::c_void);
        ext::GObj_SetupGXLink(gobj, Some(ext::HSD_GObj_JObjCallback), 0xB, 0);
        ext::HSD_GObj_SetupProc(gobj, Some(fn_80180C14), 0x15);
        ext::gm_8016895C(jobj, *lbl_804D65CC, 0);
        ext::HSD_JObjReqAnimAll(jobj, 0.0);
        ext::HSD_JObjAnimAll(jobj);
        ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);

        let gobj = ext::GObj_Create(0xE, 0xF, 0);
        let jobj = ext::HSD_JObjLoadJoint((**lbl_804D65D0).joint);
        ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_JObjKind, jobj as *mut core::ffi::c_void);
        ext::GObj_SetupGXLink(gobj, Some(ext::HSD_GObj_JObjCallback), 0xB, 0);
        ext::HSD_GObj_SetupProc(gobj, Some(fn_80180C60), 0x15);
        ext::gm_8016895C(jobj, *lbl_804D65D0, 0);
        ext::HSD_JObjReqAnimAll(jobj, 10.0);
        ext::HSD_JObjAnimAll(jobj);
        ext::HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);

        ext::HSD_JObjSetFlagsAll(jobj_next(jobj_child(jobj_child(jobj))), JOBJ_HIDDEN as u32);
        ext::HSD_JObjClearFlagsAll(jobj_next(jobj_next(jobj_child(jobj_child(jobj)))), JOBJ_HIDDEN as u32);

        ext::gm_80168F88();
    }
}

pub fn gm_80181998() {
    unsafe {
        lbl_804D65C8 = ext::lbArchive_80016DBC(
            c"IfHrNoCn".as_ptr(),
            &[
                &raw mut lbl_804D65CC as *mut core::ffi::c_void,
                c"ScInfCnt_scene_models".as_ptr() as *mut core::ffi::c_void,
            ],
        );
        lbl_804D65C8 = ext::lbArchive_80016DBC(
            c"IfHrReco".as_ptr(),
            &[
                &raw mut lbl_804D65D0 as *mut core::ffi::c_void,
                c"ScInfCnt_scene_models".as_ptr() as *mut core::ffi::c_void,
            ],
        );
    }
    fn_80181708();
}

pub fn gm_80181A00(arg0: i32, arg1: i32) {
    unsafe {
        lbl_80472E48.unk_4 = arg0;
        lbl_80472E48.unk_8 = arg1;
    }
}

#[allow(unused_imports)]
use mi as _;
