//! src/melee/lb/lbanim.c: building a JObj's AObj from a "figatree" track list (fighter figatree animation
//! on a joint). All 6 functions (4 public, the static `lbAnim_InitFrames` and the inlined
//! `lbAnim_JObjSortAnim`) are checked against the C by a state oracle (tests/tu_lbanim.rs over
//! gen/turef/tu_lbanim_ref.c): the FObj/AObj pools the stubbed allocators hand out, the JObj and the
//! callee log are compared after every call.
//!
//! The data structures are native: `FigaTrack` holds a real `u8*` (16 bytes natively, 0xC on the
//! GameCube; the GC layout is recorded below), so `track++` strides the native size, as in the C build.
//! The callees live in other TUs (baselib aobj/fobj/jobj); they are settable hooks in `ext`.
#![allow(non_snake_case, non_upper_case_globals, dead_code, static_mut_refs)]

use core::ptr::null_mut;
use melee_types::*;

pub use melee_types::{FigaTrack, FigaTree};

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
    pub static mut HSD_FOBJALLOC: Option<fn() -> *mut HSD_FObj> = None;
            #[allow(non_snake_case)]
            pub fn HSD_FObjAlloc() -> *mut HSD_FObj {
                match unsafe { HSD_FOBJALLOC } {
                    Some(f) => f(),
                    None => { return unsafe { crate::tu::fobj::HSD_FObjAlloc() as _ }; },
                }
            }
    pub static mut HSD_AOBJALLOC: Option<fn() -> *mut HSD_AObj> = None;
            #[allow(non_snake_case)]
            pub fn HSD_AObjAlloc() -> *mut HSD_AObj {
                match unsafe { HSD_AOBJALLOC } {
                    Some(f) => f(),
                    None => { return unsafe { crate::tu::aobj::HSD_AObjAlloc() as _ }; },
                }
            }
    pub static mut HSD_AOBJREMOVE: Option<fn(*mut HSD_AObj) > = None;
            #[allow(non_snake_case)]
            pub fn HSD_AObjRemove(aobj: *mut HSD_AObj)  {
                match unsafe { HSD_AOBJREMOVE } {
                    Some(f) => f(aobj),
                    None => { unsafe { let _ = crate::tu::aobj::HSD_AObjRemove(aobj as _); } return; },
                }
            }
    pub static mut HSD_AOBJSETFLAGS: Option<fn(*mut HSD_AObj, u32) > = None;
            #[allow(non_snake_case)]
            pub fn HSD_AObjSetFlags(aobj: *mut HSD_AObj, flags: u32)  {
                match unsafe { HSD_AOBJSETFLAGS } {
                    Some(f) => f(aobj, flags),
                    None => { unsafe { let _ = crate::tu::aobj::HSD_AObjSetFlags(aobj as _, flags as _); } return; },
                }
            }
    pub static mut HSD_AOBJSETFOBJ: Option<fn(*mut HSD_AObj, *mut HSD_FObj) > = None;
            #[allow(non_snake_case)]
            pub fn HSD_AObjSetFObj(aobj: *mut HSD_AObj, fobj: *mut HSD_FObj)  {
                match unsafe { HSD_AOBJSETFOBJ } {
                    Some(f) => f(aobj, fobj),
                    None => { unsafe { let _ = crate::tu::aobj::HSD_AObjSetFObj(aobj as _, fobj as _); } return; },
                }
            }
    pub static mut HSD_AOBJSETREWINDFRAME: Option<fn(*mut HSD_AObj, f32) > = None;
            #[allow(non_snake_case)]
            pub fn HSD_AObjSetRewindFrame(aobj: *mut HSD_AObj, frame: f32)  {
                match unsafe { HSD_AOBJSETREWINDFRAME } {
                    Some(f) => f(aobj, frame),
                    None => { unsafe { let _ = crate::tu::aobj::HSD_AObjSetRewindFrame(aobj as _, frame as _); } return; },
                }
            }
    pub static mut HSD_AOBJSETENDFRAME: Option<fn(*mut HSD_AObj, f32) > = None;
            #[allow(non_snake_case)]
            pub fn HSD_AObjSetEndFrame(aobj: *mut HSD_AObj, frame: f32)  {
                match unsafe { HSD_AOBJSETENDFRAME } {
                    Some(f) => f(aobj, frame),
                    None => { unsafe { let _ = crate::tu::aobj::HSD_AObjSetEndFrame(aobj as _, frame as _); } return; },
                }
            }
    hook!(HSD_JOBJSETFLAGS, HSD_JObjSetFlags, (jobj: *mut HSD_JObj, flags: u32));
    hook!(HSD_JOBJCLEARFLAGS, HSD_JObjClearFlags, (jobj: *mut HSD_JObj, flags: u32));
}

/// Copies one track into a freshly allocated FObj (the body shared by both builders).
#[inline]
unsafe fn fill_fobj(fobj: *mut HSD_FObj, track: *const FigaTrack) {
    (*fobj).startframe = (*track).startframe as i16;
    (*fobj).obj_type = (*track).obj_type;
    (*fobj).frac_value = (*track).frac_value;
    (*fobj).frac_slope = (*track).frac_slope;
    (*fobj).ad_head = (*track).ad_head;
    (*fobj).length = (*track).length as u32;
    (*fobj).flags = 0;
}

/// `static lbAnim_InitFrames(track, frames)`: one FObj per track, chained in order.
/// `frames <= 0` is undefined in the C (`fobj` is read uninitialised); callers guard `frames != 0`.
unsafe fn lbAnim_InitFrames(mut track: *const FigaTrack, frames: i8) -> *mut HSD_FObj {
    let mut fobj: *mut HSD_FObj = null_mut();
    let mut next: *mut HSD_FObj = null_mut();
    let mut result: *mut HSD_FObj = null_mut();
    let mut i: i32 = 0;
    while i < frames as i32 {
        fobj = ext::HSD_FObjAlloc();
        if i == 0 {
            result = fobj;
        }
        if !next.is_null() {
            (*next).next = fobj;
        }
        next = fobj;
        fill_fobj(fobj, track);
        track = track.add(1);
        i += 1;
    }
    (*fobj).next = null_mut();
    result
}

/// `fn_8001E60C(track, frames)`: like `lbAnim_InitFrames` but skips tracks of obj_type 5, 6 and 7. The C
/// advances `track` only for a track it kept, so a skipped track is looked at again on every remaining
/// iteration (kept bit for bit). Undefined in the C when no track is kept (`fobj` uninitialised).
pub unsafe fn fn_8001E60C(mut track: *const FigaTrack, frames: i8) -> *mut HSD_FObj {
    let mut fobj: *mut HSD_FObj = null_mut();
    let mut prev: *mut HSD_FObj = null_mut();
    let mut first: *mut HSD_FObj = null_mut();
    let mut i: i32 = 0;
    while i < frames as i32 {
        if (*track).obj_type != 5 && ((*track).obj_type.wrapping_sub(6)) > 1 {
            fobj = ext::HSD_FObjAlloc();
            if first.is_null() {
                first = fobj;
            }
            if !prev.is_null() {
                (*prev).next = fobj;
            }
            prev = fobj;
            fill_fobj(fobj, track);
            track = track.add(1);
        }
        i += 1;
    }
    (*fobj).next = null_mut();
    first
}

/// Duplicate of `HSD_JObjSortAnim` (inline in the C): moves the first TYPE_JOBJ FObj to the head.
unsafe fn lbAnim_JObjSortAnim(aobj: *mut HSD_AObj) {
    if aobj.is_null() || (*aobj).fobj.is_null() {
        return;
    }
    let mut fobj_ptr: *mut *mut HSD_FObj = &mut (*aobj).fobj;
    while !(*fobj_ptr).is_null() {
        let fobj = *fobj_ptr;
        if (*fobj).obj_type as i32 == TYPE_JOBJ {
            *fobj_ptr = (*fobj).next;
            (*fobj).next = (*aobj).fobj;
            (*aobj).fobj = fobj;
            break;
        }
        fobj_ptr = &mut (*fobj).next;
    }
}

/// Shared tail of the two public builders (they differ only in the FObj list builder).
unsafe fn set_anim(jobj: *mut HSD_JObj, tree: *const FigaTree, fobjs: unsafe fn(*const FigaTrack, i8) -> *mut HSD_FObj, track: *const FigaTrack, frames: i8) {
    if !jobj.is_null() && frames != 0 {
        if !(*jobj).aobj.is_null() {
            ext::HSD_AObjRemove((*jobj).aobj);
        }
        let aobj = ext::HSD_AObjAlloc();
        ext::HSD_AObjSetFlags(aobj, (*tree).flags);
        ext::HSD_AObjSetRewindFrame(aobj, 0.0);
        ext::HSD_AObjSetEndFrame(aobj, (*tree).frames);
        ext::HSD_AObjSetFObj(aobj, fobjs(track, frames));
        (*jobj).aobj = aobj;
        lbAnim_JObjSortAnim((*jobj).aobj);
        if (*tree).r#type & 1 != 0 {
            ext::HSD_JObjSetFlags(jobj, JOBJ_CLASSICAL_SCALE as u32);
        } else {
            ext::HSD_JObjClearFlags(jobj, JOBJ_CLASSICAL_SCALE as u32);
        }
    }
}

/// `lbAnim_8001E6D8(jobj, tree, track, frames)`: replace the JObj's animation by one FObj per track.
pub unsafe fn lbAnim_8001E6D8(jobj: *mut HSD_JObj, tree: *const FigaTree, track: *const FigaTrack, frames: i8) {
    set_anim(jobj, tree, lbAnim_InitFrames, track, frames)
}

/// `lbAnim_8001E7E8(jobj, tree, track, frames)`: as `lbAnim_8001E6D8` but via `fn_8001E60C` (no obj_type 5..7).
pub unsafe fn lbAnim_8001E7E8(jobj: *mut HSD_JObj, tree: *const FigaTree, track: *const FigaTrack, frames: i8) {
    set_anim(jobj, tree, fn_8001E60C, track, frames)
}

/// `lbAnim_8001E8F8(tree)`: the animation length, 0.0 for a null tree.
pub fn lbAnim_8001E8F8(tree: Option<&FigaTree>) -> f32 {
    tree.map_or(0.0, |t| t.frames)
}
