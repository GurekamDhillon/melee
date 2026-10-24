//! src/melee/ft/kinds/ftKirby/ftkirby.c: Kirby's init callbacks, the copy-ability hat loading
//! (`ftKb_SpecialN_800EF*` .. `ftKb_SpecialN_800F15D8`), the hat callback tables, and the m-ex
//! hooks `ftKb_MexKirbyLike` / `ftKb_MexCopyKindHat`. No fused sites (tools/rust-port/fma_sites.py:
//! this file has no float arithmetic beyond constant stores). Stateful only: smoke-tested by
//! tests/tu_ftkirby.rs through recording hooks.
//!
//! Native-pointer notes (the C sources assume 4-byte pointers): `*(HSD_DObj**)((u8*)dst + dst_off)`
//! with `dst_off` stepping by 4 is `dst[total_dobjs]`; `(u32) hat->hat_dynamics[1]` is a bit mask
//! stored in a pointer slot (`as usize as u32`); `((HSD_Archive**) &ft_80459B88)[kind]` is slot `kind`
//! of {x0, hats[0..]} (pointer-sized slots). The hat archive bytes behind `hat_dynamics[4]` (colour
//! words for Pichu) are copied as raw u32.
//!
//! Not ported here (data, no logic): `ftKb_Init_MotionStateTable` / `ftKb_Init_UnkMotionStates0`
//! (callback rows over TUs outside this shard), `ftKb_CostumeList`.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, clippy::missing_safety_doc)]

use crate::stubs as st;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;

/// Already ported (rounds 2, 5, 6): re-exported under this TU.
pub use crate::chars::ftKirby::ftkirby::ftKb_Init_UnkDemoCallbacks0;
pub use crate::chars::r5::ftKirby::ftkirby::{ftKb_Init_OnKnockbackEnter, ftKb_Init_OnKnockbackExit};
pub use crate::chars::r6::ftKirby::ftkirby::ftKb_Init_OnItemVisible;

/// `ftKirby_CostumeArchive` (ftKb_Init.static.h): a hat's joint tree and mat-anim tree.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ftKirby_CostumeArchive {
    pub joint: *mut HSD_Joint,
    pub matanim: *mut HSD_MatAnimJoint,
}
impl ftKirby_CostumeArchive {
    pub const NULL: Self = Self { joint: null_mut(), matanim: null_mut() };
}
melee_core::gc_layout!(ftKirby_CostumeArchive, "ftKirby_CostumeArchive", size 8, align 4, [
    joint @ 0x0: 4 "HSD_Joint *",
    matanim @ 0x4: 4 "HSD_MatAnimJoint *",
]);

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
                    None => panic!(concat!("TODO: ", stringify!($name), " is not ported")),
                }
            }
        };
    }
    // m-ex / skin lookups (pc/platform).
    hook!(SKINS_KIRBYROW, Skins_KirbyRow, (fk: i32, costume: i32) -> i32);
    hook!(MEX_COSTUMEVISIDX, Mex_CostumeVisIdx, (fk: i32, costume: i32) -> i32);
    hook!(MEX_INTERNALFORPORTKIND, Mex_InternalForPortKind, (fk: i32) -> i32);
    hook!(MEX_FTBASEKIND, Mex_FtBaseKind, (k: i32) -> i32);
    hook!(MEX_KIRBYFUNC, Mex_KirbyFunc, (slot: i32, internal: i32) -> *mut c_void);
    hook!(MEX_KIRBYCOSTUMES, Mex_KirbyCostumes, (internal: i32) -> *mut c_void);
    // pl/player.c
    pub static mut PLAYER_GETFLAGSBIT1: Option<fn(i32) -> i32> = None;
            #[allow(non_snake_case)]
            pub fn Player_GetFlagsBit1(slot: i32) -> i32 {
                match unsafe { PLAYER_GETFLAGSBIT1 } {
                    Some(f) => f(slot),
                    None => { return unsafe { melee_fighter::tu::player::Player_GetFlagsBit1(slot as _) as _ }; },
                }
            }
    pub static mut PLAYER_GETUNK4D: Option<fn(i32) -> i32> = None;
            #[allow(non_snake_case)]
            pub fn Player_GetUnk4D(slot: i32) -> i32 {
                match unsafe { PLAYER_GETUNK4D } {
                    Some(f) => f(slot),
                    None => { return unsafe { melee_fighter::tu::player::Player_GetUnk4D(slot as _) as _ }; },
                }
            }
    pub static mut PLAYER_GETFLAGSAEBIT1: Option<fn(i32) -> i32> = None;
            #[allow(non_snake_case)]
            pub fn Player_GetFlagsAEBit1(slot: i32) -> i32 {
                match unsafe { PLAYER_GETFLAGSAEBIT1 } {
                    Some(f) => f(slot),
                    None => { return unsafe { melee_fighter::tu::player::Player_GetFlagsAEBit1(slot as _) as _ }; },
                }
            }
    crate::port_hook!(PLAYER_80031DC8, Player_80031DC8, crate::adapt::Player_80031DC8, (cb: fn(i32, i32)));
    // baselib
    crate::port_hook!(HSD_RANDI, HSD_Randi, crate::adapt::HSD_Randi, (max: i32) -> i32);
    pub static mut HSD_IDINSERTTOTABLE: Option<fn(*mut c_void, *mut c_void, *mut HSD_JObj) > = None;
            #[allow(non_snake_case)]
            pub fn HSD_IDInsertToTable(table: *mut c_void, key: *mut c_void, value: *mut HSD_JObj)  {
                match unsafe { HSD_IDINSERTTOTABLE } {
                    Some(f) => f(table, key, value),
                    None => { unsafe { let _ = melee_runtime::tu::id::HSD_IDInsertToTable(table as _, key as _, value as _); } return; },
                }
            }
    crate::port_hook!(HSD_DOBJLOADDESC, HSD_DObjLoadDesc, crate::adapt::HSD_DObjLoadDesc, (desc: *mut HSD_DObjDesc) -> *mut HSD_DObj);

    crate::port_hook!(HSD_JOBJGETDOBJ, HSD_JObjGetDObj, crate::adapt::HSD_JObjGetDObj, (jobj: *mut HSD_JObj) -> *mut HSD_DObj);
    crate::port_hook!(HSD_DOBJRESOLVEREFSALL, HSD_DObjResolveRefsAll, crate::adapt::HSD_DObjResolveRefsAll, (dobj: *mut HSD_DObj, desc: *mut HSD_DObjDesc));

    crate::port_hook!(HSD_JOBJADDDOBJ, HSD_JObjAddDObj, crate::adapt::HSD_JObjAddDObj, (jobj: *mut HSD_JObj, dobj: *mut HSD_DObj));
    crate::port_hook!(HSD_DOBJADDANIMALL, HSD_DObjAddAnimAll, crate::adapt::HSD_DObjAddAnimAll, (dobj: *mut HSD_DObj, matanim: *mut c_void, aobj: *mut c_void));

    crate::port_hook!(HSD_DOBJREMOVEALL, HSD_DObjRemoveAll, crate::adapt::HSD_DObjRemoveAll, (dobj: *mut HSD_DObj));

    // `hsdChangeClass(mobj, &ftMObj)`.
    hook!(HSDCHANGECLASS_FTMOBJ, hsdChangeClass_ftMObj, (mobj: *mut HSD_MObj));
    pub static mut HSD_JOBJLOADJOINT: Option<fn(*mut HSD_Joint) -> *mut HSD_JObj> = None;
            #[allow(non_snake_case)]
            pub fn HSD_JObjLoadJoint(joint: *mut HSD_Joint) -> *mut HSD_JObj {
                match unsafe { HSD_JOBJLOADJOINT } {
                    Some(f) => f(joint),
                    None => { return unsafe { melee_runtime::tu::jobj::HSD_JObjLoadJoint(joint as _) as _ }; },
                }
            }
    crate::port_hook!(HSD_JOBJREMOVEALL, HSD_JObjRemoveAll, crate::adapt::HSD_JObjRemoveAll, (jobj: *mut HSD_JObj));
    crate::port_hook!(HSD_JOBJGETMTXPTR, HSD_JObjGetMtxPtr, crate::adapt::HSD_JObjGetMtxPtr, (jobj: *mut HSD_JObj) -> MtxPtr);
    crate::port_hook!(HSD_JOBJCOPYMTX, HSD_JObjCopyMtx, crate::adapt::HSD_JObjCopyMtx, (jobj: *mut HSD_JObj, mtx: MtxPtr));
    pub static mut HSD_JOBJSETMTXDIRTY: Option<fn(*mut HSD_JObj) > = None;
            #[allow(non_snake_case)]
            pub fn HSD_JObjSetMtxDirty(jobj: *mut HSD_JObj)  {
                match unsafe { HSD_JOBJSETMTXDIRTY } {
                    Some(f) => f(jobj),
                    None => { unsafe { let _ = crate::tu::ftinl::HSD_JObjSetMtxDirty(jobj as _); } return; },
                }
            }
    crate::port_hook!(HSD_JOBJDISPALL, HSD_JObjDispAll, crate::adapt::HSD_JObjDispAll, (jobj: *mut HSD_JObj, mtx: MtxPtr, rmode: u32, flags: u32));
    pub static mut HSD_GOBJ_80390EB8: Option<fn(i32) -> u32> = None;
            #[allow(non_snake_case)]
            pub fn HSD_GObj_80390EB8(arg: i32) -> u32 {
                match unsafe { HSD_GOBJ_80390EB8 } {
                    Some(f) => f(arg),
                    None => { return unsafe { melee_runtime::tu::gobj::HSD_GObj_80390EB8(arg as _) as _ }; },
                }
            }
    // `fighter_x2040_alloc_data` (the fighter's DObj-list pool).
    hook!(FIGHTER_X2040_ALLOC_DATA, fighter_x2040_alloc_data, () -> *mut c_void);
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
                    None => { unsafe { let _ = melee_runtime::tu::objalloc::HSD_ObjFree(data as _, obj as _); } return; },
                }
            }
    // lb / archives / effects
    pub static mut LB_8000CE30: Option<fn(*mut HSD_DObj, *mut HSD_DObj) > = None;
            #[allow(non_snake_case)]
            pub fn lb_8000CE30(a: *mut HSD_DObj, b: *mut HSD_DObj)  {
                match unsafe { LB_8000CE30 } {
                    Some(f) => f(a, b),
                    None => { unsafe { let _ = melee_runtime::tu::lb_00B0::lb_8000CE30(a as _, b as _); } return; },
                }
            }
    pub static mut LB_8000CE40: Option<fn(*mut HSD_JObj, *mut c_void) > = None;
            #[allow(non_snake_case)]
            pub fn lb_8000CE40(a: *mut HSD_JObj, b: *mut c_void)  {
                match unsafe { LB_8000CE40 } {
                    Some(f) => f(a, b),
                    None => { unsafe { let _ = melee_runtime::tu::lb_00B0::lb_8000CE40(a as _, b as _); } return; },
                }
            }
    crate::port_hook!(LBDVD_800178E8, lbDvd_800178E8, crate::adapt::lbDvd_800178E8, (a: i32, filename: *const i8, b: i32, c: i32, d: i32, e: i32, f: i32, g: i32, h: i32));

    // `lbArchive_80017040(NULL, filename, out, ...)`: the variadic tail without its 0 terminator.
    crate::port_hook!(LBARCHIVE_80017040, lbArchive_80017040, crate::adapt::lbArchive_80017040, (a: *mut c_void, filename: *const i8, out: *mut c_void, rest: &[*mut c_void]));
    hook!(EFASYNC_LOADASYNC, efAsync_LoadAsync, (id: i32));
    hook!(EFASYNC_LOADSYNC, efAsync_LoadSync, (id: i32));
    hook!(EFASYNC_SPAWN, efAsync_Spawn, (gobj: *mut HSD_GObj, head: *mut c_void, a: i32, id: i32, jobj: *mut HSD_JObj, pos: *mut Vec3));
    pub static mut IT_8026B3F8: Option<fn(*mut c_void, i32) > = None;
            #[allow(non_snake_case)]
            pub fn it_8026B3F8(article: *mut c_void, kind: i32)  {
                match unsafe { IT_8026B3F8 } {
                    Some(f) => f(article, kind),
                    None => { unsafe { let _ = melee_items::tu::it_26B1::it_8026B3F8(article as _, kind as _); } return; },
                }
            }
    pub static mut ITGETHOLDKIND: Option<fn(*mut Item_GObj) -> i32> = None;
            #[allow(non_snake_case)]
            pub fn itGetHoldKind(item: *mut Item_GObj) -> i32 {
                match unsafe { ITGETHOLDKIND } {
                    Some(f) => f(item),
                    None => { return unsafe { melee_items::tu::it_26B1::itGetHoldKind(item as _) as _ }; },
                }
            }
    // ft
    pub static mut FTANIM_80070FB4: Option<fn(*mut HSD_GObj, i32, i32) > = None;
            #[allow(non_snake_case)]
            pub fn ftAnim_80070FB4(gobj: *mut HSD_GObj, a: i32, b: i32)  {
                match unsafe { FTANIM_80070FB4 } {
                    Some(f) => f(gobj, a, b),
                    None => { unsafe { let _ = melee_fighter::ftanim::ftanim_80070fb4(gobj as _, a as _, b as _); } return; },
                }
            }
    hook!(FTANIM_GETNEXTJOINTINTREE, ftAnim_GetNextJointInTree, (joint: *mut *mut HSD_Joint, idx: *mut i32));
    hook!(FTANIM_GETNEXTMATANIMJOINTINTREE, ftAnim_GetNextMatAnimJointInTree, (joint: *mut *mut HSD_MatAnimJoint, idx: *mut i32));
    hook!(FTPARTSPOBJSETDEFAULTCLASS, ftPartsPObjSetDefaultClass, ());
    hook!(FTPARTSPOBJCLEARDEFAULTCLASS, ftPartsPObjClearDefaultClass, ());
    hook!(FTPARTS_800753D4, ftParts_800753D4, (fp: *mut Fighter, parts: *mut Fighter_804D6540_x0_t, joint: *mut HSD_Joint));
    hook!(FTPARTS_800755E8, ftParts_800755E8, (fp: *mut Fighter, part: *mut Fighter_804D6540_x0_t));
    hook!(FTPARTS_80075650, ftParts_80075650, (gobj: *mut HSD_GObj, jobj: *mut HSD_JObj, list: *mut DObjList));
    hook!(FTPARTS_8007487C, ftParts_8007487C, (desc: *mut FtPartsDesc, vis: *mut FtPartsVis, a: u32, l1: *mut DObjList, l2: *mut DObjList));
    // other Kirby TUs (copy-ability moves)
    pub static mut FTKB_SPECIALAIRLW_800F5524: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn ftKb_SpecialAirLw_800F5524(gobj: *mut HSD_GObj)  {
                match unsafe { FTKB_SPECIALAIRLW_800F5524 } {
                    Some(f) => f(gobj),
                    None => { unsafe { let _ = crate::tu::ftkirbyspecials::ftKb_SpecialAirLw_800F5524(gobj as _); } return; },
                }
            }
    pub static mut FTKB_ATTACKDASHAIR_800F22D4: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn ftKb_AttackDashAir_800F22D4(gobj: *mut HSD_GObj)  {
                match unsafe { FTKB_ATTACKDASHAIR_800F22D4 } {
                    Some(f) => f(gobj),
                    None => { unsafe { let _ = crate::tu::ftkirbyspecialhi::ftKb_AttackDashAir_800F22D4(gobj as _); } return; },
                }
            }
    pub static mut FTKB_SPECIALAIRLW_800F5318: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn ftKb_SpecialAirLw_800F5318(gobj: *mut HSD_GObj)  {
                match unsafe { FTKB_SPECIALAIRLW_800F5318 } {
                    Some(f) => f(gobj),
                    None => { unsafe { let _ = crate::tu::ftkirbyspeciallw::ftKb_SpecialAirLw_800F5318(gobj as _); } return; },
                }
            }
    pub static mut FTKB_SPECIALN_800F9090: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn ftKb_SpecialN_800F9090(gobj: *mut HSD_GObj)  {
                match unsafe { FTKB_SPECIALN_800F9090 } {
                    Some(f) => f(gobj),
                    None => { unsafe { let _ = crate::tu::ftkirbyspecialn::ftKb_SpecialN_800F9090(gobj as _); } return; },
                }
            }
    pub static mut FTKB_SPECIALN_800F5D04: Option<fn(*mut HSD_GObj, bool) > = None;
            #[allow(non_snake_case)]
            pub fn ftKb_SpecialN_800F5D04(gobj: *mut HSD_GObj, arg1: bool)  {
                match unsafe { FTKB_SPECIALN_800F5D04 } {
                    Some(f) => f(gobj, arg1),
                    None => { unsafe { let _ = crate::tu::ftkirbyspecialn::ftKb_SpecialN_800F5D04(gobj as _, arg1 as _); } return; },
                }
            }
    pub static mut FTKB_SPECIALN_800F5F68: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn ftKb_SpecialN_800F5F68(gobj: *mut HSD_GObj)  {
                match unsafe { FTKB_SPECIALN_800F5F68 } {
                    Some(f) => f(gobj),
                    None => { unsafe { let _ = crate::tu::ftkirbyspecialn::ftKb_SpecialN_800F5F68(gobj as _); } return; },
                }
            }
    pub static mut FTKB_SPECIALN_800F6070: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn ftKb_SpecialN_800F6070(gobj: *mut HSD_GObj)  {
                match unsafe { FTKB_SPECIALN_800F6070 } {
                    Some(f) => f(gobj),
                    None => { unsafe { let _ = crate::tu::ftkirbyspecialn::ftKb_SpecialN_800F6070(gobj as _); } return; },
                }
            }
    pub static mut FTKB_SPECIALNIC_80108D04: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn ftKb_SpecialNIc_80108D04(gobj: *mut HSD_GObj)  {
                match unsafe { FTKB_SPECIALNIC_80108D04 } {
                    Some(f) => f(gobj),
                    None => { unsafe { let _ = crate::tu::ftkirbyspecialiceclimber::ftKb_SpecialNIc_80108D04(gobj as _); } return; },
                }
            }
    pub static mut FTKB_SPECIALNPE_8010C3F4: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn ftKb_SpecialNPe_8010C3F4(gobj: *mut HSD_GObj)  {
                match unsafe { FTKB_SPECIALNPE_8010C3F4 } {
                    Some(f) => f(gobj),
                    None => { unsafe { let _ = crate::tu::ftkirbyspecialpeach::ftKb_SpecialNPe_8010C3F4(gobj as _); } return; },
                }
            }
    pub static mut FTKB_SPECIALNLK800FB800: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn ftKb_SpecialNLk800FB800(gobj: *mut HSD_GObj)  {
                match unsafe { FTKB_SPECIALNLK800FB800 } {
                    Some(f) => f(gobj),
                    None => { unsafe { let _ = crate::tu::ftkirbyspeciallink::ftKb_SpecialNLk800FB800(gobj as _); } return; },
                }
            }
    pub static mut FTKB_SPECIALNLK800FB840: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn ftKb_SpecialNLk800FB840(gobj: *mut HSD_GObj)  {
                match unsafe { FTKB_SPECIALNLK800FB840 } {
                    Some(f) => f(gobj),
                    None => { unsafe { let _ = crate::tu::ftkirbyspeciallink::ftKb_SpecialNLk800FB840(gobj as _); } return; },
                }
            }
    pub static mut FTKB_SPECIALNMT_8010709C: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn ftKb_SpecialNMt_8010709C(gobj: *mut HSD_GObj)  {
                match unsafe { FTKB_SPECIALNMT_8010709C } {
                    Some(f) => f(gobj),
                    None => { unsafe { let _ = crate::tu::ftkirbyspecialmewtwo::ftKb_SpecialNMt_8010709C(gobj as _); } return; },
                }
            }
    pub static mut FTKB_SPECIALNMT_80107130: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn ftKb_SpecialNMt_80107130(gobj: *mut HSD_GObj)  {
                match unsafe { FTKB_SPECIALNMT_80107130 } {
                    Some(f) => f(gobj),
                    None => { unsafe { let _ = crate::tu::ftkirbyspecialmewtwo::ftKb_SpecialNMt_80107130(gobj as _); } return; },
                }
            }
    pub static mut FTKB_SPECIALNSS_800FCD60: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn ftKb_SpecialNSs_800FCD60(gobj: *mut HSD_GObj)  {
                match unsafe { FTKB_SPECIALNSS_800FCD60 } {
                    Some(f) => f(gobj),
                    None => { unsafe { let _ = crate::tu::ftkirbyspecialsamus::ftKb_SpecialNSs_800FCD60(gobj as _); } return; },
                }
            }
    pub static mut FTKB_SPECIALNSK_8010603C: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn ftKb_SpecialNSk_8010603C(gobj: *mut HSD_GObj)  {
                match unsafe { FTKB_SPECIALNSK_8010603C } {
                    Some(f) => f(gobj),
                    None => { unsafe { let _ = crate::tu::ftkirbyspecialseak::ftKb_SpecialNSk_8010603C(gobj as _); } return; },
                }
            }
    pub static mut FTKB_SPECIALNYS_801093B4: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn ftKb_SpecialNYs_801093B4(gobj: *mut HSD_GObj)  {
                match unsafe { FTKB_SPECIALNYS_801093B4 } {
                    Some(f) => f(gobj),
                    None => { unsafe { let _ = crate::tu::ftkirbyspecialyoshi::ftKb_SpecialNYs_801093B4(gobj as _); } return; },
                }
            }
    pub static mut FTKB_SPECIALNYS_8010A8BC: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn ftKb_SpecialNYs_8010A8BC(gobj: *mut HSD_GObj)  {
                match unsafe { FTKB_SPECIALNYS_8010A8BC } {
                    Some(f) => f(gobj),
                    None => { unsafe { let _ = crate::tu::ftkirbyspecialyoshi::ftKb_SpecialNYs_8010A8BC(gobj as _); } return; },
                }
            }
    hook!(FTKB_SPECIALNPR_80100DE0, ftKb_SpecialNPr_80100DE0, (gobj: *mut HSD_GObj));

    pub static mut FTKB_SPECIALNKP_800FA7D4: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn ftKb_SpecialNKp_800FA7D4(gobj: *mut HSD_GObj)  {
                match unsafe { FTKB_SPECIALNKP_800FA7D4 } {
                    Some(f) => f(gobj),
                    None => { unsafe { let _ = crate::tu::ftkirbyspecialkoopa::ftKb_SpecialNKp_800FA7D4(gobj as _); } return; },
                }
            }

    /// Like `hook!`, but an unset hook falls back to group 3's port instead of panicking.
    macro_rules! hook_or {
        ($hook:ident, $name:ident, $port:path, ($($a:ident: $t:ty),*) $(-> $r:ty)?) => {
            pub static mut $hook: Option<fn($($t),*) $(-> $r)?> = None;
            #[allow(non_snake_case)]
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $hook } {
                    Some(f) => f($($a),*),
                    None => unsafe { $port($($a),*) },
                }
            }
        };
    }
    hook_or!(FIGHTER_UPDATEMODELSCALE, Fighter_UpdateModelScale, melee_fighter::tu::fighter::Fighter_UpdateModelScale, (gobj: *mut HSD_GObj));
    hook_or!(FTPARTS_80074CA0, ftParts_80074CA0, port_74ca0, (vis: *mut FtPartsVis, idx: i32, list: *const DObjList));
    hook_or!(FTPARTS_80074D7C, ftParts_80074D7C, port_74d7c, (vis: *mut FtPartsVis, idx: i32, list: *const DObjList));
    hook_or!(FTANIM_800705E0, ftAnim_800705E0, port_705e0, (list: *mut CostumeTObjList));
    hook_or!(FTANIM_80070458, ftAnim_80070458, port_70458, (fp: *const Fighter, list: *mut CostumeTObjList, idx: u32, frame: f32));
    hook_or!(FTANIM_80070200, ftAnim_80070200, port_70200, (fp: *mut Fighter, vis: *mut ftData_x8_x8, list: *mut CostumeTObjList, dobjs: *mut DObjList));
    unsafe fn port_74ca0(vis: *mut FtPartsVis, idx: i32, list: *const DObjList) {
        melee_fighter::tu::ftparts::ftParts_80074CA0(&mut *vis, idx, list)
    }
    unsafe fn port_74d7c(vis: *mut FtPartsVis, idx: i32, list: *const DObjList) {
        melee_fighter::tu::ftparts::ftParts_80074D7C(&mut *vis, idx, list)
    }
    unsafe fn port_705e0(list: *mut CostumeTObjList) {
        melee_fighter::tu::ftanim::ftAnim_800705E0(&mut *list)
    }
    unsafe fn port_70458(fp: *const Fighter, list: *mut CostumeTObjList, idx: u32, frame: f32) {
        melee_fighter::tu::ftanim::ftAnim_80070458(&*fp, &mut *list, idx, frame)
    }
    unsafe fn port_70200(fp: *mut Fighter, vis: *mut ftData_x8_x8, list: *mut CostumeTObjList, dobjs: *mut DObjList) {
        melee_fighter::tu::ftanim::ftAnim_80070200(&mut *fp, vis, list, dobjs)
    }

    /// `ftCo_800BFFD0(fp, a, b)` (colanim start): group 3's port when the hook is unset.
    pub static mut FTCO_800BFFD0: Option<fn(*mut melee_types::Fighter, i32, i32)> = None;
    #[allow(non_snake_case)]
    pub fn ftCo_800BFFD0(fp: *mut melee_types::Fighter, a: i32, b: i32) {
        match unsafe { FTCO_800BFFD0 } {
            Some(f) => f(fp, a, b),
            None => unsafe {
                melee_fighter::tu::ftcolanim::ftco_800bffd0(&mut *fp, a, b);
            },
        }
    }
}

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

/// `ftKb_SpecialN_set_cbs`-style helper for the death callbacks stored by this file.
fn cb_death3() -> HSD_GObjEvent {
    Some(crate::chars::cb::ftkirby__ftKb_Init_800EE74C)
}
fn cb_death1() -> HSD_GObjEvent {
    Some(crate::chars::cb::ftkirby__ftKb_Init_800EE7B8)
}

/// `FTKB_COPY_COSTUMES`: Kirby's retail costume count, the size of every per-costume hat row.
pub const FTKB_COPY_COSTUMES: i32 = 6;

/// `ftKb_CopyCostumeRow(costume)` (TARGET_PC): which retail row of a per-costume Kirby copy-hat table a
/// Kirby costume uses (a skin's declared hat row, else m-ex's visibility lookup index, else row 0).
pub fn ftKb_CopyCostumeRow(costume: i32) -> i32 {
    let mut row = ext::Skins_KirbyRow(Ft_Kind_Kirby as i32, costume);
    if row >= 0 && row < FTKB_COPY_COSTUMES {
        return row;
    }
    row = ext::Mex_CostumeVisIdx(Ft_Kind_Kirby as i32, costume);
    if row >= 0 && row < FTKB_COPY_COSTUMES {
        row
    } else {
        0
    }
}

/// `FTKB_COPY_ROW(fp)`.
#[inline(always)]
unsafe fn copy_row(fp: *mut Fighter) -> usize {
    ftKb_CopyCostumeRow((*fp).x619_costume_id as i32) as usize
}

/// `ft_80459B88`: the loaded hat archives (slot 0 is `x0`, slot k+1 is `hats[k]`).
pub static mut ft_80459B88: ft_80459B88_t = unsafe { core::mem::zeroed() };

/// Slot `k` of `ft_80459B88` viewed as an array of pointers (`((HSD_Archive**) &ft_80459B88)[k]`).
#[inline(always)]
unsafe fn hat_slot(k: usize) -> *mut *mut c_void {
    (&raw mut ft_80459B88 as *mut *mut c_void).add(k)
}

/// `ftKb_hatTable` (`ft_80459B88.hats`, an array: never NULL).
#[inline(always)]
unsafe fn hat_table(kind: usize) -> *mut KirbyHatStruct {
    ft_80459B88.hats[kind]
}

/// `ftKb_Init_803B7548`: ten zero floats.
pub static ftKb_Init_803B7548: [f32; 10] = [0.0; 10];

/// `extern "C"` entries stored in the hat tables below: each calls the Rust port of its target.
pub mod cb {
    use melee_types::HSD_GObj;
    macro_rules! entry {
        ($name:ident, $target:path) => {
            pub unsafe extern "C" fn $name(g: *mut HSD_GObj) {
                $target(g)
            }
        };
    }
    entry!(ftKb_SpecialN_800EFA40, super::ftKb_SpecialN_800EFA40);
    entry!(ftKb_SpecialN_800EFAF0, super::ftKb_SpecialN_800EFAF0);
    entry!(ftKb_SpecialN_800EFB4C, super::ftKb_SpecialN_800EFB4C);
    entry!(ftKb_SpecialN_800EFBFC, super::ftKb_SpecialN_800EFBFC);
    entry!(ftKb_SpecialN_800EFC58, super::ftKb_SpecialN_800EFC58);
    entry!(ftKb_SpecialN_800EFD08, super::ftKb_SpecialN_800EFD08);
    entry!(ftKb_SpecialN_800F0FC0, super::ftKb_SpecialN_800F0FC0);
    entry!(ftKb_SpecialN_800F10A4, super::ftKb_SpecialN_800F10A4);
    entry!(ftKb_SpecialN_800EFD64, super::ftKb_SpecialN_800EFD64);
    entry!(ftKb_SpecialN_800EFE1C, super::ftKb_SpecialN_800EFE1C);
    entry!(ftKb_SpecialN_800EFE80, super::ftKb_SpecialN_800EFE80);
    entry!(ftKb_SpecialN_800EFF38, super::ftKb_SpecialN_800EFF38);
    entry!(ftKb_SpecialN_800EFF9C, super::ftKb_SpecialN_800EFF9C);
    entry!(ftKb_SpecialN_800F0054, super::ftKb_SpecialN_800F0054);
    entry!(ftKb_SpecialN_800F00B8, super::ftKb_SpecialN_800F00B8);
    entry!(ftKb_SpecialN_800F0168, super::ftKb_SpecialN_800F0168);
    entry!(ftKb_SpecialN_800F01C4, super::ftKb_SpecialN_800F01C4);
    entry!(ftKb_SpecialN_800F0274, super::ftKb_SpecialN_800F0274);
    entry!(ftKb_SpecialN_800F02D0, super::ftKb_SpecialN_800F02D0);
    entry!(ftKb_SpecialN_800F0380, super::ftKb_SpecialN_800F0380);
    entry!(ftKb_SpecialN_800F03DC, super::ftKb_SpecialN_800F03DC);
    entry!(ftKb_SpecialN_800F0494, super::ftKb_SpecialN_800F0494);
    entry!(ftKb_SpecialN_800F04F8, super::ftKb_SpecialN_800F04F8);
    entry!(ftKb_SpecialN_800F05A8, super::ftKb_SpecialN_800F05A8);
    entry!(ftKb_SpecialN_800F0604, super::ftKb_SpecialN_800F0604);
    entry!(ftKb_SpecialN_800F06B4, super::ftKb_SpecialN_800F06B4);
    entry!(ftKb_SpecialN_800F10D4, super::ftKb_SpecialN_800F10D4);
    entry!(ftKb_SpecialN_800F11AC, super::ftKb_SpecialN_800F11AC);
    entry!(ftKb_SpecialN_800F11F0, super::ftKb_SpecialN_800F11F0);
    entry!(ftKb_SpecialN_800F12C8, super::ftKb_SpecialN_800F12C8);
    entry!(ftKb_SpecialN_800F0710, super::ftKb_SpecialN_800F0710);
    entry!(ftKb_SpecialN_800F07C0, super::ftKb_SpecialN_800F07C0);
    entry!(ftKb_SpecialN_800F081C, super::ftKb_SpecialN_800F081C);
    entry!(ftKb_SpecialN_800F08D4, super::ftKb_SpecialN_800F08D4);
    entry!(ftKb_SpecialN_800F0938, super::ftKb_SpecialN_800F0938);
    entry!(ftKb_SpecialN_800F09F0, super::ftKb_SpecialN_800F09F0);
    entry!(ftKb_SpecialN_800F0A54, super::ftKb_SpecialN_800F0A54);
    entry!(ftKb_SpecialN_800F0B0C, super::ftKb_SpecialN_800F0B0C);
    entry!(ftKb_SpecialN_800F0B70, super::ftKb_SpecialN_800F0B70);
    entry!(ftKb_SpecialN_800F0C20, super::ftKb_SpecialN_800F0C20);
    entry!(ftKb_SpecialN_800F130C, super::ftKb_SpecialN_800F130C);
    entry!(ftKb_SpecialN_800F13F0, super::ftKb_SpecialN_800F13F0);
    entry!(ftKb_SpecialN_800F0C7C, super::ftKb_SpecialN_800F0C7C);
    entry!(ftKb_SpecialN_800F0D34, super::ftKb_SpecialN_800F0D34);
    entry!(ftKb_SpecialN_800F14B4, super::ftKb_SpecialN_800F14B4);
    entry!(ftKb_SpecialN_800F15D8, super::ftKb_SpecialN_800F15D8);
    entry!(ftKb_SpecialN_800F0D98, super::ftKb_SpecialN_800F0D98);
    entry!(ftKb_SpecialN_800F0E48, super::ftKb_SpecialN_800F0E48);
    entry!(ftKb_SpecialN_800F0EA4, super::ftKb_SpecialN_800F0EA4);
    entry!(ftKb_SpecialN_800F0F5C, super::ftKb_SpecialN_800F0F5C);
    entry!(ftKb_SpecialNFx_800FE100, crate::tu::ftkirbyspecialfox::ftKb_SpecialNFx_800FE100);
    entry!(ftKb_SpecialNCa_800F99BC, crate::tu::ftkirbyspecialcaptain::ftKb_SpecialNCa_800F99BC);
    entry!(ftKb_SpecialNDk_800FF8EC, crate::tu::ftkirbyspecialdonkey::ftKb_SpecialNDk_800FF8EC);
    entry!(ftKb_SpecialNSs_800FCF74, crate::chars::r11::ftKirby::ftkirbyspecialsamus::ftKb_SpecialNSs_800FCF74);
    entry!(ftKb_SpecialNLg_800F9614, crate::chars::r7::ftKirby::ftkirbyspecialluigi::ftKb_SpecialNLg_800F9614);
    entry!(ftKb_SpecialNGw_8010D188, crate::tu::ftkirbyspecialgamewatch::ftKb_SpecialNGw_8010D188);
    entry!(ftKb_SpecialNFx_800FE240, crate::tu::ftkirbyspecialfox::ftKb_SpecialNFx_800FE240);
    entry!(ftKb_SpecialNCa_800F9A54, crate::tu::ftkirbyspecialcaptain::ftKb_SpecialNCa_800F9A54);
    entry!(ftKb_SpecialNDk_800FFA10, crate::tu::ftkirbyspecialdonkey::ftKb_SpecialNDk_800FFA10);
    entry!(ftKb_SpecialNSs_800FD020, crate::chars::r11::ftKirby::ftkirbyspecialsamus::ftKb_SpecialNSs_800FD020);
    entry!(ftKb_SpecialNLg_800F9684, crate::chars::r7::ftKirby::ftkirbyspecialluigi::ftKb_SpecialNLg_800F9684);
    entry!(ftKb_SpecialNGw_8010D204, crate::tu::ftkirbyspecialgamewatch::ftKb_SpecialNGw_8010D204);
}

/// Table entries for the per-character copy-ability TUs (each calls the Rust port of its target).
pub mod ext_cb {
    use melee_types::HSD_GObj;
    macro_rules! entry {
        ($name:ident, $target:path) => {
            #[allow(non_snake_case)]
            pub unsafe extern "C" fn $name(g: *mut HSD_GObj) {
                $target(g)
            }
        };
    }
    entry!(ftKb_SpecialN_800F9110, crate::tu::ftkirbyspecialmario::ftKb_SpecialN_800F9110);
    entry!(ftKb_SpecialN_800F5F68, crate::tu::ftkirbyspecialn::ftKb_SpecialN_800F5F68);
    entry!(ftKb_SpecialNKp_800FA8B4, crate::tu::ftkirbyspecialkoopa::ftKb_SpecialNKp_800FA8B4);
    entry!(ftKb_SpecialNLk800FB880, crate::tu::ftkirbyspeciallink::ftKb_SpecialNLk800FB880);
    entry!(ftKb_SpecialNSk_8010612C, crate::tu::ftkirbyspecialseak::ftKb_SpecialNSk_8010612C);
    entry!(ftKb_SpecialNNs_800FED38, crate::tu::ftkirbyspecialness::ftKb_SpecialNNs_800FED38);
    entry!(ftKb_SpecialNPe_8010C4D4, crate::tu::ftkirbyspecialpeach::ftKb_SpecialNPe_8010C4D4);
    entry!(ftKb_SpecialNIc_80108D64, crate::tu::ftkirbyspecialiceclimber::ftKb_SpecialNIc_80108D64);
    entry!(ftKb_SpecialNPk_800F9FD4, crate::tu::ftkirbyspecialpikachu::ftKb_SpecialNPk_800F9FD4);
    entry!(ftKb_SpecialNYs_8010941C, crate::tu::ftkirbyspecialyoshi::ftKb_SpecialNYs_8010941C);
    entry!(ftKb_SpecialNPr_80101560, crate::tu::ftkirbyspecialpurin::ftKb_SpecialNPr_80101560);
    entry!(ftKb_SpecialNMt_80107568, crate::tu::ftkirbyspecialmewtwo::ftKb_SpecialNMt_80107568);
    entry!(ftKb_SpecialNMs_8010B2FC, crate::tu::ftkirbyspecialmars::ftKb_SpecialNMs_8010B2FC);
    entry!(ftKb_SpecialNZd_80105B2C, crate::tu::ftkirbyspecialzelda::ftKb_SpecialNZd_80105B2C);
    entry!(ftKb_SpecialNMr_800F93CC, crate::tu::ftkirbyspecialmario::ftKb_SpecialNMr_800F93CC);
    entry!(ftKb_SpecialN_800F6070, crate::tu::ftkirbyspecialn::ftKb_SpecialN_800F6070);
    entry!(ftKb_SpecialNKp_800FA958, crate::tu::ftkirbyspecialkoopa::ftKb_SpecialNKp_800FA958);
    entry!(ftKb_SpecialNLk800FBA00, crate::tu::ftkirbyspeciallink::ftKb_SpecialNLk800FBA00);
    entry!(ftKb_SpecialNSk_801061E4, crate::tu::ftkirbyspecialseak::ftKb_SpecialNSk_801061E4);
    entry!(ftKb_SpecialNNs_800FEDD0, crate::tu::ftkirbyspecialness::ftKb_SpecialNNs_800FEDD0);
    entry!(ftKb_SpecialNPe_8010C560, crate::tu::ftkirbyspecialpeach::ftKb_SpecialNPe_8010C560);
    entry!(ftKb_SpecialNIc_80108E14, crate::tu::ftkirbyspecialiceclimber::ftKb_SpecialNIc_80108E14);
    entry!(ftKb_SpecialNPk_800FA064, crate::tu::ftkirbyspecialpikachu::ftKb_SpecialNPk_800FA064);
    entry!(ftKb_SpecialNYs_801094FC, crate::tu::ftkirbyspecialyoshi::ftKb_SpecialNYs_801094FC);
    entry!(ftKb_SpecialNPr_80101618, crate::tu::ftkirbyspecialpurin::ftKb_SpecialNPr_80101618);
    entry!(ftKb_SpecialNMt_80107638, crate::tu::ftkirbyspecialmewtwo::ftKb_SpecialNMt_80107638);
    entry!(ftKb_SpecialNMs_8010B4A0, crate::tu::ftkirbyspecialmars::ftKb_SpecialNMs_8010B4A0);
    entry!(ftKb_SpecialNZd_80105BA8, crate::tu::ftkirbyspecialzelda::ftKb_SpecialNZd_80105BA8);
}

/// `ftKb_Init_803C9CC8`: [kind * 2] = load hat, [kind * 2 + 1] = unload hat (m-ex MexKirbyFunction.OnLoad/OnDestroy).
pub static mut ftKb_Init_803C9CC8: [HSD_GObjEvent; 254] = [
    Some(cb::ftKb_SpecialN_800EFA40),
    Some(cb::ftKb_SpecialN_800EFAF0),
    Some(cb::ftKb_SpecialN_800EFB4C),
    Some(cb::ftKb_SpecialN_800EFBFC),
    Some(cb::ftKb_SpecialN_800EFC58),
    Some(cb::ftKb_SpecialN_800EFD08),
    Some(cb::ftKb_SpecialN_800F0FC0),
    Some(cb::ftKb_SpecialN_800F10A4),
    None,
    None,
    Some(cb::ftKb_SpecialN_800EFD64),
    Some(cb::ftKb_SpecialN_800EFE1C),
    Some(cb::ftKb_SpecialN_800EFE80),
    Some(cb::ftKb_SpecialN_800EFF38),
    Some(cb::ftKb_SpecialN_800EFF9C),
    Some(cb::ftKb_SpecialN_800F0054),
    Some(cb::ftKb_SpecialN_800F00B8),
    Some(cb::ftKb_SpecialN_800F0168),
    Some(cb::ftKb_SpecialN_800F01C4),
    Some(cb::ftKb_SpecialN_800F0274),
    Some(cb::ftKb_SpecialN_800F02D0),
    Some(cb::ftKb_SpecialN_800F0380),
    None,
    None,
    Some(cb::ftKb_SpecialN_800F03DC),
    Some(cb::ftKb_SpecialN_800F0494),
    Some(cb::ftKb_SpecialN_800F04F8),
    Some(cb::ftKb_SpecialN_800F05A8),
    Some(cb::ftKb_SpecialN_800F0604),
    Some(cb::ftKb_SpecialN_800F06B4),
    Some(cb::ftKb_SpecialN_800F10D4),
    Some(cb::ftKb_SpecialN_800F11AC),
    Some(cb::ftKb_SpecialN_800F11F0),
    Some(cb::ftKb_SpecialN_800F12C8),
    Some(cb::ftKb_SpecialN_800F0710),
    Some(cb::ftKb_SpecialN_800F07C0),
    Some(cb::ftKb_SpecialN_800F081C),
    Some(cb::ftKb_SpecialN_800F08D4),
    Some(cb::ftKb_SpecialN_800F0938),
    Some(cb::ftKb_SpecialN_800F09F0),
    Some(cb::ftKb_SpecialN_800F0A54),
    Some(cb::ftKb_SpecialN_800F0B0C),
    Some(cb::ftKb_SpecialN_800F0B70),
    Some(cb::ftKb_SpecialN_800F0C20),
    Some(cb::ftKb_SpecialN_800F130C),
    Some(cb::ftKb_SpecialN_800F13F0),
    Some(cb::ftKb_SpecialN_800F0C7C),
    Some(cb::ftKb_SpecialN_800F0D34),
    Some(cb::ftKb_SpecialN_800F14B4),
    Some(cb::ftKb_SpecialN_800F15D8),
    Some(cb::ftKb_SpecialN_800F0D98),
    Some(cb::ftKb_SpecialN_800F0E48),
    Some(cb::ftKb_SpecialN_800F0EA4),
    Some(cb::ftKb_SpecialN_800F0F5C),
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    Some(cb::ftKb_SpecialN_800EFD64),
    Some(cb::ftKb_SpecialN_800EFE1C),
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
];

/// `ftKb_Init_803C9DD0`: Kirby's neutral B while wearing kind K's hat (m-ex OnSpecialN). NULL = Kirby's own inhale.
pub static mut ftKb_Init_803C9DD0: [HSD_GObjEvent; 127] = [
    Some(ext_cb::ftKb_SpecialN_800F9110),
    Some(cb::ftKb_SpecialNFx_800FE100),
    Some(cb::ftKb_SpecialNCa_800F99BC),
    Some(cb::ftKb_SpecialNDk_800FF8EC),
    Some(ext_cb::ftKb_SpecialN_800F5F68),
    Some(ext_cb::ftKb_SpecialNKp_800FA8B4),
    Some(ext_cb::ftKb_SpecialNLk800FB880),
    Some(ext_cb::ftKb_SpecialNSk_8010612C),
    Some(ext_cb::ftKb_SpecialNNs_800FED38),
    Some(ext_cb::ftKb_SpecialNPe_8010C4D4),
    Some(ext_cb::ftKb_SpecialNIc_80108D64),
    None,
    Some(ext_cb::ftKb_SpecialNPk_800F9FD4),
    Some(cb::ftKb_SpecialNSs_800FCF74),
    Some(ext_cb::ftKb_SpecialNYs_8010941C),
    Some(ext_cb::ftKb_SpecialNPr_80101560),
    Some(ext_cb::ftKb_SpecialNMt_80107568),
    Some(cb::ftKb_SpecialNLg_800F9614),
    Some(ext_cb::ftKb_SpecialNMs_8010B2FC),
    Some(ext_cb::ftKb_SpecialNZd_80105B2C),
    Some(ext_cb::ftKb_SpecialNLk800FB880),
    Some(ext_cb::ftKb_SpecialN_800F9110),
    Some(cb::ftKb_SpecialNFx_800FE100),
    Some(ext_cb::ftKb_SpecialNPk_800F9FD4),
    Some(cb::ftKb_SpecialNGw_8010D188),
    Some(cb::ftKb_SpecialNCa_800F99BC),
    Some(ext_cb::ftKb_SpecialNMs_8010B2FC),
    None,
    None,
    None,
    None,
    Some(ext_cb::ftKb_SpecialNKp_800FA8B4),
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
];

/// `ftKb_Init_803C9E54`: the aerial half of the above (m-ex OnSpecialAirN).
pub static mut ftKb_Init_803C9E54: [HSD_GObjEvent; 127] = [
    Some(ext_cb::ftKb_SpecialNMr_800F93CC),
    Some(cb::ftKb_SpecialNFx_800FE240),
    Some(cb::ftKb_SpecialNCa_800F9A54),
    Some(cb::ftKb_SpecialNDk_800FFA10),
    Some(ext_cb::ftKb_SpecialN_800F6070),
    Some(ext_cb::ftKb_SpecialNKp_800FA958),
    Some(ext_cb::ftKb_SpecialNLk800FBA00),
    Some(ext_cb::ftKb_SpecialNSk_801061E4),
    Some(ext_cb::ftKb_SpecialNNs_800FEDD0),
    Some(ext_cb::ftKb_SpecialNPe_8010C560),
    Some(ext_cb::ftKb_SpecialNIc_80108E14),
    None,
    Some(ext_cb::ftKb_SpecialNPk_800FA064),
    Some(cb::ftKb_SpecialNSs_800FD020),
    Some(ext_cb::ftKb_SpecialNYs_801094FC),
    Some(ext_cb::ftKb_SpecialNPr_80101618),
    Some(ext_cb::ftKb_SpecialNMt_80107638),
    Some(cb::ftKb_SpecialNLg_800F9684),
    Some(ext_cb::ftKb_SpecialNMs_8010B4A0),
    Some(ext_cb::ftKb_SpecialNZd_80105BA8),
    Some(ext_cb::ftKb_SpecialNLk800FBA00),
    Some(ext_cb::ftKb_SpecialNMr_800F93CC),
    Some(cb::ftKb_SpecialNFx_800FE240),
    Some(ext_cb::ftKb_SpecialNPk_800FA064),
    Some(cb::ftKb_SpecialNGw_8010D204),
    Some(cb::ftKb_SpecialNCa_800F9A54),
    Some(ext_cb::ftKb_SpecialNMs_8010B4A0),
    None,
    None,
    None,
    None,
    Some(ext_cb::ftKb_SpecialNKp_800FA958),
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
];

pub static mut ftKb_Init_803C9ED8: [ftKirby_CostumeArchive; 6] = [ftKirby_CostumeArchive::NULL; 6];
pub static mut ftKb_Init_803C9F08: [ftKirby_CostumeArchive; 6] = [ftKirby_CostumeArchive::NULL; 6];
pub static mut ftKb_Init_803C9F38: [ftKirby_CostumeArchive; 6] = [ftKirby_CostumeArchive::NULL; 6];
pub static mut ftKb_Init_803C9F68: [ftKirby_CostumeArchive; 6] = [ftKirby_CostumeArchive::NULL; 6];
pub static mut ftKb_Init_803C9F98: [ftKirby_CostumeArchive; 6] = [ftKirby_CostumeArchive::NULL; 6];

/// `ftKb_Init_803C9FC8`: per-kind rows of six costume archives (hat joint + matanim), or NULL.
pub static mut ftKb_Init_803C9FC8: [*mut ftKirby_CostumeArchive; 127] = [
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    (&raw mut ftKb_Init_803C9ED8) as *mut ftKirby_CostumeArchive,
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    (&raw mut ftKb_Init_803C9F08) as *mut ftKirby_CostumeArchive,
    (&raw mut ftKb_Init_803C9F38) as *mut ftKirby_CostumeArchive,
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    (&raw mut ftKb_Init_803C9F68) as *mut ftKirby_CostumeArchive,
    core::ptr::null_mut(),
    (&raw mut ftKb_Init_803C9F98) as *mut ftKirby_CostumeArchive,
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
    core::ptr::null_mut(),
];


use super::ftkirbydata::{ftKb_Init_803CA9D0, ftKb_Init_803CB3E8, ftKb_Init_803CB46C};

/// `ftKb_Init_800EE528`: forget every loaded hat archive and clear the per-costume archive rows.
pub fn ftKb_Init_800EE528() {
    unsafe {
        // C walks `ft_80459B88` as s32[Ft_Kind_Max]: slot 0 is x0, slot i is hats[i - 1].
        for i in 0..Ft_Kind_Max as usize {
            *hat_slot(i) = null_mut();
            let unk_struct = ftKb_Init_803C9FC8[i];
            if !unk_struct.is_null() {
                for c in 0..6 {
                    (*unk_struct.add(c)).joint = null_mut();
                    (*unk_struct.add(c)).matanim = null_mut();
                }
            }
        }
    }
}

/// `ftKb_Init_OnDeath`: drop the copied hat and, if the player kept one, re-apply it.
pub fn ftKb_Init_OnDeath(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        st::ftParts_80074A4C(gobj, 0, 0);
        st::ftParts_80074A4C(gobj, 1, 0);
        (*fp).u.kb.hat.x0 = null_mut();
        (*fp).u.kb.hat.x4 = ext::HSD_Randi(5) + 1;
        (*fp).u.kb.hat.jobj = null_mut();
        (*fp).u.kb.hat.kind = Ft_Kind_Kirby;
        (*fp).u.kb.hat.x14.data = null_mut();
        (*fp).u.kb.x60 = 0;
        (*fp).u.kb.x64 = 0;
        if ext::Player_GetFlagsBit1((*fp).player_id as i32) != 0 && ext::Player_GetUnk4D((*fp).player_id as i32) != 4 {
            ftKb_SpecialN_800F1BAC(gobj, ext::Player_GetUnk4D((*fp).player_id as i32), false);
        }
    }
}

/// `ftKb_Init_OnLoad`: `PUSH_ATTRS(fp, ftKb_DatAttrs)`, multijump, and the four copy-ability articles.
pub fn ftKb_Init_OnLoad(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let item_list = (*(*fp).ft_data).x48_items;
        // PUSH_ATTRS(fp, ftKb_DatAttrs)
        let backup = (*fp).dat_attrs_backup;
        let src = (*(*fp).ft_data).ext_attr as *const ftKb_DatAttrs;
        *((*fp).dat_attrs_backup as *mut ftKb_DatAttrs) = *src;
        (*fp).dat_attrs = backup;
        (*fp).set_can_multijump(1);
        (*fp).x2D0 = (*fp).dat_attrs as *mut Fighter_x2D0_t;
        (*fp).u.kb.hat.set_x8_b0((ext::Player_GetFlagsAEBit1((*fp).player_id as i32) & 1) as u8);
        ext::it_8026B3F8(*item_list.add(0), It_Kind_Kirby_CBeam as i32);
        ext::it_8026B3F8(*item_list.add(1), It_Kind_Kirby_Hammer as i32);
        ext::it_8026B3F8(*item_list.add(2), It_Kind_Unk1 as i32);
        ext::it_8026B3F8(*item_list.add(3), It_Kind_Unk2 as i32);
    }
}

/// `ftKb_Init_800EE74C` (death2/take-damage cleanup of the copied ability, then drop death3_cb).
pub fn ftKb_Init_800EE74C(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        ext::ftKb_SpecialAirLw_800F5524(gobj);
        ext::ftKb_AttackDashAir_800F22D4(gobj);
        ext::ftKb_SpecialAirLw_800F5318(gobj);
        ext::ftKb_SpecialN_800F9090(gobj);
        ftKb_SpecialN_800F19AC(gobj);
        ext::ftKb_SpecialN_800F5D04(gobj, false);
        (*fp).death3_cb = None;
    }
}

/// `ftKb_Init_800EE7B8`: the take-damage twin of the above, dropping death1_cb.
pub fn ftKb_Init_800EE7B8(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        ext::ftKb_SpecialAirLw_800F5524(gobj);
        ext::ftKb_AttackDashAir_800F22D4(gobj);
        ext::ftKb_SpecialAirLw_800F5318(gobj);
        ext::ftKb_SpecialN_800F9090(gobj);
        ftKb_SpecialN_800F1A8C(gobj);
        (*fp).death1_cb = None;
    }
}

/// `ftKb_Init_UnkMotionStates4`: play the "fully charged" flash once the hat's charge counter fills.
pub fn ftKb_Init_UnkMotionStates4(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let da = (*fp).dat_attrs as *const ftKb_DatAttrs;
        match (*fp).u.kb.hat.kind {
            Ft_Kind_Donkey => {
                if (*fp).u.kb.xBC == (*da).specialn_dk_swings_to_full_charge {
                    ext::ftCo_800BFFD0(fp, 58, 0);
                }
            }
            Ft_Kind_Samus => {
                if ((*fp).u.kb.xA8 as f32) == (*da).specialn_ss_charge_time {
                    ext::ftCo_800BFFD0(fp, 54, 0);
                }
            }
            Ft_Kind_Mewtwo => {
                if ((*fp).u.kb.x9C as f32) == (*da).specialn_mt_charge_time {
                    ext::ftCo_800BFFD0(fp, 93, 0);
                }
            }
            Ft_Kind_Seak => {
                if (*fp).u.kb.xB4 == 6 {
                    ext::ftCo_800BFFD0(fp, 87, 0);
                }
            }
            _ => {}
        }
    }
}

/// `ftKb_Init_OnItemPickup`: pose the hand model for a light held item.
pub fn ftKb_Init_OnItemPickup(gobj: *mut HSD_GObj, arg1: bool) {
    unsafe {
        let fp = fpp(gobj);
        if st::itIsHeavy((*fp).item_gobj) == 0 {
            match ext::itGetHoldKind((*fp).item_gobj) {
                1 => ext::ftAnim_80070FB4(gobj, 1, 0),
                3 => ext::ftAnim_80070FB4(gobj, 1, 1),
                4 => ext::ftAnim_80070FB4(gobj, 1, 2),
                _ => {}
            }
            if arg1 {
                st::ftAnim_80070C48(gobj, 1);
            }
        }
    }
}

pub fn ftKb_Init_OnItemInvisible(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if st::itIsHeavy((*fp).item_gobj) == 0 {
            st::ftAnim_80070CC4(gobj, 1);
        }
    }
}

pub fn ftKb_Init_OnItemDrop(gobj: *mut HSD_GObj, bool1: i32) {
    crate::tu::ftclink::Fighter_OnItemDrop(gobj, bool1, 1, 1);
}

/// `COPY_ATTRS(gobj, ftKb_DatAttrs)`.
pub fn ftKb_Init_LoadSpecialAttrs(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        *((*fp).dat_attrs as *mut ftKb_DatAttrs) = *((*(*fp).ft_data).ext_attr as *const ftKb_DatAttrs);
    }
}

/// `ftKb_Init_800EEB00(gobj, out)`: Pichu hat's dynamic-bones pointer, or NULL.
pub fn ftKb_Init_800EEB00(_gobj: *mut HSD_GObj, arg1: *mut *mut ArticleDynamicBones) {
    unsafe {
        let hat = hat_table(Ft_Kind_Pichu as usize);
        if hat.is_null() {
            *arg1 = null_mut();
            return;
        }
        *arg1 = (*(*hat).hat_dynamics[4]).anon_0.ftDynamicBones;
    }
}

/// `ftKb_Init_800EEB1C(gobj, out)`: Pichu hat's dynamics `x4`, or 0.
pub fn ftKb_Init_800EEB1C(_gobj: *mut HSD_GObj, arg1: *mut i32) {
    unsafe {
        let hat = hat_table(Ft_Kind_Pichu as usize);
        if hat.is_null() {
            *arg1 = 0;
            return;
        }
        *arg1 = (*(*hat).hat_dynamics[4]).x4;
    }
}

/// `ftKb_Init_GetMotionFileString(kind)`: the Vi demo motion file name for Kirby's hat kind 11 / 14.
pub fn ftKb_Init_GetMotionFileString(arg0: i32) -> *mut i8 {
    unsafe {
        let mut offset: i32 = 0;
        match arg0 {
            11 => offset = 14,
            14 => offset = 17,
            _ => {}
        }
        super::ftkirbydata::ftKb_Init_803CA5A4[(offset - 14) as usize]
    }
}

/// `ftKb_SpecialN_800EEC34(kind, costume, count)`: preload kind's hat archive and costume files async.
pub fn ftKb_SpecialN_800EEC34(arg0: i32, mut arg1: i32, mut arg2: i32) {
    unsafe {
        if arg0 < 0 || arg0 >= Ft_Kind_Max as i32 {
            return;
        }
        if arg1 != 0xFF {
            arg1 = ftKb_CopyCostumeRow(arg1);
        }
        if arg2 > FTKB_COPY_COSTUMES {
            arg2 = FTKB_COPY_COSTUMES;
        }
        let k = arg0 as usize;
        if !ftKb_Init_803CA9D0[k].filename.is_null() {
            ext::lbDvd_800178E8(2, ftKb_Init_803CA9D0[k].filename, 4, 4, 0, 1, 3, 1, 0);
        }
        if !ftKb_Init_803CB3E8[k].is_null() {
            let lo;
            if arg1 == 0xFF {
                lo = 0;
            } else {
                lo = arg1;
                arg2 = arg1 + 1;
            }
            let mut i = lo;
            while i < arg2 {
                let cs = &*ftKb_Init_803CB3E8[k].add(i as usize);
                if !cs.dat_filename.is_null() {
                    ext::lbDvd_800178E8(2, cs.dat_filename, 4, 4, 0, 1, 3, 1, 0);
                }
                i += 1;
            }
        }
        // C compares a u8 with (char) -1: 255 != -1 is always true.
        if (ftKb_Init_803CB46C[k] as i32) != (-1i8 as i32) {
            ext::efAsync_LoadAsync(ftKb_Init_803CB46C[k] as i32);
        }
    }
}

/// `ftKb_SpecialN_800EED50(kind, costume)`: load kind's hat archive and the costume's hat model.
pub fn ftKb_SpecialN_800EED50(arg0: i32, arg1: i32) {
    unsafe {
        if arg0 < 0 || arg0 >= Ft_Kind_Max as i32 {
            return;
        }
        let arg1 = ftKb_CopyCostumeRow(arg1) as usize;
        if arg0 != -1 && arg0 != 4 {
            let k = arg0 as usize;
            if !ftKb_Init_803CA9D0[k].filename.is_null() && (*hat_slot(k)).is_null() {
                ext::lbArchive_80017040(
                    null_mut(),
                    ftKb_Init_803CA9D0[k].filename,
                    hat_slot(k) as *mut c_void,
                    &[ftKb_Init_803CA9D0[k].name as *mut c_void],
                );
            }
            if !ftKb_Init_803CB3E8[k].is_null() {
                let item = ftKb_Init_803C9FC8[k].add(arg1);
                if (*item).joint.is_null() {
                    let costumes = ftKb_Init_803CB3E8[k];
                    let cs = ftKb_Init_803CB3E8[k].add(arg1);
                    if !(*cs).matanim_joint_name.is_null() {
                        ext::lbArchive_80017040(
                            null_mut(),
                            (*costumes.add(arg1)).dat_filename,
                            &raw mut (*item).joint as *mut c_void,
                            &[
                                (*cs).joint_name as *mut c_void,
                                &raw mut (*item).matanim as *mut c_void,
                                (*cs).matanim_joint_name as *mut c_void,
                            ],
                        );
                    } else {
                        ext::lbArchive_80017040(
                            null_mut(),
                            (*costumes.add(arg1)).dat_filename,
                            &raw mut (*item).joint as *mut c_void,
                            &[(*cs).joint_name as *mut c_void],
                        );
                        (*item).matanim = null_mut();
                    }
                }
            }
            ext::efAsync_LoadSync(ftKb_Init_803CB46C[k] as i32);
        }
    }
}

/// `ftKb_Init_UnkMotionStates5`: register the hat loader with the player preload.
pub fn ftKb_Init_UnkMotionStates5() {
    ext::Player_80031DC8(ftKb_SpecialN_800EED50);
}

/// `ftKb_SpecialN_800EEEC4(gobj, kind)`: call the hat's unload callback, if any.
pub fn ftKb_SpecialN_800EEEC4(gobj: *mut HSD_GObj, kind: i32) {
    unsafe {
        if let Some(cb) = ftKb_Init_803C9CC8[(kind * 2 + 1) as usize] {
            cb(gobj);
        }
    }
}

/// `ftKb_UnkMtxFunc0(gobj, arg1, mtx)`: draw the hat model at bone 6.
pub fn ftKb_UnkMtxFunc0(gobj: *mut HSD_GObj, arg1: i32, mtx: MtxPtr) {
    unsafe {
        let fp = fpp(gobj);
        if (*fp).u.kb.hat.jobj.is_null() {
            return;
        }
        if (*fp).x2225_b2() == 0 {
            return;
        }
        let bone_mtx = ext::HSD_JObjGetMtxPtr((*(*fp).parts.add(6)).joint);
        let jobj = (*fp).u.kb.hat.jobj;
        ext::HSD_JObjCopyMtx((*fp).u.kb.hat.jobj, bone_mtx);
        (*jobj).flags |= (JOBJ_USER_DEF_MTX | JOBJ_MTX_INDEP_PARENT | JOBJ_MTX_INDEP_SRT) as u32;
        ext::HSD_JObjSetMtxDirty(jobj);
        ext::HSD_JObjDispAll((*fp).u.kb.hat.jobj, mtx, ext::HSD_GObj_80390EB8(arg1), 0);
    }
}

/// `ftKb_Init_UnkMotionStates6`: the hat's jobj, or (a correctness hack in the C) the gobj itself.
pub fn ftKb_Init_UnkMotionStates6(gobj: *mut HSD_GObj) -> *mut HSD_JObj {
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).u.kb.hat.jobj.is_null() {
            return (*fp).u.kb.hat.jobj;
        }
        gobj as *mut HSD_JObj
    }
}

/// `ftKb_SpecialN_800EF040(gobj, kind + 1, hat)`: attach the hat's extra body parts from the hat archive.
pub fn ftKb_SpecialN_800EF040(gobj: *mut HSD_GObj, arg1: i32, hat: *mut KirbyHatStruct) {
    unsafe {
        let mask = (*hat).hat_dynamics[1] as usize as u32;
        if mask != 0 {
            let fp = fpp(gobj);
            let ft_data = *melee_fighter::hooks::fighter_804d6540().add((*fp).kind as usize);
            let count = (*ft_data).x4;
            let joint = (*ftKb_Init_803C9FC8[arg1 as usize].add(copy_row(fp))).joint;
            let mut parts = (*ft_data).x0;
            let mut i = 0;
            while i < count {
                if (1u32 << i) & mask != 0 {
                    ext::ftParts_800753D4(fp, parts, joint);
                }
                i += 1;
                parts = parts.add(1);
            }
        }
    }
}

/// `ftKb_SpecialN_insert_joint_refs` (static inline): map every joint of the tree onto the fighter's
/// bones in the ID table, resetting the walk state.
unsafe fn ftKb_SpecialN_insert_joint_refs(
    total_dobjs: &mut i32,
    root: *mut HSD_Joint,
    fp: *mut Fighter,
    part_idx: &mut i32,
    joint: &mut *mut HSD_Joint,
    joint_idx: &mut i32,
    byte_base: &mut i32,
) {
    *byte_base = 0;
    *total_dobjs = 0;
    *part_idx = 0;
    *joint = root;
    *joint_idx = 0;
    while !(*joint).is_null() {
        let parts = (*fp).parts;
        let mut bone = parts.add(*part_idx as usize);
        while (*bone).anon_8.anon_0.flags_b1() == 0 {
            bone = bone.add(1);
            *part_idx += 1;
        }
        ext::HSD_IDInsertToTable(null_mut(), *joint as *mut c_void, (*parts.add(*part_idx as usize)).joint);
        *part_idx += 1;
        ext::ftAnim_GetNextJointInTree(joint, joint_idx);
    }
    *joint = root;
}

/// Walk to the last DObj of `jobj`'s list and append `dobj` (the C's `tail` loop + `lb_8000CE30`),
/// or make it the list head.
unsafe fn append_dobj(jobj: *mut HSD_JObj, tail: *mut HSD_DObj, dobj: *mut HSD_DObj) {
    let mut tail = tail;
    if tail.is_null() {
        ext::HSD_JObjAddDObj(jobj, dobj);
    } else {
        while !tail.is_null() {
            let nxt = (*tail).next;
            if nxt.is_null() {
                break;
            }
            tail = (*tail).next;
        }
        ext::lb_8000CE30(tail, dobj);
    }
}

/// `ftKb_SpecialN_800EF0E4(gobj, kind + 1, indices)`: instantiate the hat's DObjs on the fighter's bones,
/// recording each bone's first DObj index in `arg2`.
pub fn ftKb_SpecialN_800EF0E4(gobj: *mut HSD_GObj, arg1: i32, arg2: *mut u8) {
    unsafe {
        let fp = fpp(gobj);
        let mut current_joint: *mut HSD_Joint = null_mut();
        let mut joint_idx: i32 = 0;
        let mut total_dobjs: i32 = 0;
        let mut insert_part_idx: i32 = 0;
        let mut byte_base: i32 = 0;
        ext::ftPartsPObjSetDefaultClass();
        let root = (*ftKb_Init_803C9FC8[arg1 as usize].add(copy_row(fp))).joint;
        ftKb_SpecialN_insert_joint_refs(&mut total_dobjs, root, fp, &mut insert_part_idx, &mut current_joint, &mut joint_idx, &mut byte_base);
        joint_idx = 0;
        let mut arg2_idx: i32 = 0;
        insert_part_idx = 0;
        while !current_joint.is_null() {
            let mut group_count: i32 = 0;
            // ftKb_SpecialN_800EF0E4_find_bone
            let mut bone = (*fp).parts.add(insert_part_idx as usize);
            while (*bone).anon_8.anon_0.flags_b1() == 0 {
                bone = bone.add(1);
                arg2_idx += 1;
                insert_part_idx += 1;
            }
            let jobj = (*(*fp).parts.add(insert_part_idx as usize)).joint;
            let mut dobj = ext::HSD_DObjLoadDesc((*current_joint).u.dobjdesc);
            *arg2.add(arg2_idx as usize) = total_dobjs as u8;
            if !dobj.is_null() {
                let tail = ext::HSD_JObjGetDObj(jobj);
                ext::HSD_DObjResolveRefsAll(dobj, (*current_joint).u.dobjdesc);
                append_dobj(jobj, tail, dobj);
                loop {
                    if dobj.is_null() {
                        break;
                    }
                    if total_dobjs >= 0x20 {
                        panic!("fighter parts model dobj num over! (HSD_ASSERT ftkirby.c:0x43E)");
                    }
                    let dst = (*fp).u.kb.hat.x14.data;
                    *dst.add(total_dobjs as usize) = dobj;
                    let mobj = (*dobj).mobj;
                    if !mobj.is_null() {
                        ext::hsdChangeClass_ftMObj(mobj);
                    }
                    dobj = (*dobj).next;
                    total_dobjs += 1;
                    group_count += 1;
                }
                if group_count >= 0x80 {
                    panic!("fighter dobj num over! (HSD_ASSERT ftkirby.c:0x44C)");
                }
                (*(*fp).parts.add(insert_part_idx as usize)).anon_8.anon_0.set_flags_b6(1);
            }
            arg2_idx += 1;
            insert_part_idx += 1;
            ext::ftAnim_GetNextJointInTree(&mut current_joint, &mut joint_idx);
        }
        // ftKb_SpecialN_800EF0E4_finish
        (*fp).u.kb.hat.x14.count = total_dobjs as u32;
        ext::ftPartsPObjClearDefaultClass();
    }
}

/// `ftKb_SpecialN_800EF35C(gobj, kind + 1, indices)`: bind the hat's mat-anims to the instantiated DObjs.
pub fn ftKb_SpecialN_800EF35C(gobj: *mut HSD_GObj, arg1: i32, arg2: *mut u8) {
    unsafe {
        let fp = fpp(gobj);
        let costume_data = ftKb_Init_803C9FC8[arg1 as usize];
        let mut matanimjoint = (*costume_data.add(copy_row(fp))).matanim;
        let mut idx: i32 = 0;
        let mut arg1: i32 = 0;
        let mut arg2 = arg2;
        while !matanimjoint.is_null() {
            if melee_fighter::ftdmgfall::ftparts_8007506c((*fp).kind as i32, arg1) != 0 {
                arg1 += 1;
                arg2 = arg2.add(1);
            } else {
                if !(*matanimjoint).matanim.is_null() {
                    ext::HSD_DObjAddAnimAll(*(*fp).u.kb.hat.x14.data.add(*arg2 as usize), (*matanimjoint).matanim as *mut c_void, null_mut());
                }
                arg1 += 1;
                arg2 = arg2.add(1);
                ext::ftAnim_GetNextMatAnimJointInTree(&mut matanimjoint, &mut idx);
            }
        }
    }
}

/// `ftKb_SpecialN_800EF438(gobj, hat)`: instantiate the hat's second DObj set (`hat_dynamics[2]`).
pub fn ftKb_SpecialN_800EF438(gobj: *mut HSD_GObj, hat: *mut KirbyHatStruct) {
    unsafe {
        let root = (*hat).hat_dynamics[2] as *mut HSD_Joint;
        let fp = fpp(gobj);
        if !root.is_null() {
            let mut current_joint: *mut HSD_Joint = null_mut();
            let mut joint_idx: i32 = 0;
            let mut total_dobjs: i32 = 0;
            let mut insert_part_idx: i32 = 0;
            let mut byte_base: i32 = 0;
            ext::ftPartsPObjSetDefaultClass();
            ftKb_SpecialN_insert_joint_refs(&mut total_dobjs, root, fp, &mut insert_part_idx, &mut current_joint, &mut joint_idx, &mut byte_base);
            joint_idx = 0;
            insert_part_idx = 0;
            while !current_joint.is_null() {
                let mut group_count: i32 = 0;
                while (*(*fp).parts.add(insert_part_idx as usize)).anon_8.anon_0.flags_b1() == 0 {
                    insert_part_idx += 1;
                }
                let jobj = (*(*fp).parts.add(insert_part_idx as usize)).joint;
                let mut dobj = ext::HSD_DObjLoadDesc((*current_joint).u.dobjdesc);
                if !dobj.is_null() {
                    let tail = ext::HSD_JObjGetDObj(jobj);
                    (*(*fp).parts.add(insert_part_idx as usize)).anon_8.anon_0.set_flags2_b7(1);
                    ext::HSD_DObjResolveRefsAll(dobj, (*current_joint).u.dobjdesc);
                    append_dobj(jobj, tail, dobj);
                    loop {
                        if dobj.is_null() {
                            break;
                        }
                        if total_dobjs >= 0x20 {
                            panic!("fighter parts model dobj num over! (HSD_ASSERT ftkirby.c:0x4B9)");
                        }
                        let dst = (*fp).u.kb.hat.x1C.data;
                        *dst.add(total_dobjs as usize) = dobj;
                        let mobj = (*dobj).mobj;
                        if !mobj.is_null() {
                            ext::hsdChangeClass_ftMObj(mobj);
                        }
                        dobj = (*dobj).next;
                        total_dobjs += 1;
                        group_count += 1;
                    }
                    if group_count >= 0x80 {
                        panic!("fighter dobj num over! (HSD_ASSERT ftkirby.c:0x4C7)");
                    }
                }
                insert_part_idx += 1;
                ext::ftAnim_GetNextJointInTree(&mut current_joint, &mut joint_idx);
            }
            (*fp).u.kb.hat.x1C.count = total_dobjs as u32;
            ext::ftPartsPObjClearDefaultClass();
        } else {
            *(*fp).u.kb.hat.x1C.data = null_mut();
        }
    }
}

/// `ftKb_RemoveHatParts` (static inline): detach the hat's body parts named by `mask`.
unsafe fn ftKb_RemoveHatParts(gobj: *mut HSD_GObj, mask: u32) {
    let fp = fpp(gobj);
    let ft_data = *melee_fighter::hooks::fighter_804d6540().add((*fp).kind as usize);
    let mut i = (*ft_data).x4 - 1;
    while i >= 0 {
        if (1u32 << i) & mask != 0 {
            ext::ftParts_800755E8(fp, (*ft_data).x0.add(i as usize));
        }
        i -= 1;
    }
}

/// `ftKb_SpecialN_800EF69C(gobj, kind + 1, hat)`: tear down the hat's DObjs and body parts.
pub fn ftKb_SpecialN_800EF69C(gobj: *mut HSD_GObj, _arg1: i32, hat: *mut KirbyHatStruct) {
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).u.kb.hat.x14.data.is_null() {
            let parts_num = (*(*melee_fighter::hooks::ft_parts_table().add((*fp).kind as usize))).parts_num;
            let mut i = 0;
            while i < parts_num as usize {
                let bone = (*fp).parts.add(i);
                let jobj = (*bone).joint;
                if !jobj.is_null() && ((*bone).anon_8.anon_0.flags_b6() != 0 || (*bone).anon_8.anon_0.flags2_b7() != 0) {
                    // (*b9p >> 1) & 1 and (*b9p >> 2) & 1 are the LSB-numbered bits 1 and 2 of flags2:
                    // flags2_b6 / flags2_b5 in the MSB-first bitfield names.
                    if (*bone).anon_8.anon_0.flags2_b6() != 0 {
                        let dobj;
                        if (*bone).anon_8.anon_0.flags2_b5() != 0 {
                            dobj = *(*fp).x203C.data.add((*bone).anon_c.anon_0.xD() as usize);
                        } else {
                            dobj = *(*fp).dobj_list.data.add((*bone).anon_c.anon_0.xD() as usize);
                        }
                        ext::HSD_DObjRemoveAll(if !dobj.is_null() { (*dobj).next } else { null_mut() });
                        ext::lb_8000CE30(dobj, null_mut());
                    } else {
                        ext::HSD_DObjRemoveAll(ext::HSD_JObjGetDObj(jobj));
                        ext::lb_8000CE40(jobj, null_mut());
                    }
                    (*(*fp).parts.add(i)).anon_8.anon_0.set_flags_b6(0);
                    (*(*fp).parts.add(i)).anon_8.anon_0.set_flags2_b7(0);
                }
                i += 1;
            }
            ext::HSD_ObjFree(ext::fighter_x2040_alloc_data(), (*fp).u.kb.hat.x14.data as *mut c_void);
            ext::HSD_ObjFree(ext::fighter_x2040_alloc_data(), (*fp).u.kb.hat.x1C.data as *mut c_void);
            (*fp).u.kb.hat.x14.data = null_mut();
        }
        let mask = (*hat).hat_dynamics[1] as usize as u32;
        if mask != 0 {
            ftKb_RemoveHatParts(gobj, mask);
        }
    }
}

/// `ftKb_UnkIntBoolFunc0(fp, arg1, arg2)`: show (arg2) or hide the hat model's parts for model slot `arg1`.
pub fn ftKb_UnkIntBoolFunc0(fp: *mut Fighter, arg1: i32, arg2: bool) {
    unsafe {
        use ext::{ftParts_80074CA0, ftParts_80074D7C};
        if !(*fp).u.kb.hat.x14.data.is_null() {
            if arg2 {
                if !(*fp).u.kb.hat.jobj.is_null() {
                    ftParts_80074CA0(&raw mut (*fp).u.kb.hat.x24, arg1, &(*fp).u.kb.hat.x14);
                    return;
                }
                let idx = (*fp).x5F4_arr[0].idx;
                let var_r0 = if idx >= 2 && idx <= 6 { 1 } else { 0 };
                if var_r0 != 0 {
                    ftParts_80074D7C(&raw mut (*fp).u.kb.hat.x24, arg1, &(*fp).u.kb.hat.x14);
                    return;
                }
                if arg1 == 2 {
                    if !(*(*fp).u.kb.hat.x1C.data).is_null() {
                        ftParts_80074D7C(&raw mut (*fp).x5AC, arg1, &(*fp).x203C);
                        ftParts_80074CA0(&raw mut (*fp).u.kb.hat.x24, arg1, &(*fp).u.kb.hat.x1C);
                    }
                } else {
                    ftParts_80074D7C(&raw mut (*fp).x5AC, arg1, &(*fp).dobj_list);
                    ftParts_80074CA0(&raw mut (*fp).u.kb.hat.x24, arg1, &(*fp).u.kb.hat.x14);
                }
            } else {
                if (*fp).u.kb.hat.jobj.is_null() && arg1 == 2 {
                    ftParts_80074D7C(&raw mut (*fp).u.kb.hat.x24, arg1, &(*fp).u.kb.hat.x1C);
                    return;
                }
                ftParts_80074D7C(&raw mut (*fp).u.kb.hat.x24, arg1, &(*fp).u.kb.hat.x14);
            }
        }
    }
}

pub fn ftKb_Init_UnkCallbackPairs0_0(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).u.kb.hat.x14.data.is_null() && (*fp).u.kb.hat.jobj.is_null() {
            ext::ftAnim_800705E0(&raw mut (*fp).u.kb.x44);
        }
    }
}

pub fn ftKb_Init_UnkCallbackPairs0_1(gobj: *mut HSD_GObj, arg1: i32, arg2: f32) {
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).u.kb.hat.x14.data.is_null() && (*fp).u.kb.hat.jobj.is_null() {
            ext::ftAnim_80070458(fp, &raw mut (*fp).u.kb.x44, arg1 as u32, arg2);
        }
    }
}

/// `ftKb_SpecialN_800EFA40`: Mario's hat (the `ft_80459B88.x0` archive).
pub fn ftKb_SpecialN_800EFA40(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if (*fp).u.kb.hat.jobj.is_null() {
            let unk = ft_80459B88.x0;
            (*fp).u.kb.hat.x14.data = ext::HSD_ObjAlloc(ext::fighter_x2040_alloc_data()) as *mut *mut HSD_DObj;
            ext::ftPartsPObjSetDefaultClass();
            (*fp).u.kb.hat.jobj = ext::HSD_JObjLoadJoint((*unk).x0);
            (*fp).set_x2225_b2(1);
            ext::ftPartsPObjClearDefaultClass();
            ext::ftParts_80075650(gobj, (*fp).u.kb.hat.jobj, &mut (*fp).u.kb.hat.x14);
            ext::ftParts_8007487C(&mut (*unk).x4, &mut (*fp).u.kb.hat.x24, 0, &mut (*fp).u.kb.hat.x14, &mut (*fp).u.kb.hat.x14);
        }
    }
}

/// `ftKb_SpecialN_800EFAF0_inline` (static inline): unload a simple (one-JObj) hat.
unsafe fn ftKb_SpecialN_800EFAF0_inline(gobj: *mut HSD_GObj) {
    let fp = fpp(gobj);
    if !(*fp).u.kb.hat.jobj.is_null() {
        ext::HSD_JObjRemoveAll((*fp).u.kb.hat.jobj);
        (*fp).u.kb.hat.jobj = null_mut();
        ext::HSD_ObjFree(ext::fighter_x2040_alloc_data(), (*fp).u.kb.hat.x14.data as *mut c_void);
        (*fp).u.kb.hat.x14.data = null_mut();
    }
}

pub fn ftKb_SpecialN_800EFAF0(gobj: *mut HSD_GObj) {
    unsafe { ftKb_SpecialN_800EFAF0_inline(gobj) }
}

/// `ftKb_LoadHat` (static inline): load a simple one-JObj hat.
pub unsafe fn ftKb_LoadHat(gobj: *mut HSD_GObj, fp: *mut Fighter, hat: *mut KirbyHatStruct) {
    if hat.is_null() {
        return;
    }
    (*fp).u.kb.hat.x14.data = ext::HSD_ObjAlloc(ext::fighter_x2040_alloc_data()) as *mut *mut HSD_DObj;
    ext::ftPartsPObjSetDefaultClass();
    (*fp).u.kb.hat.jobj = ext::HSD_JObjLoadJoint((*hat).hat_joint);
    (*fp).set_x2225_b2(1);
    ext::ftPartsPObjClearDefaultClass();
    ext::ftParts_80075650(gobj, (*fp).u.kb.hat.jobj, &mut (*fp).u.kb.hat.x14);
    ext::ftParts_8007487C(&mut (*hat).desc, &mut (*fp).u.kb.hat.x24, 0, &mut (*fp).u.kb.hat.x14, &mut (*fp).u.kb.hat.x14);
}

/// Load a one-JObj hat for `kind` unless one is already worn. Shared body of the simple loaders.
unsafe fn load_simple_hat(gobj: *mut HSD_GObj, kind: usize) -> *mut Fighter {
    let fp = fpp(gobj);
    if (*fp).u.kb.hat.jobj.is_null() {
        let hat = hat_table(kind);
        ftKb_LoadHat(gobj, fp, hat);
        return fp;
    }
    null_mut()
}

macro_rules! simple_loader {
    ($name:ident, $kind:expr) => {
        pub fn $name(gobj: *mut HSD_GObj) {
            unsafe {
                load_simple_hat(gobj, $kind as usize);
            }
        }
    };
    ($name:ident, $kind:expr, $post:path) => {
        pub fn $name(gobj: *mut HSD_GObj) {
            unsafe {
                let fp = load_simple_hat(gobj, $kind as usize);
                if !fp.is_null() {
                    $post(&mut *fp);
                }
            }
        }
    };
}

macro_rules! simple_unloader {
    ($name:ident) => {
        pub fn $name(gobj: *mut HSD_GObj) {
            unsafe { ftKb_SpecialN_800EFAF0_inline(gobj) }
        }
    };
    ($name:ident, unload_bones) => {
        pub fn $name(gobj: *mut HSD_GObj) {
            unsafe {
                let fp = fpp(gobj);
                ftKb_SpecialN_800EFAF0_inline(gobj);
                melee_fighter::tu::ftdynamics::ftCo_UnloadDynamicBones(&mut *fp);
            }
        }
    };
}

use melee_fighter::tu::ftdynamics as dyn_;

// Mario (800EFA40 above is the x0 archive; the hat table's Mario row is Kirby_Unk-based).
simple_loader!(ftKb_SpecialN_800EFB4C, Ft_Kind_Mario);
simple_unloader!(ftKb_SpecialN_800EFBFC);
simple_loader!(ftKb_SpecialN_800EFC58, Ft_Kind_Fox);
simple_unloader!(ftKb_SpecialN_800EFD08);
simple_loader!(ftKb_SpecialN_800EFD64, Ft_Kind_Kirby, dyn_::ftCo_8009D4D4);
simple_unloader!(ftKb_SpecialN_800EFE1C, unload_bones);
simple_loader!(ftKb_SpecialN_800EFE80, Ft_Kind_Koopa, dyn_::ftCo_8009D074);
simple_unloader!(ftKb_SpecialN_800EFF38, unload_bones);
simple_loader!(ftKb_SpecialN_800EFF9C, Ft_Kind_Link, dyn_::ftCo_8009D704);
simple_unloader!(ftKb_SpecialN_800F0054, unload_bones);
simple_loader!(ftKb_SpecialN_800F00B8, Ft_Kind_Seak);
simple_unloader!(ftKb_SpecialN_800F0168);
simple_loader!(ftKb_SpecialN_800F01C4, Ft_Kind_Ness);
simple_unloader!(ftKb_SpecialN_800F0274);
simple_loader!(ftKb_SpecialN_800F02D0, Ft_Kind_Peach);
simple_unloader!(ftKb_SpecialN_800F0380);
simple_loader!(ftKb_SpecialN_800F03DC, Ft_Kind_Nana, dyn_::ftCo_8009D2A4);
simple_unloader!(ftKb_SpecialN_800F0494, unload_bones);
simple_loader!(ftKb_SpecialN_800F04F8, Ft_Kind_Pikachu);
simple_unloader!(ftKb_SpecialN_800F05A8);
simple_loader!(ftKb_SpecialN_800F0604, Ft_Kind_Samus);
simple_unloader!(ftKb_SpecialN_800F06B4);
simple_loader!(ftKb_SpecialN_800F0710, Ft_Kind_Mewtwo);
simple_unloader!(ftKb_SpecialN_800F07C0);
simple_loader!(ftKb_SpecialN_800F081C, Ft_Kind_Luigi, dyn_::ftCo_8009D920);
simple_unloader!(ftKb_SpecialN_800F08D4, unload_bones);
simple_loader!(ftKb_SpecialN_800F0938, Ft_Kind_Mars, dyn_::ftCo_8009D5EC);
simple_unloader!(ftKb_SpecialN_800F09F0, unload_bones);
simple_loader!(ftKb_SpecialN_800F0A54, Ft_Kind_Zelda, dyn_::ftCo_8009D18C);
simple_unloader!(ftKb_SpecialN_800F0B0C, unload_bones);
simple_loader!(ftKb_SpecialN_800F0B70, Ft_Kind_CLink);
simple_unloader!(ftKb_SpecialN_800F0C20);
simple_loader!(ftKb_SpecialN_800F0C7C, Ft_Kind_Falco, dyn_::ftCo_8009D3BC);
simple_unloader!(ftKb_SpecialN_800F0D34, unload_bones);
simple_loader!(ftKb_SpecialN_800F0D98, Ft_Kind_GameWatch);
simple_unloader!(ftKb_SpecialN_800F0E48);
simple_loader!(ftKb_SpecialN_800F0EA4, Ft_Kind_Ganon, dyn_::ftCo_8009DA38);
simple_unloader!(ftKb_SpecialN_800F0F5C, unload_bones);

/// `LOAD_HAT` (ftkirby.c macro): the multi-part hat loader. Returns false when the archive has no hat for `kind`.
unsafe fn load_hat(gobj: *mut HSD_GObj, fp: *mut Fighter, kind: usize, indices: *mut u8) -> *mut KirbyHatStruct {
    let hat = hat_table(kind);
    if hat.is_null() {
        return null_mut();
    }
    ftKb_SpecialN_800EF040(gobj, kind as i32 + 1, hat);
    (*fp).u.kb.hat.x14.data = ext::HSD_ObjAlloc(ext::fighter_x2040_alloc_data()) as *mut *mut HSD_DObj;
    (*fp).u.kb.hat.x1C.data = ext::HSD_ObjAlloc(ext::fighter_x2040_alloc_data()) as *mut *mut HSD_DObj;
    ftKb_SpecialN_800EF0E4(gobj, kind as i32 + 1, indices);
    ftKb_SpecialN_800EF35C(gobj, kind as i32 + 1, indices);
    ftKb_SpecialN_800EF438(gobj, hat);
    // `(FtPartsDesc*) hat`: the C passes the hat struct itself, not `&hat->desc`.
    ext::ftParts_8007487C(hat as *mut FtPartsDesc, &mut (*fp).u.kb.hat.x24, copy_row(fp) as u32, &mut (*fp).u.kb.hat.x14, &mut (*fp).u.kb.hat.x1C);
    ext::ftAnim_80070200(
        fp,
        &mut (*hat).desc.vis_table as *mut _ as *mut ftData_x8_x8,
        &raw mut (*fp).u.kb.x44,
        &raw mut (*fp).u.kb.hat.x14,
    );
    hat
}

/// `ftKb_SpecialN_800F0FC0`: Captain Falcon's hat.
pub fn ftKb_SpecialN_800F0FC0(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let mut part_dobj_indices = [0u8; 0x90];
        if !(*fp).u.kb.hat.x14.data.is_null() {
            return;
        }
        load_hat(gobj, fp, Ft_Kind_Captain as usize, part_dobj_indices.as_mut_ptr());
    }
}

/// `ftKb_SpecialN_800F10A4`: unload Captain Falcon's hat.
pub fn ftKb_SpecialN_800F10A4(gobj: *mut HSD_GObj) {
    unsafe {
        let hat = hat_table(Ft_Kind_Captain as usize);
        if hat.is_null() {
            return;
        }
        ftKb_SpecialN_800EF69C(gobj, 3, hat);
    }
}

/// `ftKb_SpecialN_800F10D4`: Yoshi's hat.
pub fn ftKb_SpecialN_800F10D4(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let mut part_dobj_indices = [0u8; 0x88];
        if !(*fp).u.kb.hat.x14.data.is_null() {
            return;
        }
        if load_hat(gobj, fp, Ft_Kind_Yoshi as usize, part_dobj_indices.as_mut_ptr()).is_null() {
            return;
        }
        dyn_::ftCo_8009D81C(&mut *fp);
    }
}

/// `ftKb_SpecialN_800F11AC`: unload Yoshi's hat (hat row 14).
pub fn ftKb_SpecialN_800F11AC(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let hat = hat_table(14);
        if hat.is_null() {
            return;
        }
        ftKb_SpecialN_800EF69C(gobj, 0xF, hat);
        dyn_::ftCo_UnloadDynamicBones(&mut *fp);
    }
}

/// `ftKb_SpecialN_800F11F0`: Jigglypuff's hat.
pub fn ftKb_SpecialN_800F11F0(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let mut part_dobj_indices = [0u8; 0x88];
        if !(*fp).u.kb.hat.x14.data.is_null() {
            return;
        }
        if load_hat(gobj, fp, Ft_Kind_Purin as usize, part_dobj_indices.as_mut_ptr()).is_null() {
            return;
        }
        dyn_::ftCo_8009DB50(&mut *fp);
    }
}

/// `ftKb_SpecialN_800F12C8`: unload Jigglypuff's hat.
pub fn ftKb_SpecialN_800F12C8(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let hat = hat_table(Ft_Kind_Purin as usize);
        if hat.is_null() {
            return;
        }
        ftKb_SpecialN_800EF69C(gobj, 0x10, hat);
        dyn_::ftCo_UnloadDynamicBones(&mut *fp);
    }
}

/// `ftKb_SpecialN_800F130C`: Dr. Mario's hat.
pub fn ftKb_SpecialN_800F130C(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let mut part_dobj_indices = [0u8; 0x90];
        if !(*fp).u.kb.hat.x14.data.is_null() {
            return;
        }
        load_hat(gobj, fp, Ft_Kind_DrMario as usize, part_dobj_indices.as_mut_ptr());
    }
}

/// `ftKb_SpecialN_800F13F0`: unload Dr. Mario's hat.
pub fn ftKb_SpecialN_800F13F0(gobj: *mut HSD_GObj) {
    unsafe {
        let hat = hat_table(Ft_Kind_DrMario as usize);
        if hat.is_null() {
            return;
        }
        ftKb_SpecialN_800EF69C(gobj, 0x16, hat);
    }
}

/// `ftKb_SpecialN_800F1420(gobj, colour)`: write the 4-byte colour into the diffuse of every material of
/// the hat's DObjs named by the fighter's `x5AC.xC[4]` lookup. Returns the last index pointer.
pub fn ftKb_SpecialN_800F1420(gobj: *mut HSD_GObj, arg1: *const u32) -> *mut u8 {
    unsafe {
        let fp = fpp(gobj);
        let lookup = (*fp).x5AC.xC[4];
        let mut p: *mut u8 = null_mut();
        let mut i = 0;
        while i < (*lookup).x0 {
            let entry = (*lookup).x4.add(i as usize);
            p = (*entry).x4;
            let mut j = 0;
            while j < (*entry).x0 {
                let dobj = *(*fp).u.kb.hat.x14.data.add(*p as usize);
                let mobj = if !dobj.is_null() { (*dobj).mobj } else { null_mut() };
                if !mobj.is_null() {
                    let mat = (*mobj).mat;
                    if !mat.is_null() {
                        core::ptr::write_unaligned(&raw mut (*mat).diffuse as *mut u32, core::ptr::read_unaligned(arg1));
                    }
                }
                j += 1;
                p = p.add(1);
            }
            i += 1;
        }
        p
    }
}

/// `ftKb_SpecialN_800F14B4`: Pichu's hat (recoloured with the hat archive's two colour words).
pub fn ftKb_SpecialN_800F14B4(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let mut part_dobj_indices = [0u8; 0x88];
        if !(*fp).u.kb.hat.x14.data.is_null() {
            return;
        }
        let hat = load_hat(gobj, fp, Ft_Kind_Pichu as usize, part_dobj_indices.as_mut_ptr());
        if hat.is_null() {
            return;
        }
        let lookup = (*hat).hat_dynamics[3] as *mut FtPartsVisLookup;
        (*fp).u.kb.hat.x24.xC[4] = lookup;
        (*fp).x5AC.xC[4] = lookup;
        ext::ftParts_80074D7C(&raw mut (*fp).u.kb.hat.x24, 4, &(*fp).u.kb.hat.x14);
        let colours = (*hat).hat_dynamics[4] as *const u8;
        ftKb_SpecialN_800F1420(gobj, colours.add(4) as *const u32);
        core::ptr::write_unaligned(
            &raw mut (*fp).x610_color_rgba[1] as *mut u32,
            core::ptr::read_unaligned(colours.add(8) as *const u32),
        );
        ext::Fighter_UpdateModelScale(gobj);
    }
}

/// `ftKb_SpecialN_800F15D8`: unload Pichu's hat.
pub fn ftKb_SpecialN_800F15D8(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let new_var = hat_table(Ft_Kind_Pichu as usize);
        if new_var.is_null() {
            return;
        }
        ftKb_SpecialN_800EF69C(gobj, 0x18, new_var);
        (*fp).x5AC.xC[4] = null_mut();
        ext::Fighter_UpdateModelScale(gobj);
    }
}

/// `ftKb_SpecialN_Enter`: the hat's neutral-B enter callback, or Kirby's own inhale.
pub fn ftKb_SpecialN_Enter(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if let Some(cb) = ftKb_Init_803C9DD0[(*fp).u.kb.hat.kind as usize] {
            cb(gobj);
        } else {
            ext::ftKb_SpecialN_800F5F68(gobj);
        }
    }
}

/// `ftKb_SpecialAirN_Enter`: the aerial counterpart.
pub fn ftKb_SpecialAirN_Enter(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if let Some(cb) = ftKb_Init_803C9E54[(*fp).u.kb.hat.kind as usize] {
            cb(gobj);
        } else {
            ext::ftKb_SpecialN_800F6070(gobj);
        }
    }
}

/// `ftKb_SpecialN_800F16D0(gobj, kind)`: register the copied ability's items (articles) for `kind`.
pub fn ftKb_SpecialN_800F16D0(_gobj: *mut HSD_GObj, kind: i32) {
    unsafe {
        let art = |hat: *mut KirbyHatStruct, slot: usize| (*hat).hat_dynamics[slot] as *mut c_void;
        // `hats[idx]` literals are the C's (hat table index = kind - 1 here).
        macro_rules! with_hat {
            ($idx:expr, |$h:ident| $body:block) => {{
                let $h = hat_table($idx);
                if !$h.is_null() $body
            }};
        }
        match kind as u32 {
            Ft_Kind_Mario => {
                if !ft_80459B88.x0.is_null() {
                    ext::it_8026B3F8((*ft_80459B88.x0).xC, It_Kind_Kirby_MarioFire as i32);
                }
            }
            Ft_Kind_Luigi => with_hat!(16, |hat| {
                ext::it_8026B3F8(art(hat, 0), It_Kind_Kirby_LuigiFire as i32);
            }),
            Ft_Kind_Popo => with_hat!(9, |hat| {
                ext::it_8026B3F8(art(hat, 0), It_Kind_Kirby_IceClimberIce as i32);
            }),
            Ft_Kind_Peach => with_hat!(8, |hat| {
                ext::it_8026B3F8(art(hat, 0), It_Kind_Kirby_PeachToad as i32);
                ext::it_8026B3F8(art(hat, 1), It_Kind_Kirby_PeachToadSpore as i32);
            }),
            Ft_Kind_Fox => with_hat!(0, |hat| {
                ext::it_8026B3F8(art(hat, 0), It_Kind_Kirby_FoxLaser as i32);
                ext::it_8026B3F8(art(hat, 1), It_Kind_Kirby_FoxBlaster as i32);
            }),
            Ft_Kind_Falco => with_hat!(21, |hat| {
                ext::it_8026B3F8(art(hat, 3), It_Kind_Kirby_FalcoLaser as i32);
                ext::it_8026B3F8(art(hat, 4), It_Kind_Kirby_FalcoBlaster as i32);
            }),
            Ft_Kind_Link => with_hat!(5, |hat| {
                ext::it_8026B3F8(art(hat, 0), It_Kind_Kirby_LinkArrow as i32);
                ext::it_8026B3F8(art(hat, 1), It_Kind_Kirby_LinkBow as i32);
            }),
            Ft_Kind_CLink => with_hat!(19, |hat| {
                ext::it_8026B3F8(art(hat, 0), It_Kind_Kirby_CLinkArrow as i32);
                ext::it_8026B3F8(art(hat, 1), It_Kind_Kirby_CLinkBow as i32);
            }),
            Ft_Kind_Mewtwo => with_hat!(15, |hat| {
                ext::it_8026B3F8(art(hat, 3), It_Kind_Kirby_MewtwoShadowBall as i32);
            }),
            Ft_Kind_Ness => with_hat!(7, |hat| {
                ext::it_8026B3F8(art(hat, 0), It_Kind_Kirby_NessPKFlush as i32);
                ext::it_8026B3F8(art(hat, 1), It_Kind_Kirby_NessPKFlush_Explode as i32);
            }),
            Ft_Kind_Pikachu => with_hat!(11, |hat| {
                ext::it_8026B3F8(art(hat, 0), It_Kind_Kirby_PikachuTJolt_Ground as i32);
                ext::it_8026B3F8(art(hat, 1), It_Kind_Kirby_PikachuTJolt_Air as i32);
            }),
            Ft_Kind_Pichu => with_hat!(22, |hat| {
                ext::it_8026B3F8(art(hat, 0), It_Kind_Kirby_PichuTJolt_Ground as i32);
                ext::it_8026B3F8(art(hat, 1), It_Kind_Kirby_PichuTJolt_Air as i32);
            }),
            Ft_Kind_Samus => with_hat!(12, |hat| {
                ext::it_8026B3F8(art(hat, 0), It_Kind_Kirby_SamusCharge as i32);
            }),
            Ft_Kind_Koopa => with_hat!(4, |hat| {
                ext::it_8026B3F8(art(hat, 0), It_Kind_Kirby_KoopaFlame as i32);
            }),
            Ft_Kind_GKoops => with_hat!(30, |hat| {
                ext::it_8026B3F8(art(hat, 0), It_Kind_Kirby_KoopaFlame as i32);
            }),
            Ft_Kind_Seak => with_hat!(6, |hat| {
                ext::it_8026B3F8(art(hat, 0), It_Kind_Kirby_SeakNeedleThrow as i32);
                ext::it_8026B3F8(art(hat, 1), It_Kind_Kirby_SeakNeedleHeld as i32);
            }),
            Ft_Kind_DrMario => with_hat!(20, |hat| {
                ext::it_8026B3F8(art(hat, 0), It_Kind_Kirby_DrMarioVitamin as i32);
            }),
            Ft_Kind_GameWatch => with_hat!(23, |hat| {
                ext::it_8026B3F8(art(hat, 5), It_Kind_Kirby_GameWatchChef as i32);
                ext::it_8026B3F8(art(hat, 6), It_Kind_Kirby_GameWatchChefPan as i32);
            }),
            Ft_Kind_Yoshi => with_hat!(13, |hat| {
                ext::it_8026B3F8(art(hat, 5), It_Kind_Kirby_YoshiEggLay as i32);
            }),
            _ => {}
        }
    }
}

/// `ftKb_SpecialN_800F190C(gobj, kind)`: reset the copy-ability state of Kirby's fighter vars.
pub fn ftKb_SpecialN_800F190C(gobj: *mut HSD_GObj, _kind: i32) {
    unsafe {
        let fp = fpp(gobj);
        let da = (*fp).dat_attrs as *const ftKb_DatAttrs;
        (*fp).u.kb.x68 = 9;
        (*fp).u.kb.x6C = 9;
        (*fp).u.kb.xC0 = null_mut();
        (*fp).u.kb.xC4 = 0;
        (*fp).u.kb.xC8 = 0.0;
        (*fp).u.kb.xCC = 0;
        (*fp).u.kb.xD0 = null_mut();
        (*fp).u.kb.x74 = null_mut();
        (*fp).u.kb.x78 = null_mut();
        (*fp).u.kb.ns_flash_gobj = null_mut();
        (*fp).u.kb.x80 = 0;
        (*fp).u.kb.x84 = (*da).specialn_kp_max_fuel;
        (*fp).u.kb.x88 = (*da).specialn_kp_flame_scale;
        (*fp).u.kb.x8C.x = 1.0;
        (*fp).u.kb.x8C.y = 1.0;
        (*fp).u.kb.x8C.z = 1.0;
        (*fp).u.kb.x98 = null_mut();
        (*fp).u.kb.x9C = 0;
        (*fp).u.kb.xA0 = null_mut();
        (*fp).u.kb.xA4 = null_mut();
        (*fp).u.kb.xA8 = 0;
        (*fp).u.kb.xAC = null_mut();
        (*fp).u.kb.xB0 = null_mut();
        (*fp).u.kb.xB4 = 0;
        (*fp).u.kb.xB8 = null_mut();
        (*fp).u.kb.xBC = 0;
        (*fp).u.kb.xD4 = 1;
        (*fp).u.kb.xD8 = 3;
        (*fp).u.kb.xDC = null_mut();
    }
}

/// `ftKb_SpecialN_800F19AC`: the hat's ability cleanup on death (Mewtwo uses 8010709C).
pub fn ftKb_SpecialN_800F19AC(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        match (*fp).u.kb.hat.kind {
            Ft_Kind_Popo => ext::ftKb_SpecialNIc_80108D04(gobj),
            Ft_Kind_Peach => ext::ftKb_SpecialNPe_8010C3F4(gobj),
            Ft_Kind_Fox | Ft_Kind_Falco => crate::tu::ftkirbyspecialfox::ftKb_SpecialNFx_800FDEE0(gobj),
            Ft_Kind_Link | Ft_Kind_CLink => {
                ext::ftKb_SpecialNLk800FB800(gobj);
                ext::ftKb_SpecialNLk800FB840(gobj);
            }
            Ft_Kind_Mewtwo => ext::ftKb_SpecialNMt_8010709C(gobj),
            Ft_Kind_Ness => crate::tu::ftkirbyspecialfox::ftKb_SpecialNNs_800FECE8(gobj),
            Ft_Kind_Samus => ext::ftKb_SpecialNSs_800FCD60(gobj),
            Ft_Kind_Seak => ext::ftKb_SpecialNSk_8010603C(gobj),
            Ft_Kind_Yoshi => {
                ext::ftKb_SpecialNYs_801093B4(gobj);
                ext::ftKb_SpecialNYs_8010A8BC(gobj);
            }
            Ft_Kind_Donkey => ext::ftKb_SpecialNPr_80100DE0(gobj),
            Ft_Kind_GameWatch => crate::tu::ftkirbyspecialgamewatch::ftKb_SpecialNGw_8010D0A8(gobj),
            _ => {}
        }
    }
}

/// `ftKb_SpecialN_800F1A8C`: as above for take-damage (Mewtwo uses 80107130).
pub fn ftKb_SpecialN_800F1A8C(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        match (*fp).u.kb.hat.kind {
            Ft_Kind_Popo => ext::ftKb_SpecialNIc_80108D04(gobj),
            Ft_Kind_Peach => ext::ftKb_SpecialNPe_8010C3F4(gobj),
            Ft_Kind_Fox | Ft_Kind_Falco => crate::tu::ftkirbyspecialfox::ftKb_SpecialNFx_800FDEE0(gobj),
            Ft_Kind_Link | Ft_Kind_CLink => {
                ext::ftKb_SpecialNLk800FB800(gobj);
                ext::ftKb_SpecialNLk800FB840(gobj);
            }
            Ft_Kind_Mewtwo => ext::ftKb_SpecialNMt_80107130(gobj),
            Ft_Kind_Ness => crate::tu::ftkirbyspecialfox::ftKb_SpecialNNs_800FECE8(gobj),
            Ft_Kind_Samus => ext::ftKb_SpecialNSs_800FCD60(gobj),
            Ft_Kind_Seak => ext::ftKb_SpecialNSk_8010603C(gobj),
            Ft_Kind_Yoshi => {
                ext::ftKb_SpecialNYs_801093B4(gobj);
                ext::ftKb_SpecialNYs_8010A8BC(gobj);
            }
            Ft_Kind_Donkey => ext::ftKb_SpecialNPr_80100DE0(gobj),
            Ft_Kind_GameWatch => crate::tu::ftkirbyspecialgamewatch::ftKb_SpecialNGw_8010D0A8(gobj),
            _ => {}
        }
    }
}

pub fn ftKb_Init_UnkMotionStates3(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        match (*fp).u.kb.hat.kind {
            Ft_Kind_Koopa | Ft_Kind_GKoops => ext::ftKb_SpecialNKp_800FA7D4(gobj),
            _ => {}
        }
    }
}

/// `ftKb_SpecialN_800F1BAC(gobj, kind, arg2)`: put on `kind`'s hat (SFX 0x222F3) or keep it (0x222F6).
pub fn ftKb_SpecialN_800F1BAC(gobj: *mut HSD_GObj, kind: i32, arg2: bool) {
    unsafe {
        let fp = fpp(gobj);
        if (*fp).u.kb.hat.kind as i32 != kind {
            // TARGET_PC: the hat archive table is zeroed before demo scenes; reload on demand.
            if (*hat_slot(kind as usize)).is_null() {
                ftKb_SpecialN_800EED50(kind, (*fp).x619_costume_id as i32);
            }
            (*fp).u.kb.hat.kind = kind as FighterKind;
            ftKb_SpecialN_800F190C(gobj, (*fp).u.kb.hat.kind as i32);
            if let Some(cb) = ftKb_Init_803C9CC8[((*fp).u.kb.hat.kind * 2) as usize] {
                cb(gobj);
            }
            ftKb_SpecialN_800F16D0(gobj, (*fp).u.kb.hat.kind as i32);
            if arg2 {
                melee_fighter::hooks::play_sfx(&mut *fp, 0x222F3, 0x7F, 0x40);
            }
        } else if arg2 {
            melee_fighter::hooks::play_sfx(&mut *fp, 0x222F6, 0x7F, 0x40);
        }
        (*fp).death3_cb = cb_death3();
        (*fp).death1_cb = cb_death1();
    }
}

/// `fn_800F1CA0`: hats whose ability is a held-down charge.
pub fn fn_800F1CA0(gobj: *mut HSD_GObj) -> bool {
    unsafe {
        let fp = fpp(gobj);
        matches!((*fp).u.kb.hat.kind, Ft_Kind_Captain | Ft_Kind_Seak | Ft_Kind_Popo | Ft_Kind_Nana | Ft_Kind_Samus)
    }
}

pub fn ftKb_SpecialN_800F1CD8(gobj: *mut HSD_GObj) -> i32 {
    if fn_800F1CA0(gobj) {
        return 1;
    }
    0
}

static mut KIRBY_LIKE_CACHE: [i8; Ft_Kind_Max as usize] = [0; Ft_Kind_Max as usize];

/// `ftKb_MexKirbyLike(kind)` (TARGET_PC): is m-ex kind `kind` a Kirby clone? Cached per kind once m-ex answers.
pub fn ftKb_MexKirbyLike(kind: i32) -> i32 {
    unsafe {
        if kind < Ft_Kind_Mex0 as i32 || kind >= Ft_Kind_Max as i32 {
            return 0;
        }
        let k = kind as usize;
        if KIRBY_LIKE_CACHE[k] == 0 {
            let internal = ext::Mex_InternalForPortKind(kind);
            if internal < 0 {
                return 0;
            }
            KIRBY_LIKE_CACHE[k] = if ext::Mex_FtBaseKind(internal) == Ft_Kind_Kirby as i32 { 1 } else { -1 };
            if KIRBY_LIKE_CACHE[k] > 0 {
                eprintln!("ftKb: kind {} (m-ex internal {}) is Kirby-based - takes Kirby's code paths", kind, internal);
            }
        }
        (KIRBY_LIKE_CACHE[k] > 0) as i32
    }
}

/// `FTKB_IS_KIRBY(k)` (ft/forward.h, TARGET_PC).
#[inline]
fn ftkb_is_kirby(k: FighterKind) -> bool {
    k == Ft_Kind_Kirby || (k >= Ft_Kind_Mex0 && k < Ft_Kind_Max && ftKb_MexKirbyLike(k as i32) != 0)
}

/// `ftKb_SpecialN_800F1D24`: per-frame wall-bump dust timer for Kirby-based fighters.
pub fn ftKb_SpecialN_800F1D24(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if ftkb_is_kirby((*fp).kind) {
            let x60 = (*fp).u.kb.x60;
            if x60 != 0 {
                (*fp).u.kb.x60 = x60 - 1;
            } else {
                ftKb_SpecialN_800F1DAC(gobj);
            }
            let f = (*fp).coll_data.env_flags;
            if (f & 0x800) != 0 || (f & 0x20) != 0 || (f & 0x4000) != 0 || (f & 0x10000) != 0 {
                (*fp).u.kb.x60 = 5;
            }
        }
    }
}

/// `ftKb_SpecialN_800F1DAC`: spawn the bump effect (0x49E) on the first frame of touching a wall/ceiling/floor.
pub fn ftKb_SpecialN_800F1DAC(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let coll = &(*fp).coll_data;
        let mut vec = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        let head = &raw mut (*fpp(gobj)).x60C as *mut c_void;
        let root = (*(*fp).parts).joint;
        if (coll.env_flags & 0x800) != 0 && (coll.prev_env_flags & 0xFC0) == 0 {
            vec.x = coll.ecb.left.x;
            vec.y = coll.ecb.left.y;
            vec.z = 0.0;
            ext::efAsync_Spawn(gobj, head, 2, 0x49E, root, &mut vec);
            return;
        }
        if (coll.env_flags & 0x20) != 0 && (coll.prev_env_flags & 0x3F) == 0 {
            vec.x = coll.ecb.right.x;
            vec.y = coll.ecb.right.y;
            vec.z = 0.0;
            ext::efAsync_Spawn(gobj, head, 2, 0x49E, root, &mut vec);
            return;
        }
        if (coll.env_flags & 0x4000) != 0 && (coll.prev_env_flags & 0x6000) == 0 {
            vec.x = 0.0;
            vec.y = coll.ecb.top.y;
            vec.z = 0.0;
            ext::efAsync_Spawn(gobj, head, 2, 0x49E, root, &mut vec);
            return;
        }
        if (coll.env_flags & 0x10000) != 0 && (coll.prev_env_flags & 0x18000) == 0 {
            vec.x = 0.0;
            vec.y = coll.ecb.bottom.y;
            vec.z = 0.0;
            ext::efAsync_Spawn(gobj, head, 2, 0x49E, root, &mut vec);
        }
    }
}

/// `ftKb_SpecialN_800F1F1C(gobj, pos)`: spawn the bump effect at `pos` for Kirby-based fighters.
pub fn ftKb_SpecialN_800F1F1C(gobj: *mut HSD_GObj, pos: *mut Vec3) {
    unsafe {
        let fp = fpp(gobj);
        if ftkb_is_kirby((*fp).kind) {
            ext::efAsync_Spawn(gobj, &raw mut (*fp).x60C as *mut c_void, 2, 0x49E, (*(*fp).parts).joint, pos);
        }
    }
}

/// `ftKb_MexCopyKindHat(dst, src, internal)` (TARGET_PC): fill fighter kind `dst`'s Kirby copy-ability
/// callbacks from MxDt.dat's MexData.kirby_function, for m-ex INTERNAL kind `internal`. `src` is the
/// clone base; an empty m-ex row stays empty (NULL means "run Kirby's own inhale").
pub fn ftKb_MexCopyKindHat(dst: i32, src: i32, internal: i32) {
    unsafe {
        let (d, s) = (dst as usize, src as usize);
        ftKb_Init_803C9FC8[d] = ftKb_Init_803C9FC8[s];
        ftKb_Init_803CB3E8[d] = ext::Mex_KirbyCostumes(internal) as *mut Fighter_CostumeStrings;
        if !ftKb_Init_803CB3E8[d].is_null() && ftKb_Init_803C9FC8[d].is_null() {
            eprintln!("gw: kind {} has m-ex Kirby hat costumes but no runtime array - ignored", dst);
            ftKb_Init_803CB3E8[d] = null_mut();
        }
        let f = |slot: i32| -> HSD_GObjEvent { core::mem::transmute::<*mut c_void, HSD_GObjEvent>(ext::Mex_KirbyFunc(slot, internal)) };
        ftKb_Init_803C9CC8[d * 2] = f(0);
        ftKb_Init_803C9CC8[d * 2 + 1] = f(1);
        ftKb_Init_803C9DD0[d] = f(2);
        ftKb_Init_803C9E54[d] = f(3);
    }
}
