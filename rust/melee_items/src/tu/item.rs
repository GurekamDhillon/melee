//! Port of src/melee/it/item.c: item creation and teardown, the per-frame think callbacks (animation, physics,
//! collision, hit reactions, hitlag), state changes, pickup / drop / throw and the item SFX slots, including the
//! PC build's Geno / m-ex branches. Callees of other TUs go through `ext`.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_assignments, unused_variables, unused_mut, clippy::missing_safety_doc)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;

// BEGIN GENERATED EXT (rust/melee_items/gen/mtu/gen_tu.py; the callees of the C TU)
pub mod ext {
    #![allow(unreachable_code, non_upper_case_globals, non_snake_case, unused_imports, dead_code)]
    use super::*;
    use core::ffi::c_void;
    macro_rules! ext_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)? $(=> $fb:expr)?;)*) => {
            pub mod hook {
                use super::*;
                $( pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } {
                    Some(f) => f($($a),*),
                    None => { $( return $fb; )? panic!(concat!("TODO: ", stringify!($name), " is not ported")) }
                }
            } )*
        };
    }
    ext_fns! {
    fn Camera_80029044(a0: i32) -> *mut CmSubject;
    fn Camera_800290D4(a0: *mut CmSubject);
    fn GObj_Create(a0: u16, a1: u8, a2: u8) -> *mut HSD_GObj;
    fn GObj_InitUserData(a0: *mut HSD_GObj, a1: u8, a2: Option<unsafe extern "C" fn(*mut c_void)>, a3: *mut c_void);
    fn GObj_SetupGXLink(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u8, a3: u32);
    fn Geno_ArticleDesc(a0: i32) -> *mut c_void;
    fn Geno_ArticleLogic(a0: i32) -> *mut c_void;
    fn Geno_ItemActive(a0: i32) -> i32;
    fn Geno_ItemCap(a0: i32) -> i32;
    fn Geno_ItemDesc(a0: i32) -> *mut c_void;
    fn Geno_ItemDraw(a0: *mut HSD_GObj, a1: i32);
    fn Geno_ItemFamilyEvent(a0: *mut HSD_GObj, a1: i32, a2: i32);
    fn Geno_ItemLogic(a0: i32) -> *mut c_void;
    fn HSD_GObjFree(a0: *mut HSD_GObj);
    fn HSD_GObjObject_80390A70(a0: *mut HSD_GObj, a1: u8, a2: *mut c_void);
    fn HSD_GObj_SetupProc(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: u8) -> *mut HSD_GObjProc => unsafe { melee_world::tu_ext::HSD_GObj_SetupProc(a0, a1, a2) };
    fn HSD_JObjAddAnim(a0: *mut HSD_JObj, a1: *mut HSD_AnimJoint, a2: *mut HSD_MatAnimJoint, a3: *mut HSD_ShapeAnimJoint);
    fn HSD_JObjAlloc() -> *mut HSD_JObj => unsafe { melee_runtime::tu::jobj::HSD_JObjAlloc() };
    fn HSD_JObjAnimAll(a0: *mut HSD_JObj);
    fn HSD_JObjGetDObj(a0: *mut HSD_JObj) -> *mut HSD_DObj => unsafe { crate::tu::jobj::HSD_JObjGetDObj(a0) as _ };
    fn HSD_JObjLoadJoint(a0: *mut HSD_Joint) -> *mut HSD_JObj => unsafe { melee_runtime::tu::jobj::HSD_JObjLoadJoint(a0) };
    fn HSD_JObjRemoveAnimAll(a0: *mut HSD_JObj);
    fn HSD_JObjReqAnimAll(a0: *mut HSD_JObj, a1: f32);
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) => unsafe { crate::tu::jobj::HSD_JObjSetMtxDirtySub(a0) };
    fn HSD_ObjAlloc(a0: *mut _HSD_ObjAllocData) -> *mut c_void => unsafe { melee_runtime::tu::objalloc::HSD_ObjAlloc(a0 as _) };
    fn HSD_ObjAllocInit(a0: *mut _HSD_ObjAllocData, a1: u32, a2: u32) => unsafe { melee_runtime::tu::objalloc::HSD_ObjAllocInit(a0 as _, a1 as _, a2) };
    fn HSD_ObjFree(a0: *mut _HSD_ObjAllocData, a1: *mut c_void) => unsafe { melee_runtime::tu::objalloc::HSD_ObjFree(a0 as _, a1) };
    fn Mex_ItemCustomDesc(a0: i32) -> *mut c_void;
    fn Mex_ItemCustomLogic(a0: i32) -> *mut c_void;
    fn Mex_ItemNotInitialized(a0: i32);
    fn Mex_SpawnArticleKind(a0: i32, a1: i32, a2: i32) -> i32;
    fn Mex_SsmForPortKind(a0: i32) -> i32;
    fn PSMTXIdentity(a0: *mut c_void) => unsafe { *(a0 as *mut melee_types::Mtx) = melee_runtime::tu::mtx_pc::PSMTXIdentity() };
    fn PSVECAdd(a0: *mut Vec, a1: *mut Vec, a2: *mut Vec) => unsafe { *(a2 as *mut _) = melee_runtime::tu::mtx_pc::PSVECAdd(&*(a0 as *const _), &*(a1 as *const _)) };
    fn ScriptGame_EnemyDestroyed(a0: *mut HSD_GObj) -> i32;
    fn ScriptGame_EnemyReceived(a0: *mut HSD_GObj, a1: *mut HSD_GObj, a2: i32);
    fn ScriptGame_HitRuleBaseDamage(a0: *mut HitCapsule) -> f32;
    fn ScriptGame_HitRuleRetire(a0: *mut c_void, a1: u32);
    fn Stage_GetBlastZoneBottomOffset() -> f32 => unsafe { melee_world::tu::stage::Stage_GetBlastZoneBottomOffset() };
    fn Stage_GetBlastZoneLeftOffset() -> f32 => unsafe { melee_world::tu::stage::Stage_GetBlastZoneLeftOffset() };
    fn Stage_GetBlastZoneRightOffset() -> f32 => unsafe { melee_world::tu::stage::Stage_GetBlastZoneRightOffset() };
    fn db_80225D64(a0: *mut HSD_GObj, a1: *mut HSD_GObj) => unsafe { crate::tu::dbitem::db_80225D64(a0, a1) };
    fn db_80225DD8(a0: *mut HSD_GObj, a1: *mut HSD_GObj) => unsafe { crate::tu::dbitem::db_80225DD8(a0, a1) };
    fn db_ShowItemPickupRange() -> u32 => unsafe { crate::tu::dbitem::db_ShowItemPickupRange() };
    fn efAsync_QueueClear(a0: *mut c_void);
    fn efAsync_QueueFlush(a0: *mut HSD_GObj, a1: *mut c_void);
    fn efLib_DestroyAll(a0: *mut HSD_GObj) => unsafe { melee_world::tu::eflib::efLib_DestroyAll(a0) };
    fn ftLib_80086724(a0: *mut HSD_GObj, a1: *mut HSD_GObj);
    fn ftLib_80086764(a0: *mut HSD_GObj);
    fn ftLib_800867A0(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::tu::ftlib::ftLib_800867A0(a0, a1) as _ };
    fn ftLib_800867CC(a0: *mut HSD_GObj) -> *mut HSD_GObj => unsafe { crate::tu::ftlib::ftLib_800867CC(a0) };
    fn ftLib_80086960(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::ftlib::ftLib_80086960(a0) as _ };
    fn ftLib_80086EB4(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::ftlib::ftLib_80086EB4(a0) };
    fn ftLib_8008702C(a0: i32);
    fn ftLib_80087050(a0: i32);
    fn ftLib_GetKind(a0: *mut HSD_GObj) -> u32;
    fn gm_8016AE80() -> i32;
    fn gm_8017E068() -> i32;
    fn gm_8018841C() -> i32;
    fn grLib_801C9E60(a0: *mut Vec) -> i32 => unsafe { melee_world::tu::grlib::grLib_801C9E60(a0 as _) as _ };
    fn hsdChangeClass(a0: *mut c_void, a1: *mut c_void) -> i32 => unsafe { melee_runtime::tu::class::hsdChangeClass(a0, a1) as _ };
    fn itGetTeamId(a0: *mut HSD_GObj) -> u8 => unsafe { crate::tu::it_26B1::itGetTeamId(a0) };
    fn it_8026B390(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b390(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B424(a0: i32) -> f32 => unsafe { crate::tu::it_26B1::it_8026B424(a0) };
    fn it_8026B6C8(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b6c8(&*((*a0).user_data as *mut Item)) as _ };
    fn it_8026BDCC(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026bdcc(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026C368(a0: *mut HSD_GObj) => unsafe { crate::tu::it_26B1::it_8026C368(a0) };
    fn it_8026D604(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026D604(a0) };
    fn it_8026EECC(a0: *mut HSD_GObj, a1: i32) => unsafe { crate::tu::itdraw::it_8026EECC(a0, a1) };
    fn it_8026F9A0() => unsafe { crate::tu::itcoll::it_8026F9A0() };
    fn it_802701BC(a0: *mut HSD_GObj) => unsafe { crate::tu::itcoll::it_802701BC(a0) };
    fn it_802703E8(a0: *mut HSD_GObj) => unsafe { crate::tu::itcoll::it_802703E8(a0) };
    fn it_802706D0(a0: *mut HSD_GObj) => unsafe { crate::tu::itcoll::it_802706D0(a0) };
    fn it_80270E30(a0: *mut HSD_GObj) => unsafe { crate::tu::itcoll::it_80270E30(a0) };
    fn it_8027137C(a0: *mut HSD_GObj) => unsafe { crate::tu::itcoll::it_8027137C(a0) };
    fn it_8027146C(a0: *mut HSD_GObj) => unsafe { crate::tu::itcoll::it_8027146C(a0) };
    fn it_802714C0(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_802714c0(&mut *((*a0).user_data as *mut Item)) };
    fn it_80271508(a0: *mut HSD_GObj, a1: u32) => unsafe { crate::itlib::it_80271508(&mut *((*a0).user_data as *mut Item), a1) };
    fn it_8027163C(a0: *mut HSD_GObj) => unsafe { crate::tu::itcoll::it_8027163C(a0) };
    fn it_80271A58(a0: *mut HSD_GObj) => unsafe { crate::tu::itcoll::it_80271A58(a0) };
    fn it_802721B8(a0: *mut HSD_GObj) => unsafe { crate::tu::itcoll::it_802721B8(a0) };
    fn it_80272280(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_80272280(&mut *((*a0).user_data as *mut Item)) };
    fn it_80272298(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_80272298(&mut *((*a0).user_data as *mut Item)) };
    fn it_802722B0(a0: *mut HSD_GObj) => unsafe { crate::tu::itcoll::it_802722B0(a0) };
    fn it_80272304(a0: *mut HSD_GObj) => unsafe { crate::tu::itcoll::it_80272304(a0) };
    fn it_80272460(a0: *mut HitCapsule, a1: u32, a2: *mut HSD_GObj) => unsafe { crate::tu::itcoll::it_80272460(a0, a1, a2) };
    fn it_802725D4(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_802725D4(a0) };
    fn it_802728C8(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_802728C8(a0) };
    fn it_80272D1C(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib3::it_80272d1c(a0 as _) as _ };
    fn it_80273168(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80273168(a0) };
    fn it_802731A4(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_802731A4(a0) };
    fn it_802731E0(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_802731E0(a0) };
    fn it_8027327C(a0: *mut HSD_GObj, a1: i32, a2: i32) => unsafe { crate::tu::it_2725::it_8027327C(a0, a1, a2) };
    fn it_8027346C(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8027346c(&mut *((*a0).user_data as *mut Item)) };
    fn it_80273484(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_80273484(&mut *((*a0).user_data as *mut Item)) };
    fn it_80273500(a0: *mut HSD_GObj, a1: *mut Vec) => unsafe { crate::tu::it_2725::it_80273500(a0, a1 as _) };
    fn it_80273748(a0: *mut HSD_GObj, a1: *mut Vec, a2: *mut Vec) => unsafe { crate::tu::it_2725::it_80273748(a0, a1 as _, a2 as _) };
    fn it_80273B50(a0: *mut HSD_GObj, a1: *mut Vec) => unsafe { crate::tu::it_2725::it_80273B50(a0, a1 as _) };
    fn it_80274198(a0: *mut HSD_GObj, a1: i32) => unsafe { crate::tu::it_2725::it_80274198(a0, a1) };
    fn it_802741F4(a0: *mut HSD_GObj, a1: i32) => unsafe { crate::tu::it_2725::it_802741F4(a0, a1) };
    fn it_802742F4(a0: *mut HSD_GObj, a1: *mut HSD_GObj, a2: u32) => unsafe { crate::tu::it_2725::it_802742F4(a0, a1, a2) };
    fn it_80274658(a0: *mut HSD_GObj, a1: f32) -> i32 => unsafe { crate::itlib3::it_80274658(&mut *((*a0).user_data as *mut Item), a1) };
    fn it_80274740(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80274740(a0) };
    fn it_80274A64(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80274A64(a0) };
    fn it_80274C78(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_80274c78(&*((*a0).user_data as *mut Item)) as _ };
    fn it_80274DAC(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_80274dac(&mut *((*a0).user_data as *mut Item)) };
    fn it_80274EE8() => unsafe { crate::tu::it_2725::it_80274EE8() };
    fn it_80274EF8(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_80274ef8(&mut *((*a0).user_data as *mut Item)) };
    fn it_80275158(a0: *mut HSD_GObj, a1: f32) => unsafe { crate::tu::it_2725::it_80275158(a0, a1) };
    fn it_80275390(a0: *mut HSD_GObj) => unsafe { crate::itlib3::it_80275390(&mut *((*a0).user_data as *mut Item)) };
    fn it_802753BC(a0: *mut HSD_GObj, a1: i16) => unsafe { crate::itlib::it_802753bc(&mut *((*a0).user_data as *mut Item), a1) };
    fn it_802753DC(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_802753dc(&mut *((*a0).user_data as *mut Item)) };
    fn it_80275474(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_80275474(&mut *((*a0).user_data as *mut Item)) };
    fn it_802754D4(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_802754d4(&mut *((*a0).user_data as *mut Item)) };
    fn it_80275504(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_80275504(&mut *((*a0).user_data as *mut Item)) };
    fn it_802756D0(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_802756d0(&mut *((*a0).user_data as *mut Item)) };
    fn it_802756E0(a0: *mut HSD_GObj) => unsafe { crate::itlib3::it_802756e0(&mut *((*a0).user_data as *mut Item)) };
    fn it_8027574C(a0: *mut HSD_GObj) => unsafe { crate::itkinds6::it_8027574c(&mut *((*a0).user_data as *mut Item)) };
    fn it_80275E98(a0: *mut HSD_GObj, a1: *mut SpawnItem) => unsafe { crate::tu::itmaplib::it_80275E98(a0, a1) };
    fn it_802762B0(a0: *mut Item) => unsafe { crate::item::it_802762b0(&mut *a0) };
    fn it_802762BC(a0: *mut Item) => unsafe { crate::item::it_802762bc(&mut *a0) };
    fn it_8027737C(a0: *mut HSD_GObj, a1: *mut Vec) => unsafe { crate::tu::itmaplib::it_8027737C(a0, a1 as _) };
    fn it_8027870C(a0: i32) => unsafe { crate::tu::iteffect::it_8027870C(a0) };
    fn it_802799E4(a0: *mut HSD_GObj) => unsafe { crate::tu::itanimlist::it_802799E4(a0 as _) };
    fn it_80279B64(a0: *mut Item) => unsafe { crate::tu::itanimlist::it_80279B64(a0) };
    fn it_80279B88(a0: *mut Item, a1: i32, a2: i32) => unsafe { crate::tu::itanimlist::it_80279B88(a0, a1, a2) };
    fn it_80279BBC(a0: *mut Item) => unsafe { crate::tu::itanimlist::it_80279BBC(a0) };
    fn it_80279BE0(a0: *mut HSD_GObj) => unsafe { crate::tu::itanimlist::it_80279BE0(a0 as _) };
    fn it_8027B0C4(a0: *mut HSD_GObj, a1: *mut SpawnItem) => unsafe { crate::tu::it_279C::it_8027B0C4(a0, a1) };
    fn lbAudioAx_800233EC(a0: i32) -> i32 => unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_800233EC(a0) };
    fn lbAudioAx_800236B8(a0: i32) -> i32 => unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_800236B8(a0) };
    fn lbAudioAx_800237A8(a0: i32, a1: i32, a2: i32) -> i32 => unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_800237A8(a0, a1, a2) };
    fn lbAudioAx_80026510(a0: *mut HSD_GObj) -> i32 => unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80026510(a0) as _ };
    fn lbAudioAx_800265C4(a0: *mut HSD_GObj, a1: i32) -> i32 => unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_800265C4(a0, a1) as _ };
    fn lbGetJObjCurrFrame(a0: *mut HSD_JObj) -> f32 => unsafe { melee_runtime::tu::lb_00B0::lbGetJObjCurrFrame(a0) };
    fn lb_8000B804(a0: *mut HSD_JObj, a1: *mut HSD_Joint) => unsafe { melee_runtime::tu::lb_00B0::lb_8000B804(a0, a1) };
    fn lb_8000BA0C(a0: *mut HSD_JObj, a1: f32) => unsafe { melee_runtime::tu::lb_00B0::lb_8000BA0C(a0, a1) };
    fn lb_8000FD18(a0: *mut DynamicsDesc) => unsafe { melee_runtime::tu::lb_00F9::lb_8000FD18(a0) };
    fn lb_8000FD48(a0: *mut HSD_JObj, a1: *mut DynamicsDesc, a2: u32) => unsafe { melee_runtime::tu::lb_00F9::lb_8000FD48(a0, a1, a2 as _) };
    fn lb_8001044C(a0: *mut DynamicsDesc, a1: *mut c_void, a2: i32, a3: f32, a4: i32, a5: u32, a6: i32, a7: i32) => unsafe { melee_runtime::tu::lb_00F9::lb_8001044C(a0, a1, a2, a3, a4 != 0, a5, a6, a7 != 0) };
    fn lb_80011710(a0: *mut DynamicsDesc, a1: *mut DynamicsDesc) => unsafe { melee_runtime::tu::lb_00F9::lb_80011710(a0, a1) };
    fn mpCollSetFacingDir(a0: *mut CollData, a1: i32) => unsafe { melee_world::tu::mpcoll::mpCollSetFacingDir(a0, a1) };
    fn mpGetSpeed(a0: i32, a1: *mut Vec, a2: *mut Vec) -> i32 => unsafe { melee_world::tu::mplib::mpGetSpeed(a0, a1, a2) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn Geno_ItemDraw(a0: *mut HSD_GObj, a1: i32) { super::Geno_ItemDraw(a0, a1) }
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
        pub unsafe extern "C" fn it_8026EECC(a0: *mut HSD_GObj, a1: i32) { super::it_8026EECC(a0, a1) }
    }
    // Data defined in other TUs (the tests set them).
    pub static mut HSD_GObj_JObjKind: u8 = unsafe { core::mem::zeroed() };
    pub static mut it_803F1418: [sdata_ItemGXLink; 43] = unsafe { core::mem::zeroed() };
    pub static mut it_803F14C4: [ItemLogicTable; 43] = unsafe { core::mem::zeroed() };
    pub static mut it_803F2310: [sdata_ItemGXLink; 47] = unsafe { core::mem::zeroed() };
    pub static mut it_803F23CC: [ItemLogicTable; 47] = unsafe { core::mem::zeroed() };
    pub static mut it_803F2F28: [sdata_ItemGXLink; 118] = unsafe { core::mem::zeroed() };
    pub static mut it_803F3100: [ItemLogicTable; 118] = unsafe { core::mem::zeroed() };
    pub static mut it_803F4CA8: [sdata_ItemGXLink; 30] = unsafe { core::mem::zeroed() };
    pub static mut it_803F4D20: [ItemLogicTable; 30] = unsafe { core::mem::zeroed() };
    pub static mut it_804A0E30: RandomItemSpawner = unsafe { core::mem::zeroed() };
    pub static mut it_804A0E50: ItemPickTable = unsafe { core::mem::zeroed() };
    pub static mut it_804A0E60: ItemPickTable = unsafe { core::mem::zeroed() };
    pub static mut it_804A0F60: [*mut Article; 30] = unsafe { core::mem::zeroed() };
    pub static mut it_804D6D00: i8 = unsafe { core::mem::zeroed() };
    pub static mut it_804D6D04: *mut Fighter_804D653C_t = unsafe { core::mem::zeroed() };
    pub static mut it_804D6D08: i32 = unsafe { core::mem::zeroed() };
    pub static mut it_804D6D0C: i32 = unsafe { core::mem::zeroed() };
    pub static mut it_804D6D10: u32 = unsafe { core::mem::zeroed() };
    pub static mut it_804D6D14: u32 = unsafe { core::mem::zeroed() };
    pub static mut it_804D6D24: *mut *mut Article = unsafe { core::mem::zeroed() };
    pub static mut it_804D6D28: *mut ItemCommonData = unsafe { core::mem::zeroed() };
    pub static mut it_804D6D30: *mut *mut Article = unsafe { core::mem::zeroed() };
    pub static mut it_804D6D38: *mut *mut Article = unsafe { core::mem::zeroed() };
    pub static mut item_data: *mut Item = unsafe { core::mem::zeroed() };
    pub static mut result: i32 = unsafe { core::mem::zeroed() };
}
// END GENERATED EXT

pub static mut it_mobj: [u8; 64] = [0; 64];
const SFX_NONE: i32 = -1;
const M_PI: f64 = 3.14159265358979323846;

pub static mut item_alloc_data: HSD_ObjAllocData = unsafe { core::mem::zeroed() };
pub static mut item_dynamic_bones_alloc_data: HSD_ObjAllocData = unsafe { core::mem::zeroed() };
pub static mut item_link_alloc_data: HSD_ObjAllocData = unsafe { core::mem::zeroed() };
pub static mut Item_804A0C64: HSD_ObjAllocUnk = unsafe { core::mem::zeroed() };
pub static mut Item_804A0CCC: Item_FtTrack = unsafe { core::mem::zeroed() };
pub static mut Item_804A0E24: PokemonSelectionState = unsafe { core::mem::zeroed() };

#[inline(always)]
unsafe fn get_item(gobj: *mut HSD_GObj) -> *mut Item {
    (*gobj).user_data as *mut Item
}

#[inline(always)]
unsafe fn GetItemData(gobj: *mut HSD_GObj) -> *mut Item {
    (*gobj).user_data as *mut Item
}

#[inline(always)]
unsafe fn get_jobj(gobj: *mut HSD_GObj) -> *mut HSD_JObj {
    (*gobj).hsd_obj as *mut HSD_JObj
}

#[inline(always)]
unsafe fn HSD_JObjGetChild(jobj: *mut HSD_JObj) -> *mut HSD_JObj {
    if jobj.is_null() { null_mut() } else { (*jobj).child }
}
#[inline(always)]
unsafe fn HSD_JObjGetNext(jobj: *mut HSD_JObj) -> *mut HSD_JObj {
    if jobj.is_null() { null_mut() } else { (*jobj).next }
}
#[inline(always)]
unsafe fn HSD_JObjGetParent(jobj: *mut HSD_JObj) -> *mut HSD_JObj {
    if jobj.is_null() { null_mut() } else { (*jobj).parent }
}

/// `HSD_JObjMtxIsDirty` (header inline).
#[inline(always)]
unsafe fn HSD_JObjMtxIsDirty(jobj: *mut HSD_JObj) -> bool {
    assert!(!jobj.is_null(), "HSD_ASSERT(564, jobj)");
    (*jobj).flags & JOBJ_USER_DEF_MTX as u32 == 0 && (*jobj).flags & JOBJ_MTX_DIRTY as u32 != 0
}

/// `HSD_JObjSetMtxDirty` (macro).
#[inline(always)]
pub unsafe fn HSD_JObjSetMtxDirty(jobj: *mut HSD_JObj) {
    if !jobj.is_null() && !HSD_JObjMtxIsDirty(jobj) {
        ext::HSD_JObjSetMtxDirtySub(jobj);
    }
}

#[inline(always)]
unsafe fn HSD_JObjSetRotationY(jobj: *mut HSD_JObj, y: f32) {
    assert!(!jobj.is_null(), "HSD_ASSERT(660, jobj)");
    assert!((*jobj).flags & JOBJ_USE_QUATERNION as u32 == 0, "HSD_ASSERT(661, !(jobj->flags & JOBJ_USE_QUATERNION))");
    (*jobj).rotate.y = y;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
        HSD_JObjSetMtxDirty(jobj);
    }
}

#[inline(always)]
unsafe fn HSD_JObjSetScale(jobj: *mut HSD_JObj, scale: *mut Vec3) {
    assert!(!jobj.is_null(), "HSD_ASSERT(760, jobj)");
    assert!(!scale.is_null(), "HSD_ASSERT(761, scale)");
    (*jobj).scale = *scale;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
        HSD_JObjSetMtxDirty(jobj);
    }
}

#[inline(always)]
unsafe fn HSD_JObjSetTranslate(jobj: *mut HSD_JObj, translate: *mut Vec3) {
    assert!(!jobj.is_null(), "HSD_ASSERT(916, jobj)");
    assert!(!translate.is_null(), "HSD_ASSERT(917, translate)");
    (*jobj).translate = *translate;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
        HSD_JObjSetMtxDirty(jobj);
    }
}

/// `GENO_ART_IS_KIND` (pc/geno/geno.h).
#[inline(always)]
fn geno_art_is_kind(k: i32) -> bool {
    const GENO_ART_KIND_BASE: i32 = 0x1000;
    const GENO_ART_KIND_END: i32 = 0x1000 + 32 * 8 + 32 * 8;
    const GENO_ART_EXTRA_BASE: i32 = 0x20000;
    const GENO_ART_EXTRA_END: i32 = 0x20000 + (65535 - 32) * 16;
    (k >= GENO_ART_KIND_BASE && k < GENO_ART_KIND_END) || (k >= GENO_ART_EXTRA_BASE && k < GENO_ART_EXTRA_END)
}
const GENO_ITEM_KIND_BASE: i32 = 0x1000 + 32 * 8 + 32 * 8;
const GENO_ITEM_KIND_END: i32 = GENO_ITEM_KIND_BASE + 64;

/// Check if items are enabled
pub unsafe extern "C" fn Item_80266F3C() -> bool {
    if ext::gm_8016AE80() != -1 {
        return true;
    }
    false
}

/// Check to load ItCo.dat/usd
pub unsafe extern "C" fn Item_80266F70() {
    ext::it_8027870C(Item_80266F3C() as i32);
}

/// ItCo prefunction with 0
pub unsafe extern "C" fn Item_80266FA8() {
    ext::it_8027870C(0);
}

/// Init item struct?
pub unsafe extern "C" fn Item_80266FCC() {
    ext::HSD_ObjAllocInit(&raw mut item_alloc_data, core::mem::size_of::<Item>() as u32, 4);
    ext::HSD_ObjAllocInit(&raw mut item_dynamic_bones_alloc_data, core::mem::size_of::<DynamicBoneTable>() as u32, 4);
    ext::HSD_ObjAllocInit(&raw mut item_link_alloc_data, core::mem::size_of::<ItemLink>() as u32, 4);

    let c = &raw mut Item_804A0C64;
    (*c).x0 = 0;
    (*c).x8 = 0;
    (*c).x10 = 0;
    (*c).x18 = 0;
    (*c).x1C = 0;
    (*c).x24 = 0;
    (*c).x2C = 0;
    (*c).x34 = 0;
    (*c).x3C = 0;
    (*c).x40 = 0;
    (*c).x48 = 0;
    (*c).x50 = 0;
    (*c).x58 = 0;
    (*c).x60 = 0;

    let d = ext::it_804D6D28;
    (*c).x4 = (*d).x0 as _;
    (*c).xC = (*d).x4 as _;
    (*c).x14 = (*d).x8 as _;
    (*c).x20 = (*d).xC as _;
    (*c).x28 = (*d).x10 as _;
    (*c).x30 = (*d).x14 as _;
    (*c).x38 = (*d).x18 as _;
    (*c).x44 = (*d).x1C as _;
    (*c).x4C = (*d).x24 as _;
    (*c).x54 = (*d).x20 as _;
    (*c).x5C = (*d).x28 as _;
    (*c).x64 = (*d).x148 as _;

    Item_804A0CCC.x154.anon_0.set_b0(1);
    Item_804A0CCC.count = 1;

    Item_804A0E24.last_kind = It_Kind_Unselected;
    Item_804A0E24.previous_kind = It_Kind_Unselected;
    Item_804A0E24.rare_spawned = 0;

    ext::it_804D6D00 = -1;
    ext::it_804D6D14 = 1;
    ext::it_804D6D10 = 0;
    ext::it_804D6D0C = 0;
    ext::it_804D6D08 = 0;

    ext::it_804A0E60.x8 = 0;
    ext::it_804A0E50.x8 = 0;
    ext::it_804A0E30.x4.x8 = 0;
    ext::it_804A0E60.size = 0;
    ext::it_804A0E50.size = 0;
    ext::it_804A0E30.x4.size = 0;
}

pub unsafe extern "C" fn ItUnkHoldKind(gobj: *mut HSD_GObj) {
    let it = (*gobj).user_data as *mut Item;
    match (*it).hold_kind {
        4 | 5 | 6 | 7 => {
            let temp_r3 = ext::gm_8017E068();
            if temp_r3 >= 0 {
                (*it).xC3C = (*ext::it_804D6D28).x80_float[temp_r3 as usize];
            }
        }
        _ => {}
    }
}

pub unsafe extern "C" fn HSD_JObjSetScaleItem(it: *mut Item, jobj: *mut HSD_JObj, scl: *mut Vec3) {
    (*scl).x = (*it).scl;
    (*scl).y = (*scl).x;
    (*scl).z = (*scl).y;
    HSD_JObjSetScale(jobj, scl);
}

pub unsafe extern "C" fn HSD_JObjSetFacingDirItem(jobj: *mut HSD_JObj, it: *mut Item) {
    if (*it).xDC8_word.flags.x19() == 1 {
        HSD_JObjSetRotationY(jobj, (M_PI / 2.0 * (*it).facing_dir as f64) as f32);
    }
}

/// Initialize item coordinates?
pub unsafe extern "C" fn Item_80267130(gobj: *mut HSD_GObj, spawnItem: *mut SpawnItem) {
    let item_data = get_item(gobj);
    let model = get_jobj(gobj);

    (*item_data).pos = (*spawnItem).prev_pos;
    (*item_data).facing_dir = (*spawnItem).facing_dir;
    (*item_data).init_facing_dir = (*item_data).facing_dir;
    (*item_data).owner = (*spawnItem).x0_parent_gobj;
    (*item_data).mex_original_owner = (*spawnItem).x0_parent_gobj;

    {
        let facing_dir = if (*item_data).facing_dir == -1.0 { -1 } else { 1 };
        ext::mpCollSetFacingDir(&raw mut (*item_data).x378_itemColl, facing_dir);
    }

    HSD_JObjSetFacingDirItem(model, item_data);
    ext::it_80273500(gobj, &raw mut (*spawnItem).vel);

    (*item_data).x5CC_currentAnimFrame = 0.0;
    (*item_data).x5D0_animFrameSpeed = 1.0;

    ext::it_802753BC(gobj, (*spawnItem).x3C_damage as i16);
    ext::it_802753DC(gobj);

    (*item_data).xC64_reflectGObj = null_mut();
    (*item_data).xDCC_flag.set_b0(0);

    (*item_data).xC90_absorbGObj = null_mut();
    (*item_data).xD0C = 0;

    ext::it_8026B390(gobj);
    (*item_data).xDC8_word.flags.set_x13(0);

    if (*spawnItem).x48_ground_or_air == GA_Air {
        ext::it_802762BC(item_data);
    } else {
        ext::it_802762B0(item_data);
    }

    ext::it_80275E98(gobj, spawnItem);
    ext::it_80274DAC(gobj);
    HSD_JObjSetTranslate(model, &raw mut (*item_data).pos);
    ext::it_80274658(gobj, (*ext::it_804D6D28).x6C_float);
    ext::it_802725D4(gobj);
    ext::it_80271508(gobj, HurtCapsule_Enabled as u32);
    ext::it_80272280(gobj);

    (*item_data).on_accessory = None;
    (*item_data).touched = None;
    (*item_data).entered_hitlag = None;
    (*item_data).exited_hitlag = None;
    (*item_data).jumped_on = None;

    if !(*item_data).owner.is_null() {
        ext::db_80225DD8(gobj, (*item_data).owner);
    } else {
        ext::db_80225D64(gobj, (*item_data).owner);
    }

    ext::it_8027B0C4(gobj, spawnItem);
    ext::it_80279B64(item_data);
    ItUnkHoldKind(gobj);
}

/// Remove Camera Box
pub unsafe extern "C" fn Item_80267454(gobj: *mut HSD_GObj) {
    let item_data = (*gobj).user_data as *mut Item;
    if (*item_data).xDCD_flag.anon_0.b01() != 0 && !(*item_data).x520_cameraBox.is_null() {
        ext::Camera_800290D4((*item_data).x520_cameraBox);
        (*item_data).x520_cameraBox = null_mut();
        (*item_data).xDCD_flag.anon_0.set_b01(0);
    }
}

/// Set Item Hold kind
pub unsafe extern "C" fn Item_802674AC(spawnItem: *mut SpawnItem) {
    let kind = (*spawnItem).kind;

    if kind >= GENO_ITEM_KIND_BASE && kind < GENO_ITEM_KIND_END && ext::Geno_ItemActive(kind) != 0 {
        (*spawnItem).hold_kind = ext::Geno_ItemCap(kind) as _;
        return;
    }
    if geno_art_is_kind(kind) {
        (*spawnItem).hold_kind = 8;
        return;
    }

    if kind == It_Kind_Foods {
        (*spawnItem).hold_kind = 2;
        return;
    }
    if kind == It_PKind_Random {
        (*spawnItem).hold_kind = 3;
        return;
    }
    if kind < It_Kind_L_Gun_Ray {
        (*spawnItem).hold_kind = 0;
        return;
    }
    if kind < It_Kind_Kuriboh {
        (*spawnItem).hold_kind = 1;
        return;
    }
    if kind < It_Kind_Octarock_Stone {
        (*spawnItem).hold_kind = 6;
        return;
    }
    if kind < It_Kind_Mario_Fire {
        (*spawnItem).hold_kind = 7;
        return;
    }
    if kind < It_Kind_Unk4 {
        (*spawnItem).hold_kind = 8;
        return;
    }
    if kind == It_Kind_Unk4 {
        (*spawnItem).hold_kind = 12;
        return;
    }
    if kind < It_PKind_Random {
        (*spawnItem).hold_kind = 11;
        return;
    }
    if kind < It_PKind_Terminate {
        (*spawnItem).hold_kind = 9;
        return;
    }
    if kind < It_Kind_Old_Kuri {
        (*spawnItem).hold_kind = 10;
        return;
    }
    if kind < It_Kind_Arwing_Laser {
        (*spawnItem).hold_kind = 4;
        return;
    }
    (*spawnItem).hold_kind = 5;
}

pub unsafe extern "C" fn Item_802675A8(gobj: *mut HSD_GObj) {
    let item_data = (*gobj).user_data as *mut Item;
    let c = &raw mut Item_804A0C64;
    match (*item_data).hold_kind {
        0 => (*c).x0 = (*c).x0.wrapping_sub(1),
        1 => (*c).x8 = (*c).x8.wrapping_sub(1),
        2 => (*c).x10 = (*c).x10.wrapping_sub(1),
        3 => (*c).x58 = (*c).x58.wrapping_sub(1),
        4 => (*c).x3C = (*c).x3C.wrapping_sub(1),
        5 => (*c).x40 = (*c).x40.wrapping_sub(1),
        6 => (*c).x2C = (*c).x2C.wrapping_sub(1),
        7 => (*c).x34 = (*c).x34.wrapping_sub(1),
        8 => (*c).x18 = (*c).x18.wrapping_sub(1),
        9 => (*c).x1C = (*c).x1C.wrapping_sub(1),
        10 => (*c).x24 = (*c).x24.wrapping_sub(1),
        11 => (*c).x48 = (*c).x48.wrapping_sub(1),
        12 => (*c).x50 = (*c).x50.wrapping_sub(1),
        _ => {}
    }
    if ext::gm_8018841C() != 0 && (*item_data).x18 == 1 {
        (*c).x60 = (*c).x60.wrapping_sub(1);
    }
}

pub unsafe extern "C" fn Item_802676F4(gobj: *mut HSD_GObj) {
    let item_data = (*gobj).user_data as *mut Item;
    let c = &raw mut Item_804A0C64;
    match (*item_data).hold_kind {
        0 => {
            (*c).x0 = (*c).x0.wrapping_add(1);
            if (*item_data).kind == It_Kind_M_Ball {
                (*c).x1C = (*c).x1C.wrapping_add(1);
            }
        }
        1 => (*c).x8 = (*c).x8.wrapping_add(1),
        2 => {
            (*c).x10 = (*c).x10.wrapping_add(1);
            ext::it_80274EE8();
        }
        3 => (*c).x58 = (*c).x58.wrapping_add(1),
        4 => (*c).x3C = (*c).x3C.wrapping_add(1),
        5 => (*c).x40 = (*c).x40.wrapping_add(1),
        6 => (*c).x2C = (*c).x2C.wrapping_add(1),
        7 => (*c).x34 = (*c).x34.wrapping_add(1),
        8 => (*c).x18 = (*c).x18.wrapping_add(1),
        10 => (*c).x24 = (*c).x24.wrapping_add(1),
        11 => (*c).x48 = (*c).x48.wrapping_add(1),
        12 => (*c).x50 = (*c).x50.wrapping_add(1),
        _ => {}
    }
    if ext::gm_8018841C() != 0 && (*item_data).x18 == 1 {
        (*c).x60 = (*c).x60.wrapping_add(1);
    }
}

pub unsafe extern "C" fn Item_8026784C(dropItem: enum_t, _: i32) -> bool {
    let c = &raw mut Item_804A0C64;
    let mut result = false;
    match dropItem {
        0 => {
            if (*c).x0 >= (*c).x4 {
                result = true;
            }
        }
        1 => {
            if (*c).x8 >= (*c).xC {
                result = true;
            }
        }
        2 => {
            if (*c).x10 >= (*c).x14 {
                result = true;
            }
        }
        3 => {
            if (*c).x58 >= (*c).x5C {
                result = true;
            }
        }
        5 => {
            if (*c).x40 >= (*c).x44 {
                result = true;
            }
        }
        7 => {
            if (*c).x34 >= (*c).x38 {
                result = true;
            }
        }
        9 => {
            if (*c).x1C > (*c).x20 {
                (*c).x1C = (*c).x1C.wrapping_sub(1);
                result = true;
            }
        }
        10 => {
            if (*c).x24 >= (*c).x28 {
                result = true;
            }
        }
        11 => {
            if (*c).x48 >= (*c).x4C {
                result = true;
            }
        }
        12 => {
            if (*c).x50 >= (*c).x54 {
                result = true;
            }
        }
        _ => {}
    }
    result
}

pub unsafe extern "C" fn Item_80267978(gobj: *mut HSD_GObj) {
    let item_data = (*gobj).user_data as *mut Item;
    let kind = (*item_data).kind;
    if kind < It_Kind_Kuriboh {
        // Common items
        (*item_data).xC4_article_data = *ext::it_804D6D24.add(kind as usize);
        (*item_data).xB8_itemLogicTable = &raw mut ext::it_803F14C4[kind as usize];
    } else if kind < It_PKind_Start {
        // Character items
        let idx = (kind - It_Kind_Kuriboh) as usize;
        (*item_data).xC4_article_data = *ext::it_804D6D38.add(idx);
        (*item_data).xB8_itemLogicTable = &raw mut ext::it_803F3100[idx];
    } else if kind < It_Kind_Old_Kuri {
        // Pokemon
        let idx = (kind - It_PKind_Start) as usize;
        (*item_data).xC4_article_data = *ext::it_804D6D30.add(idx);
        (*item_data).xB8_itemLogicTable = &raw mut ext::it_803F23CC[idx];
    } else if geno_art_is_kind(kind) {
        (*item_data).xC4_article_data = ext::Geno_ArticleDesc(kind) as *mut Article;
        (*item_data).xB8_itemLogicTable = ext::Geno_ArticleLogic(kind) as *mut ItemLogicTable;
    } else if kind >= GENO_ITEM_KIND_BASE && kind < GENO_ITEM_KIND_END && ext::Geno_ItemActive(kind) != 0 {
        (*item_data).xC4_article_data = ext::Geno_ItemDesc(kind) as *mut Article;
        (*item_data).xB8_itemLogicTable = ext::Geno_ItemLogic(kind) as *mut ItemLogicTable;
    } else if kind >= 237 {
        (*item_data).xC4_article_data = ext::Mex_ItemCustomDesc(kind) as *mut Article;
        (*item_data).xB8_itemLogicTable = ext::Mex_ItemCustomLogic(kind) as *mut ItemLogicTable;
    } else {
        // Stage items
        let idx = (kind - It_Kind_Old_Kuri) as usize;
        (*item_data).xC4_article_data = ext::it_804A0F60[idx];
        (*item_data).xB8_itemLogicTable = &raw mut ext::it_803F4D20[idx];
        if (*item_data).xC4_article_data.is_null() {
            panic!("not found zako model data! check ground dat file!\n");
        }
    }
    if (*item_data).xC4_article_data.is_null() {
        ext::Mex_ItemNotInitialized((*item_data).kind);
    }
    (*item_data).xBC_itemStateContainer = (*(*item_data).xB8_itemLogicTable).states;
}

/// Initialize item variables
pub unsafe extern "C" fn Item_80267AA8(gobj: *mut HSD_GObj, spawnItem: *mut SpawnItem) {
    let item_data = get_item(gobj);
    (*item_data).kind = (*spawnItem).kind;
    (*item_data).hold_kind = (*spawnItem).hold_kind;
    (*item_data).x18 = (*spawnItem).x10 as _;
    (*item_data).x1C = ext::it_804D6D10 as _;
    ext::it_804D6D10 = ext::it_804D6D10.wrapping_add(1);
    (*item_data).entity = gobj;
    Item_80267978(gobj);
    (*item_data).msid = -1;
    (*item_data).xC8_joint = (*(*(*item_data).xC4_article_data).x10_modelDesc).x0_joint;
    (*item_data).xCC_item_attr = (*(*item_data).xC4_article_data).x0_common_attr;
    (*item_data).owner = null_mut();
    (*item_data).xDC8_word.flags.set_x0(0);
    (*item_data).ecb_lock = -1;
    (*item_data).xDC4 = 0;
    (*item_data).xDC8_word.flags.set_xF(0);
    (*item_data).xCC4 = 0;
    (*item_data).xDCF_flag.anon_0.set_b6(0);
    (*item_data).xDCF_flag.anon_0.set_b7(0);
    (*item_data).xCB0_source_ply = 6;
    (*item_data).xCB4 = -1;
    (*item_data).xC38 = -1;
    (*item_data).xCE0.z = 0.0;
    (*item_data).xCE0.y = 0.0;
    (*item_data).xCE0.x = 0.0;
    (*item_data).xCD4.z = 0.0;
    (*item_data).xCD4.y = 0.0;
    (*item_data).xCD4.x = 0.0;
    (*item_data).xDD0_flag.anon_0.set_b1(0);
    (*item_data).xDC8_word.flags.set_x14(0);
    (*item_data).xDC8_word.flags.set_xE(0);
    (*item_data).xCEC_fighterGObj = null_mut();
    (*item_data).xCF0_itemGObj = null_mut();
    (*item_data).xCF4_fighterGObjUnk = null_mut();
    (*item_data).toucher = null_mut();
    (*item_data).xC64_reflectGObj = null_mut();
    (*item_data).xCFC = null_mut();
    (*item_data).atk_victim = null_mut();
    (*item_data).grab_victim = null_mut();
    (*item_data).xCCC_incDamageDirection = 0.0;
    (*item_data).xCB8_outDamageDirection = 0.0;
    (*item_data).xC68 = 0.0;
    (*item_data).xCD0 = 0.0;
    ext::it_80275158(gobj, (*ext::it_804D6D28).x30_lifetime as f32);
    (*item_data).xDD0_flag.anon_0.set_b3(0);
    (*item_data).spin_spd = (*(*item_data).xCC_item_attr).xC_spin_speed;
    (*item_data).xDC8_word.flags.set_x19((*(*item_data).xCC_item_attr).x1_3() as _);
    (*item_data).xDC8_word.flags.set_x17((*(*item_data).xCC_item_attr).x1_1() as _);
    let item_attr = (*item_data).xCC_item_attr;
    (*item_data).xBCC_unk = (*item_attr).x30_unk;
    (*item_data).xBD4_grabRange = (*item_attr).x38_grab_range;
    (*item_data).xDC8_word.flags.set_x1A((*(*item_data).xCC_item_attr).x1_4() as _);
    let item_attr = (*item_data).xCC_item_attr;
    (*item_data).xBEC = (*item_attr).x20;
    let item_attr = (*item_data).xCC_item_attr;
    (*item_data).xBDC = (*item_attr).x20;
    (*item_data).xDC8_word.flags.set_x1C(1);
    (*item_data).xDC8_word.flags.set_x1D(1);
    (*item_data).xDC8_word.flags.set_x1E(1);
    let item_attr = (*item_data).xCC_item_attr;
    (*item_data).xC1C = (*item_attr).x40;
    (*item_data).xC0C = (*item_data).xC1C;
    (*item_data).xBFC = (*item_data).xC0C;
    (*item_data).xDC8_word.flags.set_xC((*(*item_data).xCC_item_attr).x1_5() as _);
    (*item_data).xDC8_word.flags.set_xD(1);
    (*item_data).xDCD_flag.anon_0.set_b01((*(*item_data).xCC_item_attr).x1_67_cam_kind() as _);
    (*item_data).xDCE_flag.anon_0.set_b3(0);
    (*item_data).xD54_throwNum = 0;
    (*item_data).xD50_landNum = 0;
    (*item_data).xD58 = 0;
    (*item_data).xD5C = 2;
    (*item_data).xDCC_flag.set_b3(1);
    (*item_data).xDCC_flag.set_b4567(15);
    (*item_data).xAC4_ignoreItemID = (*spawnItem).x40 as _;
    (*item_data).destroy_type = 4;
    (*item_data).xDCE_flag.anon_0.set_b5(0);
    (*item_data).xDCE_flag.anon_0.set_b4(0);
    (*item_data).xC54 = 0.0;
    (*item_data).xC58.x = 0.0;
    (*item_data).xC58.x = 0.0;
    (*item_data).xC58.x = 0.0;
    (*item_data).xDC8_word.flags.set_x9(0);
    (*item_data).xDC8_word.flags.set_x3(0);
    (*item_data).xDC8_word.flags.set_x4(0);
    (*item_data).xDC8_word.flags.set_x5(0);
    (*item_data).xDC8_word.flags.set_x6(0);
    (*item_data).xDC8_word.flags.set_x7(0);
    (*item_data).xDC8_word.flags.set_x8(0);
    (*item_data).xDD0_flag.anon_0.set_b5(0);
    (*item_data).xD09 = 0;
    (*item_data).xDD0_flag.anon_0.set_b4(0);
    (*item_data).xCA8 = 0;
    (*item_data).xCBC_hitlagFrames = 0.0;
    (*item_data).xCC0 = 0.0;
    (*item_data).xDC8_word.flags.set_xA(0);
    (*item_data).xDC8_word.flags.set_xB(0);
    (*item_data).xDCC_flag.set_b1(0);
    (*item_data).xC70 = 1.0;
    (*item_data).xC6C = 1.0;
    (*item_data).xDCC_flag.set_b2(0);
    (*item_data).xDD0_flag.anon_0.set_b0(0);
    (*item_data).xDCE_flag.anon_0.set_b6(0);
    (*item_data).xDCF_flag.anon_0.set_b0(0);
    (*item_data).xDCF_flag.anon_0.set_b1(0);
    (*item_data).xDCF_flag.anon_0.set_b2(0);
    (*item_data).xDD1_flag.anon_0.set_b0(0);
    (*item_data).xDCE_flag.anon_0.set_b7(1);
    (*item_data).scl = (*(*item_data).xCC_item_attr).x60_scale;
    (*item_data).x3C = 0.0;
    (*item_data).xDCD_flag.anon_0.set_b3(0);
    (*item_data).xDCD_flag.anon_0.set_b4(0);
    (*item_data).xDD0_flag.anon_0.set_b7(0);
    (*item_data).xBC0 = null_mut();
    (*item_data).xDD1_flag.anon_0.set_b1(0);
    (*item_data).xDCD_flag.anon_0.set_b2(0);
    (*item_data).xD6C = -1;
    (*item_data).sfx_unk2 = SFX_NONE as _;
    (*item_data).sfx_unk1 = SFX_NONE as _;
    (*item_data).xD70 = (*(*item_data).xCC_item_attr).x6C;
    (*item_data).xD74 = (*(*item_data).xCC_item_attr).x70;
    (*item_data).xD78 = (*(*item_data).xCC_item_attr).x74;
    (*item_data).destroy_sfx = (*(*item_data).xCC_item_attr).destroy_sfx;
    (*item_data).xD80 = (*(*item_data).xCC_item_attr).x7C;
    let item_attr = (*item_data).xCC_item_attr;
    (*item_data).xD84 = (*item_attr).x80;

    ext::it_80275474(gobj);
    ext::it_80275504(gobj);
    ext::it_80274EF8(gobj);

    if ext::ftLib_80086960((*spawnItem).x0_parent_gobj) != 0 {
        (*item_data).xDC8_word.flags.set_x1(0);
    } else {
        (*item_data).xDC8_word.flags.set_x1(1);
    }

    (*item_data).x5C8 = 0;
    (*item_data).xBC4.a = 0;
    (*item_data).xBC4.b = 0;
    (*item_data).xBC4.g = 0;
    (*item_data).xBC4.r = 0;
    (*item_data).x5C9 = 255;

    ext::it_80279B64(item_data);

    (*item_data).xDCF_flag.anon_0.set_b3(0);
    (*item_data).xDCF_flag.anon_0.set_b4(0);
    (*item_data).xDCF_flag.anon_0.set_b5(0);

    (*item_data).anon_daa.xDAA_byte = 1;

    if ext::db_ShowItemPickupRange() != 0 {
        (*item_data).anon_daa.xDAA_flag.anon_0.set_b4(1);
    }

    if (*spawnItem).x0_parent_gobj.is_null() {
        (*item_data).x20_team_id = (-1i32) as _;
    } else if ext::ftLib_80086960((*spawnItem).x0_parent_gobj) != 0 {
        (*item_data).x20_team_id = ext::ftLib_80086EB4((*spawnItem).x0_parent_gobj) as _;
    } else if ext::it_80272D1C((*spawnItem).x0_parent_gobj) != 0 {
        (*item_data).x20_team_id = ext::itGetTeamId((*spawnItem).x0_parent_gobj) as _;
    } else {
        (*item_data).x20_team_id = (-1i32) as _;
    }
}

/// Setup Item JObj
pub unsafe extern "C" fn Item_802680CC(gobj: *mut HSD_GObj) {
    let item_data = get_item(gobj);
    if !(*item_data).xC8_joint.is_null() {
        let jobj = ext::HSD_JObjLoadJoint((*item_data).xC8_joint);
        ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_JObjKind, jobj as *mut c_void);
    } else {
        let jobj = ext::HSD_JObjAlloc();
        ext::PSMTXIdentity((*jobj).mtx.as_mut_ptr() as *mut c_void);
        (*jobj).scl = null_mut();
        ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_JObjKind, jobj as *mut c_void);
    }
}

/// Set up item render objects?
pub unsafe extern "C" fn Item_8026814C(gobj: *mut HSD_GObj) {
    let mut jobj = get_jobj(gobj);
    while !jobj.is_null() {
        let mut dobj = ext::HSD_JObjGetDObj(jobj);
        loop {
            if dobj.is_null() {
                break;
            }
            let mobj = (*dobj).mobj;
            if !mobj.is_null() {
                ext::hsdChangeClass(mobj as *mut c_void, (&raw mut it_mobj) as *mut c_void);
            }
            dobj = if !dobj.is_null() { (*dobj).next } else { null_mut() };
        }
        if !HSD_JObjGetChild(jobj).is_null() {
            jobj = HSD_JObjGetChild(jobj);
        } else if !HSD_JObjGetNext(jobj).is_null() {
            jobj = HSD_JObjGetNext(jobj);
        } else {
            loop {
                if HSD_JObjGetParent(jobj).is_null() {
                    jobj = null_mut();
                    break;
                }
                if !HSD_JObjGetNext(HSD_JObjGetParent(jobj)).is_null() {
                    jobj = HSD_JObjGetNext(HSD_JObjGetParent(jobj));
                    break;
                }
                jobj = HSD_JObjGetParent(jobj);
            }
        }
    }
}

/// Initialize item bones
pub unsafe extern "C" fn Item_802682F0(gobj: *mut HSD_GObj) -> bool {
    let ip = get_item(gobj);
    if (*(*(*ip).xC4_article_data).x10_modelDesc).x4_bone_count != 0 {
        (*ip).xBBC_dynamicBoneTable = ext::HSD_ObjAlloc(&raw mut item_dynamic_bones_alloc_data) as *mut DynamicBoneTable;
        if (*ip).xBBC_dynamicBoneTable.is_null() {
            return false;
        }
        let mut jobj = get_jobj(gobj);
        let mut i = 0usize;
        while !jobj.is_null() {
            (*(*ip).xBBC_dynamicBoneTable).bones[i] = jobj;
            i += 1;
            if !HSD_JObjGetChild(jobj).is_null() {
                jobj = HSD_JObjGetChild(jobj);
            } else if !HSD_JObjGetNext(jobj).is_null() {
                jobj = HSD_JObjGetNext(jobj);
            } else {
                loop {
                    if HSD_JObjGetParent(jobj).is_null() {
                        jobj = null_mut();
                        break;
                    }
                    if !HSD_JObjGetNext(HSD_JObjGetParent(jobj)).is_null() {
                        jobj = HSD_JObjGetNext(HSD_JObjGetParent(jobj));
                        break;
                    }
                    jobj = HSD_JObjGetParent(jobj);
                }
            }
        }
    } else {
        (*ip).xBBC_dynamicBoneTable = null_mut();
    }
    true
}

/// Set item model scale
pub unsafe extern "C" fn Item_8026849C(gobj: *mut HSD_GObj) {
    let temp_jobj = get_jobj(gobj);
    let item_data = get_item(gobj);
    let tmp = (*item_data).scl;
    let mut sp14: Vec3 = core::mem::zeroed();
    sp14.z = tmp;
    sp14.y = sp14.z;
    sp14.x = sp14.y;
    HSD_JObjSetScale(temp_jobj, &raw mut sp14);
}

/// Set up item dynamic bones
pub unsafe extern "C" fn Item_80268560(gobj: *mut HSD_GObj) {
    let ip = get_item(gobj);
    let article_data = (*ip).xC4_article_data;
    if (*article_data).x14_dynamics.is_null() {
        (*ip).x374_dynamicBonesNum = 0;
        (*ip).xB68 = 0;
        return;
    }
    (*ip).x374_dynamicBonesNum = (*(*article_data).x14_dynamics).count;
    let mut i = 0;
    while i < (*(*article_data).x14_dynamics).count {
        let desc = (*(*article_data).x14_dynamics).dyn_descs.add(i as usize);
        let jobj = (*(*ip).xBBC_dynamicBoneTable).bones[(*desc).bone_id as usize];
        ext::lb_8000FD48(jobj, &raw mut (*ip).xD4_dynamicBones[i as usize].dyn_desc, (*desc).dyn_desc.count as u32);
        (*ip).xD4_dynamicBones[i as usize].skeleton = jobj;
        (*ip).xD4_dynamicBones[i as usize].flags = 0;
        ext::lb_80011710(&raw mut (*(*(*article_data).x14_dynamics).dyn_descs.add(i as usize)).dyn_desc,
                         &raw mut (*ip).xD4_dynamicBones[i as usize].dyn_desc);
        i += 1;
    }
}

pub unsafe extern "C" fn foobar(gobj: *mut HSD_GObj) {
    let it = get_item(gobj);
    match (*it).hold_kind {
        0 | 6 => {
            if (*it).xD0C == 2 {
                (*it).xDD0_flag.anon_0.set_b7(1);
            }
            (*it).xD40 = (*ext::it_804D6D28).x2C as _;
            (*it).xDD0_flag.anon_0.set_b6(1);
            ext::it_802756D0(gobj);
            ext::it_80279B88(it, 3, 0);
        }
        _ => {
            (*it).xD40 = 0.0;
            (*it).xDD0_flag.anon_0.set_b6(0);
        }
    }
}

pub unsafe extern "C" fn foobar2(gobj: *mut HSD_GObj) {
    let it = get_item(gobj);
    // Check if item is a character item with an owner
    if (*it).kind >= It_Kind_Mario_Fire && (*it).kind < It_Kind_Unk4 && ext::ftLib_80086960((*it).owner) != 0 {
        (*it).xDC8_word.flags.set_xE(1);
        (*it).ecb_lock = ext::ftLib_GetKind((*it).owner) as _;
        ext::ftLib_8008702C((*it).ecb_lock);
    }
}

pub unsafe extern "C" fn foobar3(gobj: *mut HSD_GObj) {
    let it = get_item(gobj);
    if (*it).xDCD_flag.anon_0.b01() != 0 {
        if (*it).xDCD_flag.anon_0.b01() == 1 {
            (*it).x520_cameraBox = ext::Camera_80029044(0);
        } else {
            (*it).x520_cameraBox = ext::Camera_80029044(2);
        }
        let cam_box = (*it).x520_cameraBox;
        if !cam_box.is_null() {
            (*cam_box).target_ext.h.x = (*ext::it_804D6D28).x14C;
            (*cam_box).target_ext.h.y = (*ext::it_804D6D28).x150;
            (*cam_box).target_ext.v.x = (*ext::it_804D6D28).x154;
            (*cam_box).target_ext.v.y = (*ext::it_804D6D28).x158;
        }
    }
}

static mut reported: [u8; 208] = [0; 208];

/// Create Item
pub unsafe extern "C" fn Item_8026862C(spawnItem: *mut SpawnItem) -> *mut HSD_GObj {
    if (*spawnItem).kind >= 5000 && !geno_art_is_kind((*spawnItem).kind) {
        let parent = (*spawnItem).x0_parent_gobj;
        let cls: i32 = if !parent.is_null() { (*parent).classifier as i32 } else { -1 };
        let fk: i32 = if cls == HSD_GOBJ_CLASS_FIGHTER { ext::ftLib_GetKind(parent) as i32 } else { -1 };
        (*spawnItem).kind = ext::Mex_SpawnArticleKind((*spawnItem).kind - 5000, cls, fk);
    }
    if (*spawnItem).kind < It_Kind_Old_Kuri {
        let kind = (*spawnItem).kind;
        let article: *mut Article = if kind < It_Kind_Kuriboh {
            *ext::it_804D6D24.add(kind as usize)
        } else if kind < It_PKind_Start {
            *ext::it_804D6D38.add((kind - It_Kind_Kuriboh) as usize)
        } else {
            *ext::it_804D6D30.add((kind - It_PKind_Start) as usize)
        };
        if article.is_null() {
            if reported[kind as usize] == 0 {
                reported[kind as usize] = 1;
            }
            return null_mut();
        }
    }

    if Item_8026784C((*spawnItem).hold_kind as enum_t, (*spawnItem).kind) {
        return null_mut();
    }
    let gobj = ext::GObj_Create(HSD_GOBJ_CLASS_ITEM as u16, 9, 0);
    if gobj.is_null() {
        return null_mut();
    }
    let kind = (*spawnItem).kind;
    if kind < It_Kind_Kuriboh {
        ext::GObj_SetupGXLink(gobj, ext::it_803F1418[kind as usize].x0_renderFunc, 6, 0);
    } else if kind < It_PKind_Start {
        let idx = (kind - It_Kind_Kuriboh) as usize;
        ext::GObj_SetupGXLink(gobj, ext::it_803F2F28[idx].x0_renderFunc, 6, 0);
    } else if kind < It_Kind_Old_Kuri {
        let idx = (kind - It_PKind_Start) as usize;
        ext::GObj_SetupGXLink(gobj, ext::it_803F2310[idx].x0_renderFunc, 6, 0);
    } else if kind >= GENO_ITEM_KIND_BASE && kind < GENO_ITEM_KIND_END && ext::Geno_ItemActive(kind) != 0 {
        ext::GObj_SetupGXLink(gobj, Some(ext::cb::Geno_ItemDraw), 6, 0);
    } else if kind >= 237 {
        ext::GObj_SetupGXLink(gobj, Some(ext::cb::it_8026EECC), 6, 0);
    } else {
        let idx = (kind - It_Kind_Old_Kuri) as usize;
        ext::GObj_SetupGXLink(gobj, ext::it_803F4CA8[idx].x0_renderFunc, 6, 0);
    }
    let user_data = ext::HSD_ObjAlloc(&raw mut item_alloc_data);
    if user_data.is_null() {
        ext::HSD_GObjFree(gobj);
        return null_mut();
    }
    ext::GObj_InitUserData(gobj, 6, Some(Item_OnUserDataRemove), user_data);
    Item_80267AA8(gobj, spawnItem);
    Item_802680CC(gobj);
    if Item_802682F0(gobj) {
        Item_8026814C(gobj);
        Item_8026849C(gobj);
        ext::it_8027163C(gobj);
        Item_80268560(gobj);
        ext::HSD_GObj_SetupProc(gobj, Some(Item_802693E4), 0);
        ext::HSD_GObj_SetupProc(gobj, Some(Item_80269528), 1);
        ext::HSD_GObj_SetupProc(gobj, Some(Item_802697D4), 4);
        ext::HSD_GObj_SetupProc(gobj, Some(Item_80269978), 5);
        ext::HSD_GObj_SetupProc(gobj, Some(Item_80269A9C), 9);
        ext::HSD_GObj_SetupProc(gobj, Some(Item_80269B60), 11);
        ext::HSD_GObj_SetupProc(gobj, Some(Item_80269BE4), 12);
        ext::HSD_GObj_SetupProc(gobj, Some(Item_80269C5C), 13);
        ext::HSD_GObj_SetupProc(gobj, Some(Item_8026A294), 14);
        ext::HSD_GObj_SetupProc(gobj, Some(Item_8026A788), 16);
        Item_80267130(gobj, spawnItem);
        Item_8026A810(gobj);
        foobar(gobj);
        Item_802676F4(gobj);
        foobar2(gobj);
        foobar3(gobj);
    } else {
        ext::HSD_GObjFree(gobj);
        return null_mut();
    }
    gobj
}

/// Use the engine's exact hold-category policy (preserves all counters during a query).
pub unsafe extern "C" fn Item_ScriptSpawnAtCap(spawn: *mut SpawnItem) -> i32 {
    let mut copy: SpawnItem = *spawn;
    let saved: HSD_ObjAllocUnk = Item_804A0C64;
    Item_802674AC(&raw mut copy);
    let result = Item_8026784C(copy.hold_kind as enum_t, copy.kind);
    Item_804A0C64 = saved;
    result as i32
}

/// A disc-free query fixture: category-zero cap and caller/counter immutability.
pub unsafe extern "C" fn Item_ScriptSpawnCapTest() -> i32 {
    let saved: HSD_ObjAllocUnk = Item_804A0C64;
    let mut spawn: SpawnItem = core::mem::zeroed();
    let mut failed = 0;
    spawn.kind = It_Kind_Capsule;
    spawn.hold_kind = 99 as _;
    Item_804A0C64.x0 = 3;
    Item_804A0C64.x4 = 3;
    if Item_ScriptSpawnAtCap(&raw mut spawn) == 0 || Item_804A0C64.x0 != 3 || Item_804A0C64.x4 != 3 || spawn.hold_kind != 99 as _ {
        failed += 1;
    }
    Item_804A0C64.x0 = 2;
    if Item_ScriptSpawnAtCap(&raw mut spawn) != 0 || Item_804A0C64.x0 != 2 || Item_804A0C64.x4 != 3 || spawn.hold_kind != 99 as _ {
        failed += 1;
    }
    Item_804A0C64 = saved;
    failed
}

/// Item spawn prefunction - spawn airborne
pub unsafe extern "C" fn Item_80268B18(spawnItem: *mut SpawnItem) -> *mut HSD_GObj {
    (*spawnItem).x48_ground_or_air = GA_Air;
    (*spawnItem).x10 = 0;
    Item_802674AC(spawnItem);
    Item_8026862C(spawnItem)
}

/// Item spawn prefunction - spawn grounded
pub unsafe extern "C" fn Item_80268B5C(spawnItem: *mut SpawnItem) -> *mut HSD_GObj {
    (*spawnItem).x48_ground_or_air = GA_Ground;
    (*spawnItem).x10 = 0;
    Item_802674AC(spawnItem);
    Item_8026862C(spawnItem)
}

/// Item spawn prefunction - spawn grounded and toggle unknown true
pub unsafe extern "C" fn Item_80268B9C(spawnItem: *mut SpawnItem) {
    (*spawnItem).x48_ground_or_air = GA_Ground;
    (*spawnItem).x10 = 1;
    Item_802674AC(spawnItem);
    Item_8026862C(spawnItem);
}

/// Adds HSD_AObj instances to item model
pub unsafe extern "C" fn Item_80268BE0(item_jobj: *mut HSD_JObj, anim_joint: *mut HSD_AnimJoint,
                                       matanim_joint: *mut HSD_MatAnimJoint, shapeanim_joint: *mut HSD_ShapeAnimJoint,
                                       item_data: *mut Item) {
    if !item_jobj.is_null() {
        if !anim_joint.is_null() {
            let mut temp_r0 = 0;
            while temp_r0 < (*item_data).x374_dynamicBonesNum {
                if item_jobj == (*item_data).xD4_dynamicBones[temp_r0 as usize].skeleton {
                    if !(*anim_joint).next.is_null() {
                        let mut functionArg1: *mut HSD_AnimJoint = null_mut();
                        let mut functionArg2: *mut HSD_MatAnimJoint = null_mut();
                        let mut functionArg3: *mut HSD_ShapeAnimJoint = null_mut();
                        if !anim_joint.is_null() {
                            functionArg1 = (*anim_joint).next;
                        }
                        if !matanim_joint.is_null() {
                            functionArg2 = (*matanim_joint).next;
                        }
                        if !shapeanim_joint.is_null() {
                            functionArg3 = (*shapeanim_joint).next;
                        }
                        Item_80268BE0((*item_jobj).next, functionArg1, functionArg2, functionArg3, item_data);
                    }
                    return;
                }
                temp_r0 += 1;
            }
        }
        ext::HSD_JObjAddAnim(item_jobj, anim_joint, matanim_joint, shapeanim_joint);
        if !(*item_jobj).child.is_null() {
            let mut functionArg1: *mut HSD_AnimJoint = null_mut();
            let mut functionArg2: *mut HSD_MatAnimJoint = null_mut();
            let mut functionArg3: *mut HSD_ShapeAnimJoint = null_mut();
            if !anim_joint.is_null() {
                functionArg1 = (*anim_joint).child;
            }
            if !matanim_joint.is_null() {
                functionArg2 = (*matanim_joint).child;
            }
            if !shapeanim_joint.is_null() {
                functionArg3 = (*shapeanim_joint).child;
            }
            Item_80268BE0((*item_jobj).child, functionArg1, functionArg2, functionArg3, item_data);
        }
        if !(*item_jobj).next.is_null() {
            let mut functionArg1: *mut HSD_AnimJoint = null_mut();
            let mut functionArg2: *mut HSD_MatAnimJoint = null_mut();
            let mut functionArg3: *mut HSD_ShapeAnimJoint = null_mut();
            if !anim_joint.is_null() {
                functionArg1 = (*anim_joint).next;
            }
            if !matanim_joint.is_null() {
                functionArg2 = (*matanim_joint).next;
            }
            if !shapeanim_joint.is_null() {
                functionArg3 = (*shapeanim_joint).next;
            }
            Item_80268BE0((*item_jobj).next, functionArg1, functionArg2, functionArg3, item_data);
        }
    }
}

/// Unk Item AObj-related function
pub unsafe extern "C" fn Item_80268D34(gobj: *mut HSD_GObj, itemStateDesc: *mut ItemStateDesc) {
    let item_jobj = get_jobj(gobj);
    let item_data = get_item(gobj);
    ext::HSD_JObjRemoveAnimAll(item_jobj);
    if !(*item_data).xC8_joint.is_null() {
        let bonestruct_arg: *mut HSD_JObj = if item_jobj.is_null() { null_mut() } else { (*item_jobj).child };
        ext::lb_8000B804(bonestruct_arg, (*(*item_data).xC8_joint).child);
    }
    Item_80268BE0(item_jobj, (*itemStateDesc).x0_anim_joint, (*itemStateDesc).x4_matanim_joint, (*itemStateDesc).x8_parameters, item_data);
    ext::lb_8000BA0C(item_jobj, (*item_data).x5D0_animFrameSpeed);
    ext::HSD_JObjReqAnimAll(item_jobj, 0.0);
}

/// Advance item animation?
pub unsafe extern "C" fn Item_80268DD4(gobj: *mut HSD_GObj, frame: f32) {
    let item_data = get_item(gobj);
    let item_jobj = get_jobj(gobj);
    ext::lb_8000BA0C(item_jobj, (*item_data).x5D0_animFrameSpeed);
    ext::HSD_JObjReqAnimAll(item_jobj, frame);
    ext::HSD_JObjAnimAll(item_jobj);
    (*item_data).x5CC_currentAnimFrame = ext::lbGetJObjCurrFrame(item_jobj);
}

/// Copy item script
pub unsafe extern "C" fn Item_80268E40(item_data: *mut Item, itemStateDesc: *mut ItemStateDesc) {
    (*item_data).x524_cmd.anon_8.u = (*itemStateDesc).xC_script as _;
    (*item_data).x524_cmd.loop_count = 0;
    (*item_data).x524_cmd.timer = 0.0;
}

/// Change item state
pub unsafe extern "C" fn Item_80268E5C(gobj: *mut HSD_GObj, msid: enum_t, flags: Item_StateChangeFlags) {
    let mut sp4C: Vec3 = core::mem::zeroed();
    let mut scl: Vec3 = core::mem::zeroed();

    let item_jobj = get_jobj(gobj);
    let item_data = get_item(gobj);

    (*item_data).msid = msid;
    (*item_data).xDC8_word.flags.set_x14(0);
    HSD_JObjSetTranslate((*gobj).hsd_obj as *mut HSD_JObj, &raw mut (*item_data).pos);
    ext::efAsync_QueueFlush(gobj, (&raw mut (*item_data).xBC0) as *mut c_void);

    if (*item_data).xDC8_word.flags.x2() == 1 {
        ext::it_8026BDCC(gobj);
        ext::it_80274EF8(gobj);
    }

    if flags & ITEM_MODEL_UPDATE != 0 {
        ext::it_80274740(gobj);
    }

    if flags & ITEM_COLANIM_PRESERVE == 0
        && (*ext::it_804D6D04.add((*item_data).x548_colorOverlay.x28_colanim.i as usize)).unk5 != 0
    {
        ext::it_80279BBC(item_data);
    }

    if flags & ITEM_SFX_PRESERVE == 0 {
        Item_8026B074(item_data);
    }

    HSD_JObjSetFacingDirItem(item_jobj, item_data);

    if flags & ITEM_DROP_UPDATE != 0 {
        (*item_data).xC40 = (*item_data).xC44;
    } else {
        (*item_data).xC40 = 1.0;
    }

    if flags & ITEM_HIT_PRESERVE == 0 && (*item_data).xDC8_word.flags.x16() != 0 {
        ext::it_802725D4(gobj);
    }

    (*item_data).xCC8_knockback = 0.0;
    let temp_r30 = (*item_data).xBC_itemStateContainer.add(msid as usize);
    (*item_data).anim_id = (*temp_r30).anim_id;
    (*item_data).x5CC_currentAnimFrame = 0.0;
    let temp_r0 = (*item_data).anim_id;

    if temp_r0 != -1 {
        let temp_r29: *mut ItemStateDesc =
            (&raw mut (*(*(*item_data).xC4_article_data).xC_itemStates).x0_itemStateDesc[temp_r0 as usize]) as *mut ItemStateDesc;
        (*item_data).xD0_itemStateDesc = temp_r29;
        let temp_r23 = (*item_data).xD0_itemStateDesc;

        if !temp_r23.is_null() && flags & ITEM_ANIM_UPDATE != 0 {
            Item_80268D34(gobj, temp_r23);

            if flags & ITEM_UNK_UPDATE != 0 {
                let item_jobj1_2 = (*gobj).hsd_obj as *mut HSD_JObj;
                let item_data3 = GetItemData(gobj);
                (*item_data3).scl = (*(*item_data3).xCC_item_attr).x60_scale;
                HSD_JObjSetScaleItem(item_data3, item_jobj1_2, &raw mut sp4C);
                let item_attr = (*item_data3).xCC_item_attr;
                (*item_data3).xBCC_unk = (*item_attr).x30_unk;
                (*item_data3).xBD4_grabRange = (*item_attr).x38_grab_range;
                (*item_data3).xBEC = (*(*item_data3).xCC_item_attr).x20;
                (*item_data3).xBDC = (*(*item_data3).xCC_item_attr).x20;
            } else {
                HSD_JObjSetScaleItem((*gobj).user_data as *mut Item, (*gobj).hsd_obj as *mut HSD_JObj, &raw mut scl);
            }

            (*item_data).x524_cmd.anon_8.u = (*temp_r29).xC_script as _;
            (*item_data).x524_cmd.loop_count = 0;
            (*item_data).x524_cmd.timer = 0.0;
        } else if !temp_r23.is_null() && flags & ITEM_CMD_UPDATE != 0 {
            (*item_data).x524_cmd.anon_8.u = (*temp_r29).xC_script as _;
            (*item_data).x524_cmd.loop_count = 0;
            (*item_data).x524_cmd.timer = 0.0;
        }

        ext::HSD_JObjAnimAll(item_jobj);
        ext::it_80279BE0(gobj);
        ext::it_802799E4(gobj);
    } else {
        ext::HSD_JObjRemoveAnimAll(item_jobj);
        (*item_data).x524_cmd.anon_8.u = null_mut();
    }

    (*item_data).animated = (*temp_r30).animated;
    (*item_data).physics_updated = (*temp_r30).physics_updated;
    (*item_data).collided = (*temp_r30).collided;
    (*item_data).on_accessory = None;
    (*item_data).touched = None;
    (*item_data).entered_hitlag = None;
    (*item_data).exited_hitlag = None;
    (*item_data).jumped_on = None;

    (*item_data).xDD0_flag.anon_0.set_b5(0);
    (*item_data).xD09 = 0;

    ext::it_802714C0(gobj);
}

pub unsafe extern "C" fn Item_802693E4(gobj: *mut HSD_GObj) {
    let item_data = (*gobj).user_data as *mut Item;
    if (*item_data).xDC8_word.flags.x5() != 0 {
        Item_8026A0A0(gobj);
        (*item_data).xDC8_word.flags.set_x5(0);
    }
    if (*item_data).xDC8_word.flags.x6() != 0 {
        Item_8026A0FC(gobj);
        (*item_data).xDC8_word.flags.set_x6(0);
    }
    if (*item_data).xCBC_hitlagFrames > 0.0 {
        (*item_data).xCBC_hitlagFrames -= 1.0;
        if (*item_data).xCBC_hitlagFrames <= 0.0 {
            (*item_data).xCBC_hitlagFrames = 0.0;
            (*item_data).xDC8_word.flags.set_xB(0);
            (*item_data).xDC8_word.flags.set_xA(0);
            if (*item_data).xDC8_word.flags.x8() == 0 && (*item_data).xDC8_word.flags.x7() == 0 {
                Item_8026A1E8(gobj);
            }
        }
    }
}

/// Advance item animation + script?
pub unsafe extern "C" fn Item_802694CC(gobj: *mut HSD_GObj) {
    let item_jobj = get_jobj(gobj);
    let item_data = get_item(gobj);
    ext::HSD_JObjAnimAll(item_jobj);
    (*item_data).x5CC_currentAnimFrame = ext::lbGetJObjCurrFrame(item_jobj);
    ext::it_802799E4(gobj);
}

/// Item Think - Animation
pub unsafe extern "C" fn Item_80269528(gobj: *mut HSD_GObj) {
    let item_data = get_item(gobj);
    if (*item_data).xDC8_word.flags.x9() == 0 {
        Item_802694CC(gobj);
        if let Some(f) = (*item_data).animated {
            if f(gobj) != 0 {
                (*item_data).destroy_type = 0;
                Item_8026A8EC(gobj);
                return;
            }
        }
    }
    if (*item_data).xDD0_flag.anon_0.b6() != 0 {
        (*item_data).xD40 -= 1.0;
        if (*item_data).xD40 <= 0.0 {
            (*item_data).xDD0_flag.anon_0.set_b6(0);
            (*item_data).xD40 = 0.0;
            ext::it_80279BBC(item_data);
            if (*item_data).xDD0_flag.anon_0.b7() == 0 {
                ext::it_802756E0(gobj);
            }
        }
    }
    if (*item_data).xDC8_word.flags.x15() != 0 && (*item_data).kind < It_Kind_L_Gun_Ray && (*item_data).xDD0_flag.anon_0.b3() == 0 {
        (*item_data).xD44_lifeTimer -= 1.0;
        if (*item_data).xD44_lifeTimer <= 0.0 {
            ext::Geno_ItemFamilyEvent(gobj, 1, -1);
            (*item_data).destroy_type = 0;
            Item_8026A8EC(gobj);
            return;
        }
        if (*item_data).xD44_lifeTimer <= (*ext::it_804D6D28).x34 as f32 {
            ext::it_802728C8(gobj);
        }
    }
    ext::it_802721B8(gobj);
    ext::it_80272298(gobj);
    ext::it_80279BE0(gobj);
}

/// Item Think - Check for Blast Zones
pub unsafe extern "C" fn Item_802696CC(gobj: *mut HSD_GObj) -> bool {
    let item_data = get_item(gobj);

    if (*item_data).xDCC_flag.b4567() & 8 != 0 && (*item_data).pos.x > ext::Stage_GetBlastZoneRightOffset() {
        (*item_data).destroy_type = 3;
        Item_8026A8EC(gobj);
        return true;
    }
    if (*item_data).xDCC_flag.b4567() & 4 != 0 && (*item_data).pos.x < ext::Stage_GetBlastZoneLeftOffset() {
        (*item_data).destroy_type = 3;
        Item_8026A8EC(gobj);
        return true;
    }
    if (*item_data).xDCC_flag.b4567() & 1 != 0 && (*item_data).pos.y > 10000.0 {
        (*item_data).destroy_type = 3;
        Item_8026A8EC(gobj);
        return true;
    }
    if (*item_data).xDCC_flag.b4567() & 2 != 0 && (*item_data).pos.y < ext::Stage_GetBlastZoneBottomOffset() {
        (*item_data).destroy_type = 3;
        Item_8026A8EC(gobj);
        return true;
    }
    false
}

pub unsafe extern "C" fn Item_802697D4(gobj: *mut HSD_GObj) {
    let mut sp1C: Vec3 = core::mem::zeroed();

    let item_data = get_item(gobj);
    if (*item_data).xDC8_word.flags.x9() == 0 {
        if let Some(f) = (*item_data).physics_updated {
            f(gobj);
        }
    }
    if (*item_data).xDC8_word.flags.x13() == 0 {
        if (*item_data).xDC8_word.flags.x9() == 0 {
            ext::PSVECAdd(&raw mut (*item_data).x40_vel, &raw mut (*item_data).x70_nudge, &raw mut sp1C);
            if (*item_data).ground_or_air == 0 && (*item_data).xDCD_flag.anon_0.b3() == 0 {
                ext::it_8027737C(gobj, &raw mut sp1C);
            }
            ext::PSVECAdd(&raw mut (*item_data).pos, &raw mut sp1C, &raw mut (*item_data).pos);
        }
        if (*item_data).xDCC_flag.b3() == 1 && ((*item_data).xDCC_flag.b4567() & 0xF) != 0 && Item_802696CC(gobj) {
            return;
        }
        if (*item_data).xD3C_spinSpeed != 0.0 && (*item_data).ground_or_air == 1 {
            ext::it_80274A64(gobj);
        }
    }
    if (*item_data).xDD1_flag.anon_0.b1() != 0 && (*item_data).ground_or_air == 1 {
        ext::grLib_801C9E60(&raw mut (*item_data).x58_vec_unk);
    }
    ext::PSVECAdd(&raw mut (*item_data).pos, &raw mut (*item_data).x58_vec_unk, &raw mut (*item_data).pos);
    ext::it_80273484(gobj);
    if (*item_data).ground_or_air == 0 {
        if (*item_data).xDCE_flag.anon_0.b3() != 0 && ext::it_8026D604(gobj) != 0 {
            ext::mpGetSpeed((*item_data).xC30, &raw mut (*item_data).pos, &raw mut (*item_data).x64_vec_unk2);
        } else {
            ext::mpGetSpeed((*item_data).x378_itemColl.floor.index, &raw mut (*item_data).pos, &raw mut (*item_data).x64_vec_unk2);
        }
    }
    ext::PSVECAdd(&raw mut (*item_data).pos, &raw mut (*item_data).x64_vec_unk2, &raw mut (*item_data).pos);
    ext::it_8027346C(gobj);
    ext::it_802714C0(gobj);
}

pub unsafe extern "C" fn Item_80269978(gobj: *mut HSD_GObj) {
    let item_data = get_item(gobj);
    let mut destroyed = false;
    if let Some(f) = (*item_data).collided {
        if f(gobj) != 0 {
            destroyed = true;
        }
    }
    if destroyed {
        (*item_data).destroy_type = 1;
        Item_8026A8EC(gobj);
    } else {
        HSD_JObjSetTranslate(get_jobj(gobj), &raw mut (*item_data).pos);
        ext::it_8027574C(gobj);
        ext::it_8026C368(gobj);
    }
}

/// this function is where the item accessory callback is called if it exists
pub unsafe extern "C" fn Item_80269A9C(gobj: *mut HSD_GObj) {
    let mut item_data = get_item(gobj);
    ext::efAsync_QueueFlush(gobj, (&raw mut (*item_data).xBC0) as *mut c_void);
    if (*item_data).xDC8_word.flags.x9() == 0 {
        if let Some(f) = (*item_data).on_accessory {
            f(gobj);
        }
    }
    item_data = get_item(gobj);
    if (*item_data).xDCD_flag.anon_0.b01() != 0 {
        let cm_subject = (*item_data).x520_cameraBox;
        if !cm_subject.is_null() {
            (*cm_subject).pos = (*item_data).pos;
            (*cm_subject).bone_pos = (*item_data).pos;
        }
    }
    ext::it_80271A58(gobj);
    ext::it_80272280(gobj);
    ext::it_802722B0(gobj);
}

/// Item Think - Yellow Bar Collision (cb_JumpedOn)
pub unsafe extern "C" fn Item_80269B60(gobj: *mut HSD_GObj) {
    let item_data = (*gobj).user_data as *mut Item;
    if !(*item_data).xCFC.is_null() && (*item_data).jumped_on.is_some() && ((*item_data).jumped_on.unwrap())(gobj) != 0 {
        (*item_data).destroy_type = 2;
        Item_8026A8EC(gobj);
        return;
    }
    ext::it_8027137C(gobj);
    ext::it_8027146C(gobj);
}

pub unsafe extern "C" fn Item_80269BE4(gobj: *mut HSD_GObj) {
    let item_data = (*gobj).user_data as *mut Item;
    if (*item_data).xDD0_flag.anon_0.b5() != 0 {
        ext::it_802701BC(gobj);
        if !(*item_data).grab_victim.is_null() {
            ((*item_data).grab_dealt.unwrap())(gobj);
            ((*item_data).grabbed_for_victim.unwrap())((*item_data).grab_victim, gobj);
        }
    }
}

/// Item Think - Hit Collision Logic
pub unsafe extern "C" fn Item_80269C5C(gobj: *mut HSD_GObj) {
    ext::it_8026F9A0();
    ext::it_802703E8(gobj);
    ext::it_802706D0(gobj);
    ext::it_80270E30(gobj);
}

/// Add to damage taken
pub unsafe extern "C" fn Item_80269CA0(item_data: *mut Item, damage: i32) {
    (*item_data).xC9C = (*item_data).xC9C.wrapping_add(damage);
    if (*item_data).xC9C > 999 {
        (*item_data).xC9C = 999;
    }
}

/// Set damage struct
pub unsafe extern "C" fn Item_80269CC4(gobj: *mut HSD_GObj) {
    let temp_item = (*gobj).user_data as *mut Item;

    (*temp_item).xCA0 = 0;
    (*temp_item).xCA4 = 0;
    (*temp_item).xC94 = 0;
    (*temp_item).xCC8_knockback = 0.0;
    (*temp_item).xC34_damageDealt = 0;
    (*temp_item).xC48 = 0;
    (*temp_item).xC4C = 0;
    (*temp_item).xC50 = 0;
    (*temp_item).xCCC_incDamageDirection = 0.0;
    (*temp_item).xCB8_outDamageDirection = 0.0;
    (*temp_item).xC68 = 0.0;
    (*temp_item).xCD0 = 0.0;
    (*temp_item).xCC0 = 0.0;
    (*temp_item).xDCF_flag.anon_0.set_b6(0);
    (*temp_item).xCC4 = 0;
    (*temp_item).xCEC_fighterGObj = null_mut();
    (*temp_item).xCF0_itemGObj = null_mut();
    (*temp_item).xCF4_fighterGObjUnk = null_mut();
    (*temp_item).toucher = null_mut();
    (*temp_item).xC64_reflectGObj = null_mut();
    (*temp_item).xC90_absorbGObj = null_mut();
    (*temp_item).xCFC = null_mut();
    (*temp_item).xCB4 = -1;
    (*temp_item).xC38 = -1;
    (*temp_item).xCE0.z = 0.0;
    (*temp_item).xCE0.y = 0.0;
    (*temp_item).xCE0.x = 0.0;
    (*temp_item).xCD4.z = 0.0;
    (*temp_item).xCD4.y = 0.0;
    (*temp_item).xCD4.x = 0.0;
    (*temp_item).xDCE_flag.anon_0.set_b5(0);
    (*temp_item).xDCE_flag.anon_0.set_b4(0);
    (*temp_item).xDCC_flag.set_b1(0);
    (*temp_item).xC70 = 1.0;
    (*temp_item).xC6C = 1.0;
    (*temp_item).xDCC_flag.set_b2(0);
    (*temp_item).xDC8_word.flags.set_xB(0);
    (*temp_item).xCA8 = 0;
    (*temp_item).xDC8_word.flags.set_x3(0);
    (*temp_item).xDC8_word.flags.set_x4(0);
    (*temp_item).xDCE_flag.anon_0.set_b6(0);
}

/// Item Think - Shield Collision
pub unsafe extern "C" fn Item_80269DC8(gobj: *mut HSD_GObj) -> bool {
    let item_data = (*gobj).user_data as *mut Item;

    if (*item_data).xDCE_flag.anon_0.b5() != 0
        && (*item_data).ground_or_air == 1
        && ((*item_data).xDCE_flag.anon_0.b4() != 0
            || (*item_data).xC54 < (M_PI as f32) / 180.0 * (90.0 + (*ext::it_804D6D28).unk_degrees))
    {
        if (*item_data).xDCE_flag.anon_0.b4() == 0 {
            (*item_data).xC54 = ((*item_data).xC54 as f64 - M_PI / 2.0) as f32;
            if (*item_data).xC54 < 0.0 {
                (*item_data).xC54 = 0.0;
            }
        }

        let shield_bounced = (*(*item_data).xB8_itemLogicTable).shield_bounced;
        if let Some(f) = shield_bounced {
            if f(gobj) != 0 {
                (*item_data).destroy_type = 2;
                Item_8026A8EC(gobj);
                return true;
            }
        }
    } else {
        let hit_shield = (*(*item_data).xB8_itemLogicTable).hit_shield;

        if ext::it_80274C78(gobj) != 0 {
            (*item_data).xCA8 = (*item_data).xC50;
        }

        if let Some(f) = hit_shield {
            if f(gobj) != 0 {
                (*item_data).destroy_type = 2;
                Item_8026A8EC(gobj);
                return true;
            }
        }
    }

    false
}

/// Item Think - On Reflect
pub unsafe extern "C" fn Item_80269F14(gobj: *mut HSD_GObj) -> bool {
    let temp_item = get_item(gobj);

    if (*temp_item).xDCC_flag.b1() == 0 {
        if (*temp_item).kind == It_Kind_M_Ball {
            if (*temp_item).xDCC_flag.b2() == 0 {
                (*temp_item).owner = (*temp_item).xC64_reflectGObj;
                (*temp_item).x20_team_id = ext::ftLib_80086EB4((*temp_item).owner) as _;
            }
        } else {
            (*temp_item).owner = (*temp_item).xC64_reflectGObj;
            (*temp_item).x20_team_id = ext::ftLib_80086EB4((*temp_item).owner) as _;
        }
    }

    ext::db_80225DD8(gobj, (*temp_item).owner);
    (*temp_item).xD90.x2070_int = (*temp_item).xC74;
    (*temp_item).xD94 = (*temp_item).xC78;
    (*temp_item).xD9C = (*temp_item).xC80;
    (*temp_item).xDA4_word = (*temp_item).xC88 as _;
    (*temp_item).xDA8_short = (*temp_item).xC8C;
    let cb_OnReflect = (*(*temp_item).xB8_itemLogicTable).reflected;
    if let Some(f) = cb_OnReflect {
        if f(gobj) != 0 {
            (*temp_item).destroy_type = 2;
            Item_8026A8EC(gobj);
            return true;
        }
    }
    if (*temp_item).xDCC_flag.b0() == 0 {
        let mut i = 0usize;
        while i < 4 {
            if (*temp_item).x5D4_hitboxes[i].hit.state != HitCapsule_Disabled {
                let temp_f30: f32 = ext::ScriptGame_HitRuleBaseDamage(&raw mut (*temp_item).x5D4_hitboxes[i].hit)
                    .mul_add((*temp_item).xC6C, 0.99);
                let mut var_r27: u32 = temp_f30 as u32;
                if var_r27 > (*ext::it_804D6D28).xD8 as u32 {
                    var_r27 = (*ext::it_804D6D28).xD8 as u32;
                }
                ext::it_80272460(&raw mut (*temp_item).x5D4_hitboxes[i].hit, var_r27, gobj);
            }
            i += 1;
        }
    }
    false
}

/// Item Think - Exit Hitlag Check
pub unsafe extern "C" fn Item_8026A0A0(gobj: *mut HSD_GObj) {
    let item_data = get_item(gobj);
    if (*item_data).xDC8_word.flags.x7() != 0 {
        if (*item_data).xDC8_word.flags.xA() == 0 && (*item_data).xDC8_word.flags.x8() == 0 {
            Item_8026A1E8(gobj);
        }
        (*item_data).xDC8_word.flags.set_x7(0);
    }
}

/// Item Think - Exit Hitlag Check 2
pub unsafe extern "C" fn Item_8026A0FC(gobj: *mut HSD_GObj) {
    let item_data = get_item(gobj);
    if (*item_data).xDC8_word.flags.x8() != 0 {
        if (*item_data).xDC8_word.flags.xA() == 0 && (*item_data).xDC8_word.flags.x7() == 0 {
            Item_8026A1E8(gobj);
        }
        (*item_data).xDC8_word.flags.set_x8(0);
    }
}

pub unsafe extern "C" fn func_8026A158_helper(atkCollGObj: *mut HSD_GObj) {
    let temp_item = get_item(atkCollGObj);
    (*temp_item).xDC8_word.flags.set_x8(1);
    Item_8026A158(atkCollGObj);
}

/// Item Think - Enter Hitlag
pub unsafe extern "C" fn Item_8026A158(gobj: *mut HSD_GObj) {
    let item_data = get_item(gobj);
    if let Some(f) = (*item_data).entered_hitlag {
        f(gobj);
    }
    (*item_data).xDC8_word.flags.set_x9(1);
    if !(*item_data).atk_victim.is_null() && ext::it_80272D1C((*item_data).atk_victim) != 0 && (*item_data).xDC8_word.flags.x8() == 0 {
        func_8026A158_helper((*item_data).atk_victim);
    }
}

pub unsafe extern "C" fn func_8026A1E8_inline(atkCollGObj: *mut HSD_GObj) {
    let temp_item = get_item(atkCollGObj);
    if (*temp_item).xDC8_word.flags.x8() != 0 {
        if (*temp_item).xDC8_word.flags.xA() == 0 && (*temp_item).xDC8_word.flags.x7() == 0 {
            Item_8026A1E8(atkCollGObj);
        }
        (*temp_item).xDC8_word.flags.set_x8(0);
    }
}

/// Item Think - Exit Hitlag
pub unsafe extern "C" fn Item_8026A1E8(gobj: *mut HSD_GObj) {
    let item_data = get_item(gobj);
    if let Some(f) = (*item_data).exited_hitlag {
        f(gobj);
    }
    (*item_data).xDC8_word.flags.set_x9(0);
    if !(*item_data).atk_victim.is_null() && ext::it_80272D1C((*item_data).atk_victim) != 0 && (*item_data).xDC8_word.flags.x8() == 0 {
        func_8026A1E8_inline((*item_data).atk_victim);
    }
}

pub unsafe extern "C" fn processCallback(cb: HSD_GObjPredicate, gobj: *mut HSD_GObj, item_data: *mut Item) -> bool {
    if let Some(f) = cb {
        if f(gobj) != 0 {
            (*item_data).destroy_type = 2;
            Item_8026A8EC(gobj);
            return true;
        }
    }
    false
}

pub unsafe extern "C" fn OnTakeDamageThink(gobj: *mut HSD_GObj, item_data: *mut Item) -> bool {
    ext::ScriptGame_EnemyReceived(gobj, (*item_data).xCEC_fighterGObj, (*item_data).xCA0);
    (*item_data).xC9C = (*item_data).xC9C.wrapping_add((*item_data).xCA0);
    if (*item_data).xC9C > 999 {
        (*item_data).xC9C = 999;
    }
    (*item_data).xCA8 = (*item_data).xCA0;
    (*item_data).xDC8_word.flags.set_xB(1);
    processCallback((*(*item_data).xB8_itemLogicTable).dmg_received, gobj, item_data)
}

pub unsafe extern "C" fn OnClankThink(gobj: *mut HSD_GObj, item_data: *mut Item) -> bool {
    let cb_OnClank = (*(*item_data).xB8_itemLogicTable).clanked;
    if ext::it_80274C78(gobj) != 0 {
        (*item_data).xCA8 = (*item_data).xC48;
    }
    processCallback(cb_OnClank, gobj, item_data)
}

pub unsafe extern "C" fn OnGiveDamageThink(gobj: *mut HSD_GObj, item_data: *mut Item) -> bool {
    let cb_OnGiveDamage = (*(*item_data).xB8_itemLogicTable).dmg_dealt;
    if ext::it_80274C78(gobj) != 0 {
        (*item_data).xCA8 = (*item_data).xC34_damageDealt;
    }
    processCallback(cb_OnGiveDamage, gobj, item_data)
}

pub unsafe extern "C" fn EnterHitlagThink(gobj: *mut HSD_GObj, item_data: *mut Item) {
    if (*item_data).xDC8_word.flags.x9() == 0 {
        let item_data = GetItemData(gobj);
        if let Some(f) = (*item_data).entered_hitlag {
            f(gobj);
        }
        (*item_data).xDC8_word.flags.set_x9(1);

        if !(*item_data).atk_victim.is_null() && ext::it_80272D1C((*item_data).atk_victim) != 0 && (*item_data).xDC8_word.flags.x8() == 0 {
            func_8026A158_helper((*item_data).atk_victim);
        }
    }
}

pub unsafe extern "C" fn checkHitLag(min_value: f32, item_data: *mut Item) {
    if (*item_data).xCBC_hitlagFrames < min_value {
        (*item_data).xCBC_hitlagFrames = min_value;
    }
    (*item_data).xDC8_word.flags.set_xA(1);
}

/// Item Think - Hit Collision
pub unsafe extern "C" fn Item_8026A294(gobj: *mut HSD_GObj) {
    let item_data = get_item(gobj);

    if (*item_data).xCC8_knockback != 0.0 || (*item_data).xCA0 != 0 {
        if OnTakeDamageThink(gobj, item_data) {
            return;
        }
    } else if (*item_data).xC50 != 0 {
        if Item_80269DC8(gobj) {
            return;
        }
    } else if (*item_data).xC48 != 0 {
        if OnClankThink(gobj, item_data) {
            return;
        }
    } else if (*item_data).xC34_damageDealt != 0 || (*item_data).xC4C != 0 {
        if OnGiveDamageThink(gobj, item_data) {
            return;
        }
    } else if !(*item_data).xC64_reflectGObj.is_null() {
        if Item_80269F14(gobj) {
            return;
        }
    } else if !(*item_data).xC90_absorbGObj.is_null() {
        if processCallback((*(*item_data).xB8_itemLogicTable).absorbed, gobj, item_data) {
            return;
        }
    } else if (*item_data).xDCE_flag.anon_0.b6() != 0 {
        if processCallback((*item_data).touched, gobj, item_data) {
            return;
        }
    }

    if (*item_data).xDC8_word.flags.xD() != 0 && (*item_data).xCC0 > 0.0 {
        checkHitLag((*item_data).xCC0, item_data);
        EnterHitlagThink(gobj, item_data);
    } else if (*item_data).xCA8 != 0 {
        checkHitLag(ext::it_8026B424((*item_data).xCA8), item_data);
        EnterHitlagThink(gobj, item_data);
    }
    if (*item_data).xDC8_word.flags.x4() != 0 || (*item_data).xDC8_word.flags.x3() != 0 {
        EnterHitlagThink(gobj, item_data);

        if (*item_data).xDC8_word.flags.x4() != 0 {
            (*item_data).xDC8_word.flags.set_x8(1);
            (*item_data).xDC8_word.flags.set_x4(0);
        } else {
            (*item_data).xDC8_word.flags.set_x7(1);
            (*item_data).xDC8_word.flags.set_x3(0);
        }
    }
    Item_80269CC4(gobj);
}

/// The script prepares the ordinary item damage result. Commit through the same callbacks, hitlag and cleanup.
pub unsafe extern "C" fn Item_ScriptCommitDamage(gobj: *mut HSD_GObj) -> i32 {
    if gobj.is_null() || (*get_item(gobj)).xCA0 <= 0 {
        return 0;
    }
    Item_8026A294(gobj);
    1
}

/// Item Think - Process Dynamic Bones
pub unsafe extern "C" fn Item_8026A788(gobj: *mut HSD_GObj) {
    let item_data = (*gobj).user_data as *mut Item;
    let dynamicBonesNum = (*item_data).x374_dynamicBonesNum;
    let mut dynamicBones = (&raw mut (*item_data).xD4_dynamicBones) as *mut Item_DynamicBones;

    ext::it_80272304(gobj);
    let mut i = 0;
    while i < dynamicBonesNum {
        ext::lb_8001044C(&raw mut (*dynamicBones).dyn_desc, (&raw mut (*item_data).xB6C_vars[0].xB6C.x) as *mut c_void,
                         (*item_data).xB68 as i32, 0.0, 0, (*dynamicBones).flags as u32, 0, 1);
        i += 1;
        dynamicBones = dynamicBones.add(1);
    }
}

/// Item Think - Spawn
pub unsafe extern "C" fn Item_8026A810(gobj: *mut HSD_GObj) {
    let temp_item = get_item(gobj);
    if let Some(f) = (*(*temp_item).xB8_itemLogicTable).spawned {
        f(gobj);
    }
}

pub unsafe extern "C" fn Item_8026A848(gobj: *mut HSD_GObj, fighter_gobj: *mut HSD_GObj) {
    let temp_item = get_item(gobj);
    if (*temp_item).hold_kind == 8
        && (*temp_item).kind != It_Kind_Link_Bomb
        && (*temp_item).kind != It_Kind_CLink_Bomb
        && (*temp_item).kind != It_Kind_Peach_Turnip
        && (*temp_item).kind != It_Kind_Peach_Parasol
    {
        if ext::ftLib_800867CC(fighter_gobj) == gobj {
            ext::ftLib_80086764(fighter_gobj);
        }
    } else if ext::ftLib_800867A0(fighter_gobj, gobj) != 0 {
        ext::ftLib_80086724(fighter_gobj, gobj);
    }
}

pub unsafe extern "C" fn DestroyItemInline(gobj: *mut HSD_GObj, other_ip: *mut Item) {
    let ip = (*gobj).user_data as *mut Item;
    let other = (*other_ip).owner;

    if (*ip).hold_kind == 8
        && (*ip).kind != It_Kind_Link_Bomb
        && (*ip).kind != It_Kind_CLink_Bomb
        && (*ip).kind != It_Kind_Peach_Turnip
        && (*ip).kind != It_Kind_Peach_Parasol
    {
        if ext::ftLib_800867CC(other) == gobj {
            ext::ftLib_80086764(other);
        }
    } else if ext::ftLib_800867A0(other, gobj) != 0 {
        ext::ftLib_80086724(other, gobj);
    }
}

pub unsafe extern "C" fn ItemSwitch(gobj: *mut HSD_GObj) {
    let ip = (*gobj).user_data as *mut Item;
    let destroy_type = (*ip).destroy_type;
    match destroy_type {
        0 => ext::it_8027327C(gobj, (*(*ip).xCC_item_attr).destroy_gfx as i32, (*ip).destroy_sfx as i32),
        2 | 1 => ext::it_8027327C(gobj, (*(*ip).xCC_item_attr).x68, (*ip).xD80),
        _ => {}
    }
}

pub unsafe extern "C" fn RunGObjCallback(gobj: *mut HSD_GObj, arg1: HSD_GObjEvent) {
    if let Some(f) = arg1 {
        f(gobj);
    }
}

pub unsafe extern "C" fn func_8026A8EC_inline1(gobj: *mut HSD_GObj) {
    let ip = get_item(gobj);
    if (*ip).xDC8_word.flags.xE() != 0 {
        ext::ftLib_80087050((*ip).ecb_lock);
        (*ip).xDC8_word.flags.set_xE(0);
        (*ip).ecb_lock = -1;
    }
}

pub unsafe extern "C" fn func_8026A8EC_inline2(gobj: *mut HSD_GObj) {
    let it = get_item(gobj);
    ext::Geno_ItemFamilyEvent(gobj, 2, -1);
    RunGObjCallback(gobj, (*(*it).xB8_itemLogicTable).destroyed);
}

pub unsafe extern "C" fn func_8026A8EC_inline3(gobj: *mut HSD_GObj) {
    let it = get_item(gobj);
    if (*it).xDCD_flag.anon_0.b01() != 0 && !(*it).x520_cameraBox.is_null() {
        ext::Camera_800290D4((*it).x520_cameraBox);
        (*it).x520_cameraBox = null_mut();
        (*it).xDCD_flag.anon_0.set_b01(0);
    }
}

pub unsafe extern "C" fn Item_8026A8EC(gobj: *mut HSD_GObj) {
    let ip = get_item(gobj);

    if ext::it_80272D1C(gobj) == 0 || ip.is_null() {
        panic!("===== Not Found Item_Struct!! =====\n");
    }

    ext::it_802725D4(gobj);
    Item_802675A8(gobj);
    func_8026A8EC_inline1(gobj);
    ext::efLib_DestroyAll(gobj);

    if (*ip).xDCF_flag.anon_0.b2() == 0 && ((*ip).xDC8_word.flags.x13() == 0 || (*ip).owner.is_null()) {
        ItemSwitch(gobj);
    }

    func_8026A8EC_inline2(gobj);

    if (*ip).xDC8_word.flags.x13() != 0 && !(*ip).owner.is_null() && ext::ftLib_80086960((*ip).owner) != 0 {
        DestroyItemInline(gobj, ip);
    }

    (*ip).owner = null_mut();
    func_8026A8EC_inline3(gobj);

    {
        let mut i = 0;
        while i < (*ip).x374_dynamicBonesNum {
            ext::lb_8000FD18(&raw mut (*ip).xD4_dynamicBones[i as usize].dyn_desc);
            i += 1;
        }
    }

    Item_8026B0B4(gobj);
    ext::efAsync_QueueClear((&raw mut (*ip).xBC0) as *mut c_void);
    ext::ScriptGame_EnemyDestroyed(gobj);
    ext::HSD_GObjFree(gobj);
}

/// Pick up item
pub unsafe extern "C" fn Item_8026AB54(gobj: *mut HSD_GObj, owner_gobj: *mut HSD_GObj, part: u32) {
    let item_data = get_item(gobj);

    ext::it_80273168(gobj); // sets pickup sfx?
    ext::it_802742F4(gobj, owner_gobj, part);
    ext::Geno_ItemFamilyEvent(gobj, 0, if !owner_gobj.is_null() { (*((*owner_gobj).user_data as *mut Fighter)).player_id as i32 } else { -1 });
    RunGObjCallback(gobj, (*(*item_data).xB8_itemLogicTable).picked_up);
    Item_8026B074(item_data);
}

pub unsafe extern "C" fn Item_8026ABD8(gobj: *mut HSD_GObj, pos: *mut Vec3, arg2: f32) {
    let item_data = get_item(gobj);

    (*item_data).xC44 = arg2;
    ext::it_802731A4(gobj);
    ext::it_80273B50(gobj, pos);
    RunGObjCallback(gobj, (*(*item_data).xB8_itemLogicTable).dropped);
    ext::it_80274198(gobj, 1);
    ext::it_802754D4(gobj);

    if ext::it_8026B6C8(gobj) != 0 {
        ext::it_80275390(gobj);
    }
}

pub unsafe extern "C" fn Item_8026AC74(gobj: *mut HSD_GObj, arg1: *mut Vec3, arg2: *mut Vec3, arg3: f32) {
    let item_data = GetItemData(gobj);
    (*item_data).xC44 = arg3;
    ext::it_802731A4(gobj);
    ext::it_80273748(gobj, arg1, arg2);
    RunGObjCallback(gobj, (*(*item_data).xB8_itemLogicTable).dropped);
    ext::it_802741F4(gobj, 1);
    ext::it_802754D4(gobj);

    if ext::it_8026B6C8(gobj) != 0 {
        ext::it_80275390(gobj);
    }
}

pub unsafe extern "C" fn Item_8026AD20(gobj: *mut HSD_GObj, arg1: *mut Vec3, arg2: *mut Vec3, arg3: f32, _arg4: bool) {
    let item_data = GetItemData(gobj);
    ext::it_802731E0(gobj);
    (*item_data).xC44 = arg3;
    ext::it_80273748(gobj, arg1, arg2);
    RunGObjCallback(gobj, (*(*item_data).xB8_itemLogicTable).thrown);
    ext::it_802741F4(gobj, 1);
    ext::it_802754D4(gobj);
}

pub unsafe extern "C" fn Item_8026ADC0(gobj: *mut HSD_GObj) {
    let item_data = get_item(gobj);
    if let Some(f) = (*(*item_data).xB8_itemLogicTable).entered_air {
        f(gobj);
        (*item_data).xDC8_word.flags.set_x1F(1);
    }
}

pub unsafe extern "C" fn Item_OnUserDataRemove(user_data: *mut c_void) {
    let item_data = user_data as *mut Item;
    ext::ScriptGame_HitRuleRetire(item_data as *mut c_void, core::mem::size_of::<Item>() as u32);

    if !(*item_data).xBBC_dynamicBoneTable.is_null() {
        ext::HSD_ObjFree(&raw mut item_dynamic_bones_alloc_data, (*item_data).xBBC_dynamicBoneTable as *mut c_void);
    }
    ext::HSD_ObjFree(&raw mut item_alloc_data, item_data as *mut c_void);
}

pub unsafe extern "C" fn Item_8026AE60() -> u32 {
    let result = ext::it_804D6D14;
    ext::it_804D6D14 = ext::it_804D6D14.wrapping_add(1);
    if ext::it_804D6D14 == 0 {
        ext::it_804D6D14 = ext::it_804D6D14.wrapping_add(1);
    }
    result
}

/// An item owned by an m-ex fighter plays that fighter's relative ids (5000..9999) from the fighter's bank.
pub unsafe extern "C" fn Item_MexSfx(item_data: *mut Item, sfx: enum_t) -> enum_t {
    let owner = (*item_data).mex_original_owner;
    if sfx >= 5000 && sfx < 10000 && !owner.is_null() && ext::ftLib_80086960(owner) != 0 {
        let ssm = ext::Mex_SsmForPortKind(ext::ftLib_GetKind(owner) as i32);
        if ssm >= 0 {
            return sfx.wrapping_sub(5000).wrapping_add(ssm.wrapping_mul(10000));
        }
    }
    sfx
}

pub unsafe extern "C" fn Item_8026AE84(item_data: *mut Item, sfx: enum_t, pan: u8, volume: u8) {
    let sfx = Item_MexSfx(item_data, sfx);
    if sfx != 540000 {
        if sfx != 540001 {
            (*item_data).xD6C = ext::lbAudioAx_800237A8(ext::lbAudioAx_800233EC(sfx), pan as i32, volume as i32);
        } else {
            (*item_data).xD6C = ext::lbAudioAx_800237A8(540000, pan as i32, volume as i32);
        }
    }
}

pub unsafe extern "C" fn Item_8026AF0C(item_data: *mut Item, sfx: enum_t, pan: u8, volume: u8) {
    let sfx = Item_MexSfx(item_data, sfx);
    if sfx != 540000 {
        if sfx != 540001 {
            if (*item_data).sfx_unk1 != SFX_NONE as _ {
                ext::lbAudioAx_800236B8((*item_data).sfx_unk1 as i32);
            }
            (*item_data).sfx_unk1 = ext::lbAudioAx_800237A8(ext::lbAudioAx_800233EC(sfx), pan as i32, volume as i32) as _;
        } else {
            Item_8026B034(item_data);
        }
    }
}

pub unsafe extern "C" fn Item_8026AFA0(item_data: *mut Item, sfx: enum_t, pan: u8, volume: u8) {
    let sfx = Item_MexSfx(item_data, sfx);
    if sfx != 540000 {
        if sfx != 540001 {
            if (*item_data).sfx_unk2 != SFX_NONE as _ {
                ext::lbAudioAx_800236B8((*item_data).sfx_unk2 as i32);
            }
            (*item_data).sfx_unk2 = ext::lbAudioAx_800237A8(ext::lbAudioAx_800233EC(sfx), pan as i32, volume as i32) as _;
        } else {
            Item_8026B074(item_data);
        }
    }
}

pub unsafe extern "C" fn Item_8026B034(item_data: *mut Item) {
    if (*item_data).sfx_unk1 != SFX_NONE as _ {
        ext::lbAudioAx_800236B8((*item_data).sfx_unk1 as i32);
    }
    (*item_data).sfx_unk1 = SFX_NONE as _;
}

pub unsafe extern "C" fn Item_8026B074(item_data: *mut Item) {
    if (*item_data).sfx_unk2 != SFX_NONE as _ {
        ext::lbAudioAx_800236B8((*item_data).sfx_unk2 as i32);
    }
    (*item_data).sfx_unk2 = SFX_NONE as _;
}

/// Stop All Item SFX
pub unsafe extern "C" fn Item_8026B0B4(gobj: *mut HSD_GObj) {
    let item_data = get_item(gobj);

    ext::lbAudioAx_80026510(gobj);

    if (*item_data).xD6C != SFX_NONE {
        ext::lbAudioAx_800265C4(gobj, (*item_data).xD6C);
        (*item_data).xD6C = SFX_NONE;
    }

    if (*item_data).sfx_unk1 != SFX_NONE as _ && ext::lbAudioAx_800265C4(gobj, (*item_data).sfx_unk1 as i32) == 1 {
        if (*item_data).sfx_unk1 != SFX_NONE as _ {
            ext::lbAudioAx_800236B8((*item_data).sfx_unk1 as i32);
        }
        (*item_data).sfx_unk1 = SFX_NONE as _;
    }

    if (*item_data).sfx_unk2 != SFX_NONE as _ && ext::lbAudioAx_800265C4(gobj, (*item_data).sfx_unk2 as i32) == 1 {
        if (*item_data).sfx_unk2 != SFX_NONE as _ {
            ext::lbAudioAx_800236B8((*item_data).sfx_unk2 as i32);
        }
        (*item_data).sfx_unk2 = SFX_NONE as _;
    }

    if (*item_data).sfx_unk1 != SFX_NONE as _ {
        ext::lbAudioAx_800236B8((*item_data).sfx_unk1 as i32);
    }
    (*item_data).sfx_unk1 = SFX_NONE as _;

    if (*item_data).sfx_unk2 != SFX_NONE as _ {
        ext::lbAudioAx_800236B8((*item_data).sfx_unk2 as i32);
    }
    (*item_data).sfx_unk2 = SFX_NONE as _;
    (*item_data).xD6C = SFX_NONE;
}

/// Check if item is grabbable
pub unsafe extern "C" fn Item_IsGrabbable(gobj: *mut HSD_GObj) -> bool {
    let temp_item = (*gobj).user_data as *mut Item;
    if (*temp_item).xDC8_word.flags.x15() != 0 && (*(*temp_item).xB8_itemLogicTable).picked_up.is_some() {
        return true;
    }
    false
}
