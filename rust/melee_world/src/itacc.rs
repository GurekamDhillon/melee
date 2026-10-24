//! The plain `Item` field accessors of `src/melee/it/**` (`itGetKind`, `it_8026B294`, the Tincle /
//! coin / hitbox flag pokes, the stage article table `it_804A0F60`, ...) in the C shape (an item
//! `HSD_GObj*`, `gobj->user_data` is the `Item`), for the stage TUs that call them. The item crate
//! ports the same functions over `&Item` (its `itlib`) and over its own `ext` layer; these are the
//! raw-pointer C-named ones the world TUs need, and read the same records. No float arithmetic.
//! `Mex_IndexStageItem` (the TARGET_PC m-ex stage article registration) is the [`MEX_INDEX_STAGE_ITEM`]
//! hook: unset, a custom-stage article index (`kind >= 5000`) is not registered.
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, clippy::missing_safety_doc)]

use melee_types::*;

/// `it_804A0F60`: the stage article table, indexed by `kind - It_Kind_Old_Kuri`.
pub static mut it_804A0F60: [*mut Article; 30] = [core::ptr::null_mut(); 30];

/// `Mex_IndexStageItem(article, index)`: register a custom stage's article (platform layer).
pub static mut MEX_INDEX_STAGE_ITEM: Option<fn(*mut Article, i32)> = None;

#[inline(always)]
unsafe fn ip<'a>(gobj: *mut HSD_GObj) -> &'a mut Item {
    &mut *((*gobj).user_data as *mut Item)
}

/// `itGetKind`.
pub unsafe fn itGetKind(gobj: *mut HSD_GObj) -> ItemKind {
    ip(gobj).kind
}

/// `it_8026B294`: the item's position.
pub unsafe fn it_8026B294(gobj: *mut HSD_GObj, pos: *mut Vec) {
    *pos = ip(gobj).pos;
}

/// `it_8026B718`: set the hitlag frames.
pub unsafe fn it_8026B718(gobj: *mut HSD_GObj, hitlag_frames: f32) {
    ip(gobj).xCBC_hitlagFrames = hitlag_frames;
}

/// `it_8026C1E8`: can the item be grabbed / picked up (no victim, and not owned while flag x13).
pub unsafe fn it_8026C1E8(gobj: *mut HSD_GObj) -> bool {
    let i = ip(gobj);
    !(!i.grab_victim.is_null() || (i.xDC8_word.flags.x13() != 0 && !i.owner.is_null()))
}

/// `it_8026B3C0`: how many live items of `kind` there are (the item p-link, `HSD_GObjPLinkHead[HSD_GOBJ_PLINK_ITEM]`).
pub unsafe fn it_8026B3C0(kind: ItemKind) -> i32 {
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

/// `it_ResetStageArticleTable`.
pub fn it_ResetStageArticleTable() {
    unsafe {
        for i in 0..30 {
            it_804A0F60[i] = core::ptr::null_mut();
        }
    }
}

/// `it_8026B40C`: register a stage article (`kind >= 5000`: a custom stage's article index).
pub unsafe fn it_8026B40C(article: *mut Article, kind: i32) {
    if kind >= 5000 {
        if let Some(f) = MEX_INDEX_STAGE_ITEM {
            f(article, kind - 5000);
        }
        return;
    }
    it_804A0F60[(kind - It_Kind_Old_Kuri) as usize] = article;
}

/// `it_80275414`.
pub unsafe fn it_80275414(gobj: *mut HSD_GObj) {
    ip(gobj).xDCD_flag.anon_0.set_b7(1);
}

/// `it_802756D0`.
pub unsafe fn it_802756D0(gobj: *mut HSD_GObj) {
    ip(gobj).xD0C = 2;
}

/// `it_802714C0`: let the hurtboxes follow the item again.
pub unsafe fn it_802714C0(gobj: *mut HSD_GObj) {
    let i = ip(gobj);
    for k in 0..i.xAC8_hurtboxNum as usize {
        i.xACC_itemHurtbox[k].set_skip_update_pos(0);
    }
    i.anon_daa.xDAA_flag.anon_0.set_b1(1);
}

/// `it_802756E0`.
pub unsafe fn it_802756E0(gobj: *mut HSD_GObj) {
    ip(gobj).xD0C = 0;
    it_802714C0(gobj);
}

/// `it_80274C60`.
pub unsafe fn it_80274C60(gobj: *mut HSD_GObj) {
    ip(gobj).xDC8_word.flags.set_xC(0);
}

/// `it_802F2014` (coin): the only joint of the item collision.
pub unsafe fn it_802F2014(gobj: *mut HSD_GObj, arg1: i32) {
    ip(gobj).x378_itemColl.joint_id_only = arg1;
}

/// `it_802F2020` (coin).
pub unsafe fn it_802F2020(gobj: *mut HSD_GObj) -> i32 {
    ip(gobj).xDD4_itemVar.coin.x0
}

/// `it_802ECC8C` (Tincle).
pub unsafe fn it_802ECC8C(gobj: *mut HSD_GObj) -> i32 {
    ip(gobj).xDD4_itemVar.tincle.x20
}

/// `it_802ECC98` (Tincle).
pub unsafe fn it_802ECC98(gobj: *mut HSD_GObj, arg1: f32) {
    ip(gobj).xDD4_itemVar.tincle.x54 = arg1;
}

/// `it_802ECCA4` (Tincle): its position lifted by `x50`, and the state id.
pub unsafe fn it_802ECCA4(gobj: *mut HSD_GObj, kind: *mut i32, pos: *mut Vec) {
    let i = ip(gobj);
    *pos = i.pos;
    (*pos).y += i.xDD4_itemVar.tincle.x50;
    *kind = i.msid;
}

/// `it_802ECD1C` (Kyasarin).
pub unsafe fn it_802ECD1C(gobj: *mut HSD_GObj, v: *mut Vec) {
    *v = ip(gobj).pos;
}
