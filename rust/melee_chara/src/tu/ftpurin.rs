//! src/melee/ft/kinds/ftPurin/ftpurin.c: Jigglypuff's init callbacks and her Akaneia-aware hat
//! (a second joint tree drawn from the left leg joint). No float arithmetic, so no fused sites
//! (tools/rust-port/fma_sites.py). Not smoke-tested yet (hooks-heavy; only compiled).
//!
//! Data: the file-name strings and small tables are below; `ftPr_Init_MotionStateTable` (32 rows of
//! Anim/IASA/Phys/Coll pointers into the ftPurin special TUs and ftCommon) belongs to the
//! motion-state dispatcher and is TODO. `ftPr_CostumeList` is zero-initialised BSS.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types::{Fighter, HSD_GObj};
use crate::stubs as st;
use crate::tu::ftclink::Fighter_OnItemDrop;
use melee_types::*;

/// Already ported (round 5): re-exported under this TU.


pub const ftPr_Init_DatFilename: &str = "PlPr.dat";
pub const ftPr_Init_DataName: &str = "ftDataPurin";
pub const ftPr_Init_AnimDatFilename: &str = "PlPrAJ.dat";
/// `ftPr_Init_DemoMotionFilenames` (result, intro, ending, vi-wait).
pub const ftPr_Init_DemoMotionFilenames: [&str; 4] = [
    "ftDemoResultMotionFilePurin",
    "ftDemoIntroMotionFilePurin",
    "ftDemoEndingMotionFilePurin",
    "ftDemoViWaitMotionFilePurin",
];
/// `ftPr_Init_CostumeStrings`: (dat, joint, matanim joint) per costume.
pub const ftPr_Init_CostumeStrings: [[&str; 3]; 5] = [
    ["PlPrNr.dat", "PlyPurin5K_Share_joint", "PlyPurin5K_Share_matanim_joint"],
    ["PlPrRe.dat", "PlyPurin5KRe_Share_joint", "PlyPurin5KRe_Share_matanim_joint"],
    ["PlPrBu.dat", "PlyPurin5KBu_Share_joint", "PlyPurin5KBu_Share_matanim_joint"],
    ["PlPrGr.dat", "PlyPurin5KGr_Share_joint", "PlyPurin5KGr_Share_matanim_joint"],
    ["PlPrYe.dat", "PlyPurin5KYe_Share_joint", "PlyPurin5KYe_Share_matanim_joint"],
];
/// `ftPr_Init_803D05B4`: the hat joint name per costume (none for the default).
pub const ftPr_Init_803D05B4: [Option<&str>; 5] = [
    None,
    Some("PlyPurinReHat_TopN_joint"),
    Some("PlyPurinBuHat_TopN_joint"),
    Some("PlyPurinGrHat_TopN_joint"),
    Some("PlyPurinYeHat_TopN_joint"),
];
pub const ftPr_Init_803D05C8: [f32; 4] = [0.65, 0.7, 0.8, 1.0];
pub const ftPr_Init_803D05D8: [f32; 4] = [1.1, 1.35, 1.3, 1.2];
pub const ftPr_Init_assert_msg_0: &str = "!(jobj->flags & JOBJ_USE_QUATERNION)";
/// `ftPr_Init_803D0610`: the Pound collision box.
pub static ftPr_Init_803D0610: ftCollisionBox = ftCollisionBox {
    top: 8.0,
    bottom: 0.0,
    left: Vec2 { x: -4.0, y: 4.0 },
    right: Vec2 { x: 4.0, y: 4.0 },
};

/// `ft_8045A1E0`: the per-costume hat joint cache (one extra slot, see the C todo).
pub static mut ft_8045A1E0: [*mut HSD_Joint; 6] = [core::ptr::null_mut(); 6];

/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics.
pub mod ext {
    use core::ffi::c_void;
    use melee_types::*;
    macro_rules! hook {
        ($hook:ident, $name:ident, ($($a:ident: $t:ty),*) $(-> $r:ty)?) => {
            pub static mut $hook: Option<fn($($t),*) $(-> $r)?> = None;
            #[allow(non_snake_case)]
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $hook } {
                    Some(f) => f($($a),*),
                    None => melee_core::unported!(concat!("TODO: ", stringify!($name), " is not ported")),
                }
            }
        };
    }
    /// `Mex_CostumeVisIdx(fk, costume)` (TARGET_PC, ft/inlines.h): the raw costume id when there
    /// is no mexData (vanilla disc); the m-ex row lookup is installed here.
    pub static mut MEX_COSTUMEVISIDX: Option<fn(fk: i32, costume: i32) -> i32> = None;
    #[allow(non_snake_case)]
    pub fn Mex_CostumeVisIdx(fk: i32, costume: i32) -> i32 {
        match unsafe { MEX_COSTUMEVISIDX } {
            Some(f) => f(fk, costume),
            None => costume,
        }
    }
    /// `CostumeListsForeachCharacter[kind].costume_list`.
    crate::port_hook!(COSTUMELIST, CostumeListsForeachCharacter_costume_list, crate::adapt::CostumeListsForeachCharacter_costume_list, (kind: i32) -> *mut UnkCostumeStruct);
    crate::port_hook!(HSD_ARCHIVEGETPUBLICADDRESS, HSD_ArchiveGetPublicAddress, crate::adapt::HSD_ArchiveGetPublicAddress, (archive: *mut HSD_Archive, name: &str) -> *mut c_void);
    /// `&fighter_x2040_alloc_data`.
    crate::port_hook!(FIGHTER_X2040_ALLOC_DATA, fighter_x2040_alloc_data, crate::adapt::fighter_x2040_alloc_data, () -> *mut c_void);
    pub static mut HSD_OBJALLOC: Option<fn(*mut c_void) -> *mut c_void> = None;
            #[allow(non_snake_case)]
            pub fn HSD_ObjAlloc(data: *mut c_void) -> *mut c_void {
                match unsafe { HSD_OBJALLOC } {
                    Some(f) => f(data),
                    None => { return unsafe { melee_runtime::tu::objalloc::HSD_ObjAlloc(data as _) as _ }; },
                }
            }
    pub static mut HSD_OBJFREE: Option<fn(*mut c_void, *mut c_void) > = None;
            #[allow(non_snake_case)]
            pub fn HSD_ObjFree(data: *mut c_void, obj: *mut c_void)  {
                match unsafe { HSD_OBJFREE } {
                    Some(f) => f(data, obj),
                    None => { unsafe { melee_runtime::tu::objalloc::HSD_ObjFree(data as _, obj as _); } return; },
                }
            }
    crate::port_hook!(FTPARTSPOBJSETDEFAULTCLASS, ftPartsPObjSetDefaultClass, crate::adapt::ftPartsPObjSetDefaultClass, ());
    crate::port_hook!(FTPARTSPOBJCLEARDEFAULTCLASS, ftPartsPObjClearDefaultClass, crate::adapt::ftPartsPObjClearDefaultClass, ());
    pub static mut HSD_JOBJLOADJOINT: Option<fn(*mut HSD_Joint) -> *mut HSD_JObj> = None;
            #[allow(non_snake_case)]
            pub fn HSD_JObjLoadJoint(joint: *mut HSD_Joint) -> *mut HSD_JObj {
                match unsafe { HSD_JOBJLOADJOINT } {
                    Some(f) => f(joint),
                    None => { return unsafe { melee_runtime::tu::jobj::HSD_JObjLoadJoint(joint as _) as _ }; },
                }
            }
    crate::port_hook!(HSD_JOBJREMOVEALL, HSD_JObjRemoveAll, crate::adapt::HSD_JObjRemoveAll, (jobj: *mut HSD_JObj));
    crate::port_hook!(FTPARTS_80075650, ftParts_80075650, crate::adapt::ftParts_80075650, (gobj: *mut HSD_GObj, jobj: *mut HSD_JObj, list: *mut DObjList));
    crate::port_hook!(FTPARTS_8007487C, ftParts_8007487C, crate::adapt::ftParts_8007487C,
        (desc: *mut FtPartsDesc, vis: *mut FtPartsVis, costume_id: u32, a: *mut DObjList, b: *mut DObjList));
    pub static mut FTPARTS_80074CA0: Option<fn(*mut FtPartsVis, i32, *mut DObjList)> = None;
    #[allow(non_snake_case)]
    pub fn ftParts_80074CA0(vis: *mut FtPartsVis, idx: i32, list: *mut DObjList) {
        match unsafe { FTPARTS_80074CA0 } {
            Some(f) => f(vis, idx, list),
            None => { unsafe { melee_fighter::tu::ftparts::ftParts_80074CA0(&mut *(vis as *mut _), idx as _, list as _); } },
        }
    }
    pub static mut FTPARTS_80074D7C: Option<fn(*mut FtPartsVis, i32, *mut DObjList)> = None;
    #[allow(non_snake_case)]
    pub fn ftParts_80074D7C(vis: *mut FtPartsVis, idx: i32, list: *mut DObjList) {
        match unsafe { FTPARTS_80074D7C } {
            Some(f) => f(vis, idx, list),
            None => { unsafe { melee_fighter::tu::ftparts::ftParts_80074D7C(&mut *(vis as *mut _), idx as _, list as _); } },
        }
    }
    pub static mut FTCO_8009DC54: Option<fn(*mut Fighter)> = None;
    #[allow(non_snake_case)]
    pub fn ftCo_8009DC54(fp: *mut Fighter) {
        match unsafe { FTCO_8009DC54 } {
            Some(f) => f(fp),
            None => { unsafe { melee_fighter::tu::ftdynamics::ftCo_8009DC54(&mut *(fp as *mut _)); } },
        }
    }
    crate::port_hook!(HSD_JOBJGETMTXPTR, HSD_JObjGetMtxPtr, crate::adapt::HSD_JObjGetMtxPtr, (jobj: *mut HSD_JObj) -> *mut [f32; 4]);
    crate::port_hook!(HSD_JOBJCOPYMTX, HSD_JObjCopyMtx, crate::adapt::HSD_JObjCopyMtx, (jobj: *mut HSD_JObj, mtx: *mut [f32; 4]));

    crate::port_hook!(HSD_JOBJSETMTXDIRTYSUB, HSD_JObjSetMtxDirtySub, crate::adapt::HSD_JObjSetMtxDirtySub, (jobj: *mut HSD_JObj));
    pub static mut HSD_JOBJDISPALL: Option<fn(*mut HSD_JObj, *mut [f32; 4], u32, u32)> = None;
    #[allow(non_snake_case)]
    pub fn HSD_JObjDispAll(jobj: *mut HSD_JObj, vmtx: *mut [f32; 4], flags: u32, rendermode: u32) {
        match unsafe { HSD_JOBJDISPALL } {
            Some(f) => f(jobj, vmtx, flags, rendermode),
            None => { unsafe { melee_runtime::tu::jobj::HSD_JObjDispAll(jobj as _, vmtx as *const _, flags as _, rendermode as _); } },
        }
    }
    pub static mut HSD_GOBJ_80390EB8: Option<fn(i32) -> u32> = None;
    #[allow(non_snake_case)]
    pub fn HSD_GObj_80390EB8(arg: i32) -> u32 {
        match unsafe { HSD_GOBJ_80390EB8 } {
            Some(f) => f(arg),
            None => { unsafe { melee_runtime::tu::gobj::HSD_GObj_80390EB8(arg as _) as _ } },
        }
    }
    pub static mut ITGETHOLDKIND: Option<fn(*mut HSD_GObj) -> i32> = None;
            #[allow(non_snake_case)]
            pub fn itGetHoldKind(item: *mut HSD_GObj) -> i32 {
                match unsafe { ITGETHOLDKIND } {
                    Some(f) => f(item),
                    None => { return unsafe { melee_items::tu::it_26B1::itGetHoldKind(item as _) as _ }; },
                }
            }
    pub static mut FTANIM_80070FB4: Option<fn(*mut HSD_GObj, i32, i32)> = None;
    #[allow(non_snake_case)]
    pub fn ftAnim_80070FB4(gobj: *mut HSD_GObj, a: i32, b: i32) {
        match unsafe { FTANIM_80070FB4 } {
            Some(f) => f(gobj, a, b),
            None => { unsafe { let _ = melee_fighter::ftanim::ftanim_80070fb4(gobj as _, a as _, b as _); } },
        }
    }
    /// `efSync_Spawn(gfx_id, gobj, ...)` (variadic in C; `va` are the extra word arguments).
    pub static mut EFSYNC_SPAWN: Option<fn(i32, *mut HSD_GObj, &[usize]) -> *mut c_void> = None;
    pub fn efSync_Spawn(id: i32, gobj: *mut HSD_GObj, va: &[usize]) -> *mut c_void {
        match unsafe { EFSYNC_SPAWN } {
            Some(f) => f(id, gobj, va),
            None => unsafe { crate::adapt::efSync_Spawn_usize(id, gobj, va) },
        }
    }
}

type G = *mut HSD_GObj;

#[inline(always)]
unsafe fn fpp(gobj: G) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

/// `PUSH_ATTRS(fp, ftPurinAttributes)`.
unsafe fn push_attrs(fp: *mut Fighter) {
    let backup = (*fp).dat_attrs_backup;
    let src = (*(*fp).ft_data).ext_attr as *const ftPurinAttributes;
    *((*fp).dat_attrs_backup as *mut ftPurinAttributes) = *src;
    (*fp).dat_attrs = backup;
}

/// `Fighter_SetEffectHitlagCallbacks` (ft/inlines.h).
#[inline(always)]
unsafe fn set_effect_hitlag_callbacks(fp: *mut Fighter) {
    (*fp).pre_hitlag_cb = Some(crate::chars::cb::ext__efLib_PauseAll);
    (*fp).post_hitlag_cb = Some(crate::chars::cb::ext__efLib_ResumeAll);
}

pub fn ftPr_Init_8013C2F8() {
    melee_core::ct!("ftPr_Init_8013C2F8");
    unsafe {
        ft_8045A1E0[0] = core::ptr::null_mut();
        ft_8045A1E0[1] = core::ptr::null_mut();
        ft_8045A1E0[2] = core::ptr::null_mut();
        ft_8045A1E0[3] = core::ptr::null_mut();
        ft_8045A1E0[4] = core::ptr::null_mut();
    }
}

pub fn ftPr_Init_OnUserDataRemove(gobj: G) {
    melee_core::ct!("ftPr_Init_OnUserDataRemove");
    ftPr_Init_8013C494(gobj);
}

pub fn ftPr_Init_8013C360(gobj: G) {
    melee_core::ct!("ftPr_Init_8013C360");
    unsafe {
        let joints = ft_8045A1E0.as_mut_ptr();
        let fp = fpp(gobj);
        let vis = ext::Mex_CostumeVisIdx((*fp).kind as i32, (*fp).x619_costume_id as i32) as u32;

        if let Some(name) = ftPr_Init_803D05B4[vis as usize] {
            let items = (*(*fp).ft_data).x48_items;
            let items_shifted = *items.add(1) as *mut *mut core::ffi::c_void;

            if (*joints.add(vis as usize)).is_null() {
                let costume_list = ext::CostumeListsForeachCharacter_costume_list((*fp).kind as i32);
                *joints.add(vis as usize) = ext::HSD_ArchiveGetPublicAddress(
                    (*costume_list.add((*fp).x619_costume_id as usize)).x14_archive,
                    name,
                ) as *mut HSD_Joint;
            }

            (*fp).u.pr.x2240.data = ext::HSD_ObjAlloc(ext::fighter_x2040_alloc_data()) as *mut *mut HSD_DObj;
            ext::ftPartsPObjSetDefaultClass();
            (*fp).u.pr.x223C = ext::HSD_JObjLoadJoint(*joints.add(vis as usize));
            (*fp).set_x2225_b2(1);
            ext::ftPartsPObjClearDefaultClass();
            ext::ftParts_80075650(gobj, (*fp).u.pr.x223C, &mut (*fp).u.pr.x2240);

            ext::ftParts_8007487C(
                items_shifted.add(1) as *mut FtPartsDesc,
                &mut (*fp).u.pr.x2248,
                vis,
                &mut (*fp).u.pr.x2240,
                &mut (*fp).u.pr.x2240,
            );
            ext::ftCo_8009DC54(fp);
            return;
        }

        (*fp).u.pr.x223C = core::ptr::null_mut();
    }
}

pub fn ftPr_Init_8013C494(gobj: G) {
    melee_core::ct!("ftPr_Init_8013C494");
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).u.pr.x223C.is_null() {
            ext::HSD_JObjRemoveAll((*fp).u.pr.x223C);
            (*fp).u.pr.x223C = core::ptr::null_mut();
            ext::HSD_ObjFree(ext::fighter_x2040_alloc_data(), (*fp).u.pr.x2240.data as *mut core::ffi::c_void);
            (*fp).u.pr.x2240.data = core::ptr::null_mut();
        }
    }
}

pub fn ftPr_Init_UnkMtxFunc0(gobj: G, arg1: i32, vmtx: *mut [f32; 4]) {
    melee_core::ct!("ftPr_Init_UnkMtxFunc0");
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).u.pr.x223C.is_null() && (*fp).x2225_b2() != 0 {
            let mtx = ext::HSD_JObjGetMtxPtr((*(*fp).parts.add(FtPart_LLegJA as usize)).joint);
            let jobj = (*fp).u.pr.x223C;
            ext::HSD_JObjCopyMtx((*fp).u.pr.x223C, mtx);
            (*jobj).flags |= (JOBJ_USER_DEF_MTX | JOBJ_MTX_INDEP_PARENT | JOBJ_MTX_INDEP_SRT) as u32;
            // HSD_JObjSetMtxDirty(jobj): HSD_JObjMtxIsDirty is false once USER_DEF_MTX is set.
            let is_dirty = ((*jobj).flags & JOBJ_USER_DEF_MTX as u32) == 0 && ((*jobj).flags & JOBJ_MTX_DIRTY as u32) != 0;
            if !is_dirty {
                ext::HSD_JObjSetMtxDirtySub(jobj);
            }

            ext::HSD_JObjDispAll((*fp).u.pr.x223C, vmtx, ext::HSD_GObj_80390EB8(arg1), 0);
        }
    }
}

pub fn ftPr_Init_UnkIntBoolFunc0(fp: *mut Fighter, arg1: i32, arg2: i32) {
    melee_core::ct!("ftPr_Init_UnkIntBoolFunc0");
    unsafe {
        if !(*fp).u.pr.x223C.is_null() {
            if arg2 != 0 {
                ext::ftParts_80074CA0(&mut (*fp).u.pr.x2248, arg1, &mut (*fp).u.pr.x2240);
            } else {
                ext::ftParts_80074D7C(&mut (*fp).u.pr.x2248, arg1, &mut (*fp).u.pr.x2240);
            }
        }
    }
}

/// Returns the hat joint, or (a correctness hack in the C) the gobj itself when there is none.
pub fn ftPr_Init_UnkMotionStates6(gobj: G) -> *mut HSD_JObj {
    melee_core::ct!("ftPr_Init_UnkMotionStates6");
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).u.pr.x223C.is_null() {
            return (*fp).u.pr.x223C;
        }
        gobj as *mut HSD_JObj
    }
}

pub fn ftPr_Init_OnLoad(gobj: G) {
    melee_core::ct!("ftPr_Init_OnLoad");
    unsafe {
        let fp = fpp(gobj);
        push_attrs(fp);
        (*fp).set_can_multijump(1);
        (*fp).x2D0 = (*fp).dat_attrs as *mut Fighter_x2D0_t;
        ftPr_Init_8013C360(gobj);
    }
}

pub fn ftPr_Init_OnItemPickup(gobj: G, flag: i32) {
    melee_core::ct!("ftPr_Init_OnItemPickup");
    unsafe {
        let fp = fpp(gobj);
        if st::itIsHeavy((*fp).item_gobj) == 0 {
            match ext::itGetHoldKind((*fp).item_gobj) {
                3 => ext::ftAnim_80070FB4(gobj, 0, 0),
                4 => ext::ftAnim_80070FB4(gobj, 0, 1),
                _ => {}
            }
            if flag != 0 {
                st::ftAnim_80070C48(gobj, 0);
            }
        }
    }
}

pub fn ftPr_Init_OnItemInvisible(gobj: G) {
    melee_core::ct!("ftPr_Init_OnItemInvisible");
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).item_gobj.is_null() && st::itIsHeavy((*fp).item_gobj) == 0 {
            st::ftAnim_80070CC4(gobj, 0);
        }
    }
}

pub fn ftPr_Init_OnItemVisible(gobj: G) {
    melee_core::ct!("ftPr_Init_OnItemVisible");
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).item_gobj.is_null() && st::itIsHeavy((*fp).item_gobj) == 0 {
            st::ftAnim_80070C48(gobj, 0);
        }
    }
}

pub fn ftPr_Init_OnItemDrop(gobj: G, bool1: i32) {
    melee_core::ct!("ftPr_Init_OnItemDrop");
    Fighter_OnItemDrop(gobj, bool1, 0, 0);
}

/// `COPY_ATTRS(gobj, ftPurinAttributes)`.
pub fn ftPr_Init_LoadSpecialAttrs(gobj: G) {
    melee_core::ct!("ftPr_Init_LoadSpecialAttrs");
    unsafe {
        let fp = fpp(gobj);
        *((*fp).dat_attrs as *mut ftPurinAttributes) = *((*(*fp).ft_data).ext_attr as *const ftPurinAttributes);
    }
}

pub fn ftPr_Init_8013C94C(gobj: G) {
    melee_core::ct!("ftPr_Init_8013C94C");
    unsafe {
        let fp = fpp(gobj);
        if (*fp).x2219_b0() == 0 {
            ext::efSync_Spawn(1238, gobj, &[(*(*fp).parts.add(FtPart_WaistN as usize)).joint as usize]);
            (*fp).set_x2219_b0(1);
        }
        set_effect_hitlag_callbacks(fp);
        (*fp).accessory4_cb = None;
    }
}

/// `ftPr_CostumeList`: zero-initialised BSS array of the C (`UnkCostumeStruct ftPr_CostumeList[5]`), filled when the costumes load.
pub static mut ftPr_CostumeList: [melee_types::UnkCostumeStruct; 5] = unsafe { core::mem::zeroed() };

// ---- folded from chars (consolidation step 20/21) ----
/// `ftPr_Init_OnDeath` (src/melee/ft/kinds/ftPurin/ftpurin.c).
pub fn ftPr_Init_OnDeath(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftPr_Init_OnDeath");
    unsafe {
        st::ftParts_80074A4C(gobj, 0, 0);
    }
}

/// `ftPr_Init_OnKnockbackEnter` (src/melee/ft/kinds/ftPurin/ftpurin.c).
pub fn ftPr_Init_OnKnockbackEnter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftPr_Init_OnKnockbackEnter");
    unsafe {
        st::Fighter_OnKnockbackEnter(gobj, 1);
    }
}

/// `ftPr_Init_OnKnockbackExit` (src/melee/ft/kinds/ftPurin/ftpurin.c).
pub fn ftPr_Init_OnKnockbackExit(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftPr_Init_OnKnockbackExit");
    unsafe {
        st::Fighter_OnKnockbackExit(gobj, 1);
    }
}
