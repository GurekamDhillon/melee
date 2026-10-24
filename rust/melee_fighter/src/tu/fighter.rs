//! Native port of src/melee/ft/fighter.c (+ its TARGET_PC includes technical_ai.inc and
//! cpu_assist.inc): fighter creation, per-frame procs, motion-state change, damage, hit
//! processing, and the TARGET_PC rollback / neutral hashes.
//!
//! Globals defined by fighter.c are module statics here; the PlCo tables other ported modules
//! read through `crate::hooks` are installed there by [`Fighter_LoadCommonData`].
//! Already ported and re-exported: `Fighter_NewSpawn_80068E40`, `Fighter_UnkSetFlag_8006CFBC`,
//! `setBit`, `Fighter_GetPosX/Y`, the UCF 0.84 pad ring (`crate::fighter`), the NHV1 neutral
//! hash (`crate::neutral_hash`).
//! Fused sites (`a*b+c` in one C expression) use `melee_core::fp::fmadds` (shipping
//! contraction); each is marked `// fused`. Callees in TUs not ported yet go through `ext`.
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, clippy::missing_safety_doc, unused_unsafe)]

use crate::hooks as h;
use crate::types::*;
use melee_core::fp::fmadds;
use melee_types::inline::*;

pub use crate::fighter::{
    fighter_get_pos_x as Fighter_GetPosX, fighter_get_pos_y as Fighter_GetPosY,
    fighter_new_spawn_80068e40 as Fighter_NewSpawn_80068E40, fighter_unk_set_flag_8006cfbc as Fighter_UnkSetFlag_8006CFBC,
    ftucf_flick_count as ftUcf_FlickCount, ftucf_flick_counter as ftUcf_FlickCounter, ftucf_pad_buffer_push as ftUcf_PadBufferPush,
    ftucf_pad_raw as ftUcf_PadRaw, ftucf_rim_units as ftUcf_RimUnits, set_bit as setBit,
};

type G = *mut HSD_GObj;
type J = *mut HSD_JObj;
type F = *mut Fighter;

const M_PI_2: f64 = core::f64::consts::FRAC_PI_2;

// ---- fighter.c variables ------------------------------------------------------------------
pub static Fighter_803B7488: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
pub static vec3_803B7494: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
pub static mut fighter_alloc_data: HSD_ObjAllocData = unsafe { core::mem::zeroed() };
pub static mut fighter_dat_attrs_alloc_data: HSD_ObjAllocData = unsafe { core::mem::zeroed() };
pub static mut fighter_parts_alloc_data: HSD_ObjAllocData = unsafe { core::mem::zeroed() };
pub static mut fighter_dobj_list_alloc_data: HSD_ObjAllocData = unsafe { core::mem::zeroed() };
pub static mut fighter_x2040_alloc_data: HSD_ObjAllocData = unsafe { core::mem::zeroed() };
pub static mut fighter_x59C_alloc_data: HSD_ObjAllocData = unsafe { core::mem::zeroed() };
pub static mut Fighter_804D64FC: *mut Fighter_804D64FC_t = core::ptr::null_mut();
pub static mut gCrowdConfig: *mut core::ffi::c_void = core::ptr::null_mut();
pub static mut Fighter_804D6504: *mut HSD_Joint = core::ptr::null_mut();
pub static mut Fighter_804D6508: *mut u8 = core::ptr::null_mut();
pub static mut Fighter_804D650C: *mut u8 = core::ptr::null_mut();
pub static mut Fighter_804D6510: *mut core::ffi::c_void = core::ptr::null_mut();
pub static mut Fighter_804D6514: *mut HSD_Joint = core::ptr::null_mut();
pub static mut Fighter_804D6518: *mut Fighter_804D6518_t = core::ptr::null_mut();
pub static mut Fighter_804D651C: *mut Fighter_804D651C_t = core::ptr::null_mut();
pub static mut Fighter_804D6524: *mut Fighter_804D6524_t = core::ptr::null_mut();
pub static mut Fighter_SmashChargeShakeTable: *mut Fighter_ShakeTable_t = core::ptr::null_mut();
pub static mut Fighter_804D6534: *mut core::ffi::c_void = core::ptr::null_mut();
pub static mut Fighter_804D6538: *mut Fighter_804D653C_t = core::ptr::null_mut();
pub static mut Fighter_804D653C: *mut Fighter_804D653C_t = core::ptr::null_mut();
pub static mut Fighter_804D6548: *mut f32 = core::ptr::null_mut();
pub static mut Fighter_804D654C: *mut [f32; 5] = core::ptr::null_mut();
pub static mut Fighter_804D6550: *mut *mut i32 = core::ptr::null_mut();

/// Calls into code outside this TU. Callees that already have a native port in this crate forward to it;
/// the rest are `todo!` stubs named like the C function until their TU is ported.
#[allow(unused_variables, clippy::missing_safety_doc)]
pub mod ext {
    use crate::types::*;
    type G = *mut HSD_GObj;
    type J = *mut HSD_JObj;
    type F = *mut Fighter;
    #[inline(always)]
    unsafe fn fpg<'a>(g: G) -> &'a mut Fighter { &mut *((*g).user_data as *mut Fighter) }
    pub fn HSD_ObjAllocInit(d: *mut HSD_ObjAllocData, size: u32, align: u32) { unsafe { let _ = melee_runtime::tu::objalloc::HSD_ObjAllocInit(d as _, size as _, align as _); } }
    pub fn HSD_ObjAlloc(d: *mut HSD_ObjAllocData) -> *mut core::ffi::c_void { unsafe { melee_runtime::tu::objalloc::HSD_ObjAlloc(d as _) as _ } }
    pub fn HSD_ObjFree(d: *mut HSD_ObjAllocData, p: *mut core::ffi::c_void) { unsafe { let _ = melee_runtime::tu::objalloc::HSD_ObjFree(d as _, p as _); } }
    pub fn ftData_MexInitKinds() { unsafe { let _ = crate::tu::ftdata::ftData_MexInitKinds(); } }
    pub fn GenoDefine_InitKinds() { unsafe { let _ = crate::platform::GenoDefine_InitKinds(); } }
    pub fn ft_800852B0() { unsafe { let _ = crate::tu::ftdata::ft_800852B0(); } }
    pub fn ft_8008549C() { unsafe { let _ = crate::tu::ftdata::ft_8008549C(); } }
    pub fn ftCo_8009F4A4() { todo!("ftCo_8009F4A4 (ftCommon/ftCo_09F4.c)") }
    pub fn ftCo_800C8064() { unsafe { let _ = crate::tu::ftco_800c7ca0::ftco_800c8064(); } }
    pub fn ftCo_800C8F6C() { unsafe { let _ = crate::tu::ft_0c8c::ftco_800c8f6c(); } }
    pub fn ftData_Table_Unk1(kind: i32) -> Option<unsafe extern "C" fn()> { unsafe { crate::tu::ftdata::ftData_Table_Unk1[kind as usize] } }
    pub fn lbArchive_LoadSymbols_PlCo() -> *mut *mut core::ffi::c_void { todo!("lbArchive_LoadSymbols_PlCo (lb/lbarchive.c)") }
    pub fn Geno_DefineBaseKind(i: i32) -> i32 { unsafe { crate::platform::Geno_DefineBaseKind(i) } }
    pub fn GenoDefine_PartsTable(kind: i32) -> *mut FighterPartsTable { unsafe { crate::platform::GenoDefine_PartsTable(kind) } }
    pub fn Mex_InternalForPortKind(k: i32) -> i32 { unsafe { crate::platform::Mex_InternalForPortKind(k) } }
    pub fn Mex_FtSynth() -> i32 { unsafe { crate::platform::Mex_FtSynth() } }
    pub fn Mex_FtBaseKind(k: i32) -> i32 { unsafe { crate::platform::Mex_FtBaseKind(k) } }
    pub fn Mex_FtPartsTable(k: i32) -> usize { unsafe { crate::platform::Mex_FtPartsTable(k) } }
    pub fn Geno_DefineAiLike(kind: i32) -> i32 { unsafe { crate::platform::Geno_DefineAiLike(kind) } }
    pub fn gm_MexVanillaKind(kind: i32, ck: i32) -> i32 { todo!("gm_MexVanillaKind (gm)") }
    pub fn ftCommon_GetModelScale(fp: F) -> f32 { unsafe { crate::ftcommon2::get_model_scale(&*fp) } }
    pub fn ftCommon_800804EC(fp: F) -> f32 { unsafe { crate::ftcommon2::ftcommon_800804ec(&*fp) } }
    pub fn Player_LoadPlayerCoords(slot: i32, out: *mut Vec3) { unsafe { let _ = *out = crate::player::load_player_coords(slot); } }
    pub fn Player_GetFacingDirection(slot: i32) -> f32 { unsafe { crate::player::get_facing_direction(slot) } }
    pub fn Player_GetDamage(slot: i32) -> f32 { unsafe { crate::player::get_damage(slot) as f32 } }
    pub fn Player_GetModelScale(slot: i32) -> f32 { unsafe { crate::player::get_model_scale(slot) } }
    pub fn Player_GetPlayerId(slot: i32) -> u8 { unsafe { crate::player::get_player_id(slot) as u8 } }
    pub fn Player_GetControllerIndex(slot: i32) -> u8 { unsafe { crate::player::get_controller_index(slot) } }
    pub fn Player_GetFlagsBit3(slot: i32) -> u8 { unsafe { crate::player::get_flags_bit3(slot) } }
    pub fn Player_GetFlagsBit5(slot: i32) -> u8 { unsafe { crate::player::get_flags_bit5(slot) } }
    pub fn Player_GetFlagsBit6(slot: i32) -> u8 { unsafe { crate::player::get_flags_bit6(slot) } }
    pub fn Player_GetFlagsBit7(slot: i32) -> u8 { unsafe { crate::player::get_flags_bit7(slot) } }
    pub fn Player_GetMoreFlagsBit1(slot: i32) -> u8 { unsafe { crate::player::get_more_flags_bit1(slot) } }
    pub fn Player_GetMoreFlagsBit2(slot: i32) -> u8 { unsafe { crate::player::get_more_flags_bit2(slot) } }
    pub fn Player_GetMoreFlagsBit3(slot: i32) -> u8 { unsafe { crate::player::get_more_flags_bit3(slot) } }
    pub fn Player_GetMoreFlagsBit6(slot: i32) -> u8 { unsafe { crate::player::get_more_flags_bit6(slot) } }
    pub fn Player_GetFlagsAEBit0(slot: i32) -> u8 { unsafe { crate::player::get_flags_ae_bit0(slot) } }
    pub fn Player_GetCostumeId(slot: i32) -> i32 { unsafe { crate::player::get_costume_id(slot) as i32 } }
    pub fn Player_GetTeam(slot: i32) -> u8 { unsafe { crate::player::get_team(slot) as u8 } }
    pub fn Player_GetCpuType(slot: i32) -> i32 { unsafe { crate::player::get_cpu_type(slot) } }
    pub fn Player_GetCpuLevel(slot: i32) -> i32 { unsafe { crate::player::get_cpu_level(slot) } }
    pub fn Player_SetHPByIndex(slot: i32, sub: i32, hp: f32) { unsafe { let _ = crate::player::set_hp_by_index(slot, sub, melee_runtime::cabi::f2i32(hp as f64)); } }
    pub fn ScriptGame_HitRulePercentClear(fp: F) { todo!("ScriptGame_HitRulePercentClear (pc/gameworld)") }
    pub fn ftCo_800C88A0(fp: F) { unsafe { let _ = crate::tu::ft_0c88::ftco_800c88a0(&mut *fp); } }
    pub fn ftCo_800DEEA8(g: G) { unsafe { let _ = crate::ft0df::ftco_800deea8(g); } }
    pub fn Geno_FighterReset(fp: F) { todo!("Geno_FighterReset (pc/geno)") }
    pub fn ftCo_800D105C(g: G) { unsafe { let _ = crate::tu::ftchangeparam::ftco_800d105c(g); } }
    pub fn ftCo_800C09B4(g: G) { unsafe { let _ = crate::tu::ftCo_Bury::ftCo_800C09B4(g); } }
    pub fn ftCommon_8007E2FC(g: G) { unsafe { let _ = crate::ftcommon2::ftcommon_8007e2fc(fpg(g)); } }
    pub fn ft_80088A50(fp: F) { unsafe { let _ = crate::ft0881::ft_80088a50(&mut *fp); } }
    pub fn ft_800890BC(fp: F) { unsafe { let _ = crate::stale::clear_attack_id(&mut *(fp as *mut Fighter)); } }
    pub fn ft_800892D4(fp: F) { unsafe { let _ = crate::ft0892::ft_800892d4(&mut *fp); } }
    pub fn ft_80081B38(g: G) { unsafe { let _ = crate::tu::ft_081b::ft_80081b38(g); } }
    pub fn ft_80081938(g: G) { unsafe { let _ = crate::tu::ft_0819::ft_80081938(g); } }
    pub fn ft_80082A68(g: G) -> bool { unsafe { crate::tu::ft_081b::ft_80082a68(g) } }
    pub fn ftCommon_8007D6A4(fp: F) { unsafe { let _ = crate::ftcommon3::ftcommon_8007d6a4(&mut *fp); } }
    pub fn ftCommon_8007D5D4(fp: F) { unsafe { let _ = crate::ftcommon2::ftcommon_8007d5d4(&mut *fp); } }
    pub fn ftCamera_80076064(fp: F) { unsafe { let _ = crate::tu::ftcamera::ftcamera_80076064(&mut *fp); } }
    pub fn ftCo_800BFFAC(fp: F) { unsafe { let _ = crate::tu::ftcolanim::ftco_800bffac(&mut *fp); } }
    pub fn ftCo_800C0074(fp: F) { unsafe { let _ = crate::tu::ftcolanim::ftco_800c0074(&mut *fp); } }
    pub fn ftCo_800C8438(g: G) { unsafe { let _ = crate::tu::ftmetal::ftCo_800C8438(g); } }
    pub fn ftCo_800C89A0(g: G) { unsafe { let _ = crate::tu::ft_0c88::ftco_800c89a0(g); } }
    pub fn ftCo_800C8FC4(g: G) { unsafe { let _ = crate::tu::ft_0c8c::ftco_800c8fc4(g); } }
    pub fn ftColl_8007AFF8(g: G) { unsafe { let _ = crate::tu::ftcoll::ftcoll_8007aff8(g as _); } }
    pub fn ftColl_8007B0C0(g: G, s: i32) { unsafe { let _ = crate::tu::ftcoll::ftcoll_8007b0c0(g as _, s as _); } }
    pub fn Mex_OnDeathDispatch(kind: i32, g: G) { unsafe { let _ = crate::platform::Mex_OnDeathDispatch(kind, g); } }
    pub fn ftCo_800A101C(fp: F, cpu_type: i32, level: i32, z: i32) { todo!("ftCo_800A101C (ftCommon/ftCo_0A01.c)") }
    pub fn efAsync_QueueClear(q: *mut *mut core::ffi::c_void) { todo!("efAsync_QueueClear (ef/efasync.c)") }
    pub fn efAsync_QueueFlush(g: G, q: *mut *mut core::ffi::c_void) { todo!("efAsync_QueueFlush (ef/efasync.c)") }
    pub fn ft_8007C17C(g: G) { unsafe { let _ = crate::tu::ft_07c1::ft_8007c17c(g); } }
    pub fn ft_8007C630(g: G) { unsafe { let _ = crate::tu::ft_07c6::ft_8007c630(g); } }
    pub fn costume_joint(kind: i32, costume: u8) -> *mut HSD_Joint { unsafe { (*crate::tu::ftdata::CostumeListsForeachCharacter[kind as usize].costume_list.add(costume as usize)).joint } }
    pub fn costume_count(kind: i32) -> i32 { unsafe { crate::tu::ftdata::CostumeListsForeachCharacter[kind as usize].numCostumes as i32 } }
    pub fn ftPartsPObjSetDefaultClass() { todo!("ftPartsPObjSetDefaultClass (ft/ftparts.c)") }
    pub fn ftPartsPObjClearDefaultClass() { todo!("ftPartsPObjClearDefaultClass (ft/ftparts.c)") }
    pub fn HSD_JObjLoadJoint(j: *mut HSD_Joint) -> J { unsafe { melee_runtime::tu::jobj::HSD_JObjLoadJoint(j as _) as _ } }
    pub fn ftParts_80073758(j: J) { todo!("ftParts_80073758 (ft/ftparts.c)") }
    pub fn Geno_ToonUnlit(kind: i32) -> i32 { unsafe { crate::platform::Geno_ToonUnlit(kind) } }
    pub fn HSD_GObjObject_80390A70(g: G, kind: u8, o: *mut core::ffi::c_void) { todo!("HSD_GObjObject_80390A70 (baselib/gobjobject.c)") }
    pub fn HSD_GObj_JObjKind() -> u8 { todo!("HSD_GObj_JObjKind (baselib/gobj.c)") }
    pub fn lb_8000B1CC(j: J, off: *const Vec3, out: *mut Vec3) { unsafe { let _ = melee_runtime::tu::lb_00B0::lb_8000B1CC(j as _, off as _, out as _); } }
    pub fn Geno_FighterScale(fp: F) -> f32 { unsafe { crate::platform::Geno_FighterScale(fp) } }
    pub fn gFtDataList(kind: i32) -> *mut ftData { unsafe { crate::tu::ftdata::gFtDataList[kind as usize] } }
    pub fn ftCo_800D0FA0(g: G) { unsafe { let _ = crate::tu::ftchangeparam::ftco_800d0fa0(g); } }
    pub fn ftData_MotionStateList() -> *mut MotionState { todo!("ftData_MotionStateList (ft/ftdata.c)") }
    pub fn Mex_MoveLogicTable(kind: i32) -> *mut MotionState { unsafe { crate::platform::Mex_MoveLogicTable(kind) } }
    pub fn GenoDefine_BindFighter(fp: F) { unsafe { let _ = crate::platform::GenoDefine_BindFighter(fp); } }
    pub fn stage_grkind() -> i32 { todo!("stage_grkind (gr/stage.c stage_info)") }
    pub fn lbAnim_8001E8F8(t: *mut FigaTree) -> f32 { todo!("lbAnim_8001E8F8 (lb/lbanim.c)") }
    pub fn ftData_80085E50(fp: F, id: i32) -> *mut FigaTree { unsafe { crate::tu::ftdata::ftData_80085E50(fp, id) } }
    pub fn Geno_AnimBankBeforeSpawn() { unsafe { let _ = crate::platform::Geno_AnimBankBeforeSpawn(); } }
    pub fn GObj_Create(class: u16, p_link: u8, prio: u8) -> G { todo!("GObj_Create (baselib/gobj.c)") }
    pub fn GObj_SetupGXLink_ftDrawCommon(g: G) { todo!("GObj_SetupGXLink_ftDrawCommon (baselib/gobjgxlink.c + ft/ftdrawcommon.c)") }
    pub fn GObj_InitUserData_Fighter(g: G, fp: F) { todo!("GObj_InitUserData_Fighter (baselib/gobjuserdata.c)") }
    pub fn ftData_8008572C(kind: i32) { unsafe { let _ = crate::tu::ftdata::ftData_8008572C(kind); } }
    pub fn efAsync_LoadSync(bank: i32) { todo!("efAsync_LoadSync (ef/efasync.c)") }
    pub fn ftData_UnkBytePerCharacter(kind: i32) -> i32 { unsafe { crate::tu::ftdata::ftData_UnkBytePerCharacter[kind as usize] as i32 } }
    pub fn ftData_80085820(kind: i32, costume: u8) { unsafe { let _ = crate::tu::ftdata::ftData_80085820(kind, costume as i32); } }
    pub fn ftData_80085B10(fp: F) { unsafe { let _ = crate::tu::ftdata::ftData_80085B10(&mut *fp); } }
    pub fn ftParts_80074E58(fp: F) { unsafe { let _ = crate::tu::ftparts::ftParts_80074E58(&mut *fp); } }
    pub fn ftParts_SetupParts(g: G) { todo!("ftParts_SetupParts (ft/ftparts.c)") }
    pub fn ftCo_800C884C(g: G) { unsafe { let _ = crate::tu::ft_0c88::ftco_800c884c(g); } }
    pub fn ftParts_800749CC(g: G) { todo!("ftParts_800749CC (ft/ftparts.c)") }
    pub fn ftCo_8009CF84(fp: F) { unsafe { let _ = crate::tu::ftdynamics::ftCo_8009CF84(&mut *fp); } }
    pub fn ftCo_8009F578(fp: F) { todo!("ftCo_8009F578 (ftCommon/ftCo_09F4.c)") }
    pub fn Mex_OnLoadDispatch(kind: i32, g: G) { unsafe { let _ = crate::platform::Mex_OnLoadDispatch(kind, g); } }
    pub fn ftColl_8007B320(g: G) { unsafe { let _ = crate::tu::ftcoll::ftcoll_8007b320(g as _); } }
    pub fn Camera_80029020() -> *mut core::ffi::c_void { todo!("Camera_80029020 (cm/camera.c)") }
    pub fn lbShadow_8000ED54(s: *mut core::ffi::c_void, j: J) { unsafe { let _ = melee_runtime::tu::lbshadow::lbShadow_8000ED54(s as _, j as _); } }
    pub fn HSD_GObj_SetupProc(g: G, f: fn(G), prio: u8) { todo!("HSD_GObj_SetupProc (baselib/gobjproc.c)") }
    pub fn ftMh_MS_341_8014FE10(g: G) { todo!("ftMh_MS_341_8014FE10 (ftMasterHand)") }
    pub fn ftCh_Init_80155FCC(g: G) { todo!("ftCh_Init_80155FCC (ftCrazyHand)") }
    pub fn ftCo_800BFD04(g: G) { unsafe { let _ = crate::tu::ftcolanim::ftco_800bfd04(g); } }
    pub fn ftCo_800C61B0(g: G) { unsafe { let _ = crate::tu::ft_0c31::ftCo_800C61B0(g); } }
    pub fn ftCommon_8007D92C(g: G) { unsafe { let _ = crate::ftcommon3::ftcommon_8007d92c(g); } }
    pub fn ftLib_800867E8(g: G) { unsafe { let _ = crate::ftlib::ftlib_800867e8(g); } }
    pub fn ScriptGame_OnePSpawn(fp: F) { todo!("ScriptGame_OnePSpawn (pc/gameworld)") }
    pub fn PcTraceMotionEnabled() -> i32 { unsafe { crate::platform::PcTraceMotionEnabled() } }
    pub fn OSReport(s: &str) { todo!("OSReport (os)") }
    pub fn Geno_DefineIsNone(kind: i32) -> i32 { unsafe { crate::platform::Geno_DefineIsNone(kind) } }
    pub fn Geno_DefineThrownAs(kind: i32) -> i32 { unsafe { crate::platform::Geno_DefineThrownAs(kind) } }
    pub fn Geno_DefineThrownAsRow(kind: i32, row: i32) -> i32 { unsafe { crate::platform::Geno_DefineThrownAsRow(kind, row) } }
    pub fn ftData_ThrownStandIn(kind: i32) -> F { unsafe { crate::tu::ftdata::ftData_ThrownStandIn(kind) } }
    pub fn Script_GameEvent(what: i32, a: i32, b: i32, c: i32, d: i32) { unsafe { let _ = crate::platform::Script_GameEvent(what, a, b, c, d); } }
    pub fn Geno_OnActionChange(g: G) { todo!("Geno_OnActionChange (pc/geno)") }
    pub fn ft_8007C114(g: G) { unsafe { let _ = crate::tu::ft_07c1::ft_8007c114(g); } }
    pub fn ftColl_8007B62C(g: G, s: i32) { unsafe { let _ = crate::tu::ftcoll::ftcoll_8007b62c(g as _, s as _); } }
    pub fn ftColl_8007B4E0(g: G) { unsafe { let _ = crate::tu::ftcoll::ftcoll_8007b4e0(g as _); } }
    pub fn ftColl_8007B6EC(g: G) { unsafe { let _ = crate::tu::ftcoll::ftcoll_8007b6ec(g as _); } }
    pub fn ftColl_8007B760(g: G, v: i32) { unsafe { let _ = crate::tu::ftcoll::ftcoll_8007b760(g as _, v as _); } }
    pub fn ftColl_8007B8CC(fp: F, g: G) { unsafe { let _ = crate::ft0df::ftcoll_8007b8cc(&mut *fp, g); } }
    pub fn ftGetParasolStatus(g: G) -> i32 { unsafe { crate::ftcommon3::ft_get_parasol_status(g) } }
    pub fn ftCommon_8007E83C(g: G, a: i32, b: f32) { unsafe { let _ = crate::ftcommon3::ftcommon_8007e83c(g, a, b); } }
    pub fn ftCommon_8007ECD4(fp: F, a: i32) { unsafe { let _ = crate::ftcommon2::ftcommon_8007ecd4(&mut *fp, a); } }
    pub fn ftCommon_8007E2F4(fp: F, a: i32) { unsafe { let _ = crate::ftcommon2::ftcommon_8007e2f4(&mut *fp, a as i16); } }
    pub fn mpClearFloorSkip(c: *mut CollData) { unsafe { let _ = melee_world::tu::mpcoll::mpClearFloorSkip(c as _); } }
    pub fn ftCommon_8007F578(g: G) { unsafe { let _ = crate::ftcommon3::ftcommon_8007f578(g); } }
    pub fn ftCo_800C0134(fp: F) { unsafe { let _ = crate::tu::ftcolanim::ftco_800c0134(&mut *fp); } }
    pub fn ftCommon_8007DB24(g: G) { unsafe { let _ = crate::ftcommon3::ftcommon_8007db24(g); } }
    pub fn HSD_JObjRemoveAll(j: J) { todo!("HSD_JObjRemoveAll (baselib/jobj.c)") }
    pub fn Mex_OnLandingDispatch(kind: i32, g: G) { unsafe { let _ = crate::platform::Mex_OnLandingDispatch(kind, g); } }
    pub fn pl_8003FE1C(slot: i32, sub: i32) { unsafe { let _ = crate::plbonus::pl_8003fe1c(slot, sub); } }
    pub fn pl_80040330(slot: i32, sub: i32, v: f32) { unsafe { let _ = crate::plbonus::pl_80040330(slot, sub, v); } }
    pub fn ft_80088884(fp: F) { unsafe { let _ = crate::ft0881::ft_80088884(&mut *fp); } }
    pub fn ft_800888E0(fp: F) { unsafe { let _ = crate::ft0881::ft_800888e0(&mut *fp); } }
    pub fn ft_800887CC(fp: F) { unsafe { let _ = crate::ft0881::ft_800887cc(&mut *fp); } }
    pub fn ft_800890D0(fp: F, id: i32) { unsafe { let _ = crate::stale::new_attack(&mut *(fp as *mut Fighter), id as u32); } }
    pub fn ft_800895E0(fp: F, flags: i32) { unsafe { let _ = crate::ft0892::ft_800895e0(&mut *fp, crate::ft0892::s2070_from_gc_int(flags)); } }
    pub fn ftCo_800C8B2C(fp: F, a: i32, b: i32) -> bool { unsafe { crate::tu::ft_0c88::ftco_800c8b2c(&mut *fp, a, b != 0) } }
    pub fn pl_80037C60(g: G, v: Struct2070) { unsafe { let _ = crate::pltrick::pl_80037c60(g, v); } }
    pub fn ScriptGame_NoteMotion(fp: F, msid: i32) { todo!("ScriptGame_NoteMotion (pc/gameworld)") }
    pub fn Geno_MotionRow(fp: F, msid: i32) -> *mut MotionState { todo!("Geno_MotionRow (pc/geno)") }
    pub fn Geno_MotionAnimRow(fp: F, msid: i32) -> i32 { todo!("Geno_MotionAnimRow (pc/geno)") }
    pub fn ScriptGame_LabPreviewRow(fp: F, msid: i32) -> i32 { todo!("ScriptGame_LabPreviewRow (pc/gameworld)") }
    pub fn ftCo_8009E7B4(fp: F, p: *mut [u8; 2]) { unsafe { let _ = crate::tu::ftdynamics::ftCo_8009E7B4(&mut *fp, p as *const [u8; 2]); } }
    pub fn ftData_80085CD8(fp: F, src: F, anim: i32) { unsafe { let _ = crate::tu::ftdata::ftData_80085CD8(fp, src, anim); } }
    pub fn ftParts_JObjSetRotation(j: J, q: *const Quaternion) { unsafe { let _ = crate::tu::ftparts::ftParts_JObjSetRotation(j, q as *const Vec4); } }
    pub fn ftCo_800C0408(g: G) { unsafe { let _ = crate::tu::ftcolanim::ftco_800c0408(g); } }
    pub fn ftCommon_ClampGroundVel(fp: F, v: f32) { unsafe { let _ = crate::ftcommon::clamp_ground_vel(&mut *fp, v); } }
    pub fn ftCo_80090718(fp: F) { unsafe { let _ = crate::ftdmgfall::ftco_80090718(&mut *fp); } }
    pub fn ftCo_800C37A0(g: G) { unsafe { let _ = crate::tu::ftco_0c35::ftco_800c37a0(g); } }
    pub fn Fighter_SuperMushroomApply(g: G) -> bool { unsafe { crate::tu::ftCo_KinokoGiantStart::Fighter_SuperMushroomApply(g) } }
    pub fn Fighter_PoisonMushroomApply(g: G) -> bool { unsafe { crate::tu::ftCo_KinokoSmallStart::Fighter_PoisonMushroomApply(g) } }
    pub fn Fighter_SuperMushroomEnd(g: G) -> i32 { unsafe { crate::tu::ftCo_KinokoGiantEnd::Fighter_SuperMushroomEnd(g) } }
    pub fn Fighter_PoisonMushroomEnd(g: G) -> bool { unsafe { crate::tu::ftCo_KinokoSmallEnd::Fighter_PoisonMushroomEnd(g) } }
    pub fn ft_800819A8(g: G) { unsafe { let _ = crate::tu::ft_0819::ft_800819a8(g); } }
    pub fn Slippi_Codes() -> i32 { unsafe { crate::platform::Slippi_Codes() } }
    pub fn Camera_ScriptState() -> i32 { unsafe { crate::platform::Camera_ScriptState() } }
    pub fn ifMagnify_802FC998(slot: i32) -> bool { unsafe { melee_world::tu::ifmagnify::ifMagnify_802FC998(slot as _) } }
    pub fn gm_GetCurrentGameMode() -> i32 { todo!("gm_GetCurrentGameMode (gm)") }
    pub fn Stage_GetCamBoundsLeftOffset() -> f32 { unsafe { melee_world::tu::stage::Stage_GetCamBoundsLeftOffset() } }
    pub fn Stage_GetCamBoundsRightOffset() -> f32 { unsafe { melee_world::tu::stage::Stage_GetCamBoundsRightOffset() } }
    pub fn Stage_GetCamBoundsTopOffset() -> f32 { unsafe { melee_world::tu::stage::Stage_GetCamBoundsTopOffset() } }
    pub fn Stage_GetCamBoundsBottomOffset() -> f32 { unsafe { melee_world::tu::stage::Stage_GetCamBoundsBottomOffset() } }
    pub fn ftCo_800C0694(fp: F) -> i32 { unsafe { crate::ft0df::ftco_800c0694(&mut *fp) } }
    pub fn ftCo_800C0200(fp: F, a: i32) { unsafe { let _ = crate::tu::ftcolanim::ftco_800c0200(&mut *fp, a); } }
    pub fn it_8026B588() -> i32 { unsafe { crate::tu::ftcoll::ext::icd().xDC } }
    pub fn ft_800880D8(fp: F) { unsafe { let _ = crate::ft0877::ft_800880d8(&mut *fp); } }
    pub fn ftCommon_8007F8E8(g: G) { unsafe { let _ = crate::ftcommon3::ftcommon_8007f8e8(g); } }
    pub fn Item_8026A8EC(item: G) { unsafe { let _ = melee_items::tu::item::Item_8026A8EC(item as _); } }
    pub fn ftCommon_8007F9B4(g: G) { unsafe { let _ = crate::ftcommon3::ftcommon_8007f9b4(g); } }
    pub fn ftCo_800C8540(g: G) { unsafe { let _ = crate::tu::ftmetal::ftCo_800C8540(g); } }
    pub fn ftCo_800C9034(g: G) { unsafe { let _ = crate::tu::ft_0c8c::ftco_800c9034(g); } }
    pub fn Item_8026ABD8(item: G, v: *const Vec3, f: f32) { unsafe { let _ = melee_items::tu::item::Item_8026ABD8(item as _, v as _, f as _); } }
    pub fn ftCommon_8007FF74(g: G) { unsafe { let _ = crate::ftcommon2::ftcommon_8007ff74(fpg(g)); } }
    pub fn ftCo_800C8A64(g: G) { unsafe { let _ = crate::tu::ft_0c88::ftco_800c8a64(g); } }
    pub fn itGetKind(item: G) -> i32 { unsafe { crate::hooks::it_get_kind(item) } }
    pub fn ftCo_800968C8(g: G) { unsafe { let _ = crate::tu::ftCo_FallSpecial::ftCo_800968C8(g); } }
    pub fn ftCo_80095744(g: G, a: i32) { unsafe { let _ = crate::tu::ftCo_ItemThrow::ftCo_80095744(g, core::ptr::null_mut::<bool>()); } }
    pub fn Camera_80031144() -> f32 { todo!("Camera_80031144 (cm/camera.c)") }
    pub fn ft_80088640(fp: F, sfx: i32, vol: i32, pan: i32) { unsafe { let _ = crate::ft0881::ft_80088640(&mut *fp, sfx, vol as u8, pan as u8); } }
    pub fn pl_80040B8C(slot: i32, sub: i32, v: i32) { unsafe { let _ = crate::plbonus::pl_80040b8c(slot, sub, v); } }
    pub fn ftCo_800C511C(g: G) { unsafe { let _ = crate::tu::ftCo_HammerWait::ftCo_800C511C(g); } }
    pub fn Mex_OnFrameDispatch(kind: i32, g: G) { unsafe { let _ = crate::platform::Mex_OnFrameDispatch(kind, g); } }
    pub fn Geno_OnFrame(g: G) { todo!("Geno_OnFrame (pc/geno)") }
    pub fn ftCo_800D71D8(g: G) { unsafe { let _ = crate::tu::ftCo_Attack100::ftCo_800D71D8(g); } }
    pub fn ftColl_800764DC(g: G) { unsafe { let _ = crate::tu::ftcoll::ftcoll_800764dc(g as _); } }
    pub fn pl_800411C4(slot: i32, sub: i32) { unsafe { let _ = crate::plbonus::pl_800411c4(slot, sub); } }
    pub fn ftCo_800DEF38(g: G) { unsafe { let _ = crate::ft0df::ftco_800def38(g); } }
    pub fn Geno_PreAnim(g: G) -> i32 { todo!("Geno_PreAnim (pc/geno)") }
    pub fn ftCommon_8007E0E4(g: G) { unsafe { let _ = crate::ftcommon3::ftcommon_8007e0e4(g); } }
    pub fn ScriptGame_FighterBenchTick(fp: F) { todo!("ScriptGame_FighterBenchTick (pc/gameworld)") }
    pub fn ScriptGame_VirtualPadApply(fp: F) -> i32 { todo!("ScriptGame_VirtualPadApply (pc/gameworld)") }
    pub fn ScriptGame_VirtualPadObserve(fp: F) { todo!("ScriptGame_VirtualPadObserve (pc/gameworld)") }
    pub fn ftCo_IsCpuControlled(fp: F) -> bool { unsafe { crate::tu::ftCo_0A01::ftCo_IsCpuControlled(&*(fp as *const Fighter)) } }
    pub fn ftCo_800B3900(g: G) { todo!("ftCo_800B3900 (ftCommon/ftCo_0A01.c)") }
    pub fn ftCo_Jump_GetInput(g: G) -> bool { unsafe { crate::tu::ftCo_Jump::ftCo_Jump_GetInput(g) as i32 != 0 } }
    pub fn ftCo_800D6928(fp: F) -> bool { unsafe { crate::tu::ftCo_Attack100::ftCo_800D6928(&*fp) } }
    pub fn ftCo_800D688C(fp: F) -> bool { unsafe { crate::tu::ftCo_Attack100::ftCo_800D688C(&*fp) } }
    pub fn ftCo_SpecialS_HasInput(fp: F) -> bool { unsafe { crate::tu::ftCo_SpecialS::ftCo_SpecialS_HasInput(&*fp) } }
    pub fn ftCo_800D67C4(fp: F) -> bool { unsafe { crate::tu::ftco_0d67::ftco_800d67c4(&*fp) } }
    pub fn ftCo_GetCpuLStickX(fp: F) -> f32 { unsafe { crate::tu::ftCo_0A01::ftCo_GetCpuLStickX(&*(fp as *const Fighter)) } }
    pub fn ftCo_GetCpuLStickY(fp: F) -> f32 { unsafe { crate::tu::ftCo_0A01::ftCo_GetCpuLStickY(&*(fp as *const Fighter)) } }
    pub fn ftCo_GetCpuCStickX(fp: F) -> f32 { unsafe { crate::tu::ftCo_0A01::ftCo_GetCpuCStickX(&*(fp as *const Fighter)) } }
    pub fn ftCo_GetCpuCStickY(fp: F) -> f32 { unsafe { crate::tu::ftCo_0A01::ftCo_GetCpuCStickY(&*(fp as *const Fighter)) } }
    pub fn ftCo_GetCpuLTrigger(fp: F) -> f32 { unsafe { crate::tu::ftCo_0A01::ftCo_GetCpuLTrigger(&*(fp as *const Fighter)) } }
    pub fn ftCo_GetCpuRTrigger(fp: F) -> f32 { unsafe { crate::tu::ftCo_0A01::ftCo_GetCpuRTrigger(&*(fp as *const Fighter)) } }
    pub fn ftCo_GetCpuButtons(fp: F) -> u32 { unsafe { crate::tu::ftCo_0A01::ftCo_GetCpuButtons(&*(fp as *const Fighter)) } }
    pub fn DbLevel_below_DebugRom() -> bool { unsafe { crate::platform::DbLevel_below_DebugRom() } }
    pub fn Mex_Enabled(name: &str) -> bool { unsafe { crate::platform::Mex_Enabled(name) } }
    pub fn ScriptGame_CpuStanding(fp: F) -> i32 { todo!("ScriptGame_CpuStanding (pc/gameworld)") }
    pub fn gm_IsCurrently1PMode_inline() -> i32 { todo!("gm_IsCurrently1PMode_inline (gm)") }
    pub fn pad_nml(port: u8) -> [f32; 6] { todo!("pad_nml (baselib/controller.c HSD_PadGameStatus (stickX, stickY, subStickX, subStickY, analogL, analogR))") }
    pub fn pad_button(port: u8) -> u32 { todo!("pad_button (baselib/controller.c HSD_PadGameStatus.button)") }
    pub fn Settings_Int(key: &str, d: i32) -> i32 { unsafe { crate::platform::Settings_Int(key, d) } }
    pub fn gm_CStickSmashTargetTest() -> *mut bool { todo!("gm_CStickSmashTargetTest (gm)") }
    pub fn gm_GetDbPauseFlag(i: i32) -> bool { todo!("gm_GetDbPauseFlag (gm)") }
    pub fn Snap_InputHook(port: i32, fol: i32, v: *mut f32, b: *mut u32) -> i32 { unsafe { crate::platform::Snap_InputHook(port, fol, v, b) } }
    pub fn Replay_HasInput(port: i32, fol: i32) -> i32 { unsafe { crate::platform::Replay_HasInput(port, fol) } }
    pub fn Replay_StickX(port: i32, fol: i32) -> f32 { unsafe { crate::platform::Replay_StickX(port, fol) } }
    pub fn Replay_StickY(port: i32, fol: i32) -> f32 { unsafe { crate::platform::Replay_StickY(port, fol) } }
    pub fn Replay_CStickX(port: i32, fol: i32) -> f32 { unsafe { crate::platform::Replay_CStickX(port, fol) } }
    pub fn Replay_CStickY(port: i32, fol: i32) -> f32 { unsafe { crate::platform::Replay_CStickY(port, fol) } }
    pub fn Replay_Trigger(port: i32, fol: i32) -> f32 { unsafe { crate::platform::Replay_Trigger(port, fol) } }
    pub fn Replay_Buttons(port: i32, fol: i32) -> u32 { unsafe { crate::platform::Replay_Buttons(port, fol) } }
    pub fn Replay_RecordInput(port: i32, fol: i32, lx: f32, ly: f32, cx: f32, cy: f32, t: f32, b: u32, action: i32, x: f32, y: f32, facing: f32, pct: f32) { unsafe { let _ = crate::platform::Replay_RecordInput(port, fol, lx, ly, cx, cy, t, b, action, x, y, facing, pct); } }
    pub fn Replay_UcfCardinals(port: i32) -> i32 { unsafe { crate::platform::Replay_UcfCardinals(port) } }
    pub fn Replay_RawStick(port: i32, which: i32) -> i32 { unsafe { crate::platform::Replay_RawStick(port, which) } }
    pub fn lb_8000D148(a: f32, b: f32, c: f32, d: f32, e: f32, f: f32, g: f32) -> bool { unsafe { (melee_runtime::tu::lb_00CE::lb_8000D148(a as _, b as _, c as _, d as _, e as _, f as _, g as _) != 0 as _) } }
    pub fn Geno_ComboInput(g: G) { todo!("Geno_ComboInput (pc/geno)") }
    pub fn Geno_ComboTick(g: G) -> i32 { todo!("Geno_ComboTick (pc/geno)") }
    pub fn ScriptGame_IntrWinTick(g: G) { todo!("ScriptGame_IntrWinTick (pc/gameworld)") }
    pub fn ftCommon_8007FFD8(fp: F, v: f32) { unsafe { let _ = crate::ftcommon2::ftcommon_8007ffd8(&mut *fp, v); } }
    pub fn ftCo_800DF0D0(g: G) { unsafe { let _ = crate::ft0df::ftco_800df0d0(g); } }
    pub fn ftCommon_8008031C(g: G) { unsafe { let _ = crate::ftcommon2::ftcommon_8008031c(fpg(g)); } }
    pub fn ftCo_800C0A98(g: G) { unsafe { let _ = crate::tu::ftCo_Bury::ftCo_800C0A98(g); } }
    pub fn ftCommon_SandbagKnockbackDeaccel(v: f32, d: f32) -> f32 { unsafe { crate::ftcommon::sandbag_knockback_deaccel(v, d) } }
    pub fn ftCommon_SandbagGetKnockbackDeaccelX(fp: F) -> f32 { unsafe { crate::ftcommon2::sandbag_get_knockback_deaccel_x(&*fp) } }
    pub fn ftCommon_SandbagGetKnockbackDeaccelY(fp: F) -> f32 { unsafe { crate::ftcommon2::sandbag_get_knockback_deaccel_y(&*fp) } }
    pub fn ftCommon_ApplyGroundedKnockbackFriction(fp: F, f: f32) { unsafe { let _ = crate::ftcommon::apply_grounded_knockback_friction(&mut *fp, f); } }
    pub fn ftCommon_ApplyShieldKnockbackFriction(fp: F, f: f32) { unsafe { let _ = crate::ftcommon::apply_shield_knockback_friction(&mut *fp, f); } }
    pub fn ft_GetGroundFrictionMultiplier(fp: F) -> f32 { unsafe { crate::tu::ft_081b::ft_get_ground_friction_multiplier(&mut *fp) } }
    pub fn ftColl_GetWindOffsetVec(g: G, out: *mut Vec3) { unsafe { let _ = crate::tu::ftcoll::ftcoll_get_wind_offset_vec(g as _, &mut *(out as *mut _)); } }
    pub fn ftColl_80076528(g: G) { unsafe { let _ = crate::tu::ftcoll::ftcoll_80076528(g as _); } }
    pub fn mpGetSpeed(floor: i32, pos: *const Vec3, out: *mut Vec3) -> bool { unsafe { (melee_world::tu::mplib::mpGetSpeed(floor as _, pos as _, out as _) != 0 as _) } }
    pub fn ftCo_800D3158(g: G) { unsafe { let _ = crate::tu::ft_0d31::ftCo_800D3158(g); } }
    pub fn Stage_CalcUnkCamYBounds() -> f32 { unsafe { melee_world::tu::stage::Stage_CalcUnkCamYBounds() } }
    pub fn Stage_CalcUnkCamY() -> f32 { unsafe { melee_world::tu::stage::Stage_CalcUnkCamY() } }
    pub fn ft_PlaySFX(fp: F, sfx: i32, vol: i32, pan: i32) { unsafe { let _ = crate::ft0877::play_sfx(&mut *(fp as *mut _), sfx as _, vol as _, pan as _); } }
    pub fn un_80322258(x: f32) -> bool { unsafe { crate::tu::crowdsfx::un_80322258(x as _) } }
    pub fn ftColl_8007AF28(g: G) { unsafe { let _ = crate::ft0df::ftcoll_8007af28(g); } }
    pub fn DbLevel_at_least_DebugRom() -> bool { unsafe { crate::platform::DbLevel_at_least_DebugRom() } }
    pub fn HSD_JObjSetupMatrix(j: J) { todo!("HSD_JObjSetupMatrix (baselib/jobj.c)") }
    pub fn HSD_MtxInverse(src: *mut Mtx, dst: *mut Mtx) { unsafe { let _ = crate::bridge::hsd_mtx_inverse(src, dst); } }
    pub fn HSD_MtxSRT(m: *mut Mtx, s: *const Vec3, r: *const Vec3, t: *const Vec3, sc: *const Vec3) { unsafe { let _ = crate::bridge::hsd_mtx_srt(m, s, r, t, sc); } }
    pub fn PSMTXConcat(a: *mut Mtx, b: *mut Mtx, ab: *mut Mtx) { unsafe { let _ = crate::bridge::psmtx_concat(a, b, ab); } }
    pub fn Replay_Tracing() -> i32 { unsafe { crate::platform::Replay_Tracing() } }
    pub fn Replay_TraceFighter(port: i32, fol: i32, ckind: i32, action: i32, x: f32, y: f32, facing: f32, pct: f32, stocks: i32, ax: f32, ay: f32, kx: f32, ky: f32, gx: f32) { unsafe { let _ = crate::platform::Replay_TraceFighter(port, fol, ckind, action, x, y, facing, pct, stocks, ax, ay, kx, ky, gx); } }
    pub fn Player_GetStocks(slot: i32) -> i32 { unsafe { crate::stale::stocks(slot) } }
    pub fn ftCommon_UnlockECB(fp: F) { unsafe { let _ = crate::ftcommon2::unlock_ecb(&mut *fp); } }
    pub fn Geno_CollBegin(g: G) { todo!("Geno_CollBegin (pc/geno)") }
    pub fn Geno_CollEnd(g: G) { todo!("Geno_CollEnd (pc/geno)") }
    pub fn ftKb_SpecialN_800F1D24(g: G) { todo!("ftKb_SpecialN_800F1D24 (ftKirby)") }
    pub fn pl_80041280(slot: i32, sub: i32) { unsafe { let _ = crate::plbonus::pl_80041280(slot, sub); } }
    pub fn Replay_TraceAtProcMap() -> i32 { unsafe { crate::platform::Replay_TraceAtProcMap() } }
    pub fn Snap_Curated() -> i32 { unsafe { crate::platform::Snap_Curated() } }
    pub fn Snap_CuratedMix(w: *const u32, n: i32) { unsafe { let _ = crate::platform::Snap_CuratedMix(w, n); } }
    pub fn ScriptGame_IntrWinHashWord(entity: i32) -> u32 { todo!("ScriptGame_IntrWinHashWord (pc/gameworld)") }
    pub fn ScriptGame_BuildHashWord() -> u32 { todo!("ScriptGame_BuildHashWord (pc/gameworld)") }
    pub fn Snap_CuratedRegion(tag: i32, va: usize, len: u32) { unsafe { let _ = crate::platform::Snap_CuratedRegion(tag, va, len); } }
    pub fn ft_80089B08(g: G) { unsafe { let _ = crate::tu::ft_0899::ft_80089B08(g); } }
    pub fn Mex_FighterCallbackDispatch(g: G, cb: HSD_GObjEvent) { unsafe { let _ = crate::platform::Mex_FighterCallbackDispatch(g, cb); } }
    pub fn ftColl_8007AE80(g: G) { unsafe { let _ = crate::tu::ftcoll::ftcoll_8007ae80(g as _); } }
    pub fn ft_8007C224(g: G) { unsafe { let _ = crate::tu::ft_07c1::ft_8007c224(g); } }
    pub fn ft_8007C6DC(g: G) { unsafe { let _ = crate::tu::ft_07c6::ft_8007c6dc(g); } }
    pub fn HSD_JObjAnimAll(j: J) { todo!("HSD_JObjAnimAll (baselib/jobj.c)") }
    pub fn ifMagnify_802FB6E8(slot: i32) -> i32 { unsafe { melee_world::tu::ifmagnify::ifMagnify_802FB6E8(slot as _) } }
    pub fn Stage_UnkSetVec3TCam_Offset(out: *mut Vec3) { unsafe { let _ = melee_world::tu::stage::Stage_UnkSetVec3TCam_Offset(out as _); } }
    pub fn gm_8016B1C4() -> bool { todo!("gm_8016B1C4 (gm)") }
    pub fn ftColl_8007BA0C(g: G) { unsafe { let _ = crate::tu::ftcoll::ftcoll_8007ba0c(g as _); } }
    pub fn ftColl_80078A2C(g: G) { unsafe { let _ = crate::tu::ftcoll::ftcoll_80078a2c(g as _); } }
    pub fn ftColl_80078754(g: G, v: G, a: i32) { unsafe { let _ = crate::tu::ftcoll::ftcoll_80078754(g as _, v as _, (a != 0 as _)); } }
    pub fn ftColl_8007BC90(g: G) { unsafe { let _ = crate::tu::ftcoll::ftcoll_8007bc90(g as _); } }
    pub fn it_8027B4A4(g: G, item: G) { unsafe { let _ = melee_items::tu::it_279C::it_8027B4A4(g as _, item as _); } }
    pub fn ftColl_800765E0() { unsafe { let _ = crate::ftcoll::ftcoll_800765e0(); } }
    pub fn ftColl_80078C70(g: G) { unsafe { let _ = crate::tu::ftcoll::ftcoll_80078c70(g as _); } }
    pub fn ft_8007C77C(g: G) { unsafe { let _ = crate::tu::ft_07c6::ft_8007c77c(g); } }
    pub fn ftColl_8007925C(g: G) { unsafe { let _ = crate::tu::ftcoll::ftcoll_8007925c(g as _); } }
    pub fn ftColl_8007BAC0(g: G) { unsafe { let _ = crate::tu::ftcoll::ftcoll_8007bac0(g as _); } }
    pub fn ft_8007C4BC(g: G) { unsafe { let _ = crate::tu::ft_07c1::ft_8007c4bc(g); } }
    pub fn ftColl_8007AB48(g: G) { unsafe { let _ = crate::tu::ftcoll::ftcoll_8007ab48(g as _); } }
    pub fn ftColl_8007AB80(g: G) { unsafe { let _ = crate::tu::ftcoll::ftcoll_8007ab80(g as _); } }
    pub fn ftColl_8007BBCC(g: G) -> f32 { unsafe { crate::tu::ftcoll::ftcoll_8007bbcc(g as _) } }
    pub fn ftCommon_8007FC7C(g: G, f: f32) { unsafe { let _ = crate::ftcommon3::ftcommon_8007fc7c(g, f); } }
    pub fn ScriptGame_HitRulePercentCommit(fp: F, d: f32) -> f32 { todo!("ScriptGame_HitRulePercentCommit (pc/gameworld)") }
    pub fn ftCommon_8007EA90(fp: F, d: f32) { unsafe { let _ = crate::ftcommon3::ftcommon_8007ea90(&mut *fp, melee_runtime::cabi::f2i32(d as f64)); } }
    pub fn ScriptGame_ModDamage(slot: i32, k: i32, d: f32) -> f32 { todo!("ScriptGame_ModDamage (pc/gameworld)") }
    pub fn pl_8003EC9C(slot: i32, sub: i32, pct: f32, d: f32) { unsafe { let _ = crate::plbonus2::pl_8003ec9c(slot, sub, pct, d); } }
    pub fn ftCo_800C8C84(g: G) { unsafe { let _ = crate::tu::ft_0c8c::ftco_800c8c84(g); } }
    pub fn itIsHeavy(item: G) -> bool { unsafe { melee_items::tu::it_26B1::itIsHeavy(item as _) } }
    pub fn ftCo_8008E984(fp: F) -> bool { unsafe { crate::tu::ftCo_Damage::ftCo_8008E984(&*(fp as *const _)) } }
    pub fn HSD_Randi(n: i32) -> i32 { unsafe { crate::rng::HSD_Randi(n) } }
    pub fn it_8026B30C(item: G) -> i32 { unsafe { crate::hooks::it_8026b30c(item) } }
    pub fn it_8026B594(item: G) -> bool { unsafe { crate::tu::ft_0cdf::it_8026b594(item) } }
    pub fn ftCo_800BFFD0(fp: F, a: i32, b: i32) -> bool { unsafe { crate::tu::ftcolanim::ftco_800bffd0(&mut *fp, a, b) } }
    pub fn ftCommon_8007EBAC(fp: F, a: i32, b: i32) { unsafe { let _ = crate::ftcommon2::ftcommon_8007ebac(&mut *fp, a as u32, b as u32); } }
    pub fn Geno_OnHit(g: G) -> i32 { todo!("Geno_OnHit (pc/geno)") }
    pub fn Geno_HitReact(fp: F) -> i32 { todo!("Geno_HitReact (pc/geno)") }
    pub fn pl_8003E058(a: i32, b: i32, slot: i32, sub: i32) { unsafe { let _ = crate::plbonus2::pl_8003e058(a, b, slot, sub); } }
    pub fn ftColl_8007BE3C(g: G) { unsafe { let _ = crate::tu::ftcoll::ftcoll_8007be3c(g as _); } }
    pub fn ftCo_Damage_CalcKnockback(fp: F) { unsafe { let _ = crate::tu::ftCo_Damage::ftCo_Damage_CalcKnockback(&mut *(fp as *mut _)); } }
    pub fn ftKb_SpecialN_800F5BA4(fp: F) { todo!("ftKb_SpecialN_800F5BA4 (ftKirby)") }
    pub fn ftKb_SpecialN_800F5C34(fp: F) { todo!("ftKb_SpecialN_800F5C34 (ftKirby)") }
    pub fn ftCo_8008EC90(g: G) { unsafe { let _ = crate::tu::ftCo_Damage::ftCo_8008EC90(g as _); } }
    pub fn ftCommon_8007DB58(g: G) { unsafe { let _ = crate::ftcommon3::ftcommon_8007db58(g); } }
    pub fn ftCo_8008E908(g: G, f: f32) { unsafe { let _ = crate::tu::ftCo_Damage::ftCo_8008E908(g as _, f as _); } }
    pub fn ftCo_8008E9D0(g: G) { unsafe { let _ = crate::tu::ftCo_Damage::ftCo_8008E9D0(g as _); } }
    pub fn ftCo_8008EB58(g: G) { unsafe { let _ = crate::tu::ftCo_Damage::ftCo_8008EB58(g as _); } }
    pub fn ScriptGame_ArmorAbsorbed(fp: F) -> i32 { todo!("ScriptGame_ArmorAbsorbed (pc/gameworld)") }
    pub fn ftCo_80090594(fp: F, elem: i32, applied: i32, msid: i32, ga: i32, vib: f32) { unsafe { let _ = crate::ftdmgfall::ftco_80090594(&mut *fp, elem, applied, msid, ga, vib); } }
    pub fn ftCommon_8007ED50(fp: F, f: f32) { unsafe { let _ = crate::ftcommon2::ftcommon_8007ed50(&mut *fp, melee_runtime::cabi::f2i32(f as f64)); } }
    pub fn ftMh_MS_341_8014FE58(g: G) { todo!("ftMh_MS_341_8014FE58 (ftMasterHand)") }
    pub fn ftCh_Init_80156014(g: G) { todo!("ftCh_Init_80156014 (ftCrazyHand)") }
    pub fn ftCo_80098B20(g: G) { unsafe { let _ = crate::tu::ftCo_ShieldBreakFly::ftCo_80098B20(g); } }
    pub fn ftCo_80099D9C(g: G) { unsafe { let _ = crate::tu::ftCo_Rebound::ftCo_80099D9C(g); } }
    pub fn ftCommon_8007EE0C(fp: F, a: i32) { unsafe { let _ = crate::ftcommon2::ftcommon_8007ee0c(&mut *fp, a); } }
    pub fn ftCo_80098C9C(g: G) { unsafe { let _ = crate::tu::ftCo_ShieldBreakFly::ftCo_80098C9C(g); } }
    pub fn Mex_OnAbsorbDispatch(kind: i32, g: G) { unsafe { let _ = crate::platform::Mex_OnAbsorbDispatch(kind, g); } }
    pub fn ftCommon_800804FC(fp: F) { unsafe { let _ = crate::ftcommon2::ftcommon_800804fc(&mut *fp); } }
    pub fn ftCo_800C8D00(g: G) { unsafe { let _ = crate::tu::ft_0c8c::ftco_800c8d00(g); } }
    pub fn ftCommon_CalcHitlag(dmg: i32, msid: i32, mul: f32) -> f32 { unsafe { crate::ftcommon::calc_hitlag(dmg, msid, mul) } }
    pub fn Geno_NoHitlag(fp: F) -> i32 { todo!("Geno_NoHitlag (pc/geno)") }
    pub fn ftCommon_8007E2A4(g: G) { unsafe { let _ = crate::ftcommon2::ftcommon_8007e2a4(fpg(g)); } }
    pub fn ftCo_800C2FD8(g: G) { unsafe { let _ = crate::tu::ftafterimage::ftCo_800C2FD8(g); } }
    pub fn ftCo_800A0DA4(fp: F) { unsafe { let _ = crate::tu::ftCo_0A01::ftCo_800A0DA4(&mut *(fp as *mut Fighter)); } }
    pub fn ftCo_8009E0A8(g: G) { unsafe { let _ = crate::tu::ftdynamics::ftCo_8009E0A8(g); } }
    pub fn ftCommon_8008021C(g: G) { unsafe { let _ = crate::ftcommon2::ftcommon_8008021c(fpg(g)); } }
    pub fn Player_80032828(slot: i32, sub: i32, pos: *const Vec3) { unsafe { let _ = crate::player::player_80032828(slot, sub, &*pos); } }
    pub fn Player_SetFacingDirectionConditional(slot: i32, sub: i32, f: f32) { unsafe { let _ = crate::player::set_facing_direction_conditional(slot, sub, f); } }
    pub fn pl_8003FAA8(slot: i32, sub: i32, a: *const Vec3, b: *const Vec3) { unsafe { let _ = crate::tu::plbonuslib::pl_8003FAA8(slot as _, sub as _, a as _, b as _); } }
    pub fn Mex_OnDestroyDispatch(kind: i32, g: G) { unsafe { let _ = crate::platform::Mex_OnDestroyDispatch(kind, g); } }
    pub fn ftColl_8007B8E8(g: G) { unsafe { let _ = crate::tu::ftcoll::ftcoll_8007b8e8(g as _); } }
    pub fn it_8026B7F8(g: G) { unsafe { let _ = melee_items::tu::it_26B1::it_8026B7F8(g as _); } }
    pub fn Camera_800290D4(c: *mut core::ffi::c_void) { todo!("Camera_800290D4 (cm/camera.c)") }
    pub fn ftCo_UnloadDynamicBones(fp: F) { unsafe { let _ = crate::tu::ftdynamics::ftCo_UnloadDynamicBones(&mut *fp); } }
    pub fn ftColl_800765AC(g: G) { unsafe { let _ = crate::tu::ftcoll::ftcoll_800765ac(g as _); } }
    pub fn ft_80088C5C(g: G) { unsafe { let _ = crate::ft0881::ft_80088c5c(g); } }
    pub fn lbShadow_8000EE8C(s: *mut core::ffi::c_void) { unsafe { let _ = melee_runtime::tu::lbshadow::lbShadow_8000EE8C(s as _); } }
    pub fn HSD_JObjUnref(j: J) { unsafe { let _ = melee_runtime::tu::jobj::HSD_JObjUnref(j as _); } }
    pub fn ftData_800859A8(fp: F) { unsafe { let _ = crate::tu::ftdata::ftData_800859A8(fp); } }
    pub fn HSD_LObjRemoveAll(l: *mut HSD_LObj) { todo!("HSD_LObjRemoveAll (baselib/lobj.c)") }
    pub fn Player_80031FB0(slot: i32, sub: i32) { unsafe { let _ = crate::player::player_80031fb0(slot, sub); } }
    pub fn ScriptGame_HitRuleRetire(fp: F, size: usize) { todo!("ScriptGame_HitRuleRetire (pc/gameworld)") }
    pub fn Player_GetEntityAtIndex(slot: i32, i: i32) -> G { unsafe { crate::player::get_entity_at_index(slot, i) } }
    pub fn Player_SetStocks(slot: i32, n: i32) { unsafe { let _ = crate::stale::set_stocks(slot, n); } }
    pub fn GenoDefine_StateDigest(fp: F) -> u32 { todo!("GenoDefine_StateDigest (pc/geno)") }
    pub fn Geno_ArtStableKind(kind: i32) -> u32 { todo!("Geno_ArtStableKind (pc/geno)") }
    pub fn HSD_RandSeedPtr() -> *mut u32 { unsafe { crate::rng::HSD_RandSeedPtr() } }
    pub fn Replay_ChecksumSeed(arrived: u32, frame: i32) -> u32 { unsafe { crate::platform::Replay_ChecksumSeed(arrived, frame) } }
    pub fn RbHashWord(value: u32, symbol: &str) { unsafe { let _ = crate::platform::RbHashWord(value, symbol); } }
    pub fn RbHashRegion(region: i32, slot: i32) { unsafe { let _ = crate::platform::RbHashRegion(region, slot); } }
    pub fn RbHashSeed(arrived: u32, effective: u32) { unsafe { let _ = crate::platform::RbHashSeed(arrived, effective); } }
    pub fn RbHashLegacyEnv() -> i32 { unsafe { crate::platform::RbHashLegacyEnv() } }
    pub fn Snap_XRegion(tag: i32, va: usize, len: u32) { unsafe { let _ = crate::platform::Snap_XRegion(tag, va, len); } }
    pub fn Netplay_Enabled() -> i32 { unsafe { crate::platform::Netplay_Enabled() } }
    pub fn RB_Enabled() -> i32 { unsafe { crate::platform::RB_Enabled() } }
    pub fn Replay_Active() -> i32 { unsafe { crate::platform::Replay_Active() } }
    pub fn Player_GetPlayerSlotType(slot: i32) -> i32 { unsafe { crate::tu::player::Player_GetPlayerSlotType(slot as _) as _ } }
    pub fn Player_GetEntity(slot: i32) -> G { unsafe { crate::player::get_entity(slot) } }
    pub fn mpCheckFloor(ax: f32, ay: f32, bx: f32, by: f32, y_off: f32, floor: *mut Vec3, line: *mut i32, flags: *mut u32, normal: *mut Vec3, floor_skip: i32, joint_skip: i32, joint_only: i32, cb: *mut core::ffi::c_void, gobj: G) -> bool { todo!("mpCheckFloor (mp/mpcoll.c)") }
}

#[inline(always)]
pub(crate) unsafe fn fpm<'a>(g: G) -> &'a mut Fighter { &mut *((*g).user_data as F) }
#[inline(always)]
unsafe fn jobj_of(g: G) -> J { (*g).hsd_obj as J }
struct Dirty;
impl JObjDirty for Dirty {
    fn set_mtx_dirty(&mut self, j: &mut HSD_JObj) { super::ftanim::ext::HSD_JObjSetMtxDirtySub(j) }
    fn set_mtx_dirty_sub(&mut self, j: &mut HSD_JObj) { super::ftanim::ext::HSD_JObjSetMtxDirtySub(j) }
}
unsafe fn set_translate(j: J, v: &Vec3) { HSD_JObjSetTranslate(&mut *j, v, &mut Dirty) }
unsafe fn set_scale(j: J, v: &Vec3) { HSD_JObjSetScale(&mut *j, v, &mut Dirty) }
#[inline]
unsafe fn cd() -> &'static ftCommonData { h::common_data() }
fn v0() -> Vec3 { Vec3 { x: 0.0, y: 0.0, z: 0.0 } }

/// Clear (or set) a list of Fighter bitfields: `bits!(fp, 0; set_a, set_b)`.
macro_rules! bits {
    ($fp:expr, $v:expr; $($($s:ident).+),* $(,)?) => {{ $( $fp.$($s).+($v); )* }};
}

/// `Fighter_800679B0`: allocators, PlCo load, per-kind init hooks.
pub unsafe fn Fighter_800679B0() {
    ext::HSD_ObjAllocInit(&mut fighter_alloc_data, core::mem::size_of::<Fighter>() as u32, 4);
    ext::HSD_ObjAllocInit(&mut fighter_dat_attrs_alloc_data, 0x424, 4);
    ext::ftData_MexInitKinds();
    ext::GenoDefine_InitKinds();
    ext::ft_800852B0();
    Fighter_LoadCommonData();
    ext::ft_8008549C();
    ext::ftCo_8009F4A4();
    ext::ftCo_800C8064();
    ext::ftCo_800C8F6C();
    ext::HSD_ObjAllocInit(&mut fighter_parts_alloc_data, (MAX_FT_PARTS as usize * core::mem::size_of::<FighterBone>()) as u32, 4);
    ext::HSD_ObjAllocInit(&mut fighter_dobj_list_alloc_data, ((0x1f0 / 4) * core::mem::size_of::<*mut HSD_DObj>()) as u32, 4);
    ext::HSD_ObjAllocInit(&mut fighter_x2040_alloc_data, ((0x80 / 4) * core::mem::size_of::<*mut HSD_DObj>()) as u32, 4);
    crate::fighter::SPAWN_NUM_COUNTER = 1;
    for i in 0..Ft_Kind_Max as i32 {
        if let Some(f) = ext::ftData_Table_Unk1(i) {
            f();
        }
    }
}

/// `Fighter_FirstInitialize_80067A84` (TARGET_PC: FT_ANIM_BUF_SIZE figatree buffers).
pub unsafe fn Fighter_FirstInitialize_80067A84() {
    Fighter_800679B0();
    ext::HSD_ObjAllocInit(&mut fighter_x59C_alloc_data, FT_ANIM_BUF_SIZE as u32, 0x20);
}

static mut EXTEND_COPIES: [[*mut core::ffi::c_void; Ft_Kind_Max as usize]; 2] = [[core::ptr::null_mut(); Ft_Kind_Max as usize]; 2];

/// `ftCommonData_ExtendKindTable` (static, TARGET_PC): rebuild a PlCo kind table in port order.
pub unsafe fn ftCommonData_ExtendKindTable(loaded: *mut *mut core::ffi::c_void, slot: usize) -> *mut *mut core::ffi::c_void {
    let out = &mut EXTEND_COPIES[slot];
    for i in 0..Ft_Kind_Max as i32 {
        let base = ext::Geno_DefineBaseKind(i);
        if slot == 0 && base >= 0 {
            let own = ext::GenoDefine_PartsTable(i);
            if !own.is_null() {
                out[i as usize] = own as *mut _;
                continue;
            }
        }
        let mut k = ext::Mex_InternalForPortKind(if base >= 0 { base } else { i });
        if ext::Mex_FtSynth() != 0 {
            if i >= Ft_Kind_Mex0 as i32 && k >= 0 && ext::Mex_FtPartsTable(k) != 0 {
                out[i as usize] = if slot == 0 { ext::Mex_FtPartsTable(k) as *mut _ } else { core::ptr::null_mut() };
                continue;
            }
            k = if i < Ft_Kind_Mex0 as i32 { i } else if k >= 0 { ext::Mex_FtBaseKind(k) } else { -1 };
        }
        if k < 0 {
            k = if base >= 0 { base } else if i <= Ft_Kind_Mex0 as i32 { i } else { -1 };
        }
        out[i as usize] = if k >= 0 { *loaded.offset(k as isize) } else { core::ptr::null_mut() };
    }
    out.as_mut_ptr()
}

static mut CPU_COPY: core::mem::MaybeUninit<Fighter_804D64FC_t> = core::mem::MaybeUninit::uninit();
static mut CPU_TABLES: [[*mut core::ffi::c_void; Ft_Kind_Max as usize]; 8] = [[core::ptr::null_mut(); Ft_Kind_Max as usize]; 8];

/// `ftCommonData_ExtendCpuTables` (static, TARGET_PC): m-ex kinds take their AI-like row.
pub unsafe fn ftCommonData_ExtendCpuTables(src: *mut Fighter_804D64FC_t) -> *mut Fighter_804D64FC_t {
    if src.is_null() {
        return core::ptr::null_mut();
    }
    CPU_COPY.write(*src);
    let copy = CPU_COPY.as_mut_ptr();
    // x4..x20: eight kind-indexed pointer words in a row.
    let fields: [*mut *mut *mut core::ffi::c_void; 8] = [
        &mut (*copy).x4 as *mut _ as _, &mut (*copy).x8 as *mut _ as _, &mut (*copy).xC as *mut _ as _,
        &mut (*copy).x10 as *mut _ as _, &mut (*copy).x14 as *mut _ as _, &mut (*copy).x18 as *mut _ as _,
        &mut (*copy).x1C as *mut _ as _, &mut (*copy).x20 as *mut _ as _,
    ];
    for (j, f) in fields.iter().enumerate() {
        let loaded = **f;
        if loaded.is_null() {
            continue;
        }
        for i in 0..Ft_Kind_Max as i32 {
            let mut k = i;
            if i >= Ft_Kind_Mex0 as i32 {
                let like = ext::Geno_DefineAiLike(i);
                k = if like >= 0 { like } else { ext::gm_MexVanillaKind(i, 0) };
            }
            CPU_TABLES[j][i as usize] = if k >= 0 { *loaded.offset(k as isize) } else { core::ptr::null_mut() };
        }
        **f = CPU_TABLES[j].as_mut_ptr();
    }
    copy
}

/// `Fighter_LoadCommonData`: PlCo.dat's 23 symbols (TARGET_PC: kind tables rebuilt).
pub unsafe fn Fighter_LoadCommonData() {
    let p = ext::lbArchive_LoadSymbols_PlCo();
    let d = |i: isize| *p.offset(i);
    h::P_FT_COMMON_DATA = d(0) as *const ftCommonData;
    Fighter_804D6550 = d(1) as _;
    Fighter_804D654C = d(2) as _;
    Fighter_804D6548 = d(3) as _;
    h::FT_PARTS_TABLE = ftCommonData_ExtendKindTable(d(4) as _, 0) as _;
    h::FIGHTER_804D6540 = ftCommonData_ExtendKindTable(d(5) as _, 1) as _;
    Fighter_804D653C = d(6) as _;
    Fighter_804D6538 = d(7) as _;
    Fighter_804D6534 = d(8);
    h::FIGHTER_804D6530 = d(9) as _;
    h::FIGHTER_GRAB_MASH_SHAKE = d(10) as _;
    Fighter_SmashChargeShakeTable = d(11) as _;
    Fighter_804D6524 = d(12) as _;
    h::FIGHTER_804D6520 = d(13) as _;
    Fighter_804D651C = d(14) as _;
    Fighter_804D6518 = d(15) as _;
    Fighter_804D6514 = d(16) as _;
    Fighter_804D6510 = d(17);
    Fighter_804D650C = d(18) as _;
    Fighter_804D6508 = d(19) as _;
    Fighter_804D6504 = d(20) as _;
    gCrowdConfig = d(21) as _;
    Fighter_804D64FC = ftCommonData_ExtendCpuTables(d(22) as _);
}

/// `Fighter_UpdateModelScale`.
pub unsafe fn Fighter_UpdateModelScale(g: G) {
    let fp = fpm(g);
    let ms = ext::ftCommon_GetModelScale(fp);
    let s = Vec3 { x: if fp.x34_scale.z != 1.0 { fp.x34_scale.z } else { ms }, y: ms, z: ms };
    set_scale(jobj_of(g), &s);
}


/// `Fighter_UnkInitReset_80067C98`: respawn reset of the per-life state.
pub unsafe fn Fighter_UnkInitReset_80067C98(fp: &mut Fighter) {
    fp.x8_spawnNum = Fighter_NewSpawn_80068E40() as i32;
    let mut c = v0();
    let slot = fp.player_id as i32;
    ext::Player_LoadPlayerCoords(slot, &mut c);
    fp.facing_dir = ext::Player_GetFacingDirection(slot);
    c.x = fmadds(fp.facing_dir, ext::ftCommon_800804EC(fp), c.x); // fused
    fp.cur_pos = c;
    fp.prev_pos = c;
    fp.facing_dir1 = fp.facing_dir;
    fp.x34_scale.y = fp.x34_scale.x;
    bits!(fp, 0; set_x2220_b5, set_x2220_b6);
    fp.x200C = 0;
    fp.x2010 = 0;
    fp.x2008 = 0;
    bits!(fp, 0; set_x2219_b1, set_x2219_b2, set_x2219_b3, set_x2219_b4, set_x221A_b5, set_x221A_b6, set_x221D_b2,
        set_x221E_b7, set_x2220_b7, set_x2221_b4, set_x2221_b5, set_x2221_b7);
    fp.set_x2221_b6(1);
    fp.x61D = 255;
    fp.pos_delta = v0();
    fp.cur_anim_frame = 0.0;
    fp.x898_unk = 0.0;
    fp.frame_speed_mul = 1.0;
    fp.x8A0_unk = 1.0;
    fp.dmg.kb_applied = 0.0;
    fp.dmg.x18A4_knockbackMagnitude = 0.0;
    fp.dmg.x18A8 = 0.0;
    fp.dmg.x18ac_time_since_hit = -1;
    fp.dmg.armor0 = 0.0;
    fp.dmg.armor1 = 0.0;
    fp.x1828 = 0;
    fp.set_x221C_b6(0);
    fp.dmg.x18a0 = 0.0;
    fp.x1968_jumpsUsed = 0;
    fp.x1969_walljumpUsed = 0;
    fp.hitlag_mul = 0.0;
    fp.x2064_ledgeCooldown = 0;
    fp.dmg.x1830_percent = ext::Player_GetDamage(slot);
    ext::ScriptGame_HitRulePercentClear(fp);
    fp.dmg.x1838_percentTemp = 0.0;
    fp.dmg.x183C_applied = 0;
    fp.dmg.x18C0 = 0;
    fp.dmg.x18c4_source_ply = 6;
    fp.dmg.x18C8 = -1;
    fp.dmg.x18F0 = 0;
    fp.dmg.x18CC = 0;
    fp.dmg.x18D0 = -10;
    bits!(fp, 0; set_x221F_b5, set_x2221_b1);
    fp.dmg.x18F4 = 0;
    fp.dmg.x18F8 = 1;
    fp.dmg.x18fa_model_shift_frames = 0;
    fp.dmg.x18FD = 0;
    fp.dmg.x18FC = 0;
    fp.dmg.x1834 = 0.0;
    fp.set_x2222_b2(0);
    fp.dmg.x1840 = 0;
    bits!(fp, 0; set_x2219_b5, set_x2219_b6, set_x2219_b7, set_x221A_b0, set_x221A_b1);
    fp.dmg.x1954 = 0.0;
    fp.dmg.x1958 = 0.0;
    fp.set_allow_sdi(0);
    fp.dmg.x195c_hitlag_frames = 0.0;
    fp.set_x221A_b3(0);
    fp.x1960_vibrateMult = 1.0;
    fp.x1964 = 0.0;
    fp.dmg.x189C_unk_num_frames = 0.0;
    bits!(fp, 0; set_x2220_b3, set_x2220_b4);
    fp.dmg.x1914 = 0;
    fp.dmg.int_value = 0;
    fp.dmg.x191C = 0.0;
    fp.dmg.x1924 = 0;
    fp.dmg.x1928 = 0.0;
    fp.set_x2223_b5(0);
    fp.dmg.x1950 = 0;
    fp.dmg.x1948 = 0;
    fp.set_x2223_b4(0);
    fp.xF8_playerNudgeVel.y = 0.0;
    fp.xF8_playerNudgeVel.x = 0.0;
    fp.x100 = -1.0;
    bits!(fp, 0; set_x2222_b7, set_x2223_b0, set_fall_fast, set_x2219_b0);
    fp.x20A0_accessory = core::ptr::null_mut();
    fp.anon_2210.throw_flags = 0;
    fp.cmd_timer = 0.0;
    fp.item_gobj = core::ptr::null_mut();
    fp.x1978 = core::ptr::null_mut();
    fp.set_x221E_b3(1);
    fp.x1984_heldItemSpec = core::ptr::null_mut();
    fp.x1988 = 0;
    fp.x198C = 0;
    fp.set_x2221_b0(0);
    fp.x1990 = 0;
    fp.x1994 = 0;
    bits!(fp, 0; set_x221D_b6, anon_221b.set_x221B_b5);
    fp.victim_gobj = core::ptr::null_mut();
    fp.x1A5C = core::ptr::null_mut();
    fp.anon_221b.set_x221B_b6(0);
    fp.target_item_gobj = core::ptr::null_mut();
    fp.x1A64 = core::ptr::null_mut();
    bits!(fp, 0; anon_221b.set_x221B_b7, set_x221C_b0);
    fp.x1064_thrownHitbox.anon_134.owner = core::ptr::null_mut();
    fp.set_x221C_u16_y(0);
    fp.unk_gobj = core::ptr::null_mut();
    fp.set_x221C_b5(0);
    fp.x2150 = -1;
    fp.x2154 = -1;
    fp.x2158 = -1;
    fp.x215C = -1;
    fp.x2160 = -1;
    fp.x2144 = -1;
    fp.x2148 = -1;
    fp.x214C = -1;
    fp.x2168 = 0;
    fp.x2164 = 0;
    fp.x208C = 0;
    fp.x2090 = 0;
    fp.x2098 = 0;
    fp.x2092 = 0;
    fp.x2094 = core::ptr::null_mut();
    fp.shield_health = h::script_value(fp, 4, cd().x260_startShieldHealth);
    bits!(fp, 0; set_x221A_b7, anon_221b.set_x221B_b0, anon_221b.set_x221B_b1, anon_221b.set_x221B_b3, anon_221b.set_x221B_b4, set_x221C_b3, set_x221C_b1, set_x221C_b2);
    fp.x19A0_shieldDamageTaken = 0;
    fp.x19A4 = 0;
    fp.lightshield_amount = 0.0;
    fp.x19A8 = core::ptr::null_mut();
    fp.shield_unk0 = 0.0;
    fp.shield_unk1 = 0.0;
    fp.x19BC_shieldDamageTaken3 = 6;
    bits!(fp, 0; set_x221F_b6, set_reflecting, set_x2218_b4);
    fp.ReflectAttr.x1A3C_damageOver = 0;
    fp.ReflectAttr.x1A2C_reflectHitDirection = 0.0;
    bits!(fp, 0; set_x2218_b6, set_x2218_b7);
    fp.AbsorbAttr.x1A40_absorbHitDirection = 0.0;
    fp.AbsorbAttr.x1A44_damageTaken = 0;
    fp.AbsorbAttr.x1A48_hitsTaken = 0;
    fp.x68C_transNPos = v0();
    fp.x6A4_transNOffset = v0();
    fp.lstick_angle = 0.0;
    fp.x6C0 = v0();
    fp.x6D8 = v0();
    fp.x209C = 0;
    fp.set_x2224_b1(0);
    fp.death2_cb = None;
    fp.x2100 = -1;
    fp.set_x2101_bits_0to6(0);
    fp.accessory2_cb = None;
    fp.accessory3_cb = None;
    fp.death1_cb = None;
    fp.death3_cb = None;
    fp.set_x221E_b4(1);
    fp.x197C = core::ptr::null_mut();
    fp.set_is_metal(0);
    fp.metal_timer = 0;
    fp.metal_health = 0;
    ext::ftCo_800C88A0(fp);
    fp.set_x2227_b3(0);
    fp.x2034 = 0;
    fp.x2038 = 0;
    fp.x1980 = core::ptr::null_mut();
    bits!(fp, 0; set_x2224_b2, set_x2224_b3, set_x2224_b4);
    fp.capture_timer = 0;
    fp.set_x2224_b5(0);
    fp.x1A53 = 0;
    fp.x1A52 = 0;
    fp.wall_jump_input_timer = 254;
    fp.dmg.x1910 = 0;
    fp.set_x2225_b0(0);
    fp.set_x2225_b2(1);
    fp.set_x2225_b4(0);
    ext::ftCo_800DEEA8(fp.gobj);
    fp.dmg.x18BC = 0.0;
    fp.dmg.x18B8 = 0.0;
    fp.set_x2226_b2(0);
    fp.x2170 = 0.0;
    let b5 = fp.x2225_b5();
    fp.set_x2225_b6(b5);
    fp.dmg.x1908 = -1;
    fp.dmg.x190C = core::ptr::null_mut();
    fp.set_x2227_b4(0);
    fp.smash_attrs.x2138_smashSinceHitbox = -1.0;
    fp.x213C = -1;
    bits!(fp, 0; set_x2227_b5, set_x2228_b1);
    fp.x2140 = 0.0;
    fp.set_x2227_b6(0);
    fp.x2180 = 6;
    fp.set_x2229_b4(1);
    ext::Geno_FighterReset(fp);
}

/// `Fighter_UnkProcessDeath_80068354`: (re)spawn setup.
pub unsafe fn Fighter_UnkProcessDeath_80068354(g: G) {
    let fp = fpm(g);
    Fighter_UnkInitReset_80067C98(fp);
    set_translate(jobj_of(g), &fp.cur_pos);
    ext::ftCo_800D105C(g);
    ext::ftCo_800C09B4(g);
    ext::ftCommon_8007E2FC(g);
    ext::ft_80088A50(fp);
    ext::ft_800890BC(fp);
    ext::ft_800892D4(fp);
    ext::ft_80081B38(g);
    ext::ft_80081938(g);
    if fp.smash_attrs.x2135 == -1 && ext::ft_80082A68(g) && fp.x2229_b6() == 0 {
        ext::ftCommon_8007D6A4(fp);
    } else {
        ext::ftCommon_8007D5D4(fp);
    }
    ext::ftCamera_80076064(fp);
    set_translate(jobj_of(g), &fp.cur_pos);
    Fighter_UnkApplyTransformation_8006C0F0(g);
    Fighter_UpdateModelScale(g);
    ext::ftCo_800BFFAC(fp);
    ext::ftCo_800C0074(fp);
    ext::ftCo_800C8438(g);
    ext::ftCo_800C89A0(g);
    ext::ftCo_800C8FC4(g);
    ext::ftColl_8007AFF8(g);
    ext::ftColl_8007B0C0(g, HurtCapsule_Enabled as i32);
    ext::Mex_OnDeathDispatch(fp.kind as i32, g);
    let slot = fp.player_id as i32;
    ext::ftCo_800A101C(fp, ext::Player_GetCpuType(slot), ext::Player_GetCpuLevel(slot), 0);
    ext::efAsync_QueueClear(&mut fp.x60C);
    ext::ft_8007C17C(g);
    ext::ft_8007C630(g);
}

/// `gw_toon_unlit_tree` (static, TARGET_PC): constant-colour render mode on every material.
pub unsafe fn gw_toon_unlit_tree(mut j: J) -> i32 {
    let mut n = 0;
    while !j.is_null() {
        if (*j).flags & (JOBJ_PTCL | JOBJ_SPLINE) as u32 == 0 {
            let mut d = (*j).u.dobj;
            while !d.is_null() {
                let m = (*d).mobj;
                if !m.is_null() {
                    (*m).rendermode = ((*m).rendermode & !((RENDER_CONSTANT | RENDER_VERTEX | RENDER_DIFFUSE | RENDER_SPECULAR) as u32)) | RENDER_CONSTANT as u32;
                    n += 1;
                }
                d = (*d).next;
            }
        }
        n += gw_toon_unlit_tree((*j).child);
        j = (*j).next;
    }
    n
}

/// `Fighter_UnkUpdateCostumeJoint_800686E4`.
pub unsafe fn Fighter_UnkUpdateCostumeJoint_800686E4(g: G) {
    let fp = fpm(g);
    fp.x108_costume_joint = ext::costume_joint(fp.kind as i32, fp.x619_costume_id);
    ext::ftPartsPObjSetDefaultClass();
    let j = ext::HSD_JObjLoadJoint(fp.x108_costume_joint);
    ext::ftPartsPObjClearDefaultClass();
    ext::ftParts_80073758(j);
    if ext::Geno_ToonUnlit(fp.kind as i32) != 0 {
        gw_toon_unlit_tree(j);
        ext::OSReport("geno toon: unlit base\n");
    }
    ext::HSD_GObjObject_80390A70(g, ext::HSD_GObj_JObjKind(), j as *mut _);
}

/// `Fighter_UnkUpdateVecFromBones_8006876C`.
pub unsafe fn Fighter_UnkUpdateVecFromBones_8006876C(fp: &mut Fighter) {
    let j = (*fp.parts.offset(crate::ftdmgfall::ftparts_get_bone_index(fp, 2) as isize)).joint;
    let mut v = v0();
    let mut v2 = v0();
    HSD_JObjGetTranslation(&*j, &mut v);
    fp.x1A6C = v.y / 8.55f32;
    ext::lb_8000B1CC(j, core::ptr::null(), &mut v);
    ext::lb_8000B1CC((*fp.parts.offset(crate::ftdmgfall::ftparts_get_bone_index(fp, 1) as isize)).joint, core::ptr::null(), &mut v2);
    fp.x1A70.x = v2.x - v.x;
    fp.x1A70.y = v2.y - v.y;
    fp.x1A70.z = v2.z - v.z;
}

fn reset_input_timers(fp: &mut Fighter) {
    for t in [&mut fp.active_timer, &mut fp.active_sticky, &mut fp.active_duration, &mut fp.activity_timer] {
        t.trigger = 254;
        t.lstick.y = 254;
        t.lstick.x = 254;
    }
    fp.x67C = 255;
    fp.x67D = 255;
    fp.x67E = 255;
    fp.x67F = 255;
    fp.x680 = 255;
    fp.x681 = 255;
    fp.x682 = 255;
    fp.x683 = 255;
    fp.x684 = 255;
    fp.x685 = 255;
    fp.x686 = 255;
    fp.x687 = 255;
    fp.x688 = 255;
    fp.x689 = 255;
    fp.x68A = 255;
    fp.x68B = 255;
}

/// `Fighter_ResetInputData_80068854`.
pub unsafe fn Fighter_ResetInputData_80068854(g: G) {
    let fp = fpm(g);
    for i in 0..2 {
        fp.input.lstick[i].x = 0.0;
        fp.input.lstick[i].y = 0.0;
        fp.input.cstick[i].x = 0.0;
        fp.input.cstick[i].y = 0.0;
        fp.input.triggers[i] = 0.0;
        fp.input.held_buttons[i] = 0;
    }
    fp.input.released_buttons = 0;
    fp.input.pressed_buttons = 0;
    reset_input_timers(fp);
}

/// `Fighter_UnkInitLoad_80068914_Inner1` (static): same stores as 80068854.
pub unsafe fn Fighter_UnkInitLoad_80068914_Inner1(g: G) { Fighter_ResetInputData_80068854(g) }

/// `Fighter_UnkInitLoad_80068914`: first-time fighter field setup from the alloc info.
pub unsafe fn Fighter_UnkInitLoad_80068914(g: G, arg: &plAllocInfo) {
    let fp = fpm(g);
    fp.kind = arg.internal_id;
    fp.player_id = arg.slot;
    fp.set_is_sub_fighter(arg.anon_6.b0());
    let slot = fp.player_id as i32;
    fp.x34_scale.x = ext::Player_GetModelScale(slot);
    fp.x34_scale.x *= ext::Geno_FighterScale(fp);
    fp.x61C = arg.x5;
    fp.x618_player_id = ext::Player_GetPlayerId(slot);
    fp.x61A_controller_index = ext::Player_GetControllerIndex(slot);
    fp.set_is_always_metal(ext::Player_GetFlagsBit5(slot));
    fp.set_x2226_b3(ext::Player_GetFlagsBit6(slot));
    fp.set_x2226_b6(ext::Player_GetFlagsBit7(slot));
    fp.set_x2225_b5(ext::Player_GetMoreFlagsBit1(slot));
    fp.set_x2225_b7(ext::Player_GetMoreFlagsBit2(slot));
    fp.set_x2228_b3(ext::Player_GetMoreFlagsBit6(slot));
    fp.set_x2229_b1(ext::Player_GetFlagsAEBit0(slot));
    if fp.x61A_controller_index > 4 {
        panic!("fighter.c:828: fighter sub color num over!");
    }
    if fp.x61A_controller_index != 0 {
        let c = cd().x6DC_colorsByPlayer[fp.x61A_controller_index as usize - 1];
        let o = &mut fp.x610_color_rgba[0];
        o.r = ((c.r as i32 * c.a as i32) / 255) as u8;
        o.g = ((c.g as i32 * c.a as i32) / 255) as u8;
        o.b = ((c.b as i32 * c.a as i32) / 255) as u8;
        o.a = c.a;
    }
    let mut costume = ext::Player_GetCostumeId(slot);
    if costume >= ext::costume_count(fp.kind as i32) {
        costume = 0;
    }
    fp.x619_costume_id = costume as u8;
    fp.team = ext::Player_GetTeam(slot);
    fp.gobj = g;
    fp.ft_data = ext::gFtDataList(fp.kind as i32);
    ext::ftCo_800D0FA0(g);
    fp.x2CC = core::ptr::null_mut();
    fp.x2D0 = core::ptr::null_mut();
    fp.x18 = 0x155;
    fp.x1C_actionStateList = ext::ftData_MotionStateList();
    fp.x20_actionStateList = ext::Mex_MoveLogicTable(fp.kind as i32);
    fp.x24 = (*fp.ft_data).xC;
    ext::GenoDefine_BindFighter(fp);
    fp.x28 = (*fp.ft_data).x10;
    fp.input.lstick[2].y = 0.0;
    fp.input.lstick[2].x = 0.0;
    fp.input.cstick[2].y = 0.0;
    fp.input.cstick[2].x = 0.0;
    fp.input.triggers[2] = 0.0;
    fp.input.held_buttons[2] = 0;
    Fighter_UnkInitLoad_80068914_Inner1(g);
    fp.anon_594.x594_s32 = 0;
    fp.x21FC_flag.byte = 1;
    bits!(fp, 0; set_invisible, set_x221E_b1, set_x221E_b2, set_x221F_b1, set_x221F_b2);
    fp.x209A = 0;
    bits!(fp, 0; set_x221E_b5, set_x221F_b0);
    fp.x21EC = None;
    bits!(fp, 0; set_x221D_b3, set_x221D_b4, set_x221F_b3, set_x2220_b0, set_x2221_b2, set_no_normal_motion, set_x2229_b6,
        set_no_kb, set_x222A_b0, set_x222A_b1, set_x2228_b5, set_used_tether, set_x2221_b3, set_x2222_b0, set_can_multijump,
        set_x2222_b4, set_x2222_b5, set_x2222_b6, set_x2223_b1);
    fp.x40 = 0.0;
    fp.set_can_walljump(0);
    fp.x60C = core::ptr::null_mut();
    bits!(fp, 0; set_x2225_b3, set_is_sandbag, set_x2226_b0, set_x2226_b1, set_x2227_b0, set_x2224_b0);
    fp.smash_attrs.x2135 = -1;
    fp.x2184 = core::ptr::null_mut();
    fp.set_x2229_b3(0);
}

/// `Fighter_80068E64`: Flat Zone flattens the fighter.
pub unsafe fn Fighter_80068E64(g: G) {
    let fp = fpm(g);
    fp.x34_scale.z = if ext::stage_grkind() == Gr_Kind_Flatzone as i32 { cd().x7E4_scaleZ } else { 1.0 };
}

/// `Fighter_Create_Inline2` (static): cache the common figatrees.
pub unsafe fn Fighter_Create_Inline2(g: G) {
    let fp = fpm(g);
    if fp.no_normal_motion() == 0 {
        fp.x2EC = ext::lbAnim_8001E8F8(ext::ftData_80085E50(fp, 0x23));
        if fp.is_sandbag() == 0 {
            fp.x2DC = ext::lbAnim_8001E8F8(ext::ftData_80085E50(fp, 7));
            fp.x2E0 = ext::lbAnim_8001E8F8(ext::ftData_80085E50(fp, 8));
            fp.x2E4 = ext::lbAnim_8001E8F8(ext::ftData_80085E50(fp, 9));
            fp.x2E8 = ext::lbAnim_8001E8F8(ext::ftData_80085E50(fp, 0x25));
        }
    }
}

fn proc_8006A1BC(g: G) { unsafe { Fighter_8006A1BC(g) } }
fn proc_8006A360(g: G) { unsafe { Fighter_8006A360(g) } }
fn proc_8006ABA0(g: G) { unsafe { Fighter_8006ABA0(g) } }
fn proc_spaghetti(g: G) { unsafe { Fighter_Spaghetti_8006AD10(g) } }
fn proc_update(g: G) { unsafe { Fighter_procUpdate(g) } }
fn proc_map(g: G) { unsafe { Fighter_procMap(g) } }
fn proc_8006C5F4(g: G) { unsafe { Fighter_8006C5F4(g) } }
fn proc_accessory(g: G) { unsafe { Fighter_CallAcessoryCallbacks_8006C624(g) } }
fn proc_8006C80C(g: G) { unsafe { Fighter_8006C80C(g) } }
fn proc_grab(g: G) { unsafe { Fighter_UnkProcessGrab_8006CA5C(g) } }
fn proc_8006CB94(g: G) { unsafe { Fighter_8006CB94(g) } }
fn proc_hit(g: G) { unsafe { Fighter_ProcessHit_8006D1EC(g) } }
fn proc_8006D9AC(g: G) { unsafe { Fighter_8006D9AC(g) } }
fn proc_camera(g: G) { unsafe { Fighter_UnkCallCameraCallback_8006D9EC(g) } }
fn proc_8006DA4C(g: G) { unsafe { Fighter_8006DA4C(g) } }

/// `Fighter_Create`: allocate, load and register a fighter GObj with its 15 procs.
pub unsafe fn Fighter_Create(input: &plAllocInfo) -> G {
    ext::Geno_AnimBankBeforeSpawn();
    let g = ext::GObj_Create(HSD_GOBJ_CLASS_FIGHTER as u16, 8, 0);
    ext::GObj_SetupGXLink_ftDrawCommon(g);
    let fp = ext::HSD_ObjAlloc(&mut fighter_alloc_data) as F;
    core::ptr::write_bytes(fp, 0, 1); // TARGET_PC: Slippi's Init Player Data zeroes the block
    (*fp).dat_attrs_backup = ext::HSD_ObjAlloc(&mut fighter_dat_attrs_alloc_data) as _;
    ext::GObj_InitUserData_Fighter(g, fp);
    ext::ftData_8008572C(input.internal_id as i32);
    Fighter_UnkInitLoad_80068914(g, input);
    let fp = &mut *fp;
    ext::efAsync_LoadSync(ext::ftData_UnkBytePerCharacter(fp.kind as i32));
    ext::ftData_80085820(fp.kind as i32, fp.x619_costume_id);
    Fighter_UnkUpdateCostumeJoint_800686E4(g);
    ext::ftData_80085B10(fp);
    ext::ftParts_80074E58(fp);
    ext::ftParts_SetupParts(g);
    super::ftanim::ftAnim_80070308(g);
    ext::ftCo_800C884C(g);
    Fighter_80068E64(g);
    ext::ftParts_800749CC(g);
    super::ftanim::ftAnim_8007077C(g);
    ext::ftCo_8009CF84(fp);
    super::ftanim::ftAnim_8006FE48(g);
    Fighter_UnkUpdateVecFromBones_8006876C(fp);
    ext::ftCo_8009F578(fp);
    ext::Mex_OnLoadDispatch(fp.kind as i32, g);
    Fighter_Create_Inline2(g);
    ext::ftColl_8007B320(g);
    fp.x890_cameraBox = ext::Camera_80029020() as _;
    ext::lbShadow_8000ED54(&mut fp.x20A4 as *mut _ as *mut _, jobj_of(g));
    let procs: [(fn(G), u8); 15] = [
        (proc_8006A1BC, 0), (proc_8006A360, 1), (proc_8006ABA0, 2), (proc_spaghetti, 3), (proc_update, 4),
        (proc_map, 6), (proc_8006C5F4, 7), (proc_accessory, 8), (proc_8006C80C, 9), (proc_grab, 0xC),
        (proc_8006CB94, 0xD), (proc_hit, 0xE), (proc_8006D9AC, 0x10), (proc_camera, 0x12), (proc_8006DA4C, 0x16),
    ];
    for (f, p) in procs {
        ext::HSD_GObj_SetupProc(g, f, p);
    }
    Fighter_UnkProcessDeath_80068354(g);
    if fp.kind == Ft_Kind_MasterH {
        ext::ftMh_MS_341_8014FE10(g);
    } else if fp.kind == Ft_Kind_CrezyH {
        ext::ftCh_Init_80155FCC(g);
    } else if input.anon_6.has_transformation() != 0 {
        ext::ftCo_800BFD04(g);
    } else if ext::Player_GetFlagsBit3(fp.player_id as i32) != 0 {
        ext::ftCo_800C61B0(g);
    } else if fp.no_normal_motion() == 0 {
        ext::ftCommon_8007D92C(g);
    } else {
        panic!("fighter.c:1065: ellegal flag fp->no_normal_motion");
    }
    ext::ftLib_800867E8(g);
    ext::ScriptGame_OnePSpawn(fp);
    g
}

static mut PC_TRACE_ENABLED: i32 = -1;

/// `ft_PcTraceMotion` (static, TARGET_PC): MELEE_LOG_MOTION line per state change.
pub unsafe fn ft_PcTraceMotion(g: G, msid: i32) {
    if PC_TRACE_ENABLED < 0 {
        PC_TRACE_ENABLED = ext::PcTraceMotionEnabled();
    }
    if PC_TRACE_ENABLED == 0 || fpm(g).motion_id as i32 == msid {
        return;
    }
    ext::OSReport("motion: state change\n");
}

#[inline]
fn mf(flags: u32, f: u32) -> bool { flags & f != 0 }

/// The blend argument `ftAnim_8006EBE8` gets: -1 -> 0, nonzero -> itself, 0 -> the row's.
#[inline]
fn blend_arg(anim_blend: f32, row: u8) -> f32 {
    if anim_blend == -1.0 { 0.0 } else if anim_blend != 0.0 { anim_blend } else { row as f32 }
}

/// `Fighter_ChangeMotionState`: enter motion state `msid`.
pub unsafe fn Fighter_ChangeMotionState(g: G, msid: i32, mut flags: u32, anim_start: f32, mut anim_speed: f32, anim_blend: f32, mut arg3: G) {
    use ext as e;
    let jobj = jobj_of(g);
    let fp = fpm(g);
    let mut lab_pose_row = -1;
    let mut thrown_tbl: F = core::ptr::null_mut();
    if !arg3.is_null() {
        let thrower = (*arg3).user_data as F;
        if !thrower.is_null() && (*thrower).kind != fp.kind {
            if e::Geno_DefineIsNone(fp.kind as i32) != 0 {
                arg3 = core::ptr::null_mut();
            } else if e::Geno_DefineIsNone((*thrower).kind as i32) != 0 {
                let mut r#as = e::Geno_DefineThrownAs((*thrower).kind as i32);
                if msid >= ftCo_MS_ThrownF as i32 && msid <= ftCo_MS_ThrownLw as i32 {
                    r#as = e::Geno_DefineThrownAsRow((*thrower).kind as i32, msid - ftCo_MS_ThrownF as i32);
                }
                thrown_tbl = if r#as >= 0 { e::ftData_ThrownStandIn(r#as) } else { core::ptr::null_mut() };
                if thrown_tbl.is_null() {
                    ext::OSReport("throw: stand-in has no animation bank; the victim holds its pose\n");
                    arg3 = core::ptr::null_mut();
                    flags |= Ft_MF_SkipAnim as u32;
                }
            }
        }
    }
    ft_PcTraceMotion(g, msid);
    e::Script_GameEvent(1, fp.player_id as i32, fp.motion_id as i32, msid, fp.is_sub_fighter() as i32);
    fp.motion_id = msid as _;
    e::Geno_OnActionChange(g);
    fp.facing_dir1 = fp.facing_dir;
    set_translate(jobj, &fp.cur_pos);
    ext::efAsync_QueueFlush(g, &mut fp.x60C);
    if !mf(flags, Ft_MF_SkipHit) {
        if fp.x2219_b3() != 0 {
            ext::ftColl_8007AFF8(g);
        }
        if fp.x2219_b4() != 0 {
            e::ft_8007C114(g);
        }
    }
    if !mf(flags, Ft_MF_SkipThrowException) {
        fp.x1064_thrownHitbox.anon_134.owner = core::ptr::null_mut();
    }
    if !mf(flags, Ft_MF_KeepColAnimHitStatus) {
        if fp.x1988 != 0 {
            e::ftColl_8007B62C(g, 0);
        }
        if fp.x221A_b5() != 0 {
            ext::ftColl_8007B0C0(g, HurtCapsule_Enabled as i32);
        }
    }
    if fp.x221A_b6() != 0 {
        e::ftColl_8007B4E0(g);
    }
    if !mf(flags, Ft_MF_SkipModel) && fp.x221D_b2() != 0 {
        crate::ft0df::ftparts_80074a8c(g);
    }
    if !mf(flags, Ft_MF_SkipMatAnim) && fp.x221E_b7() != 0 {
        super::ftanim::ftAnim_80070654(g);
    }
    if !mf(flags, Ft_MF_SkipParasol) {
        fp.set_x2221_b4(0);
        if e::ftGetParasolStatus(g) != -1 && e::ftGetParasolStatus(g) != 6 {
            e::ftCommon_8007E83C(g, 6, 0.0);
        }
    }
    super::ftanim::ftAnim_80070F28(g);
    super::ftanim::ftAnim_80070E74(g);
    e::ftCommon_8007ECD4(fp, 7);
    e::ftCommon_8007ECD4(fp, 8);
    e::ftCommon_8007ECD4(fp, 0x24);
    if !mf(flags, Ft_MF_SkipRumble) {
        e::ftCommon_8007ECD4(fp, 1);
        e::ftCommon_8007ECD4(fp, 0x19);
    }
    if !mf(flags, Ft_MF_KeepColAnimPartHitStatus) && fp.x2221_b1() != 0 {
        e::ftColl_8007B6EC(g);
        e::ftColl_8007B760(g, cd().x134);
        fp.set_x2221_b1(0);
    }
    e::ftCommon_8007E2F4(fp, 0);
    if fp.dmg.x18F4 != 0 {
        fp.dmg.x18F4 = 0;
        fp.set_x2220_b4(0);
    }
    if !mf(flags, Ft_MF_Unk19) {
        fp.set_x2222_b2(0);
    }
    if !mf(flags, Ft_MF_SkipMetalB) {
        fp.set_x2223_b4(0);
    }
    if !mf(flags, Ft_MF_Unk27) {
        fp.set_x2227_b2(0);
    }
    if !mf(flags, Ft_MF_SkipHitStun) && fp.x221C_b6() != 0 {
        fp.set_x221C_b6(0);
        fp.x2098 = cd().x4CC as _;
    }
    bits!(fp, 0; set_x221F_b3, set_x2219_b1, set_x2219_b2);
    fp.dmg.x182c_behavior = 1.0;
    fp.dmg.kb_applied = 0.0;
    fp.dmg.x18A8 = 0.0;
    fp.dmg.armor1 = 0.0;
    fp.dmg.x18a0 = 0.0;
    bits!(fp, 0; set_x221A_b7, anon_221b.set_x221B_b0, set_x221C_b3);
    fp.shield_unk0 = 0.0;
    fp.shield_unk1 = 0.0;
    bits!(fp, 0; set_x221D_b5, set_reflecting, set_x2218_b6, set_x221C_b4);
    fp.x1A6A = 0;
    bits!(fp, 0; set_x221D_b7, set_invisible, set_x221E_b1, set_x221E_b2, set_x221F_b1, set_x221E_b5, set_x221E_b6,
        set_x2220_b3, set_x2220_b7);
    fp.x209C = 0;
    bits!(fp, 0; set_x2223_b0, set_x2222_b3, set_x2224_b5, set_x2225_b1, set_x2225_b4);
    e::mpClearFloorSkip(&mut fp.coll_data);
    ext::ftCo_800DEEA8(g);
    fp.smash_attrs.x2138_smashSinceHitbox = -1.0;
    fp.set_x2224_b4(0);
    if !mf(flags, Ft_MF_SkipItemVis) {
        fp.set_x221E_b3(1);
    } else if fp.x221E_b3() == 0 {
        e::ftCommon_8007F578(g);
    }
    if !mf(flags, Ft_MF_SkipModelPartVis) {
        fp.set_x221E_b4(1);
    }
    if !mf(flags, Ft_MF_SkipModelFlags) {
        fp.set_x2225_b2(1);
    }
    if !mf(flags, Ft_MF_KeepFastFall) {
        fp.set_fall_fast(0);
    }
    if !mf(flags, Ft_MF_SkipColAnim) {
        e::ftCo_800C0134(fp);
    }
    if !mf(flags, Ft_MF_KeepGfx) && fp.x2219_b0() != 0 {
        e::ftCommon_8007DB24(g);
    }
    if !mf(flags, Ft_MF_KeepAccessory) && !fp.x20A0_accessory.is_null() {
        e::HSD_JObjRemoveAll(fp.x20A0_accessory);
        fp.x20A0_accessory = core::ptr::null_mut();
    }
    if fp.ground_or_air == GA_Ground {
        e::Mex_OnLandingDispatch(fp.kind as i32, g);
        if fp.kind == Ft_Kind_Peach {
            fp.u.pe.has_float = 1;
        }
        fp.set_x2221_b5(0);
        fp.set_x2221_b7(1);
        fp.set_x2221_b6(1);
        fp.set_x2224_b1(0);
        fp.set_x2227_b1(0);
        fp.x213C = -1;
        if fp.x2227_b4() != 0 {
            e::pl_8003FE1C(fp.player_id as i32, fp.is_sub_fighter() as i32);
            fp.set_x2227_b4(0);
        }
        fp.set_x2227_b5(0);
        e::pl_80040330(fp.player_id as i32, fp.is_sub_fighter() as i32, fp.x2140);
        fp.x2140 = 0.0;
        fp.set_used_tether(0);
        fp.x2180 = 6;
    }
    if !(0xE..=0x11).contains(&msid) {
        fp.hitlag_mul = 0.0;
    }
    if !mf(flags, Ft_MF_KeepSfx) {
        e::ft_80088884(fp);
        e::ft_800888E0(fp);
        e::ft_800887CC(fp);
    }
    if !mf(flags, Ft_MF_KeepSwordTrail) {
        fp.x2100 = -1;
    }
    if !mf(flags, Ft_MF_SkipNametagVis) {
        fp.x209A = 0;
    }
    fp.set_x2222_b7(0);
    fp.x100 = if mf(flags, Ft_MF_UnkUpdatePhys) { 0.0 } else { -1.0 };
    if !mf(flags, Ft_MF_Unk24) {
        fp.set_x221C_u16_y(0);
    }
    fp.lstick_angle = 0.0;
    super::ftparts::ftPartSetRotX(fp, 0, 0.0);
    super::ftparts::ftPartSetRotY(fp, 0, (M_PI_2 * fp.facing_dir as f64) as f32);
    super::ftparts::ftPartSetRotZ(fp, 0, 0.0);
    e::ScriptGame_NoteMotion(fp, msid);
    let ms: *mut MotionState = if msid >= 0x400 {
        e::Geno_MotionRow(fp, msid)
    } else if msid >= fp.x18 as i32 {
        fp.x20_actionStateList.offset((msid - fp.x18 as i32) as isize)
    } else {
        fp.x1C_actionStateList.offset(msid as isize)
    };
    let ms = &*ms;
    if fp.ground_or_air == GA_Ground && flags & 0x40 == 0 && ms.anon_8.anon_0.anon_1.x9_b1() != 0 && fp.dmg.x18C8 == -1 {
        fp.dmg.x18C8 = if cd().x814 > 0 { cd().x814 } else { 1 };
    }
    let x2070 = fp.x2070;
    e::ft_800890D0(fp, ms.anon_8.anon_0.move_id() as i32);
    e::ft_800895E0(fp, ms.x4_flags as i32);
    fp.set_x2225_b3(ms.anon_8.anon_0.anon_1.x9_b0());
    if fp.x2226_b4() != 0 {
        if fp.x2070.anon_0.x2071_b5() != 0 {
            e::ftCo_800C8B2C(fp, 0x7E, 0);
        }
        if fp.x2070.anon_0.x2071_b6() != 0 {
            e::ftCo_800C8B2C(fp, 0x7F, 0);
        }
    }
    if let Some(cb) = fp.x21EC {
        cb(g);
        fp.x21EC = None;
    }
    if !mf(flags, Ft_MF_SkipAttackCount) {
        e::pl_80037C60(g, x2070);
    }
    fp.anim_id = ms.anim_id;
    let row = e::Geno_MotionAnimRow(fp, msid);
    if row >= 0 {
        fp.anim_id = row;
    }
    let lab_row = e::ScriptGame_LabPreviewRow(fp, msid);
    if lab_row >= 0 {
        lab_pose_row = fp.anim_id;
        fp.anim_id = lab_row;
    }
    fp.frame_speed_mul = anim_speed;
    fp.x8A0_unk = anim_speed;
    fp.cur_anim_frame = anim_start - fp.frame_speed_mul;
    fp.x898_unk = 0.0;
    let x594 = &fp.anon_594.anon_0;
    let animflags = x594.x594_b0() != 0 || x594.x594_b5() != 0;
    if fp.anim_id != -1 {
        let bone_index = fp.anon_594.anon_0.x596_bits.x7() as i32;
        if mf(flags, Ft_MF_FreezeState) {
            fp.set_x2223_b0(1);
            fp.x104 = 0x14;
            fp.frame_speed_mul = 0.0;
            anim_speed = 0.0;
        }
        let tbl: F = if !arg3.is_null() { if !thrown_tbl.is_null() { thrown_tbl } else { (*arg3).user_data as F } } else { fp };
        let mut wait = (*tbl).x24.offset(fp.anim_id as isize);
        let mut bytes = (*tbl).x28.offset(fp.anim_id as isize);
        if lab_pose_row >= 0 && arg3.is_null() {
            wait = fp.x24.offset(lab_pose_row as isize);
            bytes = fp.x28.offset(lab_pose_row as isize);
        }
        fp.anon_594.x594_s32 = (*wait).x10_animCurrFlags as i32;
        e::ftCo_8009E7B4(fp, bytes);
        if !mf(flags, Ft_MF_SkipAnim) {
            if !arg3.is_null() {
                e::ftData_80085CD8(fp, if !thrown_tbl.is_null() { thrown_tbl } else { (*arg3).user_data as F }, fp.anim_id);
                e::ftColl_8007B8CC(fp, arg3);
            } else {
                e::ftData_80085CD8(fp, fp, fp.anim_id);
            }
            fp.x3E4_fighterCmdScript.anon_8.u = (*wait).xC as _;
            fp.x3E4_fighterCmdScript.loop_count = 0;
            let row0 = (*bytes)[0];
            if anim_start != 0.0 {
                if !fp.x590.is_null() {
                    super::ftanim::ftAnim_8006EBE8(g, anim_start - anim_speed, anim_speed, blend_arg(anim_blend, row0));
                }
                super::ftanim::ftAnim_8006E9B4(g);
                if fp.anon_594.anon_0.x594_b0() != 0 {
                    fp.x6B0 = v0();
                    fp.x6A4_transNOffset = v0();
                    fp.x698 = fp.x68C_transNPos;
                }
                if fp.anon_594.anon_0.x594_b5() != 0 {
                    fp.x6E4 = v0();
                    fp.x6D8 = v0();
                    fp.x6CC = fp.x6C0;
                }
                fp.x3E4_fighterCmdScript.timer = -anim_start;
            } else {
                if !fp.x590.is_null() {
                    super::ftanim::ftAnim_8006EBE8(g, anim_start, anim_speed, blend_arg(anim_blend, row0));
                }
                fp.x3E4_fighterCmdScript.timer = 0.0;
            }
            super::ftanim::ftAnim_8006E9B4(g);
            if bone_index != 0 && (*bytes)[0] != 0 {
                let b = &*fp.parts.offset(bone_index as isize);
                let mut t = v0();
                HSD_JObjGetTranslation(&*b.x4_jobj2, &mut t);
                set_translate(b.joint, &t);
                let mut q: Quaternion = core::mem::zeroed();
                HSD_JObjGetRotation(&*b.x4_jobj2, &mut q);
                e::ftParts_JObjSetRotation(b.joint, &q);
            }
            let ground_vel = !mf(flags, Ft_MF_SkipAnimVel) && fp.ground_or_air == GA_Ground;
            if fp.anon_594.anon_0.x594_b0() != 0 {
                if anim_start == 0.0 {
                    fp.x6B0 = v0();
                    fp.x6A4_transNOffset = v0();
                    fp.x698 = fp.x68C_transNPos;
                } else if ground_vel {
                    let v = fp.x6A4_transNOffset.z * fp.facing_dir;
                    fp.self_vel.x = v;
                    fp.gr_vel = v;
                }
            }
            if fp.anon_594.anon_0.x594_b5() != 0 {
                if anim_start == 0.0 {
                    fp.x6E4 = v0();
                    fp.x6D8 = v0();
                    fp.x6CC = fp.x6C0;
                } else if ground_vel {
                    let v = fp.x6D8.z * fp.facing_dir;
                    fp.self_vel.x = v;
                    fp.gr_vel = v;
                }
            }
            if mf(flags, Ft_MF_UpdateCmd) {
                super::ftaction::ftAction_8007349C(g);
            } else if anim_start != 0.0 {
                super::ftaction::ftAction_80073354(g);
            } else {
                e::ftCo_800C0408(g);
                super::ftaction::ftAction_80073240(g);
            }
        } else {
            fp.anim_id = -1;
        }
    }
    if fp.anim_id == -1 {
        fp.anon_594.x594_s32 = 0;
        super::ftanim::ftAnim_80070758(jobj);
        super::ftanim::ftAnim_80070758(fp.x8AC_animSkeleton);
        fp.x3E4_fighterCmdScript.anon_8.u = core::ptr::null_mut();
        fp.x8A4_animBlendFrames = 0.0;
        fp.x8A8_anim_frame = 0.0;
    }
    if animflags && fp.anon_594.anon_0.x594_b0() == 0 {
        e::ftCommon_ClampGroundVel(fp, h::script_value(fp, 2, fp.co_attrs.dash_max_velocity));
    }
    fp.anim_cb = ms.anim_cb;
    fp.input_cb = ms.input_cb;
    fp.phys_cb = ms.phys_cb;
    fp.coll_cb = ms.coll_cb;
    fp.cam_cb = ms.cam_cb;
    fp.accessory1_cb = None;
    fp.accessory4_cb = None;
    fp.deal_dmg_cb = None;
    fp.shield_hit_cb = None;
    fp.reflect_hit_cb = None;
    fp.hitlag_cb = None;
    fp.x21CC = None;
    fp.post_hitlag_cb = None;
    fp.pre_hitlag_cb = None;
    fp.take_dmg_cb = None;
    fp.take_dmg_2_cb = None;
    fp.hurtbox_detect_cb = None;
    fp.x21F8 = None;
    fp.death2_cb = None;
}

/// `Fighter_8006A1BC` (proc 0): hitlag / SDI timers and mushroom queues.
pub unsafe fn Fighter_8006A1BC(g: G) {
    use ext as e;
    let fp = fpm(g);
    if fp.x221F_b3() != 0 {
        return;
    }
    if fp.dmg.x1954 > 0.0 {
        fp.dmg.x1954 -= 1.0;
        if fp.dmg.x1954 <= 0.0 {
            fp.dmg.x1954 = 0.0;
            if fp.allow_sdi() == 0 && fp.x2219_b7() == 0 {
                Fighter_8006D10C(g);
            }
        }
    }
    if fp.x221A_b1() != 0 {
        Fighter_8006CFE0(g);
        fp.set_x221A_b1(0);
    }
    if fp.dmg.x195c_hitlag_frames > 0.0 {
        fp.dmg.x195c_hitlag_frames -= 1.0;
        if fp.dmg.x195c_hitlag_frames <= 0.0 {
            fp.dmg.x195c_hitlag_frames = 0.0;
            if fp.x221A_b3() != 0 {
                e::ftCo_80090718(fp);
                fp.set_x221A_b3(0);
            }
            if fp.dmg.x1954 == 0.0 && fp.x2219_b7() == 0 {
                Fighter_8006D10C(g);
            }
            fp.set_allow_sdi(0);
        }
    }
    e::ftCo_800C37A0(g);
    while fp.x200C != 0 {
        e::Fighter_SuperMushroomApply(g);
        fp.x200C -= 1;
    }
    while fp.x2010 != 0 {
        e::Fighter_PoisonMushroomApply(g);
        fp.x2010 -= 1;
    }
    e::ft_800819A8(g);
    let b5 = fp.x2219_b5();
    fp.set_x2219_b6(b5);
}

/// `ftSlippi_IsOffscreen` (static, TARGET_PC).
pub unsafe fn ftSlippi_IsOffscreen(fp: &Fighter) -> bool {
    use ext as e;
    if e::Slippi_Codes() != 2 && e::Camera_ScriptState() == 0 {
        return e::ifMagnify_802FC998(fp.player_id as i32);
    }
    if e::gm_GetCurrentGameMode() == GM_HOME_RUN_CONTEST as i32 || fp.x221F_b1() != 0 || fp.motion_id == 4 || fp.motion_id == 6 {
        return false;
    }
    fp.cur_pos.x < e::Stage_GetCamBoundsLeftOffset()
        || fp.cur_pos.x > e::Stage_GetCamBoundsRightOffset()
        || fp.cur_pos.y > e::Stage_GetCamBoundsTopOffset()
        || fp.cur_pos.y < e::Stage_GetCamBoundsBottomOffset()
}

/// `Fighter_8006A360` (proc 1): per-frame timers, animation advance, anim callback.
pub unsafe fn Fighter_8006A360(g: G) {
    use ext as e;
    let fp = fpm(g);
    if fp.x221F_b3() != 0 {
        return;
    }
    fp.pos_delta.x = fp.cur_pos.x - fp.prev_pos.x;
    fp.pos_delta.y = fp.cur_pos.y - fp.prev_pos.y;
    fp.pos_delta.z = fp.cur_pos.z - fp.prev_pos.z;
    fp.prev_pos = fp.cur_pos;
    if fp.dmg.x18C8 != -1 && fp.dmg.x18C8 > 0 {
        fp.dmg.x18C8 -= 1;
        if fp.dmg.x18C8 == 0 {
            fp.dmg.x18c4_source_ply = 6;
            fp.dmg.x18C8 = -1;
            fp.dmg.x18D0 = -0xA;
        }
    }
    if fp.x1990 != 0 {
        fp.x1990 -= 1;
        if fp.x1990 == 0 && fp.x2221_b0() == 0 {
            fp.x198C = if fp.x1994 != 0 { 1 } else { 0 };
            if e::ftCo_800C0694(fp) == 9 {
                e::ftCo_800C0200(fp, 9);
            }
        }
    }
    if fp.x1994 != 0 {
        fp.x1994 -= 1;
        if fp.x1994 == 0 {
            fp.x198C = if fp.x2221_b0() != 0 || fp.x1990 != 0 { 2 } else { 0 };
            if e::ftCo_800C0694(fp) == 9 {
                e::ftCo_800C0200(fp, 9);
            }
        }
    }
    if fp.x221D_b6() != 0 && fp.x2004 != 0 {
        fp.x2004 -= 1;
        if fp.x2004 == 0 {
            fp.set_x221D_b6(0);
            if e::ftCo_800C0694(fp) == 0x6B {
                e::ftCo_800C0200(fp, 0x6B);
            }
        } else if fp.x2004 as i32 == e::it_8026B588() {
            e::ft_800880D8(fp);
        }
    }
    if fp.x2220_b5() != 0 || fp.x2220_b6() != 0 {
        if fp.x2008 != 0 {
            fp.x2008 -= 1;
        }
        if fp.x2008 == 0 {
            if fp.x2220_b5() != 0 {
                e::Fighter_SuperMushroomEnd(g);
            } else if fp.x2220_b6() != 0 {
                e::Fighter_PoisonMushroomEnd(g);
            }
        }
    }
    if !fp.x197C.is_null() && fp.x2014 != 0 {
        fp.x2014 -= 1;
        if fp.x2014 == 0 {
            e::ftCommon_8007F8E8(g);
            e::Item_8026A8EC(fp.x197C);
            e::ftCommon_8007F9B4(g);
        }
    }
    if fp.is_metal() != 0 && fp.metal_timer != 0 {
        fp.metal_timer -= 1;
        if fp.metal_timer == 0 || fp.metal_health <= 0 {
            e::ftCo_800C8540(g);
        }
    }
    if fp.x2227_b3() != 0 && fp.x2034 != 0 {
        fp.x2034 -= 1;
        if fp.x2034 == 0 || fp.x2038 <= 0 {
            e::ftCo_800C9034(g);
            return;
        }
    }
    if !fp.x1980.is_null() {
        fp.x2018 -= 1;
        fp.x201C -= 1;
        if fp.x201C == 0 {
            Fighter_TakeDamage_8006CC7C(fp, cd().x6F4_unkDamage as f32);
            fp.x201C = cd().x6F8 as _;
        }
        if fp.x2018 <= 0 {
            let v = Fighter_803B7488;
            e::ftCommon_8007F8E8(g);
            e::Item_8026ABD8(fp.x1980, &v, 0.0);
            e::ftCommon_8007FF74(g);
        }
    }
    if fp.x2226_b4() != 0 && fp.x2030 != 0 {
        fp.x2030 -= 1;
        if fp.x2030 == 0 {
            e::ftCo_800C8A64(g);
            return;
        }
        if fp.x2226_b3() == 0 && fp.x2030 as i32 == cd().x7D0 as i32 && e::ftCo_800C8B2C(fp, 0x7D, 0) {
            fp.set_x2226_b7(1);
        }
    }
    if fp.x2221_b4() != 0 && fp.x2104 != 0 {
        fp.x2104 -= 1;
        if fp.x2104 == 0 {
            fp.set_x2221_b4(0);
            if !fp.item_gobj.is_null() && e::itGetKind(fp.item_gobj) == It_Kind_Peach_Parasol as i32 {
                fp.set_x2221_b5(1);
                e::ftCo_800968C8(g);
            } else {
                e::ftCo_80095744(g, 0);
            }
        }
    }
    if fp.is_sub_fighter() == 0 && e::Camera_80031144() == 1.0 && fp.dmg.x1830_percent < cd().x7B0 as f32 {
        if ftSlippi_IsOffscreen(fp) && ext::Player_GetMoreFlagsBit3(fp.player_id as i32) != 0 {
            fp.dmg.x1910 += 1;
        } else {
            fp.dmg.x1910 = 0;
        }
        if fp.dmg.x1910 >= cd().x7AC as _ {
            Fighter_TakeDamage_8006CC7C(fp, cd().x7B4_unkDamage as f32);
            fp.dmg.x1910 = 0;
        }
    }
    if fp.dmg.x18F0 != 0 {
        fp.dmg.x18F0 -= 1;
        if fp.dmg.x1830_percent > 0.0 {
            fp.dmg.x1830_percent -= 1.0;
            e::ft_80088640(fp, 0x7D, 0x7F, 0x40);
            ext::Player_SetHPByIndex(fp.player_id as i32, fp.is_sub_fighter() as i32, fp.dmg.x1830_percent);
            e::pl_80040B8C(fp.player_id as i32, fp.is_sub_fighter() as i32, 1);
        }
        if fp.dmg.x1830_percent <= 0.0 {
            fp.dmg.x1830_percent = 0.0;
            fp.dmg.x18F0 = 0;
        }
        if fp.dmg.x18F0 == 0 {
            if e::ftCo_800C0694(fp) == 8 {
                e::ftCo_800C0200(fp, 8);
            }
            e::ftCommon_8007ECD4(fp, 2);
        }
    }
    if !fp.item_gobj.is_null() && e::itGetKind(fp.item_gobj) == It_Kind_Hammer as i32 {
        e::ftCo_800C511C(g);
    }
    if fp.dmg.x18fa_model_shift_frames != 0 {
        fp.dmg.x18fa_model_shift_frames -= 1;
        fp.dmg.x18FC = fp.dmg.x18FC.wrapping_add(1);
        if fp.dmg.x18FC >= fp.dmg.x18FD {
            fp.dmg.x18FC = 0;
        }
    }
    e::Mex_OnFrameDispatch(fp.kind as i32, g);
    e::Geno_OnFrame(g);
    if let Some(cb) = fp.x21CC {
        cb(g);
    }
    if fp.x2219_b5() == 0 {
        if fp.x209A > 1 && fp.x221D_b4() == 0 {
            fp.x209A -= 1;
        }
        if fp.x2223_b0() != 0 {
            if fp.x104 == 0x14 {
                super::ftanim::ftAnim_8006F0FC(g, 0.0);
            } else {
                fp.frame_speed_mul += fp.x8A0_unk;
            }
            fp.x104 -= 1;
            if fp.x104 == 0 {
                super::ftanim::ftAnim_8006F0FC(g, fp.frame_speed_mul);
                fp.x104 = 0x14;
            }
        }
        if fp.smash_attrs.x2138_smashSinceHitbox != -1.0 {
            fp.smash_attrs.x2138_smashSinceHitbox += 1.0;
        }
        if fp.dmg.x18ac_time_since_hit != -1 {
            fp.dmg.x18ac_time_since_hit += 1;
        }
        super::ftanim::ftAnim_8006EBA4(g);
        e::ftCo_800D71D8(g);
        e::ftColl_800764DC(g);
        if fp.x221C_b6() == 0 {
            e::pl_800411C4(fp.player_id as i32, fp.is_sub_fighter() as i32);
        }
        e::ftCo_800DEF38(g);
        if e::Geno_PreAnim(g) == 0 {
            if let Some(cb) = fp.anim_cb {
                cb(g);
            }
        }
    }
    e::ftCommon_8007E0E4(g);
    e::ftCo_800C0408(g);
}

/// `Fighter_8006ABA0` (proc 2): CPU think (TARGET_PC: LAB virtual pad first).
pub unsafe fn Fighter_8006ABA0(g: G) {
    use ext as e;
    let fp = fpm(g);
    e::ScriptGame_FighterBenchTick(fp);
    if e::ScriptGame_VirtualPadApply(fp) != 0 {
        return;
    }
    if fp.x221F_b3() == 0 && e::ftCo_IsCpuControlled(fp) {
        e::ftCo_800B3900(g);
    }
    e::ScriptGame_VirtualPadObserve(fp);
}

/// `Fighter_UnkIncrementCounters_8006ABEC`: input-age counters (jump, specials...).
pub unsafe fn Fighter_UnkIncrementCounters_8006ABEC(g: G) {
    use ext as e;
    let fp = fpm(g);
    if e::ftCo_Jump_GetInput(g) {
        fp.x68A = fp.x685;
        fp.x685 = 0;
    } else if fp.x685 < 255 {
        fp.x685 += 1;
    }
    if e::ftCo_800D6928(fp) {
        fp.x68B = fp.x686;
        fp.x686 = 0;
    } else if fp.x686 < 255 {
        fp.x686 += 1;
    }
    if e::ftCo_800D688C(fp) {
        fp.x687 = 0;
    } else if fp.x687 < 255 {
        fp.x687 += 1;
    }
    if e::ftCo_SpecialS_HasInput(fp) {
        fp.x688 = 0;
    } else if fp.x688 < 255 {
        fp.x688 += 1;
    }
    if e::ftCo_800D67C4(fp) {
        fp.x689 = 0;
    } else if fp.x689 < 255 {
        fp.x689 += 1;
    }
}

/// `Fighter_Spaghetti_8006AD10_Inner1` (static): pressed / released edges.
pub fn Fighter_Spaghetti_8006AD10_Inner1(fp: &mut Fighter) {
    let h0 = fp.input.held_buttons[0];
    let h1 = fp.input.held_buttons[1];
    let pressed = h0 & (h1 ^ h0);
    let released = h1 & (h1 ^ h0);
    if fp.x2219_b5() != 0 {
        fp.input.pressed_buttons |= pressed;
        fp.input.released_buttons |= released;
    } else {
        fp.input.pressed_buttons = pressed;
        fp.input.released_buttons = released;
    }
}

/// `ftUcf_Cardinal` (static, TARGET_PC): UCF 0.84 cardinal snap of one stick.
pub fn ftUcf_Cardinal(x: i32, y: i32, out: &mut Vec2) {
    if x >= 80 || x <= -80 {
        if y > 6 || y < -6 {
            return;
        }
        out.x = if x < 0 { -1.0 } else { 1.0 };
        out.y = 0.0;
    } else if y >= 80 || y <= -80 {
        if x > 6 || x < -6 {
            return;
        }
        out.x = 0.0;
        out.y = if y < 0 { -1.0 } else { 1.0 };
    }
}

#[inline]
fn fabs(v: f32) -> f32 { if v < 0.0 { -v } else { v } }
#[inline]
fn bump(v: &mut u8) {
    *v = v.wrapping_add(1);
    if *v > 254 {
        *v = 254;
    }
}

/// One stick axis's smash timers (`Fighter_ClampThreeValues` & co).
fn stick_axis(cur: f32, prev: f32, dz: f32, dur: &mut u8, timer: &mut u8, sticky: &mut u8, act: &mut u8) -> Option<bool> {
    bump(dur);
    if cur >= dz {
        if prev >= dz {
            bump(timer);
            bump(sticky);
            bump(act);
            None
        } else {
            *dur = 0;
            *sticky = 0;
            *timer = 0;
            Some(true)
        }
    } else if cur <= -dz {
        if prev <= -dz {
            bump(timer);
            bump(sticky);
            bump(act);
            None
        } else {
            *dur = 0;
            *sticky = 0;
            *timer = 0;
            Some(false)
        }
    } else {
        *act = 254;
        *sticky = 254;
        *timer = 254;
        None
    }
}

static mut CSTICK_1P_DEFAULT_READ: bool = false;
static mut CSTICK_COMBO_WAS_HELD: bool = false;

/// `Fighter_Spaghetti_8006AD10` (proc 3): read this frame's input into the fighter.
pub unsafe fn Fighter_Spaghetti_8006AD10(g: G) {
    use ext as e;
    let fp = fpm(g);
    if fp.x221F_b3() != 0 {
        return;
    }
    let c = cd();
    if fp.x2224_b2() == 0 {
        let src = if fp.x221D_b3() == 0 { 2 } else { 0 };
        fp.input.lstick[1] = fp.input.lstick[src];
        fp.input.cstick[1] = fp.input.cstick[src];
        fp.input.triggers[1] = fp.input.triggers[src];
        fp.input.held_buttons[1] = fp.input.held_buttons[src];
        if src == 2 {
            fp.set_x221D_b3(1);
        }
        let cpu = e::ftCo_IsCpuControlled(fp);
        if cpu {
            fp.input.lstick[0].x = e::ftCo_GetCpuLStickX(fp);
            fp.input.lstick[0].y = e::ftCo_GetCpuLStickY(fp);
            if (e::DbLevel_below_DebugRom() || e::Mex_Enabled("enable_c_stick_always_cpu_debug"))
                && (e::gm_IsCurrently1PMode_inline() == 0 || e::Mex_Enabled("enable_c_stick_always_interrupt2") || e::ScriptGame_CpuStanding(fp) != 0)
            {
                fp.input.cstick[0].x = e::ftCo_GetCpuCStickX(fp);
                fp.input.cstick[0].y = e::ftCo_GetCpuCStickY(fp);
            } else {
                fp.input.cstick[0].x = 0.0;
                fp.input.cstick[0].y = 0.0;
            }
            let l = e::ftCo_GetCpuLTrigger(fp);
            let r = e::ftCo_GetCpuRTrigger(fp);
            fp.input.triggers[0] = if l > r { l } else { r };
        } else {
            let p = e::pad_nml(fp.x618_player_id);
            fp.input.lstick[0].x = p[0];
            fp.input.lstick[0].y = p[1];
            let t = &mut *e::gm_CStickSmashTargetTest();
            if !CSTICK_1P_DEFAULT_READ {
                *t = e::Settings_Int("cstick_1p_attacks", 1) != 0;
                CSTICK_1P_DEFAULT_READ = true;
                ext::OSReport("C-STICK: 1P default\n");
            }
            if (e::DbLevel_below_DebugRom() || e::Mex_Enabled("enable_c_stick_always_spoof_debug_level"))
                && (e::gm_IsCurrently1PMode_inline() == 0 || *t || e::Mex_Enabled("enable_c_stick_always_interrupt1"))
            {
                fp.input.cstick[0].x = p[2];
                fp.input.cstick[0].y = p[3];
            } else {
                fp.input.cstick[0].x = 0.0;
                fp.input.cstick[0].y = 0.0;
            }
            let (l, r) = (p[4], p[5]);
            fp.input.triggers[0] = if l > r { l } else { r };
        }
        if fabs(fp.input.lstick[0].x) <= c.horizontal_stick_deadzone {
            fp.input.lstick[0].x = 0.0;
        }
        if fabs(fp.input.lstick[0].y) <= c.vertical_stick_deadzone {
            fp.input.lstick[0].y = 0.0;
        }
        if fabs(fp.input.cstick[0].x) <= c.horizontal_stick_deadzone {
            fp.input.cstick[0].x = 0.0;
        }
        if fabs(fp.input.cstick[0].y) <= c.vertical_stick_deadzone {
            fp.input.cstick[0].y = 0.0;
        }
        if fp.input.triggers[0] <= c.analog_shoulder_deadzone {
            fp.input.triggers[0] = 0.0;
        }
        fp.input.held_buttons[0] = if e::ftCo_IsCpuControlled(fp) { e::ftCo_GetCpuButtons(fp) } else { e::pad_button(fp.x618_player_id) };
        if e::ftCo_IsCpuControlled(fp) && e::ScriptGame_CpuStanding(fp) == 0 {
            technical_ai_overlay(fp);
            cpu_assist_overlay(fp);
        }
        if h::gm_8016b0fc() {
            fp.input.triggers[0] = 0.0;
            if e::ftCo_IsCpuControlled(fp) {
                fp.input.held_buttons[0] &= (HSD_PAD_A | (HSD_PAD_X | HSD_PAD_Y)) as u32;
            } else {
                fp.input.held_buttons[0] &= HSD_PAD_A as u32;
            }
        } else {
            if fp.input.held_buttons[0] & (HSD_PAD_L | HSD_PAD_R) as u32 != 0 {
                fp.input.held_buttons[0] |= HSD_PAD_LR as u32;
                fp.input.triggers[0] = 1.0;
            } else if fp.input.triggers[0] != 0.0 {
                fp.input.held_buttons[0] |= HSD_PAD_LR as u32;
            }
            if !e::gm_GetDbPauseFlag(0) && fp.input.held_buttons[0] & HSD_PAD_Z as u32 != 0 {
                fp.input.held_buttons[0] |= HSD_PAD_LR | (HSD_PAD_A as u32);
                fp.input.triggers[0] = c.z_press_analog_value;
            }
        }
        // TARGET_PC: SyncTest input hook, MELEE_SLP playback and recording.
        let port = fp.player_id as i32;
        let fol = fp.is_sub_fighter() as i32;
        let mut iv = [fp.input.lstick[0].x, fp.input.lstick[0].y, fp.input.cstick[0].x, fp.input.cstick[0].y, fp.input.triggers[0]];
        let mut ib = fp.input.held_buttons[0];
        if e::Snap_InputHook(port, fol, iv.as_mut_ptr(), &mut ib) != 0 {
            fp.input.lstick[0].x = iv[0];
            fp.input.lstick[0].y = iv[1];
            fp.input.cstick[0].x = iv[2];
            fp.input.cstick[0].y = iv[3];
            fp.input.triggers[0] = iv[4];
            fp.input.held_buttons[0] = ib;
        }
        if e::Replay_HasInput(port, fol) != 0 {
            fp.input.lstick[0].x = e::Replay_StickX(port, fol);
            fp.input.lstick[0].y = e::Replay_StickY(port, fol);
            fp.input.cstick[0].x = e::Replay_CStickX(port, fol);
            fp.input.cstick[0].y = e::Replay_CStickY(port, fol);
            fp.input.triggers[0] = e::Replay_Trigger(port, fol);
            fp.input.held_buttons[0] = e::Replay_Buttons(port, fol);
        }
        e::Replay_RecordInput(port, fol, fp.input.lstick[0].x, fp.input.lstick[0].y, fp.input.cstick[0].x, fp.input.cstick[0].y,
            fp.input.triggers[0], fp.input.held_buttons[0], fp.motion_id as i32, fp.cur_pos.x, fp.cur_pos.y, fp.facing_dir,
            fp.dmg.x1830_percent);
        Fighter_Spaghetti_8006AD10_Inner1(fp);
        if fp.x618_player_id == 0 && e::gm_IsCurrently1PMode_inline() != 0 {
            let mask = (HSD_PAD_L | HSD_PAD_R | HSD_PAD_START) as u32;
            let combo = fp.input.held_buttons[0] & mask == mask;
            if combo && !CSTICK_COMBO_WAS_HELD {
                let t = &mut *e::gm_CStickSmashTargetTest();
                *t = !*t;
                ext::OSReport("C-STICK: toggled\n");
            }
            CSTICK_COMBO_WAS_HELD = combo;
        }
        let hdz = c.horizontal_stick_smash_deadzone;
        let (lx, plx) = (fp.input.lstick[0].x, fp.input.lstick[1].x);
        if let Some(b) = stick_axis(lx, plx, hdz, &mut fp.active_duration.lstick.x, &mut fp.active_timer.lstick.x,
            &mut fp.active_sticky.lstick.x, &mut fp.activity_timer.lstick.x)
        {
            fp.set_x2228_b7(b as u8);
        }
        let vdz = c.vertical_stick_smash_deadzone;
        let (ly, ply) = (fp.input.lstick[0].y, fp.input.lstick[1].y);
        if let Some(b) = stick_axis(ly, ply, vdz, &mut fp.active_duration.lstick.y, &mut fp.active_timer.lstick.y,
            &mut fp.active_sticky.lstick.y, &mut fp.activity_timer.lstick.y)
        {
            // y up -> x2229_b0 = 0, y down -> 1
            fp.set_x2229_b0(!b as u8);
        }
        if e::lb_8000D148(fp.input.lstick[1].x, fp.input.lstick[1].y, fp.input.lstick[0].x, fp.input.lstick[0].y, 0.0, 0.0, hdz) {
            fp.activity_timer.lstick.y = 0;
            fp.activity_timer.lstick.x = 0;
        }
        let uport = fp.x618_player_id as i32;
        if !e::ftCo_IsCpuControlled(fp) && e::Replay_UcfCardinals(uport) != 0 {
            ftUcf_PadBufferPush(uport, e::Replay_RawStick(uport, 0), e::Replay_RawStick(uport, 1));
            if !(fp.kind == 0x13 && fp.motion_id == 0x15D) {
                ftUcf_Cardinal(e::Replay_RawStick(uport, 0), e::Replay_RawStick(uport, 1), &mut fp.input.lstick[0]);
                ftUcf_Cardinal(e::Replay_RawStick(uport, 2), e::Replay_RawStick(uport, 3), &mut fp.input.cstick[0]);
            }
            ftUcf_FlickCount(fp, uport);
        }
        bump(&mut fp.active_duration.trigger);
        let st = c.shield_press_threshold;
        if fp.input.triggers[0] >= st {
            if fp.input.triggers[1] >= st {
                bump(&mut fp.active_timer.trigger);
                bump(&mut fp.active_sticky.trigger);
                bump(&mut fp.activity_timer.trigger);
            } else {
                fp.activity_timer.trigger = 0;
                fp.active_duration.trigger = 0;
                fp.active_sticky.trigger = 0;
                fp.active_timer.trigger = 0;
            }
        } else {
            fp.activity_timer.trigger = 254;
            fp.active_sticky.trigger = 254;
            fp.active_timer.trigger = 254;
        }
        let pb = fp.input.pressed_buttons;
        let age = |pressed: bool, v: &mut u8| if pressed { *v = 0 } else if *v < 255 { *v += 1 };
        if pb & HSD_PAD_A as u32 != 0 {
            fp.x683 = fp.x67C;
            fp.x67C = 0;
        } else if fp.x67C < 255 {
            fp.x67C += 1;
        }
        age(pb & HSD_PAD_B as u32 != 0, &mut fp.x67D);
        age(pb & (HSD_PAD_X | HSD_PAD_Y) as u32 != 0, &mut fp.x67E);
        age(pb & HSD_PAD_DPADUP as u32 != 0, &mut fp.x681);
        age(pb & HSD_PAD_DPADDOWN as u32 != 0, &mut fp.x682);
        age(pb & HSD_PAD_LR as u32 != 0, &mut fp.x67F);
        if pb & (HSD_PAD_L | HSD_PAD_R) as u32 != 0 {
            fp.x684 = fp.x680;
            fp.x680 = 0;
        } else if fp.x680 < 255 {
            fp.x680 += 1;
        }
    }
    if fp.x221D_b4() != 0 || fp.x2224_b2() != 0 || e::gm_GetDbPauseFlag(2) {
        fp.input.lstick[2] = fp.input.lstick[0];
        fp.input.cstick[2] = fp.input.cstick[0];
        fp.input.triggers[2] = fp.input.triggers[0];
        fp.input.held_buttons[2] = fp.input.held_buttons[0];
        fp.set_x221D_b3(0);
        Fighter_UnkInitLoad_80068914_Inner1(g);
    }
    e::Geno_ComboInput(g);
    if fp.x2219_b5() == 0 {
        if !fp.x1980.is_null() {
            e::ftCommon_8007FFD8(fp, c.x6FC as f32);
        }
        e::ftCo_800DF0D0(g);
        e::ftCommon_8008031C(g);
        Fighter_UnkIncrementCounters_8006ABEC(g);
        if e::Geno_ComboTick(g) == 0 {
            if let Some(cb) = fp.input_cb {
                cb(g);
            }
        }
        e::ScriptGame_IntrWinTick(g);
    }
}


/// `Fighter_procUpdate` (proc 4): physics integration.
pub unsafe fn Fighter_procUpdate(g: G) {
    use ext as e;
    use melee_runtime::gekko::{cosf, sinf, sqrtf};
    use melee_runtime::lbtrigf::atan2f;
    let fp = fpm(g);
    ext::ScriptGame_FighterBenchTick(fp);
    if fp.x221F_b3() != 0 {
        return;
    }
    let c = cd();
    let mut wind;
    if fp.x2219_b5() == 0 {
        if fp.x2064_ledgeCooldown != 0 {
            fp.x2064_ledgeCooldown -= 1;
        }
        if fp.capture_timer != 0 {
            fp.capture_timer -= 1;
        }
        e::ftCo_800C0A98(g);
        if let Some(cb) = fp.phys_cb {
            cb(g);
        }
        let kbx0 = fp.x8c_kb_vel.x;
        if kbx0 != 0.0 || fp.x8c_kb_vel.y != 0.0 {
            if fp.ground_or_air == GA_Air {
                let kx = fp.x8c_kb_vel.x;
                let ky = fp.x8c_kb_vel.y;
                if fp.is_sandbag() != 0 {
                    fp.x8c_kb_vel.x = e::ftCommon_SandbagKnockbackDeaccel(fp.x8c_kb_vel.x, e::ftCommon_SandbagGetKnockbackDeaccelX(fp));
                    fp.x8c_kb_vel.y = e::ftCommon_SandbagKnockbackDeaccel(fp.x8c_kb_vel.y, e::ftCommon_SandbagGetKnockbackDeaccelY(fp));
                } else {
                    let a = atan2f(ky, kx);
                    let d = c.x204_knockbackFrameDecay;
                    if sqrtf(fmadds(kx, kx, ky * ky)) < d { // fused
                        fp.x8c_kb_vel.x = 0.0;
                        fp.x8c_kb_vel.y = 0.0;
                    } else {
                        fp.x8c_kb_vel.x = fmadds(-d, cosf(a), fp.x8c_kb_vel.x); // fused
                        fp.x8c_kb_vel.y = fmadds(-d, sinf(a), fp.x8c_kb_vel.y); // fused
                    }
                }
                fp.xF0_ground_kb_vel = 0.0;
            } else {
                if fp.xF0_ground_kb_vel == 0.0 {
                    fp.xF0_ground_kb_vel = kbx0;
                }
                let fr = e::ft_GetGroundFrictionMultiplier(fp) * fp.co_attrs.ground_friction * c.x200;
                e::ftCommon_ApplyGroundedKnockbackFriction(fp, fr);
                let n = fp.coll_data.floor.normal;
                fp.x8c_kb_vel.x = n.y * fp.xF0_ground_kb_vel;
                fp.x8c_kb_vel.y = -n.x * fp.xF0_ground_kb_vel;
            }
        }
        let sx0 = fp.x98_atk_shield_kb.x;
        if sx0 != 0.0 || fp.x98_atk_shield_kb.y != 0.0 {
            if fp.ground_or_air == GA_Air {
                let kx = fp.x98_atk_shield_kb.x;
                let ky = fp.x98_atk_shield_kb.y;
                let a = atan2f(ky, kx);
                let d = c.x3E8_shieldKnockbackFrameDecay;
                if sqrtf(fmadds(kx, kx, ky * ky)) < d { // fused
                    // retail bug (invisible ceiling glitch): clears kb_vel.y, not shield y
                    fp.x8c_kb_vel.y = 0.0;
                    fp.x98_atk_shield_kb.x = 0.0;
                } else {
                    fp.x98_atk_shield_kb.x = fmadds(-d, cosf(a), fp.x98_atk_shield_kb.x); // fused
                    fp.x98_atk_shield_kb.y = fmadds(-d, sinf(a), fp.x98_atk_shield_kb.y); // fused
                }
                fp.xF4_ground_attacker_shield_kb_vel = 0.0;
            } else {
                if fp.xF4_ground_attacker_shield_kb_vel == 0.0 {
                    fp.xF4_ground_attacker_shield_kb_vel = sx0;
                }
                let fr = e::ft_GetGroundFrictionMultiplier(fp) * fp.co_attrs.ground_friction * c.x3EC_shieldGroundFrictionMultiplier;
                e::ftCommon_ApplyShieldKnockbackFriction(fp, fr);
                let n = fp.coll_data.floor.normal;
                fp.x98_atk_shield_kb.x = n.y * fp.xF4_ground_attacker_shield_kb_vel;
                fp.x98_atk_shield_kb.y = -n.x * fp.xF4_ground_attacker_shield_kb_vel;
            }
        }
        fp.gr_vel += fp.xE4_ground_accel_1 + fp.xE8_ground_accel_2;
        fp.xE8_ground_accel_2 = 0.0;
        fp.xE4_ground_accel_1 = 0.0;
        fp.self_vel.x += fp.x74_self_accel.x;
        fp.self_vel.y += fp.x74_self_accel.y;
        fp.self_vel.z += fp.x74_self_accel.z;
        fp.x74_self_accel = v0();
        let mut sv = fp.self_vel;
        if fp.dmg.x1948 != 0 {
            let c2 = 1.0f32 - fp.dmg.x194C as f32 / fp.dmg.x1948 as f32;
            sv.x = fmadds(c2, fp.self_vel.x - fp.xA4_unk_vel.x, fp.xA4_unk_vel.x); // fused
            sv.y = fmadds(c2, fp.self_vel.y - fp.xA4_unk_vel.y, fp.xA4_unk_vel.y); // fused
            fp.dmg.x194C -= 1;
            if fp.dmg.x194C == 0 {
                fp.dmg.x1948 = 0;
            }
        }
        fp.cur_pos.x += fp.xF8_playerNudgeVel.x;
        fp.cur_pos.y += 0.0;
        fp.cur_pos.z += fp.xF8_playerNudgeVel.y;
        let sk = fp.x98_atk_shield_kb;
        if fp.x2222_b6() != 0 && fp.x2222_b7() == 0 {
            fp.xD4_unk_vel.x += sv.x;
            fp.xD4_unk_vel.y += sv.y;
            fp.xD4_unk_vel.z += sv.z;
            fp.xD4_unk_vel.x += fp.x8c_kb_vel.x;
            fp.xD4_unk_vel.y += fp.x8c_kb_vel.y;
            fp.xD4_unk_vel.z += 0.0;
            let bit = if fp.anon_2210.anon_0.throw_flags_b2() != 0 {
                fp.anon_2210.anon_0.set_throw_flags_b2(0);
                true
            } else {
                false
            };
            if bit || super::ftanim::ftAnim_80070FD0(fp) || fp.anon_594.anon_0.x594_b7() != 0 {
                fp.cur_pos.x += fp.xD4_unk_vel.x;
                fp.cur_pos.y += fp.xD4_unk_vel.y;
                fp.cur_pos.z += fp.xD4_unk_vel.z;
                fp.xD4_unk_vel = v0();
            }
            fp.cur_pos.x += sk.x;
            fp.cur_pos.y += sk.y;
            fp.cur_pos.z += sk.z;
        } else {
            fp.cur_pos.x += sv.x;
            fp.cur_pos.y += sv.y;
            fp.cur_pos.z += sv.z;
            fp.cur_pos.x += fp.x8c_kb_vel.x;
            fp.cur_pos.y += fp.x8c_kb_vel.y;
            fp.cur_pos.z += 0.0;
            fp.cur_pos.x += sk.x;
            fp.cur_pos.y += sk.y;
            fp.cur_pos.z += sk.z;
        }
        wind = v0();
        e::ftColl_GetWindOffsetVec(g, &mut wind);
    } else {
        wind = v0();
    }
    e::ftColl_80076528(g);
    if let Some(cb) = fp.hitlag_cb {
        cb(g);
    }
    if fp.ground_or_air == GA_Ground {
        let mut d = v0();
        if e::mpGetSpeed(fp.coll_data.floor.index, &fp.cur_pos, &mut d) {
            fp.cur_pos.x += d.x;
            fp.cur_pos.y += d.y;
            fp.cur_pos.z += d.z;
        }
    }
    fp.cur_pos.x += wind.x;
    fp.cur_pos.y += wind.y;
    fp.cur_pos.z += wind.z;
    e::ftCo_800D3158(g);
    if fp.x2225_b0() != 0 {
        if fp.prev_pos.y <= e::Stage_CalcUnkCamYBounds() && fp.cur_pos.y > e::Stage_CalcUnkCamYBounds() {
            fp.set_x2225_b0(0);
        }
    } else if fp.x222A_b1() == 0 && fp.x2228_b5() == 0 && fp.prev_pos.y >= e::Stage_CalcUnkCamY() && fp.cur_pos.y < e::Stage_CalcUnkCamY() {
        e::ft_PlaySFX(fp, 96, 127, 64);
        fp.set_x2225_b0(1);
    }
    if fp.dmg.x18A4_knockbackMagnitude != 0.0 && fp.x221C_b6() == 0 && !e::un_80322258(fp.cur_pos.x) {
        fp.dmg.x18A4_knockbackMagnitude = 0.0;
    }
    e::ftColl_8007AF28(g);
    if e::DbLevel_at_least_DebugRom() && (fp.cur_pos.x.is_nan() || fp.cur_pos.y.is_nan() || fp.cur_pos.z.is_nan()) {
        panic!("fighter.c:2517: fighter procUpdate pos error");
    }
}

/// `Fighter_UnkApplyTransformation_8006C0F0`: Flat Zone's squashed model matrix.
pub unsafe fn Fighter_UnkApplyTransformation_8006C0F0(g: G) {
    use ext as e;
    let fp = fpm(g);
    if fp.x34_scale.z != 1.0 {
        let j = jobj_of(g);
        let mut m1: Mtx = core::mem::zeroed();
        let mut m2: Mtx = core::mem::zeroed();
        e::HSD_JObjSetupMatrix(j);
        e::HSD_MtxInverse(&mut (*j).mtx, &mut m1);
        let mut s = v0();
        HSD_JObjGetScale(&*j, &mut s);
        s.x = ext::ftCommon_GetModelScale(fp);
        let mut r: Quaternion = core::mem::zeroed();
        HSD_JObjGetRotation(&*j, &mut r);
        let mut t = v0();
        HSD_JObjGetTranslation(&*j, &mut t);
        e::HSD_MtxSRT(&mut m2, &s, &r as *const Quaternion as *const Vec3, &t, core::ptr::null());
        e::PSMTXConcat(&mut m2, &mut m1, &mut fp.x44_mtx);
    }
}

/// `ftReplay_TraceFighter` (static, TARGET_PC).
pub unsafe fn ftReplay_TraceFighter(fp: &Fighter) {
    use ext as e;
    if e::Replay_Tracing() != 0 {
        e::Replay_TraceFighter(fp.player_id as i32, fp.is_sub_fighter() as i32, fp.kind as i32, fp.motion_id as i32,
            fp.cur_pos.x, fp.cur_pos.y, fp.facing_dir, fp.dmg.x1830_percent, e::Player_GetStocks(fp.player_id as i32),
            fp.self_vel.x, fp.self_vel.y, fp.x8c_kb_vel.x, fp.x8c_kb_vel.y, fp.gr_vel);
    }
}

/// `Fighter_procMap` (proc 6): map collision callback (+ TARGET_PC curated SyncTest record).
pub unsafe fn Fighter_procMap(g: G) {
    use ext as e;
    let fp = fpm(g);
    if fp.motion_id == ftCo_MS_Wait as _ && ext::ScriptGame_LabPreviewRow(fp, ftCo_MS_Wait as i32) >= 0 {
        return;
    }
    if fp.x221F_b3() == 0 {
        if fp.ecb_lock != 0 {
            fp.ecb_lock -= 1;
            if fp.ecb_lock == 0 {
                e::ftCommon_UnlockECB(fp);
            }
        }
        fp.set_x2223_b5(0);
        set_translate(jobj_of(g), &fp.cur_pos);
        if let Some(cb) = fp.coll_cb {
            e::Geno_CollBegin(g);
            cb(g);
            e::ftKb_SpecialN_800F1D24(g);
            e::Geno_CollEnd(g);
        }
        if fp.ground_or_air == GA_Ground {
            e::pl_80041280(fp.player_id as i32, fp.is_sub_fighter() as i32);
        }
        if e::DbLevel_at_least_DebugRom() && (fp.cur_pos.x.is_nan() || fp.cur_pos.y.is_nan() || fp.cur_pos.z.is_nan()) {
            panic!("fighter.c:2590: fighter procMap pos error");
        }
        set_translate(jobj_of(g), &fp.cur_pos);
    }
    if e::Replay_TraceAtProcMap() != 0 {
        ftReplay_TraceFighter(fp);
    }
    if e::Snap_Curated() != 0 {
        let w = curated_words(fp);
        e::Snap_CuratedMix(w.as_ptr(), w.len() as i32);
        let bw = e::ScriptGame_BuildHashWord();
        if bw != 0 {
            e::Snap_CuratedMix(&bw, 1);
        }
        let sub6 = if fp.is_sub_fighter() != 0 { 6 } else { 0 };
        e::Snap_CuratedRegion(fp.player_id as i32 + 1 + sub6, fp as *mut Fighter as usize, core::mem::size_of::<Fighter>() as u32);
        e::Snap_CuratedRegion(100 + fp.player_id as i32 + sub6, fp.gobj as usize, 0x40);
        let mut st: [J; 64] = [core::ptr::null_mut(); 64];
        let mut sp = 0usize;
        let mut n = 0;
        if !fp.gobj.is_null() && !(*fp.gobj).hsd_obj.is_null() {
            st[sp] = (*fp.gobj).hsd_obj as J;
            sp += 1;
        }
        while sp > 0 && n < 250 {
            sp -= 1;
            let j = st[sp];
            e::Snap_CuratedRegion(1000 + fp.player_id as i32 * 256 + n, j as usize, 0x88);
            n += 1;
            if !(*j).next.is_null() && sp < 63 {
                st[sp] = (*j).next;
                sp += 1;
            }
            if !(*j).child.is_null() && sp < 63 {
                st[sp] = (*j).child;
                sp += 1;
            }
        }
    }
}

/// The fighter's curated SyncTest record words (`Fighter_procMap`, MELEE_SYNCTEST_CURATED).
pub unsafe fn curated_words(fp: &Fighter) -> std::vec::Vec<u32> {
    let mut w = vec![
        fp.player_id as u32,
        fp.is_sub_fighter() as u32,
        fp.kind as u32,
        fp.motion_id as u32,
        fp.ground_or_air as u32,
        ext::Player_GetStocks(fp.player_id as i32) as u32,
        fp.input.held_buttons[0],
    ];
    for f in [fp.cur_pos.x, fp.cur_pos.y, fp.cur_pos.z, fp.facing_dir, fp.dmg.x1830_percent, fp.self_vel.x, fp.self_vel.y,
        fp.x8c_kb_vel.x, fp.x8c_kb_vel.y, fp.gr_vel, fp.dmg.x195c_hitlag_frames, fp.shield_health, fp.input.lstick[0].x,
        fp.input.lstick[0].y, fp.input.cstick[0].x, fp.input.cstick[0].y, fp.x3E4_fighterCmdScript.frame_count]
    {
        w.push(f.to_bits());
    }
    w.push(fp.x2219_b5() as u32);
    w.push(fp.x1968_jumpsUsed as u32);
    w.push(fp.x67F as u32);
    w.push(fp.x680 as u32);
    w.push(ext::ScriptGame_IntrWinHashWord(fp.player_id as i32 + 1 + if fp.is_sub_fighter() != 0 { 6 } else { 0 }));
    w
}

/// `Fighter_8006C5F4` (proc 7).
pub unsafe fn Fighter_8006C5F4(g: G) {
    if fpm(g).x221F_b3() == 0 {
        ext::ft_80089B08(g);
    }
}

/// `Fighter_CallAcessoryCallbacks_8006C624` (proc 8).
pub unsafe fn Fighter_CallAcessoryCallbacks_8006C624(g: G) {
    let fp = fpm(g);
    if fp.x221F_b3() != 0 {
        return;
    }
    if fp.x2219_b5() != 0 {
        if let Some(cb) = fp.accessory3_cb {
            cb(g);
        }
        return;
    }
    if let Some(cb) = fp.accessory2_cb {
        cb(g);
        set_translate(jobj_of(g), &fp.cur_pos);
    }
    if let Some(cb) = fp.accessory1_cb {
        cb(g);
        set_translate(jobj_of(g), &fp.cur_pos);
    }
}

/// `Fighter_8006C80C` (proc 9): hitbox / hurtbox world update.
pub unsafe fn Fighter_8006C80C(g: G) {
    use ext as e;
    let fp = fpm(g);
    if fp.x221F_b3() != 0 {
        return;
    }
    ext::efAsync_QueueFlush(g, &mut fp.x60C);
    Fighter_UnkApplyTransformation_8006C0F0(g);
    if fp.x2219_b5() == 0 {
        e::Mex_FighterCallbackDispatch(g, fp.accessory4_cb);
    }
    e::ftColl_8007AE80(g);
    e::ft_8007C224(g);
    e::ft_8007C6DC(g);
    if !fp.x20A0_accessory.is_null() {
        e::HSD_JObjAnimAll(fp.x20A0_accessory);
    }
    if fp.ground_or_air == GA_Air && fp.cur_pos.y < ext::Stage_GetCamBoundsBottomOffset() && e::ifMagnify_802FB6E8(fp.player_id as i32) == 3 {
        let mut o = v0();
        e::Stage_UnkSetVec3TCam_Offset(&mut o);
        if fp.cur_pos.y + o.y < fp.x2140 {
            fp.x2140 = fp.cur_pos.y + o.y;
        }
    }
}

/// `Fighter_UnkProcessGrab_8006CA5C` (proc 0xC).
pub unsafe fn Fighter_UnkProcessGrab_8006CA5C(g: G) {
    use ext as e;
    let fp = fpm(g);
    if fp.x221F_b3() != 0 || e::gm_8016B1C4() {
        return;
    }
    e::ftColl_8007BA0C(g);
    if fp.x221E_b6() == 0 {
        return;
    }
    e::ftColl_80078A2C(g);
    let sfx = (*(*fp.ft_data).x4C_sfx).x30 as i32;
    if !fp.victim_gobj.is_null() {
        if fp.x2225_b1() == 0 {
            e::ft_PlaySFX(fp, sfx, 0x7F, 0x40);
        }
        e::ftColl_80078754(g, fp.victim_gobj, 0);
        (fp.grab_cb.unwrap())(g);
        (fp.grabbed_cb.unwrap())(fp.victim_gobj, g);
        return;
    }
    e::ftColl_8007BC90(g);
    if !fp.target_item_gobj.is_null() {
        if fp.x2225_b1() == 0 {
            e::ft_PlaySFX(fp, sfx, 0x7F, 0x40);
        }
        e::it_8027B4A4(g, fp.target_item_gobj);
        if let Some(cb) = fp.x2194 {
            cb(g);
        }
    }
}

/// `Fighter_8006CB94` (proc 0xD): hit detection.
pub unsafe fn Fighter_8006CB94(g: G) {
    use ext as e;
    let fp = fpm(g);
    if fp.x221F_b3() == 0 && fp.x2219_b1() == 0 {
        e::ftColl_800765E0();
        e::ftColl_80078C70(g);
        e::ft_8007C77C(g);
        e::ftColl_8007925C(g);
        e::ftColl_8007BAC0(g);
        e::ft_8007C4BC(g);
        e::ftColl_8007AB48(g);
        e::ftColl_8007AB80(g);
        let f = e::ftColl_8007BBCC(g);
        if f > 0.0 {
            e::ftCommon_8007FC7C(g, f);
        }
    }
}

/// `Fighter_UnkTakeDamage_8006CC30` (TARGET_PC: script hit-rule percent commit first).
pub unsafe fn Fighter_UnkTakeDamage_8006CC30(fp: &mut Fighter, mut d: f32) {
    d = ext::ScriptGame_HitRulePercentCommit(fp, d);
    Fighter_TakeDamage_8006CC7C(fp, d);
    ext::ftCommon_8007EA90(fp, d);
}

/// `Fighter_TakeDamage_8006CC7C` (TARGET_PC: script damage overlay).
pub unsafe fn Fighter_TakeDamage_8006CC7C(fp: &mut Fighter, mut d: f32) {
    use ext as e;
    d = e::ScriptGame_ModDamage(fp.player_id as i32, 1, d);
    if fp.x2226_b4() == 0 || fp.x2226_b3() != 0 {
        fp.dmg.x1830_percent += d;
        if fp.metal_timer != 0 {
            fp.metal_health = ((fp.metal_health as f32) - d) as i32;
        }
        if fp.x2034 != 0 {
            fp.x2038 = ((fp.x2038 as f32) - d) as i32;
        }
        if fp.dmg.x1830_percent > 999.0 {
            fp.dmg.x1830_percent = 999.0;
        }
        ext::Player_SetHPByIndex(fp.player_id as i32, fp.is_sub_fighter() as i32, fp.dmg.x1830_percent);
        e::pl_8003EC9C(fp.player_id as i32, fp.is_sub_fighter() as i32, fp.dmg.x1830_percent, d);
        e::ftCo_800C8C84(fp.gobj);
    }
}

/// `Fighter_8006CDA4`: drop held items on a strong hit.
pub unsafe fn Fighter_8006CDA4(fp: &mut Fighter, arg1: i32) {
    use ext as e;
    let hold = !fp.item_gobj.is_null() && !e::itIsHeavy(fp.item_gobj);
    let tb = !(fp.x2220_b3() != 0 || fp.x2220_b4() != 0 || e::ftCo_8008E984(fp));
    let v = vec3_803B7494;
    let c = cd();
    if fp.motion_id != 0x145 && (fp.motion_id as u32).wrapping_sub(0x122) > 1 && fp.dmg.x1860_element != HitElement_Cape as _ && fp.x2226_b2() == 0 {
        if hold && tb
            && (e::HSD_Randi(c.x418 as i32) < arg1
                || (e::it_8026B30C(fp.item_gobj) == 3 && e::it_8026B594(fp.item_gobj) && e::HSD_Randi(c.x41C as i32) == 0))
        {
            if !fp.x1978.is_null() {
                ext::Item_8026ABD8(fp.x1978, &v, 1.0);
            }
            ext::Item_8026ABD8(fp.item_gobj, &v, 1.0);
        }
        if !fp.x197C.is_null() && e::HSD_Randi(c.x418 as i32) < arg1 {
            ext::ftCommon_8007F8E8(fp.gobj);
            ext::Item_8026ABD8(fp.x197C, &v, 1.0);
            ext::ftCommon_8007F9B4(fp.gobj);
        }
    }
}

/// `Fighter_8006CF5C`: queue a percent heal.
pub unsafe fn Fighter_8006CF5C(fp: &mut Fighter, arg1: i32) {
    if fp.x2224_b2() == 0 {
        fp.dmg.x18F0 += arg1;
        ext::ftCo_800BFFD0(fp, 8, 0);
        ext::ftCommon_8007EBAC(fp, 2, 0);
    }
}

/// `Fighter_8006CFE0`.
pub unsafe fn Fighter_8006CFE0(g: G) {
    let fp = fpm(g);
    if fp.x2219_b7() != 0 {
        if fp.allow_sdi() == 0 && fp.dmg.x1954 == 0.0 {
            Fighter_8006D10C(g);
        }
        fp.set_x2219_b7(0);
    }
}

/// `Fighter_UnkRecursiveFunc_8006D044`: enter hitlag, chained through `x1A5C`.
pub unsafe fn Fighter_UnkRecursiveFunc_8006D044(g: G) {
    let fp = fpm(g);
    if let Some(cb) = fp.pre_hitlag_cb {
        cb(g);
    }
    if fp.x2219_b5() == 0 {
        ext::Script_GameEvent(3, fp.player_id as i32, 1, fp.is_sub_fighter() as i32, 0);
    }
    fp.set_x2219_b5(1);
    if !fp.x1A5C.is_null() && fp.x2219_b7() == 0 {
        let ng = fp.x1A5C;
        setBit(ng);
        Fighter_UnkRecursiveFunc_8006D044(ng);
    }
}

/// `Fighter_8006D10C_Inline2` (static).
pub unsafe fn Fighter_8006D10C_Inline2(fp: &mut Fighter) {
    let g = fp.x1A5C;
    if !g.is_null() && fp.x2219_b7() == 0 {
        Fighter_8006CFE0(g);
    }
}

/// `Fighter_8006D10C_Inline1` (static).
pub unsafe fn Fighter_8006D10C_Inline1(g: G) {
    let fp = fpm(g);
    if fp.x2219_b7() != 0 {
        if fp.allow_sdi() == 0 && fp.dmg.x1954 == 0.0 {
            if let Some(cb) = fp.post_hitlag_cb {
                cb(g);
            }
            if fp.x2219_b5() != 0 {
                ext::Script_GameEvent(3, fp.player_id as i32, 0, fp.is_sub_fighter() as i32, 0);
            }
            fp.set_x2219_b5(0);
            Fighter_8006D10C_Inline2(fp);
        }
        fp.set_x2219_b7(0);
    }
}

/// `Fighter_8006D10C`: leave hitlag.
pub unsafe fn Fighter_8006D10C(g: G) {
    let fp = fpm(g);
    if let Some(cb) = fp.post_hitlag_cb {
        cb(g);
    }
    if fp.x2219_b5() != 0 {
        ext::Script_GameEvent(3, fp.player_id as i32, 0, fp.is_sub_fighter() as i32, 0);
    }
    fp.set_x2219_b5(0);
    if !fp.x1A5C.is_null() && fp.x2219_b7() == 0 {
        Fighter_8006D10C_Inline1(fp.x1A5C);
    }
}

/// The shield-hold drain of `Fighter_ProcessHit_8006D1EC` (two fused sites).
pub fn shield_drain(x284: f32, x288: f32, x2DC: f32, x2E0: f32, dmg_taken: i32, lightshield: f32) -> f32 {
    let inner = fmadds(lightshield, x2E0 - x2DC, x2DC); // fused
    fmadds(x284, dmg_taken as f32 * (1.0f32 - inner), x288) // fused
}

/// `Fighter_ProcessHit_8006D1EC` (proc 0xE): apply this frame's hit results.
pub unsafe fn Fighter_ProcessHit_8006D1EC(g: G) {
    use ext as e;
    let fp = fpm(g);
    let msid = fp.motion_id as i32;
    if fp.x221F_b3() != 0 {
        return;
    }
    let c = cd();
    let mut b1: i32 = 0;
    let (mut b2, mut b3, mut b4) = (false, false, false);
    e::Geno_OnHit(g);
    let lifted = e::Geno_HitReact(fp);
    let max_shield = |fp: &Fighter| h::script_value(fp, 4, c.x260_startShieldHealth);
    if fp.x221A_b7() == 0 && fp.shield_health < max_shield(fp) {
        fp.shield_health += h::script_value(fp, 10, c.x27C);
        if fp.shield_health > max_shield(fp) {
            fp.shield_health = max_shield(fp);
        }
    }
    if fp.x221A_b7() != 0 {
        fp.shield_health -= shield_drain(c.x284, c.x288, c.x2DC, c.x2E0, fp.x19A0_shieldDamageTaken as i32, fp.lightshield_amount);
        if fp.shield_health < 0.0 {
            b3 = true;
            fp.shield_health = c.x280_unkShieldHealth;
            e::pl_8003E058(fp.x19BC_shieldDamageTaken3 as i32, fp.x221F_b6() as i32, fp.player_id as i32, fp.is_sub_fighter() as i32);
        }
    }
    if fp.dmg.x189C_unk_num_frames > 0.0 {
        fp.dmg.x189C_unk_num_frames -= 1.0;
        if fp.dmg.x189C_unk_num_frames <= 0.0 && fp.dmg.kb_applied == 0.0 {
            fp.dmg.x189C_unk_num_frames = 0.0;
            e::ftColl_8007BE3C(g);
        }
    }
    let force = fp.dmg.kb_applied;
    if force != 0.0 {
        let ga = fp.ground_or_air as i32;
        fp.dmg.x189C_unk_num_frames = 0.0;
        Fighter_UnkTakeDamage_8006CC30(fp, fp.dmg.x1838_percentTemp);
        e::ftCo_Damage_CalcKnockback(fp);
        e::ftKb_SpecialN_800F5BA4(fp);
        if let Some(cb) = fp.take_dmg_2_cb {
            cb(g);
        }
        if fp.no_kb() == 0 {
            match fp.x1828 {
                0 => e::ftCo_8008EC90(g),
                1 => {
                    e::ftCommon_8007DB58(g);
                    e::ftCo_8008E908(g, 0.0);
                }
                2 => {
                    e::ftCommon_8007DB58(g);
                    e::ftCo_8008E9D0(g);
                }
                3 => {
                    e::ftCommon_8007DB58(g);
                    e::ftCo_8008EB58(g);
                }
                _ => {}
            }
            let applied = fp.dmg.x183C_applied as i32;
            b2 = true;
            if fp.x1828 == 0 && e::ScriptGame_ArmorAbsorbed(fp) != 0 {
                b2 = false;
            }
            e::ftCo_80090594(fp, fp.dmg.x1860_element as i32, applied, msid, ga, fp.x1960_vibrateMult);
            e::ftCommon_8007ED50(fp, fp.dmg.x1838_percentTemp);
            b1 = applied;
        } else {
            match fp.kind {
                Ft_Kind_MasterH => e::ftMh_MS_341_8014FE58(g),
                Ft_Kind_CrezyH => e::ftCh_Init_80156014(g),
                _ => panic!("fighter.c:3085: ellegal flag fp->no_reaction_always"),
            }
            e::ftCo_8008E9D0(g);
        }
    } else if fp.dmg.x18a0 != 0.0 {
        b1 = fp.dmg.x1840 as i32;
        b4 = true;
    } else if fp.x19A4 != 0 {
        if b3 {
            e::ftCo_80098B20(g);
            e::ft_PlaySFX(fp, 0x82, 0x7F, 0x40);
        } else if let Some(cb) = fp.shield_hit_cb {
            cb(g);
        }
        b1 = fp.x19A4;
    } else if fp.dmg.int_value != 0 {
        if fp.dmg.x191C != 0.0 && fp.victim_gobj.is_null() && fp.target_item_gobj.is_null() {
            e::ftCommon_8007DB58(g);
            e::ftCo_80099D9C(g);
        }
        b1 = fp.dmg.int_value as i32;
    } else if fp.dmg.x1914 != 0 {
        if let Some(cb) = fp.deal_dmg_cb {
            cb(g);
        }
        b1 = fp.dmg.x1914 as i32;
        if fp.x2070.anon_0.x2073 == 0x46 {
            e::ftCommon_8007EBAC(fp, 0xE, 0);
        } else {
            e::ftCommon_8007EE0C(fp, fp.dmg.x1914 as i32);
        }
    } else if fp.dmg.x1924 != 0 {
        b1 = fp.dmg.x1924 as i32;
    } else if fp.ReflectAttr.x1A3C_damageOver != 0 {
        e::ftCo_80098C9C(g);
    } else if fp.ReflectAttr.x1A2C_reflectHitDirection != 0.0 {
        if let Some(cb) = fp.reflect_hit_cb {
            cb(g);
        }
    } else if fp.AbsorbAttr.x1A40_absorbHitDirection != 0.0 {
        e::Mex_OnAbsorbDispatch(fp.kind as i32, g);
    } else if !fp.unk_gobj.is_null() {
        if let Some(cb) = fp.hurtbox_detect_cb {
            cb(g);
        }
    }
    if force == 0.0 && fp.dmg.x1838_percentTemp != 0.0 {
        Fighter_UnkTakeDamage_8006CC30(fp, fp.dmg.x1838_percentTemp);
        e::ftKb_SpecialN_800F5C34(fp);
        e::ftCommon_800804FC(fp);
    }
    e::ftCo_800C8D00(g);
    if lifted != 0 {
        fp.set_no_kb(1);
    }
    if b1 != 0 {
        fp.dmg.x195c_hitlag_frames = e::ftCommon_CalcHitlag(b1, msid, fp.x1960_vibrateMult);
        if fp.dmg.x195c_hitlag_frames < fp.x1964 {
            fp.dmg.x195c_hitlag_frames = fp.x1964;
        }
        if e::Geno_NoHitlag(fp) != 0 {
            fp.dmg.x195c_hitlag_frames = 0.0;
        }
        if fp.dmg.x195c_hitlag_frames > 0.0 {
            if fp.dmg.x195c_hitlag_frames > c.x194_unkHitLagFrames {
                fp.dmg.x195c_hitlag_frames = c.x194_unkHitLagFrames;
            }
            fp.set_allow_sdi(1);
            if b2 {
                fp.set_x221A_b3(1);
            }
            if b4 {
                fp.dmg.x189C_unk_num_frames = fp.dmg.x195c_hitlag_frames;
            }
            if fp.x2219_b5() == 0 {
                Fighter_UnkRecursiveFunc_8006D044(g);
            }
        }
    } else {
        ext::ftCo_80090718(fp);
    }
    if fp.x221A_b0() != 0 || fp.dmg.x1958 != 0.0 {
        if fp.x2219_b5() == 0 {
            Fighter_UnkRecursiveFunc_8006D044(g);
        }
        if fp.x221A_b0() != 0 {
            fp.set_x2219_b7(1);
            fp.set_x221A_b0(0);
        } else {
            if fp.dmg.x1958 > fp.dmg.x1954 {
                fp.dmg.x1954 = fp.dmg.x1958;
            }
            fp.dmg.x1958 = 0.0;
        }
    }
    if fp.dmg.x1928 != 0.0 {
        let ev = fmadds(fp.dmg.x1928, c.x3E0, c.x3E4); // fused
        fp.xF4_ground_attacker_shield_kb_vel = if fp.dmg.x192c < 0.0 { ev } else { -ev };
        e::ftCommon_8007E2A4(g);
    }
    ext::ScriptGame_HitRulePercentClear(fp);
    fp.dmg.x1838_percentTemp = 0.0;
    fp.dmg.x183C_applied = 0;
    fp.x1828 = 0;
    fp.dmg.kb_applied = 0.0;
    fp.dmg.x18a0 = 0.0;
    fp.dmg.x1840 = 0;
    fp.dmg.x1914 = 0;
    fp.dmg.int_value = 0;
    fp.dmg.x191C = 0.0;
    fp.unk_gobj = core::ptr::null_mut();
    fp.set_x221C_b5(0);
    fp.dmg.x1924 = 0;
    fp.dmg.x1928 = 0.0;
    fp.x19A0_shieldDamageTaken = 0;
    fp.x19A4 = 0;
    fp.x19A8 = core::ptr::null_mut();
    fp.ReflectAttr.x1A3C_damageOver = 0;
    fp.ReflectAttr.x1A2C_reflectHitDirection = 0.0;
    fp.AbsorbAttr.x1A40_absorbHitDirection = 0.0;
    fp.AbsorbAttr.x1A44_damageTaken = 0;
    fp.AbsorbAttr.x1A48_hitsTaken = 0;
    fp.x1960_vibrateMult = 1.0;
    fp.x1964 = 0.0;
    fp.dmg.x1950 = 0;
    if fp.x2219_b6() == 0 || fp.dmg.x18F4 != 0 {
        e::ftCo_800C2FD8(g);
    }
    e::ftCo_800A0DA4(fp);
}

/// `Fighter_8006D9AC` (proc 0x10).
pub unsafe fn Fighter_8006D9AC(g: G) {
    let fp = fpm(g);
    if fp.x221F_b3() != 0 || fp.x2219_b5() != 0 {
        return;
    }
    ext::ftCo_8009E0A8(g);
}

/// `Fighter_UnkCallCameraCallback_8006D9EC` (proc 0x12).
pub unsafe fn Fighter_UnkCallCameraCallback_8006D9EC(g: G) {
    let fp = fpm(g);
    if fp.x221F_b3() == 0 {
        ext::ftCommon_8008021C(g);
        if let Some(cb) = fp.cam_cb {
            cb(g);
        }
    }
    if ext::Replay_TraceAtProcMap() == 0 {
        ftReplay_TraceFighter(fp);
    }
}

/// `Fighter_8006DA4C` (proc 0x16): publish position / facing to the player block.
pub unsafe fn Fighter_8006DA4C(g: G) {
    use ext as e;
    let fp = fpm(g);
    if fp.x221F_b3() == 0 {
        let (s, sub) = (fp.player_id as i32, fp.is_sub_fighter() as i32);
        e::Player_80032828(s, sub, &fp.cur_pos);
        e::Player_SetFacingDirectionConditional(s, sub, fp.facing_dir);
        e::pl_8003FAA8(s, sub, &fp.cur_pos, &fp.prev_pos);
    }
}

/// `Fighter_Unload_8006DABC` (GObj user-data free).
pub unsafe fn Fighter_Unload_8006DABC(user_data: *mut core::ffi::c_void) {
    use ext as e;
    let fp = &mut *(user_data as F);
    let kind = fp.kind as i32;
    e::Mex_OnDestroyDispatch(kind, fp.gobj);
    e::ftColl_8007B8E8(fp.gobj);
    ext::efAsync_QueueClear(&mut fp.x60C);
    e::it_8026B7F8(fp.gobj);
    e::Camera_800290D4(fp.x890_cameraBox as _);
    e::ftCo_UnloadDynamicBones(fp);
    e::ftColl_800765AC(fp.gobj);
    e::ft_80088C5C(fp.gobj);
    e::lbShadow_8000EE8C(&mut fp.x20A4 as *mut _ as *mut _);
    if !fp.x20A0_accessory.is_null() {
        ext::HSD_JObjRemoveAll(fp.x20A0_accessory);
    }
    ext::HSD_JObjRemoveAll(fp.x8AC_animSkeleton);
    e::HSD_JObjUnref(fp.x2184 as _);
    e::ftData_800859A8(fp);
    e::HSD_LObjRemoveAll(fp.x588 as _);
    e::Player_80031FB0(fp.player_id as i32, fp.is_sub_fighter() as i32);
    ext::HSD_ObjFree(&mut fighter_x59C_alloc_data, fp.x59C as _);
    ext::HSD_ObjFree(&mut fighter_x59C_alloc_data, fp.x5A0 as _);
    ext::HSD_ObjFree(&mut fighter_parts_alloc_data, fp.parts as _);
    ext::HSD_ObjFree(&mut fighter_dobj_list_alloc_data, fp.dobj_list.data as _);
    ext::HSD_ObjFree(&mut fighter_x2040_alloc_data, fp.x203C.data as _);
    ext::HSD_ObjFree(&mut fighter_dat_attrs_alloc_data, fp.dat_attrs_backup as _);
    e::ScriptGame_HitRuleRetire(fp, core::mem::size_of::<Fighter>());
    ext::HSD_ObjFree(&mut fighter_alloc_data, user_data);
}

// ---------------------------------------------------------------------------------------------
// TARGET_PC rollback hashes (fighter.c "CURATED GAMEPLAY HASH" .. RB_XRegions).
// ---------------------------------------------------------------------------------------------


static mut ftRb_trace: bool = false;
static mut ftRb_hash_legacy: i32 = -1;
static mut nh_out: [u32; 16] = [0; 16];

#[inline]
fn ftRb_Mix(h: u32, v: u32) -> u32 {
    melee_types::neutral_hash::nh_mix(h, v)
}

/// `ftRb_TraceMix` (the `ftRb_Mix` macro of the C: the mixed word is also reported while tracing).
#[inline]
unsafe fn ftRb_TraceMix(h: u32, v: u32, symbol: &str) -> u32 {
    if ftRb_trace {
        ext::RbHashWord(v, symbol);
    }
    ftRb_Mix(h, v)
}

unsafe fn ftRb_Region(region: i32, slot: i32) {
    if ftRb_trace {
        ext::RbHashRegion(region, slot);
    }
}

#[inline]
fn ftRb_Bits(f: f32) -> u32 { f.to_bits() }

unsafe fn ftRb_GenoDefineWord(fp: F) -> u32 {
    let d = ext::GenoDefine_StateDigest(fp);
    if d != 0 { d ^ 0x4744_4946 } else { 0 }
}

/// `RB_HashSetLegacy`.
pub unsafe fn RB_HashSetLegacy(on: i32) {
    ftRb_hash_legacy = if on < 0 { -1 } else { (on != 0) as i32 };
}

unsafe fn ftRb_HashLegacy() -> bool {
    if ftRb_hash_legacy < 0 {
        ftRb_hash_legacy = ext::RbHashLegacyEnv();
    }
    ftRb_hash_legacy != 0
}

/// `RB_FighterHash`: the widened per-fighter words of the curated gameplay hash.
pub unsafe fn RB_FighterHash(h: u32, fp: &mut Fighter, slot: i32) -> u32 {
    let intr = ext::ScriptGame_IntrWinHashWord(if slot < 12 { (slot >> 1) + 1 + 6 * (slot & 1) } else { 0 });
    let gw = ftRb_GenoDefineWord(fp);
    let legacy = ftRb_HashLegacy();
    let stocks = if legacy { 0 } else { ext::Player_GetStocks(fp.player_id as i32) };
    let cpu = if legacy { false } else { ext::ftCo_IsCpuControlled(fp) };
    rb_fighter_hash_core(h, fp, slot, intr, gw, legacy, stocks, cpu)
}

/// The mixing of `RB_FighterHash` with its external inputs passed in: `intr` =
/// `ScriptGame_IntrWinHashWord(..)`, `gw` = the (already tagged) Geno define word, `stocks` =
/// `Player_GetStocks`, `cpu` = `ftCo_IsCpuControlled` (the last two only read when not `legacy`).
pub unsafe fn rb_fighter_hash_core(mut h: u32, fp: &Fighter, slot: i32, intr: u32, gw: u32, legacy: bool, stocks: i32, cpu: bool) -> u32 {
    macro_rules! mix { ($v:expr, $n:expr) => { h = ftRb_TraceMix(h, $v, $n) }; }
    ftRb_Region(1, slot);
    mix!((slot as u32).wrapping_add(0x100), "slot");
    mix!(fp.motion_id as u32, "motion_id");
    mix!(ftRb_Bits(fp.cur_pos.x), "cur_pos.x");
    mix!(ftRb_Bits(fp.cur_pos.y), "cur_pos.y");
    mix!(ftRb_Bits(fp.cur_pos.z), "cur_pos.z");
    mix!(ftRb_Bits(fp.self_vel.x), "self_vel.x");
    mix!(ftRb_Bits(fp.self_vel.y), "self_vel.y");
    mix!(ftRb_Bits(fp.dmg.x1830_percent), "percent");
    mix!(ftRb_Bits(fp.facing_dir), "facing_dir");
    mix!(intr, "IntrWinHashWord");
    if gw != 0 {
        h = ftRb_TraceMix(h, gw, "GenoDefine_StateDigest (tagged)");
    }
    if !legacy {
        mix!(0x5749_4445, "WIDE");
        mix!(fp.ground_or_air as u32, "ground_or_air");
        mix!(ftRb_Bits(fp.x8c_kb_vel.x), "kb_vel.x");
        mix!(ftRb_Bits(fp.x8c_kb_vel.y), "kb_vel.y");
        mix!(ftRb_Bits(fp.gr_vel), "gr_vel");
        mix!(ftRb_Bits(fp.dmg.x195c_hitlag_frames), "hitlag");
        mix!(fp.x2219_b5() as u32, "x2219_b5");
        mix!(ftRb_Bits(fp.shield_health), "shield_health");
        mix!(ftRb_Bits(fp.x3E4_fighterCmdScript.frame_count), "cmd frame_count");
        mix!(fp.x1968_jumpsUsed as u32, "jumpsUsed");
        mix!(fp.x67F as u32, "x67F");
        mix!(fp.x680 as u32, "x680");
        mix!(stocks as u32, "stocks");
        if cpu {
            let c = &fp.cpu;
            let base = c.buffer.as_ptr() as usize;
            let p = c.csP as usize;
            let off = if p >= base && p <= base + c.buffer.len() { (p - base) as u32 } else { 0 };
            mix!(0x4350_5541, "CPUA");
            mix!(c.buttons as u32, "cpu.buttons");
            mix!(((c.lstick.x as u8 as u32) << 24) | ((c.lstick.y as u8 as u32) << 16) | ((c.cstick.x as u8 as u32) << 8) | (c.cstick.y as u8 as u32), "cpu.sticks");
            mix!(((c.ltrigger as u32) << 8) | c.rtrigger as u32, "cpu.triggers");
            mix!(c.kind as u32, "cpu.kind");
            mix!(c.level as u32, "cpu.level");
            for v in [c.x14, c.x18, c.x1C, c.x20, c.x24, c.x28, c.x2C, c.x30, c.x34] {
                mix!(v as u32, "cpu.xN");
            }
            for v in [c.x38, c.x3C, c.x40] {
                mix!(ftRb_Bits(v), "cpu.xN(f)");
            }
            for v in [c.x7C, c.x80, c.x84] {
                mix!(v as u32, "cpu.xN");
            }
            mix!(c.xC8 as u32, "cpu.xC8");
            mix!(c.xEC as u32, "cpu.xEC");
            mix!(c.command_duration as u32, "cpu.command_duration");
            h = ftRb_TraceMix(h, off, "cpu.csP - cpu.buffer");
            let b = c.bf_f8;
            mix!(((b[0] as u32) << 24) | ((b[1] as u32) << 16) | ((b[2] as u32) << 8) | b[3] as u32, "cpu flag bytes");
        }
    }
    h
}

/// Items on the item plink list (`HSD_GOBJ_PLINK_ITEM`).
unsafe fn item_gobjs() -> impl Iterator<Item = G> {
    let mut g = h::plink_head(HSD_GOBJ_PLINK_ITEM);
    core::iter::from_fn(move || {
        if g.is_null() { return None; }
        let cur = g;
        g = (*g).next;
        Some(cur)
    })
}

/// `RB_ItemHash`: sum of per-item words (order independent); the count goes to `count_out`.
pub unsafe fn RB_ItemHash(count_out: Option<&mut u32>) -> u32 {
    let (mut sum, mut n) = (0u32, 0u32);
    for g in item_gobjs() {
        let ip = (*g).user_data as *mut Item;
        if ip.is_null() { continue; }
        let ip = &*ip;
        let stable = ext::Geno_ArtStableKind(ip.kind as i32);
        ftRb_Region(2, n as i32);
        let mut ih = ftRb_Mix(0x4954_454D, if stable != 0 { stable } else { ip.kind as u32 });
        ih = ftRb_Mix(ih, ip.msid as u32);
        for v in [ip.pos.x, ip.pos.y, ip.pos.z, ip.x40_vel.x, ip.x40_vel.y, ip.x40_vel.z] {
            ih = ftRb_Mix(ih, ftRb_Bits(v));
        }
        sum = sum.wrapping_add(ih);
        n += 1;
    }
    if let Some(c) = count_out { *c = n; }
    sum
}

/// `Snap_CuratedItems`.
pub unsafe fn Snap_CuratedItems() {
    static mut SEEN_MAX: u32 = 0;
    static mut SEEN_FRAMES: u32 = 0;
    let mut n = 0u32;
    for g in item_gobjs() {
        let ip = (*g).user_data as *mut Item;
        if ip.is_null() { continue; }
        let ip = &*ip;
        let w: [u32; 10] = [
            0x4954_454D, n, ip.kind as u32, ip.msid as u32,
            ip.pos.x.to_bits(), ip.pos.y.to_bits(), ip.pos.z.to_bits(),
            ip.x40_vel.x.to_bits(), ip.x40_vel.y.to_bits(), ip.x40_vel.z.to_bits(),
        ];
        ext::Snap_CuratedMix(w.as_ptr(), 10);
        n += 1;
    }
    if n != 0 { SEEN_FRAMES += 1; }
    if n > SEEN_MAX {
        SEEN_MAX = n;
        ext::OSReport(&format!("curated items: {} on the list (max so far), {} frames with items\n", n, SEEN_FRAMES));
    }
    let w = [0x4954_434Eu32, n];
    ext::Snap_CuratedMix(w.as_ptr(), 2);
}

unsafe fn ftRb_GameHash(seed: u32) -> u32 {
    let mut h = 0x811C_9DC5u32;
    ftRb_Region(0, 0);
    h = ftRb_TraceMix(h, seed, "HSD_RandSeedPtr (effective online seed)");
    let bw = ext::ScriptGame_BuildHashWord();
    if bw != 0 {
        h = ftRb_TraceMix(h, bw, "ScriptGame_BuildHashWord");
    }
    for i in 0..6 {
        for j in 0..2 {
            let g = ext::Player_GetEntityAtIndex(i, j);
            if g.is_null() { continue; }
            h = RB_FighterHash(h, &mut *((*g).user_data as F), i * 2 + j);
        }
    }
    if !ftRb_HashLegacy() {
        let mut n = 0u32;
        let w = RB_ItemHash(Some(&mut n));
        ftRb_Region(0, 1);
        if n != 0 {
            h = ftRb_TraceMix(h, n, "RB_ItemHash.count");
            h = ftRb_TraceMix(h, w, "RB_ItemHash.sum");
        }
    }
    h
}

/// `RB_GameHash`.
pub unsafe fn RB_GameHash() -> u32 {
    ftRb_GameHash(*ext::HSD_RandSeedPtr())
}

/// `RB_GameHashForFrame`.
pub unsafe fn RB_GameHashForFrame(frame: i32) -> u32 {
    let arrived = *ext::HSD_RandSeedPtr();
    let effective = ext::Replay_ChecksumSeed(arrived, frame);
    ext::RbHashSeed(arrived, effective);
    ftRb_trace = true;
    let h = ftRb_GameHash(effective);
    ftRb_trace = false;
    h
}

/// `RB_NeutralHashWord`.
pub unsafe fn RB_NeutralHashWord(i: i32) -> u32 {
    if (0..16).contains(&i) { nh_out[i as usize] } else { 0 }
}

/// `RB_NeutralHash` (NHV1).
pub unsafe fn RB_NeutralHash() -> u32 {
    use melee_types::neutral_hash::{nh_fighter, nh_item, nh_mix};
    nh_out = [0; 16];
    nh_out[1] = *ext::HSD_RandSeedPtr();
    let mut h = nh_mix(0x811C_9DC5, 0x4E48_5631);
    h = nh_mix(h, nh_out[1]);
    for i in 0..6 {
        for j in 0..2 {
            let fg = ext::Player_GetEntityAtIndex(i, j);
            if fg.is_null() || (*fg).user_data.is_null() { continue; }
            let fp = &mut *((*fg).user_data as F);
            let cpu = ext::ftCo_IsCpuControlled(fp);
            let w = nh_fighter(fp, i * 2 + j, ext::Player_GetStocks(fp.player_id as i32), cpu);
            nh_out[(4 + i * 2 + j) as usize] = w;
            h = nh_mix(h, w);
        }
    }
    let (mut sum, mut n) = (0u32, 0u32);
    for g in item_gobjs() {
        let ip = (*g).user_data as *mut Item;
        if ip.is_null() { continue; }
        sum = sum.wrapping_add(nh_item(&*ip));
        n += 1;
    }
    nh_out[2] = n;
    nh_out[3] = sum;
    h = nh_mix(h, 0x4954_434E);
    h = nh_mix(h, n);
    h = nh_mix(h, sum);
    nh_out[0] = h;
    h
}

/// `RB_Perturb`: negative control, perturbs one hashed value of the first fighter (or item).
pub unsafe fn RB_Perturb(code: i32) {
    if code == 11 {
        let g = h::plink_head(HSD_GOBJ_PLINK_ITEM);
        if !g.is_null() && !(*g).user_data.is_null() {
            (*((*g).user_data as *mut Item)).pos.x += 1.0;
        }
        return;
    }
    for i in 0..6 {
        for j in 0..2 {
            let g = ext::Player_GetEntityAtIndex(i, j);
            if g.is_null() { continue; }
            let fp = &mut *((*g).user_data as F);
            match code {
                1 => fp.dmg.x195c_hitlag_frames += 1.0,
                2 => fp.x1968_jumpsUsed = fp.x1968_jumpsUsed.wrapping_add(1),
                3 => fp.shield_health -= 1.0,
                4 => fp.x680 = fp.x680.wrapping_add(1),
                5 => fp.x67F = fp.x67F.wrapping_add(1),
                6 => fp.x8c_kb_vel.x += 0.25,
                7 => fp.gr_vel += 0.25,
                8 => fp.x3E4_fighterCmdScript.frame_count += 1.0,
                9 => { let b = fp.x2219_b5(); fp.set_x2219_b5((b == 0) as _); }
                10 => fp.cur_pos.x += 0.01,
                12 => { let p = fp.player_id as i32; ext::Player_SetStocks(p, ext::Player_GetStocks(p) + 1); }
                13 => {
                    if ext::ftCo_IsCpuControlled(fp) {
                        fp.cpu.command_duration = fp.cpu.command_duration.wrapping_add(1);
                    } else {
                        continue;
                    }
                }
                _ => {}
            }
            return;
        }
    }
}

/// Depth-first walk of a joint tree as the C does it (explicit 64-entry stack, 400 joints max).
unsafe fn walk_joints(root: J, mut f: impl FnMut(J, i32)) {
    let mut st: [J; 64] = [core::ptr::null_mut(); 64];
    let (mut sp, mut n) = (0usize, 0i32);
    st[sp] = root;
    sp += 1;
    while sp > 0 && n < 400 {
        sp -= 1;
        let jo = st[sp];
        f(jo, n);
        n += 1;
        if !(*jo).next.is_null() && sp < 63 { st[sp] = (*jo).next; sp += 1; }
        if !(*jo).child.is_null() && sp < 63 && ((*jo).flags & JOBJ_INSTANCE as u32) == 0 { st[sp] = (*jo).child; sp += 1; }
    }
}

/// `Fighter_BenchSetupMatrices`.
pub unsafe fn Fighter_BenchSetupMatrices() {
    for i in 0..6 {
        for j in 0..2 {
            let g = ext::Player_GetEntityAtIndex(i, j);
            if g.is_null() || (*g).hsd_obj.is_null() || ((*((*g).user_data as F)).motion_id as i32) < ftCo_MS_Wait as i32 { continue; }
            walk_joints((*g).hsd_obj as J, |jo, _| ext::HSD_JObjSetupMatrix(jo));
        }
    }
}

unsafe fn ftRb_XTree(tagbase: i32, g: G) {
    if g.is_null() || (*g).hsd_obj.is_null() { return; }
    walk_joints((*g).hsd_obj as J, |jo, n| ext::Snap_XRegion(tagbase + n, jo as usize, 0x84));
}

/// `RB_XRegions`.
pub unsafe fn RB_XRegions() {
    let mut n = 0;
    for i in 0..6 {
        for j in 0..2 {
            let g = ext::Player_GetEntityAtIndex(i, j);
            if g.is_null() || (*g).user_data.is_null() { continue; }
            let ud = (*g).user_data as usize;
            let k = i * 2 + j;
            ext::Snap_XRegion(0x010000 + k * 0x100, ud, 0x2144);
            ext::Snap_XRegion(0x010001 + k * 0x100, ud + 0x2164, (core::mem::size_of::<Fighter>() - 0x2164) as u32);
            ext::Snap_XRegion(0x020000 + k * 0x100, g as usize, 0x40);
            ftRb_XTree(0x030000 + k * 0x1000, g);
        }
    }
    for g in item_gobjs() {
        if n >= 64 { break; }
        n += 1;
        let k = n - 1;
        if (*g).user_data.is_null() { continue; }
        let ud = (*g).user_data as usize;
        ext::Snap_XRegion(0x040000 + k * 0x100, ud, 0xD64);
        ext::Snap_XRegion(0x040001 + k * 0x100, ud + 0xD7C, 0xDD4 - 0xD7C);
        ext::Snap_XRegion(0x040002 + k * 0x100, ud + 0xFCC, (core::mem::size_of::<Item>() - 0xFCC) as u32);
        ext::Snap_XRegion(0x050000 + k * 0x100, g as usize, 0x40);
        ftRb_XTree(0x060000 + k * 0x1000, g);
    }
}


// ---------------------------------------------------------------------------------------------
// technical_ai.inc + cpu_assist.inc (TARGET_PC-only CPU technique assists, included by fighter.c).
// ---------------------------------------------------------------------------------------------


#[derive(Clone, Copy)]
#[repr(C)]
pub struct TechAiPolicy {
    pub seed: u32,
    pub skill: i32, pub delay: i32, pub wait: i32, pub type_: i32, pub motion: i32, pub fired: i32, pub accepted: i32, pub cooldown: i32,
    pub opportunities: i32, pub lcancels: i32, pub techs: i32, pub misses: i32, pub last_event: i32,
}
#[derive(Clone, Copy, Default)]
pub struct TechAiObservation {
    pub type_: i32, pub motion: i32, pub floor: i32, pub fresh_shoulder: i32, pub tech_unlocked: i32,
    pub eta: f32,
}

/// `tech_ai_policy_init` (technical_ai_policy.h).
pub fn tech_ai_policy_init(s: &mut TechAiPolicy, skill: i32, seed: u32) {
    *s = unsafe { core::mem::zeroed() };
    s.skill = skill;
    s.seed = seed;
    s.delay = if skill == 1 { 6 } else if skill == 2 { 4 } else { 2 };
}

/// `tech_ai_random`: xorshift32.
pub fn tech_ai_random(s: &mut TechAiPolicy) -> u32 {
    let mut x = s.seed;
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    s.seed = x;
    x
}

/// `tech_ai_policy_step`: type 1 = light analogue L-cancel, type 2 = digital ground-tech attempt.
pub fn tech_ai_policy_step(s: &mut TechAiPolicy, o: &TechAiObservation) -> i32 {
    if s.skill < 1 || s.skill > 3 { return 0; }
    if s.cooldown > 0 { s.cooldown -= 1; }
    if o.floor == 0 || o.type_ < 1 || o.type_ > 2 || !(o.eta > 0.0 && o.eta <= 12.0) {
        s.type_ = 0;
        s.fired = 0;
        return 0;
    }
    if s.type_ != o.type_ || s.motion != o.motion {
        s.type_ = o.type_;
        s.motion = o.motion;
        s.wait = s.delay;
        s.fired = 0;
        let chance = if s.skill == 1 { 60 } else if s.skill == 2 { 80 } else { 95 };
        s.accepted = (((tech_ai_random(s) % 100) as i32) < chance) as i32;
        s.opportunities += 1;
        if s.accepted == 0 { s.misses += 1; }
    }
    if s.fired != 0 || s.accepted == 0 { return 0; }
    if s.wait > 0 { s.wait -= 1; return 0; }
    if s.cooldown > 0 || o.fresh_shoulder == 0 { return 0; }
    if o.type_ == 1 && o.eta > 5.0 { return 0; }
    if o.type_ == 2 && (o.tech_unlocked == 0 || o.eta > 10.0) { return 0; }
    let event = o.type_;
    s.fired = 1;
    s.last_event = event;
    s.cooldown = if event == 1 { 8 } else { 40 };
    if event == 1 { s.lcancels += 1; } else { s.techs += 1; }
    event
}

#[derive(Clone, Copy)]
pub struct TechAiSlot {
    pub fighter: *mut Fighter,
    pub spawn: i32,
    pub owner: i32,
    pub policy: TechAiPolicy,
}
pub static mut technical_ai_slots: [TechAiSlot; 6] = unsafe { core::mem::zeroed() };

unsafe fn technical_ai_live(gobj: G) -> bool {
    if gobj.is_null() { return false; }
    let mut cur = h::plink_head(HSD_GOBJ_PLINK_FIGHTER);
    while !cur.is_null() {
        if cur == gobj { return true; }
        cur = (*cur).next;
    }
    false
}

unsafe fn technical_ai_supported(fp: *mut Fighter) -> bool {
    !fp.is_null() && (*fp).is_sub_fighter() == 0 &&
        ((*fp).kind == Ft_Kind_Fox as _ || (*fp).kind == Ft_Kind_Falco as _) &&
        ext::Player_GetPlayerSlotType((*fp).player_id as i32) == Gm_PKind_Cpu as i32 &&
        ext::ftCo_IsCpuControlled(fp) && (*fp).cpu.kind == CpuKind_4 as _
}

/// `ScriptGame_CpuTechnical`.
pub unsafe fn ScriptGame_CpuTechnical(slot: i32, owner: i32, skill: i32, seed: i32) -> i32 {
    if slot < 0 || slot >= 6 || owner <= 0 || skill < 0 || skill > 3 ||
        (skill != 0 && seed <= 0) || ext::Netplay_Enabled() != 0 || ext::RB_Enabled() != 0 ||
        ext::Replay_Active() != 0 { return 0; }
    let state = &mut technical_ai_slots[slot as usize];
    if state.owner != 0 && state.owner != owner { return 0; }
    if skill == 0 { *state = core::mem::zeroed(); return 1; }
    let gobj = ext::Player_GetEntity(slot);
    let fp: *mut Fighter = if technical_ai_live(gobj) { (*gobj).user_data as F } else { core::ptr::null_mut() };
    if !technical_ai_supported(fp) { return 0; }
    state.fighter = fp;
    state.spawn = (*fp).x8_spawnNum;
    state.owner = owner;
    tech_ai_policy_init(&mut state.policy, skill, seed as u32);
    1
}

/// `ScriptGame_CpuTechnicalClear`.
pub unsafe fn ScriptGame_CpuTechnicalClear(owner: i32) {
    ScriptGame_CpuAssistClear(owner);
    for i in 0..6 {
        if owner == 0 || technical_ai_slots[i].owner == owner {
            technical_ai_slots[i] = core::mem::zeroed();
        }
    }
}

/// `ScriptGame_CpuTechnicalState`.
pub unsafe fn ScriptGame_CpuTechnicalState(slot: i32, owner: i32, field: i32) -> i32 {
    if slot < 0 || slot >= 6 || owner <= 0 { return 0; }
    let s = &technical_ai_slots[slot as usize];
    if s.owner != owner { return 0; }
    match field {
        0 => s.policy.skill,
        1 => s.policy.opportunities,
        2 => s.policy.lcancels,
        3 => s.policy.techs,
        4 => s.policy.misses,
        5 => s.policy.last_event,
        6 => s.policy.delay,
        _ => 0,
    }
}

/// The floor ray of the overlays: from the ECB bottom along velocity * `horizon` frames.
unsafe fn assist_floor_ray(fp: &Fighter, vx: f32, vy: f32, horizon: f32, floor: &mut Vec3) -> bool {
    let mut normal = v0();
    let (mut line, mut flags) = (0i32, 0u32);
    let x = fp.cur_pos.x + fp.coll_data.ecb.bottom.x;
    let y = fp.cur_pos.y + fp.coll_data.ecb.bottom.y;
    ext::mpCheckFloor(x, y + 0.01, fmadds(vx, horizon, x), fmadds(vy, horizon, y), 0.0, floor, &mut line, &mut flags, &mut normal,
        fp.coll_data.floor_skip as i32, fp.coll_data.joint_id_skip as i32, fp.coll_data.joint_id_only as i32,
        core::ptr::null_mut(), fp.gobj)
}

/// `technical_ai_overlay`: called once during CPU input synthesis, before LR normalization.
pub unsafe fn technical_ai_overlay(fp: &mut Fighter) {
    let mut o = TechAiObservation::default();
    let mut floor = v0();
    if fp.player_id >= 6 || fp.is_sub_fighter() != 0 { return; }
    let slot = &mut technical_ai_slots[fp.player_id as usize];
    if slot.owner == 0 { return; }
    if slot.fighter != fp as *mut Fighter || slot.spawn != fp.x8_spawnNum {
        *slot = core::mem::zeroed();
        return;
    }
    if ext::Netplay_Enabled() != 0 || ext::RB_Enabled() != 0 || ext::Replay_Active() != 0 {
        *slot = core::mem::zeroed();
        return;
    }
    if ext::gm_GetDbPauseFlag(0) || ext::gm_GetDbPauseFlag(2) { return; }
    if !technical_ai_supported(fp) || fp.ground_or_air != GA_Air as _ || fp.dmg.x195c_hitlag_frames > 0.0 {
        tech_ai_policy_step(&mut slot.policy, &o);
        return;
    }
    o.motion = fp.motion_id as i32;
    let m = fp.motion_id as i32;
    if m >= ftCo_MS_AttackAirN as i32 && m <= ftCo_MS_AttackAirLw as i32 && fp.cmd_vars[0] != 0 && fp.allow_interrupt() == 0 {
        o.type_ = 1;
    } else if m >= ftCo_MS_DamageFlyHi as i32 && m <= ftCo_MS_DamageFlyRoll as i32 && fp.x221C_b6() != 0 {
        o.type_ = 2;
    }
    let vx = fp.self_vel.x + fp.x8c_kb_vel.x;
    let vy = fp.self_vel.y + fp.x8c_kb_vel.y;
    if o.type_ == 0 || !(vy < -0.05 && vy > -100.0 && vx > -100.0 && vx < 100.0) {
        tech_ai_policy_step(&mut slot.policy, &o);
        return;
    }
    let y = fp.cur_pos.y + fp.coll_data.ecb.bottom.y;
    if assist_floor_ray(fp, vx, vy, 12.0, &mut floor) {
        o.floor = 1;
        o.eta = (y - floor.y) / -vy;
    }
    o.fresh_shoulder = (fp.input.held_buttons[1] & HSD_PAD_LR as u32 == 0
        && fp.input.held_buttons[0] & (HSD_PAD_L | HSD_PAD_R | HSD_PAD_Z) as u32 == 0
        && fp.input.triggers[0] == 0.0) as i32;
    o.tech_unlocked = (fp.x680 as i32 >= cd().x1C as i32) as i32;
    let event = tech_ai_policy_step(&mut slot.policy, &o);
    if event == 1 {
        fp.input.triggers[0] = 0.35;
    } else if event == 2 {
        fp.input.held_buttons[0] |= HSD_PAD_R as u32;
    }
}

/// `ScriptGame_CpuTechnicalTest`: disc-free native policy checks; nonzero reports a failed case.
pub unsafe fn ScriptGame_CpuTechnicalTest() -> i32 {
    let mut a: TechAiPolicy = core::mem::zeroed();
    let mut b: TechAiPolicy = core::mem::zeroed();
    let mut o = TechAiObservation { type_: 1, motion: 65, floor: 1, fresh_shoulder: 1, tech_unlocked: 1, eta: 4.0 };
    let prior = technical_ai_slots[5];
    let (mut pulse, mut errors) = (0, 0);
    tech_ai_policy_init(&mut a, 3, 1);
    tech_ai_policy_init(&mut b, 3, 1);
    for i in 0..10 {
        let x = tech_ai_policy_step(&mut a, &o);
        let y = tech_ai_policy_step(&mut b, &o);
        if x != y || (i < 2 && x != 0) { errors += 1; }
        pulse += (x == 1) as i32;
    }
    if pulse != 1 || a.lcancels != 1 || a.delay != 2 { errors += 1; }
    tech_ai_policy_init(&mut a, 3, 1);
    o.floor = 0;
    for _ in 0..20 { if tech_ai_policy_step(&mut a, &o) != 0 { errors += 1; } }
    if a.opportunities != 0 { errors += 1; }
    o.floor = 1; o.type_ = 2; o.tech_unlocked = 0;
    for _ in 0..10 { if tech_ai_policy_step(&mut a, &o) != 0 { errors += 1; } }
    o.tech_unlocked = 1;
    if tech_ai_policy_step(&mut a, &o) != 2 { errors += 1; }
    for _ in 0..60 { if tech_ai_policy_step(&mut a, &o) != 0 { errors += 1; } }
    tech_ai_policy_init(&mut a, 3, 1);
    o.fresh_shoulder = 0;
    for _ in 0..10 { if tech_ai_policy_step(&mut a, &o) != 0 { errors += 1; } }
    o.fresh_shoulder = 1;
    if tech_ai_policy_step(&mut a, &o) != 2 { errors += 1; }
    tech_ai_policy_init(&mut a, 1, 1);
    o.type_ = 1;
    for _ in 0..12 { if tech_ai_policy_step(&mut a, &o) != 0 { errors += 1; } }
    if a.misses != 1 || a.delay != 6 { errors += 1; }
    technical_ai_slots[5] = core::mem::zeroed();
    technical_ai_slots[5].owner = 723;
    technical_ai_slots[5].policy.skill = 3;
    ScriptGame_CpuTechnicalClear(724);
    if ScriptGame_CpuTechnicalState(5, 723, 0) != 3 || ScriptGame_CpuTechnicalState(5, 724, 0) != 0 { errors += 1; }
    ScriptGame_CpuTechnicalClear(723);
    if technical_ai_slots[5].owner != 0 { errors += 1; }
    technical_ai_slots[5] = prior;
    errors
}

// ---- cpu_assist.inc ----

pub const CA_N: usize = 5;
pub const CA_LCANCEL: usize = 0;
pub const CA_PSHIELD: usize = 1;
pub const CA_TECH: usize = 2;
pub const CA_WAVEDASH: usize = 3;
pub const CA_FASTFALL: usize = 4;
pub const CA_DIR_IN: i32 = 0;
pub const CA_DIR_AWAY: i32 = 1;
pub const CA_DIR_TOWARD: i32 = 2;
pub const CA_DIR_RANDOM: i32 = 3;
const CA_LC_ETA: i32 = 5;
const CA_TECH_ETA: i32 = 10;
const CA_PS_HOLD: i32 = 12;
const CA_WD_ANGLE_X: f32 = 0.9397;
const CA_WD_ANGLE_Y: f32 = -0.3420;

#[derive(Clone, Copy)]
pub struct CpuAssistSlot {
    pub fighter: *mut Fighter,
    pub frames: i32, pub logged: i32, pub ps_log: i32,
    pub spawn: i32, pub owner: i32,
    pub seed: u32,
    pub permille: [i32; CA_N],
    pub tech_dir: i32,
    pub opportunities: [i32; CA_N], pub performed: [i32; CA_N],
    pub prev_motion: i32, pub motion_age: i32,
    pub lc_state: i32,
    pub tech_state: i32, pub tech_hold: i32, pub tech_x: i32,
    pub ps_hold: i32, pub ps_gap: i32, pub ps_state: i32,
    pub wd_state: i32, pub wd_hold: i32, pub wd_dir: i32,
    pub ff_state: i32, pub ff_tries: i32,
}
pub static mut cpu_assist_slots: [CpuAssistSlot; 6] = unsafe { core::mem::zeroed() };

#[derive(Clone, Copy, Default)]
struct LastThreat { foe: i32, hb: i32, hc: i32, state: i32, motion: i32, frame: i32, d2: f32, r: f32, damage: f32, px: f32, py: f32, ax: f32, ay: f32 }
static mut cpu_assist_last_threat: LastThreat = LastThreat { foe: 0, hb: 0, hc: 0, state: 0, motion: 0, frame: 0, d2: 0.0, r: 0.0, damage: 0.0, px: 0.0, py: 0.0, ax: 0.0, ay: 0.0 };

/// `CA_LOG`: the first 300 injections of a slot are logged.
unsafe fn ca_log(s: &mut CpuAssistSlot, fp: &Fighter, what: &str) {
    if s.logged < 300 {
        s.logged += 1;
        ext::OSReport(&format!("CPU-ASSIST: P{} {} frame={} motion={}\n", fp.player_id as i32 + 1, what, s.frames, fp.motion_id as i32));
    }
}

/// `cpu_assist_roll`: one xorshift32 draw, 0..999.
pub fn cpu_assist_roll(s: &mut CpuAssistSlot) -> i32 {
    let mut x = s.seed;
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    s.seed = x;
    (((x >> 8) as u64 * 1000u64) >> 24) as i32
}

/// `cpu_assist_decide`: counts an opportunity of technique `t`, draws once.
pub fn cpu_assist_decide(s: &mut CpuAssistSlot, t: usize, sub: Option<&mut i32>) -> i32 {
    let r = cpu_assist_roll(s);
    let p = s.permille[t];
    s.opportunities[t] += 1;
    if r >= p { return 0; }
    if let Some(sub) = sub { *sub = if p > 0 { r * 1000 / p } else { 0 }; }
    1
}

unsafe fn cpu_assist_supported(fp: *mut Fighter) -> bool {
    !fp.is_null() && (*fp).is_sub_fighter() == 0 && (*fp).player_id < 6 &&
        ext::Player_GetPlayerSlotType((*fp).player_id as i32) == Gm_PKind_Cpu as i32 && ext::ftCo_IsCpuControlled(fp)
}

/// `cpu_assist_config_ok`.
pub fn cpu_assist_config_ok(lc: i32, ps: i32, tech: i32, tdir: i32, wd: i32, ff: i32) -> bool {
    (0..=1000).contains(&lc) && (0..=1000).contains(&ps) && (0..=1000).contains(&tech) &&
        (0..=1000).contains(&wd) && (0..=1000).contains(&ff) && (0..=CA_DIR_RANDOM).contains(&tdir)
}

/// `ScriptGame_CpuAssist`.
pub unsafe fn ScriptGame_CpuAssist(slot: i32, owner: i32, lc: i32, ps: i32, tech: i32, tdir: i32, wd: i32, ff: i32, seed: i32) -> i32 {
    if slot < 0 || slot >= 6 || owner <= 0 || seed <= 0 || !cpu_assist_config_ok(lc, ps, tech, tdir, wd, ff) ||
        ext::Netplay_Enabled() != 0 || ext::RB_Enabled() != 0 || ext::Replay_Active() != 0 { return 0; }
    let s = &mut cpu_assist_slots[slot as usize];
    if s.owner != 0 && s.owner != owner { return 0; }
    let gobj = ext::Player_GetEntity(slot);
    let fp: *mut Fighter = if technical_ai_live(gobj) { (*gobj).user_data as F } else { core::ptr::null_mut() };
    if !cpu_assist_supported(fp) { return 0; }
    *s = core::mem::zeroed();
    s.fighter = fp; s.spawn = (*fp).x8_spawnNum; s.owner = owner; s.seed = seed as u32;
    s.permille[CA_LCANCEL] = lc; s.permille[CA_PSHIELD] = ps; s.permille[CA_TECH] = tech;
    s.permille[CA_WAVEDASH] = wd; s.permille[CA_FASTFALL] = ff; s.tech_dir = tdir;
    s.prev_motion = -1;
    1
}

/// `ScriptGame_CpuAssistClearSlot`.
pub unsafe fn ScriptGame_CpuAssistClearSlot(slot: i32, owner: i32) -> i32 {
    if slot < 0 || slot >= 6 || owner <= 0 { return 0; }
    if cpu_assist_slots[slot as usize].owner != owner { return 0; }
    cpu_assist_slots[slot as usize] = core::mem::zeroed();
    1
}

/// `ScriptGame_CpuAssistClear`.
pub unsafe fn ScriptGame_CpuAssistClear(owner: i32) {
    for i in 0..6 {
        if owner == 0 || cpu_assist_slots[i].owner == owner {
            cpu_assist_slots[i] = core::mem::zeroed();
        }
    }
}

/// `ScriptGame_CpuAssistState`: 0 enabled, 1..5 permille, 6 tech_dir, 7 seed, 8..12 opportunities, 13..17 performed.
pub unsafe fn ScriptGame_CpuAssistState(slot: i32, owner: i32, field: i32) -> i32 {
    if slot < 0 || slot >= 6 || owner <= 0 { return 0; }
    let s = &cpu_assist_slots[slot as usize];
    if s.owner != owner { return 0; }
    if field == 0 { return 1; }
    if (1..=5).contains(&field) { return s.permille[(field - 1) as usize]; }
    if field == 6 { return s.tech_dir; }
    if field == 7 { return s.seed as i32; }
    if (8..=12).contains(&field) { return s.opportunities[(field - 8) as usize]; }
    if (13..=17).contains(&field) { return s.performed[(field - 13) as usize]; }
    0
}

/// `cpu_assist_eta`: landing of a falling fighter along its own velocity; Some((eta, floor_y)) when a floor lies within 12 frames.
unsafe fn cpu_assist_eta(fp: &Fighter) -> Option<(f32, f32)> {
    let vx = fp.self_vel.x + fp.x8c_kb_vel.x;
    let vy = fp.self_vel.y + fp.x8c_kb_vel.y;
    if !(vy < -0.05 && vy > -100.0 && vx > -100.0 && vx < 100.0) { return None; }
    let y = fp.cur_pos.y + fp.coll_data.ecb.bottom.y;
    let mut floor = v0();
    if !assist_floor_ray(fp, vx, vy, 12.0, &mut floor) { return None; }
    let eta = (y - floor.y) / -vy;
    if eta > 0.0 && eta <= 12.0 { Some((eta, floor.y)) } else { None }
}

unsafe fn cpu_assist_nearest_foe(fp: &Fighter) -> *mut Fighter {
    let mut best: *mut Fighter = core::ptr::null_mut();
    let mut bd = 1e30f32;
    for i in 0..6 {
        if i == fp.player_id as i32 { continue; }
        let g = ext::Player_GetEntity(i);
        if !technical_ai_live(g) { continue; }
        let o = (*g).user_data as F;
        if o.is_null() || o as *const Fighter == fp as *const Fighter || ((*o).motion_id as i32) < ftCo_MS_Wait as i32 { continue; }
        let mut d = (*o).cur_pos.x - fp.cur_pos.x;
        if d < 0.0 { d = -d; }
        if d < bd { bd = d; best = o; }
    }
    best
}

/// `cpu_assist_seg_dist2`: squared distance from point q to the segment a-b.
pub fn cpu_assist_seg_dist2(q: &Vec3, a: &Vec3, b: &Vec3) -> f32 {
    let (abx, aby, abz) = (b.x - a.x, b.y - a.y, b.z - a.z);
    let (apx, apy, apz) = (q.x - a.x, q.y - a.y, q.z - a.z);
    // fused: abx*abx + aby*aby + abz*abz
    let len2 = fmadds(abz, abz, fmadds(abx, abx, aby * aby));
    let mut t = 0.0f32;
    if len2 > 1e-6 {
        // fused: apx*abx + apy*aby + apz*abz
        t = fmadds(apz, abz, fmadds(apx, abx, apy * aby)) / len2;
        if t < 0.0 { t = 0.0; } else if t > 1.0 { t = 1.0; }
    }
    // fused: c - a*b
    let dx = fmadds(-abx, t, apx);
    let dy = fmadds(-aby, t, apy);
    let dz = fmadds(-abz, t, apz);
    // fused: dx*dx + dy*dy + dz*dz
    fmadds(dz, dz, fmadds(dx, dx, dy * dy))
}

/// A foe's hitbox moved one more frame along its own sweep overlaps one of this fighter's enabled hurtbox capsules.
unsafe fn cpu_assist_threat(fp: &Fighter) -> bool {
    for i in 0..6 {
        if i == fp.player_id as i32 { continue; }
        let g = ext::Player_GetEntity(i);
        if !technical_ai_live(g) { continue; }
        let o = (*g).user_data as F;
        if o.is_null() || o as *const Fighter == fp as *const Fighter { continue; }
        for hh in 0..4 {
            let hb = &(*o).x914[hh];
            if hb.state == HitCapsule_Disabled as _ || hb.damage <= 0.0 { continue; }
            let mut seen = false;
            for v in 0..12 {
                let vv = hb.victims_1[v].victim;
                if vv == fp as *const Fighter as *mut core::ffi::c_void || vv == fp.gobj as *mut core::ffi::c_void { seen = true; }
            }
            if seen { continue; }
            let p = Vec3 { x: hb.x4C.x + (hb.x4C.x - hb.x58.x), y: hb.x4C.y + (hb.x4C.y - hb.x58.y), z: hb.x4C.z + (hb.x4C.z - hb.x58.z) };
            let mut c = 0;
            while c < fp.hurt_capsules_len as usize && c < 15 {
                let hc = &fp.hurt_capsules[c];
                let r = hb.scale + hc.capsule.scale;
                if hc.capsule.state == HurtCapsule_Enabled as _ {
                    if cpu_assist_seg_dist2(&p, &hc.capsule.a_pos, &hc.capsule.b_pos) <= r * r {
                        let t = &mut cpu_assist_last_threat;
                        t.foe = i + 1; t.hb = hh as i32; t.hc = c as i32;
                        t.state = hb.state as i32; t.motion = (*o).motion_id as i32;
                        t.d2 = cpu_assist_seg_dist2(&p, &hc.capsule.a_pos, &hc.capsule.b_pos);
                        t.r = r; t.damage = hb.damage;
                        t.px = p.x; t.py = p.y;
                        t.ax = hc.capsule.a_pos.x; t.ay = hc.capsule.a_pos.y;
                        return true;
                    }
                }
                c += 1;
            }
        }
    }
    false
}

/// `cpu_assist_dump_hits`: PS diagnostics log.
unsafe fn cpu_assist_dump_hits(fp: &Fighter) {
    for i in 0..6 {
        if i == fp.player_id as i32 { continue; }
        let g = ext::Player_GetEntity(i);
        if !technical_ai_live(g) { continue; }
        let o = (*g).user_data as F;
        if o.is_null() || o as *const Fighter == fp as *const Fighter { continue; }
        for hh in 0..4 {
            let hb = &(*o).x914[hh];
            let (mut gap_now, mut gap_next) = (1e9f32, 1e9f32);
            if hb.state == HitCapsule_Disabled as _ { continue; }
            let p = Vec3 { x: hb.x4C.x + (hb.x4C.x - hb.x58.x), y: hb.x4C.y + (hb.x4C.y - hb.x58.y), z: hb.x4C.z + (hb.x4C.z - hb.x58.z) };
            let mut c = 0;
            while c < fp.hurt_capsules_len as usize && c < 15 {
                let hc = &fp.hurt_capsules[c];
                let r = hb.scale + hc.capsule.scale;
                if hc.capsule.state == HurtCapsule_Enabled as _ {
                    // fused: d - r*r
                    let d = fmadds(-r, r, cpu_assist_seg_dist2(&hb.x4C, &hc.capsule.a_pos, &hc.capsule.b_pos));
                    if d < gap_now { gap_now = d; }
                    let d = fmadds(-r, r, cpu_assist_seg_dist2(&p, &hc.capsule.a_pos, &hc.capsule.b_pos));
                    if d < gap_next { gap_next = d; }
                }
                c += 1;
            }
            let vict = hb.victims_1.iter().filter(|v| !v.victim.is_null()).count();
            ext::OSReport(&format!("CPU-ASSIST:     foe P{} hb{} state={} dmg={:.6} scale={:.6} gap_now={:.6} gap_next={:.6} victims={} foe_motion={} hitlag={:.6}\n",
                i + 1, hh, hb.state as i32, hb.damage as f64, hb.scale as f64, gap_now as f64, gap_next as f64, vict, (*o).motion_id as i32, (*o).dmg.x195c_hitlag_frames as f64));
        }
    }
}

fn cpu_assist_air_actionable(m: i32) -> bool {
    (m >= ftCo_MS_JumpF as i32 && m <= ftCo_MS_FallAerial as i32) || (m >= ftCo_MS_AttackAirN as i32 && m <= ftCo_MS_AttackAirLw as i32)
}

/// `cpu_assist_overlay`: the per-entity technique assist applied on top of the AI's virtual pad.
pub unsafe fn cpu_assist_overlay(fp: &mut Fighter) {
    if fp.player_id >= 6 || fp.is_sub_fighter() != 0 { return; }
    let s = &mut cpu_assist_slots[fp.player_id as usize];
    if s.owner == 0 { return; }
    if ext::Netplay_Enabled() != 0 || ext::RB_Enabled() != 0 || ext::Replay_Active() != 0 || !cpu_assist_supported(fp) {
        *s = core::mem::zeroed();
        return;
    }
    if s.fighter != fp as *mut Fighter || s.spawn != fp.x8_spawnNum {
        s.fighter = fp; s.spawn = fp.x8_spawnNum; s.prev_motion = -1;
        s.lc_state = 0; s.tech_state = 0; s.tech_hold = 0; s.ps_hold = 0; s.ps_state = 0; s.ps_gap = 0;
        s.wd_state = 0; s.wd_hold = 0; s.ff_state = 0; s.ff_tries = 0;
    }
    if ext::gm_GetDbPauseFlag(0) || ext::gm_GetDbPauseFlag(2) { return; }
    if fp.dmg.x195c_hitlag_frames > 0.0 { return; }
    let m = fp.motion_id as i32;
    let air = fp.ground_or_air == GA_Air as _;
    s.frames += 1;
    let from = s.prev_motion;
    if m != s.prev_motion { s.prev_motion = m; s.motion_age = 1; } else if s.motion_age < 1000 { s.motion_age += 1; }
    let fresh = fp.input.held_buttons[1] & HSD_PAD_LR as u32 == 0
        && fp.input.held_buttons[0] & (HSD_PAD_L | HSD_PAD_R | HSD_PAD_Z) as u32 == 0
        && fp.input.triggers[0] == 0.0;
    let air_atk = m >= ftCo_MS_AttackAirN as i32 && m <= ftCo_MS_AttackAirLw as i32;
    let dmg_fly = m >= ftCo_MS_DamageFlyHi as i32 && m <= ftCo_MS_DamageFlyRoll as i32;

    // ---- L-cancel ----
    let eta_lc = if air && air_atk && fp.cmd_vars[0] != 0 && fp.allow_interrupt() == 0 { cpu_assist_eta(fp) } else { None };
    if let Some((eta, _)) = eta_lc {
        if s.lc_state == 0 && eta <= CA_LC_ETA as f32 + 1.0 { s.lc_state = if cpu_assist_decide(s, CA_LCANCEL, None) != 0 { 2 } else { 1 }; }
        if s.lc_state == 2 && eta <= CA_LC_ETA as f32 && fresh {
            fp.input.triggers[0] = 0.35;
            s.lc_state = 3;
            s.performed[CA_LCANCEL] += 1;
            ca_log(s, fp, "lcancel");
        }
    } else if !air || !air_atk {
        s.lc_state = 0;
    }

    // ---- tech ----
    let eta_tech = if air && dmg_fly && fp.x221C_b6() != 0 { cpu_assist_eta(fp) } else { None };
    if let Some((eta, _)) = eta_tech {
        if s.tech_state == 0 && eta <= CA_TECH_ETA as f32 + 1.0 {
            let mut sub = 0;
            if cpu_assist_decide(s, CA_TECH, Some(&mut sub)) != 0 {
                let mut d = s.tech_dir;
                if d == CA_DIR_RANDOM { d = if sub < 333 { CA_DIR_IN } else if sub < 667 { CA_DIR_AWAY } else { CA_DIR_TOWARD }; }
                if d == CA_DIR_IN {
                    s.tech_x = 0;
                } else {
                    let foe = cpu_assist_nearest_foe(fp);
                    let toward = if !foe.is_null() { if (*foe).cur_pos.x >= fp.cur_pos.x { 1.0f32 } else { -1.0 } } else { fp.facing_dir };
                    s.tech_x = if (if d == CA_DIR_TOWARD { toward } else { -toward }) > 0.0 { 1 } else { -1 };
                }
                s.tech_state = 2;
            } else {
                s.tech_state = 1;
            }
        }
        if s.tech_state == 2 && eta <= CA_TECH_ETA as f32 && fresh && fp.x680 as i32 >= cd().x1C as i32 {
            fp.input.held_buttons[0] |= HSD_PAD_R as u32;
            s.tech_hold = (cd().x250 + 0.5) as i32;
            s.tech_state = 3;
            s.performed[CA_TECH] += 1;
            ca_log(s, fp, "tech");
        }
    } else if !air || !dmg_fly {
        if s.tech_state != 0 && !dmg_fly { s.tech_state = 0; }
    }
    if s.tech_hold > 0 {
        fp.input.lstick[0].x = s.tech_x as f32;
        fp.input.lstick[0].y = 0.0;
        s.tech_hold -= 1;
    }

    // ---- perfect shield ----
    if s.ps_log > 0 {
        s.ps_log -= 1;
        ext::OSReport(&format!("CPU-ASSIST: P{} ps+{} motion={} kb={:.6} ps_window={} guard={:.6}\n", fp.player_id as i32 + 1, 3 - s.ps_log, m,
            fp.dmg.kb_applied as f64, fp.x221C_b2() as i32, fp.mv.co.guard.x18 as f64));
        cpu_assist_dump_hits(fp);
    }
    if s.ps_hold > 0 {
        fp.input.held_buttons[0] |= HSD_PAD_R as u32;
        s.ps_hold -= 1;
    } else if !air && m >= ftCo_MS_Wait as i32 && m <= ftCo_MS_WalkFast as i32 {
        if cpu_assist_threat(fp) {
            if s.ps_state == 0 { s.ps_state = if cpu_assist_decide(s, CA_PSHIELD, None) != 0 { 2 } else { 1 }; }
            if s.ps_state == 2 && fresh {
                fp.input.held_buttons[0] |= HSD_PAD_R as u32;
                s.ps_hold = CA_PS_HOLD - 1;
                s.ps_state = 3;
                s.performed[CA_PSHIELD] += 1;
                ca_log(s, fp, "perfect_shield");
                let t = &cpu_assist_last_threat;
                ext::OSReport(&format!("CPU-ASSIST:   threat foe=P{} hitbox={} hurt={} state={} foe_motion={} d2={:.6} r={:.6} dmg={:.6} hit=({:.6},{:.6}) hurt_a=({:.6},{:.6})\n",
                    t.foe, t.hb, t.hc, t.state, t.motion, t.d2 as f64, t.r as f64, t.damage as f64, t.px as f64, t.py as f64, t.ax as f64, t.ay as f64));
                s.ps_log = 4;
            }
            s.ps_gap = 0;
        } else {
            s.ps_gap += 1;
            if s.ps_gap > 8 { s.ps_state = 0; }
        }
    } else {
        s.ps_gap += 1;
        if s.ps_gap > 8 { s.ps_state = 0; }
    }

    // ---- wavedash ----
    if s.wd_hold > 0 {
        fp.input.lstick[0].x = s.wd_dir as f32 * CA_WD_ANGLE_X;
        fp.input.lstick[0].y = CA_WD_ANGLE_Y;
        if s.wd_hold >= 2 { fp.input.held_buttons[0] |= HSD_PAD_R as u32; }
        s.wd_hold -= 1;
    } else if m == ftCo_MS_KneeBend as i32 && !air {
        if s.motion_age == 1 {
            let lx = fp.input.lstick[0].x;
            let foe = cpu_assist_nearest_foe(fp);
            s.wd_state = if cpu_assist_decide(s, CA_WAVEDASH, None) != 0 { 2 } else { 1 };
            if s.wd_state == 2 {
                let fx = if lx > 0.3 { 1.0f32 } else if lx < -0.3 { -1.0 }
                    else if !foe.is_null() { if (*foe).cur_pos.x >= fp.cur_pos.x { 1.0 } else { -1.0 } } else { fp.facing_dir };
                s.wd_dir = if fx > 0.0 { 1 } else { -1 };
                let (mut floor, mut normal) = (v0(), v0());
                let (mut line, mut flags) = (0i32, 0u32);
                // fused: x + fx*12.0 (both x args)
                let ax = fmadds(fx, 12.0, fp.cur_pos.x);
                if !ext::mpCheckFloor(ax, fp.cur_pos.y + 4.0, fmadds(fx, 12.0, fp.cur_pos.x), fp.cur_pos.y - 6.0, 0.0,
                    &mut floor, &mut line, &mut flags, &mut normal, -1, -1, -1, core::ptr::null_mut(), core::ptr::null_mut()) {
                    s.wd_state = 1;
                }
            }
        }
    } else if s.wd_state == 2 && from == ftCo_MS_KneeBend as i32 && (m == ftCo_MS_JumpF as i32 || m == ftCo_MS_JumpB as i32) && s.motion_age == 1 {
        s.wd_state = 3;
        s.wd_hold = 3;
        fp.input.lstick[0].x = s.wd_dir as f32 * CA_WD_ANGLE_X;
        fp.input.lstick[0].y = CA_WD_ANGLE_Y;
        fp.input.held_buttons[0] |= HSD_PAD_R as u32;
        s.wd_hold -= 1;
        s.performed[CA_WAVEDASH] += 1;
        ca_log(s, fp, "wavedash");
    } else if m != ftCo_MS_JumpF as i32 && m != ftCo_MS_JumpB as i32 {
        s.wd_state = 0;
    }

    // ---- fast fall ----
    if !air {
        s.ff_state = 0;
        s.ff_tries = 0;
    } else if cpu_assist_air_actionable(m) && fp.self_vel.y < 0.0 && fp.fall_fast() == 0 {
        if s.ff_state == 0 { s.ff_state = if cpu_assist_decide(s, CA_FASTFALL, None) != 0 { 2 } else { 1 }; }
        if s.ff_state == 2 && s.ff_tries < 4 && fp.input.lstick[0].y > -cd().x88 {
            fp.input.lstick[0].y = -1.0;
            let first = s.ff_tries == 0;
            s.ff_tries += 1;
            if first { s.performed[CA_FASTFALL] += 1; ca_log(s, fp, "fast_fall"); }
        }
    }
}

/// `ScriptGame_CpuAssistTest`: disc-free native checks of the pure parts; nonzero reports a failed case.
pub unsafe fn ScriptGame_CpuAssistTest() -> i32 {
    let mut a: CpuAssistSlot = core::mem::zeroed();
    let mut b: CpuAssistSlot = core::mem::zeroed();
    let prior = cpu_assist_slots[5];
    let mut errors = 0;
    let mut sub = 0;
    a.seed = 12345; b.seed = 12345;
    for _ in 0..500 {
        a.permille[CA_TECH] = 0;
        if cpu_assist_decide(&mut a, CA_TECH, None) != 0 { errors += 1; }
        a.permille[CA_LCANCEL] = 1000;
        if cpu_assist_decide(&mut a, CA_LCANCEL, None) == 0 { errors += 1; }
    }
    if a.opportunities[CA_TECH] != 500 || a.opportunities[CA_LCANCEL] != 500 { errors += 1; }
    a.seed = 777; b.seed = 777;
    a.permille[CA_PSHIELD] = 500; b.permille[CA_PSHIELD] = 500;
    for _ in 0..200 { if cpu_assist_roll(&mut a) != cpu_assist_roll(&mut b) { errors += 1; break; } }
    a.seed = 777; b.seed = 778;
    let mut hits = 0;
    for _ in 0..50 { hits += (cpu_assist_roll(&mut a) == cpu_assist_roll(&mut b)) as i32; }
    if hits > 10 { errors += 1; }
    a.seed = 99; a.permille[CA_WAVEDASH] = 500; a.opportunities[CA_WAVEDASH] = 0;
    hits = 0;
    for _ in 0..20000 { hits += cpu_assist_decide(&mut a, CA_WAVEDASH, Some(&mut sub)); }
    if hits < 9500 || hits > 10500 || a.opportunities[CA_WAVEDASH] != 20000 { errors += 1; }
    a.seed = 5; a.permille[CA_TECH] = 250;
    for _ in 0..4000 { if cpu_assist_decide(&mut a, CA_TECH, Some(&mut sub)) != 0 && (sub < 0 || sub > 999) { errors += 1; } }
    if cpu_assist_config_ok(-1, 0, 0, 0, 0, 0) || cpu_assist_config_ok(0, 1001, 0, 0, 0, 0) ||
        cpu_assist_config_ok(0, 0, 0, 4, 0, 0) || !cpu_assist_config_ok(1000, 1000, 1000, CA_DIR_RANDOM, 1000, 1000) { errors += 1; }
    cpu_assist_slots[5] = core::mem::zeroed();
    cpu_assist_slots[5].owner = 723;
    cpu_assist_slots[5].permille[CA_TECH] = 400;
    ScriptGame_CpuAssistClear(724);
    if ScriptGame_CpuAssistState(5, 723, 3) != 400 || ScriptGame_CpuAssistState(5, 724, 3) != 0 { errors += 1; }
    if ScriptGame_CpuAssistClearSlot(5, 724) != 0 || ScriptGame_CpuAssistClearSlot(5, 723) == 0 || cpu_assist_slots[5].owner != 0 { errors += 1; }
    cpu_assist_slots[5] = prior;
    errors
}
