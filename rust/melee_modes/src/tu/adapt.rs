//! GENERATED adapters: typed bridges from the `ext` stub signatures to the verified ports
//! (see docs/rust-port/missing.md). Each calls the port with `as` conversions of the stub's
//! parameter types; the port is the single implementation of the C function.
#![allow(non_snake_case, unused_imports, unused_unsafe, unused_variables, clippy::all)]
use core::ffi::{c_void, c_char, c_int, c_uint, c_uchar, c_ushort, c_short, c_schar, c_long, c_ulong, c_float, c_double};
use melee_types::*;
pub fn lbAudioAx_80024030__36458a(a0: i32) { unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80024030(a0) } }
pub fn HSD_JObjSetMtxDirty__955390(a0: *mut HSD_JObj) { unsafe { melee_world::tu::grinl::HSD_JObjSetMtxDirty(a0) } }
pub fn HSD_SisLib_803A5ACC__27fcb1(a0: i32, a1: i32, a2: f32, a3: f32, a4: f32, a5: f32, a6: f32) -> *mut HSD_Text { unsafe { melee_runtime::tu::sislib::HSD_SisLib_803A5ACC(a0, a1, a2, a3, a4, a5, a6) } }
pub fn HSD_SisLib_803A6368__2e8f0d(a0: *mut HSD_Text, a1: i32) { unsafe { melee_runtime::tu::sislib::HSD_SisLib_803A6368(a0, a1) } }
pub fn ScriptGame_NoteSpecialInput__c7d07e(a0: *mut Fighter) { unsafe { crate::tu::script_game::ScriptGame_NoteSpecialInput(a0) } }
pub fn ScriptGame_HitRuleItemCreate__a6017c(a0: *mut Item, a1: *mut HitCapsule) { unsafe { crate::tu::script_game::ScriptGame_HitRuleItemCreate(a0, a1) } }
pub fn ScriptGame_HitRuleDamageOnly__c1c74e(a0: *mut HitCapsule) { unsafe { crate::tu::script_game::ScriptGame_HitRuleDamageOnly(a0) } }
pub fn ScriptGame_HitRuleForget__c0be11(a0: *mut HitCapsule) { unsafe { crate::tu::script_game::ScriptGame_HitRuleForget(a0) } }
pub fn atan2f__2f62eb(a0: f32, a1: f32) -> f32 { unsafe { melee_runtime::lbtrigf::atan2f(a0, a1) } }
pub fn cosf__b177e0(a0: f32) -> f32 { unsafe { melee_runtime::gekko::cosf(a0) } }
pub fn sinf__245589(a0: f32) -> f32 { unsafe { melee_runtime::gekko::sinf(a0) } }
pub fn it_ResetStageArticleTable__fad078() { unsafe { melee_items::tu::it_26B1::it_ResetStageArticleTable() } }
pub fn it_8026B40C__05f465(a0: *mut Article, a1: i32) { unsafe { melee_items::tu::it_26B1::it_8026B40C(a0, a1) } }
pub fn it_8026BB68__233026(a0: *mut Item_GObj, a1: *mut Vec) { unsafe { melee_items::tu::it_26B1::it_8026BB68(a0 as _, a1 as _) } }
pub fn ftCo_800D105C__471042(a0: *mut Fighter_GObj) { unsafe { melee_fighter::tu::ftchangeparam::ftco_800d105c(a0 as _) } }
pub fn ftCommon_8007D5D4__8cebce(a0: *mut Fighter) { unsafe { melee_fighter::ftcommon2::ftcommon_8007d5d4(&mut *a0) } }
pub fn ftCommon_8007D7FC__490b7b(a0: *mut Fighter) { unsafe { melee_fighter::ftcommon3::ftcommon_8007d7fc(&mut *a0) } }
pub fn ftPartSetRotX__0c58b0(a0: *mut Fighter, a1: i32, a2: f32) { unsafe { melee_fighter::tu::ftparts::ftPartSetRotX(&*a0, a1, a2) } }
pub fn ftPartSetRotY__9ef908(a0: *mut Fighter, a1: i32, a2: f32) { unsafe { melee_fighter::tu::ftparts::ftPartSetRotY(&*a0, a1, a2) } }
pub fn HSD_AObjSetFlags__2e5e0e(a0: *mut HSD_AObj, a1: u32) { unsafe { melee_runtime::tu::aobj::HSD_AObjSetFlags(a0, a1) } }
pub fn HSD_AObjSetRate__1c2c90(a0: *mut HSD_AObj, a1: f32) { unsafe { melee_runtime::tu::aobj::HSD_AObjSetRate(a0, a1) } }
pub fn HSD_AObjSetEndFrame__29ad4f(a0: *mut HSD_AObj, a1: f32) { unsafe { melee_runtime::tu::aobj::HSD_AObjSetEndFrame(a0, a1) } }
pub fn lbColl_80008440__c0a79c(a0: *mut HitCapsule) { unsafe { crate::lb_ptr::lbColl_80008440(&mut *a0) } }
pub fn ftCo_800D72A0__eb6456(a0: *mut Fighter) -> i32 { unsafe { (melee_fighter::tu::ftco_0d72::ftco_800d72a0(&*a0)) as _ } }
pub fn mpCollSetFacingDir__b3708c(a0: *mut CollData, a1: i32) { unsafe { melee_world::tu::mpcoll::mpCollSetFacingDir(a0, a1) } }
pub fn lb_8000B1CC__23547e(a0: *mut HSD_JObj, a1: *mut Vec, a2: *mut Vec) { unsafe { melee_runtime::tu::lb_00B0::lb_8000B1CC(a0, a1, a2) } }
pub fn HSD_CObjGetCurrent__6f08dd() -> *mut HSD_CObj { unsafe { melee_runtime::tu::cobj::HSD_CObjGetCurrent() } }
pub fn it_8026FCF8__a44c9f(a0: *mut Item, a1: *mut HitCapsule) { unsafe { melee_items::tu::itcoll::it_8026FCF8(a0, a1) } }
pub fn it_8027129C__0e4566(a0: *mut Item_GObj, a1: i32) { unsafe { melee_items::tu::itcoll::it_8027129C(a0 as _, a1) } }
pub fn it_80272460__3b2b2c(a0: *mut HitCapsule, a1: u32, a2: *mut Item_GObj) { unsafe { melee_items::tu::itcoll::it_80272460(a0, a1, a2 as _) } }
pub fn it_8026E9A4__61791d(a0: *mut HSD_GObj, a1: *mut Vec, a2: *mut Vec, a3: *mut Vec) -> i32 { unsafe { melee_items::tu::itgroundcoll::it_8026E9A4(a0, a1 as _, a2 as _, a3 as _) } }
pub fn Item_80267978__71f703(a0: *mut HSD_GObj) { unsafe { melee_items::tu::item::Item_80267978(a0) } }
pub fn Item_80268B18__f9c679(a0: *mut SpawnItem) -> *mut Item_GObj { unsafe { (melee_items::tu::item::Item_80268B18(a0)) as _ } }
pub fn Item_80268E5C__0d30a0(a0: *mut HSD_GObj, a1: enum_t, a2: Item_StateChangeFlags) { unsafe { melee_items::tu::item::Item_80268E5C(a0, a1, a2) } }
pub fn Item_8026A8EC__e93779(a0: *mut Item_GObj) { unsafe { melee_items::tu::item::Item_8026A8EC(a0 as _) } }
pub fn mpCheckCeiling__94c9dc(a0: f32, a1: f32, a2: f32, a3: f32, a4: *mut Vec, a5: *mut i32, a6: *mut u32, a7: *mut Vec, a8: i32, a9: i32) -> i32 { unsafe { melee_world::tu::mplib::mpCheckCeiling(a0, a1, a2, a3, a4, a5, a6, a7, a8, a9) } }
pub fn mpCheckLeftWall__d5d908(a0: f32, a1: f32, a2: f32, a3: f32, a4: *mut Vec, a5: *mut i32, a6: *mut u32, a7: *mut Vec, a8: i32, a9: i32) -> i32 { unsafe { melee_world::tu::mplib::mpCheckLeftWall(a0, a1, a2, a3, a4, a5, a6, a7, a8, a9) } }
pub fn mpCheckRightWall__f24271(a0: f32, a1: f32, a2: f32, a3: f32, a4: *mut Vec, a5: *mut i32, a6: *mut u32, a7: *mut Vec, a8: i32, a9: i32) -> i32 { unsafe { melee_world::tu::mplib::mpCheckRightWall(a0, a1, a2, a3, a4, a5, a6, a7, a8, a9) } }
pub fn mpGetSpeed__f20f7d(a0: i32, a1: *mut Vec, a2: *mut Vec) -> i32 { unsafe { melee_world::tu::mplib::mpGetSpeed(a0, a1, a2) } }
pub fn Player_GetEntity__090538(a0: i32) -> *mut HSD_GObj { unsafe { melee_fighter::tu::player::Player_GetEntity(a0) } }
pub fn Item_ScriptSpawnAtCap__e6349b(a0: *mut SpawnItem) -> i32 { unsafe { melee_items::tu::item::Item_ScriptSpawnAtCap(a0) } }
pub fn Item_ScriptSpawnCapTest__c9b323() -> i32 { unsafe { melee_items::tu::item::Item_ScriptSpawnCapTest() } }
pub fn GenoFly_CursorTest__be3291() -> i32 { unsafe { crate::tu::geno_lab_mode::GenoFly_CursorTest() } }
pub fn GenoGame_AimStick__f25e58(a0: f32, a1: f32, a2: f32, a3: i32, a4: f32, a5: f32, a6: f32, a7: i32, a8: *mut f32, a9: *mut f32, a10: *mut f32) -> i32 { unsafe { crate::tu::geno_game::GenoGame_AimStick(a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10) } }
pub fn Geno_NoHitlag__37cfca(a0: *mut Fighter) -> i32 { unsafe { crate::tu::geno_game::Geno_NoHitlag(a0) } }
pub fn Geno_HitReact__0135dc(a0: *mut Fighter) -> i32 { unsafe { crate::tu::geno_game::Geno_HitReact(a0) } }
pub fn Geno_HitFlags__0264ec(a0: *mut Fighter, a1: i32, a2: *mut Fighter) -> i32 { unsafe { crate::tu::geno_game::Geno_HitFlags(a0, a1, a2) } }
pub fn Geno_TakeStunBonus__1e312f(a0: *mut Fighter) -> i32 { unsafe { crate::tu::geno_game::Geno_TakeStunBonus(a0) } }
pub fn Geno_HitStunBonus__ff0e91(a0: *mut Fighter, a1: i32, a2: *mut Fighter) { unsafe { crate::tu::geno_game::Geno_HitStunBonus(a0, a1, a2) } }
pub fn ft_did_jump__3fe6b7(a0: *mut Fighter, a1: i32) -> i32 { unsafe { (melee_fighter::tu::ftCo_JumpAerial::ft_did_jump(&mut *a0, a1 != 0)) as _ } }
pub fn ftAnim_8006FA58__8be30e(a0: *mut Fighter, a1: Fighter_Part, a2: *mut HSD_Joint) { unsafe { melee_fighter::tu::ftanim::ftAnim_8006FA58(&mut *a0, a1 as _, a2) } }
pub fn HSD_FObjInitAllocData__fad4fb() { unsafe { melee_runtime::tu::fobj::HSD_FObjInitAllocData() } }
pub fn HSD_AObjInitAllocData__a8b9a9() { unsafe { melee_runtime::tu::aobj::HSD_AObjInitAllocData() } }
pub fn HSD_JObjLoadJoint__3efee1(a0: *mut HSD_Joint) -> *mut HSD_JObj { unsafe { melee_runtime::tu::jobj::HSD_JObjLoadJoint(a0) } }
pub fn GenoGame_AttrFind__5b64ad(a0: *const i8) -> i32 { unsafe { crate::tu::geno_game::GenoGame_AttrFind(a0) } }
pub fn GenoGame_AttrOffset__be1e14(a0: i32) -> i32 { unsafe { crate::tu::geno_game::GenoGame_AttrOffset(a0) } }
pub fn GenoGame_HookFind__5430c1(a0: *const i8) -> i32 { unsafe { crate::tu::geno_game::GenoGame_HookFind(a0) } }
pub fn GenoGame_StateOf__61c016(a0: *mut Fighter) -> *mut c_void { unsafe { crate::tu::geno_game::GenoGame_StateOf(a0) } }
pub fn Geno_FighterReset__ca0a88(a0: *mut Fighter) { unsafe { crate::tu::geno_game::Geno_FighterReset(a0) } }
pub fn Geno_OnActionChange__e5d2dc(a0: *mut Fighter_GObj) { unsafe { crate::tu::geno_game::Geno_OnActionChange(a0) } }
pub fn Geno_ApplyAttrs__5e5d4e(a0: *mut Fighter) { unsafe { crate::tu::geno_game::Geno_ApplyAttrs(a0) } }
pub fn Geno_MultiJump__def9ad(a0: *mut Fighter, a1: i32, a2: *mut i32, a3: *mut f32) { unsafe { crate::tu::geno_game::Geno_MultiJump(a0, a1, a2, a3) } }
pub fn GenoGame_ScriptRange__051719(a0: u32, a1: u32) -> i32 { unsafe { crate::tu::geno_game::GenoGame_ScriptRange(a0, a1) } }
pub fn GenoGame_ScriptHasIasa__c15a1a(a0: *const c_void) -> i32 { unsafe { crate::tu::geno_game::GenoGame_ScriptHasIasa(a0) } }
pub fn GenoGame_AttrIsInt__1d3880(a0: i32) -> i32 { unsafe { crate::tu::geno_game::GenoGame_AttrIsInt(a0) } }
pub fn GenoGame_TestCapture__2dadd3(a0: i32, a1: i32) { unsafe { crate::tu::geno_game::GenoGame_TestCapture(a0, a1) } }
pub fn GenoGame_TestChanges__755eb6() -> i32 { unsafe { crate::tu::geno_game::GenoGame_TestChanges() } }
pub fn GenoGame_TestLastTarget__2835dc() -> u32 { unsafe { crate::tu::geno_game::GenoGame_TestLastTarget() } }
pub fn Geno_PreAnim__3c70fb(a0: *mut Fighter_GObj) -> i32 { unsafe { crate::tu::geno_game::Geno_PreAnim(a0) } }
pub fn Geno_CollBegin__155603(a0: *mut Fighter_GObj) { unsafe { crate::tu::geno_game::Geno_CollBegin(a0) } }
pub fn Geno_CollEnd__6ea115(a0: *mut Fighter_GObj) { unsafe { crate::tu::geno_game::Geno_CollEnd(a0) } }
pub fn Geno_GroundEdge__246052(a0: *mut Fighter, a1: i32) { unsafe { crate::tu::geno_game::Geno_GroundEdge(a0, a1) } }
pub fn Geno_Autolink__bd7888(a0: *mut Fighter, a1: *mut HitCapsule, a2: *mut f32, a3: *mut f32, a4: *mut f32) -> i32 { unsafe { crate::tu::geno_game::Geno_Autolink(a0, a1, a2, a3, a4) } }
pub fn Geno_MotionRow__2d0cf2(a0: *mut Fighter, a1: i32) -> *mut MotionState { unsafe { crate::tu::geno_game::Geno_MotionRow(a0, a1) } }
pub fn Geno_SpecialEnter__354edd(a0: *mut Fighter_GObj, a1: i32) -> i32 { unsafe { crate::tu::geno_game::Geno_SpecialEnter(a0, a1) } }
pub fn GenoGame_TestLedgeMode__20e1e5(a0: *mut Fighter) -> i32 { unsafe { crate::tu::geno_game::GenoGame_TestLedgeMode(a0) } }
pub fn GenoGame_TestStartFrame__314818(a0: *mut Fighter, a1: i32) -> f32 { unsafe { crate::tu::geno_game::GenoGame_TestStartFrame(a0, a1) } }
pub fn GenoGame_TestPairSwap__f2efa3(a0: *mut Fighter, a1: i32) -> i32 { unsafe { crate::tu::geno_game::GenoGame_TestPairSwap(a0, a1) } }
pub fn GenoGame_TestModelRotX__9f9136() -> f32 { unsafe { crate::tu::geno_game::GenoGame_TestModelRotX() } }
pub fn GenoGame_TestJointRate__51ea58() -> f32 { unsafe { crate::tu::geno_game::GenoGame_TestJointRate() } }
pub fn GenoGame_TestPoseReset__cef785(a0: f32) { unsafe { crate::tu::geno_game::GenoGame_TestPoseReset(a0) } }
pub fn ScriptGame_FighterI__fafc3a(a0: i32, a1: i32) -> i32 { unsafe { crate::tu::script_game::ScriptGame_FighterI(a0, a1) } }
pub fn ScriptGame_FighterF__01b474(a0: i32, a1: i32) -> f32 { unsafe { crate::tu::script_game::ScriptGame_FighterF(a0, a1) } }
pub fn gmFrontend_ModeSelect__ce9fbe(a0: *mut GameModeState, a1: i32, a2: *const i8) { unsafe { crate::tu::gmfrontend::gmFrontend_ModeSelect(a0, a1, a2) } }
pub fn gm_GetAllGameModes__34c35a() -> *mut GameMode { unsafe { crate::tu::gmscdata::gm_GetAllGameModes() } }
pub fn gm_GetStartMeleeRules__b73473() -> *mut StartMeleeRules { unsafe { crate::tu::gmvs::gm_GetStartMeleeRules() } }
pub fn gm_8016AE80__c0d7dc() -> i32 { unsafe { crate::tu::gmvs::gm_8016AE80() } }
pub fn it_8026D018__64d115() { unsafe { melee_items::tu::itspawn::it_8026D018() } }
pub fn it_8026D324__f6da35(a0: ItemKind) -> i32 { unsafe { (melee_items::tu::itspawn::it_8026D324(a0)) as _ } }
pub fn gm_Mode_Lab_OnLoad__64dc69() { unsafe { crate::tu::geno_lab_mode::gm_Mode_Lab_OnLoad() } }
pub fn GenoLab_ModeActive__8cb96b() -> i32 { unsafe { crate::tu::geno_lab_mode::GenoLab_ModeActive() } }
pub fn GenoLab_Leave__a89637(a0: i32) -> i32 { unsafe { crate::tu::geno_lab_mode::GenoLab_Leave(a0) } }
pub fn GenoLab_ApplyRules__92fa88(a0: *mut StartMeleeData) { unsafe { crate::tu::geno_lab_mode::GenoLab_ApplyRules(a0) } }
pub fn GenoLab_NextAfterMatch__3887c8() -> i32 { unsafe { crate::tu::geno_lab_mode::GenoLab_NextAfterMatch() } }
pub fn GenoFly_SetSpeed__aabcc6(a0: i32) { unsafe { crate::tu::geno_lab_mode::GenoFly_SetSpeed(a0) } }
pub fn GenoFly_Speed__366efc() -> f32 { unsafe { crate::tu::geno_lab_mode::GenoFly_Speed() } }
pub fn GenoFly_SetSolid__b04d6c(a0: i32) { unsafe { crate::tu::geno_lab_mode::GenoFly_SetSolid(a0) } }
pub fn GenoGame_StateSize__92165c() -> i32 { unsafe { crate::tu::geno_game::GenoGame_StateSize() } }
pub fn GenoGame_StateFieldName__96915a(a0: i32) -> *const i8 { unsafe { crate::tu::geno_game::GenoGame_StateFieldName(a0) } }
pub fn GenoGame_StateFieldBase__6bd4a2(a0: i32) -> i32 { unsafe { crate::tu::geno_game::GenoGame_StateFieldBase(a0) } }
pub fn ScriptGame_LabFighterFieldName__9ca0ff(a0: i32) -> *const i8 { unsafe { (crate::tu::script_game::ScriptGame_LabFighterFieldName(a0)) as _ } }
pub fn ScriptGame_LabFighterFieldBase__37c01f(a0: i32) -> i32 { unsafe { crate::tu::script_game::ScriptGame_LabFighterFieldBase(a0) } }
pub fn Geno_ArticleDesc__533f86(a0: i32) -> *mut c_void { unsafe { crate::tu::geno_game::Geno_ArticleDesc(a0) } }
pub fn Geno_ArticleLogic__7424dd(a0: i32) -> *mut c_void { unsafe { crate::tu::geno_game::Geno_ArticleLogic(a0) } }
pub fn Geno_OnHit__5cbd6a(a0: *mut Fighter_GObj) -> i32 { unsafe { crate::tu::geno_game::Geno_OnHit(a0) } }
pub fn GenoGame_ArticleStep__051bee(a0: i32, a1: i32, a2: i32, a3: *mut f32, a4: *mut f32, a5: f32, a6: f32, a7: i32, a8: f32, a9: f32) { unsafe { crate::tu::geno_game::GenoGame_ArticleStep(a0, a1, a2, a3, a4, a5, a6, a7, a8, a9) } }
pub fn GenoGame_DashStick__2b0e6e(a0: f32, a1: f32, a2: f32, a3: *mut f32, a4: *mut f32) -> i32 { unsafe { crate::tu::geno_game::GenoGame_DashStick(a0, a1, a2, a3, a4) } }
pub fn GenoGame_DashAim__eaca5a(a0: i32, a1: f32, a2: f32, a3: i32, a4: f32, a5: f32, a6: f32, a7: f32, a8: f32, a9: f32, a10: *mut f32, a11: *mut f32, a12: *mut f32, a13: *mut i32) { unsafe { crate::tu::geno_game::GenoGame_DashAim(a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13) } }
pub fn GenoGame_Brake__13e65e(a0: *mut f32, a1: *mut f32, a2: f32, a3: f32) { unsafe { crate::tu::geno_game::GenoGame_Brake(a0, a1, a2, a3) } }
pub fn GenoGame_LockonAim__57d893(a0: i32, a1: f32, a2: f32, a3: f32, a4: f32, a5: f32, a6: i32, a7: f32, a8: f32, a9: *mut f32, a10: *mut f32, a11: *mut f32) { unsafe { crate::tu::geno_game::GenoGame_LockonAim(a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11) } }
pub fn GenoFly_TestArm__11059c(a0: *mut Fighter) { unsafe { crate::tu::geno_lab_mode::GenoFly_TestArm(a0) } }
pub fn GenoFly_Fighter__f7b536(a0: *mut Fighter) -> i32 { unsafe { crate::tu::geno_lab_mode::GenoFly_Fighter(a0) } }
pub fn ScriptGame_HitFamilyMove__553e52(a0: *mut Fighter) -> i32 { unsafe { crate::tu::script_game::ScriptGame_HitFamilyMove(a0) } }
pub fn GenoDefine_StateDigest__f6fea2(a0: *mut Fighter) -> u32 { unsafe { crate::tu::geno_game::GenoDefine_StateDigest(a0) } }
pub fn RB_GameHash__00ac4f() -> u32 { unsafe { melee_fighter::tu::fighter::RB_GameHash() } }
pub fn RB_FighterHash__1628df(a0: u32, a1: *mut Fighter, a2: i32) -> u32 { unsafe { melee_fighter::tu::fighter::RB_FighterHash(a0, &mut *a1, a2) } }
pub fn RB_HashSetLegacy__8bf080(a0: i32) { unsafe { melee_fighter::tu::fighter::RB_HashSetLegacy(a0) } }
pub fn GenoGame_LuaBlockOf__b52deb(a0: *mut Fighter) -> *mut c_void { unsafe { crate::tu::geno_game::GenoGame_LuaBlockOf(a0) } }
pub fn GenoGame_ComboBlockOf__3da5c9(a0: *mut Fighter) -> *mut c_void { unsafe { crate::tu::geno_game::GenoGame_ComboBlockOf(a0) } }
pub fn Geno_ComboInput__b10783(a0: *mut Fighter_GObj) { unsafe { crate::tu::geno_game::Geno_ComboInput(a0) } }
pub fn Geno_ComboTick__d39f28(a0: *mut Fighter_GObj) -> i32 { unsafe { crate::tu::geno_game::Geno_ComboTick(a0) } }
pub fn Geno_ComboContact__369dfe(a0: *mut Fighter, a1: i32) { unsafe { crate::tu::geno_game::Geno_ComboContact(a0, a1) } }
pub fn lbColl_80007BCC__611461(a0: *mut HitCapsule, a1: *mut HitResult, a2: *mut c_void, a3: i32, a4: f32, a5: f32, a6: f32) -> i32 { unsafe { (melee_runtime::tu::lbcollision::lbColl_80007BCC(a0, a1, a2 as _, a3, a4, a5, a6)) as _ } }
pub fn HSD_GetHeap__32acf8() -> OSHeapHandle { unsafe { melee_runtime::tu::initialize::HSD_GetHeap() } }
pub fn HSD_SetHeap__744bee(a0: OSHeapHandle) { unsafe { melee_runtime::tu::initialize::HSD_SetHeap(a0) } }
pub fn HSD_IDInitAllocData__7a483c() { unsafe { melee_runtime::tu::id::HSD_IDInitAllocData() } }
pub fn HSD_IDSetup__f19db5() { unsafe { melee_runtime::tu::id::HSD_IDSetup() } }
pub fn HSD_VecInitAllocData__8c88b8() { unsafe { melee_runtime::tu::mtx::HSD_VecInitAllocData() } }
pub fn HSD_MtxInitAllocData__245d52() { unsafe { melee_runtime::tu::mtx::HSD_MtxInitAllocData() } }
pub fn HSD_SisLib_803A611C__de78e8(a0: i32, a1: *mut HSD_GObj, a2: u16, a3: u8, a4: u8, a5: u8, a6: u8, a7: u32) -> i32 { unsafe { melee_runtime::tu::sislib::HSD_SisLib_803A611C(a0, a1, a2, a3, a4, a5, a6, a7) } }
pub fn gmCamera_801A3098__ec0ef3() { unsafe { crate::tu::gmcamera::gmCamera_801A3098() } }
pub fn gmCamera_801A30E4__21b227() { unsafe { crate::tu::gmcamera::gmCamera_801A30E4() } }
pub fn gmCamera_801A31FC__fb65a6() { unsafe { crate::tu::gmcamera::gmCamera_801A31FC() } }
pub fn gmFrontend_AtlasSelect__26b7fc(a0: *mut GameModeState, a1: i32, a2: *const i8) { unsafe { crate::tu::gmfrontend::gmFrontend_AtlasSelect(a0, a1, a2) } }
pub fn gmVsMelee_GetKOCounts__f55015() -> *mut u8 { unsafe { crate::tu::gmvsmelee::gmVsMelee_GetKOCounts() } }
pub fn gm_8016247C__6fb2cc(a0: i32) -> i32 { unsafe { crate::tu::gm_1601::gm_8016247C(a0) } }
pub fn gm_80165268__fb784b(a0: i32) { unsafe { crate::tu::gm_1601::gm_80165268(a0) } }
pub fn gm_80167BC8__ea74bf(a0: *mut VsModeData) { unsafe { crate::tu::gm_1601::gm_80167BC8(a0) } }
pub fn gm_80167FC4__c413c9(a0: *mut SSSData) { unsafe { crate::tu::gm_1601::gm_80167FC4(a0) } }
pub fn gm_80168638__516aa9(a0: *mut MatchEnd) { unsafe { crate::tu::gm_1601::gm_80168638(a0) } }
pub fn gm_80168710__2df7b9(a0: *mut MatchEnd, a1: *mut VsModeData) { unsafe { crate::tu::gm_1601::gm_80168710(a0, a1) } }
pub fn gm_801688AC__8d2545(a0: *mut MatchEnd) -> i32 { unsafe { crate::gm_r10::gm_801688AC(&*a0) } }
pub fn gm_CameraModeVSGetPauser__ac5b00() -> i32 { unsafe { crate::tu::gmvs::gm_CameraModeVSGetPauser() } }
pub fn gm_ChangeGameModeAfterCurrentScene__2292c1(a0: i32) { unsafe { crate::tu::gm_1A3F::gm_ChangeGameModeAfterCurrentScene(a0) } }
pub fn gm_GetGameModeStateEnterData__d31bdf(a0: *mut GameModeState) -> *mut c_void { unsafe { crate::tu::gm_1A3F::gm_GetGameModeStateEnterData(a0) } }
pub fn gm_GetGameModeStateExitData__c3c609(a0: *mut GameModeState) -> *mut c_void { unsafe { crate::tu::gm_1A3F::gm_GetGameModeStateExitData(a0) } }
pub fn gm_InitVsMode__c4ead2(a0: *mut VsModeData) { unsafe { crate::tu::gm_1601::gm_InitVsMode(a0) } }
pub fn gm_LoadAnnouncer__907f2e() { unsafe { crate::tu::gm_1601::gm_LoadAnnouncer() } }
pub fn gm_LoadRumbleEnabled__62a06a(a0: *mut StartMeleeData) { unsafe { crate::tu::gmvs::gm_LoadRumbleEnabled(a0) } }
pub fn gm_SetNextGameModeStateId__031e98(a0: u8) { unsafe { crate::tu::gm_1A3F::gm_SetNextGameModeStateId(a0) } }
pub fn gm_SetupSubColors__761278(a0: *mut StartMeleeData) { unsafe { crate::tu::gm_1B03::gm_SetupSubColors(&mut *a0) } }
pub fn lbAudioAx_80026E84__5dcc27(a0: u32) -> u64 { unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80026E84(a0 as _) } }
pub fn lbAudioAx_80026EBC__431bc0(a0: u32) -> u64 { unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80026EBC(a0) } }
pub fn lbAudioAx_80026F2C__c80d0b(a0: u32) { unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80026F2C(a0) } }
pub fn lbAudioAx_8002702C__208dc3(a0: u32, a1: u64) { unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_8002702C(a0, a1) } }
pub fn lbAudioAx_80027168__342616() { unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80027168() } }
pub fn lbCardNew_AllocWorkArea__a2c203() { unsafe { melee_runtime::tu::lbcardnew::lbCardNew_AllocWorkArea() } }
pub fn lbDvd_80017740__447a8d(a0: i32, a1: i32, a2: i32, a3: i32, a4: u32, a5: i32, a6: i32, a7: u8, a8: i32) -> *mut c_void { unsafe { melee_runtime::tu::lbdvd::lbDvd_80017740(a0, a1, a2, a3, a4, a5, a6, a7, a8) } }
pub fn lbDvd_80018254__6be579() { unsafe { melee_runtime::tu::lbdvd::lbDvd_80018254() } }
pub fn lbDvd_GetPreloadCacheScene__1ed990() -> *mut PreloadedGameModeState { unsafe { melee_runtime::tu::lbdvd::lbDvd_GetPreloadCacheScene() } }
pub fn lbDvd_GetPreloadedArchive__a74d59(a0: i32) -> *mut c_void { unsafe { melee_runtime::tu::lbdvd::lbDvd_GetPreloadedArchive(a0 as _) } }
pub fn lbDvd_SetupVsPreloadCache__d38cf3() { unsafe { melee_runtime::tu::lbdvd::lbDvd_SetupVsPreloadCache() } }
pub fn lbSnap_8001E204__c45919() -> i32 { unsafe { crate::lb::lbSnap_8001E204() } }
pub fn lbSnap_8001E210__8a0d85() -> i32 { unsafe { crate::lb::lbSnap_8001E210() } }
pub fn lbSnap_8001E218__ac2c91(a0: *mut c_void, a1: *mut c_void /* Unk80433380_48 */) { unsafe { melee_runtime::tu::lbsnap::lbSnap_8001E218(a0, a1 as _) } }
pub fn Ground_801C1DE4__3e6526(a0: *mut i32, a1: *mut i32) { unsafe { melee_world::tu::ground::Ground_801C1DE4(a0, a1) } }
pub fn Ground_801C5A28__62c4d3() { unsafe { melee_world::tu::ground::Ground_801C5A28() } }
pub fn gmMainLib_8015CDC8__ddba5a() -> *mut gmm_x0_528_t { unsafe { crate::tu::gmmain_lib::gmMainLib_8015CDC8() } }
pub fn gmMainLib_8015D438__7caf41(a0: u8) -> *mut u32 { unsafe { crate::tu::gmmain_lib::gmMainLib_8015D438(a0) } }
pub fn gmMainLib_8015D450__dabb77(a0: u8) -> *mut i32 { unsafe { crate::tu::gmmain_lib::gmMainLib_8015D450(a0) } }
pub fn gmMainLib_8015D48C__ede167(a0: u8) -> i32 { unsafe { (crate::tu::gmmain_lib::gmMainLib_8015D48C(a0)) as _ } }
pub fn gmMainLib_8015D4E8__f34d9f(a0: u8, a1: i32) { unsafe { crate::tu::gmmain_lib::gmMainLib_8015D4E8(a0, a1) } }
pub fn gmMainLib_8015ED98__51617a() -> *mut gmm_retval_ED98 { unsafe { crate::tu::gmmain_lib::gmMainLib_8015ED98() } }
pub fn gmMainLib_8015EDB0__52d907() -> *mut gmm_retval_EDB0 { unsafe { crate::tu::gmmain_lib::gmMainLib_8015EDB0() } }
pub fn gmMainLib_8015EDBC__bd0e2b() -> *mut gmm_retval_EDBC { unsafe { crate::tu::gmmain_lib::gmMainLib_8015EDBC() } }
pub fn gmRegSetupEnemyColorTable__a8a833(a0: i8, a1: u8, a2: *const i8, a3: *mut u8) { unsafe { crate::tu::gmregcommon::gmRegSetupEnemyColorTable(a0, a1, a2, a3) } }
pub fn gm_80164430__8808be(a0: u16) -> i32 { unsafe { (crate::tu::gm_1601::gm_80164430(a0)) as _ } }
pub fn gm_801647F8__46ed88(a0: u8) -> i32 { unsafe { crate::tu::gm_1601::gm_801647F8(a0) } }
pub fn gm_801677F0__be41b7() -> u8 { unsafe { crate::tu::gm_1601::gm_801677F0() } }
pub fn gm_80168F88__9d899a() { unsafe { crate::tu::gm_1601::gm_80168F88() } }
pub fn gm_80172174__0e08c4() { unsafe { crate::tu::gm_16f1::gm_80172174() } }
pub fn gm_8017BE84__49c20d(a0: u32) -> i32 { unsafe { crate::gm::gm_8017BE84(a0) } }
pub fn gm_8017C9A8__94d55d(a0: *mut DebugGameOverData, a1: *mut Unk1PData, a2: u8) { unsafe { crate::tu::gm_17c0::gm_8017C9A8(a0, a1, a2) } }
pub fn gm_8017CA38__442d2c(a0: *mut DebugGameOverData, a1: *mut Unk1PData, a2: *mut gmm_x0_528_t, a3: u8) { unsafe { crate::tu::gm_17c0::gm_8017CA38(a0, a1, a2, a3) } }
pub fn gm_8017CBAC__33592d(a0: *mut UnkAdventureData, a1: *mut gmm_x0_528_t, a2: u8) { unsafe { crate::tu::gm_17c0::gm_8017CBAC(a0, a1, a2) } }
pub fn gm_8017CD94__09d838(a0: *mut UnkAdventureData, a1: i32, a2: i32, a3: i32) -> u8 { unsafe { crate::tu::gm_17c0::gm_8017CD94(a0, a1, a2, a3) } }
pub fn gm_8017D7AC__84cd0d(a0: *mut MatchExitInfo, a1: *mut Unk1PData, a2: u8) -> i32 { unsafe { crate::tu::gm_17c0::gm_8017D7AC(a0, a1, a2) } }
pub fn gm_8017DB88__2129f0(a0: *mut c_void, a1: u8, a2: i32, a3: i32, a4: *mut u8, a5: u8, a6: Option<unsafe extern "C" fn(i32, i32, u8) -> u8>, a7: Option<unsafe extern "C" fn(i32, i32, u8) -> u8>, a8: Option<unsafe extern "C" fn(i32, i32, u8) -> u8>, a9: Option<unsafe extern "C" fn(i32, i32) -> f32>, a10: Option<unsafe extern "C" fn(i32, i32) -> f32>) -> i32 { unsafe { crate::tu::gm_17c0::gm_8017DB88(a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10) } }
pub fn gm_8017EB3C__28a3ff(a0: u8, a1: u8) -> u8 { unsafe { crate::tu::gm_17eb::gm_8017EB3C(a0, a1) } }
pub fn gm_8017EB64__39144b(a0: u8, a1: u8, a2: u8) -> u8 { unsafe { crate::tu::gm_17eb::gm_8017EB64(a0, a1, a2) } }
pub fn gm_8017EB98__c0f6a1(a0: u8, a1: u8, a2: u8) -> u8 { unsafe { crate::tu::gm_17eb::gm_8017EB98(a0, a1, a2) } }
pub fn gm_8017EBCC__415a10(a0: u8, a1: u8, a2: u8) -> u8 { unsafe { crate::tu::gm_17eb::gm_8017EBCC(a0, a1, a2) } }
pub fn gm_8017EC00__0d00a7(a0: u8, a1: u8) -> f32 { unsafe { crate::tu::gm_17eb::gm_8017EC00(a0, a1) } }
pub fn gm_8017EC50__df88ed(a0: u8, a1: u8) -> f32 { unsafe { crate::tu::gm_17eb::gm_8017EC50(a0, a1) } }
pub fn gm_8017ECA0__c89fbd(a0: u8, a1: u8, a2: u8) -> u8 { unsafe { crate::tu::gm_17eb::gm_8017ECA0(a0, a1, a2) } }
pub fn gm_8017ECD4__e2693d(a0: u8, a1: u8, a2: u8) -> u8 { unsafe { crate::tu::gm_17eb::gm_8017ECD4(a0, a1, a2) } }
pub fn gm_8017ED08__4f54e0(a0: u8, a1: u8, a2: u8) -> u8 { unsafe { crate::tu::gm_17eb::gm_8017ED08(a0, a1, a2) } }
pub fn gm_8017ED3C__2915b5(a0: u8, a1: u8) -> f32 { unsafe { crate::tu::gm_17eb::gm_8017ED3C(a0, a1) } }
pub fn gm_8017ED8C__49f083(a0: u8, a1: u8) -> f32 { unsafe { crate::tu::gm_17eb::gm_8017ED8C(a0, a1) } }
pub fn gm_801B06B0__3a08bf(a0: *mut CSSData, a1: u8, a2: i8, a3: i8, a4: i8, a5: u8, a6: u8, a7: u8) { unsafe { crate::tu::gm_1B03::gm_801B06B0(&mut *a0, a1, a2, a3, a4, a5, a6, a7) } }
pub fn gm_CKindToSelKind__a9d153(a0: u8) -> u8 { unsafe { crate::gm::gm_CKindToSelKind(a0) } }
pub fn gm_GetAllStarData__0c7e4e() -> *mut UnkAllstarData { unsafe { crate::tu::gm_17eb::gm_GetAllStarData() } }
pub fn gm_IsCKindUnlocked__b6c64b(a0: u8) -> i32 { unsafe { (crate::tu::gm_1601::gm_IsCKindUnlocked(a0)) as _ } }
pub fn gm_SetGameModeStateId__147996(a0: u8) { unsafe { crate::tu::gm_1A3F::gm_SetGameModeStateId(a0) } }
pub fn gm_SetNewGameModePending__1e5e48() { unsafe { crate::tu::gm_1A3F::gm_SetNewGameModePending() } }
pub fn gm_SetPendingGameMode__90d26d(a0: u8) { unsafe { crate::tu::gm_1A3F::gm_SetPendingGameMode(a0) } }
pub fn lbDvd_80017700__5067b6(a0: i32) { unsafe { melee_runtime::tu::lbdvd::lbDvd_80017700(a0) } }
pub fn lbDvd_80018C2C__30bf87(a0: u8) { unsafe { melee_runtime::tu::lbdvd::lbDvd_80018C2C(a0) } }
pub fn lbDvd_80018C6C__4e040c() { unsafe { melee_runtime::tu::lbdvd::lbDvd_80018C6C() } }
pub fn gm_80166A98__8c24a8(a0: *mut MatchEnd, a1: u8, a2: i8, a3: u8, a4: i8, a5: u8, a6: i8, a7: u8, a8: u8, a9: u8) -> i32 { unsafe { crate::tu::gm_1601::gm_80166A98(a0, a1, a2, a3, a4, a5, a6, a7, a8, a9) } }
pub fn gm_801701A0__12d021() { unsafe { crate::tu::gm_16f1::gm_801701A0() } }
pub fn gm_80177724__c19fac(a0: *mut ResultsMatchInfo) { unsafe { crate::gm_r7::gm_80177724(&mut *a0) } }
pub fn gm_801A4B60__d469d9() { unsafe { crate::tu::gmscene::gm_801A4B60() } }
pub fn lbAudioAx_800236DC__5d9f3e() -> i32 { unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_800236DC() } }
pub fn lbAudioAx_80027648__4f40c8() { unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80027648() } }
pub fn un_802FFEE0__17ec63(a0: *mut c_void /* UnkSoundTestData0 */) { unsafe { melee_world::tu::soundtest::un_802FFEE0(a0 as _) } }
pub fn un_802FFF2C__73c528(a0: *mut StartMeleeData) { unsafe { melee_world::tu::soundtest::un_802FFF2C(a0) } }
pub fn un_80301BA8__89da53(a0: *mut c_void /* UnkSoundTestData1 */) { unsafe { melee_world::tu::soundtest::un_80301BA8(a0 as _) } }
pub fn un_80301C64__b851f6(a0: *mut un_80301C64_t) { unsafe { melee_world::tu::soundtest::un_80301C64(a0) } }
pub fn Ground_801C1D84__af7526() -> u32 { unsafe { melee_world::tu::ground::Ground_801C1D84() } }
pub fn Player_80031790__11821c(a0: i32) { unsafe { melee_fighter::player::player_80031790(a0) } }
pub fn Player_80031DA8__fbecc7(a0: i32, a1: i32) { unsafe { melee_fighter::player::player_80031da8(a0, a1) } }
pub fn Player_80036394__f86559(a0: i32) -> u32 { unsafe { (melee_fighter::player::player_80036394(a0)) as _ } }
pub fn Player_80036844__2d424f(a0: i32, a1: i32) { unsafe { melee_fighter::player::player_80036844(a0, a1) } }
pub fn Player_GetCoins__c7d98f(a0: i32) -> i32 { unsafe { melee_fighter::tu::player::Player_GetCoins(a0) } }
pub fn Player_GetEntityAtIndex__85ed4e(a0: i32, a1: i32) -> *mut HSD_GObj { unsafe { melee_fighter::tu::player::Player_GetEntityAtIndex(a0, a1) } }
pub fn Player_GetKOsByPlayerIndex__bd5a79(a0: i32, a1: i32) -> i32 { unsafe { melee_fighter::tu::player::Player_GetKOsByPlayerIndex(a0, a1) } }
pub fn Player_GetNametagSlotID__655434(a0: i32) -> u8 { unsafe { melee_fighter::tu::player::Player_GetNametagSlotID(a0) } }
pub fn Player_GetP1Stock__f5a98b() -> i32 { unsafe { melee_fighter::tu::player::Player_GetP1Stock() } }
pub fn Player_GetPlayerCharacter__fdeb58(a0: i32) -> u32 { unsafe { (melee_fighter::tu::player::Player_GetPlayerCharacter(a0)) as _ } }
pub fn Player_GetPlayerSlotType__2d05e5(a0: i32) -> Gm_PKind { unsafe { melee_fighter::tu::player::Player_GetPlayerSlotType(a0) } }
pub fn Player_GetRemainingHP__d5e70f(a0: i32) -> i32 { unsafe { melee_fighter::tu::player::Player_GetRemainingHP(a0) } }
pub fn Player_GetStocks__1c1d51(a0: i32) -> i32 { unsafe { melee_fighter::tu::player::Player_GetStocks(a0) } }
pub fn Player_SetModelScale__1d6c7a(a0: i32, a1: f32) { unsafe { melee_fighter::tu::player::Player_SetModelScale(a0, a1) } }
pub fn Player_SetPlayerAndEntityCpuType__534529(a0: i32, a1: i32) { unsafe { melee_fighter::tu::player::Player_SetPlayerAndEntityCpuType(a0, a1) } }
pub fn Player_SetStocks__1270f2(a0: i32, a1: i32) { unsafe { melee_fighter::tu::player::Player_SetStocks(a0, a1) } }
pub fn ftBossLib_8015CB9C__c24da3(a0: i32) { unsafe { melee_fighter::tu::ftbosslib::ftBossLib_8015CB9C(a0) } }
pub fn ftBossLib_8015CC14__f4f13c() { unsafe { melee_fighter::tu::ftbosslib::ftBossLib_8015CC14() } }
pub fn ftLib_80087508__b45233(a0: i8, a1: u8) { unsafe { melee_fighter::tu::ftlib::ftlib_80087508(a0, a1) } }
pub fn gmMainLib_8015CEB4__86b231(a0: i32) { unsafe { crate::tu::gmmain_lib::gmMainLib_8015CEB4(a0) } }
pub fn gmMainLib_8015CF5C__8fd145(a0: i32) -> i32 { unsafe { crate::tu::gmmain_lib::gmMainLib_8015CF5C(a0) } }
pub fn gmMainLib_8015CF70__0b6cc3(a0: i32, a1: i32) { unsafe { crate::tu::gmmain_lib::gmMainLib_8015CF70(a0, a1) } }
pub fn gmMainLib_8015CF84__b2804a() { unsafe { crate::tu::gmmain_lib::gmMainLib_8015CF84() } }
pub fn gmVs_GetSceneController__400cec() -> *mut VsSceneController { unsafe { crate::tu::gmvs::gmVs_GetSceneController() } }
pub fn gmVs_GetSceneState__53d82c() -> *mut VsSceneState { unsafe { crate::tu::gmvs::gmVs_GetSceneState() } }
pub fn gm_80162968__8ae6df(a0: u32) -> i32 { unsafe { crate::tu::gm_1601::gm_80162968(a0) } }
pub fn gm_80164504__95f551(a0: u16) { unsafe { crate::tu::gm_1601::gm_80164504(a0) } }
pub fn gm_80166378__34fe0e(a0: *mut MatchEnd) { unsafe { crate::tu::gm_1601::gm_80166378(a0) } }
pub fn gm_80167858__a75f4a(a0: i32, a1: i32, a2: i32, a3: i32) { unsafe { crate::tu::gm_1601::gm_80167858(a0, a1, a2, a3) } }
pub fn gm_80168940__6c4a67(a0: *mut MatchEnd) -> i32 { unsafe { crate::gm_ptr::gm_80168940(&*a0) } }
pub fn gm_80169384__e6b9b1() -> i32 { unsafe { crate::tu::gm_16A2::gm_80169384() } }
pub fn gm_8016A164__b4c98b() { unsafe { crate::tu::gm_16A2::gm_8016A164() } }
pub fn gm_8016A21C__1e76b5(a0: *mut StartMeleeRules) { unsafe { crate::tu::gm_16A2::gm_8016A21C(a0) } }
pub fn gm_8016A22C__e5b47e(a0: i8, a1: i8, a2: i8, a3: u8, a4: u8, a5: u8, a6: i32, a7: i32, a8: u8, a9: u8, a10: u8, a11: i32, a12: i32, a13: i32, a14: i32, a15: i32, a16: i32, a17: f32, a18: f32) { unsafe { crate::tu::gm_16A2::gm_8016A22C(a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18) } }
pub fn gm_8016A414__f962ef(a0: f32) { unsafe { crate::tu::gm_16A2::gm_8016A414(a0) } }
pub fn gm_8016A424__d0ff6a(a0: i8) { unsafe { crate::tu::gm_16A2::gm_8016A424(a0) } }
pub fn gm_8016A434__76ff7e() { unsafe { crate::tu::gm_16A2::gm_8016A434() } }
pub fn gm_8016A92C__fe3fb1(a0: *mut StartMeleeRules) { unsafe { crate::tu::gm_16A2::gm_8016A92C(a0) } }
pub fn gm_8016A9E8__ea3c0b(a0: u8, a1: i8) -> i32 { unsafe { crate::tu::gm_16A2::gm_8016A9E8(a0, a1) } }
pub fn gm_8016AC44__284fb4(a0: i8, a1: i8) -> i32 { unsafe { (crate::tu::gm_16A2::gm_8016AC44(a0, a1)) as _ } }
pub fn gm_8016AEEC__6820bb() -> u32 { unsafe { crate::tu::gmvs::gm_8016AEEC() } }
pub fn gm_8016AEFC__dd4657() -> u16 { unsafe { crate::tu::gmvs::gm_8016AEFC() } }
pub fn gm_8016B328__315877() { unsafe { crate::tu::gmvs::gm_8016B328() } }
pub fn gm_8016B33C__7b92b1(a0: i32) { unsafe { crate::tu::gmvs::gm_8016B33C(a0) } }
pub fn gm_8016B350__3b3894(a0: i32) { unsafe { crate::tu::gmvs::gm_8016B350(a0) } }
pub fn gm_8016B364__eadb9b(a0: i32) { unsafe { crate::tu::gmvs::gm_8016B364(a0) } }
pub fn gm_8016B378__8395d5(a0: i8) { unsafe { crate::tu::gmvs::gm_8016B378(a0) } }
pub fn gm_8016B774__ba5c36() -> *mut MatchEnd { unsafe { crate::tu::gmvs::gm_8016B774() } }
pub fn gm_8016EDDC__aa6fea(a0: i32, a1: *mut PlayerInitData) -> i32 { unsafe { (crate::tu::gmvs::gm_8016EDDC(a0, a1)) as _ } }
pub fn gm_8016F00C__e9523d(a0: i32) { unsafe { crate::tu::gmvs::gm_8016F00C(a0) } }
pub fn gm_80172898__ee73a3(a0: u16) -> i32 { unsafe { crate::tu::gm_16f1::gm_80172898(a0) } }
pub fn gm_801732D8__45e05d(a0: u8) -> u8 { unsafe { crate::tu::gm_16f1::gm_801732D8(a0) } }
pub fn gm_8017335C__86aa87() -> u16 { unsafe { crate::tu::gm_16f1::gm_8017335C() } }
pub fn gm_80173754__f83272(a0: u8, a1: u8) -> i32 { unsafe { crate::tu::gm_1736::gm_80173754(a0, a1) } }
pub fn gm_80173D3C__7c55ab(a0: i32) { unsafe { crate::tu::gm_1736::gm_80173D3C(a0) } }
pub fn gm_80173EEC__58a22d() { unsafe { crate::tu::gm_1736::gm_80173EEC() } }
pub fn gm_801B05F4__ce3447(a0: *mut PlayerInitData, a1: i32) { unsafe { crate::gm_ptr::gm_801B05F4(&mut *a0, a1) } }
pub fn gm_ClearDbPauseFlag__57b7e0(a0: i32) { unsafe { crate::tu::gmscene::gm_ClearDbPauseFlag(a0) } }
pub fn gm_GetCurrentGameMode__f56b36() -> u8 { unsafe { crate::tu::gm_1A3F::gm_GetCurrentGameMode() } }
pub fn gm_GetFrameCount__aa8e1a() -> u32 { unsafe { crate::tu::gmvs::gm_GetFrameCount() } }
pub fn gm_InitChallengerData__c5db86(a0: u8, a1: u8, a2: u8, a3: u8, a4: u8, a5: u8) { unsafe { crate::tu::gm_1736::gm_InitChallengerData(a0, a1, a2, a3, a4, a5) } }
pub fn gm_RumbleEnabledForPlayer__aa5085(a0: i32, a1: i32) -> i32 { unsafe { (crate::tu::gm_1601::gm_RumbleEnabledForPlayer(a0, a1)) as _ } }
pub fn gm_SetDbPauseFlag__fad6e4(a0: i32) { unsafe { crate::tu::gmscene::gm_SetDbPauseFlag(a0) } }
pub fn gm_SetGameSpeed__e4fa3d(a0: f32) { unsafe { crate::tu::gmvs::gm_SetGameSpeed(a0) } }
pub fn gm_SetupPlayerDefaults__e9a9ae(a0: *mut PlayerInitData) { unsafe { crate::tu::gm_1601::gm_SetupPlayerDefaults(a0) } }
pub fn gm_SetupRulesDefaults__e9563a(a0: *mut StartMeleeRules) { unsafe { crate::tu::gm_1601::gm_SetupRulesDefaults(a0) } }
pub fn ifStock_802FA2D0__60b4c4(a0: i32) { unsafe { melee_world::tu::ifstock::ifStock_802FA2D0(a0) } }
pub fn itEvYoshiEgg_Spawn__b8d779(a0: *mut Vec) { unsafe { melee_items::tu::itevyoshiegg::itEvYoshiEgg_Spawn(a0) } }
pub fn lbAudioAx_8002438C__35c010(a0: i32) -> i32 { unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_8002438C(a0) } }
pub fn lbAudioAx_80028B6C__bb9f49() { unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80028B6C() } }
pub fn lbAudioAx_80028B90__0093c2() { unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80028B90() } }
pub fn lbBgFlash_8002063C__95303c(a0: i32) { unsafe { melee_runtime::tu::lbbgflash::lbBgFlash_8002063C(a0) } }
pub fn lbBgFlash_80020688__4d10b5(a0: i32) { unsafe { melee_runtime::tu::lbbgflash::lbBgFlash_80020688(a0) } }
pub fn lbBgFlash_SetFlashScale__f5c112(a0: f32) { unsafe { crate::tu::lb_0219::lbBgFlash_SetFlashScale(a0) } }
pub fn pl_8003FBFC__27089d(a0: i32) -> i32 { unsafe { melee_fighter::plbonus::pl_8003fbfc(a0) } }
pub fn pl_8003FC20__860736(a0: i32) -> i32 { unsafe { melee_fighter::plbonus::pl_8003fc20(a0) } }
pub fn gmVsMelee_EnterResults__e8d931(a0: *mut GameModeState) { unsafe { crate::tu::gmvsmelee::gmVsMelee_EnterResults(a0) } }
pub fn gmVsMelee_EnterSss__1e7fb2(a0: *mut GameModeState, a1: *mut VsModeData) { unsafe { crate::tu::gmvsmelee::gmVsMelee_EnterSss(a0, a1) } }
pub fn gmVsMelee_ExitCss__4d2768(a0: *mut GameModeState, a1: *mut VsModeData) { unsafe { crate::tu::gmvsmelee::gmVsMelee_ExitCss(a0, a1) } }
pub fn gmVsMelee_ExitResults__c5c250(a0: *mut GameModeState, a1: *mut VsModeData, a2: u8) { unsafe { crate::tu::gmvsmelee::gmVsMelee_ExitResults(a0, a1, a2) } }
pub fn gmVsMelee_ExitSss__2855c9(a0: *mut GameModeState, a1: *mut VsModeData, a2: u8) { unsafe { crate::tu::gmvsmelee::gmVsMelee_ExitSss(a0, a1, a2) } }
pub fn gmVsMelee_ExitSuddenDeath__a6d8ba(a0: *mut GameModeState) { unsafe { crate::tu::gmvsmelee::gmVsMelee_ExitSuddenDeath(a0) } }
pub fn gmVsMelee_ExitVs__083a0e(a0: *mut GameModeState, a1: u8, a2: u8) { unsafe { crate::tu::gmvsmelee::gmVsMelee_ExitVs(a0, a1, a2) } }
pub fn gmVsMelee_ResetKOCounts__75dda2() { unsafe { crate::tu::gmvsmelee::gmVsMelee_ResetKOCounts() } }
pub fn gm_80165290__e2462d(a0: i32) { unsafe { crate::tu::gm_1601::gm_80165290(a0) } }
pub fn gm_ModeState_ApproachVs_OnEnter__73c56c(a0: *mut GameModeState) { unsafe { crate::tu::gmvsmode::gm_ModeState_ApproachVs_OnEnter(a0) } }
pub fn gm_ModeState_ApproachVs_OnExit__ca1d07(a0: *mut GameModeState) { unsafe { crate::tu::gmmovieend::gm_ModeState_ApproachVs_OnExit(a0) } }
pub fn gm_ModeState_Approach_OnEnter__044765(a0: *mut GameModeState) { unsafe { crate::tu::gmvsmode::gm_ModeState_Approach_OnEnter(a0) } }
pub fn gm_ModeState_Prize_OnEnter__8623b9(a0: *mut GameModeState) { unsafe { crate::tu::gmvsmode::gm_ModeState_Prize_OnEnter(a0) } }
pub fn gm_ModeState_Prize_OnExit__253ddf(a0: *mut GameModeState) { unsafe { crate::tu::gmmovieend::gm_ModeState_Prize_OnExit(a0) } }
pub fn HSD_MemAlloc__c2b3bc(a0: i32) -> *mut c_void { unsafe { melee_runtime::tu::memory::HSD_MemAlloc(a0) } }
pub fn HSD_SisLib_803A5CC4__6c7e38(a0: *mut HSD_Text) { unsafe { melee_runtime::tu::sislib::HSD_SisLib_803A5CC4(a0) } }
pub fn HSD_SisLib_803A6754__02fb41(a0: i32, a1: i32) -> *mut HSD_Text { unsafe { melee_runtime::tu::hsd_3A64::HSD_SisLib_803A6754(a0, a1) } }
pub fn HSD_SisLib_SetWideCanvas__9bb4fb(a0: i32, a1: i32) { unsafe { melee_runtime::tu::sislib::HSD_SisLib_SetWideCanvas(a0, a1) } }
pub fn gmMainLib_GetGamePrefs__12f16c() -> *mut GamePrefs { unsafe { crate::tu::gmmain_lib::gmMainLib_GetGamePrefs() } }
pub fn gmMainLib_GetGameRules__34f6cb() -> *mut GameRules { unsafe { crate::tu::gmmain_lib::gmMainLib_GetGameRules() } }
pub fn lbCardGame_LoadArchive__e9acb0(a0: i32) { unsafe { melee_runtime::tu::lbcardgame::lbCardGame_LoadArchive(a0) } }
pub fn lbDvd_8001823C__d87971() { unsafe { melee_runtime::tu::lbdvd::lbDvd_8001823C() } }
pub fn gm_80173224__600622(a0: i32, a1: i32) -> u8 { unsafe { crate::tu::gm_16f1::gm_80173224(a0, a1) } }
pub fn gm_8017390C__4e0f3c(a0: i32, a1: i32) { unsafe { crate::tu::gm_1736::gm_8017390C(a0, a1) } }
pub fn lbMthp_8001F800__fe22a6() { unsafe { melee_runtime::tu::lbmthp::lbMthp_8001F800() } }
pub fn gmVsMelee_GetVsData__4d044b() -> *mut VsModeData { unsafe { crate::tu::gmvsmelee::gmVsMelee_GetVsData() } }
pub fn gm_80164A0C__5bef0b(a0: u8) { unsafe { crate::tu::gm_1601::gm_80164A0C(a0) } }
pub fn gm_80164F18__ff202e() { unsafe { crate::tu::gm_1601::gm_80164F18() } }
pub fn gm_801B06B0__7adbb8(a0: *mut CSSData, a1: i32, a2: i32, a3: i32, a4: i32, a5: i32, a6: i32, a7: i32) { unsafe { crate::tu::gm_1B03::gm_801B06B0(&mut *a0, a1 as _, a2 as _, a3 as _, a4 as _, a5 as _, a6 as _, a7 as _) } }
pub fn Ground_801C1DD4__f2fb49() -> i32 { unsafe { melee_world::tu::ground::Ground_801C1DD4() } }
pub fn HSD_AObjSetCurrentFrame__64f5bd(a0: *mut HSD_AObj, a1: f32) { unsafe { melee_runtime::tu::aobj::HSD_AObjSetCurrentFrame(a0, a1) } }
pub fn HSD_CObjLoadDesc__66a547(a0: *mut HSD_CObjDesc) -> *mut HSD_CObj { unsafe { melee_runtime::tu::cobj::HSD_CObjLoadDesc(a0) } }
pub fn HSD_GObj_JObjCallback__9cd907(a0: *mut HSD_GObj, a1: i32) { unsafe { melee_runtime::tu::gobj::HSD_GObj_JObjCallback(a0, a1) } }
pub fn HSD_GObj_LObjCallback__aaa236(a0: *mut HSD_GObj, a1: i32) { unsafe { melee_runtime::tu::gobj::HSD_GObj_LObjCallback(a0, a1) } }
pub fn HSD_SisLib_803A62A0__58f49d(a0: i32, a1: *const i8, a2: *const i8) { unsafe { melee_runtime::tu::sislib::HSD_SisLib_803A62A0(a0, a1 as _, a2 as _) } }
pub fn HSD_SisLib_803A7548__d5953f(a0: *mut HSD_Text, a1: i32, a2: f32, a3: f32) { unsafe { melee_runtime::tu::hsd_3A64::HSD_SisLib_803A7548(a0, a1, a2, a3) } }
pub fn Player_GetPlayerId__aaadbd(a0: i32) -> i32 { unsafe { melee_fighter::tu::player::Player_GetPlayerId(a0) } }
pub fn fn_801689E4__f8906a(a0: *mut HSD_JObj, a1: *mut DynamicModelDesc, a2: i32) { unsafe { crate::tu::gm_1601::fn_801689E4(a0, a1, a2) } }
pub fn fn_80168F2C__69f670(a0: i8) { unsafe { crate::tu::gm_1601::fn_80168F2C(a0) } }
pub fn fn_80168F7C__c9139a() { unsafe { crate::tu::gm_1601::fn_80168F7C() } }
pub fn fn_8016F140__cc064d(a0: i32) { unsafe { crate::tu::gmvs::fn_8016F140(a0) } }
pub fn fn_8016F160__106c7f(a0: i32, a1: i32) -> i32 { unsafe { crate::tu::gmvs::fn_8016F160(a0, a1) } }
pub fn fn_8016F180__e8a49d(a0: i32) -> i32 { unsafe { crate::tu::gm_16f1::fn_8016F180(a0) } }
pub fn fn_8016F1F0__e5a15c(a0: i32) -> i32 { unsafe { crate::tu::gm_16f1::fn_8016F1F0(a0) } }
pub fn fn_8016F344__d76861(a0: *mut MatchEnd) { unsafe { crate::tu::gm_16f1::fn_8016F344(a0) } }
pub fn fn_8016F39C__3085d5(a0: *mut *mut HSD_Text, a1: *mut c_void, a2: u8, a3: u16, a4: u8, a5: u8) -> i32 { unsafe { crate::tu::gm_16f1::fn_8016F39C(a0, a1 as _, a2, a3, a4, a5) } }
pub fn fn_8016F548__297f1e(a0: *mut c_void, a1: u16, a2: u8, a3: u8) -> i32 { unsafe { crate::tu::gm_16f1::fn_8016F548(a0 as _, a1, a2, a3) } }
pub fn fn_8016F740__946ab3(a0: *mut c_void, a1: u16, a2: u8, a3: u8) -> i32 { unsafe { crate::tu::gm_16f1::fn_8016F740(a0 as _, a1, a2, a3) } }
pub fn fn_8016F870__a28fe9(a0: *mut c_void, a1: u16, a2: u8, a3: u8) -> i32 { unsafe { crate::tu::gm_16f1::fn_8016F870(a0 as _, a1, a2, a3) } }
pub fn fn_8016F9A8__0f140d(a0: *mut c_void, a1: u16, a2: u8, a3: u8) -> i32 { unsafe { crate::tu::gm_16f1::fn_8016F9A8(a0 as _, a1, a2, a3) } }
pub fn fn_8016FAD4__4d2bbb(a0: *mut MatchEnd, a1: i32, a2: i32, a3: u8) -> i32 { unsafe { crate::tu::gm_16f1::fn_8016FAD4(a0, a1, a2, a3) } }
pub fn fn_8016FFD4__ba90b1(a0: *mut MatchEnd, a1: i32, a2: u8) -> i32 { unsafe { crate::tu::gm_16f1::fn_8016FFD4(a0, a1, a2) } }
pub fn fn_801701C0__332d21(a0: *mut MatchEnd, a1: i32, a2: i32) -> i32 { unsafe { crate::tu::gm_16f1::fn_801701C0(a0, a1, a2) } }
pub fn fn_80171A88__4c9145() -> i32 { unsafe { crate::tu::gm_16f1::fn_80171A88() } }
pub fn gm_801A36C0__4faf6f(a0: u8) -> u64 { unsafe { crate::tu::gm_1A36::gm_801A36C0(a0) } }
pub fn gm_GetButtonsTriggered__dca604(a0: u8) -> u64 { unsafe { crate::tu::gm_1A36::gm_GetButtonsTriggered(a0) } }
pub fn grPushOn_80219204__ac5d9d(a0: i32, a1: *mut i32, a2: *mut i32) { unsafe { melee_world::tu::grpushon::grPushOn_80219204(a0, a1, a2) } }
pub fn lbAudioAx_80023F28__582fc2(a0: i32) -> i32 { unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80023F28(a0) } }
pub fn lbLang_IsSavedLanguageUS__90e41e() -> i32 { unsafe { (melee_runtime::tu::lblanguage::lbLang_IsSavedLanguageUS()) as _ } }
pub fn lb_8000B1CC__69851d(a0: *mut HSD_JObj, a1: *mut Vec3, a2: *mut Vec3) { unsafe { melee_runtime::tu::lb_00B0::lb_8000B1CC(a0, a1 as _, a2 as _) } }
pub fn lb_80011AC4__8bc38c(a0: *mut *mut LightList) -> *mut HSD_LObj { unsafe { melee_runtime::tu::lbspdisplay::lb_80011AC4(a0) } }
pub fn lb_8001204C__7ca2e9(a0: *mut HSD_JObj, a1: *mut *mut HSD_JObj, a2: *const u16, a3: i32) -> i32 { unsafe { melee_runtime::tu::lbspdisplay::lb_8001204C(a0, a1, a2 as _, a3) } }
pub fn lb_800121FC__e48dc8(a0: *mut HSD_ImageDesc, a1: i32, a2: i32, a3: u32, a4: i16) -> *mut HSD_ImageDesc { unsafe { melee_runtime::tu::lbspdisplay::lb_800121FC(a0, a1, a2, a3, a4) } }
pub fn lb_800122C8__a924f6(a0: *mut HSD_ImageDesc, a1: u16, a2: u16, a3: i32) { unsafe { melee_runtime::tu::lbspdisplay::lb_800122C8(a0, a1, a2, a3 != 0) } }
pub fn lb_800138CC__37f797(a0: *mut HSD_GObj, a1: HSD_GObjEvent) { unsafe { crate::lb_r4::lb_800138CC(&*a0, a1) } }
pub fn lb_800138D8__383f48(a0: *mut HSD_GObj, a1: i8) { unsafe { crate::lb_r4::lb_800138D8(&*a0, a1) } }
pub fn un_802FF128__59ac96(a0: i32, a1: i32, a2: i32, a3: i32) { unsafe { melee_world::tu::ifcoget::un_802FF128(a0, a1, a2, a3) } }
pub fn gm_GetNumCostumesForCKind__a594a2(a0: u8) -> u8 { unsafe { crate::tu::gm_1601::gm_GetNumCostumesForCKind(a0) } }
pub fn HSD_ArchiveGetPublicAddress__1a942f(a0: *mut HSD_Archive, a1: *const i8) -> *mut c_void { unsafe { melee_runtime::tu::archive::HSD_ArchiveGetPublicAddress(a0, a1) } }
pub fn HSD_CObjAddAnim__a45160(a0: *mut HSD_CObj, a1: *mut HSD_CameraAnim) { unsafe { melee_runtime::tu::cobj::HSD_CObjAddAnim(a0, a1) } }
pub fn HSD_CObjAnim__1b6417(a0: *mut HSD_CObj) { unsafe { melee_runtime::tu::cobj::HSD_CObjAnim(a0) } }
pub fn HSD_CObjReqAnim__1aeeec(a0: *mut HSD_CObj, a1: f32) { unsafe { melee_runtime::tu::cobj::HSD_CObjReqAnim(a0, a1) } }
pub fn HSD_GObj_FogCallback__2aeaad(a0: *mut HSD_GObj, a1: i32) { unsafe { melee_runtime::tu::gobj::HSD_GObj_FogCallback(a0, a1) } }
pub fn HSD_JObjAlloc__5ecba4() -> *mut HSD_JObj { unsafe { melee_runtime::tu::jobj::HSD_JObjAlloc() } }
pub fn Toy_803048C0__c4f25d(a0: i32) -> i32 { unsafe { melee_items::tu::toy::Toy_803048C0(a0) } }
pub fn Toy_803060BC__946c5c(a0: i32, a1: i32) -> f32 { unsafe { melee_items::tu::toy::Toy_803060BC(a0, a1) } }
pub fn gm_8016895C__f7ee90(a0: *mut HSD_JObj, a1: *mut DynamicModelDesc, a2: i32) { unsafe { crate::tu::gm_1601::gm_8016895C(a0, a1, a2) } }
pub fn gm_801A4BA8__1c9c2c() -> u32 { unsafe { crate::tu::gmscene::gm_801A4BA8() } }
pub fn gm_801BEFB0__a895fb() -> CharacterKind { unsafe { crate::tu::gmgover::gm_801BEFB0() } }
pub fn gm_GetCurrentSceneEnterData__3de8a4() -> *mut c_void { unsafe { crate::tu::gmscene::gm_GetCurrentSceneEnterData() } }
pub fn lbAudioAx_800237A8__95b1fa(a0: i32, a1: i32, a2: i32) -> i32 { unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_800237A8(a0, a1, a2) } }
pub fn lb_8000C1C0__9b8c49(a0: *mut HSD_JObj, a1: *mut HSD_JObj) { unsafe { melee_runtime::tu::lb_00B0::lb_8000C1C0(a0, a1) } }
pub fn lb_8000C290__3ed82e(a0: *mut HSD_JObj, a1: *mut HSD_JObj) { unsafe { melee_runtime::tu::lb_00B0::lb_8000C290(a0, a1) } }
pub fn lb_80013B14__a63cd0(a0: *mut HSD_CameraDescPerspective) -> *mut HSD_CObj { unsafe { melee_runtime::tu::lbspdisplay::lb_80013B14(a0) } }
pub fn tyDisplay_8031B9DC__051a66(a0: i32) -> *mut TyDspEntry { unsafe { melee_items::tu::tydisplay::tyDisplay_8031B9DC(a0) } }
pub fn tyDisplay_8031BB34__c9308d(a0: i8) -> i32 { unsafe { (melee_items::tu::tydisplay::tyDisplay_8031BB34(a0)) as _ } }
pub fn tyDisplay_8031BB94__d3b220(a0: i8) -> *mut i8 { unsafe { (melee_items::tu::tydisplay::tyDisplay_8031BB94(a0)) as _ } }
pub fn gm_80160474__9c9fb4(a0: CharacterKind, a1: GameModeKind) -> i32 { unsafe { (crate::tu::gm_1601::gm_80160474(a0, a1)) as _ } }
pub fn gm_801604DC__f2261c(a0: CharacterKind, a1: GameModeKind) -> *mut i8 { unsafe { (crate::tu::gm_1601::gm_801604DC(a0, a1)) as _ } }
pub fn gm_80160564__4558bf(a0: CharacterKind, a1: GameModeKind) -> *mut i8 { unsafe { (crate::tu::gm_1601::gm_80160564(a0, a1)) as _ } }
pub fn gm_801A7B00__0265f5() { unsafe { crate::tu::gm_1A7A::gm_801A7B00() } }
pub fn gm_801BEFD0__339fe0() -> i32 { unsafe { crate::tu::gmgover::gm_801BEFD0() } }
pub fn gm_801BF010__72a66e() -> i32 { unsafe { crate::tu::gmgover::gm_801BF010() } }
pub fn gm_801BF050__2b5449() -> i32 { unsafe { crate::tu::gmgover::gm_801BF050() } }
pub fn gm_GetCurrentSceneExitData__188bc0() -> *mut c_void { unsafe { crate::tu::gmscene::gm_GetCurrentSceneExitData() } }
pub fn HSD_CObjEndCurrent__d69e71() { unsafe { melee_runtime::tu::cobj::HSD_CObjEndCurrent() } }
pub fn HSD_CObjSetCurrent__d18716(a0: *mut HSD_CObj) -> i32 { unsafe { (melee_runtime::tu::cobj::HSD_CObjSetCurrent(a0)) as _ } }
pub fn HSD_Free__3ee311(a0: *mut c_void) { unsafe { melee_runtime::tu::memory::HSD_Free(a0) } }
pub fn HSD_ImageDescCopyFromEFB__a5dfa4(a0: *mut HSD_ImageDesc, a1: u16, a2: u16, a3: u8, a4: i32) { unsafe { melee_runtime::tu::tobj::HSD_ImageDescCopyFromEFB(a0, a1, a2, a3, a4) } }
pub fn HSD_SObjLib_803A4740__344956(a0: *mut HSD_SObj) { unsafe { melee_runtime::tu::sobjlib::HSD_SObjLib_803A4740(a0) } }
pub fn HSD_SObjLib_803A49E0__00f645(a0: *mut HSD_GObj, a1: i32) { unsafe { melee_runtime::tu::sobjlib::HSD_SObjLib_803A49E0(a0, a1) } }
pub fn HSD_SObjLib_803A54EC__11e875(a0: *mut HSD_GObj, a1: i32) { unsafe { melee_runtime::tu::sobjlib::HSD_SObjLib_803A54EC(a0, a1) } }
pub fn HSD_SObjLib_803A55DC__ba9596(a0: *mut HSD_GObj, a1: u16, a2: u16, a3: i32) { unsafe { melee_runtime::tu::sobjlib::HSD_SObjLib_803A55DC(a0, a1, a2, a3) } }
pub fn Item_80266FA8__f7386b() { unsafe { melee_items::tu::item::Item_80266FA8() } }
pub fn Item_80266FCC__dfa49f() { unsafe { melee_items::tu::item::Item_80266FCC() } }
pub fn Stage_8022524C__8c7585() { unsafe { melee_world::tu::stage::Stage_8022524C() } }
pub fn efLib_Init__0977f5() { unsafe { melee_world::tu::eflib::efLib_Init() } }
pub fn Player_80031EBC__106f49(a0: i32) { unsafe { melee_fighter::player::player_80031ebc(a0) } }
pub fn Player_80032768__a41ef7(a0: i32, a1: *mut Vec3) { unsafe { melee_fighter::player::player_80032768(a0, &*a1) } }
pub fn Player_80032A04__beba94(a0: i32, a1: *mut Vec3) { unsafe { melee_fighter::player::player_80032a04(a0, &*a1) } }
pub fn Player_80032BB0__dd6bcc(a0: i32) -> f32 { unsafe { melee_fighter::player::player_80032bb0(a0) } }
pub fn Player_80036CF0__06a169(a0: i32) { unsafe { melee_fighter::tu::player::Player_80036CF0(a0) } }
pub fn Player_80036E20__54207b(a0: CharacterKind, a1: *mut HSD_Archive, a2: i32) { unsafe { melee_fighter::tu::player::Player_80036E20(a0, a1, a2) } }
pub fn Player_80036EA0__cdc15f(a0: i32) -> *mut HSD_JObj { unsafe { melee_fighter::tu::player::Player_80036EA0(a0) } }
pub fn Player_80036F34__e8645c(a0: i32, a1: i32) { unsafe { melee_fighter::tu::player::Player_80036F34(a0, a1) } }
pub fn Player_LoadPlayerCoords__879cc0(a0: i32, a1: *mut Vec3) { unsafe { melee_fighter::tu::player::Player_LoadPlayerCoords(a0, a1) } }
pub fn Player_SetCostumeId__920e80(a0: i32, a1: i32) { unsafe { melee_fighter::tu::player::Player_SetCostumeId(a0, a1) } }
pub fn Player_SetFacingDirection__e0fc0a(a0: i32, a1: f32) { unsafe { melee_fighter::tu::player::Player_SetFacingDirection(a0, a1) } }
pub fn Player_SetPlayerCharacter__d99f81(a0: i32, a1: CharacterKind) { unsafe { melee_fighter::tu::player::Player_SetPlayerCharacter(a0, a1) } }
pub fn Player_SetPlayerId__dba656(a0: i32, a1: i32) { unsafe { melee_fighter::tu::player::Player_SetPlayerId(a0, a1) } }
pub fn Player_SetScale__3c042c(a0: i32, a1: f32) { unsafe { melee_fighter::tu::player::Player_SetScale(a0, a1) } }
pub fn Toy_803102D0__0dcc17() { unsafe { melee_items::tu::toy::Toy_803102D0() } }
pub fn Toy_803124BC__320358() { unsafe { melee_items::tu::toy::Toy_803124BC() } }
pub fn lbAudioAx_80023694__e3b7af() -> i32 { unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80023694() } }
pub fn lb_8000BA0C__a558e3(a0: *mut HSD_JObj, a1: f32) { unsafe { melee_runtime::tu::lb_00B0::lb_8000BA0C(a0, a1) } }
pub fn mn_8022F0F0__97c62c(a0: i32) { unsafe { crate::tu::mn_22EC::mn_8022F0F0(a0) } }
pub fn HSD_GObj_80390EB8__5ed1a9(a0: i32) -> u32 { unsafe { melee_runtime::gobj::hsd_gobj_80390eb8(a0) } }
pub fn HSD_TObjAnim__0f2a76(a0: *mut HSD_TObj) { unsafe { melee_runtime::tu::tobj::HSD_TObjAnim(a0) } }
pub fn Player_GetTeam__f1d095(a0: i32) -> i32 { unsafe { melee_fighter::tu::player::Player_GetTeam(a0) } }
pub fn fn_80160400__597b46(a0: CharacterKind) -> u32 { unsafe { crate::tu::gm_1601::fn_80160400(a0) } }
pub fn fn_80165418__e6c527(a0: *mut MatchEnd) -> i32 { unsafe { crate::gm_ptr::fn_80165418(&*a0) } }
pub fn fn_801654A0__ecc03d(a0: *mut MatchEnd) -> i32 { unsafe { crate::gm_ptr::fn_801654A0(&*a0) } }
pub fn fn_8016F280__c29e83(a0: i32) -> i32 { unsafe { crate::tu::gm_16f1::fn_8016F280(a0) } }
pub fn fn_801701AC__5445dd() { unsafe { crate::tu::gm_16f1::fn_801701AC() } }
pub fn fn_801701B8__58bc4c() -> i32 { unsafe { crate::tu::gm_16f1::fn_801701B8() } }
pub fn fn_80179F6C__0d22a2(a0: i32, a1: *mut HSD_GObj) { unsafe { crate::tu::gm_1798::fn_80179F6C(a0, a1) } }
pub fn fn_80179F84__bc4b92(a0: *mut HSD_JObj) { unsafe { crate::tu::gm_1798::fn_80179F84(a0) } }
pub fn fn_8017A004__2fa472() { unsafe { crate::tu::gm_1798::fn_8017A004() } }
pub fn fn_8017A318__402e70(a0: i32) -> *mut HSD_GObj { unsafe { crate::tu::gm_1798::fn_8017A318(a0) } }
pub fn fn_8017A67C__2be55f(a0: CharacterKind, a1: i32, a2: i32) -> *mut HSD_GObj { unsafe { (crate::tu::gm_1798::fn_8017A67C(a0, a1, a2)) as _ } }
pub fn fn_8017A9B4__17b847(a0: i32) { unsafe { crate::tu::gm_1798::fn_8017A9B4(a0) } }
pub fn fn_8017AA78__72317a(a0: *const u8) { unsafe { crate::tu::gm_1798::fn_8017AA78(a0) } }
pub fn fn_8017AD28__3065ff(a0: i32) -> i32 { unsafe { crate::gm::fn_8017AD28(a0) } }
pub fn fn_8017AD48__fef659(a0: i32) -> i32 { unsafe { crate::gm::fn_8017AD48(a0) } }
pub fn fn_8017AD78__e2265e(a0: i32) -> i32 { unsafe { crate::gm::fn_8017AD78(a0) } }
pub fn fn_8017B010__57e88d(a0: i32) -> i32 { unsafe { crate::tu::gm_17ad::fn_8017B010(a0) } }
pub fn fn_8017B410__473c57(a0: i32) -> i32 { unsafe { crate::tu::gm_17ad::fn_8017B410(a0) } }
pub fn fn_8017B91C__85de94(a0: i32) -> i32 { unsafe { crate::tu::gm_17ad::fn_8017B91C(a0) } }
pub fn fn_8017B9F4__6ec3d8(a0: i32) -> i32 { unsafe { crate::tu::gm_17ad::fn_8017B9F4(a0) } }
pub fn fn_8017BACC__5e81e2(a0: i32) -> i32 { unsafe { crate::tu::gm_17ba::fn_8017BACC(a0) } }
pub fn fn_8017BB30__5566ea(a0: i32) -> i32 { unsafe { crate::tu::gm_17ba::fn_8017BB30(a0) } }
pub fn fn_8017BB94__2bf7c2(a0: i32) -> *const i8 { unsafe { (crate::tu::gm_17ba::fn_8017BB94(a0)) as _ } }
pub fn fn_8017BC50__843ede(a0: i32) -> *const i8 { unsafe { (crate::tu::gm_17ba::fn_8017BC50(a0)) as _ } }
pub fn fn_8017BD0C__f30e74(a0: i32) -> *const i8 { unsafe { (crate::tu::gm_17ba::fn_8017BD0C(a0)) as _ } }
pub fn fn_8017BDC8__877377(a0: i32) -> *const i8 { unsafe { (crate::tu::gm_17ba::fn_8017BDC8(a0)) as _ } }
pub fn gm_80160854__387c80(a0: u8, a1: u8, a2: u8, a3: u8) -> u32 { unsafe { crate::gm::gm_80160854(a0, a1, a2, a3) } }
pub fn lbVector_WorldToScreen__35d0d5(a0: *mut HSD_CObj, a1: *const Vec3, a2: *mut Vec3, a3: i32) -> *mut Vec3 { unsafe { (melee_runtime::tu::lbvector::lbVector_WorldToScreen(a0, a1 as _, a2 as _, a3)) as _ } }
pub fn lb_8000C07C__443874(a0: *mut HSD_JObj, a1: i32, a2: *mut *mut HSD_AnimJoint, a3: *mut *mut HSD_MatAnimJoint, a4: *mut *mut HSD_ShapeAnimJoint) { unsafe { melee_runtime::tu::lb_00B0::lb_8000C07C(a0, a1, a2, a3, a4) } }
pub fn lb_8000C0E8__cceb94(a0: *mut HSD_JObj, a1: i32, a2: *mut DynamicModelDesc) { unsafe { melee_runtime::tu::lb_00B0::lb_8000C0E8(a0, a1, a2) } }
pub fn lb_80014574__7435d0(a0: u8, a1: i32, a2: i32, a3: i32) { unsafe { crate::tu::lb_013B::lb_80014574(a0, a1, a2, a3) } }
pub fn un_802FF1B4__2022ae() { unsafe { melee_world::tu::ifcoget::un_802FF1B4() } }
pub fn gm_80168B34__ad8428(a0: CharacterKind, a1: i32, a2: i32) -> f32 { unsafe { crate::tu::gm_1601::gm_80168B34(a0, a1, a2) } }
pub fn fn_80168E54__246570(a0: i8, a1: i8, a2: u8, a3: u8) { unsafe { crate::tu::gm_1601::fn_80168E54(a0, a1, a2, a3) } }
pub fn fn_80179854__dccf02() { unsafe { crate::tu::gm_1798::fn_80179854() } }
pub fn fn_80179D3C__70302b(a0: *mut HSD_GObj, a1: i32) { unsafe { crate::tu::gm_1798::fn_80179D3C(a0, a1) } }
pub fn fn_80179D60__4ab2eb(a0: *mut HSD_GObj, a1: i32) { unsafe { crate::tu::gm_1798::fn_80179D60(a0, a1) } }
pub fn fn_80179D84__7d94c0(a0: *mut HSD_GObj, a1: i32) { unsafe { crate::tu::gm_1798::fn_80179D84(a0, a1) } }
pub fn fn_80179DA8__94304a(a0: *mut HSD_GObj, a1: i32) { unsafe { crate::tu::gm_1798::fn_80179DA8(a0, a1) } }
pub fn fn_80179DCC__bd00a6(a0: *mut HSD_GObj, a1: i32) { unsafe { crate::tu::gm_1798::fn_80179DCC(a0, a1) } }
pub fn fn_80179E34__ecefd5(a0: *mut HSD_GObj, a1: i32) { unsafe { crate::tu::gm_1798::fn_80179E34(a0, a1) } }
pub fn fn_80179E9C__2d3024(a0: *mut HSD_GObj, a1: i32) { unsafe { crate::tu::gm_1798::fn_80179E9C(a0, a1) } }
pub fn fn_80179F04__e82117(a0: *mut HSD_GObj, a1: i32) { unsafe { crate::tu::gm_1798::fn_80179F04(a0, a1) } }
pub fn fn_80175DC8__182ff2(a0: *mut HSD_GObj) { unsafe { crate::tu::gmresult::fn_80175DC8(a0) } }
pub fn HSD_AObjReqAnim__f16456(a0: *mut HSD_AObj, a1: f32) { unsafe { melee_runtime::tu::aobj::HSD_AObjReqAnim(a0, a1) } }
pub fn HSD_TObjReqAnim__41ec43(a0: *mut HSD_TObj, a1: f32) { unsafe { melee_runtime::tu::tobj::HSD_TObjReqAnim(a0, a1) } }
pub fn HSD_TlutLoadDesc__62195e(a0: *mut HSD_TlutDesc) -> *mut HSD_Tlut { unsafe { melee_runtime::tu::tobj::HSD_TlutLoadDesc(a0) } }
pub fn HSD_TlutRemove__76ada5(a0: *mut HSD_Tlut) { unsafe { melee_runtime::tu::tobj::HSD_TlutRemove(a0) } }
pub fn lbDObjSetRateAll__b1bb98(a0: *mut HSD_DObj, a1: f32) { unsafe { melee_runtime::tu::lb_00B0::lbDObjSetRateAll(a0, a1) } }
pub fn lbDObjReqAnimAll__2dab57(a0: *mut HSD_DObj, a1: f32) { unsafe { melee_runtime::tu::lb_00B0::lbDObjReqAnimAll(a0, a1) } }
pub fn lbGetJObjCurrFrame__0391a5(a0: *mut HSD_JObj) -> f32 { unsafe { melee_runtime::tu::lb_00B0::lbGetJObjCurrFrame(a0) } }
pub fn lbArchive_LoadArchive__471a59(a0: *const i8) -> *mut HSD_Archive { unsafe { melee_runtime::tu::lbarchive::lbArchive_LoadArchive(a0 as _) } }
pub fn lbArchive_80016EFC__e6851b(a0: *mut HSD_Archive) { unsafe { melee_runtime::tu::lbarchive::lbArchive_80016EFC(a0) } }
pub fn lbDvd_8001819C__ea477a(a0: *const i8) -> *mut HSD_Archive { unsafe { melee_runtime::tu::lbdvd::lbDvd_8001819C(a0) } }
pub fn gm_MexStockMatAnim__1be326() -> *mut HSD_MatAnimJoint { unsafe { crate::tu::gm_1601::gm_MexStockMatAnim() } }
pub fn gm_MexStockFrame__77cf16(a0: i32, a1: i32) -> f32 { unsafe { crate::tu::gm_1601::gm_MexStockFrame(a0, a1) } }
pub fn Player_GetFalls__0bef1d(a0: i32) -> i32 { unsafe { melee_fighter::tu::player::Player_GetFalls(a0) } }
pub fn Player_GetSelfDestructs__ec7657(a0: i32) -> u32 { unsafe { melee_fighter::tu::player::Player_GetSelfDestructs(a0) } }
pub fn Toy_GetTrophyTotal__16ecae() -> i32 { unsafe { melee_items::tu::toy::Toy_GetTrophyTotal() } }
pub fn fn_80162CCC__01f4c9() -> i32 { unsafe { (crate::tu::gm_1601::fn_80162CCC()) as _ } }
pub fn fn_801630C4__4f1980() -> i32 { unsafe { (crate::tu::gm_1601::fn_801630C4()) as _ } }
pub fn fn_80163D24__8419dc() -> i32 { unsafe { (crate::tu::gm_1601::fn_80163D24()) as _ } }
pub fn fn_80163D74__3aad93() -> i32 { unsafe { (crate::tu::gm_1601::fn_80163D74()) as _ } }
pub fn fn_80163FA4__9717fa(a0: u8) -> i32 { unsafe { crate::tu::gm_1601::fn_80163FA4(a0) } }
pub fn fn_80160710__f1c086(a0: i32) -> u8 { unsafe { crate::gm::fn_80160710(a0) } }
pub fn fn_801607A8__8732d1(a0: i32) -> u8 { unsafe { crate::gm::fn_801607A8(a0) } }
pub fn fn_801607F4__5e4d8a(a0: i32) -> u8 { unsafe { crate::gm::fn_801607F4(a0) } }
pub fn gmMainLib_8015CCE4__74a1e5() -> *mut gmm_x0_44_t { unsafe { crate::tu::gmmain_lib::gmMainLib_8015CCE4() } }
pub fn gmMainLib_8015CEFC__7f9c8b(a0: i32) -> i32 { unsafe { (crate::tu::gmmain_lib::gmMainLib_8015CEFC(a0)) as _ } }
pub fn gmMainLib_8015CF94__564253() -> i32 { unsafe { (crate::tu::gmmain_lib::gmMainLib_8015CF94()) as _ } }
pub fn gmMainLib_8015CFCC__c8614f(a0: u8) -> i32 { unsafe { crate::tu::gmmain_lib::gmMainLib_8015CFCC(a0) } }
pub fn gmMainLib_8015D0F4__90773f(a0: u8) -> i32 { unsafe { crate::tu::gmmain_lib::gmMainLib_8015D0F4(a0) } }
pub fn gmMainLib_8015D194__9e9b6e(a0: u8) -> *mut u8 { unsafe { crate::tu::gmmain_lib::gmMainLib_8015D194(a0) } }
pub fn gmMainLib_8015D1AC__400144(a0: u8) -> i32 { unsafe { crate::tu::gmmain_lib::gmMainLib_8015D1AC(a0) } }
pub fn gmMainLib_8015D21C__a4a904(a0: u8) -> i32 { unsafe { crate::tu::gmmain_lib::gmMainLib_8015D21C(a0) } }
pub fn gmMainLib_8015D2BC__88a357(a0: u8) -> *mut u8 { unsafe { crate::tu::gmmain_lib::gmMainLib_8015D2BC(a0) } }
pub fn gmMainLib_8015D2D4__48be73(a0: u8) -> i32 { unsafe { crate::tu::gmmain_lib::gmMainLib_8015D2D4(a0) } }
pub fn gmMainLib_8015D344__7c3fa3(a0: u8) -> i32 { unsafe { crate::tu::gmmain_lib::gmMainLib_8015D344(a0) } }
pub fn gmMainLib_8015D3E4__d26187(a0: u8) -> *mut u8 { unsafe { crate::tu::gmmain_lib::gmMainLib_8015D3E4(a0) } }
pub fn gmMainLib_8015D4A8__7c7ef4(a0: u8) -> i32 { unsafe { crate::tu::gmmain_lib::gmMainLib_8015D4A8(a0) } }
pub fn gmMainLib_8015D508__130daf() -> i32 { unsafe { (crate::tu::gmmain_lib::gmMainLib_8015D508()) as _ } }
pub fn gmMainLib_8015D5DC__7e9655() -> i32 { unsafe { (crate::tu::gmmain_lib::gmMainLib_8015D5DC()) as _ } }
pub fn gmMainLib_8015D804__205f16(a0: i32) -> *mut u32 { unsafe { crate::tu::gmmain_lib::gmMainLib_8015D804(a0) } }
pub fn gmMainLib_8015D818__eb1059(a0: u32) -> i32 { unsafe { crate::tu::gmmain_lib::gmMainLib_8015D818(a0) } }
pub fn gmMainLib_8015D8B0__6f65da(a0: u32) { unsafe { crate::tu::gmmain_lib::gmMainLib_8015D8B0(a0) } }
pub fn gmMainLib_8015D8D8__3b7684(a0: u32) -> i32 { unsafe { crate::tu::gmmain_lib::gmMainLib_8015D8D8(a0) } }
pub fn gmMainLib_8015D8FC__2e3f2d(a0: u32) { unsafe { crate::tu::gmmain_lib::gmMainLib_8015D8FC(a0) } }
pub fn gmMainLib_8015D924__d962a9(a0: u32) { unsafe { crate::tu::gmmain_lib::gmMainLib_8015D924(a0) } }
pub fn gmMainLib_8015D94C__6bc50f(a0: u32) -> i32 { unsafe { crate::tu::gmmain_lib::gmMainLib_8015D94C(a0) } }
pub fn gmMainLib_8015D984__650206(a0: u32) -> i32 { unsafe { (crate::tu::gmmain_lib::gmMainLib_8015D984(a0)) as _ } }
pub fn gmMainLib_8015DA1C__db8612(a0: u32) -> i32 { unsafe { crate::tu::gmmain_lib::gmMainLib_8015DA1C(a0) } }
pub fn gmMainLib_8015DA68__a960dd(a0: u32) { unsafe { crate::tu::gmmain_lib::gmMainLib_8015DA68(a0) } }
pub fn gmMainLib_8015DB6C__005b83(a0: u8) -> u8 { unsafe { crate::tu::gmmain_lib::gmMainLib_8015DB6C(a0) } }
pub fn gmMainLib_8015EDA4__0c1318() -> *mut u16 { unsafe { crate::tu::gmmain_lib::gmMainLib_8015EDA4() } }
pub fn gmMainLib_8015EDC8__b0e6ce() -> *mut gmm_x1868_1A8_t { unsafe { crate::tu::gmmain_lib::gmMainLib_8015EDC8() } }
pub fn gmMainLib_8015EDE4__704f38() { unsafe { crate::tu::gmmain_lib::gmMainLib_8015EDE4() } }
pub fn gmMainLib_8015EDF8__ec3479() { unsafe { crate::tu::gmmain_lib::gmMainLib_8015EDF8() } }
pub fn gmMainLib_8015EE1C__25cad2() { unsafe { crate::tu::gmmain_lib::gmMainLib_8015EE1C() } }
pub fn gmMainLib_8015EE30__935172() { unsafe { crate::tu::gmmain_lib::gmMainLib_8015EE30() } }
pub fn gmMainLib_8015EE54__f451fd() { unsafe { crate::tu::gmmain_lib::gmMainLib_8015EE54() } }
pub fn gmMainLib_8015EE68__08be19() { unsafe { crate::tu::gmmain_lib::gmMainLib_8015EE68() } }
pub fn gmMainLib_8015EEA0__530f0c() { unsafe { crate::tu::gmmain_lib::gmMainLib_8015EEA0() } }
pub fn gmMainLib_8015EEB4__9edab6() { unsafe { crate::tu::gmmain_lib::gmMainLib_8015EEB4() } }
pub fn gm_80162D1C__47f299() -> i32 { unsafe { (crate::tu::gm_1601::gm_80162D1C()) as _ } }
pub fn gm_80162EC8__625f2c() -> i32 { unsafe { (crate::tu::gm_1601::gm_80162EC8()) as _ } }
pub fn gm_80162F18__c03e80() -> i32 { unsafe { (crate::tu::gm_1601::gm_80162F18()) as _ } }
pub fn gm_80163114__6184d4() -> i32 { unsafe { (crate::tu::gm_1601::gm_80163114()) as _ } }
pub fn gm_80164600__f93b0a() -> i32 { unsafe { (crate::tu::gm_1601::gm_80164600()) as _ } }
pub fn gm_80164ABC__6f0d5a() -> i32 { unsafe { (crate::tu::gm_1601::gm_80164ABC()) as _ } }
pub fn gm_8016B3D8__50cc51() -> i32 { unsafe { (crate::tu::gmvs::gm_8016B3D8()) as _ } }
pub fn gm_801BEBC0__760ddc(a0: u8) -> u8 { unsafe { crate::tu::gmevent::gm_801BEBC0(a0) } }
pub fn gm_CKindToUnlockIndex__da55c8(a0: CharacterKind) -> i8 { unsafe { crate::gm::gm_CKindToUnlockIndex(a0 as _) } }
pub fn gm_GetCKindByUnlockIndex__38a227(a0: i32) -> u8 { unsafe { crate::gm::gm_GetCKindByUnlockIndex(a0) } }
pub fn gm_GetVsPlayMatchTotal__0222a7() -> u32 { unsafe { crate::tu::gm_1601::gm_GetVsPlayMatchTotal() } }
pub fn gm_IsCurrently1PMode_inline__c5c82f() -> i32 { unsafe { (crate::tu::gmvs::gm_IsCurrently1PMode_inline()) as _ } }
pub fn gm_SelKindToUnlockIndex__eec447(a0: SelectableCharacterKind) -> u8 { unsafe { crate::gm::gm_SelKindToUnlockIndex(a0 as _) } }
pub fn lbLang_GetLanguageSetting__f30d84() -> i32 { unsafe { (melee_runtime::tu::lblanguage::lbLang_GetLanguageSetting()) as _ } }
pub fn lbLang_IsSettingUS__c5976a() -> i32 { unsafe { (melee_runtime::tu::lblanguage::lbLang_IsSettingUS()) as _ } }
pub fn lbTime_GetTimeInSeconds__050097() -> u32 { unsafe { melee_runtime::tu::lbtime::lbTime_GetTimeInSeconds() } }
pub fn pl_80038914__ddcb6e() -> *mut pl_804D6470_t { unsafe { (melee_fighter::plbonus3::pl_80038914()) as _ } }
pub fn pl_80039418__dd4c4c(a0: i32, a1: i32) -> u32 { unsafe { melee_fighter::plbonus3::pl_80039418(a0, a1) } }
pub fn pl_80039450__d7289b(a0: i32) { unsafe { melee_fighter::tu::plbonus::pl_80039450(a0) } }
pub fn pl_800407C8__ec2f30(a0: i32) -> f32 { unsafe { melee_fighter::plbonus::pl_800407c8(a0) } }
pub fn pl_80040870__e03814(a0: i32) -> f32 { unsafe { melee_fighter::plbonus::pl_80040870(a0) } }
pub fn pl_80040894__e3fbb9(a0: i32) -> u32 { unsafe { melee_fighter::plbonus::pl_80040894(a0) } }
pub fn pl_800408B8__8b6ad9(a0: i32) -> u32 { unsafe { melee_fighter::plbonus::pl_800408b8(a0) } }
pub fn pl_800408DC__53434c(a0: i32) -> i32 { unsafe { melee_fighter::plbonus::pl_800408dc(a0) } }
pub fn un_80304470__1f3d64() -> i32 { unsafe { (melee_items::tu::toy::un_80304470()) as _ } }
pub fn un_80304510__b30e24() -> i32 { unsafe { (melee_items::tu::toy::un_80304510()) as _ } }
pub fn Toy_80305918__7fb4b7(a0: i8, a1: i32, a2: i32) { unsafe { melee_items::tu::toy::Toy_80305918(a0, a1, a2) } }
pub fn fn_80164B48__c091f4() -> i32 { unsafe { (crate::tu::gm_1601::fn_80164B48()) as _ } }
pub fn gmMainLib_8015D084__344a84(a0: u8) -> *mut i32 { unsafe { crate::tu::gmmain_lib::gmMainLib_8015D084(a0) } }
pub fn gmMainLib_8015D640__08122a() -> i32 { unsafe { (crate::tu::gmmain_lib::gmMainLib_8015D640()) as _ } }
pub fn gmMainLib_8015DA40__1b7ad5(a0: u32) { unsafe { crate::tu::gmmain_lib::gmMainLib_8015DA40(a0) } }
pub fn gmMainLib_8015DADC__9e9fc5(a0: u32) -> i32 { unsafe { (crate::tu::gmmain_lib::gmMainLib_8015DADC(a0)) as _ } }
pub fn gmMainLib_8015EE90__25e57f() -> i32 { unsafe { crate::tu::gmmain_lib::gmMainLib_8015EE90() } }
pub fn gm_GetPreviousGameMode__0d6d4c() -> u8 { unsafe { crate::tu::gm_1A3F::gm_GetPreviousGameMode() } }
pub fn gm_SelKindToCKind__b7f955(a0: u8) -> u8 { unsafe { crate::gm::gm_SelKindToCKind(a0) } }
pub fn lbTime_8000AEC8__f87b66(a0: u32, a1: u32) -> u32 { unsafe { crate::lb::lbTime_8000AEC8(a0, a1) } }
pub fn lbTime_8000AEE4__5500d0(a0: u32, a1: i32) -> u32 { unsafe { crate::lb::lbTime_8000AEE4(a0, a1) } }
pub fn un_803045A0__3e8c43() -> i32 { unsafe { (melee_items::tu::toy::un_803045A0()) as _ } }
pub fn un_80304690__46ca57() -> i32 { unsafe { (melee_items::tu::toy::un_80304690()) as _ } }
pub fn un_80304780__14ca0d() -> i32 { unsafe { (melee_items::tu::toy::un_80304780()) as _ } }
pub fn Ground_801C0378__874334(a0: i32) { unsafe { melee_world::tu::ground::Ground_801C0378(a0) } }
pub fn HSD_CObjEraseScreen__c961ec(a0: *mut HSD_CObj, a1: i32, a2: i32, a3: i32) { unsafe { melee_runtime::tu::cobj::HSD_CObjEraseScreen(a0, a1, a2, a3) } }
pub fn HSD_CObjSetScissor__2d3b20(a0: *mut HSD_CObj, a1: *mut _Scissor) { unsafe { melee_runtime::tu::cobj::HSD_CObjSetScissor(a0, a1 as _) } }
pub fn HSD_ImageDescCopyFromEFB__464d65(a0: *mut HSD_ImageDesc, a1: u16, a2: u16, a3: GXBool, a4: i32) { unsafe { melee_runtime::tu::tobj::HSD_ImageDescCopyFromEFB(a0, a1, a2, a3 as _, a4) } }
pub fn Player_80032A04__2b7dce(a0: i32, a1: *mut Vec) { unsafe { melee_fighter::player::player_80032a04(a0, &*a1) } }
pub fn Player_800360D8__b9fa26(a0: i32) -> f32 { unsafe { melee_fighter::player::player_800360d8(a0) } }
pub fn Player_InitAllPlayers__1d8abf() { unsafe { melee_fighter::tu::player::Player_InitAllPlayers() } }
pub fn Player_SetSlottype__ff2974(a0: i32, a1: Gm_PKind) { unsafe { melee_fighter::tu::player::Player_SetSlottype(a0, a1) } }
pub fn Stage_802251E8__6e8910(a0: StKind, a1: *mut i32) { unsafe { melee_world::tu::stage::Stage_802251E8(a0, a1) } }
pub fn fn_80174274__726c5b() -> *mut MatchEnd { unsafe { crate::tu::gmresult::fn_80174274() } }
pub fn fn_801795D4__0f9c8c() -> i32 { unsafe { crate::tu::gmresultplayer::fn_801795D4() } }
pub fn fn_801796F0__2a3c9d(a0: i32) -> i32 { unsafe { crate::tu::gmresultplayer::fn_801796F0(a0) } }
pub fn gm_80160438__384bdb(a0: i32) -> *mut i8 { unsafe { (crate::tu::gm_1601::gm_80160438(a0)) as _ } }
pub fn gm_MexVanillaKind__b3a17c(a0: i32, a1: i32) -> i32 { unsafe { crate::tu::gm_1601::gm_MexVanillaKind(a0, a1) } }
pub fn lbBgFlash_800206D4__8952dd(a0: *mut _GXColor, a1: *mut _GXColor, a2: i32) { unsafe { melee_runtime::tu::lbbgflash::lbBgFlash_800206D4(a0 as _, a1 as _, a2) } }
pub fn lbBgFlash_800208EC__c1a64f(a0: i32) { unsafe { melee_runtime::tu::lbbgflash::lbBgFlash_800208EC(a0) } }
pub fn lb_8000FCDC__a0c24e() { unsafe { melee_runtime::tu::lb_00F9::lb_8000FCDC() } }
pub fn lb_800121FC__096a83(a0: *mut HSD_ImageDesc, a1: i32, a2: i32, a3: _GXTexFmt, a4: i16) -> *mut HSD_ImageDesc { unsafe { melee_runtime::tu::lbspdisplay::lb_800121FC(a0, a1, a2, a3 as _, a4) } }
pub fn mpColl_80041C78__63449b() { unsafe { melee_world::tu::mpcoll::mpColl_80041C78() } }
pub fn GetPersistentNameData__529241(a0: i32) -> *mut NameTagData { unsafe { crate::tu::gmmain_lib::GetPersistentNameData(a0) } }
pub fn Ground_801C1DAC__5d253e() -> u32 { unsafe { melee_world::tu::ground::Ground_801C1DAC() } }
pub fn Ground_801C5A60__d6325d() { unsafe { melee_world::tu::ground::Ground_801C5A60() } }
pub fn Ground_801C5AD0__e73f00(a0: i32) -> u32 { unsafe { melee_world::tu::ground::Ground_801C5AD0(a0) } }
pub fn Player_80031848__c31dd9(a0: i32) { unsafe { melee_fighter::player::player_80031848(a0) } }
pub fn Player_GetAttackRatio__61b7a9(a0: i32) -> f32 { unsafe { melee_fighter::tu::player::Player_GetAttackRatio(a0) } }
pub fn Player_GetCpuLevel__8735b7(a0: i32) -> i32 { unsafe { melee_fighter::tu::player::Player_GetCpuLevel(a0) } }
pub fn Player_GetCpuType__ec7fec(a0: i32) -> i32 { unsafe { melee_fighter::tu::player::Player_GetCpuType(a0) } }
pub fn Player_GetUnk50__869319(a0: i32) -> f32 { unsafe { melee_fighter::tu::player::Player_GetUnk50(a0) } }
pub fn Player_SetMoreFlagsBit3__e7d064(a0: i32, a1: u8) { unsafe { melee_fighter::tu::player::Player_SetMoreFlagsBit3(a0, a1) } }
pub fn Player_SetMoreFlagsBit4__b3407d(a0: i32, a1: u8) { unsafe { melee_fighter::tu::player::Player_SetMoreFlagsBit4(a0, a1) } }
pub fn Player_SetOtherStamina__ccc0cd(a0: i32, a1: i32) { unsafe { melee_fighter::tu::player::Player_SetOtherStamina(a0, a1) } }
pub fn Stage_80225074__bd6d40(a0: i32) -> i32 { unsafe { melee_world::tu::stage::Stage_80225074(a0) } }
pub fn Stage_8022519C__9b8f69(a0: StKind) -> GrKind { unsafe { melee_world::tu::stage::Stage_8022519C(a0) } }
pub fn Toy_SetUnlockState__c2d7ba(a0: enum_t, a1: i32) { unsafe { melee_items::tu::toy::Toy_SetUnlockState(a0 as _, a1) } }
pub fn fn_80162BFC__e70f6e(a0: i8, a1: i32) -> i32 { unsafe { (crate::tu::gm_1601::fn_80162BFC(a0, a1)) as _ } }
pub fn fn_80162DF8__3b29ac(a0: u8, a1: u32) -> i32 { unsafe { crate::tu::gm_1601::fn_80162DF8(a0, a1) } }
pub fn fn_80162FF4__5bf35f(a0: u8, a1: u32) -> i32 { unsafe { (crate::tu::gm_1601::fn_80162FF4(a0, a1)) as _ } }
pub fn fn_8016A450__cff474() { unsafe { crate::tu::gm_16A2::fn_8016A450() } }
pub fn fn_8016A46C__67847f() { unsafe { crate::tu::gm_16A2::fn_8016A46C() } }
pub fn fn_8016B4BC__213ddc() -> i32 { unsafe { (crate::tu::gmvs::fn_8016B4BC()) as _ } }
pub fn fn_8016B510__59b051() -> i32 { unsafe { (crate::tu::gmvs::fn_8016B510()) as _ } }
pub fn fn_8016F030__9a74e8(a0: *mut StartMeleeData) { unsafe { crate::tu::gmvs::fn_8016F030(a0) } }
pub fn ftBossLib_8015C2A8__1b0da4() -> i32 { unsafe { (melee_fighter::tu::ftbosslib::ftBossLib_8015C2A8()) as _ } }
pub fn ftBossLib_8015C530__eca1c0(a0: i32) -> i32 { unsafe { melee_fighter::tu::ftbosslib::ftBossLib_8015C530(a0) } }
pub fn ftBossLib_8015CA6C__e73ae5(a0: i32) { unsafe { melee_fighter::tu::ftbosslib::ftBossLib_8015CA6C(a0) } }
pub fn ftBossLib_8015CB7C__08c5b8() { unsafe { melee_fighter::tu::ftbosslib::ftBossLib_8015CB7C() } }
pub fn ftBossLib_IsMasterHandEntry__b6143e() -> i32 { unsafe { (melee_fighter::tu::ftbosslib::ftBossLib_IsMasterHandEntry()) as _ } }
pub fn ftLib_80086824__82b58f() { unsafe { melee_fighter::ftlib::ftlib_80086824() } }
pub fn ftLib_800868A4__7fb267() { unsafe { melee_fighter::ftlib::ftlib_800868a4() } }
pub fn ftLib_80087574__3ccbbb(a0: i8) { unsafe { melee_fighter::tu::ftlib::ftlib_80087574(a0) } }
pub fn ftLib_80087610__f2cb3c(a0: u8) { unsafe { melee_fighter::tu::ftlib::ftlib_80087610(a0) } }
pub fn gmMainLib_8015D134__50170c(a0: u8) { unsafe { crate::tu::gmmain_lib::gmMainLib_8015D134(a0) } }
pub fn gmMainLib_8015D1C8__160fa4(a0: u8, a1: i32) { unsafe { crate::tu::gmmain_lib::gmMainLib_8015D1C8(a0, a1) } }
pub fn gmMainLib_8015D25C__8fef81(a0: u8) { unsafe { crate::tu::gmmain_lib::gmMainLib_8015D25C(a0) } }
pub fn gmMainLib_8015D2F0__a48088(a0: u8, a1: i32) { unsafe { crate::tu::gmmain_lib::gmMainLib_8015D2F0(a0, a1) } }
pub fn gmMainLib_8015D384__d67253(a0: u8) { unsafe { crate::tu::gmmain_lib::gmMainLib_8015D384(a0) } }
pub fn gmMainLib_8015D3FC__fe331f(a0: u8) -> i32 { unsafe { crate::tu::gmmain_lib::gmMainLib_8015D3FC(a0) } }
pub fn gmMainLib_8015D418__778d54(a0: u8, a1: i32) { unsafe { crate::tu::gmmain_lib::gmMainLib_8015D418(a0, a1) } }
pub fn gm_801623D8__6fdb31() -> i32 { unsafe { crate::tu::gm_1601::gm_801623D8() } }
pub fn gm_801623FC__a1a1bf(a0: i32) { unsafe { crate::tu::gm_1601::gm_801623FC(a0) } }
pub fn gm_GetCurrentSceneIndex__5f49a4() -> u8 { unsafe { crate::tu::gm_1A3F::gm_GetCurrentSceneIndex() } }
pub fn gm_SetupHumanPlayer__13678a(a0: *mut PlayerInitData, a1: u8, a2: u8, a3: u8, a4: u8) { unsafe { crate::gm_r4::gm_SetupHumanPlayer(&mut *a0, a1, a2, a3, a4) } }
pub fn grPushOn_80219230__68fe3c(a0: i32) -> i32 { unsafe { melee_world::tu::grpushon::grPushOn_80219230(a0) } }
pub fn lbAudioAx_800237A8__04460b(a0: enum_t, a1: i32, a2: i32) -> i32 { unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_800237A8(a0 as _, a1, a2) } }
pub fn lbBgFlash_800205F0__8f9470(a0: i32) { unsafe { melee_runtime::tu::lbbgflash::lbBgFlash_800205F0(a0) } }
pub fn lbCardGame_SaveChanges__0fc9e6() { unsafe { melee_runtime::tu::lbcardgame::lbCardGame_SaveChanges() } }
pub fn lbTime_8000AF74__3857e8(a0: u32, a1: i32) -> u32 { unsafe { crate::lb::lbTime_8000AF74(a0, a1) } }
pub fn Player_GetPlayerCharacter__96861a(a0: i32) -> CharacterKind { unsafe { melee_fighter::tu::player::Player_GetPlayerCharacter(a0) } }
pub fn Player_LoseStock__6785b0(a0: i32) { unsafe { melee_fighter::tu::player::Player_LoseStock(a0) } }
pub fn gmMainLib_8015CDD4__51396a() -> *mut gmm_x0_528_t { unsafe { crate::tu::gmmain_lib::gmMainLib_8015CDD4() } }
pub fn ifStatus_802F6948__0cc861(a0: i32) { unsafe { melee_world::tu::ifstatus::ifStatus_802F6948(a0) } }
pub fn ifStatus_802F6E3C__cdcf00(a0: i32) { unsafe { melee_world::tu::ifstatus::ifStatus_802F6E3C(a0) } }
pub fn Command_Execute__d5bf79(a0: *mut CommandInfo, a1: u32) -> i32 { unsafe { (melee_runtime::tu::lbcommand::Command_Execute(a0, a1)) as _ } }
pub fn HSD_PadRumbleAdd__f8a1ce(a0: u8, a1: i32, a2: i32, a3: i32, a4: *mut c_void) -> i32 { unsafe { melee_runtime::tu::rumble::HSD_PadRumbleAdd(a0, a1, a2, a3, a4) } }
pub fn HSD_PadRumbleOn__3bcc58(a0: u8) { unsafe { melee_runtime::tu::rumble::HSD_PadRumbleOn(a0) } }
pub fn HSD_PadRumbleRemove__adbf79(a0: u8) { unsafe { melee_runtime::tu::rumble::HSD_PadRumbleRemove(a0) } }
pub fn HSD_Init_803755A8__49528d() { unsafe { melee_runtime::tu::initialize::HSD_Init_803755A8() } }
pub fn HSD_Rumble_80378524__8fd3f6(a0: i32) { unsafe { melee_runtime::tu::rumble::HSD_Rumble_80378524(a0) } }
pub fn HSD_SisLib_803A84BC__b2aa0b(a0: *mut HSD_GObj, a1: *mut HSD_Text) { unsafe { melee_runtime::tu::hsd_3A76::HSD_SisLib_803A84BC(a0, a1 as _) } }
pub fn lbAudioAx_80024F08__08bb8a() { unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80024F08() } }
pub fn lbAudioAx_80024F6C__7ce9bf() { unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80024F6C() } }
pub fn lbLang_IsSavedLanguageJP__e2b095() -> i32 { unsafe { (melee_runtime::tu::lblanguage::lbLang_IsSavedLanguageJP()) as _ } }
pub fn HSD_PadGetRawQueueCount__b9c435() -> u8 { unsafe { melee_runtime::tu::controller::HSD_PadGetRawQueueCount() } }
pub fn HSD_PadGetResetSwitch__f7c5ee() -> i32 { unsafe { melee_runtime::tu::controller::HSD_PadGetResetSwitch() } }
pub fn HSD_PadRenewCopyStatus__300d10() { unsafe { melee_runtime::tu::controller::HSD_PadRenewCopyStatus() } }
pub fn HSD_PadRenewGameStatus__5366e1() { unsafe { melee_runtime::tu::controller::HSD_PadRenewGameStatus() } }
pub fn HSD_PadRenewMasterStatus__03a172() { unsafe { melee_runtime::tu::controller::HSD_PadRenewMasterStatus() } }
pub fn HSD_PadRenewRawStatus__ba4cff(a0: i32) { unsafe { melee_runtime::tu::controller::HSD_PadRenewRawStatus(a0 != 0) } }
pub fn lbAudioAx_80027DBC__2be6f4() { unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80027DBC() } }
pub fn lbCardNew_CompleteNextTask__92582d() -> i32 { unsafe { melee_runtime::tu::lbcardnew::lbCardNew_CompleteNextTask() } }
pub fn lbSnap_8001D2BC__4abedd() { unsafe { melee_runtime::tu::lbsnap::lbSnap_8001D2BC() } }
pub fn lb_800192A8__02b4e0(a0: Option<unsafe extern "C" fn()>) { unsafe { crate::tu::lb_0192::lb_800192A8(a0) } }
pub fn lb_8001C600__203725() { unsafe { melee_runtime::tu::lbcardgame::lb_8001C600() } }
pub fn lb_8001CC84__76fb76() { unsafe { melee_runtime::tu::lbcardgame::lb_8001CC84() } }
pub fn THPDec_8032F8D4__2fd84b(a0: *mut u8, a1: *mut c_void /* THPDec_8032FD40_Data */) -> i32 { unsafe { melee_runtime::tu::THPDec::THPDec_8032F8D4(a0, a1 as _) } }
pub fn THPDec_8032FD40__cc10c1(a0: *mut c_void /* THPDec_8032FD40_Data */, a1: u16) -> i32 { unsafe { melee_runtime::tu::THPDec::THPDec_8032FD40(a0 as _, a1) } }
pub fn THPDec_80331340__4665bc(a0: *mut c_void /* _THPFileInfo */, a1: *mut c_void, a2: *mut c_void, a3: *mut c_void) { unsafe { melee_runtime::tu::THPDec::THPDec_80331340(a0 as _, a1, a2, a3) } }
pub fn THPDec_803313D0__1b8e06(a0: *mut c_void /* _THPFileInfo */, a1: *mut c_void, a2: *mut c_void, a3: *mut c_void, a4: u32) { unsafe { melee_runtime::tu::THPDec::THPDec_803313D0(a0 as _, a1, a2, a3, a4) } }
pub fn THPInit__5a9e11() { unsafe { melee_runtime::tu::THPDec::THPInit() } }
pub fn THPVideoDecode__873937(a0: *mut c_void, a1: *mut c_void, a2: *mut c_void /* _THPFileInfo */, a3: *mut c_void, a4: *mut c_void /* THPDec_8032FD40_Data */) -> *mut c_void /* _THPFileInfo */ { unsafe { (melee_runtime::tu::THPDec::THPVideoDecode(a0, a1, a2 as _, a3, a4 as _)) as _ } }
pub fn lbFile_80016760__a18afc(a0: *const i8, a1: *mut *mut c_void, a2: *mut u32) { unsafe { melee_runtime::tu::lbfile::lbFile_80016760(a0 as _, a1, a2 as _) } }
pub fn acosf__e9b06e(a0: f32) -> f32 { unsafe { melee_runtime::lbtrigf::acosf(a0) } }
pub fn lbVector_Angle__a15630(a0: *mut Vec, a1: *mut Vec) -> f32 { unsafe { melee_runtime::tu::lbvector::lbVector_Angle(a0, a1) } }
pub fn lbVector_Diff__3f20f8(a0: *mut Vec, a1: *mut Vec, a2: *mut Vec) -> *mut Vec { unsafe { melee_runtime::tu::lbvector::lbVector_Diff(a0, a1, a2) } }
pub fn lbVector_Normalize__748fa0(a0: *mut Vec) -> f32 { unsafe { melee_runtime::tu::lbvector::lbVector_Normalize(a0) } }
pub fn HSD_ObjAlloc__394117(a0: *mut _HSD_ObjAllocData) -> *mut c_void { unsafe { melee_runtime::tu::objalloc::HSD_ObjAlloc(a0 as _) } }
pub fn HSD_ObjAllocInit__d50812(a0: *mut _HSD_ObjAllocData, a1: u32, a2: u32) { unsafe { melee_runtime::tu::objalloc::HSD_ObjAllocInit(a0 as _, a1 as _, a2) } }
pub fn HSD_ObjFree__102255(a0: *mut _HSD_ObjAllocData, a1: *mut c_void) { unsafe { melee_runtime::tu::objalloc::HSD_ObjFree(a0 as _, a1) } }
pub fn fn_800208B0__5b119b(a0: u8) { unsafe { melee_runtime::tu::lbbgflash::fn_800208B0(a0) } }
pub fn lbBgFlash_InitState__bb0850(a0: *mut _GXColor) { unsafe { melee_runtime::tu::lbbgflash::lbBgFlash_InitState(a0 as _) } }
pub fn lb_80014498__63b487(a0: *mut ColorOverlay) { unsafe { crate::tu::lb_013B::lb_80014498(a0) } }
pub fn lb_800144C8__0c3a12(a0: *mut ColorOverlay, a1: *mut Fighter_804D653C_t, a2: i32, a3: i32) -> i32 { unsafe { (crate::tu::lb_013B::lb_800144C8(a0, a1, a2, a3)) as _ } }
pub fn Frontend_CaptureCharIcon__0cb8dd(a0: i32, a1: *mut HSD_JObj) { unsafe { crate::tu::gmfrontend::Frontend_CaptureCharIcon(a0, a1) } }
pub fn Frontend_CaptureIcon__520054(a0: i32, a1: *mut HSD_JObj) { unsafe { crate::tu::gmfrontend::Frontend_CaptureIcon(a0, a1) } }
pub fn Frontend_OnlinePick__100e6e() -> i32 { unsafe { crate::tu::gmfrontend::Frontend_OnlinePick() } }
pub fn GetNameText__a3637e(a0: i32) -> *mut i8 { unsafe { (crate::tu::mnname::GetNameText(a0)) as _ } }
pub fn HSD_SisLib_803A5E70__5f9c7d() { unsafe { melee_runtime::tu::sislib::HSD_SisLib_803A5E70() } }
pub fn HSD_SisLib_803A5FBC__b031ed() { unsafe { melee_runtime::tu::sislib::HSD_SisLib_803A5FBC() } }
pub fn HSD_SisLib_803A62A0__acd015(a0: i32, a1: *mut i8, a2: *mut i8) { unsafe { melee_runtime::tu::sislib::HSD_SisLib_803A62A0(a0, a1 as _, a2 as _) } }
pub fn HSD_SisLib_803A6478__1fa60b(a0: *mut u8, a1: *mut u8) -> *mut u8 { unsafe { melee_runtime::tu::hsd_3A64::HSD_SisLib_803A6478(a0, a1 as _) } }
pub fn HSD_SisLib_803A6530__05d28b(a0: i32, a1: i32, a2: i32) -> *mut u8 { unsafe { melee_runtime::tu::hsd_3A64::HSD_SisLib_803A6530(a0, a1, a2) } }
pub fn HSD_SisLib_803A660C__64b186(a0: i32, a1: i32, a2: i32) { unsafe { melee_runtime::tu::hsd_3A64::HSD_SisLib_803A660C(a0, a1, a2) } }
pub fn HSD_SisLib_803A746C__8945a6(a0: *mut HSD_Text, a1: i32, a2: f32, a3: f32) { unsafe { melee_runtime::tu::hsd_3A64::HSD_SisLib_803A746C(a0, a1, a2, a3) } }
pub fn HSD_SisLib_803A74F0__610e18(a0: *mut HSD_Text, a1: i32, a2: *mut _GXColor) { unsafe { melee_runtime::tu::hsd_3A64::HSD_SisLib_803A74F0(a0, a1, a2 as _) } }
pub fn IsNameListFull__2029e6() -> i32 { unsafe { (crate::tu::mnname::IsNameListFull()) as _ } }
pub fn atanf__3acb52(a0: f32) -> f32 { unsafe { melee_runtime::lbtrigf::atanf(a0) } }
pub fn gmMainLib_8015D0D8__be2386(a0: u8) -> i32 { unsafe { (crate::tu::gmmain_lib::gmMainLib_8015D0D8(a0)) as _ } }
pub fn gmMainLib_8015D200__713e80(a0: u8) -> i32 { unsafe { (crate::tu::gmmain_lib::gmMainLib_8015D200(a0)) as _ } }
pub fn gmMainLib_8015D328__645c26(a0: u8) -> i32 { unsafe { (crate::tu::gmmain_lib::gmMainLib_8015D328(a0)) as _ } }
pub fn gmMainLib_8015ECB0__963013() -> i32 { unsafe { crate::tu::gmmain_lib::gmMainLib_8015ECB0() } }
pub fn gm_80160980__e502e7(a0: u8) -> *const i8 { unsafe { (crate::tu::gm_1601::gm_80160980(a0)) as _ } }
pub fn gm_80162BD8__6ed8f2(a0: u8) -> i32 { unsafe { crate::tu::gm_1601::gm_80162BD8(a0) } }
pub fn gm_80162C48__9ae6cd() -> i32 { unsafe { crate::tu::gm_1601::gm_80162C48() } }
pub fn gm_80162D6C__172711() -> u8 { unsafe { crate::tu::gm_1601::gm_80162D6C() } }
pub fn gm_80162DD4__1c125a(a0: u8) -> i32 { unsafe { crate::tu::gm_1601::gm_80162DD4(a0) } }
pub fn gm_80162E44__1147bf() -> i32 { unsafe { crate::tu::gm_1601::gm_80162E44() } }
pub fn gm_80162F68__4e093d() -> u8 { unsafe { crate::tu::gm_1601::gm_80162F68() } }
pub fn gm_80162FD0__bab432(a0: u8) -> i32 { unsafe { crate::tu::gm_1601::gm_80162FD0(a0) } }
pub fn gm_80163040__1eec31() -> i32 { unsafe { crate::tu::gm_1601::gm_80163040() } }
pub fn gm_80163164__f46f0d() -> u8 { unsafe { crate::tu::gm_1601::gm_80163164() } }
pub fn gm_801631CC__ca58c4(a0: u8) -> u32 { unsafe { crate::tu::gm_1601::gm_801631CC(a0) } }
pub fn gm_801631F0__9c2074() -> u32 { unsafe { crate::tu::gm_1601::gm_801631F0() } }
pub fn gm_80163274__6b2c4a(a0: u8) -> u16 { unsafe { crate::tu::gm_1601::gm_80163274(a0) } }
pub fn gm_8016332C__1fdd1c(a0: u8) -> i32 { unsafe { crate::tu::gm_1601::gm_8016332C(a0) } }
pub fn gm_80163374__16b21a(a0: u8, a1: *mut u8, a2: *mut u8, a3: *mut u8, a4: *mut u8) { unsafe { crate::tu::gm_1601::gm_80163374(a0, a1, a2, a3, a4) } }
pub fn gm_801634D4__1f064c(a0: *mut u8, a1: *mut u8, a2: *mut u8, a3: *mut u8) -> i32 { unsafe { (crate::tu::gm_1601::gm_801634D4(a0, a1, a2, a3)) as _ } }
pub fn gm_8016365C__ec3f11(a0: u8) -> i32 { unsafe { (crate::tu::gm_1601::gm_8016365C(a0)) as _ } }
pub fn gm_80163690__c14dc4(a0: u8) -> i32 { unsafe { crate::tu::gm_1601::gm_80163690(a0) } }
pub fn gm_801636D8__faafb1(a0: u8, a1: *mut u8, a2: *mut u8, a3: *mut u8, a4: *mut u8) { unsafe { crate::tu::gm_1601::gm_801636D8(a0, a1, a2, a3, a4) } }
pub fn gm_80163838__97f44a(a0: *mut u8, a1: *mut u8, a2: *mut u8, a3: *mut u8) -> i32 { unsafe { (crate::tu::gm_1601::gm_80163838(a0, a1, a2, a3)) as _ } }
pub fn gm_801639C0__cad04b(a0: u8) -> i32 { unsafe { (crate::tu::gm_1601::gm_801639C0(a0)) as _ } }
pub fn gm_801639F4__890fba(a0: u8) -> i32 { unsafe { crate::tu::gm_1601::gm_801639F4(a0) } }
pub fn gm_80163A3C__deadfc(a0: u8, a1: *mut u8, a2: *mut u8, a3: *mut u8, a4: *mut u8) { unsafe { crate::tu::gm_1601::gm_80163A3C(a0, a1, a2, a3, a4) } }
pub fn gm_80163B9C__06e310(a0: *mut u8, a1: *mut u8, a2: *mut u8, a3: *mut u8) -> i32 { unsafe { (crate::tu::gm_1601::gm_80163B9C(a0, a1, a2, a3)) as _ } }
pub fn gm_801685D4__a2b2f9(a0: u8, a1: u8) -> i8 { unsafe { crate::tu::gm_1601::gm_801685D4(a0, a1) } }
pub fn gm_80168C5C__554ef2(a0: u32) { unsafe { crate::tu::gm_1601::gm_80168C5C(a0) } }
pub fn gm_80169264__5fece7(a0: u8) -> u8 { unsafe { crate::tu::gm_1601::gm_80169264(a0) } }
pub fn gm_80169290__fa9c3b(a0: u8) -> u8 { unsafe { crate::tu::gm_1601::gm_80169290(a0) } }
pub fn gm_801692BC__f7a9a4(a0: u8) -> u8 { unsafe { crate::tu::gm_1601::gm_801692BC(a0) } }
pub fn gm_Get15MinMultimanHighscore__923dcd(a0: u8) -> u16 { unsafe { crate::tu::gm_1601::gm_Get15MinMultimanHighscore(a0) } }
pub fn gm_Get15MinMultimanTotalHighscore__174f15() -> i32 { unsafe { crate::tu::gm_1601::gm_Get15MinMultimanTotalHighscore() } }
pub fn gm_Get3MinMultimanHighscore__9e8675(a0: u8) -> u16 { unsafe { crate::tu::gm_1601::gm_Get3MinMultimanHighscore(a0) } }
pub fn gm_Get3MinMultimanTotalHighscore__906f85() -> i32 { unsafe { crate::tu::gm_1601::gm_Get3MinMultimanTotalHighscore() } }
pub fn gm_GetCruelHighscore__d78ce6(a0: u8) -> i32 { unsafe { crate::tu::gm_1601::gm_GetCruelHighscore(a0) } }
pub fn gm_GetCruelTotalHighscore__ef30e1() -> i32 { unsafe { crate::tu::gm_1601::gm_GetCruelTotalHighscore() } }
pub fn gm_GetEndlessHighscore__1fa523(a0: u8) -> i32 { unsafe { crate::tu::gm_1601::gm_GetEndlessHighscore(a0) } }
pub fn gm_GetEndlessTotalHighscore__6d2632() -> i32 { unsafe { crate::tu::gm_1601::gm_GetEndlessTotalHighscore() } }
pub fn lbAudioAx_80023870__b1a847(a0: i32, a1: i32, a2: i32, a3: i32) -> i32 { unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80023870(a0, a1, a2, a3) } }
pub fn lbAudioAx_8002411C__198044(a0: i32) -> i32 { unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_8002411C(a0) } }
pub fn lb_800145F4__895731() { unsafe { crate::tu::lb_013B::lb_800145F4() } }
pub fn mnNameNew_EnterFromMnCharSel__8b7caf(a0: *mut HSD_Archive, a1: i32) { unsafe { crate::tu::mnnamenew::mnNameNew_EnterFromMnCharSel(a0, a1) } }
pub fn mn_8022BA1C__3b39ba(a0: *mut HSD_GObj) { unsafe { crate::tu::mnmain::mn_8022BA1C(a0) } }
pub fn mn_8022F138__462f12(a0: u16, a1: u16) { unsafe { crate::tu::mn_22EC::mn_8022F138(a0, a1) } }
pub fn mn_8022F218__a1ac6a() -> i32 { unsafe { (crate::tu::mn_22EC::mn_8022F218()) as _ } }
pub fn mn_8022F268__186f85() { unsafe { crate::tu::mn_22EC::mn_8022F268() } }
pub fn mn_80231804__20ee22(a0: *mut HSD_Archive, a1: i32) { unsafe { crate::tu::mnmainrule::mn_80231804(a0, a1) } }
pub fn GetNameCount__b961fe() -> i32 { unsafe { crate::tu::mnname::GetNameCount() } }
pub fn GetPersistentFighterData__578ad2(a0: SelectableCharacterKind) -> *mut FighterData { unsafe { crate::tu::gmmain_lib::GetPersistentFighterData(a0) } }
pub fn gmMainLib_GetBonusMatchTotal__02f786() -> *mut u32 { unsafe { crate::tu::gmmain_lib::gmMainLib_GetBonusMatchTotal() } }
pub fn gmMainLib_GetCoinMatchTotal__b5956d() -> *mut u32 { unsafe { crate::tu::gmmain_lib::gmMainLib_GetCoinMatchTotal() } }
pub fn gmMainLib_GetCombinedVSPlayTime__1127ef() -> *mut c_void { unsafe { (crate::tu::gmmain_lib::gmMainLib_GetCombinedVSPlayTime()) as _ } }
pub fn gmMainLib_GetKOTotal__df5440() -> *mut i32 { unsafe { crate::tu::gmmain_lib::gmMainLib_GetKOTotal() } }
pub fn gmMainLib_GetMatchResetCounter__75c307() -> *mut u32 { unsafe { crate::tu::gmmain_lib::gmMainLib_GetMatchResetCounter() } }
pub fn gmMainLib_GetPowerCount__4fdcf9() -> *mut c_void { unsafe { (crate::tu::gmmain_lib::gmMainLib_GetPowerCount()) as _ } }
pub fn gmMainLib_GetSelfDestructTotal__53cf84() -> *mut c_void { unsafe { (crate::tu::gmmain_lib::gmMainLib_GetSelfDestructTotal()) as _ } }
pub fn gmMainLib_GetSingleplayerTime__730b1b() -> *mut c_void { unsafe { (crate::tu::gmmain_lib::gmMainLib_GetSingleplayerTime()) as _ } }
pub fn gmMainLib_GetStockMatchTotal__5e0d06() -> *mut u32 { unsafe { crate::tu::gmmain_lib::gmMainLib_GetStockMatchTotal() } }
pub fn gmMainLib_GetTimeMatchTotal__16c4ef() -> *mut u32 { unsafe { crate::tu::gmmain_lib::gmMainLib_GetTimeMatchTotal() } }
pub fn gmMainLib_GetTotalDamage__6cca88() -> *mut c_void { unsafe { (crate::tu::gmmain_lib::gmMainLib_GetTotalDamage()) as _ } }
pub fn gmMainLib_GetVsPlayContestants__57546a() -> *mut c_void { unsafe { (crate::tu::gmmain_lib::gmMainLib_GetVsPlayContestants()) as _ } }
pub fn gmMainLib_GetVsPlayTime__b5e5c4() -> *mut c_void { unsafe { (crate::tu::gmmain_lib::gmMainLib_GetVsPlayTime()) as _ } }
pub fn gm_80160B40__54949f(a0: *mut HSD_Text, a1: u8, a2: u8) { unsafe { crate::tu::gm_1601::gm_80160B40(a0, a1, a2) } }
pub fn gm_801641CC__003261(a0: u8) -> u16 { unsafe { crate::gm::gm_801641CC(a0) } }
pub fn gm_GetPlayTime__729b79() -> i32 { unsafe { crate::tu::gm_1601::gm_GetPlayTime() } }
pub fn gm_GetPowerTime__6cb0de() -> *mut i32 { unsafe { crate::tu::gmmain_lib::gm_GetPowerTime() } }
pub fn mnDiagram_GetFighterTotalFalls__925ac9(a0: u8) -> i32 { unsafe { crate::tu::mndiagram::mnDiagram_GetFighterTotalFalls(a0) } }
pub fn mnDiagram_GetFighterTotalKOs__14ab9d(a0: u8) -> i32 { unsafe { crate::tu::mndiagram::mnDiagram_GetFighterTotalKOs(a0) } }
pub fn mn_8022ED6C__f73cae(a0: *mut HSD_JObj, a1: *mut AnimLoopSettings) -> f32 { unsafe { crate::tu::mn_22EC::mn_8022ED6C(a0, a1) } }
pub fn lbAudioAx_80027AB0__27430d(a0: i32) { unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80027AB0(a0) } }
pub fn mn_8022EC18__bee2c1(a0: *mut HSD_JObj, a1: *mut AnimLoopSettings, a2: HSD_TypeMask) -> f32 { unsafe { crate::tu::mn_22EC::mn_8022EC18(a0, a1, a2) } }
pub fn mn_8022EE84__fdce2b(a0: *mut HSD_JObj, a1: *mut AnimLoopSettings, a2: HSD_TypeMask) -> f32 { unsafe { crate::tu::mn_22EC::mn_8022EE84(a0, a1, a2) } }
pub fn mn_8022EFD8__c1f371(a0: *mut HSD_JObj, a1: *mut AnimLoopSettings) -> f32 { unsafe { crate::tu::mn_22EC::mn_8022EFD8(a0, a1) } }
pub fn mn_8022F298__0e9b78(a0: *mut HSD_JObj) -> f32 { unsafe { crate::tu::mn_22EC::mn_8022F298(a0) } }
pub fn mn_8022F3D8__9f7fac(a0: *mut HSD_JObj, a1: u8, a2: HSD_TypeMask) { unsafe { crate::tu::mn_22EC::mn_8022F3D8(a0, a1, a2) } }
pub fn gm_801603B0__bf3f82() { unsafe { crate::tu::gm_1601::gm_801603B0() } }
pub fn gm_801641E4__a151c2(a0: u8, a1: u8) { unsafe { crate::tu::gm_1601::gm_801641E4(a0, a1) } }
pub fn gm_IsStageUnlocked__19a514(a0: u16) -> i32 { unsafe { (crate::tu::gm_1601::gm_IsStageUnlocked(a0)) as _ } }
pub fn gm_801647D0__084fbc() { unsafe { crate::tu::gm_1601::gm_801647D0() } }
pub fn gm_8016505C__87cdec() { unsafe { crate::tu::gm_1601::gm_8016505C() } }
pub fn gm_801729EC__181596() { unsafe { crate::tu::gm_16f1::gm_801729EC() } }
pub fn gm_80174238__25ce87() { unsafe { crate::tu::gm_1736::gm_80174238() } }
pub fn gm_801A3EF4__94d282() { unsafe { crate::tu::gm_1A36::gm_801A3EF4() } }
pub fn gmMainLib_8015DB80__095b2c() { unsafe { crate::tu::gmmain_lib::gmMainLib_8015DB80() } }
pub fn gmMainLib_8015EEC8__753a45() { unsafe { crate::tu::gmmain_lib::gmMainLib_8015EEC8() } }
pub fn gmMainLib_8015F150__b503e7() { unsafe { crate::tu::gmmain_lib::gmMainLib_8015F150() } }
pub fn gmMainLib_8015F260__097b6f() { unsafe { crate::tu::gmmain_lib::gmMainLib_8015F260() } }
pub fn gmMainLib_8015F464__37a6c0() { unsafe { crate::tu::gmmain_lib::gmMainLib_8015F464() } }
pub fn gmMainLib_8015F490__cf404e() { unsafe { crate::tu::gmmain_lib::gmMainLib_8015F490() } }
pub fn gmMainLib_8015F4BC__88898e() { unsafe { crate::tu::gmmain_lib::gmMainLib_8015F4BC() } }
pub fn gmMainLib_8015F4E8__7f030c() -> u32 { unsafe { crate::tu::gmmain_lib::gmMainLib_8015F4E8() } }
pub fn gmMainLib_8015F588__d30fee(a0: i32) { unsafe { crate::tu::gmmain_lib::gmMainLib_8015F588(a0 != 0) } }
pub fn gmMainLib_8015FBA4__bdb1fc() { unsafe { crate::tu::gmmain_lib::gmMainLib_8015FBA4() } }
pub fn lbLang_GetSavedLanguage__c4684f() -> enum_t { unsafe { melee_runtime::tu::lblanguage::lbLang_GetSavedLanguage() } }
pub fn lbLang_SetSavedLanguage__3000cb(a0: enum_t) { unsafe { melee_runtime::tu::lblanguage::lbLang_SetSavedLanguage(a0) } }
pub fn Toy_80311960__db40d0() { unsafe { melee_items::tu::toy::Toy_80311960() } }
pub fn HSD_MemAlloc__057bba(a0: ssize_t) -> *mut c_void { unsafe { melee_runtime::tu::memory::HSD_MemAlloc(a0 as _) } }
pub fn gmMainLib_8015F4F4__9eed36(a0: u8) { unsafe { crate::tu::gmmain_lib::gmMainLib_8015F4F4(a0) } }
pub fn mn_IsFighterUnlocked__2c05f5(a0: SelectableCharacterKind) -> i32 { unsafe { (crate::tu::mnmain::mn_IsFighterUnlocked(a0)) as _ } }
pub fn mn_GetDigitAt__4d1d41(a0: i32, a1: i32) -> i32 { unsafe { crate::mn::mn_GetDigitAt(a0, a1) } }
pub fn mn_GetDigitCount__861d08(a0: i32) -> i32 { unsafe { crate::mn::mn_GetDigitCount(a0) } }
pub fn mnDiagram2_Init__f8f7d5() { unsafe { crate::tu::mndiagram2::mnDiagram2_Init() } }
pub fn mnDiagram3_Init__fc06c3(a0: *mut ::core::ffi::c_void) { unsafe { crate::tu::mndiagram3::mnDiagram3_Init(a0) } }
pub fn gm_80160C90__c185a6(a0: *mut HSD_Text, a1: u8, a2: u8) { unsafe { crate::tu::gm_1601::gm_80160C90(a0, a1, a2) } }
pub fn powi__640168(a0: i32, a1: i32) -> i32 { unsafe { crate::lb::powi(a0, a1) } }
pub fn HSD_Free__bb56fc(a0: *mut ::core::ffi::c_void) { unsafe { melee_runtime::tu::memory::HSD_Free(a0) } }
pub fn HSD_MemAlloc__e04cd7(a0: i32) -> *mut ::core::ffi::c_void { unsafe { melee_runtime::tu::memory::HSD_MemAlloc(a0) } }
pub fn mnDiagram_GetFighterByIndex__32c150(a0: i32) -> u8 { unsafe { crate::tu::mndiagram::mnDiagram_GetFighterByIndex(a0) } }
pub fn mnDiagram_GetNameByIndex__939c80(a0: i32) -> u8 { unsafe { crate::tu::mndiagram::mnDiagram_GetNameByIndex(a0) } }
pub fn mnDiagram_IsDistanceOverflow__1e9ea6(a0: u32) -> i32 { unsafe { crate::tu::mndiagram::mnDiagram_IsDistanceOverflow(a0) } }
pub fn mnDiagram_ConvertDistanceForDisplay__b66d97(a0: u32) -> u32 { unsafe { crate::tu::mndiagram::mnDiagram_ConvertDistanceForDisplay(a0) } }
pub fn mnDiagram_GetHitPercentage__c1e6b8(a0: u8, a1: u8) -> i32 { unsafe { crate::tu::mndiagram::mnDiagram_GetHitPercentage(a0, a1) } }
pub fn mnDiagram_GetPlayPercentage__9d3e1b(a0: u8, a1: u8) -> i32 { unsafe { crate::tu::mndiagram::mnDiagram_GetPlayPercentage(a0, a1) } }
pub fn mnDiagram_GetAveragePlayerCount__575a9b(a0: u8, a1: u8) -> i32 { unsafe { crate::tu::mndiagram::mnDiagram_GetAveragePlayerCount(a0, a1) } }
pub fn mnDiagram_GetNameTotalKOs__ddd3d2(a0: u8) -> i32 { unsafe { crate::tu::mndiagram::mnDiagram_GetNameTotalKOs(a0) } }
pub fn mnDiagram_GetNameTotalFalls__ae5005(a0: u8) -> i32 { unsafe { crate::tu::mndiagram::mnDiagram_GetNameTotalFalls(a0) } }
pub fn mnDiagram_FormatDecimalNumber__95f539(a0: *mut i8, a1: u32, a2: i32) { unsafe { crate::tu::mndiagram::mnDiagram_FormatDecimalNumber(a0, a1, a2) } }
pub fn mnDiagram_FormatTime__bb5e42(a0: *mut i8, a1: i32) { unsafe { crate::tu::mndiagram::mnDiagram_FormatTime(a0, a1) } }
pub fn mnDiagram_IntToStr__37a31a(a0: *mut i8, a1: u32) { unsafe { crate::tu::mndiagram::mnDiagram_IntToStr(a0, a1) } }
pub fn mnDiagram_GetPrevNameIndex__870de9(a0: i32) -> u8 { unsafe { crate::tu::mndiagram::mnDiagram_GetPrevNameIndex(a0) } }
pub fn mnDiagram_GetNextNameIndex__4c2dcf(a0: i32) -> u8 { unsafe { crate::tu::mndiagram::mnDiagram_GetNextNameIndex(a0) } }
pub fn mnDiagram_GetPrevFighterIndex__799a94(a0: i32) -> u8 { unsafe { crate::tu::mndiagram::mnDiagram_GetPrevFighterIndex(a0) } }
pub fn mnDiagram_GetNextFighterIndex__c8f522(a0: i32) -> u8 { unsafe { crate::tu::mndiagram::mnDiagram_GetNextFighterIndex(a0) } }
pub fn mnDiagram_GetNamePlayTimeByFighter__347931(a0: i32, a1: SelectableCharacterKind) -> u32 { unsafe { crate::tu::mndiagram::mnDiagram_GetNamePlayTimeByFighter(a0, a1) } }
pub fn mnDiagram_GetRankedFighterForName__652f6b(a0: i32, a1: i32, a2: Option< unsafe extern "C" fn(i32, i32) -> u32, >) -> i32 { unsafe { crate::tu::mndiagram::mnDiagram_GetRankedFighterForName(a0, a1, a2) } }
pub fn mnDiagram_GetLeastPlayedFighter__509963(a0: u8) -> u8 { unsafe { crate::tu::mndiagram::mnDiagram_GetLeastPlayedFighter(a0) } }
pub fn mnDiagram_CountUnlockedFighters__c82518() -> i32 { unsafe { crate::tu::mndiagram::mnDiagram_CountUnlockedFighters() } }
pub fn mnDiagram_CreateFighterIcon__dd1690(a0: i32, a1: i32) -> *mut HSD_JObj { unsafe { crate::tu::mndiagram::mnDiagram_CreateFighterIcon(a0, a1) } }
pub fn mnDiagram_Init__3b569c(a0: u8, a1: u8) { unsafe { crate::tu::mndiagram::mnDiagram_Init(a0, a1) } }
pub fn mnDiagram3_Init__319022(a0: *mut c_void) { unsafe { crate::tu::mndiagram3::mnDiagram3_Init(a0) } }
pub fn GetNameText__db8135(a0: u8) -> *mut i8 { unsafe { (crate::tu::mnname::GetNameText(a0 as _)) as _ } }
pub fn HSD_GObj_JObjCallback__b37ff8(a0: *mut HSD_GObj, a1: ::core::ffi::c_int) { unsafe { melee_runtime::tu::gobj::HSD_GObj_JObjCallback(a0, a1) } }
pub fn lbAudioAx_80024030__52c54d(a0: ::core::ffi::c_int) { unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80024030(a0) } }
pub fn mn_8022ED6C__a278c3(a0: *mut HSD_JObj, a1: *mut AnimLoopSettings) -> ::core::ffi::c_float { unsafe { crate::tu::mn_22EC::mn_8022ED6C(a0, a1) } }
pub fn HSD_SisLib_803A6754__6086ec(a0: ::core::ffi::c_int, a1: ::core::ffi::c_int) -> *mut HSD_Text { unsafe { melee_runtime::tu::hsd_3A64::HSD_SisLib_803A6754(a0, a1) } }
pub fn HSD_SisLib_803A5ACC__97831a(a0: ::core::ffi::c_int, a1: i32, a2: f32, a3: f32, a4: f32, a5: f32, a6: f32) -> *mut HSD_Text { unsafe { melee_runtime::tu::sislib::HSD_SisLib_803A5ACC(a0, a1, a2, a3, a4, a5, a6) } }
pub fn mnDiagram_FormatDecimalNumber__8d7594(a0: *mut ::core::ffi::c_char, a1: u32, a2: ::core::ffi::c_int) { unsafe { crate::tu::mndiagram::mnDiagram_FormatDecimalNumber(a0 as _, a1, a2) } }
pub fn mnDiagram_FormatTime__d987de(a0: *mut ::core::ffi::c_char, a1: i32) { unsafe { crate::tu::mndiagram::mnDiagram_FormatTime(a0 as _, a1) } }
pub fn mnDiagram_IntToStr__e6f436(a0: *mut ::core::ffi::c_char, a1: u32) { unsafe { crate::tu::mndiagram::mnDiagram_IntToStr(a0 as _, a1) } }
pub fn mnDiagram_CreateFighterIcon__68caba(a0: ::core::ffi::c_int, a1: ::core::ffi::c_int) -> *mut HSD_JObj { unsafe { crate::tu::mndiagram::mnDiagram_CreateFighterIcon(a0, a1) } }
pub fn mnDiagram2_IsTimeStat__252316(a0: u8) -> i32 { unsafe { (crate::mn::mnDiagram2_IsTimeStat(a0)) as _ } }
pub fn mnDiagram2_IsDistanceStat__cf39d9(a0: u8) -> i32 { unsafe { (crate::mn::mnDiagram2_IsDistanceStat(a0)) as _ } }
pub fn mnDiagram2_IsPercentageStat__405662(a0: u8) -> i32 { unsafe { (crate::mn::mnDiagram2_IsPercentageStat(a0)) as _ } }
pub fn mnDiagram2_IsIconOnlyStat__ccedb4(a0: u8) -> i32 { unsafe { (crate::mn::mnDiagram2_IsIconOnlyStat(a0)) as _ } }
pub fn mnDiagram2_GetStatValue__6f5f9f(a0: u8, a1: u8, a2: u8) -> ::core::ffi::c_int { unsafe { crate::tu::mndiagram2::mnDiagram2_GetStatValue(a0, a1, a2) } }
pub fn mnDiagram2_GetRankedFighter__bdb415(a0: u8, a1: u8) -> u8 { unsafe { crate::tu::mndiagram2::mnDiagram2_GetRankedFighter(a0, a1) } }
pub fn mnDiagram2_GetRankedName__8edd6f(a0: u8, a1: u8) -> u8 { unsafe { crate::tu::mndiagram2::mnDiagram2_GetRankedName(a0, a1) } }
pub fn mnDiagram2_GetAggregatedFighterRank__7dd640(a0: *mut mnDiagram2_SortEntry, a1: u8, a2: u8) { unsafe { crate::tu::mndiagram2::mnDiagram2_GetAggregatedFighterRank(a0, a1, a2) } }
pub fn mnDiagram2_ClearDetailView__3b2be6(a0: *mut HSD_GObj) { unsafe { crate::tu::mndiagram2::mnDiagram2_ClearDetailView(a0) } }
pub fn GetNameText__9c44ce(a0: ::core::ffi::c_int) -> *mut ::core::ffi::c_char { unsafe { crate::tu::mnname::GetNameText(a0) } }
pub fn GetNameCount__e5158d() -> ::core::ffi::c_int { unsafe { crate::tu::mnname::GetNameCount() } }
pub fn mn_802295AC__671d2a() -> u8 { unsafe { (crate::tu::mnmain::mn_802295AC()) as _ } }
pub fn mn_80229860__033db5(a0: i8) { unsafe { crate::tu::mnmain::mn_80229860(a0) } }
pub fn mn_8022EC18__c43f51(a0: *mut HSD_JObj, a1: *mut AnimLoopSettings, a2: HSD_TypeMask) -> ::core::ffi::c_float { unsafe { crate::tu::mn_22EC::mn_8022EC18(a0, a1, a2) } }
pub fn gm_801677E8__7189c6(a0: i8) { unsafe { crate::tu::gm_1601::gm_801677E8(a0) } }
pub fn gm_801BEB74__1db024(a0: u8) { unsafe { crate::tu::gmevent::gm_801BEB74(a0) } }
pub fn gm_801BEB8C__797c45(a0: u8) -> i32 { unsafe { (crate::tu::gmevent::gm_801BEB8C(a0)) as _ } }
pub fn gm_801BEBA8__dc2738(a0: u8) -> u8 { unsafe { crate::tu::gmevent::gm_801BEBA8(a0) } }
pub fn gm_801BEBF8__50ad08(a0: u8) -> u8 { unsafe { crate::tu::gmevent::gm_801BEBF8(a0) } }
pub fn gmMainLib_8015CEFC__f899c1(a0: ::core::ffi::c_int) -> i32 { unsafe { (crate::tu::gmmain_lib::gmMainLib_8015CEFC(a0)) as _ } }
pub fn HSD_CObjAlloc__2bceb5() -> *mut HSD_CObj { unsafe { melee_runtime::tu::cobj::HSD_CObjAlloc() } }
pub fn HSD_CObjSetFar__5ef574(a0: *mut HSD_CObj, a1: f32) { unsafe { melee_runtime::tu::cobj::HSD_CObjSetFar(a0, a1) } }
pub fn HSD_CObjSetNear__29587b(a0: *mut HSD_CObj, a1: f32) { unsafe { melee_runtime::tu::cobj::HSD_CObjSetNear(a0, a1) } }
pub fn HSD_CObjSetOrtho__9b799b(a0: *mut HSD_CObj, a1: f32, a2: f32, a3: f32, a4: f32) { unsafe { melee_runtime::tu::cobj::HSD_CObjSetOrtho(a0, a1, a2, a3, a4) } }
pub fn HSD_CObjSetViewport__55aa72(a0: *mut HSD_CObj, a1: *mut _HSD_RectS16) { unsafe { melee_runtime::tu::cobj::HSD_CObjSetViewport(a0, a1 as _) } }
pub fn gm_801ACC94__d71db4() -> *mut c_void { unsafe { crate::tu::gmhowto::gm_801ACC94() } }
pub fn lbAudioAx_80024E50__461466(a0: i32) { unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80024E50(a0 != 0) } }
pub fn lbMthp_8001F410__492048(a0: *const i8, a1: *mut u32, a2: *mut c_void, a3: u32, a4: i32) { unsafe { melee_runtime::tu::lbmthp::lbMthp_8001F410(a0 as _, a1, a2, a3 as _, a4) } }
pub fn lbMthp_8001F578__cdae63() { unsafe { melee_runtime::tu::lbmthp::lbMthp_8001F578() } }
pub fn lbMthp_8001F604__d79d45() -> i32 { unsafe { melee_runtime::tu::lbmthp::lbMthp_8001F604() } }
pub fn lbMthp_8001F624__0c09ac(a0: *mut HSD_GObj, a1: i32, a2: i32) -> *mut HSD_SObj { unsafe { melee_runtime::tu::lbmthp::lbMthp_8001F624(a0, a1, a2) } }
pub fn lbMthp_8001F67C__b1c012(a0: *mut HSD_GObj, a1: i32) { unsafe { melee_runtime::tu::lbmthp::lbMthp_8001F67C(a0, a1) } }
pub fn mn_8022BD6C__fc17af() { unsafe { crate::tu::mnmain::mn_8022BD6C() } }
pub fn mn_8022BD8C__db9576() { unsafe { crate::tu::mnmain::mn_8022BD8C() } }
pub fn mn_80229B2C__cb2183() -> *mut HSD_GObj { unsafe { crate::tu::mnmain::mn_80229B2C() } }
pub fn mn_80229DC0__35b68e() -> *mut HSD_GObj { unsafe { crate::tu::mnmain::mn_80229DC0() } }
pub fn mn_8022BCF8__af4c64() -> *mut HSD_GObj { unsafe { crate::tu::mnmain::mn_8022BCF8() } }
pub fn mn_8022BE34__0ceef2() -> *mut HSD_GObj { unsafe { crate::tu::mnmain::mn_8022BE34() } }
pub fn mn_8022BEDC__95f428(a0: *mut HSD_GObj) { unsafe { crate::tu::mnmain::mn_8022BEDC(a0) } }
pub fn mn_8022C304__3bbc69() { unsafe { crate::tu::mnmain::mn_8022C304() } }
pub fn mn_8022EBDC__a19b61() { unsafe { crate::tu::mnmain::mn_8022EBDC() } }
pub fn mnCharSel_802640A0__c8d517() -> i32 { unsafe { crate::tu::mncharsel::mnCharSel_802640A0() } }
pub fn CreateNameAtIndex__5eec5d(a0: i32) { unsafe { crate::tu::mnname::CreateNameAtIndex(a0) } }
pub fn gm_80190FE4__f31b7e(a0: i32) { unsafe { crate::tu::gmtou_0::gm_80190FE4(a0) } }
pub fn HSD_MemAlloc__849f1b(a0: isize) -> *mut c_void { unsafe { melee_runtime::tu::memory::HSD_MemAlloc(a0 as _) } }
pub fn lb_8001204C__f0bab9(a0: *mut HSD_JObj, a1: *mut *mut HSD_JObj, a2: *mut u16, a3: i32) -> i32 { unsafe { melee_runtime::tu::lbspdisplay::lb_8001204C(a0, a1, a2, a3) } }
pub fn mn_8022F4CC__d43938() { unsafe { crate::tu::mn_22EC::mn_8022F4CC() } }
pub fn mn_8023164C__7a202c() { unsafe { crate::tu::mnmainrule::mn_8023164C() } }
pub fn HSD_SisLib_803A7664__877b04(a0: *mut HSD_Text) { unsafe { melee_runtime::tu::hsd_3A64::HSD_SisLib_803A7664(a0) } }
pub fn lb_8001CDB4__c8eb69() { unsafe { melee_runtime::tu::lbcardgame::lb_8001CDB4() } }
pub fn lbSnap_8001D338__5d6cf2(a0: i32) -> i32 { unsafe { melee_runtime::tu::lbsnap::lbSnap_8001D338(a0) } }
pub fn lbSnap_8001D350__ca2595(a0: i32) -> i32 { unsafe { melee_runtime::tu::lbsnap::lbSnap_8001D350(a0) } }
pub fn lbSnap_8001D394__01e542(a0: i32) -> i32 { unsafe { melee_runtime::tu::lbsnap::lbSnap_8001D394(a0) } }
pub fn lbSnap_8001D3B0__264eec(a0: i32) -> i32 { unsafe { melee_runtime::tu::lbsnap::lbSnap_8001D3B0(a0) } }
pub fn lbSnap_8001D3CC__60ff2c(a0: i32) -> i32 { unsafe { melee_runtime::tu::lbsnap::lbSnap_8001D3CC(a0) } }
pub fn lbSnap_8001D3E8__7ac81e(a0: i32, a1: i32) -> i32 { unsafe { melee_runtime::tu::lbsnap::lbSnap_8001D3E8(a0, a1) } }
pub fn lbSnap_8001D40C__e67ee9(a0: i32) -> i32 { unsafe { melee_runtime::tu::lbsnap::lbSnap_8001D40C(a0) } }
pub fn lbSnap_8001D5FC__4de92a(a0: i32, a1: i32) -> i32 { unsafe { melee_runtime::tu::lbsnap::lbSnap_8001D5FC(a0, a1) } }
pub fn lbSnap_8001D7B0__62c6a7(a0: i32, a1: i32, a2: i32) -> i32 { unsafe { melee_runtime::tu::lbsnap::lbSnap_8001D7B0(a0, a1, a2) } }
pub fn lbSnap_8001DE8C__fb044e(a0: *mut c_void) -> i32 { unsafe { melee_runtime::tu::lbsnap::lbSnap_8001DE8C(a0) } }
pub fn lbSnap_8001DF6C__d1d2db(a0: i32) -> i32 { unsafe { melee_runtime::tu::lbsnap::lbSnap_8001DF6C(a0) } }
pub fn lbSnap_8001E058__14a0b6(a0: i32, a1: i32) -> i32 { unsafe { melee_runtime::tu::lbsnap::lbSnap_8001E058(a0, a1) } }
pub fn lbAudioAx_80024BD0__eb4f5a() -> i32 { unsafe { (melee_runtime::tu::lbaudio_ax::lbAudioAx_80024BD0()) as _ } }
pub fn lbAudioAx_80024C08__87c4fa(a0: enum_t) -> i32 { unsafe { (melee_runtime::tu::lbaudio_ax::lbAudioAx_80024C08(a0 as _)) as _ } }
pub fn lbAudioAx_80023090__11c684(a0: i32) -> i32 { unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80023090(a0) } }
pub fn lbAudioAx_80023968__138b6f(a0: i32) -> i32 { unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80023968(a0) } }
pub fn lbAudioAx_80023A44__d67545(a0: i32, a1: i32) -> i32 { unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80023A44(a0, a1) } }
pub fn lbAudioAx_80023B24__808380(a0: i32) -> i32 { unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80023B24(a0) } }
pub fn lbAudioAx_800245F4__23a8b4(a0: i32) -> i32 { unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_800245F4(a0) } }
pub fn lbAudioAx_80024614__7a0894(a0: i32) { unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80024614(a0) } }
pub fn mn_8022EAE0__7ca08a(a0: *mut HSD_GObj) { unsafe { crate::tu::mnmain::mn_8022EAE0(a0) } }
pub fn HSD_AObjStopAnim__99638d(a0: *mut HSD_AObj, a1: *mut c_void, a2: HSD_ObjUpdateFunc) { unsafe { melee_runtime::tu::aobj::HSD_AObjStopAnim(a0, a1, a2) } }
pub fn lb_8000C2F8__a15864(a0: *mut HSD_JObj, a1: *mut HSD_JObj) { unsafe { melee_runtime::tu::lb_00B0::lb_8000C2F8(a0, a1) } }
pub fn HSD_PadRumbleRemoveAll__95e5b6() { unsafe { melee_runtime::tu::rumble::HSD_PadRumbleRemoveAll() } }
pub fn HSD_PadRumbleOffN__6901ea(a0: u8) { unsafe { melee_runtime::tu::rumble::HSD_PadRumbleOffN(a0) } }
pub fn gm_80190EA4__7bb941() { unsafe { crate::tu::gmtou_0::gm_80190EA4() } }
pub fn HSD_CObjGetCurrent__ad58db() -> * mut HSD_CObj { unsafe { melee_runtime::tu::cobj::HSD_CObjGetCurrent() } }
pub fn HSD_ObjFree__bf3ee9(a0: * mut HSD_ObjAllocData, a1: * mut :: core :: ffi :: c_void) { unsafe { melee_runtime::tu::objalloc::HSD_ObjFree(a0, a1) } }
pub fn HSD_GObj_80390EB8__ae201c(a0: s32) -> :: core :: ffi :: c_uint { unsafe { melee_runtime::gobj::hsd_gobj_80390eb8(a0) } }
pub fn HSD_GObj_JObjCallback__392d18(a0: * mut HSD_GObj, a1: :: core :: ffi :: c_int) { unsafe { melee_runtime::tu::gobj::HSD_GObj_JObjCallback(a0, a1) } }
pub fn ftKb_MexKirbyLike__c2b696(a0: :: core :: ffi :: c_int) -> :: core :: ffi :: c_int { unsafe { melee_chara::tu::ftkirby::ftKb_MexKirbyLike(a0) } }
pub fn it_8026B40C__05955e(a0: * mut Article, a1: s32) { unsafe { melee_items::tu::it_26B1::it_8026B40C(a0, a1) } }
pub fn cosf__610bf5(a0: :: core :: ffi :: c_float) -> :: core :: ffi :: c_float { unsafe { melee_runtime::gekko::cosf(a0) } }
pub fn sinf__6f60bc(a0: :: core :: ffi :: c_float) -> :: core :: ffi :: c_float { unsafe { melee_runtime::gekko::sinf(a0) } }
pub fn HSD_ArchiveParse__d29071(a0: * mut HSD_Archive, a1: * mut :: core :: ffi :: c_uchar, a2: usize) -> s32 { unsafe { melee_runtime::tu::archive::HSD_ArchiveParse(a0, a1 as _, a2) } }
pub fn HSD_ArchiveGetPublicAddress__8ced5d(a0: * mut HSD_Archive, a1: * const :: core :: ffi :: c_char) -> * mut :: core :: ffi :: c_void { unsafe { melee_runtime::tu::archive::HSD_ArchiveGetPublicAddress(a0, a1 as _) } }
pub fn HSD_ArchiveGetExtern__36350c(a0: * mut HSD_Archive, a1: :: core :: ffi :: c_int) -> * mut :: core :: ffi :: c_char { unsafe { (melee_runtime::tu::archive::HSD_ArchiveGetExtern(a0, a1)) as _ } }
pub fn HSD_ArchiveLocateExtern__a293e4(a0: * mut HSD_Archive, a1: * const :: core :: ffi :: c_char, a2: * mut :: core :: ffi :: c_void) { unsafe { melee_runtime::tu::archive::HSD_ArchiveLocateExtern(a0, a1 as _, a2) } }
pub fn ftCo_800B3E04__214e01(a0: * mut Fighter) { unsafe { melee_fighter::tu::ftcmdscript::ftco_800b3e04(&mut *a0) } }
pub fn ftCo_800B49F4__82f580(a0: * mut Fighter) { unsafe { melee_fighter::tu::ftcmdscript::ftco_800b49f4(&mut *a0) } }
pub fn ftCo_800B4A78__b296ba(a0: * mut Fighter) { unsafe { melee_fighter::tu::ftcmdscript::ftco_800b4a78(&mut *a0) } }
pub fn ftCommon_8007D5D4__dfe1c5(a0: * mut Fighter) { unsafe { melee_fighter::ftcommon2::ftcommon_8007d5d4(&mut *a0) } }
pub fn ftCommon_8007D6A4__198fd1(a0: * mut Fighter) { unsafe { melee_fighter::ftcommon3::ftcommon_8007d6a4(&mut *a0) } }
pub fn ftCommon_8007D7FC__b6f6a2(a0: * mut Fighter) { unsafe { melee_fighter::ftcommon3::ftcommon_8007d7fc(&mut *a0) } }
pub fn ftCommon_8007F804__b9b049(a0: * mut Fighter) -> MtxPtr { unsafe { melee_fighter::ftcommon2::ftcommon_8007f804(&mut *a0) } }
pub fn ftCamera_80076064__ed1a88(a0: * mut Fighter) { unsafe { melee_fighter::tu::ftcamera::ftcamera_80076064(&mut *a0) } }
pub fn ftCo_800D105C__a3a0b4(a0: * mut Fighter_GObj) { unsafe { melee_fighter::tu::ftchangeparam::ftco_800d105c(a0 as _) } }
pub fn HSD_SListRemove__e27f25(a0: * mut HSD_SList) -> * mut HSD_SList { unsafe { melee_runtime::tu::list::HSD_SListRemove(a0) } }
pub fn HSD_JObjUnref__cb2b18(a0: * mut HSD_JObj) { unsafe { melee_runtime::tu::jobj::HSD_JObjUnref(a0) } }
pub fn HSD_JObjLoadJoint__7fbd4c(a0: * mut HSD_Joint) -> * mut HSD_JObj { unsafe { melee_runtime::tu::jobj::HSD_JObjLoadJoint(a0) } }
pub fn HSD_JObjAlloc__ec115b() -> * mut HSD_JObj { unsafe { melee_runtime::tu::jobj::HSD_JObjAlloc() } }
pub fn lbColl_80008688__ab6b4d(a0: * mut HitCapsule, a1: :: core :: ffi :: c_int, a2: * mut :: core :: ffi :: c_void) -> :: core :: ffi :: c_int { unsafe { (melee_runtime::tu::lbcollision::lbColl_80008688(a0, a1, a2)) as _ } }
pub fn lbColl_80009F54__226aa1(a0: * mut HitCapsule, a1: :: core :: ffi :: c_uint, a2: :: core :: ffi :: c_float) -> :: core :: ffi :: c_int { unsafe { (melee_runtime::tu::lbcollision::lbColl_80009F54(a0, a1, a2)) as _ } }
pub fn lbColl_8000A244__eb27ad(a0: * mut HurtCapsule, a1: :: core :: ffi :: c_uint, a2: * mut [:: core :: ffi :: c_float ; 4], a3: :: core :: ffi :: c_float) -> :: core :: ffi :: c_int { unsafe { (melee_runtime::tu::lbcollision::lbColl_8000A244(a0, a1, a2 as _, a3)) as _ } }
pub fn gm_GetCurrentSceneIndex__e366e9() -> :: core :: ffi :: c_uchar { unsafe { crate::tu::gm_1A3F::gm_GetCurrentSceneIndex() } }
pub fn gm_GetCurrentGameMode__8510a2() -> :: core :: ffi :: c_uchar { unsafe { crate::tu::gm_1A3F::gm_GetCurrentGameMode() } }
pub fn Ground_StageSlotDataFile__219c98(a0: :: core :: ffi :: c_int) -> * const :: core :: ffi :: c_char { unsafe { (melee_world::tu::ground::Ground_StageSlotDataFile(a0)) as _ } }
pub fn Ground_StageSlotProofCallbacks__175716() -> * mut StageCallbacks { unsafe { melee_world::tu::grlast::Ground_StageSlotProofCallbacks() } }
pub fn Ground_StageSlotJoints__e0ae37(a0: :: core :: ffi :: c_int, a1: * mut :: core :: ffi :: c_uint) -> * const GrJoint { unsafe { melee_world::tu::ground::Ground_StageSlotJoints(a0, a1 as _) } }
pub fn Ground_801C0498__19f0de() -> :: core :: ffi :: c_float { unsafe { melee_world::tu::ground::Ground_801C0498() } }
pub fn Ground_GetStageGObj__6785f9(a0: :: core :: ffi :: c_int) -> * mut Ground_GObj { unsafe { melee_world::tu::ground::Ground_GetStageGObj(a0) } }
pub fn Ground_801C1DD4__98f81b() -> :: core :: ffi :: c_int { unsafe { melee_world::tu::ground::Ground_801C1DD4() } }
pub fn Ground_801C20D0__6446e8() -> :: core :: ffi :: c_float { unsafe { melee_world::tu::ground::Ground_801C20D0() } }
pub fn Ground_801C2CF4__d74f32(a0: s32) -> * mut HSD_JObj { unsafe { melee_world::tu::ground::Ground_801C2CF4(a0) } }
pub fn Ground_801C2D24__23d771(a0: enum_t, a1: * mut Vec) -> :: core :: ffi :: c_int { unsafe { melee_world::tu::ground::Ground_801C2D24(a0 as _, a1 as _) } }
pub fn Ground_801C38BC__09c5a5(a0: :: core :: ffi :: c_float, a1: :: core :: ffi :: c_float) { unsafe { melee_world::tu::ground::Ground_801C38BC(a0, a1) } }
pub fn Ground_801C38D0__d9290e(a0: :: core :: ffi :: c_float, a1: :: core :: ffi :: c_float, a2: :: core :: ffi :: c_float, a3: :: core :: ffi :: c_float) { unsafe { melee_world::tu::ground::Ground_801C38D0(a0, a1, a2, a3) } }
pub fn Ground_801C38EC__15ec5f(a0: :: core :: ffi :: c_float, a1: :: core :: ffi :: c_float) { unsafe { melee_world::tu::ground::Ground_801C38EC(a0, a1) } }
pub fn Ground_801C3900__ff5c95(a0: :: core :: ffi :: c_float, a1: :: core :: ffi :: c_float, a2: :: core :: ffi :: c_float, a3: :: core :: ffi :: c_float, a4: :: core :: ffi :: c_float, a5: :: core :: ffi :: c_float, a6: :: core :: ffi :: c_float, a7: :: core :: ffi :: c_float) { unsafe { melee_world::tu::ground::Ground_801C3900(a0, a1, a2, a3, a4, a5, a6, a7) } }
pub fn Ground_801C392C__50666c(a0: :: core :: ffi :: c_float, a1: :: core :: ffi :: c_float, a2: :: core :: ffi :: c_float, a3: :: core :: ffi :: c_float, a4: :: core :: ffi :: c_float, a5: :: core :: ffi :: c_float) { unsafe { melee_world::tu::ground::Ground_801C392C(a0, a1, a2, a3, a4, a5) } }
pub fn Ground_801C3950__bf47a5(a0: :: core :: ffi :: c_float) { unsafe { melee_world::tu::ground::Ground_801C3950(a0) } }
pub fn Ground_801C3960__160db6(a0: :: core :: ffi :: c_float) { unsafe { melee_world::tu::ground::Ground_801C3960(a0) } }
pub fn Ground_801C3970__4bd2c4(a0: :: core :: ffi :: c_float) { unsafe { melee_world::tu::ground::Ground_801C3970(a0) } }
pub fn Ground_801C39C0__ef7ba7() { unsafe { melee_world::tu::ground::Ground_801C39C0() } }
pub fn Ground_801C3BB4__85d178() { unsafe { melee_world::tu::ground::Ground_801C3BB4() } }
pub fn Ground_801C4A08__d0ca52(a0: * mut HSD_GObj) { unsafe { melee_world::tu::ground::Ground_801C4A08(a0) } }
pub fn Ground_EnableMatchCamera__8fdbd1() { unsafe { melee_world::tu::ground::Ground_EnableMatchCamera() } }
pub fn Player_GetPlayerCharacter__ec3049(a0: :: core :: ffi :: c_int) -> CharacterKind { unsafe { melee_fighter::tu::player::Player_GetPlayerCharacter(a0) } }
pub fn Player_GetPlayerSlotType__b892ec(a0: s32) -> Gm_PKind { unsafe { melee_fighter::tu::player::Player_GetPlayerSlotType(a0) } }
pub fn Player_GetCostumeId__3389fa(a0: :: core :: ffi :: c_int) -> :: core :: ffi :: c_uint { unsafe { melee_fighter::tu::player::Player_GetCostumeId(a0) } }
pub fn Player_GetTeam__e2f2ad(a0: :: core :: ffi :: c_int) -> :: core :: ffi :: c_int { unsafe { melee_fighter::tu::player::Player_GetTeam(a0) } }
pub fn Player_GetCpuType__23acd7(a0: :: core :: ffi :: c_int) -> :: core :: ffi :: c_int { unsafe { melee_fighter::tu::player::Player_GetCpuType(a0) } }
pub fn Player_SetPlayerAndEntityCpuType__d3b2ae(a0: :: core :: ffi :: c_int, a1: :: core :: ffi :: c_int) { unsafe { melee_fighter::tu::player::Player_SetPlayerAndEntityCpuType(a0, a1) } }
pub fn Player_GetAttackRatio__c52a29(a0: :: core :: ffi :: c_int) -> :: core :: ffi :: c_float { unsafe { melee_fighter::tu::player::Player_GetAttackRatio(a0) } }
pub fn Player_GetDefenseRatio__53c460(a0: :: core :: ffi :: c_int) -> :: core :: ffi :: c_float { unsafe { melee_fighter::tu::player::Player_GetDefenseRatio(a0) } }
pub fn Player_GetStocks__0d01b6(a0: :: core :: ffi :: c_int) -> s32 { unsafe { melee_fighter::tu::player::Player_GetStocks(a0) } }
pub fn Player_SetStocks__f32692(a0: :: core :: ffi :: c_int, a1: :: core :: ffi :: c_int) { unsafe { melee_fighter::tu::player::Player_SetStocks(a0, a1) } }
pub fn Player_GetEntity__4ebf09(a0: s32) -> * mut HSD_GObj { unsafe { melee_fighter::tu::player::Player_GetEntity(a0) } }
pub fn Player_GetEntityAtIndex__54b10a(a0: :: core :: ffi :: c_int, a1: :: core :: ffi :: c_int) -> * mut HSD_GObj { unsafe { melee_fighter::tu::player::Player_GetEntityAtIndex(a0, a1) } }
pub fn Player_SetHUDDamage__5667ef(a0: s32, a1: s32) { unsafe { melee_fighter::tu::player::Player_SetHUDDamage(a0, a1) } }
pub fn Player_GetOtherStamina__98afbb(a0: s32) -> :: core :: ffi :: c_int { unsafe { melee_fighter::tu::player::Player_GetOtherStamina(a0) } }
pub fn Player_GetRemainingHP__3974b6(a0: s32) -> :: core :: ffi :: c_int { unsafe { melee_fighter::tu::player::Player_GetRemainingHP(a0) } }
pub fn Player_GetFalls__ac24ed(a0: s32) -> s32 { unsafe { melee_fighter::tu::player::Player_GetFalls(a0) } }
pub fn Player_GetFallsByIndex__1adaa0(a0: s32, a1: s32) -> s32 { unsafe { melee_fighter::tu::player::Player_GetFallsByIndex(a0, a1) } }
pub fn Item_80266F3C__ff0e50() -> :: core :: ffi :: c_int { unsafe { (melee_items::tu::item::Item_80266F3C()) as _ } }
pub fn Item_80268B18__3116a3(a0: * mut SpawnItem) -> * mut Item_GObj { unsafe { (melee_items::tu::item::Item_80268B18(a0)) as _ } }
pub fn Item_80269CA0__06ea06(a0: * mut Item, a1: s32) { unsafe { melee_items::item::item_80269ca0(&mut *a0, a1) } }
pub fn Item_8026A8EC__47012e(a0: * mut Item_GObj) { unsafe { melee_items::tu::item::Item_8026A8EC(a0 as _) } }
pub fn mpLib_8004D164__652d00() -> * mut MapCollData { unsafe { melee_world::tu::mplib::mpLib_8004D164() } }
pub fn mpScriptBatchBegin__6b27d1() { unsafe { melee_world::tu::mplib::mpScriptBatchBegin() } }
pub fn mpScriptBatchEnd__fff89e() { unsafe { melee_world::tu::mplib::mpScriptBatchEnd() } }
pub fn mpScriptInvalidateBounding__ddc52c() { unsafe { melee_world::tu::mplib::mpScriptInvalidateBounding() } }
pub fn mpGetGroundCollVtx__277435() -> * mut CollVtx { unsafe { melee_world::tu::mplib::mpGetGroundCollVtx() } }
pub fn mpGetGroundCollLine__1cd3a6() -> * mut CollLine { unsafe { melee_world::tu::mplib::mpGetGroundCollLine() } }
pub fn mpGetGroundCollJoint__54117b() -> * mut CollJoint { unsafe { melee_world::tu::mplib::mpGetGroundCollJoint() } }
pub fn mpLibLoad__0df8c9(a0: * mut MapCollData) { unsafe { melee_world::tu::mplib::mpLibLoad(a0) } }
pub fn mpLineGetV1Pos__3338ff(a0: :: core :: ffi :: c_int, a1: * mut Vec) { unsafe { melee_world::tu::mplib::mpLineGetV1Pos(a0, a1) } }
pub fn mpLineGetV0Pos__80a600(a0: :: core :: ffi :: c_int, a1: * mut Vec) { unsafe { melee_world::tu::mplib::mpLineGetV0Pos(a0, a1) } }
pub fn mpLib_80055E9C__84fe6b(a0: :: core :: ffi :: c_int) { unsafe { melee_world::tu::mplib::mpLib_80055E9C(a0) } }
pub fn mpLib_8005667C__76196e(a0: :: core :: ffi :: c_int) { unsafe { melee_world::tu::mplib::mpLib_8005667C(a0) } }
pub fn mpLib_80057424__edb375(a0: :: core :: ffi :: c_int) { unsafe { melee_world::tu::mplib::mpLib_80057424(a0) } }
pub fn mpLib_80057528__f6bdae(a0: :: core :: ffi :: c_int) { unsafe { melee_world::tu::mplib::mpLib_80057528(a0) } }
pub fn mpLib_800575B0__aa088d(a0: :: core :: ffi :: c_int) { unsafe { melee_world::tu::mplib::mpLib_800575B0(a0) } }
pub fn mpJointListAdd__ba67cb(a0: :: core :: ffi :: c_int) { unsafe { melee_world::tu::mplib::mpJointListAdd(a0) } }
pub fn mpLib_80057BC0__25e9db(a0: :: core :: ffi :: c_int) { unsafe { melee_world::tu::mplib::mpLib_80057BC0(a0) } }
pub fn mpUncheckBounding__cfacaf() { unsafe { melee_world::tu::mplib::mpUncheckBounding() } }
pub fn it_8026F9AC__945bc2(a0: s32, a1: * mut :: core :: ffi :: c_void, a2: * mut HitCapsule, a3: * mut Item, a4: * mut HurtCapsule) { unsafe { melee_items::tu::itcoll::it_8026F9AC(a0, a1, a2, a3, a4) } }
pub fn it_80270CD8__2e47eb(a0: * mut Item, a1: * mut HitCapsule) -> :: core :: ffi :: c_float { unsafe { melee_items::tu::itcoll::it_80270CD8(a0, a1) } }
pub fn it_80270E30__0506d2(a0: * mut Item_GObj) { unsafe { melee_items::tu::itcoll::it_80270E30(a0 as _) } }
pub fn it_8027B5B0__2814e8(a0: ItemKind, a1: * mut Vec, a2: * mut HSD_JObj, a3: * mut Vec, a4: :: core :: ffi :: c_int) -> * mut Item_GObj { unsafe { (melee_items::tu::itzako::it_8027B5B0(a0, a1 as _, a2, a3 as _, a4)) as _ } }
pub fn it_8027C56C__612d2a(a0: * mut Item_GObj, a1: :: core :: ffi :: c_float) { unsafe { melee_items::tu::itzako::it_8027C56C(a0 as _, a1) } }
pub fn it_802EA9FC__e2ed70(a0: * mut Vec, a1: s32) -> * mut Item_GObj { unsafe { melee_items::tu::itleadead::it_802EA9FC(a0 as _, a1) } }
pub fn it_802DC4BC__cbef47(a0: s32, a1: * mut Vec, a2: s32) -> * mut Item_GObj { unsafe { melee_items::tu::itlikelike::it_802DC4BC(a0, a1 as _, a2) } }
pub fn it_802DD7F0__ea09a6(a0: s32, a1: * mut Vec, a2: * mut Vec, a3: s32) -> * mut Item_GObj { unsafe { melee_items::tu::itnokonoko::it_802DD7F0(a0, a1 as _, a2 as _, a3) } }
pub fn lbAudioAx_800237A8__3c5520(a0: enum_t, a1: :: core :: ffi :: c_int, a2: :: core :: ffi :: c_int) -> :: core :: ffi :: c_int { unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_800237A8(a0 as _, a1, a2) } }
pub fn lbAudioAx_80023F28__469606(a0: :: core :: ffi :: c_int) -> :: core :: ffi :: c_int { unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80023F28(a0) } }
pub fn lbAudioAx_80024B94__675169(a0: :: core :: ffi :: c_int, a1: :: core :: ffi :: c_int) -> :: core :: ffi :: c_int { unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80024B94(a0, a1) } }
pub fn mpColl_80043680__d5fb54(a0: * mut CollData, a1: * mut Vec) { unsafe { melee_world::tu::mpcoll::mpColl_80043680(a0, a1) } }
pub fn mpCollSetFacingDir__1c7fdb(a0: * mut CollData, a1: :: core :: ffi :: c_int) { unsafe { melee_world::tu::mpcoll::mpCollSetFacingDir(a0, a1) } }
pub fn HSD_Free__8fde16(a0: * mut :: core :: ffi :: c_void) { unsafe { melee_runtime::tu::memory::HSD_Free(a0) } }
pub fn HSD_MemAlloc__8b8f38(a0: ssize_t) -> * mut :: core :: ffi :: c_void { unsafe { melee_runtime::tu::memory::HSD_MemAlloc(a0 as _) } }
pub fn lbFile_8001634C__c4f280(a0: :: core :: ffi :: c_int) -> usize { unsafe { melee_runtime::tu::lbfile::lbFile_8001634C(a0) } }
pub fn lbFile_8001668C__3aacaf(a0: * const :: core :: ffi :: c_char, a1: * mut :: core :: ffi :: c_void, a2: * mut usize) { unsafe { melee_runtime::tu::lbfile::lbFile_8001668C(a0, a1, a2) } }
pub fn lbHeap_80015BD0__b34fd8(a0: :: core :: ffi :: c_int, a1: usize) -> * mut :: core :: ffi :: c_void { unsafe { melee_runtime::tu::lbheap::lbHeap_80015BD0(a0, a1) } }
pub fn lbHeap_80015CA8__e646d5(a0: :: core :: ffi :: c_int, a1: * mut :: core :: ffi :: c_void) { unsafe { melee_runtime::tu::lbheap::lbHeap_80015CA8(a0, a1) } }
pub fn lbHeap_Capacity__dc42da(a0: :: core :: ffi :: c_int) -> :: core :: ffi :: c_uint { unsafe { melee_runtime::tu::lbheap::lbHeap_Capacity(a0) } }
pub fn lbHeap_Free__516b65(a0: :: core :: ffi :: c_int) -> :: core :: ffi :: c_uint { unsafe { melee_runtime::tu::lbheap::lbHeap_Free(a0) } }
pub fn lb_8000B1CC__6b12cc(a0: * mut HSD_JObj, a1: * mut Vec, a2: * mut Vec) { unsafe { melee_runtime::tu::lb_00B0::lb_8000B1CC(a0, a1, a2) } }
pub fn gm_8016B168__863c3b() -> :: core :: ffi :: c_int { unsafe { (crate::tu::gmvs::gm_8016B168()) as _ } }
pub fn gm_8016B0D4__af8007() -> :: core :: ffi :: c_int { unsafe { crate::tu::gmvs::gm_8016B0D4() } }
pub fn gm_8016B248__a7d73b() -> :: core :: ffi :: c_float { unsafe { crate::tu::gmvs::gm_8016B248() } }
pub fn ftCo_8008D8E8__f17b30(a0: :: core :: ffi :: c_float) -> :: core :: ffi :: c_int { unsafe { melee_fighter::tu::ftCo_Damage::ftCo_8008D8E8(a0) } }
pub fn ftCo_Damage_CalcKnockback__9478d1(a0: * mut Fighter) { unsafe { melee_fighter::tu::ftCo_Damage::ftCo_Damage_CalcKnockback(&mut *a0) } }
pub fn ftCo_800BFFD0__7622d7(a0: * mut Fighter, a1: enum_t, a2: :: core :: ffi :: c_int) -> :: core :: ffi :: c_int { unsafe { (melee_fighter::tu::ftcolanim::ftco_800bffd0(&mut *a0, a1 as _, a2)) as _ } }
pub fn ftCo_800C0074__1f5870(a0: * mut Fighter) { unsafe { melee_fighter::tu::ftcolanim::ftco_800c0074(&mut *a0) } }
pub fn ftCo_800C0200__1f3409(a0: * mut Fighter, a1: :: core :: ffi :: c_int) { unsafe { melee_fighter::tu::ftcolanim::ftco_800c0200(&mut *a0, a1) } }
pub fn ftCo_800C8348__936756(a0: * mut Fighter_GObj, a1: :: core :: ffi :: c_int, a2: :: core :: ffi :: c_int) { unsafe { melee_fighter::tu::ftmetal::ftCo_800C8348(a0 as _, a1, a2) } }
pub fn ftCo_800C8540__114dd4(a0: * mut Fighter_GObj) { unsafe { melee_fighter::tu::ftmetal::ftCo_800C8540(a0 as _) } }
pub fn it_8026D324__94e049(a0: ItemKind) -> :: core :: ffi :: c_int { unsafe { (melee_items::tu::itspawn::it_8026D324(a0)) as _ } }
pub fn grShrineRoute_ScriptQuery__a6b0e3(a0: :: core :: ffi :: c_int, a1: :: core :: ffi :: c_int) -> :: core :: ffi :: c_int { unsafe { melee_world::tu::grshrineroute::grShrineRoute_ScriptQuery(a0, a1) } }
pub fn fn_80218678__e0d59f() -> :: core :: ffi :: c_int { unsafe { melee_world::tu::grpushon::fn_80218678() } }
pub fn ftCo_800DF1C8__e872a9(a0: * mut Fighter) -> :: core :: ffi :: c_int { unsafe { (melee_fighter::ft0df::ftco_800df1c8(&*a0)) as _ } }
pub fn ftCo_800DF2D8__c6c36b(a0: * mut Fighter) -> :: core :: ffi :: c_int { unsafe { (melee_fighter::ft0df::ftco_800df2d8(&*a0)) as _ } }
pub fn ftCo_800DF3A8__8e22fa(a0: * mut Fighter) -> :: core :: ffi :: c_int { unsafe { (melee_fighter::ft0df::ftco_800df3a8(&*a0)) as _ } }
pub fn Stage_GetBlastZoneRightOffset__9f3e8b() -> :: core :: ffi :: c_float { unsafe { melee_world::tu::stage::Stage_GetBlastZoneRightOffset() } }
pub fn Stage_GetBlastZoneLeftOffset__7ceb78() -> :: core :: ffi :: c_float { unsafe { melee_world::tu::stage::Stage_GetBlastZoneLeftOffset() } }
pub fn Stage_GetBlastZoneTopOffset__04a3a9() -> :: core :: ffi :: c_float { unsafe { melee_world::tu::stage::Stage_GetBlastZoneTopOffset() } }
pub fn Stage_GetBlastZoneBottomOffset__752dc9() -> :: core :: ffi :: c_float { unsafe { melee_world::tu::stage::Stage_GetBlastZoneBottomOffset() } }
pub fn gmVs_SetEndHold__fb59b4(a0: :: core :: ffi :: c_int) { unsafe { crate::tu::gmvs::gmVs_SetEndHold(a0) } }
pub fn gmVs_GetHeldOutcome__44145d() -> :: core :: ffi :: c_int { unsafe { crate::tu::gmvs::gmVs_GetHeldOutcome() } }
pub fn lbHeap_StageSlotTryAlloc__9c8eee(a0: :: core :: ffi :: c_uint) -> * mut :: core :: ffi :: c_void { unsafe { melee_runtime::tu::lbheap::lbHeap_StageSlotTryAlloc(a0) } }
pub fn lbHeap_StageSlotDump__42ec51(a0: :: core :: ffi :: c_int) { unsafe { melee_runtime::tu::lbheap::lbHeap_StageSlotDump(a0) } }
pub fn lbAudioAx_StageSlotMusic__0aff91() -> :: core :: ffi :: c_int { unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_StageSlotMusic() } }
pub fn grAnime_801C7FF8__b10a63(a0: * mut HSD_GObj, a1: :: core :: ffi :: c_int, a2: :: core :: ffi :: c_int, a3: :: core :: ffi :: c_int, a4: :: core :: ffi :: c_float, a5: :: core :: ffi :: c_float) { unsafe { melee_world::tu::granime::grAnime_801C7FF8(a0 as _, a1, a2, a3, a4, a5) } }
pub fn grAnime_801C8138__a8596a(a0: * mut HSD_GObj, a1: enum_t, a2: :: core :: ffi :: c_int) { unsafe { melee_world::tu::granime::grAnime_801C8138(a0, a1, a2) } }
pub fn grStory_801E3030__27bc16() { unsafe { melee_world::tu::grstory::grStory_801E3030() } }
pub fn grStory_801E36D8__a338e5(a0: * mut Vec, a1: :: core :: ffi :: c_int, a2: * mut HSD_JObj) -> :: core :: ffi :: c_int { unsafe { melee_world::tu::grstory::grStory_801E36D8(a0 as _, a1, a2) } }
pub fn grIzumi_801CBB88__3ab22a() { unsafe { melee_world::tu::grizumi::grIzumi_801CBB88() } }
pub fn grIzumi_801CD280__3c103b(a0: * mut Vec, a1: :: core :: ffi :: c_int, a2: * mut HSD_JObj) -> :: core :: ffi :: c_int { unsafe { melee_world::tu::grizumi::grIzumi_801CD280(a0 as _, a1, a2) } }
pub fn Ground_StageSlotPrepare__d7a90e(a0: * mut HSD_GObj, a1: :: core :: ffi :: c_int) { unsafe { melee_world::tu::ground::Ground_StageSlotPrepare(a0, a1) } }
pub fn grStory_StageSlotParamSwap__30a7ba(a0: * mut :: core :: ffi :: c_void) -> * mut :: core :: ffi :: c_void { unsafe { melee_world::tu::grstory::grStory_StageSlotParamSwap(a0) } }
pub fn grIzumi_StageSlotParamSwap__e00bb2(a0: * mut :: core :: ffi :: c_void) -> * mut :: core :: ffi :: c_void { unsafe { melee_world::tu::grizumi::grIzumi_StageSlotParamSwap(a0) } }
pub fn hsd_8039D048__57a88e(a0: * mut :: core :: ffi :: c_void) { unsafe { melee_runtime::tu::particle::hsd_8039D048(a0) } }
pub fn Stage_StageSlotSelect__c64d66(a0: :: core :: ffi :: c_int) -> :: core :: ffi :: c_int { unsafe { melee_world::tu::stage::Stage_StageSlotSelect(a0) } }
pub fn Ground_StageSlotRetailInstall__95ae93(a0: * mut StageIdPair, a1: * mut HSD_Archive) { unsafe { melee_world::tu::ground::Ground_StageSlotRetailInstall(a0, a1) } }
pub fn Ground_StageSlotFighterLightingClear__7c49f4() { unsafe { melee_world::tu::ground::Ground_StageSlotFighterLightingClear() } }
pub fn mpIsland_StageSlotFree__a9f0f5() { unsafe { melee_world::tu::mpisland::mpIsland_StageSlotFree() } }
pub fn grZakoGenerator_StageSlotRelease__113f21() { unsafe { melee_world::tu::grzakogenerator::grZakoGenerator_StageSlotRelease() } }
pub fn Geno_GiveStunBonus__3e8019(a0: * mut Fighter, a1: :: core :: ffi :: c_int) { unsafe { crate::tu::geno_game::Geno_GiveStunBonus(a0, a1) } }
pub fn Geno_MotionRowIfAny__c473f8(a0: * mut Fighter, a1: :: core :: ffi :: c_int) -> * mut MotionState { unsafe { crate::tu::geno_game::Geno_MotionRowIfAny(a0, a1) } }
pub fn Geno_LabEnterState__96cf9a(a0: * mut Fighter, a1: :: core :: ffi :: c_int) -> :: core :: ffi :: c_int { unsafe { crate::tu::geno_game::Geno_LabEnterState(a0, a1) } }
pub fn Item_ScriptCommitDamage__cf50af(a0: * mut Item_GObj) -> :: core :: ffi :: c_int { unsafe { melee_items::tu::item::Item_ScriptCommitDamage(a0 as _) } }
pub fn Geno_ItemDesc__a0c825(a0: :: core :: ffi :: c_int) -> * mut :: core :: ffi :: c_void { unsafe { crate::tu::geno_game::Geno_ItemDesc(a0) } }
pub fn Ground_StageSlotFDBackdrop__c64489() { unsafe { melee_world::tu::grlast::Ground_StageSlotFDBackdrop() } }
pub fn mn_80229624__84b7ce(a0: u32) -> u32 { unsafe { crate::tu::mnmain::mn_80229624(a0) } }
pub fn ftAnim_8006F0FC__39c58d(a0: *mut Fighter_GObj, a1: f32) { unsafe { melee_fighter::tu::ftanim::ftAnim_8006F0FC(a0 as _, a1) } }
pub fn ftAnim_SetAnimRate__352e8e(a0: *mut Fighter_GObj, a1: f32) { unsafe { melee_fighter::tu::ftanim::ftAnim_SetAnimRate(a0 as _, a1) } }
pub fn ftAnim_IsFramesRemaining__101630(a0: *mut Fighter_GObj) -> i32 { unsafe { (melee_fighter::tu::ftanim::ftAnim_IsFramesRemaining(a0 as _)) as _ } }
pub fn Fighter_ChangeMotionState__c62dac(a0: *mut Fighter_GObj, a1: FtMotionId, a2: MotionFlags, a3: f32, a4: f32, a5: f32, a6: *mut Fighter_GObj) { unsafe { melee_fighter::tu::fighter::Fighter_ChangeMotionState(a0 as _, a1 as _, a2 as _, a3, a4, a5, a6 as _) } }
pub fn ft_8008A2BC__51106d(a0: *mut Fighter_GObj) { unsafe { melee_fighter::tu::ft_08a1::ft_8008A2BC(a0 as _) } }
pub fn ftCo_8009917C__02d948(a0: *mut Fighter_GObj) -> i32 { unsafe { (melee_fighter::tu::ftCo_Escape::ftCo_8009917C(a0 as _)) as _ } }
pub fn ftCo_Fall_Enter__5f227e(a0: *mut Fighter_GObj) { unsafe { melee_fighter::tu::ftCo_Fall::ftCo_Fall_Enter(a0 as _) } }
pub fn ftCo_Fall_IASA__163f16(a0: *mut Fighter_GObj) { unsafe { melee_fighter::tu::ftCo_Fall::ftCo_Fall_IASA(a0 as _) } }
pub fn ftCo_800968C8__1d49d6(a0: *mut Fighter_GObj) { unsafe { melee_fighter::tu::ftCo_FallSpecial::ftCo_800968C8(a0 as _) } }
pub fn ftCo_80096CC8__f60bbf(a0: *mut Fighter_GObj, a1: i32) -> i32 { unsafe { (melee_fighter::tu::ftCo_FallSpecial::ftCo_80096CC8(a0 as _, a1)) as _ } }
pub fn ftCo_80096D28__30b1a1(a0: *mut Fighter_GObj) { unsafe { melee_fighter::tu::ftCo_FallSpecial::ftCo_80096D28(a0 as _) } }
pub fn ftCo_Landing_Enter_Basic__b5080f(a0: *mut Fighter_GObj) { unsafe { melee_fighter::tu::ftCo_Landing::ftCo_Landing_Enter_Basic(a0 as _) } }
pub fn ftCo_LandingFallSpecial_Enter_Basic__94c260(a0: *mut Fighter_GObj) { unsafe { melee_fighter::tu::ftCo_Landing::ftCo_LandingFallSpecial_Enter_Basic(a0 as _) } }
pub fn ftCo_LandingFallSpecial_Enter__90d150(a0: *mut Fighter_GObj, a1: i32, a2: f32) { unsafe { melee_fighter::tu::ftCo_Landing::ftCo_LandingFallSpecial_Enter(a0 as _, a1 != 0, a2) } }
pub fn ftCo_Wait_IASA__a48c3f(a0: *mut Fighter_GObj) { unsafe { melee_fighter::tu::ftCo_Wait::ftCo_Wait_IASA(a0 as _) } }
pub fn ft_80082C74__c5d8fd(a0: *mut Fighter_GObj, a1: HSD_GObjEvent) { unsafe { melee_fighter::tu::ft_081b::ft_80082c74(a0 as _, core::mem::transmute(a1)) } }
pub fn ft_8008403C__23dfed(a0: *mut Fighter_GObj, a1: HSD_GObjEvent) { unsafe { melee_fighter::tu::ft_081b::ft_8008403c(a0 as _, core::mem::transmute(a1)) } }
pub fn ft_80084104__46f90b(a0: *mut Fighter_GObj) { unsafe { melee_fighter::tu::ft_081b::ft_80084104(a0 as _) } }
pub fn ft_80084280__930c83(a0: *mut Fighter_GObj) { unsafe { melee_fighter::tu::ft_081b::ft_80084280(a0 as _) } }
pub fn ft_80084DB0__838ea6(a0: *mut Fighter_GObj) { unsafe { melee_fighter::ft084e::ft_80084db0(a0 as _) } }
pub fn ft_80084F3C__408261(a0: *mut Fighter_GObj) { unsafe { melee_fighter::ft084e::ft_80084f3c(a0 as _) } }
pub fn ft_80081A00__8629a6(a0: *mut Fighter_GObj) -> i32 { unsafe { (melee_fighter::tu::ft_0819::ft_80081a00(a0 as _)) as _ } }
pub fn ftCliffCommon_80081298__08ff6d(a0: *mut Fighter_GObj) -> i32 { unsafe { (melee_fighter::tu::ftcliffcommon::ftcliffcommon_80081298(a0 as _)) as _ } }
pub fn ftCo_AttackAir_EnterFromMsid__52a3a7(a0: *mut Fighter_GObj, a1: FtMotionId) { unsafe { melee_fighter::tu::ftCo_AttackAir::ftCo_AttackAir_EnterFromMsid(a0 as _, a1 as _) } }
pub fn ftCo_JumpAerial_Enter_Basic__4afd40(a0: *mut Fighter_GObj) { unsafe { melee_fighter::tu::ftCo_JumpAerial::ftCo_JumpAerial_Enter_Basic(a0 as _) } }
pub fn ftCo_Jump_Enter__230c83(a0: *mut Fighter_GObj) { unsafe { melee_fighter::tu::ftCo_Jump::ftCo_Jump_Enter(a0 as _) } }
pub fn ftAction_80073240__64da7b(a0: *mut Fighter_GObj) { unsafe { melee_fighter::tu::ftaction::ftAction_80073240(a0 as _) } }
pub fn ftAction_80073354__0de40c(a0: *mut Fighter_GObj) { unsafe { melee_fighter::tu::ftaction::ftAction_80073354(a0 as _) } }
pub fn ftAction_8007349C__18c7dc(a0: *mut Fighter_GObj) { unsafe { melee_fighter::tu::ftaction::ftAction_8007349C(a0 as _) } }
pub fn ftCo_800D3158__f65bbf(a0: *mut Fighter_GObj) -> i32 { unsafe { (melee_fighter::tu::ft_0d31::ftCo_800D3158(a0 as _)) as _ } }
pub fn ftCo_80091E78__aefdda(a0: *mut Fighter_GObj, a1: f32) { unsafe { melee_fighter::tu::ftCo_Guard::ftCo_80091E78(a0 as _, a1) } }
pub fn ftLib_8008731C__becdfc(a0: *mut HSD_GObj) -> i32 { unsafe { melee_fighter::ftlib::ftlib_8008731c(a0) } }
pub fn gmVsMelee_EnterSuddenDeath__883e46(a0: *mut GameModeState, a1: *mut VsModeData, a2: Option<unsafe extern "C" fn(*mut StartMeleeData, *mut StartMeleeData)>, a3: Option<unsafe extern "C" fn(*mut PlayerInitData, *mut PlayerInitData)>) { unsafe { crate::tu::gmvsmelee::gmVsMelee_EnterSuddenDeath(a0, a1, a2, a3) } }
pub fn gmVsMelee_EnterVs__41232b(a0: *mut GameModeState, a1: *mut VsModeData, a2: Option<unsafe extern "C" fn(*mut StartMeleeData, *mut StartMeleeData)>, a3: Option<unsafe extern "C" fn(*mut PlayerInitData, *mut PlayerInitData)>) { unsafe { crate::tu::gmvsmelee::gmVsMelee_EnterVs(a0, a1, a2, a3) } }
pub fn gm_80160968__e03b47(a0: u32) -> _GXColor { unsafe { core::mem::transmute(crate::gm::gm_80160968(a0)) } }
pub fn ftCo_800C8C84__d9cbd1(a0: *mut HSD_GObj) -> i32 { unsafe { (melee_fighter::tu::ft_0c8c::ftco_800c8c84(a0)) as _ } }
pub fn ft_80087C58__344dc5(a0: *mut Fighter_GObj) -> i32 { unsafe { melee_fighter::ft0877::ft_80087c58(a0 as _) } }
pub fn ft_80087C64__aaf698(a0: *mut Fighter_GObj, a1: i32) { unsafe { melee_fighter::ft0877::ft_80087c64(a0 as _, a1) } }
pub fn mn_80229894__f7df64(a0: i32, a1: u16, a2: i32) { unsafe { crate::tu::mnmain::mn_80229894(a0, a1, a2) } }
pub fn mn_8022EA78__f27e4b(a0: *mut i8, a1: i32, a2: u32) { unsafe { crate::tu::mnmain::mn_8022EA78(a0 as _, a1, a2) } }
pub fn mn_8022EA78__a1df42(a0: *mut ::core::ffi::c_char, a1: ::core::ffi::c_int, a2: u32) { unsafe { crate::tu::mnmain::mn_8022EA78(a0, a1, a2) } }
pub fn mnName_80239EBC__737a0b(a0: *mut HSD_JObj, a1: f32) { unsafe { crate::tu::mnname::mnName_80239EBC(a0, a1) } }
pub fn mnName_80239F5C__711f9f(a0: *mut HSD_JObj, a1: f32) { unsafe { crate::tu::mnname::mnName_80239F5C(a0, a1) } }
pub fn mnName_8023A9B4__53fde4(a0: u8) { unsafe { crate::tu::mnname::mnName_8023A9B4(a0) } }
pub fn mn_80231F80__cd7b6f(a0: u8) -> i32 { unsafe { crate::tu::mnmainrule::mn_80231F80(a0) } }
pub fn efLib_DestroyAll__17a98a(a0: * mut HSD_GObj) { unsafe { melee_world::tu::eflib::efLib_DestroyAll(a0) } }
pub fn ftAnim_8006F484__1b854b(a0: * mut Fighter_GObj) -> :: core :: ffi :: c_float { unsafe { melee_fighter::tu::ftanim::ftAnim_8006F484(a0 as _) } }
pub fn Fighter_UnkProcessDeath_80068354__0d8c62(a0: * mut Fighter_GObj) { unsafe { melee_fighter::tu::fighter::Fighter_UnkProcessDeath_80068354(a0 as _) } }
pub fn Fighter_ResetInputData_80068854__e11eca(a0: * mut Fighter_GObj) { unsafe { melee_fighter::tu::fighter::Fighter_ResetInputData_80068854(a0 as _) } }
pub fn Fighter_ChangeMotionState__8da657(a0: * mut Fighter_GObj, a1: FtMotionId, a2: MotionFlags, a3: :: core :: ffi :: c_float, a4: :: core :: ffi :: c_float, a5: :: core :: ffi :: c_float, a6: * mut Fighter_GObj) { unsafe { melee_fighter::tu::fighter::Fighter_ChangeMotionState(a0 as _, a1 as _, a2 as _, a3, a4, a5, a6 as _) } }
pub fn Fighter_ProcessHit_8006D1EC__4dab97(a0: * mut Fighter_GObj) { unsafe { melee_fighter::tu::fighter::Fighter_ProcessHit_8006D1EC(a0 as _) } }
pub fn ftCommon_8007D92C__e78cfc(a0: * mut Fighter_GObj) { unsafe { melee_fighter::ftcommon3::ftcommon_8007d92c(a0 as _) } }
pub fn ftLib_80086960__b56944(a0: * mut HSD_GObj) -> :: core :: ffi :: c_int { unsafe { (melee_fighter::ftlib::ftlib_80086960(a0)) as _ } }
pub fn ft_80081C88__3d5b9d(a0: * mut Fighter_GObj, a1: :: core :: ffi :: c_float) { unsafe { melee_fighter::tu::ft_081b::ft_80081c88(a0 as _, a1) } }
pub fn lbColl_8000A584__73c219(a0: * mut HurtCapsule, a1: :: core :: ffi :: c_uint, a2: :: core :: ffi :: c_uint, a3: MtxPtr, a4: :: core :: ffi :: c_float) -> :: core :: ffi :: c_int { unsafe { (melee_runtime::tu::lbcollision::lbColl_8000A584(a0, a1, a2, core::mem::transmute(a3), a4)) as _ } }
pub fn ft_8008A2BC__b456f9(a0: * mut Fighter_GObj) { unsafe { melee_fighter::tu::ft_08a1::ft_8008A2BC(a0 as _) } }
pub fn ftCo_Fall_Enter__2d1b10(a0: * mut Fighter_GObj) { unsafe { melee_fighter::tu::ftCo_Fall::ftCo_Fall_Enter(a0 as _) } }
pub fn ftpickupitem_80094150__ba0ae9(a0: * mut Fighter_GObj, a1: * mut Item_GObj) -> :: core :: ffi :: c_int { unsafe { (melee_fighter::tu::ftpickupitem::ftpickupitem_80094150(a0 as _, a1)) as _ } }
pub fn ftpickupitem_800948A8__668622(a0: * mut Fighter_GObj, a1: * mut Item_GObj) { unsafe { melee_fighter::tu::ftpickupitem::ftpickupitem_800948A8(a0 as _, a1) } }
pub fn ftCo_800C3B10__3d9d5c(a0: * mut Fighter_GObj) -> :: core :: ffi :: c_int { unsafe { (melee_fighter::tu::ftco_aircatch::ftco_800c3b10(a0)) as _ } }
pub fn ftCo_Attack1_CheckInput__d92fe1(a0: * mut Fighter_GObj) -> :: core :: ffi :: c_int { unsafe { (melee_fighter::tu::ftCo_Attack1::ftCo_Attack1_CheckInput(a0 as _)) as _ } }
pub fn ftCo_800D6824__b21805(a0: * mut Fighter_GObj) -> :: core :: ffi :: c_int { unsafe { (melee_fighter::tu::ftCo_Attack100::ftCo_800D6824(a0 as _)) as _ } }
pub fn ftCo_800D68C0__ec1729(a0: * mut Fighter_GObj) -> :: core :: ffi :: c_int { unsafe { (melee_fighter::tu::ftCo_Attack100::ftCo_800D68C0(a0 as _)) as _ } }
pub fn ftCo_Attack100_CheckInput__61aa80(a0: * mut Fighter_GObj) -> :: core :: ffi :: c_int { unsafe { (melee_fighter::tu::ftCo_Attack100::ftCo_Attack100_CheckInput(a0 as _)) as _ } }
pub fn ftCo_Catch_CheckInput__e60751(a0: * mut Fighter_GObj) -> :: core :: ffi :: c_int { unsafe { (melee_fighter::tu::ftCo_Catch::ftCo_Catch_CheckInput(a0 as _)) as _ } }
pub fn ftCo_AttackAir_CheckItemThrowInput__1e285e(a0: * mut Fighter_GObj) -> :: core :: ffi :: c_int { unsafe { (melee_fighter::tu::ftCo_AttackAir::ftCo_AttackAir_CheckItemThrowInput(a0 as _)) as _ } }
pub fn ftCo_AttackHi3_CheckInput__5d742f(a0: * mut Fighter_GObj) -> :: core :: ffi :: c_int { unsafe { (melee_fighter::tu::ftCo_AttackHi3::ftCo_AttackHi3_CheckInput(a0 as _)) as _ } }
pub fn ftCo_AttackHi4_CheckInput__ebfa65(a0: * mut Fighter_GObj) -> :: core :: ffi :: c_int { unsafe { (melee_fighter::tu::ftCo_AttackHi4::ftCo_AttackHi4_CheckInput(a0 as _)) as _ } }
pub fn ftCo_AttackLw3_CheckInput__132bbd(a0: * mut Fighter_GObj) -> :: core :: ffi :: c_int { unsafe { (melee_fighter::tu::ftCo_AttackLw3::ftCo_AttackLw3_CheckInput(a0 as _)) as _ } }
pub fn ftCo_AttackLw4_CheckInput__e93fbf(a0: * mut Fighter_GObj) -> :: core :: ffi :: c_int { unsafe { (melee_fighter::tu::ftCo_AttackLw4::ftCo_AttackLw4_CheckInput(a0 as _)) as _ } }
pub fn ftCo_AttackS3_CheckInput__4f4014(a0: * mut Fighter_GObj) -> :: core :: ffi :: c_int { unsafe { (melee_fighter::tu::ftCo_AttackS3::ftCo_AttackS3_CheckInput(a0 as _)) as _ } }
pub fn ftCo_AttackS4_CheckInput__48fe9d(a0: * mut Fighter_GObj) -> :: core :: ffi :: c_int { unsafe { (melee_fighter::tu::ftCo_AttackS4::ftCo_AttackS4_CheckInput(a0 as _)) as _ } }
pub fn ftCo_Dash_CheckInput__1e8d28(a0: * mut Fighter_GObj) -> :: core :: ffi :: c_int { unsafe { (melee_fighter::tu::ftCo_Dash::ftCo_Dash_CheckInput(a0 as _)) as _ } }
pub fn ftCo_80099794__7a44df(a0: * mut Fighter_GObj) -> :: core :: ffi :: c_int { unsafe { (melee_fighter::tu::ftCo_Escape::ftCo_80099794(a0 as _)) as _ } }
pub fn ftCo_80099A58__225834(a0: * mut Fighter_GObj) -> :: core :: ffi :: c_int { unsafe { (melee_fighter::tu::ftCo_EscapeAir::ftCo_80099A58(a0 as _)) as _ } }
pub fn ftCo_80091A4C__5bb07f(a0: * mut Fighter_GObj) -> :: core :: ffi :: c_int { unsafe { (melee_fighter::tu::ftCo_Guard::ftCo_80091A4C(a0 as _)) as _ } }
pub fn ftCo_Jump_CheckInput__ff2f50(a0: * mut Fighter_GObj) -> :: core :: ffi :: c_int { unsafe { (melee_fighter::tu::ftCo_Jump::ftCo_Jump_CheckInput(a0 as _)) as _ } }
pub fn ftCo_800CB870__e3acbf(a0: * mut Fighter_GObj) -> :: core :: ffi :: c_int { unsafe { (melee_fighter::tu::ftCo_JumpAerial::ftCo_800CB870(a0 as _)) as _ } }
pub fn ftCo_SpecialAir_CheckInput__a14aa2(a0: * mut Fighter_GObj) -> :: core :: ffi :: c_int { unsafe { (melee_fighter::tu::ftCo_SpecialAir::ftCo_SpecialAir_CheckInput(a0 as _)) as _ } }
pub fn ftCo_SpecialS_CheckInput__9abbee(a0: * mut Fighter_GObj) -> :: core :: ffi :: c_int { unsafe { (melee_fighter::tu::ftCo_SpecialS::ftCo_SpecialS_CheckInput(a0 as _)) as _ } }
pub fn ftCo_800D5FB0__315eef(a0: * mut Fighter_GObj) -> :: core :: ffi :: c_int { unsafe { (melee_fighter::tu::ftCo_Squat::ftCo_800D5FB0(a0 as _)) as _ } }
pub fn ftCo_Turn_CheckInput__96ef34(a0: * mut Fighter_GObj) -> :: core :: ffi :: c_int { unsafe { (melee_fighter::tu::ftCo_Turn::ftCo_Turn_CheckInput(a0 as _)) as _ } }
pub fn ftCo_Walk_CheckInput__5c6c59(a0: * mut Fighter_GObj) -> :: core :: ffi :: c_int { unsafe { (melee_fighter::tu::ftCo_Walk::ftCo_Walk_CheckInput(a0 as _)) as _ } }
pub fn ftCo_800DCE34__658ea4(a0: * mut Fighter_GObj, a1: * mut Fighter_GObj) { unsafe { melee_fighter::tu::ftCo_CaptureCut::ftCo_800DCE34(a0 as _, a1 as _) } }
pub fn IsNameUnique__f9fddb(a0: *mut c_char) -> i32 { unsafe { crate::tu::mnname::IsNameUnique(a0) } }
pub fn IsNameNotAllowed__c0ffe3(a0: *mut c_char) -> i32 { unsafe { crate::tu::mnname::IsNameNotAllowed(a0) } }
pub fn mn_8022EA08__998c76(a0: *mut c_char, a1: u32) { unsafe { crate::tu::mnmain::mn_8022EA08(a0, a1) } }
pub fn HSD_ArchiveGetPublicAddress__6d6399(a0: *mut HSD_Archive, a1: *const c_char) -> *mut c_void { unsafe { melee_runtime::tu::archive::HSD_ArchiveGetPublicAddress(a0, a1 as _) } }
pub fn lbArchive_LoadArchive__992527(a0: *const c_char) -> *mut HSD_Archive { unsafe { melee_runtime::tu::lbarchive::lbArchive_LoadArchive(a0) } }
pub fn GetNameText__e732d9(a0: u8) -> *mut c_char { unsafe { crate::tu::mnname::GetNameText(a0 as _) } }
pub fn HSD_ForeachAnim__175c67(a0: *mut c_void, a1: HSD_Type, a2: HSD_TypeMask, a3: *mut c_void, a4: AObj_Arg_Type, a5: u64) { unsafe { melee_runtime::tu::aobj::HSD_ForeachAnim(a0, a1, a2, a3, a4, core::mem::transmute(a5)) } }
pub fn Stage_8022519C__a93aca(a0: u32) -> u32 { unsafe { core::mem::transmute(melee_world::tu::stage::Stage_8022519C(core::mem::transmute(a0))) } }
pub fn gm_8017CE34__75f422(a0: *mut StartMeleeData, a1: *mut Unk1PData, a2: *mut i8, a3: u8, a4: u8, a5: u8, a6: i32, a7: u32, a8: i32, a9: i32) { unsafe { crate::tu::gm_17c0::gm_8017CE34(a0, a1, a2, a3, a4, a5, a6, core::mem::transmute(a7), a8, a9) } }
pub fn Player_800325C8__4ace7f(a0: u32, a1: i32) -> i8 { unsafe { melee_fighter::player::player_800325c8(core::mem::transmute(a0), a1) } }
pub fn gmVsMelee_EnterCss__44afc7(a0: *mut GameModeState, a1: *mut VsModeData, a2: u32) { unsafe { crate::tu::gmvsmelee::gmVsMelee_EnterCss(a0, a1, core::mem::transmute(a2)) } }
pub fn HSD_SObjLib_803A477C__a0c4ae(a0: *mut HSD_GObj, a1: *mut HSD_SObjDesc, a2: u32, a3: u32, a4: u8, a5: u8) -> *mut HSD_SObj { unsafe { melee_runtime::tu::sobjlib::HSD_SObjLib_803A477C(a0, a1, core::mem::transmute(a2), core::mem::transmute(a3), a4, a5) } }
pub fn Player_SetSlottype__6536f1(a0: i32, a1: u32) { unsafe { melee_fighter::tu::player::Player_SetSlottype(a0, core::mem::transmute(a1)) } }
pub fn mn_8022F3D8__5703af(a0: *mut HSD_JObj, a1: u8, a2: u32) { unsafe { crate::tu::mn_22EC::mn_8022F3D8(a0, a1, core::mem::transmute(a2)) } }
pub fn Stage_802251E8__211944(a0: u32, a1: *mut i32) { unsafe { melee_world::tu::stage::Stage_802251E8(core::mem::transmute(a0), a1) } }
pub fn HSD_StartRender__c91577(a0: u32) { unsafe { melee_runtime::tu::initialize::HSD_StartRender(core::mem::transmute(a0)) } }
pub fn HSD_VICopyXFBAsync__35ed8e(a0: u32) { unsafe { melee_runtime::tu::video::HSD_VICopyXFBAsync(core::mem::transmute(a0)) } }
pub fn lbCardNew_CompleteNextTask__ebb0f7() -> enum_t { unsafe { core::mem::transmute(melee_runtime::tu::lbcardnew::lbCardNew_CompleteNextTask()) } }
pub fn ftCo_AttackAir_GetMsidFromCStick__a36195(a0: * mut Fighter) -> FtMotionId { unsafe { core::mem::transmute(melee_fighter::tu::ftCo_AttackAir::ftCo_AttackAir_GetMsidFromCStick(&*a0)) } }
pub fn MTXRotRad__bc3d39(a0: *mut c_void, a1: i8, a2: f32) { unsafe { melee_runtime::mtx::rot_rad(&mut *(a0 as *mut melee_runtime::mtx::Mtx), a1 as u8, a2) } }
pub fn PSMTXConcat__e4df5b(a0: *mut c_void, a1: *mut c_void, a2: *mut c_void) { unsafe { *(a2 as *mut melee_runtime::mtx::Mtx) = melee_runtime::mtx::concat(&*(a0 as *const melee_runtime::mtx::Mtx), &*(a1 as *const melee_runtime::mtx::Mtx)) } }
pub fn PSMTXMultVec__aeae10(a0: *mut c_void, a1: *mut Vec, a2: *mut Vec) { unsafe { *(a2 as *mut melee_runtime::mtx::Vec3) = melee_runtime::mtx::mult_vec(core::slice::from_raw_parts(a0 as *const [f32; 4], 3), &*(a1 as *const melee_runtime::mtx::Vec3)) } }
pub fn PSMTXQuat__fb47a1(a0: *mut c_void, a1: *mut Quaternion) { unsafe { *(a0 as *mut melee_runtime::mtx::Mtx) = melee_runtime::mtx::quat(&*(a1 as *const melee_runtime::mtx::Quaternion)) } }
pub fn PSMTXRotAxisRad__437995(a0: *mut c_void, a1: *mut Vec, a2: f32) { unsafe { *(a0 as *mut melee_runtime::mtx::Mtx) = melee_runtime::mtx::rot_axis_rad(&*(a1 as *const melee_runtime::mtx::Vec3), a2) } }
pub fn PSMTXTranspose__cc6675(a0: *mut c_void, a1: *mut c_void) { unsafe { *(a1 as *mut melee_runtime::mtx::Mtx) = melee_runtime::mtx::transpose(&*(a0 as *const melee_runtime::mtx::Mtx)) } }
pub fn PSVECMag__307672(a0: *mut Vec) -> f32 { unsafe { melee_runtime::mtx::vec_mag(&*(a0 as *const melee_runtime::mtx::Vec3)) } }
pub fn MTXOrtho__214174(a0: *mut c_void, a1: f32, a2: f32, a3: f32, a4: f32, a5: f32, a6: f32) { unsafe { *(a0 as *mut melee_runtime::mtx::Mtx44) = melee_runtime::mtx::ortho(a1, a2, a3, a4, a5, a6) } }
pub fn PSVECSubtract__594d9f(a0: *mut Vec3, a1: *mut Vec3, a2: *mut Vec3) { unsafe { *(a2 as *mut melee_runtime::mtx::Vec3) = melee_runtime::mtx::vec_sub(&*(a0 as *const melee_runtime::mtx::Vec3), &*(a1 as *const melee_runtime::mtx::Vec3)) } }
pub fn PSVECNormalize__01cbbc(a0: *mut Vec3, a1: *mut Vec3) { unsafe { *(a1 as *mut melee_runtime::mtx::Vec3) = melee_runtime::mtx::vec_normalize(&*(a0 as *const melee_runtime::mtx::Vec3)) } }
pub fn PSVECCrossProduct__0fba16(a0: *mut Vec3, a1: *mut Vec3, a2: *mut Vec3) { unsafe { *(a2 as *mut melee_runtime::mtx::Vec3) = melee_runtime::mtx::vec_cross(&*(a0 as *const melee_runtime::mtx::Vec3), &*(a1 as *const melee_runtime::mtx::Vec3)) } }
pub fn PSMTXConcat__6707f0(a0: * mut [:: core :: ffi :: c_float ; 4], a1: * mut [:: core :: ffi :: c_float ; 4], a2: * mut [:: core :: ffi :: c_float ; 4]) { unsafe { *(a2 as *mut melee_runtime::mtx::Mtx) = melee_runtime::mtx::concat(&*(a0 as *const melee_runtime::mtx::Mtx), &*(a1 as *const melee_runtime::mtx::Mtx)) } }
pub fn PSMTXInverse__c61344(a0: * mut [:: core :: ffi :: c_float ; 4], a1: * mut [:: core :: ffi :: c_float ; 4]) -> :: core :: ffi :: c_uint { unsafe { match melee_runtime::mtx::inverse(&*(a0 as *const melee_runtime::mtx::Mtx)) { Some(m) => { *(a1 as *mut melee_runtime::mtx::Mtx) = m; 1 } None => 0 } } }
pub fn PSMTXMultVec__416c01(a0: * mut [:: core :: ffi :: c_float ; 4], a1: * mut Vec, a2: * mut Vec) { unsafe { *(a2 as *mut melee_runtime::mtx::Vec3) = melee_runtime::mtx::mult_vec(core::slice::from_raw_parts(a0 as *const [f32; 4], 3), &*(a1 as *const melee_runtime::mtx::Vec3)) } }
pub fn Camera_Init__cmc(a0: i32) { unsafe { melee_world::tu::cmcamera::Camera_Init(a0) } }
pub fn Camera_Create__cmc() { unsafe { melee_world::tu::cmcamera::Camera_Create() } }
pub fn Camera_ForgetGameCamera__cmc() { unsafe { melee_world::tu::cmcamera::Camera_ForgetGameCamera() } }
pub fn Camera_RefreshViewingMtx__cmc() { unsafe { melee_world::tu::cmcamera::Camera_RefreshViewingMtx() } }
pub fn Camera_8002F3AC__cmc() { unsafe { melee_world::tu::cmcamera::Camera_8002F3AC() } }
pub fn Camera_SetQuakeScale__cmc(a0: f32) { unsafe { melee_world::tu::cmcamera::Camera_SetQuakeScale(a0) } }
pub fn Camera_PCMainCObj__cmc() -> *mut HSD_CObj { unsafe { melee_world::tu::cmcamera::Camera_PCMainCObj() as _ } }
