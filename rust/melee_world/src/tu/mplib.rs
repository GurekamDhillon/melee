//! Port of src/melee/mp/mplib.c: the map collision library (the ground collision vertex / line /
//! joint tables, line walking and the floor / ceiling / wall segment queries, joint list editing,
//! bounding checks and the debug drawing). Generated from the C by `gen/mtu/c2rs.py` (libclang AST:
//! every implicit conversion is the C one, `-ffp-contract=on` multiply-adds become `fmadds` /
//! `mul_add`, checked against `tools/rust-port/fma_sites.py`), with the goto-state-machine functions
//! that the tool cannot structure in `gen/mtu/hand/`. `bool` is MSL `int`. Callees of unported TUs
//! are `ext` hooks named like the C (unset: panics "TODO"); the TARGET_PC script hooks
//! (`ScriptGame_*`) are ext hooks too.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, dead_code, unused_assignments, unused_variables, unused_mut, unused_parens, unused_labels, unused_comparisons, clippy::all)]

use core::ffi::{c_char, c_void};
use core::ptr::null_mut;

use melee_core::fp::fmadds;
use melee_types::*;

/// `LINEID_CHECK` (hand ports): `HSD_ASSERTREPORT(line, 0, "%s:%d:not found lineID=%d")`
unsafe fn lineid_check(line: u32, line_id: i32) {
    if line_id == -1 || line_id >= (*mpLib_804D64B4).line_count {
        panic!("src/melee/mp/mplib.c:{}:not found lineID={}", line, line_id);
    }
}

/// `PSMTXMultVec` (dolphin/mtx; ported in melee_runtime::mtx): `*dst = m * *src`
unsafe fn PSMTXMultVec(m: *mut [f32; 4], src: *mut Vec, dst: *mut Vec) {
    let r = melee_runtime::mtx::mult_vec(
        core::slice::from_raw_parts(m as *const [f32; 4], 3),
        &melee_core::vec::Vec3 { x: (*src).x, y: (*src).y, z: (*src).z },
    );
    (*dst).x = r.x;
    (*dst).y = r.y;
    (*dst).z = r.z;
}

pub mod ext {
    #![allow(non_upper_case_globals, non_snake_case, unused_imports, dead_code)]
    use super::*;
    macro_rules! ext_none {
        ($name:ident, ($($a:ident),*)) => { panic!(concat!("TODO: ", stringify!($name), " is not ported")) };
        ($name:ident, ($($a:ident),*), $def:expr) => { ($def)($($a),*) };
    }
    macro_rules! ext_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)? $(= $def:expr)?;)*) => {
            pub mod hook {
                use super::*;
                $( pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } {
                    Some(f) => f($($a),*),
                    None => ext_none!($name, ($($a),*) $(, $def)?),
                }
            } )*
        };
    }
    ext_fns! {
    fn Camera_8002928C(a0: *mut CmSubject) -> i32;
    fn Camera_80030B50() -> i32 = || crate::cm::camera::world().camera_80030b50() as i32;
    fn Camera_80030B7C() -> i32 = || crate::cm::camera::world().camera_80030b7c() as i32;
    fn GObj_Create(a0: u16, a1: u8, a2: u8) -> *mut HSD_GObj = |a0: u16, a1: u8, a2: u8| unsafe { melee_runtime::tu::gobj_native::GObj_Create(a0 as _, a1 as _, a2 as _) as _ };
    fn GXBegin(a0: GXPrimitive, a1: GXVtxFmt, a2: u16);
    fn GXClearVtxDesc();
    fn GXEnd() = || { unsafe { let _ = melee_runtime::tu::sobjlib::GXEnd(); } };
    fn GXLoadPosMtxImm(a0: *mut [f32; 4], a1: u32);
    fn GXPosition3f32(a0: f32, a1: f32, a2: f32);
    fn GXSetCullMode(a0: GXCullMode);
    fn GXSetCurrentMtx(a0: u32);
    fn GXSetLineWidth(a0: u8, a1: GXTexOffset);
    fn GXSetVtxAttrFmt(a0: GXVtxFmt, a1: GXAttr, a2: GXCompCnt, a3: GXCompType, a4: u8);
    fn GXSetVtxDesc(a0: GXAttr, a1: GXAttrType);
    fn GXSetZMode(a0: u8, a1: GXCompare, a2: u8);
    fn Ground_801C0498() -> f32 = || unsafe { crate::tu::ground::Ground_801C0498() as _ };
    fn Ground_801C2D24(a0: i32, a1: *mut Vec) -> i32 = |a0: i32, a1: *mut Vec| unsafe { crate::tu::ground::Ground_801C2D24(a0 as _, a1 as _) as _ };
    fn HSD_CObjGetCurrent() -> *mut HSD_CObj = || unsafe { melee_runtime::tu::cobj::HSD_CObjGetCurrent() as _ };
    fn HSD_CObjGetViewingMtx(a0: *mut HSD_CObj, a1: *mut [f32; 4]);
    fn HSD_GObj_SetupProc(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: u8) -> *mut HSD_GObjProc = |a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: u8| unsafe { melee_runtime::tu::gobj_native::HSD_GObj_SetupProc(a0 as _, a1 as _, a2 as _) as _ };
    fn HSD_JObjGetFlags(a0: *mut HSD_JObj) -> u32 = |a0: *mut HSD_JObj| unsafe { melee_runtime::tu::jobj_world::HSD_JObjGetFlags(a0 as _) as _ };
    fn HSD_JObjSetupMatrixSub(a0: *mut HSD_JObj);
    fn HSD_LObjSetupInit(a0: *mut HSD_CObj) = |a0: *mut HSD_CObj| { unsafe { let _ = melee_runtime::tu::lobj::HSD_LObjSetupInit(a0 as _); } };
    fn HSD_MemAlloc(a0: i32) -> *mut c_void = |a0: i32| unsafe { melee_runtime::tu::memory::HSD_MemAlloc(a0 as _) as _ };
    fn HSD_SetTevRegAll();
    fn HSD_SetupChannel(a0: *mut HSD_Chan);
    fn HSD_SetupPEMode(a0: u32, a1: *mut HSD_PEDesc);
    fn HSD_SetupTevStage(a0: *mut _HSD_TevDesc);
    fn HSD_StateAssignTev() -> i32;
    fn HSD_StateInitTev();
    fn HSD_StateInvalidate(a0: i32);
    fn HSD_StateSetCullMode(a0: i32);
    fn HSD_StateSetNumChans(a0: i32);
    fn HSD_StateSetNumTevStages();
    fn HSD_StateSetNumTexGens();
    fn Mex_GrLineTypeRow(a0: i32) -> i32;
    fn ScriptGame_StageJoint(a0: i32) -> i32;
    fn ScriptGame_StageJointActive(a0: i32) -> i32;
    fn ScriptGame_StageJointCapacity() -> i32;
    fn ScriptGame_StageLineActive(a0: i32) -> i32;
    fn ScriptGame_StageSlotJointUpdate(a0: i32) -> i32;
    fn Stage_GetBlastZoneBottomOffset() -> f32 = || unsafe { crate::tu::stage::Stage_GetBlastZoneBottomOffset() as _ };
    fn Stage_GetBlastZoneLeftOffset() -> f32 = || unsafe { crate::tu::stage::Stage_GetBlastZoneLeftOffset() as _ };
    fn Stage_GetBlastZoneRightOffset() -> f32 = || unsafe { crate::tu::stage::Stage_GetBlastZoneRightOffset() as _ };
    fn Stage_GetBlastZoneTopOffset() -> f32 = || unsafe { crate::tu::stage::Stage_GetBlastZoneTopOffset() as _ };
    fn Stage_GetCamBoundsBottomOffset() -> f32 = || unsafe { crate::tu::stage::Stage_GetCamBoundsBottomOffset() as _ };
    fn Stage_GetCamBoundsLeftOffset() -> f32 = || unsafe { crate::tu::stage::Stage_GetCamBoundsLeftOffset() as _ };
    fn Stage_GetCamBoundsRightOffset() -> f32 = || unsafe { crate::tu::stage::Stage_GetCamBoundsRightOffset() as _ };
    fn Stage_GetCamBoundsTopOffset() -> f32 = || unsafe { crate::tu::stage::Stage_GetCamBoundsTopOffset() as _ };
    fn ftLib_80086984(a0: *mut HSD_GObj) -> *mut CollData = |a0: *mut HSD_GObj| unsafe { crate::ftlib::ftLib_80086984(a0 as _) as _ };
    fn grDynamicAttr_801CA0B4() = || { unsafe { let _ = crate::tu::grdynamicattr::grDynamicAttr_801CA0B4(); } };
    fn grDynamicAttr_801CA224() = || { unsafe { let _ = crate::tu::grdynamicattr::grDynamicAttr_801CA224(); } };
    fn itGetKind(a0: *mut HSD_GObj) -> ItemKind;
    fn OSReport(a0: &str);
    }
    pub static mut DbLevel: DbLKind = unsafe { core::mem::zeroed() };
    pub static mut HSD_GObjPLinkHead: *mut *mut HSD_GObj = unsafe { core::mem::zeroed() };
    pub static mut cm_804D6468: *mut CmSubject = unsafe { core::mem::zeroed() };
    pub static mut stage_info: StageInfo = unsafe { core::mem::zeroed() };
}

static mut mpLib_StageRow_warned: i32 = (-1_i32);
#[repr(C)]
#[derive(Clone, Copy)]
pub struct mpLib_803BF248_t_x4 {
    pub x0: f32,
    pub x4: [i32; 4],
    pub x14: [i32; 3],
    pub x20: [i32; 4],
    pub x30: [i32; 3],
    pub x3C: [i32; 4],
    pub x4C: [i32; 3],
}

melee_core::gc_layout!(mpLib_803BF248_t_x4, "mpLib_803BF248_t_x4", size 0x58, align 4, [
    x0 @ 0x0: 4 "float", x4 @ 0x4: 16 "int[4]", x14 @ 0x14: 12 "int[3]", x20 @ 0x20: 16 "int[4]", x30 @ 0x30: 12 "int[3]", x3C @ 0x3c: 16 "int[4]", x4C @ 0x4c: 12 "int[3]",
]);

#[repr(C)]
#[derive(Clone, Copy)]
pub struct mpLib_803BF248_t {
    pub id: GrKind,
    pub x4: *mut [*mut mpLib_803BF248_t_x4; 20],
}

melee_core::gc_layout!(mpLib_803BF248_t, "mpLib_803BF248_t", size 0x8, align 4, [
    id @ 0x0: 4 "GrKind", x4 @ 0x4: 4 "struct mpLib_803BF248_t_x4 *(*)[20]",
]);

pub static mut mpLib_80458868: [mpCollisionBox; 2] = unsafe { core::mem::zeroed() };
static mut didCheckBounding: i32 = 0;
static mut mpLib_804D64B4: *mut MapCollData = null_mut();
static groundCollVtx_count: u32 = 2048_u32;
static groundCollLine_count: u32 = 1536_u32;
static groundCollJoint_count: u32 = 256_u32;
static mut groundCollVtx: *mut CollVtx = null_mut();
static mut groundCollLine: *mut CollLine = null_mut();
static mut groundCollJoint: *mut CollJoint = null_mut();
static mut jointListStart: *mut CollJoint = null_mut();
static mut jointListEnd: *mut CollJoint = null_mut();
static mut scriptBatchDepth: i32 = 0;
static mut scriptBatchDirty: [u8; 1024] = [0; 1024];
static mut mpLib_804D64CC: i32 = 0;
static mut mpLib_804D64D0: i32 = 0;
static mut mpLib_804D64D4: i32 = 0;
static mut mpLib_804D64D8: i32 = 0;
static mut mpLib_804D64DC: i32 = 0;
static mut mpLib_804D64E0: i32 = 0;
static mut mpLib_804D64E4: i32 = 0;
pub static mut mpLib_80458888: [Vec; 128] = unsafe { core::mem::zeroed() };
pub static mut mpLib_803BD3D8: mpLib_803BF248_t_x4 = mpLib_803BF248_t_x4 { x0: 1.0f32, x4: [(-1_i32), (-1_i32), (-1_i32), (-1_i32)], x14: [0, (-1_i32), 0], x20: [(-1_i32), (-1_i32), (-1_i32), (-1_i32)], x30: [0, (-1_i32), 0], x3C: [(-1_i32), (-1_i32), (-1_i32), (-1_i32)], x4C: [0, (-1_i32), 0] };
static mut mpLib_803BD430: mpLib_803BF248_t_x4 = mpLib_803BF248_t_x4 { x0: 1.0f32, x4: [353, 353, 353, 353], x14: [1, (-1_i32), 0], x20: [(-1_i32), (-1_i32), (-1_i32), (-1_i32)], x30: [0, (-1_i32), 0], x3C: [54, 55, 56, 57], x4C: [1, (-1_i32), 0] };
static mut mpLib_803BD488: mpLib_803BF248_t_x4 = mpLib_803BF248_t_x4 { x0: 1.5f32, x4: [338, 338, 338, 338], x14: [1, (-1_i32), 0], x20: [(-1_i32), (-1_i32), (-1_i32), (-1_i32)], x30: [0, (-1_i32), 0], x3C: [29, 30, 31, 32], x4C: [1, (-1_i32), 0] };
static mut mpLib_803BD4E0: mpLib_803BF248_t_x4 = mpLib_803BF248_t_x4 { x0: 1.0f32, x4: [341, 341, 341, 341], x14: [1, (-1_i32), 0], x20: [(-1_i32), (-1_i32), (-1_i32), (-1_i32)], x30: [0, (-1_i32), 0], x3C: [34, 35, 36, 37], x4C: [1, (-1_i32), 0] };
static mut mpLib_803BD538: mpLib_803BF248_t_x4 = mpLib_803BF248_t_x4 { x0: 1.0f32, x4: [365, 365, 365, 365], x14: [1, (-1_i32), 0], x20: [(-1_i32), (-1_i32), (-1_i32), (-1_i32)], x30: [0, (-1_i32), 0], x3C: [14, 15, 16, 17], x4C: [0, (-1_i32), 0] };
static mut mpLib_803BD590: mpLib_803BF248_t_x4 = mpLib_803BF248_t_x4 { x0: 1.0f32, x4: [347, 347, 347, 347], x14: [1, (-1_i32), 0], x20: [75, 75, 76, 76], x30: [1, (-1_i32), 0], x3C: [44, 45, 46, 47], x4C: [1, (-1_i32), 1] };
static mut mpLib_803BD5E8: mpLib_803BF248_t_x4 = mpLib_803BF248_t_x4 { x0: 1.0f32, x4: [344, 344, 344, 344], x14: [1, (-1_i32), 0], x20: [(-1_i32), (-1_i32), (-1_i32), (-1_i32)], x30: [0, (-1_i32), 0], x3C: [39, 40, 41, 42], x4C: [1, (-1_i32), 0] };
static mut mpLib_803BD640: mpLib_803BF248_t_x4 = mpLib_803BF248_t_x4 { x0: 1.0f32, x4: [362, 362, 362, 362], x14: [1, (-1_i32), 0], x20: [(-1_i32), (-1_i32), (-1_i32), (-1_i32)], x30: [0, (-1_i32), 0], x3C: [59, 60, 61, 62], x4C: [1, (-1_i32), 0] };
static mut mpLib_803BD698: mpLib_803BF248_t_x4 = mpLib_803BF248_t_x4 { x0: 1.0f32, x4: [350, 350, 350, 350], x14: [0, (-1_i32), 0], x20: [(-1_i32), (-1_i32), (-1_i32), (-1_i32)], x30: [0, (-1_i32), 0], x3C: [49, 50, 51, 52], x4C: [0, (-1_i32), 0] };
static mut mpLib_803BD6F0: mpLib_803BF248_t_x4 = mpLib_803BF248_t_x4 { x0: 1.0f32, x4: [(-1_i32), (-1_i32), (-1_i32), (-1_i32)], x14: [0, (-1_i32), 0], x20: [(-1_i32), (-1_i32), (-1_i32), (-1_i32)], x30: [0, (-1_i32), 0], x3C: [(-1_i32), (-1_i32), (-1_i32), (-1_i32)], x4C: [0, (-1_i32), 0] };
static mut mpLib_803BD748: mpLib_803BF248_t_x4 = mpLib_803BF248_t_x4 { x0: 1.0f32, x4: [335, 335, 335, 335], x14: [1, (-1_i32), 0], x20: [526, 526, 526, 526], x30: [0, (-1_i32), 0], x3C: [24, 25, 26, 27], x4C: [1, (-1_i32), 0] };
static mut mpLib_803BD7A0: mpLib_803BF248_t_x4 = mpLib_803BF248_t_x4 { x0: 1.0f32, x4: [335, 335, 335, 335], x14: [1, 30007, 0], x20: [526, 526, 526, 526], x30: [0, 30008, 0], x3C: [24, 25, 26, 27], x4C: [1, 30011, 0] };
static mut mpLib_803BD7F8: mpLib_803BF248_t_x4 = mpLib_803BF248_t_x4 { x0: 1.0f32, x4: [335, 335, 335, 335], x14: [1, 30009, 0], x20: [526, 526, 526, 526], x30: [0, 30003, 0], x3C: [24, 25, 26, 27], x4C: [1, 30010, 0] };
static mut mpLib_803BD850: mpLib_803BF248_t_x4 = mpLib_803BF248_t_x4 { x0: 1.0f32, x4: [335, 335, 335, 335], x14: [1, 30026, 0], x20: [526, 526, 526, 526], x30: [0, 30009, 0], x3C: [24, 25, 26, 27], x4C: [1, (-1_i32), 0] };
static mut mpLib_803BD8A8: mpLib_803BF248_t_x4 = mpLib_803BF248_t_x4 { x0: 1.0f32, x4: [(-1_i32), (-1_i32), (-1_i32), (-1_i32)], x14: [0, (-1_i32), 0], x20: [(-1_i32), (-1_i32), (-1_i32), (-1_i32)], x30: [0, (-1_i32), 0], x3C: [(-1_i32), (-1_i32), (-1_i32), (-1_i32)], x4C: [0, (-1_i32), 0] };
static mut mpLib_803BD900: mpLib_803BF248_t_x4 = mpLib_803BF248_t_x4 { x0: 0.1f32, x4: [(-1_i32), (-1_i32), (-1_i32), (-1_i32)], x14: [0, (-1_i32), 0], x20: [(-1_i32), (-1_i32), (-1_i32), (-1_i32)], x30: [0, (-1_i32), 0], x3C: [(-1_i32), (-1_i32), (-1_i32), (-1_i32)], x4C: [0, (-1_i32), 0] };
static mut mpLib_803BD958: mpLib_803BF248_t_x4 = mpLib_803BF248_t_x4 { x0: 0.9f32, x4: [(-1_i32), (-1_i32), (-1_i32), (-1_i32)], x14: [0, (-1_i32), 0], x20: [(-1_i32), (-1_i32), (-1_i32), (-1_i32)], x30: [0, (-1_i32), 0], x3C: [(-1_i32), (-1_i32), (-1_i32), (-1_i32)], x4C: [0, (-1_i32), 0] };
static mut mpLib_803BD9B0: mpLib_803BF248_t_x4 = mpLib_803BF248_t_x4 { x0: 1.0f32, x4: [332, 332, 332, 332], x14: [1, (-1_i32), 0], x20: [(-1_i32), (-1_i32), (-1_i32), (-1_i32)], x30: [0, (-1_i32), 0], x3C: [19, 20, 21, 22], x4C: [1, (-1_i32), 0] };
static mut mpLib_803BDA08: mpLib_803BF248_t_x4 = mpLib_803BF248_t_x4 { x0: 1.0f32, x4: [332, 332, 332, 332], x14: [1, 30001, 0], x20: [(-1_i32), (-1_i32), (-1_i32), (-1_i32)], x30: [0, 30002, 0], x3C: [19, 20, 21, 22], x4C: [1, 30003, 0] };
static mut mpLib_803BDA60: mpLib_803BF248_t_x4 = mpLib_803BF248_t_x4 { x0: 0.2f32, x4: [356, 356, 356, 356], x14: [1, (-1_i32), 0], x20: [(-1_i32), (-1_i32), (-1_i32), (-1_i32)], x30: [0, (-1_i32), 0], x3C: [54, 55, 56, 57], x4C: [1, (-1_i32), 0] };
static mut mpLib_803BDAB8: mpLib_803BF248_t_x4 = mpLib_803BF248_t_x4 { x0: 1.0f32, x4: [368, 368, 368, 368], x14: [0, (-1_i32), 0], x20: [(-1_i32), (-1_i32), (-1_i32), (-1_i32)], x30: [0, (-1_i32), 0], x3C: [64, 65, 66, 67], x4C: [0, (-1_i32), 0] };
static mut mpLib_803BDB10: mpLib_803BF248_t_x4 = mpLib_803BF248_t_x4 { x0: 0.1f32, x4: [350, 350, 350, 350], x14: [1, (-1_i32), 0], x20: [(-1_i32), (-1_i32), (-1_i32), (-1_i32)], x30: [0, (-1_i32), 0], x3C: [49, 50, 51, 52], x4C: [1, (-1_i32), 0] };
static mut mpLib_803BDB68: mpLib_803BF248_t_x4 = mpLib_803BF248_t_x4 { x0: 1.0f32, x4: [359, 359, 359, 359], x14: [1, (-1_i32), 0], x20: [(-1_i32), (-1_i32), (-1_i32), (-1_i32)], x30: [0, (-1_i32), 0], x3C: [59, 60, 61, 62], x4C: [1, (-1_i32), 0] };
static mut mpLib_803BDBC0: mpLib_803BF248_t_x4 = mpLib_803BF248_t_x4 { x0: 1.0f32, x4: [(-1_i32), (-1_i32), (-1_i32), (-1_i32)], x14: [0, (-1_i32), 0], x20: [(-1_i32), (-1_i32), (-1_i32), (-1_i32)], x30: [0, (-1_i32), 0], x3C: [(-1_i32), (-1_i32), (-1_i32), (-1_i32)], x4C: [0, (-1_i32), 0] };
static mut mpLib_803BDC18: [*mut mpLib_803BF248_t_x4; 20] = [core::ptr::addr_of_mut!(mpLib_803BD3D8), core::ptr::addr_of_mut!(mpLib_803BD430), core::ptr::addr_of_mut!(mpLib_803BD488), core::ptr::addr_of_mut!(mpLib_803BD4E0), core::ptr::addr_of_mut!(mpLib_803BD538), core::ptr::addr_of_mut!(mpLib_803BD590), core::ptr::addr_of_mut!(mpLib_803BD5E8), core::ptr::addr_of_mut!(mpLib_803BD640), core::ptr::addr_of_mut!(mpLib_803BD698), core::ptr::addr_of_mut!(mpLib_803BD6F0), core::ptr::addr_of_mut!(mpLib_803BD748), core::ptr::addr_of_mut!(mpLib_803BD8A8), core::ptr::addr_of_mut!(mpLib_803BD900), core::ptr::addr_of_mut!(mpLib_803BD958), core::ptr::addr_of_mut!(mpLib_803BD9B0), core::ptr::addr_of_mut!(mpLib_803BDA60), core::ptr::addr_of_mut!(mpLib_803BDAB8), core::ptr::addr_of_mut!(mpLib_803BDB10), core::ptr::addr_of_mut!(mpLib_803BDB68), core::ptr::addr_of_mut!(mpLib_803BDBC0)];
static mut mpLib_803BDC68: [*mut mpLib_803BF248_t_x4; 20] = [core::ptr::addr_of_mut!(mpLib_803BD3D8), core::ptr::addr_of_mut!(mpLib_803BD430), core::ptr::addr_of_mut!(mpLib_803BD488), core::ptr::addr_of_mut!(mpLib_803BD4E0), core::ptr::addr_of_mut!(mpLib_803BD538), core::ptr::addr_of_mut!(mpLib_803BD590), core::ptr::addr_of_mut!(mpLib_803BD5E8), core::ptr::addr_of_mut!(mpLib_803BD640), core::ptr::addr_of_mut!(mpLib_803BD698), core::ptr::addr_of_mut!(mpLib_803BD6F0), core::ptr::addr_of_mut!(mpLib_803BD748), core::ptr::addr_of_mut!(mpLib_803BD8A8), core::ptr::addr_of_mut!(mpLib_803BD900), core::ptr::addr_of_mut!(mpLib_803BD958), core::ptr::addr_of_mut!(mpLib_803BD9B0), core::ptr::addr_of_mut!(mpLib_803BDA60), core::ptr::addr_of_mut!(mpLib_803BDAB8), core::ptr::addr_of_mut!(mpLib_803BDB10), core::ptr::addr_of_mut!(mpLib_803BDB68), core::ptr::addr_of_mut!(mpLib_803BDBC0)];
static mut mpLib_803BDCB8: [*mut mpLib_803BF248_t_x4; 20] = [core::ptr::addr_of_mut!(mpLib_803BD3D8), core::ptr::addr_of_mut!(mpLib_803BD430), core::ptr::addr_of_mut!(mpLib_803BD488), core::ptr::addr_of_mut!(mpLib_803BD4E0), core::ptr::addr_of_mut!(mpLib_803BD538), core::ptr::addr_of_mut!(mpLib_803BD590), core::ptr::addr_of_mut!(mpLib_803BD5E8), core::ptr::addr_of_mut!(mpLib_803BD640), core::ptr::addr_of_mut!(mpLib_803BD698), core::ptr::addr_of_mut!(mpLib_803BD6F0), core::ptr::addr_of_mut!(mpLib_803BD748), core::ptr::addr_of_mut!(mpLib_803BD8A8), core::ptr::addr_of_mut!(mpLib_803BD900), core::ptr::addr_of_mut!(mpLib_803BD958), core::ptr::addr_of_mut!(mpLib_803BD9B0), core::ptr::addr_of_mut!(mpLib_803BDA60), core::ptr::addr_of_mut!(mpLib_803BDAB8), core::ptr::addr_of_mut!(mpLib_803BDB10), core::ptr::addr_of_mut!(mpLib_803BDB68), core::ptr::addr_of_mut!(mpLib_803BDBC0)];
static mut mpLib_803BDD08: [*mut mpLib_803BF248_t_x4; 20] = [core::ptr::addr_of_mut!(mpLib_803BD3D8), core::ptr::addr_of_mut!(mpLib_803BD430), core::ptr::addr_of_mut!(mpLib_803BD488), core::ptr::addr_of_mut!(mpLib_803BD4E0), core::ptr::addr_of_mut!(mpLib_803BD538), core::ptr::addr_of_mut!(mpLib_803BD590), core::ptr::addr_of_mut!(mpLib_803BD5E8), core::ptr::addr_of_mut!(mpLib_803BD640), core::ptr::addr_of_mut!(mpLib_803BD698), core::ptr::addr_of_mut!(mpLib_803BD6F0), core::ptr::addr_of_mut!(mpLib_803BD748), core::ptr::addr_of_mut!(mpLib_803BD8A8), core::ptr::addr_of_mut!(mpLib_803BD900), core::ptr::addr_of_mut!(mpLib_803BD958), core::ptr::addr_of_mut!(mpLib_803BD9B0), core::ptr::addr_of_mut!(mpLib_803BDA60), core::ptr::addr_of_mut!(mpLib_803BDAB8), core::ptr::addr_of_mut!(mpLib_803BDB10), core::ptr::addr_of_mut!(mpLib_803BDB68), core::ptr::addr_of_mut!(mpLib_803BDBC0)];
static mut mpLib_803BDD58: [*mut mpLib_803BF248_t_x4; 20] = [core::ptr::addr_of_mut!(mpLib_803BD3D8), core::ptr::addr_of_mut!(mpLib_803BD430), core::ptr::addr_of_mut!(mpLib_803BD488), core::ptr::addr_of_mut!(mpLib_803BD4E0), core::ptr::addr_of_mut!(mpLib_803BD538), core::ptr::addr_of_mut!(mpLib_803BD590), core::ptr::addr_of_mut!(mpLib_803BD5E8), core::ptr::addr_of_mut!(mpLib_803BD640), core::ptr::addr_of_mut!(mpLib_803BD698), core::ptr::addr_of_mut!(mpLib_803BD6F0), core::ptr::addr_of_mut!(mpLib_803BD748), core::ptr::addr_of_mut!(mpLib_803BD8A8), core::ptr::addr_of_mut!(mpLib_803BD900), core::ptr::addr_of_mut!(mpLib_803BD958), core::ptr::addr_of_mut!(mpLib_803BD9B0), core::ptr::addr_of_mut!(mpLib_803BDA60), core::ptr::addr_of_mut!(mpLib_803BDAB8), core::ptr::addr_of_mut!(mpLib_803BDB10), core::ptr::addr_of_mut!(mpLib_803BDB68), core::ptr::addr_of_mut!(mpLib_803BDBC0)];
static mut mpLib_803BDDA8: [*mut mpLib_803BF248_t_x4; 20] = [core::ptr::addr_of_mut!(mpLib_803BD3D8), core::ptr::addr_of_mut!(mpLib_803BD430), core::ptr::addr_of_mut!(mpLib_803BD488), core::ptr::addr_of_mut!(mpLib_803BD4E0), core::ptr::addr_of_mut!(mpLib_803BD538), core::ptr::addr_of_mut!(mpLib_803BD590), core::ptr::addr_of_mut!(mpLib_803BD5E8), core::ptr::addr_of_mut!(mpLib_803BD640), core::ptr::addr_of_mut!(mpLib_803BD698), core::ptr::addr_of_mut!(mpLib_803BD6F0), core::ptr::addr_of_mut!(mpLib_803BD748), core::ptr::addr_of_mut!(mpLib_803BD8A8), core::ptr::addr_of_mut!(mpLib_803BD900), core::ptr::addr_of_mut!(mpLib_803BD958), core::ptr::addr_of_mut!(mpLib_803BD9B0), core::ptr::addr_of_mut!(mpLib_803BDA60), core::ptr::addr_of_mut!(mpLib_803BDAB8), core::ptr::addr_of_mut!(mpLib_803BDB10), core::ptr::addr_of_mut!(mpLib_803BDB68), core::ptr::addr_of_mut!(mpLib_803BDBC0)];
static mut mpLib_803BDDF8: [*mut mpLib_803BF248_t_x4; 20] = [core::ptr::addr_of_mut!(mpLib_803BD3D8), core::ptr::addr_of_mut!(mpLib_803BD430), core::ptr::addr_of_mut!(mpLib_803BD488), core::ptr::addr_of_mut!(mpLib_803BD4E0), core::ptr::addr_of_mut!(mpLib_803BD538), core::ptr::addr_of_mut!(mpLib_803BD590), core::ptr::addr_of_mut!(mpLib_803BD5E8), core::ptr::addr_of_mut!(mpLib_803BD640), core::ptr::addr_of_mut!(mpLib_803BD698), core::ptr::addr_of_mut!(mpLib_803BD6F0), core::ptr::addr_of_mut!(mpLib_803BD748), core::ptr::addr_of_mut!(mpLib_803BD8A8), core::ptr::addr_of_mut!(mpLib_803BD900), core::ptr::addr_of_mut!(mpLib_803BD958), core::ptr::addr_of_mut!(mpLib_803BD9B0), core::ptr::addr_of_mut!(mpLib_803BDA60), core::ptr::addr_of_mut!(mpLib_803BDAB8), core::ptr::addr_of_mut!(mpLib_803BDB10), core::ptr::addr_of_mut!(mpLib_803BDB68), core::ptr::addr_of_mut!(mpLib_803BDBC0)];
static mut mpLib_803BDE48: [*mut mpLib_803BF248_t_x4; 20] = [core::ptr::addr_of_mut!(mpLib_803BD3D8), core::ptr::addr_of_mut!(mpLib_803BD430), core::ptr::addr_of_mut!(mpLib_803BD488), core::ptr::addr_of_mut!(mpLib_803BD4E0), core::ptr::addr_of_mut!(mpLib_803BD538), core::ptr::addr_of_mut!(mpLib_803BD590), core::ptr::addr_of_mut!(mpLib_803BD5E8), core::ptr::addr_of_mut!(mpLib_803BD640), core::ptr::addr_of_mut!(mpLib_803BD698), core::ptr::addr_of_mut!(mpLib_803BD6F0), core::ptr::addr_of_mut!(mpLib_803BD748), core::ptr::addr_of_mut!(mpLib_803BD8A8), core::ptr::addr_of_mut!(mpLib_803BD900), core::ptr::addr_of_mut!(mpLib_803BD958), core::ptr::addr_of_mut!(mpLib_803BD9B0), core::ptr::addr_of_mut!(mpLib_803BDA60), core::ptr::addr_of_mut!(mpLib_803BDAB8), core::ptr::addr_of_mut!(mpLib_803BDB10), core::ptr::addr_of_mut!(mpLib_803BDB68), core::ptr::addr_of_mut!(mpLib_803BDBC0)];
static mut mpLib_803BDE98: [*mut mpLib_803BF248_t_x4; 20] = [core::ptr::addr_of_mut!(mpLib_803BD3D8), core::ptr::addr_of_mut!(mpLib_803BD430), core::ptr::addr_of_mut!(mpLib_803BD488), core::ptr::addr_of_mut!(mpLib_803BD4E0), core::ptr::addr_of_mut!(mpLib_803BD538), core::ptr::addr_of_mut!(mpLib_803BD590), core::ptr::addr_of_mut!(mpLib_803BD5E8), core::ptr::addr_of_mut!(mpLib_803BD640), core::ptr::addr_of_mut!(mpLib_803BD698), core::ptr::addr_of_mut!(mpLib_803BD6F0), core::ptr::addr_of_mut!(mpLib_803BD748), core::ptr::addr_of_mut!(mpLib_803BD8A8), core::ptr::addr_of_mut!(mpLib_803BD900), core::ptr::addr_of_mut!(mpLib_803BD958), core::ptr::addr_of_mut!(mpLib_803BD9B0), core::ptr::addr_of_mut!(mpLib_803BDA60), core::ptr::addr_of_mut!(mpLib_803BDAB8), core::ptr::addr_of_mut!(mpLib_803BDB10), core::ptr::addr_of_mut!(mpLib_803BDB68), core::ptr::addr_of_mut!(mpLib_803BDBC0)];
static mut mpLib_803BDEE8: [*mut mpLib_803BF248_t_x4; 20] = [core::ptr::addr_of_mut!(mpLib_803BD3D8), core::ptr::addr_of_mut!(mpLib_803BD430), core::ptr::addr_of_mut!(mpLib_803BD488), core::ptr::addr_of_mut!(mpLib_803BD4E0), core::ptr::addr_of_mut!(mpLib_803BD538), core::ptr::addr_of_mut!(mpLib_803BD590), core::ptr::addr_of_mut!(mpLib_803BD5E8), core::ptr::addr_of_mut!(mpLib_803BD640), core::ptr::addr_of_mut!(mpLib_803BD698), core::ptr::addr_of_mut!(mpLib_803BD6F0), core::ptr::addr_of_mut!(mpLib_803BD748), core::ptr::addr_of_mut!(mpLib_803BD8A8), core::ptr::addr_of_mut!(mpLib_803BD900), core::ptr::addr_of_mut!(mpLib_803BD958), core::ptr::addr_of_mut!(mpLib_803BD9B0), core::ptr::addr_of_mut!(mpLib_803BDA60), core::ptr::addr_of_mut!(mpLib_803BDAB8), core::ptr::addr_of_mut!(mpLib_803BDB10), core::ptr::addr_of_mut!(mpLib_803BDB68), core::ptr::addr_of_mut!(mpLib_803BDBC0)];
static mut mpLib_803BDF38: [*mut mpLib_803BF248_t_x4; 20] = [core::ptr::addr_of_mut!(mpLib_803BD3D8), core::ptr::addr_of_mut!(mpLib_803BD430), core::ptr::addr_of_mut!(mpLib_803BD488), core::ptr::addr_of_mut!(mpLib_803BD4E0), core::ptr::addr_of_mut!(mpLib_803BD538), core::ptr::addr_of_mut!(mpLib_803BD590), core::ptr::addr_of_mut!(mpLib_803BD5E8), core::ptr::addr_of_mut!(mpLib_803BD640), core::ptr::addr_of_mut!(mpLib_803BD698), core::ptr::addr_of_mut!(mpLib_803BD6F0), core::ptr::addr_of_mut!(mpLib_803BD748), core::ptr::addr_of_mut!(mpLib_803BD8A8), core::ptr::addr_of_mut!(mpLib_803BD900), core::ptr::addr_of_mut!(mpLib_803BD958), core::ptr::addr_of_mut!(mpLib_803BD9B0), core::ptr::addr_of_mut!(mpLib_803BDA60), core::ptr::addr_of_mut!(mpLib_803BDAB8), core::ptr::addr_of_mut!(mpLib_803BDB10), core::ptr::addr_of_mut!(mpLib_803BDB68), core::ptr::addr_of_mut!(mpLib_803BDBC0)];
static mut mpLib_803BDF88: [*mut mpLib_803BF248_t_x4; 20] = [core::ptr::addr_of_mut!(mpLib_803BD3D8), core::ptr::addr_of_mut!(mpLib_803BD430), core::ptr::addr_of_mut!(mpLib_803BD488), core::ptr::addr_of_mut!(mpLib_803BD4E0), core::ptr::addr_of_mut!(mpLib_803BD538), core::ptr::addr_of_mut!(mpLib_803BD590), core::ptr::addr_of_mut!(mpLib_803BD5E8), core::ptr::addr_of_mut!(mpLib_803BD640), core::ptr::addr_of_mut!(mpLib_803BD698), core::ptr::addr_of_mut!(mpLib_803BD6F0), core::ptr::addr_of_mut!(mpLib_803BD748), core::ptr::addr_of_mut!(mpLib_803BD8A8), core::ptr::addr_of_mut!(mpLib_803BD900), core::ptr::addr_of_mut!(mpLib_803BD958), core::ptr::addr_of_mut!(mpLib_803BD9B0), core::ptr::addr_of_mut!(mpLib_803BDA60), core::ptr::addr_of_mut!(mpLib_803BDAB8), core::ptr::addr_of_mut!(mpLib_803BDB10), core::ptr::addr_of_mut!(mpLib_803BDB68), core::ptr::addr_of_mut!(mpLib_803BDBC0)];
static mut mpLib_803BDFD8: [*mut mpLib_803BF248_t_x4; 20] = [core::ptr::addr_of_mut!(mpLib_803BD3D8), core::ptr::addr_of_mut!(mpLib_803BD430), core::ptr::addr_of_mut!(mpLib_803BD488), core::ptr::addr_of_mut!(mpLib_803BD4E0), core::ptr::addr_of_mut!(mpLib_803BD538), core::ptr::addr_of_mut!(mpLib_803BD590), core::ptr::addr_of_mut!(mpLib_803BD5E8), core::ptr::addr_of_mut!(mpLib_803BD640), core::ptr::addr_of_mut!(mpLib_803BD698), core::ptr::addr_of_mut!(mpLib_803BD6F0), core::ptr::addr_of_mut!(mpLib_803BD7A0), core::ptr::addr_of_mut!(mpLib_803BD8A8), core::ptr::addr_of_mut!(mpLib_803BD900), core::ptr::addr_of_mut!(mpLib_803BD958), core::ptr::addr_of_mut!(mpLib_803BD9B0), core::ptr::addr_of_mut!(mpLib_803BDA60), core::ptr::addr_of_mut!(mpLib_803BDAB8), core::ptr::addr_of_mut!(mpLib_803BDB10), core::ptr::addr_of_mut!(mpLib_803BDB68), core::ptr::addr_of_mut!(mpLib_803BDBC0)];
static mut mpLib_803BE028: [*mut mpLib_803BF248_t_x4; 20] = [core::ptr::addr_of_mut!(mpLib_803BD3D8), core::ptr::addr_of_mut!(mpLib_803BD430), core::ptr::addr_of_mut!(mpLib_803BD488), core::ptr::addr_of_mut!(mpLib_803BD4E0), core::ptr::addr_of_mut!(mpLib_803BD538), core::ptr::addr_of_mut!(mpLib_803BD590), core::ptr::addr_of_mut!(mpLib_803BD5E8), core::ptr::addr_of_mut!(mpLib_803BD640), core::ptr::addr_of_mut!(mpLib_803BD698), core::ptr::addr_of_mut!(mpLib_803BD6F0), core::ptr::addr_of_mut!(mpLib_803BD748), core::ptr::addr_of_mut!(mpLib_803BD8A8), core::ptr::addr_of_mut!(mpLib_803BD900), core::ptr::addr_of_mut!(mpLib_803BD958), core::ptr::addr_of_mut!(mpLib_803BD9B0), core::ptr::addr_of_mut!(mpLib_803BDA60), core::ptr::addr_of_mut!(mpLib_803BDAB8), core::ptr::addr_of_mut!(mpLib_803BDB10), core::ptr::addr_of_mut!(mpLib_803BDB68), core::ptr::addr_of_mut!(mpLib_803BDBC0)];
static mut mpLib_803BE078: [*mut mpLib_803BF248_t_x4; 20] = [core::ptr::addr_of_mut!(mpLib_803BD3D8), core::ptr::addr_of_mut!(mpLib_803BD430), core::ptr::addr_of_mut!(mpLib_803BD488), core::ptr::addr_of_mut!(mpLib_803BD4E0), core::ptr::addr_of_mut!(mpLib_803BD538), core::ptr::addr_of_mut!(mpLib_803BD590), core::ptr::addr_of_mut!(mpLib_803BD5E8), core::ptr::addr_of_mut!(mpLib_803BD640), core::ptr::addr_of_mut!(mpLib_803BD698), core::ptr::addr_of_mut!(mpLib_803BD6F0), core::ptr::addr_of_mut!(mpLib_803BD748), core::ptr::addr_of_mut!(mpLib_803BD8A8), core::ptr::addr_of_mut!(mpLib_803BD900), core::ptr::addr_of_mut!(mpLib_803BD958), core::ptr::addr_of_mut!(mpLib_803BD9B0), core::ptr::addr_of_mut!(mpLib_803BDA60), core::ptr::addr_of_mut!(mpLib_803BDAB8), core::ptr::addr_of_mut!(mpLib_803BDB10), core::ptr::addr_of_mut!(mpLib_803BDB68), core::ptr::addr_of_mut!(mpLib_803BDBC0)];
static mut mpLib_803BE0C8: [*mut mpLib_803BF248_t_x4; 20] = [core::ptr::addr_of_mut!(mpLib_803BD3D8), core::ptr::addr_of_mut!(mpLib_803BD430), core::ptr::addr_of_mut!(mpLib_803BD488), core::ptr::addr_of_mut!(mpLib_803BD4E0), core::ptr::addr_of_mut!(mpLib_803BD538), core::ptr::addr_of_mut!(mpLib_803BD590), core::ptr::addr_of_mut!(mpLib_803BD5E8), core::ptr::addr_of_mut!(mpLib_803BD640), core::ptr::addr_of_mut!(mpLib_803BD698), core::ptr::addr_of_mut!(mpLib_803BD6F0), core::ptr::addr_of_mut!(mpLib_803BD748), core::ptr::addr_of_mut!(mpLib_803BD8A8), core::ptr::addr_of_mut!(mpLib_803BD900), core::ptr::addr_of_mut!(mpLib_803BD958), core::ptr::addr_of_mut!(mpLib_803BD9B0), core::ptr::addr_of_mut!(mpLib_803BDA60), core::ptr::addr_of_mut!(mpLib_803BDAB8), core::ptr::addr_of_mut!(mpLib_803BDB10), core::ptr::addr_of_mut!(mpLib_803BDB68), core::ptr::addr_of_mut!(mpLib_803BDBC0)];
static mut mpLib_803BE118: [*mut mpLib_803BF248_t_x4; 20] = [core::ptr::addr_of_mut!(mpLib_803BD3D8), core::ptr::addr_of_mut!(mpLib_803BD430), core::ptr::addr_of_mut!(mpLib_803BD488), core::ptr::addr_of_mut!(mpLib_803BD4E0), core::ptr::addr_of_mut!(mpLib_803BD538), core::ptr::addr_of_mut!(mpLib_803BD590), core::ptr::addr_of_mut!(mpLib_803BD5E8), core::ptr::addr_of_mut!(mpLib_803BD640), core::ptr::addr_of_mut!(mpLib_803BD698), core::ptr::addr_of_mut!(mpLib_803BD6F0), core::ptr::addr_of_mut!(mpLib_803BD850), core::ptr::addr_of_mut!(mpLib_803BD8A8), core::ptr::addr_of_mut!(mpLib_803BD900), core::ptr::addr_of_mut!(mpLib_803BD958), core::ptr::addr_of_mut!(mpLib_803BD9B0), core::ptr::addr_of_mut!(mpLib_803BDA60), core::ptr::addr_of_mut!(mpLib_803BDAB8), core::ptr::addr_of_mut!(mpLib_803BDB10), core::ptr::addr_of_mut!(mpLib_803BDB68), core::ptr::addr_of_mut!(mpLib_803BDBC0)];
static mut mpLib_803BE168: [*mut mpLib_803BF248_t_x4; 20] = [core::ptr::addr_of_mut!(mpLib_803BD3D8), core::ptr::addr_of_mut!(mpLib_803BD430), core::ptr::addr_of_mut!(mpLib_803BD488), core::ptr::addr_of_mut!(mpLib_803BD4E0), core::ptr::addr_of_mut!(mpLib_803BD538), core::ptr::addr_of_mut!(mpLib_803BD590), core::ptr::addr_of_mut!(mpLib_803BD5E8), core::ptr::addr_of_mut!(mpLib_803BD640), core::ptr::addr_of_mut!(mpLib_803BD698), core::ptr::addr_of_mut!(mpLib_803BD6F0), core::ptr::addr_of_mut!(mpLib_803BD748), core::ptr::addr_of_mut!(mpLib_803BD8A8), core::ptr::addr_of_mut!(mpLib_803BD900), core::ptr::addr_of_mut!(mpLib_803BD958), core::ptr::addr_of_mut!(mpLib_803BD9B0), core::ptr::addr_of_mut!(mpLib_803BDA60), core::ptr::addr_of_mut!(mpLib_803BDAB8), core::ptr::addr_of_mut!(mpLib_803BDB10), core::ptr::addr_of_mut!(mpLib_803BDB68), core::ptr::addr_of_mut!(mpLib_803BDBC0)];
static mut mpLib_803BE1B8: [*mut mpLib_803BF248_t_x4; 20] = [core::ptr::addr_of_mut!(mpLib_803BD3D8), core::ptr::addr_of_mut!(mpLib_803BD430), core::ptr::addr_of_mut!(mpLib_803BD488), core::ptr::addr_of_mut!(mpLib_803BD4E0), core::ptr::addr_of_mut!(mpLib_803BD538), core::ptr::addr_of_mut!(mpLib_803BD590), core::ptr::addr_of_mut!(mpLib_803BD5E8), core::ptr::addr_of_mut!(mpLib_803BD640), core::ptr::addr_of_mut!(mpLib_803BD698), core::ptr::addr_of_mut!(mpLib_803BD6F0), core::ptr::addr_of_mut!(mpLib_803BD748), core::ptr::addr_of_mut!(mpLib_803BD8A8), core::ptr::addr_of_mut!(mpLib_803BD900), core::ptr::addr_of_mut!(mpLib_803BD958), core::ptr::addr_of_mut!(mpLib_803BD9B0), core::ptr::addr_of_mut!(mpLib_803BDA60), core::ptr::addr_of_mut!(mpLib_803BDAB8), core::ptr::addr_of_mut!(mpLib_803BDB10), core::ptr::addr_of_mut!(mpLib_803BDB68), core::ptr::addr_of_mut!(mpLib_803BDBC0)];
static mut mpLib_803BE208: [*mut mpLib_803BF248_t_x4; 20] = [core::ptr::addr_of_mut!(mpLib_803BD3D8), core::ptr::addr_of_mut!(mpLib_803BD430), core::ptr::addr_of_mut!(mpLib_803BD488), core::ptr::addr_of_mut!(mpLib_803BD4E0), core::ptr::addr_of_mut!(mpLib_803BD538), core::ptr::addr_of_mut!(mpLib_803BD590), core::ptr::addr_of_mut!(mpLib_803BD5E8), core::ptr::addr_of_mut!(mpLib_803BD640), core::ptr::addr_of_mut!(mpLib_803BD698), core::ptr::addr_of_mut!(mpLib_803BD6F0), core::ptr::addr_of_mut!(mpLib_803BD748), core::ptr::addr_of_mut!(mpLib_803BD8A8), core::ptr::addr_of_mut!(mpLib_803BD900), core::ptr::addr_of_mut!(mpLib_803BD958), core::ptr::addr_of_mut!(mpLib_803BD9B0), core::ptr::addr_of_mut!(mpLib_803BDA60), core::ptr::addr_of_mut!(mpLib_803BDAB8), core::ptr::addr_of_mut!(mpLib_803BDB10), core::ptr::addr_of_mut!(mpLib_803BDB68), core::ptr::addr_of_mut!(mpLib_803BDBC0)];
static mut mpLib_803BE258: [*mut mpLib_803BF248_t_x4; 20] = [core::ptr::addr_of_mut!(mpLib_803BD3D8), core::ptr::addr_of_mut!(mpLib_803BD430), core::ptr::addr_of_mut!(mpLib_803BD488), core::ptr::addr_of_mut!(mpLib_803BD4E0), core::ptr::addr_of_mut!(mpLib_803BD538), core::ptr::addr_of_mut!(mpLib_803BD590), core::ptr::addr_of_mut!(mpLib_803BD5E8), core::ptr::addr_of_mut!(mpLib_803BD640), core::ptr::addr_of_mut!(mpLib_803BD698), core::ptr::addr_of_mut!(mpLib_803BD6F0), core::ptr::addr_of_mut!(mpLib_803BD748), core::ptr::addr_of_mut!(mpLib_803BD8A8), core::ptr::addr_of_mut!(mpLib_803BD900), core::ptr::addr_of_mut!(mpLib_803BD958), core::ptr::addr_of_mut!(mpLib_803BD9B0), core::ptr::addr_of_mut!(mpLib_803BDA60), core::ptr::addr_of_mut!(mpLib_803BDAB8), core::ptr::addr_of_mut!(mpLib_803BDB10), core::ptr::addr_of_mut!(mpLib_803BDB68), core::ptr::addr_of_mut!(mpLib_803BDBC0)];
static mut mpLib_803BE2A8: [*mut mpLib_803BF248_t_x4; 20] = [core::ptr::addr_of_mut!(mpLib_803BD3D8), core::ptr::addr_of_mut!(mpLib_803BD430), core::ptr::addr_of_mut!(mpLib_803BD488), core::ptr::addr_of_mut!(mpLib_803BD4E0), core::ptr::addr_of_mut!(mpLib_803BD538), core::ptr::addr_of_mut!(mpLib_803BD590), core::ptr::addr_of_mut!(mpLib_803BD5E8), core::ptr::addr_of_mut!(mpLib_803BD640), core::ptr::addr_of_mut!(mpLib_803BD698), core::ptr::addr_of_mut!(mpLib_803BD6F0), core::ptr::addr_of_mut!(mpLib_803BD748), core::ptr::addr_of_mut!(mpLib_803BD8A8), core::ptr::addr_of_mut!(mpLib_803BD900), core::ptr::addr_of_mut!(mpLib_803BD958), core::ptr::addr_of_mut!(mpLib_803BD9B0), core::ptr::addr_of_mut!(mpLib_803BDA60), core::ptr::addr_of_mut!(mpLib_803BDAB8), core::ptr::addr_of_mut!(mpLib_803BDB10), core::ptr::addr_of_mut!(mpLib_803BDB68), core::ptr::addr_of_mut!(mpLib_803BDBC0)];
static mut mpLib_803BE2F8: [*mut mpLib_803BF248_t_x4; 20] = [core::ptr::addr_of_mut!(mpLib_803BD3D8), core::ptr::addr_of_mut!(mpLib_803BD430), core::ptr::addr_of_mut!(mpLib_803BD488), core::ptr::addr_of_mut!(mpLib_803BD4E0), core::ptr::addr_of_mut!(mpLib_803BD538), core::ptr::addr_of_mut!(mpLib_803BD590), core::ptr::addr_of_mut!(mpLib_803BD5E8), core::ptr::addr_of_mut!(mpLib_803BD640), core::ptr::addr_of_mut!(mpLib_803BD698), core::ptr::addr_of_mut!(mpLib_803BD6F0), core::ptr::addr_of_mut!(mpLib_803BD748), core::ptr::addr_of_mut!(mpLib_803BD8A8), core::ptr::addr_of_mut!(mpLib_803BD900), core::ptr::addr_of_mut!(mpLib_803BD958), core::ptr::addr_of_mut!(mpLib_803BDA08), core::ptr::addr_of_mut!(mpLib_803BDA60), core::ptr::addr_of_mut!(mpLib_803BDAB8), core::ptr::addr_of_mut!(mpLib_803BDB10), core::ptr::addr_of_mut!(mpLib_803BDB68), core::ptr::addr_of_mut!(mpLib_803BDBC0)];
static mut mpLib_803BE348: [*mut mpLib_803BF248_t_x4; 20] = [core::ptr::addr_of_mut!(mpLib_803BD3D8), core::ptr::addr_of_mut!(mpLib_803BD430), core::ptr::addr_of_mut!(mpLib_803BD488), core::ptr::addr_of_mut!(mpLib_803BD4E0), core::ptr::addr_of_mut!(mpLib_803BD538), core::ptr::addr_of_mut!(mpLib_803BD590), core::ptr::addr_of_mut!(mpLib_803BD5E8), core::ptr::addr_of_mut!(mpLib_803BD640), core::ptr::addr_of_mut!(mpLib_803BD698), core::ptr::addr_of_mut!(mpLib_803BD6F0), core::ptr::addr_of_mut!(mpLib_803BD748), core::ptr::addr_of_mut!(mpLib_803BD8A8), core::ptr::addr_of_mut!(mpLib_803BD900), core::ptr::addr_of_mut!(mpLib_803BD958), core::ptr::addr_of_mut!(mpLib_803BD9B0), core::ptr::addr_of_mut!(mpLib_803BDA60), core::ptr::addr_of_mut!(mpLib_803BDAB8), core::ptr::addr_of_mut!(mpLib_803BDB10), core::ptr::addr_of_mut!(mpLib_803BDB68), core::ptr::addr_of_mut!(mpLib_803BDBC0)];
static mut mpLib_803BE398: [*mut mpLib_803BF248_t_x4; 20] = [core::ptr::addr_of_mut!(mpLib_803BD3D8), core::ptr::addr_of_mut!(mpLib_803BD430), core::ptr::addr_of_mut!(mpLib_803BD488), core::ptr::addr_of_mut!(mpLib_803BD4E0), core::ptr::addr_of_mut!(mpLib_803BD538), core::ptr::addr_of_mut!(mpLib_803BD590), core::ptr::addr_of_mut!(mpLib_803BD5E8), core::ptr::addr_of_mut!(mpLib_803BD640), core::ptr::addr_of_mut!(mpLib_803BD698), core::ptr::addr_of_mut!(mpLib_803BD6F0), core::ptr::addr_of_mut!(mpLib_803BD748), core::ptr::addr_of_mut!(mpLib_803BD8A8), core::ptr::addr_of_mut!(mpLib_803BD900), core::ptr::addr_of_mut!(mpLib_803BD958), core::ptr::addr_of_mut!(mpLib_803BD9B0), core::ptr::addr_of_mut!(mpLib_803BDA60), core::ptr::addr_of_mut!(mpLib_803BDAB8), core::ptr::addr_of_mut!(mpLib_803BDB10), core::ptr::addr_of_mut!(mpLib_803BDB68), core::ptr::addr_of_mut!(mpLib_803BDBC0)];
static mut mpLib_803BE3E8: [*mut mpLib_803BF248_t_x4; 20] = [core::ptr::addr_of_mut!(mpLib_803BD3D8), core::ptr::addr_of_mut!(mpLib_803BD430), core::ptr::addr_of_mut!(mpLib_803BD488), core::ptr::addr_of_mut!(mpLib_803BD4E0), core::ptr::addr_of_mut!(mpLib_803BD538), core::ptr::addr_of_mut!(mpLib_803BD590), core::ptr::addr_of_mut!(mpLib_803BD5E8), core::ptr::addr_of_mut!(mpLib_803BD640), core::ptr::addr_of_mut!(mpLib_803BD698), core::ptr::addr_of_mut!(mpLib_803BD6F0), core::ptr::addr_of_mut!(mpLib_803BD748), core::ptr::addr_of_mut!(mpLib_803BD8A8), core::ptr::addr_of_mut!(mpLib_803BD900), core::ptr::addr_of_mut!(mpLib_803BD958), core::ptr::addr_of_mut!(mpLib_803BD9B0), core::ptr::addr_of_mut!(mpLib_803BDA60), core::ptr::addr_of_mut!(mpLib_803BDAB8), core::ptr::addr_of_mut!(mpLib_803BDB10), core::ptr::addr_of_mut!(mpLib_803BDB68), core::ptr::addr_of_mut!(mpLib_803BDBC0)];
static mut mpLib_803BE438: [*mut mpLib_803BF248_t_x4; 20] = [core::ptr::addr_of_mut!(mpLib_803BD3D8), core::ptr::addr_of_mut!(mpLib_803BD430), core::ptr::addr_of_mut!(mpLib_803BD488), core::ptr::addr_of_mut!(mpLib_803BD4E0), core::ptr::addr_of_mut!(mpLib_803BD538), core::ptr::addr_of_mut!(mpLib_803BD590), core::ptr::addr_of_mut!(mpLib_803BD5E8), core::ptr::addr_of_mut!(mpLib_803BD640), core::ptr::addr_of_mut!(mpLib_803BD698), core::ptr::addr_of_mut!(mpLib_803BD6F0), core::ptr::addr_of_mut!(mpLib_803BD748), core::ptr::addr_of_mut!(mpLib_803BD8A8), core::ptr::addr_of_mut!(mpLib_803BD900), core::ptr::addr_of_mut!(mpLib_803BD958), core::ptr::addr_of_mut!(mpLib_803BD9B0), core::ptr::addr_of_mut!(mpLib_803BDA60), core::ptr::addr_of_mut!(mpLib_803BDAB8), core::ptr::addr_of_mut!(mpLib_803BDB10), core::ptr::addr_of_mut!(mpLib_803BDB68), core::ptr::addr_of_mut!(mpLib_803BDBC0)];
static mut mpLib_803BE488: [*mut mpLib_803BF248_t_x4; 20] = [core::ptr::addr_of_mut!(mpLib_803BD3D8), core::ptr::addr_of_mut!(mpLib_803BD430), core::ptr::addr_of_mut!(mpLib_803BD488), core::ptr::addr_of_mut!(mpLib_803BD4E0), core::ptr::addr_of_mut!(mpLib_803BD538), core::ptr::addr_of_mut!(mpLib_803BD590), core::ptr::addr_of_mut!(mpLib_803BD5E8), core::ptr::addr_of_mut!(mpLib_803BD640), core::ptr::addr_of_mut!(mpLib_803BD698), core::ptr::addr_of_mut!(mpLib_803BD6F0), core::ptr::addr_of_mut!(mpLib_803BD748), core::ptr::addr_of_mut!(mpLib_803BD8A8), core::ptr::addr_of_mut!(mpLib_803BD900), core::ptr::addr_of_mut!(mpLib_803BD958), core::ptr::addr_of_mut!(mpLib_803BD9B0), core::ptr::addr_of_mut!(mpLib_803BDA60), core::ptr::addr_of_mut!(mpLib_803BDAB8), core::ptr::addr_of_mut!(mpLib_803BDB10), core::ptr::addr_of_mut!(mpLib_803BDB68), core::ptr::addr_of_mut!(mpLib_803BDBC0)];
static mut mpLib_803BE4D8: [*mut mpLib_803BF248_t_x4; 20] = [core::ptr::addr_of_mut!(mpLib_803BD3D8), core::ptr::addr_of_mut!(mpLib_803BD430), core::ptr::addr_of_mut!(mpLib_803BD488), core::ptr::addr_of_mut!(mpLib_803BD4E0), core::ptr::addr_of_mut!(mpLib_803BD538), core::ptr::addr_of_mut!(mpLib_803BD590), core::ptr::addr_of_mut!(mpLib_803BD5E8), core::ptr::addr_of_mut!(mpLib_803BD640), core::ptr::addr_of_mut!(mpLib_803BD698), core::ptr::addr_of_mut!(mpLib_803BD6F0), core::ptr::addr_of_mut!(mpLib_803BD748), core::ptr::addr_of_mut!(mpLib_803BD8A8), core::ptr::addr_of_mut!(mpLib_803BD900), core::ptr::addr_of_mut!(mpLib_803BD958), core::ptr::addr_of_mut!(mpLib_803BD9B0), core::ptr::addr_of_mut!(mpLib_803BDA60), core::ptr::addr_of_mut!(mpLib_803BDAB8), core::ptr::addr_of_mut!(mpLib_803BDB10), core::ptr::addr_of_mut!(mpLib_803BDB68), core::ptr::addr_of_mut!(mpLib_803BDBC0)];
static mut mpLib_803BE528: [*mut mpLib_803BF248_t_x4; 20] = [core::ptr::addr_of_mut!(mpLib_803BD3D8), core::ptr::addr_of_mut!(mpLib_803BD430), core::ptr::addr_of_mut!(mpLib_803BD488), core::ptr::addr_of_mut!(mpLib_803BD4E0), core::ptr::addr_of_mut!(mpLib_803BD538), core::ptr::addr_of_mut!(mpLib_803BD590), core::ptr::addr_of_mut!(mpLib_803BD5E8), core::ptr::addr_of_mut!(mpLib_803BD640), core::ptr::addr_of_mut!(mpLib_803BD698), core::ptr::addr_of_mut!(mpLib_803BD6F0), core::ptr::addr_of_mut!(mpLib_803BD748), core::ptr::addr_of_mut!(mpLib_803BD8A8), core::ptr::addr_of_mut!(mpLib_803BD900), core::ptr::addr_of_mut!(mpLib_803BD958), core::ptr::addr_of_mut!(mpLib_803BD9B0), core::ptr::addr_of_mut!(mpLib_803BDA60), core::ptr::addr_of_mut!(mpLib_803BDAB8), core::ptr::addr_of_mut!(mpLib_803BDB10), core::ptr::addr_of_mut!(mpLib_803BDB68), core::ptr::addr_of_mut!(mpLib_803BDBC0)];
static mut mpLib_803BE578: [*mut mpLib_803BF248_t_x4; 20] = [core::ptr::addr_of_mut!(mpLib_803BD3D8), core::ptr::addr_of_mut!(mpLib_803BD430), core::ptr::addr_of_mut!(mpLib_803BD488), core::ptr::addr_of_mut!(mpLib_803BD4E0), core::ptr::addr_of_mut!(mpLib_803BD538), core::ptr::addr_of_mut!(mpLib_803BD590), core::ptr::addr_of_mut!(mpLib_803BD5E8), core::ptr::addr_of_mut!(mpLib_803BD640), core::ptr::addr_of_mut!(mpLib_803BD698), core::ptr::addr_of_mut!(mpLib_803BD6F0), core::ptr::addr_of_mut!(mpLib_803BD748), core::ptr::addr_of_mut!(mpLib_803BD8A8), core::ptr::addr_of_mut!(mpLib_803BD900), core::ptr::addr_of_mut!(mpLib_803BD958), core::ptr::addr_of_mut!(mpLib_803BD9B0), core::ptr::addr_of_mut!(mpLib_803BDA60), core::ptr::addr_of_mut!(mpLib_803BDAB8), core::ptr::addr_of_mut!(mpLib_803BDB10), core::ptr::addr_of_mut!(mpLib_803BDB68), core::ptr::addr_of_mut!(mpLib_803BDBC0)];
static mut mpLib_803BE5C8: [*mut mpLib_803BF248_t_x4; 20] = [core::ptr::addr_of_mut!(mpLib_803BD3D8), core::ptr::addr_of_mut!(mpLib_803BD430), core::ptr::addr_of_mut!(mpLib_803BD488), core::ptr::addr_of_mut!(mpLib_803BD4E0), core::ptr::addr_of_mut!(mpLib_803BD538), core::ptr::addr_of_mut!(mpLib_803BD590), core::ptr::addr_of_mut!(mpLib_803BD5E8), core::ptr::addr_of_mut!(mpLib_803BD640), core::ptr::addr_of_mut!(mpLib_803BD698), core::ptr::addr_of_mut!(mpLib_803BD6F0), core::ptr::addr_of_mut!(mpLib_803BD748), core::ptr::addr_of_mut!(mpLib_803BD8A8), core::ptr::addr_of_mut!(mpLib_803BD900), core::ptr::addr_of_mut!(mpLib_803BD958), core::ptr::addr_of_mut!(mpLib_803BD9B0), core::ptr::addr_of_mut!(mpLib_803BDA60), core::ptr::addr_of_mut!(mpLib_803BDAB8), core::ptr::addr_of_mut!(mpLib_803BDB10), core::ptr::addr_of_mut!(mpLib_803BDB68), core::ptr::addr_of_mut!(mpLib_803BDBC0)];
static mut mpLib_803BE618: [*mut mpLib_803BF248_t_x4; 20] = [core::ptr::addr_of_mut!(mpLib_803BD3D8), core::ptr::addr_of_mut!(mpLib_803BD430), core::ptr::addr_of_mut!(mpLib_803BD488), core::ptr::addr_of_mut!(mpLib_803BD4E0), core::ptr::addr_of_mut!(mpLib_803BD538), core::ptr::addr_of_mut!(mpLib_803BD590), core::ptr::addr_of_mut!(mpLib_803BD5E8), core::ptr::addr_of_mut!(mpLib_803BD640), core::ptr::addr_of_mut!(mpLib_803BD698), core::ptr::addr_of_mut!(mpLib_803BD6F0), core::ptr::addr_of_mut!(mpLib_803BD7F8), core::ptr::addr_of_mut!(mpLib_803BD8A8), core::ptr::addr_of_mut!(mpLib_803BD900), core::ptr::addr_of_mut!(mpLib_803BD958), core::ptr::addr_of_mut!(mpLib_803BD9B0), core::ptr::addr_of_mut!(mpLib_803BDA60), core::ptr::addr_of_mut!(mpLib_803BDAB8), core::ptr::addr_of_mut!(mpLib_803BDB10), core::ptr::addr_of_mut!(mpLib_803BDB68), core::ptr::addr_of_mut!(mpLib_803BDBC0)];
static mut mpLib_803BE668: [*mut mpLib_803BF248_t_x4; 20] = [core::ptr::addr_of_mut!(mpLib_803BD3D8), core::ptr::addr_of_mut!(mpLib_803BD430), core::ptr::addr_of_mut!(mpLib_803BD488), core::ptr::addr_of_mut!(mpLib_803BD4E0), core::ptr::addr_of_mut!(mpLib_803BD538), core::ptr::addr_of_mut!(mpLib_803BD590), core::ptr::addr_of_mut!(mpLib_803BD5E8), core::ptr::addr_of_mut!(mpLib_803BD640), core::ptr::addr_of_mut!(mpLib_803BD698), core::ptr::addr_of_mut!(mpLib_803BD6F0), core::ptr::addr_of_mut!(mpLib_803BD748), core::ptr::addr_of_mut!(mpLib_803BD8A8), core::ptr::addr_of_mut!(mpLib_803BD900), core::ptr::addr_of_mut!(mpLib_803BD958), core::ptr::addr_of_mut!(mpLib_803BD9B0), core::ptr::addr_of_mut!(mpLib_803BDA60), core::ptr::addr_of_mut!(mpLib_803BDAB8), core::ptr::addr_of_mut!(mpLib_803BDB10), core::ptr::addr_of_mut!(mpLib_803BDB68), core::ptr::addr_of_mut!(mpLib_803BDBC0)];
static mut mpLib_803BE6B8: [*mut mpLib_803BF248_t_x4; 20] = [core::ptr::addr_of_mut!(mpLib_803BD3D8), core::ptr::addr_of_mut!(mpLib_803BD430), core::ptr::addr_of_mut!(mpLib_803BD488), core::ptr::addr_of_mut!(mpLib_803BD4E0), core::ptr::addr_of_mut!(mpLib_803BD538), core::ptr::addr_of_mut!(mpLib_803BD590), core::ptr::addr_of_mut!(mpLib_803BD5E8), core::ptr::addr_of_mut!(mpLib_803BD640), core::ptr::addr_of_mut!(mpLib_803BD698), core::ptr::addr_of_mut!(mpLib_803BD6F0), core::ptr::addr_of_mut!(mpLib_803BD748), core::ptr::addr_of_mut!(mpLib_803BD8A8), core::ptr::addr_of_mut!(mpLib_803BD900), core::ptr::addr_of_mut!(mpLib_803BD958), core::ptr::addr_of_mut!(mpLib_803BD9B0), core::ptr::addr_of_mut!(mpLib_803BDA60), core::ptr::addr_of_mut!(mpLib_803BDAB8), core::ptr::addr_of_mut!(mpLib_803BDB10), core::ptr::addr_of_mut!(mpLib_803BDB68), core::ptr::addr_of_mut!(mpLib_803BDBC0)];
static mut mpLib_803BE708: [*mut mpLib_803BF248_t_x4; 20] = [core::ptr::addr_of_mut!(mpLib_803BD3D8), core::ptr::addr_of_mut!(mpLib_803BD430), core::ptr::addr_of_mut!(mpLib_803BD488), core::ptr::addr_of_mut!(mpLib_803BD4E0), core::ptr::addr_of_mut!(mpLib_803BD538), core::ptr::addr_of_mut!(mpLib_803BD590), core::ptr::addr_of_mut!(mpLib_803BD5E8), core::ptr::addr_of_mut!(mpLib_803BD640), core::ptr::addr_of_mut!(mpLib_803BD698), core::ptr::addr_of_mut!(mpLib_803BD6F0), core::ptr::addr_of_mut!(mpLib_803BD748), core::ptr::addr_of_mut!(mpLib_803BD8A8), core::ptr::addr_of_mut!(mpLib_803BD900), core::ptr::addr_of_mut!(mpLib_803BD958), core::ptr::addr_of_mut!(mpLib_803BD9B0), core::ptr::addr_of_mut!(mpLib_803BDA60), core::ptr::addr_of_mut!(mpLib_803BDAB8), core::ptr::addr_of_mut!(mpLib_803BDB10), core::ptr::addr_of_mut!(mpLib_803BDB68), core::ptr::addr_of_mut!(mpLib_803BDBC0)];
static mut mpLib_803BE758: [*mut mpLib_803BF248_t_x4; 20] = [core::ptr::addr_of_mut!(mpLib_803BD3D8), core::ptr::addr_of_mut!(mpLib_803BD430), core::ptr::addr_of_mut!(mpLib_803BD488), core::ptr::addr_of_mut!(mpLib_803BD4E0), core::ptr::addr_of_mut!(mpLib_803BD538), core::ptr::addr_of_mut!(mpLib_803BD590), core::ptr::addr_of_mut!(mpLib_803BD5E8), core::ptr::addr_of_mut!(mpLib_803BD640), core::ptr::addr_of_mut!(mpLib_803BD698), core::ptr::addr_of_mut!(mpLib_803BD6F0), core::ptr::addr_of_mut!(mpLib_803BD748), core::ptr::addr_of_mut!(mpLib_803BD8A8), core::ptr::addr_of_mut!(mpLib_803BD900), core::ptr::addr_of_mut!(mpLib_803BD958), core::ptr::addr_of_mut!(mpLib_803BD9B0), core::ptr::addr_of_mut!(mpLib_803BDA60), core::ptr::addr_of_mut!(mpLib_803BDAB8), core::ptr::addr_of_mut!(mpLib_803BDB10), core::ptr::addr_of_mut!(mpLib_803BDB68), core::ptr::addr_of_mut!(mpLib_803BDBC0)];
static mut mpLib_803BE7A8: [*mut mpLib_803BF248_t_x4; 20] = [core::ptr::addr_of_mut!(mpLib_803BD3D8), core::ptr::addr_of_mut!(mpLib_803BD430), core::ptr::addr_of_mut!(mpLib_803BD488), core::ptr::addr_of_mut!(mpLib_803BD4E0), core::ptr::addr_of_mut!(mpLib_803BD538), core::ptr::addr_of_mut!(mpLib_803BD590), core::ptr::addr_of_mut!(mpLib_803BD5E8), core::ptr::addr_of_mut!(mpLib_803BD640), core::ptr::addr_of_mut!(mpLib_803BD698), core::ptr::addr_of_mut!(mpLib_803BD6F0), core::ptr::addr_of_mut!(mpLib_803BD748), core::ptr::addr_of_mut!(mpLib_803BD8A8), core::ptr::addr_of_mut!(mpLib_803BD900), core::ptr::addr_of_mut!(mpLib_803BD958), core::ptr::addr_of_mut!(mpLib_803BD9B0), core::ptr::addr_of_mut!(mpLib_803BDA60), core::ptr::addr_of_mut!(mpLib_803BDAB8), core::ptr::addr_of_mut!(mpLib_803BDB10), core::ptr::addr_of_mut!(mpLib_803BDB68), core::ptr::addr_of_mut!(mpLib_803BDBC0)];
static mut mpLib_803BE7F8: [*mut mpLib_803BF248_t_x4; 20] = [core::ptr::addr_of_mut!(mpLib_803BD3D8), core::ptr::addr_of_mut!(mpLib_803BD430), core::ptr::addr_of_mut!(mpLib_803BD488), core::ptr::addr_of_mut!(mpLib_803BD4E0), core::ptr::addr_of_mut!(mpLib_803BD538), core::ptr::addr_of_mut!(mpLib_803BD590), core::ptr::addr_of_mut!(mpLib_803BD5E8), core::ptr::addr_of_mut!(mpLib_803BD640), core::ptr::addr_of_mut!(mpLib_803BD698), core::ptr::addr_of_mut!(mpLib_803BD6F0), core::ptr::addr_of_mut!(mpLib_803BD748), core::ptr::addr_of_mut!(mpLib_803BD8A8), core::ptr::addr_of_mut!(mpLib_803BD900), core::ptr::addr_of_mut!(mpLib_803BD958), core::ptr::addr_of_mut!(mpLib_803BD9B0), core::ptr::addr_of_mut!(mpLib_803BDA60), core::ptr::addr_of_mut!(mpLib_803BDAB8), core::ptr::addr_of_mut!(mpLib_803BDB10), core::ptr::addr_of_mut!(mpLib_803BDB68), core::ptr::addr_of_mut!(mpLib_803BDBC0)];
static mut mpLib_803BE848: [*mut mpLib_803BF248_t_x4; 20] = [core::ptr::addr_of_mut!(mpLib_803BD3D8), core::ptr::addr_of_mut!(mpLib_803BD430), core::ptr::addr_of_mut!(mpLib_803BD488), core::ptr::addr_of_mut!(mpLib_803BD4E0), core::ptr::addr_of_mut!(mpLib_803BD538), core::ptr::addr_of_mut!(mpLib_803BD590), core::ptr::addr_of_mut!(mpLib_803BD5E8), core::ptr::addr_of_mut!(mpLib_803BD640), core::ptr::addr_of_mut!(mpLib_803BD698), core::ptr::addr_of_mut!(mpLib_803BD6F0), core::ptr::addr_of_mut!(mpLib_803BD748), core::ptr::addr_of_mut!(mpLib_803BD8A8), core::ptr::addr_of_mut!(mpLib_803BD900), core::ptr::addr_of_mut!(mpLib_803BD958), core::ptr::addr_of_mut!(mpLib_803BD9B0), core::ptr::addr_of_mut!(mpLib_803BDA60), core::ptr::addr_of_mut!(mpLib_803BDAB8), core::ptr::addr_of_mut!(mpLib_803BDB10), core::ptr::addr_of_mut!(mpLib_803BDB68), core::ptr::addr_of_mut!(mpLib_803BDBC0)];
static mut mpLib_803BE898: [*mut mpLib_803BF248_t_x4; 20] = [core::ptr::addr_of_mut!(mpLib_803BD3D8), core::ptr::addr_of_mut!(mpLib_803BD430), core::ptr::addr_of_mut!(mpLib_803BD488), core::ptr::addr_of_mut!(mpLib_803BD4E0), core::ptr::addr_of_mut!(mpLib_803BD538), core::ptr::addr_of_mut!(mpLib_803BD590), core::ptr::addr_of_mut!(mpLib_803BD5E8), core::ptr::addr_of_mut!(mpLib_803BD640), core::ptr::addr_of_mut!(mpLib_803BD698), core::ptr::addr_of_mut!(mpLib_803BD6F0), core::ptr::addr_of_mut!(mpLib_803BD748), core::ptr::addr_of_mut!(mpLib_803BD8A8), core::ptr::addr_of_mut!(mpLib_803BD900), core::ptr::addr_of_mut!(mpLib_803BD958), core::ptr::addr_of_mut!(mpLib_803BD9B0), core::ptr::addr_of_mut!(mpLib_803BDA60), core::ptr::addr_of_mut!(mpLib_803BDAB8), core::ptr::addr_of_mut!(mpLib_803BDB10), core::ptr::addr_of_mut!(mpLib_803BDB68), core::ptr::addr_of_mut!(mpLib_803BDBC0)];
static mut mpLib_803BE8E8: [*mut mpLib_803BF248_t_x4; 20] = [core::ptr::addr_of_mut!(mpLib_803BD3D8), core::ptr::addr_of_mut!(mpLib_803BD430), core::ptr::addr_of_mut!(mpLib_803BD488), core::ptr::addr_of_mut!(mpLib_803BD4E0), core::ptr::addr_of_mut!(mpLib_803BD538), core::ptr::addr_of_mut!(mpLib_803BD590), core::ptr::addr_of_mut!(mpLib_803BD5E8), core::ptr::addr_of_mut!(mpLib_803BD640), core::ptr::addr_of_mut!(mpLib_803BD698), core::ptr::addr_of_mut!(mpLib_803BD6F0), core::ptr::addr_of_mut!(mpLib_803BD748), core::ptr::addr_of_mut!(mpLib_803BD8A8), core::ptr::addr_of_mut!(mpLib_803BD900), core::ptr::addr_of_mut!(mpLib_803BD958), core::ptr::addr_of_mut!(mpLib_803BD9B0), core::ptr::addr_of_mut!(mpLib_803BDA60), core::ptr::addr_of_mut!(mpLib_803BDAB8), core::ptr::addr_of_mut!(mpLib_803BDB10), core::ptr::addr_of_mut!(mpLib_803BDB68), core::ptr::addr_of_mut!(mpLib_803BDBC0)];
static mut mpLib_803BE938: [*mut mpLib_803BF248_t_x4; 20] = [core::ptr::addr_of_mut!(mpLib_803BD3D8), core::ptr::addr_of_mut!(mpLib_803BD430), core::ptr::addr_of_mut!(mpLib_803BD488), core::ptr::addr_of_mut!(mpLib_803BD4E0), core::ptr::addr_of_mut!(mpLib_803BD538), core::ptr::addr_of_mut!(mpLib_803BD590), core::ptr::addr_of_mut!(mpLib_803BD5E8), core::ptr::addr_of_mut!(mpLib_803BD640), core::ptr::addr_of_mut!(mpLib_803BD698), core::ptr::addr_of_mut!(mpLib_803BD6F0), core::ptr::addr_of_mut!(mpLib_803BD748), core::ptr::addr_of_mut!(mpLib_803BD8A8), core::ptr::addr_of_mut!(mpLib_803BD900), core::ptr::addr_of_mut!(mpLib_803BD958), core::ptr::addr_of_mut!(mpLib_803BD9B0), core::ptr::addr_of_mut!(mpLib_803BDA60), core::ptr::addr_of_mut!(mpLib_803BDAB8), core::ptr::addr_of_mut!(mpLib_803BDB10), core::ptr::addr_of_mut!(mpLib_803BDB68), core::ptr::addr_of_mut!(mpLib_803BDBC0)];
static mut mpLib_803BE988: [*mut mpLib_803BF248_t_x4; 20] = [core::ptr::addr_of_mut!(mpLib_803BD3D8), core::ptr::addr_of_mut!(mpLib_803BD430), core::ptr::addr_of_mut!(mpLib_803BD488), core::ptr::addr_of_mut!(mpLib_803BD4E0), core::ptr::addr_of_mut!(mpLib_803BD538), core::ptr::addr_of_mut!(mpLib_803BD590), core::ptr::addr_of_mut!(mpLib_803BD5E8), core::ptr::addr_of_mut!(mpLib_803BD640), core::ptr::addr_of_mut!(mpLib_803BD698), core::ptr::addr_of_mut!(mpLib_803BD6F0), core::ptr::addr_of_mut!(mpLib_803BD748), core::ptr::addr_of_mut!(mpLib_803BD8A8), core::ptr::addr_of_mut!(mpLib_803BD900), core::ptr::addr_of_mut!(mpLib_803BD958), core::ptr::addr_of_mut!(mpLib_803BD9B0), core::ptr::addr_of_mut!(mpLib_803BDA60), core::ptr::addr_of_mut!(mpLib_803BDAB8), core::ptr::addr_of_mut!(mpLib_803BDB10), core::ptr::addr_of_mut!(mpLib_803BDB68), core::ptr::addr_of_mut!(mpLib_803BDBC0)];
static mut mpLib_803BE9D8: [*mut mpLib_803BF248_t_x4; 20] = [core::ptr::addr_of_mut!(mpLib_803BD3D8), core::ptr::addr_of_mut!(mpLib_803BD430), core::ptr::addr_of_mut!(mpLib_803BD488), core::ptr::addr_of_mut!(mpLib_803BD4E0), core::ptr::addr_of_mut!(mpLib_803BD538), core::ptr::addr_of_mut!(mpLib_803BD590), core::ptr::addr_of_mut!(mpLib_803BD5E8), core::ptr::addr_of_mut!(mpLib_803BD640), core::ptr::addr_of_mut!(mpLib_803BD698), core::ptr::addr_of_mut!(mpLib_803BD6F0), core::ptr::addr_of_mut!(mpLib_803BD748), core::ptr::addr_of_mut!(mpLib_803BD8A8), core::ptr::addr_of_mut!(mpLib_803BD900), core::ptr::addr_of_mut!(mpLib_803BD958), core::ptr::addr_of_mut!(mpLib_803BD9B0), core::ptr::addr_of_mut!(mpLib_803BDA60), core::ptr::addr_of_mut!(mpLib_803BDAB8), core::ptr::addr_of_mut!(mpLib_803BDB10), core::ptr::addr_of_mut!(mpLib_803BDB68), core::ptr::addr_of_mut!(mpLib_803BDBC0)];
static mut mpLib_803BEA28: [*mut mpLib_803BF248_t_x4; 20] = [core::ptr::addr_of_mut!(mpLib_803BD3D8), core::ptr::addr_of_mut!(mpLib_803BD430), core::ptr::addr_of_mut!(mpLib_803BD488), core::ptr::addr_of_mut!(mpLib_803BD4E0), core::ptr::addr_of_mut!(mpLib_803BD538), core::ptr::addr_of_mut!(mpLib_803BD590), core::ptr::addr_of_mut!(mpLib_803BD5E8), core::ptr::addr_of_mut!(mpLib_803BD640), core::ptr::addr_of_mut!(mpLib_803BD698), core::ptr::addr_of_mut!(mpLib_803BD6F0), core::ptr::addr_of_mut!(mpLib_803BD748), core::ptr::addr_of_mut!(mpLib_803BD8A8), core::ptr::addr_of_mut!(mpLib_803BD900), core::ptr::addr_of_mut!(mpLib_803BD958), core::ptr::addr_of_mut!(mpLib_803BD9B0), core::ptr::addr_of_mut!(mpLib_803BDA60), core::ptr::addr_of_mut!(mpLib_803BDAB8), core::ptr::addr_of_mut!(mpLib_803BDB10), core::ptr::addr_of_mut!(mpLib_803BDB68), core::ptr::addr_of_mut!(mpLib_803BDBC0)];
static mut mpLib_803BEA78: [*mut mpLib_803BF248_t_x4; 20] = [core::ptr::addr_of_mut!(mpLib_803BD3D8), core::ptr::addr_of_mut!(mpLib_803BD430), core::ptr::addr_of_mut!(mpLib_803BD488), core::ptr::addr_of_mut!(mpLib_803BD4E0), core::ptr::addr_of_mut!(mpLib_803BD538), core::ptr::addr_of_mut!(mpLib_803BD590), core::ptr::addr_of_mut!(mpLib_803BD5E8), core::ptr::addr_of_mut!(mpLib_803BD640), core::ptr::addr_of_mut!(mpLib_803BD698), core::ptr::addr_of_mut!(mpLib_803BD6F0), core::ptr::addr_of_mut!(mpLib_803BD748), core::ptr::addr_of_mut!(mpLib_803BD8A8), core::ptr::addr_of_mut!(mpLib_803BD900), core::ptr::addr_of_mut!(mpLib_803BD958), core::ptr::addr_of_mut!(mpLib_803BD9B0), core::ptr::addr_of_mut!(mpLib_803BDA60), core::ptr::addr_of_mut!(mpLib_803BDAB8), core::ptr::addr_of_mut!(mpLib_803BDB10), core::ptr::addr_of_mut!(mpLib_803BDB68), core::ptr::addr_of_mut!(mpLib_803BDBC0)];
static mut mpLib_803BEAC8: [*mut mpLib_803BF248_t_x4; 20] = [core::ptr::addr_of_mut!(mpLib_803BD3D8), core::ptr::addr_of_mut!(mpLib_803BD430), core::ptr::addr_of_mut!(mpLib_803BD488), core::ptr::addr_of_mut!(mpLib_803BD4E0), core::ptr::addr_of_mut!(mpLib_803BD538), core::ptr::addr_of_mut!(mpLib_803BD590), core::ptr::addr_of_mut!(mpLib_803BD5E8), core::ptr::addr_of_mut!(mpLib_803BD640), core::ptr::addr_of_mut!(mpLib_803BD698), core::ptr::addr_of_mut!(mpLib_803BD6F0), core::ptr::addr_of_mut!(mpLib_803BD748), core::ptr::addr_of_mut!(mpLib_803BD8A8), core::ptr::addr_of_mut!(mpLib_803BD900), core::ptr::addr_of_mut!(mpLib_803BD958), core::ptr::addr_of_mut!(mpLib_803BD9B0), core::ptr::addr_of_mut!(mpLib_803BDA60), core::ptr::addr_of_mut!(mpLib_803BDAB8), core::ptr::addr_of_mut!(mpLib_803BDB10), core::ptr::addr_of_mut!(mpLib_803BDB68), core::ptr::addr_of_mut!(mpLib_803BDBC0)];
static mut mpLib_803BEB18: [*mut mpLib_803BF248_t_x4; 20] = [core::ptr::addr_of_mut!(mpLib_803BD3D8), core::ptr::addr_of_mut!(mpLib_803BD430), core::ptr::addr_of_mut!(mpLib_803BD488), core::ptr::addr_of_mut!(mpLib_803BD4E0), core::ptr::addr_of_mut!(mpLib_803BD538), core::ptr::addr_of_mut!(mpLib_803BD590), core::ptr::addr_of_mut!(mpLib_803BD5E8), core::ptr::addr_of_mut!(mpLib_803BD640), core::ptr::addr_of_mut!(mpLib_803BD698), core::ptr::addr_of_mut!(mpLib_803BD6F0), core::ptr::addr_of_mut!(mpLib_803BD748), core::ptr::addr_of_mut!(mpLib_803BD8A8), core::ptr::addr_of_mut!(mpLib_803BD900), core::ptr::addr_of_mut!(mpLib_803BD958), core::ptr::addr_of_mut!(mpLib_803BD9B0), core::ptr::addr_of_mut!(mpLib_803BDA60), core::ptr::addr_of_mut!(mpLib_803BDAB8), core::ptr::addr_of_mut!(mpLib_803BDB10), core::ptr::addr_of_mut!(mpLib_803BDB68), core::ptr::addr_of_mut!(mpLib_803BDBC0)];
static mut mpLib_803BEB68: [*mut mpLib_803BF248_t_x4; 20] = [core::ptr::addr_of_mut!(mpLib_803BD3D8), core::ptr::addr_of_mut!(mpLib_803BD430), core::ptr::addr_of_mut!(mpLib_803BD488), core::ptr::addr_of_mut!(mpLib_803BD4E0), core::ptr::addr_of_mut!(mpLib_803BD538), core::ptr::addr_of_mut!(mpLib_803BD590), core::ptr::addr_of_mut!(mpLib_803BD5E8), core::ptr::addr_of_mut!(mpLib_803BD640), core::ptr::addr_of_mut!(mpLib_803BD698), core::ptr::addr_of_mut!(mpLib_803BD6F0), core::ptr::addr_of_mut!(mpLib_803BD748), core::ptr::addr_of_mut!(mpLib_803BD8A8), core::ptr::addr_of_mut!(mpLib_803BD900), core::ptr::addr_of_mut!(mpLib_803BD958), core::ptr::addr_of_mut!(mpLib_803BD9B0), core::ptr::addr_of_mut!(mpLib_803BDA60), core::ptr::addr_of_mut!(mpLib_803BDAB8), core::ptr::addr_of_mut!(mpLib_803BDB10), core::ptr::addr_of_mut!(mpLib_803BDB68), core::ptr::addr_of_mut!(mpLib_803BDBC0)];
static mut mpLib_803BEBB8: [*mut mpLib_803BF248_t_x4; 20] = [core::ptr::addr_of_mut!(mpLib_803BD3D8), core::ptr::addr_of_mut!(mpLib_803BD430), core::ptr::addr_of_mut!(mpLib_803BD488), core::ptr::addr_of_mut!(mpLib_803BD4E0), core::ptr::addr_of_mut!(mpLib_803BD538), core::ptr::addr_of_mut!(mpLib_803BD590), core::ptr::addr_of_mut!(mpLib_803BD5E8), core::ptr::addr_of_mut!(mpLib_803BD640), core::ptr::addr_of_mut!(mpLib_803BD698), core::ptr::addr_of_mut!(mpLib_803BD6F0), core::ptr::addr_of_mut!(mpLib_803BD748), core::ptr::addr_of_mut!(mpLib_803BD8A8), core::ptr::addr_of_mut!(mpLib_803BD900), core::ptr::addr_of_mut!(mpLib_803BD958), core::ptr::addr_of_mut!(mpLib_803BD9B0), core::ptr::addr_of_mut!(mpLib_803BDA60), core::ptr::addr_of_mut!(mpLib_803BDAB8), core::ptr::addr_of_mut!(mpLib_803BDB10), core::ptr::addr_of_mut!(mpLib_803BDB68), core::ptr::addr_of_mut!(mpLib_803BDBC0)];
static mut mpLib_803BEC08: [*mut mpLib_803BF248_t_x4; 20] = [core::ptr::addr_of_mut!(mpLib_803BD3D8), core::ptr::addr_of_mut!(mpLib_803BD430), core::ptr::addr_of_mut!(mpLib_803BD488), core::ptr::addr_of_mut!(mpLib_803BD4E0), core::ptr::addr_of_mut!(mpLib_803BD538), core::ptr::addr_of_mut!(mpLib_803BD590), core::ptr::addr_of_mut!(mpLib_803BD5E8), core::ptr::addr_of_mut!(mpLib_803BD640), core::ptr::addr_of_mut!(mpLib_803BD698), core::ptr::addr_of_mut!(mpLib_803BD6F0), core::ptr::addr_of_mut!(mpLib_803BD748), core::ptr::addr_of_mut!(mpLib_803BD8A8), core::ptr::addr_of_mut!(mpLib_803BD900), core::ptr::addr_of_mut!(mpLib_803BD958), core::ptr::addr_of_mut!(mpLib_803BD9B0), core::ptr::addr_of_mut!(mpLib_803BDA60), core::ptr::addr_of_mut!(mpLib_803BDAB8), core::ptr::addr_of_mut!(mpLib_803BDB10), core::ptr::addr_of_mut!(mpLib_803BDB68), core::ptr::addr_of_mut!(mpLib_803BDBC0)];
static mut mpLib_803BEC58: [*mut mpLib_803BF248_t_x4; 20] = [core::ptr::addr_of_mut!(mpLib_803BD3D8), core::ptr::addr_of_mut!(mpLib_803BD430), core::ptr::addr_of_mut!(mpLib_803BD488), core::ptr::addr_of_mut!(mpLib_803BD4E0), core::ptr::addr_of_mut!(mpLib_803BD538), core::ptr::addr_of_mut!(mpLib_803BD590), core::ptr::addr_of_mut!(mpLib_803BD5E8), core::ptr::addr_of_mut!(mpLib_803BD640), core::ptr::addr_of_mut!(mpLib_803BD698), core::ptr::addr_of_mut!(mpLib_803BD6F0), core::ptr::addr_of_mut!(mpLib_803BD748), core::ptr::addr_of_mut!(mpLib_803BD8A8), core::ptr::addr_of_mut!(mpLib_803BD900), core::ptr::addr_of_mut!(mpLib_803BD958), core::ptr::addr_of_mut!(mpLib_803BD9B0), core::ptr::addr_of_mut!(mpLib_803BDA60), core::ptr::addr_of_mut!(mpLib_803BDAB8), core::ptr::addr_of_mut!(mpLib_803BDB10), core::ptr::addr_of_mut!(mpLib_803BDB68), core::ptr::addr_of_mut!(mpLib_803BDBC0)];
static mut mpLib_803BECA8: [*mut mpLib_803BF248_t_x4; 20] = [core::ptr::addr_of_mut!(mpLib_803BD3D8), core::ptr::addr_of_mut!(mpLib_803BD430), core::ptr::addr_of_mut!(mpLib_803BD488), core::ptr::addr_of_mut!(mpLib_803BD4E0), core::ptr::addr_of_mut!(mpLib_803BD538), core::ptr::addr_of_mut!(mpLib_803BD590), core::ptr::addr_of_mut!(mpLib_803BD5E8), core::ptr::addr_of_mut!(mpLib_803BD640), core::ptr::addr_of_mut!(mpLib_803BD698), core::ptr::addr_of_mut!(mpLib_803BD6F0), core::ptr::addr_of_mut!(mpLib_803BD748), core::ptr::addr_of_mut!(mpLib_803BD8A8), core::ptr::addr_of_mut!(mpLib_803BD900), core::ptr::addr_of_mut!(mpLib_803BD958), core::ptr::addr_of_mut!(mpLib_803BD9B0), core::ptr::addr_of_mut!(mpLib_803BDA60), core::ptr::addr_of_mut!(mpLib_803BDAB8), core::ptr::addr_of_mut!(mpLib_803BDB10), core::ptr::addr_of_mut!(mpLib_803BDB68), core::ptr::addr_of_mut!(mpLib_803BDBC0)];
static mut mpLib_803BECF8: [*mut mpLib_803BF248_t_x4; 20] = [core::ptr::addr_of_mut!(mpLib_803BD3D8), core::ptr::addr_of_mut!(mpLib_803BD430), core::ptr::addr_of_mut!(mpLib_803BD488), core::ptr::addr_of_mut!(mpLib_803BD4E0), core::ptr::addr_of_mut!(mpLib_803BD538), core::ptr::addr_of_mut!(mpLib_803BD590), core::ptr::addr_of_mut!(mpLib_803BD5E8), core::ptr::addr_of_mut!(mpLib_803BD640), core::ptr::addr_of_mut!(mpLib_803BD698), core::ptr::addr_of_mut!(mpLib_803BD6F0), core::ptr::addr_of_mut!(mpLib_803BD748), core::ptr::addr_of_mut!(mpLib_803BD8A8), core::ptr::addr_of_mut!(mpLib_803BD900), core::ptr::addr_of_mut!(mpLib_803BD958), core::ptr::addr_of_mut!(mpLib_803BD9B0), core::ptr::addr_of_mut!(mpLib_803BDA60), core::ptr::addr_of_mut!(mpLib_803BDAB8), core::ptr::addr_of_mut!(mpLib_803BDB10), core::ptr::addr_of_mut!(mpLib_803BDB68), core::ptr::addr_of_mut!(mpLib_803BDBC0)];
static mut mpLib_803BED48: [*mut mpLib_803BF248_t_x4; 20] = [core::ptr::addr_of_mut!(mpLib_803BD3D8), core::ptr::addr_of_mut!(mpLib_803BD430), core::ptr::addr_of_mut!(mpLib_803BD488), core::ptr::addr_of_mut!(mpLib_803BD4E0), core::ptr::addr_of_mut!(mpLib_803BD538), core::ptr::addr_of_mut!(mpLib_803BD590), core::ptr::addr_of_mut!(mpLib_803BD5E8), core::ptr::addr_of_mut!(mpLib_803BD640), core::ptr::addr_of_mut!(mpLib_803BD698), core::ptr::addr_of_mut!(mpLib_803BD6F0), core::ptr::addr_of_mut!(mpLib_803BD748), core::ptr::addr_of_mut!(mpLib_803BD8A8), core::ptr::addr_of_mut!(mpLib_803BD900), core::ptr::addr_of_mut!(mpLib_803BD958), core::ptr::addr_of_mut!(mpLib_803BD9B0), core::ptr::addr_of_mut!(mpLib_803BDA60), core::ptr::addr_of_mut!(mpLib_803BDAB8), core::ptr::addr_of_mut!(mpLib_803BDB10), core::ptr::addr_of_mut!(mpLib_803BDB68), core::ptr::addr_of_mut!(mpLib_803BDBC0)];
static mut mpLib_803BED98: [*mut mpLib_803BF248_t_x4; 20] = [core::ptr::addr_of_mut!(mpLib_803BD3D8), core::ptr::addr_of_mut!(mpLib_803BD430), core::ptr::addr_of_mut!(mpLib_803BD488), core::ptr::addr_of_mut!(mpLib_803BD4E0), core::ptr::addr_of_mut!(mpLib_803BD538), core::ptr::addr_of_mut!(mpLib_803BD590), core::ptr::addr_of_mut!(mpLib_803BD5E8), core::ptr::addr_of_mut!(mpLib_803BD640), core::ptr::addr_of_mut!(mpLib_803BD698), core::ptr::addr_of_mut!(mpLib_803BD6F0), core::ptr::addr_of_mut!(mpLib_803BD748), core::ptr::addr_of_mut!(mpLib_803BD8A8), core::ptr::addr_of_mut!(mpLib_803BD900), core::ptr::addr_of_mut!(mpLib_803BD958), core::ptr::addr_of_mut!(mpLib_803BD9B0), core::ptr::addr_of_mut!(mpLib_803BDA60), core::ptr::addr_of_mut!(mpLib_803BDAB8), core::ptr::addr_of_mut!(mpLib_803BDB10), core::ptr::addr_of_mut!(mpLib_803BDB68), core::ptr::addr_of_mut!(mpLib_803BDBC0)];
static mut mpLib_803BEDE8: [*mut mpLib_803BF248_t_x4; 20] = [core::ptr::addr_of_mut!(mpLib_803BD3D8), core::ptr::addr_of_mut!(mpLib_803BD430), core::ptr::addr_of_mut!(mpLib_803BD488), core::ptr::addr_of_mut!(mpLib_803BD4E0), core::ptr::addr_of_mut!(mpLib_803BD538), core::ptr::addr_of_mut!(mpLib_803BD590), core::ptr::addr_of_mut!(mpLib_803BD5E8), core::ptr::addr_of_mut!(mpLib_803BD640), core::ptr::addr_of_mut!(mpLib_803BD698), core::ptr::addr_of_mut!(mpLib_803BD6F0), core::ptr::addr_of_mut!(mpLib_803BD748), core::ptr::addr_of_mut!(mpLib_803BD8A8), core::ptr::addr_of_mut!(mpLib_803BD900), core::ptr::addr_of_mut!(mpLib_803BD958), core::ptr::addr_of_mut!(mpLib_803BD9B0), core::ptr::addr_of_mut!(mpLib_803BDA60), core::ptr::addr_of_mut!(mpLib_803BDAB8), core::ptr::addr_of_mut!(mpLib_803BDB10), core::ptr::addr_of_mut!(mpLib_803BDB68), core::ptr::addr_of_mut!(mpLib_803BDBC0)];
static mut mpLib_803BEE38: [*mut mpLib_803BF248_t_x4; 20] = [core::ptr::addr_of_mut!(mpLib_803BD3D8), core::ptr::addr_of_mut!(mpLib_803BD430), core::ptr::addr_of_mut!(mpLib_803BD488), core::ptr::addr_of_mut!(mpLib_803BD4E0), core::ptr::addr_of_mut!(mpLib_803BD538), core::ptr::addr_of_mut!(mpLib_803BD590), core::ptr::addr_of_mut!(mpLib_803BD5E8), core::ptr::addr_of_mut!(mpLib_803BD640), core::ptr::addr_of_mut!(mpLib_803BD698), core::ptr::addr_of_mut!(mpLib_803BD6F0), core::ptr::addr_of_mut!(mpLib_803BD748), core::ptr::addr_of_mut!(mpLib_803BD8A8), core::ptr::addr_of_mut!(mpLib_803BD900), core::ptr::addr_of_mut!(mpLib_803BD958), core::ptr::addr_of_mut!(mpLib_803BD9B0), core::ptr::addr_of_mut!(mpLib_803BDA60), core::ptr::addr_of_mut!(mpLib_803BDAB8), core::ptr::addr_of_mut!(mpLib_803BDB10), core::ptr::addr_of_mut!(mpLib_803BDB68), core::ptr::addr_of_mut!(mpLib_803BDBC0)];
static mut mpLib_803BEE88: [*mut mpLib_803BF248_t_x4; 20] = [core::ptr::addr_of_mut!(mpLib_803BD3D8), core::ptr::addr_of_mut!(mpLib_803BD430), core::ptr::addr_of_mut!(mpLib_803BD488), core::ptr::addr_of_mut!(mpLib_803BD4E0), core::ptr::addr_of_mut!(mpLib_803BD538), core::ptr::addr_of_mut!(mpLib_803BD590), core::ptr::addr_of_mut!(mpLib_803BD5E8), core::ptr::addr_of_mut!(mpLib_803BD640), core::ptr::addr_of_mut!(mpLib_803BD698), core::ptr::addr_of_mut!(mpLib_803BD6F0), core::ptr::addr_of_mut!(mpLib_803BD748), core::ptr::addr_of_mut!(mpLib_803BD8A8), core::ptr::addr_of_mut!(mpLib_803BD900), core::ptr::addr_of_mut!(mpLib_803BD958), core::ptr::addr_of_mut!(mpLib_803BD9B0), core::ptr::addr_of_mut!(mpLib_803BDA60), core::ptr::addr_of_mut!(mpLib_803BDAB8), core::ptr::addr_of_mut!(mpLib_803BDB10), core::ptr::addr_of_mut!(mpLib_803BDB68), core::ptr::addr_of_mut!(mpLib_803BDBC0)];
static mut mpLib_803BEED8: [*mut mpLib_803BF248_t_x4; 20] = [core::ptr::addr_of_mut!(mpLib_803BD3D8), core::ptr::addr_of_mut!(mpLib_803BD430), core::ptr::addr_of_mut!(mpLib_803BD488), core::ptr::addr_of_mut!(mpLib_803BD4E0), core::ptr::addr_of_mut!(mpLib_803BD538), core::ptr::addr_of_mut!(mpLib_803BD590), core::ptr::addr_of_mut!(mpLib_803BD5E8), core::ptr::addr_of_mut!(mpLib_803BD640), core::ptr::addr_of_mut!(mpLib_803BD698), core::ptr::addr_of_mut!(mpLib_803BD6F0), core::ptr::addr_of_mut!(mpLib_803BD748), core::ptr::addr_of_mut!(mpLib_803BD8A8), core::ptr::addr_of_mut!(mpLib_803BD900), core::ptr::addr_of_mut!(mpLib_803BD958), core::ptr::addr_of_mut!(mpLib_803BD9B0), core::ptr::addr_of_mut!(mpLib_803BDA60), core::ptr::addr_of_mut!(mpLib_803BDAB8), core::ptr::addr_of_mut!(mpLib_803BDB10), core::ptr::addr_of_mut!(mpLib_803BDB68), core::ptr::addr_of_mut!(mpLib_803BDBC0)];
static mut mpLib_803BEF28: [*mut mpLib_803BF248_t_x4; 20] = [core::ptr::addr_of_mut!(mpLib_803BD3D8), core::ptr::addr_of_mut!(mpLib_803BD430), core::ptr::addr_of_mut!(mpLib_803BD488), core::ptr::addr_of_mut!(mpLib_803BD4E0), core::ptr::addr_of_mut!(mpLib_803BD538), core::ptr::addr_of_mut!(mpLib_803BD590), core::ptr::addr_of_mut!(mpLib_803BD5E8), core::ptr::addr_of_mut!(mpLib_803BD640), core::ptr::addr_of_mut!(mpLib_803BD698), core::ptr::addr_of_mut!(mpLib_803BD6F0), core::ptr::addr_of_mut!(mpLib_803BD748), core::ptr::addr_of_mut!(mpLib_803BD8A8), core::ptr::addr_of_mut!(mpLib_803BD900), core::ptr::addr_of_mut!(mpLib_803BD958), core::ptr::addr_of_mut!(mpLib_803BD9B0), core::ptr::addr_of_mut!(mpLib_803BDA60), core::ptr::addr_of_mut!(mpLib_803BDAB8), core::ptr::addr_of_mut!(mpLib_803BDB10), core::ptr::addr_of_mut!(mpLib_803BDB68), core::ptr::addr_of_mut!(mpLib_803BDBC0)];
static mut mpLib_803BEF78: [*mut mpLib_803BF248_t_x4; 20] = [core::ptr::addr_of_mut!(mpLib_803BD3D8), core::ptr::addr_of_mut!(mpLib_803BD430), core::ptr::addr_of_mut!(mpLib_803BD488), core::ptr::addr_of_mut!(mpLib_803BD4E0), core::ptr::addr_of_mut!(mpLib_803BD538), core::ptr::addr_of_mut!(mpLib_803BD590), core::ptr::addr_of_mut!(mpLib_803BD5E8), core::ptr::addr_of_mut!(mpLib_803BD640), core::ptr::addr_of_mut!(mpLib_803BD698), core::ptr::addr_of_mut!(mpLib_803BD6F0), core::ptr::addr_of_mut!(mpLib_803BD748), core::ptr::addr_of_mut!(mpLib_803BD8A8), core::ptr::addr_of_mut!(mpLib_803BD900), core::ptr::addr_of_mut!(mpLib_803BD958), core::ptr::addr_of_mut!(mpLib_803BD9B0), core::ptr::addr_of_mut!(mpLib_803BDA60), core::ptr::addr_of_mut!(mpLib_803BDAB8), core::ptr::addr_of_mut!(mpLib_803BDB10), core::ptr::addr_of_mut!(mpLib_803BDB68), core::ptr::addr_of_mut!(mpLib_803BDBC0)];
static mut mpLib_803BEFC8: [*mut mpLib_803BF248_t_x4; 20] = [core::ptr::addr_of_mut!(mpLib_803BD3D8), core::ptr::addr_of_mut!(mpLib_803BD430), core::ptr::addr_of_mut!(mpLib_803BD488), core::ptr::addr_of_mut!(mpLib_803BD4E0), core::ptr::addr_of_mut!(mpLib_803BD538), core::ptr::addr_of_mut!(mpLib_803BD590), core::ptr::addr_of_mut!(mpLib_803BD5E8), core::ptr::addr_of_mut!(mpLib_803BD640), core::ptr::addr_of_mut!(mpLib_803BD698), core::ptr::addr_of_mut!(mpLib_803BD6F0), core::ptr::addr_of_mut!(mpLib_803BD748), core::ptr::addr_of_mut!(mpLib_803BD8A8), core::ptr::addr_of_mut!(mpLib_803BD900), core::ptr::addr_of_mut!(mpLib_803BD958), core::ptr::addr_of_mut!(mpLib_803BD9B0), core::ptr::addr_of_mut!(mpLib_803BDA60), core::ptr::addr_of_mut!(mpLib_803BDAB8), core::ptr::addr_of_mut!(mpLib_803BDB10), core::ptr::addr_of_mut!(mpLib_803BDB68), core::ptr::addr_of_mut!(mpLib_803BDBC0)];
static mut mpLib_803BF018: [*mut mpLib_803BF248_t_x4; 20] = [core::ptr::addr_of_mut!(mpLib_803BD3D8), core::ptr::addr_of_mut!(mpLib_803BD430), core::ptr::addr_of_mut!(mpLib_803BD488), core::ptr::addr_of_mut!(mpLib_803BD4E0), core::ptr::addr_of_mut!(mpLib_803BD538), core::ptr::addr_of_mut!(mpLib_803BD590), core::ptr::addr_of_mut!(mpLib_803BD5E8), core::ptr::addr_of_mut!(mpLib_803BD640), core::ptr::addr_of_mut!(mpLib_803BD698), core::ptr::addr_of_mut!(mpLib_803BD6F0), core::ptr::addr_of_mut!(mpLib_803BD748), core::ptr::addr_of_mut!(mpLib_803BD8A8), core::ptr::addr_of_mut!(mpLib_803BD900), core::ptr::addr_of_mut!(mpLib_803BD958), core::ptr::addr_of_mut!(mpLib_803BD9B0), core::ptr::addr_of_mut!(mpLib_803BDA60), core::ptr::addr_of_mut!(mpLib_803BDAB8), core::ptr::addr_of_mut!(mpLib_803BDB10), core::ptr::addr_of_mut!(mpLib_803BDB68), core::ptr::addr_of_mut!(mpLib_803BDBC0)];
static mut mpLib_803BF068: [*mut mpLib_803BF248_t_x4; 20] = [core::ptr::addr_of_mut!(mpLib_803BD3D8), core::ptr::addr_of_mut!(mpLib_803BD430), core::ptr::addr_of_mut!(mpLib_803BD488), core::ptr::addr_of_mut!(mpLib_803BD4E0), core::ptr::addr_of_mut!(mpLib_803BD538), core::ptr::addr_of_mut!(mpLib_803BD590), core::ptr::addr_of_mut!(mpLib_803BD5E8), core::ptr::addr_of_mut!(mpLib_803BD640), core::ptr::addr_of_mut!(mpLib_803BD698), core::ptr::addr_of_mut!(mpLib_803BD6F0), core::ptr::addr_of_mut!(mpLib_803BD748), core::ptr::addr_of_mut!(mpLib_803BD8A8), core::ptr::addr_of_mut!(mpLib_803BD900), core::ptr::addr_of_mut!(mpLib_803BD958), core::ptr::addr_of_mut!(mpLib_803BD9B0), core::ptr::addr_of_mut!(mpLib_803BDA60), core::ptr::addr_of_mut!(mpLib_803BDAB8), core::ptr::addr_of_mut!(mpLib_803BDB10), core::ptr::addr_of_mut!(mpLib_803BDB68), core::ptr::addr_of_mut!(mpLib_803BDBC0)];
static mut mpLib_803BF0B8: [*mut mpLib_803BF248_t_x4; 20] = [core::ptr::addr_of_mut!(mpLib_803BD3D8), core::ptr::addr_of_mut!(mpLib_803BD430), core::ptr::addr_of_mut!(mpLib_803BD488), core::ptr::addr_of_mut!(mpLib_803BD4E0), core::ptr::addr_of_mut!(mpLib_803BD538), core::ptr::addr_of_mut!(mpLib_803BD590), core::ptr::addr_of_mut!(mpLib_803BD5E8), core::ptr::addr_of_mut!(mpLib_803BD640), core::ptr::addr_of_mut!(mpLib_803BD698), core::ptr::addr_of_mut!(mpLib_803BD6F0), core::ptr::addr_of_mut!(mpLib_803BD748), core::ptr::addr_of_mut!(mpLib_803BD8A8), core::ptr::addr_of_mut!(mpLib_803BD900), core::ptr::addr_of_mut!(mpLib_803BD958), core::ptr::addr_of_mut!(mpLib_803BD9B0), core::ptr::addr_of_mut!(mpLib_803BDA60), core::ptr::addr_of_mut!(mpLib_803BDAB8), core::ptr::addr_of_mut!(mpLib_803BDB10), core::ptr::addr_of_mut!(mpLib_803BDB68), core::ptr::addr_of_mut!(mpLib_803BDBC0)];
static mut mpLib_803BF108: [*mut mpLib_803BF248_t_x4; 20] = [core::ptr::addr_of_mut!(mpLib_803BD3D8), core::ptr::addr_of_mut!(mpLib_803BD430), core::ptr::addr_of_mut!(mpLib_803BD488), core::ptr::addr_of_mut!(mpLib_803BD4E0), core::ptr::addr_of_mut!(mpLib_803BD538), core::ptr::addr_of_mut!(mpLib_803BD590), core::ptr::addr_of_mut!(mpLib_803BD5E8), core::ptr::addr_of_mut!(mpLib_803BD640), core::ptr::addr_of_mut!(mpLib_803BD698), core::ptr::addr_of_mut!(mpLib_803BD6F0), core::ptr::addr_of_mut!(mpLib_803BD748), core::ptr::addr_of_mut!(mpLib_803BD8A8), core::ptr::addr_of_mut!(mpLib_803BD900), core::ptr::addr_of_mut!(mpLib_803BD958), core::ptr::addr_of_mut!(mpLib_803BD9B0), core::ptr::addr_of_mut!(mpLib_803BDA60), core::ptr::addr_of_mut!(mpLib_803BDAB8), core::ptr::addr_of_mut!(mpLib_803BDB10), core::ptr::addr_of_mut!(mpLib_803BDB68), core::ptr::addr_of_mut!(mpLib_803BDBC0)];
static mut mpLib_803BF158: [*mut mpLib_803BF248_t_x4; 20] = [core::ptr::addr_of_mut!(mpLib_803BD3D8), core::ptr::addr_of_mut!(mpLib_803BD430), core::ptr::addr_of_mut!(mpLib_803BD488), core::ptr::addr_of_mut!(mpLib_803BD4E0), core::ptr::addr_of_mut!(mpLib_803BD538), core::ptr::addr_of_mut!(mpLib_803BD590), core::ptr::addr_of_mut!(mpLib_803BD5E8), core::ptr::addr_of_mut!(mpLib_803BD640), core::ptr::addr_of_mut!(mpLib_803BD698), core::ptr::addr_of_mut!(mpLib_803BD6F0), core::ptr::addr_of_mut!(mpLib_803BD748), core::ptr::addr_of_mut!(mpLib_803BD8A8), core::ptr::addr_of_mut!(mpLib_803BD900), core::ptr::addr_of_mut!(mpLib_803BD958), core::ptr::addr_of_mut!(mpLib_803BD9B0), core::ptr::addr_of_mut!(mpLib_803BDA60), core::ptr::addr_of_mut!(mpLib_803BDAB8), core::ptr::addr_of_mut!(mpLib_803BDB10), core::ptr::addr_of_mut!(mpLib_803BDB68), core::ptr::addr_of_mut!(mpLib_803BDBC0)];
static mut mpLib_803BF1A8: [*mut mpLib_803BF248_t_x4; 20] = [core::ptr::addr_of_mut!(mpLib_803BD3D8), core::ptr::addr_of_mut!(mpLib_803BD430), core::ptr::addr_of_mut!(mpLib_803BD488), core::ptr::addr_of_mut!(mpLib_803BD4E0), core::ptr::addr_of_mut!(mpLib_803BD538), core::ptr::addr_of_mut!(mpLib_803BD590), core::ptr::addr_of_mut!(mpLib_803BD5E8), core::ptr::addr_of_mut!(mpLib_803BD640), core::ptr::addr_of_mut!(mpLib_803BD698), core::ptr::addr_of_mut!(mpLib_803BD6F0), core::ptr::addr_of_mut!(mpLib_803BD748), core::ptr::addr_of_mut!(mpLib_803BD8A8), core::ptr::addr_of_mut!(mpLib_803BD900), core::ptr::addr_of_mut!(mpLib_803BD958), core::ptr::addr_of_mut!(mpLib_803BD9B0), core::ptr::addr_of_mut!(mpLib_803BDA60), core::ptr::addr_of_mut!(mpLib_803BDAB8), core::ptr::addr_of_mut!(mpLib_803BDB10), core::ptr::addr_of_mut!(mpLib_803BDB68), core::ptr::addr_of_mut!(mpLib_803BDBC0)];
static mut mpLib_803BF1F8: [*mut mpLib_803BF248_t_x4; 20] = [core::ptr::addr_of_mut!(mpLib_803BD3D8), core::ptr::addr_of_mut!(mpLib_803BD430), core::ptr::addr_of_mut!(mpLib_803BD488), core::ptr::addr_of_mut!(mpLib_803BD4E0), core::ptr::addr_of_mut!(mpLib_803BD538), core::ptr::addr_of_mut!(mpLib_803BD590), core::ptr::addr_of_mut!(mpLib_803BD5E8), core::ptr::addr_of_mut!(mpLib_803BD640), core::ptr::addr_of_mut!(mpLib_803BD698), core::ptr::addr_of_mut!(mpLib_803BD6F0), core::ptr::addr_of_mut!(mpLib_803BD748), core::ptr::addr_of_mut!(mpLib_803BD8A8), core::ptr::addr_of_mut!(mpLib_803BD900), core::ptr::addr_of_mut!(mpLib_803BD958), core::ptr::addr_of_mut!(mpLib_803BD9B0), core::ptr::addr_of_mut!(mpLib_803BDA60), core::ptr::addr_of_mut!(mpLib_803BDAB8), core::ptr::addr_of_mut!(mpLib_803BDB10), core::ptr::addr_of_mut!(mpLib_803BDB68), core::ptr::addr_of_mut!(mpLib_803BDBC0)];
static mut mpLib_803BF248: [mpLib_803BF248_t; 71] = [mpLib_803BF248_t { id: 0_u32, x4: core::ptr::addr_of_mut!(mpLib_803BDC18) }, mpLib_803BF248_t { id: 1_u32, x4: core::ptr::addr_of_mut!(mpLib_803BDC68) }, mpLib_803BF248_t { id: 2_u32, x4: core::ptr::addr_of_mut!(mpLib_803BDCB8) }, mpLib_803BF248_t { id: 3_u32, x4: core::ptr::addr_of_mut!(mpLib_803BDD08) }, mpLib_803BF248_t { id: 4_u32, x4: core::ptr::addr_of_mut!(mpLib_803BDD58) }, mpLib_803BF248_t { id: 5_u32, x4: core::ptr::addr_of_mut!(mpLib_803BDDA8) }, mpLib_803BF248_t { id: 6_u32, x4: core::ptr::addr_of_mut!(mpLib_803BDDF8) }, mpLib_803BF248_t { id: 7_u32, x4: core::ptr::addr_of_mut!(mpLib_803BDE48) }, mpLib_803BF248_t { id: 8_u32, x4: core::ptr::addr_of_mut!(mpLib_803BDE98) }, mpLib_803BF248_t { id: 9_u32, x4: core::ptr::addr_of_mut!(mpLib_803BDEE8) }, mpLib_803BF248_t { id: 10_u32, x4: core::ptr::addr_of_mut!(mpLib_803BDF38) }, mpLib_803BF248_t { id: 11_u32, x4: core::ptr::addr_of_mut!(mpLib_803BDF88) }, mpLib_803BF248_t { id: 12_u32, x4: core::ptr::addr_of_mut!(mpLib_803BDFD8) }, mpLib_803BF248_t { id: 13_u32, x4: core::ptr::addr_of_mut!(mpLib_803BE028) }, mpLib_803BF248_t { id: 14_u32, x4: core::ptr::addr_of_mut!(mpLib_803BE078) }, mpLib_803BF248_t { id: 15_u32, x4: core::ptr::addr_of_mut!(mpLib_803BE0C8) }, mpLib_803BF248_t { id: 16_u32, x4: core::ptr::addr_of_mut!(mpLib_803BE118) }, mpLib_803BF248_t { id: 17_u32, x4: core::ptr::addr_of_mut!(mpLib_803BE168) }, mpLib_803BF248_t { id: 18_u32, x4: core::ptr::addr_of_mut!(mpLib_803BE1B8) }, mpLib_803BF248_t { id: 19_u32, x4: core::ptr::addr_of_mut!(mpLib_803BE208) }, mpLib_803BF248_t { id: 20_u32, x4: core::ptr::addr_of_mut!(mpLib_803BE258) }, mpLib_803BF248_t { id: 21_u32, x4: core::ptr::addr_of_mut!(mpLib_803BE2A8) }, mpLib_803BF248_t { id: 22_u32, x4: core::ptr::addr_of_mut!(mpLib_803BE2F8) }, mpLib_803BF248_t { id: 23_u32, x4: core::ptr::addr_of_mut!(mpLib_803BE348) }, mpLib_803BF248_t { id: 24_u32, x4: core::ptr::addr_of_mut!(mpLib_803BE398) }, mpLib_803BF248_t { id: 25_u32, x4: core::ptr::addr_of_mut!(mpLib_803BE3E8) }, mpLib_803BF248_t { id: 26_u32, x4: core::ptr::addr_of_mut!(mpLib_803BE438) }, mpLib_803BF248_t { id: 27_u32, x4: core::ptr::addr_of_mut!(mpLib_803BE488) }, mpLib_803BF248_t { id: 28_u32, x4: core::ptr::addr_of_mut!(mpLib_803BE4D8) }, mpLib_803BF248_t { id: 29_u32, x4: core::ptr::addr_of_mut!(mpLib_803BE528) }, mpLib_803BF248_t { id: 30_u32, x4: core::ptr::addr_of_mut!(mpLib_803BE578) }, mpLib_803BF248_t { id: 31_u32, x4: core::ptr::addr_of_mut!(mpLib_803BE5C8) }, mpLib_803BF248_t { id: 32_u32, x4: core::ptr::addr_of_mut!(mpLib_803BE618) }, mpLib_803BF248_t { id: 33_u32, x4: core::ptr::addr_of_mut!(mpLib_803BE668) }, mpLib_803BF248_t { id: 34_u32, x4: core::ptr::addr_of_mut!(mpLib_803BE6B8) }, mpLib_803BF248_t { id: 35_u32, x4: core::ptr::addr_of_mut!(mpLib_803BE708) }, mpLib_803BF248_t { id: 36_u32, x4: core::ptr::addr_of_mut!(mpLib_803BE758) }, mpLib_803BF248_t { id: 37_u32, x4: core::ptr::addr_of_mut!(mpLib_803BE7A8) }, mpLib_803BF248_t { id: 38_u32, x4: core::ptr::addr_of_mut!(mpLib_803BE7F8) }, mpLib_803BF248_t { id: 39_u32, x4: core::ptr::addr_of_mut!(mpLib_803BE848) }, mpLib_803BF248_t { id: 40_u32, x4: core::ptr::addr_of_mut!(mpLib_803BE898) }, mpLib_803BF248_t { id: 41_u32, x4: core::ptr::addr_of_mut!(mpLib_803BE8E8) }, mpLib_803BF248_t { id: 42_u32, x4: core::ptr::addr_of_mut!(mpLib_803BE938) }, mpLib_803BF248_t { id: 43_u32, x4: core::ptr::addr_of_mut!(mpLib_803BE988) }, mpLib_803BF248_t { id: 44_u32, x4: core::ptr::addr_of_mut!(mpLib_803BE9D8) }, mpLib_803BF248_t { id: 45_u32, x4: core::ptr::addr_of_mut!(mpLib_803BEA28) }, mpLib_803BF248_t { id: 46_u32, x4: core::ptr::addr_of_mut!(mpLib_803BEA78) }, mpLib_803BF248_t { id: 47_u32, x4: core::ptr::addr_of_mut!(mpLib_803BEAC8) }, mpLib_803BF248_t { id: 48_u32, x4: core::ptr::addr_of_mut!(mpLib_803BEB18) }, mpLib_803BF248_t { id: 49_u32, x4: core::ptr::addr_of_mut!(mpLib_803BEB68) }, mpLib_803BF248_t { id: 50_u32, x4: core::ptr::addr_of_mut!(mpLib_803BEBB8) }, mpLib_803BF248_t { id: 51_u32, x4: core::ptr::addr_of_mut!(mpLib_803BEC08) }, mpLib_803BF248_t { id: 52_u32, x4: core::ptr::addr_of_mut!(mpLib_803BEC58) }, mpLib_803BF248_t { id: 53_u32, x4: core::ptr::addr_of_mut!(mpLib_803BECA8) }, mpLib_803BF248_t { id: 54_u32, x4: core::ptr::addr_of_mut!(mpLib_803BECF8) }, mpLib_803BF248_t { id: 55_u32, x4: core::ptr::addr_of_mut!(mpLib_803BED48) }, mpLib_803BF248_t { id: 56_u32, x4: core::ptr::addr_of_mut!(mpLib_803BED98) }, mpLib_803BF248_t { id: 57_u32, x4: core::ptr::addr_of_mut!(mpLib_803BEDE8) }, mpLib_803BF248_t { id: 58_u32, x4: core::ptr::addr_of_mut!(mpLib_803BEE38) }, mpLib_803BF248_t { id: 59_u32, x4: core::ptr::addr_of_mut!(mpLib_803BEE88) }, mpLib_803BF248_t { id: 60_u32, x4: core::ptr::addr_of_mut!(mpLib_803BEED8) }, mpLib_803BF248_t { id: 61_u32, x4: core::ptr::addr_of_mut!(mpLib_803BEF28) }, mpLib_803BF248_t { id: 62_u32, x4: core::ptr::addr_of_mut!(mpLib_803BEF78) }, mpLib_803BF248_t { id: 63_u32, x4: core::ptr::addr_of_mut!(mpLib_803BEFC8) }, mpLib_803BF248_t { id: 64_u32, x4: core::ptr::addr_of_mut!(mpLib_803BF018) }, mpLib_803BF248_t { id: 65_u32, x4: core::ptr::addr_of_mut!(mpLib_803BF068) }, mpLib_803BF248_t { id: 66_u32, x4: core::ptr::addr_of_mut!(mpLib_803BF0B8) }, mpLib_803BF248_t { id: 67_u32, x4: core::ptr::addr_of_mut!(mpLib_803BF108) }, mpLib_803BF248_t { id: 68_u32, x4: core::ptr::addr_of_mut!(mpLib_803BF158) }, mpLib_803BF248_t { id: 69_u32, x4: core::ptr::addr_of_mut!(mpLib_803BF1A8) }, mpLib_803BF248_t { id: 70_u32, x4: core::ptr::addr_of_mut!(mpLib_803BF1F8) }];
static mut lbl_803BF526: [i8; 24] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
static mut mpLib_803BF540: HSD_Chan = HSD_Chan { next: null_mut(), chan: 4_u32, flags: 0_u32, amb_color: GXColor { r: 0_u8, g: 0_u8, b: 0_u8, a: 0_u8 }, mat_color: GXColor { r: 255_u8, g: 160_u8, b: 48_u8, a: 255_u8 }, enable: 0_u8, amb_src: 0_u32, mat_src: 0_u32, light_mask: 0_u32, diff_fn: 2_u32, attn_fn: 2_u32, aobj: null_mut() };
static mut lbl_803BF570: [i8; 18] = [66, 40, 37, 100, 44, 37, 100, 41, 45, 40, 37, 100, 44, 37, 100, 41, 10, 0];
static mpLib_804D80E0: _GXColor = _GXColor { r: 255_u8, g: 55_u8, b: 55_u8, a: 128_u8 };
static mpLib_804D80E4: _GXColor = _GXColor { r: 55_u8, g: 55_u8, b: 255_u8, a: 128_u8 };
static mpLib_804D80F0: _GXColor = _GXColor { r: 255_u8, g: 64_u8, b: 64_u8, a: 255_u8 };
static mpLib_804D80F4: _GXColor = _GXColor { r: 64_u8, g: 64_u8, b: 255_u8, a: 255_u8 };
static mpLib_804D80F8: _GXColor = _GXColor { r: 255_u8, g: 64_u8, b: 255_u8, a: 255_u8 };
static mpLib_804D80FC: _GXColor = _GXColor { r: 128_u8, g: 128_u8, b: 128_u8, a: 255_u8 };
static mpLib_804D8100: _GXColor = _GXColor { r: 128_u8, g: 128_u8, b: 128_u8, a: 255_u8 };
static mpLib_FloorColor: _GXColor = _GXColor { r: 192_u8, g: 192_u8, b: 192_u8, a: 255_u8 };
static mpLib_CeilingColor: _GXColor = _GXColor { r: 192_u8, g: 128_u8, b: 128_u8, a: 255_u8 };
static mpLib_RightWallColor: _GXColor = _GXColor { r: 128_u8, g: 192_u8, b: 128_u8, a: 255_u8 };
static mpLib_LeftWallColor: _GXColor = _GXColor { r: 128_u8, g: 128_u8, b: 192_u8, a: 255_u8 };
static mpLib_DynamicFloorColor: _GXColor = _GXColor { r: 96_u8, g: 96_u8, b: 96_u8, a: 255_u8 };
static mpLib_DynamicCeilingColor: _GXColor = _GXColor { r: 96_u8, g: 64_u8, b: 64_u8, a: 255_u8 };
static mpLib_DynamicRightWallColor: _GXColor = _GXColor { r: 64_u8, g: 96_u8, b: 64_u8, a: 255_u8 };
static mpLib_DynamicLeftWallColor: _GXColor = _GXColor { r: 64_u8, g: 64_u8, b: 96_u8, a: 255_u8 };
static mpLib_804D8128: _GXColor = _GXColor { r: 125_u8, g: 125_u8, b: 255_u8, a: 255_u8 };
static mpLib_804D812C: _GXColor = _GXColor { r: 125_u8, g: 255_u8, b: 128_u8, a: 255_u8 };
static mpLib_804D8130: _GXColor = _GXColor { r: 255_u8, g: 255_u8, b: 128_u8, a: 255_u8 };
static mpLib_804D8134: _GXColor = _GXColor { r: 255_u8, g: 64_u8, b: 64_u8, a: 255_u8 };
static mpLib_804D8138: _GXColor = _GXColor { r: 255_u8, g: 64_u8, b: 192_u8, a: 255_u8 };
static mpLib_804D813C: _GXColor = _GXColor { r: 255_u8, g: 255_u8, b: 255_u8, a: 255_u8 };
static mut mpLib_ItemSpawnVtxIds: [i16; 174] = [127_i16, 128_i16, 129_i16, 130_i16, 131_i16, 132_i16, 133_i16, 134_i16, 135_i16, 136_i16, 137_i16, 138_i16, 139_i16, 140_i16, 141_i16, 142_i16, 143_i16, 144_i16, 145_i16, 146_i16, 147_i16, 0_i16, 32_i16, 33_i16, 34_i16, 35_i16, 36_i16, 37_i16, 38_i16, 39_i16, 40_i16, 41_i16, 42_i16, 43_i16, 44_i16, 45_i16, 46_i16, 47_i16, 48_i16, 49_i16, 50_i16, 51_i16, 52_i16, 53_i16, 54_i16, 55_i16, 56_i16, 57_i16, 58_i16, 59_i16, 60_i16, 61_i16, 62_i16, 63_i16, 64_i16, 65_i16, 66_i16, 67_i16, 68_i16, 69_i16, 70_i16, 71_i16, 72_i16, 73_i16, 74_i16, 75_i16, 76_i16, 77_i16, 78_i16, 79_i16, 80_i16, 81_i16, 82_i16, 83_i16, 84_i16, 85_i16, 86_i16, 87_i16, 88_i16, 89_i16, 90_i16, 91_i16, 92_i16, 93_i16, 94_i16, 95_i16, 96_i16, 97_i16, 98_i16, 99_i16, 100_i16, 101_i16, 102_i16, 103_i16, 104_i16, 105_i16, 106_i16, 107_i16, 108_i16, 109_i16, 110_i16, 111_i16, 220_i16, 221_i16, 222_i16, 223_i16, 224_i16, 225_i16, 226_i16, 227_i16, 228_i16, 229_i16, 230_i16, 231_i16, 232_i16, 233_i16, 234_i16, 235_i16, 236_i16, 237_i16, 238_i16, 239_i16, 240_i16, 241_i16, 242_i16, 243_i16, 244_i16, 0_i16, 153_i16, 154_i16, 155_i16, 156_i16, 157_i16, 158_i16, 159_i16, 160_i16, 161_i16, 162_i16, 163_i16, 164_i16, 165_i16, 166_i16, 167_i16, 168_i16, 169_i16, 170_i16, 171_i16, 172_i16, 173_i16, 174_i16, 175_i16, 176_i16, 177_i16, 178_i16, 179_i16, 180_i16, 181_i16, 182_i16, 183_i16, 184_i16, 185_i16, 186_i16, 187_i16, 188_i16, 189_i16, 190_i16, 191_i16, 192_i16, 193_i16, 194_i16, 195_i16, 196_i16, 197_i16, 198_i16];
pub static mut mpLib_803BF6E0: [i8; 53] = [109, 97, 112, 32, 99, 111, 108, 108, 32, 117, 110, 100, 101, 114, 61, 37, 100, 32, 117, 112, 112, 101, 114, 61, 37, 100, 32, 108, 101, 102, 116, 61, 37, 100, 32, 114, 105, 103, 104, 116, 61, 37, 100, 32, 98, 98, 111, 120, 61, 37, 100, 10, 0];
static mut mpLib_SpawnVtxIds: [i16; 4] = [0_i16, 1_i16, 2_i16, 3_i16];
pub static mut mpLib_RespawnVtxIds: [i16; 6] = [4_i16, 5_i16, 6_i16, 7_i16, 0_i16, 0_i16];
pub static mut mpLib_803BF718: [Vec2; 2] = [Vec2 { x: -1.0f32, y: -4e+02f32 }, Vec2 { x: 1.0f32, y: -4e+02f32 }];
pub static mut mpLib_803BF728: MapLine = MapLine { v0_idx: 0_u16, v1_idx: 1_u16, prev_id0: -1_i16, next_id0: -1_i16, prev_id1: -1_i16, next_id1: -1_i16, hi_flags: 1_u16, lo_flags: 0_u16 };
pub static mut mpLib_803BF738: MapJoint = MapJoint { ranges: [MapLineRange { start: 0_i16, count: 1_i16 }, MapLineRange { start: 0_i16, count: 0_i16 }, MapLineRange { start: 0_i16, count: 0_i16 }, MapLineRange { start: 0_i16, count: 0_i16 }, MapLineRange { start: 0_i16, count: 0_i16 }], left_bound: -9.0f32, bottom_bound: -408.0f32, right_bound: 9.0f32, top_bound: -392.0f32, vtx_start: 0_i16, vtx_count: 2_i16 };
pub static mut mpLib_803BF760: MapCollData = MapCollData { verts: core::ptr::addr_of_mut!(mpLib_803BF718) as *mut Vec2, vert_count: 2, lines: core::ptr::addr_of_mut!(mpLib_803BF728), line_count: 1, ranges: [MapLineRange { start: 0_i16, count: 1_i16 }, MapLineRange { start: 0_i16, count: 0_i16 }, MapLineRange { start: 0_i16, count: 0_i16 }, MapLineRange { start: 0_i16, count: 0_i16 }, MapLineRange { start: 0_i16, count: 0_i16 }], joints: core::ptr::addr_of_mut!(mpLib_803BF738), joint_count: 1, x2C: 0 };
static mpLib_804D8140: _GXColor = _GXColor { r: 255_u8, g: 255_u8, b: 192_u8, a: 255_u8 };
static mpLib_804D8144: _GXColor = _GXColor { r: 255_u8, g: 255_u8, b: 255_u8, a: 128_u8 };
static mpLib_804D8148: _GXColor = _GXColor { r: 128_u8, g: 192_u8, b: 192_u8, a: 255_u8 };
static mpLib_804D814C: _GXColor = _GXColor { r: 192_u8, g: 255_u8, b: 255_u8, a: 128_u8 };
static mut mpLib_804D8150: [_GXColor; 2] = [_GXColor { r: 64_u8, g: 64_u8, b: 255_u8, a: 0_u8 }, _GXColor { r: 0_u8, g: 0_u8, b: 0_u8, a: 0_u8 }];

/// `mpScriptInvalidateBounding`
pub unsafe fn mpScriptInvalidateBounding() {
    didCheckBounding = 0;
}

/// `mpScriptBatchBegin`
pub unsafe fn mpScriptBatchBegin() {
    scriptBatchDepth = scriptBatchDepth + 1;
}

/// `mpScriptBatchEnd`
pub unsafe fn mpScriptBatchEnd() {
    if (scriptBatchDepth <= 0) || ({ scriptBatchDepth = scriptBatchDepth - 1; scriptBatchDepth } != 0) {
        return;
    }
    if !mpLib_804D64B4.is_null() {
        super::mpisland::mpIsland_ScriptBatch((scriptBatchDirty.as_mut_ptr()) as *const u8, (*mpLib_804D64B4).joint_count);
    }
    let _ = core::ptr::write_bytes(((scriptBatchDirty.as_mut_ptr()) as *mut c_void) as *mut u8, 0 as u8, (1024_u64 as u32) as usize);
    mpScriptInvalidateBounding();
}

/// `mpScriptIslandRefresh`
unsafe fn mpScriptIslandRefresh(mut joint: i32, mut start: i32, mut count: i32, mut enabled: i32) {
    if (((scriptBatchDepth != 0) && (joint >= 0)) && (joint < 1024)) && (ext::ScriptGame_StageJoint(joint) != 0) {
        scriptBatchDirty[joint as usize] = 1 as u8;
        return;
    }
    super::mpisland::mpIsland_8005B334(joint, start, count, enabled);
}

/// `mpLib_8004D164`
pub unsafe fn mpLib_8004D164() -> *mut MapCollData {
    return mpLib_804D64B4;
}

/// `mpGetGroundCollVtx`
pub unsafe fn mpGetGroundCollVtx() -> *mut CollVtx {
    return groundCollVtx;
}

/// `mpGetGroundCollLine`
pub unsafe fn mpGetGroundCollLine() -> *mut CollLine {
    return groundCollLine;
}

/// `mpGetGroundCollJoint`
pub unsafe fn mpGetGroundCollJoint() -> *mut CollJoint {
    return groundCollJoint;
}

/// `mpPruneEmptyLines`
pub unsafe fn mpPruneEmptyLines(mut coll_data: *mut MapCollData) {
    let mut line: *mut MapLine = null_mut();
    let mut verts: *mut Vec2 = (*coll_data).verts;
    let mut i: i32 = 0;
    if ext::stage_info.grkind == ((Gr_Kind_Pura as i32) as u32) {
        return;
    }
    line = (*coll_data).lines;
    {
        i = 0;
        'l172: while i < (*coll_data).line_count {
            'c172: {
                let mut v0: *mut Vec2 = core::ptr::addr_of_mut!(*verts.offset((*line).v0_idx as isize)) as *mut Vec2;
                let mut v1: *mut Vec2 = core::ptr::addr_of_mut!(*verts.offset((*line).v1_idx as isize)) as *mut Vec2;
                let mut other: *mut MapLine = null_mut();
                let mut j: i32 = 0;
                if ((*v0).x != (*v1).x) || ((*v0).y != (*v1).y) {
                    break 'c172;
                }
                other = (*coll_data).lines;
                {
                    j = 0;
                    'l173: while j < (*coll_data).line_count {
                        if ((*other).prev_id0 as i32) == i {
                            (*other).prev_id0 = (*line).prev_id0;
                        }
                        if ((*other).next_id0 as i32) == i {
                            (*other).next_id0 = (*line).next_id0;
                        }
                        if ((*other).prev_id1 as i32) == i {
                            (*other).prev_id1 = (*line).prev_id0;
                        }
                        if ((*other).next_id1 as i32) == i {
                            (*other).next_id1 = (*line).next_id0;
                        }
                        j = j + 1; other = other.offset(1);
                    }
                }
                (*line).hi_flags = (((*line).hi_flags as i32) | (1 << 7)) as u16;
                (*line).prev_id0 = (-1_i32) as i16;
                (*line).next_id0 = (-1_i32) as i16;
                (*line).prev_id1 = (-1_i32) as i16;
                (*line).next_id1 = (-1_i32) as i16;
            }
            i = i + 1; line = line.offset(1);
        }
    }
}

/// `mpLibLoad`
pub unsafe fn mpLibLoad(mut coll_data: *mut MapCollData) {
    let mut f0: f32 = 0.0;
    let mut f1: f32 = 0.0;
    let mut f2: f32 = 0.0;
    let mut f31: f32 = 0.0;
    let mut joint_prev: *mut CollJoint = null_mut();
    let mut joint: *mut CollJoint = null_mut();
    let mut floor_start: i32 = 0;
    let mut floor_count: i32 = 0;
    let mut ceiling_start: i32 = 0;
    let mut ceiling_count: i32 = 0;
    let mut right_wall_start: i32 = 0;
    let mut right_wall_count: i32 = 0;
    let mut left_wall_start: i32 = 0;
    let mut left_wall_count: i32 = 0;
    let mut dynamic_start: i32 = 0;
    let mut dynamic_count: i32 = 0;
    let mut i: i32 = 0;
    joint_prev = null_mut();
    groundCollVtx = ext::HSD_MemAlloc(49152_u64 as i32) as *mut CollVtx;
    if !groundCollVtx.is_null() { (); } else { panic!("mpLibLoad: assertion failed"); }
    groundCollLine = ext::HSD_MemAlloc(24576_u64 as i32) as *mut CollLine;
    if !groundCollLine.is_null() { (); } else { panic!("mpLibLoad: assertion failed"); }
    {
        groundCollJoint = ext::HSD_MemAlloc((80_u64 * (ext::ScriptGame_StageJointCapacity() as u64)) as i32) as *mut CollJoint;
    }
    if !groundCollJoint.is_null() { (); } else { panic!("mpLibLoad: assertion failed"); }
    ext::grDynamicAttr_801CA0B4();
    if coll_data.is_null() {
        coll_data = core::ptr::addr_of_mut!(mpLib_803BF760) as *mut MapCollData;
    }
    f31 = ext::Ground_801C0498();
    mpLib_80458868[0 as usize].right = f32::MAX;
    mpLib_80458868[0 as usize].top = f32::MAX;
    mpLib_80458868[0 as usize].left = -f32::MAX;
    mpLib_80458868[0 as usize].bottom = -f32::MAX;
    {
        i = 0;
        'l174: while i < (*coll_data).joint_count {
            joint = core::ptr::addr_of_mut!(*groundCollJoint.offset(i as isize)) as *mut CollJoint;
            (*joint).inner = core::ptr::addr_of_mut!(*(*coll_data).joints.offset(i as isize)) as *mut MapJoint;
            (*joint).flags = (CollJoint_Enabled as i32) as u32;
            (*joint).bounding_min.x = f31 * (*(*coll_data).joints.offset(i as isize)).left_bound;
            (*joint).bounding_min.y = f31 * (*(*coll_data).joints.offset(i as isize)).bottom_bound;
            (*joint).bounding_max.x = f31 * (*(*coll_data).joints.offset(i as isize)).right_bound;
            (*joint).bounding_max.y = f31 * (*(*coll_data).joints.offset(i as isize)).top_bound;
            (*joint).x20 = null_mut();
            (*joint).cb_data_0 = null_mut();
            (*joint).cb_0 = None;
            (*joint).cb_data_1 = null_mut();
            (*joint).cb_1 = None;
            (*joint).set_xE((1 as u8) as u8);
            if joint_prev.is_null() {
                jointListStart = joint;
            } else {
                (*joint_prev).next = joint;
            }
            joint_prev = joint;
            i = i + 1;
        }
    }
    (*joint).next = null_mut();
    jointListEnd = joint;
    mpPruneEmptyLines(coll_data);
    floor_count = (*coll_data).ranges[(MapLineGroup_Floor as i32) as usize].count as i32;
    floor_start = (*coll_data).ranges[(MapLineGroup_Floor as i32) as usize].start as i32;
    'l175: while floor_count > 0 {
        (*groundCollLine.offset(floor_start as isize)).flags = (((*(*coll_data).lines.offset(floor_start as isize)).hi_flags as i32) | (1 << 16)) as u32;
        (*groundCollLine.offset(floor_start as isize)).x0 = core::ptr::addr_of_mut!(*(*coll_data).lines.offset(floor_start as isize)) as *mut MapLine;
        floor_start = floor_start + 1;
        floor_count = floor_count - 1;
    }
    ceiling_count = (*coll_data).ranges[(MapLineGroup_Ceiling as i32) as usize].count as i32;
    ceiling_start = (*coll_data).ranges[(MapLineGroup_Ceiling as i32) as usize].start as i32;
    'l176: while ceiling_count > 0 {
        (*groundCollLine.offset(ceiling_start as isize)).flags = (((*(*coll_data).lines.offset(ceiling_start as isize)).hi_flags as i32) | (1 << 16)) as u32;
        (*groundCollLine.offset(ceiling_start as isize)).x0 = core::ptr::addr_of_mut!(*(*coll_data).lines.offset(ceiling_start as isize)) as *mut MapLine;
        ceiling_start = ceiling_start + 1;
        ceiling_count = ceiling_count - 1;
    }
    right_wall_count = (*coll_data).ranges[(MapLineGroup_RightWall as i32) as usize].count as i32;
    right_wall_start = (*coll_data).ranges[(MapLineGroup_RightWall as i32) as usize].start as i32;
    'l177: while right_wall_count > 0 {
        (*groundCollLine.offset(right_wall_start as isize)).flags = (((*(*coll_data).lines.offset(right_wall_start as isize)).hi_flags as i32) | (1 << 16)) as u32;
        (*groundCollLine.offset(right_wall_start as isize)).x0 = core::ptr::addr_of_mut!(*(*coll_data).lines.offset(right_wall_start as isize)) as *mut MapLine;
        right_wall_start = right_wall_start + 1;
        right_wall_count = right_wall_count - 1;
    }
    left_wall_count = (*coll_data).ranges[(MapLineGroup_LeftWall as i32) as usize].count as i32;
    left_wall_start = (*coll_data).ranges[(MapLineGroup_LeftWall as i32) as usize].start as i32;
    'l178: while left_wall_count > 0 {
        (*groundCollLine.offset(left_wall_start as isize)).flags = (((*(*coll_data).lines.offset(left_wall_start as isize)).hi_flags as i32) | (1 << 16)) as u32;
        (*groundCollLine.offset(left_wall_start as isize)).x0 = core::ptr::addr_of_mut!(*(*coll_data).lines.offset(left_wall_start as isize)) as *mut MapLine;
        left_wall_start = left_wall_start + 1;
        left_wall_count = left_wall_count - 1;
    }
    dynamic_count = (*coll_data).ranges[(MapLineGroup_Dynamic as i32) as usize].count as i32;
    dynamic_start = (*coll_data).ranges[(MapLineGroup_Dynamic as i32) as usize].start as i32;
    'l179: while dynamic_count > 0 {
        (*groundCollLine.offset(dynamic_start as isize)).flags = (((*(*coll_data).lines.offset(dynamic_start as isize)).hi_flags as i32) | (1 << 16)) as u32;
        (*groundCollLine.offset(dynamic_start as isize)).x0 = core::ptr::addr_of_mut!(*(*coll_data).lines.offset(dynamic_start as isize)) as *mut MapLine;
        dynamic_start = dynamic_start + 1;
        dynamic_count = dynamic_count - 1;
    }
    i = 0;
    'l180: while i < (*coll_data).vert_count {
        f0 = (*(*coll_data).verts.offset(i as isize)).x;
        f2 = f31 * f0;
        (*groundCollVtx.offset(i as isize)).x0 = f0;
        (*groundCollVtx.offset(i as isize)).pos.x = f2;
        (*groundCollVtx.offset(i as isize)).x10 = f2;
        f0 = (*(*coll_data).verts.offset(i as isize)).y;
        f1 = f31 * f0;
        (*groundCollVtx.offset(i as isize)).x4 = f0;
        (*groundCollVtx.offset(i as isize)).pos.y = f1;
        (*groundCollVtx.offset(i as isize)).x14 = f1;
        if mpLib_80458868[0 as usize].top < f1 {
            mpLib_80458868[0 as usize].top = f1;
        }
        if mpLib_80458868[0 as usize].bottom > f1 {
            mpLib_80458868[0 as usize].bottom = f1;
        }
        if mpLib_80458868[0 as usize].right < f2 {
            mpLib_80458868[0 as usize].right = f2;
        }
        if mpLib_80458868[0 as usize].left > f2 {
            mpLib_80458868[0 as usize].left = f2;
        }
        i = i + 1;
    }
    mpLib_804D64B4 = coll_data;
    if !coll_data.is_null() {
        super::mpisland::mpIsland_8005A728();
    } else {
        super::mpisland::mpIsland_8005A6F8();
    }
    mpUncheckBounding();
}

/// `mpLineGetNext`
pub unsafe fn mpLineGetNext(mut line_id: i32) -> i32 {
    let mut result: i16 = (*(*groundCollLine.offset(line_id as isize)).x0).next_id1;
    let mut ret: i32 = result as i32;
    if !(ext::ScriptGame_StageLineActive(line_id) != 0) {
        return (-1_i32);
    }
    if (result as i32) != (-1_i32) {
        let mut flags: u32 = (*groundCollLine.offset(result as isize)).flags;
        if (((flags & ((1 << 16) as u32)) != 0) && (!((flags & ((1 << 18) as u32)) != 0))) && (ext::ScriptGame_StageLineActive(result as i32) != 0) {
            let mut v1: *mut CollVtx = core::ptr::addr_of_mut!(*groundCollVtx.offset((*(*groundCollLine.offset(line_id as isize)).x0).v1_idx as isize)) as *mut CollVtx;
            let mut v0: *mut CollVtx = core::ptr::addr_of_mut!(*groundCollVtx.offset((*(*groundCollLine.offset(result as isize)).x0).v0_idx as isize)) as *mut CollVtx;
            if (fmadds((*v1).pos.x - (*v0).pos.x, (*v1).pos.x - (*v0).pos.x, ((*v1).pos.y - (*v0).pos.y) * ((*v1).pos.y - (*v0).pos.y)) as f64) < 4.0f64 {
                return ret;
            }
        }
    }
    return (*(*groundCollLine.offset(line_id as isize)).x0).next_id0 as i32;
}

/// `mpLineGetPrev`
pub unsafe fn mpLineGetPrev(mut line_id: i32) -> i32 {
    let mut result: i16 = (*(*groundCollLine.offset(line_id as isize)).x0).prev_id1;
    let mut ret: i32 = result as i32;
    if !(ext::ScriptGame_StageLineActive(line_id) != 0) {
        return (-1_i32);
    }
    if (result as i32) != (-1_i32) {
        let mut flags: u32 = (*groundCollLine.offset(result as isize)).flags;
        if (((flags & ((1 << 16) as u32)) != 0) && (!((flags & ((1 << 18) as u32)) != 0))) && (ext::ScriptGame_StageLineActive(result as i32) != 0) {
            let mut v0: *mut CollVtx = core::ptr::addr_of_mut!(*groundCollVtx.offset((*(*groundCollLine.offset(line_id as isize)).x0).v0_idx as isize)) as *mut CollVtx;
            let mut v1: *mut CollVtx = core::ptr::addr_of_mut!(*groundCollVtx.offset((*(*groundCollLine.offset(result as isize)).x0).v1_idx as isize)) as *mut CollVtx;
            if (fmadds((*v0).pos.x - (*v1).pos.x, (*v0).pos.x - (*v1).pos.x, ((*v0).pos.y - (*v1).pos.y) * ((*v0).pos.y - (*v1).pos.y)) as f64) < 4.0f64 {
                return ret;
            }
        }
    }
    return (*(*groundCollLine.offset(line_id as isize)).x0).prev_id0 as i32;
}

/// `mpRemap2d`
unsafe fn mpRemap2d(mut x_out: *mut f32, mut y_out: *mut f32, mut ax0: f32, mut ay0: f32, mut ax1: f32, mut ay1: f32, mut bx0: f32, mut by0: f32, mut bx1: f32, mut by1: f32, mut px: f32, mut py: f32) {
    let mut dx: f64 = 0.0;
    let mut dy: f64 = 0.0;
    let mut dist2: f64 = 0.0;
    let mut f30: f32 = 0.0;
    let mut f29: f32 = 0.0;
    dx = (ax1 - ax0) as f64;
    dy = (ay1 - ay0) as f64;
    f30 = px - ax0;
    f29 = py - ay0;
    dist2 = (dy).mul_add(dy, dx * dx);
    if (if dist2 < (0 as f64) { -dist2 } else { dist2 }) > 0.0001f64 {
        let mut t: f64 = (dy).mul_add(f29 as f64, dx * (f30 as f64)) / dist2;
        if t > 1.0f64 {
            t = 1.0f64;
        } else if t < 0.0f64 {
            t = 0.0f64;
        }
        *x_out = (t).mul_add((bx1 - ax1) as f64, (1.0f64 - t).mul_add((bx0 - ax0) as f64, px as f64)) as f32;
        *y_out = (t).mul_add((by1 - ay1) as f64, (1.0f64 - t).mul_add((by0 - ay0) as f64, py as f64)) as f32;
    } else {
        *x_out = (px + (bx0 - ax0)) + (bx1 - ax0);
        *y_out = (py + (by0 - ay0)) + (by1 - ay0);
    }
}

/// `mpLib_8004DD90_Floor`
pub unsafe fn mpLib_8004DD90_Floor(mut line_id: i32, mut vec: *mut Vec, mut y_out: *mut f32, mut flags_out: *mut u32, mut normal_out: *mut Vec) -> i32 {
    let mut dir: i32 = 0;
    let mut x0: f32 = 0.0;
    let mut y0: f32 = 0.0;
    let mut x1: f32 = 0.0;
    let mut y1: f32 = 0.0;
    let mut x: f32 = 0.0;
    let mut y: f32 = 0.0;
    {
        if (line_id == (-1_i32)) || (line_id >= (*mpLib_804D64B4).line_count) {
            if 0 != 0 { (); } else { ext::OSReport(&format!("{}:{}:not found lineID={}\n", core::ffi::CStr::from_ptr((c"src/melee/mp/mplib.c".as_ptr() as *mut i8) as *const core::ffi::c_char).to_string_lossy(), 774, line_id)); panic!("mpLib_8004DD90_Floor: assertion failed"); }
        }
    }
    'l181: while 1 != 0 {
        let mut line: *mut CollLine = core::ptr::addr_of_mut!(*groundCollLine.offset(line_id as isize)) as *mut CollLine;
        x0 = (*groundCollVtx.offset((*(*line).x0).v0_idx as isize)).pos.x;
        x1 = (*groundCollVtx.offset((*(*line).x0).v1_idx as isize)).pos.x;
        x = (*vec).x;
        y = (*vec).y;
        if x < x0 {
            if dir != 1 {
                let mut new_id: i32 = mpLineGetPrev(line_id);
                if (new_id == (-1_i32)) || (!(((*groundCollLine.offset(new_id as isize)).flags & ((CollLine_Floor as i32) as u32)) != 0)) {
                    if ((x - x0) as f64) < (-0.1f64) {
                        return (-1_i32);
                    }
                    x = x0;
                    break 'l181;
                }
                line_id = new_id;
                dir = (-1_i32);
            } else {
                x = x0;
                break 'l181;
            }
        } else if x > x1 {
            if dir != (-1_i32) {
                let mut new_id: i32 = mpLineGetNext(line_id);
                if (new_id == (-1_i32)) || (!(((*groundCollLine.offset(new_id as isize)).flags & ((CollLine_Floor as i32) as u32)) != 0)) {
                    if ((x - x1) as f64) > 0.1f64 {
                        return (-1_i32);
                    }
                    x = x1;
                    break 'l181;
                }
                line_id = new_id;
            } else {
                x = x1;
                break 'l181;
            }
        } else {
            break 'l181;
        }
    }
    if !flags_out.is_null() {
        *flags_out = (*(*groundCollLine.offset(line_id as isize)).x0).lo_flags as u32;
    }
    y0 = (*groundCollVtx.offset((*(*groundCollLine.offset(line_id as isize)).x0).v0_idx as isize)).pos.y;
    y1 = (*groundCollVtx.offset((*(*groundCollLine.offset(line_id as isize)).x0).v1_idx as isize)).pos.y;
    if !y_out.is_null() {
        *y_out = (((((((y1 - y0) * (x - x0)) / (x1 - x0)) + y0) - y) as f64) + 0.0001f64) as f32;
    }
    if !normal_out.is_null() {
        (*normal_out).x = -(y1 - y0);
        (*normal_out).y = x1 - x0;
        (*normal_out).z = 0.0f32;
        { let __n = melee_runtime::mtx::vec_normalize(&melee_core::vec::Vec3 { x: (*normal_out).x, y: (*normal_out).y, z: (*normal_out).z }); (*normal_out).x = __n.x; (*normal_out).y = __n.y; (*normal_out).z = __n.z; };
    }
    return line_id;
}

/// `mpLib_8004E090_Ceiling`
pub unsafe fn mpLib_8004E090_Ceiling(mut line_id: i32, mut vec: *mut Vec, mut y_out: *mut f32, mut flags_out: *mut u32, mut normal_out: *mut Vec) -> i32 {
    let mut dir: i32 = 0;
    let mut x0: f32 = 0.0;
    let mut y0: f32 = 0.0;
    let mut x1: f32 = 0.0;
    let mut y1: f32 = 0.0;
    let mut x: f32 = 0.0;
    {
        if (line_id == (-1_i32)) || (line_id >= (*mpLib_804D64B4).line_count) {
            if 0 != 0 { (); } else { ext::OSReport(&format!("{}:{}:not found lineID={}\n", core::ffi::CStr::from_ptr((c"src/melee/mp/mplib.c".as_ptr() as *mut i8) as *const core::ffi::c_char).to_string_lossy(), 893, line_id)); panic!("mpLib_8004E090_Ceiling: assertion failed"); }
        }
    }
    x = (*vec).x;
    'l182: while 1 != 0 {
        'c182: {
            let mut line_r4: *mut CollLine = core::ptr::addr_of_mut!(*groundCollLine.offset(line_id as isize)) as *mut CollLine;
            x0 = (*groundCollVtx.offset((*(*line_r4).x0).v0_idx as isize)).pos.x;
            x1 = (*groundCollVtx.offset((*(*line_r4).x0).v1_idx as isize)).pos.x;
            if (*vec).x < x1 {
                if dir != 1 {
                    let mut new_id: i32 = mpLineGetNext(line_id);
                    if (new_id == (-1_i32)) || (!(((*groundCollLine.offset(new_id as isize)).flags & ((CollLine_Ceiling as i32) as u32)) != 0)) {
                        if (((*vec).x - x1) as f64) < (-0.1f64) {
                            return (-1_i32);
                        }
                        x = x1;
                        break 'l182;
                    }
                    line_id = new_id;
                    dir = (-1_i32);
                    break 'c182;
                }
                x = x1;
            } else if (*vec).x > x0 {
                if dir != (-1_i32) {
                    let mut new_id: i32 = mpLineGetPrev(line_id);
                    if (new_id == (-1_i32)) || (!(((*groundCollLine.offset(new_id as isize)).flags & ((CollLine_Ceiling as i32) as u32)) != 0)) {
                        if (((*vec).x - x0) as f64) > 0.1f64 {
                            return (-1_i32);
                        }
                        x = x0;
                        break 'l182;
                    }
                    line_id = new_id;
                    dir = 1;
                    break 'c182;
                }
                x = x0;
            }
            break 'l182;
        }
    }
    if !flags_out.is_null() {
        *flags_out = (*(*groundCollLine.offset(line_id as isize)).x0).lo_flags as u32;
    }
    y0 = (*groundCollVtx.offset((*(*groundCollLine.offset(line_id as isize)).x0).v0_idx as isize)).pos.y;
    y1 = (*groundCollVtx.offset((*(*groundCollLine.offset(line_id as isize)).x0).v1_idx as isize)).pos.y;
    if !y_out.is_null() {
        *y_out = (((((((y1 - y0) * (x - x0)) / (x1 - x0)) + y0) - (*vec).y) as f64) - 0.0001f64) as f32;
    }
    if !normal_out.is_null() {
        (*normal_out).x = -(y1 - y0);
        (*normal_out).y = x1 - x0;
        (*normal_out).z = 0.0f32;
        { let __n = melee_runtime::mtx::vec_normalize(&melee_core::vec::Vec3 { x: (*normal_out).x, y: (*normal_out).y, z: (*normal_out).z }); (*normal_out).x = __n.x; (*normal_out).y = __n.y; (*normal_out).z = __n.z; };
    }
    return line_id;
}

/// `mpLib_8004E398_LeftWall`
pub unsafe fn mpLib_8004E398_LeftWall(mut line_id: i32, mut vec: *mut Vec, mut x_out: *mut f32, mut flags_out: *mut u32, mut normal_out: *mut Vec) -> i32 {
    let mut dir: i32 = 0;
    let mut x0: f32 = 0.0;
    let mut y0: f32 = 0.0;
    let mut x1: f32 = 0.0;
    let mut y1: f32 = 0.0;
    let mut y: f32 = 0.0;
    {
        if (line_id == (-1_i32)) || (line_id >= (*mpLib_804D64B4).line_count) {
            if 0 != 0 { (); } else { ext::OSReport(&format!("{}:{}:not found lineID={}\n", core::ffi::CStr::from_ptr((c"src/melee/mp/mplib.c".as_ptr() as *mut i8) as *const core::ffi::c_char).to_string_lossy(), 1014, line_id)); panic!("mpLib_8004E398_LeftWall: assertion failed"); }
        }
    }
    y = (*vec).y;
    'l183: while 1 != 0 {
        'c183: {
            let mut line_r4: *mut CollLine = core::ptr::addr_of_mut!(*groundCollLine.offset(line_id as isize)) as *mut CollLine;
            y0 = (*groundCollVtx.offset((*(*line_r4).x0).v0_idx as isize)).pos.y;
            y1 = (*groundCollVtx.offset((*(*line_r4).x0).v1_idx as isize)).pos.y;
            if (*vec).y < y0 {
                if dir != 1 {
                    let mut new_id: i32 = mpLineGetPrev(line_id);
                    if (new_id == (-1_i32)) || (!(((*groundCollLine.offset(new_id as isize)).flags & ((CollLine_LeftWall as i32) as u32)) != 0)) {
                        if (((*vec).y - y0) as f64) < (-0.1f64) {
                            return (-1_i32);
                        }
                        y = y0;
                        break 'l183;
                    }
                    line_id = new_id;
                    dir = (-1_i32);
                    break 'c183;
                }
            } else if (*vec).y > y1 {
                if dir != (-1_i32) {
                    let mut new_id: i32 = mpLineGetNext(line_id);
                    if (new_id == (-1_i32)) || (!(((*groundCollLine.offset(new_id as isize)).flags & ((CollLine_LeftWall as i32) as u32)) != 0)) {
                        if (((*vec).y - y1) as f64) > 0.1f64 {
                            return (-1_i32);
                        }
                        y = y1;
                        break 'l183;
                    }
                    line_id = new_id;
                    dir = 1;
                    break 'c183;
                }
            }
            break 'l183;
        }
    }
    if !flags_out.is_null() {
        *flags_out = (*(*groundCollLine.offset(line_id as isize)).x0).lo_flags as u32;
    }
    x0 = (*groundCollVtx.offset((*(*groundCollLine.offset(line_id as isize)).x0).v0_idx as isize)).pos.x;
    x1 = (*groundCollVtx.offset((*(*groundCollLine.offset(line_id as isize)).x0).v1_idx as isize)).pos.x;
    if !x_out.is_null() {
        *x_out = (x0 + (((x1 - x0) * (y - y0)) / (y1 - y0))) - (*vec).x;
    }
    if !normal_out.is_null() {
        (*normal_out).x = -(y1 - y0);
        (*normal_out).y = x1 - x0;
        (*normal_out).z = 0.0f32;
        { let __n = melee_runtime::mtx::vec_normalize(&melee_core::vec::Vec3 { x: (*normal_out).x, y: (*normal_out).y, z: (*normal_out).z }); (*normal_out).x = __n.x; (*normal_out).y = __n.y; (*normal_out).z = __n.z; };
    }
    return line_id;
}

/// `mpLib_8004E684_RightWall`
pub unsafe fn mpLib_8004E684_RightWall(mut line_id: i32, mut vec: *mut Vec, mut x_out: *mut f32, mut flags_out: *mut u32, mut normal_out: *mut Vec) -> i32 {
    let mut dir: i32 = 0;
    let mut x0: f32 = 0.0;
    let mut y0: f32 = 0.0;
    let mut x1: f32 = 0.0;
    let mut y1: f32 = 0.0;
    let mut y: f32 = 0.0;
    {
        if (line_id == (-1_i32)) || (line_id >= (*mpLib_804D64B4).line_count) {
            if 0 != 0 { (); } else { ext::OSReport(&format!("{}:{}:not found lineID={}\n", core::ffi::CStr::from_ptr((c"src/melee/mp/mplib.c".as_ptr() as *mut i8) as *const core::ffi::c_char).to_string_lossy(), 1134, line_id)); panic!("mpLib_8004E684_RightWall: assertion failed"); }
        }
    }
    y = (*vec).y;
    'l184: while 1 != 0 {
        'c184: {
            let mut line_r4: *mut CollLine = core::ptr::addr_of_mut!(*groundCollLine.offset(line_id as isize)) as *mut CollLine;
            y0 = (*groundCollVtx.offset((*(*line_r4).x0).v0_idx as isize)).pos.y;
            y1 = (*groundCollVtx.offset((*(*line_r4).x0).v1_idx as isize)).pos.y;
            if (*vec).y > y0 {
                if dir != (-1_i32) {
                    let mut new_id_r8: i32 = mpLineGetPrev(line_id);
                    if (new_id_r8 == (-1_i32)) || (!(((*groundCollLine.offset(new_id_r8 as isize)).flags & ((CollLine_RightWall as i32) as u32)) != 0)) {
                        if (((*vec).y - y0) as f64) > 0.1f64 {
                            return (-1_i32);
                        }
                        y = y0;
                        break 'l184;
                    }
                    line_id = new_id_r8;
                    dir = 1;
                    break 'c184;
                }
                y = y0;
            } else if (*vec).y < y1 {
                if dir != 1 {
                    let mut new_id_r9: i32 = mpLineGetNext(line_id);
                    if (new_id_r9 == (-1_i32)) || (!(((*groundCollLine.offset(new_id_r9 as isize)).flags & ((CollLine_RightWall as i32) as u32)) != 0)) {
                        if (((*vec).y - y1) as f64) < (-0.1f64) {
                            return (-1_i32);
                        }
                        y = y1;
                        break 'l184;
                    }
                    line_id = new_id_r9;
                    dir = (-1_i32);
                    break 'c184;
                }
                y = y1;
            }
            break 'l184;
        }
    }
    if !flags_out.is_null() {
        *flags_out = (*(*groundCollLine.offset(line_id as isize)).x0).lo_flags as u32;
    }
    x0 = (*groundCollVtx.offset((*(*groundCollLine.offset(line_id as isize)).x0).v0_idx as isize)).pos.x;
    x1 = (*groundCollVtx.offset((*(*groundCollLine.offset(line_id as isize)).x0).v1_idx as isize)).pos.x;
    if !x_out.is_null() {
        *x_out = (x0 + (((x1 - x0) * (y - y0)) / (y1 - y0))) - (*vec).x;
    }
    if !normal_out.is_null() {
        (*normal_out).x = -(y1 - y0);
        (*normal_out).y = x1 - x0;
        (*normal_out).z = 0.0f32;
        { let __n = melee_runtime::mtx::vec_normalize(&melee_core::vec::Vec3 { x: (*normal_out).x, y: (*normal_out).y, z: (*normal_out).z }); (*normal_out).x = __n.x; (*normal_out).y = __n.y; (*normal_out).z = __n.z; };
    }
    return line_id;
}

/// `mpLineIntersection`
pub unsafe fn mpLineIntersection(mut a0x: f32, mut a0y: f32, mut a1x: f32, mut a1y: f32, mut b0x: f32, mut b0y: f32, mut b1x: f32, mut b1y: f32, mut int_x: *mut f32, mut int_y: *mut f32) -> i32 {
    let mut b1_below_a: i32 = 0;
    let mut b2_above_a: i32 = 0;
    if a0x <= a1x {
        if ((b0x < a0x) && (b1x < a0x)) || ((a1x < b0x) && (a1x < b1x)) {
            return 0;
        }
    } else {
        if ((b0x < a1x) && (b1x < a1x)) || ((a0x < b0x) && (a0x < b1x)) {
            return 0;
        }
    }
    if a0y <= a1y {
        if ((b0y < a0y) && (b1y < a0y)) || ((a1y < b0y) && (a1y < b1y)) {
            return 0;
        }
    } else {
        if ((b0y < a1y) && (b1y < a1y)) || ((a0y < b0y) && (a0y < b1y)) {
            return 0;
        }
    }
    {
        let mut ah: f64 = (a1y - a0y) as f64;
        let mut d0x: f64 = (b0x - a0x) as f64;
        let mut aw: f64 = (a1x - a0x) as f64;
        let mut d0y: f64 = (b0y - a0y) as f64;
        let mut hs_b0_a: f64 = (aw).mul_add(d0y, -(ah * d0x));
        let mut d1y: f64 = 0.0;
        let mut d1x: f64 = 0.0;
        let mut det: f64 = 0.0;
        let mut hs_b1_a: f64 = 0.0;
        let mut bh: f64 = 0.0;
        let mut bw: f64 = 0.0;
        if hs_b0_a < 0.0f64 {
            if hs_b0_a < (-0.1f64) {
                return 0;
            }
            b1_below_a = 1;
        }
        d1x = (b1x - a1x) as f64;
        d1y = (b1y - a1y) as f64;
        hs_b1_a = (aw).mul_add(d1y, -(ah * d1x));
        if hs_b1_a > 0.0f64 {
            if hs_b1_a > 0.1f64 {
                return 0;
            }
            b2_above_a = 1;
        }
        if (hs_b0_a == 0.0f64) && (hs_b1_a == 0.0f64) {
            return 0;
        }
        det = (d0x).mul_add(d1y, -(d0y * d1x));
        if det < hs_b0_a {
            if det < hs_b1_a {
                return 0;
            }
        } else if det > hs_b0_a {
            if det > hs_b1_a {
                return 0;
            }
        }
        bw = (b1x - b0x) as f64;
        bh = (b1y - b0y) as f64;
        'b185: {
            if !((((bw == 0.0f64) && (bh == 0.0f64)) || ((b1_below_a != 0) && (b2_above_a != 0))) || ((hs_b0_a >= 0.0f64) && (b2_above_a != 0))) {
                let mut area: f64 = (bw).mul_add(ah, -(bh * aw));
                if (if area < (0 as f64) { -area } else { area }) > (0.0001f32 as f64) {
                    let mut t: f64 = (bw).mul_add(d0y, -(bh * d0x)) / area;
                    if t > 0.0f64 {
                        if t < 1.0f64 {
                            *int_x = (aw).mul_add(t, a0x as f64) as f32;
                            *int_y = (ah).mul_add(t, a0y as f64) as f32;
                        } else {
                            *int_x = a1x;
                            *int_y = a1y;
                        }
                    } else {
                        *int_x = a0x;
                        *int_y = a0y;
                    }
                    break 'b185;
                }
            }
            return 0;
        }
        return 1;
    }
}

/// `mpLineIntersectionH`
pub unsafe fn mpLineIntersectionH(mut int_x: *mut f32, mut int_y: *mut f32, mut a0x: f32, mut a0y: f32, mut a1x: f32, mut b0x: f32, mut b0y: f32, mut b1x: f32, mut b1y: f32) -> i32 {
    let mut max_ax: f32 = 0.0;
    let mut min_ax: f32 = 0.0;
    let mut dbx: f64 = 0.0;
    let mut dby: f64 = 0.0;
    let mut new_x: f64 = 0.0;
    let mut dx: f64 = 0.0;
    if a0x < a1x {
        if ((b0x < a0x) && (b1x < a0x)) || ((a1x < b0x) && (a1x < b1x)) {
            return 0;
        }
        if (((b0y - a0y) as f64) < (-0.0001f64)) || (((b1y - a0y) as f64) > 0.0001f64) {
            return 0;
        }
        min_ax = a0x;
        max_ax = a1x;
    } else {
        if ((b0x < a1x) && (b1x < a1x)) || ((a0x < b0x) && (a0x < b1x)) {
            return 0;
        }
        if (((b1y - a0y) as f64) < (-0.0001f64)) || (((b0y - a0y) as f64) > 0.0001f64) {
            return 0;
        }
        min_ax = a1x;
        max_ax = a0x;
    }
    dby = (b1y - b0y) as f64;
    dbx = (b1x - b0x) as f64;
    if (if dby < (0 as f64) { -dby } else { dby }) < 0.0001f64 {
        return 0;
    }
    new_x = (dbx / dby).mul_add((a0y - b0y) as f64, b0x as f64);
    dx = new_x - (min_ax as f64);
    if dx < 0.0f64 {
        if dx < (-0.1f64) {
            return 0;
        }
        new_x = min_ax as f64;
    }
    if (new_x - (max_ax as f64)) > 0.0f64 {
        if (new_x - (max_ax as f64)) > 0.1f64 {
            return 0;
        }
        new_x = max_ax as f64;
    }
    *int_x = new_x as f32;
    *int_y = a0y;
    return 1;
}

/// `mpLineGetCollLine`
unsafe fn mpLineGetCollLine(mut line_id: i32) -> *mut CollLine {
    return core::ptr::addr_of_mut!(*groundCollLine.offset(line_id as isize)) as *mut CollLine;
}

/// `mpLib_8004ED5C`
pub unsafe fn mpLib_8004ED5C(mut line_id: i32, mut x0_out: *mut f32, mut y0_out: *mut f32, mut x1_out: *mut f32, mut y1_out: *mut f32) {
    let mut calculated_distance: i32 = 0;
    let mut line: *mut CollLine = mpLineGetCollLine(line_id);
    let mut i0: i32 = 0;
    let mut i1: i32 = 0;
    let mut x0_f0: f32 = 0.0;
    let mut y0_f1: f32 = 0.0;
    let mut x1_f2: f32 = 0.0;
    let mut y1_f3: f32 = 0.0;
    let mut distance: f32 = 0.0;
    i0 = (*(*line).x0).v0_idx as i32;
    x0_f0 = (*groundCollVtx.offset(i0 as isize)).pos.x;
    y0_f1 = (*groundCollVtx.offset(i0 as isize)).pos.y;
    i1 = (*(*line).x0).v1_idx as i32;
    x1_f2 = (*groundCollVtx.offset(i1 as isize)).pos.x;
    y1_f3 = (*groundCollVtx.offset(i1 as isize)).pos.y;
    if mpLineGetPrev(line_id) != (-1_i32) {
        distance = melee_runtime::gekko::sqrtf(fmadds(x0_f0 - x1_f2, x0_f0 - x1_f2, (y0_f1 - y1_f3) * (y0_f1 - y1_f3)));
        if distance > 0.001f32 {
            x0_f0 = x0_f0 + ((x0_f0 - x1_f2) / distance);
            y0_f1 = y0_f1 + ((y0_f1 - y1_f3) / distance);
        }
        calculated_distance = 1;
    }
    if mpLineGetNext(line_id) != (-1_i32) {
        if !(calculated_distance != 0) {
            distance = melee_runtime::gekko::sqrtf(fmadds(x0_f0 - x1_f2, x0_f0 - x1_f2, (y0_f1 - y1_f3) * (y0_f1 - y1_f3)));
        }
        if distance > 0.001f32 {
            x1_f2 = x1_f2 + ((x1_f2 - x0_f0) / distance);
            y1_f3 = y1_f3 + ((y1_f3 - y0_f1) / distance);
        }
    }
    *x0_out = x0_f0;
    *y0_out = y0_f1;
    *x1_out = x1_f2;
    *y1_out = y1_f3;
}

/// `mpCheckFloor`
pub unsafe fn mpCheckFloor(mut ax: f32, mut ay: f32, mut bx: f32, mut by: f32, mut y_offset: f32, mut vec_out: *mut Vec, mut line_id_out: *mut i32, mut flags_out: *mut u32, mut normal_out: *mut Vec, mut line_id_skip: i32, mut joint_id_skip: i32, mut joint_id_only: i32, mut cb: Option<unsafe extern "C" fn(*mut HSD_GObj, i32) -> i32>, mut gobj: *mut HSD_GObj) -> i32 {
    let mut min_dist2_f30: f32 = 0.0;
    let mut joint: *mut CollJoint = null_mut();
    let mut i_r28: i32 = 0;
    let mut result_r27: i32 = 0;
    let mut already_checked: i32 = 0;
    result_r27 = 0;
    min_dist2_f30 = f32::MAX;
    already_checked = mpCheckedBounding();
    if !(already_checked != 0) {
        mpBoundingCheck2(ax, ay, bx, by);
    }
    {
        joint = jointListStart;
        'l186: while !joint.is_null() {
            'c186: {
                if !(ext::ScriptGame_StageJointActive((joint.offset_from(groundCollJoint) as i64) as i32) != 0) {
                    break 'c186;
                }
                let mut line_r26: *mut CollLine = null_mut();
                let mut var_r25: i32 = 0;
                let mut var_r24: i32 = 0;
                let mut j_inner: *mut MapJoint = null_mut();
                if ((*joint).flags & ((CollJoint_TooFar as i32) as u32)) != 0 {
                    break 'c186;
                }
                if ((joint_id_skip as i64) == (joint.offset_from(groundCollJoint) as i64)) || ((joint_id_only != (-1_i32)) && ((joint_id_only as i64) != (joint.offset_from(groundCollJoint) as i64))) {
                    break 'c186;
                }
                j_inner = (*joint).inner;
                i_r28 = 0;
                var_r25 = (*j_inner).ranges[(MapLineGroup_Floor as i32) as usize].count as i32;
                var_r24 = (*j_inner).ranges[(MapLineGroup_Dynamic as i32) as usize].count as i32;
                line_r26 = core::ptr::addr_of_mut!(*groundCollLine.offset((*j_inner).ranges[(MapLineGroup_Floor as i32) as usize].start as isize)) as *mut CollLine;
                {
                    'p187: loop {
                        'l188: while i_r28 < var_r25 {
                            'c188: {
                                let mut px_sp54: f32 = 0.0;
                                let mut py_sp50: f32 = 0.0;
                                let mut pad: [u8; 4] = [0; 4];
                                let mut x0_sp48: f32 = 0.0;
                                let mut y0_sp44: f32 = 0.0;
                                let mut x1_sp40: f32 = 0.0;
                                let mut y1_sp3C: f32 = 0.0;
                                let mut dist2: f32 = 0.0;
                                let mut line_offset: i32 = 0;
                                if (cb.is_some()) && (!(cb.unwrap()(gobj, (line_r26.offset_from(groundCollLine) as i64) as i32) != 0)) {
                                    break 'c188;
                                }
                                if line_id_skip == melee_runtime::cabi::sdiv({ line_offset = ((line_r26 as usize as i64) - (groundCollLine as usize as i64)) as i32; line_offset }, 16_u64 as i32) {
                                    break 'c188;
                                }
                                if ((!(((*line_r26).flags & ((CollLine_Floor as i32) as u32)) != 0)) || (!(((*line_r26).flags & ((1 << 16) as u32)) != 0))) || (((*line_r26).flags & ((1 << 7) as u32)) != 0) {
                                    break 'c188;
                                }
                                mpLib_8004ED5C(melee_runtime::cabi::sdiv(line_offset, 16_u64 as i32), core::ptr::addr_of_mut!(x0_sp48) as *mut f32, core::ptr::addr_of_mut!(y0_sp44) as *mut f32, core::ptr::addr_of_mut!(x1_sp40) as *mut f32, core::ptr::addr_of_mut!(y1_sp3C) as *mut f32);
                                y0_sp44 = y0_sp44 + y_offset;
                                y1_sp3C = y1_sp3C + y_offset;
                                if ((if (y0_sp44 - y1_sp3C) < (0 as f32) { -(y0_sp44 - y1_sp3C) } else { y0_sp44 - y1_sp3C }) as f64) > 0.0001f64 {
                                    if mpLineIntersection(x0_sp48, y0_sp44, x1_sp40, y1_sp3C, ax, ay, bx, by, core::ptr::addr_of_mut!(px_sp54) as *mut f32, core::ptr::addr_of_mut!(py_sp50) as *mut f32) != 0 {
                                        dist2 = fmadds(px_sp54 - ax, px_sp54 - ax, (py_sp50 - ay) * (py_sp50 - ay));
                                        if min_dist2_f30 > dist2 {
                                            min_dist2_f30 = dist2;
                                            if !vec_out.is_null() {
                                                (*vec_out).x = px_sp54;
                                                (*vec_out).y = py_sp50;
                                                (*vec_out).z = 0.0f32;
                                            }
                                            if !line_id_out.is_null() {
                                                *line_id_out = (line_r26.offset_from(groundCollLine) as i64) as i32;
                                            }
                                            if !flags_out.is_null() {
                                                *flags_out = (*(*line_r26).x0).lo_flags as u32;
                                            }
                                            if !normal_out.is_null() {
                                                (*normal_out).x = -(y1_sp3C - y0_sp44);
                                                (*normal_out).y = x1_sp40 - x0_sp48;
                                                (*normal_out).z = 0.0f32;
                                                { let __n = melee_runtime::mtx::vec_normalize(&melee_core::vec::Vec3 { x: (*normal_out).x, y: (*normal_out).y, z: (*normal_out).z }); (*normal_out).x = __n.x; (*normal_out).y = __n.y; (*normal_out).z = __n.z; };
                                            }
                                            result_r27 = 1;
                                        }
                                    }
                                } else {
                                    if (ay >= by) && (mpLineIntersectionH(core::ptr::addr_of_mut!(px_sp54) as *mut f32, core::ptr::addr_of_mut!(py_sp50) as *mut f32, x0_sp48, y0_sp44, x1_sp40, ax, ay, bx, by) != 0) {
                                        dist2 = fmadds(px_sp54 - ax, px_sp54 - ax, (py_sp50 - ay) * (py_sp50 - ay));
                                        if min_dist2_f30 > dist2 {
                                            min_dist2_f30 = dist2;
                                            if !vec_out.is_null() {
                                                (*vec_out).x = px_sp54;
                                                (*vec_out).y = py_sp50;
                                                (*vec_out).z = 0.0f32;
                                            }
                                            if !line_id_out.is_null() {
                                                *line_id_out = (line_r26.offset_from(groundCollLine) as i64) as i32;
                                            }
                                            if !flags_out.is_null() {
                                                *flags_out = (*(*line_r26).x0).lo_flags as u32;
                                            }
                                            if !normal_out.is_null() {
                                                (*normal_out).x = 0.0f32;
                                                (*normal_out).y = 1.0f32;
                                                (*normal_out).z = 0.0f32;
                                            }
                                            result_r27 = 1;
                                        }
                                    }
                                }
                            }
                            i_r28 = i_r28 + 1; line_r26 = line_r26.offset(1 as isize);
                        }
                        if var_r24 != 0 {
                            var_r25 = var_r24;
                            i_r28 = 0;
                            var_r24 = 0;
                            line_r26 = core::ptr::addr_of_mut!(*groundCollLine.offset((*(*joint).inner).ranges[(MapLineGroup_Dynamic as i32) as usize].start as isize)) as *mut CollLine;
                            continue 'p187;
                        }
                        break;
                    }
                }
            }
            joint = (*joint).next;
        }
    }
    if !(already_checked != 0) {
        mpUncheckBounding();
    }
    return result_r27;
}

/// `mpCheckFloorRemap`
pub unsafe fn mpCheckFloorRemap(mut ax: f32, mut ay: f32, mut bx: f32, mut by: f32, mut y_offset: f32, mut vec_out: *mut Vec, mut line_id_out: *mut i32, mut flags_out: *mut u32, mut normal_out: *mut Vec, mut line_id_skip: i32, mut joint_id_skip: i32, mut joint_id_only: i32, mut cb: Option<unsafe extern "C" fn(*mut HSD_GObj, i32) -> i32>, mut gobj: *mut HSD_GObj) -> i32 {
    let mut min_dist2: f32 = f32::MAX;
    let mut old_x: f32 = ax;
    let mut old_y: f32 = ay;
    let mut joint: *mut CollJoint = null_mut();
    let mut i: i32 = 0;
    let mut result: i32 = 0;
    let mut already_checked: i32 = 0;
    already_checked = mpCheckedBounding();
    if !(already_checked != 0) {
        mpBoundingCheck2(ax, ay, bx, by);
    }
    {
        joint = jointListStart;
        'l189: while !joint.is_null() {
            'c189: {
                if !(ext::ScriptGame_StageJointActive((joint.offset_from(groundCollJoint) as i64) as i32) != 0) {
                    break 'c189;
                }
                let mut line: *mut CollLine = null_mut();
                let mut count: i32 = 0;
                let mut count2: i32 = 0;
                if ((*joint).flags & ((CollJoint_TooFar as i32) as u32)) != 0 {
                    break 'c189;
                }
                if ((joint_id_skip as i64) == (joint.offset_from(groundCollJoint) as i64)) || ((joint_id_only != (-1_i32)) && ((joint_id_only as i64) != (joint.offset_from(groundCollJoint) as i64))) {
                    break 'c189;
                }
                count = (*(*joint).inner).ranges[(MapLineGroup_Floor as i32) as usize].count as i32;
                count2 = (*(*joint).inner).ranges[(MapLineGroup_Dynamic as i32) as usize].count as i32;
                line = core::ptr::addr_of_mut!(*groundCollLine.offset((*(*joint).inner).ranges[(MapLineGroup_Floor as i32) as usize].start as isize)) as *mut CollLine;
                {
                    i = 0;
                    'p190: loop {
                        'l191: while i < count {
                            'c191: {
                                if (cb.is_some()) && (!(cb.unwrap()(gobj, (line.offset_from(groundCollLine) as i64) as i32) != 0)) {
                                    break 'c191;
                                }
                                if (line_id_skip as i64) == (line.offset_from(groundCollLine) as i64) {
                                    break 'c191;
                                }
                                if ((!(((*line).flags & ((CollLine_Floor as i32) as u32)) != 0)) || (!(((*line).flags & ((1 << 16) as u32)) != 0))) || (((*line).flags & ((1 << 7) as u32)) != 0) {
                                    break 'c191;
                                }
                                {
                                    let mut v0_r5: *mut CollVtx = core::ptr::addr_of_mut!(*groundCollVtx.offset((*(*line).x0).v0_idx as isize)) as *mut CollVtx;
                                    let mut v1_r6: *mut CollVtx = core::ptr::addr_of_mut!(*groundCollVtx.offset((*(*line).x0).v1_idx as isize)) as *mut CollVtx;
                                    let mut x0: f32 = (*groundCollVtx.offset((*(*line).x0).v0_idx as isize)).pos.x;
                                    let mut y0: f32 = y_offset + (*groundCollVtx.offset((*(*line).x0).v0_idx as isize)).pos.y;
                                    let mut x1: f32 = (*groundCollVtx.offset((*(*line).x0).v1_idx as isize)).pos.x;
                                    let mut y1: f32 = y_offset + (*groundCollVtx.offset((*(*line).x0).v1_idx as isize)).pos.y;
                                    let mut dx: f32 = 0.0;
                                    let mut dy: f32 = 0.0;
                                    let mut dx2: f32 = 0.0;
                                    let mut dy2: f32 = 0.0;
                                    let mut dist2: f32 = 0.0;
                                    let mut int_x: f32 = 0.0;
                                    let mut int_y: f32 = 0.0;
                                    if ((*joint).flags & ((((CollJoint_B10 as i32) | (CollJoint_B9 as i32)) | (CollJoint_B8 as i32)) as u32)) != 0 {
                                        mpRemap2d(core::ptr::addr_of_mut!(ax) as *mut f32, core::ptr::addr_of_mut!(ay) as *mut f32, (*groundCollVtx.offset((*(*line).x0).v0_idx as isize)).x10, (*groundCollVtx.offset((*(*line).x0).v0_idx as isize)).x14, (*groundCollVtx.offset((*(*line).x0).v1_idx as isize)).x10, (*groundCollVtx.offset((*(*line).x0).v1_idx as isize)).x14, x0, y0, x1, y1, old_x, old_y);
                                    } else {
                                        ax = old_x;
                                        ay = old_y;
                                    }
                                    dx = bx - ax;
                                    dy = by - ay;
                                    if ((if (y0 - y1) < (0 as f32) { -(y0 - y1) } else { y0 - y1 }) as f64) > 0.0001f64 {
                                        if mpLineIntersection(x0, y0, x1, y1, ax, ay, bx, by, core::ptr::addr_of_mut!(int_x) as *mut f32, core::ptr::addr_of_mut!(int_y) as *mut f32) != 0 {
                                            dx2 = (int_x - old_x) * (int_x - old_x);
                                            dy2 = (int_y - old_y) * (int_y - old_y);
                                            dist2 = dx2 + dy2;
                                            if fmadds(dx, int_x - old_x, dy * (int_y - old_y)) < 0.0f32 {
                                                dist2 = -dist2;
                                            }
                                            if min_dist2 > dist2 {
                                                min_dist2 = dist2;
                                                if !vec_out.is_null() {
                                                    (*vec_out).x = int_x;
                                                    (*vec_out).y = int_y;
                                                    (*vec_out).z = 0.0f32;
                                                }
                                                if !line_id_out.is_null() {
                                                    *line_id_out = (line.offset_from(groundCollLine) as i64) as i32;
                                                }
                                                if !flags_out.is_null() {
                                                    *flags_out = (*(*line).x0).lo_flags as u32;
                                                }
                                                if !normal_out.is_null() {
                                                    (*normal_out).x = -(y1 - y0);
                                                    (*normal_out).y = x1 - x0;
                                                    (*normal_out).z = 0.0f32;
                                                    { let __n = melee_runtime::mtx::vec_normalize(&melee_core::vec::Vec3 { x: (*normal_out).x, y: (*normal_out).y, z: (*normal_out).z }); (*normal_out).x = __n.x; (*normal_out).y = __n.y; (*normal_out).z = __n.z; };
                                                }
                                                result = 1;
                                            }
                                        }
                                    } else if (ay >= by) && (mpLineIntersectionH(core::ptr::addr_of_mut!(int_x) as *mut f32, core::ptr::addr_of_mut!(int_y) as *mut f32, x0, y0, x1, ax, ay, bx, by) != 0) {
                                        dx2 = (int_x - old_x) * (int_x - old_x);
                                        dy2 = (int_y - old_y) * (int_y - old_y);
                                        dist2 = dx2 + dy2;
                                        if fmadds(dx, int_x - old_x, dy * (int_y - old_y)) < 0.0f32 {
                                            dist2 = -dist2;
                                        }
                                        if min_dist2 > dist2 {
                                            min_dist2 = dist2;
                                            if !vec_out.is_null() {
                                                (*vec_out).x = int_x;
                                                (*vec_out).y = int_y;
                                                (*vec_out).z = 0.0f32;
                                            }
                                            if !line_id_out.is_null() {
                                                *line_id_out = (line.offset_from(groundCollLine) as i64) as i32;
                                            }
                                            if !flags_out.is_null() {
                                                *flags_out = (*(*line).x0).lo_flags as u32;
                                            }
                                            if !normal_out.is_null() {
                                                (*normal_out).x = 0.0f32;
                                                (*normal_out).y = 1.0f32;
                                                (*normal_out).z = 0.0f32;
                                            }
                                            result = 1;
                                        }
                                    }
                                }
                            }
                            i = i + 1; line = line.offset(1);
                        }
                        if count2 != 0 {
                            count = count2;
                            i = 0;
                            count2 = 0;
                            line = core::ptr::addr_of_mut!(*groundCollLine.offset((*(*joint).inner).ranges[(MapLineGroup_Dynamic as i32) as usize].start as isize)) as *mut CollLine;
                            continue 'p190;
                        }
                        break;
                    }
                }
            }
            joint = (*joint).next;
        }
    }
    if !(already_checked != 0) {
        mpUncheckBounding();
    }
    return result;
}

/// `mpCheckCeiling`
pub unsafe fn mpCheckCeiling(mut ax: f32, mut ay: f32, mut bx: f32, mut by: f32, mut vec_out: *mut Vec, mut line_id_out: *mut i32, mut flags_out: *mut u32, mut normal_out: *mut Vec, mut joint_id_skip: i32, mut joint_id_only: i32) -> i32 {
    let mut min_dist2: f32 = f32::MAX;
    let mut joint: *mut CollJoint = null_mut();
    let mut i_r28: i32 = 0;
    let mut result: i32 = 0;
    let mut already_checked: i32 = 0;
    already_checked = mpCheckedBounding();
    if !(already_checked != 0) {
        mpBoundingCheck2(ax, ay, bx, by);
    }
    {
        joint = jointListStart;
        'l192: while !joint.is_null() {
            'c192: {
                if !(ext::ScriptGame_StageJointActive((joint.offset_from(groundCollJoint) as i64) as i32) != 0) {
                    break 'c192;
                }
                let mut line_r26: *mut CollLine = null_mut();
                let mut var_r25: i32 = 0;
                let mut var_r24: i32 = 0;
                let mut j_inner: *mut MapJoint = null_mut();
                if ((*joint).flags & ((CollJoint_TooFar as i32) as u32)) != 0 {
                    break 'c192;
                }
                if ((joint_id_skip as i64) == (joint.offset_from(groundCollJoint) as i64)) || ((joint_id_only != (-1_i32)) && ((joint_id_only as i64) != (joint.offset_from(groundCollJoint) as i64))) {
                    break 'c192;
                }
                j_inner = (*joint).inner;
                i_r28 = 0;
                var_r25 = (*j_inner).ranges[(MapLineGroup_Ceiling as i32) as usize].count as i32;
                var_r24 = (*j_inner).ranges[(MapLineGroup_Dynamic as i32) as usize].count as i32;
                line_r26 = core::ptr::addr_of_mut!(*groundCollLine.offset((*j_inner).ranges[(MapLineGroup_Ceiling as i32) as usize].start as isize)) as *mut CollLine;
                {
                    'p193: loop {
                        'l194: while i_r28 < var_r25 {
                            'c194: {
                                let mut int_x: f32 = 0.0;
                                let mut int_y: f32 = 0.0;
                                let mut pad: [u8; 4] = [0; 4];
                                let mut x0: f32 = 0.0;
                                let mut y0: f32 = 0.0;
                                let mut x1: f32 = 0.0;
                                let mut y1: f32 = 0.0;
                                let mut dist2: f32 = 0.0;
                                if ((!(((*line_r26).flags & ((CollLine_Ceiling as i32) as u32)) != 0)) || (!(((*line_r26).flags & ((1 << 16) as u32)) != 0))) || (((*line_r26).flags & ((1 << 7) as u32)) != 0) {
                                    break 'c194;
                                }
                                mpLib_8004ED5C((line_r26.offset_from(groundCollLine) as i64) as i32, core::ptr::addr_of_mut!(x0) as *mut f32, core::ptr::addr_of_mut!(y0) as *mut f32, core::ptr::addr_of_mut!(x1) as *mut f32, core::ptr::addr_of_mut!(y1) as *mut f32);
                                if ((if (y0 - y1) < (0 as f32) { -(y0 - y1) } else { y0 - y1 }) as f64) > 0.0001f64 {
                                    if mpLineIntersection(x0, y0, x1, y1, ax, ay, bx, by, core::ptr::addr_of_mut!(int_x) as *mut f32, core::ptr::addr_of_mut!(int_y) as *mut f32) != 0 {
                                        dist2 = fmadds(int_x - ax, int_x - ax, (int_y - ay) * (int_y - ay));
                                        if min_dist2 > dist2 {
                                            min_dist2 = dist2;
                                            if !vec_out.is_null() {
                                                (*vec_out).x = int_x;
                                                (*vec_out).y = int_y;
                                                (*vec_out).z = 0.0f32;
                                            }
                                            if !line_id_out.is_null() {
                                                *line_id_out = (line_r26.offset_from(groundCollLine) as i64) as i32;
                                            }
                                            if !flags_out.is_null() {
                                                *flags_out = (*(*line_r26).x0).lo_flags as u32;
                                            }
                                            if !normal_out.is_null() {
                                                (*normal_out).x = -(y1 - y0);
                                                (*normal_out).y = x1 - x0;
                                                (*normal_out).z = 0.0f32;
                                                { let __n = melee_runtime::mtx::vec_normalize(&melee_core::vec::Vec3 { x: (*normal_out).x, y: (*normal_out).y, z: (*normal_out).z }); (*normal_out).x = __n.x; (*normal_out).y = __n.y; (*normal_out).z = __n.z; };
                                            }
                                            result = 1;
                                        }
                                    }
                                } else {
                                    if (ay <= by) && (mpLineIntersectionH(core::ptr::addr_of_mut!(int_x) as *mut f32, core::ptr::addr_of_mut!(int_y) as *mut f32, x0, y0, x1, ax, ay, bx, by) != 0) {
                                        dist2 = fmadds(int_x - ax, int_x - ax, (int_y - ay) * (int_y - ay));
                                        if min_dist2 > dist2 {
                                            min_dist2 = dist2;
                                            if !vec_out.is_null() {
                                                (*vec_out).x = int_x;
                                                (*vec_out).y = int_y;
                                                (*vec_out).z = 0.0f32;
                                            }
                                            if !line_id_out.is_null() {
                                                *line_id_out = (line_r26.offset_from(groundCollLine) as i64) as i32;
                                            }
                                            if !flags_out.is_null() {
                                                *flags_out = (*(*line_r26).x0).lo_flags as u32;
                                            }
                                            if !normal_out.is_null() {
                                                (*normal_out).x = 0.0f32;
                                                (*normal_out).y = -1.0f32;
                                                (*normal_out).z = 0.0f32;
                                            }
                                            result = 1;
                                        }
                                    }
                                }
                            }
                            i_r28 = i_r28 + 1; line_r26 = line_r26.offset(1 as isize);
                        }
                        if var_r24 != 0 {
                            var_r25 = var_r24;
                            i_r28 = 0;
                            var_r24 = 0;
                            line_r26 = core::ptr::addr_of_mut!(*groundCollLine.offset((*(*joint).inner).ranges[(MapLineGroup_Dynamic as i32) as usize].start as isize)) as *mut CollLine;
                            continue 'p193;
                        }
                        break;
                    }
                }
            }
            joint = (*joint).next;
        }
    }
    if !(already_checked != 0) {
        mpUncheckBounding();
    }
    return result;
}

/// `mpCheckCeilingRemap`
pub unsafe fn mpCheckCeilingRemap(mut ax: f32, mut ay: f32, mut bx: f32, mut by: f32, mut vec_out: *mut Vec, mut line_id_out: *mut i32, mut flags_out: *mut u32, mut normal_out: *mut Vec, mut joint_id_skip: i32, mut joint_id_only: i32) -> i32 {
    let mut min_f30: f32 = f32::MAX;
    let mut f29: f32 = ax;
    let mut f28: f32 = ay;
    let mut joint: *mut CollJoint = null_mut();
    let mut r28: i32 = 0;
    let mut r27: i32 = 0;
    let mut already_checked: i32 = mpCheckedBounding();
    if !(already_checked != 0) {
        mpBoundingCheck2(ax, ay, bx, by);
    }
    {
        joint = jointListStart;
        'l195: while !joint.is_null() {
            'c195: {
                if !(ext::ScriptGame_StageJointActive((joint.offset_from(groundCollJoint) as i64) as i32) != 0) {
                    break 'c195;
                }
                let mut r26: *mut CollLine = null_mut();
                let mut r25: i32 = 0;
                let mut r24: i32 = 0;
                if ((*joint).flags & ((CollJoint_TooFar as i32) as u32)) != 0 {
                    break 'c195;
                }
                if ((joint_id_skip as i64) == (joint.offset_from(groundCollJoint) as i64)) || (!((joint_id_only == (-1_i32)) || ((joint_id_only as i64) == (joint.offset_from(groundCollJoint) as i64)))) {
                    break 'c195;
                }
                r25 = (*(*joint).inner).ranges[(MapLineGroup_Ceiling as i32) as usize].count as i32;
                r24 = (*(*joint).inner).ranges[(MapLineGroup_Dynamic as i32) as usize].count as i32;
                r26 = core::ptr::addr_of_mut!(*groundCollLine.offset((*(*joint).inner).ranges[(MapLineGroup_Ceiling as i32) as usize].start as isize)) as *mut CollLine;
                {
                    r28 = 0;
                    'p196: loop {
                        'l197: while r28 < r25 {
                            if ((((*r26).flags & ((CollLine_Ceiling as i32) as u32)) != 0) && (((*r26).flags & ((1 << 16) as u32)) != 0)) && (!(((*r26).flags & ((1 << 7) as u32)) != 0)) {
                                let mut dx2: f32 = 0.0;
                                let mut dy2: f32 = 0.0;
                                let mut dist2: f32 = 0.0;
                                let mut x_f23: f32 = 0.0;
                                let mut y_f22: f32 = 0.0;
                                let mut sp58: f32 = 0.0;
                                let mut sp54: f32 = 0.0;
                                let mut __: [u8; 12] = [0; 12];
                                let mut sp44: f32 = 0.0;
                                let mut sp40: f32 = 0.0;
                                let mut sp3C: f32 = 0.0;
                                let mut sp38: f32 = 0.0;
                                mpLib_8004ED5C((r26.offset_from(groundCollLine) as i64) as i32, core::ptr::addr_of_mut!(sp44) as *mut f32, core::ptr::addr_of_mut!(sp40) as *mut f32, core::ptr::addr_of_mut!(sp3C) as *mut f32, core::ptr::addr_of_mut!(sp38) as *mut f32);
                                if ((*joint).flags & ((((CollJoint_B10 as i32) | (CollJoint_B9 as i32)) | (CollJoint_B8 as i32)) as u32)) != 0 {
                                    let mut line_r3: *mut MapLine = (*r26).x0;
                                    let mut v1_r6: *mut CollVtx = core::ptr::addr_of_mut!(*groundCollVtx.offset((*line_r3).v1_idx as isize)) as *mut CollVtx;
                                    let mut v0_r5: *mut CollVtx = core::ptr::addr_of_mut!(*groundCollVtx.offset((*line_r3).v0_idx as isize)) as *mut CollVtx;
                                    mpRemap2d(core::ptr::addr_of_mut!(ax) as *mut f32, core::ptr::addr_of_mut!(ay) as *mut f32, (*v0_r5).x10, (*v0_r5).x14, (*v1_r6).x10, (*v1_r6).x14, sp44, sp40, sp3C, sp38, f29, f28);
                                } else {
                                    ax = f29;
                                    ay = f28;
                                }
                                x_f23 = bx - ax;
                                y_f22 = by - ay;
                                if ((if (sp40 - sp38) < (0 as f32) { -(sp40 - sp38) } else { sp40 - sp38 }) as f64) > 0.0001f64 {
                                    if mpLineIntersection(sp44, sp40, sp3C, sp38, ax, ay, bx, by, core::ptr::addr_of_mut!(sp58) as *mut f32, core::ptr::addr_of_mut!(sp54) as *mut f32) != 0 {
                                        dx2 = (sp58 - f29) * (sp58 - f29);
                                        dy2 = (sp54 - f28) * (sp54 - f28);
                                        dist2 = dx2 + dy2;
                                        if fmadds(x_f23, sp58 - f29, y_f22 * (sp54 - f28)) < 0.0f32 {
                                            dist2 = -dist2;
                                        }
                                        if min_f30 > dist2 {
                                            min_f30 = dist2;
                                            if !vec_out.is_null() {
                                                (*vec_out).x = sp58;
                                                (*vec_out).y = sp54;
                                                (*vec_out).z = 0.0f32;
                                            }
                                            if !line_id_out.is_null() {
                                                *line_id_out = (r26.offset_from(groundCollLine) as i64) as i32;
                                            }
                                            if !flags_out.is_null() {
                                                *flags_out = (*(*r26).x0).lo_flags as u32;
                                            }
                                            if !normal_out.is_null() {
                                                (*normal_out).x = -(sp38 - sp40);
                                                (*normal_out).y = sp3C - sp44;
                                                (*normal_out).z = 0.0f32;
                                                { let __n = melee_runtime::mtx::vec_normalize(&melee_core::vec::Vec3 { x: (*normal_out).x, y: (*normal_out).y, z: (*normal_out).z }); (*normal_out).x = __n.x; (*normal_out).y = __n.y; (*normal_out).z = __n.z; };
                                            }
                                            r27 = 1;
                                        }
                                    }
                                } else {
                                    if (ay <= by) && (mpLineIntersectionH(core::ptr::addr_of_mut!(sp58) as *mut f32, core::ptr::addr_of_mut!(sp54) as *mut f32, sp44, sp40, sp3C, ax, ay, bx, by) != 0) {
                                        dx2 = (sp58 - f29) * (sp58 - f29);
                                        dy2 = (sp54 - f28) * (sp54 - f28);
                                        dist2 = dx2 + dy2;
                                        if fmadds(x_f23, sp58 - f29, y_f22 * (sp54 - f28)) < 0.0f32 {
                                            dist2 = -dist2;
                                        }
                                        if min_f30 > dist2 {
                                            min_f30 = dist2;
                                            if !vec_out.is_null() {
                                                (*vec_out).x = sp58;
                                                (*vec_out).y = sp54;
                                                (*vec_out).z = 0.0f32;
                                            }
                                            if !line_id_out.is_null() {
                                                *line_id_out = (r26.offset_from(groundCollLine) as i64) as i32;
                                            }
                                            if !flags_out.is_null() {
                                                *flags_out = (*(*r26).x0).lo_flags as u32;
                                            }
                                            if !normal_out.is_null() {
                                                (*normal_out).x = 0.0f32;
                                                (*normal_out).y = -1.0f32;
                                                (*normal_out).z = 0.0f32;
                                            }
                                            r27 = 1;
                                        }
                                    }
                                }
                            }
                            r28 = r28 + 1; r26 = r26.offset(1);
                        }
                        if r24 != 0 {
                            r25 = r24;
                            r28 = 0;
                            r24 = 0;
                            r26 = core::ptr::addr_of_mut!(*groundCollLine.offset((*(*joint).inner).ranges[(MapLineGroup_Dynamic as i32) as usize].start as isize)) as *mut CollLine;
                            continue 'p196;
                        }
                        break;
                    }
                }
            }
            joint = (*joint).next;
        }
    }
    if !(already_checked != 0) {
        mpUncheckBounding();
    }
    return r27;
}

/// `mpLineIntersectionV`
pub unsafe fn mpLineIntersectionV(mut int_x: *mut f32, mut int_y: *mut f32, mut a0x: f32, mut a0y: f32, mut a1y: f32, mut b0x: f32, mut b0y: f32, mut b1x: f32, mut b1y: f32) -> i32 {
    let mut min_ay: f32 = 0.0;
    let mut max_ay: f32 = 0.0;
    let mut dbx: f64 = 0.0;
    let mut dby: f64 = 0.0;
    let mut new_y: f64 = 0.0;
    let mut dy: f64 = 0.0;
    if a0y < a1y {
        if ((b0y < a0y) && (b1y < a0y)) || ((a1y < b0y) && (a1y < b1y)) {
            return 0;
        }
        if (((b1x - a0x) as f64) < (-0.0001f64)) || (((b0x - a0x) as f64) > 0.0001f64) {
            return 0;
        }
        min_ay = a0y;
        max_ay = a1y;
    } else {
        if ((b0y < a1y) && (b1y < a1y)) || ((a0y < b0y) && (a0y < b1y)) {
            return 0;
        }
        if (((b0x - a0x) as f64) < (-0.0001f64)) || (((b1x - a0x) as f64) > 0.0001f64) {
            return 0;
        }
        min_ay = a1y;
        max_ay = a0y;
    }
    dby = (b1y - b0y) as f64;
    dbx = (b1x - b0x) as f64;
    if (if dbx < (0 as f64) { -dbx } else { dbx }) < 0.0001f64 {
        return 0;
    }
    new_y = (dby / dbx).mul_add((a0x - b0x) as f64, b0y as f64);
    dy = new_y - (min_ay as f64);
    if dy < 0.0f64 {
        if dy < (-0.1f64) {
            return 0;
        }
        new_y = min_ay as f64;
    }
    dy = new_y - (max_ay as f64);
    if dy > 0.0f64 {
        if dy > 0.1f64 {
            return 0;
        }
        new_y = max_ay as f64;
    }
    *int_x = a0x;
    *int_y = new_y as f32;
    return 1;
}

/// `mpCheckLeftWall`
pub unsafe fn mpCheckLeftWall(mut ax: f32, mut ay: f32, mut bx: f32, mut by: f32, mut vec_out: *mut Vec, mut line_id_out: *mut i32, mut flags_out: *mut u32, mut normal_out: *mut Vec, mut joint_id_skip: i32, mut joint_id_only: i32) -> i32 {
    let mut min_dist2: f32 = 0.0;
    let mut joint: *mut CollJoint = null_mut();
    let mut i: i32 = 0;
    let mut result: i32 = 0;
    let mut line: *mut CollLine = null_mut();
    let mut count: i32 = 0;
    let mut dynamic_count: i32 = 0;
    let mut j_inner: *mut MapJoint = null_mut();
    let mut already_checked: i32 = 0;
    result = 0;
    min_dist2 = f32::MAX;
    already_checked = mpCheckedBounding();
    if !(already_checked != 0) {
        mpBoundingCheck2(ax, ay, bx, by);
    }
    {
        joint = jointListStart;
        'l198: while !joint.is_null() {
            'c198: {
                if !(ext::ScriptGame_StageJointActive((joint.offset_from(groundCollJoint) as i64) as i32) != 0) {
                    break 'c198;
                }
                if ((*joint).flags & ((CollJoint_TooFar as i32) as u32)) != 0 {
                    break 'c198;
                }
                if ((joint_id_skip as i64) == (joint.offset_from(groundCollJoint) as i64)) || (!((joint_id_only == (-1_i32)) || ((joint_id_only as i64) == (joint.offset_from(groundCollJoint) as i64)))) {
                    break 'c198;
                }
                j_inner = (*joint).inner;
                count = (*j_inner).ranges[(MapLineGroup_LeftWall as i32) as usize].count as i32;
                dynamic_count = (*j_inner).ranges[(MapLineGroup_Dynamic as i32) as usize].count as i32;
                line = core::ptr::addr_of_mut!(*groundCollLine.offset((*j_inner).ranges[(MapLineGroup_LeftWall as i32) as usize].start as isize)) as *mut CollLine;
                {
                    i = 0;
                    'p199: loop {
                        'l200: while i < count {
                            if ((((*line).flags & ((CollLine_LeftWall as i32) as u32)) != 0) && (((*line).flags & ((1 << 16) as u32)) != 0)) && (!(((*line).flags & ((1 << 7) as u32)) != 0)) {
                                let mut inner: *mut MapLine = (*line).x0;
                                let mut v0: *mut CollVtx = core::ptr::addr_of_mut!(*groundCollVtx.offset((*inner).v0_idx as isize)) as *mut CollVtx;
                                let mut v1: *mut CollVtx = core::ptr::addr_of_mut!(*groundCollVtx.offset((*inner).v1_idx as isize)) as *mut CollVtx;
                                let mut x0: f32 = (*v0).pos.x;
                                let mut y0: f32 = (*v0).pos.y;
                                let mut x1: f32 = (*v1).pos.x;
                                let mut y1: f32 = (*v1).pos.y;
                                let mut dx2: f32 = 0.0;
                                let mut dy2: f32 = 0.0;
                                let mut dist2: f32 = 0.0;
                                let mut int_x: f32 = 0.0;
                                let mut int_y: f32 = 0.0;
                                if ((if (x0 - x1) < (0 as f32) { -(x0 - x1) } else { x0 - x1 }) as f64) > 0.0001f64 {
                                    if mpLineIntersection(x0, y0, x1, y1, ax, ay, bx, by, core::ptr::addr_of_mut!(int_x) as *mut f32, core::ptr::addr_of_mut!(int_y) as *mut f32) != 0 {
                                        dx2 = (int_x - ax) * (int_x - ax);
                                        dy2 = (int_y - ay) * (int_y - ay);
                                        dist2 = dx2 + dy2;
                                        if min_dist2 > dist2 {
                                            min_dist2 = dist2;
                                            if !vec_out.is_null() {
                                                (*vec_out).x = int_x;
                                                (*vec_out).y = int_y;
                                                (*vec_out).z = 0.0f32;
                                            }
                                            if !line_id_out.is_null() {
                                                *line_id_out = (line.offset_from(groundCollLine) as i64) as i32;
                                            }
                                            if !flags_out.is_null() {
                                                *flags_out = (*(*line).x0).lo_flags as u32;
                                            }
                                            if !normal_out.is_null() {
                                                (*normal_out).x = -(y1 - y0);
                                                (*normal_out).y = x1 - x0;
                                                (*normal_out).z = 0.0f32;
                                                { let __n = melee_runtime::mtx::vec_normalize(&melee_core::vec::Vec3 { x: (*normal_out).x, y: (*normal_out).y, z: (*normal_out).z }); (*normal_out).x = __n.x; (*normal_out).y = __n.y; (*normal_out).z = __n.z; };
                                            }
                                            result = 1;
                                        }
                                    }
                                } else {
                                    if (ax <= bx) && (mpLineIntersectionV(core::ptr::addr_of_mut!(int_x) as *mut f32, core::ptr::addr_of_mut!(int_y) as *mut f32, x0, y0, y1, ax, ay, bx, by) != 0) {
                                        dx2 = (int_x - ax) * (int_x - ax);
                                        dy2 = (int_y - ay) * (int_y - ay);
                                        dist2 = dx2 + dy2;
                                        if min_dist2 > dist2 {
                                            min_dist2 = dist2;
                                            if !vec_out.is_null() {
                                                (*vec_out).x = int_x;
                                                (*vec_out).y = int_y;
                                                (*vec_out).z = 0.0f32;
                                            }
                                            if !line_id_out.is_null() {
                                                *line_id_out = (line.offset_from(groundCollLine) as i64) as i32;
                                            }
                                            if !flags_out.is_null() {
                                                *flags_out = (*(*line).x0).lo_flags as u32;
                                            }
                                            if !normal_out.is_null() {
                                                (*normal_out).x = -1.0f32;
                                                (*normal_out).y = 0.0f32;
                                                (*normal_out).z = 0.0f32;
                                            }
                                            result = 1;
                                        }
                                    }
                                }
                            }
                            i = i + 1; line = line.offset(1);
                        }
                        if dynamic_count != 0 {
                            count = dynamic_count;
                            i = 0;
                            dynamic_count = 0;
                            line = core::ptr::addr_of_mut!(*groundCollLine.offset((*(*joint).inner).ranges[(MapLineGroup_Dynamic as i32) as usize].start as isize)) as *mut CollLine;
                            continue 'p199;
                        }
                        break;
                    }
                }
            }
            joint = (*joint).next;
        }
    }
    if !(already_checked != 0) {
        mpUncheckBounding();
    }
    return result;
}

/// `mpCheckLeftWallRemap`
pub unsafe fn mpCheckLeftWallRemap(mut ax: f32, mut ay: f32, mut bx: f32, mut by: f32, mut vec_out: *mut Vec, mut line_id_out: *mut i32, mut flags_out: *mut u32, mut normal_out: *mut Vec, mut joint_id_skip: i32, mut joint_id_only: i32) -> i32 {
    let mut min_dist2: f32 = 0.0;
    let mut old_x: f32 = ax;
    let mut old_y: f32 = ay;
    let mut joint: *mut CollJoint = null_mut();
    let mut i: i32 = 0;
    let mut result: i32 = 0;
    let mut line: *mut CollLine = null_mut();
    let mut count: i32 = 0;
    let mut dynamic_count: i32 = 0;
    let mut j_inner: *mut MapJoint = null_mut();
    let mut already_checked: i32 = 0;
    result = 0;
    min_dist2 = f32::MAX;
    already_checked = mpCheckedBounding();
    if !(already_checked != 0) {
        mpBoundingCheck2(ax, ay, bx, by);
    }
    {
        joint = jointListStart;
        'l201: while !joint.is_null() {
            'c201: {
                if !(ext::ScriptGame_StageJointActive((joint.offset_from(groundCollJoint) as i64) as i32) != 0) {
                    break 'c201;
                }
                if ((*joint).flags & ((CollJoint_TooFar as i32) as u32)) != 0 {
                    break 'c201;
                }
                if ((joint_id_skip as i64) == (joint.offset_from(groundCollJoint) as i64)) || (!((joint_id_only == (-1_i32)) || ((joint_id_only as i64) == (joint.offset_from(groundCollJoint) as i64)))) {
                    break 'c201;
                }
                j_inner = (*joint).inner;
                count = (*j_inner).ranges[(MapLineGroup_LeftWall as i32) as usize].count as i32;
                dynamic_count = (*j_inner).ranges[(MapLineGroup_Dynamic as i32) as usize].count as i32;
                line = core::ptr::addr_of_mut!(*groundCollLine.offset((*j_inner).ranges[(MapLineGroup_LeftWall as i32) as usize].start as isize)) as *mut CollLine;
                {
                    i = 0;
                    'p202: loop {
                        'l203: while i < count {
                            if ((((*line).flags & ((CollLine_LeftWall as i32) as u32)) != 0) && (((*line).flags & ((1 << 16) as u32)) != 0)) && (!(((*line).flags & ((1 << 7) as u32)) != 0)) {
                                let mut x0: f32 = (*groundCollVtx.offset((*(*line).x0).v0_idx as isize)).pos.x;
                                let mut y0: f32 = (*groundCollVtx.offset((*(*line).x0).v0_idx as isize)).pos.y;
                                let mut x1: f32 = (*groundCollVtx.offset((*(*line).x0).v1_idx as isize)).pos.x;
                                let mut y1: f32 = (*groundCollVtx.offset((*(*line).x0).v1_idx as isize)).pos.y;
                                let mut dx: f32 = 0.0;
                                let mut dy: f32 = 0.0;
                                let mut dx2: f32 = 0.0;
                                let mut dy2: f32 = 0.0;
                                let mut dist2: f32 = 0.0;
                                let mut int_x: f32 = 0.0;
                                let mut int_y: f32 = 0.0;
                                if ((*joint).flags & ((((CollJoint_B10 as i32) | (CollJoint_B9 as i32)) | (CollJoint_B8 as i32)) as u32)) != 0 {
                                    mpRemap2d(core::ptr::addr_of_mut!(ax) as *mut f32, core::ptr::addr_of_mut!(ay) as *mut f32, (*groundCollVtx.offset((*(*line).x0).v0_idx as isize)).x10, (*groundCollVtx.offset((*(*line).x0).v0_idx as isize)).x14, (*groundCollVtx.offset((*(*line).x0).v1_idx as isize)).x10, (*groundCollVtx.offset((*(*line).x0).v1_idx as isize)).x14, x0, y0, x1, y1, old_x, old_y);
                                } else {
                                    ax = old_x;
                                    ay = old_y;
                                }
                                dx = bx - ax;
                                dy = by - ay;
                                if ((if (x0 - x1) < (0 as f32) { -(x0 - x1) } else { x0 - x1 }) as f64) > 0.0001f64 {
                                    if mpLineIntersection(x0, y0, x1, y1, ax, ay, bx, by, core::ptr::addr_of_mut!(int_x) as *mut f32, core::ptr::addr_of_mut!(int_y) as *mut f32) != 0 {
                                        dx2 = (int_x - old_x) * (int_x - old_x);
                                        dy2 = (int_y - old_y) * (int_y - old_y);
                                        dist2 = dx2 + dy2;
                                        if fmadds(dx, int_x - old_x, dy * (int_y - old_y)) < 0.0f32 {
                                            dist2 = -dist2;
                                        }
                                        if min_dist2 > dist2 {
                                            min_dist2 = dist2;
                                            if !vec_out.is_null() {
                                                (*vec_out).x = int_x;
                                                (*vec_out).y = int_y;
                                                (*vec_out).z = 0.0f32;
                                            }
                                            if !line_id_out.is_null() {
                                                *line_id_out = (line.offset_from(groundCollLine) as i64) as i32;
                                            }
                                            if !flags_out.is_null() {
                                                *flags_out = (*(*line).x0).lo_flags as u32;
                                            }
                                            if !normal_out.is_null() {
                                                (*normal_out).x = -(y1 - y0);
                                                (*normal_out).y = x1 - x0;
                                                (*normal_out).z = 0.0f32;
                                                { let __n = melee_runtime::mtx::vec_normalize(&melee_core::vec::Vec3 { x: (*normal_out).x, y: (*normal_out).y, z: (*normal_out).z }); (*normal_out).x = __n.x; (*normal_out).y = __n.y; (*normal_out).z = __n.z; };
                                            }
                                            result = 1;
                                        }
                                    }
                                } else {
                                    if (ax <= bx) && (mpLineIntersectionV(core::ptr::addr_of_mut!(int_x) as *mut f32, core::ptr::addr_of_mut!(int_y) as *mut f32, x0, y0, y1, ax, ay, bx, by) != 0) {
                                        dx2 = (int_x - old_x) * (int_x - old_x);
                                        dy2 = (int_y - old_y) * (int_y - old_y);
                                        dist2 = dx2 + dy2;
                                        if fmadds(dx, int_x - old_x, dy * (int_y - old_y)) < 0.0f32 {
                                            dist2 = -dist2;
                                        }
                                        if min_dist2 > dist2 {
                                            min_dist2 = dist2;
                                            if !vec_out.is_null() {
                                                (*vec_out).x = int_x;
                                                (*vec_out).y = int_y;
                                                (*vec_out).z = 0.0f32;
                                            }
                                            if !line_id_out.is_null() {
                                                *line_id_out = (line.offset_from(groundCollLine) as i64) as i32;
                                            }
                                            if !flags_out.is_null() {
                                                *flags_out = (*(*line).x0).lo_flags as u32;
                                            }
                                            if !normal_out.is_null() {
                                                (*normal_out).x = -1.0f32;
                                                (*normal_out).y = 0.0f32;
                                                (*normal_out).z = 0.0f32;
                                            }
                                            result = 1;
                                        }
                                    }
                                }
                            }
                            i = i + 1; line = line.offset(1);
                        }
                        if dynamic_count != 0 {
                            count = dynamic_count;
                            i = 0;
                            dynamic_count = 0;
                            line = core::ptr::addr_of_mut!(*groundCollLine.offset((*(*joint).inner).ranges[(MapLineGroup_Dynamic as i32) as usize].start as isize)) as *mut CollLine;
                            continue 'p202;
                        }
                        break;
                    }
                }
            }
            joint = (*joint).next;
        }
    }
    if !(already_checked != 0) {
        mpUncheckBounding();
    }
    return result;
}

/// `mpCheckRightWall`
pub unsafe fn mpCheckRightWall(mut ax: f32, mut ay: f32, mut bx: f32, mut by: f32, mut vec_out: *mut Vec, mut line_id_out: *mut i32, mut flags_out: *mut u32, mut normal_out: *mut Vec, mut joint_id_skip: i32, mut joint_id_only: i32) -> i32 {
    let mut min_dist2: f32 = 0.0;
    let mut joint: *mut CollJoint = null_mut();
    let mut i: i32 = 0;
    let mut result: i32 = 0;
    let mut line: *mut CollLine = null_mut();
    let mut count: i32 = 0;
    let mut dynamic_count: i32 = 0;
    let mut j_inner: *mut MapJoint = null_mut();
    let mut already_checked: i32 = 0;
    result = 0;
    min_dist2 = f32::MAX;
    already_checked = mpCheckedBounding();
    if !(already_checked != 0) {
        mpBoundingCheck2(ax, ay, bx, by);
    }
    {
        joint = jointListStart;
        'l204: while !joint.is_null() {
            'c204: {
                if !(ext::ScriptGame_StageJointActive((joint.offset_from(groundCollJoint) as i64) as i32) != 0) {
                    break 'c204;
                }
                if ((*joint).flags & ((CollJoint_TooFar as i32) as u32)) != 0 {
                    break 'c204;
                }
                if ((joint_id_skip as i64) == (joint.offset_from(groundCollJoint) as i64)) || (!((joint_id_only == (-1_i32)) || ((joint_id_only as i64) == (joint.offset_from(groundCollJoint) as i64)))) {
                    break 'c204;
                }
                j_inner = (*joint).inner;
                count = (*j_inner).ranges[(MapLineGroup_RightWall as i32) as usize].count as i32;
                dynamic_count = (*j_inner).ranges[(MapLineGroup_Dynamic as i32) as usize].count as i32;
                line = core::ptr::addr_of_mut!(*groundCollLine.offset((*j_inner).ranges[(MapLineGroup_RightWall as i32) as usize].start as isize)) as *mut CollLine;
                {
                    i = 0;
                    'p205: loop {
                        'l206: while i < count {
                            if ((((*line).flags & ((CollLine_RightWall as i32) as u32)) != 0) && (((*line).flags & ((1 << 16) as u32)) != 0)) && (!(((*line).flags & ((1 << 7) as u32)) != 0)) {
                                let mut inner: *mut MapLine = (*line).x0;
                                let mut v0: *mut CollVtx = core::ptr::addr_of_mut!(*groundCollVtx.offset((*inner).v0_idx as isize)) as *mut CollVtx;
                                let mut v1: *mut CollVtx = core::ptr::addr_of_mut!(*groundCollVtx.offset((*inner).v1_idx as isize)) as *mut CollVtx;
                                let mut x0: f32 = (*v0).pos.x;
                                let mut y0: f32 = (*v0).pos.y;
                                let mut x1: f32 = (*v1).pos.x;
                                let mut y1: f32 = (*v1).pos.y;
                                let mut dx2: f32 = 0.0;
                                let mut dy2: f32 = 0.0;
                                let mut dist2: f32 = 0.0;
                                let mut int_x: f32 = 0.0;
                                let mut int_y: f32 = 0.0;
                                if ((if (x0 - x1) < (0 as f32) { -(x0 - x1) } else { x0 - x1 }) as f64) > 0.0001f64 {
                                    if mpLineIntersection(x0, y0, x1, y1, ax, ay, bx, by, core::ptr::addr_of_mut!(int_x) as *mut f32, core::ptr::addr_of_mut!(int_y) as *mut f32) != 0 {
                                        dx2 = (int_x - ax) * (int_x - ax);
                                        dy2 = (int_y - ay) * (int_y - ay);
                                        dist2 = dx2 + dy2;
                                        if min_dist2 > dist2 {
                                            min_dist2 = dist2;
                                            if !vec_out.is_null() {
                                                (*vec_out).x = int_x;
                                                (*vec_out).y = int_y;
                                                (*vec_out).z = 0.0f32;
                                            }
                                            if !line_id_out.is_null() {
                                                *line_id_out = (line.offset_from(groundCollLine) as i64) as i32;
                                            }
                                            if !flags_out.is_null() {
                                                *flags_out = (*(*line).x0).lo_flags as u32;
                                            }
                                            if !normal_out.is_null() {
                                                (*normal_out).x = -(y1 - y0);
                                                (*normal_out).y = x1 - x0;
                                                (*normal_out).z = 0.0f32;
                                                { let __n = melee_runtime::mtx::vec_normalize(&melee_core::vec::Vec3 { x: (*normal_out).x, y: (*normal_out).y, z: (*normal_out).z }); (*normal_out).x = __n.x; (*normal_out).y = __n.y; (*normal_out).z = __n.z; };
                                            }
                                            result = 1;
                                        }
                                    }
                                } else {
                                    if (ax >= bx) && (mpLineIntersectionV(core::ptr::addr_of_mut!(int_x) as *mut f32, core::ptr::addr_of_mut!(int_y) as *mut f32, x0, y0, y1, ax, ay, bx, by) != 0) {
                                        dx2 = (int_x - ax) * (int_x - ax);
                                        dy2 = (int_y - ay) * (int_y - ay);
                                        dist2 = dx2 + dy2;
                                        if min_dist2 > dist2 {
                                            min_dist2 = dist2;
                                            if !vec_out.is_null() {
                                                (*vec_out).x = int_x;
                                                (*vec_out).y = int_y;
                                                (*vec_out).z = 0.0f32;
                                            }
                                            if !line_id_out.is_null() {
                                                *line_id_out = (line.offset_from(groundCollLine) as i64) as i32;
                                            }
                                            if !flags_out.is_null() {
                                                *flags_out = (*(*line).x0).lo_flags as u32;
                                            }
                                            if !normal_out.is_null() {
                                                (*normal_out).x = 1.0f32;
                                                (*normal_out).y = 0.0f32;
                                                (*normal_out).z = 0.0f32;
                                            }
                                            result = 1;
                                        }
                                    }
                                }
                            }
                            i = i + 1; line = line.offset(1);
                        }
                        if dynamic_count != 0 {
                            count = dynamic_count;
                            i = 0;
                            dynamic_count = 0;
                            line = core::ptr::addr_of_mut!(*groundCollLine.offset((*(*joint).inner).ranges[(MapLineGroup_Dynamic as i32) as usize].start as isize)) as *mut CollLine;
                            continue 'p205;
                        }
                        break;
                    }
                }
            }
            joint = (*joint).next;
        }
    }
    if !(already_checked != 0) {
        mpUncheckBounding();
    }
    return result;
}

/// `mpCheckRightWallRemap`
pub unsafe fn mpCheckRightWallRemap(mut ax: f32, mut ay: f32, mut bx: f32, mut by: f32, mut vec_out: *mut Vec, mut line_id_out: *mut i32, mut flags_out: *mut u32, mut normal_out: *mut Vec, mut joint_id_skip: i32, mut joint_id_only: i32) -> i32 {
    let mut min_dist2: f32 = 0.0;
    let mut old_x: f32 = ax;
    let mut old_y: f32 = ay;
    let mut joint: *mut CollJoint = null_mut();
    let mut i: i32 = 0;
    let mut result: i32 = 0;
    let mut already_checked: i32 = 0;
    result = 0;
    min_dist2 = f32::MAX;
    already_checked = mpCheckedBounding();
    if !(already_checked != 0) {
        mpBoundingCheck2(ax, ay, bx, by);
    }
    {
        joint = jointListStart;
        'l207: while !joint.is_null() {
            'c207: {
                if !(ext::ScriptGame_StageJointActive((joint.offset_from(groundCollJoint) as i64) as i32) != 0) {
                    break 'c207;
                }
                let mut line: *mut CollLine = null_mut();
                let mut count: i32 = 0;
                let mut dynamic_count: i32 = 0;
                let mut j_inner: *mut MapJoint = null_mut();
                if ((*joint).flags & ((CollJoint_TooFar as i32) as u32)) != 0 {
                    break 'c207;
                }
                if ((joint_id_skip as i64) == (joint.offset_from(groundCollJoint) as i64)) || (!((joint_id_only == (-1_i32)) || ((joint_id_only as i64) == (joint.offset_from(groundCollJoint) as i64)))) {
                    break 'c207;
                }
                j_inner = (*joint).inner;
                count = (*j_inner).ranges[(MapLineGroup_RightWall as i32) as usize].count as i32;
                dynamic_count = (*j_inner).ranges[(MapLineGroup_Dynamic as i32) as usize].count as i32;
                line = core::ptr::addr_of_mut!(*groundCollLine.offset((*j_inner).ranges[(MapLineGroup_RightWall as i32) as usize].start as isize)) as *mut CollLine;
                {
                    i = 0;
                    'p208: loop {
                        'l209: while i < count {
                            if ((((*line).flags & ((CollLine_RightWall as i32) as u32)) != 0) && (((*line).flags & ((1 << 16) as u32)) != 0)) && (!(((*line).flags & ((1 << 7) as u32)) != 0)) {
                                let mut x0: f32 = (*groundCollVtx.offset((*(*line).x0).v0_idx as isize)).pos.x;
                                let mut y0: f32 = (*groundCollVtx.offset((*(*line).x0).v0_idx as isize)).pos.y;
                                let mut x1: f32 = (*groundCollVtx.offset((*(*line).x0).v1_idx as isize)).pos.x;
                                let mut y1: f32 = (*groundCollVtx.offset((*(*line).x0).v1_idx as isize)).pos.y;
                                let mut dx: f32 = 0.0;
                                let mut dy: f32 = 0.0;
                                let mut dx2: f32 = 0.0;
                                let mut dy2: f32 = 0.0;
                                let mut dist2: f32 = 0.0;
                                let mut int_x: f32 = 0.0;
                                let mut int_y: f32 = 0.0;
                                if ((*joint).flags & ((((CollJoint_B10 as i32) | (CollJoint_B9 as i32)) | (CollJoint_B8 as i32)) as u32)) != 0 {
                                    mpRemap2d(core::ptr::addr_of_mut!(ax) as *mut f32, core::ptr::addr_of_mut!(ay) as *mut f32, (*groundCollVtx.offset((*(*line).x0).v0_idx as isize)).x10, (*groundCollVtx.offset((*(*line).x0).v0_idx as isize)).x14, (*groundCollVtx.offset((*(*line).x0).v1_idx as isize)).x10, (*groundCollVtx.offset((*(*line).x0).v1_idx as isize)).x14, x0, y0, x1, y1, old_x, old_y);
                                } else {
                                    ax = old_x;
                                    ay = old_y;
                                }
                                dx = bx - ax;
                                dy = by - ay;
                                if ((if (x0 - x1) < (0 as f32) { -(x0 - x1) } else { x0 - x1 }) as f64) > 0.0001f64 {
                                    if mpLineIntersection(x0, y0, x1, y1, ax, ay, bx, by, core::ptr::addr_of_mut!(int_x) as *mut f32, core::ptr::addr_of_mut!(int_y) as *mut f32) != 0 {
                                        dx2 = (int_x - old_x) * (int_x - old_x);
                                        dy2 = (int_y - old_y) * (int_y - old_y);
                                        dist2 = dx2 + dy2;
                                        if fmadds(dx, int_x - old_x, dy * (int_y - old_y)) < 0.0f32 {
                                            dist2 = -dist2;
                                        }
                                        if min_dist2 > dist2 {
                                            min_dist2 = dist2;
                                            if !vec_out.is_null() {
                                                (*vec_out).x = int_x;
                                                (*vec_out).y = int_y;
                                                (*vec_out).z = 0.0f32;
                                            }
                                            if !line_id_out.is_null() {
                                                *line_id_out = (line.offset_from(groundCollLine) as i64) as i32;
                                            }
                                            if !flags_out.is_null() {
                                                *flags_out = (*(*line).x0).lo_flags as u32;
                                            }
                                            if !normal_out.is_null() {
                                                (*normal_out).x = -(y1 - y0);
                                                (*normal_out).y = x1 - x0;
                                                (*normal_out).z = 0.0f32;
                                                { let __n = melee_runtime::mtx::vec_normalize(&melee_core::vec::Vec3 { x: (*normal_out).x, y: (*normal_out).y, z: (*normal_out).z }); (*normal_out).x = __n.x; (*normal_out).y = __n.y; (*normal_out).z = __n.z; };
                                            }
                                            result = 1;
                                        }
                                    }
                                } else {
                                    if (ax >= bx) && (mpLineIntersectionV(core::ptr::addr_of_mut!(int_x) as *mut f32, core::ptr::addr_of_mut!(int_y) as *mut f32, x0, y0, y1, ax, ay, bx, by) != 0) {
                                        dx2 = (int_x - old_x) * (int_x - old_x);
                                        dy2 = (int_y - old_y) * (int_y - old_y);
                                        dist2 = dx2 + dy2;
                                        if fmadds(dx, int_x - old_x, dy * (int_y - old_y)) < 0.0f32 {
                                            dist2 = -dist2;
                                        }
                                        if min_dist2 > dist2 {
                                            min_dist2 = dist2;
                                            if !vec_out.is_null() {
                                                (*vec_out).x = int_x;
                                                (*vec_out).y = int_y;
                                                (*vec_out).z = 0.0f32;
                                            }
                                            if !line_id_out.is_null() {
                                                *line_id_out = (line.offset_from(groundCollLine) as i64) as i32;
                                            }
                                            if !flags_out.is_null() {
                                                *flags_out = (*(*line).x0).lo_flags as u32;
                                            }
                                            if !normal_out.is_null() {
                                                (*normal_out).x = 1.0f32;
                                                (*normal_out).y = 0.0f32;
                                                (*normal_out).z = 0.0f32;
                                            }
                                            result = 1;
                                        }
                                    }
                                }
                            }
                            i = i + 1; line = line.offset(1);
                        }
                        if dynamic_count != 0 {
                            count = dynamic_count;
                            i = 0;
                            dynamic_count = 0;
                            line = core::ptr::addr_of_mut!(*groundCollLine.offset((*(*joint).inner).ranges[(MapLineGroup_Dynamic as i32) as usize].start as isize)) as *mut CollLine;
                            continue 'p208;
                        }
                        break;
                    }
                }
            }
            joint = (*joint).next;
        }
    }
    if !(already_checked != 0) {
        mpUncheckBounding();
    }
    return result;
}

/// `mpLib_800511A4_RightWall`
pub unsafe fn mpLib_800511A4_RightWall(mut ax: f32, mut ay: f32, mut bx: f32, mut by: f32, mut cx: f32, mut cy: f32, mut dx: f32, mut dy: f32, mut line_id_out: *mut i32, mut joint_id_skip: i32, mut joint_id_only: i32) -> i32 {
    let mut min_dist2: f32 = 0.0;
    let mut joint: *mut CollJoint = null_mut();
    let mut i: i32 = 0;
    let mut result: i32 = 0;
    let mut already_checked: i32 = 0;
    result = 0;
    min_dist2 = f32::MAX;
    already_checked = mpCheckedBounding();
    if !(already_checked != 0) {
        mpBoundingCheck3(ax, ay, bx, by, cx, cy, dx, dy);
    }
    {
        joint = jointListStart;
        'l210: while !joint.is_null() {
            'c210: {
                if !(ext::ScriptGame_StageJointActive((joint.offset_from(groundCollJoint) as i64) as i32) != 0) {
                    break 'c210;
                }
                let mut line: *mut CollLine = null_mut();
                let mut count: i32 = 0;
                let mut dynamic_count: i32 = 0;
                let mut j_inner: *mut MapJoint = null_mut();
                if ((*joint).flags & ((CollJoint_TooFar as i32) as u32)) != 0 {
                    break 'c210;
                }
                if ((joint_id_skip as i64) == (joint.offset_from(groundCollJoint) as i64)) || (!((joint_id_only == (-1_i32)) || ((joint_id_only as i64) == (joint.offset_from(groundCollJoint) as i64)))) {
                    break 'c210;
                }
                j_inner = (*joint).inner;
                count = (*j_inner).ranges[(MapLineGroup_RightWall as i32) as usize].count as i32;
                dynamic_count = (*j_inner).ranges[(MapLineGroup_Dynamic as i32) as usize].count as i32;
                line = core::ptr::addr_of_mut!(*groundCollLine.offset((*j_inner).ranges[(MapLineGroup_RightWall as i32) as usize].start as isize)) as *mut CollLine;
                {
                    i = 0;
                    'p211: loop {
                        'l212: while i < count {
                            if ((((*line).flags & ((CollLine_RightWall as i32) as u32)) != 0) && (((*line).flags & ((1 << 16) as u32)) != 0)) && (!(((*line).flags & ((1 << 7) as u32)) != 0)) {
                                let mut vtx: *mut CollVtx = null_mut();
                                let mut int_x: f32 = 0.0;
                                let mut int_y: f32 = 0.0;
                                let mut x: f32 = 0.0;
                                let mut y: f32 = 0.0;
                                let mut x0: f32 = 0.0;
                                let mut y0: f32 = 0.0;
                                let mut x1: f32 = 0.0;
                                let mut y1: f32 = 0.0;
                                let mut vdx: f32 = 0.0;
                                let mut vdy: f32 = 0.0;
                                let mut dist2: f32 = 0.0;
                                {
                                    vtx = core::ptr::addr_of_mut!(*groundCollVtx.offset((*(*line).x0).v0_idx as isize)) as *mut CollVtx;
                                    x0 = (*vtx).pos.x;
                                    y0 = (*vtx).pos.y;
                                    x1 = (*vtx).x10;
                                    y1 = (*vtx).x14;
                                    mpRemap2d(core::ptr::addr_of_mut!(x) as *mut f32, core::ptr::addr_of_mut!(y) as *mut f32, ax, ay, bx, by, cx, cy, dx, dy, x1, y1);
                                    vdx = x0 - x;
                                    vdy = y0 - y;
                                    if fmadds(vdx, vdx, vdy * vdy) > 0.001f32 {
                                        if mpLineIntersection(cx, cy, dx, dy, x, y, x0, y0, core::ptr::addr_of_mut!(int_x) as *mut f32, core::ptr::addr_of_mut!(int_y) as *mut f32) != 0 {
                                            dist2 = fmadds(int_x - x1, int_x - x1, (int_y - y1) * (int_y - y1));
                                            if fmadds(vdx, int_x - x1, vdy * (int_y - y1)) < 0.0f32 {
                                                dist2 = -dist2;
                                            }
                                            if min_dist2 > dist2 {
                                                min_dist2 = dist2;
                                                if !line_id_out.is_null() {
                                                    *line_id_out = (line.offset_from(groundCollLine) as i64) as i32;
                                                }
                                                result = 1;
                                            }
                                        }
                                    }
                                }
                                {
                                    vtx = core::ptr::addr_of_mut!(*groundCollVtx.offset((*(*line).x0).v1_idx as isize)) as *mut CollVtx;
                                    x0 = (*vtx).pos.x;
                                    y0 = (*vtx).pos.y;
                                    x1 = (*vtx).x10;
                                    y1 = (*vtx).x14;
                                    mpRemap2d(core::ptr::addr_of_mut!(x) as *mut f32, core::ptr::addr_of_mut!(y) as *mut f32, ax, ay, bx, by, cx, cy, dx, dy, x1, y1);
                                    vdx = x0 - x;
                                    vdy = y0 - y;
                                    if fmadds(vdx, vdx, vdy * vdy) > 0.001f32 {
                                        if mpLineIntersection(cx, cy, dx, dy, x, y, x0, y0, core::ptr::addr_of_mut!(int_x) as *mut f32, core::ptr::addr_of_mut!(int_y) as *mut f32) != 0 {
                                            dist2 = fmadds(int_x - x1, int_x - x1, (int_y - y1) * (int_y - y1));
                                            if fmadds(vdx, int_x - x1, vdy * (int_y - y1)) < 0.0f32 {
                                                dist2 = -dist2;
                                            }
                                            if min_dist2 > dist2 {
                                                min_dist2 = dist2;
                                                if !line_id_out.is_null() {
                                                    *line_id_out = (line.offset_from(groundCollLine) as i64) as i32;
                                                }
                                                result = 1;
                                            }
                                        }
                                    }
                                }
                            }
                            i = i + 1; line = line.offset(1);
                        }
                        if dynamic_count != 0 {
                            count = dynamic_count;
                            i = 0;
                            dynamic_count = 0;
                            line = core::ptr::addr_of_mut!(*groundCollLine.offset((*(*joint).inner).ranges[(MapLineGroup_Dynamic as i32) as usize].start as isize)) as *mut CollLine;
                            continue 'p211;
                        }
                        break;
                    }
                }
            }
            joint = (*joint).next;
        }
    }
    if !(already_checked != 0) {
        mpUncheckBounding();
    }
    return result;
}

/// `mpLib_800515A0_LeftWall`
pub unsafe fn mpLib_800515A0_LeftWall(mut a0x: f32, mut a0y: f32, mut a1x: f32, mut a1y: f32, mut b0x: f32, mut b0y: f32, mut b1x: f32, mut b1y: f32, mut line_id_out: *mut i32, mut joint_id_skip: i32, mut joint_id_only: i32) -> i32 {
    let mut min_dist2: f32 = 0.0;
    let mut joint: *mut CollJoint = null_mut();
    let mut i: i32 = 0;
    let mut result: i32 = 0;
    let mut already_checked: i32 = 0;
    result = 0;
    min_dist2 = f32::MAX;
    already_checked = mpCheckedBounding();
    if !(already_checked != 0) {
        mpBoundingCheck3(a0x, a0y, a1x, a1y, b0x, b0y, b1x, b1y);
    }
    {
        joint = jointListStart;
        'l213: while !joint.is_null() {
            'c213: {
                if !(ext::ScriptGame_StageJointActive((joint.offset_from(groundCollJoint) as i64) as i32) != 0) {
                    break 'c213;
                }
                let mut line: *mut CollLine = null_mut();
                let mut count: i32 = 0;
                let mut dynamic_count: i32 = 0;
                let mut j_inner: *mut MapJoint = null_mut();
                if ((*joint).flags & ((CollJoint_TooFar as i32) as u32)) != 0 {
                    break 'c213;
                }
                if ((joint_id_skip as i64) == (joint.offset_from(groundCollJoint) as i64)) || (!((joint_id_only == (-1_i32)) || ((joint_id_only as i64) == (joint.offset_from(groundCollJoint) as i64)))) {
                    break 'c213;
                }
                j_inner = (*joint).inner;
                count = (*j_inner).ranges[(MapLineGroup_LeftWall as i32) as usize].count as i32;
                dynamic_count = (*j_inner).ranges[(MapLineGroup_Dynamic as i32) as usize].count as i32;
                line = core::ptr::addr_of_mut!(*groundCollLine.offset((*j_inner).ranges[(MapLineGroup_LeftWall as i32) as usize].start as isize)) as *mut CollLine;
                {
                    i = 0;
                    'p214: loop {
                        'l215: while i < count {
                            if ((((*line).flags & ((CollLine_LeftWall as i32) as u32)) != 0) && (((*line).flags & ((1 << 16) as u32)) != 0)) && (!(((*line).flags & ((1 << 7) as u32)) != 0)) {
                                let mut vtx: *mut CollVtx = null_mut();
                                let mut int_x: f32 = 0.0;
                                let mut int_y: f32 = 0.0;
                                let mut x: f32 = 0.0;
                                let mut y: f32 = 0.0;
                                let mut x0: f32 = 0.0;
                                let mut y0: f32 = 0.0;
                                let mut x1: f32 = 0.0;
                                let mut y1: f32 = 0.0;
                                let mut vdx: f32 = 0.0;
                                let mut vdy: f32 = 0.0;
                                let mut dist2: f32 = 0.0;
                                {
                                    vtx = core::ptr::addr_of_mut!(*groundCollVtx.offset((*(*line).x0).v0_idx as isize)) as *mut CollVtx;
                                    x0 = (*vtx).pos.x;
                                    y0 = (*vtx).pos.y;
                                    x1 = (*vtx).x10;
                                    y1 = (*vtx).x14;
                                    mpRemap2d(core::ptr::addr_of_mut!(x) as *mut f32, core::ptr::addr_of_mut!(y) as *mut f32, a0x, a0y, a1x, a1y, b0x, b0y, b1x, b1y, x1, y1);
                                    vdx = x0 - x;
                                    vdy = y0 - y;
                                    if fmadds(vdx, vdx, vdy * vdy) > 0.001f32 {
                                        if mpLineIntersection(b0x, b0y, b1x, b1y, x, y, x0, y0, core::ptr::addr_of_mut!(int_x) as *mut f32, core::ptr::addr_of_mut!(int_y) as *mut f32) != 0 {
                                            dist2 = fmadds(int_x - x1, int_x - x1, (int_y - y1) * (int_y - y1));
                                            if fmadds(vdx, int_x - x1, vdy * (int_y - y1)) < 0.0f32 {
                                                dist2 = -dist2;
                                            }
                                            if min_dist2 > dist2 {
                                                min_dist2 = dist2;
                                                if !line_id_out.is_null() {
                                                    *line_id_out = (line.offset_from(groundCollLine) as i64) as i32;
                                                }
                                                result = 1;
                                            }
                                        }
                                    }
                                }
                                {
                                    vtx = core::ptr::addr_of_mut!(*groundCollVtx.offset((*(*line).x0).v1_idx as isize)) as *mut CollVtx;
                                    x0 = (*vtx).pos.x;
                                    y0 = (*vtx).pos.y;
                                    x1 = (*vtx).x10;
                                    y1 = (*vtx).x14;
                                    mpRemap2d(core::ptr::addr_of_mut!(x) as *mut f32, core::ptr::addr_of_mut!(y) as *mut f32, a0x, a0y, a1x, a1y, b0x, b0y, b1x, b1y, x1, y1);
                                    vdx = x0 - x;
                                    vdy = y0 - y;
                                    if fmadds(vdx, vdx, vdy * vdy) > 0.001f32 {
                                        if mpLineIntersection(b0x, b0y, b1x, b1y, x, y, x0, y0, core::ptr::addr_of_mut!(int_x) as *mut f32, core::ptr::addr_of_mut!(int_y) as *mut f32) != 0 {
                                            dist2 = fmadds(int_x - x1, int_x - x1, (int_y - y1) * (int_y - y1));
                                            if fmadds(vdx, int_x - x1, vdy * (int_y - y1)) < 0.0f32 {
                                                dist2 = -dist2;
                                            }
                                            if min_dist2 > dist2 {
                                                min_dist2 = dist2;
                                                if !line_id_out.is_null() {
                                                    *line_id_out = (line.offset_from(groundCollLine) as i64) as i32;
                                                }
                                                result = 1;
                                            }
                                        }
                                    }
                                }
                            }
                            i = i + 1; line = line.offset(1);
                        }
                        if dynamic_count != 0 {
                            count = dynamic_count;
                            i = 0;
                            dynamic_count = 0;
                            line = core::ptr::addr_of_mut!(*groundCollLine.offset((*(*joint).inner).ranges[(MapLineGroup_Dynamic as i32) as usize].start as isize)) as *mut CollLine;
                            continue 'p214;
                        }
                        break;
                    }
                }
            }
            joint = (*joint).next;
        }
    }
    if !(already_checked != 0) {
        mpUncheckBounding();
    }
    return result;
}

/// `mpLib_8005199C_Floor`
pub unsafe fn mpLib_8005199C_Floor(mut vec: *mut Vec, mut joint_id_skip: i32, mut joint_id_only: i32) -> i32 {
    let mut line_id: i32 = (-1_i32);
    let mut x: f32 = (*vec).x;
    let mut y: f32 = (*vec).y;
    let mut already_checked: i32 = mpCheckedBounding();
    let mut joint: *mut CollJoint = null_mut();
    if !(already_checked != 0) {
        mpBoundingCheck2(x, y, x, y - 3e+04f32);
    }
    'b216: {
        {
            joint = jointListStart;
            'l217: while !joint.is_null() {
                'c217: {
                    if !(ext::ScriptGame_StageJointActive((joint.offset_from(groundCollJoint) as i64) as i32) != 0) {
                        break 'c217;
                    }
                    if (((*joint).flags & ((CollJoint_TooFar as i32) as u32)) != 0) || ((joint_id_skip as i64) == (joint.offset_from(groundCollJoint) as i64)) {
                        break 'c217;
                    }
                    if (joint_id_only != (-1_i32)) && ((joint_id_only as i64) != (joint.offset_from(groundCollJoint) as i64)) {
                        break 'c217;
                    }
                    {
                        let mut j_inner: *mut MapJoint = (*joint).inner;
                        let mut i: i32 = 0;
                        let mut line: *mut CollLine = core::ptr::addr_of_mut!(*groundCollLine.offset((*j_inner).ranges[(MapLineGroup_Floor as i32) as usize].start as isize)) as *mut CollLine;
                        let mut count: i32 = (*j_inner).ranges[(MapLineGroup_Floor as i32) as usize].count as i32;
                        let mut dynamic_count: i32 = (*j_inner).ranges[(MapLineGroup_Dynamic as i32) as usize].count as i32;
                        {
                            i = 0;
                            'p218: loop {
                                'l219: while i < count {
                                    if ((((*line).flags & ((CollLine_Floor as i32) as u32)) != 0) && (((*line).flags & ((1 << 16) as u32)) != 0)) && (!(((*line).flags & ((1 << 7) as u32)) != 0)) {
                                        let mut x0: f32 = (*groundCollVtx.offset((*(*line).x0).v0_idx as isize)).pos.x;
                                        let mut y0: f32 = (*groundCollVtx.offset((*(*line).x0).v0_idx as isize)).pos.y;
                                        let mut x1: f32 = (*groundCollVtx.offset((*(*line).x0).v1_idx as isize)).pos.x;
                                        let mut y1: f32 = (*groundCollVtx.offset((*(*line).x0).v1_idx as isize)).pos.y;
                                        if (x >= x0) && (x <= x1) {
                                            if (y >= y0) && (y >= y1) {
                                                line_id = (line.offset_from(groundCollLine) as i64) as i32;
                                                break 'b216;
                                            }
                                            if ((if (x1 - x0) < (0 as f32) { -(x1 - x0) } else { x1 - x0 }) as f64) > 0.0001f64 {
                                                let mut dx: f32 = x1 - x0;
                                                let mut dy: f32 = y1 - y0;
                                                if y >= fmadds(dy / dx, x - x0, y0) {
                                                    line_id = (line.offset_from(groundCollLine) as i64) as i32;
                                                    break 'b216;
                                                }
                                            }
                                        }
                                    }
                                    i = i + 1; line = line.offset(1);
                                }
                                if dynamic_count != 0 {
                                    count = dynamic_count;
                                    i = 0;
                                    line = core::ptr::addr_of_mut!(*groundCollLine.offset((*j_inner).ranges[(MapLineGroup_Dynamic as i32) as usize].start as isize)) as *mut CollLine;
                                    dynamic_count = 0;
                                    continue 'p218;
                                }
                                break;
                            }
                        }
                    }
                }
                joint = (*joint).next;
            }
        }
    }
    if !(already_checked != 0) {
        mpUncheckBounding();
    }
    return line_id;
}

/// `mpLib_80051BA8_Floor`
pub unsafe fn mpLib_80051BA8_Floor(mut out_vec: *mut Vec, mut line_id_skip: i32, mut joint_id_skip: i32, mut joint_id_only: i32, mut dir: i32, mut left: f32, mut bottom: f32, mut right: f32, mut top: f32) -> i32 {
    let mut min: f32 = 0.0;
    let mut out_x: f32 = 0.0;
    let mut out_y: f32 = 0.0;
    let mut ledge_id: i32 = (-1_i32);
    let mut already_checked: i32 = 0;
    let mut new_id: i32 = 0;
    let mut joint: *mut CollJoint = null_mut();
    if dir > 0 {
        min = f32::MAX;
    } else if dir < 0 {
        min = -f32::MAX;
    } else {
        if 0 != 0 { (); } else { panic!("mpLib_80051BA8_Floor: assertion failed"); }
    }
    already_checked = mpCheckedBounding();
    if !(already_checked != 0) {
        mpBoundingCheck(left, bottom, right, top);
    }
    {
        joint = jointListStart;
        'l220: while !joint.is_null() {
            'c220: {
                if !(ext::ScriptGame_StageJointActive((joint.offset_from(groundCollJoint) as i64) as i32) != 0) {
                    break 'c220;
                }
                if ((*joint).flags & ((CollJoint_TooFar as i32) as u32)) != 0 {
                    break 'c220;
                }
                if (joint_id_skip as i64) == (joint.offset_from(groundCollJoint) as i64) {
                    break 'c220;
                }
                if (joint_id_only == (-1_i32)) || ((joint_id_only as i64) == (joint.offset_from(groundCollJoint) as i64)) {
                    let mut i: i32 = 0;
                    let mut line: *mut CollLine = core::ptr::addr_of_mut!(*groundCollLine.offset((*(*joint).inner).ranges[(MapLineGroup_Floor as i32) as usize].start as isize)) as *mut CollLine;
                    let mut count: i32 = (*(*joint).inner).ranges[(MapLineGroup_Floor as i32) as usize].count as i32;
                    let mut dynamic_cout: i32 = (*(*joint).inner).ranges[(MapLineGroup_Dynamic as i32) as usize].count as i32;
                    {
                        i = 0;
                        'p221: loop {
                            'l222: while i < count {
                                'c222: {
                                    new_id = (line.offset_from(groundCollLine) as i64) as i32;
                                    if line_id_skip == new_id {
                                        break 'c222;
                                    }
                                    if ((((*line).flags & ((CollLine_Floor as i32) as u32)) != 0) && (((*line).flags & ((1 << 16) as u32)) != 0)) && (!(((*line).flags & ((1 << 7) as u32)) != 0)) {
                                        let mut inner: *mut MapLine = (*line).x0;
                                        if (((*inner).lo_flags as i32) & (1 << 9)) != 0 {
                                            let mut x0: f32 = (*groundCollVtx.offset((*inner).v0_idx as isize)).pos.x;
                                            let mut y0: f32 = (*groundCollVtx.offset((*inner).v0_idx as isize)).pos.y;
                                            let mut x1: f32 = (*groundCollVtx.offset((*inner).v1_idx as isize)).pos.x;
                                            let mut y1: f32 = (*groundCollVtx.offset((*inner).v1_idx as isize)).pos.y;
                                            let mut line_left: f32 = 0.0;
                                            let mut line_bottom: f32 = 0.0;
                                            let mut line_right: f32 = 0.0;
                                            let mut line_top: f32 = 0.0;
                                            let mut dist_h: f32 = 0.0;
                                            if x0 > x1 {
                                                line_right = x0;
                                                line_left = x1;
                                            } else {
                                                line_left = x0;
                                                line_right = x1;
                                            }
                                            if y0 > y1 {
                                                line_top = y0;
                                                line_bottom = y1;
                                            } else {
                                                line_bottom = y0;
                                                line_top = y1;
                                            }
                                            dist_h = if ((line_right + line_left) - (right + left)) < (0 as f32) { -((line_right + line_left) - (right + left)) } else { (line_right + line_left) - (right + left) };
                                            if dist_h < ((line_right - line_left) + (right - left)) {
                                                let mut dist_v: f32 = if ((line_top + line_bottom) - (top + bottom)) < (0 as f32) { -((line_top + line_bottom) - (top + bottom)) } else { (line_top + line_bottom) - (top + bottom) };
                                                if dist_v < ((line_top - line_bottom) + (top - bottom)) {
                                                    if dir > 0 {
                                                        if min > x0 {
                                                            min = x0;
                                                            ledge_id = new_id;
                                                            if !out_vec.is_null() {
                                                                out_x = x0;
                                                                out_y = y0;
                                                            }
                                                        }
                                                    } else if dir < 0 {
                                                        if min < x1 {
                                                            min = x1;
                                                            ledge_id = new_id;
                                                            if !out_vec.is_null() {
                                                                out_x = x1;
                                                                out_y = y1;
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                i = i + 1; line = line.offset(1);
                            }
                            if dynamic_cout != 0 {
                                count = dynamic_cout;
                                i = 0;
                                line = core::ptr::addr_of_mut!(*groundCollLine.offset((*(*joint).inner).ranges[(MapLineGroup_Dynamic as i32) as usize].start as isize)) as *mut CollLine;
                                dynamic_cout = 0;
                                continue 'p221;
                            }
                            break;
                        }
                    }
                }
            }
            joint = (*joint).next;
        }
    }
    if !(already_checked != 0) {
        mpUncheckBounding();
    }
    if (ledge_id != (-1_i32)) && (!out_vec.is_null()) {
        if out_x > right {
            out_x = right;
        } else if out_x < left {
            out_x = left;
        }
        (*out_vec).x = out_x;
        (*out_vec).y = out_y;
        (*out_vec).z = 0.0f32;
    }
    return ledge_id;
}

/// `mpCheckMultiple`
pub unsafe fn mpCheckMultiple(mut x0: f32, mut y0: f32, mut x1: f32, mut y1: f32, mut pos_out: *mut Vec, mut line_id_out: *mut i32, mut flags_out: *mut u32, mut normal_out: *mut Vec, mut checks: u32, mut joint_id_skip: i32, mut joint_id_only: i32) -> i32 {
    let mut dx: f32 = 0.0;
    let mut dy: f32 = 0.0;
    let mut min_dist2: f32 = 0.0;
    let mut already_checked: i32 = 0;
    let mut line_id: i32 = 0;
    let mut flags: u32 = 0;
    let mut temp_pos: Vec3 = unsafe { core::mem::zeroed() };
    let mut temp_normal: Vec3 = unsafe { core::mem::zeroed() };
    let mut pos: Vec3 = unsafe { core::mem::zeroed() };
    let mut normal: Vec3 = unsafe { core::mem::zeroed() };
    let mut temp_line_id: i32 = 0;
    let mut temp_flags: u32 = 0;
    min_dist2 = f32::MAX;
    already_checked = mpCheckedBounding();
    if !(already_checked != 0) {
        mpBoundingCheck2(x0, y0, x1, y1);
    }
    if (checks & (16 as u32)) != 0 {
        if ((checks & (1 as u32)) != 0) && (mpCheckFloorRemap(x0, y0, x1, y1, 0.0f32, core::ptr::addr_of_mut!(temp_pos) as *mut Vec, core::ptr::addr_of_mut!(temp_line_id) as *mut i32, core::ptr::addr_of_mut!(temp_flags) as *mut u32, core::ptr::addr_of_mut!(temp_normal) as *mut Vec, (-1_i32), joint_id_skip, joint_id_only, None, null_mut()) != 0) {
            dx = (temp_pos.x - x0) * (temp_pos.x - x0);
            dy = (temp_pos.y - y0) * (temp_pos.y - y0);
            pos = temp_pos;
            normal = temp_normal;
            min_dist2 = dx + dy;
            line_id = temp_line_id;
            flags = temp_flags;
        }
        if ((checks & (2 as u32)) != 0) && (mpCheckCeilingRemap(x0, y0, x1, y1, core::ptr::addr_of_mut!(temp_pos) as *mut Vec, core::ptr::addr_of_mut!(temp_line_id) as *mut i32, core::ptr::addr_of_mut!(temp_flags) as *mut u32, core::ptr::addr_of_mut!(temp_normal) as *mut Vec, joint_id_skip, joint_id_only) != 0) {
            dx = (temp_pos.x - x0) * (temp_pos.x - x0);
            dy = (temp_pos.y - y0) * (temp_pos.y - y0);
            if min_dist2 > (dx + dy) {
                min_dist2 = dx + dy;
                pos = temp_pos;
                normal = temp_normal;
                line_id = temp_line_id;
                flags = temp_flags;
            }
        }
        if ((checks & (4 as u32)) != 0) && (mpCheckLeftWallRemap(x0, y0, x1, y1, core::ptr::addr_of_mut!(temp_pos) as *mut Vec, core::ptr::addr_of_mut!(temp_line_id) as *mut i32, core::ptr::addr_of_mut!(temp_flags) as *mut u32, core::ptr::addr_of_mut!(temp_normal) as *mut Vec, joint_id_skip, joint_id_only) != 0) {
            dx = (temp_pos.x - x0) * (temp_pos.x - x0);
            dy = (temp_pos.y - y0) * (temp_pos.y - y0);
            if min_dist2 > (dx + dy) {
                min_dist2 = dx + dy;
                pos = temp_pos;
                normal = temp_normal;
                line_id = temp_line_id;
                flags = temp_flags;
            }
        }
        if ((checks & (8 as u32)) != 0) && (mpCheckRightWallRemap(x0, y0, x1, y1, core::ptr::addr_of_mut!(temp_pos) as *mut Vec, core::ptr::addr_of_mut!(temp_line_id) as *mut i32, core::ptr::addr_of_mut!(temp_flags) as *mut u32, core::ptr::addr_of_mut!(temp_normal) as *mut Vec, joint_id_skip, joint_id_only) != 0) {
            dx = (temp_pos.x - x0) * (temp_pos.x - x0);
            dy = (temp_pos.y - y0) * (temp_pos.y - y0);
            if min_dist2 > (dx + dy) {
                min_dist2 = dx + dy;
                pos = temp_pos;
                normal = temp_normal;
                line_id = temp_line_id;
                flags = temp_flags;
            }
        }
    } else {
        if ((checks & (1 as u32)) != 0) && (mpCheckFloor(x0, y0, x1, y1, 0.0f32, core::ptr::addr_of_mut!(temp_pos) as *mut Vec, core::ptr::addr_of_mut!(temp_line_id) as *mut i32, core::ptr::addr_of_mut!(temp_flags) as *mut u32, core::ptr::addr_of_mut!(temp_normal) as *mut Vec, (-1_i32), joint_id_skip, joint_id_only, None, null_mut()) != 0) {
            dx = (temp_pos.x - x0) * (temp_pos.x - x0);
            dy = (temp_pos.y - y0) * (temp_pos.y - y0);
            pos = temp_pos;
            normal = temp_normal;
            min_dist2 = dx + dy;
            line_id = temp_line_id;
            flags = temp_flags;
        }
        if ((checks & (2 as u32)) != 0) && (mpCheckCeiling(x0, y0, x1, y1, core::ptr::addr_of_mut!(temp_pos) as *mut Vec, core::ptr::addr_of_mut!(temp_line_id) as *mut i32, core::ptr::addr_of_mut!(temp_flags) as *mut u32, core::ptr::addr_of_mut!(temp_normal) as *mut Vec, joint_id_skip, joint_id_only) != 0) {
            dx = (temp_pos.x - x0) * (temp_pos.x - x0);
            dy = (temp_pos.y - y0) * (temp_pos.y - y0);
            if min_dist2 > (dx + dy) {
                min_dist2 = dx + dy;
                pos = temp_pos;
                normal = temp_normal;
                line_id = temp_line_id;
                flags = temp_flags;
            }
        }
        if ((checks & (4 as u32)) != 0) && (mpCheckLeftWall(x0, y0, x1, y1, core::ptr::addr_of_mut!(temp_pos) as *mut Vec, core::ptr::addr_of_mut!(temp_line_id) as *mut i32, core::ptr::addr_of_mut!(temp_flags) as *mut u32, core::ptr::addr_of_mut!(temp_normal) as *mut Vec, joint_id_skip, joint_id_only) != 0) {
            dx = (temp_pos.x - x0) * (temp_pos.x - x0);
            dy = (temp_pos.y - y0) * (temp_pos.y - y0);
            if min_dist2 > (dx + dy) {
                min_dist2 = dx + dy;
                pos = temp_pos;
                normal = temp_normal;
                line_id = temp_line_id;
                flags = temp_flags;
            }
        }
        if ((checks & (8 as u32)) != 0) && (mpCheckRightWall(x0, y0, x1, y1, core::ptr::addr_of_mut!(temp_pos) as *mut Vec, core::ptr::addr_of_mut!(temp_line_id) as *mut i32, core::ptr::addr_of_mut!(temp_flags) as *mut u32, core::ptr::addr_of_mut!(temp_normal) as *mut Vec, joint_id_skip, joint_id_only) != 0) {
            dx = (temp_pos.x - x0) * (temp_pos.x - x0);
            dy = (temp_pos.y - y0) * (temp_pos.y - y0);
            if min_dist2 > (dx + dy) {
                min_dist2 = dx + dy;
                pos = temp_pos;
                normal = temp_normal;
                line_id = temp_line_id;
                flags = temp_flags;
            }
        }
    }
    if !(already_checked != 0) {
        mpUncheckBounding();
    }
    if min_dist2 < f32::MAX {
        if !pos_out.is_null() {
            *pos_out = pos;
        }
        if !line_id_out.is_null() {
            *line_id_out = line_id;
        }
        if !flags_out.is_null() {
            *flags_out = flags;
        }
        if !normal_out.is_null() {
            *normal_out = normal;
        }
        return 1;
    }
    return 0;
}

/// `mpCheckAllRemap`
pub unsafe fn mpCheckAllRemap(mut pos_out: *mut Vec, mut line_id_out: *mut i32, mut flags_out: *mut u32, mut normal_out: *mut Vec, mut joint_id_skip: i32, mut joint_id_only: i32, mut x0: f32, mut y0: f32, mut x1: f32, mut y1: f32) -> i32 {
    return mpCheckMultiple(x0, y0, x1, y1, pos_out, line_id_out, flags_out, normal_out, 31 as u32, joint_id_skip, joint_id_only);
}

/// `mpCheckAll`
pub unsafe fn mpCheckAll(mut pos_out: *mut Vec, mut line_id_out: *mut i32, mut flags_out: *mut u32, mut normal_out: *mut Vec, mut joint_id_skip: i32, mut joint_id_only: i32, mut x0: f32, mut y0: f32, mut x1: f32, mut y1: f32) -> i32 {
    return mpCheckMultiple(x0, y0, x1, y1, pos_out, line_id_out, flags_out, normal_out, 15 as u32, joint_id_skip, joint_id_only);
}

/// `mpLineGetNextCheckInline`
unsafe fn mpLineGetNextCheckInline(mut line: *mut MapLine, mut result: i16) -> i32 {
    if (result as i32) != (-1_i32) {
        let mut flags: u32 = (*groundCollLine.offset(result as isize)).flags;
        if (((flags & ((1 << 16) as u32)) != 0) && (!((flags & ((1 << 18) as u32)) != 0))) && (ext::ScriptGame_StageLineActive(result as i32) != 0) {
            let mut v1: *mut CollVtx = core::ptr::addr_of_mut!(*groundCollVtx.offset((*line).v1_idx as isize)) as *mut CollVtx;
            let mut v0: *mut CollVtx = core::ptr::addr_of_mut!(*groundCollVtx.offset((*(*groundCollLine.offset(result as isize)).x0).v0_idx as isize)) as *mut CollVtx;
            if (fmadds((*v1).pos.x - (*v0).pos.x, (*v1).pos.x - (*v0).pos.x, ((*v1).pos.y - (*v0).pos.y) * ((*v1).pos.y - (*v0).pos.y)) as f64) < 4.0f64 {
                return result as i32;
            }
        }
    }
    return if (((*line).next_id0 as i32) != (-1_i32)) && (!(ext::ScriptGame_StageLineActive((*line).next_id0 as i32) != 0)) { (-1_i32) } else { (*line).next_id0 as i32 };
}

/// `mpLineGetPrevCheckInline`
unsafe fn mpLineGetPrevCheckInline(mut line: *mut MapLine, mut result: i16) -> i32 {
    if (result as i32) != (-1_i32) {
        let mut flags: u32 = (*groundCollLine.offset(result as isize)).flags;
        if (((flags & ((1 << 16) as u32)) != 0) && (!((flags & ((1 << 18) as u32)) != 0))) && (ext::ScriptGame_StageLineActive(result as i32) != 0) {
            let mut v0: *mut CollVtx = core::ptr::addr_of_mut!(*groundCollVtx.offset((*line).v0_idx as isize)) as *mut CollVtx;
            let mut v1: *mut CollVtx = core::ptr::addr_of_mut!(*groundCollVtx.offset((*(*groundCollLine.offset(result as isize)).x0).v1_idx as isize)) as *mut CollVtx;
            if (fmadds((*v0).pos.x - (*v1).pos.x, (*v0).pos.x - (*v1).pos.x, ((*v0).pos.y - (*v1).pos.y) * ((*v0).pos.y - (*v1).pos.y)) as f64) < 4.0f64 {
                return result as i32;
            }
        }
    }
    return if (((*line).prev_id0 as i32) != (-1_i32)) && (!(ext::ScriptGame_StageLineActive((*line).prev_id0 as i32) != 0)) { (-1_i32) } else { (*line).prev_id0 as i32 };
}

/// `mpLineGetPrevCheckInlineVtx`
unsafe fn mpLineGetPrevCheckInlineVtx(mut line: *mut MapLine, mut result: i16, mut vtx: *mut CollVtx) -> i32 {
    if (result as i32) != (-1_i32) {
        let mut flags: u32 = (*groundCollLine.offset(result as isize)).flags;
        if (((flags & ((1 << 16) as u32)) != 0) && (!((flags & ((1 << 18) as u32)) != 0))) && (ext::ScriptGame_StageLineActive(result as i32) != 0) {
            let mut v0: *mut CollVtx = core::ptr::addr_of_mut!(*vtx.offset((*line).v0_idx as isize)) as *mut CollVtx;
            let mut v1: *mut CollVtx = core::ptr::addr_of_mut!(*vtx.offset((*(*groundCollLine.offset(result as isize)).x0).v1_idx as isize)) as *mut CollVtx;
            if (fmadds((*v0).pos.x - (*v1).pos.x, (*v0).pos.x - (*v1).pos.x, ((*v0).pos.y - (*v1).pos.y) * ((*v0).pos.y - (*v1).pos.y)) as f64) < 4.0f64 {
                return result as i32;
            }
        }
    }
    result = (*line).prev_id0;
    return result as i32;
}

/// `mpLineGetNextCheckInlineVtx`
unsafe fn mpLineGetNextCheckInlineVtx(mut line: *mut MapLine, mut result: i16, mut vtx: *mut CollVtx) -> i32 {
    if (result as i32) != (-1_i32) {
        let mut flags: u32 = (*groundCollLine.offset(result as isize)).flags;
        if (((flags & ((1 << 16) as u32)) != 0) && (!((flags & ((1 << 18) as u32)) != 0))) && (ext::ScriptGame_StageLineActive(result as i32) != 0) {
            let mut v1: *mut CollVtx = core::ptr::addr_of_mut!(*vtx.offset((*line).v1_idx as isize)) as *mut CollVtx;
            let mut v0: *mut CollVtx = core::ptr::addr_of_mut!(*vtx.offset((*(*groundCollLine.offset(result as isize)).x0).v0_idx as isize)) as *mut CollVtx;
            if (fmadds((*v1).pos.x - (*v0).pos.x, (*v1).pos.x - (*v0).pos.x, ((*v1).pos.y - (*v0).pos.y) * ((*v1).pos.y - (*v0).pos.y)) as f64) < 4.0f64 {
                return result as i32;
            }
        }
    }
    result = (*line).next_id0;
    return result as i32;
}

/// `mpLineIterNonResult`
unsafe fn mpLineIterNonResult(mut new_id: i32, mut line_id: i32) -> i32 {
    let mut valid_id: i32 = 0;
    if (new_id != (-1_i32)) && (new_id != line_id) {
        valid_id = 1;
    }
    if valid_id != 0 {
        return new_id;
    }
    return (-1_i32);
}

/// `mpLineNextNonFloor`
pub unsafe fn mpLineNextNonFloor(mut line_id: i32) -> i32 {
    let mut first_line: *mut MapLine = null_mut();
    let mut line: *mut MapLine = null_mut();
    let mut new_id: i32 = 0;
    {
        if (line_id == (-1_i32)) || (line_id >= (*mpLib_804D64B4).line_count) {
            if 0 != 0 { (); } else { ext::OSReport(&format!("{}:{}:not found lineID={}\n", core::ffi::CStr::from_ptr((c"src/melee/mp/mplib.c".as_ptr() as *mut i8) as *const core::ffi::c_char).to_string_lossy(), 4139, line_id)); panic!("mpLineNextNonFloor: assertion failed"); }
        }
    }
    first_line = (*groundCollLine.offset(line_id as isize)).x0;
    new_id = mpLineGetNextCheckInline(first_line, (*first_line).next_id1);
    'l223: while ((new_id != (-1_i32)) && (new_id != line_id)) && (((*groundCollLine.offset(new_id as isize)).flags & ((CollLine_Floor as i32) as u32)) != 0) {
        line = (*groundCollLine.offset(new_id as isize)).x0;
        new_id = (*line).next_id1 as i32;
        new_id = mpLineGetNextCheckInlineVtx(line, new_id as i16, groundCollVtx);
    }
    return mpLineIterNonResult(new_id, line_id);
}

/// `mpLinePrevNonFloor`
pub unsafe fn mpLinePrevNonFloor(mut line_id: i32) -> i32 {
    let mut first_line: *mut MapLine = null_mut();
    let mut line: *mut MapLine = null_mut();
    let mut new_id: i32 = 0;
    {
        if (line_id == (-1_i32)) || (line_id >= (*mpLib_804D64B4).line_count) {
            if 0 != 0 { (); } else { ext::OSReport(&format!("{}:{}:not found lineID={}\n", core::ffi::CStr::from_ptr((c"src/melee/mp/mplib.c".as_ptr() as *mut i8) as *const core::ffi::c_char).to_string_lossy(), 4148, line_id)); panic!("mpLinePrevNonFloor: assertion failed"); }
        }
    }
    first_line = (*groundCollLine.offset(line_id as isize)).x0;
    new_id = mpLineGetPrevCheckInline(first_line, (*first_line).prev_id1);
    'l224: while ((new_id != (-1_i32)) && (new_id != line_id)) && (((*groundCollLine.offset(new_id as isize)).flags & ((CollLine_Floor as i32) as u32)) != 0) {
        line = (*groundCollLine.offset(new_id as isize)).x0;
        new_id = (*line).prev_id1 as i32;
        new_id = mpLineGetPrevCheckInlineVtx(line, new_id as i16, groundCollVtx);
    }
    return mpLineIterNonResult(new_id, line_id);
}

/// `mpLinePrevNonCeiling`
pub unsafe fn mpLinePrevNonCeiling(mut line_id: i32) -> i32 {
    let mut first_line: *mut MapLine = null_mut();
    let mut line: *mut MapLine = null_mut();
    let mut new_id: i32 = 0;
    {
        if (line_id == (-1_i32)) || (line_id >= (*mpLib_804D64B4).line_count) {
            if 0 != 0 { (); } else { ext::OSReport(&format!("{}:{}:not found lineID={}\n", core::ffi::CStr::from_ptr((c"src/melee/mp/mplib.c".as_ptr() as *mut i8) as *const core::ffi::c_char).to_string_lossy(), 4157, line_id)); panic!("mpLinePrevNonCeiling: assertion failed"); }
        }
    }
    first_line = (*groundCollLine.offset(line_id as isize)).x0;
    new_id = mpLineGetPrevCheckInline(first_line, (*first_line).prev_id1);
    'l225: while ((new_id != (-1_i32)) && (new_id != line_id)) && (((*groundCollLine.offset(new_id as isize)).flags & ((CollLine_Ceiling as i32) as u32)) != 0) {
        line = (*groundCollLine.offset(new_id as isize)).x0;
        new_id = (*line).prev_id1 as i32;
        new_id = mpLineGetPrevCheckInlineVtx(line, new_id as i16, groundCollVtx);
    }
    return mpLineIterNonResult(new_id, line_id);
}

/// `mpLineNextNonCeiling`
pub unsafe fn mpLineNextNonCeiling(mut line_id: i32) -> i32 {
    let mut first_line: *mut MapLine = null_mut();
    let mut line: *mut MapLine = null_mut();
    let mut new_id: i32 = 0;
    {
        if (line_id == (-1_i32)) || (line_id >= (*mpLib_804D64B4).line_count) {
            if 0 != 0 { (); } else { ext::OSReport(&format!("{}:{}:not found lineID={}\n", core::ffi::CStr::from_ptr((c"src/melee/mp/mplib.c".as_ptr() as *mut i8) as *const core::ffi::c_char).to_string_lossy(), 4166, line_id)); panic!("mpLineNextNonCeiling: assertion failed"); }
        }
    }
    first_line = (*groundCollLine.offset(line_id as isize)).x0;
    new_id = mpLineGetNextCheckInline(first_line, (*first_line).next_id1);
    'l226: while ((new_id != (-1_i32)) && (new_id != line_id)) && (((*groundCollLine.offset(new_id as isize)).flags & ((CollLine_Ceiling as i32) as u32)) != 0) {
        line = (*groundCollLine.offset(new_id as isize)).x0;
        new_id = (*line).next_id1 as i32;
        new_id = mpLineGetNextCheckInlineVtx(line, new_id as i16, groundCollVtx);
    }
    return mpLineIterNonResult(new_id, line_id);
}

/// `mpLineNextNonLeftWall`
pub unsafe fn mpLineNextNonLeftWall(mut line_id: i32) -> i32 {
    let mut first_line: *mut MapLine = null_mut();
    let mut line: *mut MapLine = null_mut();
    let mut new_id: i32 = 0;
    {
        if (line_id == (-1_i32)) || (line_id >= (*mpLib_804D64B4).line_count) {
            if 0 != 0 { (); } else { ext::OSReport(&format!("{}:{}:not found lineID={}\n", core::ffi::CStr::from_ptr((c"src/melee/mp/mplib.c".as_ptr() as *mut i8) as *const core::ffi::c_char).to_string_lossy(), 4175, line_id)); panic!("mpLineNextNonLeftWall: assertion failed"); }
        }
    }
    first_line = (*groundCollLine.offset(line_id as isize)).x0;
    new_id = mpLineGetNextCheckInline(first_line, (*first_line).next_id1);
    'l227: while ((new_id != (-1_i32)) && (new_id != line_id)) && (((*groundCollLine.offset(new_id as isize)).flags & ((CollLine_LeftWall as i32) as u32)) != 0) {
        line = (*groundCollLine.offset(new_id as isize)).x0;
        new_id = (*line).next_id1 as i32;
        new_id = mpLineGetNextCheckInlineVtx(line, new_id as i16, groundCollVtx);
    }
    return mpLineIterNonResult(new_id, line_id);
}

/// `mpLinePrevNonLeftWall`
pub unsafe fn mpLinePrevNonLeftWall(mut line_id: i32) -> i32 {
    let mut first_line: *mut MapLine = null_mut();
    let mut line: *mut MapLine = null_mut();
    let mut new_id: i32 = 0;
    {
        if (line_id == (-1_i32)) || (line_id >= (*mpLib_804D64B4).line_count) {
            if 0 != 0 { (); } else { ext::OSReport(&format!("{}:{}:not found lineID={}\n", core::ffi::CStr::from_ptr((c"src/melee/mp/mplib.c".as_ptr() as *mut i8) as *const core::ffi::c_char).to_string_lossy(), 4184, line_id)); panic!("mpLinePrevNonLeftWall: assertion failed"); }
        }
    }
    first_line = (*groundCollLine.offset(line_id as isize)).x0;
    new_id = mpLineGetPrevCheckInline(first_line, (*first_line).prev_id1);
    'l228: while ((new_id != (-1_i32)) && (new_id != line_id)) && (((*groundCollLine.offset(new_id as isize)).flags & ((CollLine_LeftWall as i32) as u32)) != 0) {
        line = (*groundCollLine.offset(new_id as isize)).x0;
        new_id = (*line).prev_id1 as i32;
        new_id = mpLineGetPrevCheckInlineVtx(line, new_id as i16, groundCollVtx);
    }
    return mpLineIterNonResult(new_id, line_id);
}

/// `mpLinePrevNonRightWall`
pub unsafe fn mpLinePrevNonRightWall(mut line_id: i32) -> i32 {
    let mut first_line: *mut MapLine = null_mut();
    let mut line: *mut MapLine = null_mut();
    let mut new_id: i32 = 0;
    {
        if (line_id == (-1_i32)) || (line_id >= (*mpLib_804D64B4).line_count) {
            if 0 != 0 { (); } else { ext::OSReport(&format!("{}:{}:not found lineID={}\n", core::ffi::CStr::from_ptr((c"src/melee/mp/mplib.c".as_ptr() as *mut i8) as *const core::ffi::c_char).to_string_lossy(), 4193, line_id)); panic!("mpLinePrevNonRightWall: assertion failed"); }
        }
    }
    first_line = (*groundCollLine.offset(line_id as isize)).x0;
    new_id = mpLineGetPrevCheckInline(first_line, (*first_line).prev_id1);
    'l229: while ((new_id != (-1_i32)) && (new_id != line_id)) && (((*groundCollLine.offset(new_id as isize)).flags & ((CollLine_RightWall as i32) as u32)) != 0) {
        line = (*groundCollLine.offset(new_id as isize)).x0;
        new_id = (*line).prev_id1 as i32;
        new_id = mpLineGetPrevCheckInlineVtx(line, new_id as i16, groundCollVtx);
    }
    return mpLineIterNonResult(new_id, line_id);
}

/// `mpLineNextNonRightWall`
pub unsafe fn mpLineNextNonRightWall(mut line_id: i32) -> i32 {
    let mut first_line: *mut MapLine = null_mut();
    let mut line: *mut MapLine = null_mut();
    let mut new_id: i32 = 0;
    {
        if (line_id == (-1_i32)) || (line_id >= (*mpLib_804D64B4).line_count) {
            if 0 != 0 { (); } else { ext::OSReport(&format!("{}:{}:not found lineID={}\n", core::ffi::CStr::from_ptr((c"src/melee/mp/mplib.c".as_ptr() as *mut i8) as *const core::ffi::c_char).to_string_lossy(), 4202, line_id)); panic!("mpLineNextNonRightWall: assertion failed"); }
        }
    }
    first_line = (*groundCollLine.offset(line_id as isize)).x0;
    new_id = mpLineGetNextCheckInline(first_line, (*first_line).next_id1);
    'l230: while ((new_id != (-1_i32)) && (new_id != line_id)) && (((*groundCollLine.offset(new_id as isize)).flags & ((CollLine_RightWall as i32) as u32)) != 0) {
        line = (*groundCollLine.offset(new_id as isize)).x0;
        new_id = (*line).next_id1 as i32;
        new_id = mpLineGetNextCheckInlineVtx(line, new_id as i16, groundCollVtx);
    }
    return mpLineIterNonResult(new_id, line_id);
}

/// `mpLineWalkNon`
unsafe fn mpLineWalkNon(mut line_id: i32, mut kind: i32, mut next: i32) -> i32 {
    let mut new_id: i32 = 0;
    if next != 0 {
        new_id = (*(*groundCollLine.offset(line_id as isize)).x0).next_id0 as i32;
        'l231: while (new_id != (-1_i32)) && (((*groundCollLine.offset(new_id as isize)).flags & (kind as u32)) != 0) {
            new_id = (*(*groundCollLine.offset(new_id as isize)).x0).next_id0 as i32;
        }
    } else {
        new_id = (*(*groundCollLine.offset(line_id as isize)).x0).prev_id0 as i32;
        'l232: while (new_id != (-1_i32)) && (((*groundCollLine.offset(new_id as isize)).flags & (kind as u32)) != 0) {
            new_id = (*(*groundCollLine.offset(new_id as isize)).x0).prev_id0 as i32;
        }
    }
    if new_id != (-1_i32) {
        return new_id;
    }
    return (-1_i32);
}

/// `mpLib_80053394_Floor`
pub unsafe fn mpLib_80053394_Floor(mut line_id: i32) -> i32 {
    {
        if (line_id == (-1_i32)) || (line_id >= (*mpLib_804D64B4).line_count) {
            if 0 != 0 { (); } else { ext::OSReport(&format!("{}:{}:not found lineID={}\n", core::ffi::CStr::from_ptr((c"src/melee/mp/mplib.c".as_ptr() as *mut i8) as *const core::ffi::c_char).to_string_lossy(), 4252, line_id)); panic!("mpLib_80053394_Floor: assertion failed"); }
        }
    }
    return mpLineWalkNon(line_id, CollLine_Floor as i32, 1);
}

/// `mpLib_80053448_Floor`
pub unsafe fn mpLib_80053448_Floor(mut line_id: i32) -> i32 {
    {
        if (line_id == (-1_i32)) || (line_id >= (*mpLib_804D64B4).line_count) {
            if 0 != 0 { (); } else { ext::OSReport(&format!("{}:{}:not found lineID={}\n", core::ffi::CStr::from_ptr((c"src/melee/mp/mplib.c".as_ptr() as *mut i8) as *const core::ffi::c_char).to_string_lossy(), 4261, line_id)); panic!("mpLib_80053448_Floor: assertion failed"); }
        }
    }
    return mpLineWalkNon(line_id, CollLine_Floor as i32, 0);
}

/// `mpLineGetNextInline`
unsafe fn mpLineGetNextInline(mut line_id: i32) -> i32 {
    let mut result: i32 = (*(*groundCollLine.offset(line_id as isize)).x0).next_id1 as i32;
    if result != (-1_i32) {
        let mut flags: u32 = (*groundCollLine.offset(result as isize)).flags;
        if (((flags & ((1 << 16) as u32)) != 0) && (!((flags & ((1 << 18) as u32)) != 0))) && (ext::ScriptGame_StageLineActive(result) != 0) {
            let mut v1: *mut CollVtx = core::ptr::addr_of_mut!(*groundCollVtx.offset((*(*groundCollLine.offset(line_id as isize)).x0).v1_idx as isize)) as *mut CollVtx;
            let mut v0: *mut CollVtx = core::ptr::addr_of_mut!(*groundCollVtx.offset((*(*groundCollLine.offset(result as isize)).x0).v0_idx as isize)) as *mut CollVtx;
            if (fmadds((*v1).pos.x - (*v0).pos.x, (*v1).pos.x - (*v0).pos.x, ((*v1).pos.y - (*v0).pos.y) * ((*v1).pos.y - (*v0).pos.y)) as f64) < 4.0f64 {
                return result;
            }
        }
    }
    return (*(*groundCollLine.offset(line_id as isize)).x0).next_id0 as i32;
}

/// `mpLineGetNextCachedInline`
unsafe fn mpLineGetNextCachedInline(mut line_id: i32) -> i32 {
    let mut line: *mut MapLine = null_mut();
    let mut result: i32 = (*{ line = (*groundCollLine.offset(line_id as isize)).x0; line }).next_id1 as i32;
    if result != (-1_i32) {
        let mut flags: u32 = (*groundCollLine.offset(result as isize)).flags;
        if (((flags & ((1 << 16) as u32)) != 0) && (!((flags & ((1 << 18) as u32)) != 0))) && (ext::ScriptGame_StageLineActive(result) != 0) {
            let mut v1: *mut CollVtx = core::ptr::addr_of_mut!(*groundCollVtx.offset((*line).v1_idx as isize)) as *mut CollVtx;
            let mut v0: *mut CollVtx = core::ptr::addr_of_mut!(*groundCollVtx.offset((*(*groundCollLine.offset(result as isize)).x0).v0_idx as isize)) as *mut CollVtx;
            if (fmadds((*v1).pos.x - (*v0).pos.x, (*v1).pos.x - (*v0).pos.x, ((*v1).pos.y - (*v0).pos.y) * ((*v1).pos.y - (*v0).pos.y)) as f64) < 4.0f64 {
                return result;
            }
        }
    }
    return if (((*line).next_id0 as i32) != (-1_i32)) && (!(ext::ScriptGame_StageLineActive((*line).next_id0 as i32) != 0)) { (-1_i32) } else { (*line).next_id0 as i32 };
}

/// `mpLineGetNextCheckResultFirst`
unsafe fn mpLineGetNextCheckResultFirst(mut result: i32, mut line: *mut MapLine) -> i32 {
    if result != (-1_i32) {
        let mut flags: u32 = (*groundCollLine.offset(result as isize)).flags;
        if (((flags & ((1 << 16) as u32)) != 0) && (!((flags & ((1 << 18) as u32)) != 0))) && (ext::ScriptGame_StageLineActive(result) != 0) {
            let mut v1: *mut CollVtx = core::ptr::addr_of_mut!(*groundCollVtx.offset((*line).v1_idx as isize)) as *mut CollVtx;
            let mut v0: *mut CollVtx = core::ptr::addr_of_mut!(*groundCollVtx.offset((*(*groundCollLine.offset(result as isize)).x0).v0_idx as isize)) as *mut CollVtx;
            if (fmadds((*v1).pos.x - (*v0).pos.x, (*v1).pos.x - (*v0).pos.x, ((*v1).pos.y - (*v0).pos.y) * ((*v1).pos.y - (*v0).pos.y)) as f64) < 4.0f64 {
                return result;
            }
        }
    }
    return if (((*line).next_id0 as i32) != (-1_i32)) && (!(ext::ScriptGame_StageLineActive((*line).next_id0 as i32) != 0)) { (-1_i32) } else { (*line).next_id0 as i32 };
}

/// `mpLib_800534FC_Floor`
pub unsafe fn mpLib_800534FC_Floor(mut line_id: i32) -> i32 {
    let mut line: *mut MapLine = null_mut();
    let mut result: i32 = 0;
    let mut new_id: i32 = 0;
    {
        if (line_id == (-1_i32)) || (line_id >= (*mpLib_804D64B4).line_count) {
            if 0 != 0 { (); } else { ext::OSReport(&format!("{}:{}:not found lineID={}\n", core::ffi::CStr::from_ptr((c"src/melee/mp/mplib.c".as_ptr() as *mut i8) as *const core::ffi::c_char).to_string_lossy(), 4272, line_id)); panic!("mpLib_800534FC_Floor: assertion failed"); }
        }
    }
    new_id = mpLineGetNextCachedInline(line_id);
    'l233: while new_id != (-1_i32) {
        'c233: {
            if !(((*groundCollLine.offset(new_id as isize)).flags & ((CollLine_Floor as i32) as u32)) != 0) {
                new_id = (-1_i32);
            } else if new_id != ((*(*groundCollLine.offset(line_id as isize)).x0).next_id1 as i32) {
                line_id = new_id;
                line = (*groundCollLine.offset(new_id as isize)).x0;
                new_id = mpLineGetNextCheckResultFirst((*line).next_id1 as i32, line);
                break 'c233;
            }
            break 'l233;
        }
    }
    if new_id != (-1_i32) {
        result = new_id;
    } else {
        result = (-1_i32);
    }
    return result;
}

/// `mpLineGetPrevInline`
unsafe fn mpLineGetPrevInline(mut line_id: i32) -> i32 {
    let mut line: *mut MapLine = null_mut();
    let mut result: i32 = (*{ line = (*groundCollLine.offset(line_id as isize)).x0; line }).prev_id1 as i32;
    if result != (-1_i32) {
        let mut flags: u32 = (*groundCollLine.offset(result as isize)).flags;
        if (((flags & ((1 << 16) as u32)) != 0) && (!((flags & ((1 << 18) as u32)) != 0))) && (ext::ScriptGame_StageLineActive(result) != 0) {
            let mut v0: *mut CollVtx = core::ptr::addr_of_mut!(*groundCollVtx.offset((*line).v0_idx as isize)) as *mut CollVtx;
            let mut v1: *mut CollVtx = core::ptr::addr_of_mut!(*groundCollVtx.offset((*(*groundCollLine.offset(result as isize)).x0).v1_idx as isize)) as *mut CollVtx;
            if (fmadds((*v0).pos.x - (*v1).pos.x, (*v0).pos.x - (*v1).pos.x, ((*v0).pos.y - (*v1).pos.y) * ((*v0).pos.y - (*v1).pos.y)) as f64) < 4.0f64 {
                return result;
            }
        }
    }
    return if (((*line).prev_id0 as i32) != (-1_i32)) && (!(ext::ScriptGame_StageLineActive((*line).prev_id0 as i32) != 0)) { (-1_i32) } else { (*line).prev_id0 as i32 };
}

/// `mpLineGetPrevCheckResultFirst`
unsafe fn mpLineGetPrevCheckResultFirst(mut result: i32, mut line: *mut MapLine) -> i32 {
    if result != (-1_i32) {
        let mut flags: u32 = (*groundCollLine.offset(result as isize)).flags;
        if (((flags & ((1 << 16) as u32)) != 0) && (!((flags & ((1 << 18) as u32)) != 0))) && (ext::ScriptGame_StageLineActive(result) != 0) {
            let mut v0: *mut CollVtx = core::ptr::addr_of_mut!(*groundCollVtx.offset((*line).v0_idx as isize)) as *mut CollVtx;
            let mut v1: *mut CollVtx = core::ptr::addr_of_mut!(*groundCollVtx.offset((*(*groundCollLine.offset(result as isize)).x0).v1_idx as isize)) as *mut CollVtx;
            if (fmadds((*v0).pos.x - (*v1).pos.x, (*v0).pos.x - (*v1).pos.x, ((*v0).pos.y - (*v1).pos.y) * ((*v0).pos.y - (*v1).pos.y)) as f64) < 4.0f64 {
                return result;
            }
        }
    }
    return if (((*line).prev_id0 as i32) != (-1_i32)) && (!(ext::ScriptGame_StageLineActive((*line).prev_id0 as i32) != 0)) { (-1_i32) } else { (*line).prev_id0 as i32 };
}

/// `mpLib_800536CC_Floor`
pub unsafe fn mpLib_800536CC_Floor(mut line_id: i32) -> i32 {
    let mut line: *mut MapLine = null_mut();
    let mut new_id: i32 = 0;
    let mut result: i32 = 0;
    {
        if (line_id == (-1_i32)) || (line_id >= (*mpLib_804D64B4).line_count) {
            if 0 != 0 { (); } else { ext::OSReport(&format!("{}:{}:not found lineID={}\n", core::ffi::CStr::from_ptr((c"src/melee/mp/mplib.c".as_ptr() as *mut i8) as *const core::ffi::c_char).to_string_lossy(), 4293, line_id)); panic!("mpLib_800536CC_Floor: assertion failed"); }
        }
    }
    new_id = mpLineGetPrevInline(line_id);
    'l234: while new_id != (-1_i32) {
        'c234: {
            if !(((*groundCollLine.offset(new_id as isize)).flags & ((CollLine_Floor as i32) as u32)) != 0) {
                new_id = (-1_i32);
            } else if new_id != ((*(*groundCollLine.offset(line_id as isize)).x0).prev_id1 as i32) {
                line_id = new_id;
                line = (*groundCollLine.offset(new_id as isize)).x0;
                new_id = mpLineGetPrevCheckResultFirst((*line).prev_id1 as i32, line);
                break 'c234;
            }
            break 'l234;
        }
    }
    if new_id != (-1_i32) {
        result = new_id;
    } else {
        result = (-1_i32);
    }
    return result;
}

/// `mpLib_8005389C_Ceiling`
pub unsafe fn mpLib_8005389C_Ceiling(mut line_id: i32) -> i32 {
    {
        if (line_id == (-1_i32)) || (line_id >= (*mpLib_804D64B4).line_count) {
            if 0 != 0 { (); } else { ext::OSReport(&format!("{}:{}:not found lineID={}\n", core::ffi::CStr::from_ptr((c"src/melee/mp/mplib.c".as_ptr() as *mut i8) as *const core::ffi::c_char).to_string_lossy(), 4314, line_id)); panic!("mpLib_8005389C_Ceiling: assertion failed"); }
        }
    }
    return mpLineWalkNon(line_id, CollLine_Ceiling as i32, 0);
}

/// `mpLib_80053950_Ceiling`
pub unsafe fn mpLib_80053950_Ceiling(mut line_id: i32) -> i32 {
    {
        if (line_id == (-1_i32)) || (line_id >= (*mpLib_804D64B4).line_count) {
            if 0 != 0 { (); } else { ext::OSReport(&format!("{}:{}:not found lineID={}\n", core::ffi::CStr::from_ptr((c"src/melee/mp/mplib.c".as_ptr() as *mut i8) as *const core::ffi::c_char).to_string_lossy(), 4323, line_id)); panic!("mpLib_80053950_Ceiling: assertion failed"); }
        }
    }
    return mpLineWalkNon(line_id, CollLine_Ceiling as i32, 1);
}

/// `mpLib_80053A04_Ceiling`
pub unsafe fn mpLib_80053A04_Ceiling(mut line_id: i32) -> i32 {
    let mut line: *mut MapLine = null_mut();
    let mut result: i32 = 0;
    let mut new_id: i32 = 0;
    {
        if (line_id == (-1_i32)) || (line_id >= (*mpLib_804D64B4).line_count) {
            if 0 != 0 { (); } else { ext::OSReport(&format!("{}:{}:not found lineID={}\n", core::ffi::CStr::from_ptr((c"src/melee/mp/mplib.c".as_ptr() as *mut i8) as *const core::ffi::c_char).to_string_lossy(), 4334, line_id)); panic!("mpLib_80053A04_Ceiling: assertion failed"); }
        }
    }
    new_id = mpLineGetPrevInline(line_id);
    'l235: while new_id != (-1_i32) {
        'c235: {
            if !(((*groundCollLine.offset(new_id as isize)).flags & ((CollLine_Ceiling as i32) as u32)) != 0) {
                new_id = (-1_i32);
            } else if new_id != ((*(*groundCollLine.offset(line_id as isize)).x0).prev_id1 as i32) {
                line_id = new_id;
                line = (*groundCollLine.offset(new_id as isize)).x0;
                new_id = mpLineGetPrevCheckResultFirst((*line).prev_id1 as i32, line);
                break 'c235;
            }
            break 'l235;
        }
    }
    if new_id != (-1_i32) {
        result = new_id;
    } else {
        result = (-1_i32);
    }
    return result;
}

/// `mpLib_80053BD4_Ceiling`
pub unsafe fn mpLib_80053BD4_Ceiling(mut line_id: i32) -> i32 {
    let mut result: i32 = 0;
    let mut new_id: i32 = 0;
    {
        if (line_id == (-1_i32)) || (line_id >= (*mpLib_804D64B4).line_count) {
            if 0 != 0 { (); } else { ext::OSReport(&format!("{}:{}:not found lineID={}\n", core::ffi::CStr::from_ptr((c"src/melee/mp/mplib.c".as_ptr() as *mut i8) as *const core::ffi::c_char).to_string_lossy(), 4355, line_id)); panic!("mpLib_80053BD4_Ceiling: assertion failed"); }
        }
    }
    new_id = mpLineGetNextInline(line_id);
    'l236: while new_id != (-1_i32) {
        'c236: {
            if !(((*groundCollLine.offset(new_id as isize)).flags & ((CollLine_Ceiling as i32) as u32)) != 0) {
                new_id = (-1_i32);
            } else if new_id != ((*(*groundCollLine.offset(line_id as isize)).x0).next_id1 as i32) {
                line_id = new_id;
                new_id = mpLineGetNextInline(new_id);
                break 'c236;
            }
            break 'l236;
        }
    }
    if new_id != (-1_i32) {
        result = new_id;
    } else {
        result = (-1_i32);
    }
    return result;
}

/// `mpLib_80053DA4_Floor`: the end (right vertex) of the floor run `line_id` belongs to
pub unsafe fn mpLib_80053DA4_Floor(mut line_id: i32, mut pos_out: *mut Vec3) {
    lineid_check(4428, line_id);

    // `goto skip; loop: line_id = next_id; skip: next_id = ...; if (...) goto loop;`
    loop {
        let next_id: i16 = (*(*groundCollLine.offset(line_id as isize)).x0).next_id0;
        if next_id != -1 && ((*groundCollLine.offset(next_id as isize)).flags & (CollLine_Floor as u32)) != 0 {
            line_id = next_id as i32;
            continue;
        }
        break;
    }

    lineid_check(4433, line_id);

    let vtx: *mut CollVtx = groundCollVtx.offset((*(*groundCollLine.offset(line_id as isize)).x0).v1_idx as isize);
    (*pos_out).x = (*vtx).pos.x;
    (*pos_out).y = (*vtx).pos.y;
    (*pos_out).z = 0.0f32;
}

/// `mpLib_80053ECC_Floor`: the start (left vertex) of the floor run `line_id` belongs to
pub unsafe fn mpLib_80053ECC_Floor(mut line_id: i32, mut vec: *mut Vec) {
    lineid_check(4448, line_id);

    // `goto skip; loop: line_id = prev_id; skip: prev_id = ...; if (...) goto loop;`
    loop {
        let prev_id: i16 = (*(*groundCollLine.offset(line_id as isize)).x0).prev_id0;
        if prev_id != -1 && ((*groundCollLine.offset(prev_id as isize)).flags & (CollLine_Floor as u32)) != 0 {
            line_id = prev_id as i32;
            continue;
        }
        break;
    }

    lineid_check(4453, line_id);

    let vtx: *mut CollVtx = groundCollVtx.offset((*(*groundCollLine.offset(line_id as isize)).x0).v0_idx as isize);
    (*vec).x = (*vtx).pos.x;
    (*vec).y = (*vtx).pos.y;
    (*vec).z = 0.0f32;
}

/// `mpLineGetKindInline`
unsafe fn mpLineGetKindInline(mut line_id: i32) -> u32 {
    return (*groundCollLine.offset(line_id as isize)).flags & (15 as u32);
}

/// `mpFloorGetRight`
pub unsafe fn mpFloorGetRight(mut line_id: i32, mut pos_out: *mut Vec) {
    let mut kind: u32 = 0;
    #[derive(Clone, Copy)] #[repr(C)] struct Local11 { id: i32 }
    let mut w: Local11 = unsafe { core::mem::zeroed() };
    let mut line_offset: i32 = 0;
    {
        if (line_id == (-1_i32)) || (line_id >= (*mpLib_804D64B4).line_count) {
            if 0 != 0 { (); } else { ext::OSReport(&format!("{}:{}:not found lineID={}\n", core::ffi::CStr::from_ptr((c"src/melee/mp/mplib.c".as_ptr() as *mut i8) as *const core::ffi::c_char).to_string_lossy(), 4465, line_id)); panic!("mpFloorGetRight: assertion failed"); }
        }
    }
    w.id = line_id;
    kind = mpLineGetKindInline(line_id);
    'w237: loop {
        {
            let mut line: *mut MapLine = null_mut();
            let mut next: i32 = 0;
            line = (*groundCollLine.offset((w.id as u32) as isize)).x0;
            line_offset = ((w.id as u64) * 16_u64) as i32;
            next = (*line).next_id1 as i32;
            w.id = mpLineGetNextCheckInline(line, next as i16);
        }
        if w.id == (-1_i32) {
            break 'w237;
        }
        if kind == ((*groundCollLine.offset(w.id as isize)).flags & (15 as u32)) {
            ();
            continue 'w237;
        }
        break;
    }
    {
        let mut vtx: *mut CollVtx = core::ptr::addr_of_mut!(*groundCollVtx.offset((*(*((groundCollLine as *mut u8).offset(line_offset as isize) as *mut CollLine)).x0).v1_idx as isize)) as *mut CollVtx;
        (*pos_out).x = (*vtx).pos.x;
        (*pos_out).y = (*vtx).pos.y;
        (*pos_out).z = 0.0f32;
    }
}

/// `mpFloorGetLeft`
pub unsafe fn mpFloorGetLeft(mut line_id: i32, mut pos_out: *mut Vec) {
    let mut kind: u32 = 0;
    #[derive(Clone, Copy)] #[repr(C)] struct Local11 { id: i32 }
    let mut w: Local11 = unsafe { core::mem::zeroed() };
    let mut line_offset: i32 = 0;
    {
        if (line_id == (-1_i32)) || (line_id >= (*mpLib_804D64B4).line_count) {
            if 0 != 0 { (); } else { ext::OSReport(&format!("{}:{}:not found lineID={}\n", core::ffi::CStr::from_ptr((c"src/melee/mp/mplib.c".as_ptr() as *mut i8) as *const core::ffi::c_char).to_string_lossy(), 4474, line_id)); panic!("mpFloorGetLeft: assertion failed"); }
        }
    }
    w.id = line_id;
    kind = mpLineGetKindInline(line_id);
    'w238: loop {
        {
            let mut line: *mut MapLine = null_mut();
            let mut next: i32 = 0;
            line = (*groundCollLine.offset((w.id as u32) as isize)).x0;
            line_offset = ((w.id as u64) * 16_u64) as i32;
            next = (*line).prev_id1 as i32;
            w.id = mpLineGetPrevCheckInline(line, next as i16);
        }
        if w.id == (-1_i32) {
            break 'w238;
        }
        if kind == ((*groundCollLine.offset(w.id as isize)).flags & (15 as u32)) {
            ();
            continue 'w238;
        }
        break;
    }
    {
        let mut vtx: *mut CollVtx = core::ptr::addr_of_mut!(*groundCollVtx.offset((*(*((groundCollLine as *mut u8).offset(line_offset as isize) as *mut CollLine)).x0).v0_idx as isize)) as *mut CollVtx;
        (*pos_out).x = (*vtx).pos.x;
        (*pos_out).y = (*vtx).pos.y;
        (*pos_out).z = 0.0f32;
    }
}

/// `mpCeilingGetRight`
pub unsafe fn mpCeilingGetRight(mut line_id: i32, mut pos_out: *mut Vec) {
    let mut kind: u32 = 0;
    #[derive(Clone, Copy)] #[repr(C)] struct Local11 { id: i32 }
    let mut w: Local11 = unsafe { core::mem::zeroed() };
    let mut line_offset: i32 = 0;
    {
        if (line_id == (-1_i32)) || (line_id >= (*mpLib_804D64B4).line_count) {
            if 0 != 0 { (); } else { ext::OSReport(&format!("{}:{}:not found lineID={}\n", core::ffi::CStr::from_ptr((c"src/melee/mp/mplib.c".as_ptr() as *mut i8) as *const core::ffi::c_char).to_string_lossy(), 4483, line_id)); panic!("mpCeilingGetRight: assertion failed"); }
        }
    }
    if line_id != (-1_i32) {
    }
    kind = mpLineGetKindInline(line_id);
    w.id = line_id;
    'w239: loop {
        {
            let mut line: *mut MapLine = null_mut();
            let mut next: i32 = 0;
            line = (*groundCollLine.offset((w.id as u32) as isize)).x0;
            line_offset = ((w.id as u64) * 16_u64) as i32;
            next = (*line).prev_id1 as i32;
            w.id = mpLineGetPrevCheckInline(line, next as i16);
        }
        if w.id == (-1_i32) {
            break 'w239;
        }
        if kind == ((*groundCollLine.offset(w.id as isize)).flags & (15 as u32)) {
            ();
            continue 'w239;
        }
        break;
    }
    {
        let mut vtx: *mut CollVtx = core::ptr::addr_of_mut!(*groundCollVtx.offset((*(*((groundCollLine as *mut u8).offset(line_offset as isize) as *mut CollLine)).x0).v0_idx as isize)) as *mut CollVtx;
        (*pos_out).x = (*vtx).pos.x;
        (*pos_out).y = (*vtx).pos.y;
        (*pos_out).z = 0.0f32;
    }
}

/// `mpCeilingGetLeft`
pub unsafe fn mpCeilingGetLeft(mut line_id: i32, mut pos_out: *mut Vec) {
    let mut kind: u32 = 0;
    #[derive(Clone, Copy)] #[repr(C)] struct Local11 { id: i32 }
    let mut w: Local11 = unsafe { core::mem::zeroed() };
    let mut line_offset: i32 = 0;
    {
        if (line_id == (-1_i32)) || (line_id >= (*mpLib_804D64B4).line_count) {
            if 0 != 0 { (); } else { ext::OSReport(&format!("{}:{}:not found lineID={}\n", core::ffi::CStr::from_ptr((c"src/melee/mp/mplib.c".as_ptr() as *mut i8) as *const core::ffi::c_char).to_string_lossy(), 4492, line_id)); panic!("mpCeilingGetLeft: assertion failed"); }
        }
    }
    if line_id != (-1_i32) {
    }
    w.id = line_id;
    kind = mpLineGetKindInline(line_id);
    'w240: loop {
        {
            let mut line: *mut MapLine = null_mut();
            let mut next: i32 = 0;
            line = (*groundCollLine.offset((w.id as u32) as isize)).x0;
            line_offset = ((w.id as u64) * 16_u64) as i32;
            next = (*line).next_id1 as i32;
            w.id = mpLineGetNextCheckInline(line, next as i16);
        }
        if w.id == (-1_i32) {
            break 'w240;
        }
        if kind == ((*groundCollLine.offset(w.id as isize)).flags & (15 as u32)) {
            ();
            continue 'w240;
        }
        break;
    }
    {
        let mut vtx: *mut CollVtx = core::ptr::addr_of_mut!(*groundCollVtx.offset((*(*((groundCollLine as *mut u8).offset(line_offset as isize) as *mut CollLine)).x0).v1_idx as isize)) as *mut CollVtx;
        (*pos_out).x = (*vtx).pos.x;
        (*pos_out).y = (*vtx).pos.y;
        (*pos_out).z = 0.0f32;
    }
}

/// `mpLeftWallGetTop`
pub unsafe fn mpLeftWallGetTop(mut line_id: i32, mut pos_out: *mut Vec) {
    let mut kind: u32 = 0;
    #[derive(Clone, Copy)] #[repr(C)] struct Local11 { id: i32 }
    let mut w: Local11 = unsafe { core::mem::zeroed() };
    let mut line_offset: i32 = 0;
    {
        if (line_id == (-1_i32)) || (line_id >= (*mpLib_804D64B4).line_count) {
            if 0 != 0 { (); } else { ext::OSReport(&format!("{}:{}:not found lineID={}\n", core::ffi::CStr::from_ptr((c"src/melee/mp/mplib.c".as_ptr() as *mut i8) as *const core::ffi::c_char).to_string_lossy(), 4501, line_id)); panic!("mpLeftWallGetTop: assertion failed"); }
        }
    }
    if line_id != (-1_i32) {
    }
    w.id = line_id;
    kind = mpLineGetKindInline(line_id);
    'w241: loop {
        {
            let mut line: *mut MapLine = null_mut();
            let mut next: i32 = 0;
            line = (*groundCollLine.offset((w.id as u32) as isize)).x0;
            line_offset = ((w.id as u64) * 16_u64) as i32;
            next = (*line).next_id1 as i32;
            w.id = mpLineGetNextCheckInline(line, next as i16);
        }
        if w.id == (-1_i32) {
            break 'w241;
        }
        if kind == ((*groundCollLine.offset(w.id as isize)).flags & (15 as u32)) {
            ();
            continue 'w241;
        }
        break;
    }
    {
        let mut vtx: *mut CollVtx = core::ptr::addr_of_mut!(*groundCollVtx.offset((*(*((groundCollLine as *mut u8).offset(line_offset as isize) as *mut CollLine)).x0).v1_idx as isize)) as *mut CollVtx;
        (*pos_out).x = (*vtx).pos.x;
        (*pos_out).y = (*vtx).pos.y;
        (*pos_out).z = 0.0f32;
    }
}

/// `mpLeftWallGetBottom`
pub unsafe fn mpLeftWallGetBottom(mut line_id: i32, mut pos_out: *mut Vec) {
    let mut kind: u32 = 0;
    #[derive(Clone, Copy)] #[repr(C)] struct Local11 { id: i32 }
    let mut w: Local11 = unsafe { core::mem::zeroed() };
    let mut line_offset: i32 = 0;
    {
        if (line_id == (-1_i32)) || (line_id >= (*mpLib_804D64B4).line_count) {
            if 0 != 0 { (); } else { ext::OSReport(&format!("{}:{}:not found lineID={}\n", core::ffi::CStr::from_ptr((c"src/melee/mp/mplib.c".as_ptr() as *mut i8) as *const core::ffi::c_char).to_string_lossy(), 4510, line_id)); panic!("mpLeftWallGetBottom: assertion failed"); }
        }
    }
    if line_id != (-1_i32) {
    }
    kind = mpLineGetKindInline(line_id);
    w.id = line_id;
    'w242: loop {
        {
            let mut line: *mut MapLine = null_mut();
            let mut next: i32 = 0;
            line = (*groundCollLine.offset((w.id as u32) as isize)).x0;
            line_offset = ((w.id as u64) * 16_u64) as i32;
            next = (*line).prev_id1 as i32;
            w.id = mpLineGetPrevCheckInline(line, next as i16);
        }
        if w.id == (-1_i32) {
            break 'w242;
        }
        if kind == ((*groundCollLine.offset(w.id as isize)).flags & (15 as u32)) {
            ();
            continue 'w242;
        }
        break;
    }
    {
        let mut vtx: *mut CollVtx = core::ptr::addr_of_mut!(*groundCollVtx.offset((*(*((groundCollLine as *mut u8).offset(line_offset as isize) as *mut CollLine)).x0).v0_idx as isize)) as *mut CollVtx;
        (*pos_out).x = (*vtx).pos.x;
        (*pos_out).y = (*vtx).pos.y;
        (*pos_out).z = 0.0f32;
    }
}

/// `mpRightWallGetTop`
pub unsafe fn mpRightWallGetTop(mut line_id: i32, mut pos_out: *mut Vec) {
    let mut kind: u32 = 0;
    #[derive(Clone, Copy)] #[repr(C)] struct Local11 { id: i32 }
    let mut w: Local11 = unsafe { core::mem::zeroed() };
    let mut line_offset: i32 = 0;
    {
        if (line_id == (-1_i32)) || (line_id >= (*mpLib_804D64B4).line_count) {
            if 0 != 0 { (); } else { ext::OSReport(&format!("{}:{}:not found lineID={}\n", core::ffi::CStr::from_ptr((c"src/melee/mp/mplib.c".as_ptr() as *mut i8) as *const core::ffi::c_char).to_string_lossy(), 4519, line_id)); panic!("mpRightWallGetTop: assertion failed"); }
        }
    }
    if line_id != (-1_i32) {
    }
    w.id = line_id;
    kind = mpLineGetKindInline(line_id);
    'w243: loop {
        {
            let mut line: *mut MapLine = null_mut();
            let mut next: i32 = 0;
            line = (*groundCollLine.offset((w.id as u32) as isize)).x0;
            line_offset = ((w.id as u64) * 16_u64) as i32;
            next = (*line).prev_id1 as i32;
            w.id = mpLineGetPrevCheckInline(line, next as i16);
        }
        if w.id == (-1_i32) {
            break 'w243;
        }
        if kind == ((*groundCollLine.offset(w.id as isize)).flags & (15 as u32)) {
            ();
            continue 'w243;
        }
        break;
    }
    {
        let mut vtx: *mut CollVtx = core::ptr::addr_of_mut!(*groundCollVtx.offset((*(*((groundCollLine as *mut u8).offset(line_offset as isize) as *mut CollLine)).x0).v0_idx as isize)) as *mut CollVtx;
        (*pos_out).x = (*vtx).pos.x;
        (*pos_out).y = (*vtx).pos.y;
        (*pos_out).z = 0.0f32;
    }
}

/// `mpRightWallGetBottom`
pub unsafe fn mpRightWallGetBottom(mut line_id: i32, mut pos_out: *mut Vec) {
    let mut line_offset: i32 = 0;
    #[derive(Clone, Copy)] #[repr(C)] struct Local11 { id: i32 }
    let mut w: Local11 = unsafe { core::mem::zeroed() };
    let mut kind: u32 = 0;
    {
        if (line_id == (-1_i32)) || (line_id >= (*mpLib_804D64B4).line_count) {
            if 0 != 0 { (); } else { ext::OSReport(&format!("{}:{}:not found lineID={}\n", core::ffi::CStr::from_ptr((c"src/melee/mp/mplib.c".as_ptr() as *mut i8) as *const core::ffi::c_char).to_string_lossy(), 4528, line_id)); panic!("mpRightWallGetBottom: assertion failed"); }
        }
    }
    if line_id != (-1_i32) {
    }
    w.id = line_id;
    kind = mpLineGetKindInline(line_id);
    'w244: loop {
        {
            let mut line: *mut MapLine = null_mut();
            let mut next: i32 = 0;
            line = (*groundCollLine.offset((w.id as u32) as isize)).x0;
            line_offset = ((w.id as u64) * 16_u64) as i32;
            next = (*line).next_id1 as i32;
            w.id = mpLineGetNextCheckInline(line, next as i16);
        }
        if w.id == (-1_i32) {
            break 'w244;
        }
        if kind == ((*groundCollLine.offset(w.id as isize)).flags & (15 as u32)) {
            ();
            continue 'w244;
        }
        break;
    }
    {
        let mut vtx: *mut CollVtx = core::ptr::addr_of_mut!(*groundCollVtx.offset((*(*((groundCollLine as *mut u8).offset(line_offset as isize) as *mut CollLine)).x0).v1_idx as isize)) as *mut CollVtx;
        (*pos_out).x = (*vtx).pos.x;
        (*pos_out).y = (*vtx).pos.y;
        (*pos_out).z = 0.0f32;
    }
}

/// `mpLineGetV1Pos`
pub unsafe fn mpLineGetV1Pos(mut line_id: i32, mut pos_out: *mut Vec) {
    let mut v1: *mut CollVtx = null_mut();
    {
        if (line_id == (-1_i32)) || (line_id >= (*mpLib_804D64B4).line_count) {
            if 0 != 0 { (); } else { ext::OSReport(&format!("{}:{}:not found lineID={}\n", core::ffi::CStr::from_ptr((c"src/melee/mp/mplib.c".as_ptr() as *mut i8) as *const core::ffi::c_char).to_string_lossy(), 4540, line_id)); panic!("mpLineGetV1Pos: assertion failed"); }
        }
    }
    v1 = core::ptr::addr_of_mut!(*groundCollVtx.offset((*(*groundCollLine.offset(line_id as isize)).x0).v1_idx as isize)) as *mut CollVtx;
    (*pos_out).x = (*v1).pos.x;
    (*pos_out).y = (*v1).pos.y;
    (*pos_out).z = 0.0f32;
}

/// `mpLineGetV0Pos`
pub unsafe fn mpLineGetV0Pos(mut line_id: i32, mut pos_out: *mut Vec) {
    let mut v0: *mut CollVtx = null_mut();
    {
        if (line_id == (-1_i32)) || (line_id >= (*mpLib_804D64B4).line_count) {
            if 0 != 0 { (); } else { ext::OSReport(&format!("{}:{}:not found lineID={}\n", core::ffi::CStr::from_ptr((c"src/melee/mp/mplib.c".as_ptr() as *mut i8) as *const core::ffi::c_char).to_string_lossy(), 4555, line_id)); panic!("mpLineGetV0Pos: assertion failed"); }
        }
    }
    v0 = core::ptr::addr_of_mut!(*groundCollVtx.offset((*(*groundCollLine.offset(line_id as isize)).x0).v0_idx as isize)) as *mut CollVtx;
    (*pos_out).x = (*v0).pos.x;
    (*pos_out).y = (*v0).pos.y;
    (*pos_out).z = 0.0f32;
}

/// `mpLineGetKind`
pub unsafe fn mpLineGetKind(mut line_id: i32) -> i32 {
    {
        if (line_id == (-1_i32)) || (line_id >= (*mpLib_804D64B4).line_count) {
            if 0 != 0 { (); } else { ext::OSReport(&format!("{}:{}:not found lineID={}\n", core::ffi::CStr::from_ptr((c"src/melee/mp/mplib.c".as_ptr() as *mut i8) as *const core::ffi::c_char).to_string_lossy(), 4573, line_id)); panic!("mpLineGetKind: assertion failed"); }
        }
    }
    return ((*groundCollLine.offset(line_id as isize)).flags & (15 as u32)) as i32;
}

/// `mpLineGetFlags`
pub unsafe fn mpLineGetFlags(mut line_id: i32) -> u32 {
    {
        if (line_id == (-1_i32)) || (line_id >= (*mpLib_804D64B4).line_count) {
            if 0 != 0 { (); } else { ext::OSReport(&format!("{}:{}:not found lineID={}\n", core::ffi::CStr::from_ptr((c"src/melee/mp/mplib.c".as_ptr() as *mut i8) as *const core::ffi::c_char).to_string_lossy(), 4583, line_id)); panic!("mpLineGetFlags: assertion failed"); }
        }
    }
    return (*(*groundCollLine.offset(line_id as isize)).x0).lo_flags as u32;
}

/// `mpLib_80054D68`
pub unsafe fn mpLib_80054D68(mut line_id: i32, mut flags: u32) {
    {
        if (line_id == (-1_i32)) || (line_id >= (*mpLib_804D64B4).line_count) {
            if 0 != 0 { (); } else { ext::OSReport(&format!("{}:{}:not found lineID={}\n", core::ffi::CStr::from_ptr((c"src/melee/mp/mplib.c".as_ptr() as *mut i8) as *const core::ffi::c_char).to_string_lossy(), 4595, line_id)); panic!("mpLib_80054D68: assertion failed"); }
        }
    }
    {
        let mut line: *mut MapLine = (*groundCollLine.offset(line_id as isize)).x0;
        let mut old_flags: *mut u16 = core::ptr::addr_of_mut!((*line).lo_flags) as *mut u16;
        *old_flags = (((((*old_flags) as i32) & (!255)) as u32) | flags) as u16;
    }
}

/// `mpLineGetNormal`
pub unsafe fn mpLineGetNormal(mut line_id: i32, mut normal_out: *mut Vec) -> *mut Vec {
    let mut line: *mut MapLine = null_mut();
    {
        if (line_id == (-1_i32)) || (line_id >= (*mpLib_804D64B4).line_count) {
            if 0 != 0 { (); } else { ext::OSReport(&format!("{}:{}:not found lineID={}\n", core::ffi::CStr::from_ptr((c"src/melee/mp/mplib.c".as_ptr() as *mut i8) as *const core::ffi::c_char).to_string_lossy(), 4609, line_id)); panic!("mpLineGetNormal: assertion failed"); }
        }
    }
    line = (*groundCollLine.offset(line_id as isize)).x0;
    {
        let mut y0: f32 = (*groundCollVtx.offset((*line).v0_idx as isize)).pos.y;
        let mut y1: f32 = (*groundCollVtx.offset((*line).v1_idx as isize)).pos.y;
        let mut x0: f32 = (*groundCollVtx.offset((*line).v0_idx as isize)).pos.x;
        let mut x1: f32 = (*groundCollVtx.offset((*line).v1_idx as isize)).pos.x;
        (*normal_out).x = -(y1 - y0);
        (*normal_out).y = x1 - x0;
        (*normal_out).z = 0.0f32;
        { let __n = melee_runtime::mtx::vec_normalize(&melee_core::vec::Vec3 { x: (*normal_out).x, y: (*normal_out).y, z: (*normal_out).z }); (*normal_out).x = __n.x; (*normal_out).y = __n.y; (*normal_out).z = __n.z; };
    }
    return normal_out;
}

/// `mpLib_80054ED8`
pub unsafe fn mpLib_80054ED8(mut line_id: i32) -> i32 {
    if !(ext::ScriptGame_StageLineActive(line_id) != 0) {
        return 0;
    }
    if line_id == (-1_i32) {
        return 0;
    }
    if (line_id < 0) || (line_id >= (*mpLib_804D64B4).line_count) {
        ext::OSReport(&format!("{}:{}:not found lineID={}\n", core::ffi::CStr::from_ptr((c"src/melee/mp/mplib.c".as_ptr() as *mut i8) as *const core::ffi::c_char).to_string_lossy(), 4636, line_id));
        'l245: while 1 != 0 {
        }
    }
    if (!(((*groundCollLine.offset(line_id as isize)).flags & ((1 << 16) as u32)) != 0)) || (((*groundCollLine.offset(line_id as isize)).flags & ((1 << 18) as u32)) != 0) {
        return 0;
    }
    return 1;
}

/// `mpLineGetNextFrom`
unsafe fn mpLineGetNextFrom(mut line: *mut MapLine, mut flags_base: *const u32) -> i32 {
    let mut result: i32 = (*line).next_id1 as i32;
    if result != (-1_i32) {
        let mut flags: u32 = *flags_base.offset((result * 2) as isize);
        if (((flags & ((1 << 16) as u32)) != 0) && (!((flags & ((1 << 18) as u32)) != 0))) && (ext::ScriptGame_StageLineActive(result) != 0) {
            let mut v1: *mut CollVtx = core::ptr::addr_of_mut!(*groundCollVtx.offset((*line).v1_idx as isize)) as *mut CollVtx;
            let mut v0: *mut CollVtx = core::ptr::addr_of_mut!(*groundCollVtx.offset((*(*groundCollLine.offset(result as isize)).x0).v0_idx as isize)) as *mut CollVtx;
            if (fmadds((*v1).pos.x - (*v0).pos.x, (*v1).pos.x - (*v0).pos.x, ((*v1).pos.y - (*v0).pos.y) * ((*v1).pos.y - (*v0).pos.y)) as f64) < 4.0f64 {
                return result;
            }
        }
    }
    return if (((*line).next_id0 as i32) != (-1_i32)) && (!(ext::ScriptGame_StageLineActive((*line).next_id0 as i32) != 0)) { (-1_i32) } else { (*line).next_id0 as i32 };
}

/// `mpLineGetPrevFrom`
unsafe fn mpLineGetPrevFrom(mut line: *mut MapLine, mut flags_base: *const u32) -> i32 {
    let mut result: i32 = (*line).prev_id1 as i32;
    if result != (-1_i32) {
        let mut flags: u32 = *flags_base.offset((result * 2) as isize);
        if (((flags & ((1 << 16) as u32)) != 0) && (!((flags & ((1 << 18) as u32)) != 0))) && (ext::ScriptGame_StageLineActive(result) != 0) {
            let mut v0: *mut CollVtx = core::ptr::addr_of_mut!(*groundCollVtx.offset((*line).v0_idx as isize)) as *mut CollVtx;
            let mut v1: *mut CollVtx = core::ptr::addr_of_mut!(*groundCollVtx.offset((*(*groundCollLine.offset(result as isize)).x0).v1_idx as isize)) as *mut CollVtx;
            if (fmadds((*v0).pos.x - (*v1).pos.x, (*v0).pos.x - (*v1).pos.x, ((*v0).pos.y - (*v1).pos.y) * ((*v0).pos.y - (*v1).pos.y)) as f64) < 4.0f64 {
                return result;
            }
        }
    }
    return if (((*line).prev_id0 as i32) != (-1_i32)) && (!(ext::ScriptGame_StageLineActive((*line).prev_id0 as i32) != 0)) { (-1_i32) } else { (*line).prev_id0 as i32 };
}

/// `mpLinesConnected`
pub unsafe fn mpLinesConnected(mut start_id: i32, mut target_id: i32) -> i32 {
    let mut line_id: i32 = 0;
    let mut kind: u32 = 0;
    let mut start_line: *mut MapLine = null_mut();
    let mut flags_base: *mut u32 = null_mut();
    {
        if (start_id == (-1_i32)) || (start_id >= (*mpLib_804D64B4).line_count) {
            if 0 != 0 { (); } else { ext::OSReport(&format!("{}:{}:not found lineID={}\n", core::ffi::CStr::from_ptr((c"src/melee/mp/mplib.c".as_ptr() as *mut i8) as *const core::ffi::c_char).to_string_lossy(), 4656, start_id)); panic!("mpLinesConnected: assertion failed"); }
        }
    }
    {
        if (target_id == (-1_i32)) || (target_id >= (*mpLib_804D64B4).line_count) {
            if 0 != 0 { (); } else { ext::OSReport(&format!("{}:{}:not found lineID={}\n", core::ffi::CStr::from_ptr((c"src/melee/mp/mplib.c".as_ptr() as *mut i8) as *const core::ffi::c_char).to_string_lossy(), 4657, target_id)); panic!("mpLinesConnected: assertion failed"); }
        }
    }
    if start_id == target_id {
        return 1;
    }
    start_line = (*groundCollLine.offset(start_id as isize)).x0;
    flags_base = core::ptr::addr_of_mut!((*groundCollLine).flags) as *mut u32;
    kind = (*flags_base.offset((start_id * 2) as isize)) & (15 as u32);
    line_id = mpLineGetNextFrom(start_line, flags_base as *const u32);
    'l246: while (line_id != (-1_i32)) && (kind == ((*groundCollLine.offset(line_id as isize)).flags & (15 as u32))) {
        if line_id == target_id {
            return 1;
        }
        line_id = mpLineGetNext(line_id);
    }
    line_id = mpLineGetPrevFrom(start_line, flags_base as *const u32);
    'l247: while (line_id != (-1_i32)) && (kind == ((*groundCollLine.offset(line_id as isize)).flags & (15 as u32))) {
        if line_id == target_id {
            return 1;
        }
        line_id = mpLineGetPrev(line_id);
    }
    return 0;
}

/// `mpLib_800552B0`
pub unsafe fn mpLib_800552B0(mut joint_id: i32, mut jobj: *mut HSD_JObj, mut z: i32) {
    let mut i: i32 = 0;
    let mut r7: *mut HSD_JObj = null_mut();
    {
        r7 = HSD_JObjGetChild(jobj); i = 0;
        'l248: while (!r7.is_null()) && (i != z) {
            'c248: {
                if !(((*r7).flags & ((CollJoint_TooFar as i32) as u32)) != 0) {
                    if !HSD_JObjGetChild(r7).is_null() {
                        r7 = HSD_JObjGetChild(r7);
                        break 'c248;
                    }
                }
                if !HSD_JObjGetNext(r7).is_null() {
                    r7 = HSD_JObjGetNext(r7);
                    break 'c248;
                }
                'l249: while 1 != 0 {
                    'c249: {
                        if HSD_JObjGetParent(r7).is_null() {
                            r7 = null_mut();
                        } else {
                            if !HSD_JObjGetNext(HSD_JObjGetParent(r7)).is_null() {
                                r7 = HSD_JObjGetNext(HSD_JObjGetParent(r7));
                            } else {
                                r7 = HSD_JObjGetParent(r7);
                                break 'c249;
                            }
                        }
                        break 'l249;
                    }
                }
            }
            i = i + 1;
        }
    }
    if !r7.is_null() {
        let mut joint: *mut CollJoint = core::ptr::addr_of_mut!(*groundCollJoint.offset(joint_id as isize)) as *mut CollJoint;
        (*joint).x20 = r7;
    }
}

/// `mpJointHide`
pub unsafe fn mpJointHide(mut joint_id: i32) {
    let mut joint: *mut CollJoint = null_mut();
    let mut line: *mut CollLine = null_mut();
    let mut count: i32 = 0;
    joint = core::ptr::addr_of_mut!(*groundCollJoint.offset(joint_id as isize)) as *mut CollJoint;
    (*joint).flags = (*joint).flags | ((CollJoint_Hidden as i32) as u32);
    count = (*(*joint).inner).ranges[(MapLineGroup_Floor as i32) as usize].count as i32;
    line = core::ptr::addr_of_mut!(*groundCollLine.offset((*(*joint).inner).ranges[(MapLineGroup_Floor as i32) as usize].start as isize)) as *mut CollLine;
    'l250: while { let __t251 = count; count = count - 1; __t251 } > 0 {
        (*line).flags = (*line).flags | ((1 << 18) as u32);
        line = line.offset(1);
    }
    count = (*(*joint).inner).ranges[(MapLineGroup_Ceiling as i32) as usize].count as i32;
    line = core::ptr::addr_of_mut!(*groundCollLine.offset((*(*joint).inner).ranges[(MapLineGroup_Ceiling as i32) as usize].start as isize)) as *mut CollLine;
    'l252: while { let __t253 = count; count = count - 1; __t253 } > 0 {
        (*line).flags = (*line).flags | ((1 << 18) as u32);
        line = line.offset(1);
    }
    count = (*(*joint).inner).ranges[(MapLineGroup_LeftWall as i32) as usize].count as i32;
    line = core::ptr::addr_of_mut!(*groundCollLine.offset((*(*joint).inner).ranges[(MapLineGroup_LeftWall as i32) as usize].start as isize)) as *mut CollLine;
    'l254: while { let __t255 = count; count = count - 1; __t255 } > 0 {
        (*line).flags = (*line).flags | ((1 << 18) as u32);
        line = line.offset(1);
    }
    count = (*(*joint).inner).ranges[(MapLineGroup_RightWall as i32) as usize].count as i32;
    line = core::ptr::addr_of_mut!(*groundCollLine.offset((*(*joint).inner).ranges[(MapLineGroup_RightWall as i32) as usize].start as isize)) as *mut CollLine;
    'l256: while { let __t257 = count; count = count - 1; __t257 } > 0 {
        (*line).flags = (*line).flags | ((1 << 18) as u32);
        line = line.offset(1);
    }
    count = (*(*joint).inner).ranges[(MapLineGroup_Dynamic as i32) as usize].count as i32;
    line = core::ptr::addr_of_mut!(*groundCollLine.offset((*(*joint).inner).ranges[(MapLineGroup_Dynamic as i32) as usize].start as isize)) as *mut CollLine;
    'l258: while { let __t259 = count; count = count - 1; __t259 } > 0 {
        (*line).flags = (*line).flags | ((1 << 18) as u32);
        line = line.offset(1);
    }
}

/// `mpJointUnhide`
pub unsafe fn mpJointUnhide(mut joint_id: i32) {
    let mut joint: *mut CollJoint = null_mut();
    let mut line: *mut CollLine = null_mut();
    let mut vtx: *mut CollVtx = null_mut();
    let mut count: i32 = 0;
    let mut i: i32 = 0;
    joint = core::ptr::addr_of_mut!(*groundCollJoint.offset(joint_id as isize)) as *mut CollJoint;
    (*joint).flags = (*joint).flags & ((!(CollJoint_Hidden as i32)) as u32);
    count = (*(*joint).inner).ranges[(MapLineGroup_Floor as i32) as usize].count as i32;
    line = core::ptr::addr_of_mut!(*groundCollLine.offset((*(*joint).inner).ranges[(MapLineGroup_Floor as i32) as usize].start as isize)) as *mut CollLine;
    'l260: while { let __t261 = count; count = count - 1; __t261 } > 0 {
        (*line).flags = (*line).flags & ((!(1 << 18)) as u32);
        line = line.offset(1);
    }
    count = (*(*joint).inner).ranges[(MapLineGroup_Ceiling as i32) as usize].count as i32;
    line = core::ptr::addr_of_mut!(*groundCollLine.offset((*(*joint).inner).ranges[(MapLineGroup_Ceiling as i32) as usize].start as isize)) as *mut CollLine;
    'l262: while { let __t263 = count; count = count - 1; __t263 } > 0 {
        (*line).flags = (*line).flags & ((!(1 << 18)) as u32);
        line = line.offset(1);
    }
    count = (*(*joint).inner).ranges[(MapLineGroup_LeftWall as i32) as usize].count as i32;
    line = core::ptr::addr_of_mut!(*groundCollLine.offset((*(*joint).inner).ranges[(MapLineGroup_LeftWall as i32) as usize].start as isize)) as *mut CollLine;
    'l264: while { let __t265 = count; count = count - 1; __t265 } > 0 {
        (*line).flags = (*line).flags & ((!(1 << 18)) as u32);
        line = line.offset(1);
    }
    count = (*(*joint).inner).ranges[(MapLineGroup_RightWall as i32) as usize].count as i32;
    line = core::ptr::addr_of_mut!(*groundCollLine.offset((*(*joint).inner).ranges[(MapLineGroup_RightWall as i32) as usize].start as isize)) as *mut CollLine;
    'l266: while { let __t267 = count; count = count - 1; __t267 } > 0 {
        (*line).flags = (*line).flags & ((!(1 << 18)) as u32);
        line = line.offset(1);
    }
    count = (*(*joint).inner).ranges[(MapLineGroup_Dynamic as i32) as usize].count as i32;
    line = core::ptr::addr_of_mut!(*groundCollLine.offset((*(*joint).inner).ranges[(MapLineGroup_Dynamic as i32) as usize].start as isize)) as *mut CollLine;
    'l268: while { let __t269 = count; count = count - 1; __t269 } > 0 {
        (*line).flags = (*line).flags & ((!(1 << 18)) as u32);
        line = line.offset(1);
    }
    vtx = core::ptr::addr_of_mut!(*groundCollVtx.offset((*(*joint).inner).vtx_start as isize)) as *mut CollVtx;
    {
        i = (*(*joint).inner).vtx_count as i32;
        'l270: while i > 0 {
            (*vtx).x10 = (*vtx).pos.x;
            (*vtx).x14 = (*vtx).pos.y;
            i = i - 1; vtx = vtx.offset(1);
        }
    }
}

/// `mpJointUpdateDynamics`
pub unsafe fn mpJointUpdateDynamics(mut joint_id: i32) {
    let mut TAN30: f64 = 0.577350295784245f64;
    let mut TAN60: f64 = 1.7320508368950045f64;
    let mut joint: *mut CollJoint = core::ptr::addr_of_mut!(*groundCollJoint.offset(joint_id as isize)) as *mut CollJoint;
    let mut line: *mut CollLine = null_mut();
    let mut i: i32 = 0;
    let mut count: i16 = (*(*joint).inner).ranges[(MapLineGroup_Dynamic as i32) as usize].count;
    let mut kind: u32 = 0;
    line = core::ptr::addr_of_mut!(*groundCollLine.offset((*(*joint).inner).ranges[(MapLineGroup_Dynamic as i32) as usize].start as isize)) as *mut CollLine;
    {
        i = 0;
        'l271: while i < (count as i32) {
            let mut temp: *mut MapLine = (*line).x0;
            let mut v1: *mut CollVtx = core::ptr::addr_of_mut!(*groundCollVtx.offset((*temp).v1_idx as isize)) as *mut CollVtx;
            let mut v0: *mut CollVtx = core::ptr::addr_of_mut!(*groundCollVtx.offset((*temp).v0_idx as isize)) as *mut CollVtx;
            let mut dx: f32 = (*v1).pos.x - (*v0).pos.x;
            let mut dy: f32 = (*v1).pos.y - (*v0).pos.y;
            if dx > 0.0f32 {
                if ((dy / dx) as f64) > TAN60 {
                    kind = (CollLine_LeftWall as i32) as u32;
                } else if ((dy / dx) as f64) < (-TAN60) {
                    kind = (CollLine_RightWall as i32) as u32;
                } else {
                    kind = (CollLine_Floor as i32) as u32;
                }
            } else if dx < 0.0f32 {
                if ((dy / dx) as f64) > TAN30 {
                    kind = (CollLine_RightWall as i32) as u32;
                } else if ((dy / dx) as f64) < (-TAN30) {
                    kind = (CollLine_LeftWall as i32) as u32;
                } else {
                    kind = (CollLine_Ceiling as i32) as u32;
                }
            } else if dy > 0.0f32 {
                kind = (CollLine_LeftWall as i32) as u32;
            } else if dy < 0.0f32 {
                kind = (CollLine_RightWall as i32) as u32;
            } else {
                if 0 != 0 { (); } else { panic!("mpJointUpdateDynamics: assertion failed"); }
            }
            (*line).flags = ((*line).flags & ((!15) as u32)) | kind;
            if (((*joint).flags & ((CollJoint_Enabled as i32) as u32)) != 0) && ((((*(*line).x0).lo_flags as i32) & 1024) != 0) {
                if (kind & ((CollLine_Floor as i32) as u32)) != 0 {
                    (*line).flags = (*line).flags | (((1 << 16) | (1 << 8)) as u32);
                    (*(*line).x0).lo_flags = (((*(*line).x0).lo_flags as i32) | (1 << 8)) as u16;
                } else {
                    (*line).flags = (*line).flags & ((!(1 << 16)) as u32);
                }
            }
            i = i + 1; line = line.offset(1);
        }
    }
}

/// `mpLib_80055E24`
pub unsafe fn mpLib_80055E24(mut joint_id: i32) {
    let mut var_r6: i32 = 0;
    let mut joint: *mut CollJoint = core::ptr::addr_of_mut!(*groundCollJoint.offset(joint_id as isize)) as *mut CollJoint;
    mpJointUpdateDynamics(joint_id);
    var_r6 = 0;
    if (!(((*joint).flags & (((CollJoint_Hidden as i32) | (CollJoint_B11 as i32)) as u32)) != 0)) && (((*joint).flags & ((CollJoint_Enabled as i32) as u32)) != 0) {
        var_r6 = 1;
    }
    mpScriptIslandRefresh(joint_id, (*(*joint).inner).vtx_start as i32, (*(*joint).inner).vtx_count as i32, var_r6);
}

/// `mpLib_80055E9C`
pub unsafe fn mpLib_80055E9C(mut joint_id: i32) {
    if ext::ScriptGame_StageSlotJointUpdate(joint_id) != 0 {
        return;
    }
    let mut f31: f32 = 0.0;
    let mut f30: f32 = 0.0;
    let mut f0: f32 = 0.0;
    let mut f1: f32 = 0.0;
    let mut f2: f32 = 0.0;
    let mut f3: f32 = 0.0;
    let mut __: [u8; 4] = [0; 4];
    let mut m0_3: f32 = 0.0;
    let mut m1_3: f32 = 0.0;
    let mut m0_0: f32 = 0.0;
    let mut joint: *mut CollJoint = null_mut();
    let mut jobj: *mut HSD_JObj = null_mut();
    let mut vtx_count: i32 = 0;
    let mut v_r26: *mut CollVtx = null_mut();
    let mut var_r25: i32 = 0;
    let mut v_r4: *mut CollVtx = null_mut();
    let mut var_r6: i32 = 0;
    let mut mtx: *mut [f32; 4] = null_mut();
    let mut i: i32 = 0;
    let mut sp28: Vec3 = unsafe { core::mem::zeroed() };
    super::mpcoll::mpColl_804D64AC = super::mpcoll::mpColl_804D64AC + 1;
    joint = core::ptr::addr_of_mut!(*groundCollJoint.offset(joint_id as isize)) as *mut CollJoint;
    vtx_count = (*(*joint).inner).vtx_count as i32;
    v_r4 = core::ptr::addr_of_mut!(*groundCollVtx.offset((*(*joint).inner).vtx_start as isize)) as *mut CollVtx;
    {
        i = 0;
        'l272: while i < vtx_count {
            (*v_r4).x10 = (*v_r4).pos.x;
            (*v_r4).x14 = (*v_r4).pos.y;
            i = i + 1; v_r4 = v_r4.offset(1);
        }
    }
    jobj = (*joint).x20;
    if jobj.is_null() {
        return;
    }
    if (ext::HSD_JObjGetFlags(jobj) & ((1 << 4) as u32)) != 0 {
        if !(((*joint).flags & ((CollJoint_Hidden as i32) as u32)) != 0) {
            mpJointHide(joint_id);
            var_r6 = 0;
            if (!(((*joint).flags & (((CollJoint_Hidden as i32) | (CollJoint_B11 as i32)) as u32)) != 0)) && (((*joint).flags & ((CollJoint_Enabled as i32) as u32)) != 0) {
                var_r6 = 1;
            }
            mpScriptIslandRefresh(joint_id, (*(*joint).inner).vtx_start as i32, (*(*joint).inner).vtx_count as i32, var_r6);
        }
        return;
    }
    (*joint).set_xE((1 as u8) as u8);
    var_r25 = 0;
    HSD_JObjSetupMatrix(jobj);
    mtx = HSD_JObjGetMtxPtr(jobj);
    m0_0 = *(mtx as *mut f32).offset(0 as isize);
    'b273: {
        'b274: {
            if (m0_0 == (*mtx.offset(1 as isize))[1 as usize]) && (m0_0 == (*mtx.offset(2 as isize))[2 as usize]) {
                m0_3 = (*mtx.offset(0 as isize))[3 as usize];
                m1_3 = (*mtx.offset(1 as isize))[3 as usize];
                v_r4 = core::ptr::addr_of_mut!(*groundCollVtx.offset((*(*joint).inner).vtx_start as isize)) as *mut CollVtx;
                {
                    i = 0;
                    'l275: while i < vtx_count {
                        (*v_r4).pos.x = fmadds((*v_r4).x0, m0_0, m0_3);
                        (*v_r4).pos.y = fmadds((*v_r4).x4, m0_0, m1_3);
                        i = i + 1; v_r4 = v_r4.offset(1);
                    }
                }
                (*joint).bounding_min.x = fmadds((*(*joint).inner).left_bound, m0_0, m0_3) - 3e+01f32;
                (*joint).bounding_min.y = fmadds((*(*joint).inner).bottom_bound, m0_0, m1_3) - 3e+01f32;
                (*joint).bounding_max.x = 3e+01f32 + fmadds((*(*joint).inner).right_bound, m0_0, m0_3);
                (*joint).bounding_max.y = 3e+01f32 + fmadds((*(*joint).inner).top_bound, m0_0, m1_3);
                (*joint).flags = (*joint).flags | ((CollJoint_B8 as i32) as u32);
                break 'b274;
            }
            i = 0;
            v_r26 = core::ptr::addr_of_mut!(*groundCollVtx.offset((*(*joint).inner).vtx_start as isize)) as *mut CollVtx;
            f31 = 0.0f32;
            'l276: while i < vtx_count {
                sp28.x = (*v_r26).x0;
                sp28.y = (*v_r26).x4;
                sp28.z = f31;
                PSMTXMultVec((*jobj).mtx.as_mut_ptr(), core::ptr::addr_of_mut!(sp28) as *mut Vec, core::ptr::addr_of_mut!(sp28) as *mut Vec);
                (*v_r26).pos.x = sp28.x;
                (*v_r26).pos.y = sp28.y;
                if (((mpLib_804D64CC == 0) || ((*v_r26).pos.x != (*v_r26).x10)) || ((*v_r26).pos.y != (*v_r26).x14)) || ({ var_r25 = var_r25 + 1; var_r25 } <= 1) {
                    i = i + 1;
                    v_r26 = v_r26.offset(1 as isize);
                } else {
                    break 'b273;
                }
            }
            {
                mtx = HSD_JObjGetMtxPtr(jobj);
                f1 = 0.0f32;
                if (((((f1 != (*mtx.offset(0 as isize))[1 as usize]) || (f1 != (*mtx.offset(0 as isize))[2 as usize])) || (f1 != (*mtx.offset(1 as isize))[0 as usize])) || (f1 != (*mtx.offset(1 as isize))[2 as usize])) || (f1 != (*mtx.offset(2 as isize))[0 as usize])) || (f1 != (*mtx.offset(2 as isize))[1 as usize]) {
                    (*joint).flags = (*joint).flags | ((CollJoint_B9 as i32) as u32);
                }
                (*joint).flags = (*joint).flags | ((CollJoint_B8 as i32) as u32);
                if !(((*joint).flags & ((CollJoint_B10 as i32) as u32)) != 0) {
                    sp28.x = (*(*joint).inner).left_bound;
                    sp28.y = (*(*joint).inner).bottom_bound;
                    sp28.z = 0.0f32;
                    PSMTXMultVec((*jobj).mtx.as_mut_ptr(), core::ptr::addr_of_mut!(sp28) as *mut Vec, core::ptr::addr_of_mut!(sp28) as *mut Vec);
                    (*joint).bounding_min.x = sp28.x;
                    (*joint).bounding_min.y = sp28.y;
                    sp28.x = (*(*joint).inner).right_bound;
                    sp28.y = (*(*joint).inner).top_bound;
                    sp28.z = 0.0f32;
                    PSMTXMultVec((*jobj).mtx.as_mut_ptr(), core::ptr::addr_of_mut!(sp28) as *mut Vec, core::ptr::addr_of_mut!(sp28) as *mut Vec);
                    (*joint).bounding_max.x = sp28.x;
                    (*joint).bounding_max.y = sp28.y;
                    if ((*joint).flags & ((CollJoint_B9 as i32) as u32)) != 0 {
                        sp28.x = (*(*joint).inner).right_bound;
                        sp28.y = (*(*joint).inner).bottom_bound;
                        sp28.z = 0.0f32;
                        PSMTXMultVec((*jobj).mtx.as_mut_ptr(), core::ptr::addr_of_mut!(sp28) as *mut Vec, core::ptr::addr_of_mut!(sp28) as *mut Vec);
                        f30 = sp28.x;
                        f31 = sp28.y;
                        sp28.x = (*(*joint).inner).left_bound;
                        sp28.y = (*(*joint).inner).top_bound;
                        sp28.z = 0.0f32;
                        PSMTXMultVec((*jobj).mtx.as_mut_ptr(), core::ptr::addr_of_mut!(sp28) as *mut Vec, core::ptr::addr_of_mut!(sp28) as *mut Vec);
                        f1 = (*joint).bounding_min.x;
                        f0 = (*joint).bounding_max.x;
                        f2 = sp28.x;
                        f3 = sp28.y;
                        if f1 > f0 {
                            (*joint).bounding_min.x = f0;
                        }
                        if (*joint).bounding_min.x > f30 {
                            (*joint).bounding_min.x = f30;
                        }
                        if (*joint).bounding_min.x > f2 {
                            (*joint).bounding_min.x = f2;
                        }
                        if (*joint).bounding_max.x < f1 {
                            (*joint).bounding_max.x = f1;
                        }
                        if (*joint).bounding_max.x < f30 {
                            (*joint).bounding_max.x = f30;
                        }
                        if (*joint).bounding_max.x < f2 {
                            (*joint).bounding_max.x = f2;
                        }
                        f1 = (*joint).bounding_min.y;
                        f0 = (*joint).bounding_max.y;
                        if f1 > f0 {
                            (*joint).bounding_min.y = f0;
                        }
                        if (*joint).bounding_min.y > f31 {
                            (*joint).bounding_min.y = f31;
                        }
                        if (*joint).bounding_min.y > f3 {
                            (*joint).bounding_min.y = f3;
                        }
                        if (*joint).bounding_max.y < f1 {
                            (*joint).bounding_max.y = f1;
                        }
                        if (*joint).bounding_max.y < f31 {
                            (*joint).bounding_max.y = f31;
                        }
                        if (*joint).bounding_max.y < f3 {
                            (*joint).bounding_max.y = f3;
                        }
                    }
                    (*joint).bounding_min.x = (*joint).bounding_min.x - 3e+01f32;
                    (*joint).bounding_max.x = (*joint).bounding_max.x + 3e+01f32;
                    (*joint).bounding_min.y = (*joint).bounding_min.y - 3e+01f32;
                    (*joint).bounding_max.y = (*joint).bounding_max.y + 3e+01f32;
                }
            }
        }
        mpJointUpdateDynamics(joint_id);
    }
    if ((*joint).flags & ((CollJoint_Hidden as i32) as u32)) != 0 {
        mpJointUnhide(joint_id);
    }
    var_r6 = 0;
    if (!(((*joint).flags & (((CollJoint_Hidden as i32) | (CollJoint_B11 as i32)) as u32)) != 0)) && (((*joint).flags & ((CollJoint_Enabled as i32) as u32)) != 0) {
        var_r6 = 1;
    }
    mpScriptIslandRefresh(joint_id, (*(*joint).inner).vtx_start as i32, (*(*joint).inner).vtx_count as i32, var_r6);
}

/// `mpJointUpdateBounding`
pub unsafe fn mpJointUpdateBounding(mut joint_id: i32) {
    let mut joint: *mut CollJoint = core::ptr::addr_of_mut!(*groundCollJoint.offset(joint_id as isize)) as *mut CollJoint;
    let mut vtx: *mut CollVtx = core::ptr::addr_of_mut!(*groundCollVtx.offset((*(*joint).inner).vtx_start as isize)) as *mut CollVtx;
    let mut count: i32 = (*(*joint).inner).vtx_count as i32;
    let mut i: i32 = 0;
    {
        i = 0;
        'l277: while i < count {
            if (*joint).bounding_min.x > ((*vtx).pos.x - 3e+01f32) {
                (*joint).bounding_min.x = (*vtx).pos.x - 3e+01f32;
            }
            if (*joint).bounding_max.x < ((*vtx).pos.x + 3e+01f32) {
                (*joint).bounding_max.x = (*vtx).pos.x + 3e+01f32;
            }
            if (*joint).bounding_min.y > ((*vtx).pos.y - 3e+01f32) {
                (*joint).bounding_min.y = (*vtx).pos.y - 3e+01f32;
            }
            if (*joint).bounding_max.y < ((*vtx).pos.y + 3e+01f32) {
                (*joint).bounding_max.y = (*vtx).pos.y + 3e+01f32;
            }
            vtx = vtx.offset(1 as isize);
            i = i + 1;
        }
    }
}

/// `mpLib_8005667C`
pub unsafe fn mpLib_8005667C(mut joint_id: i32) {
    let mut var_r6: i32 = 0;
    let mut joint: *mut CollJoint = core::ptr::addr_of_mut!(*groundCollJoint.offset(joint_id as isize)) as *mut CollJoint;
    if (!(((*joint).flags & (((CollJoint_Hidden as i32) | (CollJoint_B11 as i32)) as u32)) != 0)) && (((*joint).flags & ((CollJoint_Enabled as i32) as u32)) != 0) {
        var_r6 = 1;
    }
    mpScriptIslandRefresh(joint_id, (*(*joint).inner).vtx_start as i32, (*(*joint).inner).vtx_count as i32, var_r6);
}

/// `mpVtxGetPos`
pub unsafe fn mpVtxGetPos(mut vtx_id: i32, mut x_out: *mut f32, mut y_out: *mut f32) {
    let mut vtx: *mut CollVtx = core::ptr::addr_of_mut!(*groundCollVtx.offset(vtx_id as isize)) as *mut CollVtx;
    *x_out = (*vtx).pos.x;
    *y_out = (*vtx).pos.y;
}

/// `mpVtxSetPos`
pub unsafe fn mpVtxSetPos(mut vtx_id: i32, mut x: f32, mut y: f32) {
    let mut vtx: *mut CollVtx = core::ptr::addr_of_mut!(*groundCollVtx.offset(vtx_id as isize)) as *mut CollVtx;
    (*vtx).pos.x = x;
    (*vtx).pos.y = y;
}

/// `mpLineSetPos`
pub unsafe fn mpLineSetPos(mut line_id: i32, mut x0: f32, mut y0: f32, mut x1: f32, mut y1: f32) {
    let mut line: *mut CollLine = core::ptr::addr_of_mut!(*groundCollLine.offset(line_id as isize)) as *mut CollLine;
    mpVtxSetPos((*(*line).x0).v0_idx as i32, x0, y0);
    mpVtxSetPos((*(*line).x0).v1_idx as i32, x1, y1);
}

/// `mpLib_80056758`
pub unsafe fn mpLib_80056758(mut line_id: i32, mut x0: f32, mut y0: f32, mut x1: f32, mut y1: f32) {
    let mut line: *mut CollLine = core::ptr::addr_of_mut!(*groundCollLine.offset(line_id as isize)) as *mut CollLine;
    let mut vtx: *mut CollVtx = core::ptr::addr_of_mut!(*groundCollVtx.offset((*(*line).x0).v0_idx as isize)) as *mut CollVtx;
    (*vtx).pos.x = (*vtx).x0 + x0;
    (*vtx).pos.y = (*vtx).x4 + y0;
    vtx = core::ptr::addr_of_mut!(*groundCollVtx.offset((*(*line).x0).v1_idx as isize)) as *mut CollVtx;
    (*vtx).pos.x = (*vtx).x0 + x1;
    (*vtx).pos.y = (*vtx).x4 + y1;
}

/// `mpGetSpeed`
pub unsafe fn mpGetSpeed(mut line_id: i32, mut pos: *mut Vec, mut speed: *mut Vec) -> i32 {
    let mut new_x: f32 = 0.0;
    let mut new_y: f32 = 0.0;
    let mut v0_r5: *mut CollVtx = null_mut();
    let mut v1_r6: *mut CollVtx = null_mut();
    if !(mpLib_80054ED8(line_id) != 0) {
        return 0;
    }
    v0_r5 = core::ptr::addr_of_mut!(*groundCollVtx.offset((*(*groundCollLine.offset(line_id as isize)).x0).v0_idx as isize)) as *mut CollVtx;
    v1_r6 = core::ptr::addr_of_mut!(*groundCollVtx.offset((*(*groundCollLine.offset(line_id as isize)).x0).v1_idx as isize)) as *mut CollVtx;
    mpRemap2d(core::ptr::addr_of_mut!(new_x) as *mut f32, core::ptr::addr_of_mut!(new_y) as *mut f32, (*v0_r5).x10, (*v0_r5).x14, (*v1_r6).x10, (*v1_r6).x14, (*v0_r5).pos.x, (*v0_r5).pos.y, (*v1_r6).pos.x, (*v1_r6).pos.y, (*pos).x, (*pos).y);
    (*speed).x = new_x - (*pos).x;
    (*speed).y = new_y - (*pos).y;
    (*speed).z = 0.0f32;
    if ext::DbLevel >= ((DbLKind_DebugRom as i32) as u32) {
        if ((if (*speed).x < (0 as f32) { -(*speed).x } else { (*speed).x }) > 1e+04f32) || ((if (*speed).y < (0 as f32) { -(*speed).y } else { (*speed).y }) > 1e+04f32) {
            ext::OSReport(&format!("{}:{}: Error: mpGetSpeed() x={:.6} y={:.6}\n", core::ffi::CStr::from_ptr((c"src/melee/mp/mplib.c".as_ptr() as *mut i8) as *const core::ffi::c_char).to_string_lossy(), 5333, (*speed).x as f64, (*speed).y as f64));
            if !(((if (*speed).x < (0 as f32) { -(*speed).x } else { (*speed).x }) > 1e+04f32) || ((if (*speed).y < (0 as f32) { -(*speed).y } else { (*speed).y }) > 1e+04f32)) { (); } else { panic!("mpGetSpeed: assertion failed"); }
            if 0 != 0 { (); } else { panic!("mpGetSpeed: assertion failed"); }
        }
    }
    return 1;
}

/// `mpLib_StageRow`
unsafe fn mpLib_StageRow() -> i32 {
    let mut grkind: i32 = ext::stage_info.grkind as i32;
    let mut row: i32 = 0;
    if (grkind >= 0) && (grkind < (71_u64 as i32)) {
        return grkind;
    }
    row = ext::Mex_GrLineTypeRow(grkind);
    if (row >= 0) && (row < (71_u64 as i32)) {
        return row;
    }
    if mpLib_StageRow_warned != grkind {
        mpLib_StageRow_warned = grkind;
        ext::OSReport(&format!("mpLib: stage grkind={} has no row in mpLib_803BF248[{}] and mexData gave {} - using row 0\n", grkind, 71_u64 as i32, row));
    }
    return 0;
}

/// `mpLib_800569EC`
pub unsafe fn mpLib_800569EC(mut unk: u32) -> f32 {
    return (*(*mpLib_803BF248[mpLib_StageRow() as usize].x4)[(unk as u8) as usize]).x0;
}

/// `mpLib_80056A1C`
pub unsafe fn mpLib_80056A1C(mut arg0: i32, mut arg1: *mut i32) -> *mut i32 {
    let mut temp: *mut mpLib_803BF248_t_x4 = (*mpLib_803BF248[mpLib_StageRow() as usize].x4)[(arg0 as u8) as usize];
    *arg1 = (*temp).x14[0 as usize];
    return (*temp).x4.as_mut_ptr();
}

/// `mpLib_80056A54`
pub unsafe fn mpLib_80056A54(mut arg0: i32, mut arg1: *mut i32) -> i32 {
    let mut temp: *mut mpLib_803BF248_t_x4 = (*mpLib_803BF248[mpLib_StageRow() as usize].x4)[(arg0 as u8) as usize];
    *arg1 = (*temp).x14[2 as usize];
    return (*temp).x14[1 as usize];
}

/// `mpLib_80056A8C`
pub unsafe fn mpLib_80056A8C(mut arg0: i32, mut arg1: *mut i32) -> *mut i32 {
    let mut temp: *mut mpLib_803BF248_t_x4 = (*mpLib_803BF248[mpLib_StageRow() as usize].x4)[(arg0 as u8) as usize];
    *arg1 = (*temp).x30[0 as usize];
    return (*temp).x20.as_mut_ptr();
}

/// `mpLib_80056AC4`
pub unsafe fn mpLib_80056AC4(mut arg0: i32, mut arg1: *mut i32) -> i32 {
    let mut temp: *mut mpLib_803BF248_t_x4 = (*mpLib_803BF248[mpLib_StageRow() as usize].x4)[(arg0 as u8) as usize];
    *arg1 = (*temp).x30[2 as usize];
    return (*temp).x30[1 as usize];
}

/// `mpLib_80056AFC`
pub unsafe fn mpLib_80056AFC(mut arg0: i32, mut arg1: *mut i32) -> *mut i32 {
    let mut temp: *mut mpLib_803BF248_t_x4 = (*mpLib_803BF248[mpLib_StageRow() as usize].x4)[(arg0 as u8) as usize];
    *arg1 = (*temp).x4C[0 as usize];
    return (*temp).x3C.as_mut_ptr();
}

/// `mpLib_80056B34`
pub unsafe fn mpLib_80056B34(mut arg0: i32, mut arg1: *mut i32) -> i32 {
    let mut temp: *mut mpLib_803BF248_t_x4 = (*mpLib_803BF248[mpLib_StageRow() as usize].x4)[(arg0 as u8) as usize];
    *arg1 = (*temp).x4C[2 as usize];
    return (*temp).x4C[1 as usize];
}

/// `sqrtf_store`
unsafe fn sqrtf_store(mut x: f32, mut y: *mut f32) -> f32 {
    if x > 0.0f32 {
        let mut guess: f64 = melee_core::fp::frsqrte(x as f64);
        guess = (0.5f64 * guess) * (-(guess * guess)).mul_add(x as f64, 3.0f64);
        guess = (0.5f64 * guess) * (-(guess * guess)).mul_add(x as f64, 3.0f64);
        guess = (0.5f64 * guess) * (-(guess * guess)).mul_add(x as f64, 3.0f64);
        *y = ((x as f64) * guess) as f32;
        return *y;
    }
    return x;
}

/// `mpJointFromLine`
pub unsafe fn mpJointFromLine(mut line_id: i32) -> i32 {
    if line_id != (-1_i32) {
        let mut i: i32 = 0;
        let mut v0_idx: i32 = 0;
        let mut joint: *mut CollJoint = null_mut();
        let mut count: i32 = 0;
        {
            if (line_id == (-1_i32)) || (line_id >= (*mpLib_804D64B4).line_count) {
                if 0 != 0 { (); } else { ext::OSReport(&format!("{}:{}:not found lineID={}\n", core::ffi::CStr::from_ptr((c"src/melee/mp/mplib.c".as_ptr() as *mut i8) as *const core::ffi::c_char).to_string_lossy(), 5459, line_id)); panic!("mpJointFromLine: assertion failed"); }
            }
        }
        v0_idx = (*(*groundCollLine.offset(line_id as isize)).x0).v0_idx as i32;
        count = (*mpLib_804D64B4).joint_count;
        joint = groundCollJoint;
        {
            i = 0;
            'l278: while i < count {
                if (((*(*joint).inner).vtx_start as i32) <= v0_idx) && (v0_idx < (((*(*joint).inner).vtx_start as i32) + ((*(*joint).inner).vtx_count as i32))) {
                    return (joint.offset_from(groundCollJoint) as i64) as i32;
                }
                joint = joint.offset(1);
                i = i + 1;
            }
        }
    }
    return (-1_i32);
}

/// `mpLib_80056C54`: walk `var_f25` units along the floor run from `pos` (`arg7` is the budget
/// spent on wall / ceiling lines). The two `goto block_30/55` jumps into the else arm become the
/// `run` flag. `sqrt_tmp - 4` (a stack-layout hack in the C) is an ordinary scratch float here.
pub unsafe fn mpLib_80056C54(mut line_id: i32, mut pos: *mut Vec3, mut line_id_out: *mut i32, mut vec_out: *mut Vec3, mut flags_out: *mut u32, mut normal_out: *mut Vec3, mut var_f25: f32, mut arg7: f32) -> i32 {
    let mut sp64: f32 = 0.0;
    let mut sp58: Vec3 = core::mem::zeroed();
    let mut sp4C: Vec3 = core::mem::zeroed();
    let mut sqrt_tmp: f32 = 0.0;
    let mut dist_f28: f32 = 0.0;
    let mut total_dist_f27: f32 = 0.0;
    let mut x_f2: f32;
    let mut y_f0: f32;
    let mut result_r30: i32 = 1;
    let mut new_id_r5: i32;
    let mut flags_r0: u32;

    if mpLib_80054ED8(line_id) == 0 {
        return 0;
    }

    if mpLib_8004DD90_Floor(line_id, pos, &mut sp64, null_mut(), null_mut()) == -1 {
        return 0;
    }
    sp58 = *pos;
    sp58.y += sp64;
    if var_f25 > 0.0 {
        loop {
            mpLineGetV1Pos(line_id, &mut sp4C);
            x_f2 = (sp58.x - sp4C.x) * (sp58.x - sp4C.x);
            y_f0 = (sp58.y - sp4C.y) * (sp58.y - sp4C.y);
            dist_f28 = sqrtf_store(x_f2 + y_f0, &mut sqrt_tmp);
            flags_r0 = mpLineGetKind(line_id) as u32;
            let mut run = false;
            if flags_r0 & 0xC != 0 {
                total_dist_f27 += dist_f28;
                if total_dist_f27 > arg7 {
                    result_r30 = 0;
                } else {
                    run = true; // goto block_30
                }
            } else if flags_r0 & 2 != 0 {
                result_r30 = 0;
            } else {
                total_dist_f27 = 0.0;
                run = true;
            }
            if run && !(dist_f28 > var_f25) {
                new_id_r5 = mpLineGetNext(line_id);
                if new_id_r5 == -1 {
                    result_r30 = 0;
                } else {
                    var_f25 -= dist_f28;
                    line_id = new_id_r5;
                    sp58 = sp4C;
                    continue;
                }
            }
            break;
        }
    } else {
        var_f25 = -var_f25;
        loop {
            mpLineGetV0Pos(line_id, &mut sp4C);
            x_f2 = (sp58.x - sp4C.x) * (sp58.x - sp4C.x);
            y_f0 = (sp58.y - sp4C.y) * (sp58.y - sp4C.y);
            dist_f28 = sqrtf_store(x_f2 + y_f0, &mut sqrt_tmp);
            flags_r0 = mpLineGetKind(line_id) as u32;
            let mut run = false;
            if flags_r0 & 0xC != 0 {
                total_dist_f27 += dist_f28;
                if total_dist_f27 > arg7 {
                    result_r30 = 0;
                } else {
                    run = true; // goto block_55
                }
            } else if flags_r0 & 2 != 0 {
                result_r30 = 0;
            } else {
                total_dist_f27 = 0.0;
                run = true;
            }
            if run && !(dist_f28 > var_f25) {
                new_id_r5 = mpLineGetPrev(line_id);
                if new_id_r5 == -1 {
                    result_r30 = 0;
                } else {
                    var_f25 -= dist_f28;
                    line_id = new_id_r5;
                    sp58 = sp4C;
                    continue;
                }
            }
            break;
        }
    }
    if line_id != -1 {
        if (mpLineGetKind(line_id) as u32) & (CollLine_Floor as u32) == 0 {
            line_id = -1;
        }
    }
    if !line_id_out.is_null() {
        *line_id_out = line_id;
    }
    if line_id != -1 {
        if !flags_out.is_null() {
            *flags_out = mpLineGetFlags(line_id);
        }
        if !normal_out.is_null() {
            mpLineGetNormal(line_id, normal_out);
        }
    }
    if !vec_out.is_null() {
        if result_r30 != 0 {
            if (dist_f28 as f64) < 0.0001 {
                *vec_out = sp58;
            } else {
                let temp_f2_5: f32 = var_f25 / dist_f28;
                (*vec_out).x = fmadds(temp_f2_5, sp4C.x - sp58.x, sp58.x);
                (*vec_out).y = fmadds(temp_f2_5, sp4C.y - sp58.y, sp58.y);
                (*vec_out).z = fmadds(temp_f2_5, sp4C.z - sp58.z, sp58.z);
            }
        } else {
            *vec_out = sp4C;
        }
    }
    result_r30
}

/// `mpLib_80057424`
pub unsafe fn mpLib_80057424(mut joint_id: i32) {
    let mut joint: *mut CollJoint = core::ptr::addr_of_mut!(*groundCollJoint.offset(joint_id as isize)) as *mut CollJoint;
    let mut j_inner: *mut MapJoint = (*joint).inner;
    let mut count: u32 = (*j_inner).vtx_count as u32;
    let mut new_var: i32 = 0;
    let mut vtx: *mut CollVtx = core::ptr::addr_of_mut!(*groundCollVtx.offset((*j_inner).vtx_start as isize)) as *mut CollVtx;
    {
        joint_id = 0;
        'l279: while joint_id < { new_var = count as i32; new_var } {
            (*vtx).x10 = (*vtx).pos.x;
            (*vtx).x14 = (*vtx).pos.y;
            vtx = vtx.offset(1);
            joint_id = joint_id + 1;
        }
    }
}

/// `mpLib_80057528`
pub unsafe fn mpLib_80057528(mut line_id: i32) {
    let mut joint_id: i32 = mpJointFromLine(line_id);
    if joint_id != (-1_i32) {
        let mut line: *mut CollLine = core::ptr::addr_of_mut!(*groundCollLine.offset(line_id as isize)) as *mut CollLine;
        let mut joint: *mut CollJoint = core::ptr::addr_of_mut!(*groundCollJoint.offset(joint_id as isize)) as *mut CollJoint;
        (*line).flags = (*line).flags | ((1 << 16) as u32);
        mpScriptIslandRefresh(joint_id, (*(*joint).inner).vtx_start as i32, (*(*joint).inner).vtx_count as i32, (!(((*joint).flags & ((CollJoint_B11 as i32) as u32)) != 0)) as i32);
        (*joint).set_xE((1 as u8) as u8);
    }
}

/// `mpLib_800575B0`
pub unsafe fn mpLib_800575B0(mut line_id: i32) {
    let mut joint_id: i32 = mpJointFromLine(line_id);
    if joint_id != (-1_i32) {
        let mut line: *mut CollLine = core::ptr::addr_of_mut!(*groundCollLine.offset(line_id as isize)) as *mut CollLine;
        let mut joint: *mut CollJoint = core::ptr::addr_of_mut!(*groundCollJoint.offset(joint_id as isize)) as *mut CollJoint;
        (*line).flags = (*line).flags & ((!(1 << 16)) as u32);
        mpScriptIslandRefresh(joint_id, (*(*joint).inner).vtx_start as i32, (*(*joint).inner).vtx_count as i32, (!(((*joint).flags & ((CollJoint_B11 as i32) as u32)) != 0)) as i32);
        (*joint).set_xE((1 as u8) as u8);
    }
}

/// `mpJointListAdd`
pub unsafe fn mpJointListAdd(mut joint_id: i32) {
    let mut joint: *mut CollJoint = null_mut();
    let mut j_inner: *mut MapJoint = null_mut();
    let mut count_r7: i32 = 0;
    let mut line_r6: *mut CollLine = null_mut();
    joint = core::ptr::addr_of_mut!(*groundCollJoint.offset(joint_id as isize)) as *mut CollJoint;
    if ((*joint).flags & ((CollJoint_Enabled as i32) as u32)) != 0 {
        return;
    }
    (*joint).flags = (*joint).flags | ((CollJoint_Enabled as i32) as u32);
    if jointListStart.is_null() {
        jointListStart = joint;
    } else {
        (*jointListEnd).next = joint;
    }
    jointListEnd = joint;
    (*joint).next = null_mut();
    j_inner = (*joint).inner;
    count_r7 = (*j_inner).ranges[(MapLineGroup_Floor as i32) as usize].count as i32;
    line_r6 = core::ptr::addr_of_mut!(*groundCollLine.offset((*j_inner).ranges[(MapLineGroup_Floor as i32) as usize].start as isize)) as *mut CollLine;
    'l280: while { let __t281 = count_r7; count_r7 = count_r7 - 1; __t281 } > 0 {
        (*line_r6).flags = (*line_r6).flags | ((1 << 16) as u32);
        line_r6 = line_r6.offset(1);
    }
    j_inner = (*joint).inner;
    count_r7 = (*j_inner).ranges[(MapLineGroup_Ceiling as i32) as usize].count as i32;
    line_r6 = core::ptr::addr_of_mut!(*groundCollLine.offset((*j_inner).ranges[(MapLineGroup_Ceiling as i32) as usize].start as isize)) as *mut CollLine;
    'l282: while { let __t283 = count_r7; count_r7 = count_r7 - 1; __t283 } > 0 {
        (*line_r6).flags = (*line_r6).flags | ((1 << 16) as u32);
        line_r6 = line_r6.offset(1);
    }
    j_inner = (*joint).inner;
    count_r7 = (*j_inner).ranges[(MapLineGroup_LeftWall as i32) as usize].count as i32;
    line_r6 = core::ptr::addr_of_mut!(*groundCollLine.offset((*j_inner).ranges[(MapLineGroup_LeftWall as i32) as usize].start as isize)) as *mut CollLine;
    'l284: while { let __t285 = count_r7; count_r7 = count_r7 - 1; __t285 } > 0 {
        (*line_r6).flags = (*line_r6).flags | ((1 << 16) as u32);
        line_r6 = line_r6.offset(1);
    }
    j_inner = (*joint).inner;
    count_r7 = (*j_inner).ranges[(MapLineGroup_RightWall as i32) as usize].count as i32;
    line_r6 = core::ptr::addr_of_mut!(*groundCollLine.offset((*j_inner).ranges[(MapLineGroup_RightWall as i32) as usize].start as isize)) as *mut CollLine;
    'l286: while { let __t287 = count_r7; count_r7 = count_r7 - 1; __t287 } > 0 {
        (*line_r6).flags = (*line_r6).flags | ((1 << 16) as u32);
        line_r6 = line_r6.offset(1);
    }
    j_inner = (*joint).inner;
    count_r7 = (*j_inner).ranges[(MapLineGroup_Dynamic as i32) as usize].count as i32;
    line_r6 = core::ptr::addr_of_mut!(*groundCollLine.offset((*j_inner).ranges[(MapLineGroup_Dynamic as i32) as usize].start as isize)) as *mut CollLine;
    'l288: while { let __t289 = count_r7; count_r7 = count_r7 - 1; __t289 } > 0 {
        (*line_r6).flags = (*line_r6).flags | ((1 << 16) as u32);
        line_r6 = line_r6.offset(1);
    }
    mpLib_80057424(joint_id);
    j_inner = (*joint).inner;
    mpScriptIslandRefresh(joint_id, (*j_inner).vtx_start as i32, (*j_inner).vtx_count as i32, (!(((*joint).flags & ((CollJoint_B11 as i32) as u32)) != 0)) as i32);
    (*joint).set_xE((1 as u8) as u8);
}

/// `mpJointListUnlink`
pub unsafe fn mpJointListUnlink(mut arg0: *mut CollJoint) {
    let mut cur: *mut CollJoint = null_mut();
    if arg0 == jointListStart {
        if arg0 == jointListEnd {
            jointListEnd = null_mut();
            jointListStart = null_mut();
        } else {
            jointListStart = (*jointListStart).next;
        }
        return;
    }
    {
        cur = jointListStart;
        'l290: while !cur.is_null() {
            if (*cur).next == arg0 {
                if arg0 == jointListEnd {
                    jointListEnd = cur;
                }
                (*cur).next = (*(*cur).next).next;
                return;
            }
            cur = (*cur).next;
        }
    }
}

/// `mpLib_80057BC0`
pub unsafe fn mpLib_80057BC0(mut joint_id: i32) {
    let mut joint: *mut CollJoint = null_mut();
    let mut j_inner: *mut MapJoint = null_mut();
    let mut count: i32 = 0;
    let mut start: i32 = 0;
    let mut line_r6: *mut CollLine = null_mut();
    joint = core::ptr::addr_of_mut!(*groundCollJoint.offset(joint_id as isize)) as *mut CollJoint;
    if !(((*joint).flags & ((CollJoint_Enabled as i32) as u32)) != 0) {
        return;
    }
    (*joint).flags = (*joint).flags & ((!(CollJoint_Enabled as i32)) as u32);
    mpJointListUnlink(joint);
    j_inner = (*joint).inner;
    count = (*j_inner).ranges[(MapLineGroup_Floor as i32) as usize].count as i32;
    line_r6 = core::ptr::addr_of_mut!(*groundCollLine.offset((*j_inner).ranges[(MapLineGroup_Floor as i32) as usize].start as isize)) as *mut CollLine;
    'l291: while count > 0 {
        (*line_r6).flags = (*line_r6).flags & ((!(1 << 16)) as u32);
        line_r6 = line_r6.offset(1);
        count = count - 1;
    }
    j_inner = (*joint).inner;
    count = (*j_inner).ranges[(MapLineGroup_Ceiling as i32) as usize].count as i32;
    line_r6 = core::ptr::addr_of_mut!(*groundCollLine.offset((*j_inner).ranges[(MapLineGroup_Ceiling as i32) as usize].start as isize)) as *mut CollLine;
    'l292: while count > 0 {
        (*line_r6).flags = (*line_r6).flags & ((!(1 << 16)) as u32);
        line_r6 = line_r6.offset(1);
        count = count - 1;
    }
    j_inner = (*joint).inner;
    count = (*j_inner).ranges[(MapLineGroup_LeftWall as i32) as usize].count as i32;
    line_r6 = core::ptr::addr_of_mut!(*groundCollLine.offset((*j_inner).ranges[(MapLineGroup_LeftWall as i32) as usize].start as isize)) as *mut CollLine;
    'l293: while count > 0 {
        (*line_r6).flags = (*line_r6).flags & ((!(1 << 16)) as u32);
        line_r6 = line_r6.offset(1);
        count = count - 1;
    }
    j_inner = (*joint).inner;
    count = (*j_inner).ranges[(MapLineGroup_RightWall as i32) as usize].count as i32;
    line_r6 = core::ptr::addr_of_mut!(*groundCollLine.offset((*j_inner).ranges[(MapLineGroup_RightWall as i32) as usize].start as isize)) as *mut CollLine;
    'l294: while count > 0 {
        (*line_r6).flags = (*line_r6).flags & ((!(1 << 16)) as u32);
        line_r6 = line_r6.offset(1);
        count = count - 1;
    }
    j_inner = (*joint).inner;
    count = (*j_inner).ranges[(MapLineGroup_Dynamic as i32) as usize].count as i32;
    line_r6 = core::ptr::addr_of_mut!(*groundCollLine.offset((*j_inner).ranges[(MapLineGroup_Dynamic as i32) as usize].start as isize)) as *mut CollLine;
    'l295: while count > 0 {
        (*line_r6).flags = (*line_r6).flags & ((!(1 << 16)) as u32);
        line_r6 = line_r6.offset(1);
        count = count - 1;
    }
    j_inner = (*joint).inner;
    mpScriptIslandRefresh(joint_id, (*j_inner).vtx_start as i32, (*j_inner).vtx_count as i32, 0);
    (*joint).set_xE((1 as u8) as u8);
}

/// `mpLib_80057FDC`
pub unsafe fn mpLib_80057FDC(mut joint_id: i32) {
    let mut j_inner: *mut MapJoint = null_mut();
    let mut var_r6: i32 = 0;
    let mut joint: *mut CollJoint = core::ptr::addr_of_mut!(*groundCollJoint.offset(joint_id as isize)) as *mut CollJoint;
    (*joint).flags = (*joint).flags & ((!(CollJoint_B11 as i32)) as u32);
    if (!(((*joint).flags & (((CollJoint_Hidden as i32) | (CollJoint_B11 as i32)) as u32)) != 0)) && (((*joint).flags & ((CollJoint_Enabled as i32) as u32)) != 0) {
        var_r6 = 1;
    }
    j_inner = (*joint).inner;
    mpScriptIslandRefresh(joint_id, (*j_inner).vtx_start as i32, (*j_inner).vtx_count as i32, var_r6);
}

/// `mpLib_80058044`
pub unsafe fn mpLib_80058044(mut joint_id: i32) {
    let mut j_inner: *mut MapJoint = null_mut();
    let mut var_r6: i32 = 0;
    let mut joint: *mut CollJoint = core::ptr::addr_of_mut!(*groundCollJoint.offset(joint_id as isize)) as *mut CollJoint;
    (*joint).flags = (*joint).flags | ((CollJoint_B11 as i32) as u32);
    if (!(((*joint).flags & (((CollJoint_Hidden as i32) | (CollJoint_B11 as i32)) as u32)) != 0)) && (((*joint).flags & ((CollJoint_Enabled as i32) as u32)) != 0) {
        var_r6 = 1;
    }
    j_inner = (*joint).inner;
    mpScriptIslandRefresh(joint_id, (*j_inner).vtx_start as i32, (*j_inner).vtx_count as i32, var_r6);
}

/// `mpJointSetB10`
pub unsafe fn mpJointSetB10(mut joint_id: i32) {
    let mut joint: *mut CollJoint = core::ptr::addr_of_mut!(*groundCollJoint.offset(joint_id as isize)) as *mut CollJoint;
    (*joint).flags = (*joint).flags | ((CollJoint_B10 as i32) as u32);
}

/// `mpJointSetCb1`
pub unsafe fn mpJointSetCb1(mut joint_id: i32, mut user_data: *mut c_void, mut cb: Option<unsafe extern "C" fn(*mut c_void, i32, *mut CollData, i32, u32, f32)>) {
    let mut joint: *mut CollJoint = core::ptr::addr_of_mut!(*groundCollJoint.offset(joint_id as isize)) as *mut CollJoint;
    (*joint).cb_0 = cb;
    (*joint).cb_data_0 = user_data as *mut Ground;
}

/// `mpJointClearCb1`
pub unsafe fn mpJointClearCb1(mut joint_id: i32) {
    let mut joint: *mut CollJoint = core::ptr::addr_of_mut!(*groundCollJoint.offset(joint_id as isize)) as *mut CollJoint;
    (*joint).cb_0 = None;
    (*joint).cb_data_0 = null_mut();
}

/// `mpJointGetCb1`
pub unsafe fn mpJointGetCb1(mut joint_id: i32, mut cb: *mut Option<unsafe extern "C" fn(*mut c_void, i32, *mut CollData, i32, u32, f32)>, mut user_data: *mut *mut c_void) {
    let mut joint: *mut CollJoint = core::ptr::addr_of_mut!(*groundCollJoint.offset(joint_id as isize)) as *mut CollJoint;
    *cb = (*joint).cb_0;
    *user_data = (*joint).cb_data_0 as *mut c_void;
}

/// `mpLib_8005811C`
pub unsafe fn mpLib_8005811C(mut coll: *mut CollData, mut ledge_id: i32) {
    if ledge_id != (-1_i32) {
        let mut joint_id: i32 = mpJointFromLine(ledge_id);
        if joint_id != (-1_i32) {
            let mut cb: Option<unsafe extern "C" fn(*mut c_void, i32, *mut CollData, i32, u32, f32)> = (*groundCollJoint.offset(joint_id as isize)).cb_0;
            let mut gp: *mut Ground = (*groundCollJoint.offset(joint_id as isize)).cb_data_0;
            if cb.is_some() {
                cb.unwrap()(gp as *mut c_void, joint_id, coll, (*coll).x50 as i32, 3 as u32, 0.0f32);
            }
        }
    }
}

/// `mpJointSetCb2`
pub unsafe fn mpJointSetCb2(mut joint_id: i32, mut gp: *mut c_void, mut cb: Option<unsafe extern "C" fn(*mut c_void, i32, *mut CollData, i32, u32, f32)>) {
    let mut joint: *mut CollJoint = core::ptr::addr_of_mut!(*groundCollJoint.offset(joint_id as isize)) as *mut CollJoint;
    (*joint).cb_1 = cb;
    (*joint).cb_data_1 = gp as *mut Ground;
}

/// `mpJointGetCb2`
pub unsafe fn mpJointGetCb2(mut joint_id: i32, mut cb: *mut Option<unsafe extern "C" fn(*mut c_void, i32, *mut CollData, i32, u32, f32)>, mut user_data: *mut *mut c_void) {
    let mut joint: *mut CollJoint = core::ptr::addr_of_mut!(*groundCollJoint.offset(joint_id as isize)) as *mut CollJoint;
    *cb = (*joint).cb_1;
    *user_data = (*joint).cb_data_1 as *mut c_void;
}

/// `mpLib_GetJointVtxRange`
unsafe fn mpLib_GetJointVtxRange(mut joint: *mut CollJoint, mut start: *mut i32, mut count: *mut i32) {
    let mut map_joint: *mut MapJoint = (*joint).inner;
    *start = (*map_joint).vtx_start as i32;
    *count = (*map_joint).vtx_count as i32;
}

/// `mpLib_800581DC`
pub unsafe fn mpLib_800581DC(mut joint_id0: i32, mut joint_id1: i32) {
    if (ext::ScriptGame_StageJoint(joint_id0) != 0) || (ext::ScriptGame_StageJoint(joint_id1) != 0) {
        return;
    }
    let mut j0_r9: *mut CollJoint = null_mut();
    let mut j1_r10: *mut CollJoint = null_mut();
    let mut line_base: *mut CollLine = null_mut();
    let mut vi: i32 = 0;
    let mut i: i32 = 0;
    let mut vcount0: i32 = 0;
    let mut vstart0: i32 = 0;
    #[derive(Clone, Copy)] #[repr(C)] struct Local11 { p: *mut CollLine }
    let mut ln: Local11 = unsafe { core::mem::zeroed() };
    j0_r9 = core::ptr::addr_of_mut!(*groundCollJoint.offset(joint_id0 as isize)) as *mut CollJoint;
    j1_r10 = core::ptr::addr_of_mut!(*groundCollJoint.offset(joint_id1 as isize)) as *mut CollJoint;
    line_base = groundCollLine;
    {
        i = 0;
        'l296: while i < (MapLineGroup_Count as i32) {
            let mut pair: *mut MapLineRange = null_mut();
            let mut count: i32 = 0;
            let mut temp: i32 = 0;
            {
                let mut j: i32 = 0;
                let mut idx: i32 = 0;
                pair = ((*(*j0_r9).inner).ranges.as_mut_ptr()).offset(i as isize);
                count = (*pair).count as i32;
                *line_base.offset({ idx = (*pair).start as i32; idx } as isize);
                ln.p = core::ptr::addr_of_mut!(*line_base.offset(idx as isize)) as *mut CollLine;
                {
                    j = 0;
                    'l297: while j < count {
                        temp = (*(*ln.p.offset(j as isize)).x0).prev_id1 as i32;
                        if temp != (-1_i32) {
                            temp = (*(*line_base.offset(temp as isize)).x0).next_id1 as i32;
                            if (temp != (-1_i32)) && (idx != temp) {
                                (*(*ln.p.offset(j as isize)).x0).prev_id1 = (-1_i32) as i16;
                            }
                        }
                        temp = (*(*ln.p.offset(j as isize)).x0).next_id1 as i32;
                        if temp != (-1_i32) {
                            temp = (*(*line_base.offset(temp as isize)).x0).prev_id1 as i32;
                            if (temp != (-1_i32)) && (idx != temp) {
                                (*(*ln.p.offset(j as isize)).x0).next_id1 = (-1_i32) as i16;
                            }
                        }
                        j = j + 1; idx = idx + 1;
                    }
                }
            }
            {
                let mut j: i32 = 0;
                let mut idx: i32 = 0;
                pair = ((*(*j1_r10).inner).ranges.as_mut_ptr()).offset(i as isize);
                count = (*pair).count as i32;
                *line_base.offset({ idx = (*pair).start as i32; idx } as isize);
                ln.p = core::ptr::addr_of_mut!(*line_base.offset(idx as isize)) as *mut CollLine;
                {
                    j = 0;
                    'l298: while j < count {
                        temp = (*(*ln.p.offset(j as isize)).x0).prev_id1 as i32;
                        if temp != (-1_i32) {
                            temp = (*(*line_base.offset(temp as isize)).x0).next_id1 as i32;
                            if (temp != (-1_i32)) && (idx != temp) {
                                (*(*ln.p.offset(j as isize)).x0).prev_id1 = (-1_i32) as i16;
                            }
                        }
                        temp = (*(*ln.p.offset(j as isize)).x0).next_id1 as i32;
                        if temp != (-1_i32) {
                            temp = (*(*line_base.offset(temp as isize)).x0).prev_id1 as i32;
                            if (temp != (-1_i32)) && (idx != temp) {
                                (*(*ln.p.offset(j as isize)).x0).next_id1 = (-1_i32) as i16;
                            }
                        }
                        j = j + 1; idx = idx + 1;
                    }
                }
            }
            i = i + 1;
        }
    }
    mpLib_GetJointVtxRange(j0_r9, core::ptr::addr_of_mut!(vstart0) as *mut i32, core::ptr::addr_of_mut!(vcount0) as *mut i32);
    {
        vi = 0;
        'l299: while vi < vcount0 {
            let mut v0_r29: *mut CollVtx = null_mut();
            let mut v: i32 = 0;
            let mut vcount1: i32 = 0;
            let mut vid_r26: i32 = 0;
            v0_r29 = core::ptr::addr_of_mut!(*groundCollVtx.offset(vstart0 as isize)) as *mut CollVtx;
            vcount1 = (*(*j1_r10).inner).vtx_count as i32;
            vid_r26 = (*(*j1_r10).inner).vtx_start as i32;
            {
                v = 0;
                'l300: while v < vcount1 {
                    'c300: {
                        let mut var_r25: i32 = 0;
                        let mut v1: *mut CollVtx = core::ptr::addr_of_mut!(*groundCollVtx.offset(vid_r26 as isize)) as *mut CollVtx;
                        if (!(((if ((*v0_r29).pos.x - (*v1).pos.x) < (0 as f32) { -((*v0_r29).pos.x - (*v1).pos.x) } else { (*v0_r29).pos.x - (*v1).pos.x }) as f64) < 2.0f64)) || (!(((if ((*v0_r29).pos.y - (*v1).pos.y) < (0 as f32) { -((*v0_r29).pos.y - (*v1).pos.y) } else { (*v0_r29).pos.y - (*v1).pos.y }) as f64) < 2.0f64)) {
                            break 'c300;
                        }
                        {
                            var_r25 = 0;
                            'l301: while var_r25 < (MapLineGroup_Count as i32) {
                                let mut lstart_r24: i32 = 0;
                                let mut i_r23: i32 = 0;
                                let mut lcount_r22: i32 = 0;
                                lcount_r22 = (*(*j0_r9).inner).ranges[var_r25 as usize].count as i32;
                                *line_base.offset({ lstart_r24 = (*(*j0_r9).inner).ranges[var_r25 as usize].start as i32; lstart_r24 } as isize);
                                {
                                    i_r23 = 0;
                                    'l302: while i_r23 < lcount_r22 {
                                        if vstart0 == ((*(*line_base.offset(lstart_r24 as isize)).x0).v0_idx as i32) {
                                            let mut j: i32 = 0;
                                            let mut lstart_r20: i32 = 0;
                                            let mut lcount_r17: i16 = 0;
                                            {
                                                j = 0;
                                                'l303: while j < (MapLineGroup_Count as i32) {
                                                    let mut k: i32 = 0;
                                                    lcount_r17 = (*(*j1_r10).inner).ranges[j as usize].count;
                                                    *line_base.offset({ lstart_r20 = (*(*j1_r10).inner).ranges[j as usize].start as i32; lstart_r20 } as isize);
                                                    {
                                                        k = 0;
                                                        'l304: while k < (lcount_r17 as i32) {
                                                            if vid_r26 == ((*(*line_base.offset(lstart_r20 as isize)).x0).v1_idx as i32) {
                                                                (*(*line_base.offset(lstart_r24 as isize)).x0).prev_id1 = lstart_r20 as i16;
                                                                (*(*line_base.offset(lstart_r20 as isize)).x0).next_id1 = lstart_r24 as i16;
                                                            }
                                                            k = k + 1; lstart_r20 = lstart_r20 + 1;
                                                        }
                                                    }
                                                    j = j + 1;
                                                }
                                            }
                                        } else if vstart0 == ((*(*line_base.offset(lstart_r24 as isize)).x0).v1_idx as i32) {
                                            let mut j: i32 = 0;
                                            let mut lstart_r20: i32 = 0;
                                            let mut lcount_r17: i16 = 0;
                                            {
                                                j = 0;
                                                'l305: while j < (MapLineGroup_Count as i32) {
                                                    let mut k: i32 = 0;
                                                    lcount_r17 = (*(*j1_r10).inner).ranges[j as usize].count;
                                                    *line_base.offset({ lstart_r20 = (*(*j1_r10).inner).ranges[j as usize].start as i32; lstart_r20 } as isize);
                                                    {
                                                        k = 0;
                                                        'l306: while k < (lcount_r17 as i32) {
                                                            if vid_r26 == ((*(*line_base.offset(lstart_r20 as isize)).x0).v0_idx as i32) {
                                                                (*(*line_base.offset(lstart_r24 as isize)).x0).next_id1 = lstart_r20 as i16;
                                                                (*(*line_base.offset(lstart_r20 as isize)).x0).prev_id1 = lstart_r24 as i16;
                                                            }
                                                            k = k + 1; lstart_r20 = lstart_r20 + 1;
                                                        }
                                                    }
                                                    j = j + 1;
                                                }
                                            }
                                        }
                                        i_r23 = i_r23 + 1; lstart_r24 = lstart_r24 + 1;
                                    }
                                }
                                var_r25 = var_r25 + 1;
                            }
                        }
                    }
                    v = v + 1; vid_r26 = vid_r26 + 1;
                }
            }
            vi = vi + 1; vstart0 = vstart0 + 1;
        }
    }
}

/// `mpLib_80058560`
pub unsafe fn mpLib_80058560() {
    let mut temp_r29: *mut MapCollData = mpLib_804D64B4;
    let mut i: i32 = 0;
    let mut j: i32 = 0;
    let mut cur_i: *mut CollJoint = null_mut();
    let mut cur_j: *mut CollJoint = null_mut();
    {
        i = 0;
        'l307: while i < ((*temp_r29).joint_count - 1) {
            {
                j = i + 1;
                'l308: while j < (*temp_r29).joint_count {
                    cur_i = core::ptr::addr_of_mut!(*groundCollJoint.offset(i as isize)) as *mut CollJoint;
                    cur_j = core::ptr::addr_of_mut!(*groundCollJoint.offset(j as isize)) as *mut CollJoint;
                    if (((((ext::ScriptGame_StageJointActive(i) != 0) && (ext::ScriptGame_StageJointActive(j) != 0)) && (((*cur_i).flags & ((CollJoint_Enabled as i32) as u32)) != 0)) && (!(((*cur_i).flags & ((CollJoint_Hidden as i32) as u32)) != 0))) && (((*cur_j).flags & ((CollJoint_Enabled as i32) as u32)) != 0)) && (!(((*cur_j).flags & ((CollJoint_Hidden as i32) as u32)) != 0)) {
                        mpLib_800581DC(i, j);
                    }
                    j = j + 1;
                }
            }
            i = i + 1;
        }
    }
}

/// `mpLib_80058614_Floor`
pub unsafe fn mpLib_80058614_Floor() {
    let mut jp: *mut CollJoint = null_mut();
    let mut count_r8: i32 = 0;
    let mut count_r5: i32 = 0;
    let mut i: i32 = 0;
    let mut j: i32 = 0;
    let mut linebase: *mut CollLine = null_mut();
    let mut line_r31: *mut CollLine = null_mut();
    let mut count_r30: i32 = 0;
    let mut count_r29: i32 = 0;
    let mut j_inner: *mut MapJoint = null_mut();
    let mut v0: *mut CollVtx = null_mut();
    let mut v1: *mut CollVtx = null_mut();
    let mut top_p: *mut f32 = null_mut();
    let mut bottom_p: *mut f32 = null_mut();
    let mut right_p: *mut f32 = null_mut();
    let mut left_p: *mut f32 = null_mut();
    let mut joint_r7: *mut CollJoint = null_mut();
    jp = { joint_r7 = groundCollJoint; joint_r7 };
    count_r8 = (*mpLib_804D64B4).joint_count;
    {
        count_r5 = 0;
        'l309: while count_r5 < count_r8 {
            'c309: {
                if ext::ScriptGame_StageJoint(count_r5) != 0 {
                    (*jp).set_xE((0 as u8) as u8);
                    break 'c309;
                }
                if (*jp).xE() != 0 {
                    break 'l309;
                }
            }
            count_r5 = count_r5 + 1; jp = jp.offset(1);
        }
    }
    if count_r5 == count_r8 {
        return;
    }
    let __t310 = core::ptr::addr_of_mut!(*{ right_p = core::ptr::addr_of_mut!(mpLib_80458868[1 as usize].right) as *mut f32; right_p }); *__t310 = -f32::MAX;
    let __t311 = core::ptr::addr_of_mut!(*{ top_p = core::ptr::addr_of_mut!(mpLib_80458868[1 as usize].top) as *mut f32; top_p }); *__t311 = -f32::MAX;
    let __t312 = core::ptr::addr_of_mut!(*{ left_p = core::ptr::addr_of_mut!(mpLib_80458868[1 as usize].left) as *mut f32; left_p }); *__t312 = f32::MAX;
    let __t313 = core::ptr::addr_of_mut!(*{ bottom_p = core::ptr::addr_of_mut!(mpLib_80458868[1 as usize].bottom) as *mut f32; bottom_p }); *__t313 = f32::MAX;
    {
        i = 0;
        'l314: while i < count_r8 {
            'c314: {
                (*joint_r7).set_xE((0 as u8) as u8);
                if ext::ScriptGame_StageJoint(i) != 0 {
                    break 'c314;
                }
                if (!(((*joint_r7).flags & ((CollJoint_Enabled as i32) as u32)) != 0)) || (((*joint_r7).flags & ((CollJoint_Hidden as i32) as u32)) != 0) {
                    break 'c314;
                }
                {
                    j_inner = (*joint_r7).inner;
                    count_r29 = (*j_inner).ranges[(MapLineGroup_Dynamic as i32) as usize].count as i32;
                    *groundCollVtx.offset({ count_r30 = (*j_inner).ranges[(MapLineGroup_Floor as i32) as usize].count as i32; count_r30 } as isize);
                    linebase = groundCollLine;
                    line_r31 = core::ptr::addr_of_mut!(*linebase.offset((*j_inner).ranges[(MapLineGroup_Floor as i32) as usize].start as isize)) as *mut CollLine;
                    {
                        j = 0;
                        'p315: loop {
                            'l316: while j < count_r30 {
                                'c316: {
                                    let mut x0: f32 = 0.0;
                                    let mut y0: f32 = 0.0;
                                    let mut x1: f32 = 0.0;
                                    let mut y1: f32 = 0.0;
                                    if (!(((*line_r31).flags & ((CollLine_Floor as i32) as u32)) != 0)) || (!(((*line_r31).flags & ((1 << 16) as u32)) != 0)) {
                                        break 'c316;
                                    }
                                    v0 = core::ptr::addr_of_mut!(*groundCollVtx.offset((*(*line_r31).x0).v0_idx as isize)) as *mut CollVtx;
                                    v1 = core::ptr::addr_of_mut!(*groundCollVtx.offset((*(*line_r31).x0).v1_idx as isize)) as *mut CollVtx;
                                    x0 = (*v0).pos.x;
                                    y0 = (*v0).pos.y;
                                    x1 = (*v1).pos.x;
                                    y1 = (*v1).pos.y;
                                    if (*top_p) < y0 {
                                        *top_p = y0;
                                    }
                                    if (*bottom_p) > y0 {
                                        *bottom_p = y0;
                                    }
                                    if (*right_p) < x0 {
                                        *right_p = x0;
                                    }
                                    if (*left_p) > x0 {
                                        *left_p = x0;
                                    }
                                    if (*top_p) < y1 {
                                        *top_p = y1;
                                    }
                                    if (*bottom_p) > y1 {
                                        *bottom_p = y1;
                                    }
                                    if (*right_p) < x1 {
                                        *right_p = x1;
                                    }
                                    if (*left_p) > x1 {
                                        *left_p = x1;
                                    }
                                }
                                j = j + 1; line_r31 = line_r31.offset(1);
                            }
                            if count_r29 != 0 {
                                count_r30 = count_r29;
                                j = 0;
                                count_r29 = 0;
                                line_r31 = core::ptr::addr_of_mut!(*linebase.offset((*(*joint_r7).inner).ranges[(MapLineGroup_Dynamic as i32) as usize].start as isize)) as *mut CollLine;
                                continue 'p315;
                            }
                            break;
                        }
                    }
                }
            }
            i = i + 1; joint_r7 = joint_r7.offset(1);
        }
    }
}

/// `mpLib_800587FC`
pub unsafe extern "C" fn mpLib_800587FC(mut gobj: *mut HSD_GObj) {
    mpLib_80058614_Floor();
    ext::grDynamicAttr_801CA224();
}

/// `mpLib_80058820`
pub unsafe fn mpLib_80058820() {
    let mut r#box: *mut mpCollisionBox = mpLib_80458868.as_mut_ptr();
    let mut gobj: *mut HSD_GObj = ext::GObj_Create(1 as u16, 6 as u8, 0 as u8);
    if !gobj.is_null() { (); } else { panic!("mpLib_80058820: assertion failed"); }
    let _ = ext::HSD_GObj_SetupProc(gobj, Some(mpLib_800587FC), 4 as u8);
    (*r#box.offset(0 as isize)).right = 1e+04f32;
    (*r#box.offset(0 as isize)).top = 1e+04f32;
    (*r#box.offset(0 as isize)).left = -1e+04f32;
    (*r#box.offset(0 as isize)).bottom = -1e+04f32;
    *r#box.offset(1 as isize) = *r#box.offset(0 as isize);
}

/// `mpCheckedBounding`
pub unsafe fn mpCheckedBounding() -> i32 {
    return didCheckBounding;
}

/// `mpBoundingCheck`
pub unsafe fn mpBoundingCheck(mut left: f32, mut bottom: f32, mut right: f32, mut top: f32) {
    let mut curr: *mut CollJoint = jointListStart;
    'l317: while !curr.is_null() {
        if ((ext::ScriptGame_StageJointActive((curr.offset_from(groundCollJoint) as i64) as i32) != 0) && (((*curr).flags & ((CollJoint_Enabled as i32) as u32)) != 0)) && (!(((*curr).flags & ((CollJoint_Hidden as i32) as u32)) != 0)) {
            if ((*curr).flags & ((CollJoint_B10 as i32) as u32)) != 0 {
                (*curr).flags = (*curr).flags & ((!(CollJoint_TooFar as i32)) as u32);
            } else if (((left > (*curr).bounding_max.x) || (right < (*curr).bounding_min.x)) || (bottom > (*curr).bounding_max.y)) || (top < (*curr).bounding_min.y) {
                (*curr).flags = (*curr).flags | ((CollJoint_TooFar as i32) as u32);
            } else {
                (*curr).flags = (*curr).flags & ((!(CollJoint_TooFar as i32)) as u32);
            }
        } else {
            (*curr).flags = (*curr).flags | ((CollJoint_TooFar as i32) as u32);
        }
        curr = (*curr).next;
    }
    didCheckBounding = 1;
}

/// `mpBoundingCheck2`
pub unsafe fn mpBoundingCheck2(mut x1: f32, mut y1: f32, mut x2: f32, mut y2: f32) {
    let mut right: f32 = 0.0;
    let mut left: f32 = 0.0;
    let mut bottom: f32 = 0.0;
    let mut top: f32 = 0.0;
    if x1 > x2 {
        left = x2;
        right = x1;
    } else {
        right = x2;
        left = x1;
    }
    if y1 > y2 {
        bottom = y2;
        top = y1;
    } else {
        top = y2;
        bottom = y1;
    }
    mpBoundingCheck(left, bottom, right, top);
}

/// `mpBoundingCheck3`
pub unsafe fn mpBoundingCheck3(mut x0: f32, mut y0: f32, mut x1: f32, mut y1: f32, mut x2: f32, mut y2: f32, mut x3: f32, mut y3: f32) {
    let mut right: f32 = 0.0;
    let mut left: f32 = 0.0;
    let mut bottom: f32 = 0.0;
    let mut top: f32 = 0.0;
    left = x1;
    bottom = y1;
    if x0 > left {
        right = x0;
    } else {
        right = left;
        left = x0;
    }
    if y0 > bottom {
        top = y0;
    } else {
        top = bottom;
        bottom = y0;
    }
    if right < x2 {
        right = x2;
    } else if left > x2 {
        left = x2;
    }
    if top < y2 {
        top = y2;
    } else if bottom > y2 {
        bottom = y2;
    }
    if right < x3 {
        right = x3;
    } else if left > x3 {
        left = x3;
    }
    if top < y3 {
        top = y3;
    } else if bottom > y3 {
        bottom = y3;
    }
    mpBoundingCheck(left, bottom, right, top);
}

/// `mpUncheckBounding`
pub unsafe fn mpUncheckBounding() {
    let mut curr: *mut CollJoint = jointListStart;
    'l318: while !curr.is_null() {
        (*curr).flags = (*curr).flags & ((!(CollJoint_TooFar as i32)) as u32);
        curr = (*curr).next;
    }
    didCheckBounding = 0;
}

/// `mpLib_SetupDraw`
pub unsafe fn mpLib_SetupDraw(mut color: GXColor) {
    let mut spC: HSD_TevDesc = unsafe { core::mem::zeroed() };
    ext::HSD_StateInitTev();
    spC.flags = 0 as u32;
    spC.stage = ext::HSD_StateAssignTev() as u32;
    spC.coord = 255 as u32;
    spC.map = 255 as u32;
    spC.color = 4 as u32;
    spC.u.tevop.tevmode = 4 as u32;
    ext::HSD_SetupTevStage(core::ptr::addr_of_mut!(spC) as *mut _HSD_TevDesc);
    ext::HSD_SetupPEMode(1 as u32, null_mut());
    ext::HSD_SetTevRegAll();
    ext::HSD_StateSetNumTevStages();
    ext::HSD_StateSetNumTexGens();
    ext::HSD_StateSetNumChans(1);
    mpLib_803BF540.mat_color = color;
    ext::HSD_SetupChannel(core::ptr::addr_of_mut!(mpLib_803BF540) as *mut HSD_Chan);
}

/// `mpLib_DrawEcbs`
pub unsafe fn mpLib_DrawEcbs(mut cd: *mut CollData) {
    let mut spE0: GXColor = GXColor { r: 255 as u8, g: 160 as u8, b: 48 as u8, a: 255 as u8 };
    let mut spDC: GXColor = GXColor { r: 128 as u8, g: 80 as u8, b: 24 as u8, a: 255 as u8 };
    let mut spD8: GXColor = GXColor { r: 255 as u8, g: 90 as u8, b: 32 as u8, a: 255 as u8 };
    let mut spD4: GXColor = GXColor { r: 128 as u8, g: 45 as u8, b: 16 as u8, a: 255 as u8 };
    let mut spD0: GXColor = GXColor { r: 32 as u8, g: 32 as u8, b: 255 as u8, a: 255 as u8 };
    let mut spCC: GXColor = GXColor { r: 16 as u8, g: 16 as u8, b: 128 as u8, a: 255 as u8 };
    let mut spC8: GXColor = GXColor { r: 255 as u8, g: 255 as u8, b: 255 as u8, a: 128 as u8 };
    let mut spC4: GXColor = GXColor { r: 255 as u8, g: 255 as u8, b: 255 as u8, a: 128 as u8 };
    mpLib_SetupDraw(spE0);
    ext::GXBegin((GX_QUADS as i32) as u32, (GX_VTXFMT0 as i32) as u32, 4 as u16);
    ext::GXPosition3f32((*cd).cur_pos.x + (*cd).ecb.top.x, (*cd).cur_pos.y + (*cd).ecb.top.y, (*cd).cur_pos.z);
    ext::GXPosition3f32((*cd).cur_pos.x + (*cd).ecb.right.x, (*cd).cur_pos.y + (*cd).ecb.right.y, (*cd).cur_pos.z);
    ext::GXPosition3f32((*cd).cur_pos.x + (*cd).ecb.bottom.x, (*cd).cur_pos.y + (*cd).ecb.bottom.y, (*cd).cur_pos.z);
    ext::GXPosition3f32((*cd).cur_pos.x + (*cd).ecb.left.x, (*cd).cur_pos.y + (*cd).ecb.left.y, (*cd).cur_pos.z);
    ext::GXEnd();
    mpLib_SetupDraw(spDC);
    ext::GXBegin((GX_LINES as i32) as u32, (GX_VTXFMT0 as i32) as u32, 4 as u16);
    ext::GXPosition3f32((*cd).cur_pos.x - 1.0f32, (*cd).cur_pos.y, (*cd).cur_pos.z);
    ext::GXPosition3f32(1.0f32 + (*cd).cur_pos.x, (*cd).cur_pos.y, (*cd).cur_pos.z);
    ext::GXPosition3f32((*cd).cur_pos.x, (*cd).cur_pos.y - 1.0f32, (*cd).cur_pos.z);
    ext::GXPosition3f32((*cd).cur_pos.x, 1.0f32 + (*cd).cur_pos.y, (*cd).cur_pos.z);
    ext::GXEnd();
    mpLib_SetupDraw(spD8);
    ext::GXBegin((GX_QUADS as i32) as u32, (GX_VTXFMT0 as i32) as u32, 4 as u16);
    ext::GXPosition3f32((*cd).prev_pos.x + (*cd).prev_ecb.top.x, (*cd).prev_pos.y + (*cd).prev_ecb.top.y, (*cd).prev_pos.z - 0.5f32);
    ext::GXPosition3f32((*cd).prev_pos.x + (*cd).prev_ecb.right.x, (*cd).prev_pos.y + (*cd).prev_ecb.right.y, (*cd).prev_pos.z - 0.5f32);
    ext::GXPosition3f32((*cd).prev_pos.x + (*cd).prev_ecb.bottom.x, (*cd).prev_pos.y + (*cd).prev_ecb.bottom.y, (*cd).prev_pos.z - 0.5f32);
    ext::GXPosition3f32((*cd).prev_pos.x + (*cd).prev_ecb.left.x, (*cd).prev_pos.y + (*cd).prev_ecb.left.y, (*cd).prev_pos.z - 0.5f32);
    ext::GXEnd();
    mpLib_SetupDraw(spD4);
    ext::GXBegin((GX_LINES as i32) as u32, (GX_VTXFMT0 as i32) as u32, 4 as u16);
    ext::GXPosition3f32((*cd).prev_pos.x - 1.0f32, (*cd).prev_pos.y, (*cd).prev_pos.z - 0.5f32);
    ext::GXPosition3f32(1.0f32 + (*cd).prev_pos.x, (*cd).prev_pos.y, (*cd).prev_pos.z - 0.5f32);
    ext::GXPosition3f32((*cd).prev_pos.x, (*cd).prev_pos.y - 1.0f32, (*cd).prev_pos.z - 0.5f32);
    ext::GXPosition3f32((*cd).prev_pos.x, 1.0f32 + (*cd).prev_pos.y, (*cd).prev_pos.z - 0.5f32);
    ext::GXEnd();
    mpLib_SetupDraw(spD0);
    ext::GXBegin((GX_QUADS as i32) as u32, (GX_VTXFMT0 as i32) as u32, 4 as u16);
    ext::GXPosition3f32((*cd).last_pos.x + (*cd).xE4_ecb.top.x, (*cd).last_pos.y + (*cd).xE4_ecb.top.y, (*cd).last_pos.z - 1.0f32);
    ext::GXPosition3f32((*cd).last_pos.x + (*cd).xE4_ecb.right.x, (*cd).last_pos.y + (*cd).xE4_ecb.right.y, (*cd).last_pos.z - 1.0f32);
    ext::GXPosition3f32((*cd).last_pos.x + (*cd).xE4_ecb.bottom.x, (*cd).last_pos.y + (*cd).xE4_ecb.bottom.y, (*cd).last_pos.z - 1.0f32);
    ext::GXPosition3f32((*cd).last_pos.x + (*cd).xE4_ecb.left.x, (*cd).last_pos.y + (*cd).xE4_ecb.left.y, (*cd).last_pos.z - 1.0f32);
    ext::GXEnd();
    mpLib_SetupDraw(spCC);
    ext::GXBegin((GX_LINES as i32) as u32, (GX_VTXFMT0 as i32) as u32, 4 as u16);
    ext::GXPosition3f32((*cd).last_pos.x - 1.0f32, (*cd).last_pos.y, (*cd).last_pos.z - 1.0f32);
    ext::GXPosition3f32(1.0f32 + (*cd).last_pos.x, (*cd).last_pos.y, (*cd).last_pos.z - 1.0f32);
    ext::GXPosition3f32((*cd).last_pos.x, (*cd).last_pos.y - 1.0f32, (*cd).last_pos.z - 1.0f32);
    ext::GXPosition3f32((*cd).last_pos.x, 1.0f32 + (*cd).last_pos.y, (*cd).last_pos.z - 1.0f32);
    ext::GXEnd();
    mpLib_SetupDraw(spC8);
    ext::GXBegin((GX_LINESTRIP as i32) as u32, (GX_VTXFMT0 as i32) as u32, 5 as u16);
    if (*cd).x34_flags.b6() != 0 {
        ext::GXPosition3f32((*cd).x28_vec.x + (*cd).x64_ecb.top.x, (*cd).x28_vec.y + (*cd).x64_ecb.top.y, 0.5f32 + (*cd).x28_vec.z);
        ext::GXPosition3f32((*cd).x28_vec.x + (*cd).x64_ecb.right.x, (*cd).x28_vec.y + (*cd).x64_ecb.right.y, 0.5f32 + (*cd).x28_vec.z);
        ext::GXPosition3f32((*cd).x28_vec.x + (*cd).x64_ecb.bottom.x, (*cd).x28_vec.y + (*cd).x64_ecb.bottom.y, 0.5f32 + (*cd).x28_vec.z);
        ext::GXPosition3f32((*cd).x28_vec.x + (*cd).x64_ecb.left.x, (*cd).x28_vec.y + (*cd).x64_ecb.left.y, 0.5f32 + (*cd).x28_vec.z);
        ext::GXPosition3f32((*cd).x28_vec.x + (*cd).x64_ecb.top.x, (*cd).x28_vec.y + (*cd).x64_ecb.top.y, 0.5f32 + (*cd).x28_vec.z);
    } else {
        ext::GXPosition3f32((*cd).x28_vec.x + (*cd).ecb.top.x, (*cd).x28_vec.y + (*cd).ecb.top.y, 0.5f32 + (*cd).x28_vec.z);
        ext::GXPosition3f32((*cd).x28_vec.x + (*cd).ecb.right.x, (*cd).x28_vec.y + (*cd).ecb.right.y, 0.5f32 + (*cd).x28_vec.z);
        ext::GXPosition3f32((*cd).x28_vec.x + (*cd).ecb.bottom.x, (*cd).x28_vec.y + (*cd).ecb.bottom.y, 0.5f32 + (*cd).x28_vec.z);
        ext::GXPosition3f32((*cd).x28_vec.x + (*cd).ecb.left.x, (*cd).x28_vec.y + (*cd).ecb.left.y, 0.5f32 + (*cd).x28_vec.z);
        ext::GXPosition3f32((*cd).x28_vec.x + (*cd).ecb.top.x, (*cd).x28_vec.y + (*cd).ecb.top.y, 0.5f32 + (*cd).x28_vec.z);
    }
    ext::GXEnd();
    mpLib_SetupDraw(spC4);
    ext::GXBegin((GX_LINES as i32) as u32, (GX_VTXFMT0 as i32) as u32, 4_u32 as u16);
    ext::GXPosition3f32((*cd).x28_vec.x - 1.0f32, (*cd).x28_vec.y, (0.5f64 + ((*cd).x28_vec.z as f64)) as f32);
    ext::GXPosition3f32(1.0f32 + (*cd).x28_vec.x, (*cd).x28_vec.y, (0.5f64 + ((*cd).x28_vec.z as f64)) as f32);
    ext::GXPosition3f32((*cd).x28_vec.x, (*cd).x28_vec.y - 1.0f32, (0.5f64 + ((*cd).x28_vec.z as f64)) as f32);
    ext::GXPosition3f32((*cd).x28_vec.x, 1.0f32 + (*cd).x28_vec.y, (0.5f64 + ((*cd).x28_vec.z as f64)) as f32);
    ext::GXEnd();
}

/// `mpLib_CopyColor`
unsafe fn mpLib_CopyColor(mut color: GXColor) -> GXColor {
    return color;
}

/// `mpLib_DrawSnapping`
pub unsafe fn mpLib_DrawSnapping() {
    let mut left_snap_color: GXColor = mpLib_804D80E0;
    let mut right_snap_color: GXColor = mpLib_804D80E4;
    let mut item_r28: *mut HSD_GObj = null_mut();
    let mut ft_r27: *mut HSD_GObj = null_mut();
    let mut var_r31: i32 = 0;
    var_r31 = 0;
    ft_r27 = *ext::HSD_GObjPLinkHead.offset(8 as isize);
    if !ft_r27.is_null() {
        let mut spDC: [[f32; 4]; 3] = unsafe { core::mem::zeroed() };
        ext::HSD_StateSetCullMode(0);
        ext::GXClearVtxDesc();
        ext::GXSetVtxDesc((GX_VA_POS as i32) as u32, (GX_DIRECT as i32) as u32);
        ext::GXSetVtxAttrFmt((GX_VTXFMT0 as i32) as u32, (GX_VA_POS as i32) as u32, (GX_TEX_ST as i32) as u32, (GX_RGBA6 as i32) as u32, 0_u32 as u8);
        ext::HSD_CObjGetViewingMtx(ext::HSD_CObjGetCurrent(), spDC.as_mut_ptr());
        ext::GXSetCurrentMtx(0_u32);
        ext::GXLoadPosMtxImm(spDC.as_mut_ptr(), 0 as u32);
        var_r31 = 1;
        'l319: while !ft_r27.is_null() {
            'c319: {
                let mut cd: *mut CollData = ext::ftLib_80086984(ft_r27);
                if (!((*cd).x35_flags.b0() != 0)) || ((*cd).x38 != super::mpcoll::mpColl_804D64AC) {
                    break 'c319;
                }
                mpLib_DrawEcbs(cd);
                if (((*cd).ledge_snap_x as f64) != 0.0f64) || (((*cd).ledge_snap_y as f64) != 0.0f64) {
                    let mut inner_x: f32 = (*cd).cur_pos.x + (*cd).ecb.right.x;
                    let mut pos_y: f32 = (*cd).cur_pos.y;
                    let mut f30: f32 = 0.5f32 * (*cd).ledge_snap_height;
                    mpLib_SetupDraw(right_snap_color);
                    ext::GXBegin((GX_LINESTRIP as i32) as u32, (GX_VTXFMT0 as i32) as u32, 5 as u16);
                    ext::GXPosition3f32(inner_x + (*cd).ledge_snap_x, f30 + (pos_y + (*cd).ledge_snap_y), (*cd).cur_pos.z);
                    ext::GXPosition3f32(inner_x + (*cd).ledge_snap_x, (pos_y + (*cd).ledge_snap_y) - f30, (*cd).cur_pos.z);
                    ext::GXPosition3f32((*cd).cur_pos.x, (pos_y + (*cd).ledge_snap_y) - f30, (*cd).cur_pos.z);
                    ext::GXPosition3f32((*cd).cur_pos.x, f30 + (pos_y + (*cd).ledge_snap_y), (*cd).cur_pos.z);
                    ext::GXPosition3f32(inner_x + (*cd).ledge_snap_x, f30 + (pos_y + (*cd).ledge_snap_y), (*cd).cur_pos.z);
                    ext::GXEnd();
                    pos_y = (*cd).cur_pos.y;
                    inner_x = (*cd).cur_pos.x + (*cd).ecb.left.x;
                    mpLib_SetupDraw(mpLib_CopyColor(left_snap_color));
                    ext::GXBegin((GX_LINESTRIP as i32) as u32, (GX_VTXFMT0 as i32) as u32, 5 as u16);
                    ext::GXPosition3f32(inner_x - (*cd).ledge_snap_x, f30 + (pos_y + (*cd).ledge_snap_y), (*cd).cur_pos.z);
                    ext::GXPosition3f32(inner_x - (*cd).ledge_snap_x, (pos_y + (*cd).ledge_snap_y) - f30, (*cd).cur_pos.z);
                    ext::GXPosition3f32((*cd).cur_pos.x, (pos_y + (*cd).ledge_snap_y) - f30, (*cd).cur_pos.z);
                    ext::GXPosition3f32((*cd).cur_pos.x, f30 + (pos_y + (*cd).ledge_snap_y), (*cd).cur_pos.z);
                    ext::GXPosition3f32(inner_x - (*cd).ledge_snap_x, f30 + (pos_y + (*cd).ledge_snap_y), (*cd).cur_pos.z);
                    ext::GXEnd();
                }
            }
            ft_r27 = (*ft_r27).next;
        }
    }
    if !{ item_r28 = *ext::HSD_GObjPLinkHead.offset(9 as isize); item_r28 }.is_null() {
        if !(var_r31 != 0) {
            let mut sp7C: [[f32; 4]; 3] = unsafe { core::mem::zeroed() };
            ext::GXSetCullMode((GX_CULL_NONE as i32) as u32);
            ext::GXClearVtxDesc();
            ext::GXSetVtxDesc((GX_VA_POS as i32) as u32, (GX_DIRECT as i32) as u32);
            ext::GXSetVtxAttrFmt((GX_VTXFMT0 as i32) as u32, (GX_VA_POS as i32) as u32, (GX_TEX_ST as i32) as u32, (GX_RGBA6 as i32) as u32, 0 as u8);
            ext::HSD_CObjGetViewingMtx(ext::HSD_CObjGetCurrent(), sp7C.as_mut_ptr());
            ext::GXSetCurrentMtx(0 as u32);
            ext::GXLoadPosMtxImm(sp7C.as_mut_ptr(), 0 as u32);
        }
        'l320: while !item_r28.is_null() {
            'c320: {
                let mut cd: *mut CollData = core::ptr::addr_of_mut!((*(HSD_GObjGetUserData(item_r28) as *mut Item)).x378_itemColl) as *mut CollData;
                if !((*cd).x35_flags.b0() != 0) {
                    break 'c320;
                }
                if (*cd).x38 == super::mpcoll::mpColl_804D64AC {
                    mpLib_DrawEcbs(cd);
                }
                if ext::itGetKind(item_r28) == It_Kind_Link_HShot {
                    mpLib_DrawEcbs(core::ptr::addr_of_mut!((*(*(HSD_GObjGetUserData(item_r28) as *mut Item)).xDD4_itemVar.linkhookshot.x0).coll_data) as *mut CollData);
                }
            }
            item_r28 = (*item_r28).next;
        }
    }
}

/// `mpLib_DrawMatchingLines`
unsafe fn mpLib_DrawMatchingLines(mut value: i32, mut flag: i32, mut color: *const _GXColor) {
    let mut line_r31: *mut CollLine = null_mut();
    let mut count_r28: i32 = 0;
    let mut total_r27: i32 = 0;
    let mut line_r6: *mut CollLine = null_mut();
    let mut i: i32 = 0;
    count_r28 = 0;
    total_r27 = (*mpLib_804D64B4).line_count;
    line_r6 = groundCollLine;
    {
        i = 0;
        'l321: while i < total_r27 {
            if ((((*line_r6).flags & ((1 << 16) as u32)) != 0) && (!(((*line_r6).flags & ((1 << 18) as u32)) != 0))) && (value == (((*(*line_r6).x0).lo_flags as i32) & flag)) {
                count_r28 = count_r28 + 1;
            }
            line_r6 = line_r6.offset(1 as isize);
            i = i + 1;
        }
    }
    if count_r28 == 0 {
        return;
    }
    line_r31 = groundCollLine;
    mpLib_SetupDraw(*color);
    ext::GXBegin((GX_QUADS as i32) as u32, (GX_VTXFMT0 as i32) as u32, (count_r28 * 4) as u16);
    {
        i = 0;
        'l322: while i < total_r27 {
            if (((*line_r31).flags & ((1 << 16) as u32)) != 0) && (!(((*line_r31).flags & ((1 << 18) as u32)) != 0)) {
                if value == (((*(*line_r31).x0).lo_flags as i32) & flag) {
                    ext::GXPosition3f32((*groundCollVtx.offset((*(*line_r31).x0).v0_idx as isize)).pos.x, (*groundCollVtx.offset((*(*line_r31).x0).v0_idx as isize)).pos.y, 25.0f32);
                    ext::GXPosition3f32((*groundCollVtx.offset((*(*line_r31).x0).v1_idx as isize)).pos.x, (*groundCollVtx.offset((*(*line_r31).x0).v1_idx as isize)).pos.y, 25.0f32);
                    ext::GXPosition3f32((*groundCollVtx.offset((*(*line_r31).x0).v1_idx as isize)).pos.x, (*groundCollVtx.offset((*(*line_r31).x0).v1_idx as isize)).pos.y, -25.0f32);
                    ext::GXPosition3f32((*groundCollVtx.offset((*(*line_r31).x0).v0_idx as isize)).pos.x, (*groundCollVtx.offset((*(*line_r31).x0).v0_idx as isize)).pos.y, -25.0f32);
                }
            }
            line_r31 = line_r31.offset(1 as isize);
            i = i + 1;
        }
    }
    ext::GXEnd();
}

/// `mpLib_80059554`
pub unsafe fn mpLib_80059554() {
    let mut line_r7: *mut CollLine = null_mut();
    let mut line_r4: *mut CollLine = null_mut();
    let mut v0_r6: *mut CollVtx = null_mut();
    let mut v1_r4: *mut CollVtx = null_mut();
    let mut total_r29: i32 = 0;
    let mut count_r30: i32 = 0;
    let mut coll_data_r31: *mut MapCollData = null_mut();
    let mut i: i32 = 0;
    let mut pad: [u8; 8] = [0; 8];
    let mut spB4: GXColor = unsafe { core::mem::zeroed() };
    let mut spB0: GXColor = unsafe { core::mem::zeroed() };
    let mut spAC: GXColor = unsafe { core::mem::zeroed() };
    let mut spA8: GXColor = unsafe { core::mem::zeroed() };
    coll_data_r31 = mpLib_804D64B4;
    total_r29 = 0;
    count_r30 = (*coll_data_r31).ranges[(MapLineGroup_Floor as i32) as usize].count as i32;
    line_r4 = core::ptr::addr_of_mut!(*groundCollLine.offset((*coll_data_r31).ranges[(MapLineGroup_Floor as i32) as usize].start as isize)) as *mut CollLine;
    {
        i = 0;
        'l323: while i < count_r30 {
            if (((*line_r4).flags & ((1 << 16) as u32)) != 0) && (!(((*line_r4).flags & ((1 << 18) as u32)) != 0)) {
                total_r29 = total_r29 + 1;
            }
            line_r4 = line_r4.offset(1);
            i = i + 1;
        }
    }
    if total_r29 != 0 {
        mpLib_SetupDraw(mpLib_FloorColor);
        ext::GXBegin((GX_QUADS as i32) as u32, (GX_VTXFMT0 as i32) as u32, ((total_r29 * 4) & 65532) as u16);
        line_r7 = core::ptr::addr_of_mut!(*groundCollLine.offset((*coll_data_r31).ranges[(MapLineGroup_Floor as i32) as usize].start as isize)) as *mut CollLine;
        {
            i = 0;
            'l324: while i < count_r30 {
                if (((*line_r7).flags & ((1 << 16) as u32)) != 0) && (!(((*line_r7).flags & ((1 << 18) as u32)) != 0)) {
                    ext::GXPosition3f32((*groundCollVtx.offset((*(*line_r7).x0).v0_idx as isize)).pos.x, (*groundCollVtx.offset((*(*line_r7).x0).v0_idx as isize)).pos.y, 25.0f32);
                    ext::GXPosition3f32((*groundCollVtx.offset((*(*line_r7).x0).v1_idx as isize)).pos.x, (*groundCollVtx.offset((*(*line_r7).x0).v1_idx as isize)).pos.y, 25.0f32);
                    ext::GXPosition3f32((*groundCollVtx.offset((*(*line_r7).x0).v1_idx as isize)).pos.x, (*groundCollVtx.offset((*(*line_r7).x0).v1_idx as isize)).pos.y, -25.0f32);
                    ext::GXPosition3f32((*groundCollVtx.offset((*(*line_r7).x0).v0_idx as isize)).pos.x, (*groundCollVtx.offset((*(*line_r7).x0).v0_idx as isize)).pos.y, -25.0f32);
                }
                line_r7 = line_r7.offset(1);
                i = i + 1;
            }
        }
        ext::GXEnd();
    }
    total_r29 = 0;
    count_r30 = (*coll_data_r31).ranges[(MapLineGroup_Ceiling as i32) as usize].count as i32;
    line_r4 = core::ptr::addr_of_mut!(*groundCollLine.offset((*coll_data_r31).ranges[(MapLineGroup_Ceiling as i32) as usize].start as isize)) as *mut CollLine;
    {
        i = 0;
        'l325: while i < count_r30 {
            if (((*line_r4).flags & ((1 << 16) as u32)) != 0) && (!(((*line_r4).flags & ((1 << 18) as u32)) != 0)) {
                total_r29 = total_r29 + 1;
            }
            line_r4 = line_r4.offset(1 as isize);
            i = i + 1;
        }
    }
    if total_r29 != 0 {
        mpLib_SetupDraw(mpLib_CeilingColor);
        ext::GXBegin((GX_QUADS as i32) as u32, (GX_VTXFMT0 as i32) as u32, ((total_r29 * 4) & 65532) as u16);
        line_r7 = core::ptr::addr_of_mut!(*groundCollLine.offset((*coll_data_r31).ranges[(MapLineGroup_Ceiling as i32) as usize].start as isize)) as *mut CollLine;
        {
            i = 0;
            'l326: while i < count_r30 {
                if (((*line_r7).flags & ((1 << 16) as u32)) != 0) && (!(((*line_r7).flags & ((1 << 18) as u32)) != 0)) {
                    ext::GXPosition3f32((*groundCollVtx.offset((*(*line_r7).x0).v0_idx as isize)).pos.x, (*groundCollVtx.offset((*(*line_r7).x0).v0_idx as isize)).pos.y, 25.0f32);
                    ext::GXPosition3f32((*groundCollVtx.offset((*(*line_r7).x0).v1_idx as isize)).pos.x, (*groundCollVtx.offset((*(*line_r7).x0).v1_idx as isize)).pos.y, 25.0f32);
                    ext::GXPosition3f32((*groundCollVtx.offset((*(*line_r7).x0).v1_idx as isize)).pos.x, (*groundCollVtx.offset((*(*line_r7).x0).v1_idx as isize)).pos.y, -25.0f32);
                    ext::GXPosition3f32((*groundCollVtx.offset((*(*line_r7).x0).v0_idx as isize)).pos.x, (*groundCollVtx.offset((*(*line_r7).x0).v0_idx as isize)).pos.y, -25.0f32);
                }
                line_r7 = line_r7.offset(1);
                i = i + 1;
            }
        }
        ext::GXEnd();
    }
    total_r29 = 0;
    count_r30 = (*coll_data_r31).ranges[(MapLineGroup_RightWall as i32) as usize].count as i32;
    line_r4 = core::ptr::addr_of_mut!(*groundCollLine.offset((*coll_data_r31).ranges[(MapLineGroup_RightWall as i32) as usize].start as isize)) as *mut CollLine;
    {
        i = 0;
        'l327: while i < count_r30 {
            if (((*line_r4).flags & ((1 << 16) as u32)) != 0) && (!(((*line_r4).flags & ((1 << 18) as u32)) != 0)) {
                total_r29 = total_r29 + 1;
            }
            line_r4 = line_r4.offset(1 as isize);
            i = i + 1;
        }
    }
    if total_r29 != 0 {
        mpLib_SetupDraw(mpLib_RightWallColor);
        ext::GXBegin((GX_QUADS as i32) as u32, (GX_VTXFMT0 as i32) as u32, ((total_r29 * 4) & 65532) as u16);
        line_r7 = core::ptr::addr_of_mut!(*groundCollLine.offset((*coll_data_r31).ranges[(MapLineGroup_RightWall as i32) as usize].start as isize)) as *mut CollLine;
        {
            i = 0;
            'l328: while i < count_r30 {
                if (((*line_r7).flags & ((1 << 16) as u32)) != 0) && (!(((*line_r7).flags & ((1 << 18) as u32)) != 0)) {
                    ext::GXPosition3f32((*groundCollVtx.offset((*(*line_r7).x0).v0_idx as isize)).pos.x, (*groundCollVtx.offset((*(*line_r7).x0).v0_idx as isize)).pos.y, 25.0f32);
                    ext::GXPosition3f32((*groundCollVtx.offset((*(*line_r7).x0).v1_idx as isize)).pos.x, (*groundCollVtx.offset((*(*line_r7).x0).v1_idx as isize)).pos.y, 25.0f32);
                    ext::GXPosition3f32((*groundCollVtx.offset((*(*line_r7).x0).v1_idx as isize)).pos.x, (*groundCollVtx.offset((*(*line_r7).x0).v1_idx as isize)).pos.y, -25.0f32);
                    ext::GXPosition3f32((*groundCollVtx.offset((*(*line_r7).x0).v0_idx as isize)).pos.x, (*groundCollVtx.offset((*(*line_r7).x0).v0_idx as isize)).pos.y, -25.0f32);
                }
                line_r7 = line_r7.offset(1);
                i = i + 1;
            }
        }
        ext::GXEnd();
    }
    total_r29 = 0;
    count_r30 = (*coll_data_r31).ranges[(MapLineGroup_LeftWall as i32) as usize].count as i32;
    line_r4 = core::ptr::addr_of_mut!(*groundCollLine.offset((*coll_data_r31).ranges[(MapLineGroup_LeftWall as i32) as usize].start as isize)) as *mut CollLine;
    {
        i = 0;
        'l329: while i < count_r30 {
            if (((*line_r4).flags & ((1 << 16) as u32)) != 0) && (!(((*line_r4).flags & ((1 << 18) as u32)) != 0)) {
                total_r29 = total_r29 + 1;
            }
            line_r4 = line_r4.offset(1 as isize);
            i = i + 1;
        }
    }
    if total_r29 != 0 {
        mpLib_SetupDraw(mpLib_LeftWallColor);
        ext::GXBegin((GX_QUADS as i32) as u32, (GX_VTXFMT0 as i32) as u32, ((total_r29 * 4) & 65532) as u16);
        line_r7 = core::ptr::addr_of_mut!(*groundCollLine.offset((*coll_data_r31).ranges[(MapLineGroup_LeftWall as i32) as usize].start as isize)) as *mut CollLine;
        {
            i = 0;
            'l330: while i < count_r30 {
                if (((*line_r7).flags & ((1 << 16) as u32)) != 0) && (!(((*line_r7).flags & ((1 << 18) as u32)) != 0)) {
                    ext::GXPosition3f32((*groundCollVtx.offset((*(*line_r7).x0).v0_idx as isize)).pos.x, (*groundCollVtx.offset((*(*line_r7).x0).v0_idx as isize)).pos.y, 25.0f32);
                    ext::GXPosition3f32((*groundCollVtx.offset((*(*line_r7).x0).v1_idx as isize)).pos.x, (*groundCollVtx.offset((*(*line_r7).x0).v1_idx as isize)).pos.y, 25.0f32);
                    ext::GXPosition3f32((*groundCollVtx.offset((*(*line_r7).x0).v1_idx as isize)).pos.x, (*groundCollVtx.offset((*(*line_r7).x0).v1_idx as isize)).pos.y, -25.0f32);
                    ext::GXPosition3f32((*groundCollVtx.offset((*(*line_r7).x0).v0_idx as isize)).pos.x, (*groundCollVtx.offset((*(*line_r7).x0).v0_idx as isize)).pos.y, -25.0f32);
                }
                line_r7 = line_r7.offset(1);
                i = i + 1;
            }
        }
        ext::GXEnd();
    }
    count_r30 = (*coll_data_r31).ranges[(MapLineGroup_Dynamic as i32) as usize].count as i32;
    total_r29 = 0;
    spB4 = mpLib_DynamicFloorColor;
    spB0 = mpLib_DynamicCeilingColor;
    spAC = mpLib_DynamicRightWallColor;
    spA8 = mpLib_DynamicLeftWallColor;
    line_r4 = core::ptr::addr_of_mut!(*groundCollLine.offset((*coll_data_r31).ranges[(MapLineGroup_Dynamic as i32) as usize].start as isize)) as *mut CollLine;
    {
        i = 0;
        'l331: while i < count_r30 {
            if ((((*line_r4).flags & ((1 << 16) as u32)) != 0) && (((*line_r4).flags & ((CollLine_Floor as i32) as u32)) != 0)) && (!(((*line_r4).flags & ((1 << 18) as u32)) != 0)) {
                total_r29 = total_r29 + 1;
            }
            line_r4 = line_r4.offset(1 as isize);
            i = i + 1;
        }
    }
    if total_r29 != 0 {
        mpLib_SetupDraw(spB4);
        ext::GXBegin((GX_QUADS as i32) as u32, (GX_VTXFMT0 as i32) as u32, ((total_r29 * 4) & 65532) as u16);
        line_r7 = core::ptr::addr_of_mut!(*groundCollLine.offset((*coll_data_r31).ranges[(MapLineGroup_Dynamic as i32) as usize].start as isize)) as *mut CollLine;
        {
            i = 0;
            'l332: while i < count_r30 {
                if ((((*line_r7).flags & ((1 << 16) as u32)) != 0) && (((*line_r7).flags & ((CollLine_Floor as i32) as u32)) != 0)) && (!(((*line_r7).flags & ((1 << 18) as u32)) != 0)) {
                    ext::GXPosition3f32((*groundCollVtx.offset((*(*line_r7).x0).v0_idx as isize)).pos.x, (*groundCollVtx.offset((*(*line_r7).x0).v0_idx as isize)).pos.y, 25.0f32);
                    ext::GXPosition3f32((*groundCollVtx.offset((*(*line_r7).x0).v1_idx as isize)).pos.x, (*groundCollVtx.offset((*(*line_r7).x0).v1_idx as isize)).pos.y, 25.0f32);
                    ext::GXPosition3f32((*groundCollVtx.offset((*(*line_r7).x0).v1_idx as isize)).pos.x, (*groundCollVtx.offset((*(*line_r7).x0).v1_idx as isize)).pos.y, -25.0f32);
                    ext::GXPosition3f32((*groundCollVtx.offset((*(*line_r7).x0).v0_idx as isize)).pos.x, (*groundCollVtx.offset((*(*line_r7).x0).v0_idx as isize)).pos.y, -25.0f32);
                }
                line_r7 = line_r7.offset(1);
                i = i + 1;
            }
        }
        ext::GXEnd();
    }
    line_r4 = core::ptr::addr_of_mut!(*groundCollLine.offset((*coll_data_r31).ranges[(MapLineGroup_Dynamic as i32) as usize].start as isize)) as *mut CollLine;
    total_r29 = 0;
    {
        i = 0;
        'l333: while i < count_r30 {
            if ((((*line_r4).flags & ((1 << 16) as u32)) != 0) && (((*line_r4).flags & ((CollLine_Ceiling as i32) as u32)) != 0)) && (!(((*line_r4).flags & ((1 << 18) as u32)) != 0)) {
                total_r29 = total_r29 + 1;
            }
            line_r4 = line_r4.offset(1 as isize);
            i = i + 1;
        }
    }
    if total_r29 != 0 {
        mpLib_SetupDraw(spB0);
        ext::GXBegin((GX_QUADS as i32) as u32, (GX_VTXFMT0 as i32) as u32, ((total_r29 * 4) & 65532) as u16);
        line_r7 = core::ptr::addr_of_mut!(*groundCollLine.offset((*coll_data_r31).ranges[(MapLineGroup_Dynamic as i32) as usize].start as isize)) as *mut CollLine;
        {
            i = 0;
            'l334: while i < count_r30 {
                if ((((*line_r7).flags & ((1 << 16) as u32)) != 0) && (((*line_r7).flags & ((CollLine_Ceiling as i32) as u32)) != 0)) && (!(((*line_r7).flags & ((1 << 18) as u32)) != 0)) {
                    ext::GXPosition3f32((*groundCollVtx.offset((*(*line_r7).x0).v0_idx as isize)).pos.x, (*groundCollVtx.offset((*(*line_r7).x0).v0_idx as isize)).pos.y, 25.0f32);
                    ext::GXPosition3f32((*groundCollVtx.offset((*(*line_r7).x0).v1_idx as isize)).pos.x, (*groundCollVtx.offset((*(*line_r7).x0).v1_idx as isize)).pos.y, 25.0f32);
                    ext::GXPosition3f32((*groundCollVtx.offset((*(*line_r7).x0).v1_idx as isize)).pos.x, (*groundCollVtx.offset((*(*line_r7).x0).v1_idx as isize)).pos.y, -25.0f32);
                    ext::GXPosition3f32((*groundCollVtx.offset((*(*line_r7).x0).v0_idx as isize)).pos.x, (*groundCollVtx.offset((*(*line_r7).x0).v0_idx as isize)).pos.y, -25.0f32);
                }
                line_r7 = line_r7.offset(1);
                i = i + 1;
            }
        }
        ext::GXEnd();
    }
    line_r4 = core::ptr::addr_of_mut!(*groundCollLine.offset((*coll_data_r31).ranges[(MapLineGroup_Dynamic as i32) as usize].start as isize)) as *mut CollLine;
    total_r29 = 0;
    {
        i = 0;
        'l335: while i < count_r30 {
            if ((((*line_r4).flags & ((1 << 16) as u32)) != 0) && (((*line_r4).flags & ((CollLine_RightWall as i32) as u32)) != 0)) && (!(((*line_r4).flags & ((1 << 18) as u32)) != 0)) {
                total_r29 = total_r29 + 1;
            }
            line_r4 = line_r4.offset(1 as isize);
            i = i + 1;
        }
    }
    if total_r29 != 0 {
        mpLib_SetupDraw(spAC);
        ext::GXBegin((GX_QUADS as i32) as u32, (GX_VTXFMT0 as i32) as u32, ((total_r29 * 4) & 65532) as u16);
        line_r7 = core::ptr::addr_of_mut!(*groundCollLine.offset((*coll_data_r31).ranges[(MapLineGroup_Dynamic as i32) as usize].start as isize)) as *mut CollLine;
        {
            i = 0;
            'l336: while i < count_r30 {
                if ((((*line_r7).flags & ((1 << 16) as u32)) != 0) && (((*line_r7).flags & ((CollLine_RightWall as i32) as u32)) != 0)) && (!(((*line_r7).flags & ((1 << 18) as u32)) != 0)) {
                    ext::GXPosition3f32((*groundCollVtx.offset((*(*line_r7).x0).v0_idx as isize)).pos.x, (*groundCollVtx.offset((*(*line_r7).x0).v0_idx as isize)).pos.y, 25.0f32);
                    ext::GXPosition3f32((*groundCollVtx.offset((*(*line_r7).x0).v1_idx as isize)).pos.x, (*groundCollVtx.offset((*(*line_r7).x0).v1_idx as isize)).pos.y, 25.0f32);
                    ext::GXPosition3f32((*groundCollVtx.offset((*(*line_r7).x0).v1_idx as isize)).pos.x, (*groundCollVtx.offset((*(*line_r7).x0).v1_idx as isize)).pos.y, -25.0f32);
                    ext::GXPosition3f32((*groundCollVtx.offset((*(*line_r7).x0).v0_idx as isize)).pos.x, (*groundCollVtx.offset((*(*line_r7).x0).v0_idx as isize)).pos.y, -25.0f32);
                }
                line_r7 = line_r7.offset(1);
                i = i + 1;
            }
        }
        ext::GXEnd();
    }
    line_r4 = core::ptr::addr_of_mut!(*groundCollLine.offset((*coll_data_r31).ranges[(MapLineGroup_Dynamic as i32) as usize].start as isize)) as *mut CollLine;
    total_r29 = 0;
    {
        i = 0;
        'l337: while i < count_r30 {
            if ((((*line_r4).flags & ((1 << 16) as u32)) != 0) && (((*line_r4).flags & ((CollLine_LeftWall as i32) as u32)) != 0)) && (!(((*line_r4).flags & ((1 << 18) as u32)) != 0)) {
                total_r29 = total_r29 + 1;
            }
            line_r4 = line_r4.offset(1 as isize);
            i = i + 1;
        }
    }
    if total_r29 != 0 {
        mpLib_SetupDraw(spA8);
        ext::GXBegin((GX_QUADS as i32) as u32, (GX_VTXFMT0 as i32) as u32, ((total_r29 * 4) & 65532) as u16);
        line_r7 = core::ptr::addr_of_mut!(*groundCollLine.offset((*coll_data_r31).ranges[(MapLineGroup_Dynamic as i32) as usize].start as isize)) as *mut CollLine;
        {
            i = 0;
            'l338: while i < count_r30 {
                if ((((*line_r7).flags & ((1 << 16) as u32)) != 0) && (((*line_r7).flags & ((CollLine_LeftWall as i32) as u32)) != 0)) && (!(((*line_r7).flags & ((1 << 18) as u32)) != 0)) {
                    ext::GXPosition3f32((*groundCollVtx.offset((*(*line_r7).x0).v0_idx as isize)).pos.x, (*groundCollVtx.offset((*(*line_r7).x0).v0_idx as isize)).pos.y, 25.0f32);
                    ext::GXPosition3f32((*groundCollVtx.offset((*(*line_r7).x0).v1_idx as isize)).pos.x, (*groundCollVtx.offset((*(*line_r7).x0).v1_idx as isize)).pos.y, 25.0f32);
                    ext::GXPosition3f32((*groundCollVtx.offset((*(*line_r7).x0).v1_idx as isize)).pos.x, (*groundCollVtx.offset((*(*line_r7).x0).v1_idx as isize)).pos.y, -25.0f32);
                    ext::GXPosition3f32((*groundCollVtx.offset((*(*line_r7).x0).v0_idx as isize)).pos.x, (*groundCollVtx.offset((*(*line_r7).x0).v0_idx as isize)).pos.y, -25.0f32);
                }
                line_r7 = line_r7.offset(1);
                i = i + 1;
            }
        }
        ext::GXEnd();
    }
}

/// `mpLib_80059E60`
pub unsafe fn mpLib_80059E60() {
    let mut sp104: [[f32; 4]; 3] = unsafe { core::mem::zeroed() };
    ext::HSD_LObjSetupInit(ext::HSD_CObjGetCurrent());
    ext::GXSetCullMode((GX_CULL_NONE as i32) as u32);
    ext::GXClearVtxDesc();
    ext::GXSetVtxDesc((GX_VA_POS as i32) as u32, (GX_DIRECT as i32) as u32);
    ext::GXSetVtxAttrFmt((GX_VTXFMT0 as i32) as u32, (GX_VA_POS as i32) as u32, (GX_TEX_ST as i32) as u32, (GX_RGBA6 as i32) as u32, 0_u32 as u8);
    ext::HSD_CObjGetViewingMtx(ext::HSD_CObjGetCurrent(), sp104.as_mut_ptr());
    ext::GXSetCurrentMtx(0_u32);
    ext::GXLoadPosMtxImm(sp104.as_mut_ptr(), 0_u32);
    if ext::Camera_80030B50() != 0 {
        #[derive(Clone, Copy)] #[repr(C)] struct Local11 { palette: mpIsland_Palette, pad: [u8; 4], line_color: GXColor, basic_color: GXColor }
        let mut sp28: Local11 = unsafe { core::mem::zeroed() };
        let mut line_color: *mut _GXColor = null_mut();
        let mut entry: *mut mpIsland_PaletteEntry = null_mut();
        sp28.palette = super::mpisland::mpIsland_TerrainPalette;
        entry = sp28.palette.x0.as_mut_ptr();
        line_color = core::ptr::addr_of_mut!(sp28.line_color) as *mut _GXColor;
        'l339: while (*entry).kind != ((-1_i32) as u32) {
            *line_color = (*entry).color;
            mpLib_DrawMatchingLines((*entry).kind as i32, 255, line_color as *const _GXColor);
            entry = entry.offset(1);
        }
        sp28.basic_color = mpLib_804D8100;
        mpLib_DrawMatchingLines(mp_Terrain_Basic as i32, 255, (core::ptr::addr_of_mut!(sp28.basic_color) as *mut _GXColor) as *const _GXColor);
    } else if ext::Camera_80030B7C() != 0 {
        #[derive(Clone, Copy)] #[repr(C)] struct Local11 { pad: [u8; 16], ledge: GXColor, platform: GXColor, ledge_platform: GXColor, none: GXColor }
        let mut colors: Local11 = unsafe { core::mem::zeroed() };
        colors.ledge = mpLib_804D80F0;
        mpLib_DrawMatchingLines(1 << 9, 1 << 9, (core::ptr::addr_of_mut!(colors.ledge) as *mut _GXColor) as *const _GXColor);
        colors.platform = mpLib_804D80F4;
        mpLib_DrawMatchingLines(1 << 8, 1 << 8, (core::ptr::addr_of_mut!(colors.platform) as *mut _GXColor) as *const _GXColor);
        colors.ledge_platform = mpLib_804D80F8;
        mpLib_DrawMatchingLines((1 << 9) | (1 << 8), (1 << 9) | (1 << 8), (core::ptr::addr_of_mut!(colors.ledge_platform) as *mut _GXColor) as *const _GXColor);
        colors.none = mpLib_804D80FC;
        mpLib_DrawMatchingLines(0, (1 << 9) | (1 << 8), (core::ptr::addr_of_mut!(colors.none) as *mut _GXColor) as *const _GXColor);
    } else {
        mpLib_80059554();
    }
}

/// `mpLib_DrawCrosses`
pub unsafe fn mpLib_DrawCrosses(mut idx: *mut i16, mut len: i32, mut arg2: GXColor) {
    let mut sp34: Vec3 = unsafe { core::mem::zeroed() };
    let mut i: i32 = 0;
    let mut vtx: *mut Vec = null_mut();
    let mut idx_i: i32 = 0;
    let mut out_count: i32 = 0;
    {
        idx_i = 0; out_count = 0;
        'l340: while (idx_i < len) && (out_count < (128_u64 as i32)) {
            if ext::Ground_801C2D24((*idx.offset(idx_i as isize)) as i32, core::ptr::addr_of_mut!(sp34) as *mut Vec) != 0 {
                mpLib_80458888[out_count as usize] = sp34;
                out_count = out_count + 1;
            }
            idx_i = idx_i + 1;
        }
    }
    if !(out_count != 0) {
        return;
    }
    mpLib_SetupDraw(arg2);
    ext::GXBegin((GX_LINES as i32) as u32, (GX_VTXFMT0 as i32) as u32, (out_count * 6) as u16);
    {
        i = 0;
        'l341: while i < out_count {
            ext::GXPosition3f32(mpLib_80458888[i as usize].x - 3.0f32, mpLib_80458888[i as usize].y, mpLib_80458888[i as usize].z);
            ext::GXPosition3f32(3.0f32 + mpLib_80458888[i as usize].x, mpLib_80458888[i as usize].y, mpLib_80458888[i as usize].z);
            ext::GXPosition3f32(mpLib_80458888[i as usize].x, mpLib_80458888[i as usize].y - 3.0f32, mpLib_80458888[i as usize].z);
            ext::GXPosition3f32(mpLib_80458888[i as usize].x, 3.0f32 + mpLib_80458888[i as usize].y, mpLib_80458888[i as usize].z);
            ext::GXPosition3f32(mpLib_80458888[i as usize].x, mpLib_80458888[i as usize].y, mpLib_80458888[i as usize].z - 3.0f32);
            ext::GXPosition3f32(mpLib_80458888[i as usize].x, mpLib_80458888[i as usize].y, 3.0f32 + mpLib_80458888[i as usize].z);
            i = i + 1;
        }
    }
    ext::GXEnd();
}

/// `mpLib_DrawSpecialPoints`
pub unsafe fn mpLib_DrawSpecialPoints() {
    mpLib_DrawCrosses(mpLib_SpawnVtxIds.as_mut_ptr(), 4, mpLib_804D8128);
    mpLib_DrawCrosses(mpLib_RespawnVtxIds.as_mut_ptr(), 4, mpLib_804D812C);
    mpLib_DrawCrosses(mpLib_ItemSpawnVtxIds.as_mut_ptr(), 21, mpLib_804D8130);
    mpLib_DrawCrosses(core::ptr::addr_of_mut!(mpLib_ItemSpawnVtxIds[22 as usize]) as *mut i16, 80, mpLib_804D8134);
    mpLib_DrawCrosses(core::ptr::addr_of_mut!(mpLib_ItemSpawnVtxIds[102 as usize]) as *mut i16, 25, mpLib_804D8138);
    mpLib_DrawCrosses(core::ptr::addr_of_mut!(mpLib_ItemSpawnVtxIds[128 as usize]) as *mut i16, 46, mpLib_804D813C);
}

/// `mpLib_8005A2DC`
pub unsafe fn mpLib_8005A2DC() {
    mpLib_DrawSnapping();
    mpLib_80059E60();
    if mpLib_804D64D0 == 0 {
        mpLib_804D64D0 = 1;
        ();
    }
    ext::HSD_StateInvalidate((-1_i32));
}

/// `mpLib_DrawZones`
pub unsafe fn mpLib_DrawZones() {
    let mut _3: [u8; 4] = [0; 4];
    let mut sp7C: [[f32; 4]; 3] = unsafe { core::mem::zeroed() };
    let mut _2: [u8; 56] = [0; 56];
    let mut sp40: GXColor = unsafe { core::mem::zeroed() };
    let mut _5: [u8; 8] = [0; 8];
    let mut sp34: GXColor = unsafe { core::mem::zeroed() };
    let mut sp20: GXColor = unsafe { core::mem::zeroed() };
    let mut var_r30: *mut CmSubject = null_mut();
    let mut left: f32 = 0.0;
    let mut right: f32 = 0.0;
    let mut bottom: f32 = 0.0;
    let mut top: f32 = 0.0;
    ext::HSD_LObjSetupInit(ext::HSD_CObjGetCurrent());
    ext::GXSetCullMode((GX_CULL_NONE as i32) as u32);
    ext::GXClearVtxDesc();
    ext::GXSetVtxDesc((GX_VA_POS as i32) as u32, (GX_DIRECT as i32) as u32);
    ext::GXSetVtxAttrFmt((GX_VTXFMT0 as i32) as u32, (GX_VA_POS as i32) as u32, (GX_TEX_ST as i32) as u32, (GX_RGBA6 as i32) as u32, 0 as u8);
    ext::HSD_CObjGetViewingMtx(ext::HSD_CObjGetCurrent(), sp7C.as_mut_ptr());
    ext::GXSetCurrentMtx(0 as u32);
    ext::GXLoadPosMtxImm(sp7C.as_mut_ptr(), 0 as u32);
    left = ext::Stage_GetBlastZoneLeftOffset();
    right = ext::Stage_GetBlastZoneRightOffset();
    bottom = ext::Stage_GetBlastZoneBottomOffset();
    top = ext::Stage_GetBlastZoneTopOffset();
    sp40 = mpLib_804D8144;
    mpLib_SetupDraw(mpLib_804D8140);
    ext::GXSetZMode(1 as u8, (GX_LEQUAL as i32) as u32, 0 as u8);
    ext::GXSetLineWidth(16 as u8, (GX_TO_ZERO as i32) as u32);
    ext::GXBegin((GX_LINESTRIP as i32) as u32, (GX_VTXFMT0 as i32) as u32, 5 as u16);
    ext::GXPosition3f32(left, bottom, 0.0f32);
    ext::GXPosition3f32(right, bottom, 0.0f32);
    ext::GXPosition3f32(right, top, 0.0f32);
    ext::GXPosition3f32(left, top, 0.0f32);
    ext::GXPosition3f32(left, bottom, 0.0f32);
    ext::GXEnd();
    mpLib_SetupDraw(sp40);
    ext::GXSetZMode(1 as u8, (GX_GREATER as i32) as u32, 0 as u8);
    ext::GXSetLineWidth(16 as u8, (GX_TO_ZERO as i32) as u32);
    ext::GXBegin((GX_LINESTRIP as i32) as u32, (GX_VTXFMT0 as i32) as u32, 5 as u16);
    ext::GXPosition3f32(left, bottom, 0.0f32);
    ext::GXPosition3f32(right, bottom, 0.0f32);
    ext::GXPosition3f32(right, top, 0.0f32);
    ext::GXPosition3f32(left, top, 0.0f32);
    ext::GXPosition3f32(left, bottom, 0.0f32);
    ext::GXEnd();
    left = ext::Stage_GetCamBoundsLeftOffset();
    right = ext::Stage_GetCamBoundsRightOffset();
    bottom = ext::Stage_GetCamBoundsBottomOffset();
    top = ext::Stage_GetCamBoundsTopOffset();
    sp34 = mpLib_804D814C;
    mpLib_SetupDraw(mpLib_804D8148);
    ext::GXSetZMode(1 as u8, (GX_LEQUAL as i32) as u32, 0 as u8);
    ext::GXSetLineWidth(16 as u8, (GX_TO_ZERO as i32) as u32);
    ext::GXBegin((GX_LINESTRIP as i32) as u32, (GX_VTXFMT0 as i32) as u32, 5 as u16);
    sp20 = sp34;
    ext::GXPosition3f32(left, bottom, 0.0f32);
    ext::GXPosition3f32(right, bottom, 0.0f32);
    ext::GXPosition3f32(right, top, 0.0f32);
    ext::GXPosition3f32(left, top, 0.0f32);
    ext::GXPosition3f32(left, bottom, 0.0f32);
    ext::GXEnd();
    mpLib_SetupDraw(sp20);
    ext::GXSetZMode(1 as u8, (GX_GREATER as i32) as u32, 0 as u8);
    ext::GXSetLineWidth(16 as u8, (GX_TO_ZERO as i32) as u32);
    ext::GXBegin((GX_LINESTRIP as i32) as u32, (GX_VTXFMT0 as i32) as u32, 5 as u16);
    ext::GXPosition3f32(left, bottom, 0.0f32);
    ext::GXPosition3f32(right, bottom, 0.0f32);
    ext::GXPosition3f32(right, top, 0.0f32);
    ext::GXPosition3f32(left, top, 0.0f32);
    ext::GXPosition3f32(left, bottom, 0.0f32);
    ext::GXEnd();
    mpLib_SetupDraw(mpLib_804D8150[0 as usize]);
    ext::GXSetZMode(0 as u8, (GX_LEQUAL as i32) as u32, 0 as u8);
    ext::GXSetLineWidth(16 as u8, (GX_TO_ZERO as i32) as u32);
    var_r30 = ext::cm_804D6468;
    'l342: while !var_r30.is_null() {
        if ext::Camera_8002928C(var_r30) != 0 {
            ext::GXBegin((GX_LINESTRIP as i32) as u32, (GX_VTXFMT0 as i32) as u32, 5 as u16);
            ext::GXPosition3f32((*var_r30).pos.x + (*var_r30).ext.h.x, (*var_r30).pos.y + (*var_r30).ext.v.x, 0.0f32);
            ext::GXPosition3f32((*var_r30).pos.x + (*var_r30).ext.h.y, (*var_r30).pos.y + (*var_r30).ext.v.x, 0.0f32);
            ext::GXPosition3f32((*var_r30).pos.x + (*var_r30).ext.h.y, (*var_r30).pos.y + (*var_r30).ext.v.y, 0.0f32);
            ext::GXPosition3f32((*var_r30).pos.x + (*var_r30).ext.h.x, (*var_r30).pos.y + (*var_r30).ext.v.y, 0.0f32);
            ext::GXPosition3f32((*var_r30).pos.x + (*var_r30).ext.h.x, (*var_r30).pos.y + (*var_r30).ext.v.x, 0.0f32);
            ext::GXEnd();
        }
        var_r30 = (*var_r30).prev;
    }
    ext::HSD_StateInvalidate((-1_i32));
}

/// `HSD_JObjGetChild`
unsafe fn HSD_JObjGetChild(mut jobj: *mut HSD_JObj) -> *mut HSD_JObj {
    if jobj.is_null() {
        return null_mut();
    } else {
        return (*jobj).child;
    }
}

/// `HSD_JObjGetNext`
unsafe fn HSD_JObjGetNext(mut jobj: *mut HSD_JObj) -> *mut HSD_JObj {
    if jobj.is_null() {
        return null_mut();
    } else {
        return (*jobj).next;
    }
}

/// `HSD_JObjGetParent`
unsafe fn HSD_JObjGetParent(mut jobj: *mut HSD_JObj) -> *mut HSD_JObj {
    if jobj.is_null() {
        return null_mut();
    } else {
        return (*jobj).parent;
    }
}

/// `HSD_JObjSetupMatrix`
unsafe fn HSD_JObjSetupMatrix(mut jobj: *mut HSD_JObj) {
    if (!(!jobj.is_null())) || (!(HSD_JObjMtxIsDirty(jobj) != 0)) {
        return;
    }
    ext::HSD_JObjSetupMatrixSub(jobj);
}

/// `HSD_JObjGetMtxPtr`
unsafe fn HSD_JObjGetMtxPtr(mut jobj: *mut HSD_JObj) -> *mut [f32; 4] {
    if !jobj.is_null() { (); } else { panic!("HSD_JObjGetMtxPtr: assertion failed"); }
    HSD_JObjSetupMatrix(jobj);
    return (*jobj).mtx.as_mut_ptr();
}

/// `HSD_GObjGetUserData`
unsafe fn HSD_GObjGetUserData(mut gobj: *mut HSD_GObj) -> *mut c_void {
    return (*gobj).user_data;
}

/// `HSD_JObjMtxIsDirty`
unsafe fn HSD_JObjMtxIsDirty(mut jobj: *mut HSD_JObj) -> i32 {
    let mut result: i32 = 0;
    if !jobj.is_null() { (); } else { panic!("HSD_JObjMtxIsDirty: assertion failed"); }
    result = 0;
    if (!(((*jobj).flags & ((1 << 23) as u32)) != 0)) && (((*jobj).flags & ((1 << 6) as u32)) != 0) {
        result = 1;
    }
    return result;
}
