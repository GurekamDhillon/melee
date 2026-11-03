//! Native Geno translation. Shared records and bitfields come from melee_types.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code, unused_imports, unused_mut, unused_variables, unused_assignments, unused_parens, unused_unsafe, static_mut_refs, clippy::all)]
use core::ffi::c_void;
use melee_core::fp::fmadds;
pub use melee_types::*;
pub use melee_runtime::gekko::sqrtf;
macro_rules! OSReport { ($f:expr $(,$a:expr)* $(,)?) => {{
    if let Some(h) = unsafe { crate::tu::ext_hook::EXT_HOOK } { unsafe { h("OSReport", &[$f as usize as u64]); } } else {
    let text=unsafe { core::ffi::CStr::from_ptr($f) }.to_string_lossy();
    eprintln!("{} {:?}",text, &[$(crate::tu::ext_hook::Val::to_u64($a)),*] as &[u64]);
    }
}}; }
macro_rules! HSD_ForeachAnim { ($a:expr,$b:expr,$c:expr,$d:expr,$e:expr,$f:expr $(,)?) => {
    ext::HSD_ForeachAnim($a,$b,$c,$d,$e,crate::tu::ext_hook::Val::to_u64($f))
}; }
pub unsafe fn __assert(file:*const i8,line:u32,condition:*const i8)->! {
    melee_core::ct!("__assert");
    panic!("{}:{}: {}",core::ffi::CStr::from_ptr(file).to_string_lossy(),line,core::ffi::CStr::from_ptr(condition).to_string_lossy())
}
pub mod ext {
    use super::*;
    pub use super::__assert;
    /// Address-only marker for the OSReport mock (the TU calls it through the `OSReport!` macro).
    pub unsafe extern "C" fn OSReport(_a0: *const i8) {}
    macro_rules! hooks {
        ($(fn $n:ident($($a:ident:$t:ty),* $(,)?) $(->$r:ty)? $(= $def:path)?;)*) => {
            pub mod hook { use super::*; $(pub static mut $n: Option<fn($($t),*) $(->$r)?> = None;)* }
            $(pub unsafe extern "C" fn $n($($a:$t),*) $(->$r)? {
                if let Some(h) = $crate::tu::ext_hook::EXT_HOOK {
                    return <$crate::ext_rty!($($r)?) as $crate::tu::ext_hook::Val>::from_u64(
                        h(stringify!($n), &[$($crate::tu::ext_hook::Val::to_u64($a)),*]));
                }
                match hook::$n { Some(f)=>f($($a),*), None=>{ $crate::ext_dflt!([$($def)?] ($($a),*) $n) } }
            })*
            pub mod cb { pub use super::*; }
        }
    }
    hooks! {
fn Geno_SpecialSelect(
            p: i32,
            which_0: i32,
            i: i32,
        ) -> i32;
fn ScriptGame_NoteSpecialInput(fp_0: *mut Fighter) = crate::tu::adapt::ScriptGame_NoteSpecialInput__c7d07e;
fn diag_geno_stun(
            port: i32,
            base: i32,
            bonus: i32,
        );
fn Geno_ScriptAddressValid(address: u32, bytes: u32) -> i32;
fn GObj_Create(a0: u16, a1: u8, a2: u8) -> *mut HSD_GObj;
fn ScriptGame_HitRuleItemCreate(a0: *mut Item, a1: *mut HitCapsule) = crate::tu::adapt::ScriptGame_HitRuleItemCreate__a6017c;
fn ScriptGame_HitRuleDamageOnly(a0: *mut HitCapsule) = crate::tu::adapt::ScriptGame_HitRuleDamageOnly__c1c74e;
fn ScriptGame_HitRuleForget(a0: *mut HitCapsule) = crate::tu::adapt::ScriptGame_HitRuleForget__c0be11;
fn ProfBegin(a0: i32, a1: i32);
fn ProfEnd();
fn ProfActive() -> i32;
fn atan2f(a0: f32, a1: f32) -> f32 = crate::tu::adapt::atan2f__2f62eb;
fn cosf(a0: f32) -> f32 = crate::tu::adapt::cosf__b177e0;
fn sinf(a0: f32) -> f32 = crate::tu::adapt::sinf__245589;
fn ftAnim_8006F0FC(a0: *mut Fighter_GObj, a1: f32) = crate::tu::adapt::ftAnim_8006F0FC__39c58d;
fn ftAnim_SetAnimRate(a0: *mut Fighter_GObj, a1: f32) = crate::tu::adapt::ftAnim_SetAnimRate__352e8e;
fn ftAnim_IsFramesRemaining(a0: *mut Fighter_GObj) -> i32 = crate::tu::adapt::ftAnim_IsFramesRemaining__101630;
fn it_ResetStageArticleTable() = crate::tu::adapt::it_ResetStageArticleTable__fad078;
fn it_8026B40C(a0: *mut Article, a1: i32) = crate::tu::adapt::it_8026B40C__05f465;
fn it_8026B894(a0: *mut Item_GObj, a1: *mut HSD_GObj) -> i32;
fn it_8026BB68(a0: *mut Item_GObj, a1: *mut Vec) = crate::tu::adapt::it_8026BB68__233026;
fn HSD_DObjSetFlags(a0: *mut HSD_DObj, a1: u32);
fn Geno_DefineBaseKind(a0: i32) -> i32;
fn Fighter_ChangeMotionState(a0: *mut Fighter_GObj, a1: FtMotionId, a2: MotionFlags, a3: f32, a4: f32, a5: f32, a6: *mut Fighter_GObj) = crate::tu::adapt::Fighter_ChangeMotionState__c62dac;
fn ft_8008A2BC(a0: *mut Fighter_GObj) = crate::tu::adapt::ft_8008A2BC__51106d;
fn ftCo_800D105C(a0: *mut Fighter_GObj) = crate::tu::adapt::ftCo_800D105C__471042;
fn ftCommon_SetSelfMovementFromGroundedMovement(a0: *mut Fighter_GObj);
fn ftCommon_CalcSelfAccel_DeaccelAir(a0: *mut Fighter);
fn ftCommon_CalcSelfAccel_Drift(a0: *mut Fighter);
fn ftCommon_Fall(a0: *mut Fighter, a1: f32, a2: f32);
fn ftCommon_8007D5D4(a0: *mut Fighter) = crate::tu::adapt::ftCommon_8007D5D4__8cebce;
fn ftCommon_8007D7FC(a0: *mut Fighter) = crate::tu::adapt::ftCommon_8007D7FC__490b7b;
fn ftPartSetRotY(a0: *mut Fighter, a1: i32, a2: f32) = crate::tu::adapt::ftPartSetRotY__9ef908;
fn ftCo_8009917C(a0: *mut Fighter_GObj) -> i32 = crate::tu::adapt::ftCo_8009917C__02d948;
fn ftCo_Fall_Enter(a0: *mut Fighter_GObj) = crate::tu::adapt::ftCo_Fall_Enter__5f227e;
fn ftCo_800968C8(a0: *mut Fighter_GObj) = crate::tu::adapt::ftCo_800968C8__1d49d6;
fn ftCo_80096CC8(a0: *mut Fighter_GObj, a1: i32) -> i32 = crate::tu::adapt::ftCo_80096CC8__f60bbf;
fn ftCo_80096D28(a0: *mut Fighter_GObj) = crate::tu::adapt::ftCo_80096D28__30b1a1;
fn ftCo_Landing_Enter_Basic(a0: *mut Fighter_GObj) = crate::tu::adapt::ftCo_Landing_Enter_Basic__b5080f;
fn ftCo_LandingFallSpecial_Enter_Basic(a0: *mut Fighter_GObj) = crate::tu::adapt::ftCo_LandingFallSpecial_Enter_Basic__94c260;
fn ftCo_LandingFallSpecial_Enter(a0: *mut Fighter_GObj, a1: i32, a2: f32) = crate::tu::adapt::ftCo_LandingFallSpecial_Enter__90d150;
fn HSD_JObjSetupMatrixSub(a0: *mut HSD_JObj);
fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj);
fn HSD_JObjGetDObj(a0: *mut HSD_JObj) -> *mut HSD_DObj;
fn lbColl_80008440(a0: *mut HitCapsule) = crate::tu::adapt::lbColl_80008440__c0a79c;
fn HSD_Randf() -> f32;
fn HSD_Randi(a0: i32) -> i32;
fn Geno_ProfileForKind(a0: i32) -> i32;
fn Geno_MaxJumps(a0: i32) -> i32;
fn Geno_Scale(a0: i32) -> f32;
fn Geno_JumpVyCount(a0: i32) -> i32;
fn Geno_JumpVyBits(a0: i32, a1: i32) -> i32;
fn Geno_AttrCount(a0: i32) -> i32;
fn Geno_AttrIndex(a0: i32, a1: i32) -> i32;
fn Geno_AttrBits(a0: i32, a1: i32) -> i32;
fn Geno_HookCount(a0: i32, a1: i32) -> i32;
fn Geno_Hook(a0: i32, a1: i32, a2: i32) -> i32;
fn Geno_HookArg(a0: i32, a1: i32, a2: i32) -> i32;
fn Geno_Event(a0: i32, a1: i32, a2: i32, a3: i32, a4: i32);
fn Geno_SpecialCount(a0: i32) -> i32;
fn Geno_SpecialIndex(a0: i32, a1: i32) -> i32;
fn Geno_SpecialBits(a0: i32, a1: i32) -> i32;
fn Geno_OnLandCount(a0: i32) -> i32;
fn Geno_MotionAnim(a0: i32, a1: i32) -> i32;
fn Geno_OnLandFrom(a0: i32, a1: i32) -> i32;
fn Geno_OnLandTo(a0: i32, a1: i32) -> i32;
fn Geno_OverlayCount(a0: i32) -> i32;
fn Geno_OverlayAnim(a0: i32, a1: i32) -> i32;
fn Geno_OverlaySlot(a0: i32, a1: i32) -> i32;
fn Geno_SlotCount() -> i32;
fn Geno_SlotOffset(a0: i32) -> i32;
fn Geno_SlotLen(a0: i32) -> i32;
fn Geno_SlotRelative(a0: i32) -> i32;
fn Geno_PoolWords() -> i32;
fn Geno_PoolWord(a0: i32) -> i32;
fn Geno_PoolGen() -> i32;
fn Geno_StateCount(a0: i32) -> i32;
fn Geno_StateBehavior(a0: i32, a1: i32) -> i32;
fn Geno_StateCb(a0: i32, a1: i32, a2: i32) -> i32;
fn Geno_StateAnim(a0: i32, a1: i32) -> i32;
fn Geno_StateAnimFrom(a0: i32, a1: i32) -> i32;
fn Geno_StateLike(a0: i32, a1: i32) -> i32;
fn Geno_StateFlagsSet(a0: i32, a1: i32) -> i32;
fn Geno_StateFlags(a0: i32, a1: i32) -> i32;
fn Geno_StateMoveId(a0: i32, a1: i32) -> i32;
fn Geno_StateNext(a0: i32, a1: i32) -> i32;
fn Geno_StateLand(a0: i32, a1: i32) -> i32;
fn Geno_StateLagBits(a0: i32, a1: i32) -> i32;
fn Geno_StateMotion(a0: i32, a1: i32) -> i32;
fn Geno_StateGravityBits(a0: i32, a1: i32) -> i32;
fn Geno_ParamCount(a0: i32) -> i32;
fn Geno_ParamId(a0: i32, a1: i32) -> i32;
fn Geno_ParamBits(a0: i32, a1: i32) -> i32;
fn Geno_Special(a0: i32, a1: i32) -> i32;
fn Geno_ComboParam(a0: i32, a1: i32) -> i32;
fn Geno_ArticleCount(a0: i32) -> i32;
fn ft_80082C74(a0: *mut Fighter_GObj, a1: HSD_GObjEvent) = crate::tu::adapt::ft_80082C74__c5d8fd;
fn ft_80083090(a0: *mut Fighter_GObj, a1: Option<unsafe extern "C" fn(*mut Fighter_GObj, i32) -> i32>, a2: HSD_GObjEvent);
fn ft_8008403C(a0: *mut Fighter_GObj, a1: HSD_GObjEvent) = crate::tu::adapt::ft_8008403C__23dfed;
fn ft_80084104(a0: *mut Fighter_GObj) = crate::tu::adapt::ft_80084104__46f90b;
fn ft_80084280(a0: *mut Fighter_GObj) = crate::tu::adapt::ft_80084280__930c83;
fn ft_80084DB0(a0: *mut Fighter_GObj) = crate::tu::adapt::ft_80084DB0__838ea6;
fn ft_80084F3C(a0: *mut Fighter_GObj) = crate::tu::adapt::ft_80084F3C__408261;
fn ftCo_800D705C(a0: *mut Fighter_GObj) -> i32;
fn ftCo_800D7100(a0: *mut Fighter_GObj) -> i32;
fn mpCollSetFacingDir(a0: *mut CollData, a1: i32) = crate::tu::adapt::mpCollSetFacingDir__b3708c;
fn mpColl_80047AC8(a0: *mut CollData, a1: Option<unsafe extern "C" fn(*mut Fighter_GObj, enum_t) -> i32>, a2: *mut Fighter_GObj) -> i32;
fn mpColl_80047E14(a0: *mut CollData, a1: Option<unsafe extern "C" fn(*mut Fighter_GObj, enum_t) -> i32>, a2: *mut Fighter_GObj) -> i32;
fn ft_80081A00(a0: *mut Fighter_GObj) -> i32 = crate::tu::adapt::ft_80081A00__8629a6;
fn ftCliffCommon_80081298(a0: *mut Fighter_GObj) -> i32 = crate::tu::adapt::ftCliffCommon_80081298__08ff6d;
fn Geno_ProfileCount() -> i32;
fn Geno_WarnIasa(a0: i32, a1: i32);
fn ftColl_CreateReflectHit(a0: *mut Fighter_GObj, a1: *mut ReflectDesc, a2: HSD_GObjEvent);
fn ftCo_800C3B10(a0: *mut Fighter_GObj) -> i32 = crate::tu::adapt::ftCo_800C3B10__3d9d5c;
fn ftCo_AttackS4_CheckInput(a0: *mut Fighter_GObj) -> i32 = crate::tu::adapt::ftCo_AttackS4_CheckInput__48fe9d;
fn ftCo_Dash_CheckInput(a0: *mut Fighter_GObj) -> i32 = crate::tu::adapt::ftCo_Dash_CheckInput__1e8d28;
fn ftCo_80095328(a0: *mut Fighter_GObj, a1: *mut i32) -> i32;
fn ftCo_Jump_CheckInput(a0: *mut Fighter_GObj) -> i32 = crate::tu::adapt::ftCo_Jump_CheckInput__ff2f50;
fn ftCo_Jump_Enter(a0: *mut Fighter_GObj) = crate::tu::adapt::ftCo_Jump_Enter__230c83;
fn ftCo_800CB870(a0: *mut Fighter_GObj) -> i32 = crate::tu::adapt::ftCo_800CB870__e3acbf;
fn ftCo_JumpAerial_Enter_Basic(a0: *mut Fighter_GObj) = crate::tu::adapt::ftCo_JumpAerial_Enter_Basic__4afd40;
fn ftCo_Turn_CheckInput(a0: *mut Fighter_GObj) -> i32 = crate::tu::adapt::ftCo_Turn_CheckInput__96ef34;
fn ftCo_Walk_CheckInput(a0: *mut Fighter_GObj) -> i32 = crate::tu::adapt::ftCo_Walk_CheckInput__5c6c59;
fn Geno_LuaFn(a0: i32, a1: i32, a2: i32) -> i32;
fn Geno_LuaSlots(a0: i32) -> i32;
fn Geno_LuaCall(a0: i32, a1: i32, a2: i32, a3: *mut c_void) -> i32;
fn Geno_DefineOverrideCount(a0: i32) -> i32;
fn Geno_DefineOverrideField(a0: i32, a1: i32, a2: i32) -> i32;
fn db_80225DD8(a0: *mut Item_GObj, a1: *mut Fighter_GObj);
fn efAsync_QueueFlush(a0: *mut HSD_GObj, a1: *mut c_void);
fn efAsync_MexSpawn(a0: *mut HSD_GObj, a1: *mut c_void, a2: i32, a3: *mut HSD_JObj, a4: *mut Vec, a5: f32, a6: f32);
fn lb_8000B1CC(a0: *mut HSD_JObj, a1: *mut Vec, a2: *mut Vec) = crate::tu::adapt::lb_8000B1CC__23547e;
fn hsd_8039D5DC(a0: *mut HSD_JObj);
fn HSD_CObjGetEyePosition(a0: *mut HSD_CObj, a1: *mut Vec);
fn HSD_CObjGetViewingMtxPtr(a0: *mut HSD_CObj) -> MtxPtr;
fn HSD_CObjGetCurrent() -> *mut HSD_CObj = crate::tu::adapt::HSD_CObjGetCurrent__6f08dd;
fn GObj_SetupGXLink(a0: *mut HSD_GObj, a1: GObj_RenderFunc, a2: u8, a3: u32);
fn ft_PlaySFX(a0: *mut Fighter, a1: enum_t, a2: u8, a3: u8);
fn it_8026FCF8(a0: *mut Item, a1: *mut HitCapsule) = crate::tu::adapt::it_8026FCF8__a44c9f;
fn it_8027129C(a0: *mut Item_GObj, a1: i32) = crate::tu::adapt::it_8027129C__0e4566;
fn it_80272460(a0: *mut HitCapsule, a1: u32, a2: *mut Item_GObj) = crate::tu::adapt::it_80272460__3b2b2c;
fn it_8026E9A4(a0: *mut HSD_GObj, a1: *mut Vec, a2: *mut Vec, a3: *mut Vec) -> i32 = crate::tu::adapt::it_8026E9A4__61791d;
fn it_80275594(a0: *mut Item_GObj, a1: i32, a2: f32);
fn Item_80267978(a0: *mut HSD_GObj) = crate::tu::adapt::Item_80267978__71f703;
fn Item_80268B18(a0: *mut SpawnItem) -> *mut Item_GObj = crate::tu::adapt::Item_80268B18__f9c679;
fn Item_80268E5C(a0: *mut HSD_GObj, a1: enum_t, a2: Item_StateChangeFlags) = crate::tu::adapt::Item_80268E5C__0d30a0;
fn Item_8026A8EC(a0: *mut Item_GObj) = crate::tu::adapt::Item_8026A8EC__e93779;
fn Camera_800310B8() -> *mut HSD_CObj;
fn mpCheckFloor(a0: f32, a1: f32, a2: f32, a3: f32, a4: f32, a5: *mut Vec, a6: *mut i32, a7: *mut u32, a8: *mut Vec, a9: i32, a10: i32, a11: i32, a12: Option<unsafe extern "C" fn(*mut Fighter_GObj, i32) -> i32>, a13: *mut Fighter_GObj) -> i32;
fn mpCheckCeiling(a0: f32, a1: f32, a2: f32, a3: f32, a4: *mut Vec, a5: *mut i32, a6: *mut u32, a7: *mut Vec, a8: i32, a9: i32) -> i32 = crate::tu::adapt::mpCheckCeiling__94c9dc;
fn mpCheckLeftWall(a0: f32, a1: f32, a2: f32, a3: f32, a4: *mut Vec, a5: *mut i32, a6: *mut u32, a7: *mut Vec, a8: i32, a9: i32) -> i32 = crate::tu::adapt::mpCheckLeftWall__d5d908;
fn mpCheckRightWall(a0: f32, a1: f32, a2: f32, a3: f32, a4: *mut Vec, a5: *mut i32, a6: *mut u32, a7: *mut Vec, a8: i32, a9: i32) -> i32 = crate::tu::adapt::mpCheckRightWall__f24271;
fn mpGetSpeed(a0: i32, a1: *mut Vec, a2: *mut Vec) -> i32 = crate::tu::adapt::mpGetSpeed__f20f7d;
fn Geno_ArticleParam(a0: i32, a1: i32, a2: i32) -> i32;
fn Geno_ArticleHitCount(a0: i32, a1: i32) -> i32;
fn Geno_ArticleHitParam(a0: i32, a1: i32, a2: i32, a3: i32) -> i32;
fn Geno_ArticleJoint(a0: i32, a1: i32) -> i32;
fn Geno_StateCounter(a0: i32, a1: i32, a2: i32) -> i32;
fn Geno_StateHasCounter(a0: i32, a1: i32) -> i32;
fn Fx_Attach(a0: i32, a1: i32, a2: i32, a3: i32, a4: i32) -> i32;
fn Fx_Detach(a0: i32);
fn Fx_SetOwnerPort(a0: i32, a1: i32);
fn Fx_Frame(a0: i32);
fn Fx_Draw(a0: i32);
fn Fx_Stat(a0: i32) -> i32;
fn Fx_SetOwner(a0: i32, a1: i32, a2: i32);
fn diag_fx_cam(a0: i32, a1: i32, a2: i32, a3: f32, a4: f32, a5: f32, a6: f32, a7: f32, a8: f32);
fn Geno_FxBindings(a0: i32) -> i32;
fn Fx_Drive(a0: i32, a1: i32, a2: i32, a3: i32, a4: f32, a5: i32, a6: i32, a7: i32, a8: i32, a9: i32, a10: i32, a11: i32, a12: i32, a13: i32, a14: i32);
fn Player_GetEntity(a0: i32) -> *mut HSD_GObj = crate::tu::adapt::Player_GetEntity__090538;
fn Fx_Owner(a0: i32) -> i32;
fn Geno_ProfileIdWord(a0: i32) -> u32;
fn Item_ScriptSpawnAtCap(a0: *mut SpawnItem) -> i32 = crate::tu::adapt::Item_ScriptSpawnAtCap__e6349b;
fn Item_ScriptSpawnCapTest() -> i32 = crate::tu::adapt::Item_ScriptSpawnCapTest__c9b323;
fn HSD_StateInvalidate(a0: i32);
fn Geno_ItemParam(a0: i32, a1: i32) -> i32;
fn Mex_ItemRangeAvailable(a0: i32, a1: i32) -> i32;
fn ItemEvent(a0: i32, a1: i32, a2: i32, a3: i32, a4: i32, a5: i32, a6: i32, a7: i32);
fn Mex_ItemReady(a0: i32) -> i32;
fn ftCo_IsCpuControlled(a0: *mut Fighter) -> i32;
fn ItemDraw(a0: i32, a1: i32, a2: i32);
fn snap_open(a0: i32) -> i32;
fn snap_save(a0: i32);
fn snap_load(a0: i32) -> i32;
fn Geno_ComboWord(a0: i32, a1: i32, a2: i32) -> i32;
fn Geno_ComboNote(a0: i32, a1: i32, a2: i32, a3: i32, a4: i32, a5: i32);
fn ftCo_AttackAir_EnterFromMsid(a0: *mut Fighter_GObj, a1: FtMotionId) = crate::tu::adapt::ftCo_AttackAir_EnterFromMsid__52a3a7;
fn __frsqrte(a0: f64) -> f64;
fn Geno_DefineVoiceState(a0: i32, a1: u32, a2: i32) -> i32;
fn efAsync_Spawn(a0: *mut HSD_GObj, a1: *mut c_void, a2: u32, a3: u32, a4: *mut HSD_JObj);
fn HSD_ForeachAnim(a0: *mut c_void, a1: HSD_Type, a2: HSD_TypeMask, a3: *mut c_void, a4: AObj_Arg_Type, a5: u64) = crate::tu::adapt::HSD_ForeachAnim__175c67;
}
pub use melee_runtime::tu::gobj_native::HSD_GObjPLinkHead;
pub use melee_runtime::tu::gobj_native::HSD_GObj_CurrentInvokedProc;
pub use melee_fighter::hooks::FT_PARTS_TABLE as ftPartsTable;
pub use melee_runtime::shared::p_ftCommonData;
pub use melee_world::tu::mpcoll::mpColl_LedgeLevelOK;
pub static mut ftData_MotionStateList: [MotionState; 400] = unsafe { core::mem::zeroed() };
pub static mut ftMr_Init_MotionStateTable: [MotionState; 400] = unsafe { core::mem::zeroed() };
pub static mut ftCa_Init_MotionStateTable: [MotionState; 400] = unsafe { core::mem::zeroed() };
pub static mut ftFx_Init_MotionStateTable: [MotionState; 400] = unsafe { core::mem::zeroed() };
pub static mut ftFc_Init_MotionStateTable: [MotionState; 400] = unsafe { core::mem::zeroed() };
pub static mut ftMs_Init_MotionStateTable: [MotionState; 400] = unsafe { core::mem::zeroed() };
pub static mut ftPr_Init_MotionStateTable: [MotionState; 400] = unsafe { core::mem::zeroed() };
pub static mut ftSk_Init_MotionStateTable: [MotionState; 400] = unsafe { core::mem::zeroed() };
pub static mut ftZd_Init_MotionStateTable: [MotionState; 400] = unsafe { core::mem::zeroed() };
pub use melee_runtime::tu::generator::hsd_804D78FC;
pub use melee_runtime::tu::particle::hsd_804D0908;
pub use melee_items::tu::it_3F14::it_803F14C4;
pub use melee_world::itacc::it_804A0F60;
pub use melee_items::tu::it_3F14::it_804D6D24;
pub use melee_runtime::shared::it_804D6D28;
}
use ext::*;
mod data;
pub use data::{gb_sinq};
pub const M_PI: f64 = 3.14159265358979323846f64;

pub const HSD_PAD_XY: i32 = HSD_PAD_X | HSD_PAD_Y;

pub const GENO_FTCMD_OP: i32 = 59 as i32;

pub const GENO_BANK_LA_INT: i32 = 0;

pub const GENO_BANK_RA_INT: i32 = 1;

pub const GENO_BANK_LA_FLOAT: i32 = 2 as i32;

pub const GENO_VARS_PER_BANK: i32 = 64 as i32;

pub const GENO_SPECIAL_WORDS: i32 = 265 as i32;

pub const GENO_BTN_ATTACK: i32 = 0x1 as i32;

pub const GENO_BTN_SPECIAL: i32 = 0x2 as i32;

pub const GENO_BTN_JUMP: i32 = 0x4 as i32;

pub const GENO_BTN_SHIELD: i32 = 0x8 as i32;

pub const GENO_BTN_GRAB: i32 = 0x10 as i32;

pub const GENO_BTN_TAUNT: i32 = 0x20 as i32;

pub const GENO_BTN_BLAST: i32 = 0x40 as i32;

pub const GENO_CHG_NOT: i32 = 0x8 as i32;

pub const GENO_CHG_ONCE: i32 = 0x4 as i32;

pub const GENO_TGT_MOTION: u32 = 0 as u32;

pub const GENO_TGT_SPECIAL: u32 = 1 as u32;

pub const GENO_TGT_GENO: u32 = 2 as u32;

pub const GENO_TGT_AUTO: u32 = 0xfffffffe as u32;

pub const GENO_TGT_HELPLESS: u32 = 0xfffffffd as u32;

pub const GENO_TGT_ROLL: u32 = 0xfffffffb as u32;

pub const GENO_TGT_STAY: u32 = 0xfffffffc as u32;

pub const GENO_TGT_RAW: u32 = 0x8000000 as u32;

pub const GENO_TGT_KEEP_FRAME: u32 = 0x4000000 as u32;

pub const GENO_MAX_CHECKS: i32 = 8 as i32;

pub const GENO_CHECK_CONDS: i32 = 3 as i32;

pub const GENO_MAX_REHIT: i32 = 4 as i32;

pub const GENO_HBF_NO_HITLAG: i32 = 1 as i32;

pub const GENO_HBF_FLINCHLESS: i32 = 2 as i32;

pub const GENO_HBF_FORCE_REACTION: i32 = 8 as i32;

pub const GENO_MODE_SKIP: i32 = 2 as i32;

pub const GENO_MAX_PROFILES: i32 = 65535 as i32;

pub const GENO_POOL_WORDS: i32 = 65536 as i32;

pub const GENO_MAX_SLOTS: i32 = 2560 as i32;

pub const GENO_MOTION_BASE: i32 = 0x400 as i32;

pub const GENO_MAX_STATES: i32 = 48 as i32;

pub const GENO_MOVE_VARS: i32 = 8 as i32;

pub const GENO_CB_LIKE: i32 = 0xff as i32;

pub const GENO_PARAMS: i32 = 128 as i32;

pub const GENO_MOTION_LEDGE_MASK: u32 = 3 as u32;

pub const GENO_MOTION_LIFTOFF: u32 = 4 as u32;

pub const GENO_MOTION_ORIGIN: u32 = 8 as u32;

pub const GENO_MOTION_ENTRY_FACING: u32 = 16 as u32;

pub const GENO_ART_KIND_BASE: i32 = 0x1000 as i32;

pub const GENO_MAX_ARTICLES: i32 = 16 as i32;

pub const GENO_ART_PER_RANGE: i32 = 8 as i32;

pub const GENO_ART_KIND_BASE2: i32 = GENO_ART_KIND_BASE
    + 32 as i32 * GENO_ART_PER_RANGE;

pub const GENO_ART_KIND_END: i32 = GENO_ART_KIND_BASE2
    + 32 as i32 * GENO_ART_PER_RANGE;

pub const GENO_ART_EXTRA_BASE: i32 = 0x20000 as i32;

pub const GENO_ART_EXTRA_END: i32 = GENO_ART_EXTRA_BASE
    + (GENO_MAX_PROFILES - 32 as i32) * 16 as i32;

pub const GENO_ART_HITBOXES: i32 = 4 as i32;

pub const GENO_ART_HIT_ENTRIES: i32 = 8 as i32;

pub const GENO_ART_CHILDREN: i32 = 2 as i32;

pub const GENO_ART_DESPAWN_HIT: u32 = 1 as u32;

pub const GENO_ART_DESPAWN_SHIELD: u32 = 2 as u32;

pub const GENO_ART_DESPAWN_STAGE: u32 = 4 as u32;

pub const GENO_ART_DESPAWN_CLANK: u32 = 8 as u32;

pub const GENO_AHF_GROUND: u32 = 1 as u32;

pub const GENO_AHF_AIR: u32 = 2 as u32;

pub const GENO_AHF_REFLECT: u32 = 4 as u32;

pub const GENO_AHF_ABSORB: u32 = 8 as u32;

pub const GENO_AHF_COUNTER: u32 = 16 as u32;

pub const GENO_LUA_STATE_SLOTS: i32 = 16 as i32;

pub const GENO_LUA_PHASE_ENTER: i32 = 0 as i32;

pub const GENO_LUA_PHASE_FRAME: i32 = 1 as i32;

pub const GENO_LUA_MAX_CMDS: i32 = 8 as i32;

pub const GENO_LUA_IO_WORDS: i32 = 64 as i32;

pub const GENO_COMBO_MAX_ROUTES: i32 = 128 as i32;

pub const GENO_COMBO_TGT_ACT: u32 = 12 as u32;

pub const GENO_COMBO_FROM_ANY: u32 = 0xffff as u32;

pub const GENO_SF_SCRIPT: u32 = 1 as u32;

pub const GENO_PLAYERS: i32 = 6 as i32;

pub const GENO_NHOOKS: i32 = (::core::mem::size_of::<
    [Anon_anon_333; 7],
>() as usize)
    .wrapping_div(::core::mem::size_of::<Anon_anon_333>() as usize)
    as i32;

pub const GENO_NATTRS: i32 = (::core::mem::size_of::<
    [Anon_anon_766; 76],
>() as usize)
    .wrapping_div(::core::mem::size_of::<Anon_anon_766>() as usize)
    as i32;

pub const GENO_PROFILE_STORAGE_BYTES: u32 = (2 as u32)
    .wrapping_mul(1024 as u32)
    .wrapping_mul(1024 as u32);

pub const GENO_TGT_BHV: u32 = 3 as u32;

pub const GENO_AUTO: u32 = GENO_TGT_AUTO;

pub const GENO_NONE_T: u32 = 0xffffffff as u32;

pub const GENO_NPARAMDEFS: i32 = (::core::mem::size_of::<
    [GenoBhvParamDef; 90],
>() as usize)
    .wrapping_div(::core::mem::size_of::<GenoBhvParamDef>() as usize)
    as i32;

pub const GENO_BHV_SINQ_N: i32 = 1025 as i32;

pub const GENO_BHV_DZ2: f32 = 0.0826f32;

pub const GENO_NCBS: i32 = (::core::mem::size_of::<[GenoCbDef; 45]>()
    as usize)
    .wrapping_div(::core::mem::size_of::<GenoCbDef>() as usize) as i32;

pub const GENO_NBHVS: i32 = (::core::mem::size_of::<[GenoBhvDef; 17]>()
    as usize)
    .wrapping_div(::core::mem::size_of::<GenoBhvDef>() as usize) as i32;

pub const EF_MEX_MDL_START: i32 = 5000 as i32;

pub const EF_MEX_END: i32 = 9000 as i32;

pub const GENO_NSF: i32 = (::core::mem::size_of::<
    [Anon_anon_2375; 42],
>() as usize)
    .wrapping_div(::core::mem::size_of::<Anon_anon_2375>() as usize)
    as i32;

pub const GENO_ITEM_KIND_BASE: i32 = GENO_ART_KIND_END;

pub const GENO_ITEM_MAX_DEFS: i32 = 64 as i32;

pub const GENO_COMBO_STALE: i32 = 255 as i32;

pub const NULL: i32 = 0 as i32;

pub static mut Ft_MF_None: MotionFlags = 0 as MotionFlags;

pub static mut Ft_MF_KeepFastFall: MotionFlags = ((1 as i32)
    << 0 as i32) as MotionFlags;

pub static mut Ft_MF_KeepGfx: MotionFlags = ((1 as i32)
    << 1 as i32) as MotionFlags;

pub static mut Ft_MF_SkipHit: MotionFlags = ((1 as i32)
    << 3 as i32) as MotionFlags;

pub static mut Ft_MF_SkipMatAnim: MotionFlags = ((1 as i32)
    << 7 as i32) as MotionFlags;

pub static mut Ft_MF_KeepSfx: MotionFlags = ((1 as i32)
    << 9 as i32) as MotionFlags;

pub static mut Ft_MF_SkipColAnim: MotionFlags = ((1 as i32)
    << 12 as i32) as MotionFlags;

pub static mut Ft_MF_UpdateCmd: MotionFlags = ((1 as i32)
    << 14 as i32) as MotionFlags;

pub static mut Ft_MF_SkipNametagVis: MotionFlags = ((1 as i32)
    << 15 as i32) as MotionFlags;

pub static mut Ft_MF_KeepSwordTrail: MotionFlags = ((1 as i32)
    << 17 as i32) as MotionFlags;

pub static mut Ft_MF_SkipItemVis: MotionFlags = ((1 as i32)
    << 18 as i32) as MotionFlags;

pub static mut Ft_MF_SkipModelPartVis: MotionFlags = ((1 as i32)
    << 22 as i32) as MotionFlags;

pub static mut Ft_MF_SkipAttackCount: MotionFlags = ((1 as i32)
    << 25 as i32) as MotionFlags;

pub static mut Geno_StateBlock: [[GenoState; 2]; 6] = unsafe { core::mem::zeroed() };

pub static mut Geno_LuaBlock: [[GenoLuaBlock; 2]; 6] = unsafe { core::mem::zeroed() };

pub static mut geno_hooks: [Anon_anon_333; 7] = unsafe {
    [
        Anon_anon_333 {
            id: GENO_HOOK_LOG as i32,
            name: b"geno.log\0" as *const u8 as *const i8,
            r#fn: Some(
                hook_log
                    as unsafe extern "C" fn(
                        *mut Fighter_GObj,
                        *mut Fighter,
                        *mut GenoState,
                        i32,
                    ) -> i32,
            ),
        },
        Anon_anon_333 {
            id: GENO_HOOK_JUMPS_REFILL as i32,
            name: b"geno.jumps.refill\0" as *const u8 as *const i8,
            r#fn: Some(
                hook_jumps_refill
                    as unsafe extern "C" fn(
                        *mut Fighter_GObj,
                        *mut Fighter,
                        *mut GenoState,
                        i32,
                    ) -> i32,
            ),
        },
        Anon_anon_333 {
            id: GENO_HOOK_JUMPS_TO_VAR as i32,
            name: b"geno.jumps.to_var\0" as *const u8 as *const i8,
            r#fn: Some(
                hook_jumps_to_var
                    as unsafe extern "C" fn(
                        *mut Fighter_GObj,
                        *mut Fighter,
                        *mut GenoState,
                        i32,
                    ) -> i32,
            ),
        },
        Anon_anon_333 {
            id: GENO_HOOK_COUNT_FRAMES as i32,
            name: b"geno.count_frames\0" as *const u8 as *const i8,
            r#fn: Some(
                hook_count_frames
                    as unsafe extern "C" fn(
                        *mut Fighter_GObj,
                        *mut Fighter,
                        *mut GenoState,
                        i32,
                    ) -> i32,
            ),
        },
        Anon_anon_333 {
            id: GENO_HOOK_ARTICLE_SPAWN as i32,
            name: b"geno.article.spawn\0" as *const u8 as *const i8,
            r#fn: Some(
                hook_article_spawn
                    as unsafe extern "C" fn(
                        *mut Fighter_GObj,
                        *mut Fighter,
                        *mut GenoState,
                        i32,
                    ) -> i32,
            ),
        },
        Anon_anon_333 {
            id: GENO_HOOK_LOCKON as i32,
            name: b"geno.lockon\0" as *const u8 as *const i8,
            r#fn: Some(
                hook_lockon
                    as unsafe extern "C" fn(
                        *mut Fighter_GObj,
                        *mut Fighter,
                        *mut GenoState,
                        i32,
                    ) -> i32,
            ),
        },
        Anon_anon_333 {
            id: GENO_HOOK_AIM_STICK as i32,
            name: b"geno.aim_stick\0" as *const u8 as *const i8,
            r#fn: Some(
                hook_aim_stick
                    as unsafe extern "C" fn(
                        *mut Fighter_GObj,
                        *mut Fighter,
                        *mut GenoState,
                        i32,
                    ) -> i32,
            ),
        },
    ]
};

pub static mut Geno_ScriptPool: [u32; 65536] = unsafe { core::mem::zeroed() };

pub static mut Geno_ScriptPoolGen: i32 = unsafe { core::mem::zeroed() };

pub static mut Geno_OverlayOrig: [*mut CmdUnion; 2560] = unsafe { core::mem::zeroed() };

pub static mut geno_attrs: [Anon_anon_766; 76] = [
    Anon_anon_766 {
        name: b"walk_accel_mul\0" as *const u8 as *const i8,
        offset: 0 as u64 as i32,
        is_int: 0 as i32,
    },
    Anon_anon_766 {
        name: b"walk_accel_base\0" as *const u8 as *const i8,
        offset: 4 as u64 as i32,
        is_int: 0 as i32,
    },
    Anon_anon_766 {
        name: b"walk_max_vel\0" as *const u8 as *const i8,
        offset: 8 as u64 as i32,
        is_int: 0 as i32,
    },
    Anon_anon_766 {
        name: b"slow_walk_max\0" as *const u8 as *const i8,
        offset: 12 as u64 as i32,
        is_int: 0 as i32,
    },
    Anon_anon_766 {
        name: b"mid_walk_point\0" as *const u8 as *const i8,
        offset: 16 as u64 as i32,
        is_int: 0 as i32,
    },
    Anon_anon_766 {
        name: b"fast_walk_min\0" as *const u8 as *const i8,
        offset: 20 as u64 as i32,
        is_int: 0 as i32,
    },
    Anon_anon_766 {
        name: b"ground_friction\0" as *const u8 as *const i8,
        offset: 24 as u64 as i32,
        is_int: 0 as i32,
    },
    Anon_anon_766 {
        name: b"dash_initial_velocity\0" as *const u8 as *const i8,
        offset: 28 as u64 as i32,
        is_int: 0 as i32,
    },
    Anon_anon_766 {
        name: b"dash_accel_mul\0" as *const u8 as *const i8,
        offset: 32 as u64 as i32,
        is_int: 0 as i32,
    },
    Anon_anon_766 {
        name: b"dash_accel_base\0" as *const u8 as *const i8,
        offset: 36 as u64 as i32,
        is_int: 0 as i32,
    },
    Anon_anon_766 {
        name: b"dash_max_velocity\0" as *const u8 as *const i8,
        offset: 40 as u64 as i32,
        is_int: 0 as i32,
    },
    Anon_anon_766 {
        name: b"run_animation_scaling\0" as *const u8 as *const i8,
        offset: 44 as u64 as i32,
        is_int: 0 as i32,
    },
    Anon_anon_766 {
        name: b"max_run_brake_frames\0" as *const u8 as *const i8,
        offset: 48 as u64 as i32,
        is_int: 0 as i32,
    },
    Anon_anon_766 {
        name: b"ground_max_horizontal_velocity\0" as *const u8
            as *const i8,
        offset: 52 as u64 as i32,
        is_int: 0 as i32,
    },
    Anon_anon_766 {
        name: b"jump_startup_time\0" as *const u8 as *const i8,
        offset: 56 as u64 as i32,
        is_int: 0 as i32,
    },
    Anon_anon_766 {
        name: b"jump_h_initial_velocity\0" as *const u8 as *const i8,
        offset: 60 as u64 as i32,
        is_int: 0 as i32,
    },
    Anon_anon_766 {
        name: b"jump_v_initial_velocity\0" as *const u8 as *const i8,
        offset: 64 as u64 as i32,
        is_int: 0 as i32,
    },
    Anon_anon_766 {
        name: b"ground_to_air_jump_momentum_multiplier\0" as *const u8
            as *const i8,
        offset: 68 as u64 as i32,
        is_int: 0 as i32,
    },
    Anon_anon_766 {
        name: b"jump_h_max_velocity\0" as *const u8 as *const i8,
        offset: 72 as u64 as i32,
        is_int: 0 as i32,
    },
    Anon_anon_766 {
        name: b"hop_v_initial_velocity\0" as *const u8 as *const i8,
        offset: 76 as u64 as i32,
        is_int: 0 as i32,
    },
    Anon_anon_766 {
        name: b"air_jump_v_multiplier\0" as *const u8 as *const i8,
        offset: 80 as u64 as i32,
        is_int: 0 as i32,
    },
    Anon_anon_766 {
        name: b"air_jump_h_multiplier\0" as *const u8 as *const i8,
        offset: 84 as u64 as i32,
        is_int: 0 as i32,
    },
    Anon_anon_766 {
        name: b"max_jumps\0" as *const u8 as *const i8,
        offset: 88 as u64 as i32,
        is_int: 1 as i32,
    },
    Anon_anon_766 {
        name: b"gravity\0" as *const u8 as *const i8,
        offset: 92 as u64 as i32,
        is_int: 0 as i32,
    },
    Anon_anon_766 {
        name: b"terminal_velocity\0" as *const u8 as *const i8,
        offset: 96 as u64 as i32,
        is_int: 0 as i32,
    },
    Anon_anon_766 {
        name: b"air_drift_stick_mul\0" as *const u8 as *const i8,
        offset: 100 as u64 as i32,
        is_int: 0 as i32,
    },
    Anon_anon_766 {
        name: b"aerial_drift_base\0" as *const u8 as *const i8,
        offset: 104 as u64 as i32,
        is_int: 0 as i32,
    },
    Anon_anon_766 {
        name: b"air_drift_max\0" as *const u8 as *const i8,
        offset: 108 as u64 as i32,
        is_int: 0 as i32,
    },
    Anon_anon_766 {
        name: b"aerial_friction\0" as *const u8 as *const i8,
        offset: 112 as u64 as i32,
        is_int: 0 as i32,
    },
    Anon_anon_766 {
        name: b"fast_fall_velocity\0" as *const u8 as *const i8,
        offset: 116 as u64 as i32,
        is_int: 0 as i32,
    },
    Anon_anon_766 {
        name: b"air_max_horizontal_velocity\0" as *const u8
            as *const i8,
        offset: 120 as u64 as i32,
        is_int: 0 as i32,
    },
    Anon_anon_766 {
        name: b"jab_2_input_window\0" as *const u8 as *const i8,
        offset: 124 as u64 as i32,
        is_int: 0 as i32,
    },
    Anon_anon_766 {
        name: b"jab_3_input_window\0" as *const u8 as *const i8,
        offset: 128 as u64 as i32,
        is_int: 0 as i32,
    },
    Anon_anon_766 {
        name: b"standing_turn_frames\0" as *const u8 as *const i8,
        offset: 132 as u64 as i32,
        is_int: 0 as i32,
    },
    Anon_anon_766 {
        name: b"weight\0" as *const u8 as *const i8,
        offset: 136 as u64 as i32,
        is_int: 0 as i32,
    },
    Anon_anon_766 {
        name: b"model_scaling\0" as *const u8 as *const i8,
        offset: 140 as u64 as i32,
        is_int: 0 as i32,
    },
    Anon_anon_766 {
        name: b"initial_shield_size\0" as *const u8 as *const i8,
        offset: 144 as u64 as i32,
        is_int: 0 as i32,
    },
    Anon_anon_766 {
        name: b"shield_break_initial_velocity\0" as *const u8
            as *const i8,
        offset: 148 as u64 as i32,
        is_int: 0 as i32,
    },
    Anon_anon_766 {
        name: b"rapid_jab_window\0" as *const u8 as *const i8,
        offset: 152 as u64 as i32,
        is_int: 1 as i32,
    },
    Anon_anon_766 {
        name: b"clank_animation_length\0" as *const u8 as *const i8,
        offset: 156 as u64 as i32,
        is_int: 0 as i32,
    },
    Anon_anon_766 {
        name: b"hit_spark_variant\0" as *const u8 as *const i8,
        offset: 160 as u64 as i32,
        is_int: 1 as i32,
    },
    Anon_anon_766 {
        name: b"ledge_jump_horizontal_velocity\0" as *const u8
            as *const i8,
        offset: 168 as u64 as i32,
        is_int: 0 as i32,
    },
    Anon_anon_766 {
        name: b"ledge_jump_vertical_velocity\0" as *const u8
            as *const i8,
        offset: 172 as u64 as i32,
        is_int: 0 as i32,
    },
    Anon_anon_766 {
        name: b"item_throw_velocity_multiplier\0" as *const u8
            as *const i8,
        offset: 176 as u64 as i32,
        is_int: 0 as i32,
    },
    Anon_anon_766 {
        name: b"heavy_throw_velocity_multiplier\0" as *const u8
            as *const i8,
        offset: 180 as u64 as i32,
        is_int: 0 as i32,
    },
    Anon_anon_766 {
        name: b"specials_ground_speed_retention\0" as *const u8
            as *const i8,
        offset: 184 as u64 as i32,
        is_int: 0 as i32,
    },
    Anon_anon_766 {
        name: b"kirby_b_star_damage\0" as *const u8 as *const i8,
        offset: 224 as u64 as i32,
        is_int: 0 as i32,
    },
    Anon_anon_766 {
        name: b"normal_landing_lag\0" as *const u8 as *const i8,
        offset: 228 as u64 as i32,
        is_int: 0 as i32,
    },
    Anon_anon_766 {
        name: b"landingairn_lag\0" as *const u8 as *const i8,
        offset: 232 as u64 as i32,
        is_int: 0 as i32,
    },
    Anon_anon_766 {
        name: b"landingairf_lag\0" as *const u8 as *const i8,
        offset: 236 as u64 as i32,
        is_int: 0 as i32,
    },
    Anon_anon_766 {
        name: b"landingairb_lag\0" as *const u8 as *const i8,
        offset: 240 as u64 as i32,
        is_int: 0 as i32,
    },
    Anon_anon_766 {
        name: b"landingairhi_lag\0" as *const u8 as *const i8,
        offset: 244 as u64 as i32,
        is_int: 0 as i32,
    },
    Anon_anon_766 {
        name: b"landingairlw_lag\0" as *const u8 as *const i8,
        offset: 248 as u64 as i32,
        is_int: 0 as i32,
    },
    Anon_anon_766 {
        name: b"name_tag_height\0" as *const u8 as *const i8,
        offset: 252 as u64 as i32,
        is_int: 0 as i32,
    },
    Anon_anon_766 {
        name: b"passivewall_vel_x\0" as *const u8 as *const i8,
        offset: 256 as u64 as i32,
        is_int: 0 as i32,
    },
    Anon_anon_766 {
        name: b"wall_jump_horizontal_velocity\0" as *const u8
            as *const i8,
        offset: 260 as u64 as i32,
        is_int: 0 as i32,
    },
    Anon_anon_766 {
        name: b"wall_jump_vertical_velocity\0" as *const u8
            as *const i8,
        offset: 264 as u64 as i32,
        is_int: 0 as i32,
    },
    Anon_anon_766 {
        name: b"passiveceil_vel_x\0" as *const u8 as *const i8,
        offset: 268 as u64 as i32,
        is_int: 0 as i32,
    },
    Anon_anon_766 {
        name: b"trophy_scale\0" as *const u8 as *const i8,
        offset: 272 as u64 as i32,
        is_int: 0 as i32,
    },
    Anon_anon_766 {
        name: b"screw_attack_launch_velocity\0" as *const u8
            as *const i8,
        offset: 320 as u64 as i32,
        is_int: 0 as i32,
    },
    Anon_anon_766 {
        name: b"wall_jump_min_approach_speed\0" as *const u8
            as *const i8,
        offset: 328 as u64 as i32,
        is_int: 0 as i32,
    },
    Anon_anon_766 {
        name: b"damageice_ice_size\0" as *const u8 as *const i8,
        offset: 332 as u64 as i32,
        is_int: 0 as i32,
    },
    Anon_anon_766 {
        name: b"damageicejump_vel_y\0" as *const u8 as *const i8,
        offset: 344 as u64 as i32,
        is_int: 0 as i32,
    },
    Anon_anon_766 {
        name: b"damageicejump_vel_x_mult\0" as *const u8 as *const i8,
        offset: 348 as u64 as i32,
        is_int: 0 as i32,
    },
    Anon_anon_766 {
        name: b"respawn_platform_scale\0" as *const u8 as *const i8,
        offset: 352 as u64 as i32,
        is_int: 0 as i32,
    },
    Anon_anon_766 {
        name: b"warp_star_hitbox_scale\0" as *const u8 as *const i8,
        offset: 356 as u64 as i32,
        is_int: 0 as i32,
    },
    Anon_anon_766 {
        name: b"camera_zoom_target_bone\0" as *const u8 as *const i8,
        offset: 364 as u64 as i32,
        is_int: 1 as i32,
    },
    Anon_anon_766 {
        name: b"unused_0\0" as *const u8 as *const i8,
        offset: 164 as u64 as i32,
        is_int: 1 as i32,
    },
    Anon_anon_766 {
        name: b"xDC\0" as *const u8 as *const i8,
        offset: 220 as u64 as i32,
        is_int: 0 as i32,
    },
    Anon_anon_766 {
        name: b"x12C\0" as *const u8 as *const i8,
        offset: 300 as u64 as i32,
        is_int: 0 as i32,
    },
    Anon_anon_766 {
        name: b"x13C\0" as *const u8 as *const i8,
        offset: 316 as u64 as i32,
        is_int: 0 as i32,
    },
    Anon_anon_766 {
        name: b"x144\0" as *const u8 as *const i8,
        offset: 324 as u64 as i32,
        is_int: 0 as i32,
    },
    Anon_anon_766 {
        name: b"x150_damageice_unk\0" as *const u8 as *const i8,
        offset: 336 as u64 as i32,
        is_int: 0 as i32,
    },
    Anon_anon_766 {
        name: b"x154_damageice_unk\0" as *const u8 as *const i8,
        offset: 340 as u64 as i32,
        is_int: 0 as i32,
    },
    Anon_anon_766 {
        name: b"x168\0" as *const u8 as *const i8,
        offset: 360 as u64 as i32,
        is_int: 0 as i32,
    },
    Anon_anon_766 {
        name: b"x17C\0" as *const u8 as *const i8,
        offset: 380 as u64 as i32,
        is_int: 0 as i32,
    },
];

pub static mut geno_test_capture: i32 = unsafe { core::mem::zeroed() };

pub static mut geno_test_anim_end: i32 = unsafe { core::mem::zeroed() };

pub static mut geno_test_target: u32 = unsafe { core::mem::zeroed() };

pub static mut geno_test_changes: i32 = unsafe { core::mem::zeroed() };

pub static mut Geno_Rows: *mut *mut MotionState = unsafe { core::mem::zeroed() };

pub static mut Geno_Params: *mut *mut f32 = unsafe { core::mem::zeroed() };

pub static mut Geno_RowCount: *mut i32 = unsafe { core::mem::zeroed() };

pub static mut Geno_ProfileCapacity: i32 = unsafe { core::mem::zeroed() };

pub static mut Geno_ProfileStorage: Anon_anon_16 = unsafe { core::mem::zeroed() };

pub static mut Geno_ProfileStorageUsed: u32 = unsafe { core::mem::zeroed() };

pub static mut geno_bhv_params: [GenoBhvParamDef; 90] = [
    GenoBhvParamDef {
        family: b"GLIDE\0" as *const u8 as *const i8,
        name: b"max_glides\0" as *const u8 as *const i8,
        id: GENO_BP_GLIDE_max_glides as i32,
        def: 2 as i32 as f32,
    },
    GenoBhvParamDef {
        family: b"GLIDE\0" as *const u8 as *const i8,
        name: b"entry_hold\0" as *const u8 as *const i8,
        id: GENO_BP_GLIDE_entry_hold as i32,
        def: 6 as i32 as f32,
    },
    GenoBhvParamDef {
        family: b"GLIDE\0" as *const u8 as *const i8,
        name: b"flick_window\0" as *const u8 as *const i8,
        id: GENO_BP_GLIDE_flick_window as i32,
        def: 12 as i32 as f32,
    },
    GenoBhvParamDef {
        family: b"GLIDE\0" as *const u8 as *const i8,
        name: b"start_frames\0" as *const u8 as *const i8,
        id: GENO_BP_GLIDE_start_frames as i32,
        def: 12 as i32 as f32,
    },
    GenoBhvParamDef {
        family: b"GLIDE\0" as *const u8 as *const i8,
        name: b"trip_frames\0" as *const u8 as *const i8,
        id: GENO_BP_GLIDE_trip_frames as i32,
        def: 30 as i32 as f32,
    },
    GenoBhvParamDef {
        family: b"GLIDE\0" as *const u8 as *const i8,
        name: b"land_lag\0" as *const u8 as *const i8,
        id: GENO_BP_GLIDE_land_lag as i32,
        def: 14 as i32 as f32,
    },
    GenoBhvParamDef {
        family: b"GLIDE\0" as *const u8 as *const i8,
        name: b"jumpout_frames\0" as *const u8 as *const i8,
        id: GENO_BP_GLIDE_jumpout_frames as i32,
        def: 8 as i32 as f32,
    },
    GenoBhvParamDef {
        family: b"GLIDE\0" as *const u8 as *const i8,
        name: b"attack_frames\0" as *const u8 as *const i8,
        id: GENO_BP_GLIDE_attack_frames as i32,
        def: 28 as i32 as f32,
    },
    GenoBhvParamDef {
        family: b"GLIDE\0" as *const u8 as *const i8,
        name: b"attack_hit0\0" as *const u8 as *const i8,
        id: GENO_BP_GLIDE_attack_hit0 as i32,
        def: 8 as i32 as f32,
    },
    GenoBhvParamDef {
        family: b"GLIDE\0" as *const u8 as *const i8,
        name: b"attack_hit1\0" as *const u8 as *const i8,
        id: GENO_BP_GLIDE_attack_hit1 as i32,
        def: 14 as i32 as f32,
    },
    GenoBhvParamDef {
        family: b"GLIDE\0" as *const u8 as *const i8,
        name: b"attack_endlag\0" as *const u8 as *const i8,
        id: GENO_BP_GLIDE_attack_endlag as i32,
        def: 14 as i32 as f32,
    },
    GenoBhvParamDef {
        family: b"GLIDE\0" as *const u8 as *const i8,
        name: b"attack_buf_frames\0" as *const u8 as *const i8,
        id: GENO_BP_GLIDE_attack_buf_frames as i32,
        def: 6 as i32 as f32,
    },
    GenoBhvParamDef {
        family: b"GLIDE\0" as *const u8 as *const i8,
        name: b"entry_vy_max\0" as *const u8 as *const i8,
        id: GENO_BP_GLIDE_entry_vy_max as i32,
        def: 0.6f32,
    },
    GenoBhvParamDef {
        family: b"GLIDE\0" as *const u8 as *const i8,
        name: b"apex_vy\0" as *const u8 as *const i8,
        id: GENO_BP_GLIDE_apex_vy as i32,
        def: 0.25f32,
    },
    GenoBhvParamDef {
        family: b"GLIDE\0" as *const u8 as *const i8,
        name: b"flick_thr\0" as *const u8 as *const i8,
        id: GENO_BP_GLIDE_flick_thr as i32,
        def: 0.55f32,
    },
    GenoBhvParamDef {
        family: b"GLIDE\0" as *const u8 as *const i8,
        name: b"s_start\0" as *const u8 as *const i8,
        id: GENO_BP_GLIDE_s_start as i32,
        def: 1.50f32,
    },
    GenoBhvParamDef {
        family: b"GLIDE\0" as *const u8 as *const i8,
        name: b"start_accel\0" as *const u8 as *const i8,
        id: GENO_BP_GLIDE_start_accel as i32,
        def: 0.12f32,
    },
    GenoBhvParamDef {
        family: b"GLIDE\0" as *const u8 as *const i8,
        name: b"s_start_cap\0" as *const u8 as *const i8,
        id: GENO_BP_GLIDE_s_start_cap as i32,
        def: 2.0f32,
    },
    GenoBhvParamDef {
        family: b"GLIDE\0" as *const u8 as *const i8,
        name: b"theta_def\0" as *const u8 as *const i8,
        id: GENO_BP_GLIDE_theta_def as i32,
        def: -5.0f32,
    },
    GenoBhvParamDef {
        family: b"GLIDE\0" as *const u8 as *const i8,
        name: b"r_down\0" as *const u8 as *const i8,
        id: GENO_BP_GLIDE_r_down as i32,
        def: 60.0f32,
    },
    GenoBhvParamDef {
        family: b"GLIDE\0" as *const u8 as *const i8,
        name: b"r_up\0" as *const u8 as *const i8,
        id: GENO_BP_GLIDE_r_up as i32,
        def: 85.0f32,
    },
    GenoBhvParamDef {
        family: b"GLIDE\0" as *const u8 as *const i8,
        name: b"turn_rate\0" as *const u8 as *const i8,
        id: GENO_BP_GLIDE_turn_rate as i32,
        def: 3.0f32,
    },
    GenoBhvParamDef {
        family: b"GLIDE\0" as *const u8 as *const i8,
        name: b"gk\0" as *const u8 as *const i8,
        id: GENO_BP_GLIDE_gk as i32,
        def: 0.0366f32,
    },
    GenoBhvParamDef {
        family: b"GLIDE\0" as *const u8 as *const i8,
        name: b"drag\0" as *const u8 as *const i8,
        id: GENO_BP_GLIDE_drag as i32,
        def: 0.002f32,
    },
    GenoBhvParamDef {
        family: b"GLIDE\0" as *const u8 as *const i8,
        name: b"s_max\0" as *const u8 as *const i8,
        id: GENO_BP_GLIDE_s_max as i32,
        def: 2.40f32,
    },
    GenoBhvParamDef {
        family: b"GLIDE\0" as *const u8 as *const i8,
        name: b"s_stall\0" as *const u8 as *const i8,
        id: GENO_BP_GLIDE_s_stall as i32,
        def: 0.35f32,
    },
    GenoBhvParamDef {
        family: b"GLIDE\0" as *const u8 as *const i8,
        name: b"trip_angle\0" as *const u8 as *const i8,
        id: GENO_BP_GLIDE_trip_angle as i32,
        def: 30.0f32,
    },
    GenoBhvParamDef {
        family: b"GLIDE\0" as *const u8 as *const i8,
        name: b"attack_speed_keep\0" as *const u8 as *const i8,
        id: GENO_BP_GLIDE_attack_speed_keep as i32,
        def: 0.6f32,
    },
    GenoBhvParamDef {
        family: b"GLIDE\0" as *const u8 as *const i8,
        name: b"chain_speed_mul\0" as *const u8 as *const i8,
        id: GENO_BP_GLIDE_chain_speed_mul as i32,
        def: 0.75f32,
    },
    GenoBhvParamDef {
        family: b"GLIDE\0" as *const u8 as *const i8,
        name: b"chain_turn_mul\0" as *const u8 as *const i8,
        id: GENO_BP_GLIDE_chain_turn_mul as i32,
        def: 1.4f32,
    },
    GenoBhvParamDef {
        family: b"GLIDE\0" as *const u8 as *const i8,
        name: b"attack_damage\0" as *const u8 as *const i8,
        id: GENO_BP_GLIDE_attack_damage as i32,
        def: 12 as i32 as f32,
    },
    GenoBhvParamDef {
        family: b"GLIDE\0" as *const u8 as *const i8,
        name: b"auto_entry\0" as *const u8 as *const i8,
        id: GENO_BP_GLIDE_auto_entry as i32,
        def: 1.0f32,
    },
    GenoBhvParamDef {
        family: b"GLIDE\0" as *const u8 as *const i8,
        name: b"pose_center\0" as *const u8 as *const i8,
        id: GENO_BP_GLIDE_pose_center as i32,
        def: 0.0f32,
    },
    GenoBhvParamDef {
        family: b"TORNADO\0" as *const u8 as *const i8,
        name: b"start_frames\0" as *const u8 as *const i8,
        id: GENO_BP_TORNADO_start_frames as i32,
        def: 12 as i32 as f32,
    },
    GenoBhvParamDef {
        family: b"TORNADO\0" as *const u8 as *const i8,
        name: b"read_period\0" as *const u8 as *const i8,
        id: GENO_BP_TORNADO_read_period as i32,
        def: 10 as i32 as f32,
    },
    GenoBhvParamDef {
        family: b"TORNADO\0" as *const u8 as *const i8,
        name: b"read_limit\0" as *const u8 as *const i8,
        id: GENO_BP_TORNADO_read_limit as i32,
        def: 70 as i32 as f32,
    },
    GenoBhvParamDef {
        family: b"TORNADO\0" as *const u8 as *const i8,
        name: b"end_frames\0" as *const u8 as *const i8,
        id: GENO_BP_TORNADO_end_frames as i32,
        def: 29 as i32 as f32,
    },
    GenoBhvParamDef {
        family: b"TORNADO\0" as *const u8 as *const i8,
        name: b"ledge_lock\0" as *const u8 as *const i8,
        id: GENO_BP_TORNADO_ledge_lock as i32,
        def: 6 as i32 as f32,
    },
    GenoBhvParamDef {
        family: b"TORNADO\0" as *const u8 as *const i8,
        name: b"final_hit_ground\0" as *const u8 as *const i8,
        id: GENO_BP_TORNADO_final_hit_ground as i32,
        def: 3 as i32 as f32,
    },
    GenoBhvParamDef {
        family: b"TORNADO\0" as *const u8 as *const i8,
        name: b"final_hit_air\0" as *const u8 as *const i8,
        id: GENO_BP_TORNADO_final_hit_air as i32,
        def: 2 as i32 as f32,
    },
    GenoBhvParamDef {
        family: b"TORNADO\0" as *const u8 as *const i8,
        name: b"loop_damage\0" as *const u8 as *const i8,
        id: GENO_BP_TORNADO_loop_damage as i32,
        def: 1 as i32 as f32,
    },
    GenoBhvParamDef {
        family: b"TORNADO\0" as *const u8 as *const i8,
        name: b"final_damage\0" as *const u8 as *const i8,
        id: GENO_BP_TORNADO_final_damage as i32,
        def: 3 as i32 as f32,
    },
    GenoBhvParamDef {
        family: b"TORNADO\0" as *const u8 as *const i8,
        name: b"spin0\0" as *const u8 as *const i8,
        id: GENO_BP_TORNADO_spin0 as i32,
        def: 80.0f32,
    },
    GenoBhvParamDef {
        family: b"TORNADO\0" as *const u8 as *const i8,
        name: b"decay_a\0" as *const u8 as *const i8,
        id: GENO_BP_TORNADO_decay_a as i32,
        def: 1.5f32,
    },
    GenoBhvParamDef {
        family: b"TORNADO\0" as *const u8 as *const i8,
        name: b"decay_b\0" as *const u8 as *const i8,
        id: GENO_BP_TORNADO_decay_b as i32,
        def: 2.0f32,
    },
    GenoBhvParamDef {
        family: b"TORNADO\0" as *const u8 as *const i8,
        name: b"mash_add\0" as *const u8 as *const i8,
        id: GENO_BP_TORNADO_mash_add as i32,
        def: 16.0f32,
    },
    GenoBhvParamDef {
        family: b"TORNADO\0" as *const u8 as *const i8,
        name: b"spin_max\0" as *const u8 as *const i8,
        id: GENO_BP_TORNADO_spin_max as i32,
        def: 80.0f32,
    },
    GenoBhvParamDef {
        family: b"TORNADO\0" as *const u8 as *const i8,
        name: b"spin_end\0" as *const u8 as *const i8,
        id: GENO_BP_TORNADO_spin_end as i32,
        def: 10.0f32,
    },
    GenoBhvParamDef {
        family: b"TORNADO\0" as *const u8 as *const i8,
        name: b"grav_tn\0" as *const u8 as *const i8,
        id: GENO_BP_TORNADO_grav as i32,
        def: 0.0239f32,
    },
    GenoBhvParamDef {
        family: b"TORNADO\0" as *const u8 as *const i8,
        name: b"rise_impulse\0" as *const u8 as *const i8,
        id: GENO_BP_TORNADO_rise_impulse as i32,
        def: 0.9f32,
    },
    GenoBhvParamDef {
        family: b"TORNADO\0" as *const u8 as *const i8,
        name: b"rise_max\0" as *const u8 as *const i8,
        id: GENO_BP_TORNADO_rise_max as i32,
        def: 1.6f32,
    },
    GenoBhvParamDef {
        family: b"TORNADO\0" as *const u8 as *const i8,
        name: b"fall_tn\0" as *const u8 as *const i8,
        id: GENO_BP_TORNADO_fall_max as i32,
        def: 0.50f32,
    },
    GenoBhvParamDef {
        family: b"TORNADO\0" as *const u8 as *const i8,
        name: b"vx_max\0" as *const u8 as *const i8,
        id: GENO_BP_TORNADO_vx_max as i32,
        def: 1.1f32,
    },
    GenoBhvParamDef {
        family: b"TORNADO\0" as *const u8 as *const i8,
        name: b"vx_max_ground\0" as *const u8 as *const i8,
        id: GENO_BP_TORNADO_vx_max_ground as i32,
        def: 1.3f32,
    },
    GenoBhvParamDef {
        family: b"TORNADO\0" as *const u8 as *const i8,
        name: b"vx_accel\0" as *const u8 as *const i8,
        id: GENO_BP_TORNADO_vx_accel as i32,
        def: 0.06f32,
    },
    GenoBhvParamDef {
        family: b"TORNADO\0" as *const u8 as *const i8,
        name: b"jump_bonus\0" as *const u8 as *const i8,
        id: GENO_BP_TORNADO_jump_bonus as i32,
        def: 1.0f32,
    },
    GenoBhvParamDef {
        family: b"TORNADO\0" as *const u8 as *const i8,
        name: b"start_vy_keep\0" as *const u8 as *const i8,
        id: GENO_BP_TORNADO_start_vy_keep as i32,
        def: 0.2f32,
    },
    GenoBhvParamDef {
        family: b"TORNADO\0" as *const u8 as *const i8,
        name: b"start_vx_keep\0" as *const u8 as *const i8,
        id: GENO_BP_TORNADO_start_vx_keep as i32,
        def: 0.5f32,
    },
    GenoBhvParamDef {
        family: b"DRILL\0" as *const u8 as *const i8,
        name: b"move0\0" as *const u8 as *const i8,
        id: GENO_BP_DRILL_move0 as i32,
        def: 16 as i32 as f32,
    },
    GenoBhvParamDef {
        family: b"DRILL\0" as *const u8 as *const i8,
        name: b"ramp\0" as *const u8 as *const i8,
        id: GENO_BP_DRILL_ramp as i32,
        def: 4 as i32 as f32,
    },
    GenoBhvParamDef {
        family: b"DRILL\0" as *const u8 as *const i8,
        name: b"hit0\0" as *const u8 as *const i8,
        id: GENO_BP_DRILL_hit0 as i32,
        def: 26 as i32 as f32,
    },
    GenoBhvParamDef {
        family: b"DRILL\0" as *const u8 as *const i8,
        name: b"hit_period\0" as *const u8 as *const i8,
        id: GENO_BP_DRILL_hit_period as i32,
        def: 4 as i32 as f32,
    },
    GenoBhvParamDef {
        family: b"DRILL\0" as *const u8 as *const i8,
        name: b"nhits\0" as *const u8 as *const i8,
        id: GENO_BP_DRILL_nhits as i32,
        def: 10 as i32 as f32,
    },
    GenoBhvParamDef {
        family: b"DRILL\0" as *const u8 as *const i8,
        name: b"hit_end\0" as *const u8 as *const i8,
        id: GENO_BP_DRILL_hit_end as i32,
        def: 65 as i32 as f32,
    },
    GenoBhvParamDef {
        family: b"DRILL\0" as *const u8 as *const i8,
        name: b"final_frame\0" as *const u8 as *const i8,
        id: GENO_BP_DRILL_final_frame as i32,
        def: 66 as i32 as f32,
    },
    GenoBhvParamDef {
        family: b"DRILL\0" as *const u8 as *const i8,
        name: b"final_end_ground\0" as *const u8 as *const i8,
        id: GENO_BP_DRILL_final_end_ground as i32,
        def: 67 as i32 as f32,
    },
    GenoBhvParamDef {
        family: b"DRILL\0" as *const u8 as *const i8,
        name: b"final_end_air\0" as *const u8 as *const i8,
        id: GENO_BP_DRILL_final_end_air as i32,
        def: 70 as i32 as f32,
    },
    GenoBhvParamDef {
        family: b"DRILL\0" as *const u8 as *const i8,
        name: b"total\0" as *const u8 as *const i8,
        id: GENO_BP_DRILL_total as i32,
        def: 96 as i32 as f32,
    },
    GenoBhvParamDef {
        family: b"DRILL\0" as *const u8 as *const i8,
        name: b"rush_damage\0" as *const u8 as *const i8,
        id: GENO_BP_DRILL_rush_damage as i32,
        def: 1 as i32 as f32,
    },
    GenoBhvParamDef {
        family: b"DRILL\0" as *const u8 as *const i8,
        name: b"final_damage\0" as *const u8 as *const i8,
        id: GENO_BP_DRILL_final_damage as i32,
        def: 3 as i32 as f32,
    },
    GenoBhvParamDef {
        family: b"DRILL\0" as *const u8 as *const i8,
        name: b"dive_default\0" as *const u8 as *const i8,
        id: GENO_BP_DRILL_dive_default as i32,
        def: -20.0f32,
    },
    GenoBhvParamDef {
        family: b"DRILL\0" as *const u8 as *const i8,
        name: b"psi_stick\0" as *const u8 as *const i8,
        id: GENO_BP_DRILL_psi_stick as i32,
        def: 45.0f32,
    },
    GenoBhvParamDef {
        family: b"DRILL\0" as *const u8 as *const i8,
        name: b"psi_min\0" as *const u8 as *const i8,
        id: GENO_BP_DRILL_psi_min as i32,
        def: -60.0f32,
    },
    GenoBhvParamDef {
        family: b"DRILL\0" as *const u8 as *const i8,
        name: b"psi_max\0" as *const u8 as *const i8,
        id: GENO_BP_DRILL_psi_max as i32,
        def: 30.0f32,
    },
    GenoBhvParamDef {
        family: b"DRILL\0" as *const u8 as *const i8,
        name: b"steer_rate\0" as *const u8 as *const i8,
        id: GENO_BP_DRILL_steer_rate as i32,
        def: 2.0f32,
    },
    GenoBhvParamDef {
        family: b"DRILL\0" as *const u8 as *const i8,
        name: b"dr_speed\0" as *const u8 as *const i8,
        id: GENO_BP_DRILL_speed as i32,
        def: 2.0f32,
    },
    GenoBhvParamDef {
        family: b"DRILL\0" as *const u8 as *const i8,
        name: b"recoil_vy\0" as *const u8 as *const i8,
        id: GENO_BP_DRILL_recoil_vy as i32,
        def: 1.0f32,
    },
    GenoBhvParamDef {
        family: b"DRILL\0" as *const u8 as *const i8,
        name: b"end_vx_keep\0" as *const u8 as *const i8,
        id: GENO_BP_DRILL_end_vx_keep as i32,
        def: 0.2f32,
    },
    GenoBhvParamDef {
        family: b"CAPE\0" as *const u8 as *const i8,
        name: b"cape_start\0" as *const u8 as *const i8,
        id: GENO_BP_CAPE_start as i32,
        def: 6 as i32 as f32,
    },
    GenoBhvParamDef {
        family: b"CAPE\0" as *const u8 as *const i8,
        name: b"cape_end\0" as *const u8 as *const i8,
        id: GENO_BP_CAPE_end as i32,
        def: 25 as i32 as f32,
    },
    GenoBhvParamDef {
        family: b"CAPE\0" as *const u8 as *const i8,
        name: b"cape_total\0" as *const u8 as *const i8,
        id: GENO_BP_CAPE_total as i32,
        def: 40 as i32 as f32,
    },
    GenoBhvParamDef {
        family: b"CAPE\0" as *const u8 as *const i8,
        name: b"turn_thr\0" as *const u8 as *const i8,
        id: GENO_BP_CAPE_turn_thr as i32,
        def: 0.5f32,
    },
    GenoBhvParamDef {
        family: b"CAPE\0" as *const u8 as *const i8,
        name: b"stall_x\0" as *const u8 as *const i8,
        id: GENO_BP_CAPE_stall_x as i32,
        def: 0.80f32,
    },
    GenoBhvParamDef {
        family: b"CAPE\0" as *const u8 as *const i8,
        name: b"stall_y_up\0" as *const u8 as *const i8,
        id: GENO_BP_CAPE_stall_y_up as i32,
        def: 0.85f32,
    },
    GenoBhvParamDef {
        family: b"CAPE\0" as *const u8 as *const i8,
        name: b"stall_vy_hi\0" as *const u8 as *const i8,
        id: GENO_BP_CAPE_stall_vy_hi as i32,
        def: 0.0f32,
    },
    GenoBhvParamDef {
        family: b"CAPE\0" as *const u8 as *const i8,
        name: b"stall_grav\0" as *const u8 as *const i8,
        id: GENO_BP_CAPE_stall_grav as i32,
        def: 0.015f32,
    },
    GenoBhvParamDef {
        family: b"CAPE\0" as *const u8 as *const i8,
        name: b"stall_fall\0" as *const u8 as *const i8,
        id: GENO_BP_CAPE_stall_fall as i32,
        def: 0.45f32,
    },
    GenoBhvParamDef {
        family: b"CAPE\0" as *const u8 as *const i8,
        name: b"ground_x\0" as *const u8 as *const i8,
        id: GENO_BP_CAPE_ground_x as i32,
        def: 0.5f32,
    },
    GenoBhvParamDef {
        family: b"CAPE\0" as *const u8 as *const i8,
        name: b"reflect_dmg_mul\0" as *const u8 as *const i8,
        id: GENO_BP_CAPE_reflect_dmg_mul as i32,
        def: 1.5f32,
    },
    GenoBhvParamDef {
        family: b"CAPE\0" as *const u8 as *const i8,
        name: b"reflect_speed_mul\0" as *const u8 as *const i8,
        id: GENO_BP_CAPE_reflect_speed_mul as i32,
        def: 1.2f32,
    },
];



pub static mut geno_lua_io: GenoLuaIo = unsafe { core::mem::zeroed() };

pub static mut geno_lua_depth: i32 = unsafe { core::mem::zeroed() };

pub static mut geno_cbs: [GenoCbDef; 45] = unsafe {
    [
        GenoCbDef {
            id: GENO_CBID_NEXT as i32,
            slot: GENO_CB_ANIM as i32,
            name: b"next\0" as *const u8 as *const i8,
            r#fn: Some(
                geno_cb_anim_next as unsafe extern "C" fn(*mut Fighter_GObj) -> (),
            ),
        },
        GenoCbDef {
            id: GENO_CBID_LOOP as i32,
            slot: GENO_CB_ANIM as i32,
            name: b"loop\0" as *const u8 as *const i8,
            r#fn: Some(
                geno_cb_anim_loop as unsafe extern "C" fn(*mut Fighter_GObj) -> (),
            ),
        },
        GenoCbDef {
            id: GENO_CBID_LUA as i32,
            slot: GENO_CB_ANIM as i32,
            name: b"lua\0" as *const u8 as *const i8,
            r#fn: Some(geno_cb_anim_lua as unsafe extern "C" fn(*mut Fighter_GObj) -> ()),
        },
        GenoCbDef {
            id: GENO_CBID_HOLD as i32,
            slot: GENO_CB_ANIM as i32,
            name: b"hold\0" as *const u8 as *const i8,
            r#fn: Some(geno_cb_nothing as unsafe extern "C" fn(*mut Fighter_GObj) -> ()),
        },
        GenoCbDef {
            id: GENO_CBID_GLIDE_START as i32,
            slot: GENO_CB_ANIM as i32,
            name: b"glide.start\0" as *const u8 as *const i8,
            r#fn: Some(
                geno_glide_start_anim as unsafe extern "C" fn(*mut Fighter_GObj) -> (),
            ),
        },
        GenoCbDef {
            id: GENO_CBID_GLIDE as i32,
            slot: GENO_CB_ANIM as i32,
            name: b"glide\0" as *const u8 as *const i8,
            r#fn: Some(geno_glide_anim as unsafe extern "C" fn(*mut Fighter_GObj) -> ()),
        },
        GenoCbDef {
            id: GENO_CBID_TORNADO as i32,
            slot: GENO_CB_ANIM as i32,
            name: b"tornado\0" as *const u8 as *const i8,
            r#fn: Some(
                geno_tornado_anim as unsafe extern "C" fn(*mut Fighter_GObj) -> (),
            ),
        },
        GenoCbDef {
            id: GENO_CBID_DRILL as i32,
            slot: GENO_CB_ANIM as i32,
            name: b"drill\0" as *const u8 as *const i8,
            r#fn: Some(geno_drill_anim as unsafe extern "C" fn(*mut Fighter_GObj) -> ()),
        },
        GenoCbDef {
            id: GENO_CBID_DRILL_END as i32,
            slot: GENO_CB_ANIM as i32,
            name: b"drill.end\0" as *const u8 as *const i8,
            r#fn: Some(
                geno_drill_end_anim as unsafe extern "C" fn(*mut Fighter_GObj) -> (),
            ),
        },
        GenoCbDef {
            id: GENO_CBID_GLIDE_AFTER as i32,
            slot: GENO_CB_ANIM as i32,
            name: b"glide.after\0" as *const u8 as *const i8,
            r#fn: Some(
                geno_glide_after_anim as unsafe extern "C" fn(*mut Fighter_GObj) -> (),
            ),
        },
        GenoCbDef {
            id: GENO_CBID_CAPE as i32,
            slot: GENO_CB_ANIM as i32,
            name: b"cape\0" as *const u8 as *const i8,
            r#fn: Some(geno_cape_anim as unsafe extern "C" fn(*mut Fighter_GObj) -> ()),
        },
        GenoCbDef {
            id: GENO_CBID_MULTI_DASH as i32,
            slot: GENO_CB_ANIM as i32,
            name: b"multi_dash\0" as *const u8 as *const i8,
            r#fn: Some(
                geno_multi_dash_anim as unsafe extern "C" fn(*mut Fighter_GObj) -> (),
            ),
        },
        GenoCbDef {
            id: GENO_CBID_MULTI_DASH as i32,
            slot: GENO_CB_PHYS as i32,
            name: b"multi_dash\0" as *const u8 as *const i8,
            r#fn: Some(
                geno_multi_dash_phys as unsafe extern "C" fn(*mut Fighter_GObj) -> (),
            ),
        },
        GenoCbDef {
            id: GENO_CBID_MULTI_DASH as i32,
            slot: GENO_CB_COLL as i32,
            name: b"multi_dash\0" as *const u8 as *const i8,
            r#fn: Some(
                geno_multi_dash_coll as unsafe extern "C" fn(*mut Fighter_GObj) -> (),
            ),
        },
        GenoCbDef {
            id: GENO_CBID_INTERRUPT as i32,
            slot: GENO_CB_IASA as i32,
            name: b"interrupt\0" as *const u8 as *const i8,
            r#fn: Some(
                geno_cb_iasa_interrupt as unsafe extern "C" fn(*mut Fighter_GObj) -> (),
            ),
        },
        GenoCbDef {
            id: GENO_CBID_CAPE as i32,
            slot: GENO_CB_PHYS as i32,
            name: b"cape\0" as *const u8 as *const i8,
            r#fn: Some(geno_cape_phys as unsafe extern "C" fn(*mut Fighter_GObj) -> ()),
        },
        GenoCbDef {
            id: GENO_CBID_CAPE as i32,
            slot: GENO_CB_COLL as i32,
            name: b"cape\0" as *const u8 as *const i8,
            r#fn: Some(geno_cape_coll as unsafe extern "C" fn(*mut Fighter_GObj) -> ()),
        },
        GenoCbDef {
            id: GENO_CBID_CAPE_AFTER as i32,
            slot: GENO_CB_COLL as i32,
            name: b"cape.after\0" as *const u8 as *const i8,
            r#fn: Some(
                geno_cape_after_coll as unsafe extern "C" fn(*mut Fighter_GObj) -> (),
            ),
        },
        GenoCbDef {
            id: GENO_CBID_NONE as i32,
            slot: GENO_CB_IASA as i32,
            name: b"none\0" as *const u8 as *const i8,
            r#fn: Some(geno_cb_nothing as unsafe extern "C" fn(*mut Fighter_GObj) -> ()),
        },
        GenoCbDef {
            id: GENO_CBID_GLIDE as i32,
            slot: GENO_CB_IASA as i32,
            name: b"glide\0" as *const u8 as *const i8,
            r#fn: Some(geno_glide_iasa as unsafe extern "C" fn(*mut Fighter_GObj) -> ()),
        },
        GenoCbDef {
            id: GENO_CBID_NONE as i32,
            slot: GENO_CB_PHYS as i32,
            name: b"none\0" as *const u8 as *const i8,
            r#fn: Some(geno_cb_nothing as unsafe extern "C" fn(*mut Fighter_GObj) -> ()),
        },
        GenoCbDef {
            id: GENO_CBID_AIR as i32,
            slot: GENO_CB_PHYS as i32,
            name: b"air\0" as *const u8 as *const i8,
            r#fn: Some(geno_cb_phys_air as unsafe extern "C" fn(*mut Fighter_GObj) -> ()),
        },
        GenoCbDef {
            id: GENO_CBID_AIR_NODRIFT as i32,
            slot: GENO_CB_PHYS as i32,
            name: b"air_nodrift\0" as *const u8 as *const i8,
            r#fn: Some(
                geno_cb_phys_air_nodrift as unsafe extern "C" fn(*mut Fighter_GObj) -> (),
            ),
        },
        GenoCbDef {
            id: GENO_CBID_AIR_DRIFT as i32,
            slot: GENO_CB_PHYS as i32,
            name: b"air_drift\0" as *const u8 as *const i8,
            r#fn: Some(
                geno_cb_phys_air_drift as unsafe extern "C" fn(*mut Fighter_GObj) -> (),
            ),
        },
        GenoCbDef {
            id: GENO_CBID_GROUND as i32,
            slot: GENO_CB_PHYS as i32,
            name: b"ground\0" as *const u8 as *const i8,
            r#fn: Some(
                geno_cb_phys_ground as unsafe extern "C" fn(*mut Fighter_GObj) -> (),
            ),
        },
        GenoCbDef {
            id: GENO_CBID_AUTO as i32,
            slot: GENO_CB_PHYS as i32,
            name: b"auto\0" as *const u8 as *const i8,
            r#fn: Some(
                geno_cb_phys_auto as unsafe extern "C" fn(*mut Fighter_GObj) -> (),
            ),
        },
        GenoCbDef {
            id: GENO_CBID_ANIM_MOTION as i32,
            slot: GENO_CB_PHYS as i32,
            name: b"anim_motion\0" as *const u8 as *const i8,
            r#fn: Some(
                geno_cb_phys_anim_motion as unsafe extern "C" fn(*mut Fighter_GObj) -> (),
            ),
        },
        GenoCbDef {
            id: GENO_CBID_GLIDE_START as i32,
            slot: GENO_CB_PHYS as i32,
            name: b"glide.start\0" as *const u8 as *const i8,
            r#fn: Some(
                geno_glide_start_phys as unsafe extern "C" fn(*mut Fighter_GObj) -> (),
            ),
        },
        GenoCbDef {
            id: GENO_CBID_GLIDE as i32,
            slot: GENO_CB_PHYS as i32,
            name: b"glide\0" as *const u8 as *const i8,
            r#fn: Some(geno_glide_phys as unsafe extern "C" fn(*mut Fighter_GObj) -> ()),
        },
        GenoCbDef {
            id: GENO_CBID_GLIDE_ATTACK as i32,
            slot: GENO_CB_PHYS as i32,
            name: b"glide.attack\0" as *const u8 as *const i8,
            r#fn: Some(
                geno_glide_attack_phys as unsafe extern "C" fn(*mut Fighter_GObj) -> (),
            ),
        },
        GenoCbDef {
            id: GENO_CBID_GLIDE_END as i32,
            slot: GENO_CB_PHYS as i32,
            name: b"glide.end\0" as *const u8 as *const i8,
            r#fn: Some(
                geno_glide_end_phys as unsafe extern "C" fn(*mut Fighter_GObj) -> (),
            ),
        },
        GenoCbDef {
            id: GENO_CBID_TORNADO as i32,
            slot: GENO_CB_PHYS as i32,
            name: b"tornado\0" as *const u8 as *const i8,
            r#fn: Some(
                geno_tornado_phys as unsafe extern "C" fn(*mut Fighter_GObj) -> (),
            ),
        },
        GenoCbDef {
            id: GENO_CBID_DRILL as i32,
            slot: GENO_CB_PHYS as i32,
            name: b"drill\0" as *const u8 as *const i8,
            r#fn: Some(geno_drill_phys as unsafe extern "C" fn(*mut Fighter_GObj) -> ()),
        },
        GenoCbDef {
            id: GENO_CBID_DRILL_END as i32,
            slot: GENO_CB_PHYS as i32,
            name: b"drill.end\0" as *const u8 as *const i8,
            r#fn: Some(
                geno_drill_end_phys as unsafe extern "C" fn(*mut Fighter_GObj) -> (),
            ),
        },
        GenoCbDef {
            id: GENO_CBID_DRILL_START as i32,
            slot: GENO_CB_PHYS as i32,
            name: b"drill.start\0" as *const u8 as *const i8,
            r#fn: Some(
                geno_drill_start_phys as unsafe extern "C" fn(*mut Fighter_GObj) -> (),
            ),
        },
        GenoCbDef {
            id: GENO_CBID_NONE as i32,
            slot: GENO_CB_COLL as i32,
            name: b"none\0" as *const u8 as *const i8,
            r#fn: Some(geno_cb_nothing as unsafe extern "C" fn(*mut Fighter_GObj) -> ()),
        },
        GenoCbDef {
            id: GENO_CBID_AIR as i32,
            slot: GENO_CB_COLL as i32,
            name: b"air\0" as *const u8 as *const i8,
            r#fn: Some(geno_cb_coll_air as unsafe extern "C" fn(*mut Fighter_GObj) -> ()),
        },
        GenoCbDef {
            id: GENO_CBID_AIR_NOLEDGE as i32,
            slot: GENO_CB_COLL as i32,
            name: b"air_noledge\0" as *const u8 as *const i8,
            r#fn: Some(
                geno_cb_coll_air_noledge as unsafe extern "C" fn(*mut Fighter_GObj) -> (),
            ),
        },
        GenoCbDef {
            id: GENO_CBID_GROUND as i32,
            slot: GENO_CB_COLL as i32,
            name: b"ground\0" as *const u8 as *const i8,
            r#fn: Some(
                geno_cb_coll_ground as unsafe extern "C" fn(*mut Fighter_GObj) -> (),
            ),
        },
        GenoCbDef {
            id: GENO_CBID_GROUND_STOP as i32,
            slot: GENO_CB_COLL as i32,
            name: b"ground_stop\0" as *const u8 as *const i8,
            r#fn: Some(
                geno_cb_coll_ground_stop as unsafe extern "C" fn(*mut Fighter_GObj) -> (),
            ),
        },
        GenoCbDef {
            id: GENO_CBID_BOTH as i32,
            slot: GENO_CB_COLL as i32,
            name: b"both\0" as *const u8 as *const i8,
            r#fn: Some(geno_coll_both as unsafe extern "C" fn(*mut Fighter_GObj) -> ()),
        },
        GenoCbDef {
            id: GENO_CBID_ANIM_MOTION as i32,
            slot: GENO_CB_COLL as i32,
            name: b"anim_motion\0" as *const u8 as *const i8,
            r#fn: Some(
                geno_cb_coll_anim_motion as unsafe extern "C" fn(*mut Fighter_GObj) -> (),
            ),
        },
        GenoCbDef {
            id: GENO_CBID_GLIDE as i32,
            slot: GENO_CB_COLL as i32,
            name: b"glide\0" as *const u8 as *const i8,
            r#fn: Some(geno_glide_coll as unsafe extern "C" fn(*mut Fighter_GObj) -> ()),
        },
        GenoCbDef {
            id: GENO_CBID_DRILL as i32,
            slot: GENO_CB_COLL as i32,
            name: b"drill\0" as *const u8 as *const i8,
            r#fn: Some(geno_drill_coll as unsafe extern "C" fn(*mut Fighter_GObj) -> ()),
        },
        GenoCbDef {
            id: GENO_CBID_DRILL_START as i32,
            slot: GENO_CB_COLL as i32,
            name: b"drill.start\0" as *const u8 as *const i8,
            r#fn: Some(
                geno_drill_start_coll as unsafe extern "C" fn(*mut Fighter_GObj) -> (),
            ),
        },
    ]
};

pub static mut geno_bhvs: [GenoBhvDef; 17] = unsafe {
    [
        GenoBhvDef {
            id: GENO_BHV_NONE as i32,
            name: b"\0" as *const u8 as *const i8,
            cb: [
                GENO_CBID_NEXT as i32 as u8,
                GENO_CBID_NONE as i32 as u8,
                GENO_CBID_AIR as i32 as u8,
                GENO_CBID_AIR as i32 as u8,
            ],
            enter: None,
            next: GENO_AUTO,
            land: GENO_AUTO,
            like: ftCo_MS_Fall as i32,
        },
        GenoBhvDef {
            id: GENO_BHV_AIR as i32,
            name: b"geno.air\0" as *const u8 as *const i8,
            cb: [
                GENO_CBID_NEXT as i32 as u8,
                GENO_CBID_NONE as i32 as u8,
                GENO_CBID_AIR as i32 as u8,
                GENO_CBID_AIR as i32 as u8,
            ],
            enter: None,
            next: GENO_AUTO,
            land: GENO_AUTO,
            like: ftCo_MS_Fall as i32,
        },
        GenoBhvDef {
            id: GENO_BHV_GROUND as i32,
            name: b"geno.ground\0" as *const u8 as *const i8,
            cb: [
                GENO_CBID_NEXT as i32 as u8,
                GENO_CBID_NONE as i32 as u8,
                GENO_CBID_GROUND as i32 as u8,
                GENO_CBID_GROUND as i32 as u8,
            ],
            enter: None,
            next: GENO_AUTO,
            land: GENO_AUTO,
            like: ftCo_MS_Landing as i32,
        },
        GenoBhvDef {
            id: GENO_BHV_ANIM_MOTION as i32,
            name: b"geno.anim_motion\0" as *const u8 as *const i8,
            cb: [
                GENO_CBID_NEXT as i32 as u8,
                GENO_CBID_NONE as i32 as u8,
                GENO_CBID_ANIM_MOTION as i32 as u8,
                GENO_CBID_ANIM_MOTION as i32 as u8,
            ],
            enter: None,
            next: GENO_AUTO,
            land: GENO_AUTO,
            like: ftCo_MS_Fall as i32,
        },
        GenoBhvDef {
            id: GENO_BHV_GLIDE_START as i32,
            name: b"geno.glide.start\0" as *const u8 as *const i8,
            cb: [
                GENO_CBID_GLIDE_START as i32 as u8,
                GENO_CBID_NONE as i32 as u8,
                GENO_CBID_GLIDE_START as i32 as u8,
                GENO_CBID_AIR as i32 as u8,
            ],
            enter: Some(
                geno_glide_start_enter
                    as unsafe extern "C" fn(
                        *mut Fighter_GObj,
                        *mut Fighter,
                        *mut GenoState,
                        i32,
                    ) -> (),
            ),
            next: (3 as u32 & 15 as u32) << 28 as i32
                | GENO_BHV_GLIDE as i32 as u32 & 0xffff as u32,
            land: (3 as u32 & 15 as u32) << 28 as i32
                | GENO_BHV_GLIDE_LANDING as i32 as u32
                    & 0xffff as u32,
            like: ftCo_MS_Fall as i32,
        },
        GenoBhvDef {
            id: GENO_BHV_GLIDE as i32,
            name: b"geno.glide\0" as *const u8 as *const i8,
            cb: [
                GENO_CBID_GLIDE as i32 as u8,
                GENO_CBID_GLIDE as i32 as u8,
                GENO_CBID_GLIDE as i32 as u8,
                GENO_CBID_GLIDE as i32 as u8,
            ],
            enter: Some(
                geno_glide_enter
                    as unsafe extern "C" fn(
                        *mut Fighter_GObj,
                        *mut Fighter,
                        *mut GenoState,
                        i32,
                    ) -> (),
            ),
            next: (3 as u32 & 15 as u32) << 28 as i32
                | GENO_BHV_GLIDE_END as i32 as u32 & 0xffff as u32,
            land: (3 as u32 & 15 as u32) << 28 as i32
                | GENO_BHV_GLIDE_LANDING as i32 as u32
                    & 0xffff as u32,
            like: ftCo_MS_Fall as i32,
        },
        GenoBhvDef {
            id: GENO_BHV_GLIDE_ATTACK as i32,
            name: b"geno.glide.attack\0" as *const u8 as *const i8,
            cb: [
                GENO_CBID_GLIDE_AFTER as i32 as u8,
                GENO_CBID_NONE as i32 as u8,
                GENO_CBID_GLIDE_ATTACK as i32 as u8,
                GENO_CBID_AIR_NOLEDGE as i32 as u8,
            ],
            enter: Some(
                geno_glide_attack_enter
                    as unsafe extern "C" fn(
                        *mut Fighter_GObj,
                        *mut Fighter,
                        *mut GenoState,
                        i32,
                    ) -> (),
            ),
            next: (0 as u32 & 15 as u32) << 28 as i32
                | ftCo_MS_Fall as i32 as u32 & 0xffff as u32,
            land: GENO_AUTO,
            like: ftCo_MS_AttackAirN as i32,
        },
        GenoBhvDef {
            id: GENO_BHV_GLIDE_LANDING as i32,
            name: b"geno.glide.landing\0" as *const u8 as *const i8,
            cb: [
                GENO_CBID_NEXT as i32 as u8,
                GENO_CBID_NONE as i32 as u8,
                GENO_CBID_GROUND as i32 as u8,
                GENO_CBID_GROUND as i32 as u8,
            ],
            enter: Some(
                geno_glide_landing_enter
                    as unsafe extern "C" fn(
                        *mut Fighter_GObj,
                        *mut Fighter,
                        *mut GenoState,
                        i32,
                    ) -> (),
            ),
            next: GENO_AUTO,
            land: GENO_AUTO,
            like: ftCo_MS_LandingFallSpecial as i32,
        },
        GenoBhvDef {
            id: GENO_BHV_GLIDE_END as i32,
            name: b"geno.glide.end\0" as *const u8 as *const i8,
            cb: [
                GENO_CBID_GLIDE_AFTER as i32 as u8,
                GENO_CBID_NONE as i32 as u8,
                GENO_CBID_GLIDE_END as i32 as u8,
                GENO_CBID_AIR as i32 as u8,
            ],
            enter: Some(
                geno_glide_end_enter
                    as unsafe extern "C" fn(
                        *mut Fighter_GObj,
                        *mut Fighter,
                        *mut GenoState,
                        i32,
                    ) -> (),
            ),
            next: GENO_AUTO,
            land: GENO_AUTO,
            like: ftCo_MS_Fall as i32,
        },
        GenoBhvDef {
            id: GENO_BHV_TORNADO as i32,
            name: b"geno.tornado\0" as *const u8 as *const i8,
            cb: [
                GENO_CBID_TORNADO as i32 as u8,
                GENO_CBID_NONE as i32 as u8,
                GENO_CBID_TORNADO as i32 as u8,
                GENO_CBID_BOTH as i32 as u8,
            ],
            enter: Some(
                geno_tornado_enter
                    as unsafe extern "C" fn(
                        *mut Fighter_GObj,
                        *mut Fighter,
                        *mut GenoState,
                        i32,
                    ) -> (),
            ),
            next: GENO_NONE_T,
            land: GENO_AUTO,
            like: ftCo_MS_Fall as i32,
        },
        GenoBhvDef {
            id: GENO_BHV_DRILL as i32,
            name: b"geno.drill\0" as *const u8 as *const i8,
            cb: [
                GENO_CBID_DRILL as i32 as u8,
                GENO_CBID_NONE as i32 as u8,
                GENO_CBID_DRILL as i32 as u8,
                GENO_CBID_DRILL as i32 as u8,
            ],
            enter: Some(
                geno_drill_enter
                    as unsafe extern "C" fn(
                        *mut Fighter_GObj,
                        *mut Fighter,
                        *mut GenoState,
                        i32,
                    ) -> (),
            ),
            next: (3 as u32 & 15 as u32) << 28 as i32
                | GENO_BHV_DRILL_END as i32 as u32 & 0xffff as u32,
            land: GENO_AUTO,
            like: ftCo_MS_Fall as i32,
        },
        GenoBhvDef {
            id: GENO_BHV_DRILL_END as i32,
            name: b"geno.drill.end\0" as *const u8 as *const i8,
            cb: [
                GENO_CBID_DRILL_END as i32 as u8,
                GENO_CBID_NONE as i32 as u8,
                GENO_CBID_DRILL_END as i32 as u8,
                GENO_CBID_BOTH as i32 as u8,
            ],
            enter: Some(
                geno_drill_end_enter
                    as unsafe extern "C" fn(
                        *mut Fighter_GObj,
                        *mut Fighter,
                        *mut GenoState,
                        i32,
                    ) -> (),
            ),
            next: GENO_NONE_T,
            land: GENO_AUTO,
            like: ftCo_MS_Fall as i32,
        },
        GenoBhvDef {
            id: GENO_BHV_CAPE as i32,
            name: b"geno.cape\0" as *const u8 as *const i8,
            cb: [
                GENO_CBID_CAPE as i32 as u8,
                GENO_CBID_NONE as i32 as u8,
                GENO_CBID_CAPE as i32 as u8,
                GENO_CBID_CAPE as i32 as u8,
            ],
            enter: Some(
                geno_cape_enter
                    as unsafe extern "C" fn(
                        *mut Fighter_GObj,
                        *mut Fighter,
                        *mut GenoState,
                        i32,
                    ) -> (),
            ),
            next: GENO_NONE_T,
            land: GENO_AUTO,
            like: ftCo_MS_Fall as i32,
        },
        GenoBhvDef {
            id: GENO_BHV_CAPE_ATTACK as i32,
            name: b"geno.cape.attack\0" as *const u8 as *const i8,
            cb: [
                GENO_CBID_NEXT as i32 as u8,
                GENO_CBID_NONE as i32 as u8,
                GENO_CBID_ANIM_MOTION as i32 as u8,
                GENO_CBID_CAPE_AFTER as i32 as u8,
            ],
            enter: None,
            next: GENO_TGT_HELPLESS,
            land: GENO_TGT_STAY,
            like: ftCo_MS_Fall as i32,
        },
        GenoBhvDef {
            id: GENO_BHV_CAPE_END as i32,
            name: b"geno.cape.end\0" as *const u8 as *const i8,
            cb: [
                GENO_CBID_NEXT as i32 as u8,
                GENO_CBID_INTERRUPT as i32 as u8,
                GENO_CBID_ANIM_MOTION as i32 as u8,
                GENO_CBID_CAPE_AFTER as i32 as u8,
            ],
            enter: None,
            next: GENO_TGT_HELPLESS,
            land: GENO_TGT_STAY,
            like: ftCo_MS_Fall as i32,
        },
        GenoBhvDef {
            id: GENO_BHV_DRILL_START as i32,
            name: b"geno.drill.start\0" as *const u8 as *const i8,
            cb: [
                GENO_CBID_NEXT as i32 as u8,
                GENO_CBID_NONE as i32 as u8,
                GENO_CBID_DRILL_START as i32 as u8,
                GENO_CBID_DRILL_START as i32 as u8,
            ],
            enter: Some(
                geno_drill_start_enter
                    as unsafe extern "C" fn(
                        *mut Fighter_GObj,
                        *mut Fighter,
                        *mut GenoState,
                        i32,
                    ) -> (),
            ),
            next: (3 as u32 & 15 as u32) << 28 as i32
                | GENO_BHV_DRILL as i32 as u32 & 0xffff as u32,
            land: GENO_AUTO,
            like: ftCo_MS_Fall as i32,
        },
        GenoBhvDef {
            id: GENO_BHV_MULTI_DASH as i32,
            name: b"geno.multi_dash\0" as *const u8 as *const i8,
            cb: [
                GENO_CBID_MULTI_DASH as i32 as u8,
                GENO_CBID_NONE as i32 as u8,
                GENO_CBID_MULTI_DASH as i32 as u8,
                GENO_CBID_MULTI_DASH as i32 as u8,
            ],
            enter: Some(
                geno_multi_dash_enter
                    as unsafe extern "C" fn(
                        *mut Fighter_GObj,
                        *mut Fighter,
                        *mut GenoState,
                        i32,
                    ) -> (),
            ),
            next: GENO_NONE_T,
            land: GENO_AUTO,
            like: ftCo_MS_Fall as i32,
        },
    ]
};

pub static mut Geno_ArtDesc: *mut *mut Article = unsafe { core::mem::zeroed() };

pub static mut Geno_ArtAttr: *mut *mut ItemAttr = unsafe { core::mem::zeroed() };

pub static mut Geno_ArtModel: *mut *mut ItemModelDesc = unsafe { core::mem::zeroed() };

pub static mut Geno_ArtProfileCapacity: i32 = unsafe { core::mem::zeroed() };

pub static mut Geno_ArtStates: ItemStateArray = unsafe { core::mem::zeroed() };

pub static mut geno_art_states: [ItemStateTable; 1] = unsafe {
    [
        ItemStateTable {
            anim_id: -(1 as enum_t),
            animated: Some(
                geno_art_anim as unsafe extern "C" fn(*mut Item_GObj) -> i32,
            ),
            physics_updated: Some(
                geno_art_phys as unsafe extern "C" fn(*mut Item_GObj) -> (),
            ),
            collided: Some(
                geno_art_coll as unsafe extern "C" fn(*mut Item_GObj) -> i32,
            ),
        },
    ]
};

pub static mut geno_fx_frame: i32 = unsafe { core::mem::zeroed() };

pub static mut geno_fx_gobj: *mut HSD_GObj = unsafe { core::mem::zeroed() };

pub static mut geno_art_logic: ItemLogicTable = unsafe {
    ItemLogicTable {
        states: &raw const geno_art_states as *mut ItemStateTable,
        spawned: None,
        destroyed: Some(
            geno_art_destroyed as unsafe extern "C" fn(*mut Item_GObj) -> (),
        ),
        picked_up: None,
        dropped: None,
        thrown: None,
        dmg_dealt: Some(geno_art_hit as unsafe extern "C" fn(*mut Item_GObj) -> i32),
        dmg_received: None,
        entered_air: None,
        reflected: Some(
            geno_art_reflected as unsafe extern "C" fn(*mut Item_GObj) -> i32,
        ),
        clanked: Some(geno_art_clank as unsafe extern "C" fn(*mut Item_GObj) -> i32),
        absorbed: Some(
            geno_art_absorbed as unsafe extern "C" fn(*mut Item_GObj) -> i32,
        ),
        shield_bounced: Some(
            geno_art_shield as unsafe extern "C" fn(*mut Item_GObj) -> i32,
        ),
        hit_shield: Some(
            geno_art_shield as unsafe extern "C" fn(*mut Item_GObj) -> i32,
        ),
        evt_unk: Some(
            geno_art_unref as unsafe extern "C" fn(*mut Item_GObj, *mut HSD_GObj) -> (),
        ),
    }
};

pub static mut geno_sfields: [Anon_anon_2375; 42] = unsafe { [
        Anon_anon_2375 {
            name: b"profile\0" as *const u8 as *const i8,
            off: 0 as u64 as i32,
            size: ::core::mem::size_of::<i32>() as i32,
        },
        Anon_anon_2375 {
            name: b"kind\0" as *const u8 as *const i8,
            off: 4 as u64 as i32,
            size: ::core::mem::size_of::<i32>() as i32,
        },
        Anon_anon_2375 {
            name: b"flags\0" as *const u8 as *const i8,
            off: 8 as u64 as i32,
            size: ::core::mem::size_of::<u32>() as i32,
        },
        Anon_anon_2375 {
            name: b"resets\0" as *const u8 as *const i8,
            off: 12 as u64 as i32,
            size: ::core::mem::size_of::<u32>() as i32,
        },
        Anon_anon_2375 {
            name: b"la_i\0" as *const u8 as *const i8,
            off: 16 as u64 as i32,
            size: ::core::mem::size_of::<[i32; 64]>() as i32,
        },
        Anon_anon_2375 {
            name: b"ra_i\0" as *const u8 as *const i8,
            off: 272 as u64 as i32,
            size: ::core::mem::size_of::<[i32; 64]>() as i32,
        },
        Anon_anon_2375 {
            name: b"la_f\0" as *const u8 as *const i8,
            off: 528 as u64 as i32,
            size: ::core::mem::size_of::<[f32; 64]>() as i32,
        },
        Anon_anon_2375 {
            name: b"ra_f\0" as *const u8 as *const i8,
            off: 784 as u64 as i32,
            size: ::core::mem::size_of::<[f32; 64]>() as i32,
        },
        Anon_anon_2375 {
            name: b"hook_calls\0" as *const u8 as *const i8,
            off: 1040 as u64 as i32,
            size: ::core::mem::size_of::<u32>() as i32,
        },
        Anon_anon_2375 {
            name: b"extra_jumps\0" as *const u8 as *const i8,
            off: 1044 as u64 as i32,
            size: ::core::mem::size_of::<u32>() as i32,
        },
        Anon_anon_2375 {
            name: b"action_time\0" as *const u8 as *const i8,
            off: 1048 as u64 as i32,
            size: ::core::mem::size_of::<i32>() as i32,
        },
        Anon_anon_2375 {
            name: b"nchecks\0" as *const u8 as *const i8,
            off: 1052 as u64 as i32,
            size: ::core::mem::size_of::<u32>() as i32,
        },
        Anon_anon_2375 {
            name: b"last_check\0" as *const u8 as *const i8,
            off: 1056 as u64 as i32,
            size: ::core::mem::size_of::<i32>() as i32,
        },
        Anon_anon_2375 {
            name: b"checks\0" as *const u8 as *const i8,
            off: 1060 as u64 as i32,
            size: ::core::mem::size_of::<[GenoCheck; 8]>() as i32,
        },
        Anon_anon_2375 {
            name: b"rehit_period\0" as *const u8 as *const i8,
            off: 1444 as u64 as i32,
            size: ::core::mem::size_of::<[i32; 4]>() as i32,
        },
        Anon_anon_2375 {
            name: b"rehit_count\0" as *const u8 as *const i8,
            off: 1460 as u64 as i32,
            size: ::core::mem::size_of::<[i32; 4]>() as i32,
        },
        Anon_anon_2375 {
            name: b"link_mode\0" as *const u8 as *const i8,
            off: 1476 as u64 as i32,
            size: ::core::mem::size_of::<[i32; 4]>() as i32,
        },
        Anon_anon_2375 {
            name: b"in_coll\0" as *const u8 as *const i8,
            off: 1540 as u64 as i32,
            size: ::core::mem::size_of::<u32>() as i32,
        },
        Anon_anon_2375 {
            name: b"edge_pending\0" as *const u8 as *const i8,
            off: 1544 as u64 as i32,
            size: ::core::mem::size_of::<u32>() as i32,
        },
        Anon_anon_2375 {
            name: b"edge_target\0" as *const u8 as *const i8,
            off: 1548 as u64 as i32,
            size: ::core::mem::size_of::<u32>() as i32,
        },
        Anon_anon_2375 {
            name: b"changes\0" as *const u8 as *const i8,
            off: 1552 as u64 as i32,
            size: ::core::mem::size_of::<u32>() as i32,
        },
        Anon_anon_2375 {
            name: b"hold_motion\0" as *const u8 as *const i8,
            off: 1556 as u64 as i32,
            size: ::core::mem::size_of::<i32>() as i32,
        },
        Anon_anon_2375 {
            name: b"hold_frames\0" as *const u8 as *const i8,
            off: 1560 as u64 as i32,
            size: ::core::mem::size_of::<i32>() as i32,
        },
        Anon_anon_2375 {
            name: b"move_i\0" as *const u8 as *const i8,
            off: 1564 as u64 as i32,
            size: ::core::mem::size_of::<[i32; 8]>() as i32,
        },
        Anon_anon_2375 {
            name: b"move_f\0" as *const u8 as *const i8,
            off: 1596 as u64 as i32,
            size: ::core::mem::size_of::<[f32; 8]>() as i32,
        },
        Anon_anon_2375 {
            name: b"state_entries\0" as *const u8 as *const i8,
            off: 1628 as u64 as i32,
            size: ::core::mem::size_of::<u32>() as i32,
        },
        Anon_anon_2375 {
            name: b"enter_from\0" as *const u8 as *const i8,
            off: 1632 as u64 as i32,
            size: ::core::mem::size_of::<i32>() as i32,
        },
        Anon_anon_2375 {
            name: b"hidden\0" as *const u8 as *const i8,
            off: 1636 as u64 as i32,
            size: ::core::mem::size_of::<i32>() as i32,
        },
        Anon_anon_2375 {
            name: b"ledge\0" as *const u8 as *const i8,
            off: 1668 as u64 as i32,
            size: ::core::mem::size_of::<i32>() as i32,
        },
        Anon_anon_2375 {
            name: b"motion_started\0" as *const u8 as *const i8,
            off: 1672 as u64 as i32,
            size: ::core::mem::size_of::<i32>() as i32,
        },
        Anon_anon_2375 {
            name: b"motion_vy\0" as *const u8 as *const i8,
            off: 1676 as u64 as i32,
            size: ::core::mem::size_of::<f32>() as i32,
        },
        Anon_anon_2375 {
            name: b"motion_land\0" as *const u8 as *const i8,
            off: 1680 as u64 as i32,
            size: ::core::mem::size_of::<i32>() as i32,
        },
        Anon_anon_2375 {
            name: b"motion_facing\0" as *const u8 as *const i8,
            off: 1684 as u64 as i32,
            size: ::core::mem::size_of::<f32>() as i32,
        },
        Anon_anon_2375 {
            name: b"motion_gravity\0" as *const u8 as *const i8,
            off: 1688 as u64 as i32,
            size: ::core::mem::size_of::<f32>() as i32,
        },
        Anon_anon_2375 {
            name: b"enter_keep\0" as *const u8 as *const i8,
            off: 1692 as u64 as i32,
            size: ::core::mem::size_of::<i32>() as i32,
        },
        Anon_anon_2375 {
            name: b"hit_count\0" as *const u8 as *const i8,
            off: 1640 as u64 as i32,
            size: ::core::mem::size_of::<i32>() as i32,
        },
        Anon_anon_2375 {
            name: b"hit_damage\0" as *const u8 as *const i8,
            off: 1644 as u64 as i32,
            size: ::core::mem::size_of::<f32>() as i32,
        },
        Anon_anon_2375 {
            name: b"hit_port\0" as *const u8 as *const i8,
            off: 1648 as u64 as i32,
            size: ::core::mem::size_of::<i32>() as i32,
        },
        Anon_anon_2375 {
            name: b"hit_counter\0" as *const u8 as *const i8,
            off: 1652 as u64 as i32,
            size: ::core::mem::size_of::<i32>() as i32,
        },
        Anon_anon_2375 {
            name: b"counters\0" as *const u8 as *const i8,
            off: 1656 as u64 as i32,
            size: ::core::mem::size_of::<i32>() as i32,
        },
        Anon_anon_2375 {
            name: b"art_spawned\0" as *const u8 as *const i8,
            off: 1660 as u64 as i32,
            size: ::core::mem::size_of::<i32>() as i32,
        },
        Anon_anon_2375 {
            name: b"fall_ledge\0" as *const u8 as *const i8,
            off: 1664 as u64 as i32,
            size: ::core::mem::size_of::<i32>() as i32,
        },
    ] };

pub static mut geno_item_desc: [Article; 64] = unsafe { core::mem::zeroed() };

pub static mut geno_item_attr: [ItemAttr; 64] = unsafe { core::mem::zeroed() };

pub static mut geno_item_model: [ItemModelDesc; 64] = unsafe { core::mem::zeroed() };

pub static mut geno_item_joint: HSD_Joint = unsafe { core::mem::zeroed() };

pub static mut geno_item_anims: ItemStateArray = unsafe { core::mem::zeroed() };

pub static mut geno_family_items: [GenoFamilyItem; 128] = unsafe { core::mem::zeroed() };

pub static mut geno_item_states: [ItemStateTable; 1] = unsafe {
    [
        ItemStateTable {
            anim_id: -(1 as enum_t),
            animated: Some(
                geno_item_anim as unsafe extern "C" fn(*mut Item_GObj) -> i32,
            ),
            physics_updated: Some(
                geno_item_phys as unsafe extern "C" fn(*mut Item_GObj) -> (),
            ),
            collided: Some(
                geno_item_coll as unsafe extern "C" fn(*mut Item_GObj) -> i32,
            ),
        },
    ]
};

pub static mut geno_item_logic: ItemLogicTable = unsafe {
    ItemLogicTable {
        states: &raw const geno_item_states as *mut ItemStateTable,
        spawned: None,
        destroyed: Some(
            geno_item_destroyed as unsafe extern "C" fn(*mut Item_GObj) -> (),
        ),
        picked_up: None,
        dropped: None,
        thrown: None,
        dmg_dealt: None,
        dmg_received: None,
        entered_air: None,
        reflected: None,
        clanked: None,
        absorbed: None,
        shield_bounced: None,
        hit_shield: None,
        evt_unk: Some(
            geno_art_unref as unsafe extern "C" fn(*mut Item_GObj, *mut HSD_GObj) -> (),
        ),
    }
};

pub static mut geno_item_snap_fixture: Item = unsafe { core::mem::zeroed() };

pub static mut geno_item_snap_gobj: HSD_GObj = unsafe { core::mem::zeroed() };

pub static mut geno_item_snap_heads: [*mut HSD_GObj; 32] = unsafe { core::mem::zeroed() };

pub static mut Geno_ComboBlock: [[GenoComboBlock; 2]; 6] = unsafe { core::mem::zeroed() };

pub static mut geno_combo_entering: i32 = unsafe { core::mem::zeroed() };

pub unsafe extern "C" fn HSD_GObjGetUserData(
    mut gobj: *mut HSD_GObj,
) -> *mut c_void {
    melee_core::ct!("HSD_GObjGetUserData");
    return (*gobj).user_data;
}

pub unsafe extern "C" fn HSD_GObjGetHSDObj(
    mut gobj: *mut HSD_GObj,
) -> *mut c_void {
    melee_core::ct!("HSD_GObjGetHSDObj");
    return (*gobj).hsd_obj;
}

pub unsafe extern "C" fn HSD_JObjGetChild(mut jobj: *mut HSD_JObj) -> *mut HSD_JObj {
    melee_core::ct!("HSD_JObjGetChild");
    if jobj.is_null() {
        return ::core::ptr::null_mut::<HSD_JObj>()
    } else {
        return (*jobj).child as *mut HSD_JObj
    };
}

pub unsafe extern "C" fn HSD_JObjGetNext(mut jobj: *mut HSD_JObj) -> *mut HSD_JObj {
    melee_core::ct!("HSD_JObjGetNext");
    if jobj.is_null() {
        return ::core::ptr::null_mut::<HSD_JObj>()
    } else {
        return (*jobj).next as *mut HSD_JObj
    };
}

pub unsafe extern "C" fn HSD_JObjGetParent(mut jobj: *mut HSD_JObj) -> *mut HSD_JObj {
    melee_core::ct!("HSD_JObjGetParent");
    if jobj.is_null() {
        return ::core::ptr::null_mut::<HSD_JObj>()
    } else {
        return (*jobj).parent as *mut HSD_JObj
    };
}

pub unsafe extern "C" fn HSD_JObjMtxIsDirty(mut jobj: *mut HSD_JObj) -> i32 {
    melee_core::ct!("HSD_JObjMtxIsDirty");
    let mut result: i32 = 0;
    if !jobj.is_null() {} else {
        __assert(
            b"src/sysdolphin/baselib/jobj.h\0" as *const u8
                as *const i8,
            228 as u32,
            b"jobj\0" as *const u8 as *const i8,
        );
    };
    result = 0 as i32;
    if (*jobj).flags & JOBJ_USER_DEF_MTX as u32 == 0
        && (*jobj).flags & JOBJ_MTX_DIRTY as u32 != 0
    {
        result = 1 as i32;
    }
    return result;
}

pub unsafe extern "C" fn HSD_JObjSetupMatrix(mut jobj: *mut HSD_JObj) {
    melee_core::ct!("HSD_JObjSetupMatrix");
    if jobj.is_null() || HSD_JObjMtxIsDirty(jobj) == 0 {
        return;
    }
    HSD_JObjSetupMatrixSub(jobj);
}

pub unsafe extern "C" fn HSD_JObjSetRotationX(mut jobj: *mut HSD_JObj, mut x: f32) {
    melee_core::ct!("HSD_JObjSetRotationX");
    if !jobj.is_null() {} else {
        __assert(
            b"src/sysdolphin/baselib/jobj.h\0" as *const u8
                as *const i8,
            297 as u32,
            b"jobj\0" as *const u8 as *const i8,
        );
    };
    if (*jobj).flags & ((1 as i32) << 17 as i32) as u32
        == 0
    {} else {
        __assert(
            b"src/sysdolphin/baselib/jobj.h\0" as *const u8
                as *const i8,
            298 as u32,
            b"!(jobj->flags & JOBJ_USE_QUATERNION)\0" as *const u8
                as *const i8,
        );
    };
    (*jobj).rotate.x = x;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
        if !jobj.is_null() && HSD_JObjMtxIsDirty(jobj) == 0 {
            HSD_JObjSetMtxDirtySub(jobj);
        }
    }
}

pub unsafe extern "C" fn HSD_JObjSetRotationY(mut jobj: *mut HSD_JObj, mut y: f32) {
    melee_core::ct!("HSD_JObjSetRotationY");
    if !jobj.is_null() {} else {
        __assert(
            b"src/sysdolphin/baselib/jobj.h\0" as *const u8
                as *const i8,
            317 as u32,
            b"jobj\0" as *const u8 as *const i8,
        );
    };
    if (*jobj).flags & ((1 as i32) << 17 as i32) as u32
        == 0
    {} else {
        __assert(
            b"src/sysdolphin/baselib/jobj.h\0" as *const u8
                as *const i8,
            318 as u32,
            b"!(jobj->flags & JOBJ_USE_QUATERNION)\0" as *const u8
                as *const i8,
        );
    };
    (*jobj).rotate.y = y;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
        if !jobj.is_null() && HSD_JObjMtxIsDirty(jobj) == 0 {
            HSD_JObjSetMtxDirtySub(jobj);
        }
    }
}

pub unsafe extern "C" fn HSD_JObjSetRotationZ(mut jobj: *mut HSD_JObj, mut z: f32) {
    melee_core::ct!("HSD_JObjSetRotationZ");
    if !jobj.is_null() {} else {
        __assert(
            b"src/sysdolphin/baselib/jobj.h\0" as *const u8
                as *const i8,
            337 as u32,
            b"jobj\0" as *const u8 as *const i8,
        );
    };
    if (*jobj).flags & ((1 as i32) << 17 as i32) as u32
        == 0
    {} else {
        __assert(
            b"src/sysdolphin/baselib/jobj.h\0" as *const u8
                as *const i8,
            338 as u32,
            b"!(jobj->flags & JOBJ_USE_QUATERNION)\0" as *const u8
                as *const i8,
        );
    };
    (*jobj).rotate.z = z;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
        if !jobj.is_null() && HSD_JObjMtxIsDirty(jobj) == 0 {
            HSD_JObjSetMtxDirtySub(jobj);
        }
    }
}

pub unsafe extern "C" fn HSD_JObjSetTranslate(
    mut jobj: *mut HSD_JObj,
    mut translate: *mut Vec,
) {
    melee_core::ct!("HSD_JObjSetTranslate");
    if !jobj.is_null() {} else {
        __assert(
            b"src/sysdolphin/baselib/jobj.h\0" as *const u8
                as *const i8,
            481 as u32,
            b"jobj\0" as *const u8 as *const i8,
        );
    };
    if !translate.is_null() {} else {
        __assert(
            b"src/sysdolphin/baselib/jobj.h\0" as *const u8
                as *const i8,
            482 as u32,
            b"translate\0" as *const u8 as *const i8,
        );
    };
    (*jobj).translate = *translate;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
        if !jobj.is_null() && HSD_JObjMtxIsDirty(jobj) == 0 {
            HSD_JObjSetMtxDirtySub(jobj);
        }
    }
}

pub unsafe extern "C" fn geno_cmd_words(mut w0: u32) -> i32 {
    melee_core::ct!("geno_cmd_words");
    static mut len: [u8; 59] = [
        1 as i32 as u8,
        1 as i32 as u8,
        1 as i32 as u8,
        1 as i32 as u8,
        1 as i32 as u8,
        2 as i32 as u8,
        1 as i32 as u8,
        2 as i32 as u8,
        1 as i32 as u8,
        1 as i32 as u8,
        5 as i32 as u8,
        5 as i32 as u8,
        1 as i32 as u8,
        1 as i32 as u8,
        1 as i32 as u8,
        1 as i32 as u8,
        1 as i32 as u8,
        3 as i32 as u8,
        1 as i32 as u8,
        1 as i32 as u8,
        1 as i32 as u8,
        1 as i32 as u8,
        1 as i32 as u8,
        1 as i32 as u8,
        1 as i32 as u8,
        1 as i32 as u8,
        1 as i32 as u8,
        1 as i32 as u8,
        1 as i32 as u8,
        1 as i32 as u8,
        1 as i32 as u8,
        1 as i32 as u8,
        1 as i32 as u8,
        1 as i32 as u8,
        3 as i32 as u8,
        1 as i32 as u8,
        1 as i32 as u8,
        1 as i32 as u8,
        7 as i32 as u8,
        4 as i32 as u8,
        1 as i32 as u8,
        1 as i32 as u8,
        1 as i32 as u8,
        1 as i32 as u8,
        1 as i32 as u8,
        1 as i32 as u8,
        1 as i32 as u8,
        1 as i32 as u8,
        1 as i32 as u8,
        1 as i32 as u8,
        1 as i32 as u8,
        1 as i32 as u8,
        1 as i32 as u8,
        1 as i32 as u8,
        3 as i32 as u8,
        3 as i32 as u8,
        2 as i32 as u8,
        1 as i32 as u8,
        4 as i32 as u8,
    ];
    let mut op: u32 = w0 >> 26 as i32;
    if op < 59 as u32 {
        return len[op as usize] as i32;
    }
    if op == 59 as u32 {
        let mut l: u32 = w0 >> 16 as i32
            & 15 as u32;
        return if l != 0 { l as i32 } else { 1 as i32 };
    }
    return 1 as i32;
}

pub unsafe extern "C" fn geno_state(mut fp: *mut Fighter) -> *mut GenoState {
    melee_core::ct!("geno_state");
    let mut p: i32 = if ((*fp).player_id as i32)
        < GENO_PLAYERS
    {
        (*fp).player_id as i32
    } else {
        GENO_PLAYERS - 1 as i32
    };
    return (&raw mut *(&raw mut Geno_StateBlock as *mut [GenoState; 2])
        .offset(p as isize) as *mut GenoState)
        .offset(
            (if (*fp).is_sub_fighter() as i32 != 0 {
                1 as i32
            } else {
                0 as i32
            }) as isize,
        ) as *mut GenoState;
}

pub unsafe extern "C" fn geno_zero(
    mut p: *mut c_void,
    mut n: i32,
) {
    melee_core::ct!("geno_zero");
    let mut b: *mut u8 = p as *mut u8;
    loop {
        let fresh0 = n;
        n = n - 1;
        if !(fresh0 > 0 as i32) {
            break;
        }
        let fresh1 = b;
        b = b.offset(1);
        *fresh1 = 0 as u8;
    };
}

pub unsafe extern "C" fn hook_log(
    mut gobj: *mut Fighter_GObj,
    mut fp: *mut Fighter,
    mut st: *mut GenoState,
    mut arg: i32,
) -> i32 {
    melee_core::ct!("hook_log");
    Geno_Event(
        2 as i32,
        (*fp).kind as i32,
        (*fp).player_id as i32,
        arg as i32,
        (*st).la_i[0 as i32 as usize],
    );
    return 0 as i32;
}

pub unsafe extern "C" fn hook_jumps_refill(
    mut gobj: *mut Fighter_GObj,
    mut fp: *mut Fighter,
    mut st: *mut GenoState,
    mut arg: i32,
) -> i32 {
    melee_core::ct!("hook_jumps_refill");
    let mut used: i32 = (*fp).x1968_jumpsUsed as i32;
    used = if arg <= 0 as i32 {
        1 as i32
    } else {
        used - arg as i32
    };
    (*fp).x1968_jumpsUsed = (if used < 1 as i32 {
        1 as i32
    } else {
        used
    }) as u8;
    return 0 as i32;
}

pub unsafe extern "C" fn hook_jumps_to_var(
    mut gobj: *mut Fighter_GObj,
    mut fp: *mut Fighter,
    mut st: *mut GenoState,
    mut arg: i32,
) -> i32 {
    melee_core::ct!("hook_jumps_to_var");
    let mut left: i32 = (*fp).co_attrs.max_jumps
        - (*fp).x1968_jumpsUsed as i32;
    (*st)
        .la_i[(arg as i32 & GENO_VARS_PER_BANK - 1 as i32)
        as usize] = (if left < 0 as i32 {
        0 as i32
    } else {
        left
    }) as i32;
    return 0 as i32;
}

pub unsafe extern "C" fn hook_count_frames(
    mut gobj: *mut Fighter_GObj,
    mut fp: *mut Fighter,
    mut st: *mut GenoState,
    mut arg: i32,
) -> i32 {
    melee_core::ct!("hook_count_frames");
    (*st)
        .la_i[(arg as i32 & GENO_VARS_PER_BANK - 1 as i32)
        as usize] += 1 as i32;
    return 0 as i32;
}

pub unsafe extern "C" fn GenoGame_LockonAim(
    mut has: i32,
    mut dx: f32,
    mut dy: f32,
    mut sx: f32,
    mut sy: f32,
    mut facing: f32,
    mut grounded: i32,
    mut max_deg: f32,
    mut stick_deg: f32,
    mut fwd: *mut f32,
    mut up: *mut f32,
    mut face: *mut f32,
) {
    melee_core::ct!("GenoGame_LockonAim");
    let mut a: f32 = 0.0f32;
    let mut lim: f32 = max_deg * M_PI as f32 / 180.0f32;
    *face = (if facing < 0.0f32 { -1.0f32 } else { 1.0f32 }) as f32;
    if has != 0 {
        let mut ax: f32 = if dx < 0.0f32 { -dx } else { dx };
        if ax > 0.001f32 {
            *face = (if dx < 0.0f32 { -1.0f32 } else { 1.0f32 }) as f32;
        }
        a = atan2f(dy as f32, ax as f32) as f32;
        a = if a > lim { lim } else if a < -lim { -lim } else { a };
    } else if sy > 0.25f32 {
        a = (stick_deg as f32 * M_PI as f32 / 180.0f32)
            as f32;
    } else if sy < -0.25f32 {
        a = (-(stick_deg as f32) * M_PI as f32
            / 180.0f32) as f32;
    }
    if grounded != 0 && a < 0.0f32 {
        a = 0.0f32 as f32;
    }
    *fwd = cosf(a as f32) as f32;
    *up = sinf(a as f32) as f32;
}

pub unsafe extern "C" fn GenoGame_AimStick(
    mut sx: f32,
    mut sy: f32,
    mut facing: f32,
    mut grounded: i32,
    mut thr: f32,
    mut max_up: f32,
    mut max_down: f32,
    mut allow_turn: i32,
    mut fwd: *mut f32,
    mut up: *mut f32,
    mut face: *mut f32,
) -> i32 {
    melee_core::ct!("GenoGame_AimStick");
    let mut a: f32 = 0.;
    let mut ax: f32 = if sx < 0.0f32 { -sx } else { sx };
    let mut hi: f32 = max_up * M_PI as f32 / 180.0f32;
    let mut lo: f32 = -max_down * M_PI as f32 / 180.0f32;
    if fmadds(sx, sx, sy * sy) < thr * thr {
        return 0 as i32;
    }
    *face = (if facing < 0.0f32 { -1.0f32 } else { 1.0f32 }) as f32;
    if allow_turn != 0 && ax > 0.0001f32 && sx * *face < 0.0f32 {
        *face = -*face;
    }
    a = atan2f(sy as f32, ax as f32) as f32;
    a = if a > hi { hi } else if a < lo { lo } else { a };
    if grounded != 0 && a < 0.0f32 {
        a = 0.0f32 as f32;
    }
    *fwd = cosf(a as f32) as f32;
    *up = sinf(a as f32) as f32;
    return 1 as i32;
}

pub unsafe extern "C" fn hook_aim_stick(
    mut gobj: *mut Fighter_GObj,
    mut fp: *mut Fighter,
    mut st: *mut GenoState,
    mut arg: i32,
) -> i32 {
    melee_core::ct!("hook_aim_stick");
    let mut fw: f32 = 0.;
    let mut up: f32 = 0.;
    let mut face: f32 = (*fp).facing_dir as f32;
    let mut used: i32 = GenoGame_AimStick(
        (*fp).input.lstick[0 as i32 as usize].x,
        (*fp).input.lstick[0 as i32 as usize].y,
        (*fp).facing_dir as f32,
        ((*fp).ground_or_air as i32 == GA_Ground as i32)
            as i32,
        (arg as i32 & 0xff as i32) as f32 / 100.0f32,
        (arg as i32 >> 8 as i32
            & 0xff as i32) as f32,
        (arg as i32 >> 16 as i32
            & 0xff as i32) as f32,
        arg as i32 >> 24 as i32 & 1 as i32,
        &raw mut fw,
        &raw mut up,
        &raw mut face,
    );
    if used != 0 {
        (*st).move_f[2 as i32 as usize] = fw;
        (*st).move_f[0 as i32 as usize] = (*st)
            .move_f[2 as i32 as usize];
        (*st).move_f[3 as i32 as usize] = up;
        (*st).move_f[1 as i32 as usize] = (*st)
            .move_f[3 as i32 as usize];
        (*st).move_i[2 as i32 as usize] = 1 as i32 as i32;
        if face != (*fp).facing_dir {
            (*fp).facing_dir = face as f32;
            if !(*fp).parts.is_null() {
                ftPartSetRotY(
                    fp,
                    0 as i32,
                    1.5707964f32 * (*fp).facing_dir as f32,
                );
            }
        }
    } else if (*st).move_i[2 as i32 as usize] != 0 {
        (*st).move_f[0 as i32 as usize] = (*st)
            .move_f[2 as i32 as usize];
        (*st).move_f[1 as i32 as usize] = (*st)
            .move_f[3 as i32 as usize];
    }
    Geno_Event(
        42 as i32,
        (*fp).kind as i32,
        (*fp).player_id as i32,
        used,
        (atan2f(
            (*st).move_f[1 as i32 as usize],
            (*st).move_f[0 as i32 as usize],
        ) as f32 * 180.0f32 / M_PI as f32) as i32,
    );
    return 0 as i32;
}

pub unsafe extern "C" fn hook_lockon(
    mut gobj: *mut Fighter_GObj,
    mut fp: *mut Fighter,
    mut st: *mut GenoState,
    mut arg: i32,
) -> i32 {
    melee_core::ct!("hook_lockon");
    let mut cur: *mut HSD_GObj = ::core::ptr::null_mut::<HSD_GObj>();
    let mut range: f32 = (arg as i32 & 0xffff as i32)
        as f32;
    let mut best: f32 = -1.0f32;
    let mut tx: f32 = 0.0f32;
    let mut ty: f32 = 0.0f32;
    let mut face: f32 = 0.;
    let mut has: i32 = 0 as i32;
    if !HSD_GObjPLinkHead.is_null() {
        cur = *HSD_GObjPLinkHead.offset(HSD_GOBJ_PLINK_FIGHTER as isize);
        while !cur.is_null() {
            let mut o: *mut Fighter = HSD_GObjGetUserData(cur) as *mut Fighter;
            let mut dx: f32 = 0.;
            let mut dy: f32 = 0.;
            let mut d: f32 = 0.;
            if !(o.is_null() || cur == gobj as *mut HSD_GObj
                || (*o).player_id as i32
                    == (*fp).player_id as i32)
            {
                dx = (*o).cur_pos.x - (*fp).cur_pos.x;
                dy = (*o).cur_pos.y - (*fp).cur_pos.y;
                d = fmadds(dx, dx, dy * dy);
                if !(range > 0.0f32 && d > range * range) {
                    if best < 0.0f32 || d < best {
                        best = d;
                        tx = dx;
                        ty = dy;
                        has = 1 as i32;
                    }
                }
            }
            cur = (*cur).next as *mut HSD_GObj;
        }
    }
    GenoGame_LockonAim(
        has,
        tx,
        ty,
        (*fp).input.lstick[0 as i32 as usize].x,
        (*fp).input.lstick[0 as i32 as usize].y,
        (*fp).facing_dir as f32,
        ((*fp).ground_or_air as i32 == GA_Ground as i32)
            as i32,
        (arg as i32 >> 16 as i32
            & 0xff as i32) as f32,
        (arg as u32 >> 24 as i32) as f32,
        (&raw mut (*st).move_f as *mut f32).offset(0 as i32 as isize)
            as *mut f32,
        (&raw mut (*st).move_f as *mut f32).offset(1 as i32 as isize)
            as *mut f32,
        &raw mut face,
    );
    (*fp).facing_dir = face as f32;
    (*st).move_i[0 as i32 as usize] = has as i32;
    Geno_Event(
        40 as i32,
        (*fp).kind as i32,
        (*fp).player_id as i32,
        has,
        (atan2f(
            (*st).move_f[1 as i32 as usize],
            (*st).move_f[0 as i32 as usize],
        ) as f32 * 180.0f32 / M_PI as f32) as i32,
    );
    return 0 as i32;
}

pub unsafe extern "C" fn Geno_MotionAnimRow(
    mut fp: *mut Fighter,
    mut motion: i32,
) -> i32 {
    melee_core::ct!("Geno_MotionAnimRow");
    let mut st: *mut GenoState = geno_state(fp);
    if geno_inert(st) != 0 || (*st).profile < 0 as i32 {
        return -(1 as i32);
    }
    return Geno_MotionAnim((*st).profile as i32, motion);
}

pub unsafe extern "C" fn Geno_FallLimit(
    mut fp: *mut Fighter,
    mut terminal: f32,
) -> f32 {
    melee_core::ct!("Geno_FallLimit");
    let mut st: *mut GenoState = geno_state(fp);
    return if !st.is_null() && (*st).fall_limit > terminal {
        (*st).fall_limit
    } else {
        terminal
    };
}

pub unsafe extern "C" fn geno_streq(
    mut a: *const i8,
    mut b: *const i8,
) -> i32 {
    melee_core::ct!("geno_streq");
    while *a as i32 != '\0' as i32
        && *a as i32 == *b as i32
    {
        a = a.offset(1);
        b = b.offset(1);
    }
    return (*a as i32 == *b as i32) as i32;
}

pub unsafe extern "C" fn GenoGame_HookFind(
    mut name: *const i8,
) -> i32 {
    melee_core::ct!("GenoGame_HookFind");
    let mut i: i32 = 0;
    i = 0 as i32;
    while i < GENO_NHOOKS {
        if geno_streq(name, geno_hooks[i as usize].name) != 0 {
            return geno_hooks[i as usize].id;
        }
        i += 1;
    }
    return -(1 as i32);
}

pub unsafe extern "C" fn GenoGame_CallHook(
    mut gobj: *mut Fighter_GObj,
    mut id: i32,
    mut arg: i32,
) -> i32 {
    melee_core::ct!("GenoGame_CallHook");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj as *mut HSD_GObj)
        as *mut Fighter;
    let mut st: *mut GenoState = geno_state(fp);
    let mut i: i32 = 0;
    i = 0 as i32;
    while i < GENO_NHOOKS {
        if geno_hooks[i as usize].id == id {
            (*st).hook_calls = (*st).hook_calls.wrapping_add(1);
            geno_hooks[i as usize]
                .r#fn
                .expect("non-null function pointer")(gobj, fp, st, arg);
            return 0 as i32;
        }
        i += 1;
    }
    Geno_Event(
        4 as i32,
        (*fp).kind as i32,
        (*fp).player_id as i32,
        id,
        0 as i32,
    );
    return -(1 as i32);
}

pub unsafe extern "C" fn geno_run_event(
    mut gobj: *mut Fighter_GObj,
    mut st: *mut GenoState,
    mut ev: i32,
) {
    melee_core::ct!("geno_run_event");
    let mut n: i32 = Geno_HookCount(
        (*st).profile as i32,
        ev,
    );
    let mut i: i32 = 0;
    i = 0 as i32;
    while i < n {
        GenoGame_CallHook(
            gobj,
            Geno_Hook((*st).profile as i32, ev, i),
            Geno_HookArg((*st).profile as i32, ev, i) as i32,
        );
        i += 1;
    }
}

pub unsafe extern "C" fn geno_pool_sync() {
    melee_core::ct!("geno_pool_sync");
    let mut gen: i32 = Geno_PoolGen();
    let mut n: i32 = 0;
    let mut i: i32 = 0;
    if gen == Geno_ScriptPoolGen {
        return;
    }
    n = Geno_PoolWords();
    if n > GENO_POOL_WORDS {
        n = GENO_POOL_WORDS;
    }
    i = 0 as i32;
    while i < n {
        Geno_ScriptPool[i as usize] = (Geno_PoolWord(i) as u32).to_be();
        i += 1;
    }
    let mut s: i32 = 0;
    let mut slots: i32 = Geno_SlotCount();
    s = 0 as i32;
    while s < slots && s < GENO_MAX_SLOTS {
        let mut off: i32 = 0;
        let mut len: i32 = 0;
        let mut q: i32 = 0;
        if !(Geno_SlotRelative(s) == 0) {
            off = Geno_SlotOffset(s);
            len = Geno_SlotLen(s);
            if !(off < 0 as i32 || off + len > n) {
                q = 0 as i32;
                while q < len {
                    let mut w: u32 = u32::from_be(Geno_ScriptPool[(off + q) as usize]);
                    let mut op: u32 = w >> 26 as i32;
                    if (op == 5 as u32 || op == 7 as u32)
                        && (q + 1 as i32) < len
                    {
                        Geno_ScriptPool[(off + q + 1 as i32) as usize] = ((&raw mut Geno_ScriptPool
                            as *mut u32)
                            .offset(
                                (off
                                    + u32::from_be(*(&raw mut Geno_ScriptPool as *mut u32)
                                        .offset((off + q + 1 as i32) as isize))
                                        as i32) as isize,
                            ) as *mut u32 as u32).to_be();
                    }
                    q += geno_cmd_words(w as u32);
                }
            }
        }
        s += 1;
    }
    Geno_ScriptPoolGen = gen as i32;
}

pub unsafe extern "C" fn geno_install_overlays(
    mut fp: *mut Fighter,
    mut p: i32,
) {
    melee_core::ct!("geno_install_overlays");
    let mut n: i32 = Geno_OverlayCount(p);
    let mut i: i32 = 0;
    let mut table: *mut Fighter_WaitAnimData = ::core::ptr::null_mut::<
        Fighter_WaitAnimData,
    >();
    if n <= 0 as i32 || (*fp).ft_data.is_null()
        || {
            table = (*(*fp).ft_data).xC;
            table.is_null()
        }
    {
        return;
    }
    geno_pool_sync();
    i = 0 as i32;
    while i < n {
        let mut anim: i32 = Geno_OverlayAnim(p, i);
        let mut slot: i32 = Geno_OverlaySlot(p, i);
        let mut off: i32 = Geno_SlotOffset(slot);
        let mut mine: *mut CmdUnion = ::core::ptr::null_mut::<CmdUnion>();
        if Geno_DefineBaseKind((*fp).kind as i32)
            >= 0 as i32 && anim as u32 >= (*fp).x58C
        {
            static mut reduced_noted: i32 = 0;
            if (*fp).x58C <= 16 as u32 {
                if reduced_noted == 0 {
                    reduced_noted = 1 as i32;
                    OSReport!(
                        b"geno: overlay %d skipped on a reduced intro copy (rows %d)\n\0"
                            as *const u8 as *const i8
                            as *mut i8,
                        anim,
                        (*fp).x58C,
                    );
                }
            } else {
                OSReport!(
                    b"geno: native definition refused out-of-range overlay %d (rows %d)\n\0"
                        as *const u8 as *const i8
                        as *mut i8,
                    anim,
                    (*fp).x58C,
                );
            }
        } else if !(anim < 0 as i32 || slot < 0 as i32
            || slot >= GENO_MAX_SLOTS || off < 0 as i32
            || off >= GENO_POOL_WORDS)
        {
            mine = (&raw mut Geno_ScriptPool as *mut u32).offset(off as isize)
                as *mut u32 as *mut CmdUnion;
            if (*table.offset(anim as isize)).xC != mine {
                let mut was: *mut u32 = (*table.offset(anim as isize)).xC
                    as *mut u32;
                if !(was
                    >= (&raw mut Geno_ScriptPool as *mut u32)
                        .offset(0 as i32 as isize) as *mut u32
                    && was
                        < (&raw mut Geno_ScriptPool as *mut u32)
                            .offset(GENO_POOL_WORDS as isize) as *mut u32)
                {
                    Geno_OverlayOrig[slot as usize] = (*table.offset(anim as isize)).xC;
                }
                let ref mut fresh14 = (*table.offset(anim as isize)).xC;
                *fresh14 = mine;
                Geno_Event(
                    13 as i32,
                    (*fp).kind as i32,
                    (*fp).player_id as i32,
                    anim,
                    slot,
                );
            }
        }
        i += 1;
    }
}

pub unsafe extern "C" fn geno_slot_of(mut w: *mut u32) -> i32 {
    melee_core::ct!("geno_slot_of");
    let mut off: i32 = 0;
    let mut s: i32 = 0;
    let mut n: i32 = 0;
    if w
        < (&raw mut Geno_ScriptPool as *mut u32)
            .offset(0 as i32 as isize) as *mut u32
        || w
            >= (&raw mut Geno_ScriptPool as *mut u32).offset(GENO_POOL_WORDS as isize)
                as *mut u32
    {
        return -(1 as i32);
    }
    off = w
        .offset_from(
            (&raw mut Geno_ScriptPool as *mut u32)
                .offset(0 as i32 as isize) as *mut u32,
        ) as i64 as i32;
    n = Geno_SlotCount();
    s = 0 as i32;
    while s < n && s < GENO_MAX_SLOTS {
        let mut o: i32 = Geno_SlotOffset(s);
        if off >= o && off < o + Geno_SlotLen(s) {
            return s;
        }
        s += 1;
    }
    return -(1 as i32);
}

pub unsafe extern "C" fn GenoGame_ScriptRange(
    mut address: u32,
    mut bytes: u32,
) -> i32 {
    melee_core::ct!("GenoGame_ScriptRange");
    let mut s: i32 = 0;
    let mut n: i32 = Geno_SlotCount();
    if bytes == 0 as u32 || address.wrapping_add(bytes) < address
        || address & 3 as u32 != 0
    {
        return 0 as i32;
    }
    s = 0 as i32;
    while s < n && s < GENO_MAX_SLOTS {
        let mut off: i32 = Geno_SlotOffset(s);
        let mut len: i32 = Geno_SlotLen(s);
        let mut start: u32 = 0;
        if !(off < 0 as i32 || len < 1 as i32
            || off > GENO_POOL_WORDS - len)
        {
            start = (&raw mut Geno_ScriptPool as *mut u32).offset(off as isize)
                as *mut u32 as u32;
            if address >= start
                && address.wrapping_sub(start) <= (len as u32).wrapping_mul(4 as u32)
                && bytes
                    <= (len as u32)
                        .wrapping_mul(4 as u32)
                        .wrapping_sub(address.wrapping_sub(start))
            {
                return 1 as i32;
            }
        }
        s += 1;
    }
    return 0 as i32;
}

pub unsafe extern "C" fn GenoGame_ScriptOriginal(mut address: u32) -> u32 {
    melee_core::ct!("GenoGame_ScriptOriginal");
    let mut s: i32 = geno_slot_of(address as *mut u32);
    return if s >= 0 as i32 {
        Geno_OverlayOrig[s as usize] as u32
    } else {
        0 as u32
    };
}

pub unsafe extern "C" fn GenoGame_ScriptHasIasa(
    mut script: *const c_void,
) -> i32 {
    melee_core::ct!("GenoGame_ScriptHasIasa");
    let mut current_block: u64;

    static mut lengths: [u8; 59] = [
        1 as i32 as u8,
        1 as i32 as u8,
        1 as i32 as u8,
        1 as i32 as u8,
        1 as i32 as u8,
        2 as i32 as u8,
        1 as i32 as u8,
        2 as i32 as u8,
        1 as i32 as u8,
        1 as i32 as u8,
        5 as i32 as u8,
        5 as i32 as u8,
        1 as i32 as u8,
        1 as i32 as u8,
        1 as i32 as u8,
        1 as i32 as u8,
        1 as i32 as u8,
        3 as i32 as u8,
        1 as i32 as u8,
        1 as i32 as u8,
        1 as i32 as u8,
        1 as i32 as u8,
        1 as i32 as u8,
        1 as i32 as u8,
        1 as i32 as u8,
        1 as i32 as u8,
        1 as i32 as u8,
        1 as i32 as u8,
        1 as i32 as u8,
        1 as i32 as u8,
        1 as i32 as u8,
        1 as i32 as u8,
        1 as i32 as u8,
        1 as i32 as u8,
        3 as i32 as u8,
        1 as i32 as u8,
        1 as i32 as u8,
        1 as i32 as u8,
        7 as i32 as u8,
        4 as i32 as u8,
        1 as i32 as u8,
        1 as i32 as u8,
        1 as i32 as u8,
        1 as i32 as u8,
        1 as i32 as u8,
        1 as i32 as u8,
        1 as i32 as u8,
        1 as i32 as u8,
        1 as i32 as u8,
        1 as i32 as u8,
        1 as i32 as u8,
        1 as i32 as u8,
        1 as i32 as u8,
        1 as i32 as u8,
        1 as i32 as u8,
        3 as i32 as u8,
        3 as i32 as u8,
        2 as i32 as u8,
        1 as i32 as u8,
    ];
    let mut pc: *const u32 = script as *const u32;
    let mut ret: [*const u32; 8] = [::core::ptr::null::<u32>(); 8];
    let mut pending: [Anon_anon_545; 32] = unsafe { core::mem::zeroed() };
    let mut nret: i32 = 0 as i32;
    let mut npending: i32 = 0 as i32;
    let mut steps: i32 = 0;
    steps = 0 as i32;
    while !pc.is_null() && steps < 2000 as i32 {
        let mut w: u32 = 0;
        let mut op: i32 = 0;
        let mut len: i32 = 0;
        if Geno_ScriptAddressValid(pc as u32, 4 as u32) == 0 {
            current_block = 2781656338536092273;
        } else {
            w = u32::from_be(*pc);
            op = (w >> 26 as i32) as i32;
            if op == 23 as i32 {
                return 1 as i32;
            }
            if op == 0 as i32 || op == 8 as i32 {
                current_block = 2781656338536092273;
            } else if op == 5 as i32 || op == 7 as i32 {
                if Geno_ScriptAddressValid(pc as u32, 8 as u32) == 0 {
                    current_block = 2781656338536092273;
                } else {
                    if op == 5 as i32 {
                        if nret == 8 as i32 {
                            current_block = 2781656338536092273;
                        } else {
                            let fresh2 = nret;
                            nret = nret + 1;
                            ret[fresh2 as usize] = pc
                                .offset(2 as i32 as isize);
                            current_block = 3512920355445576850;
                        }
                    } else {
                        current_block = 3512920355445576850;
                    }
                    match current_block {
                        2781656338536092273 => {}
                        _ => {
                            pc = u32::from_be(*pc.offset(1 as i32 as isize)) as usize
                                as *const u32;
                            if pc.is_null() {
                                current_block = 2781656338536092273;
                            } else {
                                current_block = 12517898123489920830;
                            }
                        }
                    }
                }
            } else if op == 6 as i32 {
                if nret == 0 as i32 {
                    current_block = 2781656338536092273;
                } else {
                    nret -= 1;
                    pc = ret[nret as usize];
                    current_block = 12517898123489920830;
                }
            } else {
                if op == GENO_FTCMD_OP {
                    len = (w >> 16 as i32 & 15 as u32)
                        as i32;
                    if w >> 20 as i32 & 63 as u32
                        == GENO_SUB_ORIG as i32 as u32
                    {
                        pc = GenoGame_ScriptOriginal(pc as u32) as *const u32;
                        if pc.is_null() {
                            current_block = 2781656338536092273;
                        } else {
                            current_block = 12517898123489920830;
                        }
                    } else {
                        if len == 0 as i32 {
                            len = 1 as i32;
                        }
                        current_block = 3437258052017859086;
                    }
                } else if op > 58 as i32 {
                    current_block = 2781656338536092273;
                } else {
                    len = lengths[op as usize] as i32;
                    current_block = 3437258052017859086;
                }
                match current_block {
                    2781656338536092273 => {}
                    12517898123489920830 => {}
                    _ => {
                        if Geno_ScriptAddressValid(
                            pc as u32,
                            (len * 4 as i32) as u32,
                        ) == 0
                        {
                            current_block = 2781656338536092273;
                        } else {
                            if op == GENO_FTCMD_OP {
                                let mut sub: i32 = (w
                                    >> 20 as i32 & 63 as u32)
                                    as i32;
                                let mut skip_word: i32 = if sub
                                    == GENO_SUB_IF as i32
                                {
                                    2 as i32
                                } else if sub == GENO_SUB_IFV as i32 {
                                    3 as i32
                                } else {
                                    0 as i32
                                };
                                if skip_word != 0 && len > skip_word
                                    && npending < 32 as i32
                                {
                                    let mut skip: u32 = u32::from_be(*pc.offset(skip_word as isize));
                                    if skip
                                        <= (0xffffffff as u32)
                                            .wrapping_sub(pc as u32)
                                            .wrapping_div(4 as u32)
                                            .wrapping_sub(len as u32)
                                    {
                                        let mut r: i32 = 0;
                                        pending[npending as usize].pc = pc
                                            .wrapping_add((len as u32).wrapping_add(skip) as usize);
                                        pending[npending as usize].nret = nret;
                                        r = 0 as i32;
                                        while r < nret {
                                            pending[npending as usize].ret[r as usize] = ret[r
                                                as usize];
                                            r += 1;
                                        }
                                        npending += 1;
                                    }
                                    current_block = 1847472278776910194;
                                } else if sub == GENO_SUB_SKIP as i32
                                    && len >= 2 as i32
                                {
                                    let mut skip_0: u32 = u32::from_be(*pc
                                        .offset(1 as i32 as isize));
                                    if skip_0
                                        > (0xffffffff as u32)
                                            .wrapping_sub(pc as u32)
                                            .wrapping_div(4 as u32)
                                            .wrapping_sub(len as u32)
                                    {
                                        current_block = 2781656338536092273;
                                    } else {
                                        pc = pc.wrapping_add((len as u32).wrapping_add(skip_0) as usize);
                                        current_block = 12517898123489920830;
                                    }
                                } else {
                                    current_block = 1847472278776910194;
                                }
                            } else {
                                current_block = 1847472278776910194;
                            }
                            match current_block {
                                12517898123489920830 => {}
                                2781656338536092273 => {}
                                _ => {
                                    pc = pc.offset(len as isize);
                                    current_block = 12517898123489920830;
                                }
                            }
                        }
                    }
                }
            }
        }
        match current_block {
            2781656338536092273 => {
                if npending == 0 as i32 {
                    return 0 as i32;
                }
                let mut r_0: i32 = 0;
                npending -= 1;
                pc = pending[npending as usize].pc;
                nret = pending[npending as usize].nret;
                r_0 = 0 as i32;
                while r_0 < nret {
                    ret[r_0 as usize] = pending[npending as usize].ret[r_0 as usize];
                    r_0 += 1;
                }
            }
            _ => {}
        }
        steps += 1;
    }
    return 0 as i32;
}

pub unsafe extern "C" fn geno_inert(mut st: *mut GenoState) -> i32 {
    melee_core::ct!("geno_inert");
    return ((*st).profile < 0 as i32
        && (*st).flags as u32 & GENO_SF_SCRIPT == 0)
        as i32;
}

pub unsafe extern "C" fn geno_clear_action(mut st: *mut GenoState) {
    melee_core::ct!("geno_clear_action");
    geno_zero(
        &raw mut (*st).ra_i as *mut i32 as *mut c_void,
        ::core::mem::size_of::<[i32; 64]>() as i32,
    );
    geno_zero(
        &raw mut (*st).ra_f as *mut f32 as *mut c_void,
        ::core::mem::size_of::<[f32; 64]>() as i32,
    );
    (*st).action_time = 0 as i32 as i32;
    (*st).nchecks = 0 as u32;
    (*st).last_check = -(1 as i32) as i32;
    geno_zero(
        &raw mut (*st).rehit_period as *mut i32 as *mut c_void,
        ::core::mem::size_of::<[i32; 4]>() as i32,
    );
    geno_zero(
        &raw mut (*st).rehit_count as *mut i32 as *mut c_void,
        ::core::mem::size_of::<[i32; 4]>() as i32,
    );
    geno_zero(
        &raw mut (*st).link_mode as *mut i32 as *mut c_void,
        ::core::mem::size_of::<[i32; 4]>() as i32,
    );
    geno_zero(
        &raw mut (*st).stun_add as *mut i32 as *mut c_void,
        ::core::mem::size_of::<[i32; 4]>() as i32,
    );
    geno_zero(
        &raw mut (*st).hb_flags as *mut i32 as *mut c_void,
        ::core::mem::size_of::<[i32; 4]>() as i32,
    );
    (*st).ledge = -(1 as i32) as i32;
    (*st).motion_started = 0 as i32 as i32;
    (*st).motion_vy = 0.0f32 as f32;
    (*st).motion_land = 0 as i32 as i32;
    (*st).motion_facing = 0.0f32 as f32;
    (*st).motion_gravity = -1.0f32 as f32;
}

pub unsafe extern "C" fn Geno_FighterReset(mut fp: *mut Fighter) {
    melee_core::ct!("Geno_FighterReset");

    let mut st: *mut GenoState = geno_state(fp);
    let mut resets: u32 = (*st).resets;
    if (*fp).kind as i32 >= Ft_Kind_Mex0 as i32 {
        Geno_DefineVoiceState(
            (*fp).kind as i32,
            fp as u32,
            -(1 as i32),
        );
    }
    geno_zero(
        st as *mut c_void,
        ::core::mem::size_of::<GenoState>() as i32,
    );
    (*st).resets = resets.wrapping_add(1 as u32);
    (*st).kind = (*fp).kind as i32;
    (*st).last_check = -(1 as i32) as i32;
    (*st).profile = Geno_ProfileForKind((*fp).kind as i32) as i32;
    (*st).hold_motion = -(1 as i32) as i32;
    (*st).hold_frames = -(1 as i32) as i32;
    (*st).ledge = -(1 as i32) as i32;
    (*st).enter_from = -(1 as i32) as i32;
    (*st).hit_port = -(1 as i32) as i32;
    geno_zero(
        geno_lua_block(fp) as *mut c_void,
        ::core::mem::size_of::<GenoLuaBlock>() as i32,
    );
    geno_combo_reset(fp, st);
    if (*st).profile >= 0 as i32 {
        Geno_Event(
            0 as i32,
            (*fp).kind as i32,
            (*fp).player_id as i32,
            (*st).profile as i32,
            0 as i32,
        );
        geno_install_overlays(fp, (*st).profile as i32);
        geno_v2_build(fp, (*st).profile as i32);
        if Geno_ArticleCount((*st).profile as i32)
            > 0 as i32
        {
            geno_art_prepare((*st).profile as i32);
        }
        if !(*fp).gobj.is_null() {
            geno_run_event(
                (*fp).gobj as *mut Fighter_GObj,
                st,
                GENO_EV_INIT as i32,
            );
        }
    }
}

pub unsafe extern "C" fn GenoGame_LabReload() -> i32 {
    melee_core::ct!("GenoGame_LabReload");
    let mut cur: *mut HSD_GObj = ::core::ptr::null_mut::<HSD_GObj>();
    let mut n: i32 = 0 as i32;
    if HSD_GObjPLinkHead.is_null() {
        return 0 as i32;
    }
    cur = *HSD_GObjPLinkHead.offset(HSD_GOBJ_PLINK_FIGHTER as isize);
    while !cur.is_null() {
        let mut fp: *mut Fighter = HSD_GObjGetUserData(cur) as *mut Fighter;
        let mut st: *mut GenoState = ::core::ptr::null_mut::<GenoState>();
        if !fp.is_null() {
            st = geno_state(fp);
            (*st).profile = Geno_ProfileForKind((*fp).kind as i32) as i32;
            if !((*st).profile < 0 as i32) {
                geno_pool_sync();
                geno_install_overlays(fp, (*st).profile as i32);
                geno_v2_build(fp, (*st).profile as i32);
                if Geno_ArticleCount((*st).profile as i32)
                    > 0 as i32
                {
                    geno_art_prepare((*st).profile as i32);
                }
                ftCo_800D105C(cur as *mut Fighter_GObj);
                n += 1;
            }
        }
        cur = (*cur).next as *mut HSD_GObj;
    }
    return n;
}

pub unsafe extern "C" fn Geno_OnActionChange(mut gobj: *mut Fighter_GObj) {
    melee_core::ct!("Geno_OnActionChange");
    if ProfActive() != 0 {
        ProfBegin(
            GW_PROF_GENO_HOOK as i32,
            ((*(HSD_GObjGetUserData(gobj as *mut HSD_GObj) as *mut Fighter)).player_id
                as i32) << 16 as i32
                | (*(HSD_GObjGetUserData(gobj as *mut HSD_GObj) as *mut Fighter)).kind
                    as i32,
        );
    }
    let mut st: *mut GenoState = geno_state(
        HSD_GObjGetUserData(gobj as *mut HSD_GObj) as *mut Fighter,
    );
    if geno_inert(st) != 0 {
        ProfEnd();
        return;
    }
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj as *mut HSD_GObj)
        as *mut Fighter;
    if (*fp).kind as i32 >= Ft_Kind_Mex0 as i32 {
        Geno_DefineVoiceState(
            (*fp).kind as i32,
            fp as u32,
            (*fp).motion_id as i32,
        );
    }
    geno_clear_action(st);
    (*st).atk_connected_prev = (*st).atk_connected;
    (*st).atk_connected = 0 as i32 as i32;
    (*st).fall_limit = 0.0f32 as f32;
    geno_combo_action(HSD_GObjGetUserData(gobj as *mut HSD_GObj) as *mut Fighter, st);
    if (*(HSD_GObjGetUserData(gobj as *mut HSD_GObj) as *mut Fighter)).motion_id
        < GENO_MOTION_BASE
    {
        if (*(HSD_GObjGetUserData(gobj as *mut HSD_GObj) as *mut Fighter)).motion_id
            != ftCo_MS_FallSpecial as i32
            && (*(HSD_GObjGetUserData(gobj as *mut HSD_GObj) as *mut Fighter)).motion_id
                != ftCo_MS_FallSpecialF as i32
            && (*(HSD_GObjGetUserData(gobj as *mut HSD_GObj) as *mut Fighter)).motion_id
                != ftCo_MS_FallSpecialB as i32
        {
            (*st).fall_ledge = 0 as i32 as i32;
        }
        (*st).hidden = 0 as i32 as i32;
    }
    if (*st).profile >= 0 as i32 {
        geno_run_event(gobj, st, GENO_EV_ACTION as i32);
    }
    ProfEnd();
}

pub unsafe extern "C" fn Geno_OnFrame(mut gobj: *mut Fighter_GObj) {
    melee_core::ct!("Geno_OnFrame");
    if ProfActive() != 0 {
        ProfBegin(
            GW_PROF_GENO_HOOK as i32,
            ((*(HSD_GObjGetUserData(gobj as *mut HSD_GObj) as *mut Fighter)).player_id
                as i32) << 16 as i32
                | (*(HSD_GObjGetUserData(gobj as *mut HSD_GObj) as *mut Fighter)).kind
                    as i32,
        );
    }
    let mut st: *mut GenoState = geno_state(
        HSD_GObjGetUserData(gobj as *mut HSD_GObj) as *mut Fighter,
    );
    if (*st).profile < 0 as i32 {
        ProfEnd();
        return;
    }
    geno_run_event(gobj, st, GENO_EV_FRAME as i32);
    ProfEnd();
}

pub unsafe extern "C" fn GenoGame_AttrFind(
    mut name: *const i8,
) -> i32 {
    melee_core::ct!("GenoGame_AttrFind");
    let mut i: i32 = 0;
    i = 0 as i32;
    while i < GENO_NATTRS {
        if geno_streq(name, geno_attrs[i as usize].name) != 0 {
            return i;
        }
        i += 1;
    }
    return -(1 as i32);
}

pub unsafe extern "C" fn GenoGame_AttrIsInt(
    mut index: i32,
) -> i32 {
    melee_core::ct!("GenoGame_AttrIsInt");
    return if index >= 0 as i32 && index < GENO_NATTRS {
        geno_attrs[index as usize].is_int
    } else {
        0 as i32
    };
}

pub unsafe extern "C" fn GenoGame_AttrOffset(
    mut index: i32,
) -> i32 {
    melee_core::ct!("GenoGame_AttrOffset");
    return if index >= 0 as i32 && index < GENO_NATTRS {
        geno_attrs[index as usize].offset
    } else {
        -(1 as i32)
    };
}

pub unsafe extern "C" fn geno_apply_special(
    mut fp: *mut Fighter,
    mut p: i32,
) {
    melee_core::ct!("geno_apply_special");
    let mut n: i32 = Geno_SpecialCount(p);
    let mut i: i32 = 0;
    let mut src: *mut u32 = ::core::ptr::null_mut::<u32>();
    let mut dst: *mut u32 = ::core::ptr::null_mut::<u32>();
    if n <= 0 as i32 {
        return;
    }
    src = if !(*fp).ft_data.is_null() {
        (*(*fp).ft_data).ext_attr as *mut u32
    } else {
        ::core::ptr::null_mut::<u32>()
    };
    dst = (*fp).dat_attrs as *mut u32;
    if dst != (*fp).dat_attrs_backup as *mut u32 {
        dst = ::core::ptr::null_mut::<u32>();
    }
    i = 0 as i32;
    while i < n {
        let mut idx: i32 = Geno_SpecialIndex(p, i);
        let mut bits: u32 = Geno_SpecialBits(p, i) as u32;
        if !(idx < 0 as i32 || idx >= GENO_SPECIAL_WORDS) {
            if !src.is_null() {
                *src.offset(idx as isize) = bits;
            }
            if !dst.is_null() && dst != src {
                *dst.offset(idx as isize) = bits;
            }
        }
        i += 1;
    }
    Geno_Event(
        12 as i32,
        (*fp).kind as i32,
        (*fp).player_id as i32,
        n,
        Geno_SpecialIndex(p, 0 as i32),
    );
}

pub unsafe extern "C" fn Geno_ApplyAttrs(mut fp: *mut Fighter) {
    melee_core::ct!("Geno_ApplyAttrs");
    let mut p: i32 = Geno_ProfileForKind(
        (*fp).kind as i32,
    );
    let mut n: i32 = 0;
    let mut i: i32 = 0;
    let mut mj: i32 = 0;
    if p < 0 as i32 {
        return;
    }
    n = Geno_AttrCount(p);
    i = 0 as i32;
    while i < n {
        let mut idx: i32 = Geno_AttrIndex(p, i);
        if idx >= 0 as i32 && idx < GENO_NATTRS {
            *((&raw mut (*fp).co_attrs as *mut u8)
                .offset(geno_attrs[idx as usize].offset as isize) as *mut u32) = Geno_AttrBits(
                p,
                i,
            ) as u32;
        }
        i += 1;
    }
    mj = Geno_MaxJumps(p);
    if mj > 0 as i32 {
        (*fp).co_attrs.max_jumps = mj;
    }
    Geno_Event(
        5 as i32,
        (*fp).kind as i32,
        (*fp).player_id as i32,
        n,
        (*fp).co_attrs.max_jumps,
    );
    geno_apply_special(fp, p);
}

pub unsafe extern "C" fn Geno_FighterScale(
    mut fp: *mut Fighter,
) -> f32 {
    melee_core::ct!("Geno_FighterScale");
    let mut p: i32 = Geno_ProfileForKind(
        (*fp).kind as i32,
    );
    return if p < 0 as i32 { 1.0f32 } else { Geno_Scale(p) };
}

pub unsafe extern "C" fn Geno_MultiJump(
    mut fp: *mut Fighter,
    mut first_state: i32,
    mut msid: *mut i32,
    mut vy: *mut f32,
) {
    melee_core::ct!("Geno_MultiJump");
    let mut p: i32 = Geno_ProfileForKind(
        (*fp).kind as i32,
    );
    let mut n: i32 = (*fp).x1968_jumpsUsed as i32
        - 1 as i32;
    let mut rows: i32 = 0;
    let mut nvy: i32 = 0;
    let mut bits: Anon_anon_1941 = unsafe { core::mem::zeroed() };
    if p < 0 as i32 || (*fp).x2D0.is_null() || n < 0 as i32
    {
        return;
    }
    rows = (*(*fp).x2D0).x28;
    if rows < 1 as i32 || rows > 5 as i32 {
        rows = 5 as i32;
    }
    nvy = Geno_JumpVyCount(p);
    if n >= rows - 1 as i32 && rows > 1 as i32
        && ((*fp).x1968_jumpsUsed as i32 + 1 as i32)
            < (*fp).co_attrs.max_jumps
    {
        *msid = first_state + rows - 2 as i32;
    }
    *vy = (*(*fp).x2D0)
        .x14[(if n < rows { n } else { rows - 1 as i32 }) as usize];
    if n >= rows {
        let mut st: *mut GenoState = geno_state(fp);
        if (*fp).x1968_jumpsUsed as i32 + 1 as i32
            >= (*fp).co_attrs.max_jumps
        {
            *msid = first_state + rows - 1 as i32;
        }
        (*st).extra_jumps = (*st).extra_jumps.wrapping_add(1);
        Geno_Event(
            1 as i32,
            (*fp).kind as i32,
            (*fp).player_id as i32,
            n + 1 as i32,
            (*fp).co_attrs.max_jumps - 1 as i32,
        );
    } else {
        Geno_Event(
            6 as i32,
            (*fp).kind as i32,
            (*fp).player_id as i32,
            n + 1 as i32,
            (*fp).co_attrs.max_jumps - 1 as i32,
        );
    }
    if nvy > 0 as i32 {
        bits.u = Geno_JumpVyBits(
            p,
            if n < nvy { n } else { nvy - 1 as i32 },
        ) as u32;
        *vy = bits.f as f32;
    }
}

pub unsafe extern "C" fn geno_var_is_float(
    mut ref_0: i32,
) -> i32 {
    melee_core::ct!("geno_var_is_float");
    return (ref_0 >> 6 as i32 & 3 as i32
        >= GENO_BANK_LA_FLOAT) as i32;
}

pub unsafe extern "C" fn geno_var(
    mut st: *mut GenoState,
    mut ref_0: i32,
) -> *mut c_void {
    melee_core::ct!("geno_var");
    let mut i: i32 = ref_0 & GENO_VARS_PER_BANK - 1 as i32;
    match ref_0 >> 6 as i32 & 3 as i32 {
        GENO_BANK_LA_INT => {
            return (&raw mut (*st).la_i as *mut i32).offset(i as isize) as *mut i32
                as *mut c_void;
        }
        GENO_BANK_RA_INT => {
            return (&raw mut (*st).ra_i as *mut i32).offset(i as isize) as *mut i32
                as *mut c_void;
        }
        GENO_BANK_LA_FLOAT => {
            return (&raw mut (*st).la_f as *mut f32).offset(i as isize) as *mut f32
                as *mut c_void;
        }
        _ => {
            return (&raw mut (*st).ra_f as *mut f32).offset(i as isize) as *mut f32
                as *mut c_void;
        }
    };
}

pub unsafe extern "C" fn geno_var_as(
    mut st: *mut GenoState,
    mut ref_0: i32,
    mut want_f: i32,
) -> GenoWord {
    melee_core::ct!("geno_var_as");
    let mut r: GenoWord = unsafe { core::mem::zeroed() };
    let mut pb: *mut c_void = geno_var(st, ref_0);
    if geno_var_is_float(ref_0) != 0 {
        if want_f != 0 {
            r.f = *(pb as *mut f32);
        } else {
            r.i = *(pb as *mut f32) as i32;
        }
    } else if want_f != 0 {
        r.f = *(pb as *mut i32) as f32;
    } else {
        r.i = *(pb as *mut i32);
    }
    return r;
}

pub unsafe extern "C" fn geno_operand(
    mut st: *mut GenoState,
    mut want_f: i32,
    mut b_is_var: i32,
    mut word: u32,
) -> GenoWord {
    melee_core::ct!("geno_operand");
    let mut r: GenoWord = unsafe { core::mem::zeroed() };
    if b_is_var != 0 {
        return geno_var_as(st, (word & 0xff as u32) as i32, want_f);
    }
    r.u = word;
    return r;
}

pub unsafe extern "C" fn geno_operand_b(
    mut st: *mut GenoState,
    mut a: i32,
    mut w0: u32,
    mut w1: u32,
) -> GenoWord {
    melee_core::ct!("geno_operand_b");
    return geno_operand(
        st,
        geno_var_is_float(a),
        (w0 & 0x80 as u32 != 0 as u32) as i32,
        w1,
    );
}

pub unsafe extern "C" fn geno_cmp(
    mut is_f: i32,
    mut x: GenoWord,
    mut cmp: i32,
    mut y: GenoWord,
) -> i32 {
    melee_core::ct!("geno_cmp");
    if cmp == GENO_CMP_BIT as i32
        || cmp == GENO_CMP_NOBIT as i32
    {
        let mut v: i32 = if is_f != 0 { x.f as i32 } else { x.i };
        let mut bit: i32 = if is_f != 0 { y.f as i32 } else { y.i };
        let mut set: i32 = v as i32
            >> (bit as i32 & 31 as i32)
            & 1 as i32;
        return if cmp == GENO_CMP_BIT as i32 {
            set
        } else {
            (set == 0) as i32
        };
    }
    if is_f != 0 {
        match cmp {
            0 => return (x.f == y.f) as i32,
            1 => return (x.f != y.f) as i32,
            2 => return (x.f < y.f) as i32,
            3 => return (x.f <= y.f) as i32,
            4 => return (x.f > y.f) as i32,
            _ => return (x.f >= y.f) as i32,
        }
    }
    match cmp {
        0 => return (x.i == y.i) as i32,
        1 => return (x.i != y.i) as i32,
        2 => return (x.i < y.i) as i32,
        3 => return (x.i <= y.i) as i32,
        4 => return (x.i > y.i) as i32,
        _ => return (x.i >= y.i) as i32,
    };
}

pub unsafe extern "C" fn geno_compare(
    mut st: *mut GenoState,
    mut a: i32,
    mut cmp: i32,
    mut b: GenoWord,
) -> i32 {
    melee_core::ct!("geno_compare");
    let mut is_f: i32 = geno_var_is_float(a);
    return geno_cmp(is_f, geno_var_as(st, a, is_f), cmp, b);
}

pub unsafe extern "C" fn geno_arith(
    mut st: *mut GenoState,
    mut sub: i32,
    mut a: i32,
    mut b: GenoWord,
) {
    melee_core::ct!("geno_arith");
    let mut pa: *mut c_void = geno_var(st, a);
    if geno_var_is_float(a) != 0 {
        let mut x: *mut f32 = pa as *mut f32;
        match sub {
            1 => {
                *x = b.f;
            }
            2 => {
                *x += b.f;
            }
            3 => {
                *x -= b.f;
            }
            4 => {
                *x *= b.f;
            }
            7 => {
                if b.f != 0.0f32 {
                    *x /= b.f;
                }
            }
            10 => {
                *x = HSD_Randf() * b.f;
            }
            _ => {}
        }
    } else {
        let mut x_0: *mut i32 = pa as *mut i32;
        match sub {
            1 => {
                *x_0 = b.i;
            }
            2 => {
                *x_0 += b.i;
            }
            3 => {
                *x_0 -= b.i;
            }
            4 => {
                *x_0 *= b.i;
            }
            5 => {
                *x_0
                    |= ((1 as u32)
                        << (b.i as i32 & 31 as i32))
                        as i32;
            }
            6 => {
                *x_0
                    &= !(((1 as u32)
                        << (b.i as i32 & 31 as i32))
                        as i32);
            }
            7 => {
                if b.i != 0 as i32
                    && !(b.i == -(1 as i32)
                        && *x_0 == 0x80000000 as u32 as i32)
                {
                    *x_0 /= b.i;
                }
            }
            10 => {
                *x_0 = (if b.i > 0 as i32 {
                    HSD_Randi(b.i) as i32
                } else {
                    0 as i32
                }) as i32;
            }
            _ => {}
        }
    };
}

pub unsafe extern "C" fn geno_buttons(
    mut fp: *mut Fighter,
    mut pressed: i32,
) -> u32 {
    melee_core::ct!("geno_buttons");
    let mut b: HSD_Pad = if pressed != 0 { (*fp).input.pressed_buttons } else { (*fp).input.held_buttons[0] };
    let mut m: u32 = 0 as u32;
    if b & HSD_PAD_A as HSD_Pad != 0 {
        m |= GENO_BTN_ATTACK as u32;
    }
    if b & HSD_PAD_B as HSD_Pad != 0 {
        m |= GENO_BTN_SPECIAL as u32;
    }
    if b & HSD_PAD_XY as HSD_Pad != 0 {
        m |= GENO_BTN_JUMP as u32;
    }
    if b & (HSD_PAD_L | HSD_PAD_R | HSD_PAD_LR as i32) as HSD_Pad != 0 {
        m |= GENO_BTN_SHIELD as u32;
    }
    if pressed == 0 && !p_ftCommonData.is_null()
        && (*fp).input.triggers[0 as i32 as usize]
            >= (*p_ftCommonData).shield_press_threshold
    {
        m |= GENO_BTN_SHIELD as u32;
    }
    if b & HSD_PAD_Z as HSD_Pad != 0 {
        m |= GENO_BTN_GRAB as u32;
    }
    if b & HSD_PAD_DPADUP as HSD_Pad != 0 {
        m |= GENO_BTN_TAUNT as u32;
    }
    if b & HSD_PAD_DPADDOWN as HSD_Pad != 0 {
        m |= GENO_BTN_BLAST as u32;
    }
    return m;
}

pub unsafe extern "C" fn geno_val_is_float(mut id: u32) -> i32 {
    melee_core::ct!("geno_val_is_float");
    if id >= GENO_VAL_SPECIAL_I as i32 as u32 {
        return 0 as i32;
    }
    if id >= GENO_VAL_SPECIAL_F as i32 as u32 {
        return 1 as i32;
    }
    match id {
        0 | 14 | 15 | 17 | 18 | 19 | 20 | 23 | 24 | 25 | 26 | 28 | 30 | 48 | 49 | 54 | 55
        | 56 | 57 | 59 | 60 => return 0 as i32,
        _ => {
            if id >= GENO_VAL_MOVE_I0 as i32 as u32
                && id <= GENO_VAL_MOVE_I7 as i32 as u32
            {
                return 0 as i32;
            }
            return 1 as i32;
        }
    };
}

pub unsafe extern "C" fn geno_val_get(
    mut fp: *mut Fighter,
    mut st: *mut GenoState,
    mut id: u32,
) -> GenoWord {
    melee_core::ct!("geno_val_get");
    let mut r: GenoWord = unsafe { core::mem::zeroed() };
    r.u = 0 as u32;
    if id >= GENO_VAL_SPECIAL_F as i32 as u32
        && id < (GENO_VAL_SPECIAL_I as i32 + GENO_SPECIAL_WORDS) as u32
    {
        let mut idx: u32 = if id >= GENO_VAL_SPECIAL_I as i32 as u32 {
            id.wrapping_sub(GENO_VAL_SPECIAL_I as i32 as u32)
        } else {
            id.wrapping_sub(GENO_VAL_SPECIAL_F as i32 as u32)
        };
        if idx < GENO_SPECIAL_WORDS as u32 && !(*fp).dat_attrs.is_null() {
            r.u = *((*fp).dat_attrs as *mut u32).offset(idx as isize);
        }
        return r;
    }
    match id {
        0 => {
            r.i = ((*fp).ground_or_air as i32
                == GA_Air as i32) as i32 as i32;
        }
        1 => {
            r.f = (*fp).facing_dir as f32;
        }
        2 => {
            r.f = (*fp).self_vel.x;
        }
        3 => {
            r.f = (*fp).self_vel.y;
        }
        4 => {
            r.f = (*fp).gr_vel as f32;
        }
        5 => {
            r.f = ((*fp).self_vel.x as f32 * (*fp).facing_dir) as f32;
        }
        6 => {
            r.f = (*fp).x8c_kb_vel.x;
        }
        7 => {
            r.f = (*fp).x8c_kb_vel.y;
        }
        8 => {
            r.f = (*fp).input.lstick[0 as i32 as usize].x;
        }
        9 => {
            r.f = (*fp).input.lstick[0 as i32 as usize].y;
        }
        10 => {
            r.f = ((*fp).input.lstick[0 as i32 as usize].x
                as f32 * (*fp).facing_dir) as f32;
        }
        61 => {
            r.f = sqrtf(fmadds((*fp).input.lstick[0].x, (*fp).input.lstick[0].x, (*fp).input.lstick[0].y * (*fp).input.lstick[0].y));
        }
        11 => {
            r.f = (*fp).input.cstick[0 as i32 as usize].x;
        }
        12 => {
            r.f = (*fp).input.cstick[0 as i32 as usize].y;
        }
        13 => {
            r.f = (*fp).cur_anim_frame as f32;
        }
        14 => {
            r.i = (*st).action_time;
        }
        15 => {
            r.i = (*fp).motion_id;
        }
        16 => {
            r.f = (*fp).dmg.x1830_percent as f32;
        }
        17 => {
            r.i = (*fp).x1968_jumpsUsed as i32;
        }
        18 => {
            r.i = (*fp).co_attrs.max_jumps as i32;
        }
        19 => {
            r.i = geno_buttons(fp, 0 as i32) as i32;
        }
        20 => {
            r.i = geno_buttons(fp, 1 as i32) as i32;
        }
        21 => {
            r.f = (*fp).cur_pos.x;
        }
        22 => {
            r.f = (*fp).cur_pos.y;
        }
        23 | 24 | 25 | 26 => {
            r.i = (*fp)
                .cmd_vars[id
                .wrapping_sub(GENO_VAL_CMD_VAR0 as i32 as u32) as usize]
                as i32;
        }
        27 => {
            r.f = (*fp).frame_speed_mul as f32;
        }
        28 => {
            r.i = (if (*fp).fall_fast() as i32 != 0 {
                1 as i32
            } else {
                0 as i32
            }) as i32;
        }
        29 => {
            r.f = (*fp).input.triggers[0 as i32 as usize] as f32;
        }
        30 => {
            r.i = geno_cur_state(fp, st) as i32;
        }
        48 => {
            r.i = (*st).ledge;
        }
        49 => {
            r.i = (*st).hidden;
        }
        50 => {
            r.f = (if (*fp).anon_594.anon_0.x594_b0() as i32
                != 0
            {
                (*fp).x6A4_transNOffset.z as f32
            } else {
                0.0f32
            }) as f32;
        }
        51 => {
            r.f = (if (*fp).anon_594.anon_0.x594_b0() as i32
                != 0
            {
                (*fp).x6A4_transNOffset.y as f32
            } else {
                0.0f32
            }) as f32;
        }
        52 => {
            r.f = (*st).motion_gravity;
        }
        53 => {
            r.f = (*st).hit_damage;
        }
        54 => {
            r.i = (*st).hit_port;
        }
        55 => {
            r.i = (*st).hit_counter;
        }
        56 => {
            r.i = (*st).hit_count;
        }
        57 => {
            r.i = geno_art_live((*fp).gobj, -(1 as i32)) as i32;
        }
        59 => {
            r.i = (*st).atk_connected;
        }
        60 => {
            r.i = (*st).atk_connected_prev;
        }
        62 => {
            r.f = (*st).fall_limit;
        }
        _ => {
            if id >= GENO_VAL_MOVE_F0 as i32 as u32
                && id <= GENO_VAL_MOVE_F7 as i32 as u32
            {
                r.f = (*st)
                    .move_f[id
                    .wrapping_sub(GENO_VAL_MOVE_F0 as i32 as u32)
                    as usize];
            } else if id >= GENO_VAL_MOVE_I0 as i32 as u32
                && id <= GENO_VAL_MOVE_I7 as i32 as u32
            {
                r.i = (*st)
                    .move_i[id
                    .wrapping_sub(GENO_VAL_MOVE_I0 as i32 as u32)
                    as usize];
            }
        }
    }
    return r;
}

pub unsafe extern "C" fn geno_val_put(
    mut fp: *mut Fighter,
    mut id: u32,
    mut v: GenoWord,
) -> i32 {
    melee_core::ct!("geno_val_put");
    if id >= GENO_VAL_MOVE_F0 as i32 as u32
        && id <= GENO_VAL_MOVE_F7 as i32 as u32
    {
        (*geno_state(fp))
            .move_f[id.wrapping_sub(GENO_VAL_MOVE_F0 as i32 as u32)
            as usize] = v.f;
        return 1 as i32;
    }
    if id >= GENO_VAL_MOVE_I0 as i32 as u32
        && id <= GENO_VAL_MOVE_I7 as i32 as u32
    {
        (*geno_state(fp))
            .move_i[id.wrapping_sub(GENO_VAL_MOVE_I0 as i32 as u32)
            as usize] = v.i;
        return 1 as i32;
    }
    match id {
        48 => {
            (*geno_state(fp)).ledge = (if v.i < -(1 as i32) {
                -(1 as i32)
            } else if v.i > 2 as i32 {
                2 as i32
            } else {
                v.i as i32
            }) as i32;
            return 1 as i32;
        }
        49 => {
            (*geno_state(fp)).hidden = (v.i != 0 as i32)
                as i32 as i32;
            (*fp)
                .set_x221E_b5(
                    (v.i != 0 as i32) as i32 as u8
                        as u8,
                );
            return 1 as i32;
        }
        52 => {
            (*geno_state(fp)).motion_gravity = (if v.f < 0.0f32 {
                -1.0f32
            } else {
                v.f as f32
            }) as f32;
            return 1 as i32;
        }
        0 => {
            if v.i != 0 as i32
                && (*fp).ground_or_air as i32
                    == GA_Ground as i32
            {
                ftCommon_8007D5D4(fp);
            }
            return 1 as i32;
        }
        1 => {
            if v.f == 0.0f32 {
                (*fp).facing_dir = -(*fp).facing_dir;
            } else {
                (*fp).facing_dir = if v.f < 0.0f32 { -1.0f32 } else { 1.0f32 };
            }
            if !(*fp).parts.is_null() {
                ftPartSetRotY(
                    fp,
                    0 as i32,
                    1.5707964f32 * (*fp).facing_dir as f32,
                );
            }
            return 1 as i32;
        }
        2 => {
            (*fp).self_vel.x = v.f;
            return 1 as i32;
        }
        3 => {
            (*fp).self_vel.y = v.f;
            return 1 as i32;
        }
        4 => {
            (*fp).gr_vel = v.f as f32;
            return 1 as i32;
        }
        5 => {
            (*fp).self_vel.x = (v.f as f32 * (*fp).facing_dir) as f32;
            return 1 as i32;
        }
        27 => {
            if v.f > 0.0f32 {
                if !(*fp).gobj.is_null()
                    && !(HSD_GObjGetHSDObj((*fp).gobj) as *mut HSD_JObj).is_null()
                {
                    ftAnim_8006F0FC(
                        (*fp).gobj as *mut Fighter_GObj,
                        v.f as f32,
                    );
                } else {
                    (*fp).frame_speed_mul = v.f as f32;
                }
            }
            return 1 as i32;
        }
        59 => {
            (*geno_state(fp)).atk_connected = (v.i != 0 as i32)
                as i32 as i32;
            return 1 as i32;
        }
        62 => {
            (*geno_state(fp)).fall_limit = (if v.f > 0.0f32 {
                v.f as f32
            } else {
                0.0f32
            }) as f32;
            return 1 as i32;
        }
        17 => {
            (*fp).x1968_jumpsUsed = (if v.i < 0 as i32 {
                0 as i32
            } else if v.i > 255 as i32 {
                255 as i32
            } else {
                v.i as i32
            }) as u8;
            return 1 as i32;
        }
        23 | 24 | 25 | 26 => {
            (*fp)
                .cmd_vars[id
                .wrapping_sub(GENO_VAL_CMD_VAR0 as i32 as u32)
                as usize] = v.i as u32;
            return 1 as i32;
        }
        _ => {
            Geno_Event(
                14 as i32,
                (*fp).kind as i32,
                (*fp).player_id as i32,
                id as i32,
                0 as i32,
            );
            return 0 as i32;
        }
    };
}

pub unsafe extern "C" fn GenoGame_TestCapture(
    mut on: i32,
    mut anim_end: i32,
) {
    melee_core::ct!("GenoGame_TestCapture");
    geno_test_capture = on as i32;
    geno_test_anim_end = anim_end as i32;
    geno_test_changes = 0 as i32 as i32;
    geno_test_target = 0 as u32;
}

pub unsafe extern "C" fn GenoGame_TestChanges() -> i32 {
    melee_core::ct!("GenoGame_TestChanges");
    return geno_test_changes as i32;
}

pub unsafe extern "C" fn GenoGame_TestLastTarget() -> u32 {
    melee_core::ct!("GenoGame_TestLastTarget");
    return geno_test_target;
}

pub unsafe extern "C" fn geno_target_motion(
    mut fp: *mut Fighter,
    mut target: u32,
) -> i32 {
    melee_core::ct!("geno_target_motion");
    let mut kind: u32 = target >> 28 as i32;
    let mut id: i32 = (target & 0xffff as u32) as i32;
    if kind == GENO_TGT_MOTION {
        return id;
    }
    if kind == GENO_TGT_SPECIAL {
        return (*fp).x18 + id;
    }
    if kind == GENO_TGT_GENO && geno_state_valid(geno_state(fp), id) != 0 {
        return GENO_MOTION_BASE + id;
    }
    return -(1 as i32);
}

pub unsafe extern "C" fn geno_target_ok(
    mut fp: *mut Fighter,
    mut target: u32,
) -> i32 {
    melee_core::ct!("geno_target_ok");
    return (target >> 28 as i32 != GENO_TGT_GENO
        || geno_state_valid(
            geno_state(fp),
            (target & 0xffff as u32) as i32,
        ) != 0) as i32;
}

pub unsafe extern "C" fn geno_do_change(
    mut gobj: *mut Fighter_GObj,
    mut fp: *mut Fighter,
    mut st: *mut GenoState,
    mut target: u32,
) -> i32 {
    melee_core::ct!("geno_do_change");
    let mut msid: i32 = 0;
    if target == GENO_TGT_ROLL {
        return if (*fp).ground_or_air as i32
            == GA_Ground as i32 && ftCo_8009917C(gobj) != 0
        {
            1 as i32
        } else {
            0 as i32
        };
    }
    if target == GENO_TGT_AUTO || target == GENO_TGT_HELPLESS {
        target = ((0 as u32 & 15 as u32)
            << 28 as i32
            | (if (*fp).ground_or_air as i32
                == GA_Ground as i32
            {
                ftCo_MS_Wait as i32
            } else {
                (if target == 0xfffffffe as u32 {
                    ftCo_MS_Fall as i32
                } else {
                    ftCo_MS_FallSpecial as i32
                })
            }) as u32 & 0xffff as u32) as u32;
    }
    if target >> 28 as i32 == GENO_TGT_GENO {
        let mut s: i32 = (target & 0xffff as u32) as i32;
        if geno_state_valid(st, s) == 0 {
            Geno_Event(
                8 as i32,
                (*fp).kind as i32,
                (*fp).player_id as i32,
                s,
                0 as i32,
            );
            return 0 as i32;
        }
        return geno_enter_state(gobj, fp, st, s, target);
    }
    msid = geno_target_motion(fp, target);
    if msid < 0 as i32
        || msid < (*fp).x18 && (*fp).x1C_actionStateList.is_null()
        || msid >= (*fp).x18
            && ((*fp).x20_actionStateList.is_null()
                || msid - (*fp).x18 > 0xff as i32)
    {
        return 0 as i32;
    }
    (*st).changes = (*st).changes.wrapping_add(1);
    Geno_Event(
        7 as i32,
        (*fp).kind as i32,
        (*fp).player_id as i32,
        (*fp).motion_id,
        target as i32,
    );
    if geno_test_capture != 0 {
        geno_test_target = target;
        geno_test_changes += 1;
        (*fp).motion_id = msid as FtMotionId;
        Geno_OnActionChange(gobj);
        return 1 as i32;
    }
    if target as u32 & GENO_TGT_KEEP_FRAME != 0 {
        Fighter_ChangeMotionState(
            gobj,
            msid as FtMotionId,
            Ft_MF_KeepGfx | Ft_MF_SkipHit | Ft_MF_SkipMatAnim | Ft_MF_KeepSfx
                | Ft_MF_SkipColAnim | Ft_MF_UpdateCmd | Ft_MF_SkipNametagVis
                | Ft_MF_KeepSwordTrail | Ft_MF_SkipItemVis | Ft_MF_SkipModelPartVis
                | Ft_MF_SkipAttackCount | Ft_MF_KeepFastFall,
            (*fp).cur_anim_frame as f32,
            1.0f32,
            0.0f32,
            ::core::ptr::null_mut::<Fighter_GObj>(),
        );
        return 1 as i32;
    }
    if target as u32 & GENO_TGT_RAW == 0 {
        match msid {
            14 => {
                if (*fp).ground_or_air as i32
                    == GA_Ground as i32
                {
                    ft_8008A2BC(gobj);
                } else {
                    ftCo_Fall_Enter(gobj);
                }
                return 1 as i32;
            }
            29 => {
                ftCo_Fall_Enter(gobj);
                return 1 as i32;
            }
            35 => {
                ftCo_800968C8(gobj);
                return 1 as i32;
            }
            42 => {
                if (*fp).ground_or_air as i32
                    == GA_Ground as i32
                {
                    ftCo_Landing_Enter_Basic(gobj);
                    return 1 as i32;
                }
            }
            43 => {
                if (*fp).ground_or_air as i32
                    == GA_Ground as i32
                {
                    ftCo_LandingFallSpecial_Enter_Basic(gobj);
                    return 1 as i32;
                }
            }
            _ => {}
        }
    }
    Fighter_ChangeMotionState(
        gobj,
        msid as FtMotionId,
        Ft_MF_None,
        0.0f32,
        1.0f32,
        0.0f32,
        ::core::ptr::null_mut::<Fighter_GObj>(),
    );
    return 1 as i32;
}

pub unsafe extern "C" fn geno_cond(
    mut gobj: *mut Fighter_GObj,
    mut fp: *mut Fighter,
    mut st: *mut GenoState,
    mut c: *const GenoCond,
) -> i32 {
    melee_core::ct!("geno_cond");
    let mut kind: i32 = ((*c).head >> 8 as i32
        & 0xff as u32) as i32;
    let mut b_is_var: i32 = ((*c).head & 0x80 as u32 != 0 as u32)
        as i32;
    let mut cmp: i32 = ((*c).head >> 4 as i32 & 7 as u32)
        as i32;
    let mut r: i32 = 0 as i32;
    match kind {
        0 => {
            r = 1 as i32;
        }
        1 => {
            r = if geno_test_capture != 0 {
                geno_test_anim_end as i32
            } else {
                (ftAnim_IsFramesRemaining(gobj) == 0) as i32
            };
        }
        2 => {
            r = ((*fp).ground_or_air as i32
                == GA_Ground as i32) as i32;
        }
        3 => {
            r = ((*fp).ground_or_air as i32
                == GA_Air as i32) as i32;
        }
        4 => {
            r = (geno_buttons(fp, 1 as i32) & (*c).arg1 != 0 as u32)
                as i32;
        }
        5 => {
            r = (geno_buttons(fp, 0 as i32) & (*c).arg1 != 0 as u32)
                as i32;
        }
        6 => {
            let mut v: GenoWord = geno_var_as(
                st,
                ((*c).arg1 & 0xff as u32) as i32,
                0 as i32,
            );
            r = v.i as i32 >> ((*c).arg2 & 31 as u32)
                & 1 as i32;
        }
        7 => {
            let mut a: i32 = ((*c).arg1 & 0xff as u32)
                as i32;
            let mut is_f: i32 = geno_var_is_float(a);
            r = geno_cmp(
                is_f,
                geno_var_as(st, a, is_f),
                cmp,
                geno_operand(st, is_f, b_is_var, (*c).arg2),
            );
        }
        8 => {
            r = ((*fp).cur_anim_frame >= (*c).arg1 as i32 as f32)
                as i32;
        }
        9 => {
            let mut is_f_0: i32 = geno_val_is_float((*c).arg1);
            r = geno_cmp(
                is_f_0,
                geno_val_get(fp, st, (*c).arg1),
                cmp,
                geno_operand(st, is_f_0, b_is_var, (*c).arg2),
            );
        }
        _ => {
            r = 0 as i32;
        }
    }
    return if (*c).head & GENO_CHG_NOT as u32 != 0 {
        (r == 0) as i32
    } else {
        r
    };
}

pub unsafe extern "C" fn geno_check_true(
    mut gobj: *mut Fighter_GObj,
    mut fp: *mut Fighter,
    mut st: *mut GenoState,
    mut k: *const GenoCheck,
    mut from: i32,
) -> i32 {
    melee_core::ct!("geno_check_true");
    let mut j: u32 = 0;
    j = from as u32;
    while j < (*k).ncond && j < GENO_CHECK_CONDS as u32 {
        if geno_cond(
            gobj,
            fp,
            st,
            (&raw const (*k).cond as *const GenoCond).offset(j as isize)
                as *const GenoCond,
        ) == 0
        {
            return 0 as i32;
        }
        j = j.wrapping_add(1);
    }
    return 1 as i32;
}

pub unsafe extern "C" fn geno_register_check(
    mut fp: *mut Fighter,
    mut st: *mut GenoState,
    mut head: u32,
    mut target: u32,
    mut a1: u32,
    mut a2: u32,
) {
    melee_core::ct!("geno_register_check");
    let mut i: u32 = 0;
    let mut once: u32 = if head & GENO_CHG_ONCE as u32 != 0 {
        1 as u32
    } else {
        0 as u32
    };
    let mut k: *mut GenoCheck = ::core::ptr::null_mut::<GenoCheck>();
    head &= (0xffff as u32 & !(GENO_CHG_ONCE as u32)) as u32;
    i = 0 as u32;
    while i < (*st).nchecks && i < GENO_MAX_CHECKS as u32 {
        k = (&raw mut (*st).checks as *mut GenoCheck).offset(i as isize)
            as *mut GenoCheck;
        if (*k).target == target && (*k).once == once
            && (*k).cond[0 as i32 as usize].head == head
            && (*k).cond[0 as i32 as usize].arg1 == a1
            && (*k).cond[0 as i32 as usize].arg2 == a2
        {
            (*st).last_check = i as i32;
            return;
        }
        i = i.wrapping_add(1);
    }
    if (*st).nchecks >= GENO_MAX_CHECKS as u32 {
        Geno_Event(
            15 as i32,
            (*fp).kind as i32,
            (*fp).player_id as i32,
            GENO_MAX_CHECKS,
            0 as i32,
        );
        (*st).last_check = -(1 as i32) as i32;
        return;
    }
    k = (&raw mut (*st).checks as *mut GenoCheck).offset((*st).nchecks as isize)
        as *mut GenoCheck;
    geno_zero(
        k as *mut c_void,
        ::core::mem::size_of::<GenoCheck>() as i32,
    );
    (*k).target = target;
    (*k).once = once;
    (*k).ncond = 1 as u32;
    (*k).cond[0 as i32 as usize].head = head;
    (*k).cond[0 as i32 as usize].arg1 = a1;
    (*k).cond[0 as i32 as usize].arg2 = a2;
    (*st).last_check = (*st).nchecks as i32;
    (*st).nchecks = (*st).nchecks.wrapping_add(1);
}

pub unsafe extern "C" fn geno_and_check(
    mut st: *mut GenoState,
    mut head: u32,
    mut a1: u32,
    mut a2: u32,
) {
    melee_core::ct!("geno_and_check");
    let mut k: *mut GenoCheck = ::core::ptr::null_mut::<GenoCheck>();
    let mut j: u32 = 0;
    if (*st).last_check < 0 as i32
        || (*st).last_check as u32 >= (*st).nchecks
    {
        return;
    }
    k = (&raw mut (*st).checks as *mut GenoCheck).offset((*st).last_check as isize)
        as *mut GenoCheck;
    head &= (0xffff as u32 & !(GENO_CHG_ONCE as u32)) as u32;
    j = 0 as u32;
    while j < (*k).ncond {
        if (*k).cond[j as usize].head == head && (*k).cond[j as usize].arg1 == a1
            && (*k).cond[j as usize].arg2 == a2
        {
            return;
        }
        j = j.wrapping_add(1);
    }
    if (*k).ncond >= GENO_CHECK_CONDS as u32 {
        return;
    }
    (*k).cond[(*k).ncond as usize].head = head;
    (*k).cond[(*k).ncond as usize].arg1 = a1;
    (*k).cond[(*k).ncond as usize].arg2 = a2;
    (*k).ncond = (*k).ncond.wrapping_add(1);
}

pub unsafe extern "C" fn geno_drop_once(mut st: *mut GenoState) {
    melee_core::ct!("geno_drop_once");
    let mut i: u32 = 0;
    let mut n: u32 = 0 as u32;
    i = 0 as u32;
    while i < (*st).nchecks && i < GENO_MAX_CHECKS as u32 {
        if (*st).checks[i as usize].once == 0 {
            if n != i {
                (*st).checks[n as usize] = (*st).checks[i as usize];
            }
            n = n.wrapping_add(1);
        }
        i = i.wrapping_add(1);
    }
    if n != (*st).nchecks {
        (*st).nchecks = n;
        (*st).last_check = -(1 as i32) as i32;
    }
}

pub unsafe extern "C" fn geno_rehit_tick(mut fp: *mut Fighter, mut st: *mut GenoState) {
    melee_core::ct!("geno_rehit_tick");
    let mut i: i32 = 0;
    i = 0 as i32;
    while i < GENO_MAX_REHIT {
        if (*st).rehit_period[i as usize] > 0 as i32
            && {
                (*st).rehit_count[i as usize] += 1;
                (*st).rehit_count[i as usize] >= (*st).rehit_period[i as usize]
            }
        {
            (*st).rehit_count[i as usize] = 0 as i32 as i32;
            lbColl_80008440(
                (&raw mut (*fp).x914 as *mut HitCapsule).offset(i as isize)
                    as *mut HitCapsule,
            );
            Geno_Event(
                10 as i32,
                (*fp).kind as i32,
                (*fp).player_id as i32,
                (1 as i32) << i,
                (*st).rehit_period[i as usize],
            );
        }
        i += 1;
    }
}

pub unsafe extern "C" fn Geno_PreAnim(
    mut gobj: *mut Fighter_GObj,
) -> i32 {
    melee_core::ct!("Geno_PreAnim");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj as *mut HSD_GObj)
        as *mut Fighter;
    let mut st: *mut GenoState = geno_state(fp);
    let mut i: u32 = 0;
    let mut n: u32 = 0;
    let mut hit: i32 = -(1 as i32);
    let mut target: u32 = 0 as u32;
    if geno_inert(st) != 0 {
        return 0 as i32;
    }
    (*st).action_time += 1;
    geno_rehit_tick(fp, st);
    n = (*st).nchecks;
    i = 0 as u32;
    while i < n && i < GENO_MAX_CHECKS as u32 {
        if geno_check_true(
            gobj,
            fp,
            st,
            (&raw mut (*st).checks as *mut GenoCheck).offset(i as isize)
                as *mut GenoCheck,
            0 as i32,
        ) != 0
        {
            if geno_target_ok(fp, (*st).checks[i as usize].target) == 0 {
                Geno_Event(
                    8 as i32,
                    (*fp).kind as i32,
                    (*fp).player_id as i32,
                    ((*st).checks[i as usize].target & 0xffff as u32)
                        as i32,
                    0 as i32,
                );
            } else {
                hit = i as i32;
                target = (*st).checks[i as usize].target;
                break;
            }
        }
        i = i.wrapping_add(1);
    }
    if n != 0 as u32 {
        geno_drop_once(st);
    }
    if hit >= 0 as i32 {
        return geno_do_change(gobj, fp, st, target);
    }
    return if (*st).profile >= 0 as i32 {
        geno_v2_preanim(gobj, fp, st)
    } else {
        0 as i32
    };
}

pub unsafe extern "C" fn Geno_CollBegin(mut gobj: *mut Fighter_GObj) {
    melee_core::ct!("Geno_CollBegin");
    let mut st: *mut GenoState = geno_state(
        HSD_GObjGetUserData(gobj as *mut HSD_GObj) as *mut Fighter,
    );
    if geno_inert(st) != 0 {
        return;
    }
    (*st).in_coll = 1 as u32;
    (*st).edge_pending = 0 as u32;
}

pub unsafe extern "C" fn Geno_CollEnd(mut gobj: *mut Fighter_GObj) {
    melee_core::ct!("Geno_CollEnd");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj as *mut HSD_GObj)
        as *mut Fighter;
    let mut st: *mut GenoState = geno_state(fp);
    if geno_inert(st) != 0 {
        return;
    }
    (*st).in_coll = 0 as u32;
    if (*st).edge_pending != 0 {
        (*st).edge_pending = 0 as u32;
        geno_do_change(gobj, fp, st, (*st).edge_target);
    }
}

pub unsafe extern "C" fn Geno_GroundEdge(
    mut fp: *mut Fighter,
    mut landing: i32,
) {
    melee_core::ct!("Geno_GroundEdge");
    let mut st: *mut GenoState = geno_state(fp);
    let mut gobj: *mut Fighter_GObj = (*fp).gobj as *mut Fighter_GObj;
    let mut i: u32 = 0;
    let mut want: i32 = if landing != 0 {
        GENO_COND_GROUND as i32
    } else {
        GENO_COND_AIR as i32
    };
    if geno_inert(st) != 0 || gobj.is_null() {
        return;
    }
    if landing != 0 && (*st).profile >= 0 as i32 {
        geno_run_event(gobj, st, GENO_EV_LAND as i32);
    }
    if (*st).in_coll == 0 || (*st).edge_pending != 0 {
        return;
    }
    i = 0 as u32;
    while i < (*st).nchecks && i < GENO_MAX_CHECKS as u32 {
        let mut k: *mut GenoCheck = (&raw mut (*st).checks as *mut GenoCheck)
            .offset(i as isize) as *mut GenoCheck;
        if (*k).cond[0 as i32 as usize].head >> 8 as i32
            & 0xff as u32 == want as u32
            && (*k).cond[0 as i32 as usize].head & GENO_CHG_NOT as u32
                == 0 && geno_target_ok(fp, (*k).target) != 0
            && geno_check_true(gobj, fp, st, k, 1 as i32) != 0
        {
            (*st).edge_pending = 1 as u32;
            (*st).edge_target = (*k).target;
            Geno_Event(
                9 as i32,
                (*fp).kind as i32,
                (*fp).player_id as i32,
                (*fp).motion_id,
                (*k).target as i32,
            );
            return;
        }
        i = i.wrapping_add(1);
    }
    if landing != 0 && (*st).profile >= 0 as i32 {
        let mut n: i32 = Geno_OnLandCount(
            (*st).profile as i32,
        );
        let mut j: i32 = 0;
        j = 0 as i32;
        while j < n {
            if geno_target_motion(
                fp,
                Geno_OnLandFrom((*st).profile as i32, j) as u32,
            ) == (*fp).motion_id
            {
                (*st).edge_pending = 1 as u32;
                (*st).edge_target = Geno_OnLandTo((*st).profile as i32, j)
                    as u32;
                Geno_Event(
                    9 as i32,
                    (*fp).kind as i32,
                    (*fp).player_id as i32,
                    (*fp).motion_id,
                    (*st).edge_target as i32,
                );
                return;
            }
            j += 1;
        }
    }
}

pub unsafe extern "C" fn Geno_Autolink(
    mut attacker: *mut Fighter,
    mut hit: *mut HitCapsule,
    mut dir: *mut f32,
    mut angle: *mut f32,
    mut kb: *mut f32,
) -> i32 {
    melee_core::ct!("Geno_Autolink");
    let mut st: *mut GenoState = ::core::ptr::null_mut::<GenoState>();
    let mut idx: i32 = 0;
    let mut mode: i32 = 0;
    let mut vx: f32 = 0.;
    let mut vy: f32 = 0.;
    let mut speed2: f32 = 0.;
    let mut deg: f32 = 0.;
    if attacker.is_null()
        || hit
            < (&raw mut (*attacker).x914 as *mut HitCapsule)
                .offset(0 as i32 as isize) as *mut HitCapsule
        || hit
            >= (&raw mut (*attacker).x914 as *mut HitCapsule)
                .offset(4 as i32 as isize) as *mut HitCapsule
    {
        return 0 as i32;
    }
    st = geno_state(attacker);
    if geno_inert(st) != 0 {
        return 0 as i32;
    }
    (*st).atk_connected = 1 as i32 as i32;
    idx = hit
        .offset_from(
            (&raw mut (*attacker).x914 as *mut HitCapsule)
                .offset(0 as i32 as isize) as *mut HitCapsule,
        ) as i64 as i32;
    mode = (*st).link_mode[idx as usize] as i32;
    if mode == GENO_LINK_OFF as i32 {
        return 0 as i32;
    }
    if (*attacker).ground_or_air as i32 == GA_Ground as i32
    {
        vx = (*attacker).gr_vel;
        vy = 0.0f32;
    } else {
        vx = (*attacker).self_vel.x as f32;
        vy = (*attacker).self_vel.y as f32;
    }
    speed2 = fmadds(vx, vx, vy * vy);
    if speed2 < 0.05f32 * 0.05f32 {
        return 0 as i32;
    }
    *dir = if vx > 0.0f32 { -1.0f32 } else { 1.0f32 };
    deg = atan2f(vy, (if vx < 0.0f32 { -vx } else { vx })) * 57.29578f32;
    if deg < 0.0f32 {
        deg += 360.0f32;
    }
    *angle = (deg + 0.5f32) as i32 as f32;
    if *angle >= 360.0f32 {
        *angle -= 360.0f32;
    }
    if mode == GENO_LINK_SPEED as i32 && !p_ftCommonData.is_null()
        && (*p_ftCommonData).x100 > 0.0f32
    {
        let mut need: f32 = sqrtf(speed2) / (*p_ftCommonData).x100;
        if *kb < need {
            *kb = need;
        }
    }
    Geno_Event(
        11 as i32,
        (*attacker).kind as i32,
        (*attacker).player_id as i32,
        *angle as i32,
        *kb as i32,
    );
    return 1 as i32;
}

pub unsafe extern "C" fn Geno_FtCmd(
    mut gobj: *mut Fighter_GObj,
    mut cmd: *mut CommandInfo,
    mut mode: i32,
) {
    melee_core::ct!("Geno_FtCmd");
    if ProfActive() != 0 {
        ProfBegin(
            GW_PROF_GENO_HOOK as i32,
            ((*(HSD_GObjGetUserData(gobj as *mut HSD_GObj) as *mut Fighter)).player_id
                as i32) << 16 as i32
                | (*(HSD_GObjGetUserData(gobj as *mut HSD_GObj) as *mut Fighter)).kind
                    as i32,
        );
    }
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj as *mut HSD_GObj)
        as *mut Fighter;
    let mut st: *mut GenoState = geno_state(fp);
    let mut w: *mut u32 = (*cmd).anon_8.u as *mut u32;
    let mut w0: u32 = u32::from_be(*w.offset(0 as i32 as isize));
    let mut sub: i32 = (w0 >> 20 as i32 & 63 as u32)
        as i32;
    let mut len: i32 = (w0 >> 16 as i32 & 15 as u32)
        as i32;
    let mut a: i32 = (w0 >> 8 as i32 & 0xff as u32)
        as i32;
    let mut skip: u32 = 0 as u32;
    if len == 0 as i32 {
        len = 1 as i32;
    }
    (*st).flags |= GENO_SF_SCRIPT;
    match sub {
        0 => {}
        1 | 2 | 3 | 4 | 5 | 6 | 7 => {
            geno_arith(
                st,
                sub,
                a,
                geno_operand_b(
                    st,
                    a,
                    w0,
                    if len > 1 as i32 {
                        u32::from_be(*w.offset(1 as i32 as isize))
                    } else {
                        0 as u32
                    },
                ),
            );
        }
        10 => {
            if mode != GENO_MODE_SKIP {
                geno_arith(
                    st,
                    sub,
                    a,
                    geno_operand_b(
                        st,
                        a,
                        w0,
                        if len > 1 as i32 {
                            u32::from_be(*w.offset(1 as i32 as isize))
                        } else {
                            0 as u32
                        },
                    ),
                );
            }
        }
        8 => {
            let mut id: u32 = if len > 1 as i32 {
                u32::from_be(*w.offset(1 as i32 as isize))
            } else {
                0 as u32
            };
            let mut is_f: i32 = geno_val_is_float(id);
            let mut v: GenoWord = geno_val_get(fp, st, id);
            let mut pa: *mut c_void = geno_var(st, a);
            if geno_var_is_float(a) != 0 {
                *(pa as *mut f32) = if is_f != 0 { v.f } else { v.i as f32 };
            } else {
                *(pa as *mut i32) = if is_f != 0 { v.f as i32 } else { v.i };
            }
        }
        9 => {
            let mut id_0: u32 = if len > 1 as i32 {
                u32::from_be(*w.offset(1 as i32 as isize))
            } else {
                0 as u32
            };
            geno_val_put(
                fp,
                id_0,
                geno_operand(
                    st,
                    geno_val_is_float(id_0),
                    (w0 & 0x80 as u32 != 0 as u32) as i32,
                    if len > 2 as i32 {
                        u32::from_be(*w.offset(2 as i32 as isize))
                    } else {
                        0 as u32
                    },
                ),
            );
        }
        16 => {
            if geno_compare(
                st,
                a,
                (w0 >> 4 as i32 & 7 as u32) as i32,
                geno_operand_b(
                    st,
                    a,
                    w0,
                    if len > 1 as i32 {
                        u32::from_be(*w.offset(1 as i32 as isize))
                    } else {
                        0 as u32
                    },
                ),
            ) == 0
            {
                skip = (if len > 2 as i32 {
                    u32::from_be(*w.offset(2 as i32 as isize)) as u32
                } else {
                    0 as u32
                }) as u32;
            }
        }
        18 => {
            let mut id_1: u32 = if len > 1 as i32 {
                u32::from_be(*w.offset(1 as i32 as isize))
            } else {
                0 as u32
            };
            let mut is_f_0: i32 = geno_val_is_float(id_1);
            if geno_cmp(
                is_f_0,
                geno_val_get(fp, st, id_1),
                (w0 >> 4 as i32 & 7 as u32) as i32,
                geno_operand(
                    st,
                    is_f_0,
                    (w0 & 0x80 as u32 != 0 as u32) as i32,
                    if len > 2 as i32 {
                        u32::from_be(*w.offset(2 as i32 as isize))
                    } else {
                        0 as u32
                    },
                ),
            ) == 0
            {
                skip = (if len > 3 as i32 {
                    u32::from_be(*w.offset(3 as i32 as isize)) as u32
                } else {
                    0 as u32
                }) as u32;
            }
        }
        17 => {
            skip = (if len > 1 as i32 {
                u32::from_be(*w.offset(1 as i32 as isize)) as u32
            } else {
                0 as u32
            }) as u32;
        }
        19 => {
            let mut slot: i32 = geno_slot_of(w);
            (*cmd).anon_8.u = if slot >= 0 as i32 {
                Geno_OverlayOrig[slot as usize]
            } else {
                ::core::ptr::null_mut::<CmdUnion>()
            };
            ProfEnd();
            return;
        }
        32 => {
            if mode != GENO_MODE_SKIP {
                GenoGame_CallHook(
                    gobj,
                    (if len > 1 as i32 {
                        u32::from_be(*w.offset(1 as i32 as isize))
                            as u32
                    } else {
                        0 as u32
                    }) as i32,
                    (if len > 2 as i32 {
                        u32::from_be(*w.offset(2 as i32 as isize))
                            as u32
                    } else {
                        0 as u32
                    }) as i32,
                );
            }
        }
        48 => {
            if !(mode == GENO_MODE_SKIP && w0 & GENO_CHG_ONCE as u32 != 0) {
                geno_register_check(
                    fp,
                    st,
                    w0 & 0xffff as u32,
                    if len > 1 as i32 {
                        u32::from_be(*w.offset(1 as i32 as isize))
                    } else {
                        0 as u32
                    },
                    if len > 2 as i32 {
                        u32::from_be(*w.offset(2 as i32 as isize))
                    } else {
                        0 as u32
                    },
                    if len > 3 as i32 {
                        u32::from_be(*w.offset(3 as i32 as isize))
                    } else {
                        0 as u32
                    },
                );
            }
        }
        49 => {
            geno_and_check(
                st,
                w0 & 0xffff as u32,
                if len > 1 as i32 {
                    u32::from_be(*w.offset(1 as i32 as isize))
                } else {
                    0 as u32
                },
                if len > 2 as i32 {
                    u32::from_be(*w.offset(2 as i32 as isize))
                } else {
                    0 as u32
                },
            );
        }
        50 => {
            (*st).nchecks = 0 as u32;
            (*st).last_check = -(1 as i32) as i32;
        }
        56 => {
            let mut i: i32 = 0;
            let mut n: i32 = (if len > 1 as i32 {
                u32::from_be(*w.offset(1 as i32 as isize)) as u32
            } else {
                0 as u32
            }) as i32;
            i = 0 as i32;
            while i < GENO_MAX_REHIT {
                if a & (1 as i32) << i != 0 {
                    (*st).rehit_period[i as usize] = (if n > 0 as i32 {
                        n as i32
                    } else {
                        0 as i32
                    }) as i32;
                    (*st).rehit_count[i as usize] = 0 as i32 as i32;
                }
                i += 1;
            }
        }
        57 => {
            let mut i_0: i32 = 0;
            i_0 = 0 as i32;
            while i_0 < GENO_MAX_REHIT {
                if a & (1 as i32) << i_0 != 0 {
                    (*st).link_mode[i_0 as usize] = (if len > 1 as i32 {
                        u32::from_be(*w.offset(1 as i32 as isize))
                            as u32
                    } else {
                        0 as u32
                    }) as i32;
                }
                i_0 += 1;
            }
        }
        59 => {
            let mut i_1: i32 = 0;
            let mut f: i32 = geno_operand(
                    st,
                    0 as i32,
                    (w0 & 0x80 as u32 != 0 as u32) as i32,
                    if len > 1 as i32 {
                        u32::from_be(*w.offset(1 as i32 as isize))
                    } else {
                        0 as u32
                    },
                )
                .i;
            i_1 = 0 as i32;
            while i_1 < GENO_MAX_REHIT {
                if a & (1 as i32) << i_1 != 0 {
                    (*st).stun_add[i_1 as usize] = (if f < 0 as i32 {
                        0 as i32
                    } else if f > 255 as i32 {
                        255 as i32
                    } else {
                        f as i32
                    }) as i32;
                }
                i_1 += 1;
            }
        }
        60 => {
            let mut i_2: i32 = 0;
            let mut f_0: i32 = geno_operand(
                    st,
                    0 as i32,
                    (w0 & 0x80 as u32 != 0 as u32) as i32,
                    if len > 1 as i32 {
                        u32::from_be(*w.offset(1 as i32 as isize))
                    } else {
                        0 as u32
                    },
                )
                .i;
            i_2 = 0 as i32;
            while i_2 < GENO_MAX_REHIT {
                if a & (1 as i32) << i_2 != 0 {
                    (*st).hb_flags[i_2 as usize] = (f_0 as i32
                        & 0xf as i32) as i32;
                }
                i_2 += 1;
            }
        }
        58 => {
            let mut i_3: i32 = 0;
            let mut d: f32 = geno_operand(
                    st,
                    1 as i32,
                    (w0 & 0x80 as u32 != 0 as u32) as i32,
                    if len > 1 as i32 {
                        u32::from_be(*w.offset(1 as i32 as isize))
                    } else {
                        0 as u32
                    },
                )
                .f;
            i_3 = 0 as i32;
            while i_3 < 4 as i32 {
                if a & (1 as i32) << i_3 != 0 {
                    (*fp).x914[i_3 as usize].damage = if d < 0.0f32 {
                        0.0f32
                    } else {
                        d as f32
                    };
                    ScriptGame_HitRuleDamageOnly(
                        (&raw mut (*fp).x914 as *mut HitCapsule).offset(i_3 as isize)
                            as *mut HitCapsule,
                    );
                }
                i_3 += 1;
            }
        }
        _ => {
            Geno_Event(
                3 as i32,
                (*fp).kind as i32,
                (*fp).player_id as i32,
                sub,
                0 as i32,
            );
        }
    }
    if skip > 0x10000 as i32 as u32 {
        skip = 0x10000 as i32 as u32;
    }
    (*cmd).anon_8.u = w.offset(len as isize).offset(skip as isize)
        as *mut CmdUnion;
    ProfEnd();
}

pub unsafe extern "C" fn Geno_HitStunBonus(
    mut atk: *mut Fighter,
    mut idx: i32,
    mut vic: *mut Fighter,
) {
    melee_core::ct!("Geno_HitStunBonus");
    let mut as_0: *mut GenoState = ::core::ptr::null_mut::<GenoState>();
    let mut add: i32 = 0;
    if atk.is_null() || vic.is_null() || idx < 0 as i32
        || idx >= GENO_MAX_REHIT
    {
        return;
    }
    as_0 = geno_state(atk);
    add = (*as_0).stun_add[idx as usize];
    Geno_GiveStunBonus(vic, add as i32);
}

pub unsafe extern "C" fn Geno_HitFlags(
    mut atk: *mut Fighter,
    mut idx: i32,
    mut vic: *mut Fighter,
) -> i32 {
    melee_core::ct!("Geno_HitFlags");
    let mut s: *mut GenoState = ::core::ptr::null_mut::<GenoState>();
    let mut f: i32 = 0;
    if atk.is_null() || vic.is_null() || idx < 0 as i32
        || idx >= GENO_MAX_REHIT
    {
        return 0 as i32;
    }
    f = (*geno_state(atk)).hb_flags[idx as usize];
    if f == 0 as i32 {
        return 0 as i32;
    }
    s = geno_state(vic);

    diag_geno_stun(
        -(1000 as i32) - (*vic).player_id as i32,
        f as i32,
        (*atk).player_id as i32,
    );
    if (*s).react_frame != geno_stun_frame() {
        (*s).react_flags = 0 as i32 as i32;
    }
    (*s).react_flags |= f;
    (*s).react_frame = geno_stun_frame() as i32;
    if f as i32 & GENO_HBF_NO_HITLAG != 0 {
        s = geno_state(atk);
        if (*s).react_frame != geno_stun_frame() {
            (*s).react_flags = 0 as i32 as i32;
        }
        (*s).react_flags |= GENO_HBF_NO_HITLAG;
        (*s).react_frame = geno_stun_frame() as i32;
    }
    return f as i32;
}

pub unsafe extern "C" fn geno_react(mut fp: *mut Fighter) -> i32 {
    melee_core::ct!("geno_react");
    let mut s: *mut GenoState = geno_state(fp);
    return if (*s).react_frame == geno_stun_frame() {
        (*s).react_flags
    } else {
        0 as i32
    };
}

pub unsafe extern "C" fn Geno_HitReact(mut fp: *mut Fighter) -> i32 {
    melee_core::ct!("Geno_HitReact");
    let f: i32 = geno_react(fp) as i32;
    let mut lifted: i32 = 0 as i32;
    if f as i32 & GENO_HBF_FORCE_REACTION != 0 {
        lifted = ((*fp).no_kb() as i32 != 0 as i32)
            as i32;
        (*fp).set_no_kb(0 as u8 as u8);
    }
    if f as i32 & GENO_HBF_FLINCHLESS != 0
        && f as i32 & GENO_HBF_FORCE_REACTION == 0
    {
        (*fp).dmg.kb_applied = 0.0f32;
    }
    return lifted;
}

pub unsafe extern "C" fn Geno_NoHitlag(mut fp: *mut Fighter) -> i32 {
    melee_core::ct!("Geno_NoHitlag");
    return (geno_react(fp) as i32 & GENO_HBF_NO_HITLAG
        != 0 as i32) as i32;
}

pub unsafe extern "C" fn Geno_GiveStunBonus(
    mut vic: *mut Fighter,
    mut add: i32,
) {
    melee_core::ct!("Geno_GiveStunBonus");
    let mut vs: *mut GenoState = ::core::ptr::null_mut::<GenoState>();
    if vic.is_null() || add <= 0 as i32 {
        return;
    }
    vs = geno_state(vic);
    if (*vs).stun_bonus_frame != geno_stun_frame() || add > (*vs).stun_bonus {
        (*vs).stun_bonus = add as i32;
    }
    (*vs).stun_bonus_frame = geno_stun_frame() as i32;
}

pub unsafe extern "C" fn Geno_TakeStunBonus(
    mut vic: *mut Fighter,
) -> i32 {
    melee_core::ct!("Geno_TakeStunBonus");
    let mut vs: *mut GenoState = geno_state(vic);
    let mut b: i32 = if (*vs).stun_bonus_frame == geno_stun_frame() {
        (*vs).stun_bonus
    } else {
        0 as i32
    };
    (*vs).stun_bonus = 0 as i32 as i32;
    return b as i32;
}

pub unsafe extern "C" fn GenoGame_StateOf(
    mut fp: *mut Fighter,
) -> *mut c_void {
    melee_core::ct!("GenoGame_StateOf");
    return geno_state(fp) as *mut c_void;
}

pub unsafe extern "C" fn geno_dg(mut h: u32, mut v: u32) -> u32 {
    melee_core::ct!("geno_dg");
    h ^= v;
    h = (h as u32).wrapping_mul(0x1000193 as u32)
        as u32 as u32;
    return h ^ h >> 13 as i32;
}

pub unsafe extern "C" fn geno_dgf(mut h: u32, mut f: f32) -> u32 {
    melee_core::ct!("geno_dgf");
    let mut c: Anon_anon_109 = unsafe { core::mem::zeroed() };
    c.f = f;
    return geno_dg(h, c.u);
}

pub unsafe extern "C" fn geno_nh(mut h: u32, mut v: u32) -> u32 {
    melee_core::ct!("geno_nh");
    h ^= v;
    h = (h as u32).wrapping_mul(0x1000193 as u32)
        as u32 as u32;
    return h ^ h >> 15 as i32;
}

pub unsafe extern "C" fn Geno_NeutralDigest(mut fp: *mut Fighter) -> u32 {
    melee_core::ct!("Geno_NeutralDigest");
    let mut s: *mut GenoState = ::core::ptr::null_mut::<GenoState>();
    let mut h: u32 = 0x811c9dc5 as u32;
    let mut i: i32 = 0;
    let mut c: Anon_anon_109 = unsafe { core::mem::zeroed() };
    if fp.is_null() {
        return 0 as u32;
    }
    s = geno_state(fp);
    if (*s).profile < 0 as i32
        && (*s).flags as u32 & GENO_SF_SCRIPT == 0 as u32
    {
        return 0 as u32;
    }
    h = geno_nh(h, 0x474e5631 as u32);
    i = 0 as i32;
    while i < GENO_VARS_PER_BANK {
        h = geno_nh(h, (*s).la_i[i as usize] as u32);
        i += 1;
    }
    i = 0 as i32;
    while i < GENO_VARS_PER_BANK {
        c.f = (*s).la_f[i as usize];
        h = geno_nh(h, c.u);
        i += 1;
    }
    i = 0 as i32;
    while i < GENO_VARS_PER_BANK {
        h = geno_nh(h, (*s).ra_i[i as usize] as u32);
        i += 1;
    }
    i = 0 as i32;
    while i < GENO_VARS_PER_BANK {
        c.f = (*s).ra_f[i as usize];
        h = geno_nh(h, c.u);
        i += 1;
    }
    i = 0 as i32;
    while i < GENO_MOVE_VARS {
        h = geno_nh(h, (*s).move_i[i as usize] as u32);
        i += 1;
    }
    i = 0 as i32;
    while i < GENO_MOVE_VARS {
        c.f = (*s).move_f[i as usize];
        h = geno_nh(h, c.u);
        i += 1;
    }
    h = geno_nh(h, (*s).action_time as u32);
    return h;
}

pub unsafe extern "C" fn GenoDefine_StateDigest(mut fp: *mut Fighter) -> u32 {
    melee_core::ct!("GenoDefine_StateDigest");
    let mut s: *mut GenoState = ::core::ptr::null_mut::<GenoState>();
    let mut h: u32 = 0x47444946 as u32;
    let mut i: i32 = 0;
    let mut j: i32 = 0;
    if fp.is_null()
        || Geno_DefineBaseKind((*fp).kind as i32)
            < 0 as i32
    {
        return 0 as u32;
    }
    s = geno_state(fp);
    if (*s).profile < 0 as i32 {
        return 0 as u32;
    }
    i = 0 as i32;
    while i < GENO_VARS_PER_BANK {
        h = geno_dg(h, (*s).la_i[i as usize] as u32);
        h = geno_dg(h, (*s).ra_i[i as usize] as u32);
        h = geno_dgf(h, (*s).la_f[i as usize]);
        h = geno_dgf(h, (*s).ra_f[i as usize]);
        i += 1;
    }
    h = geno_dg(h, (*s).extra_jumps);
    h = geno_dg(h, (*s).action_time as u32);
    h = geno_dg(h, (*s).nchecks);
    h = geno_dg(h, (*s).last_check as u32);
    i = 0 as i32;
    while i < (*s).nchecks as i32 && i < GENO_MAX_CHECKS {
        h = geno_dg(h, (*s).checks[i as usize].target);
        h = geno_dg(h, (*s).checks[i as usize].once);
        h = geno_dg(h, (*s).checks[i as usize].ncond);
        j = 0 as i32;
        while j < (*s).checks[i as usize].ncond as i32
            && j < GENO_CHECK_CONDS
        {
            h = geno_dg(h, (*s).checks[i as usize].cond[j as usize].head);
            h = geno_dg(h, (*s).checks[i as usize].cond[j as usize].arg1);
            h = geno_dg(h, (*s).checks[i as usize].cond[j as usize].arg2);
            j += 1;
        }
        i += 1;
    }
    i = 0 as i32;
    while i < GENO_MAX_REHIT {
        h = geno_dg(h, (*s).rehit_period[i as usize] as u32);
        h = geno_dg(h, (*s).rehit_count[i as usize] as u32);
        h = geno_dg(h, (*s).link_mode[i as usize] as u32);
        h = geno_dg(h, (*s).stun_add[i as usize] as u32);
        h = geno_dg(h, (*s).hb_flags[i as usize] as u32);
        i += 1;
    }
    h = geno_dg(h, (*s).stun_bonus as u32);
    h = geno_dg(h, (*s).hold_motion as u32);
    h = geno_dg(h, (*s).hold_frames as u32);
    i = 0 as i32;
    while i < GENO_MOVE_VARS {
        h = geno_dg(h, (*s).move_i[i as usize] as u32);
        h = geno_dgf(h, (*s).move_f[i as usize]);
        i += 1;
    }
    h = geno_dg(h, (*s).enter_from as u32);
    h = geno_dg(h, (*s).hidden as u32);
    h = geno_dg(h, (*s).hit_count as u32);
    h = geno_dgf(h, (*s).hit_damage);
    h = geno_dg(h, (*s).hit_port as u32);
    h = geno_dg(h, (*s).hit_counter as u32);
    h = geno_dg(h, (*s).counters as u32);
    h = geno_dg(h, (*s).ledge as u32);
    h = geno_dg(h, (*s).fall_ledge as u32);
    h = geno_dg(h, (*s).motion_started as u32);
    h = geno_dgf(h, (*s).motion_vy);
    h = geno_dgf(h, (*s).motion_facing);
    h = geno_dgf(h, (*s).motion_gravity);
    h = geno_dg(h, (*s).enter_keep as u32);
    h = geno_dg(h, (*s).atk_connected as u32);
    h = geno_dg(h, (*s).atk_connected_prev as u32);
    h = geno_dgf(h, (*s).fall_limit);
    let mut ad: u32 = geno_art_digest(fp);
    if ad != 0 as u32 {
        h = geno_dg(h, ad);
    }
    let mut ld: u32 = geno_lua_digest(fp, s);
    if ld != 0 as u32 {
        h = geno_dg(h, ld);
    }
    let mut cd: u32 = geno_combo_digest(fp, s);
    if cd != 0 as u32 {
        h = geno_dg(h, cd);
    }
    return if h != 0 as u32 { h } else { 1 as u32 };
}

pub unsafe extern "C" fn geno_storage_take(
    mut buffer: *mut u8,
    mut cap: u32,
    mut used: *mut u32,
    mut bytes: i32,
) -> *mut c_void {
    melee_core::ct!("geno_storage_take");
    let mut n: u32 = 0;
    let mut result: *mut c_void = ::core::ptr::null_mut::<
        c_void,
    >();
    if bytes <= 0 as i32 || buffer.is_null() || used.is_null()
        || *used > cap
    {
        return ::core::ptr::null_mut::<c_void>();
    }
    n = (bytes as u32).wrapping_add(7 as u32)
        & !(7 as u32);
    if n < bytes as u32 || n > cap.wrapping_sub(*used) {
        return ::core::ptr::null_mut::<c_void>();
    }
    result = buffer.offset(*used as isize) as *mut c_void;
    *used = (*used).wrapping_add(n);
    return result;
}

pub unsafe extern "C" fn Geno_ProfileStorageBytes() -> u32 {
    melee_core::ct!("Geno_ProfileStorageBytes");
    return Geno_ProfileStorageUsed;
}

pub unsafe extern "C" fn geno_profile_alloc(
    mut bytes: i32,
) -> *mut c_void {
    melee_core::ct!("geno_profile_alloc");
    let mut p: *mut c_void = geno_storage_take(
        &raw mut Geno_ProfileStorage.bytes as *mut u8,
        GENO_PROFILE_STORAGE_BYTES,
        &raw mut Geno_ProfileStorageUsed,
        bytes,
    );
    if p.is_null() {
        OSReport!(
            b"geno: profile descriptor storage exhausted: requested %d bytes, used %u of %u; refused\n\0"
                as *const u8 as *const i8 as *mut i8,
            bytes,
            Geno_ProfileStorageUsed,
            GENO_PROFILE_STORAGE_BYTES,
        );
        __assert(
            b"pc/geno/geno_profiles.inc\0" as *const u8 as *const i8,
            30 as u32,
            b"Geno profile descriptor byte budget exhausted\0" as *const u8
                as *const i8,
        );
    }
    geno_zero(p, bytes);
    return p;
}

pub unsafe extern "C" fn geno_profiles_prepare() {
    melee_core::ct!("geno_profiles_prepare");
    let mut n: i32 = Geno_ProfileCount();
    if n <= Geno_ProfileCapacity {
        return;
    }
    let mut rows: *mut *mut MotionState = geno_profile_alloc(
        (n as usize).wrapping_mul(::core::mem::size_of::<*mut MotionState>() as usize)
            as i32,
    ) as *mut *mut MotionState;
    let mut params: *mut *mut f32 = geno_profile_alloc(
        (n as usize).wrapping_mul(::core::mem::size_of::<*mut f32>() as usize)
            as i32,
    ) as *mut *mut f32;
    let mut counts: *mut i32 = geno_profile_alloc(
        (n as usize).wrapping_mul(::core::mem::size_of::<i32>() as usize)
            as i32,
    ) as *mut i32;
    let mut i: i32 = 0;
    i = 0 as i32;
    while i < Geno_ProfileCapacity {
        let ref mut fresh12 = *rows.offset(i as isize);
        *fresh12 = *Geno_Rows.offset(i as isize);
        let ref mut fresh13 = *params.offset(i as isize);
        *fresh13 = *Geno_Params.offset(i as isize);
        *counts.offset(i as isize) = *Geno_RowCount.offset(i as isize);
        i += 1;
    }
    Geno_Rows = rows;
    Geno_Params = params;
    Geno_RowCount = counts;
    Geno_ProfileCapacity = n;
}

pub unsafe extern "C" fn geno_profile_prepare(mut p: i32) {
    melee_core::ct!("geno_profile_prepare");
    geno_profiles_prepare();
    if p < 0 as i32 || p >= Geno_ProfileCapacity {
        return;
    }
    if (*Geno_Rows.offset(p as isize)).is_null() {
        let ref mut fresh10 = *Geno_Rows.offset(p as isize);
        *fresh10 = geno_profile_alloc(
            (GENO_MAX_STATES as usize)
                .wrapping_mul(::core::mem::size_of::<MotionState>() as usize)
                as i32,
        ) as *mut MotionState;
    }
    if (*Geno_Params.offset(p as isize)).is_null() {
        let ref mut fresh11 = *Geno_Params.offset(p as isize);
        *fresh11 = geno_profile_alloc(
            (GENO_PARAMS as usize).wrapping_mul(::core::mem::size_of::<f32>() as usize)
                as i32,
        ) as *mut f32;
    }
}

pub unsafe extern "C" fn geno_fam_eq(
    mut a: *const i8,
    mut tag: *const i8,
) -> i32 {
    melee_core::ct!("geno_fam_eq");
    while *a as i32 != 0 && *tag as i32 != 0 {
        let mut c: i8 = *a;
        if c as i32 >= 'a' as i32 && c as i32 <= 'z' as i32
        {
            c = (c as i32 - 'a' as i32 + 'A' as i32)
                as i8;
        }
        if c as i32 != *tag as i32 {
            return 0 as i32;
        }
        a = a.offset(1);
        tag = tag.offset(1);
    }
    return (*a as i32 == *tag as i32)
        as i32;
}

pub unsafe extern "C" fn GenoGame_ParamFind(
    mut family: *const i8,
    mut name: *const i8,
) -> i32 {
    melee_core::ct!("GenoGame_ParamFind");
    let mut i: i32 = 0;
    i = 0 as i32;
    while i < GENO_NPARAMDEFS {
        if geno_fam_eq(family, geno_bhv_params[i as usize].family) != 0
            && geno_streq(name, geno_bhv_params[i as usize].name) != 0
        {
            return geno_bhv_params[i as usize].id;
        }
        i += 1;
    }
    return -(1 as i32);
}

pub unsafe extern "C" fn geno_param(
    mut st: *mut GenoState,
    mut id: i32,
) -> f32 {
    melee_core::ct!("geno_param");
    let mut p: i32 = (*st).profile as i32;
    if p < 0 as i32 || p >= Geno_ProfileCapacity
        || (*Geno_Params.offset(p as isize)).is_null() || id < 0 as i32
        || id >= GENO_PARAMS
    {
        return 0.0f32;
    }
    return *(*Geno_Params.offset(p as isize)).offset(id as isize);
}

pub unsafe extern "C" fn geno_build_params(mut p: i32) {
    melee_core::ct!("geno_build_params");
    let mut i: i32 = 0;
    let mut n: i32 = 0;
    let mut b: Anon_anon_1941 = unsafe { core::mem::zeroed() };
    i = 0 as i32;
    while i < GENO_PARAMS {
        *(*Geno_Params.offset(p as isize)).offset(i as isize) = 0.0f32 as f32;
        i += 1;
    }
    i = 0 as i32;
    while i < GENO_NPARAMDEFS {
        *(*Geno_Params.offset(p as isize))
            .offset(geno_bhv_params[i as usize].id as isize) = geno_bhv_params[i
                as usize]
            .def;
        i += 1;
    }
    n = Geno_ParamCount(p);
    i = 0 as i32;
    while i < n {
        let mut id: i32 = Geno_ParamId(p, i);
        if id >= 0 as i32 && id < GENO_PARAMS {
            b.u = Geno_ParamBits(p, i) as u32;
            *(*Geno_Params.offset(p as isize)).offset(id as isize) = b.f;
        }
        i += 1;
    }
}

pub unsafe extern "C" fn geno_state_valid(
    mut st: *mut GenoState,
    mut s: i32,
) -> i32 {
    melee_core::ct!("geno_state_valid");
    return ((*st).profile >= 0 as i32 && s >= 0 as i32
        && s < GENO_MAX_STATES
        && s < Geno_StateCount((*st).profile as i32))
        as i32;
}

pub unsafe extern "C" fn geno_cur_state(
    mut fp: *mut Fighter,
    mut st: *mut GenoState,
) -> i32 {
    melee_core::ct!("geno_cur_state");
    let mut s: i32 = (*fp).motion_id - GENO_MOTION_BASE;
    return if geno_state_valid(st, s) != 0 { s } else { -(1 as i32) };
}

pub unsafe extern "C" fn geno_state_with(
    mut st: *mut GenoState,
    mut bhv: i32,
) -> i32 {
    melee_core::ct!("geno_state_with");
    let mut s: i32 = 0;
    let mut n: i32 = if (*st).profile >= 0 as i32 {
        Geno_StateCount((*st).profile as i32)
    } else {
        0 as i32
    };
    s = 0 as i32;
    while s < n && s < GENO_MAX_STATES {
        if Geno_StateBehavior((*st).profile as i32, s) == bhv {
            return s;
        }
        s += 1;
    }
    return -(1 as i32);
}

pub unsafe extern "C" fn geno_anim_ended(mut gobj: *mut Fighter_GObj) -> i32 {
    melee_core::ct!("geno_anim_ended");
    return if geno_test_capture != 0 {
        geno_test_anim_end as i32
    } else {
        (ftAnim_IsFramesRemaining(gobj) == 0) as i32
    };
}

pub unsafe extern "C" fn geno_vanilla_row(
    mut fp: *mut Fighter,
    mut msid: i32,
) -> *mut MotionState {
    melee_core::ct!("geno_vanilla_row");
    if msid < 0 as i32 || msid >= GENO_MOTION_BASE {
        return ::core::ptr::null_mut::<MotionState>();
    }
    if msid >= (*fp).x18 {
        return if !(*fp).x20_actionStateList.is_null()
            && msid - (*fp).x18 <= 0xff as i32
        {
            (*fp).x20_actionStateList.offset((msid - (*fp).x18) as isize)
                as *mut MotionState
        } else {
            ::core::ptr::null_mut::<MotionState>()
        };
    }
    return if !(*fp).x1C_actionStateList.is_null() {
        (*fp).x1C_actionStateList.offset(msid as isize) as *mut MotionState
    } else {
        ::core::ptr::null_mut::<MotionState>()
    };
}

pub unsafe extern "C" fn geno_build_rows(mut fp: *mut Fighter, mut p: i32) {
    melee_core::ct!("geno_build_rows");
    let mut n: i32 = Geno_StateCount(p);
    let mut s: i32 = 0;
    let mut slot: i32 = 0;
    if n > GENO_MAX_STATES {
        n = GENO_MAX_STATES;
    }
    s = 0 as i32;
    while s < n {
        let mut row: *mut MotionState = (*Geno_Rows.offset(p as isize))
            .offset(s as isize) as *mut MotionState;
        let mut b: *const GenoBhvDef = geno_bhv_def(Geno_StateBehavior(p, s));
        let mut like_t: u32 = Geno_StateLike(p, s) as u32;
        let mut like: i32 = if like_t != GENO_NONE_T {
            geno_target_motion(fp, like_t)
        } else {
            (*b).like
        };
        let mut lr: *mut MotionState = geno_vanilla_row(fp, like);
        let mut anim: i32 = Geno_StateAnim(p, s);
        let mut from: u32 = Geno_StateAnimFrom(p, s) as u32;
        if lr.is_null() {
            lr = geno_vanilla_row(fp, ftCo_MS_Fall as i32);
        }
        if !lr.is_null() {
            *row = *lr;
        } else {
            geno_zero(
                row as *mut c_void,
                ::core::mem::size_of::<MotionState>() as i32,
            );
            (*row).anim_id = -(1 as i32) as enum_t;
        }
        if anim >= 0 as i32 {
            (*row).anim_id = anim as enum_t;
        } else if from != GENO_NONE_T {
            let mut fr: *mut MotionState = geno_vanilla_row(
                fp,
                geno_target_motion(fp, from),
            );
            (*row).anim_id = (if !fr.is_null() {
                (*fr).anim_id as i32
            } else {
                -(1 as i32)
            }) as enum_t;
        }
        if Geno_StateFlagsSet(p, s) != 0 {
            (*row).x4_flags = Geno_StateFlags(p, s);
        }
        if Geno_StateMoveId(p, s) >= 0 as i32 {
            (*row)
                .anon_8.anon_0
                .set_move_id(Geno_StateMoveId(p, s) as u8 as u8);
        }
        slot = 0 as i32;
        while slot < GENO_CB_SLOTS as i32 {
            let mut id: i32 = Geno_StateCb(p, s, slot);
            let mut c: *const GenoCbDef = ::core::ptr::null::<GenoCbDef>();
            let mut r#fn: GenoCbFn = None;
            if !(id == GENO_CB_LIKE) {
                if id < 0 as i32 {
                    id = (*b).cb[slot as usize] as i32;
                }
                if !(id == GENO_CB_LIKE) {
                    c = geno_cb_def(slot, id);
                    r#fn = if !c.is_null() { (*c).r#fn } else { None };
                    match slot {
                        0 => {
                            (*row).anim_cb = r#fn as HSD_GObjEvent;
                        }
                        1 => {
                            (*row).input_cb = r#fn as HSD_GObjEvent;
                        }
                        2 => {
                            (*row).phys_cb = r#fn as HSD_GObjEvent;
                        }
                        _ => {
                            (*row).coll_cb = r#fn as HSD_GObjEvent;
                        }
                    }
                }
            }
            slot += 1;
        }
        if ((*row).input_cb.is_none()
            || (*row).input_cb
                == Some(
                    geno_cb_nothing as unsafe extern "C" fn(*mut Fighter_GObj) -> (),
                )) && (*row).anim_id >= 0 as i32
            && (*row).anim_id <= 0x3ff as i32 && !(*fp).ft_data.is_null()
            && !(*(*fp).ft_data).xC.is_null()
            && GenoGame_ScriptHasIasa(
                (*(*(*fp).ft_data).xC.offset((*row).anim_id as isize)).xC
                    as *const c_void,
            ) != 0
        {
            Geno_WarnIasa(p, s);
        }
        Geno_Event(
            20 as i32,
            (*fp).kind as i32,
            (*fp).player_id as i32,
            s,
            (*row).anim_id,
        );
        s += 1;
    }
    *Geno_RowCount.offset(p as isize) = n as i32;
}

pub unsafe extern "C" fn geno_v2_build(mut fp: *mut Fighter, mut p: i32) {
    melee_core::ct!("geno_v2_build");
    if p < 0 as i32 || p >= GENO_MAX_PROFILES {
        return;
    }
    geno_profile_prepare(p);
    geno_build_params(p);
    geno_build_rows(fp, p);
}

pub unsafe extern "C" fn Geno_MotionRow(
    mut fp: *mut Fighter,
    mut msid: i32,
) -> *mut MotionState {
    melee_core::ct!("Geno_MotionRow");
    let mut st: *mut GenoState = geno_state(fp);
    let mut s: i32 = msid - GENO_MOTION_BASE;
    if geno_state_valid(st, s) != 0 && s < *Geno_RowCount.offset((*st).profile as isize)
    {
        return (*Geno_Rows.offset((*st).profile as isize)).offset(s as isize)
            as *mut MotionState;
    }
    Geno_Event(
        8 as i32,
        (*fp).kind as i32,
        (*fp).player_id as i32,
        s,
        0 as i32,
    );
    return geno_vanilla_row(fp, ftCo_MS_Fall as i32);
}

pub unsafe extern "C" fn geno_start_frame(
    mut fp: *mut Fighter,
    mut st: *mut GenoState,
    mut s: i32,
) -> f32 {
    melee_core::ct!("geno_start_frame");
    if Geno_StateBehavior((*st).profile as i32, s)
        == GENO_BHV_GLIDE as i32
    {
        let mut pose: f32 = geno_param(
            st,
            GENO_BP_GLIDE_pose_center as i32,
        );
        if pose > 0.0f32 {
            return pose;
        }
    }
    return 0.0f32;
}

pub unsafe extern "C" fn geno_enter_state(
    mut gobj: *mut Fighter_GObj,
    mut fp: *mut Fighter,
    mut st: *mut GenoState,
    mut s: i32,
    mut target: u32,
) -> i32 {
    melee_core::ct!("geno_enter_state");
    let mut b: *const GenoBhvDef = geno_bhv_def(
        Geno_StateBehavior((*st).profile as i32, s),
    );
    let mut from: i32 = (*fp).motion_id;
    let mut k_started: i32 = (*st).motion_started;
    let mut k_ledge: i32 = (*st).ledge;
    let mut k_vy: f32 = (*st).motion_vy;
    let mut k_facing: f32 = (*st).motion_facing;
    let mut k_grav: f32 = (*st).motion_gravity;
    if (*st).profile >= GENO_MAX_PROFILES
        || s >= *Geno_RowCount.offset((*st).profile as isize)
    {
        Geno_Event(
            8 as i32,
            (*fp).kind as i32,
            (*fp).player_id as i32,
            s,
            0 as i32,
        );
        return 0 as i32;
    }
    (*st).changes = (*st).changes.wrapping_add(1);
    (*st).state_entries = (*st).state_entries.wrapping_add(1);
    (*st).enter_from = from as i32;
    (*st).enter_keep = (target as u32 & GENO_TGT_KEEP_FRAME
        != 0 as u32) as i32 as i32;
    Geno_Event(
        16 as i32,
        (*fp).kind as i32,
        (*fp).player_id as i32,
        s,
        from,
    );
    if (*st).enter_keep == 0 {
        (*fp).set_allow_interrupt(0 as u8 as u8);
    }
    if geno_test_capture != 0 {
        geno_test_target = target;
        geno_test_changes += 1;
        (*fp).motion_id = (GENO_MOTION_BASE + s) as FtMotionId;
        Geno_OnActionChange(gobj);
    } else if target as u32 & GENO_TGT_KEEP_FRAME != 0 {
        Fighter_ChangeMotionState(
            gobj,
            GENO_MOTION_BASE + s as FtMotionId,
            Ft_MF_KeepGfx | Ft_MF_SkipHit | Ft_MF_SkipMatAnim | Ft_MF_KeepSfx
                | Ft_MF_SkipColAnim | Ft_MF_UpdateCmd | Ft_MF_SkipNametagVis
                | Ft_MF_KeepSwordTrail | Ft_MF_SkipItemVis | Ft_MF_SkipModelPartVis
                | Ft_MF_SkipAttackCount | Ft_MF_KeepFastFall,
            (*fp).cur_anim_frame as f32,
            1.0f32,
            0.0f32,
            ::core::ptr::null_mut::<Fighter_GObj>(),
        );
    } else {
        Fighter_ChangeMotionState(
            gobj,
            GENO_MOTION_BASE + s as FtMotionId,
            Ft_MF_None,
            geno_start_frame(fp, st, s),
            1.0f32,
            0.0f32,
            ::core::ptr::null_mut::<Fighter_GObj>(),
        );
    }
    if (*st).hidden != 0 {
        (*fp).set_x221E_b5(1 as u8 as u8);
    }
    if (*st).enter_keep != 0 {
        (*st).motion_started = k_started;
        (*st).motion_vy = k_vy;
        (*st).motion_facing = k_facing;
        (*st).motion_gravity = k_grav;
        (*st).ledge = k_ledge;
    }
    if (*b).enter.is_some() {
        (*b).enter.expect("non-null function pointer")(gobj, fp, st, s);
    }
    geno_lua_enter(gobj, fp, st, s);
    return 1 as i32;
}

pub unsafe extern "C" fn geno_go(
    mut gobj: *mut Fighter_GObj,
    mut fp: *mut Fighter,
    mut st: *mut GenoState,
    mut t: u32,
) -> i32 {
    melee_core::ct!("geno_go");
    if t == GENO_NONE_T {
        return 0 as i32;
    }
    if t == GENO_TGT_HELPLESS {
        return geno_do_change(
            gobj,
            fp,
            st,
            (0 as u32 & 15 as u32) << 28 as i32
                | (if (*fp).ground_or_air as i32
                    == GA_Ground as i32
                {
                    ftCo_MS_Wait as i32
                } else {
                    ftCo_MS_FallSpecial as i32
                }) as u32 & 0xffff as u32,
        );
    }
    if t == GENO_AUTO {
        return geno_do_change(
            gobj,
            fp,
            st,
            (0 as u32 & 15 as u32) << 28 as i32
                | (if (*fp).ground_or_air as i32
                    == GA_Ground as i32
                {
                    ftCo_MS_Wait as i32
                } else {
                    ftCo_MS_Fall as i32
                }) as u32 & 0xffff as u32,
        );
    }
    if t >> 28 as i32 == GENO_TGT_BHV {
        let mut s: i32 = geno_state_with(
            st,
            (t & 0xffff as u32) as i32,
        );
        if s < 0 as i32 {
            return geno_go(gobj, fp, st, GENO_AUTO);
        }
        t = ((2 as u32 & 15 as u32)
            << 28 as i32
            | s as u32 & 0xffff as u32) as u32;
    }
    return geno_do_change(gobj, fp, st, t);
}

pub unsafe extern "C" fn geno_next_of(
    mut st: *mut GenoState,
    mut s: i32,
) -> u32 {
    melee_core::ct!("geno_next_of");
    let mut t: u32 = Geno_StateNext((*st).profile as i32, s) as u32;
    return if t != GENO_NONE_T {
        t
    } else {
        (*geno_bhv_def(Geno_StateBehavior((*st).profile as i32, s))).next
    };
}

pub unsafe extern "C" fn geno_land_of(
    mut st: *mut GenoState,
    mut s: i32,
) -> u32 {
    melee_core::ct!("geno_land_of");
    let mut t: u32 = Geno_StateLand((*st).profile as i32, s) as u32;
    return if t != GENO_NONE_T {
        t
    } else {
        (*geno_bhv_def(Geno_StateBehavior((*st).profile as i32, s))).land
    };
}

pub unsafe extern "C" fn geno_lag_of(
    mut st: *mut GenoState,
    mut s: i32,
) -> f32 {
    melee_core::ct!("geno_lag_of");
    let mut b: Anon_anon_1941 = unsafe { core::mem::zeroed() };
    b.u = Geno_StateLagBits((*st).profile as i32, s) as u32;
    return b.f;
}

pub unsafe extern "C" fn geno_cb_anim_next(mut gobj: *mut Fighter_GObj) {
    melee_core::ct!("geno_cb_anim_next");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj as *mut HSD_GObj)
        as *mut Fighter;
    let mut st: *mut GenoState = geno_state(fp);
    let mut s: i32 = geno_cur_state(fp, st);
    if geno_anim_ended(gobj) != 0 {
        geno_go(
            gobj,
            fp,
            st,
            if s >= 0 as i32 { geno_next_of(st, s) } else { GENO_AUTO },
        );
    }
}

pub unsafe extern "C" fn geno_cb_anim_loop(mut gobj: *mut Fighter_GObj) {
    melee_core::ct!("geno_cb_anim_loop");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj as *mut HSD_GObj)
        as *mut Fighter;
    if geno_anim_ended(gobj) != 0 && geno_test_capture == 0 {
        Fighter_ChangeMotionState(
            gobj,
            (*fp).motion_id,
            Ft_MF_KeepFastFall | Ft_MF_KeepGfx | Ft_MF_KeepSfx | Ft_MF_SkipModelPartVis
                | Ft_MF_SkipItemVis | Ft_MF_SkipNametagVis | Ft_MF_KeepSwordTrail,
            0.0f32,
            1.0f32,
            0.0f32,
            ::core::ptr::null_mut::<Fighter_GObj>(),
        );
    }
}

pub unsafe extern "C" fn geno_cb_nothing(mut gobj: *mut Fighter_GObj) { melee_core::ct!("geno_cb_nothing");}

pub unsafe extern "C" fn geno_cb_phys_air(mut gobj: *mut Fighter_GObj) {
    melee_core::ct!("geno_cb_phys_air");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj as *mut HSD_GObj)
        as *mut Fighter;
    if (*geno_state(fp)).fall_ledge != 0 {
        (*fp).set_fall_fast(0 as u8 as u8);
        ftCommon_Fall(fp, (*fp).co_attrs.gravity, (*fp).co_attrs.terminal_velocity);
        ftCommon_CalcSelfAccel_Drift(fp);
        return;
    }
    ft_80084DB0(gobj);
}

pub unsafe extern "C" fn geno_cb_phys_air_nodrift(mut gobj: *mut Fighter_GObj) {
    melee_core::ct!("geno_cb_phys_air_nodrift");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj as *mut HSD_GObj)
        as *mut Fighter;
    ftCommon_Fall(fp, (*fp).co_attrs.gravity, (*fp).co_attrs.terminal_velocity);
    ftCommon_CalcSelfAccel_DeaccelAir(fp);
}

pub unsafe extern "C" fn geno_cb_phys_air_drift(mut gobj: *mut Fighter_GObj) {
    melee_core::ct!("geno_cb_phys_air_drift");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj as *mut HSD_GObj)
        as *mut Fighter;
    if (*geno_state(fp)).fall_ledge != 0 {
        (*fp).set_fall_fast(0 as u8 as u8);
    }
    if (*fp).ground_or_air as i32 == GA_Ground as i32 {
        ft_80084F3C(gobj);
    } else {
        ftCommon_CalcSelfAccel_Drift(fp);
    };
}

pub unsafe extern "C" fn geno_cb_phys_ground(mut gobj: *mut Fighter_GObj) {
    melee_core::ct!("geno_cb_phys_ground");
    ft_80084F3C(gobj);
}

pub unsafe extern "C" fn geno_cb_phys_auto(mut gobj: *mut Fighter_GObj) {
    melee_core::ct!("geno_cb_phys_auto");
    if (*(HSD_GObjGetUserData(gobj as *mut HSD_GObj) as *mut Fighter)).ground_or_air
        as i32 == GA_Ground as i32
    {
        ft_80084F3C(gobj);
    } else {
        ft_80084DB0(gobj);
    };
}

pub unsafe extern "C" fn geno_land(mut gobj: *mut Fighter_GObj) {
    melee_core::ct!("geno_land");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj as *mut HSD_GObj)
        as *mut Fighter;
    let mut st: *mut GenoState = geno_state(fp);
    let mut s: i32 = geno_cur_state(fp, st);
    let mut t: u32 = if s >= 0 as i32 {
        geno_land_of(st, s)
    } else {
        GENO_AUTO
    };
    let mut lag: f32 = if s >= 0 as i32 {
        geno_lag_of(st, s)
    } else {
        0.0f32
    };
    if s >= 0 as i32 && gb_glide_land(gobj, fp, st, s) != 0 {
        return;
    }
    if t == GENO_AUTO || t == GENO_NONE_T {
        if lag > 0.0f32 {
            ftCo_LandingFallSpecial_Enter(gobj, 1, lag as f32);
        } else {
            ftCo_Landing_Enter_Basic(gobj);
        }
        return;
    }
    ftCommon_8007D7FC(fp);
    if geno_go(gobj, fp, st, t) == 0 {
        ftCo_Landing_Enter_Basic(gobj);
    }
}

pub unsafe extern "C" fn geno_cb_coll_air(mut gobj: *mut Fighter_GObj) {
    melee_core::ct!("geno_cb_coll_air");
    if (*geno_state(HSD_GObjGetUserData(gobj as *mut HSD_GObj) as *mut Fighter)).ledge
        == 2 as i32
    {
        geno_coll_air_ledge(
            gobj,
            1 as i32,
            Some(geno_land as unsafe extern "C" fn(*mut Fighter_GObj) -> ()),
        );
        return;
    }
    ft_80083090(
        gobj,
        Some(
            ftCo_80096CC8
                as unsafe extern "C" fn(*mut Fighter_GObj, i32) -> i32,
        ),
        Some(geno_land as unsafe extern "C" fn(*mut Fighter_GObj) -> ()),
    );
}

pub unsafe extern "C" fn geno_cb_coll_air_noledge(mut gobj: *mut Fighter_GObj) {
    melee_core::ct!("geno_cb_coll_air_noledge");
    ft_80082C74(gobj, Some(geno_land as unsafe extern "C" fn(*mut Fighter_GObj) -> ()));
}

pub unsafe extern "C" fn geno_cb_coll_ground(mut gobj: *mut Fighter_GObj) {
    melee_core::ct!("geno_cb_coll_ground");
    ft_80084104(gobj);
}

pub unsafe extern "C" fn geno_cb_coll_ground_stop(mut gobj: *mut Fighter_GObj) {
    melee_core::ct!("geno_cb_coll_ground_stop");
    ft_80084280(gobj);
}

pub unsafe extern "C" fn geno_become_air(mut gobj: *mut Fighter_GObj) {
    melee_core::ct!("geno_become_air");
    ftCommon_8007D5D4(HSD_GObjGetUserData(gobj as *mut HSD_GObj) as *mut Fighter);
}

pub unsafe extern "C" fn geno_become_ground(mut gobj: *mut Fighter_GObj) {
    melee_core::ct!("geno_become_ground");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj as *mut HSD_GObj)
        as *mut Fighter;
    let mut vx: f32 = (*fp).self_vel.x;
    ftCommon_8007D7FC(fp);
    if (*fp).anon_594.anon_0.x594_b0() != 0 {
        (*fp).self_vel.x = vx;
        (*fp).gr_vel = vx as f32;
    }
}

pub unsafe extern "C" fn geno_coll_both(mut gobj: *mut Fighter_GObj) {
    melee_core::ct!("geno_coll_both");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj as *mut HSD_GObj)
        as *mut Fighter;
    if (*fp).ground_or_air as i32 == GA_Ground as i32 {
        ft_8008403C(
            gobj,
            Some(geno_become_air as unsafe extern "C" fn(*mut Fighter_GObj) -> ()),
        );
    } else {
        ft_80082C74(
            gobj,
            Some(geno_become_ground as unsafe extern "C" fn(*mut Fighter_GObj) -> ()),
        );
    };
}

pub unsafe extern "C" fn geno_motion_opts(
    mut st: *mut GenoState,
    mut s: i32,
) -> u32 {
    melee_core::ct!("geno_motion_opts");
    return if s >= 0 as i32 {
        Geno_StateMotion((*st).profile as i32, s) as u32
    } else {
        0 as u32
    };
}

pub unsafe extern "C" fn geno_motion_gravity(
    mut st: *mut GenoState,
    mut s: i32,
) -> f32 {
    melee_core::ct!("geno_motion_gravity");
    let mut b: Anon_anon_1941 = unsafe { core::mem::zeroed() };
    b.u = (if s >= 0 as i32 {
        Geno_StateGravityBits((*st).profile as i32, s)
            as u32
    } else {
        0 as u32
    }) as u32;
    return b.f;
}

pub unsafe extern "C" fn geno_cb_phys_anim_motion(mut gobj: *mut Fighter_GObj) {
    melee_core::ct!("geno_cb_phys_anim_motion");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj as *mut HSD_GObj)
        as *mut Fighter;
    let mut st: *mut GenoState = geno_state(fp);
    let mut s: i32 = geno_cur_state(fp, st);
    let mut o: u32 = geno_motion_opts(st, s);
    let mut g: f32 = if (*st).motion_gravity >= 0.0f32 {
        (*st).motion_gravity
    } else {
        geno_motion_gravity(st, s)
    };
    let mut fwd: f32 = 0.0f32;
    let mut up: f32 = 0.0f32;
    let mut dx: f32 = 0.;
    let mut dy: f32 = 0.;
    let mut lr: f32 = if (*fp).facing_dir < 0.0f32 { -1.0f32 } else { 1.0f32 };
    if o as u32 & GENO_MOTION_ENTRY_FACING != 0 {
        if (*st).motion_started == 0 {
            (*st).motion_facing = lr;
        }
        lr = (if (*st).motion_facing < 0.0f32 { -1.0f32 } else { 1.0f32 }) as f32;
    }
    if (*fp).anon_594.anon_0.x594_b0() != 0 {
        fwd = (*fp).x6A4_transNOffset.z;
        up = (*fp).x6A4_transNOffset.y;
        if (*st).motion_started == 0
            && o as u32 & GENO_MOTION_ORIGIN != 0
        {
            fwd += (*fp).x68C_transNPos.z;
            up += (*fp).x68C_transNPos.y;
        }
    }
    (*st).motion_started = 1 as i32 as i32;
    GenoBhv_AnimMotionFrameDelta(fwd, up, lr, &raw mut dx, &raw mut dy);
    if (*fp).ground_or_air as i32 == GA_Ground as i32 {
        if (*fp).anon_594.anon_0.x594_b0() == 0 {
            ft_80084F3C(gobj);
            return;
        }
        if o as u32 & GENO_MOTION_LIFTOFF != 0 && up > 0.001f32 {
            Geno_Event(
                23 as i32,
                (*fp).kind as i32,
                (*fp).player_id as i32,
                s,
                (*st).action_time as i32,
            );
            ftCommon_8007D5D4(fp);
        } else {
            (*fp).gr_vel = dx as f32;
            ftCommon_SetSelfMovementFromGroundedMovement(gobj);
            return;
        }
    }
    if g != 0.0f32 {
        (*st).motion_vy = fmadds(-(*fp).co_attrs.gravity, g, (*st).motion_vy);
        if (*st).motion_vy < -(*fp).co_attrs.terminal_velocity {
            (*st).motion_vy = -(*fp).co_attrs.terminal_velocity as f32;
        }
    }
    if (*fp).anon_594.anon_0.x594_b0() == 0 && g == 0.0f32 {
        (*fp).self_vel.x = 0.0f32 as f32;
        (*fp).self_vel.y = 0.0f32 as f32;
        return;
    }
    (*fp).self_vel.x = dx;
    (*fp).self_vel.y = dy + (*st).motion_vy;
}

pub unsafe extern "C" fn geno_motion_land(mut gobj: *mut Fighter_GObj) {
    melee_core::ct!("geno_motion_land");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj as *mut HSD_GObj)
        as *mut Fighter;
    let mut st: *mut GenoState = geno_state(fp);
    let mut s: i32 = geno_cur_state(fp, st);
    Geno_Event(
        24 as i32,
        (*fp).kind as i32,
        (*fp).player_id as i32,
        s,
        if s >= 0 as i32 {
            geno_land_of(st, s) as i32
        } else {
            -(1 as i32)
        },
    );
    if s >= 0 as i32 && geno_land_of(st, s) == GENO_TGT_STAY {
        ftCommon_8007D7FC(fp);
        return;
    }
    geno_land(gobj);
}

pub unsafe extern "C" fn geno_coll_air_ledge(
    mut gobj: *mut Fighter_GObj,
    mut both: i32,
    mut land: HSD_GObjEvent,
) {
    melee_core::ct!("geno_coll_air_ledge");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj as *mut HSD_GObj)
        as *mut Fighter;
    let mut coll: *mut CollData = &raw mut (*fp).coll_data;
    let mut touched: i32 = 0;
    (*coll).last_pos = (*coll).cur_pos;
    (*coll).cur_pos = (*fp).cur_pos;
    mpColl_LedgeLevelOK = (both != 0 && (*geno_state(fp)).fall_ledge != 0)
        as i32;
    if (*fp).x2064_ledgeCooldown != 0 || (*fp).x2224_b2() as i32 != 0 {
        touched = mpColl_80047AC8(
            coll,
            Some(
                ftCo_80096CC8
                    as unsafe extern "C" fn(
                        *mut Fighter_GObj,
                        i32,
                    ) -> i32,
            ),
            gobj,
        );
    } else {
        mpCollSetFacingDir(
            coll,
            if both != 0 {
                0 as i32
            } else if (*fp).facing_dir < 0.0f32 {
                -(1 as i32)
            } else {
                1 as i32
            },
        );
        touched = mpColl_80047E14(
            coll,
            Some(
                ftCo_80096CC8
                    as unsafe extern "C" fn(
                        *mut Fighter_GObj,
                        i32,
                    ) -> i32,
            ),
            gobj,
        );
    }
    mpColl_LedgeLevelOK = 0 as i32;
    (*fp).cur_pos = (*coll).cur_pos;
    if ft_80081A00(gobj) != 0 {
        return;
    }
    if touched != 0 {
        land.expect("non-null function pointer")(gobj as *mut HSD_GObj);
        return;
    }
    if ftCliffCommon_80081298(gobj) != 0 {
        Geno_Event(
            25 as i32,
            (*fp).kind as i32,
            (*fp).player_id as i32,
            (*fp).motion_id - GENO_MOTION_BASE,
            if both != 0 { 2 as i32 } else { 1 as i32 },
        );
    }
}

pub unsafe extern "C" fn Geno_FallSpecialColl(
    mut gobj: *mut Fighter_GObj,
) -> i32 {
    melee_core::ct!("Geno_FallSpecialColl");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj as *mut HSD_GObj)
        as *mut Fighter;
    if ((*fp).kind as i32) < Ft_Kind_Mex0 as i32
        || (*geno_state(fp)).profile < 0 as i32
        || (*geno_state(fp)).fall_ledge == 0
    {
        return 0 as i32;
    }
    geno_coll_air_ledge(
        gobj,
        1 as i32,
        Some(ftCo_80096D28 as unsafe extern "C" fn(*mut Fighter_GObj) -> ()),
    );
    return 1 as i32;
}

pub unsafe extern "C" fn geno_ledge_mode(
    mut fp: *mut Fighter,
    mut st: *mut GenoState,
) -> i32 {
    melee_core::ct!("geno_ledge_mode");
    let mut s: i32 = geno_cur_state(fp, st);
    return if (*st).ledge >= 0 as i32 {
        (*st).ledge as i32
    } else {
        (geno_motion_opts(st, s) as u32 & GENO_MOTION_LEDGE_MASK)
            as i32
    };
}

pub unsafe extern "C" fn GenoGame_TestLedgeMode(
    mut fp: *mut Fighter,
) -> i32 {
    melee_core::ct!("GenoGame_TestLedgeMode");
    return geno_ledge_mode(fp, geno_state(fp));
}

pub unsafe extern "C" fn GenoGame_TestStartFrame(
    mut fp: *mut Fighter,
    mut s: i32,
) -> f32 {
    melee_core::ct!("GenoGame_TestStartFrame");
    return geno_start_frame(fp, geno_state(fp), s);
}

pub unsafe extern "C" fn geno_cb_coll_anim_motion(mut gobj: *mut Fighter_GObj) {
    melee_core::ct!("geno_cb_coll_anim_motion");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj as *mut HSD_GObj)
        as *mut Fighter;
    let mut st: *mut GenoState = geno_state(fp);
    let mut ledge: i32 = 0;
    if (*fp).ground_or_air as i32 == GA_Ground as i32 {
        ft_8008403C(
            gobj,
            Some(geno_become_air as unsafe extern "C" fn(*mut Fighter_GObj) -> ()),
        );
        return;
    }
    ledge = geno_ledge_mode(fp, st);
    if ledge == 0 as i32 {
        ft_80082C74(
            gobj,
            Some(geno_motion_land as unsafe extern "C" fn(*mut Fighter_GObj) -> ()),
        );
    } else {
        geno_coll_air_ledge(
            gobj,
            (ledge == 2 as i32) as i32,
            Some(geno_motion_land as unsafe extern "C" fn(*mut Fighter_GObj) -> ()),
        );
    };
}

pub unsafe extern "C" fn geno_jump_held(mut fp: *mut Fighter) -> i32 {
    melee_core::ct!("geno_jump_held");
    return ((*fp).input.held_buttons[0 as i32 as usize]
        & HSD_PAD_XY as HSD_Pad != 0 as HSD_Pad
        || !p_ftCommonData.is_null()
            && (*fp).input.lstick[0 as i32 as usize].y
                >= (*p_ftCommonData).tap_jump_threshold) as i32;
}

pub unsafe extern "C" fn gb_floor(mut x: f32) -> f32 {
    melee_core::ct!("gb_floor");
    let mut i: i32 = x as i32;
    let mut fi: f32 = i as f32;
    return if fi > x { fi - 1.0f32 } else { fi };
}

pub unsafe extern "C" fn gb_clamp(mut x: f32, mut lo: f32, mut hi: f32) -> f32 {
    melee_core::ct!("gb_clamp");
    if x < lo { return lo } else if x > hi { return hi }
    return x;
}

pub unsafe extern "C" fn gb_approach(
    mut x: f32,
    mut target: f32,
    mut step: f32,
) -> f32 {
    melee_core::ct!("gb_approach");
    let mut x2: f32 = 0.;
    if x < target {
        x2 = x + step;
        if x2 > target {
            return target;
        }
        return x2;
    }
    x2 = x - step;
    if x2 < target {
        return target;
    }
    return x2;
}

pub unsafe extern "C" fn gb_abs(mut x: f32) -> f32 {
    melee_core::ct!("gb_abs");
    return if x < 0.0f32 { -x } else { x };
}

pub unsafe extern "C" fn gb_sinq_at(mut i: i32) -> f32 {
    melee_core::ct!("gb_sinq_at");
    let mut b: Anon_anon_1941 = unsafe { core::mem::zeroed() };
    b.u = gb_sinq[i as usize];
    return b.f;
}

pub unsafe extern "C" fn GenoBhv_SinqWord(mut i: i32) -> u32 {
    melee_core::ct!("GenoBhv_SinqWord");
    return if i >= 0 as i32 && i < GENO_BHV_SINQ_N {
        gb_sinq[i as usize]
    } else {
        0 as u32
    };
}

pub unsafe extern "C" fn GenoBhv_SinqCrc32() -> u32 {
    melee_core::ct!("GenoBhv_SinqCrc32");
    let mut crc: u32 = 0xffffffff as u32;
    let mut i: i32 = 0;
    let mut b: i32 = 0;
    let mut k: i32 = 0;
    i = 0 as i32;
    while i < GENO_BHV_SINQ_N {
        let mut w: u32 = gb_sinq[i as usize];
        b = 0 as i32;
        while b < 4 as i32 {
            crc
                ^= w as u32 >> 8 as i32 * b
                    & 0xff as u32;
            k = 0 as i32;
            while k < 8 as i32 {
                let mut mask: u32 = if crc as u32
                    & 1 as u32 != 0
                {
                    0xedb88320 as u32
                } else {
                    0 as u32
                };
                crc = crc >> 1 as i32 ^ mask;
                k += 1;
            }
            b += 1;
        }
        i += 1;
    }
    return !crc;
}

pub unsafe extern "C" fn GenoBhv_SinDeg(mut a: f32) -> f32 {
    melee_core::ct!("GenoBhv_SinDeg");
    let mut q: f32 = 0.;
    let mut sgn: f32 = 0.;
    let mut w: f32 = 0.;
    let mut k: f32 = 0.;
    let mut kt: f32 = 0.;
    let mut p: f32 = 0.;
    let mut fr: f32 = 0.;
    let mut lo: f32 = 0.;
    let mut hi: f32 = 0.;
    let mut d: f32 = 0.;
    let mut t: f32 = 0.;
    let mut v: f32 = 0.;
    let mut fi: f32 = 0.;
    let mut i: i32 = 0;
    k = gb_floor(a / 360.0f32);
    kt = (k as f32 * 360.0f32) as f32;
    w = a - kt;
    if w < 90.0f32 {
        q = w;
        sgn = 1.0f32 as f32;
    } else if w < 180.0f32 {
        q = 180.0f32 - w;
        sgn = 1.0f32 as f32;
    } else if w < 270.0f32 {
        q = (w as f32 - 180.0f32) as f32;
        sgn = -1.0f32 as f32;
    } else {
        q = 360.0f32 - w;
        sgn = -1.0f32 as f32;
    }
    p = (q as f32 * 11.377778f32) as f32;
    i = p as i32;
    if i < 0 as i32 {
        i = 0 as i32 as i32;
    }
    if i > 1023 as i32 {
        i = 1023 as i32 as i32;
    }
    fi = i as f32;
    fr = p - fi;
    lo = gb_sinq_at(i as i32);
    hi = gb_sinq_at(i as i32 + 1 as i32);
    d = hi - lo;
    t = d * fr;
    v = lo + t;
    return sgn * v;
}

pub unsafe extern "C" fn GenoBhv_CosDeg(mut a: f32) -> f32 {
    melee_core::ct!("GenoBhv_CosDeg");
    let mut b: f32 = a + 90.0f32;
    return GenoBhv_SinDeg(b);
}

pub unsafe extern "C" fn GenoBhv_Stick(
    mut rx: f32,
    mut ry: f32,
    mut sx: *mut f32,
    mut sy: *mut f32,
) {
    melee_core::ct!("GenoBhv_Stick");
    let mut a: f32 = rx * rx;
    let mut b: f32 = ry * ry;
    let mut m: f32 = a + b;
    if m < GENO_BHV_DZ2 {
        *sx = 0.0f32 as f32;
        *sy = 0.0f32 as f32;
    } else {
        *sx = rx;
        *sy = ry;
    };
}

pub unsafe extern "C" fn GenoBhv_GlideParams(mut p: *mut GbGlideP) {
    melee_core::ct!("GenoBhv_GlideParams");
    (*p).max_glides = 2 as i32;
    (*p).entry_hold = 6 as i32;
    (*p).flick_window = 12 as i32;
    (*p).start_frames = 12 as i32;
    (*p).trip_frames = 30 as i32;
    (*p).land_lag = 14 as i32;
    (*p).jumpout_frames = 8 as i32;
    (*p).attack_frames = 28 as i32;
    (*p).attack_hit0 = 8 as i32;
    (*p).attack_hit1 = 14 as i32;
    (*p).attack_endlag = 14 as i32;
    (*p).attack_buf_frames = 6 as i32;
    (*p).entry_vy_max = 0.6f32;
    (*p).apex_vy = 0.25f32;
    (*p).flick_thr = 0.55f32;
    (*p).s_start = 1.50f32;
    (*p).start_accel = 0.12f32;
    (*p).s_start_cap = 2.0f32;
    (*p).theta_def = -5.0f32;
    (*p).r_down = 60.0f32;
    (*p).r_up = 85.0f32;
    (*p).turn_rate = 3.0f32;
    (*p).gk = 0.0366f32;
    (*p).drag = 0.002f32;
    (*p).s_max = 2.40f32;
    (*p).s_stall = 0.35f32;
    (*p).trip_angle = 30.0f32;
    (*p).attack_speed_keep = 0.6f32;
    (*p).chain_speed_mul = 0.75f32;
    (*p).chain_turn_mul = 1.4f32;
    (*p).attack_damage = 12 as i32;
}

pub unsafe extern "C" fn GenoBhv_GlideEntryInit(mut e: *mut GbGlideEntry) {
    melee_core::ct!("GenoBhv_GlideEntryInit");
    (*e).hold_frames = -(1 as i32) as i32;
    (*e).last_dir = 0 as i32 as i32;
    (*e).age = 1000 as i32 as i32;
}

pub unsafe extern "C" fn GenoBhv_GlideEntryStep(
    mut e: *mut GbGlideEntry,
    mut p: *const GbGlideP,
    mut midair_jumped: i32,
    mut jump_held: i32,
    mut vy: f32,
    mut sx: f32,
    mut f: f32,
    mut glides_used: i32,
    mut airborne: i32,
    mut helpless: i32,
    mut chain: i32,
) -> i32 {
    melee_core::ct!("GenoBhv_GlideEntryStep");
    let mut s: f32 = sx * f;
    let mut avy: f32 = gb_abs(vy);
    let mut cur: i32 = 0 as i32;
    let mut flick: i32 = 0 as i32;
    if s <= -(*p).flick_thr {
        cur = -(1 as i32);
    } else if s >= (*p).flick_thr {
        cur = 1 as i32;
    }
    if cur != 0 as i32 {
        if (*e).last_dir == -cur && (*e).age <= (*p).flick_window {
            flick = 1 as i32;
        }
        (*e).last_dir = cur as i32;
        (*e).age = 0 as i32 as i32;
    } else if (*e).age < 1000 as i32 {
        (*e).age += 1;
    }
    if midair_jumped == 0 || airborne == 0 {
        (*e).hold_frames = -(1 as i32) as i32;
    } else if jump_held != 0 && (*e).hold_frames >= 0 as i32 {
        (*e).hold_frames += 1;
    } else {
        (*e).hold_frames = -(1 as i32) as i32;
    }
    if airborne == 0 || helpless != 0 || glides_used >= (*p).max_glides {
        return GB_GLENTRY_NONE as i32;
    }
    if chain != 0 {
        return GB_GLENTRY_CHAIN as i32;
    }
    if (*e).hold_frames >= (*p).entry_hold && vy <= (*p).entry_vy_max {
        return GB_GLENTRY_HOLD as i32;
    }
    if flick != 0 && avy <= (*p).apex_vy {
        return GB_GLENTRY_FLICK as i32;
    }
    return GB_GLENTRY_NONE as i32;
}

pub unsafe extern "C" fn gb_glide_velocity(mut g: *mut GbGlide) {
    melee_core::ct!("gb_glide_velocity");
    let mut s_th: f32 = GenoBhv_SinDeg((*g).theta);
    let mut c_th: f32 = GenoBhv_CosDeg((*g).theta);
    let mut vx: f32 = (*g).speed * c_th;
    vx = vx * (*g).f;
    (*g).vx = vx;
    (*g).vy = (*g).speed * s_th;
}

pub unsafe extern "C" fn GenoBhv_GlideBegin(
    mut g: *mut GbGlide,
    mut p: *const GbGlideP,
    mut vx: f32,
    mut vy: f32,
    mut f: f32,
    mut sx: f32,
    mut slow: i32,
    mut helpless: i32,
    mut glides_used: i32,
) {
    melee_core::ct!("GenoBhv_GlideBegin");
    let mut a: f32 = vx * vx;
    let mut b: f32 = vy * vy;
    let mut ss: f32 = a + b;
    let mut len: f32 = sqrtf(ss as f32) as f32;
    let mut t: f32 = sx * f;
    (*g).phase = GB_GL_START as i32 as i32;
    (*g).t = 0 as i32 as i32;
    (*g).attack_buf = 0 as i32 as i32;
    (*g).slow = slow;
    (*g).helpless = helpless;
    (*g).end_reason = GB_GLE_NONE as i32 as i32;
    (*g).lag = 0 as i32 as i32;
    (*g).hit_active = 0 as i32 as i32;
    (*g).jump_now = 0 as i32 as i32;
    (*g).endlag = 0 as i32 as i32;
    (*g).speed = gb_clamp(len, 0.0f32, (*p).s_start_cap);
    (*g).theta = (*p).theta_def;
    (*g).f = f;
    if t < -(*p).flick_thr {
        (*g).f = -f;
    }
    (*g).glides_used = (glides_used as i32 + 1 as i32)
        as i32;
    (*g).vx = 0.0f32 as f32;
    (*g).vy = 0.0f32 as f32;
}

pub unsafe extern "C" fn GenoBhv_GlideGround(
    mut g: *mut GbGlide,
    mut p: *const GbGlideP,
) {
    melee_core::ct!("GenoBhv_GlideGround");
    if (*g).attack_buf > 0 as i32
        || (*g).phase == GB_GL_ATTACK as i32
    {
        (*g).phase = GB_GL_LAND_CANCEL as i32 as i32;
        (*g).lag = 0 as i32 as i32;
    } else if (*g).theta < -(*p).trip_angle {
        (*g).phase = GB_GL_TRIP as i32 as i32;
        (*g).lag = (*p).trip_frames;
    } else {
        (*g).phase = GB_GL_LAND as i32 as i32;
        (*g).lag = (*p).land_lag;
    };
}

pub unsafe extern "C" fn GenoBhv_GlideStep(
    mut g: *mut GbGlide,
    mut in_0: *const GbGlideIn,
    mut p: *const GbGlideP,
) {
    melee_core::ct!("GenoBhv_GlideStep");
    let mut t0: f32 = 0.;
    let mut t1: f32 = 0.;
    let mut d: f32 = 0.;
    let mut r: f32 = 0.;
    let mut theta_t: f32 = 0.;
    let mut rate: f32 = 0.;
    let mut s_th: f32 = 0.;
    let mut grav: f32 = 0.;
    let mut gk: f32 = 0.;
    let mut dr: f32 = 0.;
    let mut cap: f32 = 0.;
    let mut d4: f32 = 0.;
    let mut pressed: i32 = 0;
    (*g).hit_active = 0 as i32 as i32;
    (*g).jump_now = 0 as i32 as i32;
    if (*g).phase >= GB_GL_LAND as i32 {
        return;
    }
    if (*g).attack_buf > 0 as i32 {
        (*g).attack_buf -= 1;
    }
    pressed = ((*in_0).attack_pressed != 0 || (*in_0).special_pressed != 0)
        as i32;
    if pressed != 0 && (*in_0).height >= 0.0f32 {
        let mut vym: f32 = gb_abs((*g).vy);
        let mut reach: f32 = vym * (*p).attack_buf_frames as f32;
        if (*in_0).height <= reach {
            (*g).attack_buf = (*p).attack_buf_frames;
        }
    }
    if (*g).phase == GB_GL_START as i32 {
        (*g).theta = (*p).theta_def;
        (*g).speed = gb_approach((*g).speed, (*p).s_start, (*p).start_accel);
        gb_glide_velocity(g);
        (*g).t += 1;
        if (*g).t == (*p).start_frames {
            (*g).phase = GB_GL_FLY as i32 as i32;
            (*g).t = 0 as i32 as i32;
        }
    } else if (*g).phase == GB_GL_FLY as i32 {
        t0 = (*in_0).sx * (*g).f;
        t1 = 0.0f32 - (*in_0).sy;
        d = t0 + t1;
        d = gb_clamp(d, -1.0f32, 1.0f32);
        if d > 0.0f32 {
            r = d * (*p).r_down;
        } else {
            r = d * (*p).r_up;
        }
        theta_t = (*p).theta_def - r;
        rate = (*p).turn_rate;
        if (*g).slow != 0 {
            rate = rate * (*p).chain_turn_mul;
        }
        (*g).theta = gb_approach((*g).theta, theta_t, rate);
        s_th = GenoBhv_SinDeg((*g).theta);
        gk = (*p).gk;
        if (*g).slow != 0 {
            gk = gk * (*p).chain_speed_mul;
        }
        grav = gk * s_th;
        (*g).speed = (*g).speed - grav;
        dr = (*g).speed * (*p).drag;
        (*g).speed = (*g).speed - dr;
        cap = (*p).s_max;
        if (*g).slow != 0 {
            cap = cap * (*p).chain_speed_mul;
        }
        if (*g).speed > cap {
            (*g).speed = cap;
        }
        gb_glide_velocity(g);
        (*g).t += 1;
    } else if (*g).phase == GB_GL_ATTACK as i32 {
        d4 = ((*p).drag as f32 * 4.0f32) as f32;
        dr = (*g).speed * d4;
        (*g).speed = (*g).speed - dr;
        gb_glide_velocity(g);
        (*g).hit_active = ((*g).t >= (*p).attack_hit0 && (*g).t <= (*p).attack_hit1)
            as i32 as i32;
        (*g).t += 1;
        if (*in_0).ground != 0 {
            GenoBhv_GlideGround(g, p);
        } else if (*g).t >= (*p).attack_frames {
            (*g).phase = GB_GL_END as i32 as i32;
            (*g).end_reason = GB_GLE_ATTACK_END as i32 as i32;
            (*g).endlag = (*p).attack_endlag;
        }
        return;
    } else if (*g).phase == GB_GL_JUMPOUT as i32 {
        dr = (*g).speed * (*p).drag;
        (*g).speed = (*g).speed - dr;
        gb_glide_velocity(g);
        (*g).t += 1;
        if (*in_0).ground != 0 {
            GenoBhv_GlideGround(g, p);
        } else if (*g).t >= (*p).jumpout_frames {
            (*g).jump_now = 1 as i32 as i32;
            (*g).phase = GB_GL_END as i32 as i32;
            (*g).end_reason = GB_GLE_JUMPOUT as i32 as i32;
        }
        return;
    }
    if (*in_0).ground != 0 {
        GenoBhv_GlideGround(g, p);
        return;
    }
    if (*in_0).jump_pressed != 0 && (*in_0).jump_left != 0 {
        (*g).phase = GB_GL_JUMPOUT as i32 as i32;
        (*g).t = 0 as i32 as i32;
        return;
    }
    if pressed != 0 {
        (*g).phase = GB_GL_ATTACK as i32 as i32;
        (*g).t = 0 as i32 as i32;
        (*g).speed = (*g).speed * (*p).attack_speed_keep;
        gb_glide_velocity(g);
        return;
    }
    if (*g).speed < (*p).s_stall {
        (*g).phase = GB_GL_END as i32 as i32;
        (*g).end_reason = GB_GLE_STALL as i32 as i32;
        (*g).endlag = 0 as i32 as i32;
    }
}

pub unsafe extern "C" fn GenoBhv_TornadoParams(mut p: *mut GbTornadoP) {
    melee_core::ct!("GenoBhv_TornadoParams");
    (*p).start_frames = 12 as i32;
    (*p).read_period = 10 as i32;
    (*p).read_limit = 70 as i32;
    (*p).end_frames = 29 as i32;
    (*p).ledge_lock = 6 as i32;
    (*p).final_hit_ground = 3 as i32;
    (*p).final_hit_air = 2 as i32;
    (*p).loop_damage = 1 as i32;
    (*p).final_damage = 3 as i32;
    (*p).spin0 = 80.0f32;
    (*p).decay_a = 1.5f32;
    (*p).decay_b = 2.0f32;
    (*p).mash_add = 16.0f32;
    (*p).spin_max = 80.0f32;
    (*p).spin_end = 10.0f32;
    (*p).grav = 0.0239f32;
    (*p).rise_impulse = 0.9f32;
    (*p).rise_max = 1.6f32;
    (*p).fall_max = 0.50f32;
    (*p).vx_max = 1.1f32;
    (*p).vx_max_ground = 1.3f32;
    (*p).vx_accel = 0.06f32;
    (*p).jump_bonus = 1.0f32;
    (*p).start_vy_keep = 0.2f32;
    (*p).start_vx_keep = 0.5f32;
}

pub unsafe extern "C" fn GenoBhv_TornadoBegin(
    mut t: *mut GbTornado,
    mut vx: f32,
    mut vy: f32,
) {
    melee_core::ct!("GenoBhv_TornadoBegin");
    (*t).phase = GB_TN_START as i32 as i32;
    (*t).n = 0 as i32 as i32;
    (*t).k = 0 as i32 as i32;
    (*t).end_t = 0 as i32 as i32;
    (*t).press_latch = 0 as i32 as i32;
    (*t).rise_impulse = 0 as i32 as i32;
    (*t).jump_start = 0 as i32 as i32;
    (*t).hits = 0 as i32 as i32;
    (*t).hit_spawn = 0 as i32 as i32;
    (*t).final_spawn = 0 as i32 as i32;
    (*t).final_active = 0 as i32 as i32;
    (*t).ledge_locked = 0 as i32 as i32;
    (*t).helpless = 0 as i32 as i32;
    (*t).spin = 0.0f32 as f32;
    (*t).rot = 0.0f32 as f32;
    (*t).vx = vx;
    (*t).vy = vy;
}

pub unsafe extern "C" fn GenoBhv_TornadoStep(
    mut t: *mut GbTornado,
    mut in_0: *const GbTornadoIn,
    mut p: *const GbTornadoP,
) {
    melee_core::ct!("GenoBhv_TornadoStep");
    let mut tx: f32 = 0.;
    (*t).hit_spawn = 0 as i32 as i32;
    (*t).final_spawn = 0 as i32 as i32;
    (*t).final_active = 0 as i32 as i32;
    (*t).ledge_locked = 0 as i32 as i32;
    if (*t).phase == GB_TN_DONE as i32 {
        return;
    }
    if (*t).n == 0 as i32 {
        (*t).jump_start = (if (*in_0).jump_held != 0 {
            1 as i32
        } else {
            0 as i32
        }) as i32;
        (*t).vy = (*t).vy * (*p).start_vy_keep;
        (*t).vx = (*t).vx * (*p).start_vx_keep;
    }
    if (*in_0).special_pressed != 0 {
        (*t).press_latch = 1 as i32 as i32;
    }
    if (*t).phase == GB_TN_START as i32 && (*t).n >= (*p).start_frames {
        (*t).phase = GB_TN_LOOP as i32 as i32;
        (*t).k = 0 as i32 as i32;
        (*t).spin = (*p).spin0;
        (*t).rot = 0.0f32 as f32;
        if (*t).jump_start != 0 {
            (*t).vy = (*t).vy + (*p).jump_bonus;
        }
    }
    if (*t).phase == GB_TN_LOOP as i32 {
        if (*t).spin <= (*p).spin_end {
            (*t).phase = GB_TN_END as i32 as i32;
            (*t).end_t = 0 as i32 as i32;
        } else {
            (*t).ledge_locked = 1 as i32 as i32;
            if (*t).k == 0 as i32 {
                (*t).hit_spawn = 1 as i32 as i32;
                (*t).hits += 1;
            }
            (*t).rot = (*t).rot + (*t).spin;
            if (*t).rot >= 360.0f32 {
                (*t).rot = ((*t).rot as f32 - 360.0f32) as f32;
                (*t).hit_spawn = 1 as i32 as i32;
                (*t).hits += 1;
            }
            if (*t).k < (*p).read_limit
                && (*t).k % (*p).read_period == 0 as i32
                && (*t).press_latch != 0
            {
                (*t).spin = (*t).spin + (*p).mash_add;
                if (*t).spin > (*p).spin_max {
                    (*t).spin = (*p).spin_max;
                }
                (*t).rise_impulse = 1 as i32 as i32;
                (*t).press_latch = 0 as i32 as i32;
            }
            if (*t).k < (*p).read_limit {
                (*t).spin = (*t).spin - (*p).decay_a;
            } else {
                (*t).spin = (*t).spin - (*p).decay_b;
            }
            (*t).k += 1;
        }
    }
    if (*t).phase == GB_TN_END as i32 {
        (*t).end_t += 1;
        if (*t).end_t == 1 as i32 {
            (*t).final_spawn = 1 as i32 as i32;
        }
        (*t).final_active = ((*t).end_t
            <= (if (*in_0).airborne != 0 {
                (*p).final_hit_air
            } else {
                (*p).final_hit_ground
            })) as i32 as i32;
        (*t).ledge_locked = ((*t).end_t <= (*p).ledge_lock) as i32 as i32;
    }
    if (*in_0).airborne != 0 {
        (*t).vy = (*t).vy - (*p).grav;
        if (*t).rise_impulse != 0 {
            (*t).vy = (*t).vy + (*p).rise_impulse;
            if (*t).vy > (*p).rise_max {
                (*t).vy = (*p).rise_max;
            }
            (*t).rise_impulse = 0 as i32 as i32;
        }
        if (*t).vy < -(*p).fall_max {
            (*t).vy = -(*p).fall_max;
        }
        tx = (*in_0).sx * (*p).vx_max;
        (*t).vx = gb_approach((*t).vx, tx, (*p).vx_accel);
    } else {
        tx = (*in_0).sx * (*p).vx_max_ground;
        (*t).vx = gb_approach((*t).vx, tx, (*p).vx_accel);
        (*t).vy = 0.0f32 as f32;
        (*t).rise_impulse = 0 as i32 as i32;
    }
    (*t).n += 1;
    if (*t).phase == GB_TN_END as i32 && (*t).end_t >= (*p).end_frames {
        (*t).phase = GB_TN_DONE as i32 as i32;
        (*t).helpless = (if (*in_0).airborne != 0 {
            1 as i32
        } else {
            0 as i32
        }) as i32;
    }
}

pub unsafe extern "C" fn GenoBhv_DrillParams(mut p: *mut GbDrillP) {
    melee_core::ct!("GenoBhv_DrillParams");
    (*p).move0 = 16 as i32;
    (*p).ramp = 4 as i32;
    (*p).hit0 = 26 as i32;
    (*p).hit_period = 4 as i32;
    (*p).nhits = 10 as i32;
    (*p).hit_end = 65 as i32;
    (*p).final_frame = 66 as i32;
    (*p).final_end_ground = 67 as i32;
    (*p).final_end_air = 70 as i32;
    (*p).total = 96 as i32;
    (*p).rush_damage = 1 as i32;
    (*p).final_damage = 3 as i32;
    (*p).dive_default = -20.0f32;
    (*p).psi_stick = 45.0f32;
    (*p).psi_min = -60.0f32;
    (*p).psi_max = 30.0f32;
    (*p).steer_rate = 2.0f32;
    (*p).speed = 2.0f32;
    (*p).recoil_vy = 1.0f32;
    (*p).end_vx_keep = 0.2f32;
}

pub unsafe extern "C" fn GenoBhv_DrillBegin(
    mut d: *mut GbDrill,
    mut p: *const GbDrillP,
    mut f: f32,
    mut sy: f32,
) {
    melee_core::ct!("GenoBhv_DrillBegin");
    let mut a: f32 = sy * (*p).psi_stick;
    let mut psi: f32 = (*p).dive_default + a;
    (*d).psi = gb_clamp(psi, (*p).psi_min, (*p).psi_max);
    (*d).phase = GB_DR_WIND as i32 as i32;
    (*d).n = 0 as i32 as i32;
    (*d).hit_spawn = 0 as i32 as i32;
    (*d).hits = 0 as i32 as i32;
    (*d).final_active = 0 as i32 as i32;
    (*d).helpless = 0 as i32 as i32;
    (*d).done = 0 as i32 as i32;
    (*d).speed = 0.0f32 as f32;
    (*d).f = f;
    (*d).vx = 0.0f32 as f32;
    (*d).vy = 0.0f32 as f32;
}

pub unsafe extern "C" fn GenoBhv_DrillStep(
    mut d: *mut GbDrill,
    mut in_0: *const GbDrillIn,
    mut p: *const GbDrillP,
) {
    melee_core::ct!("GenoBhv_DrillStep");
    let mut n: i32 = (*d).n as i32;
    let mut a: f32 = 0.;
    let mut tgt: f32 = 0.;
    let mut step: f32 = 0.;
    let mut cs: f32 = 0.;
    let mut sn: f32 = 0.;
    (*d).hit_spawn = 0 as i32 as i32;
    (*d).final_active = 0 as i32 as i32;
    if (*d).done != 0 {
        return;
    }
    if n >= (*p).move0 && n <= (*p).hit_end {
        a = (*in_0).sy * (*p).psi_stick;
        tgt = (*p).dive_default + a;
        tgt = gb_clamp(tgt, (*p).psi_min, (*p).psi_max);
        (*d).psi = gb_approach((*d).psi, tgt, (*p).steer_rate);
    }
    if (*in_0).airborne == 0 && (*d).psi < (*in_0).slope {
        (*d).psi = (*in_0).slope;
    }
    if n < (*p).final_frame {
        if n < (*p).move0 {
            (*d).speed = 0.0f32 as f32;
        } else {
            step = (*p).speed / (*p).ramp as f32;
            (*d).speed = gb_approach((*d).speed, (*p).speed, step);
        }
        cs = GenoBhv_CosDeg((*d).psi);
        sn = GenoBhv_SinDeg((*d).psi);
        (*d).vx = (*d).speed * cs;
        (*d).vx = (*d).vx * (*d).f;
        (*d).vy = (*d).speed * sn;
        if (*in_0).airborne == 0 {
            (*d).vy = 0.0f32 as f32;
        }
    } else if n == (*p).final_frame {
        (*d).vx = (*d).vx * (*p).end_vx_keep;
        if (*in_0).airborne != 0 {
            (*d).vy = (*p).recoil_vy;
        }
    }
    if n >= (*p).hit0 && n <= (*p).hit_end
        && (n as i32 - (*p).hit0) % (*p).hit_period == 0 as i32
        && (*d).hits < (*p).nhits
    {
        (*d).hit_spawn = 1 as i32 as i32;
        (*d).hits += 1;
    }
    if n >= (*p).final_frame {
        (*d).phase = GB_DR_FINAL as i32 as i32;
        (*d).final_active = (n
            <= (if (*in_0).airborne != 0 {
                (*p).final_end_air
            } else {
                (*p).final_end_ground
            })) as i32 as i32;
    } else if n >= (*p).hit0 {
        (*d).phase = GB_DR_RUSH as i32 as i32;
    } else {
        (*d).phase = GB_DR_WIND as i32 as i32;
    }
    (*d).n += 1;
    if (*d).n >= (*p).total {
        (*d).done = 1 as i32 as i32;
        (*d).phase = GB_DR_DONE as i32 as i32;
        (*d).helpless = (if (*in_0).airborne != 0 {
            1 as i32
        } else {
            0 as i32
        }) as i32;
    }
}

pub unsafe extern "C" fn GenoBhv_CapeParams(mut p: *mut GbCapeP) {
    melee_core::ct!("GenoBhv_CapeParams");
    (*p).start = 6 as i32;
    (*p).end = 25 as i32;
    (*p).total = 40 as i32;
    (*p).turn_thr = 0.5f32;
    (*p).stall_x = 0.80f32;
    (*p).stall_y_up = 0.85f32;
    (*p).stall_vy_hi = 0.0f32;
    (*p).stall_grav = 0.015f32;
    (*p).stall_fall = 0.45f32;
    (*p).ground_x = 0.5f32;
    (*p).reflect_dmg_mul = 1.5f32;
    (*p).reflect_speed_mul = 1.2f32;
}

pub unsafe extern "C" fn GenoBhv_CapeBegin(
    mut c: *mut GbCape,
    mut f: f32,
    mut vx: f32,
    mut vy: f32,
) {
    melee_core::ct!("GenoBhv_CapeBegin");
    (*c).n = 0 as i32 as i32;
    (*c).flipped = 0 as i32 as i32;
    (*c).flip_now = 0 as i32 as i32;
    (*c).reflect = 0 as i32 as i32;
    (*c).done = 0 as i32 as i32;
    (*c).f = f;
    (*c).vx = vx;
    (*c).vy = vy;
}

pub unsafe extern "C" fn GenoBhv_CapeStep(
    mut c: *mut GbCape,
    mut in_0: *const GbCapeIn,
    mut p: *const GbCapeP,
) {
    melee_core::ct!("GenoBhv_CapeStep");
    let mut active: i32 = 0;
    let mut t: f32 = 0.;
    (*c).flip_now = 0 as i32 as i32;
    (*c).reflect = 0 as i32 as i32;
    if (*c).done != 0 {
        return;
    }
    if (*c).n == (*p).start && (*c).flipped == 0 {
        (*c).flipped = 1 as i32 as i32;
        t = (*in_0).sx * (*c).f;
        if t < -(*p).turn_thr {
            (*c).flip_now = 1 as i32 as i32;
            (*c).f = -(*c).f;
        }
    }
    active = ((*c).n >= (*p).start && (*c).n <= (*p).end) as i32;
    if active != 0 {
        (*c).reflect = 1 as i32 as i32;
        if (*in_0).airborne != 0 {
            (*c).vx = (*c).vx * (*p).stall_x;
            if (*c).vy > (*p).stall_vy_hi {
                (*c).vy = (*c).vy * (*p).stall_y_up;
            } else {
                (*c).vy = (*c).vy - (*p).stall_grav;
            }
            if (*c).vy < -(*p).stall_fall {
                (*c).vy = -(*p).stall_fall;
            }
        } else {
            (*c).vx = (*c).vx * (*p).ground_x;
        }
    }
    (*c).n += 1;
    if (*c).n >= (*p).total {
        (*c).done = 1 as i32 as i32;
    }
}

pub unsafe extern "C" fn GenoBhv_MultiDashParams(mut p: *mut GbMultiDashP) {
    melee_core::ct!("GenoBhv_MultiDashParams");
    (*p).max_seg = 4 as i32 as i32;
    (*p).aim_first = 6 as i32 as i32;
    (*p).aim_later = 4 as i32 as i32;
    (*p).dash_frames = 10 as i32 as i32;
    (*p).pause_frames = 3 as i32 as i32;
    (*p).repress_frames = 8 as i32 as i32;
    (*p).end_lag = 20 as i32 as i32;
    (*p).dash_speed = 3.2f32 as f32;
    (*p).end_keep = 0.15f32 as f32;
    (*p).aim_slow = 0.1f32 as f32;
    (*p).ay_max = 0.9397f32 as f32;
    (*p).pause_keep = 0.3f32 as f32;
}

pub unsafe extern "C" fn GenoBhv_MultiDashBegin(mut m: *mut GbMultiDash, mut f: f32) {
    melee_core::ct!("GenoBhv_MultiDashBegin");
    (*m).seg = 0 as i32 as i32;
    (*m).phase = GB_MD_AIM as i32 as i32;
    (*m).t = 0 as i32 as i32;
    (*m).repress = 0 as i32 as i32;
    (*m).hit_spawn = 0 as i32 as i32;
    (*m).helpless = 0 as i32 as i32;
    (*m).done = 0 as i32 as i32;
    (*m).f = f;
    (*m).ax = f;
    (*m).ay = 0.0f32 as f32;
    (*m).speed = 0.0f32 as f32;
    (*m).vx = 0.0f32 as f32;
    (*m).vy = 0.0f32 as f32;
    (*m).dist = 0.0f32 as f32;
}

pub unsafe extern "C" fn gb_md_fix_direction(
    mut m: *mut GbMultiDash,
    mut in_0: *const GbMultiDashIn,
    mut p: *const GbMultiDashP,
) {
    melee_core::ct!("gb_md_fix_direction");
    let mut m2: f32 = (*in_0).sx * (*in_0).sx;
    let mut n2: f32 = (*in_0).sy * (*in_0).sy;
    let mut mm: f32 = m2 + n2;
    let mut ax: f32 = 0.;
    let mut ay: f32 = 0.;
    let mut len: f32 = 0.;
    let mut l2: f32 = 0.;
    let mut l3: f32 = 0.;
    let mut ll: f32 = 0.;
    let mut l: f32 = 0.;
    let mut chk: f32 = 0.;
    if mm < GENO_BHV_DZ2 {
        ax = (*m).f;
        ay = 0.0f32 as f32;
    } else {
        len = sqrtf(mm as f32) as f32;
        ax = (*in_0).sx / len;
        ay = (*in_0).sy / len;
    }
    chk = ax * (*m).f;
    if chk < 0.0f32 {
        (*m).f = -(*m).f;
    }
    if ay > (*p).ay_max || ay < -(*p).ay_max {
        ay = gb_clamp(ay, -(*p).ay_max, (*p).ay_max);
        l3 = ay * ay;
        ll = 1.0f32 - l3;
        l = sqrtf(ll as f32) as f32;
        l2 = (*m).f;
        if ax < 0.0f32 {
            l2 = -1.0f32 as f32;
        } else if ax > 0.0f32 {
            l2 = 1.0f32 as f32;
        }
        ax = l * l2;
    }
    (*m).ax = ax;
    (*m).ay = ay;
}

pub unsafe extern "C" fn GenoBhv_MultiDashStep(
    mut m: *mut GbMultiDash,
    mut in_0: *const GbMultiDashIn,
    mut p: *const GbMultiDashP,
) {
    melee_core::ct!("GenoBhv_MultiDashStep");
    let mut aim_len: i32 = 0;
    let mut step: f32 = 0.;
    (*m).hit_spawn = 0 as i32 as i32;
    if (*m).done != 0 {
        return;
    }
    if (*in_0).special_pressed != 0 {
        if (*m).phase == GB_MD_DASH as i32
            && (*m).t >= (*p).dash_frames - (*p).repress_frames
            || (*m).phase == GB_MD_PAUSE as i32
        {
            (*m).repress = 1 as i32 as i32;
        }
    }
    if (*m).phase == GB_MD_AIM as i32 {
        aim_len = (if (*m).seg == 0 as i32 {
            (*p).aim_first
        } else {
            (*p).aim_later
        }) as i32;
        (*m).vx = (*m).vx * (*p).aim_slow;
        (*m).vy = (*m).vy * (*p).aim_slow;
        (*m).t += 1;
        if (*m).t >= aim_len {
            gb_md_fix_direction(m, in_0, p);
            (*m).phase = GB_MD_DASH as i32 as i32;
            (*m).t = 0 as i32 as i32;
            (*m).speed = 0.0f32 as f32;
            (*m).dist = 0.0f32 as f32;
        }
    } else if (*m).phase == GB_MD_DASH as i32 {
        step = ((*p).dash_speed as f32 / 3.0f32) as f32;
        (*m).speed = gb_approach((*m).speed, (*p).dash_speed, step);
        (*m).vx = (*m).speed * (*m).ax;
        (*m).vy = (*m).speed * (*m).ay;
        (*m).dist = (*m).dist + (*m).speed;
        (*m).hit_spawn = 1 as i32 as i32;
        (*m).t += 1;
        if (*m).t >= (*p).dash_frames {
            (*m).phase = GB_MD_PAUSE as i32 as i32;
            (*m).t = 0 as i32 as i32;
        }
    } else if (*m).phase == GB_MD_PAUSE as i32 {
        (*m).vx = (*m).vx * (*p).pause_keep;
        (*m).vy = (*m).vy * (*p).pause_keep;
        (*m).t += 1;
        if (*m).t >= (*p).pause_frames {
            if (*m).repress != 0
                && ((*m).seg as i32 + 1 as i32)
                    < (*p).max_seg
            {
                (*m).seg += 1;
                (*m).phase = GB_MD_AIM as i32 as i32;
                (*m).t = 0 as i32 as i32;
                (*m).repress = 0 as i32 as i32;
            } else {
                (*m).phase = GB_MD_END as i32 as i32;
                (*m).t = 0 as i32 as i32;
                (*m).vx = (*m).vx * (*p).end_keep;
                (*m).vy = (*m).vy * (*p).end_keep;
            }
        }
    } else if (*m).phase == GB_MD_END as i32 {
        (*m).t += 1;
        if (*m).t >= (*p).end_lag {
            (*m).phase = GB_MD_DONE as i32 as i32;
            (*m).done = 1 as i32 as i32;
            (*m).helpless = (if (*m).seg as i32 + 1 as i32
                == (*p).max_seg
            {
                1 as i32
            } else {
                0 as i32
            }) as i32;
        }
    }
}

pub unsafe extern "C" fn GenoBhv_AnimMotionParams(mut p: *mut GbAnimMotionP) {
    melee_core::ct!("GenoBhv_AnimMotionParams");
    (*p).rate = 1.0f32 as f32;
    (*p).scale = 1.0f32 as f32;
    (*p).grav = 0.095f32 as f32;
    (*p).mode = GB_AM_DELTA as i32 as i32;
    (*p).gravity_on = 0 as i32 as i32;
    (*p).ignore_y = 0 as i32 as i32;
    (*p).r#loop = 0 as i32 as i32;
}

pub unsafe extern "C" fn GenoBhv_AnimMotionSample(
    mut track: *const f32,
    mut n: i32,
    mut t: f32,
    mut x: *mut f32,
    mut y: *mut f32,
) {
    melee_core::ct!("GenoBhv_AnimMotionSample");
    let mut last: f32 = 0.;
    let mut fr: f32 = 0.;
    let mut ax: f32 = 0.;
    let mut bx: f32 = 0.;
    let mut dx: f32 = 0.;
    let mut tx: f32 = 0.;
    let mut ay: f32 = 0.;
    let mut by: f32 = 0.;
    let mut dy: f32 = 0.;
    let mut ty: f32 = 0.;
    let mut fi: f32 = 0.;
    let mut i: i32 = 0;
    if t <= 0.0f32 {
        *x = *track.offset(0 as i32 as isize);
        *y = *track.offset(1 as i32 as isize);
        return;
    }
    last = (n - 1 as i32) as f32;
    if t >= last {
        *x = *track
            .offset((2 as i32 * (n - 1 as i32)) as isize);
        *y = *track
            .offset(
                (2 as i32 * (n - 1 as i32)
                    + 1 as i32) as isize,
            );
        return;
    }
    i = gb_floor(t) as i32;
    fi = i as f32;
    fr = t - fi;
    ax = *track.offset((2 as i32 * i) as isize);
    bx = *track
        .offset((2 as i32 * (i + 1 as i32)) as isize);
    dx = bx - ax;
    tx = dx * fr;
    *x = ax + tx;
    ay = *track.offset((2 as i32 * i + 1 as i32) as isize);
    by = *track
        .offset(
            (2 as i32 * (i + 1 as i32)
                + 1 as i32) as isize,
        );
    dy = by - ay;
    ty = dy * fr;
    *y = ay + ty;
}

pub unsafe extern "C" fn GenoBhv_AnimMotionBegin(
    mut a: *mut GbAnimMotion,
    mut pos_x: f32,
    mut pos_y: f32,
) {
    melee_core::ct!("GenoBhv_AnimMotionBegin");
    (*a).anim_t = 0.0f32 as f32;
    (*a).vy_carry = 0.0f32 as f32;
    (*a).origin_x = pos_x;
    (*a).origin_y = pos_y;
    (*a).vx = 0.0f32 as f32;
    (*a).vy = 0.0f32 as f32;
    (*a).pos_x = pos_x;
    (*a).pos_y = pos_y;
    (*a).ended = 0 as i32 as i32;
    (*a).started = 1 as i32 as i32;
}

pub unsafe extern "C" fn GenoBhv_AnimMotionStep(
    mut a: *mut GbAnimMotion,
    mut track: *const f32,
    mut n: i32,
    mut p: *const GbAnimMotionP,
    mut facing: f32,
    mut blocked: i32,
) {
    melee_core::ct!("GenoBhv_AnimMotionStep");
    let mut t0: f32 = 0.;
    let mut t1: f32 = 0.;
    let mut x0: f32 = 0.;
    let mut y0: f32 = 0.;
    let mut x1: f32 = 0.;
    let mut y1: f32 = 0.;
    let mut dx: f32 = 0.;
    let mut dy: f32 = 0.;
    let mut last: f32 = 0.;
    let mut tx: f32 = 0.;
    let mut ty: f32 = 0.;
    let mut tgx: f32 = 0.;
    let mut tgy: f32 = 0.;
    let mut delta: f32 = 0.;
    if (*a).ended != 0 {
        return;
    }
    last = (n - 1 as i32) as f32;
    t0 = (*a).anim_t;
    t1 = (*a).anim_t + (*p).rate;
    GenoBhv_AnimMotionSample(track, n, t0, &raw mut x0, &raw mut y0);
    GenoBhv_AnimMotionSample(track, n, t1, &raw mut x1, &raw mut y1);
    dx = x1 - x0;
    dy = y1 - y0;
    dx = dx * (*p).scale;
    dy = dy * (*p).scale;
    dx = dx * facing;
    if (*p).ignore_y != 0 {
        dy = 0.0f32 as f32;
    }
    if (*p).mode == GB_AM_DELTA as i32 {
        if blocked != 0 {
            dx = 0.0f32 as f32;
        }
        (*a).pos_x = (*a).pos_x + dx;
        (*a).pos_y = (*a).pos_y + dy;
    } else {
        tx = x1 * (*p).scale;
        tx = tx * facing;
        ty = y1 * (*p).scale;
        tgx = (*a).origin_x + tx;
        tgy = (*a).origin_y + ty;
        if (*p).ignore_y != 0 {
            tgy = (*a).origin_y;
        }
        if blocked != 0 {
            delta = tgx - (*a).pos_x;
            (*a).origin_x = (*a).origin_x - delta;
            tgx = (*a).pos_x;
            dx = 0.0f32 as f32;
        }
        (*a).pos_x = tgx;
        (*a).pos_y = tgy;
    }
    (*a).vx = dx;
    (*a).vy = dy;
    if (*p).gravity_on != 0 {
        (*a).vy = (*a).vy + (*a).vy_carry;
        (*a).vy_carry = (*a).vy_carry - (*p).grav;
    }
    (*a).anim_t = t1;
    if (*a).anim_t >= last {
        if (*p).r#loop != 0 {
            (*a).anim_t = (*a).anim_t - last;
        } else {
            (*a).ended = 1 as i32 as i32;
        }
    }
}

pub unsafe extern "C" fn gb_profile_params(mut st: *mut GenoState) -> *const f32 {
    melee_core::ct!("gb_profile_params");
    let mut pr: i32 = (*st).profile as i32;
    if pr < 0 as i32 || pr >= Geno_ProfileCapacity
        || (*Geno_Params.offset(pr as isize)).is_null()
    {
        return ::core::ptr::null::<f32>();
    }
    return *Geno_Params.offset(pr as isize);
}

pub unsafe extern "C" fn gb_glide_params(mut st: *mut GenoState, mut p: *mut GbGlideP) {
    melee_core::ct!("gb_glide_params");
    let mut v: *const f32 = gb_profile_params(st);
    GenoBhv_GlideParams(p);
    if !v.is_null() {
        (*p).max_glides = *v
            .offset(GENO_BP_GLIDE_max_glides as i32 as isize) as i32;
        (*p).entry_hold = *v
            .offset(GENO_BP_GLIDE_entry_hold as i32 as isize) as i32;
        (*p).flick_window = *v
            .offset(GENO_BP_GLIDE_flick_window as i32 as isize) as i32;
        (*p).start_frames = *v
            .offset(GENO_BP_GLIDE_start_frames as i32 as isize) as i32;
        (*p).trip_frames = *v
            .offset(GENO_BP_GLIDE_trip_frames as i32 as isize) as i32;
        (*p).land_lag = *v.offset(GENO_BP_GLIDE_land_lag as i32 as isize)
            as i32;
        (*p).jumpout_frames = *v
            .offset(GENO_BP_GLIDE_jumpout_frames as i32 as isize) as i32;
        (*p).attack_frames = *v
            .offset(GENO_BP_GLIDE_attack_frames as i32 as isize) as i32;
        (*p).attack_hit0 = *v
            .offset(GENO_BP_GLIDE_attack_hit0 as i32 as isize) as i32;
        (*p).attack_hit1 = *v
            .offset(GENO_BP_GLIDE_attack_hit1 as i32 as isize) as i32;
        (*p).attack_endlag = *v
            .offset(GENO_BP_GLIDE_attack_endlag as i32 as isize) as i32;
        (*p).attack_buf_frames = *v
            .offset(GENO_BP_GLIDE_attack_buf_frames as i32 as isize)
            as i32;
        (*p).entry_vy_max = *v
            .offset(GENO_BP_GLIDE_entry_vy_max as i32 as isize);
        (*p).apex_vy = *v.offset(GENO_BP_GLIDE_apex_vy as i32 as isize);
        (*p).flick_thr = *v
            .offset(GENO_BP_GLIDE_flick_thr as i32 as isize);
        (*p).s_start = *v.offset(GENO_BP_GLIDE_s_start as i32 as isize);
        (*p).start_accel = *v
            .offset(GENO_BP_GLIDE_start_accel as i32 as isize);
        (*p).s_start_cap = *v
            .offset(GENO_BP_GLIDE_s_start_cap as i32 as isize);
        (*p).theta_def = *v
            .offset(GENO_BP_GLIDE_theta_def as i32 as isize);
        (*p).r_down = *v.offset(GENO_BP_GLIDE_r_down as i32 as isize);
        (*p).r_up = *v.offset(GENO_BP_GLIDE_r_up as i32 as isize);
        (*p).turn_rate = *v
            .offset(GENO_BP_GLIDE_turn_rate as i32 as isize);
        (*p).gk = *v.offset(GENO_BP_GLIDE_gk as i32 as isize);
        (*p).drag = *v.offset(GENO_BP_GLIDE_drag as i32 as isize);
        (*p).s_max = *v.offset(GENO_BP_GLIDE_s_max as i32 as isize);
        (*p).s_stall = *v.offset(GENO_BP_GLIDE_s_stall as i32 as isize);
        (*p).trip_angle = *v
            .offset(GENO_BP_GLIDE_trip_angle as i32 as isize);
        (*p).attack_speed_keep = *v
            .offset(GENO_BP_GLIDE_attack_speed_keep as i32 as isize);
        (*p).chain_speed_mul = *v
            .offset(GENO_BP_GLIDE_chain_speed_mul as i32 as isize);
        (*p).chain_turn_mul = *v
            .offset(GENO_BP_GLIDE_chain_turn_mul as i32 as isize);
        (*p).attack_damage = *v
            .offset(GENO_BP_GLIDE_attack_damage as i32 as isize) as i32;
    }
}

pub unsafe extern "C" fn gb_tornado_params(mut st: *mut GenoState, mut p: *mut GbTornadoP) {
    melee_core::ct!("gb_tornado_params");
    let mut v: *const f32 = gb_profile_params(st);
    GenoBhv_TornadoParams(p);
    if !v.is_null() {
        (*p).start_frames = *v
            .offset(GENO_BP_TORNADO_start_frames as i32 as isize) as i32;
        (*p).read_period = *v
            .offset(GENO_BP_TORNADO_read_period as i32 as isize) as i32;
        (*p).read_limit = *v
            .offset(GENO_BP_TORNADO_read_limit as i32 as isize) as i32;
        (*p).end_frames = *v
            .offset(GENO_BP_TORNADO_end_frames as i32 as isize) as i32;
        (*p).ledge_lock = *v
            .offset(GENO_BP_TORNADO_ledge_lock as i32 as isize) as i32;
        (*p).final_hit_ground = *v
            .offset(GENO_BP_TORNADO_final_hit_ground as i32 as isize)
            as i32;
        (*p).final_hit_air = *v
            .offset(GENO_BP_TORNADO_final_hit_air as i32 as isize) as i32;
        (*p).loop_damage = *v
            .offset(GENO_BP_TORNADO_loop_damage as i32 as isize) as i32;
        (*p).final_damage = *v
            .offset(GENO_BP_TORNADO_final_damage as i32 as isize) as i32;
        (*p).spin0 = *v.offset(GENO_BP_TORNADO_spin0 as i32 as isize);
        (*p).decay_a = *v.offset(GENO_BP_TORNADO_decay_a as i32 as isize);
        (*p).decay_b = *v.offset(GENO_BP_TORNADO_decay_b as i32 as isize);
        (*p).mash_add = *v
            .offset(GENO_BP_TORNADO_mash_add as i32 as isize);
        (*p).spin_max = *v
            .offset(GENO_BP_TORNADO_spin_max as i32 as isize);
        (*p).spin_end = *v
            .offset(GENO_BP_TORNADO_spin_end as i32 as isize);
        (*p).grav = *v.offset(GENO_BP_TORNADO_grav as i32 as isize);
        (*p).rise_impulse = *v
            .offset(GENO_BP_TORNADO_rise_impulse as i32 as isize);
        (*p).rise_max = *v
            .offset(GENO_BP_TORNADO_rise_max as i32 as isize);
        (*p).fall_max = *v
            .offset(GENO_BP_TORNADO_fall_max as i32 as isize);
        (*p).vx_max = *v.offset(GENO_BP_TORNADO_vx_max as i32 as isize);
        (*p).vx_max_ground = *v
            .offset(GENO_BP_TORNADO_vx_max_ground as i32 as isize);
        (*p).vx_accel = *v
            .offset(GENO_BP_TORNADO_vx_accel as i32 as isize);
        (*p).jump_bonus = *v
            .offset(GENO_BP_TORNADO_jump_bonus as i32 as isize);
        (*p).start_vy_keep = *v
            .offset(GENO_BP_TORNADO_start_vy_keep as i32 as isize);
        (*p).start_vx_keep = *v
            .offset(GENO_BP_TORNADO_start_vx_keep as i32 as isize);
    }
}

pub unsafe extern "C" fn gb_drill_params(mut st: *mut GenoState, mut p: *mut GbDrillP) {
    melee_core::ct!("gb_drill_params");
    let mut v: *const f32 = gb_profile_params(st);
    GenoBhv_DrillParams(p);
    if !v.is_null() {
        (*p).move0 = *v.offset(GENO_BP_DRILL_move0 as i32 as isize)
            as i32;
        (*p).ramp = *v.offset(GENO_BP_DRILL_ramp as i32 as isize) as i32;
        (*p).hit0 = *v.offset(GENO_BP_DRILL_hit0 as i32 as isize) as i32;
        (*p).hit_period = *v
            .offset(GENO_BP_DRILL_hit_period as i32 as isize) as i32;
        (*p).nhits = *v.offset(GENO_BP_DRILL_nhits as i32 as isize)
            as i32;
        (*p).hit_end = *v.offset(GENO_BP_DRILL_hit_end as i32 as isize)
            as i32;
        (*p).final_frame = *v
            .offset(GENO_BP_DRILL_final_frame as i32 as isize) as i32;
        (*p).final_end_ground = *v
            .offset(GENO_BP_DRILL_final_end_ground as i32 as isize)
            as i32;
        (*p).final_end_air = *v
            .offset(GENO_BP_DRILL_final_end_air as i32 as isize) as i32;
        (*p).total = *v.offset(GENO_BP_DRILL_total as i32 as isize)
            as i32;
        (*p).rush_damage = *v
            .offset(GENO_BP_DRILL_rush_damage as i32 as isize) as i32;
        (*p).final_damage = *v
            .offset(GENO_BP_DRILL_final_damage as i32 as isize) as i32;
        (*p).dive_default = *v
            .offset(GENO_BP_DRILL_dive_default as i32 as isize);
        (*p).psi_stick = *v
            .offset(GENO_BP_DRILL_psi_stick as i32 as isize);
        (*p).psi_min = *v.offset(GENO_BP_DRILL_psi_min as i32 as isize);
        (*p).psi_max = *v.offset(GENO_BP_DRILL_psi_max as i32 as isize);
        (*p).steer_rate = *v
            .offset(GENO_BP_DRILL_steer_rate as i32 as isize);
        (*p).speed = *v.offset(GENO_BP_DRILL_speed as i32 as isize);
        (*p).recoil_vy = *v
            .offset(GENO_BP_DRILL_recoil_vy as i32 as isize);
        (*p).end_vx_keep = *v
            .offset(GENO_BP_DRILL_end_vx_keep as i32 as isize);
    }
}

pub unsafe extern "C" fn gb_cape_params(mut st: *mut GenoState, mut p: *mut GbCapeP) {
    melee_core::ct!("gb_cape_params");
    let mut v: *const f32 = gb_profile_params(st);
    GenoBhv_CapeParams(p);
    if !v.is_null() {
        (*p).start = *v.offset(GENO_BP_CAPE_start as i32 as isize) as i32;
        (*p).end = *v.offset(GENO_BP_CAPE_end as i32 as isize) as i32;
        (*p).total = *v.offset(GENO_BP_CAPE_total as i32 as isize) as i32;
        (*p).turn_thr = *v.offset(GENO_BP_CAPE_turn_thr as i32 as isize);
        (*p).stall_x = *v.offset(GENO_BP_CAPE_stall_x as i32 as isize);
        (*p).stall_y_up = *v
            .offset(GENO_BP_CAPE_stall_y_up as i32 as isize);
        (*p).stall_vy_hi = *v
            .offset(GENO_BP_CAPE_stall_vy_hi as i32 as isize);
        (*p).stall_grav = *v
            .offset(GENO_BP_CAPE_stall_grav as i32 as isize);
        (*p).stall_fall = *v
            .offset(GENO_BP_CAPE_stall_fall as i32 as isize);
        (*p).ground_x = *v.offset(GENO_BP_CAPE_ground_x as i32 as isize);
        (*p).reflect_dmg_mul = *v
            .offset(GENO_BP_CAPE_reflect_dmg_mul as i32 as isize);
        (*p).reflect_speed_mul = *v
            .offset(GENO_BP_CAPE_reflect_speed_mul as i32 as isize);
    }
}

pub unsafe extern "C" fn gb_stick_of(
    mut fp: *mut Fighter,
    mut sx: *mut f32,
    mut sy: *mut f32,
) {
    melee_core::ct!("gb_stick_of");
    GenoBhv_Stick(
        (*fp).input.lstick[0 as i32 as usize].x,
        (*fp).input.lstick[0 as i32 as usize].y,
        sx,
        sy,
    );
}

pub unsafe extern "C" fn gb_airborne(mut fp: *mut Fighter) -> i32 {
    melee_core::ct!("gb_airborne");
    return ((*fp).ground_or_air as i32 != GA_Ground as i32)
        as i32;
}

pub unsafe extern "C" fn gb_jumps_left(mut fp: *mut Fighter) -> i32 {
    melee_core::ct!("gb_jumps_left");
    return ((*fp).co_attrs.max_jumps - (*fp).x1968_jumpsUsed as i32
        > 0 as i32) as i32;
}

pub unsafe extern "C" fn gb_state_bhv(
    mut st: *mut GenoState,
    mut s: i32,
) -> i32 {
    melee_core::ct!("gb_state_bhv");
    return if (*st).profile >= 0 as i32 && s >= 0 as i32
        && s < GENO_MAX_STATES
        && s < Geno_StateCount((*st).profile as i32)
    {
        Geno_StateBehavior((*st).profile as i32, s)
    } else {
        0 as i32
    };
}

pub unsafe extern "C" fn gb_cur_bhv(
    mut fp: *mut Fighter,
    mut st: *mut GenoState,
) -> i32 {
    melee_core::ct!("gb_cur_bhv");
    return gb_state_bhv(st, geno_cur_state(fp, st));
}

pub unsafe extern "C" fn gb_from_bhv(mut st: *mut GenoState) -> i32 {
    melee_core::ct!("gb_from_bhv");
    return gb_state_bhv(st, (*st).enter_from as i32 - GENO_MOTION_BASE);
}

pub unsafe extern "C" fn gb_to_bhv(
    mut gobj: *mut Fighter_GObj,
    mut fp: *mut Fighter,
    mut st: *mut GenoState,
    mut bhv: i32,
) {
    melee_core::ct!("gb_to_bhv");
    geno_go(
        gobj,
        fp,
        st,
        (3 as u32 & 15 as u32) << 28 as i32
            | bhv as u32 & 0xffff as u32,
    );
}

pub unsafe extern "C" fn gb_finish(
    mut gobj: *mut Fighter_GObj,
    mut fp: *mut Fighter,
    mut st: *mut GenoState,
    mut helpless: i32,
) {
    melee_core::ct!("gb_finish");
    let mut s: i32 = geno_cur_state(fp, st);
    let mut nx: u32 = if s >= 0 as i32 {
        Geno_StateNext((*st).profile as i32, s) as u32
    } else {
        GENO_NONE_T
    };
    let mut dflt: u32 = if helpless != 0 { GENO_TGT_HELPLESS } else { GENO_AUTO };
    geno_go(gobj, fp, st, if nx != GENO_NONE_T { nx } else { dflt });
}

pub unsafe extern "C" fn gb_clear_hit_lists(mut fp: *mut Fighter) {
    melee_core::ct!("gb_clear_hit_lists");
    let mut i: i32 = 0;
    i = 0 as i32;
    while i < GENO_MAX_REHIT {
        lbColl_80008440(
            (&raw mut (*fp).x914 as *mut HitCapsule).offset(i as isize)
                as *mut HitCapsule,
        );
        i += 1;
    }
}

pub unsafe extern "C" fn gb_glide_load(mut st: *mut GenoState, mut g: *mut GbGlide) {
    melee_core::ct!("gb_glide_load");
    let mut i0: i32 = (*st).move_i[0 as i32 as usize];
    (*g).phase = (i0 as i32 & 15 as i32) as i32;
    (*g).end_reason = (i0 as i32 >> 4 as i32
        & 15 as i32) as i32;
    (*g).slow = (i0 as i32 >> 8 as i32
        & 1 as i32) as i32;
    (*g).helpless = (i0 as i32 >> 9 as i32
        & 1 as i32) as i32;
    (*g).t = (*st).move_i[1 as i32 as usize];
    (*g).attack_buf = ((*st).move_i[2 as i32 as usize]
        & 255 as i32) as i32;
    (*g).endlag = ((*st).move_i[2 as i32 as usize]
        >> 8 as i32 & 255 as i32) as i32;
    (*g).lag = (*st).move_i[3 as i32 as usize];
    (*g).glides_used = (*st).move_i[7 as i32 as usize];
    (*g).theta = (*st).move_f[0 as i32 as usize];
    (*g).speed = (*st).move_f[1 as i32 as usize];
    (*g).vx = (*st).move_f[2 as i32 as usize];
    (*g).vy = (*st).move_f[3 as i32 as usize];
    (*g).hit_active = 0 as i32 as i32;
    (*g).jump_now = 0 as i32 as i32;
    (*g).f = 1.0f32 as f32;
}

pub unsafe extern "C" fn gb_glide_store(mut st: *mut GenoState, mut g: *const GbGlide) {
    melee_core::ct!("gb_glide_store");
    (*st).move_i[0 as i32 as usize] = (*g).phase
        | (*g).end_reason << 4 as i32
        | (*g).slow << 8 as i32
        | (*g).helpless << 9 as i32;
    (*st).move_i[1 as i32 as usize] = (*g).t;
    (*st).move_i[2 as i32 as usize] = (*g).attack_buf
        | (*g).endlag << 8 as i32;
    (*st).move_i[3 as i32 as usize] = (*g).lag;
    (*st).move_i[7 as i32 as usize] = (*g).glides_used;
    (*st).move_f[0 as i32 as usize] = (*g).theta;
    (*st).move_f[1 as i32 as usize] = (*g).speed;
    (*st).move_f[2 as i32 as usize] = (*g).vx;
    (*st).move_f[3 as i32 as usize] = (*g).vy;
}

pub unsafe extern "C" fn geno_glide_entry(
    mut gobj: *mut Fighter_GObj,
    mut fp: *mut Fighter,
    mut st: *mut GenoState,
) -> i32 {
    melee_core::ct!("geno_glide_entry");
    let mut p: GbGlideP = unsafe { core::mem::zeroed() };
    let mut e: GbGlideEntry = unsafe { core::mem::zeroed() };
    let mut sx: f32 = 0.;
    let mut sy: f32 = 0.;
    let mut msid: i32 = 0;
    let mut allowed: i32 = 0;
    let mut jumps: i32 = 0;
    let mut midair: i32 = 0;
    let mut path: i32 = 0;
    let mut s_start: i32 = 0;
    s_start = geno_state_with(st, GENO_BHV_GLIDE_START as i32);
    if s_start < 0 as i32 {
        return 0 as i32;
    }
    if gb_airborne(fp) == 0 {
        (*st).move_i[7 as i32 as usize] = 0 as i32 as i32;
        (*st).hold_motion = -(1 as i32) as i32;
        (*st).hold_frames = -(1 as i32) as i32;
        (*st).move_i[6 as i32 as usize] = ((1000 as i32)
            << 2 as i32 | 1 as i32) as i32;
        return 0 as i32;
    }
    if geno_cur_state(fp, st) >= 0 as i32
        || geno_param(st, GENO_BP_GLIDE_auto_entry as i32) == 0.0f32
    {
        return 0 as i32;
    }
    msid = (*fp).motion_id;
    jumps = (*fp).x1968_jumpsUsed as i32;
    if jumps >= 2 as i32
        && ((*st).hold_motion < 0 as i32
            || (*st).hold_motion >> 16 as i32 != jumps)
    {
        (*st).hold_motion = (jumps << 16 as i32
            | msid & 0xffff as i32) as i32;
        (*st).hold_frames = 0 as i32 as i32;
    }
    midair = ((*st).hold_motion >= 0 as i32
        && (*st).hold_motion >> 16 as i32 >= 2 as i32)
        as i32;
    allowed = (msid == ftCo_MS_JumpF as i32
        || msid == ftCo_MS_JumpB as i32
        || msid == ftCo_MS_JumpAerialF as i32
        || msid == ftCo_MS_JumpAerialB as i32
        || msid == ftCo_MS_Fall as i32
        || msid == ftCo_MS_FallAerial as i32
        || msid == ftCo_MS_FallAerialF as i32
        || msid == ftCo_MS_FallAerialB as i32
        || midair != 0
            && msid
                == (*st).hold_motion as i32
                    & 0xffff as i32) as i32;
    if allowed == 0 {
        return 0 as i32;
    }
    gb_glide_params(st, &raw mut p);
    gb_stick_of(fp, &raw mut sx, &raw mut sy);
    e.hold_frames = (*st).hold_frames;
    e.last_dir = (((*st).move_i[6 as i32 as usize]
        & 3 as i32) - 1 as i32) as i32;
    e.age = (*st).move_i[6 as i32 as usize] >> 2 as i32;
    path = GenoBhv_GlideEntryStep(
        &raw mut e,
        &raw mut p,
        midair as i32,
        geno_jump_held(fp) as i32,
        (*fp).self_vel.y,
        sx,
        (*fp).facing_dir as f32,
        (*st).move_i[7 as i32 as usize],
        1 as i32,
        0 as i32,
        0 as i32,
    );
    (*st).hold_frames = e.hold_frames;
    (*st).move_i[6 as i32 as usize] = ((e.age as i32)
        << 2 as i32
        | e.last_dir as i32 + 1 as i32) as i32;
    if path == GB_GLENTRY_NONE as i32 {
        return 0 as i32;
    }
    (*st).hold_motion = -(1 as i32) as i32;
    (*st).hold_frames = -(1 as i32) as i32;
    (*st).move_i[6 as i32 as usize] = ((1000 as i32)
        << 2 as i32 | 1 as i32) as i32;
    return geno_go(
        gobj,
        fp,
        st,
        (3 as u32 & 15 as u32) << 28 as i32
            | GENO_BHV_GLIDE_START as i32 as u32 & 0xffff as u32,
    );
}

pub unsafe extern "C" fn geno_glide_start_enter(
    mut gobj: *mut Fighter_GObj,
    mut fp: *mut Fighter,
    mut st: *mut GenoState,
    mut s: i32,
) {
    melee_core::ct!("geno_glide_start_enter");
    let mut p: GbGlideP = unsafe { core::mem::zeroed() };
    let mut g: GbGlide = unsafe { core::mem::zeroed() };
    let mut sx: f32 = 0.;
    let mut sy: f32 = 0.;
    gb_glide_params(st, &raw mut p);
    gb_stick_of(fp, &raw mut sx, &raw mut sy);
    GenoBhv_GlideBegin(
        &raw mut g,
        &raw mut p,
        (*fp).self_vel.x,
        (*fp).self_vel.y,
        (*fp).facing_dir as f32,
        sx,
        0 as i32,
        ((*fp).x1968_jumpsUsed as i32 >= (*fp).co_attrs.max_jumps)
            as i32,
        (*st).move_i[7 as i32 as usize],
    );
    (*fp).facing_dir = g.f as f32;
    gb_glide_store(st, &raw mut g);
}

pub unsafe extern "C" fn geno_glide_enter(
    mut gobj: *mut Fighter_GObj,
    mut fp: *mut Fighter,
    mut st: *mut GenoState,
    mut s: i32,
) {
    melee_core::ct!("geno_glide_enter");
    let mut p: GbGlideP = unsafe { core::mem::zeroed() };
    let mut g: GbGlide = unsafe { core::mem::zeroed() };
    let mut sx: f32 = 0.;
    let mut sy: f32 = 0.;
    if gb_from_bhv(st) == GENO_BHV_GLIDE_START as i32 {
        gb_glide_load(st, &raw mut g);
        g.phase = GB_GL_FLY as i32 as i32;
        g.t = 0 as i32 as i32;
        gb_glide_store(st, &raw mut g);
        return;
    }
    gb_glide_params(st, &raw mut p);
    gb_stick_of(fp, &raw mut sx, &raw mut sy);
    GenoBhv_GlideBegin(
        &raw mut g,
        &raw mut p,
        (*fp).self_vel.x,
        (*fp).self_vel.y,
        (*fp).facing_dir as f32,
        sx,
        1 as i32,
        ((*fp).x1968_jumpsUsed as i32 >= (*fp).co_attrs.max_jumps)
            as i32,
        (*st).move_i[7 as i32 as usize],
    );
    (*fp).facing_dir = g.f as f32;
    g.phase = GB_GL_FLY as i32 as i32;
    g.t = 0 as i32 as i32;
    gb_glide_store(st, &raw mut g);
}

pub unsafe extern "C" fn geno_glide_attack_enter(
    mut gobj: *mut Fighter_GObj,
    mut fp: *mut Fighter,
    mut st: *mut GenoState,
    mut s: i32,
) { melee_core::ct!("geno_glide_attack_enter");}

pub unsafe extern "C" fn geno_glide_end_enter(
    mut gobj: *mut Fighter_GObj,
    mut fp: *mut Fighter,
    mut st: *mut GenoState,
    mut s: i32,
) { melee_core::ct!("geno_glide_end_enter");}

pub unsafe extern "C" fn geno_glide_landing_enter(
    mut gobj: *mut Fighter_GObj,
    mut fp: *mut Fighter,
    mut st: *mut GenoState,
    mut s: i32,
) {
    melee_core::ct!("geno_glide_landing_enter");
    let mut lag: i32 = (*st).move_i[3 as i32 as usize];
    if lag > 0 as i32 && geno_test_capture == 0 {
        ftAnim_SetAnimRate(gobj, (0.1f32 + (*fp).x2EC) / lag as f32);
    }
}

pub unsafe extern "C" fn gb_glide_tick(mut gobj: *mut Fighter_GObj) {
    melee_core::ct!("gb_glide_tick");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj as *mut HSD_GObj)
        as *mut Fighter;
    let mut st: *mut GenoState = geno_state(fp);
    let mut p: GbGlideP = unsafe { core::mem::zeroed() };
    let mut g: GbGlide = unsafe { core::mem::zeroed() };
    let mut in_0: GbGlideIn = unsafe { core::mem::zeroed() };
    let mut bhv: i32 = 0;
    gb_glide_load(st, &raw mut g);
    if g.phase >= GB_GL_END as i32 {
        return;
    }
    gb_glide_params(st, &raw mut p);
    gb_stick_of(fp, &raw mut in_0.sx, &raw mut in_0.sy);
    in_0.jump_pressed = ((*fp).input.pressed_buttons
        & ((1 as i32) << 10 as i32
            | (1 as i32) << 11 as i32) as HSD_Pad
        != 0 as HSD_Pad) as i32 as i32;
    in_0.attack_pressed = ((*fp).input.pressed_buttons
        & ((1 as i32) << 8 as i32) as HSD_Pad
        != 0 as HSD_Pad) as i32 as i32;
    in_0.special_pressed = ((*fp).input.pressed_buttons
        & ((1 as i32) << 9 as i32) as HSD_Pad
        != 0 as HSD_Pad) as i32 as i32;
    in_0.jump_left = gb_jumps_left(fp) as i32;
    in_0.ground = 0 as i32 as i32;
    in_0.height = -1.0f32 as f32;
    g.f = (if (*fp).facing_dir < 0.0f32 { -1.0f32 } else { 1.0f32 }) as f32;
    GenoBhv_GlideStep(&raw mut g, &raw mut in_0, &raw mut p);
    gb_glide_store(st, &raw mut g);
    bhv = gb_cur_bhv(fp, st);
    if g.phase == GB_GL_FLY as i32
        && bhv != GENO_BHV_GLIDE as i32
    {
        gb_to_bhv(gobj, fp, st, GENO_BHV_GLIDE as i32);
    } else if g.phase == GB_GL_ATTACK as i32
        && bhv != GENO_BHV_GLIDE_ATTACK as i32
    {
        gb_to_bhv(gobj, fp, st, GENO_BHV_GLIDE_ATTACK as i32);
    } else if g.phase == GB_GL_JUMPOUT as i32
        && bhv != GENO_BHV_GLIDE_END as i32
    {
        gb_to_bhv(gobj, fp, st, GENO_BHV_GLIDE_END as i32);
    } else if g.phase == GB_GL_END as i32 {
        if g.end_reason == GB_GLE_JUMPOUT as i32 {
            if gb_jumps_left(fp) != 0 && geno_test_capture == 0 {
                ftCo_JumpAerial_Enter_Basic(gobj);
            } else {
                gb_to_bhv(gobj, fp, st, GENO_BHV_GLIDE_END as i32);
            }
        } else if g.helpless != 0 {
            geno_go(gobj, fp, st, GENO_TGT_HELPLESS);
        } else if bhv != GENO_BHV_GLIDE_END as i32 {
            gb_to_bhv(gobj, fp, st, GENO_BHV_GLIDE_END as i32);
        }
    }
}

pub unsafe extern "C" fn geno_glide_start_anim(mut gobj: *mut Fighter_GObj) {
    melee_core::ct!("geno_glide_start_anim");
    gb_glide_tick(gobj);
}

pub unsafe extern "C" fn geno_glide_anim(mut gobj: *mut Fighter_GObj) {
    melee_core::ct!("geno_glide_anim");
    gb_glide_tick(gobj);
}

pub unsafe extern "C" fn geno_glide_after_anim(mut gobj: *mut Fighter_GObj) {
    melee_core::ct!("geno_glide_after_anim");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj as *mut HSD_GObj)
        as *mut Fighter;
    let mut st: *mut GenoState = geno_state(fp);
    let mut g: GbGlide = unsafe { core::mem::zeroed() };
    gb_glide_load(st, &raw mut g);
    if g.phase == GB_GL_ATTACK as i32
        || g.phase == GB_GL_JUMPOUT as i32
    {
        gb_glide_tick(gobj);
    } else {
        geno_cb_anim_next(gobj);
    };
}

pub unsafe extern "C" fn geno_glide_iasa(mut gobj: *mut Fighter_GObj) { melee_core::ct!("geno_glide_iasa");}

pub unsafe extern "C" fn gb_glide_phys(mut gobj: *mut Fighter_GObj) {
    melee_core::ct!("gb_glide_phys");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj as *mut HSD_GObj)
        as *mut Fighter;
    let mut g: GbGlide = unsafe { core::mem::zeroed() };
    gb_glide_load(geno_state(fp), &raw mut g);
    if g.phase <= GB_GL_JUMPOUT as i32 {
        (*fp).self_vel.x = g.vx;
        (*fp).self_vel.y = g.vy;
    } else {
        geno_cb_phys_air(gobj);
    };
}

pub unsafe extern "C" fn geno_glide_start_phys(mut gobj: *mut Fighter_GObj) {
    melee_core::ct!("geno_glide_start_phys");
    gb_glide_phys(gobj);
}

pub unsafe extern "C" fn geno_glide_phys(mut gobj: *mut Fighter_GObj) {
    melee_core::ct!("geno_glide_phys");
    gb_glide_phys(gobj);
}

pub unsafe extern "C" fn geno_glide_attack_phys(mut gobj: *mut Fighter_GObj) {
    melee_core::ct!("geno_glide_attack_phys");
    gb_glide_phys(gobj);
}

pub unsafe extern "C" fn geno_glide_end_phys(mut gobj: *mut Fighter_GObj) {
    melee_core::ct!("geno_glide_end_phys");
    gb_glide_phys(gobj);
}

pub unsafe extern "C" fn gb_glide_land(
    mut gobj: *mut Fighter_GObj,
    mut fp: *mut Fighter,
    mut st: *mut GenoState,
    mut s: i32,
) -> i32 {
    melee_core::ct!("gb_glide_land");
    let mut bhv: i32 = gb_state_bhv(st, s);
    let mut p: GbGlideP = unsafe { core::mem::zeroed() };
    let mut g: GbGlide = unsafe { core::mem::zeroed() };
    if bhv != GENO_BHV_GLIDE_START as i32
        && bhv != GENO_BHV_GLIDE as i32
        && bhv != GENO_BHV_GLIDE_ATTACK as i32
        && bhv != GENO_BHV_GLIDE_END as i32
    {
        return 0 as i32;
    }
    gb_glide_params(st, &raw mut p);
    gb_glide_load(st, &raw mut g);
    GenoBhv_GlideGround(&raw mut g, &raw mut p);
    (*st).move_i[3 as i32 as usize] = g.lag;
    gb_glide_store(st, &raw mut g);
    if g.phase == GB_GL_LAND_CANCEL as i32 {
        ftCommon_8007D7FC(fp);
        geno_go(gobj, fp, st, GENO_AUTO);
    } else if g.helpless != 0 {
        ftCo_LandingFallSpecial_Enter_Basic(gobj);
    } else if g.phase == GB_GL_TRIP as i32 {
        ftCo_LandingFallSpecial_Enter(gobj, 1, g.lag as f32);
    } else if geno_state_with(st, GENO_BHV_GLIDE_LANDING as i32)
        >= 0 as i32
    {
        ftCommon_8007D7FC(fp);
        gb_to_bhv(gobj, fp, st, GENO_BHV_GLIDE_LANDING as i32);
    } else {
        ftCo_LandingFallSpecial_Enter(gobj, 1, g.lag as f32);
    }
    return 1 as i32;
}

pub unsafe extern "C" fn geno_glide_coll(mut gobj: *mut Fighter_GObj) {
    melee_core::ct!("geno_glide_coll");
    ft_80082C74(gobj, Some(geno_land as unsafe extern "C" fn(*mut Fighter_GObj) -> ()));
}

pub unsafe extern "C" fn gb_tornado_load(mut st: *mut GenoState, mut t: *mut GbTornado) {
    melee_core::ct!("gb_tornado_load");
    let mut i0: i32 = (*st).move_i[0 as i32 as usize];
    (*t).phase = (i0 as i32 & 15 as i32) as i32;
    (*t).press_latch = (i0 as i32 >> 4 as i32
        & 1 as i32) as i32;
    (*t).rise_impulse = (i0 as i32 >> 5 as i32
        & 1 as i32) as i32;
    (*t).jump_start = (i0 as i32 >> 6 as i32
        & 1 as i32) as i32;
    (*t).n = (*st).move_i[1 as i32 as usize];
    (*t).k = (*st).move_i[2 as i32 as usize];
    (*t).end_t = (*st).move_i[3 as i32 as usize];
    (*t).hits = (*st).move_i[5 as i32 as usize];
    (*t).spin = (*st).move_f[0 as i32 as usize];
    (*t).rot = (*st).move_f[1 as i32 as usize];
}

pub unsafe extern "C" fn gb_tornado_store(mut st: *mut GenoState, mut t: *const GbTornado) {
    melee_core::ct!("gb_tornado_store");
    (*st).move_i[0 as i32 as usize] = (*t).phase
        | (*t).press_latch << 4 as i32
        | (*t).rise_impulse << 5 as i32
        | (*t).jump_start << 6 as i32;
    (*st).move_i[1 as i32 as usize] = (*t).n;
    (*st).move_i[2 as i32 as usize] = (*t).k;
    (*st).move_i[3 as i32 as usize] = (*t).end_t;
    (*st).move_i[5 as i32 as usize] = (*t).hits;
    (*st).move_f[0 as i32 as usize] = (*t).spin;
    (*st).move_f[1 as i32 as usize] = (*t).rot;
}

pub unsafe extern "C" fn geno_tornado_enter(
    mut gobj: *mut Fighter_GObj,
    mut fp: *mut Fighter,
    mut st: *mut GenoState,
    mut s: i32,
) {
    melee_core::ct!("geno_tornado_enter");
    let mut t: GbTornado = unsafe { core::mem::zeroed() };
    let mut air: i32 = gb_airborne(fp);
    GenoBhv_TornadoBegin(
        &raw mut t,
        if air != 0 { (*fp).self_vel.x } else { (*fp).gr_vel as f32 },
        if air != 0 { (*fp).self_vel.y } else { 0.0f32 },
    );
    gb_tornado_store(st, &raw mut t);
}

pub unsafe extern "C" fn geno_tornado_anim(mut gobj: *mut Fighter_GObj) {
    melee_core::ct!("geno_tornado_anim");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj as *mut HSD_GObj)
        as *mut Fighter;
    let mut st: *mut GenoState = geno_state(fp);
    let mut p: GbTornadoP = unsafe { core::mem::zeroed() };
    let mut t: GbTornado = unsafe { core::mem::zeroed() };
    let mut in_0: GbTornadoIn = unsafe { core::mem::zeroed() };
    let mut sy: f32 = 0.;
    let mut air: i32 = gb_airborne(fp);
    gb_tornado_params(st, &raw mut p);
    gb_tornado_load(st, &raw mut t);
    gb_stick_of(fp, &raw mut in_0.sx, &raw mut sy);
    in_0.special_pressed = ((*fp).input.pressed_buttons
        & ((1 as i32) << 9 as i32) as HSD_Pad
        != 0 as HSD_Pad) as i32 as i32;
    in_0.jump_held = geno_jump_held(fp) as i32;
    in_0.airborne = air as i32;
    t.vx = (if air != 0 {
        (*fp).self_vel.x as f32
    } else {
        (*fp).gr_vel
    }) as f32;
    t.vy = (if air != 0 { (*fp).self_vel.y as f32 } else { 0.0f32 })
        as f32;
    GenoBhv_TornadoStep(&raw mut t, &raw mut in_0, &raw mut p);
    if air != 0 {
        (*fp).self_vel.x = t.vx;
        (*fp).self_vel.y = t.vy;
    } else {
        (*fp).gr_vel = t.vx as f32;
    }
    if t.hit_spawn != 0 || t.final_spawn != 0 {
        gb_clear_hit_lists(fp);
    }
    gb_tornado_store(st, &raw mut t);
    if t.phase == GB_TN_DONE as i32 {
        gb_finish(gobj, fp, st, t.helpless as i32);
    }
}

pub unsafe extern "C" fn geno_tornado_phys(mut gobj: *mut Fighter_GObj) {
    melee_core::ct!("geno_tornado_phys");
    if gb_airborne(HSD_GObjGetUserData(gobj as *mut HSD_GObj) as *mut Fighter) == 0 {
        ftCommon_SetSelfMovementFromGroundedMovement(gobj);
    }
}

pub unsafe extern "C" fn gb_drill_load(mut st: *mut GenoState, mut d: *mut GbDrill) {
    melee_core::ct!("gb_drill_load");
    (*d).phase = (*st).move_i[0 as i32 as usize];
    (*d).n = (*st).move_i[1 as i32 as usize];
    (*d).hits = (*st).move_i[2 as i32 as usize];
    (*d).done = (*st).move_i[3 as i32 as usize];
    (*d).psi = (*st).move_f[0 as i32 as usize];
    (*d).speed = (*st).move_f[1 as i32 as usize];
    (*d).vx = (*st).move_f[2 as i32 as usize];
    (*d).vy = (*st).move_f[3 as i32 as usize];
    (*d).f = 1.0f32 as f32;
}

pub unsafe extern "C" fn gb_drill_store(mut st: *mut GenoState, mut d: *const GbDrill) {
    melee_core::ct!("gb_drill_store");
    (*st).move_i[0 as i32 as usize] = (*d).phase;
    (*st).move_i[1 as i32 as usize] = (*d).n;
    (*st).move_i[2 as i32 as usize] = (*d).hits;
    (*st).move_i[3 as i32 as usize] = (*d).done;
    (*st).move_f[0 as i32 as usize] = (*d).psi;
    (*st).move_f[1 as i32 as usize] = (*d).speed;
    (*st).move_f[2 as i32 as usize] = (*d).vx;
    (*st).move_f[3 as i32 as usize] = (*d).vy;
}

pub unsafe extern "C" fn gb_drill_begin(mut fp: *mut Fighter, mut st: *mut GenoState) {
    melee_core::ct!("gb_drill_begin");
    let mut p: GbDrillP = unsafe { core::mem::zeroed() };
    let mut d: GbDrill = unsafe { core::mem::zeroed() };
    let mut sx: f32 = 0.;
    let mut sy: f32 = 0.;
    gb_drill_params(st, &raw mut p);
    gb_stick_of(fp, &raw mut sx, &raw mut sy);
    GenoBhv_DrillBegin(
        &raw mut d,
        &raw mut p,
        if (*fp).facing_dir < 0.0f32 { -1.0f32 } else { 1.0f32 },
        sy,
    );
    gb_drill_store(st, &raw mut d);
}

pub unsafe extern "C" fn gb_drill_continues(mut st: *mut GenoState) -> i32 {
    melee_core::ct!("gb_drill_continues");
    let mut b: i32 = gb_from_bhv(st);
    return (b == GENO_BHV_DRILL_START as i32
        || b == GENO_BHV_DRILL as i32
        || b == GENO_BHV_DRILL_END as i32) as i32;
}

pub unsafe extern "C" fn geno_drill_start_enter(
    mut gobj: *mut Fighter_GObj,
    mut fp: *mut Fighter,
    mut st: *mut GenoState,
    mut s: i32,
) {
    melee_core::ct!("geno_drill_start_enter");
    gb_drill_begin(fp, st);
}

pub unsafe extern "C" fn geno_drill_enter(
    mut gobj: *mut Fighter_GObj,
    mut fp: *mut Fighter,
    mut st: *mut GenoState,
    mut s: i32,
) {
    melee_core::ct!("geno_drill_enter");
    if gb_drill_continues(st) == 0 {
        gb_drill_begin(fp, st);
    }
}

pub unsafe extern "C" fn geno_drill_end_enter(
    mut gobj: *mut Fighter_GObj,
    mut fp: *mut Fighter,
    mut st: *mut GenoState,
    mut s: i32,
) {
    melee_core::ct!("geno_drill_end_enter");
    if gb_drill_continues(st) == 0 {
        gb_drill_begin(fp, st);
    }
}

pub unsafe extern "C" fn gb_drill_tick(mut gobj: *mut Fighter_GObj) {
    melee_core::ct!("gb_drill_tick");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj as *mut HSD_GObj)
        as *mut Fighter;
    let mut st: *mut GenoState = geno_state(fp);
    let mut p: GbDrillP = unsafe { core::mem::zeroed() };
    let mut d: GbDrill = unsafe { core::mem::zeroed() };
    let mut in_0: GbDrillIn = unsafe { core::mem::zeroed() };
    let mut sx: f32 = 0.;
    let mut prev_n: i32 = 0;
    let mut bhv: i32 = 0;
    let mut want: i32 = 0;
    let mut air: i32 = gb_airborne(fp);
    gb_drill_params(st, &raw mut p);
    gb_drill_load(st, &raw mut d);
    if d.done != 0 {
        return;
    }
    d.f = (if (*fp).facing_dir < 0.0f32 { -1.0f32 } else { 1.0f32 }) as f32;
    gb_stick_of(fp, &raw mut sx, &raw mut in_0.sy);
    in_0.airborne = air as i32;
    in_0.slope = 0.0f32 as f32;
    prev_n = d.n as i32;
    GenoBhv_DrillStep(&raw mut d, &raw mut in_0, &raw mut p);
    if prev_n <= p.final_frame {
        if air != 0 {
            (*fp).self_vel.x = d.vx;
            (*fp).self_vel.y = d.vy;
        } else {
            (*fp).gr_vel = d.vx as f32;
        }
    }
    if d.hit_spawn != 0 {
        gb_clear_hit_lists(fp);
    }
    gb_drill_store(st, &raw mut d);
    if d.done != 0 {
        gb_finish(gobj, fp, st, d.helpless as i32);
        return;
    }
    bhv = gb_cur_bhv(fp, st);
    want = if prev_n < p.hit0 {
        GENO_BHV_DRILL_START as i32
    } else if prev_n < p.final_frame {
        GENO_BHV_DRILL as i32
    } else {
        GENO_BHV_DRILL_END as i32
    };
    if want == GENO_BHV_DRILL_START as i32
        && geno_state_with(st, want) < 0 as i32
    {
        want = GENO_BHV_DRILL as i32;
    }
    if want != bhv && geno_state_with(st, want) >= 0 as i32 {
        gb_to_bhv(gobj, fp, st, want);
    }
}

pub unsafe extern "C" fn gb_drill_phys(mut gobj: *mut Fighter_GObj) {
    melee_core::ct!("gb_drill_phys");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj as *mut HSD_GObj)
        as *mut Fighter;
    let mut p: GbDrillP = unsafe { core::mem::zeroed() };
    let mut d: GbDrill = unsafe { core::mem::zeroed() };
    gb_drill_params(geno_state(fp), &raw mut p);
    gb_drill_load(geno_state(fp), &raw mut d);
    if gb_airborne(fp) == 0 {
        ftCommon_SetSelfMovementFromGroundedMovement(gobj);
    } else if d.n > p.final_frame as i32 + 1 as i32 {
        ftCommon_Fall(fp, (*fp).co_attrs.gravity, (*fp).co_attrs.terminal_velocity);
    }
}

pub unsafe extern "C" fn geno_drill_anim(mut gobj: *mut Fighter_GObj) {
    melee_core::ct!("geno_drill_anim");
    gb_drill_tick(gobj);
}

pub unsafe extern "C" fn geno_drill_end_anim(mut gobj: *mut Fighter_GObj) {
    melee_core::ct!("geno_drill_end_anim");
    gb_drill_tick(gobj);
}

pub unsafe extern "C" fn geno_drill_phys(mut gobj: *mut Fighter_GObj) {
    melee_core::ct!("geno_drill_phys");
    gb_drill_phys(gobj);
}

pub unsafe extern "C" fn geno_drill_end_phys(mut gobj: *mut Fighter_GObj) {
    melee_core::ct!("geno_drill_end_phys");
    gb_drill_phys(gobj);
}

pub unsafe extern "C" fn geno_drill_start_phys(mut gobj: *mut Fighter_GObj) {
    melee_core::ct!("geno_drill_start_phys");
    gb_drill_tick(gobj);
    gb_drill_phys(gobj);
}

pub unsafe extern "C" fn geno_drill_coll(mut gobj: *mut Fighter_GObj) {
    melee_core::ct!("geno_drill_coll");
    geno_coll_both(gobj);
}

pub unsafe extern "C" fn geno_drill_start_coll(mut gobj: *mut Fighter_GObj) {
    melee_core::ct!("geno_drill_start_coll");
    geno_coll_both(gobj);
}

pub unsafe extern "C" fn gb_cape_reflect(
    mut gobj: *mut Fighter_GObj,
    mut fp: *mut Fighter,
    mut p: *const GbCapeP,
    mut on: i32,
) {
    melee_core::ct!("gb_cape_reflect");
    let mut d: ReflectDesc = unsafe { core::mem::zeroed() };
    if on == 0 {
        (*fp).set_reflecting(0 as u8 as u8);
        return;
    }
    if (*fp).reflecting() != 0 {
        return;
    }
    if geno_test_capture != 0 {
        (*fp).set_reflecting(1 as u8 as u8);
        return;
    }
    geno_zero(
        &raw mut d as *mut c_void,
        ::core::mem::size_of::<ReflectDesc>() as i32,
    );
    d.x0_bone_id = 1 as u32;
    d.x4_max_damage = 999 as i32 as i32;
    d.x14_size = 12.0f32;
    d.x18_damage_mul = (*p).reflect_dmg_mul as f32;
    d.x1C_speed_mul = (*p).reflect_speed_mul as f32;
    d.x20_behavior = 0 as u8;
    ftColl_CreateReflectHit(gobj, &raw mut d, None);
}

pub unsafe extern "C" fn geno_cape_enter(
    mut gobj: *mut Fighter_GObj,
    mut fp: *mut Fighter,
    mut st: *mut GenoState,
    mut s: i32,
) {
    melee_core::ct!("geno_cape_enter");
    (*st).move_i[0 as i32 as usize] = 0 as i32 as i32;
}

pub unsafe extern "C" fn geno_cape_anim(mut gobj: *mut Fighter_GObj) {
    melee_core::ct!("geno_cape_anim");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj as *mut HSD_GObj)
        as *mut Fighter;
    let mut st: *mut GenoState = geno_state(fp);
    let mut p: GbCapeP = unsafe { core::mem::zeroed() };
    let mut c: GbCape = unsafe { core::mem::zeroed() };
    let mut in_0: GbCapeIn = unsafe { core::mem::zeroed() };
    let mut sy: f32 = 0.;
    let mut air: i32 = gb_airborne(fp);
    gb_cape_params(st, &raw mut p);
    c.n = ((*st).move_i[0 as i32 as usize] & 255 as i32)
        as i32;
    c.flipped = ((*st).move_i[0 as i32 as usize]
        >> 8 as i32 & 1 as i32) as i32;
    c.done = 0 as i32 as i32;
    c.f = (if (*fp).facing_dir < 0.0f32 { -1.0f32 } else { 1.0f32 }) as f32;
    c.vx = (if air != 0 {
        (*fp).self_vel.x as f32
    } else {
        (*fp).gr_vel
    }) as f32;
    c.vy = (if air != 0 { (*fp).self_vel.y as f32 } else { 0.0f32 })
        as f32;
    gb_stick_of(fp, &raw mut in_0.sx, &raw mut sy);
    in_0.airborne = air as i32;
    GenoBhv_CapeStep(&raw mut c, &raw mut in_0, &raw mut p);
    if c.flip_now != 0 {
        (*fp).facing_dir = c.f as f32;
    }
    if air != 0 {
        (*fp).self_vel.x = c.vx;
        (*fp).self_vel.y = c.vy;
    } else {
        (*fp).gr_vel = c.vx as f32;
    }
    gb_cape_reflect(gobj, fp, &raw mut p, c.reflect as i32);
    (*st).move_i[0 as i32 as usize] = c.n
        | c.flipped << 8 as i32;
    if c.done != 0 {
        gb_finish(gobj, fp, st, 0 as i32);
    }
}

pub unsafe extern "C" fn geno_cape_phys(mut gobj: *mut Fighter_GObj) {
    melee_core::ct!("geno_cape_phys");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj as *mut HSD_GObj)
        as *mut Fighter;
    let mut p: GbCapeP = unsafe { core::mem::zeroed() };
    let mut n: i32 = (*geno_state(fp))
        .move_i[0 as i32 as usize] & 255 as i32;
    let mut stalled: i32 = 0;
    gb_cape_params(geno_state(fp), &raw mut p);
    stalled = (n > p.start && n <= p.end as i32 + 1 as i32)
        as i32;
    if gb_airborne(fp) != 0 {
        if stalled == 0 {
            geno_cb_phys_air(gobj);
        }
    } else if stalled != 0 {
        ftCommon_SetSelfMovementFromGroundedMovement(gobj);
    } else {
        ft_80084F3C(gobj);
    };
}

pub unsafe extern "C" fn geno_cape_coll(mut gobj: *mut Fighter_GObj) {
    melee_core::ct!("geno_cape_coll");
    geno_coll_both(gobj);
}

pub unsafe extern "C" fn geno_cape_after_coll(mut gobj: *mut Fighter_GObj) {
    melee_core::ct!("geno_cape_after_coll");
    geno_cb_coll_anim_motion(gobj);
}

pub unsafe extern "C" fn gb_md_load(mut st: *mut GenoState, mut m: *mut GbMultiDash) {
    melee_core::ct!("gb_md_load");
    let mut w: i32 = (*st).move_i[0 as i32 as usize];
    (*m).phase = (w as i32 & 15 as i32) as i32;
    (*m).seg = (w as i32 >> 4 as i32
        & 15 as i32) as i32;
    (*m).repress = (w as i32 >> 8 as i32
        & 1 as i32) as i32;
    (*m).helpless = (w as i32 >> 9 as i32
        & 1 as i32) as i32;
    (*m).done = (w as i32 >> 10 as i32
        & 1 as i32) as i32;
    (*m).t = (*st).move_i[1 as i32 as usize];
    (*m).hit_spawn = 0 as i32 as i32;
    (*m).ax = (*st).move_f[0 as i32 as usize];
    (*m).ay = (*st).move_f[1 as i32 as usize];
    (*m).f = (*st).move_f[2 as i32 as usize];
    (*m).speed = (*st).move_f[3 as i32 as usize];
    (*m).vx = (*st).move_f[4 as i32 as usize];
    (*m).vy = (*st).move_f[5 as i32 as usize];
    (*m).dist = (*st).move_f[6 as i32 as usize];
}

pub unsafe extern "C" fn gb_md_store(mut st: *mut GenoState, mut m: *const GbMultiDash) {
    melee_core::ct!("gb_md_store");
    (*st).move_i[0 as i32 as usize] = (*m).phase
        | (*m).seg << 4 as i32 | (*m).repress << 8 as i32
        | (*m).helpless << 9 as i32
        | (*m).done << 10 as i32;
    (*st).move_i[1 as i32 as usize] = (*m).t;
    (*st).move_f[0 as i32 as usize] = (*m).ax;
    (*st).move_f[1 as i32 as usize] = (*m).ay;
    (*st).move_f[2 as i32 as usize] = (*m).f;
    (*st).move_f[3 as i32 as usize] = (*m).speed;
    (*st).move_f[4 as i32 as usize] = (*m).vx;
    (*st).move_f[5 as i32 as usize] = (*m).vy;
    (*st).move_f[6 as i32 as usize] = (*m).dist;
}

pub unsafe extern "C" fn geno_multi_dash_enter(
    mut gobj: *mut Fighter_GObj,
    mut fp: *mut Fighter,
    mut st: *mut GenoState,
    mut s: i32,
) {
    melee_core::ct!("geno_multi_dash_enter");
    let mut m: GbMultiDash = unsafe { core::mem::zeroed() };
    let mut air: i32 = gb_airborne(fp);
    GenoBhv_MultiDashBegin(
        &raw mut m,
        if (*fp).facing_dir < 0.0f32 { -1.0f32 } else { 1.0f32 },
    );
    m.vx = (if air != 0 {
        (*fp).self_vel.x as f32
    } else {
        (*fp).gr_vel
    }) as f32;
    m.vy = (if air != 0 { (*fp).self_vel.y as f32 } else { 0.0f32 })
        as f32;
    gb_md_store(st, &raw mut m);
}

pub unsafe extern "C" fn geno_multi_dash_anim(mut gobj: *mut Fighter_GObj) {
    melee_core::ct!("geno_multi_dash_anim");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj as *mut HSD_GObj)
        as *mut Fighter;
    let mut st: *mut GenoState = geno_state(fp);
    let mut p: GbMultiDashP = unsafe { core::mem::zeroed() };
    let mut m: GbMultiDash = unsafe { core::mem::zeroed() };
    let mut in_0: GbMultiDashIn = unsafe { core::mem::zeroed() };
    let mut air: i32 = gb_airborne(fp);
    GenoBhv_MultiDashParams(&raw mut p);
    gb_md_load(st, &raw mut m);
    gb_stick_of(fp, &raw mut in_0.sx, &raw mut in_0.sy);
    in_0.special_pressed = ((*fp).input.pressed_buttons
        & ((1 as i32) << 9 as i32) as HSD_Pad
        != 0 as HSD_Pad) as i32 as i32;
    GenoBhv_MultiDashStep(&raw mut m, &raw mut in_0, &raw mut p);
    (*fp).facing_dir = m.f as f32;
    if air != 0 {
        (*fp).self_vel.x = m.vx;
        (*fp).self_vel.y = m.vy;
    } else {
        (*fp).gr_vel = m.vx as f32;
    }
    if m.hit_spawn != 0 && m.t == 1 as i32 {
        gb_clear_hit_lists(fp);
    }
    gb_md_store(st, &raw mut m);
    if m.done != 0 {
        gb_finish(gobj, fp, st, (air != 0 && m.helpless != 0) as i32);
    }
}

pub unsafe extern "C" fn geno_multi_dash_phys(mut gobj: *mut Fighter_GObj) {
    melee_core::ct!("geno_multi_dash_phys");
    if gb_airborne(HSD_GObjGetUserData(gobj as *mut HSD_GObj) as *mut Fighter) == 0 {
        ftCommon_SetSelfMovementFromGroundedMovement(gobj);
    }
}

pub unsafe extern "C" fn geno_multi_dash_coll(mut gobj: *mut Fighter_GObj) {
    melee_core::ct!("geno_multi_dash_coll");
    geno_coll_both(gobj);
}

pub unsafe extern "C" fn GenoBhv_AnimMotionFrameDelta(
    mut fwd: f32,
    mut up: f32,
    mut lr: f32,
    mut dx: *mut f32,
    mut dy: *mut f32,
) {
    melee_core::ct!("GenoBhv_AnimMotionFrameDelta");
    let mut track: [f32; 4] = [0.; 4];
    let mut p: GbAnimMotionP = unsafe { core::mem::zeroed() };
    let mut a: GbAnimMotion = unsafe { core::mem::zeroed() };
    track[0 as i32 as usize] = 0.0f32 as f32;
    track[1 as i32 as usize] = 0.0f32 as f32;
    track[2 as i32 as usize] = fwd;
    track[3 as i32 as usize] = up;
    GenoBhv_AnimMotionParams(&raw mut p);
    GenoBhv_AnimMotionBegin(&raw mut a, 0.0f32, 0.0f32);
    GenoBhv_AnimMotionStep(
        &raw mut a,
        &raw mut track as *mut f32,
        2 as i32,
        &raw mut p,
        lr,
        0 as i32,
    );
    *dx = a.vx;
    *dy = a.vy;
}

pub unsafe extern "C" fn geno_cb_iasa_interrupt(mut gobj: *mut Fighter_GObj) {
    melee_core::ct!("geno_cb_iasa_interrupt");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj as *mut HSD_GObj)
        as *mut Fighter;
    if (*fp).allow_interrupt() == 0 {
        return;
    }
    if (*fp).ground_or_air as i32 == GA_Ground as i32 {
        if ftCo_AttackS4_CheckInput(gobj) != 0 || ftCo_Jump_CheckInput(gobj) != 0
            || ftCo_Dash_CheckInput(gobj) != 0 || ftCo_Turn_CheckInput(gobj) != 0
            || ftCo_Walk_CheckInput(gobj) != 0
        {
            return;
        }
    } else if ftCo_80095328(gobj, ::core::ptr::null_mut::<i32>()) != 0
        || ftCo_800D7100(gobj) != 0 || ftCo_800C3B10(gobj) != 0
        || ftCo_800D705C(gobj) != 0 || ftCo_800CB870(gobj) != 0
    {
        return
    }
}

pub unsafe extern "C" fn geno_lua_fbits(mut f: f32) -> u32 {
    melee_core::ct!("geno_lua_fbits");
    let mut c: Anon_anon_109 = unsafe { core::mem::zeroed() };
    c.f = f;
    return c.u;
}

pub unsafe extern "C" fn geno_lua_bitsf(mut u: u32) -> f32 {
    melee_core::ct!("geno_lua_bitsf");
    let mut c: Anon_anon_109 = unsafe { core::mem::zeroed() };
    c.u = u;
    return c.f;
}

pub unsafe extern "C" fn geno_lua_block(mut fp: *mut Fighter) -> *mut GenoLuaBlock {
    melee_core::ct!("geno_lua_block");
    let mut p: i32 = if ((*fp).player_id as i32)
        < GENO_PLAYERS
    {
        (*fp).player_id as i32
    } else {
        GENO_PLAYERS - 1 as i32
    };
    return (&raw mut *(&raw mut Geno_LuaBlock as *mut [GenoLuaBlock; 2])
        .offset(p as isize) as *mut GenoLuaBlock)
        .offset(
            (if (*fp).is_sub_fighter() as i32 != 0 {
                1 as i32
            } else {
                0 as i32
            }) as isize,
        ) as *mut GenoLuaBlock;
}

pub unsafe extern "C" fn GenoGame_LuaBlockOf(
    mut fp: *mut Fighter,
) -> *mut c_void {
    melee_core::ct!("GenoGame_LuaBlockOf");
    return geno_lua_block(fp) as *mut c_void;
}

pub unsafe extern "C" fn GenoGame_LuaInfo(
    mut port: i32,
    mut what: i32,
) -> i32 {
    melee_core::ct!("GenoGame_LuaInfo");
    let mut g: *mut Fighter_GObj = ::core::ptr::null_mut::<Fighter_GObj>();
    let mut cur: *mut HSD_GObj = ::core::ptr::null_mut::<HSD_GObj>();
    if HSD_GObjPLinkHead.is_null() {
        return -(1 as i32);
    }
    cur = *HSD_GObjPLinkHead.offset(HSD_GOBJ_PLINK_FIGHTER as isize);
    while !cur.is_null() {
        let mut fp: *mut Fighter = HSD_GObjGetUserData(cur) as *mut Fighter;
        if (*fp).player_id as i32 == port && (*fp).is_sub_fighter() == 0 {
            let mut st: *mut GenoState = geno_state(fp);
            let mut lb: *mut GenoLuaBlock = geno_lua_block(fp);
            g = cur as *mut Fighter_GObj;
            if what == 0 as i32 {
                return if (*st).profile >= 0 as i32
                    && Geno_LuaSlots((*st).profile as i32)
                        > 0 as i32
                {
                    (*st).profile as i32
                } else {
                    -(1 as i32)
                };
            }
            if what == 1 as i32 {
                return (*lb).faults as i32;
            }
            if what == 2 as i32 {
                return (*lb).last_fault as i32;
            }
            if what >= 3 as i32
                && what < 3 as i32 + GENO_LUA_STATE_SLOTS
            {
                return (*lb).slot[(what - 3 as i32) as usize];
            }
            return -(1 as i32);
        }
        cur = (*cur).next as *mut HSD_GObj;
    }
    return -(1 as i32);
}

pub unsafe extern "C" fn geno_lua_has(
    mut st: *mut GenoState,
    mut s: i32,
    mut phase: i32,
) -> i32 {
    melee_core::ct!("geno_lua_has");
    return ((*st).profile >= 0 as i32 && s >= 0 as i32
        && Geno_LuaFn((*st).profile as i32, s, phase) != 0)
        as i32;
}

pub unsafe extern "C" fn geno_lua_hit_from(
    mut fp: *mut Fighter,
    mut st: *mut GenoState,
) -> u32 {
    melee_core::ct!("geno_lua_hit_from");
    let mut cur: *mut HSD_GObj = ::core::ptr::null_mut::<HSD_GObj>();
    if (*st).hit_port < 0 as i32 || HSD_GObjPLinkHead.is_null() {
        return 0 as u32;
    }
    cur = *HSD_GObjPLinkHead.offset(HSD_GOBJ_PLINK_FIGHTER as isize);
    while !cur.is_null() {
        let mut ap: *mut Fighter = HSD_GObjGetUserData(cur) as *mut Fighter;
        if ap != fp && (*ap).player_id as i32 == (*st).hit_port
            && (*ap).is_sub_fighter() == 0
        {
            let mut dx: f32 = ((*ap).cur_pos.x - (*fp).cur_pos.x)
                * (if (*fp).facing_dir < 0.0f32 { -1.0f32 } else { 1.0f32 });
            return if dx >= 0.0f32 { 1 as u32 } else { 2 as u32 };
        }
        cur = (*cur).next as *mut HSD_GObj;
    }
    return 0 as u32;
}

pub unsafe extern "C" fn geno_lua_fill(
    mut fp: *mut Fighter,
    mut st: *mut GenoState,
    mut lb: *mut GenoLuaBlock,
    mut anim_ended: i32,
) {
    melee_core::ct!("geno_lua_fill");
    let mut io: *mut GenoLuaIo = &raw mut geno_lua_io;
    let mut i: i32 = 0;
    i = 0 as i32;
    while i < GENO_LUA_IO_WORDS {
        (*io).w[i as usize] = 0 as u32;
        i += 1;
    }
    (*io).w[GENO_LUA_IO_FLAGS as i32 as usize] = ((if (*fp).ground_or_air
        as i32 != GA_Ground as i32
    {
        1 as u32
    } else {
        0 as u32
    })
        | (if (*fp).facing_dir < 0.0f32 {
            2 as u32
        } else {
            0 as u32
        })
        | (if anim_ended != 0 {
            4 as u32
        } else {
            0 as u32
        })) as u32;
    (*io).w[GENO_LUA_IO_ACTION_FRAME as i32 as usize] = (*st).action_time
        as u32;
    (*io).w[GENO_LUA_IO_MOTION as i32 as usize] = (*fp).motion_id
        as u32;
    (*io).w[GENO_LUA_IO_HELD as i32 as usize] = geno_buttons(
        fp,
        0 as i32,
    );
    (*io).w[GENO_LUA_IO_PRESSED as i32 as usize] = geno_buttons(
        fp,
        1 as i32,
    );
    (*io).w[GENO_LUA_IO_STICK_X as i32 as usize] = geno_lua_fbits(
        (*fp).input.lstick[0 as i32 as usize].x,
    );
    (*io).w[GENO_LUA_IO_STICK_Y as i32 as usize] = geno_lua_fbits(
        (*fp).input.lstick[0 as i32 as usize].y,
    );
    (*io).w[GENO_LUA_IO_POS_X as i32 as usize] = geno_lua_fbits(
        (*fp).cur_pos.x,
    );
    (*io).w[GENO_LUA_IO_POS_Y as i32 as usize] = geno_lua_fbits(
        (*fp).cur_pos.y,
    );
    (*io).w[GENO_LUA_IO_VEL_X as i32 as usize] = geno_lua_fbits(
        (*fp).self_vel.x,
    );
    (*io).w[GENO_LUA_IO_VEL_Y as i32 as usize] = geno_lua_fbits(
        (*fp).self_vel.y,
    );
    (*io).w[GENO_LUA_IO_PERCENT as i32 as usize] = geno_lua_fbits(
        (*fp).dmg.x1830_percent as f32,
    );
    (*io).w[GENO_LUA_IO_HIT_DAMAGE as i32 as usize] = geno_lua_fbits(
        (*st).hit_damage,
    );
    (*io).w[GENO_LUA_IO_HIT_FROM as i32 as usize] = geno_lua_hit_from(
        fp,
        st,
    );
    (*io).w[GENO_LUA_IO_COUNTERED as i32 as usize] = (*st).counters
        as u32;
    (*io).w[GENO_LUA_IO_KI as i32 as usize] = (*geno_combo_block(fp)).ki
        as u32;
    (*io).w[GENO_LUA_IO_LINKS as i32 as usize] = (*geno_combo_block(fp))
        .links as u32;
    (*io).w[GENO_LUA_IO_CONNECTED as i32 as usize] = (if (*st)
        .atk_connected != 0
    {
        1 as u32
    } else {
        0 as u32
    }) as u32;
    i = 0 as i32;
    while i < GENO_LUA_STATE_SLOTS {
        (*io).w[(GENO_LUA_IO_STATE as i32 + i) as usize] = (*lb)
            .slot[i as usize] as u32;
        i += 1;
    }
}

pub unsafe extern "C" fn geno_lua_run(
    mut gobj: *mut Fighter_GObj,
    mut fp: *mut Fighter,
    mut st: *mut GenoState,
    mut s: i32,
    mut phase: i32,
) -> i32 {
    melee_core::ct!("geno_lua_run");
    let mut lb: *mut GenoLuaBlock = ::core::ptr::null_mut::<GenoLuaBlock>();
    let mut io: *mut GenoLuaIo = &raw mut geno_lua_io;
    let mut rc: i32 = 0;
    let mut n: i32 = 0;
    let mut i: i32 = 0;
    let mut changed: i32 = 0 as i32;
    let mut lr: f32 = 0.;
    if geno_lua_has(st, s, phase) == 0 {
        return 0 as i32;
    }
    if geno_lua_depth >= 4 as i32 {
        lb = geno_lua_block(fp);
        (*lb).faults = (*lb).faults.wrapping_add(1);
        (*lb).last_fault = GENO_LUA_FAULT_CMDS as i32 as i32;
        return geno_go(gobj, fp, st, GENO_AUTO);
    }
    lb = geno_lua_block(fp);
    geno_lua_fill(
        fp,
        st,
        lb,
        (phase == GENO_LUA_PHASE_FRAME && geno_anim_ended(gobj) != 0)
            as i32,
    );
    geno_lua_depth += 1;
    rc = Geno_LuaCall(
        (*st).profile as i32,
        s,
        phase,
        io as *mut c_void,
    );
    if rc != 0 as i32 {
        (*lb).faults = (*lb).faults.wrapping_add(1);
        (*lb).last_fault = rc as i32;
        i = 0 as i32;
        while i < GENO_LUA_IO_WORDS {
            (*io).w[i as usize] = 0 as u32;
            i += 1;
        }
        geno_go(gobj, fp, st, GENO_AUTO);
        geno_lua_depth -= 1;
        return 1 as i32;
    }
    i = 0 as i32;
    while i < GENO_LUA_STATE_SLOTS {
        (*lb).slot[i as usize] = (*io)
            .w[(GENO_LUA_IO_STATE as i32 + i) as usize] as i32;
        i += 1;
    }
    n = (*io).w[GENO_LUA_IO_NCMDS as i32 as usize] as i32;
    if n > GENO_LUA_MAX_CMDS {
        n = GENO_LUA_MAX_CMDS;
    }
    lr = (if (*fp).facing_dir < 0.0f32 { -1.0f32 } else { 1.0f32 }) as f32;
    i = 0 as i32;
    while i < n {
        let mut op: u32 = (*io)
            .w[(GENO_LUA_IO_CMDS as i32 + 3 as i32 * i)
            as usize];
        let mut a: u32 = (*io)
            .w[(GENO_LUA_IO_CMDS as i32 + 3 as i32 * i
            + 1 as i32) as usize];
        let mut b: u32 = (*io)
            .w[(GENO_LUA_IO_CMDS as i32 + 3 as i32 * i
            + 2 as i32) as usize];
        match op {
            1 => {
                if geno_go(gobj, fp, st, a) != 0 {
                    changed = 1 as i32;
                }
            }
            2 => {
                let mut fwd: f32 = geno_lua_bitsf(a);
                let mut up: f32 = geno_lua_bitsf(b);
                if (*fp).ground_or_air as i32
                    == GA_Ground as i32
                {
                    (*fp).gr_vel = (fwd * lr) as f32;
                    if geno_test_capture != 0 {
                        (*fp).self_vel.x = fwd * lr;
                    } else {
                        ftCommon_SetSelfMovementFromGroundedMovement(gobj);
                    }
                } else {
                    (*fp).self_vel.x = fwd * lr;
                    (*fp).self_vel.y = up;
                }
            }
            3 => {
                let mut h: i32 = 0;
                let mut d: f32 = geno_lua_bitsf(b);
                h = 0 as i32;
                while h < 4 as i32 {
                    if a as u32 & (1 as u32) << h != 0 {
                        (*fp).x914[h as usize].damage = if d < 0.0f32 {
                            0.0f32
                        } else {
                            d as f32
                        };
                        ScriptGame_HitRuleDamageOnly(
                            (&raw mut (*fp).x914 as *mut HitCapsule).offset(h as isize)
                                as *mut HitCapsule,
                        );
                    }
                    h += 1;
                }
            }
            4 => {
                if geno_test_capture == 0 {
                    Fighter_ChangeMotionState(
                        gobj,
                        (*fp).motion_id,
                        Ft_MF_KeepFastFall | Ft_MF_KeepGfx | Ft_MF_KeepSfx
                            | Ft_MF_SkipModelPartVis | Ft_MF_SkipItemVis
                            | Ft_MF_SkipNametagVis | Ft_MF_KeepSwordTrail,
                        0.0f32,
                        1.0f32,
                        0.0f32,
                        ::core::ptr::null_mut::<Fighter_GObj>(),
                    );
                } else {
                    geno_test_changes += 1;
                }
            }
            6 => {
                if (*st).profile >= 0 as i32
                    && Geno_ComboParam(
                        (*st).profile as i32,
                        GENO_CP_ROUTES as i32,
                    ) > 0 as i32
                {
                    geno_combo_ki_add(fp, st, a as i32);
                }
            }
            7 => {
                (*st).ledge = (if a > 2 as u32 {
                    2 as i32
                } else {
                    a as i32
                }) as i32;
                if a == 3 as u32 {
                    (*st).fall_ledge = 1 as i32 as i32;
                }
            }
            5 => {
                (*fp).facing_dir = -(*fp).facing_dir;
                if !(*fp).parts.is_null() {
                    ftPartSetRotY(
                        fp,
                        0 as i32,
                        1.5707964f32 * (*fp).facing_dir as f32,
                    );
                }
                lr = (if (*fp).facing_dir < 0.0f32 { -1.0f32 } else { 1.0f32 }) as f32;
            }
            _ => {}
        }
        i += 1;
    }
    i = 0 as i32;
    while i < GENO_LUA_IO_WORDS {
        (*io).w[i as usize] = 0 as u32;
        i += 1;
    }
    geno_lua_depth -= 1;
    return changed;
}

pub unsafe extern "C" fn geno_lua_enter(
    mut gobj: *mut Fighter_GObj,
    mut fp: *mut Fighter,
    mut st: *mut GenoState,
    mut s: i32,
) {
    melee_core::ct!("geno_lua_enter");
    if geno_lua_has(st, s, GENO_LUA_PHASE_ENTER) != 0 {
        geno_lua_run(gobj, fp, st, s, GENO_LUA_PHASE_ENTER);
    }
}

pub unsafe extern "C" fn geno_cb_anim_lua(mut gobj: *mut Fighter_GObj) {
    melee_core::ct!("geno_cb_anim_lua");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj as *mut HSD_GObj)
        as *mut Fighter;
    let mut st: *mut GenoState = geno_state(fp);
    let mut s: i32 = geno_cur_state(fp, st);
    if s < 0 as i32 {
        geno_cb_anim_next(gobj);
        return;
    }
    if geno_lua_run(gobj, fp, st, s, GENO_LUA_PHASE_FRAME) != 0 {
        return;
    }
    if geno_anim_ended(gobj) != 0 {
        geno_go(gobj, fp, st, geno_next_of(st, s));
    }
}

pub unsafe extern "C" fn geno_lua_digest(
    mut fp: *mut Fighter,
    mut st: *mut GenoState,
) -> u32 {
    melee_core::ct!("geno_lua_digest");
    let mut lb: *mut GenoLuaBlock = ::core::ptr::null_mut::<GenoLuaBlock>();
    let mut h: u32 = 0x4c554131 as u32;
    let mut i: i32 = 0;
    if (*st).profile < 0 as i32
        || Geno_LuaSlots((*st).profile as i32) <= 0 as i32
    {
        return 0 as u32;
    }
    lb = geno_lua_block(fp);
    i = 0 as i32;
    while i < GENO_LUA_STATE_SLOTS {
        h = geno_dg(h, (*lb).slot[i as usize] as u32);
        i += 1;
    }
    h = geno_dg(h, (*lb).faults);
    h = geno_dg(h, (*lb).last_fault as u32);
    return if h != 0 as u32 { h } else { 1 as u32 };
}

pub unsafe extern "C" fn geno_cb_def(
    mut slot: i32,
    mut id: i32,
) -> *const GenoCbDef {
    melee_core::ct!("geno_cb_def");
    let mut i: i32 = 0;
    i = 0 as i32;
    while i < GENO_NCBS {
        if geno_cbs[i as usize].slot == slot && geno_cbs[i as usize].id == id {
            return (&raw const geno_cbs as *const GenoCbDef).offset(i as isize)
                as *const GenoCbDef;
        }
        i += 1;
    }
    return ::core::ptr::null::<GenoCbDef>();
}

pub unsafe extern "C" fn geno_bhv_def(mut id: i32) -> *const GenoBhvDef {
    melee_core::ct!("geno_bhv_def");
    let mut i: i32 = 0;
    i = 0 as i32;
    while i < GENO_NBHVS {
        if geno_bhvs[i as usize].id == id {
            return (&raw const geno_bhvs as *const GenoBhvDef).offset(i as isize)
                as *const GenoBhvDef;
        }
        i += 1;
    }
    return (&raw const geno_bhvs as *const GenoBhvDef)
        .offset(0 as i32 as isize) as *const GenoBhvDef;
}

pub unsafe extern "C" fn GenoGame_BehaviorFind(
    mut name: *const i8,
) -> i32 {
    melee_core::ct!("GenoGame_BehaviorFind");
    let mut i: i32 = 0;
    i = 1 as i32;
    while i < GENO_NBHVS {
        if geno_streq(name, geno_bhvs[i as usize].name) != 0 {
            return geno_bhvs[i as usize].id;
        }
        i += 1;
    }
    return -(1 as i32);
}

pub unsafe extern "C" fn GenoGame_CallbackFind(
    mut slot: i32,
    mut name: *const i8,
) -> i32 {
    melee_core::ct!("GenoGame_CallbackFind");
    let mut i: i32 = 0;
    if geno_streq(name, b"like\0" as *const u8 as *const i8) != 0 {
        return GENO_CB_LIKE;
    }
    i = 0 as i32;
    while i < GENO_NCBS {
        if geno_cbs[i as usize].slot == slot
            && geno_streq(name, geno_cbs[i as usize].name) != 0
        {
            return geno_cbs[i as usize].id;
        }
        i += 1;
    }
    return -(1 as i32);
}

pub unsafe extern "C" fn geno_v2_preanim(
    mut gobj: *mut Fighter_GObj,
    mut fp: *mut Fighter,
    mut st: *mut GenoState,
) -> i32 {
    melee_core::ct!("geno_v2_preanim");
    return geno_glide_entry(gobj, fp, st);
}

pub unsafe extern "C" fn Geno_SpecialEnter(
    mut gobj: *mut Fighter_GObj,
    mut which: i32,
) -> i32 {
    melee_core::ct!("Geno_SpecialEnter");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj as *mut HSD_GObj)
        as *mut Fighter;
    let mut st: *mut GenoState = geno_state(fp);
    let mut t: u32 = 0;

    ScriptGame_NoteSpecialInput(fp);
    if (*st).profile < 0 as i32 {
        return 0 as i32;
    }
    t = Geno_Special((*st).profile as i32, which) as u32;

    let mut ref_0: i32 = Geno_SpecialSelect(
        (*st).profile as i32,
        which,
        -(1 as i32),
    );
    if ref_0 >= 0 as i32 {
        let mut n: i32 = *(geno_var(st, ref_0) as *mut i32);
        t = (if n >= 0 as i32
            && n
                < Geno_SpecialSelect(
                    (*st).profile as i32,
                    which,
                    -(2 as i32),
                )
        {
            Geno_SpecialSelect(
                (*st).profile as i32,
                which,
                n as i32,
            ) as u32
        } else {
            GENO_NONE_T
        }) as u32;
    }
    if t == GENO_NONE_T || geno_target_ok(fp, t) == 0 {
        return 0 as i32;
    }
    Geno_Event(
        18 as i32,
        (*fp).kind as i32,
        (*fp).player_id as i32,
        which,
        t as i32,
    );
    return geno_do_change(gobj, fp, st, t);
}

pub unsafe extern "C" fn Geno_LabEnterState(
    mut fp: *mut Fighter,
    mut s: i32,
) -> i32 {
    melee_core::ct!("Geno_LabEnterState");
    let mut st: *mut GenoState = geno_state(fp);
    if (*st).profile < 0 as i32 || geno_state_valid(st, s) == 0
        || s >= *Geno_RowCount.offset((*st).profile as isize)
    {
        return -(1 as i32);
    }
    return if geno_enter_state((*fp).gobj as *mut Fighter_GObj, fp, st, s, 0 as u32)
        != 0
    {
        0 as i32
    } else {
        -(1 as i32)
    };
}

pub unsafe extern "C" fn Geno_MotionRowIfAny(
    mut fp: *mut Fighter,
    mut msid: i32,
) -> *mut MotionState {
    melee_core::ct!("Geno_MotionRowIfAny");
    let mut st: *mut GenoState = geno_state(fp);
    let mut s: i32 = msid - GENO_MOTION_BASE;
    return if geno_state_valid(st, s) != 0
        && s < *Geno_RowCount.offset((*st).profile as isize)
    {
        (*Geno_Rows.offset((*st).profile as isize)).offset(s as isize)
            as *mut MotionState
    } else {
        ::core::ptr::null_mut::<MotionState>()
    };
}

pub unsafe extern "C" fn GenoGame_DefineApplyRows(
    mut kind: i32,
    mut common: *mut MotionState,
    mut specific: *mut MotionState,
    mut subactions: i32,
) {
    melee_core::ct!("GenoGame_DefineApplyRows");
    let mut i: i32 = 0;
    let mut n: i32 = Geno_DefineOverrideCount(kind);
    i = 0 as i32;
    while i < n {
        let mut motion: i32 = Geno_DefineOverrideField(
            kind,
            i,
            0 as i32,
        );
        let mut like: i32 = Geno_DefineOverrideField(
            kind,
            i,
            1 as i32,
        );
        let mut row: *mut MotionState = if motion < ftCo_MS_Count as i32 {
            common.offset(motion as isize) as *mut MotionState
        } else {
            specific.offset((motion - ftCo_MS_Count as i32) as isize)
                as *mut MotionState
        };
        let mut field: i32 = 0;
        let mut value: i32 = 0;
        if like >= 0 as i32 {
            *row = if like < ftCo_MS_Count as i32 {
                *(&raw mut ftData_MotionStateList as *mut MotionState)
                    .offset(like as isize)
            } else {
                *if Geno_DefineBaseKind(kind) == Ft_Kind_Captain as i32 {
                    &raw mut ftCa_Init_MotionStateTable as *mut MotionState
                } else if Geno_DefineBaseKind(kind) == Ft_Kind_Fox as i32
                {
                    &raw mut ftFx_Init_MotionStateTable as *mut MotionState
                } else if Geno_DefineBaseKind(kind)
                    == Ft_Kind_Falco as i32
                {
                    &raw mut ftFc_Init_MotionStateTable as *mut MotionState
                } else if Geno_DefineBaseKind(kind) == Ft_Kind_Mars as i32
                {
                    &raw mut ftMs_Init_MotionStateTable as *mut MotionState
                } else if Geno_DefineBaseKind(kind)
                    == Ft_Kind_Purin as i32
                {
                    &raw mut ftPr_Init_MotionStateTable as *mut MotionState
                } else if Geno_DefineBaseKind(kind) == Ft_Kind_Seak as i32
                {
                    &raw mut ftSk_Init_MotionStateTable as *mut MotionState
                } else if Geno_DefineBaseKind(kind)
                    == Ft_Kind_Zelda as i32
                {
                    &raw mut ftZd_Init_MotionStateTable as *mut MotionState
                } else {
                    &raw mut ftMr_Init_MotionStateTable as *mut MotionState
                }
                    .offset((like - ftCo_MS_Count as i32) as isize)
            };
        }
        value = Geno_DefineOverrideField(kind, i, 2 as i32);
        if value < subactions {} else {
            OSReport!(
                b"Geno native state references missing subaction\n\0" as *const u8
                    as *const i8 as *mut i8,
            );
            __assert(
                b"pc/geno/geno_define_rows.inc\0" as *const u8
                    as *const i8,
                28 as u32,
                b"value < subactions\0" as *const u8 as *const i8,
            );
        };
        if value >= 0 as i32 {
            (*row).anim_id = value as enum_t;
        }
        value = Geno_DefineOverrideField(kind, i, 3 as i32);
        if value >= 0 as i32 {
            (*row).x4_flags = value as enum_t;
        }
        value = Geno_DefineOverrideField(kind, i, 4 as i32);
        if value >= 0 as i32 {
            (*row).anon_8.anon_0.set_move_id(value as u8 as u8);
        }
        field = 5 as i32;
        while field < 9 as i32 {
            let mut cb: *const GenoCbDef = ::core::ptr::null::<GenoCbDef>();
            value = Geno_DefineOverrideField(kind, i, field);
            if !(value < 0 as i32) {
                cb = geno_cb_def(field - 5 as i32, value);
                if !cb.is_null() {} else {
                    OSReport!(
                        b"Geno native state callback unavailable\n\0" as *const u8
                            as *const i8 as *mut i8,
                    );
                    __assert(
                        b"pc/geno/geno_define_rows.inc\0" as *const u8
                            as *const i8,
                        37 as u32,
                        b"cb != NULL\0" as *const u8 as *const i8,
                    );
                };
                match field {
                    5 => {
                        (*row).anim_cb = (*cb).r#fn as HSD_GObjEvent;
                    }
                    6 => {
                        (*row).input_cb = (*cb).r#fn as HSD_GObjEvent;
                    }
                    7 => {
                        (*row).phys_cb = (*cb).r#fn as HSD_GObjEvent;
                    }
                    8 => {
                        (*row).coll_cb = (*cb).r#fn as HSD_GObjEvent;
                    }
                    _ => {}
                }
            }
            field += 1;
        }
        i += 1;
    }
}

pub unsafe extern "C" fn Item_InitSpawnCommonFields(
    mut spawn: *mut SpawnItem,
    mut parent: *mut HSD_GObj,
    mut facing_dir: f32,
    mut initial_collision: i32,
) {
    melee_core::ct!("Item_InitSpawnCommonFields");
    (*spawn).facing_dir = facing_dir;
    (*spawn).x3C_damage = 0 as i16;
    (*spawn).vel.z = 0.0f32 as f32;
    (*spawn).vel.y = (*spawn).vel.z;
    (*spawn).vel.x = (*spawn).vel.y;
    (*spawn).x0_parent_gobj = parent;
    (*spawn).x4_parent_gobj2 = (*spawn).x0_parent_gobj;
    (*spawn).x44_flag.anon_0.set_b0(initial_collision as u8 as u8);
    (*spawn).x40 = 0 as i32 as i32;
}

pub unsafe extern "C" fn Item_InitSpawnPositionFromParent(
    mut spawn: *mut SpawnItem,
    mut parent: *mut HSD_GObj,
    mut pos: *mut Vec,
) {
    melee_core::ct!("Item_InitSpawnPositionFromParent");
    (*spawn).prev_pos = *pos;
    (*spawn).prev_pos.z = 0.0f32 as f32;
    it_8026BB68(parent as *mut Item_GObj, &raw mut (*spawn).pos);
}

pub unsafe extern "C" fn geno_art_vars(mut ip: *mut Item) -> *mut GenoArtVars {
    melee_core::ct!("geno_art_vars");
    return &raw mut (*ip).xDD4_itemVar as *mut GenoArtVars;
}

pub unsafe extern "C" fn geno_art_f(
    mut p: i32,
    mut a: i32,
    mut id: i32,
) -> f32 {
    melee_core::ct!("geno_art_f");
    let mut b: Anon_anon_1941 = unsafe { core::mem::zeroed() };
    b.u = Geno_ArticleParam(p, a, id) as u32;
    return b.f;
}

pub unsafe extern "C" fn geno_art_hf(
    mut p: i32,
    mut a: i32,
    mut h: i32,
    mut id: i32,
) -> f32 {
    melee_core::ct!("geno_art_hf");
    let mut b: Anon_anon_1941 = unsafe { core::mem::zeroed() };
    b.u = Geno_ArticleHitParam(p, a, h, id) as u32;
    return b.f;
}

pub unsafe extern "C" fn geno_art_of_kind(
    mut kind: i32,
    mut p: *mut i32,
    mut a: *mut i32,
) -> i32 {
    melee_core::ct!("geno_art_of_kind");
    if !(kind >= GENO_ART_KIND_BASE && kind < GENO_ART_KIND_END
        || kind >= GENO_ART_EXTRA_BASE && kind < GENO_ART_EXTRA_END)
    {
        return 0 as i32;
    }
    if kind >= GENO_ART_EXTRA_BASE {
        *p = 32 as i32
            + (kind - GENO_ART_EXTRA_BASE) / 16 as i32;
        *a = (kind - GENO_ART_EXTRA_BASE) % 16 as i32;
    } else if kind < GENO_ART_KIND_BASE2 {
        *p = (kind - GENO_ART_KIND_BASE) / GENO_ART_PER_RANGE;
        *a = (kind - GENO_ART_KIND_BASE) % GENO_ART_PER_RANGE;
    } else {
        *p = (kind - GENO_ART_KIND_BASE2) / GENO_ART_PER_RANGE;
        *a = GENO_ART_PER_RANGE + (kind - GENO_ART_KIND_BASE2) % GENO_ART_PER_RANGE;
    }
    return (*a < Geno_ArticleCount(*p)) as i32;
}

pub unsafe extern "C" fn Geno_ArticleStunBonus(
    mut item: *mut Item,
    mut hit: *mut HitCapsule,
    mut vic: *mut Fighter,
) {
    melee_core::ct!("Geno_ArticleStunBonus");
    let mut p: i32 = 0;
    let mut a: i32 = 0;
    let mut slot: i32 = 0;
    let mut h: i32 = 0;
    let mut v: *mut GenoArtVars = ::core::ptr::null_mut::<GenoArtVars>();
    if item.is_null() || hit.is_null()
        || geno_art_of_kind((*item).kind as i32, &raw mut p, &raw mut a)
            == 0
    {
        return;
    }
    slot = 0 as i32;
    while slot < 4 as i32
        && &raw mut (*(&raw mut (*item).x5D4_hitboxes as *mut ItemHitbox)
            .offset(slot as isize))
            .hit != hit
    {
        slot += 1;
    }
    if slot >= 4 as i32 || slot >= GENO_ART_HITBOXES {
        return;
    }
    v = geno_art_vars(item);
    h = (*v).cur[slot as usize] as i32;
    if h < 0 as i32 {
        return;
    }
    Geno_GiveStunBonus(
        vic,
        Geno_ArticleHitParam(p, a, h, GENO_AH_STUN as i32),
    );
}

pub unsafe extern "C" fn geno_art_prepare(mut p: i32) {
    melee_core::ct!("geno_art_prepare");
    let mut n: i32 = Geno_ProfileCount();
    let mut i: i32 = 0;
    if n > Geno_ArtProfileCapacity {
        let mut ds: *mut *mut Article = geno_profile_alloc(
            (n as usize).wrapping_mul(::core::mem::size_of::<*mut Article>() as usize)
                as i32,
        ) as *mut *mut Article;
        let mut as_0: *mut *mut ItemAttr = geno_profile_alloc(
            (n as usize).wrapping_mul(::core::mem::size_of::<*mut ItemAttr>() as usize)
                as i32,
        ) as *mut *mut ItemAttr;
        let mut ms: *mut *mut ItemModelDesc = geno_profile_alloc(
            (n as usize)
                .wrapping_mul(::core::mem::size_of::<*mut ItemModelDesc>() as usize)
                as i32,
        ) as *mut *mut ItemModelDesc;
        i = 0 as i32;
        while i < Geno_ArtProfileCapacity {
            let ref mut fresh4 = *ds.offset(i as isize);
            *fresh4 = *Geno_ArtDesc.offset(i as isize);
            let ref mut fresh5 = *as_0.offset(i as isize);
            *fresh5 = *Geno_ArtAttr.offset(i as isize);
            let ref mut fresh6 = *ms.offset(i as isize);
            *fresh6 = *Geno_ArtModel.offset(i as isize);
            i += 1;
        }
        Geno_ArtDesc = ds;
        Geno_ArtAttr = as_0;
        Geno_ArtModel = ms;
        Geno_ArtProfileCapacity = n;
    }
    if (*Geno_ArtDesc.offset(p as isize)).is_null() {
        let ref mut fresh7 = *Geno_ArtDesc.offset(p as isize);
        *fresh7 = geno_profile_alloc(
            (GENO_MAX_ARTICLES as usize)
                .wrapping_mul(::core::mem::size_of::<Article>() as usize)
                as i32,
        ) as *mut Article;
    }
    if (*Geno_ArtAttr.offset(p as isize)).is_null() {
        let ref mut fresh8 = *Geno_ArtAttr.offset(p as isize);
        *fresh8 = geno_profile_alloc(
            (GENO_MAX_ARTICLES as usize)
                .wrapping_mul(::core::mem::size_of::<ItemAttr>() as usize)
                as i32,
        ) as *mut ItemAttr;
    }
    if (*Geno_ArtModel.offset(p as isize)).is_null() {
        let ref mut fresh9 = *Geno_ArtModel.offset(p as isize);
        *fresh9 = geno_profile_alloc(
            (GENO_MAX_ARTICLES as usize)
                .wrapping_mul(::core::mem::size_of::<ItemModelDesc>() as usize)
                as i32,
        ) as *mut ItemModelDesc;
    }
}

pub unsafe extern "C" fn geno_art_build(
    mut p: i32,
    mut a: i32,
) {
    melee_core::ct!("geno_art_build");
    geno_art_prepare(p);
    let mut d: *mut Article = (*Geno_ArtDesc.offset(p as isize)).offset(a as isize)
        as *mut Article;
    let mut at: *mut ItemAttr = (*Geno_ArtAttr.offset(p as isize)).offset(a as isize)
        as *mut ItemAttr;
    let mut m: *mut ItemModelDesc = (*Geno_ArtModel.offset(p as isize))
        .offset(a as isize) as *mut ItemModelDesc;
    geno_zero(
        at as *mut c_void,
        ::core::mem::size_of::<ItemAttr>() as i32,
    );
    (*at).x60_scale = geno_art_f(p, a, GENO_AP_SCALE as i32);
    if (*at).x60_scale <= 0.0f32 {
        (*at).x60_scale = 1.0f32 as f32;
    }
    geno_zero(
        m as *mut c_void,
        ::core::mem::size_of::<ItemModelDesc>() as i32,
    );
    (*m).x0_joint = Geno_ArticleJoint(p, a) as *mut HSD_Joint;
    geno_zero(
        d as *mut c_void,
        ::core::mem::size_of::<Article>() as i32,
    );
    (*d).x0_common_attr = at;
    (*d).x4_specialAttributes = ::core::ptr::null_mut::<c_void>();
    (*d).x8_hurtbones = ::core::ptr::null_mut::<ItHurtBoneList>();
    (*d).xC_itemStates = &raw mut Geno_ArtStates;
    (*d).x10_modelDesc = m;
    (*d).x14_dynamics = ::core::ptr::null_mut::<ItemDynamics>();
}

pub unsafe extern "C" fn geno_art_unref(mut gobj: *mut Item_GObj, mut ref_0: *mut HSD_GObj) {
    melee_core::ct!("geno_art_unref");
    it_8026B894(gobj, ref_0);
}

pub unsafe extern "C" fn geno_stun_frame() -> i32 {
    melee_core::ct!("geno_stun_frame");
    return geno_fx_frame;
}

pub unsafe extern "C" fn geno_art_destroyed(mut gobj: *mut Item_GObj) {
    melee_core::ct!("geno_art_destroyed");
    hsd_8039D5DC(HSD_GObjGetHSDObj(gobj as *mut HSD_GObj) as *mut HSD_JObj);
    Fx_Detach(
        HSD_GObjGetHSDObj(gobj as *mut HSD_GObj) as *mut HSD_JObj as i32,
    );
}

pub unsafe extern "C" fn geno_art_hide_model(mut j: *mut HSD_JObj) {
    melee_core::ct!("geno_art_hide_model");
    while !j.is_null() {
        let mut d: *mut HSD_DObj = ::core::ptr::null_mut::<HSD_DObj>();
        d = (if (*j).flags & (JOBJ_PTCL | JOBJ_SPLINE | JOBJ_INSTANCE) as u32 != 0 {
            ::core::ptr::null_mut::<HSD_DObj>()
        } else {
            (*j).u.dobj
        }) as *mut HSD_DObj;
        while !d.is_null() {
            HSD_DObjSetFlags(d, DOBJ_HIDDEN as u32);
            d = (*d).next;
        }
        geno_art_hide_model((*j).child as *mut HSD_JObj);
        j = (*j).next as *mut HSD_JObj;
    }
}

pub unsafe extern "C" fn geno_fx_render(
    mut gobj: *mut HSD_GObj,
    mut code: i32,
) {
    melee_core::ct!("geno_fx_render");
    let mut cobj: *mut HSD_CObj = ::core::ptr::null_mut::<HSD_CObj>();
    let mut cur: *mut HSD_CObj = HSD_CObjGetCurrent();
    let mut mc: *mut HSD_CObj = Camera_800310B8();
    let mut e: Vec = unsafe { core::mem::zeroed() };
    let mut m: Vec = unsafe { core::mem::zeroed() };
    if !cur.is_null() {
        HSD_CObjGetEyePosition(cur, &raw mut e);
    }
    if !mc.is_null() {
        HSD_CObjGetEyePosition(mc, &raw mut m);
    }
    diag_fx_cam(
        cur as i32,
        code,
        mc as i32,
        e.x as f32,
        e.y as f32,
        e.z as f32,
        m.x as f32,
        m.y as f32,
        m.z as f32,
    );
    if code != 2 as i32 {
        return;
    }
    cobj = HSD_CObjGetCurrent();
    if !cobj.is_null() {
        Fx_Draw(HSD_CObjGetViewingMtxPtr(cobj) as i32);
    }
}

pub unsafe extern "C" fn geno_fx_setup_tree(mut j: *mut HSD_JObj) {
    melee_core::ct!("geno_fx_setup_tree");
    while !j.is_null() {
        HSD_JObjSetupMatrix(j);
        geno_fx_setup_tree((*j).child as *mut HSD_JObj);
        j = (*j).next as *mut HSD_JObj;
    }
}

pub unsafe extern "C" fn geno_fx_drive(mut frame: i32) {
    melee_core::ct!("geno_fx_drive");
    let mut slot: i32 = 0;
    slot = 0 as i32;
    while slot < 6 as i32 {
        let mut g: *mut HSD_GObj = Player_GetEntity(slot as i32);
        let mut fp: *mut Fighter = ::core::ptr::null_mut::<Fighter>();
        let mut p: i32 = 0;
        let mut set: i32 = 0;
        if !g.is_null() {
            fp = HSD_GObjGetUserData(g) as *mut Fighter;
            p = Geno_ProfileForKind((*fp).kind as i32);
            set = if p >= 0 as i32 {
                Geno_FxBindings(p)
            } else {
                -(1 as i32)
            };
            if !(set < 0 as i32 || (*fp).parts.is_null()
                || (*ftPartsTable.offset((*fp).kind as isize)).is_null())
            {
                geno_fx_setup_tree(HSD_GObjGetHSDObj(g) as *mut HSD_JObj);
                Fx_Drive(
                    set,
                    fp as i32,
                    (*fp).motion_id,
                    (*fp).anim_id as i32,
                    (*fp).cur_anim_frame,
                    (*geno_state(fp)).action_time,
                    ((*fp).ground_or_air as i32
                        == GA_Air as i32) as i32,
                    if (*fp).facing_dir < 0.0f32 {
                        -(1 as i32)
                    } else {
                        1 as i32
                    },
                    slot + 1 as i32,
                    (*fp).parts as i32,
                    ::core::mem::size_of::<FighterBone>() as i32,
                    (**ftPartsTable.offset((*fp).kind as isize)).parts_num
                        as i32,
                    (**ftPartsTable.offset((*fp).kind as isize)).joint_to_part
                        as i32,
                    HSD_GObjGetHSDObj(g) as *mut HSD_JObj as i32,
                    frame,
                );
            }
        }
        slot += 1;
    }
}

pub unsafe extern "C" fn geno_fx_tree_has(
    mut root: *mut HSD_JObj,
    mut j: *mut HSD_JObj,
    mut depth: i32,
) -> i32 {
    melee_core::ct!("geno_fx_tree_has");
    while !root.is_null() && depth < 64 as i32 {
        if root == j
            || geno_fx_tree_has(
                (*root).child as *mut HSD_JObj,
                j,
                depth + 1 as i32,
            ) != 0
        {
            return 1 as i32;
        }
        root = (*root).next as *mut HSD_JObj;
    }
    return 0 as i32;
}

pub unsafe extern "C" fn geno_fx_owner_alive(mut j: *mut HSD_JObj) -> i32 {
    melee_core::ct!("geno_fx_owner_alive");
    let mut cur: *mut HSD_GObj = ::core::ptr::null_mut::<HSD_GObj>();
    let mut slot: i32 = 0;
    if !HSD_GObjPLinkHead.is_null() {
        cur = *HSD_GObjPLinkHead.offset(HSD_GOBJ_PLINK_ITEM as isize);
        while !cur.is_null() {
            if HSD_GObjGetHSDObj(cur) as *mut HSD_JObj == j {
                return 1 as i32;
            }
            cur = (*cur).next as *mut HSD_GObj;
        }
    }
    slot = 0 as i32;
    while slot < 6 as i32 {
        let mut g: *mut HSD_GObj = Player_GetEntity(slot as i32);
        if !g.is_null()
            && geno_fx_tree_has(
                HSD_GObjGetHSDObj(g) as *mut HSD_JObj,
                j,
                0 as i32,
            ) != 0
        {
            return 1 as i32;
        }
        slot += 1;
    }
    return 0 as i32;
}

pub unsafe extern "C" fn Geno_FxFramePost(mut frame: i32) {
    melee_core::ct!("Geno_FxFramePost");
    if frame <= 1 as i32 {
        geno_fx_gobj = ::core::ptr::null_mut::<HSD_GObj>();
    }
    geno_fx_frame = frame;
    geno_fx_drive(frame);
    let mut i: i32 = 0;
    let mut owner: i32 = 0;
    i = 0 as i32;
    while i < 256 as i32 {
        owner = Fx_Owner(i);
        if owner != 0 as i32 {
            let mut oj: *mut HSD_JObj = owner as *mut HSD_JObj;
            let mut dirty: u32 = 0;
            if !(geno_fx_owner_alive(oj) == 0) {
                dirty = (*oj).flags & JOBJ_MTX_DIRTY as u32;
                HSD_JObjSetupMatrix(oj);
                (*oj).flags |= dirty;
            }
        }
        i += 1;
    }
    Fx_Frame(frame);
    if geno_fx_gobj.is_null()
        && Fx_Stat(0 as i32) > 0 as i32
    {
        geno_fx_gobj = GObj_Create(
            HSD_GOBJ_CLASS_EFFECT as u16,
            0xc as u8,
            0 as u8,
        );
        if !geno_fx_gobj.is_null() {
            GObj_SetupGXLink(
                geno_fx_gobj,
                Some(
                    geno_fx_render
                        as unsafe extern "C" fn(*mut HSD_GObj, i32) -> (),
                ),
                7 as u8,
                255 as u32,
            );
        }
    }
}

pub unsafe extern "C" fn Geno_ArticleDesc(
    mut kind: i32,
) -> *mut c_void {
    melee_core::ct!("Geno_ArticleDesc");
    let mut p: i32 = 0;
    let mut a: i32 = 0;
    if geno_art_of_kind(kind, &raw mut p, &raw mut a) == 0 {
        return ::core::ptr::null_mut::<c_void>();
    }
    geno_art_build(p, a);
    return (*Geno_ArtDesc.offset(p as isize)).offset(a as isize) as *mut Article
        as *mut c_void;
}

pub unsafe extern "C" fn Geno_ArticleLogic(
    mut kind: i32,
) -> *mut c_void {
    melee_core::ct!("Geno_ArticleLogic");
    return &raw mut geno_art_logic as *mut c_void;
}

pub unsafe extern "C" fn geno_art_live(
    mut owner: *mut HSD_GObj,
    mut article: i32,
) -> i32 {
    melee_core::ct!("geno_art_live");
    let mut cur: *mut HSD_GObj = ::core::ptr::null_mut::<HSD_GObj>();
    let mut n: i32 = 0 as i32;
    let mut p: i32 = 0;
    let mut a: i32 = 0;
    if HSD_GObjPLinkHead.is_null() || owner.is_null() {
        return 0 as i32;
    }
    cur = *HSD_GObjPLinkHead.offset(HSD_GOBJ_PLINK_ITEM as isize);
    while !cur.is_null() {
        let mut ip: *mut Item = HSD_GObjGetUserData(cur) as *mut Item;
        if !(ip.is_null() || (*ip).owner != owner
            || geno_art_of_kind((*ip).kind as i32, &raw mut p, &raw mut a)
                == 0)
        {
            if article < 0 as i32 || a == article {
                n += 1;
            }
        }
        cur = (*cur).next as *mut HSD_GObj;
    }
    return n;
}

pub unsafe extern "C" fn Geno_ArtStableKind(mut kind: i32) -> u32 {
    melee_core::ct!("Geno_ArtStableKind");
    let mut p: i32 = 0;
    let mut a: i32 = 0;
    let mut id: u32 = 0;
    if geno_art_of_kind(kind, &raw mut p, &raw mut a) == 0 {
        return 0 as u32;
    }
    id = Geno_ProfileIdWord(p) as u32;
    return geno_dg(geno_dg(0x41525447 as u32, id), a as u32) | 1 as u32;
}

pub unsafe extern "C" fn geno_art_digest(mut fp: *mut Fighter) -> u32 {
    melee_core::ct!("geno_art_digest");
    let mut cur: *mut HSD_GObj = ::core::ptr::null_mut::<HSD_GObj>();
    let mut sum: u32 = 0 as u32;
    let mut n: i32 = 0 as i32;
    let mut k: i32 = 0;
    let mut p: i32 = 0;
    let mut a: i32 = 0;
    if HSD_GObjPLinkHead.is_null() || fp.is_null() || (*fp).gobj.is_null() {
        return 0 as u32;
    }
    cur = *HSD_GObjPLinkHead.offset(HSD_GOBJ_PLINK_ITEM as isize);
    while !cur.is_null() {
        let mut ip: *mut Item = HSD_GObjGetUserData(cur) as *mut Item;
        let mut v: *mut GenoArtVars = ::core::ptr::null_mut::<GenoArtVars>();
        let mut h: u32 = 0x41525449 as u32;
        if !(ip.is_null() || (*ip).owner != (*fp).gobj
            || geno_art_of_kind((*ip).kind as i32, &raw mut p, &raw mut a)
                == 0)
        {
            v = geno_art_vars(ip);
            h = geno_dg(h, (*v).profile as u32);
            h = geno_dg(h, (*v).article as u32);
            h = geno_dg(h, (*v).frame as u32);
            h = geno_dg(h, (*v).why as u32);
            h = geno_dgf(h, (*v).vx);
            h = geno_dgf(h, (*v).vy);
            h = geno_dgf(h, (*ip).pos.x);
            h = geno_dgf(h, (*ip).pos.y);
            h = geno_dgf(h, (*ip).pos.z);
            h = geno_dgf(h, (*ip).x40_vel.x);
            h = geno_dgf(h, (*ip).x40_vel.y);
            k = 0 as i32;
            while k < GENO_ART_HITBOXES {
                h = geno_dg(h, (*v).cur[k as usize] as i32 as u32);
                k += 1;
            }
            sum = sum.wrapping_add(h);
            n += 1;
        }
        cur = (*cur).next as *mut HSD_GObj;
    }
    return if n != 0 as i32 {
        if sum != 0 as u32 { sum } else { 1 as u32 }
    } else {
        0 as u32
    };
}

pub unsafe extern "C" fn geno_art_hitbox_on(
    mut gobj: *mut Item_GObj,
    mut ip: *mut Item,
    mut p: i32,
    mut a: i32,
    mut h: i32,
    mut slot: i32,
) {
    melee_core::ct!("geno_art_hitbox_on");
    let mut hb: *mut ItemHitbox = (&raw mut (*ip).x5D4_hitboxes as *mut ItemHitbox)
        .offset(slot as isize) as *mut ItemHitbox;
    let mut hit: *mut HitCapsule = &raw mut (*hb).hit;
    let mut fl: u32 = Geno_ArticleHitParam(
        p,
        a,
        h,
        GENO_AH_FLAGS as i32,
    ) as u32;
    let mut dmg: f32 = geno_art_hf(p, a, h, GENO_AH_DAMAGE as i32);
    if (*hit).state as i32 == HitCapsule_Disabled as i32 {
        (*hit).x4 = 0 as u32;
        (*hit).state = HitCapsule_Enabled;
        (*ip).xDC8_word.flags.set_x16(1 as u32 as u32);
        (*ip).anon_daa.xDAA_flag.anon_0.set_b2(0 as u8 as u8);
        it_8026FCF8(ip, hit);
    }
    (*hit).jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    ScriptGame_HitRuleForget(hit);
    it_80272460(hit, ((*ip).xC3C * ((dmg + 0.5f32) * (*ip).xC40)) as u32, gobj);
    (*hit).scale = geno_art_hf(p, a, h, GENO_AH_SIZE as i32)
        as f32;
    (*ip).x3C = (*hit).scale as f32;
    it_80275594(gobj, slot as i32, 1.0f32 / (*ip).scl);
    (*hit).b_offset.x = geno_art_hf(p, a, h, GENO_AH_OFF_X as i32);
    (*hit).b_offset.y = geno_art_hf(p, a, h, GENO_AH_OFF_Y as i32);
    (*hit).b_offset.z = geno_art_hf(p, a, h, GENO_AH_OFF_Z as i32);
    (*hit).kb_angle = Geno_ArticleHitParam(p, a, h, GENO_AH_ANGLE as i32);
    (*hit).x24 = Geno_ArticleHitParam(p, a, h, GENO_AH_KBG as i32)
        as u32;
    (*hit).x28 = Geno_ArticleHitParam(p, a, h, GENO_AH_WKB as i32)
        as u32;
    (*hit).set_x43_b1(0 as u8 as u8);
    (*hit).x2C = Geno_ArticleHitParam(p, a, h, GENO_AH_BKB as i32)
        as u32;
    (*hit).element = Geno_ArticleHitParam(p, a, h, GENO_AH_ELEMENT as i32)
        as u32;
    (*hit).set_x40_b0(0 as u16 as u16);
    (*hit).set_x40_b1(0 as u16 as u16);
    (*hit).x34 = Geno_ArticleHitParam(
        p,
        a,
        h,
        GENO_AH_SHIELD_DAMAGE as i32,
    );
    (*hit).sfx_severity = Geno_ArticleHitParam(
        p,
        a,
        h,
        GENO_AH_SFX_SEVERITY as i32,
    );
    (*hit).sfx_kind = Geno_ArticleHitParam(
        p,
        a,
        h,
        GENO_AH_SFX_KIND as i32,
    ) as enum_t;
    (*hit)
        .set_x40_b2(
            (fl as u32 & GENO_AHF_AIR != 0 as u32)
                as i32 as u16 as u16,
        );
    (*hit)
        .set_x40_b3(
            (fl as u32 & GENO_AHF_GROUND != 0 as u32)
                as i32 as u16 as u16,
        );
    (*hit).set_x40_b4(0 as u16 as u16);
    (*hit).set_x41_b4(0 as u16 as u16);
    (*hit).set_x41_b5(0 as u16 as u16);
    (*hit).set_x41_b6(0 as u16 as u16);
    (*hit)
        .set_x41_b7(
            (fl as u32 & GENO_AHF_REFLECT != 0 as u32)
                as i32 as u16 as u16,
        );
    (*hit)
        .set_x42_b0(
            (fl as u32 & GENO_AHF_ABSORB != 0 as u32)
                as i32 as u8 as u8,
        );
    (*hit)
        .set_x42_b1(
            (fl as u32 & GENO_AHF_COUNTER != 0 as u32)
                as i32 as u8 as u8,
        );
    (*hit).set_x42_b2(0 as u8 as u8);
    (*hit).set_x42_b3(0 as u8 as u8);
    (*hit).set_x42_b4(1 as u8 as u8);
    (*hit).set_x42_b5(1 as u8 as u8);
    (*hit).set_x42_b6(0 as u8 as u8);
    (*hit).set_x42_b7(0 as u8 as u8);
    (*hit).set_x43_b0(0 as u8 as u8);
    (*hb).set_x138(1 as u8 as u8);
    (*hit).set_x43_b2(0 as u8 as u8);
    ScriptGame_HitRuleItemCreate(ip, hit);
    if !HSD_GObj_CurrentInvokedProc.is_null()
        && (*HSD_GObj_CurrentInvokedProc).s_link as i32
            > 11 as i32
    {
        it_8027129C(gobj, slot as i32);
    }
}

pub unsafe extern "C" fn geno_art_hitboxes(
    mut gobj: *mut Item_GObj,
    mut ip: *mut Item,
    mut v: *mut GenoArtVars,
) {
    melee_core::ct!("geno_art_hitboxes");
    let mut n: i32 = Geno_ArticleHitCount(
        (*v).profile as i32,
        (*v).article as i32,
    );
    let mut h: i32 = 0;
    let mut sl: i32 = 0;
    let mut want: [i32; 4] = [0; 4];
    sl = 0 as i32;
    while sl < GENO_ART_HITBOXES {
        want[sl as usize] = -(1 as i32);
        sl += 1;
    }
    h = 0 as i32;
    while h < n && h < GENO_ART_HIT_ENTRIES {
        let mut start: i32 = Geno_ArticleHitParam(
            (*v).profile as i32,
            (*v).article as i32,
            h,
            GENO_AH_START as i32,
        );
        let mut end: i32 = Geno_ArticleHitParam(
            (*v).profile as i32,
            (*v).article as i32,
            h,
            GENO_AH_END as i32,
        );
        if (*v).frame >= start && (end <= 0 as i32 || (*v).frame <= end) {
            want[(Geno_ArticleHitParam(
                (*v).profile as i32,
                (*v).article as i32,
                h,
                GENO_AH_SLOT as i32,
            ) & GENO_ART_HITBOXES - 1 as i32) as usize] = h;
        }
        h += 1;
    }
    sl = 0 as i32;
    while sl < GENO_ART_HITBOXES {
        if want[sl as usize] < 0 as i32 {
            (*ip).x5D4_hitboxes[sl as usize].hit.state = HitCapsule_Disabled;
        } else if want[sl as usize] != (*v).cur[sl as usize] as i32
            || (*ip).x5D4_hitboxes[sl as usize].hit.state as i32
                == HitCapsule_Disabled as i32
        {
            geno_art_hitbox_on(
                gobj,
                ip,
                (*v).profile as i32,
                (*v).article as i32,
                want[sl as usize],
                sl,
            );
        }
        (*v).cur[sl as usize] = want[sl as usize] as i8;
        sl += 1;
    }
}

pub unsafe extern "C" fn GenoGame_ArticleStep(
    mut p: i32,
    mut a: i32,
    mut frame: i32,
    mut vx: *mut f32,
    mut vy: *mut f32,
    mut px: f32,
    mut py: f32,
    mut has_target: i32,
    mut tx: f32,
    mut ty: f32,
) {
    melee_core::ct!("GenoGame_ArticleStep");
    let mut grav: f32 = geno_art_f(p, a, GENO_AP_GRAVITY as i32);
    let mut mf: f32 = geno_art_f(p, a, GENO_AP_MAX_FALL as i32);
    let mut acc: f32 = geno_art_f(p, a, GENO_AP_ACCEL as i32);
    let mut ms: f32 = geno_art_f(p, a, GENO_AP_MAX_SPEED as i32);
    let mut turn: f32 = geno_art_f(p, a, GENO_AP_HOMING_TURN as i32);
    let mut range: f32 = geno_art_f(p, a, GENO_AP_HOMING_RANGE as i32);
    let mut delay: f32 = geno_art_f(p, a, GENO_AP_HOMING_DELAY as i32);
    let mut speed: f32 = sqrtf(fmadds(*vx, *vx, *vy * *vy)) as f32;
    if acc != 0.0f32 && speed > 0.0001f32 {
        let mut ns: f32 = speed + acc;
        let mut mn: f32 = geno_art_f(p, a, GENO_AP_MIN_SPEED as i32);
        if acc < 0.0f32 && ns < mn {
            ns = if speed < mn { speed } else { mn };
        }
        if ns < 0.0f32 {
            ns = 0.0f32 as f32;
        }
        *vx *= ns / speed;
        *vy *= ns / speed;
        speed = ns;
    }
    if turn > 0.0f32 && has_target != 0 && frame as f32 > delay && speed > 0.0001f32 {
        let mut dx: f32 = tx - px;
        let mut dy: f32 = ty - py;
        if range <= 0.0f32 || fmadds(dx, dx, dy * dy) <= range * range {
            let mut cur: f32 = atan2f(*vy, *vx) as f32;
            let mut want: f32 = atan2f(
                dy as f32,
                dx as f32,
            ) as f32;
            let mut d: f32 = want - cur;
            let mut lim: f32 = turn * M_PI as f32 / 180.0f32;
            while d > M_PI as f32 {
                d -= (2.0f32 * M_PI as f32) as f32;
            }
            while d < -(M_PI as f32) {
                d += (2.0f32 * M_PI as f32) as f32;
            }
            d = if d > lim { lim } else if d < -lim { -lim } else { d };
            cur += d;
            *vx = (speed as f32 * cosf(cur as f32))
                as f32;
            *vy = (speed as f32 * sinf(cur as f32))
                as f32;
        }
    }
    *vy -= grav;
    if mf > 0.0f32 && *vy < -mf {
        *vy = -mf;
    }
    if ms > 0.0f32 {
        speed = sqrtf(fmadds(*vx, *vx, *vy * *vy)) as f32;
        if speed > ms {
            *vx *= ms / speed;
            *vy *= ms / speed;
        }
    }
}

pub unsafe extern "C" fn geno_art_target(
    mut ip: *mut Item,
    mut tx: *mut f32,
    mut ty: *mut f32,
) -> i32 {
    melee_core::ct!("geno_art_target");
    let mut cur: *mut HSD_GObj = ::core::ptr::null_mut::<HSD_GObj>();
    let mut best: f32 = -1.0f32;
    cur = *HSD_GObjPLinkHead.offset(HSD_GOBJ_PLINK_FIGHTER as isize);
    while !cur.is_null() {
        let mut fp: *mut Fighter = HSD_GObjGetUserData(cur) as *mut Fighter;
        let mut dx: f32 = 0.;
        let mut dy: f32 = 0.;
        let mut d: f32 = 0.;
        if !(fp.is_null() || cur == (*ip).owner
            || !(*ip).owner.is_null()
                && (*fp).player_id as i32
                    == (*(HSD_GObjGetUserData((*ip).owner) as *mut Fighter)).player_id
                        as i32)
        {
            dx = (*fp).cur_pos.x - (*ip).pos.x;
            dy = (*fp).cur_pos.y + 5.0f32 - (*ip).pos.y;
            d = fmadds(dx, dx, dy * dy);
            if best < 0.0f32 || d < best {
                best = d;
                *tx = (*fp).cur_pos.x;
                *ty = ((*fp).cur_pos.y as f32 + 5.0f32) as f32;
            }
        }
        cur = (*cur).next as *mut HSD_GObj;
    }
    return (best >= 0.0f32) as i32;
}

pub unsafe extern "C" fn geno_art_sound(
    mut owner: *mut HSD_GObj,
    mut p: i32,
    mut a: i32,
    mut id_param: i32,
    mut vol_param: i32,
    mut what: *const i8,
) {
    melee_core::ct!("geno_art_sound");
    let mut id: i32 = Geno_ArticleParam(p, a, id_param);
    let mut vol: i32 = Geno_ArticleParam(p, a, vol_param);
    if id <= 0 as i32 || owner.is_null() {
        return;
    }
    ft_PlaySFX(
        HSD_GObjGetUserData(owner) as *mut Fighter,
        id as enum_t,
        vol as u8,
        64 as u8,
    );
    OSReport!(
        b"geno: article sound: %s of article %d plays sound %d at volume %d\n\0"
            as *const u8 as *const i8 as *mut i8,
        what,
        a,
        id,
        vol,
    );
}

pub unsafe extern "C" fn geno_art_gone(
    mut ip: *mut Item,
    mut v: *mut GenoArtVars,
    mut why: i32,
) {
    melee_core::ct!("geno_art_gone");
    let mut o: *mut HSD_GObj = (*ip).owner;
    (*v).why = why as i32;
    geno_art_sound(
        o,
        (*v).profile as i32,
        (*v).article as i32,
        GENO_AP_END_SFX as i32,
        GENO_AP_END_VOL as i32,
        b"end\0" as *const u8 as *const i8,
    );
    Geno_Event(
        29 as i32,
        if !o.is_null() {
            (*(HSD_GObjGetUserData(o) as *mut Fighter)).kind as i32
        } else {
            -(1 as i32)
        },
        if !o.is_null() {
            (*(HSD_GObjGetUserData(o) as *mut Fighter)).player_id as i32
        } else {
            -(1 as i32)
        },
        (*v).article as i32,
        why,
    );
}

pub unsafe extern "C" fn geno_art_joint(
    mut gobj: *mut Item_GObj,
    mut k: i32,
) -> *mut HSD_JObj {
    melee_core::ct!("geno_art_joint");
    let mut j: *mut HSD_JObj = HSD_GObjGetHSDObj(gobj as *mut HSD_GObj) as *mut HSD_JObj;
    while !j.is_null() && k > 0 as i32 {
        if !HSD_JObjGetChild(j).is_null() {
            j = HSD_JObjGetChild(j);
        } else {
            while !j.is_null() && HSD_JObjGetNext(j).is_null() {
                j = HSD_JObjGetParent(j);
            }
            j = if !j.is_null() {
                HSD_JObjGetNext(j)
            } else {
                ::core::ptr::null_mut::<HSD_JObj>()
            };
        }
        k -= 1;
    }
    return j;
}

pub unsafe extern "C" fn geno_art_effects(
    mut ig: *mut Item_GObj,
    mut ip: *mut Item,
    mut p: i32,
    mut a: i32,
    mut frame: i32,
    mut facing: f32,
) -> i32 {
    melee_core::ct!("geno_art_effects");
    static mut zero: Vec = Vec {
        x: 0.0f32,
        y: 0.0f32,
        z: 0.0f32,
    };
    let mut e: i32 = 0;
    let mut n: i32 = 0 as i32;
    e = -(1 as i32);
    while e < 8 as i32 {
        let mut id: i32 = Geno_ArticleParam(
            p,
            a,
            if e < 0 as i32 {
                GENO_AP_EFFECT as i32
            } else {
                GENO_AP_EFFECTS as i32 + e
            },
        );
        let mut at: i32 = if e < 0 as i32 {
            0 as i32
        } else {
            Geno_ArticleParam(p, a, GENO_AP_EFFECT_AT as i32 + e)
        };
        let mut j: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
        let mut c: i32 = 0;
        let mut count: i32 = (at as u32 >> 24 as i32)
            .wrapping_add(1 as u32) as i32;
        if !(id <= 0 as i32 || at & 0xffff as i32 != frame)
        {
            j = geno_art_joint(
                ig,
                at >> 16 as i32 & 0xff as i32,
            );
            if j.is_null() {
                j = HSD_GObjGetHSDObj(ig as *mut HSD_GObj) as *mut HSD_JObj;
            }
            c = 0 as i32;
            while c < count {
                let mut off: Vec = zero;
                if id + c >= EF_MEX_MDL_START && id + c < EF_MEX_END {
                    efAsync_MexSpawn(
                        ig as *mut HSD_GObj,
                        &raw mut (*ip).xBC0 as *mut c_void,
                        id as i32 + c as i32,
                        j,
                        &raw mut off,
                        facing,
                        0.0f32,
                    );
                } else {
                    efAsync_Spawn(
                        ig as *mut HSD_GObj,
                        &raw mut (*ip).xBC0 as *mut c_void,
                        1 as u32,
                        (id + c) as u32,
                        j,
                    );
                }
                n += 1;
                c += 1;
            }
        }
        e += 1;
    }
    if n > 0 as i32 {
        efAsync_QueueFlush(
            ig as *mut HSD_GObj,
            &raw mut (*ip).xBC0 as *mut c_void,
        );
        Geno_Event(37 as i32, a, n, frame, 0 as i32);
    }
    return n;
}

pub unsafe extern "C" fn geno_art_children(
    mut gobj: *mut Item_GObj,
    mut ip: *mut Item,
    mut v: *mut GenoArtVars,
) {
    melee_core::ct!("geno_art_children");
    let mut c: i32 = 0;
    c = 0 as i32;
    while c < GENO_ART_CHILDREN {
        let mut base: i32 = GENO_AP_CHILD as i32
            + 5 as i32 * c;
        let mut art: i32 = Geno_ArticleParam(
            (*v).profile as i32,
            (*v).article as i32,
            base,
        );
        let mut f0: i32 = Geno_ArticleParam(
            (*v).profile as i32,
            (*v).article as i32,
            base + 1 as i32,
        );
        let mut every: i32 = Geno_ArticleParam(
            (*v).profile as i32,
            (*v).article as i32,
            base + 2 as i32,
        );
        let mut count: i32 = Geno_ArticleParam(
            (*v).profile as i32,
            (*v).article as i32,
            base + 3 as i32,
        );
        let mut k: i32 = 0;
        if !(art < 0 as i32 || (*v).frame < f0 || (*ip).owner.is_null()) {
            k = if every > 0 as i32 {
                ((*v).frame as i32 - f0) / every
            } else {
                0 as i32
            };
            if (if every > 0 as i32 {
                (((*v).frame as i32 - f0) % every
                    == 0 as i32) as i32
            } else {
                ((*v).frame == f0) as i32
            }) != 0 && k < count
            {
                geno_art_spawn(
                    (*ip).owner,
                    (*v).profile as i32,
                    art,
                    Geno_ArticleParam(
                        (*v).profile as i32,
                        (*v).article as i32,
                        base + 4 as i32,
                    ),
                    0.0f32,
                    &raw mut (*ip).pos,
                    (*ip).facing_dir,
                );
            }
        }
        c += 1;
    }
}

pub unsafe extern "C" fn geno_art_fx_census(
    mut gobj: *mut Item_GObj,
    mut v: *mut GenoArtVars,
) {
    melee_core::ct!("geno_art_fx_census");
    let mut g: *mut HSD_Generator = ::core::ptr::null_mut::<HSD_Generator>();
    let mut j0: *mut HSD_JObj = HSD_GObjGetHSDObj(gobj as *mut HSD_GObj)
        as *mut HSD_JObj;
    let mut ip: *mut Item = HSD_GObjGetUserData(gobj as *mut HSD_GObj) as *mut Item;
    let mut lists: [*mut u8; 32] = [::core::ptr::null_mut::<u8>(); 32];
    let mut ngen: i32 = 0 as i32;
    let mut per: [i32; 32] = [0; 32];
    let mut total: i32 = 0 as i32;
    let mut i: i32 = 0;
    let mut k: i32 = 0;
    let mut sdx: f32 = 0.0f32;
    let mut sdy: f32 = 0.0f32;
    let mut svx: f32 = 0.0f32;
    k = 0 as i32;
    while k < 32 as i32 {
        per[k as usize] = 0 as i32;
        k += 1;
    }
    g = hsd_804D78FC;
    while !g.is_null() {
        let mut gj: *mut HSD_JObj = (*g).jobj;
        while !gj.is_null() && gj != j0 {
            gj = HSD_JObjGetParent(gj);
        }
        if gj == j0 && !(*g).jobj.is_null() && ngen < 32 as i32 {
            k = ngen;
            while k > 0 as i32 {
                lists[k as usize] = lists[(k - 1 as i32) as usize];
                k -= 1;
            }
            lists[0 as i32 as usize] = (*g).cmdList;
            ngen += 1;
        }
        g = (*g).next;
    }
    i = 0 as i32;
    while i < 16 as i32 {
        let mut pp: *mut HSD_Particle = ::core::ptr::null_mut::<HSD_Particle>();
        pp = hsd_804D0908[i as usize];
        while !pp.is_null() {
            k = 0 as i32;
            while k < ngen {
                if (*pp).cmdList == lists[k as usize] {
                    per[k as usize] += 1;
                    total += 1;
                    sdx = fmadds((*pp).pos.x - (*ip).pos.x, (*ip).facing_dir, sdx);
                    sdy += (*pp).pos.y - (*ip).pos.y;
                    svx = fmadds((*pp).vel.x, (*ip).facing_dir, svx);
                    break;
                } else {
                    k += 1;
                }
            }
            pp = (*pp).next;
        }
        i += 1;
    }
    Geno_Event(
        38 as i32,
        (*v).article as i32,
        (*v).frame as i32,
        ngen,
        total,
    );
    Geno_Event(
        39 as i32,
        per[0 as i32 as usize] + per[1 as i32 as usize]
            + per[2 as i32 as usize]
            + per[3 as i32 as usize] << 8 as i32
            | per[4 as i32 as usize]
                + per[5 as i32 as usize]
                + per[6 as i32 as usize]
                + per[7 as i32 as usize],
        per[8 as i32 as usize] + per[9 as i32 as usize]
            + per[10 as i32 as usize]
            + per[11 as i32 as usize] << 8 as i32
            | per[12 as i32 as usize]
                + per[13 as i32 as usize]
                + per[14 as i32 as usize]
                + per[15 as i32 as usize],
        per[16 as i32 as usize] + per[17 as i32 as usize]
            + per[18 as i32 as usize]
            + per[19 as i32 as usize] << 8 as i32
            | per[20 as i32 as usize]
                + per[21 as i32 as usize]
                + per[22 as i32 as usize]
                + per[23 as i32 as usize],
        per[24 as i32 as usize] + per[25 as i32 as usize]
            + per[26 as i32 as usize]
            + per[27 as i32 as usize] << 8 as i32
            | per[28 as i32 as usize]
                + per[29 as i32 as usize]
                + per[30 as i32 as usize]
                + per[31 as i32 as usize],
    );
    if total > 0 as i32 {
        Geno_Event(
            41 as i32,
            (100.0f32 * sdx as f32 / total as f32)
                as i32,
            (100.0f32 * sdy as f32 / total as f32)
                as i32,
            (1000.0f32 * svx as f32 / total as f32)
                as i32,
            total,
        );
    }
}

pub unsafe extern "C" fn geno_art_anim(mut gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("geno_art_anim");
    let mut ip: *mut Item = HSD_GObjGetUserData(gobj as *mut HSD_GObj) as *mut Item;
    let mut v: *mut GenoArtVars = geno_art_vars(ip);
    let mut spin: f32 = geno_art_f(
        (*v).profile as i32,
        (*v).article as i32,
        GENO_AP_SPIN as i32,
    );
    (*v).frame += 1;
    if (*v).frame as f32
        > geno_art_f(
            (*v).profile as i32,
            (*v).article as i32,
            GENO_AP_LIFETIME as i32,
        )
    {
        geno_art_gone(ip, v, 0 as i32);
        return 1;
    }
    geno_art_hitboxes(gobj, ip, v);
    geno_art_children(gobj, ip, v);
    geno_art_effects(
        gobj,
        ip,
        (*v).profile as i32,
        (*v).article as i32,
        (*v).frame as i32,
        (*ip).facing_dir,
    );
    let mut k: i32 = 0;
    k = 0 as i32;
    while k < 4 as i32 {
        let mut sp: f32 = geno_art_f(
            (*v).profile as i32,
            (*v).article as i32,
            GENO_AP_SPINS as i32 + 2 as i32 * k
                + 1 as i32,
        );
        let mut sj: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
        if !(sp == 0.0f32) {
            sj = geno_art_joint(
                gobj,
                Geno_ArticleParam(
                    (*v).profile as i32,
                    (*v).article as i32,
                    GENO_AP_SPINS as i32 + 2 as i32 * k,
                ),
            );
            if !sj.is_null() {
                HSD_JObjSetRotationZ(sj, sp * (*v).frame as f32);
            }
        }
        k += 1;
    }
    if (*v).frame == 5 as i32 || (*v).frame == 10 as i32
        || (*v).frame == 20 as i32
        || (*v).frame == 30 as i32
    {
        geno_art_fx_census(gobj, v);
    }
    if spin != 0.0f32 {
        (*v).spin
            += spin as f32 * M_PI as f32 / 180.0f32;
        HSD_JObjSetRotationX(
            HSD_GObjGetHSDObj(gobj as *mut HSD_GObj) as *mut HSD_JObj,
            (*v).spin,
        );
    }
    return 0;
}

pub unsafe extern "C" fn geno_art_phys(mut gobj: *mut Item_GObj) {
    melee_core::ct!("geno_art_phys");
    let mut ip: *mut Item = HSD_GObjGetUserData(gobj as *mut HSD_GObj) as *mut Item;
    let mut v: *mut GenoArtVars = geno_art_vars(ip);
    let mut tx: f32 = 0.0f32;
    let mut ty: f32 = 0.0f32;
    let mut has: i32 = (geno_art_f(
        (*v).profile as i32,
        (*v).article as i32,
        GENO_AP_HOMING_TURN as i32,
    ) > 0.0f32 && geno_art_target(ip, &raw mut tx, &raw mut ty) != 0)
        as i32;
    (*v).prev = (*ip).pos;
    GenoGame_ArticleStep(
        (*v).profile as i32,
        (*v).article as i32,
        (*v).frame as i32,
        &raw mut (*v).vx,
        &raw mut (*v).vy,
        (*ip).pos.x,
        (*ip).pos.y,
        has,
        tx,
        ty,
    );
    (*ip).x40_vel.x = (*v).vx;
    (*ip).x40_vel.y = (*v).vy;
    (*ip).x40_vel.z = 0.0f32 as f32;
}

pub unsafe extern "C" fn geno_art_coll(mut gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("geno_art_coll");
    let mut ip: *mut Item = HSD_GObjGetUserData(gobj as *mut HSD_GObj) as *mut Item;
    let mut v: *mut GenoArtVars = geno_art_vars(ip);
    if Geno_ArticleParam(
        (*v).profile as i32,
        (*v).article as i32,
        GENO_AP_DESPAWN as i32,
    ) as u32 & GENO_ART_DESPAWN_STAGE != 0
        && it_8026E9A4(
            gobj as *mut HSD_GObj,
            &raw mut (*v).prev,
            &raw mut (*ip).pos,
            ::core::ptr::null_mut::<Vec>(),
        ) != 0
    {
        Geno_Event(
            35 as i32,
            (*v).article as i32,
            ((*v).prev.x as f32 * 100.0f32) as i32,
            ((*v).prev.y as f32 * 100.0f32) as i32,
            ((*ip).pos.y as f32 * 100.0f32) as i32,
        );
        geno_art_gone(ip, v, 3 as i32);
        return 1;
    }
    return 0;
}

pub unsafe extern "C" fn geno_art_react(
    mut gobj: *mut Item_GObj,
    mut bit: u32,
    mut why: i32,
) -> i32 {
    melee_core::ct!("geno_art_react");
    let mut ip: *mut Item = HSD_GObjGetUserData(gobj as *mut HSD_GObj) as *mut Item;
    let mut v: *mut GenoArtVars = geno_art_vars(ip);
    if Geno_ArticleParam(
        (*v).profile as i32,
        (*v).article as i32,
        GENO_AP_DESPAWN as i32,
    ) as u32 & bit != 0
    {
        geno_art_gone(ip, v, why);
        return 1;
    }
    return 0;
}

pub unsafe extern "C" fn geno_art_hit(mut gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("geno_art_hit");
    return geno_art_react(gobj, GENO_ART_DESPAWN_HIT, 1 as i32);
}

pub unsafe extern "C" fn geno_art_shield(mut gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("geno_art_shield");
    return geno_art_react(gobj, GENO_ART_DESPAWN_SHIELD, 2 as i32);
}

pub unsafe extern "C" fn geno_art_clank(mut gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("geno_art_clank");
    return geno_art_react(gobj, GENO_ART_DESPAWN_CLANK, 4 as i32);
}

pub unsafe extern "C" fn geno_art_reflected(mut gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("geno_art_reflected");
    let mut ip: *mut Item = HSD_GObjGetUserData(gobj as *mut HSD_GObj) as *mut Item;
    let mut v: *mut GenoArtVars = geno_art_vars(ip);
    if !(*ip).owner.is_null() {
        Fx_SetOwnerPort(
            HSD_GObjGetHSDObj(gobj as *mut HSD_GObj) as *mut HSD_JObj
                as i32,
            (*(HSD_GObjGetUserData((*ip).owner) as *mut Fighter)).player_id
                as i32 + 1 as i32,
        );
    }
    if (*ip).facing_dir != (*ip).xC68 {
        (*ip).facing_dir = (*ip).xC68;
        HSD_JObjSetRotationY(
            HSD_GObjGetHSDObj(gobj as *mut HSD_GObj) as *mut HSD_JObj,
            (M_PI / 2 as i32 as f64) as f32
                * (*ip).facing_dir,
        );
    }
    (*v).vx = (if (*v).vx < 0.0f32 { -(*v).vx } else { (*v).vx }) * (*ip).facing_dir
        * (*ip).xC70;
    (*v).vy *= (*ip).xC70;
    return 0;
}

pub unsafe extern "C" fn geno_art_absorbed(mut gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("geno_art_absorbed");
    geno_art_gone(
        HSD_GObjGetUserData(gobj as *mut HSD_GObj) as *mut Item,
        geno_art_vars(HSD_GObjGetUserData(gobj as *mut HSD_GObj) as *mut Item),
        1 as i32,
    );
    return 1;
}

pub unsafe extern "C" fn geno_art_spawn(
    mut owner: *mut HSD_GObj,
    mut p: i32,
    mut a: i32,
    mut variant: i32,
    mut angle: f32,
    mut base: *mut Vec,
    mut facing: f32,
) -> *mut Item_GObj {
    melee_core::ct!("geno_art_spawn");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(owner) as *mut Fighter;
    let mut kind: i32 = 0;
    let mut max: i32 = 0;
    let mut nv: i32 = 0;
    let mut spawn: SpawnItem = unsafe { core::mem::zeroed() };
    let mut pos: Vec = unsafe { core::mem::zeroed() };
    let mut ig: *mut Item_GObj = ::core::ptr::null_mut::<Item_GObj>();
    let mut ip: *mut Item = ::core::ptr::null_mut::<Item>();
    let mut v: *mut GenoArtVars = ::core::ptr::null_mut::<GenoArtVars>();
    let mut fwd: f32 = 0.;
    let mut up: f32 = 0.;
    let mut vf: f32 = 0.;
    let mut vu: f32 = 0.;
    let mut rad: f32 = 0.;
    let mut c: f32 = 0.;
    let mut sn: f32 = 0.;
    if p < 0 as i32 || p >= GENO_MAX_PROFILES
        || a < 0 as i32 || a >= Geno_ArticleCount(p)
    {
        Geno_Event(
            30 as i32,
            (*fp).kind as i32,
            (*fp).player_id as i32,
            a,
            0 as i32,
        );
        return ::core::ptr::null_mut::<Item_GObj>();
    }
    max = Geno_ArticleParam(p, a, GENO_AP_MAX_LIVE as i32);
    if max > 0 as i32 && geno_art_live(owner, a) >= max {
        Geno_Event(
            30 as i32,
            (*fp).kind as i32,
            (*fp).player_id as i32,
            a,
            1 as i32,
        );
        return ::core::ptr::null_mut::<Item_GObj>();
    }
    geno_art_build(p, a);
    kind = if p >= 32 as i32 {
        GENO_ART_EXTRA_BASE + (p - 32 as i32) * 16 as i32
            + a
    } else if a < GENO_ART_PER_RANGE {
        GENO_ART_KIND_BASE + p * GENO_ART_PER_RANGE + a
    } else {
        GENO_ART_KIND_BASE2 + p * GENO_ART_PER_RANGE + a - GENO_ART_PER_RANGE
    };
    nv = Geno_ArticleParam(p, a, GENO_AP_SPAWN_N as i32);
    if variant > 0 as i32 && variant < nv {
        fwd = geno_art_f(
            p,
            a,
            GENO_AP_SPAWN_V as i32 + 2 as i32 * variant,
        );
        up = geno_art_f(
            p,
            a,
            GENO_AP_SPAWN_V as i32 + 2 as i32 * variant
                + 1 as i32,
        );
    } else if nv > 0 as i32 {
        fwd = geno_art_f(p, a, GENO_AP_SPAWN_V as i32);
        up = geno_art_f(
            p,
            a,
            GENO_AP_SPAWN_V as i32 + 1 as i32,
        );
    } else {
        fwd = geno_art_f(p, a, GENO_AP_SPAWN_FWD as i32);
        up = geno_art_f(p, a, GENO_AP_SPAWN_UP as i32);
    }
    pos.x = fmadds(facing, fwd, (*base).x);
    pos.y = (*base).y + up;
    pos.z = 0.0f32 as f32;
    spawn.kind = kind as ItemKind;
    Item_InitSpawnPositionFromParent(&raw mut spawn, owner, &raw mut pos);
    Item_InitSpawnCommonFields(&raw mut spawn, owner, facing, 1);
    ig = Item_80268B18(&raw mut spawn);
    if ig.is_null() {
        Geno_Event(
            30 as i32,
            (*fp).kind as i32,
            (*fp).player_id as i32,
            a,
            2 as i32,
        );
        return ::core::ptr::null_mut::<Item_GObj>();
    }
    ip = HSD_GObjGetUserData(ig as *mut HSD_GObj) as *mut Item;
    v = geno_art_vars(ip);
    geno_zero(
        v as *mut c_void,
        ::core::mem::size_of::<GenoArtVars>() as i32,
    );
    (*v).profile = p as i32;
    (*v).article = a as i32;
    nv = 0 as i32;
    while nv < GENO_ART_HITBOXES {
        (*v).cur[nv as usize] = -(1 as i32) as i8;
        nv += 1;
    }
    vf = geno_art_f(p, a, GENO_AP_VEL_FWD as i32);
    vu = geno_art_f(p, a, GENO_AP_VEL_UP as i32);
    rad = ((geno_art_f(p, a, GENO_AP_ANGLE as i32) as f32
        + angle as f32) * M_PI as f32 / 180.0f32)
        as f32;
    c = cosf(rad as f32) as f32;
    sn = sinf(rad as f32) as f32;
    (*v).vx = facing * fmadds(vf, c, -(vu * sn));
    (*v).vy = fmadds(vf, sn, vu * c);
    (*v).prev = (*ip).pos;
    Item_80268E5C(ig as *mut HSD_GObj, 0 as enum_t, ITEM_ANIM_UPDATE);
    (*ip).x40_vel.x = (*v).vx;
    (*ip).x40_vel.y = (*v).vy;
    (*ip).x40_vel.z = 0.0f32 as f32;
    db_80225DD8(ig, owner as *mut Fighter_GObj);
    geno_art_sound(
        owner,
        p,
        a,
        GENO_AP_SPAWN_SFX as i32,
        GENO_AP_SPAWN_VOL as i32,
        b"spawn\0" as *const u8 as *const i8,
    );
    geno_art_effects(ig, ip, p, a, 0 as i32, facing);
    if Geno_ArticleParam(p, a, GENO_AP_FX as i32)
        > 0 as i32
    {
        HSD_JObjSetTranslate(
            HSD_GObjGetHSDObj(ig as *mut HSD_GObj) as *mut HSD_JObj,
            &raw mut (*ip).pos,
        );
        HSD_JObjSetupMatrix(HSD_GObjGetHSDObj(ig as *mut HSD_GObj) as *mut HSD_JObj);
        let mut h: i32 = Fx_Attach(
            Geno_ArticleParam(p, a, GENO_AP_FX as i32)
                - 1 as i32,
            HSD_GObjGetHSDObj(ig as *mut HSD_GObj) as *mut HSD_JObj
                as i32,
            0x44 as i32,
            geno_fx_frame,
            if facing < 0.0f32 {
                -(1 as i32)
            } else {
                1 as i32
            },
        );
        if h > 0 as i32 {
            Fx_SetOwner(
                h,
                2 as i32,
                (*fp).player_id as i32 + 1 as i32,
            );
        }
        if Geno_ArticleParam(p, a, GENO_AP_SHOW_MODEL as i32)
            == 0 as i32
        {
            geno_art_hide_model(HSD_GObjGetHSDObj(ig as *mut HSD_GObj) as *mut HSD_JObj);
            Geno_Event(
                45 as i32,
                a,
                2 as i32,
                0 as i32,
                0 as i32,
            );
        } else {
            Geno_Event(
                45 as i32,
                a,
                3 as i32,
                0 as i32,
                0 as i32,
            );
        }
    }
    let ref mut fresh3 = (*geno_state(fp)).art_spawned;
    *fresh3 += 1;
    Geno_Event(
        28 as i32,
        (*fp).kind as i32,
        (*fp).player_id as i32,
        a,
        kind,
    );
    let mut j: *mut HSD_JObj = HSD_GObjGetHSDObj(ig as *mut HSD_GObj) as *mut HSD_JObj;
    let mut nj: i32 = 0 as i32;
    let mut nd: i32 = 0 as i32;
    while !j.is_null() && nj < 64 as i32 {
        let mut d: *mut HSD_DObj = HSD_JObjGetDObj(j) as *mut HSD_DObj;
        nj += 1;
        while !d.is_null() {
            nd += 1;
            d = (*d).next;
        }
        j = if !HSD_JObjGetChild(j).is_null() {
            HSD_JObjGetChild(j)
        } else {
            HSD_JObjGetNext(j)
        };
    }
    Geno_Event(
        36 as i32,
        a,
        nj,
        nd,
        ((*ip).scl as f32 * 100.0f32) as i32,
    );
    Geno_Event(
        34 as i32,
        a,
        (pos.x as f32 * 100.0f32) as i32,
        (pos.y as f32 * 100.0f32) as i32,
        ((*v).vx as f32 * 1000.0f32) as i32,
    );
    return ig;
}

pub unsafe extern "C" fn GenoGame_ArticleSpawn(
    mut gobj: *mut Fighter_GObj,
    mut arg: i32,
) -> i32 {
    melee_core::ct!("GenoGame_ArticleSpawn");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj as *mut HSD_GObj)
        as *mut Fighter;
    let mut st: *mut GenoState = geno_state(fp);
    let mut a: i32 = arg & 0xff as i32;
    let mut bone: i32 = 0;
    let mut base: Vec = (*fp).cur_pos;
    if (*st).profile < 0 as i32
        || a >= Geno_ArticleCount((*st).profile as i32)
    {
        Geno_Event(
            30 as i32,
            (*fp).kind as i32,
            (*fp).player_id as i32,
            a,
            0 as i32,
        );
        return 0 as i32;
    }
    bone = Geno_ArticleParam(
        (*st).profile as i32,
        a,
        GENO_AP_BONE as i32,
    );
    if bone >= 0 as i32 && bone < 128 as i32
        && !(*fp).parts.is_null()
        && !(*(*fp).parts.offset(bone as isize)).joint.is_null()
    {
        lb_8000B1CC(
            (*(*fp).parts.offset(bone as isize)).joint as *mut HSD_JObj,
            ::core::ptr::null_mut::<Vec>(),
            &raw mut base,
        );
        base.z = 0.0f32 as f32;
    }
    return (geno_art_spawn(
        gobj as *mut HSD_GObj,
        (*st).profile as i32,
        a,
        arg >> 8 as i32 & 0xff as i32,
        (arg as u32 >> 16 as i32) as i16 as f32,
        &raw mut base,
        (*fp).facing_dir as f32,
    ) != ::core::ptr::null_mut::<Item_GObj>()) as i32;
}

pub unsafe extern "C" fn hook_article_spawn(
    mut gobj: *mut Fighter_GObj,
    mut fp: *mut Fighter,
    mut st: *mut GenoState,
    mut arg: i32,
) -> i32 {
    melee_core::ct!("hook_article_spawn");
    if geno_test_capture != 0 {
        (*st).art_spawned += 1;
        return 0 as i32;
    }
    GenoGame_ArticleSpawn(gobj, arg as i32);
    return 0 as i32;
}

pub unsafe extern "C" fn Geno_OnHit(mut gobj: *mut Fighter_GObj) -> i32 {
    melee_core::ct!("Geno_OnHit");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj as *mut HSD_GObj)
        as *mut Fighter;
    let mut st: *mut GenoState = geno_state(fp);
    let mut s: i32 = 0;
    let mut countered: i32 = 0 as i32;
    if (*st).profile < 0 as i32
        || (*fp).dmg.kb_applied == 0.0f32 && (*fp).dmg.x1838_percentTemp <= 0.0f32
    {
        return 0 as i32;
    }
    (*st).hit_count += 1;
    geno_combo_taken_hit(fp, st);
    (*st).hit_damage = (*fp).dmg.x1838_percentTemp as f32;
    (*st).hit_port = (if (*fp).dmg.x18c4_source_ply >= 0 as i32
        && (*fp).dmg.x18c4_source_ply < 6 as i32
    {
        (*fp).dmg.x18c4_source_ply
    } else {
        -(1 as i32)
    }) as i32;
    (*st).hit_counter = 0 as i32 as i32;
    Geno_Event(
        31 as i32,
        (*fp).kind as i32,
        (*fp).player_id as i32,
        ((*st).hit_damage as f32 * 100.0f32) as i32,
        (*st).hit_port as i32,
    );
    s = geno_cur_state(fp, st);
    if s >= 0 as i32
        && Geno_StateHasCounter((*st).profile as i32, s) != 0
        && (*st).action_time
            >= Geno_StateCounter(
                (*st).profile as i32,
                s,
                GENO_CTR_FROM as i32,
            )
        && (*st).action_time
            <= Geno_StateCounter(
                (*st).profile as i32,
                s,
                GENO_CTR_TO as i32,
            )
    {
        let mut t: u32 = Geno_StateCounter(
            (*st).profile as i32,
            s,
            GENO_CTR_TARGET as i32,
        ) as u32;
        countered = 1 as i32;
        (*st).hit_counter = 1 as i32 as i32;
        (*st).counters += 1;
        Geno_Event(
            32 as i32,
            (*fp).kind as i32,
            (*fp).player_id as i32,
            s,
            (*st).action_time as i32,
        );
        if Geno_StateCounter(
            (*st).profile as i32,
            s,
            GENO_CTR_NEGATE as i32,
        ) != 0
        {
            (*fp).dmg.kb_applied = 0.0f32;
            (*fp).dmg.x1838_percentTemp = 0.0f32;
        }
        if t != GENO_NONE_T {
            geno_go(gobj, fp, st, t);
        }
    }
    geno_run_event(gobj, st, GENO_EV_HIT as i32);
    return countered;
}

pub unsafe extern "C" fn GenoGame_ClearProfileStorage() {
    melee_core::ct!("GenoGame_ClearProfileStorage");
    Geno_Rows = ::core::ptr::null_mut::<*mut MotionState>();
    Geno_Params = ::core::ptr::null_mut::<*mut f32>();
    Geno_RowCount = ::core::ptr::null_mut::<i32>();
    Geno_ProfileCapacity = 0 as i32;
    Geno_ArtDesc = ::core::ptr::null_mut::<*mut Article>();
    Geno_ArtAttr = ::core::ptr::null_mut::<*mut ItemAttr>();
    Geno_ArtModel = ::core::ptr::null_mut::<*mut ItemModelDesc>();
    Geno_ArtProfileCapacity = 0 as i32;
    geno_zero(
        &raw mut Geno_ProfileStorage.bytes as *mut u8
            as *mut c_void,
        Geno_ProfileStorageUsed as i32,
    );
    Geno_ProfileStorageUsed = 0 as u32;
}

pub unsafe extern "C" fn geno_sfield(mut off: i32) -> i32 {
    melee_core::ct!("geno_sfield");
    let mut i: i32 = 0;
    i = 0 as i32;
    while i < GENO_NSF {
        if off >= geno_sfields[i as usize].off
            && off < geno_sfields[i as usize].off + geno_sfields[i as usize].size
        {
            return i;
        }
        i += 1;
    }
    return -(1 as i32);
}

pub unsafe extern "C" fn GenoGame_StateSize() -> i32 {
    melee_core::ct!("GenoGame_StateSize");
    return ::core::mem::size_of::<GenoState>() as i32;
}

pub unsafe extern "C" fn GenoGame_StateFieldName(
    mut off: i32,
) -> *const i8 {
    melee_core::ct!("GenoGame_StateFieldName");
    let mut i: i32 = geno_sfield(off);
    return if i >= 0 as i32 {
        geno_sfields[i as usize].name
    } else {
        ::core::ptr::null::<i8>()
    };
}

pub unsafe extern "C" fn GenoGame_StateFieldBase(
    mut off: i32,
) -> i32 {
    melee_core::ct!("GenoGame_StateFieldBase");
    let mut i: i32 = geno_sfield(off);
    return if i >= 0 as i32 { geno_sfields[i as usize].off } else { off };
}

pub unsafe extern "C" fn Geno_ItemActive(
    mut kind: i32,
) -> i32 {
    melee_core::ct!("Geno_ItemActive");
    let mut d: i32 = kind - GENO_ITEM_KIND_BASE;
    return (d >= 0 as i32 && d < GENO_ITEM_MAX_DEFS
        && Geno_ItemParam(d, GENO_IP_DEFINED as i32) != 0
        && Mex_ItemRangeAvailable(GENO_ITEM_KIND_BASE, GENO_ITEM_MAX_DEFS) != 0)
        as i32;
}

pub unsafe extern "C" fn geno_item_bits(mut f: f32) -> i32 {
    melee_core::ct!("geno_item_bits");
    let mut u: Anon_anon_35_2 = unsafe { core::mem::zeroed() };
    u.f = f;
    return u.i;
}

pub unsafe extern "C" fn geno_item_float(mut i: i32) -> f32 {
    melee_core::ct!("geno_item_float");
    let mut u: Anon_anon_35_2 = unsafe { core::mem::zeroed() };
    u.i = i;
    return u.f;
}

pub unsafe extern "C" fn geno_item_paramf(
    mut d: i32,
    mut f: i32,
) -> f32 {
    melee_core::ct!("geno_item_paramf");
    return geno_item_float(Geno_ItemParam(d, f));
}

pub unsafe extern "C" fn geno_item_def(mut ip: *mut Item) -> i32 {
    melee_core::ct!("geno_item_def");
    let mut d: i32 = (*ip).kind as i32
        - GENO_ITEM_KIND_BASE;
    return if d >= 0 as i32 && d < GENO_ITEM_MAX_DEFS
        && Geno_ItemActive((*ip).kind as i32) != 0
    {
        d
    } else {
        -(1 as i32)
    };
}

pub unsafe extern "C" fn geno_item_vars(mut ip: *mut Item) -> *mut GenoItemVars {
    melee_core::ct!("geno_item_vars");
    return &raw mut (*ip).xDD4_itemVar as *mut GenoItemVars;
}

pub unsafe extern "C" fn geno_item_event(
    mut ip: *mut Item,
    mut what: i32,
    mut port: i32,
) {
    melee_core::ct!("geno_item_event");
    let mut v: *mut GenoItemVars = geno_item_vars(ip);
    if (*v).terminal != 0 {
        return;
    }
    (*v).terminal = 1 as i32;
    ItemEvent(
        what,
        (*v).handle,
        geno_item_def(ip),
        port,
        geno_item_bits((*ip).pos.x),
        geno_item_bits((*ip).pos.y),
        (*v).colour,
        (*v).amount,
    );
}

pub unsafe extern "C" fn geno_item_lift(mut d: i32, mut age: f32) -> f32 {
    melee_core::ct!("geno_item_lift");
    return fmadds(geno_item_paramf(d, GENO_IP_BOB as i32), sinf(age * geno_item_paramf(d, GENO_IP_BOB_SPEED as i32) * M_PI as f32 / 180.0f32), geno_item_paramf(d, GENO_IP_HOVER as i32));
}

pub unsafe extern "C" fn geno_item_in_capsule(
    mut dx: f32,
    mut dy: f32,
    mut radius: f32,
    mut top: f32,
) -> i32 {
    melee_core::ct!("geno_item_in_capsule");
    if top < 0 as i32 as f32 {
        top = 0 as i32 as f32;
    }
    if dy > top {
        dy -= top;
    } else if dy > 0 as i32 as f32 {
        dy = 0 as i32 as f32;
    }
    return (fmadds(dx, dx, dy * dy) <= radius * radius) as i32;
}

pub unsafe extern "C" fn Geno_ItemPressDist(
    mut g: *mut HSD_GObj,
    mut fp: *mut Fighter,
) -> f32 {
    melee_core::ct!("Geno_ItemPressDist");
    let mut ip: *mut Item = ::core::ptr::null_mut::<Item>();
    let mut d: i32 = 0;
    let mut s: f32 = 0.;
    let mut half: f32 = 0.;
    let mut xr: f32 = 0.;
    let mut yr: f32 = 0.;
    let mut ix: f32 = 0.;
    let mut iy: f32 = 0.;
    let mut x0: f32 = 0.;
    let mut y0: f32 = 0.;
    let mut x1: f32 = 0.;
    let mut y1: f32 = 0.;
    let mut box_0: *mut Vec4 = ::core::ptr::null_mut::<Vec4>();
    if g.is_null() || fp.is_null() {
        return -1.0f32;
    }
    ip = HSD_GObjGetUserData(g) as *mut Item;
    d = geno_item_def(ip);
    if d < 0 as i32
        || Geno_ItemParam(d, GENO_IP_COLLECT as i32)
            != 2 as i32 || (*geno_item_vars(ip)).terminal != 0
    {
        return -1.0f32;
    }
    if (*fp).player_id as i32 >= 6 as i32
        || (*fp).is_sub_fighter() as i32 != 0
        || (*fp).motion_id <= ftCo_MS_RebirthWait as i32
        || (*fp).team as u32 >= 32 as u32
        || Geno_ItemParam(d, GENO_IP_PORT_MASK as i32)
            & (1 as i32) << (*fp).player_id as i32 == 0
        || Geno_ItemParam(d, GENO_IP_TEAM_MASK as i32)
            as u32
            & (1 as u32) << (*fp).team as i32 == 0
        || ftCo_IsCpuControlled(fp) != 0
    {
        return -1.0f32;
    }
    s = geno_item_paramf(d, GENO_IP_SCALE as i32);
    half = (geno_item_paramf(d, GENO_IP_HEIGHT as i32)
        as f32 * s as f32 * 0.5f32) as f32;
    xr = geno_item_paramf(d, GENO_IP_RADIUS as i32) * s;
    yr = if xr > half { xr } else { half };
    ix = (*ip).pos.x;
    iy = (*ip).pos.y + geno_item_lift(d, (*geno_item_vars(ip)).age as f32) + half;
    box_0 = if (*fp).ground_or_air as i32
        == GA_Ground as i32
    {
        &raw mut (*fp).x294_itPickup.gr_light_offset
    } else {
        &raw mut (*fp).x294_itPickup.air_light_offset
    };
    x0 = fmadds((*fp).facing_dir, (*box_0).x, (*fp).cur_pos.x);
    y0 = (*fp).cur_pos.y + (*box_0).y;
    x1 = (*box_0).z;
    y1 = (*box_0).w;
    if x0 - x1 - xr < ix && ix < x0 + x1 + xr && y0 - y1 - yr < iy && iy < y0 + y1 + yr {
        return fmadds(ix - x0, ix - x0, (iy - y0) * (iy - y0));
    }
    return -1.0f32;
}

pub unsafe extern "C" fn Geno_ItemPressCollect(
    mut g: *mut HSD_GObj,
    mut port: i32,
) {
    melee_core::ct!("Geno_ItemPressCollect");
    let mut ip: *mut Item = ::core::ptr::null_mut::<Item>();
    let mut d: i32 = 0;
    if g.is_null() {
        return;
    }
    ip = HSD_GObjGetUserData(g) as *mut Item;
    d = geno_item_def(ip);
    if d < 0 as i32
        || Geno_ItemParam(d, GENO_IP_COLLECT as i32)
            != 2 as i32 || (*geno_item_vars(ip)).terminal != 0
    {
        return;
    }
    geno_item_event(ip, 0 as i32, port);
    Item_8026A8EC(g as *mut Item_GObj);
}

pub unsafe extern "C" fn geno_item_anim(mut g: *mut Item_GObj) -> i32 {
    melee_core::ct!("geno_item_anim");
    let mut ip: *mut Item = HSD_GObjGetUserData(g as *mut HSD_GObj) as *mut Item;
    let mut v: *mut GenoItemVars = geno_item_vars(ip);
    let mut d: i32 = geno_item_def(ip);
    let mut life: i32 = Geno_ItemParam(
        d,
        GENO_IP_LIFETIME as i32,
    );
    let mut cur: *mut HSD_GObj = ::core::ptr::null_mut::<HSD_GObj>();
    if (*v).terminal != 0 {
        return 1;
    }
    (*v).age += 1;
    if life > 0 as i32
        && Geno_ItemParam(d, GENO_IP_BLINK as i32)
            > 0 as i32
        && life - (*v).age <= Geno_ItemParam(d, GENO_IP_BLINK as i32)
        && (*v).blink_started == 0
    {
        (*v).blink_started = 1 as i32;
        OSReport!(
            b"geno item: blink start handle=%d kind=%d age=%d remaining=%d\n\0"
                as *const u8 as *const i8 as *mut i8,
            (*v).handle,
            (*ip).kind as i32,
            (*v).age,
            life - (*v).age,
        );
    }
    if life > 0 as i32 && (*v).age >= life {
        geno_item_event(ip, 1 as i32, -(1 as i32));
        return 1;
    }
    if Geno_ItemParam(d, GENO_IP_COLLECT as i32)
        != 1 as i32 || HSD_GObjPLinkHead.is_null()
    {
        return 0;
    }
    cur = *HSD_GObjPLinkHead.offset(HSD_GOBJ_PLINK_FIGHTER as isize);
    while !cur.is_null() {
        let mut fp: *mut Fighter = HSD_GObjGetUserData(cur) as *mut Fighter;
        let mut dx: f32 = 0.;
        let mut dy: f32 = 0.;
        let mut r: f32 = 0.;
        if !(fp.is_null()
            || (*fp).player_id as i32 >= 6 as i32
            || (*fp).is_sub_fighter() as i32 != 0
            || (*fp).motion_id <= ftCo_MS_RebirthWait as i32
            || (*fp).team as u32 >= 32 as u32
            || Geno_ItemParam(d, GENO_IP_PORT_MASK as i32)
                & (1 as i32) << (*fp).player_id as i32 == 0
            || Geno_ItemParam(d, GENO_IP_TEAM_MASK as i32)
                as u32
                & (1 as u32) << (*fp).team as i32 == 0)
        {
            dx = (*fp).cur_pos.x - (*ip).pos.x;
            dy = (*fp).cur_pos.y - (*ip).pos.y;
            r = geno_item_paramf(d, GENO_IP_RADIUS as i32)
                * geno_item_paramf(d, GENO_IP_SCALE as i32);
            if geno_item_in_capsule(
                dx,
                dy,
                r,
                fmadds(geno_item_paramf(d, GENO_IP_HEIGHT as i32), geno_item_paramf(d, GENO_IP_SCALE as i32), geno_item_lift(d, (*v).age as f32)),
            ) != 0
            {
                geno_item_event(
                    ip,
                    0 as i32,
                    (*fp).player_id as i32,
                );
                return 1;
            }
        }
        cur = (*cur).next as *mut HSD_GObj;
    }
    return 0;
}

pub unsafe extern "C" fn geno_item_phys(mut g: *mut Item_GObj) {
    melee_core::ct!("geno_item_phys");
    let mut ip: *mut Item = HSD_GObjGetUserData(g as *mut HSD_GObj) as *mut Item;
    let mut v: *mut GenoItemVars = geno_item_vars(ip);
    let mut d: i32 = geno_item_def(ip);
    let mut terminal: f32 = geno_item_paramf(
        d,
        GENO_IP_TERMINAL as i32,
    );
    if (*v).floor >= 0 as i32 {
        let mut delta: Vec = unsafe { core::mem::zeroed() };
        if mpGetSpeed((*v).floor, &raw mut (*ip).pos, &raw mut delta) != 0 {
            (*ip).pos.x += delta.x;
            (*ip).pos.y += delta.y;
        } else {
            (*v).floor = -(1 as i32);
        }
    }
    (*v).prev = (*ip).pos;
    (*ip).x40_vel.y -= geno_item_paramf(d, GENO_IP_GRAVITY as i32);
    if terminal > 0 as i32 as f32 && (*ip).x40_vel.y < -terminal {
        (*ip).x40_vel.y = -terminal;
    }
}

pub unsafe extern "C" fn geno_item_floor_response(mut ip: *mut Item, mut y: f32) {
    melee_core::ct!("geno_item_floor_response");
    let mut d: i32 = geno_item_def(ip);
    let mut bounce: f32 = geno_item_paramf(d, GENO_IP_BOUNCE as i32);
    (*ip).pos.y = y;
    (*ip).x40_vel.y = -(*ip).x40_vel.y * bounce;
    (*ip).x40_vel.x
        *= (1.0f32 - geno_item_paramf(d, GENO_IP_FRICTION as i32))
            as f32;
    if Geno_ItemParam(d, GENO_IP_FLOOR_REST as i32) != 0
        && (*ip).x40_vel.y < 0.5f32
    {
        (*ip).x40_vel.y = 0 as i32 as f32;
    }
}

pub unsafe extern "C" fn geno_item_coll(mut g: *mut Item_GObj) -> i32 {
    melee_core::ct!("geno_item_coll");
    let mut ip: *mut Item = HSD_GObjGetUserData(g as *mut HSD_GObj) as *mut Item;
    let mut v: *mut GenoItemVars = geno_item_vars(ip);
    let mut d: i32 = geno_item_def(ip);
    let mut line: i32 = -(1 as i32);
    let mut hit: Vec = unsafe { core::mem::zeroed() };
    let mut bounce: f32 = geno_item_paramf(d, GENO_IP_BOUNCE as i32);
    (*v).floor = -(1 as i32);
    if Geno_ItemParam(d, GENO_IP_STAGE_COLLISION as i32) == 0 {
        return 0;
    }
    if Geno_ItemParam(d, GENO_IP_STAGE_COLLISION as i32) != 0
        && (*ip).x40_vel.y <= 0 as i32 as f32
        && mpCheckFloor(
            (*v).prev.x as f32,
            (*v).prev.y as f32 + 0.01f32,
            (*ip).pos.x as f32,
            (*ip).pos.y as f32 - 0.01f32,
            0 as i32 as f32,
            &raw mut hit,
            &raw mut line,
            ::core::ptr::null_mut::<u32>(),
            ::core::ptr::null_mut::<Vec>(),
            -(1 as i32),
            -(1 as i32),
            -(1 as i32),
            None,
            ::core::ptr::null_mut::<Fighter_GObj>(),
        ) != 0
    {
        geno_item_floor_response(ip, hit.y);
        if (*ip).x40_vel.y == 0 as i32 as f32
            && Geno_ItemParam(d, GENO_IP_FLOOR_REST as i32) != 0
        {
            (*v).floor = line;
        }
    }
    if (*ip).x40_vel.y > 0 as i32 as f32
        && mpCheckCeiling(
            (*v).prev.x as f32,
            (*v).prev.y as f32 - 0.01f32,
            (*ip).pos.x as f32,
            (*ip).pos.y as f32 + 0.01f32,
            &raw mut hit,
            ::core::ptr::null_mut::<i32>(),
            ::core::ptr::null_mut::<u32>(),
            ::core::ptr::null_mut::<Vec>(),
            -(1 as i32),
            -(1 as i32),
        ) != 0
    {
        (*ip).pos.y = (hit.y as f32 - 0.01f32) as f32;
        (*ip).x40_vel.y = -(*ip).x40_vel.y * bounce;
    }
    if (*ip).x40_vel.x != 0 as i32 as f32
        && (mpCheckLeftWall(
            (*v).prev.x as f32,
            (*v).prev.y as f32,
            (*ip).pos.x as f32,
            (*ip).pos.y as f32,
            &raw mut hit,
            ::core::ptr::null_mut::<i32>(),
            ::core::ptr::null_mut::<u32>(),
            ::core::ptr::null_mut::<Vec>(),
            -(1 as i32),
            -(1 as i32),
        ) != 0
            || mpCheckRightWall(
                (*v).prev.x as f32,
                (*v).prev.y as f32,
                (*ip).pos.x as f32,
                (*ip).pos.y as f32,
                &raw mut hit,
                ::core::ptr::null_mut::<i32>(),
                ::core::ptr::null_mut::<u32>(),
                ::core::ptr::null_mut::<Vec>(),
                -(1 as i32),
                -(1 as i32),
            ) != 0)
    {
        (*ip).pos.x = (hit.x as f32
            + (if (*ip).x40_vel.x > 0 as i32 as f32 {
                -0.01f32
            } else {
                0.01f32
            })) as f32;
        (*ip).x40_vel.x = -(*ip).x40_vel.x * bounce;
    }
    return 0;
}

pub unsafe extern "C" fn geno_item_destroyed(mut g: *mut Item_GObj) {
    melee_core::ct!("geno_item_destroyed");
    geno_item_event(
        HSD_GObjGetUserData(g as *mut HSD_GObj) as *mut Item,
        2 as i32,
        -(1 as i32),
    );
}

pub unsafe extern "C" fn Geno_ItemDesc(
    mut kind: i32,
) -> *mut c_void {
    melee_core::ct!("Geno_ItemDesc");
    let mut d: i32 = kind - GENO_ITEM_KIND_BASE;
    let mut a: *mut Article = ::core::ptr::null_mut::<Article>();
    let mut at: *mut ItemAttr = ::core::ptr::null_mut::<ItemAttr>();
    if Geno_ItemActive(kind) == 0 {
        return ::core::ptr::null_mut::<c_void>();
    }
    a = (&raw mut geno_item_desc as *mut Article).offset(d as isize) as *mut Article;
    at = (&raw mut geno_item_attr as *mut ItemAttr).offset(d as isize) as *mut ItemAttr;
    geno_zero(
        at as *mut c_void,
        ::core::mem::size_of::<ItemAttr>() as i32,
    );
    (*at).x60_scale = 1 as i32 as f32;
    (*at).destroy_gfx = -(1 as i32) as enum_t;
    (*at).destroy_sfx = -(1 as i32) as enum_t;
    (*at).x38_grab_range.y = -(1 as i32) as f32;
    (*at).x38_grab_range.x = (*at).x38_grab_range.y;
    geno_zero(
        &raw mut geno_item_joint as *mut c_void,
        ::core::mem::size_of::<HSD_Joint>() as i32,
    );
    geno_item_joint.scale.z = 1 as i32 as f32;
    geno_item_joint.scale.y = geno_item_joint.scale.z;
    geno_item_joint.scale.x = geno_item_joint.scale.y;
    geno_zero(
        (&raw mut geno_item_model as *mut ItemModelDesc).offset(d as isize)
            as *mut ItemModelDesc as *mut c_void,
        ::core::mem::size_of::<ItemModelDesc>() as i32,
    );
    geno_item_model[d as usize].x0_joint = &raw mut geno_item_joint;
    geno_zero(
        a as *mut c_void,
        ::core::mem::size_of::<Article>() as i32,
    );
    (*a).x0_common_attr = at;
    (*a).xC_itemStates = &raw mut geno_item_anims;
    (*a).x10_modelDesc = (&raw mut geno_item_model as *mut ItemModelDesc)
        .offset(d as isize) as *mut ItemModelDesc;
    return a as *mut c_void;
}

pub unsafe extern "C" fn Geno_ItemLogic(
    mut kind: i32,
) -> *mut c_void {
    melee_core::ct!("Geno_ItemLogic");
    return &raw mut geno_item_logic as *mut c_void;
}

pub unsafe extern "C" fn Geno_ItemCap(
    mut kind: i32,
) -> i32 {
    melee_core::ct!("Geno_ItemCap");
    return if Geno_ItemParam(
        kind - GENO_ITEM_KIND_BASE,
        GENO_IP_CAP as i32,
    ) != 0
    {
        0 as i32
    } else {
        8 as i32
    };
}

pub unsafe extern "C" fn geno_item_index(mut index: i32) -> *mut Item {
    melee_core::ct!("geno_item_index");
    let mut cur: *mut HSD_GObj = ::core::ptr::null_mut::<HSD_GObj>();
    if index < 0 as i32 || HSD_GObjPLinkHead.is_null() {
        return ::core::ptr::null_mut::<Item>();
    }
    cur = *HSD_GObjPLinkHead.offset(HSD_GOBJ_PLINK_ITEM as isize);
    while !cur.is_null() {
        let mut ip: *mut Item = HSD_GObjGetUserData(cur) as *mut Item;
        if !ip.is_null() && geno_item_def(ip) >= 0 as i32
            && (*geno_item_vars(ip)).terminal == 0
            && {
                let fresh15 = index;
                index = index - 1;
                fresh15 == 0 as i32
            }
        {
            return ip;
        }
        cur = (*cur).next as *mut HSD_GObj;
    }
    return ::core::ptr::null_mut::<Item>();
}

pub unsafe extern "C" fn Geno_ItemIndexForId(
    mut serial: i32,
) -> i32 {
    melee_core::ct!("Geno_ItemIndexForId");
    let mut i: i32 = 0 as i32;
    let mut cur: *mut HSD_GObj = ::core::ptr::null_mut::<HSD_GObj>();
    if HSD_GObjPLinkHead.is_null() {
        return -(1 as i32);
    }
    cur = *HSD_GObjPLinkHead.offset(HSD_GOBJ_PLINK_ITEM as isize);
    while !cur.is_null() {
        let mut ip: *mut Item = HSD_GObjGetUserData(cur) as *mut Item;
        if !(ip.is_null() || geno_item_def(ip) < 0 as i32
            || (*geno_item_vars(ip)).terminal != 0)
        {
            if (*ip).x1C == serial {
                return i;
            }
            i += 1;
        }
        cur = (*cur).next as *mut HSD_GObj;
    }
    return -(1 as i32);
}

pub unsafe extern "C" fn Geno_ItemI(
    mut index: i32,
    mut field: i32,
) -> i32 {
    melee_core::ct!("Geno_ItemI");
    let mut ip: *mut Item = geno_item_index(index);
    let mut v: *mut GenoItemVars = ::core::ptr::null_mut::<GenoItemVars>();
    let mut d: i32 = 0;
    let mut life: i32 = 0;
    let mut blink: i32 = 0;
    if ip.is_null() {
        return -(1 as i32);
    }
    v = geno_item_vars(ip);
    d = geno_item_def(ip);
    match field {
        0 => return (*v).handle,
        1 => return d,
        2 => return (*v).owner,
        3 => return (*v).colour,
        4 => return (*v).amount,
        5 => return (*v).age,
        6 => return (*ip).kind as i32,
        7 => {
            life = Geno_ItemParam(d, GENO_IP_LIFETIME as i32);
            blink = Geno_ItemParam(d, GENO_IP_BLINK as i32);
            return !(life > 0 as i32 && blink > 0 as i32
                && life - (*v).age <= blink && (*v).age & 4 as i32 != 0)
                as i32;
        }
        _ => {}
    }
    return -(1 as i32);
}

pub unsafe extern "C" fn Geno_ItemF(
    mut index: i32,
    mut field: i32,
) -> i32 {
    melee_core::ct!("Geno_ItemF");
    let mut ip: *mut Item = geno_item_index(index);
    if ip.is_null() {
        return 0 as i32;
    }
    match field {
        0 => return geno_item_bits((*ip).pos.x),
        1 => return geno_item_bits((*ip).pos.y),
        4 => return geno_item_bits((*ip).pos.z),
        2 => return geno_item_bits((*ip).x40_vel.x),
        3 => return geno_item_bits((*ip).x40_vel.y),
        _ => {}
    }
    return 0 as i32;
}

pub unsafe extern "C" fn Geno_ItemVisual(
    mut index: i32,
    mut field: i32,
) -> i32 {
    melee_core::ct!("Geno_ItemVisual");
    let mut ip: *mut Item = geno_item_index(index);
    let mut d: i32 = 0;
    let mut age: f32 = 0.;
    if ip.is_null() {
        return 0 as i32;
    }
    d = geno_item_def(ip);
    age = (*geno_item_vars(ip)).age as f32;
    match field {
        0 => return geno_item_bits((*ip).pos.y + geno_item_lift(d, age)),
        1 => {
            return geno_item_bits(
                age * geno_item_paramf(d, GENO_IP_SPIN as i32),
            );
        }
        2 => return Geno_ItemParam(d, GENO_IP_SCALE as i32),
        3 => return Geno_ItemParam(d, GENO_IP_TILT as i32),
        _ => {}
    }
    return 0 as i32;
}

pub unsafe extern "C" fn Geno_ItemSpawn(
    mut d: i32,
    mut xb: i32,
    mut yb: i32,
    mut vxb: i32,
    mut vyb: i32,
    mut handle: i32,
    mut owner: i32,
    mut colour: i32,
    mut amount: i32,
) -> i32 {
    melee_core::ct!("Geno_ItemSpawn");
    let mut s: SpawnItem = unsafe { core::mem::zeroed() };
    let mut g: *mut Item_GObj = ::core::ptr::null_mut::<Item_GObj>();
    let mut ip: *mut Item = ::core::ptr::null_mut::<Item>();
    let mut v: *mut GenoItemVars = ::core::ptr::null_mut::<GenoItemVars>();
    let mut i: i32 = 0;
    if d < 0 as i32 || d >= GENO_ITEM_MAX_DEFS
        || Geno_ItemParam(d, GENO_IP_DEFINED as i32) == 0
    {
        return GENO_ITEM_ERR_UNSUPPORTED as i32;
    }
    if Mex_ItemRangeAvailable(GENO_ITEM_KIND_BASE, GENO_ITEM_MAX_DEFS) == 0 {
        return GENO_ITEM_ERR_CONFLICT as i32;
    }
    if Geno_ItemDesc(GENO_ITEM_KIND_BASE + d).is_null() {
        return GENO_ITEM_ERR_UNINITIALIZED as i32;
    }
    i = 0 as i32;
    while !geno_item_index(i).is_null() {
        if i >= 127 as i32 {
            return GENO_ITEM_ERR_POOL as i32;
        }
        i += 1;
    }
    geno_zero(
        &raw mut s as *mut c_void,
        ::core::mem::size_of::<SpawnItem>() as i32,
    );
    s.kind = (GENO_ITEM_KIND_BASE + d) as ItemKind;
    s.pos.x = geno_item_float(xb);
    s.pos.y = geno_item_float(yb);
    s.prev_pos = s.pos;
    Item_InitSpawnCommonFields(
        &raw mut s,
        ::core::ptr::null_mut::<HSD_GObj>(),
        1 as i32 as f32,
        1,
    );
    s.vel.x = geno_item_float(vxb);
    s.vel.y = geno_item_float(vyb);
    if Item_ScriptSpawnAtCap(&raw mut s) != 0 {
        return GENO_ITEM_ERR_CAP as i32;
    }
    g = Item_80268B18(&raw mut s);
    if g.is_null() {
        return GENO_ITEM_ERR_POOL as i32;
    }
    ip = HSD_GObjGetUserData(g as *mut HSD_GObj) as *mut Item;
    v = geno_item_vars(ip);
    geno_zero(
        v as *mut c_void,
        ::core::mem::size_of::<GenoItemVars>() as i32,
    );
    (*v).handle = handle;
    (*v).owner = owner;
    (*v).colour = colour;
    (*v).amount = amount;
    (*v).prev = (*ip).pos;
    (*v).floor = -(1 as i32);
    Item_80268E5C(g as *mut HSD_GObj, 0 as enum_t, ITEM_ANIM_UPDATE);
    (*ip).x40_vel = s.vel;
    (*ip).xDC8_word.flags.set_x15(0 as u32 as u32);
    return handle;
}

pub unsafe extern "C" fn Geno_ItemDespawn(
    mut handle: i32,
    mut owner: i32,
) -> i32 {
    melee_core::ct!("Geno_ItemDespawn");
    let mut i: i32 = 0;
    let mut ip: *mut Item = ::core::ptr::null_mut::<Item>();
    let mut cur: *mut HSD_GObj = ::core::ptr::null_mut::<HSD_GObj>();
    i = 0 as i32;
    while i < 128 as i32 {
        let mut f: *mut GenoFamilyItem = (&raw mut geno_family_items
            as *mut GenoFamilyItem)
            .offset(i as isize) as *mut GenoFamilyItem;
        if (*f).g.is_null() || (*f).handle != handle
            || owner >= 0 as i32 && (*f).owner != owner
        {
            i += 1;
        } else {
            cur = if !HSD_GObjPLinkHead.is_null() {
                *HSD_GObjPLinkHead.offset(HSD_GOBJ_PLINK_ITEM as isize)
            } else {
                ::core::ptr::null_mut::<HSD_GObj>()
            };
            while !cur.is_null() {
                if cur == (*f).g
                    && (*(HSD_GObjGetUserData(cur) as *mut Item)).x1C == (*f).serial
                {
                    (*f).terminal = 1 as i32;
                    Item_8026A8EC(cur as *mut Item_GObj);
                    (*f).g = ::core::ptr::null_mut::<HSD_GObj>();
                    return 1 as i32;
                }
                cur = (*cur).next as *mut HSD_GObj;
            }
            (*f).g = ::core::ptr::null_mut::<HSD_GObj>();
            return 0 as i32;
        }
    }
    i = 0 as i32;
    loop {
        ip = geno_item_index(i);
        if ip.is_null() {
            break;
        }
        let mut v: *mut GenoItemVars = geno_item_vars(ip);
        if (*v).handle == handle
            && (owner < 0 as i32 || (*v).owner == owner)
        {
            (*v).terminal = 1 as i32;
            Item_8026A8EC((*ip).entity as *mut Item_GObj);
            return 1 as i32;
        }
        i += 1;
    }
    return 0 as i32;
}

pub unsafe extern "C" fn Geno_ItemReleaseOwner(mut owner: i32) {
    melee_core::ct!("Geno_ItemReleaseOwner");
    let mut i: i32 = 0 as i32;
    let mut ip: *mut Item = ::core::ptr::null_mut::<Item>();
    loop {
        ip = geno_item_index(i);
        if ip.is_null() {
            break;
        }
        if owner == 0 as i32 || (*geno_item_vars(ip)).owner == owner {
            Geno_ItemDespawn(
                (*geno_item_vars(ip)).handle,
                if owner == 0 as i32 {
                    -(1 as i32)
                } else {
                    owner
                },
            );
        } else {
            i += 1;
        }
    }
    i = 0 as i32;
    while i < 128 as i32 {
        if !geno_family_items[i as usize].g.is_null()
            && (owner == 0 as i32
                && geno_family_items[i as usize].owner >= 0 as i32
                || geno_family_items[i as usize].owner == owner)
        {
            Geno_ItemDespawn(
                geno_family_items[i as usize].handle,
                if owner == 0 as i32 {
                    -(1 as i32)
                } else {
                    owner
                },
            );
        }
        i += 1;
    }
}

pub unsafe extern "C" fn Geno_ItemsSceneReset() {
    melee_core::ct!("Geno_ItemsSceneReset");
    geno_zero(
        &raw mut geno_family_items as *mut GenoFamilyItem as *mut c_void,
        ::core::mem::size_of::<[GenoFamilyItem; 128]>() as i32,
    );
}

pub unsafe extern "C" fn Geno_ItemSpawnKind(
    mut kind: i32,
    mut xb: i32,
    mut yb: i32,
    mut vxb: i32,
    mut vyb: i32,
    mut handle: i32,
    mut owner: i32,
    mut colour: i32,
    mut amount: i32,
) -> i32 {
    melee_core::ct!("Geno_ItemSpawnKind");
    let mut s: SpawnItem = unsafe { core::mem::zeroed() };
    let mut g: *mut HSD_GObj = ::core::ptr::null_mut::<HSD_GObj>();
    let mut i: i32 = 0;
    let mut cur: *mut HSD_GObj = ::core::ptr::null_mut::<HSD_GObj>();
    if kind < 0 as i32 || kind >= 5000 as i32 {
        return GENO_ITEM_ERR_UNSUPPORTED as i32;
    }
    if kind >= GENO_ART_KIND_BASE && kind < GENO_ART_KIND_END
        || Geno_ItemActive(kind) != 0
    {
        return GENO_ITEM_ERR_CONFLICT as i32;
    }
    if kind > It_Kind_M_Ball as i32 && kind < 237 as i32 {
        return GENO_ITEM_ERR_UNSAFE as i32;
    }
    if kind >= 237 as i32 && Mex_ItemReady(kind) == 0 {
        return GENO_ITEM_ERR_UNINITIALIZED as i32;
    }
    if kind <= It_Kind_M_Ball as i32
        && (it_804D6D24.is_null() || (*it_804D6D24.offset(kind as isize)).is_null()
            || (**it_804D6D24.offset(kind as isize)).x0_common_attr.is_null()
            || (**it_804D6D24.offset(kind as isize)).x10_modelDesc.is_null()
            || it_803F14C4[kind as usize].states.is_null()
            || it_803F14C4[kind as usize].spawned.is_none())
    {
        return GENO_ITEM_ERR_UNINITIALIZED as i32;
    }
    i = 0 as i32;
    while i < 128 as i32 {
        let mut f: *mut GenoFamilyItem = (&raw mut geno_family_items
            as *mut GenoFamilyItem)
            .offset(i as isize) as *mut GenoFamilyItem;
        let mut live: i32 = 0 as i32;
        if !(*f).g.is_null() {
            cur = if !HSD_GObjPLinkHead.is_null() {
                *HSD_GObjPLinkHead.offset(HSD_GOBJ_PLINK_ITEM as isize)
            } else {
                ::core::ptr::null_mut::<HSD_GObj>()
            };
            while !cur.is_null() {
                if cur == (*f).g
                    && (*(HSD_GObjGetUserData(cur) as *mut Item)).x1C == (*f).serial
                {
                    live = 1 as i32;
                    break;
                } else {
                    cur = (*cur).next as *mut HSD_GObj;
                }
            }
        }
        if live == 0 {
            (*f).g = ::core::ptr::null_mut::<HSD_GObj>();
            break;
        } else {
            i += 1;
        }
    }
    if i == 128 as i32 {
        return GENO_ITEM_ERR_POOL as i32;
    }
    geno_zero(
        &raw mut s as *mut c_void,
        ::core::mem::size_of::<SpawnItem>() as i32,
    );
    s.kind = kind as ItemKind;
    s.pos.x = geno_item_float(xb);
    s.pos.y = geno_item_float(yb);
    s.prev_pos = s.pos;
    Item_InitSpawnCommonFields(
        &raw mut s,
        ::core::ptr::null_mut::<HSD_GObj>(),
        1 as i32 as f32,
        1,
    );
    s.vel.x = geno_item_float(vxb);
    s.vel.y = geno_item_float(vyb);
    if Item_ScriptSpawnAtCap(&raw mut s) != 0 {
        return GENO_ITEM_ERR_CAP as i32;
    }
    g = Item_80268B18(&raw mut s) as *mut HSD_GObj;
    if g.is_null() {
        return GENO_ITEM_ERR_POOL as i32;
    }
    geno_family_items[i as usize].g = g;
    geno_family_items[i as usize].serial = (*(HSD_GObjGetUserData(g) as *mut Item)).x1C
        as i32;
    geno_family_items[i as usize].handle = handle;
    geno_family_items[i as usize].owner = owner;
    (*(HSD_GObjGetUserData(g) as *mut Item)).x40_vel = s.vel;
    geno_family_items[i as usize].kind = kind;
    geno_family_items[i as usize].terminal = 0 as i32;
    geno_family_items[i as usize].colour = colour;
    geno_family_items[i as usize].amount = amount;
    return handle;
}

pub unsafe extern "C" fn Geno_ItemMeta(
    mut serial: i32,
    mut field: i32,
) -> i32 {
    melee_core::ct!("Geno_ItemMeta");
    let mut i: i32 = 0;
    let mut ip: *mut Item = ::core::ptr::null_mut::<Item>();
    i = 0 as i32;
    loop {
        ip = geno_item_index(i);
        if ip.is_null() {
            break;
        }
        if (*ip).x1C == serial {
            return Geno_ItemI(i, field);
        }
        i += 1;
    }
    i = 0 as i32;
    while i < 128 as i32 {
        if !geno_family_items[i as usize].g.is_null()
            && geno_family_items[i as usize].serial == serial
        {
            let mut f: *mut GenoFamilyItem = (&raw mut geno_family_items
                as *mut GenoFamilyItem)
                .offset(i as isize) as *mut GenoFamilyItem;
            match field {
                0 => return (*f).handle,
                2 => return (*f).owner,
                6 => return (*f).kind,
                1 => return -((*f).kind + 1 as i32),
                3 => return (*f).colour,
                4 => return (*f).amount,
                _ => {}
            }
        }
        i += 1;
    }
    return -(1 as i32);
}

pub unsafe extern "C" fn Geno_ItemHandleForId(
    mut serial: i32,
) -> i32 {
    melee_core::ct!("Geno_ItemHandleForId");
    return Geno_ItemMeta(serial, GENO_II_HANDLE as i32);
}

pub unsafe extern "C" fn Geno_ItemFamilyEvent(
    mut g: *mut HSD_GObj,
    mut what: i32,
    mut port: i32,
) {
    melee_core::ct!("Geno_ItemFamilyEvent");
    let mut ip: *mut Item = HSD_GObjGetUserData(g) as *mut Item;
    let mut i: i32 = 0;
    let mut slot: i32 = -(1 as i32);
    let mut cur: *mut HSD_GObj = ::core::ptr::null_mut::<HSD_GObj>();
    if geno_item_def(ip) >= 0 as i32 {
        return;
    }
    i = 0 as i32;
    while i < 128 as i32 {
        let mut f: *mut GenoFamilyItem = (&raw mut geno_family_items
            as *mut GenoFamilyItem)
            .offset(i as isize) as *mut GenoFamilyItem;
        if (*f).g != g || (*f).serial != (*ip).x1C || (*f).terminal != 0 {
            i += 1;
        } else {
            (*f).terminal = 1 as i32;
            ItemEvent(
                what,
                (*f).handle,
                -((*ip).kind as i32 + 1 as i32),
                port,
                geno_item_bits((*ip).pos.x),
                geno_item_bits((*ip).pos.y),
                (*f).colour,
                (*f).amount,
            );
            return;
        }
    }
    i = 0 as i32;
    while i < 128 as i32 {
        let mut f_0: *mut GenoFamilyItem = (&raw mut geno_family_items
            as *mut GenoFamilyItem)
            .offset(i as isize) as *mut GenoFamilyItem;
        let mut live: i32 = 0 as i32;
        if (*f_0).g == g && (*f_0).serial == (*ip).x1C {
            return;
        }
        if !(*f_0).g.is_null() {
            cur = if !HSD_GObjPLinkHead.is_null() {
                *HSD_GObjPLinkHead.offset(HSD_GOBJ_PLINK_ITEM as isize)
            } else {
                ::core::ptr::null_mut::<HSD_GObj>()
            };
            while !cur.is_null() {
                if cur == (*f_0).g
                    && (*(HSD_GObjGetUserData(cur) as *mut Item)).x1C == (*f_0).serial
                {
                    live = 1 as i32;
                    break;
                } else {
                    cur = (*cur).next as *mut HSD_GObj;
                }
            }
        }
        if live == 0 && slot < 0 as i32 {
            slot = i;
        }
        i += 1;
    }
    if what != 2 as i32 {
        if slot < 0 as i32 {
            return;
        }
        geno_zero(
            (&raw mut geno_family_items as *mut GenoFamilyItem).offset(slot as isize)
                as *mut GenoFamilyItem as *mut c_void,
            ::core::mem::size_of::<GenoFamilyItem>() as i32,
        );
        geno_family_items[slot as usize].g = g;
        geno_family_items[slot as usize].serial = (*ip).x1C as i32;
        geno_family_items[slot as usize].handle = (*ip).x1C as i32;
        geno_family_items[slot as usize].owner = -(1 as i32);
        geno_family_items[slot as usize].kind = (*ip).kind as i32;
        geno_family_items[slot as usize].terminal = 1 as i32;
    }
    ItemEvent(
        what,
        (*ip).x1C as i32,
        -((*ip).kind as i32 + 1 as i32),
        port,
        geno_item_bits((*ip).pos.x),
        geno_item_bits((*ip).pos.y),
        0 as i32,
        0 as i32,
    );
}

pub unsafe extern "C" fn Geno_ItemDraw(
    mut g: *mut HSD_GObj,
    mut code: i32,
) {
    melee_core::ct!("Geno_ItemDraw");
    let mut i: i32 = 0;
    let mut ip: *mut Item = ::core::ptr::null_mut::<Item>();
    let mut c: *mut HSD_CObj = ::core::ptr::null_mut::<HSD_CObj>();
    if code != 0 as i32 && code != 2 as i32 {
        return;
    }
    c = HSD_CObjGetCurrent();
    if c.is_null() {
        return;
    }
    i = 0 as i32;
    loop {
        ip = geno_item_index(i);
        if ip.is_null() {
            break;
        }
        if (*ip).entity == g {
            HSD_StateInvalidate(-(1 as i32));
            ItemDraw(i, HSD_CObjGetViewingMtxPtr(c) as i32, code);
            HSD_StateInvalidate(-(1 as i32));
            return;
        }
        i += 1;
    };
}

pub unsafe extern "C" fn Geno_ItemFixture(
    mut def: i32,
) -> i32 {
    melee_core::ct!("Geno_ItemFixture");
    let mut ip: Item = unsafe { core::mem::zeroed() };
    let mut saved: Item = unsafe { core::mem::zeroed() };
    let mut g: HSD_GObj = unsafe { core::mem::zeroed() };
    let mut fighter: HSD_GObj = unsafe { core::mem::zeroed() };
    let mut fp: Fighter = unsafe { core::mem::zeroed() };
    let mut old: *mut *mut HSD_GObj = ::core::ptr::null_mut::<*mut HSD_GObj>();
    let mut heads: [*mut HSD_GObj; 32] = [::core::ptr::null_mut::<HSD_GObj>(); 32];
    let mut failed: i32 = 0 as i32;
    let mut handle: i32 = 731 as i32;
    let mut life: i32 = 0;
    if def < 0 as i32 || def >= GENO_ITEM_MAX_DEFS
        || Geno_ItemParam(def, GENO_IP_DEFINED as i32) == 0
    {
        return -(1 as i32);
    }
    geno_zero(
        &raw mut ip as *mut c_void,
        ::core::mem::size_of::<Item>() as i32,
    );
    geno_zero(
        &raw mut g as *mut c_void,
        ::core::mem::size_of::<HSD_GObj>() as i32,
    );
    geno_zero(
        &raw mut fighter as *mut c_void,
        ::core::mem::size_of::<HSD_GObj>() as i32,
    );
    geno_zero(
        &raw mut fp as *mut c_void,
        ::core::mem::size_of::<Fighter>() as i32,
    );
    geno_zero(
        &raw mut heads as *mut *mut HSD_GObj as *mut c_void,
        ::core::mem::size_of::<[*mut HSD_GObj; 32]>() as i32,
    );
    ip.kind = (GENO_ITEM_KIND_BASE + def) as ItemKind;
    ip.entity = &raw mut g;
    g.user_data = &raw mut ip as *mut c_void;
    (*geno_item_vars(&raw mut ip)).handle = handle;
    (*geno_item_vars(&raw mut ip)).owner = 19 as i32;
    (*geno_item_vars(&raw mut ip)).colour = 3 as i32;
    (*geno_item_vars(&raw mut ip)).amount = 7 as i32;
    (*geno_item_vars(&raw mut ip)).floor = -(1 as i32);
    ip.pos.x = 9 as i32 as f32;
    ip.pos.y = 22 as i32 as f32;
    ip.x40_vel.y = -(4 as i32) as f32;
    saved = ip;
    geno_item_phys(&raw mut g);
    ip.pos.y = -(100 as i32) as f32;
    geno_item_floor_response(&raw mut ip, 5 as i32 as f32);
    if ip.pos.y != 5 as i32 as f32
        || ip.x40_vel.y != 0 as i32 as f32
    {
        failed += 1;
    }
    ip = saved;
    if ip.pos.y != 22 as i32 as f32
        || ip.x40_vel.y != -(4 as i32) as f32
        || (*geno_item_vars(&raw mut ip)).handle != handle
        || (*geno_item_vars(&raw mut ip)).colour != 3 as i32
        || (*geno_item_vars(&raw mut ip)).amount != 7 as i32
    {
        failed += 1;
    }
    if geno_item_in_capsule(
        0 as i32 as f32,
        0 as i32 as f32,
        5 as i32 as f32,
        17.375f32,
    ) == 0
        || geno_item_in_capsule(
            0 as i32 as f32,
            17.375f32,
            5 as i32 as f32,
            17.375f32,
        ) == 0
        || geno_item_in_capsule(
            3 as i32 as f32,
            21.375f32,
            5 as i32 as f32,
            17.375f32,
        ) == 0
        || geno_item_in_capsule(
            5.01f32,
            6 as i32 as f32,
            5 as i32 as f32,
            17.375f32,
        ) != 0
        || geno_item_in_capsule(
            0 as i32 as f32,
            22.385f32,
            5 as i32 as f32,
            17.375f32,
        ) != 0
        || geno_item_in_capsule(
            3 as i32 as f32,
            4 as i32 as f32,
            5 as i32 as f32,
            0 as i32 as f32,
        ) == 0
        || geno_item_in_capsule(
            3 as i32 as f32,
            4.01f32,
            5 as i32 as f32,
            0 as i32 as f32,
        ) != 0
    {
        failed += 1;
    }
    old = HSD_GObjPLinkHead;
    HSD_GObjPLinkHead = &raw mut heads as *mut *mut HSD_GObj;
    heads[HSD_GOBJ_PLINK_ITEM as usize] = &raw mut g;
    if Geno_ItemF(0 as i32, GENO_IF_Y as i32)
        != geno_item_bits(22 as i32 as f32)
        || Geno_ItemVisual(0 as i32, GENO_IV_Y as i32)
            != geno_item_bits(
                22 as i32 as f32
                    + geno_item_lift(def, 0 as i32 as f32),
            )
    {
        failed += 1;
    }
    (*geno_item_vars(&raw mut ip)).age = 15 as i32;
    if Geno_ItemVisual(0 as i32, GENO_IV_Y as i32)
        != geno_item_bits(
            ip.pos.y + geno_item_lift(def, 15 as i32 as f32),
        )
    {
        failed += 1;
    }
    if Geno_ItemVisual(0 as i32, GENO_IV_ROT_Y as i32)
        != geno_item_bits(
            15 as i32 as f32
                * geno_item_paramf(def, GENO_IP_SPIN as i32),
        )
        || Geno_ItemVisual(0 as i32, GENO_IV_TILT as i32)
            != Geno_ItemParam(def, GENO_IP_TILT as i32)
        || Geno_ItemVisual(0 as i32, GENO_IV_SCALE as i32)
            != Geno_ItemParam(def, GENO_IP_SCALE as i32)
    {
        failed += 1;
    }
    (*geno_item_vars(&raw mut ip)).age = 0 as i32;
    fighter.user_data = &raw mut fp as *mut c_void;
    fp.player_id = 0 as u8;
    fp.team = 0 as u8;
    fp.motion_id = ftCo_MS_Wait as i32 as FtMotionId;
    fp.cur_pos = ip.pos;
    heads[HSD_GOBJ_PLINK_FIGHTER as usize] = &raw mut fighter;
    let mut radius: f32 = geno_item_paramf(def, GENO_IP_RADIUS as i32)
        * geno_item_paramf(def, GENO_IP_SCALE as i32);
    fp.cur_pos.y = fmadds(geno_item_paramf(def, GENO_IP_HEIGHT as i32), geno_item_paramf(def, GENO_IP_SCALE as i32), ip.pos.y + geno_item_lift(def, 1 as i32 as f32));
    fp.cur_pos.x = (ip.pos.x as f32 + radius as f32
        + 0.1f32) as f32;
    if geno_item_anim(&raw mut g) != 0 || (*geno_item_vars(&raw mut ip)).terminal != 0 {
        failed += 1;
    }
    (*geno_item_vars(&raw mut ip)).age = 0 as i32;
    fp.cur_pos.x = fmadds(radius, 0.99f32, ip.pos.x);
    if geno_item_anim(&raw mut g) == 0 || (*geno_item_vars(&raw mut ip)).terminal == 0 {
        failed += 1;
    }
    if geno_item_anim(&raw mut g) == 0 {
        failed += 1;
    }
    geno_item_destroyed(&raw mut g);
    ip = saved;
    heads[HSD_GOBJ_PLINK_FIGHTER as usize] = ::core::ptr::null_mut::<HSD_GObj>();
    life = Geno_ItemParam(def, GENO_IP_LIFETIME as i32);
    (*geno_item_vars(&raw mut ip)).age = life - 1 as i32;
    if life <= 0 as i32 || geno_item_anim(&raw mut g) == 0
        || (*geno_item_vars(&raw mut ip)).terminal == 0
    {
        failed += 1;
    }
    if Geno_ItemSpawnKind(
        -(1 as i32),
        0 as i32,
        0 as i32,
        0 as i32,
        0 as i32,
        1 as i32,
        0 as i32,
        0 as i32,
        0 as i32,
    ) != GENO_ITEM_ERR_UNSUPPORTED as i32
        || Geno_ItemSpawnKind(
            5000 as i32,
            0 as i32,
            0 as i32,
            0 as i32,
            0 as i32,
            1 as i32,
            0 as i32,
            0 as i32,
            0 as i32,
        ) != GENO_ITEM_ERR_UNSUPPORTED as i32
        || Geno_ItemSpawnKind(
            35 as i32,
            0 as i32,
            0 as i32,
            0 as i32,
            0 as i32,
            1 as i32,
            0 as i32,
            0 as i32,
            0 as i32,
        ) != GENO_ITEM_ERR_UNSAFE as i32
        || Geno_ItemSpawnKind(
            200 as i32,
            0 as i32,
            0 as i32,
            0 as i32,
            0 as i32,
            1 as i32,
            0 as i32,
            0 as i32,
            0 as i32,
        ) != GENO_ITEM_ERR_UNSAFE as i32
        || Geno_ItemSpawnKind(
            GENO_ART_KIND_BASE,
            0 as i32,
            0 as i32,
            0 as i32,
            0 as i32,
            1 as i32,
            0 as i32,
            0 as i32,
            0 as i32,
        ) != GENO_ITEM_ERR_CONFLICT as i32
        || Geno_ItemSpawn(
            -(1 as i32),
            0 as i32,
            0 as i32,
            0 as i32,
            0 as i32,
            1 as i32,
            0 as i32,
            0 as i32,
            0 as i32,
        ) != GENO_ITEM_ERR_UNSUPPORTED as i32
    {
        failed += 1;
    }
    let mut articles: *mut *mut Article = it_804D6D24;
    it_804D6D24 = ::core::ptr::null_mut::<*mut Article>();
    if Geno_ItemSpawnKind(
        34 as i32,
        0 as i32,
        0 as i32,
        0 as i32,
        0 as i32,
        1 as i32,
        0 as i32,
        0 as i32,
        0 as i32,
    ) != GENO_ITEM_ERR_UNINITIALIZED as i32
    {
        failed += 1;
    }
    it_804D6D24 = articles;
    failed += Item_ScriptSpawnCapTest();
    if life > 0 as i32
        && Geno_ItemParam(def, GENO_IP_BLINK as i32)
            > 0 as i32
    {
        let mut age: i32 = 0;
        ip = saved;
        (*geno_item_vars(&raw mut ip)).age = life
            - Geno_ItemParam(def, GENO_IP_BLINK as i32)
            - 1 as i32;
        if (*geno_item_vars(&raw mut ip)).age >= 0 as i32 {
            geno_item_anim(&raw mut g);
            if (*geno_item_vars(&raw mut ip)).blink_started == 0 {
                failed += 1;
            }
        }
        age = life - Geno_ItemParam(def, GENO_IP_BLINK as i32);
        while age < life
            && age
                < life - Geno_ItemParam(def, GENO_IP_BLINK as i32)
                    + 8 as i32
        {
            (*geno_item_vars(&raw mut ip)).age = age;
            if Geno_ItemI(0 as i32, GENO_II_VISIBLE as i32)
                != (age & 4 as i32 == 0) as i32
            {
                failed += 1;
            }
            age += 1;
        }
    }
    HSD_GObjPLinkHead = old;
    return failed;
}

pub unsafe extern "C" fn Geno_ItemSnapshotFixture(
    mut def: i32,
) -> i32 {
    melee_core::ct!("Geno_ItemSnapshotFixture");
    let mut old: *mut *mut HSD_GObj = HSD_GObjPLinkHead;
    let mut result: i32 = 0 as i32;
    let mut ip: *mut Item = &raw mut geno_item_snap_fixture;
    let mut v: *mut GenoItemVars = ::core::ptr::null_mut::<GenoItemVars>();
    if def < 0 as i32 || def >= GENO_ITEM_MAX_DEFS
        || Geno_ItemParam(def, GENO_IP_DEFINED as i32) == 0
    {
        return -(1 as i32);
    }
    geno_zero(
        ip as *mut c_void,
        ::core::mem::size_of::<Item>() as i32,
    );
    geno_zero(
        &raw mut geno_item_snap_gobj as *mut c_void,
        ::core::mem::size_of::<HSD_GObj>() as i32,
    );
    geno_zero(
        &raw mut geno_item_snap_heads as *mut *mut HSD_GObj as *mut c_void,
        ::core::mem::size_of::<[*mut HSD_GObj; 32]>() as i32,
    );
    (*ip).kind = (GENO_ITEM_KIND_BASE + def) as ItemKind;
    (*ip).x1C = 731 as i32 as i32;
    (*ip).entity = &raw mut geno_item_snap_gobj;
    geno_item_snap_gobj.user_data = ip as *mut c_void;
    (*ip).pos.x = 9 as i32 as f32;
    (*ip).pos.y = 22 as i32 as f32;
    (*ip).x40_vel.y = -(4 as i32) as f32;
    v = geno_item_vars(ip);
    (*v).handle = 0x40000731 as i32;
    (*v).owner = 19 as i32;
    (*v).colour = 3 as i32;
    (*v).amount = 7 as i32;
    (*v).floor = -(1 as i32);
    geno_item_snap_heads[HSD_GOBJ_PLINK_ITEM as usize] = &raw mut geno_item_snap_gobj;
    HSD_GObjPLinkHead = &raw mut geno_item_snap_heads as *mut *mut HSD_GObj;
    if snap_open(1 as i32) != 0 as i32 {
        HSD_GObjPLinkHead = old;
        return -(2 as i32);
    }
    snap_save(0 as i32);
    (*ip).pos.y = 55 as i32 as f32;
    (*ip).x40_vel.y = 100 as i32 as f32;
    (*v).colour = 99 as i32;
    (*v).amount = 88 as i32;
    geno_item_snap_heads[HSD_GOBJ_PLINK_ITEM as usize] = ::core::ptr::null_mut::<
        HSD_GObj,
    >();
    if snap_load(0 as i32) != 0 as i32 {
        result = 1 as i32;
    } else if Geno_ItemF(0 as i32, GENO_IF_Y as i32)
        != geno_item_bits(22 as i32 as f32)
        || Geno_ItemF(0 as i32, GENO_IF_VY as i32)
            != geno_item_bits(-(4 as i32) as f32)
        || Geno_ItemI(0 as i32, GENO_II_HANDLE as i32)
            != 0x40000731 as i32
        || Geno_ItemI(0 as i32, GENO_II_COLOUR as i32)
            != 3 as i32
        || Geno_ItemI(0 as i32, GENO_II_AMOUNT as i32)
            != 7 as i32
        || Geno_ItemVisual(0 as i32, GENO_IV_Y as i32)
            != geno_item_bits(
                22 as i32 as f32
                    + geno_item_lift(def, 0 as i32 as f32),
            )
    {
        result = 2 as i32;
    }
    HSD_GObjPLinkHead = old;
    return result;
}

pub unsafe extern "C" fn Geno_ItemInterleaveFixture(
    mut def: i32,
) -> i32 {
    melee_core::ct!("Geno_ItemInterleaveFixture");
    let mut saved: [*mut Article; 30] = [::core::ptr::null_mut::<Article>(); 30];
    let mut stale: Article = unsafe { core::mem::zeroed() };
    let mut fresh: Article = unsafe { core::mem::zeroed() };
    let mut model: ItemModelDesc = unsafe { core::mem::zeroed() };
    let mut attr: ItemAttr = unsafe { core::mem::zeroed() };
    let mut joint: HSD_Joint = unsafe { core::mem::zeroed() };
    let mut probe: Item = unsafe { core::mem::zeroed() };
    let mut probe_g: HSD_GObj = unsafe { core::mem::zeroed() };
    let mut i: i32 = 0;
    let mut j: i32 = 0;
    let mut failed: i32 = 0 as i32;
    let mut drive: i32 = 0 as i32;
    let mut common: i32 = 0 as i32;
    let mut ip: *mut Item = ::core::ptr::null_mut::<Item>();
    if HSD_GObjPLinkHead.is_null() || it_804D6D28.is_null() || it_804D6D24.is_null()
        || (*it_804D6D24.offset(It_Kind_Capsule as i32 as isize))
            .is_null()
    {
        return -(1 as i32);
    }
    if Geno_ItemActive(GENO_ITEM_KIND_BASE + def) == 0 {
        return -(2 as i32);
    }
    i = 0 as i32;
    while i < 30 as i32 {
        saved[i as usize] = it_804A0F60[i as usize];
        i += 1;
    }
    stale.x10_modelDesc = 0xff73ff73 as u32 as *mut ItemModelDesc;
    fresh.x0_common_attr = &raw mut attr;
    fresh.x10_modelDesc = &raw mut model;
    model.x0_joint = &raw mut joint;
    probe_g.user_data = &raw mut probe as *mut c_void;
    probe.kind = It_Kind_Nokonoko;
    i = 0 as i32;
    while i < 4 as i32 {
        it_804A0F60[(It_Kind_Nokonoko as i32
            - It_Kind_Old_Kuri as i32) as usize] = &raw mut stale;
        it_ResetStageArticleTable();
        j = 0 as i32;
        while j < 30 as i32 {
            if !it_804A0F60[j as usize].is_null() {
                failed += 1;
            }
            j += 1;
        }
        it_8026B40C(&raw mut fresh, It_Kind_Nokonoko as i32 as i32);
        Item_80267978(&raw mut probe_g);
        if probe.xC4_article_data != &raw mut fresh {
            failed += 1;
            break;
        } else {
            if (*(*probe.xC4_article_data).x10_modelDesc).x0_joint != &raw mut joint
                || (*probe.xC4_article_data).x0_common_attr != &raw mut attr
            {
                failed += 1;
            }
            drive = Geno_ItemSpawn(
                def,
                0 as i32,
                geno_item_bits(60 as i32 as f32),
                0 as i32,
                0 as i32,
                0x74000000 as i32 + i * 2 as i32,
                701 as i32,
                0 as i32,
                1 as i32,
            );
            if drive <= 0 as i32 {
                failed += 1;
                drive = 0 as i32;
                break;
            } else {
                j = 0 as i32;
                loop {
                    ip = geno_item_index(j);
                    if ip.is_null() {
                        break;
                    }
                    if (*geno_item_vars(ip)).handle == drive {
                        break;
                    }
                    j += 1;
                }
                if ip.is_null()
                    || (*ip).xC4_article_data
                        != (&raw mut geno_item_desc as *mut Article).offset(def as isize)
                            as *mut Article
                    || (*ip).xC8_joint != &raw mut geno_item_joint
                    || (*ip).xCC_item_attr
                        != (&raw mut geno_item_attr as *mut ItemAttr)
                            .offset(def as isize) as *mut ItemAttr
                {
                    failed += 1;
                }
                j = 0 as i32;
                while j < 2 as i32 {
                    if j == 1 as i32 {
                        Geno_ItemDespawn(drive, 701 as i32);
                        drive = 0 as i32;
                    }
                    common = Geno_ItemSpawnKind(
                        It_Kind_Capsule as i32,
                        0 as i32,
                        geno_item_bits(60 as i32 as f32),
                        0 as i32,
                        0 as i32,
                        0x75000000 as i32 + i * 2 as i32
                            + j,
                        702 as i32,
                        0 as i32,
                        0 as i32,
                    );
                    if common <= 0 as i32 {
                        failed += 1;
                        common = 0 as i32;
                        break;
                    } else {
                        let mut k: i32 = 0;
                        let mut cur: *mut HSD_GObj = ::core::ptr::null_mut::<HSD_GObj>();
                        cur = *HSD_GObjPLinkHead.offset(HSD_GOBJ_PLINK_ITEM as isize);
                        while !cur.is_null() {
                            if Geno_ItemMeta(
                                (*(HSD_GObjGetUserData(cur) as *mut Item)).x1C
                                    as i32,
                                GENO_II_HANDLE as i32,
                            ) == common
                            {
                                break;
                            }
                            cur = (*cur).next as *mut HSD_GObj;
                        }
                        if cur.is_null() {
                            failed += 1;
                        } else {
                            let mut retail: *mut Item = HSD_GObjGetUserData(cur)
                                as *mut Item;
                            Item_80267978(cur);
                            if (*retail).xC4_article_data
                                != *it_804D6D24
                                    .offset(It_Kind_Capsule as i32 as isize)
                                || (*retail).xC8_joint
                                    != (*(**it_804D6D24
                                        .offset(It_Kind_Capsule as i32 as isize))
                                        .x10_modelDesc)
                                        .x0_joint
                                || (*retail).xCC_item_attr
                                    != (**it_804D6D24
                                        .offset(It_Kind_Capsule as i32 as isize))
                                        .x0_common_attr
                            {
                                failed += 1;
                            }
                            k = 0 as i32;
                            while k < GENO_ITEM_MAX_DEFS {
                                if (*retail).xC4_article_data
                                    == (&raw mut geno_item_desc as *mut Article)
                                        .offset(k as isize) as *mut Article
                                {
                                    failed += 1;
                                }
                                k += 1;
                            }
                        }
                        Geno_ItemDespawn(common, 702 as i32);
                        common = 0 as i32;
                        j += 1;
                    }
                }
                if drive != 0 {
                    Geno_ItemDespawn(drive, 701 as i32);
                    drive = 0 as i32;
                }
                i += 1;
            }
        }
    }
    if drive != 0 {
        Geno_ItemDespawn(drive, 701 as i32);
    }
    if common != 0 {
        Geno_ItemDespawn(common, 702 as i32);
    }
    i = 0 as i32;
    while i < 30 as i32 {
        it_804A0F60[i as usize] = saved[i as usize];
        i += 1;
    }
    return failed;
}

pub unsafe extern "C" fn geno_combo_block(mut fp: *mut Fighter) -> *mut GenoComboBlock {
    melee_core::ct!("geno_combo_block");
    let mut p: i32 = if ((*fp).player_id as i32)
        < GENO_PLAYERS
    {
        (*fp).player_id as i32
    } else {
        GENO_PLAYERS - 1 as i32
    };
    return (&raw mut *(&raw mut Geno_ComboBlock as *mut [GenoComboBlock; 2])
        .offset(p as isize) as *mut GenoComboBlock)
        .offset(
            (if (*fp).is_sub_fighter() as i32 != 0 {
                1 as i32
            } else {
                0 as i32
            }) as isize,
        ) as *mut GenoComboBlock;
}

pub unsafe extern "C" fn GenoGame_ComboBlockOf(
    mut fp: *mut Fighter,
) -> *mut c_void {
    melee_core::ct!("GenoGame_ComboBlockOf");
    return geno_combo_block(fp) as *mut c_void;
}

pub unsafe extern "C" fn geno_combo_on(mut st: *mut GenoState) -> i32 {
    melee_core::ct!("geno_combo_on");
    return ((*st).profile >= 0 as i32
        && Geno_ComboParam(
            (*st).profile as i32,
            GENO_CP_ROUTES as i32,
        ) > 0 as i32) as i32;
}

pub unsafe extern "C" fn geno_combo_end_string(mut cb: *mut GenoComboBlock) {
    melee_core::ct!("geno_combo_end_string");
    let mut i: i32 = 0;
    (*cb).links = 0 as i32 as i32;
    (*cb).motion = -(1 as i32) as i32;
    i = 0 as i32;
    while i < GENO_COMBO_MAX_ROUTES / 32 as i32 {
        (*cb).once[i as usize] = 0 as u32;
        i += 1;
    }
}

pub unsafe extern "C" fn geno_combo_reset(mut fp: *mut Fighter, mut st: *mut GenoState) {
    melee_core::ct!("geno_combo_reset");
    let mut cb: *mut GenoComboBlock = geno_combo_block(fp);
    let mut i: i32 = 0;
    geno_zero(
        cb as *mut c_void,
        ::core::mem::size_of::<GenoComboBlock>() as i32,
    );
    (*cb).last_route = -(1 as i32) as i32;
    (*cb).motion = -(1 as i32) as i32;
    i = 0 as i32;
    while i < GENO_CI_COUNT as i32 {
        (*cb).age[i as usize] = GENO_COMBO_STALE as i32;
        i += 1;
    }
    if geno_combo_on(st) != 0 {
        (*cb).ki = Geno_ComboParam(
            (*st).profile as i32,
            GENO_CP_KI_START as i32,
        ) as i32;
    }
}

pub unsafe extern "C" fn geno_combo_ki_add(
    mut fp: *mut Fighter,
    mut st: *mut GenoState,
    mut d: i32,
) {
    melee_core::ct!("geno_combo_ki_add");
    let mut cb: *mut GenoComboBlock = geno_combo_block(fp);
    let mut mx: i32 = Geno_ComboParam(
        (*st).profile as i32,
        GENO_CP_KI_MAX as i32,
    ) as i32;
    (*cb).ki += d;
    if (*cb).ki > mx {
        (*cb).ki = mx;
    }
    if (*cb).ki < 0 as i32 {
        (*cb).ki = 0 as i32 as i32;
    }
}

pub unsafe extern "C" fn geno_combo_action(mut fp: *mut Fighter, mut st: *mut GenoState) {
    melee_core::ct!("geno_combo_action");
    let mut cb: *mut GenoComboBlock = ::core::ptr::null_mut::<GenoComboBlock>();
    let mut i: i32 = 0;
    if geno_combo_on(st) == 0 {
        return;
    }
    cb = geno_combo_block(fp);
    i = 0 as i32;
    while i < GENO_CI_COUNT as i32 {
        (*cb).age[i as usize] = GENO_COMBO_STALE as i32;
        i += 1;
    }
    (*cb).hit = 0 as i32 as i32;
    (*cb).blocked = 0 as i32 as i32;
    if geno_combo_entering == 0 && (*cb).links > 0 as i32
        && (*fp).motion_id != (*cb).motion
    {
        geno_combo_end_string(cb);
    }
}

pub unsafe extern "C" fn Geno_ComboContact(
    mut fp: *mut Fighter,
    mut kind: i32,
) {
    melee_core::ct!("Geno_ComboContact");
    let mut st: *mut GenoState = ::core::ptr::null_mut::<GenoState>();
    let mut cb: *mut GenoComboBlock = ::core::ptr::null_mut::<GenoComboBlock>();
    if fp.is_null() {
        return;
    }
    st = geno_state(fp);
    if geno_combo_on(st) == 0 {
        return;
    }
    cb = geno_combo_block(fp);
    if kind == 1 as i32 {
        if (*cb).blocked == 0 {
            geno_combo_ki_add(
                fp,
                st,
                Geno_ComboParam(
                    (*st).profile as i32,
                    GENO_CP_KI_ON_BLOCK as i32,
                ) as i32,
            );
        }
        (*cb).blocked = 1 as i32 as i32;
        return;
    }
    (*cb).hit = 1 as i32 as i32;
    geno_combo_ki_add(
        fp,
        st,
        Geno_ComboParam(
            (*st).profile as i32,
            GENO_CP_KI_ON_HIT as i32,
        ) as i32,
    );
}

pub unsafe extern "C" fn geno_combo_taken_hit(mut fp: *mut Fighter, mut st: *mut GenoState) {
    melee_core::ct!("geno_combo_taken_hit");
    if geno_combo_on(st) != 0 {
        (*geno_combo_block(fp)).hits_seen = (*st).hit_count;
        geno_combo_ki_add(
            fp,
            st,
            Geno_ComboParam(
                (*st).profile as i32,
                GENO_CP_KI_ON_TAKEN as i32,
            ) as i32,
        );
    }
}

pub unsafe extern "C" fn geno_combo_dir(
    mut fp: *mut Fighter,
    mut x: f32,
    mut y: f32,
) -> i32 {
    melee_core::ct!("geno_combo_dir");
    let mut ax: f32 = if x < 0.0f32 { -x } else { x };
    let mut ay: f32 = if y < 0.0f32 { -y } else { y };
    let mut dz: f32 = if !p_ftCommonData.is_null() {
        (*p_ftCommonData).x98 as f32
    } else {
        0.3f32
    };
    if ax < dz && ay < dz {
        return GENO_CD_NEUTRAL as i32;
    }
    if ay > ax {
        return if y > 0.0f32 {
            GENO_CD_UP as i32
        } else {
            GENO_CD_DOWN as i32
        };
    }
    return if x as f32 * (*fp).facing_dir >= 0.0f32 {
        GENO_CD_FORWARD as i32
    } else {
        GENO_CD_BACK as i32
    };
}

pub unsafe extern "C" fn geno_combo_smash(
    mut fp: *mut Fighter,
    mut a: i32,
    mut dir: *mut i32,
) -> i32 {
    melee_core::ct!("geno_combo_smash");
    let mut thr: f32 = if !p_ftCommonData.is_null() {
        (*p_ftCommonData).dash_smash_stick_threshold as f32
    } else {
        0.8f32
    };
    let mut cx: f32 = (*fp).input.cstick[0 as i32 as usize].x;
    let mut cy: f32 = (*fp).input.cstick[0 as i32 as usize].y;
    let mut px: f32 = (*fp).input.cstick[1 as i32 as usize].x;
    let mut py: f32 = (*fp).input.cstick[1 as i32 as usize].y;
    if (if cx < 0 as i32 as f32 { -cx } else { cx }) >= thr
        && (if px < 0 as i32 as f32 { -px } else { px }) < thr
        || (if cy < 0 as i32 as f32 { -cy } else { cy }) >= thr
            && (if py < 0 as i32 as f32 { -py } else { py }) < thr
    {
        *dir = geno_combo_dir(fp, cx, cy);
        return 1 as i32;
    }
    if a == 0
        || (*fp).ground_or_air as i32 != GA_Ground as i32
    {
        return 0 as i32;
    }
    if (if (*fp).input.lstick[0 as i32 as usize].x
        < 0 as i32 as f32
    {
        -(*fp).input.lstick[0 as i32 as usize].x
    } else {
        (*fp).input.lstick[0 as i32 as usize].x
    }) >= thr
        && ((*fp).active_timer.lstick.x as i32)
            < (if !p_ftCommonData.is_null() {
                (*p_ftCommonData).dash_smash_window
            } else {
                2 as i32
            })
    {
        *dir = if (*fp).input.lstick[0 as i32 as usize].x
            as f32 * (*fp).facing_dir >= 0.0f32
        {
            GENO_CD_FORWARD as i32
        } else {
            GENO_CD_BACK as i32
        };
        return 1 as i32;
    }
    if (*fp).input.lstick[0 as i32 as usize].y
        >= (if !p_ftCommonData.is_null() { (*p_ftCommonData).xCC } else { 0.6625f32 })
        && ((*fp).active_timer.lstick.y as i32 as f32)
            < (if !p_ftCommonData.is_null() {
                (*p_ftCommonData).xD0
            } else {
                4 as i32 as f32
            })
    {
        *dir = GENO_CD_UP as i32;
        return 1 as i32;
    }
    if (*fp).input.lstick[0 as i32 as usize].y
        <= (if !p_ftCommonData.is_null() { (*p_ftCommonData).xD4 } else { -0.6625f32 })
        && ((*fp).active_timer.lstick.y as i32 as f32)
            < (if !p_ftCommonData.is_null() {
                (*p_ftCommonData).xD8
            } else {
                4 as i32 as f32
            })
    {
        *dir = GENO_CD_DOWN as i32;
        return 1 as i32;
    }
    return 0 as i32;
}

pub unsafe extern "C" fn geno_combo_press(
    mut cb: *mut GenoComboBlock,
    mut cls: i32,
    mut dir: i32,
) {
    melee_core::ct!("geno_combo_press");
    (*cb).age[cls as usize] = 0 as i32 as i32;
    (*cb).dir[cls as usize] = dir as i32;
}

pub unsafe extern "C" fn Geno_ComboInput(mut gobj: *mut Fighter_GObj) {
    melee_core::ct!("Geno_ComboInput");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj as *mut HSD_GObj)
        as *mut Fighter;
    let mut st: *mut GenoState = geno_state(fp);
    let mut cb: *mut GenoComboBlock = ::core::ptr::null_mut::<GenoComboBlock>();
    let mut pr: HSD_Pad = 0;
    let mut hd: HSD_Pad = 0;
    let mut i: i32 = 0;
    let mut a: i32 = 0;
    let mut d: i32 = 0;
    let mut lag: i32 = 0;
    if geno_combo_on(st) == 0 {
        return;
    }
    cb = geno_combo_block(fp);
    lag = if (*fp).x2219_b5() as i32 != 0 {
        1 as i32
    } else {
        0 as i32
    };
    if lag == 0 {
        i = 0 as i32;
        while i < GENO_CI_COUNT as i32 {
            if (*cb).age[i as usize] < GENO_COMBO_STALE {
                (*cb).age[i as usize] += 1;
            }
            i += 1;
        }
    }
    pr = (*fp).input.pressed_buttons;
    hd = (*fp).input.held_buttons[0 as i32 as usize];
    a = (pr & HSD_PAD_A as HSD_Pad != 0 as HSD_Pad) as i32;
    d = GENO_CD_NEUTRAL as i32;
    if geno_combo_smash(fp, a, &raw mut d) != 0 {
        geno_combo_press(cb, GENO_CI_HEAVY as i32, d);
    } else if a != 0 {
        geno_combo_press(
            cb,
            GENO_CI_LIGHT as i32,
            geno_combo_dir(
                fp,
                (*fp).input.lstick[0 as i32 as usize].x,
                (*fp).input.lstick[0 as i32 as usize].y,
            ),
        );
    }
    if pr & HSD_PAD_B as HSD_Pad != 0 {
        geno_combo_press(
            cb,
            GENO_CI_SPECIAL as i32,
            geno_combo_dir(
                fp,
                (*fp).input.lstick[0 as i32 as usize].x,
                (*fp).input.lstick[0 as i32 as usize].y,
            ),
        );
    }
    if pr & HSD_PAD_XY as HSD_Pad != 0 {
        geno_combo_press(
            cb,
            GENO_CI_JUMP as i32,
            geno_combo_dir(
                fp,
                (*fp).input.lstick[0 as i32 as usize].x,
                (*fp).input.lstick[0 as i32 as usize].y,
            ),
        );
    }
    if pr & HSD_PAD_Z as HSD_Pad != 0 {
        geno_combo_press(
            cb,
            GENO_CI_GRAB as i32,
            geno_combo_dir(
                fp,
                (*fp).input.lstick[0 as i32 as usize].x,
                (*fp).input.lstick[0 as i32 as usize].y,
            ),
        );
    }
    if pr & (HSD_PAD_L | HSD_PAD_R) as HSD_Pad != 0 {
        geno_combo_press(
            cb,
            GENO_CI_SHIELD as i32,
            geno_combo_dir(
                fp,
                (*fp).input.lstick[0 as i32 as usize].x,
                (*fp).input.lstick[0 as i32 as usize].y,
            ),
        );
    }
    if pr & HSD_PAD_DPADDOWN as HSD_Pad != 0 {
        geno_combo_press(
            cb,
            GENO_CI_BLAST as i32,
            geno_combo_dir(
                fp,
                (*fp).input.lstick[0 as i32 as usize].x,
                (*fp).input.lstick[0 as i32 as usize].y,
            ),
        );
    }
    (*cb).hold[GENO_CI_LIGHT as i32 as usize] = (if hd
        & HSD_PAD_A as HSD_Pad != 0
    {
        (*cb).hold[GENO_CI_LIGHT as i32 as usize]
            + 1 as i32
    } else {
        0 as i32
    }) as i32;
    (*cb).hold[GENO_CI_HEAVY as i32 as usize] = (if hd
        & HSD_PAD_A as HSD_Pad != 0
        || (if (*fp).input.cstick[0 as i32 as usize].x
            < 0 as i32 as f32
        {
            -(*fp).input.cstick[0 as i32 as usize].x
        } else {
            (*fp).input.cstick[0 as i32 as usize].x
        })
            >= (if !p_ftCommonData.is_null() {
                (*p_ftCommonData).dash_smash_stick_threshold
            } else {
                0.8f32
            })
        || (if (*fp).input.cstick[0 as i32 as usize].y
            < 0 as i32 as f32
        {
            -(*fp).input.cstick[0 as i32 as usize].y
        } else {
            (*fp).input.cstick[0 as i32 as usize].y
        })
            >= (if !p_ftCommonData.is_null() {
                (*p_ftCommonData).dash_smash_stick_threshold
            } else {
                0.8f32
            })
    {
        (*cb).hold[GENO_CI_HEAVY as i32 as usize]
            + 1 as i32
    } else {
        0 as i32
    }) as i32;
    (*cb).hold[GENO_CI_SPECIAL as i32 as usize] = (if hd
        & HSD_PAD_B as HSD_Pad != 0
    {
        (*cb).hold[GENO_CI_SPECIAL as i32 as usize]
            + 1 as i32
    } else {
        0 as i32
    }) as i32;
    (*cb).hold[GENO_CI_JUMP as i32 as usize] = (if hd
        & HSD_PAD_XY as HSD_Pad != 0
    {
        (*cb).hold[GENO_CI_JUMP as i32 as usize] + 1 as i32
    } else {
        0 as i32
    }) as i32;
    (*cb).hold[GENO_CI_GRAB as i32 as usize] = (if hd
        & HSD_PAD_Z as HSD_Pad != 0
    {
        (*cb).hold[GENO_CI_GRAB as i32 as usize] + 1 as i32
    } else {
        0 as i32
    }) as i32;
    (*cb).hold[GENO_CI_SHIELD as i32 as usize] = (if hd
        & (HSD_PAD_L | HSD_PAD_R) as HSD_Pad != 0
    {
        (*cb).hold[GENO_CI_SHIELD as i32 as usize]
            + 1 as i32
    } else {
        0 as i32
    }) as i32;
    (*cb).hold[GENO_CI_BLAST as i32 as usize] = (if hd
        & HSD_PAD_DPADDOWN as HSD_Pad != 0
    {
        (*cb).hold[GENO_CI_BLAST as i32 as usize]
            + 1 as i32
    } else {
        0 as i32
    }) as i32;
    i = 0 as i32;
    while i < GENO_CI_COUNT as i32 {
        if (*cb).hold[i as usize] > 9999 as i32 {
            (*cb).hold[i as usize] = 9999 as i32 as i32;
        }
        i += 1;
    }
}

pub unsafe extern "C" fn geno_combo_dir_ok(
    mut want: i32,
    mut got: i32,
) -> i32 {
    melee_core::ct!("geno_combo_dir_ok");
    if want == GENO_CD_ANY as i32 {
        return 1 as i32;
    }
    if want == GENO_CD_SIDE as i32 {
        return (got == GENO_CD_FORWARD as i32
            || got == GENO_CD_BACK as i32) as i32;
    }
    return (want == got) as i32;
}

pub unsafe extern "C" fn geno_combo_from_motion(mut from: u32) -> i32 {
    melee_core::ct!("geno_combo_from_motion");
    if from & 0xffff as u32 == GENO_COMBO_FROM_ANY {
        return -(1 as i32);
    }
    if from >> 28 as i32 == GENO_TGT_GENO {
        return GENO_MOTION_BASE + (from & 0xffff as u32) as i32;
    }
    return (from & 0xffff as u32) as i32;
}

pub unsafe extern "C" fn geno_combo_open(
    mut fp: *mut Fighter,
    mut st: *mut GenoState,
    mut cb: *mut GenoComboBlock,
    mut r: i32,
) -> i32 {
    melee_core::ct!("geno_combo_open");
    let mut p: i32 = (*st).profile as i32;
    let mut from: i32 = geno_combo_from_motion(
        Geno_ComboWord(p, r, GENO_CR_FROM as i32) as u32,
    );
    let mut on: i32 = Geno_ComboWord(
        p,
        r,
        GENO_CR_ON as i32,
    );
    let mut wh: i32 = Geno_ComboWord(
        p,
        r,
        GENO_CR_WHERE as i32,
    );
    let mut lo: i32 = Geno_ComboWord(
        p,
        r,
        GENO_CR_WIN_LO as i32,
    );
    let mut hi: i32 = Geno_ComboWord(
        p,
        r,
        GENO_CR_WIN_HI as i32,
    );
    let mut slot: i32 = Geno_ComboWord(
        p,
        r,
        GENO_CR_SLOT as i32,
    );
    let mut t: i32 = (*st).action_time;
    if from >= 0 as i32 && (*fp).motion_id != from {
        return 0 as i32;
    }
    if from < 0 as i32 && (*cb).links == 0 as i32 {
        return 0 as i32;
    }
    if wh == GENO_CW_GROUND as i32
        && (*fp).ground_or_air as i32 != GA_Ground as i32
        || wh == GENO_CW_AIR as i32
            && (*fp).ground_or_air as i32
                == GA_Ground as i32
    {
        return 0 as i32;
    }
    if t < lo || hi > 0 as i32 && t > hi {
        return 0 as i32;
    }
    if on == GENO_CO_HIT as i32 && (*cb).hit == 0
        || on == GENO_CO_WHIFF as i32
            && ((*cb).hit != 0 || (*cb).blocked != 0)
        || on == GENO_CO_BLOCK as i32 && (*cb).blocked == 0
        || on == GENO_CO_CONTACT as i32 && (*cb).hit == 0
            && (*cb).blocked == 0
    {
        return 0 as i32;
    }
    if Geno_ComboWord(p, r, GENO_CR_STARTS as i32) == 0
        && (*cb).links >= Geno_ComboWord(p, r, GENO_CR_DEPTH as i32)
    {
        return 0 as i32;
    }
    if (*cb).links >= Geno_ComboParam(p, GENO_CP_MAX_LINKS as i32)
        && Geno_ComboWord(p, r, GENO_CR_STARTS as i32) == 0
    {
        return 0 as i32;
    }
    if Geno_ComboWord(p, r, GENO_CR_ONCE as i32) != 0
        && (*cb).once[(r >> 5 as i32) as usize]
            & (1 as u32) << (r & 31 as i32) != 0
    {
        return 0 as i32;
    }
    if (*cb).ki < Geno_ComboWord(p, r, GENO_CR_KI_MIN as i32)
        || (*cb).ki < Geno_ComboWord(p, r, GENO_CR_KI_COST as i32)
    {
        return 0 as i32;
    }
    if slot >= 0 as i32 && slot < GENO_LUA_STATE_SLOTS
        && (*geno_lua_block(fp)).slot[slot as usize]
            < Geno_ComboWord(p, r, GENO_CR_SLOT_MIN as i32)
    {
        return 0 as i32;
    }
    return 1 as i32;
}

pub unsafe extern "C" fn geno_combo_input_ok(
    mut st: *mut GenoState,
    mut cb: *mut GenoComboBlock,
    mut r: i32,
) -> i32 {
    melee_core::ct!("geno_combo_input_ok");
    let mut p: i32 = (*st).profile as i32;
    let mut in_0: i32 = Geno_ComboWord(
        p,
        r,
        GENO_CR_INPUT as i32,
    );
    if in_0 == GENO_CI_NONE as i32 {
        return 1 as i32;
    }
    if in_0 < 0 as i32 || in_0 >= GENO_CI_COUNT as i32 {
        return 0 as i32;
    }
    if Geno_ComboWord(p, r, GENO_CR_HOLD as i32) > 0 as i32
    {
        return ((*cb).hold[in_0 as usize]
            >= Geno_ComboWord(p, r, GENO_CR_HOLD as i32)
            && (*cb).age[in_0 as usize] != GENO_COMBO_STALE
            && geno_combo_dir_ok(
                Geno_ComboWord(p, r, GENO_CR_DIR as i32),
                (*cb).dir[in_0 as usize],
            ) != 0) as i32;
    }
    return ((*cb).age[in_0 as usize]
        <= Geno_ComboWord(p, r, GENO_CR_BUFFER as i32)
        && geno_combo_dir_ok(
            Geno_ComboWord(p, r, GENO_CR_DIR as i32),
            (*cb).dir[in_0 as usize],
        ) != 0) as i32;
}

pub unsafe extern "C" fn geno_combo_enter(
    mut gobj: *mut Fighter_GObj,
    mut fp: *mut Fighter,
    mut st: *mut GenoState,
    mut to: u32,
) -> i32 {
    melee_core::ct!("geno_combo_enter");
    if to >> 28 as i32 == GENO_COMBO_TGT_ACT && to != GENO_TGT_AUTO
        && to != GENO_TGT_HELPLESS && to != GENO_NONE_T
    {
        let mut id: i32 = (to & 0xffff as u32) as i32;
        let mut air: i32 = ((*fp).ground_or_air as i32
            != GA_Ground as i32) as i32;
        if id <= GENO_CA_SPECIAL_LW as i32 {
            if geno_test_capture != 0 {
                geno_test_target = to;
                geno_test_changes += 1;
                return 1 as i32;
            }
            return Geno_SpecialEnter(
                gobj,
                id
                    + (if air != 0 {
                        GENO_SP_AIR_N as i32
                    } else {
                        GENO_SP_N as i32
                    }),
            );
        }
        if id == GENO_CA_JUMP as i32
            || id == GENO_CA_JUMP_SHORT as i32
        {
            if geno_test_capture != 0 {
                geno_test_target = to;
                geno_test_changes += 1;
                return 1 as i32;
            }
            if air == 0 {
                (*fp).mv.co.kneebend.is_short_hop = (id
                    == GENO_CA_JUMP_SHORT as i32) as i32;
                ftCo_Jump_Enter(gobj);
                return 1 as i32;
            }
            if ((*fp).x1968_jumpsUsed as i32) < (*fp).co_attrs.max_jumps {
                ftCo_JumpAerial_Enter_Basic(gobj);
                return 1 as i32;
            }
            return 0 as i32;
        }
        if id >= GENO_CA_AIR_N as i32
            && id <= GENO_CA_AIR_LW as i32
        {
            if air == 0 {
                return 0 as i32;
            }
            if geno_test_capture != 0 {
                geno_test_target = to;
                geno_test_changes += 1;
                return 1 as i32;
            }
            ftCo_AttackAir_EnterFromMsid(
                gobj,
                ftCo_MS_AttackAirN as i32
                    + (id - GENO_CA_AIR_N as i32),
            );
            return 1 as i32;
        }
        return 0 as i32;
    }
    return geno_go(gobj, fp, st, to);
}

pub unsafe extern "C" fn Geno_ComboTick(
    mut gobj: *mut Fighter_GObj,
) -> i32 {
    melee_core::ct!("Geno_ComboTick");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj as *mut HSD_GObj)
        as *mut Fighter;
    let mut st: *mut GenoState = geno_state(fp);
    let mut cb: *mut GenoComboBlock = ::core::ptr::null_mut::<GenoComboBlock>();
    let mut p: i32 = 0;
    let mut n: i32 = 0;
    let mut r: i32 = 0;
    if geno_combo_on(st) == 0 {
        return 0 as i32;
    }
    cb = geno_combo_block(fp);
    if (*cb).links > 0 as i32 && (*fp).motion_id != (*cb).motion {
        geno_combo_end_string(cb);
    }
    p = (*st).profile as i32;
    n = Geno_ComboParam(p, GENO_CP_ROUTES as i32);
    r = 0 as i32;
    while r < n {
        let mut to: u32 = 0;
        let mut in_0: i32 = 0;
        let mut from_motion: i32 = 0;
        let mut from_frame: i32 = 0;
        let mut entered: i32 = 0;
        if !(geno_combo_open(fp, st, cb, r) == 0 || geno_combo_input_ok(st, cb, r) == 0)
        {
            to = Geno_ComboWord(p, r, GENO_CR_TO as i32) as u32;
            in_0 = Geno_ComboWord(p, r, GENO_CR_INPUT as i32);
            from_motion = (*fp).motion_id;
            from_frame = (*st).action_time;
            if Geno_ComboWord(p, r, GENO_CR_STARTS as i32) != 0 {
                geno_combo_end_string(cb);
            }
            geno_combo_entering = 1 as i32 as i32;
            entered = geno_combo_enter(gobj, fp, st, to);
            geno_combo_entering = 0 as i32 as i32;
            if !(entered == 0) {
                cb = geno_combo_block(fp);
                if in_0 >= 0 as i32
                    && in_0 < GENO_CI_COUNT as i32
                {
                    (*cb).age[in_0 as usize] = GENO_COMBO_STALE as i32;
                }
                (*cb).ki -= Geno_ComboWord(p, r, GENO_CR_KI_COST as i32);
                if (*cb).ki < 0 as i32 {
                    (*cb).ki = 0 as i32 as i32;
                }
                if Geno_ComboWord(p, r, GENO_CR_ONCE as i32) != 0 {
                    (*cb).once[(r >> 5 as i32) as usize]
                        |= (1 as u32) << (r & 31 as i32);
                }
                (*cb).links += 1;
                (*cb).taken += 1;
                (*cb).last_route = r as i32;
                (*cb).motion = (*fp).motion_id;
                Geno_ComboNote(
                    p,
                    r,
                    from_motion,
                    (*fp).motion_id,
                    from_frame,
                    (*cb).ki
                        | ((*cb).links as i32) << 24 as i32,
                );
                return 1 as i32;
            }
        }
        r += 1;
    }
    return 0 as i32;
}

pub unsafe extern "C" fn geno_combo_digest(
    mut fp: *mut Fighter,
    mut st: *mut GenoState,
) -> u32 {
    melee_core::ct!("geno_combo_digest");
    let mut cb: *mut GenoComboBlock = ::core::ptr::null_mut::<GenoComboBlock>();
    let mut w: *const u32 = ::core::ptr::null::<u32>();
    let mut h: u32 = 0x434f4d42 as u32;
    let mut i: i32 = 0;
    if geno_combo_on(st) == 0 {
        return 0 as u32;
    }
    cb = geno_combo_block(fp);
    w = cb as *const u32;
    i = 0 as i32;
    while i
        < (::core::mem::size_of::<GenoComboBlock>() as usize).wrapping_div(4 as usize)
            as i32
    {
        h = geno_dg(h, *w.offset(i as isize));
        i += 1;
    }
    return if h != 0 as u32 { h } else { 1 as u32 };
}

pub unsafe extern "C" fn GenoGame_ComboInfo(
    mut port: i32,
    mut what: i32,
) -> i32 {
    melee_core::ct!("GenoGame_ComboInfo");
    let mut cur: *mut HSD_GObj = ::core::ptr::null_mut::<HSD_GObj>();
    if HSD_GObjPLinkHead.is_null() {
        return -(1 as i32);
    }
    cur = *HSD_GObjPLinkHead.offset(HSD_GOBJ_PLINK_FIGHTER as isize);
    while !cur.is_null() {
        let mut fp: *mut Fighter = HSD_GObjGetUserData(cur) as *mut Fighter;
        if (*fp).player_id as i32 == port && (*fp).is_sub_fighter() == 0 {
            let mut st: *mut GenoState = geno_state(fp);
            let mut cb: *mut GenoComboBlock = geno_combo_block(fp);
            if geno_combo_on(st) == 0 {
                return -(1 as i32);
            }
            match what {
                0 => return (*cb).links as i32,
                1 => return (*cb).ki as i32,
                2 => return (*cb).last_route as i32,
                3 => {
                    return Geno_ComboParam(
                        (*st).profile as i32,
                        GENO_CP_ROUTES as i32,
                    );
                }
                4 => return (*cb).hit as i32,
                5 => return (*cb).blocked as i32,
                6 => return (*cb).taken as i32,
                7 => {
                    return Geno_ComboParam(
                        (*st).profile as i32,
                        GENO_CP_MAX_LINKS as i32,
                    );
                }
                8 => {
                    return Geno_ComboParam(
                        (*st).profile as i32,
                        GENO_CP_KI_MAX as i32,
                    );
                }
                9 => return (*st).action_time as i32,
                11 => return (*st).profile as i32,
                _ => {}
            }
            if what >= 16 as i32
                && what
                    < 16 as i32
                        + Geno_ComboParam(
                            (*st).profile as i32,
                            GENO_CP_ROUTES as i32,
                        )
            {
                let mut r: i32 = what - 16 as i32;
                if geno_combo_open(fp, st, cb, r) == 0 {
                    return 0 as i32;
                }
                return if geno_combo_input_ok(st, cb, r) != 0 {
                    2 as i32
                } else {
                    1 as i32
                };
            }
            return -(1 as i32);
        }
        cur = (*cur).next as *mut HSD_GObj;
    }
    return -(1 as i32);
}

pub unsafe extern "C" fn GenoGame_ComboSetKi(
    mut port: i32,
    mut ki: i32,
) -> i32 {
    melee_core::ct!("GenoGame_ComboSetKi");
    let mut cur: *mut HSD_GObj = ::core::ptr::null_mut::<HSD_GObj>();
    if HSD_GObjPLinkHead.is_null() {
        return -(1 as i32);
    }
    cur = *HSD_GObjPLinkHead.offset(HSD_GOBJ_PLINK_FIGHTER as isize);
    while !cur.is_null() {
        let mut fp: *mut Fighter = HSD_GObjGetUserData(cur) as *mut Fighter;
        if (*fp).player_id as i32 == port && (*fp).is_sub_fighter() == 0 {
            let mut st: *mut GenoState = geno_state(fp);
            if geno_combo_on(st) == 0 {
                return -(1 as i32);
            }
            (*geno_combo_block(fp)).ki = 0 as i32 as i32;
            geno_combo_ki_add(fp, st, ki as i32);
            return (*geno_combo_block(fp)).ki as i32;
        }
        cur = (*cur).next as *mut HSD_GObj;
    }
    return -(1 as i32);
}


