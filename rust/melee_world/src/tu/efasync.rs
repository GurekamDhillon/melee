//! Port of src/melee/ef/efasync.c (TARGET_PC build): the async effect dispatcher
//! (`efAsync_Dispatch`), the effect-bank DAT table and its loaders, the deferred spawn queue,
//! and the m-ex custom-effect path (`efAsync_Mex*`).
//!
//! Every call out of this TU goes through [`EfHost`]. Its default methods forward to `mod ext`,
//! TODO stubs named exactly like the C functions (they panic until their TU is ported); a test or
//! the game wires real implementations by overriding the methods. The `HSD_JObj` inline setters
//! of jobj.h that the C expands into this TU are ported here ([`jobj`]).
//!
//! C varargs become a [`VaList`] over a slice of [`VaArg`]: `va_arg(vlist, T)` reads the next
//! entry; a callee taking a `va_list` gets a copy of the cursor (on PC `va_list` is passed by
//! value, so the callee's reads never advance the caller's).
//!
//! Float sites: `0.04f * x + 0.3f` (0x3E8) and `0.4f * HSD_Randf() + 2.8f` (0x427) are contracted
//! in the shipping build (`fmadds`, f64 multiply-add then f32); `M_TAU * HSD_Randf()` is a double
//! product narrowed to f32. Checked against the C by `tests/tu_efasync.rs`.
#![allow(non_snake_case, non_upper_case_globals, clippy::missing_safety_doc, clippy::too_many_arguments)]

use core::ffi::{c_char, c_void};
use core::ptr::null_mut;

use melee_core::fp::fmadds;
use melee_types::{
    EF_DAT_Entry, EF_Effect, EF_QueuedEffect, EF_UpdateFn, Fighter, HSD_GObj, HSD_GObjProc, HSD_Generator, HSD_JObj,
    HSD_ObjAllocData, Item, Vec3, Ft_Kind_Kirby, Ft_Kind_Max, Ft_Kind_Mex0,
};

use super::efdata::EfData;

// ---- constants (ef/types.h, ef/efasync.h, ft/it forward.h) ----
pub const EF_LOADKIND_ASYNC: u32 = 0;
pub const EF_LOADKIND_SYNC: u32 = 1;
pub const EF_SCALE_INHERIT: u8 = 0x01;
pub const EF_STATE_ASYNC: u8 = 0x80;
pub const EF_BANK_MAX: usize = 65;
pub const EF_MEX_MDL_START: i32 = 5000;
pub const EF_MEX_PTCL_START: i32 = 6000;
pub const EF_MEX_CPMDL_START: i32 = 7000;
pub const EF_MEX_CPPTCL_START: i32 = 8000;
pub const EF_MEX_END: i32 = 9000;
pub const EF_SPAWN_ATTACH: u32 = 0;
pub const EF_SPAWN_POS: u32 = 1;
pub const EF_SPAWN_POS_OFFSET: u32 = 2;
pub const EF_SPAWN_ATTACH_PARAM: u32 = 3;
pub const EF_SPAWN_POS_PARAM: u32 = 4;
pub const EF_SPAWN_POS_OFFSET_PARAM: u32 = 5;
pub const EF_SPAWN_POS_OFFSET_PARAM2: u32 = 6;
pub const EF_SPAWN_ATTACH_OFFSET: u32 = 7;
pub const EF_SPAWN_CAMERA_SHAKE: u32 = 8;
pub const EF_SPAWN_MEX: u32 = 9;
pub const HSD_GOBJ_CLASS_FIGHTER: u16 = 4;
pub const HSD_GOBJ_CLASS_ITEM: u16 = 6;
pub const JOBJ_MTX_DIRTY: u32 = 1 << 6;
pub const JOBJ_USE_QUATERNION: u32 = 1 << 17;
pub const JOBJ_MTX_INDEP_SRT: u32 = 1 << 25;
pub const M_PI: f64 = 3.14159265358979323846;
pub const M_PI_2: f64 = M_PI / 2.0;
pub const M_TAU: f64 = 6.283185307179586;

#[inline]
pub fn EF_MEX_IS_CUSTOM(id: i32) -> bool {
    melee_core::ct!("EF_MEX_IS_CUSTOM");
    (EF_MEX_MDL_START..EF_MEX_END).contains(&id)
}

// ---- varargs ----

/// One C variadic argument.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum VaArg {
    Vec3(*mut Vec3),
    JObj(*mut HSD_JObj),
    F32(*mut f32),
    U32(u32),
    Ptr(*mut c_void),
}

/// A `va_list` cursor.
#[derive(Clone, Copy, Debug)]
pub struct VaList<'a> {
    pub args: &'a [VaArg],
    pub pos: usize,
}

impl<'a> VaList<'a> {
    pub fn new(args: &'a [VaArg]) -> Self {
        Self { args, pos: 0 }
    }
    fn next(&mut self) -> VaArg {
        let a = *self.args.get(self.pos).expect("va_arg past the end of the argument list");
        self.pos += 1;
        a
    }
    /// The next argument as a pointer (any pointer kind; C reinterprets freely).
    pub fn ptr(&mut self) -> *mut c_void {
        match self.next() {
            VaArg::Vec3(p) => p.cast(),
            VaArg::JObj(p) => p.cast(),
            VaArg::F32(p) => p.cast(),
            VaArg::Ptr(p) => p,
            VaArg::U32(u) => u as usize as *mut c_void,
        }
    }
    pub fn vec3(&mut self) -> *mut Vec3 {
        self.ptr().cast()
    }
    pub fn jobj(&mut self) -> *mut HSD_JObj {
        self.ptr().cast()
    }
    pub fn f32p(&mut self) -> *mut f32 {
        self.ptr().cast()
    }
    pub fn u32(&mut self) -> u32 {
        match self.next() {
            VaArg::U32(u) => u,
            other => panic!("va_arg(u32) on {other:?}"),
        }
    }
}

// ---- TODO stubs for callees in unported TUs ----
pub mod ext {
    use super::*;
    pub unsafe fn efLib_Create_Attach(_gfx_id: u32, _gobj: *mut HSD_GObj, _jobj: *mut HSD_JObj) -> *mut EF_Effect { { return unsafe { crate::tu::eflib::efLib_Create_Attach(_gfx_id as _, _gobj as _, _jobj as _) as _ }; } }
    pub unsafe fn efLib_Create_AttachChild(_gfx_id: u32, _gobj: *mut HSD_GObj, _jobj: *mut HSD_JObj) -> *mut EF_Effect { { return unsafe { crate::tu::eflib::efLib_Create_AttachChild(_gfx_id as _, _gobj as _, _jobj as _) as _ }; } }
    pub unsafe fn efLib_Create_Attach_Pos(gfx_id: u32, gobj: *mut HSD_GObj, pos: *mut Vec3) -> *mut EF_Effect { unsafe { crate::tu::eflib::efLib_Create_Attach_Pos(gfx_id as _, gobj as _, &*(pos as *const _)) as _ } }
    pub unsafe fn efLib_CreateGenerator(gfx_id: i32, pos: *mut Vec3) -> *mut HSD_Generator { unsafe { crate::tu::eflib::efLib_CreateGenerator(gfx_id as _, &*(pos as *const _)) as _ } }
    pub unsafe fn efLib_CreateGenerator_AddAppSRT(_gfx_id: i32) -> *mut HSD_Generator { { return unsafe { crate::tu::eflib::efLib_CreateGenerator_AddAppSRT(_gfx_id as _) as _ }; } }
    pub unsafe fn efLib_CreateGenerator_Translate_FacingDir(gfx_id: i32, t: *mut Vec3, dir: f32) -> *mut HSD_Generator { unsafe { crate::tu::eflib::efLib_CreateGenerator_Translate_FacingDir(gfx_id as _, &*(t as *const _), dir as _) as _ } }
    pub unsafe fn efLib_CreateGenerator_AppSRT_SetScale(_gfx_id: i32, _vl: VaList<'_>) -> *mut HSD_Generator { panic!("TODO(group2 eflib) efLib_CreateGenerator_AppSRT_SetScale") }
    pub unsafe fn efLib_CreateGenerator_AppSRT_SetFacingDirScale(_gfx_id: i32, _vl: VaList<'_>) -> *mut HSD_Generator { panic!("TODO(group2 eflib) efLib_CreateGenerator_AppSRT_SetFacingDirScale") }
    pub unsafe fn efLib_CreateGenerator_AppSRT_SetPos(gfx_id: i32, gobj: *mut HSD_GObj, jobj: *mut HSD_JObj, pos: *mut Vec3) -> *mut EF_Effect { unsafe { crate::tu::eflib::efLib_CreateGenerator_AppSRT_SetPos(gfx_id as _, gobj as _, jobj as _, &*(pos as *const _)) as _ } }
    pub unsafe fn efLib_SetTevKonstColor(_jobj: *mut HSD_JObj, _count: i32, _konst: u32, _color: u32) { { unsafe { crate::tu::eflib::efLib_SetTevKonstColor(_jobj as _, _count as _, _konst as _, _color as _); } return; } }
    pub unsafe fn efLib_SetParamGfxId(_gobj: *mut HSD_GObj, _gfx_id: i32) { { unsafe { crate::tu::eflib::efLib_SetParamGfxId(_gobj as _, _gfx_id as _); } return; } }
    /// `((HSD_JObj**) efLib_AnimQueue)[i]` (eflib.c global).
    pub unsafe fn efLib_AnimQueue(_i: i32) -> *mut HSD_JObj { panic!("TODO(group2 eflib) efLib_AnimQueue") }
    pub unsafe extern "C" fn efLib_Cb_ApplyStoredAlpha(_e: *mut EF_Effect) { { unsafe { crate::tu::eflib::efLib_Cb_ApplyStoredAlpha(_e as _); } return; } }
    pub unsafe extern "C" fn efLib_Cb_Fall_FromParamY(_e: *mut EF_Effect) { { unsafe { crate::tu::eflib::efLib_Cb_Fall_FromParamY(_e as _); } return; } }
    pub unsafe extern "C" fn efLib_Cb_SetScale_FromParamX(_e: *mut EF_Effect) { { unsafe { crate::tu::eflib::efLib_Cb_SetScale_FromParamX(_e as _); } return; } }
    pub unsafe extern "C" fn efLib_Cb_ftCo_Bury(_e: *mut EF_Effect) { { unsafe { crate::tu::eflib::efLib_Cb_ftCo_Bury(_e as _); } return; } }
    pub unsafe fn hsd_8039EFAC(_link: i32, _bank: i32, _gfx_id: i32, _jobj: *mut HSD_JObj) -> *mut HSD_Generator { panic!("TODO(group1 generator) hsd_8039EFAC") }
    pub unsafe fn HSD_Randf() -> f32 { melee_runtime::tu::random::HSD_Randf(melee_runtime::tu::globals::rand()) }
    pub unsafe fn HSD_Randi(_max: i32) -> i32 { melee_runtime::tu::random::HSD_Randi(melee_runtime::tu::globals::rand(), _max) }
    pub unsafe fn HSD_JObjAnimAll(_jobj: *mut HSD_JObj) { panic!("TODO(group1 jobj) HSD_JObjAnimAll") }
    pub unsafe fn HSD_JObjSetMtxDirtySub(_jobj: *mut HSD_JObj) { unsafe { melee_runtime::tu::jobj_world::HSD_JObjSetMtxDirtySub(_jobj as _); } }
    pub unsafe fn lb_8000B1CC(_jobj: *mut HSD_JObj, _ofs: *const Vec3, _out: *mut Vec3) { { unsafe { melee_runtime::tu::lb_00B0::lb_8000B1CC(_jobj as _, _ofs as _, _out as _); } return; } }
    pub unsafe fn efSync_Spawn(_gfx_id: i32, _gobj: *mut HSD_GObj, _args: &[VaArg]) -> *mut c_void { panic!("TODO(group2 efsync) efSync_Spawn") }
    pub unsafe fn Camera_RequestQuake(_kind: u32, _pos: *mut Vec3) { unsafe { crate::cm::camera::Camera_RequestQuake(_kind as _, _pos as _) } }
    pub unsafe fn HSD_ObjAlloc(_d: *mut HSD_ObjAllocData) -> *mut c_void { { return unsafe { melee_runtime::tu::objalloc::HSD_ObjAlloc(_d as _) as _ }; } }
    pub unsafe fn HSD_ObjFree(_d: *mut HSD_ObjAllocData, _p: *mut c_void) { { unsafe { melee_runtime::tu::objalloc::HSD_ObjFree(_d as _, _p as _); } return; } }
    pub unsafe fn HSD_ObjAllocInit(_d: *mut HSD_ObjAllocData, _size: u32, _align: u32) { { unsafe { melee_runtime::tu::objalloc::HSD_ObjAllocInit(_d as _, _size as _, _align as _); } return; } }
    /// The global `HSD_GObj_CurrentInvokedProc` (gobj.c).
    pub unsafe fn HSD_GObj_CurrentInvokedProc() -> *mut HSD_GObjProc { unsafe { melee_runtime::tu::gobj_native::HSD_GObj_CurrentInvokedProc } }
    pub unsafe fn lbDvd_800178E8(_a: i32, _name: *const c_char, _b: i32, _c: i32, _d: i32, _e: i32, _f: i32, _g: u8, _idx: i32) { { unsafe { melee_runtime::tu::lbdvd::lbDvd_800178E8(_a as _, _name as _, _b as _, _c as _, _d as _, _e as _, _f as _, _g as _, _idx as _); } return; } }
    pub unsafe fn lbArchive_InitializeDAT(_ar: *mut c_void, _data: *mut u8, _len: u32) { { unsafe { melee_runtime::tu::lbarchive::lbArchive_InitializeDAT(_ar as _, _data as _, _len as _); } return; } }
    /// `lbArchive_80017040(dst, filename, &out, symbol, 0)`: one (out, symbol) pair.
    pub unsafe fn lbArchive_80017040(_dst: *mut *mut c_void, _file: *const c_char, _out: *mut *mut EF_DAT_Entry, _sym: *const c_char) -> bool { unsafe { let v: std::vec::Vec<*mut core::ffi::c_void> = std::vec![_out as *mut core::ffi::c_void, _sym as *mut core::ffi::c_void, core::ptr::null_mut()]; melee_runtime::tu::lbarchive::lbArchive_80017040(_dst as _, _file as _, &v) } }
    pub unsafe fn HSD_ArchiveGetPublicAddress(_ar: *mut c_void, _sym: *const c_char) -> *mut c_void { { return unsafe { melee_runtime::tu::archive::HSD_ArchiveGetPublicAddress(_ar as _, _sym as _) as _ }; } }
    pub unsafe fn psInitDataBankLocate(_cmd: *mut c_void, _tex: *mut c_void, _form: *mut i32) { { unsafe { melee_runtime::tu::particle::psInitDataBankLocate(_cmd as _, _tex as _, _form as _); } return; } }
    pub unsafe fn psInitDataBankLoad(_bank: i32, _cmd: *const i32, _tex: *const i32, _r: *const u32, _form: *const i32) { { unsafe { melee_runtime::tu::particle::psInitDataBankLoad(_bank as _, _cmd as _, _tex as _, _r as _, _form as _); } return; } }
    pub unsafe fn psInitDataBank(_bank: i32, _cmd: *mut i32, _tex: *mut i32, _r: *mut u32, _form: *mut i32) { { unsafe { melee_runtime::tu::particle::psInitDataBank(_bank as _, _cmd as _, _tex as _, _r as _, _form as _); } return; } }
    pub unsafe fn Mex_EffectCount() -> i32 { panic!("TODO(pc/platform mex) Mex_EffectCount") }
    pub unsafe fn Mex_EffectString(_i: i32, _which: i32) -> *mut c_char { panic!("TODO(pc/platform mex) Mex_EffectString") }
    /// `ftData_UnkBytePerCharacter[kind]` (ft/ftdata.c).
    pub unsafe fn ftData_UnkBytePerCharacter(_kind: i32) -> u8 { panic!("TODO(group3 ftdata) ftData_UnkBytePerCharacter") }
    pub unsafe fn ftKb_MexKirbyLike(_kind: i32) -> i32 { panic!("TODO(group4 ftkirby) ftKb_MexKirbyLike") }
    pub unsafe fn OSReport(msg: &str) { melee_runtime::tu::os_error::OSReport_str(msg) }
}

/// Calls out of efasync.c / efalt.c, plus the globals they reach. Every default forwards to the
/// matching [`ext`] stub.
pub unsafe trait EfHost {
    /// efdata.c's globals (`efLib_LoadKind`, `efLib_AnimCount`, ...).
    fn ef_data(&mut self) -> &mut EfData;
    /// efasync.c's own globals.
    fn ef_async(&mut self) -> &mut EfAsyncState;

    unsafe fn efLib_Create_Attach(&mut self, id: u32, g: *mut HSD_GObj, j: *mut HSD_JObj) -> *mut EF_Effect { ext::efLib_Create_Attach(id, g, j) }
    unsafe fn efLib_Create_AttachChild(&mut self, id: u32, g: *mut HSD_GObj, j: *mut HSD_JObj) -> *mut EF_Effect { ext::efLib_Create_AttachChild(id, g, j) }
    unsafe fn efLib_Create_Attach_Pos(&mut self, id: u32, g: *mut HSD_GObj, p: *mut Vec3) -> *mut EF_Effect { ext::efLib_Create_Attach_Pos(id, g, p) }
    unsafe fn efLib_CreateGenerator(&mut self, id: i32, p: *mut Vec3) -> *mut HSD_Generator { ext::efLib_CreateGenerator(id, p) }
    unsafe fn efLib_CreateGenerator_AddAppSRT(&mut self, id: i32) -> *mut HSD_Generator { ext::efLib_CreateGenerator_AddAppSRT(id) }
    unsafe fn efLib_CreateGenerator_Translate_FacingDir(&mut self, id: i32, t: *mut Vec3, d: f32) -> *mut HSD_Generator { ext::efLib_CreateGenerator_Translate_FacingDir(id, t, d) }
    unsafe fn efLib_CreateGenerator_AppSRT_SetScale(&mut self, id: i32, vl: VaList<'_>) -> *mut HSD_Generator { ext::efLib_CreateGenerator_AppSRT_SetScale(id, vl) }
    unsafe fn efLib_CreateGenerator_AppSRT_SetFacingDirScale(&mut self, id: i32, vl: VaList<'_>) -> *mut HSD_Generator { ext::efLib_CreateGenerator_AppSRT_SetFacingDirScale(id, vl) }
    unsafe fn efLib_CreateGenerator_AppSRT_SetPos(&mut self, id: i32, g: *mut HSD_GObj, j: *mut HSD_JObj, p: *mut Vec3) -> *mut EF_Effect { ext::efLib_CreateGenerator_AppSRT_SetPos(id, g, j, p) }
    unsafe fn efLib_SetTevKonstColor(&mut self, j: *mut HSD_JObj, n: i32, k: u32, c: u32) { ext::efLib_SetTevKonstColor(j, n, k, c) }
    unsafe fn efLib_SetParamGfxId(&mut self, g: *mut HSD_GObj, id: i32) { ext::efLib_SetParamGfxId(g, id) }
    unsafe fn efLib_AnimQueue(&mut self, i: i32) -> *mut HSD_JObj { ext::efLib_AnimQueue(i) }
    unsafe fn hsd_8039EFAC(&mut self, l: i32, b: i32, id: i32, j: *mut HSD_JObj) -> *mut HSD_Generator { ext::hsd_8039EFAC(l, b, id, j) }
    unsafe fn HSD_Randf(&mut self) -> f32 { ext::HSD_Randf() }
    unsafe fn HSD_Randi(&mut self, max: i32) -> i32 { ext::HSD_Randi(max) }
    unsafe fn HSD_JObjAnimAll(&mut self, j: *mut HSD_JObj) { ext::HSD_JObjAnimAll(j) }
    unsafe fn HSD_JObjSetMtxDirtySub(&mut self, j: *mut HSD_JObj) { ext::HSD_JObjSetMtxDirtySub(j) }
    unsafe fn lb_8000B1CC(&mut self, j: *mut HSD_JObj, o: *const Vec3, out: *mut Vec3) { ext::lb_8000B1CC(j, o, out) }
    unsafe fn efSync_Spawn(&mut self, id: i32, g: *mut HSD_GObj, args: &[VaArg]) -> *mut c_void { ext::efSync_Spawn(id, g, args) }
    unsafe fn Camera_RequestQuake(&mut self, k: u32, p: *mut Vec3) { ext::Camera_RequestQuake(k, p) }
    unsafe fn HSD_ObjAlloc(&mut self, d: *mut HSD_ObjAllocData) -> *mut c_void { ext::HSD_ObjAlloc(d) }
    unsafe fn HSD_ObjFree(&mut self, d: *mut HSD_ObjAllocData, p: *mut c_void) { ext::HSD_ObjFree(d, p) }
    unsafe fn HSD_ObjAllocInit(&mut self, d: *mut HSD_ObjAllocData, size: u32, align: u32) { ext::HSD_ObjAllocInit(d, size, align) }
    unsafe fn HSD_GObj_CurrentInvokedProc(&mut self) -> *mut HSD_GObjProc { ext::HSD_GObj_CurrentInvokedProc() }
    unsafe fn lbDvd_800178E8(&mut self, a: i32, n: *const c_char, b: i32, c: i32, d: i32, e: i32, f: i32, g: u8, idx: i32) { ext::lbDvd_800178E8(a, n, b, c, d, e, f, g, idx) }
    unsafe fn lbArchive_InitializeDAT(&mut self, ar: *mut c_void, data: *mut u8, len: u32) { ext::lbArchive_InitializeDAT(ar, data, len) }
    unsafe fn lbArchive_80017040(&mut self, dst: *mut *mut c_void, file: *const c_char, out: *mut *mut EF_DAT_Entry, sym: *const c_char) -> bool { ext::lbArchive_80017040(dst, file, out, sym) }
    unsafe fn HSD_ArchiveGetPublicAddress(&mut self, ar: *mut c_void, sym: *const c_char) -> *mut c_void { ext::HSD_ArchiveGetPublicAddress(ar, sym) }
    unsafe fn psInitDataBankLocate(&mut self, c: *mut c_void, t: *mut c_void, f: *mut i32) { ext::psInitDataBankLocate(c, t, f) }
    unsafe fn psInitDataBankLoad(&mut self, b: i32, c: *const i32, t: *const i32, r: *const u32, f: *const i32) { ext::psInitDataBankLoad(b, c, t, r, f) }
    unsafe fn psInitDataBank(&mut self, b: i32, c: *mut i32, t: *mut i32, r: *mut u32, f: *mut i32) { ext::psInitDataBank(b, c, t, r, f) }
    unsafe fn Mex_EffectCount(&mut self) -> i32 { ext::Mex_EffectCount() }
    unsafe fn Mex_EffectString(&mut self, i: i32, which: i32) -> *mut c_char { ext::Mex_EffectString(i, which) }
    unsafe fn ftData_UnkBytePerCharacter(&mut self, kind: i32) -> u8 { ext::ftData_UnkBytePerCharacter(kind) }
    unsafe fn ftKb_MexKirbyLike(&mut self, kind: i32) -> i32 { ext::ftKb_MexKirbyLike(kind) }
    unsafe fn OSReport(&mut self, msg: &str) { ext::OSReport(msg) }
}

// ---- jobj.h inlines expanded in this TU ----
pub mod jobj {
    use super::*;

    #[inline]
    #[track_caller]
    pub fn hsd_assert(cond: bool, what: &str) {
        if !cond {
            panic!("__assert(jobj.h): {what}");
        }
    }
    /// `GET_JOBJ(gobj)`.
    #[inline]
    pub unsafe fn GET_JOBJ(g: *mut HSD_GObj) -> *mut HSD_JObj {
        (*g).hsd_obj.cast()
    }
    /// The `HSD_JObjSetMtxDirty` macro.
    pub unsafe fn HSD_JObjSetMtxDirty<H: EfHost + ?Sized>(h: &mut H, j: *mut HSD_JObj) {
        if !j.is_null() && (*j).flags & JOBJ_MTX_DIRTY == 0 {
            h.HSD_JObjSetMtxDirtySub(j);
        }
    }
    unsafe fn dirty<H: EfHost + ?Sized>(h: &mut H, j: *mut HSD_JObj) {
        if (*j).flags & JOBJ_MTX_INDEP_SRT == 0 {
            HSD_JObjSetMtxDirty(h, j);
        }
    }
    pub unsafe fn HSD_JObjSetRotationY<H: EfHost + ?Sized>(h: &mut H, j: *mut HSD_JObj, y: f32) {
        hsd_assert(!j.is_null(), "jobj");
        hsd_assert((*j).flags & JOBJ_USE_QUATERNION == 0, "!(jobj->flags & JOBJ_USE_QUATERNION)");
        (*j).rotate.y = y;
        dirty(h, j);
    }
    pub unsafe fn HSD_JObjSetRotationZ<H: EfHost + ?Sized>(h: &mut H, j: *mut HSD_JObj, z: f32) {
        hsd_assert(!j.is_null(), "jobj");
        hsd_assert((*j).flags & JOBJ_USE_QUATERNION == 0, "!(jobj->flags & JOBJ_USE_QUATERNION)");
        (*j).rotate.z = z;
        dirty(h, j);
    }
    pub unsafe fn HSD_JObjSetScale<H: EfHost + ?Sized>(h: &mut H, j: *mut HSD_JObj, s: *const Vec3) {
        hsd_assert(!j.is_null(), "jobj");
        hsd_assert(!s.is_null(), "scale");
        (*j).scale = *s;
        dirty(h, j);
    }
    pub unsafe fn HSD_JObjSetScaleX<H: EfHost + ?Sized>(h: &mut H, j: *mut HSD_JObj, v: f32) {
        hsd_assert(!j.is_null(), "jobj");
        (*j).scale.x = v;
        dirty(h, j);
    }
    pub unsafe fn HSD_JObjSetScaleY<H: EfHost + ?Sized>(h: &mut H, j: *mut HSD_JObj, v: f32) {
        hsd_assert(!j.is_null(), "jobj");
        (*j).scale.y = v;
        dirty(h, j);
    }
    pub unsafe fn HSD_JObjSetScaleZ<H: EfHost + ?Sized>(h: &mut H, j: *mut HSD_JObj, v: f32) {
        hsd_assert(!j.is_null(), "jobj");
        (*j).scale.z = v;
        dirty(h, j);
    }
    pub unsafe fn HSD_JObjSetTranslateZ<H: EfHost + ?Sized>(h: &mut H, j: *mut HSD_JObj, v: f32) {
        hsd_assert(!j.is_null(), "jobj");
        (*j).translate.z = v;
        dirty(h, j);
    }
    pub unsafe fn HSD_JObjGetScale(j: *mut HSD_JObj, out: *mut Vec3) {
        hsd_assert(!j.is_null(), "jobj");
        hsd_assert(!out.is_null(), "scale");
        *out = (*j).scale;
    }
    pub unsafe fn HSD_JObjGetScaleY(j: *mut HSD_JObj) -> f32 {
        hsd_assert(!j.is_null(), "jobj");
        (*j).scale.y
    }
    pub unsafe fn HSD_JObjGetRotationY(j: *mut HSD_JObj) -> f32 {
        hsd_assert(!j.is_null(), "jobj");
        (*j).rotate.y
    }
    pub unsafe fn HSD_JObjGetTranslationZ(j: *mut HSD_JObj) -> f32 {
        hsd_assert(!j.is_null(), "jobj");
        (*j).translate.z
    }
    pub unsafe fn HSD_JObjGetChild(j: *mut HSD_JObj) -> *mut HSD_JObj {
        if j.is_null() { null_mut() } else { (*j).child }
    }
    pub unsafe fn HSD_JObjGetNext(j: *mut HSD_JObj) -> *mut HSD_JObj {
        if j.is_null() { null_mut() } else { (*j).next }
    }
}
use jobj::*;

// ---- efasync.c globals ----

/// effBehaviorTable (m-ex): how each model / generator of a bank is placed.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct EfMexBehavior {
    pub mdl_num: i32,
    pub mdl_type: *mut u8,
    pub ptcl_num: i32,
    pub ptcl_type: *mut u8,
}

/// The globals of efasync.c (`efAsync_AllocData`, `efAsync_DatEntries`, the m-ex tables and
/// `efAsync_MexMiss`'s function statics).
pub struct EfAsyncState {
    pub alloc_data: HSD_ObjAllocData,
    pub dat_entries: [EF_DAT_Entry; EF_BANK_MAX],
    pub mex_bhv: [*mut EfMexBehavior; EF_BANK_MAX],
    pub mex_ready: bool,
    pub miss_seen: [i32; 256],
    pub miss_nseen: i32,
}

/// The retail rows of `efAsync_DatEntries` (file, data-table symbol); the rest are empty.
pub const DAT_NAMES: &[(usize, &core::ffi::CStr, &core::ffi::CStr)] = &[
    (0, c"EfCoData.dat", c"effCommonDataTable"),
    (1, c"EfMrData.dat", c"effMarioDataTable"),
    (2, c"EfSsData.dat", c"effSamusDataTable"),
    (3, c"EfFxData.dat", c"effFoxDataTable"),
    (4, c"EfCaData.dat", c"effCaptainDataTable"),
    (5, c"EfKbData.dat", c"effKirbyDataTable"),
    (6, c"EfLkData.dat", c"effLinkDataTable"),
    (7, c"EfPkData.dat", c"effPikachuDataTable"),
    (8, c"EfDkData.dat", c"effDonkeyDataTable"),
    (9, c"EfYsData.dat", c"effYoshiDataTable"),
    (10, c"EfNsData.dat", c"effNessDataTable"),
    (11, c"EfPrData.dat", c"effPurinDataTable"),
    (12, c"EfKpData.dat", c"effKoopaDataTable"),
    (13, c"EfMtData.dat", c"effMewtwoDataTable"),
    (14, c"EfIcData.dat", c"effIceclimberDataTable"),
    (15, c"EfPeData.dat", c"effPeachDataTable"),
    (16, c"EfMsData.dat", c"effMarsDataTable"),
    (17, c"EfZdData.dat", c"effZeldaDataTable"),
    (18, c"EfLgData.dat", c"effLuigiDataTable"),
    (19, c"EfGnData.dat", c"effGanonDataTable"),
    (20, c"EfKbMs.dat", c"effKirbyMarsDataTable"),
    (21, c"EfKbZd.dat", c"effKirbyZeldaDataTable"),
    (31, c"EfMnData.dat", c"effMenuDataTable"),
    (32, c"EfKbMr.dat", c"effKirbyMarioDataTable"),
    (33, c"EfKbFx.dat", c"effKirbyFoxDataTable"),
    (34, c"EfKbSs.dat", c"effKirbySamusDataTable"),
    (36, c"EfKbPk.dat", c"effKirbyPikachuDataTable"),
    (37, c"EfKbLg.dat", c"effKirbyLuigiDataTable"),
    (38, c"EfKbCa.dat", c"effKirbyCaptainDataTable"),
    (39, c"EfKbDk.dat", c"effKirbyDonkeyDataTable"),
    (41, c"EfKbKp.dat", c"effKirbyKoopaDataTable"),
    (46, c"EfKbIc.dat", c"effKirbyIceDataTable"),
    (47, c"EfKbGn.dat", c"effKirbyGanonDataTable"),
    (48, c"EfKbFe.dat", c"effKirbyEmblemDataTable"),
    (49, c"EfFeData.dat", c"effEmblemDataTable"),
];

impl Default for EfAsyncState {
    fn default() -> Self {
        // SAFETY: all-zero is the C .bss / NULL-initialised state.
        let mut s: Self = Self {
            alloc_data: unsafe { core::mem::zeroed() },
            dat_entries: unsafe { core::mem::zeroed() },
            mex_bhv: [null_mut(); EF_BANK_MAX],
            mex_ready: false,
            miss_seen: [0; 256],
            miss_nseen: 0,
        };
        for &(i, f, n) in DAT_NAMES {
            {
                s.dat_entries[i].ef_DAT_file = f.as_ptr() as *mut c_char;
                s.dat_entries[i].effDataTable_name = n.as_ptr() as *mut c_char;
            }
        }
        s
    }
}

// ---- pure float helpers (checked against the C) ----

/// 0x3E8: `0.04f * x + 0.3f` (contracted) clamped to [0.3, 1.5].
#[inline]
pub fn dispatch_3e8_scale(x: f32) -> f32 {
    melee_core::ct!("dispatch_3e8_scale");
    let mut f = fmadds(0.04, x, 0.3);
    if f < 0.3 {
        f = 0.3;
    }
    if f > 1.5 {
        f = 1.5;
    }
    f
}
/// 0x427: `0.4f * r + 2.8f` (contracted).
#[inline]
pub fn dispatch_427_fall(r: f32) -> f32 {
    melee_core::ct!("dispatch_427_fall");
    fmadds(0.4, r, 2.8)
}
/// `M_TAU * HSD_Randf()` narrowed to f32.
#[inline]
pub fn tau_times(r: f32) -> f32 {
    melee_core::ct!("tau_times");
    (M_TAU * r as f64) as f32
}
/// `efAsync_SetEffectFacingDir` / the dispatchers' `rot_y`: ±M_PI_2 as f32.
#[inline]
pub fn facing_rot(dir: f32) -> f32 {
    melee_core::ct!("facing_rot");
    (if dir < 0.0 { -M_PI_2 } else { M_PI_2 }) as f32
}
/// `efAsync_MexMiss`'s dedup key (C int arithmetic, wrapping).
#[inline]
pub fn mex_miss_key(kind: i32, gfx_id: i32) -> i32 {
    melee_core::ct!("mex_miss_key");
    kind.wrapping_mul(10000)
        .wrapping_add(gfx_id.wrapping_rem(100000))
        .wrapping_mul(2)
        .wrapping_add((gfx_id >= 100000) as i32)
}

// ---- static inline helpers of efasync.c ----

unsafe fn set_effect_rotz_from_ptr<H: EfHost + ?Sized>(h: &mut H, e: *mut EF_Effect, rot: *const f32) {
    HSD_JObjSetRotationZ(h, GET_JOBJ((*e).gobj), *rot);
}
unsafe fn set_effect_random_rotz<H: EfHost + ?Sized>(h: &mut H, e: *mut EF_Effect) {
    let r = h.HSD_Randf();
    HSD_JObjSetRotationZ(h, GET_JOBJ((*e).gobj), tau_times(r));
}
unsafe fn set_effect_scale_xyz<H: EfHost + ?Sized>(h: &mut H, e: *mut EF_Effect, s: f32) {
    let j = GET_JOBJ((*e).gobj);
    let _ = (*j).scale.x;
    HSD_JObjSetScaleX(h, GET_JOBJ((*e).gobj), s);
    HSD_JObjSetScaleY(h, GET_JOBJ((*e).gobj), s);
    HSD_JObjSetScaleZ(h, GET_JOBJ((*e).gobj), s);
}
unsafe fn set_effect_facing_dir<H: EfHost + ?Sized>(h: &mut H, e: *mut EF_Effect, dir: f32) {
    HSD_JObjSetRotationY(h, GET_JOBJ((*e).gobj), facing_rot(dir));
}
unsafe fn set_scale3(g: *mut HSD_Generator, v: f32) {
    let a = (*g).appsrt;
    (*a).scale.x = v;
    (*a).scale.y = v;
    (*a).scale.z = v;
}

/// Drain `efLib_AnimQueue` (the tail loop of both dispatchers).
pub unsafe fn drain_anim_queue<H: EfHost + ?Sized>(h: &mut H) {
    melee_core::ct!("drain_anim_queue");
    while h.ef_data().anim_count != 0 {
        let cnt = h.ef_data().anim_count - 1;
        h.ef_data().anim_count = cnt;
        let j = h.efLib_AnimQueue(cnt);
        h.HSD_JObjAnimAll(j);
    }
}

fn upd(f: unsafe extern "C" fn(*mut EF_Effect)) -> EF_UpdateFn {
    Some(f)
}

/// The 0x417/0x418/0x41A tint: konst colour for `u32_1`.
#[inline]
pub fn tint_konst(c: u32) -> u32 {
    melee_core::ct!("tint_konst");
    if c.wrapping_add(0xFFA0_0000) == 0x6060 { 0x808080 } else { 0xFFFFFF }
}

/// efAsync_Dispatch.
pub unsafe fn efAsync_Dispatch<H: EfHost + ?Sized>(h: &mut H, gfx_id: i32, gobj: *mut HSD_GObj, vl: &mut VaList<'_>) -> *mut c_void {
    melee_core::ct!("efAsync_Dispatch");
    let mut ret: *mut c_void = null_mut();
    let mut scale = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut translate = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

    // Attach_Pos + EF_STATE_ASYNC (+ optional facing / rotZ) family.
    macro_rules! attach_pos_async {
        ($id:expr) => {{
            let e = h.efLib_Create_Attach_Pos($id, gobj, vl.vec3());
            ret = e.cast();
            if !e.is_null() { (*e).state_flags |= EF_STATE_ASYNC; }
            e
        }};
    }
    macro_rules! gen {
        ($id:expr) => {{ ret = h.efLib_CreateGenerator($id, vl.vec3()).cast(); }};
    }
    macro_rules! efac {
        ($b:expr, $id:expr) => {{ ret = h.hsd_8039EFAC(0, $b, $id, vl.jobj()).cast(); }};
    }
    macro_rules! translate_facing {
        ($id:expr, $neg_unless:expr) => {{
            translate = *vl.vec3();
            let f = *vl.f32p();
            let f = match $neg_unless { Some(pos_id) if gfx_id != pos_id => -f, _ => f };
            ret = h.efLib_CreateGenerator_Translate_FacingDir($id, &mut translate, f).cast();
        }};
    }
    macro_rules! appsrt_rotz {
        ($id:expr) => {{
            let g = h.efLib_CreateGenerator_AddAppSRT($id);
            if !g.is_null() {
                let v = vl.vec3();
                (*(*g).appsrt).translate = *v;
                let f = *vl.f32p();
                ret = g.cast();
                (*(*g).appsrt).rot.z = f;
            }
        }};
    }
    macro_rules! appsrt_scale_noret {
        ($id:expr) => {{
            let g = h.efLib_CreateGenerator_AddAppSRT($id);
            if !g.is_null() {
                let v = vl.vec3();
                (*(*g).appsrt).translate = *v;
                let f = *vl.f32p();
                set_scale3(g, f);
            }
        }};
    }
    macro_rules! appsrt_jobjpos_scale {
        ($id:expr) => {{
            let g = h.efLib_CreateGenerator_AddAppSRT($id);
            if !g.is_null() {
                let j = vl.jobj();
                h.lb_8000B1CC(j, core::ptr::null(), &mut (*(*g).appsrt).translate);
                let f = *vl.f32p();
                ret = g.cast();
                set_scale3(g, f);
            }
        }};
    }
    macro_rules! tint {
        ($id:expr, $count:expr) => {{
            h.ef_data().load_kind = EF_LOADKIND_SYNC;
            let e = h.efLib_Create_Attach($id, gobj, vl.jobj());
            ret = e.cast();
            if !e.is_null() {
                let j1 = GET_JOBJ((*e).gobj);
                let c = vl.u32();
                let j2 = if j1.is_null() { null_mut() } else { (*j1).child };
                h.efLib_SetTevKonstColor(j2, $count, tint_konst(c), c);
                (*e).scale_flags |= EF_SCALE_INHERIT;
                h.efLib_SetParamGfxId(gobj, gfx_id);
                (*e).update = upd(ext::efLib_Cb_ApplyStoredAlpha);
            }
        }};
    }
    macro_rules! attach_scale_f {
        ($id:expr) => {{
            let e = h.efLib_Create_Attach($id, gobj, vl.jobj());
            ret = e.cast();
            if !e.is_null() {
                let f = *vl.f32p();
                scale = Vec3 { x: f, y: f, z: f };
                HSD_JObjSetScale(h, GET_JOBJ((*e).gobj), &scale);
            }
        }};
    }
    macro_rules! jobjpos_facing_scale {
        ($id:expr) => {{
            let j = vl.jobj();
            h.lb_8000B1CC(j, core::ptr::null(), &mut translate);
            let d = *vl.f32p();
            let g = h.efLib_CreateGenerator_Translate_FacingDir($id, &mut translate, d);
            if !g.is_null() {
                let f = *vl.f32p();
                ret = g.cast();
                set_scale3(g, f);
            }
        }};
    }
    macro_rules! appsrt_rot_from_gobj {
        ($id:expr) => {{
            let g = h.efLib_CreateGenerator_AddAppSRT($id);
            if !g.is_null() {
                let j = vl.jobj();
                h.lb_8000B1CC(j, core::ptr::null(), &mut (*(*g).appsrt).translate);
                let f = *vl.f32p();
                set_scale3(g, f);
                let j3 = GET_JOBJ(gobj);
                let _ = HSD_JObjGetRotationY(j3);
                ret = g.cast();
                (*(*g).appsrt).rot.y = (*j3).rotate.y;
            }
        }};
    }

    match gfx_id {
        0x3E8 => {
            let id = if h.HSD_Randi(8) == 0 { 9 } else { 0xA };
            let e = h.efLib_Create_Attach_Pos(id, gobj, vl.vec3());
            ret = e.cast();
            if !e.is_null() {
                (*e).state_flags |= EF_STATE_ASYNC;
                let f = dispatch_3e8_scale(*vl.f32p());
                scale = Vec3 { x: f, y: f, z: f };
                HSD_JObjSetScale(h, GET_JOBJ((*e).gobj), &scale);
            }
        }
        0x3E9 => gen!(0xC),
        0x3EA => gen!(0x14),
        0x3EB => { attach_pos_async!(7); }
        0x3EC => {
            let e = attach_pos_async!(8);
            if !e.is_null() { set_effect_random_rotz(h, e); }
        }
        0x3ED => {
            let v = vl.vec3();
            translate = *v;
            let g0 = h.efLib_CreateGenerator(0x50, &mut translate);
            ret = g0.cast();
            if !g0.is_null() {
                let g = h.efLib_CreateGenerator_AddAppSRT(0x54);
                if !g.is_null() {
                    (*(*g).appsrt).translate = translate;
                    let rot_y: f64 = if *vl.f32p() < 0.0 { 0.0 } else { -M_PI };
                    (*(*g).appsrt).rot.y = rot_y as f32;
                }
            }
        }
        0x3EE => {
            let e = h.efLib_Create_Attach_Pos(0x27, gobj, vl.vec3());
            ret = e.cast();
            if !e.is_null() { set_effect_random_rotz(h, e); }
        }
        0x3EF | 0x3F0 => translate_facing!(0x42, Some(0x3F0)),
        0x3F1 | 0x3F2 => translate_facing!(0x14B, Some(0x3F2)),
        0x3F3 => gen!(0xB),
        0x3F4 => gen!(0x48),
        0x3F5 => {
            let e = attach_pos_async!(0x10);
            if !e.is_null() { set_effect_facing_dir(h, e, *vl.f32p()); }
        }
        0x3F6 => { attach_pos_async!(0x11); }
        0x3F7 | 0x3F8 | 0x3F9 | 0x3FD | 0x3FF => {
            let id = match gfx_id { 0x3F7 => 0x12, 0x3F8 => 0x13, 0x3F9 => 0x14, 0x3FD => 3, _ => 5 };
            let e = attach_pos_async!(id);
            if !e.is_null() {
                set_effect_facing_dir(h, e, *vl.f32p());
                set_effect_rotz_from_ptr(h, e, vl.f32p());
            }
        }
        0x3FA => { attach_pos_async!(0x15); }
        0x3FB => { attach_pos_async!(0x16); }
        0x3FC => { attach_pos_async!(0x17); }
        0x3FE => translate_facing!(0x107, None::<i32>),
        0x400 | 0x401 => translate_facing!(0x5A, Some(0x401)),
        0x402 => efac!(0, 0x59),
        0x403 => efac!(0, 0x5E),
        0x404 | 0x406 => {
            let e = attach_pos_async!(if gfx_id == 0x404 { 0x18 } else { 4 });
            if !e.is_null() { set_effect_rotz_from_ptr(h, e, vl.f32p()); }
        }
        0x405 => gen!(0x2C),
        0x407 => gen!(0x3C),
        0x408 => appsrt_rotz!(0x3E),
        0x409 => efac!(0, 0xE2),
        0x40A => appsrt_rotz!(0x241),
        0x40B => appsrt_rotz!(0x242),
        0x40C => gen!(0x19),
        0x40D => {
            let g = h.efLib_CreateGenerator_AddAppSRT(0x19);
            if !g.is_null() {
                let v = vl.vec3();
                (*(*g).appsrt).translate = *v;
                HSD_JObjGetScale(GET_JOBJ(gobj), &mut scale);
                ret = g.cast();
                set_scale3(g, scale.y);
            }
        }
        0x40E => gen!(0x43),
        0x40F => gen!(0xE3),
        0x410 => gen!(0x22A),
        0x411 => gen!(0x4B),
        0x412 => efac!(0, 0x13),
        0x413 => efac!(0, 0x37),
        0x414 => efac!(0, 0xE1),
        0x415 => {
            let e = h.efLib_Create_AttachChild(0x25, gobj, vl.jobj());
            ret = e.cast();
            if !e.is_null() {
                (*e).state_flags |= EF_STATE_ASYNC;
                let f = *vl.f32p();
                scale = Vec3 { x: f, y: f, z: f };
                HSD_JObjSetScale(h, GET_JOBJ((*e).gobj), &scale);
            }
        }
        0x416 => gen!(0x196),
        0x417 => tint!(0xB, 1),
        0x418 => tint!(0xC, 0),
        0x419 => {
            h.ef_data().load_kind = EF_LOADKIND_SYNC;
            let e = h.efLib_Create_Attach(0xD, gobj, vl.jobj());
            ret = e.cast();
            if !e.is_null() {
                let mut j1 = GET_JOBJ((*e).gobj);
                let c = vl.u32();
                let k = tint_konst(c);
                let child = HSD_JObjGetChild(j1);
                h.efLib_SetTevKonstColor(child, 0, k, c);
                j1 = HSD_JObjGetNext(child);
                for _ in 0..6 {
                    h.efLib_SetTevKonstColor(j1, 0, k, c);
                    j1 = HSD_JObjGetNext(j1);
                }
                (*e).scale_flags |= EF_SCALE_INHERIT;
                h.efLib_SetParamGfxId(gobj, gfx_id);
                (*e).update = upd(ext::efLib_Cb_ApplyStoredAlpha);
            }
        }
        0x41A => tint!(0xE, 0),
        0x41B => {
            h.ef_data().load_kind = EF_LOADKIND_SYNC;
            let g = h.efLib_CreateGenerator_AddAppSRT(0x31);
            if !g.is_null() {
                let j = vl.jobj();
                h.lb_8000B1CC(j, core::ptr::null(), &mut (*(*g).appsrt).translate);
                HSD_JObjGetScale(j, &mut scale);
                set_scale3(g, scale.y);
            }
        }
        0x41C => gen!(0x5D),
        0x41D => { attach_pos_async!(0xF); }
        0x41E => ret = h.efLib_CreateGenerator_AppSRT_SetScale(0x55, *vl).cast(),
        0x41F => gen!(0x5C),
        0x420 => gen!(0x159),
        0x421 => gen!(0x3F),
        0x422 => efac!(0, 0x5B),
        0x423 | 0x424 => {
            let e = h.efLib_Create_Attach(if gfx_id == 0x423 { 1 } else { 2 }, gobj, vl.jobj());
            ret = e.cast();
            if !e.is_null() { set_effect_rotz_from_ptr(h, e, vl.f32p()); }
        }
        0x425 => gen!(0x7E),
        0x426 => gen!(0x7F),
        0x427 => {
            translate = *vl.vec3();
            let mut effect: *mut EF_Effect = null_mut();
            for i in 0..6 {
                let node = h.efLib_Create_Attach_Pos(0x1B, gobj, &mut translate);
                if node.is_null() {
                    break;
                }
                (*node).update = upd(ext::efLib_Cb_Fall_FromParamY);
                (*node).lifetime = 0x28;
                let r = h.HSD_Randf();
                (*node).params.y = dispatch_427_fall(r);
                let r = h.HSD_Randf();
                let f = tau_times(r);
                HSD_JObjSetRotationY(h, GET_JOBJ((*node).gobj), f);
                if i != 0 {
                    (*effect).next = node;
                    effect = (*effect).next;
                } else {
                    effect = node;
                    ret = node.cast();
                }
            }
        }
        0x428 => ret = h.efLib_CreateGenerator_AppSRT_SetScale(0xCA, *vl).cast(),
        0x429 => ret = h.efLib_CreateGenerator_AppSRT_SetScale(0xCE, *vl).cast(),
        0x42A => ret = h.efLib_CreateGenerator_AppSRT_SetScale(0xCF, *vl).cast(),
        0x42B => {
            h.ef_data().load_kind = EF_LOADKIND_SYNC;
            let e = h.efLib_Create_Attach_Pos(0x19, gobj, vl.vec3());
            ret = e.cast();
            if !e.is_null() {
                let j1 = GET_JOBJ((*e).gobj);
                let f = *vl.f32p();
                HSD_JObjSetRotationZ(h, j1, f);
                let f = *vl.f32p();
                scale = Vec3 { x: f, y: f, z: f };
                HSD_JObjSetScale(h, j1, &scale);
                let j2 = if j1.is_null() { null_mut() } else { (*j1).child };
                let a = vl.u32();
                let b = vl.u32();
                h.efLib_SetTevKonstColor(j2, 0, a, b);
            }
        }
        0x42C => {
            h.ef_data().load_kind = EF_LOADKIND_SYNC;
            let e = h.efLib_Create_Attach_Pos(0x1A, gobj, vl.vec3());
            ret = e.cast();
            if !e.is_null() {
                let f = *vl.f32p();
                HSD_JObjSetRotationZ(h, GET_JOBJ((*e).gobj), f);
            }
        }
        0x42D => {
            h.ef_data().load_kind = EF_LOADKIND_SYNC;
            gen!(0x121)
        }
        0x42E => translate_facing!(0x13C, None::<i32>),
        0x42F | 0x431 => {
            let e = h.efLib_Create_Attach_Pos(if gfx_id == 0x42F { 0x20 } else { 0x21 }, gobj, vl.vec3());
            ret = e.cast();
            if !e.is_null() {
                let r = facing_rot(*vl.f32p());
                HSD_JObjSetRotationY(h, GET_JOBJ((*e).gobj), r);
            }
        }
        0x430 => translate_facing!(0x140, None::<i32>),
        0x432 => appsrt_scale_noret!(0x145),
        0x433 => efac!(0, 0x115),
        0x434 => gen!(0x14D),
        0x435 | 0x436 | 0x437 => {
            let id = match gfx_id { 0x435 => 0x14E, 0x436 => 0x153, _ => 0x156 };
            let g = h.efLib_CreateGenerator_AddAppSRT(id);
            if !g.is_null() {
                let v = vl.vec3();
                (*(*g).appsrt).translate = *v;
                (*(*g).appsrt).rot.y = M_PI_2 as f32;
                let f = *vl.f32p();
                ret = g.cast();
                (*(*g).appsrt).rot.z = f;
            }
        }
        0x438 => {
            let e = h.efLib_Create_Attach(0x22, gobj, vl.jobj());
            ret = e.cast();
            if !e.is_null() {
                let j3 = GET_JOBJ(gobj);
                let _ = HSD_JObjGetRotationY(j3);
                let f = (*j3).rotate.y;
                HSD_JObjSetRotationY(h, GET_JOBJ((*e).gobj), f);
                (*e).update = upd(ext::efLib_Cb_SetScale_FromParamX);
                (*e).params.x = *vl.f32p();
            }
        }
        0x439 => {
            let e = h.efLib_Create_Attach(0x23, gobj, vl.jobj());
            ret = e.cast();
            if !e.is_null() { set_effect_scale_xyz(h, e, *vl.f32p()); }
        }
        0x43A => gen!(0x193),
        0x43B => gen!(0x192),
        0x43C => gen!(0x1A0),
        0x43D => efac!(0, 0x1AF),
        0x43E => {
            h.ef_data().load_kind = EF_LOADKIND_SYNC;
            let e = h.efLib_Create_Attach(0x24, gobj, vl.jobj());
            ret = e.cast();
            if !e.is_null() { set_effect_scale_xyz(h, e, *vl.f32p()); }
        }
        0x43F => {
            h.ef_data().load_kind = EF_LOADKIND_SYNC;
            appsrt_scale_noret!(0xCA)
        }
        0x440 => translate_facing!(0x1D8, None::<i32>),
        0x441 => gen!(0x1FB),
        0x442 => gen!(0x1DC),
        0x443 => appsrt_scale_noret!(0x1F1),
        0x444 => gen!(0x1FF),
        0x445 => gen!(0x209),
        0x446 | 0x448 => {
            let j = vl.jobj();
            let p = vl.vec3();
            let (id, life) = if gfx_id == 0x446 { (0x1B, 0x16) } else { (0x92, 0xB4) };
            let e = h.efLib_CreateGenerator_AppSRT_SetPos(id, gobj, j, p);
            ret = e.cast();
            if !e.is_null() { (*e).lifetime = life; }
        }
        0x447 => {
            h.ef_data().load_kind = EF_LOADKIND_SYNC;
            let e = h.efLib_Create_Attach_Pos(0x28, gobj, vl.vec3());
            ret = e.cast();
            if !e.is_null() {
                (*e).update = upd(ext::efLib_Cb_ftCo_Bury);
                let f = *vl.f32p();
                scale = Vec3 { x: f, y: f, z: f };
                HSD_JObjSetScale(h, GET_JOBJ((*e).gobj), &scale);
            }
        }
        0x449 => ret = h.efLib_Create_Attach(0x26, gobj, vl.jobj()).cast(),
        0x44A | 0x44B | 0x44C => {
            let s = match gfx_id { 0x44A => 0.5f32, 0x44B => 1.0, _ => 2.0 };
            let g = h.efLib_CreateGenerator_AddAppSRT(0x237);
            if !g.is_null() {
                let v = vl.vec3();
                (*(*g).appsrt).translate = *v;
                set_scale3(g, s);
            }
        }
        0x44D => ret = h.efLib_CreateGenerator_AppSRT_SetScale(0x48, *vl).cast(),
        0x44E | 0x457 => ret = h.efLib_CreateGenerator_AppSRT_SetFacingDirScale(0xDA, *vl).cast(),
        0x44F | 0x458 => ret = h.efLib_CreateGenerator_AppSRT_SetFacingDirScale(0xDB, *vl).cast(),
        0x450 | 0x459 => ret = h.efLib_CreateGenerator_AppSRT_SetFacingDirScale(0xDC, *vl).cast(),
        0x451 | 0x45A => ret = h.efLib_CreateGenerator_AppSRT_SetFacingDirScale(0xDD, *vl).cast(),
        0x452 => appsrt_jobjpos_scale!(0x21E),
        0x453 => jobjpos_facing_scale!(0x234),
        0x454 => jobjpos_facing_scale!(0x235),
        0x455 => jobjpos_facing_scale!(0x236),
        0x456 => jobjpos_facing_scale!(0x23D),
        0x45B => ret = h.efLib_CreateGenerator_AppSRT_SetScale(0x8C, *vl).cast(),
        0x45C => ret = h.efLib_CreateGenerator_AppSRT_SetScale(0x8D, *vl).cast(),
        0x45D => appsrt_jobjpos_scale!(0x23F),
        0x45E => appsrt_jobjpos_scale!(0x240),
        0x45F => ret = h.efLib_CreateGenerator_AppSRT_SetScale(0x8E, *vl).cast(),
        0x460 => ret = h.efLib_CreateGenerator_AppSRT_SetScale(0x99, *vl).cast(),
        0x461 => ret = h.efLib_CreateGenerator_AppSRT_SetScale(0x95, *vl).cast(),
        0x462 => ret = h.efLib_CreateGenerator_AppSRT_SetScale(0x219, *vl).cast(),
        0x463 => attach_scale_f!(0x29),
        0x464 => attach_scale_f!(0x2A),
        0x465 => ret = h.efLib_CreateGenerator_AppSRT_SetScale(0x88, *vl).cast(),
        0x466 => ret = h.efLib_CreateGenerator_AppSRT_SetScale(0x89, *vl).cast(),
        0x467 => ret = h.efLib_CreateGenerator_AppSRT_SetScale(0x8A, *vl).cast(),
        0x468 => attach_scale_f!(0x2C),
        0x469 => attach_scale_f!(0x2E),
        0x46A => appsrt_rot_from_gobj!(0xAC),
        0x46B => attach_scale_f!(0x2B),
        0x46C => appsrt_rot_from_gobj!(0x9E),
        0x46D => attach_scale_f!(0x2D),
        0x46E => ret = h.efLib_CreateGenerator_AppSRT_SetScale(0xAE, *vl).cast(),
        0x46F => ret = h.efLib_CreateGenerator_AppSRT_SetScale(0xA0, *vl).cast(),
        0x470 => ret = h.efLib_CreateGenerator_AppSRT_SetScale(0x21B, *vl).cast(),
        0x471 => ret = h.efLib_CreateGenerator_AppSRT_SetScale(0x220, *vl).cast(),
        0x472 => ret = h.efLib_CreateGenerator_AppSRT_SetScale(0x131, *vl).cast(),
        0x473 => efac!(0, 0x82),
        0x477 => gen!(0x7918),
        0x474 => gen!(0xF7),
        0x475 => gen!(0xFC),
        0x476 => gen!(0xFF),
        _ => {}
    }
    drain_anim_queue(h);
    ret
}

/// efAsync_LoadAsync.
pub unsafe fn efAsync_LoadAsync<H: EfHost + ?Sized>(h: &mut H, index: i32) {
    melee_core::ct!("efAsync_LoadAsync");
    efAsync_MexInit(h);
    if index >= EF_BANK_MAX as i32 || index < 0 {
        return;
    }
    let file = h.ef_async().dat_entries[index as usize].ef_DAT_file;
    if file.is_null() {
        return;
    }
    h.lbDvd_800178E8(3, file, 4, 4, 0, 1, 4, 4, index);
}

/// efAsync_OnLoad.
pub unsafe fn efAsync_OnLoad<H: EfHost + ?Sized>(h: &mut H, archive: *mut c_void, data: *mut u8, length: u32, index: i32) {
    melee_core::ct!("efAsync_OnLoad");
    h.lbArchive_InitializeDAT(archive, data, length);
    let name = h.ef_async().dat_entries[index as usize].effDataTable_name;
    let result = h.HSD_ArchiveGetPublicAddress(archive, name) as *mut EF_DAT_Entry;
    efAsync_MexNoteArchive(h, index, archive);
    if !(*result).ef_DAT_file.is_null() || !(*result).effDataTable_name.is_null() {
        h.psInitDataBankLocate((*result).ef_DAT_file.cast(), (*result).effDataTable_name.cast(), null_mut());
    }
}

/// efAsync_LoadSync.
pub unsafe fn efAsync_LoadSync<H: EfHost + ?Sized>(h: &mut H, idx: i32) {
    melee_core::ct!("efAsync_LoadSync");
    efAsync_MexInit(h);
    if idx >= EF_BANK_MAX as i32 || idx < 0 {
        return;
    }
    let lookup = h.ef_async().dat_entries[idx as usize];
    if lookup.ef_DAT_file.is_null() || !lookup.data.is_null() {
        return;
    }
    let mut ar: *mut c_void = null_mut();
    let mut spc: *mut EF_DAT_Entry = null_mut();
    let chk = h.lbArchive_80017040(&mut ar, lookup.ef_DAT_file, &mut spc, lookup.effDataTable_name);
    efAsync_MexNoteArchive(h, idx, ar);
    if !(*spc).ef_DAT_file.is_null() || !(*spc).effDataTable_name.is_null() {
        if chk {
            h.psInitDataBankLoad(idx, (*spc).ef_DAT_file.cast(), (*spc).effDataTable_name.cast(), core::ptr::null(), core::ptr::null());
        } else {
            h.psInitDataBank(idx, (*spc).ef_DAT_file.cast(), (*spc).effDataTable_name.cast(), null_mut(), null_mut());
        }
    }
    h.ef_async().dat_entries[idx as usize].data = (&mut (*spc).data as *mut *mut c_void).cast();
}

/// efAsync_QueueProcessDeferred.
pub unsafe fn efAsync_QueueProcessDeferred<H: EfHost + ?Sized>(h: &mut H, gobj: *mut HSD_GObj, q: *mut EF_QueuedEffect) {
    melee_core::ct!("efAsync_QueueProcessDeferred");
    let kind = (*q).spawn_kind as u32;
    let id = (*q).gfx_id;
    let j = (*q).jobj;
    let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let params: *mut Vec3 = &mut (*q).params;
    let e1: *mut f32 = &mut (*q).extra1;
    let e2: *mut f32 = &mut (*q).extra2;
    match kind {
        EF_SPAWN_ATTACH => { h.efSync_Spawn(id, gobj, &[VaArg::JObj(j)]); }
        EF_SPAWN_POS => {
            h.lb_8000B1CC(j, core::ptr::null(), &mut pos);
            h.efSync_Spawn(id, gobj, &[VaArg::Vec3(&mut pos)]);
        }
        EF_SPAWN_POS_OFFSET => {
            h.lb_8000B1CC(j, params, &mut pos);
            h.efSync_Spawn(id, gobj, &[VaArg::Vec3(&mut pos)]);
        }
        EF_SPAWN_ATTACH_PARAM => { h.efSync_Spawn(id, gobj, &[VaArg::JObj(j), VaArg::Vec3(params)]); }
        EF_SPAWN_POS_PARAM => {
            h.lb_8000B1CC(j, core::ptr::null(), &mut pos);
            h.efSync_Spawn(id, gobj, &[VaArg::Vec3(&mut pos), VaArg::Vec3(params)]);
        }
        EF_SPAWN_POS_OFFSET_PARAM => {
            h.lb_8000B1CC(j, params, &mut pos);
            h.efSync_Spawn(id, gobj, &[VaArg::Vec3(&mut pos), VaArg::F32(e1)]);
        }
        EF_SPAWN_POS_OFFSET_PARAM2 => {
            h.lb_8000B1CC(j, params, &mut pos);
            h.efSync_Spawn(id, gobj, &[VaArg::Vec3(&mut pos), VaArg::F32(e1), VaArg::F32(e2)]);
        }
        EF_SPAWN_ATTACH_OFFSET => { h.efSync_Spawn(id, gobj, &[VaArg::JObj(j), VaArg::Vec3(params)]); }
        EF_SPAWN_CAMERA_SHAKE => {
            h.lb_8000B1CC(j, params, &mut pos);
            h.Camera_RequestQuake(id as u32, &mut pos);
        }
        EF_SPAWN_MEX => efAsync_MexProcessDeferred(h, gobj, q),
        _ => {
            h.OSReport(&format!("[EfASync] unknown type {kind}\n"));
            panic!("__assert(efasync.c, 0x7C, 0)");
        }
    }
    let ad: *mut HSD_ObjAllocData = &mut h.ef_async().alloc_data;
    h.HSD_ObjFree(ad, q.cast());
}

/// efAsync_QueueFlush.
pub unsafe fn efAsync_QueueFlush<H: EfHost + ?Sized>(h: &mut H, gobj: *mut HSD_GObj, head: *mut EF_QueuedEffect) {
    melee_core::ct!("efAsync_QueueFlush");
    let mut cur = (*head).next;
    while !cur.is_null() {
        let next = (*cur).next;
        efAsync_QueueProcessDeferred(h, gobj, cur);
        cur = next;
    }
    (*head).next = null_mut();
}

/// efAsync_QueueClear.
pub unsafe fn efAsync_QueueClear<H: EfHost + ?Sized>(h: &mut H, head: *mut EF_QueuedEffect) {
    melee_core::ct!("efAsync_QueueClear");
    let mut cur = (*head).next;
    while !cur.is_null() {
        let next = (*cur).next;
        let ad: *mut HSD_ObjAllocData = &mut h.ef_async().alloc_data;
        h.HSD_ObjFree(ad, cur.cast());
        cur = next;
    }
    (*head).next = null_mut();
}

unsafe fn enqueue_or_run<H: EfHost + ?Sized>(h: &mut H, gobj: *mut HSD_GObj, head: *mut EF_QueuedEffect, q: *mut EF_QueuedEffect) {
    let p = h.HSD_GObj_CurrentInvokedProc();
    if !p.is_null() && (*p).s_link < 9 {
        (*q).next = (*head).next;
        (*head).next = q;
        return;
    }
    efAsync_QueueProcessDeferred(h, gobj, q);
}

/// efAsync_Spawn (variadic tail as `vl`).
pub unsafe fn efAsync_Spawn<H: EfHost + ?Sized>(h: &mut H, gobj: *mut HSD_GObj, head: *mut EF_QueuedEffect, spawn_kind: u32, gfx_id: u32, jobj: *mut HSD_JObj, mut vl: VaList<'_>) {
    melee_core::ct!("efAsync_Spawn");
    let ad: *mut HSD_ObjAllocData = &mut h.ef_async().alloc_data;
    let q = h.HSD_ObjAlloc(ad) as *mut EF_QueuedEffect;
    (*q).spawn_kind = spawn_kind as u8;
    (*q).gfx_id = gfx_id as i32;
    (*q).jobj = jobj;
    match spawn_kind {
        EF_SPAWN_ATTACH | EF_SPAWN_POS => {}
        EF_SPAWN_POS_OFFSET | EF_SPAWN_ATTACH_OFFSET | EF_SPAWN_CAMERA_SHAKE => (*q).params = *vl.vec3(),
        EF_SPAWN_ATTACH_PARAM | EF_SPAWN_POS_PARAM => (*q).params.x = *vl.f32p(),
        EF_SPAWN_POS_OFFSET_PARAM => {
            let v = vl.vec3();
            let e2 = vl.f32p();
            (*q).params = *v;
            (*q).extra1 = *e2;
        }
        EF_SPAWN_POS_OFFSET_PARAM2 => {
            let v = vl.vec3();
            let e1 = vl.f32p();
            let e2 = vl.f32p();
            (*q).params = *v;
            (*q).extra1 = *e1;
            (*q).extra2 = *e2;
        }
        _ => {
            h.OSReport(&format!("[EfASync] unknown type {spawn_kind}\n"));
            panic!("__assert(efasync.c, 0xF6, 0)");
        }
    }
    enqueue_or_run(h, gobj, head, q);
}

/// efAsync_QueueInit.
pub unsafe fn efAsync_QueueInit<H: EfHost + ?Sized>(h: &mut H) {
    melee_core::ct!("efAsync_QueueInit");
    let ad: *mut HSD_ObjAllocData = &mut h.ef_async().alloc_data;
    // sizeof(EF_QueuedEffect) / sizeof(EF_QueuedEffect*) of the native build.
    h.HSD_ObjAllocInit(ad, core::mem::size_of::<EF_QueuedEffect>() as u32, core::mem::size_of::<*mut EF_QueuedEffect>() as u32);
}

// ---- m-ex ----

unsafe fn cstr(p: *const c_char) -> String {
    if p.is_null() { "(null)".into() } else { core::ffi::CStr::from_ptr(p).to_string_lossy().into_owned() }
}

/// efAsync_MexInit.
pub unsafe fn efAsync_MexInit<H: EfHost + ?Sized>(h: &mut H) {
    melee_core::ct!("efAsync_MexInit");
    if h.ef_async().mex_ready {
        return;
    }
    h.ef_async().mex_ready = true;
    let mut n = h.Mex_EffectCount();
    if n > EF_BANK_MAX as i32 {
        h.OSReport(&format!("mexeffect: {n} effect banks, the particle system holds {EF_BANK_MAX} - the rest are left out\n"));
        n = EF_BANK_MAX as i32;
    }
    for i in 0..n.max(0) {
        let file = h.Mex_EffectString(i, 0);
        let sym = h.Mex_EffectString(i, 1);
        if file.is_null() || sym.is_null() {
            continue;
        }
        let e = &mut h.ef_async().dat_entries[i as usize];
        e.ef_DAT_file = file;
        e.effDataTable_name = sym;
    }
    if n > 0 {
        h.OSReport(&format!("mexeffect: {n} effect banks from MxDt.dat\n"));
    }
}

/// efAsync_MexNoteArchive (static).
pub unsafe fn efAsync_MexNoteArchive<H: EfHost + ?Sized>(h: &mut H, bank: i32, archive: *mut c_void) {
    melee_core::ct!("efAsync_MexNoteArchive");
    if bank < 0 || bank >= EF_BANK_MAX as i32 || archive.is_null() {
        return;
    }
    let bhv = h.HSD_ArchiveGetPublicAddress(archive, c"effBehaviorTable".as_ptr()) as *mut EfMexBehavior;
    if bhv != h.ef_async().mex_bhv[bank as usize] && !bhv.is_null() {
        let f = cstr(h.ef_async().dat_entries[bank as usize].ef_DAT_file);
        h.OSReport(&format!(
            "mexeffect: bank {bank} ({f}): {} model(s), {} particle generator(s) with a behaviour\n",
            (*bhv).mdl_num, (*bhv).ptcl_num
        ));
    }
    h.ef_async().mex_bhv[bank as usize] = bhv;
}

/// efAsync_MexMiss (static).
pub unsafe fn efAsync_MexMiss<H: EfHost + ?Sized>(h: &mut H, kind: i32, gfx_id: i32, why: &str) {
    melee_core::ct!("efAsync_MexMiss");
    let key = mex_miss_key(kind, gfx_id);
    let st = h.ef_async();
    for i in 0..st.miss_nseen as usize {
        if st.miss_seen[i] == key {
            return;
        }
    }
    if (st.miss_nseen as usize) < st.miss_seen.len() {
        st.miss_seen[st.miss_nseen as usize] = key;
        st.miss_nseen += 1;
    }
    if gfx_id >= 100000 {
        h.OSReport(&format!("mexeffect: fighter kind {kind} effect {} {why}\n", gfx_id - 100000));
        return;
    }
    h.OSReport(&format!("mexeffect: fighter kind {kind} effect {gfx_id} not drawn - {why}\n"));
}

unsafe fn ftkb_is_kirby<H: EfHost + ?Sized>(h: &mut H, k: i32) -> bool {
    k == Ft_Kind_Kirby as i32 || (k >= Ft_Kind_Mex0 as i32 && k < Ft_Kind_Max as i32 && h.ftKb_MexKirbyLike(k) != 0)
}

/// efAsync_MexResolve: the behaviour type (>= 0) or -1.
pub unsafe fn efAsync_MexResolve<H: EfHost + ?Sized>(
    h: &mut H, mut gobj: *mut HSD_GObj, gfx_id: i32, final_id: &mut i32, is_ptcl: &mut i32, owner: &mut *mut HSD_GObj,
) -> i32 {
    melee_core::ct!("efAsync_MexResolve");
    if gobj.is_null() || !EF_MEX_IS_CUSTOM(gfx_id) {
        return -1;
    }
    if (*gobj).classifier == HSD_GOBJ_CLASS_ITEM {
        gobj = (*((*gobj).user_data as *mut Item)).mex_original_owner;
        if gobj.is_null() || (*gobj).classifier != HSD_GOBJ_CLASS_FIGHTER {
            return -1;
        }
    }
    if (*gobj).classifier != HSD_GOBJ_CLASS_FIGHTER {
        return -1;
    }
    let fp = (*gobj).user_data as *mut Fighter;
    let kind = (*fp).kind as i32;
    let copy: bool;
    let int_id: i32;
    if ftkb_is_kirby(h, kind)
        && (kind == Ft_Kind_Kirby as i32 || h.ftData_UnkBytePerCharacter(kind) == h.ftData_UnkBytePerCharacter(Ft_Kind_Kirby as i32))
    {
        if gfx_id >= EF_MEX_CPMDL_START {
            return -1;
        }
        copy = true;
        *is_ptcl = (gfx_id >= EF_MEX_PTCL_START) as i32;
        int_id = gfx_id - if *is_ptcl != 0 { EF_MEX_PTCL_START } else { EF_MEX_MDL_START };
    } else {
        copy = gfx_id >= EF_MEX_CPMDL_START;
        *is_ptcl = ((gfx_id >= EF_MEX_PTCL_START && gfx_id < EF_MEX_CPMDL_START) || gfx_id >= EF_MEX_CPPTCL_START) as i32;
        int_id = gfx_id % 1000;
    }
    let src_kind = if copy { (*fp).u.kb.hat.kind as i32 } else { kind };
    if src_kind < 0 || src_kind >= Ft_Kind_Max as i32 {
        return -1;
    }
    let bank = h.ftData_UnkBytePerCharacter(src_kind) as i32;
    if bank < 0 || bank >= EF_BANK_MAX as i32 || bank == 0xFF {
        efAsync_MexMiss(h, kind, gfx_id, "no effect bank");
        return -1;
    }
    let bhv = h.ef_async().mex_bhv[bank as usize];
    if bhv.is_null() || h.ef_async().dat_entries[bank as usize].data.is_null() {
        if !ftkb_is_kirby(h, kind) {
            efAsync_MexMiss(h, kind, gfx_id, "its effect bank has no effBehaviorTable (not loaded?)");
        }
        return -1;
    }
    if int_id >= if *is_ptcl != 0 { (*bhv).ptcl_num } else { (*bhv).mdl_num } {
        efAsync_MexMiss(h, kind, gfx_id, "past the bank's behaviour table");
        return -1;
    }
    let ty = *(if *is_ptcl != 0 { (*bhv).ptcl_type } else { (*bhv).mdl_type }).offset(int_id as isize) as i32;
    *final_id = bank * 1000 + int_id;
    *owner = gobj;
    let f = cstr(h.ef_async().dat_entries[bank as usize].ef_DAT_file);
    // C formats into a 128-byte static buffer with sprintf; the text never reaches that length.
    let buf = format!(
        "drawn as bank {bank} ({f}) {} {int_id}, behaviour {ty}, in motion {}",
        if *is_ptcl != 0 { "generator" } else { "model" },
        (*fp).motion_id as i32
    );
    efAsync_MexMiss(h, kind, gfx_id + 100000, &buf);
    ty
}

/// efAsync_MexSpawn.
pub unsafe fn efAsync_MexSpawn<H: EfHost + ?Sized>(
    h: &mut H, gobj: *mut HSD_GObj, head: *mut EF_QueuedEffect, gfx_id: i32, jobj: *mut HSD_JObj, offset: *const Vec3, facing: f32,
    orientation: f32,
) {
    melee_core::ct!("efAsync_MexSpawn");
    let (mut fid, mut ptcl, mut owner) = (0, 0, null_mut());
    if efAsync_MexResolve(h, gobj, gfx_id, &mut fid, &mut ptcl, &mut owner) < 0 {
        return;
    }
    let ad: *mut HSD_ObjAllocData = &mut h.ef_async().alloc_data;
    let q = h.HSD_ObjAlloc(ad) as *mut EF_QueuedEffect;
    if q.is_null() {
        return;
    }
    (*q).spawn_kind = EF_SPAWN_MEX as u8;
    (*q).gfx_id = gfx_id;
    (*q).jobj = jobj;
    (*q).params = *offset;
    (*q).extra1 = facing;
    (*q).extra2 = orientation;
    enqueue_or_run(h, gobj, head, q);
}

/// efAsync_MexProcessDeferred (static).
pub unsafe fn efAsync_MexProcessDeferred<H: EfHost + ?Sized>(h: &mut H, gobj: *mut HSD_GObj, q: *mut EF_QueuedEffect) {
    melee_core::ct!("efAsync_MexProcessDeferred");
    let id = (*q).gfx_id;
    let (mut fid, mut ptcl, mut owner) = (0, 0, null_mut());
    let ty = efAsync_MexResolve(h, gobj, id, &mut fid, &mut ptcl, &mut owner);
    if ty < 0 {
        return;
    }
    let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let e1: *mut f32 = &mut (*q).extra1;
    let e2: *mut f32 = &mut (*q).extra2;
    if ptcl == 0 {
        match ty {
            0..=4 => {
                h.lb_8000B1CC((*q).jobj, &(*q).params, &mut pos);
                h.efSync_Spawn(id, gobj, &[VaArg::Vec3(&mut pos), VaArg::F32(e1), VaArg::F32(e2)]);
            }
            5 | 6 => { h.efSync_Spawn(id, gobj, &[VaArg::JObj((*q).jobj)]); }
            _ => {}
        }
        return;
    }
    match ty {
        0 => {
            h.lb_8000B1CC((*q).jobj, &(*q).params, &mut pos);
            h.efSync_Spawn(id, gobj, &[VaArg::Vec3(&mut pos), VaArg::F32(e1), VaArg::F32(e2)]);
        }
        3 => {
            h.lb_8000B1CC((*q).jobj, &(*q).params, &mut pos);
            h.efSync_Spawn(id, gobj, &[VaArg::Vec3(&mut pos), VaArg::F32(e1)]);
        }
        5 | 7 => { h.efSync_Spawn(id, gobj, &[VaArg::JObj((*q).jobj)]); }
        6 => { h.efSync_Spawn(id, gobj, &[VaArg::JObj((*q).jobj), VaArg::F32(e1)]); }
        _ => {}
    }
}

// ---- the native host ----

static mut NATIVE_EF_DATA: Option<EfData> = None;
static mut NATIVE_EF_ASYNC: Option<EfAsyncState> = None;

/// The `EfHost` for the real game: efdata.c's / efasync.c's globals live in statics, and the
/// services that have native ports in the crates below are wired to them (the rest keep the
/// trait defaults, i.e. the `ext` hooks of this TU).
pub struct NativeEfHost;

unsafe impl EfHost for NativeEfHost {
    fn ef_data(&mut self) -> &mut EfData {
        unsafe { (*(&raw mut NATIVE_EF_DATA)).get_or_insert_with(Default::default) }
    }
    fn ef_async(&mut self) -> &mut EfAsyncState {
        unsafe { (*(&raw mut NATIVE_EF_ASYNC)).get_or_insert_with(Default::default) }
    }
    unsafe fn HSD_Randf(&mut self) -> f32 { melee_runtime::tu::random::HSD_Randf_global() }
    unsafe fn HSD_Randi(&mut self, max: i32) -> i32 { melee_runtime::tu::random::HSD_Randi_global(max) }
    unsafe fn HSD_JObjAnimAll(&mut self, j: *mut HSD_JObj) { melee_runtime::tu::jobj_ptr::HSD_JObjAnimAll(j) }
    unsafe fn HSD_JObjSetMtxDirtySub(&mut self, j: *mut HSD_JObj) { melee_runtime::tu::jobj_ptr::HSD_JObjSetMtxDirtySub(j) }
    unsafe fn lb_8000B1CC(&mut self, j: *mut HSD_JObj, o: *const Vec3, out: *mut Vec3) {
        melee_runtime::tu::lb_00B0::lb_8000B1CC(j, o as *mut Vec3, out)
    }
    unsafe fn lbDvd_800178E8(&mut self, a: i32, n: *const c_char, b: i32, c: i32, d: i32, e: i32, f: i32, g: u8, idx: i32) {
        melee_runtime::tu::lbdvd::lbDvd_800178E8(a, n, b, c, d, e, f, g, idx)
    }
    unsafe fn HSD_ArchiveGetPublicAddress(&mut self, ar: *mut c_void, sym: *const c_char) -> *mut c_void {
        melee_runtime::tu::archive::HSD_ArchiveGetPublicAddress(ar.cast(), sym)
    }
    unsafe fn lbArchive_80017040(&mut self, dst: *mut *mut c_void, file: *const c_char, out: *mut *mut EF_DAT_Entry, sym: *const c_char) -> bool {
        let rest: [*mut c_void; 3] = [out.cast(), sym as *mut c_void, null_mut()];
        melee_runtime::tu::lbarchive::lbArchive_80017040(dst.cast(), file, &rest)
    }
}

/// `efAsync_LoadAsync(index)` on the game's globals.
pub unsafe fn efAsync_LoadAsync_native(index: i32) {
    melee_core::ct!("efAsync_LoadAsync_native");
    efAsync_LoadAsync(&mut NativeEfHost, index)
}
/// `efAsync_LoadSync(index)` on the game's globals.
pub unsafe fn efAsync_LoadSync_native(index: i32) {
    melee_core::ct!("efAsync_LoadSync_native");
    efAsync_LoadSync(&mut NativeEfHost, index)
}
/// `efAsync_Spawn(gobj, head, spawn_kind, gfx_id, jobj, ...)` on the game's globals; `tail` is the
/// variadic pointers in `va_arg` order.
pub unsafe fn efAsync_Spawn_native(
    gobj: *mut HSD_GObj,
    head: *mut EF_QueuedEffect,
    spawn_kind: u32,
    gfx_id: u32,
    jobj: *mut HSD_JObj,
    tail: &[*mut c_void],
) {
    melee_core::ct!("efAsync_Spawn_native");
    let args: std::vec::Vec<VaArg> = tail.iter().map(|&p| VaArg::Ptr(p)).collect();
    efAsync_Spawn(&mut NativeEfHost, gobj, head, spawn_kind, gfx_id, jobj, VaList::new(&args))
}
