//! Native Geno translation. Shared records and bitfields come from melee_types.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code, unused_imports, unused_mut, unused_variables, unused_assignments, unused_parens, unused_unsafe, static_mut_refs, clippy::all)]
use core::ffi::c_void;
use melee_core::fp::fmadds;
pub use melee_types::*;
pub use melee_runtime::gekko::sqrtf;
macro_rules! OSReport { ($f:expr $(,$a:expr)* $(,)?) => {{
    let text=unsafe { core::ffi::CStr::from_ptr($f) }.to_string_lossy();
    eprintln!("{} {:?}",text, &[$(crate::tu::ext_hook::Val::to_u64($a)),*] as &[u64]);
}}; }
macro_rules! HSD_ForeachAnim { ($a:expr,$b:expr,$c:expr,$d:expr,$e:expr,$f:expr $(,)?) => {
    ext::HSD_ForeachAnim($a,$b,$c,$d,$e,crate::tu::ext_hook::Val::to_u64($f))
}; }
unsafe fn __assert(file:*const i8,line:u32,condition:*const i8)->! {
    panic!("{}:{}: {}",core::ffi::CStr::from_ptr(file).to_string_lossy(),line,core::ffi::CStr::from_ptr(condition).to_string_lossy())
}
pub mod ext {
    use super::*;
    macro_rules! hooks {
        ($(fn $n:ident($($a:ident:$t:ty),* $(,)?) $(->$r:ty)? $(= $def:path)?;)*) => {
            pub mod hook { use super::*; $(pub static mut $n: Option<fn($($t),*) $(->$r)?> = None;)* }
            $(pub unsafe extern "C" fn $n($($a:$t),*) $(->$r)? {
                match hook::$n { Some(f)=>f($($a),*), None=>{ $crate::ext_dflt!([$($def)?] ($($a),*) $n) } }
            })*
            pub mod cb { pub use super::*; }
        }
    }
    hooks! {
fn GenoFly_CursorTest() -> i32 = crate::tu::adapt::GenoFly_CursorTest__be3291;
fn GenoGame_AimStick(
            sx: f32,
            sy: f32,
            facing: f32,
            grounded: i32,
            thr: f32,
            max_up: f32,
            max_down: f32,
            allow_turn: i32,
            fwd: *mut f32,
            up_0: *mut f32,
            face_0: *mut f32,
        ) -> i32 = crate::tu::adapt::GenoGame_AimStick__f25e58;
fn Geno_NoHitlag(fp: *mut Fighter) -> i32 = crate::tu::adapt::Geno_NoHitlag__37cfca;
fn Geno_HitReact(fp: *mut Fighter) -> i32 = crate::tu::adapt::Geno_HitReact__0135dc;
fn Geno_HitFlags(
            atk: *mut Fighter,
            idx: i32,
            vic: *mut Fighter,
        ) -> i32 = crate::tu::adapt::Geno_HitFlags__0264ec;
fn Geno_TakeStunBonus(vic: *mut Fighter) -> i32 = crate::tu::adapt::Geno_TakeStunBonus__1e312f;
fn Geno_HitStunBonus(
            atk: *mut Fighter,
            idx: i32,
            vic: *mut Fighter,
        ) = crate::tu::adapt::Geno_HitStunBonus__ff0e91;
fn ft_did_jump(fp: *mut Fighter, arg1: i32) -> i32 = crate::tu::adapt::ft_did_jump__3fe6b7;
fn strcmp(a0: *const i8, a1: *const i8) -> i32;
fn memcmp(a0: *const c_void, a1: *const c_void, a2: usize) -> i32;
fn memset(a0: *mut c_void, a1: i32, a2: usize) -> *mut c_void;
fn memcpy(a0: *mut c_void, a1: *const c_void, a2: usize) -> *mut c_void;
fn atan2f(a0: f32, a1: f32) -> f32 = crate::tu::adapt::atan2f__2f62eb;
fn cosf(a0: f32) -> f32 = crate::tu::adapt::cosf__b177e0;
fn fabsf(a0: f32) -> f32;
fn sinf(a0: f32) -> f32 = crate::tu::adapt::sinf__245589;
fn OSInitAlloc(a0: *mut c_void, a1: *mut c_void, a2: i32) -> *mut c_void;
fn OSCreateHeap(a0: *mut c_void, a1: *mut c_void) -> i32;
fn ftAnim_8006FA58(a0: *mut Fighter, a1: Fighter_Part, a2: *mut HSD_Joint) = crate::tu::adapt::ftAnim_8006FA58__8be30e;
fn ftAction_80073240(a0: *mut Fighter_GObj) = crate::tu::adapt::ftAction_80073240__64da7b;
fn ftAction_80073354(a0: *mut Fighter_GObj) = crate::tu::adapt::ftAction_80073354__0de40c;
fn ftAction_8007349C(a0: *mut Fighter_GObj) = crate::tu::adapt::ftAction_8007349C__18c7dc;
fn HSD_FObjInitAllocData() = crate::tu::adapt::HSD_FObjInitAllocData__fad4fb;
fn HSD_AObjInitAllocData() = crate::tu::adapt::HSD_AObjInitAllocData__a8b9a9;
fn HSD_JObjRemoveAll(a0: *mut HSD_JObj);
fn HSD_JObjLoadJoint(a0: *mut HSD_Joint) -> *mut HSD_JObj = crate::tu::adapt::HSD_JObjLoadJoint__3efee1;
fn TestRegister(a0: *const i8, a1: Option<unsafe extern "C" fn() -> i32>);
fn TestFail(a0: *const i8);
fn GenoGame_AttrFind(a0: *const i8) -> i32 = crate::tu::adapt::GenoGame_AttrFind__5b64ad;
fn GenoGame_AttrOffset(a0: i32) -> i32 = crate::tu::adapt::GenoGame_AttrOffset__be1e14;
fn GenoGame_HookFind(a0: *const i8) -> i32 = crate::tu::adapt::GenoGame_HookFind__5430c1;
fn GenoGame_StateOf(a0: *mut Fighter) -> *mut c_void = crate::tu::adapt::GenoGame_StateOf__61c016;
fn Geno_FighterReset(a0: *mut Fighter) = crate::tu::adapt::Geno_FighterReset__ca0a88;
fn Geno_OnActionChange(a0: *mut Fighter_GObj) = crate::tu::adapt::Geno_OnActionChange__e5d2dc;
fn Geno_ApplyAttrs(a0: *mut Fighter) = crate::tu::adapt::Geno_ApplyAttrs__5e5d4e;
fn Geno_MultiJump(a0: *mut Fighter, a1: i32, a2: *mut i32, a3: *mut f32) = crate::tu::adapt::Geno_MultiJump__def9ad;
fn Geno_TestReloadLayout(a0: *const i8) -> i32;
fn GenoGame_ScriptRange(a0: u32, a1: u32) -> i32 = crate::tu::adapt::GenoGame_ScriptRange__051719;
fn GenoGame_ScriptHasIasa(a0: *const c_void) -> i32 = crate::tu::adapt::GenoGame_ScriptHasIasa__c15a1a;
fn Geno_TestInstall(a0: *const i8) -> i32;
fn Geno_TestRestore();
fn snap_open(a0: i32) -> i32;
fn snap_save(a0: i32);
fn snap_load(a0: i32) -> i32;
fn GenoGame_AttrIsInt(a0: i32) -> i32 = crate::tu::adapt::GenoGame_AttrIsInt__1d3880;
fn GenoGame_TestCapture(a0: i32, a1: i32) = crate::tu::adapt::GenoGame_TestCapture__2dadd3;
fn GenoGame_TestChanges() -> i32 = crate::tu::adapt::GenoGame_TestChanges__755eb6;
fn GenoGame_TestLastTarget() -> u32 = crate::tu::adapt::GenoGame_TestLastTarget__2835dc;
fn Geno_PreAnim(a0: *mut Fighter_GObj) -> i32 = crate::tu::adapt::Geno_PreAnim__3c70fb;
fn Geno_CollBegin(a0: *mut Fighter_GObj) = crate::tu::adapt::Geno_CollBegin__155603;
fn Geno_CollEnd(a0: *mut Fighter_GObj) = crate::tu::adapt::Geno_CollEnd__6ea115;
fn Geno_GroundEdge(a0: *mut Fighter, a1: i32) = crate::tu::adapt::Geno_GroundEdge__246052;
fn Geno_Autolink(a0: *mut Fighter, a1: *mut HitCapsule, a2: *mut f32, a3: *mut f32, a4: *mut f32) -> i32 = crate::tu::adapt::Geno_Autolink__bd7888;
fn Geno_MotionRow(a0: *mut Fighter, a1: i32) -> *mut MotionState = crate::tu::adapt::Geno_MotionRow__2d0cf2;
fn Geno_SpecialEnter(a0: *mut Fighter_GObj, a1: i32) -> i32 = crate::tu::adapt::Geno_SpecialEnter__354edd;
fn GenoGame_TestLedgeMode(a0: *mut Fighter) -> i32 = crate::tu::adapt::GenoGame_TestLedgeMode__20e1e5;
fn ScriptGame_FighterI(a0: i32, a1: i32) -> i32 = crate::tu::adapt::ScriptGame_FighterI__fafc3a;
fn ScriptGame_FighterF(a0: i32, a1: i32) -> f32 = crate::tu::adapt::ScriptGame_FighterF__01b474;
fn gmFrontend_ModeSelect(a0: *mut GameModeState, a1: i32, a2: *const i8) = crate::tu::adapt::gmFrontend_ModeSelect__ce9fbe;
fn gm_GetAllGameModes() -> *mut GameMode = crate::tu::adapt::gm_GetAllGameModes__34c35a;
fn gm_GetStartMeleeRules() -> *mut StartMeleeRules = crate::tu::adapt::gm_GetStartMeleeRules__b73473;
fn gm_8016AE80() -> i32 = crate::tu::adapt::gm_8016AE80__c0d7dc;
fn SceneLaunch_BootGameMode() -> i32;
fn SceneLaunch_EntryStateId() -> i32;
fn SceneLaunch_StageExternal() -> i32;
fn SceneLaunch_ItemFreq() -> i32;
fn SceneLaunch_PlayerCKind(a0: i32) -> i32;
fn SceneLaunch_PlayerSlotType(a0: i32) -> i32;
fn it_8026D018() = crate::tu::adapt::it_8026D018__64d115;
fn it_8026D324(a0: ItemKind) -> i32 = crate::tu::adapt::it_8026D324__f6da35;
fn gm_Mode_Lab_OnLoad() = crate::tu::adapt::gm_Mode_Lab_OnLoad__64dc69;
fn GenoLab_ModeActive() -> i32 = crate::tu::adapt::GenoLab_ModeActive__8cb96b;
fn GenoLab_Leave(a0: i32) -> i32 = crate::tu::adapt::GenoLab_Leave__a89637;
fn GenoLab_ApplyRules(a0: *mut StartMeleeData) = crate::tu::adapt::GenoLab_ApplyRules__92fa88;
fn GenoLab_NextAfterMatch() -> i32 = crate::tu::adapt::GenoLab_NextAfterMatch__3887c8;
fn GenoFly_SetSpeed(a0: i32) = crate::tu::adapt::GenoFly_SetSpeed__aabcc6;
fn GenoFly_Speed() -> f32 = crate::tu::adapt::GenoFly_Speed__366efc;
fn GenoFly_SetSolid(a0: i32) = crate::tu::adapt::GenoFly_SetSolid__b04d6c;
fn SceneLaunch_LoadForTest(a0: *const i8);
fn GenoGame_StateSize() -> i32 = crate::tu::adapt::GenoGame_StateSize__92165c;
fn GenoGame_StateFieldName(a0: i32) -> *const i8 = crate::tu::adapt::GenoGame_StateFieldName__96915a;
fn GenoGame_StateFieldBase(a0: i32) -> i32 = crate::tu::adapt::GenoGame_StateFieldBase__6bd4a2;
fn ScriptGame_LabFighterFieldName(a0: i32) -> *const i8 = crate::tu::adapt::ScriptGame_LabFighterFieldName__9ca0ff;
fn ScriptGame_LabFighterFieldBase(a0: i32) -> i32 = crate::tu::adapt::ScriptGame_LabFighterFieldBase__37c01f;
fn Geno_ArticleCount(a0: i32) -> i32;
fn Geno_ArticleParam(a0: i32, a1: i32, a2: i32) -> i32;
fn Geno_ArticleHitCount(a0: i32, a1: i32) -> i32;
fn Geno_ArticleHitParam(a0: i32, a1: i32, a2: i32, a3: i32) -> i32;
fn Geno_StateCounter(a0: i32, a1: i32, a2: i32) -> i32;
fn Geno_HookCount(a0: i32, a1: i32) -> i32;
fn Geno_ArticleDesc(a0: i32) -> *mut c_void = crate::tu::adapt::Geno_ArticleDesc__533f86;
fn Geno_ArticleLogic(a0: i32) -> *mut c_void = crate::tu::adapt::Geno_ArticleLogic__7424dd;
fn Geno_OnHit(a0: *mut Fighter_GObj) -> i32 = crate::tu::adapt::Geno_OnHit__5cbd6a;
fn GenoGame_ArticleStep(a0: i32, a1: i32, a2: i32, a3: *mut f32, a4: *mut f32, a5: f32, a6: f32, a7: i32, a8: f32, a9: f32) = crate::tu::adapt::GenoGame_ArticleStep__051bee;
fn Geno_SpecialSelect(a0: i32, a1: i32, a2: i32) -> i32;
fn GenoGame_CallbackFind(a0: i32, a1: *const i8) -> i32;
fn GenoGame_LockonAim(a0: i32, a1: f32, a2: f32, a3: f32, a4: f32, a5: f32, a6: i32, a7: f32, a8: f32, a9: *mut f32, a10: *mut f32, a11: *mut f32) = crate::tu::adapt::GenoGame_LockonAim__57d893;
fn GenoFly_TestArm(a0: *mut Fighter) = crate::tu::adapt::GenoFly_TestArm__11059c;
fn GenoFly_Fighter(a0: *mut Fighter) -> i32 = crate::tu::adapt::GenoFly_Fighter__f7b536;
fn ftCo_800D3158(a0: *mut Fighter_GObj) -> i32 = crate::tu::adapt::ftCo_800D3158__f65bbf;
fn GenoGame_DefinePPCViolation(a0: i32);
fn ScriptGame_HitFamilyMove(a0: *mut Fighter) -> i32 = crate::tu::adapt::ScriptGame_HitFamilyMove__553e52;
fn Geno_DefineCount() -> i32;
fn Geno_DefineCKAt(a0: i32) -> i32;
fn GenoDefine_StateDigest(a0: *mut Fighter) -> u32 = crate::tu::adapt::GenoDefine_StateDigest__f6fea2;
fn RB_GameHash() -> u32 = crate::tu::adapt::RB_GameHash__00ac4f;
fn RB_FighterHash(a0: u32, a1: *mut Fighter, a2: i32) -> u32 = crate::tu::adapt::RB_FighterHash__1628df;
fn RB_HashSetLegacy(a0: i32) = crate::tu::adapt::RB_HashSetLegacy__8bf080;
fn RB_ItemHash(a0: *mut u32) -> u32;
fn GenoGame_LuaBlockOf(a0: *mut Fighter) -> *mut c_void = crate::tu::adapt::GenoGame_LuaBlockOf__b52deb;
fn GenoGame_ComboBlockOf(a0: *mut Fighter) -> *mut c_void = crate::tu::adapt::GenoGame_ComboBlockOf__3da5c9;
fn Geno_ComboInput(a0: *mut Fighter_GObj) = crate::tu::adapt::Geno_ComboInput__b10783;
fn Geno_ComboTick(a0: *mut Fighter_GObj) -> i32 = crate::tu::adapt::Geno_ComboTick__d39f28;
fn Geno_ComboContact(a0: *mut Fighter, a1: i32) = crate::tu::adapt::Geno_ComboContact__369dfe;
fn Geno_ComboParam(a0: i32, a1: i32) -> i32;
fn Geno_MeterParam(a0: i32, a1: i32) -> i32;
fn Geno_ComboWord(a0: i32, a1: i32, a2: i32) -> i32;
fn lbColl_80007BCC(a0: *mut HitCapsule, a1: *mut HitResult, a2: *mut c_void, a3: i32, a4: f32, a5: f32, a6: f32) -> i32 = crate::tu::adapt::lbColl_80007BCC__611461;
fn HSD_GetHeap() -> OSHeapHandle = crate::tu::adapt::HSD_GetHeap__32acf8;
fn HSD_SetHeap(a0: OSHeapHandle) = crate::tu::adapt::HSD_SetHeap__744bee;
fn HSD_IDInitAllocData() = crate::tu::adapt::HSD_IDInitAllocData__7a483c;
fn HSD_IDSetup() = crate::tu::adapt::HSD_IDSetup__f19db5;
fn ftCo_80091E78(a0: *mut Fighter_GObj, a1: f32) = crate::tu::adapt::ftCo_80091E78__aefdda;
fn HSD_VecInitAllocData() = crate::tu::adapt::HSD_VecInitAllocData__8c88b8;
fn HSD_MtxInitAllocData() = crate::tu::adapt::HSD_MtxInitAllocData__245d52;
fn HSD_RObjInitAllocData();
fn GenoBhv_SinDeg(a0: f32) -> f32;
fn GenoBhv_CosDeg(a0: f32) -> f32;
fn GenoBhv_SinqWord(a0: i32) -> u32;
fn GenoBhv_SinqCrc32() -> u32;
fn GenoBhv_GlideParams(a0: *mut GbGlideP);
fn GenoBhv_GlideEntryInit(a0: *mut GbGlideEntry);
fn GenoBhv_GlideEntryStep(a0: *mut GbGlideEntry, a1: *const GbGlideP, a2: i32, a3: i32, a4: f32, a5: f32, a6: f32, a7: i32, a8: i32, a9: i32, a10: i32) -> i32;
fn GenoBhv_GlideBegin(a0: *mut GbGlide, a1: *const GbGlideP, a2: f32, a3: f32, a4: f32, a5: f32, a6: i32, a7: i32, a8: i32);
fn GenoBhv_GlideStep(a0: *mut GbGlide, a1: *const GbGlideIn, a2: *const GbGlideP);
fn GenoBhv_TornadoParams(a0: *mut GbTornadoP);
fn GenoBhv_TornadoBegin(a0: *mut GbTornado, a1: f32, a2: f32);
fn GenoBhv_TornadoStep(a0: *mut GbTornado, a1: *const GbTornadoIn, a2: *const GbTornadoP);
fn GenoBhv_DrillParams(a0: *mut GbDrillP);
fn GenoBhv_DrillBegin(a0: *mut GbDrill, a1: *const GbDrillP, a2: f32, a3: f32);
fn GenoBhv_DrillStep(a0: *mut GbDrill, a1: *const GbDrillIn, a2: *const GbDrillP);
fn GenoBhv_CapeParams(a0: *mut GbCapeP);
fn GenoBhv_CapeBegin(a0: *mut GbCape, a1: f32, a2: f32, a3: f32);
fn GenoBhv_CapeStep(a0: *mut GbCape, a1: *const GbCapeIn, a2: *const GbCapeP);
fn GenoBhv_MultiDashParams(a0: *mut GbMultiDashP);
fn GenoBhv_MultiDashBegin(a0: *mut GbMultiDash, a1: f32);
fn GenoBhv_MultiDashStep(a0: *mut GbMultiDash, a1: *const GbMultiDashIn, a2: *const GbMultiDashP);
fn GenoBhv_AnimMotionSample(a0: *const f32, a1: i32, a2: f32, a3: *mut f32, a4: *mut f32);
fn GenoBhv_AnimMotionParams(a0: *mut GbAnimMotionP);
fn GenoBhv_AnimMotionBegin(a0: *mut GbAnimMotion, a1: f32, a2: f32);
fn GenoBhv_AnimMotionStep(a0: *mut GbAnimMotion, a1: *const f32, a2: i32, a3: *const GbAnimMotionP, a4: f32, a5: i32);
fn GenoBhv_AnimMotionFrameDelta(a0: f32, a1: f32, a2: f32, a3: *mut f32, a4: *mut f32);
fn Geno_LabEnterState(a0: *mut Fighter, a1: i32) -> i32 = crate::tu::adapt::Geno_LabEnterState__96cf9a;
fn GenoDefine_FtDataSharedMask(a0: *const ftData, a1: *const ftData) -> u32;
fn GenoDefine_FtDataFieldName(a0: i32) -> *const i8;
fn __frsqrte(a0: f64) -> f64;
}
pub use melee_runtime::tu::gobj_native::HSD_GObjPLinkHead;
pub static mut Fighter_804D6540: *mut *mut Fighter_804D6540_t = unsafe { core::mem::zeroed() };
pub use melee_fighter::hooks::FT_PARTS_TABLE as ftPartsTable;
pub use melee_runtime::shared::p_ftCommonData;
pub static mut player_slots: [StaticPlayer; 0] = unsafe { core::mem::zeroed() };
pub static mut gmVsMelee_StartData: StartMeleeData = unsafe { core::mem::zeroed() };
pub use crate::tu::geno_lab_mode::gm_Mode_Lab_States;
pub static mut GenoDefine_ContentIdentity: [[u32; 2]; 127] = unsafe { core::mem::zeroed() };
pub static mut GenoDefine_InterpreterAttempts: [u32; 127] = unsafe { core::mem::zeroed() };
pub static mut HSD_RandSeedPtr: *mut u32 = unsafe { core::mem::zeroed() };
pub static mut CostumeListsForeachCharacter: [UnkCostumeList; 127] = unsafe { core::mem::zeroed() };

pub use crate::tu::geno_game::Geno_ScriptPool;
}
use ext::*;
pub const GENO_FTCMD_OP: i32 = 59 as i32;

pub const GENO_BTN_ATTACK: i32 = 0x1 as i32;

pub const GENO_BTN_SPECIAL: i32 = 0x2 as i32;

pub const GENO_BTN_TAUNT: i32 = 0x20 as i32;

pub const GENO_TGT_KEEP_FRAME: u32 = 0x4000000 as u32;

pub const GENO_MAX_CHECKS: i32 = 8 as i32;

pub const GENO_CHECK_CONDS: i32 = 3 as i32;

pub const GENO_MODE_EXEC: i32 = 0 as i32;

pub const GENO_MODE_ANIM: i32 = 1 as i32;

pub const GENO_MODE_SKIP: i32 = 2 as i32;

pub const GENO_MAX_ATTRS: i32 = 128 as i32;

pub const GENO_MOTION_BASE: i32 = 0x400 as i32;

pub const GENO_MAX_STATES: i32 = 48 as i32;

pub const GENO_ART_KIND_BASE: i32 = 0x1000 as i32;

pub const GENO_ART_PER_RANGE: i32 = 8 as i32;

pub const GENO_ART_KIND_BASE2: i32 = GENO_ART_KIND_BASE
    + 32 as i32 * GENO_ART_PER_RANGE;

pub const GENO_ART_EXTRA_BASE: i32 = 0x20000 as i32;

pub const GENO_ART_DESPAWN_STAGE: u32 = 4 as u32;

pub const GENO_ART_DESPAWN_DEFAULT: u32 = 15 as u32;

pub const GENO_AHF_REFLECT: u32 = 4 as u32;

pub const GENO_AHF_DEFAULT: u32 = 31 as u32;

pub const T_GS: i32 = 0 as i32;

pub const GENO_BHV_SINQ_N: i32 = 1025 as i32;

pub const NULL: i32 = 0 as i32;

pub static mut t_fp: Fighter = unsafe { core::mem::zeroed() };

pub static mut t_gobj: HSD_GObj = unsafe { core::mem::zeroed() };

pub static mut t_script: [u32; 96] = unsafe { core::mem::zeroed() };

pub static mut t_mj: Fighter_x2D0_t = unsafe { core::mem::zeroed() };

pub static mut t_ftdata: ftData = unsafe { core::mem::zeroed() };

pub static mut t_ext_attr: [u32; 16] = unsafe { core::mem::zeroed() };

pub static mut t_dat_attrs: [u32; 16] = unsafe { core::mem::zeroed() };

pub static mut t_subactions: [Fighter_WaitAnimData; 8] = unsafe { core::mem::zeroed() };

pub static mut t_rows: [MotionState; 400] = unsafe { core::mem::zeroed() };

pub static mut t_v2_json: [i8; 814] = unsafe {
    ::core::mem::transmute::<
        [u8; 814],
        [i8; 814],
    >(
        *b"{\"geno\":2,\"fighters\":[{\"attach\":\"kirby\",\"states\":[{\"name\":\"GlideStart\",\"behavior\":\"geno.glide.start\",\"subaction\":10},{\"name\":\"Glide\",\"behavior\":\"geno.glide\",\"subaction\":\"motion:65\"},{\"name\":\"GlideAttack\",\"behavior\":\"geno.glide.attack\",\"subaction\":12,\"move_id\":9},{\"name\":\"GlideLanding\",\"behavior\":\"geno.glide.landing\",\"subaction\":13},{\"name\":\"GlideEnd\",\"behavior\":\"geno.glide.end\",\"subaction\":14},{\"name\":\"Tornado\",\"behavior\":\"geno.tornado\",\"subaction\":15},{\"name\":\"Drill\",\"behavior\":\"geno.drill\",\"subaction\":16},{\"name\":\"DrillEnd\",\"behavior\":\"geno.drill.end\",\"subaction\":17},{\"name\":\"Plain\",\"behavior\":\"geno.air\",\"subaction\":18,\"phys\":\"none\",\"iasa\":\"like\",\"like\":\"motion:29\",\"next\":\"geno:Glide\",\"flags\":\"0x55\"}],\"glide\":{\"entry_hold\":6},\"drill\":{\"psi_min\":-60},\"specials\":{\"n\":\"geno:Tornado\",\"s\":\"geno:Drill\"}}]}\0",
    )
};

pub static mut t_v3_json: [i8; 516] = unsafe {
    ::core::mem::transmute::<
        [u8; 516],
        [i8; 516],
    >(
        *b"{\"geno\":3,\"fighters\":[{\"attach\":\"kirby\",\"states\":[{\"name\":\"Up\",\"behavior\":\"geno.anim_motion\",\"subaction\":20,\"liftoff\":false,\"ledge\":\"front\",\"land\":\"geno:GlideEnd\"},{\"name\":\"Loop\",\"behavior\":\"geno.anim_motion\",\"subaction\":21,\"ledge\":\"both\",\"gravity\":0.5,\"origin\":true,\"landing_lag\":30},{\"name\":\"GlideStart\",\"behavior\":\"geno.glide.start\",\"subaction\":10},{\"name\":\"Glide\",\"behavior\":\"geno.glide\",\"subaction\":11},{\"name\":\"GlideEnd\",\"behavior\":\"geno.glide.end\",\"subaction\":14,\"next\":\"auto\"}],\"glide\":{\"pose_center\":90}}]}\0",
    )
};

pub static mut t_v3_many: [i8; 4096] = unsafe { core::mem::zeroed() };

pub static mut t_v5_json: [i8; 788] = unsafe {
    ::core::mem::transmute::<
        [u8; 788],
        [i8; 788],
    >(
        *b"{\"geno\":4,\"fighters\":[{\"attach\":\"kirby\",\"states\":[{\"name\":\"Counter\",\"behavior\":\"geno.ground\",\"subaction\":20,\"counter\":{\"from\":3,\"to\":10,\"target\":\"geno:CounterHit\"}},{\"name\":\"CounterHit\",\"behavior\":\"geno.ground\",\"subaction\":21},{\"name\":\"Flag\",\"behavior\":\"geno.ground\",\"subaction\":22,\"counter\":{\"negate\":false}}],\"specials\":{\"lw\":\"geno:Counter\"},\"hooks\":{\"on_hit\":[\"geno.count_frames:7\"]},\"articles\":[{\"name\":\"Shot\",\"lifetime\":40,\"velocity\":[2.5,0],\"spawn\":[6,8],\"hitboxes\":[{\"damage\":7,\"size\":2.5,\"angle\":45,\"kbg\":60,\"bkb\":20,\"element\":1,\"start\":2,\"end\":30}]},{\"name\":\"Lob\",\"velocity\":[1,2],\"gravity\":0.1,\"max_fall\":1.5,\"max_speed\":3,\"homing\":{\"turn\":5,\"delay\":3},\"despawn\":{\"stage\":false},\"max_live\":2,\"hitboxes\":[{\"damage\":3,\"hits\":{\"reflect\":false}},{\"damage\":4,\"offset\":[1,2,3]}]}]}]}\0",
    )
};

pub static mut t_v51_json: [i8; 694] = unsafe {
    ::core::mem::transmute::<
        [u8; 694],
        [i8; 694],
    >(
        *b"{\"geno\":4,\"fighters\":[{\"attach\":\"kirby\",\"states\":[{\"name\":\"Fire\",\"behavior\":\"geno.ground\",\"subaction\":20},{\"name\":\"Ice\",\"behavior\":\"geno.ground\",\"subaction\":21},{\"name\":\"Thunder\",\"behavior\":\"geno.ground\",\"subaction\":22}],\"specials\":{\"n\":{\"select\":\"la_i:0\",\"targets\":[\"geno:Fire\",\"geno:Ice\",\"geno:Thunder\"]}},\"articles\":[{\"name\":\"Ice\",\"velocity\":[3.6,0],\"accel\":-0.18,\"min_speed\":0.2,\"angle\":4,\"spawns\":[[7.6,6.8],[10,2]],\"hitboxes\":[{\"damage\":2.4,\"start\":1,\"end\":3},{\"damage\":2.4,\"slot\":0,\"start\":4,\"end\":6}]},{\"name\":\"Cloud\",\"lifetime\":30,\"bone\":2,\"effect\":1147,\"children\":[{\"article\":\"Bolt\",\"frame\":3,\"every\":3,\"count\":4,\"spawn\":1}]},{\"name\":\"Bolt\",\"velocity\":[0,-3.8],\"show_model\":true}]}]}\0",
    )
};

pub static mut tc_st: *mut TestGenoState = unsafe { core::mem::zeroed() };

pub static mut gc_state_fields: [GenoCoverField; 51] = unsafe { [
        GenoCoverField {
            name: b"profile\0" as *const u8 as *const i8,
            off: 0 as u64 as u32,
            size: ::core::mem::size_of::<i32>() as u32,
            covered: 2 as i32,
            why: b"fixed at spawn: names the kind's profile (a different profile is a different fighter, not a state change); not walked\0"
                as *const u8 as *const i8,
        },
        GenoCoverField {
            name: b"kind\0" as *const u8 as *const i8,
            off: 4 as u64 as u32,
            size: ::core::mem::size_of::<i32>() as u32,
            covered: 2 as i32,
            why: b"fixed at spawn: the kind the block was reset for; not walked\0"
                as *const u8 as *const i8,
        },
        GenoCoverField {
            name: b"flags\0" as *const u8 as *const i8,
            off: 8 as u64 as u32,
            size: ::core::mem::size_of::<u32>() as u32,
            covered: 0 as i32,
            why: b"GENO_SF_SCRIPT only marks a block a script touched; a define always has a profile, so it decides nothing\0"
                as *const u8 as *const i8,
        },
        GenoCoverField {
            name: b"resets\0" as *const u8 as *const i8,
            off: 12 as u64 as u32,
            size: ::core::mem::size_of::<u32>() as u32,
            covered: 0 as i32,
            why: b"diagnostic: spawns since boot\0" as *const u8
                as *const i8,
        },
        GenoCoverField {
            name: b"la_i\0" as *const u8 as *const i8,
            off: 16 as u64 as u32,
            size: ::core::mem::size_of::<[i32; 64]>() as u32,
            covered: 1 as i32,
            why: b"\0" as *const u8 as *const i8,
        },
        GenoCoverField {
            name: b"ra_i\0" as *const u8 as *const i8,
            off: 272 as u64 as u32,
            size: ::core::mem::size_of::<[i32; 64]>() as u32,
            covered: 1 as i32,
            why: b"\0" as *const u8 as *const i8,
        },
        GenoCoverField {
            name: b"la_f\0" as *const u8 as *const i8,
            off: 528 as u64 as u32,
            size: ::core::mem::size_of::<[f32; 64]>() as u32,
            covered: 1 as i32,
            why: b"\0" as *const u8 as *const i8,
        },
        GenoCoverField {
            name: b"ra_f\0" as *const u8 as *const i8,
            off: 784 as u64 as u32,
            size: ::core::mem::size_of::<[f32; 64]>() as u32,
            covered: 1 as i32,
            why: b"\0" as *const u8 as *const i8,
        },
        GenoCoverField {
            name: b"hook_calls\0" as *const u8 as *const i8,
            off: 1040 as u64 as u32,
            size: ::core::mem::size_of::<u32>() as u32,
            covered: 0 as i32,
            why: b"diagnostic: hooks run since the reset\0" as *const u8
                as *const i8,
        },
        GenoCoverField {
            name: b"extra_jumps\0" as *const u8 as *const i8,
            off: 1044 as u64 as u32,
            size: ::core::mem::size_of::<u32>() as u32,
            covered: 1 as i32,
            why: b"\0" as *const u8 as *const i8,
        },
        GenoCoverField {
            name: b"action_time\0" as *const u8 as *const i8,
            off: 1048 as u64 as u32,
            size: ::core::mem::size_of::<i32>() as u32,
            covered: 1 as i32,
            why: b"\0" as *const u8 as *const i8,
        },
        GenoCoverField {
            name: b"nchecks\0" as *const u8 as *const i8,
            off: 1052 as u64 as u32,
            size: ::core::mem::size_of::<u32>() as u32,
            covered: 1 as i32,
            why: b"\0" as *const u8 as *const i8,
        },
        GenoCoverField {
            name: b"last_check\0" as *const u8 as *const i8,
            off: 1056 as u64 as u32,
            size: ::core::mem::size_of::<i32>() as u32,
            covered: 1 as i32,
            why: b"found by this audit: a CHGAND after a wait extends the check registered frames earlier\0"
                as *const u8 as *const i8,
        },
        GenoCoverField {
            name: b"checks\0" as *const u8 as *const i8,
            off: 1060 as u64 as u32,
            size: ::core::mem::size_of::<[GenoCheck; 8]>() as u32,
            covered: 1 as i32,
            why: b"\0" as *const u8 as *const i8,
        },
        GenoCoverField {
            name: b"rehit_period\0" as *const u8 as *const i8,
            off: 1444 as u64 as u32,
            size: ::core::mem::size_of::<[i32; 4]>() as u32,
            covered: 1 as i32,
            why: b"\0" as *const u8 as *const i8,
        },
        GenoCoverField {
            name: b"rehit_count\0" as *const u8 as *const i8,
            off: 1460 as u64 as u32,
            size: ::core::mem::size_of::<[i32; 4]>() as u32,
            covered: 1 as i32,
            why: b"\0" as *const u8 as *const i8,
        },
        GenoCoverField {
            name: b"link_mode\0" as *const u8 as *const i8,
            off: 1476 as u64 as u32,
            size: ::core::mem::size_of::<[i32; 4]>() as u32,
            covered: 1 as i32,
            why: b"\0" as *const u8 as *const i8,
        },
        GenoCoverField {
            name: b"stun_add\0" as *const u8 as *const i8,
            off: 1492 as u64 as u32,
            size: ::core::mem::size_of::<[i32; 4]>() as u32,
            covered: 1 as i32,
            why: b"\0" as *const u8 as *const i8,
        },
        GenoCoverField {
            name: b"stun_bonus\0" as *const u8 as *const i8,
            off: 1508 as u64 as u32,
            size: ::core::mem::size_of::<i32>() as u32,
            covered: 1 as i32,
            why: b"\0" as *const u8 as *const i8,
        },
        GenoCoverField {
            name: b"stun_bonus_frame\0" as *const u8 as *const i8,
            off: 1512 as u64 as u32,
            size: ::core::mem::size_of::<i32>() as u32,
            covered: 0 as i32,
            why: b"the frame stun_bonus was set in: stun_bonus is read only in that frame (geno_stun_frame)\0"
                as *const u8 as *const i8,
        },
        GenoCoverField {
            name: b"hb_flags\0" as *const u8 as *const i8,
            off: 1516 as u64 as u32,
            size: ::core::mem::size_of::<[i32; 4]>() as u32,
            covered: 1 as i32,
            why: b"\0" as *const u8 as *const i8,
        },
        GenoCoverField {
            name: b"react_flags\0" as *const u8 as *const i8,
            off: 1532 as u64 as u32,
            size: ::core::mem::size_of::<i32>() as u32,
            covered: 0 as i32,
            why: b"set and read inside one frame's hit processing, keyed to react_frame\0"
                as *const u8 as *const i8,
        },
        GenoCoverField {
            name: b"react_frame\0" as *const u8 as *const i8,
            off: 1536 as u64 as u32,
            size: ::core::mem::size_of::<i32>() as u32,
            covered: 0 as i32,
            why: b"the frame key of react_flags\0" as *const u8
                as *const i8,
        },
        GenoCoverField {
            name: b"in_coll\0" as *const u8 as *const i8,
            off: 1540 as u64 as u32,
            size: ::core::mem::size_of::<u32>() as u32,
            covered: 0 as i32,
            why: b"inside the collision callback only; zero at the start of a logic frame, where the hash is read\0"
                as *const u8 as *const i8,
        },
        GenoCoverField {
            name: b"edge_pending\0" as *const u8 as *const i8,
            off: 1544 as u64 as u32,
            size: ::core::mem::size_of::<u32>() as u32,
            covered: 0 as i32,
            why: b"as in_coll\0" as *const u8 as *const i8,
        },
        GenoCoverField {
            name: b"edge_target\0" as *const u8 as *const i8,
            off: 1548 as u64 as u32,
            size: ::core::mem::size_of::<u32>() as u32,
            covered: 0 as i32,
            why: b"as in_coll\0" as *const u8 as *const i8,
        },
        GenoCoverField {
            name: b"changes\0" as *const u8 as *const i8,
            off: 1552 as u64 as u32,
            size: ::core::mem::size_of::<u32>() as u32,
            covered: 0 as i32,
            why: b"diagnostic: Geno-made action changes since the reset\0" as *const u8
                as *const i8,
        },
        GenoCoverField {
            name: b"hold_motion\0" as *const u8 as *const i8,
            off: 1556 as u64 as u32,
            size: ::core::mem::size_of::<i32>() as u32,
            covered: 1 as i32,
            why: b"\0" as *const u8 as *const i8,
        },
        GenoCoverField {
            name: b"hold_frames\0" as *const u8 as *const i8,
            off: 1560 as u64 as u32,
            size: ::core::mem::size_of::<i32>() as u32,
            covered: 1 as i32,
            why: b"\0" as *const u8 as *const i8,
        },
        GenoCoverField {
            name: b"move_i\0" as *const u8 as *const i8,
            off: 1564 as u64 as u32,
            size: ::core::mem::size_of::<[i32; 8]>() as u32,
            covered: 1 as i32,
            why: b"\0" as *const u8 as *const i8,
        },
        GenoCoverField {
            name: b"move_f\0" as *const u8 as *const i8,
            off: 1596 as u64 as u32,
            size: ::core::mem::size_of::<[f32; 8]>() as u32,
            covered: 1 as i32,
            why: b"\0" as *const u8 as *const i8,
        },
        GenoCoverField {
            name: b"state_entries\0" as *const u8 as *const i8,
            off: 1628 as u64 as u32,
            size: ::core::mem::size_of::<u32>() as u32,
            covered: 0 as i32,
            why: b"diagnostic: Geno states entered since the reset\0" as *const u8
                as *const i8,
        },
        GenoCoverField {
            name: b"enter_from\0" as *const u8 as *const i8,
            off: 1632 as u64 as u32,
            size: ::core::mem::size_of::<i32>() as u32,
            covered: 1 as i32,
            why: b"\0" as *const u8 as *const i8,
        },
        GenoCoverField {
            name: b"hidden\0" as *const u8 as *const i8,
            off: 1636 as u64 as u32,
            size: ::core::mem::size_of::<i32>() as u32,
            covered: 1 as i32,
            why: b"\0" as *const u8 as *const i8,
        },
        GenoCoverField {
            name: b"hit_count\0" as *const u8 as *const i8,
            off: 1640 as u64 as u32,
            size: ::core::mem::size_of::<i32>() as u32,
            covered: 1 as i32,
            why: b"\0" as *const u8 as *const i8,
        },
        GenoCoverField {
            name: b"hit_damage\0" as *const u8 as *const i8,
            off: 1644 as u64 as u32,
            size: ::core::mem::size_of::<f32>() as u32,
            covered: 1 as i32,
            why: b"\0" as *const u8 as *const i8,
        },
        GenoCoverField {
            name: b"hit_port\0" as *const u8 as *const i8,
            off: 1648 as u64 as u32,
            size: ::core::mem::size_of::<i32>() as u32,
            covered: 1 as i32,
            why: b"\0" as *const u8 as *const i8,
        },
        GenoCoverField {
            name: b"hit_counter\0" as *const u8 as *const i8,
            off: 1652 as u64 as u32,
            size: ::core::mem::size_of::<i32>() as u32,
            covered: 1 as i32,
            why: b"\0" as *const u8 as *const i8,
        },
        GenoCoverField {
            name: b"counters\0" as *const u8 as *const i8,
            off: 1656 as u64 as u32,
            size: ::core::mem::size_of::<i32>() as u32,
            covered: 1 as i32,
            why: b"found by this audit after slice 5b: fighter Lua reads it (ctx.self.countered)\0"
                as *const u8 as *const i8,
        },
        GenoCoverField {
            name: b"art_spawned\0" as *const u8 as *const i8,
            off: 1660 as u64 as u32,
            size: ::core::mem::size_of::<i32>() as u32,
            covered: 0 as i32,
            why: b"diagnostic: articles spawned since the reset (the live articles themselves are hashed)\0"
                as *const u8 as *const i8,
        },
        GenoCoverField {
            name: b"fall_ledge\0" as *const u8 as *const i8,
            off: 1664 as u64 as u32,
            size: ::core::mem::size_of::<i32>() as u32,
            covered: 1 as i32,
            why: b"ctx.ledge(3): decides the FallSpecial ledge catch\0" as *const u8
                as *const i8,
        },
        GenoCoverField {
            name: b"ledge\0" as *const u8 as *const i8,
            off: 1668 as u64 as u32,
            size: ::core::mem::size_of::<i32>() as u32,
            covered: 1 as i32,
            why: b"\0" as *const u8 as *const i8,
        },
        GenoCoverField {
            name: b"motion_started\0" as *const u8 as *const i8,
            off: 1672 as u64 as u32,
            size: ::core::mem::size_of::<i32>() as u32,
            covered: 1 as i32,
            why: b"\0" as *const u8 as *const i8,
        },
        GenoCoverField {
            name: b"motion_vy\0" as *const u8 as *const i8,
            off: 1676 as u64 as u32,
            size: ::core::mem::size_of::<f32>() as u32,
            covered: 1 as i32,
            why: b"\0" as *const u8 as *const i8,
        },
        GenoCoverField {
            name: b"motion_land\0" as *const u8 as *const i8,
            off: 1680 as u64 as u32,
            size: ::core::mem::size_of::<i32>() as u32,
            covered: 0 as i32,
            why: b"diagnostic: frames the root motion has been grounded\0" as *const u8
                as *const i8,
        },
        GenoCoverField {
            name: b"motion_facing\0" as *const u8 as *const i8,
            off: 1684 as u64 as u32,
            size: ::core::mem::size_of::<f32>() as u32,
            covered: 1 as i32,
            why: b"\0" as *const u8 as *const i8,
        },
        GenoCoverField {
            name: b"motion_gravity\0" as *const u8 as *const i8,
            off: 1688 as u64 as u32,
            size: ::core::mem::size_of::<f32>() as u32,
            covered: 1 as i32,
            why: b"\0" as *const u8 as *const i8,
        },
        GenoCoverField {
            name: b"enter_keep\0" as *const u8 as *const i8,
            off: 1692 as u64 as u32,
            size: ::core::mem::size_of::<i32>() as u32,
            covered: 1 as i32,
            why: b"\0" as *const u8 as *const i8,
        },
        GenoCoverField {
            name: b"atk_connected\0" as *const u8 as *const i8,
            off: 1696 as u64 as u32,
            size: ::core::mem::size_of::<i32>() as u32,
            covered: 1 as i32,
            why: b"\0" as *const u8 as *const i8,
        },
        GenoCoverField {
            name: b"atk_connected_prev\0" as *const u8 as *const i8,
            off: 1700 as u64 as u32,
            size: ::core::mem::size_of::<i32>() as u32,
            covered: 1 as i32,
            why: b"\0" as *const u8 as *const i8,
        },
        GenoCoverField {
            name: b"fall_limit\0" as *const u8 as *const i8,
            off: 1704 as u64 as u32,
            size: ::core::mem::size_of::<f32>() as u32,
            covered: 1 as i32,
            why: b"\0" as *const u8 as *const i8,
        },
    ] };

pub static mut gc_lua_fields: [GenoCoverField; 3] = unsafe { [
        GenoCoverField {
            name: b"slot\0" as *const u8 as *const i8,
            off: 0 as u64 as u32,
            size: ::core::mem::size_of::<[i32; 16]>() as u32,
            covered: 1 as i32,
            why: b"\0" as *const u8 as *const i8,
        },
        GenoCoverField {
            name: b"faults\0" as *const u8 as *const i8,
            off: 64 as u64 as u32,
            size: ::core::mem::size_of::<u32>() as u32,
            covered: 1 as i32,
            why: b"\0" as *const u8 as *const i8,
        },
        GenoCoverField {
            name: b"last_fault\0" as *const u8 as *const i8,
            off: 68 as u64 as u32,
            size: ::core::mem::size_of::<i32>() as u32,
            covered: 1 as i32,
            why: b"\0" as *const u8 as *const i8,
        },
    ] };

pub static mut gba_rc: i32 = unsafe { core::mem::zeroed() };

pub static mut gbt_json: [i8; 501] = unsafe {
    ::core::mem::transmute::<
        [u8; 501],
        [i8; 501],
    >(
        *b"{\"geno\":3,\"fighters\":[{\"attach\":\"kirby\",\"states\":[{\"name\":\"Tor\",\"behavior\":\"geno.tornado\",\"subaction\":20},{\"name\":\"Dr\",\"behavior\":\"geno.drill\",\"subaction\":21},{\"name\":\"Cp\",\"behavior\":\"geno.cape\",\"subaction\":22},{\"name\":\"GS\",\"behavior\":\"geno.glide.start\",\"subaction\":10},{\"name\":\"G\",\"behavior\":\"geno.glide\",\"subaction\":11},{\"name\":\"GA\",\"behavior\":\"geno.glide.attack\",\"subaction\":12},{\"name\":\"GE\",\"behavior\":\"geno.glide.end\",\"subaction\":14},{\"name\":\"MD\",\"behavior\":\"geno.multi_dash\",\"subaction\":23}]}]}\0",
    )
};

pub unsafe extern "C" fn HSD_JObjGetChild(mut jobj: *mut HSD_JObj) -> *mut HSD_JObj {
    melee_core::ct!("HSD_JObjGetChild");
    if jobj.is_null() {
        return ::core::ptr::null_mut::<HSD_JObj>()
    } else {
        return (*jobj).child as *mut HSD_JObj
    };
}

pub unsafe extern "C" fn t_zero(mut p: *mut c_void, mut n: i32) {
    melee_core::ct!("t_zero");
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

pub unsafe extern "C" fn t_setup() -> *mut TestGenoState {
    melee_core::ct!("t_setup");
    t_zero(
        &raw mut t_fp as *mut c_void,
        ::core::mem::size_of::<Fighter>() as i32,
    );
    t_zero(
        &raw mut t_gobj as *mut c_void,
        ::core::mem::size_of::<HSD_GObj>() as i32,
    );
    t_gobj.user_data = &raw mut t_fp as *mut c_void;
    t_fp.gobj = &raw mut t_gobj;
    t_fp.kind = Ft_Kind_Kirby;
    t_fp.player_id = 5 as u8;
    t_fp.set_is_sub_fighter(1 as u8 as u8);
    t_fp.frame_speed_mul = 1.0f32;
    t_fp.ft_data = &raw mut t_ftdata as *mut ftData;
    t_fp.dat_attrs = &raw mut t_dat_attrs as *mut u32 as *mut c_void;
    t_fp.dat_attrs_backup = &raw mut t_dat_attrs as *mut u32
        as *mut c_void;
    t_fp.x18 = 341 as i32 as i32;
    t_fp.x1C_actionStateList = &raw mut t_rows as *mut MotionState;
    t_fp.x20_actionStateList = &raw mut t_rows as *mut MotionState;
    t_fp.facing_dir = 1.0f32;
    t_ftdata.ext_attr = &raw mut t_ext_attr as *mut u32 as *mut c_void;
    t_ftdata.xC = &raw mut t_subactions as *mut Fighter_WaitAnimData;
    Geno_FighterReset(&raw mut t_fp);
    return GenoGame_StateOf(&raw mut t_fp) as *mut TestGenoState;
}

pub unsafe extern "C" fn t_run(mut script: *mut u32, mut loop_0: i32) {
    melee_core::ct!("t_run");
    t_fp.x3E4_fighterCmdScript.anon_8.u = script as *mut CmdUnion;
    t_fp.x3E4_fighterCmdScript.timer = 0.0f32 as f32;
    t_fp.x3E4_fighterCmdScript.loop_count = 0 as u32;
    if loop_0 == GENO_MODE_SKIP {
        ftAction_8007349C(&raw mut t_gobj);
    } else if loop_0 == GENO_MODE_ANIM {
        ftAction_80073354(&raw mut t_gobj);
    } else {
        ftAction_80073240(&raw mut t_gobj);
    };
}

pub unsafe extern "C" fn t_fbits(mut f: f32) -> u32 {
    melee_core::ct!("t_fbits");
    let mut x: Anon_anon_109 = unsafe { core::mem::zeroed() };
    x.f = f as f32;
    return x.u;
}

pub unsafe extern "C" fn t_build(mut s: *mut u32) -> i32 {
    melee_core::ct!("t_build");
    let mut n: i32 = 0 as i32;
    let fresh2 = n;
    n = n + 1;
    *s.offset(fresh2 as isize) = (((GENO_FTCMD_OP << 26 as i32)
        as u32
        | (GENO_SUB_SET as i32 as u32
            & 63 as u32) << 20 as i32
        | (2 as i32 as u32 & 15 as u32)
            << 16 as i32
        | ((((0 as i32 & 3 as i32)
            << 6 as i32
            | 0 as i32 & 63 as i32)
            & 0xff as i32) << 8 as i32
            | (if 0 as i32 != 0 {
                0x80 as i32
            } else {
                0 as i32
            })
            | (0 as i32 & 7 as i32)
                << 4 as i32) as u32
            & 0xffff as u32) as u32).to_be();
    let fresh3 = n;
    n = n + 1;
    *s.offset(fresh3 as isize) = (5 as u32).to_be();
    let fresh4 = n;
    n = n + 1;
    *s.offset(fresh4 as isize) = (((GENO_FTCMD_OP << 26 as i32)
        as u32
        | (GENO_SUB_IF as i32 as u32
            & 63 as u32) << 20 as i32
        | (3 as i32 as u32 & 15 as u32)
            << 16 as i32
        | ((((0 as i32 & 3 as i32)
            << 6 as i32
            | 0 as i32 & 63 as i32)
            & 0xff as i32) << 8 as i32
            | (if 0 as i32 != 0 {
                0x80 as i32
            } else {
                0 as i32
            })
            | (GENO_CMP_EQ as i32 & 7 as i32)
                << 4 as i32) as u32
            & 0xffff as u32) as u32).to_be();
    let fresh5 = n;
    n = n + 1;
    *s.offset(fresh5 as isize) = (5 as u32).to_be();
    let fresh6 = n;
    n = n + 1;
    *s.offset(fresh6 as isize) = (5 as u32).to_be();
    let fresh7 = n;
    n = n + 1;
    *s.offset(fresh7 as isize) = (((GENO_FTCMD_OP << 26 as i32)
        as u32
        | (GENO_SUB_SET as i32 as u32
            & 63 as u32) << 20 as i32
        | (2 as i32 as u32 & 15 as u32)
            << 16 as i32
        | ((((0 as i32 & 3 as i32)
            << 6 as i32
            | 1 as i32 & 63 as i32)
            & 0xff as i32) << 8 as i32
            | (if 0 as i32 != 0 {
                0x80 as i32
            } else {
                0 as i32
            })
            | (0 as i32 & 7 as i32)
                << 4 as i32) as u32
            & 0xffff as u32) as u32).to_be();
    let fresh8 = n;
    n = n + 1;
    *s.offset(fresh8 as isize) = (111 as u32).to_be();
    let fresh9 = n;
    n = n + 1;
    *s.offset(fresh9 as isize) = ((19 as u32) << 26 as i32
        | (1 as i32 as u32) << 24 as i32
        | 42 as i32 as u32).to_be();
    let fresh10 = n;
    n = n + 1;
    *s.offset(fresh10 as isize) = (((GENO_FTCMD_OP << 26 as i32)
        as u32
        | (GENO_SUB_SKIP as i32 as u32
            & 63 as u32) << 20 as i32
        | (2 as i32 as u32 & 15 as u32)
            << 16 as i32
        | 0 as i32 as u32 & 0xffff as u32)
        as u32).to_be();
    let fresh11 = n;
    n = n + 1;
    *s.offset(fresh11 as isize) = (3 as u32).to_be();
    let fresh12 = n;
    n = n + 1;
    *s.offset(fresh12 as isize) = (((GENO_FTCMD_OP << 26 as i32)
        as u32
        | (GENO_SUB_SET as i32 as u32
            & 63 as u32) << 20 as i32
        | (2 as i32 as u32 & 15 as u32)
            << 16 as i32
        | ((((0 as i32 & 3 as i32)
            << 6 as i32
            | 1 as i32 & 63 as i32)
            & 0xff as i32) << 8 as i32
            | (if 0 as i32 != 0 {
                0x80 as i32
            } else {
                0 as i32
            })
            | (0 as i32 & 7 as i32)
                << 4 as i32) as u32
            & 0xffff as u32) as u32).to_be();
    let fresh13 = n;
    n = n + 1;
    *s.offset(fresh13 as isize) = (222 as u32).to_be();
    let fresh14 = n;
    n = n + 1;
    *s.offset(fresh14 as isize) = ((19 as u32) << 26 as i32
        | (1 as i32 as u32) << 24 as i32
        | 99 as i32 as u32).to_be();
    let fresh15 = n;
    n = n + 1;
    *s.offset(fresh15 as isize) = (((GENO_FTCMD_OP << 26 as i32)
        as u32
        | (GENO_SUB_IF as i32 as u32
            & 63 as u32) << 20 as i32
        | (3 as i32 as u32 & 15 as u32)
            << 16 as i32
        | ((((0 as i32 & 3 as i32)
            << 6 as i32
            | 0 as i32 & 63 as i32)
            & 0xff as i32) << 8 as i32
            | (if 0 as i32 != 0 {
                0x80 as i32
            } else {
                0 as i32
            })
            | (GENO_CMP_NE as i32 & 7 as i32)
                << 4 as i32) as u32
            & 0xffff as u32) as u32).to_be();
    let fresh16 = n;
    n = n + 1;
    *s.offset(fresh16 as isize) = (5 as u32).to_be();
    let fresh17 = n;
    n = n + 1;
    *s.offset(fresh17 as isize) = (4 as u32).to_be();
    let fresh18 = n;
    n = n + 1;
    *s.offset(fresh18 as isize) = (((GENO_FTCMD_OP << 26 as i32)
        as u32
        | (GENO_SUB_SET as i32 as u32
            & 63 as u32) << 20 as i32
        | (2 as i32 as u32 & 15 as u32)
            << 16 as i32
        | ((((0 as i32 & 3 as i32)
            << 6 as i32
            | 2 as i32 & 63 as i32)
            & 0xff as i32) << 8 as i32
            | (if 0 as i32 != 0 {
                0x80 as i32
            } else {
                0 as i32
            })
            | (0 as i32 & 7 as i32)
                << 4 as i32) as u32
            & 0xffff as u32) as u32).to_be();
    let fresh19 = n;
    n = n + 1;
    *s.offset(fresh19 as isize) = (1 as u32).to_be();
    let fresh20 = n;
    n = n + 1;
    *s.offset(fresh20 as isize) = (((GENO_FTCMD_OP << 26 as i32)
        as u32
        | (GENO_SUB_SKIP as i32 as u32
            & 63 as u32) << 20 as i32
        | (2 as i32 as u32 & 15 as u32)
            << 16 as i32
        | 0 as i32 as u32 & 0xffff as u32)
        as u32).to_be();
    let fresh21 = n;
    n = n + 1;
    *s.offset(fresh21 as isize) = (2 as u32).to_be();
    let fresh22 = n;
    n = n + 1;
    *s.offset(fresh22 as isize) = (((GENO_FTCMD_OP << 26 as i32)
        as u32
        | (GENO_SUB_SET as i32 as u32
            & 63 as u32) << 20 as i32
        | (2 as i32 as u32 & 15 as u32)
            << 16 as i32
        | ((((0 as i32 & 3 as i32)
            << 6 as i32
            | 2 as i32 & 63 as i32)
            & 0xff as i32) << 8 as i32
            | (if 0 as i32 != 0 {
                0x80 as i32
            } else {
                0 as i32
            })
            | (0 as i32 & 7 as i32)
                << 4 as i32) as u32
            & 0xffff as u32) as u32).to_be();
    let fresh23 = n;
    n = n + 1;
    *s.offset(fresh23 as isize) = (2 as u32).to_be();
    let fresh24 = n;
    n = n + 1;
    *s.offset(fresh24 as isize) = (((GENO_FTCMD_OP << 26 as i32)
        as u32
        | (GENO_SUB_ADD as i32 as u32
            & 63 as u32) << 20 as i32
        | (2 as i32 as u32 & 15 as u32)
            << 16 as i32
        | ((((0 as i32 & 3 as i32)
            << 6 as i32
            | 0 as i32 & 63 as i32)
            & 0xff as i32) << 8 as i32
            | (if 0 as i32 != 0 {
                0x80 as i32
            } else {
                0 as i32
            })
            | (0 as i32 & 7 as i32)
                << 4 as i32) as u32
            & 0xffff as u32) as u32).to_be();
    let fresh25 = n;
    n = n + 1;
    *s.offset(fresh25 as isize) = (10 as u32).to_be();
    let fresh26 = n;
    n = n + 1;
    *s.offset(fresh26 as isize) = (((GENO_FTCMD_OP << 26 as i32)
        as u32
        | (GENO_SUB_SUB as i32 as u32
            & 63 as u32) << 20 as i32
        | (2 as i32 as u32 & 15 as u32)
            << 16 as i32
        | ((((0 as i32 & 3 as i32)
            << 6 as i32
            | 0 as i32 & 63 as i32)
            & 0xff as i32) << 8 as i32
            | (if 0 as i32 != 0 {
                0x80 as i32
            } else {
                0 as i32
            })
            | (0 as i32 & 7 as i32)
                << 4 as i32) as u32
            & 0xffff as u32) as u32).to_be();
    let fresh27 = n;
    n = n + 1;
    *s.offset(fresh27 as isize) = (1 as u32).to_be();
    let fresh28 = n;
    n = n + 1;
    *s.offset(fresh28 as isize) = (((GENO_FTCMD_OP << 26 as i32)
        as u32
        | (GENO_SUB_MUL as i32 as u32
            & 63 as u32) << 20 as i32
        | (2 as i32 as u32 & 15 as u32)
            << 16 as i32
        | ((((0 as i32 & 3 as i32)
            << 6 as i32
            | 0 as i32 & 63 as i32)
            & 0xff as i32) << 8 as i32
            | (if 0 as i32 != 0 {
                0x80 as i32
            } else {
                0 as i32
            })
            | (0 as i32 & 7 as i32)
                << 4 as i32) as u32
            & 0xffff as u32) as u32).to_be();
    let fresh29 = n;
    n = n + 1;
    *s.offset(fresh29 as isize) = (3 as u32).to_be();
    let fresh30 = n;
    n = n + 1;
    *s.offset(fresh30 as isize) = (((GENO_FTCMD_OP << 26 as i32)
        as u32
        | (GENO_SUB_SET as i32 as u32
            & 63 as u32) << 20 as i32
        | (2 as i32 as u32 & 15 as u32)
            << 16 as i32
        | ((((2 as i32 & 3 as i32)
            << 6 as i32
            | 0 as i32 & 63 as i32)
            & 0xff as i32) << 8 as i32
            | (if 0 as i32 != 0 {
                0x80 as i32
            } else {
                0 as i32
            })
            | (0 as i32 & 7 as i32)
                << 4 as i32) as u32
            & 0xffff as u32) as u32).to_be();
    let fresh31 = n;
    n = n + 1;
    *s.offset(fresh31 as isize) = (t_fbits(1.5f32)).to_be();
    let fresh32 = n;
    n = n + 1;
    *s.offset(fresh32 as isize) = (((GENO_FTCMD_OP << 26 as i32)
        as u32
        | (GENO_SUB_MUL as i32 as u32
            & 63 as u32) << 20 as i32
        | (2 as i32 as u32 & 15 as u32)
            << 16 as i32
        | ((((2 as i32 & 3 as i32)
            << 6 as i32
            | 0 as i32 & 63 as i32)
            & 0xff as i32) << 8 as i32
            | (if 0 as i32 != 0 {
                0x80 as i32
            } else {
                0 as i32
            })
            | (0 as i32 & 7 as i32)
                << 4 as i32) as u32
            & 0xffff as u32) as u32).to_be();
    let fresh33 = n;
    n = n + 1;
    *s.offset(fresh33 as isize) = (t_fbits(2.0f32)).to_be();
    let fresh34 = n;
    n = n + 1;
    *s.offset(fresh34 as isize) = (((GENO_FTCMD_OP << 26 as i32)
        as u32
        | (GENO_SUB_IF as i32 as u32
            & 63 as u32) << 20 as i32
        | (3 as i32 as u32 & 15 as u32)
            << 16 as i32
        | ((((2 as i32 & 3 as i32)
            << 6 as i32
            | 0 as i32 & 63 as i32)
            & 0xff as i32) << 8 as i32
            | (if 0 as i32 != 0 {
                0x80 as i32
            } else {
                0 as i32
            })
            | (GENO_CMP_GT as i32 & 7 as i32)
                << 4 as i32) as u32
            & 0xffff as u32) as u32).to_be();
    let fresh35 = n;
    n = n + 1;
    *s.offset(fresh35 as isize) = (t_fbits(2.5f32)).to_be();
    let fresh36 = n;
    n = n + 1;
    *s.offset(fresh36 as isize) = (2 as u32).to_be();
    let fresh37 = n;
    n = n + 1;
    *s.offset(fresh37 as isize) = (((GENO_FTCMD_OP << 26 as i32)
        as u32
        | (GENO_SUB_SET as i32 as u32
            & 63 as u32) << 20 as i32
        | (2 as i32 as u32 & 15 as u32)
            << 16 as i32
        | ((((0 as i32 & 3 as i32)
            << 6 as i32
            | 3 as i32 & 63 as i32)
            & 0xff as i32) << 8 as i32
            | (if 0 as i32 != 0 {
                0x80 as i32
            } else {
                0 as i32
            })
            | (0 as i32 & 7 as i32)
                << 4 as i32) as u32
            & 0xffff as u32) as u32).to_be();
    let fresh38 = n;
    n = n + 1;
    *s.offset(fresh38 as isize) = (7 as u32).to_be();
    let fresh39 = n;
    n = n + 1;
    *s.offset(fresh39 as isize) = (((GENO_FTCMD_OP << 26 as i32)
        as u32
        | (GENO_SUB_SETBIT as i32 as u32
            & 63 as u32) << 20 as i32
        | (2 as i32 as u32 & 15 as u32)
            << 16 as i32
        | ((((0 as i32 & 3 as i32)
            << 6 as i32
            | 4 as i32 & 63 as i32)
            & 0xff as i32) << 8 as i32
            | (if 0 as i32 != 0 {
                0x80 as i32
            } else {
                0 as i32
            })
            | (0 as i32 & 7 as i32)
                << 4 as i32) as u32
            & 0xffff as u32) as u32).to_be();
    let fresh40 = n;
    n = n + 1;
    *s.offset(fresh40 as isize) = (3 as u32).to_be();
    let fresh41 = n;
    n = n + 1;
    *s.offset(fresh41 as isize) = (((GENO_FTCMD_OP << 26 as i32)
        as u32
        | (GENO_SUB_SETBIT as i32 as u32
            & 63 as u32) << 20 as i32
        | (2 as i32 as u32 & 15 as u32)
            << 16 as i32
        | ((((0 as i32 & 3 as i32)
            << 6 as i32
            | 4 as i32 & 63 as i32)
            & 0xff as i32) << 8 as i32
            | (if 0 as i32 != 0 {
                0x80 as i32
            } else {
                0 as i32
            })
            | (0 as i32 & 7 as i32)
                << 4 as i32) as u32
            & 0xffff as u32) as u32).to_be();
    let fresh42 = n;
    n = n + 1;
    *s.offset(fresh42 as isize) = (0 as u32).to_be();
    let fresh43 = n;
    n = n + 1;
    *s.offset(fresh43 as isize) = (((GENO_FTCMD_OP << 26 as i32)
        as u32
        | (GENO_SUB_CLRBIT as i32 as u32
            & 63 as u32) << 20 as i32
        | (2 as i32 as u32 & 15 as u32)
            << 16 as i32
        | ((((0 as i32 & 3 as i32)
            << 6 as i32
            | 4 as i32 & 63 as i32)
            & 0xff as i32) << 8 as i32
            | (if 0 as i32 != 0 {
                0x80 as i32
            } else {
                0 as i32
            })
            | (0 as i32 & 7 as i32)
                << 4 as i32) as u32
            & 0xffff as u32) as u32).to_be();
    let fresh44 = n;
    n = n + 1;
    *s.offset(fresh44 as isize) = (0 as u32).to_be();
    let fresh45 = n;
    n = n + 1;
    *s.offset(fresh45 as isize) = (((GENO_FTCMD_OP << 26 as i32)
        as u32
        | (GENO_SUB_IF as i32 as u32
            & 63 as u32) << 20 as i32
        | (3 as i32 as u32 & 15 as u32)
            << 16 as i32
        | ((((0 as i32 & 3 as i32)
            << 6 as i32
            | 4 as i32 & 63 as i32)
            & 0xff as i32) << 8 as i32
            | (if 0 as i32 != 0 {
                0x80 as i32
            } else {
                0 as i32
            })
            | (GENO_CMP_BIT as i32 & 7 as i32)
                << 4 as i32) as u32
            & 0xffff as u32) as u32).to_be();
    let fresh46 = n;
    n = n + 1;
    *s.offset(fresh46 as isize) = (3 as u32).to_be();
    let fresh47 = n;
    n = n + 1;
    *s.offset(fresh47 as isize) = (2 as u32).to_be();
    let fresh48 = n;
    n = n + 1;
    *s.offset(fresh48 as isize) = (((GENO_FTCMD_OP << 26 as i32)
        as u32
        | (GENO_SUB_SET as i32 as u32
            & 63 as u32) << 20 as i32
        | (2 as i32 as u32 & 15 as u32)
            << 16 as i32
        | ((((0 as i32 & 3 as i32)
            << 6 as i32
            | 5 as i32 & 63 as i32)
            & 0xff as i32) << 8 as i32
            | (if 0 as i32 != 0 {
                0x80 as i32
            } else {
                0 as i32
            })
            | (0 as i32 & 7 as i32)
                << 4 as i32) as u32
            & 0xffff as u32) as u32).to_be();
    let fresh49 = n;
    n = n + 1;
    *s.offset(fresh49 as isize) = (1 as u32).to_be();
    let fresh50 = n;
    n = n + 1;
    *s.offset(fresh50 as isize) = (((GENO_FTCMD_OP << 26 as i32)
        as u32
        | (GENO_SUB_SET as i32 as u32
            & 63 as u32) << 20 as i32
        | (2 as i32 as u32 & 15 as u32)
            << 16 as i32
        | ((((1 as i32 & 3 as i32)
            << 6 as i32
            | 0 as i32 & 63 as i32)
            & 0xff as i32) << 8 as i32
            | (if 1 as i32 != 0 {
                0x80 as i32
            } else {
                0 as i32
            })
            | (0 as i32 & 7 as i32)
                << 4 as i32) as u32
            & 0xffff as u32) as u32).to_be();
    let fresh51 = n;
    n = n + 1;
    *s.offset(fresh51 as isize) = (((0 as i32 & 3 as i32)
        << 6 as i32 | 0 as i32 & 63 as i32)
        as u32).to_be();
    let fresh52 = n;
    n = n + 1;
    *s.offset(fresh52 as isize) = (((GENO_FTCMD_OP << 26 as i32)
        as u32
        | (GENO_SUB_SET as i32 as u32
            & 63 as u32) << 20 as i32
        | (2 as i32 as u32 & 15 as u32)
            << 16 as i32
        | ((((0 as i32 & 3 as i32)
            << 6 as i32
            | 6 as i32 & 63 as i32)
            & 0xff as i32) << 8 as i32
            | (if 1 as i32 != 0 {
                0x80 as i32
            } else {
                0 as i32
            })
            | (0 as i32 & 7 as i32)
                << 4 as i32) as u32
            & 0xffff as u32) as u32).to_be();
    let fresh53 = n;
    n = n + 1;
    *s.offset(fresh53 as isize) = (((2 as i32 & 3 as i32)
        << 6 as i32 | 0 as i32 & 63 as i32)
        as u32).to_be();
    let fresh54 = n;
    n = n + 1;
    *s.offset(fresh54 as isize) = (((GENO_FTCMD_OP << 26 as i32)
        as u32
        | (GENO_SUB_CALL as i32 as u32
            & 63 as u32) << 20 as i32
        | (3 as i32 as u32 & 15 as u32)
            << 16 as i32
        | 0 as i32 as u32 & 0xffff as u32)
        as u32).to_be();
    let fresh55 = n;
    n = n + 1;
    *s.offset(fresh55 as isize) = (GENO_HOOK_COUNT_FRAMES as i32 as u32).to_be();
    let fresh56 = n;
    n = n + 1;
    *s.offset(fresh56 as isize) = (7 as u32).to_be();
    let fresh57 = n;
    n = n + 1;
    *s.offset(fresh57 as isize) = (((GENO_FTCMD_OP << 26 as i32)
        as u32
        | (0x3e as i32 as u32 & 63 as u32)
            << 20 as i32
        | (3 as i32 as u32 & 15 as u32)
            << 16 as i32
        | 0 as i32 as u32 & 0xffff as u32)
        as u32).to_be();
    let fresh58 = n;
    n = n + 1;
    *s.offset(fresh58 as isize) = (0xffffffff as u32 as u32).to_be();
    let fresh59 = n;
    n = n + 1;
    *s.offset(fresh59 as isize) = (0xffffffff as u32 as u32).to_be();
    let fresh60 = n;
    n = n + 1;
    *s.offset(fresh60 as isize) = ((19 as u32) << 26 as i32
        | (2 as i32 as u32) << 24 as i32
        | 7 as i32 as u32).to_be();
    let fresh61 = n;
    n = n + 1;
    *s.offset(fresh61 as isize) = (0 as u32).to_be();
    return n;
}

pub unsafe extern "C" fn test_geno_ftcmd_escape() -> i32 {
    melee_core::ct!("test_geno_ftcmd_escape");
    let mut st: *mut TestGenoState = t_setup();
    t_build(&raw mut t_script as *mut u32);
    t_run(&raw mut t_script as *mut u32, GENO_MODE_EXEC);
    if !t_fp.x3E4_fighterCmdScript.anon_8.u.is_null() {
        TestFail(
            b"the script did not reach End\0" as *const u8 as *const i8,
        );
        return 1 as i32;
    }
    if (*st).la_i[0 as i32 as usize] != 42 as i32
        || (*st).la_i[1 as i32 as usize] != 111 as i32
        || (*st).la_i[2 as i32 as usize] != 2 as i32
        || (*st).la_i[3 as i32 as usize] != 7 as i32
        || (*st).la_i[4 as i32 as usize] != 8 as i32
        || (*st).la_i[5 as i32 as usize] != 1 as i32
        || (*st).ra_i[0 as i32 as usize] != 42 as i32
        || (*st).la_i[6 as i32 as usize] != 3 as i32
        || (*st).la_f[0 as i32 as usize] != 3.0f32
    {
        TestFail(
            b"variable results wrong (expected LA0..6 = 42,111,2,7,8,1,3; RA0 42; LAF0 3)\0"
                as *const u8 as *const i8,
        );
        return 1 as i32;
    }
    if t_fp.cmd_vars[1 as i32 as usize] != 42 as u32
        || t_fp.cmd_vars[2 as i32 as usize] != 7 as u32
    {
        TestFail(
            b"vanilla SetCmdVar inside/after Geno branches did not run as expected\0"
                as *const u8 as *const i8,
        );
        return 1 as i32;
    }
    if (*st).la_i[7 as i32 as usize] != 1 as i32
        || (*st).hook_calls != 1 as u32
    {
        TestFail(
            b"CALL did not run geno.count_frames exactly once\0" as *const u8
                as *const i8,
        );
        return 1 as i32;
    }
    if (*st).flags & 1 as u32 == 0 {
        TestFail(
            b"the script flag was not set\0" as *const u8 as *const i8,
        );
        return 1 as i32;
    }
    return 0 as i32;
}

pub unsafe extern "C" fn test_geno_ftcmd_loops() -> i32 {
    melee_core::ct!("test_geno_ftcmd_loops");
    let mut st: *mut TestGenoState = t_setup();
    t_build(&raw mut t_script as *mut u32);
    t_run(&raw mut t_script as *mut u32, GENO_MODE_SKIP);
    if (*st).la_i[0 as i32 as usize] != 42 as i32
        || (*st).la_i[1 as i32 as usize] != 111 as i32
        || (*st).la_i[7 as i32 as usize] != 0 as i32
        || (*st).hook_calls != 0 as u32
    {
        TestFail(
            b"skip loop: expected vars set and no hook call\0" as *const u8
                as *const i8,
        );
        return 1 as i32;
    }
    st = t_setup();
    t_run(&raw mut t_script as *mut u32, GENO_MODE_ANIM);
    if (*st).la_i[0 as i32 as usize] != 42 as i32
        || (*st).la_i[7 as i32 as usize] != 1 as i32
    {
        TestFail(
            b"anim-start loop: expected vars set and one hook call\0" as *const u8
                as *const i8,
        );
        return 1 as i32;
    }
    return 0 as i32;
}

pub unsafe extern "C" fn test_geno_vanilla_script_untouched() -> i32 {
    melee_core::ct!("test_geno_vanilla_script_untouched");
    let mut st: *mut TestGenoState = t_setup();
    t_script[0 as i32 as usize] = ((19 as u32)
        << 26 as i32
        | (0 as i32 as u32) << 24 as i32
        | 5 as i32 as u32).to_be();
    t_script[1 as i32 as usize] = ((19 as u32)
        << 26 as i32
        | (3 as i32 as u32) << 24 as i32
        | 9 as i32 as u32).to_be();
    t_script[2 as i32 as usize] = (0 as u32).to_be();
    t_run(&raw mut t_script as *mut u32, GENO_MODE_EXEC);
    if t_fp.cmd_vars[0 as i32 as usize] != 5 as u32
        || t_fp.cmd_vars[3 as i32 as usize] != 9 as u32
        || (*st).flags != 0 as u32
    {
        TestFail(
            b"a plain script changed Geno state or did not run\0" as *const u8
                as *const i8,
        );
        return 1 as i32;
    }
    return 0 as i32;
}

pub unsafe extern "C" fn test_geno_state_resets() -> i32 {
    melee_core::ct!("test_geno_state_resets");
    let mut st: *mut TestGenoState = t_setup();
    let mut resets: u32 = (*st).resets;
    (*st).la_i[3 as i32 as usize] = 5 as i32 as i32;
    (*st).ra_i[3 as i32 as usize] = 6 as i32 as i32;
    (*st).ra_f[1 as i32 as usize] = 1.0f32 as f32;
    (*st).flags |= 1 as u32;
    Geno_OnActionChange(&raw mut t_gobj);
    if (*st).la_i[3 as i32 as usize] != 5 as i32
        || (*st).ra_i[3 as i32 as usize] != 0 as i32
        || (*st).ra_f[1 as i32 as usize] != 0.0f32
    {
        TestFail(
            b"action change must clear RA and keep LA\0" as *const u8
                as *const i8,
        );
        return 1 as i32;
    }
    Geno_FighterReset(&raw mut t_fp);
    if (*st).la_i[3 as i32 as usize] != 0 as i32
        || (*st).resets != resets.wrapping_add(1 as u32)
        || (*st).profile != -(1 as i32)
    {
        TestFail(
            b"fighter reset must clear LA (and no profile without a geno.json)\0"
                as *const u8 as *const i8,
        );
        return 1 as i32;
    }
    return 0 as i32;
}

pub unsafe extern "C" fn test_geno_attr_table() -> i32 {
    melee_core::ct!("test_geno_attr_table");
    if GenoGame_AttrOffset(
        GenoGame_AttrFind(b"max_jumps\0" as *const u8 as *const i8),
    ) != 0x58 as i32
        || GenoGame_AttrOffset(
            GenoGame_AttrFind(b"gravity\0" as *const u8 as *const i8),
        ) != 0x5c as i32
        || GenoGame_AttrOffset(
            GenoGame_AttrFind(b"weight\0" as *const u8 as *const i8),
        ) != 0x88 as i32
        || GenoGame_AttrFind(b"nope\0" as *const u8 as *const i8)
            != -(1 as i32)
    {
        TestFail(
            b"attribute name table does not match ftCo_DatAttrs\0" as *const u8
                as *const i8,
        );
        return 1 as i32;
    }
    let mut i: i32 = 0;
    let mut n: i32 = 0 as i32;
    i = 0 as i32;
    while GenoGame_AttrOffset(i) >= 0 as i32 {
        let mut off: i32 = GenoGame_AttrOffset(i);
        if off + 4 as i32
            > ::core::mem::size_of::<ftCo_DatAttrs>() as i32
            || off & 3 as i32 != 0 as i32
        {
            TestFail(
                b"an attribute offset lies outside ftCo_DatAttrs or is unaligned\0"
                    as *const u8 as *const i8,
            );
            return 1 as i32;
        }
        n += 1;
        i += 1;
    }
    if GenoGame_AttrFind(b"walk_accel_mul\0" as *const u8 as *const i8)
        != 0 as i32
        || GenoGame_AttrFind(
            b"clank_animation_length\0" as *const u8 as *const i8,
        ) != 39 as i32
        || GenoGame_AttrFind(
            b"hit_spark_variant\0" as *const u8 as *const i8,
        ) != 40 as i32
        || GenoGame_AttrFind(
            b"normal_landing_lag\0" as *const u8 as *const i8,
        ) != 47 as i32
        || GenoGame_AttrFind(
            b"landingairn_lag\0" as *const u8 as *const i8,
        ) != 48 as i32
        || GenoGame_AttrOffset(47 as i32) != 0xe4 as i32
        || GenoGame_AttrIsInt(40 as i32) != 1 as i32
        || GenoGame_AttrIsInt(47 as i32) != 0 as i32
        || n < 60 as i32 || n > GENO_MAX_ATTRS
    {
        TestFail(
            b"attribute table order or size is not the v7 table\0" as *const u8
                as *const i8,
        );
        return 1 as i32;
    }
    if GenoGame_HookFind(
        b"geno.count_frames\0" as *const u8 as *const i8,
    ) != GENO_HOOK_COUNT_FRAMES as i32
        || GenoGame_HookFind(b"geno.nope\0" as *const u8 as *const i8)
            != -(1 as i32)
    {
        TestFail(b"hook name table\0" as *const u8 as *const i8);
        return 1 as i32;
    }
    return 0 as i32;
}

pub unsafe extern "C" fn test_geno_multijump() -> i32 {
    melee_core::ct!("test_geno_multijump");
    let mut msid: i32 = 0;
    let mut vy: f32 = 0.;
    let mut i: i32 = 0;
    let mut rc: i32 = 0 as i32;
    if Geno_TestInstall(
        b"{\"geno\":1,\"fighters\":[{\"attach\":\"kirby\",\"attributes\":{\"gravity\":0.05},\"jumps\":{\"max\":9,\"air_vy\":[2.0,1.0]}}]}\0"
            as *const u8 as *const i8,
    ) != 1 as i32
    {
        TestFail(
            b"could not install the test profile\0" as *const u8
                as *const i8,
        );
        Geno_TestRestore();
        return 1 as i32;
    }
    t_setup();
    t_zero(
        &raw mut t_mj as *mut c_void,
        ::core::mem::size_of::<Fighter_x2D0_t>() as i32,
    );
    t_mj.x28 = 5 as i32;
    t_mj.x2C = 300 as i32 as enum_t;
    i = 0 as i32;
    while i < 5 as i32 {
        t_mj.x14[i as usize] = 1.5f32 - 0.25f32 * i as f32;
        i += 1;
    }
    t_fp.x2D0 = &raw mut t_mj;
    t_fp.co_attrs.max_jumps = 6 as i32;
    t_fp.co_attrs.gravity = 0.1f32;
    Geno_ApplyAttrs(&raw mut t_fp);
    if t_fp.co_attrs.max_jumps != 9 as i32
        || t_fp.co_attrs.gravity != 0.05f32
    {
        TestFail(
            b"attributes not overridden (max_jumps 9, gravity 0.05)\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    t_fp.x1968_jumpsUsed = 1 as u8;
    msid = 300 as i32;
    vy = 1.5f32;
    Geno_MultiJump(&raw mut t_fp, 300 as i32, &raw mut msid, &raw mut vy);
    if msid != 300 as i32 || vy != 2.0f32 {
        TestFail(
            b"jump 1: expected state 300, vy 2.0\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    t_fp.x1968_jumpsUsed = 7 as u8;
    msid = 306 as i32;
    vy = 123.0f32;
    Geno_MultiJump(&raw mut t_fp, 300 as i32, &raw mut msid, &raw mut vy);
    if msid != 303 as i32 || vy != 1.0f32 {
        TestFail(
            b"jump 7: expected state 303 (repeatable row), vy 1.0\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    Geno_TestRestore();
    msid = 306 as i32;
    vy = 123.0f32;
    Geno_MultiJump(&raw mut t_fp, 300 as i32, &raw mut msid, &raw mut vy);
    if msid != 306 as i32 || vy != 123.0f32 {
        TestFail(
            b"without a profile Geno_MultiJump must not touch anything\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    return rc;
}

pub unsafe extern "C" fn test_geno_multijump_script_gate() -> i32 {
    melee_core::ct!("test_geno_multijump_script_gate");
    let mut used: i32 = 0;
    let mut base: i32 = 0;
    let mut msid: i32 = 0;
    let mut rc: i32 = 0 as i32;
    let mut vy: f32 = 0.;
    Geno_TestInstall(
        b"{\"geno\":5,\"fighters\":[{\"attach\":\"kirby\",\"jumps\":{\"max\":9}}]}\0"
            as *const u8 as *const i8,
    );
    t_setup();
    t_zero(
        &raw mut t_mj as *mut c_void,
        ::core::mem::size_of::<Fighter_x2D0_t>() as i32,
    );
    t_mj.x28 = 5 as i32;
    t_fp.x2D0 = &raw mut t_mj;
    t_fp.co_attrs.max_jumps = 6 as i32;
    Geno_ApplyAttrs(&raw mut t_fp);
    t_mj.x14[3 as i32 as usize] = 1.25f32;
    t_mj.x14[4 as i32 as usize] = 1.0f32;
    base = 341 as i32;
    while base <= 346 as i32 {
        used = 1 as i32;
        while used < 9 as i32 {
            let mut row: i32 = if (used - 1 as i32)
                < 4 as i32
            {
                used - 1 as i32
            } else if used == 8 as i32 {
                4 as i32
            } else {
                3 as i32
            };
            t_fp.x1968_jumpsUsed = used as u8;
            msid = base + used - 1 as i32;
            vy = 123.0f32;
            Geno_MultiJump(&raw mut t_fp, base, &raw mut msid, &raw mut vy);
            if msid != base + row
                || vy
                    != t_mj
                        .x14[(if used > 5 as i32 {
                        4 as i32
                    } else {
                        used - 1 as i32
                    }) as usize]
            {
                TestFail(
                    b"multi-jump must keep a script with a next-jump gate until the last jump\0"
                        as *const u8 as *const i8,
                );
                rc = 1 as i32;
            }
            used += 1;
        }
        base += 5 as i32;
    }
    Geno_TestRestore();
    return rc;
}

pub unsafe extern "C" fn test_geno_reload_state_callbacks() -> i32 {
    melee_core::ct!("test_geno_reload_state_callbacks");
    let mut a: *const i8 = b"{\"geno\":5,\"fighters\":[{\"attach\":\"kirby\",\"states\":[{\"name\":\"TutorialB\",\"behavior\":\"geno.ground\",\"subaction\":3}]}]}\0"
        as *const u8 as *const i8;
    let mut b: *const i8 = b"{\"geno\":5,\"fighters\":[{\"attach\":\"kirby\",\"states\":[{\"name\":\"TutorialB\",\"behavior\":\"geno.ground\",\"subaction\":3,\"iasa\":\"interrupt\"}]}]}\0"
        as *const u8 as *const i8;
    let mut rc: i32 = 0 as i32;
    Geno_TestInstall(a);
    if Geno_TestReloadLayout(a) != 0 as i32
        || Geno_TestReloadLayout(b) != 1 as i32
    {
        TestFail(
            b"IASA callback edits must restart; unchanged rows must stay live\0"
                as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    Geno_TestRestore();
    return rc;
}

pub unsafe extern "C" fn test_geno_effective_script_range() -> i32 {
    melee_core::ct!("test_geno_effective_script_range");

    let mut rc: i32 = 0 as i32;
    Geno_TestInstall(
        b"{\"geno\":5,\"fighters\":[{\"attach\":\"kirby\",\"subactions\":[{\"index\":3,\"words\":[\"0x04000004\",\"0x5c000000\"]}]}]}\0"
            as *const u8 as *const i8,
    );
    t_setup();
    if GenoGame_ScriptRange(
        (&raw mut Geno_ScriptPool as *mut u32).offset(0 as i32 as isize)
            as *mut u32 as u32,
        12 as u32,
    ) == 0
        || GenoGame_ScriptRange(
            (&raw mut Geno_ScriptPool as *mut u32)
                .offset(0 as i32 as isize) as *mut u32 as u32,
            16 as u32,
        ) != 0
        || GenoGame_ScriptRange(
            ((&raw mut Geno_ScriptPool as *mut u32)
                .offset(0 as i32 as isize) as *mut u32 as u32)
                .wrapping_sub(4 as u32),
            4 as u32,
        ) != 0
        || GenoGame_ScriptHasIasa(
            t_subactions[3 as i32 as usize].xC
                as *const c_void,
        ) == 0
    {
        TestFail(
            b"effective overlay script must be readable only inside its installed slot\0"
                as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    t_script[0 as i32 as usize] = (((GENO_FTCMD_OP
        << 26 as i32) as u32
        | (GENO_SUB_CALL as i32 as u32
            & 63 as u32) << 20 as i32
        | (3 as i32 as u32 & 15 as u32)
            << 16 as i32
        | 0 as i32 as u32 & 0xffff as u32)
        as u32).to_be();
    t_script[1 as i32 as usize] = (0x5c000000 as i32
        as u32).to_be();
    t_script[2 as i32 as usize] = (0 as u32).to_be();
    t_script[3 as i32 as usize] = (0 as u32).to_be();
    if GenoGame_ScriptHasIasa(
        &raw mut t_script as *mut u32 as *const c_void,
    ) != 0
    {
        TestFail(
            b"IASA warning must decode command lengths rather than scan operands\0"
                as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    Geno_TestRestore();
    return rc;
}

pub unsafe extern "C" fn test_geno_ordinary_jumps() -> i32 {
    melee_core::ct!("test_geno_ordinary_jumps");


    static mut common: ftCommonData = ftCommonData {
        horizontal_stick_deadzone: 0.,
        vertical_stick_deadzone: 0.,
        horizontal_stick_smash_deadzone: 0.,
        vertical_stick_smash_deadzone: 0.,
        analog_shoulder_deadzone: 0.,
        z_press_analog_value: 0.,
        shield_press_threshold: 0.,
        x1C: 0,
        x20_radians: 0.,
        walk_stick_threshold: 0.,
        walk_middle_animation_stick_threshold: 0.,
        walk_fast_stick_threshold: 0.,
        walk_accel_taper_gain: 0.,
        x34: 0.,
        x38_someLStickXThreshold: 0.,
        dash_smash_stick_threshold: 0.,
        dash_smash_window: 0,
        x44: 0.,
        x48: 0.,
        x4C: 0.,
        x50: 0.,
        x54: 0.,
        x58_someLStickXThreshold: 0.,
        run_accel_taper_gain: 0.,
        run_dash_turn_friction_multiplier: 0.,
        x64: 0.,
        x68: 0.,
        friction_when_above_walk_speed: 0.,
        tap_jump_threshold: 0.,
        tap_jump_window: 0,
        x78: 0.,
        tap_jump_release_threshold: 0.,
        relaxed_tap_jump_threshold: 0.,
        x84: 0.,
        x88: 0.,
        x8C: 0,
        x90: 0.,
        x94: 0.,
        x98: 0.,
        x9C_radians: 0.,
        xA0_radians: 0.,
        xA4_radians: 0.,
        xA8_radians: 0.,
        attackhi3_stick_threshold_y: 0.,
        xB0: 0.,
        xB4: 0.,
        xB8_radians: 0.,
        xBC_radians: 0.,
        xC0_radians: 0.,
        xC4_radians: 0.,
        xC8: 0.,
        xCC: 0.,
        xD0: 0.,
        xD4: 0.,
        xD8: 0.,
        xDC: 0.,
        xE0: 0.,
        xE4: 0,
        xE8: 0.,
        xEC: 0.,
        xF0: 0.,
        xF4: 0.,
        xF8: 0.,
        xFC: 0,
        x100: 0.,
        kb_min: 0.,
        x108: 0.,
        x10C: 0.,
        x110: 0.,
        x114: 0.,
        x118: 0.,
        x11C: 0.,
        x120: 0.,
        kb_squat_mul: 0.,
        x128: 0.,
        x12C: 0.,
        x130: 0,
        x134: 0,
        x138: 0.,
        x13C: 0.,
        x140: 0.,
        x144_radians: 0.,
        x148: 0.,
        x14C: 0.,
        x150: 0.,
        x154: 0.,
        x158: 0.,
        x15C: 0.,
        x160: 0.,
        max_grounded_kb_on_landing: 0.,
        x168: 0.,
        x16C: 0.,
        x170: 0.,
        x174: 0.,
        x178: 0.,
        x17C: 0.,
        x180: 0.,
        x184: 0.,
        x188: 0.,
        x18C: 0,
        x190: 0.,
        x194_unkHitLagFrames: 0.,
        x198: 0.,
        x19C: 0.,
        x1A0: 0.,
        x1A4: 0.,
        x1A8: 0.,
        x1AC: 0.,
        x1B0: 0.,
        x1B4: 0.,
        x1B8: 0,
        x1BC: 0.,
        x1C0: 0.,
        x1C4: 0.,
        x1C8: 0.,
        x1CC: 0.,
        x1D0: 0.,
        x1D4: 0.,
        x1D8: 0.,
        x1DC: 0,
        x1E0: 0.,
        x1E4: 0.,
        x1E8_radians: 0.,
        x1EC: 0.,
        x1F0: 0.,
        x1F4: 0.,
        x1F8: 0.,
        aerial_friction_oob: 0.,
        x200: 0.,
        x204_knockbackFrameDecay: 0.,
        x208: 0.,
        x20C: 0.,
        x210: 0.,
        x214: 0,
        x218: 0.,
        x21C: 0.,
        x220: 0.,
        x224: 0,
        x228: 0.,
        x22C: 0.,
        x230: 0.,
        x234_radians: 0.,
        x238_radians: 0.,
        x23C: 0,
        x240: 0.,
        x244: 0.,
        x248: 0.,
        x24C: 0.,
        x250: 0.,
        x254: 0.,
        x258: 0.,
        x25C: 0.,
        x260_startShieldHealth: 0.,
        x264: 0.,
        x268: 0.,
        x26C: 0.,
        x270: 0.,
        x274: 0,
        x278: 0.,
        x27C: 0.,
        x280_unkShieldHealth: 0.,
        x284: 0.,
        x288: 0.,
        x28C: 0.,
        x290: 0.,
        x294: 0.,
        x298: 0.,
        x29C: 0.,
        powershield_input_window: 0,
        x2A4: 0.,
        x2A8: 0.,
        x2AC: 0.,
        x2B0: 0.,
        x2B4: 0.,
        x2B8: 0,
        x2BC: 0.,
        x2C0: 0.,
        x2C4: 0.,
        x2C8: 0.,
        x2CC: 0.,
        x2D0: 0.,
        x2D4: 0.,
        x2D8: 0.,
        x2DC: 0.,
        x2E0: 0.,
        x2E4: 0.,
        x2E8: 0.,
        x2EC: 0.,
        x2F0: 0.,
        x2F4: 0.,
        x2F8: 0.,
        x2FC: 0.,
        x300: 0.,
        x304: 0.,
        x308: 0.,
        x30C: 0.,
        x310: 0.,
        x314: 0.,
        x318: 0,
        x31C: 0.,
        x320: 0,
        x324: 0,
        x328: 0.,
        escapeair_deadzone: Vec2 { x: 0., y: 0. },
        x334: 0,
        escapeair_force: 0.,
        escapeair_decay: 0.,
        x340: 0.,
        x344: 0.,
        x348: 0,
        x34C: 0.,
        x350: 0.,
        x354: 0.,
        x358: 0.,
        x35C: 0.,
        x360: 0.,
        x364: 0.,
        x368: 0.,
        x36C: 0.,
        x370: 0.,
        x374: 0.,
        x378: 0.,
        x37C: 0.,
        x380: lbColl_80008D30_arg1 {
            state: HitCapsule_Disabled,
            damage: 0,
            kb_angle: 0,
            unkC: 0,
            unk10: 0,
            unk14: 0,
            element: 0,
            sfx_severity: 0,
            sfx_kind: 0,
        },
        grab_timer_decrement: 0.,
        x3A8: 0.,
        x3AC: 0.,
        x3B0: 0.,
        shouldered_anim_rate: 0.,
        x3B8: 0.,
        x3BC: 0.,
        x3C0: 0,
        x3C4: 0.,
        x3C8: 0.,
        x3CC: 0,
        x3D0: 0.,
        x3D4: 0.,
        x3D8: 0.,
        x3DC: 0.,
        x3E0: 0.,
        x3E4: 0.,
        x3E8_shieldKnockbackFrameDecay: 0.,
        x3EC_shieldGroundFrictionMultiplier: 0.,
        x3F0: 0.,
        x3F4: 0,
        x3F8: 0,
        x3FC: 0,
        x400: 0.,
        x404: 0.,
        x408: 0.,
        x40C: 0.,
        x410: 0,
        x414: 0,
        x418: 0,
        x41C: 0,
        x420: 0.,
        x424: 0.,
        x428: 0,
        x42C: 0.,
        x430: 0.,
        x434: 0.,
        x438: 0.,
        x43C: 0.,
        x440: 0.,
        x444: 0.,
        x448: 0.,
        x44C: 0.,
        x450: 0.,
        x454: 0.,
        x458: 0.,
        x45C: 0.,
        x460: 0.,
        x464: 0.,
        x468: 0.,
        x46C: 0.,
        x470: 0.,
        teeter_walk_threshold: 0.,
        x478: 0.,
        x47C: 0.,
        x480: 0.,
        x484: 0.,
        x488: 0,
        x48C: 0.,
        x490: 0.,
        x494: 0.,
        ledge_cooldown: 0,
        x49C: 0,
        x4A0: 0.,
        x4A4: 0.,
        x4A8: 0.,
        x4AC: 0.,
        sdi_min_stick_mag: 0.,
        sdi_stick_window: 0,
        sdi_pos_scale: 0.,
        x4BC: 0.,
        x4C0: 0.,
        x4C4: 0,
        x4C8: 0,
        x4CC: 0,
        x4D0: 0.,
        x4D4: 0.,
        x4D8: 0,
        x4DC: Vec2 { x: 0., y: 0. },
        x4E4: Vec { x: 0., y: 0., z: 0. },
        x4F0: 0.,
        x4F4: 0.,
        x4F8: 0,
        x4FC: 0,
        x500: 0,
        x504: 0,
        x508: 0,
        x50C: 0,
        x510: 0.,
        x514: 0.,
        x518: 0,
        x51C_radians: 0.,
        fall_common: FallCommon {
            x520: 0,
            x524: 0,
            x528: 0,
            x52C: 0,
            x530: 0,
            x534: 0,
            x538: Vec { x: 0., y: 0., z: 0. },
            x544: Vec { x: 0., y: 0., z: 0. },
            x550: 0.,
            x554: 0.,
            x558: 0.,
            x55C: 0.,
            x560_radians: 0.,
            x564: 0.,
        },
        x568: 0.,
        x56C: 0.,
        x570: 0.,
        x574: 0.,
        x578: 0.,
        x57C: 0,
        x580: 0,
        x584: 0,
        x588: 0,
        x58C: 0.,
        x590: 0.,
        open_parasol_threshold: 0.,
        close_parasol_threshold: 0.,
        x59C: 0.,
        x5A0: 0.,
        x5A4: 0,
        x5A8: 0.,
        x5AC: 0.,
        x5B0: 0.,
        x5B4: 0,
        x5B8: 0.,
        x5BC: 0,
        x5C0: 0.,
        x5C4: 0,
        x5C8: 0,
        x5CC: 0.,
        rebirth_countdown: 0,
        rebirth_wait: 0,
        x5D8: 0,
        bury_timer_unk1: 0,
        bury_timer_unk2: 0,
        bury_timer_unk3: 0,
        x5E8: 0.,
        x5EC: 0,
        x5F0: 0,
        x5F4: 0,
        x5F8: 0.,
        x5FC: 0.,
        x600: 0.,
        x604: 0.,
        x608: 0.,
        x60C: 0.,
        x610: 0.,
        x614: 0.,
        x618: 0.,
        x61C: 0.,
        x620: 0,
        x624: 0.,
        x628: 0.,
        x62C: 0.,
        x630: 0.,
        x634: 0.,
        x638: 0.,
        x63C: 0.,
        x640: 0.,
        x644: 0.,
        x648: 0,
        x64C: 0.,
        x650: 0.,
        x654: 0.,
        x658: 0.,
        x65C: 0.,
        x660: 0.,
        x664: 0.,
        pressed_inputs: 0.,
        released_inputs: 0.,
        x670: 0.,
        x674: 0.,
        x678: 0.,
        x67C: 0.,
        x680: 0.,
        x684: 0.,
        x688: 0,
        x68C: 0,
        x690: 0,
        x694: 0.,
        x698: 0.,
        warpstarfall_drift_scaling: 0.,
        warpstarfall_drift_flat: 0.,
        warpstarfall_drift_max: 0.,
        x6A8: 0.,
        x6AC: 0,
        x6B0: 0,
        x6B4: 0,
        x6B8: 0,
        x6BC: 0,
        x6C0: 0,
        x6C4: 0.,
        x6C8: 0,
        x6CC: 0,
        x6D0: 0.,
        x6D4: 0,
        x6D8: [0; 1],
        x6DC_colorsByPlayer: [_GXColor { r: 0, g: 0, b: 0, a: 0 }; 4],
        x6EC: [0; 4],
        metal_armor: 0.,
        x6F4_unkDamage: 0,
        x6F8: 0,
        x6FC: 0,
        x700: 0,
        x704: 0.,
        x708: 0.,
        x70C: 0.,
        x710: 0.,
        x714: 0.,
        kb_ice_mul: 0.,
        x71C: 0.,
        x720: 0.,
        x724: 0.,
        x728: 0.,
        x72C: 0.,
        x730: 0.,
        leadead_grab_timer_step: 0.,
        leadead_grab_break_threshold: 0.,
        x73C: 0,
        x740: 0.,
        x744: 0.,
        x748: 0.,
        x74C: 0.,
        x750: 0.,
        x754: 0.,
        x758: 0.,
        x75C: 0.,
        x760: 0,
        x764: 0,
        x768: 0.,
        x76C: 0.,
        x770: 0.,
        x774: 0,
        passive_wall_vel_y_base: 0.,
        damageice_gravity_mult: 0.,
        damageice_min_speed: 0.,
        damageice_speed_mult_on_break: 0.,
        damageice_rot_speed_min: 0.,
        damageice_rot_speed_max: 0.,
        x790_damageice_unk: 0.,
        x794_damageice_unk: 0.,
        x798_damageice_unk: 0.,
        damageice_dmg_time_reduction_mult: 0.,
        damageice_ice_size: 0.,
        damageicejump_escape_time: 0.,
        x7A8: 0.,
        x7AC: 0,
        x7B0: 0,
        x7B4_unkDamage: 0,
        x7B8: 0.,
        x7BC: 0.,
        x7C0: 0.,
        kb_smashcharge_mul: 0.,
        x7C8: 0.,
        x7CC: 0,
        x7D0: 0,
        hit_weight_mul: 0.,
        x7D8: _GXColor { r: 0, g: 0, b: 0, a: 0 },
        x7DC: 0,
        x7E0: 0,
        x7E4_scaleZ: 0.,
        unk_kb_angle_min: 0,
        unk_kb_angle_max: 0,
        x7F0: 0,
        x7F4: 0.,
        x7F8: 0.,
        x7FC: 0.,
        x800: 0.,
        x804: 0.,
        x808: Vec { x: 0., y: 0., z: 0. },
        x814: 0,
    };
    let mut saved: *mut ftCommonData = p_ftCommonData;
    let mut used: i32 = 0;
    let mut rc: i32 = 0 as i32;
    Geno_TestInstall(
        b"{\"geno\":5,\"fighters\":[{\"attach\":\"mario\",\"jumps\":{\"max\":9}}]}\0"
            as *const u8 as *const i8,
    );
    t_setup();
    t_fp.kind = Ft_Kind_Mario;
    t_fp.co_attrs.max_jumps = 2 as i32;
    Geno_ApplyAttrs(&raw mut t_fp);
    t_zero(
        &raw mut common as *mut c_void,
        ::core::mem::size_of::<ftCommonData>() as i32,
    );
    common.tap_jump_threshold = 1.0f32;
    p_ftCommonData = &raw mut common;
    t_fp.input.pressed_buttons = HSD_PAD_X as HSD_Pad;
    used = 1 as i32;
    while used < 9 as i32 {
        t_fp.x1968_jumpsUsed = used as u8;
        if ft_did_jump(&raw mut t_fp, 0) == 0 {
            rc = 1 as i32;
        }
        used += 1;
    }
    t_fp.x1968_jumpsUsed = 9 as u8;
    if ft_did_jump(&raw mut t_fp, 0) != 0
        || t_fp.co_attrs.max_jumps != 9 as i32
    {
        rc = 1 as i32;
    }
    t_fp.x1968_jumpsUsed = 2 as u8;
    t_fp.input.pressed_buttons = 0 as HSD_Pad;
    if ft_did_jump(&raw mut t_fp, 0) != 0 {
        rc = 1 as i32;
    }
    p_ftCommonData = saved;
    Geno_TestRestore();
    if rc != 0 {
        TestFail(
            b"ordinary jump input must allow eight air jumps and stop at max 9\0"
                as *const u8 as *const i8,
        );
    }
    return rc;
}

pub unsafe extern "C" fn test_geno_jump_limits() -> i32 {
    melee_core::ct!("test_geno_jump_limits");
    let mut rc: i32 = 0 as i32;
    Geno_TestInstall(
        b"{\"geno\":5,\"fighters\":[{\"attach\":\"kirby\",\"jumps\":{\"max\":1e50}}]}\0"
            as *const u8 as *const i8,
    );
    t_setup();
    Geno_ApplyAttrs(&raw mut t_fp);
    if t_fp.co_attrs.max_jumps != 250 as i32 {
        rc = 1 as i32;
    }
    Geno_TestInstall(
        b"{\"geno\":5,\"fighters\":[{\"attach\":\"kirby\",\"attributes\":{\"max_jumps\":300}}]}\0"
            as *const u8 as *const i8,
    );
    t_setup();
    Geno_ApplyAttrs(&raw mut t_fp);
    if t_fp.co_attrs.max_jumps != 250 as i32 {
        rc = 1 as i32;
    }
    Geno_TestInstall(
        b"{\"geno\":5,\"fighters\":[{\"attach\":\"kirby\",\"jumps\":{\"max\":0}}]}\0"
            as *const u8 as *const i8,
    );
    t_setup();
    Geno_ApplyAttrs(&raw mut t_fp);
    if t_fp.co_attrs.max_jumps != 1 as i32 {
        rc = 1 as i32;
    }
    Geno_TestRestore();
    if rc != 0 {
        TestFail(
            b"both max-jump keys must clamp safely to the u8-supported range\0"
                as *const u8 as *const i8,
        );
    }
    return rc;
}

pub unsafe extern "C" fn GenoTest_LabScript(
    mut original: u32,
) -> *const c_void {
    melee_core::ct!("GenoTest_LabScript");
    t_subactions[3 as i32 as usize].xC = original as *mut CmdUnion;
    Geno_TestInstall(
        b"{\"geno\":5,\"fighters\":[{\"attach\":\"kirby\",\"states\":[{\"name\":\"TutorialB\",\"subaction\":3,\"iasa\":\"interrupt\"}],\"subactions\":[{\"index\":3,\"words\":[\"0x04000005\",\"0xed030000\",0,5,\"0x2c80100c\",\"0x04000000\",0,\"0xb4990000\",\"0x0a000000\",\"0x5c000000\",\"0xec930000\",3,\"0x40000000\",\"0xed310000\"]}]}]}\0"
            as *const u8 as *const i8,
    );
    t_setup();
    t_fp.x24 = &raw mut t_subactions as *mut Fighter_WaitAnimData
        as *mut Fighter_WaitAnimData;
    return (*t_fp.x24.offset(3 as i32 as isize)).xC
        as *const c_void;
}

pub unsafe extern "C" fn GenoTest_LabScriptEnd() {
    melee_core::ct!("GenoTest_LabScriptEnd");
    t_subactions[3 as i32 as usize].xC = ::core::ptr::null_mut::<
        CmdUnion,
    >();
    Geno_TestRestore();
}

pub unsafe extern "C" fn test_geno_iasa_script_validation() -> i32 {
    melee_core::ct!("test_geno_iasa_script_validation");
    let mut rc: i32 = 0 as i32;
    Geno_TestInstall(
        b"{\"geno\":5,\"fighters\":[{\"attach\":\"kirby\",\"subactions\":[{\"index\":3,\"words\":[\"0x14000000\",1]},{\"index\":4,\"words\":[\"0xec1f0000\",\"0x5c000000\"]},{\"index\":5,\"words\":[\"0xec230000\",\"0x5c000000\",0]},{\"index\":6,\"words\":[\"0xed030000\",0,1,0,\"0x5c000000\",0]}]}]}\0"
            as *const u8 as *const i8,
    );
    t_setup();
    if GenoGame_ScriptHasIasa(
        t_subactions[3 as i32 as usize].xC as *const c_void,
    ) != 0
        || GenoGame_ScriptHasIasa(
            t_subactions[4 as i32 as usize].xC
                as *const c_void,
        ) != 0
        || GenoGame_ScriptHasIasa(
            t_subactions[5 as i32 as usize].xC
                as *const c_void,
        ) != 0
    {
        TestFail(
            b"IASA load scan must reject bad pointers/lengths and ignore operand opcodes\0"
                as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    if GenoGame_ScriptHasIasa(
        t_subactions[6 as i32 as usize].xC as *const c_void,
    ) == 0
    {
        TestFail(
            b"IASA load warning must inspect a conditional arm beyond a fall-through End\0"
                as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    Geno_TestRestore();
    return rc;
}

pub unsafe extern "C" fn test_geno_state_savestate() -> i32 {
    melee_core::ct!("test_geno_state_savestate");
    let mut st: *mut TestGenoState = t_setup();
    (*st).la_i[10 as i32 as usize] = 1234 as i32 as i32;
    (*st).la_f[2 as i32 as usize] = 2.5f32 as f32;
    (*st).ra_i[63 as i32 as usize] = -(7 as i32) as i32;
    (*st).nchecks = 2 as u32;
    (*st).checks[1 as i32 as usize].target = 0x10000005
        as u32 as u32;
    (*st)
        .checks[1 as i32 as usize]
        .cond[0 as i32 as usize]
        .arg1 = 77 as u32;
    (*st).rehit_period[2 as i32 as usize] = 6 as i32
        as i32;
    (*st).rehit_count[2 as i32 as usize] = 4 as i32 as i32;
    (*st).link_mode[3 as i32 as usize] = GENO_LINK_SPEED
        as i32 as i32;
    (*st).action_time = 31 as i32 as i32;
    (*st).edge_target = 43 as u32;
    if snap_open(1 as i32) != 0 as i32 {
        TestFail(
            b"gw_snap_open failed (no melee-pc.map beside the exe?)\0" as *const u8
                as *const i8,
        );
        return 1 as i32;
    }
    snap_save(0 as i32);
    (*st).la_i[10 as i32 as usize] = 99 as i32 as i32;
    (*st).la_f[2 as i32 as usize] = 0.0f32 as f32;
    (*st).ra_i[63 as i32 as usize] = 0 as i32 as i32;
    (*st).nchecks = 0 as u32;
    (*st).checks[1 as i32 as usize].target = 0 as u32;
    (*st)
        .checks[1 as i32 as usize]
        .cond[0 as i32 as usize]
        .arg1 = 0 as u32;
    (*st).rehit_period[2 as i32 as usize] = 0 as i32
        as i32;
    (*st).rehit_count[2 as i32 as usize] = 0 as i32 as i32;
    (*st).link_mode[3 as i32 as usize] = 0 as i32 as i32;
    (*st).action_time = 0 as i32 as i32;
    (*st).edge_target = 0 as u32;
    if snap_load(0 as i32) != 0 as i32 {
        TestFail(b"gw_snap_load failed\0" as *const u8 as *const i8);
        return 1 as i32;
    }
    if (*st).la_i[10 as i32 as usize] != 1234 as i32
        || (*st).la_f[2 as i32 as usize] != 2.5f32
        || (*st).ra_i[63 as i32 as usize] != -(7 as i32)
    {
        TestFail(
            b"the Geno state block was not restored by a savestate load\0" as *const u8
                as *const i8,
        );
        return 1 as i32;
    }
    if (*st).nchecks != 2 as u32
        || (*st).checks[1 as i32 as usize].target
            != 0x10000005 as u32
        || (*st)
            .checks[1 as i32 as usize]
            .cond[0 as i32 as usize]
            .arg1 != 77 as u32
        || (*st).rehit_period[2 as i32 as usize]
            != 6 as i32
        || (*st).rehit_count[2 as i32 as usize] != 4 as i32
        || (*st).link_mode[3 as i32 as usize]
            != GENO_LINK_SPEED as i32
        || (*st).action_time != 31 as i32
        || (*st).edge_target != 43 as u32
    {
        TestFail(
            b"the v1 state (checks, rehit, autolink, action frame, edge) was not restored\0"
                as *const u8 as *const i8,
        );
        return 1 as i32;
    }
    return 0 as i32;
}

pub unsafe extern "C" fn test_geno_v1_values() -> i32 {
    melee_core::ct!("test_geno_v1_values");
    let mut st: *mut TestGenoState = t_setup();
    let mut n: i32 = 0 as i32;
    let mut s: *mut u32 = &raw mut t_script as *mut u32;
    t_fp.ground_or_air = GA_Air;
    t_fp.self_vel.x = 1.5f32 as f32;
    t_fp.facing_dir = -1.0f32;
    t_fp.input.lstick[0 as i32 as usize].x = 0.5f32 as f32;
    t_fp.cmd_vars[2 as i32 as usize] = 7 as u32;
    t_fp.motion_id = 123 as i32 as FtMotionId;
    t_dat_attrs[3 as i32 as usize] = t_fbits(2.5f32);
    t_dat_attrs[4 as i32 as usize] = 9 as u32;
    let fresh62 = n;
    n = n + 1;
    *s.offset(fresh62 as isize) = (((GENO_FTCMD_OP << 26 as i32)
        as u32
        | (GENO_SUB_GET as i32 as u32
            & 63 as u32) << 20 as i32
        | (2 as i32 as u32 & 15 as u32)
            << 16 as i32
        | ((((2 as i32 & 3 as i32)
            << 6 as i32
            | 1 as i32 & 63 as i32)
            & 0xff as i32) << 8 as i32
            | (if 0 as i32 != 0 {
                0x80 as i32
            } else {
                0 as i32
            })
            | (0 as i32 & 7 as i32)
                << 4 as i32) as u32
            & 0xffff as u32) as u32).to_be();
    let fresh63 = n;
    n = n + 1;
    *s.offset(fresh63 as isize) = (GENO_VAL_VEL_X as i32 as u32).to_be();
    let fresh64 = n;
    n = n + 1;
    *s.offset(fresh64 as isize) = (((GENO_FTCMD_OP << 26 as i32)
        as u32
        | (GENO_SUB_GET as i32 as u32
            & 63 as u32) << 20 as i32
        | (2 as i32 as u32 & 15 as u32)
            << 16 as i32
        | ((((0 as i32 & 3 as i32)
            << 6 as i32
            | 1 as i32 & 63 as i32)
            & 0xff as i32) << 8 as i32
            | (if 0 as i32 != 0 {
                0x80 as i32
            } else {
                0 as i32
            })
            | (0 as i32 & 7 as i32)
                << 4 as i32) as u32
            & 0xffff as u32) as u32).to_be();
    let fresh65 = n;
    n = n + 1;
    *s.offset(fresh65 as isize) = (GENO_VAL_AIR as i32 as u32).to_be();
    let fresh66 = n;
    n = n + 1;
    *s.offset(fresh66 as isize) = (((GENO_FTCMD_OP << 26 as i32)
        as u32
        | (GENO_SUB_GET as i32 as u32
            & 63 as u32) << 20 as i32
        | (2 as i32 as u32 & 15 as u32)
            << 16 as i32
        | ((((2 as i32 & 3 as i32)
            << 6 as i32
            | 2 as i32 & 63 as i32)
            & 0xff as i32) << 8 as i32
            | (if 0 as i32 != 0 {
                0x80 as i32
            } else {
                0 as i32
            })
            | (0 as i32 & 7 as i32)
                << 4 as i32) as u32
            & 0xffff as u32) as u32).to_be();
    let fresh67 = n;
    n = n + 1;
    *s.offset(fresh67 as isize) = (GENO_VAL_STICK_FWD as i32 as u32).to_be();
    let fresh68 = n;
    n = n + 1;
    *s.offset(fresh68 as isize) = (((GENO_FTCMD_OP << 26 as i32)
        as u32
        | (GENO_SUB_GET as i32 as u32
            & 63 as u32) << 20 as i32
        | (2 as i32 as u32 & 15 as u32)
            << 16 as i32
        | ((((0 as i32 & 3 as i32)
            << 6 as i32
            | 2 as i32 & 63 as i32)
            & 0xff as i32) << 8 as i32
            | (if 0 as i32 != 0 {
                0x80 as i32
            } else {
                0 as i32
            })
            | (0 as i32 & 7 as i32)
                << 4 as i32) as u32
            & 0xffff as u32) as u32).to_be();
    let fresh69 = n;
    n = n + 1;
    *s.offset(fresh69 as isize) = ((GENO_VAL_CMD_VAR0 as i32
        + 2 as i32) as u32).to_be();
    let fresh70 = n;
    n = n + 1;
    *s.offset(fresh70 as isize) = (((GENO_FTCMD_OP << 26 as i32)
        as u32
        | (GENO_SUB_GET as i32 as u32
            & 63 as u32) << 20 as i32
        | (2 as i32 as u32 & 15 as u32)
            << 16 as i32
        | ((((2 as i32 & 3 as i32)
            << 6 as i32
            | 3 as i32 & 63 as i32)
            & 0xff as i32) << 8 as i32
            | (if 0 as i32 != 0 {
                0x80 as i32
            } else {
                0 as i32
            })
            | (0 as i32 & 7 as i32)
                << 4 as i32) as u32
            & 0xffff as u32) as u32).to_be();
    let fresh71 = n;
    n = n + 1;
    *s.offset(fresh71 as isize) = ((GENO_VAL_SPECIAL_F as i32
        + 3 as i32) as u32).to_be();
    let fresh72 = n;
    n = n + 1;
    *s.offset(fresh72 as isize) = (((GENO_FTCMD_OP << 26 as i32)
        as u32
        | (GENO_SUB_GET as i32 as u32
            & 63 as u32) << 20 as i32
        | (2 as i32 as u32 & 15 as u32)
            << 16 as i32
        | ((((0 as i32 & 3 as i32)
            << 6 as i32
            | 3 as i32 & 63 as i32)
            & 0xff as i32) << 8 as i32
            | (if 0 as i32 != 0 {
                0x80 as i32
            } else {
                0 as i32
            })
            | (0 as i32 & 7 as i32)
                << 4 as i32) as u32
            & 0xffff as u32) as u32).to_be();
    let fresh73 = n;
    n = n + 1;
    *s.offset(fresh73 as isize) = ((GENO_VAL_SPECIAL_I as i32
        + 4 as i32) as u32).to_be();
    let fresh74 = n;
    n = n + 1;
    *s.offset(fresh74 as isize) = (((GENO_FTCMD_OP << 26 as i32)
        as u32
        | (GENO_SUB_GET as i32 as u32
            & 63 as u32) << 20 as i32
        | (2 as i32 as u32 & 15 as u32)
            << 16 as i32
        | ((((0 as i32 & 3 as i32)
            << 6 as i32
            | 8 as i32 & 63 as i32)
            & 0xff as i32) << 8 as i32
            | (if 0 as i32 != 0 {
                0x80 as i32
            } else {
                0 as i32
            })
            | (0 as i32 & 7 as i32)
                << 4 as i32) as u32
            & 0xffff as u32) as u32).to_be();
    let fresh75 = n;
    n = n + 1;
    *s.offset(fresh75 as isize) = (GENO_VAL_MOTION as i32 as u32).to_be();
    let fresh76 = n;
    n = n + 1;
    *s.offset(fresh76 as isize) = (((GENO_FTCMD_OP << 26 as i32)
        as u32
        | (GENO_SUB_PUT as i32 as u32
            & 63 as u32) << 20 as i32
        | (3 as i32 as u32 & 15 as u32)
            << 16 as i32
        | 0 as i32 as u32 & 0xffff as u32)
        as u32).to_be();
    let fresh77 = n;
    n = n + 1;
    *s.offset(fresh77 as isize) = (GENO_VAL_VEL_Y as i32 as u32).to_be();
    let fresh78 = n;
    n = n + 1;
    *s.offset(fresh78 as isize) = (t_fbits(3.0f32)).to_be();
    let fresh79 = n;
    n = n + 1;
    *s.offset(fresh79 as isize) = (((GENO_FTCMD_OP << 26 as i32)
        as u32
        | (GENO_SUB_PUT as i32 as u32
            & 63 as u32) << 20 as i32
        | (3 as i32 as u32 & 15 as u32)
            << 16 as i32
        | 0 as i32 as u32 & 0xffff as u32)
        as u32).to_be();
    let fresh80 = n;
    n = n + 1;
    *s.offset(fresh80 as isize) = (GENO_VAL_FWD_VEL as i32 as u32).to_be();
    let fresh81 = n;
    n = n + 1;
    *s.offset(fresh81 as isize) = (t_fbits(2.0f32)).to_be();
    let fresh82 = n;
    n = n + 1;
    *s.offset(fresh82 as isize) = (((GENO_FTCMD_OP << 26 as i32)
        as u32
        | (GENO_SUB_PUT as i32 as u32
            & 63 as u32) << 20 as i32
        | (3 as i32 as u32 & 15 as u32)
            << 16 as i32
        | 0x80 as i32 as u32
            & 0xffff as u32) as u32).to_be();
    let fresh83 = n;
    n = n + 1;
    *s.offset(fresh83 as isize) = (GENO_VAL_CMD_VAR0 as i32 as u32).to_be();
    let fresh84 = n;
    n = n + 1;
    *s.offset(fresh84 as isize) = (((0 as i32 & 3 as i32)
        << 6 as i32 | 3 as i32 & 63 as i32)
        as u32).to_be();
    let fresh85 = n;
    n = n + 1;
    *s.offset(fresh85 as isize) = (((GENO_FTCMD_OP << 26 as i32)
        as u32
        | (GENO_SUB_PUT as i32 as u32
            & 63 as u32) << 20 as i32
        | (3 as i32 as u32 & 15 as u32)
            << 16 as i32
        | 0 as i32 as u32 & 0xffff as u32)
        as u32).to_be();
    let fresh86 = n;
    n = n + 1;
    *s.offset(fresh86 as isize) = (GENO_VAL_FACING as i32 as u32).to_be();
    let fresh87 = n;
    n = n + 1;
    *s.offset(fresh87 as isize) = (t_fbits(0.0f32)).to_be();
    let fresh88 = n;
    n = n + 1;
    *s.offset(fresh88 as isize) = (((GENO_FTCMD_OP << 26 as i32)
        as u32
        | (GENO_SUB_PUT as i32 as u32
            & 63 as u32) << 20 as i32
        | (3 as i32 as u32 & 15 as u32)
            << 16 as i32
        | 0 as i32 as u32 & 0xffff as u32)
        as u32).to_be();
    let fresh89 = n;
    n = n + 1;
    *s.offset(fresh89 as isize) = (GENO_VAL_MOTION as i32 as u32).to_be();
    let fresh90 = n;
    n = n + 1;
    *s.offset(fresh90 as isize) = (5 as u32).to_be();
    let fresh91 = n;
    n = n + 1;
    *s.offset(fresh91 as isize) = (((GENO_FTCMD_OP << 26 as i32)
        as u32
        | (GENO_SUB_IFV as i32 as u32
            & 63 as u32) << 20 as i32
        | (4 as i32 as u32 & 15 as u32)
            << 16 as i32
        | ((GENO_CMP_GT as i32) << 4 as i32)
            as u32 & 0xffff as u32) as u32).to_be();
    let fresh92 = n;
    n = n + 1;
    *s.offset(fresh92 as isize) = (GENO_VAL_STICK_X as i32 as u32).to_be();
    let fresh93 = n;
    n = n + 1;
    *s.offset(fresh93 as isize) = (t_fbits(0.25f32)).to_be();
    let fresh94 = n;
    n = n + 1;
    *s.offset(fresh94 as isize) = (2 as u32).to_be();
    let fresh95 = n;
    n = n + 1;
    *s.offset(fresh95 as isize) = (((GENO_FTCMD_OP << 26 as i32)
        as u32
        | (GENO_SUB_SET as i32 as u32
            & 63 as u32) << 20 as i32
        | (2 as i32 as u32 & 15 as u32)
            << 16 as i32
        | ((((0 as i32 & 3 as i32)
            << 6 as i32
            | 4 as i32 & 63 as i32)
            & 0xff as i32) << 8 as i32
            | (if 0 as i32 != 0 {
                0x80 as i32
            } else {
                0 as i32
            })
            | (0 as i32 & 7 as i32)
                << 4 as i32) as u32
            & 0xffff as u32) as u32).to_be();
    let fresh96 = n;
    n = n + 1;
    *s.offset(fresh96 as isize) = (1 as u32).to_be();
    let fresh97 = n;
    n = n + 1;
    *s.offset(fresh97 as isize) = (((GENO_FTCMD_OP << 26 as i32)
        as u32
        | (GENO_SUB_IFV as i32 as u32
            & 63 as u32) << 20 as i32
        | (4 as i32 as u32 & 15 as u32)
            << 16 as i32
        | ((GENO_CMP_EQ as i32) << 4 as i32)
            as u32 & 0xffff as u32) as u32).to_be();
    let fresh98 = n;
    n = n + 1;
    *s.offset(fresh98 as isize) = (GENO_VAL_AIR as i32 as u32).to_be();
    let fresh99 = n;
    n = n + 1;
    *s.offset(fresh99 as isize) = (0 as u32).to_be();
    let fresh100 = n;
    n = n + 1;
    *s.offset(fresh100 as isize) = (2 as u32).to_be();
    let fresh101 = n;
    n = n + 1;
    *s.offset(fresh101 as isize) = (((GENO_FTCMD_OP << 26 as i32)
        as u32
        | (GENO_SUB_SET as i32 as u32
            & 63 as u32) << 20 as i32
        | (2 as i32 as u32 & 15 as u32)
            << 16 as i32
        | ((((0 as i32 & 3 as i32)
            << 6 as i32
            | 5 as i32 & 63 as i32)
            & 0xff as i32) << 8 as i32
            | (if 0 as i32 != 0 {
                0x80 as i32
            } else {
                0 as i32
            })
            | (0 as i32 & 7 as i32)
                << 4 as i32) as u32
            & 0xffff as u32) as u32).to_be();
    let fresh102 = n;
    n = n + 1;
    *s.offset(fresh102 as isize) = (1 as u32).to_be();
    let fresh103 = n;
    n = n + 1;
    *s.offset(fresh103 as isize) = (((GENO_FTCMD_OP << 26 as i32)
        as u32
        | (GENO_SUB_SET as i32 as u32
            & 63 as u32) << 20 as i32
        | (2 as i32 as u32 & 15 as u32)
            << 16 as i32
        | ((((0 as i32 & 3 as i32)
            << 6 as i32
            | 6 as i32 & 63 as i32)
            & 0xff as i32) << 8 as i32
            | (if 0 as i32 != 0 {
                0x80 as i32
            } else {
                0 as i32
            })
            | (0 as i32 & 7 as i32)
                << 4 as i32) as u32
            & 0xffff as u32) as u32).to_be();
    let fresh104 = n;
    n = n + 1;
    *s.offset(fresh104 as isize) = (10 as u32).to_be();
    let fresh105 = n;
    n = n + 1;
    *s.offset(fresh105 as isize) = (((GENO_FTCMD_OP << 26 as i32)
        as u32
        | (GENO_SUB_DIV as i32 as u32
            & 63 as u32) << 20 as i32
        | (2 as i32 as u32 & 15 as u32)
            << 16 as i32
        | ((((0 as i32 & 3 as i32)
            << 6 as i32
            | 6 as i32 & 63 as i32)
            & 0xff as i32) << 8 as i32
            | (if 0 as i32 != 0 {
                0x80 as i32
            } else {
                0 as i32
            })
            | (0 as i32 & 7 as i32)
                << 4 as i32) as u32
            & 0xffff as u32) as u32).to_be();
    let fresh106 = n;
    n = n + 1;
    *s.offset(fresh106 as isize) = (3 as u32).to_be();
    let fresh107 = n;
    n = n + 1;
    *s.offset(fresh107 as isize) = (((GENO_FTCMD_OP << 26 as i32)
        as u32
        | (GENO_SUB_DIV as i32 as u32
            & 63 as u32) << 20 as i32
        | (2 as i32 as u32 & 15 as u32)
            << 16 as i32
        | ((((0 as i32 & 3 as i32)
            << 6 as i32
            | 6 as i32 & 63 as i32)
            & 0xff as i32) << 8 as i32
            | (if 0 as i32 != 0 {
                0x80 as i32
            } else {
                0 as i32
            })
            | (0 as i32 & 7 as i32)
                << 4 as i32) as u32
            & 0xffff as u32) as u32).to_be();
    let fresh108 = n;
    n = n + 1;
    *s.offset(fresh108 as isize) = (0 as u32).to_be();
    let fresh109 = n;
    n = n + 1;
    *s.offset(fresh109 as isize) = (((GENO_FTCMD_OP << 26 as i32)
        as u32
        | (GENO_SUB_SET as i32 as u32
            & 63 as u32) << 20 as i32
        | (2 as i32 as u32 & 15 as u32)
            << 16 as i32
        | ((((2 as i32 & 3 as i32)
            << 6 as i32
            | 4 as i32 & 63 as i32)
            & 0xff as i32) << 8 as i32
            | (if 0 as i32 != 0 {
                0x80 as i32
            } else {
                0 as i32
            })
            | (0 as i32 & 7 as i32)
                << 4 as i32) as u32
            & 0xffff as u32) as u32).to_be();
    let fresh110 = n;
    n = n + 1;
    *s.offset(fresh110 as isize) = (t_fbits(5.0f32)).to_be();
    let fresh111 = n;
    n = n + 1;
    *s.offset(fresh111 as isize) = (((GENO_FTCMD_OP << 26 as i32)
        as u32
        | (GENO_SUB_DIV as i32 as u32
            & 63 as u32) << 20 as i32
        | (2 as i32 as u32 & 15 as u32)
            << 16 as i32
        | ((((2 as i32 & 3 as i32)
            << 6 as i32
            | 4 as i32 & 63 as i32)
            & 0xff as i32) << 8 as i32
            | (if 0 as i32 != 0 {
                0x80 as i32
            } else {
                0 as i32
            })
            | (0 as i32 & 7 as i32)
                << 4 as i32) as u32
            & 0xffff as u32) as u32).to_be();
    let fresh112 = n;
    n = n + 1;
    *s.offset(fresh112 as isize) = (t_fbits(2.0f32)).to_be();
    let fresh113 = n;
    n = n + 1;
    *s.offset(fresh113 as isize) = (((GENO_FTCMD_OP << 26 as i32)
        as u32
        | (GENO_SUB_RAND as i32 as u32
            & 63 as u32) << 20 as i32
        | (2 as i32 as u32 & 15 as u32)
            << 16 as i32
        | ((((0 as i32 & 3 as i32)
            << 6 as i32
            | 7 as i32 & 63 as i32)
            & 0xff as i32) << 8 as i32
            | (if 0 as i32 != 0 {
                0x80 as i32
            } else {
                0 as i32
            })
            | (0 as i32 & 7 as i32)
                << 4 as i32) as u32
            & 0xffff as u32) as u32).to_be();
    let fresh114 = n;
    n = n + 1;
    *s.offset(fresh114 as isize) = (5 as u32).to_be();
    let fresh115 = n;
    n = n + 1;
    *s.offset(fresh115 as isize) = (0 as u32).to_be();
    t_run(&raw mut t_script as *mut u32, GENO_MODE_EXEC);
    if !t_fp.x3E4_fighterCmdScript.anon_8.u.is_null() {
        TestFail(
            b"the v1 value script did not reach End\0" as *const u8
                as *const i8,
        );
        return 1 as i32;
    }
    if (*st).la_f[1 as i32 as usize] != 1.5f32
        || (*st).la_i[1 as i32 as usize] != 1 as i32
        || (*st).la_f[2 as i32 as usize] != -0.5f32
        || (*st).la_i[2 as i32 as usize] != 7 as i32
        || (*st).la_f[3 as i32 as usize] != 2.5f32
        || (*st).la_i[3 as i32 as usize] != 9 as i32
        || (*st).la_i[8 as i32 as usize] != 123 as i32
    {
        TestFail(
            b"GET: expected VEL_X 1.5, AIR 1, STICK_FWD -0.5, CMD_VAR2 7, special 2.5 / 9, MOTION 123\0"
                as *const u8 as *const i8,
        );
        return 1 as i32;
    }
    if t_fp.self_vel.y != 3.0f32 || t_fp.self_vel.x != -2.0f32
        || t_fp.cmd_vars[0 as i32 as usize] != 9 as u32
        || t_fp.facing_dir != 1.0f32 || t_fp.motion_id != 123 as i32
    {
        TestFail(
            b"PUT: expected VEL_Y 3, VEL_X -2 (forward 2 facing left), CMD_VAR0 9, facing +1, motion untouched\0"
                as *const u8 as *const i8,
        );
        return 1 as i32;
    }
    if (*st).la_i[4 as i32 as usize] != 1 as i32
        || (*st).la_i[5 as i32 as usize] != 0 as i32
    {
        TestFail(
            b"IFV: stick > 0.25 should run, AIR == 0 should skip\0" as *const u8
                as *const i8,
        );
        return 1 as i32;
    }
    if (*st).la_i[6 as i32 as usize] != 3 as i32
        || (*st).la_f[4 as i32 as usize] != 2.5f32
        || (*st).la_i[7 as i32 as usize] < 0 as i32
        || (*st).la_i[7 as i32 as usize] > 4 as i32
    {
        TestFail(
            b"DIV / RAND results wrong\0" as *const u8 as *const i8,
        );
        return 1 as i32;
    }
    return 0 as i32;
}

pub unsafe extern "C" fn test_geno_v1_change_action() -> i32 {
    melee_core::ct!("test_geno_v1_change_action");
    let mut st: *mut TestGenoState = t_setup();
    let mut n: i32 = 0 as i32;
    let mut rc: i32 = 0 as i32;
    let mut i: i32 = 0;
    let mut s: *mut u32 = &raw mut t_script as *mut u32;
    GenoGame_TestCapture(1 as i32, 0 as i32);
    t_fp.motion_id = 50 as i32 as FtMotionId;
    t_fp.ground_or_air = GA_Air;
    let fresh116 = n;
    n = n + 1;
    *s.offset(fresh116 as isize) = (((GENO_FTCMD_OP << 26 as i32)
        as u32
        | (GENO_SUB_CHG as i32 as u32
            & 63 as u32) << 20 as i32
        | (3 as i32 as u32 & 15 as u32)
            << 16 as i32
        | ((GENO_COND_PRESSED as i32 & 0xff as i32)
            << 8 as i32
            | (if 0 as i32 != 0 {
                0x80 as i32
            } else {
                0 as i32
            })
            | (0 as i32 & 7 as i32)
                << 4 as i32
            | 0 as i32 & 0xc as i32) as u32
            & 0xffff as u32) as u32).to_be();
    let fresh117 = n;
    n = n + 1;
    *s.offset(fresh117 as isize) = (((0 as u32
        & 15 as u32) << 28 as i32
        | 60 as i32 as u32
            & 0xffff as u32) as u32).to_be();
    let fresh118 = n;
    n = n + 1;
    *s.offset(fresh118 as isize) = (GENO_BTN_ATTACK as u32).to_be();
    let fresh119 = n;
    n = n + 1;
    *s.offset(fresh119 as isize) = (((GENO_FTCMD_OP << 26 as i32)
        as u32
        | (GENO_SUB_CHG as i32 as u32
            & 63 as u32) << 20 as i32
        | (4 as i32 as u32 & 15 as u32)
            << 16 as i32
        | ((GENO_COND_BIT as i32 & 0xff as i32)
            << 8 as i32
            | (if 0 as i32 != 0 {
                0x80 as i32
            } else {
                0 as i32
            })
            | (0 as i32 & 7 as i32)
                << 4 as i32
            | 0 as i32 & 0xc as i32) as u32
            & 0xffff as u32) as u32).to_be();
    let fresh120 = n;
    n = n + 1;
    *s.offset(fresh120 as isize) = (((1 as u32
        & 15 as u32) << 28 as i32
        | 2 as i32 as u32 & 0xffff as u32)
        as u32).to_be();
    let fresh121 = n;
    n = n + 1;
    *s.offset(fresh121 as isize) = (((1 as i32 & 3 as i32)
        << 6 as i32 | 0 as i32 & 63 as i32)
        as u32).to_be();
    let fresh122 = n;
    n = n + 1;
    *s.offset(fresh122 as isize) = (3 as u32).to_be();
    let fresh123 = n;
    n = n + 1;
    *s.offset(fresh123 as isize) = (((GENO_FTCMD_OP << 26 as i32)
        as u32
        | (GENO_SUB_CHGAND as i32 as u32
            & 63 as u32) << 20 as i32
        | (2 as i32 as u32 & 15 as u32)
            << 16 as i32
        | ((GENO_COND_FRAME as i32 & 0xff as i32)
            << 8 as i32
            | (if 0 as i32 != 0 {
                0x80 as i32
            } else {
                0 as i32
            })
            | (0 as i32 & 7 as i32)
                << 4 as i32
            | 0 as i32 & 0xc as i32) as u32
            & 0xffff as u32) as u32).to_be();
    let fresh124 = n;
    n = n + 1;
    *s.offset(fresh124 as isize) = (10 as u32).to_be();
    let fresh125 = n;
    n = n + 1;
    *s.offset(fresh125 as isize) = (((GENO_FTCMD_OP << 26 as i32)
        as u32
        | (GENO_SUB_CHG as i32 as u32
            & 63 as u32) << 20 as i32
        | (2 as i32 as u32 & 15 as u32)
            << 16 as i32
        | ((GENO_COND_ANIM_END as i32 & 0xff as i32)
            << 8 as i32
            | (if 0 as i32 != 0 {
                0x80 as i32
            } else {
                0 as i32
            })
            | (0 as i32 & 7 as i32)
                << 4 as i32
            | 0 as i32 & 0xc as i32) as u32
            & 0xffff as u32) as u32).to_be();
    let fresh126 = n;
    n = n + 1;
    *s.offset(fresh126 as isize) = (((0 as u32
        & 15 as u32) << 28 as i32
        | 14 as i32 as u32
            & 0xffff as u32) as u32).to_be();
    let fresh127 = n;
    n = n + 1;
    *s.offset(fresh127 as isize) = (((GENO_FTCMD_OP << 26 as i32)
        as u32
        | (GENO_SUB_CHG as i32 as u32
            & 63 as u32) << 20 as i32
        | (2 as i32 as u32 & 15 as u32)
            << 16 as i32
        | ((GENO_COND_ALWAYS as i32 & 0xff as i32)
            << 8 as i32
            | (if 0 as i32 != 0 {
                0x80 as i32
            } else {
                0 as i32
            })
            | (0 as i32 & 7 as i32)
                << 4 as i32
            | (0x8 as i32 | 0x4 as i32)
                & 0xc as i32) as u32
            & 0xffff as u32) as u32).to_be();
    let fresh128 = n;
    n = n + 1;
    *s.offset(fresh128 as isize) = (((0 as u32
        & 15 as u32) << 28 as i32
        | 70 as i32 as u32
            & 0xffff as u32) as u32).to_be();
    let fresh129 = n;
    n = n + 1;
    *s.offset(fresh129 as isize) = (0 as u32).to_be();
    t_run(&raw mut t_script as *mut u32, GENO_MODE_EXEC);
    if (*st).nchecks != 4 as u32
        || (*st).checks[1 as i32 as usize].ncond != 2 as u32
    {
        TestFail(
            b"expected 4 checks registered, the second with 2 conditions\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    t_run(&raw mut t_script as *mut u32, GENO_MODE_EXEC);
    if (*st).nchecks != 4 as u32
        || (*st).checks[1 as i32 as usize].ncond != 2 as u32
    {
        TestFail(
            b"a re-run CHG / CHGAND must not register twice\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    t_fp.cur_anim_frame = 5.0f32;
    if Geno_PreAnim(&raw mut t_gobj) != 0 as i32
        || (*st).nchecks != 3 as u32 || (*st).action_time != 1 as i32
    {
        TestFail(
            b"frame 1: no change expected, the ONCE check dropped, action frame 1\0"
                as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    (*st).ra_i[0 as i32 as usize] = ((1 as i32)
        << 3 as i32) as i32;
    if Geno_PreAnim(&raw mut t_gobj) != 0 as i32 {
        TestFail(
            b"CHGAND FRAME >= 10 must hold the change back at frame 5\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    t_fp.cur_anim_frame = 10.0f32;
    if Geno_PreAnim(&raw mut t_gobj) != 1 as i32
        || GenoGame_TestLastTarget()
            != (1 as u32 & 15 as u32)
                << 28 as i32
                | 2 as i32 as u32
                    & 0xffff as u32
        || t_fp.motion_id != 343 as i32 || (*st).nchecks != 0 as u32
        || (*st).ra_i[0 as i32 as usize] != 0 as i32
        || (*st).action_time != 0 as i32
    {
        TestFail(
            b"bit + frame: expected special 2 (motion 343), checks and RA cleared\0"
                as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    n = 0 as i32;
    let fresh130 = n;
    n = n + 1;
    *s.offset(fresh130 as isize) = (((GENO_FTCMD_OP << 26 as i32)
        as u32
        | (GENO_SUB_CHG as i32 as u32
            & 63 as u32) << 20 as i32
        | (3 as i32 as u32 & 15 as u32)
            << 16 as i32
        | ((GENO_COND_PRESSED as i32 & 0xff as i32)
            << 8 as i32
            | (if 0 as i32 != 0 {
                0x80 as i32
            } else {
                0 as i32
            })
            | (0 as i32 & 7 as i32)
                << 4 as i32
            | 0 as i32 & 0xc as i32) as u32
            & 0xffff as u32) as u32).to_be();
    let fresh131 = n;
    n = n + 1;
    *s.offset(fresh131 as isize) = (((0 as u32
        & 15 as u32) << 28 as i32
        | 60 as i32 as u32
            & 0xffff as u32) as u32).to_be();
    let fresh132 = n;
    n = n + 1;
    *s.offset(fresh132 as isize) = ((GENO_BTN_ATTACK | GENO_BTN_SPECIAL) as u32).to_be();
    let fresh133 = n;
    n = n + 1;
    *s.offset(fresh133 as isize) = (0 as u32).to_be();
    t_run(&raw mut t_script as *mut u32, GENO_MODE_EXEC);
    t_fp.input.pressed_buttons = HSD_PAD_B as HSD_Pad;
    if Geno_PreAnim(&raw mut t_gobj) != 1 as i32
        || t_fp.motion_id != 60 as i32
    {
        TestFail(
            b"PRESSED(ATTACK|SPECIAL) with B pressed should change to 60\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    t_fp.input.pressed_buttons = 0 as HSD_Pad;
    n = 0 as i32;
    let fresh134 = n;
    n = n + 1;
    *s.offset(fresh134 as isize) = (((GENO_FTCMD_OP << 26 as i32)
        as u32
        | (GENO_SUB_CHG as i32 as u32
            & 63 as u32) << 20 as i32
        | (2 as i32 as u32 & 15 as u32)
            << 16 as i32
        | ((GENO_COND_ANIM_END as i32 & 0xff as i32)
            << 8 as i32
            | (if 0 as i32 != 0 {
                0x80 as i32
            } else {
                0 as i32
            })
            | (0 as i32 & 7 as i32)
                << 4 as i32
            | 0 as i32 & 0xc as i32) as u32
            & 0xffff as u32) as u32).to_be();
    let fresh135 = n;
    n = n + 1;
    *s.offset(fresh135 as isize) = (((0 as u32
        & 15 as u32) << 28 as i32
        | 14 as i32 as u32
            & 0xffff as u32) as u32).to_be();
    let fresh136 = n;
    n = n + 1;
    *s.offset(fresh136 as isize) = (0 as u32).to_be();
    t_run(&raw mut t_script as *mut u32, GENO_MODE_EXEC);
    if Geno_PreAnim(&raw mut t_gobj) != 0 as i32 {
        TestFail(
            b"ANIM_END must wait for the animation's end\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    GenoGame_TestCapture(1 as i32, 1 as i32);
    if Geno_PreAnim(&raw mut t_gobj) != 1 as i32
        || t_fp.motion_id != 14 as i32
    {
        TestFail(
            b"ANIM_END at the end should change to 14\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    GenoGame_TestCapture(1 as i32, 0 as i32);
    n = 0 as i32;
    let fresh137 = n;
    n = n + 1;
    *s.offset(fresh137 as isize) = (((GENO_FTCMD_OP << 26 as i32)
        as u32
        | (GENO_SUB_CHG as i32 as u32
            & 63 as u32) << 20 as i32
        | (4 as i32 as u32 & 15 as u32)
            << 16 as i32
        | ((GENO_COND_VALUE as i32 & 0xff as i32)
            << 8 as i32
            | (if 0 as i32 != 0 {
                0x80 as i32
            } else {
                0 as i32
            })
            | (GENO_CMP_GE as i32 & 7 as i32)
                << 4 as i32
            | 0x4 as i32 & 0xc as i32)
            as u32 & 0xffff as u32) as u32).to_be();
    let fresh138 = n;
    n = n + 1;
    *s.offset(fresh138 as isize) = (((0 as u32
        & 15 as u32) << 28 as i32
        | 80 as i32 as u32
            & 0xffff as u32) as u32).to_be();
    let fresh139 = n;
    n = n + 1;
    *s.offset(fresh139 as isize) = (GENO_VAL_ACTION_FRAME as i32 as u32).to_be();
    let fresh140 = n;
    n = n + 1;
    *s.offset(fresh140 as isize) = (1 as u32).to_be();
    let fresh141 = n;
    n = n + 1;
    *s.offset(fresh141 as isize) = (0 as u32).to_be();
    t_run(&raw mut t_script as *mut u32, GENO_MODE_SKIP);
    if (*st).nchecks != 0 as u32 {
        TestFail(
            b"a ONCE check must not be registered by the fast-forward pass\0"
                as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    t_run(&raw mut t_script as *mut u32, GENO_MODE_EXEC);
    if Geno_PreAnim(&raw mut t_gobj) != 1 as i32
        || t_fp.motion_id != 80 as i32
    {
        TestFail(
            b"VALUE ACTION_FRAME >= 1 ONCE should change to 80\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    n = 0 as i32;
    let fresh142 = n;
    n = n + 1;
    *s.offset(fresh142 as isize) = (((GENO_FTCMD_OP << 26 as i32)
        as u32
        | (GENO_SUB_CHG as i32 as u32
            & 63 as u32) << 20 as i32
        | (4 as i32 as u32 & 15 as u32)
            << 16 as i32
        | ((GENO_COND_VAR as i32 & 0xff as i32)
            << 8 as i32
            | (if 0 as i32 != 0 {
                0x80 as i32
            } else {
                0 as i32
            })
            | (GENO_CMP_GT as i32 & 7 as i32)
                << 4 as i32
            | 0 as i32 & 0xc as i32) as u32
            & 0xffff as u32) as u32).to_be();
    let fresh143 = n;
    n = n + 1;
    *s.offset(fresh143 as isize) = (((0 as u32
        & 15 as u32) << 28 as i32
        | 81 as i32 as u32
            & 0xffff as u32) as u32).to_be();
    let fresh144 = n;
    n = n + 1;
    *s.offset(fresh144 as isize) = (((2 as i32 & 3 as i32)
        << 6 as i32 | 0 as i32 & 63 as i32)
        as u32).to_be();
    let fresh145 = n;
    n = n + 1;
    *s.offset(fresh145 as isize) = (t_fbits(1.5f32)).to_be();
    let fresh146 = n;
    n = n + 1;
    *s.offset(fresh146 as isize) = (0 as u32).to_be();
    t_run(&raw mut t_script as *mut u32, GENO_MODE_EXEC);
    (*st).la_f[0 as i32 as usize] = 1.0f32 as f32;
    if Geno_PreAnim(&raw mut t_gobj) != 0 as i32 {
        TestFail(
            b"VAR 1.0 > 1.5 must be false\0" as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    (*st).la_f[0 as i32 as usize] = 2.0f32 as f32;
    if Geno_PreAnim(&raw mut t_gobj) != 1 as i32
        || t_fp.motion_id != 81 as i32
    {
        TestFail(
            b"VAR 2.0 > 1.5 should change to 81\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    n = 0 as i32;
    let fresh147 = n;
    n = n + 1;
    *s.offset(fresh147 as isize) = (((GENO_FTCMD_OP << 26 as i32)
        as u32
        | (GENO_SUB_CHG as i32 as u32
            & 63 as u32) << 20 as i32
        | (2 as i32 as u32 & 15 as u32)
            << 16 as i32
        | ((GENO_COND_ALWAYS as i32 & 0xff as i32)
            << 8 as i32
            | (if 0 as i32 != 0 {
                0x80 as i32
            } else {
                0 as i32
            })
            | (0 as i32 & 7 as i32)
                << 4 as i32
            | 0 as i32 & 0xc as i32) as u32
            & 0xffff as u32) as u32).to_be();
    let fresh148 = n;
    n = n + 1;
    *s.offset(fresh148 as isize) = (((2 as u32
        & 15 as u32) << 28 as i32
        | 1 as i32 as u32 & 0xffff as u32)
        as u32).to_be();
    let fresh149 = n;
    n = n + 1;
    *s.offset(fresh149 as isize) = (0 as u32).to_be();
    t_run(&raw mut t_script as *mut u32, GENO_MODE_EXEC);
    i = GenoGame_TestChanges();
    if Geno_PreAnim(&raw mut t_gobj) != 0 as i32
        || GenoGame_TestChanges() != i || (*st).nchecks != 1 as u32
    {
        TestFail(
            b"a Geno-state target must be ignored in v1\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    *s.offset(0 as i32 as isize) = (((GENO_FTCMD_OP
        << 26 as i32) as u32
        | (GENO_SUB_CHG as i32 as u32
            & 63 as u32) << 20 as i32
        | (2 as i32 as u32 & 15 as u32)
            << 16 as i32
        | ((GENO_COND_ALWAYS as i32 & 0xff as i32)
            << 8 as i32
            | (if 0 as i32 != 0 {
                0x80 as i32
            } else {
                0 as i32
            })
            | (0 as i32 & 7 as i32)
                << 4 as i32
            | 0 as i32 & 0xc as i32) as u32
            & 0xffff as u32) as u32).to_be();
    *s.offset(1 as i32 as isize) = (((0 as u32
        & 15 as u32) << 28 as i32
        | 35 as i32 as u32
            & 0xffff as u32) as u32).to_be();
    *s.offset(2 as i32 as isize) = (0 as u32).to_be();
    t_run(&raw mut t_script as *mut u32, GENO_MODE_EXEC);
    if Geno_PreAnim(&raw mut t_gobj) != 1 as i32
        || t_fp.motion_id != 35 as i32
    {
        TestFail(
            b"a fallback check after a Geno-state check must still fire in v1\0"
                as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    n = 0 as i32;
    let fresh150 = n;
    n = n + 1;
    *s.offset(fresh150 as isize) = (((GENO_FTCMD_OP << 26 as i32)
        as u32
        | (GENO_SUB_CHG as i32 as u32
            & 63 as u32) << 20 as i32
        | (2 as i32 as u32 & 15 as u32)
            << 16 as i32
        | ((GENO_COND_ALWAYS as i32 & 0xff as i32)
            << 8 as i32
            | (if 0 as i32 != 0 {
                0x80 as i32
            } else {
                0 as i32
            })
            | (0 as i32 & 7 as i32)
                << 4 as i32
            | 0 as i32 & 0xc as i32) as u32
            & 0xffff as u32) as u32).to_be();
    let fresh151 = n;
    n = n + 1;
    *s.offset(fresh151 as isize) = (((2 as u32
        & 15 as u32) << 28 as i32
        | 1 as i32 as u32 & 0xffff as u32)
        as u32).to_be();
    let fresh152 = n;
    n = n + 1;
    *s.offset(fresh152 as isize) = (0 as u32).to_be();
    t_run(&raw mut t_script as *mut u32, GENO_MODE_EXEC);
    t_script[0 as i32 as usize] = (((GENO_FTCMD_OP
        << 26 as i32) as u32
        | (GENO_SUB_CHGCLR as i32 as u32
            & 63 as u32) << 20 as i32
        | (1 as i32 as u32 & 15 as u32)
            << 16 as i32
        | 0 as i32 as u32 & 0xffff as u32)
        as u32).to_be();
    t_script[1 as i32 as usize] = (0 as u32).to_be();
    t_run(&raw mut t_script as *mut u32, GENO_MODE_EXEC);
    if (*st).nchecks != 0 as u32 {
        TestFail(
            b"CHGCLR must drop every check\0" as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    n = 0 as i32;
    i = 0 as i32;
    while i < 9 as i32 {
        let fresh153 = n;
        n = n + 1;
        *s.offset(fresh153 as isize) = (((GENO_FTCMD_OP << 26 as i32)
            as u32
            | (GENO_SUB_CHG as i32 as u32
                & 63 as u32) << 20 as i32
            | (3 as i32 as u32
                & 15 as u32) << 16 as i32
            | ((GENO_COND_HELD as i32 & 0xff as i32)
                << 8 as i32
                | (if 0 as i32 != 0 {
                    0x80 as i32
                } else {
                    0 as i32
                })
                | (0 as i32 & 7 as i32)
                    << 4 as i32
                | 0 as i32 & 0xc as i32)
                as u32 & 0xffff as u32) as u32).to_be();
        let fresh154 = n;
        n = n + 1;
        *s.offset(fresh154 as isize) = (((0 as u32
            & 15 as u32) << 28 as i32
            | (90 as i32 + i) as u32
                & 0xffff as u32) as u32).to_be();
        let fresh155 = n;
        n = n + 1;
        *s.offset(fresh155 as isize) = (GENO_BTN_TAUNT as u32).to_be();
        i += 1;
    }
    let fresh156 = n;
    n = n + 1;
    *s.offset(fresh156 as isize) = (0 as u32).to_be();
    t_run(&raw mut t_script as *mut u32, GENO_MODE_EXEC);
    if (*st).nchecks != GENO_MAX_CHECKS as u32 {
        TestFail(
            b"the 9th check must be dropped (8 max)\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    GenoGame_TestCapture(0 as i32, 0 as i32);
    return rc;
}

pub unsafe extern "C" fn test_geno_v1_ground_edge() -> i32 {
    melee_core::ct!("test_geno_v1_ground_edge");
    let mut st: *mut TestGenoState = t_setup();
    let mut rc: i32 = 0 as i32;
    let mut s: *mut u32 = &raw mut t_script as *mut u32;
    GenoGame_TestCapture(1 as i32, 0 as i32);
    t_fp.motion_id = 50 as i32 as FtMotionId;
    *s.offset(0 as i32 as isize) = (((GENO_FTCMD_OP
        << 26 as i32) as u32
        | (GENO_SUB_CHG as i32 as u32
            & 63 as u32) << 20 as i32
        | (2 as i32 as u32 & 15 as u32)
            << 16 as i32
        | ((GENO_COND_GROUND as i32 & 0xff as i32)
            << 8 as i32
            | (if 0 as i32 != 0 {
                0x80 as i32
            } else {
                0 as i32
            })
            | (0 as i32 & 7 as i32)
                << 4 as i32
            | 0 as i32 & 0xc as i32) as u32
            & 0xffff as u32) as u32).to_be();
    *s.offset(1 as i32 as isize) = (((0 as u32
        & 15 as u32) << 28 as i32
        | 43 as i32 as u32
            & 0xffff as u32) as u32).to_be();
    *s.offset(2 as i32 as isize) = (((GENO_FTCMD_OP
        << 26 as i32) as u32
        | (GENO_SUB_CHG as i32 as u32
            & 63 as u32) << 20 as i32
        | (2 as i32 as u32 & 15 as u32)
            << 16 as i32
        | ((GENO_COND_AIR as i32 & 0xff as i32)
            << 8 as i32
            | (if 0 as i32 != 0 {
                0x80 as i32
            } else {
                0 as i32
            })
            | (0 as i32 & 7 as i32)
                << 4 as i32
            | 0 as i32 & 0xc as i32) as u32
            & 0xffff as u32) as u32).to_be();
    *s.offset(3 as i32 as isize) = (((0 as u32
        & 15 as u32) << 28 as i32
        | 30 as i32 as u32
            & 0xffff as u32) as u32).to_be();
    *s.offset(4 as i32 as isize) = (0 as u32).to_be();
    t_run(&raw mut t_script as *mut u32, GENO_MODE_EXEC);
    t_fp.ground_or_air = GA_Ground;
    Geno_GroundEdge(&raw mut t_fp, 1 as i32);
    if (*st).edge_pending != 0 || GenoGame_TestChanges() != 0 as i32 {
        TestFail(
            b"an edge outside the collision callback must not pick a target\0"
                as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    Geno_CollBegin(&raw mut t_gobj);
    Geno_GroundEdge(&raw mut t_fp, 1 as i32);
    t_fp.motion_id = 42 as i32 as FtMotionId;
    Geno_CollEnd(&raw mut t_gobj);
    if GenoGame_TestLastTarget()
        != (0 as u32 & 15 as u32)
            << 28 as i32
            | 43 as i32 as u32
                & 0xffff as u32
        || t_fp.motion_id != 43 as i32 || (*st).in_coll != 0 as u32
    {
        TestFail(
            b"landing in the callback: expected the GROUND check's target 43 after it\0"
                as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    t_run(&raw mut t_script as *mut u32, GENO_MODE_EXEC);
    t_fp.ground_or_air = GA_Air;
    Geno_CollBegin(&raw mut t_gobj);
    Geno_GroundEdge(&raw mut t_fp, 0 as i32);
    Geno_CollEnd(&raw mut t_gobj);
    if t_fp.motion_id != 30 as i32 {
        TestFail(
            b"leaving the ground in the callback: expected the AIR check's target 30\0"
                as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    if Geno_TestInstall(
        b"{\"geno\":1,\"fighters\":[{\"attach\":\"kirby\",\"on_land\":[{\"from\":\"special:3\",\"to\":42,\"keep_frame\":true},{\"from\":66,\"to\":\"special:5\"}],\"hooks\":{\"on_land\":[\"geno.count_frames:9\"]}}]}\0"
            as *const u8 as *const i8,
    ) != 1 as i32
    {
        TestFail(
            b"could not install the on_land profile\0" as *const u8
                as *const i8,
        );
        Geno_TestRestore();
        GenoGame_TestCapture(0 as i32, 0 as i32);
        return 1 as i32;
    }
    st = t_setup();
    GenoGame_TestCapture(1 as i32, 0 as i32);
    t_fp.motion_id = 344 as i32 as FtMotionId;
    t_fp.ground_or_air = GA_Ground;
    Geno_CollBegin(&raw mut t_gobj);
    Geno_GroundEdge(&raw mut t_fp, 1 as i32);
    Geno_CollEnd(&raw mut t_gobj);
    if GenoGame_TestLastTarget()
        != (0 as u32 & 15 as u32)
            << 28 as i32
            | 42 as i32 as u32
                & 0xffff as u32 | GENO_TGT_KEEP_FRAME
        || (*st).la_i[9 as i32 as usize] != 1 as i32
    {
        TestFail(
            b"on_land special:3 -> 42 keep_frame, and the on_land hook once\0"
                as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    t_fp.motion_id = 66 as i32 as FtMotionId;
    Geno_CollBegin(&raw mut t_gobj);
    Geno_GroundEdge(&raw mut t_fp, 1 as i32);
    Geno_CollEnd(&raw mut t_gobj);
    if t_fp.motion_id != 346 as i32
        || (*st).la_i[9 as i32 as usize] != 2 as i32
    {
        TestFail(
            b"on_land 66 -> special:5 (motion 346)\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    t_fp.motion_id = 67 as i32 as FtMotionId;
    Geno_CollBegin(&raw mut t_gobj);
    Geno_GroundEdge(&raw mut t_fp, 1 as i32);
    Geno_CollEnd(&raw mut t_gobj);
    if t_fp.motion_id != 67 as i32 {
        TestFail(
            b"a landing from a motion with no on_land entry must change nothing\0"
                as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    Geno_TestRestore();
    GenoGame_TestCapture(0 as i32, 0 as i32);
    return rc;
}

pub unsafe extern "C" fn test_geno_v1_rehit() -> i32 {
    melee_core::ct!("test_geno_v1_rehit");
    let mut st: *mut TestGenoState = t_setup();
    let mut rc: i32 = 0 as i32;
    let mut f: i32 = 0;
    t_fp
        .x914[1 as i32 as usize]
        .victims_1[0 as i32 as usize]
        .victim = &raw mut t_gobj as *mut c_void;
    t_fp.x914[1 as i32 as usize].x44 = 1 as u8;
    t_fp
        .x914[0 as i32 as usize]
        .victims_1[0 as i32 as usize]
        .victim = &raw mut t_gobj as *mut c_void;
    t_fp.x914[0 as i32 as usize].x44 = 1 as u8;
    t_script[0 as i32 as usize] = (((GENO_FTCMD_OP
        << 26 as i32) as u32
        | (GENO_SUB_REHIT as i32 as u32
            & 63 as u32) << 20 as i32
        | (2 as i32 as u32 & 15 as u32)
            << 16 as i32
        | ((0x2 as i32) << 8 as i32) as u32
            & 0xffff as u32) as u32).to_be();
    t_script[1 as i32 as usize] = (3 as u32).to_be();
    t_script[2 as i32 as usize] = (0 as u32).to_be();
    t_run(&raw mut t_script as *mut u32, GENO_MODE_EXEC);
    f = 1 as i32;
    while f <= 2 as i32 {
        Geno_PreAnim(&raw mut t_gobj);
        f += 1;
    }
    if t_fp.x914[1 as i32 as usize].x44 as i32
        != 1 as i32
    {
        TestFail(
            b"rehit 3: the hit list must survive frames 1-2\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    Geno_PreAnim(&raw mut t_gobj);
    if t_fp.x914[1 as i32 as usize].x44 as i32
        != 0 as i32
        || !t_fp
            .x914[1 as i32 as usize]
            .victims_1[0 as i32 as usize]
            .victim
            .is_null()
    {
        TestFail(
            b"rehit 3: the hit list must be cleared on frame 3\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    if t_fp.x914[0 as i32 as usize].x44 as i32
        != 1 as i32
    {
        TestFail(
            b"rehit must leave the other hitboxes' lists alone\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    t_fp.x914[1 as i32 as usize].x44 = 1 as u8;
    Geno_PreAnim(&raw mut t_gobj);
    Geno_PreAnim(&raw mut t_gobj);
    if t_fp.x914[1 as i32 as usize].x44 as i32
        != 1 as i32
    {
        TestFail(
            b"rehit 3: cleared again too early\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    Geno_PreAnim(&raw mut t_gobj);
    if t_fp.x914[1 as i32 as usize].x44 as i32
        != 0 as i32
    {
        TestFail(
            b"rehit 3: not cleared again on frame 6\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    Geno_OnActionChange(&raw mut t_gobj);
    if (*st).rehit_period[1 as i32 as usize] != 0 as i32 {
        TestFail(
            b"an action change must stop the rehit timer\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    return rc;
}

pub unsafe extern "C" fn test_geno_v1_autolink() -> i32 {
    melee_core::ct!("test_geno_v1_autolink");
    let mut st: *mut TestGenoState = ::core::ptr::null_mut::<TestGenoState>();
    let mut rc: i32 = 0 as i32;
    let mut dir: f32 = 1.0f32;
    let mut angle: f32 = 361.0f32;
    let mut kb: f32 = 10.0f32;
    t_setup();
    if Geno_Autolink(
        &raw mut t_fp,
        (&raw mut t_fp.x914 as *mut HitCapsule).offset(0 as i32 as isize)
            as *mut HitCapsule,
        &raw mut dir,
        &raw mut angle,
        &raw mut kb,
    ) != 0 as i32 || angle != 361.0f32
    {
        TestFail(
            b"an untouched fighter must keep Melee's angle\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    st = t_setup();
    t_script[0 as i32 as usize] = (((GENO_FTCMD_OP
        << 26 as i32) as u32
        | (GENO_SUB_LINK as i32 as u32
            & 63 as u32) << 20 as i32
        | (2 as i32 as u32 & 15 as u32)
            << 16 as i32
        | ((0x1 as i32) << 8 as i32) as u32
            & 0xffff as u32) as u32).to_be();
    t_script[1 as i32 as usize] = (GENO_LINK_DIRECTION
        as i32 as u32).to_be();
    t_script[2 as i32 as usize] = (0 as u32).to_be();
    t_run(&raw mut t_script as *mut u32, GENO_MODE_EXEC);
    t_fp.ground_or_air = GA_Air;
    t_fp.self_vel.x = -1.0f32 as f32;
    t_fp.self_vel.y = 1.0f32 as f32;
    if Geno_Autolink(
        &raw mut t_fp,
        (&raw mut t_fp.x914 as *mut HitCapsule).offset(0 as i32 as isize)
            as *mut HitCapsule,
        &raw mut dir,
        &raw mut angle,
        &raw mut kb,
    ) != 1 as i32 || dir != 1.0f32 || angle != 45.0f32 || kb != 10.0f32
    {
        TestFail(
            b"up-left momentum: expected dir +1 (launch left), angle 45, kb unchanged\0"
                as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    dir = 1.0f32;
    angle = 361.0f32;
    if Geno_Autolink(
        &raw mut t_fp,
        (&raw mut t_fp.x914 as *mut HitCapsule).offset(1 as i32 as isize)
            as *mut HitCapsule,
        &raw mut dir,
        &raw mut angle,
        &raw mut kb,
    ) != 0 as i32 || angle != 361.0f32
    {
        TestFail(
            b"a hitbox without LINK must keep its angle\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    t_fp.self_vel.x = 2.0f32 as f32;
    t_fp.self_vel.y = -2.0f32 as f32;
    if Geno_Autolink(
        &raw mut t_fp,
        (&raw mut t_fp.x914 as *mut HitCapsule).offset(0 as i32 as isize)
            as *mut HitCapsule,
        &raw mut dir,
        &raw mut angle,
        &raw mut kb,
    ) != 1 as i32 || dir != -1.0f32 || angle != 315.0f32
    {
        TestFail(
            b"down-right momentum: expected dir -1 (launch right), angle 315\0"
                as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    t_fp.ground_or_air = GA_Ground;
    t_fp.gr_vel = -1.5f32;
    if Geno_Autolink(
        &raw mut t_fp,
        (&raw mut t_fp.x914 as *mut HitCapsule).offset(0 as i32 as isize)
            as *mut HitCapsule,
        &raw mut dir,
        &raw mut angle,
        &raw mut kb,
    ) != 1 as i32 || dir != 1.0f32 || angle != 0.0f32
    {
        TestFail(
            b"on the ground moving left: expected dir +1, angle 0\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    t_fp.gr_vel = 0.01f32;
    angle = 361.0f32;
    if Geno_Autolink(
        &raw mut t_fp,
        (&raw mut t_fp.x914 as *mut HitCapsule).offset(0 as i32 as isize)
            as *mut HitCapsule,
        &raw mut dir,
        &raw mut angle,
        &raw mut kb,
    ) != 0 as i32 || angle != 361.0f32
    {
        TestFail(
            b"a nearly still attacker keeps the hitbox's own angle\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    Geno_OnActionChange(&raw mut t_gobj);
    if (*st).link_mode[0 as i32 as usize] != 0 as i32 {
        TestFail(
            b"an action change must clear LINK\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    return rc;
}

pub unsafe extern "C" fn test_geno_v1_special_attrs() -> i32 {
    melee_core::ct!("test_geno_v1_special_attrs");
    let mut st: *mut TestGenoState = ::core::ptr::null_mut::<TestGenoState>();
    let mut rc: i32 = 0 as i32;
    if Geno_TestInstall(
        b"{\"geno\":1,\"fighters\":[{\"attach\":\"kirby\",\"special_attributes\":[{\"index\":2,\"float\":1.25},{\"offset\":\"0x10\",\"int\":77},{\"index\":999,\"int\":1}]}]}\0"
            as *const u8 as *const i8,
    ) != 1 as i32
    {
        TestFail(
            b"could not install the special_attributes profile\0" as *const u8
                as *const i8,
        );
        Geno_TestRestore();
        return 1 as i32;
    }
    t_zero(
        &raw mut t_ext_attr as *mut u32 as *mut c_void,
        ::core::mem::size_of::<[u32; 16]>() as i32,
    );
    t_zero(
        &raw mut t_dat_attrs as *mut u32 as *mut c_void,
        ::core::mem::size_of::<[u32; 16]>() as i32,
    );
    st = t_setup();
    t_fp.co_attrs.max_jumps = 2 as i32;
    Geno_ApplyAttrs(&raw mut t_fp);
    if t_ext_attr[2 as i32 as usize] != t_fbits(1.25f32)
        || t_ext_attr[4 as i32 as usize] != 77 as u32
        || t_dat_attrs[2 as i32 as usize] != t_fbits(1.25f32)
        || t_dat_attrs[4 as i32 as usize] != 77 as u32
        || t_ext_attr[3 as i32 as usize] != 0 as u32
        || t_fp.co_attrs.max_jumps != 2 as i32
    {
        TestFail(
            b"special words 2 (1.25) and 4 (77) not written to the file block and the buffer\0"
                as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    t_script[0 as i32 as usize] = (((GENO_FTCMD_OP
        << 26 as i32) as u32
        | (GENO_SUB_GET as i32 as u32
            & 63 as u32) << 20 as i32
        | (2 as i32 as u32 & 15 as u32)
            << 16 as i32
        | ((((2 as i32 & 3 as i32)
            << 6 as i32
            | 5 as i32 & 63 as i32)
            & 0xff as i32) << 8 as i32
            | (if 0 as i32 != 0 {
                0x80 as i32
            } else {
                0 as i32
            })
            | (0 as i32 & 7 as i32)
                << 4 as i32) as u32
            & 0xffff as u32) as u32).to_be();
    t_script[1 as i32 as usize] = ((GENO_VAL_SPECIAL_F
        as i32 + 2 as i32) as u32).to_be();
    t_script[2 as i32 as usize] = (((GENO_FTCMD_OP
        << 26 as i32) as u32
        | (GENO_SUB_GET as i32 as u32
            & 63 as u32) << 20 as i32
        | (2 as i32 as u32 & 15 as u32)
            << 16 as i32
        | ((((0 as i32 & 3 as i32)
            << 6 as i32
            | 5 as i32 & 63 as i32)
            & 0xff as i32) << 8 as i32
            | (if 0 as i32 != 0 {
                0x80 as i32
            } else {
                0 as i32
            })
            | (0 as i32 & 7 as i32)
                << 4 as i32) as u32
            & 0xffff as u32) as u32).to_be();
    t_script[3 as i32 as usize] = ((GENO_VAL_SPECIAL_I
        as i32 + 4 as i32) as u32).to_be();
    t_script[4 as i32 as usize] = (0 as u32).to_be();
    t_run(&raw mut t_script as *mut u32, GENO_MODE_EXEC);
    if (*st).la_f[5 as i32 as usize] != 1.25f32
        || (*st).la_i[5 as i32 as usize] != 77 as i32
    {
        TestFail(
            b"GET of the overridden special words\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    Geno_TestRestore();
    return rc;
}

pub unsafe extern "C" fn test_geno_v1_overlay() -> i32 {
    melee_core::ct!("test_geno_v1_overlay");
    let mut st: *mut TestGenoState = ::core::ptr::null_mut::<TestGenoState>();
    static mut orig: [u32; 4] = [0; 4];
    let mut rc: i32 = 0 as i32;
    let mut json: [i8; 400] = [0; 400];
    let mut head: *const i8 = b"{\"geno\":1,\"fighters\":[{\"attach\":\"kirby\",\"subactions\":[{\"index\":3,\"words\":[\"0xEC120100\",5,\"0xED310000\"]}]}]}\0"
        as *const u8 as *const i8;
    let mut k: i32 = 0;
    k = 0 as i32;
    while *head.offset(k as isize) as i32 != '\0' as i32
        && k
            < ::core::mem::size_of::<[i8; 400]>() as i32
                - 1 as i32
    {
        json[k as usize] = *head.offset(k as isize);
        k += 1;
    }
    json[k as usize] = '\0' as i32 as i8;
    if (GENO_FTCMD_OP << 26 as i32) as u32
        | (GENO_SUB_SET as i32 as u32
            & 63 as u32) << 20 as i32
        | (2 as i32 as u32 & 15 as u32)
            << 16 as i32
        | ((((0 as i32 & 3 as i32)
            << 6 as i32
            | 1 as i32 & 63 as i32)
            & 0xff as i32) << 8 as i32
            | (if 0 as i32 != 0 {
                0x80 as i32
            } else {
                0 as i32
            })
            | (0 as i32 & 7 as i32)
                << 4 as i32) as u32
            & 0xffff as u32 != 0xec120100 as u32
        || (GENO_FTCMD_OP << 26 as i32) as u32
            | (GENO_SUB_ORIG as i32 as u32
                & 63 as u32) << 20 as i32
            | (1 as i32 as u32
                & 15 as u32) << 16 as i32
            | 0 as i32 as u32
                & 0xffff as u32 != 0xed310000 as u32
    {
        TestFail(
            b"the overlay test's hand-encoded words are wrong\0" as *const u8
                as *const i8,
        );
        return 1 as i32;
    }
    orig[0 as i32 as usize] = ((19 as u32) << 26 as i32
        | (1 as i32 as u32) << 24 as i32
        | 42 as i32 as u32).to_be();
    orig[1 as i32 as usize] = (0 as u32).to_be();
    t_zero(
        &raw mut t_subactions as *mut Fighter_WaitAnimData as *mut c_void,
        ::core::mem::size_of::<[Fighter_WaitAnimData; 8]>() as i32,
    );
    t_subactions[3 as i32 as usize].xC = &raw mut orig as *mut u32
        as *mut CmdUnion;
    if Geno_TestInstall(&raw mut json as *mut i8)
        != 1 as i32
    {
        TestFail(
            b"could not install the overlay profile\0" as *const u8
                as *const i8,
        );
        Geno_TestRestore();
        return 1 as i32;
    }
    st = t_setup();
    if t_subactions[3 as i32 as usize].xC as *mut u32
        == &raw mut orig as *mut u32
        || *(t_subactions[3 as i32 as usize].xC as *mut u32)
            .offset(0 as i32 as isize)
            != 0xec120100 as u32
        || *(t_subactions[3 as i32 as usize].xC as *mut u32)
            .offset(1 as i32 as isize) != 5 as u32
        || *(t_subactions[3 as i32 as usize].xC as *mut u32)
            .offset(3 as i32 as isize) != 0 as u32
    {
        TestFail(
            b"subaction 3 does not point at the overlay words (with an End appended)\0"
                as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    t_run(
        t_subactions[3 as i32 as usize].xC as *mut u32,
        GENO_MODE_EXEC,
    );
    if (*st).la_i[1 as i32 as usize] != 5 as i32
        || t_fp.cmd_vars[1 as i32 as usize] != 42 as u32
        || !t_fp.x3E4_fighterCmdScript.anon_8.u.is_null()
    {
        TestFail(
            b"the overlay ran, then ORIG must run the original script to its End\0"
                as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    t_setup();
    t_run(
        t_subactions[3 as i32 as usize].xC as *mut u32,
        GENO_MODE_EXEC,
    );
    if t_fp.cmd_vars[1 as i32 as usize] != 42 as u32 {
        TestFail(
            b"after a second spawn ORIG lost the original script\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    if !t_subactions[2 as i32 as usize].xC.is_null() {
        TestFail(
            b"other subactions must be untouched\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    Geno_TestRestore();
    t_zero(
        &raw mut t_subactions as *mut Fighter_WaitAnimData as *mut c_void,
        ::core::mem::size_of::<[Fighter_WaitAnimData; 8]>() as i32,
    );
    return rc;
}

pub unsafe extern "C" fn test_geno_v1_inert() -> i32 {
    melee_core::ct!("test_geno_v1_inert");
    let mut st: *mut TestGenoState = t_setup();
    let mut dir: f32 = 1.0f32;
    let mut angle: f32 = 50.0f32;
    let mut kb: f32 = 3.0f32;
    (*st).flags = 0 as u32;
    if Geno_PreAnim(&raw mut t_gobj) != 0 as i32
        || (*st).action_time != 0 as i32
    {
        TestFail(
            b"PreAnim touched an inert fighter\0" as *const u8
                as *const i8,
        );
        return 1 as i32;
    }
    Geno_CollBegin(&raw mut t_gobj);
    Geno_GroundEdge(&raw mut t_fp, 1 as i32);
    Geno_CollEnd(&raw mut t_gobj);
    if (*st).in_coll != 0 as u32 || (*st).edge_pending != 0 as u32
        || Geno_Autolink(
            &raw mut t_fp,
            (&raw mut t_fp.x914 as *mut HitCapsule)
                .offset(0 as i32 as isize) as *mut HitCapsule,
            &raw mut dir,
            &raw mut angle,
            &raw mut kb,
        ) != 0 as i32 || angle != 50.0f32
    {
        TestFail(
            b"collision / edge / autolink touched an inert fighter\0" as *const u8
                as *const i8,
        );
        return 1 as i32;
    }
    return 0 as i32;
}

pub unsafe extern "C" fn t_cam(mut gobj: *mut Fighter_GObj) { melee_core::ct!("t_cam");}

pub unsafe extern "C" fn t_input(mut gobj: *mut Fighter_GObj) { melee_core::ct!("t_input");}

pub unsafe extern "C" fn t_v2_setup() -> *mut TestGenoState {
    melee_core::ct!("t_v2_setup");
    t_rows[ftCo_MS_Fall as i32 as usize].anim_id = 29
        as i32 as enum_t;
    t_rows[ftCo_MS_Fall as i32 as usize].x4_flags = 0x1234
        as i32 as enum_t;
    t_rows[ftCo_MS_Fall as i32 as usize].cam_cb = Some(
        t_cam as unsafe extern "C" fn(*mut Fighter_GObj) -> (),
    ) as HSD_GObjEvent;
    t_rows[ftCo_MS_Fall as i32 as usize].input_cb = Some(
        t_input as unsafe extern "C" fn(*mut Fighter_GObj) -> (),
    ) as HSD_GObjEvent;
    t_rows[65 as i32 as usize].anim_id = 777 as i32
        as enum_t;
    if Geno_TestInstall(&raw const t_v2_json as *const i8)
        != 1 as i32
    {
        return ::core::ptr::null_mut::<TestGenoState>();
    }
    return t_setup();
}

pub unsafe extern "C" fn test_geno_v2_states() -> i32 {
    melee_core::ct!("test_geno_v2_states");
    let mut st: *mut TestGenoState = t_v2_setup();
    let mut r: *mut MotionState = ::core::ptr::null_mut::<MotionState>();
    let mut rc: i32 = 0 as i32;
    if st.is_null() {
        TestFail(
            b"could not install the v2 profile\0" as *const u8
                as *const i8,
        );
        Geno_TestRestore();
        return 1 as i32;
    }
    r = Geno_MotionRow(&raw mut t_fp, GENO_MOTION_BASE + 0 as i32);
    if r.is_null() || (*r).anim_id != 10 as i32
        || (*r).x4_flags != 0x1234 as i32
        || (*r).cam_cb != Some(t_cam as unsafe extern "C" fn(*mut Fighter_GObj) -> ())
        || (*r).phys_cb.is_none() || (*r).anim_cb.is_none() || (*r).coll_cb.is_none()
    {
        TestFail(
            b"GlideStart row: subaction 10, Fall's flags and camera, Geno callbacks\0"
                as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    r = Geno_MotionRow(&raw mut t_fp, GENO_MOTION_BASE + 1 as i32);
    if (*r).anim_id != 777 as i32 {
        TestFail(
            b"Glide row: \"subaction\": \"motion:65\" must play motion 65's animation (777)\0"
                as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    r = Geno_MotionRow(&raw mut t_fp, GENO_MOTION_BASE + 2 as i32);
    if (*r).anon_8.anon_0.move_id() as i32
        != 9 as i32
    {
        TestFail(
            b"GlideAttack row: move_id 9\0" as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    r = Geno_MotionRow(&raw mut t_fp, GENO_MOTION_BASE + 8 as i32);
    if (*r).input_cb != Some(t_input as unsafe extern "C" fn(*mut Fighter_GObj) -> ())
        || (*r).x4_flags != 0x55 as i32 || (*r).phys_cb.is_none()
    {
        TestFail(
            b"Plain row: iasa \"like\" keeps Fall's input callback, flags 0x55, phys \"none\"\0"
                as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    if Geno_MotionRow(&raw mut t_fp, GENO_MOTION_BASE + 12 as i32)
        != (&raw mut t_rows as *mut MotionState)
            .offset(ftCo_MS_Fall as i32 as isize) as *mut MotionState
    {
        TestFail(
            b"an undeclared Geno motion must fall back to the Fall row\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    Geno_TestRestore();
    return rc;
}

pub unsafe extern "C" fn test_geno_v2_change_to_state() -> i32 {
    melee_core::ct!("test_geno_v2_change_to_state");
    let mut st: *mut TestGenoState = t_v2_setup();
    let mut s: *mut u32 = &raw mut t_script as *mut u32;
    let mut n: i32 = 0 as i32;
    let mut rc: i32 = 0 as i32;
    if st.is_null() {
        TestFail(
            b"could not install the v2 profile\0" as *const u8
                as *const i8,
        );
        Geno_TestRestore();
        return 1 as i32;
    }
    GenoGame_TestCapture(1 as i32, 1 as i32);
    t_fp.motion_id = 330 as i32 as FtMotionId;
    t_fp.ground_or_air = GA_Air;
    let fresh157 = n;
    n = n + 1;
    *s.offset(fresh157 as isize) = (((GENO_FTCMD_OP << 26 as i32)
        as u32
        | (GENO_SUB_CHG as i32 as u32
            & 63 as u32) << 20 as i32
        | (2 as i32 as u32 & 15 as u32)
            << 16 as i32
        | ((GENO_COND_ANIM_END as i32 & 0xff as i32)
            << 8 as i32
            | (if 0 as i32 != 0 {
                0x80 as i32
            } else {
                0 as i32
            })
            | (0 as i32 & 7 as i32)
                << 4 as i32
            | 0 as i32 & 0xc as i32) as u32
            & 0xffff as u32) as u32).to_be();
    let fresh158 = n;
    n = n + 1;
    *s.offset(fresh158 as isize) = (((2 as u32
        & 15 as u32) << 28 as i32
        | 12 as i32 as u32
            & 0xffff as u32) as u32).to_be();
    let fresh159 = n;
    n = n + 1;
    *s.offset(fresh159 as isize) = (((GENO_FTCMD_OP << 26 as i32)
        as u32
        | (GENO_SUB_CHG as i32 as u32
            & 63 as u32) << 20 as i32
        | (2 as i32 as u32 & 15 as u32)
            << 16 as i32
        | ((GENO_COND_ANIM_END as i32 & 0xff as i32)
            << 8 as i32
            | (if 0 as i32 != 0 {
                0x80 as i32
            } else {
                0 as i32
            })
            | (0 as i32 & 7 as i32)
                << 4 as i32
            | 0 as i32 & 0xc as i32) as u32
            & 0xffff as u32) as u32).to_be();
    let fresh160 = n;
    n = n + 1;
    *s.offset(fresh160 as isize) = (((2 as u32
        & 15 as u32) << 28 as i32
        | 0 as i32 as u32 & 0xffff as u32)
        as u32).to_be();
    let fresh161 = n;
    n = n + 1;
    *s.offset(fresh161 as isize) = (((GENO_FTCMD_OP << 26 as i32)
        as u32
        | (GENO_SUB_CHGAND as i32 as u32
            & 63 as u32) << 20 as i32
        | (1 as i32 as u32 & 15 as u32)
            << 16 as i32
        | ((GENO_COND_AIR as i32 & 0xff as i32)
            << 8 as i32
            | (if 0 as i32 != 0 {
                0x80 as i32
            } else {
                0 as i32
            })
            | (0 as i32 & 7 as i32)
                << 4 as i32
            | 0 as i32 & 0xc as i32) as u32
            & 0xffff as u32) as u32).to_be();
    let fresh162 = n;
    n = n + 1;
    *s.offset(fresh162 as isize) = (((GENO_FTCMD_OP << 26 as i32)
        as u32
        | (GENO_SUB_CHG as i32 as u32
            & 63 as u32) << 20 as i32
        | (2 as i32 as u32 & 15 as u32)
            << 16 as i32
        | ((GENO_COND_ANIM_END as i32 & 0xff as i32)
            << 8 as i32
            | (if 0 as i32 != 0 {
                0x80 as i32
            } else {
                0 as i32
            })
            | (0 as i32 & 7 as i32)
                << 4 as i32
            | 0 as i32 & 0xc as i32) as u32
            & 0xffff as u32) as u32).to_be();
    let fresh163 = n;
    n = n + 1;
    *s.offset(fresh163 as isize) = (((0 as u32
        & 15 as u32) << 28 as i32
        | ftCo_MS_FallSpecial as i32 as u32
            & 0xffff as u32) as u32).to_be();
    let fresh164 = n;
    n = n + 1;
    *s.offset(fresh164 as isize) = (0 as u32).to_be();
    t_run(&raw mut t_script as *mut u32, GENO_MODE_EXEC);
    if Geno_PreAnim(&raw mut t_gobj) != 1 as i32
        || t_fp.motion_id != GENO_MOTION_BASE + 0 as i32
        || GenoGame_TestLastTarget()
            != (2 as u32 & 15 as u32)
                << 28 as i32
                | 0 as i32 as u32
                    & 0xffff as u32
    {
        TestFail(
            b"CHG ANIM_END & AIR -> GENO(GlideStart) must enter the Geno state\0"
                as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    t_script[0 as i32 as usize] = (((GENO_FTCMD_OP
        << 26 as i32) as u32
        | (GENO_SUB_GET as i32 as u32
            & 63 as u32) << 20 as i32
        | (2 as i32 as u32 & 15 as u32)
            << 16 as i32
        | ((((0 as i32 & 3 as i32)
            << 6 as i32
            | 3 as i32 & 63 as i32)
            & 0xff as i32) << 8 as i32
            | (if 0 as i32 != 0 {
                0x80 as i32
            } else {
                0 as i32
            })
            | (0 as i32 & 7 as i32)
                << 4 as i32) as u32
            & 0xffff as u32) as u32).to_be();
    t_script[1 as i32 as usize] = (GENO_VAL_GENO_STATE
        as i32 as u32).to_be();
    t_script[2 as i32 as usize] = (0 as u32).to_be();
    t_run(&raw mut t_script as *mut u32, GENO_MODE_EXEC);
    if (*st).la_i[3 as i32 as usize] != T_GS {
        TestFail(
            b"GET GENO_STATE in GlideStart must read 0\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    t_fp.motion_id = 14 as i32 as FtMotionId;
    if Geno_SpecialEnter(&raw mut t_gobj, GENO_SP_N as i32)
        != 1 as i32
        || t_fp.motion_id != GENO_MOTION_BASE + 5 as i32
    {
        TestFail(b"specials.n -> Tornado\0" as *const u8 as *const i8);
        rc = 1 as i32;
    }
    if Geno_SpecialEnter(&raw mut t_gobj, GENO_SP_AIR_S as i32)
        != 1 as i32
        || t_fp.motion_id != GENO_MOTION_BASE + 6 as i32
    {
        TestFail(
            b"specials.air_s defaults to specials.s -> Drill\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    t_fp.motion_id = 14 as i32 as FtMotionId;
    t_fp.set_allow_interrupt(1 as u8 as u8);
    if Geno_SpecialEnter(&raw mut t_gobj, GENO_SP_N as i32)
        != 1 as i32 || t_fp.allow_interrupt() as i32 != 0
    {
        TestFail(
            b"entering a Geno state must clear Allow Interrupt (a whiffed counter's IASA crouched out of the next one)\0"
                as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    if Geno_SpecialEnter(&raw mut t_gobj, GENO_SP_HI as i32)
        != 0 as i32
    {
        TestFail(
            b"an unbound special must be left to the fighter's own code\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    Geno_TestRestore();
    t_setup();
    if Geno_SpecialEnter(&raw mut t_gobj, GENO_SP_N as i32)
        != 0 as i32
    {
        TestFail(
            b"a fighter with no profile: Geno_SpecialEnter must return 0\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    GenoGame_TestCapture(0 as i32, 0 as i32);
    return rc;
}

pub unsafe extern "C" fn t_near(mut a: f32, mut b: f32) -> i32 {
    melee_core::ct!("t_near");
    return (a - b < 0.001f32 && b - a < 0.001f32) as i32;
}

pub unsafe extern "C" fn test_geno_v3_anim_motion() -> i32 {
    melee_core::ct!("test_geno_v3_anim_motion");
    let mut st: *mut TestGenoState = ::core::ptr::null_mut::<TestGenoState>();
    let mut up: *mut MotionState = ::core::ptr::null_mut::<MotionState>();
    let mut loop_0: *mut MotionState = ::core::ptr::null_mut::<MotionState>();
    let mut rc: i32 = 0 as i32;
    t_rows[ftCo_MS_Fall as i32 as usize].anim_id = 29
        as i32 as enum_t;
    if Geno_TestInstall(&raw const t_v3_json as *const i8)
        != 1 as i32
        || {
            st = t_setup();
            st.is_null()
        }
    {
        TestFail(
            b"could not install the v3 profile\0" as *const u8
                as *const i8,
        );
        Geno_TestRestore();
        return 1 as i32;
    }
    GenoGame_TestCapture(1 as i32, 0 as i32);
    up = Geno_MotionRow(&raw mut t_fp, GENO_MOTION_BASE + 0 as i32);
    loop_0 = Geno_MotionRow(&raw mut t_fp, GENO_MOTION_BASE + 1 as i32);
    if (*up).anim_id != 20 as i32 || (*up).phys_cb.is_none()
        || (*up).coll_cb.is_none() || (*loop_0).anim_id != 21 as i32
    {
        TestFail(
            b"anim_motion rows: subactions 20 / 21 with the root-motion callbacks\0"
                as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    t_fp.co_attrs.gravity = 0.1f32;
    t_fp.co_attrs.terminal_velocity = 2.0f32;
    t_fp.anon_594.anon_0.set_x594_b0(1 as u8 as u8);
    t_fp.motion_id = (GENO_MOTION_BASE + 0 as i32) as FtMotionId;
    Geno_OnActionChange(&raw mut t_gobj);
    t_fp.ground_or_air = GA_Air;
    t_fp.facing_dir = 1.0f32;
    t_fp.x6A4_transNOffset.z = 3.0f32 as f32;
    t_fp.x6A4_transNOffset.y = 2.0f32 as f32;
    (*up).phys_cb.expect("non-null function pointer")(&raw mut t_gobj);
    if t_fp.self_vel.x != 3.0f32 || t_fp.self_vel.y != 2.0f32 {
        TestFail(
            b"air root motion facing right must give self_vel (3, 2)\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    t_fp.facing_dir = -1.0f32;
    (*up).phys_cb.expect("non-null function pointer")(&raw mut t_gobj);
    if t_fp.self_vel.x != -3.0f32 || t_fp.self_vel.y != 2.0f32 {
        TestFail(
            b"air root motion facing left must give self_vel (-3, 2)\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    t_fp.ground_or_air = GA_Ground;
    t_fp.facing_dir = 1.0f32;
    (*up).phys_cb.expect("non-null function pointer")(&raw mut t_gobj);
    if t_fp.ground_or_air as i32 != GA_Ground as i32
        || t_fp.gr_vel != 3.0f32
    {
        TestFail(
            b"ground root motion (no lift-off) must set gr_vel 3 and stay grounded\0"
                as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    if GenoGame_TestLedgeMode(&raw mut t_fp) != 1 as i32 {
        TestFail(
            b"state \"ledge\": \"front\" must default the ledge grab to 1\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    t_script[0 as i32 as usize] = (((GENO_FTCMD_OP
        << 26 as i32) as u32
        | (GENO_SUB_PUT as i32 as u32
            & 63 as u32) << 20 as i32
        | (3 as i32 as u32 & 15 as u32)
            << 16 as i32
        | 0 as i32 as u32 & 0xffff as u32)
        as u32).to_be();
    t_script[1 as i32 as usize] = (GENO_VAL_LEDGE as i32
        as u32).to_be();
    t_script[2 as i32 as usize] = (0 as u32).to_be();
    t_script[3 as i32 as usize] = (0 as u32).to_be();
    t_run(&raw mut t_script as *mut u32, GENO_MODE_EXEC);
    if GenoGame_TestLedgeMode(&raw mut t_fp) != 0 as i32
        || (*st).ledge != 0 as i32
    {
        TestFail(
            b"PUT LEDGE 0 must disallow the ledge grab for this action\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    t_fp.motion_id = (GENO_MOTION_BASE + 1 as i32) as FtMotionId;
    Geno_OnActionChange(&raw mut t_gobj);
    if (*st).ledge != -(1 as i32)
        || GenoGame_TestLedgeMode(&raw mut t_fp) != 2 as i32
    {
        TestFail(
            b"a new action must reset PUT LEDGE; state \"ledge\": \"both\" -> 2\0"
                as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    t_fp.ground_or_air = GA_Ground;
    t_fp.x68C_transNPos.z = 5.0f32 as f32;
    t_fp.x68C_transNPos.y = 1.0f32 as f32;
    t_fp.x6A4_transNOffset.z = 1.0f32 as f32;
    t_fp.x6A4_transNOffset.y = 2.0f32 as f32;
    (*loop_0).phys_cb.expect("non-null function pointer")(&raw mut t_gobj);
    if t_fp.ground_or_air as i32 != GA_Air as i32
        || t_near(t_fp.self_vel.x, 6.0f32) == 0 || t_near(t_fp.self_vel.y, 2.95f32) == 0
    {
        TestFail(
            b"lift-off with origin + gravity: airborne, self_vel (6, 2.95)\0"
                as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    (*loop_0).phys_cb.expect("non-null function pointer")(&raw mut t_gobj);
    if t_near(t_fp.self_vel.x, 1.0f32) == 0 || t_near(t_fp.self_vel.y, 1.9f32) == 0 {
        TestFail(
            b"second frame: no origin, gravity accumulated: self_vel (1, 1.9)\0"
                as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    if snap_open(1 as i32) == 0 as i32 {
        let mut g: f32 = (*st).motion_vy;
        snap_save(0 as i32);
        (*st).motion_vy = 55.0f32 as f32;
        (*st).ledge = 2 as i32 as i32;
        snap_load(0 as i32);
        if (*st).motion_vy != g || (*st).ledge != -(1 as i32) {
            TestFail(
                b"anim_motion gravity / ledge not restored by a savestate load\0"
                    as *const u8 as *const i8,
            );
            rc = 1 as i32;
        }
    }
    t_fp.motion_id = (GENO_MOTION_BASE + 0 as i32) as FtMotionId;
    Geno_OnActionChange(&raw mut t_gobj);
    t_fp.anon_594.anon_0.set_x594_b0(0 as u8 as u8);
    t_fp.ground_or_air = GA_Air;
    t_fp.self_vel.x = 4.0f32 as f32;
    t_fp.self_vel.y = -1.0f32 as f32;
    (*up).phys_cb.expect("non-null function pointer")(&raw mut t_gobj);
    if t_fp.self_vel.x != 0.0f32 || t_fp.self_vel.y != 0.0f32 {
        TestFail(
            b"an anim_motion state without root motion and gravity must hold still\0"
                as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    GenoGame_TestCapture(0 as i32, 0 as i32);
    Geno_TestRestore();
    return rc;
}

pub unsafe extern "C" fn test_geno_v3_many_states() -> i32 {
    melee_core::ct!("test_geno_v3_many_states");
    static mut head: [i8; 51] = unsafe {
        ::core::mem::transmute::<
            [u8; 51],
            [i8; 51],
        >(*b"{\"geno\":3,\"fighters\":[{\"attach\":\"kirby\",\"states\":[\0")
    };
    static mut a: [i8; 11] = unsafe {
        ::core::mem::transmute::<
            [u8; 11],
            [i8; 11],
        >(*b"{\"name\":\"S\0")
    };
    static mut b: [i8; 37] = unsafe {
        ::core::mem::transmute::<
            [u8; 37],
            [i8; 37],
        >(*b"\",\"behavior\":\"geno.air\",\"subaction\":\0")
    };
    static mut tail: [i8; 5] = unsafe {
        ::core::mem::transmute::<[u8; 5], [i8; 5]>(*b"]}]}\0")
    };
    let mut n: i32 = 0 as i32;
    let mut i: i32 = 0;
    let mut rc: i32 = 0 as i32;
    let mut c: *const i8 = ::core::ptr::null::<i8>();
    c = &raw const head as *const i8;
    while *c != 0 {
        let fresh165 = n;
        n = n + 1;
        t_v3_many[fresh165 as usize] = *c;
        c = c.offset(1);
    }
    i = 0 as i32;
    while i < 40 as i32 {
        c = &raw const a as *const i8;
        while *c != 0 {
            let fresh166 = n;
            n = n + 1;
            t_v3_many[fresh166 as usize] = *c;
            c = c.offset(1);
        }
        let fresh167 = n;
        n = n + 1;
        t_v3_many[fresh167 as usize] = ('0' as i32 + i / 10 as i32)
            as i8;
        let fresh168 = n;
        n = n + 1;
        t_v3_many[fresh168 as usize] = ('0' as i32 + i % 10 as i32)
            as i8;
        c = &raw const b as *const i8;
        while *c != 0 {
            let fresh169 = n;
            n = n + 1;
            t_v3_many[fresh169 as usize] = *c;
            c = c.offset(1);
        }
        let fresh170 = n;
        n = n + 1;
        t_v3_many[fresh170 as usize] = ('0' as i32
            + (100 as i32 + i) / 100 as i32)
            as i8;
        let fresh171 = n;
        n = n + 1;
        t_v3_many[fresh171 as usize] = ('0' as i32
            + (100 as i32 + i) / 10 as i32
                % 10 as i32) as i8;
        let fresh172 = n;
        n = n + 1;
        t_v3_many[fresh172 as usize] = ('0' as i32
            + (100 as i32 + i) % 10 as i32)
            as i8;
        let fresh173 = n;
        n = n + 1;
        t_v3_many[fresh173 as usize] = '}' as i32 as i8;
        if i < 39 as i32 {
            let fresh174 = n;
            n = n + 1;
            t_v3_many[fresh174 as usize] = ',' as i32 as i8;
        }
        i += 1;
    }
    c = &raw const tail as *const i8;
    while *c != 0 {
        let fresh175 = n;
        n = n + 1;
        t_v3_many[fresh175 as usize] = *c;
        c = c.offset(1);
    }
    t_v3_many[n as usize] = 0 as i8;
    if GENO_MAX_STATES < 40 as i32
        || Geno_TestInstall(&raw mut t_v3_many as *mut i8)
            != 1 as i32 || t_setup().is_null()
    {
        TestFail(
            b"could not install a 40-state profile\0" as *const u8
                as *const i8,
        );
        Geno_TestRestore();
        return 1 as i32;
    }
    if (*Geno_MotionRow(&raw mut t_fp, GENO_MOTION_BASE + 39 as i32))
        .anim_id != 139 as i32
        || (*Geno_MotionRow(&raw mut t_fp, GENO_MOTION_BASE + 16 as i32))
            .anim_id != 116 as i32
    {
        TestFail(
            b"a 40-state profile must build rows 16..39 (subactions 116 / 139)\0"
                as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    Geno_TestRestore();
    return rc;
}

pub unsafe extern "C" fn test_geno_lab_stale_fighter() -> i32 {
    melee_core::ct!("test_geno_lab_stale_fighter");
    static mut saved: StaticPlayer = unsafe { core::mem::zeroed() };
    let mut rc: i32 = 0 as i32;
    t_setup();
    saved = *(&raw mut player_slots as *mut StaticPlayer)
        .offset(5 as i32 as isize);
    (*(&raw mut player_slots as *mut StaticPlayer)
        .offset(5 as i32 as isize))
        .transformed[0 as i32 as usize] = 0 as u8;
    let ref mut fresh176 = (*(&raw mut player_slots as *mut StaticPlayer)
        .offset(5 as i32 as isize))
        .player_entity[0 as i32 as usize];
    *fresh176 = &raw mut t_gobj;
    if ScriptGame_FighterI(5 as i32, 0 as i32)
        != 0 as i32
        || ScriptGame_FighterI(5 as i32, 3 as i32)
            != -(1 as i32)
        || ScriptGame_FighterF(5 as i32, 0 as i32)
            != 0.0f32
    {
        TestFail(
            b"a slot whose fighter is not in the live fighter list must read as absent\0"
                as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    *(&raw mut player_slots as *mut StaticPlayer)
        .offset(5 as i32 as isize) = saved;
    return rc;
}

pub unsafe extern "C" fn test_geno_player_team_read() -> i32 {
    melee_core::ct!("test_geno_player_team_read");
    static mut saved1: StaticPlayer = unsafe { core::mem::zeroed() };
    static mut saved2: StaticPlayer = unsafe { core::mem::zeroed() };
    let mut rc: i32 = 0 as i32;
    t_setup();
    saved1 = *(&raw mut player_slots as *mut StaticPlayer)
        .offset(1 as i32 as isize);
    saved2 = *(&raw mut player_slots as *mut StaticPlayer)
        .offset(2 as i32 as isize);
    (*(&raw mut player_slots as *mut StaticPlayer)
        .offset(1 as i32 as isize))
        .pkind = Gm_PKind_Cpu;
    (*(&raw mut player_slots as *mut StaticPlayer)
        .offset(1 as i32 as isize))
        .team = 0 as u8;
    (*(&raw mut player_slots as *mut StaticPlayer)
        .offset(2 as i32 as isize))
        .pkind = Gm_PKind_Cpu;
    (*(&raw mut player_slots as *mut StaticPlayer)
        .offset(2 as i32 as isize))
        .team = 2 as u8;
    if ScriptGame_FighterI(1 as i32, 9 as i32)
        != 0 as i32
        || ScriptGame_FighterI(2 as i32, 9 as i32)
            != 2 as i32
    {
        TestFail(
            b"the team read must return Player_GetTeam for a present slot\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    (*(&raw mut player_slots as *mut StaticPlayer)
        .offset(2 as i32 as isize))
        .pkind = Gm_PKind_NA;
    if ScriptGame_FighterI(2 as i32, 9 as i32)
        != -(1 as i32)
        || ScriptGame_FighterI(6 as i32, 9 as i32)
            != -(1 as i32)
    {
        TestFail(
            b"an empty or out-of-range slot must read team -1\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    *(&raw mut player_slots as *mut StaticPlayer)
        .offset(1 as i32 as isize) = saved1;
    *(&raw mut player_slots as *mut StaticPlayer)
        .offset(2 as i32 as isize) = saved2;
    return rc;
}

pub unsafe extern "C" fn SceneLaunch_ApplyItemFrequency(
    mut rules: *mut StartMeleeRules,
    mut prefs: *mut GamePrefs,
) {
    melee_core::ct!("SceneLaunch_ApplyItemFrequency");
    let mut frequency: i32 = SceneLaunch_ItemFreq();
    if frequency != -(1 as i32) {
        (*rules).item_freq = (if frequency == -(2 as i32) {
            -(1 as i32)
        } else {
            frequency
        }) as i8;
        (*prefs).item_freq = (*rules).item_freq as u8;
    }
}

pub unsafe extern "C" fn test_geno_lab_mode_table() -> i32 {
    melee_core::ct!("test_geno_lab_mode_table");
    let mut m: *mut GameMode = ::core::ptr::null_mut::<GameMode>();
    let mut s: *mut GameModeState = ::core::ptr::null_mut::<GameModeState>();
    let mut found: i32 = 0 as i32;
    if GM_COUNT as i32 + 2 as i32
        <= GM_COUNT as i32
        || GM_COUNT as i32 + 2 as i32
            == GM_COUNT as i32 + 1 as i32
    {
        TestFail(
            b"GM_LAB must be past GM_COUNT and not GM_FRONTEND\0" as *const u8
                as *const i8,
        );
        return 1 as i32;
    }
    m = gm_GetAllGameModes();
    while (*m).kind as i32 != GM_COUNT as i32 {
        if (*m).kind as i32
            == GM_COUNT as i32 + 2 as i32
        {
            found += 1;
            if (*m).states != &raw mut gm_Mode_Lab_States as *mut GameModeState
                || (*m).on_load
                    != Some(gm_Mode_Lab_OnLoad as unsafe extern "C" fn() -> ())
            {
                TestFail(
                    b"the GM_LAB row does not point at LAB's states / on_load\0"
                        as *const u8 as *const i8,
                );
                return 1 as i32;
            }
        }
        m = m.offset(1);
    }
    if found != 1 as i32 {
        TestFail(
            b"GM_LAB must be in the mode table exactly once\0" as *const u8
                as *const i8,
        );
        return 1 as i32;
    }
    s = &raw mut gm_Mode_Lab_States as *mut GameModeState;
    if (*s.offset(0 as i32 as isize)).info.scene_kind
        as i32 != GS_CSS as i32
        || (*s.offset(1 as i32 as isize)).info.scene_kind
            as i32 != GS_SSS as i32
        || (*s.offset(2 as i32 as isize)).info.scene_kind
            as i32 != GS_VS as i32
        || (*s.offset(2 as i32 as isize)).info.enter_data
            != &raw mut gmVsMelee_StartData as *mut c_void
        || (*s.offset(3 as i32 as isize)).info.scene_kind
            as i32
            != GS_COUNT as i32 + 1 as i32
        || (*s.offset(4 as i32 as isize)).id as i32
            != GM_GAMEMODESTATE_TERMINATE as u8 as i32
    {
        TestFail(
            b"LAB's states must be CSS, SSS, the VS match and the loading screen\0"
                as *const u8 as *const i8,
        );
        return 1 as i32;
    }
    return 0 as i32;
}

pub unsafe extern "C" fn test_scene_item_frequency_rules() -> i32 {
    melee_core::ct!("test_scene_item_frequency_rules");
    let mut live: *mut StartMeleeRules = gm_GetStartMeleeRules() as *mut StartMeleeRules;
    let mut saved: StartMeleeRules = *live;
    let mut prefs: GamePrefs = unsafe { core::mem::zeroed() };
    let mut rc: i32 = 0 as i32;
    (*live).item_freq = 3 as i8;
    (*live).x20 = !(0 as u64) as u64;
    prefs.item_freq = 3 as u8;
    prefs.item_mask = !(0 as u64) as u64;
    SceneLaunch_LoadForTest(
        b"mode=vs;items=off\0" as *const u8 as *const i8,
    );
    SceneLaunch_ApplyItemFrequency(live, &raw mut prefs);
    if SceneLaunch_ItemFreq() != -(2 as i32)
        || (*live).item_freq as i32 != -(1 as i32)
        || prefs.item_freq as i32 != 0xff as i32
        || gm_8016AE80() != -(1 as i32)
        || (*live).x20 != !(0 as u64)
        || prefs.item_mask != !(0 as u64)
    {
        TestFail(
            b"items=off must disable native/saved frequency without changing allowed kinds\0"
                as *const u8 as *const i8,
        );
        rc = 1 as i32;
    } else {
        it_8026D018();
        if it_8026D324(It_Kind_BombHei) != 0 {
            TestFail(
                b"items=off left native random-item eligibility enabled\0" as *const u8
                    as *const i8,
            );
            rc = 1 as i32;
        }
    }
    SceneLaunch_LoadForTest(
        b"mode=vs;items=0\0" as *const u8 as *const i8,
    );
    SceneLaunch_ApplyItemFrequency(live, &raw mut prefs);
    if SceneLaunch_ItemFreq() != 0 as i32
        || (*live).item_freq as i32 != 0 as i32
        || prefs.item_freq as i32 != 0 as i32
        || gm_8016AE80() != 0 as i32
    {
        TestFail(
            b"items=0 is frequency index zero, not disabled\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    SceneLaunch_LoadForTest(::core::ptr::null::<i8>());
    (*live).item_freq = 2 as i8;
    prefs.item_freq = 2 as u8;
    SceneLaunch_ApplyItemFrequency(live, &raw mut prefs);
    if (*live).item_freq as i32 != 2 as i32
        || prefs.item_freq as i32 != 2 as i32
    {
        TestFail(
            b"unspecified item frequency must preserve existing rules\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    *live = saved;
    return rc;
}

pub unsafe extern "C" fn test_geno_lab_rules() -> i32 {
    melee_core::ct!("test_geno_lab_rules");
    static mut d: StartMeleeData = unsafe { core::mem::zeroed() };
    let mut i: i32 = 0;
    d.rules.set_match_kind(MatchKind_Stock as i32 as u32 as u32);
    d.rules.set_is_stock(1 as u32 as u32);
    d.rules.set_timer_enabled(1 as u32 as u32);
    d.rules.time_limit = 480 as u32;
    d.rules.item_freq = 2 as i8;
    d.rules.set_x4_2(1 as u32 as u32);
    i = 0 as i32;
    while i < Gm_Player_NumMax as i32 {
        d.players[i as usize].stocks = 4 as i8;
        i += 1;
    }
    GenoLab_ApplyRules(&raw mut d);
    if d.rules.match_kind() as i32 != MatchKind_Time as i32
        || d.rules.is_stock() as i32 != 0
        || d.rules.timer_enabled() as i32 != 0
        || d.rules.time_limit != 0 as u32 || d.rules.x4_2() as i32 != 0
    {
        TestFail(
            b"LAB rules: time mode with the clock off, no stock game over\0" as *const u8
                as *const i8,
        );
        return 1 as i32;
    }
    if d.rules.item_freq as i32 != -(1 as i32)
        || d.rules.disable_pausing() == 0
    {
        TestFail(
            b"LAB rules: items off, Melee's pause off\0" as *const u8
                as *const i8,
        );
        return 1 as i32;
    }
    SceneLaunch_LoadForTest(
        b"mode=lab;items=3\0" as *const u8 as *const i8,
    );
    d.rules.item_freq = -(1 as i32) as i8;
    GenoLab_ApplyRules(&raw mut d);
    if d.rules.item_freq as i32 != 3 as i32 {
        SceneLaunch_LoadForTest(::core::ptr::null::<i8>());
        TestFail(
            b"LAB rules: items=3 must reach the match\0" as *const u8
                as *const i8,
        );
        return 1 as i32;
    }
    SceneLaunch_LoadForTest(
        b"mode=lab;items=off\0" as *const u8 as *const i8,
    );
    d.rules.item_freq = 2 as i8;
    GenoLab_ApplyRules(&raw mut d);
    if d.rules.item_freq as i32 != -(1 as i32) {
        SceneLaunch_LoadForTest(::core::ptr::null::<i8>());
        TestFail(
            b"LAB rules: items=off stays off\0" as *const u8
                as *const i8,
        );
        return 1 as i32;
    }
    SceneLaunch_LoadForTest(::core::ptr::null::<i8>());
    i = 0 as i32;
    while i < Gm_Player_NumMax as i32 {
        if d.players[i as usize].stocks as i32 != 0 as i32
        {
            TestFail(
                b"LAB rules: no stocks on any player\0" as *const u8
                    as *const i8,
            );
            return 1 as i32;
        }
        i += 1;
    }
    return 0 as i32;
}

pub unsafe extern "C" fn test_geno_lab_scene() -> i32 {
    melee_core::ct!("test_geno_lab_scene");
    let mut rc: i32 = 0 as i32;
    SceneLaunch_LoadForTest(
        b"mode=lab;p1=fox;p2=falco/cpu;stage=fd\0" as *const u8
            as *const i8,
    );
    if SceneLaunch_BootGameMode()
        != GM_COUNT as i32 + 2 as i32
        || SceneLaunch_EntryStateId() != 2 as i32
        || SceneLaunch_PlayerCKind(0 as i32) < 0 as i32
        || SceneLaunch_PlayerCKind(1 as i32) < 0 as i32
        || SceneLaunch_PlayerSlotType(1 as i32)
            != Gm_PKind_Cpu as i32
        || SceneLaunch_StageExternal() != 32 as i32
    {
        TestFail(
            b"mode=lab did not give LAB's match, Fox + CPU Falco on FD\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    SceneLaunch_LoadForTest(
        b"mode=lab;at=css\0" as *const u8 as *const i8,
    );
    if rc == 0 as i32
        && (SceneLaunch_BootGameMode()
            != GM_COUNT as i32 + 2 as i32
            || SceneLaunch_EntryStateId() != 0 as i32)
    {
        TestFail(
            b"mode=lab;at=css did not open LAB's character select\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    SceneLaunch_LoadForTest(::core::ptr::null::<i8>());
    return rc;
}

pub unsafe extern "C" fn test_geno_lab_select_flow() -> i32 {
    melee_core::ct!("test_geno_lab_select_flow");
    let mut st: GameModeState = *(&raw mut gm_Mode_Lab_States as *mut GameModeState)
        .offset(0 as i32 as isize);
    gmFrontend_ModeSelect(
        &raw mut st,
        0 as i32,
        b"LAB\0" as *const u8 as *const i8,
    );
    if st.info.scene_kind as i32
        != GS_COUNT as i32 + 1 as i32
    {
        TestFail(
            b"LAB's character select must run the kit's screen\0" as *const u8
                as *const i8,
        );
        return 1 as i32;
    }
    if GenoLab_ModeActive() != 0
        || GenoLab_Leave(GENO_LAB_TO_MENU as i32)
            != 0 as i32
        || GenoLab_NextAfterMatch() != GENO_LAB_TO_CSS as i32
    {
        TestFail(
            b"outside LAB: not active, leave refused, next stays the character select\0"
                as *const u8 as *const i8,
        );
        return 1 as i32;
    }
    return 0 as i32;
}

pub unsafe extern "C" fn t_streq(
    mut a: *const i8,
    mut b: *const i8,
) -> i32 {
    melee_core::ct!("t_streq");
    if a.is_null() || b.is_null() {
        return 0 as i32;
    }
    while *a as i32 != 0 as i32
        && *a as i32 == *b as i32
    {
        a = a.offset(1);
        b = b.offset(1);
    }
    return (*a as i32 == *b as i32) as i32;
}

pub unsafe extern "C" fn test_geno_lab_mismatch_fields() -> i32 {
    melee_core::ct!("test_geno_lab_mismatch_fields");
    let mut mi: i32 = 1564 as u64
        as i32;
    let mut pos: i32 = 196 as u64
        as i32;
    let mut pct: i32 = 8112 as u64
        as i32;
    if GenoGame_StateSize() != ::core::mem::size_of::<GenoState>() as i32
    {
        TestFail(
            b"GenoGame_StateSize is not sizeof(GenoState)\0" as *const u8
                as *const i8,
        );
        return 1 as i32;
    }
    if t_streq(
        GenoGame_StateFieldName(mi + 8 as i32),
        b"move_i\0" as *const u8 as *const i8,
    ) == 0 || GenoGame_StateFieldBase(mi + 8 as i32) != mi
    {
        TestFail(
            b"GenoState +move_i[2] must name move_i at its base\0" as *const u8
                as *const i8,
        );
        return 1 as i32;
    }
    if t_streq(
        GenoGame_StateFieldName(1676 as u64 as i32),
        b"motion_vy\0" as *const u8 as *const i8,
    ) == 0
    {
        TestFail(
            b"GenoState motion_vy must be named\0" as *const u8
                as *const i8,
        );
        return 1 as i32;
    }
    if t_streq(
        ScriptGame_LabFighterFieldName(pos + 4 as i32),
        b"cur_pos\0" as *const u8 as *const i8,
    ) == 0 || ScriptGame_LabFighterFieldBase(pos + 4 as i32) != pos
    {
        TestFail(
            b"Fighter +cur_pos.y must name cur_pos\0" as *const u8
                as *const i8,
        );
        return 1 as i32;
    }
    if t_streq(
        ScriptGame_LabFighterFieldName(pct + 4 as i32),
        b"dmg\0" as *const u8 as *const i8,
    ) == 0
    {
        TestFail(
            b"Fighter dmg.x1830_percent must name dmg\0" as *const u8
                as *const i8,
        );
        return 1 as i32;
    }
    return 0 as i32;
}

pub unsafe extern "C" fn t_bitsf(mut b: i32) -> f32 {
    melee_core::ct!("t_bitsf");
    let mut x: Anon_anon_1941 = unsafe { core::mem::zeroed() };
    x.u = b as u32;
    return x.f;
}

pub unsafe extern "C" fn test_geno_v5_registry() -> i32 {
    melee_core::ct!("test_geno_v5_registry");
    let mut rc: i32 = 0 as i32;
    let mut d: *mut Article = ::core::ptr::null_mut::<Article>();
    if Geno_TestInstall(&raw const t_v5_json as *const i8)
        != 1 as i32
    {
        TestFail(
            b"could not install the v5 profile\0" as *const u8
                as *const i8,
        );
        Geno_TestRestore();
        return 1 as i32;
    }
    if Geno_ArticleCount(0 as i32) != 2 as i32
        || Geno_ArticleHitCount(0 as i32, 0 as i32)
            != 1 as i32
        || Geno_ArticleHitCount(0 as i32, 1 as i32)
            != 2 as i32
    {
        TestFail(
            b"two articles, with 1 and 2 hitboxes\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    if t_bitsf(
        Geno_ArticleParam(
            0 as i32,
            0 as i32,
            GENO_AP_LIFETIME as i32,
        ),
    ) != 40.0f32
        || t_bitsf(
            Geno_ArticleParam(
                0 as i32,
                0 as i32,
                GENO_AP_VEL_FWD as i32,
            ),
        ) != 2.5f32
        || t_bitsf(
            Geno_ArticleParam(
                0 as i32,
                0 as i32,
                GENO_AP_SPAWN_UP as i32,
            ),
        ) != 8.0f32
        || t_bitsf(
            Geno_ArticleParam(
                0 as i32,
                1 as i32,
                GENO_AP_LIFETIME as i32,
            ),
        ) != 60.0f32
        || t_bitsf(
            Geno_ArticleParam(
                0 as i32,
                1 as i32,
                GENO_AP_SCALE as i32,
            ),
        ) != 1.0f32
        || Geno_ArticleParam(
            0 as i32,
            0 as i32,
            GENO_AP_MAX_LIVE as i32,
        ) != 4 as i32
        || Geno_ArticleParam(
            0 as i32,
            1 as i32,
            GENO_AP_MAX_LIVE as i32,
        ) != 2 as i32
    {
        TestFail(
            b"article parameters: lifetime 40 / default 60, velocity, spawn, scale 1, max_live\0"
                as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    if Geno_ArticleParam(
        0 as i32,
        0 as i32,
        GENO_AP_DESPAWN as i32,
    ) != GENO_ART_DESPAWN_DEFAULT as i32
        || Geno_ArticleParam(
            0 as i32,
            1 as i32,
            GENO_AP_DESPAWN as i32,
        ) != (GENO_ART_DESPAWN_DEFAULT & !GENO_ART_DESPAWN_STAGE) as i32
    {
        TestFail(
            b"despawn mask: default hit|shield|stage|clank; \"stage\": false clears one bit\0"
                as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    if t_bitsf(
        Geno_ArticleHitParam(
            0 as i32,
            0 as i32,
            0 as i32,
            GENO_AH_DAMAGE as i32,
        ),
    ) != 7.0f32
        || Geno_ArticleHitParam(
            0 as i32,
            0 as i32,
            0 as i32,
            GENO_AH_ANGLE as i32,
        ) != 45 as i32
        || Geno_ArticleHitParam(
            0 as i32,
            0 as i32,
            0 as i32,
            GENO_AH_KBG as i32,
        ) != 60 as i32
        || Geno_ArticleHitParam(
            0 as i32,
            0 as i32,
            0 as i32,
            GENO_AH_BKB as i32,
        ) != 20 as i32
        || Geno_ArticleHitParam(
            0 as i32,
            0 as i32,
            0 as i32,
            GENO_AH_START as i32,
        ) != 2 as i32
        || Geno_ArticleHitParam(
            0 as i32,
            0 as i32,
            0 as i32,
            GENO_AH_END as i32,
        ) != 30 as i32
        || Geno_ArticleHitParam(
            0 as i32,
            1 as i32,
            0 as i32,
            GENO_AH_ANGLE as i32,
        ) != 361 as i32
        || Geno_ArticleHitParam(
            0 as i32,
            1 as i32,
            0 as i32,
            GENO_AH_FLAGS as i32,
        ) != (GENO_AHF_DEFAULT & !GENO_AHF_REFLECT) as i32
        || t_bitsf(
            Geno_ArticleHitParam(
                0 as i32,
                1 as i32,
                1 as i32,
                GENO_AH_OFF_Z as i32,
            ),
        ) != 3.0f32
    {
        TestFail(
            b"article hitboxes: numbers, defaults (angle 361), the hits flags, offsets\0"
                as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    if Geno_StateCounter(
        0 as i32,
        0 as i32,
        GENO_CTR_FROM as i32,
    ) != 3 as i32
        || Geno_StateCounter(
            0 as i32,
            0 as i32,
            GENO_CTR_TO as i32,
        ) != 10 as i32
        || Geno_StateCounter(
            0 as i32,
            0 as i32,
            GENO_CTR_TARGET as i32,
        )
            != ((2 as u32 & 15 as u32)
                << 28 as i32
                | 1 as i32 as u32
                    & 0xffff as u32) as i32
        || Geno_StateCounter(
            0 as i32,
            0 as i32,
            GENO_CTR_NEGATE as i32,
        ) != 1 as i32
        || Geno_StateCounter(
            0 as i32,
            1 as i32,
            GENO_CTR_FROM as i32,
        ) != -(1 as i32)
        || Geno_StateCounter(
            0 as i32,
            2 as i32,
            GENO_CTR_NEGATE as i32,
        ) != 0 as i32
        || Geno_StateCounter(
            0 as i32,
            2 as i32,
            GENO_CTR_TARGET as i32,
        ) != -(1 as i32)
    {
        TestFail(
            b"counter windows: frames 3-10 -> geno:CounterHit, negate default 1; none on CounterHit\0"
                as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    if Geno_HookCount(0 as i32, GENO_EV_HIT as i32)
        != 1 as i32
    {
        TestFail(
            b"hooks.on_hit must bind one hook\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    d = Geno_ArticleDesc(GENO_ART_KIND_BASE + 1 as i32) as *mut Article;
    if d.is_null() || (*d).x10_modelDesc.is_null()
        || !(*(*d).x10_modelDesc).x0_joint.is_null() || (*d).x0_common_attr.is_null()
        || (*(*d).x0_common_attr).x60_scale != 1.0f32 || !(*d).x8_hurtbones.is_null()
        || Geno_ArticleLogic(GENO_ART_KIND_BASE + 1 as i32).is_null()
    {
        TestFail(
            b"article kind 0x1001: a descriptor (no model: invisible, scale 1, no hurtbox)\0"
                as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    if !Geno_ArticleDesc(GENO_ART_KIND_BASE + 2 as i32).is_null()
        || !Geno_ArticleDesc(GENO_ART_KIND_BASE + 8 as i32).is_null()
        || !Geno_ArticleDesc(237 as i32).is_null()
    {
        TestFail(
            b"an undeclared article kind (or an m-ex kind) must have no Geno descriptor\0"
                as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    Geno_TestRestore();
    return rc;
}

pub unsafe extern "C" fn test_geno_v5_article_reflection() -> i32 {
    melee_core::ct!("test_geno_v5_article_reflection");
    static mut item: Item = unsafe { core::mem::zeroed() };
    static mut gobj: HSD_GObj = unsafe { core::mem::zeroed() };
    let mut logic: *mut ItemLogicTable = ::core::ptr::null_mut::<ItemLogicTable>();
    let mut vars: *mut GenoArtVars = ::core::ptr::null_mut::<GenoArtVars>();
    let mut rc: i32 = 0 as i32;
    if Geno_TestInstall(&raw const t_v5_json as *const i8)
        != 1 as i32
    {
        TestFail(
            b"could not install the v5 profile\0" as *const u8
                as *const i8,
        );
        return 1 as i32;
    }
    t_zero(
        &raw mut item as *mut c_void,
        ::core::mem::size_of::<Item>() as i32,
    );
    t_zero(
        &raw mut gobj as *mut c_void,
        ::core::mem::size_of::<HSD_GObj>() as i32,
    );
    gobj.user_data = &raw mut item as *mut c_void;
    vars = &raw mut item.xDD4_itemVar as *mut GenoArtVars;
    (*vars).vx = 2.0f32 as f32;
    (*vars).vy = -4.0f32 as f32;
    (*vars).frame = 1 as i32 as i32;
    item.xC68 = -1.0f32 as f32;
    item.facing_dir = item.xC68;
    item.xC70 = 1.5f32 as f32;
    logic = Geno_ArticleLogic(GENO_ART_KIND_BASE) as *mut ItemLogicTable;
    if logic.is_null()
        || (*logic).reflected.expect("non-null function pointer")(&raw mut gobj) != 0
    {
        TestFail(
            b"a reflected Geno article must survive\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    } else {
        (*(*logic).states.offset(0 as i32 as isize))
            .physics_updated
            .expect("non-null function pointer")(&raw mut gobj);
        if (*vars).vx != -3.0f32 || (*vars).vy != -6.0f32 || item.x40_vel.x != -3.0f32
            || item.x40_vel.y != -6.0f32
        {
            TestFail(
                b"reflector speed multiplier must survive the following article physics step\0"
                    as *const u8 as *const i8,
            );
            rc = 1 as i32;
        }
    }
    Geno_TestRestore();
    return rc;
}

pub unsafe extern "C" fn test_geno_v5_article_motion() -> i32 {
    melee_core::ct!("test_geno_v5_article_motion");
    let mut rc: i32 = 0 as i32;
    let mut f: i32 = 0;
    let mut vx: f32 = 2.5f32;
    let mut vy: f32 = 0.0f32;
    let mut px: f32 = 0.0f32;
    if Geno_TestInstall(&raw const t_v5_json as *const i8)
        != 1 as i32
    {
        TestFail(
            b"could not install the v5 profile\0" as *const u8
                as *const i8,
        );
        Geno_TestRestore();
        return 1 as i32;
    }
    f = 1 as i32;
    while f <= 40 as i32 {
        GenoGame_ArticleStep(
            0 as i32,
            0 as i32,
            f,
            &raw mut vx,
            &raw mut vy,
            px,
            0.0f32,
            1 as i32,
            0.0f32,
            50.0f32,
        );
        px += vx;
        f += 1;
    }
    if vx != 2.5f32 || vy != 0.0f32 || t_near(px, 100.0f32) == 0 {
        TestFail(
            b"a straight article must fly 2.5 a frame for 40 frames (100 units), no homing\0"
                as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    vx = 1.0f32 as f32;
    vy = 2.0f32 as f32;
    GenoGame_ArticleStep(
        0 as i32,
        1 as i32,
        1 as i32,
        &raw mut vx,
        &raw mut vy,
        0.0f32,
        0.0f32,
        1 as i32,
        100.0f32,
        -100.0f32,
    );
    if t_near(vx, 1.0f32) == 0 || t_near(vy, 1.9f32) == 0 {
        TestFail(
            b"before the homing delay only gravity applies (vy 2 -> 1.9)\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    let mut a0: f32 = atan2f(vy as f32, vx as f32)
        as f32;
    let mut a1: f32 = 0.;
    GenoGame_ArticleStep(
        0 as i32,
        1 as i32,
        4 as i32,
        &raw mut vx,
        &raw mut vy,
        0.0f32,
        0.0f32,
        1 as i32,
        100.0f32,
        -100.0f32,
    );
    vy += 0.1f32;
    a1 = atan2f(vy as f32, vx as f32) as f32;
    if t_near((a0 - a1) * 57.29578f32, 5.0f32) == 0 {
        TestFail(
            b"homing must turn exactly 5 degrees toward the target\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    f = 5 as i32;
    while f < 200 as i32 {
        GenoGame_ArticleStep(
            0 as i32,
            1 as i32,
            f,
            &raw mut vx,
            &raw mut vy,
            0.0f32,
            1000.0f32,
            0 as i32,
            0.0f32,
            0.0f32,
        );
        f += 1;
    }
    if t_near(vy, -1.5f32) == 0
        || sqrtf(
            vx as f32 * vx as f32
                + vy as f32 * vy as f32,
        ) > 3.0001f32
    {
        TestFail(
            b"gravity must stop at max_fall 1.5 and the speed stay under max_speed 3\0"
                as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    Geno_TestRestore();
    return rc;
}

pub unsafe extern "C" fn test_geno_v5_on_hit() -> i32 {
    melee_core::ct!("test_geno_v5_on_hit");
    let mut st: *mut TestGenoState = ::core::ptr::null_mut::<TestGenoState>();
    let mut rc: i32 = 0 as i32;
    t_rows[ftCo_MS_Fall as i32 as usize].anim_id = 29
        as i32 as enum_t;
    if Geno_TestInstall(&raw const t_v5_json as *const i8)
        != 1 as i32
        || {
            st = t_setup();
            st.is_null()
        }
    {
        TestFail(
            b"could not install the v5 profile\0" as *const u8
                as *const i8,
        );
        Geno_TestRestore();
        return 1 as i32;
    }
    GenoGame_TestCapture(1 as i32, 0 as i32);
    t_fp.ground_or_air = GA_Ground;
    t_fp.motion_id = ftCo_MS_Wait as i32 as FtMotionId;
    if Geno_SpecialEnter(&raw mut t_gobj, GENO_SP_LW as i32)
        != 1 as i32
        || t_fp.motion_id != GENO_MOTION_BASE + 0 as i32
    {
        TestFail(b"specials.lw -> Counter\0" as *const u8 as *const i8);
        rc = 1 as i32;
    }
    (*st).action_time = 2 as i32 as i32;
    t_fp.dmg.kb_applied = 40.0f32;
    t_fp.dmg.x1838_percentTemp = 12.0f32;
    t_fp.dmg.x18c4_source_ply = 1 as i32;
    if Geno_OnHit(&raw mut t_gobj) != 0 as i32
        || t_fp.dmg.kb_applied != 40.0f32 || t_fp.dmg.x1838_percentTemp != 12.0f32
        || t_fp.motion_id != GENO_MOTION_BASE + 0 as i32
        || (*st).hit_count != 1 as i32
        || (*st).la_i[7 as i32 as usize] != 1 as i32
        || (*st).hit_counter != 0 as i32
        || (*st).hit_port != 1 as i32 || (*st).hit_damage != 12.0f32
    {
        TestFail(
            b"a hit before the window: taken, on_hit hook ran, HIT_* set, still in Counter\0"
                as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    (*st).action_time = 5 as i32 as i32;
    t_fp.dmg.kb_applied = 40.0f32;
    t_fp.dmg.x1838_percentTemp = 9.0f32;
    if Geno_OnHit(&raw mut t_gobj) != 1 as i32
        || t_fp.dmg.kb_applied != 0.0f32 || t_fp.dmg.x1838_percentTemp != 0.0f32
        || t_fp.motion_id != GENO_MOTION_BASE + 1 as i32
        || (*st).hit_count != 2 as i32
        || (*st).la_i[7 as i32 as usize] != 2 as i32
        || (*st).hit_counter != 1 as i32
        || (*st).counters != 1 as i32 || (*st).hit_damage != 9.0f32
    {
        TestFail(
            b"a hit in the window: countered (no damage, no knockback), -> CounterHit, HIT_COUNTER 1\0"
                as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    (*st).action_time = 5 as i32 as i32;
    t_fp.dmg.kb_applied = 40.0f32;
    t_fp.dmg.x1838_percentTemp = 9.0f32;
    if Geno_OnHit(&raw mut t_gobj) != 0 as i32
        || t_fp.dmg.x1838_percentTemp != 9.0f32
        || (*st).hit_counter != 0 as i32
    {
        TestFail(
            b"a hit in a state without a window stands\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    t_fp.dmg.kb_applied = 0.0f32;
    t_fp.dmg.x1838_percentTemp = 0.0f32;
    if Geno_OnHit(&raw mut t_gobj) != 0 as i32
        || (*st).hit_count != 3 as i32
    {
        TestFail(b"no hit, no on_hit\0" as *const u8 as *const i8);
        rc = 1 as i32;
    }
    t_fp.motion_id = (GENO_MOTION_BASE + 2 as i32) as FtMotionId;
    (*st).action_time = 1 as i32 as i32;
    t_fp.dmg.kb_applied = 30.0f32;
    t_fp.dmg.x1838_percentTemp = 5.0f32;
    if Geno_OnHit(&raw mut t_gobj) != 1 as i32
        || t_fp.dmg.x1838_percentTemp != 5.0f32
        || t_fp.motion_id != GENO_MOTION_BASE + 2 as i32
        || (*st).hit_counter != 1 as i32
    {
        TestFail(
            b"\"negate\": false: the hit is flagged (HIT_COUNTER 1) and still taken\0"
                as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    t_script[0 as i32 as usize] = (((GENO_FTCMD_OP
        << 26 as i32) as u32
        | (GENO_SUB_GET as i32 as u32
            & 63 as u32) << 20 as i32
        | (2 as i32 as u32 & 15 as u32)
            << 16 as i32
        | ((((0 as i32 & 3 as i32)
            << 6 as i32
            | 3 as i32 & 63 as i32)
            & 0xff as i32) << 8 as i32
            | (if 0 as i32 != 0 {
                0x80 as i32
            } else {
                0 as i32
            })
            | (0 as i32 & 7 as i32)
                << 4 as i32) as u32
            & 0xffff as u32) as u32).to_be();
    t_script[1 as i32 as usize] = (GENO_VAL_HIT_COUNT as i32
        as u32).to_be();
    t_script[2 as i32 as usize] = (((GENO_FTCMD_OP
        << 26 as i32) as u32
        | (GENO_SUB_GET as i32 as u32
            & 63 as u32) << 20 as i32
        | (2 as i32 as u32 & 15 as u32)
            << 16 as i32
        | ((((0 as i32 & 3 as i32)
            << 6 as i32
            | 4 as i32 & 63 as i32)
            & 0xff as i32) << 8 as i32
            | (if 0 as i32 != 0 {
                0x80 as i32
            } else {
                0 as i32
            })
            | (0 as i32 & 7 as i32)
                << 4 as i32) as u32
            & 0xffff as u32) as u32).to_be();
    t_script[3 as i32 as usize] = (GENO_VAL_HIT_COUNTER
        as i32 as u32).to_be();
    t_script[4 as i32 as usize] = (((GENO_FTCMD_OP
        << 26 as i32) as u32
        | (GENO_SUB_CALL as i32 as u32
            & 63 as u32) << 20 as i32
        | (3 as i32 as u32 & 15 as u32)
            << 16 as i32
        | 0 as i32 as u32 & 0xffff as u32)
        as u32).to_be();
    t_script[5 as i32 as usize] = (GENO_HOOK_ARTICLE_SPAWN
        as i32 as u32).to_be();
    t_script[6 as i32 as usize] = (0 as u32).to_be();
    t_script[7 as i32 as usize] = (0 as u32).to_be();
    t_run(&raw mut t_script as *mut u32, GENO_MODE_EXEC);
    if (*st).la_i[3 as i32 as usize] != 4 as i32
        || (*st).la_i[4 as i32 as usize] != 1 as i32
        || (*st).art_spawned != 1 as i32
    {
        TestFail(
            b"GET HIT_COUNT / HIT_COUNTER, and CALL geno.article.spawn from a script\0"
                as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    GenoGame_TestCapture(0 as i32, 0 as i32);
    Geno_TestRestore();
    t_setup();
    t_fp.dmg.kb_applied = 40.0f32;
    t_fp.dmg.x1838_percentTemp = 12.0f32;
    if Geno_OnHit(&raw mut t_gobj) != 0 as i32
        || t_fp.dmg.x1838_percentTemp != 12.0f32
        || (*(GenoGame_StateOf(&raw mut t_fp) as *mut TestGenoState)).hit_count
            != 0 as i32
    {
        TestFail(
            b"a fighter with no profile: on_hit is inert\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    t_fp.dmg.kb_applied = 0.0f32;
    t_fp.dmg.x1838_percentTemp = 0.0f32;
    return rc;
}

pub unsafe extern "C" fn test_geno_v51() -> i32 {
    melee_core::ct!("test_geno_v51");
    let mut st: *mut TestGenoState = ::core::ptr::null_mut::<TestGenoState>();
    let mut rc: i32 = 0 as i32;
    let mut f: i32 = 0;
    let mut vx: f32 = 3.6f32;
    let mut vy: f32 = 0.0f32;
    if Geno_TestInstall(&raw const t_v51_json as *const i8)
        != 1 as i32
        || {
            st = t_setup();
            st.is_null()
        }
    {
        TestFail(
            b"could not install the v5.1 profile\0" as *const u8
                as *const i8,
        );
        Geno_TestRestore();
        return 1 as i32;
    }
    if Geno_ArticleHitParam(
        0 as i32,
        0 as i32,
        0 as i32,
        GENO_AH_SLOT as i32,
    ) != 0 as i32
        || Geno_ArticleHitParam(
            0 as i32,
            0 as i32,
            1 as i32,
            GENO_AH_SLOT as i32,
        ) != 0 as i32
        || Geno_ArticleParam(
            0 as i32,
            0 as i32,
            GENO_AP_SPAWN_N as i32,
        ) != 2 as i32
        || t_bitsf(
            Geno_ArticleParam(
                0 as i32,
                0 as i32,
                GENO_AP_SPAWN_V as i32 + 3 as i32,
            ),
        ) != 2.0f32
        || t_bitsf(
            Geno_ArticleParam(
                0 as i32,
                0 as i32,
                GENO_AP_ANGLE as i32,
            ),
        ) != 4.0f32
        || Geno_ArticleParam(
            0 as i32,
            0 as i32,
            GENO_AP_BONE as i32,
        ) != -(1 as i32)
        || Geno_ArticleParam(
            0 as i32,
            1 as i32,
            GENO_AP_BONE as i32,
        ) != 2 as i32
        || Geno_ArticleParam(
            0 as i32,
            1 as i32,
            GENO_AP_EFFECT as i32,
        ) != 1147 as i32
    {
        TestFail(
            b"v5.1 article keys: slots (both entries in slot 0), spawns, angle, bone, effect\0"
                as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    if Geno_ArticleParam(
        0 as i32,
        2 as i32,
        GENO_AP_SHOW_MODEL as i32,
    ) != 1 as i32
        || Geno_ArticleParam(
            0 as i32,
            0 as i32,
            GENO_AP_SHOW_MODEL as i32,
        ) != 0 as i32
        || Geno_ArticleParam(
            0 as i32,
            1 as i32,
            GENO_AP_SHOW_MODEL as i32,
        ) != 0 as i32
    {
        TestFail(
            b"v5.7 show_model: on for Bolt only, default off\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    if Geno_ArticleParam(
        0 as i32,
        1 as i32,
        GENO_AP_CHILD as i32,
    ) != 2 as i32
        || Geno_ArticleParam(
            0 as i32,
            1 as i32,
            GENO_AP_CHILD as i32 + 1 as i32,
        ) != 3 as i32
        || Geno_ArticleParam(
            0 as i32,
            1 as i32,
            GENO_AP_CHILD as i32 + 2 as i32,
        ) != 3 as i32
        || Geno_ArticleParam(
            0 as i32,
            1 as i32,
            GENO_AP_CHILD as i32 + 3 as i32,
        ) != 4 as i32
        || Geno_ArticleParam(
            0 as i32,
            1 as i32,
            GENO_AP_CHILD as i32 + 4 as i32,
        ) != 1 as i32
        || Geno_ArticleParam(
            0 as i32,
            1 as i32,
            GENO_AP_CHILD as i32 + 5 as i32,
        ) != -(1 as i32)
        || Geno_ArticleParam(
            0 as i32,
            0 as i32,
            GENO_AP_CHILD as i32,
        ) != -(1 as i32)
    {
        TestFail(
            b"children: Cloud -> Bolt (by name) at 3, every 3, 4 times, variant 1; none elsewhere\0"
                as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    f = 1 as i32;
    while f <= 5 as i32 {
        GenoGame_ArticleStep(
            0 as i32,
            0 as i32,
            f,
            &raw mut vx,
            &raw mut vy,
            0.0f32,
            0.0f32,
            0 as i32,
            0.0f32,
            0.0f32,
        );
        f += 1;
    }
    if t_near(vx, 2.7f32) == 0 {
        TestFail(
            b"brake: 5 frames of -0.18 from 3.6 = 2.7\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    f = 6 as i32;
    while f <= 60 as i32 {
        GenoGame_ArticleStep(
            0 as i32,
            0 as i32,
            f,
            &raw mut vx,
            &raw mut vy,
            0.0f32,
            0.0f32,
            0 as i32,
            0.0f32,
            0.0f32,
        );
        f += 1;
    }
    if t_near(vx, 0.2f32) == 0 {
        TestFail(
            b"brake must stop at min_speed 0.2\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    if Geno_SpecialSelect(
        0 as i32,
        GENO_SP_N as i32,
        -(1 as i32),
    )
        != (0 as i32 & 3 as i32) << 6 as i32
            | 0 as i32 & 63 as i32
        || Geno_SpecialSelect(
            0 as i32,
            GENO_SP_AIR_N as i32,
            -(2 as i32),
        ) != 3 as i32
    {
        TestFail(
            b"select: la_i:0, three targets, air_n inherits it\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    GenoGame_TestCapture(1 as i32, 0 as i32);
    t_fp.ground_or_air = GA_Ground;
    f = 0 as i32;
    while f < 3 as i32 {
        (*st).la_i[0 as i32 as usize] = f as i32;
        t_fp.motion_id = ftCo_MS_Wait as i32 as FtMotionId;
        if Geno_SpecialEnter(&raw mut t_gobj, GENO_SP_N as i32)
            != 1 as i32 || t_fp.motion_id != GENO_MOTION_BASE + f
        {
            TestFail(
                b"select: LA0 = n must enter state n\0" as *const u8
                    as *const i8,
            );
            rc = 1 as i32;
        }
        f += 1;
    }
    (*st).la_i[0 as i32 as usize] = 3 as i32 as i32;
    t_fp.motion_id = ftCo_MS_Wait as i32 as FtMotionId;
    if Geno_SpecialEnter(&raw mut t_gobj, GENO_SP_N as i32)
        != 0 as i32
    {
        TestFail(
            b"select: LA0 out of range must leave the special to the fighter\0"
                as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    GenoGame_TestCapture(0 as i32, 0 as i32);
    Geno_TestRestore();
    return rc;
}

pub unsafe extern "C" fn test_geno_v54_retired() -> i32 {
    melee_core::ct!("test_geno_v54_retired");
    if GenoGame_HookFind(
        b"geno.dash.search\0" as *const u8 as *const i8,
    ) != -(1 as i32)
        || GenoGame_HookFind(
            b"geno.dash.aim\0" as *const u8 as *const i8,
        ) != -(1 as i32)
        || GenoGame_HookFind(b"geno.brake\0" as *const u8 as *const i8)
            != -(1 as i32)
    {
        TestFail(
            b"a retired v5.4 dash hook still resolves\0" as *const u8
                as *const i8,
        );
        return 1 as i32;
    }
    if GenoGame_CallbackFind(
        GENO_CB_PHYS as i32,
        b"brake\0" as *const u8 as *const i8,
    ) != -(1 as i32)
        || GenoGame_CallbackFind(
            GENO_CB_PHYS as i32,
            b"multi_dash\0" as *const u8 as *const i8,
        ) == -(1 as i32)
    {
        TestFail(
            b"phys names: \"brake\" must be gone and \"multi_dash\" present\0"
                as *const u8 as *const i8,
        );
        return 1 as i32;
    }
    return 0 as i32;
}

pub unsafe extern "C" fn test_geno_v52_lockon() -> i32 {
    melee_core::ct!("test_geno_v52_lockon");
    let mut fw: f32 = 0.;
    let mut up: f32 = 0.;
    let mut face: f32 = 0.;
    let mut rc: i32 = 0 as i32;
    GenoGame_LockonAim(
        1 as i32,
        30.0f32,
        30.0f32,
        0.0f32,
        0.0f32,
        1.0f32,
        0 as i32,
        30.0f32,
        20.0f32,
        &raw mut fw,
        &raw mut up,
        &raw mut face,
    );
    if t_near(fw, 0.8660254f32) == 0 || t_near(up, 0.5f32) == 0 || face != 1.0f32 {
        TestFail(
            b"lockon: 45 deg up clamps to 30\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    GenoGame_LockonAim(
        1 as i32,
        -20.0f32,
        0.0f32,
        0.0f32,
        0.0f32,
        1.0f32,
        0 as i32,
        30.0f32,
        20.0f32,
        &raw mut fw,
        &raw mut up,
        &raw mut face,
    );
    if t_near(fw, 1.0f32) == 0 || t_near(up, 0.0f32) == 0 || face != -1.0f32 {
        TestFail(
            b"lockon: a target behind turns the fighter\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    GenoGame_LockonAim(
        0 as i32,
        0.0f32,
        0.0f32,
        0.0f32,
        -0.8f32,
        -1.0f32,
        0 as i32,
        30.0f32,
        20.0f32,
        &raw mut fw,
        &raw mut up,
        &raw mut face,
    );
    if t_near(up, -0.34202f32) == 0 || face != -1.0f32 {
        TestFail(
            b"lockon: no target, stick down in the air = -20 deg, facing kept\0"
                as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    GenoGame_LockonAim(
        0 as i32,
        0.0f32,
        0.0f32,
        0.0f32,
        -0.8f32,
        1.0f32,
        1 as i32,
        30.0f32,
        20.0f32,
        &raw mut fw,
        &raw mut up,
        &raw mut face,
    );
    if t_near(up, 0.0f32) == 0 || t_near(fw, 1.0f32) == 0 {
        TestFail(
            b"lockon: never aims down on the ground\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    GenoGame_LockonAim(
        0 as i32,
        0.0f32,
        0.0f32,
        0.0f32,
        0.1f32,
        1.0f32,
        0 as i32,
        30.0f32,
        20.0f32,
        &raw mut fw,
        &raw mut up,
        &raw mut face,
    );
    if t_near(up, 0.0f32) == 0 {
        TestFail(
            b"lockon: stick inside 0.25 = level\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    let mut s: *mut u32 = &raw mut t_script as *mut u32;
    let mut n: i32 = 0 as i32;
    t_setup();
    t_fp.frame_speed_mul = 1.0f32;
    let fresh177 = n;
    n = n + 1;
    *s.offset(fresh177 as isize) = (((GENO_FTCMD_OP << 26 as i32)
        as u32
        | (GENO_SUB_PUT as i32 as u32
            & 63 as u32) << 20 as i32
        | (3 as i32 as u32 & 15 as u32)
            << 16 as i32
        | 0 as i32 as u32 & 0xffff as u32)
        as u32).to_be();
    let fresh178 = n;
    n = n + 1;
    *s.offset(fresh178 as isize) = (GENO_VAL_ANIM_RATE as i32 as u32).to_be();
    let fresh179 = n;
    n = n + 1;
    *s.offset(fresh179 as isize) = (t_fbits(0.5f32)).to_be();
    let fresh180 = n;
    n = n + 1;
    *s.offset(fresh180 as isize) = (((GENO_FTCMD_OP << 26 as i32)
        as u32
        | (GENO_SUB_PUT as i32 as u32
            & 63 as u32) << 20 as i32
        | (3 as i32 as u32 & 15 as u32)
            << 16 as i32
        | 0 as i32 as u32 & 0xffff as u32)
        as u32).to_be();
    let fresh181 = n;
    n = n + 1;
    *s.offset(fresh181 as isize) = (GENO_VAL_ANIM_RATE as i32 as u32).to_be();
    let fresh182 = n;
    n = n + 1;
    *s.offset(fresh182 as isize) = (t_fbits(0.0f32)).to_be();
    let fresh183 = n;
    n = n + 1;
    *s.offset(fresh183 as isize) = (0 as u32).to_be();
    t_run(&raw mut t_script as *mut u32, GENO_MODE_EXEC);
    if t_fp.frame_speed_mul != 0.5f32 {
        TestFail(
            b"PUT ANIM_RATE 0.5 sets frame_speed_mul; 0 is refused\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    t_fp.frame_speed_mul = 1.0f32;
    let mut s_0: *mut u32 = &raw mut t_script as *mut u32;
    let mut n_0: i32 = 0 as i32;
    let mut st: *mut TestGenoState = t_setup();
    t_fp.x914[3 as i32 as usize].damage = 9.0f32;
    t_fp.x914[2 as i32 as usize].damage = t_fp
        .x914[3 as i32 as usize]
        .damage;
    t_fp.x914[1 as i32 as usize].damage = t_fp
        .x914[2 as i32 as usize]
        .damage;
    t_fp.x914[0 as i32 as usize].damage = t_fp
        .x914[1 as i32 as usize]
        .damage;
    (*st).la_f[5 as i32 as usize] = 13.5f32 as f32;
    let fresh184 = n_0;
    n_0 = n_0 + 1;
    *s_0.offset(fresh184 as isize) = ((GENO_FTCMD_OP << 26 as i32)
        as u32
        | (GENO_SUB_HBDMG as i32 as u32
            & 63 as u32) << 20 as i32
        | (2 as i32 as u32 & 15 as u32)
            << 16 as i32
        | ((0x9 as i32) << 8 as i32
            | 0x80 as i32) as u32
            & 0xffff as u32) as u32;
    let fresh185 = n_0;
    n_0 = n_0 + 1;
    *s_0.offset(fresh185 as isize) = ((2 as i32 & 3 as i32)
        << 6 as i32 | 5 as i32 & 63 as i32)
        as u32;
    let fresh186 = n_0;
    n_0 = n_0 + 1;
    *s_0.offset(fresh186 as isize) = ((GENO_FTCMD_OP << 26 as i32)
        as u32
        | (GENO_SUB_HBDMG as i32 as u32
            & 63 as u32) << 20 as i32
        | (2 as i32 as u32 & 15 as u32)
            << 16 as i32
        | ((0x2 as i32) << 8 as i32) as u32
            & 0xffff as u32) as u32;
    let fresh187 = n_0;
    n_0 = n_0 + 1;
    *s_0.offset(fresh187 as isize) = t_fbits(2.0f32);
    let fresh188 = n_0;
    n_0 = n_0 + 1;
    *s_0.offset(fresh188 as isize) = 0 as u32;
    t_run(&raw mut t_script as *mut u32, GENO_MODE_EXEC);
    if t_fp.x914[0 as i32 as usize].damage != 13.5f32
        || t_fp.x914[3 as i32 as usize].damage != 13.5f32
        || t_fp.x914[1 as i32 as usize].damage != 2.0f32
        || t_fp.x914[2 as i32 as usize].damage != 9.0f32
    {
        TestFail(
            b"HBDMG: mask 9 = LAF5, mask 2 = 2.0, hitbox 2 kept\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }


    let mut s_1: *mut u32 = &raw mut t_script as *mut u32;
    let mut n_1: i32 = 0 as i32;
    let mut b0: i32 = 0;
    let mut b0again: i32 = 0;
    let mut b1: i32 = 0;
    let mut b3: i32 = 0;
    let mut bmax: i32 = 0;
    let mut bcleared: i32 = 0;
    let mut st_0: *mut TestGenoState = t_setup();
    (*st_0).la_i[7 as i32 as usize] = 5 as i32 as i32;
    let fresh189 = n_1;
    n_1 = n_1 + 1;
    *s_1.offset(fresh189 as isize) = ((GENO_FTCMD_OP << 26 as i32)
        as u32
        | (GENO_SUB_HBSTUN as i32 as u32
            & 63 as u32) << 20 as i32
        | (2 as i32 as u32 & 15 as u32)
            << 16 as i32
        | ((0x5 as i32) << 8 as i32) as u32
            & 0xffff as u32) as u32;
    let fresh190 = n_1;
    n_1 = n_1 + 1;
    *s_1.offset(fresh190 as isize) = 12 as u32;
    let fresh191 = n_1;
    n_1 = n_1 + 1;
    *s_1.offset(fresh191 as isize) = ((GENO_FTCMD_OP << 26 as i32)
        as u32
        | (GENO_SUB_HBSTUN as i32 as u32
            & 63 as u32) << 20 as i32
        | (2 as i32 as u32 & 15 as u32)
            << 16 as i32
        | ((0x2 as i32) << 8 as i32
            | 0x80 as i32) as u32
            & 0xffff as u32) as u32;
    let fresh192 = n_1;
    n_1 = n_1 + 1;
    *s_1.offset(fresh192 as isize) = ((0 as i32 & 3 as i32)
        << 6 as i32 | 7 as i32 & 63 as i32)
        as u32;
    let fresh193 = n_1;
    n_1 = n_1 + 1;
    *s_1.offset(fresh193 as isize) = 0 as u32;
    t_run(&raw mut t_script as *mut u32, GENO_MODE_EXEC);
    if (*st_0).stun_add[0 as i32 as usize] != 12 as i32 {
        TestFail(
            b"HBSTUN: stun_add[0] != 12\0" as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    if (*st_0).stun_add[1 as i32 as usize] != 5 as i32 {
        TestFail(
            b"HBSTUN: stun_add[1] != 5\0" as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    if (*st_0).stun_add[2 as i32 as usize] != 12 as i32 {
        TestFail(
            b"HBSTUN: stun_add[2] != 12\0" as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    if (*st_0).stun_add[3 as i32 as usize] != 0 as i32 {
        TestFail(
            b"HBSTUN: stun_add[3] != 0\0" as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    Geno_HitStunBonus(&raw mut t_fp, 0 as i32, &raw mut t_fp);
    b0 = Geno_TakeStunBonus(&raw mut t_fp);
    b0again = Geno_TakeStunBonus(&raw mut t_fp);
    Geno_HitStunBonus(&raw mut t_fp, 3 as i32, &raw mut t_fp);
    b3 = Geno_TakeStunBonus(&raw mut t_fp);
    Geno_HitStunBonus(&raw mut t_fp, 1 as i32, &raw mut t_fp);
    Geno_HitStunBonus(&raw mut t_fp, 2 as i32, &raw mut t_fp);
    bmax = Geno_TakeStunBonus(&raw mut t_fp);
    Geno_HitStunBonus(&raw mut t_fp, 1 as i32, &raw mut t_fp);
    b1 = Geno_TakeStunBonus(&raw mut t_fp);
    Geno_OnActionChange(&raw mut t_gobj);
    Geno_HitStunBonus(&raw mut t_fp, 0 as i32, &raw mut t_fp);
    bcleared = Geno_TakeStunBonus(&raw mut t_fp);
    if b0 != 12 as i32 {
        TestFail(
            b"HBSTUN: hitbox 0 gives +12\0" as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    if b0again != 0 as i32 {
        TestFail(
            b"HBSTUN: the bonus is taken once\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    if b3 != 0 as i32 {
        TestFail(
            b"HBSTUN: hitbox 3 gives nothing\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    if bmax != 12 as i32 {
        TestFail(
            b"HBSTUN: two hits in a frame give the largest\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    if b1 != 5 as i32 {
        TestFail(
            b"HBSTUN: hitbox 1 from LAI7 gives 5\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    if bcleared != 0 as i32 {
        TestFail(
            b"HBSTUN: an action change clears the hitbox bonuses\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }



    let mut s_2: *mut u32 = &raw mut t_script as *mut u32;
    let mut n_2: i32 = 0 as i32;
    let mut f0: i32 = 0;
    let mut lifted: i32 = 0;
    t_setup();
    let fresh194 = n_2;
    n_2 = n_2 + 1;
    *s_2.offset(fresh194 as isize) = ((GENO_FTCMD_OP << 26 as i32)
        as u32
        | (GENO_SUB_HBFLAGS as i32 as u32
            & 63 as u32) << 20 as i32
        | (2 as i32 as u32 & 15 as u32)
            << 16 as i32
        | ((0x1 as i32) << 8 as i32) as u32
            & 0xffff as u32) as u32;
    let fresh195 = n_2;
    n_2 = n_2 + 1;
    *s_2.offset(fresh195 as isize) = 7 as u32;
    let fresh196 = n_2;
    n_2 = n_2 + 1;
    *s_2.offset(fresh196 as isize) = ((GENO_FTCMD_OP << 26 as i32)
        as u32
        | (GENO_SUB_HBFLAGS as i32 as u32
            & 63 as u32) << 20 as i32
        | (2 as i32 as u32 & 15 as u32)
            << 16 as i32
        | ((0x2 as i32) << 8 as i32) as u32
            & 0xffff as u32) as u32;
    let fresh197 = n_2;
    n_2 = n_2 + 1;
    *s_2.offset(fresh197 as isize) = 8 as u32;
    let fresh198 = n_2;
    n_2 = n_2 + 1;
    *s_2.offset(fresh198 as isize) = 0 as u32;
    t_run(&raw mut t_script as *mut u32, GENO_MODE_EXEC);
    f0 = Geno_HitFlags(&raw mut t_fp, 0 as i32, &raw mut t_fp);
    t_fp.dmg.kb_applied = 50.0f32;
    Geno_HitReact(&raw mut t_fp);
    if f0 != 7 as i32 {
        TestFail(
            b"HBFLAGS: hitbox 0 carries 7\0" as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    if t_fp.dmg.kb_applied != 0.0f32 {
        TestFail(
            b"HBFLAGS: FLINCHLESS drops the knockback\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    if Geno_NoHitlag(&raw mut t_fp) == 0 {
        TestFail(
            b"HBFLAGS: NO_HITLAG reported\0" as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    t_setup();
    *s_2.offset(0 as i32 as isize) = ((GENO_FTCMD_OP
        << 26 as i32) as u32
        | (GENO_SUB_HBFLAGS as i32 as u32
            & 63 as u32) << 20 as i32
        | (2 as i32 as u32 & 15 as u32)
            << 16 as i32
        | ((0x2 as i32) << 8 as i32) as u32
            & 0xffff as u32) as u32;
    *s_2.offset(1 as i32 as isize) = 8 as u32;
    *s_2.offset(2 as i32 as isize) = 0 as u32;
    t_run(&raw mut t_script as *mut u32, GENO_MODE_EXEC);
    Geno_HitFlags(&raw mut t_fp, 1 as i32, &raw mut t_fp);
    t_fp.set_no_kb(1 as u8 as u8);
    t_fp.dmg.kb_applied = 50.0f32;
    lifted = Geno_HitReact(&raw mut t_fp);
    if lifted != 1 as i32
        || t_fp.no_kb() as i32 != 0 as i32
        || t_fp.dmg.kb_applied != 50.0f32
    {
        TestFail(
            b"HBFLAGS: FORCE_REACTION lifts no_kb, keeps the knockback\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    t_fp.set_no_kb(0 as u8 as u8);
    if Geno_HitFlags(&raw mut t_fp, 3 as i32, &raw mut t_fp)
        != 0 as i32
    {
        TestFail(
            b"HBFLAGS: hitbox 3 has none\0" as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    Geno_OnActionChange(&raw mut t_gobj);
    if Geno_HitFlags(&raw mut t_fp, 1 as i32, &raw mut t_fp)
        != 0 as i32
    {
        TestFail(
            b"HBFLAGS: an action change clears them\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    let mut s_3: *mut u32 = &raw mut t_script as *mut u32;
    let mut n_3: i32 = 0 as i32;
    let mut dir: f32 = 1.0f32;
    let mut ang: f32 = 45.0f32;
    let mut kb: f32 = 50.0f32;
    let mut st_1: *mut TestGenoState = t_setup();
    *s_3.offset(0 as i32 as isize) = ((GENO_FTCMD_OP
        << 26 as i32) as u32
        | (0 as i32 as u32 & 63 as u32)
            << 20 as i32
        | (1 as i32 as u32 & 15 as u32)
            << 16 as i32
        | 0 as i32 as u32 & 0xffff as u32)
        as u32;
    *s_3.offset(1 as i32 as isize) = 0 as u32;
    t_run(&raw mut t_script as *mut u32, GENO_MODE_EXEC);
    Geno_OnActionChange(&raw mut t_gobj);
    Geno_Autolink(
        &raw mut t_fp,
        (&raw mut t_fp.x914 as *mut HitCapsule).offset(0 as i32 as isize)
            as *mut HitCapsule,
        &raw mut dir,
        &raw mut ang,
        &raw mut kb,
    );
    let fresh199 = n_3;
    n_3 = n_3 + 1;
    *s_3.offset(fresh199 as isize) = ((GENO_FTCMD_OP << 26 as i32)
        as u32
        | (GENO_SUB_GET as i32 as u32
            & 63 as u32) << 20 as i32
        | (2 as i32 as u32 & 15 as u32)
            << 16 as i32
        | ((((0 as i32 & 3 as i32)
            << 6 as i32
            | 20 as i32 & 63 as i32)
            & 0xff as i32) << 8 as i32
            | (if 0 as i32 != 0 {
                0x80 as i32
            } else {
                0 as i32
            })
            | (0 as i32 & 7 as i32)
                << 4 as i32) as u32
            & 0xffff as u32) as u32;
    let fresh200 = n_3;
    n_3 = n_3 + 1;
    *s_3.offset(fresh200 as isize) = GENO_VAL_ATTACK_CONNECTED as i32
        as u32;
    let fresh201 = n_3;
    n_3 = n_3 + 1;
    *s_3.offset(fresh201 as isize) = 0 as u32;
    t_run(&raw mut t_script as *mut u32, GENO_MODE_EXEC);
    if (*st_1).la_i[20 as i32 as usize] != 1 as i32 {
        TestFail(
            b"ATTACK_CONNECTED is 1 after a won hit\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    Geno_OnActionChange(&raw mut t_gobj);
    n_3 = 0 as i32;
    let fresh202 = n_3;
    n_3 = n_3 + 1;
    *s_3.offset(fresh202 as isize) = ((GENO_FTCMD_OP << 26 as i32)
        as u32
        | (GENO_SUB_GET as i32 as u32
            & 63 as u32) << 20 as i32
        | (2 as i32 as u32 & 15 as u32)
            << 16 as i32
        | ((((0 as i32 & 3 as i32)
            << 6 as i32
            | 21 as i32 & 63 as i32)
            & 0xff as i32) << 8 as i32
            | (if 0 as i32 != 0 {
                0x80 as i32
            } else {
                0 as i32
            })
            | (0 as i32 & 7 as i32)
                << 4 as i32) as u32
            & 0xffff as u32) as u32;
    let fresh203 = n_3;
    n_3 = n_3 + 1;
    *s_3.offset(fresh203 as isize) = GENO_VAL_ATTACK_CONNECTED as i32
        as u32;
    let fresh204 = n_3;
    n_3 = n_3 + 1;
    *s_3.offset(fresh204 as isize) = ((GENO_FTCMD_OP << 26 as i32)
        as u32
        | (GENO_SUB_GET as i32 as u32
            & 63 as u32) << 20 as i32
        | (2 as i32 as u32 & 15 as u32)
            << 16 as i32
        | ((((0 as i32 & 3 as i32)
            << 6 as i32
            | 22 as i32 & 63 as i32)
            & 0xff as i32) << 8 as i32
            | (if 0 as i32 != 0 {
                0x80 as i32
            } else {
                0 as i32
            })
            | (0 as i32 & 7 as i32)
                << 4 as i32) as u32
            & 0xffff as u32) as u32;
    let fresh205 = n_3;
    n_3 = n_3 + 1;
    *s_3.offset(fresh205 as isize) = GENO_VAL_ATTACK_CONNECTED_PREV as i32
        as u32;
    let fresh206 = n_3;
    n_3 = n_3 + 1;
    *s_3.offset(fresh206 as isize) = ((GENO_FTCMD_OP << 26 as i32)
        as u32
        | (GENO_SUB_PUT as i32 as u32
            & 63 as u32) << 20 as i32
        | (3 as i32 as u32 & 15 as u32)
            << 16 as i32
        | 0 as i32 as u32 & 0xffff as u32)
        as u32;
    let fresh207 = n_3;
    n_3 = n_3 + 1;
    *s_3.offset(fresh207 as isize) = GENO_VAL_ATTACK_CONNECTED as i32
        as u32;
    let fresh208 = n_3;
    n_3 = n_3 + 1;
    *s_3.offset(fresh208 as isize) = 1 as u32;
    let fresh209 = n_3;
    n_3 = n_3 + 1;
    *s_3.offset(fresh209 as isize) = ((GENO_FTCMD_OP << 26 as i32)
        as u32
        | (GENO_SUB_GET as i32 as u32
            & 63 as u32) << 20 as i32
        | (2 as i32 as u32 & 15 as u32)
            << 16 as i32
        | ((((0 as i32 & 3 as i32)
            << 6 as i32
            | 23 as i32 & 63 as i32)
            & 0xff as i32) << 8 as i32
            | (if 0 as i32 != 0 {
                0x80 as i32
            } else {
                0 as i32
            })
            | (0 as i32 & 7 as i32)
                << 4 as i32) as u32
            & 0xffff as u32) as u32;
    let fresh210 = n_3;
    n_3 = n_3 + 1;
    *s_3.offset(fresh210 as isize) = GENO_VAL_ATTACK_CONNECTED as i32
        as u32;
    let fresh211 = n_3;
    n_3 = n_3 + 1;
    *s_3.offset(fresh211 as isize) = 0 as u32;
    t_run(&raw mut t_script as *mut u32, GENO_MODE_EXEC);
    if (*st_1).la_i[21 as i32 as usize] != 0 as i32
        || (*st_1).la_i[22 as i32 as usize] != 1 as i32
        || (*st_1).la_i[23 as i32 as usize] != 1 as i32
    {
        TestFail(
            b"action change: ATTACK_CONNECTED 0, _PREV 1; PUT 1 writes it\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }

    let mut fw_0: f32 = 9.0f32;
    let mut up_0: f32 = 9.0f32;
    let mut fc: f32 = 1.0f32;
    if GenoGame_AimStick(
        0.1f32,
        0.1f32,
        1.0f32,
        0 as i32,
        0.25f32,
        60.0f32,
        60.0f32,
        1 as i32,
        &raw mut fw_0,
        &raw mut up_0,
        &raw mut fc,
    ) != 0 as i32 || fw_0 != 9.0f32
    {
        TestFail(
            b"aim_stick: below the threshold writes nothing\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    GenoGame_AimStick(
        0.5f32,
        0.5f32,
        1.0f32,
        0 as i32,
        0.25f32,
        60.0f32,
        60.0f32,
        1 as i32,
        &raw mut fw_0,
        &raw mut up_0,
        &raw mut fc,
    );
    if t_near(fw_0, 0.7071068f32) == 0 || t_near(up_0, 0.7071068f32) == 0 || fc != 1.0f32
    {
        TestFail(
            b"aim_stick: up-forward diagonal = 45 deg\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    GenoGame_AimStick(
        0.0f32,
        1.0f32,
        1.0f32,
        0 as i32,
        0.25f32,
        60.0f32,
        60.0f32,
        1 as i32,
        &raw mut fw_0,
        &raw mut up_0,
        &raw mut fc,
    );
    if t_near(up_0, 0.8660254f32) == 0 {
        TestFail(
            b"aim_stick: straight up clamps to max_up 60\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    GenoGame_AimStick(
        -0.8f32,
        -0.3f32,
        1.0f32,
        0 as i32,
        0.25f32,
        60.0f32,
        60.0f32,
        1 as i32,
        &raw mut fw_0,
        &raw mut up_0,
        &raw mut fc,
    );
    if fc != -1.0f32 || t_near(up_0, sinf(atan2f(-0.3f32, 0.8f32)) as f32) == 0 {
        TestFail(
            b"aim_stick: back-down turns and aims down\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    GenoGame_AimStick(
        0.8f32,
        -0.8f32,
        1.0f32,
        1 as i32,
        0.25f32,
        60.0f32,
        60.0f32,
        1 as i32,
        &raw mut fw_0,
        &raw mut up_0,
        &raw mut fc,
    );
    if t_near(up_0, 0.0f32) == 0 {
        TestFail(
            b"aim_stick: never down on the ground\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    if GenoGame_HookFind(b"geno.aim_stick\0" as *const u8 as *const i8)
        != GENO_HOOK_AIM_STICK as i32
        || GENO_HOOK_AIM_STICK as i32 != 7 as i32
    {
        TestFail(
            b"hook 7 is geno.aim_stick\0" as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    if GenoGame_HookFind(b"geno.lockon\0" as *const u8 as *const i8)
        != GENO_HOOK_LOCKON as i32
        || GENO_HOOK_LOCKON as i32 != 6 as i32
    {
        TestFail(b"hook 6 is geno.lockon\0" as *const u8 as *const i8);
        rc = 1 as i32;
    }
    return rc;
}

pub unsafe extern "C" fn test_geno_fly() -> i32 {
    melee_core::ct!("test_geno_fly");

    if GenoFly_CursorTest() != 0 {
        TestFail(
            b"debug attack cursor: native rearm, victim caches, convergence or hitlag failed\0"
                as *const u8 as *const i8,
        );
        return 1 as i32;
    }
    let mut old: f32 = GenoFly_Speed();
    let mut rc: i32 = 0 as i32;
    t_setup();
    if GenoFly_Fighter(&raw mut t_fp) != 0 {
        TestFail(
            b"an unarmed fighter reads as flying\0" as *const u8
                as *const i8,
        );
        return 1 as i32;
    }
    GenoFly_TestArm(&raw mut t_fp);
    GenoFly_SetSpeed(t_fbits(2.0f32) as i32);
    GenoFly_SetSolid(0 as i32);
    t_fp.input.lstick[0 as i32 as usize].x = 0.5f32 as f32;
    t_fp.input.lstick[0 as i32 as usize].y = -1.0f32 as f32;
    t_fp.x8c_kb_vel.x = 7.0f32 as f32;
    t_fp.facing_dir = -1.0f32;
    t_fp.dmg.x1910 = 30 as i32;
    t_fp.phys_cb.expect("non-null function pointer")(&raw mut t_gobj);
    if GenoFly_Fighter(&raw mut t_fp) == 0 || t_fp.self_vel.x != 1.0f32
        || t_fp.self_vel.y != -2.0f32 || t_fp.x8c_kb_vel.x != 0.0f32
        || t_fp.facing_dir != 1.0f32 || t_fp.x1988 != 2 as i32
        || t_fp.dmg.x1910 != 0 as i32
    {
        TestFail(
            b"fly phys: velocity = stick x speed, knockback cleared, facing the stick, intangible\0"
                as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    t_fp.input.held_buttons[0 as i32 as usize] = HSD_PAD_A as HSD_Pad;
    t_fp.phys_cb.expect("non-null function pointer")(&raw mut t_gobj);
    if t_fp.self_vel.x != 0.25f32 || t_fp.self_vel.y != -0.5f32 {
        TestFail(
            b"fly phys: A held is a quarter speed\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    t_fp.input.held_buttons[0 as i32 as usize] = HSD_PAD_B as HSD_Pad;
    t_fp.phys_cb.expect("non-null function pointer")(&raw mut t_gobj);
    if t_fp.self_vel.x != 4.0f32 || t_fp.self_vel.y != -8.0f32 {
        TestFail(
            b"fly phys: B held is 4x speed\0" as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    t_fp.input.held_buttons[0 as i32 as usize] = 0 as HSD_Pad;
    t_fp.input.lstick[0 as i32 as usize].x = 0.05f32 as f32;
    t_fp.input.lstick[0 as i32 as usize].y = -0.05f32 as f32;
    t_fp.phys_cb.expect("non-null function pointer")(&raw mut t_gobj);
    if t_fp.self_vel.x != 0.0f32 || t_fp.self_vel.y != 0.0f32 {
        TestFail(
            b"fly phys: a resting stick's drift does not move\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    GenoFly_SetSolid(1 as i32);
    t_fp.x1988 = 2 as i32 as enum_t;
    t_fp.phys_cb.expect("non-null function pointer")(&raw mut t_gobj);
    GenoFly_SetSolid(0 as i32);
    if t_fp.x1988 != 0 as i32 {
        TestFail(
            b"fly phys: solid keeps the hurtboxes\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    t_fp.cur_pos.x = 5.0f32 as f32;
    t_fp.cur_pos.y = 6.0f32 as f32;
    t_fp.coll_data.prev_pos.x = -300.0f32 as f32;
    t_fp.coll_data.env_flags = 0x7 as i32 as i32;
    t_fp.coll_cb.expect("non-null function pointer")(&raw mut t_gobj);
    if t_fp.coll_data.cur_pos.x != 5.0f32 || t_fp.coll_data.cur_pos.y != 6.0f32
        || t_fp.coll_data.prev_pos.x != 5.0f32 || t_fp.coll_data.last_pos.y != 6.0f32
        || t_fp.coll_data.env_flags != 0 as i32
    {
        TestFail(
            b"fly coll: the collision data follows the fighter, no floor / wall / ceiling\0"
                as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    t_fp.cur_pos.x = 1.0e6f32 as f32;
    t_fp.cur_pos.y = -1.0e6f32 as f32;
    if ftCo_800D3158(&raw mut t_gobj) != 0 {
        TestFail(
            b"a flying fighter past the blast zones was KO'd\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    GenoFly_SetSpeed(t_fbits(1000.0f32) as i32);
    if GenoFly_Speed() != 200.0f32 {
        TestFail(
            b"fly speed clamps to 200\0" as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    GenoFly_SetSpeed(t_fbits(old) as i32);
    t_setup();
    return rc;
}

pub unsafe extern "C" fn test_geno_define_snapshot() -> i32 {
    melee_core::ct!("test_geno_define_snapshot");
    let mut kind: i32 = Ft_Kind_Mex0 as i32;
    let mut rc: i32 = 0 as i32;
    let mut old0: u32 = GenoDefine_ContentIdentity[kind
        as usize][0 as i32 as usize];
    let mut old1: u32 = GenoDefine_ContentIdentity[kind
        as usize][1 as i32 as usize];
    let mut attempts: u32 = GenoDefine_InterpreterAttempts[kind as usize];
    GenoDefine_ContentIdentity[kind as usize][0 as i32 as usize] = 0x47454e4f
        as i32 as u32;
    GenoDefine_ContentIdentity[kind as usize][1 as i32 as usize] = 0x44454631
        as i32 as u32;
    if snap_open(1 as i32) != 0 as i32 {
        TestFail(
            b"native definition snapshot could not open\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    } else {
        snap_save(0 as i32);
        GenoDefine_ContentIdentity[kind as usize][0 as i32 as usize] = 0
            as u32;
        GenoDefine_ContentIdentity[kind as usize][1 as i32 as usize] = 0
            as u32;
        GenoGame_DefinePPCViolation(kind);
        if GenoDefine_InterpreterAttempts[kind as usize]
            != attempts.wrapping_add(1 as u32)
        {
            TestFail(
                b"native definition interpreter-attempt counter did not increment\0"
                    as *const u8 as *const i8,
            );
            rc = 1 as i32;
        }
        if snap_load(0 as i32) != 0 as i32
            || GenoDefine_ContentIdentity[kind
                as usize][0 as i32 as usize]
                != 0x47454e4f as i32 as u32
            || GenoDefine_ContentIdentity[kind
                as usize][1 as i32 as usize]
                != 0x44454631 as i32 as u32
            || GenoDefine_InterpreterAttempts[kind as usize] != attempts
        {
            TestFail(
                b"native definition identity/counter not restored by snapshot\0"
                    as *const u8 as *const i8,
            );
            rc = 1 as i32;
        }
    }
    GenoDefine_ContentIdentity[kind as usize][0 as i32 as usize] = old0;
    GenoDefine_ContentIdentity[kind as usize][1 as i32 as usize] = old1;
    GenoDefine_InterpreterAttempts[kind as usize] = attempts;
    return rc;
}

pub unsafe extern "C" fn test_geno_define_hit_tag() -> i32 {
    melee_core::ct!("test_geno_define_hit_tag");
    let mut rc: i32 = 0 as i32;
    let mut ck: i32 = 0;
    let mut kind: i32 = 0;
    t_setup();
    Geno_TestInstall(
        b"{\"geno\":6,\"fighters\":[]}\0" as *const u8 as *const i8,
    );
    t_fp.kind = Ft_Kind_Mario;
    t_fp.motion_id = 53 as i32 as FtMotionId;
    t_fp.anim_id = 46 as i32 as enum_t;
    if ScriptGame_HitFamilyMove(&raw mut t_fp) != 3 as i32 {
        TestFail(
            b"retail Mario tilt not reported as tilt\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    Geno_TestRestore();
    t_setup();
    Geno_TestInstall(
        b"{\"geno\":6,\"fighters\":[{\"attach\":\"kirby\",\"subactions\":[{\"index\":46,\"words\":[\"0x04000002\"],\"move_tag\":\"smash\"}]}]}\0"
            as *const u8 as *const i8,
    );
    t_fp.kind = Ft_Kind_Kirby;
    t_fp.motion_id = 53 as i32 as FtMotionId;
    t_fp.anim_id = 46 as i32 as enum_t;
    if ScriptGame_HitFamilyMove(&raw mut t_fp) != 3 as i32 {
        TestFail(
            b"attach fighter: motion tag must still win\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    t_fp.motion_id = 10 as i32 as FtMotionId;
    if ScriptGame_HitFamilyMove(&raw mut t_fp) != 4 as i32 {
        TestFail(
            b"attach fighter: declared overlay tag lost\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    Geno_TestRestore();
    t_setup();
    Geno_TestInstall(
        b"{\"geno\":7,\"fighters\":[{\"define\":{\"key\":\"hit-tag\",\"name\":\"Hit Tag\",\"base\":\"mario\",\"common\":\"melee.common.v1\",\"resources\":\"retail:mario\"},\"common_states\":[{\"motion\":53,\"move_tag\":\"smash\"},{\"motion\":44,\"move_tag\":\"special\"},{\"motion\":57}]}]}\0"
            as *const u8 as *const i8,
    );
    if Geno_DefineCount() != 1 as i32 {
        TestFail(
            b"define not bound for the hit-tag test\0" as *const u8
                as *const i8,
        );
        Geno_TestRestore();
        return 1 as i32;
    }
    ck = Geno_DefineCKAt(0 as i32);
    kind = ck - 1 as i32;
    t_fp.kind = kind as FighterKind;
    t_fp.anim_id = 46 as i32 as enum_t;
    t_fp.motion_id = 53 as i32 as FtMotionId;
    if ScriptGame_HitFamilyMove(&raw mut t_fp) != 4 as i32 {
        TestFail(
            b"define: declared smash on the tilt row not reported\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    t_fp.motion_id = 44 as i32 as FtMotionId;
    if ScriptGame_HitFamilyMove(&raw mut t_fp) != 8 as i32 {
        TestFail(
            b"define: declared special on the jab row not reported\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    t_fp.motion_id = 57 as i32 as FtMotionId;
    if ScriptGame_HitFamilyMove(&raw mut t_fp) != 3 as i32 {
        TestFail(
            b"define: undeclared row must keep the motion tag\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    t_fp.motion_id = 345 as i32 as FtMotionId;
    if ScriptGame_HitFamilyMove(&raw mut t_fp) != 8 as i32 {
        TestFail(
            b"define: donor special range must stay special\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    Geno_TestRestore();
    return rc;
}

pub unsafe extern "C" fn test_geno_define_state_hash() -> i32 {
    melee_core::ct!("test_geno_define_state_hash");
    let mut rc: i32 = 0 as i32;
    let mut kind: i32 = 0;
    let mut s: *mut TestGenoState = ::core::ptr::null_mut::<TestGenoState>();
    let mut d0: u32 = 0;
    let mut d1: u32 = 0;
    let mut old_seed: u32 = 0;
    old_seed = *HSD_RandSeedPtr;
    *HSD_RandSeedPtr = 0x12345678 as u32 as u32;
    if RB_GameHash() != 0x663876f6 as u32 {
        TestFail(
            b"RB_GameHash with no define changed\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    *HSD_RandSeedPtr = old_seed;
    t_setup();
    if GenoDefine_StateDigest(&raw mut t_fp) != 0 as u32 {
        TestFail(
            b"digest of a non-define fighter is not 0\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    Geno_TestInstall(
        b"{\"geno\":7,\"fighters\":[{\"define\":{\"key\":\"hash-def\",\"name\":\"Hash Def\",\"base\":\"mario\",\"common\":\"melee.common.v1\",\"resources\":\"retail:mario\"}}]}\0"
            as *const u8 as *const i8,
    );
    if Geno_DefineCount() != 1 as i32 {
        TestFail(
            b"define not bound for the hash test\0" as *const u8
                as *const i8,
        );
        Geno_TestRestore();
        return 1 as i32;
    }
    kind = Geno_DefineCKAt(0 as i32) - 1 as i32;
    t_fp.kind = kind as FighterKind;
    Geno_FighterReset(&raw mut t_fp);
    s = GenoGame_StateOf(&raw mut t_fp) as *mut TestGenoState;
    d0 = GenoDefine_StateDigest(&raw mut t_fp);
    if d0 == 0 as u32 {
        TestFail(
            b"digest of a define is 0\0" as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    (*s).move_i[0 as i32 as usize] = 7 as i32 as i32;
    d1 = GenoDefine_StateDigest(&raw mut t_fp);
    if d1 == d0 {
        TestFail(
            b"digest ignores move_i[0]\0" as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    (*s).la_i[3 as i32 as usize] = 5 as i32 as i32;
    if GenoDefine_StateDigest(&raw mut t_fp) == d1 {
        TestFail(
            b"digest ignores a variable\0" as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    (*s).la_i[3 as i32 as usize] = 0 as i32 as i32;
    d1 = GenoDefine_StateDigest(&raw mut t_fp);
    (*s).hook_calls = (*s).hook_calls.wrapping_add(3 as u32);
    (*s).resets = (*s).resets.wrapping_add(2 as u32);
    (*s).changes = (*s).changes.wrapping_add(4 as u32);
    (*s).state_entries = (*s).state_entries.wrapping_add(5 as u32);
    (*s).art_spawned += 6 as i32;
    (*s).motion_land += 8 as i32;
    if GenoDefine_StateDigest(&raw mut t_fp) != d1 {
        TestFail(
            b"digest moved with a diagnostics-only change\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    if snap_open(1 as i32) != 0 as i32 {
        TestFail(
            b"snapshot could not open for the hash test\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    } else {
        snap_save(0 as i32);
        (*s).move_i[0 as i32 as usize] = 99 as i32 as i32;
        (*s).move_f[1 as i32 as usize] = 2.5f32 as f32;
        if GenoDefine_StateDigest(&raw mut t_fp) == d1 {
            TestFail(
                b"mutation did not change the digest\0" as *const u8
                    as *const i8,
            );
            rc = 1 as i32;
        }
        if snap_load(0 as i32) != 0 as i32
            || GenoDefine_StateDigest(&raw mut t_fp) != d1
        {
            TestFail(
                b"digest not equal after snap_load\0" as *const u8
                    as *const i8,
            );
            rc = 1 as i32;
        }
    }
    Geno_TestRestore();
    return rc;
}

pub unsafe extern "C" fn test_geno_define_article_digest() -> i32 {
    melee_core::ct!("test_geno_define_article_digest");
    static mut ig: [HSD_GObj; 2] = unsafe { core::mem::zeroed() };
    static mut it: [Item; 2] = unsafe { core::mem::zeroed() };
    let mut rc: i32 = 0 as i32;
    let mut kind: i32 = 0;
    let mut prof: i32 = 0;
    let mut k: i32 = 0;
    let mut s: *mut TestGenoState = ::core::ptr::null_mut::<TestGenoState>();
    let mut d0: u32 = 0;
    let mut d1: u32 = 0;
    let mut d2: u32 = 0;
    let mut head_save: *mut HSD_GObj = ::core::ptr::null_mut::<HSD_GObj>();
    static mut fake_links: [*mut HSD_GObj; 64] = [::core::ptr::null::<HSD_GObj>()
        as *mut HSD_GObj; 64];
    let mut table_save: *mut *mut HSD_GObj = HSD_GObjPLinkHead;
    t_setup();
    Geno_TestInstall(
        b"{\"geno\":8,\"fighters\":[{\"define\":{\"key\":\"art-def\",\"name\":\"Art Def\",\"base\":\"mario\",\"common\":\"melee.common.v1\",\"resources\":\"retail:mario\"},\"articles\":[{\"name\":\"Bolt\",\"hitboxes\":[{\"damage\":5}]}]}]}\0"
            as *const u8 as *const i8,
    );
    if Geno_DefineCount() != 1 as i32 {
        TestFail(
            b"define with an article not bound for the digest test\0" as *const u8
                as *const i8,
        );
        Geno_TestRestore();
        return 1 as i32;
    }
    kind = Geno_DefineCKAt(0 as i32) - 1 as i32;
    t_fp.kind = kind as FighterKind;
    Geno_FighterReset(&raw mut t_fp);
    s = GenoGame_StateOf(&raw mut t_fp) as *mut TestGenoState;
    prof = (*s).profile as i32;
    if prof < 0 as i32
        || Geno_ArticleCount(prof) != 1 as i32
    {
        TestFail(
            b"the define's article is not registered\0" as *const u8
                as *const i8,
        );
        Geno_TestRestore();
        return 1 as i32;
    }
    HSD_GObjPLinkHead = &raw mut fake_links as *mut *mut HSD_GObj;
    head_save = ::core::ptr::null_mut::<HSD_GObj>();
    d0 = GenoDefine_StateDigest(&raw mut t_fp);
    k = 0 as i32;
    while k < 2 as i32 {
        t_zero(
            (&raw mut ig as *mut HSD_GObj).offset(k as isize) as *mut HSD_GObj
                as *mut c_void,
            ::core::mem::size_of::<HSD_GObj>() as i32,
        );
        t_zero(
            (&raw mut it as *mut Item).offset(k as isize) as *mut Item
                as *mut c_void,
            ::core::mem::size_of::<Item>() as i32,
        );
        ig[k as usize].user_data = (&raw mut it as *mut Item).offset(k as isize)
            as *mut Item as *mut c_void;
        it[k as usize].owner = &raw mut t_gobj;
        it[k as usize].kind = (if prof >= 32 as i32 {
            GENO_ART_EXTRA_BASE
                + (prof - 32 as i32) * 16 as i32
                + 0 as i32
        } else if (0 as i32) < GENO_ART_PER_RANGE {
            GENO_ART_KIND_BASE + prof * GENO_ART_PER_RANGE + 0 as i32
        } else {
            GENO_ART_KIND_BASE2 + prof * GENO_ART_PER_RANGE + 0 as i32
                - GENO_ART_PER_RANGE
        }) as ItemKind;
        (*(&raw mut (*(&raw mut it as *mut Item).offset(k as isize)).xDD4_itemVar
            as *mut GenoArtVars))
            .profile = prof as i32;
        (*(&raw mut (*(&raw mut it as *mut Item).offset(k as isize)).xDD4_itemVar
            as *mut GenoArtVars))
            .frame = (3 as i32 + k) as i32;
        it[k as usize].pos.x = (10.0f32 + k as f32) as f32;
        it[k as usize].x40_vel.x = 2.0f32 as f32;
        k += 1;
    }
    let ref mut fresh212 = *HSD_GObjPLinkHead.offset(HSD_GOBJ_PLINK_ITEM as isize);
    *fresh212 = (&raw mut ig as *mut HSD_GObj).offset(0 as i32 as isize)
        as *mut HSD_GObj;
    ig[0 as i32 as usize].next = ::core::ptr::null_mut::<HSD_GObj>();
    d1 = GenoDefine_StateDigest(&raw mut t_fp);
    if d1 == d0 || d1 == 0 as u32 {
        TestFail(
            b"a live article did not enter the digest\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    it[0 as i32 as usize].pos.x = 12.5f32 as f32;
    d2 = GenoDefine_StateDigest(&raw mut t_fp);
    if d2 == d1 {
        TestFail(
            b"the digest ignores an article's position\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    it[0 as i32 as usize].pos.x = 10.0f32 as f32;
    (*(&raw mut (*(&raw mut it as *mut Item).offset(0 as i32 as isize))
        .xDD4_itemVar as *mut GenoArtVars))
        .frame = 9 as i32 as i32;
    if GenoDefine_StateDigest(&raw mut t_fp) == d1 {
        TestFail(
            b"the digest ignores an article's age\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    (*(&raw mut (*(&raw mut it as *mut Item).offset(0 as i32 as isize))
        .xDD4_itemVar as *mut GenoArtVars))
        .frame = 3 as i32 as i32;
    if GenoDefine_StateDigest(&raw mut t_fp) != d1 {
        TestFail(
            b"the digest is not a function of the article state\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    let ref mut fresh213 = *HSD_GObjPLinkHead.offset(HSD_GOBJ_PLINK_ITEM as isize);
    *fresh213 = (&raw mut ig as *mut HSD_GObj).offset(0 as i32 as isize)
        as *mut HSD_GObj;
    ig[0 as i32 as usize].next = (&raw mut ig as *mut HSD_GObj)
        .offset(1 as i32 as isize) as *mut HSD_GObj as *mut HSD_GObj;
    ig[1 as i32 as usize].next = ::core::ptr::null_mut::<HSD_GObj>();
    d1 = GenoDefine_StateDigest(&raw mut t_fp);
    let ref mut fresh214 = *HSD_GObjPLinkHead.offset(HSD_GOBJ_PLINK_ITEM as isize);
    *fresh214 = (&raw mut ig as *mut HSD_GObj).offset(1 as i32 as isize)
        as *mut HSD_GObj;
    ig[1 as i32 as usize].next = (&raw mut ig as *mut HSD_GObj)
        .offset(0 as i32 as isize) as *mut HSD_GObj as *mut HSD_GObj;
    ig[0 as i32 as usize].next = ::core::ptr::null_mut::<HSD_GObj>();
    d2 = GenoDefine_StateDigest(&raw mut t_fp);
    if d1 != d2 {
        TestFail(
            b"the digest depends on the item list order\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    it[0 as i32 as usize].owner = ::core::ptr::null_mut::<HSD_GObj>();
    it[1 as i32 as usize].owner = ::core::ptr::null_mut::<HSD_GObj>();
    if GenoDefine_StateDigest(&raw mut t_fp) != d0 {
        TestFail(
            b"an article this fighter does not own moved its digest\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    let ref mut fresh215 = *HSD_GObjPLinkHead.offset(HSD_GOBJ_PLINK_ITEM as isize);
    *fresh215 = head_save;
    ig[1 as i32 as usize].next = ::core::ptr::null_mut::<HSD_GObj>();
    ig[0 as i32 as usize].next = ig[1 as i32 as usize]
        .next;
    if GenoDefine_StateDigest(&raw mut t_fp) != d0 {
        TestFail(
            b"the digest did not return to the no-article value\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    HSD_GObjPLinkHead = table_save;
    Geno_TestRestore();
    return rc;
}

pub unsafe extern "C" fn t_rbmix(mut h: u32, mut v: u32) -> u32 {
    melee_core::ct!("t_rbmix");
    h ^= v;
    h = (h as u32).wrapping_mul(0x1000193 as u32)
        as u32 as u32;
    h ^= h >> 15 as i32;
    return h;
}

pub unsafe extern "C" fn t_rbbits(mut f: f32) -> u32 {
    melee_core::ct!("t_rbbits");
    let mut u: u32 = 0;
    memcpy(
        &raw mut u as *mut c_void,
        &raw mut f as *const c_void,
        4 as usize,
    );
    return u;
}

pub unsafe extern "C" fn t_rb_old_formula(mut fp: *mut Fighter, mut h: u32) -> u32 {
    melee_core::ct!("t_rb_old_formula");
    h = t_rbmix(h, 0x100 as u32);
    h = t_rbmix(h, (*fp).motion_id as u32);
    h = t_rbmix(h, t_rbbits((*fp).cur_pos.x));
    h = t_rbmix(h, t_rbbits((*fp).cur_pos.y));
    h = t_rbmix(h, t_rbbits((*fp).cur_pos.z));
    h = t_rbmix(h, t_rbbits((*fp).self_vel.x));
    h = t_rbmix(h, t_rbbits((*fp).self_vel.y));
    h = t_rbmix(h, t_rbbits((*fp).dmg.x1830_percent as f32));
    h = t_rbmix(h, t_rbbits((*fp).facing_dir as f32));
    h = t_rbmix(h, 0 as u32);
    return h;
}

pub unsafe extern "C" fn test_rb_game_hash() -> i32 {
    melee_core::ct!("test_rb_game_hash");
    let mut rc: i32 = 0 as i32;
    let mut base: u32 = 0;
    let mut v: u32 = 0;
    let mut n: u32 = 0;
    t_setup();
    t_fp.player_id = 0 as u8;
    t_fp.kind = Ft_Kind_Mario;
    t_fp.motion_id = 14 as i32 as FtMotionId;
    t_fp.cur_pos.x = 3.5f32 as f32;
    t_fp.self_vel.y = -1.25f32 as f32;
    t_fp.facing_dir = 1.0f32;
    RB_HashSetLegacy(1 as i32);
    if RB_FighterHash(0x811c9dc5 as u32, &raw mut t_fp, 0 as i32)
        != t_rb_old_formula(&raw mut t_fp, 0x811c9dc5 as u32)
    {
        TestFail(
            b"rb hash: legacy mode no longer reproduces the pre-widening value\0"
                as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    RB_HashSetLegacy(0 as i32);
    base = RB_FighterHash(0x811c9dc5 as u32, &raw mut t_fp, 0 as i32);
    if base == t_rb_old_formula(&raw mut t_fp, 0x811c9dc5 as u32) {
        TestFail(
            b"rb hash: widened equals legacy\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    if RB_FighterHash(0x811c9dc5 as u32, &raw mut t_fp, 0 as i32)
        != base
    {
        TestFail(
            b"rb hash: not a pure function of the state\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    let mut save_: Fighter = t_fp;
    t_fp.ground_or_air = GA_Air;
    v = RB_FighterHash(0x811c9dc5 as u32, &raw mut t_fp, 0 as i32);
    if v == base {
        TestFail(
            b"rb hash: widened field does not change the hash: ground_or_air\0"
                as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    t_fp = save_;
    let mut save__0: Fighter = t_fp;
    t_fp.x8c_kb_vel.x = 2.0f32 as f32;
    v = RB_FighterHash(0x811c9dc5 as u32, &raw mut t_fp, 0 as i32);
    if v == base {
        TestFail(
            b"rb hash: widened field does not change the hash: kb_vel.x\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    t_fp = save__0;
    let mut save__1: Fighter = t_fp;
    t_fp.x8c_kb_vel.y = 2.0f32 as f32;
    v = RB_FighterHash(0x811c9dc5 as u32, &raw mut t_fp, 0 as i32);
    if v == base {
        TestFail(
            b"rb hash: widened field does not change the hash: kb_vel.y\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    t_fp = save__1;
    let mut save__2: Fighter = t_fp;
    t_fp.gr_vel = 0.75f32;
    v = RB_FighterHash(0x811c9dc5 as u32, &raw mut t_fp, 0 as i32);
    if v == base {
        TestFail(
            b"rb hash: widened field does not change the hash: gr_vel\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    t_fp = save__2;
    let mut save__3: Fighter = t_fp;
    t_fp.dmg.x195c_hitlag_frames = 3.0f32;
    v = RB_FighterHash(0x811c9dc5 as u32, &raw mut t_fp, 0 as i32);
    if v == base {
        TestFail(
            b"rb hash: widened field does not change the hash: hitlag\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    t_fp = save__3;
    let mut save__4: Fighter = t_fp;
    t_fp.set_x2219_b5(1 as u8 as u8);
    v = RB_FighterHash(0x811c9dc5 as u32, &raw mut t_fp, 0 as i32);
    if v == base {
        TestFail(
            b"rb hash: widened field does not change the hash: x2219_b5\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    t_fp = save__4;
    let mut save__5: Fighter = t_fp;
    t_fp.shield_health = 12.0f32;
    v = RB_FighterHash(0x811c9dc5 as u32, &raw mut t_fp, 0 as i32);
    if v == base {
        TestFail(
            b"rb hash: widened field does not change the hash: shield_health\0"
                as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    t_fp = save__5;
    let mut save__6: Fighter = t_fp;
    t_fp.x3E4_fighterCmdScript.frame_count = 7.0f32 as f32;
    v = RB_FighterHash(0x811c9dc5 as u32, &raw mut t_fp, 0 as i32);
    if v == base {
        TestFail(
            b"rb hash: widened field does not change the hash: cmd frame\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    t_fp = save__6;
    let mut save__7: Fighter = t_fp;
    t_fp.x1968_jumpsUsed = 1 as u8;
    v = RB_FighterHash(0x811c9dc5 as u32, &raw mut t_fp, 0 as i32);
    if v == base {
        TestFail(
            b"rb hash: widened field does not change the hash: jumpsUsed\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    t_fp = save__7;
    let mut save__8: Fighter = t_fp;
    t_fp.x67F = 2 as u8;
    v = RB_FighterHash(0x811c9dc5 as u32, &raw mut t_fp, 0 as i32);
    if v == base {
        TestFail(
            b"rb hash: widened field does not change the hash: x67F\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    t_fp = save__8;
    let mut save__9: Fighter = t_fp;
    t_fp.x680 = 2 as u8;
    v = RB_FighterHash(0x811c9dc5 as u32, &raw mut t_fp, 0 as i32);
    if v == base {
        TestFail(
            b"rb hash: widened field does not change the hash: x680\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    t_fp = save__9;
    let mut save__10: Fighter = t_fp;
    t_fp.self_vel.y = -0.0f32 as f32;
    t_fp.x8c_kb_vel.x = -0.0f32 as f32;
    v = RB_FighterHash(0x811c9dc5 as u32, &raw mut t_fp, 0 as i32);
    if v == base {
        TestFail(
            b"rb hash: widened field does not change the hash: float by bit pattern\0"
                as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    t_fp = save__10;
    let mut save__11: Fighter = t_fp;
    let ref mut fresh216 = *(&raw mut t_fp.x20A4 as *mut u8)
        .offset(0 as i32 as isize);
    *fresh216 = (*fresh216 as i32 ^ 1 as i32) as u8;
    t_fp.set_x2224_b0((t_fp.x2224_b0() == 0) as i32 as u8 as u8);
    t_fp.x2144 += 5 as i32;
    t_fp.x214C += 5 as i32;
    t_fp.x2160 += 5 as i32;
    if RB_FighterHash(0x811c9dc5 as u32, &raw mut t_fp, 0 as i32)
        != base
    {
        TestFail(
            b"rb hash: a render-owned field changed the hash\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    t_fp = save__11;
    v = RB_ItemHash(&raw mut n);
    if n != 0 as u32 || v != 0 as u32 {
        TestFail(
            b"rb hash: item word nonzero with no items\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    RB_HashSetLegacy(-(1 as i32));
    return rc;
}

pub unsafe extern "C" fn tl_setup(
    mut json: *const i8,
    mut st: *mut *mut TestGenoState,
) -> i32 {
    melee_core::ct!("tl_setup");
    let mut kind: i32 = 0;
    t_zero(
        &raw mut t_rows as *mut MotionState as *mut c_void,
        ::core::mem::size_of::<[MotionState; 400]>() as i32,
    );
    if Geno_TestInstall(json) != 1 as i32
        || Geno_DefineCount() != 1 as i32
    {
        TestFail(
            b"fighter Lua: the test define was not installed\0" as *const u8
                as *const i8,
        );
        return 0 as i32;
    }
    kind = Geno_DefineCKAt(0 as i32) - 1 as i32;
    *st = t_setup();
    t_fp.kind = kind as FighterKind;
    t_fp.ground_or_air = GA_Ground;
    Geno_FighterReset(&raw mut t_fp);
    *st = GenoGame_StateOf(&raw mut t_fp) as *mut TestGenoState;
    return 1 as i32;
}

pub unsafe extern "C" fn test_geno_lua_charge() -> i32 {
    melee_core::ct!("test_geno_lua_charge");
    let mut st: *mut TestGenoState = ::core::ptr::null_mut::<TestGenoState>();
    let mut lb: *mut GenoLuaBlock = ::core::ptr::null_mut::<GenoLuaBlock>();
    let mut row: *mut MotionState = ::core::ptr::null_mut::<MotionState>();
    let mut rc: i32 = 0 as i32;
    let mut f: i32 = 0;
    if tl_setup(
        b"{\"geno\":10,\"fighters\":[{\"define\":{\"key\":\"lua-charge\",\"name\":\"Lua T\",\"base\":\"mario\",\"common\":\"melee.common.v1\",\"resources\":\"retail:mario\"},\"lua\":{\"source\":\"local M = {} function M.enter(ctx) ctx.state.charge = 0 end function M.frame(ctx) local s = ctx.state if ctx.input.special_held and s.charge < 60 then s.charge = s.charge + 1 s.armed = s.charge >= 3 else ctx.go('Release') end end function M.rel(ctx) ctx.velocity(1.6 + 0.03 * ctx.state.charge, 0) end function M.relf(ctx) ctx.hitbox_damage(3, 6 + 0.15 * ctx.state.charge) end function M.boom(ctx) ctx.state.charge = 77 error('boom') end return M\",\"state\":{\"charge\":\"int\",\"armed\":\"bool\"}},\"states\":[{\"name\":\"Charge\",\"behavior\":\"geno.ground\",\"lua\":{\"enter\":\"enter\",\"frame\":\"frame\"}},{\"name\":\"Release\",\"behavior\":\"geno.ground\",\"lua\":{\"enter\":\"rel\",\"frame\":\"relf\"}},{\"name\":\"Bad\",\"behavior\":\"geno.ground\",\"lua\":{\"frame\":\"boom\"}}],\"specials\":{\"n\":\"geno:Charge\",\"s\":\"geno:Bad\"}}]}\0"
            as *const u8 as *const i8,
        &raw mut st,
    ) == 0
    {
        Geno_TestRestore();
        return 1 as i32;
    }
    GenoGame_TestCapture(1 as i32, 0 as i32);
    lb = GenoGame_LuaBlockOf(&raw mut t_fp) as *mut GenoLuaBlock;
    (*lb).slot[0 as i32 as usize] = 41 as i32 as i32;
    if Geno_SpecialEnter(&raw mut t_gobj, GENO_SP_N as i32) == 0
        || GenoGame_TestChanges() != 1 as i32
    {
        TestFail(
            b"fighter Lua: the neutral special did not enter the Charge state\0"
                as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    if (*lb).slot[0 as i32 as usize] != 0 as i32
        || (*lb).slot[1 as i32 as usize] != 0 as i32
    {
        TestFail(
            b"fighter Lua: the Charge state's \"enter\" must run once and zero the charge\0"
                as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    row = Geno_MotionRow(&raw mut t_fp, GENO_MOTION_BASE + 0 as i32);
    if row.is_null() || (*row).anim_cb.is_none() {
        TestFail(
            b"fighter Lua: the Charge row has no anim callback\0" as *const u8
                as *const i8,
        );
        Geno_TestRestore();
        return 1 as i32;
    }
    t_fp.motion_id = (GENO_MOTION_BASE + 0 as i32) as FtMotionId;
    t_fp.input.held_buttons[0 as i32 as usize] = HSD_PAD_B as HSD_Pad;
    f = 0 as i32;
    while f < 5 as i32 {
        (*row).anim_cb.expect("non-null function pointer")(&raw mut t_gobj);
        f += 1;
    }
    if (*lb).slot[0 as i32 as usize] != 5 as i32
        || (*lb).slot[1 as i32 as usize] != 1 as i32
        || GenoGame_TestChanges() != 1 as i32
    {
        TestFail(
            b"fighter Lua: five held frames must count 5, arm, and not change action\0"
                as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    t_fp.input.held_buttons[0 as i32 as usize] = 0 as HSD_Pad;
    (*row).anim_cb.expect("non-null function pointer")(&raw mut t_gobj);
    if GenoGame_TestChanges() != 2 as i32
        || GenoGame_TestLastTarget()
            != (2 as u32 & 15 as u32)
                << 28 as i32
                | 1 as i32 as u32
                    & 0xffff as u32
        || (*lb).slot[0 as i32 as usize] != 5 as i32
    {
        TestFail(
            b"fighter Lua: releasing must go to Release with the charge kept\0"
                as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    let mut want: f32 = (1.6f64
        + 0.03f64 * 5 as i32 as f64) as f32;
    let mut d: f32 = t_fp.self_vel.x - want;
    if d < -0.0001f32 || d > 0.0001f32
        || t_fp.gr_vel < want as f32 - 0.0001f32
        || t_fp.gr_vel > want as f32 + 0.0001f32
    {
        TestFail(
            b"fighter Lua: the Release dash must be 1.6 + 0.03 x charge forward\0"
                as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    row = Geno_MotionRow(&raw mut t_fp, GENO_MOTION_BASE + 1 as i32);
    t_fp.motion_id = (GENO_MOTION_BASE + 1 as i32) as FtMotionId;
    (*row).anim_cb.expect("non-null function pointer")(&raw mut t_gobj);
    let mut want_0: f32 = (6 as i32 as f64
        + 0.15f64 * 5 as i32 as f64) as f32;
    let mut d_0: f32 = t_fp.x914[0 as i32 as usize].damage as f32
        - want_0;
    if d_0 < -0.0001f32 || d_0 > 0.0001f32
        || t_fp.x914[1 as i32 as usize].damage
            < want_0 as f32 - 0.0001f32
        || t_fp.x914[1 as i32 as usize].damage
            > want_0 as f32 + 0.0001f32
    {
        TestFail(
            b"fighter Lua: Release frame must set hitboxes 0 and 1 to 6 + 0.15 x charge\0"
                as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    if t_fp.x914[2 as i32 as usize].damage != 0.0f32 {
        TestFail(
            b"fighter Lua: hitbox_damage(3, ..) must not touch hitbox 2\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    (*lb).slot[0 as i32 as usize] = 60 as i32 as i32;
    t_fp.motion_id = (GENO_MOTION_BASE + 0 as i32) as FtMotionId;
    t_fp.input.held_buttons[0 as i32 as usize] = HSD_PAD_B as HSD_Pad;
    (*Geno_MotionRow(&raw mut t_fp, GENO_MOTION_BASE + 0 as i32))
        .anim_cb
        .expect("non-null function pointer")(&raw mut t_gobj);
    if (*lb).slot[0 as i32 as usize] != 60 as i32
        || GenoGame_TestLastTarget()
            != (2 as u32 & 15 as u32)
                << 28 as i32
                | 1 as i32 as u32
                    & 0xffff as u32
    {
        TestFail(
            b"fighter Lua: at 60 frames the charge ends\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    Geno_FighterReset(&raw mut t_fp);
    if (*lb).slot[0 as i32 as usize] != 0 as i32
        || (*lb).faults != 0 as u32
    {
        TestFail(
            b"fighter Lua: Geno_FighterReset must clear the Lua block\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    GenoGame_TestCapture(0 as i32, 0 as i32);
    Geno_TestRestore();
    return rc;
}

pub unsafe extern "C" fn test_geno_lua_fault() -> i32 {
    melee_core::ct!("test_geno_lua_fault");
    let mut st: *mut TestGenoState = ::core::ptr::null_mut::<TestGenoState>();
    let mut lb: *mut GenoLuaBlock = ::core::ptr::null_mut::<GenoLuaBlock>();
    let mut row: *mut MotionState = ::core::ptr::null_mut::<MotionState>();
    let mut d0: u32 = 0;
    let mut d1: u32 = 0;
    let mut rc: i32 = 0 as i32;
    if tl_setup(
        b"{\"geno\":10,\"fighters\":[{\"define\":{\"key\":\"lua-fault\",\"name\":\"Lua T\",\"base\":\"mario\",\"common\":\"melee.common.v1\",\"resources\":\"retail:mario\"},\"lua\":{\"source\":\"local M = {} function M.enter(ctx) ctx.state.charge = 0 end function M.frame(ctx) local s = ctx.state if ctx.input.special_held and s.charge < 60 then s.charge = s.charge + 1 s.armed = s.charge >= 3 else ctx.go('Release') end end function M.rel(ctx) ctx.velocity(1.6 + 0.03 * ctx.state.charge, 0) end function M.relf(ctx) ctx.hitbox_damage(3, 6 + 0.15 * ctx.state.charge) end function M.boom(ctx) ctx.state.charge = 77 error('boom') end return M\",\"state\":{\"charge\":\"int\",\"armed\":\"bool\"}},\"states\":[{\"name\":\"Charge\",\"behavior\":\"geno.ground\",\"lua\":{\"enter\":\"enter\",\"frame\":\"frame\"}},{\"name\":\"Release\",\"behavior\":\"geno.ground\",\"lua\":{\"enter\":\"rel\",\"frame\":\"relf\"}},{\"name\":\"Bad\",\"behavior\":\"geno.ground\",\"lua\":{\"frame\":\"boom\"}}],\"specials\":{\"n\":\"geno:Charge\",\"s\":\"geno:Bad\"}}]}\0"
            as *const u8 as *const i8,
        &raw mut st,
    ) == 0
    {
        Geno_TestRestore();
        return 1 as i32;
    }
    GenoGame_TestCapture(1 as i32, 0 as i32);
    lb = GenoGame_LuaBlockOf(&raw mut t_fp) as *mut GenoLuaBlock;
    (*lb).slot[0 as i32 as usize] = 3 as i32 as i32;
    t_fp.motion_id = (GENO_MOTION_BASE + 2 as i32) as FtMotionId;
    row = Geno_MotionRow(&raw mut t_fp, GENO_MOTION_BASE + 2 as i32);
    d0 = GenoDefine_StateDigest(&raw mut t_fp);
    (*row).anim_cb.expect("non-null function pointer")(&raw mut t_gobj);
    if (*lb).slot[0 as i32 as usize] != 3 as i32 {
        TestFail(
            b"fighter Lua: a faulting call must not keep its state writes\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    if (*lb).faults != 1 as u32
        || (*lb).last_fault != GENO_LUA_FAULT_ERROR as i32
    {
        TestFail(
            b"fighter Lua: the fault must be counted (faults 1, last GENO_LUA_FAULT_ERROR)\0"
                as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    if GenoGame_TestChanges() != 1 as i32
        || GenoGame_TestLastTarget()
            != (0 as u32 & 15 as u32)
                << 28 as i32
                | ftCo_MS_Wait as i32 as u32
                    & 0xffff as u32
    {
        TestFail(
            b"fighter Lua: a fault must send a grounded fighter to Wait\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    d1 = GenoDefine_StateDigest(&raw mut t_fp);
    if d0 == d1 {
        TestFail(
            b"fighter Lua: the fault count must be in the digest\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    GenoGame_TestCapture(0 as i32, 0 as i32);
    Geno_TestRestore();
    return rc;
}

pub unsafe extern "C" fn test_geno_lua_digest() -> i32 {
    melee_core::ct!("test_geno_lua_digest");
    let mut st: *mut TestGenoState = ::core::ptr::null_mut::<TestGenoState>();
    let mut lb: *mut GenoLuaBlock = ::core::ptr::null_mut::<GenoLuaBlock>();
    let mut d0: u32 = 0;
    let mut d1: u32 = 0;
    let mut d2: u32 = 0;
    let mut rc: i32 = 0 as i32;
    if tl_setup(
        b"{\"geno\":10,\"fighters\":[{\"define\":{\"key\":\"lua-plain\",\"name\":\"Lua T\",\"base\":\"mario\",\"common\":\"melee.common.v1\",\"resources\":\"retail:mario\"}}]}\0"
            as *const u8 as *const i8,
        &raw mut st,
    ) == 0
    {
        Geno_TestRestore();
        return 1 as i32;
    }
    lb = GenoGame_LuaBlockOf(&raw mut t_fp) as *mut GenoLuaBlock;
    d0 = GenoDefine_StateDigest(&raw mut t_fp);
    (*lb).slot[0 as i32 as usize] = 12345 as i32 as i32;
    (*lb).faults = 9 as u32;
    if d0 == 0 as u32 || GenoDefine_StateDigest(&raw mut t_fp) != d0 {
        TestFail(
            b"fighter Lua: a define with no lua block must keep its digest whatever the block holds\0"
                as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    Geno_TestRestore();
    if tl_setup(
        b"{\"geno\":10,\"fighters\":[{\"define\":{\"key\":\"lua-digest\",\"name\":\"Lua T\",\"base\":\"mario\",\"common\":\"melee.common.v1\",\"resources\":\"retail:mario\"},\"lua\":{\"source\":\"local M = {} function M.enter(ctx) ctx.state.charge = 0 end function M.frame(ctx) local s = ctx.state if ctx.input.special_held and s.charge < 60 then s.charge = s.charge + 1 s.armed = s.charge >= 3 else ctx.go('Release') end end function M.rel(ctx) ctx.velocity(1.6 + 0.03 * ctx.state.charge, 0) end function M.relf(ctx) ctx.hitbox_damage(3, 6 + 0.15 * ctx.state.charge) end function M.boom(ctx) ctx.state.charge = 77 error('boom') end return M\",\"state\":{\"charge\":\"int\",\"armed\":\"bool\"}},\"states\":[{\"name\":\"Charge\",\"behavior\":\"geno.ground\",\"lua\":{\"enter\":\"enter\",\"frame\":\"frame\"}},{\"name\":\"Release\",\"behavior\":\"geno.ground\",\"lua\":{\"enter\":\"rel\",\"frame\":\"relf\"}},{\"name\":\"Bad\",\"behavior\":\"geno.ground\",\"lua\":{\"frame\":\"boom\"}}],\"specials\":{\"n\":\"geno:Charge\",\"s\":\"geno:Bad\"}}]}\0"
            as *const u8 as *const i8,
        &raw mut st,
    ) == 0
    {
        Geno_TestRestore();
        return 1 as i32;
    }
    lb = GenoGame_LuaBlockOf(&raw mut t_fp) as *mut GenoLuaBlock;
    d0 = GenoDefine_StateDigest(&raw mut t_fp);
    (*lb).slot[0 as i32 as usize] = 7 as i32 as i32;
    d1 = GenoDefine_StateDigest(&raw mut t_fp);
    if d0 == 0 as u32 || d1 == d0 {
        TestFail(
            b"fighter Lua: a slot must move the digest of a define that declares a layout\0"
                as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    (*lb).slot[0 as i32 as usize] = 0 as i32 as i32;
    if GenoDefine_StateDigest(&raw mut t_fp) != d0 {
        TestFail(
            b"fighter Lua: the digest must return when the slot does\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    (*lb).slot[0 as i32 as usize] = 7 as i32 as i32;
    if snap_open(1 as i32) != 0 as i32 {
        TestFail(
            b"fighter Lua: the snapshot could not open\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    } else {
        snap_save(0 as i32);
        (*lb).slot[0 as i32 as usize] = 99 as i32 as i32;
        (*lb).slot[1 as i32 as usize] = 1 as i32 as i32;
        (*lb).faults = 4 as u32;
        d2 = GenoDefine_StateDigest(&raw mut t_fp);
        if d2 == d1 {
            TestFail(
                b"fighter Lua: the mutation did not move the digest\0" as *const u8
                    as *const i8,
            );
            rc = 1 as i32;
        }
        if snap_load(0 as i32) != 0 as i32
            || (*lb).slot[0 as i32 as usize] != 7 as i32
            || (*lb).slot[1 as i32 as usize] != 0 as i32
            || (*lb).faults != 0 as u32 || GenoDefine_StateDigest(&raw mut t_fp) != d1
        {
            TestFail(
                b"fighter Lua: a savestate must restore the typed state and the digest\0"
                    as *const u8 as *const i8,
            );
            rc = 1 as i32;
        }
    }
    t_setup();
    let mut kb: *mut GenoLuaBlock = GenoGame_LuaBlockOf(&raw mut t_fp)
        as *mut GenoLuaBlock;
    (*kb).slot[0 as i32 as usize] = 5 as i32 as i32;
    if GenoDefine_StateDigest(&raw mut t_fp) != 0 as u32 {
        TestFail(
            b"fighter Lua: a non-define fighter's digest must stay 0\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    Geno_TestRestore();
    return rc;
}

pub unsafe extern "C" fn test_geno_lua_counter() -> i32 {
    melee_core::ct!("test_geno_lua_counter");
    let mut st: *mut TestGenoState = ::core::ptr::null_mut::<TestGenoState>();
    let mut lb: *mut GenoLuaBlock = ::core::ptr::null_mut::<GenoLuaBlock>();
    let mut rc: i32 = 0 as i32;
    let mut want: f32 = 0.;
    if tl_setup(
        b"{\"geno\":10,\"fighters\":[{\"define\":{\"key\":\"lua-counter\",\"name\":\"Lua T\",\"base\":\"mario\",\"common\":\"melee.common.v1\",\"resources\":\"retail:mario\"},\"lua\":{\"source\":\"local M = {} function M.answer(ctx) local s = ctx.state s.power = math.min(30, math.max(9, 1.5 * ctx.self.hit_damage)) s.seen = ctx.self.countered s.from = ctx.self.hit_from end function M.strike(ctx) ctx.hitbox_damage(3, ctx.state.power) end function M.spin(ctx) ctx.turn() ctx.velocity(2, 0) end return M\",\"state\":{\"power\":\"float\",\"seen\":\"int\",\"from\":\"int\"}},\"states\":[{\"name\":\"Parry\",\"behavior\":\"geno.ground\",\"counter\":{\"from\":2,\"to\":9,\"target\":\"geno:Answer\"}},{\"name\":\"Answer\",\"behavior\":\"geno.ground\",\"lua\":{\"enter\":\"answer\",\"frame\":\"strike\"}},{\"name\":\"Spin\",\"behavior\":\"geno.ground\",\"lua\":{\"enter\":\"spin\"}}],\"specials\":{\"lw\":\"geno:Parry\",\"s\":\"geno:Spin\"}}]}\0"
            as *const u8 as *const i8,
        &raw mut st,
    ) == 0
    {
        Geno_TestRestore();
        return 1 as i32;
    }
    GenoGame_TestCapture(1 as i32, 0 as i32);
    lb = GenoGame_LuaBlockOf(&raw mut t_fp) as *mut GenoLuaBlock;
    (*st).hit_damage = 12.0f32 as f32;
    (*st).hit_port = -(1 as i32) as i32;
    if Geno_SpecialEnter(&raw mut t_gobj, GENO_SP_LW as i32) == 0 {
        TestFail(
            b"fighter Lua counter: the down special did not enter Parry\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    t_fp.motion_id = (GENO_MOTION_BASE + 0 as i32) as FtMotionId;
    (*st).action_time = 4 as i32 as i32;
    t_fp.dmg.x1838_percentTemp = 12.0f32;
    Geno_OnHit(&raw mut t_gobj);
    want = 18.0f32 as f32;
    if (*lb).slot[1 as i32 as usize] != 1 as i32
        || (*lb).slot[2 as i32 as usize] != 0 as i32
        || GenoGame_TestLastTarget()
            != (2 as u32 & 15 as u32)
                << 28 as i32
                | 1 as i32 as u32
                    & 0xffff as u32
    {
        TestFail(
            b"fighter Lua counter: the answer must be entered with countered 1 and hit_from 0\0"
                as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    let mut u: Anon_anon_294 = unsafe { core::mem::zeroed() };
    u.i = (*lb).slot[0 as i32 as usize];
    if u.f < want as f32 - 0.0001f32
        || u.f > want as f32 + 0.0001f32
    {
        TestFail(
            b"fighter Lua counter: power must be 1.5 x the countered hit (18)\0"
                as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    (*st).hit_damage = 2.0f32 as f32;
    Geno_MotionRow(&raw mut t_fp, GENO_MOTION_BASE + 1 as i32);
    t_fp.motion_id = (GENO_MOTION_BASE + 0 as i32) as FtMotionId;
    (*st).action_time = 4 as i32 as i32;
    t_fp.dmg.x1838_percentTemp = 2.0f32;
    Geno_OnHit(&raw mut t_gobj);
    let mut u_0: Anon_anon_294 = unsafe { core::mem::zeroed() };
    u_0.i = (*lb).slot[0 as i32 as usize];
    if u_0.f != 9.0f32 {
        TestFail(
            b"fighter Lua counter: a 2 percent hit must answer with the floor 9\0"
                as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    (*st).action_time = 4 as i32 as i32;
    t_fp.motion_id = (GENO_MOTION_BASE + 0 as i32) as FtMotionId;
    t_fp.dmg.x1838_percentTemp = 40.0f32;
    Geno_OnHit(&raw mut t_gobj);
    let mut u_1: Anon_anon_294 = unsafe { core::mem::zeroed() };
    u_1.i = (*lb).slot[0 as i32 as usize];
    if u_1.f != 30.0f32 {
        TestFail(
            b"fighter Lua counter: a 40 percent hit must answer with the cap 30\0"
                as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    t_fp.motion_id = (GENO_MOTION_BASE + 1 as i32) as FtMotionId;
    (*Geno_MotionRow(&raw mut t_fp, GENO_MOTION_BASE + 1 as i32))
        .anim_cb
        .expect("non-null function pointer")(&raw mut t_gobj);
    if t_fp.x914[0 as i32 as usize].damage != 30.0f32
        || t_fp.x914[1 as i32 as usize].damage != 30.0f32
        || t_fp.x914[2 as i32 as usize].damage != 0.0f32
    {
        TestFail(
            b"fighter Lua counter: the strike frame must set hitboxes 0 and 1 to power 30\0"
                as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    t_fp.facing_dir = 1.0f32;
    t_fp.ground_or_air = GA_Ground;
    if Geno_SpecialEnter(&raw mut t_gobj, GENO_SP_S as i32) == 0
        || t_fp.facing_dir != -1.0f32
    {
        TestFail(
            b"fighter Lua counter: ctx.turn must flip the facing\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    if t_fp.gr_vel != -2.0f32 {
        TestFail(
            b"fighter Lua counter: a velocity queued after turn is relative to the new facing\0"
                as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    GenoGame_TestCapture(0 as i32, 0 as i32);
    Geno_TestRestore();
    return rc;
}

pub unsafe extern "C" fn tc_setup(
    mut json: *const i8,
) -> i32 {
    melee_core::ct!("tc_setup");
    let mut kind: i32 = 0;
    t_zero(
        &raw mut t_rows as *mut MotionState as *mut c_void,
        ::core::mem::size_of::<[MotionState; 400]>() as i32,
    );
    if Geno_TestInstall(json) != 1 as i32
        || Geno_DefineCount() != 1 as i32
    {
        return 0 as i32;
    }
    kind = Geno_DefineCKAt(0 as i32) - 1 as i32;
    tc_st = t_setup();
    t_fp.kind = kind as FighterKind;
    t_fp.ground_or_air = GA_Ground;
    Geno_FighterReset(&raw mut t_fp);
    tc_st = GenoGame_StateOf(&raw mut t_fp) as *mut TestGenoState;
    return 1 as i32;
}

pub unsafe extern "C" fn tc_frame(
    mut pressed: HSD_Pad,
    mut sx: f32,
    mut sy: f32,
    mut hitlag: i32,
) {
    melee_core::ct!("tc_frame");
    t_fp.input.pressed_buttons = pressed;
    t_fp.input.held_buttons[0 as i32 as usize] = pressed;
    t_fp.input.lstick[0 as i32 as usize].x = sx;
    t_fp.input.lstick[0 as i32 as usize].y = sy;
    t_fp.input.cstick[0 as i32 as usize].y = 0.0f32 as f32;
    t_fp.input.cstick[0 as i32 as usize].x = t_fp
        .input
        .cstick[0 as i32 as usize]
        .y;
    t_fp.input.cstick[1 as i32 as usize].y = 0.0f32 as f32;
    t_fp.input.cstick[1 as i32 as usize].x = t_fp
        .input
        .cstick[1 as i32 as usize]
        .y;
    t_fp.active_timer.lstick.x = (if sx != 0.0f32 {
        0 as i32
    } else {
        50 as i32
    }) as u8;
    t_fp.active_timer.lstick.y = (if sy != 0.0f32 {
        0 as i32
    } else {
        50 as i32
    }) as u8;
    t_fp.set_x2219_b5(
        (if hitlag != 0 { 1 as i32 } else { 0 as i32 })
            as u8 as u8,
    );
    Geno_ComboInput(&raw mut t_gobj);
    t_fp.set_x2219_b5(0 as u8 as u8);
}

pub unsafe extern "C" fn tc_enter_motion(mut motion: i32) {
    melee_core::ct!("tc_enter_motion");
    t_fp.motion_id = motion as FtMotionId;
    Geno_OnActionChange(&raw mut t_gobj);
}

pub unsafe extern "C" fn test_geno_combo_parse() -> i32 {
    melee_core::ct!("test_geno_combo_parse");
    let mut rc: i32 = 0 as i32;
    if Geno_TestInstall(
        b"{\"geno\":10,\"fighters\":[{\"define\":{\"key\":\"combo-v10\",\"name\":\"Combo T\",\"base\":\"mario\",\"common\":\"melee.common.v1\",\"resources\":\"retail:mario\"},\"lua\":{\"source\":\"local M = {} function M.f(ctx) end return M\",\"state\":{\"armed\":\"int\"}},\"states\":[{\"name\":\"Light2\",\"behavior\":\"geno.ground\"},{\"name\":\"Heavy\",\"behavior\":\"geno.ground\"}],\"combo\":{\"buffer\":3,\"max_links\":3,\"ki\":{\"max\":100,\"start\":20,\"on_hit\":5,\"on_block\":1,\"on_taken\":2},\"routes\":[{\"name\":\"L1>L2\",\"from\":\"jab1\",\"to\":\"geno:Light2\",\"input\":\"light\",\"on\":\"hit\",\"window\":[5,20]},{\"name\":\"L1>H\",\"from\":\"jab1\",\"to\":\"geno:Heavy\",\"input\":\"heavy\",\"dir\":\"forward\",\"on\":\"hit\",\"window\":[5,20],\"priority\":5},{\"name\":\"L2>H\",\"from\":\"geno:Light2\",\"to\":\"geno:Heavy\",\"input\":\"heavy\",\"on\":\"hit\",\"ki_cost\":10,\"when\":{\"slot\":\"armed\",\"min\":1}},{\"name\":\"H>L2\",\"from\":[\"geno:Heavy\"],\"to\":\"geno:Light2\",\"input\":\"light\",\"on\":\"contact\"}],\"strings\":[{\"name\":\"L1 L2 H\",\"steps\":[\"jab1\",\"geno:Light2\",\"geno:Heavy\"]}]}}]}\0"
            as *const u8 as *const i8,
    ) != 0 as i32
    {
        TestFail(
            b"combo: a \"combo\" block in a geno 10 file must refuse the entry\0"
                as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    if Geno_TestInstall(
        b"{\"geno\":11,\"fighters\":[{\"define\":{\"key\":\"combo-bad\",\"name\":\"Combo T\",\"base\":\"mario\",\"common\":\"melee.common.v1\",\"resources\":\"retail:mario\"},\"states\":[{\"name\":\"Light2\",\"behavior\":\"geno.ground\"},{\"name\":\"Heavy\",\"behavior\":\"geno.ground\"}],\"combo\":{\"routes\":[{\"from\":\"jab1\",\"to\":\"geno:Nope\",\"input\":\"light\"}]}}]}\0"
            as *const u8 as *const i8,
    ) != 0 as i32
    {
        TestFail(
            b"combo: a route to an undeclared state must refuse the entry\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    if Geno_TestInstall(
        b"{\"geno\":11,\"fighters\":[{\"define\":{\"key\":\"combo-bad2\",\"name\":\"Combo T\",\"base\":\"mario\",\"common\":\"melee.common.v1\",\"resources\":\"retail:mario\"},\"states\":[{\"name\":\"Light2\",\"behavior\":\"geno.ground\"},{\"name\":\"Heavy\",\"behavior\":\"geno.ground\"}],\"combo\":{\"routes\":[{\"from\":\"jab1\",\"to\":\"geno:Light2\",\"input\":\"kick\"}]}}]}\0"
            as *const u8 as *const i8,
    ) != 0 as i32
    {
        TestFail(
            b"combo: an unknown input class must refuse the entry\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    if Geno_TestInstall(
        b"{\"geno\":11,\"fighters\":[{\"define\":{\"key\":\"combo-bad3\",\"name\":\"Combo T\",\"base\":\"mario\",\"common\":\"melee.common.v1\",\"resources\":\"retail:mario\"},\"states\":[{\"name\":\"Light2\",\"behavior\":\"geno.ground\"},{\"name\":\"Heavy\",\"behavior\":\"geno.ground\"}],\"combo\":{\"routes\":[{\"from\":\"jab1\",\"to\":\"geno:Light2\",\"input\":\"light\",\"when\":{\"slot\":\"nope\"}}]}}]}\0"
            as *const u8 as *const i8,
    ) != 0 as i32
    {
        TestFail(
            b"combo: a Lua gate naming no lua.state slot must refuse the entry\0"
                as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    if Geno_TestInstall(
        b"{\"geno\":11,\"fighters\":[{\"attach\":\"mario\",\"combo\":{\"routes\":[]}}]}\0"
            as *const u8 as *const i8,
    ) != 0 as i32
    {
        TestFail(
            b"combo: an attach entry must not carry a combo graph\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    if tc_setup(
        b"{\"geno\":11,\"fighters\":[{\"define\":{\"key\":\"combo-ok\",\"name\":\"Combo T\",\"base\":\"mario\",\"common\":\"melee.common.v1\",\"resources\":\"retail:mario\"},\"lua\":{\"source\":\"local M = {} function M.f(ctx) end return M\",\"state\":{\"armed\":\"int\"}},\"states\":[{\"name\":\"Light2\",\"behavior\":\"geno.ground\"},{\"name\":\"Heavy\",\"behavior\":\"geno.ground\"}],\"combo\":{\"buffer\":3,\"max_links\":3,\"ki\":{\"max\":100,\"start\":20,\"on_hit\":5,\"on_block\":1,\"on_taken\":2},\"routes\":[{\"name\":\"L1>L2\",\"from\":\"jab1\",\"to\":\"geno:Light2\",\"input\":\"light\",\"on\":\"hit\",\"window\":[5,20]},{\"name\":\"L1>H\",\"from\":\"jab1\",\"to\":\"geno:Heavy\",\"input\":\"heavy\",\"dir\":\"forward\",\"on\":\"hit\",\"window\":[5,20],\"priority\":5},{\"name\":\"L2>H\",\"from\":\"geno:Light2\",\"to\":\"geno:Heavy\",\"input\":\"heavy\",\"on\":\"hit\",\"ki_cost\":10,\"when\":{\"slot\":\"armed\",\"min\":1}},{\"name\":\"H>L2\",\"from\":[\"geno:Heavy\"],\"to\":\"geno:Light2\",\"input\":\"light\",\"on\":\"contact\"}],\"strings\":[{\"name\":\"L1 L2 H\",\"steps\":[\"jab1\",\"geno:Light2\",\"geno:Heavy\"]}]}}]}\0"
            as *const u8 as *const i8,
    ) == 0
    {
        TestFail(
            b"combo: the test define was not installed\0" as *const u8
                as *const i8,
        );
        Geno_TestRestore();
        return 1 as i32;
    }
    if Geno_ComboParam(
        (*tc_st).profile as i32,
        GENO_CP_ROUTES as i32,
    ) != 4 as i32
        || Geno_ComboParam(
            (*tc_st).profile as i32,
            GENO_CP_MAX_LINKS as i32,
        ) != 3 as i32
        || Geno_ComboParam(
            (*tc_st).profile as i32,
            GENO_CP_KI_START as i32,
        ) != 2000 as i32
    {
        TestFail(
            b"combo: 4 routes, max_links 3, ki start 20.00\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    if Geno_ComboWord(
        (*tc_st).profile as i32,
        0 as i32,
        GENO_CR_PRIORITY as i32,
    ) != 5 as i32
        || Geno_ComboWord(
            (*tc_st).profile as i32,
            1 as i32,
            GENO_CR_TO as i32,
        )
            != ((2 as u32 & 15 as u32)
                << 28 as i32
                | 0 as i32 as u32
                    & 0xffff as u32) as i32
        || Geno_ComboWord(
            (*tc_st).profile as i32,
            0 as i32,
            GENO_CR_FROM as i32,
        )
            != ((0 as u32 & 15 as u32)
                << 28 as i32
                | 44 as i32 as u32
                    & 0xffff as u32) as i32
        || Geno_ComboWord(
            (*tc_st).profile as i32,
            1 as i32,
            GENO_CR_BUFFER as i32,
        ) != 3 as i32
        || Geno_ComboWord(
            (*tc_st).profile as i32,
            2 as i32,
            GENO_CR_KI_COST as i32,
        ) != 1000 as i32
    {
        TestFail(
            b"combo: parsed words (sort by priority, from jab1 = motion 44, default buffer, ki cost in hundredths)\0"
                as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    if (*(GenoGame_ComboBlockOf(&raw mut t_fp) as *mut GenoComboBlock)).ki
        != 2000 as i32
    {
        TestFail(
            b"combo: the meter starts at ki.start at the reset\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    Geno_TestRestore();
    if tc_setup(
        b"{\"geno\":11,\"fighters\":[{\"define\":{\"key\":\"combo-hop\",\"name\":\"Combo T\",\"base\":\"mario\",\"common\":\"melee.common.v1\",\"resources\":\"retail:mario\"},\"states\":[{\"name\":\"Light2\",\"behavior\":\"geno.ground\"},{\"name\":\"Heavy\",\"behavior\":\"geno.ground\"}],\"combo\":{\"routes\":[{\"from\":\"jab1\",\"to\":\"jump_short\",\"input\":\"jump\"},{\"from\":\"jab2\",\"to\":\"jump\",\"input\":\"jump\"}]}}]}\0"
            as *const u8 as *const i8,
    ) == 0
    {
        TestFail(
            b"combo: a jump_short / jump route define was not installed\0" as *const u8
                as *const i8,
        );
        Geno_TestRestore();
        t_setup();
        return 1 as i32;
    }
    if Geno_ComboWord(
        (*tc_st).profile as i32,
        0 as i32,
        GENO_CR_TO as i32,
    )
        != ((12 as u32 & 15 as u32)
            << 28 as i32
            | GENO_CA_JUMP_SHORT as i32 as u32
                & 0xffff as u32) as i32
        || Geno_ComboWord(
            (*tc_st).profile as i32,
            1 as i32,
            GENO_CR_TO as i32,
        )
            != ((12 as u32 & 15 as u32)
                << 28 as i32
                | GENO_CA_JUMP as i32 as u32
                    & 0xffff as u32) as i32
    {
        TestFail(
            b"combo: \"jump_short\" must parse to GENO_CA_JUMP_SHORT and \"jump\" to GENO_CA_JUMP\0"
                as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    if Geno_TestInstall(
        b"{\"geno\":11,\"fighters\":[{\"define\":{\"key\":\"meter-bad1\",\"name\":\"Combo T\",\"base\":\"mario\",\"common\":\"melee.common.v1\",\"resources\":\"retail:mario\"},\"states\":[{\"name\":\"Light2\",\"behavior\":\"geno.ground\"},{\"name\":\"Heavy\",\"behavior\":\"geno.ground\"}],\"meter\":{\"name\":\"KI\"}}]}\0"
            as *const u8 as *const i8,
    ) != 0 as i32
    {
        TestFail(
            b"combo: a meter without a combo block must refuse the entry\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    if Geno_TestInstall(
        b"{\"geno\":11,\"fighters\":[{\"define\":{\"key\":\"meter-bad2\",\"name\":\"Combo T\",\"base\":\"mario\",\"common\":\"melee.common.v1\",\"resources\":\"retail:mario\"},\"states\":[{\"name\":\"Light2\",\"behavior\":\"geno.ground\"},{\"name\":\"Heavy\",\"behavior\":\"geno.ground\"}],\"combo\":{\"ki\":{\"max\":100},\"routes\":[]},\"meter\":{\"color\":\"red\"}}]}\0"
            as *const u8 as *const i8,
    ) != 0 as i32
    {
        TestFail(
            b"combo: a meter colour that is not #RRGGBB must refuse the entry\0"
                as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    if tc_setup(
        b"{\"geno\":11,\"fighters\":[{\"define\":{\"key\":\"meter-ok\",\"name\":\"Combo T\",\"base\":\"mario\",\"common\":\"melee.common.v1\",\"resources\":\"retail:mario\"},\"states\":[{\"name\":\"Light2\",\"behavior\":\"geno.ground\"},{\"name\":\"Heavy\",\"behavior\":\"geno.ground\"}],\"combo\":{\"ki\":{\"max\":80},\"routes\":[]},\"meter\":{\"name\":\"CHI\",\"color\":\"#10A0FF\",\"position\":[100,420],\"size\":[90,6]}}]}\0"
            as *const u8 as *const i8,
    ) == 0
    {
        TestFail(
            b"combo: a define with a meter was not installed\0" as *const u8
                as *const i8,
        );
        Geno_TestRestore();
        t_setup();
        return 1 as i32;
    }
    let mut pr: i32 = (*tc_st).profile as i32;
    if Geno_MeterParam(pr, 0 as i32) == 0
        || Geno_MeterParam(pr, 7 as i32) != 'C' as i32
        || Geno_MeterParam(pr, 8 as i32) != 'H' as i32
        || Geno_MeterParam(pr, 9 as i32) != 'I' as i32
        || Geno_MeterParam(pr, 10 as i32) != 0 as i32
        || Geno_MeterParam(pr, 1 as i32) as u32
            != 0x10a0ffff as u32
        || Geno_MeterParam(pr, 2 as i32) != 10000 as i32
        || Geno_MeterParam(pr, 3 as i32) != 42000 as i32
        || Geno_MeterParam(pr, 4 as i32) != 9000 as i32
        || Geno_MeterParam(pr, 5 as i32) != 600 as i32
        || Geno_MeterParam(pr, 6 as i32) != 8000 as i32
    {
        TestFail(
            b"combo: the meter's name CHI, colour #10A0FF, position 100,420, size 90x6 and max (combo.ki.max 80 = 8000 hundredths)\0"
                as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    Geno_TestRestore();
    t_setup();
    return rc;
}

pub unsafe extern "C" fn test_geno_combo_routes() -> i32 {
    melee_core::ct!("test_geno_combo_routes");
    let mut cb: *mut GenoComboBlock = ::core::ptr::null_mut::<GenoComboBlock>();
    let mut lb: *mut GenoLuaBlock = ::core::ptr::null_mut::<GenoLuaBlock>();
    let mut rc: i32 = 0 as i32;
    let mut i: i32 = 0;
    if tc_setup(
        b"{\"geno\":11,\"fighters\":[{\"define\":{\"key\":\"combo-routes\",\"name\":\"Combo T\",\"base\":\"mario\",\"common\":\"melee.common.v1\",\"resources\":\"retail:mario\"},\"lua\":{\"source\":\"local M = {} function M.f(ctx) end return M\",\"state\":{\"armed\":\"int\"}},\"states\":[{\"name\":\"Light2\",\"behavior\":\"geno.ground\"},{\"name\":\"Heavy\",\"behavior\":\"geno.ground\"}],\"combo\":{\"buffer\":3,\"max_links\":3,\"ki\":{\"max\":100,\"start\":20,\"on_hit\":5,\"on_block\":1,\"on_taken\":2},\"routes\":[{\"name\":\"L1>L2\",\"from\":\"jab1\",\"to\":\"geno:Light2\",\"input\":\"light\",\"on\":\"hit\",\"window\":[5,20]},{\"name\":\"L1>H\",\"from\":\"jab1\",\"to\":\"geno:Heavy\",\"input\":\"heavy\",\"dir\":\"forward\",\"on\":\"hit\",\"window\":[5,20],\"priority\":5},{\"name\":\"L2>H\",\"from\":\"geno:Light2\",\"to\":\"geno:Heavy\",\"input\":\"heavy\",\"on\":\"hit\",\"ki_cost\":10,\"when\":{\"slot\":\"armed\",\"min\":1}},{\"name\":\"H>L2\",\"from\":[\"geno:Heavy\"],\"to\":\"geno:Light2\",\"input\":\"light\",\"on\":\"contact\"}],\"strings\":[{\"name\":\"L1 L2 H\",\"steps\":[\"jab1\",\"geno:Light2\",\"geno:Heavy\"]}]}}]}\0"
            as *const u8 as *const i8,
    ) == 0
    {
        TestFail(
            b"combo: the test define was not installed\0" as *const u8
                as *const i8,
        );
        Geno_TestRestore();
        return 1 as i32;
    }
    GenoGame_TestCapture(1 as i32, 0 as i32);
    cb = GenoGame_ComboBlockOf(&raw mut t_fp) as *mut GenoComboBlock;
    lb = GenoGame_LuaBlockOf(&raw mut t_fp) as *mut GenoLuaBlock;
    tc_frame(
        HSD_PAD_A as HSD_Pad,
        0 as i32 as f32,
        0 as i32 as f32,
        0 as i32,
    );
    tc_enter_motion(44 as i32);
    (*tc_st).action_time = 6 as i32 as i32;
    Geno_ComboContact(&raw mut t_fp, 0 as i32);
    if Geno_ComboTick(&raw mut t_gobj) != 0 as i32
        || GenoGame_TestChanges() != 0 as i32
    {
        TestFail(
            b"combo: the press that began the move must not feed its routes\0"
                as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    if (*cb).ki != 2500 as i32 {
        TestFail(
            b"combo: a hit landed adds ki.on_hit (20 + 5)\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    tc_enter_motion(44 as i32);
    (*tc_st).action_time = 6 as i32 as i32;
    tc_frame(
        HSD_PAD_A as HSD_Pad,
        0 as i32 as f32,
        0 as i32 as f32,
        0 as i32,
    );
    if Geno_ComboTick(&raw mut t_gobj) != 0 as i32 {
        TestFail(
            b"combo: an on-hit route fired on a whiff\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    tc_enter_motion(44 as i32);
    (*tc_st).action_time = 3 as i32 as i32;
    Geno_ComboContact(&raw mut t_fp, 0 as i32);
    tc_frame(
        HSD_PAD_A as HSD_Pad,
        0 as i32 as f32,
        0 as i32 as f32,
        0 as i32,
    );
    if Geno_ComboTick(&raw mut t_gobj) != 0 as i32 {
        TestFail(
            b"combo: a route fired before its window\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    i = 0 as i32;
    while i < 10 as i32 {
        tc_frame(
            0 as HSD_Pad,
            0 as i32 as f32,
            0 as i32 as f32,
            1 as i32,
        );
        i += 1;
    }
    if (*cb).age[GENO_CI_LIGHT as i32 as usize] != 0 as i32
    {
        TestFail(
            b"combo: hitlag frames must not age a buffered press\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    tc_frame(
        0 as HSD_Pad,
        0 as i32 as f32,
        0 as i32 as f32,
        0 as i32,
    );
    tc_frame(
        0 as HSD_Pad,
        0 as i32 as f32,
        0 as i32 as f32,
        0 as i32,
    );
    (*tc_st).action_time = 5 as i32 as i32;
    if Geno_ComboTick(&raw mut t_gobj) != 1 as i32
        || GenoGame_TestLastTarget()
            != (2 as u32 & 15 as u32)
                << 28 as i32
                | 0 as i32 as u32
                    & 0xffff as u32
        || (*cb).links != 1 as i32
        || (*cb).age[GENO_CI_LIGHT as i32 as usize]
            != 255 as i32
        || t_fp.motion_id != GENO_MOTION_BASE + 0 as i32
    {
        TestFail(
            b"combo: a buffered light press (age 2 <= buffer 3) must take L1>L2 when the window opens, consume the press, link 1\0"
                as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    tc_enter_motion(44 as i32);
    (*cb).links = 0 as i32 as i32;
    Geno_ComboContact(&raw mut t_fp, 0 as i32);
    tc_frame(
        HSD_PAD_A as HSD_Pad,
        0 as i32 as f32,
        0 as i32 as f32,
        0 as i32,
    );
    i = 0 as i32;
    while i < 4 as i32 {
        tc_frame(
            0 as HSD_Pad,
            0 as i32 as f32,
            0 as i32 as f32,
            0 as i32,
        );
        i += 1;
    }
    (*tc_st).action_time = 8 as i32 as i32;
    if Geno_ComboTick(&raw mut t_gobj) != 0 as i32 {
        TestFail(
            b"combo: a press older than the buffer (4 > 3) fired\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    tc_enter_motion(44 as i32);
    Geno_ComboContact(&raw mut t_fp, 0 as i32);
    (*tc_st).action_time = 8 as i32 as i32;
    tc_frame(
        HSD_PAD_A as HSD_Pad,
        1.0f32,
        0 as i32 as f32,
        0 as i32,
    );
    if (*cb).age[GENO_CI_HEAVY as i32 as usize] != 0 as i32
        || (*cb).dir[GENO_CI_HEAVY as i32 as usize]
            != GENO_CD_FORWARD as i32
        || (*cb).age[GENO_CI_LIGHT as i32 as usize]
            != 255 as i32
    {
        TestFail(
            b"combo: A with a fresh forward flick is a heavy forward press, not a light one\0"
                as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    if Geno_ComboTick(&raw mut t_gobj) != 1 as i32
        || GenoGame_TestLastTarget()
            != (2 as u32 & 15 as u32)
                << 28 as i32
                | 1 as i32 as u32
                    & 0xffff as u32
    {
        TestFail(
            b"combo: the heavy forward press must take L1>H\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    tc_enter_motion(44 as i32);
    Geno_ComboContact(&raw mut t_fp, 0 as i32);
    (*tc_st).action_time = 8 as i32 as i32;
    tc_frame(
        HSD_PAD_A as HSD_Pad,
        -1.0f32,
        0 as i32 as f32,
        0 as i32,
    );
    if Geno_ComboTick(&raw mut t_gobj) != 0 as i32 {
        TestFail(
            b"combo: a heavy back press must not take a forward route\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    tc_enter_motion(GENO_MOTION_BASE + 0 as i32);
    (*cb).links = 1 as i32 as i32;
    (*cb).motion = (GENO_MOTION_BASE + 0 as i32) as i32;
    Geno_ComboContact(&raw mut t_fp, 0 as i32);
    (*tc_st).action_time = 2 as i32 as i32;
    (*lb).slot[0 as i32 as usize] = 0 as i32 as i32;
    tc_frame(
        HSD_PAD_A as HSD_Pad,
        1.0f32,
        0 as i32 as f32,
        0 as i32,
    );
    if Geno_ComboTick(&raw mut t_gobj) != 0 as i32 {
        TestFail(
            b"combo: the Lua gate (armed 0 < 1) must hold the route\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    (*lb).slot[0 as i32 as usize] = 1 as i32 as i32;
    (*cb).ki = 900 as i32 as i32;
    if Geno_ComboTick(&raw mut t_gobj) != 0 as i32 {
        TestFail(
            b"combo: 9 ki < cost 10 must hold the route\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    (*cb).ki = 3000 as i32 as i32;
    if Geno_ComboTick(&raw mut t_gobj) != 1 as i32
        || (*cb).ki != 2000 as i32
        || (*cb).links != 2 as i32
    {
        TestFail(
            b"combo: with the gate and 30 ki the route fires, pays 10, link 2\0"
                as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    Geno_ComboContact(&raw mut t_fp, 1 as i32);
    if (*cb).blocked != 1 as i32 || (*cb).hit != 0 as i32 {
        TestFail(
            b"combo: a shield hit marks blocked only\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    (*tc_st).action_time = 2 as i32 as i32;
    tc_frame(
        HSD_PAD_A as HSD_Pad,
        0 as i32 as f32,
        0 as i32 as f32,
        0 as i32,
    );
    if Geno_ComboTick(&raw mut t_gobj) != 1 as i32
        || (*cb).links != 3 as i32
    {
        TestFail(
            b"combo: H>L2 on contact (block) must fire, link 3\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    Geno_ComboContact(&raw mut t_fp, 0 as i32);
    (*lb).slot[0 as i32 as usize] = 1 as i32 as i32;
    tc_frame(
        HSD_PAD_A as HSD_Pad,
        1.0f32,
        0 as i32 as f32,
        0 as i32,
    );
    if Geno_ComboTick(&raw mut t_gobj) != 0 as i32 {
        TestFail(
            b"combo: max_links 3 must stop a fourth link\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    tc_enter_motion(14 as i32);
    if (*cb).links != 0 as i32 {
        TestFail(
            b"combo: leaving the last link's motion must end the string\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    GenoGame_TestCapture(0 as i32, 0 as i32);
    Geno_TestRestore();
    t_setup();
    return rc;
}

pub unsafe extern "C" fn test_geno_combo_digest() -> i32 {
    melee_core::ct!("test_geno_combo_digest");
    static mut snap_cb: GenoComboBlock = unsafe { core::mem::zeroed() };
    static mut snap_st: TestGenoState = unsafe { core::mem::zeroed() };
    let mut cb: *mut GenoComboBlock = ::core::ptr::null_mut::<GenoComboBlock>();
    let mut d0: u32 = 0;
    let mut d1: u32 = 0;
    let mut targets: [[u32; 6]; 2] = [[0; 6]; 2];
    let mut rc: i32 = 0 as i32;
    let mut i: i32 = 0;
    let mut pass: i32 = 0;
    if tc_setup(
        b"{\"geno\":11,\"fighters\":[{\"define\":{\"key\":\"combo-digest\",\"name\":\"Combo T\",\"base\":\"mario\",\"common\":\"melee.common.v1\",\"resources\":\"retail:mario\"},\"lua\":{\"source\":\"local M = {} function M.f(ctx) end return M\",\"state\":{\"armed\":\"int\"}},\"states\":[{\"name\":\"Light2\",\"behavior\":\"geno.ground\"},{\"name\":\"Heavy\",\"behavior\":\"geno.ground\"}],\"combo\":{\"buffer\":3,\"max_links\":3,\"ki\":{\"max\":100,\"start\":20,\"on_hit\":5,\"on_block\":1,\"on_taken\":2},\"routes\":[{\"name\":\"L1>L2\",\"from\":\"jab1\",\"to\":\"geno:Light2\",\"input\":\"light\",\"on\":\"hit\",\"window\":[5,20]},{\"name\":\"L1>H\",\"from\":\"jab1\",\"to\":\"geno:Heavy\",\"input\":\"heavy\",\"dir\":\"forward\",\"on\":\"hit\",\"window\":[5,20],\"priority\":5},{\"name\":\"L2>H\",\"from\":\"geno:Light2\",\"to\":\"geno:Heavy\",\"input\":\"heavy\",\"on\":\"hit\",\"ki_cost\":10,\"when\":{\"slot\":\"armed\",\"min\":1}},{\"name\":\"H>L2\",\"from\":[\"geno:Heavy\"],\"to\":\"geno:Light2\",\"input\":\"light\",\"on\":\"contact\"}],\"strings\":[{\"name\":\"L1 L2 H\",\"steps\":[\"jab1\",\"geno:Light2\",\"geno:Heavy\"]}]}}]}\0"
            as *const u8 as *const i8,
    ) == 0
    {
        TestFail(
            b"combo: the test define was not installed\0" as *const u8
                as *const i8,
        );
        Geno_TestRestore();
        return 1 as i32;
    }
    cb = GenoGame_ComboBlockOf(&raw mut t_fp) as *mut GenoComboBlock;
    d0 = GenoDefine_StateDigest(&raw mut t_fp);
    i = 0 as i32;
    while i
        < (::core::mem::size_of::<GenoComboBlock>() as usize).wrapping_div(4 as usize)
            as i32
    {
        let mut w: *mut u32 = (cb as *mut u32).offset(i as isize);
        let mut keep: u32 = *w;
        *w ^= 0x10 as u32;
        if GenoDefine_StateDigest(&raw mut t_fp) == d0 {
            TestFail(
                b"combo: a word of GenoComboBlock is not in the digest\0" as *const u8
                    as *const i8,
            );
            rc = 1 as i32;
        }
        *w = keep;
        i += 1;
    }
    if GenoDefine_StateDigest(&raw mut t_fp) != d0 {
        TestFail(
            b"combo: the digest must come back with the block\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    GenoGame_TestCapture(1 as i32, 0 as i32);
    tc_enter_motion(44 as i32);
    memcpy(
        &raw mut snap_cb as *mut c_void,
        cb as *const c_void,
        ::core::mem::size_of::<GenoComboBlock>() as usize,
    );
    memcpy(
        &raw mut snap_st as *mut c_void,
        tc_st as *const c_void,
        ::core::mem::size_of::<TestGenoState>() as usize,
    );
    pass = 0 as i32;
    while pass < 2 as i32 {
        memcpy(
            cb as *mut c_void,
            &raw mut snap_cb as *const c_void,
            ::core::mem::size_of::<GenoComboBlock>() as usize,
        );
        memcpy(
            tc_st as *mut c_void,
            &raw mut snap_st as *const c_void,
            ::core::mem::size_of::<TestGenoState>() as usize,
        );
        t_fp.motion_id = 44 as i32 as FtMotionId;
        i = 0 as i32;
        while i < 6 as i32 {
            tc_frame(
                (if i == 1 as i32 {
                    HSD_PAD_A
                } else {
                    0 as i32
                }) as HSD_Pad,
                if i == 3 as i32 {
                    1.0f32
                } else {
                    0 as i32 as f32
                },
                0 as i32 as f32,
                (i == 2 as i32) as i32,
            );
            if i == 0 as i32 {
                Geno_ComboContact(&raw mut t_fp, 0 as i32);
            }
            (*tc_st).action_time = (5 as i32 + i) as i32;
            targets[pass as usize][i as usize] = if Geno_ComboTick(&raw mut t_gobj) != 0
            {
                GenoGame_TestLastTarget()
            } else {
                0 as u32
            };
            i += 1;
        }
        if pass == 0 as i32 {
            d1 = GenoDefine_StateDigest(&raw mut t_fp);
        } else if GenoDefine_StateDigest(&raw mut t_fp) != d1
            || memcmp(
                &raw mut *(&raw mut targets as *mut [u32; 6])
                    .offset(0 as i32 as isize) as *mut u32
                    as *const c_void,
                &raw mut *(&raw mut targets as *mut [u32; 6])
                    .offset(1 as i32 as isize) as *mut u32
                    as *const c_void,
                ::core::mem::size_of::<[u32; 6]>() as usize,
            ) != 0 as i32
        {
            TestFail(
                b"combo: a resimulation from the snapshot must give the same block and the same links\0"
                    as *const u8 as *const i8,
            );
            rc = 1 as i32;
        }
        pass += 1;
    }
    if targets[0 as i32 as usize][1 as i32 as usize]
        != (2 as u32 & 15 as u32)
            << 28 as i32
            | 0 as i32 as u32
                & 0xffff as u32
    {
        TestFail(
            b"combo: the resimulated sequence must take L1>L2 on the press frame\0"
                as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    GenoGame_TestCapture(0 as i32, 0 as i32);
    Geno_TestRestore();
    if tc_setup(
        b"{\"geno\":11,\"fighters\":[{\"define\":{\"key\":\"combo-none\",\"name\":\"Combo T\",\"base\":\"mario\",\"common\":\"melee.common.v1\",\"resources\":\"retail:mario\"},\"states\":[{\"name\":\"Light2\",\"behavior\":\"geno.ground\"},{\"name\":\"Heavy\",\"behavior\":\"geno.ground\"}]}]}\0"
            as *const u8 as *const i8,
    ) == 0
    {
        TestFail(
            b"combo: the plain define was not installed\0" as *const u8
                as *const i8,
        );
        Geno_TestRestore();
        return 1 as i32;
    }
    cb = GenoGame_ComboBlockOf(&raw mut t_fp) as *mut GenoComboBlock;
    d0 = GenoDefine_StateDigest(&raw mut t_fp);
    (*cb).ki = 1234 as i32 as i32;
    (*cb).links = 3 as i32 as i32;
    if GenoDefine_StateDigest(&raw mut t_fp) != d0 {
        TestFail(
            b"combo: a define without a combo block must keep its digest\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    Geno_TestRestore();
    t_setup();
    return rc;
}

pub unsafe extern "C" fn test_geno_none_guard_contact() -> i32 {
    melee_core::ct!("test_geno_none_guard_contact");
    static mut rest: [HSD_Joint; 4] = unsafe { core::mem::zeroed() };
    static mut guard: [HSD_Joint; 4] = unsafe { core::mem::zeroed() };
    static mut guard_trees: [*mut HSD_Joint; 3] = [::core::ptr::null::<HSD_Joint>()
        as *mut HSD_Joint; 3];
    static mut guard_data: ftData_x20 = ftData_x20 {
        x0: ::core::ptr::null::<*mut HSD_Joint>() as *mut *mut HSD_Joint,
        x8: 0.,
    };
    static mut bones: [FighterBone; 4] = unsafe { core::mem::zeroed() };
    static mut x8: ftData_x8 = ftData_x8 {
        x0: FtPartsDesc {
            model_num: 0,
            vis_table: ::core::ptr::null::<[*mut c_void; 4]>()
                as *mut [*mut c_void; 4],
        },
        x8: ftData_x8_x8 {
            x8: 0,
            xC: ::core::ptr::null::<*mut u16>() as *mut *mut u16,
        },
        x10: 0,
        x11: 0,
        x12: 0,
        x13: 0,
        x14: 0,
    };
    static mut common: ftCommonData = ftCommonData {
        horizontal_stick_deadzone: 0.,
        vertical_stick_deadzone: 0.,
        horizontal_stick_smash_deadzone: 0.,
        vertical_stick_smash_deadzone: 0.,
        analog_shoulder_deadzone: 0.,
        z_press_analog_value: 0.,
        shield_press_threshold: 0.,
        x1C: 0,
        x20_radians: 0.,
        walk_stick_threshold: 0.,
        walk_middle_animation_stick_threshold: 0.,
        walk_fast_stick_threshold: 0.,
        walk_accel_taper_gain: 0.,
        x34: 0.,
        x38_someLStickXThreshold: 0.,
        dash_smash_stick_threshold: 0.,
        dash_smash_window: 0,
        x44: 0.,
        x48: 0.,
        x4C: 0.,
        x50: 0.,
        x54: 0.,
        x58_someLStickXThreshold: 0.,
        run_accel_taper_gain: 0.,
        run_dash_turn_friction_multiplier: 0.,
        x64: 0.,
        x68: 0.,
        friction_when_above_walk_speed: 0.,
        tap_jump_threshold: 0.,
        tap_jump_window: 0,
        x78: 0.,
        tap_jump_release_threshold: 0.,
        relaxed_tap_jump_threshold: 0.,
        x84: 0.,
        x88: 0.,
        x8C: 0,
        x90: 0.,
        x94: 0.,
        x98: 0.,
        x9C_radians: 0.,
        xA0_radians: 0.,
        xA4_radians: 0.,
        xA8_radians: 0.,
        attackhi3_stick_threshold_y: 0.,
        xB0: 0.,
        xB4: 0.,
        xB8_radians: 0.,
        xBC_radians: 0.,
        xC0_radians: 0.,
        xC4_radians: 0.,
        xC8: 0.,
        xCC: 0.,
        xD0: 0.,
        xD4: 0.,
        xD8: 0.,
        xDC: 0.,
        xE0: 0.,
        xE4: 0,
        xE8: 0.,
        xEC: 0.,
        xF0: 0.,
        xF4: 0.,
        xF8: 0.,
        xFC: 0,
        x100: 0.,
        kb_min: 0.,
        x108: 0.,
        x10C: 0.,
        x110: 0.,
        x114: 0.,
        x118: 0.,
        x11C: 0.,
        x120: 0.,
        kb_squat_mul: 0.,
        x128: 0.,
        x12C: 0.,
        x130: 0,
        x134: 0,
        x138: 0.,
        x13C: 0.,
        x140: 0.,
        x144_radians: 0.,
        x148: 0.,
        x14C: 0.,
        x150: 0.,
        x154: 0.,
        x158: 0.,
        x15C: 0.,
        x160: 0.,
        max_grounded_kb_on_landing: 0.,
        x168: 0.,
        x16C: 0.,
        x170: 0.,
        x174: 0.,
        x178: 0.,
        x17C: 0.,
        x180: 0.,
        x184: 0.,
        x188: 0.,
        x18C: 0,
        x190: 0.,
        x194_unkHitLagFrames: 0.,
        x198: 0.,
        x19C: 0.,
        x1A0: 0.,
        x1A4: 0.,
        x1A8: 0.,
        x1AC: 0.,
        x1B0: 0.,
        x1B4: 0.,
        x1B8: 0,
        x1BC: 0.,
        x1C0: 0.,
        x1C4: 0.,
        x1C8: 0.,
        x1CC: 0.,
        x1D0: 0.,
        x1D4: 0.,
        x1D8: 0.,
        x1DC: 0,
        x1E0: 0.,
        x1E4: 0.,
        x1E8_radians: 0.,
        x1EC: 0.,
        x1F0: 0.,
        x1F4: 0.,
        x1F8: 0.,
        aerial_friction_oob: 0.,
        x200: 0.,
        x204_knockbackFrameDecay: 0.,
        x208: 0.,
        x20C: 0.,
        x210: 0.,
        x214: 0,
        x218: 0.,
        x21C: 0.,
        x220: 0.,
        x224: 0,
        x228: 0.,
        x22C: 0.,
        x230: 0.,
        x234_radians: 0.,
        x238_radians: 0.,
        x23C: 0,
        x240: 0.,
        x244: 0.,
        x248: 0.,
        x24C: 0.,
        x250: 0.,
        x254: 0.,
        x258: 0.,
        x25C: 0.,
        x260_startShieldHealth: 0.,
        x264: 0.,
        x268: 0.,
        x26C: 0.,
        x270: 0.,
        x274: 0,
        x278: 0.,
        x27C: 0.,
        x280_unkShieldHealth: 0.,
        x284: 0.,
        x288: 0.,
        x28C: 0.,
        x290: 0.,
        x294: 0.,
        x298: 0.,
        x29C: 0.,
        powershield_input_window: 0,
        x2A4: 0.,
        x2A8: 0.,
        x2AC: 0.,
        x2B0: 0.,
        x2B4: 0.,
        x2B8: 0,
        x2BC: 0.,
        x2C0: 0.,
        x2C4: 0.,
        x2C8: 0.,
        x2CC: 0.,
        x2D0: 0.,
        x2D4: 0.,
        x2D8: 0.,
        x2DC: 0.,
        x2E0: 0.,
        x2E4: 0.,
        x2E8: 0.,
        x2EC: 0.,
        x2F0: 0.,
        x2F4: 0.,
        x2F8: 0.,
        x2FC: 0.,
        x300: 0.,
        x304: 0.,
        x308: 0.,
        x30C: 0.,
        x310: 0.,
        x314: 0.,
        x318: 0,
        x31C: 0.,
        x320: 0,
        x324: 0,
        x328: 0.,
        escapeair_deadzone: Vec2 { x: 0., y: 0. },
        x334: 0,
        escapeair_force: 0.,
        escapeair_decay: 0.,
        x340: 0.,
        x344: 0.,
        x348: 0,
        x34C: 0.,
        x350: 0.,
        x354: 0.,
        x358: 0.,
        x35C: 0.,
        x360: 0.,
        x364: 0.,
        x368: 0.,
        x36C: 0.,
        x370: 0.,
        x374: 0.,
        x378: 0.,
        x37C: 0.,
        x380: lbColl_80008D30_arg1 {
            state: HitCapsule_Disabled,
            damage: 0,
            kb_angle: 0,
            unkC: 0,
            unk10: 0,
            unk14: 0,
            element: 0,
            sfx_severity: 0,
            sfx_kind: 0,
        },
        grab_timer_decrement: 0.,
        x3A8: 0.,
        x3AC: 0.,
        x3B0: 0.,
        shouldered_anim_rate: 0.,
        x3B8: 0.,
        x3BC: 0.,
        x3C0: 0,
        x3C4: 0.,
        x3C8: 0.,
        x3CC: 0,
        x3D0: 0.,
        x3D4: 0.,
        x3D8: 0.,
        x3DC: 0.,
        x3E0: 0.,
        x3E4: 0.,
        x3E8_shieldKnockbackFrameDecay: 0.,
        x3EC_shieldGroundFrictionMultiplier: 0.,
        x3F0: 0.,
        x3F4: 0,
        x3F8: 0,
        x3FC: 0,
        x400: 0.,
        x404: 0.,
        x408: 0.,
        x40C: 0.,
        x410: 0,
        x414: 0,
        x418: 0,
        x41C: 0,
        x420: 0.,
        x424: 0.,
        x428: 0,
        x42C: 0.,
        x430: 0.,
        x434: 0.,
        x438: 0.,
        x43C: 0.,
        x440: 0.,
        x444: 0.,
        x448: 0.,
        x44C: 0.,
        x450: 0.,
        x454: 0.,
        x458: 0.,
        x45C: 0.,
        x460: 0.,
        x464: 0.,
        x468: 0.,
        x46C: 0.,
        x470: 0.,
        teeter_walk_threshold: 0.,
        x478: 0.,
        x47C: 0.,
        x480: 0.,
        x484: 0.,
        x488: 0,
        x48C: 0.,
        x490: 0.,
        x494: 0.,
        ledge_cooldown: 0,
        x49C: 0,
        x4A0: 0.,
        x4A4: 0.,
        x4A8: 0.,
        x4AC: 0.,
        sdi_min_stick_mag: 0.,
        sdi_stick_window: 0,
        sdi_pos_scale: 0.,
        x4BC: 0.,
        x4C0: 0.,
        x4C4: 0,
        x4C8: 0,
        x4CC: 0,
        x4D0: 0.,
        x4D4: 0.,
        x4D8: 0,
        x4DC: Vec2 { x: 0., y: 0. },
        x4E4: Vec { x: 0., y: 0., z: 0. },
        x4F0: 0.,
        x4F4: 0.,
        x4F8: 0,
        x4FC: 0,
        x500: 0,
        x504: 0,
        x508: 0,
        x50C: 0,
        x510: 0.,
        x514: 0.,
        x518: 0,
        x51C_radians: 0.,
        fall_common: FallCommon {
            x520: 0,
            x524: 0,
            x528: 0,
            x52C: 0,
            x530: 0,
            x534: 0,
            x538: Vec { x: 0., y: 0., z: 0. },
            x544: Vec { x: 0., y: 0., z: 0. },
            x550: 0.,
            x554: 0.,
            x558: 0.,
            x55C: 0.,
            x560_radians: 0.,
            x564: 0.,
        },
        x568: 0.,
        x56C: 0.,
        x570: 0.,
        x574: 0.,
        x578: 0.,
        x57C: 0,
        x580: 0,
        x584: 0,
        x588: 0,
        x58C: 0.,
        x590: 0.,
        open_parasol_threshold: 0.,
        close_parasol_threshold: 0.,
        x59C: 0.,
        x5A0: 0.,
        x5A4: 0,
        x5A8: 0.,
        x5AC: 0.,
        x5B0: 0.,
        x5B4: 0,
        x5B8: 0.,
        x5BC: 0,
        x5C0: 0.,
        x5C4: 0,
        x5C8: 0,
        x5CC: 0.,
        rebirth_countdown: 0,
        rebirth_wait: 0,
        x5D8: 0,
        bury_timer_unk1: 0,
        bury_timer_unk2: 0,
        bury_timer_unk3: 0,
        x5E8: 0.,
        x5EC: 0,
        x5F0: 0,
        x5F4: 0,
        x5F8: 0.,
        x5FC: 0.,
        x600: 0.,
        x604: 0.,
        x608: 0.,
        x60C: 0.,
        x610: 0.,
        x614: 0.,
        x618: 0.,
        x61C: 0.,
        x620: 0,
        x624: 0.,
        x628: 0.,
        x62C: 0.,
        x630: 0.,
        x634: 0.,
        x638: 0.,
        x63C: 0.,
        x640: 0.,
        x644: 0.,
        x648: 0,
        x64C: 0.,
        x650: 0.,
        x654: 0.,
        x658: 0.,
        x65C: 0.,
        x660: 0.,
        x664: 0.,
        pressed_inputs: 0.,
        released_inputs: 0.,
        x670: 0.,
        x674: 0.,
        x678: 0.,
        x67C: 0.,
        x680: 0.,
        x684: 0.,
        x688: 0,
        x68C: 0,
        x690: 0,
        x694: 0.,
        x698: 0.,
        warpstarfall_drift_scaling: 0.,
        warpstarfall_drift_flat: 0.,
        warpstarfall_drift_max: 0.,
        x6A8: 0.,
        x6AC: 0,
        x6B0: 0,
        x6B4: 0,
        x6B8: 0,
        x6BC: 0,
        x6C0: 0,
        x6C4: 0.,
        x6C8: 0,
        x6CC: 0,
        x6D0: 0.,
        x6D4: 0,
        x6D8: [0; 1],
        x6DC_colorsByPlayer: [_GXColor { r: 0, g: 0, b: 0, a: 0 }; 4],
        x6EC: [0; 4],
        metal_armor: 0.,
        x6F4_unkDamage: 0,
        x6F8: 0,
        x6FC: 0,
        x700: 0,
        x704: 0.,
        x708: 0.,
        x70C: 0.,
        x710: 0.,
        x714: 0.,
        kb_ice_mul: 0.,
        x71C: 0.,
        x720: 0.,
        x724: 0.,
        x728: 0.,
        x72C: 0.,
        x730: 0.,
        leadead_grab_timer_step: 0.,
        leadead_grab_break_threshold: 0.,
        x73C: 0,
        x740: 0.,
        x744: 0.,
        x748: 0.,
        x74C: 0.,
        x750: 0.,
        x754: 0.,
        x758: 0.,
        x75C: 0.,
        x760: 0,
        x764: 0,
        x768: 0.,
        x76C: 0.,
        x770: 0.,
        x774: 0,
        passive_wall_vel_y_base: 0.,
        damageice_gravity_mult: 0.,
        damageice_min_speed: 0.,
        damageice_speed_mult_on_break: 0.,
        damageice_rot_speed_min: 0.,
        damageice_rot_speed_max: 0.,
        x790_damageice_unk: 0.,
        x794_damageice_unk: 0.,
        x798_damageice_unk: 0.,
        damageice_dmg_time_reduction_mult: 0.,
        damageice_ice_size: 0.,
        damageicejump_escape_time: 0.,
        x7A8: 0.,
        x7AC: 0,
        x7B0: 0,
        x7B4_unkDamage: 0,
        x7B8: 0.,
        x7BC: 0.,
        x7C0: 0.,
        kb_smashcharge_mul: 0.,
        x7C8: 0.,
        x7CC: 0,
        x7D0: 0,
        hit_weight_mul: 0.,
        x7D8: _GXColor { r: 0, g: 0, b: 0, a: 0 },
        x7DC: 0,
        x7E0: 0,
        x7E4_scaleZ: 0.,
        unk_kb_angle_min: 0,
        unk_kb_angle_max: 0,
        x7F0: 0,
        x7F4: 0.,
        x7F8: 0.,
        x7FC: 0.,
        x800: 0.,
        x804: 0.,
        x808: Vec { x: 0., y: 0., z: 0. },
        x814: 0,
    };
    static mut parts_table: FighterPartsTable = unsafe { core::mem::zeroed() };
    static mut parts_tables: [*mut FighterPartsTable; 127] = [::core::ptr::null::<
        FighterPartsTable,
    >() as *mut FighterPartsTable; 127];
    static mut anim_parts: [*mut Fighter_804D6540_t; 127] = [::core::ptr::null::<
        Fighter_804D6540_t,
    >() as *mut Fighter_804D6540_t; 127];
    static mut costume: UnkCostumeStruct = unsafe { core::mem::zeroed() };
    static mut rows: [Fighter_WaitAnimData; 40] = unsafe { core::mem::zeroed() };
    static mut tracks: [FigaTrack; 2] = unsafe { core::mem::zeroed() };
    static mut tree: FigaTree = unsafe { core::mem::zeroed() };
    static mut arena: [u8; 1048576] = [0; 1048576];
    static mut nodes: [i8; 5] = [
        0 as i32 as i8,
        0 as i32 as i8,
        1 as i32 as i8,
        1 as i32 as i8,
        -(1 as i32) as i8,
    ];
    static mut rot_y: [u8; 5] = [
        0x12 as i32 as u8,
        6 as i32 as u8,
        1 as i32 as u8,
        6 as i32 as u8,
        1 as i32 as u8,
    ];
    static mut throw_y: [u8; 5] = [
        0x12 as i32 as u8,
        253 as i32 as u8,
        1 as i32 as u8,
        253 as i32 as u8,
        1 as i32 as u8,
    ];
    let mut live: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut blend: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut j: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut b: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut old_parts: *mut *mut FighterPartsTable = ftPartsTable;
    let mut old_anim_parts: *mut *mut Fighter_804D6540_t = Fighter_804D6540;
    let mut old_costumes: UnkCostumeList = unsafe { core::mem::zeroed() };
    let mut old_common: *mut ftCommonData = p_ftCommonData;
    let mut old_x8: *mut ftData_x8 = t_ftdata.x8 as *mut ftData_x8;
    let mut old_x20: *mut ftData_x20 = t_ftdata.x20;
    let mut old_guard_row: i32 = t_rows[ftCo_MS_Guard
            as i32 as usize]
        .anim_id as i32;
    let mut hit: HitCapsule = unsafe { core::mem::zeroed() };
    let mut heap_start: *mut c_void = ::core::ptr::null_mut::<
        c_void,
    >();
    let mut kind: i32 = 0;
    let mut i: i32 = 0;
    let mut rc: i32 = 0 as i32;
    let mut old_heap: i32 = HSD_GetHeap() as i32;
    let mut heap: i32 = 0;
    if Geno_TestInstall(
        b"{\"geno\":10,\"fighters\":[{\"define\":{\"key\":\"guard-test\",\"name\":\"Guard test\",\"base\":\"none\",\"common\":\"melee.common.v1\",\"resources\":\"mod:files\"},\"fighter\":{\"plan\":\"plan.json\",\"animation\":\"bank.dat\",\"costumes\":[{\"file\":\"model.dat\",\"joint\":\"j\"}]}}]}\0"
            as *const u8 as *const i8,
    ) != 1 as i32
    {
        TestFail(
            b"guard fixture registration\0" as *const u8 as *const i8,
        );
        Geno_TestRestore();
        return 1 as i32;
    }
    kind = Geno_DefineCKAt(0 as i32) - 1 as i32;
    t_setup();
    t_fp.kind = kind as FighterKind;
    memset(
        &raw mut bones as *mut FighterBone as *mut c_void,
        0 as i32,
        ::core::mem::size_of::<[FighterBone; 4]>() as usize,
    );
    t_fp.parts = &raw mut bones as *mut FighterBone;
    heap_start = OSInitAlloc(
        &raw mut arena as *mut u8 as *mut c_void,
        (&raw mut arena as *mut u8)
            .offset(::core::mem::size_of::<[u8; 1048576]>() as usize as isize)
            as *mut c_void,
        4 as i32,
    );
    heap_start = ((heap_start as uintptr_t)
        .wrapping_add(31 as i32 as uintptr_t)
        & !(31 as i32 as uintptr_t)) as *mut c_void;
    heap = OSCreateHeap(
        heap_start,
        (&raw mut arena as *mut u8)
            .offset(::core::mem::size_of::<[u8; 1048576]>() as usize as isize)
            as *mut c_void,
    );
    if heap < 0 as i32 {
        TestFail(b"guard fixture heap\0" as *const u8 as *const i8);
        Geno_TestRestore();
        return 1 as i32;
    }
    HSD_SetHeap(heap as OSHeapHandle);
    HSD_AObjInitAllocData();
    HSD_FObjInitAllocData();
    HSD_IDInitAllocData();
    HSD_VecInitAllocData();
    HSD_MtxInitAllocData();
    HSD_RObjInitAllocData();
    HSD_IDSetup();
    memset(
        &raw mut rest as *mut HSD_Joint as *mut c_void,
        0 as i32,
        ::core::mem::size_of::<[HSD_Joint; 4]>() as usize,
    );
    i = 0 as i32;
    while i < 4 as i32 {
        rest[i as usize].scale.z = 1 as i32 as f32;
        rest[i as usize].scale.y = rest[i as usize].scale.z;
        rest[i as usize].scale.x = rest[i as usize].scale.y;
        if i < 3 as i32 {
            rest[i as usize].child = (&raw mut rest as *mut HSD_Joint)
                .offset((i + 1 as i32) as isize) as *mut HSD_Joint
                as *mut HSD_Joint;
        }
        i += 1;
    }
    rest[2 as i32 as usize].position.y = 10 as i32
        as f32;
    rest[3 as i32 as usize].position.y = -(10 as i32)
        as f32;
    live = HSD_JObjLoadJoint(&raw mut rest as *mut HSD_Joint);
    blend = HSD_JObjLoadJoint(&raw mut rest as *mut HSD_Joint);
    j = live;
    b = blend;
    i = 0 as i32;
    while i < 4 as i32 {
        let ref mut fresh217 = (*t_fp.parts.offset(i as isize)).joint;
        *fresh217 = j as *mut HSD_JObj;
        let ref mut fresh218 = (*t_fp.parts.offset(i as isize)).x4_jobj2;
        *fresh218 = b as *mut HSD_JObj;
        let ref mut fresh219 = (*t_fp.parts.offset(i as isize))
            .anon_8.anon_0;
        (*fresh219).set_flags_b1(1 as u8 as u8);
        j = HSD_JObjGetChild(j);
        b = HSD_JObjGetChild(b);
        i += 1;
    }
    ftPartsTable = &raw mut parts_tables as *mut *mut FighterPartsTable;
    Fighter_804D6540 = &raw mut anim_parts as *mut *mut Fighter_804D6540_t;
    old_costumes = CostumeListsForeachCharacter[kind as usize];
    parts_table.parts_num = 4 as u32;
    let ref mut fresh220 = *ftPartsTable.offset(kind as isize);
    *fresh220 = &raw mut parts_table;
    costume.joint = &raw mut rest as *mut HSD_Joint as *mut HSD_Joint;
    CostumeListsForeachCharacter[kind as usize].costume_list = &raw mut costume;
    memset(
        &raw mut common as *mut c_void,
        0 as i32,
        ::core::mem::size_of::<ftCommonData>() as usize,
    );
    common.x260_startShieldHealth = 60 as i32 as f32;
    common.x2D8 = 1 as i32 as f32;
    common.x2D4 = common.x2D8;
    p_ftCommonData = &raw mut common;
    memset(
        &raw mut x8 as *mut c_void,
        0 as i32,
        ::core::mem::size_of::<ftData_x8>() as usize,
    );
    x8.x10 = 255 as u8;
    x8.x11 = 3 as u8;
    t_ftdata.x8 = &raw mut x8 as *mut ftData_x8;
    t_fp.x108_costume_joint = &raw mut rest as *mut HSD_Joint as *mut HSD_Joint;
    t_fp.x8AC_animSkeleton = blend as *mut HSD_JObj;
    (*t_fp.x1C_actionStateList.offset(ftCo_MS_Guard as i32 as isize))
        .anim_id = 39 as i32 as enum_t;
    memset(
        &raw mut rows as *mut Fighter_WaitAnimData as *mut c_void,
        0 as i32,
        ::core::mem::size_of::<[Fighter_WaitAnimData; 40]>() as usize,
    );
    rows[39 as i32 as usize].x14 = 1 as u32;
    t_fp.x24 = &raw mut rows as *mut Fighter_WaitAnimData as *mut Fighter_WaitAnimData;
    t_fp.x58C = 40 as u32;
    t_fp.x5A8 = 1 as i32 as *mut c_void;
    memset(
        &raw mut tracks as *mut FigaTrack as *mut c_void,
        0 as i32,
        ::core::mem::size_of::<[FigaTrack; 2]>() as usize,
    );
    i = 0 as i32;
    while i < 2 as i32 {
        tracks[i as usize].length = 5 as u16;
        tracks[i as usize].obj_type = 6 as u8;
        tracks[i as usize].frac_slope = 0x60 as u8;
        tracks[i as usize].frac_value = tracks[i as usize].frac_slope;
        i += 1;
    }
    tracks[0 as i32 as usize].ad_head = &raw mut rot_y as *mut u8;
    tracks[1 as i32 as usize].ad_head = &raw mut throw_y as *mut u8;
    tree.r#type = 1 as i32;
    tree.frames = 1 as i32 as f32;
    tree.nodes = &raw mut nodes as *mut i8;
    tree.tracks = &raw mut tracks as *mut FigaTrack;
    t_fp.x598 = &raw mut tree;
    t_fp.anon_221b.set_x221B_b0(1 as u8 as u8);
    t_fp.shield_health = 60 as i32 as f32;
    t_fp.co_attrs.initial_shield_size = 2 as i32 as f32;
    ftCo_80091E78(&raw mut t_gobj, 1 as i32 as f32);
    t_fp.shield_hit.bone = (*t_fp.parts.offset(3 as i32 as isize)).joint;
    t_fp.shield_hit.size = 1 as i32 as f32;
    memset(
        &raw mut hit as *mut c_void,
        0 as i32,
        ::core::mem::size_of::<HitCapsule>() as usize,
    );
    hit.x4C.x = 2.5f32 as f32;
    hit.x58.x = hit.x4C.x;
    hit.x4C.y = 3 as i32 as f32;
    hit.x58.y = hit.x4C.y;
    hit.scale = 1 as i32 as f32;
    if lbColl_80007BCC(
        &raw mut hit,
        &raw mut t_fp.shield_hit,
        ::core::ptr::null_mut::<c_void>(),
        0 as i32,
        1 as i32 as f32,
        1 as i32 as f32,
        0 as i32 as f32,
    ) == 0
    {
        TestFail(
            b"authored guard shield must contact the low smash (rest pose misses)\0"
                as *const u8 as *const i8,
        );
        rc = 1 as i32;
    }
    if fabsf(
        t_fp.shield_hit.pos.y as f32
            - 3 as i32 as f32,
    ) > 0.001f32
    {
        TestFail(
            b"guard uses own row 39 and preserves authored shield anchor\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    ftAnim_8006FA58(
        &raw mut t_fp,
        FtPart_TransN,
        rest[0 as i32 as usize].child,
    );
    ftCo_80091E78(&raw mut t_gobj, 0.5f32);
    if fabsf(
        (*bones[2 as i32 as usize].joint).translate.y
            as f32
            + (*bones[3 as i32 as usize].joint).translate.y
                as f32 - 1.5f32,
    ) > 0.001f32
    {
        TestFail(
            b"guard-on half blend retains the authored anchor\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    t_fp.mv.co.guard.x4 = 0.5f32;
    ftCo_80091E78(&raw mut t_gobj, 1 as i32 as f32);
    if fabsf(
        (*bones[2 as i32 as usize].joint).translate.y
            as f32
            + (*bones[3 as i32 as usize].joint).translate.y
                as f32 - 3 as i32 as f32,
    ) > 0.001f32
    {
        TestFail(
            b"tilted guard keeps the authored base pose and own row\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    memcpy(
        &raw mut guard as *mut HSD_Joint as *mut c_void,
        &raw mut rest as *mut HSD_Joint as *const c_void,
        ::core::mem::size_of::<[HSD_Joint; 4]>() as usize,
    );
    i = 0 as i32;
    while i < 3 as i32 {
        guard[i as usize].child = (&raw mut guard as *mut HSD_Joint)
            .offset((i + 1 as i32) as isize) as *mut HSD_Joint
            as *mut HSD_Joint;
        i += 1;
    }
    guard[2 as i32 as usize].position.y = 6 as i32
        as f32;
    guard[3 as i32 as usize].position.y = -(3 as i32)
        as f32;
    guard_trees[2 as i32 as usize] = (&raw mut guard as *mut HSD_Joint)
        .offset(1 as i32 as isize) as *mut HSD_Joint;
    guard_data.x0 = &raw mut guard_trees as *mut *mut HSD_Joint as *mut *mut HSD_Joint;
    t_ftdata.x20 = &raw mut guard_data;
    t_fp.mv.co.guard.x4 = 0 as i32 as f32;
    parts_tables[Ft_Kind_Mario as i32 as usize] = &raw mut parts_table;
    i = 0 as i32;
    while i < 2 as i32 {
        if i == 0 as i32 {
            t_fp.kind = Ft_Kind_Mario;
        } else if Geno_TestInstall(
            b"{\"geno\":10,\"fighters\":[{\"define\":{\"key\":\"donor-guard-test\",\"name\":\"Donor guard test\",\"base\":\"mario\",\"common\":\"melee.common.v1\",\"resources\":\"retail:mario\"}}]}\0"
                as *const u8 as *const i8,
        ) != 1 as i32
        {
            TestFail(
                b"donor guard fixture registration\0" as *const u8
                    as *const i8,
            );
            rc = 1 as i32;
            break;
        } else {
            t_fp.kind = (Geno_DefineCKAt(0 as i32)
                - 1 as i32) as FighterKind;
        }
        ftCo_80091E78(&raw mut t_gobj, 1 as i32 as f32);
        t_fp.shield_hit.set_skip_update_pos(0 as u8 as u8);
        if lbColl_80007BCC(
            &raw mut hit,
            &raw mut t_fp.shield_hit,
            ::core::ptr::null_mut::<c_void>(),
            0 as i32,
            1 as i32 as f32,
            1 as i32 as f32,
            0 as i32 as f32,
        ) == 0
            || fabsf(
                t_fp.shield_hit.pos.y as f32
                    - 3 as i32 as f32,
            ) > 0.001f32
        {
            TestFail(
                b"retail/donor guard keeps the original x20 shield pose\0" as *const u8
                    as *const i8,
            );
            rc = 1 as i32;
        }
        i += 1;
    }
    HSD_JObjRemoveAll(live);
    HSD_JObjRemoveAll(blend);
    HSD_SetHeap(old_heap as OSHeapHandle);
    ftPartsTable = old_parts;
    CostumeListsForeachCharacter[kind as usize] = old_costumes;
    Fighter_804D6540 = old_anim_parts;
    p_ftCommonData = old_common;
    t_ftdata.x8 = old_x8 as *mut ftData_x8;
    t_ftdata.x20 = old_x20;
    t_rows[ftCo_MS_Guard as i32 as usize].anim_id = old_guard_row
        as enum_t;
    Geno_TestRestore();
    t_setup();
    return rc;
}

pub unsafe extern "C" fn gc_fail(
    mut what: *const i8,
    mut field: *const i8,
    mut off: u32,
    mut tail: *const i8,
    mut more: *const i8,
) {
    melee_core::ct!("gc_fail");
    let mut buf: [i8; 400] = [0; 400];
    let mut num: [i8; 12] = [0; 12];
    let mut n: i32 = 0 as i32;
    let mut k: i32 = 0 as i32;
    let mut d: i32 = 0;
    let mut parts: [*const i8; 4] = [::core::ptr::null::<
        i8,
    >(); 4];
    parts[0 as i32 as usize] = what;
    parts[1 as i32 as usize] = b".\0" as *const u8
        as *const i8;
    parts[2 as i32 as usize] = if !field.is_null() {
        field
    } else {
        b"?\0" as *const u8 as *const i8
    };
    parts[3 as i32 as usize] = b" (byte \0" as *const u8
        as *const i8;
    d = 0 as i32;
    while d < 4 as i32 {
        let mut s: *const i8 = parts[d as usize];
        while *s as i32 != 0 as i32
            && n < 360 as i32
        {
            let fresh221 = s;
            s = s.offset(1);
            let fresh222 = n;
            n = n + 1;
            buf[fresh222 as usize] = *fresh221;
        }
        d += 1;
    }
    if off == 0 as u32 {
        let fresh223 = k;
        k = k + 1;
        num[fresh223 as usize] = '0' as i32 as i8;
    }
    while off != 0 as u32 && k < 11 as i32 {
        let fresh224 = k;
        k = k + 1;
        num[fresh224 as usize] = ('0' as i32 as u32)
            .wrapping_add(off.wrapping_rem(10 as u32)) as i8;
        off = off.wrapping_div(10 as u32);
    }
    while k > 0 as i32 {
        k -= 1;
        let fresh225 = n;
        n = n + 1;
        buf[fresh225 as usize] = num[k as usize];
    }
    let fresh226 = n;
    n = n + 1;
    buf[fresh226 as usize] = ')' as i32 as i8;
    let fresh227 = n;
    n = n + 1;
    buf[fresh227 as usize] = ' ' as i32 as i8;
    while *tail as i32 != 0 as i32
        && n < 380 as i32
    {
        let fresh228 = tail;
        tail = tail.offset(1);
        let fresh229 = n;
        n = n + 1;
        buf[fresh229 as usize] = *fresh228;
    }
    while !more.is_null() && *more as i32 != 0 as i32
        && n < 399 as i32
    {
        let fresh230 = more;
        more = more.offset(1);
        let fresh231 = n;
        n = n + 1;
        buf[fresh231 as usize] = *fresh230;
    }
    buf[n as usize] = 0 as i8;
    TestFail(&raw mut buf as *mut i8);
}

pub unsafe extern "C" fn gc_walk(
    mut what: *const i8,
    mut block: *mut u32,
    mut bytes: u32,
    mut f: *const GenoCoverField,
    mut nf: i32,
    mut fp: *mut Fighter,
) -> i32 {
    melee_core::ct!("gc_walk");
    let mut w: u32 = 0;
    let mut i: u32 = 0;
    let mut base: u32 = 0;
    let mut rc: u32 = 0 as u32;
    w = 0 as u32;
    while w < bytes.wrapping_div(4 as u32) {
        let mut off: u32 = w.wrapping_mul(4 as u32);
        let mut orig: u32 = *block.offset(w as isize);
        let mut after: u32 = 0;
        let mut hit: *const GenoCoverField = ::core::ptr::null::<GenoCoverField>();
        i = 0 as u32;
        while i < nf as u32 {
            if off >= (*f.offset(i as isize)).off
                && off
                    < (*f.offset(i as isize))
                        .off
                        .wrapping_add((*f.offset(i as isize)).size)
            {
                hit = f.offset(i as isize) as *const GenoCoverField;
            }
            i = i.wrapping_add(1);
        }
        if hit.is_null() {
            gc_fail(
                what,
                b"?\0" as *const u8 as *const i8,
                off,
                b"belongs to no field of the coverage table: a field was added without deciding whether the rollback hash covers it\0"
                    as *const u8 as *const i8,
                ::core::ptr::null::<i8>(),
            );
            rc = rc.wrapping_add(1);
        } else if !((*hit).covered == 2 as i32) {
            base = GenoDefine_StateDigest(fp);
            *block.offset(w as isize) = (orig as u32
                ^ 0x400401 as u32) as u32;
            after = GenoDefine_StateDigest(fp);
            *block.offset(w as isize) = orig;
            if (*hit).covered != 0 && after == base {
                gc_fail(
                    what,
                    (*hit).name,
                    off,
                    b"decides the simulation but does not move the rollback hash\0"
                        as *const u8 as *const i8,
                    ::core::ptr::null::<i8>(),
                );
                rc = rc.wrapping_add(1);
            } else if (*hit).covered == 0 && after != base {
                gc_fail(
                    what,
                    (*hit).name,
                    off,
                    b"is listed as outside the hash but moves it: \0" as *const u8
                        as *const i8,
                    (*hit).why,
                );
                rc = rc.wrapping_add(1);
            }
        }
        w = w.wrapping_add(1);
    }
    return rc as i32;
}

pub unsafe extern "C" fn test_geno_digest_coverage() -> i32 {
    melee_core::ct!("test_geno_digest_coverage");
    let mut st: *mut TestGenoState = ::core::ptr::null_mut::<TestGenoState>();
    let mut lb: *mut GenoLuaBlock = ::core::ptr::null_mut::<GenoLuaBlock>();
    let mut rc: i32 = 0 as i32;
    let mut i: i32 = 0;
    let mut j: i32 = 0;
    if tl_setup(
        b"{\"geno\":10,\"fighters\":[{\"define\":{\"key\":\"cover-lua\",\"name\":\"Lua T\",\"base\":\"mario\",\"common\":\"melee.common.v1\",\"resources\":\"retail:mario\"},\"lua\":{\"source\":\"local M = {} function M.enter(ctx) ctx.state.charge = 0 end function M.frame(ctx) local s = ctx.state if ctx.input.special_held and s.charge < 60 then s.charge = s.charge + 1 s.armed = s.charge >= 3 else ctx.go('Release') end end function M.rel(ctx) ctx.velocity(1.6 + 0.03 * ctx.state.charge, 0) end function M.relf(ctx) ctx.hitbox_damage(3, 6 + 0.15 * ctx.state.charge) end function M.boom(ctx) ctx.state.charge = 77 error('boom') end return M\",\"state\":{\"charge\":\"int\",\"armed\":\"bool\"}},\"states\":[{\"name\":\"Charge\",\"behavior\":\"geno.ground\",\"lua\":{\"enter\":\"enter\",\"frame\":\"frame\"}},{\"name\":\"Release\",\"behavior\":\"geno.ground\",\"lua\":{\"enter\":\"rel\",\"frame\":\"relf\"}},{\"name\":\"Bad\",\"behavior\":\"geno.ground\",\"lua\":{\"frame\":\"boom\"}}],\"specials\":{\"n\":\"geno:Charge\",\"s\":\"geno:Bad\"}}]}\0"
            as *const u8 as *const i8,
        &raw mut st,
    ) == 0
    {
        Geno_TestRestore();
        return 1 as i32;
    }
    (*st).nchecks = GENO_MAX_CHECKS as u32;
    i = 0 as i32;
    while i < GENO_MAX_CHECKS {
        (*st).checks[i as usize].ncond = GENO_CHECK_CONDS as u32;
        j = 0 as i32;
        while j < GENO_CHECK_CONDS {
            (*st).checks[i as usize].cond[j as usize].head = 1 as u32;
            j += 1;
        }
        i += 1;
    }
    rc
        += gc_walk(
            b"GenoState\0" as *const u8 as *const i8,
            st as *mut u32,
            ::core::mem::size_of::<GenoState>() as u32,
            &raw const gc_state_fields as *const GenoCoverField,
            (::core::mem::size_of::<[GenoCoverField; 51]>() as usize)
                .wrapping_div(::core::mem::size_of::<GenoCoverField>() as usize)
                as i32,
            &raw mut t_fp,
        );
    lb = GenoGame_LuaBlockOf(&raw mut t_fp) as *mut GenoLuaBlock;
    rc
        += gc_walk(
            b"GenoLuaBlock\0" as *const u8 as *const i8,
            lb as *mut u32,
            ::core::mem::size_of::<GenoLuaBlock>() as u32,
            &raw const gc_lua_fields as *const GenoCoverField,
            (::core::mem::size_of::<[GenoCoverField; 3]>() as usize)
                .wrapping_div(::core::mem::size_of::<GenoCoverField>() as usize)
                as i32,
            &raw mut t_fp,
        );
    Geno_TestRestore();
    return (rc != 0 as i32) as i32;
}

pub unsafe extern "C" fn gba_i(mut v: f32) -> i32 {
    melee_core::ct!("gba_i");
    return (if v >= 0.0f32 {
        v as f32 * 1000.0f32 + 0.5f32
    } else {
        v as f32 * 1000.0f32 - 0.5f32
    }) as i32;
}

pub unsafe extern "C" fn gba_abs(mut v: f32) -> f32 {
    melee_core::ct!("gba_abs");
    return if v < 0.0f32 { -v } else { v };
}

pub unsafe extern "C" fn gba(
    mut id: *const i8,
    mut ok: i32,
    mut meas: i32,
) {
    melee_core::ct!("gba");
    OSReport!(
        b"GBACC %s %s %d\n\0" as *const u8 as *const i8
            as *mut i8,
        id,
        if ok != 0 {
            b"PASS\0" as *const u8 as *const i8
        } else {
            b"FAIL\0" as *const u8 as *const i8
        },
        meas,
    );
    if ok == 0 {
        TestFail(id);
        gba_rc = 1 as i32;
    }
}

pub unsafe extern "C" fn gba_near(
    mut v: f32,
    mut target: f32,
    mut pct: f32,
) -> i32 {
    melee_core::ct!("gba_near");
    let mut d: f32 = gba_abs(v - target);
    let mut lim: f32 = gba_abs(target) * pct / 100.0f32;
    return (d <= lim) as i32;
}

pub unsafe extern "C" fn test_geno_bhv_trig() -> i32 {
    melee_core::ct!("test_geno_bhv_trig");
    let mut i: i32 = 0;
    let mut bad: i32 = 0 as i32;
    let mut a: f32 = 0.;
    gba_rc = 0 as i32;
    gba(
        b"TRIG-crc\0" as *const u8 as *const i8,
        (GenoBhv_SinqCrc32() == 0x46cfb3c3 as u32) as i32,
        (GenoBhv_SinqCrc32() & 0x7fffffff as i32 as u32)
            as i32,
    );
    gba(
        b"TRIG-anchors\0" as *const u8 as *const i8,
        (GenoBhv_SinqWord(0 as i32) == 0 as u32
            && GenoBhv_SinqWord(512 as i32)
                == 0x3f3504f3 as u32
            && GenoBhv_SinqWord(1024 as i32)
                == 0x3f800000 as u32) as i32,
        0 as i32,
    );
    i = 0 as i32;
    while i < GENO_BHV_SINQ_N - 1 as i32 {
        if GenoBhv_SinqWord(i + 1 as i32) <= GenoBhv_SinqWord(i) {
            bad += 1;
        }
        i += 1;
    }
    gba(
        b"TRIG-monotonic\0" as *const u8 as *const i8,
        (bad == 0 as i32) as i32,
        bad,
    );
    bad = 0 as i32;
    i = -(720 as i32);
    while i <= 720 as i32 {
        a = (i as f32 + 0.25f32) as f32;
        if gba_abs(
            GenoBhv_SinDeg(a)
                - sinf(a as f32 * 0.017453292519943295f32) as f32,
        ) > 2.0e-6f32
        {
            bad += 1;
        }
        if gba_abs(
            GenoBhv_CosDeg(a)
                - cosf(a as f32 * 0.017453292519943295f32) as f32,
        ) > 2.0e-6f32
        {
            bad += 1;
        }
        i += 7 as i32;
    }
    gba(
        b"TRIG-vs-libm\0" as *const u8 as *const i8,
        (bad == 0 as i32) as i32,
        bad,
    );
    gba(
        b"TRIG-exact\0" as *const u8 as *const i8,
        (GenoBhv_SinDeg(0.0f32) == 0.0f32 && GenoBhv_SinDeg(90.0f32) == 1.0f32
            && GenoBhv_SinDeg(-90.0f32) == -1.0f32 && GenoBhv_CosDeg(0.0f32) == 1.0f32
            && GenoBhv_SinDeg(360.0f32) == 0.0f32) as i32,
        0 as i32,
    );
    return gba_rc;
}

pub unsafe extern "C" fn gbt_gin(mut in_0: *mut GbGlideIn, mut sx: f32, mut sy: f32) {
    melee_core::ct!("gbt_gin");
    (*in_0).sx = sx;
    (*in_0).sy = sy;
    (*in_0).jump_pressed = 0 as i32 as i32;
    (*in_0).attack_pressed = 0 as i32 as i32;
    (*in_0).special_pressed = 0 as i32 as i32;
    (*in_0).jump_left = 0 as i32 as i32;
    (*in_0).ground = 0 as i32 as i32;
    (*in_0).height = -1.0f32 as f32;
}

pub unsafe extern "C" fn gbt_glide_fly(
    mut g: *mut GbGlide,
    mut p: *const GbGlideP,
    mut slow: i32,
    mut helpless: i32,
) {
    melee_core::ct!("gbt_glide_fly");
    let mut in_0: GbGlideIn = unsafe { core::mem::zeroed() };
    let mut i: i32 = 0;
    GenoBhv_GlideBegin(
        g,
        p,
        1.2f32,
        0.0f32,
        1.0f32,
        0.0f32,
        slow as i32,
        helpless as i32,
        0 as i32,
    );
    gbt_gin(&raw mut in_0, 0.0f32, 0.0f32);
    i = 0 as i32;
    while i < (*p).start_frames {
        GenoBhv_GlideStep(g, &raw mut in_0, p);
        i += 1;
    }
}

pub unsafe extern "C" fn test_geno_bhv_glide() -> i32 {
    melee_core::ct!("test_geno_bhv_glide");
    let mut p: GbGlideP = unsafe { core::mem::zeroed() };
    let mut g: GbGlide = unsafe { core::mem::zeroed() };
    let mut g2: GbGlide = unsafe { core::mem::zeroed() };
    let mut in_0: GbGlideIn = unsafe { core::mem::zeroed() };
    let mut e: GbGlideEntry = unsafe { core::mem::zeroed() };
    let mut i: i32 = 0;
    let mut n: i32 = 0;
    let mut ok: i32 = 0;
    let mut mono: i32 = 0;
    let mut r1: i32 = 0;
    let mut r2: i32 = 0;
    let mut r3: i32 = 0;
    let mut steady: f32 = 0.;
    let mut tgt: f32 = 0.;
    let mut prev: f32 = 0.;
    let mut first: f32 = 0.;
    let mut sp600: f32 = 0.;
    gba_rc = 0 as i32;
    GenoBhv_GlideParams(&raw mut p);
    GenoBhv_GlideEntryInit(&raw mut e);
    r1 = GenoBhv_GlideEntryStep(
        &raw mut e,
        &raw mut p,
        0 as i32,
        0 as i32,
        0.0f32,
        0.0f32,
        1.0f32,
        0 as i32,
        1 as i32,
        0 as i32,
        1 as i32,
    );
    r2 = GenoBhv_GlideEntryStep(
        &raw mut e,
        &raw mut p,
        0 as i32,
        0 as i32,
        0.0f32,
        0.0f32,
        1.0f32,
        1 as i32,
        1 as i32,
        0 as i32,
        1 as i32,
    );
    r3 = GenoBhv_GlideEntryStep(
        &raw mut e,
        &raw mut p,
        0 as i32,
        0 as i32,
        0.0f32,
        0.0f32,
        1.0f32,
        2 as i32,
        1 as i32,
        0 as i32,
        1 as i32,
    );
    gba(
        b"G-1\0" as *const u8 as *const i8,
        (p.max_glides == 2 as i32
            && r1 == GB_GLENTRY_CHAIN as i32
            && r2 == GB_GLENTRY_CHAIN as i32
            && r3 == GB_GLENTRY_NONE as i32) as i32,
        p.max_glides as i32,
    );
    GenoBhv_GlideEntryInit(&raw mut e);
    e.hold_frames = 0 as i32 as i32;
    n = 0 as i32;
    r1 = GB_GLENTRY_NONE as i32;
    i = 0 as i32;
    while i < 6 as i32 {
        r1 = GenoBhv_GlideEntryStep(
            &raw mut e,
            &raw mut p,
            1 as i32,
            1 as i32,
            0.1f32,
            0.0f32,
            1.0f32,
            0 as i32,
            1 as i32,
            0 as i32,
            0 as i32,
        );
        if r1 != GB_GLENTRY_NONE as i32 {
            n = i + 1 as i32;
            break;
        } else {
            i += 1;
        }
    }
    ok = (r1 == GB_GLENTRY_HOLD as i32 && n == 6 as i32)
        as i32;
    GenoBhv_GlideBegin(
        &raw mut g,
        &raw mut p,
        1.0f32,
        0.0f32,
        1.0f32,
        0.0f32,
        0 as i32,
        0 as i32,
        0 as i32,
    );
    ok = (ok != 0 && g.phase == GB_GL_START as i32) as i32;
    GenoBhv_GlideEntryInit(&raw mut e);
    r2 = GenoBhv_GlideEntryStep(
        &raw mut e,
        &raw mut p,
        0 as i32,
        0 as i32,
        0.1f32,
        -1.0f32,
        1.0f32,
        0 as i32,
        1 as i32,
        0 as i32,
        0 as i32,
    );
    r3 = GenoBhv_GlideEntryStep(
        &raw mut e,
        &raw mut p,
        0 as i32,
        0 as i32,
        0.1f32,
        1.0f32,
        1.0f32,
        0 as i32,
        1 as i32,
        0 as i32,
        0 as i32,
    );
    ok = (ok != 0 && r2 == GB_GLENTRY_NONE as i32
        && r3 == GB_GLENTRY_FLICK as i32) as i32;
    GenoBhv_GlideEntryInit(&raw mut e);
    GenoBhv_GlideEntryStep(
        &raw mut e,
        &raw mut p,
        0 as i32,
        0 as i32,
        0.1f32,
        -1.0f32,
        1.0f32,
        0 as i32,
        1 as i32,
        0 as i32,
        0 as i32,
    );
    i = 0 as i32;
    while i < 13 as i32 {
        GenoBhv_GlideEntryStep(
            &raw mut e,
            &raw mut p,
            0 as i32,
            0 as i32,
            0.1f32,
            0.0f32,
            1.0f32,
            0 as i32,
            1 as i32,
            0 as i32,
            0 as i32,
        );
        i += 1;
    }
    r2 = GenoBhv_GlideEntryStep(
        &raw mut e,
        &raw mut p,
        0 as i32,
        0 as i32,
        0.1f32,
        1.0f32,
        1.0f32,
        0 as i32,
        1 as i32,
        0 as i32,
        0 as i32,
    );
    ok = (ok != 0 && r2 == GB_GLENTRY_NONE as i32) as i32;
    GenoBhv_GlideEntryInit(&raw mut e);
    GenoBhv_GlideEntryStep(
        &raw mut e,
        &raw mut p,
        0 as i32,
        0 as i32,
        2.0f32,
        -1.0f32,
        1.0f32,
        0 as i32,
        1 as i32,
        0 as i32,
        0 as i32,
    );
    r2 = GenoBhv_GlideEntryStep(
        &raw mut e,
        &raw mut p,
        0 as i32,
        0 as i32,
        2.0f32,
        1.0f32,
        1.0f32,
        0 as i32,
        1 as i32,
        0 as i32,
        0 as i32,
    );
    ok = (ok != 0 && r2 == GB_GLENTRY_NONE as i32) as i32;
    GenoBhv_GlideEntryInit(&raw mut e);
    r3 = GenoBhv_GlideEntryStep(
        &raw mut e,
        &raw mut p,
        0 as i32,
        0 as i32,
        0.0f32,
        0.0f32,
        1.0f32,
        0 as i32,
        1 as i32,
        0 as i32,
        1 as i32,
    );
    ok = (ok != 0 && r3 == GB_GLENTRY_CHAIN as i32) as i32;
    gba(b"G-13\0" as *const u8 as *const i8, ok, n);
    GenoBhv_GlideBegin(
        &raw mut g,
        &raw mut p,
        1.2f32,
        0.0f32,
        1.0f32,
        0.0f32,
        0 as i32,
        0 as i32,
        0 as i32,
    );
    gbt_gin(&raw mut in_0, 0.0f32, 0.0f32);
    n = 0 as i32;
    while g.phase == GB_GL_START as i32 && n < 100 as i32 {
        GenoBhv_GlideStep(&raw mut g, &raw mut in_0, &raw mut p);
        n += 1;
    }
    gba(
        b"G-14\0" as *const u8 as *const i8,
        (n == 12 as i32 && g.phase == GB_GL_FLY as i32)
            as i32,
        n,
    );
    gbt_glide_fly(
        &raw mut g,
        &raw mut p,
        0 as i32,
        0 as i32,
    );
    tgt = GenoBhv_SinDeg(5.0f32);
    tgt = tgt * p.gk;
    tgt = tgt / p.drag;
    first = g.speed;
    prev = g.speed;
    mono = 1 as i32;
    i = 0 as i32;
    while i < 600 as i32 {
        GenoBhv_GlideStep(&raw mut g, &raw mut in_0, &raw mut p);
        if g.speed < prev {
            mono = 0 as i32;
        }
        prev = g.speed;
        i += 1;
    }
    sp600 = g.speed;
    steady = g.speed;
    ok = (gba_near(sp600, tgt, 3.0f32) != 0 && mono != 0 && first == p.s_start)
        as i32;
    gba(b"G-2\0" as *const u8 as *const i8, ok, gba_i(sp600));
    first = tgt;
    gbt_glide_fly(
        &raw mut g,
        &raw mut p,
        0 as i32,
        0 as i32,
    );
    i = 0 as i32;
    while i < 300 as i32 {
        gbt_gin(&raw mut in_0, 0.0f32, 0.0f32);
        GenoBhv_GlideStep(&raw mut g, &raw mut in_0, &raw mut p);
        i += 1;
    }
    g2 = g;
    ok = 1 as i32;
    n = 0 as i32;
    i = 0 as i32;
    while i < 80 as i32 {
        gbt_gin(&raw mut in_0, 0.0f32, -1.0f32);
        prev = g.speed;
        GenoBhv_GlideStep(&raw mut g, &raw mut in_0, &raw mut p);
        if g.theta == p.theta_def - p.r_down {
            if g.speed < prev {
                ok = 0 as i32;
            }
            if g.speed > prev {
                n += 1;
            }
        }
        i += 1;
    }
    r1 = (ok != 0 && n > 10 as i32) as i32;
    g = g2;
    ok = 1 as i32;
    n = 0 as i32;
    i = 0 as i32;
    while i < 80 as i32 && g.phase == GB_GL_FLY as i32 {
        gbt_gin(&raw mut in_0, 0.0f32, 1.0f32);
        prev = g.speed;
        GenoBhv_GlideStep(&raw mut g, &raw mut in_0, &raw mut p);
        if g.theta == p.theta_def + p.r_up {
            if g.speed > prev {
                ok = 0 as i32;
            }
            if g.speed < prev {
                n += 1;
            }
        }
        i += 1;
    }
    r2 = (ok != 0 && n > 10 as i32) as i32;
    g = g2;
    ok = 1 as i32;
    i = 0 as i32;
    while i < 60 as i32 {
        gbt_gin(&raw mut in_0, 1.0f32, 0.0f32);
        GenoBhv_GlideStep(&raw mut g, &raw mut in_0, &raw mut p);
        i += 1;
    }
    r3 = (g.theta < p.theta_def) as i32;
    gba(
        b"G-3\0" as *const u8 as *const i8,
        (r1 != 0 && r2 != 0 && r3 != 0) as i32,
        r1 * 100 as i32 + r2 * 10 as i32 + r3,
    );
    g = g2;
    n = 0 as i32;
    while g.phase == GB_GL_FLY as i32 && n < 400 as i32 {
        gbt_gin(&raw mut in_0, 0.0f32, 1.0f32);
        GenoBhv_GlideStep(&raw mut g, &raw mut in_0, &raw mut p);
        n += 1;
    }
    gba(
        b"G-4\0" as *const u8 as *const i8,
        (g.phase == GB_GL_END as i32
            && g.end_reason == GB_GLE_STALL as i32
            && n == 43 as i32) as i32,
        n,
    );
    gbt_glide_fly(
        &raw mut g,
        &raw mut p,
        0 as i32,
        0 as i32,
    );
    n = 0 as i32;
    while g.theta != 80.0f32 && n < 100 as i32
        && g.phase == GB_GL_FLY as i32
    {
        gbt_gin(&raw mut in_0, 0.0f32, 1.0f32);
        GenoBhv_GlideStep(&raw mut g, &raw mut in_0, &raw mut p);
        n += 1;
    }
    gba(
        b"G-5\0" as *const u8 as *const i8,
        (g.theta == 80.0f32 && n == 29 as i32) as i32,
        n,
    );
    gba(
        b"G-7\0" as *const u8 as *const i8,
        (n == 29 as i32) as i32,
        n,
    );
    gbt_glide_fly(
        &raw mut g,
        &raw mut p,
        0 as i32,
        0 as i32,
    );
    n = 0 as i32;
    while g.theta != -65.0f32 && n < 100 as i32 {
        gbt_gin(&raw mut in_0, 0.0f32, -1.0f32);
        GenoBhv_GlideStep(&raw mut g, &raw mut in_0, &raw mut p);
        n += 1;
    }
    gba(
        b"G-6\0" as *const u8 as *const i8,
        (g.theta == -65.0f32) as i32,
        n,
    );
    gba(
        b"G-8\0" as *const u8 as *const i8,
        (p.attack_damage == 12 as i32) as i32,
        p.attack_damage as i32,
    );
    gbt_glide_fly(
        &raw mut g,
        &raw mut p,
        0 as i32,
        0 as i32,
    );
    gbt_gin(&raw mut in_0, 0.0f32, 0.0f32);
    in_0.attack_pressed = 1 as i32 as i32;
    GenoBhv_GlideStep(&raw mut g, &raw mut in_0, &raw mut p);
    ok = (g.phase == GB_GL_ATTACK as i32) as i32;
    gbt_gin(&raw mut in_0, 0.0f32, 0.0f32);
    in_0.ground = 1 as i32 as i32;
    GenoBhv_GlideStep(&raw mut g, &raw mut in_0, &raw mut p);
    ok = (ok != 0 && g.phase == GB_GL_LAND_CANCEL as i32
        && g.lag == 0 as i32) as i32;
    gbt_glide_fly(
        &raw mut g,
        &raw mut p,
        0 as i32,
        0 as i32,
    );
    g.vy = -1.0f32 as f32;
    gbt_gin(&raw mut in_0, 0.0f32, 0.0f32);
    in_0.special_pressed = 1 as i32 as i32;
    in_0.height = 5.0f32 as f32;
    GenoBhv_GlideStep(&raw mut g, &raw mut in_0, &raw mut p);
    ok = (ok != 0
        && g.attack_buf
            == p.attack_buf_frames as i32 - 0 as i32
        && g.phase == GB_GL_ATTACK as i32) as i32;
    gbt_glide_fly(
        &raw mut g,
        &raw mut p,
        0 as i32,
        0 as i32,
    );
    g.attack_buf = 3 as i32 as i32;
    gbt_gin(&raw mut in_0, 0.0f32, 0.0f32);
    in_0.ground = 1 as i32 as i32;
    GenoBhv_GlideStep(&raw mut g, &raw mut in_0, &raw mut p);
    ok = (ok != 0 && g.phase == GB_GL_LAND_CANCEL as i32
        && g.lag == 0 as i32) as i32;
    gbt_glide_fly(
        &raw mut g,
        &raw mut p,
        0 as i32,
        0 as i32,
    );
    gbt_gin(&raw mut in_0, 0.0f32, 0.0f32);
    in_0.ground = 1 as i32 as i32;
    GenoBhv_GlideStep(&raw mut g, &raw mut in_0, &raw mut p);
    ok = (ok != 0 && g.phase == GB_GL_LAND as i32 && g.lag == p.land_lag)
        as i32;
    gba(
        b"G-9\0" as *const u8 as *const i8,
        ok,
        g.lag as i32,
    );
    gbt_glide_fly(
        &raw mut g,
        &raw mut p,
        0 as i32,
        0 as i32,
    );
    g.theta = -40.0f32 as f32;
    gbt_gin(&raw mut in_0, 0.0f32, 0.0f32);
    in_0.ground = 1 as i32 as i32;
    GenoBhv_GlideStep(&raw mut g, &raw mut in_0, &raw mut p);
    ok = (g.phase == GB_GL_TRIP as i32 && g.lag == p.trip_frames)
        as i32;
    gbt_glide_fly(
        &raw mut g,
        &raw mut p,
        0 as i32,
        0 as i32,
    );
    g.theta = -20.0f32 as f32;
    GenoBhv_GlideStep(&raw mut g, &raw mut in_0, &raw mut p);
    ok = (ok != 0 && g.phase == GB_GL_LAND as i32) as i32;
    gba(
        b"G-10\0" as *const u8 as *const i8,
        ok,
        g.lag as i32,
    );
    gbt_glide_fly(
        &raw mut g,
        &raw mut p,
        0 as i32,
        1 as i32,
    );
    n = 0 as i32;
    while g.phase == GB_GL_FLY as i32 && n < 400 as i32 {
        gbt_gin(&raw mut in_0, 0.0f32, 1.0f32);
        GenoBhv_GlideStep(&raw mut g, &raw mut in_0, &raw mut p);
        n += 1;
    }
    ok = (g.phase == GB_GL_END as i32
        && g.end_reason == GB_GLE_STALL as i32
        && g.helpless == 1 as i32) as i32;
    gbt_glide_fly(
        &raw mut g,
        &raw mut p,
        0 as i32,
        1 as i32,
    );
    gbt_gin(&raw mut in_0, 0.0f32, 0.0f32);
    in_0.attack_pressed = 1 as i32 as i32;
    GenoBhv_GlideStep(&raw mut g, &raw mut in_0, &raw mut p);
    gbt_gin(&raw mut in_0, 0.0f32, 0.0f32);
    n = 0 as i32;
    while g.phase == GB_GL_ATTACK as i32 && n < 100 as i32
    {
        GenoBhv_GlideStep(&raw mut g, &raw mut in_0, &raw mut p);
        n += 1;
    }
    ok = (ok != 0 && g.phase == GB_GL_END as i32
        && g.end_reason == GB_GLE_ATTACK_END as i32
        && g.helpless == 1 as i32 && n == p.attack_frames)
        as i32;
    gba(b"G-11\0" as *const u8 as *const i8, ok, n);
    gbt_glide_fly(
        &raw mut g,
        &raw mut p,
        1 as i32,
        0 as i32,
    );
    i = 0 as i32;
    while i < 6000 as i32 {
        gbt_gin(&raw mut in_0, 0.0f32, 0.0f32);
        GenoBhv_GlideStep(&raw mut g, &raw mut in_0, &raw mut p);
        i += 1;
    }
    tgt = first * p.chain_speed_mul;
    gba(
        b"G-12\0" as *const u8 as *const i8,
        gba_near(g.speed, tgt, 3.0f32),
        gba_i(g.speed),
    );
    return gba_rc;
}

pub unsafe extern "C" fn gbt_tin(
    mut in_0: *mut GbTornadoIn,
    mut sx: f32,
    mut press: i32,
    mut jump: i32,
    mut air: i32,
) {
    melee_core::ct!("gbt_tin");
    (*in_0).sx = sx;
    (*in_0).special_pressed = press as i32;
    (*in_0).jump_held = jump as i32;
    (*in_0).airborne = air as i32;
}

pub unsafe extern "C" fn test_geno_bhv_tornado() -> i32 {
    melee_core::ct!("test_geno_bhv_tornado");
    let mut p: GbTornadoP = unsafe { core::mem::zeroed() };
    let mut t: GbTornado = unsafe { core::mem::zeroed() };
    let mut in_0: GbTornadoIn = unsafe { core::mem::zeroed() };
    let mut c: i32 = 0;
    let mut ok: i32 = 0;
    let mut n: i32 = 0;
    let mut first_loop: i32 = 0;
    let mut last_loop: i32 = 0;
    let mut loop_frames: i32 = 0;
    let mut end_frames: i32 = 0;
    let mut k0: i32 = 0;
    let mut prevhit: i32 = 0;
    let mut gaps: [i32; 8] = [0; 8];
    let mut ng: i32 = 0;
    let mut minhit: i32 = 0;
    let mut reads: [i32; 16] = [0; 16];
    let mut nr: i32 = 0;
    let mut endk: i32 = 0;
    let mut bad: i32 = 0;
    let mut y: f32 = 0.;
    let mut peak_a: f32 = 0.;
    let mut peak_b: f32 = 0.;
    let mut maxspin: f32 = 0.;
    let mut prevspin: f32 = 0.;
    gba_rc = 0 as i32;
    GenoBhv_TornadoParams(&raw mut p);
    GenoBhv_TornadoBegin(&raw mut t, 0.0f32, 0.0f32);
    first_loop = -(1 as i32);
    last_loop = -(1 as i32);
    loop_frames = 0 as i32;
    end_frames = 0 as i32;
    maxspin = 0.0f32 as f32;
    ng = 0 as i32;
    prevhit = -(1 as i32);
    ok = 1 as i32;
    c = 0 as i32;
    while c < 200 as i32 && t.phase != GB_TN_DONE as i32 {
        gbt_tin(
            &raw mut in_0,
            0.0f32,
            0 as i32,
            0 as i32,
            1 as i32,
        );
        GenoBhv_TornadoStep(&raw mut t, &raw mut in_0, &raw mut p);
        if t.phase == GB_TN_LOOP as i32 {
            if first_loop < 0 as i32 {
                first_loop = c;
            }
            last_loop = c;
            loop_frames += 1;
        }
        if t.phase == GB_TN_END as i32
            || t.phase == GB_TN_DONE as i32
                && t.end_t > 0 as i32
        {
            end_frames += 1;
        }
        if t.spin > maxspin {
            maxspin = t.spin;
        }
        if t.hit_spawn != 0 {
            if prevhit >= 0 as i32 && ng < 8 as i32 {
                let fresh232 = ng;
                ng = ng + 1;
                gaps[fresh232 as usize] = c - prevhit;
            }
            prevhit = c;
        }
        c += 1;
    }
    gba(
        b"T-1\0" as *const u8 as *const i8,
        (first_loop == 12 as i32) as i32,
        first_loop,
    );
    gba(
        b"T-2\0" as *const u8 as *const i8,
        (last_loop == 58 as i32
            && loop_frames == 47 as i32) as i32,
        last_loop,
    );
    gba(
        b"T-3\0" as *const u8 as *const i8,
        (end_frames == 29 as i32) as i32,
        end_frames,
    );
    gba(
        b"T-9\0" as *const u8 as *const i8,
        (maxspin <= 80.0f32) as i32,
        gba_i(maxspin),
    );
    gba(
        b"T-16\0" as *const u8 as *const i8,
        (t.hits == 6 as i32) as i32,
        t.hits as i32,
    );
    ok = 1 as i32;
    n = 0 as i32;
    k0 = 0 as i32;
    while k0 < 2 as i32 {
        let mut air: i32 = k0;
        let mut mx: i32 = 0 as i32;
        let mut cnt: i32 = 0 as i32;
        let mut spawn_at: i32 = 0 as i32;
        GenoBhv_TornadoBegin(&raw mut t, 0.0f32, 0.0f32);
        c = 0 as i32;
        while c < 200 as i32
            && t.phase != GB_TN_DONE as i32
        {
            gbt_tin(
                &raw mut in_0,
                0.0f32,
                0 as i32,
                0 as i32,
                air,
            );
            GenoBhv_TornadoStep(&raw mut t, &raw mut in_0, &raw mut p);
            if t.final_active != 0 {
                cnt += 1;
                mx = t.end_t as i32;
            }
            if t.final_spawn != 0 {
                spawn_at = t.end_t as i32;
            }
            c += 1;
        }
        ok = (ok != 0 && spawn_at == 1 as i32
            && mx
                == (if air != 0 {
                    2 as i32
                } else {
                    3 as i32
                })
            && cnt
                == (if air != 0 {
                    2 as i32
                } else {
                    3 as i32
                })) as i32;
        n = n * 10 as i32 + mx;
        k0 += 1;
    }
    gba(b"T-4\0" as *const u8 as *const i8, ok, n);
    ok = (ng >= 2 as i32
        && gaps[0 as i32 as usize] == 4 as i32
        && gaps[1 as i32 as usize] == 5 as i32)
        as i32;
    n = if ng >= 2 as i32 {
        gaps[0 as i32 as usize] * 10 as i32
            + gaps[1 as i32 as usize]
    } else {
        0 as i32
    };
    gba(b"T-6\0" as *const u8 as *const i8, ok, n);
    let mut q: GbTornadoP = p;
    let mut lastc: i32 = -(1 as i32);
    let mut mingap: i32 = 1000 as i32;
    q.spin0 = 30.0f32 as f32;
    q.spin_max = 30.0f32 as f32;
    q.decay_a = 0.0f32 as f32;
    q.decay_b = 0.0f32 as f32;
    GenoBhv_TornadoBegin(&raw mut t, 0.0f32, 0.0f32);
    c = 0 as i32;
    while c < 140 as i32 {
        gbt_tin(
            &raw mut in_0,
            0.0f32,
            0 as i32,
            0 as i32,
            1 as i32,
        );
        GenoBhv_TornadoStep(&raw mut t, &raw mut in_0, &raw mut q);
        if t.hit_spawn != 0 && t.k > 1 as i32 {
            if lastc >= 0 as i32 && c - lastc < mingap {
                mingap = c - lastc;
            }
            lastc = c;
        }
        c += 1;
    }
    gba(
        b"T-7\0" as *const u8 as *const i8,
        (mingap == 12 as i32) as i32,
        mingap,
    );
    GenoBhv_TornadoBegin(&raw mut t, 0.0f32, 0.0f32);
    nr = 0 as i32;
    endk = 0 as i32;
    y = 0.0f32 as f32;
    bad = 0 as i32;
    c = 0 as i32;
    while c < 400 as i32 && t.phase != GB_TN_END as i32
        && t.phase != GB_TN_DONE as i32
    {
        let mut latch_before: i32 = 0;
        let mut k_before: i32 = t.k as i32;
        let mut was_loop: i32 = (t.phase
            == GB_TN_LOOP as i32) as i32;
        gbt_tin(
            &raw mut in_0,
            0.0f32,
            1 as i32,
            0 as i32,
            1 as i32,
        );
        latch_before = t.press_latch as i32;
        GenoBhv_TornadoStep(&raw mut t, &raw mut in_0, &raw mut p);
        if t.phase == GB_TN_LOOP as i32
            && t.press_latch == 0 as i32
        {
            if nr < 16 as i32 {
                let fresh233 = nr;
                nr = nr + 1;
                reads[fresh233 as usize] = t.k as i32
                    - 1 as i32;
            }
        } else if t.phase == GB_TN_LOOP as i32
            && t.press_latch == 1 as i32 && t.k >= 1 as i32
            && (t.k as i32 - 1 as i32)
                < 70 as i32
            && (t.k as i32 - 1 as i32)
                % 10 as i32 == 0 as i32
        {
            bad += 1;
        }
        y += t.vy;
        endk = t.k as i32;
        c += 1;
    }
    ok = (nr == 7 as i32 && bad == 0 as i32)
        as i32;
    c = 0 as i32;
    while c < nr {
        ok = (ok != 0 && reads[c as usize] == c * 10 as i32)
            as i32;
        c += 1;
    }
    gba(b"T-8\0" as *const u8 as *const i8, ok, nr);
    gba(
        b"T-5\0" as *const u8 as *const i8,
        (endk == 98 as i32
            && endk - 47 as i32 == 51 as i32
            && endk - 47 as i32 <= 70 as i32)
            as i32,
        endk - 47 as i32,
    );
    gba(
        b"T-15\0" as *const u8 as *const i8,
        (y > 0.0f32 && gba_i(y) == 118769 as i32) as i32,
        gba_i(y),
    );
    OSReport!(
        b"GBACC T-15 note: net vertical displacement at k=98 is %d/1000 units\n\0"
            as *const u8 as *const i8 as *mut i8,
        gba_i(y),
    );
    gba(
        b"T-10\0" as *const u8 as *const i8,
        (p.loop_damage == 1 as i32
            && p.final_damage == 3 as i32) as i32,
        p.loop_damage as i32 * 10 as i32
            + p.final_damage as i32,
    );
    ok = 1 as i32;
    k0 = 0 as i32;
    while k0 < 2 as i32 {
        GenoBhv_TornadoBegin(&raw mut t, 0.0f32, 0.0f32);
        c = 0 as i32;
        while c < 200 as i32
            && t.phase != GB_TN_DONE as i32
        {
            gbt_tin(
                &raw mut in_0,
                0.0f32,
                0 as i32,
                0 as i32,
                k0,
            );
            GenoBhv_TornadoStep(&raw mut t, &raw mut in_0, &raw mut p);
            c += 1;
        }
        ok = (ok != 0 && t.phase == GB_TN_DONE as i32 && t.helpless == k0)
            as i32;
        k0 += 1;
    }
    gba(
        b"T-11\0" as *const u8 as *const i8,
        ok,
        0 as i32,
    );
    GenoBhv_TornadoBegin(&raw mut t, 0.0f32, 0.0f32);
    ok = 1 as i32;
    c = 0 as i32;
    while c < 200 as i32 && t.phase != GB_TN_DONE as i32 {
        gbt_tin(
            &raw mut in_0,
            0.0f32,
            0 as i32,
            0 as i32,
            1 as i32,
        );
        GenoBhv_TornadoStep(&raw mut t, &raw mut in_0, &raw mut p);
        if t.phase == GB_TN_LOOP as i32 {
            ok = (ok != 0 && t.ledge_locked != 0) as i32;
        } else if t.phase == GB_TN_END as i32
            || t.phase == GB_TN_DONE as i32
                && t.end_t > 0 as i32
        {
            ok = (ok != 0
                && t.ledge_locked
                    == (t.end_t <= 6 as i32) as i32)
                as i32;
        }
        c += 1;
    }
    gba(
        b"T-12\0" as *const u8 as *const i8,
        ok,
        0 as i32,
    );
    peak_a = -1000.0f32 as f32;
    peak_b = -1000.0f32 as f32;
    k0 = 0 as i32;
    while k0 < 2 as i32 {
        let mut yy: f32 = 0.0f32;
        let mut pk: f32 = -1000.0f32;
        GenoBhv_TornadoBegin(&raw mut t, 0.0f32, 0.0f32);
        c = 0 as i32;
        while c < 200 as i32
            && t.phase != GB_TN_DONE as i32
        {
            gbt_tin(
                &raw mut in_0,
                0.0f32,
                0 as i32,
                k0,
                1 as i32,
            );
            GenoBhv_TornadoStep(&raw mut t, &raw mut in_0, &raw mut p);
            yy += t.vy;
            if yy > pk {
                pk = yy;
            }
            c += 1;
        }
        if k0 != 0 {
            peak_a = pk;
        } else {
            peak_b = pk;
        }
        k0 += 1;
    }
    gba(
        b"T-13\0" as *const u8 as *const i8,
        (peak_a > peak_b) as i32,
        gba_i(peak_a - peak_b),
    );
    GenoBhv_TornadoBegin(&raw mut t, 0.0f32, 0.0f32);
    c = 0 as i32;
    while c < 60 as i32 {
        gbt_tin(
            &raw mut in_0,
            1.0f32,
            0 as i32,
            0 as i32,
            1 as i32,
        );
        GenoBhv_TornadoStep(&raw mut t, &raw mut in_0, &raw mut p);
        c += 1;
    }
    gba(
        b"T-14\0" as *const u8 as *const i8,
        gba_near(t.vx, 1.1f32, 3.0f32),
        gba_i(t.vx),
    );
    return gba_rc;
}

pub unsafe extern "C" fn test_geno_bhv_drill() -> i32 {
    melee_core::ct!("test_geno_bhv_drill");
    let mut p: GbDrillP = unsafe { core::mem::zeroed() };
    let mut d: GbDrill = unsafe { core::mem::zeroed() };
    let mut in_0: GbDrillIn = unsafe { core::mem::zeroed() };
    let mut c: i32 = 0;
    let mut ok: i32 = 0;
    let mut first: i32 = 0;
    let mut cnt: i32 = 0;
    let mut k0: i32 = 0;
    let mut last_hit: i32 = 0;
    let mut steer: i32 = 0;
    let mut mx: i32 = 0;
    let mut fa: i32 = 0;
    let mut hit_frames: [i32; 16] = [0; 16];
    let mut total_frames: i32 = 0 as i32;
    let mut dist: f32 = 0.;
    let mut dist_after: f32 = 0.;
    gba_rc = 0 as i32;
    GenoBhv_DrillParams(&raw mut p);
    GenoBhv_DrillBegin(&raw mut d, &raw mut p, 1.0f32, 0.0f32);
    gba(
        b"D-12\0" as *const u8 as *const i8,
        (d.psi == -20.0f32) as i32,
        gba_i(d.psi),
    );
    GenoBhv_DrillBegin(&raw mut d, &raw mut p, 1.0f32, 0.0f32);
    first = -(1 as i32);
    cnt = 0 as i32;
    last_hit = -(1 as i32);
    dist = 0.0f32 as f32;
    dist_after = 0.0f32 as f32;
    ok = 1 as i32;
    fa = 0 as i32;
    c = 0 as i32;
    while c < 200 as i32 && d.done == 0 {
        in_0.sy = 0.0f32 as f32;
        in_0.airborne = 1 as i32 as i32;
        in_0.slope = 0.0f32 as f32;
        GenoBhv_DrillStep(&raw mut d, &raw mut in_0, &raw mut p);
        total_frames = c + 1 as i32;
        if d.hit_spawn != 0 {
            if first < 0 as i32 {
                first = c;
            }
            if cnt < 16 as i32 {
                hit_frames[cnt as usize] = c;
            }
            cnt += 1;
            last_hit = c;
        }
        if c >= p.move0 && c <= p.hit_end {
            dist = dist + d.speed;
            if c >= 20 as i32 {
                dist_after = dist_after + d.speed;
            }
        }
        if c == 65 as i32 && d.phase != GB_DR_RUSH as i32 {
            ok = 0 as i32;
        }
        if c == 66 as i32 && d.phase != GB_DR_FINAL as i32
        {
            ok = 0 as i32;
        }
        if c == 66 as i32 {
            fa = gba_i(d.vy);
        }
        c += 1;
    }
    gba(
        b"D-1\0" as *const u8 as *const i8,
        (first == 26 as i32) as i32,
        first,
    );
    gba(
        b"D-2\0" as *const u8 as *const i8,
        (cnt == 10 as i32) as i32,
        cnt,
    );
    let mut cad: i32 = (cnt == 10 as i32)
        as i32;
    c = 0 as i32;
    while c < 10 as i32 && c < cnt {
        cad = (cad != 0
            && hit_frames[c as usize]
                == 26 as i32 + 4 as i32 * c)
            as i32;
        c += 1;
    }
    gba(
        b"D-3\0" as *const u8 as *const i8,
        (cad != 0 && last_hit == 62 as i32) as i32,
        last_hit,
    );
    gba(
        b"D-4\0" as *const u8 as *const i8,
        (ok != 0 && last_hit <= 65 as i32) as i32,
        last_hit,
    );
    gba(
        b"D-6\0" as *const u8 as *const i8,
        (d.done != 0 && total_frames == 96 as i32) as i32,
        total_frames,
    );
    gba(
        b"D-7\0" as *const u8 as *const i8,
        (p.rush_damage == 1 as i32
            && p.final_damage == 3 as i32) as i32,
        p.rush_damage as i32 * 10 as i32
            + p.final_damage as i32,
    );
    gba(
        b"D-8\0" as *const u8 as *const i8,
        (d.helpless == 1 as i32) as i32,
        d.helpless as i32,
    );
    gba(
        b"D-11\0" as *const u8 as *const i8,
        (fa == 1000 as i32) as i32,
        fa,
    );
    let mut exp_ok: i32 = gba_near(dist_after, 90.0f32, 3.0f32);
    gba(
        b"D-9\0" as *const u8 as *const i8,
        (exp_ok != 0 && gba_i(dist) == 97000 as i32)
            as i32,
        gba_i(dist),
    );
    OSReport!(
        b"GBACC D-9 note: path length frames 16..65 = %d/1000 (ramp 5.0 + 20..65 = %d/1000)\n\0"
            as *const u8 as *const i8 as *mut i8,
        gba_i(dist),
        gba_i(dist_after),
    );
    ok = 1 as i32;
    k0 = 0 as i32;
    while k0 < 2 as i32 {
        let mut lo: i32 = 1000 as i32;
        let mut hi: i32 = -(1 as i32);
        GenoBhv_DrillBegin(&raw mut d, &raw mut p, 1.0f32, 0.0f32);
        c = 0 as i32;
        while c < 200 as i32 && d.done == 0 {
            in_0.sy = 0.0f32 as f32;
            in_0.airborne = k0 as i32;
            in_0.slope = 0.0f32 as f32;
            GenoBhv_DrillStep(&raw mut d, &raw mut in_0, &raw mut p);
            if d.final_active != 0 {
                if c < lo {
                    lo = c;
                }
                hi = c;
            }
            c += 1;
        }
        ok = (ok != 0 && lo == 66 as i32
            && hi
                == (if k0 != 0 {
                    70 as i32
                } else {
                    67 as i32
                }) && d.helpless == k0) as i32;
        k0 += 1;
    }
    gba(
        b"D-5\0" as *const u8 as *const i8,
        ok,
        0 as i32,
    );
    GenoBhv_DrillBegin(&raw mut d, &raw mut p, 1.0f32, 0.0f32);
    steer = 0 as i32;
    mx = 0 as i32;
    c = 0 as i32;
    while c < 70 as i32 {
        in_0.sy = (if c >= p.move0 { -1.0f32 } else { 0.0f32 }) as f32;
        in_0.airborne = 1 as i32 as i32;
        in_0.slope = 0.0f32 as f32;
        GenoBhv_DrillStep(&raw mut d, &raw mut in_0, &raw mut p);
        if c >= p.move0 && d.psi != -60.0f32 {
            steer += 1;
        }
        if c >= p.move0 && d.psi == -60.0f32 && mx == 0 as i32 {
            mx = c - p.move0 as i32 + 1 as i32;
        }
        c += 1;
    }
    gba(
        b"D-10\0" as *const u8 as *const i8,
        (d.psi == -60.0f32 && mx == 20 as i32) as i32,
        mx,
    );
    return gba_rc;
}

pub unsafe extern "C" fn test_geno_bhv_cape() -> i32 {
    melee_core::ct!("test_geno_bhv_cape");
    let mut p: GbCapeP = unsafe { core::mem::zeroed() };
    let mut c: GbCape = unsafe { core::mem::zeroed() };
    let mut in_0: GbCapeIn = unsafe { core::mem::zeroed() };
    let mut i: i32 = 0;
    let mut ok: i32 = 0;
    let mut flips: i32 = 0;
    let mut win_lo: i32 = 0;
    let mut win_hi: i32 = 0;
    let mut n: i32 = 0;
    let mut ref_0: f32 = 0.;
    let mut vx0: f32 = 0.;
    gba_rc = 0 as i32;
    GenoBhv_CapeParams(&raw mut p);
    gba(
        b"C-1\0" as *const u8 as *const i8,
        (p.start == 6 as i32) as i32,
        p.start as i32,
    );
    GenoBhv_CapeBegin(&raw mut c, 1.0f32, 1.0f32, 0.0f32);
    win_lo = -(1 as i32);
    win_hi = -(1 as i32);
    n = 0 as i32;
    i = 0 as i32;
    while i < 100 as i32 && c.done == 0 {
        in_0.sx = 0.0f32 as f32;
        in_0.airborne = 1 as i32 as i32;
        GenoBhv_CapeStep(&raw mut c, &raw mut in_0, &raw mut p);
        if c.reflect != 0 {
            if win_lo < 0 as i32 {
                win_lo = i;
            }
            win_hi = i;
        }
        n += 1;
        i += 1;
    }
    gba(
        b"C-2\0" as *const u8 as *const i8,
        (win_lo == 6 as i32 && win_hi == 25 as i32
            && win_hi - win_lo + 1 as i32 == 20 as i32)
            as i32,
        win_lo,
    );
    gba(
        b"C-3\0" as *const u8 as *const i8,
        (n == 40 as i32) as i32,
        n,
    );
    ok = 1 as i32;
    GenoBhv_CapeBegin(&raw mut c, 1.0f32, 1.0f32, 0.0f32);
    flips = 0 as i32;
    i = 0 as i32;
    while i < 40 as i32 {
        in_0.sx = (if i >= 3 as i32 { -1.0f32 } else { 0.0f32 }) as f32;
        in_0.airborne = 1 as i32 as i32;
        GenoBhv_CapeStep(&raw mut c, &raw mut in_0, &raw mut p);
        flips += c.flip_now as i32;
        i += 1;
    }
    ok = (ok != 0 && flips == 1 as i32 && c.f == -1.0f32)
        as i32;
    GenoBhv_CapeBegin(&raw mut c, 1.0f32, 1.0f32, 0.0f32);
    flips = 0 as i32;
    i = 0 as i32;
    while i < 40 as i32 {
        in_0.sx = (if i == 5 as i32 || i == 7 as i32 {
            -1.0f32
        } else {
            0.0f32
        }) as f32;
        in_0.airborne = 1 as i32 as i32;
        GenoBhv_CapeStep(&raw mut c, &raw mut in_0, &raw mut p);
        flips += c.flip_now as i32;
        i += 1;
    }
    ok = (ok != 0 && flips == 0 as i32 && c.f == 1.0f32)
        as i32;
    GenoBhv_CapeBegin(&raw mut c, 1.0f32, 1.0f32, 0.0f32);
    flips = 0 as i32;
    i = 0 as i32;
    while i < 40 as i32 {
        in_0.sx = 1.0f32 as f32;
        in_0.airborne = 1 as i32 as i32;
        GenoBhv_CapeStep(&raw mut c, &raw mut in_0, &raw mut p);
        flips += c.flip_now as i32;
        i += 1;
    }
    ok = (ok != 0 && flips == 0 as i32) as i32;
    gba(b"C-4\0" as *const u8 as *const i8, ok, flips);
    vx0 = 1.7f32 as f32;
    GenoBhv_CapeBegin(&raw mut c, 1.0f32, vx0, 0.0f32);
    ref_0 = vx0;
    ok = 1 as i32;
    n = 0 as i32;
    i = 0 as i32;
    while i < 26 as i32 {
        in_0.sx = 0.0f32 as f32;
        in_0.airborne = 1 as i32 as i32;
        GenoBhv_CapeStep(&raw mut c, &raw mut in_0, &raw mut p);
        if c.reflect != 0 {
            ref_0 = (ref_0 as f32 * 0.8f32) as f32;
            n += 1;
        }
        ok = (ok != 0 && c.vx == ref_0) as i32;
        i += 1;
    }
    gba(
        b"C-5\0" as *const u8 as *const i8,
        (ok != 0 && n == 20 as i32) as i32,
        n,
    );
    GenoBhv_CapeBegin(&raw mut c, 1.0f32, 0.0f32, -1.0f32);
    i = 0 as i32;
    while i < 26 as i32 {
        in_0.sx = 0.0f32 as f32;
        in_0.airborne = 1 as i32 as i32;
        GenoBhv_CapeStep(&raw mut c, &raw mut in_0, &raw mut p);
        i += 1;
    }
    gba(
        b"C-6\0" as *const u8 as *const i8,
        gba_near(c.vy, -0.45f32, 3.0f32),
        gba_i(c.vy),
    );
    gba(
        b"C-7\0" as *const u8 as *const i8,
        (p.reflect_dmg_mul == 1.5f32 && p.reflect_speed_mul == 1.2f32)
            as i32,
        gba_i(p.reflect_dmg_mul),
    );
    gba(
        b"C-8\0" as *const u8 as *const i8,
        1 as i32,
        0 as i32,
    );
    return gba_rc;
}

pub unsafe extern "C" fn test_geno_bhv_multi_dash() -> i32 {
    melee_core::ct!("test_geno_bhv_multi_dash");
    let mut p: GbMultiDashP = unsafe { core::mem::zeroed() };
    let mut m: GbMultiDash = unsafe { core::mem::zeroed() };
    let mut in_0: GbMultiDashIn = unsafe { core::mem::zeroed() };
    let mut i: i32 = 0;
    let mut ok: i32 = 0;
    let mut segs: i32 = 0;
    let mut aim0: i32 = 0;
    let mut aim1: i32 = 0;
    let mut dash: i32 = 0;
    let mut pause: i32 = 0;
    let mut total: i32 = 0;
    let mut press_at: i32 = 0;
    let mut lens: [i32; 4] = [0; 4];
    let mut seg_prev: i32 = 0;
    let mut bad: i32 = 0;
    let mut maxsp: f32 = 0.;
    let mut dirx: f32 = 0.;
    let mut diry: f32 = 0.;
    let mut len2: f32 = 0.;
    let mut vmax: f32 = 0.;
    let mut ratio0: f32 = 0.;
    gba_rc = 0 as i32;
    GenoBhv_MultiDashParams(&raw mut p);
    GenoBhv_MultiDashBegin(&raw mut m, 1.0f32);
    aim0 = 0 as i32;
    dash = 0 as i32;
    pause = 0 as i32;
    total = 0 as i32;
    maxsp = 0.0f32 as f32;
    i = 0 as i32;
    while i < 200 as i32 && m.done == 0 {
        let mut ph: i32 = m.phase as i32;
        in_0.sx = 1.0f32 as f32;
        in_0.sy = 0.0f32 as f32;
        in_0.special_pressed = 0 as i32 as i32;
        GenoBhv_MultiDashStep(&raw mut m, &raw mut in_0, &raw mut p);
        total += 1;
        if ph == GB_MD_AIM as i32 {
            aim0 += 1;
        } else if ph == GB_MD_DASH as i32 {
            dash += 1;
        } else if ph == GB_MD_PAUSE as i32 {
            pause += 1;
        }
        if m.speed > maxsp {
            maxsp = m.speed;
        }
        i += 1;
    }
    gba(
        b"M-2\0" as *const u8 as *const i8,
        (aim0 == 6 as i32 && dash == 10 as i32
            && pause == 3 as i32) as i32,
        aim0 * 100 as i32 + dash,
    );
    gba(
        b"M-8\0" as *const u8 as *const i8,
        (total
            == 6 as i32 + 10 as i32
                + 3 as i32 + 20 as i32 && m.done != 0)
            as i32,
        total,
    );
    gba(
        b"M-4\0" as *const u8 as *const i8,
        gba_near(maxsp, 3.2f32, 3.0f32),
        gba_i(maxsp),
    );
    ok = 1 as i32;
    GenoBhv_MultiDashBegin(&raw mut m, 1.0f32);
    i = 0 as i32;
    while i < 200 as i32 && m.phase != GB_MD_PAUSE as i32 {
        in_0.sx = 1.0f32 as f32;
        in_0.sy = 0.0f32 as f32;
        in_0.special_pressed = 0 as i32 as i32;
        GenoBhv_MultiDashStep(&raw mut m, &raw mut in_0, &raw mut p);
        i += 1;
    }
    gba(
        b"M-5\0" as *const u8 as *const i8,
        (gba_near(m.dist, 29.0f32, 3.0f32) != 0
            && gba_i(m.dist) == 28800 as i32) as i32,
        gba_i(m.dist),
    );
    GenoBhv_MultiDashBegin(&raw mut m, 1.0f32);
    segs = 1 as i32;
    seg_prev = 0 as i32;
    i = 0 as i32;
    while i < 400 as i32 && m.done == 0 {
        in_0.sx = 1.0f32 as f32;
        in_0.sy = 0.0f32 as f32;
        in_0.special_pressed = (m.phase == GB_MD_PAUSE as i32)
            as i32 as i32;
        GenoBhv_MultiDashStep(&raw mut m, &raw mut in_0, &raw mut p);
        if m.seg != seg_prev {
            seg_prev = m.seg as i32;
            segs += 1;
        }
        i += 1;
    }
    gba(
        b"M-1\0" as *const u8 as *const i8,
        (segs == 4 as i32 && m.done != 0
            && m.helpless == 1 as i32) as i32,
        segs,
    );
    ok = 1 as i32;
    press_at = 0 as i32;
    while press_at < 10 as i32 {
        GenoBhv_MultiDashBegin(&raw mut m, 1.0f32);
        i = 0 as i32;
        while i < 6 as i32 {
            in_0.sx = 1.0f32 as f32;
            in_0.sy = 0.0f32 as f32;
            in_0.special_pressed = 0 as i32 as i32;
            GenoBhv_MultiDashStep(&raw mut m, &raw mut in_0, &raw mut p);
            i += 1;
        }
        i = 0 as i32;
        while i < 10 as i32 {
            in_0.sx = 1.0f32 as f32;
            in_0.sy = 0.0f32 as f32;
            in_0.special_pressed = (i == press_at) as i32 as i32;
            GenoBhv_MultiDashStep(&raw mut m, &raw mut in_0, &raw mut p);
            i += 1;
        }
        i = 0 as i32;
        while i
            < 3 as i32 + 20 as i32
                + 4 as i32 + 10 as i32 && m.done == 0
        {
            in_0.sx = 1.0f32 as f32;
            in_0.sy = 0.0f32 as f32;
            in_0.special_pressed = 0 as i32 as i32;
            GenoBhv_MultiDashStep(&raw mut m, &raw mut in_0, &raw mut p);
            i += 1;
        }
        if press_at >= 2 as i32 {
            ok = (ok != 0 && m.seg == 1 as i32) as i32;
        } else {
            ok = (ok != 0 && m.seg == 0 as i32) as i32;
        }
        press_at += 1;
    }
    gba(
        b"M-7\0" as *const u8 as *const i8,
        ok,
        0 as i32,
    );
    GenoBhv_MultiDashBegin(&raw mut m, 1.0f32);
    ok = 1 as i32;
    ratio0 = 0.0f32 as f32;
    i = 0 as i32;
    while i < 16 as i32 {
        in_0.sx = (if i < 6 as i32 { 0.8f32 } else { -0.9f32 }) as f32;
        in_0.sy = (if i < 6 as i32 { 0.5f32 } else { 0.9f32 }) as f32;
        in_0.special_pressed = 0 as i32 as i32;
        GenoBhv_MultiDashStep(&raw mut m, &raw mut in_0, &raw mut p);
        if i == 6 as i32 {
            ratio0 = m.ay / m.ax;
        }
        if i > 6 as i32 && m.phase == GB_MD_DASH as i32 {
            let mut r: f32 = m.ay / m.ax;
            ok = (ok != 0 && gba_abs(r - ratio0) < 1.0e-6f32) as i32;
        }
        i += 1;
    }
    gba(b"M-3\0" as *const u8 as *const i8, ok, gba_i(ratio0));
    bad = 0 as i32;
    vmax = 0.0f32 as f32;
    ok = 1 as i32;
    let mut sticks: [[f32; 2]; 6] = [[0.; 2]; 6];
    sticks[0 as i32 as usize][0 as i32 as usize] = 0.0f32
        as f32;
    sticks[0 as i32 as usize][1 as i32 as usize] = 1.0f32
        as f32;
    sticks[1 as i32 as usize][0 as i32 as usize] = 0.3f32
        as f32;
    sticks[1 as i32 as usize][1 as i32 as usize] = 0.95f32
        as f32;
    sticks[2 as i32 as usize][0 as i32 as usize] = 0.8f32
        as f32;
    sticks[2 as i32 as usize][1 as i32 as usize] = 0.6f32
        as f32;
    sticks[3 as i32 as usize][0 as i32 as usize] = -0.7f32
        as f32;
    sticks[3 as i32 as usize][1 as i32 as usize] = -0.7f32
        as f32;
    sticks[4 as i32 as usize][0 as i32 as usize] = 0.0f32
        as f32;
    sticks[4 as i32 as usize][1 as i32 as usize] = -1.0f32
        as f32;
    sticks[5 as i32 as usize][0 as i32 as usize] = 0.1f32
        as f32;
    sticks[5 as i32 as usize][1 as i32 as usize] = 0.0f32
        as f32;
    i = 0 as i32;
    while i < 6 as i32 {
        GenoBhv_MultiDashBegin(&raw mut m, 1.0f32);
        dash = 0 as i32;
        while dash < 6 as i32 {
            in_0.sx = sticks[i as usize][0 as i32 as usize];
            in_0.sy = sticks[i as usize][1 as i32 as usize];
            in_0.special_pressed = 0 as i32 as i32;
            GenoBhv_MultiDashStep(&raw mut m, &raw mut in_0, &raw mut p);
            dash += 1;
        }
        len2 = m.ax * m.ax;
        dirx = m.ay * m.ay;
        diry = len2 + dirx;
        if gba_abs(diry - 1.0f32) > 1.0e-6f32
            && !(sticks[i as usize][0 as i32 as usize] == 0.1f32)
        {
            bad += 1;
        }
        if gba_abs(m.ay) > vmax {
            vmax = gba_abs(m.ay);
        }
        i += 1;
    }
    gba(
        b"M-9\0" as *const u8 as *const i8,
        (bad == 0 as i32) as i32,
        bad,
    );
    gba(
        b"M-6\0" as *const u8 as *const i8,
        gba_near(vmax, 0.9397f32, 3.0f32),
        gba_i(vmax),
    );
    return gba_rc;
}

pub unsafe extern "C" fn test_geno_bhv_anim_motion() -> i32 {
    melee_core::ct!("test_geno_bhv_anim_motion");
    let mut track: [f32; 22] = [0.; 22];
    let mut p: GbAnimMotionP = unsafe { core::mem::zeroed() };
    let mut a: GbAnimMotion = unsafe { core::mem::zeroed() };
    let mut b: GbAnimMotion = unsafe { core::mem::zeroed() };
    let mut x: f32 = 0.;
    let mut y: f32 = 0.;
    let mut mid: f32 = 0.;
    let mut tx: f32 = 0.;
    let mut ty: f32 = 0.;
    let mut expect: f32 = 0.;
    let mut i: i32 = 0;
    let mut n: i32 = 11 as i32;
    let mut frames: i32 = 0;
    let mut ok: i32 = 0;
    gba_rc = 0 as i32;
    i = 0 as i32;
    while i < n {
        track[(2 as i32 * i) as usize] = ((i * i) as f32
            * 0.5f32) as f32;
        track[(2 as i32 * i + 1 as i32) as usize] = (i
            as f32 * 1.25f32) as f32;
        i += 1;
    }
    GenoBhv_AnimMotionParams(&raw mut p);
    GenoBhv_AnimMotionBegin(&raw mut a, 0.0f32, 0.0f32);
    frames = 0 as i32;
    while a.ended == 0 && frames < 100 as i32 {
        GenoBhv_AnimMotionStep(
            &raw mut a,
            &raw mut track as *mut f32,
            n,
            &raw mut p,
            1.0f32,
            0 as i32,
        );
        frames += 1;
    }
    gba(
        b"A-1\0" as *const u8 as *const i8,
        (frames == n - 1 as i32) as i32,
        frames,
    );
    expect = track[(2 as i32 * (n - 1 as i32)) as usize]
        - track[0 as i32 as usize];
    gba(
        b"A-2\0" as *const u8 as *const i8,
        (gba_near(a.pos_x, expect, 3.0f32) != 0
            && gba_near(
                a.pos_y,
                track[(2 as i32 * (n - 1 as i32)
                    + 1 as i32) as usize]
                    - track[1 as i32 as usize],
                3.0f32,
            ) != 0) as i32,
        gba_i(a.pos_x),
    );
    OSReport!(
        b"GBACC A-2 note: relative error %d ppm\n\0" as *const u8
            as *const i8 as *mut i8,
        (gba_abs(a.pos_x - expect) as f32
            / expect as f32 * 1.0e6f32) as i32,
    );
    p.rate = 0.5f32 as f32;
    GenoBhv_AnimMotionBegin(&raw mut a, 0.0f32, 0.0f32);
    frames = 0 as i32;
    while a.ended == 0 && frames < 200 as i32 {
        GenoBhv_AnimMotionStep(
            &raw mut a,
            &raw mut track as *mut f32,
            n,
            &raw mut p,
            1.0f32,
            0 as i32,
        );
        frames += 1;
    }
    gba(
        b"A-3\0" as *const u8 as *const i8,
        (frames == 2 as i32 * (n - 1 as i32))
            as i32,
        frames,
    );
    p.rate = 2.0f32 as f32;
    GenoBhv_AnimMotionBegin(&raw mut a, 0.0f32, 0.0f32);
    frames = 0 as i32;
    while a.ended == 0 && frames < 200 as i32 {
        GenoBhv_AnimMotionStep(
            &raw mut a,
            &raw mut track as *mut f32,
            n,
            &raw mut p,
            1.0f32,
            0 as i32,
        );
        frames += 1;
    }
    gba(
        b"A-4\0" as *const u8 as *const i8,
        (frames
            == (n - 1 as i32 + 1 as i32)
                / 2 as i32) as i32,
        frames,
    );
    p.rate = 1.0f32 as f32;
    track[0 as i32 as usize] = 0.0f32 as f32;
    track[1 as i32 as usize] = 0.0f32 as f32;
    GenoBhv_AnimMotionBegin(&raw mut a, 3.0f32, 4.0f32);
    GenoBhv_AnimMotionBegin(&raw mut b, 3.0f32, 4.0f32);
    p.mode = GB_AM_DELTA as i32 as i32;
    i = 0 as i32;
    while i < n - 1 as i32 {
        GenoBhv_AnimMotionStep(
            &raw mut a,
            &raw mut track as *mut f32,
            n,
            &raw mut p,
            1.0f32,
            0 as i32,
        );
        i += 1;
    }
    p.mode = GB_AM_ABSOLUTE as i32 as i32;
    i = 0 as i32;
    while i < n - 1 as i32 {
        GenoBhv_AnimMotionStep(
            &raw mut b,
            &raw mut track as *mut f32,
            n,
            &raw mut p,
            1.0f32,
            0 as i32,
        );
        i += 1;
    }
    gba(
        b"A-5\0" as *const u8 as *const i8,
        (gba_near(a.pos_x, b.pos_x, 3.0f32) != 0
            && gba_near(a.pos_y, b.pos_y, 3.0f32) != 0) as i32,
        gba_i(b.pos_x),
    );
    p.mode = GB_AM_DELTA as i32 as i32;
    ok = 1 as i32;
    i = 0 as i32;
    while i < n - 1 as i32 {
        GenoBhv_AnimMotionSample(
            &raw mut track as *mut f32,
            n,
            i as f32 + 0.5f32,
            &raw mut x,
            &raw mut y,
        );
        tx = track[(2 as i32 * (i + 1 as i32)) as usize]
            - track[(2 as i32 * i) as usize];
        tx = (tx as f32 * 0.5f32) as f32;
        tx = track[(2 as i32 * i) as usize] + tx;
        ty = track[(2 as i32 * (i + 1 as i32)
            + 1 as i32) as usize]
            - track[(2 as i32 * i + 1 as i32) as usize];
        ty = (ty as f32 * 0.5f32) as f32;
        ty = track[(2 as i32 * i + 1 as i32) as usize]
            + ty;
        ok = (ok != 0 && x == tx && y == ty) as i32;
        i += 1;
    }
    gba(
        b"A-6\0" as *const u8 as *const i8,
        ok,
        0 as i32,
    );
    GenoBhv_AnimMotionBegin(&raw mut a, 0.0f32, 0.0f32);
    GenoBhv_AnimMotionBegin(&raw mut b, 0.0f32, 0.0f32);
    i = 0 as i32;
    while i < n - 1 as i32 {
        GenoBhv_AnimMotionStep(
            &raw mut a,
            &raw mut track as *mut f32,
            n,
            &raw mut p,
            1.0f32,
            0 as i32,
        );
        GenoBhv_AnimMotionStep(
            &raw mut b,
            &raw mut track as *mut f32,
            n,
            &raw mut p,
            -1.0f32,
            0 as i32,
        );
        i += 1;
    }
    gba(
        b"A-7\0" as *const u8 as *const i8,
        (b.pos_x == -a.pos_x && b.pos_y == a.pos_y) as i32,
        gba_i(b.pos_x),
    );
    GenoBhv_AnimMotionBegin(&raw mut a, 0.0f32, 0.0f32);
    GenoBhv_AnimMotionStep(
        &raw mut a,
        &raw mut track as *mut f32,
        n,
        &raw mut p,
        1.0f32,
        0 as i32,
    );
    GenoBhv_AnimMotionStep(
        &raw mut a,
        &raw mut track as *mut f32,
        n,
        &raw mut p,
        1.0f32,
        0 as i32,
    );
    x = a.pos_x;
    y = a.anim_t;
    GenoBhv_AnimMotionStep(
        &raw mut a,
        &raw mut track as *mut f32,
        n,
        &raw mut p,
        1.0f32,
        1 as i32,
    );
    ok = (a.pos_x == x && a.anim_t == y as f32 + 1.0f32
        && a.pos_y > 0.0f32) as i32;
    p.mode = GB_AM_ABSOLUTE as i32 as i32;
    GenoBhv_AnimMotionBegin(&raw mut b, 0.0f32, 0.0f32);
    GenoBhv_AnimMotionStep(
        &raw mut b,
        &raw mut track as *mut f32,
        n,
        &raw mut p,
        1.0f32,
        0 as i32,
    );
    GenoBhv_AnimMotionStep(
        &raw mut b,
        &raw mut track as *mut f32,
        n,
        &raw mut p,
        1.0f32,
        1 as i32,
    );
    x = b.pos_x;
    GenoBhv_AnimMotionStep(
        &raw mut b,
        &raw mut track as *mut f32,
        n,
        &raw mut p,
        1.0f32,
        0 as i32,
    );
    ok = (ok != 0 && b.pos_x > x && b.origin_x < 0.0f32) as i32;
    p.mode = GB_AM_DELTA as i32 as i32;
    gba(b"A-8\0" as *const u8 as *const i8, ok, gba_i(a.pos_x));
    p.r#loop = 1 as i32 as i32;
    p.rate = 3.0f32 as f32;
    GenoBhv_AnimMotionBegin(&raw mut a, 0.0f32, 0.0f32);
    ok = 1 as i32;
    i = 0 as i32;
    while i < 4 as i32 {
        GenoBhv_AnimMotionStep(
            &raw mut a,
            &raw mut track as *mut f32,
            n,
            &raw mut p,
            1.0f32,
            0 as i32,
        );
        i += 1;
    }
    ok = (a.anim_t == 2.0f32 && a.ended == 0) as i32;
    gba(b"A-9\0" as *const u8 as *const i8, ok, gba_i(a.anim_t));
    return gba_rc;
}

pub unsafe extern "C" fn gbt_row_tick(mut phys: i32) {
    melee_core::ct!("gbt_row_tick");
    let mut r: *mut MotionState = Geno_MotionRow(&raw mut t_fp, t_fp.motion_id);
    if !r.is_null() {
        if (*r).anim_cb.is_some() {
            (*r).anim_cb.expect("non-null function pointer")(&raw mut t_gobj);
        }
        if (*r).input_cb.is_some() {
            (*r).input_cb.expect("non-null function pointer")(&raw mut t_gobj);
        }
        if phys != 0 && (*r).phys_cb.is_some() {
            (*r).phys_cb.expect("non-null function pointer")(&raw mut t_gobj);
        }
    }
}

pub unsafe extern "C" fn gbt_setup(mut air: i32) -> *mut TestGenoState {
    melee_core::ct!("gbt_setup");
    let mut st: *mut TestGenoState = ::core::ptr::null_mut::<TestGenoState>();
    t_rows[ftCo_MS_Fall as i32 as usize].anim_id = 29
        as i32 as enum_t;
    if Geno_TestInstall(&raw const gbt_json as *const i8)
        != 1 as i32
        || {
            st = t_setup();
            st.is_null()
        }
    {
        TestFail(
            b"could not install the behaviours profile\0" as *const u8
                as *const i8,
        );
        Geno_TestRestore();
        return ::core::ptr::null_mut::<TestGenoState>();
    }
    GenoGame_TestCapture(1 as i32, 0 as i32);
    t_fp.ground_or_air = (if air != 0 {
        GA_Air as i32
    } else {
        GA_Ground as i32
    }) as GroundOrAir;
    t_fp.co_attrs.gravity = 0.1f32;
    t_fp.co_attrs.terminal_velocity = 2.0f32;
    t_fp.co_attrs.max_jumps = 2 as i32;
    t_fp.facing_dir = 1.0f32;
    return st;
}

pub unsafe extern "C" fn test_geno_bhv_adapters() -> i32 {
    melee_core::ct!("test_geno_bhv_adapters");
    let mut st: *mut TestGenoState = ::core::ptr::null_mut::<TestGenoState>();
    let mut i: i32 = 0;
    let mut c0: i32 = 0;
    let mut n: i32 = 0;
    let mut ok: i32 = 0;
    let mut y: f32 = 0.;
    gba_rc = 0 as i32;
    st = gbt_setup(1 as i32);
    if st.is_null() {
        return 1 as i32;
    }
    Geno_LabEnterState(&raw mut t_fp, 0 as i32);
    c0 = GenoGame_TestChanges();
    n = 0 as i32;
    while n < 200 as i32 && GenoGame_TestChanges() == c0 {
        t_fp.input.pressed_buttons = 0 as HSD_Pad;
        gbt_row_tick(1 as i32);
        n += 1;
    }
    gba(
        b"AD-tornado-frames\0" as *const u8 as *const i8,
        (n == 88 as i32) as i32,
        n,
    );
    Geno_LabEnterState(&raw mut t_fp, 0 as i32);
    c0 = GenoGame_TestChanges();
    y = 0.0f32 as f32;
    n = 0 as i32;
    while n < 300 as i32 && GenoGame_TestChanges() == c0 {
        t_fp.input.pressed_buttons = (if n >= 12 as i32
            && (n - 12 as i32) % 10 as i32
                == 0 as i32
        {
            HSD_PAD_B
        } else {
            0 as i32
        }) as HSD_Pad;
        gbt_row_tick(1 as i32);
        y += t_fp.self_vel.y;
        n += 1;
    }
    gba(
        b"AD-tornado-mashed-frames\0" as *const u8 as *const i8,
        (n == 139 as i32) as i32,
        n,
    );
    gba(
        b"AD-tornado-mashed-rise\0" as *const u8 as *const i8,
        (y > 0.0f32) as i32,
        gba_i(y),
    );
    gba(
        b"AD-tornado-helpless\0" as *const u8 as *const i8,
        (GenoGame_TestLastTarget()
            == (0 as u32 & 15 as u32)
                << 28 as i32
                | ftCo_MS_FallSpecial as i32 as u32
                    & 0xffff as u32) as i32,
        0 as i32,
    );
    Geno_TestRestore();
    st = gbt_setup(1 as i32);
    if st.is_null() {
        return 1 as i32;
    }
    Geno_LabEnterState(&raw mut t_fp, 1 as i32);
    c0 = GenoGame_TestChanges();
    ok = 1 as i32;
    n = 0 as i32;
    while n < 200 as i32 && GenoGame_TestChanges() == c0 {
        t_fp.input.pressed_buttons = 0 as HSD_Pad;
        gbt_row_tick(1 as i32);
        if n == 30 as i32 {
            ok = (ok != 0 && t_fp.self_vel.x > 1.8f32 && t_fp.self_vel.y < -0.5f32)
                as i32;
        }
        if n == 66 as i32 {
            ok = (ok != 0 && t_fp.self_vel.y == 1.0f32) as i32;
        }
        n += 1;
    }
    gba(
        b"AD-drill-frames\0" as *const u8 as *const i8,
        (n == 96 as i32) as i32,
        n,
    );
    gba(
        b"AD-drill-velocity\0" as *const u8 as *const i8,
        ok,
        gba_i(t_fp.self_vel.y),
    );
    gba(
        b"AD-drill-helpless\0" as *const u8 as *const i8,
        (GenoGame_TestLastTarget()
            == (0 as u32 & 15 as u32)
                << 28 as i32
                | ftCo_MS_FallSpecial as i32 as u32
                    & 0xffff as u32) as i32,
        0 as i32,
    );
    Geno_TestRestore();
    st = gbt_setup(1 as i32);
    if st.is_null() {
        return 1 as i32;
    }
    Geno_LabEnterState(&raw mut t_fp, 2 as i32);
    c0 = GenoGame_TestChanges();
    ok = 1 as i32;
    t_fp.self_vel.x = 1.0f32 as f32;
    t_fp.self_vel.y = 0.0f32 as f32;
    t_fp.input.lstick[0 as i32 as usize].x = -1.0f32 as f32;
    n = 0 as i32;
    while n < 200 as i32 && GenoGame_TestChanges() == c0 {
        gbt_row_tick(0 as i32);
        if n == 10 as i32 {
            ok = (ok != 0 && t_fp.reflecting() as i32 != 0)
                as i32;
        }
        if n == 30 as i32 {
            ok = (ok != 0 && t_fp.reflecting() == 0) as i32;
        }
        n += 1;
    }
    gba(
        b"AD-cape-frames\0" as *const u8 as *const i8,
        (n == 40 as i32 && ok != 0) as i32,
        n,
    );
    gba(
        b"AD-cape-flip\0" as *const u8 as *const i8,
        (t_fp.facing_dir == -1.0f32) as i32,
        gba_i(t_fp.facing_dir as f32),
    );
    gba(
        b"AD-cape-not-helpless\0" as *const u8 as *const i8,
        (GenoGame_TestLastTarget()
            == (0 as u32 & 15 as u32)
                << 28 as i32
                | ftCo_MS_Fall as i32 as u32
                    & 0xffff as u32) as i32,
        0 as i32,
    );
    Geno_TestRestore();
    st = gbt_setup(1 as i32);
    if st.is_null() {
        return 1 as i32;
    }
    t_fp.motion_id = ftCo_MS_JumpAerialF as i32 as FtMotionId;
    t_fp.x1968_jumpsUsed = 2 as u8;
    Geno_OnActionChange(&raw mut t_gobj);
    t_fp.self_vel.x = 0.8f32 as f32;
    t_fp.self_vel.y = 0.1f32 as f32;
    t_fp.input.held_buttons[0 as i32 as usize] = HSD_PAD_X as HSD_Pad;
    ok = 0 as i32;
    n = 0 as i32;
    while n < 12 as i32 {
        if Geno_PreAnim(&raw mut t_gobj) == 1 as i32 {
            ok = 1 as i32;
            break;
        } else {
            n += 1;
        }
    }
    gba(
        b"AD-glide-entry-hold\0" as *const u8 as *const i8,
        (ok != 0 && t_fp.motion_id == GENO_MOTION_BASE + 3 as i32
            && n + 1 as i32 >= 6 as i32
            && n + 1 as i32 <= 8 as i32)
            as i32,
        n + 1 as i32,
    );
    n = 0 as i32;
    while n < 40 as i32
        && t_fp.motion_id == GENO_MOTION_BASE + 3 as i32
    {
        gbt_row_tick(1 as i32);
        n += 1;
    }
    gba(
        b"AD-glide-start-12\0" as *const u8 as *const i8,
        (n == 12 as i32
            && t_fp.motion_id == GENO_MOTION_BASE + 4 as i32)
            as i32,
        n,
    );
    i = 0 as i32;
    while i < 20 as i32 {
        gbt_row_tick(1 as i32);
        i += 1;
    }
    gba(
        b"AD-glide-fly-velocity\0" as *const u8 as *const i8,
        (t_fp.self_vel.x > 1.0f32 && t_fp.self_vel.y < 0.0f32) as i32,
        gba_i(t_fp.self_vel.x),
    );
    t_fp.input.pressed_buttons = HSD_PAD_A as HSD_Pad;
    gbt_row_tick(1 as i32);
    t_fp.input.pressed_buttons = 0 as HSD_Pad;
    gba(
        b"AD-glide-attack\0" as *const u8 as *const i8,
        (t_fp.motion_id == GENO_MOTION_BASE + 5 as i32)
            as i32,
        t_fp.motion_id - GENO_MOTION_BASE,
    );
    Geno_TestRestore();
    st = gbt_setup(1 as i32);
    if st.is_null() {
        return 1 as i32;
    }
    Geno_LabEnterState(&raw mut t_fp, 7 as i32);
    c0 = GenoGame_TestChanges();
    ok = 1 as i32;
    t_fp.input.lstick[0 as i32 as usize].x = 1.0f32 as f32;
    t_fp.input.lstick[0 as i32 as usize].y = 0.0f32 as f32;
    n = 0 as i32;
    while n < 200 as i32 && GenoGame_TestChanges() == c0 {
        t_fp.input.pressed_buttons = 0 as HSD_Pad;
        gbt_row_tick(1 as i32);
        if n == 10 as i32 {
            ok = (ok != 0 && t_fp.self_vel.x == 3.2f32 && t_fp.self_vel.y == 0.0f32)
                as i32;
        }
        n += 1;
    }
    gba(
        b"AD-multi-dash-frames\0" as *const u8 as *const i8,
        (n == 39 as i32 && ok != 0) as i32,
        n,
    );
    gba(
        b"AD-multi-dash-not-helpless\0" as *const u8 as *const i8,
        (GenoGame_TestLastTarget()
            == (0 as u32 & 15 as u32)
                << 28 as i32
                | ftCo_MS_Fall as i32 as u32
                    & 0xffff as u32) as i32,
        0 as i32,
    );
    Geno_LabEnterState(&raw mut t_fp, 7 as i32);
    c0 = GenoGame_TestChanges();
    n = 0 as i32;
    while n < 300 as i32 && GenoGame_TestChanges() == c0 {
        t_fp.input.pressed_buttons = HSD_PAD_B as HSD_Pad;
        gbt_row_tick(1 as i32);
        n += 1;
    }
    t_fp.input.pressed_buttons = 0 as HSD_Pad;
    gba(
        b"AD-multi-dash-chain-frames\0" as *const u8 as *const i8,
        (n == 90 as i32) as i32,
        n,
    );
    gba(
        b"AD-multi-dash-helpless\0" as *const u8 as *const i8,
        (GenoGame_TestLastTarget()
            == (0 as u32 & 15 as u32)
                << 28 as i32
                | ftCo_MS_FallSpecial as i32 as u32
                    & 0xffff as u32) as i32,
        0 as i32,
    );
    Geno_TestRestore();
    let mut dx: f32 = 0.;
    let mut dy: f32 = 0.;
    GenoBhv_AnimMotionFrameDelta(2.0f32, 1.0f32, -1.0f32, &raw mut dx, &raw mut dy);
    ok = (dx == -2.0f32 && dy == 1.0f32) as i32;
    GenoBhv_AnimMotionFrameDelta(0.75f32, -0.5f32, 1.0f32, &raw mut dx, &raw mut dy);
    ok = (ok != 0 && dx == 0.75f32 && dy == -0.5f32) as i32;
    gba(
        b"AD-anim-motion-delta\0" as *const u8 as *const i8,
        ok,
        gba_i(dx),
    );
    return gba_rc;
}

pub unsafe extern "C" fn test_geno_none_independence() -> i32 {
    melee_core::ct!("test_geno_none_independence");
    static mut a: ftData = ftData {
        x0: ::core::ptr::null::<ftCo_DatAttrs>() as *mut ftCo_DatAttrs,
        ext_attr: ::core::ptr::null::<c_void>() as *mut c_void,
        x8: ::core::ptr::null::<ftData_x8>() as *mut ftData_x8,
        xC: ::core::ptr::null::<Fighter_WaitAnimData>() as *mut Fighter_WaitAnimData,
        x10: ::core::ptr::null::<[u8; 2]>() as *mut [u8; 2],
        x14: ::core::ptr::null::<Fighter_WaitAnimData>() as *mut Fighter_WaitAnimData,
        x18: ::core::ptr::null::<[u8; 2]>() as *mut [u8; 2],
        x1C: ::core::ptr::null::<*mut ftData_x1C>() as *mut *mut ftData_x1C,
        x20: ::core::ptr::null::<ftData_x20>() as *mut ftData_x20,
        x24: ::core::ptr::null_mut(),
        x28: ::core::ptr::null::<WaitStruct>() as *mut WaitStruct,
        x2C: ::core::ptr::null::<ftDynamics>() as *mut ftDynamics,
        x30: ::core::ptr::null::<ftData_x30>() as *mut ftData_x30,
        x34: ::core::ptr::null::<ftData_x34>() as *mut ftData_x34,
        x38: ::core::ptr::null::<ftData_x38>() as *mut ftData_x38,
        x3C: ::core::ptr::null::<UnkFloat6_Camera>() as *mut UnkFloat6_Camera,
        x40: ::core::ptr::null::<itPickup>() as *mut itPickup,
        x44: ::core::ptr::null::<ftData_x44_t>() as *mut ftData_x44_t,
        x48_items: ::core::ptr::null::<*mut c_void>()
            as *mut *mut c_void,
        x4C_sfx: ::core::ptr::null::<FtSFX>() as *mut FtSFX,
        x50: ::core::ptr::null::<Vec2>() as *mut Vec2,
        x54: ::core::ptr::null::<i32>() as *mut i32,
        x58: ::core::ptr::null::<ftData_x58_t>() as *mut ftData_x58_t,
        x5C: ::core::ptr::null::<HSD_Joint>() as *mut HSD_Joint,
    };
    static mut b: ftData = ftData {
        x0: ::core::ptr::null::<ftCo_DatAttrs>() as *mut ftCo_DatAttrs,
        ext_attr: ::core::ptr::null::<c_void>() as *mut c_void,
        x8: ::core::ptr::null::<ftData_x8>() as *mut ftData_x8,
        xC: ::core::ptr::null::<Fighter_WaitAnimData>() as *mut Fighter_WaitAnimData,
        x10: ::core::ptr::null::<[u8; 2]>() as *mut [u8; 2],
        x14: ::core::ptr::null::<Fighter_WaitAnimData>() as *mut Fighter_WaitAnimData,
        x18: ::core::ptr::null::<[u8; 2]>() as *mut [u8; 2],
        x1C: ::core::ptr::null::<*mut ftData_x1C>() as *mut *mut ftData_x1C,
        x20: ::core::ptr::null::<ftData_x20>() as *mut ftData_x20,
        x24: ::core::ptr::null_mut(),
        x28: ::core::ptr::null::<WaitStruct>() as *mut WaitStruct,
        x2C: ::core::ptr::null::<ftDynamics>() as *mut ftDynamics,
        x30: ::core::ptr::null::<ftData_x30>() as *mut ftData_x30,
        x34: ::core::ptr::null::<ftData_x34>() as *mut ftData_x34,
        x38: ::core::ptr::null::<ftData_x38>() as *mut ftData_x38,
        x3C: ::core::ptr::null::<UnkFloat6_Camera>() as *mut UnkFloat6_Camera,
        x40: ::core::ptr::null::<itPickup>() as *mut itPickup,
        x44: ::core::ptr::null::<ftData_x44_t>() as *mut ftData_x44_t,
        x48_items: ::core::ptr::null::<*mut c_void>()
            as *mut *mut c_void,
        x4C_sfx: ::core::ptr::null::<FtSFX>() as *mut FtSFX,
        x50: ::core::ptr::null::<Vec2>() as *mut Vec2,
        x54: ::core::ptr::null::<i32>() as *mut i32,
        x58: ::core::ptr::null::<ftData_x58_t>() as *mut ftData_x58_t,
        x5C: ::core::ptr::null::<HSD_Joint>() as *mut HSD_Joint,
    };
    static mut marker: [i32; 4] = [0; 4];
    let mut m: u32 = 0;
    let mut rc: i32 = 0 as i32;
    memset(
        &raw mut a as *mut c_void,
        0 as i32,
        ::core::mem::size_of::<ftData>() as usize,
    );
    memset(
        &raw mut b as *mut c_void,
        0 as i32,
        ::core::mem::size_of::<ftData>() as usize,
    );
    if GenoDefine_FtDataSharedMask(&raw mut a, &raw mut b) != 0 as u32 {
        TestFail(
            b"census: empty structs share nothing (NULL is not shared)\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    a.x20 = (&raw mut marker as *mut i32)
        .offset(0 as i32 as isize) as *mut i32
        as *mut ftData_x20;
    b.x20 = (&raw mut marker as *mut i32)
        .offset(0 as i32 as isize) as *mut i32
        as *mut ftData_x20;
    a.x3C = (&raw mut marker as *mut i32)
        .offset(1 as i32 as isize) as *mut i32
        as *mut UnkFloat6_Camera;
    b.x3C = (&raw mut marker as *mut i32)
        .offset(2 as i32 as isize) as *mut i32
        as *mut UnkFloat6_Camera;
    a.x5C = (&raw mut marker as *mut i32)
        .offset(3 as i32 as isize) as *mut i32
        as *mut HSD_Joint as *mut HSD_Joint;
    b.x5C = (&raw mut marker as *mut i32)
        .offset(3 as i32 as isize) as *mut i32
        as *mut HSD_Joint as *mut HSD_Joint;
    m = GenoDefine_FtDataSharedMask(&raw mut a, &raw mut b);
    if m
        != (1 as u32) << 8 as i32
            | (1 as u32) << 23 as i32
    {
        TestFail(
            b"census: exactly x20 and x5C are the shared pointers\0" as *const u8
                as *const i8,
        );
        rc = 1 as i32;
    }
    if strcmp(
        GenoDefine_FtDataFieldName(8 as i32),
        b"x20\0" as *const u8 as *const i8,
    ) != 0 as i32
        || strcmp(
            GenoDefine_FtDataFieldName(23 as i32),
            b"x5C\0" as *const u8 as *const i8,
        ) != 0 as i32
    {
        TestFail(b"census: field names\0" as *const u8 as *const i8);
        rc = 1 as i32;
    }
    return rc;
}

pub unsafe extern "C" fn GenoTestRegisterAll() {
    melee_core::ct!("GenoTestRegisterAll");
    TestRegister(
        b"geno_none_guard_contact\0" as *const u8 as *const i8,
        Some(
            test_geno_none_guard_contact as unsafe extern "C" fn() -> i32,
        ),
    );
    TestRegister(
        b"geno_lua_charge\0" as *const u8 as *const i8,
        Some(test_geno_lua_charge as unsafe extern "C" fn() -> i32),
    );
    TestRegister(
        b"geno_combo_parse\0" as *const u8 as *const i8,
        Some(test_geno_combo_parse as unsafe extern "C" fn() -> i32),
    );
    TestRegister(
        b"geno_combo_routes\0" as *const u8 as *const i8,
        Some(test_geno_combo_routes as unsafe extern "C" fn() -> i32),
    );
    TestRegister(
        b"geno_combo_digest\0" as *const u8 as *const i8,
        Some(test_geno_combo_digest as unsafe extern "C" fn() -> i32),
    );
    TestRegister(
        b"geno_lua_fault\0" as *const u8 as *const i8,
        Some(test_geno_lua_fault as unsafe extern "C" fn() -> i32),
    );
    TestRegister(
        b"geno_lua_digest\0" as *const u8 as *const i8,
        Some(test_geno_lua_digest as unsafe extern "C" fn() -> i32),
    );
    TestRegister(
        b"geno_digest_coverage\0" as *const u8 as *const i8,
        Some(test_geno_digest_coverage as unsafe extern "C" fn() -> i32),
    );
    TestRegister(
        b"geno_lua_counter\0" as *const u8 as *const i8,
        Some(test_geno_lua_counter as unsafe extern "C" fn() -> i32),
    );
    TestRegister(
        b"geno_define_snapshot\0" as *const u8 as *const i8,
        Some(test_geno_define_snapshot as unsafe extern "C" fn() -> i32),
    );
    TestRegister(
        b"geno_none_independence\0" as *const u8 as *const i8,
        Some(test_geno_none_independence as unsafe extern "C" fn() -> i32),
    );
    TestRegister(
        b"geno_define_hit_tag\0" as *const u8 as *const i8,
        Some(test_geno_define_hit_tag as unsafe extern "C" fn() -> i32),
    );
    TestRegister(
        b"geno_define_state_hash\0" as *const u8 as *const i8,
        Some(test_geno_define_state_hash as unsafe extern "C" fn() -> i32),
    );
    TestRegister(
        b"geno_define_article_digest\0" as *const u8 as *const i8,
        Some(
            test_geno_define_article_digest
                as unsafe extern "C" fn() -> i32,
        ),
    );
    TestRegister(
        b"rb_game_hash\0" as *const u8 as *const i8,
        Some(test_rb_game_hash as unsafe extern "C" fn() -> i32),
    );
    TestRegister(
        b"geno_fly\0" as *const u8 as *const i8,
        Some(test_geno_fly as unsafe extern "C" fn() -> i32),
    );
    TestRegister(
        b"geno_ftcmd_escape\0" as *const u8 as *const i8,
        Some(test_geno_ftcmd_escape as unsafe extern "C" fn() -> i32),
    );
    TestRegister(
        b"geno_ftcmd_loops\0" as *const u8 as *const i8,
        Some(test_geno_ftcmd_loops as unsafe extern "C" fn() -> i32),
    );
    TestRegister(
        b"geno_vanilla_script_untouched\0" as *const u8 as *const i8,
        Some(
            test_geno_vanilla_script_untouched
                as unsafe extern "C" fn() -> i32,
        ),
    );
    TestRegister(
        b"geno_state_resets\0" as *const u8 as *const i8,
        Some(test_geno_state_resets as unsafe extern "C" fn() -> i32),
    );
    TestRegister(
        b"geno_attr_table\0" as *const u8 as *const i8,
        Some(test_geno_attr_table as unsafe extern "C" fn() -> i32),
    );
    TestRegister(
        b"geno_multijump\0" as *const u8 as *const i8,
        Some(test_geno_multijump as unsafe extern "C" fn() -> i32),
    );
    TestRegister(
        b"geno_multijump_script_gate\0" as *const u8 as *const i8,
        Some(
            test_geno_multijump_script_gate
                as unsafe extern "C" fn() -> i32,
        ),
    );
    TestRegister(
        b"geno_ordinary_jumps\0" as *const u8 as *const i8,
        Some(test_geno_ordinary_jumps as unsafe extern "C" fn() -> i32),
    );
    TestRegister(
        b"geno_jump_limits\0" as *const u8 as *const i8,
        Some(test_geno_jump_limits as unsafe extern "C" fn() -> i32),
    );
    TestRegister(
        b"geno_reload_state_callbacks\0" as *const u8 as *const i8,
        Some(
            test_geno_reload_state_callbacks
                as unsafe extern "C" fn() -> i32,
        ),
    );
    TestRegister(
        b"geno_effective_script_range\0" as *const u8 as *const i8,
        Some(
            test_geno_effective_script_range
                as unsafe extern "C" fn() -> i32,
        ),
    );
    TestRegister(
        b"geno_iasa_script_validation\0" as *const u8 as *const i8,
        Some(
            test_geno_iasa_script_validation
                as unsafe extern "C" fn() -> i32,
        ),
    );
    TestRegister(
        b"geno_state_savestate\0" as *const u8 as *const i8,
        Some(test_geno_state_savestate as unsafe extern "C" fn() -> i32),
    );
    TestRegister(
        b"geno_v1_values\0" as *const u8 as *const i8,
        Some(test_geno_v1_values as unsafe extern "C" fn() -> i32),
    );
    TestRegister(
        b"geno_v1_change_action\0" as *const u8 as *const i8,
        Some(test_geno_v1_change_action as unsafe extern "C" fn() -> i32),
    );
    TestRegister(
        b"geno_v1_ground_edge\0" as *const u8 as *const i8,
        Some(test_geno_v1_ground_edge as unsafe extern "C" fn() -> i32),
    );
    TestRegister(
        b"geno_v1_rehit\0" as *const u8 as *const i8,
        Some(test_geno_v1_rehit as unsafe extern "C" fn() -> i32),
    );
    TestRegister(
        b"geno_v1_autolink\0" as *const u8 as *const i8,
        Some(test_geno_v1_autolink as unsafe extern "C" fn() -> i32),
    );
    TestRegister(
        b"geno_v1_special_attrs\0" as *const u8 as *const i8,
        Some(test_geno_v1_special_attrs as unsafe extern "C" fn() -> i32),
    );
    TestRegister(
        b"geno_v1_overlay\0" as *const u8 as *const i8,
        Some(test_geno_v1_overlay as unsafe extern "C" fn() -> i32),
    );
    TestRegister(
        b"geno_v1_inert\0" as *const u8 as *const i8,
        Some(test_geno_v1_inert as unsafe extern "C" fn() -> i32),
    );
    TestRegister(
        b"geno_v2_states\0" as *const u8 as *const i8,
        Some(test_geno_v2_states as unsafe extern "C" fn() -> i32),
    );
    TestRegister(
        b"geno_v2_change_to_state\0" as *const u8 as *const i8,
        Some(
            test_geno_v2_change_to_state as unsafe extern "C" fn() -> i32,
        ),
    );
    TestRegister(
        b"geno_v3_anim_motion\0" as *const u8 as *const i8,
        Some(test_geno_v3_anim_motion as unsafe extern "C" fn() -> i32),
    );
    TestRegister(
        b"geno_v3_many_states\0" as *const u8 as *const i8,
        Some(test_geno_v3_many_states as unsafe extern "C" fn() -> i32),
    );
    TestRegister(
        b"geno_lab_stale_fighter\0" as *const u8 as *const i8,
        Some(test_geno_lab_stale_fighter as unsafe extern "C" fn() -> i32),
    );
    TestRegister(
        b"geno_player_team_read\0" as *const u8 as *const i8,
        Some(test_geno_player_team_read as unsafe extern "C" fn() -> i32),
    );
    TestRegister(
        b"geno_lab_mode_table\0" as *const u8 as *const i8,
        Some(test_geno_lab_mode_table as unsafe extern "C" fn() -> i32),
    );
    TestRegister(
        b"geno_lab_rules\0" as *const u8 as *const i8,
        Some(test_geno_lab_rules as unsafe extern "C" fn() -> i32),
    );
    TestRegister(
        b"scene_item_frequency_rules\0" as *const u8 as *const i8,
        Some(
            test_scene_item_frequency_rules
                as unsafe extern "C" fn() -> i32,
        ),
    );
    TestRegister(
        b"geno_lab_scene\0" as *const u8 as *const i8,
        Some(test_geno_lab_scene as unsafe extern "C" fn() -> i32),
    );
    TestRegister(
        b"geno_lab_select_flow\0" as *const u8 as *const i8,
        Some(test_geno_lab_select_flow as unsafe extern "C" fn() -> i32),
    );
    TestRegister(
        b"geno_lab_mismatch_fields\0" as *const u8 as *const i8,
        Some(
            test_geno_lab_mismatch_fields as unsafe extern "C" fn() -> i32,
        ),
    );
    TestRegister(
        b"geno_v5_registry\0" as *const u8 as *const i8,
        Some(test_geno_v5_registry as unsafe extern "C" fn() -> i32),
    );
    TestRegister(
        b"geno_v5_article_motion\0" as *const u8 as *const i8,
        Some(test_geno_v5_article_motion as unsafe extern "C" fn() -> i32),
    );
    TestRegister(
        b"geno_v5_article_reflection\0" as *const u8 as *const i8,
        Some(
            test_geno_v5_article_reflection
                as unsafe extern "C" fn() -> i32,
        ),
    );
    TestRegister(
        b"geno_v5_on_hit\0" as *const u8 as *const i8,
        Some(test_geno_v5_on_hit as unsafe extern "C" fn() -> i32),
    );
    TestRegister(
        b"geno_v51\0" as *const u8 as *const i8,
        Some(test_geno_v51 as unsafe extern "C" fn() -> i32),
    );
    TestRegister(
        b"geno_v52_lockon\0" as *const u8 as *const i8,
        Some(test_geno_v52_lockon as unsafe extern "C" fn() -> i32),
    );
    TestRegister(
        b"geno_v54_retired\0" as *const u8 as *const i8,
        Some(test_geno_v54_retired as unsafe extern "C" fn() -> i32),
    );
    TestRegister(
        b"geno_bhv_trig\0" as *const u8 as *const i8,
        Some(test_geno_bhv_trig as unsafe extern "C" fn() -> i32),
    );
    TestRegister(
        b"geno_bhv_glide\0" as *const u8 as *const i8,
        Some(test_geno_bhv_glide as unsafe extern "C" fn() -> i32),
    );
    TestRegister(
        b"geno_bhv_tornado\0" as *const u8 as *const i8,
        Some(test_geno_bhv_tornado as unsafe extern "C" fn() -> i32),
    );
    TestRegister(
        b"geno_bhv_drill\0" as *const u8 as *const i8,
        Some(test_geno_bhv_drill as unsafe extern "C" fn() -> i32),
    );
    TestRegister(
        b"geno_bhv_cape\0" as *const u8 as *const i8,
        Some(test_geno_bhv_cape as unsafe extern "C" fn() -> i32),
    );
    TestRegister(
        b"geno_bhv_multi_dash\0" as *const u8 as *const i8,
        Some(test_geno_bhv_multi_dash as unsafe extern "C" fn() -> i32),
    );
    TestRegister(
        b"geno_bhv_anim_motion\0" as *const u8 as *const i8,
        Some(test_geno_bhv_anim_motion as unsafe extern "C" fn() -> i32),
    );
    TestRegister(
        b"geno_bhv_adapters\0" as *const u8 as *const i8,
        Some(test_geno_bhv_adapters as unsafe extern "C" fn() -> i32),
    );
}


