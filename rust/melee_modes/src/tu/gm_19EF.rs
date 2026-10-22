//! src/melee/gm/gm_19EF.c: the Game Over screen (continue prompt, coin counter, "Go!") and the
//! "Coming Soon" screen.
//!
//! The file's statics are `pub static mut` so the state oracle (tests/tu_gm_19EF.rs against
//! gen/turef/tu_gm_19EF_ref.c, the C TU compiled natively with shipping contraction) can seed and
//! compare them. The C addresses the ten coin jobj slots as `(HSD_JObj**) ((u8*) &state + 0x28)[i]`
//! and `jobj_slots[i]` (32-bit pointer arithmetic that only holds on the GC layout); here slot `j`
//! (1..=10) is `x28_jobjs[j - 1]` on the native layout, which is what those expressions mean.
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs)]

use crate::tu_util::*;
use core::ffi::{c_char, c_void};
use core::ptr::null_mut;
use melee_types::inline as mi;
use melee_types::*;

/// Callees outside this TU (hooks; unset they panic `TODO: <name> is not ported`).
pub mod ext {
    pub use crate::tu_ext::{
        fn_80168F2C, fn_80168F7C, gm_801604DC, gm_80160474, gm_80160564, gm_80167858, gm_8016895C,
        gm_801A4B60, gm_GetButtonsTriggered, gm_GetCurrentGameMode, lbArchive_80016DBC,
        lbArchive_80016EFC, lbArchive_LoadSymbols, lbAudioAx_800236DC, lbAudioAx_80023F28,
        lbAudioAx_80024030, lbAudioAx_800237A8, lb_80011AC4, lb_80011E24, lb_8000C1C0, lb_8000C290,
        GObj_Create, GObj_SetupGXLink, GObj_SetupGXLinkMax, HSD_CObjLoadDesc, HSD_GObjObject_80390A70,
        HSD_GObj_803910D8, HSD_GObj_CameraKind, HSD_GObj_JObjCallback, HSD_GObj_JObjKind,
        HSD_GObj_LObjCallback, HSD_GObj_LightKind, HSD_GObj_SetupProc, HSD_JObjAnimAll,
        HSD_JObjClearFlags, HSD_JObjLoadJoint, HSD_JObjReqAnimAll, HSD_JObjSetFlags,
        HSD_JObjSetFlagsAll, HSD_MObjAnim, HSD_MObjReqAnim, HSD_PadCopyStatus, HSD_SisLib_803A611C,
        HSD_SisLib_803A62A0, HSD_SisLib_803A6754, HSD_SisLib_803A6B98, HSD_SisLib_803A7664,
        Toy_803060BC, Toy_803102D0, Toy_803124BC,
    };
}

/// `struct unk_jobj_node` (8 bytes on the GC).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct unk_jobj_node {
    pub x0: *mut HSD_JObj,
    pub x4: i32,
}
melee_core::gc_layout!(unk_jobj_node, "unk_jobj_node", size 0x8, align 4, [
    x0 @ 0x0: 4 "HSD_JObj*",
    x4 @ 0x4: 4 "s32",
]);

/// The anonymous 0x78-byte state `lbl_80479A98`. On the GC `jobj_slots[20]` aliases `x4..x54`;
/// `x28` is `{ gobj; jobjs[10] }`.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct gm_19EF_state_t {
    pub x0: u8,
    pub x4: i32,
    pub x8: i32,
    pub xC: i32,
    pub x10: u32,
    pub x14: u8,
    pub x15: u8,
    pub x16: u8,
    pub x18: i32,
    pub x1C: u16,
    pub x1E: u16,
    pub x20: u8,
    pub x22: u16,
    pub x24: *mut HSD_Text,
    pub x28_gobj: *mut HSD_GObj,
    pub x28_jobjs: [*mut HSD_JObj; 10],
    pub x54: *mut HSD_JObj,
    pub x58: *mut HSD_JObj,
    pub x5C: *mut HSD_JObj,
    pub x60: u8,
    pub x64: i32,
    pub x68: i32,
    pub x6C: i32,
    pub x70: i32,
}
melee_core::gc_layout!(gm_19EF_state_t, "lbl_80479A98", size 0x78, align 4, [
    x0 @ 0x0: 1 "u8",
    x4 @ 0x4: 4 "s32",
    x8 @ 0x8: 4 "int",
    xC @ 0xC: 4 "s32",
    x10 @ 0x10: 4 "u32",
    x14 @ 0x14: 1 "u8",
    x15 @ 0x15: 1 "u8",
    x16 @ 0x16: 1 "u8",
    x18 @ 0x18: 4 "int",
    x1C @ 0x1C: 2 "u16",
    x1E @ 0x1E: 2 "u16",
    x20 @ 0x20: 1 "u8",
    x22 @ 0x22: 2 "u16",
    x24 @ 0x24: 4 "HSD_Text*",
    x28_gobj @ 0x28: 4 "HSD_GObj*",
    x28_jobjs @ 0x2C: 40 "HSD_JObj*[10]",
    x54 @ 0x54: 4 "HSD_JObj*",
    x58 @ 0x58: 4 "HSD_JObj*",
    x5C @ 0x5C: 4 "HSD_JObj*",
    x60 @ 0x60: 1 "u8",
    x64 @ 0x64: 4 "s32",
    x68 @ 0x68: 4 "s32",
    x6C @ 0x6C: 4 "s32",
    x70 @ 0x70: 4 "s32",
]);

pub static mut lbl_804D6698: *mut HSD_Archive = null_mut();
pub static mut lbl_804D669C: *mut SceneDesc = null_mut();
pub static mut lbl_804D66A0: *mut SceneDesc = null_mut();
pub static mut lbl_804D66A4: *mut SceneDesc = null_mut();
pub static mut lbl_804D66A8: *mut SceneDesc = null_mut();
pub static mut lbl_804D66AC: *mut HSD_Joint = null_mut();
const N0: unk_jobj_node = unk_jobj_node { x0: null_mut(), x4: 0 };
pub static mut lbl_804D66B0: unk_jobj_node = N0;
pub static mut lbl_804D66B8: unk_jobj_node = N0;
pub static mut lbl_804D66C0: unk_jobj_node = N0;
pub static mut lbl_804D66C8: unk_jobj_node = N0;
pub static mut lbl_804D66D0: unk_jobj_node = N0;
pub static mut lbl_804D66D8: unk_jobj_node = N0;
pub static mut lbl_804D66E0: unk_jobj_node = N0;
pub static mut lbl_804D66E8: unk_jobj_node = N0;
pub static mut lbl_804D66F0: *mut HSD_Archive = null_mut();
pub static mut lbl_804D66F4: *mut HSD_JObj = null_mut();
pub static mut lbl_804D66F8: u32 = 0;
pub static mut lbl_80479A98: gm_19EF_state_t = unsafe { core::mem::zeroed() };

#[inline]
fn sfx_forward() {
    ext::lbAudioAx_80024030(1);
}
#[inline]
fn sfx_move() {
    ext::lbAudioAx_80024030(2);
}

pub unsafe fn fn_8019EFC4(_pad: *mut HSD_PadStatus) {
    let st = &mut lbl_80479A98;
    let mut jobj = (*st.x28_gobj).hsd_obj as *mut HSD_JObj;
    if st.x60 != 0 {
        jobj = jobj_child(jobj);
        jobj = jobj_next(jobj);
        let child_next = jobj;
        if st.x70 != 0 {
            st.x64 += 1;
            if (st.x64 % 5 == 0) && st.x70 != 0 {
                st.x70 -= 1;
            }
            if st.x64 > 0 && st.x64 <= 5 {
                st.x68 = st.x64 + (st.x68 / 5 * 5);
                if st.x68 > 0 && st.x68 <= 5 {
                    st.x6C = st.x68 + (st.x6C / 5 * 5);
                }
            } else if st.x64 == 0x32 {
                st.x64 = 0;
            }
        }
        ext::HSD_JObjReqAnimAll(st.x54, st.x64 as f32);
        ext::HSD_JObjReqAnimAll(st.x58, st.x68 as f32);
        ext::HSD_JObjReqAnimAll(st.x5C, st.x6C as f32);
        {
            let mut i = 10i32;
            while i > 0 {
                let slot = st.x28_jobjs[(i - 1) as usize];
                if i > st.x70 {
                    ext::HSD_JObjSetFlags(slot, JOBJ_HIDDEN as u32);
                } else {
                    ext::HSD_JObjClearFlags(slot, JOBJ_HIDDEN as u32);
                }
                i -= 1;
            }
        }
        ext::HSD_JObjAnimAll(child_next);
    }
}

/// seems to handle "Go!" animations, based on .dat filenames
pub unsafe fn fn_8019F1D0() {
    let mut sp8 = [0u8; 28];
    let st = &mut lbl_80479A98;
    let mut count: u8 = 0;
    let mut value = st.x8 as u32;
    ext::HSD_SisLib_803A7664(st.x24);
    if value != 0 {
        while value != 0 {
            value /= 10;
            count += 1;
        }
        let mut rem = st.x8 as u32;
        sp8[count as usize] = 0;
        while count != 0 {
            sp8[count as usize - 1] = ((rem % 10) + 0x30) as u8;
            rem /= 10;
            count -= 1;
        }
    } else {
        sp8[0] = 0x30;
        sp8[1] = 0;
    }
    ext::HSD_SisLib_803A6B98(st.x24, 240.0, 176.0, sp8.as_ptr() as *const c_char);
}

#[inline]
unsafe fn inline1(mobj: *mut HSD_MObj, frame: f32) {
    ext::HSD_MObjReqAnim(mobj, frame);
    ext::HSD_MObjAnim(mobj);
}
#[inline]
unsafe fn inline2(jobj: *mut HSD_JObj, frame: f32) {
    ext::HSD_JObjReqAnimAll(jobj, frame);
    ext::HSD_JObjAnimAll(jobj);
}
/// `jobj->u.dobj->mobj`
#[inline]
unsafe fn mobj_of(jobj: *mut HSD_JObj) -> *mut HSD_MObj {
    (*(*jobj).u.dobj).mobj
}

pub unsafe fn fn_8019F2D4(arg0: u32) {
    let st = &mut lbl_80479A98;
    if lbl_804D66C8.x4 == 0xA {
        ext::gm_80167858(st.x15 as i8 as i32, st.x16 as i32, 0xD, 0xA);
    }
    if lbl_804D66C8.x4 == 0x19 {
        ext::gm_80167858(st.x15 as i8 as i32, st.x16 as i32, 0xD, 0x14);
    }
    if st.x0 == 0 {
        if lbl_804D66C8.x4 < 0x50 {
            lbl_804D66C8.x4 += 1;
        } else {
            st.x0 = 1;
        }
        lbl_804D66C0.x4 = lbl_804D66C8.x4;
        inline2(lbl_804D66C8.x0, lbl_804D66C8.x4 as f32);
        inline2(lbl_804D66C0.x0, lbl_804D66C0.x4 as f32);
        inline1(mobj_of(lbl_804D66C8.x0), 0.0);
        inline1(mobj_of(lbl_804D66C0.x0), 1.0);
    } else if st.x0 == 1 {
        if st.x14 != 0 {
            inline1(mobj_of(lbl_804D66C8.x0), 0.0);
            inline1(mobj_of(lbl_804D66C0.x0), 1.0);
        } else {
            inline1(mobj_of(lbl_804D66C8.x0), 1.0);
            inline1(mobj_of(lbl_804D66C0.x0), 0.0);
        }
        if arg0 & 0x1100 != 0 {
            if st.x14 != 0 {
                if st.x1E >= st.x1C {
                    st.x0 = 8;
                    sfx_forward();
                    st.x60 = 1;
                    st.x22 = (st.x22 as i32 - (st.x1C as i32 * 0xA)) as u16;
                    st.x1E = (st.x1E as i32 - st.x1C as i32) as u16;
                    lbl_804D66C8.x4 = 0x5A;
                    lbl_804D66C0.x4 = 0x64;
                    return;
                }
                ext::lbAudioAx_80024030(3);
                return;
            }
            st.x0 = 8;
            sfx_forward();
            ext::lbAudioAx_80023F28(0x20);
            ext::lbAudioAx_800237A8(0x9C43, 0x7F, 0x40);
            lbl_804D66C8.x4 = 0x6E;
            lbl_804D66C0.x4 = 0x78;
            return;
        }
        if (arg0 & 0x40000) != 0 && st.x14 == 0 {
            sfx_move();
            st.x14 = 1;
        }
        if (arg0 & 0x80000) != 0 && st.x14 == 1 {
            sfx_move();
            st.x14 = 0;
        }
    } else if st.x0 == 8 {
        inline2(lbl_804D66C0.x0, lbl_804D66C8.x4 as f32);
        inline2(lbl_804D66C8.x0, lbl_804D66C8.x4 as f32);
        if lbl_804D66C8.x4 < lbl_804D66C0.x4 {
            lbl_804D66C8.x4 += 1;
        } else if st.x14 != 0 {
            st.x0 = 9;
            lbl_804D66E0.x4 = 0x3C;
        } else {
            st.x0 = 0xA;
        }
        if st.x14 != 0 {
            inline1(mobj_of(lbl_804D66C0.x0), 2.0);
            return;
        }
        inline1(mobj_of(lbl_804D66C8.x0), 2.0);
    }
}

pub unsafe extern "C" fn fn_8019F6EC(gobj: *mut HSD_GObj) {
    let st = &mut lbl_80479A98;
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;

    ext::HSD_JObjReqAnimAll(jobj, lbl_804D66E0.x4 as f32);
    if st.x0 == 0 {
        if lbl_804D66E0.x4 < 0x32 {
            lbl_804D66E0.x4 += 1;
        } else if st.x20 != 0 {
            ext::lbAudioAx_800237A8(0x9C43, 0x7F, 0x40);
            ext::lbAudioAx_80023F28(0x20);
            st.x0 = 0xA;
        }
    } else if st.x0 == 9 {
        if lbl_804D66E0.x4 < 0x50 {
            lbl_804D66E0.x4 += 1;
        } else if st.x8 as u32 == st.xC as u32 && st.x70 == 0 {
            ext::gm_801A4B60();
            st.x18 = 1;
        }
    }
    ext::HSD_JObjAnimAll(jobj);
    ext::HSD_JObjAnimAll(lbl_804D66B8.x0);
    ext::HSD_JObjAnimAll(lbl_804D66E8.x0);
}
/// `fn_8019F810` registered as a proc (the C casts a `void(void)` to `HSD_GObjEvent`).
pub unsafe extern "C" fn fn_8019F810_cb(_gobj: *mut HSD_GObj) {
    fn_8019F810()
}

pub unsafe fn fn_8019F810() {
    let st = &mut lbl_80479A98;
    let trigger = ext::HSD_PadCopyStatus[st.x15 as usize].trigger;
    fn_8019EFC4(&raw mut ext::HSD_PadCopyStatus[st.x15 as usize]);
    if (st.x0 as u32) < 8 {
        lbl_804D66B0.x4 += 1;
        if lbl_804D66B0.x4 > 0x31F {
            lbl_804D66B0.x4 = 0x190;
        }
        ext::HSD_JObjReqAnimAll(lbl_804D66B0.x0, lbl_804D66B0.x4 as f32);
        ext::HSD_JObjAnimAll(lbl_804D66B0.x0);
    } else {
        ext::HSD_JObjSetFlagsAll(lbl_804D66B0.x0, JOBJ_HIDDEN as u32);
    }
    if st.x0 == 9 {
        if st.x8 as u32 == st.xC as u32 {
            match st.x4 {
                0 | 1 | 2 => st.x18 = 1,
                _ => {}
            }
        } else {
            ext::fn_80168F2C(0);
            // `(x8 - x10) < xC ? xC : x8 - x10` is unsigned (x10 is u32)
            let d = (st.x8 as u32).wrapping_sub(st.x10);
            let xc = st.xC as u32;
            st.x8 = (if d < xc { xc } else { d }) as i32;
        }
    }
    if st.x0 == 0xA {
        ext::HSD_JObjReqAnimAll(lbl_804D66D0.x0, lbl_804D66D0.x4 as f32);
        ext::HSD_JObjAnimAll(lbl_804D66D0.x0);
        if lbl_804D66D0.x4 < 0x2B2 {
            lbl_804D66D0.x4 += 1;
        } else {
            ext::gm_801A4B60();
            st.x18 = 0;
        }
    }
    fn_8019F2D4(trigger);
    fn_8019F1D0();
}

/// `fn_8019F9C4_GetCharIdx`
unsafe fn get_char_idx(arg0: u32) -> i32 {
    let mode = ext::gm_GetCurrentGameMode() as i32;
    if mode == GM_CLASSIC as i32 {
        ext::gm_80160474(arg0 as i32, GM_CLASSIC as i32)
    } else if mode == GM_ADVENTURE as i32 {
        ext::gm_80160474(arg0 as i32, GM_ADVENTURE as i32)
    } else {
        ext::gm_80160474(arg0 as i32, GM_ALLSTAR as i32)
    }
}

/// `fn_8019F9C4_LoadSymbols`
unsafe fn load_symbols(arg0: u32) {
    let game_mode = ext::gm_GetCurrentGameMode();
    let model_name = ext::gm_80160564(arg0 as i32, game_mode as i32);
    let scene_name = ext::gm_801604DC(arg0 as i32, game_mode as i32);

    ext::lbArchive_LoadSymbols(
        scene_name,
        &[&raw mut lbl_804D66AC as *mut c_void, model_name as *mut c_void],
    );
    ext::lbArchive_LoadSymbols(
        c"GmGoAnim.dat".as_ptr(),
        &[&raw mut lbl_804D66A4 as *mut c_void, c"ScGamRegGover_scene_data".as_ptr() as *mut c_void],
    );
    ext::lbArchive_LoadSymbols(
        c"GmRgStnd.dat".as_ptr(),
        &[&raw mut lbl_804D66A8 as *mut c_void, c"standScene".as_ptr() as *mut c_void],
    );
}

/// `fn_8019F9C4_inline1`: x54 / x58 / x5C are the child, its next and that one's next.
unsafe fn inline_digits(next: *mut HSD_JObj, child: &mut *mut HSD_JObj) {
    let st = &mut lbl_80479A98;
    *child = jobj_child(next);
    st.x54 = *child;

    *child = jobj_next(st.x54);
    st.x58 = *child;

    *child = jobj_next(st.x58);
}

/// `fn_8019F9C4_inline2`
unsafe fn inline_attach(next_jobj: *mut HSD_JObj, model: *mut HSD_JObj) {
    let node = jobj_child(next_jobj);
    ext::lb_8000C290(model, node);
}

/// `fn_8019F9C4_inline3`
unsafe fn inline_character(model: *mut HSD_JObj, object: *mut HSD_GObj, arg0: u32) {
    lbl_804D66B8.x0 = model;
    ext::HSD_GObjObject_80390A70(object, ext::HSD_GObj_JObjKind, model as *mut c_void);
    ext::GObj_SetupGXLink(object, Some(ext::HSD_GObj_JObjCallback), 0xB, 0);
    let char_idx = get_char_idx(arg0);

    let child = jobj_child(model);

    mi::HSD_JObjSetTranslateX(&mut *child, ext::Toy_803060BC(char_idx, 0), &mut Dirty);
    mi::HSD_JObjSetTranslateY(&mut *child, ext::Toy_803060BC(char_idx, 1), &mut Dirty);
    mi::HSD_JObjSetTranslateZ(&mut *child, ext::Toy_803060BC(char_idx, 2), &mut Dirty);
    mi::HSD_JObjSetRotationY(&mut *child, 0.017453292f32 * ext::Toy_803060BC(char_idx, 5), &mut Dirty);
    let f = ext::Toy_803060BC(char_idx, 4);
    let scale = ext::Toy_803060BC(char_idx, 3) / f;
    mi::HSD_JObjSetScaleX(&mut *child, scale, &mut Dirty);
    mi::HSD_JObjSetScaleY(&mut *child, scale, &mut Dirty);
    mi::HSD_JObjSetScaleZ(&mut *child, scale, &mut Dirty);
    mi::HSD_JObjSetScaleX(&mut *model, 2.0, &mut Dirty);
    mi::HSD_JObjSetScaleY(&mut *model, 2.0, &mut Dirty);
    mi::HSD_JObjSetScaleZ(&mut *model, 2.0, &mut Dirty);
}

pub unsafe fn fn_8019F9C4(mut arg0: u32) {
    let st = &mut lbl_80479A98;

    let char_idx = get_char_idx(arg0);
    arg0 = if char_idx == -1 { 8 } else { arg0 };
    ext::fn_80168F7C();
    lbl_804D6698 = ext::lbArchive_80016DBC(
        c"GmGover.dat".as_ptr(),
        &[
            &raw mut lbl_804D669C as *mut c_void,
            c"ScGamRegGover_scene_data".as_ptr() as *mut c_void,
        ],
    );
    ext::lbArchive_80016DBC(
        c"GmGoCoin.dat".as_ptr(),
        &[
            &raw mut lbl_804D66A0 as *mut c_void,
            c"ScGamRegGover_scene_data".as_ptr() as *mut c_void,
        ],
    );
    ext::Toy_803124BC();
    ext::Toy_803102D0();
    load_symbols(arg0);
    let cobj = ext::HSD_CObjLoadDesc((*(*lbl_804D669C).cameras).desc);
    let cam_gobj = ext::GObj_Create(0x13, 0x14, 0);
    ext::HSD_GObjObject_80390A70(cam_gobj, ext::HSD_GObj_CameraKind, cobj as *mut c_void);
    ext::GObj_SetupGXLinkMax(cam_gobj, Some(ext::HSD_GObj_803910D8), 0);
    (*cam_gobj).gxlink_prios = 0x801;

    let mut gobj = ext::GObj_Create(0xB, 0xF, 0);
    {
        let lobj = ext::lb_80011AC4((*lbl_804D669C).lights);
        ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_LightKind as u8, lobj as *mut c_void);
    }
    ext::GObj_SetupGXLink(gobj, Some(ext::HSD_GObj_LObjCallback), 0, 0);

    gobj = ext::GObj_Create(0xE, 0xF, 0);
    let mut jobj = ext::HSD_JObjLoadJoint((**(*lbl_804D669C).models).joint);
    ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_JObjKind, jobj as *mut c_void);
    ext::GObj_SetupGXLink(gobj, Some(ext::HSD_GObj_JObjCallback), 0xB, 0);
    ext::HSD_GObj_SetupProc(gobj, Some(fn_8019F810_cb), 0);
    ext::gm_8016895C(jobj, *(*lbl_804D669C).models, 0);
    ext::HSD_JObjReqAnimAll(jobj, 0.0);
    ext::HSD_JObjAnimAll(jobj);
    st.x14 = 1;
    st.x0 = 0;
    lbl_804D66B0.x4 = 0;
    lbl_804D66C0.x4 = 0;
    lbl_804D66C8.x4 = 0;
    lbl_804D66E0.x4 = 0;
    lbl_804D66D0.x4 = 0x24E;
    ext::lb_80011E24(jobj, &mut lbl_804D66B0.x0, 0x38, -1);
    ext::lb_80011E24(jobj, &mut lbl_804D66D8.x0, 1, -1);
    ext::lb_80011E24(jobj, &mut lbl_804D66D0.x0, 2, -1);
    ext::lb_80011E24(jobj, &mut lbl_804D66C8.x0, 0x35, -1);
    ext::lb_80011E24(jobj, &mut lbl_804D66C0.x0, 0x37, -1);
    if st.x20 != 0 {
        ext::HSD_JObjSetFlagsAll(lbl_804D66B0.x0, JOBJ_HIDDEN as u32);
        ext::HSD_JObjSetFlagsAll(lbl_804D66C8.x0, JOBJ_HIDDEN as u32);
        ext::HSD_JObjSetFlagsAll(lbl_804D66C0.x0, JOBJ_HIDDEN as u32);
    }

    gobj = ext::GObj_Create(0xE, 0xF, 0);
    jobj = ext::HSD_JObjLoadJoint((**(*lbl_804D66A0).models).joint);
    ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_JObjKind, jobj as *mut c_void);
    ext::GObj_SetupGXLink(gobj, Some(ext::HSD_GObj_JObjCallback), 0xB, 0);
    ext::gm_8016895C(jobj, *(*lbl_804D66A0).models, 0);
    st.x28_gobj = gobj;

    let mut child = jobj_child(jobj);
    let mut node = if child.is_null() { null_mut() } else { (*child).child };

    for i in 10..20usize {
        st.x28_jobjs[i - 10] = node;
        if !(*node).next.is_null() {
            node = (*node).next;
        }
    }

    st.x70 = st.x1C as i32;

    child = jobj_child(jobj);
    let next = jobj_next(child);

    inline_digits(next, &mut child);
    st.x5C = child;

    {
        let val = st.x1E as i32;
        st.x6C = ((0xA - ((val / 100) % 10)) * 5) % 50;
        st.x68 = ((0xA - ((val / 10) % 10)) * 5) % 50;
        st.x64 = ((0xA - (val % 10)) * 5) % 50;
    }
    ext::HSD_JObjReqAnimAll(st.x5C, st.x6C as f32);
    ext::HSD_JObjReqAnimAll(st.x58, st.x68 as f32);
    ext::HSD_JObjReqAnimAll(st.x54, st.x64 as f32);
    {
        let mut j = 10i32;
        while j > 0 {
            let p = st.x28_jobjs[(j - 1) as usize];
            if j > st.x70 {
                ext::HSD_JObjSetFlags(p, JOBJ_HIDDEN as u32);
            } else {
                ext::HSD_JObjClearFlags(p, JOBJ_HIDDEN as u32);
            }
            j -= 1;
        }
    }
    ext::HSD_JObjAnimAll(jobj);

    gobj = ext::GObj_Create(0xE, 0xF, 0);
    jobj = ext::HSD_JObjLoadJoint((**(*lbl_804D66A4).models).joint);
    lbl_804D66E0.x0 = jobj;
    ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_JObjKind, jobj as *mut c_void);
    ext::GObj_SetupGXLink(gobj, Some(ext::HSD_GObj_JObjCallback), 0xB, 0);
    ext::HSD_GObj_SetupProc(gobj, Some(fn_8019F6EC), 0);
    ext::gm_8016895C(jobj, *(*lbl_804D66A4).models, 0);
    ext::HSD_JObjReqAnimAll(jobj, 0.0);
    ext::HSD_JObjAnimAll(jobj);

    gobj = ext::GObj_Create(0xE, 0xF, 0);
    jobj = ext::HSD_JObjLoadJoint((**(*lbl_804D66A8).models).joint);
    lbl_804D66E8.x0 = jobj;
    ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_JObjKind, jobj as *mut c_void);
    ext::GObj_SetupGXLink(gobj, Some(ext::HSD_GObj_JObjCallback), 0xB, 0);
    mi::HSD_JObjSetScaleX(&mut *jobj, 2.0, &mut Dirty);
    mi::HSD_JObjSetScaleY(&mut *jobj, 2.0, &mut Dirty);
    mi::HSD_JObjSetScaleZ(&mut *jobj, 2.0, &mut Dirty);

    child = jobj_child(lbl_804D66E0.x0);
    let mut next = jobj_child(child);
    node = jobj_child(next);
    ext::lb_8000C1C0(jobj, node);

    child = jobj_child(lbl_804D66E0.x0);
    next = jobj_child(child);
    inline_attach(next, jobj);
    let character_gobj = ext::GObj_Create(0xE, 0xF, 0);
    gobj = character_gobj;
    jobj = ext::HSD_JObjLoadJoint(lbl_804D66AC);
    inline_character(jobj, gobj, arg0);

    ext::lb_8000C1C0(jobj, lbl_804D66E8.x0);
    ext::lb_8000C290(jobj, lbl_804D66E8.x0);

    ext::HSD_SisLib_803A611C(0, cam_gobj, 9, 0x12, 0, 0xB, 0, 0x13);
    ext::HSD_SisLib_803A62A0(0, c"SdIntro.dat".as_ptr() as *mut c_char, c"SIS_IntroData".as_ptr() as *mut c_char);
    st.x24 = ext::HSD_SisLib_803A6754(0, 0);
    {
        let text = st.x24;
        (*text).font_size.x = 0.1;
        (*text).font_size.y = 0.1;
    }
    (*st.x24).default_alignment = 2;
    ext::HSD_SisLib_803A6B98(st.x24, 240.0, 176.0, c" ".as_ptr());
    if st.x20 == 0 {
        ext::lbAudioAx_800237A8(0x9C42, 0x7F, 0x40);
        ext::lbAudioAx_80023F28(5);
    }
}

pub unsafe fn gm_Scene_GOver_OnEnter(arg0_: *mut c_void) {
    let arg0 = &*(arg0_ as *const DebugGameOverData);
    let st = &mut lbl_80479A98;

    st.x60 = 0;
    st.x22 = arg0.x18;
    st.x1E = (arg0.x18 as i32 / 10) as u16;
    if st.x1E as u32 > 0x3E7 {
        st.x1E = 0x3E7;
    }
    st.x1C = arg0.x16;
    if st.x1E >= st.x1C {
        st.x20 = 0;
    } else {
        st.x20 = 1;
    }
    st.x16 = arg0.x15;
    st.x4 = arg0.x8 as i32;
    st.x8 = arg0.x0 as i32;
    st.xC = (arg0.x0 as i32) / 2;
    st.x10 = ((st.x8 - st.xC) as u32) / 120;
    if st.x10 == 0 {
        st.x10 = 1;
    }
    st.x15 = arg0.slot;
    fn_8019F9C4(arg0.ckind);
}

pub unsafe fn gm_Scene_GOver_OnExit(arg0_: *mut c_void) {
    let arg0 = &mut *(arg0_ as *mut DebugGameOverData);
    let st = &lbl_80479A98;
    arg0.x4 = st.x8;
    arg0.xC = st.x18;
    arg0.x18 = st.x22;
    ext::lbAudioAx_800236DC();
    ext::lbArchive_80016EFC(lbl_804D6698);
}

pub unsafe extern "C" fn fn_801A0B60(gobj: *mut HSD_GObj) {
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    ext::HSD_JObjReqAnimAll(jobj, lbl_804D66F8 as f32);
    ext::HSD_JObjAnimAll(jobj);
    if lbl_804D66F8 == 0xBD {
        if (ext::gm_GetButtonsTriggered(PAD_MAX_CONTROLLERS as u8) as u32 & PAD_BUTTON_START) != 0
            || (ext::gm_GetButtonsTriggered(PAD_MAX_CONTROLLERS as u8) as u32 & PAD_BUTTON_A) != 0
        {
            lbl_804D66F8 += 1;
        }
    } else {
        lbl_804D66F8 += 1;
    }
    let aobj = (*lbl_804D66F4).aobj;
    assert!(!aobj.is_null(), "aobj.h:0x92");
    let cur = (*aobj).curr_frame;
    assert!(!aobj.is_null(), "aobj.h:0xAA");
    let end = (*aobj).end_frame;
    if (cur as i32) >= (end as i32) {
        ext::gm_801A4B60();
    }
}

pub unsafe fn gm_Scene_ComingSoon_OnEnter(_unused: *mut c_void) {
    let mut sp10: *mut SceneDesc = null_mut();

    lbl_804D66F8 = 0;
    lbl_804D66F0 = ext::lbArchive_80016DBC(
        c"IfComSn.dat".as_ptr(),
        &[
            &raw mut sp10 as *mut c_void,
            c"ScComSoon_scene_data".as_ptr() as *mut c_void,
        ],
    );

    let temp_r30 = ext::GObj_Create(0x13, 0x14, 0);
    let cobj = ext::HSD_CObjLoadDesc((*(*sp10).cameras).desc);
    ext::HSD_GObjObject_80390A70(temp_r30, ext::HSD_GObj_CameraKind, cobj as *mut c_void);
    ext::GObj_SetupGXLinkMax(temp_r30, Some(ext::HSD_GObj_803910D8), 0);
    (*temp_r30).gxlink_prios = 0x801;

    let temp_r30_2 = ext::GObj_Create(0xB, 0xF, 0);
    let lobj = ext::lb_80011AC4((*sp10).lights);
    ext::HSD_GObjObject_80390A70(temp_r30_2, ext::HSD_GObj_LightKind as u8, lobj as *mut c_void);
    ext::GObj_SetupGXLink(temp_r30_2, Some(ext::HSD_GObj_LObjCallback), 0, 0);
    let temp_r30_3 = ext::GObj_Create(0xE, 0xF, 0);
    let temp_r3 = ext::HSD_JObjLoadJoint((**(*sp10).models).joint);
    ext::HSD_GObjObject_80390A70(temp_r30_3, ext::HSD_GObj_JObjKind, temp_r3 as *mut c_void);
    ext::GObj_SetupGXLink(temp_r30_3, Some(ext::HSD_GObj_JObjCallback), 0xB, 0);
    ext::HSD_GObj_SetupProc(temp_r30_3, Some(fn_801A0B60), 0);
    ext::gm_8016895C(temp_r3, *(*sp10).models, 0);
    ext::HSD_JObjReqAnimAll(temp_r3, 0.0);
    ext::HSD_JObjAnimAll(temp_r3);
    ext::lb_80011E24(temp_r3, &raw mut lbl_804D66F4, 1, -1);
}

pub unsafe fn gm_Scene_ComingSoon_OnExit(_unused: *mut c_void) {
    ext::lbArchive_80016EFC(lbl_804D66F0);
    ext::lbAudioAx_800236DC();
}
