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
// BEGIN GENERATED EXT (rust/melee_runtime/gen/mtu/gen_tu.py; the callees of the C TU)
pub mod ext {
    #![allow(non_upper_case_globals, non_snake_case, unused_imports, dead_code)]
    use super::*;
    use core::ffi::c_void;
    macro_rules! ext_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)? $(= $def:expr)?;)*) => {
            pub mod hook {
                use super::*;
                $( pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } {
                    Some(f) => f($($a),*),
                    None => crate::tu::hookmac::fallback!(stringify!($name) $(, $def)?),
                }
            } )*
        };
    }
    ext_fns! {
    fn HSD_AObjAlloc() -> *mut HSD_AObj = crate::tu::aobj::HSD_AObjAlloc() as _;
    fn HSD_AObjRemove(a0: *mut HSD_AObj) = crate::tu::aobj::HSD_AObjRemove(a0 as _);
    fn HSD_AObjSetEndFrame(a0: *mut HSD_AObj, a1: f32) = crate::tu::aobj::HSD_AObjSetEndFrame(a0 as _, a1 as _);
    fn HSD_AObjSetFObj(a0: *mut HSD_AObj, a1: *mut HSD_FObj) = crate::tu::aobj::HSD_AObjSetFObj(a0 as _, a1 as _);
    fn HSD_AObjSetFlags(a0: *mut HSD_AObj, a1: u32) = crate::tu::aobj::HSD_AObjSetFlags(a0 as _, a1 as _);
    fn HSD_AObjSetRewindFrame(a0: *mut HSD_AObj, a1: f32) = crate::tu::aobj::HSD_AObjSetRewindFrame(a0 as _, a1 as _);
    fn HSD_FObjAlloc() -> *mut HSD_FObj = crate::tu::fobj::HSD_FObjAlloc() as _;
    fn HSD_JObjClearFlags(a0: *mut HSD_JObj, a1: u32) = crate::tu::jobj_world::HSD_JObjClearFlags(a0, a1);
    fn HSD_JObjSetFlags(a0: *mut HSD_JObj, a1: u32) = crate::tu::jobj_world::HSD_JObjSetFlags(a0, a1);
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
}
// END GENERATED EXT

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
pub unsafe fn lbAnim_InitFrames(mut track: *const FigaTrack, frames: i8) -> *mut HSD_FObj {
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
