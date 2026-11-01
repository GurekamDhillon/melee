//! it_26B1.c: ports folded from the hand-written modules (consolidation steps 21/22).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types::*;
#[allow(unused_imports)]
use crate::itacc::{MEX_INDEX_STAGE_ITEM, ip, it_804A0F60};

// ---- folded from itacc.rs (consolidation steps 21/22) ----
/// `itGetKind_fp`.
pub unsafe fn itGetKind_fp(gobj: *mut HSD_GObj) -> ItemKind {
    melee_core::ct!("itGetKind_fp");
    ip(gobj).kind
}

/// `it_8026B294_fp`: the item's position.
pub unsafe fn it_8026B294_fp(gobj: *mut HSD_GObj, pos: *mut Vec) {
    melee_core::ct!("it_8026B294_fp");
    *pos = ip(gobj).pos;
}

/// `it_8026B718_fp`: set the hitlag frames.
pub unsafe fn it_8026B718_fp(gobj: *mut HSD_GObj, hitlag_frames: f32) {
    melee_core::ct!("it_8026B718_fp");
    ip(gobj).xCBC_hitlagFrames = hitlag_frames;
}

/// `it_8026C1E8_fp`: can the item be grabbed / picked up (no victim, and not owned while flag x13).
pub unsafe fn it_8026C1E8_fp(gobj: *mut HSD_GObj) -> bool {
    melee_core::ct!("it_8026C1E8_fp");
    let i = ip(gobj);
    !(!i.grab_victim.is_null() || (i.xDC8_word.flags.x13() != 0 && !i.owner.is_null()))
}

/// `it_8026B3C0_fp`: how many live items of `kind` there are (the item p-link, `HSD_GObjPLinkHead[HSD_GOBJ_PLINK_ITEM]`).
pub unsafe fn it_8026B3C0_fp(kind: ItemKind) -> i32 {
    melee_core::ct!("it_8026B3C0_fp");
    let mut n = 0;
    let mut cur = *melee_runtime::tu::gobj_native::HSD_GObjPLinkHead.add(HSD_GOBJ_PLINK_ITEM as usize);
    while !cur.is_null() {
        if ip(cur).kind == kind {
            n += 1;
        }
        cur = (*cur).next;
    }
    n
}

/// `it_ResetStageArticleTable_fp`.
pub fn it_ResetStageArticleTable_fp() {
    melee_core::ct!("it_ResetStageArticleTable_fp");
    unsafe {
        for i in 0..30 {
            it_804A0F60[i] = core::ptr::null_mut();
        }
    }
}

/// `it_8026B40C_fp`: register a stage article (`kind >= 5000`: a custom stage's article index).
pub unsafe fn it_8026B40C_fp(article: *mut Article, kind: i32) {
    melee_core::ct!("it_8026B40C_fp");
    if kind >= 5000 {
        if let Some(f) = MEX_INDEX_STAGE_ITEM {
            f(article, kind - 5000);
        }
        return;
    }
    it_804A0F60[(kind - It_Kind_Old_Kuri) as usize] = article;
}
