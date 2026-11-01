//! lbspdisplay.c: ports folded from the hand-written modules (consolidation steps 21/22).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::lb_ptr::checkJObjFlags;
#[allow(unused_imports)]
use crate::lb_r8::lb_80011B74;
#[allow(unused_imports)]
use melee_types::HSD_JObj as HSD_JObj_lb_r9;
#[allow(unused_imports)]
use melee_types::HSD_DObj;
#[allow(unused_imports)]
use melee_types as mt;
#[allow(unused_imports)]
use mt::CameraBlurData;
#[allow(unused_imports)]
use mt::HSD_GObj;
#[allow(unused_imports)]
use mt::HSD_GObjEvent;
#[allow(unused_imports)]
use mt::HSD_JObj;

// ---- folded from lb_r4.rs (consolidation steps 21/22) ----
/// lbspdisplay.c `lb_800138CC_fp`: set the camera blur's callback.
pub fn lb_800138CC_fp(gobj: &HSD_GObj, blur_callback: HSD_GObjEvent) {
    melee_core::ct!("lb_800138CC_fp");
    // SAFETY: the blur gobj's user data is its CameraBlurData (the C does not check it).
    let data = unsafe { &mut *(mt::inline::HSD_GObjGetUserData(gobj) as *mut CameraBlurData) };
    data.callback = blur_callback;
}

/// lbspdisplay.c `lb_800138D8_fp`: blur mode 1 with the given size.
pub fn lb_800138D8_fp(gobj: &HSD_GObj, size: i8) {
    melee_core::ct!("lb_800138D8_fp");
    // SAFETY: as in `lb_800138CC_fp`.
    let data = unsafe { &mut *(mt::inline::HSD_GObjGetUserData(gobj) as *mut CameraBlurData) };
    data.mode = 1;
    data.blur_size = size as u8;
}

/// lbspdisplay.c `lb_8001204C_fp`: for each index in `indices[..count]`, the joint at that
/// depth-first position under `root` (instances' children skipped), or NULL past the end;
/// returns how many were found. Ascending indices continue the walk from the previous one.
pub fn lb_8001204C_fp(root: Option<&HSD_JObj>, result: Option<&mut [*mut HSD_JObj]>, indices: Option<&[u16]>,
                   count: i32) -> i32 {
    melee_core::ct!("lb_8001204C_fp");
    let (Some(root), Some(out), Some(idx)) = (root, result, indices) else { return 0 };
    if count == 0 {
        return 0;
    }
    let root = root as *const HSD_JObj as *mut HSD_JObj;
    // SAFETY (all derefs below): the joints form a well-linked tree (child/next/parent).
    let get = |p: *mut HSD_JObj| unsafe { p.as_ref() };
    let mut found = 0;
    let mut prev: i32 = -1;
    let mut jobj: *mut HSD_JObj = core::ptr::null_mut();
    let mut cur: i32;
    let mut k = 0usize;
    let mut i = count;
    while i > 0 {
        let target = idx[k];
        if prev > target as i32 || prev == -1 {
            jobj = root;
            cur = 0;
        } else {
            cur = prev;
        }
        while let Some(j) = get(jobj) {
            if cur == target as i32 {
                break;
            }
            let mut saved = jobj;
            let next_node;
            if j.flags & mt::JOBJ_INSTANCE as u32 == 0 && !mt::inline::HSD_JObjGetChild(Some(j)).is_null() {
                next_node = mt::inline::HSD_JObjGetChild(Some(j));
            } else if !mt::inline::HSD_JObjGetNext(Some(j)).is_null() {
                next_node = mt::inline::HSD_JObjGetNext(Some(j));
            } else {
                loop {
                    let parent = mt::inline::HSD_JObjGetParent(get(saved));
                    if parent.is_null() {
                        next_node = core::ptr::null_mut();
                        break;
                    }
                    let pn = mt::inline::HSD_JObjGetNext(get(parent));
                    if !pn.is_null() {
                        saved = pn;
                        next_node = saved;
                        break;
                    }
                    saved = parent;
                }
            }
            jobj = next_node;
            cur += 1;
        }
        out[k] = jobj;
        prev = cur;
        if !jobj.is_null() {
            found += 1;
        }
        k += 1;
        i -= 1;
    }
    found
}

// ---- folded from lb_r8.rs (consolidation steps 21/22) ----
/// lbspdisplay.c `lb_80011B74_fp`: OR `flags` into the render mode of every DObj's MObj, last DObj
/// in the `next` chain first (the C recurses before it writes).
///
/// # Safety
/// `dobj`'s `next` chain ends in NULL and each DObj's `mobj` is live.
pub unsafe fn lb_80011B74_fp(dobj: &mut HSD_DObj, flags: u32) {
    melee_core::ct!("lb_80011B74_fp");
    if let Some(n) = dobj.next.as_mut() {
        lb_80011B74_fp(n, flags);
    }
    (*dobj.mobj).rendermode |= flags;
}

// ---- folded from lb_r9.rs (consolidation steps 21/22) ----
/// lbspdisplay.c `lb_80011C18_fp`: OR `flags` into the render mode of every MObj under `jobj`
/// (its child and next subtrees, the child/next joints' DObj chains, the joint's own DObjs).
///
/// # Safety
/// `jobj` is a live JObj tree; every joint `checkJObjFlags` accepts holds a NULL or live DObj
/// chain whose DObjs all have a live MObj (as in the C, which does not check `mobj`).
pub unsafe fn lb_80011C18_fp(jobj: &mut HSD_JObj_lb_r9, flags: u32) {
    melee_core::ct!("lb_80011C18_fp");
    for cur in [jobj.child, jobj.next] {
        if let Some(cur) = cur.as_mut() {
            if let Some(c) = cur.child.as_mut() {
                lb_80011C18_fp(c, flags);
            }
            if let Some(n) = cur.next.as_mut() {
                lb_80011C18_fp(n, flags);
            }
            if checkJObjFlags(cur) {
                if let Some(d) = cur.u.dobj.as_mut() {
                    lb_80011B74(d, flags);
                }
            }
        }
    }
    if checkJObjFlags(jobj) {
        if let Some(d) = jobj.u.dobj.as_mut() {
            if let Some(n) = d.next.as_mut() {
                lb_80011B74(n, flags);
            }
            (*d.mobj).rendermode |= flags;
        }
    }
}
