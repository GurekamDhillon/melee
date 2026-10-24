//! src/melee/gm/gm_1ADD.c: the notice window ("NtMsgWin") used by the memory-card scene: a linked list of
//! message boxes (`gm_803DD4D0`), each with a list of buttons, built from the archive's joints and SIS text.
//! All 12 functions (and the `get_master_status` / `get_copy_status` inlines) are checked against the C by a state oracle (tests/tu_gm_1ADD.rs over
//! gen/turef/tu_gm_1ADD_ref.c): the game's gm_1ADD.c compiled natively (shipping contraction) with the engine
//! calls stubbed and logged; the list, the objects the functions touch and the callee log are compared.
//! The four `fmuladd` sites of `gm_801AE050` (`2.2F * x - 1.0F`, `2.232143F * -y - 1.3F`) are fused as
//! `fmadds`, as the shipping build does (tools/rust-port/fma_sites.py).
#![allow(non_snake_case, non_upper_case_globals, dead_code, static_mut_refs, unused_unsafe)]

use core::ffi::c_void;
use melee_core::fp::fmadds;
use melee_types::*;

pub const JOBJ_HIDDEN_FLAG: u32 = JOBJ_HIDDEN as u32;

/// `struct unkd4d0_8` (a button of a message box), GameCube size 0x28.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct unkd4d0_8 {
    pub next: *mut unkd4d0_8,
    pub unk4: *mut unkd4d0_8,
    pub unk8: i32,
    pub unkC: i32,
    pub unk10: f32,
    pub unk14: f32,
    pub pad18: [u8; 8],
    pub unk20: *mut HSD_GObj,
    pub unk24: *mut HSD_Text,
}
melee_core::gc_layout!(unkd4d0_8, "unkd4d0_8", size 0x28, align 4, [
    next @ 0x0: 4 "struct unkd4d0_8*",
    unk4 @ 0x4: 4 "struct unkd4d0_8*",
    unk8 @ 0x8: 4 "int",
    unkC @ 0xC: 4 "int",
    unk10 @ 0x10: 4 "float",
    unk14 @ 0x14: 4 "float",
    pad18 @ 0x18: 8 "u8[8]",
    unk20 @ 0x20: 4 "HSD_GObj*",
    unk24 @ 0x24: 4 "HSD_Text*",
]);

/// `static struct unkd4d0` (a message box), GameCube size 0x24.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct unkd4d0 {
    pub next: *mut unkd4d0,
    pub unk4: *mut unkd4d0,
    pub unk8: *mut unkd4d0_8,
    pub unkC: i32,
    pub unk10: i32,
    pub unk14: f32,
    pub unk18: f32,
    pub unk1C: *mut HSD_GObj,
    pub unk20: *mut HSD_Text,
}
melee_core::gc_layout!(unkd4d0, "unkd4d0", size 0x24, align 4, [
    next @ 0x0: 4 "struct unkd4d0*",
    unk4 @ 0x4: 4 "struct unkd4d0*",
    unk8 @ 0x8: 4 "struct unkd4d0_8*",
    unkC @ 0xC: 4 "int",
    unk10 @ 0x10: 4 "int",
    unk14 @ 0x14: 4 "float",
    unk18 @ 0x18: 4 "float",
    unk1C @ 0x1C: 4 "HSD_GObj*",
    unk20 @ 0x20: 4 "HSD_Text*",
]);

/// `static struct unkd4d0 gm_803DD4D0`: the list head (its `next` is the first box).
pub static mut gm_803DD4D0: unkd4d0 = unkd4d0 {
    next: core::ptr::null_mut(),
    unk4: core::ptr::null_mut(),
    unk8: core::ptr::null_mut(),
    unkC: 0,
    unk10: 0,
    unk14: 0.0,
    unk18: 0.0,
    unk1C: core::ptr::null_mut(),
    unk20: core::ptr::null_mut(),
};
/// `static SceneDesc* gm_804D6868;` (the loaded NtMsgWin scene data).
pub static mut gm_804D6868: *mut SceneDesc = core::ptr::null_mut();
/// `static int gm_804D686C;` (the SIS text group).
pub static mut gm_804D686C: i32 = 0;

/// Calls into TUs that are not ported yet. Each is a settable hook; an unset hook panics.
pub mod ext {
    use super::*;
    macro_rules! hook {
        ($hook:ident, $name:ident, ($($a:ident: $t:ty),*) $(-> $r:ty)?) => {
            pub static mut $hook: Option<fn($($t),*) $(-> $r)?> = None;
            #[allow(non_snake_case)]
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $hook } {
                    Some(f) => f($($a),*),
                    None => panic!(concat!("TODO: ", stringify!($name), " is not ported")),
                }
            }
        };
    }
    // sysdolphin/baselib/memory.c
    pub static mut HSD_MEMALLOC: Option<fn(isize) -> *mut c_void> = None;
            #[allow(non_snake_case)]
            pub fn HSD_MemAlloc(size: isize) -> *mut c_void {
                match unsafe { HSD_MEMALLOC } {
                    Some(f) => f(size),
                    None => { return unsafe { melee_runtime::tu::memory::HSD_MemAlloc(size as _) as _ }; },
                }
            }
    pub static mut HSD_FREE: Option<fn(*mut c_void) > = None;
            #[allow(non_snake_case)]
            pub fn HSD_Free(ptr: *mut c_void)  {
                match unsafe { HSD_FREE } {
                    Some(f) => f(ptr),
                    None => { unsafe { let _ = melee_runtime::tu::memory::HSD_Free(ptr as _); } return; },
                }
            }
    // sysdolphin/baselib/sislib.c
    pub static mut HSD_SISLIB_803A5ACC: Option<fn(i32, i32, f32, f32, f32, f32, f32) -> *mut HSD_Text> = None;
            #[allow(non_snake_case)]
            pub fn HSD_SisLib_803A5ACC(a: i32, b: i32, x: f32, y: f32, z: f32, w: f32, h: f32) -> *mut HSD_Text {
                match unsafe { HSD_SISLIB_803A5ACC } {
                    Some(f) => f(a, b, x, y, z, w, h),
                    None => { return unsafe { melee_runtime::tu::sislib::HSD_SisLib_803A5ACC(a as _, b as _, x as _, y as _, z as _, w as _, h as _) as _ }; },
                }
            }
    pub static mut HSD_SISLIB_803A6368: Option<fn(*mut HSD_Text, i32) > = None;
            #[allow(non_snake_case)]
            pub fn HSD_SisLib_803A6368(text: *mut HSD_Text, v: i32)  {
                match unsafe { HSD_SISLIB_803A6368 } {
                    Some(f) => f(text, v),
                    None => { unsafe { let _ = melee_runtime::tu::sislib::HSD_SisLib_803A6368(text as _, v as _); } return; },
                }
            }
    hook!(HSD_SISLIB_803A62A0, HSD_SisLib_803A62A0, (a: i32, file: &str, sym: &str));
    hook!(HSD_SISLIB_803A611C, HSD_SisLib_803A611C, (a: i32, g: *mut HSD_GObj, b: u16, c: u8, d: u8, e: u8, f: u8, h: u32) -> i32);
    hook!(HSD_SISLIB_803A5CC4, HSD_SisLib_803A5CC4, (text: *mut HSD_Text));
    // sysdolphin/baselib/jobj.c, cobj.c, gobj*.c
    hook!(HSD_JOBJSETMTXDIRTYSUB, HSD_JObjSetMtxDirtySub, (j: *mut HSD_JObj));
    hook!(HSD_JOBJSETFLAGSALL, HSD_JObjSetFlagsAll, (j: *mut HSD_JObj, f: u32));
    pub static mut HSD_JOBJLOADJOINT: Option<fn(*mut HSD_Joint) -> *mut HSD_JObj> = None;
            #[allow(non_snake_case)]
            pub fn HSD_JObjLoadJoint(joint: *mut HSD_Joint) -> *mut HSD_JObj {
                match unsafe { HSD_JOBJLOADJOINT } {
                    Some(f) => f(joint),
                    None => { return unsafe { melee_runtime::tu::jobj::HSD_JObjLoadJoint(joint as _) as _ }; },
                }
            }
    hook!(HSD_JOBJREQANIMALL, HSD_JObjReqAnimAll, (jobj: *mut HSD_JObj, frame: f32));
    hook!(HSD_JOBJANIMALL, HSD_JObjAnimAll, (jobj: *mut HSD_JObj));
    pub static mut HSD_COBJLOADDESC: Option<fn(*mut HSD_CObjDesc) -> *mut HSD_CObj> = None;
            #[allow(non_snake_case)]
            pub fn HSD_CObjLoadDesc(desc: *mut HSD_CObjDesc) -> *mut HSD_CObj {
                match unsafe { HSD_COBJLOADDESC } {
                    Some(f) => f(desc),
                    None => { return unsafe { melee_runtime::tu::cobj::HSD_CObjLoadDesc(desc as _) as _ }; },
                }
            }
    hook!(GOBJ_CREATE, GObj_Create, (classifier: u16, p_link: u8, priority: u8) -> *mut HSD_GObj);
    hook!(HSD_GOBJOBJECT_80390A70, HSD_GObjObject_80390A70, (gobj: *mut HSD_GObj, kind: u8, obj: *mut c_void));
    hook!(GOBJ_SETUPGXLINK, GObj_SetupGXLink, (gobj: *mut HSD_GObj, render: usize, gx_link: u8, priority: u32));
    hook!(GOBJ_SETUPGXLINKMAX, GObj_SetupGXLinkMax, (gobj: *mut HSD_GObj, render: usize, priority: u32));
    hook!(HSD_GOBJ_SETUPPROC, HSD_GObj_SetupProc, (gobj: *mut HSD_GObj, proc: usize, prio: u8));
    hook!(HSD_GOBJFREE, HSD_GObjFree, (gobj: *mut HSD_GObj));
    /// `HSD_GObj_JObjKind` / `HSD_GObj_CameraKind` (sysdolphin/baselib/gobj.c).
    hook!(HSD_GOBJ_JOBJKIND, hsd_gobj_jobj_kind, () -> u8);
    hook!(HSD_GOBJ_CAMERAKIND, hsd_gobj_camera_kind, () -> u8);
    // lb/lbarchive.c, lb/lblanguage.c, gm/gm_1601.c
    hook!(LBARCHIVE_80016DBC, lbArchive_80016DBC, (filename: &str, symbols: *mut *mut SceneDesc, sym: &str, end: i32));
    pub static mut LBLANG_ISSAVEDLANGUAGEUS: Option<fn() -> bool> = None;
            #[allow(non_snake_case)]
            pub fn lbLang_IsSavedLanguageUS() -> bool {
                match unsafe { LBLANG_ISSAVEDLANGUAGEUS } {
                    Some(f) => f(),
                    None => { return unsafe { melee_runtime::tu::lblanguage::lbLang_IsSavedLanguageUS() as _ }; },
                }
            }
    pub static mut GM_8016895C: Option<fn(*mut HSD_JObj, *mut DynamicModelDesc, i32) > = None;
            #[allow(non_snake_case)]
            pub fn gm_8016895C(jobj: *mut HSD_JObj, mdl: *mut DynamicModelDesc, idx: i32)  {
                match unsafe { GM_8016895C } {
                    Some(f) => f(jobj, mdl, idx),
                    None => { unsafe { let _ = crate::tu::gm_1601::gm_8016895C(jobj as _, mdl as _, idx as _); } return; },
                }
            }
    /// `HSD_PadMasterStatus[idx].err` and `HSD_PadCopyStatus[idx].trigger` (sysdolphin/baselib/controller.c).
    hook!(HSD_PADMASTERSTATUS_ERR, hsd_pad_master_status_err, (idx: usize) -> i8);
    hook!(HSD_PADCOPYSTATUS_TRIGGER, hsd_pad_copy_status_trigger, (idx: usize) -> u32);

    /// `HSD_GObj_803910D8` / `HSD_GObj_JObjCallback` (sysdolphin/baselib/gobj.c): GX-link render callbacks; address only.
    pub unsafe extern "C" fn HSD_GObj_803910D8(_gobj: *mut HSD_GObj, _code: i32) {
        panic!("TODO(sysdolphin): HSD_GObj_803910D8 is not ported");
    }
    pub unsafe extern "C" fn HSD_GObj_JObjCallback(_gobj: *mut HSD_GObj, _code: i32) {
        { unsafe { let _ = melee_runtime::tu::gobj::HSD_GObj_JObjCallback(_gobj as _, _code as _); } return; };
    }
}

/// `memzero(mem, size)` (lb/lb_00B0.c, ported in `lb_r6`).
unsafe fn memzero(mem: *mut u8, size: usize) {
    crate::lb_r6::memzero(core::slice::from_raw_parts_mut(mem, size), size as isize);
}

/// `GET_JOBJ(gobj)` (gobj.h): `gobj->hsd_obj`.
#[inline(always)]
unsafe fn GET_JOBJ(gobj: *mut HSD_GObj) -> *mut HSD_JObj {
    (*gobj).hsd_obj as *mut HSD_JObj
}

/// `HSD_JObjSetMtxDirty(jobj)` (jobj.h macro).
#[inline(always)]
unsafe fn HSD_JObjSetMtxDirty(jobj: *mut HSD_JObj) {
    if !jobj.is_null() {
        let flags = (*jobj).flags;
        let is_dirty = (flags & (JOBJ_USER_DEF_MTX as u32)) == 0 && (flags & (JOBJ_MTX_DIRTY as u32)) != 0;
        if !is_dirty {
            ext::HSD_JObjSetMtxDirtySub(jobj);
        }
    }
}

/// `HSD_JObjSetTranslateX(jobj, x)` (jobj.h, static inline).
#[inline(always)]
unsafe fn HSD_JObjSetTranslateX(jobj: *mut HSD_JObj, x: f32) {
    (*jobj).translate.x = x;
    if (*jobj).flags & (JOBJ_MTX_INDEP_SRT as u32) == 0 {
        HSD_JObjSetMtxDirty(jobj);
    }
}

/// `HSD_JObjSetTranslateY(jobj, y)` (jobj.h, static inline).
#[inline(always)]
unsafe fn HSD_JObjSetTranslateY(jobj: *mut HSD_JObj, y: f32) {
    (*jobj).translate.y = y;
    if (*jobj).flags & (JOBJ_MTX_INDEP_SRT as u32) == 0 {
        HSD_JObjSetMtxDirty(jobj);
    }
}

/// `HSD_JObjAddTranslationY(jobj, y)` (jobj.h, static inline).
#[inline(always)]
unsafe fn HSD_JObjAddTranslationY(jobj: *mut HSD_JObj, y: f32) {
    (*jobj).translate.y += y;
    if (*jobj).flags & (JOBJ_MTX_INDEP_SRT as u32) == 0 {
        HSD_JObjSetMtxDirty(jobj);
    }
}

/// `gm_801ADDD8()`: clear the list head, mark it (`unkC = -0x3E7`) and build the scene.
pub unsafe fn gm_801ADDD8() {
    memzero(&raw mut gm_803DD4D0 as *mut u8, core::mem::size_of::<unkd4d0>());
    gm_803DD4D0.unkC = -0x3E7;
    gm_801AEBB0();
}

/// `gm_801ADE1C(arg0, arg1, arg2, arg3)`: append a message box with text id `arg1` at (`arg2`, `arg3`); always true.
pub unsafe fn gm_801ADE1C(arg0: i32, arg1: i32, arg2: f32, arg3: f32) -> bool {
    let mut cur: *mut unkd4d0 = &raw mut gm_803DD4D0;
    let next = ext::HSD_MemAlloc(core::mem::size_of::<unkd4d0>() as isize) as *mut unkd4d0;
    memzero(next as *mut u8, core::mem::size_of::<unkd4d0>());
    (*next).unkC = arg0;
    (*next).unk10 = arg1;
    (*next).unk14 = arg2;
    (*next).unk18 = arg3;

    loop {
        if !(*cur).next.is_null() {
            cur = (*cur).next;
            continue;
        }

        (*cur).next = next;
        (*next).unk4 = cur;
        (*next).next = core::ptr::null_mut();
        (*next).unk8 = core::ptr::null_mut();
        (*next).unk1C = gm_801AECC4(1);
        if !(*next).unk1C.is_null() {
            HSD_JObjSetTranslateX(GET_JOBJ((*next).unk1C), (*next).unk14);
        }
        if !(*next).unk1C.is_null() {
            HSD_JObjSetTranslateY(GET_JOBJ((*next).unk1C), (*next).unk18);
        }
        (*next).unk20 =
            ext::HSD_SisLib_803A5ACC(3, gm_804D686C, -14.0f32 + (*next).unk14, -9.0f32 - (*next).unk18, 0.0, 30.8, 4.6);
        let text = (*next).unk20;
        (*text).x34.x = 0.079;
        (*text).x34.y = 0.079;
        (*(*next).unk20).default_alignment = 0;
        (*(*next).unk20).default_fitting = 0;
        ext::HSD_SisLib_803A6368((*next).unk20, (*next).unk10);
        break;
    }

    true
}

/// The shared tail of `gm_801AE050`: build the button's joint and text. (The C repeats it in both branches;
/// the first-button branch re-reads `unk20` into a temporary, which has no effect on the result.)
unsafe fn button_setup(temp: *mut unkd4d0_8) {
    (*temp).unk20 = gm_801AECC4(0);
    if !(*temp).unk20.is_null() {
        HSD_JObjSetTranslateX(GET_JOBJ((*temp).unk20), (*temp).unk10);
    }
    if !(*temp).unk20.is_null() {
        HSD_JObjSetTranslateY(GET_JOBJ((*temp).unk20), (*temp).unk14);
    }
    // `2.2F * x - 1.0F` and `2.232143F * -y - 1.3F` are fmuladd sites (fused as fmadds).
    (*temp).unk24 = ext::HSD_SisLib_803A5ACC(
        3,
        gm_804D686C,
        fmadds(2.2f32, (*temp).unk10, -1.0f32),
        fmadds(2.232143f32, -(*temp).unk14, -1.3f32),
        0.0,
        2.0,
        2.0,
    );
    let t = (*temp).unk24;
    (*t).x34.x = 0.1;
    (*t).x34.y = 0.1;
    (*(*temp).unk24).default_alignment = 1;
    (*(*temp).unk24).default_fitting = 0;
    ext::HSD_SisLib_803A6368((*temp).unk24, (*temp).unkC);
}

/// `gm_801AE050(arg0, arg1, arg2, arg3, arg4)`: append button `arg1` (text id `arg2`, at `arg3`/`arg4`) to
/// every box whose id is `arg0`; always true. Note the C keeps scanning boxes after a match, and one allocated
/// button is linked into every matching box.
pub unsafe fn gm_801AE050(arg0: i32, arg1: i32, arg2: i32, arg3: f32, arg4: f32) -> bool {
    let mut var_r31 = gm_803DD4D0.next;
    let temp_r3 = ext::HSD_MemAlloc(core::mem::size_of::<unkd4d0_8>() as isize) as *mut unkd4d0_8;
    memzero(temp_r3 as *mut u8, core::mem::size_of::<unkd4d0_8>());
    (*temp_r3).unk8 = arg1;
    (*temp_r3).unkC = arg2;
    (*temp_r3).unk10 = arg3;
    (*temp_r3).unk14 = arg4;
    while !var_r31.is_null() {
        if (*var_r31).unkC == arg0 {
            let mut var_r4 = (*var_r31).unk8;
            if var_r4.is_null() {
                (*var_r31).unk8 = temp_r3;
                (*temp_r3).next = core::ptr::null_mut();
                (*temp_r3).unk4 = core::ptr::null_mut();
                button_setup(temp_r3);
                var_r31 = (*var_r31).next;
                continue;
            }
            loop {
                if !(*var_r4).next.is_null() {
                    var_r4 = (*var_r4).next;
                    continue;
                }
                (*var_r4).next = temp_r3;
                (*temp_r3).next = core::ptr::null_mut();
                (*temp_r3).unk4 = var_r4;
                button_setup(temp_r3);
                break;
            }
        }
        var_r31 = (*var_r31).next;
    }
    true
}

/// `gm_801AE44C(arg0, val)`: move the label joint of every box `arg0` by `-val` in Y.
pub unsafe fn gm_801AE44C(arg0: i32, val: f32) {
    let mut cur = gm_803DD4D0.next;
    while !cur.is_null() {
        if (*cur).unkC == arg0 {
            let jobj = GET_JOBJ((*cur).unk1C);
            HSD_JObjAddTranslationY((*(*(*(*jobj).child).child).child).next, -val);
        }
        cur = (*cur).next;
    }
}

/// `gm_801AE544(arg0, val)`: as gm_801AE44C, on the child of that joint.
pub unsafe fn gm_801AE544(arg0: i32, val: f32) {
    let mut cur = gm_803DD4D0.next;
    while !cur.is_null() {
        if (*cur).unkC == arg0 {
            let jobj = GET_JOBJ((*cur).unk1C);
            HSD_JObjAddTranslationY((*(*(*(*(*jobj).child).child).child).next).child, -val);
        }
        cur = (*cur).next;
    }
}

/// Set the diffuse colour of a button's joint and the colour of its text, then re-set the text.
unsafe fn recolor(var_r30: *mut unkd4d0_8, r: u8, g: u8, b: u8) {
    if !(*var_r30).unk20.is_null() {
        let jobj = GET_JOBJ((*var_r30).unk20);
        let mat = (*(*(*(*(*jobj).child).child).u.dobj).mobj).mat;
        (*mat).diffuse.r = r;
        (*mat).diffuse.g = g;
        (*mat).diffuse.b = b;
    }
    if !(*var_r30).unk24.is_null() {
        (*(*var_r30).unk24).text_color.r = r;
        (*(*var_r30).unk24).text_color.g = g;
        (*(*var_r30).unk24).text_color.b = b;
        ext::HSD_SisLib_803A6368((*var_r30).unk24, (*var_r30).unkC);
    }
}

unsafe fn recolor_matching(arg1: bool, r: u8, g: u8, b: u8) {
    let mut var_r31 = gm_803DD4D0.next;
    while !var_r31.is_null() {
        let mut var_r30 = (*var_r31).unk8;
        while !var_r30.is_null() {
            if (*var_r30).unk8 == arg1 as i32 {
                recolor(var_r30, r, g, b);
            }
            var_r30 = (*var_r30).next;
        }
        var_r31 = (*var_r31).next;
    }
}

/// `gm_801AE640(arg0, arg1)`: highlight (0xFF, 0xCC, 0x33) every button whose id is `arg1`.
pub unsafe fn gm_801AE640(_arg0: i32, arg1: bool) {
    recolor_matching(arg1, 0xFF, 0xCC, 0x33);
}

/// `gm_801AE74C(arg0, arg1)`: grey out (0x66) every button whose id is `arg1`.
pub unsafe fn gm_801AE74C(_arg0: i32, arg1: bool) {
    recolor_matching(arg1, 0x66, 0x66, 0x66);
}

/// `gm_801AE848(arg0)`: free every box whose id is `arg0` with its buttons, unlinking it from the list.
pub unsafe fn gm_801AE848(arg0: i32) {
    let mut var_r31 = gm_803DD4D0.next;
    while !var_r31.is_null() {
        if (*var_r31).unkC == arg0 {
            let mut var_r30 = (*var_r31).unk8;
            while !var_r30.is_null() {
                let temp_r29 = var_r30;
                var_r30 = (*var_r30).next;
                if !(*temp_r29).unk20.is_null() {
                    ext::HSD_GObjFree((*temp_r29).unk20);
                }
                if !(*temp_r29).unk24.is_null() {
                    ext::HSD_SisLib_803A5CC4((*temp_r29).unk24);
                }
                ext::HSD_Free(temp_r29 as *mut c_void);
            }
            if !(*var_r31).unk4.is_null() {
                (*(*var_r31).unk4).next = (*var_r31).next;
            }
            if !(*var_r31).next.is_null() {
                (*(*var_r31).next).unk4 = (*var_r31).unk4;
            }
            let temp_r30 = var_r31;
            var_r31 = (*var_r31).next;
            if !(*temp_r30).unk1C.is_null() {
                ext::HSD_GObjFree((*temp_r30).unk1C);
            }
            if !(*temp_r30).unk20.is_null() {
                ext::HSD_SisLib_803A5CC4((*temp_r30).unk20);
            }
            ext::HSD_Free(temp_r30 as *mut c_void);
        } else {
            var_r31 = (*var_r31).next;
        }
    }
}

/// `fn_801AE948(arg0)`: the window's per-frame proc; re-apply every box's and button's position.
pub unsafe fn fn_801AE948(_arg0: *mut HSD_GObj) {
    let mut var_r31 = gm_803DD4D0.next;
    while !var_r31.is_null() {
        let mut var_r30 = (*var_r31).unk8;
        if !(*var_r31).unk1C.is_null() {
            HSD_JObjSetTranslateX(GET_JOBJ((*var_r31).unk1C), (*var_r31).unk14);
            HSD_JObjSetTranslateY(GET_JOBJ((*var_r31).unk1C), (*var_r31).unk18);
        }
        while !var_r30.is_null() {
            if !(*var_r30).unk20.is_null() {
                HSD_JObjSetTranslateX(GET_JOBJ((*var_r30).unk20), (*var_r30).unk10);
                HSD_JObjSetTranslateY(GET_JOBJ((*var_r30).unk20), (*var_r30).unk14);
            }
            var_r30 = (*var_r30).next;
        }
        var_r31 = (*var_r31).next;
    }
}

/// `gm_801AEBB0()`: load NtMsgWin.dat and the message SIS, create the camera and the SIS text group, and
/// start the per-frame proc.
pub unsafe fn gm_801AEBB0() {
    ext::lbArchive_80016DBC("NtMsgWin.dat", &raw mut gm_804D6868, "ScNtcCommon_scene_data", 0);

    if ext::lbLang_IsSavedLanguageUS() {
        ext::HSD_SisLib_803A62A0(3, "SdMsgBox.usd", "SIS_MessageData");
    } else {
        ext::HSD_SisLib_803A62A0(3, "SdMsgBox.dat", "SIS_MessageData");
    }

    let gobj = ext::GObj_Create(20, 21, 0);
    {
        let desc = ext::HSD_CObjLoadDesc((*(*gm_804D6868).cameras).desc as *mut HSD_CObjDesc);
        ext::HSD_GObjObject_80390A70(gobj, ext::hsd_gobj_camera_kind(), desc as *mut c_void);
    }
    ext::GObj_SetupGXLinkMax(gobj, ext::HSD_GObj_803910D8 as usize, 0);
    (*gobj).gxlink_prios = 0x4000;

    gm_804D686C = ext::HSD_SisLib_803A611C(3, gobj, 9, 13, 0, 14, 0, 18);
    ext::HSD_GObj_SetupProc(ext::GObj_Create(15, 17, 0), fn_801AE948 as usize, 0);
}

/// `gm_801AECC4(model_idx)`: create a window piece from model `model_idx` (the box, 1, hides two of its parts).
pub unsafe fn gm_801AECC4(model_idx: i32) -> *mut HSD_GObj {
    let gobj = ext::GObj_Create(9, 13, 0);
    let model = *(*gm_804D6868).models.offset(model_idx as isize);
    let jobj = ext::HSD_JObjLoadJoint((*model).joint);
    ext::HSD_GObjObject_80390A70(gobj, ext::hsd_gobj_jobj_kind(), jobj as *mut c_void);
    ext::GObj_SetupGXLink(gobj, ext::HSD_GObj_JObjCallback as usize, 14, 0);
    ext::gm_8016895C(jobj, model, 0);
    ext::HSD_JObjReqAnimAll(jobj, 0.0);
    ext::HSD_JObjAnimAll(jobj);
    if model_idx == 1 {
        ext::HSD_JObjSetFlagsAll((*(*(*(*(*jobj).child).child).next).child).next, JOBJ_HIDDEN_FLAG);
        ext::HSD_JObjSetFlagsAll((*(*(*(*jobj).child).child).next).next, JOBJ_HIDDEN_FLAG);
    }
    gobj
}

/// `gm_801AEDC8()`: the triggers of every connected controller, OR'd.
pub unsafe fn gm_801AEDC8() -> u32 {
    let mut result = 0u32;
    for i in 0..4usize {
        if ext::hsd_pad_master_status_err(i) == 0 {
            result |= ext::hsd_pad_copy_status_trigger(i);
        }
    }
    result
}
